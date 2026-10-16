//! Private native-packet integration; no client process or game assets in Git.
use osfr_tools::{
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    opening::{OpeningEngine, OpeningTrigger},
    smelting::payload_packet as authored_payload_packet,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn payload_packet(instance: i32, text: &str) -> std::io::Result<Vec<u8>> {
    let mut native = authored_payload_packet(instance, text)?;
    native[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    Ok(native)
}

#[test]
fn older_gameplay_snapshot_defaults_absent_smelting_state() {
    let old = json!({"inventory":{"coins":0,"items":[],"equipment":[]},
        "quests":{"version":1,"quests":{},"trial_best_ms":null,"trial_completions":0},
        "combat":{"enemy_health":100,"defeats":0}});
    let state: GameplayState = serde_json::from_value(old.clone()).unwrap();
    assert!(state.smelting.is_none());
    assert_eq!(serde_json::to_value(state).unwrap(), old);
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut b = vec![5, 0, 1];
    b.extend((inner.len() as u32).to_le_bytes());
    b.extend(inner);
    b
}
fn send(g: &mut Gateway, inner: &[u8]) -> osfr_tools::gateway::GatewayOutcome {
    g.handle_at(&wrap(inner), 1000).unwrap()
}
fn login(g: &mut Gateway) {
    let mut b = 1u16.to_le_bytes().to_vec();
    for (i, s) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        b.extend((s.len() as u32).to_le_bytes());
        b.extend(s.as_bytes());
        if i == 0 {
            b.extend(1u64.to_le_bytes());
        }
    }
    g.handle_at(&b, 0).unwrap();
    send(g, &13u16.to_le_bytes());
    send(g, &10u16.to_le_bytes());
}
fn select(g: &mut Gateway, profile: i32) {
    let mut b = vec![26, 0, 13, 0];
    b.extend(profile.to_le_bytes());
    send(g, &b);
}
fn position(g: &mut Gateway, p: [f32; 3]) {
    let mut b = 125u16.to_le_bytes().to_vec();
    b.extend(1u64.to_le_bytes());
    for n in p.into_iter().chain([0.0, 0.0, 1.0]) {
        b.extend(n.to_le_bytes());
    }
    b.extend([0, 0]);
    send(g, &b);
}
fn interact(g: &mut Gateway, guid: u64) -> osfr_tools::gateway::GatewayOutcome {
    let mut b = vec![26, 0, 10, 0];
    b.extend(guid.to_le_bytes());
    b.extend(1i32.to_le_bytes());
    send(g, &b)
}
fn count(g: &Gateway, definition: i32) -> u32 {
    g.snapshot()
        .gameplay
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|i| i.definition_id == definition && i.rental.is_none())
        .map(|i| i.count)
        .sum()
}
fn instance(g: &Gateway) -> i32 {
    g.snapshot()
        .gameplay
        .unwrap()
        .smelting
        .unwrap()
        .active
        .unwrap()
        .instance_id
}
fn start(g: &mut Gateway) {
    let mut b = vec![39, 0, 5];
    for n in [instance(g), -1, -1] {
        b.extend(n.to_le_bytes());
    }
    let out = send(g, &b);
    assert_eq!(out.replies.len(), 2); // Native start + real movie launch.
    let out = send(
        g,
        &payload_packet(instance(g), "FRServer_ReadyForGameData").unwrap(),
    );
    assert_eq!(out.replies.len(), 6);
}
fn rounds(g: &mut Gateway) {
    for stage in 101..=104 {
        for kind in 0..=3 {
            send(
                g,
                &payload_packet(
                    instance(g),
                    &format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)"),
                )
                .unwrap(),
            );
        }
    }
}

#[test]
#[ignore = "requires private opening/reference content; native packet simulation, no client launch"]
fn private_mining_smelting_reward_atomic_once_restore_and_cancel() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references");
    let content = std::env::var_os("OSFR_OPENING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| base.join("rust-opening"));
    let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| base.join("rust-gateway-reference"));
    let engine = OpeningEngine::load(&content, None).unwrap();
    let ore = engine
        .actors()
        .iter()
        .find(|a| matches!(a.action, Some(OpeningTrigger::Gather { resource_id: 1003 })))
        .unwrap();
    let forge = engine
        .actors()
        .iter()
        .find(|a| matches!(a.action, Some(OpeningTrigger::Craft { recipe_id: 2002 })))
        .unwrap();
    let mut state = Gameplay::load(&content, None).unwrap().state();
    let original: Value =
        serde_json::from_slice(&fs::read(content.join("opening.json")).unwrap()).unwrap();
    // Fixture starts at the accepted original-story mining step. Earlier steps
    // are marked complete only in memory; no user save or content hash changes.
    for stage in original["stages"].as_array().unwrap() {
        let id = stage["id"].as_u64().unwrap() as u32;
        let progress = state.opening.as_mut().unwrap().stages.get_mut(&id).unwrap();
        if id < 910005 {
            progress.accepted = true;
            progress.completed = true;
            progress.rewarded = true;
            progress.counts = stage["objectives"]
                .as_array()
                .unwrap()
                .iter()
                .map(|g| g["required"].as_u64().unwrap() as u32)
                .collect();
        } else if id == 910005 {
            progress.accepted = true;
            progress.counts = vec![1, 1, 0, 0];
        }
    }
    state.profiles.as_mut().unwrap().active = 14;
    state.profiles.as_mut().unwrap().unlocked.insert(14);
    // Test older saves with no optional smelting field still load unchanged.
    let mut old = serde_json::to_value(&state).unwrap();
    old.as_object_mut().unwrap().remove("smelting");
    state = serde_json::from_value(old).unwrap();
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.zone_id = ore.zone_id;
    character.position = ore.position;
    let mut g = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    login(&mut g);
    assert_eq!(
        (count(&g, 2739), count(&g, 879), count(&g, 21785)),
        (0, 0, 0)
    );
    let mut unaccepted = g.snapshot();
    let progress = unaccepted
        .gameplay
        .as_mut()
        .unwrap()
        .opening
        .as_mut()
        .unwrap()
        .stages
        .get_mut(&910005)
        .unwrap();
    progress.accepted = false;
    progress.counts = vec![0; 4];
    let mut gated = Gateway::load_gameplay(
        &reference,
        unaccepted.character,
        &content,
        unaccepted.gameplay,
    )
    .unwrap();
    login(&mut gated);
    let before = gated.snapshot();
    interact(&mut gated, ore.guid);
    assert_eq!(gated.snapshot(), before); // Unaccepted goal cannot farm.

    let mut empty_recipe = g.snapshot();
    empty_recipe.character.position = forge.position;
    empty_recipe
        .gameplay
        .as_mut()
        .unwrap()
        .opening
        .as_mut()
        .unwrap()
        .stages
        .get_mut(&910005)
        .unwrap()
        .counts = vec![1, 1, 5, 0];
    let mut gated = Gateway::load_gameplay(
        &reference,
        empty_recipe.character,
        &content,
        empty_recipe.gameplay,
    )
    .unwrap();
    login(&mut gated);
    let before = gated.snapshot();
    interact(&mut gated, forge.guid);
    assert_eq!(gated.snapshot(), before); // Accepted goal still requires material inputs.

    position(&mut g, [0.0; 3]);
    let before = g.snapshot();
    interact(&mut g, ore.guid);
    assert_eq!(g.snapshot(), before); // Remote spoof cannot farm.
    position(&mut g, ore.position);
    select(&mut g, 1);
    let before = g.snapshot();
    interact(&mut g, ore.guid);
    assert_eq!(g.snapshot(), before); // Wrong class cannot farm.
    select(&mut g, 14);
    for _ in 0..5 {
        interact(&mut g, ore.guid);
    }
    assert_eq!((count(&g, 2739), count(&g, 879)), (5, 5));
    let mined = g.snapshot();
    for _ in 0..5 {
        interact(&mut g, ore.guid);
    }
    assert_eq!(g.snapshot(), mined); // Exhausted goal cannot farm.

    position(&mut g, forge.position);
    interact(&mut g, forge.guid);
    assert!(g
        .snapshot()
        .gameplay
        .as_ref()
        .unwrap()
        .smelting
        .as_ref()
        .unwrap()
        .active
        .is_some());
    assert_eq!(
        g.snapshot()
            .gameplay
            .as_ref()
            .unwrap()
            .opening
            .as_ref()
            .unwrap()
            .stages[&910005]
            .counts,
        vec![1, 1, 5, 0]
    );
    let before = g.snapshot();
    select(&mut g, 1);
    assert_eq!(g.snapshot(), before); // Movie keeps Miner job.
    start(&mut g);
    rounds(&mut g);
    let win = payload_packet(instance(&g), "FRServer_GameWon\t900\t1").unwrap();
    // A malformed later child must roll back inventory, quest and victory.
    let before = g.snapshot();
    let mut bundle = vec![0, 25];
    for child in [wrap(&win), wrap(&[39, 0, 14])] {
        assert!(child.len() < 255);
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    assert!(g.handle_at(&bundle, 1000).is_err());
    assert_eq!(g.snapshot(), before);
    send(&mut g, &win);
    assert_eq!(
        (count(&g, 2739), count(&g, 879), count(&g, 21785)),
        (0, 2, 5) // Client tutorial locale 1772250646 awards five Copper Bars.
    );
    let won = g.snapshot();
    let progress = won.gameplay.as_ref().unwrap();
    assert_eq!(
        progress.opening.as_ref().unwrap().stages[&910005].counts,
        vec![1, 1, 5, 5]
    );
    assert!(!progress.opening.as_ref().unwrap().stages[&910005].rewarded);
    assert_eq!(progress.smelting.as_ref().unwrap().completed, 1);
    send(&mut g, &win);
    assert_eq!(g.snapshot(), won);

    let path = std::env::temp_dir().join(format!(
        "osfr-smelting-integration-{}.json",
        std::process::id()
    ));
    game_store::save(&path, &won).unwrap();
    let loaded = game_store::load_or_create(&path, won.character.clone()).unwrap();
    fs::remove_file(path).unwrap();
    assert_eq!(loaded, won);
    let mut restored =
        Gateway::load_gameplay(&reference, loaded.character, &content, loaded.gameplay).unwrap();
    login(&mut restored); // New movie session cancels stale data_sent, never awards.
    assert_eq!((count(&restored, 879), count(&restored, 21785)), (2, 5));
    let saved = restored.snapshot();
    assert!(saved
        .gameplay
        .as_ref()
        .unwrap()
        .smelting
        .as_ref()
        .unwrap()
        .active
        .is_none());
    assert_eq!(
        saved
            .gameplay
            .as_ref()
            .unwrap()
            .smelting
            .as_ref()
            .unwrap()
            .completed,
        1
    );
    send(&mut restored, &win);
    assert_eq!(restored.snapshot(), saved);
    let mut ack = vec![39, 0, 7];
    for n in [
        won.gameplay
            .as_ref()
            .unwrap()
            .smelting
            .as_ref()
            .unwrap()
            .active
            .as_ref()
            .unwrap()
            .instance_id,
        -1,
        -1,
    ] {
        ack.extend(n.to_le_bytes());
    }
    assert!(send(&mut g, &ack).replies.is_empty());
    assert_eq!(g.snapshot(), won);
    for bad in [vec![26, 0, 42], vec![26, 0, 42, 0, 0]] {
        assert!(g.handle_at(&wrap(&bad), 1000).is_err());
        assert_eq!(g.snapshot(), won);
    }
    let closed = send(&mut g, &[26, 0, 42, 0]);
    assert!(closed.state_changed && closed.replies.is_empty());
    assert_eq!(g.state().zone_id, won.character.zone_id); // Close does not dungeon-zone.
    assert_eq!(count(&g, 21785), 5);
    assert!(g
        .snapshot()
        .gameplay
        .unwrap()
        .smelting
        .unwrap()
        .active
        .is_none());

    // Restart while still playing cancels without consumption or victory.
    let mut retry =
        Gateway::load_gameplay(&reference, mined.character, &content, mined.gameplay).unwrap();
    login(&mut retry);
    position(&mut retry, forge.position);
    interact(&mut retry, forge.guid);
    start(&mut retry);
    let active = retry.snapshot();
    let old_id = instance(&retry);
    let mut disconnected =
        Gateway::load_gameplay(&reference, active.character, &content, active.gameplay).unwrap();
    login(&mut disconnected);
    assert_eq!(
        (
            count(&disconnected, 2739),
            count(&disconnected, 879),
            count(&disconnected, 21785)
        ),
        (5, 5, 0)
    );
    interact(&mut disconnected, forge.guid);
    assert!(instance(&disconnected) > old_id);
    send(&mut disconnected, &[26, 0, 42, 0]);
    assert_eq!(
        disconnected
            .snapshot()
            .gameplay
            .unwrap()
            .smelting
            .unwrap()
            .completed,
        0
    );
}
