//! A lobby game, as one player's `Gameplay` sees it: the offer, and the World's table lent around
//! one message. The rules are `crate::chess` and `crate::checkers_lobby`;
//! `bin/osfr_gateway/room_routing.rs` owns the table.
use super::*;
use crate::chess::{ChessMessage, ChessParticipant, ChessTable};
use crate::hosted_table::{CHECKERS_LOBBY, HostedTable};

impl Gameplay {
    /// The lobby game's offer for this player; `None` when they cannot take it, with a line why.
    pub(crate) fn begin_hosted(
        &mut self,
        lobby_type: u32,
        player_id: u64,
        table_id: u64,
        now_ms: u64,
    ) -> io::Result<(Outcome, Option<i32>)> {
        let mut out = Outcome::default();
        if lobby_type == CHECKERS_LOBBY {
            let Some((table, instance)) = self.checkers.begin_hosted(table_id, now_ms)? else {
                return Ok((out, None));
            };
            out.changed = table.changed;
            out.replies = table.replies;
            return Ok((out, Some(instance)));
        }
        let Some(chess) = self.chess.as_mut() else {
            return Ok((out, None));
        };
        match chess.begin_hosted(player_id, table_id, now_ms) {
            Ok((change, instance)) => {
                self.apply_chess(change, &mut out)?;
                Ok((out, Some(instance)))
            }
            Err(denied) => {
                out.replies.push(chat::system(denied.message()));
                Ok((out, None))
            }
        }
    }

    /// Free to sit down when a countdown ends.
    pub(crate) fn can_host(&self) -> bool {
        self.chess.is_some() && self.warp_blocker().is_none()
    }

    pub(crate) fn settle_hosted_chess(
        &mut self,
        player_id: u64,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let rewards = self.reward_snapshot();
        let Some(chess) = self.chess.as_mut() else {
            return Ok(out);
        };
        let change = chess.settle_hosted(player_id, now_ms, &rewards)?;
        self.apply_chess(change, &mut out)?;
        Ok(out)
    }

    pub(crate) fn open_hosted_chess(
        &mut self,
        table_id: u64,
        white: ChessParticipant,
        black: ChessParticipant,
    ) -> io::Result<ChessTable> {
        let chess = self.chess.as_mut().ok_or_else(|| invalid("chess is not loaded"))?;
        chess.open_hosted(table_id, white, black)
    }

    pub(crate) fn hosted_table(&self) -> Option<u64> {
        let chess = self.chess.as_ref().and_then(ChessService::hosted_table);
        let derby = || self.derby.as_ref()?.hosted_room().map(u64::from);
        chess.or(self.checkers.hosted_table()).or_else(derby)
    }

    /// The World no longer holds the game this player sits in.
    pub(crate) fn orphan_table(&mut self) {
        if let Some(derby) = self.derby.as_mut() {
            derby.orphan();
        }
    }

    pub(crate) fn lend_table(&mut self, table: HostedTable) {
        match table {
            HostedTable::Chess(table) => {
                if let Some(chess) = self.chess.as_mut() {
                    chess.lend(table);
                }
            }
            HostedTable::Checkers(game) => self.checkers.lend(game),
            HostedTable::Derby(derby) => {
                if let Some(mine) = self.derby.as_mut() {
                    mine.lend(*derby);
                }
            }
        }
    }

    pub(crate) fn reclaim_table(&mut self) -> Option<(HostedTable, Vec<ChessMessage>)> {
        let chess = self.chess.as_mut().and_then(ChessService::reclaim);
        let chess = chess.map(|(table, relay)| (HostedTable::Chess(table), relay));
        let checkers = self.checkers.reclaim();
        let derby = || {
            let (table, relay) = self.derby.as_mut()?.reclaim()?;
            Some((HostedTable::Derby(Box::new(table)), crate::hosted_table::messages(relay)))
        };
        let board = chess.or(checkers.map(|(game, relay)| (HostedTable::Checkers(game), relay)));
        board.or_else(derby)
    }
}
