//! Independent review harness for the MMO005 soccer slice, now fully
//! synthetic (AGENTS: raw captures live in the lab; tests are synthetic).
//! The exact captured-corpus round trip is the ignored, env-gated gate in
//! `tests/soccer.rs`.
//!
//! These tests pin the repaired behavior: the direction gate (client bytes can
//! never score or move authoritative state), goal rejection during the restart
//! hold, finite ball geometry, and registered possession. The four probes that
//! were RED before the repair are ordinary guard tests here; none are ignored.

use osfr_tools::soccer::{
    Direction, Evidence, Match, Message, Phase, SoccerPacket, decode, encode,
};

const HOLDER_SLOT2_TEAM1: u64 = 0x0000_0060_0080_06b2;
const PREVIOUS_HOLDER: u64 = 0x0000_0040_0080_06b2;

fn frame(packet: SoccerPacket) -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet,
    })
}

fn colours() -> Vec<u8> {
    frame(SoccerPacket::TeamColours {
        team_a_name: "sweetpea".to_string(),
        team_b_name: "ocean".to_string(),
        team_a_rgba: [255, 100, 35, 0],
        team_b_rgba: [15, 50, 255, 0],
        team_a_id: 411_073,
        team_b_id: 411_070,
        tail: [0],
    })
}

fn acquire(holder: u64, previous: u64) -> Vec<u8> {
    frame(SoccerPacket::AcquireBall {
        flag: 0,
        holder,
        previous,
    })
}

fn free_ball(position: [f32; 3], previous: u64) -> Vec<u8> {
    frame(SoccerPacket::FreeBall {
        flag: 1,
        position,
        scale: 1.0,
        previous,
    })
}

fn controls() -> Vec<u8> {
    frame(SoccerPacket::UpdatePlayerControls {
        player: 0x0000_0010_0080_06b2,
        direction: [0.0, -1.0],
        controls: 0,
    })
}

fn player_pos() -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: false,
        packet: SoccerPacket::UpdatePlayerPos {
            player: 0x0000_0010_0080_06b2,
            position: [237.22, -0.0086, 203.952],
            tail: [0; 45],
        },
    })
}

fn register(player: u64) -> Vec<u8> {
    frame(SoccerPacket::RegisterPlayer {
        player,
        marker: [0, 1],
    })
}

fn running_match() -> Match {
    let mut game = Match::new();
    game.handle(Direction::ServerToClient, &game_time(180.0, 30.662_061))
        .unwrap();
    assert_eq!(game.phase(), Phase::Running);
    game
}

fn goal(fields: [u32; 3]) -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::GoalMade { fields },
    })
}

fn game_time(remaining: f32, elapsed: f32) -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdateGameTime { remaining, elapsed },
    })
}

#[test]
fn synthetic_vectors_round_trip_and_keep_the_flag_byte() {
    for bytes in [
        goal([0, 1, 1]),
        game_time(180.0, 30.662_061),
        game_time(151.411_88, 60.622_398),
        colours(),
        acquire(HOLDER_SLOT2_TEAM1, PREVIOUS_HOLDER),
        free_ball([243.276, 2.481, 210.306], HOLDER_SLOT2_TEAM1),
        controls(),
        player_pos(),
    ] {
        let message = decode(&bytes).unwrap();
        assert_eq!(encode(&message), bytes, "synthetic round trip");
    }
    let position = decode(&player_pos()).unwrap();
    assert!(!position.flag, "player position keeps the opaque flag 0");
}

#[test]
fn reconstructed_goal_candidate_needs_a_consistent_single_step_scoreboard() {
    // Synthetic GoalMade bytes exercise a candidate relay only. W37 marks goal output and score
    // policy reconstructed; older traces do not establish client facts. See notes/Research/
    // Soccer Service.md (Reconstructed service rules).
    let mut game = running_match();
    assert!(
        game.handle(Direction::ServerToClient, &goal([0, 1, 3]))
            .is_err(),
        "total must equal A + B"
    );
    assert!(
        game.handle(Direction::ServerToClient, &goal([2, 0, 2]))
            .is_err(),
        "score must advance by one"
    );
    game.handle(Direction::ServerToClient, &goal([0, 1, 1]))
        .unwrap();
    assert_eq!(game.score(), [0, 1]);
    assert_eq!(game.goals()[0].scoring_team, 1);
    assert_eq!(game.goals()[0].source, Evidence::Reconstructed);
    // Replaying the goal does not double-count.
    assert!(
        game.handle(Direction::ServerToClient, &goal([0, 1, 1]))
            .is_err(),
        "replayed goal rejected"
    );
    assert_eq!(game.score(), [0, 1]);
}

#[test]
fn clock_rejects_non_finite_and_out_of_order_samples() {
    let mut game = running_match();
    assert!(
        game.handle(Direction::ServerToClient, &game_time(f32::NAN, 40.0))
            .is_err()
    );
    assert!(
        game.handle(Direction::ServerToClient, &game_time(150.0, f32::INFINITY))
            .is_err()
    );
    // elapsed went backwards (kickoff was 30.662061)
    assert!(
        game.handle(Direction::ServerToClient, &game_time(170.0, 10.0))
            .is_err()
    );
    // remaining increased
    assert!(
        game.handle(Direction::ServerToClient, &game_time(181.0, 40.0))
            .is_err()
    );
    // captured descending hold is accepted
    game.handle(Direction::ServerToClient, &game_time(151.412, 60.622))
        .unwrap();
    assert!((game.clock().remaining - 151.412).abs() < 1e-3);
}

#[test]
fn restart_and_end_handling_are_gated_on_the_captured_phases() {
    let mut game = running_match();
    assert!(
        game.restart().is_empty(),
        "no restart during a running half"
    );
    game.handle(Direction::ServerToClient, &goal([0, 1, 1]))
        .unwrap();
    assert_eq!(game.phase(), Phase::Restart);
    game.handle(
        Direction::ServerToClient,
        &game_time(151.411_88, 60.622_398),
    )
    .unwrap();
    let replies = game.restart();
    assert_eq!(replies.len(), 1);
    assert_eq!(game.phase(), Phase::Running);
    assert_eq!(game.restart_source(), Evidence::Reconstructed);
    // Reconstructed half end.
    game.handle(Direction::ServerToClient, &game_time(0.0, 300.0))
        .unwrap();
    assert_eq!(game.phase(), Phase::Finished);
    assert!(game.restart().is_empty(), "no restart after the half ended");
    assert!(
        game.handle(Direction::ServerToClient, &goal([0, 2, 2]))
            .is_err(),
        "no goals after full time"
    );
}

#[test]
fn team_assignment_rejects_duplicate_slots_and_out_of_range_teams() {
    let mut game = Match::new();
    let team = |player: u64, team: u32, slot: u32| {
        encode(&Message {
            outer: 5,
            flag: true,
            packet: SoccerPacket::SetPlayerTeam { player, team, slot },
        })
    };
    game.handle(Direction::ServerToClient, &team(0x40, 1, 0))
        .unwrap();
    // Same bytes again are an idempotent resend, like the capture's repeats.
    game.handle(Direction::ServerToClient, &team(0x40, 1, 0))
        .unwrap();
    assert!(
        game.handle(Direction::ServerToClient, &team(0x50, 1, 0))
            .is_err(),
        "slot already taken"
    );
    assert!(
        game.handle(Direction::ServerToClient, &team(0x40, 0, 1))
            .is_err(),
        "reassignment rejected"
    );
    assert!(
        game.handle(Direction::ServerToClient, &team(0x60, 2, 1))
            .is_err(),
        "team out of range"
    );
    assert!(
        game.handle(Direction::ServerToClient, &team(0x60, 1, 4))
            .is_err(),
        "slot out of range"
    );
}

#[test]
fn possession_chain_and_decode_reject_malformed_input() {
    let mut game = Match::new();
    game.handle(Direction::ServerToClient, &register(PREVIOUS_HOLDER))
        .unwrap();
    game.handle(Direction::ServerToClient, &register(HOLDER_SLOT2_TEAM1))
        .unwrap();
    game.handle(Direction::ServerToClient, &acquire(PREVIOUS_HOLDER, 0))
        .unwrap();
    assert_eq!(game.possession(), Some(PREVIOUS_HOLDER));
    game.handle(
        Direction::ServerToClient,
        &free_ball([242.5, 2.0, 205.0], PREVIOUS_HOLDER),
    )
    .unwrap();
    assert_eq!(game.possession(), None);
    // The chain resumes: 0x60... may only take the ball from 0x40...
    game.handle(
        Direction::ServerToClient,
        &acquire(HOLDER_SLOT2_TEAM1, PREVIOUS_HOLDER),
    )
    .unwrap();
    assert_eq!(game.possession(), Some(HOLDER_SLOT2_TEAM1));
    assert!(
        game.handle(
            Direction::ServerToClient,
            &acquire(HOLDER_SLOT2_TEAM1, PREVIOUS_HOLDER)
        )
        .is_err(),
        "previous-holder chain enforced"
    );
    let wrong_previous = encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::AcquireBall {
            flag: 1,
            holder: 0x20,
            previous: 0x30,
        },
    });
    assert!(
        game.handle(Direction::ServerToClient, &wrong_previous)
            .is_err()
    );
    game.handle(
        Direction::ServerToClient,
        &free_ball([243.276, 2.481, 210.306], HOLDER_SLOT2_TEAM1),
    )
    .unwrap();
    assert_eq!(game.possession(), None);

    let goal_bytes = goal([0, 1, 1]);
    assert!(decode(&goal_bytes[..10]).is_err(), "truncated envelope");
    let mut bad_outer = goal_bytes.clone();
    bad_outer[0] = 7;
    assert!(decode(&bad_outer).is_err(), "non-tunnel outer opcode");
    let mut bad_flag = goal_bytes;
    bad_flag[2] = 2;
    assert!(decode(&bad_flag).is_err(), "unknown flag byte");
    let mut trailing = goal([0, 1, 1]);
    trailing.push(0);
    let size = (trailing.len() - 7) as u32;
    trailing[3..7].copy_from_slice(&size.to_le_bytes());
    assert!(
        decode(&trailing).is_err(),
        "trailing payload bytes rejected"
    );
}

// ---------------------------------------------------------------------------
// Promoted guard tests. These were the four RED counterexamples before the
// repair; each now pins the fixed behavior (findings R1-R4).
// ---------------------------------------------------------------------------

#[test]
fn client_direction_never_scores_or_moves_authoritative_state() {
    // R1: the captured GoalMade is server->client only. Client-direction bytes
    // never change match state, and a goal-shaped client message is refused.
    let mut game = running_match();
    let before = (
        game.phase(),
        game.score(),
        game.clock(),
        game.ball_position(),
    );
    game.handle(Direction::ClientToServer, &controls()).unwrap();
    game.handle(Direction::ClientToServer, &player_pos())
        .unwrap();
    assert_eq!(
        (
            game.phase(),
            game.score(),
            game.clock(),
            game.ball_position()
        ),
        before
    );
    assert!(
        game.handle(Direction::ClientToServer, &goal([1, 0, 1]))
            .is_err(),
        "client goal refused: score={:?}",
        game.score()
    );
    assert_eq!(game.score(), [0, 0]);
    // The same message is accepted when the server is the source.
    game.handle(Direction::ServerToClient, &goal([1, 0, 1]))
        .unwrap();
    assert_eq!(game.score(), [1, 0]);
}

#[test]
fn goal_during_restart_hold_is_rejected() {
    // R2: the capture holds the clock after the goal; the hold is not a
    // scoring window.
    let mut game = running_match();
    game.handle(Direction::ServerToClient, &goal([0, 1, 1]))
        .unwrap();
    game.handle(
        Direction::ServerToClient,
        &game_time(151.411_88, 60.622_398),
    )
    .unwrap();
    assert_eq!(game.phase(), Phase::Restart);
    assert!(
        game.handle(Direction::ServerToClient, &goal([0, 2, 2]))
            .is_err(),
        "goal counted during the frozen hold: score={:?}",
        game.score()
    );
    assert_eq!(game.score(), [0, 1]);
    // After the reconstructed kickoff resumes, the next goal does count.
    assert_eq!(game.restart().len(), 1);
    game.handle(Direction::ServerToClient, &goal([0, 2, 2]))
        .unwrap();
    assert_eq!(game.score(), [0, 2]);
}

#[test]
fn nonfinite_ball_geometry_is_rejected_before_mutation() {
    // R3: clock values already reject NaN/inf; ball geometry must too.
    let mut game = Match::new();
    let mut transform = [0.0f32; 16];
    transform[0] = 1.0;
    transform[5] = 1.0;
    transform[10] = 1.0;
    transform[12] = f32::NAN;
    transform[13] = f32::INFINITY;
    let bad = encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdateSoccerBall {
            transform,
            velocity: [f32::NAN, 0.0, 0.0],
            tail: [0; 15],
        },
    });
    assert!(
        game.handle(Direction::ServerToClient, &bad).is_err(),
        "non-finite ball position stored: {:?}",
        game.ball_position()
    );
    assert_eq!(game.ball_position(), None, "no mutation on rejection");

    // A finite update (the synthetic spawn shape) still lands.
    transform[12] = 226.7;
    transform[13] = 2.0;
    transform[14] = 212.905;
    let good = encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdateSoccerBall {
            transform,
            velocity: [0.0; 3],
            tail: [0; 15],
        },
    });
    game.handle(Direction::ServerToClient, &good).unwrap();
    assert_eq!(game.ball_position(), Some([226.7, 2.0, 212.905]));
}

#[test]
fn unregistered_holder_is_rejected_before_mutation() {
    // R4: possession requires a holder the server registered for this match.
    let mut game = running_match();
    let message = encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::AcquireBall {
            flag: 1,
            holder: PREVIOUS_HOLDER + 0x1234_5678,
            previous: 0,
        },
    });
    assert!(
        game.handle(Direction::ServerToClient, &message).is_err(),
        "unregistered holder accepted: {:?}",
        game.possession()
    );
    assert_eq!(game.possession(), None, "no mutation on rejection");
    // The captured registration for that player is required first.
    game.handle(Direction::ClientToServer, &register(PREVIOUS_HOLDER))
        .unwrap();
    assert_eq!(
        game.possession(),
        None,
        "client registration is not authority"
    );
    game.handle(Direction::ServerToClient, &register(PREVIOUS_HOLDER))
        .unwrap();
    game.handle(Direction::ServerToClient, &acquire(PREVIOUS_HOLDER, 0))
        .unwrap();
    assert_eq!(game.possession(), Some(PREVIOUS_HOLDER));
}
