//! T763/T770 (F785): the card service's client frames, hand-built in the DLL writers' field
//! order, shared by `tests/card_service.rs` and the gateway's card listener tests so both check
//! the same bytes and a change to the server's own string/header/frame writers cannot move a
//! client fixture with it. Test-only.
//!
//! Client evidence (`FreeRealmsTCG.dll`): string writer `10097b40`, per-level header writer
//! `1019de90`, 414 sender `101935d0` -> writer `10121d20`, 411 sender `103c9140` -> writer
//! `1017b700` (field names `1017b4b0`), ping writer `10190020`, frame writer `101921e0`.
#![allow(dead_code)] // each includer uses a subset

use osfr_tools::card_service::put_int;

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
