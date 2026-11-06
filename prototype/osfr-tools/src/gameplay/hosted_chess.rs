//! A lobby game of chess, as one player's `Gameplay` sees it: the offer, and the World's table lent
//! around one message. The rules are `crate::chess`; `bin/osfr_gateway/room_routing.rs` owns the table.
use super::*;
use crate::chess::{ChessMessage, ChessParticipant, ChessTable};

impl Gameplay {
    /// The lobby game's offer for this player; `None` when they cannot take it, with a line why.
    pub(crate) fn begin_hosted_chess(
        &mut self,
        player_id: u64,
        table_id: u64,
        now_ms: u64,
    ) -> io::Result<(Outcome, Option<i32>)> {
        let mut out = Outcome::default();
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
    pub(crate) fn can_host_chess(&self) -> bool {
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

    pub(crate) fn hosted_chess_table(&self) -> Option<u64> {
        self.chess.as_ref()?.hosted_table()
    }

    pub(crate) fn lend_chess_table(&mut self, table: ChessTable) {
        if let Some(chess) = self.chess.as_mut() {
            chess.lend(table);
        }
    }

    pub(crate) fn reclaim_chess_table(&mut self) -> Option<(ChessTable, Vec<ChessMessage>)> {
        self.chess.as_mut()?.reclaim()
    }
}
