//! PB3: party planners and venues through the socket path. Every packet goes through
//! `Session::dispatch_message` on a `test_support::Table` (three logged-in clients on one gateway,
//! one world and one membership); the party tick is the server's own `party_routing::tick`.
//! Session `index` is character `index + 1`: the host, a guest and a stranger, standing by
//! Preston Daly in Sanctuary.
#[path = "../../../tests/common/round70_content.rs"]
#[allow(dead_code)] // this test uses the paths, not the fixture builder
mod round70_content;
use super::super::tests::test_support::{self, Scratch, Table};
use super::*;
use osfr_tools::gameplay::{Gameplay, GameplayState};
use osfr_tools::npc_interaction::NpcInteractRequest;
use osfr_tools::party_planner::GUID_TAG;
use osfr_tools::placed_cakes::is_cake;

const PRESTON: u64 = 9_500_000_514;
const PRESTON_AT: [f32; 3] = [-1391.5509, -27.172358, 334.9232];
const HOST: usize = 0;
const GUEST: usize = 1;
const STRANGER: usize = 2;
const NAMES: [&str; 3] = ["Alpha", "Beta", "Gamma"];
/// The private and friends rows' response ids (`wire::Choice`), and the Join row.
const PRIVATE: i32 = 0x5000_0001;
const FRIENDS: i32 = 0x5000_0002;
const JOIN: i32 = 0x5000_0003;
/// The boss cake's four transforms (`placed_cakes.json` row 7974; B1 added Cakenstein, 22).
const BOSSES: [i32; 4] = [2055, 1944, 1702, 22];
const HOUR_MS: u64 = 3_600_000;

fn guid(index: usize) -> u64 {
    index as u64 + 1
}

fn player(content: &Path, guid: u64, at: [f32; 3]) -> (CharacterState, GameplayState) {
    let mut character = Gateway::reference_state(&round70_content::reference()).unwrap();
    character.guid = guid;
    character.zone_id = 0;
    character.position = at;
    let mut game = Gameplay::load(content, None).unwrap();
    game.initialize_world(&character, 0).unwrap();
    (character, game.state())
}

/// Three clients beside Preston, named and online as the world entry leaves them (the rig's
/// login stops short of `join_player`, as t889's derby test notes).
fn table(tag: &str) -> Option<Table> {
    if !test_support::bin_test_target("pb3") {
        return None;
    }
    let scratch = Scratch::new(tag);
    let content = test_support::repo_and_runtime_content(&scratch, &round70_content::source());
    let players: Vec<_> = (0..3)
        .map(|index| {
            let at = [
                PRESTON_AT[0] + 1.0 + index as f32,
                PRESTON_AT[1],
                PRESTON_AT[2],
            ];
            player(&content, guid(index), at)
        })
        .collect();
    let mut table = Table::new(scratch, &content, &round70_content::reference(), &players)?;
    for index in 0..3 {
        let (who, id) = (guid(index), table.server.sessions[&table.keys[index]].id);
        let identity = SocialIdentity {
            account_id: who + 10_000,
            character_guid: who,
            name: SocialName::new(NAMES[index], "Tester").unwrap(),
        };
        let world = &mut table.server.world;
        world.identities.insert(who, identity.clone());
        if world.social.profile(who).is_none() {
            world
                .social
                .register(SocialLists::new(identity).unwrap())
                .unwrap();
        }
        world.social.login(who).unwrap();
        world.active_sessions.insert(who, id);
        assert!(
            world.players.get(id).is_some(),
            "the rig registered the player"
        );
        world.joined.insert(id);
    }
    table.tick(1_000);
    Some(table)
}

fn click(target: u64) -> Vec<u8> {
    NpcInteractRequest {
        target,
        trailing_flag: false,
    }
    .encode()
}

fn row(response: i32) -> Vec<u8> {
    [vec![26, 0, 6, 0], response.to_le_bytes().to_vec()].concat()
}

fn housing(sub: u8, party: u64) -> Vec<u8> {
    let mut packet = vec![127, 0, sub, 0];
    packet.extend(party.to_le_bytes());
    if sub == 19 {
        // The client writer (`00abca80`): u64 guid, u32 0, u8 flag.
        packet.extend(0_u32.to_le_bytes());
        packet.push(0);
    }
    packet
}

fn dword(packet: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

fn u64_at(packet: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(packet[at..at + 8].try_into().unwrap())
}

/// The 26/3 dialog's body text id and its rows' `(response, label)`.
fn dialog(replies: &[Vec<u8>]) -> Option<(u32, Vec<(i32, u32)>)> {
    let packet = replies.iter().find(|p| p.starts_with(&[26, 0, 3, 0]))?;
    let rows_at = 4 + 16 + 1 + 4;
    let rows = (0..dword(packet, rows_at) as usize)
        .map(|n| {
            let at = rows_at + 4 + n * 20;
            (dword(packet, at) as i32, dword(packet, at + 8))
        })
        .collect();
    Some((dword(packet, 4), rows))
}

fn said(replies: &[Vec<u8>], needle: &str) -> bool {
    replies.iter().any(|packet| {
        packet.starts_with(&[15, 0, 3, 0]) && String::from_utf8_lossy(&packet[8..]).contains(needle)
    })
}

fn zoned(replies: &[Vec<u8>]) -> bool {
    replies.iter().any(|p| p.starts_with(&[31, 0]))
}

/// The character's membership instance.
fn instance(table: &Table, index: usize) -> Option<i32> {
    let location = table.server.membership.player_location(guid(index))?;
    Some(location.instance_id)
}

fn state(table: &Table, index: usize) -> CharacterState {
    table.server.sessions[&table.keys[index]]
        .gateway
        .state()
        .clone()
}

/// A client that zoned loads the scene again: Ready and Loaded.
fn load(table: &mut Table, index: usize, now: u64) {
    table.send(index, &[13, 0], now);
    table.send(index, &[10, 0], now + 1);
}

/// The host clicks Preston and takes `choice`; the host's replies.
fn book(table: &mut Table, choice: i32, now: u64) -> Vec<Vec<u8>> {
    table.send(HOST, &click(PRESTON), now);
    table.send(HOST, &row(choice), now + 10)
}

fn the_party(table: &Table) -> (u64, i32) {
    let party = table
        .server
        .membership
        .parties()
        .iter()
        .next()
        .expect("a live party");
    (party.guid, party.instance)
}

fn tick(table: &mut Table, now: u64) {
    party_routing::tick(&mut table.server, now);
}

#[test]
fn pb3_a_planner_click_offers_booking_and_a_far_click_is_silent() {
    let Some(mut table) = table("pb3-offer") else {
        return;
    };
    let replies = table.send(HOST, &click(PRESTON), 2_000);
    assert!(
        replies.iter().any(|p| p == &[26, 0, 4, 0]),
        "the window closes first"
    );
    let (body, rows) = dialog(&replies).expect("the booking dialog");
    assert_eq!(body, 422_717, "Preston's own line");
    let labels: Vec<u32> = rows.iter().map(|(_, label)| *label).collect();
    assert_eq!(
        labels,
        [422_707, 3_482, 8_075],
        "private ticket, Friends, Goodbye"
    );
    // Goodbye closes the window and books nothing.
    let bye = table.send(HOST, &row(300), 2_100);
    assert!(bye.iter().any(|p| p == &[26, 0, 4, 0]));
    assert!(table.server.membership.parties().iter().next().is_none());
    // A row with no prompt open is not the party's (gameplay answers it as before).
    table.send(HOST, &row(PRIVATE), 2_200);
    assert!(table.server.membership.parties().iter().next().is_none());
    // From 30 m away the click opens nothing.
    let id = table.server.sessions[&table.keys[STRANGER]].id;
    let far = [PRESTON_AT[0] + 30.0, PRESTON_AT[1], PRESTON_AT[2]];
    let zone = table.server.world.players.get(id).unwrap().zone;
    table
        .server
        .world
        .players
        .update_pose_with_state(id, zone, far, 0.0, 0, 0)
        .unwrap();
    assert!(dialog(&table.send(STRANGER, &click(PRESTON), 2_300)).is_none());
}

#[test]
fn pb3_the_host_books_a_private_venue_lands_there_and_is_shown_its_props() {
    let Some(mut table) = table("pb3-book") else {
        return;
    };
    let before = state(&table, HOST);
    let replies = book(&mut table, PRIVATE, 2_000);
    assert!(zoned(&replies), "opcode-31 begin zoning");
    assert!(
        said(&replies, "Welcome to the Pixiewood Hills party"),
        "{replies:?}"
    );
    let (party, venue) = the_party(&table);
    assert!(party & 0xffff_ffff_0000_0000 == GUID_TAG);
    assert_eq!(instance(&table, HOST), Some(venue));
    let kind = &table.server.membership.instance(venue).unwrap().kind;
    assert_eq!(*kind, InstanceKind::Party { party_guid: party });
    assert_eq!(
        state(&table, HOST).position,
        [-1194.0, 23.0, 506.0],
        "the first spot"
    );
    assert_ne!(venue, 0, "its own instance, not the open world's");
    // A second booking is refused while hosting.
    table.send(HOST, &click(PRESTON), 2_100);
    // (the host is in the venue now; the planner is out of reach there)
    assert_eq!(table.server.membership.parties().iter().count(), 1);

    // Loaded in the venue, the next tick shows the decorations, the cakes and the boombox.
    load(&mut table, HOST, 3_000);
    tick(&mut table, 3_100);
    let props = table.drain(HOST, 3_100);
    let added: Vec<u64> = (props.iter())
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .map(|p| u64_at(p, 4))
        .collect();
    let decorations = added.iter().filter(|g| **g & !0xff == party).count();
    assert_eq!(decorations, 3, "{added:x?}");
    assert_eq!(
        added.iter().filter(|g| is_cake(**g)).count(),
        2,
        "both venue cakes"
    );
    assert_eq!(table.server.world.cakes.live.len(), 2);
    tick(&mut table, 3_200);
    assert!(
        !table
            .drain(HOST, 3_200)
            .iter()
            .any(|p| p.starts_with(&[35, 0, 2, 0]) && is_cake(u64_at(p, 4))),
        "props once a stay"
    );
    // The boombox tick shows the venue's box (the 'BOOM' guid) with its song (35/41).
    super::super::boombox_routing::tick(&mut table.server.world, &mut table.server.sessions, 3_300);
    let boombox = table.drain(HOST, 3_300);
    let boom = |p: &&Vec<u8>| p.len() >= 12 && u64_at(p, 4) >> 32 == 0x424f_4f4d;
    assert!(
        boombox
            .iter()
            .filter(boom)
            .any(|p| p.starts_with(&[35, 0, 2, 0])),
        "{boombox:?}"
    );
    assert!(
        boombox
            .iter()
            .filter(boom)
            .any(|p| p.starts_with(&[35, 0, 41, 0])),
        "its song"
    );
    // The host's way back is where they booked.
    let back = table
        .server
        .membership
        .parties()
        .get(party)
        .unwrap()
        .guests()
        .next()
        .unwrap()
        .1;
    assert_eq!(back.position, before.position);
}

#[test]
fn pb3_an_invited_guest_lands_in_the_same_instance_and_a_stranger_cannot_enter() {
    let Some(mut table) = table("pb3-invite") else {
        return;
    };
    book(&mut table, PRIVATE, 2_000);
    let (party, venue) = the_party(&table);
    load(&mut table, HOST, 2_500);
    // The host invites by name from inside the venue.
    let invite = osfr_tools::chat::local_request("realm party invite beta tester");
    assert!(said(
        &table.send(HOST, &invite, 3_000),
        "Invited Beta Tester"
    ));
    let got = table.drain(GUEST, 3_000);
    let prompt = got
        .iter()
        .find(|p| p.starts_with(&[127, 0, 47, 0]))
        .expect("the invite prompt");
    assert_eq!(
        u64_at(prompt, 20),
        party,
        "the prompt is keyed by the party"
    );
    assert!(said(
        &got,
        "Alpha Tester invites you to the Pixiewood Hills party"
    ));
    // The stranger got nothing, and an unknown name is refused.
    assert!(
        !table
            .drain(STRANGER, 3_000)
            .iter()
            .any(|p| p.starts_with(&[127, 0, 47, 0]))
    );
    let nobody = osfr_tools::chat::local_request("realm party invite Nobody Here");
    assert!(said(
        &table.send(HOST, &nobody, 3_050),
        "No player named Nobody Here"
    ));

    // The guest accepts: the same instance, at the next spot.
    let accepted = table.send(GUEST, &housing(19, party), 3_100);
    assert!(
        zoned(&accepted) && said(&accepted, "Welcome to"),
        "{accepted:?}"
    );
    assert_eq!(instance(&table, GUEST), Some(venue));
    assert_eq!(state(&table, GUEST).position, [-1196.5, 22.77, 505.0]);
    load(&mut table, GUEST, 3_200);
    tick(&mut table, 3_300);
    assert!(
        table
            .drain(GUEST, 3_300)
            .iter()
            .any(|p| p.starts_with(&[35, 0, 2, 0]))
    );

    // The stranger forges an accept for the same party: refused, still in the open world.
    let refused = table.send(STRANGER, &housing(19, party), 3_400);
    assert!(!zoned(&refused));
    assert!(said(&refused, "That party is private"), "{refused:?}");
    assert_eq!(instance(&table, STRANGER), Some(0));
    // The planner offers the stranger no way in either.
    let (_, rows) = dialog(&table.send(STRANGER, &click(PRESTON), 3_500)).unwrap();
    assert!(
        !rows.iter().any(|(response, _)| *response == JOIN),
        "{rows:?}"
    );
    let joined = table.send(STRANGER, &row(JOIN), 3_600);
    assert!(!zoned(&joined));
    assert_eq!(instance(&table, STRANGER), Some(0));
}

#[test]
fn pb3_friends_and_group_join_through_the_planner_and_a_decline_is_told_to_the_host() {
    let Some(mut table) = table("pb3-friends") else {
        return;
    };
    // Host and guest are mutual friends; the stranger is not.
    let world = &mut table.server.world;
    for (a, b) in [(guid(HOST), guid(GUEST)), (guid(GUEST), guid(HOST))] {
        let name = world.identities[&b].name.clone();
        world.social.profile_mut(a).unwrap().friends.insert(b, name);
    }
    book(&mut table, FRIENDS, 2_000);
    let (party, venue) = the_party(&table);
    load(&mut table, HOST, 2_500);
    let (_, rows) = dialog(&table.send(GUEST, &click(PRESTON), 3_000)).unwrap();
    assert!(
        rows.iter().any(|(response, _)| *response == JOIN),
        "{rows:?}"
    );
    let joined = table.send(GUEST, &row(JOIN), 3_100);
    assert!(zoned(&joined), "{joined:?}");
    assert_eq!(instance(&table, GUEST), Some(venue));
    // The stranger is not a friend: no Join row, and an accept without an invite is refused.
    let (_, rows) = dialog(&table.send(STRANGER, &click(PRESTON), 3_200)).unwrap();
    assert!(!rows.iter().any(|(response, _)| *response == JOIN));
    assert!(!zoned(&table.send(STRANGER, &housing(19, party), 3_300)));
    // Invited, the stranger declines; the host hears of it and the invite is gone.
    let invite = osfr_tools::chat::local_request("realm party invite Gamma Tester");
    table.send(HOST, &invite, 3_400);
    assert!(
        table
            .server
            .membership
            .parties()
            .get(party)
            .unwrap()
            .is_invited(guid(STRANGER))
    );
    table.send(STRANGER, &housing(16, party), 3_500);
    assert!(said(
        &table.drain(HOST, 3_500),
        "Gamma Tester declined your party invitation"
    ));
    assert!(
        !table
            .server
            .membership
            .parties()
            .get(party)
            .unwrap()
            .is_invited(guid(STRANGER))
    );
}

#[test]
fn pb3_a_venue_cake_click_gives_its_temporary_effect() {
    let Some(mut table) = table("pb3-cake") else {
        return;
    };
    book(&mut table, PRIVATE, 2_000);
    load(&mut table, HOST, 2_500);
    tick(&mut table, 2_600);
    let props = table.drain(HOST, 2_600);
    let cakes: Vec<u64> = (props.iter())
        .filter(|p| p.starts_with(&[35, 0, 2, 0]) && is_cake(u64_at(p, 4)))
        .map(|p| u64_at(p, 4))
        .collect();
    // The boss cake (the second venue cake) stands 4.5 m from the host's spot, inside its 5 m.
    let boss = cakes[1];
    let got = table.send(HOST, &click(boss), 3_000);
    let transforms: Vec<i32> = (got.iter())
        .filter(|p| p.starts_with(&[35, 0, 14, 0]))
        .map(|p| dword(p, 4) as i32)
        .collect();
    assert_eq!(transforms.len(), 1, "{got:?}");
    assert!(BOSSES.contains(&transforms[0]), "{transforms:?}");
    // The first cake is out of reach from here: silent.
    assert!(
        !table
            .send(HOST, &click(cakes[0]), 9_000)
            .iter()
            .any(|p| p.starts_with(&[35, 0, 14, 0]))
    );
}

#[test]
fn pb3_the_party_ends_on_time_and_everyone_goes_back_where_they_came_from() {
    let Some(mut table) = table("pb3-time") else {
        return;
    };
    let (host_from, guest_from) = (state(&table, HOST).position, state(&table, GUEST).position);
    book(&mut table, PRIVATE, 2_000);
    let (party, venue) = the_party(&table);
    load(&mut table, HOST, 2_050);
    table.send(
        HOST,
        &osfr_tools::chat::local_request("realm party invite Beta Tester"),
        2_100,
    );
    table.send(GUEST, &housing(19, party), 2_200);
    load(&mut table, GUEST, 2_400);
    tick(&mut table, 2_500);
    assert_eq!(
        (instance(&table, HOST), instance(&table, GUEST)),
        (Some(venue), Some(venue))
    );
    // One tick before the hour nothing happens; at the hour both go back.
    let ends = table
        .server
        .membership
        .parties()
        .get(party)
        .unwrap()
        .ends_ms;
    assert_eq!(ends, 2_010 + HOUR_MS);
    tick(&mut table, ends - 1);
    assert!(table.server.membership.parties().get(party).is_some());
    tick(&mut table, ends);
    assert!(table.server.membership.parties().get(party).is_none());
    for (index, from) in [(HOST, host_from), (GUEST, guest_from)] {
        let replies = table.drain(index, ends);
        assert!(
            zoned(&replies) && said(&replies, "The party is over"),
            "{index}: {replies:?}"
        );
        assert_eq!(
            instance(&table, index),
            Some(0),
            "{index}: back in the open world"
        );
        assert_eq!(
            state(&table, index).position,
            from,
            "{index}: where they came from"
        );
    }
    assert!(
        table.server.world.cakes.live.is_empty(),
        "the venue cakes went with it"
    );
}

#[test]
fn pb3_the_party_ends_when_the_host_leaves_and_the_guest_goes_back() {
    let Some(mut table) = table("pb3-host-left") else {
        return;
    };
    let guest_from = state(&table, GUEST).position;
    book(&mut table, PRIVATE, 2_000);
    let (party, venue) = the_party(&table);
    load(&mut table, HOST, 2_050);
    table.send(
        HOST,
        &osfr_tools::chat::local_request("realm party invite Beta Tester"),
        2_100,
    );
    table.send(GUEST, &housing(19, party), 2_200);
    load(&mut table, GUEST, 2_300);
    assert_eq!(instance(&table, GUEST), Some(venue));
    // The host logs out.
    let key = table.keys[HOST];
    let session = table.server.sessions.remove(&key).unwrap();
    table.server.end_session(key, session, "pb3_test_logout");
    tick(&mut table, 3_000);
    assert!(table.server.membership.parties().get(party).is_none());
    let replies = table.drain(GUEST, 3_000);
    assert!(said(&replies, "The host has left"), "{replies:?}");
    assert_eq!(instance(&table, GUEST), Some(0));
    assert_eq!(state(&table, GUEST).position, guest_from);
}

#[test]
fn pb3_a_guest_leaves_with_a_word_and_the_host_ends_with_one() {
    let Some(mut table) = table("pb3-leave") else {
        return;
    };
    book(&mut table, PRIVATE, 2_000);
    let (party, venue) = the_party(&table);
    load(&mut table, HOST, 2_050);
    table.send(
        HOST,
        &osfr_tools::chat::local_request("realm party invite Beta Tester"),
        2_100,
    );
    table.send(GUEST, &housing(19, party), 2_200);
    load(&mut table, GUEST, 2_300);
    // The guest's own words take them back where they came from.
    let left = table.send(
        GUEST,
        &osfr_tools::chat::local_request("realm party leave"),
        2_400,
    );
    assert!(
        zoned(&left) && said(&left, "You have left the party"),
        "{left:?}"
    );
    assert_eq!(instance(&table, GUEST), Some(0));
    assert!(
        !table
            .server
            .membership
            .parties()
            .get(party)
            .unwrap()
            .has_guest(guid(GUEST))
    );
    // The host ends the party with a word: back where they booked.
    let ended = table.send(
        HOST,
        &osfr_tools::chat::local_request("realm party end"),
        2_500,
    );
    assert!(
        zoned(&ended) && said(&ended, "You ended your party"),
        "{ended:?}"
    );
    assert!(table.server.membership.parties().get(party).is_none());
    assert_ne!(instance(&table, HOST), Some(venue));
}
