//! T313: the shipped demolition-derby package is playable from the four arena drivers.
//!
//! `data/content/derby_entry.json` carries the captured `ReadVehicleDefinition` bytes from the
//! shared kart/derby vehicle block (client reader `0x009d7b50`; the derby creator `0x009f3800` and
//! the kart creator `0x00ba7370` both call it). The definition is the first 6108 bytes of the
//! captured `33/11` vehicle body: 22-byte body asset name, the `400.0` first attribute, and the
//! attribute block up to the captured owner GUID. It is not the synthetic `[1, 2, 3, 4]` fixture.
//!
//! Every rule value in the package is reconstructed (R): eight drivers, knockout points 2/3/1
//! (client locale 1086358690/3417922616/2192467831), the Wildwood 1000 first-to-22 target (client
//! locale 2192467831), 100 health and one-hit contact damage for the slice, and the 25 coin /
//! 10 Demo Derby Driver XP completion reward from T219. The four gate NPCs and the 3053 lobby label
//! are the shipped `derby_gates.json` (T221).
//!
//! The tests drive the real path: `Gateway` 26/8 -> 26/10 -> `Gameplay::begin_derby`, the native
//! 39/16 offer -> 39/5 Go -> 34/6 register -> phase/countdown acknowledgements -> 34/16 contacts ->
//! results payout -> 26/42 close. No test calls a copy of the feature.

use osfr_tools::{
    character_store::CharacterState,
    derby::{
        DerbyMessage, DerbyPhase, ParticipantKey, VehicleHitReport, VehiclePose, decode_inner,
        encode_inner,
    },
    derby_gateway::{DEMO_DERBY_DRIVER_PROFILE, DerbyGates, DerbyGateway, DerbyGatewayContent},
    game_store::Snapshot,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    mining::{self, Type8Engine},
    npc_interaction::NpcInteraction,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CRATE: &str = env!("CARGO_MANIFEST_DIR");
/// The four arena drivers shipped by `derby_gates.json` (T221).
const MAD_JACK: u64 = 9_000_000_391;
const CRASH: u64 = 9_000_000_289;
const BURT: u64 = 9_500_000_731;
const RIP: u64 = 9_000_000_183;
/// A real Tangletrack NPC that is not a derby gate.
const NOT_A_GATE: u64 = 9_000_000_048;
const DERBY_LABEL: u32 = 3053;
const COMPLETION_COINS: u32 = 25;
const COMPLETION_XP: u32 = 10;
/// The captured `ReadVehicleDefinition` byte count for the shared kart/derby vehicle.
const DEFINITION_LEN: usize = 6108;
/// SHA-256 of the recovered definition bytes (see notes/Autopilot/T313.md).
const DEFINITION_SHA256: &str = "0a62b7e0a974a6423bd092089d7570621bba80d4e8141add16151eb5ef3a6c5e";
/// The first length-prefixed name inside the recovered `ReadVehicleDefinition` bytes.
const BODY_ASSET: &str = "kt_kart08_body_bbe.cdt";
const NOW: u64 = 10_000;
static NEXT: AtomicU64 = AtomicU64::new(0);

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

/// A scratch gameplay package: the reference world plus the two shipped derby files.
struct Shipped(PathBuf);

impl Shipped {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-derby-shipped-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&reference_content(), &path);
        for file in ["derby_gates.json", "derby_entry.json"] {
            let source = data().join(file);
            // `derby_entry.json` is what makes the route playable; a missing package must reach
            // the driver menu as "no offer", so a canary fails on the behaviour, not on a copy.
            if source.is_file() {
                fs::copy(source, path.join(file)).unwrap();
            }
        }
        Self(path)
    }
}

impl Drop for Shipped {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A scratch directory with only the two derby content files, for loader refusals.
struct Package(PathBuf);

impl Package {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-derby-package-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        for file in ["derby_entry.json", "derby_gates.json"] {
            fs::copy(data().join(file), path.join(file)).unwrap();
        }
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Package {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn npc_position(id: u64) -> [f32; 3] {
    for entry in fs::read_dir(data().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let document = read_json(&path);
        if let Some(row) = document["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(id))
        {
            return serde_json::from_value(row["position"].clone()).unwrap();
        }
    }
    panic!("NPC {id} content record");
}

fn character_at(position: [f32; 3]) -> CharacterState {
    let mut character = Gateway::reference_state(&reference()).unwrap();
    character.zone_id = 0;
    character.position = position;
    character
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut packet = vec![5, 0, 1];
    packet.extend((inner.len() as u32).to_le_bytes());
    packet.extend(inner);
    packet
}

/// The application handshake the T221 gateway suite already uses; all fields are client values.
fn login(gateway: &mut Gateway) {
    let guid = gateway.state().guid;
    let mut packet = 1u16.to_le_bytes().to_vec();
    for (index, value) in ["t313-private-test", "1.910.1.530630", ""]
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

fn gateway_at(content: &Shipped, state: GameplayState, position: [f32; 3]) -> Gateway {
    let mut gateway = Gateway::load_gameplay(
        &reference(),
        character_at(position),
        &content.0,
        Some(state),
    )
    .unwrap();
    login(&mut gateway);
    gateway
}

fn save_bytes(gateway: &Gateway) -> Vec<u8> {
    serde_json::to_vec(&gateway.snapshot()).unwrap()
}

fn inner(replies: &[Vec<u8>]) -> Vec<Vec<u8>> {
    replies
        .iter()
        .map(|packet| {
            assert!(
                packet.len() >= 7 && packet[..3] == [5, 0, 1],
                "a gateway reply is a tunneled application packet"
            );
            packet[7..].to_vec()
        })
        .collect()
}

fn bodies(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    inner(&outcome.replies)
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

fn offer_instance(replies: &[Vec<u8>]) -> i32 {
    replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .map(|packet| i32::from_le_bytes(packet[3..7].try_into().unwrap()))
        .expect("the derby offer")
}

fn start_request(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 5];
    bytes.extend(instance.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

fn register_for(key: ParticipantKey) -> Vec<u8> {
    encode_inner(&DerbyMessage::RegisterPlayer {
        key,
        unknown_key: 0,
    })
    .unwrap()
}

fn phase_for(key: ParticipantKey, phase: i32) -> Vec<u8> {
    encode_inner(&DerbyMessage::PlayerPhase { key, phase }).unwrap()
}

fn countdown_for(key: ParticipantKey, count: u32) -> Vec<u8> {
    encode_inner(&DerbyMessage::CountdownStatus { key, count }).unwrap()
}

fn pose(key: ParticipantKey, position: [f32; 3]) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: [
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            position[0],
            position[1],
            position[2],
            1.0,
        ]
        .map(f32::to_bits),
        vector_a_bits: [0; 4],
        vector_b_bits: [0; 4],
        controls_bits: [0; 3],
        flags: [0; 4],
        sequence: 1,
    }))
    .unwrap()
}

fn hit_for(reporter: ParticipantKey, other: ParticipantKey, impact: f32) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter,
        other,
        impact_bits: impact.to_bits(),
    }))
    .unwrap()
}

/// Open the configured driver menu through the real gateway and take the derby option.
fn enter_derby(gateway: &mut Gateway, driver: u64, base: u64) -> i32 {
    let menu = bodies(&click(gateway, driver, base));
    let option = menu_option(&menu, DERBY_LABEL).expect("the driver offers the derby");
    let entered = choose(gateway, driver, option, base + 100);
    assert!(
        entered.gameplay_changed,
        "the offered choice starts a match"
    );
    offer_instance(&bodies(&entered))
}

/// Open the configured driver menu through `Gameplay` (no outer dispatch) and take the option.
fn enter_derby_gameplay(game: &mut Gameplay, driver: u64, base: u64) -> i32 {
    let character = character_at(npc_position(driver));
    let menu = game
        .handle_in_world(&npc_click(driver), &character, base)
        .unwrap();
    let option = menu_option(&menu.replies, DERBY_LABEL).expect("the driver offers the derby");
    let entered = game
        .handle_in_world(&npc_select(driver, option), &character, base + 100)
        .unwrap();
    assert!(entered.changed, "the offered choice starts a match");
    offer_instance(&entered.replies)
}

/// Send the native 39/5 Go, register the offered match and drive the client-owned readiness gates
/// and countdown to Running. Returns the local key and the seven server-created NPC keys.
fn start_and_ready(
    gateway: &mut Gateway,
    instance: i32,
    base: u64,
) -> (ParticipantKey, Vec<ParticipantKey>) {
    let key = ParticipantKey(gateway.state().guid);
    let started = bodies(
        &gateway
            .handle_at(&wrap(&start_request(instance)), base + 300)
            .unwrap(),
    );
    assert!(started.iter().any(|packet| packet[..3] == [39, 0, 17]));
    assert!(
        started
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 20])),
        "the native Go sends the type-11 active carrier"
    );
    let setup = bodies(
        &gateway
            .handle_at(&wrap(&register_for(key)), base + 400)
            .unwrap(),
    );
    let npcs: Vec<_> = setup
        .iter()
        .filter(|packet| packet.starts_with(&[34, 0, 10]))
        .filter_map(|packet| match decode_inner(packet) {
            Ok(DerbyMessage::CreateVehicle(vehicle)) if vehicle.key != key => Some(vehicle.key),
            _ => None,
        })
        .collect();
    assert_eq!(npcs.len(), 7, "the shipped max_drivers fills seven NPCs");
    for value in 3..=8 {
        let out = bodies(
            &gateway
                .handle_at(&wrap(&phase_for(key, value)), base + 1_000 + value as u64)
                .unwrap(),
        );
        assert!(!out.is_empty(), "phase {value}");
    }
    for value in (0..=4).rev() {
        let out = bodies(
            &gateway
                .handle_at(
                    &wrap(&countdown_for(key, value)),
                    base + 2_000 + (4 - value) as u64,
                )
                .unwrap(),
        );
        assert!(!out.is_empty(), "countdown {value}");
    }
    assert_eq!(
        gateway.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Running)
    );
    (key, npcs)
}

/// Knock out all seven NPC drivers, wait out their respawn, then knock out four more for the
/// recovered first-to-22 points. Returns the results record's clock and replies.
fn drive_to_results(
    gateway: &mut Gateway,
    key: ParticipantKey,
    npcs: &[ParticipantKey],
    base: u64,
) -> (u64, Vec<Vec<u8>>) {
    gateway
        .handle_at(&wrap(&pose(key, [0.0, 0.0, 0.0])), base + 2_900)
        .unwrap();
    for (index, npc) in npcs.iter().enumerate() {
        let out = gateway
            .handle_at(
                &wrap(&hit_for(key, *npc, 100.0)),
                base + 3_000 + index as u64 * 100,
            )
            .unwrap();
        assert!(
            out.gameplay_changed,
            "the contact is resolved by the server"
        );
        assert_eq!(
            gateway.gameplay().unwrap().derby_phase(),
            Some(DerbyPhase::Running),
            "the first seven knockouts stay below the 22-point target"
        );
    }
    let respawn_at = base + 8_000;
    gateway
        .handle_at(&wrap(&pose(key, [0.0, 0.0, 0.0])), respawn_at - 100)
        .unwrap();
    let respawn = gateway.tick_at(respawn_at).unwrap();
    assert!(
        inner(&respawn.replies)
            .iter()
            .any(|packet| packet.starts_with(&[34, 0, 18])),
        "the KO'd drivers reset"
    );
    for (index, npc) in npcs.iter().take(4).enumerate() {
        let out = gateway
            .handle_at(
                &wrap(&hit_for(key, *npc, 100.0)),
                respawn_at + 100 + index as u64 * 100,
            )
            .unwrap();
        if index == 3 {
            assert_eq!(
                gateway.gameplay().unwrap().derby_phase(),
                Some(DerbyPhase::Results)
            );
            return (respawn_at + 400, out.replies);
        }
    }
    panic!("the shipped 22-point target was not reached");
}

fn completion_bundle(replies: &[Vec<u8>]) -> Vec<u8> {
    inner(replies)
        .into_iter()
        .find(|packet| packet.starts_with(&[50, 0, 1]))
        .expect("the completion reward bundle")
}

#[test]
fn the_shipped_package_loads_and_offers_from_all_four_drivers() {
    // The real world load comes first: deleting the package must fail here because no driver
    // offers the derby, not because a test check noticed a missing file.
    let content = Shipped::new();
    let state = Gameplay::load(&content.0, None).unwrap().state();
    for driver in [MAD_JACK, CRASH, BURT, RIP] {
        let mut gateway = gateway_at(&content, state.clone(), npc_position(driver));
        let menu = bodies(&click(&mut gateway, driver, NOW));
        let option = menu_option(&menu, DERBY_LABEL)
            .unwrap_or_else(|| panic!("driver {driver} offers the derby"));
        let entered = choose(&mut gateway, driver, option, NOW + 100);
        assert!(entered.gameplay_changed, "driver {driver}");
        assert!(
            bodies(&entered)
                .iter()
                .any(|packet| packet.starts_with(&[39, 0, 16])),
            "driver {driver} sends the native 39/16 offer"
        );
        assert!(gateway.gameplay().unwrap().derby_active());
        // Retire the offer before the next driver so every gate is entered exactly once.
        let retired = gateway
            .handle_at(&wrap(&[26, 0, 42, 0]), NOW + 200)
            .unwrap();
        assert!(retired.gameplay_changed);
        assert!(!gateway.gameplay().unwrap().derby_active());
    }

    // The production loader accepts the shipped package and the four-id gate list.
    let shipped = DerbyGateway::load(&data(), None)
        .expect("the shipped package respects every loader bound")
        .expect("the shipped derby package exists");
    assert_eq!(shipped.gate_npcs(), &[MAD_JACK, CRASH, BURT, RIP][..]);
    assert_eq!(shipped.menu_string_id(), DERBY_LABEL);
    let gates: DerbyGates =
        serde_json::from_slice(&fs::read(data().join("derby_gates.json")).unwrap()).unwrap();
    assert_eq!(gates.npcs, vec![MAD_JACK, CRASH, BURT, RIP]);
    let mut interaction =
        NpcInteraction::load(&data().join("npcs"), &data().join("stores.json")).unwrap();
    assert!(
        interaction
            .set_derby_gates(shipped.gate_npcs(), shipped.menu_string_id())
            .is_empty(),
        "every shipped gate id is an NPC content record"
    );
}

#[test]
fn a_finished_match_pays_the_flat_reward_once() {
    let content = Shipped::new();
    let state = Gameplay::load(&content.0, None).unwrap().state();
    let mut gateway = gateway_at(&content, state, npc_position(MAD_JACK));
    let before = gateway.snapshot().gameplay.unwrap();
    let before_coins = before.inventory.coins;
    let before_job = before.profiles.as_ref().unwrap().active;
    assert_ne!(before_job, DEMO_DERBY_DRIVER_PROFILE);

    let instance = enter_derby(&mut gateway, MAD_JACK, NOW);
    assert_eq!(
        gateway.snapshot().gameplay.unwrap().derby,
        None,
        "the live match is not a save field"
    );
    let (key, npcs) = start_and_ready(&mut gateway, instance, NOW);
    assert_eq!(
        gateway.snapshot().gameplay.unwrap().inventory.coins,
        before_coins,
        "playing a match is not paying"
    );
    let (finished_at, results) = drive_to_results(&mut gateway, key, &npcs, NOW);
    assert!(
        inner(&results)
            .iter()
            .filter(|packet| packet.starts_with(&[34, 0, 1]))
            .any(|packet| decode_inner(packet) == Ok(DerbyMessage::GameState(10))),
        "the results state"
    );

    let paid = gateway.snapshot().gameplay.unwrap();
    assert_eq!(
        paid.inventory.coins,
        before_coins + COMPLETION_COINS,
        "one flat completion payout"
    );
    assert_eq!(
        paid.profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&DEMO_DERBY_DRIVER_PROFILE),
        Some(&COMPLETION_XP),
        "the completion pays the Demo Derby Driver XP, not the prior job"
    );
    assert_eq!(
        paid.derby.as_ref().unwrap().paid_matches,
        vec![instance],
        "the paid match instance is recorded"
    );
    let bundle = completion_bundle(&results);
    assert_eq!(
        i32::from_le_bytes(bundle[4..8].try_into().unwrap()),
        COMPLETION_COINS as i32,
        "the 50/1 bundle carries the content coin amount"
    );
    assert_eq!(
        i32::from_le_bytes(bundle[8..12].try_into().unwrap()),
        COMPLETION_XP as i32,
        "the 50/1 bundle carries the content XP amount"
    );
    let coin_update = inner(&results)
        .into_iter()
        .find(|packet| packet.starts_with(&[38, 0, 19]))
        .expect("the coin update");
    assert_eq!(
        i32::from_le_bytes(coin_update[4..8].try_into().unwrap()),
        (before_coins + COMPLETION_COINS) as i32
    );

    // A replay of the record that drove the match to results cannot pay again.
    let replay = gateway
        .handle_at(&wrap(&hit_for(key, npcs[0], 100.0)), finished_at + 100)
        .unwrap();
    assert!(!replay.gameplay_changed);
    assert!(replay.replies.is_empty());
    assert_eq!(
        gateway.snapshot().gameplay.as_ref(),
        Some(&paid),
        "a repeated results record pays nothing"
    );

    let close_ready = gateway.tick_at(finished_at + 10_000).unwrap();
    assert!(
        bodies(&close_ready)
            .iter()
            .filter(|packet| packet.starts_with(&[34, 0, 1]))
            .any(|packet| decode_inner(packet) == Ok(DerbyMessage::GameState(11))),
        "the results hold enables Close"
    );
    let closed = gateway
        .handle_at(&wrap(&[26, 0, 42, 0]), finished_at + 10_100)
        .unwrap();
    assert!(closed.gameplay_changed);
    let after = gateway.snapshot().gameplay.unwrap();
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert_eq!(
        after.profiles.as_ref().unwrap().active,
        before_job,
        "Close restores the job active before entry"
    );
    assert_eq!(after.inventory.coins, before_coins + COMPLETION_COINS);
    assert_eq!(
        after.derby.as_ref().unwrap().paid_matches,
        vec![instance],
        "the close adds no ledger row"
    );
    assert!(
        !inner(&closed.replies)
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "the close sends no reward bundle"
    );

    // A replay of the same instance's close pays nothing.
    let replay_close = gateway
        .handle_at(&wrap(&[26, 0, 42, 0]), finished_at + 10_200)
        .unwrap();
    assert!(!replay_close.gameplay_changed);
    assert_eq!(
        gateway.snapshot().gameplay.as_ref(),
        Some(&after),
        "a repeated close changes nothing"
    );
}

#[test]
fn the_paid_ledger_and_prior_job_survive_a_reconnect() {
    let content = Shipped::new();
    let state = Gameplay::load(&content.0, None).unwrap().state();
    let before = state.clone();
    let before_coins = before.inventory.coins;
    let before_job = before.profiles.as_ref().unwrap().active;

    let mut gateway = gateway_at(&content, state, npc_position(MAD_JACK));
    let instance = enter_derby(&mut gateway, MAD_JACK, NOW);
    let (key, npcs) = start_and_ready(&mut gateway, instance, NOW);
    let _ = drive_to_results(&mut gateway, key, &npcs, NOW);
    let paid = gateway.snapshot();
    assert_eq!(
        paid.gameplay.as_ref().unwrap().inventory.coins,
        before_coins + COMPLETION_COINS
    );

    // The save encoding (not just the live struct) carries the ledger and the prior job.
    let encoded = serde_json::to_vec(&paid).unwrap();
    let reloaded: Snapshot = serde_json::from_slice(&encoded).unwrap();
    let saved = reloaded.gameplay.unwrap();
    assert_eq!(saved.derby.as_ref().unwrap().paid_matches, vec![instance]);
    assert_eq!(saved.profiles.as_ref().unwrap().active, before_job);

    let mut game = Gameplay::load(&content.0, Some(saved)).unwrap();
    assert_eq!(game.active_profile(), Some(before_job));
    assert!(!game.derby_active());
    assert_eq!(
        game.state().inventory.coins,
        before_coins + COMPLETION_COINS
    );

    // A fresh match resumes the instance counter above the paid ledger.
    let next = enter_derby_gameplay(&mut game, MAD_JACK, NOW + 20_000);
    assert!(next > instance, "a paid instance id is never reissued");
    assert_eq!(
        game.state().derby.as_ref().unwrap().paid_matches,
        vec![instance],
        "starting a match does not touch the ledger"
    );
}

#[test]
fn a_refused_entry_changes_nothing() {
    // A real NPC that is not a gate lists no derby option and changes nothing.
    let content = Shipped::new();
    let mut gateway = gateway_at(
        &content,
        Gameplay::load(&content.0, None).unwrap().state(),
        npc_position(MAD_JACK),
    );
    let before = save_bytes(&gateway);
    let _ = gateway
        .handle_at(&wrap(&npc_click(NOT_A_GATE)), NOW)
        .unwrap();
    let _ = gateway
        .handle_at(&wrap(&npc_select(NOT_A_GATE, 2)), NOW + 100)
        .unwrap();
    assert_eq!(
        save_bytes(&gateway),
        before,
        "a non-gate refusal changes no save state"
    );
    assert!(
        menu_option(
            &bodies(&click(&mut gateway, MAD_JACK, NOW + 200)),
            DERBY_LABEL
        )
        .is_some(),
        "the driver menu still opens after a non-gate refusal"
    );

    // A live Mining run closes ordinary NPC interaction and refuses the driver; the state is
    // byte-identical, and a clean session still gets the shipped offer.
    let busy_content = Shipped::new();
    mining::write_content(&busy_content.0, mining::MINING_GAME_ID).unwrap();
    let mut state = Gameplay::load(&busy_content.0, None).unwrap().state();
    let mut engine = Type8Engine::load(
        &busy_content.0,
        mining::CONTENT_FILE,
        mining::MINING_GAME_ID,
        mining::FIRST_MINING_INSTANCE,
        None,
    )
    .unwrap();
    engine.begin(mining::MINER_PROFILE_ID).unwrap();
    assert!(engine.active());
    state.mining = Some(engine.state().clone());
    let mut busy = Gameplay::load(&busy_content.0, Some(state)).unwrap();
    let character = character_at(npc_position(MAD_JACK));
    let before = serde_json::to_vec(&busy.state()).unwrap();
    let menu = busy
        .handle_in_world(&npc_click(MAD_JACK), &character, NOW)
        .unwrap();
    assert!(
        !menu
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "a busy player is not offered the NPC menu"
    );
    let refused = busy
        .handle_in_world(&npc_select(MAD_JACK, 2), &character, NOW + 100)
        .unwrap();
    assert!(!refused.changed);
    assert_eq!(
        serde_json::to_vec(&busy.state()).unwrap(),
        before,
        "a busy refusal changes no save state"
    );
    let mut clean = Gameplay::load(&busy_content.0, None).unwrap();
    let menu = clean
        .handle_in_world(&npc_click(MAD_JACK), &character, NOW)
        .unwrap();
    assert!(
        menu_option(&menu.replies, DERBY_LABEL).is_some(),
        "the same driver offers the derby once the activity is free"
    );

    // Already playing: the repeated choice must not start a second match or move the save; the
    // menu works again after the offer is retired.
    let mut active = gateway_at(
        &content,
        Gameplay::load(&content.0, None).unwrap().state(),
        npc_position(MAD_JACK),
    );
    let instance = enter_derby(&mut active, MAD_JACK, NOW);
    let live = save_bytes(&active);
    let repeated = choose(&mut active, MAD_JACK, 2, NOW + 200);
    assert!(!repeated.gameplay_changed);
    assert_eq!(
        save_bytes(&active),
        live,
        "an already-playing refusal changes no save state"
    );
    assert!(active.gameplay().unwrap().derby_active());
    let _ = active.handle_at(&wrap(&[26, 0, 42, 0]), NOW + 300).unwrap();
    assert!(!active.gameplay().unwrap().derby_active());
    let menu = bodies(&click(&mut active, MAD_JACK, NOW + 400));
    let option = menu_option(&menu, DERBY_LABEL).expect("the driver menu is usable again");
    let again = choose(&mut active, MAD_JACK, option, NOW + 500);
    assert_eq!(
        offer_instance(&bodies(&again)),
        instance + 1,
        "retiring an offer does not reissue its instance id"
    );

    // Two sessions have independent matches: the second offer does not touch the first session.
    let mut first = gateway_at(
        &content,
        Gameplay::load(&content.0, None).unwrap().state(),
        npc_position(CRASH),
    );
    let mut second = gateway_at(
        &content,
        Gameplay::load(&content.0, None).unwrap().state(),
        npc_position(CRASH),
    );
    let first_instance = enter_derby(&mut first, CRASH, NOW);
    let first_bytes = save_bytes(&first);
    let second_instance = enter_derby(&mut second, CRASH, NOW);
    assert_eq!(
        first_instance, second_instance,
        "each session starts its own counter"
    );
    assert_eq!(
        save_bytes(&first),
        first_bytes,
        "another session's entry must not move this one"
    );
}

#[test]
fn malformed_content_is_refused_and_the_shipped_definition_is_real() {
    // The production loader is the first gate: setting `max_drivers` to 1 must be refused here,
    // before any content assertion below sees the bytes.
    assert!(
        DerbyGateway::load(&data(), None)
            .expect("the shipped package respects every loader bound")
            .is_some()
    );
    let bytes = fs::read(data().join("derby_entry.json")).unwrap();
    let shipped: DerbyGatewayContent = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(shipped.schema_version, 1);
    assert_eq!(shipped.game_id, 39);
    assert_eq!(shipped.rules.max_drivers, 8);
    assert_eq!(shipped.rules.score_target, Some(22));
    assert_eq!(shipped.rules.time_limit_ms, None);
    assert_eq!(shipped.completion_reward_coins, COMPLETION_COINS);
    assert_eq!(shipped.completion_reward_xp, COMPLETION_XP);
    assert!(shipped.reward.is_none());
    assert!(shipped.pickups.is_empty());
    assert_eq!(
        shipped.vehicle.name,
        b"Demo Derby Car".to_vec(),
        "the name names the derby car, not a player"
    );
    let definition = &shipped.vehicle.definition;
    assert_eq!(definition.len(), DEFINITION_LEN);
    assert_eq!(
        format!("{:x}", Sha256::digest(definition)),
        DEFINITION_SHA256,
        "the shipped definition is the recovered client byte sequence"
    );
    assert_ne!(
        &definition[..],
        &[1u8, 2, 3, 4][..],
        "not the synthetic fixture"
    );
    let mut prefix = (BODY_ASSET.len() as u32).to_le_bytes().to_vec();
    prefix.extend(BODY_ASSET.as_bytes());
    prefix.extend(400.0_f32.to_le_bytes());
    assert_eq!(
        &definition[..prefix.len()],
        &prefix[..],
        "ReadVehicleDefinition 0x009d7b50 reads the body asset then the captured 400.0 float"
    );
    assert_eq!(
        &shipped.vehicle.start_transform_bits, &[0; 16],
        "the start pose is not separately recovered (R)"
    );

    // Every malformed package is refused at load, and the directory is left untouched.
    let package = Package::new();
    let entry = package.path().join("derby_entry.json");
    let gates_path = package.path().join("derby_gates.json");
    let original_entry = fs::read(&entry).unwrap();
    let original_gates = fs::read(&gates_path).unwrap();
    let assert_refused = |mutate: &dyn Fn(&mut Value)| {
        let mut value = read_json(&entry);
        mutate(&mut value);
        fs::write(&entry, serde_json::to_vec(&value).unwrap()).unwrap();
        let before = fs::read(&entry).unwrap();
        assert!(
            DerbyGateway::load(package.path(), None).is_err(),
            "the malformed package must be refused"
        );
        assert_eq!(
            fs::read(&entry).unwrap(),
            before,
            "a refused load does not rewrite the package"
        );
        fs::write(&entry, &original_entry).unwrap();
        assert_eq!(fs::read(&entry).unwrap(), original_entry);
    };

    assert_refused(&|value| value["rules"]["contact_damage_per_unit"] = json!(1e39));
    assert_refused(&|value| value["rules"]["max_drivers"] = json!(1));
    assert_refused(&|value| {
        value["rules"]
            .as_object_mut()
            .unwrap()
            .remove("start_health");
    });
    assert_refused(&|value| {
        value["vehicle"]
            .as_object_mut()
            .unwrap()
            .remove("definition");
    });
    assert_refused(&|value| value["unexpected_field"] = json!(true));

    // A duplicate gate list is refused by the same loader.
    let mut gates = read_json(&gates_path);
    gates["npcs"] = json!([MAD_JACK, MAD_JACK, CRASH, BURT, RIP]);
    fs::write(&gates_path, serde_json::to_vec(&gates).unwrap()).unwrap();
    let before_gates = fs::read(&gates_path).unwrap();
    assert!(DerbyGateway::load(package.path(), None).is_err());
    assert_eq!(
        fs::read(&gates_path).unwrap(),
        before_gates,
        "a refused gate list does not rewrite the file"
    );
    fs::write(&gates_path, &original_gates).unwrap();

    // The restored shipped package loads again and is unchanged.
    let restored = DerbyGateway::load(package.path(), None)
        .unwrap()
        .expect("the restored package");
    assert_eq!(restored.gate_npcs(), &[MAD_JACK, CRASH, BURT, RIP][..]);
    assert_eq!(restored.menu_string_id(), DERBY_LABEL);
    assert_eq!(fs::read(&entry).unwrap(), original_entry);
    assert_eq!(fs::read(&gates_path).unwrap(), original_gates);

    // A fifth gate id that is not NPC content is not listed; no fifth arena is invented.
    let mut interaction =
        NpcInteraction::load(&data().join("npcs"), &data().join("stores.json")).unwrap();
    let mut npcs = vec![MAD_JACK, CRASH, BURT, RIP];
    npcs.push(1);
    let unknown = interaction.set_derby_gates(&npcs, DERBY_LABEL);
    assert_eq!(unknown, vec![1]);
}
