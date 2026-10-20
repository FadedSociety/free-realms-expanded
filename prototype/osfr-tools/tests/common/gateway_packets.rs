//! Request bodies shared by the S6 packet tests and the vendor suites.
//!
//! Client-derived shapes: the chat request reader is `00b11600`; the NPC menu
//! constructor is 26/8 `008b6bc0` (u64 target + bool); the menu selection
//! constructor is 26/10 `00a36f80` (u64 target + i32 choice); 26/13
//! `CommandPacketSetProfile` sender `00938e10` (constructor `008a29f0`, body one
//! i32 profile); 36/10 ability writer `00a342e0` (i32 bar, i32 slot, i32 target
//! mode, then a u64 target when mode 0); 42/3 `InventoryPacketEquipByGuid`
//! writer `00bdb450` (sender `00bda510`: i32 instance, i32 profile, i32 slot).
//! The login body is a synthetic ticket request for the gateway session path.

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
/// `guid` after the ticket. The gateway compares that guid with the loaded
/// character (`src/gateway.rs` `parse_message` login arm), so the caller supplies
/// it.
#[allow(dead_code)] // The private S6 module reuses the request builders, not login.
pub fn login(guid: u64) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(guid.to_le_bytes());
        }
    }
    bytes
}

/// The five request families a live movie refuses. The payloads are the shipped
/// client's own shapes (see the module doc): 26/13 job select, 26/8 interact,
/// 26/10 menu select, 36/10 ability and 42/3 equip.
#[allow(dead_code)] // Some include sites use only the single-request builders.
pub fn blocked_families(target: u64) -> [Vec<u8>; 5] {
    let mut job = vec![26, 0, 13, 0];
    job.extend(1_i32.to_le_bytes());
    let mut attack = vec![36, 0, 10, 0];
    attack.extend([0; 12]);
    attack.extend(target.to_le_bytes());
    let mut equip = vec![42, 0, 3, 0];
    equip.extend(i32::MAX.to_le_bytes()); // unowned instance id
    equip.extend(1_i32.to_le_bytes()); // profile key
    equip.extend(0_i32.to_le_bytes()); // equip slot
    [job, interact(target), select(target, 1), attack, equip]
}
