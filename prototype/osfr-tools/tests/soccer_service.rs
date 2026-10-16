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
use osfr_tools::soccer_match::{FieldBounds, MatchConfig, MatchPhase, PhysicsConfig};
use osfr_tools::zone_transition;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use soccer_service::{
    AdapterPolicy, R_ENTRY_POSITION, R_ENTRY_ROTATION, REGISTER_MARKER, ServiceEvent,
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
    config
}

/// Small pitch: a +z shot at power 4 crosses the goal line after 10 ticks.
fn goal_config() -> MatchConfig {
    let mut config = MatchConfig::new(FieldBounds {
        half_x: 100.0,
        half_z: 10.0,
        goal_half_width: 10.0,
        goal_height: 5.0,
    });
    config.kickoff_spawn = [0.0, 0.0, 0.0];
    config.physics = PhysicsConfig {
        gravity: 0.0,
        ..PhysicsConfig::default()
    };
    config
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
        .offer_minigame(session, 1201, 1, reward_state(), reward_grant(session, 0))
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
    assert_eq!(packet.id, 0);
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
            &client_state(KICKER, 17, 255, [1.0, 0.0, 0.0, 1.0]),
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
    let error = pass_points(&mut service, None).expect_err("pass without a target");
    assert!(error.to_string().contains("target"), "{error}");
    assert_eq!(service.snapshot(), before);
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
        .offer_minigame(KICKER, 1201, 1, initial.clone(), reward_grant(KICKER, 1))
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
            &client_state(KICKER, 17, 255, [0.0, 0.0, 1.0, 1.0]),
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
        .finish_minigame(KICKER)
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
    assert_eq!(
        service
            .reward_state(KICKER)
            .unwrap()
            .inventory
            .state()
            .coins,
        1
    );

    let duplicate = service
        .finish_minigame(KICKER)
        .expect("duplicate result attempt");
    assert!(!duplicate.accepted);
    assert!(duplicate.reward_packets.is_empty());
    assert!(duplicate.reward_state.is_none());
    assert_eq!(
        service
            .reward_state(KICKER)
            .unwrap()
            .inventory
            .state()
            .coins,
        1
    );
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
        .offer_minigame(KICKER, 1201, 1, initial.clone(), reward_grant(KICKER, 1))
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
        .finish_minigame(KICKER)
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
            &client_state(KICKER, 17, 255, [0.0, 0.0, 1.0, 1.0]),
            2,
            None,
        )
        .expect("shot at the +z goal");

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
    let result = service.finish_minigame(KICKER).expect("generic result");
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
