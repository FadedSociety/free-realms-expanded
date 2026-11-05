//! L934: the shipped Type8 boards (mining, harvesting, mail sorting) against the evidence they
//! cite: each ore row pays the ore its bar smelts, Mining Practice pays nothing, every game an
//! engine offers has a Games window row, and the Miner rank floors are the live rows' sets.

use osfr_tools::{
    activity_portal::load_live,
    harvesting, mail_sorting,
    mining::{self, Type8Engine},
    recipes::{CraftingJob, RecipeCatalog},
};
use std::{collections::BTreeMap, path::PathBuf};

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn release() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release")
}

/// (MiniGameData mining row, smelting row of its bar): Copper, Tin, Iron, Silver, Gold.
const ORE_GAMES: [(i32, u32); 5] = [(91, 77), (93, 134), (95, 135), (97, 136), (99, 137)];

fn mining_engine() -> Type8Engine {
    Type8Engine::load(
        &content(),
        mining::CONTENT_FILE,
        mining::MINING_GAME_ID,
        mining::FIRST_MINING_INSTANCE,
        None,
    )
    .expect("the shipped mining file loads")
}

#[test]
fn each_ore_row_pays_the_ore_its_bar_smelts() {
    let recipes = RecipeCatalog::load_runtime(&content()).unwrap();
    let engine = mining_engine();
    let inventory = osfr_tools::inventory::Inventory::load(&release(), None).unwrap();
    // The ore is the first ingredient of the Miner recipe whose smelting game is the pair's.
    let smelts: BTreeMap<u32, i32> = recipes
        .records
        .iter()
        .filter(|recipe| recipe.job == CraftingJob::Miner)
        .filter_map(|recipe| {
            Some((
                recipe.minigame_id?,
                recipe.ingredients.first()?.definition_id,
            ))
        })
        .collect();
    for (game, smelting) in ORE_GAMES {
        let ore = smelts[&smelting];
        let spec = engine.spec_for(Some(game)).expect("the ore row is offered");
        assert!(!spec.outputs.is_empty(), "game {game} pays nothing");
        for output in &spec.outputs {
            assert_eq!(output.definition_id, ore, "game {game} pays another ore");
        }
        assert!(inventory.has_definition(ore), "ore {ore} has no definition");
    }
    // Five distinct ores, one per row.
    let distinct: std::collections::BTreeSet<_> =
        ORE_GAMES.iter().map(|(_, s)| smelts[s]).collect();
    assert_eq!(distinct.len(), 5);
}

#[test]
fn mining_practice_has_no_outputs_and_the_node_game_is_copper() {
    let engine = mining_engine();
    let practice = engine
        .spec_for(Some(375))
        .expect("Mining Practice is offered");
    assert!(practice.outputs.is_empty(), "practice pays no ore");
    assert!(
        !practice.goals.is_empty(),
        "practice still has a board to play"
    );
    let node = engine.spec_for(None).unwrap();
    assert!(
        node.outputs
            .iter()
            .all(|output| output.definition_id == 2739)
    );
}

/// Every game a Type8 file offers, except the node game 18, has a live Games window row (the
/// client's `content_id`), so Play can start it. The ore rows' RequirementSets are the rank floors.
#[test]
fn every_offered_game_has_a_games_window_row_and_ore_rows_carry_their_rank_sets() {
    let rows = load_live(&content()).unwrap();
    let engines = [
        mining_engine(),
        Type8Engine::load(
            &content(),
            harvesting::CONTENT_FILE,
            111,
            harvesting::FIRST_INSTANCE_ID,
            None,
        )
        .unwrap(),
        Type8Engine::load(
            &content(),
            mail_sorting::CONTENT_FILE,
            mail_sorting::GAME_ID,
            mail_sorting::FIRST_INSTANCE_ID,
            None,
        )
        .unwrap(),
    ];
    for engine in &engines {
        for (game, _) in engine.games() {
            if game == mining::MINING_GAME_ID {
                continue;
            }
            assert!(
                rows.iter()
                    .any(|row| row.entry_type == 2 && row.content_id == game as u32),
                "game {game} has no Games window row"
            );
        }
    }
    // RequirementSets 15-18 are the Miner ranks 5, 9, 13 and 17 (Requirements 17-20); the sets
    // go 15, 16, 17, 18 up Tin, Iron, Silver and Gold.
    for (game, set) in [(93, 15), (95, 16), (97, 17), (99, 18)] {
        let row = rows
            .iter()
            .find(|row| row.entry_type == 2 && row.content_id == game)
            .unwrap();
        assert_eq!(row.recommended_set_id, set, "game {game}");
        assert_eq!(
            mining::rank_floor(game as i32),
            5 + 4 * (set - 15),
            "game {game}"
        );
    }
    for free in [18, 91, 375] {
        assert_eq!(mining::rank_floor(free), 0, "game {free}");
    }
}
