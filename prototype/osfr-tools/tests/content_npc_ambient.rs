//! T507/T552: the ambient NPC table is bounded and every client reference exists.
//!
//! The row data is Sanctuary's `Resources/NpcAmbient.json`
//! (`Sanctuary-main-20261019@10f129e`, values only, no code). The ids must be
//! client `AnimationSlot` ids and the model ids must be the client `Models.txt`
//! ids that draw the row's file, so the runtime key is the AddNpc model id.
//! T552 adds conversation, idle and look-at rows to the same table.

#[path = "support/json.rs"]
mod json;
#[path = "support/models.rs"]
mod models;
#[path = "support/xml.rs"]
mod xml;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

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

fn slot_ids(
    slots: &HashSet<i32>,
    owner: &str,
    label: &str,
    ids: Option<&Vec<serde_json::Value>>,
) -> Vec<i64> {
    let Some(ids) = ids else {
        return Vec::new();
    };
    assert!(ids.len() <= 16, "{owner}: at most 16 {label} animation ids");
    let mut seen = HashSet::new();
    ids.iter()
        .map(|id| {
            let id = id
                .as_i64()
                .unwrap_or_else(|| panic!("{owner}: non-integer {label} id"));
            assert!(
                slots.contains(&i32::try_from(id).unwrap()),
                "{owner}: {label} slot {id}"
            );
            assert!(seen.insert(id), "{owner}: repeated {label} slot {id}");
            id
        })
        .collect()
}

#[test]
fn ambient_rows_name_client_slots_and_models() {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let document = json::read(&content.join("npc_ambient.json"));
    let slots = xml::ids(
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
    let conversation_radius = document["conversation_radius_m"]
        .as_f64()
        .expect("the conversation radius is a number");
    assert!(
        (1.0..=30.0).contains(&conversation_radius),
        "the conversation radius is 1..=30 m"
    );
    assert_eq!(
        conversation_radius, 6.0,
        "the reference's 6 m XZ grouping radius"
    );
    let rows = document["rows"].as_array().expect("rows is an array");
    assert_eq!(
        rows.len(),
        51,
        "every Sanctuary row, including the idle-only dog row"
    );

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
        let animations = slot_ids(&slots, &owner, "animation", row["animation_ids"].as_array());
        let conversations = slot_ids(
            &slots,
            &owner,
            "conversation",
            row.get("conversation_animation_ids")
                .and_then(serde_json::Value::as_array),
        );
        let idles = slot_ids(
            &slots,
            &owner,
            "idle",
            row.get("idle_animation_ids")
                .and_then(serde_json::Value::as_array),
        );
        let radius = row
            .get("look_at_radius")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);

        let min = row
            .get("min_interval_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let max = row
            .get("max_interval_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if animations.is_empty() {
            assert_eq!((min, max), (0, 0), "{owner}: no emote interval without ids");
        } else {
            assert!(500 <= min && min <= max && max <= 600_000, "{owner}");
        }

        let conversation_min = row
            .get("conversation_min_gap_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let conversation_max = row
            .get("conversation_max_gap_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if conversations.is_empty() {
            assert_eq!(
                (conversation_min, conversation_max),
                (0, 0),
                "{owner}: no conversation gap without ids"
            );
        } else {
            assert!(
                conversation_min <= conversation_max && conversation_max <= 60_000,
                "{owner}: conversation gaps are 0..=60000 ms"
            );
        }

        let idle_interval = row
            .get("idle_interval_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if idles.is_empty() {
            assert_eq!(idle_interval, 0, "{owner}: no idle interval without ids");
        } else {
            assert!(
                (500..=600_000).contains(&idle_interval),
                "{owner}: idle interval is 500..=600000 ms"
            );
        }

        let look_at_interval = row
            .get("look_at_interval_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if radius > 0.0 {
            assert!(
                (1.0..=30.0).contains(&radius),
                "{owner}: look-at radius is 1..=30 m"
            );
            assert!(
                (500..=600_000).contains(&look_at_interval),
                "{owner}: look-at interval is 500..=600000 ms"
            );
            assert!(
                announced.contains("LookAtRadius/LookAtIntervalMs"),
                "{owner}: look-at provenance"
            );
        } else {
            assert_eq!(radius, 0.0, "{owner}: no radius without a look-at");
            assert_eq!(
                look_at_interval, 0,
                "{owner}: no look-at interval without a radius"
            );
        }
        if !conversations.is_empty() {
            assert!(
                announced.contains("ConversationAnimationIds/ConversationMinGapMs"),
                "{owner}: conversation provenance"
            );
        }
        if !idles.is_empty() {
            assert!(
                announced.contains("IdleAnimationIds/IdleIntervalMs"),
                "{owner}: idle provenance"
            );
        }
        assert!(
            !animations.is_empty()
                || !conversations.is_empty()
                || !idles.is_empty()
                || radius > 0.0,
            "{owner}: a row schedules nothing"
        );

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
    assert_eq!(
        chugawug["conversation_animation_ids"],
        serde_json::json!([3302, 3306])
    );
    assert_eq!(chugawug["conversation_min_gap_ms"], 495);
    assert_eq!(chugawug["conversation_max_gap_ms"], 3702);

    // T552: the idle-only Sanctuary row and the two dog look-at rows.
    let pointy = rows
        .iter()
        .find(|row| row["model_file_name"] == "dog_small_pointy.adr")
        .expect("the idle-only dog row is imported");
    assert_eq!(pointy["model_ids"], serde_json::json!([173, 976]));
    assert_eq!(pointy["animation_ids"], serde_json::json!([]));
    assert_eq!(pointy["idle_animation_ids"], serde_json::json!([5403]));
    assert_eq!(pointy["idle_interval_ms"], 3400);
    let dalmation = rows
        .iter()
        .find(|row| row["model_file_name"] == "dog_large_dalmation.adr")
        .expect("the dalmatian look-at row is imported");
    assert_eq!(dalmation["look_at_radius"], 20);
    assert_eq!(dalmation["look_at_interval_ms"], 1250);
    let child = rows
        .iter()
        .find(|row| row["model_file_name"] == "humanchild_f_skirtandtunic.adr")
        .expect("the child conversation/idle row is imported");
    assert_eq!(
        child["conversation_animation_ids"],
        serde_json::json!([3302, 3306])
    );
    assert_eq!(child["conversation_min_gap_ms"], 170);
    assert_eq!(child["conversation_max_gap_ms"], 3118);
    assert_eq!(child["idle_animation_ids"], serde_json::json!([2100]));
    assert_eq!(child["idle_interval_ms"], 3621);
}
