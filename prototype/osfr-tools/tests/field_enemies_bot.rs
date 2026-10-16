//! Private W105 route gate. Python helper owns the scratch gateway lifecycle.
#[allow(dead_code)]
#[path = "../src/bin/osfr-bot.rs"]
mod bot;

use bot::{Bot, BotConfig, Step};
use std::{
    fs,
    time::{Duration, Instant},
};

#[test]
#[ignore = "requires W105_GATEWAY and W105_REFERENCE from scratch_gate.py"]
fn field_enemy_leashes_on_the_wire() {
    let gateway: std::net::SocketAddr = std::env::var("W105_GATEWAY").unwrap().parse().unwrap();
    assert!(
        gateway.ip().is_loopback(),
        "scratch gate must stay loopback-only"
    );
    let reference = std::env::var("W105_REFERENCE").unwrap();
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(std::path::Path::new(&reference).join("manifest.json")).unwrap(),
    )
    .unwrap();
    let config = BotConfig {
        gateway,
        name: String::new(),
        connection_id: std::process::id(),
        guid: manifest["initial_state"]["guid"].as_u64().unwrap(),
        ticket: osfr_tools::login::DEFAULT_GATEWAY_TICKET.to_owned(),
        version: manifest["client_version"].as_str().unwrap().to_owned(),
        expect_timeout_ms: 5000,
        quiet: true,
    };
    let mut b = Bot::connect(config).unwrap();
    b.login().unwrap();
    for command in [
        "realm god",
        "realm reload behaviours",
        "realm reload enemies",
        "realm goto -1899.204 313.902",
    ] {
        b.run_step(&Step::Chat(command.into())).unwrap();
        b.run_step(&Step::Expect {
            family: 15,
            sub: Some(3),
            timeout_ms: None,
        })
        .unwrap();
    }
    const GUID: u64 = 131000102001;
    let home = [-1899.204_f32, -50.59, 307.902];
    let deadline = Instant::now() + Duration::from_secs(25);
    let mut max_radius = 0.0_f32;
    let mut returning = false;
    let mut returned = false;
    let mut previous = 0.0_f32;
    let mut count = 0;
    let start = Instant::now();
    while Instant::now() < deadline {
        let packet = b
            .wait_for(Duration::from_secs(5), |p| {
                p.len() >= 22 && p.starts_with(&[125, 0]) && p[2..10] == GUID.to_le_bytes()
            })
            .unwrap();
        let f = |at| f32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
        let at = [f(10), f(14), f(18)];
        let radius = (at[0] - home[0]).hypot(at[2] - home[2]);
        println!(
            "received 0x007d guid={GUID} t={:.3} position={at:?} home_distance={radius:.4}",
            start.elapsed().as_secs_f32()
        );
        assert!(radius <= 12.01, "field actor crossed home leash");
        max_radius = max_radius.max(radius);
        count += 1;
        if max_radius > 10.0 && radius < previous - 0.05 {
            returning = true;
        }
        if returning && radius < 0.05 {
            returned = true;
            break;
        }
        previous = radius;
        // Chase with a player always 6 m ahead: give-up by target distance
        // cannot satisfy this gate. While returning this also tests re-aggro.
        b.run_step(&Step::Chat(format!("realm goto {} {}", at[0], at[2] + 6.0)))
            .unwrap();
        b.run_step(&Step::Wait(120)).unwrap();
    }
    b.disconnect();
    assert!(
        max_radius > 10.0 && returning && returned,
        "max={max_radius}, return={returning}, home={returned}"
    );
    println!(
        "PASS: {count} received movement updates; max home distance {max_radius:.4} m; returned home; Studio disabled"
    );
}
