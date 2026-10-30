//! T789: accepted job-ability casts and spent single-use items credit `use_ability` /
//! `use_item` quest objectives.
//!
//! Every test drives the real request path: `Gameplay::handle_in_world` (the 36/10 targeted
//! ability and item-bar requests) or `Gameplay::handle_shared_field` (the shared-field cast),
//! with the shipped request encoders from `item_bar::use_wire` (`00a342e0` body, sender
//! `00a35150`; 42/6 assign `00c14720`; 42/16 check `00c109f0`; 42/15 use `00c10940`). The quest
//! catalogue is the production split loader over a scratch copy of `data/content`; only the
//! objective under test differs, because the shipped quests that name these steps are still
//! disabled (line endings and provenance stay in the authored files). The one exception is
//! Pondblaster Payback (3010011), enabled by T895 and walked here from the shipped data.
//!
//! The session fixture, saved-state round trip and quest packet helpers come from the canonical
//! `tests/support/npc_quest_chain.rs`; this file adds only the synthetic field population, the
//! use-goal catalogue and the ability/item request helpers.
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    Fixture, accept, assert_packet_changes_nothing, character_at_for, content_dir,
    content_position, counts, finish_quest, has_family, npc_interact, npc_select,
    objective_complete, objective_update, offer, pending_turnin, quest_reply, quest_status,
    read_json, release_end_screen, saved, seed_quest, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    encounter::{ActiveEncounter, EncounterPhase, EnemyState, SharedFieldAction},
    gameplay::{Gameplay, GameplayState, Outcome},
    inventory::InventoryItem,
    item_bar::use_wire::{
        ImmediateActivation, Target, encode_client_request_start_ability,
        encode_immediate_activation, encode_item_action_bar_assign,
    },
    quest_engine::{CharacterEligibility, QuestEngine, QuestState, QuestStatus},
};
use quest_records::shipped_quest;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs};

/// Two players for the shared-field test.
const A: u64 = 701;
const B: u64 = 702;
const ARCHER: i32 = 35;
/// Archer: Achieve Level 5, the shipped prerequisite of Pondblaster Payback.
const ARCHER_5: u32 = 3_010_009;
/// Archer toolbar 2, level 5 (`job_abilities.json` definition 62019, name 4830 "Explosive Arrow").
const EXPLOSIVE_ARROW: i32 = 62019;
/// The shipped `Robgoblin Pondblasters` enemy definition (`enemies.json` row 1294).
const POND_BLASTER: u32 = 1294;
/// A definition that is not the objective's target.
const OTHER_ENEMY: u32 = 1296;
/// A single-use potion row the built-in `consumables.json` table carries (35% heal, 10 s).
const POTION: i32 = 11002;
/// A catalogued boombox (`boomboxes.json`): pressing it never spends the item.
const BOOMBOX: i32 = 8185;
const QUEST: u32 = 9_200_001;
const GIVER: u64 = 9_000_200_001;
const OBJECTIVE: u32 = 1;
const ENEMY_GUID: u64 = 1_789_001;
const ENEMY_HEALTH: u32 = 50_000;

fn quest_source() -> Value {
    json!([{"kind": "reconstructed", "evidence": "(R) T789 scratch use goal"}])
}

fn ability_goal(ability_id: i32, count: u32, target: Option<u32>) -> Value {
    let mut goal = json!({
        "kind": "use_ability",
        "id": OBJECTIVE,
        "ability_id": ability_id,
        "count": count,
        "source": quest_source(),
    });
    if let Some(target) = target {
        goal["target_actor_kind_id"] = json!(target);
    }
    goal
}

fn item_goal(item_id: i32, count: u32) -> Value {
    json!({
        "kind": "use_item",
        "id": OBJECTIVE,
        "item_id": item_id,
        "count": count,
        "source": quest_source(),
    })
}

/// One synthetic field actor of `definition_id`; `quest_ids` empty means always present.
fn enemy(definition_id: u32, position: [f32; 3]) -> Value {
    json!({
        "guid": ENEMY_GUID,
        "definition_id": definition_id,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic target",
        "texture_alias": "",
        "position": position,
        "max_health": ENEMY_HEALTH,
        "damage": 1,
        "boss": false,
        "level": 1,
        "respawn_seconds": 60,
        "aggro_range": 8.0,
        "leash_radius": 40.0,
        "behaviour": {"mode":"stationary","reach":2.5,"attack_ms":1000},
        "quest_ids": []
    })
}

/// Write the one synthetic quest into this fixture's narrowed catalogue. The split loader
/// requires every quest to be listed by a `jobs.json` record, so the quest is added there too.
fn write_quest(fixture: &Fixture, goal: Value) {
    let catalogue = fixture.quest_catalogue();
    let record = json!({
        "id": QUEST,
        "title_string_id": 700_020,
        "giver_npc_id": GIVER,
        "prerequisite_quest_ids": [],
        "objectives": [goal],
        "rewards": [{"coins": 5, "items": [], "job_xp": {}, "source": quest_source()}],
        "source": quest_source(),
    });
    fs::write(
        catalogue.join("quests/use-events.json"),
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
        nonmember_quest_limit: 20,
        member_quest_limit: 20,
        ..Default::default()
    }
}

/// The accepted quest as a saved `QuestState`, built by the production split loader.
fn accepted(fixture: &Fixture, guid: u64) -> QuestState {
    let catalogue = fixture.quest_catalogue();
    let mut engine = QuestEngine::from_split_content(&catalogue, guid, eligibility(), None)
        .expect("the scratch use-goal catalogue loads");
    assert!(
        engine.offer(QUEST, GIVER).changed,
        "the scratch use quest is offered"
    );
    assert!(
        engine.accept(QUEST).changed,
        "the scratch use quest is accepted"
    );
    engine.state().clone()
}

fn use_fixture(goal: Value, enemies: &[Value]) -> Fixture {
    let fixture = Fixture::for_character(A).with_field_enemies(enemies);
    // Only the two files `Gameplay::load` needs beyond the opening fixture: the job roster and
    // the boombox catalogue. `consumables.json` is absent on purpose: the built-in potion table
    // (2173, 11002) is the production fallback and carries the potion this suite uses.
    fixture.copy_content("job_abilities.json");
    fixture.copy_content("boomboxes.json");
    write_quest(&fixture, goal);
    fixture
}

fn give_item(state: &mut GameplayState, definition_id: i32, count: u32) -> i32 {
    let id = state
        .inventory
        .items
        .iter()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        + 1;
    state.inventory.items.push(InventoryItem {
        definition_id,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    id
}

/// The fresh opening save has no outfit. Give the Archer the shipped class-18 bow
/// (`inventory.json` definition 4287) in native weapon slot 7, so the cast gate admits the job
/// ability exactly as it does for a played character's gear.
fn equip_archer_bow(state: &mut GameplayState) {
    let instance = give_item(state, 4_287, 1);
    let row = state
        .inventory
        .equipment
        .iter_mut()
        .find(|entry| entry.profile_id == ARCHER && entry.slot == 7)
        .expect("the opening fixture has the Archer weapon slot");
    row.item_id = instance;
}

/// An Archer at rank 5 (400 XP on the local 100-XP curve) with an active scratch quest, before
/// the world is initialized.
fn archer_state(fixture: &Fixture, guid: u64) -> GameplayState {
    let mut state = Gameplay::load(fixture.path(), None).unwrap().state();
    state.npc_quests = Some(accepted(fixture, guid));
    {
        let profiles = state.profiles.as_mut().expect("the fixture has profiles");
        profiles.active = ARCHER;
        profiles.unlocked.insert(ARCHER);
        profiles.experience.insert(ARCHER, 400);
    }
    equip_archer_bow(&mut state);
    state
}

/// An Archer at rank 5 with an active scratch quest, world-loaded.
fn archer(fixture: &Fixture, guid: u64, position: [f32; 3]) -> Gameplay {
    let state = archer_state(fixture, guid);
    let mut game = fixture.game_as(guid, Some(state));
    game.initialize_world(&character_at_for(guid, 0, position), 0)
        .unwrap();
    game
}

/// Put the character in the fixture's own dungeon (411) so the ability path reads an active
/// encounter. `Lost` with zero health is a knocked-out player; `Won` with a cleared, rewarded
/// run is a live player in the wrong phase. Both are save shapes the encounter loader validates.
fn enter_dungeon(
    state: &mut GameplayState,
    fixture: &Fixture,
    guid: u64,
    phase: EncounterPhase,
    player_health: u32,
) {
    let content = read_json(&fixture.path().join("encounters.json"));
    let dungeon = &content["dungeons"][0];
    let dungeon_id = u32::try_from(dungeon["id"].as_u64().unwrap()).unwrap();
    let cleared = phase == EncounterPhase::Won;
    let enemies: Vec<EnemyState> = dungeon["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|actor| EnemyState {
            guid: actor["guid"].as_u64().unwrap(),
            health: if cleared {
                0
            } else {
                u32::try_from(actor["max_health"].as_u64().unwrap()).unwrap()
            },
            position: serde_json::from_value(actor["position"].clone()).unwrap(),
            dormant: false,
        })
        .collect();
    let encounter = state
        .encounter
        .as_mut()
        .expect("the fixture loads an encounter engine");
    encounter.next_instance_id = encounter.next_instance_id.max(2);
    encounter.player_health = player_health;
    if cleared {
        encounter.rewarded.insert(dungeon_id);
    }
    encounter.active = Some(ActiveEncounter {
        encounter_id: dungeon_id,
        instance_id: 1,
        phase,
        return_location: character_at_for(guid, 0, [0.0, 0.0, 0.0]),
        enemies,
    });
}

/// An Archer inside the fixture's dungeon in `phase`, world-loaded.
fn dungeon_player(
    fixture: &Fixture,
    guid: u64,
    phase: EncounterPhase,
    player_health: u32,
) -> Gameplay {
    let mut state = archer_state(fixture, guid);
    enter_dungeon(&mut state, fixture, guid, phase, player_health);
    let mut game = fixture.game_as(guid, Some(state));
    game.initialize_world(&character_at_for(guid, 0, [0.0, 0.0, 0.0]), 0)
        .unwrap();
    game
}

/// A player holding a stack of `count` `definition_id` instances, with the active scratch
/// quest; returns the game and the owned instance id.
fn item_player(
    fixture: &Fixture,
    guid: u64,
    position: [f32; 3],
    definition_id: i32,
    count: u32,
) -> (Gameplay, i32) {
    let mut state = Gameplay::load(fixture.path(), None).unwrap().state();
    state.npc_quests = Some(accepted(fixture, guid));
    let instance = give_item(&mut state, definition_id, count);
    let mut game = fixture.game_as(guid, Some(state));
    game.initialize_world(&character_at_for(guid, 0, position), 0)
        .unwrap();
    (game, instance)
}

fn cast(
    game: &mut Gameplay,
    character: &CharacterState,
    slot: i32,
    target: u64,
    now: u64,
) -> Outcome {
    let packet = encode_client_request_start_ability(1, slot, Target::Guid(target));
    send(game, character, &packet, now)
}

fn activate_item(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: u64,
) -> Outcome {
    activate_item_slot(game, character, 0, target, now)
}

fn activate_item_slot(
    game: &mut Gameplay,
    character: &CharacterState,
    slot: i32,
    target: u64,
    now: u64,
) -> Outcome {
    let packet = encode_client_request_start_ability(2, slot, Target::Guid(target));
    send(game, character, &packet, now)
}

fn assign(game: &mut Gameplay, character: &CharacterState, instance: i32, now: u64) -> Outcome {
    assign_slot(game, character, 0, instance, now)
}

fn assign_slot(
    game: &mut Gameplay,
    character: &CharacterState,
    slot: i32,
    instance: i32,
    now: u64,
) -> Outcome {
    send(
        game,
        character,
        &encode_item_action_bar_assign(slot, instance),
        now,
    )
}

fn quest_packets(outcome: &Outcome) -> bool {
    outcome.replies.iter().any(|packet| {
        has_family(packet, 49, 8, None)
            || has_family(packet, 49, 9, None)
            || has_family(packet, 49, 10, None)
    })
}

fn assert_refused_nothing(game: &Gameplay, outcome: &Outcome, before: &[u8], label: &str) {
    assert!(!outcome.changed, "{label}: a refusal changed gameplay");
    assert!(
        !quest_packets(outcome),
        "{label}: a refusal emitted a quest progress packet"
    );
    assert_eq!(
        saved(game),
        before,
        "{label}: a refusal changed the serialized state"
    );
}

fn owned(game: &Gameplay, instance: i32) -> bool {
    stack(game, instance) > 0
}

fn stack(game: &Gameplay, instance: i32) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .find(|item| item.id == instance)
        .map_or(0, |item| item.count.saturating_sub(item.consumed_count))
}

#[test]
fn an_accepted_ability_on_the_target_credits_the_objective() {
    let fixture = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, Some(POND_BLASTER)),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = archer(&fixture, A, at);
    assert_eq!(counts(&game, QUEST), [0]);

    let out = cast(&mut game, &character, 2, ENEMY_GUID, 0);
    assert_eq!(counts(&game, QUEST), [1], "one accepted cast, one credit");
    assert!(
        objective_update(&out, QUEST, OBJECTIVE, 1),
        "49/9 must report the credited objective"
    );
    assert!(
        objective_complete(&out, QUEST, OBJECTIVE),
        "the single goal completes"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert!(game.state().encounter.as_ref().unwrap().active.is_none());
}

/// T789 regression: a lethal cast on the last enemy of a run is the same accepted cast a
/// `use_ability` step counts, but `win_run` moves the phase to Won before the event is built.
/// The target kind must be captured from the resolved actor before the blow lands.
#[test]
fn a_lethal_cast_that_ends_the_run_credits_the_targeted_goal() {
    let fixture = Fixture::for_character(A);
    fixture.copy_content("job_abilities.json");
    let dungeon = read_json(&fixture.path().join("encounters.json"))["dungeons"][0].clone();
    let actor = dungeon["enemies"][0].clone();
    let guid = actor["guid"].as_u64().unwrap();
    let kind = u32::try_from(actor["definition_id"].as_u64().unwrap()).unwrap();
    let zone = u32::try_from(dungeon["zone"]["zone_id"].as_u64().unwrap()).unwrap();
    let at = character_at_for(
        A,
        zone,
        serde_json::from_value(actor["position"].clone()).unwrap(),
    );
    write_quest(&fixture, ability_goal(EXPLOSIVE_ARROW, 1, Some(kind)));

    let mut state = archer_state(&fixture, A);
    enter_dungeon(&mut state, &fixture, A, EncounterPhase::Fighting, 1_000);
    {
        // Every other enemy is already down and the primary has one point left: this cast is
        // the run's killing blow, so the winning phase change happens inside the cast.
        let active = state.encounter.as_mut().unwrap().active.as_mut().unwrap();
        for enemy in &mut active.enemies {
            enemy.health = 0;
        }
        active
            .enemies
            .iter_mut()
            .find(|enemy| enemy.guid == guid)
            .expect("the primary is in the run")
            .health = 1;
    }
    let mut game = fixture.game_as(A, Some(state));
    game.initialize_world(&at, 0).unwrap();

    let out = cast(&mut game, &at, 2, guid, 0);
    assert_eq!(
        game.state()
            .encounter
            .as_ref()
            .unwrap()
            .active
            .as_ref()
            .unwrap()
            .phase,
        EncounterPhase::Won,
        "the cast was the run's killing blow"
    );
    assert_eq!(
        counts(&game, QUEST),
        [1],
        "the targeted goal ticks on the lethal cast"
    );
    assert!(objective_update(&out, QUEST, OBJECTIVE, 1));
    assert!(objective_complete(&out, QUEST, OBJECTIVE));
}

#[test]
fn refused_cast_credits_nothing() {
    // Cooldown: the first cast is accepted and credited; the immediate repeat is refused by the
    // per-slot deadline after the accepted cast already changed the saved state.
    let fixture = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 2, None),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = archer(&fixture, A, at);
    cast(&mut game, &character, 2, ENEMY_GUID, 0);
    assert_eq!(counts(&game, QUEST), [1]);
    let before = saved(&game);
    let refused = cast(&mut game, &character, 2, ENEMY_GUID, 0);
    assert_eq!(counts(&game, QUEST), [1], "a refused cast counts once");
    assert_refused_nothing(&game, &refused, &before, "cooldown");
    // The 36/1 refusal body is one i32 (client reader 00a31470), not a silent drop.
    assert!(
        refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[36, 0, 1, 0])),
        "the cooldown refusal must answer with 36/1"
    );

    // Out of reach: the client names the enemy but the server's range check refuses it.
    let far = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, None),
        &[enemy(POND_BLASTER, [1_000.0, 0.0, 0.0])],
    );
    let mut game = archer(&far, A, at);
    let before = saved(&game);
    let refused = cast(&mut game, &character, 2, ENEMY_GUID, 0);
    assert_eq!(counts(&game, QUEST), [0], "no reach means no credit");
    assert_refused_nothing(&game, &refused, &before, "no target in reach");

    // Not enough energy: six accepted casts leave 10 energy, the seventh costs 15.
    let mut game = archer(&fixture, A, at);
    for now in [0, 5_000, 10_000, 15_000, 20_000, 25_000] {
        cast(&mut game, &character, 2, ENEMY_GUID, now);
    }
    assert_eq!(counts(&game, QUEST), [2], "the goal is already complete");
    let before = saved(&game);
    let refused = cast(&mut game, &character, 2, ENEMY_GUID, 30_000);
    assert_refused_nothing(&game, &refused, &before, "not enough energy");
    assert_eq!(counts(&game, QUEST), [2]);

    // A knocked-out player (the saved Lost dungeon state) is refused before targeting.
    let dead = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, None),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let mut game = dungeon_player(&dead, A, EncounterPhase::Lost, 0);
    let before = saved(&game);
    let refused = cast(&mut game, &character, 2, 0, 0);
    assert_refused_nothing(&game, &refused, &before, "knocked out");
    assert_eq!(counts(&game, QUEST), [0]);

    // A live player in a finished run (the saved Won state) is refused by the phase check.
    let finished = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, None),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let mut game = dungeon_player(&finished, A, EncounterPhase::Won, 1_000);
    let before = saved(&game);
    let refused = cast(&mut game, &character, 2, 0, 0);
    assert_refused_nothing(&game, &refused, &before, "wrong phase");
    assert_eq!(counts(&game, QUEST), [0]);
}

#[test]
fn other_ability_changes_nothing() {
    let fixture = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, Some(POND_BLASTER)),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = archer(&fixture, A, at);
    let before = saved(&game);
    // Toolbar slot 1 resolves to another Archer ability (62020 Multi-Arrow), not the objective.
    let out = cast(&mut game, &character, 1, ENEMY_GUID, 0);
    assert!(!quest_packets(&out), "another ability credits nothing");
    assert_eq!(counts(&game, QUEST), [0]);
    assert_eq!(
        saved(&game),
        before,
        "the accepted other-ability cast kept the quest state"
    );
}

#[test]
fn wrong_target_kind_changes_nothing() {
    let fixture = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, Some(POND_BLASTER)),
        &[enemy(OTHER_ENEMY, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = archer(&fixture, A, at);
    let before = saved(&game);
    let out = cast(&mut game, &character, 2, ENEMY_GUID, 0);
    assert!(
        !quest_packets(&out),
        "a different enemy kind credits nothing"
    );
    assert_eq!(counts(&game, QUEST), [0]);
    assert_eq!(
        saved(&game),
        before,
        "the accepted wrong-target cast kept the quest state"
    );
}

#[test]
fn untargeted_goal_matches_any_target() {
    let fixture = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, None),
        &[enemy(OTHER_ENEMY, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = archer(&fixture, A, at);
    let out = cast(&mut game, &character, 2, ENEMY_GUID, 0);
    assert_eq!(
        counts(&game, QUEST),
        [1],
        "a goal without a target matches any enemy"
    );
    assert!(objective_update(&out, QUEST, OBJECTIVE, 1));
}

#[test]
fn an_accepted_item_use_credits_the_objective() {
    let fixture = use_fixture(
        item_goal(POTION, 1),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let (mut game, instance) = item_player(&fixture, A, at, POTION, 1);
    assign(&mut game, &character, instance, 0);

    // 42/16 opens the confirm window; the item is still owned and nothing is credited.
    let check = send(
        &mut game,
        &character,
        &encode_immediate_activation(ImmediateActivation::Check, instance),
        1,
    );
    assert!(
        check
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 30, 0]))
    );
    assert_eq!(
        counts(&game, QUEST),
        [0],
        "the check spends and credits nothing"
    );
    assert!(owned(&game, instance));

    // 42/15 is the confirmation: it spends exactly one and credits the goal.
    let used = send(
        &mut game,
        &character,
        &encode_immediate_activation(ImmediateActivation::Use, instance),
        2,
    );
    assert_eq!(counts(&game, QUEST), [1]);
    assert!(objective_complete(&used, QUEST, OBJECTIVE));
    assert!(!owned(&game, instance), "the single-use item was spent");
}

/// The 36/10 bar-2 activation is the other accepted use path: it spends through the same
/// `ItemBar::consume` and must credit the goal and decrement the stack.
#[test]
fn an_activated_item_credits_the_objective_and_spends_the_stack() {
    let fixture = use_fixture(item_goal(POTION, 1), &[]);
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let (mut game, instance) = item_player(&fixture, A, at, POTION, 1);
    assign(&mut game, &character, instance, 0);

    // 36/10 bar 2, slot 0, self target (`00a342e0`, sender `00a35150`).
    let used = activate_item(&mut game, &character, A, 1);
    assert_eq!(counts(&game, QUEST), [1]);
    assert!(objective_complete(&used, QUEST, OBJECTIVE));
    assert_eq!(stack(&game, instance), 0, "the single-use item was spent");
}

/// T789 follow-up: a refused 42/15 must keep the confirmation it was sent with. One potion is
/// checked, the second bar slot (same stack) is activated and stamps the shared recast, so the
/// checked confirm is refused by that recast; the same confirmation must then succeed after it.
#[test]
fn a_refused_confirm_keeps_the_pending_check() {
    let fixture = use_fixture(item_goal(POTION, 2), &[]);
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let (mut game, instance) = item_player(&fixture, A, at, POTION, 2);
    assign_slot(&mut game, &character, 0, instance, 0);
    assign_slot(&mut game, &character, 1, instance, 0);

    // 42/16 stages the confirmation; nothing is spent.
    let check = send(
        &mut game,
        &character,
        &encode_immediate_activation(ImmediateActivation::Check, instance),
        0,
    );
    assert!(
        check
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 30, 0]))
    );
    assert_eq!(stack(&game, instance), 2);

    // 36/10 bar 2, slot 1: accepted, spends one and stamps the 10 s recast.
    activate_item_slot(&mut game, &character, 1, A, 1);
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(stack(&game, instance), 1);

    // 42/15 while the recast is live: refused, but it must keep the pending confirmation.
    let before = saved(&game);
    let refused = send(
        &mut game,
        &character,
        &encode_immediate_activation(ImmediateActivation::Use, instance),
        2,
    );
    assert_refused_nothing(&game, &refused, &before, "confirm during recast");
    assert_eq!(
        stack(&game, instance),
        1,
        "the refused confirm spends nothing"
    );
    assert_eq!(counts(&game, QUEST), [1]);

    // After the recast the same confirmation still works: the refusal did not clear it.
    let used = send(
        &mut game,
        &character,
        &encode_immediate_activation(ImmediateActivation::Use, instance),
        11_000,
    );
    assert_eq!(counts(&game, QUEST), [2]);
    assert_eq!(
        stack(&game, instance),
        0,
        "the preserved confirmation spends the last potion"
    );
    assert!(objective_complete(&used, QUEST, OBJECTIVE));
}

#[test]
fn refused_item_use_credits_nothing() {
    let fixture = use_fixture(
        item_goal(POTION, 2),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    // A two-item stack keeps the slot bound after the first use, so the recast refusal is the
    // item bar's own check rather than an empty slot.
    let (mut game, instance) = item_player(&fixture, A, at, POTION, 2);
    assign(&mut game, &character, instance, 0);

    // 42/15 with no preceding 42/16 check: refused, nothing spent, nothing credited.
    let before = saved(&game);
    let unconfirmed = send(
        &mut game,
        &character,
        &encode_immediate_activation(ImmediateActivation::Use, instance),
        1,
    );
    assert_refused_nothing(&game, &unconfirmed, &before, "unconfirmed use");
    assert_eq!(stack(&game, instance), 2);

    // A cancelled confirmation: the check only stages the session-only pending id. The item and
    // the quest state stay byte-equal.
    let check = send(
        &mut game,
        &character,
        &encode_immediate_activation(ImmediateActivation::Check, instance),
        2,
    );
    assert!(
        check
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 30, 0]))
    );
    assert!(!quest_packets(&check));
    assert_eq!(
        saved(&game),
        before,
        "a cancelled confirmation changed the save"
    );
    assert_eq!(counts(&game, QUEST), [0]);

    // A forged target guid is still refused by the self-target guard: no credit, no spend. It
    // arrives before the confirmation so the pending check stays usable.
    let forged = activate_item(&mut game, &character, A + 99, 3);
    assert!(
        forged
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[36, 0, 1, 0])),
        "a forged target gets the 36/1 failure"
    );
    assert_refused_nothing(&game, &forged, &before, "forged item target");
    assert_eq!(stack(&game, instance), 2);

    // The confirmed use spends exactly one item and credits one step.
    let used = send(
        &mut game,
        &character,
        &encode_immediate_activation(ImmediateActivation::Use, instance),
        4,
    );
    assert_eq!(counts(&game, QUEST), [1]);
    assert!(objective_update(&used, QUEST, OBJECTIVE, 1));
    assert_eq!(stack(&game, instance), 1, "one-use decrement");

    // The immediate repeat is refused by the recast: nothing spent, nothing credited.
    let before = saved(&game);
    let recast = activate_item(&mut game, &character, 0, 5);
    assert_refused_nothing(&game, &recast, &before, "item recast");
    assert_eq!(stack(&game, instance), 1);
    assert_eq!(counts(&game, QUEST), [1]);
}

#[test]
fn forged_target_is_still_refused() {
    let fixture = use_fixture(
        item_goal(POTION, 1),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let (mut game, instance) = item_player(&fixture, A, at, POTION, 1);
    assign(&mut game, &character, instance, 0);
    let before = saved(&game);
    let forged = activate_item(&mut game, &character, A + 1, 1);
    assert_refused_nothing(&game, &forged, &before, "forged item target");
    assert!(owned(&game, instance), "a forged target spends nothing");
    assert_eq!(counts(&game, QUEST), [0]);
}

#[test]
fn boombox_press_credits_nothing() {
    let fixture = use_fixture(
        item_goal(BOOMBOX, 1),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let (mut game, instance) = item_player(&fixture, A, at, BOOMBOX, 1);
    assign(&mut game, &character, instance, 0);
    let before = saved(&game);
    let pressed = activate_item(&mut game, &character, 0, 1);
    assert_eq!(pressed.boombox, Some(BOOMBOX), "the press places the box");
    assert!(!pressed.changed, "a boombox press changes no saved state");
    assert_eq!(
        saved(&game),
        before,
        "a boombox press must not credit a use_item objective"
    );
    assert!(!quest_packets(&pressed));
    assert_eq!(counts(&game, QUEST), [0]);
    assert!(owned(&game, instance), "a press never spends the box");
}

#[test]
fn use_after_completion_changes_nothing() {
    let fixture = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, Some(POND_BLASTER)),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = archer(&fixture, A, at);
    cast(&mut game, &character, 2, ENEMY_GUID, 0);
    assert_eq!(counts(&game, QUEST), [1]);
    let before = saved(&game);
    let after = cast(&mut game, &character, 2, ENEMY_GUID, 5_000);
    assert!(
        !quest_packets(&after),
        "a completed objective must not report progress again"
    );
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(
        saved(&game),
        before,
        "a use after completion changed the save"
    );
}

#[test]
fn reload_between_use_and_turnin_keeps_progress() {
    let fixture = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 2, Some(POND_BLASTER)),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let at = [0.0, 0.0, 0.0];
    let character = character_at_for(A, 0, at);
    let mut game = archer(&fixture, A, at);
    cast(&mut game, &character, 2, ENEMY_GUID, 0);
    assert_eq!(counts(&game, QUEST), [1]);

    // Disconnect and reconnect through the production save envelope (`game_store`), not a
    // `GameplayState` proxy: the counter is part of the quest block the real login restores.
    let mut game = fixture.save_and_reload(&game, &character);
    game.initialize_world(&character, 0).unwrap();
    assert_eq!(
        counts(&game, QUEST),
        [1],
        "the counter survives the reconnect"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    cast(&mut game, &character, 2, ENEMY_GUID, 5_000);
    assert_eq!(
        counts(&game, QUEST),
        [2],
        "the reconnected session credits the second use"
    );
    let mut game = fixture.save_and_reload(&game, &character);
    game.initialize_world(&character, 0).unwrap();
    assert_eq!(counts(&game, QUEST), [2]);
    assert_eq!(
        quest_status(&game, QUEST),
        QuestStatus::Active,
        "a completed non-talk goal waits at its turn-in NPC"
    );
}

#[test]
fn only_the_caster_is_credited() {
    let fixture = use_fixture(
        ability_goal(EXPLOSIVE_ARROW, 1, Some(POND_BLASTER)),
        &[enemy(POND_BLASTER, [1.0, 0.0, 0.0])],
    );
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at_for(A, 0, a_at);
    let b_character = character_at_for(B, 0, b_at);
    let mut a = archer(&fixture, A, a_at);
    let mut b = archer(&fixture, B, b_at);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([ENEMY_GUID]);

    let out = a
        .handle_shared_field(
            &SharedFieldAction::Ability {
                slot: 2,
                target: ENEMY_GUID,
            },
            &mut field,
            &known,
            &a_character,
            0,
        )
        .unwrap();
    assert_eq!(counts(&a, QUEST), [1], "the caster is credited");
    assert!(objective_update(&out, QUEST, OBJECTIVE, 1));
    assert_eq!(counts(&b, QUEST), [0], "the other player is not credited");
    let before = saved(&b);
    b.tick_shared_effects(&mut field, &b_character, 100)
        .unwrap();
    assert_eq!(
        saved(&b),
        before,
        "the shared tick left the other player unchanged"
    );
}

/// T789 follow-up: 3010011's use step casts an Archer-only ability (Explosive Arrow 62019), so
/// the record must gate its offer to that job and level. Without the gate another job could
/// accept it and reach a step its toolbar can never resolve; the shipped content carries the
/// guide's Band 2 (Archer level 5 to 10) as `job: 35, min_level: 5`.
#[test]
fn the_pondblaster_quest_requires_archer_level_five() {
    const SHIPPED_QUEST: u32 = 3_010_011;
    const GIVER_NPC: u64 = 9_000_000_106;

    let fixture = Fixture::for_character(A);
    let catalogue = fixture.quest_catalogue();
    let path = catalogue.join("quests/archer.json");
    let mut document = read_json(&path);
    let record = document["records"]
        .as_array_mut()
        .expect("archer records")
        .iter_mut()
        .find(|record| record["id"].as_u64() == Some(u64::from(SHIPPED_QUEST)))
        .expect("the shipped Archer record exists");
    // The offer gate fields are asserted as shipped; this private copy only drops the
    // prerequisite so the gate is the one thing under test.
    assert_eq!(
        record["job"].as_u64(),
        Some(35),
        "the offer gate is the Archer job"
    );
    assert_eq!(record["min_level"].as_u64(), Some(5), "Archer level 5");
    record["enabled"] = json!(true);
    record["prerequisite_quest_ids"] = json!([]);
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();

    let offered = |job: &str, level: u8| {
        let mut eligibility = eligibility();
        eligibility.job_levels = std::collections::BTreeMap::from([(job.to_owned(), level)]);
        let mut engine = QuestEngine::from_split_content(&catalogue, A, eligibility, None)
            .expect("the altered catalogue loads");
        engine.offer(SHIPPED_QUEST, GIVER_NPC).changed
    };
    assert!(offered("archer", 5), "an Archer at level 5 is offered it");
    assert!(
        !offered("archer", 4),
        "an Archer below level 5 is not offered the level-5 step"
    );
    assert!(
        !offered("brawler", 20),
        "another job cannot accept a step its bar can never cast"
    );
}

#[test]
fn the_shipped_pondblaster_quest_completes_end_to_end() {
    /// The authored record this slice fills in: Archer "Pondblaster Payback".
    const SHIPPED_QUEST: u32 = 3_010_011;
    /// The record's shipped giver in `npcs/greenwood_forest.json`.
    const GIVER_NPC: u64 = 9_000_000_106;
    /// The record's end text (locale 383172).
    const END_TEXT: u32 = 383_172;
    const ATTACK_CHOICE: i32 = 103;

    let document = read_json(&content_dir().join("field_enemies.json"));
    let field_row = document["enemies"]
        .as_array()
        .expect("shipped field enemies")
        .iter()
        .find(|row| row["definition_id"].as_u64() == Some(u64::from(POND_BLASTER)))
        .expect("the shipped field content carries the Pondblaster")
        .clone();
    let enemy_guid = field_row["guid"].as_u64().unwrap();
    let enemy_at: [f32; 3] = serde_json::from_value(field_row["position"].clone()).unwrap();
    let fixture = Fixture::for_character(A).with_field_enemies(&[field_row]);
    fixture.copy_content("job_abilities.json");
    // T895: the shipped record is enabled behind Archer 5 (3010009), so this walks the shipped
    // data unchanged: the prerequisite is done, nothing is patched.
    let record = shipped_quest(SHIPPED_QUEST);
    assert!(record.enabled, "the shipped record is enabled");
    assert_eq!(record.prerequisite_quest_ids, [ARCHER_5]);
    let (coins, profile_xp) = record.reward_totals(ARCHER);
    assert!(coins > 0 && profile_xp > 0, "the reward is authored");

    let mut state = fixture.game_as(A, None).state();
    seed_quest(&mut state, ARCHER_5, QuestStatus::Completed, &[1, 1]);
    {
        let profiles = state.profiles.as_mut().expect("the fixture has profiles");
        profiles.active = ARCHER;
        profiles.unlocked.insert(ARCHER);
        profiles.experience.insert(ARCHER, 400);
    }
    equip_archer_bow(&mut state);
    let mut game = fixture.game_as(A, Some(state));
    let at_giver = character_at_for(A, 0, content_position("greenwood_forest.json", GIVER_NPC));
    game.initialize_world(&at_giver, 0).unwrap();

    let mut now = 10_000;
    offer(&mut game, &at_giver, GIVER_NPC, SHIPPED_QUEST, now);
    accept(&mut game, &at_giver, SHIPPED_QUEST, now + 2);
    now += 20;
    // The quest-gated field actor spawns on the session tick once the quest is active.
    let at_enemy = character_at_for(A, 0, [enemy_at[0], enemy_at[1], enemy_at[2] + 5.0]);
    game.tick(&at_enemy, now).unwrap();
    now += 10;

    // Explosive Arrow on the Pondblaster credits the use_ability step; the same cast usually
    // kills it, which credits the defeat step. Finish the kill with basic attacks if not.
    let out = cast(&mut game, &at_enemy, 2, enemy_guid, now);
    assert!(
        objective_update(&out, SHIPPED_QUEST, 2, 1),
        "the authored use_ability step reports 1/1"
    );
    assert_eq!(counts(&game, SHIPPED_QUEST)[1], 1);
    now += 700;
    for _ in 0..8 {
        if counts(&game, SHIPPED_QUEST)[0] > 0 {
            break;
        }
        send(&mut game, &at_enemy, &npc_interact(enemy_guid), now);
        send(
            &mut game,
            &at_enemy,
            &npc_select(enemy_guid, ATTACK_CHOICE),
            now + 1,
        );
        now += 700;
    }
    assert_eq!(
        counts(&game, SHIPPED_QUEST),
        [1, 1],
        "both authored steps complete"
    );

    // The real giver opens the end screen; 49/14 completes the record and pays its reward once.
    let talked = talk(&mut game, &at_giver, GIVER_NPC, now);
    assert!(talked.changed, "the final Talk opens the turn-in");
    assert_eq!(pending_turnin(&game), Some(SHIPPED_QUEST));
    let end = release_end_screen(
        &mut game,
        &at_giver,
        SHIPPED_QUEST,
        GIVER_NPC,
        END_TEXT,
        now + 2,
    );
    assert!(end.starts_with(&[49, 0, 13, 0, 0, 0]));
    now += 4;
    finish_quest(
        &mut game,
        &at_giver,
        SHIPPED_QUEST,
        ARCHER,
        coins,
        profile_xp,
        now,
    );
    assert_eq!(quest_status(&game, SHIPPED_QUEST), QuestStatus::Completed);
}

/// T895: Archer Departure (3010012) follows Pondblaster Payback (3010011), the guide's order, so
/// Hilforn offers exactly one of the two at a time and the Departure accept is refused until the
/// Pondblasters are dealt with. Driven through the real offer and accept packets.
#[test]
fn archer_departure_waits_for_pondblaster_payback() {
    const PONDBLASTER: u32 = 3_010_011;
    const DEPARTURE: u32 = 3_010_012;
    const GIVER_NPC: u64 = 9_000_000_106;
    assert_eq!(
        shipped_quest(DEPARTURE).prerequisite_quest_ids,
        [PONDBLASTER]
    );

    // An Archer at rank 5 who finished Archer 5; `done` adds quests finished on top.
    let hilforn = |done: &[u32]| {
        let fixture = Fixture::for_character(A);
        let mut state = fixture.game_as(A, None).state();
        {
            let profiles = state.profiles.as_mut().expect("the fixture has profiles");
            profiles.active = ARCHER;
            profiles.unlocked.insert(ARCHER);
            profiles.experience.insert(ARCHER, 400);
        }
        for quest in std::iter::once(&ARCHER_5).chain(done) {
            seed_quest(&mut state, *quest, QuestStatus::Completed, &[1, 1]);
        }
        let mut game = fixture.game_as(A, Some(state));
        let at = character_at_for(A, 0, content_position("greenwood_forest.json", GIVER_NPC));
        game.initialize_world(&at, 0).unwrap();
        (game, at)
    };

    let (mut game, at) = hilforn(&[]);
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_reply(DEPARTURE, true),
        100,
        "accepting Departure before Pondblaster Payback",
    );
    offer(&mut game, &at, GIVER_NPC, PONDBLASTER, 1_000);

    let (mut game, at) = hilforn(&[PONDBLASTER]);
    offer(&mut game, &at, GIVER_NPC, DEPARTURE, 1_000);
}
