//! S14 reload pins: a developer field reload is one staged swap, so a refused
//! read returns no packets and leaves every field structure exactly as it was.
//! A successful battle reload still replaces the roster and keeps the spawner
//! table; an omitted row is dropped. These drive the production engine entry
//! points the `realm reload` path calls.
use osfr_tools::{character_store::CharacterState, encounter::EncounterEngine};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

const KEPT_BATTLE: u32 = 12310;
const DROPPED_BATTLE: u32 = 96;

struct Fixture {
    dir: PathBuf,
    battle_file: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

impl Fixture {
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "w359-field-reload-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = json!({
            "zone_id": 0,
            "name": "SyntheticWorld",
            "sky": "",
            "geometry_id": 214,
            "position": [0, 0, 0],
            "rotation": [0, 0, 0],
            "tutorial": false
        });
        let mut cave = zone.clone();
        cave["zone_id"] = json!(411);
        fs::write(
            dir.join("encounters.json"),
            json!({
                "schema_version": 1,
                "template_file": "npc.bin",
                "template_sha256": format!("{:x}", Sha256::digest(&template)),
                "world": zone,
                "world_enemies": [],
                "dungeons": [{
                    "id": 411,
                    "title_id": 12,
                    "description_id": 13,
                    "objective_id": 14,
                    "zone": cave,
                    "guide": null,
                    "enemies": [{
                        "guid": 200,
                        "definition_id": 400,
                        "model_id": 3,
                        "name_id": 100,
                        "name": "Synthetic actor",
                        "texture_alias": "",
                        "position": [0, 0, 0],
                        "max_health": 100,
                        "damage": 1,
                        "boss": false
                    }],
                    "reward_coins": 0,
                    "reward_items": []
                }]
            })
            .to_string(),
        )
        .unwrap();
        Self {
            dir,
            battle_file: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("data/content/field_battles.json"),
        }
    }

    fn engine(&self) -> EncounterEngine {
        let mut engine = EncounterEngine::load(&self.dir, None).unwrap();
        engine.load_field_battles(&self.battle_file).unwrap();
        engine
    }

    fn write(&self, name: &str, value: &Value) -> PathBuf {
        let path = self.dir.join(name);
        fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
        path
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn battle<'a>(file: &'a Value, id: u32) -> &'a Value {
    file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(id)))
        .unwrap()
}

fn spawner_guid(file: &Value, id: u32) -> u64 {
    battle(file, id)["spawner"]["guid"].as_u64().unwrap()
}

fn spawner_home(file: &Value, id: u32) -> [f32; 3] {
    let points = battle(file, id)["spawner"]["position"].as_array().unwrap();
    [
        points[0].as_f64().unwrap() as f32,
        points[1].as_f64().unwrap() as f32,
        points[2].as_f64().unwrap() as f32,
    ]
}

fn location() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [9.0, 2.0, 3.0],
        rotation: [0.1, 0.2, 0.3],
    }
}

/// Walk one battle marker to its first waypoint, where it pauses for 3 s, so a
/// cleared `wander` map would visibly restart it from home.
fn walk_marker_to_pause(engine: &mut EncounterEngine, spawner: u64, observer: [f32; 3]) -> u64 {
    let mut now = 100;
    for _ in 0..40 {
        let out = engine.tick(observer, 43, now).unwrap();
        if out.replies.iter().any(|packet| {
            packet.starts_with(&[35, 0, 8, 0])
                && packet[4..12] == spawner.to_le_bytes()
                && packet[12..16] == 1_i32.to_le_bytes()
        }) {
            return now;
        }
        now += 250;
    }
    panic!("marker {spawner} did not reach its first waypoint");
}

#[test]
fn a_malformed_reload_leaves_every_field_structure_unchanged() {
    let fixture = Fixture::new();
    let shipped = read_json(&fixture.battle_file);
    let spawner = spawner_guid(&shipped, KEPT_BATTLE);
    let home = spawner_home(&shipped, KEPT_BATTLE);
    let mut engine = fixture.engine();
    let spawners = engine.battle_spawners();
    // The open start screen and the marker's paused walk are part of the state a
    // refusal must keep, alongside battles, dungeons and resolved behaviours.
    engine.reopen_battle_screen(KEPT_BATTLE, 43);
    let observer = [home[0] + 20.0, home[1], home[2]];
    let paused_at = walk_marker_to_pause(&mut engine, spawner, observer);
    let state = engine.state().clone();
    let cave_behaviour = engine.behaviour(200).cloned();
    assert!(cave_behaviour.is_some());

    // A bad preset fails `load_field_battles` after it has already appended the
    // new dungeon and battle row, so the refusal has to roll both back.
    let mut file = shipped.clone();
    let mut row = battle(&shipped, KEPT_BATTLE).clone();
    row["id"] = json!(4242);
    row["spawner"]["guid"] = json!(910_424_001_u64);
    for (index, enemy) in row["enemies"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .enumerate()
    {
        enemy["guid"] = json!(910_424_100_u64 + index as u64);
    }
    row["enemies"][0]["behaviour"] = json!("missing_preset");
    file["battles"].as_array_mut().unwrap().push(row);
    let path = fixture.write("malformed-battles.json", &file);

    let error = engine.reload_field_battles(&path).unwrap_err().to_string();
    assert!(error.contains("unknown behaviour preset"), "{error}");

    assert_eq!(
        engine.battle_spawners(),
        spawners,
        "a refused battle reload replaces no row"
    );
    assert_eq!(
        engine.state(),
        &state,
        "a refused battle reload changes no encounter state"
    );
    assert_eq!(
        engine.behaviour(200).cloned(),
        cave_behaviour,
        "a refused battle reload leaves the resolved behaviours alone"
    );
    assert!(
        engine.behaviour(910_424_100).is_none(),
        "the refused row's actors never enter the behaviour table"
    );
    assert!(
        engine
            .begin(4242, location(), 43)
            .unwrap_err()
            .to_string()
            .contains("unknown encounter"),
        "the refused row's dungeon never enters the content"
    );

    // The marker is still paused: a cleared wander would recreate it at home
    // and walk immediately (125), while the surviving walk stays silent.
    let out = engine.tick(observer, 43, paused_at + 100).unwrap();
    assert!(
        !out.replies.iter().any(|packet| {
            packet.starts_with(&[125, 0]) && packet[2..10] == spawner.to_le_bytes()
        }),
        "a refused battle reload must not clear the marker walk"
    );

    // The open start screen survived too (the walk position is within its 8 m).
    assert_eq!(engine.take_pending_battle(home), Some(KEPT_BATTLE));
}

#[test]
fn same_file_battle_reload_keeps_the_spawner_table() {
    let fixture = Fixture::new();
    let shipped = read_json(&fixture.battle_file);
    let spawner = spawner_guid(&shipped, KEPT_BATTLE);
    let home = spawner_home(&shipped, KEPT_BATTLE);
    let mut engine = fixture.engine();
    let spawners = engine.battle_spawners();
    engine.reopen_battle_screen(KEPT_BATTLE, 43);

    engine.reload_field_battles(&fixture.battle_file).unwrap();

    assert_eq!(
        engine.battle_spawners(),
        spawners,
        "reloading the same file keeps the spawner table"
    );
    assert!(engine.is_battle_spawner(spawner));
    assert_eq!(
        engine.take_pending_battle(home),
        None,
        "a successful reload consumes the open start screen"
    );
    assert!(!engine.in_battle_arena());
}

#[test]
fn a_battle_omitted_from_the_reload_file_is_dropped() {
    let fixture = Fixture::new();
    let shipped = read_json(&fixture.battle_file);
    let kept_spawner = spawner_guid(&shipped, KEPT_BATTLE);
    let dropped_spawner = spawner_guid(&shipped, DROPPED_BATTLE);
    let mut engine = fixture.engine();
    assert!(engine.is_battle_spawner(dropped_spawner));

    let kept = battle(&shipped, KEPT_BATTLE).clone();
    let path = fixture.write(
        "one-battle.json",
        &json!({"schema_version": 1, "battles": [kept]}),
    );
    engine.reload_field_battles(&path).unwrap();

    let spawners = engine.battle_spawners();
    assert_eq!(spawners.len(), 1, "the omitted battle is dropped");
    assert_eq!(spawners[0].0, kept_spawner);
    assert!(!engine.is_battle_spawner(dropped_spawner));
    assert!(
        engine
            .begin(DROPPED_BATTLE, location(), 43)
            .unwrap_err()
            .to_string()
            .contains("unknown encounter"),
        "the omitted battle's dungeon is dropped with its row"
    );
}

#[test]
fn a_reload_while_a_fight_is_active_is_refused() {
    let fixture = Fixture::new();
    let mut engine = fixture.engine();
    engine.begin(KEPT_BATTLE, location(), 43).unwrap();
    engine.on_zone_ready(43, 100).unwrap();
    assert!(engine.active());
    let state = engine.state().clone();
    let spawners = engine.battle_spawners();

    let error = engine
        .reload_field_battles(&fixture.battle_file)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("finish or exit the encounter before reloading battles"),
        "{error}"
    );
    let enemies = fixture.write(
        "field-enemies.json",
        &json!({"schema_version": 1, "enemies": []}),
    );
    let error = engine
        .reload_field_enemies(&enemies)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("finish or exit the encounter before reloading enemies"),
        "{error}"
    );

    assert_eq!(
        engine.state(),
        &state,
        "a mid-fight reload refusal changes no encounter state"
    );
    assert_eq!(
        engine.battle_spawners(),
        spawners,
        "a mid-fight reload refusal keeps every battle row"
    );
}
