//! L043: the moved character's side of an admin `realm move`. The host stages it on a clone
//! of the target session's gateway and commits it through that session's own
//! `commit_outcome` (pose fanout, membership, save), as it does an instance transfer.

use super::{Gateway, GatewayOutcome, invalid, wrapped};
use crate::{
    character_store::CharacterState,
    dev_teleport,
    movement::{ExpectedDiscontinuity, MovementState},
};
use std::io;

impl Gateway {
    /// Move this character to an open-world `position`; `rotation` `None` keeps its facing.
    /// An error refuses and stages nothing: still loading, no gameplay, a pose out of bounds,
    /// or anything `Gameplay::admin_relocation` refuses as a 122 rescue would.
    pub fn stage_admin_relocation(
        &mut self,
        position: [f32; 3],
        rotation: Option<[f32; 3]>,
        now_ms: u64,
    ) -> io::Result<GatewayOutcome> {
        if !self.loaded {
            return Err(invalid("it is still loading"));
        }
        let state = CharacterState {
            position,
            rotation: rotation.unwrap_or(self.state.rotation),
            ..self.state.clone()
        };
        state.validate()?;
        let relocation = dev_teleport::Relocation {
            position: state.position,
            rotation: state.rotation,
        };
        let result = self
            .gameplay
            .as_mut()
            .ok_or_else(|| invalid("it has no gameplay session"))?
            .admin_relocation(relocation, &self.state)?;
        // Staged exactly like a relocation in `handle_at`: pose, movement baseline and the
        // expected jump, then the 38/12 location packet (client reader 0x008e66e0).
        self.state = state;
        self.movement_state =
            MovementState::new(self.state.zone_id, self.state.position, self.state.rotation);
        self.expected_discontinuity = Some(ExpectedDiscontinuity::developer_teleport(
            self.state.zone_id,
            self.state.position,
        ));
        self.last_movement_ms = Some(now_ms);
        let mut replies = vec![wrapped(&dev_teleport::location_packet(
            self.state.position,
            self.state.rotation,
        ))];
        replies.extend(result.replies.iter().map(|reply| wrapped(reply)));
        Ok(GatewayOutcome {
            replies,
            state_changed: true,
            // Saved inside the commit, never deferred like a walked pose: a failed save then
            // commits nothing.
            gameplay_changed: true,
            ..GatewayOutcome::default()
        })
    }
}
