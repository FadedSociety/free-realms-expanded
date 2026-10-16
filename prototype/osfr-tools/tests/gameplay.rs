//! Explicit integration oracle: private game data is never embedded in Git.
use osfr_tools::{game_store, gateway::Gateway, inventory::InventoryState};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

fn wrapped(inner: &[u8]) -> Vec<u8> {
    let mut p = vec![5, 0, 1];
    p.extend((inner.len() as u32).to_le_bytes());
    p.extend(inner);
    p
}
fn chat(text: &str) -> Vec<u8> {
    let mut p = vec![15, 0, 1, 0, 0, 0];
    p.extend(1u64.to_le_bytes());
    p.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        p.extend([0; 12]);
        p.extend(0u32.to_le_bytes());
        p.extend(0u32.to_le_bytes());
    }
    p.extend((text.len() as u32).to_le_bytes());
    p.extend(text.as_bytes());
    p.extend([0; 28]);
    wrapped(&p)
}
fn position(p: [f32; 3]) -> Vec<u8> {
    let mut b = 125u16.to_le_bytes().to_vec();
    b.extend(1u64.to_le_bytes());
    for v in p.into_iter().chain([0.0, 0.0, 1.0]) {
        b.extend(v.to_le_bytes());
    }
    b.extend([0, 0]);
    wrapped(&b)
}
fn count(inventory: &InventoryState) -> u32 {
    inventory
        .items
        .iter()
        .filter(|x| x.definition_id == 1195 && x.rental.is_none())
        .map(|x| x.count)
        .sum()
}
fn replay(gateway: &mut Gateway, capture: &Path) {
    let analysis: Value =
        serde_json::from_slice(&fs::read(capture.join("analysis/packet-analysis.json")).unwrap())
            .unwrap();
    for record in analysis["directions"]["client-to-server"]["reliable_messages"]
        .as_array()
        .unwrap()
    {
        gateway
            .handle_at(
                &fs::read(capture.join(record["path"].as_str().unwrap())).unwrap(),
                0,
            )
            .unwrap();
    }
}

#[test]
#[ignore = "requires private gameplay content, reference and actual-client capture"]
fn private_complete_loop_reward_once_atomic_bundle_and_fresh_restore() {
    let reference = PathBuf::from(std::env::var("OSFR_GATEWAY_REFERENCE").expect("reference"));
    let content = PathBuf::from(std::env::var("OSFR_GAMEPLAY_CONTENT").expect("gameplay content"));
    let capture = PathBuf::from(std::env::var("OSFR_GATEWAY_CAPTURE").expect("capture"));
    let initial = Gateway::reference_state(&reference).unwrap();
    let mut gateway = Gateway::load(&reference, initial).unwrap();
    gateway.enable_gameplay(&content, None).unwrap();
    replay(&mut gateway, &capture);
    let start = gateway.snapshot().gameplay.unwrap().inventory;
    assert_eq!(start.items.len(), 1386);
    assert_eq!(start.coins, 0);
    let unchanged = gateway.snapshot();
    assert!(gateway.handle_at(&wrapped(&[15, 0, 1, 0]), 500).is_ok());
    assert!(
        gateway
            .handle_at(&chat("realm quest accept unknown"), 500)
            .is_ok()
    );
    assert_eq!(gateway.snapshot(), unchanged);
    gateway
        .handle_at(&chat("realm quest accept 900001"), 1000)
        .unwrap();
    gateway
        .handle_at(&position([-1407.0, -28.2, 344.0]), 1000)
        .unwrap();
    let mut clock = 2000;
    for enemy in 0..3 {
        if enemy != 0 {
            gateway.handle_at(&chat("realm respawn"), clock).unwrap();
            clock += 800;
        }
        for _ in 0..4 {
            gateway.handle_at(&chat("realm attack"), clock).unwrap();
            clock += 800;
        }
    }
    let completed = gateway.snapshot();
    let progress = completed.gameplay.as_ref().unwrap();
    assert_eq!(progress.inventory.coins, 25);
    assert_eq!(count(&progress.inventory), count(&start) + 1);
    gateway.handle_at(&chat("realm attack"), clock).unwrap();
    gateway
        .handle_at(&chat("realm quest"), clock + 800)
        .unwrap();
    assert_eq!(gateway.snapshot(), completed); // Duplicate completion cannot award twice.
    // Ordinary activity precondition errors are feedback, not a disconnect.
    assert!(
        gateway
            .handle_at(&chat("realm race start"), clock + 900)
            .is_ok()
    );
    assert_eq!(gateway.snapshot(), completed);

    // First child would respawn the enemy. Second is a semantically invalid
    // equip request: neither gameplay nor character state may commit.
    let mut invalid_equip = vec![42, 0, 3, 0];
    invalid_equip.extend((-1i32).to_le_bytes());
    invalid_equip.extend([0; 8]);
    let children = [chat("realm respawn"), wrapped(&invalid_equip)];
    let mut bundle = vec![0, 25];
    for child in children {
        assert!(child.len() < 255);
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    assert!(gateway.handle_at(&bundle, clock + 1600).is_err());
    assert_eq!(gateway.snapshot(), completed);

    let directory =
        std::env::temp_dir().join(format!("osfr-gameplay-oracle-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("snapshot.json");
    game_store::save(&path, &completed).unwrap();
    let loaded = game_store::load_or_create(&path, completed.character.clone()).unwrap();
    assert_eq!(loaded, completed);
    let mut fresh = Gateway::load(&reference, loaded.character).unwrap();
    fresh.enable_gameplay(&content, loaded.gameplay).unwrap();
    replay(&mut fresh, &capture);
    // This replay includes the old spawn position, so compare gameplay only.
    assert_eq!(fresh.snapshot().gameplay, completed.gameplay);
    fs::remove_file(path).unwrap();
    fs::remove_dir(directory).unwrap();
}
