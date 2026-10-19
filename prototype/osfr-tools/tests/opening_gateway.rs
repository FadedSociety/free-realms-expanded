//! Private packet-driven integration; this is not an actual-client playthrough.
use osfr_tools::{
    gameplay::Gameplay,
    gateway::Gateway,
    opening::{OpeningEngine, OpeningTrigger},
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
#[path = "common/cooking_movie.rs"]
mod cooking_movie;
#[path = "common/forging_movie.rs"]
mod forging_movie;
#[path = "common/smelting_movie.rs"]
mod smelting_movie;

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut p = vec![5, 0, 1];
    p.extend((inner.len() as u32).to_le_bytes());
    p.extend(inner);
    p
}
fn login(gateway: &mut Gateway, now: &mut u64) {
    let mut p = 1u16.to_le_bytes().to_vec();
    for (i, s) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        p.extend((s.len() as u32).to_le_bytes());
        p.extend(s.as_bytes());
        if i == 0 {
            p.extend(1u64.to_le_bytes());
        }
    }
    gateway.handle_at(&p, *now).unwrap();
    ready(gateway, now);
}
fn ready(gateway: &mut Gateway, now: &mut u64) {
    gateway
        .handle_at(&wrap(&13u16.to_le_bytes()), *now)
        .unwrap();
    gateway
        .handle_at(&wrap(&10u16.to_le_bytes()), *now)
        .unwrap();
}
fn move_to(g: &mut Gateway, p: [f32; 3], now: &mut u64) {
    let mut b = 125u16.to_le_bytes().to_vec();
    b.extend(1u64.to_le_bytes());
    for v in p.into_iter().chain([0.0, 0.0, 1.0]) {
        b.extend(v.to_le_bytes());
    }
    b.extend([0, 0]);
    *now += 800;
    g.handle_at(&wrap(&b), *now).unwrap();
}
fn select(g: &mut Gateway, id: i32, now: &mut u64) {
    let mut p = vec![26, 0, 13, 0];
    p.extend(id.to_le_bytes());
    *now += 800;
    g.handle_at(&wrap(&p), *now).unwrap();
}
fn interact(g: &mut Gateway, guid: u64, choice: i32, now: &mut u64) {
    let mut p = vec![26, 0, 10, 0];
    p.extend(guid.to_le_bytes());
    p.extend(choice.to_le_bytes());
    *now += 800;
    g.handle_at(&wrap(&p), *now).unwrap();
}
fn engine(g: &Gateway, content: &Path) -> OpeningEngine {
    OpeningEngine::load(content, g.snapshot().gameplay.unwrap().opening).unwrap()
}
fn restart(g: Gateway, reference: &Path, content: &Path, now: &mut u64) -> Gateway {
    let snapshot = g.snapshot();
    let saved = serde_json::to_vec(&snapshot).unwrap();
    let snapshot: osfr_tools::game_store::Snapshot = serde_json::from_slice(&saved).unwrap();
    let mut fresh =
        Gateway::load_gameplay(reference, snapshot.character, content, snapshot.gameplay).unwrap();
    login(&mut fresh, now);
    fresh
}
fn kill(g: &mut Gateway, guid: u64, pos: [f32; 3], now: &mut u64) {
    move_to(g, pos, now);
    for _ in 0..100 {
        let state = g.snapshot().gameplay.unwrap().encounter.unwrap();
        let enemies = state
            .active
            .as_ref()
            .map(|a| &a.enemies)
            .unwrap_or(&state.world_enemies);
        if enemies.iter().find(|e| e.guid == guid).unwrap().health == 0 {
            return;
        }
        // Actual native key1 request: bar category1, slot0, entity target self.
        let attack = [
            vec![36, 0, 10, 0],
            1_i32.to_le_bytes().to_vec(),
            0_i32.to_le_bytes().to_vec(),
            0_i32.to_le_bytes().to_vec(),
            1_u64.to_le_bytes().to_vec(),
        ]
        .concat();
        *now += 800;
        g.handle_at(&wrap(&attack), *now).unwrap();
    }
    panic!("enemy did not die");
}

#[test]
#[ignore = "requires private opening/reference content; packet simulation is not native playthrough proof"]
fn private_all_opening_actions_dungeon_return_and_restarts() {
    let reference = PathBuf::from(std::env::var_os("OSFR_GATEWAY_REFERENCE").unwrap());
    let content = PathBuf::from(std::env::var_os("OSFR_OPENING_REFERENCE").unwrap());
    let encounters: Value =
        serde_json::from_slice(&fs::read(content.join("encounters.json")).unwrap()).unwrap();
    let initial =
        Gameplay::initial_character(&content, Gateway::reference_state(&reference).unwrap())
            .unwrap();
    let mut g = Gateway::load_gameplay(&reference, initial, &content, None).unwrap();
    let mut now = 0;
    login(&mut g, &mut now);
    assert_eq!(
        g.snapshot()
            .gameplay
            .as_ref()
            .unwrap()
            .inventory
            .items
            .len(),
        7
    );
    // A valid-looking remote entrance cannot accept a quest or zone the player.
    let guide = engine(&g, &content).actors()[0].clone();
    move_to(&mut g, [0.0, 0.0, 0.0], &mut now);
    let before = g.snapshot();
    interact(&mut g, guide.guid, 1, &mut now);
    assert_eq!(g.snapshot(), before);
    for stage_index in 0..17 {
        let e = engine(&g, &content);
        let stage = e.active_stage().unwrap().clone();
        assert_eq!(stage.id, 910001 + stage_index);
        let giver = e
            .actors()
            .iter()
            .find(|a| a.guid == stage.giver_guid)
            .unwrap()
            .clone();
        move_to(&mut g, giver.position, &mut now);
        interact(&mut g, giver.guid, 1, &mut now);
        for _ in 0..200 {
            let e = engine(&g, &content);
            let progress = &e.state().stages[&stage.id];
            if progress.completed {
                break;
            }
            if g.snapshot()
                .gameplay
                .as_ref()
                .unwrap()
                .encounter
                .as_ref()
                .unwrap()
                .active
                .is_some()
            {
                ready(&mut g, &mut now);
                select(&mut g, 43, &mut now);
                let dungeon = &encounters["dungeons"][0];
                for enemy in dungeon["enemies"].as_array().unwrap() {
                    let pos: [f32; 3] = serde_json::from_value(enemy["position"].clone()).unwrap();
                    kill(&mut g, enemy["guid"].as_u64().unwrap(), pos, &mut now);
                }
                assert_eq!(
                    serde_json::to_value(
                        &g.snapshot()
                            .gameplay
                            .as_ref()
                            .unwrap()
                            .encounter
                            .as_ref()
                            .unwrap()
                            .active
                            .as_ref()
                            .unwrap()
                            .phase
                    )
                    .unwrap(),
                    "won"
                );
                let coins = g.snapshot().gameplay.as_ref().unwrap().inventory.coins;
                g = restart(g, &reference, &content, &mut now);
                assert_eq!(
                    g.snapshot().gameplay.as_ref().unwrap().inventory.coins,
                    coins
                );
                g.handle_at(&wrap(&[26, 0, 42, 0]), now).unwrap();
                assert_eq!(g.state().zone_id, 0);
                ready(&mut g, &mut now);
                let farm = e
                    .actors()
                    .iter()
                    .find(|a| a.guid == stage.turnin_guid)
                    .unwrap();
                let mut arrival = farm.position;
                arrival[0] += 3.0;
                assert_eq!(g.state().position, arrival);
                let coins = g.snapshot().gameplay.as_ref().unwrap().inventory.coins;
                // A saved old return point can use the same nearby story guide;
                // remote interaction cannot invoke the magic door.
                move_to(&mut g, [0.0; 3], &mut now);
                let before = g.snapshot();
                interact(&mut g, giver.guid, 1, &mut now);
                assert_eq!(g.snapshot(), before);
                move_to(&mut g, giver.position, &mut now);
                interact(&mut g, giver.guid, 1, &mut now);
                assert_eq!(g.state().position, arrival);
                assert_eq!(
                    g.snapshot().gameplay.as_ref().unwrap().inventory.coins,
                    coins
                );
                ready(&mut g, &mut now);
                continue;
            }
            let pending = stage
                .objectives
                .iter()
                .zip(&progress.counts)
                .find(|(goal, count)| **count < goal.required);
            let Some((goal, _)) = pending else {
                let actor = e
                    .actors()
                    .iter()
                    .find(|a| a.guid == stage.turnin_guid)
                    .unwrap();
                move_to(&mut g, actor.position, &mut now);
                interact(&mut g, actor.guid, 1, &mut now);
                continue;
            };
            match goal.trigger {
                OpeningTrigger::Talk { actor_guid } | OpeningTrigger::Interact { actor_guid } => {
                    let actor = e.actors().iter().find(|a| a.guid == actor_guid).unwrap();
                    if let Some(id) = actor.required_profile {
                        select(&mut g, id, &mut now);
                    }
                    move_to(&mut g, actor.position, &mut now);
                    interact(&mut g, actor.guid, 1, &mut now);
                }
                OpeningTrigger::Gather { .. } | OpeningTrigger::Craft { .. } => {
                    let target = serde_json::to_value(&goal.trigger).unwrap();
                    let actor = e
                        .actors()
                        .iter()
                        .find(|a| {
                            a.action
                                .as_ref()
                                .is_some_and(|t| serde_json::to_value(t).unwrap() == target)
                        })
                        .unwrap();
                    if let Some(id) = actor.required_profile {
                        select(&mut g, id, &mut now);
                    }
                    move_to(&mut g, actor.position, &mut now);
                    interact(&mut g, actor.guid, 1, &mut now);
                    if matches!(goal.trigger, OpeningTrigger::Craft { recipe_id: 2001 })
                        && g.snapshot()
                            .gameplay
                            .as_ref()
                            .unwrap()
                            .cooking
                            .as_ref()
                            .is_some_and(|s| s.active.is_some())
                    {
                        cooking_movie::play(&mut g, now);
                    }
                    if matches!(goal.trigger, OpeningTrigger::Craft { recipe_id: 2002 })
                        && g.snapshot()
                            .gameplay
                            .as_ref()
                            .unwrap()
                            .smelting
                            .as_ref()
                            .is_some_and(|s| s.active.is_some())
                    {
                        smelting_movie::play(&mut g, now);
                    }
                    if matches!(goal.trigger, OpeningTrigger::Craft { recipe_id: 2003 })
                        && g.snapshot()
                            .gameplay
                            .as_ref()
                            .unwrap()
                            .forging
                            .as_ref()
                            .is_some_and(|s| s.active.is_some())
                    {
                        forging_movie::play(&mut g, now);
                    }
                }
                OpeningTrigger::Visit { position, .. } => move_to(&mut g, position, &mut now),
                OpeningTrigger::ClassSelected { profile_id } => {
                    select(&mut g, profile_id, &mut now)
                }
                OpeningTrigger::CosmeticEquipped { definition_id } => {
                    let state = g.snapshot().gameplay.unwrap();
                    let item = state
                        .inventory
                        .items
                        .iter()
                        .find(|i| i.definition_id == definition_id)
                        .unwrap();
                    let inventory: Value =
                        serde_json::from_slice(&fs::read(content.join("inventory.json")).unwrap())
                            .unwrap();
                    let def = inventory["definitions"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|d| d["id"] == definition_id)
                        .unwrap();
                    let mut p = vec![42, 0, 3, 0];
                    p.extend(item.id.to_le_bytes());
                    p.extend(state.profiles.unwrap().active.to_le_bytes());
                    p.extend((def["slot"].as_i64().unwrap() as i32).to_le_bytes());
                    g.handle_at(&wrap(&p), now).unwrap();
                }
                OpeningTrigger::Defeat { definition_id } => {
                    let state = g.snapshot().gameplay.unwrap().encounter.unwrap();
                    let target = encounters["world_enemies"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|a| {
                            a["definition_id"] == definition_id
                                && state
                                    .world_enemies
                                    .iter()
                                    .any(|s| s.guid == a["guid"].as_u64().unwrap() && s.health > 0)
                        })
                        .expect("uncredited quest enemy exhausted");
                    kill(
                        &mut g,
                        target["guid"].as_u64().unwrap(),
                        serde_json::from_value(target["position"].clone()).unwrap(),
                        &mut now,
                    );
                }
                OpeningTrigger::DungeonCompleted { .. } => {
                    move_to(&mut g, giver.position, &mut now);
                    interact(&mut g, giver.guid, 1, &mut now);
                }
                OpeningTrigger::AbilityUsed { .. } => {
                    panic!("ability goal needs a verified native request")
                }
            }
        }
        assert!(
            engine(&g, &content).state().stages[&stage.id].completed,
            "stage {} did not finish",
            stage.id
        );
        g = restart(g, &reference, &content, &mut now);
    }
    for d in engine(&g, &content).discoveries().to_vec() {
        move_to(&mut g, d.position, &mut now);
    }
    assert!(engine(&g, &content).complete());
    let before = g.snapshot();
    for d in engine(&g, &content).discoveries().to_vec() {
        move_to(&mut g, d.position, &mut now);
    }
    assert_eq!(before.gameplay, g.snapshot().gameplay);
}
