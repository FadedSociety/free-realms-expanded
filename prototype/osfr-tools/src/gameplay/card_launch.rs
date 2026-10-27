//! T765: Trading Card Game launch glue for `Gameplay`; the run and ticket rules live in
//! `crate::card_game::Launches`. Play on an admitted card row offers a type-16 run (39/16, naming
//! the card asset list); once the client has loaded its card assets its card manager asks for the
//! launch ticket (39/37) and gets it in 39/0x28, which enables the Start button; the click
//! (39/5) is answered with 39/17 (T795: the client's order). Leaving the card window (39/6, then
//! its 39/7 ack) ends the run, and the generic 26/42 Close sent just before is not the run's
//! (T782). Session-only: nothing here marks a save.
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

    /// The lifecycle packets of the card run (live, or just ended for its 39/7 ack); `None`
    /// leaves the packet to later owners.
    pub(super) fn handle_card_game(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(launches) = self.card_launches.as_mut().filter(|l| l.owns(inner)) else {
            return Ok(None);
        };
        let run = launches.route(inner, character.guid, now_ms)?;
        Ok(run.accepted.then(|| Outcome {
            replies: run.replies,
            ..Outcome::default()
        }))
    }

    /// T798: the login's session-data packet for the card window; `None` without a launch block.
    pub fn card_session_data(&self) -> io::Result<Option<Vec<u8>>> {
        (self.card_launches.is_some())
            .then(crate::card_game::session_data_packet)
            .transpose()
    }

    /// T766's card listener: redeem a card login's launch ticket and character id in the
    /// session that minted it. `Ok(character)` once; never again for that ticket. A session
    /// without a launch block holds no ticket.
    pub fn redeem_card_ticket(
        &mut self,
        ticket: &str,
        character: &str,
        now_ms: u64,
    ) -> Result<u64, &'static str> {
        self.card_launches
            .as_mut()
            .ok_or(crate::card_service::UNKNOWN_TICKET)?
            .redeem(ticket, character, now_ms)
    }
}
