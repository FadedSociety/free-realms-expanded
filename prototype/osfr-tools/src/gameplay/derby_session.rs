//! Derby gameplay integration: job switching, cancellation, reportless close, and session ticking.
use super::*;

/// Predecessor saves recorded the job a connection-local match replaced. Current saves keep the
/// profile itself on that job (`saved_profile_state`), so only an older save carries this marker.
pub(super) fn restore_saved_derby_job(
    saved: &crate::derby_gateway::DerbyGatewayState,
    profiles: &mut Option<Profiles>,
    inventory: &Inventory,
) -> io::Result<()> {
    let Some(job) = saved.previous_job else {
        return Ok(());
    };
    if job <= 0 {
        return Err(invalid("invalid saved derby previous job"));
    }
    if let Some(profiles) = profiles.as_mut() {
        profiles.select(job, inventory)?;
    }
    Ok(())
}

impl Gameplay {
    pub fn derby_phase(&self) -> Option<crate::derby::DerbyPhase> {
        self.derby
            .as_ref()
            .and_then(crate::derby_gateway::DerbyGateway::phase)
    }

    /// Drop the connection-local match on session reset and restore the profile the derby
    /// replaced. The client is gone, so no activation is sent; login self data carries the job.
    pub(super) fn retire_derby_for_reset(&mut self) {
        let Some(job) = self.derby.as_mut().and_then(|derby| derby.exit()) else {
            return;
        };
        if let Some(profiles) = &mut self.profiles {
            let _ = profiles.select(job, &self.inventory);
        }
    }

    pub(super) fn saved_profile_state(&self) -> Option<ProfileState> {
        self.profiles.as_ref().map(|profiles| {
            let mut state = profiles.state().clone();
            // A live derby match is connection-local. Persist the job it replaced so a
            // reconnect at any save boundary cannot restore the temporary Derby Driver.
            if let Some(previous_job) = self
                .derby
                .as_ref()
                .filter(|derby| derby.active())
                .and_then(crate::derby_gateway::DerbyGateway::previous_job)
            {
                state.active = previous_job;
            }
            state
        })
    }

    pub(super) fn tick_derby(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut staged = self.clone();
        let replies = staged.derby.as_mut().unwrap().tick(now_ms)?;
        let previous_job = (!staged
            .derby
            .as_ref()
            .is_some_and(crate::derby_gateway::DerbyGateway::active))
        .then(|| staged.derby.as_mut().unwrap().exit())
        .flatten();
        let mut out = Outcome {
            changed: !replies.is_empty(),
            replies,
            ..Default::default()
        };
        if let Some(job) = previous_job {
            staged.restore_job(job, character.guid, &mut out)?;
            out.replies
                .push(crate::kart_gateway::player_status(character.guid, 1));
            out.changed = true;
            staged.refresh_npc_quest_eligibility(current_unix_seconds());
        }
        *self = staged;
        Ok(out)
    }

    /// (R) Switch to Demo Derby Driver and open the native type-11 offer. The caller has already
    /// checked the NPC gate and that no other activity is running.
    pub(super) fn begin_derby(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let profiles = self
            .profiles
            .as_mut()
            .ok_or_else(|| invalid("derby entry requires class content"))?;
        let previous_job = profiles.active();
        let outfit_before = self.inventory.profile_attachments(previous_job)?;
        let first_time = profiles.unlock(crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE)?;
        profiles.activate(crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE)?;
        if first_time {
            self.give_starter_outfit(
                crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE,
                false,
                &mut out,
            )?;
        }
        self.ensure_outfit(crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE, &mut out)?;
        // Last, after the outfit repair: the 38/21 attachment list comes from what
        // the player wears now (client reader `0x0091b080`).
        let activation = self.profiles.as_ref().unwrap().activation(
            crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE,
            &self.inventory,
        )?;
        out.replies.insert(0, activation);
        self.announce_profile_outfit(
            character.guid,
            &outfit_before,
            previous_job != crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE,
            &mut out,
        )?;
        let derby = self.derby.as_mut().unwrap();
        let offer = derby.begin_at(character, now_ms)?;
        derby.set_previous_job(previous_job);
        out.replies.insert(0, END_DIALOG_26_4.to_vec());
        out.replies.push(offer);
        out.changed = true;
        Ok(out)
    }

    /// Route the native derby records while this session owns a match. The final 26/42 Close
    /// retires the match and restores the job; it grants nothing (F164), and an early, malformed
    /// or replayed Close changes nothing.
    pub(super) fn handle_derby(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if inner == [26, 0, 42, 0] {
            if self.derby.as_ref().and_then(|derby| derby.phase())
                != Some(crate::derby::DerbyPhase::CloseEnabled)
            {
                return self.cancel_derby(inner, character);
            }
            if !self.derby.as_mut().unwrap().close(inner)? {
                return Ok(out);
            }
            let previous_job = self.derby.as_mut().unwrap().exit();
            if let Some(job) = previous_job {
                self.restore_job(job, character.guid, &mut out)?;
            }
            out.replies
                .push(crate::kart_gateway::player_status(character.guid, 1));
            out.changed = true;
            self.refresh_npc_quest_eligibility(current_unix_seconds());
            return Ok(out);
        }
        // Client quit is 39/6; the pending-game cancel at 009bef50 is 39/7.
        if inner.starts_with(&[39, 0, 6]) || inner.starts_with(&[39, 0, 7]) {
            return self.cancel_derby(inner, character);
        }
        if inner.starts_with(&[39, 0, 5]) {
            let before = self.derby.as_ref().unwrap().phase();
            out.replies = self.derby.as_mut().unwrap().start(inner)?;
            out.changed = self.derby.as_ref().unwrap().phase() != before;
            return Ok(out);
        }
        if inner.starts_with(&[34, 0]) {
            out.replies = self.derby.as_mut().unwrap().handle(inner, now_ms)?;
            out.changed = !out.replies.is_empty();
        }
        Ok(out)
    }

    fn cancel_derby(&mut self, inner: &[u8], character: &CharacterState) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let removals = self.derby.as_mut().unwrap().cancel(inner)?;
        if removals.is_empty() {
            return Ok(out);
        }
        let previous_job = self.derby.as_mut().unwrap().exit();
        out.replies = removals;
        if let Some(job) = previous_job {
            self.restore_job(job, character.guid, &mut out)?;
        }
        out.replies
            .push(crate::kart_gateway::player_status(character.guid, 1));
        out.changed = true;
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        Ok(out)
    }
}
