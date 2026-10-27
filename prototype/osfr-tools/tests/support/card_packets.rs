//! T763/T770 (F785): the card service's client frames, hand-built in the DLL writers' field
//! order, and the server replies the tests read back (the 39/0x28 launch ticket and the 81
//! login results), shared by `tests/card_service.rs`, `tests/card_launch.rs` and the gateway's
//! card listener tests so both check the same bytes and a change to the server's own
//! string/header/frame writers cannot move a client fixture with it. Test-only.
//!
//! Client evidence (`FreeRealmsTCG.dll`): string writer `10097b40`, per-level header writer
//! `1019de90`, 414 sender `101935d0` -> writer `10121d20`, 411 sender `103c9140` -> writer
//! `1017b700` (field names `1017b4b0`), ping writer `10190020`, frame writer `101921e0`;
//! client-side readers: 39/0x28 `009b7f30`, 81 `10179c70`.
#![allow(dead_code)] // each includer uses a subset

use osfr_tools::card_service::{LOGIN_RESULTS, Reader, put_int};

/// The 81 reply read the way the client's reader `10179c70` reads it at version 5: success, code,
/// account id, the flag and the word its handler passes on (both 0, R), then the session id.
pub fn read_results(body: &[u8]) -> (bool, i32, i32, String) {
    let mut reader = Reader::new(body);
    let version = reader.headers(LOGIN_RESULTS).unwrap();
    assert_eq!(version, 5);
    let success = reader.int().unwrap() != 0;
    let code = reader.int().unwrap(); // version >= 3
    let account = reader.int().unwrap();
    reader.int().unwrap(); // version > 1
    reader.int().unwrap(); // version > 3
    let session = reader.string().unwrap(); // version > 4
    assert!(reader.finished(), "no zlib blob below version 6");
    (success, code, account, session)
}

/// The routing words the retail client puts after the length on a logged-in frame.
pub const ROUTING: [u32; 3] = [1_000_000_000, 0, 0];

/// A compact integer. This one calls the server's `put_int`; `compact_integers_match_the_dll_writer`
/// (`tests/card_service.rs`) holds it to hand-written bytes from the DLL writer `10097a40`.
pub fn int(value: i32) -> Vec<u8> {
    let mut out = Vec::new();
    put_int(&mut out, value);
    out
}

/// A client string as `10097b40` writes it: length, text, NUL. The compact length is spelled
/// out by hand: one byte below 16, else `0x20 | low nibble` and the rest.
pub fn text(value: &str) -> Vec<u8> {
    let length = value.len();
    assert!(length < 256, "fixture strings stay short");
    let prefix = if length < 16 {
        vec![length as u8]
    } else {
        vec![0x20 | (length & 0xF) as u8, (length >> 4) as u8]
    };
    [prefix, value.as_bytes().to_vec(), vec![0]].concat()
}

/// Three class levels of `[id, version]` as the client writes them (`1019de90`).
pub fn headers(id: &[u8], version: u8) -> Vec<u8> {
    [id, &[version], id, &[version], id, &[version]].concat()
}

/// The client's 414 (`101935d0` -> `10121d20`): session id, user name, empty host, 0, false, 87.
pub fn gateway_request() -> Vec<u8> {
    [
        headers(&[0x2E, 0x19], 1),
        text("w-ticket"),
        text("tester"),
        text(""),
        vec![0x00, 0x00, 0x27, 0x05],
    ]
    .concat()
}

/// The client's 411 at version 8 (`103c9140` -> `1017b700`), fields in the writer's order with
/// the names from `1017b4b0`.
pub fn session_login(character_id: &str, ticket: &str) -> Vec<u8> {
    [
        headers(&[0x2B, 0x19], 8),
        text("w-session"), // SessionID (the game's --sessionID)
        // {1: 1314, 2: 7, 4: 9}: count 3, then key/value pairs.
        vec![0x03, 0x01, 0x22, 0x52, 0x02, 0x07, 0x04, 0x09],
        text("winXP"),  // Platform
        text(""),       // Match Challenge Key
        vec![0x01],     // Is Parent Session ID (the exe passes --host=fr)
        text("FR"),     // Login Client
        text("en_US"),  // Locale
        text("Tester"), // CharacterName
        text(character_id),
        text(ticket),
        text("STATION"), // Login Namespace
    ]
    .concat()
}

/// The client's ping (`10190020`): one header (112, 1), then the spoof value.
pub fn ping(spoof: i32) -> Vec<u8> {
    [int(112), int(1), int(spoof)].concat()
}

/// One client frame: `htonl(length)`, then the routing words when routed, then the body
/// (`101921e0`; the 414 is sent unrouted).
pub fn frame(routing: Option<[u32; 3]>, body: &[u8]) -> Vec<u8> {
    let words: Vec<u8> = routing
        .into_iter()
        .flatten()
        .flat_map(u32::to_be_bytes)
        .collect();
    let mut out = ((words.len() + body.len()) as u32).to_be_bytes().to_vec();
    out.extend(words);
    out.extend(body);
    out
}

/// The 64-bit launch ticket a 39/0x28 carries (client reader `009b7f30`, server writer
/// `card_game::card_start_packet`): the run's 15-byte header, then the two i32 halves, low then
/// high.
pub fn ticket_of(packet: &[u8]) -> u64 {
    assert_eq!(
        (packet[0], packet[2], packet.len()),
        (39, 0x28, 15 + 8),
        "one 39/0x28 ticket packet"
    );
    (u64::from(u32::from_le_bytes(packet[19..23].try_into().unwrap())) << 32)
        | u64::from(u32::from_le_bytes(packet[15..19].try_into().unwrap()))
}

/// The one 39/0x28 among `replies` (the answer to the client's 39/37), as [`ticket_of`]. A bin
/// test's gateway replies may still carry the `005` application tunnel, so the packet is found
/// by its header inside the reply.
pub fn ticket_in(replies: &[Vec<u8>]) -> u64 {
    for reply in replies {
        let Some(at) = reply.windows(3).position(|w| w == [39, 0, 0x28]) else {
            continue;
        };
        if let Some(packet) = reply.get(at..at + 15 + 8) {
            return ticket_of(packet);
        }
    }
    panic!(
        "a 39/0x28 answers the 39/37; got reply lengths {:?}",
        replies.iter().map(Vec::len).collect::<Vec<_>>()
    );
}
