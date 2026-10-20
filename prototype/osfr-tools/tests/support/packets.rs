//! Generic family/subtype reply helpers shared by the field acceptance tests.
//!
//! `shared_field_kill_quest_credit.rs`, `field_kill_rewards.rs` and
//! `field_camps_high_band.rs` include this file with
//! `#[path = "support/packets.rs"] mod packets;`. Test-only: the builders pin the
//! bytes the client readers expect, and the exact bytes stay at each assertion.

fn is_family(packet: &[u8], family: u16, subtype: u16) -> bool {
    packet.len() >= 4
        && packet[..2] == family.to_le_bytes()
        && packet[2..4] == subtype.to_le_bytes()
}

/// How many replies carry this family/subtype.
pub fn count(replies: &[Vec<u8>], family: u16, subtype: u16) -> usize {
    replies
        .iter()
        .filter(|packet| is_family(packet, family, subtype))
        .count()
}

/// The one reply of a family/subtype; the tests assert its exact bytes.
pub fn packet<'a>(replies: &'a [Vec<u8>], family: u16, subtype: u16) -> &'a [u8] {
    replies
        .iter()
        .find(|packet| is_family(packet, family, subtype))
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("missing {family}/{subtype}"))
}

/// The 4-byte header these reward packets carry (`[family u16][subtype u16]`)
/// followed by `values` as i32 LE.
pub fn carrier(family: u16, subtype: u16, values: &[i32]) -> Vec<u8> {
    let mut bytes = vec![
        family as u8,
        (family >> 8) as u8,
        subtype as u8,
        (subtype >> 8) as u8,
    ];
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}
