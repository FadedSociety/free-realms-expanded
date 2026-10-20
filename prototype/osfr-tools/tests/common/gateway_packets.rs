//! Request bodies shared by the S6 packet tests and the vendor suites.
//!
//! Client-derived shapes: the chat request reader is `00b11600`; the NPC menu
//! constructor is 26/8 `008b6bc0` (u64 target + bool); the menu selection
//! constructor is 26/10 `00a36f80` (u64 target + i32 choice). The login body is
//! a synthetic ticket request for the gateway session path.

/// Client chat request reader 0x00b11600: header, guid pair, two zero context
/// records, the text and its trailing 28-byte block.
#[allow(dead_code)] // The vendor suites do not send chat.
pub fn chat(text: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0, 0, 0];
    packet.extend(1_u64.to_le_bytes());
    packet.extend(0_u64.to_le_bytes());
    for _ in 0..2 {
        packet.extend([0; 12]);
        packet.extend(0_u32.to_le_bytes());
        packet.extend(0_u32.to_le_bytes());
    }
    packet.extend((text.len() as u32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.extend([0; 28]);
    packet
}

/// Client 26/8 constructor 0x008b6bc0: u64 target plus bool.
pub fn interact(npc_id: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.push(0);
    packet
}

/// Client 26/10 constructor 0x00a36f80: u64 target plus i32 choice.
pub fn select(npc_id: u64, option: u32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.extend(option.to_le_bytes());
    packet
}

/// Synthetic ticket login: u16 count, then three length-prefixed strings, with
/// the account guid after the ticket.
#[allow(dead_code)] // The vendor suites build their login through `gateway_vendor`.
pub fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}
