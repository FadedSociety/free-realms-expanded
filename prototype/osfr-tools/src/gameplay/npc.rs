//! NPC interaction router: the 26/x, 45/7, 49/x, 165/x and 66/x families, NPC visibility, and the dialogue, merchant and quest-offer packet builders.
use super::npc_quests::added_npcs;
use super::*;

impl Gameplay {
    pub fn exclude_startup_npcs(&mut self, actor_guids: impl IntoIterator<Item = u64>) {
        if let Some(visibility) = &mut self.npc_visibility {
            visibility.exclude_actor_guids(actor_guids);
        }
    }

    pub fn update_npc_visibility(
        &mut self,
        character: &CharacterState,
    ) -> io::Result<Vec<Vec<u8>>> {
        let mut packets = self
            .npc_visibility
            .as_mut()
            .map(|visibility| visibility.update(character.zone_id, character.position))
            .unwrap_or_else(|| Ok(Vec::new()))?;
        if !packets.is_empty() {
            npc_event(serde_json::json!({
                "event": "npc_visibility",
                "zone_id": character.zone_id,
                "position": character.position,
                "added": packets.iter().filter(|p| p.starts_with(&[35, 0, 2, 0])).count(),
                "removed": packets.iter().filter(|p| p.starts_with(&[35, 0, 3, 0])).count(),
            }));
        }
        self.append_quest_markers(&mut packets);
        self.append_pending_turnin_screen(&mut packets)?;
        Ok(packets)
    }

    /// (R) Common NPC budget, computed without mutating staged Gameplay clones.
    pub fn npc_visibility_revision(&self) -> u64 {
        self.npc_visibility.as_ref().map_or(0, |v| v.revision())
    }

    /// T507: the per-model ambient animation table, when the content set has one.
    pub fn npc_ambient(&self) -> Option<&crate::npc_ambient::NpcAmbient> {
        self.npc_ambient.as_deref()
    }

    /// Client model of a passive content actor; excluded field actors have none.
    pub fn npc_model(&self, guid: u64) -> Option<u32> {
        self.npc_visibility.as_ref()?.model_of(guid)
    }

    /// T753: the world zone/region/place table `NpcVisibility::load` read, or `None`
    /// when the content set has no `regions.json`. The later zone slices call this.
    pub fn region_table(&self) -> Option<&crate::world_regions::RegionTable> {
        self.npc_visibility.as_ref()?.region_table()
    }

    pub fn shared_npc_view(
        &self,
        character: &CharacterState,
        fields: &[crate::world::NpcInterest],
        known: &BTreeSet<u64>,
    ) -> io::Result<(BTreeSet<u64>, Vec<Vec<u8>>)> {
        let (visible, mut packets) = self
            .npc_visibility
            .as_ref()
            .unwrap_or(&crate::world::NpcVisibility::default())
            .shared_view(character.zone_id, character.position, fields, known)?;
        packets.extend(self.marker_packets(&added_npcs(&packets), &BTreeSet::new()));
        self.append_shared_pending_end_screen(&mut packets);
        Ok((visible, packets))
    }

    /// Canonical runtime-GUID -> content-key map for a client interaction target; opening-scene
    /// actors are excluded because the opening trigger path drives them, not the NPC router.
    pub(super) fn content_npc_key(&self, runtime_guid: u64) -> Option<u64> {
        let opening = self.opening.as_ref();
        if opening.is_some_and(|o| o.actors().iter().any(|a| a.guid == runtime_guid)) {
            return None;
        }
        self.content_key_for_turnin(runtime_guid)
    }

    /// (R) A live activity closes ordinary regional NPC interaction. `handle_in_world_inner`
    /// reaches this router before the opening handlers' activity refusals, so the shared busy
    /// table has to hold here too. A refusal changes no menu, quest or save state.
    fn busy_interaction_refusal(&self) -> Option<Outcome> {
        self.activity_busy().then(|| Outcome {
            replies: vec![chat::system(
                "Finish the active activity before interacting.",
            )],
            ..Outcome::default()
        })
    }

    /// Stage one NPC service choice: clone this player's gameplay, run the choice on the
    /// clone, and commit it only when the choice reported a change. A refusal keeps the
    /// live state, including the open menu and the pending offer.
    fn commit_if_changed(
        &mut self,
        f: impl FnOnce(&mut Gameplay) -> io::Result<Outcome>,
    ) -> io::Result<Outcome> {
        let mut staged = self.clone();
        let out = f(&mut staged)?;
        if out.changed {
            *self = staged;
        }
        Ok(out)
    }

    pub(super) fn handle_npc_packet(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        // Overworld enemies answer interact/select with the encounter's Attack
        // menu; Legacy also lists their GUIDs as ordinary NPCs (run 002115).
        if (inner.starts_with(&[26, 0, 8, 0]) || inner.starts_with(&[26, 0, 10, 0]))
            && inner.len() >= 12
            && self.encounter.as_ref().is_some_and(|encounter| {
                encounter.is_field_actor(u64::from_le_bytes(inner[4..12].try_into().unwrap()))
            })
        {
            return Ok(None);
        }
        if let Some(out) = self.npc_dialogue_26_6(inner)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_dialogue_close_26_4(inner)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_client_objective_45_7(inner)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_journal_49_15(inner)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_quest_reply_49_2(inner)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_quest_end_49_14(inner)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_quest_abandon_26_23(inner)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.station_interaction_26_8(inner, character)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_interact_26_8(inner, character)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_select_26_10(inner, character, now_ms)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_store_165(inner, character, now_ms)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_merchant_66(inner, character)? {
            return Ok(Some(out));
        }
        if let Some(out) = self.npc_merchant_buy_66_43(inner, character, now_ms)? {
            return Ok(Some(out));
        }
        Ok(None)
    }

    fn npc_dialogue_26_6(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if inner.starts_with(&[26, 0, 6, 0]) {
            let selected = npc_interaction::decode_dialogue_response_26_6(inner)
                .map_err(|_| invalid("invalid dialogue response"))?;
            npc_event(serde_json::json!({"event": "dialogue_choice", "value": selected}));
            // T773: a greeter tips row re-renders the same dialogue with that page's
            // body. It is read-only, so the interaction (and its offered rows) stays
            // open for another page or Goodbye.
            if let Some((key, context)) = self
                .npc_interaction
                .as_ref()
                .and_then(|interaction| interaction.explainer_page(selected))
            {
                let mut dialogue = dialogue_packet_with(&context, None)
                    .map_err(|_| invalid("NPC dialogue could not be encoded"))?;
                // The dialog camera target stays on the actor this player clicked; the
                // last Talk recorded it, because 26/6 carries only the response id.
                let runtime_guid = self
                    .last_npc_dialogue
                    .as_ref()
                    .filter(|(_, _, last_key, _)| *last_key == key)
                    .map(|(_, guid, _, _)| *guid)
                    .unwrap_or(key);
                patch_npc_actor(&mut dialogue, runtime_guid);
                npc_event(serde_json::json!({
                    "event": "greeter_tips",
                    "content_key": key,
                    "response_id": selected
                }));
                return Ok(Some(Outcome {
                    replies: vec![dialogue],
                    ..Outcome::default()
                }));
            }
            // (R) The choice reply carries the row's response id (or its index);
            // the quest row is first, so accept either form.
            if selected == npc_interaction::QUEST_CHOICE_RESPONSE || selected == 0 {
                if let Some((runtime_guid, key, quest_id)) = self.pending_quest_choice.take() {
                    let mut held = std::mem::take(&mut self.pending_choice_packets);
                    if !held.is_empty() {
                        // 49/13 names the turn-in speaker by runtime actor GUID. The quest
                        // Talk named the content NPC key, which differs for an extra-spawn
                        // actor bound through a content_key, so patch the speaker here where
                        // the clicked runtime GUID is still known.
                        for packet in &mut held {
                            patch_npc_actor(packet, runtime_guid);
                        }
                        npc_event(
                            serde_json::json!({"event": "quest_end_from_choice", "quest_id": quest_id}),
                        );
                        return Ok(Some(Outcome {
                            replies: held,
                            ..Outcome::default()
                        }));
                    }
                    let mut out = Outcome::default();
                    if let Some(engine) = self.npc_quests.as_mut() {
                        let result = engine.offer(quest_id, key);
                        self.append_npc_quest_outcome(result, &mut out)?;
                    }
                    for packet in &mut out.replies {
                        patch_npc_actor(packet, runtime_guid);
                    }
                    npc_event(
                        serde_json::json!({"event": "quest_offer_from_choice", "quest_id": quest_id}),
                    );
                    return Ok(Some(out));
                }
            }
            self.pending_quest_choice = None;
            self.pending_choice_packets.clear();
            if let Some(interaction) = &mut self.npc_interaction {
                interaction.close_dialogue();
            }
            return Ok(Some(Outcome {
                replies: vec![npc_interaction::encode_dialogue_close_26_4()],
                ..Outcome::default()
            }));
        }
        Ok(None)
    }

    fn npc_dialogue_close_26_4(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if inner.starts_with(&[26, 0, 4, 0]) {
            if inner.len() != 4 {
                return Err(invalid("invalid dialogue close request"));
            }
            // (R) Also accept an inbound empty 26/4 if the client mirrors EndDialog.
            if let Some(interaction) = &mut self.npc_interaction {
                interaction.close_dialogue();
            }
            return Ok(Some(Outcome::default()));
        }
        Ok(None)
    }

    /// 45/7: the client says a goal it judges itself is met (W175). Only an open goal of
    /// the character's own active quest can move; anything else changes nothing (R).
    fn npc_client_objective_45_7(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if let Some(client_id) = crate::quest_engine::decode_client_complete(inner)
            .map_err(|_| invalid("invalid client objective completion"))?
        {
            let mut out = Outcome::default();
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::ClientReported { client_id },
                &mut out,
            )?;
            npc_event(serde_json::json!({
                "event": "client_objective",
                "outcome": if out.changed { "advanced" } else { "ignored" }
            }));
            return Ok(Some(out));
        }
        Ok(None)
    }

    fn npc_journal_49_15(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if let Some(request) = crate::quest_journal::decode(inner)
            .map_err(|_| invalid("invalid quest journal request"))?
        {
            // 49/15 names an ORIGINAL quest id; an id the crosswalk does not list gets one
            // chat line and changes nothing (R).
            let Some(request) =
                crate::quest_journal::translate(request, &self.breadcrumb_crosswalk)
            else {
                npc_event(serde_json::json!({"event": "quest_journal", "outcome": "unlisted"}));
                return Ok(Some(Outcome {
                    replies: vec![chat::system(crate::quest_journal::BREADCRUMB_REFUSED)],
                    ..Outcome::default()
                }));
            };
            let Some(engine) = self.npc_quests.as_mut() else {
                return Ok(Some(Outcome::default()));
            };
            let breadcrumb = matches!(
                request,
                crate::quest_journal::JournalRequest::Breadcrumb { .. }
            );
            let result = crate::quest_journal::handle(engine, request, &self.breadcrumb_quests);
            let refused = breadcrumb && result.denial.is_some() && result.outbound.is_empty();
            npc_event(serde_json::json!({
                "event": "quest_journal",
                "request": format!("{request:?}"),
                "outcome": if result.changed {
                    "changed"
                } else if result.denial.is_some() {
                    "rejected"
                } else {
                    "unchanged"
                }
            }));
            let mut out = Outcome::default();
            self.append_npc_quest_outcome(result, &mut out)?;
            if refused {
                out.replies
                    .push(chat::system(crate::quest_journal::BREADCRUMB_REFUSED));
            }
            return Ok(Some(out));
        }
        Ok(None)
    }

    fn npc_quest_reply_49_2(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if inner.starts_with(&[49, 0, 2, 0]) {
            let (quest_id, accept) = npc_interaction::decode_quest_reply_49_2(inner)
                .map_err(|_| invalid("invalid quest accept/decline reply"))?;
            let Some(engine) = self.npc_quests.as_mut() else {
                npc_event(serde_json::json!({
                    "event": "quest_reply",
                    "quest_id": quest_id,
                    "accepted": accept,
                    "outcome": "unavailable"
                }));
                return Ok(Some(Outcome::default()));
            };
            let result = if accept {
                engine.accept(quest_id)
            } else {
                engine.decline(quest_id)
            };
            // T418: an accept of a closed live event withdraws the offer
            // (changed, with a denial); it is not an accept.
            let accepted = accept && result.changed && result.denial.is_none();
            npc_event(serde_json::json!({
                "event": "quest_reply",
                "quest_id": quest_id,
                "accepted": accept,
                "outcome": if !result.changed {
                    "rejected"
                } else if result.denial.is_some() {
                    "withdrawn"
                } else if accept {
                    "accepted"
                } else {
                    "declined"
                }
            }));
            let mut out = Outcome::default();
            self.append_npc_quest_outcome(result, &mut out)?;
            // T361: an accept can complete a job-level goal the character already
            // meets. A refused or withdrawn accept starts no quest.
            if accepted {
                self.sync_job_level_objectives(&mut out)?;
            }
            // End the NPC dialog so the client leaves dialog camera and restores
            // the HUD (native run 200411 stayed in dialog view after accepting).
            out.replies.push(END_DIALOG_26_4.to_vec());
            return Ok(Some(out));
        }
        Ok(None)
    }

    fn npc_quest_end_49_14(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if inner.starts_with(&[49, 0, 14, 0]) {
            let quest_id = npc_interaction::decode_quest_end_reply_49_14(inner)
                .map_err(|_| invalid("invalid quest end reply"))?;
            let Some(engine) = &self.npc_quests else {
                return Ok(Some(Outcome::default()));
            };
            let profile_state = self
                .profiles
                .as_ref()
                .map(|profiles| profiles.state().clone())
                .unwrap_or_else(|| ProfileState {
                    version: 1,
                    active: 1,
                    unlocked: BTreeSet::from([1]),
                    experience: Default::default(),
                });
            // (R) Preserve pre-existing stack counts even if the reconstructed 100-slot
            // limit is lower; any new slot still has to fit within that existing floor.
            let bag_capacity = self.inventory.state().items.len().max(100);
            let reward_state =
                RewardState::new(self.inventory.clone(), profile_state.clone(), bag_capacity);
            let mut staged_engine = engine.clone();
            let (quest, next_state) = staged_engine.finish_turnin(quest_id, &reward_state)?;
            let next_profiles = if self.profiles.is_some() {
                if next_state.profiles == profile_state {
                    None
                } else {
                    let directory = self
                        .profile_content_directory
                        .as_deref()
                        .ok_or_else(|| invalid("quest XP needs profile content"))?;
                    Some(Profiles::adopt(
                        &self.profiles,
                        directory,
                        next_state.profiles.clone(),
                    )?)
                }
            } else if next_state.profiles != profile_state {
                return Err(invalid("quest XP needs character profile state"));
            } else {
                None
            };

            self.npc_quests = Some(staged_engine);
            self.inventory = next_state.inventory;
            if let Some(profiles) = next_profiles {
                self.profiles = Some(profiles);
            }
            self.refresh_npc_quest_eligibility(current_unix_seconds());
            // The accepted turn-in is the one-shot completion record. A replayed 49/14 is denied
            // (no pending turn-in), and the job-gate release sets `changed` with a denial.
            let completed = quest.denial.is_none() && quest.changed;
            let mut out = Outcome::default();
            self.append_npc_quest_outcome(quest, &mut out)?;
            // T361: a quest's own job XP can raise the level that another active
            // job-level goal watches, so the turn-in notifies like every level path.
            self.sync_job_level_objectives(&mut out)?;
            if completed {
                // T229: the quest's Station Cash kind is one more reward on the staged clone.
                self.pay_earned_station_cash(
                    crate::station_cash::EarnSource::Quest { id: quest_id },
                    &mut out,
                )?;
            }
            out.replies.push(END_DIALOG_26_4.to_vec());
            return Ok(Some(out));
        }
        Ok(None)
    }

    /// Client body is 8 bytes: writer `0x00c0a350` runs the preamble `0x0090ac70` (u16 family
    /// 26, u16 subtype 23) and then appends the u32 quest id from `this+0x10` (ctor `0x00c09f60`).
    fn npc_quest_abandon_26_23(&mut self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        if inner.starts_with(&[26, 0, 23, 0]) {
            if inner.len() != 8 {
                return Err(invalid("invalid quest abandon request"));
            }
            let quest_id = u32::from_le_bytes(inner[4..8].try_into().unwrap());
            let mut out = Outcome::default();
            let result = self
                .npc_quests
                .as_mut()
                .map(|engine| engine.abandon(quest_id));
            if let Some(result) = result {
                self.append_npc_quest_outcome(result, &mut out)?;
            }
            return Ok(Some(out));
        }
        Ok(None)
    }

    fn npc_interact_26_8(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        if inner.starts_with(&[26, 0, 8, 0]) {
            let request = npc_interaction::decode_npc_interact_26_8(inner)
                .map_err(|_| invalid("invalid NPC interaction request"))?;
            let Some(key) = self.content_npc_key(request.target) else {
                npc_event(serde_json::json!({
                    "event": "npc_interact",
                    "guid": request.target,
                    "content_key": serde_json::Value::Null,
                    "outcome": "none",
                    "reason": "unknown_runtime_guid"
                }));
                return Ok(None);
            };
            if let Some(refusal) = self.busy_interaction_refusal() {
                npc_event(serde_json::json!({
                    "event": "npc_interact",
                    "guid": request.target,
                    "content_key": key,
                    "outcome": "refused",
                    "reason": "busy"
                }));
                return Ok(Some(refusal));
            }
            npc_event(serde_json::json!({
                "event": "npc_interact",
                "guid": request.target,
                "content_key": key
            }));
            let Some(interaction) = &mut self.npc_interaction else {
                npc_event(serde_json::json!({
                    "event": "npc_menu",
                    "guid": request.target,
                    "content_key": key,
                    "options": [],
                    "outcome": "none",
                    "reason": "npc_content_unavailable"
                }));
                return Ok(None);
            };
            let menu = interaction
                .interact(
                    key,
                    character.zone_id,
                    character.position,
                    self.npc_quests.as_ref(),
                )
                .map_err(|_| invalid("NPC interaction is unavailable"))?;
            let options: Vec<_> = menu
                .options
                .iter()
                .map(|option| {
                    let action = match option.action {
                        npc_interaction::NpcAction::Talk => "talk",
                        npc_interaction::NpcAction::TurnInQuest { .. } => "turn_in_quest",
                        npc_interaction::NpcAction::Shop => "shop",
                        npc_interaction::NpcAction::Race => "race",
                        npc_interaction::NpcAction::Derby => "derby",
                        npc_interaction::NpcAction::Fish => "fish",
                        npc_interaction::NpcAction::Learn { .. } => "learn",
                    };
                    serde_json::json!({
                        "id": option.id,
                        "action": action,
                        "string_id": option.string_id
                    })
                })
                .collect();
            npc_event(serde_json::json!({
                "event": "npc_menu",
                "guid": request.target,
                "content_key": key,
                "options": options
            }));
            let mut packet = menu.packet;
            // (R) The client request target is treated as the runtime actor GUID; the content
            // service uses the stable regional NPC key internally.
            packet[4..12].copy_from_slice(&request.target.to_le_bytes());
            let mut out = Outcome::default();
            // An accepted menu interaction is distinct from the Talk action selected by 26/10.
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::InteractNpc {
                    npc_id: request.target,
                },
                &mut out,
            )?;
            out.replies.push(packet);
            return Ok(Some(out));
        }
        Ok(None)
    }

    fn npc_select_26_10(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if inner.starts_with(&[26, 0, 10, 0]) {
            if inner.len() != 16 {
                return Err(invalid("invalid NPC menu selection"));
            }
            let runtime_guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
            let option_id = i32::from_le_bytes(inner[12..16].try_into().unwrap());
            let Some(key) = self.content_npc_key(runtime_guid) else {
                npc_event(select_event_with_reason(
                    runtime_guid,
                    None,
                    option_id,
                    "none",
                    Some("unknown_runtime_guid"),
                ));
                return Ok(None);
            };
            if let Some(refusal) = self.busy_interaction_refusal() {
                npc_event(select_event_with_reason(
                    runtime_guid,
                    Some(key),
                    option_id,
                    "refused",
                    Some("busy"),
                ));
                return Ok(Some(refusal));
            }
            // Clone the menu for the select() call only: the tables are Arc-backed and the
            // owned active menu is small. Assigning it back ends the clone before any
            // service call, so nothing is held across gameplay work or a save. The gateway
            // only keeps this clone when the whole request succeeds, so a later failed child
            // cannot leave the shared menu consumed.
            let Some(mut interaction) = self.npc_interaction.clone() else {
                npc_event(select_event_with_reason(
                    runtime_guid,
                    Some(key),
                    option_id,
                    "none",
                    Some("npc_content_unavailable"),
                ));
                return Ok(None);
            };
            let mut selection = inner.to_vec();
            selection[4..12].copy_from_slice(&key.to_le_bytes());
            let selected = interaction.select(
                &selection,
                character.zone_id,
                character.position,
                self.npc_quests.as_mut(),
            );
            // Refusals change nothing: a selection the service rejected must not commit the
            // staged clone (it can carry menu/shop mutations made before the error).
            if selected.is_ok() {
                self.npc_interaction = Some(interaction);
            }
            let mut out = Outcome::default();
            let mut consume_menu = false;
            match selected {
                Ok(NpcSelection::Talk(context)) => {
                    self.talk_to_npc(
                        runtime_guid,
                        key,
                        option_id,
                        context,
                        character.zone_id,
                        &mut out,
                    )?;
                }
                Ok(NpcSelection::Shop(window)) => {
                    // The coin vendor's stock is 165/10, not the 66/42 Station Cash list: 66/42
                    // opens the client's Station Cash window and its rows are store bundle ids.
                    // 165/10 raises HandlerMerchant:Show("CoinStore") with the coin window and
                    // fills its stock rows. The row carries the display fields from the item
                    // definition; the displayed price is the definition's coin purchase price,
                    // so a stale definition is updated from stores.json first.
                    let shop = self.npc_coin_shop.as_ref();
                    let (packets, rows) =
                        coin_merchant_packets(&window, &self.inventory, shop, now_ms)
                            .map_err(|_| invalid("NPC merchant list could not be encoded"))?;
                    out.replies.extend(packets);
                    npc_event(select_event(runtime_guid, Some(key), option_id, "shop"));
                    npc_event(shop_open_event(runtime_guid, key, rows));
                }
                Ok(NpcSelection::Race) => {
                    consume_menu = true;
                    npc_event(select_event(runtime_guid, Some(key), option_id, "race"));
                    // (R) Ordinary entry: the NPC service proved range and that this exact
                    // option was offered; the kart service decides admission. Staged, so a
                    // failure after the job switch leaves the player exactly as they were.
                    let busy = self.activity_busy();
                    match self.kart_entry.as_ref().map(|kart| kart.admit(key, busy)) {
                        Some(Ok(())) => {
                            out =
                                self.commit_if_changed(|staged| staged.begin_kart_race(character))?;
                        }
                        Some(Err(denied)) => out.replies.push(chat::system(denied.message())),
                        None => out.replies.push(chat::system(
                            crate::kart_gateway::EntryDenied::NoRace.message(),
                        )),
                    }
                }
                Ok(NpcSelection::Derby) => {
                    consume_menu = true;
                    npc_event(select_event(runtime_guid, Some(key), option_id, "derby"));
                    // (R) The NPC service proved range and that this exact option was offered;
                    // the derby service decides admission. Staging keeps the job switch atomic.
                    let busy = self.activity_busy();
                    match self.derby.as_ref().map(|derby| derby.admit(key, busy)) {
                        Some(Ok(())) => {
                            out = self.commit_if_changed(|staged| {
                                staged.begin_derby(character, now_ms)
                            })?;
                            // The dialog camera closes before the native 39/16 offer.
                            out.replies.insert(0, END_DIALOG_26_4.to_vec());
                        }
                        Some(Err(denied)) => out.replies.push(chat::system(denied.message())),
                        None => out.replies.push(chat::system(
                            crate::derby_gateway::DerbyEntryDenied::Unavailable.message(),
                        )),
                    }
                }
                Ok(NpcSelection::Learn(trainer)) => {
                    consume_menu = true;
                    npc_event(select_event(runtime_guid, Some(key), option_id, "learn"));
                    // (R) Ordinary unlock: the NPC service proved range and that this exact
                    // option was offered; `learn_job` decides the rest.
                    out = self.commit_if_changed(|staged| staged.learn_job(&trainer))?;
                    // Leave the NPC dialog camera first (see the fishing choice).
                    out.replies.insert(0, END_DIALOG_26_4.to_vec());
                }
                Ok(NpcSelection::Fish) => {
                    consume_menu = true;
                    npc_event(select_event(runtime_guid, Some(key), option_id, "fish"));
                    // (R) Ordinary entry: the NPC service proved range and that this exact
                    // option was offered; the fishing service decides admission. Staged, so a
                    // refusal leaves the player exactly as they were.
                    out = self.commit_if_changed(|staged| staged.begin_fishing(key, character))?;
                    // Leave the NPC dialog camera first (it otherwise outlives a menu choice,
                    // as the quest accept showed in native run 200411).
                    out.replies.insert(0, END_DIALOG_26_4.to_vec());
                }
                Ok(NpcSelection::Quest(quest)) => {
                    let has_offer = quest.outbound.iter().any(|message| {
                        matches!(
                            message,
                            crate::quest_engine::OutboundMessage::Quest(
                                crate::quest_engine::QuestMessage::QuestInfo { .. }
                            )
                        )
                    });
                    self.append_npc_quest_outcome(quest, &mut out)?;
                    npc_event(select_event_with_reason(
                        runtime_guid,
                        Some(key),
                        option_id,
                        if has_offer { "offer" } else { "none" },
                        if has_offer {
                            None
                        } else {
                            Some("quest_turnin")
                        },
                    ));
                }
                Err(crate::npc_interaction::NpcInteractionError::NeedsQuestEngine) => {
                    out.replies
                        .push(chat::system("Quest data is unavailable in this area."));
                    npc_event(select_event_with_reason(
                        runtime_guid,
                        Some(key),
                        option_id,
                        "none",
                        Some("quest_engine_unavailable"),
                    ));
                }
                Err(_) => {
                    npc_event(select_event_with_reason(
                        runtime_guid,
                        Some(key),
                        option_id,
                        "none",
                        Some("selection_rejected"),
                    ));
                    return Err(invalid("NPC menu choice is unavailable"));
                }
            }
            if consume_menu && out.changed {
                if let Some(interaction) = &mut self.npc_interaction {
                    interaction.consume_active();
                }
            }
            return Ok(Some(out));
        }
        Ok(None)
    }

    /// 165/4 buy and 165/5 sell, plus the siblings (W172): 2 definitions, 8 list update,
    /// 12 buy-back, 14 gift (always refused).
    fn npc_store_165(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if matches!(inner, [0xa5, 0, 2 | 4 | 5 | 8 | 12 | 14, 0, ..]) {
            if let Some(shop) = &mut self.npc_coin_shop {
                let Some(interaction) = &mut self.npc_interaction else {
                    return Ok(None);
                };
                let Some(visibility) = &self.npc_visibility else {
                    return Ok(None);
                };
                // Only buys and sells name a merchant; 165/8 is 4 bytes and 165/2 and 165/12
                // carry other fields at 4..12, so those are never rewritten (W172).
                let merchant = inner
                    .get(4..12)
                    .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
                let mut request = inner.to_vec();
                if matches!(inner[2], 4 | 5) {
                    if let Some(key) = visibility.content_key(merchant) {
                        request[4..12].copy_from_slice(&key.to_le_bytes());
                    }
                }
                let content_key = request
                    .get(4..12)
                    .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
                // The shop was built with the login-time inventory; without this
                // a purchase overwrote everything earned since (10-15 audit).
                shop.sync_inventory(self.inventory.clone());
                shop.sync_pet_room(self.pets.room());
                let before = self.inventory.state().clone();
                let outcome = interaction.handle_coin_store(
                    shop,
                    &request,
                    character.zone_id,
                    character.position,
                );
                match outcome {
                    Ok(outcome) => {
                        npc_event(serde_json::json!({
                            "event": "shop_txn",
                            "guid": merchant,
                            "content_key": content_key,
                            "subtype": u16::from_le_bytes(inner[2..4].try_into().unwrap()),
                            "outcome": "accepted"
                        }));
                        self.inventory = shop.inventory().clone();
                        let mut out = Outcome {
                            replies: outcome.replies,
                            changed: outcome.changed,
                            ..Outcome::default()
                        };
                        self.adopt_gained_pets(&before, now_ms, &mut out)?;
                        return Ok(Some(out));
                    }
                    Err(error) => {
                        npc_event(serde_json::json!({
                            "event": "shop_txn",
                            "guid": merchant,
                            "content_key": content_key,
                            "subtype": u16::from_le_bytes(inner[2..4].try_into().unwrap()),
                            "outcome": "rejected",
                            "reason": format!("{error:?}")
                        }));
                        return Err(invalid("NPC Coin Store transaction was rejected"));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Marketplace siblings (W172): wallet, status, SKU and code lists, account info, and the
    /// Station Cash order rows. The shop answers; a paid order is one staged commit.
    fn npc_merchant_66(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<Outcome>> {
        if matches!(
            inner,
            [
                66,
                0,
                1 | 3 | 10 | 12 | 14 | 16 | 18 | 20 | 22 | 25 | 27 | 38,
                0,
                ..
            ]
        ) {
            // T230: the shared busy table gates a Station Cash order exactly like an activity.
            let busy = self.activity_busy();
            let Some(shop) = &mut self.npc_coin_shop else {
                return Ok(None);
            };
            // The shop was built with the login-time inventory; a reward earned since must not be
            // lost by the order's staged grant (the coin buy path does the same), and the
            // ownership check of a preview or an order and the 66/38 owned flags (T484) read it.
            // A wallet read never touches the inventory.
            if matches!(inner.get(2..4), Some([1 | 3 | 38, 0])) {
                shop.sync_inventory(self.inventory.clone());
            }
            // L046: a paid deed grants this character's lot in the housing session; the order
            // debits only after that grant commits, and both land in the one gameplay save.
            let housing = &mut self.housing;
            let outcome = shop.handle_marketplace(inner, busy, &mut |deed, commit| {
                housing.as_mut()?.grant_deed(character.guid, deed, commit)
            })?;
            if outcome.changed {
                self.inventory = shop.inventory().clone();
            }
            return Ok(Some(Outcome {
                replies: outcome.replies,
                changed: outcome.changed,
                ..Outcome::default()
            }));
        }
        Ok(None)
    }

    fn npc_merchant_buy_66_43(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if inner.starts_with(&[66, 0, 43, 0]) {
            let Ok(request) = npc_interaction::decode_merchant_buy_66_43(inner) else {
                return Ok(Some(Outcome {
                    replies: vec![crate::coin_shop::merchant_buy_result_packet(false)],
                    ..Outcome::default()
                }));
            };
            // (R) The Lua callsite uses its first argument as the item key, its fifth as
            // quantity, and the generated request id as the transaction key. Client price is
            // ignored; stores.json supplies the authoritative local Coin price.
            let item_id = i32::try_from(request.initial_dwords[0]).unwrap_or(-1);
            let quantity = request.trailing_dwords[0];
            let transaction_id = request.trailing_dwords[2];
            let Some(interaction) = &self.npc_interaction else {
                return Ok(Some(Outcome {
                    replies: vec![crate::coin_shop::merchant_buy_result_packet(false)],
                    ..Outcome::default()
                }));
            };
            let refusal_code = interaction
                .validate_merchant_item_66_43(item_id, character.zone_id, character.position)
                .err()
                .map(|error| {
                    if error == npc_interaction::NpcInteractionError::InvalidOption {
                        crate::coin_shop::RESULT_INVALID_ITEM
                    } else {
                        crate::coin_shop::RESULT_INVALID_MERCHANT
                    }
                });
            let Some(shop) = &mut self.npc_coin_shop else {
                return Ok(Some(Outcome {
                    replies: vec![crate::coin_shop::merchant_buy_result_packet(false)],
                    ..Outcome::default()
                }));
            };
            shop.sync_inventory(self.inventory.clone());
            shop.sync_pet_room(self.pets.room());
            let before = self.inventory.state().clone();
            let outcome = if let Some(result_code) = refusal_code {
                shop.refuse_vendor_buy(transaction_id, result_code)
            } else {
                match shop.buy_vendor(transaction_id, item_id, quantity) {
                    Ok(outcome) => outcome,
                    Err(_) => shop.refuse_vendor_buy(
                        transaction_id,
                        crate::coin_shop::RESULT_SERVICE_UNAVAILABLE,
                    ),
                }
            };
            self.inventory = shop.inventory().clone();
            npc_event(serde_json::json!({
                "event": "shop_txn",
                "subtype": 43,
                "item_id": item_id,
                "quantity": quantity,
                "outcome": if outcome.result_code == crate::coin_shop::RESULT_SUCCESS {
                    "accepted"
                } else {
                    "refused"
                }
            }));
            let mut out = Outcome {
                replies: outcome.replies,
                changed: outcome.changed,
                ..Outcome::default()
            };
            self.adopt_gained_pets(&before, now_ms, &mut out)?;
            return Ok(Some(out));
        }
        Ok(None)
    }

    /// Switches to Kart Driver and enters the track. Shared by the developer `realm race` command
    /// and the ordinary NPC menu choice; the caller has already checked admission and `busy`.
    pub(super) fn begin_kart_race(&mut self, character: &CharacterState) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let profiles = self
            .profiles
            .as_mut()
            .ok_or_else(|| invalid("kart entry requires class content"))?;
        let previous_job = profiles.active();
        let outfit_before = self.inventory.profile_attachments(previous_job)?;
        // (R) First use still unlocks Kart Driver for saves that predate trainers; the
        // option is idempotent. W176 also offers the job from a trainer. The prior job is
        // restored on exit.
        let first_time = profiles.unlock(48)?;
        profiles.activate(48)?;
        if first_time {
            self.give_starter_outfit(48, false, &mut out)?;
        }
        self.ensure_outfit(48, &mut out)?;
        self.refresh_npc_quest_eligibility(current_unix_seconds());
        // Last, after the outfit repair: the 38/21 attachment list comes from what
        // the player wears now (client reader `0x0091b080`).
        out.replies.insert(
            0,
            self.profiles
                .as_ref()
                .unwrap()
                .activation(48, &self.inventory)?,
        );
        self.announce_profile_outfit(character.guid, &outfit_before, previous_job != 48, &mut out)?;
        let kart = self.kart_entry.as_mut().unwrap();
        let (replies, scene) = kart.begin(character)?;
        kart.set_previous_job(previous_job);
        out.replies.extend(replies);
        out.scene_zoning = Some(scene);
        out.changed = true;
        Ok(out)
    }

    /// W160 (R): start the fishing round the chosen guide offers. The caller stages `self`, so a
    /// refusal (a chat line, `changed` false) leaves the player exactly as they were.
    pub(super) fn begin_fishing(
        &mut self,
        gate: u64,
        character: &CharacterState,
    ) -> io::Result<Outcome> {
        use crate::fishing_service::{EntryDenied, FISHERMAN_PROFILE};
        let mut out = Outcome::default();
        let busy = self.activity_busy();
        let Some(spots) = self.fishing_spots.clone() else {
            out.replies
                .push(chat::system(EntryDenied::Unavailable.message()));
            return Ok(out);
        };
        let Some(profiles) = self.profiles.as_mut() else {
            out.replies
                .push(chat::system(EntryDenied::Unavailable.message()));
            return Ok(out);
        };
        // (R) The Fisherman chapter is built (quests/fisherman.json) and gated on the
        // active job by its content. The trip unlocks 137; the player activates it from
        // the job list (26/13), whose starter outfit the active play package supplies.
        let _ = profiles.unlock(FISHERMAN_PROFILE);
        let level = self.job_level(FISHERMAN_PROFILE);
        // (R) Same bag floor as quest turn-ins: never below what the player already holds.
        let rewards = self.reward_snapshot();
        match self.fishing.enter(
            &spots,
            gate,
            character.guid,
            character.zone_id,
            character.position,
            busy,
            level,
            rewards,
        ) {
            Ok(change) => self.start_fishing_trip(change, character, &mut out),
            Err(denied) => out.replies.push(chat::system(denied.message())),
        }
        Ok(out)
    }

    /// The Talk arm of one 26/10 choice. Quest events and the named turn-in run before
    /// the dialogue packet; this only reads the menu, after those events, so no guard is
    /// held across a service call or a save.
    fn talk_to_npc(
        &mut self,
        runtime_guid: u64,
        key: u64,
        option_id: i32,
        mut context: DialogueContext,
        zone_id: u32,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let pending_before = self
            .npc_quests
            .as_ref()
            .and_then(|engine| engine.state().pending_turnin);
        if let Some(engine) = &mut self.npc_quests {
            let quest =
                engine.on_event(crate::quest_engine::QuestEvent::TalkToNpc { npc_id: key })?;
            // A completed talk step speaks its own line (Shakey's bubble was
            // blank in native run 215840).
            if let Some(line) = engine.step_dialogue_for(&quest) {
                context.body_string_id = Some(line);
            }
            self.append_npc_quest_outcome_synced(quest, out)?;
        }
        let pending_after = self
            .npc_quests
            .as_ref()
            .and_then(|engine| engine.state().pending_turnin);
        if let Some(quest_id) = pending_before {
            // Reopening is allowed only at the actor that opened the screen; a
            // different actor gets no packet and the pending state is untouched.
            let result = self
                .npc_quests
                .as_ref()
                .expect("pending turn-in requires the NPC quest engine")
                .reopen_turnin(quest_id, key);
            self.append_npc_quest_outcome(result, out)?;
        } else if pending_after.is_none() {
            // `on_event` opens the screen itself when this Talk completed the
            // quest; beginning the same turn-in again would queue a second 49/13.
            let target_quest = self.npc_quests.as_ref().and_then(|engine| {
                self.npc_interaction
                    .as_ref()?
                    .named_target_quest_for_talk(key, engine)
            });
            if let Some(quest_id) = target_quest {
                let result = self
                    .npc_quests
                    .as_mut()
                    .unwrap()
                    .begin_named_target_turnin(quest_id, key);
                self.append_npc_quest_outcome(result, out)?;
            }
        }
        let already_offering = out
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0]));
        let turning_in = out
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0]));
        self.pending_choice_packets.clear();
        let offer_outcome = if already_offering {
            "offer"
        } else if turning_in {
            // Same shape as the offer: the 49/13 end screen opens from a
            // dialog choice (run 202324 drew it over the open dialog).
            let (held, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut out.replies)
                .into_iter()
                .partition(|packet| packet.starts_with(&[49, 0, 13, 0]));
            out.replies = rest;
            let quest_id = held
                .first()
                .and_then(|packet| packet.get(14..18))
                .map(|id| u32::from_le_bytes(id.try_into().unwrap()));
            match quest_id {
                Some(quest_id) => {
                    self.pending_quest_choice = Some((runtime_guid, key, quest_id));
                    self.pending_choice_packets = held;
                    "quest_choice"
                }
                None => {
                    out.replies.extend(held);
                    "dialogue"
                }
            }
        } else {
            // (R) The client leaves the 26/10-to-quest action unresolved; W67 chooses
            // Talk as the server action that starts the separate 49/1 quest window.
            let quest_id = self.npc_quests.as_ref().and_then(|engine| {
                self.npc_interaction
                    .as_ref()?
                    .offerable_quest_for_talk(key, engine)
            });
            // Native runs 200411/201127: the offer window must sit on an
            // open NPC dialog, and the server's 26/4 EndDialog after the
            // reply restores camera and HUD. The quest is therefore a
            // dialog choice; choosing it (26/6) sends the 49/1 offer.
            if let Some(quest_id) = quest_id {
                self.pending_quest_choice = Some((runtime_guid, key, quest_id));
                "quest_choice"
            } else {
                "dialogue"
            }
        };
        let quest_choice = match (offer_outcome, self.pending_quest_choice) {
            ("quest_choice", Some((_, _, quest_id))) => {
                let title = self
                    .npc_quests
                    .as_ref()
                    .and_then(|engine| engine.title_string_id(quest_id))
                    .unwrap_or(0);
                Some(npc_interaction::quest_dialogue_choice(title))
            }
            _ => None,
        };
        let mut dialogue = dialogue_packet_with(&context, quest_choice)
            .map_err(|_| invalid("NPC dialogue could not be encoded"))?;
        patch_npc_actor(&mut dialogue, runtime_guid);
        self.last_npc_dialogue = Some((zone_id, runtime_guid, key, dialogue.clone()));
        // A quest offer is shown alone, like the original: the 49/1 window
        // draws its own speech bubble. (Earlier runs were silently ignored
        // because 49/1 was 120 bytes, not for lack of a 26/3; a leading 26/3
        // left a blank bubble and "Goodbye" behind the quest window.)
        if offer_outcome != "offer" {
            out.replies.push(dialogue);
        }
        let reason = if offer_outcome == "offer" {
            None
        } else if turning_in {
            Some("quest_turnin")
        } else if self.npc_quests.is_none() {
            Some("quest_engine_unavailable")
        } else {
            Some("no_eligible_quest")
        };
        npc_event(select_event_with_reason(
            runtime_guid,
            Some(key),
            option_id,
            offer_outcome,
            reason,
        ));
        Ok(())
    }
}

/// One `npc_select` telemetry row for the reason-less outcomes (shop, race, derby, fish
/// and learn): the five-key shape the old literals emitted.
fn select_event(
    guid: u64,
    content_key: Option<u64>,
    option: i32,
    outcome: &str,
) -> serde_json::Value {
    serde_json::json!({
        "event": "npc_select",
        "guid": guid,
        "content_key": content_key,
        "option": option,
        "outcome": outcome
    })
}

/// One `npc_select` row that carries `reason`; `None` serializes as null, as the old
/// Talk and quest outcomes did.
fn select_event_with_reason(
    guid: u64,
    content_key: Option<u64>,
    option: i32,
    outcome: &str,
    reason: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "event": "npc_select",
        "guid": guid,
        "content_key": content_key,
        "option": option,
        "outcome": outcome,
        "reason": reason
    })
}

/// The extra event a shop choice emits next to its `npc_select` row.
fn shop_open_event(guid: u64, content_key: u64, item_count: usize) -> serde_json::Value {
    serde_json::json!({
        "event": "shop_open",
        "guid": guid,
        "content_key": content_key,
        "item_count": item_count
    })
}

/// Point a server-built NPC packet at the runtime actor the client knows. This is one
/// helper because all three packet families carry the same actor bytes but in different
/// positions; each family is documented here:
/// - 26/3 ShowDialog: the dialog camera target (`ControllerDialog`, switched in by
///   `ReplacePointerAtECAndDispatch` `008d2ba0`) at dwords 2-3 of the leading four
///   (`ShowDialogFromPacket` `009f5ae0` reads object +0x18/+0x1c), so it zooms in on
///   the NPC with the bubble to their left. A zero target falls back to the player
///   (native run 230409, Frida `dialog_camera` hooks).
/// - 49/1 QuestInfo: the actor after the reward bundle, i.e. the packet's last 14
///   bytes start with it (dispatcher `00c7cea2` passes object +0xe0/+0xe4).
/// - 49/13 QuestEnd: the turn-in speaker at offset 6, not the dialog camera. Its
///   `ReadFields` (`00c7b990`) reads the 6-byte prefix, then this 8-byte actor, and
///   `ShowQuestEndScreen` (`00a95420`) resolves it through the client actor table
///   (`00984960`). A quest Talk event names the content NPC key, which differs from
///   the runtime actor for an extra-spawn NPC bound through a `content_key`; the
///   held screen is patched here when 26/6 releases it.
pub(super) fn patch_npc_actor(packet: &mut [u8], npc_guid: u64) {
    let guid = npc_guid.to_le_bytes();
    if packet.starts_with(&[26, 0, 3, 0]) && packet.len() >= 20 {
        packet[12..20].copy_from_slice(&guid);
    } else if packet.starts_with(&[49, 0, 1, 0]) && packet.len() >= 14 {
        let at = packet.len() - 14;
        packet[at..at + 8].copy_from_slice(&guid);
    } else if packet.starts_with(&[49, 0, 13, 0]) && packet.len() >= 14 {
        packet[6..14].copy_from_slice(&guid);
    }
}

pub(super) fn dialogue_packet(
    context: &DialogueContext,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    dialogue_packet_with(context, None)
}

pub(super) fn dialogue_packet_with(
    context: &DialogueContext,
    quest_choice: Option<[u32; 5]>,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    // The locale export maps 8075 to "Goodbye". The five-word response row is
    // client-reader-backed; its 300/17/3 close tuple follows Lua's AddResponseNode
    // convention (notes/Research/Pieces/071 NPC dialogue branching.md). The body
    // field is a locale greeting when one is recovered; it is never the NPC name.
    // T773: a greeter's tips rows follow the greeting. A quest offer keeps its row
    // alone, because the 49/1 window draws its own speech bubble.
    let mut choices: Vec<[u32; 5]> = quest_choice.into_iter().collect();
    if choices.is_empty() {
        choices.extend(
            context
                .explainer_topics
                .iter()
                .map(npc_interaction::explainer_dialogue_choice),
        );
    }
    choices.push(npc_interaction::goodbye_dialogue_choice());
    npc_interaction::encode_dialogue_26_3(&npc_interaction::DialoguePacket26_3 {
        initial_dwords: [context.body_string_id.unwrap_or(0), 0, 0, 0],
        initial_flag: false,
        initial_float_bits: 0,
        choices,
        choice_vectors: [[0; 4]; 2],
        vector_flag: false,
        final_vector: [0; 4],
        float_after_vector_bits: 0,
        final_flags: [false; 3],
        final_float_bits: 0,
        final_dword: 0,
    })
}

/// Build the coin merchant window: the 35/37 definition updates the client needs, then the
/// 165/10 merchant list. Returns the packets and the row count for the shop-open event.
///
/// The client's `MerchantSellInventoryDataSource` resolves every row through its item definition
/// cache and displays the definition's coin purchase price (`+0x24`, serialized dword 7), not a
/// price in the merchant row. A row the login catalogue did not carry is pushed so the window
/// can render it; a cached row whose price differs from `stores.json` is replaced
/// (`00a3c290` overwrites by id), keeping the server the price authority.
///
/// T434: the same batch carries the Sell tab's Worth. The tab lists each owned item at its
/// definition's ResellValue (`00cf08d0`, `00a6e5f0` case 27), so an owned definition that shows
/// something other than what 165/5 pays is pushed with `CoinShop::sell_worth_override`. One
/// record per id carries both patches: the batch refuses two different rows for one id.
fn coin_merchant_packets(
    window: &MerchantWindow,
    inventory: &Inventory,
    shop: Option<&crate::coin_shop::CoinShop>,
    now_ms: u64,
) -> Result<(Vec<Vec<u8>>, usize), npc_interaction::NpcInteractionError> {
    let mut definitions = BTreeMap::new();
    let mut entries = Vec::new();
    for item in &window.items {
        // The stock list is the buy list: a row without a purchase price cannot be bought.
        let Some(price) = item.purchase_price_coins else {
            continue;
        };
        if !inventory.has_definition(item.item_id) {
            continue;
        }
        // T529: do not advertise a row the coin store refuses (an unbound pet item, a ride, a
        // house lot). The same `sale_block` gate decides the buy, so the window and the request
        // cannot disagree.
        if shop.is_some_and(|shop| shop.sale_block(item.item_id).is_some()) {
            continue;
        }
        let presentation = inventory
            .presentation(item.item_id)
            .map_err(|_| npc_interaction::NpcInteractionError::InvalidPacket)?;
        let price_matches = inventory.definition_coin_price(item.item_id) == Some(price);
        if !inventory.definition_was_captured(item.item_id) || !price_matches {
            if let Some(record) = inventory.definition_record_with_coin_price(item.item_id, price) {
                definitions.insert(item.item_id, record);
            }
        }
        entries.push(npc_interaction::MerchantEntryV2 {
            item_id: item.item_id as u32,
            icon_id: u32::try_from(presentation.icon_id).unwrap_or(0),
            tint_value: u32::try_from(presentation.icon_tint).unwrap_or(0),
            name_id: u32::try_from(presentation.name_id).unwrap_or(0),
            description_id: u32::try_from(presentation.description_id).unwrap_or(0),
            // Unlimited server stock; the client's display of 0 is unverified (R).
            quantity_available: 0,
            // Membership and the available tint set live in the definition, not this row.
            members_only: false,
            field_7: 0,
            first_item_id: 0,
            available_tint_group_id: 0,
            can_buy: true,
        });
    }
    // ponytail: one batch bounded by the bag (MAX_ITEMS), like the login catalogue, and one
    // linear `stores.json` lookup per owned definition; split the batch as
    // `coin_shop::item_definitions` does, or index the store by id, if a bag outgrows them.
    let owned: BTreeSet<i32> = inventory
        .state()
        .items
        .iter()
        .map(|item| item.definition_id)
        .collect();
    for id in owned {
        let (Some(worth), Some(base)) = (
            shop.and_then(|shop| shop.sell_worth_override(id)),
            inventory.definition_record(id),
        ) else {
            continue;
        };
        let record = definitions.entry(id).or_insert_with(|| base.to_vec());
        crate::inventory::set_resell_value(record, worth)
            .ok_or(npc_interaction::NpcInteractionError::InvalidPacket)?;
    }
    let rows = entries.len();
    let mut packets = Vec::new();
    if !definitions.is_empty() {
        let definitions: Vec<_> = definitions.into_values().collect();
        packets.push(
            crate::inventory_v2_fixes::encode_definition_batch_v2(&definitions, &[])
                .map_err(|_| npc_interaction::NpcInteractionError::InvalidPacket)?,
        );
    }
    let npc = window.npc_id;
    packets.push(npc_interaction::encode_coin_merchant_list_165_10(
        &npc_interaction::MerchantGroupV2 {
            // [record scalar, pair 1, merchant GUID low/high, scalar 2, unused].
            fields: [now_ms as u32, 0, 0, npc as u32, (npc >> 32) as u32, 0, 0],
            entries,
        },
    )?);
    Ok((packets, rows))
}

pub(super) fn quest_offer_packet(
    engine: &crate::quest_engine::QuestEngine,
    quest_id: u32,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    quest_offer_packet_with(engine, quest_id, None)
}

pub(super) fn quest_offer_packet_with(
    engine: &crate::quest_engine::QuestEngine,
    quest_id: u32,
    intro_override: Option<u32>,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    // The 49/1 reader's common-prefix dword is the quest ID. Subsequent fields
    // feed ShowQuestStartScreen in client order. Locale IDs come from reviewed
    // split content; missing IDs and offer policy are reconstructed (R). Lua
    // QuestStartHandler supplies its own card icon (454), Show Details and the
    // Accept/Decline buttons, so none are synthesized into this payload.
    let title_id = engine.title_string_id(quest_id).unwrap_or(0);
    npc_interaction::encode_quest_offer_49_1(&npc_interaction::QuestOfferPacket49_1 {
        quest_id,
        intro_text_string_id: intro_override
            .unwrap_or_else(|| engine.intro_text_string_id(quest_id).unwrap_or(0)),
        // Slot order probe-verified in native run 143124: dword 3 is the card
        // title and dword 4 the description; dwords 2 and 5 are not shown.
        field_2: title_id,
        name_string_id: title_id,
        description_string_id: engine.description_string_id(quest_id).unwrap_or(0),
        field_5: title_id,
        members_only: false,
        // The 49/1 card and the 49/13 end screen read the same ClientRewardBundle
        // (`sub_8E7930`, client 0x008e7930; 49/1 reader 0x00c7b7a0): the quest's
        // authored coins, summed job-XP stars and counted display rows. Display only;
        // the 49/14 turn-in still pays.
        reward_bundle_body: engine
            .quest_reward_bundle(quest_id)
            .map_err(|_| npc_interaction::NpcInteractionError::InvalidPacket)?,
        trailing_dwords: [0; 2],
        trailing_float_bits: 0,
        trailing_flags: [false; 2],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn keys(value: &serde_json::Value) -> BTreeSet<&str> {
        value
            .as_object()
            .expect("an event is a JSON object")
            .keys()
            .map(String::as_str)
            .collect()
    }

    /// The five-key rows for the choices that have no reason today.
    #[test]
    fn select_event_keeps_the_five_key_reasonless_outcomes() {
        let keys5 = BTreeSet::from(["event", "guid", "content_key", "option", "outcome"]);
        for outcome in ["shop", "race", "derby", "fish", "learn"] {
            let event = select_event(9, Some(11), 2, outcome);
            assert_eq!(keys(&event), keys5, "{outcome}");
            assert!(
                !event.as_object().unwrap().contains_key("reason"),
                "{outcome} gained a reason field"
            );
            assert_eq!(event["event"], "npc_select", "{outcome}");
            assert_eq!(event["guid"], 9, "{outcome}");
            assert_eq!(event["content_key"], 11, "{outcome}");
            assert_eq!(event["option"], 2, "{outcome}");
            assert_eq!(event["outcome"], outcome);
        }
    }

    /// The six-key rows keep `reason`: null for the old null outcomes, a name otherwise.
    #[test]
    fn select_event_with_reason_keeps_null_versus_named() {
        let keys6 = BTreeSet::from([
            "event",
            "guid",
            "content_key",
            "option",
            "outcome",
            "reason",
        ]);
        let unknown = select_event_with_reason(7, None, 3, "none", Some("unknown_runtime_guid"));
        assert_eq!(keys(&unknown), keys6);
        assert_eq!(unknown["event"], "npc_select");
        assert_eq!(unknown["guid"], 7);
        assert!(unknown["content_key"].is_null());
        assert_eq!(unknown["option"], 3);
        assert_eq!(unknown["outcome"], "none");
        assert_eq!(unknown["reason"], "unknown_runtime_guid");

        for (outcome, reason) in [
            ("refused", Some("busy")),
            ("offer", None),
            ("dialogue", None),
            ("quest_choice", Some("no_eligible_quest")),
            ("quest_turnin", Some("quest_turnin")),
            ("none", Some("npc_content_unavailable")),
            ("none", Some("selection_rejected")),
            ("none", Some("quest_engine_unavailable")),
        ] {
            let event = select_event_with_reason(9, Some(11), 2, outcome, reason);
            assert_eq!(keys(&event), keys6, "{outcome}");
            assert_eq!(event["content_key"], 11, "{outcome}");
            assert_eq!(event["outcome"], outcome, "{outcome}");
            assert_eq!(event["reason"].as_str(), reason, "{outcome}");
            if reason.is_none() {
                assert!(event["reason"].is_null(), "{outcome}");
            }
        }
    }

    #[test]
    fn shop_open_keeps_its_extra_row() {
        let event = shop_open_event(9, 11, 4);
        assert_eq!(
            keys(&event),
            BTreeSet::from(["event", "guid", "content_key", "item_count"])
        );
        assert_eq!(event["event"], "shop_open");
        assert_eq!(event["guid"], 9);
        assert_eq!(event["content_key"], 11);
        assert_eq!(event["item_count"], 4);
    }

    /// T318: a display row the client entry reader cannot instantiate makes the 49/1
    /// builder refuse instead of sending a partial card. Content cannot author display
    /// rows yet, so this synthetic engine is the only reachable construction of the
    /// failure; the builder is pure, so a refusal changes nothing.
    #[test]
    fn a_malformed_reward_row_is_refused_by_the_offer_builder() {
        use crate::{
            quest_engine::{CharacterEligibility, QuestDefinition, QuestEngine, QuestRequirements},
            rewards::{RewardBundleRow, RewardDisplay, RewardGrant},
        };
        use std::collections::BTreeMap;

        let definition = |row_type: u32| QuestDefinition {
            id: 7,
            title: "Quest 7".into(),
            description: String::new(),
            giver_npc_id: 11,
            turnin_npc_id: 11,
            prerequisite_quest_ids: Vec::new(),
            requirements: QuestRequirements::default(),
            ordered: false,
            objectives: Vec::new(),
            reward: RewardGrant {
                coins: 5,
                items: Vec::new(),
                job_xp: BTreeMap::new(),
                sc: 0,
                display: RewardDisplay {
                    source_guid: 11,
                    player_guid: 3,
                    reward_kind: 0,
                    icon_id: 0,
                    name_id: 0,
                    rows: vec![RewardBundleRow {
                        type_id: row_type,
                        body: vec![0; 34],
                    }],
                },
            },
        };
        let engine = |row_type| {
            QuestEngine::new(
                3,
                vec![definition(row_type)],
                CharacterEligibility::default(),
                None,
            )
            .expect("display rows are not validated at engine construction")
        };

        let offered = quest_offer_packet_with(&engine(3), 7, None).expect("entry type 3 is read");
        let decoded = crate::npc_interaction::decode_quest_offer_49_1(&offered)
            .expect("the populated offer decodes");
        let body = decoded.reward_bundle_body;
        assert_eq!(
            body.len(),
            69 + 4 + 34,
            "a populated reward appends its counted rows"
        );
        assert_eq!(
            i32::from_le_bytes(body[61..65].try_into().unwrap()),
            1,
            "the client loops over exactly one row"
        );
        assert_eq!(u32::from_le_bytes(body[65..69].try_into().unwrap()), 3);
        assert_eq!(&body[69..103], &[0; 34]);
        assert_eq!(i32::from_le_bytes(body[103..107].try_into().unwrap()), 0);

        assert_eq!(
            quest_offer_packet_with(&engine(2), 7, None).unwrap_err(),
            npc_interaction::NpcInteractionError::InvalidPacket
        );
    }
}
