//! Ride glue for `Gameplay`; the ride rules live in `crate::mounts`.
use super::{Gameplay, Outcome};

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
}
