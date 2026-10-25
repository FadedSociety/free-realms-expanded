//! T695: Snowhill's remaining undrawn NPC rows stand at the one reference spawn that
//! names them.
//!
//! `lab/fleet/quests/talk_spawns.py` joins a regional NPC row to a FreeRealms-Legacy /
//! Sanctuary-main spawn by the client name the spawn's `Npcs.json` NameId resolves. The
//! rows this brief places (`PLACED`, by id) are `legacy_data` / `legacy_data_spawn`, the
//! only placement `world.rs` draws from `npcs/*.json`. They are sent as the ordinary
//! 35/2 AddNpc body: u64 actor, i32 name id, i32 model id, then the transform (client
//! reader `0x00921a10`). No new wire shape.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{content_dir, read_json, seen_from};
use osfr_tools::{startup_world::NpcSpawn, world::NpcVisibility};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

/// The six rows this brief placed, by npc_id (R: the data decides the spawn; the ids are
/// the brief's list, so a copy edit can never silently drop one from the check).
const PLACED: [u64; 6] = [
    9_000_000_020,
    9_500_000_729,
    9_000_000_069,
    9_000_000_115,
    9_500_000_766,
    9_000_000_201,
];

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

/// The brief's rows: all six exist, are `legacy_data` and come from the Legacy spawn table.
fn placed_rows() -> Vec<Value> {
    let all = rows(&content_dir().join("npcs/snowhill.json"));
    PLACED
        .iter()
        .map(|id| {
            let row = all
                .iter()
                .find(|row| row["npc_id"].as_u64() == Some(*id))
                .unwrap_or_else(|| panic!("{id}: no Snowhill row"));
            assert_eq!(row["placement"], "legacy_data", "{id}: placement");
            assert_eq!(row["placement_source"], "legacy_data_spawn", "{id}: source");
            row.clone()
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

#[test]
fn every_row_this_brief_placed_is_drawn_at_its_spawn_position() {
    let placed = placed_rows();
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

/// Every NPC the client sees: `legacy_data` regional rows (actor = npc_id) and the extras
/// bodies that carry a regional row's `content_key` (actor = spawn_guid). Scenery extras
/// without a regional row legitimately overlap, so they stay out of the scan.
#[test]
fn drawn_npc_rows_do_not_share_a_position() {
    let mut drawn: Vec<(String, u64, [f32; 3])> = Vec::new();
    let mut regional_ids = BTreeSet::new();
    for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        for row in rows(&path) {
            let id = row["npc_id"].as_u64().unwrap();
            regional_ids.insert(id);
            if row["placement"] == "legacy_data" {
                drawn.push((format!("{} ({id})", path.display()), id, position(&row)));
            }
        }
    }
    for body in rows(&content_dir().join("npc_spawns_extra.json")) {
        let drawable = body["capture_match"].is_null() && body["content_key"].as_u64() != Some(0);
        if drawable
            && body["content_key"]
                .as_u64()
                .is_some_and(|key| regional_ids.contains(&key))
        {
            let guid = body["spawn_guid"].as_u64().unwrap();
            drawn.push((format!("extras body {guid}"), guid, position(&body)));
        }
    }
    for id in PLACED {
        assert!(
            drawn.iter().any(|(_, drawn_id, _)| *drawn_id == id),
            "{id}: the placement scan must include this brief's row"
        );
    }
    let actors: BTreeSet<_> = drawn.iter().map(|(_, id, _)| *id).collect();
    assert_eq!(actors.len(), drawn.len(), "an actor id is drawn twice");
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
