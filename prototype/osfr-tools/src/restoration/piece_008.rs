//! Chess (original minigame type 6): the recovered launch/result contract plus a bounded
//! server-side session model with legal move generation, check/mate/stalemate detection and
//! reconnect.
//!
//! ORIGINAL, recovered (see this item's lab report `piece-008-20261009T024347Z/`):
//! - Launch rows: `MiniGameData.txt` ids 20, 65, 66, 67, 543-550 carry `TYPE 6` with
//!   `NAME_ID 950`, `DESCRIPTION_ID 951`, `ICON_ID 917` and difficulty codes 1/2/3/5;
//!   `MiniGameTypeData.txt` type 6 is native (IS_FLASH 0). The shipped movie
//!   `Minigames/Chess/game_chess.swf` (`ConfigServer.GAME_ID = 20`, `STORAGE_NAME = "chess_data"`)
//!   and `GameTutorialData.txt` row 10 (`../minigames/chess/tutorial_chess.swf`, name id 950)
//!   agree with the table rows.
//! - Host ABI: `ScriptsBase.lua` exposes `ChessInit(computer)`, `ChessMovePlayer(from,to,promote,
//!   special)` and `ChessMoveRequest(skill)` -> movie `OnChessMoveResult(from,to,promote,special)`;
//!   the native client implements them at `0x00c08010`, `0x00c080f0`, `0x00c081a0`.
//! - Event table and message shapes: `ChessNetworkEvent.as` (types 0/1/2/10/11/12/13/14/15/16/17/
//!   20/30/31/40/41/42 and piece codes 1..6), `GameClientNetwork.as` (client->server message
//!   names and argument order), `ChessMachine.as`, `WaitingForUser.as`, `WaitingForNetwork.as`,
//!   `GameState.as` (board index `y*8+x`, `63-spot` flip for the second player),
//!   `ConfigClient.END_GAME_DELAY = 3000`.
//! - Result codes: the native `Chess` status word and `Chess::get_winner` switch (status 1 draw,
//!   2/3 mate, 4/5 resign; winner 0/1, 2 none); `Chess::is_resign` = status 4 or 5;
//!   `Chess::mate`/`Chess::is_stalemate`/`Chess::is_draw` set those codes.
//! - The four-byte move the flash boundary passes: `[from, to, promote, special]`, encoded by
//!   `Chess::convert_player_move` / `convert_player_promote` / `convert_castle_to_king`
//!   (double push `|0x80`, occupied target `|0x60`, pawn diagonal to an empty square `0x64`,
//!   promotion `|0x40`, castles 1/2/8/0x10 -> king targets 58/62/2/6).
//! - Client square indices use a8 = 0, h1 = 63; white's king starts at 60 and black's at 4.
//!   Event-11 castling is encoded `[0, 0x3e, 0, flag]`; see the current Chess build guide.
//!
//! RECONSTRUCTED (not historical): this module is a server-side session model. The emulator
//! server decompilation contains no chess handler, and no chess capture exists in the lab, so
//! turn relay, session identity, reconnect and resignation-by-side are our policy shaped to the
//! recovered client contract, not recovered server behavior.
//!
//! GAPS: the exact SOE wire frames that carried the chess messages, the original server's
//! authority model, the `game_chess.swf` dialogue that invokes `ChessInterface.resign`, and any
//! reward/rating amounts. Nothing here grants currency, items or XP.

use std::collections::HashMap;

/// Original minigame type (`MiniGameStatusHandler.CHESS_TYPE` and the shipped table's `TYPE`).
pub const CHESS_GAME_TYPE: u8 = 6;
/// Original game id the shipped chess movie reports (`ConfigServer.GAME_ID`).
pub const CHESS_MOVIE_GAME_ID: u32 = 20;
pub const CHESS_NAME_ID: u32 = 950;
pub const CHESS_DESCRIPTION_ID: u32 = 951;
pub const CHESS_ICON_ID: u32 = 917;
/// Every shipped `TYPE 6` row in `MiniGameData.txt`.
pub const CHESS_ROW_IDS: [u32; 12] = [20, 65, 66, 67, 543, 544, 545, 546, 547, 548, 549, 550];
/// Distinct declared difficulties across those rows (1, 2, 3 and 5 appear).
pub const CHESS_DIFFICULTY_CODES: [u8; 4] = [1, 2, 3, 5];
/// `GameTutorialData.txt` row 10 and `GameChess.mTutorial`.
pub const CHESS_TUTORIAL_ID: u32 = 10;

pub const EVENT_INIT: u8 = 0;
pub const EVENT_TURN_START: u8 = 1;
pub const EVENT_TURN_END: u8 = 2;
pub const EVENT_MOVE: u8 = 10;
pub const EVENT_CASTLE: u8 = 11;
pub const EVENT_PROMOTE: u8 = 12;
pub const EVENT_ENPASSANT: u8 = 13;
pub const EVENT_CHECK: u8 = 14;
pub const EVENT_CHECKMATE: u8 = 15;
pub const EVENT_DRAW: u8 = 16;
pub const EVENT_STALEMATE: u8 = 17;
pub const EVENT_MOVE_INVALID: u8 = 20;
pub const EVENT_MOVE_REQUEST: u8 = 30;
pub const EVENT_MOVE_DATA: u8 = 31;
pub const EVENT_PLAYERLEFT: u8 = 40;
pub const EVENT_WAITING: u8 = 41;
pub const EVENT_LOCAL_AI: u8 = 42;

/// The seven names the movie actually populates in `ChessNetworkEvent.nameArray`
/// (`Draw`, `Stalemate`, `Check`, `Checkmate`, `MOVE_REQUEST`, `MOVE_DATA`, `PLAYERLEFT`,
/// `WAITING` and `LOCAL_AI` are declared as constants but left unnamed in the shipped movie).
pub const CHESS_EVENT_NAMES: [(u8, &str); 7] = [
    (EVENT_INIT, "Init"),
    (EVENT_TURN_START, "Start turn"),
    (EVENT_TURN_END, "End turn"),
    (EVENT_MOVE, "Move"),
    (EVENT_CASTLE, "Castle"),
    (EVENT_PROMOTE, "Promote"),
    (EVENT_MOVE_INVALID, "Invalid"),
];

pub fn event_name(code: u8) -> Option<&'static str> {
    CHESS_EVENT_NAMES
        .iter()
        .find(|(event, _)| *event == code)
        .map(|(_, name)| *name)
}

pub const PIECE_NONE: u8 = 0;
pub const PIECE_PAWN: u8 = 1;
pub const PIECE_ROOK: u8 = 2;
pub const PIECE_KNIGHT: u8 = 3;
pub const PIECE_BISHOP: u8 = 4;
pub const PIECE_QUEEN: u8 = 5;
pub const PIECE_KING: u8 = 6;

/// `Chess::piece_to_num` / `num_to_piece` in the native client agree with the movie's piece codes.
pub const PLAYER_CODES: [(Kind, char); 6] = [
    (Kind::Pawn, 'P'),
    (Kind::Rook, 'R'),
    (Kind::Knight, 'N'),
    (Kind::Bishop, 'B'),
    (Kind::Queen, 'Q'),
    (Kind::King, 'K'),
];

pub const CASTLE_FLAG_BLACK_QUEENSIDE: u8 = 0x01;
pub const CASTLE_FLAG_BLACK_KINGSIDE: u8 = 0x02;
pub const CASTLE_FLAG_WHITE_QUEENSIDE: u8 = 0x08;
pub const CASTLE_FLAG_WHITE_KINGSIDE: u8 = 0x10;
/// `Chess::convert_player_promote` sets this bit on a promotion move.
pub const FLAG_PROMOTION: u8 = 0x40;
/// `Chess::convert_player_move` sets this bit on a two-square pawn push.
pub const FLAG_DOUBLE_PUSH: u8 = 0x80;
/// `Chess::convert_player_move` sets these bits when the destination square is occupied.
pub const FLAG_OCCUPIED: u8 = 0x60;
/// `Chess::convert_player_move` uses this byte for an en passant capture.
pub const FLAG_EN_PASSANT: u8 = 0x64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub fn code(self) -> u8 {
        match self {
            Side::White => 0,
            Side::Black => 1,
        }
    }

    pub fn opponent(self) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl Kind {
    pub fn code(self) -> u8 {
        match self {
            Kind::Pawn => PIECE_PAWN,
            Kind::Rook => PIECE_ROOK,
            Kind::Knight => PIECE_KNIGHT,
            Kind::Bishop => PIECE_BISHOP,
            Kind::Queen => PIECE_QUEEN,
            Kind::King => PIECE_KING,
        }
    }

    pub fn from_code(code: u8) -> Option<Kind> {
        match code {
            PIECE_PAWN => Some(Kind::Pawn),
            PIECE_ROOK => Some(Kind::Rook),
            PIECE_KNIGHT => Some(Kind::Knight),
            PIECE_BISHOP => Some(Kind::Bishop),
            PIECE_QUEEN => Some(Kind::Queen),
            PIECE_KING => Some(Kind::King),
            _ => None,
        }
    }

    fn promotion_options() -> [Kind; 4] {
        [Kind::Queen, Kind::Rook, Kind::Bishop, Kind::Knight]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: Kind,
    pub side: Side,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub from: u8,
    pub to: u8,
    pub promote: Option<Kind>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    InProgress,
    Draw,
    WhiteMated,
    BlackMated,
    WhiteResigned,
    BlackResigned,
}

impl Status {
    /// The native status word: 0 in progress, 1 draw, 2/3 mate, 4/5 resign.
    pub fn code(self) -> u8 {
        match self {
            Status::InProgress => 0,
            Status::Draw => 1,
            Status::WhiteMated => 2,
            Status::BlackMated => 3,
            Status::WhiteResigned => 4,
            Status::BlackResigned => 5,
        }
    }

    pub fn from_code(code: u8) -> Option<Status> {
        match code {
            0 => Some(Status::InProgress),
            1 => Some(Status::Draw),
            2 => Some(Status::WhiteMated),
            3 => Some(Status::BlackMated),
            4 => Some(Status::WhiteResigned),
            5 => Some(Status::BlackResigned),
            _ => None,
        }
    }

    /// `Chess::get_winner`: {2,4} -> 1, {3,5} -> 0, everything else -> 2 (no winner).
    pub fn winner(self) -> Winner {
        match self {
            Status::WhiteMated | Status::WhiteResigned => Winner::Black,
            Status::BlackMated | Status::BlackResigned => Winner::White,
            Status::InProgress | Status::Draw => Winner::None,
        }
    }

    /// `Chess::is_resign`: status 4 or 5.
    pub fn is_resign(self) -> bool {
        matches!(self, Status::WhiteResigned | Status::BlackResigned)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Winner {
    White,
    Black,
    None,
}

impl Winner {
    pub fn code(self) -> u8 {
        match self {
            Winner::White => 0,
            Winner::Black => 1,
            Winner::None => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChessError {
    IllegalMove,
    GameOver,
    UnknownSession(u64),
    InvalidFen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveOutcome {
    pub captured: Option<Kind>,
    pub check: bool,
    pub status: Status,
}

fn file(square: u8) -> u8 {
    square % 8
}

fn row(square: u8) -> u8 {
    square / 8
}

fn piece_code(piece: Option<Piece>) -> u8 {
    match piece {
        None => 0,
        Some(piece) => {
            let base = piece.kind.code();
            match piece.side {
                Side::White => base,
                Side::Black => base + 8,
            }
        }
    }
}

#[derive(Clone)]
pub struct Position {
    squares: [Option<Piece>; 64],
    to_move: Side,
    /// 1 white O-O, 2 white O-O-O, 4 black O-O, 8 black O-O-O.
    castling: u8,
    en_passant: Option<u8>,
    halfmove: u16,
    fullmove: u32,
    /// `[piece codes; 64][to_move][castling][en passant]` for every position seen, oldest first.
    history: Vec<[u8; 67]>,
}

const CASTLE_WHITE_KINGSIDE: u8 = 1;
const CASTLE_WHITE_QUEENSIDE: u8 = 2;
const CASTLE_BLACK_KINGSIDE: u8 = 4;
const CASTLE_BLACK_QUEENSIDE: u8 = 8;

impl Position {
    pub fn start() -> Position {
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("static start position is valid")
    }

    pub fn from_fen(fen: &str) -> Result<Position, ChessError> {
        let fields: Vec<&str> = fen.split_whitespace().collect();
        if fields.len() < 4 {
            return Err(ChessError::InvalidFen);
        }
        let rows: Vec<&str> = fields[0].split('/').collect();
        if rows.len() != 8 {
            return Err(ChessError::InvalidFen);
        }
        let mut squares = [None; 64];
        for (row_index, row) in rows.iter().enumerate() {
            let board_rank = row_index as u8;
            let mut board_file = 0u8;
            for symbol in row.chars() {
                if let Some(skip) = symbol.to_digit(10) {
                    if skip == 0 || skip > 8 {
                        return Err(ChessError::InvalidFen);
                    }
                    board_file += skip as u8;
                    continue;
                }
                if board_file > 7 {
                    return Err(ChessError::InvalidFen);
                }
                let side = if symbol.is_ascii_uppercase() {
                    Side::White
                } else if symbol.is_ascii_lowercase() {
                    Side::Black
                } else {
                    return Err(ChessError::InvalidFen);
                };
                let kind = match symbol.to_ascii_lowercase() {
                    'p' => Kind::Pawn,
                    'r' => Kind::Rook,
                    'n' => Kind::Knight,
                    'b' => Kind::Bishop,
                    'q' => Kind::Queen,
                    'k' => Kind::King,
                    _ => return Err(ChessError::InvalidFen),
                };
                squares[(board_rank * 8 + board_file) as usize] = Some(Piece { kind, side });
                board_file += 1;
            }
            if board_file != 8 {
                return Err(ChessError::InvalidFen);
            }
        }

        let to_move = match fields[1] {
            "w" => Side::White,
            "b" => Side::Black,
            _ => return Err(ChessError::InvalidFen),
        };

        let mut castling = 0u8;
        if fields[2] != "-" {
            for flag in fields[2].chars() {
                castling |= match flag {
                    'K' => CASTLE_WHITE_KINGSIDE,
                    'Q' => CASTLE_WHITE_QUEENSIDE,
                    'k' => CASTLE_BLACK_KINGSIDE,
                    'q' => CASTLE_BLACK_QUEENSIDE,
                    _ => return Err(ChessError::InvalidFen),
                };
            }
        }

        let en_passant = if fields[3] == "-" {
            None
        } else {
            let bytes = fields[3].as_bytes();
            if bytes.len() != 2
                || !(b'a'..=b'h').contains(&bytes[0])
                || !(b'1'..=b'8').contains(&bytes[1])
            {
                return Err(ChessError::InvalidFen);
            }
            Some((b'8' - bytes[1]) * 8 + (bytes[0] - b'a'))
        };

        let halfmove = fields
            .get(4)
            .map(|value| value.parse::<u16>().map_err(|_| ChessError::InvalidFen))
            .transpose()?
            .unwrap_or(0);
        let fullmove = fields
            .get(5)
            .map(|value| value.parse::<u32>().map_err(|_| ChessError::InvalidFen))
            .transpose()?
            .unwrap_or(1);

        let mut position = Position {
            squares,
            to_move,
            castling,
            en_passant,
            halfmove,
            fullmove,
            history: Vec::new(),
        };
        position.history.push(position.key());
        Ok(position)
    }

    pub fn to_fen(&self) -> String {
        let mut board = String::new();
        for board_rank in 0..8 {
            let mut empty = 0u8;
            for board_file in 0..8 {
                match self.squares[(board_rank * 8 + board_file) as usize] {
                    None => empty += 1,
                    Some(piece) => {
                        if empty > 0 {
                            board.push_str(&empty.to_string());
                            empty = 0;
                        }
                        let letter = match piece.kind {
                            Kind::Pawn => 'p',
                            Kind::Rook => 'r',
                            Kind::Knight => 'n',
                            Kind::Bishop => 'b',
                            Kind::Queen => 'q',
                            Kind::King => 'k',
                        };
                        board.push(if piece.side == Side::White {
                            letter.to_ascii_uppercase()
                        } else {
                            letter
                        });
                    }
                }
            }
            if empty > 0 {
                board.push_str(&empty.to_string());
            }
            if board_rank < 7 {
                board.push('/');
            }
        }

        let mut castling = String::new();
        for (flag, letter) in [
            (CASTLE_WHITE_KINGSIDE, 'K'),
            (CASTLE_WHITE_QUEENSIDE, 'Q'),
            (CASTLE_BLACK_KINGSIDE, 'k'),
            (CASTLE_BLACK_QUEENSIDE, 'q'),
        ] {
            if self.castling & flag != 0 {
                castling.push(letter);
            }
        }
        if castling.is_empty() {
            castling.push('-');
        }

        let en_passant = match self.en_passant {
            None => "-".to_string(),
            Some(square) => format!(
                "{}{}",
                (b'a' + file(square)) as char,
                (b'8' - row(square)) as char
            ),
        };

        format!(
            "{board} {} {castling} {en_passant} {} {}",
            if self.to_move == Side::White {
                "w"
            } else {
                "b"
            },
            self.halfmove,
            self.fullmove
        )
    }

    pub fn side_to_move(&self) -> Side {
        self.to_move
    }

    pub fn piece_at(&self, square: u8) -> Option<Piece> {
        self.squares.get(square as usize).copied().flatten()
    }

    fn key(&self) -> [u8; 67] {
        let mut key = [0u8; 67];
        for (index, square) in self.squares.iter().enumerate() {
            key[index] = piece_code(*square);
        }
        key[64] = self.to_move.code();
        key[65] = self.castling;
        key[66] = self.en_passant.map(|square| square + 1).unwrap_or(0);
        key
    }

    fn repetition_count(&self) -> usize {
        let current = self.key();
        self.history.iter().filter(|key| **key == current).count()
    }

    fn king_square(&self, side: Side) -> Option<u8> {
        self.squares
            .iter()
            .position(|square| {
                matches!(
                    square,
                    Some(Piece {
                        kind: Kind::King,
                        side: found
                    }) if *found == side
                )
            })
            .map(|index| index as u8)
    }

    /// Is `square` attacked by `by`? Direct pattern test, no move generation.
    fn attacked_by(&self, square: u8, by: Side) -> bool {
        let pawn_rank = row(square);
        if by == Side::White {
            if pawn_rank < 7 {
                for delta in [-1i8, 1] {
                    let attacker_file = file(square) as i8 + delta;
                    if (0..8).contains(&attacker_file) {
                        let attacker = (pawn_rank + 1) * 8 + attacker_file as u8;
                        if self.squares[attacker as usize]
                            == Some(Piece {
                                kind: Kind::Pawn,
                                side: Side::White,
                            })
                        {
                            return true;
                        }
                    }
                }
            }
        } else if pawn_rank > 0 {
            for delta in [-1i8, 1] {
                let attacker_file = file(square) as i8 + delta;
                if (0..8).contains(&attacker_file) {
                    let attacker = (pawn_rank - 1) * 8 + attacker_file as u8;
                    if self.squares[attacker as usize]
                        == Some(Piece {
                            kind: Kind::Pawn,
                            side: Side::Black,
                        })
                    {
                        return true;
                    }
                }
            }
        }

        let square_file = file(square) as i8;
        let square_rank = row(square) as i8;
        for (delta_file, delta_rank) in [
            (-2i8, -1i8),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ] {
            let target_file = square_file + delta_file;
            let target_rank = square_rank + delta_rank;
            if (0..8).contains(&target_file) && (0..8).contains(&target_rank) {
                let attacker = (target_rank * 8 + target_file) as u8;
                if self.squares[attacker as usize]
                    == Some(Piece {
                        kind: Kind::Knight,
                        side: by,
                    })
                {
                    return true;
                }
            }
        }

        for (delta_file, delta_rank) in [
            (-1i8, -1i8),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ] {
            let target_file = square_file + delta_file;
            let target_rank = square_rank + delta_rank;
            if (0..8).contains(&target_file) && (0..8).contains(&target_rank) {
                let attacker = (target_rank * 8 + target_file) as u8;
                if self.squares[attacker as usize]
                    == Some(Piece {
                        kind: Kind::King,
                        side: by,
                    })
                {
                    return true;
                }
            }
        }

        for (delta_file, delta_rank, kinds) in [
            (1i8, 0i8, [Kind::Rook, Kind::Queen]),
            (-1, 0, [Kind::Rook, Kind::Queen]),
            (0, 1, [Kind::Rook, Kind::Queen]),
            (0, -1, [Kind::Rook, Kind::Queen]),
            (1, 1, [Kind::Bishop, Kind::Queen]),
            (1, -1, [Kind::Bishop, Kind::Queen]),
            (-1, 1, [Kind::Bishop, Kind::Queen]),
            (-1, -1, [Kind::Bishop, Kind::Queen]),
        ] {
            let mut target_file = square_file + delta_file;
            let mut target_rank = square_rank + delta_rank;
            while (0..8).contains(&target_file) && (0..8).contains(&target_rank) {
                let target = (target_rank * 8 + target_file) as u8;
                if let Some(piece) = self.squares[target as usize] {
                    if piece.side == by && kinds.contains(&piece.kind) {
                        return true;
                    }
                    break;
                }
                target_file += delta_file;
                target_rank += delta_rank;
            }
        }

        false
    }

    pub fn in_check(&self, side: Side) -> bool {
        match self.king_square(side) {
            Some(square) => self.attacked_by(square, side.opponent()),
            None => false,
        }
    }

    fn push(&self, moves: &mut Vec<ChessMove>, from: u8, to: u8) {
        moves.push(ChessMove {
            from,
            to,
            promote: None,
        });
    }

    fn push_pawn(&self, moves: &mut Vec<ChessMove>, from: u8, to: u8) {
        let promotion_rank =
            if self.squares[from as usize].map(|piece| piece.side) == Some(Side::White) {
                0
            } else {
                7
            };
        if row(to) == promotion_rank {
            for kind in Kind::promotion_options() {
                moves.push(ChessMove {
                    from,
                    to,
                    promote: Some(kind),
                });
            }
        } else {
            self.push(moves, from, to);
        }
    }

    fn pseudo_moves(&self) -> Vec<ChessMove> {
        let mut moves = Vec::new();
        for (index, square) in self.squares.iter().enumerate() {
            let Some(piece) = *square else { continue };
            if piece.side != self.to_move {
                continue;
            }
            let from = index as u8;
            let from_file = file(from) as i8;
            let from_rank = row(from) as i8;
            match piece.kind {
                Kind::Pawn => {
                    let forward: i8 = if piece.side == Side::White { -1 } else { 1 };
                    let start_rank = if piece.side == Side::White { 6 } else { 1 };
                    let one_step = from_rank + forward;
                    if (0..8).contains(&one_step) {
                        let one = (one_step * 8) as u8 + from_file as u8;
                        if self.squares[one as usize].is_none() {
                            self.push_pawn(&mut moves, from, one);
                            let two_step = from_rank + forward * 2;
                            if from_rank == start_rank {
                                let two = (two_step * 8) as u8 + from_file as u8;
                                if self.squares[two as usize].is_none() {
                                    self.push(&mut moves, from, two);
                                }
                            }
                        }
                    }
                    for delta_file in [-1i8, 1] {
                        let target_file = from_file + delta_file;
                        let target_rank = from_rank + forward;
                        if !(0..8).contains(&target_file) || !(0..8).contains(&target_rank) {
                            continue;
                        }
                        let target = (target_rank * 8 + target_file) as u8;
                        let occupied = self.squares[target as usize];
                        if let Some(victim) = occupied {
                            if victim.side != piece.side {
                                self.push_pawn(&mut moves, from, target);
                            }
                        } else if self.en_passant == Some(target) {
                            // The captured pawn sits directly behind the target square.
                            let victim_square = (target_rank * 8 + target_file) as i8 - forward * 8;
                            if (0..64).contains(&victim_square)
                                && self.squares[victim_square as usize]
                                    == Some(Piece {
                                        kind: Kind::Pawn,
                                        side: piece.side.opponent(),
                                    })
                            {
                                self.push_pawn(&mut moves, from, target);
                            }
                        }
                    }
                }
                Kind::Knight => {
                    for (delta_file, delta_rank) in [
                        (-2i8, -1i8),
                        (-2, 1),
                        (-1, -2),
                        (-1, 2),
                        (1, -2),
                        (1, 2),
                        (2, -1),
                        (2, 1),
                    ] {
                        let target_file = from_file + delta_file;
                        let target_rank = from_rank + delta_rank;
                        if !(0..8).contains(&target_file) || !(0..8).contains(&target_rank) {
                            continue;
                        }
                        let target = (target_rank * 8 + target_file) as u8;
                        if self.squares[target as usize]
                            .is_none_or(|other| other.side != piece.side)
                        {
                            self.push(&mut moves, from, target);
                        }
                    }
                }
                Kind::Bishop | Kind::Rook | Kind::Queen => {
                    let directions: &[(i8, i8)] = match piece.kind {
                        Kind::Bishop => &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
                        Kind::Rook => &[(1, 0), (-1, 0), (0, 1), (0, -1)],
                        _ => &[
                            (1, 1),
                            (1, -1),
                            (-1, 1),
                            (-1, -1),
                            (1, 0),
                            (-1, 0),
                            (0, 1),
                            (0, -1),
                        ],
                    };
                    for (delta_file, delta_rank) in directions {
                        let mut target_file = from_file + delta_file;
                        let mut target_rank = from_rank + delta_rank;
                        while (0..8).contains(&target_file) && (0..8).contains(&target_rank) {
                            let target = (target_rank * 8 + target_file) as u8;
                            match self.squares[target as usize] {
                                None => self.push(&mut moves, from, target),
                                Some(other) => {
                                    if other.side != piece.side {
                                        self.push(&mut moves, from, target);
                                    }
                                    break;
                                }
                            }
                            target_file += delta_file;
                            target_rank += delta_rank;
                        }
                    }
                }
                Kind::King => {
                    for (delta_file, delta_rank) in [
                        (-1i8, -1i8),
                        (-1, 0),
                        (-1, 1),
                        (0, -1),
                        (0, 1),
                        (1, -1),
                        (1, 0),
                        (1, 1),
                    ] {
                        let target_file = from_file + delta_file;
                        let target_rank = from_rank + delta_rank;
                        if !(0..8).contains(&target_file) || !(0..8).contains(&target_rank) {
                            continue;
                        }
                        let target = (target_rank * 8 + target_file) as u8;
                        if self.squares[target as usize]
                            .is_none_or(|other| other.side != piece.side)
                        {
                            self.push(&mut moves, from, target);
                        }
                    }
                    // Castling uses the rook that is still on its corner: the movie builds the
                    // castle move from both king and rook (`Model.Setup`), matching these rights.
                    if piece.side == Side::White && from == 60 {
                        if self.castling & CASTLE_WHITE_KINGSIDE != 0
                            && self.squares[61].is_none()
                            && self.squares[62].is_none()
                            && Some(Piece {
                                kind: Kind::Rook,
                                side: Side::White,
                            }) == self.squares[63]
                            && !self.attacked_by(60, Side::Black)
                            && !self.attacked_by(61, Side::Black)
                            && !self.attacked_by(62, Side::Black)
                        {
                            self.push(&mut moves, 60, 62);
                        }
                        if self.castling & CASTLE_WHITE_QUEENSIDE != 0
                            && self.squares[57].is_none()
                            && self.squares[58].is_none()
                            && self.squares[59].is_none()
                            && Some(Piece {
                                kind: Kind::Rook,
                                side: Side::White,
                            }) == self.squares[56]
                            && !self.attacked_by(60, Side::Black)
                            && !self.attacked_by(59, Side::Black)
                            && !self.attacked_by(58, Side::Black)
                        {
                            self.push(&mut moves, 60, 58);
                        }
                    } else if piece.side == Side::Black && from == 4 {
                        if self.castling & CASTLE_BLACK_KINGSIDE != 0
                            && self.squares[5].is_none()
                            && self.squares[6].is_none()
                            && Some(Piece {
                                kind: Kind::Rook,
                                side: Side::Black,
                            }) == self.squares[7]
                            && !self.attacked_by(4, Side::White)
                            && !self.attacked_by(5, Side::White)
                            && !self.attacked_by(6, Side::White)
                        {
                            self.push(&mut moves, 4, 6);
                        }
                        if self.castling & CASTLE_BLACK_QUEENSIDE != 0
                            && self.squares[1].is_none()
                            && self.squares[2].is_none()
                            && self.squares[3].is_none()
                            && Some(Piece {
                                kind: Kind::Rook,
                                side: Side::Black,
                            }) == self.squares[0]
                            && !self.attacked_by(4, Side::White)
                            && !self.attacked_by(3, Side::White)
                            && !self.attacked_by(2, Side::White)
                        {
                            self.push(&mut moves, 4, 2);
                        }
                    }
                }
            }
        }
        moves
    }

    /// Apply a move that is already known to be in `legal_moves`, returning the captured kind.
    fn apply_unchecked(&mut self, mv: &ChessMove) -> Option<Kind> {
        let piece = self.squares[mv.from as usize].expect("legal moves always start on a piece");
        let mut captured = None;
        let mut captured_square = None;

        if let Some(victim) = self.squares[mv.to as usize] {
            captured = Some(victim.kind);
            captured_square = Some(mv.to);
        } else if piece.kind == Kind::Pawn && file(mv.from) != file(mv.to) {
            // En passant: the victim is on the same file as the destination, one rank behind.
            let victim_rank = row(mv.from);
            let victim_square = victim_rank * 8 + file(mv.to);
            captured = Some(Kind::Pawn);
            captured_square = Some(victim_square);
        }

        self.squares[mv.from as usize] = None;
        let placed = match mv.promote {
            Some(kind) if piece.kind == Kind::Pawn => Piece {
                kind,
                side: piece.side,
            },
            _ => piece,
        };
        self.squares[mv.to as usize] = Some(placed);

        if let Some(square) = captured_square {
            self.squares[square as usize] = None;
            if square == 0 {
                self.castling &= !CASTLE_BLACK_QUEENSIDE;
            }
            if square == 7 {
                self.castling &= !CASTLE_BLACK_KINGSIDE;
            }
            if square == 56 {
                self.castling &= !CASTLE_WHITE_QUEENSIDE;
            }
            if square == 63 {
                self.castling &= !CASTLE_WHITE_KINGSIDE;
            }
        }

        // Move the rook when the king castles two files.
        if piece.kind == Kind::King && file(mv.from).abs_diff(file(mv.to)) == 2 {
            let (rook_from, rook_to) = match mv.to {
                6 => (7, 5),
                2 => (0, 3),
                62 => (63, 61),
                58 => (56, 59),
                _ => unreachable!("legal king castle has a known target"),
            };
            let rook = self.squares[rook_from as usize].take();
            self.squares[rook_to as usize] = rook;
        }

        if piece.kind == Kind::King {
            match piece.side {
                Side::White => self.castling &= !(CASTLE_WHITE_KINGSIDE | CASTLE_WHITE_QUEENSIDE),
                Side::Black => self.castling &= !(CASTLE_BLACK_KINGSIDE | CASTLE_BLACK_QUEENSIDE),
            }
        }
        if piece.kind == Kind::Rook {
            match mv.from {
                0 => self.castling &= !CASTLE_BLACK_QUEENSIDE,
                7 => self.castling &= !CASTLE_BLACK_KINGSIDE,
                56 => self.castling &= !CASTLE_WHITE_QUEENSIDE,
                63 => self.castling &= !CASTLE_WHITE_KINGSIDE,
                _ => {}
            }
        }

        self.en_passant = if piece.kind == Kind::Pawn
            && file(mv.from) == file(mv.to)
            && row(mv.from).abs_diff(row(mv.to)) == 2
        {
            Some((row(mv.from) + row(mv.to)) / 2 * 8 + file(mv.from))
        } else {
            None
        };

        if piece.kind == Kind::Pawn || captured.is_some() {
            self.halfmove = 0;
        } else {
            self.halfmove = self.halfmove.saturating_add(1);
        }
        if piece.side == Side::Black {
            self.fullmove += 1;
        }
        self.to_move = self.to_move.opponent();
        self.history.push(self.key());
        captured
    }

    pub fn legal_moves(&self) -> Vec<ChessMove> {
        let mut legal = Vec::new();
        for mv in self.pseudo_moves() {
            let mut probe = self.clone();
            let piece_side = probe.squares[mv.from as usize]
                .map(|piece| piece.side)
                .unwrap_or(self.to_move);
            probe.apply_unchecked(&mv);
            if !probe.in_check(piece_side) {
                legal.push(mv);
            }
        }
        legal
    }

    pub fn is_legal(&self, mv: &ChessMove) -> bool {
        self.legal_moves().contains(mv)
    }

    /// Positions with no pawns, rooks or queens where neither side can deliver mate:
    /// king versus king, a single minor piece, or opposite-side bishops on one colour.
    fn insufficient_material(&self) -> bool {
        let mut white_minors = Vec::new();
        let mut black_minors = Vec::new();
        let mut white_majors = 0;
        let mut black_majors = 0;
        for square in self.squares.iter().flatten() {
            match (square.side, square.kind) {
                (_, Kind::Pawn) | (_, Kind::Rook) | (_, Kind::Queen) => match square.side {
                    Side::White => white_majors += 1,
                    Side::Black => black_majors += 1,
                },
                (Side::White, Kind::Bishop) | (Side::White, Kind::Knight) => {
                    white_minors.push(*square)
                }
                (Side::Black, Kind::Bishop) | (Side::Black, Kind::Knight) => {
                    black_minors.push(*square)
                }
                _ => {}
            }
        }
        if white_majors > 0 || black_majors > 0 {
            return false;
        }
        if white_minors.is_empty() && black_minors.is_empty() {
            return true;
        }
        if white_minors.len() == 1
            && black_minors.is_empty()
            && white_minors[0].kind == Kind::Bishop
        {
            return true;
        }
        if black_minors.len() == 1
            && white_minors.is_empty()
            && black_minors[0].kind == Kind::Bishop
        {
            return true;
        }
        white_minors.len() == 1
            && black_minors.len() == 1
            && white_minors[0].kind == Kind::Bishop
            && black_minors[0].kind == Kind::Bishop
    }

    pub fn status(&self) -> Status {
        if self.legal_moves().is_empty() {
            if self.in_check(self.to_move) {
                return match self.to_move {
                    Side::White => Status::WhiteMated,
                    Side::Black => Status::BlackMated,
                };
            }
            return Status::Draw;
        }
        if self.insufficient_material() || self.repetition_count() >= 3 {
            return Status::Draw;
        }
        Status::InProgress
    }

    /// The fourth byte of the recovered native `ChessMove` for `mv` in this position.
    pub fn special_code(&self, mv: &ChessMove) -> u8 {
        let piece = self.squares[mv.from as usize];
        if let Some(Piece {
            kind: Kind::King,
            side,
        }) = piece
        {
            if file(mv.from).abs_diff(file(mv.to)) == 2 {
                return match (side, mv.to) {
                    (Side::White, 62) => CASTLE_FLAG_WHITE_KINGSIDE,
                    (Side::White, _) => CASTLE_FLAG_WHITE_QUEENSIDE,
                    (Side::Black, 6) => CASTLE_FLAG_BLACK_KINGSIDE,
                    (Side::Black, _) => CASTLE_FLAG_BLACK_QUEENSIDE,
                };
            }
        }
        let mut code = 0u8;
        if let Some(Piece {
            kind: Kind::Pawn, ..
        }) = piece
        {
            if file(mv.from) != file(mv.to) {
                code = FLAG_PROMOTION;
                if self.squares[mv.to as usize].is_none() {
                    code = FLAG_EN_PASSANT;
                }
            } else if row(mv.from).abs_diff(row(mv.to)) == 2 {
                code |= FLAG_DOUBLE_PUSH;
            }
        }
        if self.squares[mv.to as usize].is_some() {
            code |= FLAG_OCCUPIED;
        }
        if mv.promote.is_some() {
            code |= FLAG_PROMOTION;
        }
        code
    }

    /// Recovered client engine bytes `[from, to, promote, special]`; castle uses its sentinel.
    fn engine_move_bytes(&self, mv: &ChessMove) -> [u8; 4] {
        let special = self.special_code(mv);
        let promote = mv.promote.map(Kind::code).unwrap_or(PIECE_NONE);
        let is_castle = self.squares[mv.from as usize].is_some_and(|piece| {
            piece.kind == Kind::King && file(mv.from).abs_diff(file(mv.to)) == 2
        });
        if is_castle {
            [0, 0x3e, promote, special]
        } else {
            [mv.from, mv.to, promote, special]
        }
    }

    pub fn apply(&mut self, mv: &ChessMove) -> Result<MoveOutcome, ChessError> {
        if !self.is_legal(mv) {
            return Err(ChessError::IllegalMove);
        }
        let captured = self.apply_unchecked(mv);
        let status = self.status();
        Ok(MoveOutcome {
            captured,
            check: self.in_check(self.to_move),
            status,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub id: u64,
    pub fen: String,
    pub status_code: u8,
    pub winner_code: u8,
    pub side_to_move: u8,
    pub computer: Option<u8>,
    pub left_side: Option<Side>,
    /// The recovered four-byte engine move form, including the client's castle sentinel.
    pub moves: Vec<[u8; 4]>,
}

#[derive(Clone)]
pub struct ChessSession {
    id: u64,
    computer: Option<Side>,
    position: Position,
    status: Status,
    moves: Vec<[u8; 4]>,
    left: Option<Side>,
}

impl ChessSession {
    pub fn launch(id: u64, computer: Option<Side>) -> ChessSession {
        ChessSession {
            id,
            computer,
            position: Position::start(),
            status: Status::InProgress,
            moves: Vec::new(),
            left: None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn computer(&self) -> Option<Side> {
        self.computer
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn moves(&self) -> &[[u8; 4]] {
        &self.moves
    }

    pub fn left_side(&self) -> Option<Side> {
        self.left
    }

    /// Original `Chess::is_computer_move`: the side to move is the configured computer side.
    pub fn is_computer_move(&self) -> bool {
        self.computer == Some(self.position.side_to_move())
    }

    pub fn winner(&self) -> Winner {
        if self.left.is_some() {
            Winner::None
        } else {
            self.status.winner()
        }
    }

    pub fn play(&mut self, mv: &ChessMove) -> Result<Status, ChessError> {
        if self.status != Status::InProgress || self.left.is_some() {
            return Err(ChessError::GameOver);
        }
        let move_bytes = self.position.engine_move_bytes(mv);
        let outcome = self.position.apply(mv)?;
        self.moves.push(move_bytes);
        self.status = outcome.status;
        Ok(self.status)
    }

    /// Reconstructed API shape: the movie calls the native `resign` with no side argument, so the
    /// resigning side is passed explicitly here.
    pub fn resign(&mut self, side: Side) -> Result<Status, ChessError> {
        if self.status != Status::InProgress || self.left.is_some() {
            return Err(ChessError::GameOver);
        }
        self.status = match side {
            Side::White => Status::WhiteResigned,
            Side::Black => Status::BlackResigned,
        };
        Ok(self.status)
    }

    /// Original `PLAYERLEFT` event: the table closes after the message is dismissed and no winner
    /// is granted by the departure itself (`GameChess.OtherPlayerLeft` -> `EndOfGame("OtherLeft")`).
    pub fn player_left(&mut self, side: Side) {
        self.left = Some(side);
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            id: self.id,
            fen: self.position.to_fen(),
            status_code: self.status.code(),
            winner_code: self.winner().code(),
            side_to_move: self.position.side_to_move().code(),
            computer: self.computer.map(Side::code),
            left_side: self.left,
            moves: self.moves.clone(),
        }
    }
}

#[derive(Default)]
pub struct ChessSessions {
    sessions: HashMap<u64, ChessSession>,
}

impl ChessSessions {
    pub fn new() -> ChessSessions {
        ChessSessions::default()
    }

    /// Start (or restart) the table with this id. A relaunch discards the previous table state.
    pub fn launch(&mut self, id: u64, computer: Option<Side>) -> u64 {
        self.sessions.insert(id, ChessSession::launch(id, computer));
        id
    }

    pub fn get(&self, id: u64) -> Option<&ChessSession> {
        self.sessions.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut ChessSession> {
        self.sessions.get_mut(&id)
    }

    /// Rebuild the client view after a reconnect. Unknown ids are rejected instead of silently
    /// creating an empty table.
    pub fn reconnect(&mut self, id: u64) -> Result<Snapshot, ChessError> {
        match self.sessions.get(&id) {
            Some(session) => Ok(session.snapshot()),
            None => Err(ChessError::UnknownSession(id)),
        }
    }

    pub fn close(&mut self, id: u64) -> bool {
        self.sessions.remove(&id).is_some()
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}
