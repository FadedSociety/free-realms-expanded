use osfr_tools::{
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterPhase},
    enemies::EnemyEngine,
    inventory::Inventory,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

const TASK_ROSTER: [u32; 5] = [96, 12301, 12302, 12310, 12311];
const NEW_ROSTER: [u32; 2] = [12310, 12311];
const STONE_HEART: u32 = 12303;
const BRAWLER: i32 = 43;
const RUNTIME_CATALOGUE: &str = "../../../lab/reports/round70-native-20261016T061000Z/content";

static SERIAL: AtomicU64 = AtomicU64::new(0);
static RUNTIME_INVENTORY: OnceLock<Inventory> = OnceLock::new();

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Fixture {
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "field-battle-roster-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();

        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();

        let actor = json!({
            "guid": 900,
            "definition_id": 400,
            "model_id": 3,
            "name_id": 100,
            "name": "Synthetic actor",
            "texture_alias": "",
            "position": [2.0, 0.0, 0.0],
            "max_health": 100,
            "damage": 10,
            "boss": false
        });
        let zone = json!({
            "zone_id": 0,
            "name": "SyntheticWorld",
            "sky": "",
            "geometry_id": 214,
            "position": [0.0, 0.0, 0.0],
            "rotation": [0.0, 0.0, 0.0],
            "tutorial": false
        });
        let mut cave = zone.clone();
        cave["zone_id"] = json!(411);
        cave["name"] = json!("SyntheticCave");
        cave["tutorial"] = json!(true);
        let content = json!({
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
                "enemies": [actor],
                "reward_coins": 0,
                "reward_items": []
            }]
        });
        fs::write(
            dir.join("encounters.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self(dir)
    }

    fn empty_engine(&self) -> EncounterEngine {
        EncounterEngine::load(&self.0, None).unwrap()
    }

    fn engine(&self) -> EncounterEngine {
        let mut engine = self.empty_engine();
        engine.load_field_battles(&battle_path()).unwrap();
        engine
    }

    fn write(&self, name: &str, value: &Value) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
        path
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn battle_path() -> PathBuf {
    manifest_dir().join("data/content/field_battles.json")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn battles() -> Value {
    read_json(&battle_path())
}

fn battle<'a>(file: &'a Value, id: u32) -> &'a Value {
    file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(id)))
        .unwrap()
}

fn i32_field(value: &Value, field: &str) -> i32 {
    value[field].as_i64().unwrap() as i32
}

fn u64_field(value: &Value, field: &str) -> u64 {
    value[field].as_u64().unwrap()
}

fn position(value: &Value) -> [f32; 3] {
    let points = value.as_array().unwrap();
    [
        points[0].as_f64().unwrap() as f32,
        points[1].as_f64().unwrap() as f32,
        points[2].as_f64().unwrap() as f32,
    ]
}

fn world_location(marker: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: marker,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn interact(guid: u64) -> Vec<u8> {
    [vec![26, 0, 8, 0], guid.to_le_bytes().to_vec(), vec![0]].concat()
}

fn i32_at(packet: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

fn runtime_inventory() -> &'static Inventory {
    RUNTIME_INVENTORY.get_or_init(|| {
        Inventory::load(&manifest_dir().join(RUNTIME_CATALOGUE), None)
            .expect("the round70 runtime item catalogue must load")
    })
}

fn checked_load(engine: &mut EncounterEngine, path: &Path) -> Result<(), String> {
    let content: Value = read_json(path);
    for battle in content["battles"].as_array().unwrap() {
        for prize in battle["prizes"].as_array().unwrap() {
            let item_id = i32_field(prize, "item_id");
            if !runtime_inventory().has_definition(item_id) {
                return Err(format!(
                    "field battle {} prize {} is absent from the runtime catalogue",
                    battle["id"], item_id
                ));
            }
        }
    }
    engine
        .load_field_battles(path)
        .map_err(|error| error.to_string())
}

#[test]
fn five_distinct_field_battles_load_with_the_existing_schema() {
    let fixture = Fixture::new();
    let engine = fixture.engine();
    let file = battles();
    let rows = file["battles"].as_array().unwrap();
    let ids: Vec<u32> = rows
        .iter()
        .map(|row| row["id"].as_u64().unwrap() as u32)
        .collect();

    // T016 already integrated Stone Heart at this autopilot tip and retains
    // its own route test. T103 still completes the designated five-battle
    // roster without deleting that adjacent work.
    assert_eq!(ids, [96, 12301, 12302, STONE_HEART, 12310, 12311]);
    assert_eq!(ids.iter().filter(|id| TASK_ROSTER.contains(id)).count(), 5);

    let mut guids = BTreeSet::new();
    for row in rows {
        for actor in
            std::iter::once(&row["spawner"]).chain(row["enemies"].as_array().unwrap().iter())
        {
            let guid = u64_field(actor, "guid");
            assert!(
                guid > 1,
                "field battle {} has a non-positive GUID",
                row["id"]
            );
            assert!(
                guids.insert(guid),
                "field battle {} repeats GUID {guid}",
                row["id"]
            );
        }
    }

    let spawners = engine.battle_spawners();
    let task_spawners: Vec<_> = rows
        .iter()
        .filter(|row| TASK_ROSTER.contains(&(row["id"].as_u64().unwrap() as u32)))
        .collect();
    assert_eq!(task_spawners.len(), 5);
    for row in &task_spawners {
        let marker = &row["spawner"];
        let guid = u64_field(marker, "guid");
        let loaded = spawners
            .iter()
            .find(|entry| entry.0 == guid)
            .expect("every designated battle publishes one spawner");
        assert_eq!(loaded.1, marker["name"].as_str().unwrap());
        assert_eq!(loaded.2, i32_field(marker, "name_id"));
        assert_eq!(loaded.3, i32_field(marker, "model_id"));
        assert_eq!(loaded.4, position(&marker["position"]));
        assert!(loaded.5 > 0, "the crossed-swords nameplate must be visible");
    }
    assert_eq!(spawners.len(), rows.len());

    for row in rows
        .iter()
        .filter(|row| TASK_ROSTER.contains(&(row["id"].as_u64().unwrap() as u32)))
    {
        let prizes = row["prizes"].as_array().unwrap();
        assert!(
            !prizes.is_empty(),
            "battle {} has no usable prize",
            row["id"]
        );
        assert!(prizes.len() <= 4);
        assert!(prizes.iter().all(|prize| {
            i32_field(prize, "item_id") > 0
                && i32_field(prize, "name_id") > 0
                && i32_field(prize, "icon_id") > 0
        }));
        assert!((1..=1_000_000).contains(&u64_field(row, "reward_coins")));
    }

    for id in NEW_ROSTER {
        let row = battle(&file, id);
        assert_eq!(row["difficulty"], 2);
        assert_eq!(row["required_combat_level"], 0);
        assert_eq!(row["zone"]["zone_id"], 9600);
        assert_eq!(row["zone"]["name"], "sg_random_encounter_clearing");
        assert_eq!(row["enemies"].as_array().unwrap().len(), 3);
    }
}

#[test]
fn every_prize_resolves_and_every_enemy_batch_is_available() {
    let file = battles();
    for id in TASK_ROSTER {
        for prize in battle(&file, id)["prizes"].as_array().unwrap() {
            let item_id = i32_field(prize, "item_id");
            assert!(
                runtime_inventory().has_definition(item_id),
                "battle {id} prize {item_id} is absent from the runtime catalogue"
            );
        }
    }

    let content = manifest_dir().join("data/content");
    EnemyEngine::load(&content).expect("the shipped enemy batches load");
    let definitions = read_json(&content.join("enemies.json"))["definitions"]
        .as_array()
        .unwrap()
        .clone();
    let locale: Vec<Value> =
        read_json(&manifest_dir().join("../../../lab/reports/locale-id-to-text.json"))
            .as_array()
            .unwrap()
            .clone();
    let names: BTreeSet<(i32, String)> = locale
        .iter()
        .map(|row| {
            (
                row["id"].as_i64().unwrap() as i32,
                row["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    let models: BTreeSet<i32> = fs::read_to_string(
        manifest_dir().join("../../../lab/working/OSFR.Client/OSFR Client/Resources/Models.txt"),
    )
    .unwrap()
    .lines()
    .filter_map(|line| line.split('^').next()?.parse().ok())
    .collect();

    for id in NEW_ROSTER {
        let row = battle(&file, id);
        for actor in
            std::iter::once(&row["spawner"]).chain(row["enemies"].as_array().unwrap().iter())
        {
            let definition_id = i32_field(actor, "definition_id");
            let model_id = i32_field(actor, "model_id");
            let name_id = i32_field(actor, "name_id");
            let name = actor["name"].as_str().unwrap();
            let definition = definitions
                .iter()
                .find(|definition| {
                    definition["definition_id"].as_i64() == Some(i64::from(definition_id))
                        && definition["model_id"].as_i64() == Some(i64::from(model_id))
                        && definition["name_string_id"].as_i64() == Some(i64::from(name_id))
                })
                .unwrap_or_else(|| {
                    panic!(
                        "battle {id} actor {name} has no matching enemy definition/model/name row"
                    )
                });
            assert_eq!(definition["boss"], false);
            assert!(
                models.contains(&model_id),
                "battle {id} model {model_id} is absent from Models.txt"
            );
            assert!(
                names.contains(&(name_id, name.to_owned())),
                "battle {id} name {name_id} does not resolve to {name}"
            );
        }
    }
}

#[test]
fn petty_yetis_full_route_pays_once() {
    full_route_pays_once(12310);
}

#[test]
fn pixie_hunters_full_route_pays_once() {
    full_route_pays_once(12311);
}

fn full_route_pays_once(id: u32) {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, id);
    let spawner_guid = u64_field(&row["spawner"], "guid");
    let marker = position(&row["spawner"]["position"]);
    let enemies = row["enemies"].as_array().unwrap().clone();
    let enemy_guids: Vec<u64> = enemies
        .iter()
        .map(|enemy| u64_field(enemy, "guid"))
        .collect();
    let prize_ids: BTreeSet<i32> = row["prizes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|prize| i32_field(prize, "item_id"))
        .collect();

    let mut engine = fixture.engine();
    engine.set_player_job(BRAWLER, 3);

    let screen = engine
        .handle(&interact(spawner_guid), 1, marker, BRAWLER, 0)
        .unwrap();
    let details = screen
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
        .expect("clicking the marker opens 41/114");
    assert_eq!(i32_at(details, 47), i32_field(row, "title_id"));
    assert_eq!(i32_at(details, 51), -1);
    assert_eq!(i32_at(details, 55), i32_field(row, "description_id"));
    assert_eq!(i32_at(details, 59), 2);
    assert_eq!(engine.take_pending_battle(), Some(id));

    let entered = engine.begin(id, world_location(marker), BRAWLER).unwrap();
    let arena = entered.zoning.expect("Go zones into the shared arena");
    assert_eq!(arena.zone_id, 9600);
    assert_eq!(arena.name, "sg_random_encounter_clearing");
    assert_eq!(arena.position, [136.0, -0.5, 155.0]);

    let ready = engine.on_zone_ready(BRAWLER, 1000).unwrap();
    for guid in &enemy_guids {
        assert!(
            ready
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[35, 0, 2, 0])
                    && packet[4..12] == guid.to_le_bytes()),
            "enemy {guid} spawned"
        );
    }
    assert!(
        ready.replies.iter().any(|packet| packet
            .windows(4)
            .any(|field| field == i32_field(row, "objective_id").to_le_bytes())),
        "the recovered goal reaches the client"
    );

    let mut now = 2000;
    let mut win = None;
    for guid in &enemy_guids {
        loop {
            let (health, position) = {
                let active = engine.state().active.as_ref().unwrap();
                let enemy = active
                    .enemies
                    .iter()
                    .find(|enemy| enemy.guid == *guid)
                    .unwrap();
                (enemy.health, enemy.position)
            };
            if health == 0 {
                break;
            }
            now += 700;
            let outcome = engine.attack(*guid, position, BRAWLER, 0, now).unwrap();
            if !outcome.rewards.is_empty() {
                assert!(win.is_none(), "the kill-all goal paid more than once");
                win = Some(outcome);
            }
        }
    }

    let win = win.expect("the final required kill completes the battle");
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        EncounterPhase::Won
    );
    assert_eq!(win.rewards.len(), 1, "one reward bundle");
    assert_eq!(win.rewards[0].coins, u64_field(row, "reward_coins") as u32);
    assert_eq!(win.rewards[0].job_xp, 50);
    assert_eq!(win.rewards[0].items.len(), 1);
    let (item_id, quantity) = win.rewards[0].items[0];
    assert!(prize_ids.contains(&item_id));
    assert_eq!(quantity, 1);

    let repeated = engine
        .attack(enemy_guids[0], [0.0; 3], BRAWLER, 0, now + 700)
        .unwrap();
    assert!(repeated.rewards.is_empty(), "a won fight cannot pay twice");

    let exit = engine
        .handle(&[26, 0, 42, 0], 1, [0.0; 3], BRAWLER, now + 1400)
        .unwrap();
    assert_eq!(exit.completed, Some(id));
    assert!(exit.rewards.is_empty(), "closing results grants nothing");
    let returned = exit.zoning.expect("results Close returns to the world");
    assert_eq!(returned.zone_id, 0);
    assert_eq!(returned.position, marker);
    assert!(!engine.active(), "the active encounter is cleared");

    let duplicate = engine
        .handle(&[26, 0, 42, 0], 1, [0.0; 3], BRAWLER, now + 2100)
        .unwrap();
    assert!(duplicate.rewards.is_empty());
    assert!(duplicate.zoning.is_none());
    assert!(!duplicate.changed);
}

#[test]
fn marker_collision_and_missing_catalogue_prize_are_rejected() {
    let fixture = Fixture::new();
    let valid = battles();
    let mut subset = valid.clone();
    subset["battles"] = json!([battle(&valid, 96).clone(), battle(&valid, 12310).clone()]);
    let valid_path = fixture.write("valid-subset.json", &subset);

    let mut engine = fixture.empty_engine();
    checked_load(&mut engine, &valid_path).unwrap();
    let before = engine.battle_spawners();
    assert_eq!(before.len(), 2);

    let mut existing = valid.clone();
    existing["battles"] = json!([battle(&valid, 12302).clone()]);
    let mut collision_engine = fixture.empty_engine();
    checked_load(
        &mut collision_engine,
        &fixture.write("valid-existing.json", &existing),
    )
    .unwrap();
    let before_collision = collision_engine.battle_spawners();
    let mut collision = valid.clone();
    collision["battles"] = json!([battle(&valid, 96).clone(), battle(&valid, 12310).clone()]);
    let marker_guid = u64_field(&collision["battles"][0]["spawner"], "guid");
    collision["battles"][1]["spawner"]["guid"] = json!(marker_guid);
    let collision_path = fixture.write("marker-collision.json", &collision);
    assert!(
        checked_load(&mut collision_engine, &collision_path)
            .unwrap_err()
            .contains("duplicate field battle GUID")
    );
    assert_eq!(
        collision_engine.battle_spawners(),
        before_collision,
        "a rejected marker collision must not replace a valid battle"
    );

    let mut missing = subset.clone();
    missing["battles"][1]["prizes"][0]["item_id"] = json!(2_147_483_647_i64);
    let missing_path = fixture.write("missing-prize.json", &missing);
    let mut second = fixture.empty_engine();
    checked_load(&mut second, &valid_path).unwrap();
    let before = second.battle_spawners();
    assert!(
        checked_load(&mut second, &missing_path)
            .unwrap_err()
            .contains("absent from the runtime catalogue")
    );
    assert_eq!(
        second.battle_spawners(),
        before,
        "a missing catalogue prize must not mutate the loaded roster"
    );
}
