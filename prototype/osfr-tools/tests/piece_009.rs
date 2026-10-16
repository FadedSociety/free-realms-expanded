//! Checkers (queue item 009) behavioral regression for
//! `src/restoration/piece_009.rs`.
//!
//! The module is included by path so this file exercises the recovered rules
//! without touching `lib.rs`. Every position here is synthetic: no user save,
//! native client or gateway state is read or written.
//!
//! Expectations come from the original `game_checkers.gfx` movie
//! (`BoardProxy.as`, `GameServer.as`, `CheckersNetworkEvent.as`,
//! `GameClientNetwork.as`); provenance and limits are recorded in
//! `notes/Research/Pieces/009 Checkers.md`.
#[path = "../src/restoration/piece_009.rs"]
mod piece_009;

use piece_009::Side::{Black, Red};
use piece_009::{
    Action, AppliedMove, Board, CHECKERS_GAME_ID, CheckersEvent, CheckersSession, EV_BOARD,
    EV_INIT, EV_MOVE, EV_MOVE_INVALID, EV_MOVE_JUMPED, EV_MOVE_KING, EV_PLAYERLEFT, EV_TURN_END,
    EV_TURN_START, EV_VS_AI, EV_WAITING, Jump, MoveError, PIECES, PIECES_PER_SIDE, PayloadError,
    Phase, Piece, SPOTS, Side, decode_move_payload, encode_move_payload,
};

const HOST: i32 = 7_009_001;

fn pawn(side: Side) -> Piece {
    Piece { side, king: false }
}

fn king(side: Side) -> Piece {
    Piece { side, king: true }
}

fn position(pieces: &[(usize, Piece)]) -> Board {
    let mut board = Board::empty();
    for (spot, piece) in pieces {
        assert!(
            board.set(*spot, Some(*piece)),
            "fixture spot {spot} in range"
        );
    }
    board
}

fn session(board: Board, first: Side) -> CheckersSession {
    CheckersSession::from_board(HOST, board, first)
}

#[test]
fn recovered_constants_match_the_movie() {
    assert_eq!(CHECKERS_GAME_ID, 21);
    assert_eq!(SPOTS, 32);
    assert_eq!(PIECES_PER_SIDE, 12);
    assert_eq!(PIECES, 24);
    assert_eq!(
        [
            EV_INIT,
            EV_TURN_START,
            EV_TURN_END,
            EV_MOVE,
            EV_MOVE_JUMPED,
            EV_MOVE_KING,
            EV_MOVE_INVALID,
            EV_BOARD,
            EV_VS_AI,
            EV_PLAYERLEFT,
            EV_WAITING,
        ],
        [0, 1, 2, 10, 11, 12, 20, 30, 32, 40, 41]
    );
    // Recovered bootstrap: `GameServer.OnReady` sends `INIT(0, 1)` and then
    // `TURN_START(0)`, so player 0 (black) moves first and player 1 is red.
    assert_eq!(Side::Black.wire_index(), 0);
    assert_eq!(Side::Red.wire_index(), 1);
    assert_eq!(Side::plays_first(), Black);
    assert_eq!(Black.opponent(), Red);
}

#[test]
fn initial_layout_matches_recovered_setup() {
    let board = Board::initial();
    let mut red = 0;
    let mut black = 0;
    for spot in 0..SPOTS {
        match board.piece(spot) {
            Some(piece) => {
                assert!(!piece.king, "the movie resets every piece as a pawn");
                if piece.side == Red {
                    assert!(spot < 12, "red starts on spots 0..=11, saw {spot}");
                    red += 1;
                } else {
                    assert!(spot >= 20, "black starts on spots 20..=31, saw {spot}");
                    black += 1;
                }
            }
            None => assert!(
                (12..20).contains(&spot),
                "only the middle two rows start empty, saw {spot}"
            ),
        }
    }
    assert_eq!(red, PIECES_PER_SIDE);
    assert_eq!(black, PIECES_PER_SIDE);

    let fresh = CheckersSession::new(HOST);
    assert_eq!(fresh.instance_id(), HOST);
    assert_eq!(fresh.phase(), Phase::Playing);
    assert_eq!(fresh.current(), Black);
    assert!(!fresh.board().legal_actions(Black).is_empty());
    assert!(!fresh.is_finished());
}

#[test]
fn pawn_geometry_matches_movie_offsets() {
    // Exact `BoardProxy.GetSpotFromOffset` math: odd rows are shifted left by
    // one column slot, so dx 0 and 1 are the two real diagonals.
    assert_eq!(Board::spot_from_offset(0, 0, 1), Some(4));
    assert_eq!(Board::spot_from_offset(0, 1, 1), Some(5));
    assert_eq!(Board::spot_from_offset(0, 0, -1), None);
    assert_eq!(Board::spot_from_offset(0, 1, -1), None);
    assert_eq!(Board::spot_from_offset(4, 0, -1), None);
    assert_eq!(Board::spot_from_offset(4, 1, -1), Some(0));
    assert_eq!(Board::spot_from_offset(4, 0, 1), None);
    assert_eq!(Board::spot_from_offset(4, 1, 1), Some(8));
    assert_eq!(Board::spot_from_offset(31, 0, -1), Some(26));
    assert_eq!(Board::spot_from_offset(31, 1, -1), Some(27));
}

#[test]
fn pawns_move_forward_only_and_kings_keep_all_four_diagonals() {
    let board = position(&[(21, pawn(Black)), (10, pawn(Red))]);
    assert_eq!(
        board.steps(21),
        vec![16, 17],
        "black pawns walk up the board"
    );
    assert_eq!(
        board.steps(10),
        vec![14, 15],
        "red pawns walk down the board"
    );

    let board = position(&[(18, king(Black))]);
    let mut steps = board.steps(18);
    steps.sort_unstable();
    assert_eq!(steps, vec![14, 15, 22, 23]);
}

#[test]
fn capture_needs_an_enemy_target_and_an_empty_landing() {
    let same_colour = position(&[(21, pawn(Black)), (17, pawn(Black))]);
    assert!(
        same_colour.jumps(21).is_empty(),
        "a same-colour neighbour is not a jump target"
    );

    let enemy = position(&[(21, pawn(Black)), (17, pawn(Red))]);
    assert_eq!(enemy.jumps(21), vec![Jump { over: 17, land: 14 }]);

    let blocked = position(&[(21, pawn(Black)), (17, pawn(Red)), (14, pawn(Black))]);
    assert!(
        blocked.jumps(21).is_empty(),
        "the landing square must be empty"
    );
}

#[test]
fn compulsory_capture_forbids_quiet_moves() {
    let board = position(&[(21, pawn(Black)), (24, pawn(Black)), (17, pawn(Red))]);
    let fixture = board.clone();

    assert_eq!(
        board.legal_actions(Black),
        vec![Action {
            from: 21,
            path: vec![14]
        }],
        "only the compulsory capture is legal while a jump exists"
    );

    let mut session = session(board, Black);
    assert_eq!(
        session.apply(Black, 24, &[20]),
        Err(MoveError::CompulsoryCapture)
    );
    assert_eq!(session.board(), &fixture, "a rejected move changes nothing");
}

#[test]
fn jump_removes_the_enemy_and_emits_movie_events() {
    let board = position(&[(21, pawn(Black)), (17, pawn(Red)), (31, pawn(Black))]);

    let mut session = session(board, Black);
    let events = session.apply(Black, 21, &[14]).expect("legal capture");
    assert_eq!(
        events,
        vec![
            CheckersEvent::new(EV_MOVE, 21, 14),
            CheckersEvent::single(EV_MOVE_JUMPED, 17),
            CheckersEvent::single(EV_TURN_END, 0),
            CheckersEvent::single(EV_TURN_START, 1),
        ]
    );
    assert_eq!(session.board().piece(21), None);
    assert_eq!(session.board().piece(17), None);
    assert_eq!(session.board().piece(14), Some(pawn(Black)));
    assert_eq!(session.board().piece(31), Some(pawn(Black)));
    assert_eq!(session.current(), Red);
    assert_eq!(session.events(), events.as_slice());
}

#[test]
fn double_jump_chain_must_run_to_completion() {
    // Black pawn 22 jumps 17 and then must keep jumping 9 as one move.
    let board = position(&[
        (22, pawn(Black)),
        (17, pawn(Red)),
        (9, pawn(Red)),
        (31, pawn(Black)),
    ]);

    let mut session = session(board, Black);
    assert_eq!(
        session.apply(Black, 22, &[13]),
        Err(MoveError::CaptureContinuationRequired),
        "the movie locks the turn to the jumping piece until no jumps remain"
    );

    let events = session.apply(Black, 22, &[13, 6]).expect("complete chain");
    assert_eq!(
        events,
        vec![
            CheckersEvent::new(EV_MOVE, 22, 13),
            CheckersEvent::single(EV_MOVE_JUMPED, 17),
            CheckersEvent::new(EV_MOVE, 13, 6),
            CheckersEvent::single(EV_MOVE_JUMPED, 9),
            CheckersEvent::single(EV_TURN_END, 0),
            CheckersEvent::single(EV_TURN_START, 1),
        ]
    );
    assert_eq!(session.board().piece(6), Some(pawn(Black)));
    assert_eq!(session.board().piece(17), None);
    assert_eq!(session.board().piece(9), None);
    assert_eq!(session.current(), Red);
}

#[test]
fn promotion_emits_the_king_event_and_a_king_may_move_backwards() {
    let board = position(&[(20, pawn(Red)), (24, pawn(Black)), (31, pawn(Black))]);

    let mut session = session(board, Red);
    let events = session.apply(Red, 20, &[29]).expect("promoting capture");
    assert_eq!(
        events,
        vec![
            CheckersEvent::new(EV_MOVE, 20, 29),
            CheckersEvent::single(EV_MOVE_JUMPED, 24),
            CheckersEvent::single(EV_MOVE_KING, 29),
            CheckersEvent::single(EV_TURN_END, 1),
            CheckersEvent::single(EV_TURN_START, 0),
        ]
    );
    assert_eq!(session.board().piece(29), Some(king(Red)));

    let mut moves: Vec<Vec<usize>> = session
        .board()
        .legal_actions(Red)
        .into_iter()
        .filter(|action| action.from == 29)
        .map(|action| action.path)
        .collect();
    moves.sort();
    assert_eq!(
        moves,
        vec![vec![24], vec![25]],
        "the new king moves back up the board"
    );
}

#[test]
fn a_side_without_legal_moves_loses_and_the_session_closes() {
    // Black pawn 20 is blocked: 16 is occupied by red and the only jump
    // landing square, 13, is occupied too.
    let board = position(&[(20, pawn(Black)), (16, pawn(Red)), (13, pawn(Red))]);

    let mut session = session(board, Black);
    assert!(session.board().legal_actions(Black).is_empty());
    assert!(session.is_finished());
    assert_eq!(session.phase(), Phase::Finished);
    assert_eq!(session.winner(), Some(Red));
    assert_eq!(
        session.apply(Black, 20, &[24]),
        Err(MoveError::GameFinished),
        "a closed session refuses every later action"
    );
}

#[test]
fn a_king_stepping_back_onto_a_promotion_row_re_emits_the_king_event() {
    // `GameServer.IsKing(dest)` only checks the destination row, so the movie
    // re-sends MOVE_KING for an already-kinged piece.
    let board = position(&[(4, king(Red)), (31, pawn(Black))]);
    let mut session = session(board, Red);
    let events = session.apply(Red, 4, &[0]).expect("king step to row 0");
    assert_eq!(
        events,
        vec![
            CheckersEvent::new(EV_MOVE, 4, 0),
            CheckersEvent::single(EV_MOVE_KING, 0),
            CheckersEvent::single(EV_TURN_END, 1),
            CheckersEvent::single(EV_TURN_START, 0),
        ]
    );
    assert_eq!(session.board().piece(0), Some(king(Red)));
}

#[test]
fn turn_replay_and_bounds_are_enforced_without_state_change() {
    let mut session = CheckersSession::new(HOST);
    session.apply(Black, 21, &[16]).expect("black opening move");
    assert_eq!(
        session.applied(),
        &[AppliedMove {
            side: Black,
            from: 21,
            path: vec![16],
        }]
    );
    let after_first = session.clone();

    assert_eq!(session.apply(Black, 21, &[16]), Err(MoveError::Replayed));
    assert_eq!(&session, &after_first);

    assert_eq!(session.apply(Black, 22, &[17]), Err(MoveError::NotYourTurn));
    assert_eq!(&session, &after_first);

    assert_eq!(session.apply(Red, 99, &[12]), Err(MoveError::OutOfRange));
    assert_eq!(session.apply(Red, 12, &[]), Err(MoveError::EmptyChain));
    assert_eq!(&session, &after_first);
}

#[test]
fn late_replays_after_later_turns_are_still_rejected() {
    let mut session = CheckersSession::new(HOST);
    session.apply(Black, 21, &[16]).expect("black move");
    session.apply(Red, 9, &[13]).expect("red reply");

    assert_eq!(session.apply(Black, 21, &[16]), Err(MoveError::Replayed));
    assert_eq!(session.apply(Red, 9, &[13]), Err(MoveError::Replayed));
    assert_eq!(session.apply(Red, 10, &[14]), Err(MoveError::NotYourTurn));
}

#[test]
fn illegal_destinations_and_wrong_pieces_are_named() {
    let mut session = CheckersSession::new(HOST);
    assert_eq!(session.apply(Black, 12, &[16]), Err(MoveError::NoPiece));
    assert_eq!(
        session.apply(Black, 8, &[12]),
        Err(MoveError::WrongSide),
        "spot 8 holds a red pawn on black's turn"
    );
    assert_eq!(
        session.apply(Black, 21, &[12]),
        Err(MoveError::IllegalDestination),
        "12 is not a black pawn diagonal"
    );
    assert_eq!(session.apply(Black, 21, &[]), Err(MoveError::EmptyChain));
}

#[test]
fn move_payload_matches_the_movie_send_move_text() {
    assert_eq!(
        encode_move_payload(9, &[14, 18]).unwrap(),
        "OnMoveMsg\t9\t14,18"
    );
    assert_eq!(
        decode_move_payload("OnMoveMsg\t9\t14,18"),
        Ok((9, vec![14, 18]))
    );
    assert_eq!(decode_move_payload("OnMoveMsg\t0\t4"), Ok((0, vec![4])));
    assert_eq!(
        decode_move_payload("OnMoveMsg\t9\t"),
        Err(PayloadError::EmptyMoves)
    );
    assert_eq!(
        decode_move_payload("OnMoveMsg\t9\tx"),
        Err(PayloadError::MalformedNumber)
    );
    assert_eq!(
        decode_move_payload("OnMoveMsg\t32\t4"),
        Err(PayloadError::SpotOutOfRange)
    );
    assert_eq!(
        decode_move_payload("OnReadyMsg"),
        Err(PayloadError::WrongMessage)
    );
    assert_eq!(encode_move_payload(32, &[4]), Err(MoveError::OutOfRange));
}
