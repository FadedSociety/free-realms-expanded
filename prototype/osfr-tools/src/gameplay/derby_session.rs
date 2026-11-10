//! Derby gameplay integration: job switching, completion payout, cancellation and session ticking.
use super::*;

const NO_CLASS_CONTENT: &str = "derby entry requires class content";
/// (R) The results line when a goal row's finished match misses its goal: paid, not credited.
pub const DERBY_GOAL_MISSED: &str =
    "You didn't meet this derby's goal. The match still paid; try it again for the quest.";

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
    /// L931: where a car of the live derby is now (its last pose).
    pub fn derby_driver_position(&self, key: crate::derby::ParticipantKey) -> Option<[f32; 3]> {
        self.derby.as_ref()?.driver_position(key)
    }

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
        // Self data at login carries the job, so the activation reply is not needed.
        if let Some(profiles) = &mut self.profiles
            && let Err(error) = profiles.activate(job)
        {
            eprintln!("derby: could not restore job {job} on reset ({error})");
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
        // A lobby room's match: its start records, or a seat lost from under this driver.
        if let Some(out) = staged.hosted_derby_step(character)? {
            *self = staged;
            return Ok(out);
        }
        let Some(derby) = staged.derby.as_mut() else {
            return Ok(Outcome::default());
        };
        let replies = derby.tick(now_ms)?;
        let ended = derby.phase().is_some() && !derby.active();
        let previous_job = if derby.active() { None } else { derby.exit() };
        // L931: the driven NPCs' 34/5 poses are ephemeral; they must not force a save every tick.
        let mut out = Outcome {
            changed: replies.iter().any(|reply| !reply.starts_with(&[34, 0, 5])),
            replies,
            ..Default::default()
        };
        staged.pay_derby_completion(character, &mut out)?;
        if ended {
            staged.leave_derby_arena(&mut out)?;
        }
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

    /// (R) Switch to Demo Derby Driver and open the native type-11 offer for `row` (the gameplay
    /// selection, `None` for the default package). The caller has already checked the NPC gate and
    /// that no other activity is running.
    pub(super) fn begin_derby(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
        row: Option<i32>,
    ) -> io::Result<Outcome> {
        let mut out = self.derby_job(character)?;
        // Both callers stage this call, so a refusal here discards the job switch above.
        let derby = self
            .derby
            .as_mut()
            .ok_or_else(|| invalid("derby entry requires the derby service"))?;
        // L936: the player stays a Demo Derby Driver after the match (client 101319).
        // The garage's chosen derby car rides in (`vehicle_looks.json`); none keeps the package's.
        derby.set_player_vehicle(self.kart_entry.as_ref().and_then(|kart| kart.chosen_look(2)));
        let offer = derby.begin_row(character, now_ms, row)?;
        let mut start = vec![offer];
        start.extend(derby.ready_packet());
        // L931: a row with an arena zones the player in; the offer waits for the arena's Loaded.
        match derby.scene().cloned() {
            Some(scene) => {
                derby.enter_arena(character.clone(), start);
                out.scene_zoning = Some(scene);
            }
            None => out.replies.extend(start),
        }
        out.changed = true;
        Ok(out)
    }

    /// The switch to Demo Derby Driver a match starts with: the job, its outfit and the
    /// activation, for a solo offer and a lobby room's start alike.
    pub(super) fn derby_job(&mut self, character: &CharacterState) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let profiles = self
            .profiles
            .as_mut()
            .ok_or_else(|| invalid(NO_CLASS_CONTENT))?;
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
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        // Last, after the outfit repair: the 38/21 attachment list comes from what
        // the player wears now (client reader `0x0091b080`).
        let activation = self
            .profiles
            .as_ref()
            .ok_or_else(|| invalid(NO_CLASS_CONTENT))?
            .activation(
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
        // The caller routes here only while `derby_active()`.
        let Some(derby) = self.derby.as_mut() else {
            return Ok(out);
        };
        if inner == [26, 0, 42, 0] {
            if derby.phase() != Some(crate::derby::DerbyPhase::CloseEnabled) {
                return self.cancel_derby(inner, character);
            }
            if !derby.close(inner)? {
                return Ok(out);
            }
            let previous_job = derby.exit();
            self.leave_activity_job(previous_job, character.guid, &mut out)?;
            out.replies
                .push(crate::kart_gateway::player_status(character.guid, 1));
            self.leave_derby_arena(&mut out)?;
            out.changed = true;
            self.refresh_npc_quest_eligibility(current_unix_seconds());
            return Ok(out);
        }
        // L958: a quit in the race ends it through the results screen, unpaid; its Close returns
        // the player as at a race end.
        if let Some(replies) = derby.forfeit(inner, now_ms)? {
            out.replies = replies;
            out.changed = true;
            return Ok(out);
        }
        // Client quit is 39/6; the pending-game cancel at 009bef50 is 39/7.
        if inner.starts_with(&[39, 0, 6]) || inner.starts_with(&[39, 0, 7]) {
            return self.cancel_derby(inner, character);
        }
        if inner.starts_with(&[39, 0, 5]) {
            let before = derby.phase();
            out.replies = derby.start(inner)?;
            out.changed = derby.phase() != before;
            return Ok(out);
        }
        if inner.starts_with(&[34, 0]) {
            out.replies = derby.handle(inner, now_ms)?;
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
    /// server-observed `FinishMinigame` credit for the entered row's game id (`DerbyGateway::game_id`,
    /// the MiniGameData row the 39/16 offer names) whether the flat payout fits, is refused or is
    /// tuned to zero. A payout the wallet or job progress cannot represent is a player-local
    /// refusal: one system line, the consumed ledger and the quest credit stay committed, and the
    /// session keeps running (never an `io::Error` that ends it).
    fn pay_derby_completion(
        &mut self,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        // L927: the Demo Derby Driver's `activity_xp` per match (`job_xp_curve.json`).
        let rate = self.profiles.as_ref().and_then(|profiles| {
            profiles.activity_xp(crate::derby_gateway::DEMO_DERBY_DRIVER_PROFILE)
        });
        let Some(derby) = self.derby.as_mut() else {
            return Ok(());
        };
        let Some(instance) = derby.take_completion() else {
            return Ok(());
        };
        // The ledger row is taken: persist it with the quest credit, whatever the payout does.
        out.changed = true;
        let (minigame_id, grant) = (
            derby.game_id(),
            derby.completion_grant(character.guid, rate),
        );
        // T787: this participant's own score, placement and knock-outs, never another
        // driver's. A missing result (a phase race) claims no fact, so no bound the
        // producer can report is met by accident; `knocked_out` stays conservative.
        let facts = derby.completion_result(character.guid);
        // A missing result claims no fact; a goal row with no facts cannot have met its goal.
        let goal_free = !derby.has_goal();
        let result = facts.map_or_else(
            || crate::quest_engine::MinigameResult {
                won: goal_free,
                elapsed_seconds: None,
                score: None,
                placement: None,
                // No result row means no observed knock-out run; claiming none would
                // let a `no_knockout` bound credit by accident.
                knocked_out: true,
            },
            crate::derby::DerbyParticipantResult::quest_result,
        );
        // (R) A row with a goal (knock-outs, named drivers, power-ups, placement) credits its
        // quest only when the goal was met; the match still pays its completion. Free play and
        // the tutorials have no goal, so they credit on finishing as before.
        if result.won {
            self.credit_minigame(minigame_id, result, out)?;
        } else {
            out.replies.push(chat::system(DERBY_GOAL_MISSED));
        }
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
        let Some(derby) = self.derby.as_mut() else {
            return Ok(out);
        };
        let removals = derby.cancel(inner)?;
        if removals.is_empty() {
            return Ok(out);
        }
        let previous_job = derby.exit();
        out.replies = removals;
        self.leave_activity_job(previous_job, character.guid, &mut out)?;
        out.replies
            .push(crate::kart_gateway::player_status(character.guid, 1));
        self.leave_derby_arena(&mut out)?;
        out.changed = true;
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        Ok(out)
    }

    /// L931: every way out of an arena match zones the player back to the pose kept at entry,
    /// once (the visit is consumed here).
    pub(super) fn leave_derby_arena(&mut self, out: &mut Outcome) -> io::Result<()> {
        // A lobby room's driver: the client's lobby drops the game before the way back.
        let lobby = self.derby.as_mut().and_then(|derby| derby.take_lobby_exit());
        out.replies.extend(lobby);
        let back = self
            .derby
            .as_mut()
            .and_then(crate::derby_gateway::DerbyGateway::leave_arena);
        if let Some(back) = back {
            out.scene_zoning = Some(self.world_zoning(&back)?);
        }
        Ok(())
    }

    /// L931: the arena scene while a match visit lasts (`current_zone`, `supports_location`).
    pub(super) fn derby_arena_scene(&self) -> Option<&crate::zone_transition::ZoneTransition> {
        self.derby.as_ref()?.arena_scene()
    }

    /// L931: the arena's held start records after its Loaded (`initialize_world`).
    pub(super) fn derby_arena_loaded(&mut self) -> Option<Vec<Vec<u8>>> {
        self.derby.as_mut()?.arena_loaded()
    }
}
