//! Client `53/*` pet request bodies shared by the gateway's session tests and
//! `tests/pet_routing.rs`. Test-only; production never includes this file.
//!
//! `SendPetSummonByItemId` (`00b60110`) writes subtype 32, then the `ItemRecord`
//! definition (`+4`) and tint (`+8`) that `00a1ec00` pushes at `00a1f2c2`-`00a1f2d2`;
//! the summon request is `00b5a650` (u32 id then action byte).
//! T485: the care panel's utility (53/16), feed (53/19) and water (53/28) bodies.
//! T488: groom (53/18) and the tactile stroke report (53/22).
//! T489: perform trick (53/7), gesture result (53/8) and favourite trick (53/14).
#![allow(dead_code)] // each includer uses a subset

/// Client `SendPetSummon` (`00b5a650`): `53 00 | 4 | u32 pet id | action 1`.
pub fn summon(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![1]].concat()
}

/// Client `Pet.Dismiss` (`00c07420`): action 0 with an id word it never writes, so any id is
/// carried and the action alone decides (`PetRequest::decode`).
pub fn dismiss(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![0]].concat()
}

/// Client `SendPetSummonByItemId` (`00b60110`): `53 00 | 32 | definition u32 | tint u32`,
/// never a pet or instance id.
pub fn summon_by_item(definition_id: u32, tint: u32) -> Vec<u8> {
    [
        vec![53, 0, 32],
        definition_id.to_le_bytes().to_vec(),
        tint.to_le_bytes().to_vec(),
    ]
    .concat()
}

/// Client `Pet.EquipUtility`/`UnequipUtility` (writer `00c06ea0`):
/// `53 00 | 16 | u8 remove | u32 pet id | u32 utility slot | u32 item`.
pub fn utility(remove: bool, pet_id: u32, slot: u32, item: u32) -> Vec<u8> {
    let mut bytes = vec![53, 0, 16, u8::from(remove)];
    for value in [pet_id, slot, item] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// Client `Pet.StartFeed` (`00b5f720`): `53 00 | 19 | u32 active pet id | u32 0`.
pub fn feed(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 19], pet_id.to_le_bytes().to_vec(), vec![0; 4]].concat()
}

/// Client 53/28 (`00b60350`): `53 00 | 28 | u8 in water`.
pub fn water(in_water: bool) -> Vec<u8> {
    vec![53, 0, 28, u8::from(in_water)]
}

/// Client `Pet.StartGroom` (`00b5f8c0`, packet ctor `00b51610`): `53 00 | 18 | u32 active pet id |
/// u32 0`.
pub fn groom(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 18], pet_id.to_le_bytes().to_vec(), vec![0; 4]].concat()
}

/// Client PetTactileNotify (ctor `00ccdb70`, writer `00ccea80` after the header): `53 00 | 22 | u32
/// mode (the pet handler's state word) | u32 3 | u32 stroke time ms | u32 movement`.
pub fn tactile(mode: u32, duration_ms: u32, movement: u32) -> Vec<u8> {
    let mut bytes = vec![53, 0, 22];
    for value in [mode, 3, duration_ms, movement] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// Client `Pet.PerformTrick` writer (`00b5ff50`, ctor `00b50a10`): `53 00 | 7 | u32 trick id`.
pub fn perform_trick(trick_id: u32) -> Vec<u8> {
    [vec![53, 0, 7], trick_id.to_le_bytes().to_vec()].concat()
}

/// Client `PetTrickGestureResult` writer (`00b5ffe0`, ctor `00b50c20`): `53 00 | 8 | u32 trick id`,
/// no grade.
pub fn trick_result(trick_id: u32) -> Vec<u8> {
    [vec![53, 0, 8], trick_id.to_le_bytes().to_vec()].concat()
}

/// Client `Pet.SetFavoriteTrick` (`00b5f670`, ctor `00b51290`): `53 00 | 14 | u32 active pet id |
/// u32 slot (the one-based UI slot minus one) | u32 trick id`.
pub fn favorite(pet_id: u32, slot: u32, trick_id: u32) -> Vec<u8> {
    let mut bytes = vec![53, 0, 14];
    for value in [pet_id, slot, trick_id] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}
