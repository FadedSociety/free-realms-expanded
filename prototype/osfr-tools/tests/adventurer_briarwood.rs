//! T377: "Chapter 1: Rumors of Briarwood" (`3000033`) and "Chapter 1: Shrouded
//! History" (`3000034`) are playable story beats after T330's Thugamug chapter.
//!
//! The two records live in `data/content/quests/adventurer.json`; the giver is
//! Geoffrey (`9000000330`), already bound in `npcs/sanctuary.json`. E5's scout
//! anchor is the shipped Sergeant Cargug row (`npcs/briarwood.json:9500000209`)
//! and E6's druid target is Arch Druid Camellia
//! (`npcs/shrouded_glade.json:9500000650`); both are positioned `legacy_data`
//! rows, so `world::NpcVisibility` draws them in the open-world zone 0 at their
//! authored position. No NPC, spawn, item, battle or wire shape is added.
//!
//! These tests drive the production paths: `Gameplay::handle_in_world` (26/8 ->
//! 26/10 -> 26/6 -> 49/1/49/2), `Gameplay::on_location` (the server pose ->
//! VisitPosition -> 49/9/49/10), the ordinary 49/13 end screen released by 26/6,
//! the 49/14 turn-in through `rewards::apply_grant`, and a real `game_store`
//! save/reload. The shared fixture and packet builders live in
//! `tests/support/npc_quest_chain.rs`.
//!
//! Client layouts used here (IDA, 2014 client; no new shape):
//! - 26/8 interact `0x008b6bc0`, 26/10 select `0x00a36f80`, 26/6 dialogue
//!   `0x009f5890`.
//! - 49/1 offer reader `0x00c7bb60` / body `0x00c7b7a0` (six dwords, bool,
//!   reward bundle `0x008e7930`, 8 bytes, float, two bools).
//! - 49/2 reply `0x00a86400`; 49/13 end reader `0x00c7b990` (actor u64, quest,
//!   end text, animation dword, reward bundle, trailing float); 49/14 reply
//!   `0x00a864c0`.
//! - 49/9 objective update `0x00c7af60`; 49/10 objective complete `0x00c7b070`.
//!
//! The E2/E3 records (`3000030`/`3000031`) stay disabled and hollow: their
//! backfilled text describes gathering Ammi Lilies, and the shipped content has
//! no Ammi Lily item or resource. E5 therefore chains from the playable E4
//! (`3000032`), and E6 from E5.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, end_screen_reward,
    finish_quest, has_family, objective_activated, objective_complete, objective_update, offer,
    pending_offer, pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status,
    read_json, release_end_screen, saved, select_job, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    game_store::{self, Snapshot},
    gameplay::{Gameplay, GameplayState, Outcome},
    npc_interaction::{QuestOfferPacket49_1, decode_quest_offer_49_1},
    profiles::ProfileState,
    quest_engine::{QuestProgress, QuestState, QuestStatus},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const GEOFFREY: u64 = 9_000_000_330;
const CARGUG: u64 = 9_500_000_209;
const CAMELLIA: u64 = 9_500_000_650;
const OTHER: u64 = CHARACTER + 1;
/// T279's playable E1; the chapter entry already in the save.
const MISSING_SISTERS: u32 = 3_000_029;
/// T330's playable E4; E5's prerequisite.
const THIEVERY: u32 = 3_000_032;
const RUMORS: u32 = 3_000_033;
const SHROUDED: u32 = 3_000_034;
/// The hollow retail middle records this slice deliberately leaves disabled.
const SKIPPED: [u32; 2] = [3_000_030, 3_000_031];
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
const REWARD_COINS: u32 = 8;
const REWARD_XP: u32 = 20;
const RUMORS_INTRO: u32 = 90_725;
const RUMORS_TITLE: u32 = 90_726;
const RUMORS_DESCRIPTION: u32 = 90_727;
const RUMORS_END: u32 = 90_728;
const SHROUDED_INTRO: u32 = 90_770;
const SHROUDED_TITLE: u32 = 90_771;
const SHROUDED_DESCRIPTION: u32 = 90_772;
const SHROUDED_END: u32 = 90_773;
/// Sergeant Cargug's shipped `legacy_data` position in `npcs/briarwood.json`.
const CARGUG_POSITION: [f32; 3] = [-487.5454, -75.8251, 2242.396];
/// Arch Druid Camellia's shipped `legacy_data` position in `npcs/shrouded_glade.json`.
const CAMELLIA_POSITION: [f32; 3] = [103.6414, 69.12641, 1235.769];
/// (R) T279's authored visit radius; E5 reuses it instead of inventing one.
const VISIT_RADIUS: f32 = 8.0;

fn character(guid: u64, position: [f32; 3]) -> CharacterState {
    character_at_for(guid, 0, position)
}

fn geoffrey_position() -> [f32; 3] {
    content_position("sanctuary.json", GEOFFREY)
}

fn quest_record(id: u32) -> Value {
    read_json(&content_dir().join("quests/adventurer.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(id)))
        .cloned()
        .unwrap_or_else(|| panic!("quest {id} is authored"))
}

fn npc_row(file: &str, npc_id: u64) -> Value {
    read_json(&content_dir().join("npcs").join(file))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(npc_id))
        .cloned()
        .unwrap_or_else(|| panic!("NPC {npc_id} is authored in {file}"))
}

fn row_position(row: &Value) -> [f32; 3] {
    serde_json::from_value(row["position"].clone()).expect("finite NPC position")
}

/// The completed save state of the chapter's earlier playable records. Their own
/// runs (E1's wolf field, E4's T282 battle) are pinned by T279/T330; this slice
/// seeds only the prerequisite state and lets the production loader validate
/// every counter when the catalogue binds.
fn completed(character_id: u64, entries: &[(u32, Vec<u32>)]) -> QuestState {
    let quests = entries
        .iter()
        .map(|(id, counts)| {
            (
                *id,
                QuestProgress {
                    status: QuestStatus::Completed,
                    counts: counts.clone(),
                },
            )
        })
        .collect();
    QuestState {
        version: 1,
        character_id,
        quests,
        pending_offer: None,
        pending_turnin: None,
        pending_turnin_actor: None,
        selected_quest_id: None,
        auto_switch_selected: false,
    }
}

/// E1 and E4 completed, so Geoffrey's next eligible record is E5.
fn chapter_prerequisites(character_id: u64) -> QuestState {
    completed(
        character_id,
        &[(MISSING_SISTERS, vec![1, 3, 1]), (THIEVERY, vec![1, 1])],
    )
}

fn player(fixture: &Fixture, character_id: u64, npc_quests: QuestState) -> Gameplay {
    player_with_coins(fixture, character_id, npc_quests, None)
}

/// As [`player`], with an optional wallet override so a near-cap refusal can be
/// driven through the same production turn-in path.
fn player_with_coins(
    fixture: &Fixture,
    character_id: u64,
    npc_quests: QuestState,
    coins: Option<u32>,
) -> Gameplay {
    let base = Gameplay::load(fixture.path(), None).expect("the reference opening content loads");
    let mut state = base.state();
    state.npc_quests = Some(npc_quests);
    if let Some(coins) = coins {
        state.inventory.coins = coins;
    }
    state.profiles = Some(ProfileState {
        version: 1,
        active: ADVENTURER,
        unlocked: [ADVENTURER, MINER, BRAWLER].into_iter().collect(),
        experience: BTreeMap::new(),
    });
    fixture.game_as(character_id, Some(state))
}

/// The real save envelope: `game_store::save` -> `load_or_create` -> the same
/// split-catalogue bind a login performs.
fn save_and_reload(
    fixture: &Fixture,
    character_id: u64,
    game: &Gameplay,
    at: &CharacterState,
    name: &str,
) -> Gameplay {
    let path = fixture.path().join(name);
    game_store::save(
        &path,
        &Snapshot {
            character: at.clone(),
            gameplay: Some(game.state()),
            pets: None,
        },
    )
    .expect("the character save writes");
    let snapshot = game_store::load_or_create(&path, at.clone()).expect("the character save loads");
    let reloaded = fixture.game_as(character_id, snapshot.gameplay);
    assert_eq!(
        saved(&reloaded),
        saved(game),
        "the save round trip preserved the serialized state"
    );
    reloaded
}

fn offer_packet(outcome: &Outcome, quest: u32) -> &[u8] {
    outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(quest)))
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("quest {quest} was not offered"))
}

fn decoded_offer(outcome: &Outcome, quest: u32) -> QuestOfferPacket49_1 {
    decode_quest_offer_49_1(offer_packet(outcome, quest)).expect("the 49/1 offer decodes")
}

/// A Talk to Geoffrey and the 26/6 that releases the held 49/13.
fn open_end_screen(
    game: &mut Gameplay,
    at: &CharacterState,
    quest: u32,
    end_text: u32,
    now: u64,
) -> Vec<u8> {
    let dialogue = talk(game, at, GEOFFREY, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Geoffrey opens the 26/3 dialogue"
    );
    assert!(
        !dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the end screen waits for the client's 26/6 choice"
    );
    release_end_screen(game, at, quest, GEOFFREY, end_text, now + 2)
}

/// Accept, scout and turn in E5, collecting every production reply the path sent.
fn complete_rumors(
    game: &mut Gameplay,
    at_geoffrey: &CharacterState,
    at_cargug: &CharacterState,
    now: &mut u64,
) -> Vec<Vec<u8>> {
    let mut replies = Vec::new();
    let offered = offer(game, at_geoffrey, GEOFFREY, RUMORS, *now);
    replies.extend(offered.replies);
    *now += 2;
    let accepted = accept(game, at_geoffrey, RUMORS, *now);
    replies.extend(accepted.replies);
    *now += 100;
    let visited = game
        .on_location(at_cargug, *now)
        .expect("the scout position is accepted");
    assert!(visited.changed, "the scout advances E5");
    assert_eq!(counts(game, RUMORS), [1, 0]);
    replies.extend(visited.replies);
    *now += 100;
    let end = open_end_screen(game, at_geoffrey, RUMORS, RUMORS_END, *now);
    assert_eq!(end_screen_reward(&end), (REWARD_COINS, REWARD_XP));
    replies.push(end);
    *now += 100;
    let paid = finish_quest(
        game,
        at_geoffrey,
        RUMORS,
        ADVENTURER,
        REWARD_COINS,
        REWARD_XP,
        *now,
    );
    replies.extend(paid.replies);
    *now += 100;
    replies
}

fn capture(game: &Gameplay) -> (GameplayState, Vec<u8>) {
    (game.state(), saved(game))
}

fn assert_unchanged(game: &Gameplay, before: &(GameplayState, Vec<u8>), label: &str) {
    assert_eq!(
        &game.state(),
        &before.0,
        "{label} changed the gameplay state"
    );
    assert_eq!(&saved(game), &before.1, "{label} changed the save bytes");
}

#[test]
fn rumors_of_briarwood_is_offered_after_thugamugs_thievery() {
    // The record: E5's text ids, a prerequisite on the playable E4, the copied
    // Cargug anchor, the return Talk and the small-referral reward.
    let record = quest_record(RUMORS);
    assert_eq!(record["enabled"], json!(true));
    assert_eq!(record["title_string_id"], json!(RUMORS_TITLE));
    assert_eq!(record["description_string_id"], json!(RUMORS_DESCRIPTION));
    assert_eq!(record["intro_text_string_id"], json!(RUMORS_INTRO));
    assert_eq!(record["end_text_string_id"], json!(RUMORS_END));
    assert_eq!(record["giver_npc_id"], json!(GEOFFREY));
    assert_eq!(record["prerequisite_quest_ids"], json!([THIEVERY]));
    let objectives = record["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 2);
    assert_eq!(objectives[0]["kind"], json!("visit_position"));
    assert_eq!(objectives[0]["id"], json!(1));
    assert_eq!(objectives[0]["zone_id"], json!(0));
    let authored_anchor: [f32; 3] =
        serde_json::from_value(objectives[0]["position"].clone()).expect("finite visit position");
    assert_eq!(authored_anchor, CARGUG_POSITION);
    assert_eq!(objectives[0]["radius"], json!(VISIT_RADIUS));
    assert_eq!(objectives[1]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[1]["id"], json!(2));
    assert_eq!(objectives[1]["npc_id"], json!(GEOFFREY));
    for objective in objectives {
        assert!(
            objective["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty()),
            "every objective carries provenance"
        );
    }
    let rewards = record["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(REWARD_COINS));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"1": REWARD_XP}));
    assert!(
        rewards[0]["source"]
            .as_array()
            .is_some_and(|source| !source.is_empty()),
        "the reward row carries provenance"
    );

    // The anchor is the shipped Sergeant Cargug legacy_data row, copied verbatim.
    let cargug = npc_row("briarwood.json", CARGUG);
    assert_eq!(cargug["name"], json!("Sergeant Cargug"));
    assert_eq!(cargug["placement"], json!("legacy_data"));
    assert!(
        cargug["roles"]
            .as_array()
            .is_some_and(|roles| roles.iter().any(|role| role == "quest_giver")),
        "the anchor is an established quest giver"
    );
    assert_eq!(row_position(&cargug), CARGUG_POSITION);

    // E4 stays enabled; E2/E3 stay disabled and hollow, so the chain can only be
    // E4 -> E5 rather than E2/E3 -> E5.
    assert_eq!(quest_record(THIEVERY)["enabled"], json!(true));
    for skipped in SKIPPED {
        let row = quest_record(skipped);
        assert_eq!(row["enabled"], json!(false), "{skipped} must stay disabled");
        assert_eq!(row["objectives"], json!([]));
        assert_eq!(row["rewards"], json!([]));
    }

    // Before E4 completes, Geoffrey's next tell is E4 itself, never E5.
    let fixture = Fixture::minimal(CHARACTER);
    let at = character(CHARACTER, geoffrey_position());
    let mut game = player(
        &fixture,
        CHARACTER,
        completed(CHARACTER, &[(MISSING_SISTERS, vec![1, 3, 1])]),
    );
    let mut now = 1_000;
    let offered = offer(&mut game, &at, GEOFFREY, THIEVERY, now);
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(RUMORS))),
        "E5 must not be offered before E4 completes"
    );

    // After E4 completes, one 26/8 -> 26/10 -> 26/6 opens the 49/1 for E5 and not E6.
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, chapter_prerequisites(CHARACTER));
    now = 1_000;
    let offered = offer(&mut game, &at, GEOFFREY, RUMORS, now);
    let decoded = decoded_offer(&offered, RUMORS);
    assert_eq!(decoded.intro_text_string_id, RUMORS_INTRO);
    assert_eq!(decoded.name_string_id, RUMORS_TITLE);
    assert_eq!(decoded.description_string_id, RUMORS_DESCRIPTION);
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(SHROUDED))),
        "E6 must wait for E5 to complete"
    );
    assert_eq!(pending_offer(&game), Some(RUMORS));
    assert_eq!(quest_status(&game, RUMORS), QuestStatus::Offered);
    assert!(
        game.state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .get(&SHROUDED)
            .is_none(),
        "E6 has no progress before E5 completes"
    );
}

#[test]
fn the_briarwood_scout_and_return_pay_once_and_survive_reload() {
    let fixture = Fixture::minimal(CHARACTER);
    let at_geoffrey = character(CHARACTER, geoffrey_position());
    let at_cargug = character(CHARACTER, CARGUG_POSITION);
    let mut game = player(&fixture, CHARACTER, chapter_prerequisites(CHARACTER));
    let mut now = 1_000;

    offer(&mut game, &at_geoffrey, GEOFFREY, RUMORS, now);
    accept(&mut game, &at_geoffrey, RUMORS, now + 2);
    assert_eq!(counts(&game, RUMORS), [0, 0]);

    // Mid-quest reconnect: the accepted quest and its zero counters survive.
    let mut game = save_and_reload(
        &fixture,
        CHARACTER,
        &game,
        &at_geoffrey,
        "rumors-mid-quest.json",
    );
    assert_eq!(quest_status(&game, RUMORS), QuestStatus::Active);
    assert_eq!(counts(&game, RUMORS), [0, 0]);

    // The server pose at the copied Cargug anchor completes objective 1 and
    // activates the return Talk (49/9 -> 49/10 -> 49/8).
    now += 100;
    let visited = game
        .on_location(&at_cargug, now)
        .expect("the scout position is accepted");
    assert!(visited.changed, "the scout position advances E5");
    assert_eq!(counts(&game, RUMORS), [1, 0]);
    assert!(objective_update(&visited, RUMORS, 1, 1));
    assert!(objective_complete(&visited, RUMORS, 1));
    assert!(objective_activated(&visited, RUMORS, 2));

    // A second visit cannot double-count a completed visit objective.
    let again = game
        .on_location(&at_cargug, now + 1)
        .expect("the repeated position is accepted");
    assert!(!again.changed, "a completed visit does not advance again");
    assert_eq!(counts(&game, RUMORS), [1, 0]);

    // The return Talk completes objective 2 and leaves the 49/13 pending for 26/6.
    now += 100;
    let talked = talk(&mut game, &at_geoffrey, GEOFFREY, now);
    assert!(talked.changed, "the return Talk advances E5");
    assert_eq!(counts(&game, RUMORS), [1, 1]);
    assert_eq!(pending_turnin(&game), Some(RUMORS));
    assert!(
        !talked
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the end screen waits for the client's 26/6"
    );
    let coins_before_pay = coins(&game);

    // Pending turn-in reconnect: the saved pending screen reopens at Geoffrey.
    let mut game = save_and_reload(
        &fixture,
        CHARACTER,
        &game,
        &at_geoffrey,
        "rumors-pending.json",
    );
    assert_eq!(pending_turnin(&game), Some(RUMORS));
    assert_eq!(counts(&game, RUMORS), [1, 1]);
    now += 100;
    let end = open_end_screen(&mut game, &at_geoffrey, RUMORS, RUMORS_END, now);
    assert_eq!(end_screen_reward(&end), (REWARD_COINS, REWARD_XP));
    assert_eq!(
        coins(&game),
        coins_before_pay,
        "the end screen alone pays nothing"
    );

    // A job switch before 49/14 must not move the configured profile-1 XP.
    now += 100;
    let switched = send(&mut game, &at_geoffrey, &select_job(MINER), now);
    assert!(switched.changed, "the Miner outfit is selectable");
    assert_eq!(active_job(&game), MINER);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    let miner_before = profile_xp(&game, MINER);
    let brawler_before = profile_xp(&game, BRAWLER);
    let coins_before = coins(&game);
    let items_before = game.state().inventory.items.clone();
    finish_quest(
        &mut game,
        &at_geoffrey,
        RUMORS,
        ADVENTURER,
        REWARD_COINS,
        REWARD_XP,
        now + 100,
    );
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + REWARD_XP);
    assert_eq!(profile_xp(&game, MINER), miner_before);
    assert_eq!(profile_xp(&game, BRAWLER), brawler_before);
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(
        game.state().inventory.items,
        items_before,
        "the chapter grants no item"
    );
    assert_eq!(quest_status(&game, RUMORS), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);

    // The completed payout survives the save, and a replayed 49/14 pays nothing.
    let mut reloaded = save_and_reload(
        &fixture,
        CHARACTER,
        &game,
        &at_geoffrey,
        "rumors-completed.json",
    );
    assert_eq!(quest_status(&reloaded, RUMORS), QuestStatus::Completed);
    assert_eq!(coins(&reloaded), coins_before + REWARD_COINS);
    assert_eq!(
        profile_xp(&reloaded, ADVENTURER),
        adventurer_before + REWARD_XP
    );
    let before = capture(&reloaded);
    assert_packet_changes_nothing(
        &mut reloaded,
        &at_geoffrey,
        &quest_end_reply(RUMORS),
        now + 200,
        "a replayed 49/14 after the payout",
    );
    assert_unchanged(&reloaded, &before, "a replayed 49/14 after the payout");
    assert_eq!(pending_turnin(&reloaded), None);
}

#[test]
fn shrouded_history_is_offered_after_the_briarwood_scout() {
    let record = quest_record(SHROUDED);
    assert_eq!(record["enabled"], json!(true));
    assert_eq!(record["title_string_id"], json!(SHROUDED_TITLE));
    assert_eq!(record["description_string_id"], json!(SHROUDED_DESCRIPTION));
    assert_eq!(record["intro_text_string_id"], json!(SHROUDED_INTRO));
    assert_eq!(record["end_text_string_id"], json!(SHROUDED_END));
    assert_eq!(record["giver_npc_id"], json!(GEOFFREY));
    assert_eq!(record["prerequisite_quest_ids"], json!([RUMORS]));
    let objectives = record["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 2);
    assert_eq!(objectives[0]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[0]["npc_id"], json!(CAMELLIA));
    assert_eq!(objectives[1]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[1]["npc_id"], json!(GEOFFREY));
    for objective in objectives {
        assert!(
            objective["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty()),
            "every objective carries provenance"
        );
    }
    let rewards = record["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(REWARD_COINS));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"1": REWARD_XP}));

    let camellia = npc_row("shrouded_glade.json", CAMELLIA);
    assert_eq!(camellia["name"], json!("Arch Druid Camellia"));
    assert_eq!(camellia["placement"], json!("legacy_data"));
    assert_eq!(row_position(&camellia), CAMELLIA_POSITION);

    let fixture = Fixture::minimal(CHARACTER);
    let at_geoffrey = character(CHARACTER, geoffrey_position());
    let at_cargug = character(CHARACTER, CARGUG_POSITION);
    let at_camellia = character(CHARACTER, CAMELLIA_POSITION);
    let mut game = player(&fixture, CHARACTER, chapter_prerequisites(CHARACTER));
    let mut now = 1_000;

    // Before E5 completes, Geoffrey's next tell is E5; E6 is not offered.
    let offered = offer(&mut game, &at_geoffrey, GEOFFREY, RUMORS, now);
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(SHROUDED))),
        "E6 must not be offered before E5 completes"
    );
    accept(&mut game, &at_geoffrey, RUMORS, now + 2);

    // Complete E5 through its real scout and turn-in.
    now += 100;
    let visited = game
        .on_location(&at_cargug, now)
        .expect("the scout position is accepted");
    assert!(visited.changed);
    assert_eq!(counts(&game, RUMORS), [1, 0]);
    now += 100;
    let end = open_end_screen(&mut game, &at_geoffrey, RUMORS, RUMORS_END, now);
    assert_eq!(end_screen_reward(&end), (REWARD_COINS, REWARD_XP));
    now += 100;
    finish_quest(
        &mut game,
        &at_geoffrey,
        RUMORS,
        ADVENTURER,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(quest_status(&game, RUMORS), QuestStatus::Completed);

    // Now E6 is the next eligible tell at Geoffrey.
    now += 100;
    let offered = offer(&mut game, &at_geoffrey, GEOFFREY, SHROUDED, now);
    let decoded = decoded_offer(&offered, SHROUDED);
    assert_eq!(decoded.intro_text_string_id, SHROUDED_INTRO);
    assert_eq!(decoded.name_string_id, SHROUDED_TITLE);
    assert_eq!(decoded.description_string_id, SHROUDED_DESCRIPTION);
    accept(&mut game, &at_geoffrey, SHROUDED, now + 2);
    assert_eq!(counts(&game, SHROUDED), [0, 0]);

    // Mid-quest reconnect: the druid step is still waiting.
    let mut game = save_and_reload(
        &fixture,
        CHARACTER,
        &game,
        &at_geoffrey,
        "shrouded-mid-quest.json",
    );
    assert_eq!(quest_status(&game, SHROUDED), QuestStatus::Active);
    assert_eq!(counts(&game, SHROUDED), [0, 0]);

    // The shipped Arch Druid Camellia actor completes the druid-records step.
    now += 100;
    let talked = talk(&mut game, &at_camellia, CAMELLIA, now);
    assert!(talked.changed, "Camellia advances the druid step");
    assert_eq!(counts(&game, SHROUDED), [1, 0]);
    assert!(objective_update(&talked, SHROUDED, 1, 1));
    assert!(objective_complete(&talked, SHROUDED, 1));
    assert!(objective_activated(&talked, SHROUDED, 2));

    // Talking to her again changes nothing: objective 1 is done and she offers no quest.
    let mut repeat_now = now + 10;
    assert_no_offer(
        &mut game,
        &at_camellia,
        CAMELLIA,
        &mut repeat_now,
        "a repeated druid Talk",
    );
    assert_eq!(counts(&game, SHROUDED), [1, 0]);

    // Return Talk to Geoffrey completes objective 2 and holds the 49/13.
    now += 100;
    let talked = talk(&mut game, &at_geoffrey, GEOFFREY, now);
    assert!(talked.changed, "the return Talk advances E6");
    assert_eq!(counts(&game, SHROUDED), [1, 1]);
    assert_eq!(pending_turnin(&game), Some(SHROUDED));

    // Pending turn-in reconnect: the screen reopens at Geoffrey and pays once.
    let mut game = save_and_reload(
        &fixture,
        CHARACTER,
        &game,
        &at_geoffrey,
        "shrouded-pending.json",
    );
    assert_eq!(pending_turnin(&game), Some(SHROUDED));
    now += 100;
    let end = open_end_screen(&mut game, &at_geoffrey, SHROUDED, SHROUDED_END, now);
    assert_eq!(end_screen_reward(&end), (REWARD_COINS, REWARD_XP));
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, ADVENTURER);
    let items_before = game.state().inventory.items.clone();
    now += 100;
    finish_quest(
        &mut game,
        &at_geoffrey,
        SHROUDED,
        ADVENTURER,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + REWARD_XP);
    assert_eq!(game.state().inventory.items, items_before);
    assert_eq!(quest_status(&game, SHROUDED), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);

    // The completed chapter survives a real save; replaying 49/14 pays nothing.
    let mut reloaded = save_and_reload(
        &fixture,
        CHARACTER,
        &game,
        &at_geoffrey,
        "shrouded-completed.json",
    );
    assert_eq!(quest_status(&reloaded, SHROUDED), QuestStatus::Completed);
    assert_eq!(coins(&reloaded), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&reloaded, ADVENTURER), xp_before + REWARD_XP);
    let before = capture(&reloaded);
    assert_packet_changes_nothing(
        &mut reloaded,
        &at_geoffrey,
        &quest_end_reply(SHROUDED),
        now + 200,
        "a replayed 49/14 after the payout",
    );
    assert_unchanged(&reloaded, &before, "a replayed 49/14 after the payout");
}

#[test]
fn a_wrong_position_or_npc_or_forged_turn_in_changes_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let at_geoffrey = character(CHARACTER, geoffrey_position());
    let at_cargug = character(CHARACTER, CARGUG_POSITION);
    let mut game = player(&fixture, CHARACTER, chapter_prerequisites(CHARACTER));
    let mut now = 1_000;
    offer(&mut game, &at_geoffrey, GEOFFREY, RUMORS, now);
    accept(&mut game, &at_geoffrey, RUMORS, now + 2);

    // Nine metres east of the anchor is outside the 8 m radius.
    let outside = character(
        CHARACTER,
        [
            CARGUG_POSITION[0] + 9.0,
            CARGUG_POSITION[1],
            CARGUG_POSITION[2],
        ],
    );
    let before = capture(&game);
    let moved = game
        .on_location(&outside, now + 10)
        .expect("the outside position is accepted");
    assert!(!moved.changed, "a position outside 8 m is not the scout");
    assert_unchanged(&game, &before, "a visit outside the 8 m radius");
    assert_eq!(counts(&game, RUMORS), [0, 0]);

    // The same point in the NPC row's authored regional zone is not the open-world
    // anchor. (The visit objective is authored in local zone 0.)
    let mut wrong_zone = at_cargug.clone();
    wrong_zone.zone_id = 2;
    let before = capture(&game);
    let moved = game
        .on_location(&wrong_zone, now + 20)
        .expect("the regional-zone position is accepted");
    assert!(!moved.changed, "another zone is not the scout anchor");
    assert_unchanged(&game, &before, "a visit in another zone");
    assert_eq!(counts(&game, RUMORS), [0, 0]);

    // A nearby non-target NPC (Cargug is a quest giver with no bound quest) cannot
    // advance the scout or consume the open menu.
    let before = capture(&game);
    assert_no_offer(
        &mut game,
        &at_cargug,
        CARGUG,
        &mut now,
        "a nearby non-target NPC",
    );
    assert_unchanged(&game, &before, "a nearby non-target NPC");
    assert_eq!(counts(&game, RUMORS), [0, 0]);

    // Forged accepts and premature turn-ins are clean refusals.
    let before = capture(&game);
    assert_packet_changes_nothing(
        &mut game,
        &at_geoffrey,
        &quest_reply(RUMORS, true),
        now + 30,
        "a forged 49/2",
    );
    assert_unchanged(&game, &before, "a forged 49/2");

    let before = capture(&game);
    assert_packet_changes_nothing(
        &mut game,
        &at_geoffrey,
        &quest_end_reply(RUMORS),
        now + 40,
        "a 49/14 before the scout",
    );
    assert_unchanged(&game, &before, "a 49/14 before the scout");

    let before = capture(&game);
    assert_packet_changes_nothing(
        &mut game,
        &at_geoffrey,
        &quest_end_reply(SHROUDED),
        now + 50,
        "a 49/14 for an unstarted quest",
    );
    assert_unchanged(&game, &before, "a 49/14 for an unstarted quest");

    // After the scout but before the return Talk the quest is not ready yet.
    now += 100;
    let visited = game
        .on_location(&at_cargug, now)
        .expect("the scout position is accepted");
    assert!(visited.changed);
    assert_eq!(counts(&game, RUMORS), [1, 0]);
    let before = capture(&game);
    assert_packet_changes_nothing(
        &mut game,
        &at_geoffrey,
        &quest_end_reply(RUMORS),
        now + 10,
        "a premature 49/14",
    );
    assert_unchanged(&game, &before, "a premature 49/14");
    assert_eq!(counts(&game, RUMORS), [1, 0]);

    // A near-cap wallet makes the content's 8-coin grant overflow the client's
    // i32 coin field. The refusal must not pay XP or an item, must leave the
    // pending screen retryable, and must not change one serialized byte. (No
    // reward row has an item, so bag capacity is not consulted by this grant.)
    let near_cap = i32::MAX as u32 - (REWARD_COINS - 1);
    let mut capped = player_with_coins(
        &fixture,
        CHARACTER,
        chapter_prerequisites(CHARACTER),
        Some(near_cap),
    );
    let capped_now = 1_000;
    offer(&mut capped, &at_geoffrey, GEOFFREY, RUMORS, capped_now);
    accept(&mut capped, &at_geoffrey, RUMORS, capped_now + 2);
    capped
        .on_location(&at_cargug, capped_now + 100)
        .expect("the scout position is accepted");
    let talked = talk(&mut capped, &at_geoffrey, GEOFFREY, capped_now + 200);
    assert!(talked.changed);
    assert_eq!(pending_turnin(&capped), Some(RUMORS));
    let items_before = capped.state().inventory.items.clone();
    let before = capture(&capped);
    let _ = capped.handle_in_world(&quest_end_reply(RUMORS), &at_geoffrey, capped_now + 300);
    assert_eq!(coins(&capped), near_cap, "a refused grant pays no coins");
    assert_eq!(
        profile_xp(&capped, ADVENTURER),
        0,
        "a refused grant pays no XP"
    );
    assert_eq!(
        capped.state().inventory.items,
        items_before,
        "a refused grant pays no item"
    );
    assert_eq!(quest_status(&capped, RUMORS), QuestStatus::Active);
    assert_eq!(
        pending_turnin(&capped),
        Some(RUMORS),
        "the pending screen stays retryable"
    );
    assert_unchanged(&capped, &before, "an overflowing reward grant");
}

#[test]
fn two_players_each_own_the_chain() {
    let fixture = Fixture::minimal(CHARACTER);
    let a_at = character(CHARACTER, geoffrey_position());
    let b_at = character(OTHER, geoffrey_position());
    let mut a = player(&fixture, CHARACTER, chapter_prerequisites(CHARACTER));
    let mut b = player(&fixture, OTHER, completed(OTHER, &[]));

    // B is a fresh Adventurer: E5 is neither offered nor paid to B by A's run.
    let b_before = capture(&b);
    let a_at_cargug = character(CHARACTER, CARGUG_POSITION);
    let mut now = 1_000;
    complete_rumors(&mut a, &a_at, &a_at_cargug, &mut now);
    assert_eq!(quest_status(&a, RUMORS), QuestStatus::Completed);
    assert_unchanged(&b, &b_before, "A's chapter");
    assert!(
        b.state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .get(&RUMORS)
            .is_none(),
        "A's accepted chain is not B's"
    );
    let b_coins = coins(&b);
    let b_xp = profile_xp(&b, ADVENTURER);

    // B's own first tell is E1; A's completed E5 is not offered.
    let offered = offer(&mut b, &b_at, GEOFFREY, MISSING_SISTERS, now + 100);
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(RUMORS))),
        "A's completion must not open E5 for B"
    );
    assert_eq!(coins(&b), b_coins, "A's payout did not pay B");
    assert_eq!(profile_xp(&b, ADVENTURER), b_xp);
}

#[test]
fn the_chapter_uses_no_dev_commands_and_leaves_the_character_free() {
    let fixture = Fixture::minimal(CHARACTER);
    let at_geoffrey = character(CHARACTER, geoffrey_position());
    let at_cargug = character(CHARACTER, CARGUG_POSITION);
    let mut game = player(&fixture, CHARACTER, chapter_prerequisites(CHARACTER));
    let mut now = 1_000;
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, ADVENTURER);
    let items_before = game.state().inventory.items.clone();

    // Only the production quest/menu packets drive the chapter: no realm chat
    // command, no dev teleport, no `realm level`/`realm coins`.
    let replies = complete_rumors(&mut game, &at_geoffrey, &at_cargug, &mut now);
    for packet in &replies {
        assert!(
            !packet.starts_with(&[38, 0, 12, 0]),
            "the chapter must not send a teleport/update-location packet"
        );
    }

    // The turn-in paid exactly its content reward and left nothing open.
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + REWARD_XP);
    assert_eq!(game.state().inventory.items, items_before);
    assert_eq!(quest_status(&game, RUMORS), QuestStatus::Completed);
    assert_eq!(pending_offer(&game), None);
    assert_eq!(pending_turnin(&game), None);
    assert!(
        game.state().encounter.is_none(),
        "the chapter leaves no encounter or battle open"
    );

    // Nothing is pending: an unrelated location event changes nothing.
    let far = character(CHARACTER, [10_000.0, 0.0, 10_000.0]);
    let moved = game
        .on_location(&far, now + 10)
        .expect("an unrelated location is accepted");
    assert!(
        !moved.changed,
        "the finished chapter leaves no pending goal"
    );

    // The character is free to start the next chapter beat.
    let offered = offer(&mut game, &at_geoffrey, GEOFFREY, SHROUDED, now + 100);
    assert_eq!(pending_offer(&game), Some(SHROUDED));
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 12, 0])),
        "the next offer is not a teleport"
    );
}
