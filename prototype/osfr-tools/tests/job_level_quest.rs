//! T361: the `job_level` objective and the Warrior "Achieve Level 5" gate (3150005).
//!
//! Nothing new is sent on the wire. The objective uses the existing 49/8
//! `ObjectiveActivated` reader `0x00c7aeb0`, 49/9 `ObjectiveUpdate` reader
//! `0x00c7af60` and 49/10 `ObjectiveComplete` reader `0x00c7b070`; the turn-in uses
//! the existing 49/13 `QuestEndPacket::ReadFields` reader `0x00c7b990` with the
//! reward bundle `0x008e7930` and 49/14 reply constructor `0x00a864c0`. The NPC
//! path is the existing 26/8 ctor `0x008b6bc0`, 26/10 ctor `0x00a36f80`, 26/6 ctor
//! `0x009f5890`, 49/1 reader `0x00c7bb60` and 49/2 ctor `0x00a86400`.
//!
//! Client text ids: title 387433, intro 387432, description 387434, end text
//! 387435; the W5 line is 248116931 ("Reach level 5 as a warrior, then return to
//! Steele."), the wait line 1524930698 and the journal 2755651467.
//!
//! Every test drives the production in-world router (`Gameplay::handle_in_world` ->
//! `gameplay/npc.rs` / `gameplay/npc_quests.rs` -> `QuestEngine`) or the production
//! loader. The field test uses the production shared-field kill reward path; the
//! level-change tests use the developer `realm level` command and the load-error test
//! drives `QuestEngine::from_split_content`. (R) The level target itself is
//! reconstructed server policy: the client text names level 5, but the client does not
//! send a rank objective.

#[path = "common/field_reward_fixture.rs"]
mod field_reward_fixture;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use field_reward_fixture::kill_one;
use npc_quest_chain::*;
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, GameplayState},
    profiles::ProfileState,
    quest_engine::{
        CharacterEligibility, QuestEngine, QuestEvent, QuestProgress, QuestState, QuestStatus,
    },
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const STEELE: u64 = 9_000_000_208;
const MORNINGLORY: u64 = 9_000_000_155;
const WARRIOR: i32 = 32;
const BRAWLER: i32 = 43;
const W4: u32 = 3_150_004;
const W5: u32 = 3_150_005;
const W5_END: u32 = 387_435;
const W5_OBJECTIVE: u32 = 1;
const W5_REWARD_COINS: u32 = 15;
const RANK_FIVE_XP: u32 = 400;
const OPEN_WORLD: u32 = 0;
const SECOND_CHARACTER: u64 = 194_019;
const FIELD_GUIDS: [u64; 5] = [
    131_000_361_001,
    131_000_361_002,
    131_000_361_003,
    131_000_361_004,
    131_000_361_005,
];
const SYNTHETIC_QUEST: u32 = 3_150_905;
const SYNTHETIC_GIVER: u64 = 9_000_100_005;
/// A shipped-catalogue copy plus one Warrior quest that pays 100 job XP on turn-in.
const TURNIN_QUEST: u32 = 3_150_906;
const W1: u32 = 3_150_001;

static NEXT_CONTENT: AtomicU64 = AtomicU64::new(0);

fn avatar_at(npc_id: u64) -> CharacterState {
    character_at(OPEN_WORLD, content_position("snowhill.json", npc_id))
}

fn avatar_for(guid: u64, npc_id: u64) -> CharacterState {
    character_at_for(guid, OPEN_WORLD, content_position("snowhill.json", npc_id))
}

fn said(out: &osfr_tools::gameplay::Outcome, needle: &str) -> bool {
    out.replies.iter().any(|packet| {
        packet.starts_with(&[15, 0, 3, 0])
            && packet.len() >= 9
            && String::from_utf8_lossy(&packet[8..packet.len() - 1]).contains(needle)
    })
}

/// 45/7 `ClientComplete`: a client claim can only name a client-reported goal.
fn client_claim(objective: u32) -> Vec<u8> {
    let mut packet = vec![45, 0, 7];
    packet.extend((objective as i32).to_le_bytes());
    packet
}

/// A Warrior save with W4 completed (W5's prerequisite) and no W5 progress.
fn warrior_state(fixture: &Fixture, character: u64, warrior_xp: u32) -> GameplayState {
    let mut state = Gameplay::load(fixture.path(), None)
        .expect("opening fixture loads")
        .state();
    state.profiles = Some(ProfileState {
        version: 1,
        active: WARRIOR,
        unlocked: BTreeSet::from([1, WARRIOR, BRAWLER]),
        experience: BTreeMap::from([(WARRIOR, warrior_xp)]),
    });
    state.npc_quests = Some(QuestState {
        version: 1,
        character_id: character,
        quests: BTreeMap::from([(
            W4,
            QuestProgress {
                status: QuestStatus::Completed,
                counts: vec![1],
            },
        )]),
        pending_offer: None,
        pending_turnin: None,
        pending_turnin_actor: None,
        selected_quest_id: None,
        auto_switch_selected: false,
    });
    state
}

fn warrior_game(fixture: &Fixture, warrior_xp: u32) -> Gameplay {
    fixture.game(Some(warrior_state(fixture, CHARACTER, warrior_xp)))
}

fn accept_w5(
    game: &mut Gameplay,
    character: &CharacterState,
    now: &mut u64,
) -> osfr_tools::gameplay::Outcome {
    *now += 10;
    offer(game, character, STEELE, W5, *now);
    *now += 10;
    accept(game, character, W5, *now)
}

fn complete_with_realm_level(game: &mut Gameplay, character: &CharacterState, now: u64) {
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    let level = send(game, character, &chat::local_request("realm level 5"), now);
    assert!(
        level
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[15, 0, 3, 0])),
        "the developer level command answers in chat"
    );
}

/// The captured initial state equips the Brawler but not Warrior (the opening
/// package predates Warrior). Add the shipped canonical Warrior starter row (the
/// same W113 row `job_basic_attacks` uses, class 17) to the fixture inventory and
/// grant/equip it through the production `realm give` and 42/3 paths.
fn add_warrior_starter_weapon(fixture: &Fixture) {
    let path = fixture.path().join("inventory.json");
    let mut inventory: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let definitions = inventory["definitions"].as_array_mut().unwrap();
    if !definitions.iter().any(|row| row["id"] == 29963) {
        definitions.push(json!({
            "id": 29963,
            "slot": 7,
            "class_id": 17,
            "default_tint": 0,
            "max_stack": -1,
            "model": "club_ar_ag_weapon_cudgel.adr",
            "texture": "",
            "tint_alias": "",
            "effect": 0,
            "wield_type": 12,
            "serialized_hex": "0b75000000000000000000000000000000000000000000000000000000000000110000000000000007000000000000000000000000000000",
            "sha256": "dd8b3607d16fd953e370c2af82240ee38bb5a5f4f2aef8d42b1ddc0a51799e96"
        }));
    }
    fs::write(&path, serde_json::to_vec(&inventory).unwrap()).unwrap();
}

fn equip_warrior_weapon(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    *now += 10;
    let given = send(
        game,
        character,
        &chat::local_request("realm give 29963"),
        *now,
    );
    assert!(given.changed, "the Warrior starter weapon is granted");
    let item = game
        .state()
        .inventory
        .items
        .iter()
        .find(|item| item.definition_id == 29_963 && item.count > 0)
        .expect("the granted Warrior weapon is owned")
        .id;
    let mut equip = vec![42, 0, 3, 0];
    for value in [item, WARRIOR, 7] {
        equip.extend(value.to_le_bytes());
    }
    *now += 10;
    let equipped = send(game, character, &equip, *now);
    assert!(equipped.changed, "the Warrior weapon equips");
    assert!(
        game.state().inventory.equipment.iter().any(|entry| {
            entry.profile_id == WARRIOR && entry.slot == 7 && entry.item_id == item
        })
    );
}

fn field_actor(guid: u64) -> Value {
    json!({
        "guid": guid,
        "definition_id": 3156,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic level-gate field actor",
        "texture_alias": "",
        "position": [1.0, 0.0, 0.0],
        "max_health": 1,
        "damage": 1,
        "boss": false,
        "level": 1,
        "respawn_seconds": 60,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode":"stationary","reach":2.5,"attack_ms":1000},
        "quest_ids": []
    })
}

/// One production field kill pays 100 active-job XP, so four kills reach rank 5.
fn field_fixture() -> Fixture {
    let fixture = Fixture::new().with_field_enemies(&FIELD_GUIDS.map(field_actor));
    // The reference opening fixture predates the canonical six-job basic roster;
    // the shipped one declares Warrior (profile 32) so the production basic attack
    // can be used without inventing combat content in the test.
    fs::copy(
        content_dir().join("opening-abilities.json"),
        fixture.path().join("opening-abilities.json"),
    )
    .unwrap();
    add_warrior_starter_weapon(&fixture);
    fs::write(
        fixture.path().join("field_kill_rewards.json"),
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "level_rewards": [{
                "min_level": 1,
                "max_level": 20,
                "coins": 0,
                "job_xp": 100,
                "source": [{
                    "kind": "reconstructed",
                    "evidence": "(R) T361: test-only 100 XP per field kill rate"
                }]
            }],
            "definition_overrides": []
        }))
        .unwrap(),
    )
    .unwrap();
    fixture
}

fn kill_field(
    game: &mut Gameplay,
    field: &mut osfr_tools::encounter::SharedFieldState,
    target: u64,
    now: u64,
) -> (osfr_tools::gameplay::Outcome, u64) {
    let character = character_at_for(CHARACTER, OPEN_WORLD, [0.0, 0.0, 0.0]);
    kill_one(game, field, &character, target, now)
}

#[test]
fn the_record_loads_and_offers_to_a_warrior() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, 0);

    let offered = accept_w5(&mut game, &steele, &mut now);
    assert_eq!(quest_status(&game, W5), QuestStatus::Active);
    assert_eq!(counts(&game, W5), vec![0]);
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 10, 0, 0, 0])),
        "accepting at rank 1 must not complete the level gate"
    );
}

#[test]
fn real_xp_to_level_five_completes_it_once() {
    let fixture = field_fixture();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, 0);
    accept_w5(&mut game, &steele, &mut now);
    equip_warrior_weapon(&mut game, &steele, &mut now);

    let field_character = character_at_for(CHARACTER, OPEN_WORLD, [0.0, 0.0, 0.0]);
    game.initialize_world(&field_character, 0).unwrap();
    let mut field = game.take_shared_fields().expect("the field is published");

    for guid in FIELD_GUIDS.iter().take(3) {
        let (_, next) = kill_field(&mut game, &mut field, *guid, now);
        now = next;
    }
    assert_eq!(profile_xp(&game, WARRIOR), 300);
    assert_eq!(counts(&game, W5), vec![0], "rank 4 is still short of 5");

    let (fourth, next) = kill_field(&mut game, &mut field, FIELD_GUIDS[3], now);
    now = next;
    assert_eq!(profile_xp(&game, WARRIOR), RANK_FIVE_XP);
    assert_eq!(counts(&game, W5), vec![1], "rank 5 completes the goal once");
    assert!(
        objective_update(&fourth, W5, W5_OBJECTIVE, 1),
        "the completion emits the existing 49/9 update"
    );
    assert!(
        objective_complete(&fourth, W5, W5_OBJECTIVE),
        "the completion emits the existing 49/10 frame"
    );
    let complete = fourth
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[49, 0, 10, 0, 0, 0]))
        .expect("49/10 completion frame");
    assert_eq!(
        complete,
        &objective_complete_packet(W5, W5_OBJECTIVE, 1.0),
        "49/10 must match reader 0x00c7b070: quest, objective, progress, dword, bool"
    );

    let quests_before = game.state().npc_quests.clone();
    let (fifth, _) = kill_field(&mut game, &mut field, FIELD_GUIDS[4], now);
    assert_eq!(
        profile_xp(&game, WARRIOR),
        500,
        "rank 6 paid for the fifth kill"
    );
    assert_eq!(
        counts(&game, W5),
        vec![1],
        "the goal never regresses or re-credits"
    );
    assert_eq!(game.state().npc_quests, quests_before);
    assert!(
        !fifth
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 10, 0, 0, 0])),
        "a later level change must not emit a second completion"
    );
}

#[test]
fn the_developer_level_command_completes_it_too() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, 0);
    accept_w5(&mut game, &steele, &mut now);

    complete_with_realm_level(&mut game, &steele, now + 10);
    assert_eq!(profile_xp(&game, WARRIOR), RANK_FIVE_XP);
    assert_eq!(counts(&game, W5), vec![1]);
    let once = saved(&game);

    let repeated = send(
        &mut game,
        &steele,
        &chat::local_request("realm level 5"),
        now + 20,
    );
    assert_eq!(profile_xp(&game, WARRIOR), RANK_FIVE_XP);
    assert_eq!(counts(&game, W5), vec![1]);
    assert!(
        !repeated
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 10, 0, 0, 0])),
        "a repeated realm level must not emit 49/10 again"
    );
    assert_eq!(
        saved(&game),
        once,
        "a repeated realm level changed the save"
    );
}

#[test]
fn accepting_at_level_five_completes_immediately() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, RANK_FIVE_XP);

    let accepted = accept_w5(&mut game, &steele, &mut now);
    assert_eq!(counts(&game, W5), vec![1]);
    assert_eq!(
        profile_xp(&game, WARRIOR),
        RANK_FIVE_XP,
        "no XP event was needed"
    );
    assert!(objective_complete(&accepted, W5, W5_OBJECTIVE));
}

#[test]
fn a_higher_rank_still_completes_it() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, RANK_FIVE_XP + 100);

    let accepted = accept_w5(&mut game, &steele, &mut now);
    assert_eq!(
        profile_xp(&game, WARRIOR),
        RANK_FIVE_XP + 100,
        "the character is rank 6, not rank 5"
    );
    assert_eq!(counts(&game, W5), vec![1]);
    assert!(
        objective_complete(&accepted, W5, W5_OBJECTIVE),
        "the match is event level >= objective level, not equality"
    );
}

#[test]
fn a_job_level_event_is_profile_exact_and_uses_at_least() {
    let state = QuestState {
        version: 1,
        character_id: CHARACTER,
        quests: BTreeMap::from([
            (
                W4,
                QuestProgress {
                    status: QuestStatus::Completed,
                    counts: vec![1],
                },
            ),
            (
                W5,
                QuestProgress {
                    status: QuestStatus::Active,
                    counts: vec![0],
                },
            ),
        ]),
        pending_offer: None,
        pending_turnin: None,
        pending_turnin_actor: None,
        selected_quest_id: None,
        auto_switch_selected: false,
    };
    let mut engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility::default(),
        Some(state),
    )
    .expect("the shipped catalogue and the active W5 state load");

    let wrong_profile = engine
        .on_event(QuestEvent::JobLevel {
            profile_id: BRAWLER,
            level: 20,
        })
        .unwrap();
    assert!(!wrong_profile.changed, "another profile must not credit W5");
    assert_eq!(engine.progress(W5).unwrap().counts, vec![0]);

    let short = engine
        .on_event(QuestEvent::JobLevel {
            profile_id: WARRIOR,
            level: 4,
        })
        .unwrap();
    assert!(!short.changed, "rank 4 is short of level 5");
    assert_eq!(engine.progress(W5).unwrap().counts, vec![0]);

    let completed = engine
        .on_event(QuestEvent::JobLevel {
            profile_id: WARRIOR,
            level: 5,
        })
        .unwrap();
    assert!(completed.changed);
    assert_eq!(engine.progress(W5).unwrap().counts, vec![1]);
}

#[test]
fn a_reconnect_at_a_late_level_completes_it_on_load() {
    let fixture = Fixture::new();
    // A save written before the level notification existed: active W5, count 0,
    // but the Warrior is already rank 5.
    let mut state = warrior_state(&fixture, CHARACTER, RANK_FIVE_XP);
    state.npc_quests.as_mut().unwrap().quests.insert(
        W5,
        QuestProgress {
            status: QuestStatus::Active,
            counts: vec![0],
        },
    );
    let mut game = fixture.game(Some(state));
    assert_eq!(
        counts(&game, W5),
        vec![1],
        "the catalogue bind completes a satisfied active job-level goal"
    );
    assert_eq!(quest_status(&game, W5), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), None);

    let character = avatar_at(STEELE);
    let startup = game.initialize_world(&character, 0).unwrap();
    let expected = objective_complete_packet(W5, W5_OBJECTIVE, 1.0);
    assert!(
        startup.replies.contains(&expected),
        "the login sync re-sends the 49/10 frame for the load-time completion"
    );
}

#[test]
fn another_profile_and_another_player_are_not_credited() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, 0);
    accept_w5(&mut game, &steele, &mut now);

    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    now += 10;
    let switched = send(&mut game, &steele, &select_job(BRAWLER), now);
    assert!(switched.changed, "the fixture unlocks Brawler");
    now += 10;
    let leveled = send(
        &mut game,
        &steele,
        &chat::local_request("realm level 5"),
        now,
    );
    assert_eq!(profile_xp(&game, BRAWLER), RANK_FIVE_XP);
    assert_eq!(
        counts(&game, W5),
        vec![0],
        "another job's rank is not Warrior"
    );
    assert!(
        !leveled
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 10, 0, 0, 0])),
        "a Brawler level change must not complete a Warrior objective"
    );

    let mut second = fixture.game_as(
        SECOND_CHARACTER,
        Some(warrior_state(&fixture, SECOND_CHARACTER, 0)),
    );
    let second_steele = avatar_for(SECOND_CHARACTER, STEELE);
    let mut second_now = 2_000;
    accept_w5(&mut second, &second_steele, &mut second_now);
    complete_with_realm_level(&mut second, &second_steele, second_now + 10);
    assert_eq!(
        counts(&second, W5),
        vec![1],
        "the second character completes its own copy"
    );
    assert_eq!(
        counts(&game, W5),
        vec![0],
        "the first character is not credited"
    );
}

#[test]
fn a_live_change_before_the_turn_in_survives() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, 0);
    accept_w5(&mut game, &steele, &mut now);

    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    let before = coins(&game);
    now += 10;
    let granted = send(
        &mut game,
        &steele,
        &chat::local_request("realm coins 7"),
        now,
    );
    assert!(granted.changed);
    assert_eq!(coins(&game), before + 7);

    complete_with_realm_level(&mut game, &steele, now + 10);
    now += 20;
    talk(&mut game, &steele, STEELE, now);
    release_end_screen(&mut game, &steele, W5, STEELE, W5_END, now + 10);
    finish_quest(
        &mut game,
        &steele,
        W5,
        WARRIOR,
        W5_REWARD_COINS,
        0,
        now + 20,
    );
    assert_eq!(
        coins(&game),
        before + 7 + W5_REWARD_COINS,
        "the live coin grant survives the turn-in payout"
    );
    assert_eq!(quest_status(&game, W5), QuestStatus::Completed);
}

#[test]
fn a_near_cap_wallet_refuses_the_turn_in_atomically() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut state = warrior_state(&fixture, CHARACTER, 0);
    state.inventory.coins = i32::MAX as u32 - 10;
    let mut game = fixture.game(Some(state));
    accept_w5(&mut game, &steele, &mut now);
    complete_with_realm_level(&mut game, &steele, now + 10);
    now += 20;
    talk(&mut game, &steele, STEELE, now);
    release_end_screen(&mut game, &steele, W5, STEELE, W5_END, now + 10);

    let before = saved(&game);
    let wallet = coins(&game);
    let refused = game.handle_in_world(&quest_end_reply(W5), &steele, now + 20);
    assert!(
        refused.is_err(),
        "a 15-coin payout over the wire coin cap is refused"
    );
    assert_eq!(saved(&game), before, "the refused payout changed the save");
    assert_eq!(coins(&game), wallet);
    assert_eq!(quest_status(&game, W5), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), Some(W5));
}

#[test]
fn the_progress_and_reward_survive_a_reload() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, 0);
    accept_w5(&mut game, &steele, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, W5), QuestStatus::Active);
    assert_eq!(
        counts(&game, W5),
        vec![0],
        "mid-quest progress survives a reload"
    );
    assert_eq!(pending_turnin(&game), None);

    complete_with_realm_level(&mut game, &steele, now + 10);
    assert_eq!(counts(&game, W5), vec![1]);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(
        counts(&game, W5),
        vec![1],
        "the completion survives a reload"
    );
    assert_eq!(quest_status(&game, W5), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), None);

    now += 20;
    let opened = talk(&mut game, &steele, STEELE, now);
    assert_eq!(
        pending_turnin(&game),
        Some(W5),
        "the completed W5 turn-in opens at Steele"
    );
    assert!(
        !opened
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );

    now += 10;
    let end = release_end_screen(&mut game, &steele, W5, STEELE, W5_END, now);
    assert_eq!(end_screen_reward(&end), (W5_REWARD_COINS, 0));
    now += 10;
    let coins_before = coins(&game);
    finish_quest(&mut game, &steele, W5, WARRIOR, W5_REWARD_COINS, 0, now);
    let paid_coins = coins(&game);
    assert_eq!(paid_coins, coins_before + W5_REWARD_COINS);

    let mut paid = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&paid, W5), QuestStatus::Completed);
    assert_eq!(counts(&paid, W5), vec![1]);
    assert_eq!(coins(&paid), paid_coins, "the 15 coins survive a reload");

    now += 10;
    assert_packet_changes_nothing(
        &mut paid,
        &steele,
        &quest_end_reply(W5),
        now,
        "a replayed 49/14 after payment",
    );
    assert_eq!(coins(&paid), paid_coins, "a replayed 49/14 pays nothing");
}

#[test]
fn refusals_change_nothing() {
    let fixture = Fixture::new();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;
    let mut game = warrior_game(&fixture, 0);
    accept_w5(&mut game, &steele, &mut now);

    let before = saved(&game);
    let wallet = coins(&game);
    assert_packet_changes_nothing(
        &mut game,
        &steele,
        &quest_end_reply(W5),
        now + 10,
        "49/14 before completion",
    );
    assert_eq!(saved(&game), before);
    assert_eq!(coins(&game), wallet);
    assert_eq!(counts(&game, W5), vec![0]);

    assert_packet_changes_nothing(
        &mut game,
        &steele,
        &quest_reply(9_999_999, true),
        now + 20,
        "a forged quest id",
    );
    assert_eq!(saved(&game), before);
    assert_eq!(coins(&game), wallet);

    assert_packet_changes_nothing(
        &mut game,
        &steele,
        &client_claim(W5_OBJECTIVE),
        now + 25,
        "a forged 45/7 client claim",
    );
    assert_eq!(saved(&game), before);
    assert_eq!(coins(&game), wallet);
    assert_eq!(counts(&game, W5), vec![0]);

    let morninglory = avatar_at(MORNINGLORY);
    assert_no_offer(
        &mut game,
        &morninglory,
        MORNINGLORY,
        &mut now,
        "a wrong-NPC Talk for W5",
    );
    assert_eq!(coins(&game), wallet);
    assert_eq!(counts(&game, W5), vec![0]);

    // A Talk out of range is refused before the production NPC service opens a
    // menu; the gateway's staged clone is dropped on the same error.
    let mut far = steele.clone();
    far.position = [
        steele.position[0] + 1_000.0,
        steele.position[1],
        steele.position[2],
    ];
    let range_before = saved(&game);
    assert!(
        game.handle_in_world(&npc_interact(STEELE), &far, now + 30)
            .is_err(),
        "an out-of-range 26/8 must be refused"
    );
    assert_eq!(saved(&game), range_before);
    assert_eq!(coins(&game), wallet);
    assert_eq!(counts(&game, W5), vec![0]);

    // A real encounter occupation closes the NPC menu gate before any offer or
    // turn-in can be consumed; the same request must change nothing.
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    let started = send(
        &mut game,
        &steele,
        &chat::local_request("realm battle 411"),
        now + 40,
    );
    assert!(
        game.state()
            .encounter
            .as_ref()
            .and_then(|encounter| encounter.active.as_ref())
            .is_some(),
        "the developer battle starts the encounter occupation: {:?}",
        started.replies.len()
    );
    let busy_before = saved(&game);
    let busy_coins = coins(&game);
    let busy_quests = game.state().npc_quests.clone();
    let busy = send(&mut game, &steele, &npc_interact(STEELE), now + 50);
    assert!(
        said(&busy, "Finish the active activity before interacting."),
        "the busy NPC interaction is refused"
    );
    assert_eq!(saved(&game), busy_before, "a busy 26/8 changed the save");
    assert_eq!(coins(&game), busy_coins);
    assert_eq!(game.state().npc_quests, busy_quests);
    assert_eq!(counts(&game, W5), vec![0]);
    assert_packet_changes_nothing(
        &mut game,
        &steele,
        &quest_reply(W5, true),
        now + 60,
        "a busy accept",
    );
    assert_eq!(coins(&game), busy_coins);
    assert_eq!(counts(&game, W5), vec![0]);
}

#[test]
fn a_quest_turn_in_job_xp_completes_it_too() {
    let fixture = Fixture::new();
    // The shipped records cannot raise a Warrior's level on their own turn-in
    // (only W1 and W4 pay 25 XP each), so the fourth level-change path needs a
    // payer. It is one extra warrior.json-equivalent record in a private copy.
    let content = turnin_xp_content();
    let steele = avatar_at(STEELE);
    let mut now = 1_000;

    let mut state = warrior_state(&fixture, CHARACTER, 300);
    let quests = &mut state.npc_quests.as_mut().unwrap().quests;
    quests.insert(
        W1,
        QuestProgress {
            status: QuestStatus::Completed,
            counts: vec![1],
        },
    );
    quests.insert(
        W5,
        QuestProgress {
            status: QuestStatus::Active,
            counts: vec![0],
        },
    );
    let mut game = Gameplay::load(fixture.path(), Some(state)).expect("opening fixture loads");
    game.bind_npc_quest_catalogue(content.0.as_path(), CHARACTER)
        .expect("the shipped catalogue plus the turn-in payer loads");

    assert_eq!(
        profile_xp(&game, WARRIOR),
        300,
        "the Warrior binds at rank 4"
    );
    assert_eq!(
        counts(&game, W5),
        vec![0],
        "rank 4 is short of the level target"
    );

    offer(&mut game, &steele, STEELE, TURNIN_QUEST, now);
    now += 10;
    accept(&mut game, &steele, TURNIN_QUEST, now);
    now += 10;
    talk_step(&mut game, &steele, STEELE, now, "the turn-in payer's Talk");
    now += 10;
    let end = release_end_screen(&mut game, &steele, TURNIN_QUEST, STEELE, 0, now);
    assert_eq!(end_screen_reward(&end), (0, 100));
    assert_eq!(
        counts(&game, W5),
        vec![0],
        "the 49/13 display grants nothing"
    );

    now += 10;
    let turned = send(&mut game, &steele, &quest_end_reply(TURNIN_QUEST), now);
    assert_eq!(profile_xp(&game, WARRIOR), RANK_FIVE_XP);
    assert_eq!(quest_status(&game, TURNIN_QUEST), QuestStatus::Completed);
    assert_eq!(
        counts(&game, W5),
        vec![1],
        "the turn-in's own job XP completes the level goal in the same outcome"
    );
    assert!(
        objective_complete(&turned, W5, W5_OBJECTIVE),
        "the turn-in path emits the existing 49/10 completion frame"
    );
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn synthetic_content(objectives: Value) -> Scratch {
    let root = std::env::temp_dir().join(format!(
        "osfr-t361-job-level-{}-{}",
        std::process::id(),
        NEXT_CONTENT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("quests")).unwrap();
    fs::create_dir_all(root.join("npcs")).unwrap();
    let source = json!([{
        "kind": "reconstructed",
        "evidence": "(R) T361 synthetic job_level fixture"
    }]);
    let quests = json!({
        "schema_version": 1,
        "records": [{
            "id": SYNTHETIC_QUEST,
            "title_string_id": 1,
            "giver_npc_id": SYNTHETIC_GIVER,
            "prerequisite_quest_ids": [],
            "objectives": objectives,
            "rewards": [{"coins": 1, "items": [], "job_xp": {}, "source": source}],
            "source": source
        }]
    });
    fs::write(
        root.join("quests/synthetic.json"),
        serde_json::to_vec(&quests).unwrap(),
    )
    .unwrap();
    let jobs = json!({
        "schema_version": 1,
        "records": [{
            "id": "warrior-fixture",
            "name": "Warrior fixture",
            "profile_id": WARRIOR,
            "category": "combat",
            "max_level": 20,
            "members_only": false,
            "trainer_npc_ids": [],
            "quest_ids": [SYNTHETIC_QUEST],
            "abilities": [],
            "source": source
        }]
    });
    fs::write(root.join("jobs.json"), serde_json::to_vec(&jobs).unwrap()).unwrap();
    let npcs = json!({
        "schema_version": 1,
        "records": [{
            "npc_id": SYNTHETIC_GIVER,
            "quest_ids_given": [SYNTHETIC_QUEST],
            "quests_given": [],
            "quests_accepted": []
        }]
    });
    fs::write(
        root.join("npcs/synthetic.json"),
        serde_json::to_vec(&npcs).unwrap(),
    )
    .unwrap();
    Scratch(root)
}

/// A private copy of the shipped split content with one extra Warrior-given quest
/// whose 100 job XP is paid on 49/14. It exercises the turn-in level path without
/// touching the shipped records or the pinned `quest_job_xp` table.
fn turnin_xp_content() -> Scratch {
    let root = std::env::temp_dir().join(format!(
        "osfr-t361-job-level-turnin-{}-{}",
        std::process::id(),
        NEXT_CONTENT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    copy_tree(&content_dir(), &root);
    let jobs_path = root.join("jobs.json");
    let mut jobs: Value = serde_json::from_slice(&fs::read(&jobs_path).unwrap()).unwrap();
    let warrior = jobs["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["profile_id"].as_i64() == Some(i64::from(WARRIOR)))
        .expect("jobs.json declares the Warrior profile");
    // The split loader requires every quest file to sit in one job's union.
    warrior["quest_ids"]
        .as_array_mut()
        .unwrap()
        .push(json!(TURNIN_QUEST));
    fs::write(&jobs_path, serde_json::to_vec(&jobs).unwrap()).unwrap();
    let source = json!([{
        "kind": "reconstructed",
        "evidence": "(R) T361 synthetic turn-in job-XP payer for the level notification"
    }]);
    let quest = json!({
        "schema_version": 1,
        "records": [{
            "id": TURNIN_QUEST,
            "title_string_id": 1,
            "giver_npc_id": STEELE,
            "prerequisite_quest_ids": [],
            "objectives": [{
                "kind": "talk_to_npc",
                "id": 1,
                "npc_id": STEELE,
                "source": source
            }],
            "rewards": [{
                "coins": 0,
                "items": [],
                "job_xp": {"32": 100},
                "source": source
            }],
            "source": source
        }]
    });
    fs::write(
        root.join("quests/t361_turnin.json"),
        serde_json::to_vec(&quest).unwrap(),
    )
    .unwrap();
    Scratch(root)
}

fn job_level_goal(profile_id: i32, level: u8) -> Value {
    json!([{
        "kind": "job_level",
        "id": 1,
        "profile_id": profile_id,
        "level": level,
        "source": [{
            "kind": "reconstructed",
            "evidence": "(R) T361 synthetic job_level goal"
        }]
    }])
}

fn load(goal: Value) -> std::io::Result<QuestEngine> {
    let scratch = synthetic_content(goal);
    QuestEngine::from_split_content(
        scratch.0.as_path(),
        1,
        CharacterEligibility::default(),
        None,
    )
}

fn talk_goal(id: u32) -> Value {
    json!({
        "kind": "talk_to_npc",
        "id": id,
        "npc_id": SYNTHETIC_GIVER,
        "source": [{
            "kind": "reconstructed",
            "evidence": "(R) F576 synthetic ordered step"
        }]
    })
}

/// A malformed record must refuse the whole load, not quietly disable the quest.
#[test]
fn a_malformed_job_level_goal_is_a_load_error() {
    let clean = load(job_level_goal(WARRIOR, 5)).expect("the clean goal loads");
    assert!(
        clean.is_enabled(SYNTHETIC_QUEST),
        "the clean goal is enabled"
    );
    // F576: a job-level goal may open an ordered quest, so the refusal below is about
    // its position, not about mixing kinds.
    let mut first = job_level_goal(WARRIOR, 5);
    first.as_array_mut().unwrap().push(talk_goal(2));
    assert!(
        load(first).is_ok(),
        "a job-level goal as the first ordered step loads"
    );

    for (label, goal) in [
        ("profile 0", job_level_goal(0, 5)),
        ("unknown profile 99", job_level_goal(99, 5)),
        ("level 0", job_level_goal(WARRIOR, 0)),
        ("level 21", job_level_goal(WARRIOR, 21)),
        (
            "duplicate objective id",
            json!([
                job_level_goal(WARRIOR, 5)[0].clone(),
                job_level_goal(WARRIOR, 5)[0].clone()
            ]),
        ),
        (
            "missing provenance",
            json!([{
                "kind": "job_level",
                "id": 1,
                "profile_id": WARRIOR,
                "level": 5,
                "source": []
            }]),
        ),
        (
            // F576: nothing re-checks a job-level goal when an earlier ordered step
            // completes, so a later job-level step could never be credited.
            "a job-level goal after an earlier ordered step",
            json!([talk_goal(1), {
                "kind": "job_level",
                "id": 2,
                "profile_id": WARRIOR,
                "level": 5,
                "source": [{
                    "kind": "reconstructed",
                    "evidence": "(R) F576 synthetic later job_level step"
                }]
            }]),
        ),
    ] {
        assert!(load(goal).is_err(), "{label} must refuse the whole load");
    }
}
