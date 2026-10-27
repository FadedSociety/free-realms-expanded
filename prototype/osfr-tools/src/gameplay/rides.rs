//! Ride and pet glue for `Gameplay`; the rules live in `crate::mounts` and `crate::pets`. The pet
//! glue moved here verbatim from the hub (T458).
use super::*;
use crate::item_bar::use_wire;

mod pet_treasure;

/// F778: the shared client `53/*` pet request bodies, the same file the gateway's session tests
/// and `tests/pet_routing.rs` include (test-only).
#[cfg(test)]
#[path = "../bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;

/// The terrain height under (x, z) in `zone`: the pet's follow step and its chest stand on it.
fn terrain_height(
    terrain: Option<std::sync::Arc<crate::terrain::Terrain>>,
    zone: u32,
) -> impl Fn(f32, f32) -> Option<f32> {
    move |x, z| terrain.as_ref()?.height_at(zone, x, z)
}

impl Gameplay {
    /// World entry (T424): the pet roster and the owned-ride roster (168/5). The ride roster is
    /// always sent, so a character without rides gets count 0, which also replaces the captured
    /// ride rows of the login self data (`startup_selfdata` `pre_action_records`). Later
    /// ownership changes are sent by `settle_mounts`. T487: the loaded signal then brings back the
    /// pet a zone change left behind, after the rosters the client reads it from.
    pub(super) fn enter_world(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) {
        out.replies.push(self.pets.list_packet());
        if let Some(mounts) = &mut self.mounts {
            out.replies.push(mounts.list_packet(self.inventory.state()));
        }
        self.resummon_pet(character, now_ms, out);
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

    /// T532: the gateway's movement speed policy: a boarded ride's speed plus a live happy pet's
    /// bonus. `None` (no gameplay yet) is the plain on-foot (R) policy the gateway literal built.
    pub fn speed_modifiers(gameplay: Option<&Gameplay>) -> crate::movement::SpeedModifiers {
        let Some(gameplay) = gameplay else {
            return crate::movement::SpeedModifiers::riding(None);
        };
        crate::movement::SpeedModifiers::riding(gameplay.mounted_speed())
            .with_speed_bonus(gameplay.happy_pet_speed_bonus())
    }

    /// T532: the live happy-pet bonus for the movement allowance; 0 without the buff.
    fn happy_pet_speed_bonus(&self) -> f32 {
        self.pets.happy_speed_bonus().unwrap_or(0.0)
    }

    /// T532: one happy-speed-buff step. The buff-bar row goes to the owner, and the character
    /// speed stat follows so the client's own motion matches the server's allowance (the stat is
    /// the live bonus, so a row that just ended restores the on-foot value). A ride's own 38/7
    /// owns the speed while seated, so the stat waits for the next refresh on foot.
    fn sync_happy_buff(&mut self, character: &CharacterState, now_ms: u64, out: &mut Outcome) {
        let Some(rows) = self.pets.happy_buff_tick(now_ms) else {
            return;
        };
        out.replies.extend(rows);
        if self.is_mounted() {
            return;
        }
        if let Some(stat) = self
            .mounts
            .as_ref()
            .and_then(|mounts| mounts.speed_stat(character.guid, self.happy_pet_speed_bonus()))
        {
            out.replies.push(stat);
        }
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
    /// character, never from request bytes. T532: the happy-speed buff follows the pet.
    /// T788: an accepted 53/7 also credits a `perform_pet_trick` quest objective.
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
        let Some(wire) = wire else {
            return Ok(None);
        };
        // T788: only an accepted 53/7 carries the trick, so a refusal or a 53/8 credits no quest.
        let accepted_trick = wire.accepted_trick;
        let mut out = Self::pet_outcome(wire);
        self.sync_happy_buff(character, now_ms, &mut out);
        self.pet_trick_quest_event(accepted_trick, &mut out)?;
        Ok(Some(out))
    }

    /// T772: adoption items an outcome granted become pets at once, with the 53/5 roster after
    /// them. The one settlement for a purchase, a buy-back and `realm give`; a refusal granted
    /// nothing and costs one empty check.
    pub(super) fn settle_granted_pets(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        if out.granted_items.is_empty() {
            return Ok(());
        }
        let granted = std::mem::take(&mut out.granted_items);
        let replies = self
            .pets
            .adopt_granted(&granted, &mut self.inventory, now_ms / 1_000)?;
        out.changed |= !replies.is_empty();
        out.replies.extend(replies);
        Ok(())
    }

    /// T772/F792: `realm give <item> [count]`. A pet adoption item beyond the roster's room is
    /// refused before the grant, so no category-200 item is stranded in the bag.
    pub(super) fn realm_give(
        &mut self,
        item: &str,
        rest: &[&str],
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<bool> {
        let (Ok(item), count) = (
            item.parse::<i32>(),
            rest.first()
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(1)
                .clamp(1, 999),
        ) else {
            out.replies
                .push(chat::system("Usage: realm give <item id> [count]"));
            return Ok(true);
        };
        let room = self.pets.room();
        if count as usize > room && self.pets.is_adoption_item(item) {
            out.replies.push(chat::system(&format!(
                "The pet roster has room for {room} more; giving {count} would leave pet items hidden in the bag."
            )));
            return Ok(true);
        }
        match self.grant_reward(0, &[(item, count)], out) {
            Ok(()) => {
                self.settle_granted_pets(now_ms, out)?;
                out.replies
                    .push(chat::system(&format!("Gave {count} x item {item}.")));
            }
            Err(error) => out.replies.push(chat::system(&format!(
                "Could not give item {item}: {error}"
            ))),
        }
        Ok(true)
    }

    /// T487: the pet a zone change left behind comes back through the normal summon path, beside
    /// its owner, once the destination allows pets. A pets-blocking activity (a race, a battle)
    /// keeps it for the first update or tick after the block ends. The transition is session-only,
    /// so it never marks a save; a summon the server cannot place is logged for that player and
    /// never an `Err` out of the loaded path or the world tick. Nothing is remembered on almost
    /// every tick, so the memory is checked before the occupation table is asked.
    fn resummon_pet(&mut self, character: &CharacterState, now_ms: u64, out: &mut Outcome) -> bool {
        if self.pets.remembered_pet_id().is_none() {
            return false;
        }
        let busy = crate::pets::PetBusy {
            minigame_active: self.blocks(|gates| gates.pets),
        };
        let guid = character.guid;
        match self.pets.resummon(
            crate::pets::PetService::pet_guid(guid),
            guid,
            character.position,
            busy,
            now_ms / 1_000,
        ) {
            Ok(Some(wire)) => {
                out.replies.extend(wire.private);
                out.nearby.extend(wire.shared);
                true
            }
            Ok(None) => false,
            Err(error) => {
                eprintln!("pet: no pet came back for character {guid}: {error}");
                false
            }
        }
    }

    /// T455: the summoned pet follows its owner after an accepted owner move or a world tick.
    /// Frames go to the owner (replies) and to viewers (nearby); a follow step never saves.
    /// T487: with no pet out, the first accepted move is also where a remembered one returns.
    pub(super) fn follow_pet(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) {
        if self.pets.active_pet_id().is_none() && !self.resummon_pet(character, now_ms, out) {
            return;
        }
        let ground = terrain_height(self.terrain.clone(), character.zone_id);
        let wire = self
            .pets
            .follow_owner(character.guid, character.position, ground, now_ms);
        out.replies.extend(wire.private);
        out.nearby.extend(wire.shared);
    }

    /// T455: the world tick for a summoned pet (a still owner settles it); `None` without one.
    /// T485: also its care timers and live needs; a feed's apply step and T488 a grooming
    /// session's end mark a save.
    /// T487: the first tick after a race or battle ends is also where a remembered pet returns
    /// (`follow_pet` covers both cases). T533: and its treasure hunt, which also runs for a chest
    /// still waiting after its pet is gone; a find marks the save (the cooldown). T532: the
    /// happy-speed buff ticks too, so a mood drop or dismissal ends it here. `None` means there is
    /// still nothing to send.
    pub fn pet_tick(&mut self, character: &CharacterState, now_ms: u64) -> Option<Outcome> {
        let mut out = Outcome::default();
        self.follow_pet(character, now_ms, &mut out);
        self.sync_happy_buff(character, now_ms, &mut out);
        if self.pets.active_pet_id().is_some() {
            let care = self.pets.care_tick(&mut self.inventory, now_ms);
            out.changed |= care.changed;
            out.replies.extend(care.private);
            // T488: a grooming session that times out plays its dry-off animation for the viewers.
            out.nearby.extend(care.shared);
        } else if !self.pets.has_treasure() {
            return (!out.replies.is_empty()).then_some(out);
        }
        self.tick_pet_treasure(character, now_ms, &mut out);
        Some(out)
    }

    /// F778: the world tick for the gateway branches that run `tick` on a clone (the opening
    /// scene, a kart race loading, the derby, soccer). The pet's follow step and care timers run
    /// there too, so a feed started in the opening zone still eats and ends. The common path runs
    /// `pet_tick` alone, without `tick`.
    pub fn tick_with_pets(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = self.tick(character, now_ms)?;
        if let Some(pet) = self.pet_tick(character, now_ms) {
            out.changed |= pet.changed;
            out.replies.extend(pet.replies);
            out.nearby.extend(pet.nearby);
        }
        Ok(out)
    }

    /// W177 (R): a pet does not zone with its owner; T487 remembers it so the destination's
    /// loaded signal (`enter_world`) or the first update there can summon it again. T532: the
    /// happy pet's buff ends with the pet, so the zone change also drops its speed.
    pub fn settle_pets(
        &mut self,
        character: &CharacterState,
        zoned: bool,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        if !zoned {
            return Ok(Outcome::default());
        }
        // T533: the chest does not zone either; the cooldown it started stays.
        let chest = self.pets.clear_treasure();
        if self.pets.active_pet_id().is_none() {
            return Ok(Outcome {
                replies: chest,
                ..Outcome::default()
            });
        }
        let mut pets = self.pets.clone();
        let care_end = pets.care_end_packets();
        pets.rebase_clock(now_ms / 1_000);
        let transition = pets.zone_away(now_ms / 1_000)?;
        self.pets = pets;
        let mut out = Self::pet_outcome(self.pets.wire(transition));
        out.replies.extend(care_end);
        out.replies.splice(0..0, chest);
        self.sync_happy_buff(character, now_ms, &mut out);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishing_service::FishingSpots;
    use crate::gameplay::occupation::Occupation;
    use crate::pets::{PetEquipmentSlot, PetService};
    use pet_wire::{feed, groom, summon};

    /// The opening package's catalogue: bowl 8 (slot 204), Simple Pet Food 1835 (slot 205),
    /// shampoo 1836 (slot 200) and brush 56778 (slot 203), all type 4 and class 73, so the bundled
    /// Char (model class 73) takes them.
    const BOWL_DEF: i32 = 8;
    const FOOD_DEF: i32 = 1_835;
    const SHAMPOO_DEF: i32 = 1_836;
    const BRUSH_DEF: i32 = 56_778;
    /// A fishing guide in the bundled `fishing_spots.json`.
    const REED: u64 = 9_000_000_181;

    fn who() -> CharacterState {
        CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        }
    }

    /// A 53/20 PetUtilityNotify for `pet`: the kind and four zero words.
    fn notify(pet: u32, kind: u32) -> Vec<u8> {
        let mut bytes = vec![53, 0, 20];
        for value in [pet, kind, 0, 0, 0, 0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes
    }

    fn first_owned(game: &Gameplay, definition: i32) -> i32 {
        game.inventory_state()
            .items
            .iter()
            .find(|item| item.definition_id == definition && item.count > 0)
            .unwrap_or_else(|| panic!("no owned item {definition}"))
            .id
    }

    /// F778: `Gameplay::handle_pets` computes the pets busy flag from the occupation table, which
    /// the service-level pet_care suite never exercises. A player fishing (a real
    /// `Occupation::Fishing`), with a bowl and food fitted and the pet out, gets kind 6 and no
    /// state change; once free the same feed begins.
    #[test]
    fn a_busy_player_cannot_feed_and_the_refusal_changes_nothing() {
        let reference = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.inventory.grant(0, BOWL_DEF, 1).unwrap();
        game.inventory.grant(0, FOOD_DEF, 2).unwrap();
        let content = PetService::bundled_content().unwrap();
        let mut pets = PetService::new(content.clone()).unwrap();
        let definition = content.definition(1_000_015).unwrap();
        let pet = pets.adopt_vendor_grant(1, &definition, 0).unwrap();
        for (definition, slot) in [(BOWL_DEF, 5), (FOOD_DEF, 6)] {
            let item = first_owned(&game, definition);
            pets.equip_item(pet, item, PetEquipmentSlot::Utility(slot), &game.inventory)
                .unwrap();
        }
        game.adopt_pets(pets.document_value()).unwrap();
        game.initialize_world(&who(), 0).unwrap();
        game.handle_in_world(&summon(pet), &who(), 1_000).unwrap();
        assert_eq!(game.pet_view(1).map(|view| view.pet_id), Some(pet));

        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let spots = FishingSpots::load(&directory).unwrap().unwrap();
        let rewards = game.reward_snapshot();
        game.fishing
            .enter(&spots, REED, 1, 0, who().position, false, 1, rewards)
            .unwrap();
        assert!(
            game.blocks(|gates| gates.pets),
            "fishing closes the pets gate"
        );
        let inventory = game.inventory_state().clone();
        let roster = game.pets_document();
        let refused = game.handle_in_world(&feed(pet), &who(), 2_000).unwrap();
        assert_eq!(
            refused.replies,
            vec![notify(pet, 6)],
            "the refusal is 53/20 unlock"
        );
        assert!(!refused.changed);
        assert_eq!(game.inventory_state(), &inventory, "nothing was spent");
        assert_eq!(game.pets_document(), roster, "the roster is unchanged");

        game.fishing.reset_all();
        let started = game.handle_in_world(&feed(pet), &who(), 3_000).unwrap();
        assert_eq!(
            started.replies,
            vec![notify(pet, 1)],
            "free again, the feed begins"
        );
    }

    /// T488/F796: a grooming session is an `Occupation`, so the shared busy table refuses another
    /// activity while it runs. The session itself keeps the pets gate open (its 53/22 strokes are
    /// pet packets), and its timeout frees the player again.
    #[test]
    fn a_grooming_session_closes_the_start_gate_and_keeps_the_pets_gate_open() {
        let reference = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.inventory.grant(0, SHAMPOO_DEF, 2).unwrap();
        game.inventory.grant(0, BRUSH_DEF, 1).unwrap();
        let content = PetService::bundled_content().unwrap();
        let mut pets = PetService::new(content.clone()).unwrap();
        let definition = content.definition(1_000_015).unwrap();
        let pet = pets.adopt_vendor_grant(1, &definition, 0).unwrap();
        for (definition, slot) in [(SHAMPOO_DEF, 1), (BRUSH_DEF, 4)] {
            let item = first_owned(&game, definition);
            pets.equip_item(pet, item, PetEquipmentSlot::Utility(slot), &game.inventory)
                .unwrap();
        }
        game.adopt_pets(pets.document_value()).unwrap();
        game.initialize_world(&who(), 0).unwrap();
        game.handle_in_world(&summon(pet), &who(), 1_000).unwrap();
        assert!(!game.activity_busy(), "a summoned pet is not an activity");

        let started = game.handle_in_world(&groom(pet), &who(), 2_000).unwrap();
        assert!(
            started
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[53, 0, 20])),
            "the groom starts"
        );
        assert!(
            game.is_occupied(Occupation::Groom),
            "the groom occupies the player"
        );
        assert!(
            game.activity_busy(),
            "another activity must refuse to start"
        );
        assert!(
            !game.blocks(|gates| gates.pets),
            "pet care itself stays open"
        );

        // The session's timeout frees the player.
        let ended = game.pet_tick(&who(), 62_000).unwrap();
        assert!(
            ended
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[53, 0, 20])),
            "the timeout ends it"
        );
        assert!(!game.is_occupied(Occupation::Groom));
        assert!(!game.activity_busy(), "the player is free again");
    }
}
