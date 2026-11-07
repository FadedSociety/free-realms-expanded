//! A running lobby game's table as the World holds it: a seat's message borrows it for its own
//! transaction, and it comes back after the commit (`bin/osfr_gateway/room_routing.rs`).
use crate::{
    checkers_lobby::HostedGame,
    chess::{ChessMessage, ChessTable},
};
use std::io;

/// Lobby type 5, the capture's 102/2 checkers row; any other is chess.
pub const CHECKERS_LOBBY: u32 = 5;

#[derive(Clone, Debug)]
pub enum HostedTable {
    Chess(ChessTable),
    Checkers(HostedGame),
}

impl HostedTable {
    pub fn id(&self) -> u32 {
        match self {
            Self::Chess(table) => table.id() as u32,
            Self::Checkers(game) => game.id(),
        }
    }

    pub fn seated(&self, guid: u64) -> bool {
        match self {
            Self::Chess(table) => table.seated(guid),
            Self::Checkers(game) => game.seated(guid),
        }
    }

    /// Gives up `guid`'s seat: what the other seat is owed.
    pub fn forfeit(&mut self, guid: u64) -> io::Result<Vec<ChessMessage>> {
        match self {
            Self::Chess(table) => Ok(table.forfeit(guid)?.messages),
            Self::Checkers(game) => game.forfeit(guid),
        }
    }

    /// The seat the game is waiting on, while it has no result.
    pub fn waiting_on(&self) -> Option<u64> {
        match self {
            Self::Chess(table) => table.waiting_on(),
            Self::Checkers(game) => game.waiting_on(),
        }
    }

    /// The seat it waits on took too long: it loses, and both seats hear. Each client's own
    /// `FRServer_GameOver` settles it; one that has gone is released when its session ends.
    pub fn time_out(&mut self) -> io::Result<Vec<ChessMessage>> {
        let Some(guid) = self.waiting_on() else {
            return Ok(Vec::new());
        };
        let mut messages = self.forfeit(guid)?;
        if let Self::Checkers(game) = self {
            messages.extend(game.timed_out(guid)?);
        }
        Ok(messages)
    }

    pub fn release(&mut self, guid: u64) {
        match self {
            Self::Chess(table) => table.release(guid),
            Self::Checkers(game) => game.release(guid),
        }
    }

    pub fn is_settled(&self, guid: u64) -> bool {
        match self {
            Self::Chess(table) => table.is_settled(guid),
            Self::Checkers(game) => game.is_settled(guid),
        }
    }

    pub fn all_settled(&self) -> bool {
        match self {
            Self::Chess(table) => table.all_settled(),
            Self::Checkers(game) => game.all_settled(),
        }
    }
}
