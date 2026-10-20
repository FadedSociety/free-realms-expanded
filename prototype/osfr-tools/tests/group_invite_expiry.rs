#[path = "common/group_invite_wire.rs"]
mod group_invite_wire;

use group_invite_wire::invitation_id;
use osfr_tools::social_lists::{
    CharacterIdentity, CharacterLookup, SocialLists, SocialName, SocialService,
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
    ] {
        let person = identity(account, guid, first);
        directory.add(person.clone());
        social.register(SocialLists::new(person).unwrap()).unwrap();
        social.login(guid).unwrap();
    }
    (social, directory)
}

fn group_result(messages: &[osfr_tools::social_lists::SocialMessage], sub: u16) -> Option<u32> {
    messages
        .iter()
        .find(|message| {
            u16::from_le_bytes([message.bytes[0], message.bytes[1]]) == 40
                && u16::from_le_bytes([message.bytes[2], message.bytes[3]]) == sub
        })
        .map(|message| u32::from_le_bytes(message.bytes[4..8].try_into().unwrap()))
}

#[test]
fn invite_remains_valid_at_fifty_nine_thousand_nine_hundred_ninety_nine_ms() {
    let (mut social, directory) = service(2);
    let target = identity(20, 202, "FixtureB").name;
    let invited = social
        .invite_to_group(101, &target, &directory, 1_000)
        .unwrap();
    let invite_id = invitation_id(&invited, 202);
    let group_id = social.group_of(101).unwrap();

    let accepted = social
        .respond_group_invite(202, invite_id, true, 60_999)
        .unwrap();
    assert_eq!(social.group_members(group_id).unwrap(), &[101, 202]);
    assert!(accepted.iter().any(|message| {
        u16::from_le_bytes([message.bytes[0], message.bytes[1]]) == 40
            && u16::from_le_bytes([message.bytes[2], message.bytes[3]]) == 4
    }));
}

#[test]
fn response_at_sixty_seconds_is_refused_without_membership_change() {
    let (mut social, directory) = service(2);
    let target = identity(20, 202, "FixtureB").name;
    let invited = social
        .invite_to_group(101, &target, &directory, 1_000)
        .unwrap();
    let invite_id = invitation_id(&invited, 202);
    let group_id = social.group_of(101).unwrap();
    let profiles_before = [social.profile(101).cloned(), social.profile(202).cloned()];

    let refused = social
        .respond_group_invite(202, invite_id, true, 61_000)
        .unwrap();

    assert_eq!(group_result(&refused, 5), Some(3));
    assert_eq!(social.group_invitee(202), None);
    assert_eq!(social.group_members(group_id).unwrap(), &[101]);
    assert_eq!(social.group_of(202), None);
    assert_eq!(
        [social.profile(101).cloned(), social.profile(202).cloned()],
        profiles_before
    );
}

#[test]
fn replacement_invite_releases_expired_capacity_without_a_cleanup_call() {
    let (mut social, directory) = service(2);
    let member = identity(20, 202, "FixtureB").name;
    let replacement = identity(30, 303, "FixtureC").name;
    let first = social
        .invite_to_group(101, &member, &directory, 1_000)
        .unwrap();
    let old_invite = invitation_id(&first, 202);
    let group_id = social.group_of(101).unwrap();

    let renewed = social
        .invite_to_group(101, &replacement, &directory, 61_000)
        .unwrap();
    assert_eq!(group_result(&renewed, 2), Some(0));
    let new_invite = invitation_id(&renewed, 303);
    assert_ne!(old_invite, new_invite);
    assert_eq!(social.group_invitee(303), Some((group_id, 101)));
    assert_eq!(social.group_invitee(202), None);

    let stale = social
        .respond_group_invite(202, old_invite, true, 61_001)
        .unwrap();
    assert_eq!(group_result(&stale, 5), Some(3));
    assert_eq!(social.group_invitee(303), Some((group_id, 101)));
    assert_eq!(social.group_members(group_id).unwrap(), &[101]);
    assert_eq!(social.group_of(202), None);
}

#[test]
fn response_expiry_does_not_change_group_membership_or_persistence() {
    let (mut social, directory) = service(3);
    let member = identity(20, 202, "FixtureB").name;
    let invitee = identity(30, 303, "FixtureC").name;
    let first = social
        .invite_to_group(101, &member, &directory, 1_000)
        .unwrap();
    let group_id = social.group_of(101).unwrap();
    social
        .respond_group_invite(202, invitation_id(&first, 202), true, 1_001)
        .unwrap();
    let second = social
        .invite_to_group(101, &invitee, &directory, 2_000)
        .unwrap();

    let members = social.group_members(group_id).unwrap().to_vec();
    let leader = social.group_leader(group_id);
    let profile_before = social.profile(101).unwrap().clone();
    let folder = std::env::temp_dir().join(format!(
        "fr-group-invite-expiry-{}-persistence",
        std::process::id()
    ));
    let save = folder.join("social.json");
    profile_before.save(&save).unwrap();
    let saved_before = std::fs::read(&save).unwrap();

    let refused = social
        .respond_group_invite(303, invitation_id(&second, 303), true, 62_000)
        .unwrap();

    assert_eq!(group_result(&refused, 5), Some(3));
    assert_eq!(social.group_invitee(303), None);
    assert_eq!(social.group_members(group_id).unwrap(), members);
    assert_eq!(social.group_leader(group_id), leader);
    assert_eq!(social.group_of(303), None);
    assert_eq!(social.profile(101), Some(&profile_before));
    assert_eq!(std::fs::read(&save).unwrap(), saved_before);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn invite_lifetime_is_configurable_and_refuses_out_of_range_values() {
    let (mut social, directory) = service(3);
    assert_eq!(social.group_invite_ttl_ms(), 60_000);
    assert!(social.set_group_invite_ttl(999).is_err());
    assert!(social.set_group_invite_ttl(600_001).is_err());
    assert_eq!(social.group_invite_ttl_ms(), 60_000);
    social.set_group_invite_ttl(600_000).unwrap();
    social.set_group_invite_ttl(1_000).unwrap();

    let target = identity(20, 202, "FixtureB").name;
    let invited = social
        .invite_to_group(101, &target, &directory, 5_000)
        .unwrap();
    let group_id = social.group_of(101).unwrap();
    social
        .respond_group_invite(202, invitation_id(&invited, 202), true, 5_999)
        .unwrap();
    assert_eq!(social.group_of(202), social.group_of(101));

    let invitee = identity(30, 303, "FixtureC").name;
    let renewed = social
        .invite_to_group(101, &invitee, &directory, 6_000)
        .unwrap();
    let refused = social
        .respond_group_invite(303, invitation_id(&renewed, 303), true, 7_000)
        .unwrap();
    assert_eq!(group_result(&refused, 5), Some(3));
    assert_eq!(social.group_members(group_id).unwrap(), &[101, 202]);
}

#[test]
fn same_group_reinvitation_gets_a_fresh_generation_and_refuses_the_old_one() {
    let (mut social, directory) = service(3);
    let target = identity(20, 202, "FixtureB").name;
    let first = social
        .invite_to_group(101, &target, &directory, 1_000)
        .unwrap();
    let group_id = social.group_of(101).unwrap();
    let stale_id = invitation_id(&first, 202);

    // The same leader re-invites the same target into the same live group: the group id
    // is reused, so only a fresh generation can tell the new pop-up from the old one.
    let second = social
        .invite_to_group(101, &target, &directory, 61_000)
        .unwrap();
    assert_eq!(group_result(&second, 2), Some(0));
    let fresh_id = invitation_id(&second, 202);
    assert_ne!(stale_id, fresh_id);
    // The echoed generation is not the group id, so a group-id guard cannot accept it.
    assert_ne!(fresh_id, group_id);
    let pending = social.group_invitee(202);
    assert_eq!(pending, Some((group_id, 101)));

    for (accepted, now_ms) in [(false, 61_001), (true, 61_002)] {
        let stale = social
            .respond_group_invite(202, stale_id, accepted, now_ms)
            .unwrap();
        assert_eq!(group_result(&stale, 5), Some(3));
        assert!(!stale.iter().any(|message| {
            u16::from_le_bytes([message.bytes[0], message.bytes[1]]) == 40
                && u16::from_le_bytes([message.bytes[2], message.bytes[3]]) == 4
        }));
        assert_eq!(social.group_invitee(202), pending);
        assert_eq!(social.group_members(group_id).unwrap(), &[101]);
        assert_eq!(social.group_of(202), None);
    }

    let accepted = social
        .respond_group_invite(202, fresh_id, true, 61_003)
        .unwrap();
    assert!(accepted.iter().any(|message| {
        u16::from_le_bytes([message.bytes[0], message.bytes[1]]) == 40
            && u16::from_le_bytes([message.bytes[2], message.bytes[3]]) == 4
    }));
    assert_eq!(social.group_members(group_id).unwrap(), &[101, 202]);
    assert_eq!(social.group_invitee(202), None);
}
