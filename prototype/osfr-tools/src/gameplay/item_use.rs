//! The item bar and the consumable effect stack on `Gameplay` (W146, T474).
//!
//! The stack (`consumable_effects::EffectStack`) is session-only: it is not in the save, so a
//! relog starts with no effects and no buff rows (R, both reference servers do the same).
//! Packets for the owner go to `Outcome.replies`, packets neighbours must see to
//! `Outcome.nearby`; the gateway fans those out through the interest set.
use super::*;
use crate::consumable_effects::{self, Emit, Refusal, Spec};

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
            out.absorb(outcome_of(emit));
        }
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
        Ok(Some(outcome_of(emit)))
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
        Ok(())
    }

    /// W146: item-bar requests. The potion is consumed and the heal applied in this one call, so
    /// the saved state never holds one without the other. Without an encounter engine the player
    /// has no tracked health, so nothing is claimed and nothing is spent.
    ///
    /// T461: a boombox press is never spent. It is refused (36/1, nothing changed) while the
    /// player is mounted (the Mounts guide's wiki rule), busy (the start gate of the occupation
    /// table) or outside the open world (R), and otherwise leaves the definition in
    /// `Outcome::boombox` for the host to place the box after the commit.
    pub(super) fn handle_item_bar(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(encounter) = self.encounter.as_ref() else {
            return Ok(None);
        };
        let player = crate::item_bar::Player {
            guid: character.guid,
            alive: encounter.state().player_health > 0,
            can_place_box: character.zone_id == crate::world::OPEN_WORLD_ZONE
                && !self.is_mounted()
                && !self.blocks(|gates| gates.start),
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
        Ok(Some(out))
    }

    /// T461: the catalogue row of a boombox definition a press named, for the host to place.
    pub fn boombox_row(&self, definition: i32) -> Option<crate::boombox::Boombox> {
        self.item_bar.boombox(definition).cloned()
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
}
