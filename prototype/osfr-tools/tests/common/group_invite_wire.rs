//! The 40/1 pop-up dword shared by the group-invite service tests.
//!
//! The client reads the dword at record offset `+0x120` from the 40/1 invite
//! (`00921640` -> `00912090`) and copies it into its 40/4 answer (`00a8b640`), so the server
//! compares the value it actually delivered. Tests read that field from the delivered bytes
//! instead of rebuilding it from server internals, and one helper keeps the offset in one place.

use osfr_tools::social_lists::SocialMessage;

/// The generation in the target's delivered 40/1 invite; panics if none was delivered.
#[allow(dead_code)] // The socket suites only have the raw packet.
pub fn invitation_id(messages: &[SocialMessage], target: u64) -> u32 {
    let packet = messages
        .iter()
        .find(|message| {
            message.recipient_guid == target && message.bytes.starts_with(&[40, 0, 1, 0])
        })
        .expect("target received a 40/1 invite");
    invite_generation(&packet.bytes)
}

/// The generation dword in a delivered 40/1 payload. A tunnel header, when present, is in
/// front, so the `len - 24` read lands on the same field for wrapped and bare packets.
#[allow(dead_code)] // The service suites look the packet up by recipient first.
pub fn invite_generation(packet: &[u8]) -> u32 {
    // The generation sits before the five trailing opaque dwords.
    let at = packet.len() - 6 * 4;
    u32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}
