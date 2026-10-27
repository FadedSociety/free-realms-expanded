//! T765: Trading Card Game launch glue for `Gameplay`; the run and ticket rules live in
//! `crate::card_game::Launches`. Play on an admitted card row offers a type-16 run (39/16); the
//! client starts it (39/5 -> 39/17), its card manager asks for the launch ticket (39/37) and
//! gets it in 39/0x28; leaving the card window (39/6) ends the run. Session-only: nothing here
//! marks a save.
use super::*;

impl Gameplay {
    /// Play on an admitted Trading Card Game row: offer its run. `admit` already refused busy
    /// players and rows without a launch block.
    pub(super) fn begin_card_game(&mut self, activity: u32, out: &mut Outcome) -> io::Result<()> {
        let Some(cards) = self.activities.cards().cloned() else {
            return Ok(());
        };
        let (Some(row), Some(launches)) = (cards.row(activity), self.card_launches.as_mut()) else {
            return Ok(());
        };
        out.replies.extend(launches.begin(row)?.replies);
        Ok(())
    }

    /// The lifecycle packets of an active card run; `None` leaves the packet to later owners.
    pub(super) fn handle_card_game(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(launches) = self.card_launches.as_mut().filter(|l| l.is_active()) else {
            return Ok(None);
        };
        let run = launches.route(inner, character.guid, now_ms)?;
        Ok(run.accepted.then(|| Outcome {
            replies: run.replies,
            ..Outcome::default()
        }))
    }

    /// T766's card listener: redeem a card login's launch ticket and character id in the
    /// session that minted it. `Some(character)` once; never again for that ticket.
    pub fn redeem_card_ticket(
        &mut self,
        ticket: &str,
        character: &str,
        now_ms: u64,
    ) -> Option<u64> {
        self.card_launches
            .as_mut()?
            .redeem(ticket, character, now_ms)
    }
}
