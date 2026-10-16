//! Opening cave replay: after the campaign, the ordinary interaction with the
//! guide who owned the dungeon objective starts the bound dungeon again.
//!
//! Fixture note: the post-campaign save is built through the public
//! `GameplayState` + `Gameplay::load` API, the same documented pattern as
//! `tests/economy_review.rs` and `tests/cooking_gameplay.rs`. The fixture only
//! marks the story chain completed/rewarded and the dungeon rewarded; the
//! guide/encounter binding is derived by production from the opening content,
//! so an existing completed save needs no migration record. There is no
//! test-only production hook.

use osfr_tools::{
    character_store::CharacterState, encounter::EncounterPhase, gameplay::Gameplay,
    gateway::Gateway,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

const NOW: u64 = 10_000;

fn paths() -> (PathBuf, PathBuf) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references");
    (
        std::env::var_os("OSFR_GAMEPLAY_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join("opening-journal-text-v2/content")),
        std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join("rust-gateway-reference")),
    )
}

fn content_json(content: &Path, file: &str) -> Value {
    serde_json::from_slice(&fs::read(content.join(file)).unwrap()).unwrap()
}

fn position(value: &Value) -> [f32; 3] {
    let values = value.as_array().unwrap();
    [
        values[0].as_f64().unwrap() as f32,
        values[1].as_f64().unwrap() as f32,
        values[2].as_f64().unwrap() as f32,
    ]
}

/// Everything the replay needs, derived from the opening/encounter content.
struct Binding {
    giver_guid: u64,
    giver_position: [f32; 3],
    giver_zone: u32,
    encounter_id: u32,
    enemy_max_health: Vec<u32>,
    dungeon_zone: u32,
    dungeon_position: [f32; 3],
}

fn binding(content: &Path) -> Binding {
    let opening = content_json(content, "opening.json");
    let encounters = content_json(content, "encounters.json");
    let mut found = None;
    for stage in opening["stages"].as_array().unwrap() {
        for goal in stage["objectives"].as_array().unwrap() {
            if goal["trigger"]["kind"] == "dungeon_completed" {
                found = Some((
                    stage["id"].as_u64().unwrap() as u32,
                    stage["giver_guid"].as_u64().unwrap(),
                    goal["trigger"]["encounter_id"].as_u64().unwrap() as u32,
                ));
            }
        }
    }
    let (_stage_id, giver_guid, encounter_id) =
        found.expect("opening content binds a dungeon objective to a giver");
    let giver = opening["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|actor| actor["guid"].as_u64() == Some(giver_guid))
        .expect("the bound giver exists in the content")
        .clone();
    let dungeon = encounters["dungeons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|dungeon| dungeon["id"].as_u64() == Some(u64::from(encounter_id)))
        .expect("the bound encounter exists in the content")
        .clone();
    let enemies = dungeon["enemies"].as_array().unwrap();
    Binding {
        giver_guid,
        giver_position: position(&giver["position"]),
        giver_zone: giver["zone_id"].as_u64().unwrap() as u32,
        encounter_id,
        enemy_max_health: enemies
            .iter()
            .map(|enemy| enemy["max_health"].as_u64().unwrap() as u32)
            .collect(),
        dungeon_zone: dungeon["zone"]["zone_id"].as_u64().unwrap() as u32,
        dungeon_position: position(&dungeon["zone"]["position"]),
    }
}

struct Fixture {
    gameplay: Gameplay,
    character: CharacterState,
    binding: Binding,
}

/// `complete_chain`: every story stage completed+rewarded (false leaves the
/// final stage pending). `rewarded_proof`: the dungeon already rewarded once.
fn fixture(complete_chain: bool, rewarded_proof: bool) -> Fixture {
    let (content, reference) = paths();
    let binding = binding(&content);
    let base = Gameplay::load(&content, None).expect("opening content loads");
    let mut state = base.state();
    let opening = content_json(&content, "opening.json");
    let stages = opening["stages"].as_array().unwrap();
    let last_stage = stages.last().unwrap()["id"].as_u64().unwrap() as u32;
    for stage in stages {
        let id = stage["id"].as_u64().unwrap() as u32;
        if !complete_chain && id == last_stage {
            continue;
        }
        let progress = state.opening.as_mut().unwrap().stages.get_mut(&id).unwrap();
        progress.accepted = true;
        progress.completed = true;
        progress.rewarded = true;
        progress.counts = stage["objectives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|goal| goal["required"].as_u64().unwrap() as u32)
            .collect();
        for unlocked in stage["unlock_profiles"].as_array().unwrap() {
            state
                .profiles
                .as_mut()
                .unwrap()
                .unlocked
                .insert(unlocked.as_i64().unwrap() as i32);
        }
    }
    if rewarded_proof {
        state
            .encounter
            .as_mut()
            .unwrap()
            .rewarded
            .insert(binding.encounter_id);
    }
    let mut character = Gateway::reference_state(&reference).expect("reference content loads");
    character.zone_id = binding.giver_zone;
    character.position = binding.giver_position;
    let gameplay =
        Gameplay::load(&content, Some(state)).expect("the crafted post-campaign save is valid");
    Fixture {
        gameplay,
        character,
        binding,
    }
}

fn interact(fixture: &mut Fixture, guid: u64, choice: i32) -> osfr_tools::gameplay::Outcome {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(guid.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    fixture
        .gameplay
        .handle_in_world(&packet, &fixture.character, NOW)
        .expect("the interaction frame is accepted")
}

fn active(fixture: &Fixture) -> Option<osfr_tools::encounter::ActiveEncounter> {
    fixture
        .gameplay
        .state()
        .encounter
        .and_then(|encounter| encounter.active)
}

fn enter_dungeon(fixture: &mut Fixture) {
    let mut character = fixture.character.clone();
    character.zone_id = fixture.binding.dungeon_zone;
    character.position = fixture.binding.dungeon_position;
    fixture.character = character.clone();
    fixture
        .gameplay
        .initialize_world(&character, NOW + 1)
        .expect("the cave zone initializes");
}

/// Kill every bound enemy through the ordinary action-bar packet, moving the
/// character onto each spawn as the client would.
fn clear_cave(fixture: &mut Fixture) {
    let enemies: Vec<(u64, [f32; 3])> = fixture
        .gameplay
        .state()
        .encounter
        .unwrap()
        .active
        .unwrap()
        .enemies
        .into_iter()
        .map(|enemy| (enemy.guid, enemy.position))
        .collect();
    let mut now = NOW + 10;
    for (guid, position) in enemies {
        fixture.character.position = position;
        let mut guard = 0;
        loop {
            let health = fixture
                .gameplay
                .state()
                .encounter
                .unwrap()
                .active
                .unwrap()
                .enemies
                .iter()
                .find(|enemy| enemy.guid == guid)
                .unwrap()
                .health;
            if health == 0 {
                break;
            }
            let mut packet = vec![36, 0, 10, 0];
            packet.extend(1i32.to_le_bytes());
            packet.extend(0i32.to_le_bytes());
            packet.extend(0i32.to_le_bytes());
            packet.extend(guid.to_le_bytes());
            fixture
                .gameplay
                .handle_in_world(&packet, &fixture.character, now)
                .expect("the attack frame is accepted");
            now += 700;
            guard += 1;
            assert!(guard < 64, "enemy {guid} did not die");
        }
    }
}

fn close_results(fixture: &mut Fixture) {
    fixture
        .gameplay
        .handle_in_world(&[26, 0, 42, 0], &fixture.character, NOW + 5000)
        .expect("the results close is accepted");
}

fn chat(text: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0, 0, 0];
    packet.extend(1u64.to_le_bytes());
    packet.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        packet.extend([0; 12]);
        packet.extend(0u32.to_le_bytes());
        packet.extend(0u32.to_le_bytes());
    }
    packet.extend((text.len() as u32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.extend([0; 28]);
    packet
}

/// The ordinary leave, available in every phase (`realm exit`).
fn exit_cave(fixture: &mut Fixture) {
    fixture
        .gameplay
        .handle_in_world(&chat("realm exit"), &fixture.character, NOW + 5000)
        .expect("the exit command is accepted");
}

#[test]
fn post_campaign_ordinary_guide_can_start_the_cave_again() {
    // The save carries only the ordinary post-campaign progress: the binding
    // is derived from the opening content, so no migration record is needed.
    let mut fixture = fixture(true, true);
    let before = fixture.gameplay.state();
    let giver = fixture.binding.giver_guid;
    let outcome = interact(&mut fixture, giver, 1);
    let run = active(&fixture).expect("the finished cave must start again");
    assert_eq!(run.encounter_id, fixture.binding.encounter_id);
    assert_eq!(run.enemies.len(), fixture.binding.enemy_max_health.len());
    assert!(
        run.enemies
            .iter()
            .zip(&fixture.binding.enemy_max_health)
            .all(|(enemy, max)| enemy.health == *max),
        "a replay starts a fresh instance with every enemy at full health"
    );
    assert!(outcome.zoning.is_some(), "the run zones into the cave");
    assert_eq!(
        before.inventory,
        fixture.gameplay.state().inventory,
        "entering the cave grants nothing"
    );
    assert_eq!(
        before.opening.unwrap().stages,
        fixture.gameplay.state().opening.unwrap().stages,
        "entering the cave changes no stage progress"
    );
}

#[test]
fn replay_requires_a_real_exit_before_another_run() {
    let mut fixture = fixture(true, true);
    let giver = fixture.binding.giver_guid;
    interact(&mut fixture, giver, 1);
    let first = active(&fixture).expect("first replay run");
    interact(&mut fixture, giver, 1);
    let still = active(&fixture).expect("the run stays active");
    assert_eq!(
        still.instance_id, first.instance_id,
        "a second interaction must not start another run"
    );
    exit_cave(&mut fixture);
    assert!(active(&fixture).is_none(), "the ordinary exit ends the run");
    interact(&mut fixture, giver, 1);
    let second = active(&fixture).expect("a fresh run starts after the real exit");
    assert!(
        second.instance_id > first.instance_id,
        "the fresh run gets a new instance id"
    );
    assert!(
        second
            .enemies
            .iter()
            .zip(&fixture.binding.enemy_max_health)
            .all(|(enemy, max)| enemy.health == *max),
        "the fresh entry resets every enemy"
    );
}

#[test]
fn wrong_giver_selection_or_range_never_starts_a_replay() {
    // A different guide cannot replay the cave.
    let mut game = fixture(true, true);
    let (content, _) = paths();
    let opening = content_json(&content, "opening.json");
    let other = opening["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|actor| {
            actor["kind"] == "guide" && actor["guid"].as_u64() != Some(game.binding.giver_guid)
        })
        .expect("another guide exists")
        .clone();
    game.character.position = position(&other["position"]);
    interact(&mut game, other["guid"].as_u64().unwrap(), 1);
    assert!(
        active(&game).is_none(),
        "another guide must not start the bound cave"
    );

    // Only the menu's selection 1 is the ordinary start.
    let mut game = fixture(true, true);
    let giver = game.binding.giver_guid;
    interact(&mut game, giver, 0);
    assert!(
        active(&game).is_none(),
        "a non-start menu choice must not replay"
    );

    // Out of range interactions are refused with the ordinary message.
    let mut game = fixture(true, true);
    let giver = game.binding.giver_guid;
    game.character.position = [
        game.binding.giver_position[0] + 50.0,
        game.binding.giver_position[1],
        game.binding.giver_position[2],
    ];
    let outcome = interact(&mut game, giver, 1);
    assert!(active(&game).is_none(), "range still applies to replays");
    assert!(
        outcome
            .replies
            .iter()
            .any(|packet| String::from_utf8_lossy(packet).contains("Move closer")),
        "the ordinary move-closer reply is used"
    );
}

#[test]
fn partial_chain_or_missing_reward_proof_never_replays() {
    // The final story stage is still pending: no replay.
    let mut pending = fixture(false, true);
    let giver = pending.binding.giver_guid;
    interact(&mut pending, giver, 1);
    assert!(
        active(&pending).is_none(),
        "an unfinished chain must not replay"
    );

    // The dungeon was never rewarded: no reward proof, no replay.
    let mut unproven = fixture(true, false);
    let giver = unproven.binding.giver_guid;
    interact(&mut unproven, giver, 1);
    assert!(
        active(&unproven).is_none(),
        "an unrewarded dungeon must not replay"
    );
}

#[test]
fn replay_grants_no_duplicate_economy_or_stage_progress() {
    let mut fixture = fixture(true, true);
    let before = fixture.gameplay.state();
    let giver = fixture.binding.giver_guid;
    interact(&mut fixture, giver, 1);
    enter_dungeon(&mut fixture);
    clear_cave(&mut fixture);
    let finished = fixture.gameplay.state();
    let run = finished.encounter.clone().unwrap().active.unwrap();
    assert_eq!(run.phase, EncounterPhase::Won);
    assert_eq!(
        finished.inventory, before.inventory,
        "a repeat clear must not grant coins or items again"
    );
    assert_eq!(
        finished.opening.unwrap().stages,
        before.opening.unwrap().stages,
        "a repeat clear must not change stage progress"
    );
    let encounter = finished.encounter.unwrap();
    assert_eq!(
        encounter.rewarded,
        before.encounter.unwrap().rewarded,
        "the rewarded set stays once-only"
    );
    close_results(&mut fixture);
    assert!(active(&fixture).is_none());
    assert_eq!(
        fixture.gameplay.state().inventory,
        before.inventory,
        "closing the results grants nothing"
    );
}
