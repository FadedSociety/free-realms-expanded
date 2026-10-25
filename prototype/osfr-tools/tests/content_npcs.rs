use osfr_tools::content;

use content::ContentDb;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
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
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
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

fn validate_sources(owner: &str, value: &Value) {
    let sources = value
        .as_array()
        .unwrap_or_else(|| panic!("{owner}: source tags must be an array"));
    assert!(!sources.is_empty(), "{owner}: source tags are empty");
    for source in sources {
        let kind = source["kind"].as_str().unwrap_or_default();
        let evidence = source["evidence"].as_str().unwrap_or_default();
        assert!(
            matches!(
                kind,
                "client" | "video" | "wiki" | "reconstructed" | "legacy-data"
            ),
            "{owner}: invalid source kind {kind:?}"
        );
        assert!(
            !evidence.trim().is_empty(),
            "{owner}: empty source evidence"
        );
        if kind == "reconstructed" {
            assert!(
                evidence.starts_with("(R)"),
                "{owner}: reconstructed evidence must start with (R)"
            );
        }
    }
}

fn content_npc_row(
    actor_id: u64,
    zone_id: u32,
    name_string_id: u32,
    roles: &[String],
    quest_ids: &[u32],
    source: &Value,
) -> Value {
    let mut loader_roles = Vec::new();
    for role in roles {
        let mapped = match role.as_str() {
            "vendor" => Some("vendor"),
            "quest_giver" | "trainer" => Some("quest_giver"),
            "ambient" | "enemy" => Some("ambient"),
            _ => None,
        };
        if let Some(mapped) = mapped {
            if !loader_roles.iter().any(|existing| *existing == mapped) {
                loader_roles.push(mapped);
            }
        }
    }
    if loader_roles.is_empty() {
        loader_roles.push("ambient");
    }

    json!({
        "actor_id": actor_id,
        "zone_id": zone_id,
        "model_id": 0,
        "name_string_id": name_string_id,
        "transform": {
            "position": [0.0, 0.0, 0.0],
            "position_w": null,
            "rotation": {"kind": "heading", "value": 0.0},
            "scale": null
        },
        "role": loader_roles,
        "quest_ids": quest_ids,
        "source": source
    })
}

#[test]
fn regional_npc_batch_and_resolved_job_references_validate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let zones = read_json(&root.join("zones.json"));
    let zone_rows = zones["records"].as_array().unwrap();
    assert_eq!(zone_rows.len(), 17);
    let mut zone_ids = HashSet::new();
    let mut zone_id_by_name = HashMap::new();
    for zone in zone_rows {
        validate_sources("zone", &zone["source"]);
        let id = zone["id"].as_u64().unwrap() as u32;
        let name = zone["name"].as_str().unwrap().to_owned();
        assert!(zone_ids.insert(id), "duplicate zone id {id}");
        zone_id_by_name.insert(name, id);
    }
    let fallback_zone = zone_rows[0]["id"].as_u64().unwrap() as u32;

    let mut npc_files: Vec<PathBuf> = fs::read_dir(root.join("npcs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    npc_files.sort();
    assert_eq!(npc_files.len(), 17);

    let mut canonical_ids = HashSet::new();
    let mut resolved_reference_ids = HashSet::new();
    let mut reference_owner = HashMap::<u64, u64>::new();
    let mut content_npcs = Vec::<(Value, u64, u32)>::new();
    let mut position_known = 0usize;
    let mut position_unknown = 0usize;
    for path in &npc_files {
        let document = read_json(path);
        assert_eq!(document["schema_version"], 1, "{}", path.display());
        let region = document["region_page"].as_str().unwrap();
        let region_zone_id = document["region_zone_id"].as_u64().unwrap() as u32;
        assert_eq!(zone_id_by_name.get(region), Some(&region_zone_id));
        for npc in document["records"].as_array().unwrap() {
            let npc_id = npc["npc_id"].as_u64().unwrap();
            assert!(canonical_ids.insert(npc_id), "duplicate npc_id {npc_id}");
            reference_owner.insert(npc_id, npc_id);
            validate_sources(&format!("NPC {npc_id}"), &npc["source"]);

            let name = npc["name"].as_str().unwrap_or_default();
            assert!(!name.trim().is_empty(), "NPC {npc_id}: missing name");
            assert!(
                npc["roles"]
                    .as_array()
                    .is_some_and(|roles| !roles.is_empty()),
                "NPC {npc_id}: missing role"
            );
            if npc["roles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|role| role == "enemy")
            {
                assert!(
                    npc["enemy_kind"]
                        .as_str()
                        .is_some_and(|kind| !kind.trim().is_empty()),
                    "NPC {npc_id}: enemy role has no kind"
                );
            }
            let npc_fields = npc.as_object().unwrap();
            let tagged_fields = npc["field_sources"].as_object().unwrap();
            assert_eq!(tagged_fields.len(), npc_fields.len() - 2, "NPC {npc_id}");
            for field in npc_fields.keys() {
                if field != "source" && field != "field_sources" {
                    assert!(
                        tagged_fields.contains_key(field),
                        "NPC {npc_id}: missing field source for {field}"
                    );
                }
            }
            for field_sources in tagged_fields.values() {
                validate_sources(&format!("NPC {npc_id} field"), field_sources);
            }
            for field in [
                "npc_id",
                "name",
                "name_string_id",
                "name_string_ids",
                "zone",
                "roles",
                "quests_given",
                "quests_accepted",
                "vendor_stock",
                "services",
                "position",
            ] {
                assert!(
                    npc["field_sources"].get(field).is_some(),
                    "NPC {npc_id}: missing source tags for {field}"
                );
            }
            if npc["zone"].is_null() {
                assert!(
                    npc["zone_reason"]
                        .as_str()
                        .is_some_and(|reason| !reason.trim().is_empty()),
                    "NPC {npc_id}: unknown zone has no reason"
                );
            } else {
                let zone = npc["zone"].as_str().unwrap();
                assert_eq!(
                    npc["zone_id"].as_u64().map(|id| id as u32),
                    zone_id_by_name.get(zone).copied(),
                    "NPC {npc_id}: zone name/id mismatch"
                );
            }
            if npc["position"].is_null() {
                position_unknown += 1;
                assert!(
                    npc["position_reason"]
                        .as_str()
                        .is_some_and(|reason| !reason.trim().is_empty()),
                    "NPC {npc_id}: unknown position has no reason"
                );
            } else {
                position_known += 1;
                let tags = npc["field_sources"]["position"].as_array().unwrap();
                assert!(
                    tags.iter().any(|tag| {
                        tag["kind"] == "client"
                            || tag["kind"] == "video"
                            || tag["kind"] == "legacy-data"
                    }),
                    "NPC {npc_id}: position lacks a client/video source"
                );
            }
            if !npc["vendor_stock"].is_null() {
                assert!(
                    npc["roles"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|role| role == "vendor")
                );
            }
            let aliases: Vec<u64> = npc["reference_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| id.as_u64().unwrap())
                .collect();
            for alias in aliases {
                resolved_reference_ids.insert(alias);
                if let Some(owner) = reference_owner.insert(alias, npc_id) {
                    assert_eq!(owner, npc_id, "reference id {alias} maps to two NPCs");
                }
            }
            let zone_id = npc["zone_id"]
                .as_u64()
                .map_or(fallback_zone, |id| id as u32);
            content_npcs.push((npc.clone(), npc_id, zone_id));
        }
    }

    let unresolved = read_json(&root.join("npcs_unresolved.json"));
    assert_eq!(unresolved["schema_version"], 1);
    let unresolved_rows = unresolved["records"].as_array().unwrap();
    let mut unresolved_ids = HashSet::new();
    for row in unresolved_rows {
        let id = row["ref_id"].as_u64().unwrap();
        assert!(
            unresolved_ids.insert(id),
            "duplicate unresolved reference {id}"
        );
        assert!(
            !resolved_reference_ids.contains(&id),
            "resolved reference {id} remains unresolved"
        );
        validate_sources(&format!("unresolved NPC {id}"), &row["source"]);
    }
    assert_eq!(resolved_reference_ids.len() + unresolved_ids.len(), 465);

    let jobs = read_json(&root.join("jobs.json"));
    assert_eq!(jobs["records"].as_array().unwrap().len(), 17);
    let mut trainer_reference_ids = HashSet::new();
    for job in jobs["records"].as_array().unwrap() {
        for id in job["trainer_npc_ids"].as_array().unwrap() {
            trainer_reference_ids.insert(id.as_u64().unwrap());
        }
    }
    for id in &trainer_reference_ids {
        assert!(
            resolved_reference_ids.contains(id) || unresolved_ids.contains(id),
            "job trainer reference {id} is neither resolved nor unresolved"
        );
    }

    let mut quest_files: Vec<PathBuf> = fs::read_dir(root.join("quests"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    quest_files.sort();
    // T466 added quests/zone_seaside.json (five records), the first zone-quests slice;
    // T498 added quests/zone_wilds.json (four records); T500 added quests/zone_sanctuary.json (three);
    // T501 added quests/zone_merry_vale.json (five); T502 added zone_wugachug_briarwood.json (five).
    assert_eq!(quest_files.len(), 23);
    let mut quest_records = Vec::new();
    let mut quest_ids_by_giver = BTreeMap::<u64, Vec<u32>>::new();
    let mut resolved_giver_occurrences = 0usize;
    let mut unresolved_giver_occurrences = 0usize;
    for path in &quest_files {
        let document = read_json(path);
        assert_eq!(document["schema_version"], 1, "{}", path.display());
        for quest in document["records"].as_array().unwrap() {
            let giver = quest["giver_npc_id"].as_u64().unwrap();
            let quest_id = quest["id"].as_u64().unwrap() as u32;
            if unresolved_ids.contains(&giver) {
                unresolved_giver_occurrences += 1;
            } else {
                resolved_giver_occurrences += 1;
                assert!(
                    reference_owner.contains_key(&giver),
                    "quest giver {giver} is not an NPC id or reference alias"
                );
            }
            quest_ids_by_giver.entry(giver).or_default().push(quest_id);
            // NPC-screen locale IDs extend the split quest format. The temporary
            // ContentDb v1 adapter validates the established flat Quest schema.
            let mut quest = quest.clone();
            if let Some(quest) = quest.as_object_mut() {
                quest.remove("intro_text_string_id");
                quest.remove("description_string_id");
                quest.remove("end_text_string_id");
                for goal in quest["objectives"].as_array_mut().unwrap() {
                    goal.as_object_mut().unwrap().remove("text_string_id");
                }
            }
            quest_records.push(quest);
        }
    }

    // Project the page data into the unchanged strict ContentDb v1 NPC shape.
    // Zero transforms and fallback zones below exist only in this temporary loader adapter.
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch =
        std::env::temp_dir().join(format!("osfr-npc-content-{}-{stamp}", std::process::id()));
    fs::create_dir(&scratch).unwrap();
    let _scratch = Scratch(scratch.clone());
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

    let mut loader_npcs = Vec::new();
    let mut emitted_actor_ids = HashSet::new();
    for (npc, canonical_id, zone_id) in &content_npcs {
        let name_id = npc["name_string_id"]
            .as_u64()
            .or_else(|| {
                npc["name_string_ids"]
                    .as_array()
                    .and_then(|ids| ids.first())
                    .and_then(|item| item["id"].as_u64())
            })
            .unwrap_or(0) as u32;
        let aliases = npc["reference_ids"].as_array().unwrap();
        let mut actor_ids = BTreeSet::from([*canonical_id]);
        actor_ids.extend(aliases.iter().map(|id| id.as_u64().unwrap()));
        let roles: Vec<String> = npc["roles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|role| role.as_str().unwrap().to_owned())
            .collect();
        let adapter_source = if npc["zone_id"].is_null() {
            let mut tags = npc["source"].as_array().unwrap().clone();
            tags.push(json!({
                "kind": "reconstructed",
                "evidence": "(R) temporary ContentDb adapter uses a fallback zone for this unknown-zone NPC; not a placement claim"
            }));
            Value::Array(tags)
        } else {
            npc["source"].clone()
        };
        for actor_id in actor_ids {
            assert!(
                emitted_actor_ids.insert(actor_id),
                "duplicate loader actor id {actor_id}"
            );
            let quest_ids = quest_ids_by_giver
                .get(&actor_id)
                .cloned()
                .unwrap_or_default();
            loader_npcs.push(content_npc_row(
                actor_id,
                *zone_id,
                name_id,
                &roles,
                &quest_ids,
                &adapter_source,
            ));
        }
    }
    for row in unresolved_rows {
        let actor_id = row["ref_id"].as_u64().unwrap();
        assert!(
            emitted_actor_ids.insert(actor_id),
            "duplicate unresolved actor id {actor_id}"
        );
        let row_zone = row["zone"]
            .as_str()
            .and_then(|name| zone_id_by_name.get(name).copied());
        let adapter_source = json!([{
            "kind": "reconstructed",
            "evidence": "(R) temporary ContentDb adapter placeholder for a still-unresolved reference; not an NPC data record"
        }]);
        loader_npcs.push(content_npc_row(
            actor_id,
            row_zone.unwrap_or(fallback_zone),
            row["name_string_id"].as_u64().unwrap_or(0) as u32,
            &["quest_giver".to_owned()],
            quest_ids_by_giver
                .get(&actor_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            &adapter_source,
        ));
    }
    // T379 appended two unanchored repair-guy binding rows.
    // T498 appended the Burglz row (the Wilds quarry troll) to npcs/wilds.json; T500 appended Hasti
    // to npcs/sanctuary.json (1,203); T501 appended Shannon and Sugaboo to npcs/merry_vale.json (1,205).
    assert_eq!(content_npcs.len(), 1205);
    let expected_npc_count = loader_npcs.len();
    write_json(
        &scratch.join("npcs.json"),
        &json!({"schema_version": 1, "records": loader_npcs}),
    );

    let db = ContentDb::load(&scratch).unwrap();
    assert_eq!(db.zones.len(), 17);
    assert_eq!(db.npcs.len(), expected_npc_count);
    assert_eq!(db.jobs.len(), 17);
    assert_eq!(db.quests.len(), 677); // T498: four Wilds zone quests; T500: three Sanctuary ones; T501: five Merry Vale ones; T502: five Wugachug/Briarwood ones
    assert_eq!(resolved_reference_ids.len(), 295);
    assert_eq!(unresolved_ids.len(), 170);
    assert_eq!(
        resolved_giver_occurrences + unresolved_giver_occurrences,
        677
    );
    println!(
        "NPC refs: {} resolved, {} unresolved; quest giver occurrences: {} resolved, {} unresolved; positions: {} known, {} unknown",
        resolved_reference_ids.len(),
        unresolved_ids.len(),
        resolved_giver_occurrences,
        unresolved_giver_occurrences,
        position_known,
        position_unknown
    );
}
