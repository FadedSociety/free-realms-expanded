//! F276: a refused field action through the real session writes no character save.
//!
//! The integration suite (`tests/field_kill_rewards.rs`) can only see
//! `Gameplay`; this module drives `Session::dispatch_message` ->
//! `commit_outcome`, which is where the production save happens
//! (`src/bin/osfr-gateway.rs`: `outcome.state_changed |= result.changed`, and
//! `save` only when `state_changed`). Test-only; no production code changes.
#[path = "fishing_fixtures.rs"]
#[allow(dead_code)] // the shared fixtures module also ships builders this test does not use
mod fixtures;
use super::test_support::{self, Scratch};
use super::*;
use fixtures::copy_tree;
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
};

const ACTOR: u64 = 131000110001;
const ACTOR_DEFINITION: i32 = 930108;
const BRAWLER: i32 = 43;
const NONCOMBAT: i32 = 45;

/// 26/10 select with the field attack choice (the shared field's basic attack).
fn attack(target: u64) -> Vec<u8> {
    [
        vec![26, 0, 10, 0],
        target.to_le_bytes().to_vec(),
        103_i32.to_le_bytes().to_vec(),
    ]
    .concat()
}

struct Fixture {
    scratch: Scratch,
    content: PathBuf,
    reference: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let scratch = Scratch::new("f276-field-session");
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let reference = lab.join("references/rust-gateway-reference");
        let content = scratch.path().join("content");
        fs::create_dir_all(&content).unwrap();
        // The repo's content fills the files the runtime pack does not ship
        // (zones, NPCs, field actors); the runtime pack then wins for the files
        // both carry (startup, world and encounter content).
        copy_tree(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"),
            &content,
        );
        copy_tree(
            &lab.join("reports/round70-native-20261016T061000Z/content"),
            &content,
        );
        // The synthetic population keeps the fight short and the payout fixed.
        fs::write(
            content.join("field_enemies.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "enemies": [{
                    "guid": ACTOR,
                    "definition_id": ACTOR_DEFINITION,
                    "model_id": 3,
                    "name_id": 100,
                    "name": "Synthetic field reward",
                    "texture_alias": "",
                    "position": [0.0, 0.0, 0.0],
                    "max_health": 100,
                    "damage": 1,
                    "boss": false,
                    "level": 1,
                    "respawn_seconds": 60,
                    "aggro_range": 8.0,
                    "leash_radius": 12.0,
                    "behaviour": {"mode": "stationary", "reach": 2.5, "attack_ms": 1000},
                    "quest_ids": []
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            content.join("field_kill_rewards.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "level_rewards": [{
                    "min_level": 1,
                    "max_level": 2,
                    "coins": 2,
                    "job_xp": 5,
                    "source": [{"kind": "reconstructed", "evidence": "(R) local test balance"}]
                }],
                "definition_overrides": []
            }))
            .unwrap(),
        )
        .unwrap();
        let _ = fs::remove_file(content.join("harvesting.json"));
        Self {
            scratch,
            content,
            reference,
        }
    }

    fn character(&self) -> CharacterState {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.zone_id = 0;
        character.position = [1.0, 0.0, 0.0];
        character.rotation = [0.0, 0.0, 1.0];
        character
    }

    fn gameplay(&self, character: &CharacterState) -> GameplayState {
        let mut game = Gameplay::load(&self.content, None).unwrap();
        game.initialize_world(character, 0).unwrap();
        let mut state = game.state();
        state.inventory.coins = 0;
        let profiles = state.profiles.as_mut().unwrap();
        profiles.unlocked.extend([35, BRAWLER, NONCOMBAT]);
        profiles.active = BRAWLER;
        state
    }
}

#[test]
fn a_refused_field_replay_through_the_session_writes_no_save() {
    if !super::test_support::bin_test_target("f276") {
        return;
    }
    let fixture = Fixture::new();
    let options = Options {
        content: Some(fixture.content.clone()),
        reference: fixture.reference.clone(),
        state: fixture.scratch.state(),
        dev_tools: false,
        ..test_support::options(fixture.scratch.state())
    };
    let character = fixture.character();
    let gameplay = fixture.gameplay(&character);
    osfr_tools::game_store::save(
        &options.state,
        &osfr_tools::game_store::Snapshot {
            character: character.clone(),
            gameplay: Some(gameplay.clone()),
            pets: None,
        },
    )
    .unwrap();
    let mut seed = Gameplay::load(&fixture.content, Some(gameplay)).unwrap();
    seed.initialize_world(&character, 0).unwrap();
    let field = seed
        .take_shared_fields()
        .expect("the seed engine owns its field");
    let mut scene = test_support::Scene::with_field(&options, 41, field);
    for (index, message) in [13_u16.to_le_bytes(), 10_u16.to_le_bytes()]
        .into_iter()
        .enumerate()
    {
        scene
            .send(&options, &message, 1_000 + index as u64)
            .expect("initialization message handles");
    }
    // The shared-field view cache is server-owned; the session must know the
    // actor before its attack intent can name it (production fills this from
    // `Server::sync_field_views`).
    scene.session.known_npcs.insert(ACTOR);

    // Attack until the actor dies; each lethal commit saves the character.
    let mut dead_at = None;
    for step in 0..12_u64 {
        let now = 1_100 + step * 700;
        scene.send(&options, &attack(ACTOR), now).unwrap();
        scene.drain(now);
        if scene.membership.field(0).unwrap().health(ACTOR) == Some(0) {
            dead_at = Some(now);
            break;
        }
    }
    let dead_at = dead_at.expect("the field actor dies through the session");
    assert_eq!(
        scene.session.gateway.state().guid,
        character.guid,
        "the session owns the saved character"
    );
    let saved = fs::read(&options.state).expect("the kill saved the character");
    assert!(
        scene
            .session
            .gateway
            .gameplay()
            .is_some_and(|game| game.state().inventory.coins == 2),
        "the killer was paid through the real session"
    );

    // Hit the corpse: the production commit sees `changed == false` and must not
    // write a save (the finding 8 regression). Break only the next save the way
    // L011 does: the good save file is a component of the target path, so any
    // save attempt fails the dispatch instead of silently rewriting the file.
    let good_save = options.state.clone();
    scene.break_next_save(&good_save);
    scene
        .send(&options, &attack(ACTOR), dead_at + 6_000)
        .expect("a refused replay does not attempt a save");
    let replies = scene.drain(dead_at + 6_000);
    assert!(
        replies.is_empty(),
        "a refused replay replies with nothing: {replies:?}"
    );
    scene.repair_save(&good_save);
    assert_eq!(
        fs::read(&good_save).expect("the save is still readable"),
        saved,
        "a refused field replay wrote the character save"
    );
}
