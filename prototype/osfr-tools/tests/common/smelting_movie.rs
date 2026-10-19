//! Original Smelting messages with synthetic reports, not native mouse-play proof.
#![allow(dead_code)]
use osfr_tools::{
    gateway::{Gateway, GatewayOutcome},
    smelting::payload_packet as authored_payload_packet,
};

pub fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut b = vec![5, 0, 1];
    b.extend((inner.len() as u32).to_le_bytes());
    b.extend(inner);
    b
}
pub fn send(g: &mut Gateway, inner: &[u8], now: u64) -> GatewayOutcome {
    g.handle_at(&wrap(inner), now).unwrap()
}
pub fn instance(g: &Gateway) -> i32 {
    g.snapshot()
        .gameplay
        .unwrap()
        .smelting
        .unwrap()
        .active
        .unwrap()
        .instance_id
}
pub fn payload_packet(instance: i32, text: &str) -> std::io::Result<Vec<u8>> {
    let mut native = authored_payload_packet(instance, text)?;
    native[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    Ok(native)
}
pub fn start(g: &mut Gateway, now: u64) {
    let mut b = vec![39, 0, 5];
    for n in [instance(g), -1, -1] {
        b.extend(n.to_le_bytes());
    }
    assert_eq!(send(g, &b, now).replies.len(), 2); // Native start + real movie launch.
    let out = send(
        g,
        &payload_packet(instance(g), "FRServer_ReadyForGameData").unwrap(),
        now,
    );
    assert_eq!(out.replies.len(), 6);
}
pub fn rounds(g: &mut Gateway, now: u64) {
    for stage in 101..=104 {
        for kind in 0..=3 {
            send(
                g,
                &payload_packet(
                    instance(g),
                    &format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)"),
                )
                .unwrap(),
                now,
            );
        }
    }
}
pub fn play(g: &mut Gateway, now: u64) {
    let previous = g.snapshot().gameplay.unwrap().smelting.unwrap().completed;
    start(g, now);
    rounds(g, now);
    send(
        g,
        &payload_packet(instance(g), "FRServer_GameWon\t900\t10").unwrap(),
        now,
    );
    assert_eq!(
        g.snapshot().gameplay.unwrap().smelting.unwrap().completed,
        previous + 1
    );
    send(g, &[26, 0, 42, 0], now);
    assert!(
        g.snapshot()
            .gameplay
            .unwrap()
            .smelting
            .unwrap()
            .active
            .is_none()
    );
}
