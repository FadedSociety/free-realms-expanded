//! Narrow inventory wire fixes and a fail-closed startup catalogue encoder.
//!
//! Recovered 38/3 widths/order: 008d8350 reads five i32 values and then eight
//! bytes; 008b25e0 initializes the two final dwords. Their semantic mapping
//! follows the guide's reconstructed interpretation (R). Catalogue framing:
//! 35/37, 00a3db70.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryV2Error {
    InvalidDefinitionRecord,
    InvalidDefinitionId(i32),
    ConflictingDefinition(i32),
    MissingDefinitions(Vec<i32>),
    CatalogueTooLarge,
}

impl std::fmt::Display for InventoryV2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for InventoryV2Error {}

/// Build the startup 35/37 packet only when every currently owned definition
/// has a record in the cache payload.
///
/// The current inventory path emits its available definition map without
/// checking the self body's definition IDs. That is unsafe for search:
/// 00990d80 resolves every owned definition, and 00990e17 dereferences the
/// result without a null check. The fail-closed requirement is reconstructed
/// server policy (R); the packet shape and client dereference are recovered.
pub fn encode_definition_batch_v2(
    definitions: &[Vec<u8>],
    owned_definition_ids: &[i32],
) -> Result<Vec<u8>, InventoryV2Error> {
    // Match Inventory's existing BTreeMap behavior: one byte-identical record
    // per definition id, in id order.
    let mut by_id = BTreeMap::<i32, &[u8]>::new();
    for record in definitions {
        if record.len() < 4 {
            return Err(InventoryV2Error::InvalidDefinitionRecord);
        }
        let id = i32::from_le_bytes(record[..4].try_into().unwrap());
        if id <= 0 {
            return Err(InventoryV2Error::InvalidDefinitionId(id));
        }
        if let Some(previous) = by_id.get(&id) {
            if *previous != record.as_slice() {
                return Err(InventoryV2Error::ConflictingDefinition(id));
            }
        } else {
            by_id.insert(id, record);
        }
    }

    let mut missing = BTreeSet::new();
    for &id in owned_definition_ids {
        if id <= 0 {
            return Err(InventoryV2Error::InvalidDefinitionId(id));
        }
        if !by_id.contains_key(&id) {
            missing.insert(id);
        }
    }
    if !missing.is_empty() {
        return Err(InventoryV2Error::MissingDefinitions(
            missing.into_iter().collect(),
        ));
    }

    let payload_len = by_id
        .values()
        .try_fold(4usize, |len, record| len.checked_add(record.len()))
        .ok_or(InventoryV2Error::CatalogueTooLarge)?;
    let payload_len_i32 =
        i32::try_from(payload_len).map_err(|_| InventoryV2Error::CatalogueTooLarge)?;
    let count_i32 = i32::try_from(by_id.len()).map_err(|_| InventoryV2Error::CatalogueTooLarge)?;

    let mut packet = Vec::with_capacity(8 + payload_len);
    packet.extend([35u16.to_le_bytes(), 37u16.to_le_bytes()].concat());
    packet.extend(payload_len_i32.to_le_bytes());
    packet.extend(count_i32.to_le_bytes());
    for record in by_id.values() {
        packet.extend_from_slice(record);
    }
    Ok(packet)
}

/// Values written by the recovered seven-i32 38/3 item update. Semantic field
/// names follow the guide's reconstructed mapping (R).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUpdateV2 {
    pub item_id: i32,
    pub count: i32,
    /// -1 when this update does not assign an item-bar slot.
    pub item_bar_slot: i32,
    pub consumed_count: i32,
    pub stored_count: i32,
    /// The last two wire dwords form this 64-bit field: the item's rental expiry, 0 for an owned
    /// item. The client's keyed setter (`008c7090`, reached from the 38/3 case at `009397b0`)
    /// compares it with the item's rental block at `+0x28` or, when there is none, with the null
    /// time `dword_1BE48B8` (zero); any other value creates the block through `0103e080`, and the
    /// Stuff window then lists the item as a Rental. The last-cast time travels only in the
    /// instance record (`008cb5a0` stores it at `+0x20`), never here.
    pub rental_expiration: i64,
}

/// Encode 38/3 with the client field order: key, count, bar slot,
/// consumed count, stored count, then the 64-bit rental expiry.
pub fn encode_item_update_v2(update: ItemUpdateV2) -> Vec<u8> {
    let mut packet = [38u16.to_le_bytes(), 3u16.to_le_bytes()].concat();
    for value in [
        update.item_id,
        update.count,
        update.item_bar_slot,
        update.consumed_count,
        update.stored_count,
    ] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(update.rental_expiration.to_le_bytes());
    packet
}
