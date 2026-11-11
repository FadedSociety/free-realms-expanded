//! F3 (rc-0136 m-derby): a solo derby driven the way the client drives it. The car's pose (34/5)
//! comes direct (unreliable) every frame; the transport used to refuse it, so the match never had
//! the player's car: the AI hunted each other, and every hit the player reported failed the
//! fresh-pose check (0 damage on the tally). Here the pose takes the socket path
//! (`Session::handle` with a direct datagram), is held for the tick, and the derby then has the
//! player's car: the hunting AI close in, a reported hit lands on the tally, and every tick keeps
//! posing the AI.
#[path = "fishing_fixtures.rs"]
#[allow(dead_code)] // the shared fixtures module also ships builders this test does not use
mod fixtures;
#[path = "../../../tests/common/round70_content.rs"]
#[allow(dead_code)] // this test uses the paths, not the fixture builder
mod round70_content;
#[path = "../../../tests/support/derby_play.rs"]
mod derby_play;
use super::test_support::{self, Scratch, Table};
use super::*;
use derby_play::{countdown_for, hit_for, phase_for, register_for, start_request};
use fixtures::{fish_option, npc};
use osfr_tools::{
    character_store::CharacterState,
    derby::{DerbyMessage, DerbyPhase, ParticipantKey, VehiclePose, decode_inner, encode_inner},
    gameplay::Gameplay,
};

/// L096: the Wildwood Speedway turnstile's derby actor and where it stands.
const DERBY_GATE: u64 = 9500001021;
const TURNSTILE_AT: [f32; 3] = [-2268.808, -16.374, 586.806];
/// "Demo Derby": the solo row on the gate.
const DERBY_LABEL: u32 = 382_789;
const BRAWLER: i32 = 43;

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut app = vec![5, 0, 1];
    app.extend((inner.len() as u32).to_le_bytes());
    app.extend(inner);
    app
}

/// A direct datagram as the client sends one: opcode 5, the uncompressed flag, the tunnel, CRC.
fn direct(app: &[u8]) -> Vec<u8> {
    let mut wire = vec![app[0], 0];
    wire.extend(&app[1..]);
    let crc = osfr_tools::soe::seeded_crc32(CRC_SEED, &wire).to_be_bytes();
    wire.extend(&crc[1..]);
    wire
}

/// The car's pose at `at` with the throttle held (W), stamped `stamp`, as `009f0ce0` writes it.
fn pose(key: ParticipantKey, at: [f32; 3], stamp: u64) -> Vec<u8> {
    #[rustfmt::skip]
    let matrix = [1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, at[0], at[1], at[2], 1.0];
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: matrix.map(f32::to_bits),
        vector_a_bits: [0.0_f32, 0.0, 8.0, 1.0].map(f32::to_bits),
        vector_b_bits: [0.0_f32, 0.0, 0.0, 1.0].map(f32::to_bits),
        controls_bits: [1.0_f32, 0.0, 0.0].map(f32::to_bits),
        flags: [0; 4],
        sequence: stamp,
    }))
    .unwrap()
}

fn handle_wire(table: &mut Table, wire: &[u8], now: u64) {
    let key = table.keys[0];
    let server = &mut table.server;
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut session = server.sessions.remove(&key).unwrap();
    let result = session.handle(
        &socket,
        wire,
        &server.options,
        now,
        &mut server.world,
        &mut server.membership,
        &mut server.sessions,
    );
    server.sessions.insert(key, session);
    result.unwrap();
}

fn tick_client(table: &mut Table, now: u64) -> Vec<Vec<u8>> {
    let key = table.keys[0];
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
    table.drain(0, now)
}

fn gameplay(table: &Table) -> &Gameplay {
    table.server.sessions[&table.keys[0]].gateway.gameplay().unwrap()
}

fn messages(replies: &[Vec<u8>]) -> Vec<DerbyMessage> {
    (replies.iter())
        .filter(|reply| reply.starts_with(&[34, 0]))
        .filter_map(|reply| decode_inner(reply).ok())
        .collect()
}

fn gap(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

#[test]
fn a_direct_car_pose_reaches_the_match_so_the_ai_hunt_and_the_players_hits_count() {
    if !test_support::bin_test_target("f3") {
        return;
    }
    let scratch = Scratch::new("f3-derby-pose");
    let content = test_support::repo_and_runtime_content(&scratch, &round70_content::source());
    let mut character = Gateway::reference_state(&round70_content::reference()).unwrap();
    character.zone_id = 0;
    character.position = TURNSTILE_AT;
    let mut game = Gameplay::load(&content, None).unwrap();
    game.initialize_world(&character, 0).unwrap();
    let mut saved = game.state();
    let profiles = saved.profiles.as_mut().unwrap();
    profiles.unlocked.extend([BRAWLER]);
    profiles.active = BRAWLER;
    let players: Vec<(CharacterState, _)> = vec![(character.clone(), saved)];
    let Some(mut table) = Table::new(scratch, &content, &round70_content::reference(), &players)
    else {
        return;
    };
    let me = ParticipantKey(character.guid);

    // The gate's Demo Derby row, the arena's zoning, its Ready and Loaded, then the offer.
    let menu = table.send(0, &npc(8, DERBY_GATE, None), 1_000);
    let row = fish_option(&menu, DERBY_LABEL).expect("the turnstile offers the derby");
    let mut start = table.send(0, &npc(10, DERBY_GATE, Some(row)), 1_100);
    start.extend(table.send(0, &[13, 0], 1_200));
    start.extend(table.send(0, &[10, 0], 1_300));
    let offer = start.iter().find(|reply| reply.starts_with(&[39, 0, 16]));
    let instance = i32::from_le_bytes(offer.expect("the offer")[3..7].try_into().unwrap());
    table.send(0, &start_request(instance), 1_400);
    let setup = table.send(0, &register_for(me), 1_500);
    // The AI drivers' 35/2 name the car keys (L959: each driver goes before its car; the cars'
    // own 34/10 records are fragments this rig does not reassemble).
    let npcs: Vec<ParticipantKey> = (setup.iter())
        .filter(|reply| reply.starts_with(&[35, 0, 2, 0]))
        .map(|reply| ParticipantKey(u64::from_le_bytes(reply[4..12].try_into().unwrap())))
        .collect();
    assert_eq!(npcs.len(), 7, "seven AI cars");
    // L931: the live Wildwood start slot (`derby_shipped_package.rs`).
    let mut car = [373.99_f32, 6.0, 274.55];
    for phase in 3..=8 {
        table.send(0, &phase_for(me, phase), 1_600 + phase as u64);
    }
    for count in (0..=4).rev() {
        table.send(0, &countdown_for(me, count), 2_000 + (4 - count) as u64 * 1_000);
    }
    assert_eq!(gameplay(&table).derby_phase(), Some(DerbyPhase::Running));

    // The player's car (throttle held) arrives direct, three frames out of order; it waits for
    // the tick, which hands the newest to the match.
    let mut now = 7_000;
    tick_client(&mut table, now);
    assert_eq!(gameplay(&table).derby_driver_position(me), None, "no pose of the player's car yet");
    for stamp in [10, 30, 20] {
        let at = [car[0], car[1], car[2] + stamp as f32 / 10.0];
        handle_wire(&mut table, &direct(&wrap(&pose(me, at, stamp))), now + 5);
    }
    car[2] += 3.0;
    now += 50;
    let tick = tick_client(&mut table, now);
    assert_eq!(
        gameplay(&table).derby_driver_position(me),
        Some(car),
        "the newest direct pose reached the match"
    );
    let poses = |replies: &[Vec<u8>]| {
        (messages(replies).into_iter())
            .filter(|message| matches!(message, DerbyMessage::VehiclePose(pose) if pose.key != me))
            .count()
    };
    assert!(poses(&tick) > 0, "the tick poses the AI");
    let ai_at = |table: &Table, key| gameplay(table).derby_driver_position(key);

    // The four chasers close on the player's car over the next two seconds.
    let start_gaps: Vec<f32> = npcs[..4].iter().map(|key| gap(ai_at(&table, *key).unwrap(), car)).collect();
    for step in 1..=40_u64 {
        car[2] += 0.4;
        handle_wire(&mut table, &direct(&wrap(&pose(me, car, 100 + step))), now + 5);
        now += 50;
        let replies = tick_client(&mut table, now);
        assert!(poses(&replies) > 0, "tick {step}: the AI keep posing (no stall)");
    }
    let chased = npcs[..4]
        .iter()
        .zip(&start_gaps)
        .filter(|(key, from)| gap(ai_at(&table, **key).unwrap(), car) < **from - 5.0)
        .count();
    assert!(chased >= 2, "the hunting AI close in on the player's car ({chased} of 4)");

    // A hit the player reports on the nearest AI car lands: the tally counts it.
    let victim = *npcs
        .iter()
        .min_by(|a, b| gap(ai_at(&table, **a).unwrap(), car).total_cmp(&gap(ai_at(&table, **b).unwrap(), car)))
        .unwrap();
    car = ai_at(&table, victim).unwrap();
    car[0] += 2.0;
    handle_wire(&mut table, &direct(&wrap(&pose(me, car, 1_000))), now + 5);
    now += 50;
    tick_client(&mut table, now);
    let hit = table.send(0, &hit_for(me, victim, 2_000.0), now + 10);
    let tally = (messages(&hit).into_iter()).find_map(|message| match message {
        DerbyMessage::VehicleState(state) if state.key == me => Some(state.damage_dealt),
        _ => None,
    });
    assert!(tally.is_some_and(|dealt| dealt > 0), "the hit is on the player's tally: {tally:?}");
}
