//! T889: what two clients on one shared gateway get from a field enemy click, from the derby
//! trainer and from the kart race NPC. Every packet takes the socket path
//! (`Session::dispatch_message`, the shared-field step) through `test_support::Table`; nothing
//! calls an internal.
#[path = "fishing_fixtures.rs"]
#[allow(dead_code)] // the shared fixtures module also ships builders this test does not use
mod fixtures;
#[path = "../../../tests/support/pickup_packets.rs"]
mod pickup_packets;
#[path = "../../../tests/common/round70_content.rs"]
#[allow(dead_code)] // this test uses the paths, not the fixture builder
mod round70_content;
use super::test_support::{self, Scratch, Table};
use super::*;
use fixtures::{fish_option, npc};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
};
use pickup_packets::said as says;

const HOOLIGAN: u64 = 100000004667;
const HOOLIGAN_AT: [f32; 3] = [-2033.0, -34.1, 634.0];
const ARCHER: u64 = 100000004675;
const ARCHER_AT: [f32; 3] = [-2090.0, -25.9, 694.0];
const WOLF: u64 = 131000102001;
const WOLF_AT: [f32; 3] = [-1899.204, -50.59, 307.902];
const MAD_JACK: u64 = 9000000391;
const MAD_JACK_AT: [f32; 3] = [-2253.199, -17.16412, 590.1331];
const JOHNNY: u64 = 9500000911;
const JOHNNY_AT: [f32; 3] = [-2255.642, -16.61412, 579.8773];
/// L096: the Wildwood Speedway turnstile actors the live server spawned (kart race, derby).
const KART_GATE: u64 = 9500001020;
const DERBY_GATE: u64 = 9500001021;
const TURNSTILE_AT: [f32; 3] = [-2268.808, -16.374, 586.806];
const BRAWLER: i32 = 43;
/// A job the opening ability table has no basic attack for (the starting job).
const NO_BASIC: i32 = 1;
/// "Demo Derby": the solo match's row on a gate that also lists the lobby (`derby_lobby.json`).
const DERBY_LABEL: u32 = 382_789;
/// "Demo Derby Lobby": the game lobby's row, lobby type 2.
const DERBY_LOBBY_LABEL: u32 = 3053;
const DERBY_LOBBY: u32 = 2;
const LEARN_LABEL: u32 = 412342;
const RACE_LABEL: u32 = 3052;
const MOVE_CLOSER: &str = "Move closer to attack.";
const SWITCH_JOB: &str = "Switch to a combat job before attacking.";

/// A fresh character (guid `guid`) standing at `at` in the open world on job `profile`.
fn player(
    content: &Path,
    guid: u64,
    at: [f32; 3],
    profile: i32,
) -> (CharacterState, GameplayState) {
    let mut character = Gateway::reference_state(&round70_content::reference()).unwrap();
    character.guid = guid;
    character.zone_id = 0;
    character.position = at;
    let mut game = Gameplay::load(content, None).unwrap();
    game.initialize_world(&character, 0).unwrap();
    let mut saved = game.state();
    let profiles = saved.profiles.as_mut().unwrap();
    profiles.unlocked.extend([NO_BASIC, BRAWLER]);
    profiles.active = profile;
    (character, saved)
}

/// Two clients at `at`, one on each of `jobs`, entered and settled.
fn table(tag: &str, at: [f32; 3], jobs: [i32; 2]) -> Option<Table> {
    if !test_support::bin_test_target("t889") {
        return None;
    }
    let scratch = Scratch::new(tag);
    let content = test_support::repo_and_runtime_content(&scratch, &round70_content::source());
    let players: Vec<_> = jobs
        .iter()
        .enumerate()
        .map(|(index, job)| player(&content, index as u64 + 1, at, *job))
        .collect();
    let mut table = Table::new(scratch, &content, &round70_content::reference(), &players)?;
    table.tick(1_000);
    Some(table)
}

fn near_wolf(metres: f32) -> [f32; 3] {
    [WOLF_AT[0] + metres, WOLF_AT[1], WOLF_AT[2]]
}

fn menu(replies: &[Vec<u8>]) -> bool {
    replies
        .iter()
        .any(|reply| reply.starts_with(&[26, 0, 9, 0]))
}

fn health(table: &Table, guid: u64) -> u32 {
    table.field().health(guid).expect("the field enemy exists")
}

#[test]
fn both_clients_of_one_gateway_open_the_fight_with_a_field_enemy_click() {
    let Some(mut table) = table("t889-fight", near_wolf(1.0), [BRAWLER, BRAWLER]) else {
        return;
    };
    for index in 0..2 {
        let now = 2_000 + index as u64 * 1_000;
        let before = health(&table, WOLF);
        let opened = table.send(index, &npc(8, WOLF, None), now);
        assert!(
            menu(&opened),
            "client {index}: the click answers the Attack menu"
        );
        table.send(index, &npc(10, WOLF, Some(103)), now + 100);
        assert!(
            health(&table, WOLF) < before,
            "client {index}: the Attack choice hurts the wolf"
        );
    }
}

#[test]
fn a_job_without_a_basic_attack_is_told_why_on_the_shared_field() {
    let Some(mut table) = table("t889-nobasic", near_wolf(1.0), [BRAWLER, NO_BASIC]) else {
        return;
    };
    let before = health(&table, WOLF);
    // Client 1 is on a job with no basic attack; it used to get silence for the Attack choice.
    assert!(menu(&table.send(1, &npc(8, WOLF, None), 2_000)));
    let refused = table.send(1, &npc(10, WOLF, Some(103)), 2_100);
    assert!(
        says(&refused, SWITCH_JOB),
        "the Attack choice says why: {refused:?}"
    );
    // The key-1 cast (36/10, bar 1, slot 0, kind 0, target) gets the client's cast failure.
    let mut cast = vec![36, 0, 10, 0];
    for word in [1_i32, 0, 0] {
        cast.extend(word.to_le_bytes());
    }
    cast.extend(WOLF.to_le_bytes());
    let refused = table.send(1, &cast, 2_200);
    assert_eq!(
        refused,
        vec![osfr_tools::abilities::unsupported_cast_failure()],
        "the cast answers the client's own failure"
    );
    assert_eq!(
        health(&table, WOLF),
        before,
        "a refused attack hurts nothing"
    );
    // The other client's fight is untouched by it.
    table.send(0, &npc(10, WOLF, Some(103)), 3_000);
    assert!(health(&table, WOLF) < before, "the brawler still fights");
}

#[test]
fn a_click_beyond_reach_says_to_move_closer() {
    let Some(mut table) = table("t889-reach", near_wolf(8.0), [BRAWLER, BRAWLER]) else {
        return;
    };
    for index in 0..2 {
        let replies = table.send(index, &npc(8, WOLF, None), 2_000 + index as u64);
        assert!(!menu(&replies), "client {index}: no menu from 8 m");
        assert!(
            says(&replies, MOVE_CLOSER),
            "client {index}: told to move closer: {replies:?}"
        );
    }
}

#[test]
fn the_derby_trainer_grants_the_job_and_the_lobby_becomes_ready_for_both_clients() {
    let Some(mut table) = table("t889-derby", MAD_JACK_AT, [BRAWLER, BRAWLER]) else {
        return;
    };
    for index in 0..2 {
        let now = 2_000 + index as u64 * 10_000;
        let opened = table.send(index, &npc(8, MAD_JACK, None), now);
        let learn = fish_option(&opened, LEARN_LABEL).expect("the trainer offers the Learn row");
        let derby = fish_option(&opened, DERBY_LABEL).expect("the trainer offers the derby row");
        // "Become a Demo Derby driver" grants the job.
        let learned = table.send(index, &npc(10, MAD_JACK, Some(learn)), now + 100);
        assert!(
            says(&learned, "You learned the Demo Derby Driver job"),
            "{learned:?}"
        );
        // The lobby: the 39/16 offer, then the 41/107 that turns "Loading..." into Go.
        table.send(index, &npc(8, MAD_JACK, None), now + 200);
        let mut lobby = table.send(index, &npc(10, MAD_JACK, Some(derby)), now + 300);
        // L931: the choice zones into the arena; the offer waits for its load (Ready, Loaded).
        lobby.extend(table.send(index, &[13, 0], now + 301));
        lobby.extend(table.send(index, &[10, 0], now + 302));
        let offer = lobby
            .iter()
            .position(|reply| reply.starts_with(&[39, 0, 16]))
            .expect("the derby lobby offers its start dialog");
        let ready = lobby
            .iter()
            .position(|reply| reply.starts_with(&[41, 0, 107, 0]))
            .expect("the start dialog gets its ready signal");
        assert!(offer < ready, "the ready follows the offer");
        let (game, instance) = (&lobby[offer][11..15], &lobby[offer][3..7]);
        assert_eq!(
            &lobby[ready][4..8],
            game,
            "the ready names the offered game"
        );
        assert_eq!(
            &lobby[ready][8..12],
            instance,
            "the ready names the offered instance"
        );
        // Go is still answered (39/5 -> 39/17): the ready did not disturb the start.
        let mut go = vec![39, 0, 5];
        go.extend_from_slice(instance);
        go.extend((-1_i32).to_le_bytes());
        go.extend((-1_i32).to_le_bytes());
        let started = table.send(index, &go, now + 400);
        assert!(
            started.iter().any(|reply| reply.starts_with(&[39, 0, 17])),
            "{started:?}"
        );
    }
}

#[test]
fn the_kart_race_option_enters_the_race_zone_for_each_client() {
    let Some(mut table) = table("t889-race", TURNSTILE_AT, [BRAWLER, BRAWLER]) else {
        return;
    };
    for index in 0..2 {
        let now = 2_000 + index as u64 * 10_000;
        let opened = table.send(index, &npc(8, KART_GATE, None), now);
        let race = fish_option(&opened, RACE_LABEL).expect("the turnstile offers the race row");
        let entered = table.send(index, &npc(10, KART_GATE, Some(race)), now + 100);
        assert!(
            entered
                .iter()
                .any(|reply| reply.starts_with(&[31, 0, 11, 0])),
            "client {index}: the race zone load begins: {entered:?}"
        );
        assert!(
            entered
                .iter()
                .any(|reply| reply.starts_with(&[41, 0, 114, 0])),
            "client {index}: the kart session opens: {entered:?}"
        );
        let session = &table.server.sessions[&table.keys[index]];
        assert_eq!(session.gateway.state().zone_id, 48000001);
    }
}

/// L096 (owner 2026-11-01: the speedway gates, not NPCs, let you into the races): the two actors
/// the live server stood on the Wildwood turnstile offer the race and the derby; the racer who
/// stood in for the gate (Johnny Thunder) no longer offers the race.
#[test]
fn the_wildwood_turnstile_offers_the_race_and_the_derby() {
    let Some(mut gates) = table("t889-turnstile", TURNSTILE_AT, [BRAWLER, BRAWLER]) else {
        return;
    };
    let kart = gates.send(0, &npc(8, KART_GATE, None), 2_000);
    assert!(fish_option(&kart, RACE_LABEL).is_some(), "{kart:?}");
    let derby = gates.send(0, &npc(8, DERBY_GATE, None), 3_000);
    assert!(fish_option(&derby, DERBY_LABEL).is_some(), "{derby:?}");
    assert!(fish_option(&derby, RACE_LABEL).is_none());

    let Some(mut johnny) = table("t889-no-stand-in", JOHNNY_AT, [BRAWLER, BRAWLER]) else {
        return;
    };
    let menu = johnny.send(0, &npc(8, JOHNNY, None), 2_000);
    assert!(fish_option(&menu, RACE_LABEL).is_none(), "{menu:?}");
}

/// The Legacy Hooligan (4667) and Hooligan Archer (4675) bodies the playtest clicked were passive
/// extra spawns: a blue nameplate and `unknown_runtime_guid` on a click. Each now has a hostile
/// field enemy with the same GUID, which hides the passive copy; both clients can fight it.
#[test]
fn the_tent_hooligans_are_hostile_field_enemies_both_clients_can_fight() {
    for (body, at) in [(HOOLIGAN, HOOLIGAN_AT), (ARCHER, ARCHER_AT)] {
        let Some(mut table) = table("t889-tent", [at[0] + 1.0, at[1], at[2]], [BRAWLER, BRAWLER])
        else {
            return;
        };
        table.tick(2_000);
        for index in 0..2 {
            // One 35/2 AddNpc for the GUID, and it is the hostile one (disposition word 0, red);
            // the passive Legacy copy is hidden, not drawn a second time as a blue prop.
            let added: Vec<_> = table.seen[index]
                .iter()
                .filter(|reply| {
                    reply.starts_with(&[35, 0, 2, 0]) && reply[4..12] == body.to_le_bytes()
                })
                .collect();
            assert_eq!(added.len(), 1, "client {index} is shown {body} once");
            assert_eq!(
                i32::from_le_bytes(added[0][77..81].try_into().unwrap()),
                0,
                "client {index} is shown {body} as hostile"
            );
            let now = 3_000 + index as u64 * 1_000;
            let before = health(&table, body);
            let opened = table.send(index, &npc(8, body, None), now);
            assert!(
                menu(&opened),
                "client {index}: the click answers {body}'s Attack menu: {opened:?}"
            );
            table.send(index, &npc(10, body, Some(103)), now + 100);
            assert!(
                health(&table, body) < before,
                "client {index}: the Attack choice hurts {body}"
            );
        }
    }
}

/// The sparkling plant near the Hooligan tent (collection spawn 830, `collection_nodes.json`):
/// each client within reach is shown it and a click gathers it. The server side of the click is
/// sound; whether the tent hides it from the pointer is a placement question for the client.
#[test]
fn the_collection_plant_near_the_hooligan_tent_is_shown_and_gathered_by_both_clients() {
    let at = [-2047.9564, -25.932749, 682.0194];
    let Some(mut table) = table("t889-plant", at, [BRAWLER, BRAWLER]) else {
        return;
    };
    table.tick(2_000);
    for index in 0..2 {
        let known = &table.server.sessions[&table.keys[index]].known_npcs;
        let plant = known
            .iter()
            .copied()
            .find(|guid| guid >> 40 == 0x4e_4f_44 && guid & 0xffff_ffff == 830)
            .expect("the plant is shown");
        let gathered = table.send(index, &npc(8, plant, None), 3_000 + index as u64 * 1_000);
        assert!(
            gathered
                .iter()
                .any(|reply| reply.starts_with(&[38, 0, 2, 0])),
            "client {index}: the click gathers the plant: {gathered:?}"
        );
    }
}

/// One `Session::tick` of client `index`, as `Server::step` runs it.
pub(super) fn tick_client(table: &mut Table, index: usize, now: u64) {
    let key = table.keys[index];
    let server = &mut table.server;
    let mut session = server.sessions.remove(&key).unwrap();
    let result = session.tick(
        &server.options,
        now,
        &mut server.world,
        &mut server.membership,
        &mut server.sessions,
    );
    server.sessions.insert(key, session);
    result.unwrap();
}

fn lobby_request(sub: u8, words: &[u32]) -> Vec<u8> {
    let mut packet = vec![101, 0, sub];
    for word in words {
        packet.extend(word.to_le_bytes());
    }
    packet
}

/// The Demo Derby Lobby over the socket path: both clients open the list at the gate, one creates
/// a two-seat room, the other joins, the host starts, and after the countdown each client's own
/// tick zones it into the arena with one shared match. The cars are made for both when the second
/// client registers, and a client that leaves the world gives its car up to the other.
#[test]
fn two_clients_start_one_derby_from_the_lobby() {
    use osfr_tools::derby::{DerbyMessage, DerbyPhase, ParticipantKey, encode_inner};
    let Some(mut table) = table("t889-derby-room", MAD_JACK_AT, [BRAWLER, BRAWLER]) else {
        return;
    };
    let has = |replies: &[Vec<u8>], head: [u8; 3]| replies.iter().any(|r| r.starts_with(&head));
    // The rig's login stops short of `join_player`: name each character and its session, as the
    // world entry does, so the lobby can seat and reach them.
    for index in 0..2 {
        let (guid, id) = (index as u64 + 1, table.server.sessions[&table.keys[index]].id);
        let identity = SocialIdentity {
            account_id: guid + 10_000,
            character_guid: guid,
            name: SocialName::new(["Alpha", "Beta"][index], "Tester").unwrap(),
        };
        table.server.world.identities.insert(guid, identity);
        table.server.world.active_sessions.insert(guid, id);
        // Joined, as `join_player` leaves a session (the rig has its registry row already): a
        // zone change then runs `zone_out` and `leave_player`.
        assert!(table.server.world.players.get(id).is_some(), "the rig registered the player");
        table.server.world.joined.insert(id);
    }
    for index in 0..2 {
        let opened = table.send(index, &npc(8, MAD_JACK, None), 2_000);
        let lobby = fish_option(&opened, DERBY_LOBBY_LABEL).expect("the gate lists its lobby");
        let list = table.send(index, &npc(10, MAD_JACK, Some(lobby)), 2_100);
        assert!(has(&list, [101, 0, 6]), "client {index}: the lobby's list opens");
    }
    // Max Players 2, Enable AI yes, Total Time 120 s, as the panel's 101/5 carries them.
    let create = lobby_request(5, &[1, 0, DERBY_LOBBY, 3, 1, 1, 8, 14, 10, 19]);
    assert!(has(&table.send(0, &create, 3_000), [101, 0, 1]), "the host is seated");
    assert!(has(&table.drain(1, 3_000), [101, 0, 4]), "the other list gets the row");
    let id = table.server.world.rooms.room_of(1).unwrap().id;
    assert_eq!(table.server.world.rooms.room(id).unwrap().capacity, 2, "the host's Max Players");
    assert!(has(&table.send(1, &lobby_request(1, &[id, DERBY_LOBBY]), 3_100), [101, 0, 1]));
    assert!(has(&table.send(0, &lobby_request(3, &[id, DERBY_LOBBY]), 3_200), [101, 0, 3]));
    assert!(has(&table.drain(1, 3_200), [101, 0, 3]), "both seats count down");

    room_routing::tick(&mut table.server, 8_300);
    assert!(table.server.world.tables.contains_key(&id), "the countdown opened the match");
    let register = |guid: u64| {
        let key = ParticipantKey(guid);
        encode_inner(&DerbyMessage::RegisterPlayer { key, unknown_key: 0 }).unwrap()
    };
    let phase = |table: &Table| match table.server.world.tables.get(&id) {
        Some(osfr_tools::hosted_table::HostedTable::Derby(derby)) => Some(derby.service().phase()),
        _ => None,
    };
    // The guest zones in, loads and registers before the host's own tick zones the host in. A
    // zone change is not leaving the world: both keep their seat, their car and their list.
    let mut instances = Vec::new();
    for index in [1, 0] {
        tick_client(&mut table, index, 8_350);
        for guid in [1, 2] {
            let seated = table.server.world.tables[&id].seated(guid);
            assert!(seated, "client {index} zoning in: driver {guid} keeps the seat");
            assert_eq!(table.server.world.rooms.watching(guid), Some(DERBY_LOBBY), "and the list");
        }
        assert_eq!(phase(&table), Some(DerbyPhase::Starting), "no setup before both registered");
        let mut start = table.drain(index, 8_350);
        start.extend(table.send(index, &[13, 0], 8_400));
        start.extend(table.send(index, &[10, 0], 8_401));
        let offer = start.iter().find(|reply| reply.starts_with(&[39, 0, 16]));
        let offer = offer.expect("each client gets the match's offer after its arena loads");
        assert!(has(&start, [39, 0, 17]), "client {index}: the countdown was the Go");
        instances.push(offer[3..7].to_vec());
        if index == 1 {
            table.send(1, &register(2), 8_500);
        }
    }
    assert_eq!(instances[0], instances[1], "one match");
    let heads = |replies: &[Vec<u8>]| -> Vec<Vec<u8>> {
        replies.iter().map(|reply| reply[..reply.len().min(3)].to_vec()).collect()
    };
    // The match is set up when the second client registers: state 3 goes to both, the sender in
    // its own replies and the client that waited through the relay.
    table.drain(1, 9_000);
    let own = table.send(0, &register(1), 9_100);
    assert_eq!(phase(&table), Some(DerbyPhase::WaitingForState(3)));
    assert_eq!(own.iter().filter(|r| r.starts_with(&[34, 0, 1])).count(), 1, "the sender's own setup, once: {:?}", heads(&own));
    let relayed = table.drain(1, 9_100);
    assert_eq!(relayed.iter().filter(|r| r.starts_with(&[34, 0, 1])).count(), 1, "the same for the client that waited, once: {:?}", heads(&relayed));
    // After the match the list is still theirs: a kept lobby can create again.
    assert_eq!(table.server.world.rooms.watching(1), Some(DERBY_LOBBY));
    // Each state gate takes both clients' acknowledgements, whichever comes last.
    let ack = |guid: u64| encode_inner(&DerbyMessage::PlayerPhase { key: ParticipantKey(guid), phase: 3 }).unwrap();
    table.send(1, &ack(2), 9_200);
    assert_eq!(phase(&table), Some(DerbyPhase::WaitingForState(3)));
    table.send(0, &ack(1), 9_300);
    assert_eq!(phase(&table), Some(DerbyPhase::WaitingForState(4)));

    // A client that leaves the world gives its car up; the other is told, and the room is its own.
    let gone = table.server.world.leave_rooms(2);
    assert!(!gone.is_empty(), "the other driver hears the car go");
    assert!(table.server.world.tables.contains_key(&id));
    assert!(table.server.world.rooms.room_of(2).is_none());
    table.server.world.leave_rooms(1);
    assert!(!table.server.world.tables.contains_key(&id), "the last driver out closes the match");
}

/// A room whose only driver never registers: the start times out on them in the real room tick,
/// the settled match leaves the World, and the driver's own next tick takes them out of the game
/// and the seat.
#[test]
fn a_start_that_times_out_on_its_only_driver_leaves_no_table_and_frees_them() {
    let Some(mut table) = table("t889-derby-timeout", MAD_JACK_AT, [BRAWLER, BRAWLER]) else {
        return;
    };
    let id = table.server.sessions[&table.keys[0]].id;
    let name = SocialName::new("Alpha", "Tester").unwrap();
    let identity = SocialIdentity { account_id: 10_001, character_guid: 1, name };
    table.server.world.identities.insert(1, identity);
    table.server.world.active_sessions.insert(1, id);
    let opened = table.send(0, &npc(8, MAD_JACK, None), 2_000);
    let lobby = fish_option(&opened, DERBY_LOBBY_LABEL).expect("the gate lists its lobby");
    table.send(0, &npc(10, MAD_JACK, Some(lobby)), 2_100);
    table.send(0, &lobby_request(5, &[1, 0, DERBY_LOBBY, 2, 1, 16, 8, 14]), 3_000);
    let room = table.server.world.rooms.room_of(1).unwrap().id;
    table.send(0, &lobby_request(3, &[room, DERBY_LOBBY]), 3_200);
    room_routing::tick(&mut table.server, 8_300);
    tick_client(&mut table, 0, 8_350);
    table.drain(0, 8_350);
    table.send(0, &[13, 0], 8_400);
    table.send(0, &[10, 0], 8_401);
    let gateway = |table: &Table| table.server.sessions[&table.keys[0]].gateway.clone();
    assert!(gateway(&table).gameplay().unwrap().derby_active());

    room_routing::tick(&mut table.server, 8_500);
    assert!(table.server.world.tables.contains_key(&room), "inside the deadline");
    room_routing::tick(&mut table.server, 8_500 + 120_000);
    assert!(!table.server.world.tables.contains_key(&room), "the settled match is removed");
    assert!(table.server.world.rooms.room(room).is_none(), "and its room");

    tick_client(&mut table, 0, 129_000);
    let out = table.drain(0, 129_000);
    assert!(out.iter().any(|reply| reply.starts_with(&[39, 0, 19])), "the client's game goes");
    assert!(!gateway(&table).gameplay().unwrap().derby_active(), "the driver is out");
    assert_eq!(gateway(&table).hosted_table(), None);
}
