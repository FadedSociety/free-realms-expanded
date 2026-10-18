//! Focused tests for the bounded chess (original minigame type 6) session model.
//!
//! The module is not yet registered in `src/lib.rs` (lead-owned). Including the exact source file
//! keeps this target compiling against the real artifact. Integration step for the lead: add
//! `pub mod restoration;` plus the module registration and drop this `#[path]` include.
//!
//! Client frame and engine-byte expectations follow `notes/Build Guides/Gameplay/Chess.md`,
//! especially “Coordinates and conversion,” “Network,” and “Key functions.”

#[path = "../src/restoration/piece_008.rs"]
#[allow(dead_code)] // The full codec is included while this test target exercises a subset.
mod piece_008;

use piece_008::{
    CHESS_DESCRIPTION_ID, CHESS_DIFFICULTY_CODES, CHESS_GAME_TYPE, CHESS_ICON_ID,
    CHESS_MOVIE_GAME_ID, CHESS_NAME_ID, CHESS_ROW_IDS, CHESS_TUTORIAL_ID, ChessError, ChessMove,
    ChessSession, ChessSessions, EVENT_CASTLE, EVENT_CHECK, EVENT_CHECKMATE, EVENT_DRAW,
    EVENT_ENPASSANT, EVENT_INIT, EVENT_LOCAL_AI, EVENT_MOVE, EVENT_MOVE_DATA, EVENT_MOVE_INVALID,
    EVENT_MOVE_REQUEST, EVENT_PLAYERLEFT, EVENT_PROMOTE, EVENT_STALEMATE, EVENT_TURN_END,
    EVENT_TURN_START, EVENT_WAITING, Kind, PIECE_PAWN, PLAYER_CODES, Side, Status, Winner,
    event_name,
};

/// Rank offset 0 is rank 1; convert to the client engine's a8 = 0 frame.
fn sq(file: u8, rank_from_one: u8) -> u8 {
    (7 - rank_from_one) * 8 + file
}

fn mv(from: u8, to: u8) -> ChessMove {
    ChessMove {
        from,
        to,
        promote: None,
    }
}

fn promote(from: u8, to: u8, kind: Kind) -> ChessMove {
    ChessMove {
        from,
        to,
        promote: Some(kind),
    }
}

fn start_session() -> ChessSession {
    ChessSession::launch(20, Some(Side::Black))
}

#[test]
fn launch_contract_matches_the_recovered_client_rows() {
    assert_eq!(CHESS_GAME_TYPE, 6);
    assert_eq!(CHESS_MOVIE_GAME_ID, 20);
    assert_eq!(CHESS_NAME_ID, 950);
    assert_eq!(CHESS_DESCRIPTION_ID, 951);
    assert_eq!(CHESS_ICON_ID, 917);
    assert_eq!(
        CHESS_ROW_IDS,
        [20, 65, 66, 67, 543, 544, 545, 546, 547, 548, 549, 550]
    );
    assert_eq!(CHESS_DIFFICULTY_CODES, [1, 2, 3, 5]);
    assert_eq!(CHESS_TUTORIAL_ID, 10);
}

#[test]
fn board_coordinates_match_the_clients_a8_origin() {
    assert_eq!(sq(0, 7), 0, "a8");
    assert_eq!(sq(7, 7), 7, "h8");
    assert_eq!(sq(4, 7), 4, "e8");
    assert_eq!(sq(0, 0), 56, "a1");
    assert_eq!(sq(4, 0), 60, "e1");
    assert_eq!(sq(7, 0), 63, "h1");

    let start = piece_008::Position::start();
    assert_eq!(
        start.to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
    assert_eq!(
        start.piece_at(0),
        Some(piece_008::Piece {
            kind: Kind::Rook,
            side: Side::Black
        })
    );
    assert_eq!(
        start.piece_at(60),
        Some(piece_008::Piece {
            kind: Kind::King,
            side: Side::White
        })
    );
    assert_eq!(
        start.piece_at(4),
        Some(piece_008::Piece {
            kind: Kind::King,
            side: Side::Black
        })
    );
}

#[test]
fn network_event_and_piece_codes_match_the_shipped_movie() {
    assert_eq!(EVENT_INIT, 0);
    assert_eq!(EVENT_TURN_START, 1);
    assert_eq!(EVENT_TURN_END, 2);
    assert_eq!(EVENT_MOVE, 10);
    assert_eq!(EVENT_ENPASSANT, 13);
    assert_eq!(EVENT_PROMOTE, 12);
    assert_eq!(EVENT_CHECK, 14);
    assert_eq!(EVENT_CHECKMATE, 15);
    assert_eq!(EVENT_DRAW, 16);
    assert_eq!(EVENT_STALEMATE, 17);
    assert_eq!(EVENT_MOVE_INVALID, 20);
    assert_eq!(EVENT_MOVE_REQUEST, 30);
    assert_eq!(EVENT_MOVE_DATA, 31);
    assert_eq!(EVENT_PLAYERLEFT, 40);
    assert_eq!(EVENT_WAITING, 41);
    assert_eq!(EVENT_LOCAL_AI, 42);

    assert_eq!(event_name(EVENT_INIT), Some("Init"));
    assert_eq!(event_name(EVENT_MOVE), Some("Move"));
    assert_eq!(event_name(EVENT_CASTLE), Some("Castle"));
    assert_eq!(event_name(EVENT_PROMOTE), Some("Promote"));
    assert_eq!(event_name(EVENT_MOVE_INVALID), Some("Invalid"));
    assert_eq!(event_name(99), None);

    assert_eq!(PIECE_PAWN, 1);
    assert_eq!(Kind::Pawn.code(), 1);
    assert_eq!(Kind::Rook.code(), 2);
    assert_eq!(Kind::Knight.code(), 3);
    assert_eq!(Kind::Bishop.code(), 4);
    assert_eq!(Kind::Queen.code(), 5);
    assert_eq!(Kind::King.code(), 6);
    assert_eq!(
        PLAYER_CODES,
        [
            (Kind::Pawn, 'P'),
            (Kind::Rook, 'R'),
            (Kind::Knight, 'N'),
            (Kind::Bishop, 'B'),
            (Kind::Queen, 'Q'),
            (Kind::King, 'K')
        ]
    );
}

#[test]
fn status_and_winner_codes_match_the_recovered_native_switch() {
    // `Chess::get_winner` switches on the status word: {2,4} -> 1, {3,5} -> 0, else 2.
    assert_eq!(Status::InProgress.code(), 0);
    assert_eq!(Status::Draw.code(), 1);
    assert_eq!(Status::WhiteMated.code(), 2);
    assert_eq!(Status::BlackMated.code(), 3);
    assert_eq!(Status::WhiteResigned.code(), 4);
    assert_eq!(Status::BlackResigned.code(), 5);
    assert_eq!(Status::from_code(2), Some(Status::WhiteMated));
    assert_eq!(Status::from_code(6), None);

    assert_eq!(Status::WhiteMated.winner(), Winner::Black);
    assert_eq!(Status::BlackMated.winner(), Winner::White);
    assert_eq!(Status::WhiteResigned.winner(), Winner::Black);
    assert_eq!(Status::BlackResigned.winner(), Winner::White);
    assert_eq!(Status::Draw.winner(), Winner::None);
    assert_eq!(Status::InProgress.winner(), Winner::None);
    assert_eq!(Winner::White.code(), 0);
    assert_eq!(Winner::Black.code(), 1);
    assert_eq!(Winner::None.code(), 2);
    assert!(Status::WhiteResigned.is_resign());
    assert!(!Status::Draw.is_resign());
}

#[test]
fn opening_position_has_the_twenty_legal_pawn_and_knight_moves() {
    let position = piece_008::Position::start();
    let mut moves = position.legal_moves();
    assert_eq!(moves.len(), 20);

    let e2 = sq(4, 1);
    let e4 = sq(4, 3);
    let e3 = sq(4, 2);
    let e5 = sq(4, 4);
    assert!(position.is_legal(&mv(e2, e4)));
    assert!(position.is_legal(&mv(e2, e3)));
    assert!(!position.is_legal(&mv(e2, e5)));
    assert!(
        !position.is_legal(&mv(sq(4, 6), sq(4, 4))),
        "black to move is not White"
    );

    moves.sort_by_key(|m| (m.from, m.to));
    assert!(
        moves.contains(&mv(sq(6, 0), sq(5, 2))),
        "Ng1-f3 must be legal"
    );
    assert_eq!(position.side_to_move(), Side::White);
}

#[test]
fn positions_reject_moves_that_leave_the_king_in_check() {
    // White king e1, black rook d2, black king e8. Kxe2 would step onto the rook's file.
    let mut position = piece_008::Position::from_fen("4k3/8/8/8/8/8/3r4/4K3 w - - 0 1").unwrap();
    let e1 = sq(4, 0);
    let e2 = sq(4, 1);
    let d2 = sq(3, 1);
    assert!(!position.in_check(Side::White));
    assert!(!position.is_legal(&mv(e1, e2)));
    assert!(
        position.is_legal(&mv(e1, d2)),
        "capturing the rook is legal"
    );

    let outcome = position.apply(&mv(e1, d2)).unwrap();
    assert_eq!(outcome.captured, Some(Kind::Rook));
    // Only the two kings are left, so the capture ends in the insufficient-material draw.
    assert_eq!(outcome.status, Status::Draw);
}

#[test]
fn castling_requires_rights_and_an_attack_free_path() {
    // Black rook f8 covers f1: kingside castling through f1 is illegal, queenside is not.
    let position = piece_008::Position::from_fen("5r2/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    let e1 = sq(4, 0);
    let g1 = sq(6, 0);
    let c1 = sq(2, 0);
    assert!(
        !position.is_legal(&mv(e1, g1)),
        "king may not cross f1 while attacked"
    );
    assert!(position.is_legal(&mv(e1, c1)));

    let open = piece_008::Position::from_fen("5k2/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(open.is_legal(&mv(e1, g1)));
    assert!(open.is_legal(&mv(e1, c1)));

    let no_right = piece_008::Position::from_fen("5k2/8/8/8/8/8/8/R3K2R w - - 0 1").unwrap();
    assert!(!no_right.is_legal(&mv(e1, g1)));
}

#[test]
fn castling_rights_follow_a8_origin_corner_indices() {
    let mut rook_moved = piece_008::Position::from_fen("5k2/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    rook_moved.apply(&mv(sq(0, 0), sq(0, 1))).unwrap(); // a1-a2
    assert_eq!(rook_moved.to_fen().split_whitespace().nth(2), Some("K"));

    let mut rook_captured =
        piece_008::Position::from_fen("r3k2r/1B6/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    rook_captured.apply(&mv(sq(1, 6), sq(0, 7))).unwrap(); // Bxa8
    assert_eq!(
        rook_captured.to_fen().split_whitespace().nth(2),
        Some("KQk")
    );
}

#[test]
fn en_passant_and_promotion_are_generated_and_encoded() {
    let mut position = piece_008::Position::from_fen("4k3/3p4/8/4P3/8/8/8/4K3 b - - 0 1").unwrap();
    let d7 = sq(3, 6);
    let d5 = sq(3, 4);
    position.apply(&mv(d7, d5)).unwrap();

    let e5 = sq(4, 4);
    let d6 = sq(3, 5);
    let ep = mv(e5, d6);
    assert!(position.is_legal(&ep), "en passant capture must be legal");
    assert_eq!(position.special_code(&ep), 0x64);
    let outcome = position.apply(&ep).unwrap();
    assert_eq!(outcome.captured, Some(Kind::Pawn));
    assert!(position.piece_at(d5).is_none(), "captured pawn leaves d5");

    let mut promo = piece_008::Position::from_fen("8/P6k/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let a7 = sq(0, 6);
    let a8 = sq(0, 7);
    let queen = promote(a7, a8, Kind::Queen);
    assert!(promo.is_legal(&queen));
    assert_eq!(promo.special_code(&queen), 0x40);
    promo.apply(&queen).unwrap();
    assert_eq!(
        promo.piece_at(a8),
        Some(piece_008::Piece {
            kind: Kind::Queen,
            side: Side::White
        })
    );
}

#[test]
fn special_codes_follow_the_recovered_native_move_encoder() {
    // `Chess::convert_player_move`: double push -> |0x80, occupied target -> |0x60, pawn
    // diagonal onto an empty square -> 0x64. `Chess::convert_player_promote` -> |0x40.
    // Castling flags 1/2/8/0x10 map to king targets 58/62/2/6; event-11 engine bytes are
    // `[0, 0x3e, 0, flag]` per the Chess build guide.
    let start = piece_008::Position::start();
    assert_eq!(start.special_code(&mv(sq(4, 1), sq(4, 3))), 0x80);
    assert_eq!(start.special_code(&mv(sq(6, 0), sq(5, 2))), 0);

    let capture = piece_008::Position::from_fen("4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1").unwrap();
    assert_eq!(capture.special_code(&mv(sq(4, 3), sq(3, 4))), 0x60);

    let open = piece_008::Position::from_fen("5k2/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert_eq!(
        open.special_code(&mv(sq(4, 0), sq(6, 0))),
        0x10,
        "white O-O"
    );
    assert_eq!(
        open.special_code(&mv(sq(4, 0), sq(2, 0))),
        0x08,
        "white O-O-O"
    );

    let black = piece_008::Position::from_fen("r3k2r/8/8/8/8/8/8/5K2 b kq - 0 1").unwrap();
    assert_eq!(
        black.special_code(&mv(sq(4, 7), sq(6, 7))),
        0x02,
        "black O-O"
    );
    assert_eq!(
        black.special_code(&mv(sq(4, 7), sq(2, 7))),
        0x01,
        "black O-O-O"
    );
}

#[test]
fn session_move_bytes_follow_the_client_four_byte_layout() {
    let mut session = ChessSession::launch(20, None);
    session.play(&mv(sq(4, 1), sq(4, 3))).unwrap();
    assert_eq!(session.moves(), &[[52, 36, 0, 0x80]]); // e2-e4, double-push bit

    for step in [
        mv(sq(4, 6), sq(4, 4)), // e7-e5
        mv(sq(6, 0), sq(5, 2)), // Ng1-f3
        mv(sq(1, 7), sq(2, 5)), // Nb8-c6
        mv(sq(5, 0), sq(4, 1)), // Bf1-e2
        mv(sq(6, 7), sq(5, 5)), // Ng8-f6
        mv(sq(4, 0), sq(6, 0)), // white O-O
    ] {
        session.play(&step).unwrap();
    }
    assert_eq!(session.moves().last(), Some(&[0, 0x3e, 0, 0x10]));
}

#[test]
fn checkmate_and_stalemate_are_detected_with_recovered_status_codes() {
    // Fool's mate: 1. f3 e5 2. g4 Qh4#.
    let mut session = start_session();
    for step in [
        mv(sq(5, 1), sq(5, 2)),
        mv(sq(4, 6), sq(4, 4)),
        mv(sq(6, 1), sq(6, 3)),
        mv(sq(3, 7), sq(7, 3)),
    ] {
        session.play(&step).unwrap();
    }
    assert!(session.position().in_check(Side::White));
    assert_eq!(session.status(), Status::WhiteMated);
    assert_eq!(session.winner(), Winner::Black);
    assert_eq!(session.status().code(), 2);

    // Classic stalemate: black king h8, white queen f7, white king g6, black to move.
    let stalemate = piece_008::Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
    assert!(!stalemate.in_check(Side::Black));
    assert!(stalemate.legal_moves().is_empty());
    assert_eq!(stalemate.status(), Status::Draw);

    // Insufficient material: bare kings, and king + bishop versus king.
    let bare = piece_008::Position::from_fen("8/8/8/4k3/8/8/8/4K3 w - - 0 1").unwrap();
    assert_eq!(bare.status(), Status::Draw);
    let bishop = piece_008::Position::from_fen("8/8/8/4k3/8/8/8/3BK3 w - - 0 1").unwrap();
    assert_eq!(bishop.status(), Status::Draw);
    let rook = piece_008::Position::from_fen("8/8/8/4k3/8/8/8/3RK3 w - - 0 1").unwrap();
    assert_eq!(rook.status(), Status::InProgress);
}

#[test]
fn threefold_repetition_is_a_draw() {
    let mut session = start_session();
    // Nf3 Nf6 Ng1 Ng8 Nf3 Nf6 Ng1 Ng8 -> the start position occurs three times.
    for step in [
        mv(sq(6, 0), sq(5, 2)),
        mv(sq(6, 7), sq(5, 5)),
        mv(sq(5, 2), sq(6, 0)),
        mv(sq(5, 5), sq(6, 7)),
        mv(sq(6, 0), sq(5, 2)),
        mv(sq(6, 7), sq(5, 5)),
        mv(sq(5, 2), sq(6, 0)),
        mv(sq(5, 5), sq(6, 7)),
    ] {
        session.play(&step).unwrap();
    }
    assert_eq!(session.status(), Status::Draw);
    assert_eq!(session.winner(), Winner::None);
}

#[test]
fn sessions_alternate_turns_and_lock_after_the_result() {
    let mut session = ChessSession::launch(20, None);
    assert_eq!(session.id(), 20);
    assert!(!session.is_computer_move());
    assert_eq!(
        session.play(&mv(sq(4, 1), sq(4, 3))).unwrap(),
        Status::InProgress
    );
    assert_eq!(session.position().side_to_move(), Side::Black);
    assert_eq!(
        session.play(&mv(sq(4, 1), sq(4, 2))),
        Err(ChessError::IllegalMove),
        "White may not move twice"
    );
    assert_eq!(
        session.play(&mv(sq(4, 6), sq(4, 4))).unwrap(),
        Status::InProgress
    );
    assert_eq!(session.position().side_to_move(), Side::White);
    assert_eq!(session.moves().len(), 2);

    let mut computer = ChessSession::launch(21, Some(Side::Black));
    assert!(!computer.is_computer_move());
    computer.play(&mv(sq(4, 1), sq(4, 3))).unwrap();
    assert!(computer.is_computer_move(), "computer plays Black here");
}

#[test]
fn resign_records_the_recovered_status_and_close_blocks_moves() {
    let mut session = start_session();
    session.play(&mv(sq(4, 1), sq(4, 3))).unwrap();
    assert_eq!(session.resign(Side::White).unwrap(), Status::WhiteResigned);
    assert_eq!(session.winner(), Winner::Black);
    assert_eq!(
        session.play(&mv(sq(4, 6), sq(4, 4))),
        Err(ChessError::GameOver)
    );
    assert_eq!(session.resign(Side::Black), Err(ChessError::GameOver));
}

#[test]
fn reconnect_restores_the_live_session_and_rejects_unknown_ids() {
    let mut sessions = ChessSessions::new();
    sessions.launch(20, Some(Side::Black));
    {
        let session = sessions.get_mut(20).unwrap();
        session.play(&mv(sq(4, 1), sq(4, 3))).unwrap();
        session.play(&mv(sq(4, 6), sq(4, 4))).unwrap();
    }
    let fen = sessions.get(20).unwrap().position().to_fen();

    let snapshot = sessions.reconnect(20).unwrap();
    assert_eq!(snapshot.id, 20);
    assert_eq!(snapshot.fen, fen);
    assert_eq!(snapshot.status_code, Status::InProgress.code());
    assert_eq!(snapshot.moves.len(), 2);
    assert_eq!(snapshot.side_to_move, Side::White.code());
    assert_eq!(
        sessions.reconnect(404),
        Err(ChessError::UnknownSession(404))
    );
}

#[test]
fn a_finished_session_stays_finished_across_reconnect_and_player_left() {
    let mut sessions = ChessSessions::new();
    sessions.launch(20, None);
    {
        let session = sessions.get_mut(20).unwrap();
        for step in [
            mv(sq(5, 1), sq(5, 2)),
            mv(sq(4, 6), sq(4, 4)),
            mv(sq(6, 1), sq(6, 3)),
            mv(sq(3, 7), sq(7, 3)),
        ] {
            session.play(&step).unwrap();
        }
    }
    let finished = sessions.reconnect(20).unwrap();
    assert_eq!(finished.status_code, Status::WhiteMated.code());
    assert_eq!(finished.winner_code, Winner::Black.code());
    assert_eq!(finished.moves.len(), 4);
    assert_eq!(
        sessions.get_mut(20).unwrap().play(&mv(sq(0, 1), sq(0, 2))),
        Err(ChessError::GameOver)
    );

    let mut left = ChessSessions::new();
    left.launch(30, None);
    left.get_mut(30)
        .unwrap()
        .play(&mv(sq(4, 1), sq(4, 3)))
        .unwrap();
    left.get_mut(30).unwrap().player_left(Side::Black);
    let snapshot = left.reconnect(30).unwrap();
    assert_eq!(snapshot.left_side, Some(Side::Black));
    assert_eq!(snapshot.winner_code, Winner::None.code());
    assert_eq!(
        left.get_mut(30).unwrap().play(&mv(sq(4, 6), sq(4, 4))),
        Err(ChessError::GameOver),
        "a departure closes the table without inventing a winner"
    );
}
