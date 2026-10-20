//! Client `53/*` pet request bodies shared by the gateway's session tests and
//! `tests/pet_routing.rs`. Test-only; production never includes this file.
//!
//! `SendPetSummonByItemId` (`00b60110`) writes subtype 32, then the `ItemRecord`
//! definition (`+4`) and tint (`+8`) that `00a1ec00` pushes at `00a1f2c2`-`00a1f2d2`;
//! the summon request is `00b5a650` (u32 id then action byte).

/// Client `SendPetSummon` (`00b5a650`): `53 00 | 4 | u32 pet id | action 1`.
pub fn summon(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![1]].concat()
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
