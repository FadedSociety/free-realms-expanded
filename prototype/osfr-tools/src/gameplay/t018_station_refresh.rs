use super::*;
use crate::world::{PLACEMENT_GUID_BASE, Placement, PlacementFile};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const STATION: i32 = 990001;
const FIRST: [f32; 3] = [321.0, 0.0, 654.0];
const SECOND: [f32; 3] = [1_321.0, 0.0, 1_654.0];

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

fn placement(position: [f32; 3]) -> Placement {
    Placement {
        guid: PLACEMENT_GUID_BASE + 1,
        model_id: 1733,
        name_string_id: 0,
        name: None,
        position,
        heading: 0.0,
        scale: 1.0,
        extra: serde_json::Map::new(),
    }
}

fn write_placements(dir: &Path, positions: &[[f32; 3]]) {
    let records = positions.iter().copied().map(placement).collect();
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

fn start_request() -> Vec<u8> {
    let mut packet = vec![26, 0, 24, 0];
    packet.extend(STATION.to_le_bytes());
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
    game.apply_smelting(
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
    write_placements(&dir, &[FIRST]);

    let mut game = station_game(&dir);
    assert!(game.near_craft_station(&character_at(FIRST), crate::recipes::CraftingJob::Miner));
    write_placements(&dir, &[SECOND]);
    reload(&mut game);
    assert!(!game.near_craft_station(&character_at(FIRST), crate::recipes::CraftingJob::Miner));
    assert!(game.near_craft_station(&character_at(SECOND), crate::recipes::CraftingJob::Miner));

    let refused = start(&mut game, FIRST);
    assert!(String::from_utf8_lossy(&refused.replies.concat()).contains("Move closer"));
    assert!(
        game.smelting
            .as_ref()
            .is_none_or(|smelting| !smelting.active())
    );
    start(&mut game, SECOND);
    assert!(game.smelting.as_ref().is_some_and(Smelting::active));

    let mut completion_game = station_game(&dir);
    write_placements(&dir, &[FIRST]);
    reload(&mut completion_game);
    start(&mut completion_game, FIRST);
    write_placements(&dir, &[SECOND]);
    reload(&mut completion_game);
    let error = complete(&mut completion_game, FIRST).unwrap_err();
    assert!(error.to_string().contains("outside a Miner station"));

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn removed_station_changes_recipe_start_and_completion_coordinates() {
    let dir = live_dir();
    fs::create_dir_all(&dir).unwrap();
    write_placements(&dir, &[FIRST]);

    let mut game = station_game(&dir);
    assert!(game.near_craft_station(&character_at(FIRST), crate::recipes::CraftingJob::Miner));
    write_placements(&dir, &[]);
    reload(&mut game);
    assert!(!game.near_craft_station(&character_at(FIRST), crate::recipes::CraftingJob::Miner));
    let refused = start(&mut game, FIRST);
    assert!(String::from_utf8_lossy(&refused.replies.concat()).contains("Move closer"));
    assert!(
        game.smelting
            .as_ref()
            .is_none_or(|smelting| !smelting.active())
    );

    let mut completion_game = station_game(&dir);
    write_placements(&dir, &[FIRST]);
    reload(&mut completion_game);
    start(&mut completion_game, FIRST);
    write_placements(&dir, &[]);
    reload(&mut completion_game);
    let error = complete(&mut completion_game, FIRST).unwrap_err();
    assert!(error.to_string().contains("outside a Miner station"));

    let _ = fs::remove_dir_all(dir);
}
