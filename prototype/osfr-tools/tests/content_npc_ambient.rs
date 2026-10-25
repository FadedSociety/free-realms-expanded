//! T507: the ambient NPC table is bounded and every client reference exists.
//!
//! The row data is Sanctuary's `Resources/NpcAmbient.json`
//! (`Sanctuary-main-20261019@10f129e`, values only, no code). The ids must be
//! client `AnimationSlot` ids and the model ids must be the client `Models.txt`
//! ids that draw the row's file, so the runtime key is the AddNpc model id.

#[path = "support/models.rs"]
mod models;

use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn client_resources() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("lab/working/OSFR.Client/OSFR Client/Resources")
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

/// One client XML element table, e.g. every `AnimationSlot` id.
fn xml_ids(path: &Path, element: &str) -> HashSet<i32> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut ids = HashSet::new();
    let mut rest = text.as_str();
    while let Some(open) = rest.find(&format!("<{element}")) {
        let tail = &rest[open..];
        let Some(end) = tail.find('>') else {
            break;
        };
        if let Some(id) = attribute(&tail[..end], "id").and_then(|id| id.parse().ok()) {
            ids.insert(id);
        }
        rest = &tail[end..];
    }
    ids
}

#[test]
fn ambient_rows_name_client_slots_and_models() {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let document = read_json(&content.join("npc_ambient.json"));
    let slots = xml_ids(
        &client_resources().join("AnimationTypes.xml"),
        "AnimationSlot",
    );
    assert!(
        slots.contains(&3302) && slots.contains(&3333) && slots.contains(&202),
        "the parser reads the client slot table"
    );
    let models = models::read_models();
    let mut model_ids: HashMap<i64, String> = HashMap::new();

    assert_eq!(document["schema_version"], 1);
    assert!(
        document["source"]
            .as_str()
            .is_some_and(|source| source.contains("Sanctuary-main-20261019@10f129e")),
        "the table names its Sanctuary provenance"
    );
    let cap = document["max_packets_per_viewer_per_tick"]
        .as_u64()
        .expect("the per-viewer cap is a number");
    assert!((1..=16).contains(&cap), "the per-viewer cap is 1..=16");
    let rows = document["rows"].as_array().expect("rows is an array");
    assert_eq!(rows.len(), 50, "every Sanctuary row with AnimationIds");

    for (index, row) in rows.iter().enumerate() {
        let owner = format!("row {index} ({})", row["model_file_name"]);
        let name = row["model_file_name"]
            .as_str()
            .unwrap_or_else(|| panic!("{owner}: no model file name"));
        let file = name.trim().to_ascii_lowercase();
        let announced = row["source"]
            .as_str()
            .unwrap_or_else(|| panic!("{owner}: no source"));
        assert!(
            announced.contains("Sanctuary-main-20261019@10f129e")
                && announced.contains(&format!("ModelFileName={name} (R)")),
            "{owner}: source must name the Sanctuary row"
        );
        let ids = row["animation_ids"]
            .as_array()
            .unwrap_or_else(|| panic!("{owner}: no animation ids"));
        assert!(
            (1..=16).contains(&ids.len()),
            "{owner}: 1..=16 animation ids"
        );
        let mut seen = HashSet::new();
        for id in ids {
            let id = id
                .as_i64()
                .unwrap_or_else(|| panic!("{owner}: non-integer id"));
            assert!(
                slots.contains(&i32::try_from(id).unwrap()),
                "{owner}: slot {id}"
            );
            assert!(seen.insert(id), "{owner}: repeated slot {id}");
        }
        let min = row["min_interval_ms"]
            .as_u64()
            .unwrap_or_else(|| panic!("{owner}: no minimum interval"));
        let max = row["max_interval_ms"]
            .as_u64()
            .unwrap_or_else(|| panic!("{owner}: no maximum interval"));
        assert!(500 <= min && min <= max && max <= 600_000, "{owner}");

        // Every client id bound to this file draws this row, and no other id does.
        let bound: HashSet<i64> = models
            .iter()
            .filter(|(_, names)| names.contains(&file))
            .map(|(id, _)| *id)
            .collect();
        let listed: HashSet<i64> = row["model_ids"]
            .as_array()
            .unwrap_or_else(|| panic!("{owner}: no model ids"))
            .iter()
            .map(|id| {
                id.as_i64()
                    .unwrap_or_else(|| panic!("{owner}: non-integer model"))
            })
            .collect();
        assert_eq!(listed, bound, "{owner}: model ids must be Models.txt's");
        for id in &listed {
            assert!(
                models.get(id).is_some_and(|names| names.contains(&file)),
                "{owner}: Models.txt {id} is not {name}"
            );
            assert_eq!(
                model_ids.insert(*id, name.to_owned()),
                None,
                "{owner}: model {id} is bound twice"
            );
        }
    }

    // The handoff case: the client maps chugawug_f_01.adr to model 13, whose
    // applaud/cheer row is the one the gateway test drives.
    let chugawug = rows
        .iter()
        .find(|row| row["model_file_name"] == "chugawug_f_01.adr")
        .expect("the chugawug row is imported");
    assert_eq!(chugawug["model_ids"], serde_json::json!([13]));
    assert_eq!(chugawug["animation_ids"], serde_json::json!([3302, 3306]));
    assert_eq!(chugawug["min_interval_ms"], 3796);
    assert_eq!(chugawug["max_interval_ms"], 5563);
}
