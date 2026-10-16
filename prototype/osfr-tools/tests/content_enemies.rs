use osfr_tools::enemies::EnemyEngine;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn row_names(row: &Value) -> Vec<String> {
    if let Some(names) = row["names"].as_array() {
        return names
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
    }
    row["name"]["value"]
        .as_str()
        .or_else(|| row["name"].as_str())
        .map(|name| vec![name.to_owned()])
        .unwrap_or_default()
}

fn row_key(file: &str, field: &str, index: usize, name: &str) -> (String, String, usize, String) {
    (file.to_owned(), field.to_owned(), index, name.to_owned())
}

#[test]
fn enemy_definitions_load_and_model_assets_match_client_tables() {
    let root = root();
    EnemyEngine::load(&root).unwrap();

    let enemy_file = read_json(&root.join("enemies.json"));
    let unresolved = read_json(&root.join("enemies_unresolved.json"));
    let defs = enemy_file["definitions"].as_array().unwrap();
    let mut definition_ids = HashSet::new();
    for def in defs {
        assert!(definition_ids.insert(def["definition_id"].as_i64().unwrap()));
    }

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lab = manifest.join("../../../lab");
    let locale: HashMap<u64, String> = read_json(&lab.join("reports/locale-id-to-text.json"))
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["id"].as_u64().unwrap(),
                row["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect();

    let mut models: HashMap<i64, HashSet<String>> = HashMap::new();
    let model_text =
        fs::read_to_string(lab.join("working/OSFR.Client/OSFR Client/Resources/Models.txt"))
            .unwrap();
    for line in model_text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('^').collect();
        let Ok(id) = fields[0].parse::<i64>() else {
            continue;
        };
        assert!(fields.len() > 1, "Models.txt row for {id} has no filename");
        models.entry(id).or_default().insert(fields[1].to_owned());
    }

    let catalog = fs::read_to_string(lab.join("extracted/client-assets/catalog.tsv")).unwrap();
    let asset_names: HashSet<_> = catalog
        .lines()
        .skip(1)
        .filter_map(|line| line.split('\t').nth(2))
        .map(str::to_ascii_lowercase)
        .collect();

    let bindings: HashMap<i64, &Value> = unresolved["model_asset_bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|binding| (binding["definition_id"].as_i64().unwrap(), binding))
        .collect();
    assert_eq!(bindings.len(), defs.len());

    let mut present = 0;
    let mut missing = HashSet::new();
    for def in defs {
        let definition_id = def["definition_id"].as_i64().unwrap();
        let model_id = def["model_id"].as_i64().unwrap();
        let name_id = def["name_string_id"].as_u64().unwrap();
        assert!(
            locale.contains_key(&name_id),
            "unresolved name id {name_id}"
        );
        assert_eq!(
            locale.get(&name_id).map(String::as_str),
            def["kind"].as_str(),
            "name id {name_id} does not resolve to the enemy kind"
        );

        let binding = bindings[&definition_id];
        let adr = binding["adr"].as_str().unwrap();
        assert_eq!(binding["model_id"].as_i64(), Some(model_id));
        assert!(
            models
                .get(&model_id)
                .is_some_and(|names| names.contains(adr)),
            "Models.txt has no {model_id} -> {adr} mapping"
        );
        assert!(binding["models_txt"]["line"].as_u64().unwrap() > 0);

        if binding["catalog_status"] == "present" {
            present += 1;
            assert!(
                asset_names.contains(&adr.to_ascii_lowercase()),
                "catalog missing {adr}"
            );
            assert!(binding["catalog"]["extracted_path"].as_str().is_some());
        } else {
            assert_eq!(binding["catalog_status"], "missing");
            missing.insert((model_id, adr.to_owned()));
        }
        if let Some(capture) = binding["capture"].as_object() {
            assert_eq!(capture["model_id"].as_i64(), Some(model_id));
            assert_eq!(capture["name_string_id"].as_u64(), Some(name_id));
            assert!(capture["sha256"].as_str().is_some());
            assert_eq!(binding["name_binding_status"], "client");
        }
    }

    let documented_gaps: HashSet<_> = unresolved["model_asset_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|gap| {
            (
                gap["model_id"].as_i64().unwrap(),
                gap["adr"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(missing, documented_gaps);
    assert_eq!(missing.len(), 2);
    assert_eq!(present + missing.len(), defs.len());
    assert_eq!(
        unresolved["coverage"]["client_model_table_pairs_checked"].as_u64(),
        Some(defs.len() as u64)
    );
    assert_eq!(
        unresolved["coverage"]["catalog_pairs_present"].as_u64(),
        Some(present as u64)
    );

    println!(
        "Enemy model pairs: {}/{} in Models.txt; {}/{} in catalog; {} documented catalog gaps.",
        defs.len(),
        defs.len(),
        present,
        defs.len(),
        missing.len()
    );
}

#[test]
fn every_dungeon_enemy_name_is_crosswalked_or_listed_unresolved() {
    let root = root();
    let directory = root.join("dungeons");
    let unresolved = read_json(&root.join("enemies_unresolved.json"));
    let unresolved_rows = unresolved["dungeon_rows"].as_array().unwrap();
    let unresolved_keys: HashSet<_> = unresolved_rows
        .iter()
        .map(|row| {
            row_key(
                row["dungeon_file"].as_str().unwrap(),
                row["source_field"].as_str().unwrap(),
                row["row_index"].as_u64().unwrap() as usize,
                row["name"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(unresolved_keys.len(), unresolved_rows.len());

    let enemy_file = read_json(&root.join("enemies.json"));
    let definition_ids: HashSet<_> = enemy_file["definitions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|def| def["definition_id"].as_i64().unwrap())
        .collect();
    let mut expected_unresolved = HashSet::new();
    let mut all = 0;
    let mut resolved = 0;
    let mut all_names = HashSet::new();
    let mut resolved_names = HashSet::new();
    let mut files: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(Result::unwrap)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();

    for path in files {
        let file = read_json(&path);
        let rows = file["records"].as_array().unwrap();
        assert_eq!(rows.len(), 1, "{}", path.display());
        let detail = &rows[0]["dungeon_data"];
        let refs = detail["enemy_refs"].as_array().unwrap();
        for field in ["enemies", "bosses"] {
            let Some(source_rows) = detail[field]["value"].as_array() else {
                continue;
            };
            for (index, row) in source_rows.iter().enumerate() {
                for name in row_names(row) {
                    all += 1;
                    all_names.insert(name.clone());
                    let matches: Vec<_> = refs
                        .iter()
                        .filter(|reference| {
                            reference["source_field"].as_str() == Some(field)
                                && reference["row_index"].as_u64() == Some(index as u64)
                                && reference["name"].as_str() == Some(name.as_str())
                        })
                        .collect();
                    assert_eq!(
                        matches.len(),
                        1,
                        "{} {field}[{index}] {name}",
                        path.display()
                    );
                    let reference = matches[0];
                    let key = row_key(
                        path.file_name().unwrap().to_str().unwrap(),
                        field,
                        index,
                        &name,
                    );
                    if reference["status"] == "resolved" {
                        resolved += 1;
                        resolved_names.insert(name.clone());
                        let id = reference["definition_id"].as_i64().unwrap();
                        assert!(definition_ids.contains(&id));
                        assert!(!unresolved_keys.contains(&key));
                    } else {
                        assert_eq!(reference["status"], "unresolved");
                        assert!(reference["definition_id"].is_null());
                        assert!(reference["reason"].as_str().is_some_and(|s| !s.is_empty()));
                        expected_unresolved.insert(key);
                    }
                }
            }
        }
    }
    assert_eq!(expected_unresolved, unresolved_keys);
    assert_eq!(
        all,
        unresolved["coverage"]["dungeon_name_occurrences"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        resolved,
        unresolved["coverage"]["resolved_dungeon_occurrences"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        all_names.len(),
        unresolved["coverage"]["unique_dungeon_names"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        resolved_names.len(),
        unresolved["coverage"]["resolved_unique_dungeon_names"]
            .as_u64()
            .unwrap() as usize
    );

    let before = &unresolved["coverage"]["before_w58b"];
    println!(
        "W58 -> W58b dungeon coverage: {}/{} occurrences and {}/{} unique names -> {resolved}/{all} occurrences and {}/{} unique names.",
        before["resolved_dungeon_occurrences"].as_u64().unwrap(),
        before["dungeon_name_occurrences"].as_u64().unwrap(),
        before["resolved_unique_dungeon_names"].as_u64().unwrap(),
        before["unique_dungeon_names"].as_u64().unwrap(),
        resolved_names.len(),
        all_names.len()
    );

    let world_rows = unresolved["world_rows"].as_array().unwrap();
    let mut world_resolved = 0;
    let mut world_partial = 0;
    for row in world_rows {
        let status = row["status"].as_str().unwrap();
        let ids = row["definition_ids"].as_array().unwrap();
        for id in ids {
            assert!(definition_ids.contains(&id.as_i64().unwrap()));
        }
        match status {
            "resolved" => {
                world_resolved += 1;
                assert!(!ids.is_empty());
                assert!(row["unresolved_names"].as_array().unwrap().is_empty());
            }
            "partial" => {
                world_partial += 1;
                assert!(!ids.is_empty());
                assert!(!row["unresolved_names"].as_array().unwrap().is_empty());
            }
            "unresolved" => {
                assert!(row["reason"].as_str().is_some_and(|s| !s.is_empty()));
            }
            other => panic!("unexpected regional crosswalk status {other}"),
        }
    }
    assert_eq!(world_rows.len(), 82);
    assert_eq!(
        world_resolved,
        unresolved["coverage"]["world_rows_resolved"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        world_partial,
        unresolved["coverage"]["world_rows_partial"]
            .as_u64()
            .unwrap() as usize
    );
    let before_world = &unresolved["coverage"]["before_w58b"];
    println!(
        "W58 -> W58b regional enemy_kind coverage: {}/{} rows -> {world_resolved}/{} fully resolved; {world_partial} partial rows.",
        before_world["resolved_regional_enemy_kind_rows"]
            .as_u64()
            .unwrap(),
        before_world["regional_enemy_kind_rows"].as_u64().unwrap(),
        world_rows.len()
    );
}
