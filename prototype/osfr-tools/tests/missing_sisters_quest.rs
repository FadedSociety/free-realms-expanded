//! T279: "Chapter 1: The Missing Sisters" is a playable story beat and Geoffrey
//! (NPC 9000000330) is a drawable Royal Palace actor.
//!
//! The quest record (3000029) and the drawable spawn row live in
//! `data/content/quests/adventurer.json` and `data/content/npc_spawns_extra.json`.
//! These tests drive the production paths: `world::NpcVisibility` (the NPC loader
//! and the directed 35/2), `Gameplay::handle_in_world` (26/8 -> 26/10 -> 26/6 ->
//! 49/1 -> 49/2), `Gameplay::on_location` (movement -> VisitPosition),
//! `Gameplay::handle_shared_field` (the shared field's basic attack and T256 kill
//! credit), and the ordinary 49/13 -> 49/14 turn-in. The shared fixture and packet
//! builders live in `tests/support/npc_quest_chain.rs`.
//!
//! Client layouts used here (IDA):
//! - 35/2 AddNpc body reader `0x00921a10` (actor u64, name id, model id, transform).
//! - 26/8 interact `0x008b6bc0`; 26/10 select `0x00a36f80`; 26/6 dialog `0x009f5890`.
//! - 49/1 offer reader `0x00c7bb60`; 49/2 reply `0x00a86400`; 49/13 end reader
//!   `0x00c7b990` / `0x00c7bbd0`; 49/14 reply `0x00a864c0`.
//! - 41/114 encounter result reader `0x009bdd70`; the wolf kills use the existing
//!   shared-field attack path (36/10) and add no packet.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, finish_quest, has_family,
    kill_shared_target, npc_interact, objective_update, offer, pending_offer, pending_turnin,
    profile_xp, quest_end_reply, quest_reply, quest_status, read_json, release_end_screen, saved,
    select_job, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    game_store,
    gameplay::{Gameplay, GameplayState, Outcome},
    profiles::ProfileState,
    quest_engine::{CharacterEligibility, QuestEngine, QuestState, QuestStatus},
    startup_world, world,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const GEOFFREY: u64 = 9_000_000_330;
const OTHER: u64 = CHARACTER + 1;
/// A placed quest giver with no live quest, for the wrong-NPC refusal. Anne
/// 9500000266 is no longer usable here: T360 gives her quest 3000061.
const BO: u64 = 9_500_000_268;
const QUEST: u32 = 3_000_029;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
const WOLF: u64 = 3_156;
const QUEST_COINS: u32 = 5;
const QUEST_XP: u32 = 25;
const INTRO: u32 = 90_621;
const TITLE: u32 = 90_622;
const DESCRIPTION: u32 = 90_623;
const END_TEXT: u32 = 90_624;
/// The recovered `Global.Text` row for "Geoffrey"; the NPC row's 509620257 does not resolve.
const GEOFFREY_NAME_ID: u32 = 70_318;
const NPC_ROW_NAME_ID: u64 = 509_620_257;
const GEOFFREY_MODEL: i32 = 415;
/// `starting_farnum_wolves` W102-F1 centre; the quest's visit anchor.
const WOLF_CENTRE: [f32; 3] = [-1899.204, -50.59, 307.902];

fn character(guid: u64, position: [f32; 3]) -> CharacterState {
    character_at_for(guid, 0, position)
}

fn geoffrey_position() -> [f32; 3] {
    content_position("sanctuary.json", GEOFFREY)
}

/// The shipped extra-spawn row for Geoffrey.
fn geoffrey_spawn() -> Value {
    read_json(&content_dir().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["spawn_guid"].as_u64() == Some(GEOFFREY))
        .cloned()
        .expect("Geoffrey's drawable spawn row")
}

/// The unchanged Sanctuary NPC row (identity, giver, reconstructed position).
fn geoffrey_npc() -> Value {
    read_json(&content_dir().join("npcs/sanctuary.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(GEOFFREY))
        .cloned()
        .expect("the Sanctuary Geoffrey row")
}

fn spawn_position(spawn: &Value) -> [f32; 3] {
    serde_json::from_value(spawn["position"].clone()).expect("finite spawn position")
}

fn row_position(row: &Value) -> [f32; 3] {
    serde_json::from_value(row["position"].clone()).expect("finite NPC position")
}

/// The three live `starting_farnum_wolves` actors (definition 3156). The 180 HP /
/// 25 damage rows are the separate level-5 `mid_farnum_wolves` camp and are not
/// this quest's "three wolves".
fn starting_wolves() -> Vec<(u64, [f32; 3])> {
    let document = read_json(&content_dir().join("field_enemies.json"));
    let group = document["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["id"] == "starting_farnum_wolves")
        .expect("the starting_farnum_wolves group");
    assert_eq!(group["respawn_seconds"], json!(45));
    let members: BTreeSet<u64> = group["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_u64().unwrap())
        .collect();
    let mut wolves: Vec<(u64, [f32; 3])> = document["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| members.contains(&enemy["guid"].as_u64().unwrap()))
        .map(|enemy| {
            assert_eq!(enemy["definition_id"].as_u64(), Some(WOLF));
            assert_eq!(enemy["level"], json!(1));
            (
                enemy["guid"].as_u64().unwrap(),
                serde_json::from_value(enemy["position"].clone()).unwrap(),
            )
        })
        .collect();
    wolves.sort_by_key(|(guid, _)| *guid);
    assert_eq!(
        wolves.len(),
        3,
        "the group has exactly three starter wolves"
    );
    for enemy in document["enemies"].as_array().unwrap() {
        if members.contains(&enemy["guid"].as_u64().unwrap()) {
            assert_eq!(enemy["max_health"], json!(100), "starter wolf HP");
            assert_eq!(enemy["damage"], json!(7), "starter wolf damage");
        }
    }
    assert_eq!(wolves[0].1, WOLF_CENTRE, "W102-F1 is the visit anchor");
    wolves
}

/// The two level-5 camp wolves; B uses one after A has killed the starter three.
fn mid_wolves() -> Vec<(u64, [f32; 3])> {
    let document = read_json(&content_dir().join("field_enemies.json"));
    let group = document["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["id"] == "mid_farnum_wolves")
        .expect("the mid_farnum_wolves group");
    let members: BTreeSet<u64> = group["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_u64().unwrap())
        .collect();
    document["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| members.contains(&enemy["guid"].as_u64().unwrap()))
        .map(|enemy| {
            assert_eq!(enemy["definition_id"].as_u64(), Some(WOLF));
            assert_eq!(enemy["level"], json!(5));
            (
                enemy["guid"].as_u64().unwrap(),
                serde_json::from_value(enemy["position"].clone()).unwrap(),
            )
        })
        .collect()
}

/// A player at a chosen position with explicit profiles. Every test starts with the
/// opening fixture's inventory and the repository's split quest/NPC content.
fn player(
    fixture: &Fixture,
    character_id: u64,
    active: i32,
    unlocked: &[i32],
    position: [f32; 3],
) -> Gameplay {
    let base = Gameplay::load(fixture.path(), None).expect("the opening fixture loads");
    let mut state = base.state();
    state.profiles = Some(ProfileState {
        version: 1,
        active,
        unlocked: unlocked.iter().copied().collect(),
        experience: BTreeMap::new(),
    });
    let mut game = fixture.game_as(character_id, Some(state));
    game.initialize_world(&character(character_id, position), 0)
        .expect("the opening world initializes");
    game
}

/// A Talk to Geoffrey followed by the 26/6 that releases the held 49/13.
fn open_end_screen(game: &mut Gameplay, at: &CharacterState, now: u64) -> Vec<u8> {
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
        "the end screen waits for the 26/6 choice"
    );
    release_end_screen(game, at, QUEST, GEOFFREY, END_TEXT, now + 2)
}

/// 49/13: actor 6..14, quest 14..18, end text 18..22, bundle success 26, coins
/// 27..31, displayed XP 31..35 (client reader `0x00c7b990`, `008e7930`).
fn assert_end_reward(packet: &[u8], coins: u32, xp: u32) {
    assert_eq!(
        packet.len(),
        99,
        "the empty-display reward bundle is 99 bytes"
    );
    assert_eq!(
        u64::from_le_bytes(packet[6..14].try_into().unwrap()),
        GEOFFREY,
        "the end screen names the clicked runtime actor"
    );
    assert_eq!(packet[26], 1, "reward bundle success flag");
    assert_eq!(
        i32::from_le_bytes(packet[27..31].try_into().unwrap()),
        coins as i32
    );
    assert_eq!(
        i32::from_le_bytes(packet[31..35].try_into().unwrap()),
        xp as i32
    );
}

fn offer_packet(outcome: &Outcome) -> &[u8] {
    outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(QUEST)))
        .map(Vec::as_slice)
        .expect("the 49/1 offer")
}

fn word(packet: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
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

fn accept_quest(game: &mut Gameplay, at: &CharacterState, now: u64) {
    offer(game, at, GEOFFREY, QUEST, now);
    accept(game, at, QUEST, now + 2);
}

fn active_quests(content: &Path, character_id: u64) -> QuestState {
    let eligibility = CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::from([("adventurer".into(), 20)]),
        member: true,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    };
    let mut engine =
        QuestEngine::from_split_content(content, character_id, eligibility, None).unwrap();
    assert!(
        engine.offer(QUEST, GEOFFREY).changed,
        "the quest is offered"
    );
    assert!(engine.accept(QUEST).changed, "the quest is accepted");
    engine.state().clone()
}

#[test]
fn geoffrey_is_drawable_and_offers_the_missing_sisters() {
    let npc = geoffrey_npc();
    assert_eq!(npc["placement"], json!("unanchored"));
    assert_eq!(npc["name"], json!("Geoffrey"));
    assert_eq!(npc["name_string_id"], json!(NPC_ROW_NAME_ID));
    assert_eq!(npc["zone_id"], json!(10));

    // The client's text table resolves the spawn's name id; the NPC row's ucdt
    // id does not (T201's AddNpc name-field lesson). The NPC row stays untouched.
    let locale: Value = read_json(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/reports/locale-id-to-text.json"),
    );
    let locale = locale.as_array().unwrap();
    assert!(
        locale.iter().any(
            |row| row["id"].as_u64() == Some(u64::from(GEOFFREY_NAME_ID))
                && row["text"] == json!("Geoffrey")
        ),
        "Global.Text 70318 is Geoffrey"
    );
    assert!(
        !locale
            .iter()
            .any(|row| row["id"].as_u64() == Some(NPC_ROW_NAME_ID)),
        "the NPC row's 509620257 is not a Global.Text id"
    );

    // The production NPC loader draws him for a nearby viewer. The extra spawn
    // row is the loader input; with it removed there is no AddNpc for Geoffrey.
    let mut visibility = world::NpcVisibility::load(&content_dir()).unwrap();
    assert_eq!(visibility.content_key(GEOFFREY), Some(GEOFFREY));
    let drawn = visibility
        .update(0, row_position(&npc))
        .unwrap()
        .iter()
        .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == GEOFFREY)
        .expect("Geoffrey's AddNpc for a nearby viewer");
    assert_eq!(drawn.model_id, GEOFFREY_MODEL);
    assert_eq!(drawn.name_string_id, GEOFFREY_NAME_ID as i32);
    assert_eq!(
        [drawn.position[0], drawn.position[1], drawn.position[2]],
        row_position(&npc)
    );

    // The drawn row carries the NPC's identity and the recovered model/name ids.
    let spawn = geoffrey_spawn();
    assert_eq!(spawn["content_key"].as_u64(), Some(GEOFFREY));
    assert!(spawn["legacy_npc_id"].is_null());
    assert!(spawn["capture_match"].is_null());
    assert_eq!(spawn["model_id"], json!(GEOFFREY_MODEL));
    assert_eq!(spawn["model_file_name"], json!("fairy_m_geoffrey.adr"));
    assert_eq!(spawn["name"], json!("Geoffrey"));
    assert_eq!(spawn["name_string_id"], json!(GEOFFREY_NAME_ID));
    assert!(spawn["heading"].as_f64().is_some_and(f64::is_finite));
    assert_eq!(row_position(&npc), spawn_position(&spawn));

    // A character on a non-Adventurer job receives exactly one 49/1. The quest has
    // no `job` field; the Adventurer job's quest list is only the content home.
    let fixture = Fixture::for_character(CHARACTER);
    let at = character(CHARACTER, spawn_position(&spawn));
    let mut game = player(
        &fixture,
        CHARACTER,
        MINER,
        &[ADVENTURER, MINER, BRAWLER],
        at.position,
    );
    let mut now = 1_000;
    let offered = offer(&mut game, &at, GEOFFREY, QUEST, now);
    assert_eq!(
        offered
            .replies
            .iter()
            .filter(|packet| has_family(packet, 49, 1, Some(QUEST)))
            .count(),
        1,
        "one Talk offers the quest once"
    );
    let packet = offer_packet(&offered);
    assert_eq!(word(packet, 6), QUEST);
    assert_eq!(word(packet, 10), INTRO, "49/1 intro id");
    assert_eq!(word(packet, 14), TITLE, "49/1 title id");
    assert_eq!(word(packet, 18), TITLE, "49/1 title id (card)");
    assert_eq!(word(packet, 22), DESCRIPTION, "49/1 description id");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Offered);
    assert_eq!(pending_offer(&game), Some(QUEST));

    // A second Talk while offered and again while active offers no duplicate.
    now += 100;
    let before = capture(&game);
    assert_no_offer(&mut game, &at, GEOFFREY, &mut now, "a pending offer");
    assert_unchanged(&game, &before, "a pending offer Talk");
    accept(&mut game, &at, QUEST, now);
    now += 100;
    assert_no_offer(&mut game, &at, GEOFFREY, &mut now, "an active quest");
    assert_eq!(counts(&game, QUEST), [0, 0, 0]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
}

#[test]
fn the_search_visit_three_wolf_kills_and_final_talk_pay_once() {
    let fixture = Fixture::for_character(CHARACTER);
    let at_geoffrey = character(CHARACTER, geoffrey_position());
    let mut game = player(
        &fixture,
        CHARACTER,
        ADVENTURER,
        &[ADVENTURER, MINER, BRAWLER],
        geoffrey_position(),
    );
    let mut now = 1_000;
    let offered = offer(&mut game, &at_geoffrey, GEOFFREY, QUEST, now);
    let packet = offer_packet(&offered);
    assert_eq!(word(packet, 14), TITLE);
    accept(&mut game, &at_geoffrey, QUEST, now + 2);
    assert_eq!(counts(&game, QUEST), [0, 0, 0]);

    // Switch to the Brawler before the shared-field fight; the reward map still
    // names profile 1, so the active job must not decide the payer.
    now += 100;
    let switched = send(&mut game, &at_geoffrey, &select_job(BRAWLER), now);
    assert!(switched.changed, "the Brawler outfit is selectable");
    assert_eq!(active_job(&game), BRAWLER);

    // The search-party visit at the W102-F1 centre completes objective 1.
    now += 100;
    let visited = game
        .on_location(&character(CHARACTER, WOLF_CENTRE), now)
        .expect("movement is accepted");
    assert!(visited.changed, "the search-party visit advances the quest");
    assert_eq!(counts(&game, QUEST), [1, 0, 0]);

    let wolves = starting_wolves();
    let known: BTreeSet<u64> = wolves.iter().map(|(guid, _)| *guid).collect();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    for (index, (guid, position)) in wolves.iter().enumerate() {
        now += 100;
        let at = character(CHARACTER, [position[0] + 1.0, position[1], position[2]]);
        let (killed, next) = kill_shared_target(&mut game, &mut field, &known, &at, *guid, now);
        now = next;
        assert_eq!(field.health(*guid), Some(0), "wolf {index} is dead");
        assert_eq!(
            counts(&game, QUEST),
            [1, index as u32 + 1, 0],
            "kill {} credits the killer once",
            index + 1
        );
        assert!(
            objective_update(&killed, QUEST, 2, index as u32 + 1),
            "49/9 reports the objective counter"
        );
    }
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    // The return Talk to Geoffrey opens the 49/13; the reward map names profile 1.
    now += 100;
    let coins_before = coins(&game);
    let end = open_end_screen(&mut game, &at_geoffrey, now);
    assert_end_reward(&end, QUEST_COINS, QUEST_XP);
    // Confirm the end screen pays nothing by itself.
    assert_eq!(coins(&game), coins_before);

    // Switch the active job to Miner before 49/14: the XP must land on profile 1.
    now += 100;
    let switched = send(&mut game, &at_geoffrey, &select_job(MINER), now);
    assert!(switched.changed, "the Miner outfit is selectable");
    assert_eq!(active_job(&game), MINER);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    let miner_before = profile_xp(&game, MINER);
    let brawler_before = profile_xp(&game, BRAWLER);
    now += 100;
    finish_quest(
        &mut game,
        &at_geoffrey,
        QUEST,
        ADVENTURER,
        QUEST_COINS,
        QUEST_XP,
        now,
    );
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + QUEST_XP);
    assert_eq!(
        profile_xp(&game, MINER),
        miner_before,
        "the active job is not paid"
    );
    assert_eq!(profile_xp(&game, BRAWLER), brawler_before);
    assert_eq!(counts(&game, QUEST), [1, 3, 1]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}

#[test]
fn a_second_player_cannot_credit_or_pay_the_first() {
    let fixture = Fixture::for_character(CHARACTER);
    let a_at = character(CHARACTER, geoffrey_position());
    let b_at = character(OTHER, geoffrey_position());
    let mut a = player(
        &fixture,
        CHARACTER,
        ADVENTURER,
        &[ADVENTURER, BRAWLER],
        geoffrey_position(),
    );
    let mut b = player(
        &fixture,
        OTHER,
        ADVENTURER,
        &[ADVENTURER, BRAWLER],
        geoffrey_position(),
    );
    let mut now = 1_000;
    accept_quest(&mut a, &a_at, now);
    accept_quest(&mut b, &b_at, now + 10);
    assert_eq!(counts(&a, QUEST), [0, 0, 0]);
    assert_eq!(counts(&b, QUEST), [0, 0, 0]);

    // A fights on the shared field; B is present but does nothing.
    now += 100;
    send(&mut a, &a_at, &select_job(BRAWLER), now);
    a.on_location(&character(CHARACTER, WOLF_CENTRE), now + 50)
        .expect("A's movement is accepted");
    let mut field = a.take_shared_fields().expect("A owns the shared field");
    b.take_shared_fields();
    let wolves = starting_wolves();
    let known: BTreeSet<u64> = wolves.iter().map(|(guid, _)| *guid).collect();
    for (guid, position) in wolves.iter() {
        now += 100;
        let at = character(CHARACTER, [position[0] + 1.0, position[1], position[2]]);
        let (_, next) = kill_shared_target(&mut a, &mut field, &known, &at, *guid, now);
        now = next;
    }
    assert_eq!(counts(&a, QUEST), [1, 3, 0]);
    assert_eq!(
        counts(&b, QUEST),
        [0, 0, 0],
        "A's kills credit no bystander"
    );
    let b_before = capture(&b);
    assert_unchanged(&b, &b_before, "A's kills");

    // A turns in; B's wallet and profile stay untouched.
    now += 100;
    let a_coins = coins(&a);
    let b_coins = coins(&b);
    let b_xp = profile_xp(&b, ADVENTURER);
    let end = open_end_screen(&mut a, &a_at, now + 100);
    assert_end_reward(&end, QUEST_COINS, QUEST_XP);
    finish_quest(
        &mut a,
        &a_at,
        QUEST,
        ADVENTURER,
        QUEST_COINS,
        QUEST_XP,
        now + 200,
    );
    assert_eq!(coins(&a), a_coins + QUEST_COINS);
    assert_eq!(coins(&b), b_coins);
    assert_eq!(profile_xp(&b, ADVENTURER), b_xp);
    assert!(
        quest_status(&b, QUEST) == QuestStatus::Active,
        "A's turn-in does not complete B's quest"
    );

    // B independently visits and kills a still-live level-5 camp wolf (3156).
    now += 300;
    b.on_location(&character(OTHER, WOLF_CENTRE), now)
        .expect("B's movement is accepted");
    assert_eq!(counts(&b, QUEST), [1, 0, 0]);
    send(&mut b, &b_at, &select_job(BRAWLER), now + 50);
    let mids = mid_wolves();
    let (target, position) = mids[0];
    let b_known: BTreeSet<u64> = mids.iter().map(|(guid, _)| *guid).collect();
    let b_at_wolf = character(OTHER, [position[0] + 1.0, position[1], position[2]]);
    now += 100;
    let (killed, _) = kill_shared_target(&mut b, &mut field, &b_known, &b_at_wolf, target, now);
    assert_eq!(counts(&b, QUEST), [1, 1, 0], "B is credited for B's kill");
    assert!(
        objective_update(&killed, QUEST, 2, 1),
        "B's 49/9 reports B's counter"
    );
    assert_eq!(counts(&a, QUEST), [1, 3, 1], "B's kill does not change A");
}

#[test]
fn refusals_and_reconnect_change_nothing() {
    let fixture = Fixture::for_character(CHARACTER);
    let at_geoffrey = character(CHARACTER, geoffrey_position());
    let mut game = player(
        &fixture,
        CHARACTER,
        ADVENTURER,
        &[ADVENTURER, BRAWLER],
        geoffrey_position(),
    );
    let mut now = 1_000;

    // Wrong NPC: Bo's menu offers no Missing Sisters quest and the save is equal.
    let bo = character(CHARACTER, content_position("farnum_farmstead.json", BO));
    let before = capture(&game);
    assert_no_offer(&mut game, &bo, BO, &mut now, "a wrong quest giver");
    assert_unchanged(&game, &before, "a wrong quest giver");

    // Out-of-range 26/8 is refused with an error and no state change.
    let far = character(CHARACTER, [10_000.0, 0.0, 10_000.0]);
    let before = capture(&game);
    assert!(
        game.handle_in_world(&npc_interact(GEOFFREY), &far, now)
            .is_err(),
        "an out-of-range Geoffrey click is refused"
    );
    assert_unchanged(&game, &before, "an out-of-range click");

    // Forged accept and a 49/14 with no server-opened end screen are clean no-ops.
    let before = capture(&game);
    assert_packet_changes_nothing(
        &mut game,
        &at_geoffrey,
        &quest_reply(QUEST, true),
        now + 10,
        "a forged 49/2",
    );
    assert_unchanged(&game, &before, "a forged 49/2");
    let before = capture(&game);
    assert_packet_changes_nothing(
        &mut game,
        &at_geoffrey,
        &quest_end_reply(QUEST),
        now + 20,
        "a 49/14 with no end screen",
    );
    assert_unchanged(&game, &before, "a 49/14 with no end screen");

    // A real accept followed by a visit outside the 8 m radius changes nothing.
    accept_quest(&mut game, &at_geoffrey, now + 30);
    let before = capture(&game);
    let outside = character(CHARACTER, [10_000.0, 0.0, 10_000.0]);
    let moved = game
        .on_location(&outside, now + 50)
        .expect("movement is accepted");
    assert!(!moved.changed, "a far visit is not the search-party spot");
    assert_unchanged(&game, &before, "a visit outside the radius");
    assert_eq!(counts(&game, QUEST), [0, 0, 0]);

    // Complete the quest once, then reload and replay 49/14: nothing changes.
    now += 100;
    send(&mut game, &at_geoffrey, &select_job(BRAWLER), now);
    game.on_location(&character(CHARACTER, WOLF_CENTRE), now + 50)
        .expect("movement is accepted");
    let wolves = starting_wolves();
    let known: BTreeSet<u64> = wolves.iter().map(|(guid, _)| *guid).collect();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    for (guid, position) in wolves.iter() {
        now += 100;
        let at = character(CHARACTER, [position[0] + 1.0, position[1], position[2]]);
        let (_, next) = kill_shared_target(&mut game, &mut field, &known, &at, *guid, now);
        now = next;
    }
    now += 100;
    let end = open_end_screen(&mut game, &at_geoffrey, now);
    assert_end_reward(&end, QUEST_COINS, QUEST_XP);
    finish_quest(
        &mut game,
        &at_geoffrey,
        QUEST,
        ADVENTURER,
        QUEST_COINS,
        QUEST_XP,
        now + 100,
    );

    let mut reloaded = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&reloaded, QUEST), QuestStatus::Completed);
    let before = capture(&reloaded);
    assert_packet_changes_nothing(
        &mut reloaded,
        &at_geoffrey,
        &quest_end_reply(QUEST),
        now + 200,
        "a replayed 49/14 after reconnect",
    );
    assert_unchanged(&reloaded, &before, "a replayed 49/14 after reconnect");
}

#[test]
fn the_spawn_is_unique_and_the_quest_target_resolves() {
    let document = read_json(&content_dir().join("npc_spawns_extra.json"));
    let rows = document["records"].as_array().unwrap();
    let geoffrey: Vec<&Value> = rows
        .iter()
        .filter(|row| row["spawn_guid"].as_u64() == Some(GEOFFREY))
        .collect();
    assert_eq!(geoffrey.len(), 1, "exactly one Geoffrey spawn row");
    let keys: BTreeSet<u64> = rows
        .iter()
        .map(|row| row["spawn_guid"].as_u64().unwrap())
        .collect();
    assert_eq!(keys.len(), rows.len(), "spawn GUIDs stay unique");
    assert_eq!(
        rows.iter()
            .filter(|row| row["content_key"].as_u64() == Some(GEOFFREY))
            .count(),
        1,
        "no second row aliases Geoffrey's content key"
    );
    assert!(
        rows.iter()
            .any(|row| row["spawn_guid"].as_u64() == Some(9_000_000_042)),
        "the Carlos row this row was inserted after is still present"
    );

    let spawn = geoffrey[0];
    let npc = geoffrey_npc();
    assert_eq!(spawn["model_id"], json!(GEOFFREY_MODEL));
    assert_eq!(spawn["name_string_id"], json!(GEOFFREY_NAME_ID));
    assert_eq!(row_position(&npc), spawn_position(spawn));
    for needle in ["Models.txt row 415", "Assets_manifest.txt", "(R)"] {
        assert!(
            spawn["source"]
                .as_array()
                .unwrap()
                .iter()
                .any(|tag| tag["evidence"]
                    .as_str()
                    .is_some_and(|text| text.contains(needle))),
            "spawn source documents {needle:?}"
        );
    }

    // The production loader accepts the file and keeps the content key.
    let visibility = world::NpcVisibility::load(&content_dir()).unwrap();
    assert_eq!(visibility.content_key(GEOFFREY), Some(GEOFFREY));

    // Two sessions at the Royal Palace both see the one shared actor.
    let fixture = Fixture::for_character(CHARACTER);
    let a = fixture.game_as(CHARACTER, None);
    let b = fixture.game_as(OTHER, None);
    let a_at = character(CHARACTER, geoffrey_position());
    let b_at = character(OTHER, geoffrey_position());
    for (game, at) in [(&a, &a_at), (&b, &b_at)] {
        let (_, packets) = game.shared_npc_view(at, &[], &BTreeSet::new()).unwrap();
        let seen = packets
            .iter()
            .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
            .find(|npc| npc.actor_id == GEOFFREY)
            .expect("Geoffrey is drawn to every nearby session");
        assert_eq!(seen.model_id, GEOFFREY_MODEL);
        assert_eq!(seen.name_string_id, GEOFFREY_NAME_ID as i32);
    }

    // The quest target resolves to the same actor the spawn draws.
    let engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility::default(),
        None,
    )
    .unwrap();
    assert_eq!(engine.giver_npc_id(QUEST), Some(GEOFFREY));
    assert!(engine.is_enabled(QUEST));
    let enabled: Vec<u32> = engine
        .quest_ids_for_giver(GEOFFREY)
        .filter(|id| engine.is_enabled(*id))
        .collect();
    assert_eq!(
        enabled,
        vec![QUEST, 3_000_032, 3_000_033, 3_000_034],
        "only the enabled Chapter 1 rows are offered at Geoffrey"
    );
    let record = read_json(&content_dir().join("quests/adventurer.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(QUEST)))
        .cloned()
        .expect("the 3000029 record");
    assert_eq!(record["enabled"], json!(true));
    assert_eq!(record["prerequisite_quest_ids"], json!([]));
    assert!(
        record.get("job").is_none(),
        "the playtest entry has no job gate"
    );
    assert_eq!(record["objectives"].as_array().unwrap().len(), 3);
    assert_eq!(record["rewards"][0]["coins"], json!(QUEST_COINS));
    assert_eq!(record["rewards"][0]["items"], json!([]));
    assert_eq!(record["rewards"][0]["job_xp"], json!({"1": QUEST_XP}));
}

#[test]
fn progress_survives_a_reconnect() {
    let fixture = Fixture::for_character(CHARACTER);
    let at_geoffrey = character(CHARACTER, geoffrey_position());
    let mut game = player(
        &fixture,
        CHARACTER,
        ADVENTURER,
        &[ADVENTURER, BRAWLER],
        geoffrey_position(),
    );
    let mut now = 1_000;
    accept_quest(&mut game, &at_geoffrey, now);
    now += 100;
    send(&mut game, &at_geoffrey, &select_job(BRAWLER), now);
    game.on_location(&character(CHARACTER, WOLF_CENTRE), now + 50)
        .expect("movement is accepted");
    let wolves = starting_wolves();
    let known: BTreeSet<u64> = wolves.iter().map(|(guid, _)| *guid).collect();
    let mut field = game.take_shared_fields().expect("the shared field exists");
    now += 100;
    let (first, position) = wolves[0];
    let (_, next) = kill_shared_target(
        &mut game,
        &mut field,
        &known,
        &character(CHARACTER, [position[0] + 1.0, position[1], position[2]]),
        first,
        now,
    );
    now = next;
    assert_eq!(counts(&game, QUEST), [1, 1, 0]);

    // Production save envelope, then the same bind a reconnecting session runs.
    let path = fixture.path().join("character.json");
    let saved_character = character(CHARACTER, WOLF_CENTRE);
    let snapshot = game_store::Snapshot {
        character: saved_character.clone(),
        gameplay: Some(game.state()),
    };
    game_store::save(&path, &snapshot).expect("the save envelope writes");
    let loaded = game_store::load_or_create(&path, saved_character.clone())
        .expect("the save envelope loads");
    let mut game = fixture.game(loaded.gameplay);
    game.initialize_world(&saved_character, now + 100)
        .expect("the reconnect initializes");
    assert_eq!(counts(&game, QUEST), [1, 1, 0]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    // The shared field is server-owned; the reconnect sees a fresh population and
    // finishes the two remaining starter wolves.
    let mut field = game.take_shared_fields().expect("the shared field exists");
    for (guid, position) in wolves.iter().skip(1) {
        now += 100;
        let (_, next) = kill_shared_target(
            &mut game,
            &mut field,
            &known,
            &character(CHARACTER, [position[0] + 1.0, position[1], position[2]]),
            *guid,
            now,
        );
        now = next;
    }
    assert_eq!(counts(&game, QUEST), [1, 3, 0]);

    // Turn in once and reload again: the completion and payout persist.
    now += 100;
    let end = open_end_screen(&mut game, &at_geoffrey, now);
    assert_end_reward(&end, QUEST_COINS, QUEST_XP);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    finish_quest(
        &mut game,
        &at_geoffrey,
        QUEST,
        ADVENTURER,
        QUEST_COINS,
        QUEST_XP,
        now + 100,
    );
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + QUEST_XP);
    assert_eq!(counts(&game, QUEST), [1, 3, 1]);

    let snapshot = game_store::Snapshot {
        character: saved_character.clone(),
        gameplay: Some(game.state()),
    };
    game_store::save(&path, &snapshot).expect("the completed save writes");
    let loaded = game_store::load_or_create(&path, saved_character.clone())
        .expect("the completed save loads");
    let mut reloaded = fixture.game(loaded.gameplay);
    assert_eq!(quest_status(&reloaded, QUEST), QuestStatus::Completed);
    assert_eq!(counts(&reloaded, QUEST), [1, 3, 1]);
    let before = capture(&reloaded);
    assert_packet_changes_nothing(
        &mut reloaded,
        &at_geoffrey,
        &quest_end_reply(QUEST),
        now + 200,
        "a replayed 49/14 after the second reconnect",
    );
    assert_unchanged(
        &reloaded,
        &before,
        "a replayed 49/14 after the second reconnect",
    );
    assert_eq!(pending_turnin(&reloaded), None);
}

#[test]
fn a_failing_quest_event_for_one_player_does_not_stop_the_other() {
    // A's saved quest state belongs to another character, so A's catalogue bind is
    // refused and A's kills cannot credit it. A's wallet is also at the wire cap:
    // the T240 kill reward is refused for A alone, never a panic.
    let fixture = Fixture::for_character(CHARACTER);
    // The opening reference has no field-kill reward table; add the shipped one so
    // A's over-cap reward is actually generated and refused.
    fs::copy(
        content_dir().join("field_kill_rewards.json"),
        fixture.path().join("field_kill_rewards.json"),
    )
    .expect("the field kill reward table copies");
    let mut stale = active_quests(&content_dir(), CHARACTER);
    stale.character_id = OTHER;
    let base = Gameplay::load(fixture.path(), None).expect("the opening fixture loads");
    let mut state = base.state();
    state.npc_quests = Some(stale);
    state.inventory.coins = i32::MAX as u32;
    {
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = BRAWLER;
        profiles.unlocked.insert(BRAWLER);
        profiles.experience.insert(BRAWLER, u32::MAX);
    }
    let mut a = Gameplay::load(fixture.path(), Some(state)).expect("A's gameplay loads");
    let error = a
        .bind_npc_quest_catalogue(&content_dir(), CHARACTER)
        .expect_err("A's stale quest save is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    let kept = a
        .state()
        .npc_quests
        .expect("the refused bind keeps the saved state");
    assert_eq!(kept.character_id, OTHER);

    let at_geoffrey = character(OTHER, geoffrey_position());
    let mut b = player(
        &fixture,
        OTHER,
        ADVENTURER,
        &[ADVENTURER, BRAWLER],
        geoffrey_position(),
    );
    let mut now = 1_000;
    accept_quest(&mut b, &at_geoffrey, now);
    now += 100;
    send(&mut b, &at_geoffrey, &select_job(BRAWLER), now);
    b.on_location(&character(OTHER, WOLF_CENTRE), now + 50)
        .expect("B's movement is accepted");

    let wolves = starting_wolves();
    let known: BTreeSet<u64> = wolves.iter().map(|(guid, _)| *guid).collect();
    let mut field = a.take_shared_fields().expect("A owns the shared field");
    b.take_shared_fields();

    // A's kill is accepted; the quest state is not credited and the capped reward
    // is refused for A without stopping the request.
    now += 100;
    let (a_target, position) = wolves[0];
    let a_at = character(CHARACTER, [position[0] + 1.0, position[1], position[2]]);
    let (out, next) = kill_shared_target(&mut a, &mut field, &known, &a_at, a_target, now);
    now = next;
    assert_eq!(field.health(a_target), Some(0));
    assert!(
        !out.replies
            .iter()
            .any(|packet| has_family(packet, 49, 9, None)),
        "A's stale quest is never credited"
    );
    assert_eq!(counts(&a, QUEST), [0, 0, 0]);
    assert_eq!(a.state().inventory.coins, i32::MAX as u32);
    assert!(
        !out.replies
            .iter()
            .any(|packet| has_family(packet, 38, 19, None) || has_family(packet, 38, 14, None)),
        "A's over-cap coin/XP pair is refused for A alone"
    );
    assert_eq!(
        a.state().profiles.as_ref().unwrap().experience[&BRAWLER],
        u32::MAX
    );

    // B's kill in the same field is credited and paid normally.
    now += 100;
    let (b_target, position) = wolves[1];
    let b_at = character(OTHER, [position[0] + 1.0, position[1], position[2]]);
    let (out, _) = kill_shared_target(&mut b, &mut field, &known, &b_at, b_target, now);
    assert_eq!(field.health(b_target), Some(0));
    assert_eq!(counts(&b, QUEST), [1, 1, 0]);
    assert!(
        objective_update(&out, QUEST, 2, 1),
        "B's 49/9 still reports B's kill"
    );
    assert!(
        out.replies
            .iter()
            .any(|packet| has_family(packet, 38, 19, None)),
        "B's field kill still pays coins"
    );
    assert!(
        out.replies
            .iter()
            .any(|packet| has_family(packet, 38, 14, None)),
        "B's field kill still pays active-job XP"
    );
    assert!(coins(&b) > 0, "the level-1 field reward pays B");
}
