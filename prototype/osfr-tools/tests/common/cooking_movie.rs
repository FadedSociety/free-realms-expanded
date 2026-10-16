//! Original Cooking messages with synthetic reports, not native mouse-play proof.
#![allow(dead_code)]
use osfr_tools::{cooking::payload_packet, gateway::Gateway};

pub fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut b = vec![5, 0, 1];
    b.extend((inner.len() as u32).to_le_bytes());
    b.extend(inner);
    b
}
pub fn send(g: &mut Gateway, inner: &[u8], now: u64) -> osfr_tools::gateway::GatewayOutcome {
    g.handle_at(&wrap(inner), now).unwrap()
}
pub fn instance(g: &Gateway) -> i32 {
    g.snapshot()
        .gameplay
        .unwrap()
        .cooking
        .unwrap()
        .active
        .unwrap()
        .instance_id
}
pub fn text(g: &mut Gateway, text: &str, now: u64) -> osfr_tools::gateway::GatewayOutcome {
    let mut native = payload_packet(instance(g), text).unwrap();
    native[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    send(g, &native, now)
}
pub fn start(g: &mut Gateway, now: u64) {
    let mut b = vec![39, 0, 5];
    for n in [instance(g), -1, -1] {
        b.extend(n.to_le_bytes());
    }
    assert_eq!(send(g, &b, now).replies.len(), 2);
    assert_eq!(text(g, "FRServer_ReadyForGameData", now).replies.len(), 8);
}
pub fn rounds(g: &mut Gateway, now: u64) {
    for stage in [1, 2, 3, 6, 17, 7] {
        for kind in 0..4 {
            text(g, &format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)"), now);
        }
    }
    for kind in 4..8 {
        text(g, &format!("FRServer_StatUpdate\t-1\t{kind}\t6)"), now);
    }
}
pub fn play(g: &mut Gateway, now: u64) {
    let previous = g.snapshot().gameplay.unwrap().cooking.unwrap().completed;
    start(g, now);
    rounds(g, now);
    text(g, "FRServer_GameWon\t900\t0", now);
    assert_eq!(
        g.snapshot().gameplay.unwrap().cooking.unwrap().completed,
        previous + 1
    );
    send(g, &[26, 0, 42, 0], now);
    assert!(
        g.snapshot()
            .gameplay
            .unwrap()
            .cooking
            .unwrap()
            .active
            .is_none()
    );
}
