//! Chess table service for client minigame type 6.
//!
//! Original facts: the client uses a8=0, move bytes `[from,to,promote,special]`, family 39/14
//! NUL-terminated text payloads, and events documented below. Evidence is the shipped client map
//! (`00f49480`, `00c081a0`, `009be6b0`) and `notes/Build Guides/Gameplay/Chess.md`.
//!
//! Reconstructed policy (R): table pairing, `INIT` values, AI skill values, terminal ordering,
//! resign/forfeit results, and the 100-ply draw rule. The server chooses and applies the
//! computer side's moves; a client `OnMoveComputerMsg` is never trusted.

use crate::{
    minigame::{Activity, ActivityFinish, MinigameRuns, RunOutcome},
    power_ups::Rng,
    rewards::{RewardGrant, RewardState},
};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    io,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    White,
    Black,
}
impl Side {
    pub fn code(self) -> u8 {
        if self == Side::White { 0 } else { 1 }
    }
    pub fn opponent(self) -> Side {
        if self == Side::White {
            Side::Black
        } else {
            Side::White
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
            Kind::Pawn => 1,
            Kind::Rook => 2,
            Kind::Knight => 3,
            Kind::Bishop => 4,
            Kind::Queen => 5,
            Kind::King => 6,
        }
    }
    pub fn from_code(code: u8) -> Option<Kind> {
        match code {
            1 => Some(Kind::Pawn),
            2 => Some(Kind::Rook),
            3 => Some(Kind::Knight),
            4 => Some(Kind::Bishop),
            5 => Some(Kind::Queen),
            6 => Some(Kind::King),
            _ => None,
        }
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
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChessError {
    IllegalMove,
    GameOver,
    InvalidFen,
}

const CWK: u8 = 1;
const CWQ: u8 = 2;
const CBK: u8 = 4;
const CBQ: u8 = 8;
const PROMOTIONS: [Kind; 4] = [Kind::Queen, Kind::Rook, Kind::Bishop, Kind::Knight];

pub const FAMILY_MINIGAME: u16 = 39;
pub const SUB_PAYLOAD: u8 = 14;
pub const GAME_ID: i32 = 20;
pub const GAME_TYPE: i32 = 6;
pub const ROW_IDS: [u32; 12] = [20, 65, 66, 67, 543, 544, 545, 546, 547, 548, 549, 550];
/// Client `MiniGameData` row facts shared by every type-6 row: NAME_ID 950,
/// DESCRIPTION_ID 951, ICON_ID 917. The movie's own `GAME_ID` constant is 20.
pub const NAME_ID: i32 = 950;
pub const DESCRIPTION_ID: i32 = 951;
pub const ICON_ID: i32 = 917;
/// Flash movie loaded by `26/12` for the chess start screen (pack manifest).
pub const MOVIE: &str = "game_chess.gfx";

pub const EVENT_INIT: u8 = 0;
pub const EVENT_TURN_START: u8 = 1;
pub const EVENT_TURN_END: u8 = 2;
pub const EVENT_MOVE: u8 = 10;
pub const EVENT_CASTLE: u8 = 11;
pub const EVENT_PROMOTE: u8 = 12;
pub const EVENT_ENPASSANT: u8 = 13;
pub const EVENT_CHECK: u8 = 14;
pub const EVENT_CHECKMATE: u8 = 15;
pub const EVENT_MOVE_INVALID: u8 = 20;
pub const EVENT_MOVE_REQUEST: u8 = 30;
pub const EVENT_MOVE_DATA: u8 = 31;
pub const EVENT_PLAYERLEFT: u8 = 40;
pub const EVENT_WAITING: u8 = 41;
pub const EVENT_LOCAL_AI: u8 = 42;

// Server-side bound (R); the reviewed guide asks for payload strings under 128 characters.
const MAX_TEXT_BYTES: usize = 127;
/// (R) Default window, in `score_move` units, for the computer's near-equal move choice.
pub const DEFAULT_NEAR_EQUAL_MARGIN: i64 = 25;
/// (R) Content sanity bound for [`DEFAULT_NEAR_EQUAL_MARGIN`]. A window must
/// be positive: zero would always choose the single best move, which makes the
/// computer reply a function of the position and replayable.
pub const MAX_NEAR_EQUAL_MARGIN: i64 = 500;
/// (R) Content sanity bounds for the reconstructed chooser weights.
pub const MAX_SCORE_VALUE: i64 = 100_000_000;
pub const MAX_CENTRALISATION: i64 = 1_000;
const FLAG_EN_PASSANT: u8 = 0x64;
const FLAG_DOUBLE_PUSH: u8 = 0x80;
const FLAG_CAPTURE: u8 = 0x60;
const FLAG_PROMOTION: u8 = 0x40;

/// (R) Weights for the server's one-ply computer move chooser, in score units
/// compared only against each other. Content must supply every value, bounded
/// by [`MAX_SCORE_VALUE`] and [`MAX_CENTRALISATION`]; the compiled [`Default`]
/// only seeds a table catalogue until `ChessContent` applies its block.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChessScoring {
    /// Score added for a move that mates the opponent.
    pub mate: i64,
    /// Score added for a move that gives check.
    pub check: i64,
    /// Score added for a move that draws (a penalty).
    pub draw: i64,
    /// Score added when the move captures a pawn.
    pub pawn: i64,
    /// Score added when the move captures a rook; a promotion to rook scores
    /// the same value.
    pub rook: i64,
    /// Score added when the move captures a knight; a promotion scores the same.
    pub knight: i64,
    /// Score added when the move captures a bishop; a promotion scores the same.
    pub bishop: i64,
    /// Score added when the move captures a queen; a promotion scores the same.
    pub queen: i64,
    /// Score added when the move captures a king (unreachable in a legal game);
    /// kept so the block covers every kind.
    pub king: i64,
    /// Weight of the destination-centre term; the origin term is half.
    pub centralisation: i64,
}

impl Default for ChessScoring {
    fn default() -> Self {
        Self {
            mate: 1_000_000,
            check: 5_000,
            draw: -100_000,
            pawn: 100,
            rook: 500,
            knight: 300,
            bishop: 320,
            queen: 900,
            king: 10_000,
            centralisation: 1,
        }
    }
}

impl ChessScoring {
    pub fn value(self, kind: Kind) -> i64 {
        match kind {
            Kind::Pawn => self.pawn,
            Kind::Rook => self.rook,
            Kind::Knight => self.knight,
            Kind::Bishop => self.bishop,
            Kind::Queen => self.queen,
            Kind::King => self.king,
        }
    }

    /// Reject a content block outside the local bounds, never clamp it.
    pub fn validate(self) -> io::Result<()> {
        let values = [
            self.mate,
            self.pawn,
            self.rook,
            self.knight,
            self.bishop,
            self.queen,
            self.king,
        ];
        if values
            .iter()
            .any(|value| !(0..=MAX_SCORE_VALUE).contains(value))
            || self.check < 0
            || self.check > MAX_SCORE_VALUE
            || !(-MAX_SCORE_VALUE..=0).contains(&self.draw)
            || !(0..=MAX_CENTRALISATION).contains(&self.centralisation)
        {
            return Err(invalid("chess scoring is outside the local bounds"));
        }
        Ok(())
    }
}

/// Shipped row ids classify nine single-player rows and three two-player rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Single { skill: u8 },
    TwoPlayer,
}

impl Mode {
    pub fn for_row(row_id: u32) -> Option<Mode> {
        match row_id {
            20 | 543 | 547 => Some(Mode::Single { skill: 20 }),
            66 | 544 | 548 => Some(Mode::Single { skill: 60 }),
            67 | 545 | 549 => Some(Mode::Single { skill: 100 }),
            65 | 546 | 550 => Some(Mode::TwoPlayer),
            _ => None,
        }
    }
}

/// Reconstructed client-engine skill choices: Easy 20, Medium 60, Hard 100 (R).
pub fn skill_for_row(row_id: u32) -> Option<u8> {
    match Mode::for_row(row_id)? {
        Mode::Single { skill } => Some(skill),
        Mode::TwoPlayer => None,
    }
}

#[derive(Clone)]
pub struct Position {
    squares: [Option<Piece>; 64],
    turn: Side,
    castling: u8,
    ep: Option<u8>,
    halfmove: u16,
    fullmove: u32,
    history: Vec<[u8; 67]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveOutcome {
    pub captured: Option<Kind>,
    pub check: bool,
    pub status: Status,
}

impl Position {
    pub fn start() -> Position {
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn from_fen(fen: &str) -> Result<Position, ChessError> {
        let f: Vec<_> = fen.split_whitespace().collect();
        if f.len() < 4 {
            return Err(ChessError::InvalidFen);
        }
        let rows: Vec<_> = f[0].split('/').collect();
        if rows.len() != 8 {
            return Err(ChessError::InvalidFen);
        }
        let mut squares = [None; 64];
        for (rank, text) in rows.iter().enumerate() {
            let mut file = 0usize;
            for c in text.chars() {
                if let Some(n) = c.to_digit(10) {
                    if n == 0 || n > 8 {
                        return Err(ChessError::InvalidFen);
                    }
                    file += n as usize;
                    continue;
                }
                if file >= 8 {
                    return Err(ChessError::InvalidFen);
                }
                let side = if c.is_ascii_uppercase() {
                    Side::White
                } else {
                    Side::Black
                };
                let kind = match c.to_ascii_lowercase() {
                    'p' => Kind::Pawn,
                    'r' => Kind::Rook,
                    'n' => Kind::Knight,
                    'b' => Kind::Bishop,
                    'q' => Kind::Queen,
                    'k' => Kind::King,
                    _ => return Err(ChessError::InvalidFen),
                };
                squares[rank * 8 + file] = Some(Piece { kind, side });
                file += 1;
            }
            if file != 8 {
                return Err(ChessError::InvalidFen);
            }
        }
        let turn = match f[1] {
            "w" => Side::White,
            "b" => Side::Black,
            _ => return Err(ChessError::InvalidFen),
        };
        let mut castling = 0;
        if f[2] != "-" {
            for c in f[2].chars() {
                castling |= match c {
                    'K' => CWK,
                    'Q' => CWQ,
                    'k' => CBK,
                    'q' => CBQ,
                    _ => return Err(ChessError::InvalidFen),
                };
            }
        }
        let ep = if f[3] == "-" {
            None
        } else {
            let b = f[3].as_bytes();
            if b.len() != 2 || !(b'a'..=b'h').contains(&b[0]) || !(b'1'..=b'8').contains(&b[1]) {
                return Err(ChessError::InvalidFen);
            }
            Some((b'8' - b[1]) * 8 + b[0] - b'a')
        };
        let halfmove = f
            .get(4)
            .map(|s| s.parse().map_err(|_| ChessError::InvalidFen))
            .transpose()?
            .unwrap_or(0);
        let fullmove = f
            .get(5)
            .map(|s| s.parse().map_err(|_| ChessError::InvalidFen))
            .transpose()?
            .unwrap_or(1);
        let mut p = Position {
            squares,
            turn,
            castling,
            ep,
            halfmove,
            fullmove,
            history: Vec::new(),
        };
        p.history.push(p.key());
        Ok(p)
    }

    pub fn to_fen(&self) -> String {
        let mut board = String::new();
        for r in 0..8 {
            let mut empty = 0;
            for f in 0..8 {
                match self.squares[r * 8 + f] {
                    None => empty += 1,
                    Some(p) => {
                        if empty > 0 {
                            board.push(char::from(b'0' + empty));
                            empty = 0
                        }
                        let c = match p.kind {
                            Kind::Pawn => 'p',
                            Kind::Rook => 'r',
                            Kind::Knight => 'n',
                            Kind::Bishop => 'b',
                            Kind::Queen => 'q',
                            Kind::King => 'k',
                        };
                        board.push(if p.side == Side::White {
                            c.to_ascii_uppercase()
                        } else {
                            c
                        })
                    }
                }
            }
            if empty > 0 {
                board.push(char::from(b'0' + empty))
            }
            if r < 7 {
                board.push('/')
            }
        }
        let mut rights = String::new();
        for (flag, c) in [(CWK, 'K'), (CWQ, 'Q'), (CBK, 'k'), (CBQ, 'q')] {
            if self.castling & flag != 0 {
                rights.push(c)
            }
        }
        if rights.is_empty() {
            rights.push('-')
        }
        let ep = self.ep.map_or_else(
            || "-".to_owned(),
            |s| format!("{}{}", char::from(b'a' + s % 8), char::from(b'8' - s / 8)),
        );
        format!(
            "{board} {} {rights} {ep} {} {}",
            if self.turn == Side::White { "w" } else { "b" },
            self.halfmove,
            self.fullmove
        )
    }

    pub fn side_to_move(&self) -> Side {
        self.turn
    }

    pub fn piece_at(&self, square: u8) -> Option<Piece> {
        self.squares.get(square as usize).copied().flatten()
    }

    pub fn in_check(&self, side: Side) -> bool {
        self.king_square(side)
            .is_some_and(|sq| self.attacked_by(sq, side.opponent()))
    }

    fn king_square(&self, side: Side) -> Option<u8> {
        self.squares
            .iter()
            .position(|p| {
                *p == Some(Piece {
                    kind: Kind::King,
                    side,
                })
            })
            .map(|i| i as u8)
    }
    fn key(&self) -> [u8; 67] {
        let mut key = [0; 67];
        for (i, p) in self.squares.iter().enumerate() {
            key[i] = p.map_or(0, |x| {
                x.kind.code() + if x.side == Side::Black { 8 } else { 0 }
            })
        }
        key[64] = self.turn.code();
        key[65] = self.castling;
        key[66] = self.ep.map_or(0, |s| s + 1);
        key
    }
    fn attacked_by(&self, s: u8, by: Side) -> bool {
        let (f, r) = (i16::from(s % 8), i16::from(s / 8));
        let pr = r + if by == Side::White { 1 } else { -1 };
        for df in [-1, 1] {
            let x = f + df;
            if (0..8).contains(&x)
                && (0..8).contains(&pr)
                && self.squares[(pr * 8 + x) as usize]
                    == Some(Piece {
                        kind: Kind::Pawn,
                        side: by,
                    })
            {
                return true;
            }
        }
        for (df, dr) in [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ] {
            let (x, y) = (f + df, r + dr);
            if (0..8).contains(&x)
                && (0..8).contains(&y)
                && self.squares[(y * 8 + x) as usize]
                    == Some(Piece {
                        kind: Kind::Knight,
                        side: by,
                    })
            {
                return true;
            }
        }
        for (df, dr) in [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ] {
            let (x, y) = (f + df, r + dr);
            if (0..8).contains(&x)
                && (0..8).contains(&y)
                && self.squares[(y * 8 + x) as usize]
                    == Some(Piece {
                        kind: Kind::King,
                        side: by,
                    })
            {
                return true;
            }
        }
        for (df, dr, diag) in [
            (1, 0, false),
            (-1, 0, false),
            (0, 1, false),
            (0, -1, false),
            (1, 1, true),
            (1, -1, true),
            (-1, 1, true),
            (-1, -1, true),
        ] {
            let (mut x, mut y) = (f + df, r + dr);
            while (0..8).contains(&x) && (0..8).contains(&y) {
                if let Some(p) = self.squares[(y * 8 + x) as usize] {
                    if p.side == by
                        && (p.kind == Kind::Queen
                            || (diag && p.kind == Kind::Bishop)
                            || (!diag && p.kind == Kind::Rook))
                    {
                        return true;
                    }
                    break;
                }
                x += df;
                y += dr
            }
        }
        false
    }

    pub fn legal_moves(&self) -> Vec<ChessMove> {
        self.pseudo_moves()
            .into_iter()
            .filter(|m| {
                if self.piece_at(m.to).is_some_and(|p| p.kind == Kind::King) {
                    return false;
                }
                let mut n = self.clone();
                n.apply_unchecked(m);
                !n.in_check(self.turn)
            })
            .collect()
    }

    pub fn is_legal(&self, mv: &ChessMove) -> bool {
        self.legal_moves().contains(mv)
    }

    pub fn status(&self) -> Status {
        let legal = self.legal_moves();
        if legal.is_empty() {
            return if self.in_check(self.side_to_move()) {
                match self.side_to_move() {
                    Side::White => Status::WhiteMated,
                    Side::Black => Status::BlackMated,
                }
            } else {
                Status::Draw
            };
        }
        let key = self.key();
        if self.halfmove >= 100
            || self.history.iter().filter(|k| **k == key).count() >= 3
            || self.insufficient_material()
        {
            Status::Draw
        } else {
            Status::InProgress
        }
    }

    pub fn is_stalemate(&self) -> bool {
        self.legal_moves().is_empty() && !self.in_check(self.side_to_move())
    }

    pub fn apply(&mut self, mv: &ChessMove) -> Result<MoveOutcome, ChessError> {
        if !self.is_legal(mv) {
            return Err(ChessError::IllegalMove);
        }
        let captured = self.apply_unchecked(mv);
        Ok(MoveOutcome {
            captured,
            check: self.in_check(self.side_to_move()),
            status: self.status(),
        })
    }

    pub fn perft(&self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }
        self.legal_moves()
            .into_iter()
            .map(|m| {
                let mut n = self.clone();
                n.apply_unchecked(&m);
                n.perft(depth - 1)
            })
            .sum()
    }

    fn pseudo_moves(&self) -> Vec<ChessMove> {
        let mut out = Vec::new();
        for (i, slot) in self.squares.iter().enumerate() {
            let Some(p) = *slot else { continue };
            if p.side != self.turn {
                continue;
            }
            let from = i as u8;
            let (f, r) = (i16::from(from % 8), i16::from(from / 8));
            match p.kind {
                Kind::Pawn => {
                    let step = if p.side == Side::White { -1 } else { 1 };
                    let start = if p.side == Side::White { 6 } else { 1 };
                    let nr = r + step;
                    if (0..8).contains(&nr) {
                        let one = (nr * 8 + f) as u8;
                        if self.squares[one as usize].is_none() {
                            push_pawn(&mut out, p, from, one);
                            let tr = r + step * 2;
                            if r == start && self.squares[(tr * 8 + f) as usize].is_none() {
                                out.push(ChessMove {
                                    from,
                                    to: (tr * 8 + f) as u8,
                                    promote: None,
                                })
                            }
                        }
                        for df in [-1, 1] {
                            let tf = f + df;
                            if !(0..8).contains(&tf) {
                                continue;
                            }
                            let to = (nr * 8 + tf) as u8;
                            if self.squares[to as usize]
                                .is_some_and(|v| v.side != p.side && v.kind != Kind::King)
                            {
                                push_pawn(&mut out, p, from, to)
                            } else if self.squares[to as usize].is_none()
                                && self.ep == Some(to)
                                && self.squares[(r * 8 + tf) as usize]
                                    == Some(Piece {
                                        kind: Kind::Pawn,
                                        side: p.side.opponent(),
                                    })
                            {
                                push_pawn(&mut out, p, from, to)
                            }
                        }
                    }
                }
                Kind::Knight => {
                    for (df, dr) in [
                        (-2, -1),
                        (-2, 1),
                        (-1, -2),
                        (-1, 2),
                        (1, -2),
                        (1, 2),
                        (2, -1),
                        (2, 1),
                    ] {
                        push_step(self, &mut out, p, from, f + df, r + dr)
                    }
                }
                Kind::Bishop | Kind::Rook | Kind::Queen => {
                    let dirs: &[(i16, i16)] = match p.kind {
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
                    for (df, dr) in dirs {
                        let (mut x, mut y) = (f + df, r + dr);
                        while (0..8).contains(&x) && (0..8).contains(&y) {
                            let to = (y * 8 + x) as u8;
                            match self.squares[to as usize] {
                                None => out.push(ChessMove {
                                    from,
                                    to,
                                    promote: None,
                                }),
                                Some(v) => {
                                    if v.side != p.side && v.kind != Kind::King {
                                        out.push(ChessMove {
                                            from,
                                            to,
                                            promote: None,
                                        })
                                    }
                                    break;
                                }
                            }
                            x += df;
                            y += dr
                        }
                    }
                }
                Kind::King => {
                    for (df, dr) in [
                        (-1, -1),
                        (-1, 0),
                        (-1, 1),
                        (0, -1),
                        (0, 1),
                        (1, -1),
                        (1, 0),
                        (1, 1),
                    ] {
                        push_step(self, &mut out, p, from, f + df, r + dr)
                    }
                    self.add_castles(&mut out, p, from)
                }
            }
        }
        out
    }

    fn add_castles(&self, out: &mut Vec<ChessMove>, p: Piece, from: u8) {
        let (king, enemy, kf, qf, kr, qr) = if p.side == Side::White {
            (60, Side::Black, CWK, CWQ, 63, 56)
        } else {
            (4, Side::White, CBK, CBQ, 7, 0)
        };
        if from != king {
            return;
        }
        let ks_safe = [king, king + 1, king + 2]
            .into_iter()
            .all(|s| !self.attacked_by(s, enemy));
        if self.castling & kf != 0
            && self.squares[(king + 1) as usize].is_none()
            && self.squares[(king + 2) as usize].is_none()
            && self.squares[kr as usize]
                == Some(Piece {
                    kind: Kind::Rook,
                    side: p.side,
                })
            && ks_safe
        {
            out.push(ChessMove {
                from,
                to: king + 2,
                promote: None,
            })
        }
        let qs_safe = [king - 2, king - 1, king]
            .into_iter()
            .all(|s| !self.attacked_by(s, enemy));
        if self.castling & qf != 0
            && self.squares[(king - 1) as usize].is_none()
            && self.squares[(king - 2) as usize].is_none()
            && self.squares[(king - 3) as usize].is_none()
            && self.squares[qr as usize]
                == Some(Piece {
                    kind: Kind::Rook,
                    side: p.side,
                })
            && qs_safe
        {
            out.push(ChessMove {
                from,
                to: king - 2,
                promote: None,
            })
        }
    }

    fn apply_unchecked(&mut self, m: &ChessMove) -> Option<Kind> {
        let p = self.squares[m.from as usize].unwrap();
        let mut capture_square = if self.squares[m.to as usize].is_some() {
            Some(m.to)
        } else {
            None
        };
        let captured = self.squares[m.to as usize].map(|v| v.kind).or_else(|| {
            if p.kind == Kind::Pawn && m.from % 8 != m.to % 8 {
                capture_square = Some((m.from / 8) * 8 + m.to % 8);
                Some(Kind::Pawn)
            } else {
                None
            }
        });
        self.squares[m.from as usize] = None;
        if let Some(s) = capture_square {
            self.squares[s as usize] = None;
            match s {
                0 => self.castling &= !CBQ,
                7 => self.castling &= !CBK,
                56 => self.castling &= !CWQ,
                63 => self.castling &= !CWK,
                _ => {}
            }
        }
        self.squares[m.to as usize] = Some(Piece {
            kind: m.promote.unwrap_or(p.kind),
            side: p.side,
        });
        if p.kind == Kind::King && (i16::from(m.from % 8) - i16::from(m.to % 8)).abs() == 2 {
            let (rf, rt) = match m.to {
                6 => (7, 5),
                2 => (0, 3),
                62 => (63, 61),
                58 => (56, 59),
                _ => unreachable!(),
            };
            self.squares[rt] = self.squares[rf].take()
        }
        if p.kind == Kind::King {
            self.castling &= if p.side == Side::White {
                !(CWK | CWQ)
            } else {
                !(CBK | CBQ)
            }
        }
        if p.kind == Kind::Rook {
            match m.from {
                0 => self.castling &= !CBQ,
                7 => self.castling &= !CBK,
                56 => self.castling &= !CWQ,
                63 => self.castling &= !CWK,
                _ => {}
            }
        }
        self.ep =
            if p.kind == Kind::Pawn && (i16::from(m.from / 8) - i16::from(m.to / 8)).abs() == 2 {
                Some((m.from / 8 + m.to / 8) / 2 * 8 + m.from % 8)
            } else {
                None
            };
        if p.kind == Kind::Pawn || captured.is_some() {
            self.halfmove = 0
        } else {
            self.halfmove = self.halfmove.saturating_add(1)
        }
        if p.side == Side::Black {
            self.fullmove += 1
        }
        self.turn = self.turn.opponent();
        self.history.push(self.key());
        captured
    }

    fn insufficient_material(&self) -> bool {
        let mut minors = Vec::new();
        for (i, p) in self.squares.iter().enumerate() {
            if let Some(p) = p {
                match p.kind {
                    Kind::King => {}
                    Kind::Bishop | Kind::Knight => minors.push((p.kind, i as u8)),
                    _ => return false,
                }
            }
        }
        match minors.as_slice() {
            [] => true,
            [(_, _)] => true,
            [(Kind::Bishop, a), (Kind::Bishop, b)] => {
                ((a % 8 + a / 8) % 2) == ((b % 8 + b / 8) % 2)
            }
            _ => false,
        }
    }

    /// Exact engine byte form from the Chess guide. The 008 candidate's castle flag assignment
    /// was reversed; this method follows its event destination mapping and native encoder bytes.
    pub fn engine_move_bytes(&self, mv: &ChessMove) -> Result<[u8; 4], ChessError> {
        if !self.is_legal(mv) {
            return Err(ChessError::IllegalMove);
        }
        let piece = self.piece_at(mv.from).ok_or(ChessError::IllegalMove)?;
        if piece.kind == Kind::King && (i16::from(mv.from % 8) - i16::from(mv.to % 8)).abs() == 2 {
            let flag = match mv.to {
                58 => 0x01,
                62 => 0x02,
                2 => 0x08,
                6 => 0x10,
                _ => return Err(ChessError::IllegalMove),
            };
            return Ok([0, 0x3e, 0, flag]);
        }
        let mut special = 0;
        if piece.kind == Kind::Pawn {
            if mv.from % 8 != mv.to % 8 && self.piece_at(mv.to).is_none() {
                special = FLAG_EN_PASSANT;
            } else if (i16::from(mv.from / 8) - i16::from(mv.to / 8)).abs() == 2 {
                special = FLAG_DOUBLE_PUSH;
            }
        }
        if self.piece_at(mv.to).is_some() {
            special = FLAG_CAPTURE;
        }
        if mv.promote.is_some() && special == 0 {
            special = FLAG_PROMOTION;
        }
        Ok([
            mv.from,
            mv.to,
            mv.promote.map(Kind::code).unwrap_or(0),
            special,
        ])
    }

    pub fn move_from_engine_bytes(&self, bytes: [u8; 4]) -> Option<ChessMove> {
        self.legal_moves()
            .into_iter()
            .find(|mv| self.engine_move_bytes(mv).ok() == Some(bytes))
    }
}

fn push_pawn(out: &mut Vec<ChessMove>, piece: Piece, from: u8, to: u8) {
    if to / 8 == if piece.side == Side::White { 0 } else { 7 } {
        for kind in PROMOTIONS {
            out.push(ChessMove {
                from,
                to,
                promote: Some(kind),
            })
        }
    } else {
        out.push(ChessMove {
            from,
            to,
            promote: None,
        })
    }
}
fn push_step(
    pos: &Position,
    out: &mut Vec<ChessMove>,
    piece: Piece,
    from: u8,
    file: i16,
    rank: i16,
) {
    if !(0..8).contains(&file) || !(0..8).contains(&rank) {
        return;
    }
    let to = (rank * 8 + file) as u8;
    if pos.squares[to as usize]
        .is_none_or(|other| other.side != piece.side && other.kind != Kind::King)
    {
        out.push(ChessMove {
            from,
            to,
            promote: None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessParticipant {
    pub player_id: u64,
    pub instance_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndReason {
    Checkmate,
    Stalemate,
    Draw,
    Resignation,
    Disconnect,
    EngineNoMove,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessResult {
    pub winner: Option<u64>,
    pub reason: EndReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChessMessage {
    pub player_id: u64,
    pub text: String,
    pub packet: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChessOutcome {
    pub accepted: bool,
    pub changed: bool,
    pub messages: Vec<ChessMessage>,
    pub terminal: Option<ChessResult>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TablePhase {
    Waiting,
    Playing,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TableMode {
    Single {
        human_side: Side,
        computer_side: Side,
        skill: u8,
        /// (R) Near-equal window handed down from content.
        near_equal_margin: i64,
    },
    TwoPlayer,
}

#[derive(Clone)]
pub struct ChessTable {
    table_id: u64,
    row_id: u32,
    mode: TableMode,
    white: ChessParticipant,
    black: Option<ChessParticipant>,
    position: Position,
    phase: TablePhase,
    connected: HashSet<u64>,
    ready: HashSet<u64>,
    result: Option<ChessResult>,
    game_over: HashSet<u64>,
    settled: HashSet<u64>,
    /// (R) Per-table random stream for the computer's near-equal move choice.
    rng: Rng,
    /// (R) Content-supplied chooser weights for this table.
    scoring: ChessScoring,
}

impl ChessTable {
    /// A content-supplied skill (0 to 100) and near-equal window. `seed`
    /// starts the table's private random stream.
    fn single_with_skill(
        table_id: u64,
        row_id: u32,
        player: ChessParticipant,
        human_side: Side,
        skill: u8,
        near_equal_margin: i64,
        scoring: ChessScoring,
        seed: u64,
    ) -> io::Result<ChessTable> {
        if !matches!(Mode::for_row(row_id), Some(Mode::Single { .. }))
            || skill > 100
            || !(1..=MAX_NEAR_EQUAL_MARGIN).contains(&near_equal_margin)
        {
            return Err(invalid("invalid single-player chess row, skill or window"));
        }
        validate_participant(player)?;
        let computer_side = human_side.opponent();
        let (white, black) = if human_side == Side::White {
            (player, None)
        } else {
            (
                ChessParticipant {
                    player_id: 0,
                    instance_id: player.instance_id,
                },
                Some(player),
            )
        };
        Ok(ChessTable {
            table_id,
            row_id,
            mode: TableMode::Single {
                human_side,
                computer_side,
                skill,
                near_equal_margin,
            },
            white,
            black,
            position: Position::start(),
            phase: TablePhase::Waiting,
            connected: HashSet::new(),
            ready: HashSet::new(),
            result: None,
            game_over: HashSet::new(),
            settled: HashSet::new(),
            rng: Rng::seeded(seed),
            scoring,
        })
    }

    fn two_player(
        table_id: u64,
        row_id: u32,
        white: ChessParticipant,
        black: ChessParticipant,
        scoring: ChessScoring,
        seed: u64,
    ) -> io::Result<ChessTable> {
        if Mode::for_row(row_id) != Some(Mode::TwoPlayer) {
            return Err(invalid("row is not a two-player chess row"));
        }
        validate_participant(white)?;
        validate_participant(black)?;
        if white.player_id == black.player_id || white.instance_id == black.instance_id {
            return Err(invalid(
                "chess players and minigame instances must be distinct",
            ));
        }
        Ok(ChessTable {
            table_id,
            row_id,
            mode: TableMode::TwoPlayer,
            white,
            black: Some(black),
            position: Position::start(),
            phase: TablePhase::Waiting,
            connected: HashSet::new(),
            ready: HashSet::new(),
            result: None,
            game_over: HashSet::new(),
            settled: HashSet::new(),
            rng: Rng::seeded(seed),
            scoring,
        })
    }

    pub fn id(&self) -> u64 {
        self.table_id
    }

    pub fn row_id(&self) -> u32 {
        self.row_id
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn result(&self) -> Option<ChessResult> {
        self.result
    }

    pub fn settled_count(&self) -> usize {
        self.settled.len()
    }

    fn participant(&self, player_id: u64) -> Option<ChessParticipant> {
        if self.white.player_id == player_id {
            return Some(self.white);
        }
        self.black
            .filter(|participant| participant.player_id == player_id)
    }

    fn participant_for_side(&self, side: Side) -> Option<ChessParticipant> {
        match side {
            Side::White if self.white.player_id != 0 => Some(self.white),
            Side::White => None,
            Side::Black => self.black,
        }
    }

    fn emit(&self, out: &mut ChessOutcome, player_id: u64, text: String) -> io::Result<()> {
        let Some(participant) = self.participant(player_id) else {
            return Err(invalid("chess message recipient is not seated at table"));
        };
        out.messages.push(ChessMessage {
            player_id,
            packet: payload_packet(participant.instance_id, &text)?,
            text,
        });
        Ok(())
    }

    fn event(
        &self,
        out: &mut ChessOutcome,
        player_id: u64,
        event: u8,
        args: &[String],
    ) -> io::Result<()> {
        let mut text = format!("OnChessEventMsg\t{event}");
        for arg in args {
            text.push('\t');
            text.push_str(arg);
        }
        self.emit(out, player_id, text)
    }

    fn event_all(&self, out: &mut ChessOutcome, event: u8, args: &[String]) -> io::Result<()> {
        for participant in self.players() {
            self.event(out, participant.player_id, event, args)?;
        }
        Ok(())
    }

    fn players(&self) -> Vec<ChessParticipant> {
        let mut players = Vec::new();
        if self.white.player_id != 0 {
            players.push(self.white);
        }
        if let Some(black) = self.black.filter(|black| black.player_id != 0) {
            players.push(black);
        }
        players
    }

    fn start(&mut self) -> io::Result<ChessOutcome> {
        self.phase = TablePhase::Playing;
        let mut out = ChessOutcome {
            accepted: true,
            changed: true,
            ..Default::default()
        };
        match self.mode {
            TableMode::Single {
                human_side,
                computer_side,
                ..
            } => {
                let player = self.participant_for_side(human_side).unwrap();
                // Equality selects white. The actual meaning of INIT values is unknown (R).
                self.event(
                    &mut out,
                    player.player_id,
                    EVENT_INIT,
                    &init_args(human_side),
                )?;
                self.event(
                    &mut out,
                    player.player_id,
                    EVENT_LOCAL_AI,
                    &[computer_side.code().to_string()],
                )?;
                if computer_side == Side::White {
                    let reply = self.play_computer_turn(player.player_id)?;
                    append_outcome(&mut out, reply);
                } else {
                    self.event(&mut out, player.player_id, EVENT_TURN_START, &[])?;
                }
            }
            TableMode::TwoPlayer => {
                for side in [Side::White, Side::Black] {
                    let participant = self.participant_for_side(side).unwrap();
                    self.event(
                        &mut out,
                        participant.player_id,
                        EVENT_INIT,
                        &init_args(side),
                    )?;
                }
                self.event_all(&mut out, EVENT_TURN_START, &[])?;
            }
        }
        Ok(out)
    }

    /// (R) Server-authoritative computer reply, pushed as a normal move event.
    /// The shipped movie applies it in its network state (`game_chess`
    /// `WaitingForNetwork.OnMoveMsg`); its native engine only moves on event
    /// 30, whose binding (`ChessMoveRequest` 0x00c081a0) is the sole caller of
    /// `Chess::make_computer_move` 0x00f507c0, and this server never sends it.
    fn play_computer_turn(&mut self, player_id: u64) -> io::Result<ChessOutcome> {
        let Some(mv) = self.server_move() else {
            return Ok(ChessOutcome::default());
        };
        self.accept_move(player_id, mv, true)
    }

    /// (R) Pick a legal reply locally from every move within the near-equal
    /// window of the best score. The choice comes from this table's
    /// server-side random stream, so a recorded winning game cannot be
    /// replayed against the same replies; `skill` widens the window for
    /// easier rows. The client engine's proposed move is never an input here,
    /// so a modified client cannot choose the opponent's move or its loss.
    fn server_move(&mut self) -> Option<ChessMove> {
        let mut ranked: Vec<(i64, ChessMove)> = self
            .position
            .legal_moves()
            .into_iter()
            .map(|mv| (self.score_move(&mv), mv))
            .collect();
        ranked.sort_by(|(sa, a), (sb, b)| {
            sb.cmp(sa)
                .then_with(|| a.from.cmp(&b.from))
                .then_with(|| a.to.cmp(&b.to))
                .then_with(|| a.promote.map(Kind::code).cmp(&b.promote.map(Kind::code)))
        });
        let (skill, margin) = match self.mode {
            TableMode::Single {
                skill,
                near_equal_margin,
                ..
            } => (skill, near_equal_margin),
            TableMode::TwoPlayer => (100, DEFAULT_NEAR_EQUAL_MARGIN),
        };
        let window = margin + margin * i64::from(100u8.saturating_sub(skill)) / 100;
        let floor = ranked.first()?.0.saturating_sub(window);
        let candidates = ranked
            .iter()
            .take_while(|(score, _)| *score >= floor)
            .count();
        let index = (self.rng.next_u64() % candidates.max(1) as u64) as usize;
        ranked.get(index).map(|(_, mv)| *mv)
    }

    fn score_move(&self, mv: &ChessMove) -> i64 {
        let Ok(outcome) = self.position.clone().apply(mv) else {
            return i64::MIN;
        };
        let mut score = 0;
        match outcome.status {
            Status::WhiteMated | Status::BlackMated => score += self.scoring.mate,
            Status::Draw => score += self.scoring.draw,
            Status::InProgress => {}
        }
        if outcome.check {
            score += self.scoring.check;
        }
        if let Some(kind) = outcome.captured {
            score += self.scoring.value(kind);
        }
        if let Some(kind) = mv.promote {
            score += self.scoring.value(kind);
        }
        let centre = |square: u8| {
            7 - (i64::from(square % 8) - 3)
                .abs()
                .max((i64::from(square / 8) - 3).abs())
        };
        score + (centre(mv.to) - centre(mv.from) / 2) * self.scoring.centralisation
    }

    fn handle_text(&mut self, player_id: u64, message: &str) -> io::Result<ChessOutcome> {
        let Some(participant) = self.participant(player_id) else {
            return Ok(ChessOutcome::default());
        };
        let fields: Vec<&str> = message.split('\t').collect();
        if fields.is_empty()
            || fields[0].is_empty()
            || fields.iter().any(|field| field.contains('\0'))
        {
            return Err(invalid("malformed chess text payload"));
        }
        match fields[0] {
            "OnConnectMsg" if fields.len() == 1 => {
                if !self.connected.insert(player_id) {
                    return Ok(ChessOutcome::default());
                }
                let mut out = ChessOutcome {
                    accepted: true,
                    changed: true,
                    ..Default::default()
                };
                self.emit(&mut out, player_id, "OnServerReadyMsg".into())?;
                Ok(out)
            }
            "OnStartGameMsg" if fields.len() == 2 && fields[1] == "0" => {
                if self.phase != TablePhase::Waiting || !self.connected.contains(&player_id) {
                    return Ok(ChessOutcome::default());
                }
                if !self.ready.insert(player_id) {
                    return Ok(ChessOutcome::default());
                }
                if matches!(self.mode, TableMode::Single { .. }) || self.ready.len() == 2 {
                    self.start()
                } else {
                    let mut out = ChessOutcome {
                        accepted: true,
                        changed: true,
                        ..Default::default()
                    };
                    self.event(&mut out, participant.player_id, EVENT_WAITING, &[])?;
                    Ok(out)
                }
            }
            "OnMoveMsg" | "OnCastleMsg" | "OnPromoteMsg" | "OnEnPassantMsg" => {
                self.handle_human_move(player_id, &fields)
            }
            // The computer is server-authoritative. Older/malicious clients may
            // still send the native engine's reply; it is inert here.
            "OnMoveComputerMsg" => Ok(ChessOutcome::default()),
            "FRServer_GameOver" if fields.len() == 1 => self.game_over(player_id),
            "FRServer_GameClose" if fields.len() == 1 => Ok(ChessOutcome::default()),
            _ => Ok(ChessOutcome::default()),
        }
    }

    fn handle_human_move(&mut self, player_id: u64, fields: &[&str]) -> io::Result<ChessOutcome> {
        if self.phase != TablePhase::Playing || self.result.is_some() {
            return Ok(ChessOutcome::default());
        }
        let side = self.side_for_player(player_id);
        if side != Some(self.position.side_to_move()) {
            return self.invalid_move(player_id);
        }
        let Some(mv) = parse_human_move(&self.position, fields) else {
            return self.invalid_move(player_id);
        };
        self.accept_move(player_id, mv, false)
    }

    fn side_for_player(&self, player_id: u64) -> Option<Side> {
        if self.white.player_id == player_id {
            return Some(Side::White);
        }
        if self.black.is_some_and(|black| black.player_id == player_id) {
            return Some(Side::Black);
        }
        None
    }

    fn invalid_move(&self, player_id: u64) -> io::Result<ChessOutcome> {
        let mut out = ChessOutcome {
            accepted: false,
            changed: false,
            ..Default::default()
        };
        self.event(&mut out, player_id, EVENT_MOVE_INVALID, &[])?;
        Ok(out)
    }

    fn accept_move(
        &mut self,
        mover_id: u64,
        mv: ChessMove,
        computer_move: bool,
    ) -> io::Result<ChessOutcome> {
        let captured_at_target = self.position.piece_at(mv.to).is_some();
        let engine_bytes = self
            .position
            .engine_move_bytes(&mv)
            .map_err(|_| invalid("move became illegal before encoding"))?;
        let outcome = self
            .position
            .apply(&mv)
            .map_err(|_| invalid("move became illegal before application"))?;
        let mut out = ChessOutcome {
            accepted: true,
            changed: true,
            ..Default::default()
        };

        if computer_move {
            let participant = self.participant(mover_id).unwrap();
            self.send_move_event(&mut out, participant.player_id, &mv, captured_at_target)?;
        } else if matches!(self.mode, TableMode::Single { .. }) {
            self.event(
                &mut out,
                mover_id,
                EVENT_MOVE_DATA,
                &engine_bytes.iter().map(u8::to_string).collect::<Vec<_>>(),
            )?;
        } else {
            let recipient = self
                .players()
                .into_iter()
                .find(|player| player.player_id != mover_id)
                .unwrap();
            self.send_move_event(&mut out, recipient.player_id, &mv, captured_at_target)?;
        }

        // The client does not reveal whether CHECK/CHECKMATE precedes TURN_END; this order is R.
        if self.position.in_check(self.position.side_to_move()) {
            self.event_all(&mut out, EVENT_CHECK, &[])?;
        }
        match outcome.status {
            Status::WhiteMated | Status::BlackMated => {
                self.event_all(&mut out, EVENT_CHECKMATE, &[])?;
                self.event_all(&mut out, EVENT_TURN_END, &[])?;
                let result = ChessResult {
                    winner: if computer_move && matches!(self.mode, TableMode::Single { .. }) {
                        None
                    } else {
                        Some(mover_id)
                    },
                    reason: EndReason::Checkmate,
                };
                self.emit_result_messages(&mut out, result, None)?;
                self.finish(result, &mut out);
            }
            Status::Draw => {
                self.event_all(&mut out, EVENT_TURN_END, &[])?;
                let reason = if self.position.is_stalemate() {
                    EndReason::Stalemate
                } else {
                    EndReason::Draw
                };
                // R: the movie has no draw/stalemate handler, so the coordinator closes the run.
                for participant in self.players() {
                    self.emit(&mut out, participant.player_id, "OnGameStoppedMsg".into())?;
                }
                self.finish(
                    ChessResult {
                        winner: None,
                        reason,
                    },
                    &mut out,
                );
            }
            _ => {
                self.event_all(&mut out, EVENT_TURN_END, &[])?;
                match self.mode {
                    TableMode::Single {
                        human_side,
                        computer_side,
                        ..
                    } if self.position.side_to_move() == computer_side => {
                        let player = self.participant_for_side(human_side).unwrap();
                        let reply = self.play_computer_turn(player.player_id)?;
                        append_outcome(&mut out, reply);
                    }
                    TableMode::Single { human_side, .. } => {
                        let player = self.participant_for_side(human_side).unwrap();
                        self.event(&mut out, player.player_id, EVENT_TURN_START, &[])?;
                    }
                    TableMode::TwoPlayer => self.event_all(&mut out, EVENT_TURN_START, &[])?,
                }
            }
        }
        Ok(out)
    }

    fn send_move_event(
        &self,
        out: &mut ChessOutcome,
        player_id: u64,
        mv: &ChessMove,
        captured_at_target: bool,
    ) -> io::Result<()> {
        let piece = self.position.piece_at(mv.to).unwrap();
        if piece.kind == Kind::King && (i16::from(mv.from % 8) - i16::from(mv.to % 8)).abs() == 2 {
            self.event(out, player_id, EVENT_CASTLE, &[mv.to.to_string()])
        } else if let Some(kind) = mv.promote {
            self.event(
                out,
                player_id,
                EVENT_PROMOTE,
                &[
                    mv.from.to_string(),
                    mv.to.to_string(),
                    kind.code().to_string(),
                ],
            )
        } else if piece.kind == Kind::Pawn && mv.from % 8 != mv.to % 8 && !captured_at_target {
            // After application, the captured en-passant pawn is one rank behind the destination.
            let captured = (mv.from / 8) * 8 + mv.to % 8;
            self.event(
                out,
                player_id,
                EVENT_ENPASSANT,
                &[mv.from.to_string(), mv.to.to_string(), captured.to_string()],
            )
        } else {
            self.event(
                out,
                player_id,
                EVENT_MOVE,
                &[mv.from.to_string(), mv.to.to_string()],
            )
        }
    }

    fn finish(&mut self, result: ChessResult, out: &mut ChessOutcome) {
        self.phase = TablePhase::Finished;
        self.result = Some(result);
        out.terminal = Some(result);
    }

    fn game_over(&mut self, player_id: u64) -> io::Result<ChessOutcome> {
        let Some(result) = self.result else {
            return Ok(ChessOutcome::default());
        };
        if matches!(result.reason, EndReason::Draw | EndReason::Stalemate)
            || self.participant(player_id).is_none()
            || !self.game_over.insert(player_id)
        {
            return Ok(ChessOutcome::default());
        }
        Ok(ChessOutcome {
            accepted: true,
            changed: true,
            terminal: Some(result),
            ..Default::default()
        })
    }

    fn resign(&mut self, player_id: u64) -> io::Result<ChessOutcome> {
        if self.phase != TablePhase::Playing
            || self.result.is_some()
            || self.participant(player_id).is_none()
        {
            return Ok(ChessOutcome::default());
        }
        let winner = self
            .players()
            .into_iter()
            .find(|p| p.player_id != player_id)
            .map(|p| p.player_id);
        let result = ChessResult {
            winner,
            reason: EndReason::Resignation,
        };
        let mut out = ChessOutcome {
            accepted: true,
            changed: true,
            ..Default::default()
        };
        self.emit_result_messages(&mut out, result, None)?;
        self.finish(result, &mut out);
        Ok(out)
    }

    fn disconnect(&mut self, player_id: u64) -> io::Result<ChessOutcome> {
        if self.phase != TablePhase::Playing
            || self.result.is_some()
            || self.participant(player_id).is_none()
        {
            return Ok(ChessOutcome::default());
        }
        let winner = self
            .players()
            .into_iter()
            .find(|p| p.player_id != player_id)
            .map(|p| p.player_id);
        let result = ChessResult {
            winner,
            reason: EndReason::Disconnect,
        };
        let mut out = ChessOutcome {
            accepted: true,
            changed: true,
            ..Default::default()
        };
        // A disconnected seat cannot answer the movie's GameOver message; settlement is ready.
        self.game_over.insert(player_id);
        if let Some(winner_id) = winner {
            self.event(&mut out, winner_id, EVENT_PLAYERLEFT, &[])?;
        }
        self.emit_result_messages(&mut out, result, Some(player_id))?;
        self.finish(result, &mut out);
        Ok(out)
    }

    fn emit_result_messages(
        &self,
        out: &mut ChessOutcome,
        result: ChessResult,
        skip_player: Option<u64>,
    ) -> io::Result<()> {
        for participant in self.players() {
            if Some(participant.player_id) == skip_player {
                continue;
            }
            let message = if result.winner == Some(participant.player_id) {
                "OnGameWonMsg"
            } else {
                "OnGameLostMsg"
            };
            self.emit(out, participant.player_id, message.into())?;
        }
        Ok(())
    }

    fn settle_player(
        &mut self,
        player_id: u64,
        runs: &mut MinigameRuns,
        score: u32,
        stars: u32,
        elapsed_seconds: u32,
        reward: Option<(&RewardState, &RewardGrant)>,
    ) -> io::Result<RunOutcome> {
        let Some(result) = self.result else {
            return Ok(RunOutcome::default());
        };
        if self.settled.contains(&player_id) {
            return Ok(RunOutcome::default());
        }
        let Some(participant) = self.participant(player_id) else {
            return Ok(RunOutcome::default());
        };
        if !self.game_over.contains(&player_id)
            && !matches!(result.reason, EndReason::Draw | EndReason::Stalemate)
        {
            return Ok(RunOutcome::default());
        }
        // R: ActivityFinish has no draw value; a draw or stalemate settles as a loss for both seats.
        let won = result.winner == Some(player_id);
        let activity = ChessFinishActivity {
            score,
            result: ActivityFinish {
                won,
                stars,
                elapsed_seconds,
            },
        };
        // A capped win pays nothing; the named settlement keeps the shared
        // lifecycle's win/grant invariant for every other activity.
        let outcome = match (won, reward) {
            (true, Some((state, grant))) => {
                runs.finish(participant.instance_id, &activity, Some((state, grant)))?
            }
            (true, None) => runs.finish_win_without_reward(participant.instance_id, &activity)?,
            (false, _) => runs.finish(participant.instance_id, &activity, None)?,
        };
        if outcome.accepted {
            self.settled.insert(player_id);
        }
        Ok(outcome)
    }
}

struct ChessFinishActivity {
    score: u32,
    result: ActivityFinish,
}

impl Activity for ChessFinishActivity {
    fn score(&self) -> u32 {
        self.score
    }
    fn finish(&self) -> Option<ActivityFinish> {
        Some(self.result)
    }
}

#[derive(Clone)]
pub struct ChessTables {
    tables: HashMap<u64, ChessTable>,
    /// (R) Per-process seed from OS entropy. Each opened table mixes it with an
    /// open counter, so replaying a game cannot reproduce the computer's moves.
    seed: u64,
    opened: u64,
    /// (R) Chooser weights handed to every table this catalogue opens.
    scoring: ChessScoring,
}

impl ChessTables {
    /// (R) OS-entropy seed for the server's tables. Each open mixes it with the
    /// table id and an open counter, so replaying cannot reproduce replies.
    pub fn entropy() -> io::Result<Self> {
        Ok(Self::with_seed(Rng::from_entropy()?.next_u64()))
    }

    /// (R) Deterministic seed source for tests and offline fixtures. No client
    /// value reaches either constructor.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            tables: HashMap::new(),
            seed,
            opened: 0,
            scoring: ChessScoring::default(),
        }
    }

    /// (R) The chooser weights every table opened after this call uses. The
    /// block is validated here so no caller can bypass its bounds.
    pub(crate) fn set_scoring(&mut self, scoring: ChessScoring) -> io::Result<()> {
        scoring.validate()?;
        self.scoring = scoring;
        Ok(())
    }

    /// (R) A deterministic stream for tests and offline fixtures. Only tables
    /// opened after this call use the new seed.
    pub(crate) fn reseed(&mut self, seed: u64) {
        self.seed = seed;
        self.opened = 0;
    }

    fn open_seed(&mut self, table_id: u64) -> u64 {
        self.opened = self.opened.wrapping_add(1);
        Rng::mix(self.seed ^ table_id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ self.opened)
    }

    pub fn open_single(
        &mut self,
        table_id: u64,
        row_id: u32,
        player: ChessParticipant,
        human_side: Side,
    ) -> io::Result<()> {
        if self.tables.contains_key(&table_id) {
            return Err(invalid("chess table already exists"));
        }
        let Some(skill) = skill_for_row(row_id) else {
            return Err(invalid("row is not a single-player chess row"));
        };
        let seed = self.open_seed(table_id);
        self.tables.insert(
            table_id,
            ChessTable::single_with_skill(
                table_id,
                row_id,
                player,
                human_side,
                skill,
                DEFAULT_NEAR_EQUAL_MARGIN,
                self.scoring,
                seed,
            )?,
        );
        Ok(())
    }

    /// Open a single-player table with the content row's `skill` and
    /// near-equal window instead of the defaults for that row id.
    pub fn open_single_with_skill(
        &mut self,
        table_id: u64,
        row_id: u32,
        player: ChessParticipant,
        human_side: Side,
        skill: u8,
        near_equal_margin: i64,
    ) -> io::Result<()> {
        if self.tables.contains_key(&table_id) {
            return Err(invalid("chess table already exists"));
        }
        let seed = self.open_seed(table_id);
        self.tables.insert(
            table_id,
            ChessTable::single_with_skill(
                table_id,
                row_id,
                player,
                human_side,
                skill,
                near_equal_margin,
                self.scoring,
                seed,
            )?,
        );
        Ok(())
    }

    pub fn open_two_player(
        &mut self,
        table_id: u64,
        row_id: u32,
        white: ChessParticipant,
        black: ChessParticipant,
    ) -> io::Result<()> {
        if self.tables.contains_key(&table_id) {
            return Err(invalid("chess table already exists"));
        }
        let seed = self.open_seed(table_id);
        self.tables.insert(
            table_id,
            ChessTable::two_player(table_id, row_id, white, black, self.scoring, seed)?,
        );
        Ok(())
    }

    pub fn table(&self, table_id: u64) -> Option<&ChessTable> {
        self.tables.get(&table_id)
    }

    pub fn table_mut(&mut self, table_id: u64) -> Option<&mut ChessTable> {
        self.tables.get_mut(&table_id)
    }

    pub fn handle_payload(
        &mut self,
        table_id: u64,
        player_id: u64,
        packet: &[u8],
    ) -> io::Result<ChessOutcome> {
        let table = self
            .tables
            .get_mut(&table_id)
            .ok_or_else(|| invalid("unknown chess table"))?;
        let participant = table
            .participant(player_id)
            .ok_or_else(|| invalid("player is not seated at chess table"))?;
        let message = parse_payload_packet(participant.instance_id, packet)?;
        table.handle_text(player_id, &message)
    }

    pub fn resign(&mut self, table_id: u64, player_id: u64) -> io::Result<ChessOutcome> {
        self.tables
            .get_mut(&table_id)
            .ok_or_else(|| invalid("unknown chess table"))?
            .resign(player_id)
    }

    pub fn disconnect(&mut self, table_id: u64, player_id: u64) -> io::Result<ChessOutcome> {
        self.tables
            .get_mut(&table_id)
            .ok_or_else(|| invalid("unknown chess table"))?
            .disconnect(player_id)
    }

    pub fn settle_player(
        &mut self,
        table_id: u64,
        player_id: u64,
        runs: &mut MinigameRuns,
        score: u32,
        stars: u32,
        elapsed_seconds: u32,
        reward: Option<(&RewardState, &RewardGrant)>,
    ) -> io::Result<RunOutcome> {
        self.tables
            .get_mut(&table_id)
            .ok_or_else(|| invalid("unknown chess table"))?
            .settle_player(player_id, runs, score, stars, elapsed_seconds, reward)
    }

    pub fn close(&mut self, table_id: u64) -> bool {
        self.tables.remove(&table_id).is_some()
    }
}

/// Build the exact 39/14 header/body. The first id is the minigame instance; group is -1 and the
/// Chess movie id is 20. The payload length includes its trailing NUL (native `add eax,1`).
pub fn payload_packet(instance_id: i32, message: &str) -> io::Result<Vec<u8>> {
    validate_text(message)?;
    let mut packet = vec![39, 0, SUB_PAYLOAD];
    for value in [instance_id, -1, GAME_ID] {
        packet.extend(value.to_le_bytes());
    }
    let length = i32::try_from(message.len() + 1).map_err(|_| invalid("chess payload too long"))?;
    packet.extend(length.to_le_bytes());
    packet.extend(message.as_bytes());
    packet.push(0);
    Ok(packet)
}

fn parse_payload_packet(instance_id: i32, packet: &[u8]) -> io::Result<String> {
    if packet.len() < 20 || packet[..3] != [39, 0, SUB_PAYLOAD] {
        return Err(invalid("not a family 39/14 chess payload"));
    }
    let read_i32 =
        |offset: usize| i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap());
    if read_i32(3) != instance_id || read_i32(7) != -1 || !matches!(read_i32(11), -1 | GAME_ID) {
        return Err(invalid("chess payload header does not match table"));
    }
    let length =
        usize::try_from(read_i32(15)).map_err(|_| invalid("negative chess payload length"))?;
    if length == 0
        || length > MAX_TEXT_BYTES + 1
        || packet.len() != 19 + length
        || packet.last() != Some(&0)
    {
        return Err(invalid("invalid NUL-terminated chess payload length"));
    }
    if packet[19..packet.len() - 1].contains(&0) {
        return Err(invalid("embedded NUL in chess payload"));
    }
    String::from_utf8(packet[19..packet.len() - 1].to_vec())
        .map_err(|_| invalid("chess payload is not valid UTF-8"))
}

fn validate_text(message: &str) -> io::Result<()> {
    if message.is_empty() || message.len() > MAX_TEXT_BYTES || message.as_bytes().contains(&0) {
        return Err(invalid("chess payload must be 1 to 127 bytes without NUL"));
    }
    Ok(())
}

fn init_args(side: Side) -> Vec<String> {
    if side == Side::White {
        vec!["1".into(), "1".into()]
    } else {
        vec!["1".into(), "0".into()]
    }
}

fn parse_human_move(position: &Position, fields: &[&str]) -> Option<ChessMove> {
    let number = |index: usize| fields.get(index)?.parse::<u8>().ok();
    let (from, to) = match fields[0] {
        "OnMoveMsg" if fields.len() == 3 => (number(1)?, number(2)?),
        "OnCastleMsg" if fields.len() == 2 => {
            let from = match position.side_to_move() {
                Side::White => 60,
                Side::Black => 4,
            };
            (from, number(1)?)
        }
        "OnPromoteMsg" if fields.len() == 4 => (number(1)?, number(2)?),
        "OnEnPassantMsg" if fields.len() == 4 => (number(1)?, number(2)?),
        _ => return None,
    };
    if from >= 64 || to >= 64 {
        return None;
    }
    let piece = position.piece_at(from)?;
    let mv = match fields[0] {
        "OnCastleMsg" => {
            if piece.kind != Kind::King || (i16::from(from % 8) - i16::from(to % 8)).abs() != 2 {
                return None;
            }
            ChessMove {
                from,
                to,
                promote: None,
            }
        }
        "OnPromoteMsg" => {
            let kind = Kind::from_code(number(3)?)?;
            if !matches!(kind, Kind::Rook | Kind::Knight | Kind::Bishop | Kind::Queen) {
                return None;
            }
            ChessMove {
                from,
                to,
                promote: Some(kind),
            }
        }
        "OnEnPassantMsg" => {
            let captured = number(3)?;
            if piece.kind != Kind::Pawn
                || from % 8 == to % 8
                || position.piece_at(to).is_some()
                || captured != (from / 8) * 8 + to % 8
            {
                return None;
            }
            ChessMove {
                from,
                to,
                promote: None,
            }
        }
        _ => {
            if piece.kind == Kind::King && (i16::from(from % 8) - i16::from(to % 8)).abs() == 2 {
                return None;
            }
            if piece.kind == Kind::Pawn && matches!(to / 8, 0 | 7) {
                return None;
            }
            ChessMove {
                from,
                to,
                promote: None,
            }
        }
    };
    position.is_legal(&mv).then_some(mv)
}

fn validate_participant(participant: ChessParticipant) -> io::Result<()> {
    if participant.player_id == 0 || participant.instance_id <= 0 {
        return Err(invalid("invalid chess participant identity"));
    }
    Ok(())
}

fn append_outcome(out: &mut ChessOutcome, next: ChessOutcome) {
    out.accepted |= next.accepted;
    out.changed |= next.changed;
    out.messages.extend(next.messages);
    if next.terminal.is_some() {
        out.terminal = next.terminal;
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Black queen on d4 can take the white bishop on c5 or the rook on e5.
    fn scoring_table(scoring: ChessScoring) -> ChessTable {
        let mut table = ChessTable::single_with_skill(
            1,
            20,
            ChessParticipant {
                player_id: 1,
                instance_id: 1,
            },
            Side::White,
            100,
            // A positive window is required; 1 still isolates the top move here
            // because the two captures differ by more than 100 points.
            1,
            scoring,
            7,
        )
        .unwrap();
        table.position = Position::from_fen("7k/8/8/2B1R3/3q4/8/8/7K b - - 0 1").unwrap();
        table
    }

    #[test]
    fn content_piece_values_choose_the_computer_reply() {
        let mut defaults = scoring_table(ChessScoring {
            check: 0,
            centralisation: 0,
            ..ChessScoring::default()
        });
        assert_eq!(
            defaults.server_move(),
            Some(ChessMove {
                from: 35,
                to: 28,
                promote: None
            }),
            "the default scoring takes the rook"
        );
        let mut flipped = scoring_table(ChessScoring {
            check: 0,
            centralisation: 0,
            rook: 1,
            bishop: 999,
            ..ChessScoring::default()
        });
        assert_eq!(
            flipped.server_move(),
            Some(ChessMove {
                from: 35,
                to: 26,
                promote: None
            }),
            "content piece values decide the capture"
        );
    }

    #[test]
    fn table_scoring_setter_refuses_out_of_bounds() {
        let mut tables = ChessTables::with_seed(7);
        assert!(
            tables
                .set_scoring(ChessScoring {
                    mate: -1,
                    ..ChessScoring::default()
                })
                .is_err()
        );
    }

    #[test]
    fn content_scoring_is_bounded() {
        ChessScoring::default().validate().unwrap();
        for scoring in [
            ChessScoring {
                mate: -1,
                ..ChessScoring::default()
            },
            ChessScoring {
                draw: 1,
                ..ChessScoring::default()
            },
            ChessScoring {
                queen: MAX_SCORE_VALUE + 1,
                ..ChessScoring::default()
            },
            ChessScoring {
                centralisation: MAX_CENTRALISATION + 1,
                ..ChessScoring::default()
            },
        ] {
            assert!(scoring.validate().is_err());
        }
    }
}
