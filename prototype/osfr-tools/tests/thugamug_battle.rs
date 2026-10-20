//! T282: Thugamug's bridge as a level-5 field battle (Thugamug and his two Hands).
//!
//! Content-only slice. The battle is one new `field_battles.json` row and the
//! existing route: activity-portal Play (167/1/2) or marker 26/8 -> 41/114
//! start screen -> 41/108 Go (client constructor 0x008b6e70, sender
//! 0x00938f00) -> arena zone 9600 -> 26/42 Close. The 41/114 reader chain is
//! 0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 -> 0x009bdd70; the marker actor is
//! 35/2 AddNpc (constructor 0x0091cac0, reader 0x00921a10); the coin balance
//! is 38/19 (0x0089e670). No new wire field, handler, opcode or engine code is
//! added, so the byte layouts stay the ones `field_battle_roster` pins.
//!
//! Placement evidence: the `9500000322` Thugamug row is the Greenwood bridge
//! poi_anchor at [-1580.39915,-45.37643,-555.223024]. Its full 5 m wander disc
//! reaches 40.06 degrees against this project's 25 degree authoring rule, so
//! the marker uses the nearest 1 m-grid safe point 3.16 m away,
//! [-1581.39915,-45.29,-552.22302], whose 105-sample disc is terrain-covered
//! and at most 24.81 degrees (see the battle's source rows).
//!
//! Thresholds (R, the same conservative authoring rules as T103): centre
//! height within 0.05 m of the bilinear terrain height, 24 ring samples all
//! finite, the 105-sample full disc all finite and at most 25 degrees, and at
//! least 25 m to the nearest spawnable NPC, field enemy and other battle
//! marker. Only `legacy_data` NPC rows spawn (`world.rs` skips POI/zone-centre
//! scaffolding), so the nearest *spawned* NPC is measured, not the Thugamug
//! poi_anchor itself.

use osfr_tools::{
    account_store::AccountRole,
    activity_portal::{Catalogue, Kind, Refusal, Viewer},
    chat,
    encounter::EncounterEngine,
    game_store,
    gameplay::GameplayState,
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState, MAX_ITEMS},
    terrain::{HeightGrid, Terrain},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

/// The new battle. 12340 was free on this base; the row is found by this id
/// and test 1 also pins its published values.
const THUGAMUG: u32 = 12340;
const TITLE: u32 = 90703;
const DESCRIPTION: u32 = 90704;
const OBJECTIVE: u32 = 90715;
const DIFFICULTY: u32 = 3;
const REQUIRED_LEVEL: u32 = 5;
const REWARD_COINS: u32 = 25;
const MARKER_GUID: u64 = 100001234000;
const ENEMY_GUIDS: [u64; 3] = [100001234001, 100001234002, 100001234003];
const MARKER: [f32; 3] = [-1581.39915, -45.29, -552.22302];
const ORIGINAL_ANCHOR: [f32; 3] = [-1580.39915, -45.37643, -555.223024];
const NAME_THUGAMUG: i32 = 103031;
const NAME_HENCHMAN: i32 = 103726;
const MODEL: i32 = 199;
const BRAWLER: i32 = 43;
const RUNTIME_CONTENT: &str = "../../../lab/reports/round70-native-20261016T061000Z/content";
const REFERENCE: &str = "../../../lab/references/rust-gateway-reference";
const REFUSAL_PRIZE: &str = "one of its rewards cannot be granted";
const BATTLE_JOB_XP: u32 = 50;

static SERIAL: AtomicU64 = AtomicU64::new(0);
static RUNTIME_INVENTORY: OnceLock<Inventory> = OnceLock::new();

// ---------------------------------------------------------------------------
// Content fixture and battle row helpers.
// ---------------------------------------------------------------------------

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
            "thugamug-battle-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let reference = manifest_dir().join(REFERENCE);
        let version = read_json(&reference.join("manifest.json"))["client_version"]
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            root,
            reference,
            content: manifest_dir().join(RUNTIME_CONTENT),
            version,
        }
    }

    fn login(&self, gateway: &mut Gateway) {
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
        gateway.handle_at(&login, 0).unwrap();
    }

    /// A logged-in admin session at `position` with the bundled content loaded
    /// through the production restore path.
    fn gateway(&self, position: [f32; 3], guid: u64) -> Gateway {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.guid = guid;
        character.zone_id = 0;
        character.position = position;
        character.rotation = [0.0, 0.0, 1.0];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, None).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        send(&mut gateway, &13_u16.to_le_bytes(), 100);
        send(&mut gateway, &10_u16.to_le_bytes(), 200);
        gateway
    }

    /// The seed session with a bag of exactly `capacity` stacks, restored
    /// through the production `Gateway::load_gameplay` path (no test seam).
    fn gateway_with_bag(
        &self,
        position: [f32; 3],
        guid: u64,
        capacity: usize,
    ) -> (Gateway, InventoryState) {
        let mut seed = self.gateway(position, guid);
        assert!(send(&mut seed, &switch_job(BRAWLER), 300).state_changed);
        send(&mut seed, &chat::local_request("realm level 5"), 350);
        assert_eq!(level(&seed, BRAWLER), REQUIRED_LEVEL);

        let mut snapshot = seed.snapshot();
        let bag = full_bag(&snapshot.gameplay.as_ref().unwrap().inventory, capacity);
        snapshot.gameplay.as_mut().unwrap().inventory = bag.clone();
        let gameplay: GameplayState = snapshot.gameplay.unwrap();

        let mut gateway = Gateway::load_gameplay(
            &self.reference,
            seed.state().clone(),
            &self.content,
            Some(gameplay),
        )
        .unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        send(&mut gateway, &13_u16.to_le_bytes(), 400);
        let loaded = send(&mut gateway, &10_u16.to_le_bytes(), 500);
        assert!(loaded.entered_world);
        assert_eq!(
            gateway.snapshot().gameplay.unwrap().inventory,
            bag,
            "world entry restores the test bag unchanged"
        );
        (gateway, bag)
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

fn lab(relative: &str) -> PathBuf {
    manifest_dir().join("../../../lab").join(relative)
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
        .unwrap_or_else(|| panic!("battle {id} is missing"))
}

fn i32_field(value: &Value, field: &str) -> i32 {
    value[field].as_i64().unwrap() as i32
}

fn u32_field(value: &Value, field: &str) -> u32 {
    value[field].as_u64().unwrap() as u32
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

fn prize_ids(row: &Value) -> BTreeSet<i32> {
    row["prizes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|prize| i32_field(prize, "item_id"))
        .collect()
}

// ---------------------------------------------------------------------------
// Packet helpers (shapes pinned by `field_battle_roster`).
// ---------------------------------------------------------------------------

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

/// 41/108 Go; 0x008b6e70 accepts the encounter and instance from the start
/// screen and 0x00938f00 writes them (the trailing session is zero).
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

fn portal_play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
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

fn i32_at(packet: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
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

fn saved_bytes(gateway: &Gateway) -> Vec<u8> {
    serde_json::to_vec(&gateway.snapshot()).unwrap()
}

// ---------------------------------------------------------------------------
// Economy and fight helpers.
// ---------------------------------------------------------------------------

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

fn active_enemy(gateway: &Gateway) -> Option<(u64, [f32; 3], u32)> {
    let state = gateway.snapshot().gameplay?.encounter?;
    let active = state.active?;
    let enemy = active.enemies.iter().find(|enemy| enemy.health > 0)?;
    Some((enemy.guid, enemy.position, enemy.health))
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

fn open_battle(gateway: &mut Gateway, spawner_guid: u64, row: &Value, now_ms: u64) -> Vec<u8> {
    let screen = send(gateway, &interact(spawner_guid), now_ms);
    let details = details(&screen).expect("clicking the marker opens 41/114");
    assert_eq!(i32_at(details, 4), THUGAMUG as i32);
    assert!(i32_at(details, 8) > 0, "the start screen has an instance");

    // 41/114 reader chain 0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 ->
    // 0x009bdd70: title, icon, description, difficulty, profile type,
    // MiniGameTypes.
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
    assert!(
        active_enemy(gateway).is_some(),
        "the arena starts with a fightable enemy"
    );
    enemy_guids
}

fn fight_to_win(gateway: &mut Gateway, now_ms: &mut u64) {
    let player_guid = gateway.state().guid;
    for _ in 0..200 {
        let Some((guid, position, _)) = active_enemy(gateway) else {
            break;
        };
        let phase = gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .unwrap()
            .phase;
        if phase != osfr_tools::encounter::EncounterPhase::Fighting {
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
    let phase = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .map(|active| active.phase);
    assert_eq!(
        phase,
        Some(osfr_tools::encounter::EncounterPhase::Won),
        "the three actors die through the real attack path"
    );
}

fn assert_payout(before: &Economy, after: &Economy, row: &Value) {
    let prizes = prize_ids(row);
    assert_eq!(after.coins, before.coins + u32_field(row, "reward_coins"));
    assert_eq!(after.experience, before.experience + BATTLE_JOB_XP);
    let gains: Vec<(i32, u32)> = after
        .prizes
        .iter()
        .filter_map(|(id, count)| {
            let prior = before.prizes.get(id).copied().unwrap_or(0);
            (count > &prior).then_some((*id, count - prior))
        })
        .collect();
    assert_eq!(gains.len(), 1, "exactly one prize was granted");
    assert!(prizes.contains(&gains[0].0));
    assert_eq!(gains[0].1, 1);
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

// ---------------------------------------------------------------------------
// Bag fixture: real catalogue items, no prize definition, exact stack count.
// ---------------------------------------------------------------------------

fn runtime_inventory() -> &'static Inventory {
    RUNTIME_INVENTORY.get_or_init(|| {
        Inventory::load(&manifest_dir().join(RUNTIME_CONTENT), None)
            .expect("the round70 runtime item catalogue loads")
    })
}

/// A real uncapped catalogue row that is not one of this battle's prizes, so
/// each filler stack stays separate and no granted prize merges into it.
fn filler_definition() -> i32 {
    static FILLER: OnceLock<i32> = OnceLock::new();
    *FILLER.get_or_init(|| {
        let prizes = prize_ids(battle(&battles(), THUGAMUG));
        let inventory = runtime_inventory();
        inventory
            .definition_ids()
            .find(|id| {
                !prizes.contains(id)
                    && inventory
                        .trade_flags(*id)
                        .is_some_and(|(_, max_stack)| max_stack.is_none())
            })
            .expect("the runtime catalogue has an uncapped filler item")
    })
}

fn stack(id: i32, definition_id: i32, count: u32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

/// Replace the base bag's prize stacks (so each previewed prize needs a new
/// stack) and pad to exactly `capacity` stacks. `capacity == MAX_ITEMS`
/// therefore leaves no room for the four-row bundle; `MAX_ITEMS - 4` does.
fn full_bag(base: &InventoryState, capacity: usize) -> InventoryState {
    assert!(
        base.items.len() <= capacity,
        "the base bag ({} stacks) is larger than {capacity}",
        base.items.len()
    );
    let prizes = prize_ids(battle(&battles(), THUGAMUG));
    let mut state = base.clone();
    let removed: BTreeSet<i32> = state
        .items
        .iter()
        .filter(|item| prizes.contains(&item.definition_id))
        .map(|item| item.id)
        .collect();
    state
        .items
        .retain(|item| !prizes.contains(&item.definition_id));
    state
        .equipment
        .retain(|item| !removed.contains(&item.item_id));
    let mut id = state.items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    let filler = filler_definition();
    while state.items.len() < capacity {
        state.items.push(stack(id, filler, 1));
        id += 1;
    }
    state.next_instance_id = Some(i64::from(id));
    state
}

// ---------------------------------------------------------------------------
// Terrain helpers, mirroring the T103 bounded authoring check.
// ---------------------------------------------------------------------------

fn terrain() -> Terrain {
    Terrain::load(&lab("terrain")).expect("the shipped zone-0/9600 terrain grids load")
}

fn disc_points(x: f32, z: f32) -> Vec<(f32, f32)> {
    let mut points = Vec::new();
    for dx in -5..=5 {
        for dz in -5..=5 {
            let (dx, dz) = (dx as f32, dz as f32);
            if dx * dx + dz * dz <= 25.0 {
                points.push((x + dx, z + dz));
            }
        }
    }
    for angle in (0..360).step_by(15) {
        let radians = (angle as f32).to_radians();
        points.push((x + 5.0 * radians.cos(), z + 5.0 * radians.sin()));
    }
    points
}

fn slope_degrees(grid: &HeightGrid, x: f32, z: f32) -> f32 {
    let [west, east, south, north] = [
        grid.height_at(x - 1.0, z),
        grid.height_at(x + 1.0, z),
        grid.height_at(x, z - 1.0),
        grid.height_at(x, z + 1.0),
    ]
    .map(|height| height.expect("terrain around a sampled point is covered"));
    let dx = (east - west) / 2.0;
    let dz = (north - south) / 2.0;
    dx.hypot(dz).atan().to_degrees()
}

/// `Some(distance)` to the nearest spawned regional NPC: only `legacy_data`
/// rows are actors (`world.rs` skips POI/zone-centre scaffolding).
fn nearest_spawned_npc(marker: [f32; 3]) -> (f32, u64) {
    let npc_dir = manifest_dir().join("data/content/npcs");
    let mut nearest = (f32::INFINITY, 0_u64);
    for entry in fs::read_dir(npc_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let records = read_json(&path);
        for record in records["records"].as_array().unwrap() {
            if record["placement"] != "legacy_data" {
                continue;
            }
            let Some(point) = record["position"].as_array() else {
                continue;
            };
            let distance = ((point[0].as_f64().unwrap() as f32 - marker[0]).powi(2)
                + (point[2].as_f64().unwrap() as f32 - marker[2]).powi(2))
            .sqrt();
            if distance < nearest.0 {
                nearest = (distance, record["npc_id"].as_u64().unwrap());
            }
        }
    }
    nearest
}

fn nearest_field_enemy(marker: [f32; 3]) -> f32 {
    let field = read_json(&manifest_dir().join("data/content/field_enemies.json"));
    field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|enemy| {
            let point = &enemy["position"];
            ((point[0].as_f64().unwrap() as f32 - marker[0]).powi(2)
                + (point[2].as_f64().unwrap() as f32 - marker[2]).powi(2))
            .sqrt()
        })
        .fold(f32::INFINITY, f32::min)
}

fn nearest_other_battle_marker(marker: [f32; 3]) -> f32 {
    battles()["battles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["id"].as_u64() != Some(u64::from(THUGAMUG)))
        .map(|row| {
            let point = position(&row["spawner"]["position"]);
            (point[0] - marker[0]).hypot(point[2] - marker[2])
        })
        .fold(f32::INFINITY, f32::min)
}

// ---------------------------------------------------------------------------
// 167/1/1 reader: header, entry type, count, ten dwords, CanPlayerJoin, then
// the variable-length tail (writer/reader chain 0x00be79d0 -> 0x00be63e0).
// ---------------------------------------------------------------------------

struct PortalRow {
    id: u32,
    title: u32,
    description: u32,
    difficulty: u32,
    can_join: bool,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Cursor<'_> {
    fn take(&mut self, length: usize) -> &[u8] {
        assert!(
            self.bytes.len() >= self.at + length,
            "client stream ran out of bytes"
        );
        let slice = &self.bytes[self.at..self.at + length];
        self.at += length;
        slice
    }

    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take(4).try_into().unwrap())
    }
}

fn decode_list(packet: &[u8]) -> Vec<PortalRow> {
    let mut cursor = Cursor {
        bytes: packet,
        at: 0,
    };
    assert_eq!(cursor.take(2), 167_u16.to_le_bytes());
    assert_eq!(cursor.take(1), [1], "activity service");
    assert_eq!(cursor.take(1), [1], "list sub-op");
    let _entry = cursor.u32();
    let count = cursor.i32();
    assert!(count >= 0);
    let mut rows = Vec::new();
    for _ in 0..count {
        let mut words = [0_u32; 10];
        for word in &mut words {
            *word = cursor.u32();
        }
        let can_join = cursor.take(1)[0] != 0;
        let _preferred = cursor.u32();
        let featured = cursor.u32();
        assert_eq!(featured, 0, "no featured windows");
        let _tutorial = cursor.u32();
        let _members = cursor.take(1);
        for _ in 0..2 {
            let length = cursor.i32();
            assert!(length >= 0);
            cursor.take(length as usize);
        }
        let difficulty = cursor.u32();
        let _unknown_0xc = cursor.u32();
        let _unknown_0x68 = cursor.u32();
        let _mystery_chest_icon = cursor.u32();
        rows.push(PortalRow {
            id: words[0],
            title: words[5],
            description: words[6],
            difficulty,
            can_join,
        });
    }
    assert_eq!(
        cursor.at,
        packet.len(),
        "trailing bytes make the client drop the list"
    );
    rows
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// 1. The real loader accepts the file, the portal publishes the row with the
/// verified ids, and every actor id/name/model is well formed.
#[test]
fn the_battle_loads_and_is_published_with_the_expected_values() {
    let content = manifest_dir().join(RUNTIME_CONTENT);
    let path = battle_path();
    let mut engine = EncounterEngine::load(&content, None).unwrap();
    engine
        .load_field_battles(&path)
        .expect("the real field-battle loader accepts the file");
    let spawners = engine.battle_spawners();
    let marker = spawners
        .iter()
        .find(|entry| entry.0 == MARKER_GUID)
        .expect("the Thugamug marker is published to the world");
    assert_eq!(marker.1, "Thugamug");
    assert_eq!(marker.2, NAME_THUGAMUG);
    assert_eq!(marker.3, MODEL);
    assert_eq!(marker.4, MARKER);
    assert_eq!(marker.5, 11970, "the crossed-swords nameplate is visible");

    let file = battles();
    let row = battle(&file, THUGAMUG);
    assert_eq!(u32_field(row, "title_id"), TITLE);
    assert_eq!(u32_field(row, "description_id"), DESCRIPTION);
    assert_eq!(u32_field(row, "objective_id"), OBJECTIVE);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    assert_eq!(u32_field(row, "required_combat_level"), REQUIRED_LEVEL);
    assert_eq!(u32_field(row, "reward_coins"), REWARD_COINS);
    assert_eq!(i32_field(row, "icon_id"), -1);
    assert_eq!(row["spawner"]["boss"], false);
    assert_eq!(u32_field(&row["spawner"], "max_health"), 1);

    // The roster: one boss and two Hands, all model 199, with distinct GUIDs
    // and battle-local definition ids. The guide gives no gender evidence for
    // the pair, so the single recovered male bandit body is used for both (R).
    let enemies = row["enemies"].as_array().unwrap();
    assert_eq!(enemies.len(), 3);
    assert_eq!(u64_field(&enemies[0], "guid"), ENEMY_GUIDS[0]);
    assert_eq!(enemies[0]["boss"], true);
    assert_eq!(u32_field(&enemies[0], "max_health"), 220);
    assert_eq!(u32_field(&enemies[0], "damage"), 12);
    for hand in &enemies[1..] {
        assert_eq!(hand["boss"], false);
        assert_eq!(
            u32_field(hand, "name_id"),
            u32::try_from(NAME_HENCHMAN).unwrap()
        );
        assert_eq!(u32_field(hand, "model_id"), u32::try_from(MODEL).unwrap());
        assert_eq!(u32_field(hand, "max_health"), 140);
        assert_eq!(u32_field(hand, "damage"), 9);
    }
    let guids: BTreeSet<u64> = std::iter::once(&row["spawner"])
        .chain(enemies.iter())
        .map(|actor| u64_field(actor, "guid"))
        .collect();
    let definitions: BTreeSet<i32> = std::iter::once(&row["spawner"])
        .chain(enemies.iter())
        .map(|actor| i32_field(actor, "definition_id"))
        .collect();
    assert_eq!(guids.len(), 4, "marker and three enemies have unique GUIDs");
    assert_eq!(
        definitions.len(),
        4,
        "each actor has its own battle-local definition id"
    );
    assert!(
        guids.contains(&MARKER_GUID),
        "the marker GUID is one of the new ids"
    );

    // Names resolve in the client locale report; models are drawable.
    let locale: Value = read_json(&lab("reports/locale-id-to-text.json"));
    let names: BTreeMap<i32, String> = locale
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["id"].as_i64().unwrap() as i32,
                entry["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        names.get(&NAME_THUGAMUG).map(String::as_str),
        Some("Thugamug")
    );
    assert_eq!(
        names.get(&NAME_HENCHMAN).map(String::as_str),
        Some("Thugamug Henchman")
    );
    let models =
        fs::read_to_string(lab("working/OSFR.Client/OSFR Client/Resources/Models.txt")).unwrap();
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("199^chugawug_m_bandit_01.adr^")),
        "model 199 is in Models.txt"
    );
    let catalog = fs::read_to_string(lab("extracted/client-assets/catalog.tsv")).unwrap();
    assert!(
        catalog.contains("chugawug_m_bandit_01.adr"),
        "model 199 is present in catalog.tsv"
    );

    // The activity portal publishes the row with the same ids and difficulty.
    let catalogue = Catalogue::load(&path).unwrap();
    let activity = catalogue
        .activities()
        .iter()
        .find(|activity| activity.id == THUGAMUG)
        .expect("the portal publishes the new battle");
    assert_eq!(activity.name_id, TITLE);
    assert_eq!(activity.description_id, DESCRIPTION);
    assert_eq!(activity.difficulty, DIFFICULTY);
    assert_eq!(activity.required_level, REQUIRED_LEVEL);
    assert_eq!(activity.kind, Kind::Battle);
    assert_eq!(
        catalogue
            .admit(
                THUGAMUG,
                Viewer {
                    combat_job: true,
                    level: REQUIRED_LEVEL
                },
                false
            )
            .unwrap()
            .id,
        THUGAMUG
    );
    assert_eq!(
        catalogue.admit(
            THUGAMUG,
            Viewer {
                combat_job: true,
                level: REQUIRED_LEVEL - 1
            },
            false
        ),
        Err(Refusal::LevelTooLow(REQUIRED_LEVEL))
    );
    assert_eq!(
        catalogue.admit(
            THUGAMUG,
            Viewer {
                combat_job: false,
                level: 20
            },
            false
        ),
        Err(Refusal::NotCombatJob)
    );

    // The 167/1/1 row the client reads carries the same values.
    let rows = decode_list(&catalogue.list_packet(Viewer {
        combat_job: true,
        level: REQUIRED_LEVEL,
    }));
    let published = rows.iter().find(|row| row.id == THUGAMUG).unwrap();
    assert_eq!(published.title, TITLE);
    assert_eq!(published.description, DESCRIPTION);
    assert_eq!(published.difficulty, DIFFICULTY);
    assert!(published.can_join);
    let refused = decode_list(&catalogue.list_packet(Viewer {
        combat_job: true,
        level: REQUIRED_LEVEL - 1,
    }));
    assert!(
        !refused
            .iter()
            .find(|row| row.id == THUGAMUG)
            .unwrap()
            .can_join
    );
}

/// 2. The chosen marker is on the terrain, its full 5 m wander disc is
/// standable under the project's authoring rule, and it is clear of the
/// nearest spawned NPC, field enemy and other battle marker. The arena
/// arrival is inside zone 9600.
#[test]
fn the_marker_is_on_the_ground_and_clear() {
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    assert!(
        (marker[0] - ORIGINAL_ANCHOR[0]).hypot(marker[2] - ORIGINAL_ANCHOR[2]) <= 5.0,
        "the marker stays on the bridge landmark"
    );

    let zone = &row["zone"];
    assert_eq!(u32_field(zone, "zone_id"), 9600);
    assert_eq!(zone["name"], "sg_random_encounter_clearing");
    assert_eq!(u32_field(zone, "geometry_id"), 214);
    assert_eq!(zone["sky"], "sky_wilds24.xml");
    let arrival = position(&zone["position"]);
    assert_eq!(arrival, [136.0, -0.5, 155.0]);

    let terrain = terrain();
    let grid = terrain.grid(0).expect("zone 0 is loaded");
    let height = grid
        .height_at(marker[0], marker[2])
        .expect("the marker is on covered ground");
    assert!(
        (marker[1] - height).abs() <= 0.05,
        "marker y {} is {:.4} m from the sampled ground {height}",
        marker[1],
        (marker[1] - height).abs()
    );

    // The 24-point 5 m ring, then all 105 disc samples.
    for angle in (0..360).step_by(15) {
        let radians = (angle as f32).to_radians();
        let (x, z) = (
            marker[0] + 5.0 * radians.cos(),
            marker[2] + 5.0 * radians.sin(),
        );
        let sample = grid
            .height_at(x, z)
            .unwrap_or_else(|| panic!("ring sample {angle} degrees has no ground"));
        assert!(sample.is_finite());
    }
    let disc = disc_points(marker[0], marker[2]);
    assert_eq!(disc.len(), 105);
    for (x, z) in &disc {
        assert!(
            grid.height_at(*x, *z).is_some(),
            "disc sample ({x},{z}) has no ground"
        );
    }
    let max_slope = disc
        .iter()
        .map(|(x, z)| slope_degrees(grid, *x, *z))
        .fold(0.0_f32, f32::max);
    assert!(
        max_slope <= 25.0,
        "the full 5 m wander disc reaches {max_slope:.2} degrees"
    );

    // The bridge poi_anchor itself is why the marker moved: its disc is not
    // standable under the same rule.
    let bridge_slope = disc_points(ORIGINAL_ANCHOR[0], ORIGINAL_ANCHOR[2])
        .iter()
        .map(|(x, z)| slope_degrees(grid, *x, *z))
        .fold(0.0_f32, f32::max);
    assert!(
        bridge_slope > 25.0,
        "the poi_anchor disc should need the recorded 3.16 m adaptation"
    );

    // Thresholds: 25 m to a spawnable NPC, field enemy and other battle
    // marker (T103's conservative authoring gate).
    let (npc, npc_id) = nearest_spawned_npc(marker);
    assert!(
        npc >= 25.0,
        "nearest spawned NPC {npc_id} is {npc:.1} m away"
    );
    let enemy = nearest_field_enemy(marker);
    assert!(enemy >= 25.0, "nearest field enemy is {enemy:.1} m away");
    let other = nearest_other_battle_marker(marker);
    assert!(other >= 25.0, "nearest battle marker is {other:.1} m away");

    let arena = terrain.grid(9600).expect("zone 9600 is loaded");
    assert!(
        arena.height_at(arrival[0], arrival[2]).is_some(),
        "the arena arrival is inside the zone-9600 grid"
    );
}

/// 3. A level-5 combat job opens the 41/114 screen, enters through Go, kills
/// the three actors through the real attack path, and is paid 25 coins, one
/// prize and the active job's 50 XP exactly once.
#[test]
fn a_level_five_character_can_start_and_win_it_once() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(marker, 1001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 5"), 350);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL);

    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    let mut now = 2_000;
    fight_to_win(&mut gateway, &mut now);

    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row);
    let won = gateway.snapshot();
    let replay = send(&mut gateway, &melee(enemy_guids[0]), now + 700);
    assert!(!replay.state_changed, "a won fight cannot pay twice");
    assert_eq!(gateway.snapshot(), won);

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_400);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, marker);
    assert_eq!(
        economy(&gateway, &prizes),
        paid,
        "leaving pays nothing more"
    );
}

/// 4. A level-4 character and a non-combat job are refused by the portal with
/// the existing message, and the refusal changes no saved state.
#[test]
fn a_level_four_or_non_combat_job_is_refused_and_changes_nothing() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(MARKER, 1002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 4"), 350);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL - 1);

    let before = saved_bytes(&gateway);
    let refused = send(&mut gateway, &portal_play(THUGAMUG), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("This activity needs level 5."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        saved_bytes(&gateway),
        before,
        "a level refusal changes no encounter, inventory, coins or save"
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

    // A fresh session stays on the starting Adventurer profile, which is not
    // a combat job. The earlier session cannot switch back to it because the
    // Adventurer outfit lacks the required Chest/Legs slots, so this gate is
    // driven from the real default job.
    let mut civilian = fixture.gateway(MARKER, 1003);
    let before = saved_bytes(&civilian);
    let refused = send(&mut civilian, &portal_play(THUGAMUG), 700);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("Switch to a combat job before starting a battle."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        saved_bytes(&civilian),
        before,
        "a job refusal changes no state"
    );
}

/// 5. T243's whole-bundle preflight: a full bag with none of the prize
/// definitions refuses without changing anything; a bag with four free stacks
/// starts the same battle.
#[test]
fn the_battle_prize_preflight_and_full_bag_refuse() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    let prizes = prize_ids(row);

    // Full bag: the four uncapped prize rows each need a new stack.
    let (mut full, bag) = fixture.gateway_with_bag(marker, 1003, MAX_ITEMS);
    assert_eq!(bag.items.len(), MAX_ITEMS);
    assert!(
        !bag.items
            .iter()
            .any(|item| prizes.contains(&item.definition_id)),
        "no prize definition is owned, so each needs a new stack"
    );
    let before = saved_bytes(&full);
    let refused = send(&mut full, &portal_play(THUGAMUG), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        saved_bytes(&full),
        before,
        "a refused prize bundle changes nothing"
    );
    assert!(
        full.snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );

    // Four free stacks: the whole previewed bundle fits and the battle starts.
    let (mut fitting, bag) = fixture.gateway_with_bag(marker, 1004, MAX_ITEMS - 4);
    assert_eq!(bag.items.len(), MAX_ITEMS - 4);
    assert!(
        !bag.items
            .iter()
            .any(|item| prizes.contains(&item.definition_id)),
        "the fitting bag also starts without the prizes"
    );
    let started = send(&mut fitting, &portal_play(THUGAMUG), 900);
    assert!(started.began_zoning, "the fitting bundle starts the battle");
    assert_eq!(fitting.state().zone_id, 9600);
}

/// 6. Finish, close, save/reload and replay the result/win: the single payout
/// survives the reconnect and is never paid again by the old win.
#[test]
fn reconnect_and_duplicate_win_do_not_pay_twice() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(marker, 1005);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 5"), 350);

    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    arena_spawned(&mut gateway, 1_500);
    let won = send(&mut gateway, &chat::local_request("realm win"), 2_000);
    assert!(won.state_changed);
    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row);

    // Replaying the win on the won run pays nothing before the close.
    let won_state = gateway.snapshot();
    let duplicate_win = send(&mut gateway, &chat::local_request("realm win"), 2_100);
    assert!(!duplicate_win.state_changed);
    assert_eq!(gateway.snapshot(), won_state);

    let closed = send(&mut gateway, &[26, 0, 42, 0], 2_200);
    assert!(closed.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, marker);
    assert_eq!(economy(&gateway, &prizes), paid);
    let after_close = gateway.snapshot();
    let duplicate_close = send(&mut gateway, &[26, 0, 42, 0], 2_300);
    assert!(!duplicate_close.began_zoning && !duplicate_close.state_changed);
    assert_eq!(
        gateway.snapshot(),
        after_close,
        "a duplicate results Close changes nothing"
    );

    // Reconnect: the paid economy and the marker return survive; the old
    // won run is gone and replaying its result pays nothing.
    let mut connected = reopen(&fixture, &gateway, "won.json", 3_000);
    assert_eq!(economy(&connected, &prizes), paid);
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
    let reconnect_before = saved_bytes(&connected);
    let old_hit = send(&mut connected, &melee(enemy_guids[0]), 3_100);
    assert!(!old_hit.state_changed, "the old enemy GUID pays nothing");
    assert_eq!(saved_bytes(&connected), reconnect_before);
    let old_close = send(&mut connected, &[26, 0, 42, 0], 3_100);
    assert!(!old_close.state_changed, "the replayed Close pays nothing");
    assert_eq!(saved_bytes(&connected), reconnect_before);
    let duplicate = send(&mut connected, &chat::local_request("realm win"), 3_200);
    assert!(!duplicate.state_changed, "an orphaned win pays nothing");
    assert_eq!(economy(&connected, &prizes), paid);
}

/// 7. Every battle that existed before this content slice keeps its title,
/// difficulty, required level and coin payout; only the new row is added.
#[test]
fn the_pre_existing_battles_stay_unchanged() {
    let file = battles();
    // The pre-change roster on this base: T252's 12400 is already the seventh
    // row, so seven battles predate T282. Values are the loader-consumed ones
    // the existing tests pin; a balance edit to any row fails here.
    let pre_existing: [(u32, u32, u32, u32, u32); 7] = [
        (96, 42056, 1, 0, 25),
        (12301, 75452, 1, 0, 25),
        (12302, 42054, 2, 0, 25),
        (12303, 426971, 5, 13, 0),
        (12310, 93274, 2, 0, 25),
        (12311, 141039, 2, 0, 25),
        (12400, 24276, 1, 0, 19),
    ];
    for (id, title, difficulty, required, coins) in pre_existing {
        let row = battle(&file, id);
        assert_eq!(u32_field(row, "title_id"), title, "battle {id} title");
        assert_eq!(
            u32_field(row, "difficulty"),
            difficulty,
            "battle {id} difficulty"
        );
        assert_eq!(
            u32_field(row, "required_combat_level"),
            required,
            "battle {id} required level"
        );
        assert_eq!(
            u32_field(row, "reward_coins"),
            coins,
            "battle {id} coin payout"
        );
    }
    let ids: Vec<u32> = file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap() as u32)
        .collect();
    // The insertion anchor is immediately after battle 96, not at the end.
    assert_eq!(ids[0], 96);
    assert_eq!(ids[1], THUGAMUG);
    assert_eq!(ids.len(), pre_existing.len() + 1);
    let existing: BTreeSet<u32> = pre_existing.iter().map(|row| row.0).collect();
    let actual: BTreeSet<u32> = ids.iter().copied().collect();
    assert_eq!(
        actual
            .difference(&BTreeSet::from([THUGAMUG]))
            .copied()
            .collect::<BTreeSet<u32>>(),
        existing,
        "only the new row was added"
    );
    assert_eq!(
        file["battles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["title_id"] == json!(TITLE))
            .count(),
        1,
        "the new title is unique"
    );
}

/// 8. Two sessions run their own arena instance: one player's kill and payout
/// never credit or pay the other, and a malformed request from one leaves the
/// other's battle running.
#[test]
fn a_second_player_can_run_its_own_instance() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, THUGAMUG);
    let marker = position(&row["spawner"]["position"]);
    let prizes = prize_ids(row);

    let mut first = fixture.gateway(marker, 2001);
    let mut second = fixture.gateway(marker, 2002);
    for gateway in [&mut first, &mut second] {
        assert!(send(gateway, &switch_job(BRAWLER), 300).state_changed);
        send(gateway, &chat::local_request("realm level 5"), 350);
    }
    let first_screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let second_screen = open_battle(&mut second, MARKER_GUID, row, 500);
    enter_battle(&mut first, &first_screen, row, 1_000);
    enter_battle(&mut second, &second_screen, row, 1_000);

    let first_before = economy(&first, &prizes);
    let second_before = economy(&second, &prizes);
    let (target, target_position, target_health) = active_enemy(&first).unwrap();
    assert_eq!(target, ENEMY_GUIDS[0]);
    let second_target = active_enemy(&second).unwrap();
    assert_eq!(second_target.2, target_health, "both instances start equal");

    // One real hit on the first player's boss. The second player's copy of the
    // same GUID stays at full health and is still fightable.
    let player_guid = first.state().guid;
    let mut now = 2_000;
    walk(
        &mut first,
        player_guid,
        [
            target_position[0] + 1.5,
            target_position[1],
            target_position[2],
        ],
        &mut now,
    );
    now += 700;
    send(&mut first, &melee(target), now);
    first.tick_at(now + 100).unwrap();
    assert!(
        active_enemy(&first)
            .is_none_or(|(guid, _, health)| { guid != target || health < target_health }),
        "the first player's hit lands on its own boss"
    );
    assert_eq!(
        active_enemy(&second).unwrap().2,
        target_health,
        "the first player's hit never damages the second player's boss"
    );
    assert_eq!(economy(&second, &prizes), second_before);

    // The first player's payout is its own.
    let won = send(&mut first, &chat::local_request("realm win"), now + 800);
    assert!(won.state_changed);
    let first_paid = economy(&first, &prizes);
    assert_payout(&first_before, &first_paid, row);
    assert_eq!(
        economy(&second, &prizes),
        second_before,
        "the first player's win never pays the second"
    );

    // A malformed Play on the won first session is its own no-op, and the
    // second player's battle still runs and pays.
    let malformed = send(&mut first, &[167, 0, 1, 2], now + 900);
    assert!(!malformed.began_zoning && !malformed.state_changed);
    assert!(
        malformed.replies.is_empty(),
        "a short Play is silently dropped"
    );
    let still_running = second
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active;
    assert!(still_running.is_some(), "the second battle still runs");
    let won = send(&mut second, &chat::local_request("realm win"), now + 1_000);
    assert!(won.state_changed);
    let second_paid = economy(&second, &prizes);
    assert_payout(&second_before, &second_paid, row);
    assert_eq!(
        economy(&first, &prizes),
        first_paid,
        "the second player's win never pays the first"
    );
}
