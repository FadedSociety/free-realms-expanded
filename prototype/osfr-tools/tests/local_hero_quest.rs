//! T277: the Royal Courier's "Local Hero" entry quest (3000024).
//!
//! The quest is content-only: a five-kill `defeat` goal against the live level-3
//! field Hooligan kind 930050 and a final Talk to the Courier, then 13 coins and
//! 25 Adventurer (profile 1) XP. These tests drive the production in-world router
//! (`Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `QuestEngine`) and the
//! production shared-field request/tick seams, never engine internals.
//!
//! Client wire evidence used here:
//! - 26/8 NPC interact reader `0x008b6bc0`; 26/10 menu writer `0x00a36f80`;
//!   26/6 dialogue reader `0x009f5890`; 26/4 ends the dialogue.
//! - 49/1 QuestInfo reader `0x00c7bb60`: quest id dword 0 at 6..10, intro at
//!   10..14, card title at 18..22 (the writer also repeats the title in the two
//!   probe-unresolved dwords).
//! - 49/2 accept constructor `0x00a86400`; 49/3 QuestAdd reader `0x00c7bd10`;
//!   49/13 end-screen writer/reader `0x00c7bbd0`/`0x00c7b990`; 49/14 reply
//!   constructor `0x00a864c0`.
//! - 49/9 ObjectiveUpdate / 49/10 GoalComplete / 49/8 ObjectiveActivated are the
//!   existing T256 packets for a shared-field kill.
//!
//! The shared fixture, packet builders and shared-field/objective helpers live in
//! `tests/support/npc_quest_chain.rs`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::*;
use osfr_tools::{
    character_store::CharacterState,
    encounter::SharedFieldState,
    game_store,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};
use std::collections::BTreeSet;

const QUEST: u32 = 3_000_024;
const COURIER: u64 = 9_000_000_444;
/// `title_string_id`, `intro_text_string_id`, `description_string_id` and
/// `end_text_string_id` of 3000024.
const TITLE_ID: u32 = 389_187;
const INTRO_ID: u32 = 389_186;
const DESCRIPTION_ID: u32 = 389_188;
const END_TEXT: u32 = 389_189;
/// The record's reward row and objective ordinals.
const REWARD_COINS: u32 = 13;
const REWARD_XP: u32 = 25;
const ADVENTURER: i32 = 1;
const MEDIC: i32 = 11;
const BRAWLER: i32 = 43;
const DEFEAT_OBJECTIVE: u32 = 1;
const TALK_OBJECTIVE: u32 = 2;
/// The three always-on level-3 930050 Hooligans of `starting_highroad_hooligans`
/// and one level-7 930060 from the mid camp (data/content/field_enemies.json).
const LIVE_HOOLIGANS: [u64; 3] = [131_000_102_004, 131_000_102_005, 131_000_102_006];
const MID_HOOLIGAN: u64 = 131_000_102_015;
const A: u64 = 101;
const B: u64 = 202;
/// One field spawn cycle of `starting_highroad_hooligans`.
const RESPAWN_WAIT_MS: u64 = 61_000;

fn courier_character(guid: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("greenwood_forest.json", COURIER))
}

fn player_at(guid: u64, position: [f32; 3]) -> CharacterState {
    character_at_for(guid, 0, position)
}

fn hooligans() -> BTreeSet<u64> {
    let mut known = BTreeSet::from(LIVE_HOOLIGANS);
    known.insert(MID_HOOLIGAN);
    known
}

/// The opening fixture's default profile state, with the chosen job active. The
/// quest has no `job` gate; this only gives the shared-field attacks a weapon.
fn game_with_job(fixture: &Fixture, character_id: u64, job: i32) -> Gameplay {
    let mut state = Gameplay::load(fixture.path(), None)
        .expect("the opening fixture loads")
        .state();
    let profiles = state
        .profiles
        .as_mut()
        .expect("the opening fixture stores profiles");
    profiles.active = job;
    profiles.unlocked.insert(job);
    fixture.game_as(character_id, Some(state))
}

/// The 49/1 offer for the record carries the four screen fields at the client
/// reader's offsets (intro, repeated title, card title, description).
fn assert_offer(outcome: &Outcome) {
    let offer = outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(QUEST)))
        .expect("the Courier sends the 49/1 Local Hero offer");
    let word = |offset: usize| u32::from_le_bytes(offer[offset..offset + 4].try_into().unwrap());
    assert_eq!(
        word(10),
        INTRO_ID,
        "49/1 intro field (client reader 0x00c7b7a0)"
    );
    assert_eq!(word(14), TITLE_ID, "49/1 repeats the card title");
    assert_eq!(word(18), TITLE_ID, "49/1 card title is Local Hero (389187)");
    assert_eq!(word(22), DESCRIPTION_ID, "49/1 description field");
}

/// Accept Local Hero at the Courier through the real 26/8 -> 26/10 -> 26/6 ->
/// 49/1 -> 49/2 path. Returns the game and the next free timestamp.
fn start_local_hero(fixture: &Fixture, character_id: u64) -> (Gameplay, u64) {
    let mut game = game_with_job(fixture, character_id, BRAWLER);
    let character = courier_character(character_id);
    let offered = offer(&mut game, &character, COURIER, QUEST, 1_000);
    assert_offer(&offered);
    accept(&mut game, &character, QUEST, 1_010);
    (game, 1_020)
}

/// Three live kills plus two kills after one respawn cycle.
fn five_kills(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    known: &BTreeSet<u64>,
    character: &CharacterState,
    mut now: u64,
) -> u64 {
    for target in LIVE_HOOLIGANS {
        let (_, next) = kill_shared_target(game, field, known, character, target, now);
        now = next;
    }
    now += RESPAWN_WAIT_MS;
    tick_shared_field(game, field, known, character.guid, now, RESPAWN_WAIT_MS);
    for target in [LIVE_HOOLIGANS[0], LIVE_HOOLIGANS[1]] {
        let (_, next) = kill_shared_target(game, field, known, character, target, now);
        now = next;
    }
    now
}

fn reward_packets(outcome: &Outcome, subtype: u16) -> usize {
    outcome
        .replies
        .iter()
        .filter(|packet| has_family(packet, 38, subtype, None))
        .count()
}

#[test]
fn the_royal_courier_offers_local_hero_to_every_job() {
    let fixture = Fixture::new();
    for (index, job) in [BRAWLER, MEDIC, ADVENTURER].into_iter().enumerate() {
        let character_id = 300 + index as u64;
        let mut game = game_with_job(&fixture, character_id, job);
        let character = courier_character(character_id);
        let offered = offer(&mut game, &character, COURIER, QUEST, 1_000);
        assert_offer(&offered);
        assert_eq!(
            quest_status(&game, QUEST),
            QuestStatus::Offered,
            "job {job} sees exactly one Local Hero offer"
        );
        accept(&mut game, &character, QUEST, 1_010);
        assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

        // A second Talk while the quest is active must not offer it again.
        let mut now = 1_100;
        assert_no_offer(
            &mut game,
            &character,
            COURIER,
            &mut now,
            &format!("active Local Hero talk as job {job}"),
        );
        assert_eq!(counts(&game, QUEST), [0, 0]);
        assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    }
}

#[test]
fn five_hooligan_kills_advance_the_hunt_once() {
    let fixture = Fixture::new();
    let (mut game, mut now) = start_local_hero(&fixture, A);
    let a_character = courier_character(A);
    let mut field = game
        .take_shared_fields()
        .expect("the opening field is available");
    let known = hooligans();

    let (first, next) = kill_shared_target(
        &mut game,
        &mut field,
        &known,
        &a_character,
        LIVE_HOOLIGANS[0],
        now,
    );
    now = next;
    assert_eq!(counts(&game, QUEST), [1, 0], "one kill is one credit");
    assert!(
        objective_update(&first, QUEST, DEFEAT_OBJECTIVE, 1),
        "49/9 ObjectiveUpdate reports 1/5"
    );

    for target in [LIVE_HOOLIGANS[1], LIVE_HOOLIGANS[2]] {
        let (_, next) =
            kill_shared_target(&mut game, &mut field, &known, &a_character, target, now);
        now = next;
    }
    assert_eq!(counts(&game, QUEST), [3, 0]);
    // Replaying a request on the same corpse must not credit it twice.
    let saved = game.state();
    let corpse = player_at(A, field.position(LIVE_HOOLIGANS[0]).unwrap());
    let replay = shared_attack(
        &mut game,
        &mut field,
        &corpse,
        &known,
        LIVE_HOOLIGANS[0],
        now,
    );
    now += SHARED_ATTACK_INTERVAL_MS;
    assert!(
        !replay.changed && replay.replies.is_empty(),
        "a corpse replay changes nothing"
    );
    assert_eq!(game.state(), saved, "a corpse replay changed the save");

    // The level-7 930060 is a different kind and must not advance Local Hero.
    let (mid, next) = kill_shared_target(
        &mut game,
        &mut field,
        &known,
        &a_character,
        MID_HOOLIGAN,
        now,
    );
    now = next;
    assert_eq!(field.health(MID_HOOLIGAN), Some(0));
    assert_eq!(
        counts(&game, QUEST),
        [3, 0],
        "only 930050 counts for the defeat goal"
    );
    assert!(
        !objective_update(&mid, QUEST, DEFEAT_OBJECTIVE, 4)
            && !objective_complete(&mid, QUEST, DEFEAT_OBJECTIVE),
        "the 930060 kill produces no Local Hero objective packet"
    );

    // Three live actors plus one respawn cycle complete the five-kill hunt.
    now += RESPAWN_WAIT_MS;
    tick_shared_field(&mut game, &mut field, &known, A, now, RESPAWN_WAIT_MS);
    for target in [LIVE_HOOLIGANS[0], LIVE_HOOLIGANS[1]] {
        assert!(
            field.health(target).unwrap() > 0,
            "the 930050 at {target} respawned"
        );
    }
    let mut fifth = Outcome::default();
    for target in [LIVE_HOOLIGANS[0], LIVE_HOOLIGANS[1]] {
        let (outcome, next) =
            kill_shared_target(&mut game, &mut field, &known, &a_character, target, now);
        now = next;
        fifth = outcome;
    }
    assert_eq!(counts(&game, QUEST), [5, 0]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert!(
        objective_complete(&fifth, QUEST, DEFEAT_OBJECTIVE),
        "the fifth counted kill sends 49/10 GoalComplete"
    );
    assert!(
        objective_activated(&fifth, QUEST, TALK_OBJECTIVE),
        "the completed kill goal activates the Courier Talk objective"
    );
}

#[test]
fn the_courier_turn_in_pays_thirteen_coins_and_adventurer_xp_once() {
    let fixture = Fixture::new();
    let (mut game, mut now) = start_local_hero(&fixture, A);
    let courier = courier_character(A);
    let mut field = game
        .take_shared_fields()
        .expect("the opening field is available");
    let known = hooligans();
    now = five_kills(&mut game, &mut field, &known, &courier, now);
    assert_eq!(counts(&game, QUEST), [5, 0]);

    let talked = talk(&mut game, &courier, COURIER, now);
    now += 10;
    assert!(talked.changed, "the final Talk advances the objective");
    assert_eq!(counts(&game, QUEST), [5, 1]);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    release_end_screen(&mut game, &courier, QUEST, COURIER, END_TEXT, now);
    now += 10;

    let coins_before = coins(&game);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    let brawler_before = profile_xp(&game, BRAWLER);
    assert_eq!(
        active_job(&game),
        BRAWLER,
        "the reward must not come from the active job"
    );
    let completed = finish_quest(
        &mut game,
        &courier,
        QUEST,
        ADVENTURER,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    now += 10;
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        adventurer_before + REWARD_XP,
        "profile 1 (Adventurer) receives the XP"
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before,
        "the active Brawler job receives no quest XP"
    );
    assert_eq!(reward_packets(&completed, 19), 1, "one coin grant");
    assert_eq!(reward_packets(&completed, 14), 1, "one XP grant");

    // Paid once: a replayed 49/14 changes neither state nor save.
    assert_packet_changes_nothing(
        &mut game,
        &courier,
        &quest_end_reply(QUEST),
        now,
        "replayed Local Hero turn-in",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + REWARD_XP);
}

#[test]
fn a_refused_or_out_of_range_request_changes_nothing() {
    let fixture = Fixture::new();
    let mut game = game_with_job(&fixture, A, BRAWLER);
    let courier = courier_character(A);
    let mut now = 5_000;

    // A non-giver NPC's Talk cannot offer or start Local Hero.
    let vendor = player_at(A, content_position("greenwood_forest.json", 9_500_000_285));
    assert_no_offer(
        &mut game,
        &vendor,
        9_500_000_285,
        &mut now,
        "Local Hero at a non-giver",
    );

    // Forged accept with no offer open.
    assert_packet_changes_nothing(
        &mut game,
        &courier,
        &quest_reply(QUEST, true),
        now + 1,
        "forged Local Hero accept",
    );
    // A 49/14 with no pending end screen.
    assert_packet_changes_nothing(
        &mut game,
        &courier,
        &quest_end_reply(QUEST),
        now + 2,
        "Local Hero turn-in with no end screen",
    );

    // 26/8 out of range is refused before any menu opens.
    let far = player_at(
        A,
        [
            courier.position[0] + 400.0,
            courier.position[1],
            courier.position[2],
        ],
    );
    let before = saved(&game);
    let refused = game.handle_in_world(&npc_interact(COURIER), &far, now + 3);
    assert!(refused.is_err(), "the out-of-range interact is refused");
    assert_eq!(
        saved(&game),
        before,
        "the out-of-range interact changed the state"
    );

    // A real offer, then a duplicate offer attempt keeps the open offer intact.
    now += 10;
    let offered = offer(&mut game, &courier, COURIER, QUEST, now);
    assert_offer(&offered);
    let held = saved(&game);
    now += 10;
    let menu = interact(&mut game, &courier, COURIER, now);
    let choice = menu_option(&menu, TALK_LABEL);
    let dialogue = send(&mut game, &courier, &npc_select(COURIER, choice), now + 1);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the second Talk still opens the dialogue"
    );
    let duplicate = send(&mut game, &courier, &dialogue_choice(QUEST_CHOICE), now + 2);
    assert!(
        !duplicate
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(QUEST))),
        "an open offer is not offered a second time"
    );
    assert!(!duplicate.changed);
    assert_eq!(
        saved(&game),
        held,
        "the duplicate offer attempt changed the state"
    );

    // Accept, then a declined 49/2 with no pending offer is a refusal.
    accept(&mut game, &courier, QUEST, now + 3);
    assert_packet_changes_nothing(
        &mut game,
        &courier,
        &quest_reply(QUEST, false),
        now + 4,
        "declined 49/2 with no pending offer",
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
}

#[test]
fn progress_survives_a_reconnect() {
    let fixture = Fixture::new();
    let (mut game, mut now) = start_local_hero(&fixture, A);
    let save_path = fixture.path().join("reconnect-save.json");
    let character = courier_character(A);
    let coins_before = coins(&game);
    let adventurer_before = profile_xp(&game, ADVENTURER);

    let mut field = game
        .take_shared_fields()
        .expect("the opening field is available");
    let known = hooligans();
    for target in LIVE_HOOLIGANS {
        let (_, next) = kill_shared_target(&mut game, &mut field, &known, &character, target, now);
        now = next;
    }
    assert_eq!(counts(&game, QUEST), [3, 0]);

    // Disconnect with three kills saved, then reconnect through the real
    // character-document envelope.
    let held = game.state();
    game_store::save(
        &save_path,
        &game_store::Snapshot {
            character: character.clone(),
            gameplay: Some(held.clone()),
            pets: None,
        },
    )
    .expect("the character document saves");
    let loaded = game_store::load_or_create(&save_path, character.clone())
        .expect("the character document loads");
    assert_eq!(loaded.gameplay.as_ref(), Some(&held));
    game = fixture.game_as(A, loaded.gameplay);
    assert_eq!(counts(&game, QUEST), [3, 0], "the counter survives reload");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    let mut field = game
        .take_shared_fields()
        .expect("the reloaded field is available");
    for target in [LIVE_HOOLIGANS[0], LIVE_HOOLIGANS[1]] {
        let (_, next) = kill_shared_target(&mut game, &mut field, &known, &character, target, now);
        now = next;
    }
    assert_eq!(counts(&game, QUEST), [5, 0]);

    let talked = talk(&mut game, &character, COURIER, now);
    now += 10;
    assert!(talked.changed);
    release_end_screen(&mut game, &character, QUEST, COURIER, END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        QUEST,
        ADVENTURER,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    let expected = game.state();
    let expected_bytes = saved(&game);

    // A second reload keeps the completed history and the paid balance.
    game_store::save(
        &save_path,
        &game_store::Snapshot {
            character: character.clone(),
            gameplay: Some(expected.clone()),
            pets: None,
        },
    )
    .expect("the finished character document saves");
    let loaded = game_store::load_or_create(&save_path, character.clone())
        .expect("the finished character document loads");
    assert_eq!(loaded.gameplay.as_ref(), Some(&expected));
    game = fixture.game_as(A, loaded.gameplay);
    assert_eq!(
        saved(&game),
        expected_bytes,
        "the second reload changed the save"
    );
    assert_eq!(counts(&game, QUEST), [5, 1]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + REWARD_XP);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(QUEST),
        now + 10,
        "replayed turn-in after reload",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + REWARD_XP);
}

#[test]
fn a_second_player_killing_the_same_hooligans_does_not_advance_the_first() {
    let fixture = Fixture::new();
    let (mut a, now) = start_local_hero(&fixture, A);
    let (mut b, _) = start_local_hero(&fixture, B);
    let a_character = courier_character(A);
    let b_character = courier_character(B);
    let mut field = a
        .take_shared_fields()
        .expect("the shared opening field is available");
    b.take_shared_fields();
    let known = hooligans();

    let b_before = b.state();
    let (_, now) = kill_shared_target(
        &mut a,
        &mut field,
        &known,
        &a_character,
        LIVE_HOOLIGANS[0],
        now,
    );
    assert_eq!(counts(&a, QUEST), [1, 0], "the killer is credited");
    assert_eq!(counts(&b, QUEST), [0, 0], "the bystander is not credited");
    assert_eq!(b.state(), b_before, "A's kill changed B's saved state");

    let (_, now) = kill_shared_target(
        &mut b,
        &mut field,
        &known,
        &b_character,
        LIVE_HOOLIGANS[1],
        now,
    );
    assert_eq!(counts(&b, QUEST), [1, 0], "B's own kill credits B");
    assert_eq!(counts(&a, QUEST), [1, 0], "B's kill does not credit A");
    let b_after_own = b.state();

    let (_, mut now) = kill_shared_target(
        &mut a,
        &mut field,
        &known,
        &a_character,
        LIVE_HOOLIGANS[2],
        now,
    );
    assert_eq!(counts(&a, QUEST), [2, 0]);
    assert_eq!(
        b.state(),
        b_after_own,
        "A's second kill changed B's saved state"
    );

    // Both players can still take their own kills from the same field.
    now += RESPAWN_WAIT_MS;
    tick_shared_field(&mut a, &mut field, &known, A, now, RESPAWN_WAIT_MS);
    let (_, _) = kill_shared_target(
        &mut b,
        &mut field,
        &known,
        &b_character,
        LIVE_HOOLIGANS[0],
        now,
    );
    assert_eq!(counts(&b, QUEST), [2, 0]);
    assert_eq!(counts(&a, QUEST), [2, 0]);
}

#[test]
fn a_failing_quest_event_for_one_player_does_not_stop_the_other() {
    let fixture = Fixture::new();

    // A's save belongs to another character: every production bind refuses it.
    let (valid, _) = start_local_hero(&fixture, A);
    let mut stale = valid.state();
    stale
        .npc_quests
        .as_mut()
        .expect("Local Hero is active")
        .character_id = B;
    let mut a = Gameplay::load(fixture.path(), Some(stale)).expect("A's stale session loads");
    let error = a
        .bind_npc_quest_catalogue(&content_dir(), A)
        .err()
        .expect("A's stale quest save is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    let (mut b, mut now_b) = start_local_hero(&fixture, B);
    let b_character = courier_character(B);
    let mut field = a
        .take_shared_fields()
        .expect("the stale session still has a shared field");
    b.take_shared_fields();
    let known = hooligans();

    // A's kill is accepted but cannot credit the unbound stale state.
    let (_, _) = kill_shared_target(
        &mut a,
        &mut field,
        &known,
        &courier_character(A),
        LIVE_HOOLIGANS[0],
        10_000,
    );
    assert_eq!(counts(&a, QUEST), [0, 0], "the stale save is not credited");

    // B's engine in the same field still advances normally.
    let (out, next) = kill_shared_target(
        &mut b,
        &mut field,
        &known,
        &b_character,
        LIVE_HOOLIGANS[1],
        now_b,
    );
    now_b = next;
    assert_eq!(counts(&b, QUEST), [1, 0]);
    assert!(
        objective_update(&out, QUEST, DEFEAT_OBJECTIVE, 1),
        "B's kill still updates B's objective"
    );

    // Near-cap variant: A's reward cannot fit the wire i32 coin field. The
    // refusal is A's alone; B and the shared field stay usable.
    let (mut capped, mut now_c) = start_local_hero(&fixture, A);
    let a_character = courier_character(A);
    let mut capped_field = capped
        .take_shared_fields()
        .expect("A's capped session has a field");
    now_c = five_kills(&mut capped, &mut capped_field, &known, &a_character, now_c);
    let talked = talk(&mut capped, &a_character, COURIER, now_c);
    now_c += 10;
    assert!(talked.changed);
    assert_eq!(pending_turnin(&capped), Some(QUEST));
    release_end_screen(&mut capped, &a_character, QUEST, COURIER, END_TEXT, now_c);
    now_c += 10;
    let mut state = capped.state();
    state.inventory.coins = i32::MAX as u32;
    let mut capped = fixture.game_as(A, Some(state));
    let before = saved(&capped);
    let refused = capped.handle_in_world(&quest_end_reply(QUEST), &a_character, now_c);
    assert!(
        refused.is_err(),
        "a coin grant past i32::MAX is refused for that player"
    );
    assert_eq!(
        saved(&capped),
        before,
        "the refused payout changed A's save"
    );
    assert_eq!(pending_turnin(&capped), Some(QUEST));

    // B still kills and credits in the unchanged shared field.
    let (_, _) = kill_shared_target(
        &mut b,
        &mut field,
        &known,
        &b_character,
        LIVE_HOOLIGANS[2],
        now_b,
    );
    assert_eq!(counts(&b, QUEST), [2, 0]);
    assert_eq!(field.health(LIVE_HOOLIGANS[2]), Some(0));
}
