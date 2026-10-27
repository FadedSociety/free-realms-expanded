//! Ride and pet glue for `Gameplay`; the rules live in `crate::mounts` and `crate::pets`. The pet
//! glue moved here verbatim from the hub (T458).
use super::*;
use crate::item_bar::use_wire;

impl Gameplay {
    /// World entry (T424): the pet roster and the owned-ride roster (168/5). The ride roster is
    /// always sent, so a character without rides gets count 0, which also replaces the captured
    /// ride rows of the login self data (`startup_selfdata` `pre_action_records`). Later
    /// ownership changes are sent by `settle_mounts`.
    pub(super) fn push_rosters(&mut self, out: &mut Outcome) {
        out.replies.push(self.pets.list_packet());
        if let Some(mounts) = &mut self.mounts {
            out.replies.push(mounts.list_packet(self.inventory.state()));
        }
    }

    /// W162: true while the rider sits on a ride (the emote rules refuse emotes then).
    pub fn is_mounted(&self) -> bool {
        self.mounts
            .as_ref()
            .is_some_and(crate::mounts::MountService::is_mounted)
    }

    /// W162: the summoned ride as other players must see it; `None` without one.
    pub fn ride_view(&self, guid: u64) -> Option<crate::mounts::RideView> {
        self.mounts.as_ref()?.view(guid)
    }

    /// T451: the boarded ride's speed, for the gateway's movement allowance; `None` on foot.
    pub fn mounted_speed(&self) -> Option<f32> {
        self.mounts.as_ref()?.mounted_speed()
    }

    /// (R) No per-zone riding data was recovered, so every zone allows rides; fights (an instance
    /// battle or a field enemy chasing the player), kart races and a knockout do not.
    fn ride_restrictions(&self) -> crate::mounts::RideRestrictions {
        let in_minigame = self.blocks(|gates| gates.ride);
        crate::mounts::RideRestrictions {
            zone_allows_riding: true,
            in_minigame,
            ability_active: false,
            knocked_out: self
                .encounter
                .as_ref()
                .is_some_and(|e| e.state().player_health == 0),
        }
    }

    fn ride_outcome(wire: crate::mounts::RideWire) -> Outcome {
        let mut out = Outcome::default();
        if let Some(notice) = wire.notice {
            out.replies.push(chat::system(notice));
        }
        out.replies.extend(wire.private);
        out.replies.extend(wire.shared.iter().cloned());
        out.nearby = wire.shared;
        out
    }

    /// W162: summon, board and put away a ride (family 168). Ownership is read from the
    /// inventory on every request.
    pub(super) fn handle_mounts(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        // T527: 168/14 consumes a saddle and writes the saved ride flags, so it is staged here
        // before the session-scoped mount service sees it.
        if let Some(crate::mounts::MountRequest::Upgrade {
            row_id,
            item_definition_id,
        }) = crate::mounts::decode_request(inner)?
        {
            return self.upgrade_ride(row_id, item_definition_id);
        }
        let restrictions = self.ride_restrictions();
        let Some(mounts) = self.mounts.as_mut() else {
            return Ok(None);
        };
        let context = crate::mounts::RideContext {
            guid: character.guid,
            inventory: self.inventory.state(),
            restrictions,
            pose: crate::mounts::RidePose {
                position: character.position,
                heading: character.rotation[0].atan2(character.rotation[2]),
            },
            now_ms,
        };
        Ok(mounts.handle(inner, &context)?.map(Self::ride_outcome))
    }

    /// T528: a saddle use from the bag arrives as 42/16 (check) and then 42/15 (use) — the client's
    /// `InventoryHandler:useItem` calls `EquipScreen.CheckImmediateActivationItem`, shows the
    /// dialog from the 38/30 reply and sends `UseImmediateActivationItem` on Okay. The check
    /// answers with the saddle's own name and description; the use sends 36/17 ExecuteClientLua
    /// (`00a30060`), which calls `BrowserV2:ShowUpgradableMounts(itemDefinitionId)`. Nothing is
    /// consumed either way: the saddle is spent by the upgrade itself (168/14). An instance that
    /// is not a saddle, or not held, keeps the item bar's, chest route's or inventory's handling;
    /// a saddle that is held but unusable (consumed, or riding is blocked) is refused with a
    /// notice and no packet.
    pub(super) fn use_upgrade_item(&self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        let Some((kind, instance)) = use_wire::decode_immediate_activation(inner) else {
            return Ok(None);
        };
        let Some(mounts) = self.mounts.as_ref() else {
            return Ok(None);
        };
        let definition = match mounts.upgrade_use_plan(
            self.inventory.state(),
            self.ride_restrictions(),
            instance,
        ) {
            Ok(definition) => definition,
            Err(crate::mounts::MountError::NotUpgradeItem) => return Ok(None),
            Err(error) => return Ok(Some(Self::ride_refusal(error))),
        };
        let mut out = Outcome::default();
        match kind {
            use_wire::ImmediateActivation::Check => {
                let Some(presentation) = self.inventory.presentation(definition).ok() else {
                    // A catalogue row the server cannot render sends no dialog (R).
                    return Ok(Some(Self::ride_refusal(
                        crate::mounts::MountError::NotUpgradeItem,
                    )));
                };
                out.replies
                    .push(use_wire::encode_immediate_activation_check(
                        instance,
                        0,
                        0,
                        presentation.name_id,
                        presentation.description_id,
                        true,
                    ));
            }
            use_wire::ImmediateActivation::Use => out
                .replies
                .push(crate::mounts::MountService::upgrade_picker(definition)),
        }
        Ok(Some(out))
    }

    /// A ride refusal that sends the owner one system-chat line and no packet.
    fn ride_refusal(error: crate::mounts::MountError) -> Outcome {
        Self::ride_outcome(crate::mounts::RideWire {
            notice: Some(crate::mounts::refusal_text(error)),
            ..crate::mounts::RideWire::default()
        })
    }

    /// T527: a Training Saddle upgrades one owned winged ride. The plan is pure, the saddle
    /// consume is transactional and the flag is recorded only after it succeeds, so a refusal
    /// changes nothing. Success replies: the item update, the roster (168/5) with that row's
    /// is_upgraded set, then 168/15 naming the row the client re-summons.
    fn upgrade_ride(
        &mut self,
        row_id: i32,
        item_definition_id: i32,
    ) -> io::Result<Option<Outcome>> {
        let restrictions = self.ride_restrictions();
        let Some(mounts) = self.mounts.as_mut() else {
            return Ok(None);
        };
        let plan = match mounts.upgrade_plan(
            self.inventory.state(),
            restrictions,
            row_id,
            item_definition_id,
        ) {
            Ok(plan) => plan,
            Err(error) => {
                let wire = mounts.refusal(self.inventory.state(), error);
                return Ok(Some(Self::ride_outcome(wire)));
            }
        };
        // The plan just found this instance; a failed consume is an internal error, and the
        // gateway discards this staged game (nothing changes).
        let consumed = self.inventory.consume(plan.item_instance_id, 1)?;
        mounts.commit_upgrade(plan.row_id);
        let mut replies = consumed.replies;
        replies.push(mounts.list_packet(self.inventory.state()));
        replies.push(crate::mounts::MountService::upgrade_reply(plan.row_id));
        Ok(Some(Outcome {
            replies,
            changed: true,
            ..Outcome::default()
        }))
    }

    /// W162 (R): a ride ends when the player zones, a fight or kart race starts, the player is
    /// knocked out or the ride item is gone; T424: the roster (168/5) follows every ownership
    /// change. The gateway calls this after every handled message, shared-field action and tick.
    pub fn settle_mounts(&mut self, character: &CharacterState, zoned: bool) -> Outcome {
        let restrictions = self.ride_restrictions();
        let Some(mounts) = self.mounts.as_mut() else {
            return Outcome::default();
        };
        let end = zoned || restrictions.in_minigame || restrictions.knocked_out;
        let ended = mounts.settle(character.guid, self.inventory.state(), end);
        let pose = crate::mounts::RidePose {
            position: character.position,
            heading: 0.0,
        };
        Self::ride_outcome(mounts.wire(&ended, pose))
    }

    /// S17: the saved roster comes from the character document (`Snapshot::pets`) before
    /// world entry; `pets_document` hands it back for every save of that document.
    pub fn adopt_pets(&mut self, pets: Option<serde_json::Value>) -> io::Result<()> {
        let content = crate::pets::PetService::bundled_content()?;
        self.pets = crate::pets::PetService::from_document(content, pets)?;
        Ok(())
    }

    pub fn pets_document(&self) -> Option<serde_json::Value> {
        self.pets.document_value()
    }

    /// W177: the active pet as other players need it for AddNpc replay.
    pub fn pet_view(&self, owner_guid: u64) -> Option<crate::pets::PetPresence> {
        self.pets.view(owner_guid)
    }

    fn pet_outcome(wire: crate::pets::PetWire) -> Outcome {
        let mut out = Outcome::default();
        out.changed = wire.changed;
        if let Some(notice) = wire.notice {
            out.replies.push(chat::system(notice));
        }
        out.replies.extend(wire.private);
        out.nearby = wire.shared;
        out
    }

    /// W177: summon, dismiss and UI mode (family 53). Ownership comes only from the saved
    /// roster loaded by `adopt_pets`; the actor GUID is derived from the authenticated
    /// character, never from request bytes.
    pub(super) fn handle_pets(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let minigame_active = self.blocks(|gates| gates.pets);
        let busy = crate::pets::PetBusy { minigame_active };
        let pets = &mut self.pets;
        let inventory = &mut self.inventory;
        // Pet need clocks are whole seconds and care timers (T485) milliseconds; the pet service
        // converts.
        let wire = pets.handle_at_ms(
            inner,
            character.guid,
            character.position,
            inventory,
            busy,
            now_ms,
        )?;
        Ok(wire.map(Self::pet_outcome))
    }

    /// T772: pet adoption items this transaction added (a coin buy, a grant) become pets at once,
    /// with the 53/5 roster after them.
    pub(super) fn adopt_gained_pets(
        &mut self,
        before: &crate::inventory::InventoryState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let replies = self
            .pets
            .adopt_gained(before, &mut self.inventory, now_ms / 1_000)?;
        out.changed |= !replies.is_empty();
        out.replies.extend(replies);
        Ok(())
    }

    /// T455: the summoned pet follows its owner after an accepted owner move or a world tick.
    /// Frames go to the owner (replies) and to viewers (nearby); a follow step never saves.
    pub(super) fn follow_pet(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) {
        let zone = character.zone_id;
        let terrain = self.terrain.clone();
        let ground = move |x: f32, z: f32| terrain.as_ref()?.height_at(zone, x, z);
        let wire = self
            .pets
            .follow_owner(character.guid, character.position, ground, now_ms);
        out.replies.extend(wire.private);
        out.nearby.extend(wire.shared);
    }

    /// T455: the world tick for a summoned pet (a still owner settles it); `None` without one.
    /// T485: also its care timers and live needs; only a feed's apply step marks a save.
    pub fn pet_tick(&mut self, character: &CharacterState, now_ms: u64) -> Option<Outcome> {
        self.pets.active_pet_id()?;
        let mut out = Outcome::default();
        self.follow_pet(character, now_ms, &mut out);
        let care = self.pets.care_tick(&mut self.inventory, now_ms);
        out.changed |= care.changed;
        out.replies.extend(care.private);
        // T488: a grooming session that times out plays its dry-off animation for the viewers.
        out.nearby.extend(care.shared);
        Some(out)
    }

    /// W177 (R): a pet does not zone with its owner; a fresh summon is required.
    pub fn settle_pets(&mut self, zoned: bool, now_ms: u64) -> io::Result<Outcome> {
        if !zoned || self.pets.active_pet_id().is_none() {
            return Ok(Outcome::default());
        }
        let mut pets = self.pets.clone();
        let feed_end = pets.care_end_packet();
        pets.rebase_clock(now_ms / 1_000);
        let transition = pets.dismiss(now_ms / 1_000)?;
        self.pets = pets;
        let mut out = Self::pet_outcome(self.pets.wire(transition));
        out.replies.extend(feed_end);
        Ok(out)
    }
}
