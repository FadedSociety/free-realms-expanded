//! T696: the two Sanctuary rows that one reference spawn names, and no drawn row owns yet,
//! now stand at it.
//!
//! `lab/references/quest-lines/wiring/placements/sanctuary.json` matched 51 undrawn
//! regional rows to a single FreeRealms-Legacy/Sanctuary-main spawn each. Forty-two were
//! already drawn by T470 (a `reference_data` row plus its `npc_spawns_extra.json` body at
//! the same position), seven are the other half of a duplicate row or are already drawn at
//! the spawn (Bruce and Jeni Shortfuse by Snowhill rows, T695), so this slice places the
//! remaining two as `legacy_data` spawns.
//!
//! The wire path is unchanged: `NpcVisibility::load` draws a regional `legacy_data` row as
//! its own actor, and `update` sends the 35/2 AddNpc the client reads (reader `0x00921a10`:
//! u64 actor id, i32 name id, i32 model id first, then the transform captured by
//! `startup_world::parse_add_npc`). No new opcode, field or selector.
//!
//! Positions and names are read from the shipped data; the only pinned literals are the
//! placement brief's marker text, the cited spawn call shape and the 1 m duplicate rule.

#[path = "support/json.rs"]
mod json;

use osfr_tools::{startup_world::parse_add_npc, world::NpcVisibility};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

/// The marker the placement brief requires in every position reason it writes.
const MARKER: &str = "matched by the spawn's client name (talk_spawns.py) (R)";
/// `world.rs` loads every regional and extra spawn into the open-world label (R).
const WORLD_ZONE: u32 = 0;
const MIN_SEPARATION_M: f64 = 1.0;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn xyz(value: &Value) -> [f32; 3] {
    serde_json::from_value(value.clone()).expect("a finite xyz position")
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| f64::from(a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// The rows this slice placed: the `sanctuary.json` rows whose position reason carries the
/// brief's marker. Other zones' briefs write the same marker into their own files, so the
/// scan stays in this file. The marker, not a hard-coded id, is the selector; a row that
/// loses its citation drops out, and a row that is not a `legacy_data` spawn fails the
/// placement contract.
fn placed_rows() -> Vec<(String, Value)> {
    let file = "sanctuary.json";
    json::read(&content_dir().join("npcs").join(file))["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["position_reason"]
                .as_str()
                .unwrap_or_default()
                .contains(MARKER)
        })
        .map(|row| (file.to_owned(), row.clone()))
        .collect()
}

/// The spawn the row itself cites: `spawnNpcWithGuid(ref, guid, x, y, z, heading)`.
fn cited_spawn(reason: &str) -> ([f32; 3], f32) {
    let call = reason
        .split("spawnNpcWithGuid(")
        .nth(1)
        .expect("a cited spawnNpcWithGuid call");
    let numbers: Vec<f64> = call
        .split(')')
        .next()
        .unwrap()
        .split(',')
        .map(|value| value.trim().parse().expect("a numeric spawn argument"))
        .collect();
    assert_eq!(
        numbers.len(),
        6,
        "the cited spawn has ref, guid, x, y, z, h"
    );
    (
        [numbers[2] as f32, numbers[3] as f32, numbers[4] as f32],
        numbers[5] as f32,
    )
}

#[test]
fn every_placed_row_is_drawn_at_its_position() {
    let rows = placed_rows();
    assert!(
        !rows.is_empty(),
        "no NPC row carries the talk-spawns placement marker"
    );
    let visibility = NpcVisibility::load(&content_dir()).unwrap();
    let mut ids = BTreeSet::new();
    for (file, row) in &rows {
        let id = row["npc_id"].as_u64().unwrap();
        let owner = format!("{file} NPC {id} ({})", row["name"]);
        assert_eq!(row["placement"], "legacy_data", "{owner}: not a spawn");
        assert_eq!(
            row["placement_source"], "legacy_data_spawn",
            "{owner}: unexpected source"
        );
        // The row's own citation is the independent witness: it must name the transform
        // the row stores, or the row was moved away from the evidence it cites.
        let position = xyz(&row["position"]);
        let heading = row["heading"].as_f64().unwrap() as f32;
        let (cited_position, cited_heading) = cited_spawn(row["position_reason"].as_str().unwrap());
        assert_eq!(
            position, cited_position,
            "{owner}: position differs from its cited spawn"
        );
        assert_eq!(
            heading, cited_heading,
            "{owner}: heading differs from its cited spawn"
        );
        assert!(ids.insert(id), "{owner}: one placed row per actor id");
        // The production loader draws the regional row as its own actor, at its position.
        assert_eq!(
            visibility.actor_position(id),
            Some(position),
            "{owner}: world.rs does not draw the row at its position"
        );
        assert_eq!(
            visibility.content_key(id),
            Some(id),
            "{owner}: the drawn actor is not bound to the row"
        );
        // And the 35/2 body the client receives carries the row's own identity and place.
        let adds: Vec<_> = visibility
            .clone()
            .update(WORLD_ZONE, position)
            .unwrap()
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .filter_map(|packet| parse_add_npc(packet).ok())
            .filter(|npc| npc.actor_id == id)
            .collect();
        assert_eq!(adds.len(), 1, "{owner}: exactly one AddNpc");
        assert_eq!(
            adds[0].model_id,
            row["model_id"].as_i64().unwrap() as i32,
            "{owner}: AddNpc model"
        );
        assert_eq!(
            adds[0].name_string_id,
            row["name_string_id"].as_i64().unwrap() as i32,
            "{owner}: AddNpc name id"
        );
        assert_eq!(
            [
                adds[0].position[0],
                adds[0].position[1],
                adds[0].position[2]
            ],
            position,
            "{owner}: AddNpc position"
        );
        assert_eq!(adds[0].position[3], 1.0, "{owner}: AddNpc W");
    }
    println!("T696 placed {} rows: {ids:?}", rows.len());
}

/// The brief's duplicate rule, over every drawn NPC row the loader binds to content: a
/// regional `legacy_data` row or an `npc_spawns_extra.json` body with a `content_key`.
/// Unnamed scene props are not NPC rows and are excluded, as is anything the loader drops.
#[test]
fn no_two_drawn_npc_rows_share_a_position() {
    let visibility = NpcVisibility::load(&content_dir()).unwrap();
    let placed: BTreeSet<u64> = placed_rows()
        .iter()
        .map(|(_, row)| row["npc_id"].as_u64().unwrap())
        .collect();
    let mut drawn: Vec<(u64, [f32; 3])> = visibility
        .teleport_targets(WORLD_ZONE)
        .into_iter()
        .filter(|target| visibility.content_key(target.guid).is_some())
        .map(|target| (target.guid, target.position))
        .collect();
    drawn.sort_by_key(|(id, _)| *id);
    for id in &placed {
        assert!(
            drawn.iter().any(|(actor, _)| actor == id),
            "placed NPC {id} is not in the drawn NPC set"
        );
    }
    for (index, (id, position)) in drawn.iter().enumerate() {
        for (other, other_position) in &drawn[index + 1..] {
            let apart = distance(*position, *other_position);
            assert!(
                apart > MIN_SEPARATION_M,
                "drawn NPC rows {id} and {other} stand {apart:.3} m apart"
            );
        }
    }
    println!("T696: {} content-keyed drawn NPC rows checked", drawn.len());
}
