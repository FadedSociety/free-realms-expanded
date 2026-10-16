//! House permission acceptance for `src/restoration/piece_023.rs` (MMO work item 023).
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/server/LibSOE/SOE/BasePackets.cs:78` - `BaseHousingPacket = 127`.
//! * `lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs` - the supplied server's housing
//!   inner-opcode enum, including `ClientHousingPacketSetEditMode = 6`,
//!   `ClientHousingPacketInvitePlayer = 15`, `ClientHousingPacketDeclineInvite = 16`,
//!   `ClientHousingPacketEnterRequest = 19`, `ClientHousingPacketRequestVisitToNeighbor = 21`,
//!   `ClientHousingPacketLockHouse = 22`, `ClientHousingPacketRequestGrant = 23`,
//!   `HousingPacketInstanceData = 28`, `HousingPacketUpdateHouseInfo = 44`,
//!   `HousingPacketHousingError = 46`.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - the same numbers appear in the client
//!   constructors (`FUN_00aa4990(6)` before `ClientHousingPacketSetEditMode::vftable` at 1384085,
//!   `FUN_00aa4990(0x17)` before `ClientHousingPacketRequestGrant::vftable` at 1384220,
//!   `FUN_00aa4990(0x2e)` before `HousingPacketHousingError::vftable` at 1384530). The client
//!   error handler `FUN_00ab9080` (1400087) reads exactly one 4-byte field (1400096) and switches
//!   on it (1400101): 1 server-not-found, 2/6 permission-denied, 3 zone-not-found,
//!   4 zone-failed-to-launch, 5 zone-full, with the denial message
//!   "You do not have permission to enter that house." (1397350, locale id 421055).
//! * `lab/references/Sanctuary/src/Sanctuary.Packet.Common/Housing/InstancePermission.cs` -
//!   `ulong Guid`, `int Level`, comment `// 3 - Can Edit`.
//! * `lab/references/Sanctuary/src/Sanctuary.Packet.Common/Housing/PlayerHousingInstanceData.cs` -
//!   `OwnerGuid`, `IsLocked`, `Dictionary<int, InstancePermission> Permissions`, written as
//!   count + (int key, permission) by `Sanctuary.Core/IO/PacketWriter.cs`.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers it in `src/lib.rs`. Integration step for the lead: register the module and drop the
//! include.
#[path = "../src/restoration/piece_023.rs"]
#[allow(dead_code)]
mod piece_023;

use piece_023::{
    AccessRole, BASE_HOUSING_PACKET, ERROR_PERMISSION_DENIED, ERROR_PERMISSION_DENIED_ALT,
    ERROR_SERVER_NOT_FOUND, ERROR_ZONE_FAILED_TO_LAUNCH, ERROR_ZONE_FULL, ERROR_ZONE_NOT_FOUND,
    HouseAccess, HousingDenial, InstancePermission, PERMISSION_CAN_EDIT, PermissionError,
    decode_housing_error, decode_permissions, encode_housing_error, encode_permissions,
    housing_error_message, opcodes,
};

const OWNER: u64 = 0x1122_3344_5566_7788;
const EDITOR: u64 = 0x0000_0000_0000_00aa;
const FRIEND: u64 = 0x0000_0000_0000_00bb;
const STRANGER: u64 = 0x0000_0000_0000_00cc;

fn house(locked: bool, max_visitors: Option<u32>) -> HouseAccess {
    HouseAccess {
        owner_guid: OWNER,
        locked,
        visitors: 0,
        max_visitors,
        permissions: vec![
            InstancePermission {
                key: 1,
                guid: EDITOR,
                level: PERMISSION_CAN_EDIT,
            },
            InstancePermission {
                key: 2,
                guid: FRIEND,
                level: 1,
            },
        ],
    }
}

/// The supplied server's enum is the source of the numbers this module publishes.
#[test]
fn opcode_table_matches_supplied_server_enum() {
    assert_eq!(BASE_HOUSING_PACKET, 127);
    assert_eq!(opcodes::CLIENT_SET_EDIT_MODE, 6);
    assert_eq!(opcodes::CLIENT_LEAVE_HOUSE, 10);
    assert_eq!(opcodes::CLIENT_TOGGLE_LOCKED, 11);
    assert_eq!(opcodes::CLIENT_REQUEST_PLAYER_HOUSES, 14);
    assert_eq!(opcodes::CLIENT_INVITE_PLAYER, 15);
    assert_eq!(opcodes::CLIENT_DECLINE_INVITE, 16);
    assert_eq!(opcodes::CLIENT_ENTER_REQUEST, 19);
    assert_eq!(opcodes::CLIENT_REQUEST_VISIT_TO_NEIGHBOR, 21);
    assert_eq!(opcodes::CLIENT_LOCK_HOUSE, 22);
    assert_eq!(opcodes::CLIENT_REQUEST_GRANT, 23);
    assert_eq!(opcodes::INSTANCE_DATA, 28);
    assert_eq!(opcodes::UPDATE_HOUSE_INFO, 44);
    assert_eq!(opcodes::ZONE_DATA, 45);
    assert_eq!(opcodes::HOUSING_ERROR, 46);
    assert_eq!(opcodes::INVITE_NOTIFICATION, 47);
    assert_eq!(opcodes::DECLINE_NOTIFICATION, 48);
    assert_eq!(opcodes::NOTIFY_HOUSE_ADDED, 49);
}

/// The recovered list layout is `count:i32` then `key:i32, guid:u64, level:i32` per entry.
#[test]
fn permission_list_round_trips_exact_wire_bytes() {
    let entries = vec![InstancePermission {
        key: 0,
        guid: OWNER,
        level: PERMISSION_CAN_EDIT,
    }];
    let bytes = encode_permissions(&entries).unwrap();
    assert_eq!(
        bytes,
        vec![
            1, 0, 0, 0, // count
            0, 0, 0, 0, // key
            0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // guid
            3, 0, 0, 0, // level
        ]
    );
    assert_eq!(decode_permissions(&bytes).unwrap(), entries);
}

#[test]
fn permission_list_preserves_multiple_entries_and_levels() {
    let entries = house(false, None).permissions;
    let decoded = decode_permissions(&encode_permissions(&entries).unwrap()).unwrap();
    assert_eq!(decoded, entries);
    assert!(decoded[0].can_edit());
    assert!(!decoded[1].can_edit());
}

/// A truncated list, a trailing byte and an impossible count are all rejected.
#[test]
fn permission_list_rejects_malformed_bodies() {
    assert_eq!(
        decode_permissions(&[1, 0, 0, 0]),
        Err(PermissionError::Truncated)
    );
    assert_eq!(
        decode_permissions(&[0, 0, 0, 0, 9]),
        Err(PermissionError::TrailingBytes)
    );
    assert_eq!(
        decode_permissions(&[0xff, 0xff, 0xff, 0x7f]),
        Err(PermissionError::TooManyEntries)
    );
}

#[test]
fn owner_may_enter_and_edit_own_house() {
    let mut h = house(true, Some(1));
    h.visitors = 1;
    assert_eq!(h.role_of(OWNER), AccessRole::Owner);
    assert_eq!(h.can_enter(OWNER), Ok(AccessRole::Owner));
    assert_eq!(h.can_edit(OWNER), Ok(()));
    assert!(h.client_can_edit(OWNER));
}

#[test]
fn editor_level_three_may_enter_and_edit() {
    let h = house(true, None);
    assert_eq!(h.role_of(EDITOR), AccessRole::Editor);
    assert_eq!(h.can_enter(EDITOR), Ok(AccessRole::Editor));
    assert_eq!(h.can_edit(EDITOR), Ok(()));
    assert!(h.client_can_edit(EDITOR));
}

#[test]
fn listed_guest_enters_but_unauthorized_edit_is_rejected() {
    let h = house(true, None);
    assert_eq!(h.role_of(FRIEND), AccessRole::Guest);
    assert_eq!(h.can_enter(FRIEND), Ok(AccessRole::Guest));
    assert_eq!(
        h.can_edit(FRIEND),
        Err(HousingDenial::PermissionDenied),
        "a listed level below 3 must not reach edit mode"
    );
    assert!(!h.client_can_edit(FRIEND));
}

#[test]
fn locked_house_denies_a_stranger_with_the_recovered_code() {
    let h = house(true, None);
    assert_eq!(h.role_of(STRANGER), AccessRole::Blocked);
    let denial = h.can_enter(STRANGER).unwrap_err();
    assert_eq!(denial, HousingDenial::PermissionDenied);
    assert_eq!(denial.code(), ERROR_PERMISSION_DENIED);
    assert_eq!(
        h.can_edit(STRANGER),
        Err(HousingDenial::PermissionDenied),
        "an unknown actor never reaches edit mode"
    );
}

#[test]
fn unlocked_house_admits_an_unlisted_visitor_as_guest() {
    let h = house(false, None);
    assert_eq!(h.can_enter(STRANGER), Ok(AccessRole::Guest));
    assert_eq!(h.can_edit(STRANGER), Err(HousingDenial::PermissionDenied));
    assert!(!h.client_can_edit(STRANGER));
}

#[test]
fn visitor_cap_denies_guests_but_never_the_owner() {
    let mut h = house(false, Some(2));
    h.visitors = 2;
    let denial = h.can_enter(STRANGER).unwrap_err();
    assert_eq!(denial, HousingDenial::ZoneFull);
    assert_eq!(denial.code(), ERROR_ZONE_FULL);
    assert_eq!(h.can_enter(EDITOR), Err(HousingDenial::ZoneFull));
    assert_eq!(h.can_enter(OWNER), Ok(AccessRole::Owner));
}

/// `HousingPacketHousingError` carries exactly one 4-byte code.
#[test]
fn housing_error_reply_is_one_little_endian_i32() {
    assert_eq!(
        encode_housing_error(ERROR_PERMISSION_DENIED),
        vec![2, 0, 0, 0]
    );
    assert_eq!(
        decode_housing_error(&[2, 0, 0, 0]),
        Ok(ERROR_PERMISSION_DENIED)
    );
    assert_eq!(
        decode_housing_error(&[2, 0, 0, 0, 0]),
        Err(PermissionError::TrailingBytes)
    );
    assert_eq!(
        decode_housing_error(&[2, 0]),
        Err(PermissionError::Truncated)
    );
}

#[test]
fn recovered_error_codes_keep_their_client_messages() {
    assert_eq!(
        housing_error_message(ERROR_SERVER_NOT_FOUND),
        Some("The housing server is not currently available.")
    );
    assert_eq!(
        housing_error_message(ERROR_PERMISSION_DENIED),
        Some("You do not have permission to enter that house.")
    );
    assert_eq!(
        housing_error_message(ERROR_PERMISSION_DENIED_ALT),
        Some("You do not have permission to enter that house.")
    );
    assert_eq!(
        housing_error_message(ERROR_ZONE_NOT_FOUND),
        Some("That housing zone could not be found.")
    );
    assert_eq!(
        housing_error_message(ERROR_ZONE_FAILED_TO_LAUNCH),
        Some("The housing zone failed to launch.")
    );
    assert_eq!(
        housing_error_message(ERROR_ZONE_FULL),
        Some("The house already has the maximum number of visitors.")
    );
    assert_eq!(housing_error_message(0), None);
    assert_eq!(housing_error_message(7), None);
}

/// A guest list is not an editing grant: a level below 3 must not flip the client's edit flag.
#[test]
fn client_edit_flag_follows_recovered_level_three() {
    let h = house(false, None);
    assert_eq!(h.client_permission_level(OWNER), Some(PERMISSION_CAN_EDIT));
    assert_eq!(h.client_permission_level(EDITOR), Some(PERMISSION_CAN_EDIT));
    assert_eq!(h.client_permission_level(FRIEND), Some(1));
    assert_eq!(h.client_permission_level(STRANGER), None);
}
