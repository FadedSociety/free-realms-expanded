//! The one decoder for the T499 quest-marker packets and the 35/2 spawn they follow. The test
//! suites (`tests/quest_markers.rs`, the gateway view tests) include it by path, so a layout
//! change is made once. Test-only.
//!
//! 35/10 `PlayerUpdatePacketAddNotifications`: u16 35, u16 10, i32 count, then 39-byte rows from
//! offset 8 (u64 guid at 0, kind at 9, state at 13, image id at 17, name string id at 25, sub-text
//! id at 29; reader `008db310`). 35/11
//! `RemoveNotifications`: the same header, then 16-byte rows (u64 guid first; reader
//! `008db550`). A 35/2 AddNpc is read by `startup_world::parse_add_npc` (reader `00921a10`).
#![allow(dead_code)]

use osfr_tools::startup_world::parse_add_npc;

/// One row of a 35/10: where its packet sat in the batch, the actor, its kind, overhead state,
/// marker image, the name string id and the sub-text id the row carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Marker {
    pub index: usize,
    pub guid: u64,
    pub kind: i32,
    pub state: i32,
    pub image: i32,
    pub name: i32,
    pub subtext: i32,
}

fn word(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn guid(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

/// Every 35/10 row and every 35/11 guid in `packets`, in order.
pub fn markers(packets: &[Vec<u8>]) -> (Vec<Marker>, Vec<u64>) {
    let (mut shown, mut cleared) = (Vec::new(), Vec::new());
    for (index, packet) in packets.iter().enumerate() {
        let rows = |size: usize| (0..word(packet, 4) as usize).map(move |row| 8 + row * size);
        if packet.starts_with(&[35, 0, 10, 0]) {
            shown.extend(rows(39).map(|at| Marker {
                index,
                guid: guid(packet, at),
                kind: word(packet, at + 9),
                state: word(packet, at + 13),
                image: word(packet, at + 17),
                name: word(packet, at + 25),
                subtext: word(packet, at + 29),
            }));
        } else if packet.starts_with(&[35, 0, 11, 0]) {
            cleared.extend(rows(16).map(|at| guid(packet, at)));
        }
    }
    (shown, cleared)
}

/// The position of the 35/2 AddNpc that spawns `actor`.
pub fn spawn_index(packets: &[Vec<u8>], actor: u64) -> Option<usize> {
    packets.iter().position(|packet| {
        packet.starts_with(&[35, 0, 2, 0])
            && parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == actor)
    })
}
