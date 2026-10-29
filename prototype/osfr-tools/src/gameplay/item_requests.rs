//! Mystery chest requests, answered before the shared 42 inventory branch, and the 35/6 an
//! equip change sends neighbours (moved out of the `gameplay.rs` hub).
use super::*;

impl Gameplay {
    pub(super) fn handle_mystery_chest(
        &mut self,
        inner: &[u8],
        player_guid: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(content) = &self.mystery_chests else {
            return Ok(None);
        };
        let Some(request) = mystery_chest::parse_request(inner)? else {
            return Ok(None);
        };
        let before = self.inventory.state().clone();
        let result = content.handle(
            &mut self.inventory,
            &mut self.pending_chest,
            request,
            player_guid,
        )?;
        let mut out = Outcome {
            replies: result.replies,
            changed: result.changed,
            ..Outcome::default()
        };
        self.npc_inventory_events(&before, &mut out)?;
        Ok(Some(out))
    }
}

/// (R) 35/6 EquipItemChange a neighbour needs after an active-job equip or
/// unequip. Inventory content names its own player guid in the packet, so the
/// broadcast names the authenticated character instead. An empty model is the
/// client's removal path (`0092f460` case 6).
pub(super) fn item_change_packet(inner: &[u8], guid: u64, replies: &[Vec<u8>]) -> Option<Vec<u8>> {
    if inner.len() == 12 && inner.starts_with(&[42, 0, 2, 0]) {
        let slot = i32::from_le_bytes(inner[4..8].try_into().ok()?);
        let profile = i32::from_le_bytes(inner[8..12].try_into().ok()?);
        // The inventory owns the one 35/6 encoder; an empty attachment clears the slot.
        return Some(crate::inventory::equip_change_packet(
            guid, 0, None, slot, profile,
        ));
    }
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 6, 0]))?;
    let mut packet = packet.clone();
    packet.get_mut(4..12)?.copy_from_slice(&guid.to_le_bytes());
    Some(packet)
}
