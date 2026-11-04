//! L104: the checkers table's glue for `Gameplay`: Play on the Games window row offers the
//! table, and its lifecycle and movie packets route to it. The rules and the table live in
//! `crate::checkers`.
use super::*;

impl Gameplay {
    /// Play on the checkers row: the 39/16 offer, or one refusal line while a game is open.
    pub(super) fn begin_checkers(&mut self) -> io::Result<Outcome> {
        Ok(match self.checkers.begin()? {
            Some(table) => Outcome {
                changed: table.changed,
                replies: table.replies,
                ..Outcome::default()
            },
            None => Outcome {
                replies: vec![chat::system("Finish your checkers game first.")],
                ..Outcome::default()
            },
        })
    }

    /// One packet while the table is open (`None` otherwise). As for chess, only the lifecycle
    /// result (39/18) credits the minigame, once.
    /// L920: a won game pays its coins on the results page, adopted before the reward packets.
    pub(super) fn handle_checkers(
        &mut self,
        inner: &[u8],
        player_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !self.checkers.is_active() {
            return Ok(None);
        }
        let rewards = self.reward_snapshot();
        let Some(table) = self
            .checkers
            .route_with(inner, now_ms, Some((player_id, &rewards)))?
        else {
            return Ok(None);
        };
        let mut out = Outcome {
            changed: table.changed,
            replies: table.replies.clone(),
            ..Outcome::default()
        };
        if let Some(next) = table.reward_state {
            self.adopt_reward_state(next, &mut out)?;
        }
        out.replies.extend(table.reward_packets);
        if table.replies.iter().any(|p| p.starts_with(&[39, 0, 18])) {
            self.npc_minigame_win(&table.replies, &mut out)?;
        }
        Ok(Some(out))
    }
}
