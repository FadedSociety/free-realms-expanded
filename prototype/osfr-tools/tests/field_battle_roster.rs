use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::{EncounterEngine, EncounterPhase},
    enemies::EnemyEngine,
    game_store,
    gateway::{Gateway, GatewayOutcome},
    inventory::Inventory,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
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

struct Fixture {
    root: PathBuf,
    reference: PathBuf,
    content: PathBuf,
    version: String,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "field-battle-roster-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();

        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(root.join("npc.bin"), &template).unwrap();

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
            root.join("encounters.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        let reference = manifest_dir().join("../../../lab/references/rust-gateway-reference");
        let version = read_json(&reference.join("manifest.json"))["client_version"]
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            root,
            reference,
            content: manifest_dir().join(RUNTIME_CATALOGUE),
            version,
        }
    }

    fn empty_engine(&self) -> EncounterEngine {
        EncounterEngine::load(&self.root, None).unwrap()
    }

    fn engine(&self) -> EncounterEngine {
        let mut engine = self.empty_engine();
        engine.load_field_battles(&battle_path()).unwrap();
        engine
    }

    fn write(&self, name: &str, value: &Value) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
        path
    }

    fn gateway(&self, position: [f32; 3]) -> Gateway {
        self.gateway_from(&self.content, position)
    }

    fn gateway_from(&self, content: &Path, position: [f32; 3]) -> Gateway {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.zone_id = 0;
        character.position = position;
        character.rotation = [0.0, 0.0, 1.0];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, content, None).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        send(&mut gateway, &13_u16.to_le_bytes(), 100);
        send(&mut gateway, &10_u16.to_le_bytes(), 200);
        gateway
    }

    fn content_copy(&self) -> PathBuf {
        let target = self.root.join("invalid-content");
        fs::create_dir(&target).unwrap();
        for entry in fs::read_dir(&self.content).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_file()
                || matches!(
                    entry.file_name().to_str(),
                    Some("field_battles.json" | "npc_spawns_extra.json" | "zones.json")
                )
            {
                continue;
            }
            let destination = target.join(entry.file_name());
            if fs::hard_link(entry.path(), &destination).is_err() {
                fs::copy(entry.path(), destination).unwrap();
            }
        }
        fs::create_dir(target.join("npcs")).unwrap();
        fs::write(
            target.join("zones.json"),
            json!({
                "schema_version": 1,
                "records": [{"id": 0, "name": "SyntheticWorld"}]
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            target.join("npcs/synthetic.json"),
            json!({
                "schema_version": 1,
                "records": [{
                    "npc_id": 1,
                    "name": "Synthetic NPC",
                    "zone_id": 0,
                    "position": [0.0, 0.0, 0.0],
                    "placement": "legacy_data",
                    "model_id": 1
                }]
            })
            .to_string(),
        )
        .unwrap();
        fs::write(
            target.join("npc_spawns_extra.json"),
            json!({"schema_version": 1, "records": []}).to_string(),
        )
        .unwrap();
        target
    }

    fn login(&self, gateway: &mut Gateway) -> GatewayOutcome {
        let mut login = 1_u16.to_le_bytes().to_vec();
        for (index, value) in ["local-private-test", self.version.as_str(), ""]
            .into_iter()
            .enumerate()
        {
            login.extend((value.len() as u32).to_le_bytes());
            login.extend(value.as_bytes());
            if index == 0 {
                login.extend(gateway.state().guid.to_le_bytes());
            }
        }
        gateway.handle_at(&login, 0).unwrap()
    }

    fn save(&self, gateway: &Gateway, name: &str) -> PathBuf {
        let path = self.root.join(name);
        game_store::save(&path, &gateway.snapshot()).unwrap();
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

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn unframe(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

fn send(gateway: &mut Gateway, inner: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(inner), now_ms)
        .expect("gateway frame")
}

fn interact(guid: u64) -> Vec<u8> {
    [vec![26, 0, 8, 0], guid.to_le_bytes().to_vec(), vec![0]].concat()
}

fn switch_job(profile: i32) -> Vec<u8> {
    [vec![26, 0, 13, 0], profile.to_le_bytes().to_vec()].concat()
}

fn go(details: &[u8]) -> Vec<u8> {
    let mut packet = vec![41, 0, 108, 0];
    packet.extend(&details[4..12]);
    packet.extend(0_u64.to_le_bytes());
    packet
}

fn melee(guid: u64) -> Vec<u8> {
    [
        vec![36, 0, 10, 0],
        1_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        guid.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn move_to(guid: u64, position: [f32; 3]) -> Vec<u8> {
    let mut packet = 125_u16.to_le_bytes().to_vec();
    packet.extend(guid.to_le_bytes());
    for value in position.into_iter().chain([0.0, 0.0, 1.0]) {
        packet.extend(value.to_le_bytes());
    }
    packet.extend([0, 0]);
    packet
}

fn chat_text(outcome: &GatewayOutcome) -> String {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8)
        .map(|packet| {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&packet[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn details(outcome: &GatewayOutcome) -> Option<&[u8]> {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
}

fn arena_spawned(gateway: &mut Gateway, now_ms: u64) -> GatewayOutcome {
    let mut outcome = send(gateway, &13_u16.to_le_bytes(), now_ms);
    outcome.replies.extend(
        send(gateway, &10_u16.to_le_bytes(), now_ms + 100)
            .replies
            .into_iter(),
    );
    outcome
}

fn level(gateway: &Gateway, profile: i32) -> u32 {
    let state = gateway.snapshot().gameplay.unwrap();
    1 + state
        .profiles
        .unwrap()
        .experience
        .get(&profile)
        .copied()
        .unwrap_or(0)
        / 100
}

#[derive(Debug, PartialEq)]
struct Economy {
    coins: u32,
    prizes: BTreeMap<i32, u32>,
    experience: u32,
}

fn economy(gateway: &Gateway, prize_ids: &BTreeSet<i32>) -> Economy {
    let state = gateway.snapshot().gameplay.unwrap();
    Economy {
        coins: state.inventory.coins,
        prizes: prize_ids
            .iter()
            .map(|id| {
                let count = state
                    .inventory
                    .items
                    .iter()
                    .filter(|item| item.definition_id == *id)
                    .map(|item| item.count)
                    .sum();
                (*id, count)
            })
            .collect(),
        experience: state
            .profiles
            .unwrap()
            .experience
            .get(&BRAWLER)
            .copied()
            .unwrap_or(0),
    }
}

fn active_enemy(gateway: &Gateway) -> Option<(u64, [f32; 3], EncounterPhase)> {
    let state = gateway.snapshot().gameplay?.encounter?;
    let active = state.active?;
    let enemy = active.enemies.iter().find(|enemy| enemy.health > 0)?;
    Some((enemy.guid, enemy.position, active.phase))
}

fn walk(gateway: &mut Gateway, guid: u64, to: [f32; 3], now_ms: &mut u64) {
    let from = gateway.state().position;
    let distance = ((to[0] - from[0]).powi(2) + (to[2] - from[2]).powi(2)).sqrt();
    let steps = (distance / 8.0).ceil().max(1.0) as usize;
    for step in 1..=steps {
        let amount = step as f32 / steps as f32;
        let position = [0, 1, 2].map(|axis| from[axis] + (to[axis] - from[axis]) * amount);
        *now_ms += 400;
        let moved = send(gateway, &move_to(guid, position), *now_ms);
        assert!(!moved.replies.iter().any(|reply| {
            let packet = unframe(reply);
            packet.starts_with(&[38, 0, 12, 0])
        }));
        assert_eq!(gateway.state().position, position);
    }
}

fn reopen(fixture: &Fixture, gateway: &Gateway, name: &str, now_ms: u64) -> Gateway {
    let path = fixture.save(gateway, name);
    let saved = game_store::load_or_create(&path, gateway.state().clone()).unwrap();
    let mut restored = Gateway::load_gameplay(
        &fixture.reference,
        saved.character,
        &fixture.content,
        saved.gameplay,
    )
    .unwrap();
    restored.set_dev_tools(true);
    restored.set_role(AccountRole::Admin);
    fixture.login(&mut restored);
    send(&mut restored, &13_u16.to_le_bytes(), now_ms);
    send(&mut restored, &10_u16.to_le_bytes(), now_ms + 100);
    restored
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
    full_route_pays_once(12310, true);
}

#[test]
fn pixie_hunters_full_route_pays_once() {
    full_route_pays_once(12311, false);
}

fn open_battle(gateway: &mut Gateway, spawner_guid: u64, row: &Value, now_ms: u64) -> Vec<u8> {
    let screen = send(gateway, &interact(spawner_guid), now_ms);
    let details = details(&screen).expect("clicking the marker opens 41/114");
    assert_eq!(i32_at(details, 4), u64_field(row, "id") as i32);
    assert!(i32_at(details, 8) > 0, "the start screen has an instance");

    // 41/114 client reader chain 0x00aa3430 -> 0x00aa32d0 ->
    // 0x00a29120 -> 0x009bdd70. The last reader takes the title, icon,
    // description, difficulty and combat profile type consecutively.
    let expected: Vec<u8> = [
        i32_field(row, "title_id"),
        i32_field(row, "icon_id"),
        i32_field(row, "description_id"),
        i32_field(row, "difficulty"),
        2, // MiniGameInfo.ProfileType: combat
        4, // MiniGameTypes.COMBAT_TYPE
    ]
    .into_iter()
    .flat_map(i32::to_le_bytes)
    .collect();
    assert_eq!(
        &details[47..71],
        expected.as_slice(),
        "41/114 fields must agree byte-for-byte with the client reader"
    );
    details.to_vec()
}

fn enter_battle(gateway: &mut Gateway, details: &[u8], row: &Value, now_ms: u64) -> Vec<u64> {
    // 41/108 client constructor 0x008b6e70; 0x00938f00 fills it with the
    // start-screen encounter and instance. The trailing session is zero in
    // native 41/108 frames.
    let entered = send(gateway, &go(details), now_ms);
    assert!(entered.began_zoning, "Go enters the arena");
    assert_eq!(gateway.state().zone_id, 9600);

    let enemy_guids: Vec<u64> = row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|enemy| u64_field(enemy, "guid"))
        .collect();
    let ready = arena_spawned(gateway, now_ms + 100);
    for guid in &enemy_guids {
        assert!(
            ready
                .replies
                .iter()
                .map(|packet| unframe(packet))
                .any(|packet| packet.starts_with(&[35, 0, 2, 0])
                    && packet[4..12] == guid.to_le_bytes()),
            "arena enemy {guid} spawned"
        );
    }
    assert!(
        ready
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .any(|packet| packet
                .windows(4)
                .any(|field| field == i32_field(row, "objective_id").to_le_bytes())),
        "the recovered goal reaches the client"
    );
    assert_eq!(
        active_enemy(gateway).map(|(_, _, phase)| phase),
        Some(EncounterPhase::Fighting)
    );
    enemy_guids
}

fn fight_to_win(gateway: &mut Gateway, now_ms: &mut u64) {
    let player_guid = gateway.state().guid;
    for _ in 0..200 {
        let Some((guid, position, phase)) = active_enemy(gateway) else {
            break;
        };
        if phase != EncounterPhase::Fighting {
            break;
        }
        walk(
            gateway,
            player_guid,
            [position[0] + 1.5, position[1], position[2]],
            now_ms,
        );
        *now_ms += 700;
        // 36/10 writer 0x00a342e0: bar 1, slot 0, target mode 0, target GUID.
        send(gateway, &melee(guid), *now_ms);
        walk(gateway, player_guid, [136.0, -0.5, 155.0], now_ms);
        *now_ms += 100;
        gateway.tick_at(*now_ms).unwrap();
    }
    let state = gateway.snapshot().gameplay.unwrap().encounter.unwrap();
    assert_eq!(
        state.active.as_ref().map(|active| active.phase.clone()),
        Some(EncounterPhase::Won)
    );
}

fn assert_payout(before: &Economy, after: &Economy, row: &Value, prize_ids: &BTreeSet<i32>) {
    assert_eq!(
        after.coins,
        before.coins + u64_field(row, "reward_coins") as u32
    );
    assert_eq!(after.experience, before.experience + 50);
    let gains: Vec<(i32, u32)> = after
        .prizes
        .iter()
        .filter_map(|(id, count)| {
            let prior = before.prizes.get(id).copied().unwrap_or(0);
            (count > &prior).then_some((*id, count - prior))
        })
        .collect();
    assert_eq!(gains.len(), 1, "exactly one prize was granted");
    assert!(prize_ids.contains(&gains[0].0));
    assert_eq!(gains[0].1, 1);
}

fn full_route_pays_once(id: u32, reconnect_mid_activity: bool) {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, id);
    let spawner_guid = u64_field(&row["spawner"], "guid");
    let marker = position(&row["spawner"]["position"]);
    let prize_ids: BTreeSet<i32> = row["prizes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|prize| i32_field(prize, "item_id"))
        .collect();
    let mut gateway = fixture.gateway(marker);

    let details = open_battle(&mut gateway, spawner_guid, row, 500);
    let before_refusal = gateway.snapshot();
    let refused = send(&mut gateway, &go(&details), 600);
    assert!(!refused.began_zoning);
    assert!(!refused.state_changed);
    assert!(
        chat_text(&refused).contains("Switch to a combat job before starting a battle."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        gateway.snapshot(),
        before_refusal,
        "a refused Go preserves the open start screen and all saved state"
    );

    let switched = send(&mut gateway, &switch_job(BRAWLER), 700);
    assert!(switched.state_changed);
    let lowered = send(&mut gateway, &chat::local_request("realm level 3"), 800);
    assert!(
        chat_text(&lowered).contains("level 3"),
        "{:?}",
        lowered.replies
    );
    assert_eq!(level(&gateway, BRAWLER), 3);

    let before = economy(&gateway, &prize_ids);
    let enemy_guids = enter_battle(&mut gateway, &details, row, 1_000);
    let mut now = 2_000;

    if reconnect_mid_activity {
        let connected = reopen(&fixture, &gateway, "mid-battle.json", 20_000);
        assert_eq!(
            economy(&connected, &prize_ids),
            before,
            "a reconnect mid-battle changes no economy state"
        );
        assert_eq!(connected.state().zone_id, 0);
        assert_eq!(connected.state().position, marker);
        assert!(
            connected
                .snapshot()
                .gameplay
                .unwrap()
                .encounter
                .unwrap()
                .active
                .is_none(),
            "a reconnect releases the abandoned battle"
        );
        gateway = connected;
        let details = open_battle(&mut gateway, spawner_guid, row, 20_500);
        let enemy_guids = enter_battle(&mut gateway, &details, row, 21_000);
        now = 22_000;
        assert_eq!(enemy_guids.len(), 3);
    }

    fight_to_win(&mut gateway, &mut now);
    let paid = economy(&gateway, &prize_ids);
    assert_payout(&before, &paid, row, &prize_ids);
    let won = gateway.snapshot();
    let repeated = send(&mut gateway, &melee(enemy_guids[0]), now + 700);
    assert!(!repeated.state_changed);
    assert_eq!(gateway.snapshot(), won, "a won fight cannot pay twice");

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_400);
    assert!(closed.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, marker);
    assert_eq!(economy(&gateway, &prize_ids), paid);
    let after_close = gateway.snapshot();
    let duplicate = send(&mut gateway, &[26, 0, 42, 0], now + 2_100);
    assert!(!duplicate.state_changed);
    assert!(!duplicate.began_zoning);
    assert_eq!(
        gateway.snapshot(),
        after_close,
        "a duplicate results Close changes nothing"
    );

    let connected = reopen(&fixture, &gateway, "closed.json", now + 3_000);
    assert_eq!(economy(&connected, &prize_ids), paid);
    assert_eq!(connected.state().zone_id, 0);
    assert_eq!(connected.state().position, marker);
    assert!(
        connected
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );
}

#[test]
fn marker_collision_is_rejected_without_replacing_the_roster() {
    let fixture = Fixture::new();
    let valid = battles();
    let mut existing = valid.clone();
    existing["battles"] = json!([battle(&valid, 12302).clone()]);
    let mut collision_engine = fixture.empty_engine();
    collision_engine
        .load_field_battles(&fixture.write("valid-existing.json", &existing))
        .unwrap();
    let before_collision = collision_engine.battle_spawners();
    let mut collision = valid.clone();
    collision["battles"] = json!([battle(&valid, 96).clone(), battle(&valid, 12310).clone()]);
    let marker_guid = u64_field(&collision["battles"][0]["spawner"], "guid");
    collision["battles"][1]["spawner"]["guid"] = json!(marker_guid);
    let collision_path = fixture.write("marker-collision.json", &collision);
    let before_collision_state = collision_engine.state().clone();
    assert!(
        collision_engine
            .reload_field_battles(&collision_path)
            .unwrap_err()
            .to_string()
            .contains("duplicate field battle GUID")
    );
    assert_eq!(
        collision_engine.battle_spawners(),
        before_collision,
        "a rejected marker collision must not replace a valid battle"
    );
    assert_eq!(collision_engine.state(), &before_collision_state);
}

#[test]
fn unavailable_field_battle_prize_is_refused_through_gameplay() {
    let fixture = Fixture::new();
    let valid = battles();
    let row = battle(&valid, 12310);
    let marker = position(&row["spawner"]["position"]);
    let spawner_guid = u64_field(&row["spawner"], "guid");
    let mut invalid = valid.clone();
    let invalid_row = invalid["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|battle| battle["id"] == 12310)
        .unwrap();
    invalid_row["prizes"][0]["item_id"] = json!(2_147_483_647_i64);
    let content = fixture.content_copy();
    fs::write(
        content.join("field_battles.json"),
        serde_json::to_vec(&invalid).unwrap(),
    )
    .unwrap();

    let mut gateway = fixture.gateway_from(&content, marker);
    send(&mut gateway, &switch_job(BRAWLER), 300);
    send(&mut gateway, &chat::local_request("realm level 3"), 400);
    let reloaded = send(
        &mut gateway,
        &chat::local_request("realm reload battles"),
        500,
    );
    assert!(
        chat_text(&reloaded).contains("battles"),
        "{}",
        chat_text(&reloaded)
    );

    let before_portal = gateway.snapshot();
    let mut play = vec![167, 0, 1, 2];
    play.extend(12310_u32.to_le_bytes());
    play.extend(0_u32.to_le_bytes());
    let portal = send(&mut gateway, &play, 600);
    assert!(
        chat_text(&portal).contains("one of its rewards is missing"),
        "{}",
        chat_text(&portal)
    );
    assert_eq!(gateway.snapshot(), before_portal);

    let before_dev = gateway.snapshot();
    let dev = send(
        &mut gateway,
        &chat::local_request("realm battle 12310"),
        700,
    );
    assert!(
        chat_text(&dev).contains("one of its rewards is missing"),
        "{}",
        chat_text(&dev)
    );
    assert_eq!(gateway.snapshot(), before_dev);

    let before_sim = gateway.snapshot();
    let sim = send(&mut gateway, &chat::local_request("realm sim 12310"), 800);
    assert!(
        chat_text(&sim).contains("one of its rewards is missing"),
        "{}",
        chat_text(&sim)
    );
    assert_eq!(gateway.snapshot(), before_sim);

    let start = open_battle(&mut gateway, spawner_guid, row, 900);
    let before_go = gateway.snapshot();
    let refused = send(&mut gateway, &go(&start), 1_000);
    assert!(!refused.began_zoning);
    assert!(!refused.state_changed);
    assert!(
        chat_text(&refused).contains("one of its rewards is missing"),
        "{}",
        chat_text(&refused)
    );
    assert!(
        details(&refused).is_some(),
        "the refused start screen remains open"
    );
    assert_eq!(
        gateway.snapshot(),
        before_go,
        "an unavailable prize changes no saved state"
    );
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );
}
