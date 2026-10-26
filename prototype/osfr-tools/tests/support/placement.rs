//! Position helpers shared by the NPC placement suites (T696 and later).
//!
//! Test-only; production reads positions through `world.rs`.
#![allow(dead_code)]

use osfr_tools::world::NpcVisibility;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
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

/// The CartoonMMO root, the parent of the repository checkout and `lab/`.
pub fn project_parent() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Every regional NPC row of `data/content/npcs/*.json`, in file order.
pub fn regional_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for entry in
        fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/npcs")).unwrap()
    {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            rows.extend(read_json(&path)["records"].as_array().unwrap().clone());
        }
    }
    rows
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
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
    let rows = read_json(&project_parent().join(path));
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
