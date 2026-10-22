//! Public appearance updates after a profile or outfit change.
//!
//! The client applies a 35/7 `EquippedItemsChange` as the active profile's
//! attachment group (reader `0x0091af50`, constructor `0x0091d3b0`).
use super::*;

impl Gameplay {
    /// (R) 35/7 EquippedItemsChange for a job switch: the new active outfit,
    /// which the client applies as a profile-swap attachment group. Client
    /// reader 0x0091af50 (ctor 0x0091d3b0).
    fn profile_switch_packet(&self, guid: u64, attachments: &[u8]) -> Vec<u8> {
        let mut packet = vec![35, 0, 7, 0];
        packet.extend(guid.to_le_bytes());
        packet.extend_from_slice(attachments);
        packet.extend(0i32.to_le_bytes()); // (R) trailing value; the client reads one i32.
        packet
    }

    /// Announce the active outfit after the complete profile operation, not
    /// just a change of profile id: activating the same job can restore a
    /// missing starter outfit.
    pub(super) fn announce_profile_outfit(
        &self,
        guid: u64,
        before: &[u8],
        switched: bool,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let Some(profiles) = &self.profiles else {
            return Ok(());
        };
        let after = self.profile_attachments(profiles.shown())?;
        if !switched && after == before {
            return Ok(());
        }
        out.nearby.push(self.profile_switch_packet(guid, &after));
        Ok(())
    }

    /// AddPc wield type for `profile` (reader `0x0091aee4`; spawn calls the actor's wield setter
    /// `0x009582c0` at `0x009300f6`, as 35/6 does at `0x009315a9`).
    pub fn public_wield_type(&self, profile: i32) -> i32 {
        self.inventory.equipped_wield_type(profile)
    }
}
