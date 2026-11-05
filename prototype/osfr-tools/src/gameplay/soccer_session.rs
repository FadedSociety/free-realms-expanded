//! Starting a soccer match: the Games window row, a soccer gate, or the row an open quest names (L932).
use super::*;

impl Gameplay {
    /// (R) Begin a portal soccer row: join the authenticated player, fill the
    /// solo roster with passive bots, and offer the shared type-12 run. The
    /// client sees the captured scene entry first; roster and the 39/16 offer
    /// follow after Loaded. Any failure leaves the player untouched.
    pub(super) fn begin_soccer(
        &mut self,
        activity: u32,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let Some(content) = self.soccer_content.clone() else {
            out.replies
                .push(chat::system("Soccer is not available right now."));
            return Ok(());
        };
        // L936: the match switches the player into Soccer Star (client 101319), staged so a
        // refused start changes nothing. L927: the per-match `activity_xp`, only while that job
        // is unlocked (the payout refuses an XP row for a locked job whole).
        let star = crate::soccer_entry::SOCCER_STAR_PROFILE;
        let mut staged = self.clone();
        let mut switch = Outcome::default();
        staged.enter_activity_job(star, character.guid, &mut switch)?;
        let xp = staged
            .profiles
            .as_ref()
            .filter(|p| p.state().unlocked.contains(&star))
            .and_then(|p| p.activity_xp(star));
        // L932: the match credits the open quest objective's row (a Games window row otherwise).
        let open = self
            .npc_quests
            .as_ref()
            .map(crate::quest_engine::QuestEngine::open_finish_minigame_ids)
            .unwrap_or_default();
        let row = content.credited_row(activity, &open);
        match crate::soccer_entry::Session::begin(&content, row, character, xp) {
            Ok(session) => {
                *self = staged;
                out.replies.extend(switch.replies);
                out.nearby.extend(switch.nearby);
                out.scene_zoning = Some(session.scene().clone());
                self.soccer = Some(Box::new(session));
                out.changed = true;
            }
            Err(_) => out
                .replies
                .push(chat::system("You cannot start that activity here.")),
        }
        Ok(())
    }

    /// L932: a soccer gate (`soccer.json` `gates`) starts the row chosen from its menu, or the
    /// row an open quest objective names, the way the portal's Play does.
    pub(super) fn begin_soccer_gate(
        &mut self,
        game: u32,
        character: &CharacterState,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if self.warp_blocker().is_some() {
            let busy = crate::activity_portal::Refusal::Busy.message();
            out.replies.push(chat::system(&busy));
            return Ok(out);
        }
        self.begin_soccer(game, character, &mut out)?;
        Ok(out)
    }
}
