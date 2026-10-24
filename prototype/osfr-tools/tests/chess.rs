use chess::{ChessMove, ChessParticipant, ChessTables, Kind, Mode, Side, Status, payload_packet};
use inventory::Inventory;
use minigame::{MinigameRuns, RunMode, RunSpec, RunState};
use osfr_tools::chess_service::{ChessContent, ChessRewardLedger, ChessService};
use osfr_tools::{
    chess, chess_service, inventory, minigame, profiles, rewards, station_cash::EarnSource,
};
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

#[path = "support/chess_bot.rs"]
mod chess_bot;
use chess_bot::{EndOfGame, play_line, terminal};

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
        sc: 0,
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
    let mut tables = ChessTables::entropy().unwrap();
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
    let mut tables = ChessTables::entropy().unwrap();
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
fn single_player_server_replies_without_client_move_authority() {
    let mut tables = ChessTables::entropy().unwrap();
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
    assert!(
        !human_events
            .iter()
            .any(|event| event.starts_with("OnChessEventMsg\t30"))
    );
    assert!(human_events.iter().any(|event| {
        [
            "OnChessEventMsg\t10",
            "OnChessEventMsg\t11",
            "OnChessEventMsg\t12",
            "OnChessEventMsg\t13",
        ]
        .iter()
        .any(|prefix| event.starts_with(prefix))
    }));
    assert!(human_events.contains(&"OnChessEventMsg\t1".into()));

    let before = tables.table(7).unwrap().position().to_fen();
    let rejected = send(&mut tables, 7, 71, 701, "OnMoveComputerMsg\t12\t28\t0\t128");
    assert!(!rejected.accepted && rejected.messages.is_empty());
    assert_eq!(tables.table(7).unwrap().position().to_fen(), before);
    assert_eq!(
        tables.table(7).unwrap().position().side_to_move(),
        Side::White
    );
}

#[test]
fn server_opens_for_white_when_the_human_plays_black() {
    let mut tables = ChessTables::entropy().unwrap();
    tables
        .open_single(
            8,
            67,
            ChessParticipant {
                player_id: 81,
                instance_id: 801,
            },
            Side::Black,
        )
        .unwrap();
    send(&mut tables, 8, 81, 801, "OnConnectMsg");
    let started = send(&mut tables, 8, 81, 801, "OnStartGameMsg\t0");
    let events = text(&started.messages, 81);
    assert!(
        !events
            .iter()
            .any(|event| event.starts_with("OnChessEventMsg\t30"))
    );
    assert!(events.iter().any(|event| {
        [
            "OnChessEventMsg\t10",
            "OnChessEventMsg\t11",
            "OnChessEventMsg\t12",
            "OnChessEventMsg\t13",
        ]
        .iter()
        .any(|prefix| event.starts_with(prefix))
    }));
    assert!(events.contains(&"OnChessEventMsg\t1".into()));
    assert_eq!(
        tables.table(8).unwrap().position().side_to_move(),
        Side::Black
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
    assert_eq!(
        tables.table(40).unwrap().position().plies(),
        4,
        "fool's mate is four plies"
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
    assert_eq!(
        tables.table(61).unwrap().position().plies(),
        0,
        "an instant resign is a game of zero plies"
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
    assert_eq!(repetition.plies(), 8, "two four-ply cycles");
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
    assert!(ChessTables::entropy().unwrap().table(999).is_none());
}

#[test]
fn content_skill_keeps_the_server_reply_path() {
    let mut tables = ChessTables::entropy().unwrap();
    tables
        .open_single_with_skill(
            9,
            20,
            ChessParticipant {
                player_id: 41,
                instance_id: 901,
            },
            Side::White,
            77,
            chess::DEFAULT_NEAR_EQUAL_MARGIN,
        )
        .unwrap();
    send(&mut tables, 9, 41, 901, "OnConnectMsg");
    send(&mut tables, 9, 41, 901, "OnStartGameMsg\t0");
    let moved = send(
        &mut tables,
        9,
        41,
        901,
        &format!("OnMoveMsg\t{}\t{}", sq(4, 1), sq(4, 3)),
    );
    let events = text(&moved.messages, 41);
    assert!(
        !events
            .iter()
            .any(|event| event.starts_with("OnChessEventMsg\t30"))
    );
    assert!(events.iter().any(|event| {
        [
            "OnChessEventMsg\t10",
            "OnChessEventMsg\t11",
            "OnChessEventMsg\t12",
            "OnChessEventMsg\t13",
        ]
        .iter()
        .any(|prefix| event.starts_with(prefix))
    }));
    assert!(events.contains(&"OnChessEventMsg\t1".into()));

    // The default opener keeps the row's own skill.
    let mut defaults = ChessTables::entropy().unwrap();
    defaults
        .open_single(
            10,
            20,
            ChessParticipant {
                player_id: 42,
                instance_id: 902,
            },
            Side::White,
        )
        .unwrap();
    send(&mut defaults, 10, 42, 902, "OnConnectMsg");
    send(&mut defaults, 10, 42, 902, "OnStartGameMsg\t0");
    let default_move = send(
        &mut defaults,
        10,
        42,
        902,
        &format!("OnMoveMsg\t{}\t{}", sq(4, 1), sq(4, 3)),
    );
    let events = text(&default_move.messages, 42);
    assert!(
        !events
            .iter()
            .any(|event| event.starts_with("OnChessEventMsg\t30"))
    );
    assert!(events.contains(&"OnChessEventMsg\t1".into()));
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

const SINGLE_ROW: u32 = 20;
const CHESS_PLAYER: u64 = 71;
/// Test clock day: proves the ledger follows the wall clock, not `now_ms`.
const TODAY: u64 = 20_000;

fn service_content(daily_reward_coins: u32) -> ChessContent {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/chess.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    value["daily_reward_coins"] = json!(daily_reward_coins);
    value["rows"] = json!([
        {"row_id": SINGLE_ROW, "difficulty": 1, "skill": 20, "success_coins": 10}
    ]);
    serde_json::from_value(value).unwrap()
}

fn service_texts(outcome: &chess_service::ChessActivityOutcome) -> Vec<String> {
    outcome
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[39, 0, 14]))
        .map(|packet| {
            let length = i32::from_le_bytes(packet[15..19].try_into().unwrap()) as usize;
            String::from_utf8(packet[19..19 + length - 1].to_vec()).unwrap()
        })
        .collect()
}

fn offer_instance(outcome: &chess_service::ChessActivityOutcome) -> i32 {
    let offer = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .expect("39/16 offer");
    i32::from_le_bytes(offer[3..7].try_into().unwrap())
}

fn route(
    service: &mut ChessService,
    instance: i32,
    text: &str,
    now_ms: u64,
    rewards: &RewardState,
) -> chess_service::ChessActivityOutcome {
    service
        .route(
            CHESS_PLAYER,
            &payload_packet(instance, text).unwrap(),
            now_ms,
            rewards,
        )
        .unwrap()
        .expect("chess packet")
}

/// Begin, start and connect one offered chess run.
fn open_service_game(service: &mut ChessService, rewards: &RewardState, now_ms: u64) -> i32 {
    let offered = service.begin(SINGLE_ROW, CHESS_PLAYER).unwrap();
    let instance = offer_instance(&offered);
    service
        .route(CHESS_PLAYER, &request(5, instance), now_ms + 1, rewards)
        .unwrap()
        .expect("start packet");
    route(service, instance, "OnConnectMsg", now_ms + 2, rewards);
    route(service, instance, "OnStartGameMsg\t0", now_ms + 3, rewards);
    instance
}

/// Plays the greedy human to the end through the shared bot. The returned
/// texts are the human's own move messages; the outcome is the client's
/// `FRServer_GameOver` acknowledgement.
fn play_to_end(
    service: &mut ChessService,
    rewards: &RewardState,
    now_ms: u64,
) -> (EndOfGame, chess_service::ChessActivityOutcome, Vec<String>) {
    let instance = open_service_game(service, rewards, now_ms);
    let mut now = now_ms;
    let (end, line) = play_line(|text| {
        now += 10;
        let outcome = route(service, instance, text, now, rewards);
        service_texts(&outcome)
    });
    let ack = if end == EndOfGame::Stopped {
        chess_service::ChessActivityOutcome::default()
    } else {
        route(service, instance, "FRServer_GameOver", now + 5, rewards)
    };
    service
        .route(CHESS_PLAYER, &[26, 0, 42, 0], now + 6, rewards)
        .unwrap();
    (end, ack, line)
}

/// Replays recorded human move messages against a fresh service. `None` means
/// the line diverged (a replayed move was illegal in the new game).
fn replay(
    service: &mut ChessService,
    rewards: &RewardState,
    line: &[String],
    now_ms: u64,
) -> Option<EndOfGame> {
    let instance = open_service_game(service, rewards, now_ms);
    for (ply, text) in line.iter().enumerate() {
        let outcome = route(
            service,
            instance,
            text,
            now_ms + 10 + ply as u64 * 10,
            rewards,
        );
        let texts = service_texts(&outcome);
        if texts.iter().any(|event| event == "OnChessEventMsg\t20") {
            return None;
        }
        if let Some(end) = terminal(&texts) {
            if end != EndOfGame::Stopped {
                route(
                    service,
                    instance,
                    "FRServer_GameOver",
                    now_ms + 20 + ply as u64 * 10,
                    rewards,
                );
            }
            service
                .route(
                    CHESS_PLAYER,
                    &[26, 0, 42, 0],
                    now_ms + 21 + ply as u64 * 10,
                    rewards,
                )
                .unwrap();
            return Some(end);
        }
    }
    None
}

#[test]
fn computer_replies_vary_between_fresh_tables() {
    let mut tables = ChessTables::entropy().unwrap();
    let mut replies = BTreeSet::new();
    for table in 1..=64u64 {
        let player = 1_000 + table;
        let instance = 100 + table as i32;
        tables
            .open_single(
                table,
                SINGLE_ROW,
                ChessParticipant {
                    player_id: player,
                    instance_id: instance,
                },
                Side::White,
            )
            .unwrap();
        send(&mut tables, table, player, instance, "OnConnectMsg");
        send(&mut tables, table, player, instance, "OnStartGameMsg\t0");
        let moved = send(
            &mut tables,
            table,
            player,
            instance,
            &format!("OnMoveMsg\t{}\t{}", sq(4, 1), sq(4, 3)),
        );
        let reply = text(&moved.messages, player)
            .into_iter()
            .find(|event| {
                event.strip_prefix("OnChessEventMsg\t").is_some_and(|rest| {
                    matches!(rest.split('\t').next(), Some("10" | "11" | "12" | "13"))
                })
            })
            .expect("the computer replied");
        replies.insert(reply);
    }
    assert!(
        replies.len() > 1,
        "the computer repeated one reply in every game: {replies:?}"
    );
}

/// The production row's first computer reply for one seeded table.
fn seeded_opening_reply(row: u32, seed: u64) -> String {
    let mut tables = ChessTables::with_seed(seed);
    let (table, player, instance) = (1, 71, 901);
    tables
        .open_single(
            table,
            row,
            ChessParticipant {
                player_id: player,
                instance_id: instance,
            },
            Side::White,
        )
        .unwrap();
    send(&mut tables, table, player, instance, "OnConnectMsg");
    send(&mut tables, table, player, instance, "OnStartGameMsg\t0");
    let moved = send(
        &mut tables,
        table,
        player,
        instance,
        &format!("OnMoveMsg\t{}\t{}", sq(4, 1), sq(4, 3)),
    );
    text(&moved.messages, player)
        .into_iter()
        .find(|event| {
            event.strip_prefix("OnChessEventMsg\t").is_some_and(|rest| {
                matches!(rest.split('\t').next(), Some("10" | "11" | "12" | "13"))
            })
        })
        .expect("the computer replied")
}

#[test]
fn hard_rows_diverge_across_seeded_tables() {
    // Production rows 66 (skill 60) and 67 (skill 100). Fixed seeds keep this
    // deterministic; the easy row is covered by the fresh-table test above.
    for row in [66u32, 67] {
        let replies: BTreeSet<String> = (1..=16u64)
            .map(|seed| seeded_opening_reply(row, seed))
            .collect();
        assert!(
            replies.len() > 1,
            "row {row} repeated one computer reply across seeded tables: {replies:?}"
        );
    }
}

#[test]
fn a_zero_near_equal_window_is_refused() {
    // A zero window always takes the single best move, which made the computer
    // reply a function of the position and therefore replayable.
    let mut content = service_content(1_000);
    content.near_equal_margin = 0;
    assert!(content.validate().is_err(), "a zero window was accepted");

    let mut tables = ChessTables::with_seed(1);
    assert!(
        tables
            .open_single_with_skill(
                1,
                SINGLE_ROW,
                ChessParticipant {
                    player_id: 71,
                    instance_id: 901,
                },
                Side::White,
                20,
                0,
            )
            .is_err(),
        "a zero window must not open a table"
    );
    assert!(
        tables.table(1).is_none(),
        "a refused table must not be inserted"
    );
}

#[test]
fn a_recorded_winning_game_does_not_replay_against_fresh_tables() {
    let fixture = RewardFixture::new(CHESS_PLAYER);
    let mut original = ChessService::with_seed(service_content(1_000), 22).unwrap();
    let (end, ack, line) = play_to_end(&mut original, &fixture.state(), 1_000);
    assert_eq!(end, EndOfGame::Won, "the seeded game must be a win");
    assert!(ack.reward_state.is_some(), "the seeded win pays once");
    assert!(line.len() > 2, "the recorded line is a real game");

    // The same seed reproduces the same game, so the recorded line is a win there.
    let mut same = ChessService::with_seed(service_content(1_000), 22).unwrap();
    assert_eq!(
        replay(&mut same, &fixture.state(), &line, 2_000),
        Some(EndOfGame::Won)
    );

    // Other seeded tables answer differently, so the line stops being a win.
    // Fixed seeds keep the assertion deterministic instead of a dice roll.
    let mut outcomes = Vec::new();
    for seed in 1..=8u64 {
        let mut service = ChessService::with_seed(service_content(1_000), seed).unwrap();
        outcomes.push(replay(&mut service, &fixture.state(), &line, 10_000));
    }
    assert!(
        outcomes.iter().any(|end| *end != Some(EndOfGame::Won)),
        "the recorded line still won every fresh game: {outcomes:?}"
    );

    // The production OS-entropy constructor is still exercised: a table seeded
    // by `with_state` must not reproduce the recorded line either.
    let mut entropy = ChessService::with_state(service_content(1_000), None).unwrap();
    assert_ne!(
        replay(&mut entropy, &fixture.state(), &line, 20_000),
        Some(EndOfGame::Won),
        "an OS-seeded table replayed the recorded winning line"
    );
}

#[test]
fn the_daily_cap_stops_payouts_survives_restore_and_refusals_change_nothing() {
    let fixture = RewardFixture::new(CHESS_PLAYER);
    let content = service_content(10);
    let mut service = ChessService::with_seed(content.clone(), 22).unwrap();
    service.set_day(TODAY);
    // Coins earned before the game stay, and the payout adds to them.
    let mut live = fixture.state();
    live.inventory.grant_coins(7).unwrap();
    assert_eq!(live.inventory.state().coins, 7);
    let (end, ack, _) = play_to_end(&mut service, &live, 1_000);
    assert_eq!(end, EndOfGame::Won);
    assert_eq!(
        ack.reward_state.as_ref().unwrap().inventory.state().coins,
        17
    );
    let ledger = service.ledger().expect("the paid win is recorded");
    assert_eq!(
        ledger,
        ChessRewardLedger {
            day: TODAY,
            paid_coins: 10
        },
        "the ledger day is the clock day, not now_ms"
    );

    // Reconnect: the same game against the saved ledger pays nothing, but the
    // result still settles and nothing about the ledger changes.
    let mut restored = ChessService::with_seed(content.clone(), 22).unwrap();
    restored.set_day(TODAY);
    restored.restore_ledger(Some(ledger)).unwrap();
    let (end, capped, _) = play_to_end(&mut restored, &fixture.state(), 50_000);
    assert_eq!(end, EndOfGame::Won);
    assert!(capped.reward_state.is_none(), "a capped win must not pay");
    assert!(capped.reward_packets.is_empty());
    assert!(
        capped
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[39, 0, 18])),
        "the capped result still settles through the lifecycle"
    );
    assert_eq!(
        restored.ledger(),
        Some(ledger),
        "the cap changed the ledger"
    );

    // A refusal changes nothing: an unknown row starts nothing.
    assert!(!restored.is_active());
    assert!(restored.begin(999, CHESS_PLAYER).is_err());
    assert_eq!(restored.ledger(), Some(ledger));
    // A second Play while a run is open is refused and leaves the run alive.
    let instance = open_service_game(&mut restored, &fixture.state(), 60_000);
    let before = restored.ledger();
    assert!(restored.begin(SINGLE_ROW, CHESS_PLAYER).is_err());
    assert_eq!(restored.ledger(), before);
    assert!(restored.is_active());
    restored
        .route(
            CHESS_PLAYER,
            &request(6, instance),
            60_100,
            &fixture.state(),
        )
        .unwrap();
    assert!(!restored.is_active());

    // The next UTC day starts a fresh allowance for the restored ledger.
    let mut next_day = ChessService::with_seed(content.clone(), 22).unwrap();
    next_day.set_day(TODAY + 1);
    next_day.restore_ledger(Some(ledger)).unwrap();
    let (end, paid_again, _) = play_to_end(&mut next_day, &fixture.state(), 2_000);
    assert_eq!(end, EndOfGame::Won);
    assert!(paid_again.reward_state.is_some(), "a new day pays again");
    assert_eq!(
        next_day.ledger(),
        Some(ChessRewardLedger {
            day: TODAY + 1,
            paid_coins: 10
        })
    );

    // Restoring a ledger beyond the content bound is refused, not clamped.
    let mut corrupt = ChessService::with_seed(content, 22).unwrap();
    assert!(
        corrupt
            .restore_ledger(Some(ChessRewardLedger {
                day: 0,
                paid_coins: chess_service::MAX_DAILY_REWARD_COINS + 1,
            }))
            .is_err()
    );
    assert_eq!(corrupt.ledger(), None);
}

/// A settled game hands the earn table what it judges by: whether the player won or drew, and
/// how many plies were played (the earned Station Cash gate lives in `EarnTable`).
#[test]
fn a_settled_game_reports_its_result_and_length_for_the_earn_table() {
    let fixture = RewardFixture::new(CHESS_PLAYER);
    let mut service = ChessService::with_seed(service_content(1_000), 22).unwrap();
    let (end, ack, line) = play_to_end(&mut service, &fixture.state(), 1_000);
    assert_eq!(end, EndOfGame::Won, "the seeded game must be a win");
    let Some(EarnSource::ChessGame { won_or_drew, plies }) = ack.completed else {
        panic!("the acknowledged win settles: {:?}", ack.completed);
    };
    assert!(won_or_drew);
    assert!(
        plies >= line.len() as u32,
        "every human move is a ply ({plies} plies, {} moves)",
        line.len()
    );
}
