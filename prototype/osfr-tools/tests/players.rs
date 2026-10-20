use osfr_tools::players;

use players::{
    InterestPolicy, MessageKind, OutboundMessage, PlayerPresentation, PlayerRegistry,
    RegistryError, SessionId, ZoneInstance,
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

fn join_presented(
    registry: &mut PlayerRegistry,
    session: u64,
    account: u64,
    character: u64,
    zone: ZoneInstance,
    position: [f32; 3],
    heading: f32,
    mut presentation: PlayerPresentation,
) -> Vec<OutboundMessage> {
    presentation.character_guid = character;
    registry
        .join_with_presentation(
            SessionId(session),
            account,
            character,
            zone,
            position,
            heading,
            presentation,
        )
        .unwrap()
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut out = vec![5, 0, 1];
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn assert_default_add_pc(bytes: &[u8], guid: u64, position: [f32; 3], heading: f32) {
    assert_eq!(&bytes[..3], &[5, 0, 1]);
    assert_eq!(
        u32::from_le_bytes(bytes[3..7].try_into().unwrap()) as usize,
        bytes.len() - 7
    );
    let inner = &bytes[7..];
    assert_eq!(inner.len(), 195);
    assert_eq!(u16::from_le_bytes(inner[..2].try_into().unwrap()), 35);
    assert_eq!(u16::from_le_bytes(inner[2..4].try_into().unwrap()), 1);
    assert_eq!(u64::from_le_bytes(inner[4..12].try_into().unwrap()), guid);
    for (index, expected) in position.into_iter().chain([1.0]).enumerate() {
        assert!((read_f32(inner, 48 + index * 4) - expected).abs() < 1e-6);
    }
    for (index, expected) in [heading.sin(), 0.0, heading.cos(), 0.0]
        .into_iter()
        .enumerate()
    {
        assert!((read_f32(inner, 64 + index * 4) - expected).abs() < 1e-6);
    }
}

fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn oracle_presentation() -> PlayerPresentation {
    let mut attachments = 1_i32.to_le_bytes().to_vec();
    for value in ["mdl", "", "t"] {
        attachments.extend_from_slice(&(value.len() as i32).to_le_bytes());
        attachments.extend_from_slice(value.as_bytes());
    }
    for value in [7_i32, 8, 9] {
        attachments.extend_from_slice(&value.to_le_bytes());
    }
    PlayerPresentation {
        character_guid: 0x0102_0304_0506_0708,
        name_ids: [0x0102_0304, 0x1122_3344, -17],
        first_name: String::new(),
        last_name: String::new(),
        model: 0x5566_7788,
        head: "head".into(),
        hair: "hair".into(),
        hair_color: 21,
        eye_color: 22,
        skin_tone: "skin".into(),
        face_paint: "paint".into(),
        model_customization: "custom".into(),
        max_movement_speed: 12.5,
        underage: true,
        member: true,
        referee: false,
        active_profile: 33,
        attachments_wire: attachments,
        wield_type: 44,
    }
}

fn bytes_from_hex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

// Generated with the `PC` schema embedded in notes/Research/AddPc Layout.md.
const ADD_PC_ORACLE_HEX: &str = concat!(
    "050001f60000002300010008070605040302010403020144332211efffffff0000000000000000887766550000000000000000000000000000803f00000040000040400000803f",
    "00000000000000000000803f0000000001000000030000006d646c0000000001000000740700000008000000090000000400000068656164040000006861697215000000160000000000000004000000736b696e",
    "050000007061696e7406000000637573746f6d0000484101010000000000000000002100000000000000000000000000000000000000000000000000000000000000ffffffffffffffff000000002c00000000000000000000000000000000000000"
);

fn oracle_packet() -> Vec<u8> {
    bytes_from_hex(ADD_PC_ORACLE_HEX)
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

fn presentation_with(model: &str, profile: i32) -> PlayerPresentation {
    let mut presentation = oracle_presentation();
    presentation.character_guid = 0;
    presentation.active_profile = profile;
    let mut attachments = 1_i32.to_le_bytes().to_vec();
    for value in [model, "", "t"] {
        attachments.extend_from_slice(&(value.len() as i32).to_le_bytes());
        attachments.extend_from_slice(value.as_bytes());
    }
    for value in [7_i32, 8, 9] {
        attachments.extend_from_slice(&value.to_le_bytes());
    }
    presentation.attachments_wire = attachments;
    presentation
}

fn message<'a>(messages: &'a [OutboundMessage], recipient: u64, kind: MessageKind) -> &'a [u8] {
    &messages
        .iter()
        .find(|message| message.recipient == SessionId(recipient) && message.kind == kind)
        .expect("expected message")
        .bytes
}

fn spawn_message(messages: &[OutboundMessage], recipient: u64, guid: u64) -> &[u8] {
    messages
        .iter()
        .find(|message| {
            message.recipient == SessionId(recipient)
                && message.kind == MessageKind::Spawn
                && u64::from_le_bytes(message.bytes[11..19].try_into().unwrap()) == guid
        })
        .expect("expected spawn")
        .bytes
        .as_slice()
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
    assert_default_add_pc(
        message(&joined, 1, MessageKind::Spawn),
        1002,
        [4.0, 5.0, 6.0],
        1.0,
    );
    assert_default_add_pc(
        message(&joined, 2, MessageKind::Spawn),
        1001,
        [1.0, 2.0, 3.0],
        0.0,
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
fn contains_tracks_the_live_registry_membership() {
    let mut registry = registry(30.0, 10);
    assert!(!registry.contains(SessionId(1)));
    join(&mut registry, 1, 101, 1001, zone(7, 11), [0.0; 3], 0.0);
    assert!(registry.contains(SessionId(1)));
    registry.leave(SessionId(1));
    assert!(!registry.contains(SessionId(1)));
}

/// T027: a live outfit/job change updates the AddPc snapshot a later viewer
/// gets; a snapshot bound to another character is refused.
#[test]
fn a_live_presentation_change_reaches_a_later_viewer() {
    let mut registry = registry(30.0, 10);
    let shared_zone = zone(7, 11);
    join_presented(
        &mut registry,
        1,
        101,
        1001,
        shared_zone,
        [1.0, 2.0, 3.0],
        0.0,
        presentation_with("oldmodel", 1),
    );
    let mut updated = presentation_with("newmodel", 43);
    updated.character_guid = 1001;
    registry.set_presentation(SessionId(1), updated).unwrap();
    let joined = join_presented(
        &mut registry,
        2,
        102,
        1002,
        shared_zone,
        [4.0, 5.0, 6.0],
        1.0,
        presentation_with("viewer", 1),
    );
    let spawn = spawn_message(&joined, 2, 1001);
    assert!(
        spawn.windows(8).any(|window| window == b"newmodel"),
        "the late viewer got the stale outfit"
    );
    assert!(!spawn.windows(8).any(|window| window == b"oldmodel"));
    assert_eq!(
        registry
            .get(SessionId(1))
            .unwrap()
            .presentation
            .active_profile,
        43
    );

    // A snapshot bound to another character, or an unknown session, is refused.
    let mut wrong_character = presentation_with("other", 1);
    wrong_character.character_guid = 1002;
    assert_eq!(
        registry.set_presentation(SessionId(1), wrong_character),
        Err(RegistryError::InvalidPresentation)
    );
    assert_eq!(
        registry.set_presentation(SessionId(9), presentation_with("other", 1)),
        Err(RegistryError::UnknownSession)
    );
    assert_eq!(
        registry
            .get(SessionId(1))
            .unwrap()
            .presentation
            .active_profile,
        43,
        "a refused update changed the live snapshot"
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
    assert_default_add_pc(
        message(&third, 1, MessageKind::Spawn),
        103,
        [2.0, 0.0, 0.0],
        0.5,
    );
    assert_default_add_pc(
        message(&third, 2, MessageKind::Spawn),
        103,
        [2.0, 0.0, 0.0],
        0.5,
    );
    assert_default_add_pc(spawn_message(&third, 3, 101), 101, [0.0; 3], 0.0);
    assert_default_add_pc(spawn_message(&third, 3, 102), 102, [1.0, 0.0, 0.0], 0.25);

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
    assert_default_add_pc(message(&moved, 2, MessageKind::Spawn), 103, [2.0; 3], 0.0);
    assert_default_add_pc(message(&moved, 3, MessageKind::Spawn), 102, [3.0; 3], 0.5);
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
    assert_default_add_pc(
        message(&at_boundary, 1, MessageKind::Spawn),
        102,
        [10.0, 0.0, 0.0],
        0.0,
    );
    assert_default_add_pc(
        message(&at_boundary, 2, MessageKind::Spawn),
        101,
        [0.0; 3],
        0.0,
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
    assert_default_add_pc(spawn_message(&third, 1, 103), 103, [10.0, 0.0, 0.0], 0.0);
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
    assert_default_add_pc(spawn_message(&third, 2, 103), 103, [10.0, 0.0, 0.0], 0.0);
    assert_default_add_pc(spawn_message(&third, 3, 101), 101, [0.0; 3], 0.0);
}

#[test]
fn add_pc_matches_the_research_oracle_schema_byte_for_byte() {
    let mut registry = registry(20.0, 10);
    let shared_zone = zone(8, 13);
    join(&mut registry, 1, 101, 1001, shared_zone, [0.0; 3], 0.0);
    let joined = join_presented(
        &mut registry,
        2,
        102,
        0x0102_0304_0506_0708,
        shared_zone,
        [1.0, 2.0, 3.0],
        0.0,
        oracle_presentation(),
    );

    assert_eq!(message(&joined, 1, MessageKind::Spawn), oracle_packet());
}

#[test]
fn add_pc_direction_matches_native_yaw_for_cardinal_headings() {
    for heading in [
        0.0,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        -std::f32::consts::FRAC_PI_2,
    ] {
        let mut registry = registry(20.0, 10);
        let shared_zone = zone(8, 13);
        join(&mut registry, 1, 101, 1001, shared_zone, [0.0; 3], 0.0);
        let joined = join_presented(
            &mut registry,
            2,
            102,
            1002,
            shared_zone,
            [0.0; 3],
            heading,
            PlayerPresentation::default(),
        );
        let bytes = message(&joined, 1, MessageKind::Spawn);
        let inner = &bytes[7..];
        let actual = [
            read_f32(inner, 64),
            read_f32(inner, 68),
            read_f32(inner, 72),
            read_f32(inner, 76),
        ];
        let expected = [heading.sin(), 0.0, heading.cos(), 0.0];
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-6);
        }
    }
}

#[test]
fn state_only_changes_relay_both_opaque_bytes_to_other_viewers() {
    let mut registry = registry(20.0, 10);
    let shared_zone = zone(8, 13);
    join(
        &mut registry,
        1,
        101,
        1001,
        shared_zone,
        [1.0, 2.0, 3.0],
        0.5,
    );
    join(
        &mut registry,
        2,
        102,
        1002,
        shared_zone,
        [4.0, 5.0, 6.0],
        1.0,
    );

    let state_changed = registry
        .update_pose_with_state(SessionId(2), shared_zone, [4.0, 5.0, 6.0], 1.0, 2, 0)
        .unwrap();
    assert_eq!(state_changed.len(), 1);
    let state_wire = message(&state_changed, 1, MessageKind::Position);
    assert_eq!(state_wire.len(), 43);
    assert_eq!(&state_wire[7 + 34..7 + 36], &[2, 0]);
    assert!(
        !state_changed
            .iter()
            .any(|item| item.recipient == SessionId(2))
    );

    let unknown_changed = registry
        .update_pose_with_state(SessionId(2), shared_zone, [4.0, 5.0, 6.0], 1.0, 2, 255)
        .unwrap();
    assert_eq!(unknown_changed.len(), 1);
    assert_eq!(
        &message(&unknown_changed, 1, MessageKind::Position)[7 + 34..7 + 36],
        &[2, 255]
    );
    assert!(
        registry
            .update_pose_with_state(SessionId(2), shared_zone, [4.0, 5.0, 6.0], 1.0, 2, 255)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn oversized_and_invalid_presentations_are_rejected_without_joining() {
    let mut oversized = PlayerPresentation::default();
    oversized.character_guid = 1001;
    oversized.model_customization = "x".repeat(64 * 1024);
    let mut malformed_attachment = PlayerPresentation::default();
    malformed_attachment.character_guid = 1001;
    malformed_attachment.attachments_wire = (-1_i32).to_le_bytes().to_vec();
    let mut invalid_attachment_text = PlayerPresentation::default();
    invalid_attachment_text.character_guid = 1001;
    invalid_attachment_text.attachments_wire = 1_i32.to_le_bytes().to_vec();
    invalid_attachment_text
        .attachments_wire
        .extend_from_slice(&1_i32.to_le_bytes());
    invalid_attachment_text.attachments_wire.push(0xff);
    for value in [0_i32; 5] {
        invalid_attachment_text
            .attachments_wire
            .extend_from_slice(&value.to_le_bytes());
    }
    let mut non_finite_speed = PlayerPresentation::default();
    non_finite_speed.character_guid = 1001;
    non_finite_speed.max_movement_speed = f32::NAN;
    let mut negative_speed = PlayerPresentation::default();
    negative_speed.character_guid = 1001;
    negative_speed.max_movement_speed = -1.0;
    let mut mismatched_character = PlayerPresentation::default();
    mismatched_character.character_guid = 1002;

    for presentation in [
        oversized,
        malformed_attachment,
        invalid_attachment_text,
        non_finite_speed,
        negative_speed,
        mismatched_character,
    ] {
        let mut registry = registry(20.0, 10);
        assert_eq!(
            registry.join_with_presentation(
                SessionId(1),
                101,
                1001,
                zone(8, 13),
                [0.0; 3],
                0.0,
                presentation,
            ),
            Err(RegistryError::InvalidPresentation)
        );
        assert!(registry.get(SessionId(1)).is_none());
        assert!(join(&mut registry, 1, 101, 1001, zone(8, 13), [0.0; 3], 0.0).is_empty());
    }
}

// T266 removed the server-side jump-relay window. The relay's only per-session
// state is now the directed view, so keep the protected-pass name and pin that
// the view is per session and dropped by `leave`: a rejoined session id must
// rebuild its own relay audience instead of inheriting one.
#[test]
fn jump_relay_window_is_per_session_and_clears_on_leave() {
    let mut registry = registry(50.0, 10);
    join(&mut registry, 1, 101, 1001, zone(8, 13), [0.0; 3], 0.0);
    join(
        &mut registry,
        2,
        102,
        1002,
        zone(8, 13),
        [1.0, 0.0, 0.0],
        0.0,
    );
    assert!(registry.knows_character(SessionId(1), 1002));
    assert!(registry.knows_character(SessionId(2), 1001));
    assert!(
        registry
            .nearby_sessions(zone(8, 13), [0.0; 3])
            .contains(&SessionId(2)),
        "the relay audience query sees the nearby directed peer"
    );
    registry.leave(SessionId(1));
    assert!(
        !registry.knows_character(SessionId(1), 1002),
        "leave drops the session's directed view"
    );
    assert!(
        !registry
            .nearby_sessions(zone(8, 13), [0.0; 3])
            .contains(&SessionId(1)),
        "leave drops the session from the relay audience"
    );
    join(&mut registry, 1, 101, 1001, zone(8, 13), [0.0; 3], 0.0);
    assert!(
        registry.knows_character(SessionId(1), 1002),
        "a rejoined session rebuilds its own view"
    );
    assert!(
        registry
            .nearby_sessions(zone(8, 13), [0.0; 3])
            .contains(&SessionId(1)),
        "a rejoined session is visible to the relay audience again"
    );
}
