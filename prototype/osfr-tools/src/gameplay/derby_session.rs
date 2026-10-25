//! Derby gameplay integration: job switching, completion payout, cancellation and session ticking.
use super::*;
use crate::derby_gateway::DerbyGateway;

/// Predecessor saves recorded the job a connection-local match replaced. Current saves keep the
/// profile itself on that job (`saved_profile_state`), so only an older save carries this marker.
/// A marker the current profile content rejects is logged and skipped: a stale legacy field must
/// not stop the character from loading (F206). The saved job stays active in that case (a T023
/// save therefore keeps the temporary Derby Driver); the player's other unlocked jobs stay
/// selectable through the ordinary 26/13 switch.
pub(super) fn restore_saved_derby_job(
    saved: &crate::derby_gateway::DerbyGatewayState,
    profiles: &mut Option<Profiles>,
) {
    let Some(job) = saved.previous_job else {
        return;
    };
    if job <= 0 {
        eprintln!("derby: ignoring invalid saved previous job {job}");
        return;
    }
    if let Some(profiles) = profiles.as_mut()
        && let Err(error) = profiles.activate(job)
    {
        eprintln!(
            "derby: previous job {job} is unavailable ({error}); keeping saved job {}",
            profiles.active()
        );
    }
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
            // Self data at login carries the job, so the activation reply is not needed.
            let _ = profiles.activate(job);
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
        staged.pay_derby_completion(character, &mut out)?;
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
        out.replies.push(offer);
        out.changed = true;
        Ok(out)
    }

    /// Route the native derby records while this session owns a match. The results phase pays the
    /// flat completion reward (once per match instance); the final 26/42 Close only retires the
    /// match and restores the job. An early, malformed or replayed record changes nothing.
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
            self.pay_derby_completion(character, &mut out)?;
        }
        // T528: a saddle or chest check/use mid-match is refused with a line, not dropped (the
        // soccer route's rule); the derby owns the rest of 42/*.
        if crate::item_bar::use_wire::decode_immediate_activation(inner).is_some() {
            out.replies
                .push(chat::system("Finish the derby before using equipment."));
        }
        Ok(out)
    }

    /// Pay the flat completion reward when this session's match reaches the results phase.
    ///
    /// The trigger is the server-observed lifecycle (`DerbyPhase::Results`), never a collision, a
    /// score or a client-supplied result. `DerbyGateway::take_completion` records the match
    /// instance in the ledger as it hands it over, so a repeated record or a reconnect cannot pay
    /// the same match twice. The grant is staged with `rewards::apply_grant` over
    /// `reward_snapshot()` and adopted by `adopt_reward_state` (the shared minigame reward path).
    ///
    /// The same once-per-instance handover emits the quest event: a finished derby is a
    /// server-observed `FinishMinigame` credit for the package's game id (`DerbyGateway::game_id`,
    /// the MiniGameData row the 39/16 offer names) whether the flat payout fits, is refused or is
    /// tuned to zero. A payout the wallet or job progress cannot represent is a player-local
    /// refusal: one system line, the consumed ledger and the quest credit stay committed, and the
    /// session keeps running (never an `io::Error` that ends it).
    fn pay_derby_completion(
        &mut self,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let Some(instance) = self.derby.as_mut().and_then(DerbyGateway::take_completion) else {
            return Ok(());
        };
        // The ledger row is taken: persist it with the quest credit, whatever the payout does.
        out.changed = true;
        let derby = self.derby.as_ref().unwrap();
        let (minigame_id, grant) = (derby.game_id(), derby.completion_grant(character.guid));
        self.npc_quest_event(
            crate::quest_engine::QuestEvent::FinishMinigame { minigame_id },
            out,
        )?;
        // T229: the same handover pays the derby's Station Cash kind, whatever the coin grant does.
        self.pay_earned_station_cash(crate::station_cash::EarnSource::DerbyMatch, out)?;
        let Some(grant) = grant else {
            return Ok(());
        };
        let rewards = self.reward_snapshot();
        let key = format!("derby-completion-{instance}");
        match crate::rewards::apply_grant(&rewards, &grant, &key) {
            Ok((next, replies)) => {
                out.replies.extend(replies);
                self.adopt_reward_state(next, out)
            }
            Err(error) => {
                eprintln!("derby completion payout for instance {instance} refused: {error}");
                out.replies
                    .push(chat::system(super::encounter_rewards::DERBY_PAYOUT_REFUSED));
                Ok(())
            }
        }
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
