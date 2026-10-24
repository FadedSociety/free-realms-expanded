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
}
