//! T514: a fishing round at a hole whose scene the client has (`client_scene_cached`) is fished
//! in that scene, in the housing pattern (`gameplay/housing.rs`). Go fish! keeps the world pose
//! and zones the player into the hole; the round's start packets wait until the hole has loaded;
//! every end of the round zones the player back to the kept pose, once. The round itself is
//! `crate::fishing_service`; this file only binds it to the session.
use super::*;
use crate::fishing_service::Change;

#[derive(Clone, Debug)]
pub(super) struct FishingHole {
    /// The world pose captured at Go fish!.
    return_to: CharacterState,
    /// The round's start packets (its 39/16 offer and what makes it ready), sent once the hole
    /// scene has loaded.
    pending: Vec<Vec<u8>>,
    /// The hole scene sent at Go fish!. `current_zone` reports it, so the membership registry
    /// gives the visit its own instance, as for a house lot.
    scene: crate::zone_transition::ZoneTransition,
}

impl Gameplay {
    /// The NPC menu's admitted Go fish!. An in-place round answers now; a hole round zones the
    /// player into the hole and holds its start packets for the load.
    pub(super) fn start_fishing_trip(
        &mut self,
        change: Change,
        character: &CharacterState,
        out: &mut Outcome,
    ) {
        out.changed = true;
        let Some(scene) = change.scene else {
            out.replies = change.replies;
            return;
        };
        out.scene_zoning = Some(scene.clone());
        self.fishing_hole = Some(FishingHole {
            return_to: character.clone(),
            pending: change.replies,
            scene,
        });
    }

    /// After a hole or (L931) the derby arena loads (`initialize_world`, like `housing_loaded`):
    /// the held start packets. A second load resends nothing.
    pub(super) fn minigame_scene_loaded(&mut self) -> Option<Vec<Vec<u8>>> {
        let hole = (self.fishing_hole.as_mut()).map(|hole| std::mem::take(&mut hole.pending));
        hole.or_else(|| self.derby_arena_loaded())
            .or_else(|| self.pirates_arena_loaded())
            .or_else(|| self.garage_loaded())
    }

    /// The scene a fishing hole or (L931) the derby arena put the player in.
    pub(super) fn minigame_scene(&self) -> Option<&crate::zone_transition::ZoneTransition> {
        let hole = self.fishing_hole.as_ref().map(|hole| &hole.scene);
        let pirates = (self.pirates.as_ref().map(|arena| &arena.scene))
            .or_else(|| self.garage.as_ref().map(|garage| &garage.scene));
        hole.or_else(|| self.derby_arena_scene()).or(pirates)
    }

    /// The hole, or the derby arena while its visit lasts, is where the player is
    /// (`supports_location`).
    pub(super) fn in_minigame_scene(&self, character: &CharacterState) -> bool {
        let hole = self.fishing_hole.as_ref().map(|hole| &hole.scene);
        hole.is_some_and(|scene| u32::try_from(scene.id) == Ok(character.zone_id))
            || self.derby_arena_scene().is_some()
            || self.pirates.is_some()
            || self.garage.as_ref().is_some_and(|garage| garage.entered)
    }

    /// A save taken in a house or a fishing hole stores the world pose, never scene coordinates.
    pub fn scene_return(&self) -> Option<CharacterState> {
        let hole = self
            .fishing_hole
            .as_ref()
            .map(|hole| hole.return_to.clone());
        // L931: a derby arena match stores its entry pose the same way.
        let arena = self
            .derby
            .as_ref()
            .and_then(|derby| derby.arena_return().cloned());
        let pirates = self.pirates.as_ref().map(|visit| visit.return_to.clone());
        // Off the offer the player is still in the world and may have walked on.
        let garage = (self.garage.as_ref().filter(|visit| visit.entered))
            .map(|visit| visit.return_to.clone());
        self.house_return().or(hole).or(arena).or(pirates).or(garage)
    }

    /// A reconnect leaves a house or a fishing hole and resumes at the world pose. The fishing
    /// round itself goes with `reset_session`, which runs next.
    pub fn abandon_scene_for_reconnect(&mut self) -> Option<CharacterState> {
        let hole = self.fishing_hole.take().map(|hole| hole.return_to);
        let arena = self
            .derby
            .as_mut()
            .and_then(crate::derby_gateway::DerbyGateway::leave_arena);
        let pirates = self.pirates.take().map(|visit| visit.return_to);
        let garage = (self.garage.take().filter(|visit| visit.entered))
            .map(|visit| visit.return_to);
        self.abandon_house_for_reconnect()
            .or(hole)
            .or(arena)
            .or(pirates)
            .or(garage)
    }

    /// W160 per-tick upkeep: the bite timers and the end of a round that left its spot (nothing
    /// here is saved), then the way out of a hole whose round ended.
    pub(super) fn supervise_fishing(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if let Ok(Some(change)) = self.fishing.supervise(
            character.guid,
            character.zone_id,
            character.position,
            now_ms,
        ) {
            out.replies = change.replies;
        }
        self.leave_fishing_hole(&mut out)?;
        Ok(out)
    }

    /// Every end of a hole round (39/6, 26/42, 41/109, a server `end`) zones the player back to
    /// the world pose kept at Go fish!. The visit is consumed here, so the return happens once.
    pub(super) fn leave_fishing_hole(&mut self, out: &mut Outcome) -> io::Result<()> {
        if let Some(hole) = self
            .fishing_hole
            .as_ref()
            .filter(|_| self.fishing.is_empty())
        {
            out.scene_zoning = Some(self.world_zoning(&hole.return_to)?);
            self.fishing_hole = None;
        }
        Ok(())
    }
}
