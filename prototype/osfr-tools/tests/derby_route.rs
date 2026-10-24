//! Entry-to-close coverage for the routed demolition derby.
//!
//! The package is synthetic and intentionally small. It exercises the real native opcode 34
//! codec, the real 39/16 offer -> 39/5 Go exchange, the client phase/countdown acknowledgements,
//! the results hold, the completion payout, cancellation and save compatibility. All timing and
//! combat values are reconstructed test policy (R). T219 pays a flat completion reward when the
//! server-observed match reaches its results phase; contacts, damage and score never change the
//! amount, and the paid-match ledger pays each instance once.

use osfr_tools::{
    character_store::CharacterState,
    derby::{
        DerbyConfig, DerbyMessage, DerbyPhase, DerbyRules, DerbyVehicleTemplate, ParticipantKey,
        VehicleHitReport, decode_inner, encode_inner,
    },
    derby_gateway::{
        DEMO_DERBY_DRIVER_PROFILE, DerbyGates, DerbyGateway, DerbyGatewayContent,
        DerbyGatewayState, FIRST_DERBY_INSTANCE, MAX_COMPLETION_REWARD, MAX_PAID_MATCHES,
    },
    game_store,
    gameplay::{Gameplay, GameplayState},
    harvesting,
    mining::Type8Engine,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
#[path = "support/sc_earn.rs"]
mod sc_earn;

const OWNER: ParticipantKey = ParticipantKey(0x1000_0000_0000_0001);
const OTHER: ParticipantKey = ParticipantKey(0x1000_0000_0000_0002);
const GATE: u64 = 9_000_000_301;
const INSTANCE_GAME: i32 = 39;
/// (R) Synthetic package payout, small enough for the fixture wallet.
const COMPLETION_COINS: u32 = 25;
/// (R) Synthetic Demo Derby Driver XP for the same completion.
const COMPLETION_XP: u32 = 10;
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-derby-route-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let fixture = Self { path };
        fixture.write_inventory();
        fixture.write_content();
        fixture
    }

    fn write_inventory(&self) {
        let coins = 10_i32;
        let content = json!({
            "schema_version": 1,
            "player_guid": OWNER.raw(),
            "default_state": {
                "coins": coins,
                "items": [],
                "equipment": [],
                "next_instance_id": null
            },
            "profiles": [],
            "spans": [
                span("coins", 13, 0, &coins.to_le_bytes()),
                span("items", 17, 0, &0_i32.to_le_bytes())
            ],
            "definitions": []
        });
        fs::write(
            self.path.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
    }

    fn write_content(&self) {
        fs::write(
            self.path.join("derby_entry.json"),
            serde_json::to_vec(&content()).unwrap(),
        )
        .unwrap();
        fs::write(
            self.path.join("derby_gates.json"),
            serde_json::to_vec(&DerbyGates {
                schema_version: 1,
                menu_string_id: 3053,
                npcs: vec![GATE],
                reward_cooldown_s: None,
            })
            .unwrap(),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct GameplayFixture {
    path: PathBuf,
}

impl GameplayFixture {
    fn new() -> Self {
        Self::build(false)
    }

    fn with_opening() -> Self {
        Self::build(true)
    }

    fn build(with_opening: bool) -> Self {
        let source = std::env::var_os("OSFR_DERBY_ROUTE_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/build/w104/scratch/content")
            });
        assert!(
            source.join("inventory.json").is_file() && source.join("profiles.json").is_file(),
            "derby route fixture needs a prepared gameplay content directory: {}",
            source.display()
        );
        let path = std::env::temp_dir().join(format!(
            "osfr-derby-gameplay-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        for file in [
            "inventory.json",
            "profiles.json",
            "categories.json",
            "combat.json",
            "combat-template.bin",
            "quests.json",
            "quest-journal.bin",
            "zones.json",
        ] {
            let source_file = source.join(file);
            if source_file.is_file() {
                fs::copy(&source_file, path.join(file)).unwrap();
            }
        }
        if with_opening {
            for entry in fs::read_dir(&source).unwrap() {
                let source_file = entry.unwrap().path();
                let extension = source_file.extension().and_then(|value| value.to_str());
                if source_file.is_file()
                    && matches!(extension, Some("json" | "bin"))
                    && source_file.file_name().and_then(|value| value.to_str())
                        != Some("item-catalogue.json")
                {
                    fs::copy(&source_file, path.join(source_file.file_name().unwrap())).unwrap();
                }
            }
        }
        fs::create_dir(path.join("npcs")).unwrap();
        fs::write(
            path.join("npcs/synthetic.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "records": [{
                    "npc_id": GATE,
                    "name": "Synthetic derby guide",
                    "zone_id": 0,
                    "position": [0.0, 0.0, 0.0],
                    "placement": "legacy_data",
                    "model_id": 1,
                    "name_string_id": 1,
                    "roles": ["derby_guide"]
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            path.join("npc_spawns_extra.json"),
            br#"{"schema_version":1,"records":[]}"#,
        )
        .unwrap();
        fs::write(
            path.join("stores.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "marketplace": {
                    "enabled_on_handshake": false,
                    "orders_enabled": false,
                    "payment_sources_enabled": false,
                    "not_available_message": "disabled"
                },
                // T229: the earn table needs an enabled Station Cash policy and wallet.
                "station_cash": {"enabled": true},
                "items": []
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            path.join("vendors.json"),
            br#"{"schema_version":1,"vendors":[]}"#,
        )
        .unwrap();
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/sc_rewards.json"),
            path.join("sc_rewards.json"),
        )
        .unwrap();
        if !path.join("zones.json").is_file() {
            fs::write(
                path.join("zones.json"),
                br#"{"schema_version":1,"records":[{"id":0,"name":"Synthetic zone"}]}"#,
            )
            .unwrap();
        }
        let fixture = Self { path };
        fixture.write_derby_content();
        fixture
    }

    fn write_derby_content(&self) {
        fs::write(
            self.path.join("derby_entry.json"),
            serde_json::to_vec(&content()).unwrap(),
        )
        .unwrap();
        fs::write(
            self.path.join("derby_gates.json"),
            serde_json::to_vec(&DerbyGates {
                schema_version: 1,
                menu_string_id: 3053,
                npcs: vec![GATE],
                reward_cooldown_s: None,
            })
            .unwrap(),
        )
        .unwrap();
    }
}

impl Drop for GameplayFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn character() -> CharacterState {
    CharacterState {
        guid: OWNER.raw(),
        zone_id: 0,
        position: [10.0, 20.0, 30.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

/// The routed NPC gate is at the origin and the interaction radius is 12 m, so a routed
/// character stands there (the `character()` helper above stays far away for gateway-only tests).
fn routed_character() -> CharacterState {
    CharacterState {
        guid: OWNER.raw(),
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    }
}

fn content() -> DerbyGatewayContent {
    DerbyGatewayContent {
        schema_version: 1,
        game_id: INSTANCE_GAME,
        vehicle: DerbyVehicleTemplate {
            name: b"synthetic-derby".to_vec(),
            definition: vec![1, 2, 3, 4],
            start_transform_bits: [0; 16],
            unknown_flag: false,
            unknown_value_bits: 0,
        },
        rules: DerbyRules {
            max_drivers: 8,
            start_health: 20,
            contact_damage_per_unit: 1.0,
            contact_cooldown_ms: 100,
            contact_radius_m: 5.0,
            max_impact: 50.0,
            max_pose_age_ms: 5_000,
            max_damage_per_contact: 100,
            lobby_timeout_ms: 120_000,
            pickup_radius: 3.0,
            health_pickup: 5,
            boost_pickup: 5.0,
            shield_duration_ms: 1_000,
            attack_duration_ms: 1_000,
            ultimate_duration_ms: 1_000,
            attack_multiplier: 2.0,
            ultimate_multiplier: 2.0,
            respawn_delay_ms: 1_000,
            results_duration_ms: 1_000,
            knockout_points: 2,
            leader_knockout_points: 3,
            knockout_point_loss: 1,
            score_target: Some(2),
            time_limit_ms: None,
            config: DerbyConfig {
                value_a_bits: 0,
                value_b: 0,
            },
        },
        pickups: Vec::new(),
        completion_reward_coins: COMPLETION_COINS,
        completion_reward_xp: COMPLETION_XP,
        reward: None,
    }
}

fn span(kind: &str, offset: usize, profile_id: i32, bytes: &[u8]) -> serde_json::Value {
    json!({
        "kind": kind,
        "offset": offset,
        "length": bytes.len(),
        "profile_id": profile_id,
        "sha256": format!("{:x}", Sha256::digest(bytes))
    })
}

fn start_request(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 5];
    bytes.extend(instance.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

fn quit_request(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 6];
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

fn register() -> Vec<u8> {
    register_for(OWNER)
}

fn pose(key: ParticipantKey, position: [f32; 3]) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehiclePose(osfr_tools::derby::VehiclePose {
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

fn npc_menu() -> Vec<u8> {
    let mut bytes = vec![26, 0, 8, 0];
    bytes.extend(GATE.to_le_bytes());
    bytes.push(0);
    bytes
}

fn npc_choice(choice: i32) -> Vec<u8> {
    let mut bytes = vec![26, 0, 10, 0];
    bytes.extend(GATE.to_le_bytes());
    bytes.extend(choice.to_le_bytes());
    bytes
}

fn phase_for(key: ParticipantKey, phase: i32) -> Vec<u8> {
    encode_inner(&DerbyMessage::PlayerPhase { key, phase }).unwrap()
}

fn phase(phase: i32) -> Vec<u8> {
    phase_for(OWNER, phase)
}

fn countdown_for(key: ParticipantKey, count: u32) -> Vec<u8> {
    encode_inner(&DerbyMessage::CountdownStatus { key, count }).unwrap()
}

fn countdown(count: u32) -> Vec<u8> {
    countdown_for(OWNER, count)
}

fn messages(packets: &[Vec<u8>]) -> Vec<DerbyMessage> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[34, 0]))
        .map(|packet| decode_inner(packet).unwrap())
        .collect()
}

fn offer_instance(outcome: &osfr_tools::gameplay::Outcome) -> i32 {
    outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .map(|packet| i32::from_le_bytes(packet[3..7].try_into().unwrap()))
        .expect("derby offer")
}

/// Open the configured NPC menu and take the derby option; returns the offered instance id.
fn enter_derby(game: &mut Gameplay, character: &CharacterState, base: u64) -> i32 {
    let menu = game.handle_in_world(&npc_menu(), character, base).unwrap();
    assert!(menu.replies.iter().any(|packet| packet[..3] == [26, 0, 9]));
    let entered = game
        .handle_in_world(&npc_choice(2), character, base + 100)
        .unwrap();
    assert!(entered.changed);
    offer_instance(&entered)
}

/// Register the offered match and return the server-created NPC driver key.
fn register_and_npc(game: &mut Gameplay, character: &CharacterState, base: u64) -> ParticipantKey {
    let key = ParticipantKey(character.guid);
    let setup = game
        .handle_in_world(&register_for(key), character, base + 400)
        .unwrap();
    messages(&setup.replies)
        .into_iter()
        .find_map(|message| match message {
            DerbyMessage::CreateVehicle(vehicle) if vehicle.key != key => Some(vehicle.key),
            _ => None,
        })
        .expect("NPC vehicle")
}

/// Drive the client-owned phase and countdown acknowledgements to the running phase.
fn acknowledge_readiness(game: &mut Gameplay, character: &CharacterState, base: u64) {
    let key = ParticipantKey(character.guid);
    for phase_value in 3..=8 {
        let out = game
            .handle_in_world(
                &phase_for(key, phase_value),
                character,
                base + 1_000 + phase_value as u64,
            )
            .unwrap();
        assert!(!out.replies.is_empty(), "phase {phase_value}");
    }
    for value in (0..=4).rev() {
        let out = game
            .handle_in_world(
                &countdown_for(key, value),
                character,
                base + 2_000 + (4 - value) as u64,
            )
            .unwrap();
        assert!(!out.replies.is_empty(), "countdown {value}");
    }
    assert_eq!(game.derby_phase(), Some(DerbyPhase::Running));
}

/// Start the offered match and drive the client-owned readiness gates to Running; returns the
/// server-created NPC driver key.
fn run_to_running(
    game: &mut Gameplay,
    character: &CharacterState,
    instance: i32,
    base: u64,
) -> ParticipantKey {
    let started = game
        .handle_in_world(&start_request(instance), character, base + 300)
        .unwrap();
    assert!(
        started
            .replies
            .iter()
            .any(|packet| packet[..3] == [39, 0, 17])
    );
    let npc = register_and_npc(game, character, base);
    acknowledge_readiness(game, character, base);
    npc
}

fn hit_for(reporter: ParticipantKey, npc: ParticipantKey, impact: f32) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter,
        other: npc,
        impact_bits: impact.to_bits(),
    }))
    .unwrap()
}

fn hit(npc: ParticipantKey, impact: f32) -> Vec<u8> {
    hit_for(OWNER, npc, impact)
}

/// Send a fresh pose and one accepted contact per impact. The last contact must reach the results
/// phase; its replies (including the completion payout) are returned.
fn drive_to_results(
    game: &mut Gameplay,
    character: &CharacterState,
    npc: ParticipantKey,
    impacts: &[f32],
    base: u64,
) -> Vec<Vec<u8>> {
    let key = ParticipantKey(character.guid);
    game.handle_in_world(&pose(key, [0.0, 0.0, 0.0]), character, base + 2_900)
        .unwrap();
    let mut replies = Vec::new();
    for (index, impact) in impacts.iter().enumerate() {
        let out = game
            .handle_in_world(
                &hit_for(key, npc, *impact),
                character,
                base + 3_000 + index as u64 * 200,
            )
            .unwrap();
        replies = out.replies;
        if game.derby_phase() == Some(DerbyPhase::Results) {
            assert_eq!(index + 1, impacts.len(), "the match ended early");
        }
    }
    assert_eq!(game.derby_phase(), Some(DerbyPhase::Results));
    replies
}

#[test]
fn npc_entry_reaches_running_results_and_closes_once() {
    let fixture = Fixture::new();
    let mut gateway = DerbyGateway::load(&fixture.path, None).unwrap().unwrap();
    assert_eq!(gateway.gate_npcs(), &[GATE]);
    assert!(gateway.admit(OTHER.raw(), false).is_err());
    assert!(gateway.admit(GATE, true).is_err());
    assert_eq!(gateway.admit(GATE, false), Ok(()));

    let offer = gateway.begin(&character()).unwrap();
    assert_eq!(&offer[..3], &[39, 0, 16]);
    let instance = gateway.instance_id().unwrap();

    // A close before the results screen and a spoofed register change nothing.
    assert!(!gateway.close(&[26, 0, 42, 0]).unwrap());
    assert!(
        gateway
            .handle(
                &encode_inner(&DerbyMessage::RegisterPlayer {
                    key: OTHER,
                    unknown_key: 0,
                })
                .unwrap(),
                1_500,
            )
            .unwrap()
            .is_empty()
    );

    let started = gateway.start(&start_request(instance)).unwrap();
    assert_eq!(&started[0][..3], &[39, 0, 17]);
    assert_eq!(started[1][..4], [35, 0, 20, 0]);
    assert_eq!(started[1].len(), 16);
    assert_eq!(
        u32::from_le_bytes(started[1][12..16].try_into().unwrap()),
        0x801
    );

    let setup = gateway.handle(&register(), 2_000).unwrap();
    let npc = messages(&setup)
        .into_iter()
        .find_map(|message| match message {
            DerbyMessage::CreateVehicle(vehicle) if vehicle.key != OWNER => Some(vehicle.key),
            _ => None,
        })
        .expect("NPC vehicle");
    // Continue through the client-owned readiness gates and countdown.
    for phase_value in 3..=8 {
        let replies = gateway
            .handle(&phase(phase_value), 2_100 + phase_value as u64)
            .unwrap();
        assert!(!replies.is_empty(), "phase {phase_value}");
    }
    for value in (0..=4).rev() {
        let replies = gateway
            .handle(&countdown(value), 3_000 + (4 - value) as u64)
            .unwrap();
        assert!(!replies.is_empty(), "countdown {value}");
    }
    assert_eq!(
        gateway.phase(),
        Some(osfr_tools::derby::DerbyPhase::Running)
    );

    gateway
        .handle(&pose(OWNER, [0.0, 0.0, 0.0]), 3_900)
        .unwrap();
    let hit = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: OWNER,
        other: npc,
        impact_bits: 20.0_f32.to_bits(),
    }))
    .unwrap();
    let results = gateway.handle(&hit, 4_000).unwrap();
    assert!(messages(&results).contains(&DerbyMessage::GameState(10)));
    assert_eq!(
        gateway.phase(),
        Some(osfr_tools::derby::DerbyPhase::Results)
    );
    let close_ready = gateway.tick(5_000).unwrap();
    assert_eq!(messages(&close_ready), vec![DerbyMessage::GameState(11)]);

    let closed = gateway.close(&[26, 0, 42, 0]).unwrap();
    assert!(closed);
    assert_eq!(gateway.phase(), Some(osfr_tools::derby::DerbyPhase::Closed));

    let replay = gateway.close(&[26, 0, 42, 0]).unwrap();
    assert!(!replay);
}

#[test]
fn inactive_close_is_inert() {
    let fixture = Fixture::new();
    let mut gateway = DerbyGateway::load(&fixture.path, None).unwrap().unwrap();
    assert!(!gateway.close(&[26, 0, 42, 0]).unwrap());
    assert_eq!(gateway.phase(), None);
}

#[test]
fn offer_cancel_and_running_quit_retire_without_rewards_and_stale_quit_is_inert() {
    let fixture = Fixture::new();
    let mut gateway = DerbyGateway::load(&fixture.path, None).unwrap().unwrap();

    gateway.begin(&character()).unwrap();
    let cancelled = gateway.cancel(&[26, 0, 42, 0]).unwrap();
    assert!(!gateway.active());
    assert_eq!(&cancelled[0][..3], &[39, 0, 19]);

    gateway.begin(&character()).unwrap();
    let instance = gateway.instance_id().unwrap();
    gateway.start(&start_request(instance)).unwrap();
    gateway.handle(&register(), 2_000).unwrap();
    for phase_value in 3..=8 {
        gateway
            .handle(&phase(phase_value), 2_000 + phase_value as u64)
            .unwrap();
    }
    for value in (0..=4).rev() {
        gateway
            .handle(&countdown(value), 3_000 + (4 - value) as u64)
            .unwrap();
    }
    assert_eq!(
        gateway.phase(),
        Some(osfr_tools::derby::DerbyPhase::Running)
    );
    assert!(
        gateway
            .cancel(&quit_request(instance + 1))
            .unwrap()
            .is_empty()
    );
    let mut stale_ack = quit_request(instance + 1);
    stale_ack[2] = 7;
    assert!(gateway.cancel(&stale_ack).unwrap().is_empty());
    assert_eq!(
        gateway.phase(),
        Some(osfr_tools::derby::DerbyPhase::Running)
    );
    assert!(gateway.cancel(&[39, 0, 6, 0]).unwrap().is_empty());
    let quit = gateway.cancel(&quit_request(instance)).unwrap();
    assert_eq!(&quit[0][..3], &[39, 0, 19]);
    assert!(!gateway.active());
    assert!(!gateway.close(&[26, 0, 42, 0]).unwrap());
}

/// Write a real save document through the production envelope reader with one predecessor derby
/// shape, then load it the same way a reconnect does.
fn load_saved_state(
    fixture: &GameplayFixture,
    state: &osfr_tools::gameplay::GameplayState,
    derby: serde_json::Value,
    active: i32,
) -> Gameplay {
    try_load_saved_state(fixture, state, derby, active).unwrap()
}

fn try_load_saved_state(
    fixture: &GameplayFixture,
    state: &osfr_tools::gameplay::GameplayState,
    derby: serde_json::Value,
    active: i32,
) -> io::Result<Gameplay> {
    let character = character();
    let mut value = serde_json::to_value(state).unwrap();
    value["profiles"]["active"] = json!(active);
    value["derby"] = derby;
    let path = fixture.path.join("predecessor.save");
    fs::write(
        &path,
        serde_json::to_vec(&json!({
            "schema_version": 7,
            "character": character,
            "gameplay": value,
        }))
        .unwrap(),
    )
    .unwrap();
    let snapshot = game_store::load_or_create(&path, character).unwrap();
    Gameplay::load(&fixture.path, snapshot.gameplay)
}

#[test]
fn predecessor_and_paid_epoch_derby_saves_load_and_restore_the_job() {
    let fixture = GameplayFixture::new();
    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let character = CharacterState {
        guid: OWNER.raw(),
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    };
    let before_job = game.active_profile().unwrap();
    game.handle_in_world(&npc_menu(), &character, 1_000)
        .unwrap();
    game.handle_in_world(&npc_choice(2), &character, 1_100)
        .unwrap();
    assert!(game.derby_active());
    let live = game.state();

    // T023 shape: the save still names the temporary driver as active and carries the job to
    // restore. Loading must succeed and put the profile back on the prior job.
    let legacy = load_saved_state(
        &fixture,
        &live,
        json!({"previous_job": before_job}),
        DEMO_DERBY_DRIVER_PROFILE,
    );
    assert_eq!(legacy.active_profile(), Some(before_job));
    assert!(!legacy.derby_active());

    // F117 shape: a paid-run epoch with the profile already restored. The field still parses.
    let paid = load_saved_state(
        &fixture,
        &live,
        json!({"last_paid_unix": 1_000_000}),
        before_job,
    );
    assert_eq!(paid.active_profile(), Some(before_job));
    assert!(!paid.derby_active());
}

#[test]
fn a_predecessor_job_the_profile_content_rejects_is_skipped() {
    let fixture = GameplayFixture::new();
    let base = Gameplay::load(&fixture.path, None).unwrap();
    let active = base.active_profile().unwrap();
    assert_ne!(active, DEMO_DERBY_DRIVER_PROFILE);
    for previous_job in [999_999, 0] {
        // Neither id is in this fixture's profiles.json and 0 is malformed. The old restore
        // returned Err out of Gameplay::load, so one stale marker bricked the character.
        let game = load_saved_state(
            &fixture,
            &base.state(),
            json!({"previous_job": previous_job}),
            active,
        );
        assert_eq!(
            game.active_profile(),
            Some(active),
            "a rejected previous job {previous_job} keeps the saved active job"
        );
        assert!(!game.derby_active());
    }
}

/// F206 review: T023 unlocked the pre-derby job as well as naming it in the marker. If the
/// operator retires that job from `profiles.json`, the save must still load: only the marker-named
/// id and its experience are dropped, and the restore keeps the saved active job.
#[test]
fn a_previous_job_removed_from_content_still_loads_the_t023_save() {
    let fixture = GameplayFixture::new();
    let base = Gameplay::load(&fixture.path, None).unwrap();
    let mut state = base.state();
    let path = fixture.path.join("profiles.json");
    let mut content: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let retired = content["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|profile| profile["id"].as_i64())
        .map(|id| id as i32)
        .find(|id| {
            !state.profiles.as_ref().unwrap().unlocked.contains(id)
                && *id != DEMO_DERBY_DRIVER_PROFILE
        })
        .unwrap();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.unlocked.insert(retired);
    profiles.unlocked.insert(DEMO_DERBY_DRIVER_PROFILE);
    profiles.experience.insert(retired, 250);
    content["profiles"] = serde_json::Value::Array(
        content["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|profile| profile["id"].as_i64() != Some(i64::from(retired)))
            .cloned()
            .collect(),
    );
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();

    let game = load_saved_state(
        &fixture,
        &state,
        json!({"previous_job": retired}),
        DEMO_DERBY_DRIVER_PROFILE,
    );
    assert_eq!(game.active_profile(), Some(DEMO_DERBY_DRIVER_PROFILE));
    assert!(!game.derby_active(), "a save marker is never a live match");
    let visible = game.public_look().unwrap().profiles;
    assert!(
        !visible.iter().any(|(job, _)| *job == retired),
        "the retired job is dropped from the saved unlocks"
    );
    assert!(
        visible
            .iter()
            .any(|(job, _)| *job == DEMO_DERBY_DRIVER_PROFILE),
        "the saved active job stays"
    );
    let saved = game.state().profiles.unwrap();
    assert!(
        !saved.unlocked.contains(&retired) && !saved.experience.contains_key(&retired),
        "the next save carries the repaired snapshot, so a reconnect does not repair again"
    );
}

/// F206 review: a T023 save has the temporary Derby Driver active. When the marker's job is in
/// content but locked, the load keeps the saved job and logs the skip, without unlocking it.
#[test]
fn a_t023_save_keeps_the_temporary_job_when_the_previous_job_is_locked() {
    let fixture = GameplayFixture::new();
    let base = Gameplay::load(&fixture.path, None).unwrap();
    let mut state = base.state();
    let content: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.path.join("profiles.json")).unwrap()).unwrap();
    let unlocked = state.profiles.as_ref().unwrap().unlocked.clone();
    let locked = content["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|profile| profile["id"].as_i64())
        .map(|id| id as i32)
        .find(|id| !unlocked.contains(id) && *id != DEMO_DERBY_DRIVER_PROFILE)
        .unwrap();
    state
        .profiles
        .as_mut()
        .unwrap()
        .unlocked
        .insert(DEMO_DERBY_DRIVER_PROFILE);

    let game = load_saved_state(
        &fixture,
        &state,
        json!({"previous_job": locked}),
        DEMO_DERBY_DRIVER_PROFILE,
    );
    assert_eq!(
        game.active_profile(),
        Some(DEMO_DERBY_DRIVER_PROFILE),
        "the logged skip keeps the job the save selected"
    );
    assert!(!game.derby_active());
    let visible = game.public_look().unwrap().profiles;
    assert!(
        !visible.iter().any(|(job, _)| *job == locked),
        "the skipped job is not unlocked by the repair"
    );
    assert!(
        visible
            .iter()
            .any(|(job, _)| *job != DEMO_DERBY_DRIVER_PROFILE),
        "another unlocked job stays selectable"
    );
}

/// The repair is scoped to the marker-named id: an unrelated unknown unlock still fails the load.
#[test]
fn the_retired_job_repair_leaves_other_unknown_unlocks_fail_closed() {
    let fixture = GameplayFixture::new();
    let base = Gameplay::load(&fixture.path, None).unwrap();
    let mut state = base.state();
    let content: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.path.join("profiles.json")).unwrap()).unwrap();
    let unlocked = state.profiles.as_ref().unwrap().unlocked.clone();
    let marker = content["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|profile| profile["id"].as_i64())
        .map(|id| id as i32)
        .find(|id| !unlocked.contains(id) && *id != DEMO_DERBY_DRIVER_PROFILE)
        .unwrap();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.unlocked.insert(DEMO_DERBY_DRIVER_PROFILE);
    // Not the marker-named job, so the repair must not drop it.
    profiles.unlocked.insert(999_998);

    let error = try_load_saved_state(
        &fixture,
        &state,
        json!({"previous_job": marker}),
        DEMO_DERBY_DRIVER_PROFILE,
    )
    .err()
    .expect("an unrelated unknown unlock still fails the snapshot check");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{error}");
}

/// F117 content carried a `reward` row and `reward_cooldown_s`. F164 removed both; the decoder
/// must still accept an older operator package so a stale key cannot fail the whole world load.
#[test]
fn predecessor_content_keys_load_instead_of_failing_the_world() {
    let fixture = Fixture::new();
    for (file, key, value) in [
        ("derby_entry.json", "reward", json!({"coins": 5})),
        ("derby_gates.json", "reward_cooldown_s", json!(600)),
    ] {
        let path = fixture.path.join(file);
        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document[key] = value;
        fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    }
    let mut gateway = DerbyGateway::load(&fixture.path, None)
        .expect("F117 content must not fail the world load")
        .expect("the derby route stays enabled");
    assert_eq!(gateway.gate_npcs(), &[GATE]);
    assert_eq!(gateway.admit(GATE, false), Ok(()));
    gateway.begin(&character()).unwrap();
    assert!(gateway.active());
}

#[test]
fn busy_derby_refusal_keeps_the_exact_menu_for_a_retry() {
    let fixture = GameplayFixture::with_opening();
    let base = Gameplay::load(&fixture.path, None).unwrap();
    let mut state = base.state();
    let mut harvesting = Type8Engine::load(
        &fixture.path,
        harvesting::CONTENT_FILE,
        111,
        harvesting::FIRST_INSTANCE_ID,
        None,
    )
    .unwrap();
    harvesting.begin(45).unwrap();
    state.harvesting = Some(harvesting.state().clone());
    let mut game = Gameplay::load(&fixture.path, Some(state)).unwrap();
    let character = CharacterState {
        guid: OWNER.raw(),
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    };
    let before = game.state();

    game.handle_in_world(&npc_menu(), &character, 1_000)
        .unwrap();
    let first = game
        .handle_in_world(&npc_choice(2), &character, 1_100)
        .expect("Busy is a reported refusal, not a lost interaction");
    assert!(!first.changed);
    assert!(!game.derby_active());
    assert_eq!(game.state(), before);

    let second = game
        .handle_in_world(&npc_choice(2), &character, 1_200)
        .expect("the original menu remains retryable after refusal");
    assert!(!second.changed);
    assert!(!game.derby_active());
    assert_eq!(game.state(), before);
}

#[test]
fn offer_cancel_through_gameplay_restores_the_job_and_keeps_stale_quit_inert() {
    let fixture = GameplayFixture::new();
    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let character = CharacterState {
        guid: OWNER.raw(),
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    };
    let before_job = game.active_profile().unwrap();
    let before_coins = game.state().inventory.coins;
    game.handle_in_world(&npc_menu(), &character, 1_000)
        .unwrap();
    let entered = game
        .handle_in_world(&npc_choice(2), &character, 1_100)
        .unwrap();
    let instance = entered
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .map(|packet| i32::from_le_bytes(packet[3..7].try_into().unwrap()))
        .unwrap();
    assert!(game.derby_active());

    let cancelled = game
        .handle_in_world(&[26, 0, 42, 0], &character, 1_200)
        .unwrap();
    assert!(cancelled.changed);
    assert!(!game.derby_active());
    assert_eq!(game.active_profile(), Some(before_job));
    assert_eq!(game.state().inventory.coins, before_coins);
    assert!(
        cancelled
            .replies
            .iter()
            .any(|packet| packet[..3] == [39, 0, 19])
    );
    assert!(
        cancelled
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 20]))
    );

    let after = game.state();
    let stale = game
        .handle_in_world(&quit_request(instance), &character, 1_300)
        .unwrap();
    assert!(!stale.changed);
    assert_eq!(game.state(), after);
}

/// The F164 regression, kept by name for the routed offer. A fabricated stationary collision is a
/// client report, so it is resolved by the server (it damages the NPC) but it cannot pay by itself:
/// T219 pays only at the server-observed results phase. Leaving before results therefore pays
/// nothing and writes no ledger row.
#[test]
fn gameplay_routes_the_npc_offer_and_closes_a_fabricated_contact_without_paying() {
    let fixture = GameplayFixture::new();
    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let character = routed_character();
    let before = game.state();
    let before_coins = before.inventory.coins;

    let menu = game
        .handle_in_world(&npc_menu(), &character, 1_000)
        .unwrap();
    assert!(menu.replies.iter().any(|packet| packet[..3] == [26, 0, 9]));
    assert!(
        game.handle_in_world(&npc_choice(99), &character, 1_100)
            .is_err()
    );
    assert!(!game.derby_active());
    assert_eq!(game.state(), before);

    let entered = game
        .handle_in_world(&npc_choice(2), &character, 1_200)
        .unwrap();
    assert!(entered.changed);
    let instance = offer_instance(&entered);
    let npc = run_to_running(&mut game, &character, instance, 1_000);

    game.handle_in_world(&pose(OWNER, [0.0, 0.0, 0.0]), &character, 3_200)
        .unwrap();
    let contact = game
        .handle_in_world(&hit(npc, 5.0), &character, 3_300)
        .unwrap();
    assert!(
        !contact.replies.is_empty(),
        "the fabricated contact is resolved"
    );
    assert_eq!(game.derby_phase(), Some(DerbyPhase::Running));
    assert_eq!(game.state().inventory.coins, before_coins);
    assert_eq!(game.state().derby, None, "a contact writes no ledger row");
    assert!(
        !contact
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "a contact sends no reward bundle"
    );

    let quit = game
        .handle_in_world(&quit_request(instance), &character, 3_500)
        .unwrap();
    assert!(quit.changed);
    assert!(!game.derby_active());
    assert_eq!(game.state().inventory.coins, before_coins);
    assert_eq!(game.state().derby, None);
}

#[test]
fn gameplay_routes_the_offer_pays_one_completion_and_ignores_replays() {
    let fixture = GameplayFixture::new();
    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let character = routed_character();
    let before = game.state();
    let before_job = before.profiles.as_ref().unwrap().active;
    let before_coins = before.inventory.coins;
    assert_ne!(before_job, DEMO_DERBY_DRIVER_PROFILE);

    let menu = game
        .handle_in_world(&npc_menu(), &character, 1_000)
        .unwrap();
    assert!(menu.replies.iter().any(|packet| packet[..3] == [26, 0, 9]));
    assert!(
        game.handle_in_world(&npc_choice(99), &character, 1_100)
            .is_err()
    );
    assert!(!game.derby_active());
    assert_eq!(game.state(), before);

    let entered = game
        .handle_in_world(&npc_choice(2), &character, 1_200)
        .unwrap();
    assert!(entered.changed);
    assert!(game.derby_active());
    assert_eq!(game.active_profile(), Some(DEMO_DERBY_DRIVER_PROFILE));
    let driver_outfit = game.profile_attachments(DEMO_DERBY_DRIVER_PROFILE).unwrap();
    let entry_switch = entered
        .nearby
        .iter()
        .find(|packet| {
            packet.starts_with(&[35, 0, 7, 0])
                && packet.get(4..12) == Some(&OWNER.raw().to_le_bytes()[..])
        })
        .expect("derby entry announces the new visible outfit");
    assert_eq!(
        &entry_switch[12..entry_switch.len() - 4],
        &driver_outfit[..]
    );
    let instance = offer_instance(&entered);
    let saved = game.state();
    assert_eq!(
        saved.profiles.as_ref().unwrap().active,
        before_job,
        "the temporary Derby Driver job is never persisted"
    );
    assert!(saved.derby.is_none(), "an unpaid match writes no ledger");
    let restored = Gameplay::load(&fixture.path, Some(saved)).unwrap();
    assert!(!restored.derby_active());
    assert_eq!(
        restored.state().profiles.as_ref().unwrap().active,
        before_job,
        "a mid-match save restores the job and drops only the match"
    );

    let started = game
        .handle_in_world(&start_request(instance), &character, 1_300)
        .unwrap();
    assert!(
        started
            .replies
            .iter()
            .any(|packet| packet[..3] == [39, 0, 17])
    );
    assert!(
        started
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 20]))
    );
    let npc = register_and_npc(&mut game, &character, 1_000);
    acknowledge_readiness(&mut game, &character, 1_000);
    assert_eq!(
        game.state().inventory.coins,
        before_coins,
        "playing a match is not paying"
    );

    // A stationary fabricated contact finishes the match. The payout is flat: it does not depend
    // on the contact, the damage or the score, and it is paid at the results phase.
    let results = drive_to_results(&mut game, &character, npc, &[20.0], 1_000);
    assert!(
        messages(&results).contains(&DerbyMessage::GameState(10)),
        "results state"
    );
    let paid = game.state();
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
        "the completion also pays the driver XP once"
    );
    assert_eq!(
        paid.derby.as_ref().unwrap().paid_matches,
        vec![instance],
        "the paid match instance is recorded"
    );
    // T229: the same completion record pays the derby Station Cash kind once, through the one
    // reward applier, plus the first-activity-of-the-day bonus.
    let wallet = paid
        .sc_wallet
        .clone()
        .expect("the derby completion writes the Station Cash wallet");
    assert_eq!(
        wallet.balance,
        sc_earn::first_of_day(osfr_tools::station_cash::EarnSource::DerbyMatch),
        "one derby completion plus the first-activity-of-the-day bonus"
    );
    assert!(
        results
            .iter()
            .any(|packet| packet.starts_with(&[66, 0, 11, 0])),
        "the earned Station Cash pushes the wallet response"
    );
    let coin_update = results
        .iter()
        .find(|packet| packet.starts_with(&[38, 0, 19]))
        .expect("the coin update");
    assert_eq!(
        i32::from_le_bytes(coin_update[4..8].try_into().unwrap()),
        (before_coins + COMPLETION_COINS) as i32
    );
    // 50/1 layout from client reader 0x008e7930: coins at inner offset 4, XP/stars at 8.
    let bundle = results
        .iter()
        .find(|packet| packet.starts_with(&[50, 0, 1]))
        .expect("the reward bundle");
    assert_eq!(
        i32::from_le_bytes(bundle[4..8].try_into().unwrap()),
        COMPLETION_COINS as i32
    );
    assert_eq!(
        i32::from_le_bytes(bundle[8..12].try_into().unwrap()),
        COMPLETION_XP as i32
    );

    // The same contact report in the results phase is a repeat, not a second finish.
    let replay = game
        .handle_in_world(&hit(npc, 20.0), &character, 4_200)
        .unwrap();
    assert!(!replay.changed);
    assert!(replay.replies.is_empty());
    assert_eq!(game.state(), paid, "a repeated results record pays nothing");

    let close_ready = game.tick(&character, 5_000).unwrap();
    assert_eq!(
        messages(&close_ready.replies),
        vec![DerbyMessage::GameState(11)]
    );
    let closed = game
        .handle_in_world(&[26, 0, 42, 0], &character, 5_100)
        .unwrap();
    assert!(closed.changed);
    assert!(!game.derby_active());
    assert_eq!(
        game.state().profiles.as_ref().unwrap().active,
        before_job,
        "leaving restores the job active before entry"
    );
    assert_eq!(
        game.state().inventory.coins,
        before_coins + COMPLETION_COINS,
        "closing the results screen pays nothing more"
    );
    assert!(
        !closed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "the close sends no reward bundle"
    );
    let restored_outfit = game.profile_attachments(before_job).unwrap();
    let close_switch = closed
        .nearby
        .iter()
        .find(|packet| {
            packet.starts_with(&[35, 0, 7, 0])
                && packet.get(4..12) == Some(&OWNER.raw().to_le_bytes()[..])
        })
        .expect("an accepted derby close announces the restored visible outfit");
    assert_eq!(
        &close_switch[12..close_switch.len() - 4],
        &restored_outfit[..]
    );
    assert!(
        closed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 20]))
    );
    assert!(
        closed
            .nearby
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 7, 0]))
    );

    let replay_close = game
        .handle_in_world(&[26, 0, 42, 0], &character, 5_200)
        .unwrap();
    assert!(!replay_close.changed);
    let reloaded = Gameplay::load(&fixture.path, Some(game.state())).unwrap();
    assert!(!reloaded.derby_active());
    assert_eq!(reloaded.active_profile(), Some(before_job));
    assert_eq!(
        reloaded.state().inventory.coins,
        before_coins + COMPLETION_COINS
    );
    assert_eq!(
        reloaded.state().derby.unwrap().paid_matches,
        vec![instance],
        "the ledger survives the reconnect"
    );
}

/// The time limit is a server-side finish too: a player who stays to the results phase without
/// reporting a single contact is paid. This also covers the session-tick payout path.
#[test]
fn staying_to_the_time_limit_pays_without_any_contact() {
    let fixture = GameplayFixture::new();
    let path = fixture.path.join("derby_entry.json");
    let mut content: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    content["rules"]["score_target"] = json!(null);
    content["rules"]["time_limit_ms"] = json!(1_000);
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let character = routed_character();
    let before_coins = game.state().inventory.coins;

    let instance = enter_derby(&mut game, &character, 1_000);
    run_to_running(&mut game, &character, instance, 1_000);
    let results = game.tick(&character, 4_100).unwrap();
    assert!(messages(&results.replies).contains(&DerbyMessage::GameState(10)));
    assert_eq!(game.derby_phase(), Some(DerbyPhase::Results));
    assert!(
        results
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1]))
    );
    let paid = game.state();
    assert_eq!(paid.inventory.coins, before_coins + COMPLETION_COINS);
    assert_eq!(paid.derby.unwrap().paid_matches, vec![instance]);
}

/// Leaving or disconnecting before the results phase pays nothing and writes no ledger.
#[test]
fn leaving_or_resetting_before_results_pays_nothing() {
    let fixture = GameplayFixture::new();
    let character = routed_character();
    let before_coins = Gameplay::load(&fixture.path, None)
        .unwrap()
        .state()
        .inventory
        .coins;

    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let instance = enter_derby(&mut game, &character, 1_000);
    run_to_running(&mut game, &character, instance, 1_000);
    let quit = game
        .handle_in_world(&quit_request(instance), &character, 3_500)
        .unwrap();
    assert!(quit.changed);
    assert!(!game.derby_active());
    let after_quit = game.state();
    assert_eq!(after_quit.inventory.coins, before_coins);
    assert!(
        after_quit.derby.is_none(),
        "an unfinished match writes no ledger row"
    );
    assert!(
        after_quit.sc_wallet.is_none(),
        "an unfinished match pays no Station Cash"
    );

    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let instance = enter_derby(&mut game, &character, 5_000);
    let npc = run_to_running(&mut game, &character, instance, 5_000);
    game.reset_session();
    assert!(!game.derby_active());
    assert_eq!(game.state().inventory.coins, before_coins);
    assert!(game.state().derby.is_none());
    assert!(
        game.state().sc_wallet.is_none(),
        "a disconnect before the results pays no Station Cash"
    );
    // The old contact report cannot finish a match that the disconnect dropped.
    let replay = game
        .handle_in_world(&hit(npc, 20.0), &character, 9_000)
        .unwrap();
    assert!(!replay.changed);
    assert_eq!(game.state().inventory.coins, before_coins);
}

/// Contacts and damage never size the payout: a match that needed four reports pays the same flat
/// amount as one that needed a single report.
/// A reconnect keeps the ledger, the old match cannot pay again, and the next match is a new
/// instance that pays again: farming finished matches is allowed.
#[test]
fn a_reconnect_keeps_the_ledger_and_the_next_match_pays_again() {
    let fixture = GameplayFixture::new();
    let character = routed_character();
    let before_coins = Gameplay::load(&fixture.path, None)
        .unwrap()
        .state()
        .inventory
        .coins;
    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let first = enter_derby(&mut game, &character, 1_000);
    let npc = run_to_running(&mut game, &character, first, 1_000);
    drive_to_results(&mut game, &character, npc, &[20.0], 1_000);
    assert_eq!(
        game.state().inventory.coins,
        before_coins + COMPLETION_COINS
    );

    // Reconnect from the serialized save payload, not just the live struct: the ledger is what
    // the character save carries across a disconnect.
    let saved: GameplayState =
        serde_json::from_slice(&serde_json::to_vec(&game.state()).unwrap()).unwrap();
    assert_eq!(
        saved.derby.as_ref().unwrap().paid_matches,
        vec![first],
        "the paid ledger survives the save encoding"
    );
    let mut game = Gameplay::load(&fixture.path, Some(saved)).unwrap();
    assert!(!game.derby_active());
    assert_eq!(
        game.state().inventory.coins,
        before_coins + COMPLETION_COINS
    );
    let replay = game
        .handle_in_world(&hit(npc, 20.0), &character, 7_000)
        .unwrap();
    assert!(!replay.changed);
    assert_eq!(
        game.state().inventory.coins,
        before_coins + COMPLETION_COINS,
        "the old match cannot pay after a reconnect"
    );

    let second = enter_derby(&mut game, &character, 8_000);
    assert!(
        second > first,
        "a paid instance id is not reissued across a reconnect"
    );
    let npc = run_to_running(&mut game, &character, second, 8_000);
    drive_to_results(&mut game, &character, npc, &[20.0], 8_000);
    let farmed = game.state();
    assert_eq!(farmed.inventory.coins, before_coins + 2 * COMPLETION_COINS);
    assert_eq!(
        farmed
            .profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&DEMO_DERBY_DRIVER_PROFILE),
        Some(&(2 * COMPLETION_XP))
    );
    assert_eq!(farmed.derby.unwrap().paid_matches, vec![first, second]);
}

#[test]
fn collision_reports_never_change_the_payout() {
    let fixture = GameplayFixture::new();
    let character = routed_character();
    let before_coins = Gameplay::load(&fixture.path, None)
        .unwrap()
        .state()
        .inventory
        .coins;

    let mut single = Gameplay::load(&fixture.path, None).unwrap();
    let instance = enter_derby(&mut single, &character, 1_000);
    let npc = run_to_running(&mut single, &character, instance, 1_000);
    drive_to_results(&mut single, &character, npc, &[20.0], 1_000);
    assert_eq!(
        single.state().inventory.coins,
        before_coins + COMPLETION_COINS
    );

    let mut several = Gameplay::load(&fixture.path, None).unwrap();
    let instance = enter_derby(&mut several, &character, 1_000);
    let npc = run_to_running(&mut several, &character, instance, 1_000);
    drive_to_results(&mut several, &character, npc, &[5.0, 5.0, 5.0, 5.0], 1_000);
    let longer = several.state();
    assert_eq!(
        longer.inventory.coins,
        before_coins + COMPLETION_COINS,
        "a longer match pays the same flat completion"
    );
    assert_eq!(
        longer
            .profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&DEMO_DERBY_DRIVER_PROFILE),
        Some(&COMPLETION_XP)
    );
}

/// The ledger is keyed by character and match instance: two characters whose saves both allocate
/// the same instance id are each paid exactly once.
#[test]
fn two_characters_with_the_same_match_instance_are_each_paid_once() {
    let first_fixture = GameplayFixture::new();
    let second_fixture = GameplayFixture::new();
    let mut first = Gameplay::load(&first_fixture.path, None).unwrap();
    let mut second = Gameplay::load(&second_fixture.path, None).unwrap();
    let owner = routed_character();
    let other = CharacterState {
        guid: OTHER.raw(),
        ..routed_character()
    };
    let owner_coins = first.state().inventory.coins;
    let other_coins = second.state().inventory.coins;

    let first_instance = enter_derby(&mut first, &owner, 1_000);
    let second_instance = enter_derby(&mut second, &other, 1_000);
    assert_eq!(
        first_instance, second_instance,
        "each save allocates its own private instance ids"
    );
    let first_npc = run_to_running(&mut first, &owner, first_instance, 1_000);
    let second_npc = run_to_running(&mut second, &other, second_instance, 1_000);
    drive_to_results(&mut first, &owner, first_npc, &[20.0], 1_000);
    drive_to_results(&mut second, &other, second_npc, &[20.0], 1_000);

    for (game, coins, guid) in [
        (&first, owner_coins, owner.guid),
        (&second, other_coins, other.guid),
    ] {
        let paid = game.state();
        assert_eq!(
            paid.inventory.coins,
            coins + COMPLETION_COINS,
            "character {guid} is paid once"
        );
        assert_eq!(
            paid.derby.as_ref().unwrap().paid_matches,
            vec![first_instance],
            "each character keeps its own ledger for the same match id"
        );
    }
}

/// The ledger is bounded: the newest `MAX_PAID_MATCHES` matches are kept, each match is paid once,
/// and a reload resumes the instance counter above the newest kept row.
#[test]
fn the_paid_ledger_is_bounded_and_resumes_above_it() {
    let fixture = Fixture::new();
    let mut gateway = DerbyGateway::load(&fixture.path, None).unwrap().unwrap();
    let mut paid = Vec::new();
    for _ in 0..(MAX_PAID_MATCHES + 3) {
        gateway.begin(&character()).unwrap();
        let instance = gateway.instance_id().unwrap();
        gateway.start(&start_request(instance)).unwrap();
        let setup = gateway.handle(&register(), 1_400).unwrap();
        let npc = messages(&setup)
            .into_iter()
            .find_map(|message| match message {
                DerbyMessage::CreateVehicle(vehicle) if vehicle.key != OWNER => Some(vehicle.key),
                _ => None,
            })
            .expect("NPC vehicle");
        for phase_value in 3..=8 {
            gateway
                .handle(&phase(phase_value), 2_000 + phase_value as u64)
                .unwrap();
        }
        for value in (0..=4).rev() {
            gateway
                .handle(&countdown(value), 3_000 + (4 - value) as u64)
                .unwrap();
        }
        gateway
            .handle(&pose(OWNER, [0.0, 0.0, 0.0]), 3_900)
            .unwrap();
        let hit = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
            reporter: OWNER,
            other: npc,
            impact_bits: 20.0_f32.to_bits(),
        }))
        .unwrap();
        gateway.handle(&hit, 4_000).unwrap();
        assert_eq!(gateway.phase(), Some(DerbyPhase::Results));
        assert_eq!(gateway.take_completion(), Some(instance));
        assert_eq!(gateway.take_completion(), None, "one match pays once");
        gateway.exit();
        paid.push(instance);
    }

    let saved = gateway.saved_state().expect("paid ledger");
    assert_eq!(saved.paid_matches.len(), MAX_PAID_MATCHES);
    assert_eq!(saved.paid_matches.last().copied(), paid.last().copied());
    assert!(
        !saved.paid_matches.contains(&paid[0]),
        "the oldest row is dropped, not the newest"
    );
    let mut restored = DerbyGateway::load(&fixture.path, Some(&saved))
        .unwrap()
        .unwrap();
    restored.begin(&character()).unwrap();
    assert!(
        restored.instance_id().unwrap() > *paid.last().unwrap(),
        "a paid instance id is not reissued after a reload"
    );
}

/// Content and save bounds fail closed instead of paying an unbounded or malformed ledger.
#[test]
fn completion_content_and_ledger_bounds_fail_closed() {
    let fixture = Fixture::new();
    let path = fixture.path.join("derby_entry.json");
    let mut content: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    content["completion_reward_coins"] = json!(MAX_COMPLETION_REWARD + 1);
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let error = DerbyGateway::load(&fixture.path, None)
        .err()
        .expect("an over-bound reward is refused");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);

    let clean = Fixture::new();
    for paid_matches in [vec![0], vec![-1], vec![1; MAX_PAID_MATCHES + 1]] {
        let ledger = DerbyGatewayState {
            paid_matches,
            ..Default::default()
        };
        let error = DerbyGateway::load(&clean.path, Some(&ledger))
            .err()
            .expect("a malformed ledger is refused");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
    // A ledger inside the bounds loads and resumes the counter above its newest row.
    let first = FIRST_DERBY_INSTANCE as i32;
    let ledger = DerbyGatewayState {
        paid_matches: vec![first, first + 5],
        ..Default::default()
    };
    let mut gateway = DerbyGateway::load(&clean.path, Some(&ledger))
        .unwrap()
        .unwrap();
    gateway.begin(&character()).unwrap();
    assert_eq!(gateway.instance_id(), Some(first + 6));
}
