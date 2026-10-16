use osfr_tools::players;

use players::{
    InterestPolicy, MessageKind, OutboundMessage, PlayerRegistry, SessionId, ZoneInstance,
};

fn registry(radius: f32, max_visible_count: usize) -> PlayerRegistry {
    PlayerRegistry::new(InterestPolicy::new(radius, max_visible_count).unwrap())
}

fn zone(zone_id: u32, instance_id: u64) -> ZoneInstance {
    ZoneInstance {
        zone_id,
        instance_id,
    }
}

fn join(
    registry: &mut PlayerRegistry,
    session: u64,
    account: u64,
    character: u64,
    zone: ZoneInstance,
    position: [f32; 3],
    heading: f32,
) -> Vec<OutboundMessage> {
    registry
        .join(
            SessionId(session),
            account,
            character,
            zone,
            position,
            heading,
        )
        .unwrap()
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut out = vec![5, 0, 1];
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn expected_add_pc(guid: u64, position: [f32; 3], heading: f32) -> Vec<u8> {
    let mut inner = vec![35, 0, 1, 0];
    inner.extend_from_slice(&guid.to_le_bytes());
    for value in [0_i32, 0, 0] {
        inner.extend_from_slice(&value.to_le_bytes());
    }
    for _ in 0..2 {
        inner.extend_from_slice(&0_i32.to_le_bytes());
    }
    inner.extend_from_slice(&0_i32.to_le_bytes()); // model
    for _ in 0..3 {
        inner.extend_from_slice(&0_i32.to_le_bytes());
    }
    for value in [position[0], position[1], position[2], 1.0] {
        inner.extend_from_slice(&value.to_le_bytes());
    }
    for value in [0.0, (heading * 0.5).sin(), 0.0, (heading * 0.5).cos()] {
        inner.extend_from_slice(&value.to_le_bytes());
    }
    inner.extend_from_slice(&0_i32.to_le_bytes()); // attachments
    for _ in 0..2 {
        inner.extend_from_slice(&0_i32.to_le_bytes());
    }
    for _ in 0..3 {
        inner.extend_from_slice(&0_i32.to_le_bytes());
    }
    for _ in 0..3 {
        inner.extend_from_slice(&0_i32.to_le_bytes());
    }
    inner.extend_from_slice(&0.0_f32.to_le_bytes());
    inner.extend_from_slice(&[0, 0, 0]);
    inner.extend_from_slice(&0_i32.to_le_bytes()); // temporary appearance
    inner.extend_from_slice(&0_i32.to_le_bytes()); // guild count
    inner.extend_from_slice(&0_i32.to_le_bytes()); // active profile
    for _ in 0..4 {
        inner.extend_from_slice(&0_i32.to_le_bytes());
    }
    inner.extend_from_slice(&0_i32.to_le_bytes()); // effect tag count
    inner.extend_from_slice(&0_u64.to_le_bytes()); // mount guid
    for _ in 0..2 {
        inner.extend_from_slice(&0_i32.to_le_bytes());
    }
    inner.extend_from_slice(&0.0_f32.to_le_bytes());
    inner.extend_from_slice(&0_i32.to_le_bytes());
    inner.extend_from_slice(&0.0_f32.to_le_bytes());
    for _ in 0..3 {
        inner.extend_from_slice(&0_i32.to_le_bytes());
    }
    wrap(&inner)
}

fn expected_position(guid: u64, position: [f32; 3], heading: f32) -> Vec<u8> {
    let mut inner = vec![125, 0];
    inner.extend_from_slice(&guid.to_le_bytes());
    for value in position {
        inner.extend_from_slice(&value.to_le_bytes());
    }
    for value in [heading.sin(), 0.0, heading.cos()] {
        inner.extend_from_slice(&value.to_le_bytes());
    }
    inner.extend_from_slice(&[0, 0]);
    wrap(&inner)
}

fn expected_remove_player(guid: u64) -> Vec<u8> {
    let mut inner = vec![35, 0, 3, 0, 0, 0];
    inner.extend_from_slice(&guid.to_le_bytes());
    wrap(&inner)
}

fn message<'a>(messages: &'a [OutboundMessage], recipient: u64, kind: MessageKind) -> &'a [u8] {
    &messages
        .iter()
        .find(|message| message.recipient == SessionId(recipient) && message.kind == kind)
        .expect("expected message")
        .bytes
}

#[test]
fn two_sessions_spawn_move_and_disconnect_with_exact_wire_bytes() {
    let mut registry = registry(30.0, 10);
    let shared_zone = zone(7, 11);
    assert!(
        join(
            &mut registry,
            1,
            101,
            1001,
            shared_zone,
            [1.0, 2.0, 3.0],
            0.0,
        )
        .is_empty()
    );

    let joined = join(
        &mut registry,
        2,
        102,
        1002,
        shared_zone,
        [4.0, 5.0, 6.0],
        1.0,
    );
    assert_eq!(joined.len(), 2);
    assert_eq!(
        message(&joined, 1, MessageKind::Spawn),
        expected_add_pc(1002, [4.0, 5.0, 6.0], 1.0)
    );
    assert_eq!(
        message(&joined, 2, MessageKind::Spawn),
        expected_add_pc(1001, [1.0, 2.0, 3.0], 0.0)
    );

    let moved = registry
        .update_pose(SessionId(2), shared_zone, [7.0, 8.0, 9.0], 1.5)
        .unwrap();
    assert_eq!(moved.len(), 1);
    assert_eq!(
        message(&moved, 1, MessageKind::Position),
        expected_position(1002, [7.0, 8.0, 9.0], 1.5)
    );

    let left = registry.leave(SessionId(2));
    assert_eq!(left.len(), 1);
    assert_eq!(
        message(&left, 1, MessageKind::Despawn),
        expected_remove_player(1002)
    );
}

#[test]
fn three_sessions_receive_each_others_spawns_moves_and_despawns() {
    let mut registry = registry(50.0, 10);
    let shared_zone = zone(4, 9);
    join(&mut registry, 1, 11, 101, shared_zone, [0.0; 3], 0.0);
    join(
        &mut registry,
        2,
        12,
        102,
        shared_zone,
        [1.0, 0.0, 0.0],
        0.25,
    );

    let third = join(&mut registry, 3, 13, 103, shared_zone, [2.0, 0.0, 0.0], 0.5);
    assert_eq!(third.len(), 4);
    assert_eq!(
        message(&third, 1, MessageKind::Spawn),
        expected_add_pc(103, [2.0, 0.0, 0.0], 0.5)
    );
    assert_eq!(
        message(&third, 2, MessageKind::Spawn),
        expected_add_pc(103, [2.0, 0.0, 0.0], 0.5)
    );
    assert_eq!(
        message(&third, 2, MessageKind::Spawn),
        expected_add_pc(103, [2.0, 0.0, 0.0], 0.5)
    );
    assert_eq!(
        message(&third, 3, MessageKind::Spawn),
        expected_add_pc(101, [0.0; 3], 0.0)
    );
    assert!(third.iter().any(|item| {
        item.recipient == SessionId(3)
            && item.kind == MessageKind::Spawn
            && item.bytes == expected_add_pc(102, [1.0, 0.0, 0.0], 0.25)
    }));

    let moved = registry
        .update_pose(SessionId(3), shared_zone, [3.0, 0.0, 0.0], 0.75)
        .unwrap();
    assert_eq!(moved.len(), 2);
    assert_eq!(
        message(&moved, 1, MessageKind::Position),
        expected_position(103, [3.0, 0.0, 0.0], 0.75)
    );
    assert_eq!(
        message(&moved, 2, MessageKind::Position),
        expected_position(103, [3.0, 0.0, 0.0], 0.75)
    );

    let left = registry.leave(SessionId(3));
    assert_eq!(left.len(), 2);
    assert_eq!(
        message(&left, 1, MessageKind::Despawn),
        expected_remove_player(103)
    );
    assert_eq!(
        message(&left, 2, MessageKind::Despawn),
        expected_remove_player(103)
    );
}

#[test]
fn different_zone_instances_have_no_visibility() {
    let mut registry = registry(50.0, 10);
    assert!(join(&mut registry, 1, 11, 101, zone(4, 9), [0.0; 3], 0.0).is_empty());
    assert!(join(&mut registry, 2, 12, 102, zone(4, 10), [0.0; 3], 0.0).is_empty());
    assert!(
        registry
            .update_pose(SessionId(2), zone(5, 10), [1.0; 3], 0.0)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn zone_change_despawns_old_view_and_spawns_new_view() {
    let mut registry = registry(50.0, 10);
    join(&mut registry, 1, 11, 101, zone(1, 1), [0.0; 3], 0.0);
    join(&mut registry, 2, 12, 102, zone(1, 1), [1.0; 3], 0.0);
    join(&mut registry, 3, 13, 103, zone(2, 1), [2.0; 3], 0.0);

    let moved = registry
        .update_pose(SessionId(2), zone(2, 1), [3.0; 3], 0.5)
        .unwrap();
    assert_eq!(moved.len(), 4);
    assert_eq!(
        message(&moved, 1, MessageKind::Despawn),
        expected_remove_player(102)
    );
    assert_eq!(
        message(&moved, 2, MessageKind::Despawn),
        expected_remove_player(101)
    );
    assert_eq!(
        message(&moved, 2, MessageKind::Spawn),
        expected_add_pc(103, [2.0; 3], 0.0)
    );
    assert_eq!(
        message(&moved, 3, MessageKind::Spawn),
        expected_add_pc(102, [3.0; 3], 0.5)
    );
}

#[test]
fn duplicate_account_or_character_guid_is_rejected() {
    let mut registry = registry(20.0, 10);
    let shared_zone = zone(1, 1);
    join(&mut registry, 1, 11, 101, shared_zone, [0.0; 3], 0.0);

    assert!(
        registry
            .join(SessionId(2), 12, 101, shared_zone, [1.0; 3], 0.0)
            .is_err()
    );
    assert!(
        registry
            .join(SessionId(3), 11, 103, shared_zone, [1.0; 3], 0.0)
            .is_err()
    );
    assert!(registry.get(SessionId(2)).is_none());
    assert!(registry.get(SessionId(3)).is_none());
}

#[test]
fn radius_boundary_is_inclusive_and_one_step_outside_is_hidden() {
    let mut registry = registry(10.0, 10);
    let shared_zone = zone(1, 1);
    join(&mut registry, 1, 11, 101, shared_zone, [0.0; 3], 0.0);
    let at_boundary = join(
        &mut registry,
        2,
        12,
        102,
        shared_zone,
        [10.0, 0.0, 0.0],
        0.0,
    );
    assert_eq!(at_boundary.len(), 2);
    assert_eq!(
        message(&at_boundary, 1, MessageKind::Spawn),
        expected_add_pc(102, [10.0, 0.0, 0.0], 0.0)
    );
    assert_eq!(
        message(&at_boundary, 2, MessageKind::Spawn),
        expected_add_pc(101, [0.0; 3], 0.0)
    );

    let moved = registry
        .update_pose(SessionId(2), shared_zone, [10.000_001, 0.0, 0.0], 0.0)
        .unwrap();
    assert_eq!(moved.len(), 2);
    assert_eq!(
        message(&moved, 1, MessageKind::Despawn),
        expected_remove_player(102)
    );
    assert_eq!(
        message(&moved, 2, MessageKind::Despawn),
        expected_remove_player(101)
    );
}

#[test]
fn visible_count_selects_nearest_then_session_id() {
    let mut registry = registry(100.0, 1);
    let shared_zone = zone(1, 1);
    join(&mut registry, 1, 11, 101, shared_zone, [0.0; 3], 0.0);
    join(
        &mut registry,
        2,
        12,
        102,
        shared_zone,
        [20.0, 0.0, 0.0],
        0.0,
    );
    let third = join(
        &mut registry,
        3,
        13,
        103,
        shared_zone,
        [10.0, 0.0, 0.0],
        0.0,
    );

    assert_eq!(third.len(), 5);
    assert!(third.iter().any(|item| {
        item.recipient == SessionId(1)
            && item.kind == MessageKind::Spawn
            && item.bytes == expected_add_pc(103, [10.0, 0.0, 0.0], 0.0)
    }));
    assert!(third.iter().any(|item| {
        item.recipient == SessionId(2)
            && item.kind == MessageKind::Despawn
            && item.bytes == expected_remove_player(101)
    }));
    assert!(third.iter().any(|item| {
        item.recipient == SessionId(1)
            && item.kind == MessageKind::Despawn
            && item.bytes == expected_remove_player(102)
    }));
    assert!(third.iter().any(|item| {
        item.recipient == SessionId(2)
            && item.kind == MessageKind::Spawn
            && item.bytes == expected_add_pc(103, [10.0, 0.0, 0.0], 0.0)
    }));
    assert!(third.iter().any(|item| {
        item.recipient == SessionId(3)
            && item.kind == MessageKind::Spawn
            && item.bytes == expected_add_pc(101, [0.0; 3], 0.0)
    }));
}
