//! Work item 087 acceptance: friends and presence for the local restoration.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` sha256
//!   `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` - the shipped client
//!   constructors. Each friend subclass calls `FUN_00c7d870(<sub>)` past the shared
//!   `BaseFriendPacket` constructor (`FUN_00c7d870` stores the family/sub pair; the
//!   `BaseFriendsPacket` family is 74 in the supplied server's `BasePackets.cs`), and each command
//!   subclass calls `FUN_008a2820(<sub>, 0)`, which stores family `param_1[1] = 0x1a` (26).
//!   Lines 1747086-1749937 and 1035628/1264146/1378646/1667324/1672341/1672377 are the exact
//!   constructor bodies cited per opcode in `piece_087.rs`.
//! * `lab/decompiled/server/LibSOE/SOE/BaseFriendPackets.cs` sha256
//!   `6fd27d902cda661f4590a5743dbd084e5b112ab53b0d93cfae00a9b70afbd6c9` - the supplied server's
//!   own friend sub-opcode enum, recovered from the managed assembly. It agrees with all nine
//!   client constructors.
//! * `lab/decompiled/server/Gateway/Gateway.Login/LoginManager.cs` sha256
//!   `631fc8da1871bd73ed85ccce134554c72ea7726535a8839dd17dee2ecb06bc32` - the supplied server has
//!   exactly one friend-related handler (`CommandPacketFriendsPositionRequest`) and it answers with
//!   a hard-coded `PlayerUpdatePacketUpdatePosition` (125) for a fabricated character
//!   `5427660847725618161` at `(-1597.2408, -42.257397, -456.4496)`. That is canned emulator
//!   output, not a roster: no `FriendListPacket`/`FriendAddPacket` producer exists in the supplied
//!   server. The local piece therefore rebuilds the roster from the recovered wire table plus the
//!   community model, and never copies that canned packet.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` sha256
//!   `c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6` - the client's own
//!   `FriendsList_DS` data source (18 fields, line 3244) and `FriendsListDsColumn` (13 columns,
//!   line 3951). That is the recovered roster row shape shown to the player.
//!
//! INFERRED (community, pinned) evidence used for the byte layout: `lab/references/Sanctuary/`
//! `src/Sanctuary.Packet/BaseFriendPacket/*.cs` and `src/Sanctuary.Packet.Common/{FriendData,
//! FriendStatus,FriendMessageType,NameData}.cs`. Those files serialize the family/sub header as
//! `short` + `byte`, length-prefixed UTF-8 strings, and the field orders this module encodes. No
//! original server serializer was recovered, so the layout is labelled INFERRED in the module and
//! the tests below pin the bytes so a future original capture can falsify them quickly.
//!
//! RECONSTRUCTED: the request lifecycle rules (`add`, `confirm`, `remove`, ignore gating, online
//! fan-out) are a bounded local policy. The supplied server never implemented them.
//!
//! MISSING / explicit gaps: the original friend-roster retention, request expiry timing, the
//! status code meanings (the community comment claims 110 online / 111 offline), the `IgnoreRequest`
//! (30) body, teleport-to-friend rules and any per-zone fan-out limits. Nothing here is a native
//! acceptance claim.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_087;` under `src/restoration/`.
#[path = "../src/restoration/piece_087.rs"]
#[allow(dead_code)]
mod piece_087;

use piece_087::{
    ACQUAINTANCE_FAMILY, COMMAND_FAMILY, COMMAND_SUB_OPCODES, Delivery, FRIENDS_FAMILY,
    FriendCommand, FriendDenial, FriendError, FriendMessageType, FriendRegistry, FriendStatus,
    FriendSubOpcode, IGNORE_FAMILY, MAX_ROSTER_ENTRIES, NameData, Notification, ROSTER_FIELDS,
    ROSTER_UI_COLUMNS, UNUSED_SUB_OPCODES, decode_command, decode_friend_list, encode_friend_list,
    encode_notification,
};

fn name(first: &str, last: &str) -> NameData {
    NameData {
        first_name_id: 1001,
        last_name_prefix_id: 0,
        last_name_suffix_id: 0,
        first_name: first.to_string(),
        last_name: last.to_string(),
    }
}

fn status() -> FriendStatus {
    FriendStatus {
        status: 110,
        current_mini_game_type: 0,
        current_mini_game_name_id: 0,
        profile_id: 7,
        profile_name_id: 300,
        profile_icon_id: 41,
        profile_background_image_id: 52,
        profile_rank: 3,
    }
}

const NOVA: u64 = 0x1122_3344_5566_7788;
const REED: u64 = 0x8877_6655_4433_2211;
const STRANGER: u64 = 0x0102_0304_0506_0708;

/// The canonical online roster row used by the wire vectors below.
fn nova_row() -> piece_087::FriendEntry {
    piece_087::FriendEntry {
        name: name("Nova", "Reed"),
        guid: NOVA,
        online: true,
        in_encounter: false,
        status: status(),
        location_x: 10.5,
        location_z: -4.25,
        popularity: 12,
        is_local: true,
        is_in_static_zone: true,
        offline_unknown: 0,
    }
}

#[test]
fn family_and_sub_opcodes_match_the_client_and_the_supplied_server() {
    assert_eq!(FRIENDS_FAMILY, 74);
    assert_eq!(COMMAND_FAMILY, 26);
    assert_eq!(ACQUAINTANCE_FAMILY, 72);
    assert_eq!(IGNORE_FAMILY, 112);
    assert_eq!(UNUSED_SUB_OPCODES, [4, 8]);

    let recovered: Vec<(u8, &str)> = FriendSubOpcode::ALL
        .iter()
        .map(|sub| (sub.code(), sub.class_name()))
        .collect();
    assert_eq!(
        recovered,
        vec![
            (0, "FriendMessagePacket"),
            (1, "FriendListPacket"),
            (2, "FriendOnlinePacket"),
            (3, "FriendOfflinePacket"),
            (5, "FriendUpdatePositionsPacket"),
            (6, "FriendAddPacket"),
            (7, "FriendRemovePacket"),
            (9, "FriendStatusPacket"),
            (10, "FriendRenamePacket"),
        ]
    );

    // Slots 4 and 8 belong to no friend subclass in either the client or the supplied server.
    assert!(FriendSubOpcode::from_code(4).is_none());
    assert!(FriendSubOpcode::from_code(8).is_none());
    assert_eq!(
        FriendSubOpcode::from_code(0),
        Some(FriendSubOpcode::Message)
    );
    assert_eq!(
        FriendSubOpcode::from_code(10),
        Some(FriendSubOpcode::Rename)
    );
    assert_eq!(FriendSubOpcode::from_code(11), None);

    // Every published sub-opcode cites a client constructor line and the server enum name.
    for sub in FriendSubOpcode::ALL {
        assert!(!sub.provenance().is_empty(), "{sub:?} lost its provenance");
        assert!(sub.provenance().contains("pseudocode.c"));
    }
}

#[test]
fn command_sub_opcodes_match_the_client_constructors() {
    let recovered: Vec<(u16, &str)> = COMMAND_SUB_OPCODES
        .iter()
        .map(|command| (command.code(), command.class_name()))
        .collect();
    assert_eq!(
        recovered,
        vec![
            (14, "CommandPacketAddFriendRequest"),
            (15, "CommandPacketRemoveFriendRequest"),
            (16, "CommandPacketConfirmFriendRequest"),
            (17, "CommandPacketConfirmFriendResponse"),
            (21, "CommandPacketFriendsPositionRequest"),
            (30, "CommandPacketIgnoreRequest"),
        ]
    );
    assert_eq!(
        FriendCommand::from_code(14),
        Some(FriendCommand::AddFriendRequest)
    );
    assert_eq!(
        FriendCommand::from_code(16),
        Some(FriendCommand::ConfirmFriendRequest)
    );
    assert_eq!(
        FriendCommand::from_code(17),
        Some(FriendCommand::ConfirmFriendResponse)
    );
    assert_eq!(FriendCommand::from_code(31), None);
    assert!(
        FriendCommand::IgnoreRequest
            .provenance()
            .contains("pseudocode.c")
    );
}

#[test]
fn roster_field_sets_match_the_client_lua_datasource() {
    assert_eq!(ROSTER_FIELDS.len(), 18);
    assert_eq!(ROSTER_FIELDS[0], ("FriendName", 0));
    assert_eq!(ROSTER_FIELDS[1], ("LocationX", 1));
    assert_eq!(ROSTER_FIELDS[3], ("Guid", 3));
    assert_eq!(ROSTER_FIELDS[4], ("Online", 4));
    assert_eq!(ROSTER_FIELDS[9], ("Profile_Background_Image_ID", 9));
    assert_eq!(ROSTER_FIELDS[12], ("Is_Local", 12));
    assert_eq!(ROSTER_FIELDS[13], ("Status", 13));
    assert_eq!(ROSTER_FIELDS[17], ("Is_In_Static_Zone", 17));

    assert_eq!(ROSTER_UI_COLUMNS.len(), 13);
    assert_eq!(ROSTER_UI_COLUMNS[0], ("Name", 0));
    assert_eq!(ROSTER_UI_COLUMNS[4], ("Online", 4));
    assert_eq!(ROSTER_UI_COLUMNS[11], ("InEncounter", 11));
    assert_eq!(ROSTER_UI_COLUMNS[12], ("IsLocal", 12));
}

#[test]
fn friend_list_encodes_the_inferred_layout_byte_for_byte() {
    let bytes = encode_friend_list(&[nova_row()]);
    // count(4) + NameData(28) + guid(8) + online(1) + in_encounter(1) + status(32)
    // + x(4) + z(4) + popularity(4) + is_local(1) + is_in_static_zone(1) = 88
    assert_eq!(bytes.len(), 88);
    assert_eq!(&bytes[0..4], &1i32.to_le_bytes());
    assert_eq!(&bytes[4..8], &1001i32.to_le_bytes());
    assert_eq!(&bytes[8..12], &0i32.to_le_bytes());
    assert_eq!(&bytes[12..16], &0i32.to_le_bytes());
    assert_eq!(&bytes[16..20], &4i32.to_le_bytes());
    assert_eq!(&bytes[20..24], b"Nova");
    assert_eq!(&bytes[24..28], &4i32.to_le_bytes());
    assert_eq!(&bytes[28..32], b"Reed");
    assert_eq!(&bytes[32..40], &NOVA.to_le_bytes());
    assert_eq!(bytes[40], 1);
    assert_eq!(bytes[41], 0);
    assert_eq!(&bytes[42..46], &110i32.to_le_bytes());
    assert_eq!(&bytes[74..78], &10.5f32.to_le_bytes());
    assert_eq!(&bytes[78..82], &(-4.25f32).to_le_bytes());
    assert_eq!(&bytes[82..86], &12i32.to_le_bytes());
    assert_eq!(bytes[86], 1);
    assert_eq!(bytes[87], 1);

    let decoded = decode_friend_list(&bytes).expect("round trip");
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0], nova_row());
}

#[test]
fn friend_list_round_trips_offline_rows() {
    let mut row = nova_row();
    row.online = false;
    row.offline_unknown = 0xdead_beef_dead_beef;
    // The offline branch carries no is_local/status/position block, so the decoded row has the
    // neutral values the community serializer writes for an absent block.
    row.is_local = false;
    row.in_encounter = false;
    row.location_x = 0.0;
    row.location_z = 0.0;
    row.popularity = 0;
    row.status = FriendStatus::default();
    let bytes = encode_friend_list(&[row.clone()]);
    // offline skips the encounter/status/position/popularity block but keeps the unknown u64.
    let decoded = decode_friend_list(&bytes).expect("round trip");
    assert_eq!(decoded, vec![row]);
}

#[test]
fn friend_list_rejects_malformed_bodies() {
    let bytes = encode_friend_list(&[nova_row()]);

    let mut truncated = bytes.clone();
    truncated.truncate(bytes.len() - 1);
    assert!(matches!(
        decode_friend_list(&truncated),
        Err(FriendError::Truncated)
    ));

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(matches!(
        decode_friend_list(&trailing),
        Err(FriendError::TrailingBytes(1))
    ));

    let mut too_many = (MAX_ROSTER_ENTRIES as i32 + 1).to_le_bytes().to_vec();
    too_many.push(0);
    assert!(matches!(
        decode_friend_list(&too_many),
        Err(FriendError::RosterOutOfBounds(count)) if count as usize == MAX_ROSTER_ENTRIES + 1
    ));

    let mut non_finite = bytes.clone();
    non_finite[74..78].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(matches!(
        decode_friend_list(&non_finite),
        Err(FriendError::NonFiniteCoordinate)
    ));

    let mut overlong_name = 1i32.to_le_bytes().to_vec();
    overlong_name.extend_from_slice(&1i32.to_le_bytes());
    overlong_name.extend_from_slice(&0i32.to_le_bytes());
    overlong_name.extend_from_slice(&0i32.to_le_bytes());
    overlong_name.extend_from_slice(&(129i32).to_le_bytes());
    overlong_name.extend(vec![b'A'; 129]);
    assert!(matches!(
        decode_friend_list(&overlong_name),
        Err(FriendError::StringTooLong(129))
    ));
}

#[test]
fn client_requests_decode_from_the_recovered_opcodes_only() {
    let mut add = 9i32.to_le_bytes().to_vec();
    add.extend_from_slice(b"Nova Reed");
    assert_eq!(
        decode_command(FriendCommand::AddFriendRequest.code(), &add).expect("add request"),
        piece_087::FriendRequest::AddFriend {
            name: "Nova Reed".to_string()
        }
    );

    let remove_body = name("Nova", "Reed").encode();
    assert_eq!(
        decode_command(FriendCommand::RemoveFriendRequest.code(), &remove_body).expect("remove"),
        piece_087::FriendRequest::RemoveFriend {
            name: name("Nova", "Reed")
        }
    );

    let mut confirm = NOVA.to_le_bytes().to_vec();
    confirm.extend_from_slice(&0i32.to_le_bytes());
    confirm.extend_from_slice(&name("Nova", "Reed").encode());
    assert_eq!(
        decode_command(FriendCommand::ConfirmFriendResponse.code(), &confirm).expect("confirm"),
        piece_087::FriendRequest::ConfirmFriend {
            guid: NOVA,
            status: piece_087::ConfirmStatus::Accepted,
            name: name("Nova", "Reed"),
        }
    );

    assert_eq!(
        decode_command(FriendCommand::FriendsPositionRequest.code(), &[]).expect("positions"),
        piece_087::FriendRequest::FriendsPositionRequest
    );

    // Identity validation: a status outside the recovered accept/decline/timeout range is rejected.
    for bad in [3i32, -1, i32::MAX] {
        let mut body = NOVA.to_le_bytes().to_vec();
        body.extend_from_slice(&bad.to_le_bytes());
        body.extend_from_slice(&name("Nova", "Reed").encode());
        assert!(matches!(
            decode_command(FriendCommand::ConfirmFriendResponse.code(), &body),
            Err(FriendError::InvalidConfirmStatus(value)) if value == bad
        ));
    }

    assert!(matches!(
        decode_command(4, &[]),
        Err(FriendError::UnknownSubOpcode(4))
    ));
    assert!(matches!(
        decode_command(8, &[]),
        Err(FriendError::UnknownSubOpcode(8))
    ));

    // The recovered IgnoreRequest opcode 30 exists but its body was never recovered: refuse it
    // instead of guessing a shape.
    assert!(matches!(
        decode_command(FriendCommand::IgnoreRequest.code(), &[]),
        Err(FriendError::UnrecoveredBody("CommandPacketIgnoreRequest"))
    ));

    let mut trailing = add.clone();
    trailing.push(0);
    assert!(matches!(
        decode_command(FriendCommand::AddFriendRequest.code(), &trailing),
        Err(FriendError::TrailingBytes(1))
    ));
}

#[test]
fn notifications_encode_with_the_family_and_sub_header() {
    let remove = encode_notification(&Notification::Remove { guid: NOVA });
    assert_eq!(&remove[0..2], &74u16.to_le_bytes());
    assert_eq!(remove[2], 7);
    assert_eq!(&remove[3..11], &NOVA.to_le_bytes());
    assert_eq!(remove.len(), 11);

    let offline = encode_notification(&Notification::Offline { guid: NOVA });
    assert_eq!(offline[2], 3);
    assert_eq!(offline.len(), 11);

    let online = encode_notification(&Notification::Online {
        name: name("Nova", "Reed"),
        guid: NOVA,
        is_local: true,
    });
    assert_eq!(online[2], 2);
    // header(3) + NameData(28) + guid(8) + is_local(1)
    assert_eq!(online.len(), 3 + 28 + 8 + 1);

    let rename = encode_notification(&Notification::Rename {
        name: name("Nova", "Reed"),
        guid: NOVA,
    });
    assert_eq!(rename[2], 10);
    assert_eq!(rename.len(), 3 + 28 + 8);

    let status = encode_notification(&Notification::Status {
        name: name("Nova", "Reed"),
        guid: NOVA,
        status: status(),
    });
    assert_eq!(status[2], 9);
    assert_eq!(status.len(), 3 + 28 + 8 + 32);

    let add = encode_notification(&Notification::Add { entry: nova_row() });
    assert_eq!(add[2], 6);
    // header(3) + FriendData(84): the same row the list body carries without its count.
    assert_eq!(add.len(), 3 + 84);

    let message = encode_notification(&Notification::Message {
        message_type: FriendMessageType::AddRequested,
        guid: NOVA,
        name: name("Nova", "Reed"),
        new_name: name("Nova", "Reed"),
    });
    assert_eq!(message[2], 0);
    assert_eq!(message[3], 5);

    let positions = encode_notification(&Notification::UpdatePositions {
        entries: vec![piece_087::PositionEntry {
            guid: NOVA,
            name: name("Nova", "Reed"),
            in_encounter: false,
            location_x: 10.5,
            location_z: -4.25,
        }],
    });
    assert_eq!(positions[2], 5);
    assert_eq!(&positions[3..7], &1i32.to_le_bytes());
}

#[test]
fn add_requests_validate_identity_and_permissions() {
    let mut registry = FriendRegistry::new();
    registry.register(NOVA, name("Nova", "Reed"));
    registry.register(REED, name("Reed", "Stone"));
    registry.register(STRANGER, name("Stranger", "Guest"));

    assert_eq!(
        registry.add_request(NOVA, NOVA),
        Err(FriendDenial::SelfRequest)
    );
    assert_eq!(
        registry.add_request(NOVA, 0x9999),
        Err(FriendDenial::UnknownTarget)
    );

    // A target that ignores the requester is not offered the request at all.
    registry.ignore(REED, NOVA);
    assert_eq!(
        registry.add_request(NOVA, REED),
        Err(FriendDenial::TargetIgnoresSender)
    );
    registry.unignore(REED, NOVA);

    let deliveries = registry.add_request(NOVA, REED).expect("first request");
    // The requester learns the request was filed; the target gets the confirm request.
    assert_eq!(deliveries.len(), 2);
    assert_eq!(deliveries[0].to, NOVA);
    assert_eq!(deliveries[0].packet_code(), 0);
    assert_eq!(deliveries[1].to, REED);
    assert_eq!(deliveries[1].packet_code(), 16);

    assert_eq!(
        registry.add_request(NOVA, REED),
        Err(FriendDenial::AlreadyPending)
    );
    assert_eq!(
        registry.add_request(REED, NOVA),
        Err(FriendDenial::AlreadyPending)
    );
}

#[test]
fn accepting_a_request_makes_the_roster_mutual() {
    let mut registry = FriendRegistry::new();
    registry.register(NOVA, name("Nova", "Reed"));
    registry.register(REED, name("Reed", "Stone"));
    registry.add_request(NOVA, REED).expect("request");

    let deliveries = registry
        .confirm(REED, NOVA, piece_087::ConfirmStatus::Accepted)
        .expect("accept");
    assert_eq!(deliveries.len(), 3);
    assert_eq!(deliveries[0].to, NOVA);
    assert_eq!(deliveries[0].packet_code(), 0);
    assert_eq!(deliveries[1].to, REED);
    assert_eq!(deliveries[1].packet_code(), 6);
    assert_eq!(deliveries[2].to, NOVA);
    assert_eq!(deliveries[2].packet_code(), 6);

    assert!(registry.is_friend(NOVA, REED));
    assert!(registry.is_friend(REED, NOVA));
    // Both sides now see the friendship, so a repeat request is rejected instead of duplicated.
    assert_eq!(
        registry.add_request(NOVA, REED),
        Err(FriendDenial::AlreadyFriends)
    );
    // The pending request was consumed; a second confirm has nothing to answer.
    assert_eq!(
        registry.confirm(REED, NOVA, piece_087::ConfirmStatus::Accepted),
        Err(FriendDenial::NoPendingRequest)
    );
}

#[test]
fn declining_and_timing_out_leave_no_friendship() {
    for (confirm_status, message) in [
        (piece_087::ConfirmStatus::Declined, 2u8),
        (piece_087::ConfirmStatus::TimedOut, 3u8),
    ] {
        let mut registry = FriendRegistry::new();
        registry.register(NOVA, name("Nova", "Reed"));
        registry.register(REED, name("Reed", "Stone"));
        registry.add_request(NOVA, REED).expect("request");

        let deliveries = registry
            .confirm(REED, NOVA, confirm_status)
            .expect("answer");
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].to, NOVA);
        assert_eq!(deliveries[0].packet_code(), 0);
        assert_eq!(deliveries[0].message_type(), Some(message));
        assert!(!registry.is_friend(NOVA, REED));
        assert!(!registry.is_friend(REED, NOVA));
    }
}

#[test]
fn only_the_addressed_recipient_may_answer_a_request() {
    let mut registry = FriendRegistry::new();
    registry.register(NOVA, name("Nova", "Reed"));
    registry.register(REED, name("Reed", "Stone"));
    registry.register(STRANGER, name("Stranger", "Guest"));
    registry.add_request(NOVA, REED).expect("request");

    // A third character cannot accept a request addressed to someone else.
    assert_eq!(
        registry.confirm(STRANGER, NOVA, piece_087::ConfirmStatus::Accepted),
        Err(FriendDenial::NoPendingRequest)
    );
    // And the requester cannot answer their own request.
    assert_eq!(
        registry.confirm(NOVA, REED, piece_087::ConfirmStatus::Accepted),
        Err(FriendDenial::NoPendingRequest)
    );
}

#[test]
fn removal_requires_an_existing_friendship_and_notifies_both_sides() {
    let mut registry = FriendRegistry::new();
    registry.register(NOVA, name("Nova", "Reed"));
    registry.register(REED, name("Reed", "Stone"));

    assert_eq!(registry.remove(NOVA, REED), Err(FriendDenial::NotFriends));

    registry.add_request(NOVA, REED).expect("request");
    registry
        .confirm(REED, NOVA, piece_087::ConfirmStatus::Accepted)
        .expect("accept");

    let deliveries = registry.remove(NOVA, REED).expect("remove");
    assert_eq!(deliveries.len(), 2);
    assert_eq!(deliveries[0].to, NOVA);
    assert_eq!(deliveries[0].packet_code(), 7);
    assert_eq!(deliveries[1].to, REED);
    assert_eq!(deliveries[1].packet_code(), 7);
    assert!(!registry.is_friend(NOVA, REED));
    assert!(!registry.is_friend(REED, NOVA));
    assert_eq!(registry.remove(NOVA, REED), Err(FriendDenial::NotFriends));
}

#[test]
fn presence_changes_only_reach_friends() {
    let mut registry = FriendRegistry::new();
    registry.register(NOVA, name("Nova", "Reed"));
    registry.register(REED, name("Reed", "Stone"));
    registry.register(STRANGER, name("Stranger", "Guest"));
    registry.add_request(NOVA, REED).expect("request");
    registry
        .confirm(REED, NOVA, piece_087::ConfirmStatus::Accepted)
        .expect("accept");

    let online = registry.set_presence(NOVA, true).expect("online");
    let audience: Vec<u64> = online.iter().map(|delivery| delivery.to).collect();
    assert_eq!(audience, vec![REED]);
    assert_eq!(online[0].packet_code(), 2);

    let status = registry.publish_status(NOVA, status()).expect("status");
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].to, REED);
    assert_eq!(status[0].packet_code(), 9);

    let offline = registry.set_presence(NOVA, false).expect("offline");
    assert_eq!(offline.len(), 1);
    assert_eq!(offline[0].to, REED);
    assert_eq!(offline[0].packet_code(), 3);

    // A character with no friendship gets nothing, even for the same event.
    let empty = registry
        .set_presence(STRANGER, true)
        .expect("stranger online");
    assert!(empty.is_empty());
    assert!(!registry.is_friend(NOVA, STRANGER));
}

#[test]
fn position_requests_return_only_local_friends_and_reject_bad_input() {
    let mut registry = FriendRegistry::new();
    registry.register(NOVA, name("Nova", "Reed"));
    registry.register(REED, name("Reed", "Stone"));
    registry.register(STRANGER, name("Stranger", "Guest"));
    registry.add_request(NOVA, REED).expect("request");
    registry
        .confirm(REED, NOVA, piece_087::ConfirmStatus::Accepted)
        .expect("accept");

    assert_eq!(
        registry.update_position(REED, f32::NAN, 0.0, false),
        Err(FriendDenial::NonFinitePosition)
    );
    assert_eq!(
        registry.update_position(REED, 0.0, f32::INFINITY, false),
        Err(FriendDenial::NonFinitePosition)
    );

    registry
        .update_position(REED, 11.0, -2.0, false)
        .expect("position");
    registry.set_presence(REED, true).expect("online");

    let entries = registry.positions_for(NOVA).expect("friend positions");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].guid, REED);
    assert_eq!(entries[0].location_x, 11.0);
    assert_eq!(entries[0].location_z, -2.0);

    // A stranger's position request returns nothing for an unrelated character.
    assert!(
        registry
            .positions_for(STRANGER)
            .expect("stranger")
            .is_empty()
    );

    // A friend inside an encounter is reported without coordinates.
    registry
        .update_position(REED, 11.0, -2.0, true)
        .expect("encounter");
    let entries = registry.positions_for(NOVA).expect("friend positions");
    assert_eq!(entries.len(), 1);
    assert!(entries[0].in_encounter);
}

#[test]
fn rename_is_bounded_and_notifies_friends() {
    let mut registry = FriendRegistry::new();
    registry.register(NOVA, name("Nova", "Reed"));
    registry.register(REED, name("Reed", "Stone"));
    registry.add_request(NOVA, REED).expect("request");
    registry
        .confirm(REED, NOVA, piece_087::ConfirmStatus::Accepted)
        .expect("accept");

    assert_eq!(registry.rename(NOVA, ""), Err(FriendDenial::InvalidName));
    assert_eq!(
        registry.rename(NOVA, &"A".repeat(129)),
        Err(FriendDenial::InvalidName)
    );

    let deliveries = registry.rename(NOVA, "Nova Star").expect("rename");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].to, REED);
    assert_eq!(deliveries[0].packet_code(), 10);
    assert_eq!(registry.display_name(NOVA).as_deref(), Some("Nova Star"));
}

#[test]
fn deliveries_recover_the_decoded_notification() {
    let mut registry = FriendRegistry::new();
    registry.register(NOVA, name("Nova", "Reed"));
    registry.register(REED, name("Reed", "Stone"));
    registry.add_request(NOVA, REED).expect("request");
    registry
        .confirm(REED, NOVA, piece_087::ConfirmStatus::Accepted)
        .expect("accept");

    let deliveries: Vec<Delivery> = registry.remove(NOVA, REED).expect("remove");
    for delivery in deliveries {
        let notification = delivery.into_notification();
        assert!(matches!(notification, Some(Notification::Remove { .. })));
    }

    let list = registry.friend_list(NOVA).expect("roster");
    let encoded = encode_friend_list(&list);
    assert_eq!(decode_friend_list(&encoded).expect("round trip"), list);

    let list_notification = encode_notification(&Notification::List {
        friends: list.clone(),
    });
    assert_eq!(&list_notification[0..2], &74u16.to_le_bytes());
    assert_eq!(list_notification[2], 1);
    assert_eq!(&list_notification[3..], &encoded[..]);
}
