//! W151 (W134 brief #12): friend positions (26/21 -> 74/5), group job-change indication
//! (40/14 -> 40/15) and group map ping (40/16). Packet shapes are checked against the recovered
//! readers cited in `social_lists.rs`; the scope rules are reconstructed policy (R).
//! Canary values (guids, coordinates) must never appear in a packet to an out-of-scope recipient.

use osfr_tools::social_lists::{
    CharacterIdentity, CharacterLookup, FriendPosition, GroupMemberRecord, GroupRecordName,
    LocationRequest, MAX_MAP_COORD, MAX_POSITION_ENTRIES, MapSpot, SocialError, SocialLists,
    SocialMessage, SocialName, SocialService, encode_friend_positions, encode_group_update,
    parse_location_request,
};
use std::collections::BTreeMap;

const A: u64 = 101;
const B: u64 = 202;
const C: u64 = 303;
const D: u64 = 404;
const ZONE: u32 = 7;

#[derive(Default)]
struct Directory(BTreeMap<(String, String), CharacterIdentity>);

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

fn who(guid: u64) -> &'static str {
    match guid {
        A => "FixtureA",
        B => "FixtureB",
        C => "FixtureC",
        _ => "FixtureD",
    }
}

fn name(guid: u64) -> SocialName {
    SocialName::new(who(guid), "Fixture").unwrap()
}

/// The 40/1 dword at +0x120 that the client copies from the pop-up and echoes in 40/4.
fn invitation_id(messages: &[SocialMessage], target: u64) -> u32 {
    let packet = messages
        .iter()
        .find(|message| {
            message.recipient_guid == target && message.bytes.starts_with(&[40, 0, 1, 0])
        })
        .expect("target received a 40/1 invite");
    let at = packet.bytes.len() - 6 * 4;
    u32::from_le_bytes(packet.bytes[at..at + 4].try_into().unwrap())
}

/// Four online characters; every pair of `friends` is a mutual friendship.
fn world(friends: &[(u64, u64)]) -> (SocialService, Directory) {
    let mut social = SocialService::new(5).unwrap();
    let mut directory = Directory::default();
    for (account, guid) in [(10, A), (20, B), (30, C), (40, D)] {
        let identity = CharacterIdentity {
            account_id: account,
            character_guid: guid,
            name: name(guid),
        };
        directory.0.insert(
            (who(guid).to_ascii_lowercase(), "fixture".into()),
            identity.clone(),
        );
        social
            .register(SocialLists::new(identity).unwrap())
            .unwrap();
        social.login(guid).unwrap();
    }
    for (a, b) in friends {
        social.request_friend(*a, &name(*b), &directory).unwrap();
        social.respond_friend(*b, *a, true).unwrap();
    }
    (social, directory)
}

fn group(social: &mut SocialService, directory: &Directory, leader: u64, members: &[u64]) {
    for member in members {
        let invited = social
            .invite_to_group(leader, &name(*member), directory, 0)
            .unwrap();
        social
            .respond_group_invite(*member, invitation_id(&invited, *member), true, 0)
            .unwrap();
    }
}

fn spot(zone_id: u32, instance_id: u64, x: f32, z: f32) -> MapSpot {
    MapSpot {
        zone_id,
        instance_id,
        in_battle: false,
        x,
        z,
    }
}

type Spots = BTreeMap<u64, MapSpot>;

fn at(spots: &Spots) -> impl Fn(u64) -> Option<MapSpot> + '_ {
    |guid| spots.get(&guid).copied()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn f(x: f32) -> [u8; 4] {
    x.to_le_bytes()
}

/// Decoded 74/5 body, following reader `00c81450`: `None` is a hidden entry (flag set).
fn decode_positions(bytes: &[u8]) -> Vec<(u64, Option<[f32; 2]>)> {
    assert_eq!(&bytes[..3], &[74, 0, 5]);
    let count = i32::from_le_bytes(bytes[3..7].try_into().unwrap()) as usize;
    let mut at = 7;
    let mut out = Vec::new();
    for _ in 0..count {
        let guid = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        assert_eq!(
            &bytes[at + 8..at + 12],
            &[0; 4],
            "discarded string is empty"
        );
        let flag = bytes[at + 12];
        at += 13;
        if flag == 0 {
            let x = f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
            let z = f32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
            at += 8;
            out.push((guid, Some([x, z])));
        } else {
            assert_eq!(flag, 1);
            out.push((guid, None));
        }
    }
    assert_eq!(at, bytes.len(), "no trailing bytes");
    out
}

fn positions(social: &SocialService, who: u64, spots: &Spots) -> Vec<(u64, Option<[f32; 2]>)> {
    social
        .friend_positions(who, at(spots))
        .unwrap()
        .map(|message| {
            assert_eq!(
                message.recipient_guid, who,
                "reply goes to the requester only"
            );
            decode_positions(&message.bytes)
        })
        .unwrap_or_default()
}

fn to(messages: &[SocialMessage], guid: u64) -> Vec<&SocialMessage> {
    messages
        .iter()
        .filter(|m| m.recipient_guid == guid)
        .collect()
}

// ---- wire shapes -------------------------------------------------------------------------

#[test]
fn parser_accepts_exact_client_shapes_and_nothing_else() {
    assert_eq!(
        parse_location_request(&[26, 0, 21, 0]),
        Some(LocationRequest::FriendPositions)
    );
    assert_eq!(
        parse_location_request(&[40, 0, 14, 0, 1]),
        Some(LocationRequest::ChangingJob(true))
    );
    assert_eq!(
        parse_location_request(&[40, 0, 14, 0, 0]),
        Some(LocationRequest::ChangingJob(false))
    );
    let ping = |guid: u64, x: i32, z: i32| {
        let mut bytes = vec![40, 0, 16, 0];
        bytes.extend(guid.to_le_bytes());
        bytes.extend(x.to_le_bytes());
        bytes.extend(z.to_le_bytes());
        bytes
    };
    assert_eq!(
        parse_location_request(&ping(0xdead_beef, -1200, 3400)),
        Some(LocationRequest::MapPing { x: -1200, z: 3400 })
    );
    // Truncations, trailing bytes, bad booleans and out-of-range or extreme values are refused.
    let full = ping(1, 5, 6);
    for cut in 0..full.len() {
        assert_eq!(parse_location_request(&full[..cut]), None, "cut {cut}");
    }
    let mut long = full.clone();
    long.push(0);
    assert_eq!(parse_location_request(&long), None);
    assert_eq!(parse_location_request(&[26, 0, 21, 0, 0]), None);
    assert_eq!(parse_location_request(&[40, 0, 14, 0]), None);
    assert_eq!(parse_location_request(&[40, 0, 14, 0, 2]), None);
    assert_eq!(parse_location_request(&[40, 0, 14, 0, 1, 0]), None);
    for bad in [MAX_MAP_COORD + 1, -MAX_MAP_COORD - 1, i32::MAX, i32::MIN] {
        assert_eq!(parse_location_request(&ping(1, bad, 0)), None, "{bad}");
        assert_eq!(parse_location_request(&ping(1, 0, bad)), None, "{bad}");
    }
    assert!(parse_location_request(&ping(1, MAX_MAP_COORD, -MAX_MAP_COORD)).is_some());
    // Neighbouring selectors belong to the existing router, not to this parser.
    for other in [
        [26, 0, 14, 0],
        [26, 0, 22, 0],
        [40, 0, 15, 0],
        [40, 0, 3, 0],
    ] {
        assert_eq!(parse_location_request(&other), None);
    }
}

#[test]
fn positions_packet_matches_recovered_reader_layout() {
    let bytes = encode_friend_positions(&[
        FriendPosition {
            guid: B,
            location: Some([12.5, -40.0]),
        },
        FriendPosition {
            guid: C,
            location: None,
        },
    ])
    .unwrap();
    let mut expected = vec![74, 0, 5];
    expected.extend(2i32.to_le_bytes());
    expected.extend(B.to_le_bytes());
    expected.extend(0i32.to_le_bytes());
    expected.push(0);
    expected.extend(f(12.5));
    expected.extend(f(-40.0));
    expected.extend(C.to_le_bytes());
    expected.extend(0i32.to_le_bytes());
    expected.push(1);
    assert_eq!(bytes, expected);

    for bad in [f32::NAN, f32::INFINITY, 1_000_001.0, -1_000_001.0] {
        assert_eq!(
            encode_friend_positions(&[FriendPosition {
                guid: B,
                location: Some([bad, 0.0]),
            }]),
            Err(SocialError::InvalidPosition)
        );
    }
    let too_many = vec![
        FriendPosition {
            guid: B,
            location: None
        };
        MAX_POSITION_ENTRIES + 1
    ];
    assert_eq!(
        encode_friend_positions(&too_many),
        Err(SocialError::InvalidPosition)
    );
}

// ---- friend positions --------------------------------------------------------------------

#[test]
fn mutual_online_friends_in_the_same_scope_get_real_coordinates() {
    let (social, _) = world(&[(A, B), (A, C)]);
    let spots = Spots::from([
        (A, spot(ZONE, 1, 0.0, 0.0)),
        (B, spot(ZONE, 1, 123.5, -45.25)),
        (C, spot(ZONE, 1, 9.0, 8.0)),
        (D, spot(ZONE, 1, 7777.0, 8888.0)),
    ]);
    assert_eq!(
        positions(&social, A, &spots),
        vec![(B, Some([123.5, -45.25])), (C, Some([9.0, 8.0]))]
    );
    // The friend sees the requester symmetrically, and only that one friend.
    assert_eq!(positions(&social, B, &spots), vec![(A, Some([0.0, 0.0]))]);
    // Canary: D is nobody's friend, so its coordinates reach no one.
    for who in [A, B, C] {
        let bytes = social
            .friend_positions(who, at(&spots))
            .unwrap()
            .unwrap()
            .bytes;
        assert!(!contains(&bytes, &f(7777.0)) && !contains(&bytes, &D.to_le_bytes()));
    }
    assert_eq!(positions(&social, D, &spots), vec![]);
}

#[test]
fn other_zone_instance_and_battle_friends_are_hidden_without_coordinates() {
    let (social, _) = world(&[(A, B), (A, C), (A, D)]);
    let spots = Spots::from([
        (A, spot(ZONE, 1, 0.0, 0.0)),
        (B, spot(ZONE + 1, 1, 6111.0, 6112.0)), // another zone
        (C, spot(ZONE, 2, 6113.0, 6114.0)),     // same zone, another instance
        (
            D,
            MapSpot {
                in_battle: true,
                ..spot(ZONE, 1, 6115.0, 6116.0)
            },
        ),
    ]);
    let message = social.friend_positions(A, at(&spots)).unwrap().unwrap();
    assert_eq!(
        decode_positions(&message.bytes),
        vec![(B, None), (C, None), (D, None)]
    );
    for canary in [6111.0, 6112.0, 6113.0, 6114.0, 6115.0, 6116.0] {
        assert!(!contains(&message.bytes, &f(canary)), "{canary} leaked");
    }
}

#[test]
fn offline_one_sided_ignored_unknown_and_unplaced_identities_are_absent() {
    let (mut social, directory) = world(&[(A, B), (A, C), (A, D)]);
    let spots = Spots::from([
        (A, spot(ZONE, 1, 0.0, 0.0)),
        (B, spot(ZONE, 1, 5001.0, 5002.0)),
        (C, spot(ZONE, 1, 5003.0, 5004.0)),
        (D, spot(ZONE, 1, 5005.0, 5006.0)),
    ]);
    assert_eq!(positions(&social, A, &spots).len(), 3);

    // B removes A: A still lists B, but B no longer lists A, so B's location is not shared.
    social.remove_friend(B, &name(A), &directory).unwrap();
    // C logs out.
    social.logout(C).unwrap();
    // D ignores A.
    social.add_ignore(D, &name(A), &directory).unwrap();
    let after = social.friend_positions(A, at(&spots)).unwrap();
    assert!(after.is_none(), "{after:?}");

    // A ignoring a mutual friend hides that friend too (either direction).
    let (mut social, directory) = world(&[(A, B)]);
    social.add_ignore(A, &name(B), &directory).unwrap();
    assert_eq!(positions(&social, A, &spots), vec![]);

    // A friend the registry cannot place (unknown guid in the closure) yields nothing.
    let (social, _) = world(&[(A, B)]);
    let only_a = Spots::from([(A, spot(ZONE, 1, 0.0, 0.0))]);
    assert_eq!(positions(&social, A, &only_a), vec![]);
    // A requester the registry cannot place gets no reply at all.
    let only_b = Spots::from([(B, spot(ZONE, 1, 4242.0, 4343.0))]);
    assert!(social.friend_positions(A, at(&only_b)).unwrap().is_none());
    // A requester with no social profile is an error, not a reply.
    assert_eq!(
        social.friend_positions(9999, at(&only_b)),
        Err(SocialError::MissingProfile)
    );
}

#[test]
fn offline_requester_and_non_finite_registry_values_are_never_encoded() {
    let (mut social, _) = world(&[(A, B)]);
    let spots = Spots::from([
        (A, spot(ZONE, 1, 0.0, 0.0)),
        (B, spot(ZONE, 1, f32::NAN, 5.0)),
    ]);
    // A corrupt registry position hides the friend instead of reaching the client as NaN.
    let message = social.friend_positions(A, at(&spots)).unwrap().unwrap();
    assert_eq!(decode_positions(&message.bytes), vec![(B, None)]);
    let far = Spots::from([
        (A, spot(ZONE, 1, 0.0, 0.0)),
        (B, spot(ZONE, 1, 2_000_000.0, 0.0)),
    ]);
    let message = social.friend_positions(A, at(&far)).unwrap().unwrap();
    assert_eq!(decode_positions(&message.bytes), vec![(B, None)]);
    social.logout(A).unwrap();
    assert!(social.friend_positions(A, at(&far)).unwrap().is_none());
}

#[test]
fn online_friend_rows_open_the_atlas_gate_and_offline_rows_do_not() {
    // Row +0xa1 (`Is_In_Static_Zone`) is the final byte of the only row in each 74/1 list.
    let list = |messages: Vec<SocialMessage>| {
        messages
            .into_iter()
            .find(|m| m.bytes.starts_with(&[74, 0, 1]))
            .unwrap()
            .bytes
    };
    let (mut social, _) = world(&[(A, B)]);
    social.logout(A).unwrap();
    assert_eq!(*list(social.login(A).unwrap()).last().unwrap(), 1);
    social.logout(A).unwrap();
    social.logout(B).unwrap();
    assert_eq!(*list(social.login(A).unwrap()).last().unwrap(), 0);
}

// ---- group job change --------------------------------------------------------------------

#[test]
fn job_change_reaches_exactly_the_group_with_the_authenticated_guid() {
    let (mut social, directory) = world(&[]);
    group(&mut social, &directory, A, &[B, C]);
    let sent = social.group_changing_job(B, true).unwrap();
    let mut expected = vec![40, 0, 15, 0];
    expected.extend(B.to_le_bytes());
    expected.push(1);
    let mut recipients: Vec<u64> = sent.iter().map(|m| m.recipient_guid).collect();
    recipients.sort();
    assert_eq!(recipients, vec![A, B, C]);
    assert!(sent.iter().all(|m| m.bytes == expected));

    // The non-member D can neither set the flag nor receive anything.
    assert_eq!(
        social.group_changing_job(D, true),
        Err(SocialError::NotInGroup)
    );
    assert!(to(&sent, D).is_empty());
    // Repeating the state sends nothing; clearing sends the cleared flag.
    assert!(social.group_changing_job(B, true).unwrap().is_empty());
    let cleared = social.group_changing_job(B, false).unwrap();
    assert_eq!(cleared.len(), 3);
    assert_eq!(*cleared[0].bytes.last().unwrap(), 0);
    assert!(social.group_changing_job(B, false).unwrap().is_empty());
}

fn member_record(guid: u64, changing: bool) -> GroupMemberRecord {
    GroupMemberRecord {
        guid,
        name: GroupRecordName {
            fields: [0; 3],
            first: who(guid).into(),
            last: "Fixture".into(),
        },
        opaque_fields: [0, 0, 0, u32::MAX, u32::MAX, 1, 0],
        flags: [changing, false],
        minigame_type: 0,
    }
}

#[test]
fn job_change_flag_survives_snapshots_and_dies_with_membership() {
    let (mut social, directory) = world(&[]);
    group(&mut social, &directory, A, &[B]);
    social.group_changing_job(B, true).unwrap();
    // C joins: the 40/8 snapshot still marks B as changing job.
    let invited_c = social.invite_to_group(A, &name(C), &directory, 0).unwrap();
    let joined = social
        .respond_group_invite(C, invitation_id(&invited_c, C), true, 0)
        .unwrap();
    let snapshot = joined
        .iter()
        .find(|m| m.recipient_guid == C && m.bytes.starts_with(&[40, 0, 8, 0]))
        .unwrap();
    let expected = encode_group_update(&[
        member_record(A, false),
        member_record(B, true),
        member_record(C, false),
    ])
    .unwrap();
    assert_eq!(snapshot.bytes, expected);

    // B leaves while flagged, then rejoins: the flag was cleared.
    social.leave_group(B).unwrap();
    let invited_b = social.invite_to_group(A, &name(B), &directory, 0).unwrap();
    let rejoined = social
        .respond_group_invite(B, invitation_id(&invited_b, B), true, 0)
        .unwrap();
    let snapshot = rejoined
        .iter()
        .find(|m| m.recipient_guid == B && m.bytes.starts_with(&[40, 0, 8, 0]))
        .unwrap();
    assert_eq!(
        snapshot.bytes,
        encode_group_update(&[
            member_record(A, false),
            member_record(C, false),
            member_record(B, false)
        ])
        .unwrap()
    );
    // Disconnect clears it too, and a dissolved group leaves no flag behind.
    social.group_changing_job(C, true).unwrap();
    social.disconnect(C).unwrap();
    social.login(C).unwrap();
    social.leave_group(B).unwrap();
    assert_eq!(social.group_of(A), None);
    let third = social.invite_to_group(A, &name(C), &directory, 0).unwrap();
    let again = social
        .respond_group_invite(C, invitation_id(&third, C), true, 0)
        .unwrap();
    let snapshot = again
        .iter()
        .find(|m| m.recipient_guid == C && m.bytes.starts_with(&[40, 0, 8, 0]))
        .unwrap();
    assert_eq!(
        snapshot.bytes,
        encode_group_update(&[member_record(A, false), member_record(C, false)]).unwrap()
    );
}

// ---- group map ping ----------------------------------------------------------------------

fn ping_bytes(sender: u64, x: i32, z: i32) -> Vec<u8> {
    let mut bytes = vec![40, 0, 16, 0];
    bytes.extend(sender.to_le_bytes());
    bytes.extend(x.to_le_bytes());
    bytes.extend(z.to_le_bytes());
    bytes
}

#[test]
fn ping_reaches_group_members_in_the_same_zone_and_instance_only() {
    let (mut social, directory) = world(&[]);
    group(&mut social, &directory, A, &[B, C]);
    let spots = Spots::from([
        (A, spot(ZONE, 1, 0.0, 0.0)),
        (B, spot(ZONE, 1, 50.0, 50.0)),
        (C, spot(ZONE, 2, 60.0, 60.0)), // same zone, other instance
        (D, spot(ZONE, 1, 70.0, 70.0)), // same scope but not in the group
    ]);
    let sent = social.group_map_ping(A, -250, 900, at(&spots)).unwrap();
    let mut recipients: Vec<u64> = sent.iter().map(|m| m.recipient_guid).collect();
    recipients.sort();
    assert_eq!(recipients, vec![A, B], "sender and in-scope member only");
    assert!(sent.iter().all(|m| m.bytes == ping_bytes(A, -250, 900)));

    // Another zone is out of scope as well.
    let mut elsewhere = spots.clone();
    elsewhere.insert(B, spot(ZONE + 1, 1, 50.0, 50.0));
    let sent = social.group_map_ping(A, 1, 2, at(&elsewhere)).unwrap();
    assert_eq!(
        sent.iter().map(|m| m.recipient_guid).collect::<Vec<_>>(),
        vec![A]
    );
}

#[test]
fn ping_refuses_nonmembers_bad_values_unplaced_and_ignoring_recipients() {
    let (mut social, directory) = world(&[]);
    group(&mut social, &directory, A, &[B, C]);
    let spots = Spots::from([
        (A, spot(ZONE, 1, 0.0, 0.0)),
        (B, spot(ZONE, 1, 1.0, 1.0)),
        (C, spot(ZONE, 1, 2.0, 2.0)),
        (D, spot(ZONE, 1, 3.0, 3.0)),
    ]);
    // D is in the same scope but not in the group: refused, nothing sent.
    assert_eq!(
        social.group_map_ping(D, 5, 5, at(&spots)),
        Err(SocialError::NotInGroup)
    );
    for bad in [MAX_MAP_COORD + 1, -MAX_MAP_COORD - 1, i32::MIN, i32::MAX] {
        assert_eq!(
            social.group_map_ping(A, bad, 0, at(&spots)),
            Err(SocialError::InvalidPosition)
        );
    }
    // A sender the registry cannot place sends nothing.
    let without_a = Spots::from([(B, spot(ZONE, 1, 1.0, 1.0))]);
    assert!(
        social
            .group_map_ping(A, 1, 1, at(&without_a))
            .unwrap()
            .is_empty()
    );
    // C ignores A: C gets no ping from A, B still does.
    social.add_ignore(C, &name(A), &directory).unwrap();
    let sent = social.group_map_ping(A, 11, 12, at(&spots)).unwrap();
    let mut recipients: Vec<u64> = sent.iter().map(|m| m.recipient_guid).collect();
    recipients.sort();
    assert_eq!(recipients, vec![A, B]);
}

#[test]
fn forged_sender_guid_in_a_ping_never_reaches_a_recipient() {
    let (mut social, directory) = world(&[]);
    group(&mut social, &directory, A, &[B]);
    let spots = Spots::from([(A, spot(ZONE, 1, 0.0, 0.0)), (B, spot(ZONE, 1, 1.0, 1.0))]);
    // A forges D's guid in the body; D is not even in the group.
    let Some(LocationRequest::MapPing { x, z }) = parse_location_request(&ping_bytes(D, 10, 20))
    else {
        panic!("well-formed ping must parse");
    };
    let sent = social.group_map_ping(A, x, z, at(&spots)).unwrap();
    assert_eq!(sent.len(), 2);
    for message in &sent {
        assert_eq!(message.bytes, ping_bytes(A, 10, 20));
        assert!(!contains(&message.bytes, &D.to_le_bytes()));
    }
}

// ---- forwarding (shared-file patch proof) --------------------------------------------------

/// `gateway::application_handler` decides whether an application message reaches the binary's
/// social router or is dropped as unknown. Until its `social_router` arm lists 26/21 and 40/14|16
/// the three new requests never leave the gateway, so the packets above are unreachable.
#[test]
fn gateway_forwards_the_new_selectors_to_the_social_router() {
    let source = include_str!("../src/gateway.rs");
    let handler = source
        .split("fn application_handler(")
        .nth(1)
        .unwrap()
        .split("fn set_trace(")
        .next()
        .unwrap();
    // Every arm that ends in `=> "social_router"`, back to the previous arm's `",`.
    let mut pieces: Vec<&str> = handler.split("=> \"social_router\"").collect();
    pieces.pop();
    let arm: String = pieces
        .iter()
        .map(|piece| &piece[piece.rfind("\",").map_or(0, |at| at + 2)..])
        .collect();
    let listed = |family: u16, sub: u16| {
        arm.split("(Some(").skip(1).any(|pattern| {
            let (head, rest) = pattern.split_once("),").unwrap();
            head.trim().parse::<u16>().unwrap() == family
                && rest
                    .trim_start()
                    .strip_prefix("Some(")
                    .unwrap()
                    .split(')')
                    .next()
                    .unwrap()
                    .split('|')
                    .any(|s| s.trim().parse::<u16>().ok() == Some(sub))
        })
    };
    for (family, sub) in [(26, 14), (40, 1), (26, 21), (40, 14), (40, 16)] {
        assert!(
            listed(family, sub),
            "{family}/{sub} must reach social_router"
        );
    }
}
