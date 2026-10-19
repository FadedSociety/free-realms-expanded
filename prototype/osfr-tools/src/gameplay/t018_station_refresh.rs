use super::*;
use crate::smelting::SmeltingOutcome;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

// The placement fixture is shared with `tests/crafting_stations_route.rs`
// through `tests/common/station.rs`; `#[cfg(test)]` helpers are not importable
// from integration tests, so the integration test includes that file directly.
#[path = "../../tests/common/station.rs"]
mod station;

static NEXT: AtomicU64 = AtomicU64::new(0);
const RECIPE: i32 = 4;

fn content_directory() -> PathBuf {
    std::env::var_os("OSFR_OPENING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn live_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "osfr-station-refresh-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn character_at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn start_request() -> Vec<u8> {
    let mut packet = vec![26, 0, 24, 0];
    packet.extend(RECIPE.to_le_bytes());
    packet
}

fn stock_recipe(state: &mut GameplayState) {
    let mut next = state
        .inventory
        .items
        .iter()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        + 1;
    for (definition_id, count) in [(2739, 5), (879, 3)] {
        state.inventory.items.push(crate::inventory::InventoryItem {
            definition_id,
            tint: 0,
            id: next,
            count,
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
}

fn station_game(dir: &Path) -> Gameplay {
    let content = content_directory();
    assert!(content.join("opening.json").is_file());
    let mut state = Gameplay::load(&content, None).unwrap().state();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.unlocked.insert(14);
    profiles.active = 14;
    stock_recipe(&mut state);
    let mut game = Gameplay::load(&content, Some(state)).unwrap();
    game.set_world_dir(Some(dir.to_path_buf())).unwrap();
    game
}

fn reload(game: &mut Gameplay) {
    let dir = game.live_dir();
    game.set_world_dir(Some(dir)).unwrap();
}

fn start(game: &mut Gameplay, at: [f32; 3]) -> Outcome {
    game.handle_in_world(&start_request(), &character_at(at), 2_000)
        .unwrap()
}

fn complete(game: &mut Gameplay, at: [f32; 3]) -> io::Result<()> {
    game.apply_craft(
        crate::recipes::CraftingJob::Miner,
        SmeltingOutcome {
            transaction: Some(crate::smelting::SmeltingTransaction {
                consumes: Vec::new(),
                grants: Vec::new(),
                experience_points: 0,
            }),
            completed: true,
            ..Default::default()
        },
        &character_at(at),
        &mut Outcome::default(),
    )
}

#[test]
fn moved_station_changes_recipe_start_and_completion_coordinates() {
    let dir = live_dir();
    fs::create_dir_all(&dir).unwrap();
    station::write_placements(&dir, Some(station::NEAR));

    let mut game = station_game(&dir);
    assert!(game.near_craft_station(
        &character_at(station::NEAR),
        crate::recipes::CraftingJob::Miner
    ));
    station::write_placements(&dir, Some(station::AWAY));
    reload(&mut game);
    assert!(!game.near_craft_station(
        &character_at(station::NEAR),
        crate::recipes::CraftingJob::Miner
    ));
    assert!(game.near_craft_station(
        &character_at(station::AWAY),
        crate::recipes::CraftingJob::Miner
    ));

    let refused = start(&mut game, station::NEAR);
    assert!(String::from_utf8_lossy(&refused.replies.concat()).contains("Move closer"));
    assert!(
        game.smelting
            .as_ref()
            .is_none_or(|smelting| !smelting.active())
    );
    start(&mut game, station::AWAY);
    assert!(game.smelting.as_ref().is_some_and(Smelting::active));

    let mut completion_game = station_game(&dir);
    station::write_placements(&dir, Some(station::NEAR));
    reload(&mut completion_game);
    start(&mut completion_game, station::NEAR);
    station::write_placements(&dir, Some(station::AWAY));
    reload(&mut completion_game);
    let error = complete(&mut completion_game, station::NEAR).unwrap_err();
    assert!(error.to_string().contains("outside a Miner station"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn removed_station_changes_recipe_start_and_completion_coordinates() {
    let dir = live_dir();
    fs::create_dir_all(&dir).unwrap();
    station::write_placements(&dir, Some(station::NEAR));

    let mut game = station_game(&dir);
    assert!(game.near_craft_station(
        &character_at(station::NEAR),
        crate::recipes::CraftingJob::Miner
    ));
    station::write_placements(&dir, None);
    reload(&mut game);
    assert!(!game.near_craft_station(
        &character_at(station::NEAR),
        crate::recipes::CraftingJob::Miner
    ));
    let refused = start(&mut game, station::NEAR);
    assert!(String::from_utf8_lossy(&refused.replies.concat()).contains("Move closer"));
    assert!(
        game.smelting
            .as_ref()
            .is_none_or(|smelting| !smelting.active())
    );

    let mut completion_game = station_game(&dir);
    station::write_placements(&dir, Some(station::NEAR));
    reload(&mut completion_game);
    start(&mut completion_game, station::NEAR);
    station::write_placements(&dir, None);
    reload(&mut completion_game);
    let error = complete(&mut completion_game, station::NEAR).unwrap_err();
    assert!(error.to_string().contains("outside a Miner station"));

    let _ = fs::remove_dir_all(dir);
}
