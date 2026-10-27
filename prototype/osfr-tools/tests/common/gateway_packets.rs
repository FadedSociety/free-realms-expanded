//! Request bodies and the two gateway envelope helpers shared by the S6 packet
//! tests, the vendor suites and the field-battle fixtures.
//!
//! The `005` tunnel (`wrap`) and its reply reader (`body`) are the client's
//! application framing; the 125 movement body is the gateway's Position event.
//!
//! Client-derived shapes: the chat request reader is `00b11600`; the NPC menu
//! constructor is 26/8 `008b6bc0` (u64 target + bool); the menu selection
//! constructor is 26/10 `00a36f80` (u64 target + i32 choice); 26/13
//! `CommandPacketSetProfile` sender `00938e10` (constructor `008a29f0`, body one
//! i32 profile); 26/6 dialogue response `009f6aa0` (the clicked row's stored id at
//! `this+4+index`; packet constructor `009f5890`); 36/10
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

/// 127/19 `ClientHousingPacketEnterRequest`: the captured 17-byte body
/// `7f 00 13 00 <house guid:8> <unknown:i32> <unknown2:bool>`. Client constructor
/// `sub_AA83C0` (`0x00aa83c0`, base `sub_AA4990(19)`) stores the guid as two dwords at
/// +16/+20, an i32 at +24 and a byte at +28; `sub_AC3FE0` fills the guid and a zero i32.
#[allow(dead_code)] // Included by sites that never enter a house.
pub fn enter_house(lot: u64) -> Vec<u8> {
    let mut packet = vec![127, 0, 19, 0];
    packet.extend(lot.to_le_bytes());
    packet.extend(0i32.to_le_bytes());
    packet.push(0);
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

/// Client 26/6 dialogue response: the clicked row's stored id (`009f6aa0` reads
/// `this+4+index`; `009f5890` constructs the packet), one i32.
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

/// The client's combined application datagram (Group writer `010e6910`, child prefix
/// `010dae40`): `[0, 25]` then each tunnel-wrapped child with a variable-length prefix.
/// `Gateway::handle_at` decodes every child before committing, so one malformed child rejects
/// the whole bundle.
#[allow(dead_code)] // Included by sites that send no combined datagram.
pub fn bundle(messages: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for message in messages {
        if message.len() < 254 {
            bytes.push(message.len() as u8);
        } else if message.len() < 65_535 {
            bytes.push(255);
            bytes.extend_from_slice(&(message.len() as u16).to_be_bytes());
        } else {
            bytes.extend_from_slice(&[255, 255, 255]);
            bytes.extend_from_slice(&(message.len() as u32).to_be_bytes());
        }
        bytes.extend_from_slice(message);
    }
    bytes
}

/// The `005` application tunnel a client uses for its initialization and movement
/// messages: `[5, 0, 1]`, the inner length, then the body.
#[allow(dead_code)] // Included by sites that only read the bodies.
pub fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut message = vec![5, 0, 1];
    message.extend((inner.len() as u32).to_le_bytes());
    message.extend(inner);
    message
}

/// The application body of a gateway reply (its `005` tunnel stripped when the
/// length matches), so a test reads the family the client reads.
#[allow(dead_code)] // Included by sites that never unwrap a reply.
pub fn body(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

/// Client opcode 125 (`Event::Position`; production reader `gateway::parse_message`,
/// native sender `00934a20` in `movement.rs`): the character guid, position, a
/// `[0, 0, 1]` direction and the two zero state bytes.
#[allow(dead_code)] // Included by suites that do not move a character.
pub fn move_to(guid: u64, position: [f32; 3]) -> Vec<u8> {
    let mut packet = 125_u16.to_le_bytes().to_vec();
    packet.extend(guid.to_le_bytes());
    for value in position.into_iter().chain([0.0, 0.0, 1.0]) {
        packet.extend(value.to_le_bytes());
    }
    packet.extend([0, 0]);
    packet
}

/// T744: Marketplace 66/27 store-bundle content request (`00b41a30` -> `00b3dd80` ->
/// `00b37140`): a count, then per entry store id, bundle id, item-id count and the requested item
/// ids. The native client sends one 24-byte request (one 16-byte entry) per bundle whose item
/// definitions it lacks (2026-10-26 trace: 878, 76927 and 76928 only).
#[allow(dead_code)] // Included by suites that never request Marketplace content.
pub fn marketplace_content(entries: &[(u32, u32, &[u32])]) -> Vec<u8> {
    let mut bytes = vec![66, 0, 27, 0];
    bytes.extend((entries.len() as u32).to_le_bytes());
    for (store, bundle, item_ids) in entries {
        bytes.extend(store.to_le_bytes());
        bytes.extend(bundle.to_le_bytes());
        bytes.extend((item_ids.len() as u32).to_le_bytes());
        for id in *item_ids {
            bytes.extend(id.to_le_bytes());
        }
    }
    bytes
}

/// A 35/37 item-definition batch (`sub_8BFC00` reads the payload byte length; `00a3db70` reads
/// the count and the records). Rows carry no per-record length, so the caller compares the
/// concatenated payload with the expected records.
#[allow(dead_code)] // Included by suites that never read a definition batch.
pub fn definition_batch(packet: &[u8]) -> (i32, Vec<u8>) {
    assert_eq!(&packet[..4], &[35, 0, 37, 0], "35/37 header");
    let length = i32::from_le_bytes(packet[4..8].try_into().unwrap());
    let count = i32::from_le_bytes(packet[8..12].try_into().unwrap());
    assert_eq!(
        length as usize,
        packet.len() - 8,
        "35/37 payload length counts the record-count dword"
    );
    assert!(count >= 1, "an empty batch is never emitted");
    (count, packet[12..].to_vec())
}

/// A Marketplace 66/3 place-order request with one detail, as the client's order serializer
/// `00b3fc60`/`00b3af80` writes it: two empty strings, the tracking id, the locale, three more
/// strings, one detail (bundle id, store 1, quantity, tint string) and a trailing string. The
/// shipped bundle id is the item id.
#[allow(dead_code)] // Included by suites that never order from the Marketplace.
pub fn marketplace_order(tracking: u32, bundle: u32, quantity: u32, tint: &str) -> Vec<u8> {
    fn put_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend((value.len() as i32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    let mut bytes = vec![66, 0, 3, 0];
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(tracking.to_le_bytes());
    put_string(&mut bytes, "en_US");
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(bundle.to_le_bytes());
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(quantity.to_le_bytes());
    put_string(&mut bytes, tint);
    put_string(&mut bytes, "");
    bytes
}

/// The 66/4 result: the second dword after the header (`00b2ded0` obj+0x1C, T453).
#[allow(dead_code)] // Included by suites that never read a Marketplace order reply.
pub fn order_result(packets: &[Vec<u8>]) -> u32 {
    packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 4, 0]))
        .map(|packet| u32::from_le_bytes(packet[8..12].try_into().unwrap()))
        .expect("order response exists")
}

/// The Station Cash balance in a 66/11 wallet reply (`00b3e120`).
#[allow(dead_code)] // Included by suites that never read a wallet reply.
pub fn wallet_balance(packets: &[Vec<u8>]) -> u32 {
    packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 11, 0]))
        .map(|packet| u32::from_le_bytes(packet[9..13].try_into().unwrap()))
        .expect("wallet response exists")
}
