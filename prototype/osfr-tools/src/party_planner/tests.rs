use super::wire::{self, Choice, Word};
use super::*;
use crate::social_chat::NameData;
use crate::startup_world::parse_add_npc;
use std::path::PathBuf;

const HOST: u64 = 1;
const GUEST: u64 = 2;
const STRANGER: u64 = 3;
const PRESTON: u64 = 9_500_000_514;

fn shipped() -> Venues {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    Venues::load(&dir).unwrap()
}

fn back() -> Back {
    Back {
        instance: 0,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

fn parties() -> (Parties, Arc<Venue>) {
    let venues = shipped();
    let venue = Arc::clone(venues.by_planner(PRESTON).unwrap());
    let mut parties = Parties::default();
    parties.set_venues(venues);
    (parties, venue)
}

#[test]
fn the_shipped_table_binds_the_three_planners_to_their_venues() {
    let venues = shipped();
    let planners: Vec<_> = venues.iter().map(|v| (v.key.as_str(), v.planner)).collect();
    assert_eq!(
        planners,
        [
            ("pixiewood", PRESTON),
            ("lakeshore_treehouse", 100_000_003_084),
            ("tanglewood_towers", 9_500_000_172),
        ]
    );
    assert_eq!(venues.duration_ms, 3_600_000, "60 minutes (R)");
    assert!(venues.max_guests >= 2);
    assert!(venues.by_planner(GUEST).is_none());
    // A missing file is no venues, not an error.
    assert_eq!(
        Venues::load(Path::new("no-such-dir")).unwrap(),
        Venues::default()
    );
}

#[test]
fn a_host_books_once_and_a_private_party_admits_only_the_invited() {
    let (mut parties, venue) = parties();
    let guid = parties
        .book(
            HOST,
            Arc::clone(&venue),
            Privacy::Private,
            70,
            back(),
            1_000,
        )
        .unwrap();
    assert!(is_party(guid) && guid & 0xff == 0);
    assert_eq!(parties.get(guid).unwrap().ends_ms, 3_601_000);
    assert_eq!(
        parties.book(
            HOST,
            Arc::clone(&venue),
            Privacy::Private,
            71,
            back(),
            1_000
        ),
        Err(Refusal::Hosting)
    );
    // Not invited: refused, even as a friend of a private host.
    assert_eq!(
        parties.may_enter(guid, GUEST, true),
        Err(Refusal::NotInvited)
    );
    assert_eq!(parties.invite(HOST, HOST), Err(Refusal::Yourself));
    assert_eq!(parties.invite(GUEST, STRANGER), Err(Refusal::NotHosting));
    assert_eq!(parties.invite(HOST, GUEST), Ok(guid));
    assert_eq!(parties.may_enter(guid, GUEST, false), Ok(()));
    assert_eq!(
        parties.may_enter(guid, STRANGER, false),
        Err(Refusal::NotInvited)
    );
    assert_eq!(
        parties.may_enter(guid ^ 0x100, GUEST, false),
        Err(Refusal::Unknown)
    );
    // Admitted guests take the spots in turn; nobody is in two parties.
    // The host has the first spot; the first guest gets the second.
    assert_eq!(parties.host_spot(guid), Some(venue.spots[0]));
    let first = parties.admit(guid, GUEST, back()).unwrap();
    assert_eq!(first, venue.spots[1]);
    assert_eq!(parties.may_enter(guid, GUEST, false), Err(Refusal::InParty));
    assert_eq!(
        parties.book(GUEST, venue, Privacy::Private, 72, back(), 1_000),
        Err(Refusal::InParty)
    );
    assert_eq!(parties.containing(GUEST).map(|p| p.guid), Some(guid));
    // A decline forgets the invite and names the host.
    parties.invite(HOST, STRANGER).unwrap();
    assert_eq!(parties.decline(guid, STRANGER), Some(HOST));
    assert_eq!(parties.decline(guid, STRANGER), None);
}

#[test]
fn friends_and_group_come_without_an_invite_and_the_party_fills() {
    let (mut parties, venue) = parties();
    let guid = (parties.book(
        HOST,
        Arc::clone(&venue),
        Privacy::FriendsAndGroup,
        70,
        back(),
        0,
    ))
    .unwrap();
    assert_eq!(parties.may_enter(guid, GUEST, true), Ok(()));
    assert_eq!(
        parties.may_enter(guid, STRANGER, false),
        Err(Refusal::NotInvited)
    );
    let cap = parties.venues().unwrap().max_guests as u64;
    for guest in 10..10 + cap - 1 {
        parties.admit(guid, guest, back()).unwrap();
    }
    assert_eq!(parties.may_enter(guid, GUEST, true), Err(Refusal::Full));
    // The planner finds the party for a related player, and an invited one first.
    assert_eq!(parties.joinable("pixiewood", GUEST, |_| true), None, "full");
    parties.leave(guid, 10).unwrap();
    assert_eq!(
        parties.joinable("pixiewood", GUEST, |host| host == HOST),
        Some(guid)
    );
    assert_eq!(parties.joinable("pixiewood", GUEST, |_| false), None);
    assert_eq!(
        parties.joinable("lakeshore_treehouse", GUEST, |_| true),
        None
    );
}

#[test]
fn a_party_is_due_at_its_end_and_ending_returns_everyone_with_their_way_back() {
    let (mut parties, venue) = parties();
    let guid = parties
        .book(HOST, venue, Privacy::Private, 70, back(), 0)
        .unwrap();
    parties.invite(HOST, GUEST).unwrap();
    let away = Back {
        instance: 5,
        ..back()
    };
    parties.admit(guid, GUEST, away).unwrap();
    assert!(parties.mark_shown(guid, GUEST));
    assert!(!parties.mark_shown(guid, GUEST), "once per stay");
    assert!(parties.due(3_599_999).is_empty());
    assert_eq!(parties.due(3_600_000), [guid]);
    let ended = parties.end(guid).unwrap();
    let guests: Vec<_> = ended.guests().map(|(who, at)| (who, at.instance)).collect();
    assert_eq!(guests, [(HOST, 0), (GUEST, 5)]);
    assert!(parties.get(guid).is_none() && parties.end(guid).is_none());
    // The host may book again once the party is over.
    let venue = Arc::clone(&ended.venue);
    assert!(
        parties
            .book(HOST, venue, Privacy::Private, 71, back(), 0)
            .is_ok()
    );
}

#[test]
fn the_invite_is_the_client_house_invite_keyed_by_the_party() {
    let party = GUID_TAG | 0x100;
    let host = NameData::new("Alpha", "Tester");
    let guest = NameData::new("Beta", "Tester");
    let packet = wire::invite(party, HOST, GUEST, &host, &guest);
    // `00ab8bd0`: two i16, four u64 (`00ab3540`), two NameData (`008e7290`), one u64.
    assert_eq!(&packet[..4], [127, 0, 47, 0]);
    let word = |at: usize| u64::from_le_bytes(packet[at..at + 8].try_into().unwrap());
    assert_eq!(
        [word(4), word(12), word(20), word(28)],
        [HOST, GUEST, party, 0]
    );
    let name = |at: usize| {
        let length = u32::from_le_bytes(packet[at..at + 4].try_into().unwrap()) as usize;
        (
            String::from_utf8(packet[at + 4..at + 4 + length].to_vec()).unwrap(),
            at + 4 + length,
        )
    };
    let (first, at) = name(36 + 12);
    let (last, at) = name(at);
    assert_eq!((first.as_str(), last.as_str()), ("Alpha", "Tester"));
    let (first, at) = name(at + 12);
    let (_, at) = name(at);
    assert_eq!(first, "Beta");
    assert_eq!(packet.len(), at + 8, "the last u64 ends the body");
    // The client's answers name the party back; a house's do not.
    let mut accept = vec![127, 0, 19, 0];
    accept.extend(party.to_le_bytes());
    accept.extend(0_u32.to_le_bytes());
    accept.push(0); // writer `00abca80`: u64 guid, u32, u8 flag
    assert_eq!(wire::accepted(&accept), Some(party));
    let mut house = vec![127, 0, 19, 0];
    house.extend(0xA_u64.to_le_bytes());
    house.extend(0_u32.to_le_bytes());
    house.push(0);
    assert_eq!(wire::accepted(&house), None);
    let mut decline = vec![127, 0, 16, 0];
    decline.extend(party.to_le_bytes());
    assert_eq!(wire::declined(&decline), Some(party));
    assert_eq!(wire::declined(&decline[..11]), None);
    assert_eq!(wire::declined(&[decline.as_slice(), &[0]].concat()), None);
}

#[test]
fn the_booking_dialog_speaks_the_planners_line_and_its_rows_answer_back() {
    let venues = shipped();
    let venue = venues.by_planner(PRESTON).unwrap();
    let packets = wire::dialog(
        venue,
        &[Choice::Private, Choice::FriendsAndGroup, Choice::Goodbye],
    );
    assert_eq!(packets[0], [26, 0, 4, 0]);
    let dialog = &packets[1];
    let dword = |at: usize| u32::from_le_bytes(dialog[at..at + 4].try_into().unwrap());
    assert_eq!(&dialog[..4], [26, 0, 3, 0]);
    assert_eq!(dword(4), 422_717, "Preston's line");
    assert_eq!(u64::from(dword(12)) | u64::from(dword(16)) << 32, PRESTON);
    // After the four dwords, a flag byte and a float, the row count, then five dwords a row.
    let rows_at = 4 + 16 + 1 + 4;
    assert_eq!(dword(rows_at), 3);
    let row = |n: usize| {
        let at = rows_at + 4 + n * 20;
        (dword(at) as i32, dword(at + 8))
    };
    assert_eq!(row(0).1, 422_707, "Pixiewood Private Party Ticket");
    assert_eq!(row(1).1, wire::FRIENDS_LABEL_ID);
    assert_eq!(row(2).1, 8_075, "Goodbye");
    for (n, choice) in [Choice::Private, Choice::FriendsAndGroup, Choice::Goodbye]
        .iter()
        .enumerate()
    {
        assert_eq!(Choice::from_response(row(n).0), Some(*choice));
    }
    for other in [0, 1, 5, 301, 310, 0x4D00_0001, 0x5000_0000, 0x5000_0006] {
        assert_eq!(Choice::from_response(other), None, "{other:#x}");
    }
}

#[test]
fn party_words_parse_from_either_prefix() {
    assert_eq!(wire::word("realm party"), Some(Word::Help));
    assert_eq!(wire::word("/party invite"), Some(Word::Invite(None)));
    assert_eq!(
        wire::word("  Realm Party invite  Beta   Tester "),
        Some(Word::Invite(Some("Beta Tester".into())))
    );
    assert_eq!(wire::word("realm party end"), Some(Word::End));
    assert_eq!(wire::word("realm party leave"), Some(Word::Leave));
    assert_eq!(wire::word("realm party dance"), Some(Word::Help));
    assert_eq!(wire::word("realm partyhat"), None);
    assert_eq!(wire::word("realm mail"), None);
    assert_eq!(wire::word("hello party people"), None);
}

#[test]
fn decorations_are_still_props_under_the_party_guid() {
    let venues = shipped();
    for venue in venues.iter() {
        let party = GUID_TAG | 0x200;
        let packets = decoration_packets(party, venue).unwrap();
        assert_eq!(packets.len(), venue.decorations.len(), "{}", venue.key);
        for (index, packet) in packets.iter().enumerate() {
            let npc = parse_add_npc(packet).unwrap();
            assert_eq!(npc.actor_id, party | (index as u64 + 1));
            assert!(!is_party(npc.actor_id), "a prop is not a party guid");
            assert_eq!(npc.model_id, venue.decorations[index].id);
            assert_eq!((npc.is_interactable, npc.hide_nameplate), (0, 1));
        }
    }
}
