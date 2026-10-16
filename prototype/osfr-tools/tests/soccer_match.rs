//! Focused tests for the server-owned soccer match authority
//! (`prototype/osfr-tools/src/soccer_match.rs`).
//!
//! History: the first RED run (missing module) is
//! `lab/reports/soccer-match-authority-20261009T0315Z/red-test.log`; the F1-F6
//! repair RED probes (five behavioral failures against the reviewed source)
//! are `lab/reports/soccer-match-authority-20261009T0745Z/red-probes.log`, and
//! the post-repair GREEN run is the sibling `green.log`.
//!
//! Field, goal, spawn and clock values below are synthetic R policy. No native
//! match geometry or duration is claimed.

use std::io;

use osfr_tools::soccer::{Evidence, HALF_SECONDS, Message, SoccerPacket, encode};
use osfr_tools::soccer_match::{
    Authority, ClientPacket, FieldBounds, HalfPolicy, Intent, KickKind, MatchConfig, MatchPhase,
    client_intent, client_packet,
};

// Synthetic client packets built from the codec layouts.
fn vec_player_pos() -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: false,
        packet: SoccerPacket::UpdatePlayerPos {
            player: 0x0000_0010_0080_06b2,
            position: [7.25, 0.0, 9.5],
            tail: [0; 45],
        },
    })
}

fn vec_controls() -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdatePlayerControls {
            player: 0x0000_0010_0080_06b2,
            direction: [0.0, -1.0],
            controls: 0,
        },
    })
}

fn vec_player_state() -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdatePlayerState {
            player: 0x0000_0010_0080_06b2,
            state: 2,
            aux: 0,
            state_counter: 1,
            vectors: [
                [0.0, 1.0, 0.0, 0.0],
                [0.824_885, 0.0, 0.565_299, 0.0],
                [7.25, 0.0, 9.5, 1.0],
            ],
        },
    })
}

fn vec_register() -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::RegisterPlayer {
            player: 0,
            marker: [0, 0],
        },
    })
}

fn vec_goal() -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::GoalMade { fields: [0, 1, 1] },
    })
}

fn vec_impulse() -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::ImpulseBall {
            word: 0,
            vector: [0.0; 4],
            value_a: 0.0,
            value_b: 0.0,
            action: 0,
            value_c: 0.0,
        },
    })
}

/// Arbitrary caller-supplied test bounds; not historical values.
fn bounds() -> FieldBounds {
    FieldBounds {
        half_x: 50.0,
        half_z: 30.0,
        goal_half_width: 4.0,
        goal_height: 3.0,
    }
}

fn config() -> MatchConfig {
    let mut config = MatchConfig::new(bounds());
    // Deterministic linear motion for the tests; the equations are unchanged.
    config.physics.gravity = 0.0;
    config
}

fn authority() -> Authority {
    let mut game = Authority::new(config()).unwrap();
    game.register(0x10, 0, 0).unwrap();
    game.register(0x11, 0, 1).unwrap();
    game.register(0x20, 1, 0).unwrap();
    game.kickoff().unwrap();
    game
}

fn position(player: u64, sequence: u64, position: [f32; 3]) -> Intent {
    Intent::Position {
        player,
        sequence,
        position,
    }
}

fn shot(player: u64, sequence: u64, direction: [f32; 3], power: f32) -> Intent {
    Intent::Kick {
        player,
        sequence,
        kind: KickKind::Shot,
        direction,
        power,
        target: None,
    }
}

fn pass(player: u64, sequence: u64, target: u64, power: f32) -> Intent {
    Intent::Kick {
        player,
        sequence,
        kind: KickKind::Pass,
        direction: [0.0, 0.0, 1.0],
        power,
        target: Some(target),
    }
}

fn pos(x: f32, z: f32) -> [f32; 3] {
    [x, 1.0, z]
}

/// Send one intent from `player` at the authority's current server time.
fn send(game: &mut Authority, player: u64, intent: Intent) -> io::Result<()> {
    let now = game.server_time();
    game.apply(player, now, intent)
}

/// Walk a player through bounded waypoints. Each step spends `0.25 s` of
/// authoritative server time, which is the movement budget for a `10`-unit
/// step at the default `max_speed`; returns the next free sequence.
fn walk(game: &mut Authority, player: u64, mut sequence: u64, waypoints: &[[f32; 3]]) -> u64 {
    for point in waypoints {
        game.advance(0.25).unwrap();
        send(game, player, position(player, sequence, *point)).unwrap();
        sequence += 1;
    }
    sequence
}

fn run_half(game: &mut Authority) {
    let mut ticks = 0;
    while game.phase() == MatchPhase::Running && ticks < 100_000 {
        game.advance(0.25).unwrap();
        ticks += 1;
    }
}

#[test]
fn client_goal_ball_and_unmapped_packets_are_rejected() {
    // GoalMade is server -> client; a client cannot assert it.
    assert!(client_intent(&vec_goal(), 1).is_err());
    assert!(client_packet(&vec_goal(), 1).is_err());
    let spoofed = encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::GoalMade { fields: [1, 0, 1] },
    });
    assert!(client_intent(&spoofed, 1).is_err());

    // A client cannot move the ball directly (sub 4 UpdateSoccerBall).
    let mut transform = [0.0f32; 16];
    transform[0] = 1.0;
    transform[5] = 1.0;
    transform[10] = 1.0;
    let ball = encode(&Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdateSoccerBall {
            transform,
            velocity: [0.0; 3],
            tail: [0; 15],
        },
    });
    assert!(client_packet(&ball, 1).is_err());

    // ImpulseBall (13) has a recovered layout, but no authenticated
    // client-to-server send was observed: it stays refused by policy.
    assert!(client_intent(&vec_impulse(), 1).is_err());

    let game = authority();
    assert_eq!(game.score(), [0, 0]);
}

#[test]
fn client_packets_map_to_position_and_controls_intents() {
    let packet = client_packet(&vec_player_pos(), 7).unwrap();
    match packet {
        ClientPacket::Intent(intent) => {
            assert_eq!(intent.player(), 0x0000_0010_0080_06b2);
            assert_eq!(intent.sequence(), 7);
            match intent {
                Intent::Position { position, .. } => {
                    assert!((position[0] - 7.25).abs() < 0.01, "{position:?}");
                }
                other => panic!("unexpected {other:?}"),
            }
        }
        other => panic!("unexpected {other:?}"),
    }
    let intent = client_intent(&vec_controls(), 8).unwrap();
    assert_eq!(intent.player(), 0x0000_0010_0080_06b2);
    match intent {
        Intent::Controls { direction, .. } => assert_eq!(direction, [0.0, -1.0]),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn registration_and_player_state_are_read_only_decodes() {
    // F5: the client sends ids 3 and 14; both decode
    // read-only rather than using client inputs as server authority.
    match client_packet(&vec_register(), 1).unwrap() {
        ClientPacket::ReadOnly(SoccerPacket::RegisterPlayer { player, .. }) => {
            assert_eq!(
                player, 0,
                "client registration request carries no resolved player"
            );
        }
        other => panic!("unexpected {other:?}"),
    }
    match client_packet(&vec_player_state(), 2).unwrap() {
        ClientPacket::ReadOnly(SoccerPacket::UpdatePlayerState {
            player, vectors, ..
        }) => {
            assert_eq!(player, 0x0000_0010_0080_06b2);
            assert!((vectors[2][0] - 7.25).abs() < 0.01, "{:?}", vectors[2]);
        }
        other => panic!("unexpected {other:?}"),
    }
    // The intent-only view refuses them with their actual reason.
    let error = client_intent(&vec_register(), 1).unwrap_err();
    assert!(error.to_string().contains("read-only"), "{error}");
    let error = client_intent(&vec_player_state(), 2).unwrap_err();
    assert!(error.to_string().contains("read-only"), "{error}");
}

#[test]
fn intents_reject_unregistered_stale_and_non_finite_input() {
    let mut game = authority();
    assert!(
        send(&mut game, 0x99, position(0x99, 1, pos(0.0, 0.0))).is_err(),
        "unregistered player"
    );
    assert!(
        send(&mut game, 0x10, position(0x10, 0, pos(0.0, 0.0))).is_err(),
        "sequence 0 is not newer than the initial state"
    );
    assert!(
        send(&mut game, 0x10, position(0x10, 1, [1.0, f32::NAN, 1.0])).is_err(),
        "non-finite position"
    );
    send(&mut game, 0x10, position(0x10, 1, pos(1.0, 0.0))).unwrap();
    assert!(
        send(&mut game, 0x10, position(0x10, 1, pos(1.0, 0.0))).is_err(),
        "stale sequence"
    );
    assert!(
        send(
            &mut game,
            0x11,
            Intent::Controls {
                player: 0x11,
                sequence: 1,
                direction: [f32::NAN, 0.0],
                controls: 0,
            }
        )
        .is_err(),
        "non-finite direction"
    );
    // Rejected intents do not consume the sequence.
    game.advance(0.25).unwrap();
    send(&mut game, 0x10, position(0x10, 2, pos(1.5, 0.0))).unwrap();
}

#[test]
fn sender_identity_is_checked_before_any_victim_mutation() {
    // F1/F2: the authenticated sender is separate from the payload, and a
    // mismatch is refused before the victim's state is touched.
    let mut game = authority();
    let victim = 0x11u64;
    assert!(
        send(
            &mut game,
            0x10,
            Intent::Controls {
                player: victim,
                sequence: u64::MAX,
                direction: [0.0, 0.0],
                controls: 0,
            }
        )
        .is_err(),
        "spoofed controls accepted"
    );
    assert!(
        send(&mut game, 0x10, position(victim, u64::MAX, pos(3.0, 4.0))).is_err(),
        "spoofed position accepted"
    );
    assert_eq!(game.player(victim).unwrap().sequence, 0, "victim untouched");
    // The victim's own next intent still applies.
    send(&mut game, victim, position(victim, 1, pos(1.0, 0.0))).unwrap();
    assert_eq!(game.player(victim).unwrap().sequence, 1);
}

#[test]
fn position_intents_are_bounded_by_distance_field_and_server_time() {
    // F3: a position step costs authoritative server time.
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(1.0, 0.0))).unwrap();
    assert!(
        send(&mut game, 0x10, position(0x10, 2, pos(2.0, 0.0))).is_err(),
        "zero-elapsed movement accepted"
    );
    game.advance(0.25).unwrap(); // budget = 40 * 0.25 = 10
    send(&mut game, 0x10, position(0x10, 2, pos(5.0, 0.0))).unwrap();
    game.advance(0.1).unwrap(); // budget = 4
    assert!(
        send(&mut game, 0x10, position(0x10, 3, pos(11.0, 0.0))).is_err(),
        "step 6 within max_position_step but beyond the time budget"
    );
    game.advance(0.15).unwrap(); // total 0.25 since the last position
    send(&mut game, 0x10, position(0x10, 3, pos(11.0, 0.0))).unwrap();
    assert!(
        send(&mut game, 0x10, position(0x10, 4, pos(22.0, 0.0))).is_err(),
        "single step beyond max_position_step"
    );
    assert!(
        send(&mut game, 0x10, position(0x10, 4, pos(51.0, 0.0))).is_err(),
        "outside the caller-supplied field"
    );
}

#[test]
fn intent_clock_is_trusted_and_clamped() {
    // The `now` argument is the gateway's trusted clock, never a payload
    // value: malformed times are refused, stale times grant no budget, and a
    // forward spike grants no extra budget and never moves the clock.
    let mut game = authority();
    game.apply(0x10, 0.0, position(0x10, 1, pos(1.0, 0.0)))
        .unwrap();
    game.advance(0.25).unwrap();
    assert!(
        game.apply(0x10, f32::NAN, position(0x10, 2, pos(2.0, 0.0)))
            .is_err(),
        "non-finite time"
    );
    assert!(
        game.apply(0x10, -1.0, position(0x10, 2, pos(2.0, 0.0)))
            .is_err(),
        "negative time"
    );
    assert!(
        game.apply(0x10, 0.0, position(0x10, 2, pos(2.0, 0.0)))
            .is_err(),
        "stale clock grants no movement budget"
    );
    assert!(
        game.apply(0x10, 1000.0, position(0x10, 2, pos(30.0, 0.0)))
            .is_err(),
        "forward spike is not a teleport"
    );
    assert_eq!(game.server_time(), 0.25, "intents never move the clock");
    // The real timestamp still applies the full allowance, exactly.
    game.apply(0x10, 0.25, position(0x10, 2, pos(11.0, 0.0)))
        .unwrap();
    assert_eq!(
        game.player(0x10).unwrap().position.unwrap(),
        [11.0, 1.0, 0.0]
    );
}

#[test]
fn client_packets_never_supply_time() {
    let first = client_packet(&vec_player_pos(), 5).unwrap();
    let second = client_packet(&vec_player_pos(), 5).unwrap();
    assert_eq!(first, second, "the packet mapping carries no timing");
    let mut game = authority();
    let clock = game.server_time();
    let _ = game.apply(0x10, 1234.5, position(0x10, 1, pos(1.0, 0.0)));
    assert_eq!(
        game.server_time(),
        clock,
        "only advance and restore move the authority clock"
    );
}

#[test]
fn intents_are_rejected_outside_running_play() {
    let mut game = Authority::new(config()).unwrap();
    game.register(0x10, 0, 0).unwrap();
    assert_eq!(game.phase(), MatchPhase::Kickoff);
    assert!(send(&mut game, 0x10, position(0x10, 1, pos(0.0, 0.0))).is_err());
    game.kickoff().unwrap();
    send(&mut game, 0x10, position(0x10, 1, pos(0.0, 0.0))).unwrap();
    assert!(game.kickoff().is_err(), "kickoff only from Kickoff/Restart");
}

#[test]
fn kick_requires_possession_and_proximity_to_the_ball() {
    let mut game = authority();
    assert!(
        send(&mut game, 0x11, shot(0x11, 1, [0.0, 0.0, 1.0], 5.0)).is_err(),
        "no possession"
    );
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.25).unwrap();
    assert_eq!(game.possession(), Some(0x10), "server assigned possession");

    // Walk away from the ball (bounded step) and the kick is refused.
    let next = walk(&mut game, 0x10, 2, &[pos(0.5, 9.0)]);
    assert!(
        send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, 1.0], 5.0)).is_err(),
        "holder too far from the ball"
    );

    // Return, let possession re-pin the ball, then the power and direction
    // guards hold while in possession.
    let next = walk(&mut game, 0x10, next + 1, &[pos(0.5, 0.0)]);
    game.advance(0.02).unwrap();
    assert!(
        send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, 1.0], 1e6)).is_err(),
        "power beyond max_kick_power"
    );
    assert!(
        send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, 0.0], 5.0)).is_err(),
        "zero direction"
    );
    send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, 1.0], 5.0)).unwrap();
    assert_eq!(game.possession(), None);
    assert_eq!(game.ball_velocity(), [0.0, 0.0, 5.0]);
}

#[test]
fn ball_motion_is_deterministic_and_clock_advances() {
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.25).unwrap();
    send(&mut game, 0x10, shot(0x10, 2, [0.0, 0.0, 1.0], 5.0)).unwrap();
    let before = game.ball_position();
    let remaining = game.remaining();
    let server = game.server_time();
    game.advance(0.1).unwrap();
    let after = game.ball_position();
    assert!((after[0] - before[0]).abs() < 1e-4);
    assert!((after[2] - (before[2] + 0.5)).abs() < 1e-4, "{after:?}");
    assert!((remaining - game.remaining() - 0.1).abs() < 1e-4);
    assert!((game.server_time() - server - 0.1).abs() < 1e-4);

    // Gravity is a caller-supplied reconstructed constant applied after the
    // position step: v_y = power - g*dt.
    let mut config = MatchConfig::new(bounds());
    config.physics.gravity = 9.81;
    let mut game = Authority::new(config).unwrap();
    game.register(0x10, 0, 0).unwrap();
    game.kickoff().unwrap();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.25).unwrap();
    send(&mut game, 0x10, shot(0x10, 2, [0.0, 1.0, 0.0], 9.81)).unwrap();
    game.advance(0.1).unwrap();
    assert!((game.ball_velocity()[1] - (9.81 - 9.81 * 0.1)).abs() < 1e-3);
}

#[test]
fn possession_is_assigned_by_the_authority_near_the_ball() {
    let mut game = authority();
    // Nobody near the ball: possession stays unassigned.
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), None);
    // A registered player at distance ~5 (beyond control_radius 2) misses it.
    send(&mut game, 0x11, position(0x11, 1, pos(5.0, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), None);
    // The closest registered player inside control_radius gets the ball.
    send(&mut game, 0x20, position(0x20, 1, pos(1.0, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(0x20));
    // Possession carries the ball with the holder on the next step.
    assert!((game.ball_position()[0] - 1.0).abs() < 1e-4);
}

#[test]
fn pass_progresses_to_the_target_and_transfers_possession() {
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(0x10));
    send(&mut game, 0x11, position(0x11, 1, pos(0.0, 6.0))).unwrap();
    send(&mut game, 0x10, pass(0x10, 2, 0x11, 4.0)).unwrap();
    assert_eq!(game.possession(), None);
    let mut steps = 0;
    while game.possession().is_none() && steps < 40 {
        game.advance(0.1).unwrap();
        steps += 1;
    }
    assert_eq!(game.possession(), Some(0x11), "pass reached the teammate");
    assert_eq!(game.ball_velocity(), [0.0, 0.0, 0.0]);
}

#[test]
fn pass_requires_a_registered_teammate_in_range() {
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    assert!(
        send(&mut game, 0x10, pass(0x10, 2, 0x20, 4.0)).is_err(),
        "enemy target rejected"
    );
    assert!(
        send(&mut game, 0x10, pass(0x10, 2, 0x77, 4.0)).is_err(),
        "unregistered target rejected"
    );
    walk(
        &mut game,
        0x11,
        1,
        &[
            pos(9.0, 0.0),
            pos(18.0, 0.0),
            pos(27.0, 0.0),
            pos(36.0, 0.0),
            pos(45.0, 0.0),
        ],
    );
    assert!(
        send(&mut game, 0x10, pass(0x10, 2, 0x11, 4.0)).is_err(),
        "target beyond pass_radius"
    );
}

#[test]
fn goal_crossing_scores_once_inside_the_mouth_only() {
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    // Bounded steps to the +z goal line that team 0 attacks in half 1.
    let next = walk(
        &mut game,
        0x10,
        2,
        &[pos(0.0, 9.0), pos(0.0, 18.0), pos(0.0, 28.0)],
    );
    game.advance(0.02).unwrap(); // dribble: the ball follows the holder
    send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, 1.0], 5.0)).unwrap();
    for _ in 0..8 {
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.1).unwrap();
    }
    assert_eq!(game.phase(), MatchPhase::Restart);
    assert_eq!(game.score(), [1, 0], "team 0 attacks +z");
    assert_eq!(game.ball_velocity(), [0.0, 0.0, 0.0]);
    // Once-only: no further advance or scoring while the goal is latched.
    assert!(game.advance(0.1).is_err());
    assert_eq!(game.score(), [1, 0]);

    // Outside the mouth: no goal, and the ball restarts at the spawn instead
    // of stranding (F4).
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    let next = walk(
        &mut game,
        0x10,
        2,
        &[
            pos(5.0, 4.5),
            pos(10.0, 9.0),
            pos(10.0, 18.0),
            pos(10.0, 28.0),
        ],
    );
    game.advance(0.02).unwrap();
    send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, 1.0], 5.0)).unwrap();
    for _ in 0..8 {
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.1).unwrap();
    }
    assert_eq!(game.score(), [0, 0]);
    assert_eq!(game.phase(), MatchPhase::Restart);
    assert_eq!(game.ball_position(), game.config().kickoff_spawn);
}

#[test]
fn out_of_play_restarts_bounded_at_the_spawn() {
    // F4: a miss over the touchline or byline restarts at the spawn through
    // kickoff() instead of leaving a dead ball.
    let mut config = MatchConfig::new(FieldBounds {
        half_x: 10.0,
        half_z: 30.0,
        goal_half_width: 4.0,
        goal_height: 3.0,
    });
    config.physics.gravity = 0.0;
    let mut game = Authority::new(config).unwrap();
    game.register(0x10, 0, 0).unwrap();
    game.kickoff().unwrap();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    let next = walk(
        &mut game,
        0x10,
        2,
        &[pos(4.0, 9.0), pos(8.0, 18.0), pos(8.0, 28.0)],
    );
    game.advance(0.02).unwrap();
    send(&mut game, 0x10, shot(0x10, next, [1.0, 0.0, 0.0], 5.0)).unwrap();
    for _ in 0..8 {
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.1).unwrap();
    }
    assert_eq!(game.phase(), MatchPhase::Restart, "bounded restart");
    assert_eq!(game.score(), [0, 0]);
    assert_eq!(game.ball_position(), game.config().kickoff_spawn);
    assert_eq!(game.ball_velocity(), [0.0; 3]);
    assert_eq!(game.possession(), None);
    // Play resumes through the normal kickoff path.
    game.kickoff().unwrap();
    assert_eq!(game.phase(), MatchPhase::Running);
    assert_eq!(game.ball_position(), game.config().kickoff_spawn);
}

#[test]
fn kickoff_reset_resumes_play_and_preserves_the_score() {
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    let next = walk(
        &mut game,
        0x10,
        2,
        &[pos(0.0, 9.0), pos(0.0, 18.0), pos(0.0, 28.0)],
    );
    game.advance(0.02).unwrap();
    send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, 1.0], 30.0)).unwrap();
    for _ in 0..8 {
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.1).unwrap();
    }
    assert_eq!(game.score(), [1, 0]);
    game.kickoff().unwrap();
    assert_eq!(game.phase(), MatchPhase::Running);
    assert_eq!(game.score(), [1, 0], "score preserved across kickoff");
    assert_eq!(game.ball_position(), game.config().kickoff_spawn);

    // The same route scores again: scoring is once per crossing, not once per
    // match. Walk back to the ball in bounded steps, then out to the line.
    let next = walk(
        &mut game,
        0x10,
        next + 1,
        &[pos(0.0, 18.0), pos(0.5, 9.0), pos(0.5, 0.0)],
    );
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(0x10));
    let next = walk(
        &mut game,
        0x10,
        next,
        &[pos(0.0, 9.0), pos(0.0, 18.0), pos(0.0, 28.0)],
    );
    game.advance(0.02).unwrap();
    send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, 1.0], 30.0)).unwrap();
    for _ in 0..8 {
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.1).unwrap();
    }
    assert_eq!(game.score(), [2, 0]);
}

#[test]
fn halves_transition_and_the_finished_outcome_is_once_only() {
    let mut config = MatchConfig::new(bounds());
    config.half_seconds = 1.0; // R: caller-supplied test duration.
    config.physics.gravity = 0.0;
    let mut game = Authority::new(config).unwrap();
    game.register(0x10, 0, 0).unwrap();
    game.kickoff().unwrap();
    run_half(&mut game);
    assert_eq!(game.phase(), MatchPhase::HalfTime);
    assert!(game.advance(0.25).is_err(), "no play during the interval");
    game.start_second_half().unwrap();
    assert_eq!(game.half(), 2);
    assert_eq!(game.phase(), MatchPhase::Running);
    run_half(&mut game);
    assert_eq!(game.phase(), MatchPhase::Finished);
    let outcome = game.outcome().unwrap();
    assert_eq!(outcome.score, [0, 0]);
    assert_eq!(outcome.winner, None);
    assert_eq!(outcome.reward, Evidence::Missing, "no price is invented");
    assert_eq!(outcome.half_seconds_evidence, Evidence::Reconstructed);
    assert_eq!(game.outcome(), Some(outcome), "outcome is stable");
    assert!(game.advance(0.25).is_err());
    assert!(game.kickoff().is_err(), "no restart after full time");
    assert!(game.start_second_half().is_err());
}

#[test]
fn second_half_switches_attack_sides() {
    // F6: with the default SwitchSides policy, team 0 attacks -z in half 2.
    let mut config = MatchConfig::new(bounds());
    config.half_seconds = 5.0;
    config.physics.gravity = 0.0;
    assert_eq!(config.half_policy, HalfPolicy::SwitchSides);
    let mut game = Authority::new(config).unwrap();
    game.register(0x10, 0, 0).unwrap();
    game.kickoff().unwrap();
    run_half(&mut game);
    assert_eq!(game.phase(), MatchPhase::HalfTime);
    game.start_second_half().unwrap();
    assert_eq!(game.half(), 2);
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    let next = walk(
        &mut game,
        0x10,
        2,
        &[pos(0.0, -9.0), pos(0.0, -18.0), pos(0.0, -28.0)],
    );
    game.advance(0.02).unwrap();
    send(&mut game, 0x10, shot(0x10, next, [0.0, 0.0, -1.0], 30.0)).unwrap();
    for _ in 0..8 {
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.1).unwrap();
    }
    assert_eq!(game.score(), [1, 0], "team 0 attacks -z after the interval");
}

#[test]
fn keep_sides_policy_is_caller_owned() {
    // F6: the caller can keep the first-half ends instead.
    let mut config = MatchConfig::new(bounds());
    config.half_seconds = 5.0;
    config.physics.gravity = 0.0;
    config.half_policy = HalfPolicy::KeepSides;
    let mut game = Authority::new(config).unwrap();
    game.register(0x20, 1, 0).unwrap();
    game.kickoff().unwrap();
    run_half(&mut game);
    game.start_second_half().unwrap();
    // Team 1 still attacks -z, so its shot over the -z line scores for team 1.
    send(&mut game, 0x20, position(0x20, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    let next = walk(
        &mut game,
        0x20,
        2,
        &[pos(0.0, -9.0), pos(0.0, -18.0), pos(0.0, -28.0)],
    );
    game.advance(0.02).unwrap();
    send(&mut game, 0x20, shot(0x20, next, [0.0, 0.0, -1.0], 30.0)).unwrap();
    for _ in 0..8 {
        if game.phase() != MatchPhase::Running {
            break;
        }
        game.advance(0.1).unwrap();
    }
    assert_eq!(
        game.score(),
        [0, 1],
        "team 1 keeps the -z attack with KeepSides"
    );
}

#[test]
fn fallback_half_length_is_labelled_reconstructed() {
    let mut game = Authority::new(MatchConfig::new(bounds())).unwrap();
    assert!((game.remaining() - HALF_SECONDS).abs() < 1e-4);
    game.register(0x10, 0, 0).unwrap();
    game.kickoff().unwrap();
    run_half(&mut game);
    game.start_second_half().unwrap();
    run_half(&mut game);
    let outcome = game.outcome().unwrap();
    assert_eq!(outcome.half_seconds, HALF_SECONDS);
    assert_eq!(outcome.half_seconds_evidence, Evidence::Reconstructed);

    // A caller-supplied half is labelled reconstructed instead.
    let mut config = MatchConfig::new(bounds());
    config.half_seconds = 300.0;
    let mut game = Authority::new(config).unwrap();
    game.register(0x10, 0, 0).unwrap();
    game.kickoff().unwrap();
    run_half(&mut game);
    assert_eq!(game.phase(), MatchPhase::HalfTime);
    assert!(game.outcome().is_none(), "no outcome before full time");
    game.start_second_half().unwrap();
    run_half(&mut game);
    assert_eq!(
        game.outcome().unwrap().half_seconds_evidence,
        Evidence::Reconstructed
    );
}

#[test]
fn snapshot_restore_survives_reconnect_and_rejects_replays() {
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(0.5, 0.0))).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(0x10));
    let early = game.snapshot();
    let revision = game.revision();

    // Reconnect: a fresh authority restores the saved state exactly.
    let mut reconnected = Authority::new(game.config().clone()).unwrap();
    reconnected.restore(&early).unwrap();
    assert_eq!(reconnected.score(), game.score());
    assert_eq!(reconnected.possession(), Some(0x10));
    assert!((reconnected.remaining() - game.remaining()).abs() < 1e-4);
    // The receiver keeps its own clock domain; the snapshot's movement clock
    // is rebased onto it (allowances are preserved, see the rebase test).
    assert_eq!(reconnected.server_time(), 0.0, "receiver clock kept");
    // Replayed intents are stale after the restore.
    assert!(
        send(&mut reconnected, 0x10, position(0x10, 1, pos(0.5, 0.0))).is_err(),
        "replayed intent"
    );

    // A newer state rejects an older snapshot.
    game.advance(0.25).unwrap();
    send(&mut game, 0x10, position(0x10, 2, pos(3.0, 0.0))).unwrap();
    assert!(game.revision() > revision);
    assert!(game.restore(&early).is_err(), "stale snapshot");
    let late = game.snapshot();
    game.restore(&late).unwrap();
}

#[test]
fn field_bounds_and_registration_are_validated() {
    let mut bad = bounds();
    bad.half_x = 0.0;
    assert!(Authority::new(MatchConfig::new(bad)).is_err());
    let mut bad = bounds();
    bad.half_z = f32::NAN;
    assert!(Authority::new(MatchConfig::new(bad)).is_err());
    let mut bad = bounds();
    bad.goal_half_width = 60.0;
    assert!(
        Authority::new(MatchConfig::new(bad)).is_err(),
        "goal wider than the pitch"
    );
    let mut game = Authority::new(config()).unwrap();
    assert!(game.register(0x10, 2, 0).is_err(), "team out of range");
    assert!(game.register(0x10, 0, 4).is_err(), "slot out of range");
    game.register(0x10, 0, 0).unwrap();
    assert!(game.register(0x11, 0, 0).is_err(), "slot taken");
    assert!(game.register(0x10, 1, 0).is_err(), "reassignment");
}

#[test]
fn caller_spawn_coordinates_and_state_accessors_are_usable() {
    // R: the spawn and bounds are caller-supplied test values.
    const TEST_SPAWN: [f32; 3] = [10.0, 0.0, 10.0];
    let bounds = FieldBounds {
        half_x: 300.0,
        half_z: 300.0,
        goal_half_width: 20.0,
        goal_height: 6.0,
    };
    let mut config = MatchConfig::new(bounds);
    config.kickoff_spawn = TEST_SPAWN;
    config.physics.gravity = 0.0;
    let mut game = Authority::new(config).unwrap();
    game.register(0x10, 0, 0).unwrap();
    game.kickoff().unwrap();
    assert_eq!(game.ball_position(), TEST_SPAWN);
    send(&mut game, 0x10, position(0x10, 1, [9.5, 0.0, 10.0])).unwrap();
    game.advance(0.02).unwrap();
    assert_eq!(game.possession(), Some(0x10));
    assert_eq!(game.player(0x10).unwrap().team, 0);
    assert!(game.player(0x10).unwrap().position.is_some());
    send(&mut game, 0x10, shot(0x10, 2, [0.0, 0.0, 1.0], 5.0)).unwrap();
    assert_eq!(game.last_touch(), Some(0x10));
    let elapsed = game.elapsed();
    let server = game.server_time();
    game.advance(0.1).unwrap();
    assert!(game.elapsed() > elapsed);
    assert!(game.server_time() > server);
}

// ---------------------------------------------------------------------------
// Promoted N1-N3 guards (RED probes in
// `lab/reports/soccer-match-authority-20261009T0830Z/red-probes.log`).
// ---------------------------------------------------------------------------

#[test]
fn equal_time_spam_cannot_accumulate_movement() {
    let mut game = authority();
    send(&mut game, 0x10, position(0x10, 1, pos(1.0, 0.0))).unwrap();
    let before = game.player(0x10).unwrap().position.unwrap();
    let mut sequence = 2;
    for step in 0..100 {
        let x = before[0] + (step as f32 + 1.0) * 0.00005;
        let _ = send(&mut game, 0x10, position(0x10, sequence, [x, 1.0, 0.0]));
        sequence += 1;
    }
    assert_eq!(
        game.player(0x10).unwrap().position.unwrap(),
        before,
        "equal-time spam accumulated movement"
    );
    // Even a tiny nonzero displacement at zero elapsed is refused, and the
    // refusal consumes no sequence.
    assert!(
        send(
            &mut game,
            0x10,
            position(0x10, sequence, [before[0] + 1e-6, 1.0, 0.0])
        )
        .is_err(),
        "1e-6 displacement at dt 0 accepted"
    );
    // After real time passes, the exact allowance still applies.
    game.advance(0.25).unwrap();
    send(&mut game, 0x10, position(0x10, sequence, pos(11.0, 0.0))).unwrap();
    assert_eq!(
        game.player(0x10).unwrap().position.unwrap(),
        [11.0, 1.0, 0.0]
    );
}

#[test]
fn forward_clock_spike_cannot_poison_the_clock() {
    let mut game = authority();
    game.apply(0x10, 0.0, position(0x10, 1, pos(1.0, 0.0)))
        .unwrap();
    game.advance(0.25).unwrap();
    let before = game.server_time();
    // A spiked timestamp is clamped: it is neither a teleport nor a clock jump.
    assert!(
        game.apply(0x10, 1000.0, position(0x10, 2, pos(30.0, 0.0)))
            .is_err(),
        "spike granted unlimited movement"
    );
    assert_eq!(game.server_time(), before, "spike moved the shared clock");
    // Real timestamps keep working for everyone afterwards.
    game.apply(0x10, before, position(0x10, 2, pos(11.0, 0.0)))
        .unwrap();
    game.apply(0x11, before, position(0x11, 1, pos(2.0, 2.0)))
        .unwrap();
}

#[test]
fn restore_normalises_the_clock_domain_and_preserves_allowances() {
    // Source process: one accepted position, then 0.1 s of movement allowance
    // (4.0 units) accrued before the snapshot.
    let mut source = authority();
    source
        .apply(0x10, 0.0, position(0x10, 1, pos(1.0, 0.0)))
        .unwrap();
    source.advance(0.1).unwrap();
    let snapshot = source.snapshot();

    // Reconnecting process: its monotonic base restarts at 0.0, a different
    // clock domain from the snapshot's 0.1.
    let mut reconnected = Authority::new(config()).unwrap();
    reconnected.restore(&snapshot).unwrap();
    assert_eq!(reconnected.server_time(), 0.0, "receiver clock kept");
    // The allowance survives the rebase exactly: 4.5 is refused, 4.0 fits.
    assert!(
        reconnected
            .apply(0x10, 0.0, position(0x10, 2, pos(5.5, 0.0)))
            .is_err(),
        "rebase inflated the allowance"
    );
    reconnected
        .apply(0x10, 0.0, position(0x10, 2, pos(5.0, 0.0)))
        .unwrap();
    assert_eq!(
        reconnected.player(0x10).unwrap().position.unwrap(),
        [5.0, 1.0, 0.0]
    );
    // Zero distance still fits afterwards: no domain lockout.
    assert!(
        reconnected
            .apply(0x10, 0.0, position(0x10, 3, pos(5.0, 0.0)))
            .is_ok(),
        "fresh clock domain locked out after restore"
    );
}
