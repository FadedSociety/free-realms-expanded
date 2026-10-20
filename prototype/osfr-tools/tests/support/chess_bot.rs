//! Shared deterministic chess opponent for the chess integration tests.
//!
//! Mirrors the shipped movie: the human's drag applies locally, and the
//! server's move events (10-13) are applied for the computer side. The move
//! choice is a deterministic greedy line; both test crates drive it.
use osfr_tools::chess::{ChessMove, Kind, Position, Status};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndOfGame {
    Won,
    Lost,
    Stopped,
}

/// Mirrors the shipped movie: user moves are applied locally by the drag, and
/// the server's move events (10-13) are applied for the other side.
pub struct Human {
    pub position: Position,
    /// FEN identity without move counters, for the repetition penalty.
    seen: BTreeSet<String>,
}

impl Human {
    pub fn new() -> Self {
        let position = Position::start();
        let mut human = Self {
            position,
            seen: BTreeSet::new(),
        };
        human.remember();
        human
    }

    pub fn remember(&mut self) {
        let key = self
            .position
            .to_fen()
            .split(' ')
            .take(4)
            .collect::<Vec<_>>()
            .join(" ");
        self.seen.insert(key);
    }

    /// Deterministic greedy play: mate in one, then checks, captures, the
    /// closest approach to the enemy king and pawn advances. Positions that
    /// would repeat twice are demoted so the bot does not shuffle into a draw.
    pub fn choose(&self) -> ChessMove {
        let legal = self.position.legal_moves();
        let enemy = self.position.side_to_move().opponent();
        let enemy_king = (0..64)
            .find(|square| {
                self.position
                    .piece_at(*square)
                    .is_some_and(|piece| piece.kind == Kind::King && piece.side == enemy)
            })
            .expect("the enemy king is on the board");
        let halfmove: i64 = self
            .position
            .to_fen()
            .split(' ')
            .nth(4)
            .and_then(|field| field.parse().ok())
            .unwrap_or(0);
        let mut ranked = legal
            .iter()
            .map(|mv| {
                let mut next = self.position.clone();
                let outcome = next.apply(mv).expect("a legal move applies");
                let mut score = 0;
                if matches!(outcome.status, Status::WhiteMated | Status::BlackMated) {
                    score += 1_000_000;
                } else if outcome.status == Status::Draw {
                    // Never trade into stalemate, repetition or a bare-king draw.
                    score -= 1_000_000;
                }
                if outcome.check {
                    score += 50_000;
                }
                if let Some(kind) = outcome.captured {
                    score += 2_000 * piece_rank(kind);
                }
                if let Some(kind) = mv.promote {
                    score += 800 * piece_rank(kind);
                }
                if let Some(piece) = self.position.piece_at(mv.from) {
                    if piece.kind == Kind::Pawn {
                        score += i64::from(6 - mv.to / 8) * 30;
                        // Reset the halfmove clock before the 100-ply draw rule bites.
                        if halfmove > 60 {
                            score += 20_000;
                        }
                    }
                }
                let distance = (i64::from(mv.to % 8) - i64::from(enemy_king % 8))
                    .abs()
                    .max((i64::from(mv.to / 8) - i64::from(enemy_king / 8)).abs());
                score += (14 - distance) * 50;
                let key = next
                    .to_fen()
                    .split(' ')
                    .take(4)
                    .collect::<Vec<_>>()
                    .join(" ");
                if self.seen.iter().filter(|seen| **seen == key).count() >= 2 {
                    score -= 1_000_000;
                }
                (score, *mv)
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|(sa, a), (sb, b)| {
            sb.cmp(sa)
                .then_with(|| a.from.cmp(&b.from))
                .then_with(|| a.to.cmp(&b.to))
        });
        ranked.first().expect("a legal human move").1
    }

    pub fn move_text(&self, mv: &ChessMove) -> String {
        let piece = self.position.piece_at(mv.from).expect("mover");
        if piece.kind == Kind::King && (i16::from(mv.from % 8) - i16::from(mv.to % 8)).abs() == 2 {
            return format!("OnCastleMsg\t{}", mv.to);
        }
        if let Some(kind) = mv.promote {
            return format!("OnPromoteMsg\t{}\t{}\t{}", mv.from, mv.to, kind.code());
        }
        if piece.kind == Kind::Pawn
            && mv.from % 8 != mv.to % 8
            && self.position.piece_at(mv.to).is_none()
        {
            let captured = (mv.from / 8) * 8 + mv.to % 8;
            return format!("OnEnPassantMsg\t{}\t{}\t{}", mv.from, mv.to, captured);
        }
        format!("OnMoveMsg\t{}\t{}", mv.from, mv.to)
    }

    /// Applies the server's own move events and reports a terminal message.
    pub fn apply_events(&mut self, texts: &[String]) -> Option<EndOfGame> {
        for text in texts {
            let Some(rest) = text.strip_prefix("OnChessEventMsg\t") else {
                continue;
            };
            let fields: Vec<&str> = rest.split('\t').collect();
            let mv = match fields.first().copied() {
                Some("10") => ChessMove {
                    from: fields[1].parse().unwrap(),
                    to: fields[2].parse().unwrap(),
                    promote: None,
                },
                Some("11") => {
                    let to: u8 = fields[1].parse().unwrap();
                    let from = match to {
                        2 | 6 => 4,
                        58 | 62 => 60,
                        _ => panic!("unknown castle destination {to}"),
                    };
                    ChessMove {
                        from,
                        to,
                        promote: None,
                    }
                }
                Some("12") => ChessMove {
                    from: fields[1].parse().unwrap(),
                    to: fields[2].parse().unwrap(),
                    promote: Kind::from_code(fields[3].parse().unwrap()),
                },
                Some("13") => ChessMove {
                    from: fields[1].parse().unwrap(),
                    to: fields[2].parse().unwrap(),
                    promote: None,
                },
                _ => continue,
            };
            self.position
                .apply(&mv)
                .expect("server move is legal on the mirrored board");
            self.remember();
        }
        terminal(texts)
    }
}

/// Plays the deterministic greedy human through `send` until the server
/// reports a terminal message, returning the result and the move texts sent.
/// The acknowledgement and the results close stay with the caller, whose
/// outcome types differ.
pub fn play_line(mut send: impl FnMut(&str) -> Vec<String>) -> (EndOfGame, Vec<String>) {
    let mut human = Human::new();
    let mut line = Vec::new();
    for _ in 0..400 {
        let movement = human.choose();
        let text = human.move_text(&movement);
        human.position.apply(&movement).unwrap();
        human.remember();
        line.push(text.clone());
        if let Some(end) = human.apply_events(&send(&text)) {
            return (end, line);
        }
    }
    panic!("the seeded game must finish within 400 plies");
}

/// The last terminal message in the server's text stream, if any.
pub fn terminal(texts: &[String]) -> Option<EndOfGame> {
    texts.iter().rev().find_map(|text| match text.as_str() {
        "OnGameWonMsg" => Some(EndOfGame::Won),
        "OnGameLostMsg" => Some(EndOfGame::Lost),
        "OnGameStoppedMsg" => Some(EndOfGame::Stopped),
        _ => None,
    })
}

fn piece_rank(kind: Kind) -> i64 {
    match kind {
        Kind::Queen => 9,
        Kind::Rook => 5,
        Kind::Bishop | Kind::Knight => 3,
        Kind::Pawn => 1,
        Kind::King => 0,
    }
}
