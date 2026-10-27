//! Ride and pet glue on a session gateway (W162/W177), moved verbatim from the hub (T458). The
//! ride and pet rules live in `crate::mounts` and `crate::pets`.

use super::{Gateway, GatewayOutcome, wrapped};
use std::io;

impl Gateway {
    /// W162: the summoned ride as other players must see it; `None` without one.
    pub fn ride_view(&self) -> Option<crate::mounts::RideView> {
        self.gameplay.as_ref()?.ride_view(self.state.guid)
    }

    /// S17: adopt the roster read from the character document (`Snapshot::pets`) before world
    /// entry. Without gameplay it stays on disk untouched (`snapshot` then sends `None`).
    pub fn adopt_pets(&mut self, pets: Option<serde_json::Value>) -> io::Result<()> {
        if let Some(game) = self.gameplay.as_mut() {
            game.adopt_pets(pets)?;
        }
        Ok(())
    }

    /// W177: the active pet for AddNpc fanout and late viewers.
    pub fn pet_view(&self) -> Option<crate::pets::PetPresence> {
        self.gameplay.as_ref()?.pet_view(self.state.guid)
    }

    /// T455: the summoned pet's follow step on the world tick, on the live gameplay (no clone).
    pub(super) fn pet_tick(&mut self, now_ms: u64) -> GatewayOutcome {
        let mut outcome = GatewayOutcome::default();
        let state = &self.state;
        if let Some(result) = self
            .gameplay
            .as_mut()
            .and_then(|g| g.pet_tick(state, now_ms))
        {
            // T485: a feed's apply step spends a food, so it is saved like any other change.
            outcome.state_changed = result.changed;
            outcome.gameplay_changed = result.changed;
            outcome.replies = result.replies.iter().map(|r| wrapped(r)).collect();
            outcome.nearby = result.nearby;
        }
        outcome
    }

    /// W177 (R): a zone transition dismisses the active pet and removes its actor.
    pub fn settle_pets(&mut self, zoned: bool, now_ms: u64) -> io::Result<GatewayOutcome> {
        let mut outcome = GatewayOutcome::default();
        let state = &self.state;
        if let Some(game) = self.gameplay.as_mut() {
            let ended = game.settle_pets(state, zoned, now_ms)?;
            outcome.state_changed = ended.changed;
            outcome.gameplay_changed = ended.changed;
            outcome.replies = ended.replies.iter().map(|r| wrapped(r)).collect();
            outcome.nearby = ended.nearby;
        }
        Ok(outcome)
    }

    /// W162: end the ride after a message or tick when the player zoned, entered a fight or
    /// minigame, or was knocked out. Replies are wrapped like every gateway reply.
    pub fn settle_rides(&mut self, zoned: bool) -> GatewayOutcome {
        let mut outcome = GatewayOutcome::default();
        if let Some(game) = self.gameplay.as_mut() {
            let ended = game.settle_mounts(&self.state, zoned);
            outcome.replies = ended.replies.iter().map(|r| wrapped(r)).collect();
            outcome.nearby = ended.nearby;
        }
        outcome
    }
}
