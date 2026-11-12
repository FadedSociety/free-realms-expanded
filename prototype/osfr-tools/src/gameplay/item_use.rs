//! The item bar and the consumable effect stack on `Gameplay` (W146, T474).
//!
//! The stack (`consumable_effects::EffectStack`) is session-only: it is not in the save, so a
//! relog starts with no effects and no buff rows (R, both reference servers do the same).
//! Packets for the owner go to `Outcome.replies`, packets neighbours must see to
//! `Outcome.nearby`; the gateway fans those out through the interest set.
use super::*;
use crate::consumable_effects::{self, Emit, Refusal, Spec};

/// C3: the high word of a shared potion's group-heal key ('POTI'); kill keys carry enemy guids.
const SHARED_POTION: u64 = 0x504f_5449_0000_0000;

/// `Outcome.changed` follows what a stranger can see (R): the AddPc snapshot a late joiner gets
/// must be refreshed, so a visual change marks the character changed. That is one save per apply
/// or expiry, which is rare. A buff row alone changes nothing visible.
fn outcome_of(emit: Emit) -> Outcome {
    Outcome {
        changed: !emit.to_nearby.is_empty(),
        replies: emit.to_self,
        nearby: emit.to_nearby,
        ..Outcome::default()
    }
}

impl Gameplay {
    /// The live effects, for the spawn snapshot of a late joiner.
    pub fn consumable_effects(&self) -> &consumable_effects::EffectStack {
        &self.consumable_effects
    }

    /// Applies a treat, cake or transformation to this player. A refusal changes nothing, so a
    /// caller that spends an item stages with `EffectStack::admits` first and spends only on `Ok`.
    pub fn apply_consumable_effect(
        &mut self,
        guid: u64,
        position: [f32; 3],
        spec: Spec,
        now_ms: u64,
    ) -> Result<Outcome, Refusal> {
        let (_, emit) = self
            .consumable_effects
            .apply(guid, position, spec, now_ms)?;
        Ok(outcome_of(emit))
    }

    /// Expires due effects. With an empty stack this allocates nothing and touches nothing.
    pub(super) fn tick_consumable_effects(&mut self, guid: u64, now_ms: u64, out: &mut Outcome) {
        let emit = self.consumable_effects.tick(guid, now_ms);
        if !emit.is_empty() {
            self.sync_lure(guid);
            out.absorb(outcome_of(emit));
        }
    }

    /// C3: a lure that ended (on time or cancelled) comes off the line. There is no session to
    /// clear outside fishing, so the error is the no-op it looks like.
    fn sync_lure(&mut self, guid: u64) {
        if !self.consumable_effects.has_lure() {
            let _ = self.fishing.set_lure(guid, None);
        }
    }

    /// C3: a lure item pressed, checked or used while the player is not fishing is refused with a
    /// reason, before the bar spends it: the lure goes on a fishing session's line.
    fn lure_refusal(&self, inner: &[u8]) -> Option<Outcome> {
        if !self.fishing.is_empty() {
            return None;
        }
        let pressed = inner.starts_with(&[36, 0, 10, 0]);
        let instance = if pressed {
            if inner.get(4..8) != Some(&2_i32.to_le_bytes()[..]) {
                return None;
            }
            let slot = i32::from_le_bytes(inner.get(8..12)?.try_into().ok()?);
            let bound = self.item_bar.bindings().iter().find(|b| b.slot == slot)?;
            bound.item_instance_id
        } else {
            crate::item_bar::use_wire::decode_immediate_activation(inner)?.1
        };
        let item = self.inventory.state().items.iter().find(|i| i.id == instance)?;
        crate::fishing_service::FishingService::lure_key_for_item(item.definition_id)?;
        let mut replies = vec![crate::chat::system("Use a lure while you are fishing.")];
        if pressed {
            replies.insert(0, crate::abilities::unsupported_cast_failure());
        }
        Some(Outcome {
            replies,
            ..Outcome::default()
        })
    }

    /// 35/33, the buff bar's cancel (client sender `00934740`, one i32 id). Only an effect this
    /// stack owns ends: an unknown id, or a job or power-up tag, is ignored and sends nothing (R,
    /// OSFR also ignores it). Cancelling Super Shield would also have to clear its caster buff
    /// (`encounter/power_up_use.rs`); that is not wired here. Activities never block a cancel.
    pub(super) fn handle_effect_strip(
        &mut self,
        inner: &[u8],
        guid: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(id) = consumable_effects::parse_strip(inner)? else {
            return Ok(None);
        };
        let emit = self.consumable_effects.strip(guid, id).unwrap_or_default();
        self.sync_consumable_buff();
        self.sync_lure(guid);
        Ok(Some(outcome_of(emit)))
    }

    /// C3: a restore row: its cast on the user (seen by neighbours too), its health and energy at
    /// once or over time through the encounter engine, and, shared, the gateway's group-heal route
    /// for the health part (the user's group in the same place). The key is this use's own, so
    /// every shared potion heals once. The energy half is the user's alone (R): the group route
    /// carries health only.
    fn apply_restore(
        &mut self,
        restore: crate::item_bar::Restore,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) {
        if restore.cast_effect_id > 0 {
            let cast = crate::abilities::effect_packets::encode_play_composite_effect(
                character.guid,
                restore.cast_effect_id,
                character.position,
            );
            out.nearby.push(cast.clone());
            out.replies.push(cast);
        }
        let Some(engine) = self.encounter.as_mut() else {
            return;
        };
        let (health, energy) = (restore.health_percent, restore.energy_percent);
        // The part at once (all of it without `over_ms`), then the rest a step a second.
        let now = |percent: u32| {
            if restore.over_ms == 0 {
                percent
            } else {
                restore.instant_percent.min(percent)
            }
        };
        if health > 0 && now(health) > 0 {
            out.replies.extend(engine.heal_player_percent(now(health)));
        }
        if energy > 0 && now(energy) > 0 {
            out.replies.extend(engine.restore_player_energy_percent(now(energy)));
        }
        if restore.over_ms > 0 {
            if health > now(health) {
                engine.heal_player_over_time(health - now(health), restore.over_ms, now_ms);
            }
            if energy > now(energy) {
                let rest = energy - now(energy);
                engine.restore_player_energy_over_time(rest, restore.over_ms, now_ms);
            }
        }
        if restore.shared && health > 0 {
            out.group_heal = Some(crate::encounter::GroupHeal {
                percent: health,
                kill: (SHARED_POTION | character.guid, now_ms),
            });
        }
    }

    /// C3: the encounter engine's share of the live combat buff: Attack (stat 14) deals its value
    /// in percent more damage and Defense (stat 18) takes its value in percent less, until the
    /// buff ends (the engine drops it on time). MaxHealth and MaxMana have no engine hook, so only
    /// their buff row shows. Run after every apply, cancel and scene load (the engine clears its
    /// buffs on a knock-out, a job change and a session reset).
    fn sync_consumable_buff(&mut self) {
        let Some(engine) = self.encounter.as_mut() else {
            return;
        };
        let percent = |value: i32| u32::try_from(value).unwrap_or(0);
        let (damage, taken, until) = match self.consumable_effects.combat_buff() {
            Some((14, value, until)) => (percent(value), 0, until),
            Some((18, value, until)) => (0, percent(value), until),
            _ => (0, 0, 0),
        };
        engine.set_consumable_buff(damage, taken, until);
    }

    /// W146: the four item-bar slots (38/25 at world entry), then the live effects again (R):
    /// the client rebuilds its own actor when it zones, so the model, size, auras and buff rows
    /// must follow every scene load. Empty after a fresh login.
    pub(super) fn append_item_bar(
        &mut self,
        guid: u64,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<()> {
        out.replies
            .extend(self.item_bar.refresh(&self.inventory, now_ms)?);
        out.replies
            .extend(self.consumable_effects.resend_to_self(guid, now_ms));
        self.sync_consumable_buff();
        Ok(())
    }

    /// W146: item-bar requests (42/6, 42/9, 42/15 and 42/16 for bar items, 36/10 on bar 2). They
    /// are routed before the chest route and the shared 42 branch, in both the opening path (after
    /// its movie block) and the no-opening path; a 36/10 on any other bar still reaches
    /// `unsupported_cast`.
    ///
    /// A single-use item is consumed and its effect applied in this one call, so the saved state
    /// never holds one without the other. A heal row needs the encounter engine's health; with
    /// none the player counts as alive and the heal row is refused (R), while an aura, transform
    /// or scale row still goes through the effect stack.
    ///
    /// T461: a boombox press is never spent. It is refused (36/1, nothing changed) while the
    /// player is mounted (the Mounts guide's wiki rule), busy (the start gate of the occupation
    /// table) or outside the open world (R), and otherwise leaves the definition in
    /// `Outcome::boombox` for the host to place the box after the commit.
    ///
    /// L088: a battle item flies at the clicked enemy, else the nearest in reach; with none in reach
    /// (or the clicked one out of it) the bar refuses it and nothing is spent. `field` is the shared
    /// field when the press was aimed at one of its enemies.
    pub(super) fn handle_item_bar(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
        mut field: Option<&mut crate::encounter::SharedFieldState>,
    ) -> io::Result<Option<Outcome>> {
        // T528: a saddle is not a bar item; its 42/16/42/15 pair opens the client's own ride
        // upgrade picker (36/17) and changes nothing.
        if let Some(out) = self.use_upgrade_item(inner)? {
            return Ok(Some(out));
        }
        if let Some(refused) = self.lure_refusal(inner) {
            return Ok(Some(refused));
        }
        let aimed = inner
            .starts_with(&[36, 0, 10, 0])
            .then(|| crate::item_bar::use_wire::decode_client_request_start_ability(inner))
            .flatten()
            .and_then(|request| match request.target {
                crate::item_bar::use_wire::Target::Guid(guid)
                    if guid != 0 && guid != character.guid =>
                {
                    Some(guid)
                }
                _ => None,
            });
        let position = character.position;
        // Only an item-bar press, check or use can spend a battle item: nothing else pays the scan.
        let item_request = inner.starts_with(&[36, 0, 10, 0])
            && inner.get(4..8) == Some(&2_i32.to_le_bytes()[..])
            || inner.starts_with(&[42, 0, 15, 0])
            || inner.starts_with(&[42, 0, 16, 0]);
        let can_strike = match (self.encounter.as_mut(), field.as_deref_mut()) {
            _ if !item_request => false,
            (Some(engine), Some(field)) => engine
                .with_shared_fields(field, character.guid, |engine| {
                    Ok(engine.battle_item_target(position, aimed))
                })?
                .is_some(),
            (Some(engine), None) => engine.battle_item_target(position, aimed).is_some(),
            (None, _) => false,
        };
        let player = crate::item_bar::Player {
            guid: character.guid,
            alive: self
                .encounter
                .as_ref()
                .is_none_or(|engine| engine.state().player_health > 0),
            can_place_box: character.zone_id == crate::world::OPEN_WORLD_ZONE
                && !self.is_mounted()
                && !self.blocks(|gates| gates.start),
            can_heal: self.encounter.is_some(),
            transformed: self.consumable_effects.has_transform(),
            can_strike,
        };
        let Some(reply) = self
            .item_bar
            .handle(inner, &mut self.inventory, &player, now_ms)?
        else {
            return Ok(None);
        };
        let mut out = Outcome {
            replies: reply.replies,
            changed: reply.changed,
            boombox: reply.boombox,
            ..Outcome::default()
        };
        if reply.heal_percent > 0 {
            if let Some(encounter) = self.encounter.as_mut() {
                out.replies
                    .extend(encounter.heal_player_percent(reply.heal_percent));
            }
        }
        // L085: the rest of a heal over time comes a step a second from the engine's tick.
        if let (Some((percent, over_ms)), Some(encounter)) =
            (reply.heal_over, self.encounter.as_mut())
        {
            encounter.heal_player_over_time(percent, over_ms, now_ms);
        }
        if let Some(restore) = reply.restore {
            self.apply_restore(restore, character, now_ms, &mut out);
        }
        if let Some(spec) = reply.effect {
            // The item bar admitted this exact spec in the same call, before spending the item,
            // and `apply` runs the same pure check: an error here is unreachable.
            if let Ok(applied) =
                self.apply_consumable_effect(character.guid, character.position, spec, now_ms)
            {
                out.absorb(applied);
            }
            self.sync_consumable_buff();
        }
        // C3: a lure goes on the fishing session's line (`lure_refusal` saw one open).
        let lure = reply
            .used_definition
            .and_then(crate::fishing_service::FishingService::lure_key_for_item);
        if let Some(key) = lure {
            let _ = self.fishing.set_lure(character.guid, Some(key));
        }
        if let Some(strike) = reply.strike {
            let throw = move |engine: &mut crate::encounter::EncounterEngine| {
                engine.use_battle_item(strike, aimed, position, now_ms)
            };
            let result = match (self.encounter.as_mut(), field.as_deref_mut()) {
                (Some(engine), Some(field)) => {
                    Some(engine.with_shared_fields(field, character.guid, throw)?)
                }
                (Some(engine), None) => Some(throw(engine)?),
                (None, _) => None,
            };
            if let Some(result) = result {
                self.apply_encounter(result, &mut out)?;
            }
        }
        // L092: a scare sends the enemies around the player running (the shared field's when the
        // press was aimed at one of them).
        if let Some(scare) = reply.scare {
            let run = move |engine: &mut crate::encounter::EncounterEngine| {
                Ok(engine.scare(scare.radius, scare.flee_ms, position, now_ms))
            };
            let result = match (self.encounter.as_mut(), field) {
                (Some(engine), Some(field)) => {
                    Some(engine.with_shared_fields(field, character.guid, run)?)
                }
                (Some(engine), None) => Some(run(engine)?),
                (None, _) => None,
            };
            if let Some(result) = result {
                self.apply_encounter(result, &mut out)?;
            }
        }
        // T557: a cake is placed by the host after the commit; this press spent nothing.
        out.shared_cake = reply
            .place
            .map(|row| crate::placed_cakes::Action::Place { row });
        // T789: only a single-use item that was actually spent credits a `use_item` objective.
        self.emit_item_use(reply.used_definition, &mut out)?;
        Ok(Some(out))
    }

    /// L088: an item-bar press aimed at a shared field enemy, under the same interest restriction
    /// as a job ability; the field is lent to the encounter engine for the check and the throw.
    pub(super) fn throw_battle_item(
        &mut self,
        action: &crate::encounter::SharedFieldAction,
        field: &mut crate::encounter::SharedFieldState,
        known: &std::collections::BTreeSet<u64>,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let presence = field.restrict_to(known);
        let result = self.handle_item_bar(&action.request(), character, now_ms, Some(field));
        field.restore_presence(presence);
        let mut out = result?.unwrap_or_default();
        self.sync_tracked_target(character, &mut out);
        Ok(out)
    }

    /// T557: the host could not place the cake `definition` a press named (the world holds its
    /// cap of cakes). The press never happened: its recast is taken back, and the player gets the
    /// bar's refusal and the bar as it stands.
    pub fn refuse_cake_place(&mut self, definition: i32, now_ms: u64) -> io::Result<Vec<Vec<u8>>> {
        let mut replies = vec![crate::abilities::unsupported_cast_failure()];
        replies.extend(
            self.item_bar
                .cancel_use(definition, &self.inventory, now_ms)?,
        );
        Ok(replies)
    }

    /// T461: the catalogue row of a boombox definition a press named, for the host to place. The
    /// catalogue's shared pointer, so the host never copies the independent clip tables.
    pub fn boombox_row(&self, definition: i32) -> Option<std::sync::Arc<crate::boombox::Boombox>> {
        self.item_bar.boombox(definition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::InventoryItem;

    const TOTEM: i32 = 8185;
    const REED: u64 = 9_000_000_181; // a fishing guide in the bundled `fishing_spots.json`

    fn who() -> CharacterState {
        CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-440.0, 0.8, 1292.0],
            rotation: [1.0, 0.0, 0.0],
        }
    }

    fn press() -> Vec<u8> {
        let mut bytes = [36_u16.to_le_bytes(), 10_u16.to_le_bytes()].concat();
        for field in [2_i32, 0, 0] {
            bytes.extend(field.to_le_bytes());
        }
        bytes.extend(1_u64.to_le_bytes());
        bytes
    }

    /// A player who holds a box on bar slot 0 and is fishing (a busy occupation), or stands
    /// outside the open world, cannot place it; the refusal is a bare 36/1 that changes and
    /// stamps nothing, so the press works as soon as the player is free.
    #[test]
    fn a_busy_or_away_player_is_refused_without_a_recast_stamp() {
        let reference = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        let mut saved = Gameplay::load(&reference, None).unwrap().state();
        let id = saved
            .inventory
            .items
            .iter()
            .map(|i| i.id)
            .max()
            .unwrap_or(0)
            + 1000;
        saved.inventory.items.push(InventoryItem {
            definition_id: TOTEM,
            tint: 0,
            id,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        let mut game = Gameplay::load(&reference, Some(saved)).unwrap();
        game.initialize_world(&who(), 0).unwrap();
        let mut assign = [42_u16.to_le_bytes(), 6_u16.to_le_bytes()].concat();
        assign.extend(0_i32.to_le_bytes());
        assign.extend(id.to_le_bytes());
        game.handle_in_world(&assign, &who(), 500).unwrap();

        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let spots = crate::fishing_service::FishingSpots::load(&directory)
            .unwrap()
            .unwrap();
        let rewards = game.reward_snapshot();
        game.fishing
            .enter(&spots, REED, 1, 0, who().position, false, 1, rewards)
            .unwrap();
        assert!(game.is_occupied(super::occupation::Occupation::Fishing));

        let before = game.state();
        let refused = game.handle_in_world(&press(), &who(), 1_000).unwrap();
        assert!(refused.boombox.is_none() && !refused.changed);
        assert!(refused.replies.len() == 1 && refused.replies[0].starts_with(&[36, 0, 1, 0]));
        assert_eq!(game.state().inventory, before.inventory);

        game.fishing.reset_all();
        // Outside the open world (zone 5), free of any occupation: still refused.
        let away = CharacterState {
            zone_id: 5,
            ..who()
        };
        let refused = game.handle_in_world(&press(), &away, 1_000).unwrap();
        assert!(refused.boombox.is_none() && !refused.changed);
        assert!(refused.replies.len() == 1 && refused.replies[0].starts_with(&[36, 0, 1, 0]));
        assert_eq!(game.state().inventory, before.inventory);

        let placed = game.handle_in_world(&press(), &who(), 1_001).unwrap();
        assert_eq!(
            placed.boombox,
            Some(TOTEM),
            "no recast was stamped by the refusal"
        );
    }

    /// T557: a player in a live activity does not click a cake (silently), and the same click
    /// reaches the registry once the player is free.
    #[test]
    fn a_busy_player_does_not_click_a_cake() {
        let reference = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        let mut game = Gameplay::load(&reference, None).unwrap();
        game.initialize_world(&who(), 0).unwrap();
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let spots = crate::fishing_service::FishingSpots::load(&directory)
            .unwrap()
            .unwrap();
        let rewards = game.reward_snapshot();
        game.fishing
            .enter(&spots, REED, 1, 0, who().position, false, 1, rewards)
            .unwrap();
        let guid = crate::placed_cakes::GUID_TAG | 1;
        let click = crate::npc_interaction::NpcInteractRequest {
            target: guid,
            trailing_flag: false,
        }
        .encode();
        let busy = game.handle_in_world(&click, &who(), 1_000).unwrap();
        assert!(busy.shared_cake.is_none() && busy.replies.is_empty() && !busy.changed);
        game.fishing.reset_all();
        let free = game.handle_in_world(&click, &who(), 1_001).unwrap();
        assert_eq!(
            free.shared_cake,
            Some(crate::placed_cakes::Action::Click { guid })
        );
    }

    /// C3: a lure is refused (and kept) with a reason outside fishing; used while fishing it goes
    /// on the session's line, and comes off when its buff ends.
    #[test]
    fn c3_a_lure_goes_on_the_fishing_line_for_its_time() {
        use crate::item_bar::use_wire::{ImmediateActivation, encode_immediate_activation};
        const LURE: i32 = 68152; // 16oz Steak
        let reference = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        // The reference with the release item tables beside it, which define the lure.
        let dir = std::env::temp_dir().join(format!("osfr-c3-lure-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for entry in std::fs::read_dir(&reference).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                std::fs::copy(&path, dir.join(path.file_name().unwrap())).unwrap();
            }
        }
        let release = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release");
        for name in ["item-table.json", "item-table-supplement.json"] {
            std::fs::copy(release.join(name), dir.join(name)).unwrap();
        }
        let mut saved = Gameplay::load(&dir, None).unwrap().state();
        let id = saved.inventory.items.iter().map(|i| i.id).max().unwrap_or(0) + 1000;
        saved.inventory.items.push(InventoryItem {
            definition_id: LURE,
            tint: 0,
            id,
            count: 2,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        let mut game = Gameplay::load(&dir, Some(saved)).unwrap();
        game.initialize_world(&who(), 0).unwrap();
        let check = encode_immediate_activation(ImmediateActivation::Check, id);
        let use_it = encode_immediate_activation(ImmediateActivation::Use, id);
        let refused = game.handle_in_world(&check, &who(), 1_000).unwrap();
        let said = |out: &Outcome| {
            out.replies
                .iter()
                .any(|p| String::from_utf8_lossy(p).contains("Use a lure while you are fishing."))
        };
        assert!(said(&refused), "not fishing: the reason");
        let held = |game: &Gameplay| {
            let items = &game.state().inventory.items;
            items.iter().find(|i| i.id == id).map(|i| i.count - i.consumed_count)
        };
        assert_eq!(held(&game), Some(2), "kept");

        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let spots = crate::fishing_service::FishingSpots::load(&directory)
            .unwrap()
            .unwrap();
        let rewards = game.reward_snapshot();
        game.fishing
            .enter(&spots, REED, 1, 0, who().position, false, 1, rewards)
            .unwrap();
        game.handle_in_world(&check, &who(), 2_000).unwrap();
        game.handle_in_world(&use_it, &who(), 2_001).unwrap();
        assert_eq!(game.fishing.lure_key(1), Some("16oz_steak"), "on the line");
        assert_eq!(held(&game), Some(1), "one lure spent");
        let length = match game.item_bar.effect(LURE).map(|row| row.effect) {
            Some(crate::item_bar::EffectKind::Stat { duration_ms, .. }) => duration_ms,
            other => panic!("{other:?}"),
        };
        let mut out = Outcome::default();
        game.tick_consumable_effects(1, 2_001 + length, &mut out);
        assert_eq!(game.fishing.lure_key(1), None, "off the line when the buff ends");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// F690: the host's row is the catalogue's shared pointer, not a fresh copy of the clip table.
    #[test]
    fn a_boombox_row_lookup_returns_the_shared_catalogue_row() {
        let reference = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        let game = Gameplay::load(&reference, None).unwrap();
        let first = game
            .boombox_row(TOTEM)
            .expect("the bundled catalogue has the totem");
        let second = game.boombox_row(TOTEM).unwrap();
        assert!(
            std::sync::Arc::ptr_eq(&first, &second),
            "the lookup copied the row instead of handing out the catalogue's pointer"
        );
    }
}
