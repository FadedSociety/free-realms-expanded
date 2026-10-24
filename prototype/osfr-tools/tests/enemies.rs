#[path = "common/content_copy.rs"]
mod content_copy;

use osfr_tools::enemies::{EnemyEngine, EnemyMode, ZoneInstanceKey};
use osfr_tools::{
    abilities::{
        AbilityDefinition, AbilityEffect, ActorKind, CombatState, EffectKind, EffectRecipient,
        Sourced, TargetKind,
    },
    encounter::EncounterEngine,
    inventory::Inventory,
    profiles::ProfileState,
    quest_engine::QuestEvent,
    rewards::RewardState,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const PLAYER_GUID: u64 = 34;

fn test_engine(respawn_ms: u64) -> EnemyEngine {
    let content = json!({
        "schema_version": 1,
        "sources": ["(R) synthetic test data"],
        "definitions": [{
            "id": "test_wolf",
            "kind": "Test Wolf",
            "actor_kind_id": 7,
            "definition_id": 7,
            "model_id": 9,
            "name_string_id": 11,
            "texture_alias": "",
            "level": null,
            "max_health": 40,
            "damage": 25,
            "move_speed": 3.0,
            "aggro_radius": 12.0,
            "leash_radius": 20.0,
            "attack_range": 2.5,
            "attack_cooldown_ms": 1000,
            "respawn_ms": respawn_ms,
            "boss": false,
            "reward": {
                "coins": 5,
                "xp": 7,
                "items": [],
                "source": "(R) synthetic test reward"
            },
            "source": "(R) synthetic test definition"
        }],
        "spawn_groups": [{
            "zone_id": 5,
            "stage_id": null,
            "source": "(R) synthetic test group",
            "spawns": [{
                "definition": "test_wolf",
                "guid": 17,
                "position": [0.0, 0.0, 0.0],
                "waypoints": [],
                "source": "(R) synthetic spawn"
            }]
        }]
    });
    EnemyEngine::from_json(&serde_json::to_vec(&content).unwrap()).unwrap()
}

fn player(guid: u64, position: [f32; 3], health: i32, max_health: i32) -> CombatState {
    CombatState::new(
        guid,
        ActorKind::Player,
        43,
        position,
        health,
        max_health,
        100,
        100,
    )
}

fn sourced<T>(value: T) -> Sourced<T> {
    Sourced {
        value,
        source: "(R) synthetic test ability".into(),
    }
}

fn damage_ability(amount: i32) -> AbilityDefinition {
    AbilityDefinition {
        id: sourced(77),
        name: sourced("Test strike".to_owned()),
        job_profile: sourced(43),
        slot: sourced(0),
        target_kind: sourced(TargetKind::Hostile),
        range: sourced(5.0),
        cost: sourced(1),
        cooldown_ms: sourced(100),
        cast_time_ms: sourced(0),
        effects: vec![AbilityEffect {
            kind: sourced(EffectKind::Damage),
            recipient: sourced(EffectRecipient::Target),
            amount: Some(sourced(amount)),
            effect_id: None,
            duration_ms: None,
            icon_id: None,
            name_id: None,
        }],
    }
}

struct RewardFixture {
    directory: PathBuf,
    state: RewardState,
}

impl RewardFixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-enemies-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let coins = 0_i32.to_le_bytes();
        let empty_count = 0_i32.to_le_bytes();
        let definition_bytes = [100_i32.to_le_bytes(), 8_i32.to_le_bytes()].concat();
        let content = json!({
            "schema_version": 1,
            "player_guid": 800,
            "default_state": {
                "coins": 0,
                "items": [],
                "equipment": [],
                "next_instance_id": null
            },
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [
                span("coins", 13, 0, &coins),
                span("items", 17, 0, &empty_count),
                span("equipment", 21, 1, &empty_count)
            ],
            "definitions": [{
                "id": 100,
                "slot": 3,
                "class_id": 1,
                "default_tint": 5,
                "max_stack": 10,
                "model": "synthetic.adr",
                "texture": "synthetic",
                "tint_alias": "synthetic",
                "effect": 0,
                "wield_type": 0,
                "serialized_hex": hex(&definition_bytes),
                "sha256": digest(&definition_bytes)
            }]
        });
        fs::write(
            directory.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        let inventory = Inventory::load(&directory, None).unwrap();
        let profiles = ProfileState {
            version: 1,
            active: 43,
            unlocked: BTreeSet::from([43]),
            experience: BTreeMap::from([(43, 0)]),
        };
        Self {
            directory,
            state: RewardState::new(inventory, profiles, 4),
        }
    }
}

impl Drop for RewardFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("inventory.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}

fn span(kind: &str, offset: usize, profile_id: i32, bytes: &[u8]) -> Value {
    json!({
        "kind": kind,
        "offset": offset,
        "length": bytes.len(),
        "profile_id": profile_id,
        "sha256": digest(bytes)
    })
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn from_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn nearby_player_aggroes_chases_and_attack_uses_exact_32_7_bytes() {
    let key = ZoneInstanceKey {
        zone_id: 5,
        instance_id: 0,
    };
    let mut engine = test_engine(500);
    let spawned = engine.enter_instance(key, None).unwrap();
    assert_eq!(&spawned.messages[0][..4], &[35, 0, 2, 0]);
    assert_eq!(&spawned.messages[1][..4], &[35, 0, 5, 0]);

    let mut players = [player(PLAYER_GUID, [3.0, 0.0, 0.0], 100, 100)];
    let chase = engine.tick(key, &mut players, 100).unwrap();
    assert_eq!(chase.messages.len(), 2);
    assert_eq!(&chase.messages[0][..2], &[125, 0]);
    assert_eq!(&chase.messages[1][..4], &[35, 0, 8, 0]);
    assert_eq!(
        engine.enemy(key, 17).unwrap().mode,
        EnemyMode::Chase {
            target_guid: PLAYER_GUID
        }
    );

    let attack = engine.tick(key, &mut players, 350).unwrap();
    assert_eq!(attack.messages.len(), 5);
    assert_eq!(&attack.messages[1][..4], &[35, 0, 8, 0]); // stop running
    assert_eq!(&attack.messages[2][..4], &[35, 0, 8, 0]); // attack pose
    assert_eq!(
        attack.messages[3],
        from_hex(
            "200007001100000000000000110000000000000022000000000000001900000064000000000000000000000000004b000000"
        )
    );
    assert_eq!(
        &attack.messages[4],
        &[38, 0, 1, 0, 75, 0, 0, 0, 100, 0, 0, 0]
    );
    assert_eq!(players[0].health, 75);
}

#[test]
fn ability_damage_kills_once_and_respawns_after_its_timer() {
    let key = ZoneInstanceKey {
        zone_id: 5,
        instance_id: 0,
    };
    let mut engine = test_engine(500);
    engine.enter_instance(key, None).unwrap();
    let mut caster = player(PLAYER_GUID, [1.0, 0.0, 0.0], 100, 100);
    caster.owned_abilities.insert(77);
    caster.slots.insert(0, 77);
    let ability = damage_ability(50);
    let fixture = RewardFixture::new();

    let mut resolution = engine
        .use_ability_on_enemy(key, &mut caster, 17, &ability, 100, &fixture.state)
        .unwrap();
    let kill = resolution.kill.take().unwrap();
    assert_eq!(kill.guid, 17);
    assert!(!kill.reward_packets.is_empty());
    assert_eq!(&resolution.ability.packets[0][..4], &[32, 0, 7, 0]);
    assert_eq!(&resolution.ability.packets[1][..4], &[35, 0, 5, 0]);
    assert_eq!(&kill.messages[0][..4], &[35, 0, 3, 0]);
    assert_eq!(
        kill.quest_event,
        QuestEvent::KillActorKind {
            actor_kind_id: 7,
            count: 1
        }
    );
    assert_eq!(kill.reward_state.inventory.state().coins, 5);
    assert_eq!(kill.reward_state.profiles.experience[&43], 7);
    assert_eq!(
        kill.reward_state.applied_grant_keys,
        BTreeSet::from([kill.grant_key.clone()])
    );
    assert_eq!(engine.enemy(key, 17).unwrap().mode, EnemyMode::Dead);
    assert_eq!(engine.enemy(key, 17).unwrap().respawn_at_ms, Some(600));

    let duplicate = engine
        .use_ability_on_enemy(key, &mut caster, 17, &ability, 200, &kill.reward_state)
        .unwrap();
    assert!(duplicate.kill.is_none());
    assert!(duplicate.ability.failure.is_some());

    assert!(engine.tick(key, &mut [], 500).unwrap().messages.is_empty());
    let respawn = engine.tick(key, &mut [], 600).unwrap();
    assert_eq!(respawn.messages.len(), 2);
    assert_eq!(&respawn.messages[0][..4], &[35, 0, 2, 0]);
    assert_eq!(&respawn.messages[1][..4], &[35, 0, 5, 0]);
    let alive = engine.enemy(key, 17).unwrap();
    assert_eq!(alive.health, alive.max_health);
    assert_eq!(alive.life, 1);
    assert_eq!(alive.mode, EnemyMode::Idle);
}

#[test]
fn enemy_returns_to_its_spawn_after_player_breaks_leash() {
    let key = ZoneInstanceKey {
        zone_id: 5,
        instance_id: 0,
    };
    let mut engine = test_engine(500);
    engine.enter_instance(key, None).unwrap();
    let mut players = [player(PLAYER_GUID, [8.0, 0.0, 0.0], 100, 100)];
    engine.tick(key, &mut players, 100).unwrap();
    let chased = engine.enemy(key, 17).unwrap();
    assert_eq!(
        chased.mode,
        EnemyMode::Chase {
            target_guid: PLAYER_GUID
        }
    );
    assert!(chased.position[0] > 0.0);

    players[0].position = [25.0, 0.0, 0.0];
    engine.tick(key, &mut players, 200).unwrap();
    let returned = engine.enemy(key, 17).unwrap();
    assert_eq!(returned.mode, EnemyMode::Idle);
    assert!(returned.position.iter().all(|value| value.abs() < 0.001));
}

#[test]
fn player_outside_aggro_radius_is_ignored() {
    let key = ZoneInstanceKey {
        zone_id: 5,
        instance_id: 0,
    };
    let mut engine = test_engine(500);
    engine.enter_instance(key, None).unwrap();
    let mut players = [player(PLAYER_GUID, [13.0, 0.0, 0.0], 100, 100)];
    let outcome = engine.tick(key, &mut players, 100).unwrap();
    assert!(outcome.messages.is_empty());
    assert_eq!(engine.enemy(key, 17).unwrap().mode, EnemyMode::Idle);
    assert_eq!(engine.enemy(key, 17).unwrap().position, [0.0; 3]);
}

#[test]
fn opening_wolf_messages_match_encounter_engine_byte_for_byte() {
    const WOLF: u64 = 5_715_144_128_442_925_074;
    const STAGE: u32 = 910_007;
    let project_content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let lab_content = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-journal-text-v2/content");
    assert!(lab_content.join("encounters.json").is_file());
    // This proves the message layout, and `EnemyEngine` keeps its own unscaled numbers, so
    // the encounter engine runs on the identity scale (its content directory's own
    // `combat_scale.json` wins over the bundled one).
    let scratch = std::env::temp_dir().join(format!("osfr-enemies-scale-{}", std::process::id()));
    content_copy::copy_tree(&lab_content, &scratch);
    let mut scale: Value =
        serde_json::from_slice(&fs::read(project_content.join("combat_scale.json")).unwrap())
            .unwrap();
    scale["player_base_health"] = json!(1000);
    scale["enemy_health_scale"] = json!(1.0);
    scale["enemy_damage_scale"] = json!(1.0);
    fs::write(
        scratch.join("combat_scale.json"),
        serde_json::to_vec(&scale).unwrap(),
    )
    .unwrap();

    let key = ZoneInstanceKey {
        zone_id: 0,
        instance_id: 0,
    };
    let mut enemies = EnemyEngine::load(project_content).unwrap();
    enemies.enter_instance(key, Some(STAGE)).unwrap();
    let wolf_position = enemies.enemy(key, WOLF).unwrap().position;
    let mut players = [player(
        1,
        [wolf_position[0] + 1.0, wolf_position[1], wolf_position[2]],
        1000,
        1000,
    )];
    let enemy_messages = enemies.tick(key, &mut players, 100).unwrap().messages;

    let mut encounter = EncounterEngine::load(&scratch, None).unwrap();
    encounter.set_world_stage(STAGE);
    let encounter_messages = encounter
        .tick(players[0].position, 43, 100)
        .unwrap()
        .replies;
    fs::remove_dir_all(&scratch).unwrap();
    assert_eq!(enemy_messages, encounter_messages);
}

#[test]
fn zone_instances_get_distinct_enemy_guids_and_independent_health() {
    let public = ZoneInstanceKey {
        zone_id: 5,
        instance_id: 0,
    };
    let private = ZoneInstanceKey {
        zone_id: 5,
        instance_id: 9,
    };
    let mut engine = test_engine(500);
    engine.enter_instance(public, None).unwrap();
    engine.enter_instance(private, None).unwrap();
    let public_guid = engine.enemy_guids(public)[0];
    let private_guid = engine.enemy_guids(private)[0];
    assert_ne!(public_guid, private_guid);

    let mut caster = player(PLAYER_GUID, [1.0, 0.0, 0.0], 100, 100);
    caster.owned_abilities.insert(77);
    caster.slots.insert(0, 77);
    let fixture = RewardFixture::new();
    let result = engine
        .use_ability_on_enemy(
            public,
            &mut caster,
            public_guid,
            &damage_ability(10),
            100,
            &fixture.state,
        )
        .unwrap();
    assert!(result.kill.is_none());
    assert_eq!(engine.enemy(public, public_guid).unwrap().health, 30);
    assert_eq!(engine.enemy(private, private_guid).unwrap().health, 40);
}
