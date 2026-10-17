//! W167: the native AddPc mount block. A rider's later AddPc carries the ride (GUID, seat, queue,
//! lift) so a viewer who meets the rider afterwards mounts it the way the client does itself
//! (`0092f460` case 1 -> `SetPendingMountInfo` `0095ad80` -> `Mount` `00972d30`). Needs the
//! `apply_follow_patch.py` edit of `players.rs`; drop this file in as `tests/mount_follow.rs`.
use osfr_tools::{
    mounts::MountTail,
    players::{
        InterestPolicy, MessageKind, OutboundMessage, PlayerRegistry, SessionId, ZoneInstance,
    },
};

const ZONE: ZoneInstance = ZoneInstance {
    zone_id: 1,
    instance_id: 1,
};
const RIDER: u64 = 7;
const RIDE: u64 = RIDER ^ 0xc000_0000_0000_0000;

fn registry() -> PlayerRegistry {
    PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap())
}

fn join(players: &mut PlayerRegistry, id: u64, x: f32) -> Vec<OutboundMessage> {
    players
        .join(SessionId(id), id, id, ZONE, [x, 2.0, 30.0], 0.5)
        .unwrap()
}

/// The AddPc (35/1) of `guid` that `viewer` was sent, if any.
fn add_pc_of(messages: &[OutboundMessage], viewer: u64, guid: u64) -> Option<&[u8]> {
    messages
        .iter()
        .filter(|m| m.recipient == SessionId(viewer) && m.kind == MessageKind::Spawn)
        .map(|m| m.bytes.as_slice())
        .find(|bytes| bytes.get(11..19) == Some(&guid.to_le_bytes()[..]))
}

/// The mount block of an AddPc frame: it ends 40 bytes before the end (wield type, VIP progress
/// and three more dwords follow it), and is the GUID, seat, queue position and lift.
fn tail_of(add_pc: &[u8]) -> MountTail {
    let block = &add_pc[add_pc.len() - 40..add_pc.len() - 20];
    MountTail {
        ride_guid: u64::from_le_bytes(block[..8].try_into().unwrap()),
        seat: i32::from_le_bytes(block[8..12].try_into().unwrap()),
        queue: i32::from_le_bytes(block[12..16].try_into().unwrap()),
        lift: f32::from_le_bytes(block[16..20].try_into().unwrap()),
    }
}

fn seated() -> MountTail {
    MountTail {
        ride_guid: RIDE,
        seat: 0,
        queue: 1,
        lift: 0.0,
    }
}

#[test]
fn an_unmounted_player_keeps_the_sentinel_tail() {
    let mut players = registry();
    join(&mut players, RIDER, 10.0);
    let later = join(&mut players, 8, 20.0);
    let add_pc = add_pc_of(&later, 8, RIDER).expect("the rider is shown to the newcomer");
    assert_eq!(tail_of(add_pc), MountTail::NONE);
    assert_eq!(MountTail::NONE.ride_guid, 0);
    assert_eq!((MountTail::NONE.seat, MountTail::NONE.queue), (-1, -1));
}

#[test]
fn a_viewer_who_arrives_later_gets_the_riders_mount_block() {
    let mut players = registry();
    join(&mut players, RIDER, 10.0);
    players.set_mount(SessionId(RIDER), seated());
    let later = join(&mut players, 8, 20.0);
    let add_pc = add_pc_of(&later, 8, RIDER).expect("the rider is shown to the newcomer");
    assert_eq!(tail_of(add_pc), seated());
    // The frame keeps its fixed size: only the five tail values change.
    let plain = {
        let mut other = registry();
        join(&mut other, RIDER, 10.0);
        let frames = join(&mut other, 8, 20.0);
        add_pc_of(&frames, 8, RIDER).unwrap().to_vec()
    };
    assert_eq!(add_pc.len(), plain.len());
    let (head, rest) = add_pc.split_at(add_pc.len() - 40);
    assert_eq!(head, &plain[..plain.len() - 40]);
    assert_eq!(rest[20..], plain[plain.len() - 20..]);
}

#[test]
fn setting_a_mount_sends_nothing_and_clearing_it_restores_the_sentinel() {
    let mut players = registry();
    join(&mut players, RIDER, 10.0);
    join(&mut players, 8, 20.0);
    players.set_mount(SessionId(RIDER), seated());
    // A rider who walks out of range and back is shown again, still seated; after the ride is
    // put away the next AddPc is the plain one.
    let away = players
        .update_pose(SessionId(RIDER), ZONE, [900.0, 2.0, 30.0], 0.5)
        .unwrap();
    assert!(
        away.iter()
            .any(|m| m.recipient == SessionId(8) && m.kind == MessageKind::Despawn)
    );
    let back = players
        .update_pose(SessionId(RIDER), ZONE, [30.0, 2.0, 30.0], 0.5)
        .unwrap();
    assert_eq!(tail_of(add_pc_of(&back, 8, RIDER).unwrap()), seated());

    players.set_mount(SessionId(RIDER), MountTail::NONE);
    players
        .update_pose(SessionId(RIDER), ZONE, [900.0, 2.0, 30.0], 0.5)
        .unwrap();
    let again = players
        .update_pose(SessionId(RIDER), ZONE, [30.0, 2.0, 30.0], 0.5)
        .unwrap();
    assert_eq!(
        tail_of(add_pc_of(&again, 8, RIDER).unwrap()),
        MountTail::NONE
    );

    // An unknown session is ignored.
    players.set_mount(SessionId(999), seated());
}
