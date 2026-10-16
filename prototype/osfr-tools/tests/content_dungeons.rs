use content::{ContentDb, Dungeon};
use osfr_tools::content;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn source_tag(kind: &str, evidence: &str) -> Value {
    json!({"kind": kind, "evidence": evidence})
}

fn records(value: &Value) -> &[Value] {
    value["records"].as_array().unwrap()
}

fn key(row: &Value) -> (String, String, String, String) {
    (
        row["owner"].as_str().unwrap().to_owned(),
        row["kind"].as_str().unwrap().to_owned(),
        row["target_name"].as_str().unwrap_or("").to_owned(),
        row["target_id"]
            .as_u64()
            .map(|id| id.to_string())
            .unwrap_or_default(),
    )
}

fn check_source_tags(value: &Value, path: &str) {
    match value {
        Value::Array(rows) => {
            for (index, row) in rows.iter().enumerate() {
                check_source_tags(row, &format!("{path}[{index}]"));
            }
        }
        Value::Object(fields) => {
            if let Some(sources) = fields.get("source") {
                let sources = sources
                    .as_array()
                    .unwrap_or_else(|| panic!("{path}.source must be an array"));
                assert!(!sources.is_empty(), "{path}.source must not be empty");
                for source in sources {
                    let kind = source["kind"].as_str().unwrap();
                    let evidence = source["evidence"].as_str().unwrap();
                    assert!(!evidence.trim().is_empty(), "{path} has blank evidence");
                    if kind == "reconstructed" {
                        assert!(evidence.starts_with("(R)"), "{path}: {evidence}");
                    }
                }
            }
            if fields.get("value") == Some(&Value::Null) {
                assert!(
                    fields
                        .get("reason")
                        .and_then(Value::as_str)
                        .is_some_and(|s| !s.trim().is_empty()),
                    "{path}: null value needs a reason"
                );
            }
            for (name, child) in fields {
                check_source_tags(child, &format!("{path}.{name}"));
            }
        }
        _ => {}
    }
}

#[derive(Default)]
struct Catalogues {
    zones: HashSet<u64>,
    zone_names: HashSet<String>,
    npc_actor_ids: HashSet<u64>,
    npc_content_ids: HashSet<u64>,
    npc_names: HashMap<String, HashSet<u64>>,
    item_ids: HashSet<u64>,
}

impl Catalogues {
    fn load(root: &Path) -> Self {
        let mut result = Self::default();
        let zones_path = root.join("zones.json");
        if zones_path.is_file() {
            for row in read_json(&zones_path)["records"]
                .as_array()
                .into_iter()
                .flatten()
            {
                if let Some(id) = row["id"].as_u64() {
                    result.zones.insert(id);
                }
                if let Some(name) = row["name"].as_str() {
                    result.zone_names.insert(name.to_owned());
                }
            }
        }
        let npcs_path = root.join("npcs.json");
        if npcs_path.is_file() {
            for row in read_json(&npcs_path)["records"]
                .as_array()
                .into_iter()
                .flatten()
            {
                if let Some(id) = row["actor_id"].as_u64() {
                    result.npc_actor_ids.insert(id);
                }
            }
        }
        let npc_dir = root.join("npcs");
        if npc_dir.is_dir() {
            for entry in fs::read_dir(npc_dir).unwrap().filter_map(Result::ok) {
                let path = entry.path();
                if !path.extension().is_some_and(|ext| ext == "json") {
                    continue;
                }
                for row in read_json(&path)["records"].as_array().into_iter().flatten() {
                    let Some(id) = row["npc_id"].as_u64() else {
                        continue;
                    };
                    result.npc_content_ids.insert(id);
                    if let Some(name) = row["name"].as_str() {
                        result
                            .npc_names
                            .entry(name.to_lowercase())
                            .or_default()
                            .insert(id);
                    }
                }
            }
        }
        let items_path = root.join("items.json");
        if items_path.is_file() {
            for row in read_json(&items_path)["records"]
                .as_array()
                .into_iter()
                .flatten()
            {
                if let Some(id) = row["id"].as_u64() {
                    result.item_ids.insert(id);
                }
            }
        }
        result
    }

    fn resolves(&self, reference: &Value) -> bool {
        let kind = reference["kind"].as_str().unwrap_or("");
        if let Some(id) = reference["target_id"].as_u64() {
            return match kind {
                "zone" => self.zones.contains(&id),
                "npc" => self.npc_actor_ids.contains(&id) || self.npc_content_ids.contains(&id),
                "item" => self.item_ids.contains(&id),
                _ => false,
            };
        }
        let name = reference["target_name"].as_str().unwrap_or("");
        match kind {
            "zone" => self.zone_names.contains(name),
            "npc" => self
                .npc_names
                .get(&name.to_lowercase())
                .is_some_and(|ids| ids.len() == 1),
            _ => false,
        }
    }
}

fn validation_fixture(root: &Path, dungeons: &[Dungeon]) -> Scratch {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "osfr-dungeon-content-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    let scratch = Scratch(dir.clone());
    let fixtures = root.join("tests/fixtures/content");
    for entry in fs::read_dir(fixtures).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), dir.join(entry.file_name())).unwrap();
    }

    let zones_path = dir.join("zones.json");
    let mut zones = read_json(&zones_path);
    let zone_rows = zones["records"].as_array_mut().unwrap();
    if !zone_rows.iter().any(|row| row["id"] == 411) {
        zone_rows.push(json!({
            "id": 411,
            "name": "sg_npx_cavern_01",
            "geometry_id": 214,
            "zone_type": null,
            "tutorial": true,
            "sky": "sky_blackspore24.xml",
            "source": [source_tag("reconstructed", "(R) Test-only row mirrors the private opening cave content so ContentDb can validate its zone reference")]
        }));
    }
    write_json(&zones_path, &zones);

    let items_path = dir.join("items.json");
    let mut items = read_json(&items_path);
    let item_rows = items["records"].as_array_mut().unwrap();
    if !item_rows.iter().any(|row| row["id"] == 1195) {
        item_rows.push(json!({
            "id": 1195,
            "name_string_id": 0,
            "description_string_id": null,
            "category_id": null,
            "source": [source_tag("reconstructed", "(R) Test-only placeholder for the opening cave's private configured reward item")]
        }));
    }
    write_json(&items_path, &items);
    write_json(
        &dir.join("minigames.json"),
        &json!({"schema_version": 1, "records": []}),
    );
    write_json(
        &dir.join("dungeons.json"),
        &json!({"schema_version": 1, "records": dungeons}),
    );
    scratch
}

#[test]
fn every_dungeon_file_loads_and_validates() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join("data/content");
    let directory = root.join("dungeons");
    let mut paths: Vec<_> = fs::read_dir(&directory)
        .unwrap()
        .map(Result::unwrap)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 75, "74 guide pages plus opening_cave.json");

    let mut dungeons = Vec::new();
    let mut references = Vec::new();
    let mut ids = HashSet::new();
    for path in paths {
        let file = read_json(&path);
        assert_eq!(file["schema_version"], 1, "{}", path.display());
        let rows = records(&file);
        assert_eq!(rows.len(), 1, "{}", path.display());
        check_source_tags(&rows[0], &path.display().to_string());
        let dungeon: Dungeon = serde_json::from_value(rows[0]["record"].clone()).unwrap();
        assert_eq!(
            path.file_stem().unwrap().to_string_lossy(),
            dungeon.id,
            "{}",
            path.display()
        );
        assert!(
            ids.insert(dungeon.id.clone()),
            "duplicate dungeon {}",
            dungeon.id
        );
        assert!(rows[0]["dungeon_data"]["entrance"].is_object());
        assert!(rows[0]["dungeon_data"].get("instance_scene_name").is_some());
        assert!(rows[0]["dungeon_data"].get("goals").is_some());
        assert!(rows[0]["dungeon_data"].get("phases").is_some());
        assert!(rows[0]["dungeon_data"].get("enemies").is_some());
        assert!(rows[0]["dungeon_data"].get("bosses").is_some());
        assert!(rows[0]["dungeon_data"].get("rewards").is_some());
        assert!(rows[0]["dungeon_data"].get("exits").is_some());
        references.extend(
            rows[0]["dungeon_data"]["references"]
                .as_array()
                .unwrap()
                .iter()
                .cloned(),
        );
        dungeons.push(dungeon);
    }

    let scratch = validation_fixture(&manifest.join("."), &dungeons);
    let db = ContentDb::load(&scratch.0).unwrap();
    assert_eq!(db.dungeons.len(), 75);
}

#[test]
fn external_references_resolve_or_are_listed() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest.join("data/content");
    let directory = root.join("dungeons");
    let unresolved = read_json(&root.join("dungeons_unresolved.json"));
    assert_eq!(unresolved["schema_version"], 1);
    let unresolved_rows = unresolved["references"].as_array().unwrap();
    let unresolved_keys: HashSet<_> = unresolved_rows.iter().map(key).collect();
    assert_eq!(
        unresolved_keys.len(),
        unresolved_rows.len(),
        "duplicate unresolved row"
    );

    let mut all_keys = HashSet::new();
    let mut expected_unresolved = HashSet::new();
    let catalogues = Catalogues::load(&root);
    for entry in fs::read_dir(&directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let file = read_json(&path);
        for reference in file["records"][0]["dungeon_data"]["references"]
            .as_array()
            .unwrap()
        {
            let reference_key = key(reference);
            assert!(
                all_keys.insert(reference_key.clone()),
                "duplicate reference {reference_key:?}"
            );
            if catalogues.resolves(reference) {
                assert!(
                    !unresolved_keys.contains(&reference_key),
                    "resolved reference is still unresolved: {reference_key:?}"
                );
            } else {
                expected_unresolved.insert(reference_key.clone());
                assert!(
                    unresolved_keys.contains(&reference_key),
                    "missing unresolved reference: {reference_key:?}"
                );
            }
        }
    }
    assert_eq!(
        expected_unresolved, unresolved_keys,
        "unresolved list must match current unresolved references"
    );
}

#[test]
fn opening_cave_matches_the_native_accepted_content_definition() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path =
        manifest.join("../../../lab/references/opening-journal-text-v2/content/encounters.json");
    let source = read_json(&source_path);
    let expected = source["dungeons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == 411)
        .expect("opening cave encounter 411 in accepted content");
    let converted = read_json(&manifest.join("data/content/dungeons/opening_cave.json"));
    let row = &converted["records"][0];
    let native = &row["dungeon_data"]["native_config"];

    assert_eq!(native["encounter_id"], expected["id"]);
    assert_eq!(native["title_id"], expected["title_id"]);
    assert_eq!(native["description_id"], expected["description_id"]);
    assert_eq!(native["objective_id"], expected["objective_id"]);
    assert_eq!(native["zone"]["value"], expected["zone"]);
    assert_eq!(native["guide"]["value"], expected["guide"]);
    assert_eq!(native["enemies"]["value"], expected["enemies"]);
    assert_eq!(native["reward_grant"]["coins"], expected["reward_coins"]);
    let converted_items: Vec<_> = native["reward_grant"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| json!([item["definition_id"], item["count"]]))
        .collect();
    assert_eq!(Value::Array(converted_items), expected["reward_items"]);

    assert_eq!(
        row["dungeon_data"]["instance_scene_name"]["value"],
        expected["zone"]["name"]
    );
    assert_eq!(
        row["dungeon_data"]["entrance"]["initiator"]["id"]["value"],
        expected["guide"]["guid"]
    );
    assert_eq!(
        row["dungeon_data"]["entrance"]["initiator"]["model_id"]["value"],
        expected["guide"]["model_id"]
    );
    let converted_enemies = row["dungeon_data"]["enemies"]["value"].as_array().unwrap();
    let expected_enemies = expected["enemies"].as_array().unwrap();
    assert_eq!(converted_enemies.len(), expected_enemies.len());
    for (converted, expected) in converted_enemies.iter().zip(expected_enemies) {
        assert_eq!(converted["name"]["value"], expected["name"]);
        assert_eq!(
            converted["definition_id"]["value"],
            expected["definition_id"]
        );
        assert_eq!(converted["model_id"]["value"], expected["model_id"]);
        assert_eq!(converted["name_string_id"]["value"], expected["name_id"]);
        assert_eq!(converted["guid"]["value"], expected["guid"]);
        assert_eq!(
            converted["stats"]["value"]["max_health"],
            expected["max_health"]
        );
        assert_eq!(converted["stats"]["value"]["damage"], expected["damage"]);
        assert_eq!(converted["boss"]["value"], expected["boss"]);
    }

    let record: Dungeon = serde_json::from_value(row["record"].clone()).unwrap();
    assert_eq!(record.id, "opening_cave");
    assert_eq!(
        record.name_string_id,
        Some(expected["title_id"].as_i64().unwrap() as u32)
    );
    assert_eq!(
        record.zone_id,
        Some(expected["zone"]["zone_id"].as_u64().unwrap() as u32)
    );
    assert_eq!(
        record.encounters[0].rewards[0].coins,
        expected["reward_coins"]
    );
    let record_items: Vec<_> = record.encounters[0].rewards[0]
        .items
        .iter()
        .map(|item| json!([item.definition_id, item.count]))
        .collect();
    assert_eq!(Value::Array(record_items), expected["reward_items"]);
}
