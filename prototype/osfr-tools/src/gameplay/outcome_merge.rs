//! How a staged `Outcome` joins the one being built. Kept apart from `gameplay.rs`, the hub.
use super::*;

impl Outcome {
    /// Publish a staged outcome into this one. Every field is named in the
    /// destructuring pattern, so adding an `Outcome` field fails to compile here
    /// until its merge rule is decided.
    pub(super) fn absorb(&mut self, other: Outcome) {
        let Outcome {
            dev_command_denied,
            shared_field,
            replies,
            changed,
            granted_items,
            zoning,
            scene_zoning,
            relocation,
            group_heal,
            nearby,
            broadcast,
            admin_move,
            world_event,
            boombox,
            shared_cake,
            house_broadcast,
            open_lobby,
        } = other;
        self.dev_command_denied |= dev_command_denied;
        self.shared_field = shared_field.or(self.shared_field.take());
        self.replies.extend(replies);
        self.changed |= changed;
        self.granted_items.extend(granted_items);
        self.zoning = zoning.or(self.zoning.take());
        self.scene_zoning = scene_zoning.or(self.scene_zoning.take());
        self.relocation = relocation.or(self.relocation.take());
        self.group_heal = group_heal.or(self.group_heal.take());
        self.nearby.extend(nearby);
        self.broadcast = broadcast.or(self.broadcast.take());
        self.admin_move = admin_move.or(self.admin_move.take());
        self.world_event = world_event.or(self.world_event.take());
        self.boombox = boombox.or(self.boombox.take());
        self.shared_cake = shared_cake.or(self.shared_cake.take());
        self.house_broadcast = crate::housing_service::HouseBroadcast::merge(
            self.house_broadcast.take(),
            house_broadcast,
        );
        self.open_lobby = self.open_lobby.or(open_lobby);
    }
}
