//! Shared readers for the actor-list and system-chat packets the pickup suites parse.
//!
//! Include with `#[path = "support/pickup_packets.rs"] mod pickup_packets;`, or through
//! `pickup_fixture`/`npc_quest_chain`, which re-export these readers. Every reader takes the
//! server-emitted packet including its four-byte family/subtype header. Test-only: production
//! never reads its own packets back.
//!
//! Field layouts are the client readers:
//! - 35/2 `AddNpc` `0x00921a10`: the four-byte header, then the actor guid at `4..12`.
//! - 35/3 removal `0x008d69f0` (base `0x008d6830`): the header, a `u16` at `4..6`, then the
//!   actor guid at `6..14`.
//! - 15/3 system chat `0x00b0f5f0` (base `0x00b0f550`): the header, a `u32` byte length at
//!   `4..8`, that many text bytes, then one flag byte.
#![allow(dead_code)]

/// 35/2 `AddNpc` of `guid`: the guid follows the four-byte header (reader `0x00921a10`).
pub fn added(packets: &[Vec<u8>], guid: u64) -> bool {
    packets.iter().any(|packet| {
        packet.starts_with(&[35, 0, 2, 0]) && packet.get(4..12) == Some(&guid.to_le_bytes()[..])
    })
}

/// 35/3 removal of `guid`: a `u16` at `4..6`, then the guid at `6..14` (reader `0x008d69f0`).
pub fn removed(packets: &[Vec<u8>], guid: u64) -> bool {
    packets.iter().any(|packet| {
        packet.starts_with(&[35, 0, 3, 0]) && packet.get(6..14) == Some(&guid.to_le_bytes()[..])
    })
}

/// Whether a 15/3 system chat line contains `needle` (reader `0x00b0f5f0`).
pub fn said(packets: &[Vec<u8>], needle: &str) -> bool {
    packets.iter().any(|packet| {
        if !packet.starts_with(&[15, 0, 3, 0]) || packet.len() < 9 {
            return false;
        }
        let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
        packet
            .get(8..8 + length)
            .is_some_and(|text| String::from_utf8_lossy(text).contains(needle))
    })
}
