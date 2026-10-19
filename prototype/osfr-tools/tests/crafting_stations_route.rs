//! Recipe-book crafting at ordinary world stations through the gateway request path.
//!
//! The three positions below come from `npc_spawns_extra.json`. None is one of
//! the tutorial craft actors, so this test proves the world actors themselves
//! are registered rather than only the opening scene.

#[path = "common/cooking_movie.rs"]
mod cooking_movie;
#[path = "common/forging_movie.rs"]
mod forging_movie;

use osfr_tools::{
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::InventoryItem,
    smelting,
    world::{PLACEMENT_GUID_BASE, Placement, PlacementFile},
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const COPPER_RECIPE: i32 = 990001;
const WORLD_COOKING_TABLE: [f32; 3] = [-1244.241, -14.34194, 405.1309];
const WORLD_FORGE: [f32; 3] = [-2631.933, 0.03999766, -216.5819];
const WORLD_ANVIL: [f32; 3] = [354.3665, 48.24031, 316.3203];
/// A live-world `sh_forge_02` placement (model 1733, `data/content/crafting_stations.json`)
/// at a position no shipped station occupies.
const STATION_NEAR: [f32; 3] = [321.0, 0.0, 654.0];
const STATION_AWAY: [f32; 3] = [1_321.0, 0.0, 1_654.0];
const STATION_MODEL: u32 = 1733;
const STATION_GUID: u64 = PLACEMENT_GUID_BASE + 1;

#[derive(Clone, Copy)]
enum Craft {
    Cooking,
    Smelting,
    Forging,
}

struct Case {
    craft: Craft,
    profile: i32,
    recipe: i32,
    station: [f32; 3],
    inputs: &'static [(i32, u32)],
    output: i32,
    output_quantity: u32,
}

const CASES: [Case; 3] = [
    Case {
        craft: Craft::Cooking,
        profile: 45,
        recipe: 70,
        station: WORLD_COOKING_TABLE,
        inputs: &[(2862, 4), (2875, 4), (2884, 4), (2900, 4)],
        output: 35,
        output_quantity: 1,
    },
    Case {
        craft: Craft::Smelting,
        profile: 14,
        recipe: COPPER_RECIPE,
        station: WORLD_FORGE,
        inputs: &[(2739, 5), (879, 3)],
        output: 21785,
        output_quantity: 5,
    },
    Case {
        craft: Craft::Forging,
        profile: 16,
        recipe: 11,
        station: WORLD_ANVIL,
        inputs: &[(21785, 10), (2752, 5), (55, 5), (56, 5)],
        output: 30280,
        output_quantity: 1,
    },
];

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

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn inner(packet: &[u8]) -> &[u8] {
    if packet.starts_with(&[5, 0, 1]) && packet.len() >= 7 {
        let length = u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize;
        if length == packet.len() - 7 {
            return &packet[7..];
        }
    }
    packet
}

fn send(gateway: &mut Gateway, packet: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(packet), now_ms)
        .expect("gateway frame")
}

fn login(gateway: &mut Gateway) -> Vec<GatewayOutcome> {
    let mut body = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        body.extend((text.len() as u32).to_le_bytes());
        body.extend(text.as_bytes());
        if index == 0 {
            body.extend(1u64.to_le_bytes());
        }
    }
    vec![
        gateway.handle_at(&body, 0).unwrap(),
        send(gateway, &13u16.to_le_bytes(), 1_000),
        send(gateway, &10u16.to_le_bytes(), 1_000),
    ]
}

fn count(gateway: &Gateway, definition_id: i32) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

fn book_start(gateway: &mut Gateway, recipe: i32) -> GatewayOutcome {
    let mut packet = vec![26, 0, 24, 0];
    packet.extend(recipe.to_le_bytes());
    send(gateway, &packet, 2_000)
}

fn recipe_at(gateway: &Gateway, craft: Craft) -> Option<i32> {
    let gameplay = gateway.snapshot().gameplay.unwrap();
    match craft {
        Craft::Cooking => gameplay
            .cooking
            .and_then(|state| state.active.map(|run| run.instance_id)),
        Craft::Smelting => gameplay
            .smelting
            .and_then(|state| state.active.map(|run| run.instance_id)),
        Craft::Forging => gameplay
            .forging
            .and_then(|state| state.active.map(|run| run.instance_id)),
    }
}

fn completed(gateway: &Gateway, craft: Craft) -> u32 {
    let gameplay = gateway.snapshot().gameplay.unwrap();
    match craft {
        Craft::Cooking => gameplay.cooking.unwrap().completed,
        Craft::Smelting => gameplay.smelting.unwrap().completed,
        Craft::Forging => gameplay.forging.unwrap().completed,
    }
}

fn smelting_text(gateway: &mut Gateway, text: &str) {
    let id = recipe_at(gateway, Craft::Smelting).expect("smelting run");
    let mut packet = smelting::payload_packet(id, text).unwrap();
    packet[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    cooking_movie::send(gateway, &packet, 2_000);
}

/// The native movie lifecycle up to (not including) the win report.
fn smelting_rounds(gateway: &mut Gateway) {
    let id = recipe_at(gateway, Craft::Smelting).expect("smelting run");
    let mut start = vec![39, 0, 5];
    for value in [id, -1, -1] {
        start.extend(value.to_le_bytes());
    }
    assert_eq!(cooking_movie::send(gateway, &start, 2_000).replies.len(), 2);
    let mut ready = smelting::payload_packet(id, "FRServer_ReadyForGameData").unwrap();
    ready[11..15].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(cooking_movie::send(gateway, &ready, 2_000).replies.len(), 6);
    for stage in 101..=104 {
        for kind in 0..=3 {
            smelting_text(
                gateway,
                &format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)"),
            );
        }
    }
}

fn play_smelting(gateway: &mut Gateway) {
    smelting_rounds(gateway);
    smelting_text(gateway, "FRServer_GameWon\t900\t1");
    cooking_movie::send(gateway, &[26, 0, 42, 0], 2_000);
}

/// Write the live-world forge placement, or an empty placement file.
fn write_station(dir: &Path, at: Option<[f32; 3]>) {
    let records = at
        .map(|position| Placement {
            guid: STATION_GUID,
            model_id: STATION_MODEL,
            name_string_id: 0,
            name: None,
            position,
            heading: 0.0,
            scale: 1.0,
            extra: serde_json::Map::new(),
        })
        .into_iter()
        .collect();
    fs::write(
        dir.join("npc_placements.json"),
        serde_json::to_vec(&PlacementFile {
            schema_version: 1,
            records,
        })
        .unwrap(),
    )
    .unwrap();
}

fn fixture(case: &Case) -> (Gateway, PathBuf, PathBuf) {
    gateway_at(case, case.station)
}

fn gateway_at(case: &Case, position: [f32; 3]) -> (Gateway, PathBuf, PathBuf) {
    let (content, reference) = paths();
    assert!(content.join("opening.json").is_file());
    assert!(content.join("cooking-campaign.json").is_file());
    assert!(content.join("forging-campaign.json").is_file());
    let original: Value =
        serde_json::from_slice(&fs::read(content.join("opening.json")).unwrap()).unwrap();
    let mut state: GameplayState = Gameplay::load(&content, None).unwrap().state();
    for stage in original["stages"].as_array().unwrap() {
        let id = stage["id"].as_u64().unwrap() as u32;
        let progress = state.opening.as_mut().unwrap().stages.get_mut(&id).unwrap();
        progress.accepted = true;
        progress.completed = true;
        progress.rewarded = true;
        progress.counts = stage["objectives"]
            .as_array()
            .unwrap()
            .iter()
            .map(|goal| goal["required"].as_u64().unwrap() as u32)
            .collect();
        for profile in stage["unlock_profiles"].as_array().unwrap() {
            state
                .profiles
                .as_mut()
                .unwrap()
                .unlocked
                .insert(profile.as_i64().unwrap() as i32);
        }
    }
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = case.profile;
    profiles.unlocked.insert(case.profile);

    let mut next = state
        .inventory
        .items
        .iter()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    for &(definition_id, quantity) in case.inputs {
        state.inventory.items.push(InventoryItem {
            definition_id,
            tint: 0,
            id: next,
            count: quantity,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: true,
        });
        next += 1;
    }
    state.inventory.next_instance_id = Some(i64::from(next));

    let mut character = Gateway::reference_state(&reference).unwrap();
    character.zone_id = 0;
    character.position = position;
    let mut gateway = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    let sent = login(&mut gateway);
    assert!(
        sent.iter().any(|outcome| {
            outcome.replies.iter().any(|packet| {
                let packet = inner(packet);
                packet.starts_with(&[63, 0, 5])
                    && packet
                        .windows(4)
                        .any(|window| window == case.recipe.to_le_bytes())
            })
        }),
        "the learned recipe {} is sent to the client",
        case.recipe
    );
    (gateway, content, reference)
}

#[test]
fn world_stations_start_and_finish_all_three_crafts_through_26_24() {
    for case in CASES {
        let (mut gateway, _content, _reference) = fixture(&case);
        let before: Vec<_> = case
            .inputs
            .iter()
            .map(|&(item, _)| count(&gateway, item))
            .collect();
        let output_before = count(&gateway, case.output);
        let completed_before = completed(&gateway, case.craft);

        book_start(&mut gateway, case.recipe);
        assert!(
            recipe_at(&gateway, case.craft).is_some(),
            "recipe {} starts at its world station {:?}",
            case.recipe,
            case.station
        );

        match case.craft {
            Craft::Cooking => cooking_movie::play(&mut gateway, 2_000),
            Craft::Smelting => play_smelting(&mut gateway),
            Craft::Forging => forging_movie::play(&mut gateway, 2_000),
        }

        for (&(item, required), previous) in case.inputs.iter().zip(before) {
            assert_eq!(count(&gateway, item) + required, previous, "input {item}");
        }
        assert_eq!(
            count(&gateway, case.output),
            output_before + case.output_quantity,
            "output {}",
            case.output
        );
        assert_eq!(
            completed(&gateway, case.craft),
            completed_before + 1,
            "recipe {} completion",
            case.recipe
        );
    }
}

#[test]
fn a_moved_or_removed_station_refuses_completion_without_changing_state() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    // The client movie lifecycle drives the transaction; a placement reload
    // between the rounds and the win removes the station it was started at.
    for moved in [true, false] {
        let dir = std::env::temp_dir().join(format!(
            "osfr-station-completion-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        write_station(&dir, Some(STATION_NEAR));

        let (mut gateway, content, reference) = gateway_at(&CASES[1], STATION_NEAR);
        gateway.set_world_dir(Some(dir.clone())).unwrap();
        book_start(&mut gateway, COPPER_RECIPE);
        assert!(
            recipe_at(&gateway, Craft::Smelting).is_some(),
            "moved={moved}: the placed forge starts the recipe"
        );
        smelting_rounds(&mut gateway);

        write_station(&dir, moved.then_some(STATION_AWAY));
        gateway.set_world_dir(Some(dir.clone())).unwrap();
        let before = gateway.snapshot();
        let id = recipe_at(&gateway, Craft::Smelting).unwrap();
        let mut win = smelting::payload_packet(id, "FRServer_GameWon\t900\t1").unwrap();
        win[11..15].copy_from_slice(&(-1i32).to_le_bytes());
        let refused = cooking_movie::send(&mut gateway, &win, 2_000);
        assert!(
            !refused.state_changed,
            "moved={moved}: a refused completion must not request a save"
        );
        assert!(
            !refused.gameplay_changed,
            "moved={moved}: a refused completion must not persist gameplay"
        );
        assert_eq!(
            gateway.snapshot(),
            before,
            "moved={moved}: a refused completion changes nothing"
        );
        assert!(
            recipe_at(&gateway, Craft::Smelting).is_some(),
            "moved={moved}: the refused completion leaves the run open"
        );

        // A disconnect and reconnect cancels the stored movie without paying.
        let saved = gateway.snapshot();
        let mut reconnected =
            Gateway::load_gameplay(&reference, saved.character, &content, saved.gameplay).unwrap();
        reconnected.set_world_dir(Some(dir.clone())).unwrap();
        login(&mut reconnected);
        assert!(
            recipe_at(&reconnected, Craft::Smelting).is_none(),
            "moved={moved}: a reconnect releases the refused run"
        );
        assert_eq!(
            count(&reconnected, 21785),
            0,
            "moved={moved}: a refused completion never pays"
        );
        let _ = fs::remove_dir_all(dir);
    }
}

#[test]
fn a_refused_world_station_request_changes_nothing() {
    for case in CASES {
        let (mut gateway, content, reference) = fixture(&case);
        let mut character = gateway.snapshot().character;
        character.position = [case.station[0] + 100.0, case.station[1], case.station[2]];
        let before = gateway.snapshot();
        gateway = Gateway::load_gameplay(&reference, character, &content, before.gameplay.clone())
            .unwrap();
        login(&mut gateway);
        let refused = gateway.snapshot();
        let outcome = book_start(&mut gateway, case.recipe);
        assert!(!outcome.state_changed && !outcome.gameplay_changed);
        assert_eq!(gateway.snapshot(), refused, "recipe {}", case.recipe);
        assert!(recipe_at(&gateway, case.craft).is_none());
    }
}
