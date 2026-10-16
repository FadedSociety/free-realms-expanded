//! Private native-packet integration; no native client or user save is touched.
#[path = "common/forging_movie.rs"]
mod movie;
use movie::{send, wrap};
use osfr_tools::{
    forging::{ForgingPhase, payload_packet},
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    opening::{OpeningEngine, OpeningTrigger},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[test]
fn older_gameplay_snapshot_defaults_absent_forging_state() {
    let old = json!({"inventory":{"coins":0,"items":[],"equipment":[]},
        "quests":{"version":1,"quests":{},"trial_best_ms":null,"trial_completions":0},
        "combat":{"enemy_health":100,"defeats":0}});
    let state: GameplayState = serde_json::from_value(old.clone()).unwrap();
    assert!(state.forging.is_none());
    assert_eq!(serde_json::to_value(state).unwrap(), old);
}
fn paths() -> (PathBuf, PathBuf) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references");
    (
        std::env::var_os("OSFR_OPENING_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or(base.join("rust-opening-forging")),
        std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or(base.join("rust-gateway-reference")),
    )
}
fn login(g: &mut Gateway) -> Vec<Vec<u8>> {
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
    let mut replies = send(g, &13u16.to_le_bytes(), 1000).replies;
    replies.extend(send(g, &10u16.to_le_bytes(), 1000).replies);
    replies
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
fn fixture(
    stage: u32,
) -> (
    Gateway,
    PathBuf,
    PathBuf,
    u64,
    osfr_tools::opening::OpeningActor,
) {
    let (content, reference) = paths();
    assert!(content.join("forging-campaign.json").is_file());
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
    let opening = OpeningEngine::load(&content, state.opening.clone()).unwrap();
    let stage = opening.active_stage().unwrap();
    let giver = opening
        .actors()
        .iter()
        .find(|a| a.guid == stage.giver_guid)
        .unwrap();
    let anvil = opening
        .actors()
        .iter()
        .find(|a| matches!(a.action, Some(OpeningTrigger::Craft { recipe_id: 2003 })))
        .unwrap()
        .clone();
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.zone_id = giver.zone_id;
    character.position = giver.position;
    let mut g = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    login(&mut g);
    (g, content, reference, giver.guid, anvil)
}

#[test]
#[ignore = "requires private versioned forging campaign; no native launch"]
fn private_acceptance_grant_and_native_forging_win_atomic_once_restore_cancel() {
    let (mut g, content, reference, giver, anvil) = fixture(910006);
    let initial = g.snapshot();
    position(&mut g, [0.0; 3]);
    let remote = g.snapshot();
    interact(&mut g, giver);
    assert_eq!(g.snapshot(), remote);
    position(&mut g, initial.character.position);
    // A later malformed child rolls back acceptance and every material grant.
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
    assert_eq!(
        (
            count(&g, 21785),
            count(&g, 2752),
            count(&g, 55),
            count(&g, 56)
        ),
        (10, 5, 5, 5)
    );
    let granted = g.snapshot();
    interact(&mut g, giver);
    assert_eq!(g.snapshot(), granted);
    let mut g = Gateway::load_gameplay(
        &reference,
        granted.character.clone(),
        &content,
        granted.gameplay.clone(),
    )
    .unwrap();
    login(&mut g);
    interact(&mut g, giver);
    assert_eq!((count(&g, 21785), count(&g, 55)), (10, 5));
    position(&mut g, anvil.position);
    let before = g.snapshot();
    interact(&mut g, anvil.guid);
    assert_eq!(g.snapshot(), before); // Wrong job.
    select(&mut g, 16);
    interact(&mut g, anvil.guid);
    assert!(
        g.snapshot()
            .gameplay
            .unwrap()
            .forging
            .unwrap()
            .active
            .is_some()
    );
    assert_eq!(
        g.snapshot().gameplay.unwrap().opening.unwrap().stages[&910006].counts,
        vec![1, 0]
    );
    let before = g.snapshot();
    select(&mut g, 43);
    assert_eq!(g.snapshot(), before);
    movie::start(&mut g, 1000);
    let playing = g.snapshot();
    let mut restarted = Gateway::load_gameplay(
        &reference,
        playing.character.clone(),
        &content,
        playing.gameplay.clone(),
    )
    .unwrap();
    login(&mut restarted);
    assert!(
        restarted
            .snapshot()
            .gameplay
            .unwrap()
            .forging
            .unwrap()
            .active
            .is_none()
    );
    assert_eq!(
        (count(&restarted, 21785), count(&restarted, 30280)),
        (10, 0)
    );
    interact(&mut restarted, anvil.guid);
    assert!(movie::instance(&restarted) > movie::instance(&g));
    send(&mut restarted, &[26, 0, 42, 0], 1000);
    assert_eq!(
        (count(&restarted, 21785), count(&restarted, 30280)),
        (10, 0)
    );
    movie::text(&mut g, "FRServer_GameWon\t999\t999", 1000);
    assert_eq!(count(&g, 30280), 0);
    movie::reach_quota(&mut g, 1000);
    assert_eq!((count(&g, 21785), count(&g, 30280)), (10, 0)); // Quota alone cannot award.
    let next = payload_packet(movie::instance(&g), "OnNextLevelMsg").unwrap();
    let before = g.snapshot();
    let mut bundle = vec![0, 25];
    for child in [wrap(&next), wrap(&[39, 0, 14])] {
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    assert!(g.handle_at(&bundle, 1000).is_err());
    assert_eq!(g.snapshot(), before);
    send(&mut g, &next, 1000);
    assert_eq!(
        (
            count(&g, 21785),
            count(&g, 2752),
            count(&g, 55),
            count(&g, 56),
            count(&g, 30280)
        ),
        (0, 0, 0, 0, 1)
    );
    let won = g.snapshot();
    assert_eq!(
        won.gameplay
            .as_ref()
            .unwrap()
            .opening
            .as_ref()
            .unwrap()
            .stages[&910006]
            .counts,
        vec![1, 1]
    );
    assert_eq!(
        won.gameplay
            .as_ref()
            .unwrap()
            .forging
            .as_ref()
            .unwrap()
            .completed,
        1
    );
    send(&mut g, &next, 1000);
    assert_eq!(g.snapshot(), won);
    let path = std::env::temp_dir().join(format!(
        "osfr-forging-integration-{}.json",
        std::process::id()
    ));
    game_store::save(&path, &won).unwrap();
    let loaded = game_store::load_or_create(&path, won.character.clone()).unwrap();
    fs::remove_file(path).unwrap();
    assert_eq!(loaded, won);
    // Original board callback precedes native results; ACK is not Close.
    movie::text(&mut g, "FRServer_GameOver", 1000);
    let terminal = g.snapshot();
    let mut ack = vec![39, 0, 7];
    for n in [
        terminal
            .gameplay
            .as_ref()
            .unwrap()
            .forging
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
    assert!(send(&mut g, &ack, 1000).replies.is_empty());
    assert_eq!(g.snapshot(), terminal);
    for bad in [vec![26, 0, 42], vec![26, 0, 42, 0, 0]] {
        assert!(g.handle_at(&wrap(&bad), 1000).is_err());
        assert_eq!(g.snapshot(), terminal);
    }
    let closed = send(&mut g, &[26, 0, 42, 0], 1000);
    assert!(closed.state_changed && closed.replies.is_empty());
    assert!(
        g.snapshot()
            .gameplay
            .unwrap()
            .forging
            .unwrap()
            .active
            .is_none()
    );
    assert_eq!(count(&g, 30280), 1);
    let mut restored =
        Gateway::load_gameplay(&reference, loaded.character, &content, loaded.gameplay).unwrap();
    login(&mut restored);
    assert_eq!(count(&restored, 30280), 1);
    let before = restored.snapshot();
    send(&mut restored, &next, 1000);
    assert_eq!(restored.snapshot(), before);
    // Losing to authoritative elapsed time never consumes ingredients or grants.
    let mut loss =
        Gateway::load_gameplay(&reference, playing.character, &content, playing.gameplay).unwrap();
    login(&mut loss);
    interact(&mut loss, anvil.guid);
    movie::start(&mut loss, 1000);
    assert!(loss.tick_at(181001).unwrap().state_changed);
    assert_eq!(
        loss.snapshot()
            .gameplay
            .unwrap()
            .forging
            .unwrap()
            .active
            .unwrap()
            .phase,
        ForgingPhase::Lost
    );
    assert_eq!((count(&loss, 21785), count(&loss, 30280)), (10, 0));
}

#[test]
#[ignore = "requires private versioned forging campaign; no native launch"]
fn private_repeat_step_supplies_are_once_only_and_enable_smelting_before_forging() {
    let (mut g, _content, _reference, giver, _) = fixture(910015);
    interact(&mut g, giver);
    assert_eq!(
        (
            count(&g, 2739),
            count(&g, 879),
            count(&g, 21785),
            count(&g, 2752),
            count(&g, 55),
            count(&g, 56)
        ),
        (5, 3, 0, 5, 5, 5)
    );
    let before = g.snapshot();
    interact(&mut g, giver);
    assert_eq!(g.snapshot(), before);
    assert_eq!(
        g.snapshot().gameplay.unwrap().opening.unwrap().stages[&910015].counts,
        vec![0, 0, 0, 0]
    );
}

#[test]
#[ignore = "requires private versioned actor repair campaign; no native launch"]
fn private_fire_extinguishing_is_ordered_nearby_persistent_and_not_reinteractive() {
    let (mut g, content, reference, giver, _) = fixture(910003);
    let engine = OpeningEngine::load(&content, None).unwrap();
    let marker: Value =
        serde_json::from_slice(&fs::read(content.join("forging-campaign.json")).unwrap()).unwrap();
    let fires: Vec<_> = marker["disappear_after_interaction"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            engine
                .actors()
                .iter()
                .find(|a| Some(a.guid) == b["actor_guid"].as_u64())
                .unwrap()
                .clone()
        })
        .collect();
    assert_eq!(fires.len(), 3);
    let well = engine
        .actors()
        .iter()
        .find(|a| a.name == "Water Well")
        .unwrap();
    let removal = |guid: u64| {
        let mut b = vec![35, 0, 3, 0, 1, 0];
        b.extend(guid.to_le_bytes());
        b.extend([0; 17]);
        wrap(&b)
    };
    let click = |guid: u64| {
        let mut b = vec![26, 0, 10, 0];
        b.extend(guid.to_le_bytes());
        b.extend(1i32.to_le_bytes());
        b
    };
    // Unaccepted/reordered/remote requests cannot remove a visible fire.
    position(&mut g, fires[0].position);
    let before = g.snapshot();
    assert!(
        !send(&mut g, &click(fires[0].guid), 1000)
            .replies
            .contains(&removal(fires[0].guid))
    );
    assert_eq!(g.snapshot(), before);
    let giver_position = engine
        .actors()
        .iter()
        .find(|a| a.guid == giver)
        .unwrap()
        .position;
    position(&mut g, giver_position);
    interact(&mut g, giver);
    position(&mut g, fires[0].position);
    let before = g.snapshot();
    let out = send(&mut g, &click(fires[0].guid), 1000);
    assert!(!out.replies.contains(&removal(fires[0].guid)));
    assert_eq!(g.snapshot(), before);
    position(&mut g, well.position);
    interact(&mut g, well.guid);
    position(&mut g, [0.0; 3]);
    let before = g.snapshot();
    let out = send(&mut g, &click(fires[0].guid), 1000);
    assert!(!out.replies.contains(&removal(fires[0].guid)));
    assert_eq!(g.snapshot(), before);
    position(&mut g, fires[0].position);
    let before = g.snapshot();
    let mut bundle = vec![0, 25];
    for child in [wrap(&click(fires[0].guid)), wrap(&[26, 0, 10, 0])] {
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    assert!(g.handle_at(&bundle, 1000).is_err());
    assert_eq!(g.snapshot(), before);
    let out = send(&mut g, &click(fires[0].guid), 1000);
    assert_eq!(
        out.replies
            .iter()
            .filter(|p| **p == removal(fires[0].guid))
            .count(),
        1
    );
    let after = g.snapshot();
    assert_eq!(
        after
            .gameplay
            .as_ref()
            .unwrap()
            .opening
            .as_ref()
            .unwrap()
            .stages[&910003]
            .counts,
        vec![1, 1, 0, 0]
    );
    let mut menu = vec![26, 0, 8, 0];
    menu.extend(fires[0].guid.to_le_bytes());
    menu.push(0);
    assert!(send(&mut g, &menu, 1000).replies.is_empty());
    assert!(send(&mut g, &click(fires[0].guid), 1000).replies.is_empty());
    assert_eq!(g.snapshot(), after);
    let serialized = serde_json::to_vec(&after).unwrap();
    let restored: game_store::Snapshot = serde_json::from_slice(&serialized).unwrap();
    let mut g = Gateway::load_gameplay(&reference, restored.character, &content, restored.gameplay)
        .unwrap();
    let packets = login(&mut g);
    let spawns: BTreeSet<_> = packets
        .iter()
        .filter(|p| p.len() >= 19 && p[7..11] == [35, 0, 2, 0])
        .map(|p| u64::from_le_bytes(p[11..19].try_into().unwrap()))
        .collect();
    assert!(!spawns.contains(&fires[0].guid));
    assert!(spawns.contains(&fires[1].guid));
    assert!(spawns.contains(&fires[2].guid));
    assert!(spawns.contains(&well.guid));
    let mut well_menu = vec![26, 0, 8, 0];
    well_menu.extend(well.guid.to_le_bytes());
    well_menu.push(0);
    position(&mut g, well.position);
    assert!(!send(&mut g, &well_menu, 1000).replies.is_empty());
    for fire in &fires[1..] {
        position(&mut g, fire.position);
        assert!(
            send(&mut g, &click(fire.guid), 1000)
                .replies
                .contains(&removal(fire.guid))
        );
    }
    assert_eq!(
        g.snapshot().gameplay.unwrap().opening.unwrap().stages[&910003].counts,
        vec![1, 1, 1, 1]
    );
}
