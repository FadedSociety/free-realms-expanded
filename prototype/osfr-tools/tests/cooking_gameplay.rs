//! Private packet simulation. User saves and native client are never touched.
#[path = "common/cooking_movie.rs"]
mod movie;
use movie::{send, wrap};
use osfr_tools::{
    cooking::payload_packet,
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    opening::{OpeningEngine, OpeningTrigger},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[test]
fn older_snapshot_defaults_absent_cooking_state() {
    let old = json!({"inventory":{"coins":0,"items":[],"equipment":[]},"quests":{"version":1,"quests":{},"trial_best_ms":null,"trial_completions":0},"combat":{"enemy_health":100,"defeats":0}});
    let state: GameplayState = serde_json::from_value(old.clone()).unwrap();
    assert!(state.cooking.is_none());
    assert_eq!(serde_json::to_value(state).unwrap(), old);
}
fn paths() -> (PathBuf, PathBuf) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references");
    (
        std::env::var_os("OSFR_OPENING_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or(base.join("rust-opening-cooking-original-assets")),
        std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or(base.join("rust-gateway-reference")),
    )
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
    send(g, &13u16.to_le_bytes(), 1000);
    send(g, &10u16.to_le_bytes(), 1000);
}
fn interact(g: &mut Gateway, guid: u64) {
    let mut b = vec![26, 0, 10, 0];
    b.extend(guid.to_le_bytes());
    b.extend(1i32.to_le_bytes());
    send(g, &b, 1000);
}
fn select(g: &mut Gateway, id: i32) {
    let mut b = vec![26, 0, 13, 0];
    b.extend(id.to_le_bytes());
    send(g, &b, 1000);
}
fn position(g: &mut Gateway, p: [f32; 3]) {
    let mut b = 125u16.to_le_bytes().to_vec();
    b.extend(1u64.to_le_bytes());
    for v in p.into_iter().chain([0.0, 0.0, 1.0]) {
        b.extend(v.to_le_bytes());
    }
    b.extend([0, 0]);
    send(g, &b, 1000);
}
fn count(g: &Gateway, id: i32) -> u32 {
    g.snapshot()
        .gameplay
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|i| i.definition_id == id && i.rental.is_none())
        .map(|i| i.count)
        .sum()
}
fn fixture(stage: u32) -> (Gateway, PathBuf, PathBuf) {
    let (content, reference) = paths();
    assert!(content.join("cooking-campaign.json").is_file());
    let original: Value =
        serde_json::from_slice(&fs::read(content.join("opening.json")).unwrap()).unwrap();
    let mut state = Gameplay::load(&content, None).unwrap().state();
    for s in original["stages"].as_array().unwrap() {
        let id = s["id"].as_u64().unwrap() as u32;
        if id < stage {
            let p = state.opening.as_mut().unwrap().stages.get_mut(&id).unwrap();
            p.accepted = true;
            p.completed = true;
            p.rewarded = true;
            p.counts = s["objectives"]
                .as_array()
                .unwrap()
                .iter()
                .map(|g| g["required"].as_u64().unwrap() as u32)
                .collect();
            for id in s["unlock_profiles"].as_array().unwrap() {
                state
                    .profiles
                    .as_mut()
                    .unwrap()
                    .unlocked
                    .insert(id.as_i64().unwrap() as i32);
            }
        }
    }
    let engine = OpeningEngine::load(&content, state.opening.clone()).unwrap();
    let giver = engine
        .actors()
        .iter()
        .find(|a| a.guid == engine.active_stage().unwrap().giver_guid)
        .unwrap();
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.zone_id = giver.zone_id;
    character.position = giver.position;
    let mut g = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    login(&mut g);
    (g, content, reference)
}

#[test]
#[ignore = "requires private versioned Cooking content; no native launch"]
fn private_chef_gather_cooking_atomic_once_gates_restore_cancel() {
    let (mut g, content, reference) = fixture(910004);
    let engine = OpeningEngine::load(&content, g.snapshot().gameplay.unwrap().opening).unwrap();
    let giver = engine.active_stage().unwrap().giver_guid;
    let leaf = engine
        .actors()
        .iter()
        .find(|a| matches!(a.action, Some(OpeningTrigger::Gather { resource_id: 1001 })))
        .unwrap()
        .clone();
    let fruit = engine
        .actors()
        .iter()
        .find(|a| matches!(a.action, Some(OpeningTrigger::Gather { resource_id: 1002 })))
        .unwrap()
        .clone();
    let station = engine
        .actors()
        .iter()
        .find(|a| matches!(a.action, Some(OpeningTrigger::Craft { recipe_id: 2001 })))
        .unwrap()
        .clone();
    let initial = g.snapshot();
    position(&mut g, [0.0; 3]);
    let before = g.snapshot();
    interact(&mut g, giver);
    assert_eq!(g.snapshot(), before);
    position(&mut g, initial.character.position);
    let mut accept = vec![26, 0, 10, 0];
    accept.extend(giver.to_le_bytes());
    accept.extend(1i32.to_le_bytes());
    let mut bundle = vec![0, 25];
    for child in [wrap(&accept), wrap(&[26, 0, 10, 0])] {
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    let before = g.snapshot();
    assert!(g.handle_at(&bundle, 1000).is_err());
    assert_eq!(g.snapshot(), before);
    interact(&mut g, giver);
    assert_eq!((count(&g, 2862), count(&g, 2900)), (4, 4));
    let accepted = g.snapshot();
    interact(&mut g, giver);
    assert_eq!(g.snapshot(), accepted);
    // Accepted alone cannot gather out of order or launch the station.
    position(&mut g, leaf.position);
    let before = g.snapshot();
    interact(&mut g, leaf.guid);
    assert_eq!(g.snapshot(), before);
    position(&mut g, station.position);
    let before = g.snapshot();
    interact(&mut g, station.guid);
    assert_eq!(g.snapshot(), before);
    select(&mut g, 45); // Selecting also sees the original starter shirt already equipped.
    position(&mut g, [0.0; 3]);
    let before = g.snapshot();
    interact(&mut g, leaf.guid);
    assert_eq!(g.snapshot(), before);
    position(&mut g, leaf.position);
    select(&mut g, 1);
    let before = g.snapshot();
    interact(&mut g, leaf.guid);
    assert_eq!(g.snapshot(), before);
    select(&mut g, 45);
    for _ in 0..4 {
        interact(&mut g, leaf.guid);
    }
    assert_eq!(count(&g, 2884), 4);
    let before = g.snapshot();
    for _ in 0..4 {
        interact(&mut g, leaf.guid);
    }
    assert_eq!(g.snapshot(), before);
    position(&mut g, fruit.position);
    for _ in 0..4 {
        interact(&mut g, fruit.guid);
    }
    assert_eq!(count(&g, 2875), 4);
    let gathered = g.snapshot();
    let mut missing = gathered.clone();
    missing
        .gameplay
        .as_mut()
        .unwrap()
        .inventory
        .items
        .retain(|i| i.definition_id != 2862);
    missing.character.position = station.position;
    let mut gated =
        Gateway::load_gameplay(&reference, missing.character, &content, missing.gameplay).unwrap();
    login(&mut gated);
    let before = gated.snapshot();
    interact(&mut gated, station.guid);
    assert_eq!(gated.snapshot(), before);
    position(&mut g, station.position);
    select(&mut g, 1);
    let before = g.snapshot();
    interact(&mut g, station.guid);
    assert_eq!(g.snapshot(), before);
    select(&mut g, 45);
    interact(&mut g, station.guid);
    assert!(
        g.snapshot()
            .gameplay
            .unwrap()
            .cooking
            .unwrap()
            .active
            .is_some()
    );
    let before = g.snapshot();
    select(&mut g, 1);
    interact(&mut g, leaf.guid);
    assert_eq!(g.snapshot(), before);
    movie::start(&mut g, 1000);
    let playing = g.snapshot();
    let old_id = movie::instance(&g);
    let mut restarted =
        Gateway::load_gameplay(&reference, playing.character, &content, playing.gameplay).unwrap();
    login(&mut restarted);
    assert!(
        restarted
            .snapshot()
            .gameplay
            .unwrap()
            .cooking
            .unwrap()
            .active
            .is_none()
    );
    assert_eq!((count(&restarted, 2862), count(&restarted, 35)), (4, 0));
    interact(&mut restarted, station.guid);
    assert!(movie::instance(&restarted) > old_id);
    send(&mut restarted, &[26, 0, 42, 0], 1000);
    assert_eq!((count(&restarted, 2862), count(&restarted, 35)), (4, 0));
    let early = payload_packet(movie::instance(&g), "FRServer_GameWon\t900\t0").unwrap();
    let before = g.snapshot();
    assert!(g.handle_at(&wrap(&early), 1000).is_err());
    assert_eq!(g.snapshot(), before);
    movie::rounds(&mut g, 1000);
    let win = payload_packet(movie::instance(&g), "FRServer_GameWon\t900\t0").unwrap();
    let before = g.snapshot();
    let mut bundle = vec![0, 25];
    for child in [wrap(&win), wrap(&[39, 0, 14])] {
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    assert!(g.handle_at(&bundle, 1000).is_err());
    assert_eq!(g.snapshot(), before);
    // Movement during the movie cannot remotely complete the transaction.
    position(&mut g, [0.0; 3]);
    let before = g.snapshot();
    assert!(g.handle_at(&wrap(&win), 1000).is_err());
    assert_eq!(g.snapshot(), before);
    position(&mut g, station.position);
    send(&mut g, &win, 1000);
    assert_eq!(
        (
            count(&g, 2862),
            count(&g, 2875),
            count(&g, 2884),
            count(&g, 2900),
            count(&g, 35),
            count(&g, 14794)
        ),
        (0, 0, 0, 0, 1, 0)
    );
    let won = g.snapshot();
    assert_eq!(
        won.gameplay
            .as_ref()
            .unwrap()
            .opening
            .as_ref()
            .unwrap()
            .stages[&910004]
            .counts,
        vec![1, 1, 1, 4, 4, 1]
    );
    assert_eq!(
        won.gameplay
            .as_ref()
            .unwrap()
            .cooking
            .as_ref()
            .unwrap()
            .completed,
        1
    );
    send(&mut g, &win, 1000);
    assert_eq!(g.snapshot(), won);
    let path = std::env::temp_dir().join(format!(
        "osfr-cooking-integration-{}.json",
        std::process::id()
    ));
    game_store::save(&path, &won).unwrap();
    let loaded = game_store::load_or_create(&path, won.character.clone()).unwrap();
    fs::remove_file(path).unwrap();
    assert_eq!(loaded, won);
    let mut ack = vec![39, 0, 7];
    for n in [movie::instance(&g), -1, -1] {
        ack.extend(n.to_le_bytes());
    }
    assert!(send(&mut g, &ack, 1000).replies.is_empty());
    assert_eq!(g.snapshot(), won);
    for bad in [vec![26, 0, 42], vec![26, 0, 42, 0, 0]] {
        assert!(g.handle_at(&wrap(&bad), 1000).is_err());
        assert_eq!(g.snapshot(), won);
    }
    let closed = send(&mut g, &[26, 0, 42, 0], 1000);
    assert!(closed.state_changed && closed.replies.is_empty());
    assert_eq!(g.state().zone_id, 0);
    position(&mut g, initial.character.position);
    interact(&mut g, giver);
    assert_eq!(count(&g, 14794), 1);
    assert_eq!(count(&g, 35), 1);
    assert!(g.snapshot().gameplay.unwrap().opening.unwrap().stages[&910004].rewarded);
    let mut restored =
        Gateway::load_gameplay(&reference, loaded.character, &content, loaded.gameplay).unwrap();
    login(&mut restored);
    assert_eq!((count(&restored, 35), count(&restored, 2862)), (1, 0));
    let saved = restored.snapshot();
    send(&mut restored, &win, 1000);
    assert_eq!(restored.snapshot(), saved);
}

#[test]
#[ignore = "requires private versioned Cooking content; no native launch"]
fn private_repeat_chef_supplies_once_and_old_campaign_compatibility() {
    let (mut g, content, reference) = fixture(910013);
    let engine = OpeningEngine::load(&content, g.snapshot().gameplay.unwrap().opening).unwrap();
    let giver = engine.active_stage().unwrap().giver_guid;
    interact(&mut g, giver);
    assert_eq!((count(&g, 2862), count(&g, 2900)), (4, 4));
    let accepted = g.snapshot();
    interact(&mut g, giver);
    assert_eq!(g.snapshot(), accepted);
    let saved = g.snapshot();
    let mut restored =
        Gateway::load_gameplay(&reference, saved.character, &content, saved.gameplay).unwrap();
    login(&mut restored);
    interact(&mut restored, giver);
    assert_eq!((count(&restored, 2862), count(&restored, 2900)), (4, 4));
    let previous = content
        .parent()
        .unwrap()
        .join("rust-opening-forging-assets");
    let old = Gameplay::load(&previous, None).unwrap().state();
    assert!(old.cooking.is_none());
    assert!(Gameplay::load(&previous, Some(old)).is_ok());
}
