//! The live client's `127/5` (move) body, shared by the housing service and session suites.
//!
//! Include with `#[path = "support/housing_move.rs"] mod housing_move;`. Test-only: the layout is
//! the client writer `00abf620`: fixture guid, position float4, rotation float4, a zero dword
//! (where `127/2` carries the placement scale), then a 24-byte tail ending in 1.0. The captured
//! bytes are in `tests/housing_service.rs` (`RETAIL_MOVE`, `p2.pcap` frame 32782).

/// One `127/5` move exactly as the client sends it.
pub fn move_frame(fixture_guid: u64, position: [f32; 4], rotation: [f32; 4]) -> Vec<u8> {
    let mut inner = vec![127, 0, 5, 0];
    inner.extend(fixture_guid.to_le_bytes());
    for value in position.into_iter().chain(rotation) {
        inner.extend(value.to_le_bytes());
    }
    inner.extend([0u8; 24]);
    inner.extend(1.0f32.to_le_bytes());
    inner
}
