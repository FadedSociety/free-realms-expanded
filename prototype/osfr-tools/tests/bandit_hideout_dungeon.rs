//! T353: the Bandit Hideout as a difficulty-4, level-9 field battle.
//!
//! Content-only slice: one new `field_battles.json` row and the existing route
//! (26/8 marker -> 41/114 start screen -> 41/108 Go -> arena zone 9600 ->
//! 36/10 attacks -> results -> 26/42 Close, or 167/1/2 portal Play). No new
//! wire field, handler, opcode, selector or engine code is added.
//!
//! Client shapes checked in the supplied 2014 client:
//! * 41/108 Go constructor `0x008b6e70` (stores the encounter and instance at
//!   +6/+7 and calls `sub_8A2FD0(108)`).
//! * 41/114 start-screen reader chain `0x00aa3430` -> `0x00aa32d0` ->
//!   `0x00a29120` -> `0x009bdd70`; the trailing reader reads the title, icon,
//!   description, difficulty, combat profile type and type consecutively.
//! * 35/2 AddNpc body reader `0x00921a10` reads the u64 actor id, name id and
//!   model id; the dispatcher is `0x0092f460` (case 2). 35/3 removal is the
//!   same dispatcher, case 3, built by `0x008b3840`.
//! * 32/7 AttackProcessed is built by `0x00a2a470` (`sub_A297E0(7)`); hit
//!   feedback does not change the authoritative HP this test asserts.
//!
//! Placement evidence: the guide gives only "click the barrier near the
//! Greenwood Camp"; no barrier prop or entrance transform is recovered (R).
//! The marker is the visible Muggenstomp model 200 at
//! `[-1903.0, -34.75, -660.0]`, south-east of Lugabow on the camp approach.
//! Using `lab/terrain/zone_0.hgt`, y is the bilinear height `-34.7500`
//! (delta 0.0000 m); all 105 samples of the 5 m wander disc (81 grid + 24 ring)
//! are terrain-covered with a finite y; the full disc reaches at most 14.04
//! degrees of slope. The nearest spawned NPC is Lugabow at 26.78 m, the nearest
//! other battle marker (Thugamug, 12340) is 339.18 m, and the nearest
//! field-enemy home is 946.62 m. Water, object collision and a native walkable
//! route remain unverified, as in the T282/T319 rows.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::{EncounterEngine, EncounterPhase},
    game_store,
    gameplay::Gameplay,
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState, MAX_ITEMS},
    mining::{MINER_PROFILE_ID, MINING_GAME_ID},
    startup_world::{NpcSpawn, parse_add_npc},
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

/// The new row, appended after battle 12400.
const BANDIT_HIDEOUT: u32 = 12360;
const TITLE: i32 = 30422;
const DESCRIPTION: i32 = 6995;
const OBJECTIVE: i32 = 6997;
const DIFFICULTY: u32 = 4;
const REQUIRED_LEVEL: u32 = 9;
const REWARD_COINS: u32 = 60;
const MARKER_GUID: u64 = 100001236000;
const ENEMY_GUIDS: [u64; 5] = [
    100001236001,
    100001236002,
    100001236003,
    100001236004,
    100001236005,
];
/// `-34.75` is the bilinear zone-0 height, already on a decimetre node.
const MARKER: [f32; 3] = [-1903.0, -34.75, -660.0];
const LUGABOW: [f32; 3] = [-1915.146, -36.9307, -636.1354];
const GLUGMORT: [f32; 3] = [-1943.932, -39.94606, -600.8123];
const MUGGENSTOMP_NAME: i32 = 7058;
const BIG_BANDIT_NAME: i32 = 139345;
const BOSS_MODEL: i32 = 200;
const BANDIT_MODEL: i32 = 199;
const BOSS_BASE_HEALTH: u32 = 320;
const BOSS_BASE_DAMAGE: u32 = 22;
const BANDIT_BASE_HEALTH: u32 = 160;
const BANDIT_BASE_DAMAGE: u32 = 12;
/// The loader's difficulty-4 factor is `level_health(9) / 1000 = 3063 / 1000`;
/// the live arena health is the authored baseline times that factor.
const BOSS_LIVE_HEALTH: u32 = 980;
const BANDIT_LIVE_HEALTH: u32 = 490;
const NAMEPLATE: i32 = 11970;
const BATTLE_ICON: i32 = 4543;
const BRAWLER: i32 = 43;
const BATTLE_JOB_XP: u32 = 50;
const ARENA_ZONE: u32 = 9600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
const PRIZE_IDS: [i32; 4] = [10482, 3023, 75354, 11002];
/// The designated five-battle roster from `field_battle_roster`; the new row
/// is deliberately not part of it.
const TASK_ROSTER: [u32; 5] = [96, 12301, 12302, 12310, 12311];
const RUNTIME_CONTENT: &str = "../../../lab/reports/round70-native-20261016T061000Z/content";
const MINING_CONTENT: &str = "../../../lab/references/opening-journal-text-v2/content";
const REFERENCE: &str = "../../../lab/references/rust-gateway-reference";
const REFUSAL_PRIZE: &str = "one of its rewards cannot be granted";
/// The exact refusal sentence is a local (R) copy in
/// `gameplay/encounter_rewards.rs` and has changed wording twice on the
/// integration tip; the stable contract is that the reward was not granted,
/// which the byte-equal economy assertion also proves.
const REWARD_REFUSED: &str = "not granted";

/// The fixed fields of the 41/114 details packet this test decodes. The header
/// is 12 bytes; the six i32 at 47..71 are title/icon/description/difficulty/
/// profile type/type; 138 zero bytes then the prize bundle at 210; the roster
/// count sits at 525 after the two i32 bundles and 69 zero bytes.
const DETAILS_TITLE: usize = 47;
const DETAILS_PRIZE_BUNDLE: usize = 210;
const DETAILS_PRIZE_COUNT: usize = DETAILS_PRIZE_BUNDLE + 61;
const DETAILS_PRIZE_ENTRY: usize = 38;
const DETAILS_ROSTER: usize = 525;

static SERIAL: AtomicU64 = AtomicU64::new(0);
static RUNTIME_INVENTORY: OnceLock<Inventory> = OnceLock::new();

// ---------------------------------------------------------------------------
// Production-shaped session fixture.
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
            "bandit-hideout-{}-{}",
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

    /// A logged-in session at `position`, including the 10/13 world-entry
    /// replies (AddNpc visibility, portal list and the encounter state).
    fn session(&self, position: [f32; 3], guid: u64) -> (Gateway, GatewayOutcome) {
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
        let loaded = send(&mut gateway, &10_u16.to_le_bytes(), 200);
        (gateway, loaded)
    }

    fn gateway(&self, position: [f32; 3], guid: u64) -> Gateway {
        self.session(position, guid).0
    }

    /// Restore a save built by the production `game_store` path. The seed has
    /// already entered the world and is on a combat job.
    fn gateway_with_bag(
        &self,
        position: [f32; 3],
        guid: u64,
        capacity: usize,
    ) -> (Gateway, InventoryState) {
        let mut seed = self.gateway(position, guid);
        assert!(send(&mut seed, &switch_job(BRAWLER), 300).state_changed);
        send(&mut seed, &chat::local_request("realm level 20"), 350);
        assert_eq!(level(&seed, BRAWLER), 20);

        let mut snapshot = seed.snapshot();
        let bag = full_bag(&snapshot.gameplay.as_ref().unwrap().inventory, capacity);
        snapshot.gameplay.as_mut().unwrap().inventory = bag.clone();
        let gameplay = snapshot.gameplay.unwrap();
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

    /// The same restore path, with a coin balance a save can hold but a 60-coin
    /// payout cannot add to.
    fn gateway_with_coins(&self, position: [f32; 3], guid: u64, coins: u32) -> Gateway {
        let mut seed = self.gateway(position, guid);
        assert!(send(&mut seed, &switch_job(BRAWLER), 300).state_changed);
        send(&mut seed, &chat::local_request("realm level 20"), 350);

        let mut snapshot = seed.snapshot();
        snapshot.gameplay.as_mut().unwrap().inventory.coins = coins;
        let gameplay = snapshot.gameplay.unwrap();
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
        assert!(send(&mut gateway, &10_u16.to_le_bytes(), 500).entered_world);
        assert_eq!(gateway.snapshot().gameplay.unwrap().inventory.coins, coins);
        gateway
    }

    /// A session on a private opening package with a live Miner and a bound
    /// mining game, for the busy refusal. It copies the package to the temp
    /// root first; no lab content is written.
    fn mining_gateway(&self, guid: u64) -> Gateway {
        let directory = self.root.join("mining-content");
        copy_dir(&manifest_dir().join(MINING_CONTENT), &directory);
        osfr_tools::mining::write_content(&directory, MINING_GAME_ID).unwrap();
        let mut state = Gameplay::load(&directory, None).unwrap().state();
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = MINER_PROFILE_ID;
        profiles.unlocked.insert(MINER_PROFILE_ID);
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.guid = guid;
        character.zone_id = 0;
        character.position = MARKER;
        character.rotation = [0.0, 0.0, 1.0];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &directory, Some(state)).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        self.login(&mut gateway);
        send(&mut gateway, &13_u16.to_le_bytes(), 100);
        send(&mut gateway, &10_u16.to_le_bytes(), 200);
        gateway
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

fn content_dir() -> PathBuf {
    manifest_dir().join("data/content")
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

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
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

fn portal_play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0_u32.to_le_bytes());
    inner
}

/// 41/108 Go; constructor `0x008b6e70` accepts the encounter and instance from
/// the 41/114 screen. The trailing session is zero in native frames.
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

fn i32_at(packet: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
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

fn has_add_npc(outcome: &GatewayOutcome, guid: u64) -> bool {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .any(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == guid.to_le_bytes())
}

// ---------------------------------------------------------------------------
// Directed visibility: the 35/2 AddNpc the client reader 0x00921a10 parses.
// ---------------------------------------------------------------------------

fn marker_packet<'a>(replies: &'a [Vec<u8>], guid: u64) -> &'a [u8] {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && packet.len() >= 12
                && u64::from_le_bytes(packet[4..12].try_into().unwrap()) == guid
        })
        .unwrap_or_else(|| panic!("actor {guid} was not sent as 35/2 AddNpc"))
}

fn marker_spawn(packet: &[u8]) -> NpcSpawn {
    parse_add_npc(packet).expect("the 35/2 AddNpc body parses through the production reader")
}

// ---------------------------------------------------------------------------
// 167/1/1 reader: header, entry type, count, ten dwords, CanPlayerJoin, then
// the variable-length tail (reader chain 0x00be79d0 -> 0x00be63e0).
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

fn published_list(replies: &[Vec<u8>]) -> Vec<PortalRow> {
    let packet = replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[167, 0, 1, 1]))
        .expect("a 167/1/1 activity list was sent");
    decode_list(packet)
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

fn next_instance(gateway: &Gateway) -> i32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .next_instance_id
}

// ---------------------------------------------------------------------------
// Economy, fight and reconnect helpers.
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
struct Economy {
    coins: u32,
    prizes: BTreeMap<i32, u32>,
    experience: BTreeMap<i32, u32>,
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
        experience: state.profiles.unwrap().experience,
    }
}

fn count_definition(gateway: &Gateway, definition_id: i32) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
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

fn open_battle(gateway: &mut Gateway, spawner_guid: u64, row: &Value, now_ms: u64) -> Vec<u8> {
    let screen = send(gateway, &interact(spawner_guid), now_ms);
    let details = details(&screen).expect("clicking the marker opens 41/114");
    assert_eq!(i32_at(details, 4), BANDIT_HIDEOUT as i32);
    assert!(i32_at(details, 8) > 0, "the start screen has an instance");

    // 41/114 reader chain 0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 ->
    // 0x009bdd70: title, icon, description, difficulty, profile type, type.
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
        &details[DETAILS_TITLE..DETAILS_TITLE + expected.len()],
        expected.as_slice(),
        "41/114 fields must agree byte-for-byte with the client reader"
    );
    details.to_vec()
}

/// Decode the start screen's prize preview through the 41/114 layout:
/// (icon id, name id, quantity, item id) per bundle entry.
fn prize_preview(details: &[u8]) -> Vec<(i32, i32, i32, i32)> {
    let count = i32_at(details, DETAILS_PRIZE_COUNT);
    assert!(count >= 0 && count <= 4);
    (0..count as usize)
        .map(|index| {
            let entry = DETAILS_PRIZE_BUNDLE + 61 + 4 + index * DETAILS_PRIZE_ENTRY;
            (
                i32_at(details, entry + 5),
                i32_at(details, entry + 13),
                i32_at(details, entry + 17),
                i32_at(details, entry + 21),
            )
        })
        .collect()
}

fn enter_battle(gateway: &mut Gateway, details: &[u8], row: &Value, now_ms: u64) -> Vec<u64> {
    let entered = send(gateway, &go(details), now_ms);
    assert!(entered.began_zoning, "Go enters the arena");
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);

    let enemy_guids: Vec<u64> = row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|enemy| u64_field(enemy, "guid"))
        .collect();
    let ready = arena_spawned(gateway, now_ms + 100);
    for guid in &enemy_guids {
        assert!(has_add_npc(&ready, *guid), "arena enemy {guid} spawned");
    }
    assert!(
        ready
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .any(|packet| packet.starts_with(&[41, 0, 114, 0])),
        "the running encounter refreshes through 41/114"
    );
    assert!(
        ready
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .any(|packet| packet
                .windows(4)
                .any(|field| field == OBJECTIVE.to_le_bytes())),
        "the verified goal 6997 reaches the client"
    );
    let active = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .expect("the arena run is active");
    assert_eq!(active.phase, EncounterPhase::Fighting);
    assert_eq!(active.enemies.len(), enemy_guids.len());
    assert_eq!(active.instance_id, i32_at(details, 8));
    for (index, (actor, state)) in row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .zip(&active.enemies)
        .enumerate()
    {
        assert_eq!(state.guid, u64_field(actor, "guid"));
        assert_eq!(state.position, position(&actor["position"]));
        assert_eq!(
            state.health,
            if index == 0 {
                BOSS_LIVE_HEALTH
            } else {
                BANDIT_LIVE_HEALTH
            },
            "the loader scales the authored baseline once"
        );
    }
    enemy_guids
}

/// Kill every arena actor through the ordinary 36/10 path. Returns every reply
/// the fight produced so the caller can assert the results packets.
fn fight_to_win(gateway: &mut Gateway, now_ms: &mut u64) -> Vec<Vec<u8>> {
    let player_guid = gateway.state().guid;
    let mut replies = Vec::new();
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
        let hit = send(gateway, &melee(guid), *now_ms);
        replies.extend(hit.replies);
        walk(gateway, player_guid, ARENA_ARRIVAL, now_ms);
        *now_ms += 100;
        let tick = gateway.tick_at(*now_ms).unwrap();
        replies.extend(tick.replies);
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
        Some(EncounterPhase::Won),
        "the five actors die through the real attack path"
    );
    replies
}

/// The result lifecycle produced by `present_results`: 39/18 (game over),
/// 39/47 (scores) and 39/19 (results UI). 39/45 is the loot wheel.
fn results_lifecycle(replies: &[Vec<u8>]) -> Vec<u8> {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[39, 0]) && packet.len() >= 3)
        .map(|packet| packet[2])
        .filter(|sub| [18, 47, 19].contains(sub))
        .collect()
}

fn won_flag(replies: &[Vec<u8>]) -> Option<u8> {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.len() >= 16 && packet.starts_with(&[39, 0, 18]))
        .map(|packet| packet[15])
}

fn assert_payout(before: &Economy, after: &Economy, row: &Value, prize_ids: &BTreeSet<i32>) {
    assert_eq!(after.coins, before.coins + u32_field(row, "reward_coins"));
    // The active combat job is the only profile that gains the field-battle XP.
    for (profile, experience) in &after.experience {
        let prior = before.experience.get(profile).copied().unwrap_or(0);
        let delta = experience - prior;
        assert_eq!(
            delta,
            if *profile == BRAWLER {
                BATTLE_JOB_XP
            } else {
                0
            },
            "profile {profile} gained {delta} XP"
        );
    }
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

fn saved_bytes(gateway: &Gateway) -> Vec<u8> {
    // The typed snapshot payload the save envelope writes.
    serde_json::to_vec(&gateway.snapshot()).unwrap()
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
// Full bag: real catalogue items, no prize definition, exact stack count.
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
        let prizes = prize_ids(battle(&battles(), BANDIT_HIDEOUT));
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
/// leaves no room for the four-row bundle; `MAX_ITEMS - 4` does.
fn full_bag(base: &InventoryState, capacity: usize) -> InventoryState {
    assert!(
        base.items.len() <= capacity,
        "the base bag ({} stacks) is larger than {capacity}",
        base.items.len()
    );
    let prizes = prize_ids(battle(&battles(), BANDIT_HIDEOUT));
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
// Terrain helpers, mirroring the T103/T282 bounded authoring check.
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
    let npc_dir = content_dir().join("npcs");
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
    let field = read_json(&content_dir().join("field_enemies.json"));
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
        .filter(|row| row["id"].as_u64() != Some(u64::from(BANDIT_HIDEOUT)))
        .map(|row| {
            let point = position(&row["spawner"]["position"]);
            (point[0] - marker[0]).hypot(point[2] - marker[2])
        })
        .fold(f32::INFINITY, f32::min)
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// A level-8 combat job cannot start the battle through Go: 41/108 answers
/// with the shared level refusal, changes no saved state and leaves the same
/// pending screen usable after the job reaches level 9.
#[test]
fn level_eight_is_refused_and_reopens_the_screen() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let mut gateway = fixture.gateway(MARKER, 1001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 8"), 350);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL - 1);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let before = gateway.snapshot();
    let refused = send(&mut gateway, &go(&screen), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("This activity needs level 9."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "a level refusal consumes no open screen, coin, item or cooldown"
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
    // The 41/108 level branch answers in chat and leaves the pending screen in
    // place; it does not re-send 41/114. The identical Go starts once the job
    // reaches the floor, which proves the screen was not consumed.
    assert!(
        details(&refused).is_none(),
        "the level branch answers in chat without a second 41/114"
    );
    send(&mut gateway, &chat::local_request("realm level 9"), 700);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL);
    let started = send(&mut gateway, &go(&screen), 800);
    assert!(started.began_zoning, "the pending screen still starts");
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
}

/// A level-9 combat job enters the arena: five actors spawn in zone 9600 with
/// the loader's single difficulty-4 scaling, and an explicit 41/109 exit
/// returns the character to the marker.
#[test]
fn level_nine_starts_the_battle() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let mut gateway = fixture.gateway(MARKER, 1002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 9"), 350);
    assert_eq!(level(&gateway, BRAWLER), REQUIRED_LEVEL);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let instance = i32_at(&screen, 8);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    assert_eq!(enemy_guids, ENEMY_GUIDS);
    let active = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .unwrap();
    assert_eq!(active.encounter_id, BANDIT_HIDEOUT);
    assert_eq!(active.instance_id, instance);

    // 41/109 LeaveMiniGameOrEncounter: encounter 0 and the launch instance.
    let cancel = [
        vec![41, 0, 109, 0],
        0_i32.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
    ]
    .concat();
    let left = send(&mut gateway, &cancel, 2_000);
    assert!(left.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none(),
        "the exit clears the encounter, leaving no stuck busy state"
    );
}

/// The 41/114 start screen carries this row's instance, roster count, title,
/// icon, description and difficulty, and the four previewed prizes.
#[test]
fn the_start_screen_carries_the_instance_and_roster_count() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let mut gateway = fixture.gateway(MARKER, 1003);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 9"), 350);

    let before = next_instance(&gateway);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    assert_eq!(
        i32_at(&screen, 8),
        before,
        "the screen carries the instance"
    );
    assert_eq!(
        i32_at(&screen, DETAILS_ROSTER),
        row["enemies"].as_array().unwrap().len() as i32,
        "the screen carries the roster count"
    );
    let preview = prize_preview(&screen);
    assert_eq!(preview.len(), 4);
    for (entry, prize) in preview.iter().zip(row["prizes"].as_array().unwrap()) {
        assert_eq!(
            *entry,
            (
                i32_field(prize, "icon_id"),
                i32_field(prize, "name_id"),
                1,
                i32_field(prize, "item_id"),
            ),
            "the prize preview matches the row"
        );
    }
    // The client start screen has no level field: the level floor is enforced
    // server-side at 41/108 and reported in chat.
    assert!(screen.len() > DETAILS_ROSTER);

    let started = send(&mut gateway, &go(&screen), 600);
    assert!(started.began_zoning);
    assert_eq!(next_instance(&gateway), before + 1);
    let active = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .unwrap();
    assert_eq!(active.instance_id, before);
    assert_eq!(active.enemies.len(), 5);
}

/// The full client route: marker 26/8, Go, five kills through 36/10, results
/// 39/18/47/19, then Close. It pays 60 coins, one prize and 50 job XP once;
/// replayed Go, wheel acknowledgement and Close pay nothing and the Close
/// restores the pre-entry pose.
#[test]
fn the_full_route_pays_sixty_coins_and_one_prize() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(MARKER, 1004);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let pose = (gateway.state().position, gateway.state().rotation);
    let before = economy(&gateway, &prizes);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    assert_eq!(enemy_guids, ENEMY_GUIDS);
    let mut now = 2_000;
    let replies = fight_to_win(&mut gateway, &mut now);
    assert_eq!(
        results_lifecycle(&replies),
        vec![18, 47, 19],
        "the results open through 39/18, 39/47 and 39/19"
    );
    assert_eq!(won_flag(&replies), Some(1), "the results report a win");

    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row, &prizes);
    assert_eq!(
        paid.experience.get(&BRAWLER).copied().unwrap_or(0),
        before.experience.get(&BRAWLER).copied().unwrap_or(0) + BATTLE_JOB_XP,
        "the active job gains 50 XP once"
    );

    // A wheel acknowledgement (39/46), a repeated hit and the old Go are inert.
    let won = gateway.snapshot();
    let wheel = send(&mut gateway, &[39, 0, 46, 0], now + 700);
    assert!(!wheel.state_changed && wheel.replies.is_empty());
    let repeated = send(&mut gateway, &melee(enemy_guids[0]), now + 800);
    assert!(!repeated.state_changed, "a won fight cannot pay twice");
    let replay_go = send(&mut gateway, &go(&screen), now + 900);
    assert!(!replay_go.began_zoning && !replay_go.state_changed);
    assert_eq!(gateway.snapshot(), won);

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_400);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(
        (gateway.state().position, gateway.state().rotation),
        pose,
        "the results Close restores the pre-entry pose"
    );
    assert_eq!(
        economy(&gateway, &prizes),
        paid,
        "leaving pays nothing more"
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
    let after_close = gateway.snapshot();
    let duplicate = send(&mut gateway, &[26, 0, 42, 0], now + 2_100);
    assert!(!duplicate.began_zoning && !duplicate.state_changed);
    assert_eq!(
        gateway.snapshot(),
        after_close,
        "a duplicate results Close changes nothing"
    );
}

/// The activity portal publishes the row automatically and applies the same
/// level floor and whole-bundle prize preflight as Go.
#[test]
fn the_portal_row_applies_the_same_level_and_prize_gates() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    assert_eq!(i32_field(row, "title_id"), TITLE);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    let mut gateway = fixture.gateway(MARKER, 1005);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 8"), 350);
    let below = published_list(&send(&mut gateway, &[167, 0, 1, 4], 400).replies);
    let below = below
        .iter()
        .find(|row| row.id == BANDIT_HIDEOUT)
        .expect("the portal publishes Bandit Hideout");
    assert!(!below.can_join, "a level-8 combat job cannot join");
    assert_eq!(below.title, TITLE as u32);
    assert_eq!(below.description, DESCRIPTION as u32);
    assert_eq!(below.difficulty, DIFFICULTY);

    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let refused = send(&mut gateway, &portal_play(BANDIT_HIDEOUT), 500);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("This activity needs level 9."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);

    send(&mut gateway, &chat::local_request("realm level 9"), 600);
    let allowed = published_list(&send(&mut gateway, &[167, 0, 1, 4], 700).replies);
    assert!(
        allowed
            .iter()
            .find(|row| row.id == BANDIT_HIDEOUT)
            .is_some_and(|row| row.can_join),
        "a level-9 combat job may join"
    );
    let started = send(&mut gateway, &portal_play(BANDIT_HIDEOUT), 800);
    assert!(started.began_zoning);
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
    arena_spawned(&mut gateway, 850);
    let active = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .unwrap();
    assert_eq!(active.encounter_id, BANDIT_HIDEOUT);
    assert_eq!(active.enemies.len(), 5);

    // The same Play on a full bag refuses at the whole-bundle preflight.
    let (mut full, bag) = fixture.gateway_with_bag(MARKER, 1006, MAX_ITEMS);
    assert_eq!(bag.items.len(), MAX_ITEMS);
    let before = (full.snapshot(), saved_bytes(&full));
    let refused = send(&mut full, &portal_play(BANDIT_HIDEOUT), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&refused)
    );
    assert_eq!((full.snapshot(), saved_bytes(&full)), before);
    assert!(
        full.snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );

    let (mut fitting, bag) = fixture.gateway_with_bag(MARKER, 1007, MAX_ITEMS - 4);
    assert_eq!(bag.items.len(), MAX_ITEMS - 4);
    let started = send(&mut fitting, &portal_play(BANDIT_HIDEOUT), 900);
    assert!(started.began_zoning, "the fitting bundle starts the battle");
    assert_eq!(fitting.state().zone_id, ARENA_ZONE);
}

/// Forged, out-of-range, knocked-out, full-bag and already-consumed requests
/// change no saved state; the controls prove the same requests start when the
/// qualifier holds.
#[test]
fn refusals_leave_state_and_pending_battle_unchanged() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);

    // Forged Go with no open screen.
    let mut gateway = fixture.gateway(MARKER, 2001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let mut forged = vec![41, 0, 108, 0];
    forged.extend(BANDIT_HIDEOUT.to_le_bytes());
    forged.extend(0x7fff_i32.to_le_bytes());
    forged.extend(0_u64.to_le_bytes());
    let refused = send(&mut gateway, &forged, 400);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(refused.replies.is_empty(), "a forged Go has no reply");
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);

    // Wrong marker: a forged actor GUID opens no start screen.
    let wrong = send(&mut gateway, &interact(0xdead_beef), 500);
    assert!(details(&wrong).is_none());
    assert!(!wrong.state_changed && !wrong.began_zoning);
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);

    // Out-of-range Go: the screen opens at the marker, the player walks 50 m
    // away and Go is inert. The out-of-reach screen is dropped, so the same Go
    // stays inert after walking back; reopening at the marker works.
    let mut gateway = fixture.gateway(MARKER, 2002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let player = gateway.state().guid;
    let mut now = 600;
    walk(
        &mut gateway,
        player,
        [MARKER[0] + 50.0, MARKER[1], MARKER[2]],
        &mut now,
    );
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let refused = send(&mut gateway, &go(&screen), now + 100);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);
    walk(&mut gateway, player, MARKER, &mut now);
    let replay = send(&mut gateway, &go(&screen), now + 100);
    assert!(
        !replay.began_zoning && !replay.state_changed,
        "an out-of-reach screen was dropped, not kept for a later Go"
    );
    let screen = open_battle(&mut gateway, MARKER_GUID, row, now + 200);
    assert!(send(&mut gateway, &go(&screen), now + 300).began_zoning);

    // Knocked out: Go is a chat refusal that reopens the same screen.
    let mut knocked = fixture.gateway(MARKER, 2003);
    assert!(send(&mut knocked, &switch_job(BRAWLER), 300).state_changed);
    send(&mut knocked, &chat::local_request("realm level 20"), 350);
    let screen = open_battle(&mut knocked, MARKER_GUID, row, 400);
    knocked.receive_field_damage(u32::MAX, 500, true).unwrap();
    assert_eq!(
        knocked
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .player_health,
        0
    );
    let before = (knocked.snapshot(), saved_bytes(&knocked));
    let refused = knocked
        .handle_at(&wrap(&go(&screen)), 600)
        .expect("a knocked-out Go is a chat refusal, not a session error");
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("knocked out"),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        details(&refused).expect("the start screen reopens"),
        screen.as_slice()
    );
    assert_eq!((knocked.snapshot(), saved_bytes(&knocked)), before);

    // Full bag: the four-prize bundle has no room, so Go is refused and the
    // same pending screen is re-sent; a replay refuses identically.
    let (mut full, bag) = fixture.gateway_with_bag(MARKER, 2004, MAX_ITEMS);
    assert_eq!(bag.items.len(), MAX_ITEMS);
    assert!(
        !bag.items
            .iter()
            .any(|item| prizes.contains(&item.definition_id))
    );
    let screen = open_battle(&mut full, MARKER_GUID, row, 600);
    let before = (full.snapshot(), saved_bytes(&full));
    let refused = send(&mut full, &go(&screen), 700);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&refused)
    );
    let shown = details(&refused)
        .expect("the refused start screen remains open")
        .to_vec();
    assert_eq!((full.snapshot(), saved_bytes(&full)), before);
    let repeated = send(&mut full, &go(&screen), 800);
    assert!(!repeated.began_zoning && !repeated.state_changed);
    assert!(
        chat_text(&repeated).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&repeated)
    );
    assert_eq!(
        details(&repeated).expect("a refused Go keeps the start screen pending"),
        shown,
        "the same start screen is shown again"
    );

    // Already consumed: once the arena is live, the old Go is inert.
    let (mut fitting, _) = fixture.gateway_with_bag(MARKER, 2005, MAX_ITEMS - 4);
    let screen = open_battle(&mut fitting, MARKER_GUID, row, 800);
    assert!(send(&mut fitting, &go(&screen), 900).began_zoning);
    arena_spawned(&mut fitting, 950);
    let before = (fitting.snapshot(), saved_bytes(&fitting));
    let replay = send(&mut fitting, &go(&screen), 1_000);
    assert!(!replay.began_zoning && !replay.state_changed);
    assert!(
        details(&replay).is_none(),
        "the consumed screen stays closed"
    );
    assert_eq!((fitting.snapshot(), saved_bytes(&fitting)), before);
}

/// Two independent sessions run their own instance, kills and payout; one
/// player's win neither removes nor damages the other's actors.
#[test]
fn a_second_player_runs_its_own_instance_and_is_paid_once() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);
    let mut first = fixture.gateway(MARKER, 3001);
    let mut second = fixture.gateway(MARKER, 3002);
    for session in [&mut first, &mut second] {
        assert!(send(session, &switch_job(BRAWLER), 300).state_changed);
        send(session, &chat::local_request("realm level 20"), 350);
    }
    let first_before = economy(&first, &prizes);
    let second_before = economy(&second, &prizes);
    let first_next = next_instance(&first);
    let second_next = next_instance(&second);

    let first_screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let second_screen = open_battle(&mut second, MARKER_GUID, row, 500);
    assert_eq!(i32_at(&first_screen, 8), first_next);
    assert_eq!(i32_at(&second_screen, 8), second_next);
    enter_battle(&mut first, &first_screen, row, 600);
    enter_battle(&mut second, &second_screen, row, 600);
    assert_eq!(next_instance(&first), first_next + 1);
    assert_eq!(next_instance(&second), second_next + 1);

    let second_active = second
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .expect("the second run is live");
    assert_eq!(second_active.instance_id, second_next);
    assert_eq!(second_active.enemies.len(), 5);

    assert!(send(&mut first, &chat::local_request("realm win"), 800).state_changed);
    let first_paid = economy(&first, &prizes);
    assert_payout(&first_before, &first_paid, row, &prizes);
    assert_eq!(
        economy(&second, &prizes),
        second_before,
        "the first player's win never pays the second"
    );
    assert_eq!(
        second
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active,
        Some(second_active),
        "the first player's win never removes or damages the second's actors"
    );

    assert!(send(&mut second, &chat::local_request("realm win"), 900).state_changed);
    let second_paid = economy(&second, &prizes);
    assert_payout(&second_before, &second_paid, row, &prizes);
    assert_eq!(
        economy(&first, &prizes),
        first_paid,
        "the second player's win never pays the first"
    );
}

/// A mid-fight save/reload drops the run and returns to the marker without a
/// payout; a won save/reload keeps the coins and the prize and cannot pay the
/// old win again.
#[test]
fn the_win_and_the_mid_fight_return_survive_a_reload() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);

    let mut gateway = fixture.gateway(MARKER, 4001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let mut dropped = reopen(&fixture, &gateway, "mid-run.json", 2_000);
    assert_eq!(dropped.state().zone_id, 0);
    assert_eq!(dropped.state().position, MARKER);
    assert_eq!(economy(&dropped, &prizes), before);
    assert!(
        dropped
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none(),
        "a reload drops the abandoned run"
    );
    let screen = open_battle(&mut dropped, MARKER_GUID, row, 2_500);
    let restarted = send(&mut dropped, &go(&screen), 2_600);
    assert!(
        restarted.began_zoning,
        "the dropped run leaves no stuck busy state"
    );

    let mut gateway = fixture.gateway(MARKER, 4002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let won = send(&mut gateway, &chat::local_request("realm win"), 800);
    assert!(won.state_changed);
    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row, &prizes);
    let mut dropped = reopen(&fixture, &gateway, "won-unclosed.json", 2_000);
    assert_eq!(economy(&dropped, &prizes), paid);
    assert_eq!(dropped.state().zone_id, 0);
    assert_eq!(dropped.state().position, MARKER);
    assert!(
        dropped
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none()
    );
    let before_replay = saved_bytes(&dropped);
    let replay = send(&mut dropped, &chat::local_request("realm win"), 2_100);
    assert!(!replay.state_changed, "a reloaded win pays nothing");
    assert_eq!(saved_bytes(&dropped), before_replay);
}

/// Every previewed prize resolves against the shipped catalogue, fits one
/// stack, and the real loader accepts the row's bounds.
#[test]
fn every_previewed_prize_resolves_for_the_shipped_catalogue() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let inventory = runtime_inventory();
    let prizes = row["prizes"].as_array().unwrap();
    assert_eq!(prizes.len(), 4);
    assert_eq!(
        prize_ids(row),
        PRIZE_IDS.into_iter().collect::<BTreeSet<_>>()
    );
    for prize in prizes {
        let item = i32_field(prize, "item_id");
        let quantity = prize["quantity"].as_u64().unwrap_or(1) as u32;
        assert!(i32_field(prize, "name_id") > 0);
        assert!(i32_field(prize, "icon_id") > 0);
        assert!(
            inventory.accepts_reward(item, quantity),
            "prize {item} cannot be granted"
        );
    }
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the real loader accepts the new row");
    assert!(
        engine.battle_prizes_available(BANDIT_HIDEOUT, |item, quantity| {
            inventory.accepts_reward(item, quantity)
        }),
        "T243's whole-bundle preflight accepts the four prizes"
    );
    assert_eq!(
        engine.battle_spawners().len(),
        file["battles"].as_array().unwrap().len(),
        "every published battle row supplies its marker spawner"
    );
}

/// The new GUIDs collide with no battle, field actor or encounter actor, and
/// the loader refuses a duplicate GUID or a duplicate id.
#[test]
fn guids_are_unique_across_the_shipped_content() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let mut own = BTreeSet::new();
    for actor in std::iter::once(&row["spawner"]).chain(row["enemies"].as_array().unwrap().iter()) {
        let guid = u64_field(actor, "guid");
        assert!(guid > 1);
        assert!(own.insert(guid), "the new row repeats GUID {guid}");
    }
    let mut all = own.clone();
    for other in file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|other| other["id"] != json!(BANDIT_HIDEOUT))
    {
        for actor in
            std::iter::once(&other["spawner"]).chain(other["enemies"].as_array().unwrap().iter())
        {
            assert!(
                all.insert(u64_field(actor, "guid")),
                "a shipped battle repeats GUID {}",
                u64_field(actor, "guid")
            );
        }
    }
    let field = read_json(&content_dir().join("field_enemies.json"));
    for enemy in field["enemies"].as_array().unwrap() {
        let guid = u64_field(enemy, "guid");
        assert!(
            !own.contains(&guid),
            "field enemy {guid} reuses a Bandit Hideout GUID"
        );
    }
    let encounters = read_json(&content_dir().join("encounters.json"));
    for dungeon in encounters["dungeons"].as_array().unwrap() {
        assert_ne!(dungeon["id"].as_u64().unwrap() as u32, BANDIT_HIDEOUT);
        for enemy in dungeon["enemies"].as_array().unwrap() {
            let guid = u64_field(enemy, "guid");
            assert!(
                !own.contains(&guid),
                "encounter actor {guid} reuses a Bandit Hideout GUID"
            );
        }
        if let Some(guide) = dungeon["guide"].as_object() {
            let guid = guide["guid"].as_u64().unwrap();
            assert!(!own.contains(&guid), "encounter guide {guid} collides");
        }
    }
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the real loader accepts the file");

    // A duplicate GUID and a duplicate row are refused at load.
    let mut collision = file.clone();
    collision["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == json!(BANDIT_HIDEOUT))
        .unwrap()["enemies"][0]["guid"] = json!(MARKER_GUID);
    let collision_path = fixture.root.join("duplicate-guid.json");
    fs::write(&collision_path, serde_json::to_vec(&collision).unwrap()).unwrap();
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    let error = engine
        .load_field_battles(&collision_path)
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate field battle GUID"), "{error}");

    let mut duplicate = file.clone();
    let copied = battle(&duplicate, BANDIT_HIDEOUT).clone();
    duplicate["battles"].as_array_mut().unwrap().push(copied);
    let duplicate_path = fixture.root.join("duplicate-row.json");
    fs::write(&duplicate_path, serde_json::to_vec(&duplicate).unwrap()).unwrap();
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    let error = engine
        .load_field_battles(&duplicate_path)
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid field battle"), "{error}");
}

/// A live mining movie refuses the marker click and the portal Play through the
/// existing busy table, and changes nothing.
#[test]
fn a_live_mining_movie_refuses_the_battle() {
    let fixture = Fixture::new();
    let mut gateway = fixture.mining_gateway(5001);
    let started = send(&mut gateway, &chat::local_request("realm mine"), 1_000);
    assert!(
        chat_text(&started).contains("Mining test started"),
        "{}",
        chat_text(&started)
    );
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .mining
            .is_some_and(|mining| mining.active.is_some()),
        "the mining movie is live"
    );

    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let click = send(&mut gateway, &interact(MARKER_GUID), 1_100);
    assert!(!click.began_zoning && !click.state_changed);
    assert!(
        chat_text(&click).contains("Close the mining game"),
        "{}",
        chat_text(&click)
    );
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);

    let play = send(&mut gateway, &portal_play(BANDIT_HIDEOUT), 1_200);
    assert!(!play.began_zoning && !play.state_changed);
    assert!(
        chat_text(&play).contains("Finish what you are doing before starting an activity."),
        "{}",
        chat_text(&play)
    );
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none(),
        "the refused battle never starts"
    );
}

/// A second independent session sees the Bandit Hideout marker in zone 0 and
/// never the arena actors; a late joiner sees the same marker.
#[test]
fn a_second_session_sees_the_marker_not_the_arena() {
    let fixture = Fixture::new();
    let (mut first, first_entry) = fixture.session(MARKER, 6001);
    let (mut second, second_entry) = fixture.session(MARKER, 6002);
    let first_packet = marker_packet(&first_entry.replies, MARKER_GUID);
    let second_packet = marker_packet(&second_entry.replies, MARKER_GUID);
    assert_eq!(
        first_packet, second_packet,
        "the marker is identical for both sessions (no per-player actor)"
    );
    let spawn = marker_spawn(first_packet);
    assert_eq!(spawn.actor_id, MARKER_GUID);
    assert_eq!(spawn.name_string_id, MUGGENSTOMP_NAME);
    assert_eq!(spawn.model_id, BOSS_MODEL);
    assert_eq!(spawn.position[0], MARKER[0]);
    assert_eq!(spawn.position[1], MARKER[1]);
    assert_eq!(spawn.position[2], MARKER[2]);
    assert_eq!(spawn.nameplate_image_id, NAMEPLATE);
    for guid in ENEMY_GUIDS {
        assert!(
            !has_add_npc(&first_entry, guid) && !has_add_npc(&second_entry, guid),
            "arena enemy {guid} is not sent at world entry"
        );
    }

    assert!(send(&mut first, &switch_job(BRAWLER), 300).state_changed);
    send(&mut first, &chat::local_request("realm level 20"), 350);
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let entered = send(&mut first, &go(&screen), 600);
    assert!(entered.began_zoning);
    let arena = arena_spawned(&mut first, 700);
    for guid in ENEMY_GUIDS {
        assert!(
            has_add_npc(&arena, guid),
            "the owner sees arena enemy {guid}"
        );
    }

    // The second session's world is unchanged by the first player's arena:
    // its refresh carries only deltas, so it has no arena AddNpc.
    let again = arena_spawned(&mut second, 700);
    for guid in ENEMY_GUIDS {
        assert!(
            !has_add_npc(&again, guid),
            "the first player's arena enemy {guid} is not aliased into the second stream"
        );
    }
    assert_eq!(
        marker_packet(&second_entry.replies, MARKER_GUID),
        first_packet
    );

    // A late joiner gets the same static marker and no arena actors.
    let (_late, late_entry) = fixture.session(MARKER, 6003);
    assert_eq!(
        marker_packet(&late_entry.replies, MARKER_GUID),
        first_packet
    );
    for guid in ENEMY_GUIDS {
        assert!(!has_add_npc(&late_entry, guid));
    }
}

/// The win pays from the live bag, coins and profiles: a dev grant between Go
/// and the killing blow survives the payout, which adds its own 60 coins and
/// one prize.
#[test]
fn a_live_change_before_the_close_survives_the_payout() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);
    let filler = filler_definition();
    assert!(!prizes.contains(&filler));
    let mut gateway = fixture.gateway(MARKER, 7001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &chat::local_request("realm level 20"), 350);
    let before = economy(&gateway, &prizes);
    let filler_before = count_definition(&gateway, filler);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let coins = send(&mut gateway, &chat::local_request("realm coins 7"), 700);
    assert!(
        chat_text(&coins).contains("Gave 7 coins"),
        "{}",
        chat_text(&coins)
    );
    let given = send(
        &mut gateway,
        &chat::local_request(&format!("realm give {filler} 2")),
        800,
    );
    assert!(
        chat_text(&given).contains("Gave 2 x item"),
        "{}",
        chat_text(&given)
    );
    assert_eq!(count_definition(&gateway, filler), filler_before + 2);

    let mut now = 1_000;
    let replies = fight_to_win(&mut gateway, &mut now);
    assert_eq!(results_lifecycle(&replies), vec![18, 47, 19]);
    let paid = economy(&gateway, &prizes);
    assert_eq!(
        paid.coins,
        before.coins + 7 + u32_field(row, "reward_coins"),
        "the payout starts from the live wallet"
    );
    assert_eq!(
        count_definition(&gateway, filler),
        filler_before + 2,
        "the live item grant is not restored over"
    );
    assert_eq!(
        paid.experience.get(&BRAWLER).copied().unwrap_or(0),
        before.experience.get(&BRAWLER).copied().unwrap_or(0) + BATTLE_JOB_XP,
        "the profile is paid live"
    );
    let gains: Vec<(i32, u32)> = paid
        .prizes
        .iter()
        .filter_map(|(id, count)| {
            let prior = before.prizes.get(id).copied().unwrap_or(0);
            (count > &prior).then_some((*id, count - prior))
        })
        .collect();
    assert_eq!(gains.len(), 1, "exactly one prize is added to the live bag");

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 200);
    assert!(closed.began_zoning);
    assert_eq!(economy(&gateway, &prizes), paid);
    assert_eq!(count_definition(&gateway, filler), filler_before + 2);
}

/// A refused start and an unrepresentable payout in one session leave the
/// other session running; neither error escapes the gateway.
#[test]
fn a_failing_start_or_payout_does_not_stop_the_other_session() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    let prizes = prize_ids(row);

    // Session A: the full-bag preflight refuses the start.
    let (mut failing, _) = fixture.gateway_with_bag(MARKER, 8001, MAX_ITEMS);
    let before = (failing.snapshot(), saved_bytes(&failing));
    let refused = send(&mut failing, &portal_play(BANDIT_HIDEOUT), 600);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(chat_text(&refused).contains(REFUSAL_PRIZE));
    assert_eq!((failing.snapshot(), saved_bytes(&failing)), before);

    // Session B runs and is paid normally.
    let mut healthy = fixture.gateway(MARKER, 8002);
    assert!(send(&mut healthy, &switch_job(BRAWLER), 300).state_changed);
    send(&mut healthy, &chat::local_request("realm level 20"), 350);
    let healthy_before = economy(&healthy, &prizes);
    let started = send(&mut healthy, &portal_play(BANDIT_HIDEOUT), 600);
    assert!(started.began_zoning);
    arena_spawned(&mut healthy, 650);
    assert!(send(&mut healthy, &chat::local_request("realm win"), 700).state_changed);
    let healthy_paid = economy(&healthy, &prizes);
    assert_payout(&healthy_before, &healthy_paid, row, &prizes);

    // Session C cannot represent the payout: the win refuses the reward alone.
    let mut capped = fixture.gateway_with_coins(MARKER, 8003, i32::MAX as u32);
    let capped_before = economy(&capped, &prizes);
    let started = send(&mut capped, &portal_play(BANDIT_HIDEOUT), 600);
    assert!(started.began_zoning);
    arena_spawned(&mut capped, 650);
    let won = send(&mut capped, &chat::local_request("realm win"), 700);
    assert!(won.state_changed);
    assert!(
        chat_text(&won).contains(REWARD_REFUSED),
        "{}",
        chat_text(&won)
    );
    assert_eq!(
        economy(&capped, &prizes),
        capped_before,
        "an unrepresentable payout leaves the capped session's economy byte-equal"
    );
    let closed = send(&mut capped, &[26, 0, 42, 0], 800);
    assert!(closed.began_zoning);
    assert_eq!(capped.state().zone_id, 0);
    // The client sends the scene-ready pair after every zoning reply.
    arena_spawned(&mut capped, 900);

    // Neither failing session disturbed the healthy one, and all still answer.
    assert_eq!(economy(&healthy, &prizes), healthy_paid);
    for (name, session) in [
        ("failing", &mut failing),
        ("healthy", &mut healthy),
        ("capped", &mut capped),
    ] {
        let refreshed = send(session, &[167, 0, 1, 4], 1_500);
        let list = refreshed
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .find(|packet| packet.starts_with(&[167, 0, 1, 1]));
        let rows = list.map(decode_list).unwrap_or_default();
        assert!(
            rows.iter().any(|row| row.id == BANDIT_HIDEOUT),
            "{name} did not answer the portal refresh: {:?}",
            refreshed
                .replies
                .iter()
                .map(|packet| {
                    let packet = unframe(packet);
                    packet.get(..4).unwrap_or_default().to_vec()
                })
                .collect::<Vec<_>>()
        );
    }
}

/// The row's identity, roster and zone match the recovered client text and
/// models, and the marker is on the sampled ground, clear of every spawn and
/// other marker.
#[test]
fn the_marker_is_on_the_ground_and_clear() {
    let file = battles();
    let row = battle(&file, BANDIT_HIDEOUT);
    assert_eq!(u32_field(row, "title_id"), TITLE as u32);
    assert_eq!(u32_field(row, "description_id"), DESCRIPTION as u32);
    assert_eq!(u32_field(row, "objective_id"), OBJECTIVE as u32);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    assert_eq!(u32_field(row, "required_combat_level"), REQUIRED_LEVEL);
    assert_eq!(u32_field(row, "reward_coins"), REWARD_COINS);
    assert_eq!(i32_field(row, "icon_id"), BATTLE_ICON);
    assert_eq!(i32_field(row, "nameplate_image_id"), NAMEPLATE);
    let marker = position(&row["spawner"]["position"]);
    assert_eq!(marker, MARKER);
    assert_eq!(i32_field(&row["spawner"], "name_id"), MUGGENSTOMP_NAME);
    assert_eq!(i32_field(&row["spawner"], "model_id"), BOSS_MODEL);
    assert_eq!(row["spawner"]["boss"], false);
    assert_eq!(u32_field(&row["spawner"], "max_health"), 1);
    assert_eq!(u32_field(&row["spawner"], "damage"), 0);
    let lua_dist = (marker[0] - LUGABOW[0]).hypot(marker[2] - LUGABOW[2]);
    assert!(
        (15.0..=40.0).contains(&lua_dist),
        "the marker is {lua_dist:.2} m from Lugabow, on the camp approach"
    );
    assert!(lua_dist < (marker[0] - GLUGMORT[0]).hypot(marker[2] - GLUGMORT[2]));

    let enemies = row["enemies"].as_array().unwrap();
    assert_eq!(enemies.len(), 5);
    let boss = &enemies[0];
    assert_eq!(u32_field(boss, "max_health"), BOSS_BASE_HEALTH);
    assert_eq!(u32_field(boss, "damage"), BOSS_BASE_DAMAGE);
    assert_eq!(boss["boss"], true);
    assert_eq!(i32_field(boss, "model_id"), BOSS_MODEL);
    assert_eq!(i32_field(boss, "name_id"), MUGGENSTOMP_NAME);
    assert_eq!(position(&boss["position"]), [136.0, -0.6, 175.0]);
    for (index, enemy) in enemies[1..].iter().enumerate() {
        assert_eq!(u32_field(enemy, "max_health"), BANDIT_BASE_HEALTH);
        assert_eq!(u32_field(enemy, "damage"), BANDIT_BASE_DAMAGE);
        assert_eq!(enemy["boss"], false);
        assert_eq!(i32_field(enemy, "model_id"), BANDIT_MODEL);
        assert_eq!(i32_field(enemy, "name_id"), BIG_BANDIT_NAME);
        assert_eq!(u64_field(enemy, "guid"), 100001236002 + index as u64);
    }
    assert_eq!(
        enemies
            .iter()
            .map(|enemy| position(&enemy["position"]))
            .collect::<Vec<_>>(),
        [
            [136.0, -0.6, 175.0],
            [133.0, -0.6, 173.0],
            [139.0, -0.6, 173.0],
            [134.0, -0.6, 176.0],
            [138.0, -0.6, 176.0],
        ]
    );

    let zone = &row["zone"];
    assert_eq!(u32_field(zone, "zone_id"), ARENA_ZONE);
    assert_eq!(zone["name"], "sg_random_encounter_clearing");
    assert_eq!(zone["sky"], "sky_wilds24.xml");
    assert_eq!(u32_field(zone, "geometry_id"), 214);
    assert_eq!(position(&zone["position"]), ARENA_ARRIVAL);
    assert_eq!(position(&zone["rotation"]), [0.0, 0.0, 0.0]);
    assert_eq!(zone["tutorial"], false);

    // The full 5 m wander disc is standable under the T103 authoring rule.
    let terrain = terrain();
    let grid = terrain.grid(0).expect("zone 0 is loaded");
    let ground = grid
        .height_at(marker[0], marker[2])
        .expect("the marker is on covered ground");
    assert!(
        (marker[1] - ground).abs() <= 0.05,
        "marker y {} is {:.4} m from the sampled ground {ground}",
        marker[1],
        (marker[1] - ground).abs()
    );
    for angle in (0..360).step_by(15) {
        let radians = (angle as f32).to_radians();
        let sample = grid
            .height_at(
                marker[0] + 5.0 * radians.cos(),
                marker[2] + 5.0 * radians.sin(),
            )
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
    let (npc, npc_id) = nearest_spawned_npc(marker);
    assert!(
        npc >= 15.0,
        "nearest spawned NPC {npc_id} is {npc:.2} m away"
    );
    let enemy = nearest_field_enemy(marker);
    assert!(enemy >= 20.0, "nearest field enemy is {enemy:.1} m away");
    let other = nearest_other_battle_marker(marker);
    assert!(
        other >= 100.0,
        "the nearest battle marker is {other:.1} m away, so the marker stays well clear of Thugamug"
    );

    let arena = terrain.grid(ARENA_ZONE).expect("zone 9600 is loaded");
    assert!(
        arena
            .height_at(ARENA_ARRIVAL[0], ARENA_ARRIVAL[2])
            .is_some(),
        "the arena arrival is inside the zone-9600 grid"
    );

    // Client evidence: locale rows, both bandit models and the two images.
    let catalogued = read_json(&lab("reports/locale-id-to-text.json"));
    let names: BTreeMap<i32, String> = catalogued
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
        names.get(&TITLE).map(String::as_str),
        Some("Bandit Hideout!")
    );
    assert_eq!(
        names.get(&DESCRIPTION).map(String::as_str),
        Some(
            "An unruly band of thugawugs are inside the Bandit Hideout! Defeat their leader, Muggenstomp, to end their greedy ways!"
        )
    );
    assert_eq!(
        names.get(&OBJECTIVE).map(String::as_str),
        Some("Defeat Muggenstomp!")
    );
    assert_eq!(
        names.get(&MUGGENSTOMP_NAME).map(String::as_str),
        Some("Muggenstomp")
    );
    assert_eq!(
        names.get(&BIG_BANDIT_NAME).map(String::as_str),
        Some("Big Bandit")
    );
    let models =
        fs::read_to_string(lab("working/OSFR.Client/OSFR Client/Resources/Models.txt")).unwrap();
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("200^chugawug_f_bandit_01.adr^")),
        "model 200 is the female bandit body"
    );
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("199^chugawug_m_bandit_01.adr^")),
        "model 199 is the male bandit body"
    );
    let images = fs::read_to_string(lab(
        "working/OSFR.Client/OSFR Client/Resources/Images/Images.txt",
    ))
    .unwrap();
    assert!(
        images.lines().any(|line| line
            .starts_with("4543^icon_ui_combatstart_panelgraphic_trollsummonermadness.dds^")),
        "the start-panel graphic is a client image"
    );
    assert!(
        images
            .lines()
            .any(|line| line
                .starts_with("11970^icon_minigame_combat_encounter_start_hard_free_64.dds^")),
        "the shared battle marker icon is a client image"
    );
    let catalog = fs::read_to_string(lab("extracted/client-assets/catalog.tsv")).unwrap();
    assert!(catalog.contains("\tchugawug_f_bandit_01.adr\t"));
    assert!(catalog.contains("\tchugawug_m_bandit_01.adr\t"));
}

/// Every earlier row keeps its title, difficulty, required level and payout;
/// the new row is the only addition and sits after battle 12400.
#[test]
fn the_shipped_rows_are_unchanged_and_the_new_row_is_appended() {
    let file = battles();
    let expected: [(u32, u32, u32, u32, u32); 9] = [
        (96, 42056, 1, 0, 25),
        (12340, 90703, 3, 5, 25),
        (12301, 75452, 1, 0, 25),
        (12302, 42054, 2, 0, 25),
        (12303, 426971, 5, 13, 0),
        (12350, 30404, 1, 0, 25),
        (12310, 93274, 2, 0, 25),
        (12311, 141039, 2, 0, 25),
        (12400, 24276, 1, 0, 19),
    ];
    let ids: Vec<u32> = file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap() as u32)
        .collect();
    // Concurrent content slices insert their own rows (the Bixie Hive slice
    // after battle 96), so pin every shipped row's values and relative order
    // plus this row's place after 12400, not the whole file.
    let mut previous = None;
    for (id, title, difficulty, required, coins) in expected {
        let row = battle(&file, id);
        assert_eq!(u32_field(row, "title_id"), title, "battle {id} title");
        assert_eq!(u32_field(row, "difficulty"), difficulty, "battle {id}");
        assert_eq!(
            u32_field(row, "required_combat_level"),
            required,
            "battle {id}"
        );
        assert_eq!(u32_field(row, "reward_coins"), coins, "battle {id}");
        let at = ids
            .iter()
            .position(|candidate| *candidate == id)
            .unwrap_or_else(|| panic!("battle {id} was removed"));
        if let Some(previous) = previous {
            assert!(previous < at, "battle {id} was re-ordered");
        }
        previous = Some(at);
    }
    let existing_last = previous.expect("the shipped rows are non-empty");
    let new_at = ids
        .iter()
        .position(|id| *id == BANDIT_HIDEOUT)
        .expect("the new row is published");
    assert_eq!(
        ids.iter().filter(|id| **id == BANDIT_HIDEOUT).count(),
        1,
        "the new row is published exactly once"
    );
    assert!(
        new_at > existing_last,
        "the new row is appended after battle 12400"
    );
    assert!(!TASK_ROSTER.contains(&BANDIT_HIDEOUT));
    assert_eq!(ids.iter().filter(|id| TASK_ROSTER.contains(id)).count(), 5);

    // No dungeon data file is added: the runtime row is the playable backing.
    // Later content slices may add their own dungeon files, so only pin that
    // this slice removed none; the new row itself is not an encounters.json id.
    let dungeon_files = fs::read_dir(content_dir().join("dungeons"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "json")
        })
        .count();
    assert!(
        dungeon_files >= 75,
        "a pre-existing dungeon data file was removed"
    );
    let encounters = read_json(&content_dir().join("encounters.json"));
    assert!(
        !encounters["dungeons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|dungeon| dungeon["id"].as_u64() == Some(u64::from(BANDIT_HIDEOUT)))
    );
}
