//! T698: the regional NPC rows placed from a name-matched reference spawn
//! (`lab/fleet/quests/talk_spawns.py`) stand where their cited spawn says and are
//! drawn by the production NPC visibility loader.
//!
//! The placement marker in `position_reason` is the brief's own wording; every
//! value the test compares is read from the content, the cited `FabledRealms.lua`
//! line and that source's `Resources/Npcs.json`, never from a pinned list.
//! Client wire: 35/2 AddNpc reader `0x00921a10` (u64 actor, i32 name id, i32 model
//! id) via the shared `seen_from` (`parse_add_npc`); no wire shape changed.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/placement.rs"]
mod placement;

use npc_quest_chain::{content_dir, read_json, seen_from};
use osfr_tools::world::NpcVisibility;
use placement::{
    cited_spawn, distance, has_source_kind, normalize_name, reference_npcs_row, regional_rows, xyz,
    xyz_f64,
};
use serde_json::Value;

/// The provenance phrase T698 requires on every row it places.
const SPAWN_MARKER: &str = "matched by the spawn's client name (talk_spawns.py) (R)";
/// Two NPCs closer than this are the same standing spot (the brief's 1 m rule).
const SAME_SPOT_METRES: f64 = 1.0;

/// The zone files this brief's rows live in. The Sanctuary and Snowhill briefs write the
/// same marker into their own files (with their own citation shape), so the scan stays here.
const BRIEF_ZONES: [&str; 13] = [
    "blackspore_swamp",
    "briarwood",
    "cobblestone_village",
    "farnum_farmstead",
    "greenwood_forest",
    "highroad_junction",
    "merry_vale",
    "seaside",
    "shrouded_glade",
    "stillwater_crossing",
    "sunstone_valley",
    "wilds",
    "wugachug",
];

/// The rows this brief placed: in its zone files, `position_reason` carries the marker.
fn placed_by_this_brief() -> Vec<Value> {
    let mut rows = Vec::new();
    for zone in BRIEF_ZONES {
        let path = content_dir().join("npcs").join(format!("{zone}.json"));
        for row in read_json(&path)["records"].as_array().unwrap() {
            if row["position_reason"]
                .as_str()
                .is_some_and(|reason| reason.contains(SPAWN_MARKER))
            {
                rows.push(row.clone());
            }
        }
    }
    rows
}

#[test]
fn placed_rows_match_the_cited_spawn_and_the_loader_draws_them() {
    let rows = placed_by_this_brief();
    assert_eq!(
        rows.len(),
        16,
        "the brief places 16 rows; a row lost its reference-spawn marker"
    );
    let locale = npc_greetings::locale_id_to_text();
    let base = NpcVisibility::load(&content_dir()).unwrap();
    for row in rows {
        let id = row["npc_id"].as_u64().unwrap();
        let owner = format!("{id} ({})", row["name"]);
        assert_eq!(row["placement"], "legacy_data", "{owner}: placement");
        assert_eq!(
            row["placement_source"], "legacy_data_spawn",
            "{owner}: placement source"
        );
        assert_eq!(
            row["placement_confidence"], "medium",
            "{owner}: placement confidence"
        );
        assert!(
            row["placement_candidates"].as_array().unwrap().is_empty(),
            "{owner}: placement candidates"
        );

        let reason = row["position_reason"].as_str().unwrap();
        let spawn = cited_spawn(reason);
        assert!(
            reason.contains(&format!("{}:{}", spawn.source, spawn.line)),
            "{owner}: reason does not cite its line"
        );
        let authored = xyz_f64(&row["position"]);
        for axis in 0..3 {
            assert!(
                (authored[axis] - spawn.position[axis]).abs() <= 1e-9,
                "{owner}: position axis {axis} differs from {}:{}",
                spawn.source,
                spawn.line
            );
        }
        let position = xyz(&row["position"]);
        let heading = row["heading"].as_f64().unwrap();
        assert!(
            (heading - spawn.heading).abs() <= 1e-9,
            "{owner}: heading differs from {}:{}",
            spawn.source,
            spawn.line
        );
        let reference = reference_npcs_row(&spawn.source, spawn.ref_id);
        assert_eq!(
            row["model_id"].as_u64(),
            reference["ModelId"].as_u64(),
            "{owner}: model id"
        );
        assert_eq!(
            row["model_file_name"].as_str(),
            reference["ModelFileName"].as_str(),
            "{owner}: model file"
        );
        let name_id = reference["NameId"].as_u64().unwrap();
        let locale_name = locale
            .get(&name_id)
            .unwrap_or_else(|| panic!("{owner}: spawn NameId {name_id} does not resolve"));
        assert_eq!(
            normalize_name(row["name"].as_str().unwrap()),
            normalize_name(locale_name),
            "{owner}: spawn NameId {name_id} does not name this row"
        );
        let bound = row["name_string_id"].as_u64().unwrap_or_else(|| {
            panic!("{owner}: T698 regressed this row's name id to null; bind the spawn's NameId")
        });
        assert_eq!(
            bound, name_id,
            "{owner}: bound name id is not the spawn's NameId"
        );

        let rotation = row["rotation"].as_array().unwrap();
        let yaw = 2.0
            * rotation[1]
                .as_f64()
                .unwrap()
                .atan2(rotation[3].as_f64().unwrap());
        assert!(
            (yaw - heading).abs() <= 1e-9,
            "{owner}: rotation is not the spawn heading"
        );
        assert!(
            has_source_kind(&row["field_sources"]["position"], "legacy-data"),
            "{owner}: position lacks Legacy provenance"
        );
        assert!(
            has_source_kind(&row["source"], "legacy-data"),
            "{owner}: source lacks a Legacy tag"
        );

        assert_eq!(base.content_key(id), Some(id), "{owner}: content key");
        assert_eq!(
            base.actor_position(id),
            Some(position),
            "{owner}: loader position"
        );
        let seen = seen_from(&base, position);
        let drawn: Vec<_> = seen.iter().filter(|npc| npc.actor_id == id).collect();
        assert_eq!(drawn.len(), 1, "{owner}: exactly one drawn actor");
        assert_eq!(
            drawn[0].model_id,
            row["model_id"].as_u64().unwrap() as i32,
            "{owner}: AddNpc model"
        );
        assert_eq!(
            drawn[0].name_string_id, name_id as i32,
            "{owner}: AddNpc name id"
        );
        for axis in 0..3 {
            assert!(
                (drawn[0].position[axis] - position[axis]).abs() <= 1e-3,
                "{owner}: AddNpc position axis {axis}"
            );
        }
    }
}

#[test]
fn no_two_drawn_regional_npc_rows_share_a_position() {
    let rows = regional_rows();
    let drawn: Vec<(u64, String, [f32; 3])> = rows
        .iter()
        .filter(|row| row["placement"] == "legacy_data")
        .filter_map(|row| {
            let position = xyz(&row["position"]);
            position.iter().all(|value| value.is_finite()).then(|| {
                (
                    row["npc_id"].as_u64().unwrap(),
                    row["name"].as_str().unwrap_or_default().to_owned(),
                    position,
                )
            })
        })
        .collect();
    assert!(drawn.len() > 1, "the content has drawn regional NPCs");
    for (index, (left_id, left_name, left)) in drawn.iter().enumerate() {
        for (right_id, right_name, right) in drawn.iter().skip(index + 1) {
            let apart = distance(*left, *right);
            assert!(
                apart > SAME_SPOT_METRES,
                "{left_id} ({left_name}) and {right_id} ({right_name}) share a spot ({apart:.2} m)"
            );
        }
    }

    // The real loader also draws extras and placements: no actor but the row
    // itself may stand on a T698 row's spot.
    let base = NpcVisibility::load(&content_dir()).unwrap();
    for row in placed_by_this_brief() {
        let id = row["npc_id"].as_u64().unwrap();
        let position = xyz(&row["position"]);
        for other in seen_from(&base, position) {
            if other.actor_id == id {
                continue;
            }
            let apart = distance(
                position,
                [other.position[0], other.position[1], other.position[2]],
            );
            assert!(
                apart > SAME_SPOT_METRES,
                "NPC {id} shares its spot with actor {} ({apart:.2} m)",
                other.actor_id
            );
        }
    }
}
