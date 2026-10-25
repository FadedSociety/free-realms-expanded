//! T695: Snowhill's remaining undrawn NPC rows stand at the one reference spawn that
//! names them.
//!
//! `lab/fleet/quests/talk_spawns.py` joins a regional NPC row to a FreeRealms-Legacy /
//! Sanctuary-main spawn by the client name the spawn's `Npcs.json` NameId resolves. The
//! rows this brief places record that join in `position_reason` and are `legacy_data`,
//! the only placement `world.rs` draws from `npcs/*.json`. They are sent as the ordinary
//! 35/2 AddNpc body: u64 actor, i32 name id, i32 model id, then the transform (client
//! reader `0x00921a10`). No new wire shape.

use osfr_tools::{
    startup_world::{NpcSpawn, parse_add_npc},
    world::NpcVisibility,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// (R) The join marker this brief's rows carry; the data decides which ids they are.
const SPAWN_MATCH: &str = "matched by the spawn's client name (talk_spawns.py)";
const OPEN_WORLD_ZONE: u32 = 0;

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn rows(path: &Path) -> Vec<Value> {
    read_json(path)["records"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: records", path.display()))
        .clone()
}

fn position(row: &Value) -> [f32; 3] {
    serde_json::from_value(row["position"].clone())
        .unwrap_or_else(|error| panic!("NPC {}: position: {error}", row["npc_id"]))
}

/// The rows this brief placed: `legacy_data` at a reference spawn, marked with the join.
fn placed_rows() -> Vec<Value> {
    rows(&content_dir().join("npcs/snowhill.json"))
        .into_iter()
        .filter(|row| {
            row["placement"] == "legacy_data"
                && row["position_reason"]
                    .as_str()
                    .is_some_and(|reason| reason.contains(SPAWN_MATCH))
        })
        .collect()
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// A fresh view from `position`: every actor the visibility sends, by AddNpc body.
fn seen_from(base: &NpcVisibility, position: [f32; 3]) -> Vec<NpcSpawn> {
    base.clone()
        .update(OPEN_WORLD_ZONE, position)
        .unwrap()
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .map(|packet| parse_add_npc(packet).unwrap())
        .collect()
}

#[test]
fn every_row_this_brief_placed_is_drawn_at_its_spawn_position() {
    let placed = placed_rows();
    assert!(
        !placed.is_empty(),
        "no Snowhill row records the talk_spawns spawn join"
    );
    let base = NpcVisibility::load(&content_dir()).unwrap();
    for row in &placed {
        let id = row["npc_id"].as_u64().unwrap();
        let owner = format!("{id} ({})", row["name"].as_str().unwrap_or("unnamed"));
        let expected = position(row);
        let seen = seen_from(&base, expected);
        let mine: Vec<&NpcSpawn> = seen.iter().filter(|npc| npc.actor_id == id).collect();
        assert_eq!(mine.len(), 1, "{owner}: drawn {} times", mine.len());
        let drawn_position = [
            mine[0].position[0],
            mine[0].position[1],
            mine[0].position[2],
        ];
        assert_eq!(drawn_position, expected, "{owner}: drawn position");
        assert_eq!(
            mine[0].model_id,
            row["model_id"].as_i64().unwrap() as i32,
            "{owner}: model id"
        );
        assert_eq!(
            mine[0].name_string_id,
            row["name_string_id"].as_i64().unwrap() as i32,
            "{owner}: name string id"
        );
        for other in &seen {
            if other.actor_id == id {
                continue;
            }
            let other_position = [other.position[0], other.position[1], other.position[2]];
            let gap = distance(other_position, expected);
            assert!(
                gap > 1.0,
                "{owner}: drawn actor {} stands {gap:.2} m away",
                other.actor_id
            );
        }
    }
}

#[test]
fn drawn_npc_rows_do_not_share_a_position() {
    let mut drawn = Vec::new();
    for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        for row in rows(&path) {
            if row["placement"] != "legacy_data" {
                continue;
            }
            let id = row["npc_id"].as_u64().unwrap();
            let owner = format!("{} ({id})", path.display());
            drawn.push((owner, id, position(&row)));
        }
    }
    for row in placed_rows() {
        let id = row["npc_id"].as_u64().unwrap();
        assert!(
            drawn.iter().any(|(_, drawn_id, _)| *drawn_id == id),
            "{id}: the placement scan must include this brief's row"
        );
    }
    for (index, (left_owner, _, left)) in drawn.iter().enumerate() {
        for (right_owner, _, right) in &drawn[index + 1..] {
            let gap = distance(*left, *right);
            assert!(
                gap > 1.0,
                "{left_owner} and {right_owner} stand {gap:.2} m apart"
            );
        }
    }
}
