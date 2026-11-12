//! Combat quest steps (combat design section 12): defeat any of several kinds, defeat in a
//! place, conditional defeats, `any_of` over defeats, entering a fight, and the save shape.
//!
//! Every test here drives a real request path:
//!
//! * a field kill is the 26/8 menu and 26/10 Attack through `Gameplay::handle_in_world`
//!   (`kill_field_enemy`), or `Gameplay::handle_shared_field` and the shared tick for two
//!   players;
//! * a kill inside a fight is the 36/10 basic attack in the fixture's dungeon;
//! * entering a fight is the gate click, the 41/114 start screen, 41/108 Go and the arena's
//!   zone ready.
//!
//! The quest catalogue is the production split loader over a scratch copy of `data/content`
//! with one synthetic quest added, because no shipped quest uses the new step shapes yet (this
//! piece enables none). Facts the fight engine does not report yet (`ability`, `item`, `area`,
//! fight goals, freed captives) are covered next to their consumer, in
//! `src/gameplay/quest_combat.rs`.
#[path = "common/combat_scale.rs"]
mod combat_scale;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, character_at_for, content_dir, counts, has_family, interact,
    kill_field_enemy, objective_complete, objective_update, quest_status, read_json, seed_quest,
    send,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    encounter::{
        ActiveEncounter, EncounterPhase, EnemyState, KillFact, Rank, SharedFieldAction,
        SharedFieldState,
    },
    gameplay::{Gameplay, GameplayState, Outcome},
    item_bar::use_wire::{Target, encode_client_request_start_ability},
    quest_engine::{
        CharacterEligibility, ObjectiveKind, QuestEngine, QuestEvent, QuestProgress, QuestState,
        QuestStatus,
    },
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs};

const A: u64 = CHARACTER;
const B: u64 = 702;
const BRAWLER: i32 = 43;
const ARCHER: i32 = 35;
const QUEST: u32 = 9_200_101;
const GIVER: u64 = 9_000_200_101;
/// Three synthetic enemy kinds. None is an opening-story target.
const BIXIE: u32 = 940_201;
const ROBGOBLIN: u32 = 940_202;
const TROLL: u32 = 940_203;
const FIRST_GUID: u64 = 1_790_001;
const OPEN_WORLD: u32 = 0;

fn source() -> Value {
    json!([{"kind": "reconstructed", "evidence": "(R) scratch combat step"}])
}

fn goal(id: u32, mut body: Value) -> Value {
    body["id"] = json!(id);
    body["source"] = source();
    body
}

/// A field enemy one Brawler swing kills.
fn enemy(index: u64, definition_id: u32, swings: u32) -> Value {
    json!({
        "guid": FIRST_GUID + index,
        "definition_id": definition_id,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic target",
        "texture_alias": "",
        "position": spot(index),
        "max_health": combat_scale::authored_for(combat_scale::strike(BRAWLER, 1), swings),
        "damage": 1,
        "boss": false,
        "level": 1,
        "respawn_seconds": 600,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode":"stationary","reach":2.5,"attack_ms":1000},
        "quest_ids": []
    })
}

fn spot(index: u64) -> [f32; 3] {
    [index as f32 * 30.0, 0.0, 0.0]
}

/// Add the one synthetic quest to this fixture's narrowed catalogue. The split loader wants
/// every quest listed by a `jobs.json` record, so it is added there too.
fn write_quest(fixture: &Fixture, goals: Vec<Value>) {
    let catalogue = fixture.quest_catalogue();
    let record = json!({
        "id": QUEST,
        "title_string_id": 700_020,
        "giver_npc_id": GIVER,
        "prerequisite_quest_ids": [],
        "objectives": goals,
        "rewards": [{"coins": 5, "items": [], "job_xp": {}, "source": source()}],
        "source": source(),
    });
    fs::write(
        catalogue.join("quests/combat-steps.json"),
        serde_json::to_vec(&json!({"schema_version": 1, "records": [record]})).unwrap(),
    )
    .unwrap();
    let jobs_path = catalogue.join("jobs.json");
    let mut jobs = read_json(&jobs_path);
    jobs["records"][0]["quest_ids"]
        .as_array_mut()
        .expect("the fixture jobs list quest ids")
        .push(json!(QUEST));
    fs::write(&jobs_path, serde_json::to_vec(&jobs).unwrap()).unwrap();
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
        ..Default::default()
    }
}

/// The scratch quest accepted, as the saved `QuestState` the production bind restores.
fn accepted(fixture: &Fixture, guid: u64) -> QuestState {
    let mut engine =
        QuestEngine::from_split_content(&fixture.quest_catalogue(), guid, eligibility(), None)
            .expect("the scratch combat catalogue loads");
    assert!(engine.offer(QUEST, GIVER).changed, "the quest is offered");
    assert!(engine.accept(QUEST).changed, "the quest is accepted");
    engine.state().clone()
}

fn fixture(goals: Vec<Value>, enemies: &[Value]) -> Fixture {
    // No synthetic field: keep the shipped world (its battle gates) as the fixture has it.
    let fixture = if enemies.is_empty() {
        Fixture::for_character(A)
    } else {
        Fixture::for_character(A).with_field_enemies(enemies)
    };
    write_quest(&fixture, goals);
    fixture
}

fn state(fixture: &Fixture, guid: u64, profile: i32) -> GameplayState {
    let mut state = Gameplay::load(fixture.path(), None).unwrap().state();
    state.npc_quests = Some(accepted(fixture, guid));
    let profiles = state.profiles.as_mut().expect("the fixture has profiles");
    profiles.active = profile;
    profiles.unlocked.insert(profile);
    state
}

/// A player on `profile` with the scratch quest active, world-loaded at `at`.
fn player(fixture: &Fixture, guid: u64, profile: i32, at: [f32; 3]) -> Gameplay {
    let mut game = fixture.game_as(guid, Some(state(fixture, guid, profile)));
    game.initialize_world(&character_at_for(guid, OPEN_WORLD, at), 0)
        .unwrap();
    game
}

/// Kill field enemy `index` through the real menu path.
fn kill(game: &mut Gameplay, index: u64, now: &mut u64) {
    assert!(
        kill_field_enemy(game, spot(index), FIRST_GUID + index, now),
        "field enemy {index} dies"
    );
}

#[test]
fn defeat_any_of_several_kinds_counts_only_the_listed_kinds() {
    let fixture = fixture(
        vec![goal(
            1,
            json!({"kind": "defeat", "kinds": [BIXIE, ROBGOBLIN], "count": 2}),
        )],
        &[
            enemy(0, TROLL, 1),
            enemy(1, BIXIE, 1),
            enemy(2, ROBGOBLIN, 1),
        ],
    );
    let mut game = player(&fixture, A, BRAWLER, spot(0));
    let mut now = 0;
    kill(&mut game, 0, &mut now);
    assert_eq!(
        counts(&game, QUEST),
        [0],
        "a kind off the list counts nothing"
    );
    kill(&mut game, 1, &mut now);
    assert_eq!(counts(&game, QUEST), [1]);
    kill(&mut game, 2, &mut now);
    assert_eq!(
        counts(&game, QUEST),
        [2],
        "each listed kind counts once per kill"
    );
}

#[test]
fn an_any_of_over_defeats_shares_one_counter() {
    let fixture = fixture(
        vec![goal(
            1,
            json!({"kind": "any_of", "count": 2, "alternatives": [
                {"kind": "defeat", "model_id": BIXIE},
                {"kind": "defeat", "model_id": ROBGOBLIN},
            ]}),
        )],
        &[
            enemy(0, TROLL, 1),
            enemy(1, BIXIE, 1),
            enemy(2, ROBGOBLIN, 1),
        ],
    );
    let mut game = player(&fixture, A, BRAWLER, spot(0));
    let mut now = 0;
    kill(&mut game, 0, &mut now);
    assert_eq!(counts(&game, QUEST), [0]);
    kill(&mut game, 1, &mut now);
    kill(&mut game, 2, &mut now);
    assert_eq!(counts(&game, QUEST), [2]);
}

#[test]
fn defeat_in_a_zone_counts_any_enemy_there_and_none_elsewhere() {
    // "Defeat enemies in <place> 0/2": no kind at all, the place decides.
    let here = fixture(
        vec![goal(
            1,
            json!({"kind": "defeat", "in_zone": OPEN_WORLD, "count": 2}),
        )],
        &[enemy(0, TROLL, 1), enemy(1, BIXIE, 1)],
    );
    let mut game = player(&here, A, BRAWLER, spot(0));
    let mut now = 0;
    kill(&mut game, 0, &mut now);
    kill(&mut game, 1, &mut now);
    assert_eq!(counts(&game, QUEST), [2], "every kind in the zone counts");

    let elsewhere = fixture(
        vec![goal(
            1,
            json!({"kind": "defeat", "model_id": TROLL, "in_zone": 9_600, "count": 1}),
        )],
        &[enemy(0, TROLL, 1)],
    );
    let mut game = player(&elsewhere, A, BRAWLER, spot(0));
    kill(&mut game, 0, &mut 0);
    assert_eq!(
        counts(&game, QUEST),
        [0],
        "the right kind in the wrong zone counts nothing"
    );
}

#[test]
fn a_conditional_defeat_checks_the_job_and_solo() {
    let with = |job: i32| {
        json!({"kind": "defeat", "model_id": TROLL, "count": 1,
            "with": {"solo": true, "job": job}})
    };
    let named = fixture(vec![goal(1, with(BRAWLER))], &[enemy(0, TROLL, 1)]);
    let mut game = player(&named, A, BRAWLER, spot(0));
    kill(&mut game, 0, &mut 0);
    assert_eq!(counts(&game, QUEST), [1], "alone and on the named job");

    // The same kill by the same Brawler is not "as an Archer".
    let other = fixture(vec![goal(1, with(ARCHER))], &[enemy(0, TROLL, 1)]);
    let mut game = player(&other, A, BRAWLER, spot(0));
    kill(&mut game, 0, &mut 0);
    assert_eq!(
        counts(&game, QUEST),
        [0],
        "another job's step is not counted"
    );
}

fn shared_attack(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    target: u64,
    now_ms: u64,
) -> Outcome {
    game.handle_shared_field(
        &SharedFieldAction::Ability { slot: 0, target },
        field,
        &BTreeSet::from([target]),
        character,
        now_ms,
    )
    .unwrap()
}

/// B lands one hit, A the lethal one; B then claims its credit on its shared tick.
fn two_players_kill_one(fixture: &Fixture) -> (Gameplay, Gameplay) {
    let a_character = character_at_for(A, OPEN_WORLD, [1.0, 0.0, 0.0]);
    let b_character = character_at_for(B, OPEN_WORLD, [-1.0, 0.0, 0.0]);
    let mut a = player(fixture, A, BRAWLER, a_character.position);
    let mut b = player(fixture, B, BRAWLER, b_character.position);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    shared_attack(&mut b, &mut field, &b_character, FIRST_GUID, 0);
    shared_attack(&mut a, &mut field, &a_character, FIRST_GUID, 100);
    assert_eq!(
        field.health(FIRST_GUID),
        Some(0),
        "the shared enemy is dead"
    );
    b.tick_shared_effects(&mut field, &b_character, 150)
        .unwrap();
    (a, b)
}

#[test]
fn a_shared_field_kill_credits_both_players_a_rule_step() {
    // A two-swing enemy, so B's hit does not kill it.
    let fixture = fixture(
        vec![goal(
            1,
            json!({"kind": "defeat", "kinds": [TROLL, BIXIE], "count": 1}),
        )],
        &[enemy(0, TROLL, 2)],
    );
    let (a, b) = two_players_kill_one(&fixture);
    assert_eq!(counts(&a, QUEST), [1], "the killer gets a full count");
    assert_eq!(counts(&b, QUEST), [1], "so does the player who helped");
}

#[test]
fn a_solo_step_counts_for_nobody_when_two_players_share_the_kill() {
    let fixture = fixture(
        vec![goal(
            1,
            json!({"kind": "defeat", "model_id": TROLL, "count": 1, "with": {"solo": true}}),
        )],
        &[enemy(0, TROLL, 2)],
    );
    let (a, b) = two_players_kill_one(&fixture);
    assert_eq!(
        counts(&a, QUEST),
        [0],
        "two names on the credit list is not solo"
    );
    assert_eq!(counts(&b, QUEST), [0]);
}

/// Put the saved character inside the fixture's own dungeon, mid-fight.
fn enter_dungeon(
    state: &mut GameplayState,
    fixture: &Fixture,
    guid: u64,
) -> (u32, Vec<(u64, u32)>) {
    let content = read_json(&fixture.path().join("encounters.json"));
    let dungeon = &content["dungeons"][0];
    let dungeon_id = u32::try_from(dungeon["id"].as_u64().unwrap()).unwrap();
    let rows = dungeon["enemies"].as_array().unwrap();
    let encounter = state
        .encounter
        .as_mut()
        .expect("the fixture has encounters");
    encounter.next_instance_id = encounter.next_instance_id.max(2);
    encounter.active = Some(ActiveEncounter {
        encounter_id: dungeon_id,
        instance_id: 1,
        phase: EncounterPhase::Fighting,
        return_location: character_at_for(guid, OPEN_WORLD, [0.0, 0.0, 0.0]),
        enemies: rows
            .iter()
            .map(|actor| EnemyState {
                guid: actor["guid"].as_u64().unwrap(),
                health: u32::try_from(actor["max_health"].as_u64().unwrap()).unwrap(),
                position: serde_json::from_value(actor["position"].clone()).unwrap(),
                dormant: false,
            })
            .collect(),
        run_save: None,
    });
    let kinds = rows
        .iter()
        .map(|actor| {
            (
                actor["guid"].as_u64().unwrap(),
                u32::try_from(actor["definition_id"].as_u64().unwrap()).unwrap(),
            )
        })
        .collect();
    (dungeon_id, kinds)
}

/// Swing the 36/10 basic attack at `target` until the step moves or the swings run out.
fn fight_until_counted(game: &mut Gameplay, character: &CharacterState, target: u64) -> Vec<u32> {
    let mut now = 1_000;
    for _ in 0..200 {
        now += 700;
        let packet = encode_client_request_start_ability(1, 0, Target::Guid(target));
        let out = send(game, character, &packet, now);
        let gone = out.replies.iter().any(|packet| {
            has_family(packet, 35, 3, None) && packet.get(6..14) == Some(&target.to_le_bytes()[..])
        });
        if gone || counts(game, QUEST) != [0] {
            break;
        }
    }
    counts(game, QUEST)
}

/// Kill the first enemy of the fixture's dungeon with a quest whose step names fight
/// `in_fight(dungeon id)`; returns the step's count.
fn dungeon_kill(in_fight: impl Fn(u32) -> u32) -> Vec<u32> {
    let probe = Fixture::for_character(A);
    let dungeon_id = read_json(&probe.path().join("encounters.json"))["dungeons"][0]["id"]
        .as_u64()
        .unwrap() as u32;
    let fixture = fixture(
        vec![goal(
            1,
            json!({"kind": "defeat", "in_fight": in_fight(dungeon_id), "count": 1}),
        )],
        &[],
    );
    let mut saved = state(&fixture, A, BRAWLER);
    let (_, kinds) = enter_dungeon(&mut saved, &fixture, A);
    let mut game = fixture.game_as(A, Some(saved));
    let dungeon = read_json(&fixture.path().join("encounters.json"));
    let enemy = &dungeon["dungeons"][0]["enemies"][0];
    let position: [f32; 3] = serde_json::from_value(enemy["position"].clone()).unwrap();
    let zone = dungeon["dungeons"][0]["zone"]["zone_id"].as_u64().unwrap() as u32;
    let character = character_at_for(A, zone, position);
    game.initialize_world(&character, 0).unwrap();
    fight_until_counted(&mut game, &character, kinds[0].0)
}

#[test]
fn defeat_in_a_fight_counts_kills_inside_that_fight_only() {
    assert_eq!(
        dungeon_kill(|dungeon| dungeon),
        [1],
        "a kill inside the named fight counts"
    );
    assert_eq!(
        dungeon_kill(|dungeon| dungeon + 1),
        [0],
        "the same kill does not count for another fight"
    );
}

/// King of the Ring's gate at Cobblestone, its battle and arena (`field_battles.json` 12400).
const RING: u32 = 12_400;
const GATE: u64 = 100_000_124_000;
const GATE_POSITION: [f32; 3] = [-1908.92576, -39.4, 410.69792];
const ARENA_ZONE: u32 = 9_600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];

#[test]
fn entering_a_fight_completes_the_enter_step_and_the_win_the_next() {
    let fixture = fixture(
        vec![
            goal(1, json!({"kind": "enter_fight", "fight_id": RING})),
            goal(2, json!({"kind": "complete_dungeon", "encounter_id": RING})),
        ],
        &[],
    );
    let mut at = character_at_for(A, OPEN_WORLD, GATE_POSITION);
    let mut game = fixture.game_as(A, Some(state(&fixture, A, BRAWLER)));
    game.initialize_world(&at, 0).unwrap();
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    assert_eq!(
        counts(&game, QUEST),
        [0, 0],
        "standing at the gate is not entering"
    );

    // Gate -> 41/114 start screen -> 41/108 Go -> the arena's zone ready.
    let screen = interact(&mut game, &at, GATE, 1_100);
    let details = screen
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
        .cloned()
        .expect("the gate opens the 41/114 start screen");
    let mut go = vec![41, 0, 108, 0];
    go.extend(&details[4..12]);
    go.extend(0_u64.to_le_bytes());
    let zoned = send(&mut game, &at, &go, 1_200);
    assert_eq!(zoned.zoning.expect("Go zones").zone_id, ARENA_ZONE);
    assert_eq!(counts(&game, QUEST), [0, 0], "Go alone is not inside yet");
    at.zone_id = ARENA_ZONE;
    at.position = ARENA_ARRIVAL;
    let loaded = game.initialize_world(&at, 1_300).unwrap();
    assert_eq!(
        counts(&game, QUEST),
        [1, 0],
        "the loaded arena is the entry"
    );
    assert!(objective_complete(&loaded, QUEST, 1));

    // A reconnect into the running fight is not a second entry and not a win.
    game.initialize_world(&at, 1_350).unwrap();
    assert_eq!(counts(&game, QUEST), [1, 0]);

    let won = send(&mut game, &at, &chat::local_request("realm win"), 1_400);
    assert_eq!(
        counts(&game, QUEST),
        [1, 1],
        "the win is the fight's `Won` fact"
    );
    assert!(objective_update(&won, QUEST, 2, 1));
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
}

fn kill_of(kind: u32) -> QuestEvent {
    QuestEvent::Kill {
        fact: KillFact {
            kind,
            guid: 1,
            rank: Rank::Normal,
            tags: Vec::new(),
            zone: OPEN_WORLD,
            area: None,
            fight: None,
            solo: true,
            job: BRAWLER as u32,
            ability: None,
            item: None,
        },
    }
}

/// The shipped quests this piece must leave working: seven `defeat` steps and four
/// `complete_dungeon` steps (`data/content/quests`), as (quest, step index, target).
const SHIPPED_DEFEATS: [(u32, usize, u32); 7] = [
    (3_000_009, 0, 3_156),
    (3_000_010, 0, 920_001),
    (3_000_024, 0, 930_050),
    (3_000_029, 1, 3_156),
    (3_010_011, 0, 1_294),
    (3_030_007, 0, 4_154), // L1192: the Hooligan Delinquent (930050 is the Alpha Wolf)
    (3_110_002, 0, 930_096),
];
const SHIPPED_DUNGEONS: [(u32, usize, u32); 4] = [
    (3_000_032, 0, 12_340),
    (3_000_061, 0, 12_350),
    (3_030_004, 1, 12_400),
    (3_030_005, 0, 12_400),
];

/// The shipped catalogue with `quest` active and every step before `step` done.
fn shipped_engine_at(quest: u32, step: usize) -> QuestEngine {
    let engine = QuestEngine::from_split_content(&content_dir(), A, eligibility(), None).unwrap();
    let mut state = engine.state().clone();
    let required: Vec<u32> = engine_required(&engine, quest);
    let counts = required
        .iter()
        .enumerate()
        .map(|(index, required)| if index < step { *required } else { 0 })
        .collect();
    state.quests.insert(
        quest,
        QuestProgress {
            status: QuestStatus::Active,
            counts,
        },
    );
    QuestEngine::from_split_content(&content_dir(), A, eligibility(), Some(state)).unwrap()
}

/// Every step's required count, read from the shipped record itself.
fn engine_required(engine: &QuestEngine, quest: u32) -> Vec<u32> {
    assert!(engine.is_enabled(quest), "shipped quest {quest} is enabled");
    for entry in fs::read_dir(content_dir().join("quests")).unwrap() {
        let document = read_json(&entry.unwrap().path());
        let Some(record) = document["records"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|record| record["id"] == quest)
        else {
            continue;
        };
        return record["objectives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|goal| {
                ["count", "quantity"]
                    .iter()
                    .find_map(|key| goal[*key].as_u64())
                    .unwrap_or(1) as u32
            })
            .collect();
    }
    panic!("quest {quest} is not in data/content/quests");
}

#[test]
fn the_shipped_defeat_and_dungeon_quests_still_count_from_facts() {
    for (quest, step, kind) in SHIPPED_DEFEATS {
        let mut engine = shipped_engine_at(quest, step);
        // A plain one-kind defeat keeps the kind every save was built on.
        assert_eq!(
            engine.current_objectives(quest)[0].kind,
            ObjectiveKind::KillActorKind {
                actor_kind_id: kind
            },
            "quest {quest}"
        );
        assert!(
            !engine.on_event(kill_of(kind + 1)).unwrap().changed,
            "quest {quest}: another kind counts nothing"
        );
        assert!(
            engine.on_event(kill_of(kind)).unwrap().changed,
            "quest {quest}: its own kind counts from a kill fact"
        );
        assert_eq!(engine.progress(quest).unwrap().counts[step], 1);
    }
    for (quest, step, fight) in SHIPPED_DUNGEONS {
        let mut engine = shipped_engine_at(quest, step);
        let won = engine
            .on_event(QuestEvent::CompleteDungeon {
                encounter_id: fight,
            })
            .unwrap();
        assert!(won.changed, "quest {quest}: the win counts");
        assert_eq!(engine.progress(quest).unwrap().counts[step], 1);
    }
}

#[test]
fn a_save_with_defeat_progress_loads_and_keeps_counting() {
    // "Fighting the Pack": three of five wolves down, saved before this piece. The save holds
    // counts only, and the step is still a plain kind, so it loads as it was.
    const PACK: u32 = 3_000_009;
    const WOLF: u32 = 3_156;
    let fixture = Fixture::for_character(A).with_field_enemies(&[enemy(0, WOLF, 1)]);
    let mut saved = Gameplay::load(fixture.path(), None).unwrap().state();
    saved.npc_quests = Some(
        QuestEngine::from_split_content(&fixture.quest_catalogue(), A, eligibility(), None)
            .unwrap()
            .state()
            .clone(),
    );
    seed_quest(&mut saved, PACK, QuestStatus::Active, &[3, 0]);
    {
        let profiles = saved.profiles.as_mut().unwrap();
        profiles.active = BRAWLER;
        profiles.unlocked.insert(BRAWLER);
    }
    // Through the serialized form, exactly as a stored character comes back.
    let bytes = serde_json::to_vec(&saved).unwrap();
    let restored: GameplayState = serde_json::from_slice(&bytes).unwrap();
    let mut game = fixture.game_as(A, Some(restored));
    game.initialize_world(&character_at_for(A, OPEN_WORLD, spot(0)), 0)
        .unwrap();
    assert_eq!(
        counts(&game, PACK),
        [3, 0],
        "the saved progress loads unchanged"
    );
    kill(&mut game, 0, &mut 0);
    assert_eq!(counts(&game, PACK), [4, 0], "and the next kill counts");
}

#[test]
fn bad_combat_steps_are_refused_at_load() {
    let cases = [
        (
            json!({"kind": "defeat", "count": 1}),
            "no kind and no place",
        ),
        (
            json!({"kind": "defeat", "model_id": TROLL, "kinds": [BIXIE], "count": 1}),
            "model_id and kinds together",
        ),
        (
            json!({"kind": "defeat", "kinds": [BIXIE, BIXIE], "count": 1}),
            "a repeated kind",
        ),
        (
            json!({"kind": "defeat", "model_id": TROLL, "count": 1, "with": {}}),
            "an empty condition",
        ),
        (
            json!({"kind": "defeat", "model_id": TROLL, "count": 1, "in_fight": 0}),
            "fight zero",
        ),
        (json!({"kind": "enter_fight", "fight_id": 0}), "fight zero"),
        (
            json!({"kind": "fight_goal", "fight_id": RING, "goal": "", "count": 1}),
            "an unnamed goal",
        ),
        (
            json!({"kind": "fight_goal", "fight_id": RING, "goal": "hold", "count": 0}),
            "a zero count",
        ),
        (
            json!({"kind": "interact_actor", "actor_guid": 5, "actor_kind_id": TROLL}),
            "a kind on an unguarded click",
        ),
        (
            json!({"kind": "interact_actor", "guarded": true}),
            "a captive with no target",
        ),
        (
            json!({"kind": "interact_actor", "guarded": true, "actor_guid": 5, "count": 3}),
            "one captive freed three times",
        ),
    ];
    for (body, label) in cases {
        let fixture = Fixture::for_character(A);
        write_quest(&fixture, vec![goal(1, body)]);
        let loaded =
            QuestEngine::from_split_content(&fixture.quest_catalogue(), A, eligibility(), None);
        assert!(loaded.is_err(), "{label} must not load");
    }
    // The well-formed shapes do load, each as its own step kind.
    let fixture = Fixture::for_character(A);
    write_quest(
        &fixture,
        vec![
            goal(
                1,
                json!({"kind": "fight_goal", "fight_id": RING, "goal": "hold", "count": 3}),
            ),
            goal(
                2,
                json!({"kind": "interact_actor", "guarded": true, "actor_kind_id": TROLL, "count": 5}),
            ),
            goal(
                3,
                json!({"kind": "defeat", "model_id": TROLL, "count": 4,
                "with": {"ability_id": 62_019, "item_id": 11_002}}),
            ),
        ],
    );
    QuestEngine::from_split_content(&fixture.quest_catalogue(), A, eligibility(), None)
        .expect("the well-formed combat steps load");
}
