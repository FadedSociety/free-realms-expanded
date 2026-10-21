//! T319: Sheep Watch as a playable level-1 field battle at Farnum Farmstead.
//!
//! Content-only slice: one new `field_battles.json` row and the existing route
//! (167/1/2 Play or marker 26/8 -> 41/114 start screen -> 41/108 Go -> arena
//! zone 9600 -> 36/10 attacks -> results -> 26/42 Close). No new wire field,
//! handler, opcode, selector or engine code is added.
//!
//! Client shapes checked in the supplied 2014 client:
//! * 41/108 entrance constructor `0x008b6e70` (Go).
//! * 26/42 results Close constructor `0x008a2eb0`, sender `0x0092f300`.
//! * 36/10 basic attack writer `0x00a342e0` (and its reader `0x00a35150`):
//!   bar 1, slot 0, target mode 0, target GUID.
//! * 41/114 start-screen reader chain `0x00aa3430` -> `0x00aa32d0` ->
//!   `0x00a29120` -> `0x009bdd70`; this test checks the title/icon/
//!   description/difficulty fields. The win results are `present_results`
//!   (39/18 game over, 39/47 scores, 39/19 UI), already pinned by
//!   `tests/encounter.rs`; 41/114 is the start/running-encounter screen, not
//!   the results screen (the brief's "41/114 results" wording is corrected
//!   here).
//! * 35/2 AddNpc client reader `0x00921a10` reads the u64 actor id, name id,
//!   model id and transform this test asserts.
//! * 167/1/1 list reader `0x00be79d0` -> `0x00be75c0` -> `0x00be6fa0` ->
//!   `0x00be63e0`; the row layout is the one pinned by `activity_portal.rs`.
//!
//! Placement evidence: the guide's Farnum Farmstead entrance and the video
//! marker 19 m from Pappy. The marker uses the visible Alpha Wolf model (R),
//! like T282's Thugamug marker, at a terrain-sampled point 19.00 m east of
//! Pappy. Its 105-sample 5 m wander disc is terrain-covered and at most 19.96
//! degrees; the nearest other spawned NPC is 17.38 m away (the 5 m disc stays
//! outside the 12 m NPC interaction radius), and the nearest field enemy and
//! other battle marker are over 200 m away. Water, object collision and a
//! native walkable route remain unverified.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::{EncounterEngine, EncounterPhase},
    game_store,
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState, MAX_ITEMS},
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

/// The new row. 12350 is free on this base and sits after Stone Heart (12303).
const SHEEP_WATCH: u32 = 12350;
const TITLE: i32 = 30404;
const DESCRIPTION: i32 = 92659;
const OBJECTIVE: i32 = 388319;
const DIFFICULTY: u32 = 1;
const REWARD_COINS: u32 = 25;
const MARKER_GUID: u64 = 100001235000;
const ENEMY_GUIDS: [u64; 5] = [
    100001235001,
    100001235002,
    100001235003,
    100001235004,
    100001235005,
];
/// `-32.96` is the bilinear zone-0 height `-32.9577` rounded.
const MARKER: [f32; 3] = [-2165.513, -32.96, 328.1112];
const PAPPY: [f32; 3] = [-2184.513, -31.53728, 328.1112];
const ALPHA_NAME: i32 = 102128;
const WOLF_NAME: i32 = 374;
const MODEL: i32 = 3284;
const NAMEPLATE: i32 = 11970;
const BRAWLER: i32 = 43;
const BATTLE_JOB_XP: u32 = 50;
const ARENA_ZONE: u32 = 9600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
/// The designated five-battle roster from `field_battle_roster`; the new row
/// is deliberately not part of it.
const TASK_ROSTER: [u32; 5] = [96, 12301, 12302, 12310, 12311];
const RUNTIME_CONTENT: &str = "../../../lab/reports/round70-native-20261016T061000Z/content";
const REFERENCE: &str = "../../../lab/references/rust-gateway-reference";
const REFUSAL_PRIZE: &str = "one of its rewards cannot be granted";

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
            "sheep-watch-{}-{}",
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
        assert_eq!(level(&seed, BRAWLER), 1);

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

fn active_profile(gateway: &Gateway) -> i32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .profiles
        .unwrap()
        .active
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
    assert_eq!(i32_at(details, 4), SHEEP_WATCH as i32);
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
        &details[47..71],
        expected.as_slice(),
        "41/114 fields must agree byte-for-byte with the client reader"
    );
    details.to_vec()
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
        "the verified goal 388319 reaches the client"
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
    for (actor, state) in row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .zip(&active.enemies)
    {
        assert_eq!(state.guid, u64_field(actor, "guid"));
        assert_eq!(state.position, position(&actor["position"]));
        assert_eq!(state.health, u32_field(actor, "max_health"));
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
/// 39/47 (scores) and 39/19 (results UI). 39/45 is the loot wheel and is not
/// part of this assertion.
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
    // The typed snapshot payload the save envelope writes; the full-bag
    // refusal fixture is larger than game_store's 1 MiB document limit, so
    // compare the payload directly (as T282 does).
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
        let prizes = prize_ids(battle(&battles(), SHEEP_WATCH));
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
    let prizes = prize_ids(battle(&battles(), SHEEP_WATCH));
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
        .filter(|row| row["id"].as_u64() != Some(u64::from(SHEEP_WATCH)))
        .map(|row| {
            let point = position(&row["spawner"]["position"]);
            (point[0] - marker[0]).hypot(point[2] - marker[2])
        })
        .fold(f32::INFINITY, f32::min)
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// 1. The row publishes a drawable 35/2 Alpha Wolf marker at the sampled
/// Farnum point, a second independent session receives the identical actor,
/// and the portal lists Sheep Watch once with `can_join` set for a level-1
/// combat job.
#[test]
fn the_marker_is_drawable_and_the_portal_publishes_sheep_watch() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    assert_eq!(u32_field(row, "title_id"), TITLE as u32);
    assert_eq!(u32_field(row, "description_id"), DESCRIPTION as u32);
    assert_eq!(u32_field(row, "objective_id"), OBJECTIVE as u32);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    assert_eq!(u32_field(row, "required_combat_level"), 0);
    assert_eq!(u32_field(row, "reward_coins"), REWARD_COINS);
    assert_eq!(i32_field(row, "icon_id"), -1);
    assert_eq!(i32_field(row, "nameplate_image_id"), NAMEPLATE);
    assert_eq!(row["spawner"]["boss"], false);
    assert_eq!(u32_field(&row["spawner"], "max_health"), 1);
    assert_eq!(u32_field(&row["spawner"], "damage"), 0);

    let marker = position(&row["spawner"]["position"]);
    assert_eq!(marker, MARKER);
    let pappy_distance = (marker[0] - PAPPY[0]).hypot(marker[2] - PAPPY[2]);
    assert!(
        (8.0..=25.0).contains(&pappy_distance),
        "the marker is {pappy_distance:.2} m from Pappy Farnum"
    );
    assert_eq!(u32_field(&row["zone"], "zone_id"), ARENA_ZONE);
    assert_eq!(row["zone"]["name"], "sg_random_encounter_clearing");
    assert_eq!(row["zone"]["sky"], "sky_wilds24.xml");
    assert_eq!(u32_field(&row["zone"], "geometry_id"), 214);
    assert_eq!(position(&row["zone"]["position"]), ARENA_ARRIVAL);

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
    // Clear of other click targets: the 5 m disc stays outside the 12 m NPC
    // interaction radius, and the field-enemy/marker gap follows T103.
    let (npc, npc_id) = nearest_spawned_npc(marker);
    assert!(
        npc - 5.0 >= 12.0,
        "nearest spawned NPC {npc_id} is {npc:.2} m away"
    );
    let enemy = nearest_field_enemy(marker);
    assert!(enemy >= 25.0, "nearest field enemy is {enemy:.1} m away");
    let other = nearest_other_battle_marker(marker);
    assert!(other >= 25.0, "nearest battle marker is {other:.1} m away");

    // Two independent sessions each receive the same static marker.
    let (mut first, first_entry) = fixture.session(MARKER, 1001);
    let (_second, second_entry) = fixture.session(MARKER, 1002);
    let first_packet = marker_packet(&first_entry.replies, MARKER_GUID);
    let second_packet = marker_packet(&second_entry.replies, MARKER_GUID);
    assert_eq!(
        first_packet, second_packet,
        "the marker is identical for both sessions (no per-player actor)"
    );
    let spawn = marker_spawn(first_packet);
    assert_eq!(spawn.actor_id, MARKER_GUID);
    assert_eq!(spawn.name_string_id, ALPHA_NAME);
    assert_eq!(spawn.model_id, MODEL);
    assert_eq!(spawn.position[0], MARKER[0]);
    assert_eq!(spawn.position[1], MARKER[1]);
    assert_eq!(spawn.position[2], MARKER[2]);
    assert_eq!(spawn.nameplate_image_id, NAMEPLATE);
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
        names.get(&ALPHA_NAME).map(String::as_str),
        Some("Alpha Wolf")
    );
    assert_eq!(names.get(&WOLF_NAME).map(String::as_str), Some("Wolf"));
    assert_eq!(names.get(&TITLE).map(String::as_str), Some("Sheep Watch"));
    assert_eq!(
        names.get(&DESCRIPTION).map(String::as_str),
        Some(
            "A pack of wolves is attacking the sheep at Farnum Farmstead. Save the sheep by defeating the wolves!"
        )
    );
    assert_eq!(
        names.get(&OBJECTIVE).map(String::as_str),
        Some("Defeat the Ravenous Wolves at Sheep Watch")
    );
    let models =
        fs::read_to_string(lab("working/OSFR.Client/OSFR Client/Resources/Models.txt")).unwrap();
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("3284^wolf_generic.adr^")),
        "model 3284 is in Models.txt"
    );
    let catalog = fs::read_to_string(lab("extracted/client-assets/catalog.tsv")).unwrap();
    assert!(
        catalog.contains("\twolf_generic.adr\t"),
        "model 3284 is present in catalog.tsv"
    );

    // At world entry the default Adventurer job is not a combat job; the
    // production job switch plus 167/1/4 refresh flips CanPlayerJoin.
    let entry_rows = published_list(&first_entry.replies);
    let entry_row = entry_rows
        .iter()
        .find(|row| row.id == SHEEP_WATCH)
        .expect("Sheep Watch is in the 167/1/1 list");
    assert_eq!(
        entry_rows
            .iter()
            .filter(|row| row.id == SHEEP_WATCH)
            .count(),
        1
    );
    assert_eq!(entry_row.title, TITLE as u32);
    assert_eq!(entry_row.description, DESCRIPTION as u32);
    assert_eq!(entry_row.difficulty, DIFFICULTY);
    assert!(!entry_row.can_join, "a non-combat job cannot join");

    assert!(send(&mut first, &switch_job(BRAWLER), 300).state_changed);
    assert_eq!(level(&first, BRAWLER), 1);
    let refreshed = send(&mut first, &[167, 0, 1, 4], 400);
    let rows = published_list(&refreshed.replies);
    let published = rows
        .iter()
        .find(|row| row.id == SHEEP_WATCH)
        .expect("the refreshed list still contains Sheep Watch");
    assert!(published.can_join, "a level-1 combat job may join");
    assert_eq!(published.title, TITLE as u32);
    assert_eq!(published.description, DESCRIPTION as u32);
    assert_eq!(published.difficulty, DIFFICULTY);
}

/// 2. A level-1 combat job starts from the marker, five enemies spawn in zone
/// 9600, the live run refuses another portal Play through the shared busy gate,
/// ordinary 36/10 attacks win, the results packets open, Close returns to the
/// marker, and 25 coins, one prize and 50 job XP (active Brawler only) are paid
/// exactly once.
#[test]
fn a_level_one_character_starts_wins_and_returns() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(MARKER, 2001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    assert_eq!(active_profile(&gateway), BRAWLER);
    assert_eq!(level(&gateway, BRAWLER), 1);

    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    assert_eq!(enemy_guids, ENEMY_GUIDS);
    // The same activity is refused while its own run is live (the shared busy
    // gate), without touching the arena state.
    let busy = send(&mut gateway, &portal_play(SHEEP_WATCH), 1_900);
    assert!(!busy.began_zoning && !busy.state_changed);
    assert!(
        chat_text(&busy).contains("Finish what you are doing"),
        "{}",
        chat_text(&busy)
    );
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .map(|active| active.phase),
        Some(EncounterPhase::Fighting)
    );
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
    assert_eq!(active_profile(&gateway), BRAWLER);
    assert_eq!(
        paid.experience.get(&BRAWLER).copied().unwrap_or(0),
        before.experience.get(&BRAWLER).copied().unwrap_or(0) + BATTLE_JOB_XP
    );

    let won = gateway.snapshot();
    let repeated = send(&mut gateway, &melee(enemy_guids[0]), now + 700);
    assert!(!repeated.state_changed, "a won fight cannot pay twice");
    assert_eq!(gateway.snapshot(), won);

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_400);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);
    assert_eq!(
        economy(&gateway, &prizes),
        paid,
        "leaving pays nothing more"
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

/// 3. Field battles are repeatable (`guide.is_none()`): a second completed
/// run pays 25 coins, one prize and 50 job XP again, and the first run's Go,
/// enemy hit and Close replay without paying.
#[test]
fn a_second_run_pays_again_once_and_only_once() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(MARKER, 3001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = economy(&gateway, &prizes);

    let mut now = 500;
    let first_screen = open_battle(&mut gateway, MARKER_GUID, row, now);
    now += 100;
    let first_enemies = enter_battle(&mut gateway, &first_screen, row, now);
    let first_replies = fight_to_win(&mut gateway, &mut now);
    assert_eq!(results_lifecycle(&first_replies), vec![18, 47, 19]);
    let paid_once = economy(&gateway, &prizes);
    assert_payout(&before, &paid_once, row, &prizes);
    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 200);
    assert!(closed.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);
    // The client sends the scene-ready pair after every zoning reply; load the
    // world again before the second start.
    arena_spawned(&mut gateway, now + 300);

    now += 500;
    let second_screen = open_battle(&mut gateway, MARKER_GUID, row, now);
    now += 100;
    enter_battle(&mut gateway, &second_screen, row, now);
    let second_replies = fight_to_win(&mut gateway, &mut now);
    assert_eq!(results_lifecycle(&second_replies), vec![18, 47, 19]);
    let paid_twice = economy(&gateway, &prizes);
    assert_payout(&paid_once, &paid_twice, row, &prizes);
    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 200);
    assert!(closed.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    arena_spawned(&mut gateway, now + 300);

    // The first run's screen/Go, enemy GUID and results Close are inert now.
    // 41/114 is a server-to-client screen; its client-side replay is the Go
    // built from those bytes, which is what this asserts.
    let after_runs = gateway.snapshot();
    let replay_go = send(&mut gateway, &go(&first_screen), now + 500);
    assert!(!replay_go.began_zoning && !replay_go.state_changed);
    let replay_hit = send(&mut gateway, &melee(first_enemies[0]), now + 600);
    assert!(!replay_hit.state_changed);
    let replay_close = send(&mut gateway, &[26, 0, 42, 0], now + 700);
    assert!(!replay_close.began_zoning && !replay_close.state_changed);
    assert_eq!(
        gateway.snapshot(),
        after_runs,
        "first-run packets pay nothing after the second run"
    );
    assert_eq!(economy(&gateway, &prizes), paid_twice);
}

/// 4. Forged Go, a wrong marker, out-of-range Go, a non-combat job, a
/// knocked-out character and the full-bag preflight each change no typed
/// state or save bytes and leave the session alive. Controls prove the same
/// requests start under the qualifying condition.
#[test]
fn refusals_change_nothing() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);

    // Forged Go with no open start screen.
    let mut gateway = fixture.gateway(MARKER, 4001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let mut forged = vec![41, 0, 108, 0];
    forged.extend(SHEEP_WATCH.to_le_bytes());
    forged.extend(0x7fff_i32.to_le_bytes());
    forged.extend(0_u64.to_le_bytes());
    let refused = send(&mut gateway, &forged, 400);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(refused.replies.is_empty(), "a forged Go has no reply");
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "a forged Go changes no state"
    );
    // The session still answers an ordinary refresh.
    assert!(
        published_list(&send(&mut gateway, &[167, 0, 1, 4], 450).replies)
            .iter()
            .any(|row| row.id == SHEEP_WATCH)
    );

    // Wrong marker: a forged actor GUID opens no start screen.
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let wrong = send(&mut gateway, &interact(0xdead_beef), 500);
    assert!(details(&wrong).is_none());
    assert!(!wrong.state_changed && !wrong.began_zoning);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "a wrong marker changes no state"
    );

    // Out-of-range Go: the screen opens at the marker, the player walks 50 m
    // away and Go is inert. Reopening the screen at the marker then works.
    let mut gateway = fixture.gateway(MARKER, 4002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
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
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "an out-of-range Go changes no state"
    );
    walk(&mut gateway, player, MARKER, &mut now);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, now + 100);
    assert!(send(&mut gateway, &go(&screen), now + 200).began_zoning);
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
    arena_spawned(&mut gateway, now + 250);
    let instance = i32_at(&screen, 8);
    let cancel = [
        vec![41, 0, 109, 0],
        0_i32.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
    ]
    .concat();
    assert!(send(&mut gateway, &cancel, now + 300).began_zoning);
    assert_eq!(gateway.state().position, MARKER);

    // Non-combat job: the same Go is a chat refusal that reopens the screen.
    let mut civilian = fixture.gateway(MARKER, 4003);
    let screen = open_battle(&mut civilian, MARKER_GUID, row, 400);
    let before = (civilian.snapshot(), saved_bytes(&civilian));
    let refused = send(&mut civilian, &go(&screen), 500);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("Switch to a combat job before starting a battle."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        details(&refused).expect("the start screen reopens"),
        screen.as_slice()
    );
    assert_eq!(
        (civilian.snapshot(), saved_bytes(&civilian)),
        before,
        "a job refusal changes no state"
    );
    assert!(send(&mut civilian, &switch_job(BRAWLER), 600).state_changed);
    let started = send(&mut civilian, &go(&screen), 700);
    assert!(started.began_zoning, "the control job starts the battle");

    // Knocked out: Go is a refusal, not a session-ending error.
    let mut knocked = fixture.gateway(MARKER, 4004);
    assert!(send(&mut knocked, &switch_job(BRAWLER), 300).state_changed);
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
    assert_eq!(
        (knocked.snapshot(), saved_bytes(&knocked)),
        before,
        "a knocked-out Go changes no state"
    );

    // Full bag: the four-prize bundle has no room, so Go is refused.
    let (mut full, bag) = fixture.gateway_with_bag(MARKER, 4005, MAX_ITEMS);
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
    assert_eq!(
        details(&refused).expect("the start screen reopens"),
        screen.as_slice()
    );
    assert_eq!(
        (full.snapshot(), saved_bytes(&full)),
        before,
        "a full bag changes no state"
    );

    // Control: four free stacks fit the bundle and the same battle starts.
    let (mut fitting, bag) = fixture.gateway_with_bag(MARKER, 4006, MAX_ITEMS - 4);
    assert_eq!(bag.items.len(), MAX_ITEMS - 4);
    let screen = open_battle(&mut fitting, MARKER_GUID, row, 800);
    let started = send(&mut fitting, &go(&screen), 900);
    assert!(started.began_zoning, "the fitting bundle starts the battle");
    assert_eq!(fitting.state().zone_id, ARENA_ZONE);
}

/// 5. A run dropped by save/reload returns to the world without a second
/// payout or stuck busy state; two independent sessions keep their own runs.
#[test]
fn a_disconnect_mid_run_drops_the_run_without_paying() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let prizes = prize_ids(row);

    // No-win variant: a fighting run is saved and reloaded. No payout, no
    // active run, and the marker can start a fresh battle.
    let mut gateway = fixture.gateway(MARKER, 5001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
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

    // Won, unclosed variant: the payout is already applied; reloading must
    // not pay again and a replayed win must be inert.
    let mut gateway = fixture.gateway(MARKER, 5002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
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
    let before_replay = saved_bytes(&dropped);
    let replay = send(&mut dropped, &chat::local_request("realm win"), 2_100);
    assert!(!replay.state_changed, "a reloaded win pays nothing");
    assert_eq!(saved_bytes(&dropped), before_replay);
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

    // Two independent sessions: the first player's win and disconnect do not
    // change the second player's live run or save.
    let mut first = fixture.gateway(MARKER, 5101);
    let mut second = fixture.gateway(MARKER, 5102);
    for session in [&mut first, &mut second] {
        assert!(send(session, &switch_job(BRAWLER), 300).state_changed);
    }
    let first_before = economy(&first, &prizes);
    let second_before = economy(&second, &prizes);
    let first_screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let second_screen = open_battle(&mut second, MARKER_GUID, row, 500);
    enter_battle(&mut first, &first_screen, row, 600);
    enter_battle(&mut second, &second_screen, row, 600);
    let second_save = saved_bytes(&second);
    let second_active = second
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active;
    assert!(second_active.is_some());

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
        second_active,
        "the second player's run is unchanged"
    );
    let dropped = reopen(&fixture, &first, "first-dropped.json", 1_500);
    assert_eq!(economy(&dropped, &prizes), first_paid);
    assert_eq!(
        saved_bytes(&second),
        second_save,
        "the first player's disconnect never changes the second player's save"
    );
    assert_eq!(
        second
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active,
        second_active,
        "the second player's run survives the first player's disconnect"
    );
    assert!(send(&mut second, &chat::local_request("realm win"), 1_600).state_changed);
    let second_paid = economy(&second, &prizes);
    assert_payout(&second_before, &second_paid, row, &prizes);
    assert_eq!(
        economy(&dropped, &prizes),
        first_paid,
        "the second player's win never pays the first"
    );
}

/// 6. Loader and reward bounds: unique ids/GUIDs, 1..=32 enemies, positive
/// text ids, a bounded coin payout, one-prize-per-win rows and the zone-9600
/// arena; synthetic duplicate rows/GUIDs are refused by the real loader.
#[test]
fn the_row_is_inside_the_loader_and_reward_bounds() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, SHEEP_WATCH);
    let rows = file["battles"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .filter(|row| row["id"].as_u64() == Some(u64::from(SHEEP_WATCH)))
            .count(),
        1
    );
    let mut guids = BTreeSet::new();
    for actor in std::iter::once(&row["spawner"]).chain(row["enemies"].as_array().unwrap().iter()) {
        let guid = u64_field(actor, "guid");
        assert!(guid > 1);
        assert!(guids.insert(guid), "the new row repeats GUID {guid}");
    }
    for actor in rows.iter().filter(|row| row["id"] != json!(SHEEP_WATCH)) {
        for actor in
            std::iter::once(&actor["spawner"]).chain(actor["enemies"].as_array().unwrap().iter())
        {
            assert!(
                guids.insert(u64_field(actor, "guid")),
                "a shipped battle repeats GUID {}",
                u64_field(actor, "guid")
            );
        }
    }
    let enemies = row["enemies"].as_array().unwrap();
    assert!((1..=32).contains(&enemies.len()));
    assert!(i32_field(row, "title_id") > 0);
    assert!(i32_field(row, "description_id") > 0);
    assert!(i32_field(row, "objective_id") > 0);
    assert!(u32_field(row, "reward_coins") <= 1_000_000);
    let prizes = row["prizes"].as_array().unwrap();
    assert!(
        (1..=4).contains(&prizes.len()),
        "one prize is drawn per win"
    );
    assert_eq!(u32_field(&row["zone"], "zone_id"), ARENA_ZONE);

    let inventory = runtime_inventory();
    for prize in prizes {
        let item = i32_field(prize, "item_id");
        assert!(
            inventory.accepts_reward(item, 1),
            "prize {item} cannot be granted"
        );
    }
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the real loader accepts the new row");
    assert!(
        engine.battle_prizes_available(SHEEP_WATCH, |item, quantity| {
            inventory.accepts_reward(item, quantity)
        }),
        "T243's whole-bundle preflight accepts the four prizes"
    );

    // A duplicate GUID and a duplicate row are refused at load.
    let mut collision = file.clone();
    collision["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == json!(SHEEP_WATCH))
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
    let copied = battle(&duplicate, SHEEP_WATCH).clone();
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

/// 7. Every battle that existed before this slice keeps its title, difficulty,
/// required level, payout, actor GUIDs and positions; the new row is the only
/// addition, after 12303, and the 75 dungeon records still load.
#[test]
fn the_existing_battles_and_content_are_unchanged() {
    let file = battles();
    let expected: [(u32, u32, u32, u32, u32); 8] = [
        (96, 42056, 1, 0, 25),
        (12340, 90703, 3, 5, 25),
        (12301, 75452, 1, 0, 25),
        (12302, 42054, 2, 0, 25),
        (12303, 426971, 5, 13, 0),
        (12310, 93274, 2, 0, 25),
        (12311, 141039, 2, 0, 25),
        (12400, 24276, 1, 0, 19),
    ];
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
    }

    // Actor GUIDs and positions are pinned per shipped row.
    let actors: [(u32, &[(u64, [f32; 3])]); 8] = [
        (
            96,
            &[
                (100000096000, [-440.0, 0.8, 1290.0]),
                (100000096001, [136.0, -0.6, 175.0]),
                (100000096002, [133.0, -0.6, 173.0]),
                (100000096003, [139.0, -0.6, 173.0]),
                (100000096004, [133.0, -0.6, 173.0]),
                (100000096005, [139.0, -0.6, 173.0]),
                (100000096006, [133.0, -0.6, 173.0]),
                (100000096007, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12340,
            &[
                (100001234000, [-1581.39915, -45.29, -552.22302]),
                (100001234001, [136.0, -0.6, 175.0]),
                (100001234002, [133.0, -0.6, 173.0]),
                (100001234003, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12301,
            &[
                (100001230100, [-726.0, -7.1, 152.0]),
                (100001230101, [133.0, -0.6, 173.0]),
                (100001230102, [136.0, -0.6, 175.0]),
                (100001230103, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12302,
            &[
                (100001230200, [-540.0, 0.5, 1010.0]),
                (100001230201, [133.0, -0.6, 173.0]),
                (100001230202, [136.0, -0.6, 175.0]),
                (100001230203, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12303,
            &[
                (100000004750, [-950.582, -34.42554, -651.9259]),
                (100001230301, [136.0, -0.6, 175.0]),
                (100001230302, [133.0, -0.6, 173.0]),
                (100001230303, [136.0, -0.6, 173.0]),
                (100001230304, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12310,
            &[
                (100001231000, [364.0, 53.3, 572.0]),
                (100001231001, [133.0, -0.6, 173.0]),
                (100001231002, [136.0, -0.6, 175.0]),
                (100001231003, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12311,
            &[
                (100001231100, [-888.0, -33.5, -376.0]),
                (100001231101, [133.0, -0.6, 173.0]),
                (100001231102, [136.0, -0.6, 175.0]),
                (100001231103, [139.0, -0.6, 173.0]),
            ],
        ),
        (
            12400,
            &[
                (100000124000, [-1908.92576, -39.4, 410.69792]),
                (100000124001, [136.0, -0.6, 175.0]),
            ],
        ),
    ];
    for (id, actors) in actors {
        let row = battle(&file, id);
        let actual: Vec<(u64, [f32; 3])> = std::iter::once(&row["spawner"])
            .chain(row["enemies"].as_array().unwrap().iter())
            .map(|actor| (u64_field(actor, "guid"), position(&actor["position"])))
            .collect();
        assert_eq!(actual, actors, "battle {id} actors changed");
    }

    let ids: Vec<u32> = file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap() as u32)
        .collect();
    assert_eq!(ids.len(), expected.len() + 1, "only one row was added");
    assert_eq!(ids[4], 12303, "the insertion anchor is Stone Heart");
    assert_eq!(ids[5], SHEEP_WATCH, "Sheep Watch sits after battle 12303");
    assert!(!TASK_ROSTER.contains(&SHEEP_WATCH));

    let dungeon_dir = content_dir().join("dungeons");
    let dungeon_files = fs::read_dir(dungeon_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "json")
        })
        .count();
    assert_eq!(dungeon_files, 75, "74 guide pages plus opening_cave.json");
    let sheep_watch = read_json(&content_dir().join("dungeons/sheep_watch.json"));
    assert_eq!(
        sheep_watch["records"][0]["record"]["name_string_id"],
        1279835357
    );
    assert_eq!(
        sheep_watch["records"][0]["dungeon_data"]["difficulty"]["value"],
        1
    );
}
