//! Builder tests for work item 092, "Mounts and vehicles".
//!
//! The module is not registered in `src/lib.rs` (a shared file owned by the lead), so this target
//! compiles the exact source with `#[path]`, as the other piece-NNN targets do. Lead integration
//! step: add the module and drop the include.
//!
//! These tests pin the recovered wire contract (mount base packets 168/3, 168/6, 168/8, 168/2,
//! 168/4 and the stat block 38/7), the captured ownership rows the supplied character carries, and
//! the reconstructed session policy. They never claim native behaviour.

#[path = "../src/restoration/piece_092.rs"]
mod piece_092;

use piece_092::{
    CAPTURED_MOUNTS, CLIENT_UPDATE_OPCODE, CLIENT_UPDATE_STAT_SUBOPCODE, DEFAULT_MOUNT_SPEED,
    DISMOUNT_SPEED, FIRE_HAWK_ITEM_DEFINITION_ID, FIRE_HAWK_MOUNT_NUMBER, FIRE_HAWK_NAME_ID,
    GLIDE_ACCEL_STAT, GLIDE_DEFAULT_FORWARD_SPEED_STAT, GLIDE_ENABLED_STAT, GLIDE_FALL_SPEED_STAT,
    GLIDE_FALL_TIME_STAT, GLIDE_MAX_FORWARD_SPEED_STAT, GLIDE_MIN_FORWARD_SPEED_STAT,
    JUMP_HEIGHT_STAT, MAX_MOVEMENT_SPEED_STAT, MOUNT_BASE_OPCODE, MOUNT_TELEPORT_EFFECT_ID,
    MountClientRequest, MountRejection, MountRequestError, MountSession, MountStat,
    MountTransition, PLAYER_UPDATE_ADD_NPC_SUBOPCODE, PLAYER_UPDATE_OPCODE,
    PLAYER_UPDATE_REMOVE_NPC_SUBOPCODE, StatValue, ZERO_SPEED_MOUNT_NUMBER, captured_mount,
    decode_client_request, dismount_stats, encode_dismount_response, encode_mount_response,
    encode_stat_update, item_bound_mount, mount_stats, sub_opcode, validate,
};

fn stat_id(stats: &[MountStat], id: i32) -> Option<&MountStat> {
    stats.iter().find(|stat| stat.id == id)
}

fn float_stat(stats: &[MountStat], id: i32) -> Option<f32> {
    match stat_id(stats, id)?.value {
        StatValue::Float(value) => Some(value),
        StatValue::Int(_) => None,
    }
}

fn int_stat(stats: &[MountStat], id: i32) -> Option<i32> {
    match stat_id(stats, id)?.value {
        StatValue::Int(value) => Some(value),
        StatValue::Float(_) => None,
    }
}

#[test]
fn recovered_packet_family_constants() {
    assert_eq!(MOUNT_BASE_OPCODE, 168);
    assert_eq!(CLIENT_UPDATE_OPCODE, 38);
    assert_eq!(CLIENT_UPDATE_STAT_SUBOPCODE, 7);
    assert_eq!(MOUNT_TELEPORT_EFFECT_ID, 46);
    assert_eq!(PLAYER_UPDATE_OPCODE, 35);
    assert_eq!(PLAYER_UPDATE_ADD_NPC_SUBOPCODE, 2);
    assert_eq!(PLAYER_UPDATE_REMOVE_NPC_SUBOPCODE, 3);
    assert_eq!(FIRE_HAWK_ITEM_DEFINITION_ID, 78397);

    assert_eq!(sub_opcode::MOUNT_REQUEST, 1);
    assert_eq!(sub_opcode::MOUNT_RESPONSE, 2);
    assert_eq!(sub_opcode::DISMOUNT_REQUEST, 3);
    assert_eq!(sub_opcode::DISMOUNT_RESPONSE, 4);
    assert_eq!(sub_opcode::MOUNT_LIST, 5);
    assert_eq!(sub_opcode::MOUNT_SPAWN, 6);
    assert_eq!(sub_opcode::MOUNT_SPAWN_BY_ITEM_DEFINITION_ID, 8);
    assert_eq!(sub_opcode::SET_AUTOMOUNT, 9);
    assert_eq!(sub_opcode::MOUNT_START_TRIAL, 10);
    assert_eq!(sub_opcode::MOUNT_END_TRIAL, 11);
    assert_eq!(sub_opcode::MOUNT_TRIAL_FAILED, 13);
    assert_eq!(sub_opcode::MOUNT_UPGRADE_REQUEST, 14);
    assert_eq!(sub_opcode::MOUNT_UPGRADE_RESPONSE, 15);
}

#[test]
fn decode_mount_spawn_request() {
    let request = decode_client_request(&[6, 0x0A, 0x04, 0x00, 0x00]).expect("mount spawn");
    assert_eq!(
        request,
        MountClientRequest::MountSpawn { mount_number: 1034 }
    );
}

#[test]
fn decode_dismount_request_has_no_payload() {
    assert_eq!(
        decode_client_request(&[3]).expect("dismount"),
        MountClientRequest::DismountRequest
    );
    assert_eq!(
        decode_client_request(&[3, 0]),
        Err(MountRequestError::TrailingBytes {
            expected: 0,
            got: 1
        })
    );
}

#[test]
fn decode_spawn_by_item_definition_request() {
    let request = decode_client_request(&[8, 0x25, 0x32, 0x01, 0x00, 0x01]).expect("item spawn");
    assert_eq!(
        request,
        MountClientRequest::MountSpawnByItemDefinitionId {
            item_definition_id: 78373,
            unknown: true,
        }
    );
    assert_eq!(
        decode_client_request(&[8, 0x25, 0x32, 0x01, 0x00]),
        Err(MountRequestError::Truncated)
    );
}

#[test]
fn decode_rejects_malformed_and_absent_requests() {
    assert_eq!(decode_client_request(&[]), Err(MountRequestError::Empty));
    assert_eq!(
        decode_client_request(&[6, 0x01]),
        Err(MountRequestError::Truncated)
    );
    assert_eq!(
        decode_client_request(&[6, 0, 0, 0, 0]),
        Err(MountRequestError::InvalidMountNumber(0))
    );
    assert_eq!(
        decode_client_request(&[99, 0, 0, 0, 0]),
        Err(MountRequestError::UnknownSubOpcode(99))
    );
}

#[test]
fn decode_reports_unrecovered_request_bodies() {
    let request = decode_client_request(&[9, 1, 2, 3]).expect("set automount");
    assert_eq!(
        request,
        MountClientRequest::Unrecovered {
            sub_opcode: sub_opcode::SET_AUTOMOUNT,
            payload: vec![1, 2, 3],
        }
    );
}

#[test]
fn captured_character_carries_the_recovered_ownership_rows() {
    assert_eq!(CAPTURED_MOUNTS.len(), 34);
    assert_eq!(validate(), Ok(()));

    let fire_hawk = captured_mount(FIRE_HAWK_MOUNT_NUMBER).expect("fire hawk row");
    assert_eq!(fire_hawk.number, 1030);
    assert_eq!(fire_hawk.name_id, FIRE_HAWK_NAME_ID);
    assert_eq!(fire_hawk.model_id, 4590);
    assert_eq!(fire_hawk.item_definition_id, 78397);
    assert!(fire_hawk.able_to_fly);
    assert_eq!(fire_hawk.tint_alias, "");

    let dragon = captured_mount(1002).expect("scaled dragon row");
    assert_eq!(dragon.name_id, 4897);
    assert_eq!(dragon.tint_alias, "scales-blue");
    assert!(dragon.able_to_fly);

    let member_only = captured_mount(1017).expect("member row");
    assert!(member_only.members_only);
    assert!(member_only.fly_training);

    assert_eq!(captured_mount(999), None);
    assert_eq!(
        item_bound_mount(FIRE_HAWK_ITEM_DEFINITION_ID).map(|mount| mount.number),
        Some(FIRE_HAWK_MOUNT_NUMBER)
    );
    assert_eq!(item_bound_mount(1), None);
}

#[test]
fn round_trip_owns_the_request_surface() {
    let session_rows = CAPTURED_MOUNTS.len();
    assert_eq!(session_rows, 34);

    let mut session = MountSession::new();
    assert_eq!(session.mounted_mount(), None);
    assert_eq!(
        session.apply(MountClientRequest::DismountRequest),
        MountTransition::IgnoredNotMounted
    );
    assert_eq!(
        session.apply(MountClientRequest::MountSpawn { mount_number: 1 }),
        MountTransition::Rejected(MountRejection::UnownedMount(1))
    );

    let mounted = session.apply(MountClientRequest::MountSpawn {
        mount_number: FIRE_HAWK_MOUNT_NUMBER,
    });
    assert_eq!(
        mounted,
        MountTransition::Mounted {
            number: FIRE_HAWK_MOUNT_NUMBER,
            replaced: None,
        }
    );
    assert_eq!(session.mounted_mount(), Some(FIRE_HAWK_MOUNT_NUMBER));

    let replaced = session.apply(MountClientRequest::MountSpawnByItemDefinitionId {
        item_definition_id: 11038,
        unknown: true,
    });
    assert_eq!(
        replaced,
        MountTransition::Mounted {
            number: 1002,
            replaced: Some(FIRE_HAWK_MOUNT_NUMBER),
        }
    );

    assert_eq!(
        session.apply(MountClientRequest::DismountRequest),
        MountTransition::Dismounted { number: 1002 }
    );
    assert_eq!(session.mounted_mount(), None);

    let unknown_item = session.apply(MountClientRequest::MountSpawnByItemDefinitionId {
        item_definition_id: 4,
        unknown: false,
    });
    assert_eq!(
        unknown_item,
        MountTransition::Rejected(MountRejection::UnknownItemDefinition(4))
    );
    assert_eq!(
        session.apply(MountClientRequest::Unrecovered {
            sub_opcode: sub_opcode::MOUNT_REQUEST,
            payload: vec![],
        }),
        MountTransition::Unsupported {
            sub_opcode: sub_opcode::MOUNT_REQUEST,
        }
    );
    assert_eq!(session.mounted_mount(), None);
}

#[test]
fn mount_speed_profiles_follow_recovered_values() {
    let fire_hawk = captured_mount(FIRE_HAWK_MOUNT_NUMBER).expect("fire hawk");
    let fire_hawk_stats = mount_stats(fire_hawk);
    assert_eq!(fire_hawk_stats.len(), 9);
    assert_eq!(
        float_stat(&fire_hawk_stats, MAX_MOVEMENT_SPEED_STAT),
        Some(12.5)
    );
    assert_eq!(
        float_stat(&fire_hawk_stats, GLIDE_DEFAULT_FORWARD_SPEED_STAT),
        Some(12.0)
    );
    assert_eq!(
        float_stat(&fire_hawk_stats, GLIDE_MIN_FORWARD_SPEED_STAT),
        Some(6.0)
    );
    assert_eq!(
        float_stat(&fire_hawk_stats, GLIDE_MAX_FORWARD_SPEED_STAT),
        Some(22.0)
    );
    assert_eq!(
        float_stat(&fire_hawk_stats, GLIDE_FALL_TIME_STAT),
        Some(0.75)
    );
    assert_eq!(
        float_stat(&fire_hawk_stats, GLIDE_FALL_SPEED_STAT),
        Some(3.0)
    );
    assert_eq!(int_stat(&fire_hawk_stats, GLIDE_ENABLED_STAT), Some(1));
    assert_eq!(float_stat(&fire_hawk_stats, GLIDE_ACCEL_STAT), Some(4.0));
    assert_eq!(float_stat(&fire_hawk_stats, JUMP_HEIGHT_STAT), Some(5.0));

    let dragon = captured_mount(1002).expect("scaled dragon");
    let dragon_stats = mount_stats(dragon);
    assert_eq!(dragon_stats.len(), 8);
    assert_eq!(
        float_stat(&dragon_stats, MAX_MOVEMENT_SPEED_STAT),
        Some(DEFAULT_MOUNT_SPEED)
    );
    assert_eq!(
        float_stat(&dragon_stats, GLIDE_DEFAULT_FORWARD_SPEED_STAT),
        Some(8.0)
    );
    assert_eq!(
        float_stat(&dragon_stats, GLIDE_MIN_FORWARD_SPEED_STAT),
        Some(2.0)
    );
    assert_eq!(
        float_stat(&dragon_stats, GLIDE_MAX_FORWARD_SPEED_STAT),
        Some(18.0)
    );
    assert_eq!(float_stat(&dragon_stats, GLIDE_FALL_TIME_STAT), Some(0.75));
    assert_eq!(float_stat(&dragon_stats, GLIDE_FALL_SPEED_STAT), Some(4.0));
    assert_eq!(int_stat(&dragon_stats, GLIDE_ENABLED_STAT), Some(1));
    assert_eq!(float_stat(&dragon_stats, GLIDE_ACCEL_STAT), Some(4.0));
    assert_eq!(stat_id(&dragon_stats, JUMP_HEIGHT_STAT), None);

    let ground = captured_mount(1000).expect("ground mount");
    let ground_stats = mount_stats(ground);
    assert_eq!(ground_stats.len(), 1);
    assert_eq!(
        float_stat(&ground_stats, MAX_MOVEMENT_SPEED_STAT),
        Some(DEFAULT_MOUNT_SPEED)
    );

    let zero_speed = captured_mount(ZERO_SPEED_MOUNT_NUMBER).expect("zero-speed branch row");
    let zero_speed_stats = mount_stats(zero_speed);
    assert_eq!(zero_speed_stats.len(), 1);
    assert_eq!(
        float_stat(&zero_speed_stats, MAX_MOVEMENT_SPEED_STAT),
        Some(0.0)
    );

    let dismount = dismount_stats();
    assert_eq!(dismount.len(), 3);
    assert_eq!(
        float_stat(&dismount, MAX_MOVEMENT_SPEED_STAT),
        Some(DISMOUNT_SPEED)
    );
    assert_eq!(int_stat(&dismount, GLIDE_ENABLED_STAT), Some(0));
    assert_eq!(float_stat(&dismount, JUMP_HEIGHT_STAT), Some(0.0));
}

#[test]
fn stat_block_bytes_match_the_recovered_field_order() {
    let stats = [MountStat {
        id: MAX_MOVEMENT_SPEED_STAT,
        value: StatValue::Float(12.5),
    }];
    let bytes = encode_stat_update(0x0000_0000_0000_0042, &stats);
    let mut expected = Vec::new();
    expected.extend_from_slice(&CLIENT_UPDATE_OPCODE.to_le_bytes());
    expected.extend_from_slice(&CLIENT_UPDATE_STAT_SUBOPCODE.to_le_bytes());
    expected.extend_from_slice(&0x42u64.to_le_bytes());
    expected.extend_from_slice(&1i32.to_le_bytes());
    expected.extend_from_slice(&MAX_MOVEMENT_SPEED_STAT.to_le_bytes());
    expected.extend_from_slice(&StatValue::Float(12.5).kind().to_le_bytes());
    expected.extend_from_slice(&12.5f32.to_le_bytes());
    assert_eq!(bytes, expected);

    let int_bytes = encode_stat_update(
        1,
        &[MountStat {
            id: GLIDE_ENABLED_STAT,
            value: StatValue::Int(0),
        }],
    );
    let mut expected_int = Vec::new();
    expected_int.extend_from_slice(&CLIENT_UPDATE_OPCODE.to_le_bytes());
    expected_int.extend_from_slice(&CLIENT_UPDATE_STAT_SUBOPCODE.to_le_bytes());
    expected_int.extend_from_slice(&1u64.to_le_bytes());
    expected_int.extend_from_slice(&1i32.to_le_bytes());
    expected_int.extend_from_slice(&GLIDE_ENABLED_STAT.to_le_bytes());
    expected_int.extend_from_slice(&0i32.to_le_bytes());
    expected_int.extend_from_slice(&0i32.to_le_bytes());
    assert_eq!(int_bytes, expected_int);
}

#[test]
fn server_responses_match_the_recovered_field_order() {
    let mut expected = Vec::new();
    expected.extend_from_slice(&MOUNT_BASE_OPCODE.to_le_bytes());
    expected.push(sub_opcode::MOUNT_RESPONSE);
    expected.extend_from_slice(&7u64.to_le_bytes());
    expected.extend_from_slice(&9u64.to_le_bytes());
    expected.extend_from_slice(&0i32.to_le_bytes());
    expected.extend_from_slice(&1i32.to_le_bytes());
    expected.extend_from_slice(&1i32.to_le_bytes());
    expected.extend_from_slice(&46i32.to_le_bytes());
    expected.extend_from_slice(&0.0f32.to_le_bytes());
    assert_eq!(encode_mount_response(7, 9, 0, 1, 1, 46, 0.0), expected);

    let mut expected_dismount = Vec::new();
    expected_dismount.extend_from_slice(&MOUNT_BASE_OPCODE.to_le_bytes());
    expected_dismount.push(sub_opcode::DISMOUNT_RESPONSE);
    expected_dismount.extend_from_slice(&7u64.to_le_bytes());
    expected_dismount.extend_from_slice(&46i32.to_le_bytes());
    assert_eq!(encode_dismount_response(7, 46), expected_dismount);
}
