//! Entry-to-payout coverage for the routed demolition derby.
//!
//! The package is synthetic and intentionally small. It exercises the real native opcode 34
//! codec, the real 39/16 offer -> 39/5 Go exchange, the client phase/countdown acknowledgements,
//! server-owned contact damage, the results hold, and the reward transaction. All timing and
//! combat values are reconstructed test policy (R).

use osfr_tools::{
    character_store::CharacterState,
    derby::{
        DerbyConfig, DerbyMessage, DerbyRules, DerbyVehicleTemplate, ParticipantKey,
        VehicleHitReport, decode_inner, encode_inner,
    },
    derby_gateway::{DerbyGates, DerbyGateway, DerbyGatewayContent, DerbyReward},
    gameplay::Gameplay,
    harvesting,
    inventory::Inventory,
    mining::Type8Engine,
    profiles::ProfileState,
    rewards::{RewardState, apply_grant},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const OWNER: ParticipantKey = ParticipantKey(0x1000_0000_0000_0001);
const OTHER: ParticipantKey = ParticipantKey(0x1000_0000_0000_0002);
const GATE: u64 = 9_000_000_301;
const INSTANCE_GAME: i32 = 39;
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
                reward_cooldown_s: 600,
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
                reward_cooldown_s: 600,
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
        reward: DerbyReward {
            coins: 5,
            job_xp: Default::default(),
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
        },
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

fn register() -> Vec<u8> {
    encode_inner(&DerbyMessage::RegisterPlayer {
        key: OWNER,
        unknown_key: 0,
    })
    .unwrap()
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

fn phase(phase: i32) -> Vec<u8> {
    encode_inner(&DerbyMessage::PlayerPhase { key: OWNER, phase }).unwrap()
}

fn countdown(count: u32) -> Vec<u8> {
    encode_inner(&DerbyMessage::CountdownStatus { key: OWNER, count }).unwrap()
}

fn messages(packets: &[Vec<u8>]) -> Vec<DerbyMessage> {
    packets
        .iter()
        .map(|packet| decode_inner(packet).unwrap())
        .collect()
}

fn reward_state(fixture: &Fixture) -> RewardState {
    RewardState::new(
        Inventory::load(&fixture.path, None).unwrap(),
        ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::from([1]),
            experience: Default::default(),
        },
        100,
    )
}

#[test]
fn npc_entry_reaches_running_results_and_one_payout() {
    let fixture = Fixture::new();
    let mut gateway = DerbyGateway::load(&fixture.path).unwrap().unwrap();
    assert_eq!(gateway.gate_npcs(), &[GATE]);
    assert!(gateway.admit(OTHER.raw(), false).is_err());
    assert!(gateway.admit(GATE, true).is_err());
    assert_eq!(gateway.admit(GATE, false), Ok(()));

    let offer = gateway.begin(&character()).unwrap();
    assert_eq!(&offer[..3], &[39, 0, 16]);
    let instance = gateway.instance_id().unwrap();
    let rewards = reward_state(&fixture);

    // A close before the results screen and a spoofed register change nothing.
    let refused = gateway.close(&[26, 0, 42, 0], &rewards).unwrap();
    assert!(!refused.accepted);
    assert!(refused.reward_state.is_none());
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

    let paid = gateway.close(&[26, 0, 42, 0], &rewards).unwrap();
    assert!(paid.accepted);
    let next = paid.reward_state.expect("accepted close carries rewards");
    assert_eq!(next.inventory.state().coins, 15);
    assert!(
        paid.reward_packets
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1]))
    );
    assert_eq!(gateway.phase(), Some(osfr_tools::derby::DerbyPhase::Closed));

    let replay = gateway.close(&[26, 0, 42, 0], &next).unwrap();
    assert!(!replay.accepted);
    assert!(replay.reward_state.is_none());
    assert_eq!(next.inventory.state().coins, 15);
}

#[test]
fn reward_apply_is_atomic_and_an_inactive_close_is_inert() {
    let fixture = Fixture::new();
    let mut gateway = DerbyGateway::load(&fixture.path).unwrap().unwrap();
    let rewards = reward_state(&fixture);
    let before = serde_json::to_value(&rewards.inventory.state()).unwrap();
    let grant = osfr_tools::rewards::RewardGrant {
        coins: 5,
        items: Vec::new(),
        job_xp: Default::default(),
        display: osfr_tools::rewards::RewardDisplay {
            source_guid: OWNER.raw(),
            player_guid: OWNER.raw(),
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    };
    let (next, packets) = apply_grant(&rewards, &grant, "derby-test-grant").unwrap();
    assert_eq!(next.inventory.state().coins, 15);
    assert!(!packets.is_empty());
    assert_eq!(
        serde_json::to_value(&rewards.inventory.state()).unwrap(),
        before
    );

    let refused = gateway.close(&[26, 0, 42, 0], &rewards).unwrap();
    assert!(!refused.accepted);
    assert_eq!(gateway.phase(), None);
}

#[test]
fn offer_cancel_and_running_quit_retire_without_rewards_and_stale_quit_is_inert() {
    let fixture = Fixture::new();
    let mut gateway = DerbyGateway::load(&fixture.path).unwrap().unwrap();
    let rewards = reward_state(&fixture);
    let before = serde_json::to_value(rewards.inventory.state()).unwrap();

    gateway.begin(&character()).unwrap();
    let cancelled = gateway.cancel(&[26, 0, 42, 0]).unwrap();
    assert!(!gateway.active());
    assert_eq!(&cancelled[0][..3], &[39, 0, 19]);
    assert_eq!(
        serde_json::to_value(rewards.inventory.state()).unwrap(),
        before
    );

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
    assert_eq!(
        serde_json::to_value(rewards.inventory.state()).unwrap(),
        before
    );
    assert!(!gateway.close(&[26, 0, 42, 0], &rewards).unwrap().accepted);
}

#[test]
fn paid_derby_cooldown_survives_reload_and_blocks_immediate_reentry() {
    let fixture = Fixture::new();
    let mut gateway = DerbyGateway::load(&fixture.path).unwrap().unwrap();
    let rewards = reward_state(&fixture);
    gateway.begin(&character()).unwrap();
    let instance = gateway.instance_id().unwrap();
    gateway.start(&start_request(instance)).unwrap();
    let setup = gateway.handle(&register(), 2_000).unwrap();
    let npc = messages(&setup)
        .into_iter()
        .find_map(|message| match message {
            DerbyMessage::CreateVehicle(vehicle) if vehicle.key != OWNER => Some(vehicle.key),
            _ => None,
        })
        .unwrap();
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
    gateway.tick(5_000).unwrap();
    let paid = gateway
        .close_at(&[26, 0, 42, 0], &rewards, 1_000_000)
        .unwrap();
    assert!(paid.accepted);
    gateway.exit();

    assert_eq!(
        gateway.admit_at(GATE, false, 1_000_599),
        Err(osfr_tools::derby_gateway::DerbyEntryDenied::Cooldown)
    );
    assert_eq!(gateway.admit_at(GATE, false, 1_000_600), Ok(()));
    let saved = gateway.snapshot().expect("paid run stores its epoch");
    let restored = DerbyGateway::load_with_state(&fixture.path, Some(saved.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(
        restored.admit_at(GATE, false, 1_000_599),
        Err(osfr_tools::derby_gateway::DerbyEntryDenied::Cooldown)
    );
    assert_eq!(saved.last_paid_unix, 1_000_000);
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

#[test]
fn gameplay_routes_the_npc_offer_and_commits_the_close_payout() {
    let fixture = GameplayFixture::new();
    let mut game = Gameplay::load(&fixture.path, None).unwrap();
    let character = CharacterState {
        guid: OWNER.raw(),
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    };
    let before = game.state().clone();
    let before_job = before.profiles.as_ref().unwrap().active;
    let before_coins = before.inventory.coins;
    assert_ne!(
        before_job,
        osfr_tools::derby_gateway::DEMO_DERBY_DRIVER_PROFILE
    );

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
    assert_eq!(
        game.active_profile(),
        Some(osfr_tools::derby_gateway::DEMO_DERBY_DRIVER_PROFILE)
    );
    let driver_outfit = game
        .profile_attachments(osfr_tools::derby_gateway::DEMO_DERBY_DRIVER_PROFILE)
        .unwrap();
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
    let offer = entered
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .expect("NPC choice emits the derby offer");
    let instance = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    let saved = game.state();
    assert_eq!(
        saved.profiles.as_ref().unwrap().active,
        before_job,
        "the temporary Derby Driver job is never persisted"
    );
    assert!(saved.derby.is_none());
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

    let setup = game
        .handle_in_world(&register(), &character, 1_400)
        .unwrap();
    let npc = messages(&setup.replies)
        .into_iter()
        .find_map(|message| match message {
            DerbyMessage::CreateVehicle(vehicle) if vehicle.key != OWNER => Some(vehicle.key),
            _ => None,
        })
        .expect("NPC vehicle");
    for phase_value in 3..=8 {
        let out = game
            .handle_in_world(&phase(phase_value), &character, 2_000 + phase_value as u64)
            .unwrap();
        assert!(!out.replies.is_empty(), "phase {phase_value}");
    }
    for value in (0..=4).rev() {
        let out = game
            .handle_in_world(&countdown(value), &character, 3_000 + (4 - value) as u64)
            .unwrap();
        assert!(!out.replies.is_empty(), "countdown {value}");
    }
    let mid_match = game.state();
    assert_eq!(mid_match.profiles.as_ref().unwrap().active, before_job);
    assert!(mid_match.derby.is_none());
    let restored = Gameplay::load(&fixture.path, Some(mid_match)).unwrap();
    assert!(!restored.derby_active());
    assert_eq!(restored.active_profile(), Some(before_job));
    assert_eq!(restored.state().inventory.coins, before_coins);

    game.handle_in_world(&pose(OWNER, [0.0, 0.0, 0.0]), &character, 3_900)
        .unwrap();
    let hit = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: OWNER,
        other: npc,
        impact_bits: 20.0_f32.to_bits(),
    }))
    .unwrap();
    let results = game.handle_in_world(&hit, &character, 4_000).unwrap();
    assert!(
        messages(&results.replies).contains(&DerbyMessage::GameState(10)),
        "results state"
    );
    let close_ready = game.tick(&character, 5_000).unwrap();
    assert_eq!(
        messages(&close_ready.replies),
        vec![DerbyMessage::GameState(11)]
    );

    let paid = game
        .handle_in_world(&[26, 0, 42, 0], &character, 5_100)
        .unwrap();
    assert!(paid.changed);
    assert!(!game.derby_active());
    assert_eq!(
        game.state().profiles.as_ref().unwrap().active,
        before_job,
        "leaving restores the job active before entry"
    );
    let restored_outfit = game.profile_attachments(before_job).unwrap();
    let close_switch = paid
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
    assert_eq!(game.state().inventory.coins, before_coins + 5);
    assert!(
        paid.replies
            .iter()
            .any(|packet| packet.starts_with(&[50, 0, 1]))
    );
    assert!(
        paid.replies
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 20]))
    );
    assert!(
        paid.nearby
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 7, 0]))
    );

    let replay = game
        .handle_in_world(&[26, 0, 42, 0], &character, 5_200)
        .unwrap();
    assert!(!replay.changed);
    assert_eq!(game.state().inventory.coins, before_coins + 5);
    let paid_state = game.state();
    assert!(
        paid_state
            .derby
            .as_ref()
            .is_some_and(|derby| derby.last_paid_unix > 0)
    );
    let reloaded = Gameplay::load(&fixture.path, Some(paid_state)).unwrap();
    assert!(!reloaded.derby_active());
    assert_eq!(reloaded.active_profile(), Some(before_job));
    assert_eq!(reloaded.state().inventory.coins, before_coins + 5);
}
