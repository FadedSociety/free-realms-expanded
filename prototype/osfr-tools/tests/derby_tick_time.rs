//! F3 (rc-0136 m-derby, "server lag?"): what one session tick of a running Wildwood derby costs,
//! split the way the gateway binary pays it (`Session::tick`): the staged copy of the session's
//! `Gateway`, `Gateway::tick_at` (the derby's own staged `Gameplay` copy, the AI step and the
//! replies), and, on a tick whose outcome changed the save, the snapshot serialization the save
//! writes. The player's car posts a pose every tick, as the direct 34/5 does since F3.
//!
//! A measurement, not a gate: timings vary by machine and by the parallel suite, so it is
//! ignored by default. Run it on a release build:
//! `cargo test --release --test derby_tick_time -- --ignored --nocapture`.

#[path = "support/derby_play.rs"]
mod derby_play;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/soccer_play.rs"]
mod soccer_play;

use derby_play::{countdown_for, phase_for, register_for, start_request};
use npc_quest_chain::{
    Fixture, gateway_send, menu_option_bytes, npc_interact, npc_location, npc_select,
};
use osfr_tools::derby::{DerbyMessage, DerbyPhase, ParticipantKey, VehiclePose, encode_inner};
use soccer_play::offer_instance;
use std::time::{Duration, Instant};

const MAD_JACK: u64 = 9_000_000_391;
const DERBY_LABEL: u32 = 3_053;
const TICKS: u64 = 400;

fn pose(key: ParticipantKey, at: [f32; 3], stamp: u64) -> Vec<u8> {
    #[rustfmt::skip]
    let matrix = [1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, at[0], at[1], at[2], 1.0];
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: matrix.map(f32::to_bits),
        vector_a_bits: [0; 4],
        vector_b_bits: [0; 4],
        controls_bits: [1.0_f32.to_bits(), 0, 0],
        flags: [0; 4],
        sequence: stamp,
    }))
    .unwrap()
}

fn summary(name: &str, samples: &mut [Duration]) {
    samples.sort();
    let at = |q: f64| samples[((samples.len() - 1) as f64 * q) as usize].as_micros();
    let mean = samples.iter().sum::<Duration>().as_micros() / samples.len().max(1) as u128;
    println!(
        "F3 tick time {name}: n {} mean {mean} us, p50 {} us, p95 {} us, max {} us",
        samples.len(),
        at(0.5),
        at(0.95),
        at(1.0)
    );
}

#[test]
#[ignore = "a release-build measurement: cargo test --release --test derby_tick_time -- --ignored"]
fn a_running_derby_tick_costs_this_much() {
    let guid = 931_777;
    let fixture = Fixture::for_character(guid);
    for file in ["derby_entry.json", "derby_gates.json", "derby_rows.json", "derby_minigames.json"] {
        fixture.copy_content(file);
    }
    let (_, position) = npc_location(MAD_JACK);
    let mut gateway = fixture.session(guid, position, None);
    let menu = gateway_send(&mut gateway, &npc_interact(MAD_JACK), 10_000);
    let packet = menu.iter().find(|packet| packet.starts_with(&[26, 0, 9, 0])).unwrap();
    let option = menu_option_bytes(packet, DERBY_LABEL);
    let mut entered = gateway_send(&mut gateway, &npc_select(MAD_JACK, option), 10_001);
    entered.extend(gateway_send(&mut gateway, &13_u16.to_le_bytes(), 10_002));
    entered.extend(gateway_send(&mut gateway, &10_u16.to_le_bytes(), 10_003));
    let instance = offer_instance(&entered);
    let me = ParticipantKey(guid);
    gateway_send(&mut gateway, &start_request(instance), 10_300);
    gateway_send(&mut gateway, &register_for(me), 10_400);
    for phase in 3..=8 {
        gateway_send(&mut gateway, &phase_for(me, phase), 11_000 + phase as u64);
    }
    for count in (0..=4).rev() {
        gateway_send(&mut gateway, &countdown_for(me, count), 12_000 + (4 - count) as u64);
    }
    assert_eq!(gateway.gameplay().unwrap().derby_phase(), Some(DerbyPhase::Running));

    let (mut clone, mut tick, mut save) = (Vec::new(), Vec::new(), Vec::new());
    let mut now = 13_000;
    let mut car = [373.99_f32, 6.0, 274.55];
    for step in 0..TICKS {
        car[2] += 0.3;
        gateway_send(&mut gateway, &pose(me, car, step + 1), now);
        now += 50;
        let started = Instant::now();
        let mut staged = gateway.clone();
        clone.push(started.elapsed());
        let started = Instant::now();
        let outcome = staged.tick_at(now).unwrap();
        tick.push(started.elapsed());
        if outcome.gameplay_changed || outcome.state_changed {
            let started = Instant::now();
            let bytes = serde_json::to_vec(&staged.snapshot()).unwrap();
            save.push(started.elapsed());
            assert!(!bytes.is_empty());
        }
        gateway = staged;
    }
    assert_eq!(gateway.gameplay().unwrap().derby_phase(), Some(DerbyPhase::Running));
    summary("gateway clone", &mut clone);
    summary("tick_at", &mut tick);
    if !save.is_empty() {
        summary("save serialize (ticks that changed the save)", &mut save);
    }
    println!("F3 tick time: {} of {TICKS} ticks changed the save", save.len());
}
