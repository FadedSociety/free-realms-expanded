//! P-quest-games: the games and data that held back the Postman, Miner, Chef and Blacksmith
//! quests. The rich ore rows (MiniGameData 92-100) pay their ore and the client's own bonus gem,
//! their veins stand at Dartmoor and Snorg's camp and may name only a row the board offers,
//! Biting Back (301) is Loryn's catch of the PCA Tricksters, the wand recipes run their own
//! forging rows and the two Chef recipes with no id have theirs.
//!
//! Each guard reads the shipped `data/content`; the play paths are `tests/gathering_nodes.rs`
//! (the rich vein) and `src/gameplay/world_challenge/tests.rs` (243 and 301).

use osfr_tools::{
    harvest_nodes::GatheringNodes,
    mining::{self, Type8Engine},
    recipes::RecipeCatalog,
    world_challenge::{Challenges, Kind},
};
use std::{fs, path::PathBuf};

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn engine() -> Type8Engine {
    Type8Engine::load(
        &content(),
        mining::CONTENT_FILE,
        mining::MINING_GAME_ID,
        mining::FIRST_MINING_INSTANCE,
        None,
    )
    .expect("the shipped mining board loads")
}

/// (row, its plain row, ore definition, gem definition, the gem's bonus tile): the client's
/// `game_mining` TileGenerator draws bonus tile 700 + n as `aBonusNames[n]`, Tourmaline, Topaz,
/// Onyx, Moonstone, Turquoise, Amethyst, Garnet, Ruby, Fireagate, Malachite.
const RICH_ROWS: [(i32, i32, i32, i32, u16); 5] = [
    (92, 91, 2739, 2746, 709),
    (94, 93, 2740, 2745, 706),
    (96, 95, 2741, 2753, 705),
    (98, 97, 2742, 2749, 707),
    (100, 99, 2743, 2744, 708),
];

#[test]
fn the_rich_ore_rows_pay_their_ore_and_the_clients_bonus_gem() {
    let engine = engine();
    for (row, plain, ore, gem, tile) in RICH_ROWS {
        let spec = engine
            .spec_for(Some(row))
            .unwrap_or_else(|| panic!("mining.json offers row {row}"));
        let output = |definition: i32| {
            spec.outputs
                .iter()
                .find(|output| output.definition_id == definition)
                .unwrap_or_else(|| panic!("row {row} pays {definition}"))
        };
        let plain_spec = engine.spec_for(Some(plain)).expect("the plain row");
        assert_eq!(
            output(ore).value,
            plain_spec
                .outputs
                .iter()
                .find(|o| o.definition_id == ore)
                .expect("the plain row's ore")
                .value,
            "row {row} drops its ore on the plain row's tile"
        );
        assert_eq!(output(gem).value, tile, "row {row}'s gem is the client's bonus tile");
        assert!(
            spec.collectable.contains(&tile) && spec.tiles.iter().any(|t| t.value == tile),
            "row {row}'s gem tile is on the board"
        );
        assert!(
            spec.goals.iter().any(|goal| goal.value == tile && goal.quota == 0),
            "row {row}'s gem is a bonus goal, never required for the win"
        );
        assert_eq!(
            mining::rank_floor(row),
            mining::rank_floor(plain),
            "row {row} asks its ore's Miner rank"
        );
        assert!(mining::is_ore_row(row) && mining::is_ore_row(plain));
    }
    assert!(!mining::is_ore_row(mining::MINING_GAME_ID) && !mining::is_ore_row(375));
}

#[test]
fn every_rich_vein_starts_a_row_the_board_offers() {
    let engine = engine();
    let nodes = GatheringNodes::load(&content())
        .expect("node content loads")
        .expect("node content is shipped");
    let mut started = Vec::new();
    for actor in nodes.actors() {
        if let Some(game) = nodes.game_id(actor.guid)
            && actor.name.ends_with("Vein")
        {
            assert!(
                engine.spec_for(Some(game)).is_some(),
                "{} starts row {game}, which mining.json offers",
                actor.name
            );
            started.push(game);
        }
    }
    started.sort_unstable();
    started.dedup();
    assert_eq!(
        started,
        RICH_ROWS.map(|(row, ..)| row).to_vec(),
        "a vein stands for every rich row"
    );
}

/// The loader accepts any of the client's ore rows (91-100) for a Miner pool and refuses every
/// other game; a row the board does not offer is refused later, at the click
/// (`tests/gathering_nodes.rs`).
#[test]
fn a_miner_pool_naming_a_game_that_is_not_an_ore_row_is_refused_at_load() {
    let scratch = std::env::temp_dir().join(format!(
        "osfr-quest-games-{}-{}",
        std::process::id(),
        line!()
    ));
    fs::create_dir_all(&scratch).unwrap();
    let mut doc: serde_json::Value =
        serde_json::from_slice(&fs::read(content().join("mining_nodes.json")).unwrap()).unwrap();
    let pool = doc["pools"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|pool| pool["key"] == "dartmoor-rich-copper")
        .expect("the Dartmoor rich copper pool");
    // Cooking's 46 is a real game, but no vein starts it.
    pool["game_id"] = serde_json::json!(46);
    fs::write(
        scratch.join("mining_nodes.json"),
        serde_json::to_vec(&doc).unwrap(),
    )
    .unwrap();
    let refused = GatheringNodes::load(&scratch);
    let _ = fs::remove_dir_all(&scratch);
    assert!(
        refused.is_err(),
        "a Miner pool may only name the client's ore rows"
    );
}

#[test]
fn the_wand_recipes_run_their_own_forging_rows_and_the_chef_pots_have_recipe_ids() {
    let recipes = RecipeCatalog::load_runtime(&content()).expect("recipes load");
    let recipe = |key: &str| recipes.by_key(key).unwrap_or_else(|| panic!("recipe {key}"));
    let tempest = recipe("blacksmith.tempest_woven_wand");
    assert_eq!((tempest.recipe_id, tempest.minigame_id), (Some(200), Some(367)));
    assert_eq!(tempest.output.definition_id, 30480, "the wand the quest hands over");
    let void = recipe("blacksmith.void_woven_wand");
    assert_eq!((void.recipe_id, void.minigame_id), (Some(202), Some(369)));
    let eclipse = recipe("blacksmith.eclipse_infused_wand");
    assert_eq!(eclipse.minigame_id, Some(365), "the Eclipse row keeps its own game");
    let roast = recipe("chef.mystery_pot_roast");
    assert_eq!(
        (roast.recipe_id, roast.recipe_item_definition_id, roast.minigame_id),
        (Some(40), Some(850), Some(180))
    );
    let surprise = recipe("chef.briarmelon_surprise");
    assert_eq!(
        (surprise.recipe_id, surprise.recipe_item_definition_id, surprise.minigame_id),
        (Some(58), Some(868), Some(198))
    );
    let mut ids: Vec<u32> = recipes.records.iter().filter_map(|r| r.recipe_id).collect();
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count, "recipe ids stay unique");
}

#[test]
fn biting_back_is_loryns_catch_of_the_tricksters_and_bixie_bud_blitz_is_honeys() {
    let games = Challenges::load(&content())
        .expect("world challenges load")
        .expect("world challenges are shipped");
    let biting = games.get(301).expect("Biting Back is served");
    assert_eq!((biting.giver, biting.profile), (9_000_000_132, 4), "Loryn, for the Postman");
    assert_eq!(biting.stages.len(), 1);
    assert_eq!(biting.stages[0].kind, Kind::Catch);
    assert_eq!(biting.stages[0].model_id, 489, "the PCA Trickster body (as PCA Takedown's Campers)");
    assert_eq!(biting.stages[0].label_id, 130_764, "the things read 'PCA Trickster'");
    let buds = games.get(243).expect("Bixie Bud Blitz is served");
    assert_eq!(buds.giver, 9_000_000_107, "Honey");
    assert_eq!(
        buds.stages.iter().map(|s| s.kind).collect::<Vec<_>>(),
        vec![Kind::Collect, Kind::Return]
    );
}
