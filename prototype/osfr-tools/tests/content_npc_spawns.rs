#[path = "support/models.rs"]
mod models;

use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn finite_xyz(row: &Value, owner: &str) {
    let position = row["position"]
        .as_array()
        .unwrap_or_else(|| panic!("{owner}: position is missing"));
    assert_eq!(position.len(), 3, "{owner}: expected XYZ position");
    for component in position {
        assert!(
            component.as_f64().is_some_and(f64::is_finite),
            "{owner}: position contains a non-finite component"
        );
    }
}

fn has_source(row: &Value, kind: &str) -> bool {
    row["source"].as_array().is_some_and(|sources| {
        sources
            .iter()
            .any(|source| source["kind"].as_str() == Some(kind))
    })
}

#[test]
fn npc_spawn_models_names_positions_and_capture_matches_are_valid() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root.parent().unwrap().parent().unwrap();
    let lab_root = project_root.parent().unwrap().join("lab");
    let content_root = crate_root.join("data/content");

    let locale: Value = read_json(&lab_root.join("reports/locale-id-to-text.json"));
    let locale_ids: HashSet<u64> = locale
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap())
        .collect();
    let catalog = fs::read_to_string(lab_root.join("extracted/client-assets/catalog.tsv")).unwrap();
    let catalog_names: HashSet<String> = catalog
        .lines()
        .skip(1)
        .filter_map(|line| line.split('\t').nth(2))
        .map(str::to_ascii_lowercase)
        .collect();
    // The extracted pack catalogue is not complete for every model named by the
    // client itself: 145 Models.txt `.adr` names are absent. The client's own
    // Assets_manifest.txt is the shipped asset list (`.z` entries), so a model
    // name is accepted when either source lists it. `catalog_names` stays the
    // catalog-only set for the knocker `.dme` assertion below.
    let manifest =
        fs::read_to_string(lab_root.join("working/OSFR.Client/OSFR Client/Assets_manifest.txt"))
            .unwrap();
    let shipped_names: HashSet<String> = catalog_names
        .iter()
        .cloned()
        .chain(manifest.lines().filter_map(|line| {
            line.split(',')
                .next()
                .map(|name| name.trim_end_matches(".z").to_ascii_lowercase())
        }))
        .collect();
    // The client's own model table is the authority on which `.adr` a model id draws:
    // a row whose file name is shipped but whose model_id points at another body must
    // fail here, not in the client.
    let models = models::read_models();

    let capture_doc = read_json(&lab_root.join("content/startup-snapshot/startup.json"));
    let captures: HashMap<u64, &Value> = capture_doc["world"]["npc_spawns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["actor_id"].as_u64().unwrap(), row))
        .collect();

    let mut regional_legacy = 0;
    let mut regional_reconstructed = 0;
    for entry in fs::read_dir(content_root.join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let document = read_json(&path);
        for row in document["records"].as_array().unwrap() {
            let owner = format!("{} ({})", row["npc_id"], row["name"]);
            let source = row["placement_source"].as_str().unwrap();
            if source == "legacy_data_spawn" {
                regional_legacy += 1;
                assert_eq!(row["placement"], "legacy_data", "{owner}");
                assert!(
                    has_source(row, "legacy-data"),
                    "{owner}: missing Legacy tag"
                );
                assert!(
                    row["field_sources"]["position"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|tag| tag["kind"] == "legacy-data"),
                    "{owner}: position lacks Legacy provenance"
                );
                let model_file = row["model_file_name"].as_str().unwrap();
                assert!(
                    catalog_names.contains(&model_file.to_ascii_lowercase()),
                    "{owner}: model file {model_file:?} is absent from catalog.tsv"
                );
                let name_id = row["name_string_id"].as_u64().unwrap();
                assert!(
                    locale_ids.contains(&name_id),
                    "{owner}: locale id {name_id} is absent"
                );
                assert!(
                    row["heading"].as_f64().is_some_and(f64::is_finite),
                    "{owner}"
                );
                finite_xyz(row, &owner);
            } else {
                regional_reconstructed += 1;
            }
        }
    }

    let extra = read_json(&content_root.join("npc_spawns_extra.json"));
    assert_eq!(extra["schema_version"], 1);
    let extra_rows = extra["records"].as_array().unwrap();
    let mut captured_actors = HashSet::new();
    for row in extra_rows {
        let owner = format!("Legacy spawn {}", row["spawn_guid"]);
        let model_file = row["model_file_name"].as_str().unwrap();
        assert!(
            shipped_names.contains(&model_file.to_ascii_lowercase()),
            "{owner}: model file {model_file:?} is absent from catalog.tsv and Assets_manifest.txt"
        );
        let model_id = row["model_id"].as_u64().unwrap();
        let model_file_key = model_file.trim().to_ascii_lowercase();
        let model_key = i64::try_from(model_id).expect("model id fits a signed table id");
        assert!(
            models
                .get(&model_key)
                .is_some_and(|names| names.contains(&model_file_key)),
            "{owner}: Models.txt does not map model {model_id} to {model_file:?}"
        );
        if let Some(name_id) = row["name_string_id"].as_u64() {
            assert!(
                locale_ids.contains(&name_id),
                "{owner}: locale id {name_id} is absent"
            );
        }
        assert!(
            has_source(row, "legacy-data"),
            "{owner}: missing Legacy tag"
        );
        finite_xyz(row, &owner);
        assert!(
            row["heading"].as_f64().is_some_and(f64::is_finite),
            "{owner}"
        );
        if let Some(position_w) = row["position_w"].as_f64() {
            assert!(
                position_w.is_finite(),
                "{owner}: non-finite W position lane"
            );
        }

        if let Some(capture_match) = row["capture_match"].as_object() {
            let actor_id = capture_match["actor_id"].as_u64().unwrap();
            assert!(
                captured_actors.insert(actor_id),
                "duplicate capture actor {actor_id}"
            );
            assert!(
                has_source(row, "client"),
                "{owner}: capture lacks client provenance"
            );
            let capture = captures.get(&actor_id).unwrap_or_else(|| {
                panic!("{owner}: actor {actor_id} is absent from startup capture")
            });
            for axis in 0..3 {
                let written = row["position"][axis].as_f64().unwrap();
                let captured = capture["position"][axis].as_f64().unwrap();
                assert!(
                    (written - captured).abs() <= 0.01,
                    "{owner}: capture axis {axis} differs"
                );
            }
            let written_w = row["position_w"].as_f64().unwrap();
            let captured_w = capture["position"][3].as_f64().unwrap();
            assert!(
                (written_w - captured_w).abs() <= 0.01,
                "{owner}: capture W differs"
            );
            for axis in 0..4 {
                let written = row["rotation"][axis].as_f64().unwrap();
                let captured = capture["rotation"][axis].as_f64().unwrap();
                assert!(
                    (written - captured).abs() <= 0.01,
                    "{owner}: rotation axis {axis} differs"
                );
            }
            assert_eq!(
                row["model_id"], capture["model_id"],
                "{owner}: capture model differs"
            );
            assert!(
                capture_match["legacy_distance_m"].as_f64().unwrap() < 0.6,
                "{owner}: Legacy-to-capture match exceeds 0.6 m"
            );
        }
    }

    let broken_model_rows: Vec<_> = extra_rows
        .iter()
        .filter(|row| row["model_file_name"] == "knocker_m_boss.adr")
        .collect();
    assert_eq!(broken_model_rows.len(), 3);
    assert!(!catalog_names.contains("knocker_m_boss.dme"));

    // Placing a guessed NPC with evidence moves it between these two counts (W117 moved 4);
    // the regional total is the invariant, not either count.
    assert_eq!(regional_legacy + regional_reconstructed, 1199);
    assert!(
        regional_legacy >= 578,
        "regional NPCs dropped to {regional_legacy}"
    );
    // T257's appended rows, T279's Geoffrey row inserted after Carlos, T300's
    // appended Therin row, and T331's appended Petra row.
    assert_eq!(extra_rows.len(), 3272);
    assert_eq!(captured_actors.len(), 9);
    println!(
        "NPC placements: {regional_legacy} Legacy, {regional_reconstructed} reconstructed; {} extra spawns, {} capture-confirmed",
        extra_rows.len(),
        captured_actors.len()
    );
}
