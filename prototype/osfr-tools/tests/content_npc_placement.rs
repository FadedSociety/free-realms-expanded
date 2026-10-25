#![allow(dead_code)]

use osfr_tools::content;
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn number(value: &Value, owner: &str) -> f64 {
    let number = value
        .as_f64()
        .unwrap_or_else(|| panic!("{owner}: expected a numeric transform component"));
    assert!(
        number.is_finite(),
        "{owner}: transform component is not finite"
    );
    number
}

#[test]
fn regional_npcs_have_unique_sourced_placements_and_captures_match_exactly() {
    assert_eq!(content::SCHEMA_VERSION, 1);
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let data_root = crate_root.join("data/content/npcs");
    let project_root = crate_root.parent().unwrap().parent().unwrap();
    let lab_root = project_root.parent().unwrap().join("lab");
    let capture = read_json(&lab_root.join("content/startup-snapshot/startup.json"));
    let captures = capture["world"]["npc_spawns"].as_array().unwrap();

    let mut files: Vec<PathBuf> = fs::read_dir(&data_root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    files.sort();

    let mut positions = HashSet::new();
    let mut counts = BTreeMap::<String, usize>::new();
    let mut npc_count = 0usize;
    for file in files {
        let document = read_json(&file);
        for npc in document["records"].as_array().unwrap() {
            npc_count += 1;
            let name = npc["name"].as_str().unwrap_or("unnamed");
            let owner = format!("{} ({name})", npc["npc_id"]);
            let placement = npc["placement"]
                .as_str()
                .unwrap_or_else(|| panic!("{owner}: missing placement classification"));
            let source = npc["placement_source"]
                .as_str()
                .unwrap_or_else(|| panic!("{owner}: missing placement source"));
            assert!(!source.trim().is_empty(), "{owner}: empty placement source");
            assert!(
                npc["placement_confidence"]
                    .as_str()
                    .is_some_and(|confidence| !confidence.trim().is_empty()),
                "{owner}: missing placement confidence"
            );
            assert!(
                matches!(
                    placement,
                    "captured" | "legacy_data" | "poi_anchor" | "unanchored"
                ),
                "{owner}: unknown placement class {placement:?}"
            );
            if placement == "unanchored" {
                assert!(
                    matches!(
                        source,
                        "reconstructed_zone_center" | "reconstructed_unanchored_default"
                    ),
                    "{owner}: unanchored placement has unexpected source {source:?}"
                );
            }
            let confidence = npc["placement_confidence"].as_str().unwrap();
            let candidates = npc["placement_candidates"]
                .as_array()
                .unwrap_or_else(|| panic!("{owner}: missing placement candidates"));
            if placement == "poi_anchor" {
                assert!(
                    !candidates.is_empty(),
                    "{owner}: POI anchor has no candidates"
                );
                if candidates.len() > 1 {
                    assert_eq!(
                        confidence, "low",
                        "{owner}: ambiguous POI match must be low confidence"
                    );
                }
                assert_eq!(source, "reconstructed_poi_name_match");
                for pair in candidates.windows(2) {
                    let left_score = pair[0]["match_score"].as_u64().unwrap();
                    let right_score = pair[1]["match_score"].as_u64().unwrap();
                    let left_id = pair[0]["poi_id"].as_u64().unwrap();
                    let right_id = pair[1]["poi_id"].as_u64().unwrap();
                    assert!(
                        left_score > right_score
                            || (left_score == right_score && left_id < right_id),
                        "{owner}: POI candidates are not sorted by score then id"
                    );
                }
            }
            if placement == "legacy_data" {
                assert_eq!(
                    source, "legacy_data_spawn",
                    "{owner}: unexpected Legacy source"
                );
                assert!(
                    candidates.is_empty(),
                    "{owner}: Legacy placement has POI candidates"
                );
                assert!(
                    npc["model_id"].as_u64().is_some(),
                    "{owner}: missing Legacy model id"
                );
                assert!(
                    npc["heading"].as_f64().is_some(),
                    "{owner}: missing Legacy heading"
                );
            }

            let position = npc["position"]
                .as_array()
                .unwrap_or_else(|| panic!("{owner}: missing xyz position"));
            assert_eq!(position.len(), 3, "{owner}: expected xyz position");
            let xyz = [
                number(&position[0], &owner),
                number(&position[1], &owner),
                number(&position[2], &owner),
            ];
            if source.starts_with("reconstructed_") {
                let key = xyz.map(|value| if value == 0.0 { 0 } else { value.to_bits() });
                assert!(
                    positions.insert(key),
                    "{owner}: reconstructed position duplicates another NPC"
                );
            }
            *counts.entry(source.to_owned()).or_default() += 1;

            if let Some(capture_match) = npc["capture_match"].as_object() {
                let actor_id = capture_match["actor_id"].as_u64().unwrap();
                let captured = captures
                    .iter()
                    .find(|captured| captured["actor_id"].as_u64() == Some(actor_id))
                    .unwrap_or_else(|| panic!("{owner}: capture actor {actor_id} is absent"));
                assert_eq!(placement, "captured", "{owner}: capture not marked exact");
                let captured_position = captured["position"].as_array().unwrap();
                for axis in 0..3 {
                    assert_eq!(
                        number(&position[axis], &owner),
                        number(&captured_position[axis], &owner),
                        "{owner}: captured position axis {axis} differs"
                    );
                }
                assert_eq!(
                    npc["position_w"].as_f64(),
                    captured_position[3].as_f64(),
                    "{owner}: captured position W differs"
                );
                let rotation = npc["rotation"]
                    .as_array()
                    .unwrap_or_else(|| panic!("{owner}: captured rotation missing"));
                assert_eq!(
                    rotation.len(),
                    4,
                    "{owner}: expected captured float4 rotation"
                );
                for index in 0..4 {
                    assert_eq!(
                        number(&rotation[index], &owner),
                        number(&captured["rotation"][index], &owner),
                        "{owner}: captured rotation component {index} differs"
                    );
                }
                assert_eq!(
                    npc["capture_match"]["actor_id"].as_u64(),
                    captured["actor_id"].as_u64()
                );
            } else {
                assert_ne!(placement, "captured", "{owner}: no matching client capture");
                assert!(
                    npc["capture_match"].is_null(),
                    "{owner}: unmatched NPC has a capture match"
                );
                let rotation = npc["rotation"]
                    .as_array()
                    .unwrap_or_else(|| panic!("{owner}: reconstructed rotation missing"));
                assert_eq!(
                    rotation.len(),
                    4,
                    "{owner}: expected reconstructed quaternion"
                );
                let norm = rotation
                    .iter()
                    .map(|component| number(component, &owner).powi(2))
                    .sum::<f64>();
                assert!(
                    (norm - 1.0).abs() < 0.00001,
                    "{owner}: reconstructed rotation is not normalized"
                );
            }
        }
    }

    // T379 appended two unanchored repair-guy binding rows.
    // T498 appended the Burglz row (1,202); T500 appended Hasti (1,203).
    assert_eq!(npc_count, 1_203);
    println!("NPC placement sources: {counts:?}");
}
