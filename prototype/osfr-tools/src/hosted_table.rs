//! A running lobby game's table as the World holds it: a seat's message borrows it for its own
//! transaction, and it comes back after the commit (`bin/osfr_gateway/room_routing.rs`).
use crate::{
    checkers_lobby::HostedGame,
    chess::{ChessMessage, ChessTable},
    derby_gateway::HostedDerby,
};
use std::io;

/// Lobby type 5, the capture's 102/2 checkers row; any other is chess.
pub const CHECKERS_LOBBY: u32 = 5;

#[derive(Clone, Debug)]
pub enum HostedTable {
    Chess(ChessTable),
    Checkers(HostedGame),
    /// A derby room's match: several drivers, and its own clock rather than turns.
    Derby(Box<HostedDerby>),
}

/// Addressed packets as the relay carries them.
pub fn messages(packets: Vec<(u64, Vec<u8>)>) -> Vec<ChessMessage> {
    let message = |(player_id, packet): (u64, Vec<u8>)| ChessMessage { player_id, text: String::new(), packet };
    packets.into_iter().map(message).collect()
}

impl HostedTable {
    pub fn id(&self) -> u32 {
        match self {
            Self::Chess(table) => table.id() as u32,
            Self::Checkers(game) => game.id(),
            Self::Derby(derby) => derby.id(),
        }
    }

    pub fn seated(&self, guid: u64) -> bool {
        match self {
            Self::Chess(table) => table.seated(guid),
            Self::Checkers(game) => game.seated(guid),
            Self::Derby(derby) => derby.seated(guid),
        }
    }

    /// Gives up `guid`'s seat: what the other seat is owed.
    pub fn forfeit(&mut self, guid: u64) -> io::Result<Vec<ChessMessage>> {
        match self {
            Self::Chess(table) => Ok(table.forfeit(guid)?.messages),
            Self::Checkers(game) => game.forfeit(guid),
            Self::Derby(derby) => derby.forfeit(guid).map(messages),
        }
    }

    /// The seat the game is waiting on, while it has no result.
    pub fn waiting_on(&self) -> Option<u64> {
        match self {
            Self::Chess(table) => table.waiting_on(),
            Self::Checkers(game) => game.waiting_on(),
            // A derby waits on nobody's turn; its start has its own watch.
            Self::Derby(_) => None,
        }
    }

    /// Once a tick: what a game with its own clock owes its seats.
    pub fn watch(&mut self, now_ms: u64) -> io::Result<Vec<ChessMessage>> {
        match self {
            Self::Derby(derby) => derby.watch(now_ms).map(messages),
            _ => Ok(Vec::new()),
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
            Self::Derby(derby) => derby.release(guid),
        }
    }

    pub fn is_settled(&self, guid: u64) -> bool {
        match self {
            Self::Chess(table) => table.is_settled(guid),
            Self::Checkers(game) => game.is_settled(guid),
            Self::Derby(derby) => derby.is_settled(guid),
        }
    }

    pub fn all_settled(&self) -> bool {
        match self {
            Self::Chess(table) => table.all_settled(),
            Self::Checkers(game) => game.all_settled(),
            Self::Derby(derby) => derby.all_settled(),
        }
    }
}
