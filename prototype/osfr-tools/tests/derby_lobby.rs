//! The Demo Derby Lobby: several real players in one match. The gate's lobby option, the room's
//! options against the captured catalogue, and a match walked from the countdown to each driver's
//! own results, payout and way back, with one small stand-in for the gateway binary's room table
//! (`bin/osfr_gateway/room_routing.rs`): lend the match around one message, then relay.
#[path = "support/derby_play.rs"]
mod derby_play;

use derby_play::{countdown_for, hit_for, phase_for, register_for, start_request};
use osfr_tools::{
    character_store::CharacterState,
    derby::{DerbyMessage, DerbyPhase, ParticipantKey, VehiclePose, decode_inner, encode_inner},
    derby_gateway::DerbyGateway,
    game_room::{Limits, Phase, Rooms, Seat},
    gateway::{Gateway, GatewayOutcome},
    hosted_table::HostedTable,
    lobby_wire,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CRATE: &str = env!("CARGO_MANIFEST_DIR");
const DERBY_GATE: u64 = 9_500_001_021;
const LOBBY: u32 = 2;
const LOBBY_LABEL: u32 = 3053;
const SOLO_LABEL: u32 = 382_789;
const COMPLETION_COINS: u32 = 25;
/// Max Players 3, Difficulty, Wildwood, Enable AI yes, Total Time 120 s, Auto-Start no.
const OPTIONS: [(u32, u32); 7] = [(1, 16), (2, 4), (6, 10), (7, 9), (8, 14), (10, 19), (13, 15)];
/// The same room with Enable AI off and two seats.
const NO_AI: [(u32, u32); 4] = [(1, 1), (6, 10), (8, 15), (10, 19)];
static NEXT: AtomicU64 = AtomicU64::new(0);

fn data() -> PathBuf {
    Path::new(CRATE).join("data/content")
}

fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_TYPE8_CONTENT").map(PathBuf::from).unwrap_or_else(|| {
        Path::new(CRATE).join("../../../lab/references/opening-journal-text-v2/content")
    })
}

fn reference() -> PathBuf {
    std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(CRATE).join("../../../lab/references/rust-gateway-reference"))
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// A scratch gameplay package: the reference world plus the shipped derby files and its lobby.
struct Shipped(PathBuf);

impl Shipped {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-derby-lobby-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&reference_content(), &path);
        for file in [
            "derby_gates.json",
            "derby_entry.json",
            "derby_rows.json",
            "derby_minigames.json",
            "derby_lobby.json",
        ] {
            fs::copy(data().join(file), path.join(file)).unwrap();
        }
        Self(path)
    }
}

impl Drop for Shipped {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn npc_position(id: u64) -> [f32; 3] {
    for entry in fs::read_dir(data().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let document = read_json(&path);
        let rows = document["records"].as_array().unwrap();
        if let Some(row) = rows.iter().find(|row| row["npc_id"].as_u64() == Some(id)) {
            return serde_json::from_value(row["position"].clone()).unwrap();
        }
    }
    panic!("NPC {id} content record");
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut packet = vec![5, 0, 1];
    packet.extend((inner.len() as u32).to_le_bytes());
    packet.extend(inner);
    packet
}

fn login(gateway: &mut Gateway) {
    let guid = gateway.state().guid;
    let mut packet = 1u16.to_le_bytes().to_vec();
    for (index, value) in ["derby-lobby-test", "1.910.1.530630", ""].iter().enumerate() {
        packet.extend((value.len() as u32).to_le_bytes());
        packet.extend(value.as_bytes());
        if index == 0 {
            packet.extend(guid.to_le_bytes());
        }
    }
    gateway.handle_at(&packet, 0).unwrap();
    for packet in [wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway.handle_at(&packet, 1_000).unwrap();
    }
}

/// A logged-in player standing at the derby gate.
fn driver(content: &Shipped, guid: u64) -> Gateway {
    let mut character: CharacterState = Gateway::reference_state(&reference()).unwrap();
    character.guid = guid;
    character.zone_id = 0;
    character.position = npc_position(DERBY_GATE);
    let mut gateway = Gateway::load_gameplay(&reference(), character, &content.0, None).unwrap();
    login(&mut gateway);
    gateway
}

fn bodies(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    let inner = |packet: &Vec<u8>| {
        assert!(packet.len() >= 7 && packet[..3] == [5, 0, 1], "a tunneled application packet");
        packet[7..].to_vec()
    };
    outcome.replies.iter().map(inner).collect()
}

fn npc_click(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

/// The option id whose 26/9 row carries `label`, from the client menu reader shape.
fn menu_option(menu: &[Vec<u8>], label: u32) -> Option<i32> {
    let packet = menu.iter().find(|packet| packet.starts_with(&[26, 0, 9, 0]))?;
    let count = i32::from_le_bytes(packet[13..17].try_into().ok()?) as usize;
    (0..count).find_map(|index| {
        let at = 17 + index * 28;
        let id = i32::from_le_bytes(packet[at..at + 4].try_into().ok()?);
        let row_label = u32::from_le_bytes(packet[at + 8..at + 12].try_into().ok()?);
        (row_label == label).then_some(id)
    })
}

fn pose(key: ParticipantKey, position: [f32; 3]) -> Vec<u8> {
    let mut transform = [0.0_f32; 16];
    (transform[0], transform[5], transform[10], transform[15]) = (1.0, 1.0, 1.0, 1.0);
    transform[12..15].copy_from_slice(&position);
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: transform.map(f32::to_bits),
        vector_a_bits: [0; 4],
        vector_b_bits: [0; 4],
        controls_bits: [0; 3],
        flags: [0; 4],
        sequence: 1,
    }))
    .unwrap()
}

/// The room table of the gateway binary, cut to what a match needs: the World's copy of the
/// match, and what each driver is owed by the others' messages.
#[derive(Default)]
struct World {
    table: Option<HostedTable>,
    inbox: BTreeMap<u64, Vec<Vec<u8>>>,
}

impl World {
    /// One transaction of `gateway`, as `Session::dispatch_message` and `Session::tick` run it:
    /// lend, act, reclaim, publish. Returns the driver's own replies.
    fn run(
        &mut self,
        gateway: &mut Gateway,
        act: impl FnOnce(&mut Gateway) -> GatewayOutcome,
    ) -> Vec<Vec<u8>> {
        match (gateway.hosted_table(), &self.table) {
            (Some(_), Some(table)) => gateway.lend_table(table.clone()),
            (Some(_), None) => gateway.orphan_table(),
            _ => {}
        }
        let outcome = act(gateway);
        if let Some((mut table, mut relay)) = gateway.reclaim_table() {
            let guid = gateway.state().guid;
            if gateway.hosted_table().is_none() {
                let owed = table.forfeit(guid).unwrap();
                relay.extend(owed.into_iter().filter(|message| message.player_id != guid));
                table.release(guid);
            }
            for message in relay {
                self.inbox.entry(message.player_id).or_default().push(message.packet);
            }
            self.table = (!table.all_settled()).then_some(table);
        }
        bodies(&outcome)
    }

    fn send(&mut self, gateway: &mut Gateway, inner: &[u8], now: u64) -> Vec<Vec<u8>> {
        self.run(gateway, |gateway| gateway.handle_at(&wrap(inner), now).unwrap())
    }

    fn tick(&mut self, gateway: &mut Gateway, now: u64) -> Vec<Vec<u8>> {
        self.run(gateway, |gateway| gateway.tick_at(now).unwrap())
    }

    /// The room's own tick (`room_routing::tick`).
    fn watch(&mut self, now: u64) {
        let Some(table) = self.table.as_mut() else {
            return;
        };
        for message in table.watch(now).unwrap() {
            self.inbox.entry(message.player_id).or_default().push(message.packet);
        }
        // As `room_routing::watch`: a match its watch settled has nobody left to publish its end.
        if table.all_settled() {
            self.table = None;
        }
    }

    fn heard(&mut self, gateway: &Gateway) -> Vec<Vec<u8>> {
        self.inbox.remove(&gateway.state().guid).unwrap_or_default()
    }

    fn phase(&self) -> Option<DerbyPhase> {
        match self.table.as_ref()? {
            HostedTable::Derby(derby) => Some(derby.service().phase()),
            _ => None,
        }
    }
}

fn seat(gateway: &Gateway, name: &str) -> Seat {
    Seat { guid: gateway.state().guid, first: name.to_owned(), last: String::new() }
}

/// Seat `drivers` in a room with `options`, run its countdown out, and open its match as
/// `room_routing::open_derby` does. The first driver hosts.
fn open_room(drivers: &mut [&mut Gateway], options: &[(u32, u32)]) -> (World, Rooms, u32) {
    let mut rooms = Rooms::default();
    let limits = drivers[0].lobby_limits(LOBBY, options).unwrap().expect("the derby's lobby");
    rooms.create_with(LOBBY, limits, seat(drivers[0], "Host"), options.to_vec()).unwrap();
    let id = rooms.room_of(drivers[0].state().guid).unwrap().id;
    for guest in &drivers[1..] {
        rooms.join(id, seat(guest, "Guest")).unwrap();
    }
    let started = rooms.start(drivers[0].state().guid, 1_000).unwrap();
    assert!(started.iter().all(|packet| packet.bytes == lobby_wire::starting(id, LOBBY)));
    assert_eq!(rooms.due(6_000), [id]);
    let names = ["Host One", "Guest Two", "Guest Three"];
    let seats: Vec<_> = (drivers.iter().zip(names))
        .map(|(driver, name)| driver.derby_seat(name.to_owned()).unwrap())
        .collect();
    let table = drivers[0].host_derby(id, options, &seats).unwrap();
    for driver in drivers.iter_mut() {
        driver.sit_derby(&table);
    }
    let world = World { table: Some(HostedTable::Derby(Box::new(table))), ..World::default() };
    (world, rooms, id)
}

/// A driver's first tick after the countdown and their arena's load: everything they were sent.
fn enter(world: &mut World, gateway: &mut Gateway, now: u64) -> Vec<Vec<u8>> {
    let mut replies = world.tick(gateway, now);
    for (step, packet) in [[13u8, 0], [10, 0]].iter().enumerate() {
        replies.extend(world.send(gateway, packet, now + 1 + step as u64));
    }
    replies
}

fn key(gateway: &Gateway) -> ParticipantKey {
    ParticipantKey(gateway.state().guid)
}

fn count(packets: &[Vec<u8>], family: u8, sub: u8) -> usize {
    packets.iter().filter(|packet| packet.starts_with(&[family, 0, sub])).count()
}

fn creates(packets: &[Vec<u8>]) -> Vec<osfr_tools::derby::VehicleCreate> {
    let create = |packet: &Vec<u8>| match decode_inner(packet).ok()? {
        DerbyMessage::CreateVehicle(vehicle) => Some(vehicle),
        _ => None,
    };
    packets.iter().filter(|packet| packet.starts_with(&[34, 0])).filter_map(create).collect()
}

/// Both drivers in, registered, through the six state gates and the countdown: the race runs.
fn race(world: &mut World, a: &mut Gateway, b: &mut Gateway) -> u64 {
    let (ka, kb) = (key(a), key(b));
    enter(world, a, 6_050);
    enter(world, b, 6_050);
    world.send(a, &register_for(ka), 7_000);
    world.send(b, &register_for(kb), 7_000);
    for phase in 3..=8 {
        world.send(a, &phase_for(ka, phase), 7_100);
        world.send(b, &phase_for(kb, phase), 7_100);
    }
    for count in (0..=4).rev() {
        world.send(a, &countdown_for(ka, count), 8_000);
        world.send(b, &countdown_for(kb, count), 8_000);
    }
    assert_eq!(world.phase(), Some(DerbyPhase::Running));
    world.heard(a);
    world.heard(b);
    8_000
}

fn coins(gateway: &Gateway) -> u32 {
    gateway.snapshot().gameplay.unwrap().inventory.coins
}

fn paid(gateway: &Gateway) -> Vec<i32> {
    let state = gateway.snapshot().gameplay.unwrap();
    state.derby.map(|derby| derby.paid_matches).unwrap_or_default()
}

/// `derby_lobby.json` reads the captured catalogue's own values: every option value it names is a
/// value of that setting for lobby type 2, and the numbers are the catalogue's display texts.
#[test]
fn the_lobby_options_are_the_captured_catalogues() {
    let catalogue = read_json(&data().join("kart_lobby.json"));
    let lobby = read_json(&data().join("derby_lobby.json"));
    let rows = |table: &str| catalogue[table].as_array().unwrap().clone();
    let offered = |setting: u64| -> Vec<u64> {
        let of = |row: &Value| (row["unknown"][0].as_u64() == Some(setting)).then(|| row["unknown"][1].as_u64());
        rows("values").iter().filter_map(of).flatten().collect()
    };
    let text = |key: u64| -> u64 {
        let row = rows("display").into_iter().find(|row| row["key"].as_u64() == Some(key)).unwrap();
        let bytes: Vec<u8> = serde_json::from_value(row["text"].clone()).unwrap();
        String::from_utf8(bytes).unwrap().parse().unwrap()
    };
    let derby_settings: Vec<u64> = (rows("options").iter())
        .filter(|row| row["unknown"][0].as_u64() == Some(u64::from(LOBBY)))
        .filter_map(|row| row["unknown"][1].as_u64())
        .collect();
    for (setting, map, numbers) in [
        ("max_players_setting", "max_players", true),
        ("total_time_setting", "total_time_s", true),
        ("track_setting", "tracks", false),
        ("difficulty_setting", "difficulty_speed", false),
    ] {
        let setting = lobby[setting].as_u64().unwrap();
        assert!(derby_settings.contains(&setting), "setting {setting} is the derby lobby's");
        let mut keys = Vec::new();
        for (key, value) in lobby[map].as_object().unwrap() {
            let key: u64 = key.parse().unwrap();
            keys.push(key);
            if numbers {
                assert_eq!(value.as_u64(), Some(text(key)), "{map} {key}");
            }
        }
        keys.sort_unstable();
        let mut listed = offered(setting);
        listed.sort_unstable();
        assert_eq!(keys, listed, "{map} names every value of setting {setting}");
    }
    for switch in ["ai_setting", "auto_start_setting"] {
        let setting = lobby[switch].as_u64().unwrap();
        assert!(derby_settings.contains(&setting));
        assert!(offered(setting).contains(&lobby["yes"].as_u64().unwrap()));
    }
    assert_eq!(text(lobby["yes"].as_u64().unwrap()), 1, "Yes is the catalogue's 1");

    // Every track is a hosted free-play row, and every room the panel can build is a match.
    let derby = DerbyGateway::load(&data(), None).unwrap().unwrap();
    for (seats, time, track) in [(1, 19, 10), (16, 20, 11), (18, 21, 8)] {
        let options = [(1, seats), (6, track), (8, 14), (10, time), (13, 15)];
        let limits = derby.lobby_limits(LOBBY, &options).unwrap().unwrap();
        assert_eq!((limits.min_seats, limits.auto_start), (1, false));
    }
    let short = derby.lobby_limits(LOBBY, &NO_AI).unwrap().unwrap();
    assert_eq!(short, Limits { capacity: 2, min_seats: 2, auto_start: false });
    assert!(derby.lobby_limits(LOBBY, &[(1, 99)]).is_err(), "a value the catalogue lacks");
    assert!(derby.lobby_limits(LOBBY, &[(6, 7)]).is_err(), "a kart track is no derby arena");
    assert_eq!(derby.lobby_limits(4, &OPTIONS), Ok(None), "chess keeps its two seats");
    let goal_free = |id: i32| derby.rows().iter().any(|row| row.game_id() == id && row.package().goal.is_none());
    assert!([39, 40, 41].into_iter().all(goal_free), "lobby tracks are free play");
}

/// The gate lists the solo derby and, under it, the lobby; the lobby option opens the list and
/// starts nothing, and the solo option still starts its match.
#[test]
fn the_gate_lists_the_lobby_under_the_solo_derby() {
    let content = Shipped::new();
    let mut gateway = driver(&content, 7_001);
    let menu = bodies(&gateway.handle_at(&wrap(&npc_click(DERBY_GATE)), 2_000).unwrap());
    let solo = menu_option(&menu, SOLO_LABEL).expect("the solo derby");
    let lobby = menu_option(&menu, LOBBY_LABEL).expect("the Demo Derby Lobby");
    assert!(solo < lobby, "the lobby comes after the match");

    let chosen = gateway.handle_at(&wrap(&npc_select(DERBY_GATE, lobby)), 2_100).unwrap();
    assert_eq!(chosen.open_lobby, Some(LOBBY));
    assert!(!gateway.gameplay().unwrap().derby_active(), "a list is not a match");

    gateway.handle_at(&wrap(&npc_click(DERBY_GATE)), 3_000).unwrap();
    let entered = gateway.handle_at(&wrap(&npc_select(DERBY_GATE, solo)), 3_100).unwrap();
    assert!(entered.gameplay_changed && gateway.gameplay().unwrap().derby_active());
}

/// Two drivers and one AI car in a three-seat room: each driver gets their own start records,
/// both clients are told of all three cars in their own start slots, poses and hits pass between
/// them, a knock-out is scored once for both, and at the clock each is paid once, on their own
/// save, and closes their own way out.
#[test]
fn two_drivers_share_one_match_and_each_is_paid_once() {
    let content = Shipped::new();
    let (mut a, mut b) = (driver(&content, 7_001), driver(&content, 7_002));
    let (ka, kb) = (key(&a), key(&b));
    let before = (coins(&a), coins(&b));
    let (mut world, _rooms, id) = open_room(&mut [&mut a, &mut b], &OPTIONS);

    // Each driver's first tick: the job, the arena, then the hidden offer, its ready and the Go.
    let started = enter(&mut world, &mut a, 6_050);
    let offer = started.iter().find(|packet| packet.starts_with(&[39, 0, 16])).expect("the offer");
    let instance = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    assert_eq!(offer[offer.len() - 9], 1, "the offer window stays shut");
    assert_eq!(count(&started, 39, 17), 1, "the countdown was the Go");
    assert_eq!(count(&started, 41, 107), 1);
    assert!(count(&enter(&mut world, &mut a, 6_500), 39, 16) == 0, "once");
    let guest = enter(&mut world, &mut b, 6_050);
    let theirs = guest.iter().find(|packet| packet.starts_with(&[39, 0, 16])).unwrap();
    assert_eq!(theirs[3..7], offer[3..7], "one match, one instance");
    // The client's own Go after the server's changes nothing.
    assert_eq!(count(&world.send(&mut a, &start_request(instance), 6_600), 39, 17), 0);

    // The cars are made when the last driver has registered, for both clients alike.
    assert!(creates(&world.send(&mut a, &register_for(ka), 7_000)).is_empty());
    let made = world.send(&mut b, &register_for(kb), 7_000);
    let cars = creates(&made);
    assert_eq!(cars, creates(&world.heard(&a)), "both clients hear the same setup");
    assert_eq!(cars.len(), 3, "two drivers and the AI car that fills the third seat");
    let named = |name: &str| cars.iter().find(|car| car.name == name.as_bytes()).unwrap();
    assert_eq!(named("Host One").key, ka);
    assert_eq!(named("Guest Two").key, kb);
    let mut slots: Vec<_> = cars.iter().map(|car| car.start_transform_bits).collect();
    slots.sort_unstable();
    slots.dedup();
    assert_eq!(slots.len(), 3, "no two cars start in one slot");

    // Each gate waits for both drivers.
    for phase in 3..=8 {
        assert_eq!(count(&world.send(&mut a, &phase_for(ka, phase), 7_100), 34, 1), 0);
        assert_eq!(count(&world.send(&mut b, &phase_for(kb, phase), 7_100), 34, 1), usize::from(phase < 8));
    }
    for count_down in (0..=4).rev() {
        world.send(&mut a, &countdown_for(ka, count_down), 8_000);
        world.send(&mut b, &countdown_for(kb, count_down), 8_000);
    }
    assert_eq!(world.phase(), Some(DerbyPhase::Running));
    world.heard(&a);
    world.heard(&b);

    // A pose goes to the other driver only; a hit lands on the other human's own car.
    assert_eq!(count(&world.send(&mut a, &pose(ka, [10.0, 0.0, 10.0]), 9_000), 34, 5), 0);
    assert_eq!(count(&world.heard(&b), 34, 5), 1);
    world.send(&mut b, &pose(kb, [11.0, 0.0, 10.0]), 9_000);
    world.heard(&a);
    world.send(&mut a, &hit_for(ka, kb, 6_000.0), 9_100);
    world.send(&mut a, &hit_for(ka, kb, 6_000.0), 9_200);
    world.send(&mut a, &hit_for(ka, kb, 6_000.0), 9_700);
    let HostedTable::Derby(derby) = world.table.as_ref().unwrap() else {
        panic!("a derby table");
    };
    let (host, hit) = (derby.service().driver(ka).unwrap(), derby.service().driver(kb).unwrap());
    assert_eq!((host.knockouts_made, hit.knockouts_taken, hit.health), (1, 1, 0), "one knock-out, the cooldown held");
    assert!(count(&world.heard(&b), 34, 20) >= 1, "the knocked-out driver hears the announcer");

    // The host's two minutes: the results reach both, and each driver's own tick pays them.
    let end = 8_000 + 120_000;
    let results = world.tick(&mut a, end);
    assert!(results.iter().any(|packet| packet[..] == encode_inner(&DerbyMessage::GameState(10)).unwrap()[..]));
    assert_eq!(world.phase(), Some(DerbyPhase::Results));
    world.tick(&mut b, end);
    world.tick(&mut a, end + 50);
    world.tick(&mut b, end + 50);
    assert_eq!((coins(&a), coins(&b)), (before.0 + COMPLETION_COINS, before.1 + COMPLETION_COINS));
    assert_eq!((paid(&a), paid(&b)), (vec![instance], vec![instance]));

    // The Close opens for both; each closes alone, hears the lobby drop the game, and goes back.
    world.tick(&mut a, end + 10_000);
    assert_eq!(world.phase(), Some(DerbyPhase::CloseEnabled));
    world.tick(&mut b, end + 10_000);
    let closed = world.send(&mut a, &[26, 0, 42, 0], end + 11_000);
    assert!(closed.contains(&lobby_wire::left(id, LOBBY)));
    assert!(!a.gameplay().unwrap().derby_active());
    assert!(world.table.is_some(), "the other driver is still on the board");
    assert!(b.gameplay().unwrap().derby_active());
    world.send(&mut b, &[26, 0, 42, 0], end + 12_000);
    assert!(world.table.is_none(), "the match closes with its last driver");
    assert_eq!((coins(&a), coins(&b)), (before.0 + COMPLETION_COINS, before.1 + COMPLETION_COINS));

    // The next match of either driver takes a fresh id, so the ledger can never refuse it.
    let (world, ..) = open_room(&mut [&mut b, &mut a], &OPTIONS);
    let HostedTable::Derby(next) = world.table.as_ref().unwrap() else {
        panic!("a derby table");
    };
    assert!(next.instance_id() > instance);
}

/// A driver who quits mid-race takes only their own car: they get the results screen unpaid, the
/// other drives on, is told the car is gone, and is still paid at the clock.
#[test]
fn a_driver_who_quits_leaves_the_other_racing() {
    let content = Shipped::new();
    let (mut a, mut b) = (driver(&content, 7_001), driver(&content, 7_002));
    let (ka, kb) = (key(&a), key(&b));
    let before = (coins(&a), coins(&b));
    let (mut world, _rooms, id) = open_room(&mut [&mut a, &mut b], &OPTIONS);
    let started = race(&mut world, &mut a, &mut b);
    let HostedTable::Derby(derby) = world.table.as_ref().unwrap() else {
        panic!("a derby table");
    };
    let instance = derby.instance_id();

    let mut quit = vec![39, 0, 6];
    for word in [instance, -1, -1] {
        quit.extend(word.to_le_bytes());
    }
    let own = world.send(&mut b, &quit, started + 5_000);
    assert!(own.iter().any(|packet| packet[..] == encode_inner(&DerbyMessage::GameState(10)).unwrap()[..]));
    assert_eq!(world.phase(), Some(DerbyPhase::Running), "the race goes on");
    let told = world.heard(&a);
    assert_eq!(count(&told, 34, 8), 1, "the quitter's car is taken away");
    assert!(b.gameplay().unwrap().derby_active(), "their own results still hold");

    // The quitter's results run out on their own gateway, and their Close pays nothing.
    world.tick(&mut b, started + 16_000);
    let closed = world.send(&mut b, &[26, 0, 42, 0], started + 17_000);
    assert!(closed.contains(&lobby_wire::left(id, LOBBY)));
    assert!(!b.gameplay().unwrap().derby_active());
    assert_eq!((coins(&b), paid(&b)), (before.1, Vec::new()));

    let end = started + 120_000;
    world.tick(&mut a, end);
    world.tick(&mut a, end + 50);
    assert_eq!((coins(&a), paid(&a)), (before.0 + COMPLETION_COINS, vec![instance]));
}

/// A driver whose client never loads does not hold the start for ever: after the room's timeout
/// the match goes on without them, and their own next tick takes them out of it.
#[test]
fn a_start_goes_on_without_a_driver_who_never_loads() {
    let content = Shipped::new();
    let (mut a, mut b) = (driver(&content, 7_001), driver(&content, 7_002));
    let (ka, kb) = (key(&a), key(&b));
    let (mut world, ..) = open_room(&mut [&mut a, &mut b], &NO_AI);
    enter(&mut world, &mut a, 6_050);
    world.watch(6_100);
    assert!(creates(&world.send(&mut a, &register_for(ka), 7_000)).is_empty(), "waits for the guest");
    world.watch(60_000);
    assert_eq!(world.phase(), Some(DerbyPhase::Starting));
    world.watch(6_100 + 120_000);
    assert_eq!(world.phase(), Some(DerbyPhase::WaitingForState(3)));
    assert_eq!(creates(&world.heard(&a)).len(), 1, "the host's car alone, with AI off");

    let out = world.tick(&mut b, 130_000);
    assert!(!b.gameplay().unwrap().derby_active(), "the late driver is out");
    assert_eq!(count(&out, 39, 16), 0, "and never gets the offer");
    assert_eq!(world.phase(), Some(DerbyPhase::WaitingForState(3)));
}

/// The start has one deadline, not one per step: a driver who answers each step late cannot hold
/// the others past it.
#[test]
fn a_slow_driver_can_not_restart_the_start_clock() {
    let content = Shipped::new();
    let (mut a, mut b) = (driver(&content, 7_001), driver(&content, 7_002));
    let (ka, kb) = (key(&a), key(&b));
    let (mut world, ..) = open_room(&mut [&mut a, &mut b], &NO_AI);
    enter(&mut world, &mut a, 6_050);
    enter(&mut world, &mut b, 6_050);
    world.watch(6_100);
    // Both register late, then the guest stalls on the first state gate.
    world.send(&mut a, &register_for(ka), 100_000);
    world.send(&mut b, &register_for(kb), 100_000);
    world.watch(100_050);
    world.send(&mut a, &phase_for(ka, 3), 100_100);
    assert_eq!(world.phase(), Some(DerbyPhase::WaitingForState(3)));
    world.watch(6_100 + 119_000);
    assert_eq!(world.phase(), Some(DerbyPhase::WaitingForState(3)), "inside the deadline");
    world.watch(6_100 + 120_000);
    assert_eq!(world.phase(), Some(DerbyPhase::WaitingForState(4)), "cut 120 s after the start began");
    assert_eq!(count(&world.heard(&a), 34, 8), 1, "the stalled car goes");
    world.tick(&mut b, 127_000);
    assert!(!b.gameplay().unwrap().derby_active());
}

/// The only driver of a room never registers (a client the unsolicited Go does not move): the
/// start times out on them, the match and its table go, and their own next tick takes them out
/// of the arena, the game and the seat, not a relog.
#[test]
fn a_start_that_times_out_on_its_last_driver_frees_them() {
    let content = Shipped::new();
    let mut a = driver(&content, 7_001);
    let (mut world, _rooms, id) = open_room(&mut [&mut a], &OPTIONS);
    let started = enter(&mut world, &mut a, 6_050);
    assert_eq!(count(&started, 39, 16), 1);
    world.watch(6_100);
    world.watch(6_100 + 120_000);
    assert!(world.table.is_none(), "the settled match leaves the World");
    assert!(a.gameplay().unwrap().derby_active(), "the seat is still theirs to leave");

    let out = world.tick(&mut a, 127_000);
    assert_eq!(count(&out, 39, 19), 1, "the client's game is taken away");
    assert!(out.contains(&lobby_wire::left(id, LOBBY)), "and its lobby drops the game");
    assert!(!a.gameplay().unwrap().derby_active());
    assert_eq!(a.hosted_table(), None);
    assert!(world.tick(&mut a, 128_000).iter().all(|packet| !packet.starts_with(&[39, 0])), "once");
}

/// A driver who leaves the world mid-race (`SharedWorld::leave_rooms`) gives their car up, and the
/// last one out ends the match.
#[test]
fn a_driver_who_drops_gives_their_car_up_and_the_last_ends_the_match() {
    let content = Shipped::new();
    let (mut a, mut b) = (driver(&content, 7_001), driver(&content, 7_002));
    let (ka, kb) = (key(&a), key(&b));
    let (mut world, mut rooms, id) = open_room(&mut [&mut a, &mut b], &OPTIONS);
    race(&mut world, &mut a, &mut b);
    assert_eq!(rooms.room(id).unwrap().phase, Phase::Running);

    let table = world.table.as_mut().unwrap();
    let owed = table.forfeit(kb.raw()).unwrap();
    table.release(kb.raw());
    assert!(owed.iter().any(|message| message.player_id == ka.raw() && message.packet.starts_with(&[34, 0, 8])));
    assert!(!table.seated(kb.raw()) && table.seated(ka.raw()));
    assert!(!table.all_settled());
    rooms.leave(kb.raw()).unwrap();

    table.forfeit(ka.raw()).unwrap();
    table.release(ka.raw());
    assert!(table.all_settled());
    rooms.leave(ka.raw()).unwrap();
    assert!(rooms.room(id).is_none(), "the room goes with its last seat");
}
