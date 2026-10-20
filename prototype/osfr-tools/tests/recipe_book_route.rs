//! 26/24 through the real gateway: recipe book -> station -> cooking movie ->
//! one atomic reward. Private Cooking content; no native launch.
#[path = "common/cooking_movie.rs"]
mod movie;
use movie::{send, wrap};
use osfr_tools::{
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    inventory::InventoryItem,
    opening::{OpeningEngine, OpeningTrigger},
};
use serde_json::Value;
use std::{fs, path::PathBuf};

// The tutorial stew row (chef.strong_arm_stew.sample): the reference item
// definitions only cover its inputs and product.
const GUMBO: i32 = 70;
const GUMBO_ITEM: i32 = 35;
const INPUTS: [(i32, u32); 4] = [(2862, 4), (2875, 4), (2884, 4), (2900, 4)];

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
fn select(g: &mut Gateway, id: i32) {
    let mut b = vec![26, 0, 13, 0];
    b.extend(id.to_le_bytes());
    send(g, &b, 1000);
}
/// Movement is speed-checked against the clock, so each hop gets a long gap.
fn position(g: &mut Gateway, p: [f32; 3], now: u64) {
    let mut b = 125u16.to_le_bytes().to_vec();
    b.extend(1u64.to_le_bytes());
    for v in p.into_iter().chain([0.0, 0.0, 1.0]) {
        b.extend(v.to_le_bytes());
    }
    b.extend([0, 0]);
    send(g, &b, now);
    assert_eq!(g.snapshot().character.position, p, "move accepted");
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
/// Compare two snapshots; on mismatch name the changed paths, not the dump.
fn assert_same(
    a: &osfr_tools::game_store::Snapshot,
    b: &osfr_tools::game_store::Snapshot,
    what: &str,
) {
    fn walk(path: String, a: &Value, b: &Value, out: &mut Vec<String>) {
        if a == b {
            return;
        }
        match (a, b) {
            (Value::Object(x), Value::Object(y)) => {
                for k in x.keys().chain(y.keys()) {
                    walk(
                        format!("{path}/{k}"),
                        &x.get(k).unwrap_or(&Value::Null),
                        y.get(k).unwrap_or(&Value::Null),
                        out,
                    );
                }
            }
            _ => out.push(path),
        }
    }
    let (a, b) = (
        serde_json::to_value(a).unwrap(),
        serde_json::to_value(b).unwrap(),
    );
    let mut diff = Vec::new();
    walk(String::new(), &a, &b, &mut diff);
    diff.dedup();
    assert!(diff.is_empty(), "{what}: changed {diff:?}");
}
fn book_start(g: &mut Gateway, recipe: i32) {
    let mut b = vec![26, 0, 24, 0];
    b.extend(recipe.to_le_bytes());
    send(g, &b, 1000);
}
fn cooking_active(g: &Gateway) -> bool {
    g.snapshot()
        .gameplay
        .unwrap()
        .cooking
        .unwrap()
        .active
        .is_some()
}

/// A finished tutorial (no pending Craft objective), Chef unlocked, recipe 35
/// learned, its four inputs owned, standing at the stove.
fn fixture(learned: bool, with_inputs: bool) -> (Gateway, PathBuf, PathBuf, [f32; 3]) {
    let (content, reference) = paths();
    let original: Value =
        serde_json::from_slice(&fs::read(content.join("opening.json")).unwrap()).unwrap();
    let mut state: GameplayState = Gameplay::load(&content, None).unwrap().state();
    for s in original["stages"].as_array().unwrap() {
        let id = s["id"].as_u64().unwrap() as u32;
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
    // The finished tutorial already learned the stew row; make it explicit.
    if learned {
        state.learned_recipes.insert(GUMBO as u32);
    } else {
        state.learned_recipes.remove(&(GUMBO as u32));
    }
    if with_inputs {
        for (n, (definition_id, quantity)) in INPUTS.into_iter().enumerate() {
            state.inventory.items.push(InventoryItem {
                definition_id,
                tint: 0,
                id: 5_000_001 + n as i32,
                count: quantity,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: 0,
                rental: None,
                ability_count: 0,
                activate_enabled: true,
            });
        }
    }
    let engine = OpeningEngine::load(&content, state.opening.clone()).unwrap();
    let station = engine
        .actors()
        .iter()
        .find(|a| matches!(a.action, Some(OpeningTrigger::Craft { recipe_id: 2001 })))
        .unwrap()
        .clone();
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.zone_id = station.zone_id;
    character.position = station.position;
    let mut g = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    login(&mut g);
    select(&mut g, 45);
    position(&mut g, station.position, 100_000); // baseline clock for the speed check
    (g, content, reference, station.position)
}

#[test]
#[ignore = "requires private versioned Cooking content; no native launch"]
fn recipe_book_refusals_change_nothing() {
    // Missing inputs.
    let (mut g, ..) = fixture(true, false);
    let before = g.snapshot();
    book_start(&mut g, GUMBO);
    assert_same(&g.snapshot(), &before, "refusal");
    // Unknown ids, and away from every station.
    let (mut g, _, _, station) = fixture(true, true);
    let before = g.snapshot();
    for id in [0, -1, 12345, i32::MIN, i32::MAX] {
        book_start(&mut g, id);
        assert_eq!(g.snapshot(), before, "id {id}");
    }
    position(&mut g, [0.0; 3], 1_000_000);
    let away = g.snapshot();
    book_start(&mut g, GUMBO);
    assert_eq!(g.snapshot(), away);
    // Wrong active job. Profile 1 has no equipped Chest/Legs, so the switch is
    // refused and the player stays Chef; profile 43 has the required outfit
    // slots and actually switches, which is the case this block must exercise.
    position(&mut g, station, 2_000_000);
    select(&mut g, 43);
    let other_job = g.snapshot();
    assert_eq!(
        other_job
            .gameplay
            .as_ref()
            .unwrap()
            .profiles
            .as_ref()
            .unwrap()
            .active,
        43,
        "the wrong-job fixture must actually switch jobs"
    );
    book_start(&mut g, GUMBO);
    assert_eq!(g.snapshot(), other_job);
    // Malformed lengths never start a run.
    select(&mut g, 45);
    let ready = g.snapshot();
    for bad in [vec![26, 0, 24, 0], vec![26, 0, 24, 0, 1, 0, 0, 0, 0]] {
        // The gateway drops a rejected single packet; nothing may change.
        let _ = g.handle_at(&wrap(&bad), 1000);
        assert_same(&g.snapshot(), &ready, "malformed");
    }
}

#[test]
#[ignore = "requires private versioned Cooking content; no native launch"]
fn recipe_book_cooks_a_learned_recipe_once() {
    let (mut g, _, _, _) = fixture(true, true);
    assert!(!cooking_active(&g));
    book_start(&mut g, GUMBO);
    assert!(cooking_active(&g));
    let started = g.snapshot();
    // A replay while the movie runs neither restarts nor doubles anything.
    book_start(&mut g, GUMBO);
    assert_eq!(g.snapshot(), started);
    // Starting consumes nothing; the win consumes exactly the inputs once.
    assert_eq!(count(&g, GUMBO_ITEM), 0);
    movie::start(&mut g, 1000);
    movie::rounds(&mut g, 1000);
    let win = osfr_tools::cooking::payload_packet(movie::instance(&g), "FRServer_GameWon\t900\t0")
        .unwrap();
    // Walking away mid-movie cannot complete the craft remotely.
    let at_station = g.snapshot().character.position;
    position(&mut g, [0.0; 3], 1_000_000);
    let before = g.snapshot();
    let _ = g.handle_at(&wrap(&win), 1000);
    assert_same(&g.snapshot(), &before, "refusal");
    position(&mut g, at_station, 2_000_000);
    send(&mut g, &win, 1000);
    for (definition, _) in INPUTS {
        assert_eq!(count(&g, definition), 0, "input {definition}");
    }
    assert_eq!(count(&g, GUMBO_ITEM), 1);
    let won = g.snapshot();
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
    // Ordinary exit restores the world; the recipe cannot be re-run without inputs.
    send(&mut g, &[26, 0, 42, 0], 1000);
    assert!(!cooking_active(&g));
    let idle = g.snapshot();
    book_start(&mut g, GUMBO);
    assert_eq!(g.snapshot(), idle);
}
