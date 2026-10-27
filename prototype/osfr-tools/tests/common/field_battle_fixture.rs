//! Shared production-shaped fixture for the field-battle integration suites
//! (T282 Thugamug, T319 Sheep Watch).
//!
//! Include with `#[path = "common/field_battle_fixture.rs"] mod field_battle_fixture;`
//! and `use field_battle_fixture::*;`. Test-only; production never builds this
//! module. The bag helpers take the battle id because each suite fills the bag
//! around its own prize pool.
#![allow(dead_code)] // each include site uses a different subset of the helpers

use osfr_tools::{
    account_store::AccountRole,
    encounter::EncounterPhase,
    game_store,
    gameplay::GameplayState,
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState},
    quest_engine::QuestStatus,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

#[path = "combat_scale.rs"]
mod combat_scale;
#[path = "gateway_packets.rs"]
mod gateway_packets;

#[allow(unused_imports)] // Each suite uses the subset its route needs.
pub use gateway_packets::{
    body as unframe, chat, dialogue_choice, interact, melee, move_to, quest_end_reply, quest_reply,
    wrap,
};

pub const RUNTIME_CONTENT: &str = "../../../lab/reports/round70-native-20261016T061000Z/content";
pub const REFERENCE: &str = "../../../lab/references/rust-gateway-reference";

static SERIAL: AtomicU64 = AtomicU64::new(0);
static RUNTIME_INVENTORY: OnceLock<Inventory> = OnceLock::new();

// ---------------------------------------------------------------------------
// Production-shaped session fixture.
// ---------------------------------------------------------------------------

pub struct Fixture {
    pub root: PathBuf,
    pub reference: PathBuf,
    pub content: PathBuf,
    pub version: String,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Fixture {
    pub fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "field-battle-{}-{}",
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

    pub fn login(&self, gateway: &mut Gateway) -> GatewayOutcome {
        let login = gateway_packets::login_with_version(gateway.state().guid, &self.version);
        gateway.handle_at(&login, 0).unwrap()
    }

    /// A session at `position` that has logged in but not yet entered the world, with the
    /// login's own replies.
    pub fn login_session(&self, position: [f32; 3], guid: u64) -> (Gateway, GatewayOutcome) {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.guid = guid;
        character.zone_id = 0;
        character.position = position;
        character.rotation = [0.0, 0.0, 1.0];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, None).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        let login = self.login(&mut gateway);
        (gateway, login)
    }

    /// A logged-in session at `position`, including the 10/13 world-entry
    /// replies (AddNpc visibility, portal list and the encounter state).
    pub fn session(&self, position: [f32; 3], guid: u64) -> (Gateway, GatewayOutcome) {
        let (mut gateway, _) = self.login_session(position, guid);
        send(&mut gateway, &13_u16.to_le_bytes(), 100);
        let loaded = send(&mut gateway, &10_u16.to_le_bytes(), 200);
        (gateway, loaded)
    }

    pub fn gateway(&self, position: [f32; 3], guid: u64) -> Gateway {
        self.session(position, guid).0
    }

    /// Restore a save built by the production `game_store` path. `seed` runs on
    /// the seed session before its gameplay state is snapshotted (job switch and
    /// any level command); the restored world then holds the rebuilt bag.
    pub fn gateway_with_bag_seeded(
        &self,
        position: [f32; 3],
        guid: u64,
        capacity: usize,
        battle: u32,
        seed: impl FnOnce(&mut Gateway),
    ) -> (Gateway, InventoryState) {
        let mut session = self.gateway(position, guid);
        seed(&mut session);
        let mut snapshot = session.snapshot();
        let bag = full_bag(
            &snapshot.gameplay.as_ref().unwrap().inventory,
            capacity,
            battle,
            PrizeStacks::MakeRoom,
        );
        snapshot.gameplay.as_mut().unwrap().inventory = bag.clone();
        let gameplay = snapshot.gameplay.unwrap();
        let mut gateway = Gateway::load_gameplay(
            &self.reference,
            session.state().clone(),
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

    pub fn save(&self, gateway: &Gateway, name: &str) -> PathBuf {
        let path = self.root.join(name);
        game_store::save(&path, &gateway.snapshot()).unwrap();
        path
    }
}

pub fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn battle_path() -> PathBuf {
    manifest_dir().join("data/content/field_battles.json")
}

pub fn content_dir() -> PathBuf {
    manifest_dir().join("data/content")
}

pub fn lab(relative: &str) -> PathBuf {
    manifest_dir().join("../../../lab").join(relative)
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

pub fn battles() -> Value {
    read_json(&battle_path())
}

pub fn battle<'a>(file: &'a Value, id: u32) -> &'a Value {
    file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(id)))
        .unwrap_or_else(|| panic!("battle {id} is missing"))
}

pub fn i32_field(value: &Value, field: &str) -> i32 {
    value[field].as_i64().unwrap() as i32
}

pub fn u32_field(value: &Value, field: &str) -> u32 {
    value[field].as_u64().unwrap() as u32
}

pub fn u64_field(value: &Value, field: &str) -> u64 {
    value[field].as_u64().unwrap()
}

pub fn position(value: &Value) -> [f32; 3] {
    let points = value.as_array().unwrap();
    [
        points[0].as_f64().unwrap() as f32,
        points[1].as_f64().unwrap() as f32,
        points[2].as_f64().unwrap() as f32,
    ]
}

pub fn prize_ids(row: &Value) -> std::collections::BTreeSet<i32> {
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

pub fn send(gateway: &mut Gateway, inner: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(inner), now_ms)
        .expect("gateway frame")
}

/// 26/10 menu selection: the client constructor takes an i32 choice.
pub fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    gateway_packets::select(target, choice as u32)
}

pub fn switch_job(profile: i32) -> Vec<u8> {
    [vec![26, 0, 13, 0], profile.to_le_bytes().to_vec()].concat()
}

/// 41/108 Go; constructor `0x008b6e70` accepts the encounter and instance from
/// the 41/114 screen. The trailing session is zero in native frames.
pub fn go(details: &[u8]) -> Vec<u8> {
    let mut packet = vec![41, 0, 108, 0];
    packet.extend(&details[4..12]);
    packet.extend(0_u64.to_le_bytes());
    packet
}

pub fn portal_play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0_u32.to_le_bytes());
    inner
}

pub fn chat_text(outcome: &GatewayOutcome) -> String {
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

pub fn details(outcome: &GatewayOutcome) -> Option<&[u8]> {
    outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
}

/// 26/9 interaction menu: find the option whose label string id is `label`.
pub fn menu_option(outcome: &GatewayOutcome, label: u32) -> Option<i32> {
    let menu = outcome
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))?;
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    (0..count).find_map(|index| {
        let offset = 17 + usize::try_from(index).unwrap() * 28;
        let id = i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap());
        let string_id = u32::from_le_bytes(menu[offset + 8..offset + 12].try_into().unwrap());
        (string_id == label).then_some(id)
    })
}

pub fn i32_at(packet: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

pub fn arena_spawned(gateway: &mut Gateway, now_ms: u64) -> GatewayOutcome {
    let mut outcome = send(gateway, &13_u16.to_le_bytes(), now_ms);
    outcome.replies.extend(
        send(gateway, &10_u16.to_le_bytes(), now_ms + 100)
            .replies
            .into_iter(),
    );
    outcome
}

pub fn level(gateway: &Gateway, profile: i32) -> u32 {
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

pub fn saved_bytes(gateway: &Gateway) -> Vec<u8> {
    // The typed snapshot payload the save envelope writes; the full-bag
    // refusal fixture is larger than game_store's 1 MiB document limit, so
    // compare the payload directly (as T282 does).
    serde_json::to_vec(&gateway.snapshot()).unwrap()
}

pub fn quest_counts(gateway: &Gateway, quest: u32) -> Vec<u32> {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .npc_quests
        .unwrap()
        .quests
        .get(&quest)
        .unwrap_or_else(|| panic!("quest {quest} is missing"))
        .counts
        .clone()
}

pub fn quest_status(gateway: &Gateway, quest: u32) -> QuestStatus {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .npc_quests
        .unwrap()
        .quests
        .get(&quest)
        .unwrap_or_else(|| panic!("quest {quest} is missing"))
        .status
        .clone()
}

pub fn walk(gateway: &mut Gateway, guid: u64, to: [f32; 3], now_ms: &mut u64) {
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

pub fn reopen(fixture: &Fixture, gateway: &Gateway, name: &str, now_ms: u64) -> Gateway {
    reopen_with(fixture, gateway, name, now_ms, |_| {})
}

/// Save, optionally patch the decoded gameplay state, then load it through the
/// same `game_store` envelope and `Gateway::load_gameplay` a reconnect uses.
pub fn reopen_with(
    fixture: &Fixture,
    gateway: &Gateway,
    name: &str,
    now_ms: u64,
    patch: impl FnOnce(&mut GameplayState),
) -> Gateway {
    let path = fixture.save(gateway, name);
    let mut saved = game_store::load_or_create(&path, gateway.state().clone()).unwrap();
    if let Some(gameplay) = saved.gameplay.as_mut() {
        patch(gameplay);
    }
    restore(fixture, saved, now_ms)
}

/// The same reconnect, with a saved position, so a quest accepted at one NPC
/// can be tested at the battle marker without walking across the zone.
pub fn reopen_at(
    fixture: &Fixture,
    gateway: &Gateway,
    name: &str,
    position: [f32; 3],
    now_ms: u64,
) -> Gateway {
    let path = fixture.save(gateway, name);
    let mut saved = game_store::load_or_create(&path, gateway.state().clone()).unwrap();
    saved.character.position = position;
    restore(fixture, saved, now_ms)
}

fn restore(fixture: &Fixture, saved: game_store::Snapshot, now_ms: u64) -> Gateway {
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
// Route and economy helpers shared by the field-battle suites. The battle id,
// arena, arrival and goal come from the row, so every suite that starts a
// battle runs this one packet chain instead of a local copy.
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct Economy {
    pub coins: u32,
    pub prizes: BTreeMap<i32, u32>,
    pub experience: BTreeMap<i32, u32>,
    /// The active job at snapshot time: the only profile a field-battle win pays.
    pub active: i32,
}

pub fn economy(gateway: &Gateway, prize_ids: &BTreeSet<i32>) -> Economy {
    let state = gateway.snapshot().gameplay.unwrap();
    let profiles = state.profiles.unwrap();
    let active = profiles.active;
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
        experience: profiles.experience,
        active,
    }
}

pub fn coins(gateway: &Gateway) -> u32 {
    gateway.snapshot().gameplay.unwrap().inventory.coins
}

pub fn active_profile(gateway: &Gateway) -> i32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .profiles
        .unwrap()
        .active
}

/// The first living arena enemy: (GUID, position, health).
pub fn active_enemy(gateway: &Gateway) -> Option<(u64, [f32; 3], u32)> {
    let state = gateway.snapshot().gameplay?.encounter?;
    let active = state.active?;
    let enemy = active.enemies.iter().find(|enemy| enemy.health > 0)?;
    Some((enemy.guid, enemy.position, enemy.health))
}

fn encounter_phase(gateway: &Gateway) -> Option<EncounterPhase> {
    gateway
        .snapshot()
        .gameplay?
        .encounter?
        .active
        .map(|active| active.phase)
}

/// The 41/114 start screen against the row, from the client reader chain
/// 0x00aa3430 -> 0x00aa32d0 -> 0x00a29120 -> 0x009bdd70: the id, an allocated
/// instance, then [title, icon, description, difficulty, profile type, type]
/// at bytes 47..71.
pub fn assert_start_screen(details: &[u8], row: &Value) {
    assert_eq!(
        i32_at(details, 4),
        u32_field(row, "id") as i32,
        "the 41/114 screen carries this battle's id"
    );
    assert!(i32_at(details, 8) > 0, "the start screen has an instance");
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
}

/// Click the marker, then pin its 41/114 start screen against the row.
pub fn open_battle(gateway: &mut Gateway, spawner_guid: u64, row: &Value, now_ms: u64) -> Vec<u8> {
    let screen = send(gateway, &interact(spawner_guid), now_ms);
    let details = details(&screen).expect("clicking the marker opens 41/114");
    assert_start_screen(details, row);
    details.to_vec()
}

/// Go, then assert every authored arena actor spawns with the row's state and
/// the running encounter refreshes through 41/114.
pub fn enter_battle(gateway: &mut Gateway, details: &[u8], row: &Value, now_ms: u64) -> Vec<u64> {
    let entered = send(gateway, &go(details), now_ms);
    assert!(entered.began_zoning, "Go enters the arena");
    assert_eq!(gateway.state().zone_id, u32_field(&row["zone"], "zone_id"));

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
    let objective = u32_field(row, "objective_id");
    assert!(
        ready
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .any(|packet| packet
                .windows(4)
                .any(|field| field == objective.to_le_bytes())),
        "the row's goal {objective} reaches the client"
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
    // The loader scales enemy health by the combat scale and the difficulty
    // factor (R), and only difficulty 1's factor is 1.0, so the scaled health
    // is observable exactly there; higher difficulties spawn at or above it.
    let authored_health = u32_field(row, "difficulty") == 1;
    for (actor, state) in row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .zip(&active.enemies)
    {
        assert_eq!(state.guid, u64_field(actor, "guid"));
        assert_eq!(state.position, position(&actor["position"]));
        if authored_health {
            assert_eq!(
                state.health,
                combat_scale::enemy_health(u32_field(actor, "max_health"))
            );
        } else {
            assert!(
                state.health >= combat_scale::enemy_health(u32_field(actor, "max_health")),
                "actor {} spawned at {} health",
                state.guid,
                state.health
            );
        }
    }
    enemy_guids
}

/// Kill every arena actor through the ordinary 36/10 path. Returns every reply
/// so the caller can assert the results packets.
pub fn fight_to_win(gateway: &mut Gateway, now_ms: &mut u64, row: &Value) -> Vec<Vec<u8>> {
    let player_guid = gateway.state().guid;
    let arrival = position(&row["zone"]["position"]);
    let mut replies = Vec::new();
    for _ in 0..200 {
        let Some((guid, at, _)) = active_enemy(gateway) else {
            break;
        };
        if encounter_phase(gateway) != Some(EncounterPhase::Fighting) {
            break;
        }
        walk(gateway, player_guid, [at[0] + 1.5, at[1], at[2]], now_ms);
        *now_ms += 700;
        // 36/10 writer 0x00a342e0: bar 1, slot 0, target mode 0, target GUID.
        let hit = send(gateway, &melee(guid), *now_ms);
        replies.extend(hit.replies);
        walk(gateway, player_guid, arrival, now_ms);
        *now_ms += 100;
        let tick = gateway.tick_at(*now_ms).unwrap();
        replies.extend(tick.replies);
    }
    assert_eq!(
        encounter_phase(gateway),
        Some(EncounterPhase::Won),
        "the arena actors die through the real attack path"
    );
    replies
}

/// The result lifecycle produced by `present_results`: 39/18 (game over),
/// 39/47 (scores) and 39/19 (results UI). 39/45 is the loot wheel and is not
/// part of this assertion.
pub fn results_lifecycle(replies: &[Vec<u8>]) -> Vec<u8> {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[39, 0]) && packet.len() >= 3)
        .map(|packet| packet[2])
        .filter(|sub| [18, 47, 19].contains(sub))
        .collect()
}

pub fn won_flag(replies: &[Vec<u8>]) -> Option<u8> {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.len() >= 16 && packet.starts_with(&[39, 0, 18]))
        .map(|packet| packet[15])
}

/// One win pays the row's coins, exactly one of the previewed prizes and the
/// active job's XP; no other profile gains any.
pub fn assert_payout(
    before: &Economy,
    after: &Economy,
    row: &Value,
    prize_ids: &BTreeSet<i32>,
    job_xp: u32,
) {
    assert_eq!(after.coins, before.coins + u32_field(row, "reward_coins"));
    for (profile, experience) in &after.experience {
        let prior = before.experience.get(profile).copied().unwrap_or(0);
        let delta = experience - prior;
        assert_eq!(
            delta,
            if *profile == before.active { job_xp } else { 0 },
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

/// The zone/position/rotation a results Close restores (the pre-entry marker).
pub fn assert_returned(gateway: &Gateway, marker: [f32; 3]) {
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, marker);
    assert_eq!(gateway.state().rotation, [0.0, 0.0, 1.0]);
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .is_none(),
        "the finished run is cleared"
    );
}

// ---------------------------------------------------------------------------
// 167/1/1 reader: header, entry type, count, ten dwords, CanPlayerJoin, then
// the variable-length tail (reader chain 0x00be79d0 -> 0x00be63e0).
// ---------------------------------------------------------------------------

pub struct PortalRow {
    pub id: u32,
    /// `+0x10`: the client files the row under `Activities.Category<category>`.
    pub category: u32,
    /// `+0x14` ImageSetId, the strip icon.
    pub icon: u32,
    pub title: u32,
    /// `+0x28` NameId, the name the battle finder strip shows.
    pub name: u32,
    pub description: u32,
    pub difficulty: u32,
    pub can_join: bool,
}

pub struct Cursor<'a> {
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

pub fn decode_list(packet: &[u8]) -> Vec<PortalRow> {
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
            category: words[2],
            icon: words[3],
            title: words[5],
            name: words[7],
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
// The 5 m authoring sample lives in `tests/support/terrain.rs`; placement suites
// include that module directly so the rule has one owner.
// ---------------------------------------------------------------------------

/// The nearest **regional** spawned NPC: only `legacy_data` rows in
/// `npcs/*.json` are actors there (`world.rs` skips POI/zone-centre
/// scaffolding). `npc_spawns_extra.json` is the second visible-actor source
/// that `nearest_extra_spawn` scans; a placement test must check both.
pub fn nearest_spawned_npc(marker: [f32; 3]) -> (f32, u64) {
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

/// The nearest visible actor from `npc_spawns_extra.json`, the second spawn
/// source `world.rs` publishes. Rows filtered at load (a captured match, the
/// missing knocker base model, a non-finite position or `content_key` 0) are
/// filtered here too.
pub fn nearest_extra_spawn(marker: [f32; 3]) -> (f32, u64) {
    let extras = read_json(&content_dir().join("npc_spawns_extra.json"));
    let mut nearest = (f32::INFINITY, 0_u64);
    for record in extras["records"].as_array().unwrap() {
        if !record["capture_match"].is_null()
            || record["model_file_name"]
                .as_str()
                .is_some_and(|name| name.eq_ignore_ascii_case("knocker_m_boss.adr"))
            || record["content_key"].as_u64() == Some(0)
        {
            continue;
        }
        let Some(point) = record["position"].as_array() else {
            continue;
        };
        if point.len() < 3
            || !point[..3]
                .iter()
                .all(|value| value.as_f64().is_some_and(f64::is_finite))
        {
            continue;
        }
        let distance = ((point[0].as_f64().unwrap() as f32 - marker[0]).powi(2)
            + (point[2].as_f64().unwrap() as f32 - marker[2]).powi(2))
        .sqrt();
        if distance < nearest.0 {
            nearest = (distance, record["spawn_guid"].as_u64().unwrap());
        }
    }
    nearest
}

pub fn nearest_field_enemy(marker: [f32; 3]) -> f32 {
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

/// The nearest other battle marker, excluding `battle`'s own spawner.
pub fn nearest_other_battle_marker(marker: [f32; 3], battle: u32) -> f32 {
    battles()["battles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["id"].as_u64() != Some(u64::from(battle)))
        .map(|row| {
            let point = position(&row["spawner"]["position"]);
            (point[0] - marker[0]).hypot(point[2] - marker[2])
        })
        .fold(f32::INFINITY, f32::min)
}

// ---------------------------------------------------------------------------
// Full bag: real catalogue items, no prize definition, exact stack count.
// ---------------------------------------------------------------------------

pub fn runtime_inventory() -> &'static Inventory {
    RUNTIME_INVENTORY.get_or_init(|| {
        Inventory::load(&manifest_dir().join(RUNTIME_CONTENT), None)
            .expect("the round70 runtime item catalogue loads")
    })
}

/// The definition ids a battle's chance-drop rows name.
fn battle_drop_ids(row: &Value) -> BTreeSet<i32> {
    row["drops"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|drop| i32::try_from(drop["item_id"].as_i64()?).ok())
        .collect()
}

/// A real uncapped catalogue row that is not one of this battle's prizes or
/// chance drops, so each filler stack stays separate and no granted item
/// merges into it.
pub fn filler_definition(battle_id: u32) -> i32 {
    let file = battles();
    let row = battle(&file, battle_id);
    let mut excluded = prize_ids(row);
    excluded.extend(battle_drop_ids(row));
    let inventory = runtime_inventory();
    inventory
        .definition_ids()
        .find(|id| {
            !excluded.contains(id)
                && inventory
                    .trade_flags(*id)
                    .is_some_and(|(_, max_stack)| max_stack.is_none())
        })
        .expect("the runtime catalogue has an uncapped filler item")
}

pub fn stack(id: i32, definition_id: i32, count: u32) -> InventoryItem {
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

/// What `full_bag` does with the base bag's own prize stacks.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PrizeStacks {
    /// Clear them, so each previewed prize needs a new stack.
    MakeRoom,
    /// Keep them (adding one of each the base lacks), so a granted prize
    /// merges into an existing stack instead of needing a slot.
    Keep,
}

/// Replace or keep the base bag's prize stacks (a battle's chance-drop stacks
/// are always removed), then pad to exactly `capacity` stacks with unique
/// filler. `capacity == MAX_ITEMS` leaves no room for a granted prize or a
/// drop; `MAX_ITEMS - 1` leaves one.
pub fn full_bag(
    base: &InventoryState,
    capacity: usize,
    battle_id: u32,
    prize_stacks: PrizeStacks,
) -> InventoryState {
    assert!(
        base.items.len() <= capacity,
        "the base bag ({} stacks) is larger than {capacity}",
        base.items.len()
    );
    let file = battles();
    let row = battle(&file, battle_id);
    let prizes = prize_ids(row);
    let mut state = base.clone();
    // A chance drop the base bag already holds would count as this win's
    // grant; start without one so the tests measure only the win's drop.
    let drops = battle_drop_ids(row);
    if !drops.is_empty() {
        let removed: std::collections::BTreeSet<i32> = state
            .items
            .iter()
            .filter(|item| drops.contains(&item.definition_id))
            .map(|item| item.id)
            .collect();
        state
            .items
            .retain(|item| !drops.contains(&item.definition_id));
        state
            .equipment
            .retain(|item| !removed.contains(&item.item_id));
    }
    if prize_stacks == PrizeStacks::MakeRoom {
        let removed: std::collections::BTreeSet<i32> = state
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
    }
    let mut id = state.items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    if prize_stacks == PrizeStacks::Keep {
        for &prize in &prizes {
            if !state.items.iter().any(|item| item.definition_id == prize) {
                state.items.push(stack(id, prize, 1));
                id += 1;
            }
        }
        assert!(
            state.items.len() <= capacity,
            "the kept prize stacks ({}) are larger than {capacity}",
            state.items.len()
        );
    }
    let filler = filler_definition(battle_id);
    while state.items.len() < capacity {
        state.items.push(stack(id, filler, 1));
        id += 1;
    }
    state.next_instance_id = Some(i64::from(id));
    state
}
