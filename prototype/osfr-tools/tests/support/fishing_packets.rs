//! Shared request builders for the fishing tests (`fishing_entry`, `fishing_service`
//! and `fisherman_entry_chain`). Test-only; production never includes this file.
//!
//! 138 request shapes come from the client writers `00b6a380|400|460|4d0`; the
//! family-39 envelope from the base constructor `008a4420` (family, subtype, then
//! three i32: instance, `-1`, game id). 39/5 start and 39/6 quit are sent by
//! `009be7d0` / `009be8e0`, both through `009bdc60`; 41/109 is the existing
//! encounter close.
#![allow(dead_code)]

use osfr_tools::fishing::Handle;

pub fn handle_bytes(handle: Handle) -> Vec<u8> {
    [handle.id.to_le_bytes(), handle.seq.to_le_bytes()].concat()
}

/// One opcode-138 request: `u16` opcode, `u16` sub-opcode, body.
pub fn request(sub: u16, body: &[u8]) -> Vec<u8> {
    let mut packet = vec![138, 0];
    packet.extend(sub.to_le_bytes());
    packet.extend(body);
    packet
}

pub fn register(handle: Handle) -> Vec<u8> {
    let mut body = handle_bytes(handle);
    body.push(0);
    request(2, &body)
}

/// 138/6 cast: handle, target vector (w = 1), flag byte.
pub fn cast_at(handle: Handle, origin: [f32; 3], distance: f32) -> Vec<u8> {
    let mut body = handle_bytes(handle);
    for coordinate in [origin[0] + distance, origin[1], origin[2], 1.0f32] {
        body.extend(coordinate.to_le_bytes());
    }
    body.push(0);
    request(6, &body)
}

/// 138/7 reel: handle, flag byte.
pub fn reel(handle: Handle) -> Vec<u8> {
    let mut body = handle_bytes(handle);
    body.push(1);
    request(7, &body)
}

/// Family-39 lifecycle envelope: instance, `-1`, game id.
pub fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet
}

pub fn encounter_close(instance: i32) -> Vec<u8> {
    let mut packet = vec![41, 0, 109, 0];
    packet.extend(0i32.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet
}
