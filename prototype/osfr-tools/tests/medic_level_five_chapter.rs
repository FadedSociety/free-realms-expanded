//! T278: the Medic level-five chapter — Palatable Pills (3090004) and
//! Achieve Level 5 (3090006).
//!
//! The chapter is content-only. These tests drive the production in-world router
//! (`Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `QuestEngine`), the same
//! path the gateway calls for every in-world request. The farm-ground visit is the
//! production location event (`gateway.rs` -> `Gameplay::on_location`), a server
//! position check; no packet is added or read for it.
//!
//! Client request/reply layouts used here (pinned in the native map; the live IDA
//! broker was not answering on 2026-10-20, recorded in notes/Autopilot/T278.md):
//! - 26/8 NPC interact: constructor `008b6bc0`; u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; u64 target + i32 choice.
//! - 26/6 dialogue response: constructor `009f5890`; i32 row (301 is the quest row).
//! - 49/2 quest reply: constructor `00a86400`; u32 quest + u8 accept.
//! - 49/14 end reply: constructor `00a864c0`; u32 quest.
//! 49/1 offers are read `00c7bb60` (quest id at 6..10, intro at 10..14); 49/8
//! QuestAdd is read `00c7bd10`; 49/13 end screens are read by `00c7bbd0` /
//! `00c7b990` (speaker 6..14, quest 14..18, end text 18..22).
//!
//! The literal retail gather is abstracted `(R)` as a `visit_position` at
//! Sunflower's actual Queensfields farm row; harvesting is Chef-only on this
//! server, so no `gather_resource` goal or invented item is used. The shared
//! fixture and packet helpers live in `tests/support/npc_quest_chain.rs`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

#[path = "support/medic_entry.rs"]
mod medic_entry;

use medic_entry::*;
use npc_quest_chain::*;
use osfr_tools::{
    character_store::CharacterState,
    game_store,
    gameplay::{Gameplay, GameplayState, Outcome},
    quest_engine::QuestStatus,
};
use serde_json::Value;

/// `visit_position` objectives in this content package use the open-world scene.
const OPEN_WORLD: u32 = 0;

fn npc_character(npc_id: u64) -> CharacterState {
    npc_character_for(CHARACTER, npc_id)
}

fn npc_character_for(guid: u64, npc_id: u64) -> CharacterState {
    character_at_for(guid, SANCTUARY, content_position("sanctuary.json", npc_id))
}

/// Wire dword 1 of a 49/1 offer: the quest-start intro locale id.
fn offer_intro_text_id(outcome: &Outcome, quest_id: u32) -> u32 {
    let offer = outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(quest_id)))
        .expect("49/1 offer");
    u32::from_le_bytes(offer[10..14].try_into().unwrap())
}

fn medic_record(quest_id: u32) -> Value {
    let document = read_json(&content_dir().join("quests/medic.json"));
    document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(quest_id)))
        .unwrap_or_else(|| panic!("quest {quest_id} is authored"))
        .clone()
}

fn learn_medic(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    let active_before = active_job(game);
    let menu = interact(game, character, NAIA, *now);
    let learn = menu_option(&menu, LEARN_LABEL);
    *now += 10;
    let learned = send(game, character, &npc_select(NAIA, learn), *now);
    assert!(learned.changed, "Naia's Learn choice must commit");
    assert!(
        game.state()
            .profiles
            .as_ref()
            .unwrap()
            .unlocked
            .contains(&MEDIC),
        "Naia's trainer row must unlock Medic"
    );
    assert_eq!(
        active_job(game),
        active_before,
        "learning a job must not switch to it"
    );
    *now += 10;
    let switched = send(game, character, &select_job(MEDIC), *now);
    assert!(
        switched.changed,
        "Medic selection must activate; its starter outfit has Chest and Legs"
    );
    assert_eq!(active_job(game), MEDIC, "the job gate needs Medic active");
}

fn complete_exam(game: &mut Gameplay, naia: &CharacterState, now: &mut u64) {
    *now += 10;
    offer(game, naia, NAIA, EXAM, *now);
    *now += 10;
    accept(game, naia, EXAM, *now);
    *now += 10;
    let talked = talk(game, naia, NAIA, *now);
    assert!(talked.changed, "the exam talk must complete the objective");
    assert_eq!(pending_turnin(game), Some(EXAM));
    *now += 10;
    release_end_screen(game, naia, EXAM, NAIA, EXAM_END_TEXT, *now);
    *now += 10;
    finish_quest(game, naia, EXAM, MEDIC, EXAM_COINS, EXAM_XP, *now);
}

fn complete_house_call(
    game: &mut Gameplay,
    naia: &CharacterState,
    greyli: &CharacterState,
    now: &mut u64,
) {
    *now += 10;
    offer(game, naia, NAIA, HOUSE_CALL, *now);
    *now += 10;
    accept(game, naia, HOUSE_CALL, *now);
    *now += 10;
    let talked = talk(game, greyli, GREYLI, *now);
    assert!(
        talked.changed,
        "the House Call talk must complete the objective"
    );
    assert_eq!(pending_turnin(game), Some(HOUSE_CALL));
    *now += 10;
    release_end_screen(game, greyli, HOUSE_CALL, GREYLI, HOUSE_END_TEXT, *now);
    *now += 10;
    finish_quest(game, greyli, HOUSE_CALL, MEDIC, HOUSE_COINS, HOUSE_XP, *now);
}

/// Learn Medic, complete the exam and House Call, and return the three actor
/// states plus the game. `guid` owns the quest and reward ledger.
fn medic_ready(
    fixture: &Fixture,
    guid: u64,
    now: &mut u64,
) -> (Gameplay, CharacterState, CharacterState, CharacterState) {
    let naia = npc_character_for(guid, NAIA);
    let greyli = npc_character_for(guid, GREYLI);
    let gordon = npc_character_for(guid, GORDON);
    let mut game = fixture.game_as(guid, None);
    learn_medic(&mut game, &naia, now);
    complete_exam(&mut game, &naia, now);
    complete_house_call(&mut game, &naia, &greyli, now);
    (game, naia, greyli, gordon)
}

/// The production location event that a `visit_position` objective consumes.
fn visit(game: &mut Gameplay, guid: u64, position: [f32; 3], now: u64) -> Outcome {
    game.on_location(&character_at_for(guid, OPEN_WORLD, position), now)
        .expect("location event")
}

fn assert_visit_changes_nothing(
    game: &mut Gameplay,
    guid: u64,
    position: [f32; 3],
    now: u64,
    label: &str,
) {
    let before_state = game.state();
    let before_bytes = saved(game);
    let outcome = visit(game, guid, position, now);
    assert!(!outcome.changed, "{label} changed gameplay");
    assert_eq!(
        game.state(),
        before_state,
        "{label} changed the full GameplayState"
    );
    assert_eq!(saved(game), before_bytes, "{label} changed the save bytes");
}

fn sunflower_position() -> [f32; 3] {
    content_position("sanctuary.json", SUNFLOWER)
}

fn accept_pills(game: &mut Gameplay, gordon: &CharacterState, now: &mut u64) -> Outcome {
    *now += 10;
    let offered = offer(game, gordon, GORDON, PILLS, *now);
    assert_eq!(
        offer_intro_text_id(&offered, PILLS),
        PILL_INTRO_TEXT,
        "the offer carries the recovered intro line 77436"
    );
    *now += 10;
    accept(game, gordon, PILLS, *now)
}

fn complete_pills(
    game: &mut Gameplay,
    gordon: &CharacterState,
    naia: &CharacterState,
    guid: u64,
    now: &mut u64,
) {
    accept_pills(game, gordon, now);
    *now += 10;
    let visited = visit(game, guid, sunflower_position(), *now);
    assert!(
        visited.changed,
        "the Sunflower farm-ground visit must complete objective 1"
    );
    assert_eq!(counts(game, PILLS), vec![1, 0]);
    *now += 10;
    let talked = talk(game, naia, NAIA, *now);
    assert!(
        talked.changed,
        "the Naia hand-off must complete objective 2"
    );
    assert_eq!(counts(game, PILLS), vec![1, 1]);
    assert_eq!(pending_turnin(game), Some(PILLS));
    *now += 10;
    release_end_screen(game, naia, PILLS, NAIA, PILL_END_TEXT, *now);
    *now += 10;
    finish_quest(game, naia, PILLS, MEDIC, PILL_COINS, PILL_XP, *now);
}

fn complete_vitamins(
    game: &mut Gameplay,
    gordon: &CharacterState,
    greyli: &CharacterState,
    now: &mut u64,
) {
    *now += 10;
    offer(game, gordon, GORDON, VITAMINS, *now);
    *now += 10;
    accept(game, gordon, VITAMINS, *now);
    *now += 10;
    let talked = talk(game, greyli, GREYLI, *now);
    assert!(
        talked.changed,
        "the vitamins talk must complete the objective"
    );
    *now += 10;
    // No completion line is recovered for 3090005; keep the documented 0 gap.
    release_end_screen(game, greyli, VITAMINS, GREYLI, 0, *now);
    *now += 10;
    finish_quest(
        game,
        greyli,
        VITAMINS,
        MEDIC,
        VITAMIN_COINS,
        VITAMIN_XP,
        *now,
    );
}

/// Replace the saved state through the same production bind a reconnect uses.
fn patch_state(
    fixture: &Fixture,
    guid: u64,
    game: &mut Gameplay,
    patch: impl FnOnce(&mut GameplayState),
) {
    let mut state = game.state();
    patch(&mut state);
    *game = fixture.game_as(guid, Some(state));
}

fn set_medic_xp(fixture: &Fixture, guid: u64, game: &mut Gameplay, xp: u32) {
    patch_state(fixture, guid, game, |state| {
        state
            .profiles
            .as_mut()
            .expect("the fixture has profile state")
            .experience
            .insert(MEDIC, xp);
    });
    assert_eq!(medic_xp(game), xp);
}

fn medic_xp(game: &Gameplay) -> u32 {
    profile_xp(game, MEDIC)
}

/// Finish the whole chapter through Achieve Level 5 at rank 5.
fn finish_chapter(
    fixture: &Fixture,
    guid: u64,
    now: &mut u64,
) -> (Gameplay, CharacterState, CharacterState, CharacterState) {
    let (mut game, naia, greyli, gordon) = medic_ready(fixture, guid, now);
    complete_pills(&mut game, &gordon, &naia, guid, now);
    complete_vitamins(&mut game, &gordon, &greyli, now);
    set_medic_xp(fixture, guid, &mut game, 400);
    assert_eq!(
        game.state()
            .profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&MEDIC)
            .copied(),
        Some(400),
        "rank 5 is 400 XP on the local one-rank-per-100-XP curve"
    );
    *now += 10;
    offer(&mut game, &gordon, GORDON, LEVEL_FIVE, *now);
    *now += 10;
    accept(&mut game, &gordon, LEVEL_FIVE, *now);
    *now += 10;
    let talked = talk(&mut game, &gordon, GORDON, *now);
    assert!(
        talked.changed,
        "the report to Gordon must complete Achieve Level 5"
    );
    assert_eq!(pending_turnin(&game), Some(LEVEL_FIVE));
    *now += 10;
    release_end_screen(&mut game, &gordon, LEVEL_FIVE, GORDON, LEVEL_END_TEXT, *now);
    *now += 10;
    finish_quest(
        &mut game,
        &gordon,
        LEVEL_FIVE,
        MEDIC,
        LEVEL_COINS,
        LEVEL_XP,
        *now,
    );
    (game, naia, greyli, gordon)
}

#[test]
fn gordon_offers_palatable_pills_after_house_call_only_to_medics() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 10_000;
    let naia = npc_character(NAIA);
    let gordon = npc_character(GORDON);

    // A character who has not learned Medic sees no chapter offer.
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "unlearned character at Gordon",
    );
    assert_eq!(
        game.state(),
        before_state,
        "the unlearned refusal changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "the unlearned refusal changed the save"
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&PILLS)
    );

    // A Medic who has only passed the exam still misses the House Call prerequisite.
    learn_medic(&mut game, &naia, &mut now);
    complete_exam(&mut game, &naia, &mut now);
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "exam-only Medic at Gordon",
    );
    assert_eq!(
        game.state(),
        before_state,
        "the prereq refusal changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "the prereq refusal changed the save"
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&PILLS),
        "Palatable Pills waits for House Call"
    );

    // House Call unlocks exactly one 49/1, for 3090004.
    complete_house_call(&mut game, &naia, &npc_character(GREYLI), &mut now);
    now += 10;
    let offered = offer(&mut game, &gordon, GORDON, PILLS, now);
    assert_eq!(offer_intro_text_id(&offered, PILLS), PILL_INTRO_TEXT);
    assert_eq!(pending_offer(&game), Some(PILLS));

    // A re-talker while the offer is open starts nothing and changes nothing.
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "second Talk with the pills offer open",
    );
    assert_eq!(game.state(), before_state, "the re-talk changed state");
    assert_eq!(saved(&game), before_bytes, "the re-talk changed the save");
    assert_eq!(pending_offer(&game), Some(PILLS));

    // Accepting activates the two authored objectives and pays nothing yet.
    let coins_before = coins(&game);
    let xp_before = medic_xp(&game);
    now += 10;
    accept(&mut game, &gordon, PILLS, now);
    assert_eq!(counts(&game, PILLS), vec![0, 0]);
    assert_eq!(pending_offer(&game), None);
    assert_eq!(coins(&game), coins_before);
    assert_eq!(medic_xp(&game), xp_before);

    // The authored trip is a position check plus the Naia talk; the literal
    // ingredient gather is not faked with an item or a gather_resource goal.
    let record = medic_record(PILLS);
    assert_eq!(record["enabled"].as_bool(), Some(true));
    assert_eq!(record["job"].as_i64(), Some(i64::from(MEDIC)));
    assert_eq!(
        record["prerequisite_quest_ids"],
        serde_json::json!([HOUSE_CALL])
    );
    let objectives = record["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 2);
    assert_eq!(objectives[0]["kind"], "visit_position");
    assert_eq!(objectives[0]["zone_id"].as_u64(), Some(0));
    assert_eq!(objectives[0]["radius"].as_f64(), Some(8.0));
    assert_eq!(objectives[1]["kind"], "talk_to_npc");
    assert_eq!(objectives[1]["npc_id"].as_u64(), Some(NAIA));
    assert!(
        objectives.iter().all(|goal| !matches!(
            goal["kind"].as_str(),
            Some("gather_resource" | "collect_item")
        )),
        "a Medic cannot harvest; the collection stays an explicit visit abstraction"
    );
}

#[test]
fn the_queensfields_visit_and_naia_talk_pay_once() {
    let fixture = Fixture::new();
    let mut now = 20_000;
    let (mut game, naia, _greyli, gordon) = medic_ready(&fixture, CHARACTER, &mut now);
    accept_pills(&mut game, &gordon, &mut now);
    let coins_before = coins(&game);
    let xp_before = medic_xp(&game);
    assert_eq!(counts(&game, PILLS), vec![0, 0]);

    // Outside the 8 m ring changes nothing.
    let farm = sunflower_position();
    let outside = [farm[0] + 8.5, farm[1], farm[2]];
    assert_visit_changes_nothing(
        &mut game,
        CHARACTER,
        outside,
        now + 10,
        "visit outside the farm ground",
    );
    assert_eq!(counts(&game, PILLS), vec![0, 0]);
    assert_eq!(coins(&game), coins_before);
    assert_eq!(medic_xp(&game), xp_before);

    // Inside the ring completes the first objective and survives a reconnect.
    now += 20;
    let visited = visit(&mut game, CHARACTER, farm, now);
    assert!(
        visited.changed,
        "the in-radius visit must advance the quest"
    );
    assert_eq!(counts(&game, PILLS), vec![1, 0]);
    let mid_bytes = saved(&game);

    now += 10;
    assert_visit_changes_nothing(&mut game, CHARACTER, farm, now, "replayed in-radius visit");
    assert_eq!(counts(&game, PILLS), vec![1, 0]);

    let save_path = fixture.path().join("gameplay-save.json");
    let snapshot = game_store::Snapshot {
        character: naia.clone(),
        gameplay: Some(game.state()),
        pets: None,
    };
    game_store::save(&save_path, &snapshot).unwrap();
    let loaded = game_store::load_or_create(&save_path, naia.clone()).unwrap();
    assert_eq!(
        loaded.gameplay.as_ref(),
        snapshot.gameplay.as_ref(),
        "the save envelope changed the visit counter"
    );
    game = fixture.game(loaded.gameplay);
    assert_eq!(
        saved(&game),
        mid_bytes,
        "the mid-chain reconnect changed the save"
    );
    assert_eq!(counts(&game, PILLS), vec![1, 0]);

    // Switch away before the hand-off: the reward still lands on profile 11.
    now += 10;
    let switched = send(&mut game, &naia, &select_job(BRAWLER), now);
    assert!(switched.changed, "Brawler must be selectable");
    assert_eq!(active_job(&game), BRAWLER);
    let brawler_before = profile_xp(&game, BRAWLER);

    now += 10;
    let talked = talk(&mut game, &naia, NAIA, now);
    assert!(talked.changed, "the Naia talk must complete objective 2");
    assert_eq!(counts(&game, PILLS), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(PILLS));
    now += 10;
    release_end_screen(&mut game, &naia, PILLS, NAIA, PILL_END_TEXT, now);
    now += 10;
    finish_quest(&mut game, &naia, PILLS, MEDIC, PILL_COINS, PILL_XP, now);

    assert_eq!(coins(&game), coins_before + PILL_COINS);
    assert_eq!(medic_xp(&game), xp_before + PILL_XP);
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before,
        "the active job must not receive the Medic XP"
    );
    assert_eq!(
        active_job(&game),
        BRAWLER,
        "the turn-in must not switch jobs"
    );

    // Replayed visit/turn-in and a later Talk pay nothing.
    now += 10;
    assert_visit_changes_nothing(&mut game, CHARACTER, farm, now, "visit after completion");
    assert_packet_changes_nothing(
        &mut game,
        &naia,
        &quest_end_reply(PILLS),
        now + 1,
        "replayed Palatable Pills turn-in",
    );
    assert_no_offer(
        &mut game,
        &naia,
        NAIA,
        &mut now,
        "completed Palatable Pills at Naia",
    );
    assert_eq!(coins(&game), coins_before + PILL_COINS);
    assert_eq!(medic_xp(&game), xp_before + PILL_XP);
}

#[test]
fn gordon_offers_achieve_level_five_only_at_rank_five() {
    let fixture = Fixture::new();
    let mut now = 30_000;
    let (mut game, naia, greyli, gordon) = medic_ready(&fixture, CHARACTER, &mut now);
    complete_pills(&mut game, &gordon, &naia, CHARACTER, &mut now);
    complete_vitamins(&mut game, &gordon, &greyli, &mut now);
    assert_eq!(quest_status(&game, PILLS), QuestStatus::Completed);
    assert_eq!(quest_status(&game, VITAMINS), QuestStatus::Completed);

    // Rank 4: the prerequisite is met but the authored level gate refuses.
    set_medic_xp(&fixture, CHARACTER, &mut game, 300);
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "rank 4 Medic at Gordon",
    );
    assert_eq!(
        game.state(),
        before_state,
        "the rank-4 refusal changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "the rank-4 refusal changed the save"
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&LEVEL_FIVE),
        "Achieve Level 5 waits for rank 5"
    );

    // Rank 5: the same Talk now offers exactly one 49/1 for 3090006.
    set_medic_xp(&fixture, CHARACTER, &mut game, 400);
    now += 10;
    let offered = offer(&mut game, &gordon, GORDON, LEVEL_FIVE, now);
    assert_eq!(offer_intro_text_id(&offered, LEVEL_FIVE), LEVEL_INTRO_TEXT);
    let record = medic_record(LEVEL_FIVE);
    assert_eq!(record["enabled"].as_bool(), Some(true));
    assert_eq!(record["job"].as_i64(), Some(i64::from(MEDIC)));
    assert_eq!(record["min_level"].as_u64(), Some(5));
    assert_eq!(
        record["prerequisite_quest_ids"],
        serde_json::json!([PILLS, VITAMINS])
    );

    now += 10;
    accept(&mut game, &gordon, LEVEL_FIVE, now);
    assert_eq!(counts(&game, LEVEL_FIVE), vec![0]);

    // Switch away before the report: the reward still lands on profile 11.
    now += 10;
    let switched = send(&mut game, &naia, &select_job(BRAWLER), now);
    assert!(switched.changed);
    let coins_before = coins(&game);
    let xp_before = medic_xp(&game);
    let brawler_before = profile_xp(&game, BRAWLER);

    now += 10;
    let talked = talk(&mut game, &gordon, GORDON, now);
    assert!(
        talked.changed,
        "the report to Gordon must complete the objective"
    );
    assert_eq!(pending_turnin(&game), Some(LEVEL_FIVE));
    now += 10;
    release_end_screen(&mut game, &gordon, LEVEL_FIVE, GORDON, LEVEL_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &gordon,
        LEVEL_FIVE,
        MEDIC,
        LEVEL_COINS,
        LEVEL_XP,
        now,
    );
    assert_eq!(coins(&game), coins_before + LEVEL_COINS);
    assert_eq!(medic_xp(&game), xp_before + LEVEL_XP);
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before,
        "the active job must not receive the Medic XP"
    );

    // Paid exactly once.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &gordon,
        &quest_end_reply(LEVEL_FIVE),
        now,
        "replayed Achieve Level 5 turn-in",
    );
    assert_eq!(coins(&game), coins_before + LEVEL_COINS);
    assert_eq!(medic_xp(&game), xp_before + LEVEL_XP);
}

#[test]
fn refusals_and_reconnect_change_nothing() {
    let fixture = Fixture::new();
    let mut now = 40_000;
    let naia = npc_character(NAIA);
    let gordon = npc_character(GORDON);
    let greyli = npc_character(GREYLI);
    let mut game = fixture.game(None);
    learn_medic(&mut game, &naia, &mut now);
    complete_exam(&mut game, &naia, &mut now);

    // Missing prerequisite: no 49/1 and no state/save change.
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "Palatable Pills before House Call",
    );
    assert_eq!(
        game.state(),
        before_state,
        "the missing-prereq refusal changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "the missing-prereq refusal changed the save"
    );

    complete_house_call(&mut game, &naia, &greyli, &mut now);
    accept_pills(&mut game, &gordon, &mut now);

    // Wrong NPC: Greyli cannot advance the Naia hand-off.
    let before_state = game.state();
    let before_bytes = saved(&game);
    let wrong = talk(&mut game, &greyli, GREYLI, now + 1);
    assert!(!wrong.changed, "a wrong NPC must not advance the quest");
    assert_eq!(
        game.state(),
        before_state,
        "the wrong-NPC talk changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "the wrong-NPC talk changed the save"
    );
    assert_eq!(counts(&game, PILLS), vec![0, 0]);

    // Forged accept while the quest is already active changes nothing.
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &gordon,
        &quest_reply(PILLS, true),
        now + 2,
        "forged Palatable Pills accept",
    );
    assert_eq!(
        game.state(),
        before_state,
        "the forged accept changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "the forged accept changed the save"
    );

    // Outside the visit radius changes nothing.
    let farm = sunflower_position();
    let outside = [farm[0] + 8.5, farm[1], farm[2]];
    assert_visit_changes_nothing(
        &mut game,
        CHARACTER,
        outside,
        now + 3,
        "outside-radius visit",
    );
    assert_eq!(counts(&game, PILLS), vec![0, 0]);

    // The in-radius visit advances objective 1 and survives a real reconnect.
    now += 10;
    let visited = visit(&mut game, CHARACTER, farm, now);
    assert!(visited.changed);
    assert_eq!(counts(&game, PILLS), vec![1, 0]);
    let mid_state = game.state();
    let mid_bytes = saved(&game);
    let save_path = fixture.path().join("gameplay-save.json");
    let snapshot = game_store::Snapshot {
        character: naia.clone(),
        gameplay: Some(mid_state.clone()),
        pets: None,
    };
    game_store::save(&save_path, &snapshot).unwrap();
    let loaded = game_store::load_or_create(&save_path, naia.clone()).unwrap();
    assert_eq!(
        loaded.gameplay.as_ref(),
        Some(&mid_state),
        "the save envelope changed the chapter state"
    );
    game = fixture.game(loaded.gameplay);
    assert_eq!(
        saved(&game),
        mid_bytes,
        "the reconnect changed the save bytes"
    );
    assert_eq!(counts(&game, PILLS), vec![1, 0]);

    // 49/14 with no end screen is a no-op.
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &naia,
        &quest_end_reply(PILLS),
        now + 1,
        "Palatable Pills 49/14 with no end screen",
    );
    assert_eq!(game.state(), before_state, "the early 49/14 changed state");
    assert_eq!(
        saved(&game),
        before_bytes,
        "the early 49/14 changed the save"
    );

    // The Naia talk completes objective 2 and holds the end screen.
    now += 10;
    let talked = talk(&mut game, &naia, NAIA, now);
    assert!(talked.changed);
    assert_eq!(counts(&game, PILLS), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(PILLS));

    // A duplicate Talk only re-sends the held end screen; the save is unchanged.
    let before_state = game.state();
    let before_bytes = saved(&game);
    let duplicate = talk(&mut game, &naia, NAIA, now + 1);
    assert!(!duplicate.changed, "a duplicate Talk changed gameplay");
    assert_eq!(
        game.state(),
        before_state,
        "the duplicate Talk changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "the duplicate Talk changed the save"
    );
    assert_eq!(pending_turnin(&game), Some(PILLS));
    assert_eq!(counts(&game, PILLS), vec![1, 1]);

    // Release and settle the turn-in through the real 49/13 -> 49/14 path.
    now += 10;
    release_end_screen(&mut game, &naia, PILLS, NAIA, PILL_END_TEXT, now);
    now += 10;
    finish_quest(&mut game, &naia, PILLS, MEDIC, PILL_COINS, PILL_XP, now);
    let finished_coins = coins(&game);
    let finished_xp = medic_xp(&game);

    // Replayed packets after completion change neither the state nor the save.
    let before_state = game.state();
    let before_bytes = saved(&game);
    for (label, packet) in [
        ("replayed Palatable Pills turn-in", quest_end_reply(PILLS)),
        ("replayed Palatable Pills accept", quest_reply(PILLS, true)),
    ] {
        let outcome = send(&mut game, &naia, &packet, now + 10);
        assert!(!outcome.changed, "{label} changed gameplay");
    }
    assert_eq!(
        game.state(),
        before_state,
        "a replayed packet changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "a replayed packet changed the save"
    );
    assert_eq!(coins(&game), finished_coins);
    assert_eq!(medic_xp(&game), finished_xp);
    assert_eq!(quest_status(&game, PILLS), QuestStatus::Completed);
}

#[test]
fn surgical_precision_stays_disabled() {
    let fixture = Fixture::new();
    let mut now = 50_000;
    let (mut game, _naia, _greyli, gordon) = finish_chapter(&fixture, CHARACTER, &mut now);

    // The next record stays closed in content, and every later one does too.
    let surgical = medic_record(SURGICAL_PRECISION);
    assert_eq!(surgical["enabled"].as_bool(), Some(false));
    let document = read_json(&content_dir().join("quests/medic.json"));
    for record in document["records"].as_array().unwrap() {
        let id = record["id"].as_u64().unwrap();
        // T743 enables Callahan's Seaside level-20 gate; the rest stay off.
        if id >= 3_090_007 && id != u64::from(LEVEL_TWENTY) {
            assert_eq!(
                record["enabled"].as_bool(),
                Some(false),
                "later Medic record {id} must stay disabled"
            );
        }
    }

    // Even with every prerequisite and the rank-five job gate satisfied, the
    // disabled record is not offered.
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "Surgical Precision after Achieve Level 5",
    );
    assert_eq!(
        game.state(),
        before_state,
        "the disabled-record Talk changed state"
    );
    assert_eq!(
        saved(&game),
        before_bytes,
        "the disabled-record Talk changed the save"
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&SURGICAL_PRECISION),
        "Surgical Precision must not be offered"
    );
}

#[test]
fn a_second_medic_does_not_share_progress() {
    let fixture = Fixture::new();
    let mut now = 60_000;
    let (mut first, first_naia, first_greyli, first_gordon) =
        medic_ready(&fixture, CHARACTER, &mut now);
    let (mut second, second_naia, second_greyli, second_gordon) =
        medic_ready(&fixture, SECOND_CHARACTER, &mut now);

    // Only the first character starts the chapter.
    accept_pills(&mut first, &first_gordon, &mut now);
    now += 10;
    let visited = visit(&mut first, CHARACTER, sunflower_position(), now);
    assert!(visited.changed);
    assert_eq!(counts(&first, PILLS), vec![1, 0]);
    assert!(
        !second
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&PILLS),
        "the second Medic must not inherit the first's chapter"
    );
    let second_coins = coins(&second);
    let second_xp = medic_xp(&second);

    // The second character runs its own copy of the same chapter.
    accept_pills(&mut second, &second_gordon, &mut now);
    now += 10;
    let visited = visit(&mut second, SECOND_CHARACTER, sunflower_position(), now);
    assert!(visited.changed);
    assert_eq!(counts(&second, PILLS), vec![1, 0]);
    assert_eq!(counts(&first, PILLS), vec![1, 0]);

    // The first turn-in pays the first character only.
    now += 10;
    let talked = talk(&mut first, &first_naia, NAIA, now);
    assert!(talked.changed);
    now += 10;
    release_end_screen(&mut first, &first_naia, PILLS, NAIA, PILL_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut first,
        &first_naia,
        PILLS,
        MEDIC,
        PILL_COINS,
        PILL_XP,
        now,
    );
    assert_eq!(coins(&second), second_coins);
    assert_eq!(medic_xp(&second), second_xp);
    assert_eq!(counts(&second, PILLS), vec![1, 0]);

    // The second turn-in pays the second character only.
    now += 10;
    let talked = talk(&mut second, &second_naia, NAIA, now);
    assert!(talked.changed);
    now += 10;
    release_end_screen(&mut second, &second_naia, PILLS, NAIA, PILL_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut second,
        &second_naia,
        PILLS,
        MEDIC,
        PILL_COINS,
        PILL_XP,
        now,
    );
    assert_eq!(quest_status(&first, PILLS), QuestStatus::Completed);
    assert_eq!(quest_status(&second, PILLS), QuestStatus::Completed);
    assert_eq!(
        second.state().npc_quests.as_ref().unwrap().quests[&PILLS].counts,
        vec![1, 1]
    );
    assert_eq!(
        first.state().npc_quests.as_ref().unwrap().quests[&PILLS].counts,
        vec![1, 1]
    );

    // Both characters take the vitamins branch, then the level-five gate, and
    // neither reward leaks into the other's state.
    complete_vitamins(&mut first, &first_gordon, &first_greyli, &mut now);
    complete_vitamins(&mut second, &second_gordon, &second_greyli, &mut now);
    assert_eq!(quest_status(&first, VITAMINS), QuestStatus::Completed);
    assert_eq!(quest_status(&second, VITAMINS), QuestStatus::Completed);

    set_medic_xp(&fixture, CHARACTER, &mut first, 400);
    set_medic_xp(&fixture, SECOND_CHARACTER, &mut second, 400);
    let second_coins = coins(&second);
    let second_xp = medic_xp(&second);
    now += 10;
    offer(&mut first, &first_gordon, GORDON, LEVEL_FIVE, now);
    now += 10;
    accept(&mut first, &first_gordon, LEVEL_FIVE, now);
    now += 10;
    let talked = talk(&mut first, &first_gordon, GORDON, now);
    assert!(talked.changed);
    now += 10;
    release_end_screen(
        &mut first,
        &first_gordon,
        LEVEL_FIVE,
        GORDON,
        LEVEL_END_TEXT,
        now,
    );
    now += 10;
    finish_quest(
        &mut first,
        &first_gordon,
        LEVEL_FIVE,
        MEDIC,
        LEVEL_COINS,
        LEVEL_XP,
        now,
    );
    assert_eq!(coins(&second), second_coins);
    assert_eq!(medic_xp(&second), second_xp);
    assert!(
        !second
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&LEVEL_FIVE),
        "the second Medic must not inherit Achieve Level 5"
    );

    now += 10;
    offer(&mut second, &second_gordon, GORDON, LEVEL_FIVE, now);
    now += 10;
    accept(&mut second, &second_gordon, LEVEL_FIVE, now);
    now += 10;
    let talked = talk(&mut second, &second_gordon, GORDON, now);
    assert!(talked.changed);
    now += 10;
    release_end_screen(
        &mut second,
        &second_gordon,
        LEVEL_FIVE,
        GORDON,
        LEVEL_END_TEXT,
        now,
    );
    now += 10;
    finish_quest(
        &mut second,
        &second_gordon,
        LEVEL_FIVE,
        MEDIC,
        LEVEL_COINS,
        LEVEL_XP,
        now,
    );
    assert_eq!(quest_status(&first, LEVEL_FIVE), QuestStatus::Completed);
    assert_eq!(quest_status(&second, LEVEL_FIVE), QuestStatus::Completed);
    assert_eq!(medic_xp(&first), 425);
    assert_eq!(medic_xp(&second), 425);
}

#[test]
fn a_failing_quest_event_for_one_player_does_not_stop_the_other() {
    let fixture = Fixture::new();
    let mut now = 70_000;
    let (mut first, first_naia, _first_greyli, first_gordon) =
        medic_ready(&fixture, CHARACTER, &mut now);
    let (mut second, second_naia, second_greyli, second_gordon) =
        medic_ready(&fixture, SECOND_CHARACTER, &mut now);

    // The first character holds a pending end screen with both objectives done.
    accept_pills(&mut first, &first_gordon, &mut now);
    now += 10;
    let visited = visit(&mut first, CHARACTER, sunflower_position(), now);
    assert!(visited.changed);
    now += 10;
    let talked = talk(&mut first, &first_naia, NAIA, now);
    assert!(talked.changed);
    assert_eq!(pending_turnin(&first), Some(PILLS));

    // Wallet cap: the near-cap coin balance refuses for this player only.
    patch_state(&fixture, CHARACTER, &mut first, |state| {
        state.inventory.coins = i32::MAX as u32 - 2;
    });
    let before_state = first.state();
    let before_bytes = saved(&first);
    // T382: a per-player denial, not a session-ending error or a panic.
    let failed = first
        .handle_in_world(&quest_end_reply(PILLS), &first_naia, now + 1)
        .expect("a near-cap wallet is a refusal, not an error");
    assert!(!failed.changed, "a near-cap wallet must refuse the turn-in");
    assert_eq!(
        first.state(),
        before_state,
        "the wallet refusal changed the first player's state"
    );
    assert_eq!(
        saved(&first),
        before_bytes,
        "the wallet refusal changed the first player's save"
    );
    assert_eq!(pending_turnin(&first), Some(PILLS));

    // Profile cap: the same pending turn-in now fails at the XP add, again
    // leaving the first player's state and save untouched.
    patch_state(&fixture, CHARACTER, &mut first, |state| {
        state.inventory.coins = 0;
        state
            .profiles
            .as_mut()
            .expect("profile state")
            .experience
            .insert(MEDIC, u32::MAX - 10);
    });
    let before_state = first.state();
    let before_bytes = saved(&first);
    let failed = first
        .handle_in_world(&quest_end_reply(PILLS), &first_naia, now + 2)
        .expect("a near-cap profile is a refusal, not an error");
    assert!(
        !failed.changed,
        "a near-cap profile must refuse the turn-in"
    );
    assert_eq!(
        first.state(),
        before_state,
        "the profile refusal changed the first player's state"
    );
    assert_eq!(
        saved(&first),
        before_bytes,
        "the profile refusal changed the first player's save"
    );
    assert_eq!(pending_turnin(&first), Some(PILLS));

    // A stale reply for another quest changes nothing and does not fault the session.
    let stale = send(
        &mut first,
        &first_naia,
        &quest_end_reply(LEVEL_FIVE),
        now + 3,
    );
    assert!(!stale.changed, "a stale 49/14 changed gameplay");
    assert_eq!(
        saved(&first),
        before_bytes,
        "a stale 49/14 changed the save"
    );

    // The second player runs the ordinary chapter to completion in the meantime.
    let second_coins = coins(&second);
    complete_pills(
        &mut second,
        &second_gordon,
        &second_naia,
        SECOND_CHARACTER,
        &mut now,
    );
    complete_vitamins(&mut second, &second_gordon, &second_greyli, &mut now);
    set_medic_xp(&fixture, SECOND_CHARACTER, &mut second, 400);
    let second_xp = medic_xp(&second);
    now += 10;
    offer(&mut second, &second_gordon, GORDON, LEVEL_FIVE, now);
    now += 10;
    accept(&mut second, &second_gordon, LEVEL_FIVE, now);
    now += 10;
    let talked = talk(&mut second, &second_gordon, GORDON, now);
    assert!(talked.changed);
    now += 10;
    release_end_screen(
        &mut second,
        &second_gordon,
        LEVEL_FIVE,
        GORDON,
        LEVEL_END_TEXT,
        now,
    );
    now += 10;
    finish_quest(
        &mut second,
        &second_gordon,
        LEVEL_FIVE,
        MEDIC,
        LEVEL_COINS,
        LEVEL_XP,
        now,
    );
    assert_eq!(
        coins(&second),
        second_coins + PILL_COINS + VITAMIN_COINS + LEVEL_COINS
    );
    assert_eq!(medic_xp(&second), second_xp + LEVEL_XP);

    // The first player's failed turn-in is still isolated and the session remains
    // usable: a normal interact succeeds and changes no saved state.
    let before_state = first.state();
    let before_bytes = saved(&first);
    let menu = first
        .handle_in_world(&npc_interact(NAIA), &first_naia, now + 10)
        .expect("the first session stays usable after a refused turn-in");
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "the first player can still open the NPC menu"
    );
    assert_eq!(
        first.state(),
        before_state,
        "the interact changed saved state"
    );
    assert_eq!(saved(&first), before_bytes, "the interact changed the save");
    assert_eq!(pending_turnin(&first), Some(PILLS));
}
