//! NPC quest glue: catalogue binding, breadcrumbs, eligibility and quest events for NPC interactions.
use super::npc::quest_offer_packet;
use super::*;

impl Gameplay {
    /// Bind the split quest catalogue and its character-owned eligibility.
    ///
    /// Both loads are fallible, so build them before touching live state: a refused
    /// rebind (corrupt content) keeps the bound catalogue and the saved progress.
    /// A live rebind rebuilds from `npc_quest_state` (the bound engine outranks the
    /// saved copy), so it keeps in-flight quests instead of clearing them.
    pub fn bind_npc_quest_catalogue(
        &mut self,
        content_dir: &Path,
        character_id: u64,
    ) -> io::Result<()> {
        let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let source = if content_dir.join("quests").is_dir()
            && content_dir.join("jobs.json").is_file()
            && content_dir.join("npcs").is_dir()
        {
            content_dir
        } else if bundled.join("quests").is_dir()
            && bundled.join("jobs.json").is_file()
            && bundled.join("npcs").is_dir()
        {
            bundled.as_path()
        } else {
            if self.saved_npc_quests.is_some() {
                return Err(invalid("saved NPC quests require split quest content"));
            }
            return Ok(());
        };
        let crosswalk = crate::quest_journal::load_crosswalk(source)?;
        let state = self.npc_quest_state();
        let engine = crate::quest_engine::QuestEngine::from_split_content(
            source,
            character_id,
            crate::quest_engine::CharacterEligibility::default(),
            state,
        )?;
        self.npc_quests = Some(engine);
        self.saved_npc_quests = None;
        self.load_live_events(); // T418: before any Talk, not on the first tick
        self.register_breadcrumb_crosswalk(crosswalk);
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        // Bind is the load-time level-change notification. The login packet sync
        // (`append_npc_quest_login_sync`) rebuilds the 49/3 + completion frames from
        // this committed state, so dropping the staging outcome here loses nothing.
        // The sync also re-arms session-only client-objective registrations
        // (45/6, 45/10, 45/11), whose packets would be dropped with it; keep the
        // session's view of them and let the login/`handle_in_world` sync send them.
        let client_objectives = std::mem::take(&mut self.client_objectives);
        let mut sync = Outcome::default();
        let synced = self.sync_job_level_objectives(&mut sync);
        self.client_objectives = client_objectives;
        synced?;
        Ok(())
    }

    /// The character's quest state source: the bound engine outranks the saved copy
    /// (a reconnect loads the save before the catalogue binds).
    ///
    /// `Gameplay::state` (save serialization) and `bind_npc_quest_catalogue` (a live
    /// rebind) both read it here, so a fallback change cannot make the save and the
    /// engine rebuild from different state.
    pub(super) fn npc_quest_state(&self) -> Option<crate::quest_engine::QuestState> {
        self.npc_quests
            .as_ref()
            .map(|engine| engine.state().clone())
            .or_else(|| self.saved_npc_quests.clone())
    }

    /// Allow `quest_ids` to be started by a 49/15 breadcrumb request. Ids without a quest
    /// definition are dropped (R).
    pub fn register_breadcrumb_quests(&mut self, quest_ids: impl IntoIterator<Item = u32>) {
        let Some(engine) = &self.npc_quests else {
            return;
        };
        self.breadcrumb_quests = quest_ids
            .into_iter()
            .filter_map(|id| Some((id, engine.giver_npc_id(id)?)))
            .collect();
    }

    /// Bind the original-id crosswalk (W175): a 49/15 start may only name an original id it
    /// lists, and only quests with a definition can be started (R).
    pub fn register_breadcrumb_crosswalk(&mut self, crosswalk: crate::quest_journal::Crosswalk) {
        self.register_breadcrumb_quests(crosswalk.values().copied());
        self.breadcrumb_crosswalk = crosswalk;
    }

    /// Replace account rules after account configuration or membership changes.
    pub fn set_game_rules(&mut self, rules: GameRuleSet) {
        self.game_rules = rules;
        self.refresh_npc_quest_eligibility(current_unix_seconds());
    }

    pub(super) fn character_eligibility(
        &self,
        now_unix_seconds: u64,
    ) -> crate::quest_engine::CharacterEligibility {
        let profile_state = self.profiles.as_ref().map(Profiles::state);
        let job_levels = self
            .npc_quests
            .as_ref()
            .into_iter()
            .flat_map(|engine| engine.job_profile_ids())
            .filter_map(|(job_id, profile_id)| {
                let state = profile_state?;
                if profile_id <= 0
                    || profile_id != state.active
                    || !state.unlocked.contains(&profile_id)
                {
                    return None;
                }
                // (R) Reuse Profiles' existing local rank curve: one base rank,
                // one rank per 100 XP, capped at rank 20. Native quest gates do
                // not establish this XP-to-rank mapping.
                let xp = state.experience.get(&profile_id).copied().unwrap_or(0);
                Some((job_id.to_owned(), (1 + xp / 100).min(20) as u8))
            })
            .collect();
        let limits = &self.game_rules.policy.quest_limits;
        crate::quest_engine::CharacterEligibility {
            // (R) No separate avatar level is saved; use the active job's local rank curve.
            player_level: profile_state.map_or(1, |state| self.job_level(state.active) as u8),
            job_levels,
            member: self.game_rules.membership.is_active_at(now_unix_seconds),
            // (R) The client reads both quest-limit values from self data, but the historical
            // numbers are unknown; retain the existing permissive fallback until configured.
            nonmember_quest_limit: limits
                .nonmember
                .map_or(100, |limit| usize::try_from(limit).unwrap_or(usize::MAX)),
            member_quest_limit: limits
                .member
                .map_or(100, |limit| usize::try_from(limit).unwrap_or(usize::MAX)),
        }
    }

    pub(super) fn refresh_npc_quest_eligibility(&mut self, now_unix_seconds: u64) {
        let eligibility = self.character_eligibility(now_unix_seconds);
        if let Some(engine) = &mut self.npc_quests {
            engine.set_eligibility(eligibility);
        }
    }

    /// F576: a payout replaced or raised the character's profiles (an opening reward or
    /// unlock, a minigame payout, a claim). Refresh the quest gates and credit any
    /// job-level goal the new rank meets. Callers run it only when profiles changed,
    /// never per tick or per movement packet.
    pub(super) fn npc_quest_profiles_changed(&mut self, out: &mut Outcome) -> io::Result<()> {
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        self.sync_job_level_objectives(out)
    }

    /// Emit one authoritative `QuestEvent::JobLevel` for every open job-level goal
    /// the character's current rank satisfies. Active quests only; the caller runs
    /// it where the rank can have changed (an XP award, the developer level, an
    /// accept, a turn-in, a payout through `npc_quest_profiles_changed` or a
    /// catalogue bind), never on a steady tick or an unrelated packet: a movement
    /// packet reaches it only when the opening paid a reward or unlocked a job.
    /// Idempotent: a satisfied goal is no longer open, so a repeat emits nothing
    /// and changes nothing.
    pub(super) fn sync_job_level_objectives(&mut self, out: &mut Outcome) -> io::Result<()> {
        let Some(engine) = &self.npc_quests else {
            return Ok(());
        };
        let goals = engine.open_job_level_objectives();
        if goals.is_empty() {
            return Ok(());
        }
        let Some(profiles) = &self.profiles else {
            return Ok(());
        };
        let satisfied: Vec<_> = goals
            .into_iter()
            .filter_map(|(profile_id, level)| {
                // (R) Same 100-XP rank curve as the quest gate and `job_level`.
                let rank = profiles.rank(profile_id) as u8;
                (rank >= level).then_some((profile_id, rank))
            })
            .collect();
        for (profile_id, level) in satisfied {
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::JobLevel { profile_id, level },
                out,
            )?;
        }
        Ok(())
    }

    pub(super) fn append_npc_quest_outcome(
        &self,
        quest: crate::quest_engine::QuestOutcome,
        out: &mut Outcome,
    ) -> io::Result<()> {
        out.changed |= quest.changed;
        for message in quest.outbound {
            match message {
                crate::quest_engine::OutboundMessage::Quest(
                    crate::quest_engine::QuestMessage::QuestInfo { quest_id },
                ) => {
                    let engine = self
                        .npc_quests
                        .as_ref()
                        .ok_or_else(|| invalid("NPC quest engine is unavailable"))?;
                    out.replies.push(
                        quest_offer_packet(engine, quest_id)
                            .map_err(|_| invalid("NPC quest offer could not be encoded"))?,
                    );
                    npc_event(serde_json::json!({
                        "event": "quest_offer",
                        "quest_id": quest_id
                    }));
                }
                crate::quest_engine::OutboundMessage::Quest(
                    crate::quest_engine::QuestMessage::QuestAdd { quest_id },
                ) => {
                    let packet = self
                        .npc_quests
                        .as_ref()
                        .ok_or_else(|| invalid("NPC quest engine is unavailable"))?
                        .quest_add_packet(quest_id)?;
                    out.replies.push(packet);
                }
                crate::quest_engine::OutboundMessage::Quest(
                    crate::quest_engine::QuestMessage::QuestEnd { quest_id, actor_id },
                ) => {
                    let packet = self
                        .npc_quests
                        .as_ref()
                        .ok_or_else(|| invalid("NPC quest engine is unavailable"))?
                        .quest_end_packet(quest_id, actor_id)?;
                    out.replies.push(packet);
                }
                crate::quest_engine::OutboundMessage::Quest(message) => {
                    if let Some(packet) = message.encode_exact() {
                        out.replies.push(packet);
                    }
                }
                crate::quest_engine::OutboundMessage::Reward(packet) => out.replies.push(packet),
            }
        }
        // At most one reward-denial line (T382). `quest.messages` stays unspoken: the client
        // writes its own progress and completion text from 49/9 (0x00a931b0) and 49/4
        // (0x00a928b0), and split titles are only `Quest <id>` placeholders.
        if let Some(line) = quest
            .denial
            .as_ref()
            .and_then(crate::quest_engine::QuestDenial::message)
        {
            out.replies.push(chat::system(line));
        }
        Ok(())
    }

    pub(super) fn npc_quest_event(
        &mut self,
        event: crate::quest_engine::QuestEvent,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(engine) = &mut self.npc_quests {
            let result = engine.on_event(event)?;
            self.append_npc_quest_outcome(result, out)?;
        }
        // Quest events also arrive from movement, ticks and automatic outcomes, not only
        // `handle_in_world`. Synchronize here so a goal opened on any of those paths is
        // registered with the client in the same outcome.
        self.sync_client_objectives(out);
        Ok(())
    }

    pub(super) fn npc_location_event(
        &mut self,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        // (R) Scene entry is an authoritative location event. Area and reach
        // objectives must advance without waiting for the first movement packet.
        self.npc_quest_event(
            crate::quest_engine::QuestEvent::VisitPosition {
                zone_id: character.zone_id,
                position: character.position,
            },
            out,
        )
    }

    // (R) Collect counts newly granted, non-rental items, not login snapshots
    // or existing stock. Equip counts changed owned slots, not packet claims.
    pub(super) fn npc_inventory_events(
        &mut self,
        before: &InventoryState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if self.npc_quests.is_none() {
            return Ok(());
        }
        let totals = |state: &InventoryState| {
            let mut counts = std::collections::BTreeMap::<i32, u64>::new();
            for item in state.items.iter().filter(|item| item.rental.is_none()) {
                *counts.entry(item.definition_id).or_default() += u64::from(item.count);
            }
            counts
        };
        let previous = totals(before);
        for (item_id, total) in totals(self.inventory.state()) {
            let quantity = total
                .saturating_sub(*previous.get(&item_id).unwrap_or(&0))
                .min(1_000_000) as u32;
            if quantity > 0 {
                self.npc_quest_event(
                    crate::quest_engine::QuestEvent::CollectItem { item_id, quantity },
                    out,
                )?;
            }
        }
        let equipped: BTreeSet<_> = self
            .inventory
            .state()
            .equipment
            .iter()
            .filter(|slot| slot.item_id != 0 && !before.equipment.contains(slot))
            .filter_map(|slot| {
                self.inventory
                    .state()
                    .items
                    .iter()
                    .find(|item| item.id == slot.item_id && item.rental.is_none())
                    .map(|item| item.definition_id)
            })
            .collect();
        for definition_id in equipped {
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::EquipItem { definition_id },
                out,
            )?;
        }
        Ok(())
    }

    // Only inspect server-generated results after a successful payout. Never
    // interpret an incoming win/score packet as quest authority (R policy).
    // A settled soccer match is one such result: `tick_world` calls this once
    // per settlement with the server-built replies (`39/14` reader 0x009b89e0;
    // `39/18` `ApplyMiniGameGameOverResult` 0x009ad480, body 0x009b7ec0: shared
    // header, `u8 won`, three `i32`).
    pub(super) fn npc_minigame_win(
        &mut self,
        replies: &[Vec<u8>],
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(packet) = replies.iter().find(|p| {
            (p.len() >= 16 && p.starts_with(&[39, 0, 18]) && p[15] == 1)
                || (p.len() >= 19 && p.starts_with(&[39, 0, 14]) && &p[19..] == b"OnGameWonMsg\0")
        }) {
            let id = i32::from_le_bytes(packet[11..15].try_into().unwrap());
            if id > 0 {
                self.npc_quest_event(
                    crate::quest_engine::QuestEvent::FinishMinigame {
                        minigame_id: id as u32,
                    },
                    out,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn append_npc_quest_login_sync(&mut self, out: &mut Outcome) -> io::Result<()> {
        if self.npc_quests_synced {
            return Ok(());
        }
        let Some(engine) = self.npc_quests.as_ref() else {
            return Ok(());
        };
        let opening_ids = self
            .opening
            .as_ref()
            .map(|opening| {
                opening
                    .journal_entries()
                    .into_iter()
                    .map(|entry| entry.quest_id)
                    .collect()
            })
            .unwrap_or_default();
        out.replies
            .extend(engine.active_login_packets(&opening_ids)?);
        self.npc_quests_synced = true;
        Ok(())
    }
}
