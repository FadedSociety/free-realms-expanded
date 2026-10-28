//! Position helpers shared by the NPC placement suites (T696 and later).
//!
//! Test-only; production reads positions through `world.rs`. The JSON reader and the content
//! root come from the canonical `support` helpers, so a placement scan reads the same tree as
//! the loaders it checks instead of a second hardcoded path.
#![allow(dead_code)]

#[path = "json.rs"]
mod json;
#[path = "npc_visibility.rs"]
mod npc_visibility;
#[path = "paths.rs"]
mod paths;

pub use paths::project_parent;

use osfr_tools::world::NpcVisibility;
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
};

/// A content row's `position` (or any `[x, y, z]` array) as `f32`s.
pub fn xyz(value: &Value) -> [f32; 3] {
    serde_json::from_value(value.clone()).expect("a finite xyz position")
}

/// The authored `f64` position, for comparing a row with its cited source line
/// before the loader narrows it to `f32`. Panics when the value is not three
/// finite numbers.
pub fn xyz_f64(value: &Value) -> [f64; 3] {
    serde_json::from_value(value.clone()).expect("a finite xyz position")
}

/// One NPC body the loader draws in the open world: its actor id, the regional row it is
/// bound to (`None` for scene props and enemies) and where it stands.
pub struct Drawn {
    pub actor: u64,
    pub content_key: Option<u64>,
    pub position: [f32; 3],
}

/// Every body `NpcVisibility::load` draws, ordered by actor id. `teleport_targets` backs the
/// same rows as the 35/2 AddNpc stream (placed, extras and Studio placements, minus the
/// loader's skips), so a placement scan reads this and never re-filters the content files.
pub fn drawn_npcs(visibility: &NpcVisibility) -> Vec<Drawn> {
    let mut drawn: Vec<Drawn> = visibility
        .teleport_targets(0) // the open-world label `world.rs` files every spawn under (R)
        .into_iter()
        .map(|target| Drawn {
            actor: target.guid,
            content_key: visibility.content_key(target.guid),
            position: target.position,
        })
        .collect();
    drawn.sort_by_key(|npc| npc.actor);
    drawn
}

/// Straight-line distance in metres, computed in `f64` so a 1 m rule has no `f32` slack.
pub fn distance(left: [f32; 3], right: [f32; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Distance in the X/Z plane, the metric `piece_077` uses for warpstone attunement.
pub fn planar_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    (left[0] - right[0]).hypot(left[2] - right[2])
}

/// Every regional NPC row of `data/content/npcs/*.json`, in file order.
pub fn regional_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(npc_visibility::content_dir().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            rows.extend(json::read(&path)["records"].as_array().unwrap().clone());
        }
    }
    rows
}

/// The source line and spawn transform a placement row's `position_reason` cites:
/// `copied from <source>:<line>;` followed by the `spawnNpc` call on that line.
pub struct CitedSpawn {
    pub source: String,
    pub line: usize,
    pub ref_id: u64,
    pub position: [f64; 3],
    pub heading: f64,
}

pub fn cited_spawn(reason: &str) -> CitedSpawn {
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

/// The cited zone script's `Resources/Npcs.json` row for `ref_id`.
pub fn reference_npcs_row(source: &str, ref_id: u64) -> Value {
    let path = source.replace("/Scripts/Zone/FabledRealms.lua", "/Resources/Npcs.json");
    assert_ne!(path, source, "source path does not name the zone script");
    let rows = json::read(&project_parent().join(path));
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| row["Id"].as_u64() == Some(ref_id))
        .unwrap_or_else(|| panic!("{source}: Npcs.json Id {ref_id}"))
        .clone()
}

/// A client name in the form two locale strings compare under.
pub fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}

/// Whether a row's `source`/`field_sources` tags carry `kind`.
pub fn has_source_kind(value: &Value, kind: &str) -> bool {
    value
        .as_array()
        .is_some_and(|tags| tags.iter().any(|tag| tag["kind"] == kind))
}

/// Assert one `legacy_data` row placed from a cited `spawnNpc` line still stands where that
/// line says, names and models the cited reference `Npcs.json` row, and is drawn by the
/// production loader as that actor (`NpcVisibility::load` -> 35/2 AddNpc, reader `0x00921a10`).
///
/// `locale` is `npc_greetings::locale_id_to_text`; `models` is the client `Models.txt` map.
pub fn assert_placed_row_matches_cited_spawn(
    row: &Value,
    locale: &BTreeMap<u64, String>,
    models: &HashMap<i64, HashSet<String>>,
    visibility: &NpcVisibility,
) {
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

    // The row's own citation is the independent witness: it must name the transform and the
    // reference body the row stores, or the row was moved away from the evidence it cites.
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

    // The cited reference row and the client's own model and locale tables.
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
    assert_eq!(
        row["name_string_id"].as_u64(),
        Some(name_id),
        "{owner}: the row binds a name id that is not the spawn's NameId"
    );
    let model_id = row["model_id"].as_u64().unwrap();
    let model_file = row["model_file_name"].as_str().unwrap();
    assert!(
        models
            .get(&i64::try_from(model_id).unwrap())
            .is_some_and(|files| files.contains(&model_file.to_ascii_lowercase())),
        "{owner}: Models.txt does not map model {model_id} to {model_file}"
    );

    // The content rotation is the spawn heading as a Y-axis quaternion.
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

    // The production loader draws the row, and the 35/2 body the client receives carries the
    // row's own identity, model and place.
    assert_eq!(visibility.content_key(id), Some(id), "{owner}: content key");
    assert_eq!(
        visibility.actor_position(id),
        Some(position),
        "{owner}: loader position"
    );
    let seen = npc_visibility::seen_from(visibility, position);
    let drawn: Vec<_> = seen.iter().filter(|npc| npc.actor_id == id).collect();
    assert_eq!(drawn.len(), 1, "{owner}: exactly one drawn actor");
    assert_eq!(
        drawn[0].model_id,
        i32::try_from(model_id).unwrap(),
        "{owner}: AddNpc model"
    );
    assert_eq!(
        drawn[0].name_string_id,
        i32::try_from(name_id).unwrap(),
        "{owner}: AddNpc name id"
    );
    for axis in 0..3 {
        assert!(
            (drawn[0].position[axis] - position[axis]).abs() <= 1e-3,
            "{owner}: AddNpc position axis {axis}"
        );
    }
}
