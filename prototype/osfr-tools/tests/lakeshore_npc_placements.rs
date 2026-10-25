//! T697: Lakeshore NPC placement stays drawable and does not stack.
//!
//! `world.rs::NpcVisibility::load` draws a regional row only when its placement is
//! `legacy_data` (the client actor is the row's `npc_id`) or when a renderable
//! `npc_spawns_extra.json` row carries the NPC's `content_key` (the existing
//! content-keyed bodies, including T470's and T372's). Client evidence: 35/2
//! AddNpc reader `0x00921a10` reads the actor u64, the name i32, the model i32,
//! the chat-bubble colours and size, the scale f32 (this+84), a position vec4 and a
//! rotation vec4, so a drawn NPC stands where its row says it does.

#[path = "support/json.rs"]
mod json;

use osfr_tools::{startup_world::parse_add_npc, world::NpcVisibility};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn lakeshore_rows() -> Vec<Value> {
    json::read(&content_dir().join("npcs/lakeshore.json"))["records"]
        .as_array()
        .unwrap()
        .clone()
}

/// Every row that world.rs draws from the regional file itself must reach a player
/// standing at it, in the 35/2 AddNpc the gateway sends, exactly once and at the row's
/// own position.
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
        let drawn: Vec<_> = visibility
            .clone()
            .update(0, position)
            .unwrap()
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .map(|packet| parse_add_npc(packet).unwrap())
            .filter(|npc| npc.actor_id == npc_id)
            .collect();
        assert_eq!(
            drawn.len(),
            1,
            "NPC {npc_id} ({}) must have exactly one drawn body",
            row["name"]
        );
        assert_eq!(
            drawn[0].position[..3],
            position,
            "NPC {npc_id} ({}) must be drawn at its declared position",
            row["name"]
        );
    }
    assert!(legacy > 0, "lakeshore.json has no legacy_data row to check");
}

/// Every Lakeshore NPC body the loader draws (a legacy_data row or an extras row
/// keyed to a Lakeshore row) appears once, and no two bodies stand within 1 m.
#[test]
fn drawn_lakeshore_bodies_are_one_per_npc_and_a_metre_apart() {
    let content = content_dir();
    let rows = lakeshore_rows();
    let ids: BTreeSet<u64> = rows
        .iter()
        .map(|row| row["npc_id"].as_u64().unwrap())
        .collect();
    let mut expected: BTreeSet<u64> = rows
        .iter()
        .filter(|row| row["placement"] == "legacy_data")
        .map(|row| row["npc_id"].as_u64().unwrap())
        .collect();
    for extra in json::read(&content.join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
    {
        // The loader skips capture-matched rows, the model-less knocker boss and key 0
        // (`world.rs` NpcVisibility::load); the expectation must skip the same rows.
        let unrenderable = !extra["capture_match"].is_null()
            || extra["model_file_name"]
                .as_str()
                .is_some_and(|name| name.eq_ignore_ascii_case("knocker_m_boss.adr"));
        if !unrenderable
            && extra["content_key"]
                .as_u64()
                .is_some_and(|key| key != 0 && ids.contains(&key))
        {
            expected.insert(extra["spawn_guid"].as_u64().unwrap());
        }
    }

    let drawn: Vec<_> = NpcVisibility::load(&content)
        .unwrap()
        .teleport_targets(0)
        .into_iter()
        .filter(|target| expected.contains(&target.guid))
        .collect();
    assert_eq!(
        drawn.len(),
        expected.len(),
        "every Lakeshore NPC must have exactly one drawn body"
    );
    let mut guids = BTreeSet::new();
    for target in &drawn {
        assert!(
            guids.insert(target.guid),
            "NPC {} has more than one drawn body",
            target.guid
        );
    }
    for (index, target) in drawn.iter().enumerate() {
        for other in &drawn[index + 1..] {
            let distance = target
                .position
                .iter()
                .zip(other.position)
                .map(|(a, b)| f64::from(a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(
                distance > 1.0,
                "NPCs {} and {} stand {distance:.3} m apart",
                target.guid,
                other.guid
            );
        }
    }
}
