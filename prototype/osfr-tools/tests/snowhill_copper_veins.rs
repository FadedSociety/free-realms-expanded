//! T547: Snowhill's copper veins are the client's own vein network.
//!
//! Sanctuary-main-20261019 places 17 copper-vein actors across Snowhill (`Resources/Npcs.json`
//! Ids 33585..36303, NameId 7228, ModelId 4019 `sg_mining_node_snowhillcopper_01.adr`, Scale
//! 0.5, TextureAlias "Copper", CursorId 20; `Scripts/Zone/FabledRealms.lua:3807..4266`). This
//! suite pins that import, the AddNpc appearance it draws (35/2 body reader `00921a10`), the
//! ordinary 26/8 gather and Mining18 payout (Copper Ore 2739), the non-Miner refusal, two
//! players on one vein, and the 3 m terrain check against the client ground grid.
//!
//! The synthetic one-goal Mining18 board, the opening fixture and the packet builders come from
//! `tests/support/mining_quest_chain.rs` and `tests/support/npc_quest_chain.rs`. Node content is
//! the repository's `data/content`: the opening package carries no NPC tree, so `Gameplay::load`
//! binds the bundled set (`content::npc_content`). Terrain is `tests/support/terrain.rs`.

#[path = "support/json.rs"]
mod json;
#[path = "support/mining_quest_chain.rs"]
mod mining_quest_chain;
#[path = "support/models.rs"]
mod models;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/terrain.rs"]
mod terrain;

use mining_quest_chain::{
    Fixture, LEGACY_COPPER_MODEL, active_mining, copper_nodes, copper_ore, gather_copper,
    nodes_of_model,
};
use models::read_models;
use npc_quest_chain::{
    ADVENTURER, CHARACTER, character_at_for, content_dir, npc_interact, saved, send, visible_npcs,
};
use osfr_tools::{
    gameplay::Gameplay, harvest_nodes::NodeActor, mining::MINER_PROFILE_ID, startup_world::NpcSpawn,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// `Resources/Models.txt` 4019 = `sg_mining_node_snowhillcopper_01.adr`.
const COPPER_MODEL: u32 = 4_019;
const VEIN_TYPE: &str = "snowhill-copper-vein";
const VEIN_POOL: &str = "snowhill-copper-world";
/// The 17 copper template ids (ModelId 4019) in `Resources/Npcs.json`, ascending. Each is the
/// content spawn id of its `spawnNpcWithGuid` row.
const VEIN_IDS: [u32; 17] = [
    33_585, 33_587, 33_608, 33_616, 33_629, 33_738, 33_771, 33_820, 33_821, 33_860, 33_939, 33_940,
    33_999, 34_010, 34_012, 36_116, 36_303,
];

/// The content spawn id a node guid carries: "NOD", the mining file's tag byte (2, the
/// `NODE_FILES` entry in `harvest_nodes.rs`), then the spawn id.
fn spawn_id(guid: u64) -> u32 {
    u32::try_from(guid & 0xffff_ffff).expect("a spawn id fits u32")
}

fn vein_content() -> Value {
    json::read(&content_dir().join("mining_nodes.json"))
}

/// Every spawn row of the new pool: (Sanctuary id, position, heading), ascending by id.
fn vein_spawns() -> Vec<(u32, [f32; 3], f32)> {
    let content = vein_content();
    let mut rows: Vec<(u32, [f32; 3], f32)> = content["spawns"]
        .as_array()
        .expect("node spawns")
        .iter()
        .filter(|row| row["pool_key"] == VEIN_POOL)
        .map(|row| {
            (
                row["id"].as_u64().expect("a spawn id") as u32,
                serde_json::from_value(row["position"].clone()).expect("a spawn position"),
                row["heading"].as_f64().expect("a heading") as f32,
            )
        })
        .collect();
    rows.sort_by_key(|(id, _, _)| *id);
    rows
}

/// The AddNpc bodies the production visibility batch sends while a session walks `actors`.
fn drawn(game: &mut Gameplay, actors: &[NodeActor]) -> BTreeMap<u64, NpcSpawn> {
    let mut spawns = BTreeMap::new();
    for actor in actors {
        let character = character_at_for(CHARACTER, 0, actor.position);
        for spawn in visible_npcs(game, &character) {
            spawns.insert(spawn.actor_id, spawn);
        }
    }
    spawns
}

#[test]
fn the_import_carries_the_seventeen_recovered_veins() {
    let content = vein_content();
    let types = content["node_types"].as_array().expect("node types");
    let vein = types
        .iter()
        .find(|row| row["key"] == VEIN_TYPE)
        .expect("the T547 vein type");
    assert_eq!(vein["name"], "Copper Vein");
    assert_eq!(vein["model_id"], json!(COPPER_MODEL));
    assert_eq!(vein["scale"], json!(0.5));
    assert_eq!(vein["texture_alias"], "Copper");
    assert_eq!(vein["cursor_id"], json!(20));
    assert_eq!(vein["interact_range"], json!(12));
    assert_eq!(vein["placement_y_offset"], json!(0.0));
    assert_eq!(
        vein["drop_table"].as_array().map(Vec::len),
        Some(0),
        "the Mining18 board pays, not a node table"
    );
    let old = types
        .iter()
        .find(|row| row["key"] == "copper-vein")
        .expect("the reconstructed copper-vein type");
    assert_eq!(old["model_id"], json!(LEGACY_COPPER_MODEL));

    let pools = content["pools"].as_array().expect("pools");
    let pool = pools
        .iter()
        .find(|row| row["key"] == VEIN_POOL)
        .expect("the world pool");
    assert_eq!(pool["zone_definition_id"], json!(13));
    assert_eq!(pool["node_type"], VEIN_TYPE);
    assert_eq!(pool["max_active_nodes"], json!(8));
    assert_eq!(pool["respawn_seconds"], json!(60));
    let old_pool = pools
        .iter()
        .find(|row| row["key"] == "snowhill-copper")
        .expect("the old pool");
    assert_eq!(old_pool["node_type"], "copper-vein");
    assert_eq!(old_pool["max_active_nodes"], json!(4));

    let spawns = content["spawns"].as_array().expect("spawns");
    let mut all_ids: Vec<u32> = spawns
        .iter()
        .map(|row| row["id"].as_u64().expect("a spawn id") as u32)
        .collect();
    all_ids.sort_unstable();
    all_ids.dedup();
    assert_eq!(all_ids.len(), spawns.len(), "two node rows share an id");

    let rows: Vec<(u32, [f32; 3], f32)> = vein_spawns();
    let ids: Vec<u32> = rows.iter().map(|(id, _, _)| *id).collect();
    assert_eq!(ids, VEIN_IDS);
    for row in spawns.iter().filter(|row| row["pool_key"] == VEIN_POOL) {
        let id = row["id"].as_u64().expect("a spawn id");
        let evidence = row["source_tags"]["row"]
            .as_str()
            .unwrap_or_else(|| panic!("spawn {id} carries no evidence"));
        assert!(
            evidence.contains("FabledRealms.lua:") && evidence.contains(&id.to_string()),
            "spawn {id} does not name its lua line: {evidence}"
        );
        assert_eq!(row["zone_definition_id"], json!(13));
        // Position and heading are the lua row's own numbers (the heading is content only: node
        // actors send no facing).
        let lua_args = format!(
            "{}, {}, {}, {}",
            row["position"][0], row["position"][1], row["position"][2], row["heading"]
        );
        assert!(
            evidence.contains(&lua_args),
            "spawn {id} differs from its lua row {lua_args}: {evidence}"
        );
    }
    assert_eq!(
        spawns
            .iter()
            .filter(|row| row["pool_key"] == "snowhill-copper")
            .count(),
        5,
        "the five reconstructed copper spawns stay"
    );
    for id in 1..=5u32 {
        assert!(all_ids.contains(&id), "the old spawn {id} stays");
        assert!(!VEIN_IDS.contains(&id), "the old spawn {id} aliases a vein");
    }
    assert!(rows.iter().all(|(_, position, heading)| {
        position
            .iter()
            .chain(std::iter::once(heading))
            .all(|value| value.is_finite())
    }));

    // Node guids: "NOD", the mining tag byte (2), then the content spawn id. The old pool's ids
    // 1..5 and the 17 Sanctuary ids cannot collide, and the pool cap places eight of seventeen.
    let veins = nodes_of_model(COPPER_MODEL);
    assert_eq!(veins.len(), 8, "max_active_nodes rows are placed");
    for actor in &veins {
        assert_eq!((actor.guid >> 32) & 0xff, 2, "the mining node tag");
        assert!(VEIN_IDS.contains(&spawn_id(actor.guid)));
    }
    let old = nodes_of_model(LEGACY_COPPER_MODEL);
    assert_eq!(old.len(), 4, "the old pool's max_active_nodes rows");
    // The Miner chapter suites click `copper_nodes()`: it stays the legacy pool, not the veins.
    assert_eq!(
        copper_nodes(),
        old.iter()
            .map(|actor| (actor.guid, actor.position))
            .collect::<Vec<_>>()
    );
    assert!(
        old.iter()
            .all(|actor| (1..=5).contains(&spawn_id(actor.guid)))
    );
}

#[test]
fn the_world_draws_the_veins_with_the_client_model_scale_and_texture() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER_PROFILE_ID, &[ADVENTURER, MINER_PROFILE_ID]);
    let veins = nodes_of_model(COPPER_MODEL);
    assert_eq!(veins.len(), 8);
    // The old pool still draws the generic ore pile at the captured template scale and textures.
    let old = nodes_of_model(LEGACY_COPPER_MODEL);
    assert_eq!(old.len(), 4);
    let mut walked = veins.clone();
    walked.extend(old.iter().cloned());
    let spawns = drawn(&mut game, &walked);
    for vein in &veins {
        let spawn = spawns
            .get(&vein.guid)
            .unwrap_or_else(|| panic!("vein {:#x} is not drawn", vein.guid));
        assert_eq!(
            spawn.model_id, COPPER_MODEL as i32,
            "Models.txt 4019 sg_mining_node_snowhillcopper_01.adr"
        );
        assert_eq!(spawn.scale, 0.5, "Sanctuary Scale 0.5");
        assert_eq!(spawn.texture_alias, "Copper", "Sanctuary TextureAlias");
        assert_eq!(spawn.is_interactable, 1);
        assert_eq!(
            spawn.position,
            [vein.position[0], vein.position[1], vein.position[2], 1.0]
        );
    }
    for vein in &old {
        let spawn = spawns
            .get(&vein.guid)
            .unwrap_or_else(|| panic!("old node {:#x} is not drawn", vein.guid));
        assert_eq!(spawn.model_id, LEGACY_COPPER_MODEL as i32);
        assert_eq!(spawn.scale, 1.0);
        assert_eq!(spawn.texture_alias, "");
    }
}

#[test]
fn a_miner_clicks_a_recovered_vein_and_is_paid_copper_ore() {
    let fixture = Fixture::new();
    let vein = nodes_of_model(COPPER_MODEL).remove(0);
    let mut game = fixture.game(MINER_PROFILE_ID, &[ADVENTURER, MINER_PROFILE_ID]);
    let character = character_at_for(CHARACTER, 0, vein.position);
    assert_eq!(copper_ore(&game), 0);
    let mut now = 1_000;
    assert!(
        gather_copper(&mut game, &character, vein.guid, vein.position, &mut now).is_some(),
        "the recovered vein starts a Mining18 round"
    );
    assert_eq!(copper_ore(&game), 1, "the board pays Copper Ore 2739");
    assert_eq!(active_mining(&game), None, "the paid run closed");
}

#[test]
fn a_non_miner_click_is_refused_and_changes_nothing() {
    let fixture = Fixture::new();
    let vein = nodes_of_model(COPPER_MODEL).remove(0);
    let mut game = fixture.game(ADVENTURER, &[ADVENTURER]);
    let character = character_at_for(CHARACTER, 0, vein.position);
    let before = saved(&game);
    let before_save = fixture.saved_bytes(&game, &character);

    let refused = send(&mut game, &character, &npc_interact(vein.guid), 1_000);
    assert!(!refused.changed, "a non-Miner click changed gameplay");
    assert!(!refused.replies.is_empty(), "the refusal was silent");
    assert_eq!(saved(&game), before, "a non-Miner click changed the state");
    assert_eq!(
        fixture.saved_bytes(&game, &character),
        before_save,
        "a non-Miner click changed the save"
    );
    assert_eq!(active_mining(&game), None, "a non-Miner started a run");
    assert_eq!(copper_ore(&game), 0);
}

#[test]
fn one_players_mined_vein_stays_a_vein_for_the_other() {
    let fixture = Fixture::new();
    let vein = nodes_of_model(COPPER_MODEL).remove(0);
    let ids = [CHARACTER, CHARACTER + 1];
    // Both sessions are live before either mines, so shared node state would show.
    let mut games =
        ids.map(|id| fixture.game_as(id, MINER_PROFILE_ID, &[ADVENTURER, MINER_PROFILE_ID]));
    let characters = ids.map(|id| character_at_for(id, 0, vein.position));
    let mut now = 1_000;
    assert!(
        gather_copper(
            &mut games[0],
            &characters[0],
            vein.guid,
            vein.position,
            &mut now
        )
        .is_some(),
        "player one starts the run"
    );
    assert_eq!(copper_ore(&games[0]), 1, "player one is paid");
    // Player one's paid vein is gone for player one only (a personal node, as today).
    now += 1_000;
    send(&mut games[0], &characters[0], &npc_interact(vein.guid), now);
    assert_eq!(
        active_mining(&games[0]),
        None,
        "the spent vein started a second run"
    );
    assert_eq!(copper_ore(&games[0]), 1);
    assert_eq!(
        copper_ore(&games[1]),
        0,
        "player two was paid by player one's run"
    );
    assert!(
        gather_copper(
            &mut games[1],
            &characters[1],
            vein.guid,
            vein.position,
            &mut now
        )
        .is_some(),
        "player two still finds the vein"
    );
    assert_eq!(copper_ore(&games[1]), 1, "player two is paid");
    assert_eq!(copper_ore(&games[0]), 1, "player two's run paid player one");
}

#[test]
fn every_recovered_vein_stands_on_the_client_terrain() {
    let terrain = terrain::load();
    let grid = terrain.grid(0).expect("the zone-0 ground grid");
    let rows = vein_spawns();
    assert_eq!(rows.len(), 17);
    for (id, position, _) in rows {
        let ground = grid
            .height_at(position[0], position[2])
            .unwrap_or_else(|| panic!("zone-0 terrain covers vein {id}"));
        let error = (position[1] - ground).abs();
        assert!(
            error <= 3.0,
            "vein {id} at {position:?} is {error:.2} m off the client terrain"
        );
    }
}

#[test]
fn the_client_names_the_model_the_import_uses() {
    // `Resources/Cursors.txt` 20 = `cursor_interaction_mining.cur` (checked when importing;
    // `cursor_id` is loaded but not sent, so no gate depends on it).
    let models = read_models();
    let files = models
        .get(&i64::from(COPPER_MODEL))
        .expect("Models.txt row 4019");
    assert!(
        files.contains("sg_mining_node_snowhillcopper_01.adr"),
        "Models.txt 4019 is the Snowhill copper vein: {files:?}"
    );
}
