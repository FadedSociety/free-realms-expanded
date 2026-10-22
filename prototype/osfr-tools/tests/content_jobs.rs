use osfr_tools::content;

use content::ContentDb;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
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

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

/// Item rows for every item the quests reference (W110 rewards come from the
/// generated catalogue, which this scratch content does not include).
fn referenced_items(quests: &[Value]) -> Value {
    let mut ids = std::collections::BTreeSet::new();
    for quest in quests {
        for reward in quest["rewards"].as_array().into_iter().flatten() {
            for item in reward["items"].as_array().into_iter().flatten() {
                ids.extend(item["definition_id"].as_i64());
            }
        }
        for goal in quest["objectives"].as_array().into_iter().flatten() {
            ids.extend(goal["item_id"].as_i64().or(goal["definition_id"].as_i64()));
        }
    }
    let source = json!([{"kind": "reconstructed", "evidence": "(R) test fixture: a quest-referenced catalogue item"}]);
    json!({"schema_version": 1, "records": ids.iter().map(|id| json!({
        "id": id, "name_string_id": 0, "description_string_id": null,
        "category_id": null, "source": source,
    })).collect::<Vec<_>>()})
}

fn empty_file() -> Value {
    json!({"schema_version": 1, "records": []})
}

#[test]
fn job_batch_loads_and_npc_references_resolve_or_are_declared() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let jobs: Value = serde_json::from_slice(&fs::read(root.join("jobs.json")).unwrap()).unwrap();
    let unresolved: Value =
        serde_json::from_slice(&fs::read(root.join("npcs_unresolved.json")).unwrap()).unwrap();
    let unresolved_rows = unresolved["records"].as_array().unwrap();
    let unresolved_ids: HashSet<u64> = unresolved_rows
        .iter()
        .map(|row| row["ref_id"].as_u64().unwrap())
        .collect();
    assert_eq!(jobs["records"].as_array().unwrap().len(), 17);

    let mut quest_records = Vec::new();
    for entry in fs::read_dir(root.join("quests")).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let file: Value = serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
        assert_eq!(file["schema_version"], 1, "{}", entry.path().display());
        quest_records.extend(file["records"].as_array().unwrap().iter().map(|record| {
            // The split quest files carry NPC-screen locale IDs as an extension;
            // ContentDb v1 validates only its established flat Quest schema.
            let mut record = record.clone();
            if let Some(record) = record.as_object_mut() {
                record.remove("intro_text_string_id");
                record.remove("description_string_id");
                record.remove("end_text_string_id");
                for goal in record["objectives"].as_array_mut().unwrap() {
                    goal.as_object_mut().unwrap().remove("text_string_id");
                }
            }
            record
        }));
    }
    // T466 added quests/zone_seaside.json, the first zone-quests slice.
    assert_eq!(fs::read_dir(root.join("quests")).unwrap().count(), 19);

    let mut referenced_npcs = HashSet::new();
    for job in jobs["records"].as_array().unwrap() {
        for npc_id in job["trainer_npc_ids"].as_array().unwrap() {
            referenced_npcs.insert(npc_id.as_u64().unwrap());
        }
    }
    for quest in &quest_records {
        referenced_npcs.insert(quest["giver_npc_id"].as_u64().unwrap());
        // Talk objectives name NPCs too (Call the Crew!: Shakey).
        for goal in quest["objectives"].as_array().unwrap() {
            if let Some(npc_id) = goal["npc_id"].as_u64() {
                referenced_npcs.insert(npc_id);
            }
        }
    }
    let mut resolved_ids = HashSet::new();
    for entry in fs::read_dir(root.join("npcs")).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let file: Value = serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
        for npc in file["records"].as_array().unwrap() {
            resolved_ids.insert(npc["npc_id"].as_u64().unwrap());
            resolved_ids.extend(
                npc["reference_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_u64().unwrap()),
            );
        }
    }
    assert!(
        referenced_npcs
            .iter()
            .all(|id| resolved_ids.contains(id) || unresolved_ids.contains(id)),
        "a job or quest NPC reference is neither resolved nor listed as unresolved"
    );

    // ContentDb v1 loads one flat quests.json and needs all eight datasets.
    // Assemble a temporary validation package from this batch plus its existing
    // empty schema fixtures; every referenced key receives a synthetic NPC row
    // here only so the unchanged loader can check every numeric relationship.
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch =
        std::env::temp_dir().join(format!("osfr-job-content-{}-{stamp}", std::process::id()));
    fs::create_dir(&scratch).unwrap();
    let _scratch = Scratch(scratch.clone());
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content");
    let zones: Value =
        serde_json::from_slice(&fs::read(fixtures.join("zones.json")).unwrap()).unwrap();
    write_json(&scratch.join("zones.json"), &zones);
    write_json(&scratch.join("jobs.json"), &jobs);
    write_json(
        &scratch.join("quests.json"),
        &json!({"schema_version": 1, "records": quest_records}),
    );
    write_json(&scratch.join("pois.json"), &empty_file());
    write_json(
        &scratch.join("items.json"),
        &referenced_items(&quest_records),
    );
    write_json(&scratch.join("dungeons.json"), &empty_file());
    write_json(&scratch.join("minigames.json"), &empty_file());

    let mut quest_ids_by_npc: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
    for quest in &quest_records {
        quest_ids_by_npc
            .entry(quest["giver_npc_id"].as_u64().unwrap())
            .or_default()
            .push(quest["id"].as_u64().unwrap() as u32);
    }
    let npc_rows: Vec<Value> = referenced_npcs
        .iter()
        .map(|actor_id| {
            json!({
                "actor_id": actor_id,
                "zone_id": 7,
                "model_id": 0,
                "name_string_id": 0,
                "transform": {
                    "position": [0.0, 0.0, 0.0],
                    "position_w": null,
                    "rotation": {"kind": "heading", "value": 0.0},
                    "scale": null
                },
                "role": ["quest_giver"],
                "quest_ids": quest_ids_by_npc.remove(actor_id).unwrap_or_default(),
                "source": [{
                    "kind": "reconstructed",
                    "evidence": "(R) temporary test adapter only; synthetic NPC row permits ContentDb v1 cross-reference validation"
                }]
            })
        })
        .collect();
    write_json(
        &scratch.join("npcs.json"),
        &json!({"schema_version": 1, "records": npc_rows}),
    );

    let db = ContentDb::load(&scratch).unwrap();
    assert_eq!(db.jobs.len(), 17);
    assert_eq!(db.quests.len(), quest_records.len());
    println!(
        "unresolved NPC references explicitly listed: {}",
        unresolved_ids.len()
    );
}
