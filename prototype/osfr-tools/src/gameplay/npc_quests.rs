//! NPC quest glue: catalogue binding, breadcrumbs, eligibility and quest events for NPC interactions.
use super::npc::quest_offer_packet;
use super::*;
use crate::quest_markers::{encode_add_notifications, encode_remove_notifications, marker_for};

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
            // T852: no split quest content. The saved regional progress stays in
            // `saved_npc_quests` (which `npc_quest_state` writes back), so a smaller content
            // set keeps the journal instead of refusing the login.
            content_presence::record_skip(&mut self.presence, "regional_quests", "quests/");
            return Ok(());
        };
        let (mut crosswalk, listed) = crate::quest_journal::read_crosswalk(source)?;
        let job_browser = crate::job_browser::load(source, &listed)?;
        crosswalk.extend(job_browser.crosswalk);
        let markers = source.join(crate::quest_markers::CONTENT_FILE);
        let markers = markers
            .is_file()
            .then(|| crate::quest_markers::QuestMarkerContent::load(&markers))
            .transpose()?;
        let state = self.npc_quest_state();
        let mut engine = crate::quest_engine::QuestEngine::from_split_content(
            source,
            character_id,
            crate::quest_engine::CharacterEligibility::default(),
            state,
        )?;
        engine.continue_revision(self.quest_marker_revision());
        self.quest_markers.content = markers.map(Arc::new);
        self.npc_quests = Some(engine);
        self.saved_npc_quests = None;
        self.load_live_events(); // T418: before any Talk, not on the first tick
        self.register_breadcrumb_crosswalk(crosswalk);
        self.job_browser_packet = job_browser.packet;
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

    /// T743: [`Self::append_npc_quest_outcome`] for an event outcome. A step that just opened
    /// may be a job-level goal the character's rank already meets, so credit it now.
    pub(super) fn append_npc_quest_outcome_synced(
        &mut self,
        quest: crate::quest_engine::QuestOutcome,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let activated = quest.activated_step();
        self.append_npc_quest_outcome(quest, out)?;
        if activated {
            self.sync_job_level_objectives(out)?;
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
            self.append_npc_quest_outcome_synced(result, out)?;
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
        self.npc_minigame_win_with(replies, true, out)
    }

    /// The same settle when the 39/18 score and timer came from the client
    /// (cooking, smelting): only the game id is quest authority, so the result
    /// event is never emitted.
    pub(super) fn npc_minigame_win_id_only(
        &mut self,
        replies: &[Vec<u8>],
        out: &mut Outcome,
    ) -> io::Result<()> {
        self.npc_minigame_win_with(replies, false, out)
    }

    fn npc_minigame_win_with(
        &mut self,
        replies: &[Vec<u8>],
        report_facts: bool,
        out: &mut Outcome,
    ) -> io::Result<()> {
        // Prefer the 39/18 result record when the same outcome also carries the
        // movie's 39/14 win message: only the record has the run's facts.
        let packet = winning_game_over(replies).or_else(|| winning_movie_message(replies));
        if let Some(packet) = packet {
            let Some(id) = game_over_game_id(packet) else {
                return Ok(());
            };
            if report_facts && packet.starts_with(&[39, 0, 18]) {
                self.credit_minigame(id, game_over_result(packet), out)?;
            } else {
                self.npc_quest_event(
                    crate::quest_engine::QuestEvent::FinishMinigame { minigame_id: id },
                    out,
                )?;
            }
        }
        Ok(())
    }

    /// T545 (R): developer `realm duel <minigame_id>`. The card game has no result path yet,
    /// so this is the one native way to prove the Card Duelist chain: it credits one
    /// `FinishMinigame` event for a client TYPE 16 row and pays nothing itself. Rewards stay
    /// in the quest and are paid only by its own 49/14 turn-in.
    ///
    /// A non-TYPE 16 id (kart 18, derby 39, Treasure War 727, zero, an unknown row) is
    /// refused without an event. The command is admin-only: `developer_command_denied`
    /// runs before the realm dispatch, so no player or builder reaches this method.
    pub(super) fn credit_card_duel(
        &mut self,
        minigame_id: u32,
        out: &mut Outcome,
    ) -> io::Result<String> {
        if !self
            .activities
            .cards()
            .is_some_and(|cards| cards.is_duel_row(minigame_id))
        {
            return Ok(format!(
                "No card duel {minigame_id}: not a client TYPE 16 MiniGameData row; nothing was credited."
            ));
        }
        self.npc_quest_event(
            crate::quest_engine::QuestEvent::FinishMinigame { minigame_id },
            out,
        )?;
        self.sync_job_level_objectives(out)?;
        Ok(format!("Credited card duel {minigame_id}."))
    }

    /// The result record of a run whose id-only credit already happened on the
    /// earlier 39/14 win message (Forging, Mining and Harvesting present the record
    /// only after the client's `FRServer_GameOver`). Sends only `MinigameResult`, so
    /// a multi-completion id-only objective cannot count the same run twice.
    pub(super) fn npc_minigame_result(
        &mut self,
        replies: &[Vec<u8>],
        out: &mut Outcome,
    ) -> io::Result<()> {
        let Some(packet) = winning_game_over(replies) else {
            return Ok(());
        };
        let Some(id) = game_over_game_id(packet) else {
            return Ok(());
        };
        self.npc_quest_event(
            crate::quest_engine::QuestEvent::MinigameResult {
                minigame_id: id,
                result: game_over_result(packet),
            },
            out,
        )
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
        // The Jobs window's Become-a list: the same static rows for everyone (R).
        if !self.job_browser_packet.is_empty() {
            out.replies.push(self.job_browser_packet.clone());
        }
        self.npc_quests_synced = true;
        Ok(())
    }

    /// The login self-data quest journal (`00940c60`): this character's own quests, never
    /// the captured character's. Opening content owns it when loaded (its accepted,
    /// unrewarded stages); otherwise the captured records of the local quests the character
    /// holds (1617, with its bound counter); with no gameplay, none. Split quests stay out:
    /// `append_npc_quest_login_sync` adds them with 49/3, and the client inserts without a
    /// duplicate check (`009225d0` -> `008e9330`), so listing them here would show them twice.
    pub fn write_self_quests(
        game: Option<&Self>,
        data: &mut crate::startup_selfdata::SelfData,
    ) -> io::Result<()> {
        use crate::startup_selfdata::{replace_quest_journal, retain_quests};
        let Some(game) = game else {
            retain_quests(data, |_| false, 0);
            return Ok(());
        };
        data.quests = game.quests.state().clone();
        if let (Some(journal), Some(opening)) = (&game.journal, &game.opening) {
            replace_quest_journal(data, &journal.journal(&opening.journal_entries())?);
            return Ok(());
        }
        let quests = &game.quests.state().quests;
        let completed = quests.values().filter(|quest| quest.rewarded).count() as i32;
        let own = |id| quests.get(&id).is_some_and(|q| q.accepted && !q.rewarded);
        retain_quests(data, own, completed);
        Ok(())
    }
}

/// Runtime guids of the 35/2 AddNpc packets in `packets` (actor id at 4..12, reader `00921a10`).
pub(super) fn added_npcs(packets: &[Vec<u8>]) -> BTreeSet<u64> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .filter_map(|packet| Some(u64::from_le_bytes(packet.get(4..12)?.try_into().ok()?)))
        .collect()
}

/// T499 quest markers: '!' and '?' over NPCs, from this player's own quest state. Markers are
/// per player and never saved; a reconnect recomputes them when the NPCs are added again.
impl Gameplay {
    /// The engine's quest revision, which the marker passes compare; 0 without a catalogue.
    pub fn quest_marker_revision(&self) -> u64 {
        self.npc_quests
            .as_ref()
            .map_or(0, |engine| engine.revision())
    }

    /// Shared-field path: refresh the markers of NPCs the client already `known`s after the quest
    /// revision moved (new NPCs get theirs from [`Self::shared_npc_view`]).
    pub fn quest_marker_packets(&self, known: &BTreeSet<u64>) -> Vec<Vec<u8>> {
        self.marker_packets(&BTreeSet::new(), known)
    }

    /// Per-player path: after a batch, add markers for the NPCs it spawned and, when the quest
    /// revision moved since the last batch, refresh the rest of what the tracker shows.
    /// Cost with nothing new and an unchanged revision: one scan of `packets`.
    pub(super) fn append_quest_markers(&mut self, packets: &mut Vec<Vec<u8>>) {
        let revision = self.quest_marker_revision();
        let added = added_npcs(packets);
        let stale = match &self.npc_visibility {
            Some(visibility) if self.quest_markers.rendered != revision => {
                visibility.visible().difference(&added).copied().collect()
            }
            _ => BTreeSet::new(),
        };
        self.quest_markers.rendered = revision;
        if !added.is_empty() || !stale.is_empty() {
            // A trailing 26/4 (close dialogue) stays the last packet of its reply.
            let end =
                packets.len() - usize::from(packets.last().is_some_and(|p| p == &[26, 0, 4, 0]));
            let markers = self.marker_packets(&added, &stale);
            packets.splice(end..end, markers);
        }
    }

    /// 35/10 for each NPC with a marker, then one 35/11 for the `known` quest NPCs that have
    /// none. `fresh` NPCs only get a 35/10: their AddNpc is already in the batch ahead of this
    /// one, and a new actor has nothing to clear. Guids are runtime guids; a guid the NPC table
    /// does not show gets nothing. Reads the quest state, never changes it.
    pub(super) fn marker_packets(
        &self,
        fresh: &BTreeSet<u64>,
        known: &BTreeSet<u64>,
    ) -> Vec<Vec<u8>> {
        let (Some(engine), Some(content), Some(npcs)) = (
            self.npc_quests.as_ref(),
            self.quest_markers.content.as_deref(),
            self.npc_visibility.as_ref(),
        ) else {
            return Vec::new();
        };
        let (mut show, mut clear) = (Vec::new(), Vec::new());
        for (guids, refresh) in [(fresh, false), (known, true)] {
            if guids.is_empty() {
                continue;
            }
            for (guid, key, name_id) in npcs.marker_targets(guids) {
                match marker_for(engine, key) {
                    Some(marker) => show.push((guid, content.image_id(marker), name_id)),
                    None if refresh && engine.is_quest_npc(key) => clear.push(guid),
                    None => {}
                }
            }
        }
        // The encoders refuse only more than 256 rows and the view holds at most 64 NPCs, so a
        // refusal is a skipped marker for this player, logged, never an error out of the tick.
        let encoded = [
            (!show.is_empty()).then(|| encode_add_notifications(&show)),
            (!clear.is_empty()).then(|| encode_remove_notifications(&clear)),
        ];
        encoded
            .into_iter()
            .flatten()
            .filter_map(|packet| {
                if packet.is_none() {
                    npc_event(serde_json::json!({"event": "quest_marker_skipped"}));
                }
                packet
            })
            .collect()
    }
}

/// The server-built winning `39/18` result record. Body: shared header, `u8 won`
/// at 15, then stars, elapsed seconds and score at 16/20/24 (client 39/18 reader
/// 0x009b7ec0 -> `ApplyMiniGameGameOverResult` 0x009ad480; field body 0x009b48b0).
fn winning_game_over(replies: &[Vec<u8>]) -> Option<&Vec<u8>> {
    replies
        .iter()
        .find(|packet| packet.len() >= 28 && packet.starts_with(&[39, 0, 18]) && packet[15] == 1)
}

/// The movie's `39/14` `OnGameWonMsg` (client reader 0x009b89e0); it carries no
/// result facts.
fn winning_movie_message(replies: &[Vec<u8>]) -> Option<&Vec<u8>> {
    replies.iter().find(|packet| {
        packet.len() >= 19 && packet.starts_with(&[39, 0, 14]) && &packet[19..] == b"OnGameWonMsg\0"
    })
}

/// The game id at bytes 11..15 of either winning message. Only a positive id can
/// name an objective.
fn game_over_game_id(packet: &[u8]) -> Option<u32> {
    packet
        .get(11..15)
        .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
        .map(i32::from_le_bytes)
        .and_then(|id| u32::try_from(id).ok())
        .filter(|id| *id > 0)
}

fn game_over_result(packet: &[u8]) -> crate::quest_engine::MinigameResult {
    crate::quest_engine::MinigameResult {
        won: true,
        elapsed_seconds: game_over_field(packet, 20, crate::content::MAX_MINIGAME_ELAPSED_SECONDS),
        score: game_over_field(packet, 24, crate::content::MAX_MINIGAME_SCORE),
        placement: None,
        knocked_out: false,
    }
}

/// One `39/18` result dword at `offset`, reported only when it is a non-negative
/// value inside `max`. A value outside the sanity bound is not reported rather than
/// failing the quest event and the settled match.
fn game_over_field(packet: &[u8], offset: usize, max: u32) -> Option<u32> {
    let bytes: [u8; 4] = packet.get(offset..offset + 4)?.try_into().ok()?;
    u32::try_from(i32::from_le_bytes(bytes))
        .ok()
        .filter(|value| *value <= max)
}
