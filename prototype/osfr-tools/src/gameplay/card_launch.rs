//! T765: Trading Card Game launch glue for `Gameplay`; the run and ticket rules live in
//! `crate::card_game::Launches`. Play on an admitted card row offers a type-16 run (39/16, naming
//! the card asset list); once the client has loaded its card assets its card manager asks for the
//! launch ticket (39/37) and gets it in 39/0x28, which enables the Start button; the click
//! (39/5) is answered with 39/17 (T795: the client's order). Leaving the card window (39/6, then
//! its 39/7 ack) ends the run, and the generic 26/42 Close sent just before is not the run's
//! (T782). Session-only: nothing here marks a save. L935: the card window plays its games itself
//! and reports them (`card_single_player`): a bounded win credits the launched row and pays the
//! duel purse, once.
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

    /// (R) An NPC's deck, picked from its menu: the same run as a Games-window row, refused
    /// while the player is busy with something else.
    pub(super) fn begin_npc_card_duel(&mut self, activity: u32) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if self.warp_blocker().is_some() {
            out.replies.push(chat::system(&crate::activity_portal::Refusal::Busy.message()));
            return Ok(out);
        }
        self.begin_card_game(activity, &mut out)?;
        Ok(out)
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

    /// L935: the card window's single-player record for `character` (the session's own). A win
    /// that passes `Launches::record` credits the launched row to the quest engine and pays the
    /// duel purse (coins and the Card Duelist's `activity_xp`) through the shared reward applier;
    /// everything else only moves the tracker. `Some` carries the win and what to persist and
    /// send; the card start can never pay twice (see `Launches::record`).
    pub fn card_single_player(
        &mut self,
        character: u64,
        record: &crate::card_service::SinglePlayerRecord,
        now_ms: u64,
    ) -> io::Result<Option<(crate::card_game::Win, Outcome)>> {
        let Some(win) = self
            .card_launches
            .as_mut()
            .and_then(|launches| launches.record(record, now_ms))
        else {
            return Ok(None);
        };
        let mut out = Outcome {
            changed: true,
            ..Outcome::default()
        };
        // Only the server-measured time is a fact; the DLL's own score is not certified.
        let result = crate::quest_engine::MinigameResult {
            won: true,
            elapsed_seconds: Some(win.elapsed_seconds),
            score: None,
            placement: None,
            knocked_out: false,
        };
        self.credit_minigame(win.minigame_id, result, &mut out)?;
        self.pay_card_win(character, &win, &mut out)?;
        Ok(Some((win, out)))
    }

    /// The duel purse: `win.coins` and the Card Duelist's `activity_xp`, the XP only while that
    /// job is unlocked (the applier refuses an XP row for a locked job whole, and the coins
    /// should still pay). A refusal is a player-local chat line; the credit stays committed.
    fn pay_card_win(
        &mut self,
        character: u64,
        win: &crate::card_game::Win,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let profile = crate::card_game::CARD_DUELIST_PROFILE;
        let xp = self
            .profiles
            .as_ref()
            .filter(|profiles| profiles.state().unlocked.contains(&profile))
            .and_then(|profiles| profiles.activity_xp(profile))
            .filter(|xp| *xp > 0);
        if win.coins == 0 && xp.is_none() {
            return Ok(());
        }
        let grant = crate::rewards::RewardGrant {
            coins: win.coins,
            items: Vec::new(),
            job_xp: xp.map(|xp| (profile, xp)).into_iter().collect(),
            sc: 0,
            display: crate::rewards::RewardDisplay {
                source_guid: character,
                player_guid: character,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        };
        let key = format!("card-duel-{}", win.started);
        match crate::rewards::apply_grant(&self.reward_snapshot(), &grant, &key) {
            Ok((next, replies)) => {
                out.replies.extend(replies);
                self.adopt_reward_state(next, out)
            }
            Err(error) => {
                eprintln!("card duel payout {key} refused: {error}");
                out.replies
                    .push(chat::system("The duel prize could not be paid right now."));
                Ok(())
            }
        }
    }

    /// T766's card listener: redeem a card login's launch ticket in the session that minted it
    /// (a character id the 411 carries, if any, must name the owner). `Ok(character)` once, never
    /// again for that ticket; `None` when this session holds no such ticket, so the listener
    /// moves on to the next live session. A session without a launch block holds no ticket.
    pub fn redeem_card_ticket(
        &mut self,
        ticket: &str,
        character: &str,
        now_ms: u64,
    ) -> Option<Result<u64, &'static str>> {
        self.card_launches
            .as_mut()?
            .redeem(ticket, character, now_ms)
    }

    /// The client scenario of the card row in play, for the card login's challenge.
    pub fn card_scenario(&self) -> Option<i32> {
        self.card_launches.as_ref()?.scenario()
    }
}
