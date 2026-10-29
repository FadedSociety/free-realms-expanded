//! T553: the client's farm plots start the regional Harvesting games.
//!
//! Sanctuary-main-20261019 places 13 plot actors (`Resources/Npcs.json` ModelId 274
//! `sg_farming_node_common_01.adr`, CursorId 18, `Scripts/Zone/FabledRealms.lua`). MiniGameData
//! rows 109, 115 and 117 (type 8) are the Queensfields, Perry's Pastures and Lakesfield
//! Harvesting games. This suite pins the import, the pool-to-game binding and its load refusals,
//! and the shipped `harvesting.json` `games` rows against the recovered produce sets. The
//! gateway half (a Chef's 26/8 click, the region's name and produce, refusals, the mushrooms
//! staying on game 111) is in `gathering_nodes.rs`, which owns the node fixture.

#[path = "support/json.rs"]
mod json;
#[path = "support/models.rs"]
mod models;
#[path = "support/node_content.rs"]
mod node_content;
#[path = "support/scratch.rs"]
mod scratch;
#[path = "support/terrain.rs"]
mod terrain;

use json::read;
use node_content::{content_dir, shipped_nodes};
use osfr_tools::{harvest_nodes::GatheringNodes, harvesting, mining::Type8Engine};
use scratch::Scratch;
use serde_json::{Value, json};
use std::collections::BTreeSet;

const FILE: &str = "harvest_nodes.json";
const PLOT_MODEL: u32 = 274;
/// (pool, MiniGameData game, plot count): `Npcs.json` NameId 100833 / 100836 / 100837.
const POOLS: [(&str, i64, usize); 3] = [
    ("queensfields", 109, 1),
    ("perrys-pastures", 115, 6),
    ("lakesfield", 117, 6),
];

fn plots() -> Vec<Value> {
    let content = read(&content_dir().join(FILE));
    content["spawns"]
        .as_array()
        .expect("node spawns")
        .iter()
        .filter(|row| POOLS.iter().any(|(pool, _, _)| row["pool_key"] == *pool))
        .cloned()
        .collect()
}

#[test]
fn the_import_carries_the_thirteen_recovered_plots() {
    let content = read(&content_dir().join(FILE));
    let plot = content["node_types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["key"] == "farm-plot")
        .expect("the farm-plot type");
    assert_eq!(plot["name"], "Harvesting");
    assert_eq!(plot["model_id"], json!(PLOT_MODEL));
    assert_eq!(plot["scale"], json!(1.0));
    assert_eq!(plot["cursor_id"], json!(18));
    assert_eq!(plot["interact_range"], json!(12));
    assert_eq!(plot["placement_y_offset"], json!(0.0));
    assert_eq!(plot["composite_effect_id"], json!(0));
    assert_eq!(plot["drop_table"], json!([]), "the Harvesting board pays");

    let mushrooms = content["pools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["key"] == "briarwood-mushrooms")
        .unwrap();
    assert!(
        mushrooms.get("game_id").is_none(),
        "the mushrooms keep the engine's own game"
    );
    for (key, game, count) in POOLS {
        let pool = content["pools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["key"] == key)
            .unwrap_or_else(|| panic!("pool {key}"));
        assert_eq!(pool["node_type"], "farm-plot");
        assert_eq!(pool["game_id"], json!(game));
        assert_eq!(pool["max_active_nodes"], json!(count), "{key}");
        assert_eq!(
            pool["respawn_seconds"], mushrooms["respawn_seconds"],
            "{key}"
        );
        let rows: Vec<_> = plots()
            .into_iter()
            .filter(|row| row["pool_key"] == key)
            .collect();
        assert_eq!(rows.len(), count, "{key} spawn rows");
    }

    let rows = plots();
    assert_eq!(rows.len(), 13);
    let ids: BTreeSet<u64> = rows.iter().map(|row| row["id"].as_u64().unwrap()).collect();
    assert_eq!(ids.len(), 13, "two plots share an id");
    let all: Vec<u64> = content["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap())
        .collect();
    assert_eq!(
        all.iter().collect::<BTreeSet<_>>().len(),
        all.len(),
        "a plot id aliases another node"
    );
    for row in &rows {
        let id = row["id"].as_u64().unwrap();
        let evidence = row["source_tags"]["row"]
            .as_str()
            .unwrap_or_else(|| panic!("plot {id} carries no evidence"));
        assert!(
            evidence.contains("FabledRealms.lua:") && evidence.contains("ModelId 274"),
            "plot {id}: {evidence}"
        );
        // Position and heading are the lua row's own numbers (the heading is content only).
        let lua_args = format!(
            "spawnNpcWithGuid({id}, 1000000{id:05}, {}, {}, {}, {})",
            row["position"][0], row["position"][1], row["position"][2], row["heading"]
        );
        assert!(
            evidence.contains(&lua_args),
            "plot {id} differs from its lua row {lua_args}: {evidence}"
        );
        let terrain_id = row["terrain_object_id"].as_i64().expect("a terrain id");
        assert!(
            row["source_tags"]["terrain_object_id"]
                .as_str()
                .unwrap()
                .contains(&format!("TerrainObjectId {terrain_id}")),
            "plot {id} terrain id"
        );
    }
}

#[test]
fn every_plot_is_placed_and_names_its_regions_game() {
    let nodes = shipped_nodes();
    let placed: Vec<_> = nodes
        .actors()
        .filter(|actor| actor.model_id == PLOT_MODEL)
        .collect();
    assert_eq!(placed.len(), 13, "every recovered plot is active");
    for (_, game, count) in POOLS {
        let of_game: Vec<_> = placed
            .iter()
            .filter(|actor| nodes.game_id(actor.guid) == Some(game as i32))
            .collect();
        assert_eq!(of_game.len(), count, "game {game}");
    }
    for actor in placed {
        assert_eq!((actor.guid >> 32) & 0xff, 1, "the harvest file's tag byte");
        // The mushrooms stay on the engine's own game.
        assert!(nodes.is_node(actor.guid) && nodes.game_id(actor.guid).is_some());
    }
    assert!(
        nodes
            .actors()
            .filter(|actor| actor.model_id != PLOT_MODEL)
            .all(|actor| nodes.game_id(actor.guid).is_none()),
        "only plots name a game"
    );
}

#[test]
fn the_client_names_the_plot_model_and_the_plots_stand_on_the_terrain() {
    // `Resources/Models.txt` 274 = `sg_farming_node_common_01.adr` ('Farming Node DPO - Common').
    let models = models::read_models();
    let files = models
        .get(&i64::from(PLOT_MODEL))
        .expect("Models.txt row 274");
    assert!(
        files.contains("sg_farming_node_common_01.adr"),
        "Models.txt 274: {files:?}"
    );
    let terrain = terrain::load();
    let grid = terrain.grid(0).expect("the zone-0 ground grid");
    for row in plots() {
        let position: [f32; 3] = serde_json::from_value(row["position"].clone()).unwrap();
        let ground = grid
            .height_at(position[0], position[2])
            .unwrap_or_else(|| panic!("zone-0 terrain covers plot {}", row["id"]));
        let error = (position[1] - ground).abs();
        assert!(
            error <= 3.0,
            "plot {} at {position:?} is {error:.2} m off the client terrain",
            row["id"]
        );
    }
}

/// A scratch copy of the node file with `edit` applied, loaded through the production loader.
fn load_edited(edit: impl FnOnce(&mut Value)) -> std::io::Result<Option<GatheringNodes>> {
    let scratch = Scratch::new("farm-plots");
    let mut content = read(&content_dir().join(FILE));
    edit(&mut content);
    json::write_pretty(&scratch.path().join(FILE), &content);
    GatheringNodes::load(scratch.path())
}

fn pool_mut<'a>(content: &'a mut Value, key: &str) -> &'a mut Value {
    content["pools"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["key"] == key)
        .unwrap()
}

#[test]
fn a_pool_must_name_a_game_the_client_knows_and_its_file_may_start() {
    // Control: the shipped file loads, and so does an edit that changes nothing.
    assert!(load_edited(|_| {}).unwrap().is_some());
    // Canary: game 999 is no MiniGameData Harvesting row, so the whole load is refused.
    let refused = load_edited(|content| {
        pool_mut(content, "queensfields")["game_id"] = json!(999);
    })
    .map(|_| ())
    .expect_err("an unknown game id must refuse the load");
    assert!(refused.to_string().contains("cannot start"), "{refused}");
    // 110 is a real MiniGameData row with no recovered produce, so it is no region either.
    assert!(
        load_edited(|content| pool_mut(content, "lakesfield")["game_id"] = json!(110)).is_err()
    );
    // A recovered region is accepted on any harvest pool, including the mushrooms.
    assert!(
        load_edited(|content| pool_mut(content, "briarwood-mushrooms")["game_id"] = json!(113))
            .unwrap()
            .is_some()
    );
}

#[test]
fn only_the_harvest_file_may_name_a_game() {
    let scratch = Scratch::new("farm-plots-mining");
    let mut content = read(&content_dir().join("mining_nodes.json"));
    json::write_pretty(&scratch.path().join("mining_nodes.json"), &content);
    assert!(GatheringNodes::load(scratch.path()).unwrap().is_some());
    pool_mut(&mut content, "snowhill-copper-world")["game_id"] = json!(109);
    json::write_pretty(&scratch.path().join("mining_nodes.json"), &content);
    assert!(
        GatheringNodes::load(scratch.path()).is_err(),
        "a mining pool named a Harvesting game"
    );
}

#[test]
fn the_shipped_harvesting_file_offers_each_plot_game_with_the_recovered_produce() {
    let directory = Scratch::new("farm-plots-harvesting");
    let shipped = content_dir().join(harvesting::CONTENT_FILE);
    std::fs::copy(&shipped, directory.path().join(harvesting::CONTENT_FILE)).unwrap();
    let engine = Type8Engine::load(
        directory.path(),
        harvesting::CONTENT_FILE,
        111,
        harvesting::FIRST_INSTANCE_ID,
        None,
    )
    .expect("the shipped harvesting file loads with its games");
    let offered: Vec<i32> = engine.games().map(|(game, _)| game).collect();
    assert_eq!(offered, [109, 111, 115, 117]);

    let file = read(&shipped);
    let base = &file["spec"];
    for row in file["games"].as_array().unwrap() {
        let game = row["game_id"].as_i64().unwrap() as i32;
        let region = harvesting::region(game).expect("a recovered region");
        let recovered = harvesting::board_spec(game).unwrap();
        // The client's names and the recovered produce (locale 94149..94154) are the
        // generator's; the quotas and bundle sizes are the file's own tuning.
        assert_eq!(row["name_id"], json!(region.name_id), "{game}");
        assert_eq!(
            row["description_id"],
            json!(region.description_id),
            "{game}"
        );
        assert_eq!(row["icon_id"], json!(region.icon_id), "{game}");
        assert_eq!(row["selectable"], json!(recovered.selectable), "{game}");
        assert_eq!(
            row["tiles"],
            serde_json::to_value(&recovered.tiles).unwrap()
        );
        for (slot, goal) in row["goals"].as_array().unwrap().iter().enumerate() {
            assert_eq!(goal["value"], json!(recovered.goals[slot].value), "{game}");
            assert_eq!(goal["icon_id"], json!(recovered.goals[slot].icon_id));
            assert_eq!(goal["quota"], base["goals"][slot]["quota"], "{game}");
            assert_eq!(goal["per_reward"], base["goals"][slot]["per_reward"]);
        }
        for (slot, output) in row["outputs"].as_array().unwrap().iter().enumerate() {
            assert_eq!(output["value"], json!(recovered.outputs[slot].value));
            assert_eq!(
                output["definition_id"],
                json!(recovered.outputs[slot].definition_id),
                "{game}"
            );
            assert_eq!(
                output["cluster_quantity"],
                base["outputs"][slot]["cluster_quantity"]
            );
        }
    }
    // Every game a plot pool names is offered, so no shipped plot is unavailable.
    let nodes = shipped_nodes();
    for actor in nodes.actors() {
        if let Some(game) = nodes.game_id(actor.guid) {
            assert!(offered.contains(&game), "plot game {game} is not offered");
        }
    }
}

/// The shipped harvesting file, edited by `edit`, loaded through the production engine loader.
fn load_engine(edit: impl FnOnce(&mut Value)) -> std::io::Result<Type8Engine> {
    let directory = Scratch::new("farm-plots-engine");
    let mut file = read(&content_dir().join(harvesting::CONTENT_FILE));
    edit(&mut file);
    json::write_pretty(&directory.path().join(harvesting::CONTENT_FILE), &file);
    Type8Engine::load(
        directory.path(),
        harvesting::CONTENT_FILE,
        111,
        harvesting::FIRST_INSTANCE_ID,
        None,
    )
}

#[test]
fn a_game_variant_that_cannot_share_the_board_refuses_the_load() {
    assert!(load_engine(|_| {}).is_ok(), "control: the shipped file");
    // The same id as the file's own game, or twice.
    assert!(load_engine(|file| file["games"][0]["game_id"] = json!(111)).is_err());
    assert!(load_engine(|file| file["games"][1]["game_id"] = json!(109)).is_err());
    // A saved run must stay valid under every game, so the goal count cannot change.
    assert!(
        load_engine(|file| {
            file["games"][0]["goals"].as_array_mut().unwrap().pop();
        })
        .is_err()
    );
    // A variant is held to the engine's own spec check: here a goal on a value that is no
    // spawnable tile.
    assert!(
        load_engine(|file| file["games"][0]["goals"][0]["value"] = json!(499)).is_err(),
        "a goal on a tile the board cannot spawn"
    );
}

#[test]
fn begin_game_switches_only_an_idle_engine_and_a_refusal_changes_nothing() {
    let mut engine = load_engine(|_| {}).unwrap();
    let chef = harvesting::CHEF_PROFILE_ID;
    assert_eq!(engine.game_id(), 111);
    assert!(engine.spec_for(Some(110)).is_none() && engine.spec_for(Some(115)).is_some());

    // An unoffered game, and a wrong job, leave the engine exactly as it was.
    let idle = engine.clone();
    assert!(engine.begin_game(Some(110), chef).is_err());
    assert!(engine.begin_game(Some(109), chef + 1).is_err());
    assert_eq!((engine.game_id(), engine.state()), (111, idle.state()));

    let started = engine.begin_game(Some(109), chef).unwrap();
    assert_eq!(engine.game_id(), 109);
    assert!(
        started.replies[0].starts_with(&[39, 0, 16]),
        "the 39/16 info"
    );
    assert_eq!(&started.replies[0][11..15], &109_i32.to_le_bytes());
    assert_eq!(&started.replies[0][15..19], &100_833_i32.to_le_bytes());

    // A run under way cannot be switched to another game, and the refusal is not a half switch.
    let running = engine.clone();
    assert!(engine.begin_game(Some(115), chef).is_err());
    assert!(engine.begin_game(None, chef).is_err());
    assert_eq!((engine.game_id(), engine.state()), (109, running.state()));

    // Once idle again, no game named is the file's own.
    engine.cancel();
    engine.begin_game(None, chef).unwrap();
    assert_eq!(engine.game_id(), 111);
}
