//! T440: the invited member's side of a group encounter. The host stages an Accept on a clone of
//! the member's gateway and commits it through that session's own `commit_outcome`.

use super::{Gateway, GatewayOutcome, invalid, wrapped};
use crate::players::ZoneInstance;
use std::io;

impl Gateway {
    /// This player's live field-battle run as (battle id, run instance, title string id).
    pub fn joinable_run(&self) -> Option<(u32, i32, i32)> {
        self.gameplay.as_ref()?.joinable_run()
    }

    /// Whether this loaded session could start `battle` now (the Go gates).
    pub fn can_join_group_battle(&self, battle: u32) -> bool {
        self.loaded
            && self
                .gameplay
                .as_ref()
                .is_some_and(|game| game.can_join_group_battle(battle))
    }

    /// Begin `battle` for this player inside the inviter's World instance `run`. An error
    /// refuses and stages nothing: still loading, a failed Go gate, or a bad scene.
    pub fn stage_encounter_join(
        &mut self,
        battle: u32,
        run: i32,
        now_ms: u64,
    ) -> io::Result<GatewayOutcome> {
        let mut game = self
            .gameplay
            .clone()
            .ok_or_else(|| invalid("no gameplay session"))?;
        let mut result = game.join_group_battle(battle, &self.state)?;
        let scene = result
            .zoning
            .take()
            .ok_or_else(|| invalid("battle did not start a scene"))?;
        let target = ZoneInstance::from_world(scene.zone_id, run).map_err(io::Error::other)?;
        let mut outcome = self.stage_instance_transfer(target, scene, now_ms)?;
        self.gameplay = Some(game);
        // The battle's own replies go ahead of the zone-begin packet, as on the Go path.
        outcome
            .replies
            .splice(0..0, result.replies.iter().map(|reply| wrapped(reply)));
        outcome.gameplay_changed = true;
        Ok(outcome)
    }
}
