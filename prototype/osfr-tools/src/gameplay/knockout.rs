//! Knockout on the gameplay side: field damage that can knock the player out,
//! and the paid revive back up.
//!
//! The encounter engine owns health and every revive rule
//! (`encounter/revive.rs`). The paid revive also spends coins, and the wallet
//! lives here, so this module charges the engine's price in the same staged
//! `Gameplay` as the revive: both commit together or neither does.
use super::*;

impl Gameplay {
    pub fn receive_field_damage(
        &mut self,
        damage: u32,
        now_ms: u64,
        engaged: bool,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        self.field_engaged = engaged;
        if let Some(engine) = &mut self.encounter {
            let result = engine.receive_field_damage(damage, now_ms, engaged);
            self.apply_encounter(result, &mut out)?;
        }
        Ok(out)
    }

    /// "Revive here" (41/122 with its bool set, client sender `0x0093be30`).
    /// Returns false for any other packet; the encounter handles the free
    /// revive and refuses a malformed one. The engine decides (knocked out, in
    /// the overworld, wallet at the price) and returns the price; a refusal
    /// costs 0 and changes nothing.
    pub(super) fn handle_paid_revive(
        &mut self,
        inner: &[u8],
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<bool> {
        if inner.len() != 13 || !inner.starts_with(&[41, 0, 122, 0]) || inner[12] != 1 {
            return Ok(false);
        }
        let Some(engine) = self.encounter.as_mut() else {
            return Ok(false);
        };
        let (result, price) = engine.paid_revive(self.inventory.state().coins, now_ms);
        let paid = self.inventory.spend_coins(price)?;
        out.changed |= paid.changed;
        out.replies.extend(paid.replies);
        self.apply_encounter(result, out)?;
        Ok(true)
    }
}
