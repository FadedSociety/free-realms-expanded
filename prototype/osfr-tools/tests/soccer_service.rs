//! Acceptance for the bounded native soccer lifecycle/wire service
//! (`prototype/osfr-tools/src/soccer_service.rs`).
//!
//! Every byte is synthetic and built with the Rust serializers. Client facts
//! are limited to layouts and field semantics supported by the native map;
//! match outputs and zoning values are reconstructed policy (R).
//!
//! The service composes three already-reviewed pieces instead of re-deriving
//! them: `soccer` (wire codec), `soccer_match` (server authority) and the
//! frozen private sub-14 adapter (`soccer-state-adapter-20261009T174320Z`,
//! sha256 `11d291c2284922e3cde7549632a969b85261a2c12a7ba59464178281c2de5dd7`),
//! whose adapter code is copied verbatim into the service module.

use osfr_tools::{inventory::Inventory, profiles::ProfileState, soccer_service};

use osfr_tools::minigame::RunState;
use osfr_tools::rewards::{RewardDisplay, RewardGrant, RewardState};
use osfr_tools::soccer::{Evidence, Message, SoccerPacket, decode as soccer_decode};
use osfr_tools::soccer_ball::BallTuning;
use osfr_tools::soccer_match::{FieldBounds, MatchConfig, MatchPhase, PhysicsConfig};
use osfr_tools::zone_transition;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use soccer_service::{
    AdapterPolicy, R_ENTRY_POSITION, R_ENTRY_ROTATION, REGISTER_MARKER, SCENE_ID, ServiceEvent, Setup,
    SoccerService, emit_packet, entry_frame,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    sync::atomic::{AtomicU64, Ordering},
};

const KICKER: u64 = 0x0000_0010_0080_06b2;
const TEAMMATE: u64 = 0x0000_0010_0080_06b3;
const OPPONENT: u64 = 0x0000_0010_0080_06b4;
const UNREGISTERED: u64 = 0x0000_0010_0080_06b5;
static REWARD_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

fn config() -> MatchConfig {
    let mut config = MatchConfig::new(FieldBounds {
        centre: [0.0; 3],
        half_x: 100.0,
        half_z: 100.0,
        goal_half_width: 10.0,
        goal_height: 5.0,
    });
    config.kickoff_spawn = [0.0, 0.0, 0.0];
    config.physics = PhysicsConfig {
        gravity: 0.0,
        ..PhysicsConfig::default()
    };
    config.ball = rolling_ball();
    config
}

/// Small pitch: a +z shot at power 4 crosses the goal line after 10 ticks.
fn goal_config() -> MatchConfig {
    let mut config = MatchConfig::new(FieldBounds {
        centre: [0.0; 3],
        // L932: the goals are on the x ends; the 10-unit goal axis is kept.
        half_x: 10.0,
        half_z: 100.0,
        goal_half_width: 10.0,
        goal_height: 5.0,
    });
    config.kickoff_spawn = [0.0, 0.0, 0.0];
    config.physics = PhysicsConfig {
        gravity: 0.0,
        ..PhysicsConfig::default()
    };
    config.ball = rolling_ball();
    config
}

/// L955: a ball that keeps its speed on the grass, so the synthetic shots travel in straight lines.
fn rolling_ball() -> BallTuning {
    BallTuning {
        drag: 0.0,
        friction: 0.0,
        linear_friction: 0.0,
        ..BallTuning::default()
    }
}

fn policy() -> AdapterPolicy {
    AdapterPolicy {
        max_power: 4.0,
        charge_floor: 0.01,
        min_direction_norm: 1.0e-3,
    }
}

fn service() -> SoccerService {
    SoccerService::new(config(), policy()).expect("service config")
}

fn fixture_span(kind: &str, offset: usize, profile_id: i32, bytes: &[u8]) -> Value {
    json!({
        "kind": kind,
        "offset": offset,
        "length": bytes.len(),
        "profile_id": profile_id,
        "sha256": format!("{:x}", Sha256::digest(bytes))
    })
}

fn reward_state() -> RewardState {
    let path = std::env::temp_dir().join(format!(
        "osfr-soccer-reward-{}-{}",
        std::process::id(),
        REWARD_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).expect("create synthetic inventory directory");
    let coins = 0_i32.to_le_bytes();
    let empty = 0_i32.to_le_bytes();
    let content = json!({
        "schema_version": 1,
        "player_guid": KICKER,
        "default_state": {
            "coins": 0,
            "items": [],
            "equipment": [],
            "next_instance_id": null
        },
        "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
        "spans": [
            fixture_span("coins", 13, 0, &coins),
            fixture_span("items", 17, 0, &empty),
            fixture_span("equipment", 21, 1, &empty)
        ],
        "definitions": []
    });
    fs::write(
        path.join("inventory.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .expect("write synthetic inventory fixture");
    let inventory = Inventory::load(&path, None).expect("load synthetic inventory fixture");
    fs::remove_file(path.join("inventory.json")).expect("remove fixture file");
    fs::remove_dir(path).expect("remove fixture directory");
    RewardState::new(
        inventory,
        ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::from([1]),
            experience: BTreeMap::new(),
        },
        8,
    )
}

fn reward_grant(player: u64, coins: u32) -> RewardGrant {
    RewardGrant {
        coins,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        sc: 0,
        display: RewardDisplay {
            source_guid: 0,
            player_guid: player,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn minigame_request(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for value in [instance, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

fn offer_and_start(service: &mut SoccerService, session: u64) -> Vec<ServiceEvent> {
    let offer = service
        .offer_minigame(session, 1201, 1, reward_grant(session, 0))
        .expect("offer minigame");
    let instance = service.instance_id(session).expect("instance allocated");
    assert_eq!(offer.state, Some(RunState::Offered));
    let start = service
        .handle_minigame_lifecycle(session, &minigame_request(5, instance))
        .expect("route start through MinigameRuns");
    assert_eq!(start.run.state, Some(RunState::Starting));
    let events = service.mark_started(session).expect("mark activity ready");
    assert_eq!(service.run_state(session), Some(RunState::Playing));
    events
}

fn running_service() -> SoccerService {
    let mut service = service();
    service.join(KICKER, 0, 0).expect("join kicker");
    service.join(TEAMMATE, 0, 1).expect("join teammate");
    service.join(OPPONENT, 1, 0).expect("join opponent");
    offer_and_start(&mut service, KICKER);
    offer_and_start(&mut service, TEAMMATE);
    offer_and_start(&mut service, OPPONENT);
    service
        .handle_client(
            KICKER,
            0.0,
            &client_position(KICKER, [0.0, 0.0, 0.0]),
            1,
            None,
        )
        .expect("kicker position");
    service
        .handle_client(
            TEAMMATE,
            0.0,
            &client_position(TEAMMATE, [3.0, 0.0, 0.0]),
            1,
            None,
        )
        .expect("teammate position");
    service.tick(0.05).expect("kickoff tick");
    assert_eq!(service.possession(), Some(KICKER), "kicker holds the ball");
    service
}

fn client_position(player: u64, position: [f32; 3]) -> Vec<u8> {
    osfr_tools::soccer::encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdatePlayerPos {
            player,
            position,
            tail: [0; 45],
        },
    })
}

fn client_state(player: u64, state: u8, aux: u8, vec_a: [f32; 4]) -> Vec<u8> {
    osfr_tools::soccer::encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdatePlayerState {
            player,
            state,
            aux,
            state_counter: 0x0BAD_F00D,
            vectors: [vec_a, [9.0, -9.0, 9.0, -9.0], [123.0, 456.0, 789.0, 1.0]],
        },
    })
}

fn client_controls(player: u64, direction: [f32; 2], controls: u32) -> Vec<u8> {
    osfr_tools::soccer::encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdatePlayerControls {
            player,
            direction,
            controls,
        },
    })
}

fn client_goal(fields: [u32; 3]) -> Vec<u8> {
    osfr_tools::soccer::encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::GoalMade { fields },
    })
}

fn impulse_event(events: &[ServiceEvent]) -> &ServiceEvent {
    events
        .iter()
        .find(|event| matches!(event, ServiceEvent::Impulse { .. }))
        .expect("an impulse event")
}

#[test]
fn reconstructed_entry_frame_round_trips_its_candidate_values() {
    let frame = entry_frame(R_ENTRY_POSITION, R_ENTRY_ROTATION).expect("entry frame encodes");
    let packet = zone_transition::decode(&frame).expect("entry frame decodes");
    assert_eq!(packet.scene, "sg_soccer");
    assert_eq!(packet.envelope_flag, 1);
    assert_eq!(packet.type_id, 2);
    assert_eq!(packet.position, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(packet.rotation, [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(packet.sky, "sky_wilds24.xml");
    assert!(!packet.tutorial);
    assert_eq!(packet.unknown, 1);
    assert_eq!(packet.id, SCENE_ID);
    assert_ne!(
        packet.id, 0,
        "the arena is not the public overworld instance"
    );
    assert_eq!(packet.update_radius, 0.0);
    assert_eq!(packet.geometry_id, 107);
    assert!(packet.override_update_radius);
    assert!(!packet.wait_for_zone_ready);
    assert_eq!(
        zone_transition::encode(&packet).expect("re-encode"),
        frame,
        "entry frame round-trips byte-exact"
    );
    assert!(
        entry_frame([f32::NAN, 0.0, 0.0, 1.0], R_ENTRY_ROTATION).is_err(),
        "non-finite spawn is refused"
    );
}

#[test]
fn join_emits_the_candidate_registration_then_team_assignment() {
    let mut service = service();
    let events = service.join(KICKER, 0, 2).expect("join");
    assert_eq!(
        events,
        vec![
            ServiceEvent::Registered {
                player: KICKER,
                marker: REGISTER_MARKER,
            },
            ServiceEvent::TeamAssigned {
                player: KICKER,
                team: 0,
                slot: 2,
            },
        ]
    );
    for event in &events {
        let bytes = event.wire();
        let message = soccer_decode(&bytes).expect("service event wire decodes");
        assert!(message.flag, "candidate service frames carry the flag");
        assert_eq!(message.packet, event.packet());
    }
    assert_eq!(service.session_team(KICKER), Some((0, 2)));
    assert_eq!(service.session_count(), 1);

    let error = service.join(KICKER, 0, 3).expect_err("duplicate join");
    assert!(error.to_string().contains("already joined"), "{error}");
    assert!(
        service.join(OPPONENT, 2, 0).is_err(),
        "team index stays bounded"
    );
    assert!(
        service.join(OPPONENT, 1, 4).is_err(),
        "the four-slot roster stays bounded"
    );
}

#[test]
fn kickoff_and_tick_emit_monotonic_game_time_and_refuse_bad_steps() {
    let mut service = service();
    service.join(KICKER, 0, 0).expect("join");
    let events = offer_and_start(&mut service, KICKER);
    assert_eq!(
        events,
        vec![ServiceEvent::GameTime {
            remaining: 180.0,
            elapsed: 0.0,
        }]
    );
    let tick_events = service.tick(0.25).expect("tick");
    assert_eq!(tick_events.len(), 1, "{tick_events:?}");
    match tick_events[0] {
        ServiceEvent::GameTime { remaining, elapsed } => {
            assert!((remaining - 179.75).abs() < 1.0e-3, "{remaining}");
            assert!((elapsed - 0.25).abs() < 1.0e-3, "{elapsed}");
        }
        ref other => panic!("expected game time, got {other:?}"),
    }
    let bytes = tick_events[0].wire();
    assert_eq!(
        soccer_decode(&bytes).expect("wire decodes").packet,
        tick_events[0].packet()
    );

    let before = service.snapshot();
    for bad in [0.0f32, -0.25, f32::NAN, f32::INFINITY, 1.0] {
        let error = service.tick(bad).expect_err("bad tick refused");
        assert!(error.to_string().contains("step"), "{error}");
        assert_eq!(service.snapshot(), before);
    }
}

#[test]
fn client_shot_moves_the_ball_and_emits_the_recovered_impulse_layout() {
    let mut service = running_service();
    let events = service
        .handle_client(
            KICKER,
            0.05,
            // L955: 16, a Space-only standing kick; a 17 straight at the team-mate standing 3 m
            // along +x (and away from the goal) is the client's Alt pass.
            &client_state(KICKER, 16, 255, [1.0, 0.0, 0.0, 1.0]),
            2,
            None,
        )
        .expect("shot applies");
    assert_eq!(events.len(), 1, "{events:?}");
    let impulse_event = impulse_event(&events);
    let ServiceEvent::Impulse {
        kind,
        player,
        vector,
        charge,
        power,
        action,
        value_c,
    } = impulse_event
    else {
        panic!("expected an impulse");
    };
    assert_eq!(*player, KICKER);
    assert_eq!(*kind, osfr_tools::soccer_match::KickKind::Shot);
    assert_eq!(*vector, [1.0, 0.0, 0.0, 0.0]);
    assert!((charge - 1.0).abs() < 1.0e-6);
    assert!((power - 4.0).abs() < 1.0e-6);
    assert_eq!(*action, 0, "reconstructed shot action code");
    assert_eq!(*value_c, 0.0, "unnamed tail float stays zero");
    assert_eq!(service.possession(), None, "the kick releases possession");

    let velocity = service.ball_velocity();
    assert!((velocity[0] - 4.0).abs() < 1.0e-5, "{velocity:?}");
    assert!(
        velocity[1].abs() < 1.0e-6 && velocity[2].abs() < 1.0e-6,
        "{velocity:?}"
    );
    let bytes = impulse_event.wire();
    let message = soccer_decode(&bytes).expect("impulse wire decodes");
    assert!(message.flag);
    assert_eq!(
        message.packet,
        SoccerPacket::ImpulseBall {
            word: 0,
            vector: [1.0, 0.0, 0.0, 0.0],
            value_a: 1.0,
            value_b: 4.0,
            action: 0,
            value_c: 0.0,
        }
    );
    let before = service.ball_position();
    service.tick(0.25).expect("advance");
    assert!(
        service.ball_position()[0] > before[0],
        "ball moves along +x after the shot"
    );
}

#[test]
fn pass_uses_the_verified_teammate_target_and_refuses_unverified_targets() {
    let mut service = running_service();
    let pass_points = |service: &mut SoccerService, target: Option<u64>| {
        service.handle_client(
            KICKER,
            0.05,
            &client_state(KICKER, 27, 128, [0.0, 1.0, 0.0, 1.0]),
            2,
            target,
        )
    };

    let before = service.snapshot();
    let error = pass_points(&mut service, Some(UNREGISTERED)).expect_err("unregistered target");
    assert!(error.to_string().contains("target"), "{error}");
    assert_eq!(service.snapshot(), before);
    let error = pass_points(&mut service, Some(OPPONENT)).expect_err("opponent target");
    assert!(error.to_string().contains("teammate"), "{error}");
    assert_eq!(service.snapshot(), before);

    let events = pass_points(&mut service, Some(TEAMMATE)).expect("verified teammate pass");
    let impulse_event = impulse_event(&events);
    let ServiceEvent::Impulse {
        kind,
        vector,
        charge,
        power,
        action,
        ..
    } = impulse_event
    else {
        panic!("expected an impulse");
    };
    assert_eq!(*kind, osfr_tools::soccer_match::KickKind::Pass);
    assert!(
        (vector[0] - 1.0).abs() < 1.0e-5 && vector[1].abs() < 1.0e-6 && vector[2].abs() < 1.0e-6,
        "pass direction comes from the verified teammate, not vecA {vector:?}"
    );
    let expected_power = (128.0 / 255.0) * 4.0;
    assert!((charge - 128.0 / 255.0).abs() < 1.0e-6);
    assert!((power - expected_power).abs() < 1.0e-6);
    assert_eq!(*action, 1, "reconstructed pass action code");
    let velocity = service.ball_velocity();
    assert!(
        (velocity[0] - expected_power).abs() < 1.0e-5,
        "{velocity:?}"
    );
    assert!(
        velocity[1].abs() < 1.0e-6 && velocity[2].abs() < 1.0e-6,
        "{velocity:?}"
    );
}

/// L955: the client names no pass target, so the service resolves it as the client chose it
/// (`00b77480`: the team-mate best aligned with the kick): a pass state at the team-mate passes to
/// it, a standing 17 at the team-mate is the client's Alt pass, a standing 17 at the goal is a
/// shot, and a pass with no mate in line goes as a kick. Before, every pass left the ball at the
/// player's feet (owner: team-mates never got it).
#[test]
fn the_service_finds_the_pass_target_the_client_aimed_at() {
    use osfr_tools::soccer_match::KickKind;
    // TEAMMATE stands 3 m along +x from KICKER; team 0 attacks -x in the first half.
    let kick = |state: u8, direction: [f32; 4]| {
        let mut service = running_service();
        let events = service
            .handle_client(KICKER, 0.05, &client_state(KICKER, state, 128, direction), 2, None)
            .expect("kick applies");
        let ServiceEvent::Impulse { kind, .. } = impulse_event(&events) else {
            panic!("expected an impulse");
        };
        (*kind, service.ball_velocity())
    };
    assert_eq!(kick(27, [1.0, 0.0, 0.0, 1.0]).0, KickKind::Pass, "running pass");
    assert_eq!(kick(17, [1.0, 0.0, 0.0, 1.0]).0, KickKind::Pass, "standing pass");
    assert_eq!(kick(17, [-1.0, 0.0, 0.0, 1.0]).0, KickKind::Shot, "shot at goal");
    let (kind, velocity) = kick(27, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(kind, KickKind::Shot, "no team-mate in line");
    assert!(velocity[2] > 0.0, "the ball still leaves the player's feet");
}

#[test]
fn controls_are_advisory_and_replay_or_spoof_is_refused_untouched() {
    let mut service = running_service();
    let events = service
        .handle_client(
            KICKER,
            0.05,
            &client_controls(KICKER, [0.0, -1.0], 0x11),
            2,
            None,
        )
        .expect("controls accepted");
    assert!(events.is_empty(), "controls emit no authoritative packet");
    assert_eq!(service.possession(), Some(KICKER), "controls never kick");
    assert_eq!(service.ball_velocity(), [0.0, 0.0, 0.0]);

    let before = service.snapshot();
    let error = service
        .handle_client(
            KICKER,
            0.10,
            &client_controls(KICKER, [0.0, -1.0], 1),
            2,
            None,
        )
        .expect_err("replayed transport sequence");
    assert!(error.to_string().contains("stale"), "{error}");
    assert_eq!(service.snapshot(), before);

    let error = service
        .handle_client(
            OPPONENT,
            0.10,
            &client_controls(KICKER, [0.0, -1.0], 1),
            3,
            None,
        )
        .expect_err("spoofed sender");
    assert!(error.to_string().contains("sender"), "{error}");
    assert_eq!(service.snapshot(), before);
}

#[test]
fn client_owned_score_goal_and_reward_authority_is_refused() {
    let mut service = running_service();
    let before = service.snapshot();
    let error = service
        .handle_client(KICKER, 0.05, &client_goal([0, 1, 1]), 2, None)
        .expect_err("client GoalMade");
    assert!(error.to_string().contains("goal"), "{error}");
    assert_eq!(service.score(), [0, 0]);
    assert_eq!(service.snapshot(), before);

    // The only located impulse author path is a client-apply path with
    // want-to-send = 0; an inbound sub-13 frame stays refused.
    let impulse = osfr_tools::soccer::encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::ImpulseBall {
            word: 0,
            vector: [1.0, 0.0, 0.0, 0.0],
            value_a: 1.0,
            value_b: 4.0,
            action: 0,
            value_c: 0.0,
        },
    });
    assert!(
        service
            .handle_client(KICKER, 0.05, &impulse, 2, None)
            .is_err()
    );
    assert_eq!(service.snapshot(), before);
    assert_eq!(service.score(), [0, 0]);
    assert!(service.outcome().is_none(), "no fabricated completion");
}

#[test]
fn read_only_client_packets_are_refused_with_their_actual_reason() {
    let mut service = running_service();
    let before = service.snapshot();
    let registration = osfr_tools::soccer::encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::RegisterPlayer {
            player: KICKER,
            marker: [0, 1],
        },
    });
    let error = service
        .handle_client(KICKER, 0.05, &registration, 2, None)
        .expect_err("client registration is server-assigned");
    assert!(error.to_string().contains("read-only"), "{error}");
    let idle = client_state(KICKER, 1, 0, [1.0, 0.0, 0.0, 1.0]);
    let error = service
        .handle_client(KICKER, 0.05, &idle, 2, None)
        .expect_err("non kick/pass state is advisory");
    assert!(error.to_string().contains("read-only"), "{error}");
    assert_eq!(service.snapshot(), before);
}

#[test]
fn malformed_and_non_finite_client_actions_leave_the_match_unchanged() {
    let mut service = running_service();
    let before = service.snapshot();
    let non_finite = client_state(KICKER, 17, 255, [f32::NAN, 0.0, 1.0, 1.0]);
    let error = service
        .handle_client(KICKER, 0.05, &non_finite, 3, None)
        .expect_err("non-finite action direction rejected");
    assert!(error.to_string().contains("finite"), "{error}");
    assert_eq!(service.snapshot(), before);

    let mut truncated = client_state(KICKER, 17, 255, [0.0, 0.0, 1.0, 1.0]);
    truncated.pop();
    let error = service
        .handle_client(KICKER, 0.05, &truncated, 3, None)
        .expect_err("truncated state rejected");
    assert!(error.to_string().contains("length mismatch"), "{error}");
    assert_eq!(service.snapshot(), before);
}

#[test]
fn solo_bots_match_runs_through_shared_lifecycle_to_one_win_payment() {
    let mut config = goal_config();
    config.half_seconds = 3.0;
    let mut service = SoccerService::new(config, policy()).expect("short solo config");
    service.join(KICKER, 0, 0).expect("join human");
    let bots = service
        .fill_bots_for_solo(KICKER)
        .expect("fill solo roster");
    assert_eq!(service.bot_count(), 7);
    assert_eq!(bots.len(), 14);

    let initial = reward_state();
    let offered = service
        .offer_minigame(KICKER, 1201, 1, reward_grant(KICKER, 1))
        .expect("offer Soccer lifecycle");
    assert_eq!(offered.state, Some(RunState::Offered));
    let instance = service.instance_id(KICKER).expect("run id");
    let start = service
        .handle_minigame_lifecycle(KICKER, &minigame_request(5, instance))
        .expect("generic start packet");
    assert_eq!(start.run.state, Some(RunState::Starting));
    assert_eq!(service.phase(), MatchPhase::Kickoff);
    let kickoff = service.mark_started(KICKER).expect("activity ready");
    assert_eq!(service.run_state(KICKER), Some(RunState::Playing));
    assert_eq!(service.phase(), MatchPhase::Running);
    assert!(matches!(
        kickoff.as_slice(),
        [ServiceEvent::GameTime { .. }]
    ));

    service
        .handle_client(
            KICKER,
            0.0,
            &client_position(KICKER, [0.0, 0.0, 0.0]),
            1,
            None,
        )
        .expect("human position");
    service.tick(0.05).expect("ball acquisition tick");
    assert_eq!(service.possession(), Some(KICKER));
    service
        .handle_client(
            KICKER,
            0.05,
            &client_state(KICKER, 17, 255, [-1.0, 0.0, 0.0, 1.0]),
            2,
            None,
        )
        .expect("validated shot");
    for _ in 0..20 {
        if service.phase() != MatchPhase::Running {
            break;
        }
        service.tick(0.25).expect("first-half play");
    }
    assert_eq!(service.score(), [1, 0]);
    assert_eq!(service.phase(), MatchPhase::Restart);
    service.start().expect("resume after goal");
    for _ in 0..20 {
        if service.phase() != MatchPhase::Running {
            break;
        }
        service.tick(0.25).expect("finish first half");
    }
    assert_eq!(service.phase(), MatchPhase::HalfTime);

    service.second_half().expect("second half");
    for _ in 0..20 {
        if service.phase() != MatchPhase::Running {
            break;
        }
        service.tick(0.25).expect("finish second half");
    }
    assert_eq!(service.phase(), MatchPhase::Finished);
    assert_eq!(service.outcome().unwrap().winner, Some(0));

    let result = service
        .finish_minigame(KICKER, &initial)
        .expect("settle generic result");
    assert_eq!(result.state, Some(RunState::Won));
    assert!(!result.replies.is_empty(), "generic result packets exist");
    assert_eq!(
        result.reward_packets.len(),
        2,
        "one coin grant plus display"
    );
    let paid = result.reward_state.as_ref().expect("winner reward state");
    assert_eq!(paid.inventory.state().coins, 1);
    assert_eq!(paid.applied_grant_keys.len(), 1);
    assert!(
        paid.applied_grant_keys
            .contains(&format!("minigame-run:{instance}"))
    );

    let duplicate = service
        .finish_minigame(KICKER, &initial)
        .expect("duplicate result attempt");
    assert!(!duplicate.accepted);
    assert!(duplicate.reward_packets.is_empty());
    assert!(duplicate.reward_state.is_none());
    assert_eq!(
        initial.inventory.state().coins,
        0,
        "staging leaves input intact"
    );
}

#[test]
fn disconnect_mid_match_cancels_run_and_never_pays() {
    let mut service = service();
    service.join(KICKER, 0, 0).expect("join");
    service
        .fill_bots_for_solo(KICKER)
        .expect("fill solo roster");
    let initial = reward_state();
    service
        .offer_minigame(KICKER, 1201, 1, reward_grant(KICKER, 1))
        .expect("offer");
    let instance = service.instance_id(KICKER).unwrap();
    service
        .handle_minigame_lifecycle(KICKER, &minigame_request(5, instance))
        .expect("start packet");
    service.mark_started(KICKER).expect("start match");
    assert_eq!(service.phase(), MatchPhase::Running);
    assert_eq!(service.run_state(KICKER), Some(RunState::Playing));

    service.disconnect(KICKER).expect("disconnect session");
    assert!(service.cancelled());
    assert_eq!(service.session_count(), 0);
    assert!(service.player(KICKER).is_none());
    assert_eq!(service.run_state(KICKER), Some(RunState::Cancelled));
    assert!(service.tick(0.1).is_err(), "cancelled solo match stops");
    assert_eq!(initial.inventory.state().coins, 0);
    assert!(initial.applied_grant_keys.is_empty());
    let result = service
        .finish_minigame(KICKER, &initial)
        .expect("cancelled run cannot settle");
    assert!(!result.accepted);
    assert!(result.reward_packets.is_empty());
    assert!(result.reward_state.is_none());
}

#[test]
fn server_tick_alone_scores_and_the_restart_seam_resumes_play() {
    let mut service = SoccerService::new(goal_config(), policy()).expect("goal config");
    service.join(KICKER, 0, 0).expect("join");
    offer_and_start(&mut service, KICKER);
    service
        .handle_client(
            KICKER,
            0.0,
            &client_position(KICKER, [0.0, 0.0, 0.0]),
            1,
            None,
        )
        .expect("kicker position");
    service.tick(0.05).expect("acquire possession");
    assert_eq!(service.possession(), Some(KICKER));
    service
        .handle_client(
            KICKER,
            0.05,
            &client_state(KICKER, 17, 255, [-1.0, 0.0, 0.0, 1.0]),
            2,
            None,
        )
        .expect("shot at the -x goal (L932: team 0 attacks -x)");

    let mut goal_events = Vec::new();
    for step in 1..=14 {
        if service.phase() != MatchPhase::Running {
            break;
        }
        let events = service.tick(0.25).unwrap_or_else(|error| {
            panic!("tick {step} failed: {error}");
        });
        goal_events.extend(events);
    }
    assert!(
        goal_events
            .iter()
            .any(|event| *event == ServiceEvent::Goal { score: [1, 0] }),
        "only the server tick scores: {goal_events:?}"
    );
    assert_eq!(service.score(), [1, 0]);
    assert_eq!(service.phase(), MatchPhase::Restart);
    assert!(
        service.outcome().is_none(),
        "the restart hold is not a fabricated result"
    );

    let resume = service.start().expect("restart from the goal hold");
    assert!(matches!(resume.as_slice(), [ServiceEvent::GameTime { .. }]));
    assert_eq!(service.phase(), MatchPhase::Running);
    let error = service
        .start()
        .expect_err("a running match cannot kick off again");
    assert!(error.to_string().contains("kickoff"), "{error}");
}

#[test]
fn leaving_releases_the_ball_and_ends_the_session() {
    let mut service = running_service();
    let ball = service.ball_position();
    let events = service.leave(KICKER).expect("leave while holding the ball");
    assert_eq!(
        events,
        vec![ServiceEvent::FreeBall {
            position: ball,
            scale: 1.0,
            previous: KICKER,
        }]
    );
    assert_eq!(service.session_count(), 2);
    assert_eq!(service.possession(), None);
    let error = service
        .handle_client(
            KICKER,
            0.10,
            &client_controls(KICKER, [0.0, -1.0], 1),
            2,
            None,
        )
        .expect_err("a left session is unknown");
    assert!(error.to_string().contains("not joined"), "{error}");
    let error = service.leave(KICKER).expect_err("second leave");
    assert!(error.to_string().contains("not joined"), "{error}");
}

#[test]
fn unsupported_wire_is_refused_and_the_supported_set_round_trips() {
    let supported = [
        SoccerPacket::RegisterPlayer {
            player: 1,
            marker: REGISTER_MARKER,
        },
        SoccerPacket::SetPlayerTeam {
            player: 1,
            team: 0,
            slot: 0,
        },
        SoccerPacket::GoalMade { fields: [0, 1, 1] },
        SoccerPacket::AcquireBall {
            flag: 0,
            holder: 1,
            previous: 0,
        },
        SoccerPacket::FreeBall {
            flag: 0,
            position: [1.0, 2.0, 3.0],
            scale: 1.0,
            previous: 1,
        },
        SoccerPacket::ImpulseBall {
            word: 0,
            vector: [1.0, 0.0, 0.0, 0.0],
            value_a: 1.0,
            value_b: 4.0,
            action: 0,
            value_c: 0.0,
        },
        SoccerPacket::UpdateGameTime {
            remaining: 180.0,
            elapsed: 0.0,
        },
    ];
    for packet in &supported {
        let bytes = emit_packet(packet).expect("candidate codec payload emits");
        let message = soccer_decode(&bytes).expect("wire decodes");
        assert!(message.flag);
        assert_eq!(&message.packet, packet);
    }

    let refused = [
        SoccerPacket::UpdatePlayerPos {
            player: 1,
            position: [0.0; 3],
            tail: [0; 45],
        },
        SoccerPacket::UpdateSoccerBall {
            transform: [0.0; 16],
            velocity: [0.0; 3],
            tail: [0; 15],
        },
        SoccerPacket::UpdateGameState { words: [0; 18] },
        SoccerPacket::UpdatePlayerState {
            player: 1,
            state: 2,
            aux: 0,
            state_counter: 0,
            vectors: [[0.0; 4]; 3],
        },
        SoccerPacket::UpdatePlayerControls {
            player: 1,
            direction: [0.0; 2],
            controls: 0,
        },
        SoccerPacket::UpdatePlayerStats {
            player: 1,
            words: [0; 5],
        },
        SoccerPacket::HighArcBall {
            head: [0.0; 4],
            tail: [0; 5],
        },
        SoccerPacket::BallBounce { words: [0, 1] },
        SoccerPacket::Unknown {
            sub: 17,
            payload: vec![0; 4],
        },
    ];
    for packet in &refused {
        assert!(
            emit_packet(packet).is_err(),
            "{packet:?} is outside the bounded candidate codec output set"
        );
    }
}

#[test]
fn snapshot_restore_seam_preserves_the_authority_and_refuses_a_stale_save() {
    let mut service = running_service();
    service.tick(0.2).expect("tick");
    let saved = service.snapshot();
    let revision = service.revision();
    service.tick(0.2).expect("later tick");
    assert!(service.revision() > revision);

    let error = service
        .restore(&saved)
        .expect_err("an older revision cannot roll the match back");
    assert!(error.to_string().contains("stale"), "{error}");

    let current = service.snapshot();
    service
        .restore(&current)
        .expect("a same-revision snapshot restores in place");
    assert_eq!(service.snapshot(), current);
    assert_eq!(service.score(), [0, 0]);
    assert_eq!(
        service.session_count(),
        3,
        "session entries survive in place"
    );

    let mut restored = SoccerService::new(config(), policy()).expect("fresh service");
    restored
        .restore(&current)
        .expect("the authority snapshot restores into a fresh service");
    assert_eq!(restored.remaining(), current.remaining());
    assert_eq!(restored.elapsed(), current.elapsed());
    assert_eq!(restored.score(), current.score());
    assert_eq!(restored.ball_position(), current.ball_position());
}

#[test]
fn full_match_finishes_with_a_missing_reward_and_no_result_body() {
    let mut service = service();
    service.join(KICKER, 0, 0).expect("join");
    offer_and_start(&mut service, KICKER);
    for _ in 0..720 {
        service.tick(0.25).expect("first half");
    }
    assert_eq!(service.phase(), MatchPhase::HalfTime);
    service.second_half().expect("second half kickoff");
    for _ in 0..720 {
        service.tick(0.25).expect("second half");
    }
    assert_eq!(service.phase(), MatchPhase::Finished);
    let outcome = service.outcome().expect("finished outcome");
    assert_eq!(outcome.score, [0, 0]);
    assert_eq!(outcome.winner, None);
    assert_eq!(
        outcome.reward,
        Evidence::Missing,
        "no native reward table is established"
    );
    let result = service
        .finish_minigame(KICKER, &reward_state())
        .expect("generic result");
    assert_eq!(result.state, Some(RunState::Lost));
    assert!(!result.replies.is_empty());
    assert!(result.reward_packets.is_empty());
    assert!(result.reward_state.is_none());
    // The opaque native game-state/statistics bodies stay outside the R relay.
    assert!(emit_packet(&SoccerPacket::UpdateGameState { words: [0; 18] }).is_err());
    assert!(
        emit_packet(&SoccerPacket::UpdatePlayerStats {
            player: KICKER,
            words: [0; 5]
        })
        .is_err()
    );
}

/// L932 review: with `bot_ai` on, the solo fill runs. A moving bot goes out as a 76/1 pose whose
/// 45-byte tail is w 1.0, facing, move direction, time, speed and one zero byte; the second half
/// lines every bot up again on its kickoff spot at the other end.
#[test]
fn bot_ai_poses_carry_the_live_tail_and_half_time_lines_the_bots_up_again() {
    let mut config = config();
    config.bot_ai = true;
    config.half_seconds = 3.0;
    let mut service = SoccerService::new(config, policy()).expect("service config");
    service.join(KICKER, 0, 0).expect("join");
    service.fill_bots_for_solo(KICKER).expect("bots");
    let poses = |events: &[ServiceEvent]| -> Vec<(u64, f32, f32, Vec<u8>)> {
        events
            .iter()
            .filter_map(|event| match event {
                ServiceEvent::PlayerPos {
                    player,
                    position,
                    speed,
                    ..
                } => {
                    let inner = event.inner();
                    Some((
                        *player,
                        position[0],
                        *speed,
                        inner[inner.len() - 45..].to_vec(),
                    ))
                }
                _ => None,
            })
            .collect()
    };
    let float = |tail: &[u8], at: usize| f32::from_le_bytes(tail[at..at + 4].try_into().unwrap());
    let kickoff = poses(&offer_and_start(&mut service, KICKER));
    assert_eq!(kickoff.len(), 7, "every bot is lined up at kickoff");
    assert!(kickoff.iter().all(|pose| pose.2 == 0.0));

    let mut running = Vec::new();
    for _ in 0..4 {
        running.extend(poses(&service.tick(0.25).expect("tick")));
    }
    let (_, _, speed, tail) = running
        .iter()
        .find(|pose| pose.2 > 0.0)
        .expect("a bot runs");
    assert_eq!(float(tail, 0), 1.0, "w");
    assert_eq!(
        float(tail, 4),
        float(tail, 20),
        "facing and move direction agree"
    );
    assert_eq!(float(tail, 40), *speed, "the speed slot");
    assert_eq!(tail[44], 0);

    for _ in 0..40 {
        match service.phase() {
            MatchPhase::HalfTime => break,
            MatchPhase::Restart => drop(service.start().expect("restart")),
            _ => drop(service.tick(0.25).expect("tick")),
        }
    }
    assert_eq!(service.phase(), MatchPhase::HalfTime);
    let second = poses(&service.second_half().expect("second half"));
    assert_eq!(second.len(), 7, "every bot is lined up again");
    let keepers: Vec<_> = kickoff.iter().filter(|pose| pose.1.abs() > 25.0).collect();
    assert_eq!(keepers.len(), 2);
    for keeper in keepers {
        let after = second.iter().find(|pose| pose.0 == keeper.0).unwrap();
        assert!(
            (after.1 + keeper.1).abs() < 0.01,
            "the keeper swaps ends: {after:?}"
        );
    }
}

/// L940 (native 2026-11-03): with the live setup records, the first kickoff sends them in the live
/// order (state steps, the match config, the teams, every player's 76/21 + 76/19 by role, then the
/// kickoff steps 7, 11, 12); the roster itself went out before, at Loaded, without them.
#[test]
fn setup_records_follow_the_encounter_start_in_the_live_order() {
    let mut service = service();
    let roles = ["human", "team_0", "team_1", "keeper_0", "keeper_1"]
        .into_iter()
        .enumerate()
        .map(|(index, role)| (role.to_owned(), ([index as f32; 42], [index as u32; 9])))
        .collect();
    service.set_setup(Setup { match_config: vec![7; 221], teams: vec![8; 38], roles });
    let mut roster = service.join(KICKER, 0, 0).unwrap();
    roster.extend(service.fill_bots_for_solo(KICKER).unwrap());
    assert!(
        roster.iter().all(|event| !matches!(event, ServiceEvent::Setup(_))),
        "nothing of the setup goes out with the roster at Loaded"
    );
    let events = offer_and_start(&mut service, KICKER);
    let state = |event: &ServiceEvent| match event.packet() {
        SoccerPacket::UpdateGameState { words } => Some((words[12], words[13])),
        _ => None,
    };
    let subs: Vec<String> = events
        .iter()
        .filter(|event| !matches!(event, ServiceEvent::PlayerPos { .. }))
        .map(|event| match state(event) {
            Some((step, players)) => format!("5({step},{players})"),
            None => event.inner()[2].to_string(),
        })
        .collect();
    let stats = ["21", "19"].repeat(8);
    let mut expected = vec!["5(1,0)", "5(2,0)", "7", "5(2,1)", "5(3,1)", "5(4,1)", "5(5,8)", "8"];
    expected.extend(stats);
    // L948: the kickoff drops the ball (76/4), as live with step 11.
    expected.extend(["5(6,8)", "5(7,8)", "5(11,8)", "5(12,8)", "4", "23"]);
    assert_eq!(subs, expected);
    // The human gets the human row; a keeper bot its side's keeper row.
    let human_stats = events.iter().find_map(|event| match event.packet() {
        SoccerPacket::SetSoccerPlayerOneTimeStats { player: KICKER, stats } => Some(stats[0]),
        _ => None,
    });
    assert_eq!(human_stats, Some(0.0));
}

/// L945: the player's own tackle (client 76/14 StealTackle 35 / SlideTackleStart 36) runs the
/// authority's tackle: an opponent in reach of the carrier steals the ball; a teammate cannot.
#[test]
fn a_players_tackle_steals_from_an_opposing_carrier_in_reach() {
    let mut service = running_service();
    assert!(service.player_tackle(TEAMMATE).unwrap().is_empty(), "no tackle on a teammate");
    assert_eq!(service.possession(), Some(KICKER));
    service
        .handle_client(OPPONENT, 0.05, &client_position(OPPONENT, [1.0, 0.0, 0.0]), 1, None)
        .expect("opponent position");
    let events = service.player_tackle(OPPONENT).unwrap();
    // L955: as live, the victim frees the ball (76/12), then the tackler takes it (76/11).
    assert!(matches!(
        events.as_slice(),
        [
            ServiceEvent::FreeBall { previous: KICKER, .. },
            ServiceEvent::Possession {
                holder: OPPONENT,
                previous: KICKER
            }
        ]
    ));
    assert_eq!(service.possession(), Some(OPPONENT), "the tackle steals the ball");
}

/// L955: the player's tackle has a lag allowance over the bots' reach (the client shows the bots a
/// moment behind the server), so a tackle inside the carrier's circle on screen lands; far away it
/// still misses.
#[test]
fn a_players_tackle_allows_for_the_lag_but_not_a_long_way() {
    let mut service = running_service();
    service
        .handle_client(
            OPPONENT,
            0.5,
            &client_position(OPPONENT, [3.5, 0.0, 0.0]),
            1,
            None,
        )
        .expect("opponent far");
    assert!(
        service.player_tackle(OPPONENT).unwrap().is_empty(),
        "3.5 m is out of reach"
    );
    // Past the first slide's window and the slide cooldown.
    for _ in 0..4 {
        service.tick(0.25).expect("server time for the next slide");
    }
    service
        .handle_client(
            OPPONENT,
            1.0,
            &client_position(OPPONENT, [2.8, 0.0, 0.0]),
            2,
            None,
        )
        .expect("opponent near");
    assert!(
        !service.player_tackle(OPPONENT).unwrap().is_empty(),
        "2.8 m lands"
    );
    assert_eq!(service.possession(), Some(OPPONENT));
}

/// L955: the player's slide can take the ball for `PLAYER_SLIDE_WINDOW` (0.5 s) after it starts,
/// as the client slides them on (live resolved tackles 0.1-0.4 s into the slide); after that it
/// is spent. Before, it was judged once, at its start.
#[test]
fn a_players_slide_lands_while_it_closes_in() {
    let slide = |wait: f32| {
        let mut service = running_service();
        service
            .handle_client(
                OPPONENT,
                0.5,
                &client_position(OPPONENT, [5.0, 0.0, 0.0]),
                1,
                None,
            )
            .expect("opponent out of reach");
        assert!(service.player_tackle(OPPONENT).unwrap().is_empty(), "5 m misses");
        service.tick(wait).expect("the slide goes on");
        service
            .handle_client(
                OPPONENT,
                1.0,
                &client_position(OPPONENT, [2.5, 0.0, 0.0]),
                2,
                None,
            )
            .expect("opponent closes in");
        service.tick(0.05).expect("tick");
        service.possession()
    };
    assert_eq!(slide(0.25), Some(OPPONENT), "inside the window the slide lands");
    let mut late = running_service();
    late.player_tackle(OPPONENT).unwrap();
    for _ in 0..3 {
        late.tick(0.25).expect("tick");
    }
    late.handle_client(
        OPPONENT,
        1.0,
        &client_position(OPPONENT, [2.5, 0.0, 0.0]),
        2,
        None,
    )
    .expect("opponent closes in late");
    late.tick(0.05).expect("tick");
    assert_eq!(late.possession(), Some(KICKER), "a spent slide takes nothing");

    // Opus review: spamming the slide does not reopen it inside SLIDE_COOLDOWN.
    let mut spam = running_service();
    spam.player_tackle(OPPONENT).unwrap();
    for _ in 0..3 {
        spam.tick(0.25).expect("tick");
        spam.player_tackle(OPPONENT).unwrap();
    }
    spam.handle_client(
        OPPONENT,
        1.0,
        &client_position(OPPONENT, [2.5, 0.0, 0.0]),
        2,
        None,
    )
    .expect("opponent closes in");
    spam.tick(0.04).expect("tick");
    assert_eq!(
        spam.possession(),
        Some(KICKER),
        "the spammed slides took nothing"
    );
}

/// L955 (Opus review): a slide that lands later in its window sends one free and one acquire, not
/// the pair again from the tick's own possession change.
#[test]
fn a_slide_landing_late_sends_one_free_and_one_acquire() {
    let mut service = running_service();
    service.set_setup(Setup {
        match_config: Vec::new(),
        teams: Vec::new(),
        roles: BTreeMap::new(),
    });
    service
        .handle_client(
            OPPONENT,
            0.5,
            &client_position(OPPONENT, [5.0, 0.0, 0.0]),
            1,
            None,
        )
        .expect("opponent out of reach");
    assert!(service.player_tackle(OPPONENT).unwrap().is_empty());
    service.tick(0.25).expect("the slide goes on");
    service
        .handle_client(
            OPPONENT,
            1.0,
            &client_position(OPPONENT, [2.5, 0.0, 0.0]),
            2,
            None,
        )
        .expect("opponent closes in");
    let events = service.tick(0.05).expect("tick");
    let frees = events
        .iter()
        .filter(|e| matches!(e, ServiceEvent::FreeBall { .. }))
        .count();
    let takes = events
        .iter()
        .filter(|e| matches!(e, ServiceEvent::Possession { .. }))
        .count();
    assert_eq!((frees, takes), (1, 1), "{events:?}");
    assert_eq!(service.possession(), Some(OPPONENT));
}

/// L955: a bot's steal goes out as live sent it: the tackler's SlideTackleStart (36), the victim
/// freeing the ball (76/12) and then the tackler taking it (76/11), then Middle (37) and End (38).
/// Before, the ball changed feet with no animation.
#[test]
fn a_bots_steal_slides_frees_then_acquires() {
    let centre = [226.7, 0.0, 212.9];
    let mut config = MatchConfig::new(FieldBounds {
        centre,
        half_x: 27.0,
        half_z: 17.5,
        goal_half_width: 8.0,
        goal_height: 5.0,
    });
    config.kickoff_spawn = centre;
    config.bot_ai = true;
    let mut service = SoccerService::new(config, policy()).expect("service config");
    service.set_setup(Setup {
        match_config: Vec::new(),
        teams: Vec::new(),
        roles: BTreeMap::new(),
    });
    service.join(KICKER, 0, 0).expect("join");
    service.fill_bots_for_solo(KICKER).expect("bots");
    offer_and_start(&mut service, KICKER);
    let (mut steals, mut middles) = (0, 0);
    for _ in 0..2_000 {
        let events = match service.phase() {
            MatchPhase::Restart => service.start().expect("restart"),
            MatchPhase::HalfTime => service.second_half().expect("second half"),
            MatchPhase::Running => service.tick(0.06).expect("tick"),
            _ => break,
        };
        for (at, event) in events.iter().enumerate() {
            if let ServiceEvent::BotState { player, state: 36, .. } = event {
                steals += 1;
                let rest = &events[at..];
                let freed = rest
                    .iter()
                    .position(|e| matches!(e, ServiceEvent::FreeBall { .. }));
                let taken = rest.iter().position(
                    |e| matches!(e, ServiceEvent::Possession { holder, .. } if holder == player),
                );
                assert!(
                    matches!((freed, taken), (Some(f), Some(t)) if f < t),
                    "free then acquire after the slide: {rest:?}"
                );
            }
            middles += usize::from(matches!(event, ServiceEvent::BotState { state: 37, .. }));
        }
    }
    assert!(steals > 0, "the bots tackled");
    assert!(middles > 0, "the slides played on");
}

/// L946: with the live setup, a goal brings the celebration steps (76/5 16, 17) and the next kickoff
/// ends it first (18, then 11 and 12), as the live server sent them.
#[test]
fn a_goal_celebrates_before_the_next_kickoff() {
    let mut service = SoccerService::new(goal_config(), policy()).expect("goal config");
    service.set_setup(Setup {
        match_config: Vec::new(),
        teams: Vec::new(),
        roles: BTreeMap::new(),
    });
    service.join(KICKER, 0, 0).expect("join");
    offer_and_start(&mut service, KICKER);
    service
        .handle_client(KICKER, 0.0, &client_position(KICKER, [0.0, 0.0, 0.0]), 1, None)
        .expect("kicker position");
    service.tick(0.05).expect("acquire possession");
    let kick = service
        .handle_client(
            KICKER,
            0.05,
            &client_state(KICKER, 17, 255, [-1.0, 0.0, 0.0, 1.0]),
            2,
            None,
        )
        .expect("shot at the -x goal");
    // L948: no 76/13 (the live server sent none); the ball leaves the kicker (76/12) and goes out
    // moving (76/4): in play, physics on, a full-charge kick's effect.
    assert!(matches!(
        kick[0],
        ServiceEvent::FreeBall {
            previous: KICKER,
            ..
        }
    ));
    let (_, velocity, tail) = kick.iter().find_map(ball_of).expect("the kicked ball");
    assert!(velocity[0] < 0.0, "toward the -x goal");
    assert_eq!(tail[4..6], [0xFF, 0xC3]);
    assert_eq!(f32_at(&tail, 6), 10.0);
    assert_eq!(f32_at(&tail, 10), 5.0);
    let mut goal_events = Vec::new();
    while service.phase() == MatchPhase::Running {
        goal_events.extend(service.tick(0.25).expect("tick"));
    }
    let steps = |events: &[ServiceEvent]| -> Vec<u32> {
        events
            .iter()
            .filter_map(|event| match event.packet() {
                SoccerPacket::UpdateGameState { words } => Some(words[12]),
                _ => None,
            })
            .collect()
    };
    assert_eq!(service.score(), [1, 0]);
    assert_eq!(steps(&goal_events), [16, 17]);
    // L947: the stage carries the scoreboard (client handler 0xb82fb6): last scoring team, half,
    // both scores; and the scorer's results row counts the goal (76/15 word 3).
    let words = goal_events
        .iter()
        .rev()
        .find_map(|event| match event.packet() {
            SoccerPacket::UpdateGameState { words } => Some(words),
            _ => None,
        })
        .unwrap();
    assert_eq!(words[..4], [0, 0, 1, 0], "team 0 scored in the first half: 1-0");
    assert!(goal_events.iter().any(|event| matches!(
        event.packet(),
        SoccerPacket::UpdatePlayerStats { player: KICKER, words } if words[3] == 1
    )));
    assert!(service.celebrating());
    // L948: the loose ball went out each tick; the goal's carries the scoring team, out of play.
    let balls: Vec<_> = goal_events.iter().filter_map(ball_of).collect();
    assert!(balls.len() > 1, "the ball streams while it is loose");
    let (_, carried, goal) = balls.last().unwrap();
    assert_eq!(goal[4..6], [0, 0xC2], "team 0 scored; out of play");
    assert!(carried[0] < 0.0, "still moving into the net");
    let restart = service.start().expect("restart");
    assert_eq!(steps(&restart), [18, 11, 12]);
    assert!(!service.celebrating());
    // L948: the kickoff drops the ball on its spot (snap byte), in play again.
    let (transform, velocity, drop) = restart.iter().find_map(ball_of).expect("the kickoff drop");
    assert_eq!((drop[4], drop[5], drop[14]), (0, 0xE3, 1));
    assert_eq!(velocity, [0.0; 3]);
    // L955: the client's ball position is its centre, 0.3 above our floor-level spot.
    let spawn = goal_config().kickoff_spawn;
    assert_eq!(transform[12..15], [spawn[0], spawn[1] + 0.3, spawn[2]]);
}

/// L948: the ball packet's fields (76/4: transform, velocity, the 15-byte tail).
fn ball_of(event: &ServiceEvent) -> Option<([f32; 16], [f32; 3], [u8; 15])> {
    match event.packet() {
        SoccerPacket::UpdateSoccerBall {
            transform,
            velocity,
            tail,
        } => Some((transform, velocity, tail)),
        _ => None,
    }
}

fn f32_at(bytes: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

/// L955: the bots move as live drove them (Fable review): an 8-way control stick (76/16, each axis
/// -1, 0 or 1) on every heading change, a Run or Idle 76/14 only with the stick's starts and stops,
/// and poses that follow the stick at the bot's pace, so the client's own dead reckoning (pos +=
/// stick x speed x dt) lands where the server has the bot. Before, the client parked the bots
/// between poses and they slid (owner, native 2026-11-03).
#[test]
fn bots_move_along_their_control_stick() {
    let centre = [226.7, 0.0, 212.9];
    let mut config = MatchConfig::new(FieldBounds {
        centre,
        half_x: 27.0,
        half_z: 17.5,
        goal_half_width: 8.0,
        goal_height: 5.0,
    });
    config.kickoff_spawn = centre;
    config.bot_ai = true;
    let mut service = SoccerService::new(config, policy()).expect("service config");
    service.set_setup(Setup {
        match_config: Vec::new(),
        teams: Vec::new(),
        roles: BTreeMap::new(),
    });
    service.join(KICKER, 0, 0).expect("join");
    service.fill_bots_for_solo(KICKER).expect("bots");
    offer_and_start(&mut service, KICKER);
    let mut stick: BTreeMap<u64, [f32; 2]> = BTreeMap::new();
    let mut last_pose: BTreeMap<u64, (f32, [f32; 3], [f32; 2], f32)> = BTreeMap::new();
    let (mut changes, mut problems, mut seconds) = (0usize, Vec::new(), 0.0f32);
    let mut kinds = BTreeMap::new();
    for _ in 0..2_000 {
        let events = match service.phase() {
            MatchPhase::Restart => service.start().expect("restart"),
            MatchPhase::HalfTime => service.second_half().expect("second half"),
            MatchPhase::Running => {
                seconds += 0.06;
                service.tick(0.06).expect("tick")
            }
            _ => break,
        };
        let steered: BTreeSet<u64> = events
            .iter()
            .filter_map(|event| match event {
                ServiceEvent::Controls { player, stick } => {
                    if !stick.iter().all(|v| [-1.0, 0.0, 1.0].contains(v)) {
                        problems.push(format!("{player:x}: stick {stick:?}"));
                    }
                    Some(*player)
                }
                _ => None,
            })
            .collect();
        // A kick, tackle, save or celebration has its own states; Run and Idle are the stick's.
        let acting: BTreeSet<u64> = events
            .iter()
            .filter_map(|event| match event {
                ServiceEvent::BotState { player, state, .. } if *state > 2 => Some(*player),
                _ => None,
            })
            .collect();
        for event in &events {
            match event {
                ServiceEvent::Controls { player, stick: now } => {
                    changes += 1;
                    let before = stick.get(player).copied().unwrap_or([0.0; 2]);
                    let kind = match (before == [0.0; 2], *now == [0.0; 2]) {
                        (true, false) => "start",
                        (false, true) => "stop",
                        _ => "turn",
                    };
                    *kinds.entry((*player, kind)).or_insert(0usize) += 1;
                    stick.insert(*player, *now);
                    last_pose.remove(player);
                }
                ServiceEvent::BotState { player, state, .. }
                    if *state <= 2 && !steered.contains(player) && !acting.contains(player) =>
                {
                    // After a kick hold the state goes out again; it must agree with the stick.
                    let standing = stick.get(player).is_none_or(|s| *s == [0.0, 0.0]);
                    if (*state == 1) != standing {
                        problems.push(format!("{player:x}: state {state} against its stick"));
                    }
                }
                ServiceEvent::PlayerPos {
                    player,
                    position,
                    facing,
                    time,
                    speed,
                    snap,
                } => {
                    if *facing == [0.0; 2] {
                        problems.push(format!("{player:x}: zero facing"));
                    }
                    let along = stick.get(player).copied().unwrap_or([0.0; 2]);
                    if let Some((at, from, was, pace)) = last_pose.get(player).copied() {
                        if !snap && along != [0.0; 2] && was == along && pace == *speed && *time > at {
                            let way = [position[0] - from[0], position[2] - from[2]];
                            let length = way[0].hypot(way[1]);
                            // The stick's world heading: normalize(x, -z).
                            let norm = along[0].hypot(along[1]);
                            let heading = [along[0] / norm, -along[1] / norm];
                            let dot = (way[0] * heading[0] + way[1] * heading[1]) / length.max(1e-6);
                            let expected = speed * (time - at);
                            if length > 0.05 && (dot < 0.99 || (length - expected).abs() > 0.05 * expected.max(1.0)) {
                                problems.push(format!(
                                    "{player:x}: moved {way:?} in {:.2} s on stick {along:?} at {speed}",
                                    time - at
                                ));
                            }
                        }
                    }
                    last_pose.insert(*player, (*time, *position, along, *speed));
                }
                _ => {}
            }
        }
    }
    assert!(problems.is_empty(), "{} problems: {:?}", problems.len(), &problems[..problems.len().min(8)]);
    let per_bot_minute = changes as f32 / 7.0 / (seconds / 60.0);
    // Live: about 23 (961 sticks for 7 bots). Ours is about 112 since L956's marking and team
    // shape: about one change per bot per SUPPORT_REPLAN (0.5 s), plus goal line-ups, kicks and
    // chase switches. Per-tick flapping would be about 1,000; a doubled re-plan cadence trips this.
    assert!(
        per_bot_minute < 150.0,
        "stick changes per bot per minute: {per_bot_minute} {kinds:?}"
    );
}

/// L949: with the live setup a keeper's save goes out as its goalie state (76/14 51-62), and a goal
/// leaves the side that let it in disappointed (48-50), as live.
#[test]
fn keepers_save_and_a_goal_disappoints_the_side_that_let_it_in() {
    // The live pitch (`data/content/soccer.json`).
    let centre = [226.7, 0.0, 212.9];
    let mut config = MatchConfig::new(FieldBounds {
        centre,
        half_x: 27.0,
        half_z: 17.5,
        goal_half_width: 8.0,
        goal_height: 5.0,
    });
    config.kickoff_spawn = centre;
    config.bot_ai = true;
    let mut service = SoccerService::new(config, policy()).expect("service config");
    service.set_setup(Setup {
        match_config: Vec::new(),
        teams: Vec::new(),
        roles: BTreeMap::new(),
    });
    service.join(KICKER, 0, 0).expect("join");
    service.fill_bots_for_solo(KICKER).expect("bots");
    offer_and_start(&mut service, KICKER);
    let (mut saved, mut disappointed) = (false, false);
    for _ in 0..8_000 {
        let events = match service.phase() {
            MatchPhase::Restart => service.start().expect("restart"),
            MatchPhase::HalfTime => service.second_half().expect("second half"),
            MatchPhase::Running => service.tick(0.05).expect("tick"),
            _ => break,
        };
        for event in &events {
            if let ServiceEvent::BotState { state, .. } = event {
                saved |= (51..=62).contains(state);
                disappointed |= (48..=50).contains(state);
            }
        }
        if saved && disappointed {
            break;
        }
    }
    assert!(saved, "a keeper saves");
    assert!(disappointed, "a goal disappoints the other side");
}

/// L948: a bot's kick goes out as the live server sent it: its 76/14 stop, charge and kick states,
/// the free ball (76/12), then the moving ball (76/4); never a 76/13. The kicker's next state waits
/// for the kick to play out.
#[test]
fn a_bots_kick_goes_out_as_its_states_and_the_moving_ball() {
    let mut config = config();
    config.bot_ai = true;
    let mut service = SoccerService::new(config, policy()).expect("service config");
    service.set_setup(Setup {
        match_config: Vec::new(),
        teams: Vec::new(),
        roles: BTreeMap::new(),
    });
    service.join(KICKER, 0, 0).expect("join");
    service.fill_bots_for_solo(KICKER).expect("bots");
    let mut events = offer_and_start(&mut service, KICKER);
    let mut kick_at = None;
    for _ in 0..800 {
        let ticked = service.tick(0.05).expect("tick");
        let kicked = ticked.iter().position(|event| {
            matches!(event, ServiceEvent::BotState { state, .. } if *state == 17 || *state == 22)
        });
        if let Some(at) = kicked {
            kick_at = Some(events.len() + at);
        }
        events.extend(ticked);
        if kick_at.is_some() {
            break;
        }
    }
    let kick = kick_at.expect("a bot kicks");
    let ServiceEvent::BotState {
        player: kicker,
        aux,
        counter,
        ..
    } = events[kick]
    else {
        unreachable!()
    };
    let states: Vec<_> = events[kick - 2..=kick]
        .iter()
        .map(|event| match event {
            ServiceEvent::BotState {
                player,
                state,
                aux,
                counter,
                ..
            } if *player == kicker => (*state, *aux, *counter),
            other => panic!("not the kicker's state: {other:?}"),
        })
        .collect();
    assert_eq!(states[..2], [(1, 0, counter - 1), (1, aux, counter - 1)]);
    assert!(aux > 0);
    let free = events[kick..]
        .iter()
        .position(
            |event| matches!(event, ServiceEvent::FreeBall { previous, .. } if *previous == kicker),
        )
        .expect("the ball leaves the kicker");
    let (_, velocity, tail) = events[kick + free..]
        .iter()
        .find_map(ball_of)
        .expect("then the moving ball");
    assert!(velocity.iter().any(|v| *v != 0.0));
    assert_eq!(tail[4..6], [0xFF, 0xC3]);
    assert!(f32_at(&tail, 6) > 0.0, "the kick's effect");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.packet(), SoccerPacket::ImpulseBall { .. })),
        "the live server sent no 76/13"
    );
    // Within the kick's 0.4 s the kicker sends no state.
    let mut next = Vec::new();
    for _ in 0..7 {
        next.extend(service.tick(0.05).expect("tick"));
    }
    let states: Vec<_> = next
        .iter()
        .filter(|event| matches!(event, ServiceEvent::BotState { player, .. } if *player == kicker))
        .collect();
    assert!(states.is_empty(), "{states:?}");
}

/// L952: 76/5 word 0 at the results (step 10) is the winner, which the client compares with the
/// local player's team for YOU WIN / TRY AGAIN. A 4-3 win whose last goal was the other side's
/// showed TRY AGAIN (owner playtest 2026-11-03). Before the results it is the last scorer.
#[test]
fn the_results_name_the_winner_not_the_last_scorer() {
    use soccer_service::scoreboard_team;
    assert_eq!(scoreboard_team(10, [4, 3], Some(1)), 0);
    assert_eq!(scoreboard_team(10, [2, 3], Some(0)), 1);
    assert_eq!(
        scoreboard_team(10, [1, 1], Some(0)),
        u32::MAX,
        "a draw has no winner"
    );
    assert_eq!(
        scoreboard_team(5, [4, 3], Some(1)),
        1,
        "mid-match: the last scorer"
    );
    assert_eq!(scoreboard_team(5, [0, 0], None), u32::MAX);
}
