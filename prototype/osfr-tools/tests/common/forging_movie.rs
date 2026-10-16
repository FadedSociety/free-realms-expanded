//! Test driver for actual original139 messages, never edited board/counters.
#![allow(dead_code)] // Shared by the full story and narrower transaction tests.
use osfr_tools::{
    forging::{ForgingPhase, payload_packet},
    gateway::Gateway,
};

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
        .forging
        .unwrap()
        .active
        .unwrap()
        .instance_id
}
pub fn text(g: &mut Gateway, s: &str, now: u64) -> osfr_tools::gateway::GatewayOutcome {
    let mut native = payload_packet(instance(g), s).unwrap();
    native[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    send(g, &native, now)
}
pub fn start(g: &mut Gateway, now: u64) {
    let mut b = vec![39, 0, 5];
    for n in [instance(g), -1, -1] {
        b.extend(n.to_le_bytes());
    }
    assert_eq!(send(g, &b, now).replies.len(), 2);
    text(g, "OnConnectMsg", now);
    text(g, "OnStartGameMsg\t0", now);
    text(g, "OnStartLevelMsg\t0", now);
}
pub fn reach_quota(g: &mut Gateway, now: u64) {
    for _ in 0..2000 {
        let a = g
            .snapshot()
            .gameplay
            .unwrap()
            .forging
            .unwrap()
            .active
            .unwrap();
        if a.phase == ForgingPhase::LevelComplete {
            return;
        }
        assert_eq!(a.phase, ForgingPhase::Playing);
        let mut idle = String::from("OnIdleMsg\t");
        for y in 0..14 {
            for x in 0..14 {
                if (401..=405).contains(&a.board[y * 14 + x]) || a.board[y * 14 + x] == 606 {
                    idle += &format!("{x},{y},");
                }
            }
        }
        text(g, &idle, now);
        let a = g
            .snapshot()
            .gameplay
            .unwrap()
            .forging
            .unwrap()
            .active
            .unwrap();
        if let Some((x, y)) = (0..13)
            .flat_map(|y| (0..14).map(move |x| (x, y)))
            .find(|&(x, y)| a.board[y * 14 + x] == 606 && a.board[(y + 1) * 14 + x] == 100)
        {
            text(g, &format!("OnLevelEventMsg\t50\t{x}\t{y}"), now);
            continue;
        }
        let mut found = None;
        'outer: for y in (1..13).rev() {
            for x in 1..13 {
                let v = a.board[y * 14 + x];
                if !(401..=405).contains(&v) {
                    continue;
                }
                for (x2, y2) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    if a.board[y2 * 14 + x2] != v {
                        continue;
                    }
                    for (x3, y3) in [
                        (x2.saturating_sub(1), y2),
                        (x2 + 1, y2),
                        (x2, y2.saturating_sub(1)),
                        (x2, y2 + 1),
                    ] {
                        if x3 < 14 && y3 < 14 && (x3, y3) != (x, y) && a.board[y3 * 14 + x3] == v {
                            found = Some(format!("OnChainMsg\t{x},{y},{x2},{y2},{x3},{y3},"));
                            break 'outer;
                        }
                    }
                }
            }
        }
        text(
            g,
            &found.expect("server must reshuffle a board without a move"),
            now,
        );
    }
    panic!("bounded original forging transcript did not reach quota");
}
pub fn play(g: &mut Gateway, now: u64) {
    let prior = g.snapshot().gameplay.unwrap().forging.unwrap().completed;
    start(g, now);
    reach_quota(g, now);
    text(g, "OnNextLevelMsg", now);
    assert_eq!(
        g.snapshot().gameplay.unwrap().forging.unwrap().completed,
        prior + 1
    );
    text(g, "FRServer_GameOver", now);
    send(g, &[26, 0, 42, 0], now);
    assert!(
        g.snapshot()
            .gameplay
            .unwrap()
            .forging
            .unwrap()
            .active
            .is_none()
    );
}
