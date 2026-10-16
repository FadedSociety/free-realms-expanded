//! Focused tests for the bounded Soccer codec and reconstructed reducer.
//!
//! Every fixture in this file is built explicitly from synthetic field values
//! through the production codec's own `encode`. Raw captured bytes are not
//! embedded (AGENTS: raw captures live in the lab, tests are synthetic); the
//! optional lab corpus gate is a byte-codec smoke check only and is not used as
//! client behavior evidence.

use osfr_tools::soccer::{
    self, Direction, Evidence, Match, Message, Phase, SoccerPacket, decode, encode,
};

fn close(actual: [f32; 3], expected: [f32; 3], tolerance: f32) -> bool {
    actual
        .iter()
        .zip(expected.iter())
        .all(|(a, e)| (a - e).abs() <= tolerance)
}

fn frame(packet: SoccerPacket) -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag: true,
        packet,
    })
}

fn frame_with_flag(flag: bool, packet: SoccerPacket) -> Vec<u8> {
    encode(&Message {
        outer: 5,
        flag,
        packet,
    })
}

fn vec_goal() -> Vec<u8> {
    frame(SoccerPacket::GoalMade { fields: [0, 1, 1] })
}

fn vec_register(player: u64) -> Vec<u8> {
    frame(SoccerPacket::RegisterPlayer {
        player,
        marker: [0, 1],
    })
}

fn vec_team(player: u64, team: u32, slot: u32) -> Vec<u8> {
    frame(SoccerPacket::SetPlayerTeam { player, team, slot })
}

fn vec_colours() -> Vec<u8> {
    frame(SoccerPacket::TeamColours {
        team_a_name: "synthetic-a".to_string(),
        team_b_name: "synthetic-b".to_string(),
        team_a_rgba: [255, 100, 35, 0],
        team_b_rgba: [15, 50, 255, 0],
        team_a_id: 1001,
        team_b_id: 1002,
        tail: [0],
    })
}

fn vec_time(remaining: f32, elapsed: f32) -> Vec<u8> {
    frame(SoccerPacket::UpdateGameTime { remaining, elapsed })
}

fn identity_ball(position: [f32; 3], velocity: [f32; 3]) -> Vec<u8> {
    let mut transform = [0.0f32; 16];
    transform[0] = 1.0;
    transform[5] = 1.0;
    transform[10] = 1.0;
    transform[12] = position[0];
    transform[13] = position[1];
    transform[14] = position[2];
    transform[15] = 1.0;
    frame(SoccerPacket::UpdateSoccerBall {
        transform,
        velocity,
        tail: [0; 15],
    })
}

fn vec_config() -> Vec<u8> {
    let asset = "synthetic-soccer-ball.adr";
    let spawn = [3.0f32, 1.0, 4.0, 1.0];
    let mut raw = vec![0u8; 221];
    raw[124..128].copy_from_slice(&(asset.len() as u32).to_le_bytes());
    raw[128..128 + asset.len()].copy_from_slice(asset.as_bytes());
    for (index, value) in spawn.iter().enumerate() {
        raw[157 + index * 4..161 + index * 4].copy_from_slice(&value.to_le_bytes());
    }
    frame(SoccerPacket::SetClientConfig {
        raw,
        ball_asset: asset.to_string(),
        ball_spawn: spawn,
    })
}

fn vec_player_pos() -> Vec<u8> {
    frame_with_flag(
        false,
        SoccerPacket::UpdatePlayerPos {
            player: 0x0000_0010_0080_06b2,
            position: [7.25, 0.0, 9.5],
            tail: [0; 45],
        },
    )
}

fn vec_player_state() -> Vec<u8> {
    frame(SoccerPacket::UpdatePlayerState {
        player: 0x0000_0010_0080_06b2,
        state: 2,
        aux: 0,
        state_counter: 1,
        vectors: [
            [0.0, 1.0, 0.0, 0.0],
            [0.8, 0.0, 0.6, 0.0],
            [7.25, 0.0, 9.5, 1.0],
        ],
    })
}

fn vec_controls() -> Vec<u8> {
    frame(SoccerPacket::UpdatePlayerControls {
        player: 0x0000_0010_0080_06b2,
        direction: [0.0, -1.0],
        controls: 0,
    })
}

fn all_vectors() -> Vec<Vec<u8>> {
    let mut vectors = vec![
        vec_goal(),
        vec_colours(),
        vec_time(0.0, 0.5),
        vec_time(120.0, 1.0),
        vec_time(110.0, 5.0),
        identity_ball([3.0, 1.0, 4.0], [0.0, 0.0, 0.0]),
        identity_ball([3.2, 1.8, 3.8], [0.4, 2.5, -0.2]),
        vec_config(),
        vec_player_pos(),
        vec_player_state(),
        vec_controls(),
    ];
    for player in [0x0000_0010_0080_06b2, 0x0000_0070_0080_06b2] {
        vectors.push(vec_register(player));
    }
    for (player, team, slot) in roster() {
        vectors.push(vec_team(player, team, slot));
    }
    vectors
}

fn roster() -> [(u64, u32, u32); 8] {
    [
        (0x4b52_cc6a_9573_ca91u64, 0u32, 0u32),
        (0x0000_0010_0080_06b2, 0, 1),
        (0x0000_0020_0080_06b2, 0, 2),
        (0x0000_0030_0080_06b2, 0, 3),
        (0x0000_0040_0080_06b2, 1, 0),
        (0x0000_0050_0080_06b2, 1, 1),
        (0x0000_0060_0080_06b2, 1, 2),
        (0x0000_0070_0080_06b2, 1, 3),
    ]
}

fn match_session() -> Match {
    let mut game = Match::new();
    game.handle(Direction::ServerToClient, &vec_colours())
        .unwrap();
    for (player, team, slot) in roster() {
        game.handle(Direction::ServerToClient, &vec_team(player, team, slot))
            .unwrap();
    }
    for index in 0..=6u64 {
        game.handle(
            Direction::ServerToClient,
            &vec_register(0x0000_0010_0080_06b2u64 + index * 0x10_0000_0000),
        )
        .unwrap();
    }
    game
}

#[test]
fn session_registration_builds_four_v_four_roster() {
    let game = match_session();
    let session = game.session();
    assert_eq!(session.teams[0].name, "synthetic-a");
    assert_eq!(session.teams[1].name, "synthetic-b");
    assert_eq!(session.teams[0].colour_rgba, [255, 100, 35, 0]);
    assert_eq!(session.teams[1].colour_rgba, [15, 50, 255, 0]);
    assert_eq!(session.roster.len(), 8);
    assert_eq!(session.roster_of_team(0).count(), 4);
    assert_eq!(session.roster_of_team(1).count(), 4);
    assert_eq!(game.player_team(0x4b52_cc6a_9573_ca91), Some((0, 0)));
    assert_eq!(game.player_team(0x0000_0030_0080_06b2), Some((0, 3)));
    assert_eq!(game.player_team(0x0000_0040_0080_06b2), Some((1, 0)));
    assert_eq!(game.player_team(0x0000_0070_0080_06b2), Some((1, 3)));
}

#[test]
fn kickoff_starts_half_clock() {
    let mut game = match_session();
    let pre = decode(&vec_time(0.0, 0.5)).unwrap();
    match pre.packet {
        SoccerPacket::UpdateGameTime { remaining, elapsed } => {
            assert_eq!(remaining, 0.0);
            assert!((elapsed - 0.5).abs() < 0.001);
        }
        other => panic!("unexpected {other:?}"),
    }
    game.handle(Direction::ServerToClient, &vec_time(0.0, 0.5))
        .unwrap();
    assert_eq!(game.phase(), Phase::Lobby);

    game.handle(Direction::ServerToClient, &vec_time(120.0, 1.0))
        .unwrap();
    assert_eq!(game.phase(), Phase::Running);
    assert_eq!(game.clock().remaining, 120.0);
    assert!((game.clock().elapsed - 1.0).abs() < 0.001);
    assert_eq!(game.clock().duration, Evidence::Reconstructed);
}

#[test]
fn goal_sets_scoreboard_holds_clock_and_restarts() {
    let mut game = match_session();
    game.handle(Direction::ServerToClient, &vec_time(0.0, 0.5))
        .unwrap();
    game.handle(Direction::ServerToClient, &vec_time(120.0, 1.0))
        .unwrap();
    game.handle(Direction::ServerToClient, &vec_goal()).unwrap();
    assert_eq!(game.phase(), Phase::Restart);
    assert_eq!(game.score(), [0, 1]);
    assert_eq!(game.goals().len(), 1);
    assert_eq!(game.goals()[0].scoring_team, 1);
    assert_eq!(game.goals()[0].source, Evidence::Reconstructed);

    game.handle(Direction::ServerToClient, &vec_time(110.0, 5.0))
        .unwrap();
    assert!((game.clock().remaining - 110.0).abs() < 0.001);
    assert_eq!(game.phase(), Phase::Restart);

    let replies = game.restart();
    assert_eq!(game.phase(), Phase::Running);
    let resumed = decode(&replies[0]).unwrap();
    match resumed.packet {
        SoccerPacket::UpdateGameTime { remaining, elapsed } => {
            assert!((remaining - 110.0).abs() < 0.001);
            assert!((elapsed - 5.0).abs() < 0.001);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(game.restart_source(), Evidence::Reconstructed);
}

#[test]
fn client_direction_cannot_change_authoritative_state() {
    let mut game = match_session();
    game.handle(Direction::ServerToClient, &vec_time(120.0, 1.0))
        .unwrap();
    let before = (
        game.phase(),
        game.score(),
        game.clock(),
        game.possession(),
        game.ball_position(),
    );

    for vector in [vec_player_pos(), vec_controls(), vec_player_state()] {
        game.handle(Direction::ClientToServer, &vector).unwrap();
    }
    assert_eq!(
        (
            game.phase(),
            game.score(),
            game.clock(),
            game.possession(),
            game.ball_position(),
        ),
        before
    );

    assert!(
        game.handle(Direction::ClientToServer, &vec_goal()).is_err(),
        "a client goal is refused outright"
    );
    assert_eq!(game.score(), [0, 0]);
    game.handle(Direction::ServerToClient, &vec_goal()).unwrap();
    assert_eq!(game.score(), [0, 1]);
}

#[test]
fn possession_chain_tracks_acquire_and_free_ball() {
    let mut game = match_session();
    let holders: [(usize, Option<u64>); 11] = [
        (0, Some(0x0000_0040_0080_06b2)),
        (1, None),
        (2, Some(0x0000_0030_0080_06b2)),
        (3, None),
        (4, Some(0x0000_0020_0080_06b2)),
        (5, None),
        (6, Some(0x0000_0040_0080_06b2)),
        (7, None),
        (8, Some(0x0000_0060_0080_06b2)),
        (9, None),
        (10, None),
    ];
    let mut last = 0u64;
    let mut last_free = Vec::new();
    for (index, step) in holders.iter().enumerate() {
        let previous = last;
        let bytes = match step.1 {
            Some(holder) => {
                last = holder;
                frame(SoccerPacket::AcquireBall {
                    flag: 0,
                    holder,
                    previous,
                })
            }
            None => {
                let position = if index >= 9 {
                    [5.5, 1.25, 6.0]
                } else {
                    [2.0 + index as f32, 1.0, 3.0]
                };
                let bytes = frame(SoccerPacket::FreeBall {
                    flag: 1,
                    position,
                    scale: 1.0,
                    previous,
                });
                last_free = bytes.clone();
                bytes
            }
        };
        game.handle(Direction::ServerToClient, &bytes).unwrap();
        assert_eq!(game.possession(), step.1, "step {index}");
    }
    let last = decode(&last_free).unwrap();
    match last.packet {
        SoccerPacket::FreeBall { position, .. } => {
            assert!(close(position, [5.5, 1.25, 6.0], 0.01));
            assert!(close(game.ball_position().unwrap(), [5.5, 1.25, 6.0], 0.01));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn ball_transform_controls_and_player_packets_decode() {
    let spawn = decode(&identity_ball([3.0, 1.0, 4.0], [0.0, 0.0, 0.0])).unwrap();
    match spawn.packet {
        SoccerPacket::UpdateSoccerBall {
            transform,
            velocity,
            ..
        } => {
            assert_eq!(transform[0], 1.0);
            assert_eq!(transform[5], 1.0);
            assert_eq!(transform[10], 1.0);
            assert!(close(
                [transform[12], transform[13], transform[14]],
                [3.0, 1.0, 4.0],
                0.001
            ));
            assert_eq!(velocity, [0.0, 0.0, 0.0]);
        }
        other => panic!("unexpected {other:?}"),
    }

    let mut moving = [0.0f32; 16];
    moving[0] = 1.0;
    moving[5] = 1.0;
    moving[10] = 1.0;
    moving[13] = 0.5;
    let moving = decode(&frame(SoccerPacket::UpdateSoccerBall {
        transform: moving,
        velocity: [1.2, 2.5, -0.4],
        tail: [0; 15],
    }))
    .unwrap();
    match moving.packet {
        SoccerPacket::UpdateSoccerBall {
            transform,
            velocity,
            ..
        } => {
            assert!((transform[13] - 0.5).abs() < 0.001, "synthetic y value");
            assert!((velocity[1] - 2.5).abs() < 0.01);
        }
        other => panic!("unexpected {other:?}"),
    }

    let controls = decode(&vec_controls()).unwrap();
    assert_eq!(
        controls.packet,
        SoccerPacket::UpdatePlayerControls {
            player: 0x0000_0010_0080_06b2,
            direction: [0.0, -1.0],
            controls: 0
        }
    );
    let position = decode(&vec_player_pos()).unwrap();
    assert!(!position.flag, "synthetic position keeps envelope flag 0");
    match position.packet {
        SoccerPacket::UpdatePlayerPos {
            player, position, ..
        } => {
            assert_eq!(player, 0x0000_0010_0080_06b2);
            assert!(close(position, [7.25, 0.0, 9.5], 0.001));
        }
        other => panic!("unexpected {other:?}"),
    }
    let state = decode(&vec_player_state()).unwrap();
    match state.packet {
        SoccerPacket::UpdatePlayerState {
            player, vectors, ..
        } => {
            assert_eq!(player, 0x0000_0010_0080_06b2);
            assert!(close(
                [vectors[2][0], vectors[2][1], vectors[2][2]],
                [7.25, 0.0, 9.5],
                0.001
            ));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn config_exposes_ball_asset_and_spawn() {
    let config = decode(&vec_config()).unwrap();
    match config.packet {
        SoccerPacket::SetClientConfig {
            ball_asset,
            ball_spawn,
            ..
        } => {
            assert_eq!(ball_asset, "synthetic-soccer-ball.adr");
            assert!((ball_spawn[0] - 3.0).abs() < 0.001);
            assert_eq!(ball_spawn[1], 1.0);
            assert!((ball_spawn[2] - 4.0).abs() < 0.001);
            assert_eq!(ball_spawn[3], 1.0);
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn synthetic_envelopes_round_trip_byte_exact() {
    for bytes in all_vectors() {
        let message = decode(&bytes).unwrap();
        assert_eq!(encode(&message), bytes);
    }
}

#[test]
fn unknown_soccer_opcode_round_trips() {
    let mut bytes = Vec::new();
    bytes.extend(5u16.to_le_bytes());
    bytes.push(1);
    bytes.extend(12u32.to_le_bytes());
    bytes.extend(76u16.to_le_bytes());
    bytes.extend(17u16.to_le_bytes());
    bytes.extend([1u8, 2, 3, 4, 5, 6, 7, 8]);
    let message = decode(&bytes).unwrap();
    assert_eq!(
        message.packet,
        SoccerPacket::Unknown {
            sub: 17,
            payload: vec![1, 2, 3, 4, 5, 6, 7, 8]
        }
    );
    assert_eq!(encode(&message), bytes);
}

#[test]
fn authority_waits_for_messages_before_moving_the_ball() {
    let mut game = match_session();
    game.handle(
        Direction::ServerToClient,
        &identity_ball([3.0, 1.0, 4.0], [0.0, 0.0, 0.0]),
    )
    .unwrap();
    let before = game.ball_position();
    assert!(close(before.unwrap(), [3.0, 1.0, 4.0], 0.001));
    game.handle(Direction::ServerToClient, &vec_time(120.0, 1.0))
        .unwrap();
    game.handle(Direction::ClientToServer, &vec_controls())
        .unwrap();
    assert_eq!(game.ball_position(), before, "no invented ball physics");
}

#[test]
fn timeout_finishes_match_and_proposal_is_labeled() {
    let mut game = match_session();
    game.handle(Direction::ServerToClient, &vec_time(120.0, 1.0))
        .unwrap();
    game.handle(Direction::ServerToClient, &vec_goal()).unwrap();
    game.restart();
    let timeout = encode(&soccer::Message {
        outer: 5,
        flag: true,
        packet: SoccerPacket::UpdateGameTime {
            remaining: 0.0,
            elapsed: 300.0,
        },
    });
    game.handle(Direction::ServerToClient, &timeout).unwrap();
    assert_eq!(game.phase(), Phase::Finished);

    let proposal = game.reward_proposal();
    assert_eq!(proposal.minigame_type_id, 12);
    assert_eq!(proposal.zone, "sg_soccer");
    assert_eq!(proposal.score, [0, 1]);
    assert_eq!(proposal.winner, Some(1));
    assert_eq!(proposal.reward_table, Evidence::Missing);
}

/// Exact captured-corpus coverage stays in the lab: point this gate at
/// `lab/reports/original-minigame-packets` to round-trip **every** preserved
/// opcode-76 frame byte-exactly. The per-sub counts are pinned so a corpus
/// change fails loudly instead of silently shrinking coverage; frames whose
/// sub id is still opaque would still be asserted byte-exact and counted as
/// opaque (expected: none).
#[test]
#[ignore = "private lab corpus: set OSFR_SOCCER_CAPTURE_DIR to lab/reports/original-minigame-packets"]
fn captured_corpus_round_trips_byte_exact() {
    let dir = std::env::var("OSFR_SOCCER_CAPTURE_DIR").expect("OSFR_SOCCER_CAPTURE_DIR");
    let expected: &[(u16, usize)] = &[
        (1, 1191),
        (2, 1),
        (3, 8),
        (4, 83),
        (5, 17),
        (6, 8),
        (7, 1),
        (8, 1),
        (9, 8),
        (10, 2),
        (11, 5),
        (12, 5),
        (14, 250),
        (15, 4),
        (16, 1216),
        (18, 1),
        (19, 10),
        (21, 8),
        (23, 78),
        (25, 20),
    ];
    let mut counts = std::collections::BTreeMap::new();
    let mut total = 0usize;
    let mut opaque = 0usize;
    for entry in std::fs::read_dir(&dir).expect("capture dir") {
        let path = entry.expect("entry").path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if !name.contains("-op76-") {
            continue;
        }
        let bytes = std::fs::read(&path).expect("frame bytes");
        let message = decode(&bytes).expect("capture decodes");
        assert_eq!(encode(&message), bytes, "round trip {name}");
        let sub = u16::from_le_bytes(bytes[9..11].try_into().unwrap());
        *counts.entry(sub).or_insert(0usize) += 1;
        total += 1;
        if matches!(message.packet, SoccerPacket::Unknown { .. }) {
            opaque += 1;
        }
    }
    let observed: Vec<(u16, usize)> = counts.into_iter().collect();
    assert_eq!(observed, expected, "opcode-76 per-sub corpus counts");
    assert_eq!(total, 2917, "opcode-76 corpus total");
    assert_eq!(opaque, 0, "no corpus frame should still decode as Unknown");
}
