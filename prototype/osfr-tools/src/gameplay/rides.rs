//! Ride glue for `Gameplay`; the ride rules live in `crate::mounts`.
use super::*;

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
}
