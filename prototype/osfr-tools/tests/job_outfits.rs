//! (R) A job never stays bare: activating a job whose outfit is empty gives
//! its starter outfit. Private: needs a play package (OSFR_PLAY_PACKAGE, e.g.
//! lab/reports/round70-native-20261016T061000Z) whose saved character 1 has
//! Kart Driver unlocked with nothing equipped, as found natively on 2026-10-15.

use osfr_tools::gateway::Gateway;
use std::{fs, path::PathBuf};

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut packet = vec![5, 0, 1];
    packet.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    packet.extend_from_slice(inner);
    packet
}

/// Login, ready and loaded, as the native client does before playing.
fn enter_world(g: &mut Gateway) {
    let mut login = 1u16.to_le_bytes().to_vec();
    for (i, s) in ["local-private-test", "1.910.1.530630", ""].iter().enumerate() {
        login.extend((s.len() as u32).to_le_bytes());
        login.extend(s.bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    g.handle_at(&login, 0).unwrap();
    g.handle_at(&wrap(&[13, 0]), 100).unwrap();
    g.handle_at(&wrap(&[10, 0]), 200).unwrap();
}

#[test]
#[ignore = "requires OSFR_PLAY_PACKAGE pointing at a private play package"]
fn private_switching_to_a_bare_job_dresses_it() {
    let root = PathBuf::from(std::env::var_os("OSFR_PLAY_PACKAGE").unwrap());
    let reference = root.join("../../references/rust-gateway-reference");
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("characters/1.json")).unwrap()).unwrap();
    let character = serde_json::from_value(doc["character"].clone()).unwrap();
    let saved = serde_json::from_value(doc["gameplay"].clone()).unwrap();
    let mut g = Gateway::load_gameplay(&reference, character, &root.join("content"), Some(saved))
        .unwrap();
    enter_world(&mut g);
    let job = |g: &Gateway| g.snapshot().gameplay.unwrap().profiles.unwrap().active;
    let dressed = |g: &Gateway, job: i32| {
        let state = g.snapshot().gameplay.unwrap().inventory;
        state
            .equipment
            .iter()
            .filter(|e| e.profile_id == job && e.item_id != 0)
            .count()
    };
    assert_ne!(job(&g), 48);
    assert_eq!(dressed(&g, 48), 0, "the package save should start bare");
    g.handle_at(&wrap(&[26, 0, 13, 0, 48, 0, 0, 0]), 1_000).unwrap();
    assert!(dressed(&g, 48) >= 3, "Kart Driver got its starter outfit");
    let items = g.snapshot().gameplay.unwrap().inventory.items.len();
    g.handle_at(&wrap(&[26, 0, 13, 0, 43, 0, 0, 0]), 2_000).unwrap();
    g.handle_at(&wrap(&[26, 0, 13, 0, 48, 0, 0, 0]), 3_000).unwrap();
    assert_eq!(
        g.snapshot().gameplay.unwrap().inventory.items.len(),
        items,
        "switching back does not grant duplicates"
    );
}
