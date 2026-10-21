//! Request bodies shared by the S6 packet tests, the vendor suites and the
//! field-battle fixtures.
//!
//! Client-derived shapes: the chat request reader is `00b11600`; the NPC menu
//! constructor is 26/8 `008b6bc0` (u64 target + bool); the menu selection
//! constructor is 26/10 `00a36f80` (u64 target + i32 choice); 26/13
//! `CommandPacketSetProfile` sender `00938e10` (constructor `008a29f0`, body one
//! i32 profile); 26/6 dialogue response reader `009f5890` (one i32 row); 36/10
//! ability writer `00a342e0` (i32 bar, i32 slot, i32 target mode, then a u64
//! target when mode 0); 42/3 `InventoryPacketEquipByGuid` writer `00bdb450`
//! (sender `00bda510`: i32 instance, i32 profile, i32 slot); 49/2 `QuestReply`
//! constructor `00a86400` (u32 quest id + u8 accept, captured natively in run
//! 143124); 49/14 constructor `00a864c0` (six-byte header + u32 quest id). The
//! login body is a synthetic ticket request for the gateway session path; the
//! caller supplies the manifest version.

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

/// The client version the vendor/save-failure suites send; it is the reference
/// manifest's `client_version` (`lab/references/rust-gateway-reference`).
pub const DEFAULT_CLIENT_VERSION: &str = "1.910.1.530630";

/// The pinned-version form re-exported by `tests/common/gateway_vendor.rs` for
/// the vendor suites and `priced_vendors_drawable`.
#[allow(dead_code)]
pub fn login(guid: u64) -> Vec<u8> {
    login_with_version(guid, DEFAULT_CLIENT_VERSION)
}

/// Synthetic ticket login: u16 count, then three length-prefixed strings, with
/// `guid` after the ticket. The gateway compares that guid with the loaded
/// character (`src/gateway.rs` `parse_message` login arm) and `version` with its
/// manifest, so the caller supplies both.
#[allow(dead_code)] // The private S6 module and the field-battle fixture use it.
pub fn login_with_version(guid: u64, version: &str) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["synthetic-ticket", version, ""].iter().enumerate() {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(guid.to_le_bytes());
        }
    }
    bytes
}

/// Client 26/6 dialogue response reader `009f5890`: one i32 row.
#[allow(dead_code)] // Included by sites that do not open a quest dialogue.
pub fn dialogue_choice(row: i32) -> Vec<u8> {
    [vec![26, 0, 6, 0], row.to_le_bytes().to_vec()].concat()
}

/// Client 49/2 `QuestReply` constructor `00a86400`: six-byte header, u32 quest
/// id and the u8 accept flag.
#[allow(dead_code)] // Included by sites that do not accept a quest.
pub fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    [
        vec![49, 0, 2, 0, 0, 0],
        quest_id.to_le_bytes().to_vec(),
        vec![u8::from(accept)],
    ]
    .concat()
}

/// Client 49/14 quest end reply constructor `00a864c0`: six-byte header plus the
/// u32 quest id.
#[allow(dead_code)] // Included by sites that do not close a quest.
pub fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    [vec![49, 0, 14, 0, 0, 0], quest_id.to_le_bytes().to_vec()].concat()
}

/// Client 36/10 ability writer `00a342e0`: bar, slot 0, target mode 0, then the
/// u64 target.
pub fn ability(target: u64, bar: i32) -> Vec<u8> {
    let mut packet = vec![36, 0, 10, 0];
    for value in [bar, 0, 0] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(target.to_le_bytes());
    packet
}

/// The basic-attack bar the field-battle suites use.
#[allow(dead_code)] // Included by sites that only need the movie-blocked families.
pub fn melee(target: u64) -> Vec<u8> {
    ability(target, 1)
}

/// The five request families a live movie refuses. The payloads are the shipped
/// client's own shapes (see the module doc): 26/13 job select, 26/8 interact,
/// 26/10 menu select, 36/10 ability and 42/3 equip.
#[allow(dead_code)] // Some include sites use only the single-request builders.
pub fn blocked_families(target: u64) -> [Vec<u8>; 5] {
    let mut job = vec![26, 0, 13, 0];
    job.extend(1_i32.to_le_bytes());
    let mut equip = vec![42, 0, 3, 0];
    equip.extend(i32::MAX.to_le_bytes()); // unowned instance id
    equip.extend(1_i32.to_le_bytes()); // profile key
    equip.extend(0_i32.to_le_bytes()); // equip slot
    [
        job,
        interact(target),
        select(target, 1),
        ability(target, 0),
        equip,
    ]
}
