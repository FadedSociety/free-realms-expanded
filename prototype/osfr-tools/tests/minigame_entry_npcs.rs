//! T221: the kart and derby entry drivers stand at the speedway portals and start the race UI.
//!
//! The content side is `data/content/kart_gates.json` and `data/content/derby_gates.json`; the
//! drivers themselves are the speedway NPCs already spawned by the client content (the portal
//! objects and their positions are in `notes/Research/Minigame Entry Points.md`). No new actor or
//! art is invented: every gate NPC is an existing drawable driver at its speedway entrance.
//!
//! The gateway tests copy the reference gameplay package, add the shipped kart/derby files and
//! drive the ordinary 26/8 -> 26/10 application path through `Gateway`; the NPC content comes from
//! the shipped `data/content/npcs` through the production fallback, so a gate id has to resolve to
//! a real NPC record. All timing values here are test clocks, not game data.

use osfr_tools::{
    character_store::CharacterState,
    derby_gateway::{DerbyGates, DerbyGateway, DerbyGatewayContent},
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    kart_gateway::{DEFAULT_MENU_STRING_ID, KartGateway},
    mining,
    npc_interaction::NpcInteraction,
    social_chat::ActionBudgets,
    world::NpcVisibility,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

const CRATE: &str = env!("CARGO_MANIFEST_DIR");
/// The four kart trainers already listed before T221 plus the two missing speedways.
const CHASE: u64 = 9_000_000_273;
const CALE: u64 = 9_000_000_039;
const MUSTANG: u64 = 9_500_000_759;
const JOHNNY: u64 = 9_500_000_911;
const SPUD: u64 = 9_500_000_630;
/// The four derby trainers at the four arenas (Sandstorm has no derby).
const MAD_JACK: u64 = 9_000_000_391;
const CRASH: u64 = 9_000_000_289;
const BURT: u64 = 9_500_000_731;
const RIP: u64 = 9_000_000_183;
/// Clack: a Tangletrack NPC that is not a kart gate.
const NOT_A_GATE: u64 = 9_000_000_048;
const KART_JOB: i32 = 48;
const PLAYER: u64 = 0x1000_0000_0000_0001;
const NOW: u64 = 10_000;
static NEXT: AtomicU64 = AtomicU64::new(0);

/// `(npc, portal object, portal position)`; the object positions come from the client GCNK
/// world-object catalogue (`lab/reports/world-objects-20261015.json`). A driver further than this
/// from the entrance is not "in front of the portal" any more.
const KART_PORTALS: [(u64, &str, [f32; 3]); 5] = [
    (
        CHASE,
        "sg_turnstyle_01 @ Thunder Falls",
        [-915.95, -39.16, 1205.96],
    ),
    (
        CALE,
        "sg_turnstyle_01 @ Tangletrack",
        [-271.49, -58.86, 1952.21],
    ),
    (
        MUSTANG,
        "sg_raceway_01 @ Frost Ridge",
        [-237.6, 1.05, -51.89],
    ),
    (
        JOHNNY,
        "sg_raceway_01 @ Wildwood",
        [-2262.71, -16.44, 585.04],
    ),
    (SPUD, "ss_raceway_01 @ Sandstorm", [31.6, -14.6, -1192.3]),
];
const DERBY_PORTALS: [(u64, &str, [f32; 3]); 4] = [
    (
        MAD_JACK,
        "sg_raceway_01 @ Wildwood",
        [-2262.71, -16.44, 585.04],
    ),
    (
        CRASH,
        "sg_destruction_derby_01 @ Thunder Falls",
        [-911.67, -39.82, 1213.06],
    ),
    (BURT, "sg_raceway_01 @ Frost Ridge", [-237.6, 1.05, -51.89]),
    (
        RIP,
        "bw_destruction_derby_01 @ Tangletrack",
        [-264.65, -58.96, 1947.87],
    ),
];
const PORTAL_REACH_M: f32 = 25.0;

fn data() -> PathBuf {
    Path::new(CRATE).join("data/content")
}

fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_TYPE8_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(CRATE).join("../../../lab/references/opening-journal-text-v2/content")
        })
}

fn reference() -> PathBuf {
    std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(CRATE).join("../../../lab/references/rust-gateway-reference"))
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

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn npc_record(id: u64) -> Value {
    for entry in fs::read_dir(data().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let document: Value = read_json(&path);
        if let Some(row) = document["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(id))
        {
            return row.clone();
        }
    }
    panic!("NPC {id} content record");
}

fn npc_position(id: u64) -> [f32; 3] {
    serde_json::from_value(npc_record(id)["position"].clone()).unwrap()
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(left, right)| {
            let delta = *left - right;
            delta * delta
        })
        .sum::<f32>()
        .sqrt()
}

fn assert_drawable(id: u64, visibility: &NpcVisibility) {
    let record = npc_record(id);
    let owner = format!("NPC {id} ({})", record["name"]);
    assert_eq!(
        record["placement"], "legacy_data",
        "{owner}: only legacy_data regional rows are drawn"
    );
    let model_id = record["model_id"]
        .as_u64()
        .unwrap_or_else(|| panic!("{owner}: model id"));
    assert!(model_id > 0, "{owner}: model id 0 is not drawable");
    let model_file = record["model_file_name"]
        .as_str()
        .unwrap_or_else(|| panic!("{owner}: model file"));
    assert!(
        model_file.ends_with(".agr") || model_file.ends_with(".adr"),
        "{owner}: {model_file:?} is a client actor model"
    );
    let position: [f32; 3] = serde_json::from_value(record["position"].clone()).unwrap();
    assert!(
        position.iter().all(|value| value.is_finite()),
        "{owner}: finite position"
    );
    // The production visibility loader is what emits the AddNpc actor; a content record alone is
    // not a spawn.
    assert_eq!(
        visibility.content_key(id),
        Some(id),
        "{owner}: the click identity is the NPC content key"
    );
    assert!(
        visibility.model_actors().any(|(_zone, model, at)| {
            u64::from(model) == model_id && distance(at, position) < 0.01
        }),
        "{owner}: no drawable actor is emitted at {position:?}"
    );
}

/// An operator-shaped `derby_entry.json` (parsed, not a Rust struct literal), so the entry suite
/// stays independent of derby fields other tasks add. Completion rewards are absent (and therefore
/// zero on a tip that pays one), so this package offers a match and pays nothing.
fn derby_content() -> DerbyGatewayContent {
    serde_json::from_value(json!({
        "schema_version": 1,
        "game_id": 39,
        "vehicle": {
            "name": [115, 121, 110, 116, 104, 101, 116, 105, 99, 45, 100, 101, 114, 98, 121],
            "definition": [1, 2, 3, 4],
            "start_transform_bits": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            "unknown_flag": false,
            "unknown_value_bits": 0
        },
        "rules": {
            "max_drivers": 8,
            "start_health": 20,
            "contact_damage_per_unit": 1.0,
            "contact_cooldown_ms": 100,
            "contact_radius_m": 5.0,
            "max_impact": 50.0,
            "max_pose_age_ms": 5000,
            "max_damage_per_contact": 100,
            "lobby_timeout_ms": 120000,
            "pickup_radius": 3.0,
            "health_pickup": 5,
            "boost_pickup": 5.0,
            "shield_duration_ms": 1000,
            "attack_duration_ms": 1000,
            "ultimate_duration_ms": 1000,
            "attack_multiplier": 2.0,
            "ultimate_multiplier": 2.0,
            "respawn_delay_ms": 1000,
            "results_duration_ms": 1000,
            "knockout_points": 2,
            "leader_knockout_points": 3,
            "knockout_point_loss": 1,
            "score_target": 2,
            "time_limit_ms": null,
            "config": { "value_a_bits": 0, "value_b": 0 }
        },
        "pickups": []
    }))
    .expect("operator-shaped derby package")
}

/// Reference gameplay content plus the shipped kart/derby gate files. NPC content is deliberately
/// absent so the production fallback loads the shipped `data/content/npcs`, exactly like a bundle
/// that ships only the activity files.
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-minigame-entry-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&reference_content(), &dir);
        for file in ["kart_entry.json", "kart_gates.json", "derby_gates.json"] {
            fs::copy(data().join(file), dir.join(file)).unwrap();
        }
        Self(dir)
    }

    fn write_derby_package(&self) {
        fs::write(
            self.0.join("derby_entry.json"),
            serde_json::to_vec(&derby_content()).unwrap(),
        )
        .unwrap();
    }

    fn state(&self) -> GameplayState {
        Gameplay::load(&self.0, None).unwrap().state()
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut packet = vec![5, 0, 1];
    packet.extend((inner.len() as u32).to_le_bytes());
    packet.extend(inner);
    packet
}

fn login(gateway: &mut Gateway) {
    let guid = gateway.state().guid;
    let mut packet = 1u16.to_le_bytes().to_vec();
    for (index, value) in ["t221-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        packet.extend((value.len() as u32).to_le_bytes());
        packet.extend(value.as_bytes());
        if index == 0 {
            packet.extend(guid.to_le_bytes());
        }
    }
    gateway.handle_at(&packet, 0).unwrap();
    for packet in [wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway.handle_at(&packet, 1_000).unwrap();
    }
}

fn gateway_at(content: &Content, state: GameplayState, position: [f32; 3]) -> Gateway {
    let mut character: CharacterState = Gateway::reference_state(&reference()).unwrap();
    character.zone_id = 0;
    character.position = position;
    let mut gateway =
        Gateway::load_gameplay(&reference(), character, &content.0, Some(state)).unwrap();
    login(&mut gateway);
    gateway
}

fn npc_click(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

fn click(gateway: &mut Gateway, target: u64, now: u64) -> GatewayOutcome {
    gateway.handle_at(&wrap(&npc_click(target)), now).unwrap()
}

fn choose(gateway: &mut Gateway, target: u64, choice: i32, now: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(&npc_select(target, choice)), now)
        .unwrap()
}

fn bodies(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|packet| {
            assert!(packet.len() >= 7 && packet[..3] == [5, 0, 1]);
            packet[7..].to_vec()
        })
        .collect()
}

/// The option id whose 26/9 row carries `label`, from the client menu reader shape.
fn menu_option(menu: &[Vec<u8>], label: u32) -> Option<i32> {
    let packet = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))?;
    let count = i32::from_le_bytes(packet[13..17].try_into().ok()?) as usize;
    (0..count).find_map(|index| {
        let at = 17 + index * 28;
        let id = i32::from_le_bytes(packet[at..at + 4].try_into().ok()?);
        let row_label = u32::from_le_bytes(packet[at + 8..at + 12].try_into().ok()?);
        (row_label == label).then_some(id)
    })
}

fn save_bytes(gateway: &Gateway) -> Vec<u8> {
    serde_json::to_vec(&gateway.snapshot()).unwrap()
}

// ---- content ------------------------------------------------------------------------------

#[test]
fn shipped_kart_gates_load_and_every_gate_is_a_drawable_driver_at_its_portal() {
    let content = data();
    let gateway = KartGateway::load(&content, None)
        .expect("kart gate bounds")
        .expect("the shipped kart package");
    assert_eq!(
        gateway.gate_npcs(),
        &[CHASE, CALE, MUSTANG, JOHNNY, SPUD][..],
        "the three reviewed gates stay and Wildwood/Sandstorm join them"
    );
    assert_eq!(gateway.menu_string_id(), DEFAULT_MENU_STRING_ID);

    let visibility = NpcVisibility::load(&content).unwrap();
    for (id, object, portal) in KART_PORTALS {
        assert_drawable(id, &visibility);
        let at = npc_position(id);
        assert!(
            distance(at, portal) <= PORTAL_REACH_M,
            "NPC {id} at {at:?} is {:.1} m from {object}, not in front of the portal",
            distance(at, portal)
        );
    }

    let mut interaction =
        NpcInteraction::load(&content.join("npcs"), &content.join("stores.json")).unwrap();
    assert!(
        interaction
            .set_race_gates(gateway.gate_npcs(), gateway.menu_string_id())
            .is_empty(),
        "every kart gate id must be NPC content the menu can list"
    );
}

#[test]
fn shipped_derby_gates_load_and_every_gate_is_a_drawable_driver_at_its_arena() {
    let content = data();
    let gates: DerbyGates =
        serde_json::from_slice(&fs::read(content.join("derby_gates.json")).unwrap()).unwrap();
    assert_eq!(gates.schema_version, 1);
    assert_eq!(gates.menu_string_id, 3053, "Demo Derby Lobby");
    assert_eq!(gates.npcs, vec![MAD_JACK, CRASH, BURT, RIP]);

    // The production validator accepts the shipped file with a real derby package.
    let valid = DerbyGateway::new(derby_content(), Some(gates.clone())).unwrap();
    assert_eq!(valid.gate_npcs(), gates.npcs);
    assert_eq!(valid.menu_string_id(), gates.menu_string_id);

    let visibility = NpcVisibility::load(&content).unwrap();
    for (id, object, portal) in DERBY_PORTALS {
        assert_drawable(id, &visibility);
        let at = npc_position(id);
        assert!(
            distance(at, portal) <= PORTAL_REACH_M,
            "NPC {id} at {at:?} is {:.1} m from {object}, not in front of the arena",
            distance(at, portal)
        );
    }

    let mut interaction =
        NpcInteraction::load(&content.join("npcs"), &content.join("stores.json")).unwrap();
    assert!(
        interaction
            .set_derby_gates(&gates.npcs, gates.menu_string_id)
            .is_empty(),
        "every derby gate id must be NPC content the menu can list"
    );
}

#[test]
fn the_shipped_package_offers_from_every_gate() {
    let content = data();
    // The production loader is the check: the shipped package and the four gate ids are read
    // together, so every driver's menu publishes the derby.
    let shipped = DerbyGateway::load(&content, None)
        .expect("the shipped package respects every loader bound")
        .expect("the shipped derby package exists");
    assert_eq!(shipped.gate_npcs(), &[MAD_JACK, CRASH, BURT, RIP][..]);
    assert_eq!(shipped.menu_string_id(), 3053);
    let mut interaction =
        NpcInteraction::load(&content.join("npcs"), &content.join("stores.json")).unwrap();
    assert!(
        interaction
            .set_derby_gates(shipped.gate_npcs(), shipped.menu_string_id())
            .is_empty(),
        "every shipped gate offers the derby exactly once"
    );
}

#[test]
fn a_kart_gate_list_is_dormant_without_race_content() {
    // Existing dormant rule: the kart list is only published with race content.
    let fixture = Content::new();
    let mut entry = read_json(&fixture.0.join("kart_entry.json"));
    entry["race"] = Value::Null;
    fs::write(
        fixture.0.join("kart_entry.json"),
        serde_json::to_vec(&entry).unwrap(),
    )
    .unwrap();
    let dormant = KartGateway::load(&fixture.0, None).unwrap().unwrap();
    assert!(
        dormant.gate_npcs().is_empty(),
        "a gate list must not be offered without race content"
    );
}

#[test]
fn the_race_choice_shares_the_account_action_budget() {
    let mut budgets = ActionBudgets::default();
    let now = Instant::now();
    let choice = npc_select(JOHNNY, 2);
    for _ in 0..10 {
        assert!(
            budgets.allow(PLAYER, &choice, now),
            "26/10 is a budgeted application request"
        );
    }
    assert!(
        !budgets.allow(PLAYER, &choice, now),
        "a burst of race selections is refused before gameplay sees it"
    );
}

// ---- real gateway -------------------------------------------------------------------------

#[test]
fn the_wildwood_driver_offers_and_starts_the_kart_race_for_two_players() {
    let content = Content::new();
    let position = npc_position(JOHNNY);
    let base = content.state();
    let prior_job = base.profiles.as_ref().unwrap().active;

    // Each session owns its own gameplay clone; the NPC keeps no per-player state.
    let mut first = gateway_at(&content, base.clone(), position);
    let mut second = gateway_at(&content, base, position);

    let menu = bodies(&click(&mut first, JOHNNY, NOW));
    let option = menu_option(&menu, DEFAULT_MENU_STRING_ID).expect("Kart Race Lobby option");
    let started = choose(&mut first, JOHNNY, option, NOW + 100);
    assert!(started.gameplay_changed, "the choice must be saved");
    assert!(
        bodies(&started)
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 114, 0])),
        "the race UI starts with the encounter details"
    );
    let first_state = first.snapshot().gameplay.unwrap();
    let entry = first_state.kart_entry.as_ref().unwrap();
    assert!(entry.active.is_some(), "the race entry is active");
    assert_eq!(first_state.profiles.unwrap().active, KART_JOB);
    let first_after_start = save_bytes(&first);

    // The second player at the same driver gets the same offer and their own entry; the first
    // player's session is untouched by it.
    let second_menu = bodies(&click(&mut second, JOHNNY, NOW));
    let second_option =
        menu_option(&second_menu, DEFAULT_MENU_STRING_ID).expect("second player's menu");
    let second_started = choose(&mut second, JOHNNY, second_option, NOW + 100);
    assert!(second_started.gameplay_changed);
    assert!(
        second
            .snapshot()
            .gameplay
            .unwrap()
            .kart_entry
            .unwrap()
            .active
            .is_some()
    );
    assert_eq!(
        save_bytes(&first),
        first_after_start,
        "another player's entry must not touch this session"
    );

    // The client loads the race scene (13/10) before the native close returns it to the world.
    for loading in [wrap(&[13, 0]), wrap(&[10, 0])] {
        first.handle_at(&loading, NOW + 150).unwrap();
    }
    let context = first_state.kart_entry.unwrap().active.unwrap().context;
    let mut close = vec![39, 0, 7];
    close.extend(context.encounter_id.to_le_bytes());
    close.extend([255; 8]);
    let coins = first.snapshot().gameplay.unwrap().inventory.coins;
    let closed = first
        .handle_at(&wrap(&close), NOW + 200)
        .expect("close reply");
    assert!(
        closed.state_changed || !closed.replies.is_empty(),
        "the native close must be accepted, not dropped"
    );
    let after = first.snapshot().gameplay.unwrap();
    assert!(after.kart_entry.unwrap().active.is_none());
    assert_eq!(after.profiles.unwrap().active, prior_job);
    assert_eq!(after.inventory.coins, coins);
}

#[test]
fn a_busy_player_is_refused_and_nothing_changes() {
    let content = Content::new();
    mining::write_content(&content.0, mining::MINING_GAME_ID).unwrap();
    let base = Gameplay::load(&content.0, None).unwrap();
    let mut state = base.state();
    let mut engine = mining::Type8Engine::load(
        &content.0,
        mining::CONTENT_FILE,
        mining::MINING_GAME_ID,
        mining::FIRST_MINING_INSTANCE,
        None,
    )
    .unwrap();
    engine.begin(mining::MINER_PROFILE_ID).unwrap();
    assert!(engine.active());
    state.mining = Some(engine.state().clone());
    let mut game = Gameplay::load(&content.0, Some(state)).unwrap();
    assert!(
        game.state().mining.is_some(),
        "the busy Mining run must load"
    );

    let mut character: CharacterState = Gateway::reference_state(&reference()).unwrap();
    character.zone_id = 0;
    character.position = npc_position(JOHNNY);
    let before = serde_json::to_vec(&game.state()).unwrap();
    let menu = game
        .handle_in_world(&npc_click(JOHNNY), &character, NOW)
        .unwrap();
    assert!(
        !menu
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "a busy player must not get the driver's menu"
    );
    let refused = game
        .handle_in_world(&npc_select(JOHNNY, 2), &character, NOW + 100)
        .expect("a busy refusal is reported, not a disconnect");
    assert!(!refused.changed);
    assert_eq!(
        serde_json::to_vec(&game.state()).unwrap(),
        before,
        "a busy refusal changes no save state"
    );
    assert!(
        game.state().kart_entry.unwrap().active.is_none(),
        "a busy player must not enter the race"
    );
}

#[test]
fn an_ineligible_or_forged_choice_changes_nothing() {
    let content = Content::new();
    let position = npc_position(NOT_A_GATE);
    let mut gateway = gateway_at(&content, content.state(), position);
    let before = save_bytes(&gateway);
    let menu = bodies(&click(&mut gateway, NOT_A_GATE, NOW));
    assert!(
        menu_option(&menu, DEFAULT_MENU_STRING_ID).is_none(),
        "an NPC that is not a gate must not list the race"
    );
    let _ = gateway.handle_at(&wrap(&npc_select(NOT_A_GATE, 2)), NOW + 100);
    assert_eq!(
        save_bytes(&gateway),
        before,
        "an unlisted NPC cannot start a race"
    );

    // A gate's open menu refuses a forged choice without consuming the menu.
    let mut gateway = gateway_at(&content, content.state(), npc_position(JOHNNY));
    let before = save_bytes(&gateway);
    let menu = bodies(&click(&mut gateway, JOHNNY, NOW));
    let option = menu_option(&menu, DEFAULT_MENU_STRING_ID).expect("race option");
    let _ = gateway.handle_at(&wrap(&npc_select(JOHNNY, 99)), NOW + 100);
    assert_eq!(
        save_bytes(&gateway),
        before,
        "a forged choice changes nothing"
    );
    let started = choose(&mut gateway, JOHNNY, option, NOW + 200);
    assert!(started.gameplay_changed, "the offered choice still works");
}

#[test]
fn a_kart_driver_offers_nothing_while_the_track_is_dormant() {
    let content = Content::new();
    let mut entry = read_json(&content.0.join("kart_entry.json"));
    entry["race"] = Value::Null;
    fs::write(
        content.0.join("kart_entry.json"),
        serde_json::to_vec(&entry).unwrap(),
    )
    .unwrap();
    let mut gateway = gateway_at(&content, content.state(), npc_position(JOHNNY));
    let menu = bodies(&click(&mut gateway, JOHNNY, NOW));
    assert!(
        menu_option(&menu, DEFAULT_MENU_STRING_ID).is_none(),
        "a dormant kart track must not list the race"
    );
    let before = save_bytes(&gateway);
    let _ = gateway.handle_at(&wrap(&npc_select(JOHNNY, 2)), NOW + 100);
    assert_eq!(save_bytes(&gateway), before);
}

#[test]
fn the_derby_driver_offers_the_derby_when_a_package_exists() {
    let content = Content::new();
    content.write_derby_package();
    let position = npc_position(MAD_JACK);
    let mut gateway = gateway_at(&content, content.state(), position);
    let menu = bodies(&click(&mut gateway, MAD_JACK, NOW));
    let option = menu_option(&menu, 3053).expect("Demo Derby Lobby option");
    let started = choose(&mut gateway, MAD_JACK, option, NOW + 100);
    assert!(started.gameplay_changed);
    assert!(
        bodies(&started)
            .iter()
            .any(|packet| packet.starts_with(&[39, 0, 16])),
        "the derby offer is the existing 39/16 packet"
    );
    let racings = save_bytes(&gateway);
    let _ = gateway.handle_at(&wrap(&npc_select(MAD_JACK, option)), NOW + 200);
    assert_eq!(
        save_bytes(&gateway),
        racings,
        "a live derby is not entered twice"
    );
}

#[test]
fn a_derby_driver_offers_nothing_while_the_derby_package_is_absent() {
    let content = Content::new();
    assert!(!content.0.join("derby_entry.json").exists());
    let position = npc_position(MAD_JACK);
    let mut gateway = gateway_at(&content, content.state(), position);
    let menu = bodies(&click(&mut gateway, MAD_JACK, NOW));
    assert!(
        menu_option(&menu, 3053).is_none(),
        "a dormant derby must not be listed"
    );
    let before = save_bytes(&gateway);
    let _ = gateway.handle_at(&wrap(&npc_select(MAD_JACK, 99)), NOW + 100);
    assert_eq!(save_bytes(&gateway), before);
}
