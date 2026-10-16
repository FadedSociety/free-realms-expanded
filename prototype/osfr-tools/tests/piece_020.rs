//! Multiplayer activity grouping acceptance for `src/restoration/piece_020.rs`.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` sha256
//!   `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` - the client
//!   `ClientActivityLaunchBasePacket` constructor `FUN_009c0bc0`/`FUN_009c0c30` sets the base
//!   opcode field to `0xaf` (175) and stores the sub-opcode in a 32-bit slot; each derived
//!   constructor is `FUN_009c0c30(<sub>, param_2, param_3)` immediately before its
//!   `ClientActivityLaunchPacket*::vftable` assignment. That is where every sub-opcode below is
//!   read from. `ClientActivityLaunchPacketUpdateMemberState` therefore takes sub-opcode 9.
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`
//!   sha256 `b8e568af2b5e69b64b4c2642b9a788ebe2bcb1d6444ff2811e9d182a436fdb75` - an implemented
//!   community reader: opcode 175 reads a 4-byte sub-opcode, 141 a 2-byte sub-opcode, 147/167 a
//!   1-byte sub-opcode, 39 a 1-byte sub-opcode and 40 a 2-byte sub-opcode.
//! * `lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs` sha256
//!   `f3f1287671de00cd009dd86ceaeec4765be581cd823d1da3306910361ccd0869` - the same numbering for the
//!   launch family, except that this older enum duplicates `8` for `RemoveMember` and
//!   `UpdateMemberState`; the binary and the newer reader both put `UpdateMemberState` at 9.
//! * Four original retail capture frames, reassembled under
//!   `lab/reports/original-minigame-packets/` and re-checked from these byte vectors:
//!   `demoderby-43464-flow1-frame41371-op141-sub1.bin` (19 B, sha256
//!   `b01cd62dd0c7ba457cb1dcca6c951add4ef391be116882aa3a00275501caf986`),
//!   `demoderby-43469-flow1-frame41374-op39-sub7.bin` (22 B, sha256
//!   `2f37481af7abca619b5754b35dc2ed02efea21c473f125e3a0098d862ea73309`),
//!   `demoderby-43472-flow1-frame41374-op167-sub1.bin` (11 B, sha256
//!   `8a47f6c040246dd8c588fd7464d8fafd649882f90291044a4dacd20dec4cb3e6`),
//!   `demoderby-23088-flow0-frame5078-op39-sub23.bin` (30 B, sha256
//!   `a6aa79938ed60a9255d0a51df9e20a02d1d2b065f97fecdbd0971f5487bdbbf9`).
//! * `lab/references/freerealms-wiki/wiki.sqlite` page "Groups" (revision 1304) - "Groups are two
//!   or more players who are able to battle together or fight in dungeons through an invite."
//!
//! GAP this test does not close: no captured 175 traffic and no community serializer exist, so the
//! per-field payload body of the launch messages stays opaque. Only the family header, the
//! sub-opcode table and the membership policy are asserted.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_020;` in `src/restoration`. Integration step for the lead: register the
//! module and drop the include.
#[path = "../src/restoration/piece_020.rs"]
#[allow(dead_code)]
mod piece_020;

use piece_020::{
    ACTIVITY_LAUNCH_FAMILY, ActivityGroupRegistry, Evidence, GroupError, LaunchMessage,
    MemberState, Reply, parse_family_header,
};

const OWNER: u64 = 0x0000_0040_0000_0001;
const MEMBER: u64 = 0x0000_0060_0000_0002;
const MATE: u64 = 0x0000_0060_0000_0003;
const STRANGER: u64 = 0x0000_0090_0000_dead;
const ACTIVITY: i32 = 12;
const OTHER_ACTIVITY: i32 = 21;
const FIRST_INSTANCE: i32 = 90_000_001;

/// The four original capture frames above, verbatim.
const CAPTURE_VECTORS: [(&str, &str); 4] = [
    (
        "demoderby 141/1 ListQueuesRequest",
        "0600010c000000 8d00 0100 91ca73956acc524b",
    ),
    (
        "demoderby 39/7",
        "0500010f000000 2700 07 41001400 ffffffffffffffff",
    ),
    (
        "demoderby 167/1/4 ActivityPacketActivityListRefreshRequest",
        "05000104000000 a700 01 04",
    ),
    (
        "demoderby 39/23 MiniGameKnockOutPacket",
        "05000117000000 2700 17 ffffffffffffffffffffffff 0000000000000000",
    ),
];

fn bytes(hex: &str) -> Vec<u8> {
    let compact: String = hex.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(compact.len() % 2, 0, "hex fixture must be whole bytes");
    (0..compact.len() / 2)
        .map(|i| u8::from_str_radix(&compact[i * 2..i * 2 + 2], 16).unwrap())
        .collect()
}

fn launched_registry() -> (ActivityGroupRegistry, i32) {
    let mut registry = ActivityGroupRegistry::new(FIRST_INSTANCE);
    registry.open(ACTIVITY, OWNER, 4).unwrap();
    registry.set_ready(ACTIVITY, OWNER, true).unwrap();
    registry.invite(ACTIVITY, OWNER, MEMBER).unwrap();
    registry.accept(ACTIVITY, MEMBER).unwrap();
    registry.set_ready(ACTIVITY, MEMBER, true).unwrap();
    let Reply::Launched { instance_id, .. } = registry.launch(ACTIVITY, OWNER).unwrap() else {
        panic!("owner launch must report the created instance");
    };
    (registry, instance_id)
}

#[test]
fn original_capture_frames_decode_with_the_recovered_family_widths() {
    let expected = [
        (6u16, 1u8, 141u16, 1i32, 8usize),
        (5, 1, 39, 7, 12),
        (5, 1, 167, 1, 1),
        (5, 1, 39, 23, 20),
    ];
    for ((label, hex), (outer, flag, opcode, sub, body_len)) in CAPTURE_VECTORS.iter().zip(expected)
    {
        let frame = bytes(hex);
        let parsed =
            parse_family_header(&frame).unwrap_or_else(|error| panic!("{label}: {error:?}"));
        assert_eq!(parsed.outer, outer, "{label}");
        assert_eq!(parsed.flag, flag, "{label}");
        assert_eq!(parsed.opcode, opcode, "{label}");
        assert_eq!(parsed.sub, sub, "{label}");
        assert_eq!(parsed.body.len(), body_len, "{label}");
        assert_eq!(parsed.inner_length as usize, frame.len() - 7, "{label}");
    }

    // Spot-check the bodies of the two message shapes this piece leans on.
    let list_queues = parse_family_header(&bytes(CAPTURE_VECTORS[0].1)).unwrap();
    assert_eq!(list_queues.body, 0x4b52_cc6a_9573_ca91u64.to_le_bytes());
    let knock_out = parse_family_header(&bytes(CAPTURE_VECTORS[3].1)).unwrap();
    assert_eq!(knock_out.body[..12], [0xff; 12]);
    assert_eq!(knock_out.body[12..], [0x00; 8]);
}

#[test]
fn malformed_frames_are_rejected_instead_of_guessed() {
    let frame = bytes(CAPTURE_VECTORS[1].1);
    assert_eq!(
        parse_family_header(&frame[..6]),
        Err(GroupError::MalformedEnvelope)
    );

    let mut wrong_outer = frame.clone();
    wrong_outer[0] = 7;
    assert_eq!(
        parse_family_header(&wrong_outer),
        Err(GroupError::MalformedEnvelope)
    );

    let mut wrong_length = frame.clone();
    wrong_length[3] = 0x0e;
    assert_eq!(
        parse_family_header(&wrong_length),
        Err(GroupError::MalformedEnvelope)
    );

    // 175 carries its sub-opcode as a 4-byte int; a 3-byte tail is truncated, not a sub-opcode.
    let mut truncated = bytes("05000105000000 af00 160000");
    assert_eq!(
        parse_family_header(&truncated),
        Err(GroupError::MalformedEnvelope)
    );
    truncated.pop();
    assert_eq!(
        parse_family_header(&truncated),
        Err(GroupError::MalformedEnvelope)
    );
}

#[test]
fn activity_launch_sub_opcodes_match_the_recovered_class_table() {
    let table = [
        (1, "ClientActivityLaunchPacketInviteDetails"),
        (2, "ClientActivityLaunchPacketInviteMemberReply"),
        (3, "ClientActivityLaunchPacketInviteMemberResult"),
        (4, "ClientActivityLaunchPacketPlayerHasBeenAccepted"),
        (5, "ClientActivityLaunchPacketActivityLaunched"),
        (6, "ClientActivityLaunchPacketActivityCanceled"),
        (7, "ClientActivityLaunchPacketAddMember"),
        (8, "ClientActivityLaunchPacketRemoveMember"),
        (9, "ClientActivityLaunchPacketUpdateMemberState"),
        (13, "ClientActivityLaunchPacketKickMemberEvent"),
        (14, "ClientActivityLaunchPacketOnMatchmakingStart"),
        (15, "ClientActivityLaunchPacketOnMatchmakingLaunch"),
        (16, "ClientActivityLaunchPacketOnMatchmakingCancel"),
        (17, "ClientActivityLaunchPacketInviteResponse"),
        (18, "ClientActivityLaunchPacketMemberReady"),
        (19, "ClientActivityLaunchPacketMemberQuit"),
        (20, "ClientActivityLaunchPacketInviteMemberRequest"),
        (21, "ClientActivityLaunchPacketKickMemberRequest"),
        (22, "ClientActivityLaunchPacketOwnerLaunchRequest"),
        (23, "ClientActivityLaunchPacketOwnerMatchmakingRequest"),
    ];

    assert_eq!(LaunchMessage::ALL.len(), table.len());
    for (sub, name) in table {
        let message = LaunchMessage::from_sub(sub)
            .unwrap_or_else(|| panic!("sub-opcode {sub} from the client binary must be known"));
        assert_eq!(message.sub_opcode(), sub);
        assert_eq!(message.class_name(), name);
        assert_eq!(message.evidence(), Evidence::Original);
    }

    // The older community enum duplicates 8; the client binary and the newer reader agree on 9.
    assert_eq!(
        LaunchMessage::from_sub(8),
        Some(LaunchMessage::RemoveMember)
    );
    assert_eq!(
        LaunchMessage::from_sub(9),
        Some(LaunchMessage::UpdateMemberState)
    );
    assert_eq!(LaunchMessage::from_sub(10), None);
    assert_eq!(LaunchMessage::from_sub(11), None);
    assert_eq!(LaunchMessage::from_sub(12), None);
    assert_eq!(LaunchMessage::from_sub(24), None);
    assert_eq!(ACTIVITY_LAUNCH_FAMILY, 175);
}

#[test]
fn activity_launch_frames_use_a_four_byte_sub_opcode_and_stay_opaque() {
    // 05 00 | 01 | 09 00 00 00 | af 00 | 16 00 00 00 | 01 02 03
    let frame = bytes("05000109000000 af00 16000000 010203");
    let parsed = parse_family_header(&frame).unwrap();
    assert_eq!(parsed.opcode, ACTIVITY_LAUNCH_FAMILY);
    assert_eq!(parsed.sub, 22);
    assert_eq!(parsed.body, vec![1, 2, 3]);
    assert_eq!(
        LaunchMessage::from_sub(parsed.sub),
        Some(LaunchMessage::OwnerLaunchRequest)
    );
}

#[test]
fn owner_invites_accepts_readies_and_launches_one_instance() {
    let mut registry = ActivityGroupRegistry::new(FIRST_INSTANCE);
    assert_eq!(
        registry.open(ACTIVITY, OWNER, 4).unwrap(),
        Reply::GroupOpened {
            activity_id: ACTIVITY,
            owner: OWNER,
        }
    );
    assert_eq!(
        registry.member_state(ACTIVITY, OWNER),
        Some(MemberState::Accepted)
    );

    assert_eq!(
        registry.invite(ACTIVITY, OWNER, MEMBER).unwrap(),
        Reply::InviteSent {
            activity_id: ACTIVITY,
            target: MEMBER,
        }
    );
    assert_eq!(
        registry.member_state(ACTIVITY, MEMBER),
        Some(MemberState::Invited)
    );

    assert_eq!(
        registry.accept(ACTIVITY, MEMBER).unwrap(),
        Reply::MemberAdded {
            activity_id: ACTIVITY,
            player: MEMBER,
        }
    );

    registry.set_ready(ACTIVITY, OWNER, true).unwrap();
    assert_eq!(
        registry.set_ready(ACTIVITY, MEMBER, true).unwrap(),
        Reply::MemberStateUpdated {
            activity_id: ACTIVITY,
            player: MEMBER,
            state: MemberState::Ready,
        }
    );

    let launched = registry.launch(ACTIVITY, OWNER).unwrap();
    let Reply::Launched {
        activity_id,
        instance_id,
        members,
    } = launched
    else {
        panic!("expected a launched instance");
    };
    assert_eq!(activity_id, ACTIVITY);
    assert_eq!(instance_id, FIRST_INSTANCE);
    assert_eq!(members, vec![OWNER, MEMBER]);
    assert_eq!(
        registry.instance_members(instance_id),
        Some(&[OWNER, MEMBER][..])
    );
    assert_eq!(registry.instance_of(OWNER), Some(instance_id));
    assert_eq!(registry.instance_of(MEMBER), Some(instance_id));
    assert_eq!(registry.instance_of(MATE), None);

    // The instance boundary is the membership check the game families need.
    assert_eq!(registry.authorize_instance(instance_id, MEMBER), Ok(()));
    assert_eq!(
        registry.authorize_instance(instance_id, STRANGER),
        Err(GroupError::NotInInstance)
    );
    assert_eq!(
        registry.authorize_instance(instance_id + 1, OWNER),
        Err(GroupError::UnknownInstance)
    );
}

#[test]
fn duplicate_and_non_member_requests_are_rejected() {
    let mut registry = ActivityGroupRegistry::new(FIRST_INSTANCE);
    registry.open(ACTIVITY, OWNER, 4).unwrap();
    registry.invite(ACTIVITY, OWNER, MEMBER).unwrap();

    assert_eq!(
        registry.invite(ACTIVITY, OWNER, MEMBER),
        Err(GroupError::AlreadyInvited)
    );
    assert_eq!(
        registry.invite(ACTIVITY, OWNER, OWNER),
        Err(GroupError::AlreadyMember)
    );
    assert_eq!(
        registry.invite(ACTIVITY, STRANGER, STRANGER),
        Err(GroupError::NotAMember)
    );
    assert_eq!(
        registry.accept(ACTIVITY, STRANGER),
        Err(GroupError::NotAMember)
    );
    assert_eq!(
        registry.set_ready(ACTIVITY, STRANGER, true),
        Err(GroupError::NotAMember)
    );
    assert_eq!(
        registry.quit(ACTIVITY, STRANGER),
        Err(GroupError::NotAMember)
    );
    assert_eq!(
        registry.kick(ACTIVITY, STRANGER, MEMBER),
        Err(GroupError::NotAMember)
    );

    registry.accept(ACTIVITY, MEMBER).unwrap();
    assert_eq!(
        registry.accept(ACTIVITY, MEMBER),
        Err(GroupError::AlreadyMember)
    );

    // Only the owner may remove another member, and only the owner may launch.
    assert_eq!(
        registry.kick(ACTIVITY, MEMBER, OWNER),
        Err(GroupError::NotOwner)
    );
    assert_eq!(registry.launch(ACTIVITY, MEMBER), Err(GroupError::NotOwner));
    assert_eq!(registry.cancel(ACTIVITY, MEMBER), Err(GroupError::NotOwner));

    // Operations never leak into a different activity id.
    assert_eq!(
        registry.invite(OTHER_ACTIVITY, OWNER, MATE),
        Err(GroupError::UnknownActivity)
    );
    assert_eq!(
        registry.quit(OTHER_ACTIVITY, MEMBER),
        Err(GroupError::UnknownActivity)
    );
}

#[test]
fn launch_requires_every_accepted_member_to_be_ready() {
    let mut registry = ActivityGroupRegistry::new(FIRST_INSTANCE);
    registry.open(ACTIVITY, OWNER, 4).unwrap();
    registry.invite(ACTIVITY, OWNER, MEMBER).unwrap();
    registry.accept(ACTIVITY, MEMBER).unwrap();
    registry.set_ready(ACTIVITY, OWNER, true).unwrap();

    assert_eq!(registry.launch(ACTIVITY, OWNER), Err(GroupError::NotReady));
    assert_eq!(
        registry.phase(ACTIVITY),
        Some(piece_020::GroupPhase::Forming)
    );
    assert_eq!(registry.instance_of(OWNER), None);

    registry.set_ready(ACTIVITY, MEMBER, true).unwrap();
    assert!(registry.launch(ACTIVITY, OWNER).is_ok());

    // Ready can be taken back while forming.
    assert!(matches!(
        registry
            .set_ready(ACTIVITY, MEMBER, false)
            .expect("a launched group still tracks member state"),
        Reply::MemberStateUpdated {
            state: MemberState::Accepted,
            ..
        }
    ));
}

#[test]
fn quit_kick_and_capacity_keep_the_member_list_exact() {
    let mut registry = ActivityGroupRegistry::new(FIRST_INSTANCE);
    registry.open(ACTIVITY, OWNER, 2).unwrap();
    registry.invite(ACTIVITY, OWNER, MEMBER).unwrap();
    registry.accept(ACTIVITY, MEMBER).unwrap();

    assert_eq!(
        registry.invite(ACTIVITY, OWNER, MATE),
        Err(GroupError::GroupFull)
    );

    assert_eq!(
        registry.quit(ACTIVITY, MEMBER).unwrap(),
        Reply::MemberRemoved {
            activity_id: ACTIVITY,
            player: MEMBER,
            kicked: false,
        }
    );
    assert_eq!(registry.member_state(ACTIVITY, MEMBER), None);
    assert_eq!(registry.quit(ACTIVITY, MEMBER), Err(GroupError::NotAMember));

    registry.invite(ACTIVITY, OWNER, MEMBER).unwrap();
    registry.accept(ACTIVITY, MEMBER).unwrap();
    assert_eq!(
        registry.kick(ACTIVITY, OWNER, MEMBER).unwrap(),
        Reply::MemberRemoved {
            activity_id: ACTIVITY,
            player: MEMBER,
            kicked: true,
        }
    );
    assert_eq!(
        registry.set_ready(ACTIVITY, MEMBER, true),
        Err(GroupError::NotAMember)
    );
    assert_eq!(registry.members(ACTIVITY).unwrap(), &[OWNER][..]);
}

#[test]
fn member_disconnect_leaves_the_group_and_the_instance() {
    let mut registry = ActivityGroupRegistry::new(FIRST_INSTANCE);
    registry.open(ACTIVITY, OWNER, 4).unwrap();
    registry.set_ready(ACTIVITY, OWNER, true).unwrap();
    registry.invite(ACTIVITY, OWNER, MEMBER).unwrap();
    registry.accept(ACTIVITY, MEMBER).unwrap();
    registry.set_ready(ACTIVITY, MEMBER, true).unwrap();
    let Reply::Launched { instance_id, .. } = registry.launch(ACTIVITY, OWNER).unwrap() else {
        panic!("expected a launched instance");
    };

    let replies = registry.disconnect(MEMBER);
    assert_eq!(
        replies,
        vec![Reply::MemberDisconnected {
            activity_id: ACTIVITY,
            player: MEMBER,
            instance_id: Some(instance_id),
        }]
    );
    assert_eq!(registry.instance_members(instance_id), Some(&[OWNER][..]));
    assert_eq!(registry.instance_of(MEMBER), None);
    assert_eq!(
        registry.authorize_instance(instance_id, MEMBER),
        Err(GroupError::NotInInstance)
    );

    // The last member leaving closes the instance, so stale packets cannot address it later.
    let replies = registry.disconnect(OWNER);
    assert_eq!(
        replies,
        vec![
            Reply::MemberDisconnected {
                activity_id: ACTIVITY,
                player: OWNER,
                instance_id: Some(instance_id),
            },
            Reply::InstanceClosed {
                activity_id: ACTIVITY,
                instance_id,
            },
        ]
    );
    assert_eq!(registry.instance_members(instance_id), None);
    assert_eq!(
        registry.authorize_instance(instance_id, OWNER),
        Err(GroupError::UnknownInstance)
    );
}

#[test]
fn owner_disconnect_cancels_a_group_that_has_not_launched() {
    let mut registry = ActivityGroupRegistry::new(FIRST_INSTANCE);
    registry.open(ACTIVITY, OWNER, 4).unwrap();
    registry.invite(ACTIVITY, OWNER, MEMBER).unwrap();
    registry.accept(ACTIVITY, MEMBER).unwrap();

    let replies = registry.disconnect(OWNER);
    assert_eq!(
        replies,
        vec![
            Reply::MemberDisconnected {
                activity_id: ACTIVITY,
                player: OWNER,
                instance_id: None,
            },
            Reply::Canceled {
                activity_id: ACTIVITY,
            },
        ]
    );
    assert_eq!(registry.member_state(ACTIVITY, OWNER), None);
    assert_eq!(registry.member_state(ACTIVITY, MEMBER), None);
    assert_eq!(
        registry.accept(ACTIVITY, MEMBER),
        Err(GroupError::UnknownActivity)
    );
    assert!(registry.disconnect(STRANGER).is_empty());
}

#[test]
fn membership_mutation_after_launch_is_rejected() {
    let (mut registry, instance_id) = launched_registry();

    assert_eq!(
        registry.invite(ACTIVITY, OWNER, MATE),
        Err(GroupError::AlreadyLaunched)
    );
    assert_eq!(
        registry.accept(ACTIVITY, MATE),
        Err(GroupError::AlreadyLaunched)
    );
    assert_eq!(
        registry.quit(ACTIVITY, MEMBER),
        Err(GroupError::AlreadyLaunched)
    );
    assert_eq!(
        registry.kick(ACTIVITY, OWNER, MEMBER),
        Err(GroupError::AlreadyLaunched)
    );
    assert_eq!(
        registry.launch(ACTIVITY, OWNER),
        Err(GroupError::AlreadyLaunched)
    );
    assert_eq!(registry.authorize_instance(instance_id, OWNER), Ok(()));
}

#[test]
fn group_state_snapshot_reports_members_and_instances() {
    let (registry, instance_id) = launched_registry();
    let snapshot = registry.state();
    assert_eq!(
        snapshot["groups"].as_array().map(|rows| rows.len()),
        Some(1)
    );
    assert_eq!(snapshot["groups"][0]["activity_id"], ACTIVITY);
    assert_eq!(snapshot["groups"][0]["owner"], OWNER);
    assert_eq!(
        snapshot["groups"][0]["members"]
            .as_array()
            .map(|rows| rows.len()),
        Some(2)
    );
    assert_eq!(snapshot["instances"][0]["id"], instance_id);
    assert_eq!(
        snapshot["instances"][0]["members"]
            .as_array()
            .map(|rows| rows.len()),
        Some(2)
    );
}
