//! NPC interaction router: the 26/x, 45/7, 49/x, 165/x and 66/x families, NPC visibility, and the dialogue, merchant and quest-offer packet builders.
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
        self.append_pending_turnin_screen(&mut packets)?;
        Ok(packets)
    }

    /// Reopen a saved pending turn-in only after its speaker's 35/2 spawn is in the same
    /// batch. `ShowQuestEndScreen` (0x00A95420) resolves the packet's actor through the
    /// client actor table first and does not retry when it is absent; sending the 49/13
    /// from login synchronization put it before every visibility spawn.
    fn append_pending_turnin_screen(&self, packets: &mut Vec<Vec<u8>>) -> io::Result<()> {
        if packets.is_empty() {
            return Ok(());
        }
        let Some(engine) = self.npc_quests.as_ref() else {
            return Ok(());
        };
        let Some(quest_id) = engine.state().pending_turnin else {
            return Ok(());
        };
        let Some(content_key) = engine.turnin_npc_id(quest_id) else {
            return Ok(());
        };
        let Some(runtime_guid) = packets.iter().find_map(|packet| {
            let spawn = crate::startup_world::parse_add_npc(packet).ok()?;
            let same_npc = spawn.actor_id == content_key
                || self
                    .npc_visibility
                    .as_ref()
                    .and_then(|visibility| visibility.content_key(spawn.actor_id))
                    == Some(content_key);
            same_npc.then_some(spawn.actor_id)
        }) else {
            return Ok(());
        };
        let mut screen = engine.quest_end_packet(quest_id)?;
        if screen.len() < 14 {
            return Ok(());
        }
        screen[6..14].copy_from_slice(&runtime_guid.to_le_bytes());
        packets.push(screen);
        Ok(())
    }

    pub(super) fn content_npc_key(&self, runtime_guid: u64) -> Option<u64> {
        if self.opening.as_ref().is_some_and(|opening| {
            opening
                .actors()
                .iter()
                .any(|actor| actor.guid == runtime_guid)
        }) {
            return None;
        }
        self.npc_visibility
            .as_ref()
            .and_then(|visibility| visibility.content_key(runtime_guid))
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
        if inner.starts_with(&[26, 0, 6, 0]) {
            let selected = npc_interaction::decode_dialogue_response_26_6(inner)
                .map_err(|_| invalid("invalid dialogue response"))?;
            npc_event(serde_json::json!({"event": "dialogue_choice", "value": selected}));
            // (R) The choice reply carries the row's response id (or its index);
            // the quest row is first, so accept either form.
            if selected == npc_interaction::QUEST_CHOICE_RESPONSE || selected == 0 {
                if let Some((runtime_guid, key, quest_id)) = self.pending_quest_choice.take() {
                    let held = std::mem::take(&mut self.pending_choice_packets);
                    if !held.is_empty() {
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
                        set_dialog_camera_target(packet, runtime_guid);
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
        // 45/7: the client says a goal it judges itself is met (W175). Only an open goal of
        // the character's own active quest can move; anything else changes nothing (R).
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
            npc_event(serde_json::json!({
                "event": "quest_reply",
                "quest_id": quest_id,
                "accepted": accept,
                "outcome": if result.changed {
                    if accept { "accepted" } else { "declined" }
                } else {
                    "rejected"
                }
            }));
            let mut out = Outcome::default();
            self.append_npc_quest_outcome(result, &mut out)?;
            // End the NPC dialog so the client leaves dialog camera and restores
            // the HUD (native run 200411 stayed in dialog view after accepting).
            out.replies.push(END_DIALOG_26_4.to_vec());
            return Ok(Some(out));
        }
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
                    Some(Profiles::load(
                        directory,
                        Some(next_state.profiles.clone()),
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
            let mut out = Outcome::default();
            self.append_npc_quest_outcome(quest, &mut out)?;
            out.replies.push(END_DIALOG_26_4.to_vec());
            return Ok(Some(out));
        }
        if inner.starts_with(&[26, 0, 23, 0, 0, 0]) {
            if inner.len() != 10 {
                return Err(invalid("invalid quest abandon request"));
            }
            let quest_id = u32::from_le_bytes(inner[6..10].try_into().unwrap());
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

        if inner.starts_with(&[26, 0, 10, 0]) {
            if inner.len() != 16 {
                return Err(invalid("invalid NPC menu selection"));
            }
            let runtime_guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
            let option_id = i32::from_le_bytes(inner[12..16].try_into().unwrap());
            let Some(key) = self.content_npc_key(runtime_guid) else {
                npc_event(serde_json::json!({
                    "event": "npc_select",
                    "guid": runtime_guid,
                    "content_key": serde_json::Value::Null,
                    "option": option_id,
                    "outcome": "none",
                    "reason": "unknown_runtime_guid"
                }));
                return Ok(None);
            };
            // Stage the menu/shop mutation on a cheap clone (shared content, owned
            // active state), then commit it to this staged Gameplay once the choice is
            // read. The gateway only keeps this clone when the whole request succeeds,
            // so a later failed child cannot leave the shared menu consumed.
            let Some(mut interaction) = self.npc_interaction.clone() else {
                npc_event(serde_json::json!({
                    "event": "npc_select",
                    "guid": runtime_guid,
                    "content_key": key,
                    "option": option_id,
                    "outcome": "none",
                    "reason": "npc_content_unavailable"
                }));
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
            self.npc_interaction = Some(interaction);
            let mut out = Outcome::default();
            let mut race_gate = None;
            let mut derby_gate = None;
            let mut fish_gate = None;
            let mut learn_trainer = None;
            let mut consume_menu = false;
            match selected {
                Ok(NpcSelection::Talk(mut context)) => {
                    let pending_before = self
                        .npc_quests
                        .as_ref()
                        .and_then(|engine| engine.state().pending_turnin);
                    if let Some(engine) = &mut self.npc_quests {
                        let quest = engine
                            .on_event(crate::quest_engine::QuestEvent::TalkToNpc { npc_id: key })?;
                        // A completed talk step speaks its own line (Shakey's bubble was
                        // blank in native run 215840).
                        if let Some(line) = engine.step_dialogue_for(&quest) {
                            context.body_string_id = Some(line);
                        }
                        self.append_npc_quest_outcome(quest, &mut out)?;
                    }
                    if let Some(quest_id) = pending_before {
                        let result = self
                            .npc_quests
                            .as_mut()
                            .expect("pending turn-in requires the NPC quest engine")
                            .begin_turnin(quest_id, key);
                        self.append_npc_quest_outcome(result, &mut out)?;
                    } else {
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
                                .begin_named_target_turnin(quest_id);
                            self.append_npc_quest_outcome(result, &mut out)?;
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
                        let mut held = held;
                        set_end_screen_actor(&mut held, runtime_guid);
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
                    set_dialog_camera_target(&mut dialogue, runtime_guid);
                    self.last_npc_dialogue =
                        Some((character.zone_id, runtime_guid, key, dialogue.clone()));
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
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": offer_outcome,
                        "reason": reason
                    }));
                }
                Ok(NpcSelection::Shop(window)) => {
                    out.replies.push(
                        merchant_packet(&window)
                            .map_err(|_| invalid("NPC merchant list could not be encoded"))?,
                    );
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "shop"
                    }));
                    npc_event(serde_json::json!({
                        "event": "shop_open",
                        "guid": runtime_guid,
                        "content_key": key,
                        "item_count": window.items.len()
                    }));
                }
                Ok(NpcSelection::Race) => {
                    race_gate = Some(key);
                    consume_menu = true;
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "race"
                    }));
                }
                Ok(NpcSelection::Derby) => {
                    derby_gate = Some(key);
                    consume_menu = true;
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "derby"
                    }));
                }
                Ok(NpcSelection::Fish) => {
                    fish_gate = Some(key);
                    consume_menu = true;
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "fish"
                    }));
                }
                Ok(NpcSelection::Learn(trainer)) => {
                    learn_trainer = Some(trainer);
                    consume_menu = true;
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "learn"
                    }));
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
                    set_end_screen_actor(&mut out.replies, runtime_guid);
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": if has_offer { "offer" } else { "none" },
                        "reason": if has_offer { serde_json::Value::Null } else { serde_json::json!("quest_turnin") }
                    }));
                }
                Err(crate::npc_interaction::NpcInteractionError::NeedsQuestEngine) => {
                    out.replies
                        .push(chat::system("Quest data is unavailable in this area."));
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "none",
                        "reason": "quest_engine_unavailable"
                    }));
                }
                Err(_) => {
                    npc_event(serde_json::json!({
                        "event": "npc_select",
                        "guid": runtime_guid,
                        "content_key": key,
                        "option": option_id,
                        "outcome": "none",
                        "reason": "selection_rejected"
                    }));
                    return Err(invalid("NPC menu choice is unavailable"));
                }
            }
            if let Some(gate) = race_gate {
                // (R) Ordinary entry: the NPC service proved range and that this exact option
                // was offered; the kart service decides admission. Staged, so a failure after
                // the job switch leaves the player exactly as they were.
                let busy = self.activity_busy();
                match self.kart_entry.as_ref().map(|kart| kart.admit(gate, busy)) {
                    Some(Ok(())) => {
                        let mut staged = self.clone();
                        out = staged.begin_kart_race(character)?;
                        *self = staged;
                    }
                    Some(Err(denied)) => out.replies.push(chat::system(denied.message())),
                    None => out.replies.push(chat::system(
                        crate::kart_gateway::EntryDenied::NoRace.message(),
                    )),
                }
            }
            if let Some(gate) = derby_gate {
                // (R) The NPC service proved range and that this exact option was offered;
                // the derby service decides admission. Staging keeps the job switch atomic.
                let busy = self.activity_busy();
                match self.derby.as_ref().map(|derby| derby.admit(gate, busy)) {
                    Some(Ok(())) => {
                        let mut staged = self.clone();
                        out = staged.begin_derby(character, now_ms)?;
                        *self = staged;
                    }
                    Some(Err(denied)) => out.replies.push(chat::system(denied.message())),
                    None => out.replies.push(chat::system(
                        crate::derby_gateway::DerbyEntryDenied::Unavailable.message(),
                    )),
                }
            }
            if let Some(trainer) = learn_trainer {
                // (R) Ordinary unlock: the NPC service proved range and that this exact option
                // was offered; `learn_job` decides the rest.
                let mut staged = self.clone();
                out = staged.learn_job(&trainer)?;
                // Leave the NPC dialog camera first (see the fishing choice).
                out.replies.insert(0, END_DIALOG_26_4.to_vec());
                if out.changed {
                    *self = staged;
                }
            }
            if let Some(gate) = fish_gate {
                // (R) Ordinary entry: the NPC service proved range and that this exact option
                // was offered; the fishing service decides admission. Staged, so a refusal
                // leaves the player exactly as they were.
                let mut staged = self.clone();
                out = staged.begin_fishing(gate, character)?;
                // Leave the NPC dialog camera first (it otherwise outlives a menu choice, as
                // the quest accept showed in native run 200411).
                out.replies.insert(0, END_DIALOG_26_4.to_vec());
                if out.changed {
                    *self = staged;
                }
            }
            if consume_menu && out.changed {
                if let Some(interaction) = &mut self.npc_interaction {
                    interaction.consume_active();
                }
            }
            return Ok(Some(out));
        }

        // 165/4 buy and 165/5 sell, plus the siblings (W172): 2 definitions, 8 list update,
        // 12 buy-back, 14 gift (always refused).
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
                        return Ok(Some(Outcome {
                            replies: outcome.replies,
                            changed: outcome.changed,
                            ..Outcome::default()
                        }));
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

        // Marketplace siblings (W172): wallet, status, SKU and code lists, account info, and the
        // refused order rows. Stateless and payment-free; `handle_marketplace` replies or errors.
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
            let Some(shop) = &mut self.npc_coin_shop else {
                return Ok(None);
            };
            let outcome = shop.handle_marketplace(inner)?;
            return Ok(Some(Outcome {
                replies: outcome.replies,
                ..Outcome::default()
            }));
        }

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
            return Ok(Some(Outcome {
                replies: outcome.replies,
                changed: outcome.changed,
                ..Outcome::default()
            }));
        }

        Ok(None)
    }
}

/// Point the client's dialog camera (`ControllerDialog`, switched in by
/// `ReplacePointerAtECAndDispatch` `008d2ba0`) at the NPC, so it zooms in on them
/// with the bubble to their left, as in the original. A zero target falls back to
/// the player (native run 230409, Frida `dialog_camera` hooks):
/// - 26/3 ShowDialog: dwords 2-3 of the leading four (`ShowDialogFromPacket`
///   `009f5ae0` reads object +0x18/+0x1c).
/// - 49/1 QuestInfo: the u64 after the reward bundle, i.e. the packet's last 14
///   bytes start with it (dispatcher `00c7cea2` passes object +0xe0/+0xe4).
/// The 49/13 end screen carries it at offset 6 (set where the turn-in is held).
pub(super) fn set_dialog_camera_target(packet: &mut [u8], npc_guid: u64) {
    let guid = npc_guid.to_le_bytes();
    if packet.starts_with(&[26, 0, 3, 0]) && packet.len() >= 20 {
        packet[12..20].copy_from_slice(&guid);
    } else if packet.starts_with(&[49, 0, 1, 0]) && packet.len() >= 14 {
        let at = packet.len() - 14;
        packet[at..at + 8].copy_from_slice(&guid);
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
    let mut choices: Vec<[u32; 5]> = quest_choice.into_iter().collect();
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

pub(super) fn merchant_packet(
    window: &MerchantWindow,
) -> Result<Vec<u8>, npc_interaction::NpcInteractionError> {
    let entries = window
        .items
        .iter()
        .map(|item| npc_interaction::MerchantEntryV2 {
            // (R) IDs and reference prices are carried in opaque fields by local convention.
            first_six_dwords: [
                item.item_id as u32,
                item.purchase_price_coins.unwrap_or(0),
                item.sell_price_coins.unwrap_or(0),
                0,
                0,
                0,
            ],
            flag_after_six: false,
            final_three_dwords: [0; 3],
            final_flag: false,
        })
        .collect();
    let npc = window.npc_id;
    // (R) 66/42 layout is recovered, while all semantic values remain opaque. The stable content
    // key identifies the merchant in this reconstructed listing.
    npc_interaction::encode_merchant_list_66_42(&npc_interaction::MerchantListV2 {
        initial_i16s: [0; 2],
        initial_dword: npc as u32,
        group: npc_interaction::MerchantGroupV2 {
            fields: [npc as u32, (npc >> 32) as u32, 0, 0, 0, 0, 0],
            entries,
        },
    })
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
        reward_bundle_body: vec![0; 69],
        trailing_dwords: [0; 2],
        trailing_float_bits: 0,
        trailing_flags: [false; 2],
    })
}

/// 49/13 offset 6 is the actor GUID, not the quest id: `ShowQuestEndScreen`
/// (`00a95420`) looks it up in the actor table (`00984960`) to expand
/// `[*player*]` in the end text and to play the NPC animation. With the quest
/// id there, native run 224000 showed `[*player*]` literally. Both live NPC
/// paths (Talk and the menu TurnInQuest choice) set the runtime actor here;
/// the reconnect replay patches the same field in `append_pending_turnin_screen`.
fn set_end_screen_actor(packets: &mut [Vec<u8>], actor_id: u64) {
    for packet in packets
        .iter_mut()
        .filter(|packet| packet.starts_with(&[49, 0, 13, 0]) && packet.len() >= 14)
    {
        packet[6..14].copy_from_slice(&actor_id.to_le_bytes());
    }
}
