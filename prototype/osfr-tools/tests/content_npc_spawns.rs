#[path = "support/client_assets.rs"]
mod client_assets;
#[path = "support/models.rs"]
mod models;
#[path = "support/paths.rs"]
mod paths;

use osfr_tools::{startup_world, world};
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

/// The client's own model table maps `model_id` to `model_file`. The shipped file name
/// alone is not enough: a row can name a real body while its id points at another one.
fn models_map(models: &HashMap<i64, HashSet<String>>, model_id: u64, model_file: &str) -> bool {
    let key = i64::try_from(model_id).expect("model id fits a signed table id");
    models
        .get(&key)
        .is_some_and(|names| names.contains(&model_file.trim().to_ascii_lowercase()))
}

#[test]
fn npc_spawn_models_names_positions_and_capture_matches_are_valid() {
    let content_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let lab_root = paths::project_parent().join("lab");

    let locale: Value = read_json(&lab_root.join("reports/locale-id-to-text.json"));
    let locale_ids: HashSet<u64> = locale
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_u64().unwrap())
        .collect();
    let catalog_names: HashSet<String> = client_assets::Assets::open()
        .expect("the private extracted client assets are present")
        .names();
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

    // world.rs spawns legacy_data regional rows as actor `npc_id` and extra rows as actor
    // `spawn_guid`, so each id may name one spawn only (T479).
    let mut actors = HashSet::new();
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
                let actor = row["npc_id"].as_u64().unwrap();
                assert!(actors.insert(actor), "{owner}: actor {actor} spawned twice");
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
                // T698: three reference-spawn models ship only in the client
                // Assets_manifest.txt (Robbie's robgoblin, Rumble King Don and
                // General Wugashima), not the extracted catalog.tsv. Accept
                // either shipped list, as the extras branch below already does.
                assert!(
                    shipped_names.contains(&model_file.to_ascii_lowercase()),
                    "{owner}: model file {model_file:?} is absent from catalog.tsv and Assets_manifest.txt"
                );
                // F733: the shipped file name alone is not enough. The client draws the
                // body its model table maps the id to, so gate the id/file pair here too.
                let model_id = row["model_id"]
                    .as_u64()
                    .unwrap_or_else(|| panic!("{owner}: model id is missing"));
                assert!(
                    models_map(&models, model_id, model_file),
                    "{owner}: Models.txt does not map model {model_id} to {model_file:?}"
                );
                // F733: every reference-spawn row must bind the spawn's client NameId. A
                // missing id would draw the body with name id 0 and no nameplate, so it is
                // a content error, not a silent allowance.
                let name_id = row["name_string_id"]
                    .as_u64()
                    .unwrap_or_else(|| panic!("{owner}: name_string_id is missing"));
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
        let actor = row["spawn_guid"].as_u64().unwrap();
        assert!(actors.insert(actor), "{owner}: actor {actor} spawned twice");
        let model_file = row["model_file_name"].as_str().unwrap();
        assert!(
            shipped_names.contains(&model_file.to_ascii_lowercase()),
            "{owner}: model file {model_file:?} is absent from catalog.tsv and Assets_manifest.txt"
        );
        let model_id = row["model_id"].as_u64().unwrap();
        assert!(
            models_map(&models, model_id, model_file),
            "{owner}: Models.txt does not map model {model_id} to {model_file:?}"
        );
        if let Some(name_id) = row["name_string_id"].as_u64() {
            assert!(
                locale_ids.contains(&name_id),
                "{owner}: locale id {name_id} is absent"
            );
        }
        // T508: a subtitle is a client locale line; an alias is non-empty text of at most 120 chars.
        if let Some(sub_text) = row["sub_text_name_string_id"].as_u64() {
            assert!(
                locale_ids.contains(&sub_text),
                "{owner}: sub text locale id {sub_text} is absent"
            );
        }
        for key in ["texture_alias", "tint_alias"] {
            if let Some(alias) = row.get(key) {
                let alias = alias.as_str().unwrap_or_default();
                assert!(
                    (1..=120).contains(&alias.len()),
                    "{owner}: {key} must be 1..=120 chars"
                );
            }
        }
        // T470: the 90 named Sanctuary NPCs carry a `reference` tag (Sanctuary-main) instead.
        assert!(
            has_source(row, "legacy-data") || has_source(row, "reference"),
            "{owner}: missing Legacy or reference tag"
        );
        for tag in row["source"].as_array().unwrap() {
            if tag["kind"] == "reference" {
                assert!(
                    tag["evidence"]
                        .as_str()
                        .is_some_and(|text| text.contains("Sanctuary-main-20261019@10f129e")),
                    "{owner}: a reference tag must name the Sanctuary-main commit"
                );
            }
        }
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
    // T468 retired 10170 and 17596; 25841 is still in the world.
    assert_eq!(broken_model_rows.len(), 1);
    assert!(!catalog_names.contains("knocker_m_boss.dme"));

    // Placing a guessed NPC with evidence moves it between these two counts (W117 moved 4);
    // the regional total is the invariant, not either count. T379 appended two unanchored
    // repair-guy binding rows, so the total is 1,201; T498 appended the Burglz legacy spawn (1,202);
    // T500 appended the unanchored Hasti row (1,203) and moved Dani and Honey to legacy spawns;
    // T501 appended the unanchored Shannon and Sugaboo rows (1,205); T739 appended four
    // `legacy_data` clickable quest props (1,209, all Legacy spawns).
    assert_eq!(regional_legacy + regional_reconstructed, 1209);
    assert!(
        regional_legacy >= 582,
        "regional NPCs dropped to {regional_legacy}"
    );
    // T257's appended rows, T279's Geoffrey row inserted after Carlos, T300's
    // appended Therin row, T331's appended Petra row, T372's appended 25
    // priced-vendor spawns, T385's appended 4 fishing-guide spawns, T409's
    // appended Manager Landon, T467's two Sanctuary card tables, and T479's 3
    // exploration coins (3,307). T468 moved the 632 Legacy rows Lxzxrus 9c5ef6b
    // pruned as misplaced to npc_spawns_retired.json (2,675). T470 drew 90 named
    // Sanctuary-main NPCs as extras (2,765); T500 already draws Dani and Honey as
    // `legacy_data` regional rows, so they must not gain a second body. T536 drew Sanctuary's two
    // permanent-dance radios (2,767); T546 drew the Sanctuary, Lakeshore and Snowhill warpstone
    // actors Sanctuary-main places at those three hubs (2,770). T544 converted Gerold 9000000098
    // and Tevin 9000000215 from T470's reference row plus extras body to a drawn `legacy_data`
    // regional row, removing those two duplicate bodies (2,768). T549 appended the 141
    // ambient critter and townsfolk rows Sanctuary-main places that nothing else drew (2,909);
    // T550 appended the 194 harvest plants and 115 static props among the same 895 spawns (3,218).
    assert_eq!(extra_rows.len(), 3218);
    assert_eq!(captured_actors.len(), 9);
    println!(
        "NPC placements: {regional_legacy} Legacy, {regional_reconstructed} reconstructed; {} extra spawns, {} capture-confirmed",
        extra_rows.len(),
        captured_actors.len()
    );
}

/// T468: the 632 Legacy rows Lxzxrus 9c5ef6b removed as out-of-bounds or misplaced
/// live in `npc_spawns_retired.json`, which `world.rs` never reads. Its `exempt`
/// forges (crafting_stations.json model 1733) stay in the world.
#[test]
fn retired_legacy_spawns_stay_out_of_the_world() {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let retired = read_json(&content.join("npc_spawns_retired.json"));
    let extra = read_json(&content.join("npc_spawns_extra.json"));
    let live_guids: HashSet<u64> = extra["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["spawn_guid"].as_u64().unwrap())
        .collect();
    let live_legacy_ids: HashSet<u64> = extra["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["legacy_npc_id"].as_u64())
        .collect();

    let rows = retired["records"].as_array().unwrap();
    assert_eq!(rows.len(), 632);
    let mut retired_guids = HashSet::new();
    for row in rows {
        let guid = row["spawn_guid"].as_u64().unwrap();
        let legacy_id = row["legacy_npc_id"].as_u64().unwrap();
        assert_eq!(row["retired"]["kind"], "reference", "retired spawn {guid}");
        assert!(
            row["retired"]["evidence"]
                .as_str()
                .is_some_and(|evidence| evidence.contains("9c5ef6b")),
            "retired spawn {guid}: evidence must name Lxzxrus 9c5ef6b"
        );
        assert!(
            !live_guids.contains(&guid),
            "retired spawn {guid} is back in npc_spawns_extra.json"
        );
        assert!(
            !live_legacy_ids.contains(&legacy_id),
            "retired Legacy id {legacy_id} is back in npc_spawns_extra.json"
        );
        assert!(retired_guids.insert(guid), "retired spawn {guid} repeats");
    }
    let exempt = retired["exempt"]["spawn_guids"].as_array().unwrap();
    assert_eq!(exempt.len(), 3);
    for guid in exempt {
        let guid = guid.as_u64().unwrap();
        assert!(
            live_guids.contains(&guid),
            "exempt forge {guid} left npc_spawns_extra.json"
        );
    }

    // Legacy 1215, inside the west-edge stacks: nothing retired is drawn there.
    let mut visibility = world::NpcVisibility::load(&content).unwrap();
    for packet in visibility
        .update(0, [-2538.731, 0.04500806, -425.8167])
        .unwrap()
    {
        if packet.starts_with(&[35, 0, 2, 0]) {
            let guid = startup_world::parse_add_npc(&packet).unwrap().actor_id;
            assert!(
                !retired_guids.contains(&guid),
                "retired spawn {guid} is drawn"
            );
        }
    }
}
