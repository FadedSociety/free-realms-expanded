//! Recipe-book crafting at ordinary world stations through the gateway request path.
//!
//! The three positions below come from `npc_spawns_extra.json`. None is one of
//! the tutorial craft actors, so this test proves the world actors themselves
//! are registered rather than only the opening scene. T400: interacting with a
//! station (26/8) opens its recipe window (26/25).

#[path = "common/cooking_movie.rs"]
mod cooking_movie;
#[path = "common/forging_movie.rs"]
mod forging_movie;
#[allow(dead_code)] // Only the shared reply-text extractor is used here.
#[path = "common/s6.rs"]
mod s6;
#[path = "common/station.rs"]
mod station;

use osfr_tools::{
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::InventoryItem,
    recipes::CraftingJob,
    smelting, startup_world,
};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const COPPER_RECIPE: i32 = 4;
const WORLD_COOKING_TABLE: [f32; 3] = [-1244.241, -14.34194, 405.1309];
const WORLD_FORGE: [f32; 3] = [-2631.933, 0.03999766, -216.5819];
const WORLD_ANVIL: [f32; 3] = [354.3665, 48.24031, 316.3203];

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
    /// The station's world spawn (`npc_spawns_extra.json`).
    station_guid: u64,
    /// The job's recipe filter column (Chef 10, Miner 9, Blacksmith 12 (R)).
    filter: u8,
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
        station_guid: 100_000_002_417,
        filter: 10,
        inputs: &[(2862, 4), (2875, 4), (2884, 4), (2900, 4)],
        output: 35,
        output_quantity: 1,
    },
    Case {
        craft: Craft::Smelting,
        profile: 14,
        recipe: COPPER_RECIPE,
        station: WORLD_FORGE,
        station_guid: 100_000_004_582,
        filter: 9,
        inputs: &[(2739, 5), (879, 3)],
        output: 21785,
        output_quantity: 5,
    },
    Case {
        craft: Craft::Forging,
        profile: 16,
        recipe: 11,
        station: WORLD_ANVIL,
        station_guid: 100_000_003_600,
        filter: 12,
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
        let (mut gateway, _content, _reference) = gateway_at(&case, case.station);
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
        station::write_placements(&dir, Some(station::NEAR));

        let (mut gateway, content, reference) = gateway_at(&CASES[1], station::NEAR);
        gateway.set_world_dir(Some(dir.clone())).unwrap();
        book_start(&mut gateway, COPPER_RECIPE);
        assert!(
            recipe_at(&gateway, Craft::Smelting).is_some(),
            "moved={moved}: the placed forge starts the recipe"
        );
        smelting_rounds(&mut gateway);

        station::write_placements(&dir, moved.then_some(station::AWAY));
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

        // A disconnect and reconnect through the production save store cancels
        // the stored movie without paying.
        let saved = gateway.snapshot();
        let save_path = dir.join("character.json");
        game_store::save(&save_path, &saved).unwrap();
        let loaded = game_store::load_or_create(&save_path, saved.character.clone()).unwrap();
        assert_eq!(
            loaded, saved,
            "moved={moved}: the refused run survives a save round trip"
        );
        let mut reconnected =
            Gateway::load_gameplay(&reference, loaded.character, &content, loaded.gameplay)
                .unwrap();
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
        let (mut gateway, content, reference) = gateway_at(&case, case.station);
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

#[test]
fn a_wrong_job_recipe_request_changes_nothing() {
    // Chef recipe 70 at the cooking table while the active job is Miner, the
    // gateway-level partner of `recipe_book_route` (which needs private
    // content). The wrong-job refusal must not start a run or touch the save.
    let case = &CASES[0];
    let (gateway, content, reference) = gateway_at(case, case.station);
    let mut state = gateway.snapshot().gameplay.unwrap();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = CraftingJob::Miner.profile_id();
    profiles.unlocked.insert(profiles.active);
    let mut gateway = Gateway::load_gameplay(
        &reference,
        gateway.snapshot().character,
        &content,
        Some(state),
    )
    .unwrap();
    login(&mut gateway);
    let before = gateway.snapshot();
    assert_eq!(
        before
            .gameplay
            .as_ref()
            .unwrap()
            .profiles
            .as_ref()
            .unwrap()
            .active,
        CraftingJob::Miner.profile_id(),
        "the fixture must be on a real wrong job"
    );
    let outcome = book_start(&mut gateway, case.recipe);
    assert!(!outcome.state_changed && !outcome.gameplay_changed);
    assert_eq!(
        gateway.snapshot(),
        before,
        "a wrong-job recipe request changes nothing"
    );
    assert!(
        recipe_at(&gateway, case.craft).is_none(),
        "the Chef recipe must not start on the Miner"
    );
    let chat = s6::chat_texts(&outcome.replies);
    assert!(
        chat.iter().any(|text| text.contains("Switch to Chef")),
        "{chat:?}"
    );
}

/// Client 26/8 (`008b6bc0`): actor GUID and a zero flag.
fn interact(gateway: &mut Gateway, guid: u64) -> GatewayOutcome {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(guid.to_le_bytes());
    packet.push(0);
    send(gateway, &packet, 2_000)
}

/// Every 26/25 recipe-window command in a reply.
fn windows(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|packet| inner(packet))
        .filter(|packet| packet.starts_with(&[26, 0, 25, 0]))
        .map(<[u8]>::to_vec)
        .collect()
}

/// The 26/25 bytes `00aa0920` reads: header, the filter, then a zero.
fn window(filter: u8) -> Vec<u8> {
    vec![26, 0, 25, 0, filter, 0, 0, 0, 0, 0, 0, 0]
}

/// Reload the saved gateway after `edit`, logging in again.
fn reload(
    gateway: &Gateway,
    content: &std::path::Path,
    reference: &std::path::Path,
    edit: impl FnOnce(&mut game_store::Snapshot),
) -> (Gateway, Vec<GatewayOutcome>) {
    let mut saved = gateway.snapshot();
    edit(&mut saved);
    let mut gateway =
        Gateway::load_gameplay(reference, saved.character, content, saved.gameplay).unwrap();
    let sent = login(&mut gateway);
    (gateway, sent)
}

#[test]
fn a_world_station_spawn_offers_the_x_prompt_at_the_station_radius() {
    let case = &CASES[0];
    let (gateway, content, reference) = gateway_at(case, case.station);
    let (_gateway, sent) = reload(&gateway, &content, &reference, |_| {});
    let packet = sent
        .iter()
        .flat_map(|outcome| &outcome.replies)
        .map(|packet| inner(packet))
        .find(|packet| {
            packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == case.station_guid.to_le_bytes()
        })
        .expect("the world Cooking Table spawns near the player");
    let spawn = startup_world::parse_add_npc(packet).unwrap();
    assert_eq!((spawn.model_id, spawn.name_string_id), (419, 2061));
    // AddNpc reader 00921a10 with an empty name: the interactable flag at byte
    // 224 and, at 237, the prompt reach dispatch 0092f997 stores at
    // actor+0x1818 (a zero falls back to the session's 7 m, run 133027).
    assert_eq!(packet.len(), 334);
    assert_eq!(packet[224], 1);
    assert_eq!(packet[237..241], 12.0f32.to_le_bytes());
    assert_eq!(spawn.unknown_float_51, 12.0);
}

#[test]
fn interacting_with_each_world_station_opens_its_recipe_window() {
    for case in CASES {
        let (mut gateway, _content, _reference) = gateway_at(&case, case.station);
        let before = gateway.snapshot();
        let outcome = interact(&mut gateway, case.station_guid);
        assert_eq!(windows(&outcome), [window(case.filter)], "{}", case.recipe);
        assert!(!outcome.state_changed && !outcome.gameplay_changed);
        assert_eq!(
            gateway.snapshot(),
            before,
            "opening the window changes nothing"
        );
    }
}

#[test]
fn a_station_of_another_job_or_out_of_reach_answers_the_refusal_line() {
    let case = &CASES[0];
    let (gateway, content, reference) = gateway_at(case, case.station);
    let (mut miner, _) = reload(&gateway, &content, &reference, |saved| {
        let profiles = saved.gameplay.as_mut().unwrap().profiles.as_mut().unwrap();
        profiles.active = CraftingJob::Miner.profile_id();
        profiles.unlocked.insert(profiles.active);
    });
    let (mut away, _) = reload(&gateway, &content, &reference, |saved| {
        saved.character.position[0] += 100.0;
    });
    for (gateway, line) in [
        (&mut miner, "Switch to Chef to make that recipe."),
        (&mut away, "Move closer to a cooking station."),
    ] {
        let before = gateway.snapshot();
        let outcome = interact(gateway, case.station_guid);
        assert!(windows(&outcome).is_empty(), "{line}");
        assert_eq!(s6::chat_texts(&outcome.replies), [line]);
        assert!(!outcome.state_changed && !outcome.gameplay_changed);
        assert_eq!(gateway.snapshot(), before, "a refusal changes nothing");
    }
}

#[test]
fn the_farm_cooking_table_keeps_its_tutorial_menu_until_the_tutorial_craft_is_done() {
    // The opening actor and the world spawn of the same Legacy table (2537) share
    // the farm position; run 133027 delivered both to the client.
    const FARM_TABLE: [f32; 3] = [-2109.244, -56.61908, 210.265];
    const OPENING_TABLE: u64 = 5_715_144_128_442_925_070;
    const WORLD_TABLE: u64 = 100_000_002_537;
    let case = &CASES[0];
    let (mut done, content, reference) = gateway_at(case, FARM_TABLE);
    for guid in [OPENING_TABLE, WORLD_TABLE] {
        assert_eq!(windows(&interact(&mut done, guid)), [window(10)], "{guid}");
    }
    // Cooking 101 (910004) accepted with only its craft objective left; the
    // stages after it are not started yet (content order is by id).
    let (mut pending, _) = reload(&done, &content, &reference, |saved| {
        let opening = saved.gameplay.as_mut().unwrap().opening.as_mut().unwrap();
        for (id, stage) in opening.stages.range_mut(910004..) {
            stage.accepted = *id == 910004;
            stage.completed = false;
            stage.rewarded = false;
            stage.counts.iter_mut().for_each(|count| *count = 0);
        }
        opening.stages.get_mut(&910004).unwrap().counts = vec![1, 1, 1, 4, 4, 0];
    });
    let outcome = interact(&mut pending, OPENING_TABLE);
    assert!(windows(&outcome).is_empty());
    assert!(
        outcome
            .replies
            .iter()
            .any(|packet| inner(packet).starts_with(&[26, 0, 9, 0])),
        "the pending tutorial craft keeps the opening menu"
    );
}
