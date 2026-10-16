use chess::{ChessMove, ChessParticipant, ChessTables, Kind, Mode, Side, Status, payload_packet};
use inventory::Inventory;
use minigame::{MinigameRuns, RunMode, RunSpec, RunState};
use osfr_tools::{chess, inventory, minigame, profiles, rewards};
use profiles::ProfileState;
use rewards::{RewardDisplay, RewardGrant, RewardState};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn sq(file: u8, rank_from_zero: u8) -> u8 {
    (7 - rank_from_zero) * 8 + file
}

fn mv(from: u8, to: u8) -> ChessMove {
    ChessMove {
        from,
        to,
        promote: None,
    }
}

fn perft(position: &chess::Position, depth: u8) -> u64 {
    position.perft(depth)
}

fn request(sub: u8, instance_id: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for field in [instance_id, -1, -1] {
        packet.extend(field.to_le_bytes());
    }
    packet
}

fn start_run(instance_id: i32) -> MinigameRuns {
    let mut runs = MinigameRuns::new(instance_id).unwrap();
    runs.offer(RunSpec::empty(
        20,
        6,
        RunMode::Flash {
            movie: "game_chess.gfx".into(),
        },
    ))
    .unwrap();
    let start = runs.handle_client(&request(5, instance_id)).unwrap();
    assert_eq!(start.state, Some(RunState::Starting));
    assert_eq!(runs.started(instance_id).state, Some(RunState::Playing));
    runs
}

struct RewardFixture(PathBuf);

impl RewardFixture {
    fn new(player_id: u64) -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-chess-reward-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let coins = 0_i32.to_le_bytes();
        let empty = 0_i32.to_le_bytes();
        let span = |kind: &str, offset: usize, profile_id: i32, bytes: &[u8]| {
            json!({
                "kind": kind,
                "offset": offset,
                "length": bytes.len(),
                "profile_id": profile_id,
                "sha256": format!("{:x}", Sha256::digest(bytes))
            })
        };
        let manifest = json!({
            "schema_version": 1,
            "player_guid": player_id,
            "default_state": {"coins": 0, "items": [], "equipment": [], "next_instance_id": null},
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [
                span("coins", 13, 0, &coins),
                span("items", 17, 0, &empty),
                span("equipment", 21, 1, &empty)
            ],
            "definitions": []
        });
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        Self(path)
    }

    fn state(&self) -> RewardState {
        RewardState::new(
            Inventory::load(&self.0, None).unwrap(),
            ProfileState {
                version: 1,
                active: 1,
                unlocked: BTreeSet::from([1]),
                experience: BTreeMap::new(),
            },
            8,
        )
    }
}

impl Drop for RewardFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("inventory.json"));
        let _ = fs::remove_dir(&self.0);
    }
}

fn grant(player_id: u64) -> RewardGrant {
    RewardGrant {
        coins: 7,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        display: RewardDisplay {
            source_guid: 0,
            player_guid: player_id,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn text(messages: &[chess::ChessMessage], player: u64) -> Vec<String> {
    messages
        .iter()
        .filter(|message| message.player_id == player)
        .map(|message| message.text.clone())
        .collect()
}

fn send(
    tables: &mut ChessTables,
    table: u64,
    player: u64,
    instance: i32,
    message: &str,
) -> chess::ChessOutcome {
    tables
        .handle_payload(table, player, &payload_packet(instance, message).unwrap())
        .unwrap()
}

fn start_two_player(table_id: u64) -> (ChessTables, Vec<chess::ChessMessage>) {
    let mut tables = ChessTables::default();
    tables
        .open_two_player(
            table_id,
            65,
            ChessParticipant {
                player_id: 101,
                instance_id: 1001,
            },
            ChessParticipant {
                player_id: 202,
                instance_id: 1002,
            },
        )
        .unwrap();
    send(&mut tables, table_id, 101, 1001, "OnConnectMsg");
    send(&mut tables, table_id, 202, 1002, "OnConnectMsg");
    send(&mut tables, table_id, 101, 1001, "OnStartGameMsg\t0");
    let started = send(&mut tables, table_id, 202, 1002, "OnStartGameMsg\t0");
    (tables, started.messages)
}

#[test]
fn start_position_perft_depth_one_through_four() {
    let position = chess::Position::start();
    assert_eq!((perft(&position, 1), perft(&position, 2)), (20, 400));
    assert_eq!((perft(&position, 3), perft(&position, 4)), (8_902, 197_281));
}

#[test]
fn standard_castling_en_passant_and_promotion_positions_match_perft() {
    let kiwipete = chess::Position::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    )
    .unwrap();
    assert_eq!(
        (
            perft(&kiwipete, 1),
            perft(&kiwipete, 2),
            perft(&kiwipete, 3)
        ),
        (48, 2_039, 97_862)
    );

    let special =
        chess::Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
            .unwrap();
    assert_eq!(
        (perft(&special, 1), perft(&special, 2), perft(&special, 3)),
        (44, 1_486, 62_379)
    );
}

#[test]
fn illegal_move_is_rejected_and_does_not_change_the_board() {
    let mut tables = ChessTables::default();
    tables
        .open_single(
            3,
            20,
            ChessParticipant {
                player_id: 31,
                instance_id: 301,
            },
            Side::White,
        )
        .unwrap();
    send(&mut tables, 3, 31, 301, "OnConnectMsg");
    send(&mut tables, 3, 31, 301, "OnStartGameMsg\t0");
    let before = tables.table(3).unwrap().position().to_fen();
    let rejected = send(
        &mut tables,
        3,
        31,
        301,
        &format!("OnMoveMsg\t{}\t{}", sq(4, 1), sq(4, 4)),
    );
    assert!(!rejected.accepted);
    assert!(text(&rejected.messages, 31).contains(&"OnChessEventMsg\t20".into()));
    assert_eq!(tables.table(3).unwrap().position().to_fen(), before);

    let pinned = chess::Position::from_fen("4k3/8/8/8/8/8/3r4/4K3 w - - 0 1").unwrap();
    assert!(!pinned.is_legal(&mv(sq(4, 0), sq(4, 1))));
}

#[test]
fn discovered_queen_check_is_detected_and_black_must_answer_it() {
    let mut position = chess::Position::start();
    for movement in [
        mv(sq(4, 1), sq(4, 3)),
        mv(sq(5, 6), sq(5, 4)),
        mv(sq(3, 0), sq(7, 4)),
    ] {
        position.apply(&movement).unwrap();
    }
    assert!(position.in_check(Side::Black));
    assert!(position.legal_moves().iter().all(|m| {
        let mut next = position.clone();
        next.apply(m).unwrap();
        !next.in_check(Side::Black)
    }));
}

#[test]
fn single_player_validates_engine_proposals_and_requests_each_computer_turn() {
    let mut tables = ChessTables::default();
    tables
        .open_single(
            7,
            20,
            ChessParticipant {
                player_id: 71,
                instance_id: 701,
            },
            Side::White,
        )
        .unwrap();
    send(&mut tables, 7, 71, 701, "OnConnectMsg");
    let started = send(&mut tables, 7, 71, 701, "OnStartGameMsg\t0");
    assert!(text(&started.messages, 71).contains(&"OnChessEventMsg\t0\t1\t1".into()));
    assert!(text(&started.messages, 71).contains(&"OnChessEventMsg\t42\t1".into()));
    assert!(text(&started.messages, 71).contains(&"OnChessEventMsg\t1".into()));

    let human = send(
        &mut tables,
        7,
        71,
        701,
        &format!("OnMoveMsg\t{}\t{}", sq(4, 1), sq(4, 3)),
    );
    let human_events = text(&human.messages, 71);
    assert!(human_events.contains(&"OnChessEventMsg\t31\t52\t36\t0\t128".into()));
    assert!(human_events.contains(&"OnChessEventMsg\t2".into()));
    assert!(human_events.contains(&"OnChessEventMsg\t30\t1\t20".into()));
    assert!(!human_events.contains(&"OnChessEventMsg\t1".into()));

    let before = tables.table(7).unwrap().position().to_fen();
    let rejected = send(&mut tables, 7, 71, 701, "OnMoveComputerMsg\t12\t28\t0\t0");
    assert!(!rejected.accepted);
    assert_eq!(tables.table(7).unwrap().position().to_fen(), before);
    assert!(text(&rejected.messages, 71).contains(&"OnChessEventMsg\t30\t1\t20".into()));

    let computer = send(&mut tables, 7, 71, 701, "OnMoveComputerMsg\t12\t28\t0\t128");
    let events = text(&computer.messages, 71);
    assert!(computer.accepted);
    assert!(events.contains(&"OnChessEventMsg\t10\t12\t28".into()));
    assert!(events.contains(&"OnChessEventMsg\t2".into()));
    assert!(events.contains(&"OnChessEventMsg\t1".into()));
    assert_eq!(
        tables.table(7).unwrap().position().side_to_move(),
        Side::White
    );
}

#[test]
fn fools_mate_emits_checkmate_and_settles_one_reward_once() {
    let (mut tables, started) = start_two_player(40);
    assert!(text(&started, 101).contains(&"OnChessEventMsg\t0\t1\t1".into()));
    assert!(text(&started, 202).contains(&"OnChessEventMsg\t0\t1\t0".into()));
    let steps = [
        (101, "OnMoveMsg", sq(5, 1), sq(5, 2)),
        (202, "OnMoveMsg", sq(4, 6), sq(4, 4)),
        (101, "OnMoveMsg", sq(6, 1), sq(6, 3)),
        (202, "OnMoveMsg", sq(3, 7), sq(7, 3)),
    ];
    let mut last = chess::ChessOutcome::default();
    for (player, name, from, to) in steps {
        let instance = if player == 101 { 1001 } else { 1002 };
        last = send(
            &mut tables,
            40,
            player,
            instance,
            &format!("{name}\t{from}\t{to}"),
        );
    }
    assert!(last.accepted);
    assert_eq!(
        last.terminal.as_ref().unwrap().reason,
        chess::EndReason::Checkmate
    );
    assert_eq!(last.terminal.as_ref().unwrap().winner, Some(202));
    for player in [101, 202] {
        let messages = text(&last.messages, player);
        assert!(messages.iter().any(|m| m == "OnChessEventMsg\t14"));
        assert!(messages.iter().any(|m| m == "OnChessEventMsg\t15"));
        assert!(messages.iter().any(|m| m == "OnChessEventMsg\t2"));
        let ordered = messages
            .iter()
            .filter(|message| {
                [
                    "OnChessEventMsg\t14",
                    "OnChessEventMsg\t15",
                    "OnChessEventMsg\t2",
                ]
                .iter()
                .any(|prefix| message == prefix)
            })
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            ordered,
            [
                "OnChessEventMsg\t14",
                "OnChessEventMsg\t15",
                "OnChessEventMsg\t2"
            ]
        );
    }
    assert!(text(&last.messages, 202).contains(&"OnGameWonMsg".into()));
    assert!(text(&last.messages, 101).contains(&"OnGameLostMsg".into()));

    let winner_reward = RewardFixture::new(202).state();
    let payout = grant(202);
    let mut black_runs = start_run(1002);
    let premature = tables
        .settle_player(
            40,
            202,
            &mut black_runs,
            0,
            0,
            0,
            Some((&winner_reward, &payout)),
        )
        .unwrap();
    assert!(!premature.accepted);
    assert!(send(&mut tables, 40, 202, 1002, "FRServer_GameOver").accepted);
    assert!(!send(&mut tables, 40, 202, 1002, "FRServer_GameOver").accepted);
    let won = tables
        .settle_player(
            40,
            202,
            &mut black_runs,
            0,
            0,
            0,
            Some((&winner_reward, &payout)),
        )
        .unwrap();
    assert_eq!(won.state, Some(RunState::Won));
    assert_eq!(won.reward_packets.len(), 2);
    assert_eq!(won.reward_state.unwrap().inventory.state().coins, 7);
    let duplicate = tables
        .settle_player(
            40,
            202,
            &mut black_runs,
            0,
            0,
            0,
            Some((&winner_reward, &payout)),
        )
        .unwrap();
    assert!(!duplicate.accepted);
    assert!(duplicate.reward_packets.is_empty());
    assert_eq!(tables.table(40).unwrap().settled_count(), 1);

    let loser_reward = RewardFixture::new(101).state();
    let mut white_runs = start_run(1001);
    assert!(send(&mut tables, 40, 101, 1001, "FRServer_GameOver").accepted);
    let lost = tables
        .settle_player(
            40,
            101,
            &mut white_runs,
            0,
            0,
            0,
            Some((&loser_reward, &payout)),
        )
        .unwrap();
    assert_eq!(lost.state, Some(RunState::Lost));
    assert!(lost.reward_packets.is_empty());
}

#[test]
fn threefold_draw_stops_movie_and_settles_both_runs_once() {
    let (mut tables, _) = start_two_player(41);
    let cycle = [
        (101, 1001, sq(6, 0), sq(5, 2)),
        (202, 1002, sq(6, 7), sq(5, 5)),
        (101, 1001, sq(5, 2), sq(6, 0)),
        (202, 1002, sq(5, 5), sq(6, 7)),
    ];
    let mut terminal = chess::ChessOutcome::default();
    for _ in 0..2 {
        for (player, instance, from, to) in cycle {
            terminal = send(
                &mut tables,
                41,
                player,
                instance,
                &format!("OnMoveMsg\t{from}\t{to}"),
            );
        }
    }
    assert_eq!(terminal.terminal.unwrap().reason, chess::EndReason::Draw);
    assert_eq!(terminal.terminal.unwrap().winner, None);
    assert!(text(&terminal.messages, 101).contains(&"OnGameStoppedMsg".into()));
    assert!(text(&terminal.messages, 202).contains(&"OnGameStoppedMsg".into()));
    assert!(!text(&terminal.messages, 101).contains(&"OnGameWonMsg".into()));

    let mut white_run = start_run(1001);
    let mut black_run = start_run(1002);
    let white = tables
        .settle_player(41, 101, &mut white_run, 0, 0, 0, None)
        .unwrap();
    let black = tables
        .settle_player(41, 202, &mut black_run, 0, 0, 0, None)
        .unwrap();
    assert_eq!(white.state, Some(RunState::Lost));
    assert_eq!(black.state, Some(RunState::Lost));
    assert!(
        !tables
            .settle_player(41, 101, &mut white_run, 0, 0, 0, None)
            .unwrap()
            .accepted
    );
    assert_eq!(tables.table(41).unwrap().settled_count(), 2);
}

#[test]
fn two_player_move_is_relayed_only_to_the_other_player() {
    let (mut tables, _) = start_two_player(50);
    let result = send(
        &mut tables,
        50,
        101,
        1001,
        &format!("OnMoveMsg\t{}\t{}", sq(4, 1), sq(4, 3)),
    );
    assert!(result.accepted);
    assert!(
        text(&result.messages, 101)
            .iter()
            .any(|m| m == "OnChessEventMsg\t2")
    );
    assert!(
        text(&result.messages, 101)
            .iter()
            .any(|m| m == "OnChessEventMsg\t1")
    );
    let black = text(&result.messages, 202);
    assert!(
        black
            .iter()
            .any(|m| m == &format!("OnChessEventMsg\t10\t{}\t{}", sq(4, 1), sq(4, 3)))
    );
    assert!(black.iter().any(|m| m == "OnChessEventMsg\t2"));
    assert!(black.iter().any(|m| m == "OnChessEventMsg\t1"));
    assert!(
        !text(&result.messages, 101)
            .iter()
            .any(|m| m.starts_with("OnChessEventMsg\t10"))
    );
}

#[test]
fn disconnect_forfeits_and_notifies_survivor() {
    let (mut tables, _) = start_two_player(60);
    let lost = tables.disconnect(60, 101).unwrap();
    assert_eq!(lost.terminal.as_ref().unwrap().winner, Some(202));
    assert_eq!(
        lost.terminal.as_ref().unwrap().reason,
        chess::EndReason::Disconnect
    );
    assert!(text(&lost.messages, 202).contains(&"OnChessEventMsg\t40".into()));
    assert!(text(&lost.messages, 202).contains(&"OnGameWonMsg".into()));
    assert!(text(&lost.messages, 101).is_empty());

    let disconnected_state = RewardFixture::new(101).state();
    let disconnected_payout = grant(101);
    let mut disconnected_run = start_run(1001);
    let settled = tables
        .settle_player(
            60,
            101,
            &mut disconnected_run,
            0,
            0,
            0,
            Some((&disconnected_state, &disconnected_payout)),
        )
        .unwrap();
    assert_eq!(settled.state, Some(RunState::Lost));
    assert!(settled.reward_packets.is_empty());
}

#[test]
fn resignation_and_draw_rules_reach_terminal_states() {
    let (mut tables, _) = start_two_player(61);
    let resigned = tables.resign(61, 101).unwrap();
    assert_eq!(resigned.terminal.unwrap().winner, Some(202));
    assert_eq!(
        resigned.terminal.unwrap().reason,
        chess::EndReason::Resignation
    );

    let stalemate = chess::Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
    assert_eq!(stalemate.status(), Status::Draw);
    assert!(stalemate.is_stalemate());
    assert!(!stalemate.in_check(Side::Black));

    let fifty_move =
        chess::Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 100 51")
            .unwrap();
    assert_eq!(fifty_move.status(), Status::Draw);

    let mut repetition = chess::Position::start();
    let cycle = [
        mv(sq(6, 0), sq(5, 2)),
        mv(sq(6, 7), sq(5, 5)),
        mv(sq(5, 2), sq(6, 0)),
        mv(sq(5, 5), sq(6, 7)),
    ];
    for _ in 0..2 {
        for movement in cycle {
            repetition.apply(&movement).unwrap();
        }
    }
    assert_eq!(repetition.status(), Status::Draw);
}

#[test]
fn payload_header_and_difficulty_mapping_are_bounded_and_stable() {
    let packet = payload_packet(123, "OnChessEventMsg\t1").unwrap();
    assert_eq!(
        &packet[..15],
        &[39, 0, 14, 123, 0, 0, 0, 255, 255, 255, 255, 20, 0, 0, 0]
    );
    assert_eq!(*packet.last().unwrap(), 0);
    assert_eq!(chess::skill_for_row(20), Some(20));
    assert_eq!(chess::skill_for_row(66), Some(60));
    assert_eq!(chess::skill_for_row(67), Some(100));
    assert_eq!(chess::skill_for_row(65), None);
    assert!(ChessTables::default().table(999).is_none());
}

#[test]
fn chess_mode_and_move_encoding_cover_client_castling_events() {
    assert!(matches!(Mode::for_row(65).unwrap(), Mode::TwoPlayer));
    let mut position = chess::Position::from_fen("5k2/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    let castle = mv(sq(4, 0), sq(6, 0));
    assert!(position.is_legal(&castle));
    assert_eq!(
        position.engine_move_bytes(&castle).unwrap(),
        [0, 0x3e, 0, 0x02]
    );
    position.apply(&castle).unwrap();
    assert_eq!(position.piece_at(sq(5, 0)).unwrap().kind, Kind::Rook);
    assert_eq!(
        Status::Draw,
        chess::Position::from_fen("7k/8/6K1/8/8/8/8/8 w - - 0 1")
            .unwrap()
            .status()
    );
}
