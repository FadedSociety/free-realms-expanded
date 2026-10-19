use osfr_tools::{social_chat, social_lists};

use social_lists::{
    CharacterIdentity, CharacterLookup, FriendResultCode, GroupMemberRecord, GroupRecordName,
    SocialError, SocialLists, SocialName, SocialService, encode_friend_status,
    encode_group_member_update, encode_group_result, encode_group_update,
};
use std::collections::BTreeMap;

#[derive(Default)]
struct Directory(BTreeMap<(String, String), CharacterIdentity>);

impl Directory {
    fn add(&mut self, identity: CharacterIdentity) {
        self.0.insert(
            (
                identity.name.first.to_ascii_lowercase(),
                identity.name.last.to_ascii_lowercase(),
            ),
            identity,
        );
    }
}

impl CharacterLookup for Directory {
    fn find_by_name(&self, name: &SocialName) -> Option<CharacterIdentity> {
        self.0
            .get(&(
                name.first.to_ascii_lowercase(),
                name.last.to_ascii_lowercase(),
            ))
            .cloned()
    }
}

fn identity(account_id: u64, character_guid: u64, first: &str) -> CharacterIdentity {
    CharacterIdentity {
        account_id,
        character_guid,
        name: SocialName::new(first, "Fixture").unwrap(),
    }
}

fn service(size: usize) -> (SocialService, Directory) {
    let mut social = SocialService::new(size).unwrap();
    let mut directory = Directory::default();
    for (account, guid, first) in [
        (10, 101, "FixtureA"),
        (20, 202, "FixtureB"),
        (30, 303, "FixtureC"),
        (40, 404, "FixtureD"),
    ] {
        let person = identity(account, guid, first);
        directory.add(person.clone());
        social.register(SocialLists::new(person).unwrap()).unwrap();
    }
    (social, directory)
}

fn sub(message: &social_lists::SocialMessage) -> u16 {
    let family = u16::from_le_bytes([message.bytes[0], message.bytes[1]]);
    if family == 74 || family == 112 {
        message.bytes[2] as u16
    } else {
        u16::from_le_bytes([message.bytes[2], message.bytes[3]])
    }
}

fn family(message: &social_lists::SocialMessage) -> u16 {
    u16::from_le_bytes([message.bytes[0], message.bytes[1]])
}

fn result(message: &social_lists::SocialMessage) -> u32 {
    u32::from_le_bytes(message.bytes[4..8].try_into().unwrap())
}

#[test]
fn friend_request_accept_remove_and_all_client_result_codes() {
    let (mut social, directory) = service(5);
    social.login(101).unwrap();
    social.login(202).unwrap();
    let target_name = identity(20, 202, "FixtureB").name;

    let requested = social
        .request_friend(101, &target_name, &directory)
        .unwrap();
    assert!(requested.iter().any(|packet| {
        packet.recipient_guid == 101
            && family(packet) == 74
            && sub(packet) == 8
            && packet.bytes[3] == 5
    }));
    assert!(requested.iter().any(|packet| {
        packet.recipient_guid == 202 && family(packet) == 26 && sub(packet) == 16
    }));
    let invite = requested
        .iter()
        .find(|packet| packet.recipient_guid == 202 && family(packet) == 26 && sub(packet) == 16)
        .unwrap();
    assert_eq!(&invite.bytes[..4], &[26, 0, 16, 0]);
    assert_eq!(&invite.bytes[4..8], &[0; 4]);
    assert_eq!(&invite.bytes[8..16], &101u64.to_le_bytes());
    assert_eq!(
        i32::from_le_bytes(invite.bytes[28..32].try_into().unwrap()),
        8
    );
    assert_eq!(&invite.bytes[32..40], b"FixtureA");
    assert_eq!(
        i32::from_le_bytes(invite.bytes[40..44].try_into().unwrap()),
        7
    );
    assert_eq!(&invite.bytes[44..51], b"Fixture");
    assert_eq!(invite.bytes[51], 0);
    assert_eq!(invite.bytes.len(), 52);

    let accepted = social.respond_friend(202, 101, true).unwrap();
    assert!(social.profile(101).unwrap().friends.contains_key(&202));
    assert!(social.profile(202).unwrap().friends.contains_key(&101));
    assert!(
        accepted
            .iter()
            .any(|packet| family(packet) == 74 && sub(packet) == 6)
    );
    assert!(accepted.iter().any(|packet| {
        packet.recipient_guid == 101
            && family(packet) == 74
            && sub(packet) == 8
            && packet.bytes[3] == 1
    }));

    let removed = social.remove_friend(101, &target_name, &directory).unwrap();
    assert_eq!(removed.len(), 1);
    assert_eq!(family(&removed[0]), 74);
    assert_eq!(sub(&removed[0]), 7);
    assert!(!social.profile(101).unwrap().friends.contains_key(&202));
    // (R) Removal is local-only; the original server's reciprocal behavior is unknown.
    assert!(social.profile(202).unwrap().friends.contains_key(&101));

    let a = SocialName::new("FixtureA", "Fixture").unwrap();
    let b = SocialName::new("FixtureB", "Fixture").unwrap();
    let mut codes = Vec::new();
    for code in FriendResultCode::ALL {
        let bytes = encode_friend_status(code, 202, &a, &b).unwrap();
        assert_eq!(&bytes[..3], &[74, 0, 8]);
        codes.push(bytes[3]);
    }
    assert_eq!(codes, (1u8..=11).collect::<Vec<_>>());
}

#[test]
fn friend_decline_timeout_duplicate_and_login_logout_presence() {
    let (mut social, directory) = service(5);
    let name_b = identity(20, 202, "FixtureB").name;
    social.login(101).unwrap();
    social.request_friend(101, &name_b, &directory).unwrap();
    let duplicate = social.request_friend(101, &name_b, &directory).unwrap();
    assert_eq!(
        duplicate[0].bytes[3],
        FriendResultCode::NoFriendSpam.wire() as u8
    );
    social.respond_friend(202, 101, false).unwrap();
    let declined = social.request_friend(101, &name_b, &directory).unwrap();
    assert!(
        declined
            .iter()
            .any(|packet| packet.bytes.get(3) == Some(&5))
    );
    let timed_out = social.timeout_friend_request(101, 202).unwrap();
    assert_eq!(
        timed_out[0].bytes[3],
        FriendResultCode::TimedOut.wire() as u8
    );

    social.request_friend(101, &name_b, &directory).unwrap();
    social.respond_friend(202, 101, true).unwrap();
    let logged_in = social.login(202).unwrap();
    assert!(logged_in.iter().any(|packet| {
        packet.recipient_guid == 101 && family(packet) == 74 && sub(packet) == 2
    }));
    let logged_out = social.logout(202).unwrap();
    assert_eq!(logged_out.len(), 1);
    assert_eq!(logged_out[0].recipient_guid, 101);
    assert_eq!(family(&logged_out[0]), 74);
    assert_eq!(sub(&logged_out[0]), 3);
}

#[test]
fn ignore_add_remove_and_target_ignore_blocks_group_invite() {
    let (mut social, directory) = service(5);
    let name_a = identity(10, 101, "FixtureA").name;
    let ignored = social
        .add_ignore(101, &identity(20, 202, "FixtureB").name, &directory)
        .unwrap();
    assert_eq!((family(&ignored[0]), sub(&ignored[0])), (112, 2));
    let _: social_chat::IgnoreLists = social.chat_ignore_lists();

    let invite = social.invite_to_group(202, &name_a, &directory, 0).unwrap();
    assert_eq!(invite.len(), 1);
    assert_eq!((family(&invite[0]), sub(&invite[0])), (40, 2));
    assert_eq!(
        u32::from_le_bytes(invite[0].bytes[4..8].try_into().unwrap()),
        2
    );
    assert!(social.group_invitee(101).is_none());

    let removed = social
        .remove_ignore(101, &identity(20, 202, "FixtureB").name, &directory)
        .unwrap();
    assert_eq!((family(&removed[0]), sub(&removed[0])), (112, 3));
}

#[test]
fn group_invite_accept_capacity_kick_leader_leave_and_disband() {
    let (mut social, directory) = service(3);
    for guid in [101, 202, 303, 404] {
        social.login(guid).unwrap();
    }
    let b_name = identity(20, 202, "FixtureB").name;
    let c_name = identity(30, 303, "FixtureC").name;
    let d_name = identity(40, 404, "FixtureD").name;

    let invite_b = social.invite_to_group(101, &b_name, &directory, 0).unwrap();
    let group_id = social.group_of(101).unwrap();
    assert_eq!(
        invite_b
            .iter()
            .find(|packet| packet.recipient_guid == 202)
            .map(sub),
        Some(1)
    );
    assert!(invite_b.iter().any(|packet| {
        packet.recipient_guid == 101 && family(packet) == 40 && sub(packet) == 8
    }));
    let invite_result = invite_b
        .iter()
        .find(|packet| packet.recipient_guid == 101 && family(packet) == 40 && sub(packet) == 2)
        .unwrap();
    assert_eq!(
        u32::from_le_bytes(invite_result.bytes[4..8].try_into().unwrap()),
        0
    );
    let accepted_b = social.respond_group_invite(202, group_id, true, 0).unwrap();
    assert_eq!(social.group_members(group_id).unwrap(), &[101, 202]);
    assert!(
        accepted_b
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 4)
    );
    assert!(
        accepted_b
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 8)
    );
    assert!(
        accepted_b
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 9)
    );

    social.invite_to_group(101, &c_name, &directory, 0).unwrap();
    social.respond_group_invite(303, group_id, true, 0).unwrap();
    assert_eq!(social.group_members(group_id).unwrap(), &[101, 202, 303]);
    let full = social.invite_to_group(101, &d_name, &directory, 0).unwrap();
    let full_reply = full
        .iter()
        .find(|packet| packet.recipient_guid == 101)
        .unwrap();
    assert_eq!((family(full_reply), sub(full_reply)), (40, 2));
    assert_eq!(
        u32::from_le_bytes(full_reply.bytes[4..8].try_into().unwrap()),
        7
    );

    let leader_left = social.leave_group(101).unwrap();
    assert_eq!(social.group_leader(group_id), Some(202));
    assert_eq!(social.group_members(group_id).unwrap(), &[202, 303]);
    assert!(
        leader_left
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 3)
    );
    assert!(
        leader_left
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 8)
    );
    assert!(
        leader_left
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 9)
    );

    let kicked = social.kick_from_group(202, 303).unwrap();
    assert!(
        kicked
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 6)
    );
    assert_eq!(social.group_of(303), None);
    // (R) A group with one remaining member dissolves.
    assert_eq!(social.group_of(202), None);
}

#[test]
fn client_autorefuse_group_answer_declines_and_disconnect_cleans_state() {
    let (mut social, directory) = service(5);
    for guid in [101, 202, 303, 404] {
        social.login(guid).unwrap();
    }
    let b_name = identity(20, 202, "FixtureB").name;
    let c_name = identity(30, 303, "FixtureC").name;
    let d_name = identity(40, 404, "FixtureD").name;
    let invite = social.invite_to_group(101, &b_name, &directory, 0).unwrap();
    let group_id = social.group_of(101).unwrap();
    assert!(
        invite
            .iter()
            .any(|packet| packet.recipient_guid == 202 && family(packet) == 40 && sub(packet) == 1)
    );
    // The client checks AutoRefuse[GroupInvitation] at 00917b60 index 6, then sends 40/4 false.
    let declined = social
        .respond_group_invite(202, group_id, false, 0)
        .unwrap();
    assert!(
        declined
            .iter()
            .all(|packet| family(packet) == 40 && sub(packet) == 4)
    );
    assert!(social.group_invitee(202).is_none());

    social.invite_to_group(101, &b_name, &directory, 0).unwrap();
    social.respond_group_invite(202, group_id, true, 0).unwrap();
    social.invite_to_group(101, &c_name, &directory, 0).unwrap();
    social.respond_group_invite(303, group_id, true, 0).unwrap();
    social.invite_to_group(101, &d_name, &directory, 0).unwrap();
    let disconnected = social.disconnect(101).unwrap();
    assert_eq!(social.group_of(101), None);
    assert_eq!(social.group_leader(group_id), Some(202));
    assert_eq!(social.group_members(group_id).unwrap(), &[202, 303]);
    assert_eq!(social.group_invitee(404), None);
    assert!(
        disconnected
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 3)
    );
    assert!(
        disconnected
            .iter()
            .any(|packet| family(packet) == 40 && sub(packet) == 8)
    );
}

#[test]
fn per_character_social_document_roundtrips_and_rejects_another_owner() {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let folder = std::env::temp_dir().join(format!(
        "fr-social-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let path = folder.join("social.json");
    let mut profile = SocialLists::new(identity(10, 101, "FixtureA")).unwrap();
    profile
        .friends
        .insert(202, identity(20, 202, "FixtureB").name);
    profile
        .ignored
        .insert(303, identity(30, 303, "FixtureC").name);
    profile.save(&path).unwrap();

    let loaded = SocialLists::load(&path, 10, 101).unwrap();
    assert_eq!(loaded, profile);
    assert_eq!(
        SocialLists::load(&path, 10, 202).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn group_reader_builders_match_recovered_headers_and_member_order() {
    let row = GroupMemberRecord {
        guid: 202,
        name: GroupRecordName {
            fields: [1, 2, 3],
            first: "FixtureB".to_owned(),
            last: "Fixture".to_owned(),
        },
        opaque_fields: [11, 12, 13, 14, 15, 16, 17],
        flags: [true, false],
        minigame_type: 18,
    };
    let encoded_member = row.encode().unwrap();
    assert_eq!(&encoded_member[..8], &202u64.to_le_bytes());
    assert_eq!(
        &encoded_member[8..20],
        &[1i32, 2, 3]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        i32::from_le_bytes(encoded_member[20..24].try_into().unwrap()),
        8
    );
    assert_eq!(&encoded_member[24..32], b"FixtureB");
    assert_eq!(
        i32::from_le_bytes(encoded_member[32..36].try_into().unwrap()),
        7
    );
    assert_eq!(&encoded_member[36..43], b"Fixture");

    let member_update = encode_group_member_update(&row).unwrap();
    assert_eq!(&member_update[..4], &[40, 0, 9, 0]);
    assert_eq!(&member_update[8..8 + encoded_member.len()], encoded_member);
    assert_eq!(member_update.last(), Some(&0));

    let group_update = encode_group_update(&[row]).unwrap();
    assert_eq!(&group_update[..4], &[40, 0, 8, 0]);
    assert_eq!(
        i32::from_le_bytes(group_update[8..12].try_into().unwrap()),
        1
    );
    assert_eq!(encode_group_result(5, 2), [40, 0, 5, 0, 2, 0, 0, 0]);
}

#[test]
fn offline_group_invite_uses_not_found_result_without_creating_a_group() {
    let (mut social, directory) = service(3);
    let result = social
        .invite_to_group(101, &identity(20, 202, "FixtureB").name, &directory, 0)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!((family(&result[0]), sub(&result[0])), (40, 2));
    assert_eq!(
        u32::from_le_bytes(result[0].bytes[4..8].try_into().unwrap()),
        1
    );
    assert_eq!(social.group_of(101), None);
}

#[test]
fn refused_group_invites_leave_membership_and_pending_invites_unchanged() {
    let (mut social, directory) = service(3);
    for guid in [101, 202, 303] {
        social.login(guid).unwrap();
    }
    let a_name = identity(10, 101, "FixtureA").name;
    let b_name = identity(20, 202, "FixtureB").name;
    let c_name = identity(30, 303, "FixtureC").name;

    // (R) Self-invite is refused before a solo group can be created.
    let self_without_group = social.invite_to_group(303, &c_name, &directory, 0).unwrap();
    assert_eq!(
        (family(&self_without_group[0]), sub(&self_without_group[0])),
        (40, 2)
    );
    assert_eq!(result(&self_without_group[0]), 3);
    assert_eq!(social.group_of(303), None);
    assert_eq!(social.group_invitee(303), None);

    social.invite_to_group(101, &b_name, &directory, 0).unwrap();
    let group_id = social.group_of(101).unwrap();
    let membership = [101, 202, 303].map(|guid| social.group_of(guid));
    let pending = social.group_invitee(202);

    let repeated = social.invite_to_group(101, &b_name, &directory, 0).unwrap();
    assert_eq!((family(&repeated[0]), sub(&repeated[0])), (40, 2));
    assert_eq!(result(&repeated[0]), 6);
    assert_eq!(social.group_of(101), Some(group_id));
    assert_eq!(social.group_members(group_id), Some([101].as_slice()));
    assert_eq!(social.group_invitee(202), pending);

    // A failed invite from an ungrouped player must not leave that player in a phantom group.
    let cross_invite = social.invite_to_group(303, &b_name, &directory, 0).unwrap();
    assert_eq!((family(&cross_invite[0]), sub(&cross_invite[0])), (40, 2));
    assert_eq!(result(&cross_invite[0]), 4);
    assert_eq!(social.group_of(303), None);
    assert_eq!(
        [101, 202, 303].map(|guid| social.group_of(guid)),
        membership
    );
    assert_eq!(social.group_invitee(202), pending);

    let self_while_grouped = social.invite_to_group(101, &a_name, &directory, 0).unwrap();
    assert_eq!(
        (family(&self_while_grouped[0]), sub(&self_while_grouped[0])),
        (40, 2)
    );
    assert_eq!(result(&self_while_grouped[0]), 3);
    assert_eq!(social.group_members(group_id), Some([101].as_slice()));
    assert_eq!(social.group_invitee(202), pending);
}

#[test]
fn grouped_acceptance_is_refused_without_overlapping_party_membership() {
    let (mut social, directory) = service(3);
    for guid in [101, 202, 303] {
        social.login(guid).unwrap();
    }
    let b_name = identity(20, 202, "FixtureB").name;
    let c_name = identity(30, 303, "FixtureC").name;

    social.invite_to_group(101, &b_name, &directory, 0).unwrap();
    let a_group = social.group_of(101).unwrap();
    social.invite_to_group(202, &c_name, &directory, 0).unwrap();
    let b_group = social.group_of(202).unwrap();
    assert_ne!(a_group, b_group);
    social.respond_group_invite(303, b_group, true, 0).unwrap();

    // B accepted C into B's party after A had invited B. A second acceptance
    // must not move B out of that party or append B to A's roster.
    let refused = social.respond_group_invite(202, a_group, true, 0).unwrap();
    assert_eq!((family(&refused[0]), sub(&refused[0])), (40, 5));
    assert_eq!(result(&refused[0]), 1);
    assert_eq!(social.group_of(101), Some(a_group));
    assert_eq!(social.group_of(202), Some(b_group));
    assert_eq!(social.group_of(303), Some(b_group));
    assert_eq!(social.group_members(a_group), Some([101].as_slice()));
    assert_eq!(social.group_members(b_group), Some([202, 303].as_slice()));
    assert_eq!(social.group_invitee(202), Some((a_group, 101)));

    // Dissolving B's party must not remove B's mapping from A's unrelated party.
    social.leave_group(303).unwrap();
    assert_eq!(social.group_of(202), None);
    assert_eq!(social.group_of(303), None);
    assert_eq!(social.group_of(101), Some(a_group));
    assert_eq!(social.group_members(a_group), Some([101].as_slice()));
    assert_eq!(social.group_invitee(202), Some((a_group, 101)));

    // With the conflicting party gone, B can still decline the older invitation.
    let declined = social.respond_group_invite(202, a_group, false, 0).unwrap();
    assert!(
        declined
            .iter()
            .all(|packet| family(packet) == 40 && sub(packet) == 4)
    );
    assert_eq!(social.group_invitee(202), None);
    assert_eq!(social.group_members(a_group), Some([101].as_slice()));
}

#[test]
fn friend_validation_and_group_result_tables_reject_invalid_requests_cleanly() {
    let (mut social, directory) = service(5);
    let self_name = identity(10, 101, "FixtureA").name;
    assert_eq!(
        social.request_friend(101, &self_name, &directory),
        Err(SocialError::SelfFriend)
    );
    assert!(matches!(
        SocialService::new(1),
        Err(SocialError::InvalidGroupLimit)
    ));

    for (sub_op, codes) in [
        (2u16, vec![0, 1, 2, 3, 4, 5, 6, 7]),
        (5, vec![1, 2, 3]),
        (7, vec![1, 2, 3]),
    ] {
        for code in codes {
            let bytes = encode_group_result(sub_op, code);
            assert_eq!(u16::from_le_bytes([bytes[2], bytes[3]]), sub_op);
            assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), code);
        }
    }
}
