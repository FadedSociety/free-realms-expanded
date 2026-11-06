//! A lobby game of checkers: the one board two clients share. The World owns it and a seat's
//! `CheckersTable` borrows it for one message, as for chess (`hosted_table.rs`).
//!
//! The movie plays two people already (`BoardProxy`): `INIT <me> <red>` names a client's seat and
//! the red seat, and red's client mirrors every network square (`SpotFromNetwork`, 31 - spot) so
//! that it too sees itself at the bottom. So the board is the solo one, seat 0 on squares 20-31,
//! seat 1 on 0-11, and both clients hear the same events in the same numbers. A client ignores
//! `MOVE` on its own turn, having already moved the piece itself.
//!
//! (R) The seat that goes first is the host, and seat 1 is red, as the movie's solo game has it.
use crate::{
    checkers::{self, Board, TURN_END, TURN_START},
    chess::ChessMessage,
};
use std::io;

/// MiniGameData's two-player checkers row (the capture's 102/2 lobby type 5).
pub const HOSTED_ROW: i32 = 68;
const RED: u8 = 1;
/// `_root.mMoveIndicator.WaitingForOtherPlayer`.
const WAITING: u8 = 41;
const INIT: u8 = checkers::INIT;
const MOVE_INVALID: u8 = checkers::MOVE_INVALID;

#[derive(Clone, Debug)]
pub struct HostedGame {
    id: u32,
    board: Board,
    /// Each seat's player and run instance, seat 0 first.
    seats: [(u64, i32); 2],
    ready: [bool; 2],
    turn: usize,
    winner: Option<usize>,
    done: [bool; 2],
}

impl HostedGame {
    pub fn new(id: u32, first: (u64, i32), second: (u64, i32)) -> Self {
        Self {
            id,
            board: Board::default(),
            seats: [first, second],
            ready: [false; 2],
            turn: 0,
            winner: None,
            done: [false; 2],
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    #[cfg(test)]
    pub(crate) fn set_board(&mut self, board: Board) {
        self.board = board;
    }

    fn seat(&self, player: u64) -> Option<usize> {
        self.seats.iter().position(|seat| seat.0 == player)
    }

    pub fn seated(&self, player: u64) -> bool {
        self.seat(player).is_some()
    }

    fn to(&self, seat: usize, text: &str) -> io::Result<ChessMessage> {
        let (player_id, instance) = self.seats[seat];
        Ok(ChessMessage {
            player_id,
            text: text.to_owned(),
            packet: checkers::payload_packet(instance, HOSTED_ROW, text)?,
        })
    }

    fn both(&self, out: &mut Vec<ChessMessage>, text: &str) -> io::Result<()> {
        out.push(self.to(0, text)?);
        out.push(self.to(1, text)?);
        Ok(())
    }

    fn result(&self, out: &mut Vec<ChessMessage>, winner: usize) -> io::Result<()> {
        out.push(self.to(winner, "OnGameWonMsg")?);
        out.push(self.to(1 - winner, "OnGameLostMsg")?);
        Ok(())
    }

    /// `OnReadyMsg`: this seat's `INIT`, and the first turn once both have sat down. A second
    /// Ready changes nothing: unlike the solo game, the board is not this client's alone.
    pub fn ready(&mut self, player: u64) -> io::Result<Vec<ChessMessage>> {
        let mut out = Vec::new();
        let Some(seat) = self.seat(player).filter(|seat| !self.ready[*seat]) else {
            return Ok(out);
        };
        self.ready[seat] = true;
        out.push(self.to(seat, &checkers::message(INIT, &[seat as u8, RED]))?);
        if let Some(winner) = self.winner {
            // The seat that gave up has gone, and may be in another game by now.
            if winner == seat {
                out.push(self.to(seat, "OnGameWonMsg")?);
            }
        } else if self.ready == [true; 2] {
            self.both(&mut out, &checkers::message(TURN_START, &[0]))?;
        } else {
            out.push(self.to(seat, &checkers::message(WAITING, &[]))?);
        }
        Ok(out)
    }

    /// `OnMoveMsg` from the seat whose turn it is; anything else is ignored, and a move the rules
    /// refuse re-opens that seat's turn, as for the solo table.
    pub fn play(&mut self, player: u64, start: u8, path: &[u8]) -> io::Result<Vec<ChessMessage>> {
        let mut out = Vec::new();
        let Some(seat) = self.seat(player).filter(|seat| *seat == self.turn) else {
            return Ok(out);
        };
        if self.winner.is_some() || self.ready != [true; 2] {
            return Ok(out);
        }
        let Ok(events) = self.board.play(seat as u8, start, path) else {
            out.push(self.to(seat, &checkers::message(MOVE_INVALID, &[seat as u8]))?);
            out.push(self.to(seat, &checkers::message(TURN_START, &[seat as u8]))?);
            return Ok(out);
        };
        for event in events {
            self.both(&mut out, &event.text())?;
        }
        self.both(&mut out, &checkers::message(TURN_END, &[seat as u8]))?;
        let next = 1 - seat;
        if self.board.can_move(next as u8) {
            self.turn = next;
            self.both(&mut out, &checkers::message(TURN_START, &[next as u8]))?;
        } else {
            self.winner = Some(seat);
            self.result(&mut out, seat)?;
        }
        Ok(out)
    }

    /// A seat gave up, in play or still sitting down. The other hears a plain win: the movie's
    /// `PLAYERLEFT` would stop it sending the `FRServer_GameOver` that settles it (`OnGameWon`).
    pub fn forfeit(&mut self, player: u64) -> io::Result<Vec<ChessMessage>> {
        let mut out = Vec::new();
        let Some(seat) = self.seat(player).filter(|_| self.winner.is_none()) else {
            return Ok(out);
        };
        self.winner = Some(1 - seat);
        // A client still loading would lose the message, so `ready` repeats it.
        if self.ready[1 - seat] {
            out.push(self.to(1 - seat, "OnGameWonMsg")?);
        }
        Ok(out)
    }

    /// Whether this player won, once the game has a result.
    pub fn won(&self, player: u64) -> Option<bool> {
        let winner = self.winner?;
        self.seat(player).map(|seat| seat == winner)
    }

    pub fn is_settled(&self, player: u64) -> bool {
        self.seat(player).is_some_and(|seat| self.done[seat])
    }

    /// A seat that has settled, or has gone.
    pub fn release(&mut self, player: u64) {
        if let Some(seat) = self.seat(player) {
            self.done[seat] = true;
        }
    }

    pub fn all_settled(&self) -> bool {
        self.done == [true; 2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALPHA: u64 = 11;
    const BETA: u64 = 22;

    fn seated() -> HostedGame {
        let mut game = HostedGame::new(5, (ALPHA, 1), (BETA, 2));
        game.ready(ALPHA).unwrap();
        game.ready(BETA).unwrap();
        game
    }

    fn texts(out: &[ChessMessage], player: u64) -> Vec<String> {
        let mine = out.iter().filter(|message| message.player_id == player);
        mine.map(|message| message.text.clone()).collect()
    }

    #[test]
    fn each_seat_hears_its_own_side_and_the_game_waits_for_both() {
        let mut game = HostedGame::new(5, (ALPHA, 1), (BETA, 2));
        let first = game.ready(ALPHA).unwrap();
        assert_eq!(
            texts(&first, ALPHA),
            [checkers::message(INIT, &[0, RED]), checkers::message(WAITING, &[])]
        );
        assert!(game.play(ALPHA, 21, &[17]).unwrap().is_empty(), "nobody moves alone");
        let second = game.ready(BETA).unwrap();
        assert_eq!(texts(&second, BETA)[0], checkers::message(INIT, &[1, RED]));
        assert_eq!(texts(&second, ALPHA), [checkers::message(TURN_START, &[0])]);
        assert!(game.ready(BETA).unwrap().is_empty(), "a Ready never resets a shared board");
    }

    #[test]
    fn a_move_reaches_both_seats_and_passes_the_turn() {
        let mut game = seated();
        assert!(game.play(BETA, 8, &[12]).unwrap().is_empty(), "not Beta's turn");
        let out = game.play(ALPHA, 21, &[17]).unwrap();
        assert_eq!(texts(&out, ALPHA), texts(&out, BETA));
        assert_eq!(texts(&out, BETA).last().unwrap(), &checkers::message(TURN_START, &[1]));
        assert!(game.play(ALPHA, 22, &[18]).unwrap().is_empty(), "the turn has passed");
        let refused = game.play(BETA, 0, &[31]).unwrap();
        assert_eq!(texts(&refused, BETA)[1], checkers::message(TURN_START, &[1]));
        assert!(texts(&refused, ALPHA).is_empty());
        assert!(!game.play(BETA, 8, &[12]).unwrap().is_empty(), "a refused move leaves the turn");
    }

    #[test]
    fn a_forfeit_is_a_plain_win_and_a_seat_still_loading_hears_it_when_ready() {
        let mut game = seated();
        let out = game.forfeit(ALPHA).unwrap();
        assert_eq!(texts(&out, BETA), ["OnGameWonMsg"]);
        assert_eq!((game.won(BETA), game.won(ALPHA)), (Some(true), Some(false)));
        assert!(game.forfeit(BETA).unwrap().is_empty(), "the game is decided");

        let mut game = HostedGame::new(6, (ALPHA, 1), (BETA, 2));
        game.ready(ALPHA).unwrap();
        assert!(game.forfeit(ALPHA).unwrap().is_empty());
        let out = game.ready(BETA).unwrap();
        assert_eq!(texts(&out, BETA)[1], "OnGameWonMsg");
        assert!(texts(&out, ALPHA).is_empty(), "the seat that left hears nothing");
    }

    #[test]
    fn the_game_ends_when_both_seats_are_done() {
        let mut game = seated();
        game.forfeit(ALPHA).unwrap();
        game.release(ALPHA);
        assert!(!game.all_settled(), "the winner is still owed their result");
        assert!(game.is_settled(ALPHA) && !game.is_settled(BETA));
        game.release(BETA);
        assert!(game.all_settled());
    }
}
