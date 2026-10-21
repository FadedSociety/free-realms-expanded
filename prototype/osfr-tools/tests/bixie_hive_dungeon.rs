//! T354: Bixie Hive as a playable level-1 field battle in Queensfields.
//!
//! Content-only slice: one new `field_battles.json` row and the existing route
//! (marker 26/8 click -> 41/114 start screen -> 41/108 Go -> arena zone 9600 ->
//! 36/10 attacks -> results -> 26/42 Close), plus the automatic activity-portal
//! row (167/1/1 Play, 167/1/2 Play). No new wire field, handler, opcode,
//! selector or engine code is added.
//!
//! Client shapes checked in the supplied 2014 client:
//! * 41/108 Go constructor `0x008b6e70`, sender `0x00938f00` (family 108 of
//!   the encounter packet; the sender fills the encounter and instance).
//! * 35/2 AddNpc body reader `0x00921a10`: u64 actor id, name id, model id,
//!   transform and nameplate. The dispatcher `0x0092f460` case 2 is 35/2.
//! * 35/3 removal dispatcher `0x0092f460` case 3 (constructor `0x008b3840`).
//! * 32/7 AttackProcessed reader `0x00a2a910`, handler `0x00a2ba40`.
//! * 41/114 start-screen reader chain `0x009bdd70` (called by `0x00a29120`);
//!   the field order is [title, icon, description, difficulty, profile type,
//!   type] at bytes 47..71 for this four-prize layout.
//! * 26/42 results Close constructor `0x008a2eb0`, sender `0x0092f300`.
//! * 36/10 basic-attack writer `0x00a342e0` (bar 1, slot 0, target mode 0,
//!   target GUID) and 167/1/1 list reader `0x00be79d0`.
//!
//! Placement evidence: the guide puts the Bixie Hive door in Queensfields, in
//! Sanctuary, among the roaming Bixie field actors. The authored marker
//! `[-670.0, 2.4, 252.0]` is the bilinear zone-0 height `2.40` rounded to 0.01.
//! The full 105-sample 5 m wander disc (81 integer-offset points inside the
//! disc plus 24 ring points at 15 degree steps) is terrain-covered with a
//! finite y; maximum sampled slope 0.00 degrees; nearest field-enemy home
//! 27.17 m (Bixie Soldier `131000102009` at `[-651.667,2.4,231.947]`); nearest
//! other battle marker 114.6 m (battle 12301); nearest spawned NPC 183.7 m.
//! Water, object collision and a native walkable route remain unverified, as
//! in battles 12301/12310/12350/12340.

use osfr_tools::{
    chat,
    encounter::{EncounterEngine, EncounterPhase},
    gateway::Gateway,
    inventory::MAX_ITEMS,
    startup_world::{NpcSpawn, parse_add_npc},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "support/terrain.rs"]
mod terrain;

use field_battle_fixture::*;
use terrain::{disc_points, slope_degrees};

/// The new row, inserted directly after battle 96.
const BIXIE_HIVE: u32 = 12370;
const TITLE: i32 = 5702;
const DESCRIPTION: i32 = 15336;
const OBJECTIVE: i32 = 16791;
const DIFFICULTY: u32 = 1;
const REWARD_COINS: u32 = 68;
const MARKER_GUID: u64 = 100001237000;
const ENEMY_GUIDS: [u64; 5] = [
    100001237001,
    100001237002,
    100001237003,
    100001237004,
    100001237005,
];
/// Marker definition: a non-combat gate adaptation (R), not an enemy kind.
const MARKER_DEFINITION: i32 = 1237000;
const ENEMY_DEFINITIONS: [i32; 5] = [1237001, 1237002, 1237003, 1237004, 1237005];
const BOSS_NAME: i32 = 38510;
const SOLDIER_NAME: i32 = 440711;
const MAGE_NAME: i32 = 440712;
const SOLDIER_MODEL: i32 = 220;
const MAGE_MODEL: i32 = 211;
/// Bilinear zone-0 height 2.40 rounded to 0.01 (T103 authoring rule).
const MARKER: [f32; 3] = [-670.0, 2.4, 252.0];
const NAMEPLATE: i32 = 11970;
/// Images.txt 662: icon_ui_combatstart_panelgraphic_bixiehive.dds.
const ICON: i32 = 662;
const ARENA_ZONE: u32 = 9600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
const BRAWLER: i32 = 43;
const BATTLE_JOB_XP: u32 = 50;
/// The existing Bixie field actors whose GUIDs and homes the loader also
/// rejects on collision.
const BIXIE_FIELD_GUIDS: [u64; 3] = [131000102007, 131000102008, 131000102009];
/// The designated five-battle roster from `field_battle_roster`; the new row
/// is deliberately not part of it.
const TASK_ROSTER: [u32; 5] = [96, 12301, 12302, 12310, 12311];
const REFUSAL_PRIZE: &str = "one of its rewards cannot be granted";
/// 41/114 details layout for four prizes, from `details_packet`:
/// header 4 + id 4 + instance 4 + 24 + 3 + 8 + the six text dwords (47..71) +
/// 1 + 138 + 61 + prize count 4 + four 38-byte entries + 4 = 431; then the
/// objective block and 69 zero bytes place the roster count at 525. The whole
/// packet is 575 bytes.
const DETAILS_LEN: usize = 575;
const DETAILS_ID: usize = 4;
const DETAILS_INSTANCE: usize = 8;
const DETAILS_FIELDS: usize = 47;
const DETAILS_PRIZE_COUNT: usize = 271;
const DETAILS_PRIZES: usize = 275;
const DETAILS_ROSTER_COUNT: usize = 525;
/// One prize-bundle Item entry: type i32, hidden u8, icon/name/quantity/item
/// and four more i32, then members-only u8 (38 bytes, `prize_bundle`).
const PRIZE_ENTRY: usize = 38;
const PRIZE_ICON: usize = 5;
const PRIZE_NAME: usize = 13;
const PRIZE_QUANTITY: usize = 17;
const PRIZE_ITEM: usize = 21;

// ---------------------------------------------------------------------------
// Local fixture seams: the full bag (same production save path as T319).
// ---------------------------------------------------------------------------

impl Fixture {
    /// Restore a save built by the production `game_store` path with a bag of
    /// exactly `capacity` stacks. The seed has entered the world and is on a
    /// level-1 combat job.
    fn gateway_with_bag(
        &self,
        position: [f32; 3],
        guid: u64,
        capacity: usize,
    ) -> (Gateway, osfr_tools::inventory::InventoryState) {
        self.gateway_with_bag_seeded(position, guid, capacity, BIXIE_HIVE, |seed| {
            assert!(send(seed, &switch_job(BRAWLER), 300).state_changed);
            assert_eq!(level(seed, BRAWLER), 1);
        })
    }
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

fn add_npc_guids(replies: &[Vec<u8>]) -> Vec<u64> {
    replies
        .iter()
        .map(|packet| unframe(packet))
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet.len() >= 12)
        .map(|packet| u64::from_le_bytes(packet[4..12].try_into().unwrap()))
        .collect()
}

// ---------------------------------------------------------------------------
// Published 167/1/1 activity list (shared row reader in the fixture module).
// ---------------------------------------------------------------------------

fn published_list(replies: &[Vec<u8>]) -> Vec<PortalRow> {
    let packet = replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[167, 0, 1, 1]))
        .expect("a 167/1/1 activity list was sent");
    decode_list(packet)
}

fn published_row(replies: &[Vec<u8>]) -> PortalRow {
    let rows = published_list(replies);
    assert_eq!(
        rows.iter().filter(|row| row.id == BIXIE_HIVE).count(),
        1,
        "Bixie Hive is published exactly once"
    );
    rows.into_iter()
        .find(|row| row.id == BIXIE_HIVE)
        .expect("Bixie Hive is in the 167/1/1 list")
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

fn coins(gateway: &Gateway) -> u32 {
    gateway.snapshot().gameplay.unwrap().inventory.coins
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

fn open_battle(gateway: &mut Gateway, spawner_guid: u64, row: &Value, now_ms: u64) -> Vec<u8> {
    let screen = send(gateway, &interact(spawner_guid), now_ms);
    let details = details(&screen).expect("clicking the marker opens 41/114");
    assert_eq!(i32_at(details, DETAILS_ID), BIXIE_HIVE as i32);
    assert!(
        i32_at(details, DETAILS_INSTANCE) > 0,
        "the start screen has an instance"
    );

    // 41/114 reader chain 0x009bdd70: title, icon, description, difficulty,
    // profile type, type.
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
        &details[DETAILS_FIELDS..DETAILS_FIELDS + 24],
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
        "the verified goal 16791 reaches the client"
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
/// 39/47 (scores) and 39/19 (results UI).
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

/// The zone/position/rotation a results Close restores (the marker pose).
fn assert_returned(gateway: &Gateway) {
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);
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
// Tests.
// ---------------------------------------------------------------------------

/// 1. The start screen reports this battle's instance, exact 41/114 text
/// fields, four-prize preview and five-actor roster count through the existing
/// packet; closing it releases the pending Go.
#[test]
fn the_start_screen_carries_the_instance_and_roster_count() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let mut gateway = fixture.gateway(MARKER, 1001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);

    let screen = send(&mut gateway, &interact(MARKER_GUID), 500);
    let details = details(&screen).expect("the marker opens 41/114");
    assert_eq!(details.len(), DETAILS_LEN, "no field was added or dropped");
    assert_eq!(i32_at(details, DETAILS_ID), BIXIE_HIVE as i32);
    let instance = i32_at(details, DETAILS_INSTANCE);
    assert!(instance > 0, "the instance is allocated, not zero");
    assert_eq!(
        i32_at(details, DETAILS_PRIZE_COUNT),
        4,
        "the four previewed prizes are in the bundle"
    );
    for (index, prize) in row["prizes"].as_array().unwrap().iter().enumerate() {
        let at = DETAILS_PRIZES + index * PRIZE_ENTRY;
        // Native reader 0x009bdd70 and the prize-bundle reader 0x008e7930:
        // each previewed row is a type-1 Item entry with icon, name, quantity
        // and item id.
        assert_eq!(i32_at(details, at), 1, "prize {index} is an Item entry");
        assert_eq!(
            i32_at(details, at + PRIZE_ICON),
            i32_field(prize, "icon_id")
        );
        assert_eq!(
            i32_at(details, at + PRIZE_NAME),
            i32_field(prize, "name_id")
        );
        assert_eq!(
            i32_at(details, at + PRIZE_QUANTITY),
            prize["quantity"].as_u64().unwrap_or(1) as i32
        );
        assert_eq!(
            i32_at(details, at + PRIZE_ITEM),
            i32_field(prize, "item_id")
        );
    }
    assert_eq!(
        i32_at(details, DETAILS_ROSTER_COUNT),
        5,
        "the client receives the five-actor roster count"
    );

    // 41/107 ZoneIsReady carries the same encounter and instance.
    let ready = screen
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[41, 0, 107, 0]))
        .expect("the start screen is marked ready");
    assert_eq!(i32_at(ready, DETAILS_ID), BIXIE_HIVE as i32);
    assert_eq!(i32_at(ready, DETAILS_INSTANCE), instance);

    // 26/42 closes the start screen (constructor 0x008a2eb0) without zoning.
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let closed = send(&mut gateway, &[26, 0, 42, 0], 700);
    assert!(!closed.began_zoning && !closed.state_changed);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "closing the start screen changes no saved state"
    );
    let replay = send(&mut gateway, &go(&details.to_vec()), 800);
    assert!(!replay.began_zoning && !replay.state_changed);
    assert_eq!(
        gateway.state().zone_id,
        0,
        "a closed screen cannot be started by a replayed Go"
    );
}

/// 2. The content row is exactly the authored slice: ids, five actors, models,
/// names, positions, health/damage (before the difficulty-1 factor of 1),
/// payout, arena and prize pool. A level-1 combat job starts the battle
/// through marker Go; every arena actor spawns with the authored state.
#[test]
fn a_level_one_character_can_start() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    assert_eq!(u32_field(row, "title_id"), TITLE as u32);
    assert_eq!(u32_field(row, "description_id"), DESCRIPTION as u32);
    assert_eq!(u32_field(row, "objective_id"), OBJECTIVE as u32);
    assert_eq!(u32_field(row, "difficulty"), DIFFICULTY);
    assert_eq!(u32_field(row, "required_combat_level"), 0);
    assert_eq!(u32_field(row, "reward_coins"), REWARD_COINS);
    assert_eq!(i32_field(row, "icon_id"), ICON);
    assert_eq!(i32_field(row, "nameplate_image_id"), NAMEPLATE);
    assert_eq!(u32_field(&row["zone"], "zone_id"), ARENA_ZONE);
    assert_eq!(row["zone"]["name"], "sg_random_encounter_clearing");
    assert_eq!(row["zone"]["sky"], "sky_wilds24.xml");
    assert_eq!(u32_field(&row["zone"], "geometry_id"), 214);
    assert_eq!(position(&row["zone"]["position"]), ARENA_ARRIVAL);
    assert_eq!(position(&row["spawner"]["position"]), MARKER);

    let spawner = &row["spawner"];
    assert_eq!(u64_field(spawner, "guid"), MARKER_GUID);
    assert_eq!(i32_field(spawner, "definition_id"), MARKER_DEFINITION);
    assert_eq!(i32_field(spawner, "model_id"), SOLDIER_MODEL);
    assert_eq!(i32_field(spawner, "name_id"), SOLDIER_NAME);
    assert_eq!(spawner["name"], "Bixie Soldier");
    assert_eq!(u32_field(spawner, "max_health"), 1);
    assert_eq!(u32_field(spawner, "damage"), 0);
    assert_eq!(spawner["boss"], false);

    let enemies = row["enemies"].as_array().unwrap();
    assert_eq!(enemies.len(), 5);
    assert_eq!(u32_field(&row["zone"], "zone_id"), ARENA_ZONE);
    let expected = [
        (
            ENEMY_GUIDS[0],
            ENEMY_DEFINITIONS[0],
            SOLDIER_MODEL,
            BOSS_NAME,
            "Drone Fauzz",
            [136.0, -0.6, 175.0],
            260,
            10,
            true,
        ),
        (
            ENEMY_GUIDS[1],
            ENEMY_DEFINITIONS[1],
            SOLDIER_MODEL,
            SOLDIER_NAME,
            "Bixie Soldier",
            [133.0, -0.6, 173.0],
            120,
            5,
            false,
        ),
        (
            ENEMY_GUIDS[2],
            ENEMY_DEFINITIONS[2],
            SOLDIER_MODEL,
            SOLDIER_NAME,
            "Bixie Soldier",
            [139.0, -0.6, 173.0],
            120,
            5,
            false,
        ),
        (
            ENEMY_GUIDS[3],
            ENEMY_DEFINITIONS[3],
            MAGE_MODEL,
            MAGE_NAME,
            "Bixie Battlemage",
            [134.0, -0.6, 176.0],
            90,
            7,
            false,
        ),
        (
            ENEMY_GUIDS[4],
            ENEMY_DEFINITIONS[4],
            MAGE_MODEL,
            MAGE_NAME,
            "Bixie Battlemage",
            [138.0, -0.6, 176.0],
            90,
            7,
            false,
        ),
    ];
    for (enemy, (guid, definition, model, name_id, name, at, health, damage, boss)) in
        enemies.iter().zip(expected)
    {
        assert_eq!(u64_field(enemy, "guid"), guid);
        assert_eq!(i32_field(enemy, "definition_id"), definition);
        assert_eq!(i32_field(enemy, "model_id"), model);
        assert_eq!(i32_field(enemy, "name_id"), name_id);
        assert_eq!(enemy["name"], name);
        assert_eq!(position(&enemy["position"]), at);
        assert_eq!(u32_field(enemy, "max_health"), health);
        assert_eq!(u32_field(enemy, "damage"), damage);
        assert_eq!(enemy["boss"], boss);
    }

    let mut gateway = fixture.gateway(MARKER, 2001);
    assert_ne!(
        active_profile(&gateway),
        BRAWLER,
        "the default job is not the combat job"
    );
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    assert_eq!(active_profile(&gateway), BRAWLER);
    assert_eq!(level(&gateway, BRAWLER), 1, "level-1 combat job");
    assert!(
        !TASK_ROSTER.contains(&BIXIE_HIVE),
        "the new row is outside the designated five-battle roster"
    );

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let enemy_guids = enter_battle(&mut gateway, &screen, row, 1_000);
    assert_eq!(enemy_guids, ENEMY_GUIDS);

    // Replaying the consumed Go is inert while the run is live.
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let replay = send(&mut gateway, &go(&screen), 1_900);
    assert!(!replay.began_zoning && !replay.state_changed);
    assert_eq!((gateway.snapshot(), saved_bytes(&gateway)), before);
}

/// 3. The marker satisfies the T103/T282/T319 bounded terrain rule: the
/// bilinear zone-0 height rounded to 0.01, all 105 samples of the 5 m wander
/// disc terrain-covered with a finite y, maximum slope at most 25 degrees, and
/// the authored exclusions from every field-enemy home (the three Bixie
/// actors), another battle marker and a spawned NPC.
#[test]
fn the_marker_is_terrain_safe_and_clear_of_the_bixie_camp() {
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let marker = position(&row["spawner"]["position"]);
    assert_eq!(marker, MARKER);

    let terrain = terrain::load();
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

    // The three existing Bixie field actors are the nearby homes: the marker
    // stays outside the 20 m exclusion and outside the whole 5 m disc.
    let field = read_json(&content_dir().join("field_enemies.json"));
    for guid in BIXIE_FIELD_GUIDS {
        let home = field["enemies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|enemy| u64_field(enemy, "guid") == guid)
            .unwrap_or_else(|| panic!("Bixie field actor {guid} is missing"));
        let home = position(&home["position"]);
        let distance = (marker[0] - home[0]).hypot(marker[2] - home[2]);
        assert!(
            distance >= 25.0,
            "Bixie actor {guid} is {distance:.2} m from the marker"
        );
    }
    let enemy = nearest_field_enemy(marker);
    assert!(enemy >= 25.0, "nearest field enemy is {enemy:.1} m away");
    let other = nearest_other_battle_marker(marker, BIXIE_HIVE);
    assert!(other >= 25.0, "nearest battle marker is {other:.1} m away");
    let (npc, npc_id) = nearest_spawned_npc(marker);
    assert!(
        npc >= 15.0,
        "nearest spawned NPC {npc_id} is {npc:.2} m away"
    );

    // The samples recorded in the row's source and this header match the real
    // grid: a flat 2.40 m disc and at least 27 m to the nearest Bixie home.
    assert!(max_slope < 0.01, "the authored disc is flat");
    assert!((marker[1] - 2.4).abs() < 0.01);
}

/// 4. A non-combat active job is refused by the existing 41/108 gate, the
/// start screen reopens unchanged, and the same Go starts after a job switch.
#[test]
fn a_non_combat_job_is_refused() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let mut gateway = fixture.gateway(MARKER, 2101);
    assert_ne!(
        active_profile(&gateway),
        BRAWLER,
        "the default job is not a combat job"
    );

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 400);
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let refused = send(&mut gateway, &go(&screen), 500);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("Switch to a combat job before starting a battle."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        details(&refused).expect("the start screen reopens"),
        screen.as_slice(),
        "the same screen returns byte-for-byte"
    );
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "a job refusal changes no state"
    );

    // The control: after switching to the level-1 Brawler the same screen starts.
    assert!(send(&mut gateway, &switch_job(BRAWLER), 600).state_changed);
    let started = send(&mut gateway, &go(&screen), 700);
    assert!(started.began_zoning, "the control job starts the battle");
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
}

/// 5. A live arena run is a busy activity: the portal refuses the same and
/// another battle, a marker click opens no screen, and nothing changes.
#[test]
fn a_busy_character_is_refused() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let mut gateway = fixture.gateway(MARKER, 2201);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let before = (gateway.snapshot(), saved_bytes(&gateway));

    for activity in [BIXIE_HIVE, 12350] {
        let refused = send(&mut gateway, &portal_play(activity), 1_500);
        assert!(!refused.began_zoning && !refused.state_changed);
        assert!(
            chat_text(&refused).contains("Finish what you are doing"),
            "activity {activity}: {}",
            chat_text(&refused)
        );
    }
    let clicked = send(&mut gateway, &interact(MARKER_GUID), 1_600);
    assert!(
        details(&clicked).is_none(),
        "a busy marker click opens no start screen"
    );
    assert!(!clicked.state_changed && !clicked.began_zoning);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "busy refusals change no state"
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
        Some(EncounterPhase::Fighting),
        "the live run keeps running"
    );
}

/// 6. The full marker -> Go -> five kills -> results route pays 68 coins, one
/// prize and 50 job XP exactly once; results Close returns the pre-entry pose
/// and clears the run, and replayed win/close packets pay nothing.
#[test]
fn the_full_route_pays_sixty_eight_coins_and_one_prize() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(MARKER, 3001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = economy(&gateway, &prizes);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let first_instance = i32_at(&screen, DETAILS_INSTANCE);
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
    assert_eq!(active_profile(&gateway), BRAWLER);

    // A replayed attack on the won fight pays nothing.
    let won = gateway.snapshot();
    let repeated = send(&mut gateway, &melee(enemy_guids[0]), now + 700);
    assert!(!repeated.state_changed, "a won fight cannot pay twice");
    assert_eq!(gateway.snapshot(), won);

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_400);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_returned(&gateway);
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

    // A second start on the same session allocates the next instance.
    arena_spawned(&mut gateway, now + 2_200);
    let again = open_battle(&mut gateway, MARKER_GUID, row, now + 2_300);
    assert!(
        i32_at(&again, DETAILS_INSTANCE) > first_instance,
        "a later match never reuses the earlier instance"
    );
    let dismissed = send(&mut gateway, &[26, 0, 42, 0], now + 2_400);
    assert!(!dismissed.began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
}

/// 7. Forged Go, a forged portal id, a wrong marker, out-of-range Go, an
/// arbitrary 36/10 target, a non-combat job (separately tested above), a
/// knocked-out character and a full bag each change no typed state or save
/// bytes; controls prove the same requests start under the qualifying
/// condition.
#[test]
fn refusals_leave_everything_unchanged() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let prizes = prize_ids(row);

    // Forged Go with no open start screen.
    let mut gateway = fixture.gateway(MARKER, 4001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let mut forged = vec![41, 0, 108, 0];
    forged.extend(BIXIE_HIVE.to_le_bytes());
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

    // A forged 167/1/2 Play id: the catalogue does not publish it.
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let unknown = send(&mut gateway, &portal_play(0x00ff_ffff), 430);
    assert!(!unknown.began_zoning && !unknown.state_changed);
    assert!(
        chat_text(&unknown).contains("That activity is not available."),
        "{}",
        chat_text(&unknown)
    );
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "a forged portal Play id changes no state"
    );
    assert!(
        published_list(&send(&mut gateway, &[167, 0, 1, 4], 450).replies)
            .iter()
            .any(|row| row.id == BIXIE_HIVE),
        "the session still answers an ordinary refresh"
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

    // An arbitrary 36/10 target GUID is not an actor in this arena.
    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let arbitrary = send(&mut gateway, &melee(0xdead_beef), now + 450);
    assert!(!arbitrary.began_zoning && !arbitrary.state_changed);
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "an arbitrary 36/10 target changes no state"
    );
    let instance = i32_at(&screen, DETAILS_INSTANCE);
    let cancel = [
        vec![41, 0, 109, 0],
        0_i32.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
    ]
    .concat();
    assert!(send(&mut gateway, &cancel, now + 500).began_zoning);
    assert_eq!(gateway.state().position, MARKER);

    // Knocked out: Go is a chat refusal, not a session-ending error.
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

/// 8. Every previewed prize resolves in the shipped catalogue and the whole
/// bundle fits an empty bag; the production loader accepts the row and the
/// preflight refuses a synthetic unresolvable prize.
#[test]
fn every_previewed_prize_resolves_and_fits() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let inventory = runtime_inventory();
    let prizes = row["prizes"].as_array().unwrap();
    assert!(!prizes.is_empty());
    let mut bundle = Vec::new();
    for prize in prizes {
        let item = i32_field(prize, "item_id");
        let quantity = prize["quantity"].as_u64().unwrap_or(1) as u32;
        assert!(
            inventory.has_definition(item),
            "prize {item} resolves in the shipped catalogue"
        );
        assert!(
            inventory.accepts_reward(item, quantity),
            "prize {item} x{quantity} fits one stack"
        );
        bundle.push((item, quantity));
    }
    assert!(
        inventory.accepts_reward_bundle(&bundle),
        "all four previewed prizes fit the bag together"
    );

    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the real loader accepts the shipped row");
    assert!(
        engine.battle_prizes_available(BIXIE_HIVE, |item, quantity| {
            inventory.accepts_reward(item, quantity)
        }),
        "the whole-bundle preflight accepts the four prizes"
    );

    // A prize id absent from the catalogue passes the loader's positive-id
    // check but fails the gameplay preflight (UNAVAILABLE_BATTLE_PRIZE).
    let mut invalid = file.clone();
    invalid["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == json!(BIXIE_HIVE))
        .unwrap()["prizes"][0]["item_id"] = json!(2_147_483_647_i64);
    let path = fixture.root.join("unresolvable-prize.json");
    fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&path)
        .expect("the loader only checks that the prize id is positive");
    assert!(
        !engine.battle_prizes_available(BIXIE_HIVE, |item, quantity| {
            inventory.accepts_reward(item, quantity)
        }),
        "an unresolvable previewed prize is refused before the start"
    );
}

/// 9. The new GUIDs never collide with another battle, field actor, encounter
/// actor or the Bixie field actors; the loader accepts the file. Synthetic
/// duplicates are refused by the real loader.
#[test]
fn guids_are_unique_across_shipped_content() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let mut ours = BTreeSet::new();
    for actor in std::iter::once(&row["spawner"]).chain(row["enemies"].as_array().unwrap().iter()) {
        let guid = u64_field(actor, "guid");
        assert!(guid > 1);
        assert!(ours.insert(guid), "the new row repeats GUID {guid}");
    }
    assert_eq!(ours.len(), 6);

    let rows = file["battles"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .filter(|row| row["id"].as_u64() == Some(u64::from(BIXIE_HIVE)))
            .count(),
        1,
        "the battle id is unique"
    );
    let mut all = BTreeSet::new();
    for battle in rows {
        for actor in
            std::iter::once(&battle["spawner"]).chain(battle["enemies"].as_array().unwrap().iter())
        {
            let guid = u64_field(actor, "guid");
            assert!(guid > 1);
            assert!(
                all.insert(guid),
                "battle {} repeats GUID {guid}",
                battle["id"]
            );
        }
    }

    let field = read_json(&content_dir().join("field_enemies.json"));
    let mut field_guids = BTreeSet::new();
    for enemy in field["enemies"].as_array().unwrap() {
        let guid = u64_field(enemy, "guid");
        assert!(field_guids.insert(guid));
        assert!(
            !ours.contains(&guid),
            "field actor {guid} collides with the new battle"
        );
    }
    for guid in BIXIE_FIELD_GUIDS {
        assert!(
            field_guids.contains(&guid),
            "the existing Bixie field actor {guid} is present"
        );
        assert!(!ours.contains(&guid));
    }

    let encounters = read_json(&content_dir().join("encounters.json"));
    let mut encounter_guids = BTreeSet::new();
    for enemy in encounters["world_enemies"].as_array().unwrap() {
        encounter_guids.insert(u64_field(enemy, "guid"));
    }
    for dungeon in encounters["dungeons"].as_array().unwrap() {
        for enemy in dungeon["enemies"].as_array().unwrap() {
            encounter_guids.insert(u64_field(enemy, "guid"));
        }
        encounter_guids.insert(u64_field(&dungeon["guide"], "guid"));
        assert_ne!(
            u32_field(dungeon, "id"),
            BIXIE_HIVE,
            "the battle id is unique against encounters.json"
        );
    }
    assert!(
        encounter_guids.is_disjoint(&ours),
        "a new GUID collides with an encounter actor"
    );

    let mut engine = EncounterEngine::load(&fixture.content, None).unwrap();
    engine
        .load_field_battles(&battle_path())
        .expect("the real loader accepts every actor GUID");
    let spawner = engine
        .battle_spawners()
        .into_iter()
        .find(|entry| entry.0 == MARKER_GUID)
        .expect("the marker is published as a battle spawner");
    assert_eq!(spawner.1, "Bixie Soldier");
    assert_eq!(spawner.2, SOLDIER_NAME);
    assert_eq!(spawner.3, SOLDIER_MODEL);
    assert_eq!(spawner.4, MARKER);
    assert_eq!(spawner.5, NAMEPLATE);

    // A duplicate GUID and a duplicate row are refused at load.
    let mut collision = file.clone();
    collision["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == json!(BIXIE_HIVE))
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
    let copied = battle(&duplicate, BIXIE_HIVE).clone();
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

/// 10. Two gateways run the battle independently: neither sees the other's
/// actors, one win does not damage the other run, and each is paid once.
#[test]
fn a_second_player_runs_its_own_instance_and_is_paid_once() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let prizes = prize_ids(row);
    let mut first = fixture.gateway(MARKER, 5001);
    let mut second = fixture.gateway(MARKER, 5002);
    for session in [&mut first, &mut second] {
        assert!(send(session, &switch_job(BRAWLER), 300).state_changed);
    }
    let first_before = economy(&first, &prizes);
    let second_before = economy(&second, &prizes);
    let first_screen = open_battle(&mut first, MARKER_GUID, row, 500);
    let second_screen = open_battle(&mut second, MARKER_GUID, row, 500);
    assert!(i32_at(&first_screen, DETAILS_INSTANCE) > 0);
    assert!(i32_at(&second_screen, DETAILS_INSTANCE) > 0);
    enter_battle(&mut first, &first_screen, row, 600);
    enter_battle(&mut second, &second_screen, row, 600);

    let second_active = second
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active;
    let second_save = saved_bytes(&second);
    assert!(second_active.is_some());

    let mut first_now = 700;
    let won = fight_to_win(&mut first, &mut first_now);
    assert_eq!(results_lifecycle(&won), vec![18, 47, 19]);
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
        "the second player's actors and health are unchanged"
    );
    assert_eq!(
        saved_bytes(&second),
        second_save,
        "the first player's win never changes the second save"
    );

    let mut second_now = 700;
    let won = fight_to_win(&mut second, &mut second_now);
    assert_eq!(results_lifecycle(&won), vec![18, 47, 19]);
    let second_paid = economy(&second, &prizes);
    assert_payout(&second_before, &second_paid, row, &prizes);
    assert_eq!(
        economy(&first, &prizes),
        first_paid,
        "each player is paid exactly once"
    );
}

/// 11. A run dropped by save/reload returns to the marker without a second
/// payout or stuck busy state; a won, unclosed run keeps its payout and
/// survives reload without paying again (engine-wide; asserted, not changed).
#[test]
fn the_win_and_the_mid_fight_return_survive_a_reload() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let prizes = prize_ids(row);

    // Mid-fight: no payout, no active run, and the marker can start again.
    let mut gateway = fixture.gateway(MARKER, 6001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let mut dropped = reopen(&fixture, &gateway, "bixie-mid-run.json", 2_000);
    assert_returned(&dropped);
    assert_eq!(economy(&dropped, &prizes), before);
    let screen = open_battle(&mut dropped, MARKER_GUID, row, 2_500);
    let restarted = send(&mut dropped, &go(&screen), 2_600);
    assert!(
        restarted.began_zoning,
        "the dropped run leaves no stuck busy state"
    );

    // Won, unclosed: the real fight pays; reloading must not pay again.
    let mut gateway = fixture.gateway(MARKER, 6002);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = economy(&gateway, &prizes);
    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    enter_battle(&mut gateway, &screen, row, 600);
    let mut now = 700;
    let won = fight_to_win(&mut gateway, &mut now);
    assert_eq!(results_lifecycle(&won), vec![18, 47, 19]);
    let paid = economy(&gateway, &prizes);
    assert_payout(&before, &paid, row, &prizes);
    let mut dropped = reopen(&fixture, &gateway, "bixie-won-unclosed.json", now + 1_000);
    assert_returned(&dropped);
    assert_eq!(economy(&dropped, &prizes), paid);
    let before_replay = saved_bytes(&dropped);
    let replay = send(&mut dropped, &melee(ENEMY_GUIDS[0]), now + 1_100);
    assert!(!replay.state_changed, "a reloaded win pays nothing");
    assert_eq!(saved_bytes(&dropped), before_replay);
}

/// 12. The portal publishes the battle automatically; a non-combat job cannot
/// Play it, a level-1 combat job can, and the Play starts the same run through
/// the same gates and prize preflight.
#[test]
fn the_portal_row_applies_the_same_gates() {
    let fixture = Fixture::new();
    let (mut gateway, entry) = fixture.session(MARKER, 7001);
    let published = published_row(&entry.replies);
    assert_eq!(published.title, TITLE as u32);
    assert_eq!(published.description, DESCRIPTION as u32);
    assert_eq!(published.difficulty, DIFFICULTY);
    assert!(
        !published.can_join,
        "the default non-combat job cannot join"
    );

    let before = (gateway.snapshot(), saved_bytes(&gateway));
    let refused = send(&mut gateway, &portal_play(BIXIE_HIVE), 300);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains("Switch to a combat job before starting a battle."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        (gateway.snapshot(), saved_bytes(&gateway)),
        before,
        "a portal job refusal changes no state"
    );

    assert!(send(&mut gateway, &switch_job(BRAWLER), 400).state_changed);
    let refreshed = send(&mut gateway, &[167, 0, 1, 4], 700);
    assert!(
        published_row(&refreshed.replies).can_join,
        "a level-1 combat job may join"
    );
    let before = economy(&gateway, &prize_ids(battle(&battles(), BIXIE_HIVE)));
    let arena = send(&mut gateway, &portal_play(BIXIE_HIVE), 800);
    assert!(arena.began_zoning, "Play starts the same run");
    assert_eq!(gateway.state().zone_id, ARENA_ZONE);
    let ready = arena_spawned(&mut gateway, 900);
    for guid in ENEMY_GUIDS {
        assert!(
            ready
                .replies
                .iter()
                .map(|packet| unframe(packet))
                .any(|packet| packet.starts_with(&[35, 0, 2, 0])
                    && packet[4..12] == guid.to_le_bytes()),
            "portal Play spawned arena enemy {guid}"
        );
    }
    assert!(
        ready
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .any(|packet| packet.starts_with(&[41, 0, 114, 0])),
        "the running portal run refreshes through 41/114"
    );
    let active = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .expect("the portal run is active");
    assert_eq!(active.phase, EncounterPhase::Fighting);
    assert_eq!(active.enemies.len(), ENEMY_GUIDS.len());
    assert!(active.instance_id > 0);
    assert_eq!(
        economy(&gateway, &prize_ids(battle(&battles(), BIXIE_HIVE))),
        before,
        "starting the portal run pays nothing"
    );
    let cancel = [
        vec![41, 0, 109, 0],
        0_i32.to_le_bytes().to_vec(),
        active.instance_id.to_le_bytes().to_vec(),
    ]
    .concat();
    assert!(send(&mut gateway, &cancel, 1_500).began_zoning);
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, MARKER);

    // The portal path runs the same whole-bundle prize preflight: a full bag
    // is refused with UNAVAILABLE_BATTLE_PRIZE and changes nothing.
    let (mut full, bag) = fixture.gateway_with_bag(MARKER, 7002, MAX_ITEMS);
    assert_eq!(bag.items.len(), MAX_ITEMS);
    let before = (full.snapshot(), saved_bytes(&full));
    let refused = send(&mut full, &portal_play(BIXIE_HIVE), 800);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        (full.snapshot(), saved_bytes(&full)),
        before,
        "a portal prize refusal changes no state"
    );
}

/// 13. A second session in zone 0 receives the new marker's 35/2 AddNpc at the
/// authored position and never the arena actors; a late joiner gets the same.
#[test]
fn a_second_session_sees_the_marker_not_the_arena() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let (mut first, first_entry) = fixture.session(MARKER, 8001);
    let (_second, second_entry) = fixture.session(MARKER, 8002);
    let first_packet = marker_packet(&first_entry.replies, MARKER_GUID);
    let second_packet = marker_packet(&second_entry.replies, MARKER_GUID);
    assert_eq!(
        first_packet, second_packet,
        "the marker is identical for both sessions (no per-player actor)"
    );
    let spawn = marker_spawn(second_packet);
    assert_eq!(spawn.actor_id, MARKER_GUID);
    assert_eq!(spawn.name_string_id, SOLDIER_NAME);
    assert_eq!(spawn.model_id, SOLDIER_MODEL);
    assert_eq!(spawn.position[..3], MARKER);
    assert_eq!(spawn.nameplate_image_id, NAMEPLATE);
    assert!(
        add_npc_guids(&second_entry.replies)
            .iter()
            .all(|guid| !ENEMY_GUIDS.contains(guid)),
        "the second session never receives the arena actors"
    );

    assert!(send(&mut first, &switch_job(BRAWLER), 300).state_changed);
    let screen = open_battle(&mut first, MARKER_GUID, row, 500);
    enter_battle(&mut first, &screen, row, 600);
    assert!(
        add_npc_guids(&second_entry.replies)
            .iter()
            .all(|guid| !ENEMY_GUIDS.contains(guid)),
        "the first player entering the arena spawns nothing for the second"
    );

    // A late joiner after the first player is already fighting gets the marker.
    let (_late, late_entry) = fixture.session(MARKER, 8003);
    assert_eq!(
        marker_packet(&late_entry.replies, MARKER_GUID),
        second_packet,
        "a late joiner sees the same static marker"
    );
    assert!(
        add_npc_guids(&late_entry.replies)
            .iter()
            .all(|guid| !ENEMY_GUIDS.contains(guid)),
        "a late joiner never receives another player's arena actors"
    );
    assert_eq!(
        marker_spawn(marker_packet(&late_entry.replies, MARKER_GUID)).position[..3],
        MARKER
    );
}

/// 14. The payout applies deltas to the live state: a coin grant made after
/// the start screen opens survives the 68-coin win payout and the Close.
#[test]
fn a_live_change_before_the_close_survives_the_payout() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let prizes = prize_ids(row);
    let mut gateway = fixture.gateway(MARKER, 9001);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 300).state_changed);
    let before = economy(&gateway, &prizes);

    let screen = open_battle(&mut gateway, MARKER_GUID, row, 500);
    let granted = send(&mut gateway, &chat::local_request("realm coins 7"), 600);
    assert!(granted.state_changed, "the live coin grant committed");
    assert_eq!(coins(&gateway), before.coins + 7);
    let live = economy(&gateway, &prizes);
    assert_eq!(live.coins, before.coins + 7);

    enter_battle(&mut gateway, &screen, row, 700);
    let mut now = 800;
    let won = fight_to_win(&mut gateway, &mut now);
    assert_eq!(results_lifecycle(&won), vec![18, 47, 19]);
    let paid = economy(&gateway, &prizes);
    assert_payout(&live, &paid, row, &prizes);
    assert_eq!(
        paid.coins,
        before.coins + 7 + REWARD_COINS,
        "the payout adds to the live coins instead of restoring the entry snapshot"
    );

    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 700);
    assert!(closed.began_zoning);
    assert_returned(&gateway);
    assert_eq!(
        economy(&gateway, &prizes),
        paid,
        "Close does not overwrite the live bag or coins"
    );
}

/// 15. One session's refused start (non-combat job and full bag) changes only
/// that session; the other keeps running and is paid, and the refused session
/// can still start after switching jobs.
#[test]
fn a_failing_start_does_not_stop_the_other_session() {
    let fixture = Fixture::new();
    let file = battles();
    let row = battle(&file, BIXIE_HIVE);
    let prizes = prize_ids(row);

    let mut failing = fixture.gateway(MARKER, 9501);
    let mut healthy = fixture.gateway(MARKER, 9502);
    assert!(send(&mut healthy, &switch_job(BRAWLER), 300).state_changed);

    // Session one fails the combat-job gate.
    let screen = open_battle(&mut failing, MARKER_GUID, row, 400);
    let failed_before = (failing.snapshot(), saved_bytes(&failing));
    let refused = send(&mut failing, &go(&screen), 500);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert_eq!(
        (failing.snapshot(), saved_bytes(&failing)),
        failed_before,
        "the refused start changed only itself"
    );

    // Session two is unaffected and wins through the normal route.
    let healthy_before = economy(&healthy, &prizes);
    let screen = open_battle(&mut healthy, MARKER_GUID, row, 500);
    enter_battle(&mut healthy, &screen, row, 600);
    let mut now = 700;
    let won = fight_to_win(&mut healthy, &mut now);
    assert_eq!(results_lifecycle(&won), vec![18, 47, 19]);
    let healthy_paid = economy(&healthy, &prizes);
    assert_payout(&healthy_before, &healthy_paid, row, &prizes);
    assert_eq!(
        (failing.snapshot(), saved_bytes(&failing)),
        failed_before,
        "the other player's win never touches the failed session"
    );

    // A second failing start (full bag / unavailable prize) also stays local.
    let (mut full, bag) = fixture.gateway_with_bag(MARKER, 9503, MAX_ITEMS);
    assert_eq!(bag.items.len(), MAX_ITEMS);
    let screen = open_battle(&mut full, MARKER_GUID, row, 800);
    let full_before = (full.snapshot(), saved_bytes(&full));
    let refused = send(&mut full, &go(&screen), 900);
    assert!(!refused.began_zoning && !refused.state_changed);
    assert!(
        chat_text(&refused).contains(REFUSAL_PRIZE),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        (full.snapshot(), saved_bytes(&full)),
        full_before,
        "the over-cap refusal changed only the third session"
    );
    assert_eq!(
        economy(&healthy, &prizes),
        healthy_paid,
        "the third session's refusal never touches the paid session"
    );
    assert_eq!(saved_bytes(&failing), failed_before.1);

    // The failed session still answers a refresh and can start after the fix.
    assert!(
        published_list(&send(&mut failing, &[167, 0, 1, 4], 1_000).replies)
            .iter()
            .any(|row| row.id == BIXIE_HIVE)
    );
    assert!(send(&mut failing, &switch_job(BRAWLER), 1_100).state_changed);
    let screen = open_battle(&mut failing, MARKER_GUID, row, 1_200);
    let started = send(&mut failing, &go(&screen), 1_300);
    assert!(
        started.began_zoning,
        "the once-refused session starts after switching jobs"
    );
    assert_eq!(failing.state().zone_id, ARENA_ZONE);
}
