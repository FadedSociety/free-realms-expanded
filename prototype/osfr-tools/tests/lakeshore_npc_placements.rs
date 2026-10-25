//! T697: Lakeshore NPC placement stays drawable and does not stack.
//!
//! `world.rs::NpcVisibility::load` draws a regional row only when its placement is
//! `legacy_data` (the client actor is the row's `npc_id`) or when a renderable
//! `npc_spawns_extra.json` row carries the NPC's `content_key` (the existing
//! content-keyed bodies, including T470's and T372's). Client evidence: 35/2
//! AddNpc reader `0x00921a10` reads the actor u64, the name i32, the model i32,
//! the chat-bubble colours and size, the scale f32 (this+84), a position vec4 and a
//! rotation vec4, so a drawn NPC stands where its row says it does. The draw is read
//! the way a player gets it: `NpcVisibility::update` (single-session view) and the live
//! fan-out `NpcVisibility::shared_view` (gateways run in shared mode, where `update`
//! sends nothing).

#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;

use npc_visibility::{content_dir, seen_from, shared_seen_from};
use osfr_tools::world::NpcVisibility;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn lakeshore_rows() -> Vec<Value> {
    json::read(&content_dir().join("npcs/lakeshore.json"))["records"]
        .as_array()
        .unwrap()
        .clone()
}

/// Every row that world.rs draws from the regional file itself must reach a player
/// standing at it, in the 35/2 AddNpc the gateway sends (single-session and shared
/// fan-out), exactly once and at the row's own position.
#[test]
fn every_lakeshore_legacy_row_is_drawn_at_its_position() {
    let visibility = NpcVisibility::load(&content_dir()).unwrap();
    let mut legacy = 0;
    for row in lakeshore_rows() {
        if row["placement"] != "legacy_data" {
            continue;
        }
        legacy += 1;
        let npc_id = row["npc_id"].as_u64().unwrap();
        let position: [f32; 3] = serde_json::from_value(row["position"].clone()).unwrap();
        for (path, seen) in [
            ("update", seen_from(&visibility, position)),
            ("shared_view", shared_seen_from(&visibility, position)),
        ] {
            let drawn: Vec<_> = seen.iter().filter(|npc| npc.actor_id == npc_id).collect();
            assert_eq!(
                drawn.len(),
                1,
                "{path}: NPC {npc_id} ({}) must have exactly one drawn body",
                row["name"]
            );
            assert_eq!(
                drawn[0].position[..3],
                position,
                "{path}: NPC {npc_id} ({}) must be drawn at its declared position",
                row["name"]
            );
        }
    }
    assert!(legacy > 0, "lakeshore.json has no legacy_data row to check");
}

/// One drawn actor per Lakeshore NPC (grouped by the loader's `content_key` binding, so an
/// extras row with its own `spawn_guid` still counts against the NPC it is keyed to), and
/// no two bodies stand within 1 m. Bodies are read from the live shared 35/2 view at every
/// Lakeshore row and every keyed extras row.
#[test]
fn drawn_lakeshore_bodies_are_one_per_npc_and_a_metre_apart() {
    let content = content_dir();
    let visibility = NpcVisibility::load(&content).unwrap();
    let rows = lakeshore_rows();
    let ids: BTreeSet<u64> = rows
        .iter()
        .map(|row| row["npc_id"].as_u64().unwrap())
        .collect();
    let mut viewers: Vec<[f32; 3]> = rows
        .iter()
        .filter_map(|row| serde_json::from_value(row["position"].clone()).ok())
        .collect();
    for extra in json::read(&content.join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
    {
        if extra["content_key"]
            .as_u64()
            .is_some_and(|key| ids.contains(&key))
        {
            viewers.push(serde_json::from_value(extra["position"].clone()).unwrap());
        }
    }

    // actor id -> (Lakeshore NPC it stands for, position)
    let mut bodies = BTreeMap::new();
    for viewer in viewers {
        for npc in shared_seen_from(&visibility, viewer) {
            if let Some(key) = visibility
                .content_key(npc.actor_id)
                .filter(|key| ids.contains(key))
            {
                bodies.insert(npc.actor_id, (key, npc.position));
            }
        }
    }
    assert!(!bodies.is_empty(), "no Lakeshore body is drawn at all");
    let mut per_npc = BTreeMap::<u64, Vec<u64>>::new();
    for (actor, (key, _)) in &bodies {
        per_npc.entry(*key).or_default().push(*actor);
    }
    for (key, actors) in &per_npc {
        assert_eq!(
            actors.len(),
            1,
            "Lakeshore NPC {key} is drawn as more than one body: {actors:?}"
        );
    }
    let all: Vec<_> = bodies.iter().collect();
    for (index, (actor, (_, position))) in all.iter().enumerate() {
        for (other, (_, other_position)) in &all[index + 1..] {
            let distance = position
                .iter()
                .zip(other_position)
                .take(3)
                .map(|(a, b)| f64::from(a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(
                distance > 1.0,
                "NPCs {actor} and {other} stand {distance:.3} m apart"
            );
        }
    }
}
