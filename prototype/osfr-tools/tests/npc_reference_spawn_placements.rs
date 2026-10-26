//! T698: the regional NPC rows placed from a name-matched reference spawn
//! (`lab/fleet/quests/talk_spawns.py`) stand where their cited spawn says and are
//! drawn by the production NPC visibility loader.
//!
//! The placement marker in `position_reason` is the brief's own wording; every
//! value the test compares is read from the content, the cited `FabledRealms.lua`
//! line and that source's `Resources/Npcs.json`, never from a pinned list.
//! Client wire: 35/2 AddNpc reader `0x00921a10` (u64 actor, i32 name id, i32 model
//! id) via the shared `seen_from` (`parse_add_npc`); no wire shape changed.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{content_dir, read_json, seen_from};
use osfr_tools::world::NpcVisibility;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The provenance phrase T698 requires on every row it places.
const SPAWN_MARKER: &str = "matched by the spawn's client name (talk_spawns.py) (R)";
/// Two NPCs closer than this are the same standing spot (the brief's 1 m rule).
const SAME_SPOT_METRES: f64 = 1.0;

fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The CartoonMMO root, the parent of the repository checkout and `lab/`.
fn project_parent() -> PathBuf {
    crate_root()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// The repository's `lab/` sibling, which holds the cited source and locale data.
fn lab_dir() -> PathBuf {
    project_parent().join("lab")
}

fn regional_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            rows.extend(read_json(&path)["records"].as_array().unwrap().clone());
        }
    }
    rows
}

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

fn xyz(row: &Value) -> [f32; 3] {
    let position = row["position"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: position array", row["npc_id"]));
    assert_eq!(position.len(), 3, "{}: xyz position", row["npc_id"]);
    [
        position[0].as_f64().expect("x") as f32,
        position[1].as_f64().expect("y") as f32,
        position[2].as_f64().expect("z") as f32,
    ]
}

/// The authored f64 position, for comparing the row with its cited source line
/// before the loader narrows it to f32.
fn xyz_f64(row: &Value) -> [f64; 3] {
    let position = row["position"].as_array().unwrap();
    assert_eq!(position.len(), 3, "{}: xyz position", row["npc_id"]);
    [
        position[0].as_f64().expect("x"),
        position[1].as_f64().expect("y"),
        position[2].as_f64().expect("z"),
    ]
}

fn has_kind(value: &Value, kind: &str) -> bool {
    value
        .as_array()
        .is_some_and(|tags| tags.iter().any(|tag| tag["kind"] == kind))
}

/// The source line and spawn values cited by a `position_reason`.
struct CitedSpawn {
    source: String,
    line: usize,
    ref_id: u64,
    position: [f64; 3],
    heading: f64,
}

fn cited_spawn(reason: &str) -> CitedSpawn {
    let start = reason
        .find("copied from ")
        .unwrap_or_else(|| panic!("position reason does not cite a source: {reason:?}"))
        + "copied from ".len();
    let citation = reason[start..].split(';').next().unwrap().trim().to_owned();
    let (source, line) = citation
        .rsplit_once(':')
        .unwrap_or_else(|| panic!("citation has no line: {citation:?}"));
    let line = line.parse::<usize>().unwrap();
    let text = fs::read_to_string(project_parent().join(source))
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    let row = text
        .lines()
        .nth(line - 1)
        .unwrap_or_else(|| panic!("{source}:{line} is past the end"));
    let call = row
        .find("spawnNpc")
        .unwrap_or_else(|| panic!("{source}:{line}: no spawnNpc call"));
    let open = call
        + row[call..]
            .find('(')
            .unwrap_or_else(|| panic!("{source}:{line}: no argument list"));
    let close = open
        + row[open..]
            .find(')')
            .unwrap_or_else(|| panic!("{source}:{line}: unterminated argument list"));
    let args: Vec<&str> = row[open + 1..close].split(',').map(str::trim).collect();
    assert!(args.len() >= 5, "{source}:{line}: unexpected spawn arity");
    let number = |index: usize| -> f64 {
        args[index]
            .parse()
            .unwrap_or_else(|error| panic!("{source}:{line}: {error}"))
    };
    let last = args.len();
    CitedSpawn {
        source: source.to_owned(),
        line,
        ref_id: args[0]
            .parse()
            .unwrap_or_else(|error| panic!("{source}:{line}: npc id: {error}")),
        position: [number(last - 4), number(last - 3), number(last - 2)],
        heading: number(last - 1),
    }
}

fn npcs_row(source: &str, ref_id: u64) -> Value {
    let path = source.replace("/Scripts/Zone/FabledRealms.lua", "/Resources/Npcs.json");
    assert_ne!(path, source, "source path does not name the zone script");
    let rows = read_json(&project_parent().join(path));
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| row["Id"].as_u64() == Some(ref_id))
        .unwrap_or_else(|| panic!("{source}: Npcs.json Id {ref_id}"))
        .clone()
}

fn locale_text(name_id: u64) -> String {
    let rows = read_json(&lab_dir().join("reports/locale-id-to-text.json"));
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(name_id))
        .and_then(|row| row["text"].as_str())
        .unwrap_or_else(|| panic!("NameId {name_id} does not resolve in the report"))
        .to_owned()
}

fn normalize(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

#[test]
fn placed_rows_match_the_cited_spawn_and_the_loader_draws_them() {
    let rows = placed_by_this_brief();
    assert!(
        !rows.is_empty(),
        "no row carries the T698 reference-spawn marker"
    );
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
        let authored = xyz_f64(&row);
        for axis in 0..3 {
            assert!(
                (authored[axis] - spawn.position[axis]).abs() <= 1e-9,
                "{owner}: position axis {axis} differs from {}:{}",
                spawn.source,
                spawn.line
            );
        }
        let position = xyz(&row);
        let heading = row["heading"].as_f64().unwrap();
        assert!(
            (heading - spawn.heading).abs() <= 1e-9,
            "{owner}: heading differs from {}:{}",
            spawn.source,
            spawn.line
        );
        let reference = npcs_row(&spawn.source, spawn.ref_id);
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
        assert_eq!(
            normalize(row["name"].as_str().unwrap()),
            normalize(&locale_text(name_id)),
            "{owner}: spawn NameId {name_id} does not name this row"
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
            has_kind(&row["field_sources"]["position"], "legacy-data"),
            "{owner}: position lacks Legacy provenance"
        );
        assert!(
            has_kind(&row["source"], "legacy-data"),
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
            let position = xyz(row);
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
            let distance = left
                .iter()
                .zip(right)
                .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(
                distance > SAME_SPOT_METRES,
                "{left_id} ({left_name}) and {right_id} ({right_name}) share a spot ({distance:.2} m)"
            );
        }
    }

    // The real loader also draws extras and placements: no actor but the row
    // itself may stand on a T698 row's spot.
    let base = NpcVisibility::load(&content_dir()).unwrap();
    for row in placed_by_this_brief() {
        let id = row["npc_id"].as_u64().unwrap();
        let position = xyz(&row);
        for other in seen_from(&base, position) {
            if other.actor_id == id {
                continue;
            }
            let distance = position
                .iter()
                .zip(&other.position)
                .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(
                distance > SAME_SPOT_METRES,
                "NPC {id} shares its spot with actor {} ({distance:.2} m)",
                other.actor_id
            );
        }
    }
}
