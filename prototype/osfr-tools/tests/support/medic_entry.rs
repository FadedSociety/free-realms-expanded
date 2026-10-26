//! Shared Medic content vocabulary and checks.
//!
//! `medic_entry_chain.rs` includes this with
//! `#[path = "support/medic_entry.rs"] mod medic_entry;` and re-exports the
//! constants. Test-only; production never copies content.
#![allow(dead_code)]

use super::npc_quest_chain::{content_dir, content_position, read_json};
use serde_json::Value;
use std::fs;

/// A second character id for the per-character ledger test.
pub const SECOND_CHARACTER: u64 = 194_018;
pub const NAIA: u64 = 9_000_000_161;
pub const GORDON: u64 = 9_000_000_073;
pub const GREYLI: u64 = 9_500_000_488;
pub const MEDIC: i32 = 11;
pub const BRAWLER: i32 = 43;
pub const EXAM: u32 = 3_090_001;
pub const HOUSE_CALL: u32 = 3_090_003;
pub const PILLS: u32 = 3_090_004;
pub const VITAMINS: u32 = 3_090_005;
pub const LEVEL_FIVE: u32 = 3_090_006;
/// The next Medic record; it stays disabled in this slice (no friendly-NPC
/// Target Vitals path), and no later record may become offerable by accident.
pub const SURGICAL_PRECISION: u32 = 3_090_008;
pub const SUNFLOWER: u64 = 9_500_000_536;
pub const EXAM_COINS: u32 = 5;
pub const EXAM_XP: u32 = 15;
pub const HOUSE_COINS: u32 = 10;
pub const HOUSE_XP: u32 = 20;
pub const PILL_COINS: u32 = 5;
pub const PILL_XP: u32 = 20;
pub const VITAMIN_COINS: u32 = 15;
pub const VITAMIN_XP: u32 = 25;
pub const LEVEL_COINS: u32 = 5;
pub const LEVEL_XP: u32 = 25;
/// Client title and text ids for the two level-five records; every value resolves
/// in the client locale table (77437/387463 are the titles the guide names).
pub const PILL_TITLE: u32 = 77_437;
pub const PILL_INTRO_TEXT: u32 = 77_436;
pub const PILL_DESCRIPTION: u32 = 77_438;
pub const PILL_END_TEXT: u32 = 77_439;
pub const LEVEL_TITLE: u32 = 387_463;
pub const LEVEL_INTRO_TEXT: u32 = 387_462;
pub const LEVEL_DESCRIPTION: u32 = 387_464;
pub const LEVEL_END_TEXT: u32 = 387_465;
pub const SANCTUARY: u32 = 10;
/// `job_trainers.json` row for Nurse Naia (profile 11).
pub const LEARN_LABEL: u32 = 412_422;
/// Client completion line for 3090001 ("...has shown Nurse Naia..."):
/// raw locale hash 219584158 = Global.Text 24259.
pub const EXAM_END_TEXT: u32 = 24_259;
/// Client completion line for 3090003 ("By helping out with Doctor Gordon's
/// workload..."): raw locale hash 1171210927 = Global.Text 24266.
pub const HOUSE_END_TEXT: u32 = 24_266;

/// The chapter content check; the protected chain test calls it (the integrator pins that name).
pub fn assert_enabled_medic_chapter_records() {
    let document = read_json(&content_dir().join("quests/medic.json"));
    let records = document["records"].as_array().unwrap();
    let enabled: Vec<&Value> = records
        .iter()
        .filter(|record| record["enabled"].as_bool().unwrap_or(false))
        .collect();
    let enabled_ids: Vec<u64> = enabled
        .iter()
        .map(|record| record["id"].as_u64().unwrap())
        .collect();
    assert_eq!(
        enabled_ids,
        vec![
            u64::from(EXAM),
            u64::from(HOUSE_CALL),
            u64::from(PILLS),
            u64::from(VITAMINS),
            u64::from(LEVEL_FIVE)
        ]
    );

    // Every enabled record has a spawned giver whose authored NPC row names the
    // quest in both arrays and carries the record's client title id, the value
    // the dialogue title and turn-in option are built from.
    let mut bindings = std::collections::BTreeMap::new();
    for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let npcs = read_json(&path);
        for row in npcs["records"].as_array().unwrap() {
            let id = row["npc_id"].as_u64().unwrap();
            let quest_ids_given: Vec<u64> = row["quest_ids_given"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|id| id.as_u64())
                .collect();
            let quests_given: Vec<(u64, Option<u64>)> = row["quests_given"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|row| {
                    row["quest_id"]
                        .as_u64()
                        .map(|id| (id, row["title_string_id"].as_u64()))
                })
                .collect();
            bindings.insert(id, (quest_ids_given, quests_given));
            if [NAIA, GORDON, GREYLI].contains(&id) {
                assert_eq!(
                    row["placement"].as_str(),
                    Some("legacy_data"),
                    "NPC {id} must be a spawned Legacy row"
                );
                assert_eq!(row["zone_id"].as_u64(), Some(u64::from(SANCTUARY)));
                assert!(row["position"].is_array());
                assert!(
                    row["roles"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|role| role == "quest_giver"),
                    "NPC {id} must be a quest giver"
                );
                assert!(
                    row["model_id"].as_u64().is_some_and(|model| model > 0),
                    "NPC {id} must have a renderable Legacy model"
                );
                assert!(
                    row["heading"].as_f64().is_some_and(f64::is_finite),
                    "NPC {id} must have a Legacy heading"
                );
                if id == GORDON {
                    assert_eq!(
                        row["model_file_name"].as_str(),
                        Some("human_m_doctor_gordon.agr")
                    );
                    assert_eq!(row["name_string_id"].as_u64(), Some(7_610));
                }
            }
        }
    }
    for record in &enabled {
        let quest_id = record["id"].as_u64().unwrap();
        let giver = record["giver_npc_id"].as_u64().unwrap();
        let (quest_ids_given, quests_given) = bindings
            .get(&giver)
            .unwrap_or_else(|| panic!("quest {quest_id}: giver {giver} is not a spawned NPC"));
        assert!(
            quest_ids_given.contains(&quest_id),
            "quest {quest_id}: giver {giver} omits it from quest_ids_given"
        );
        let title_string_id = quests_given
            .iter()
            .find_map(|(id, title)| (*id == quest_id).then_some(*title))
            .unwrap_or_else(|| {
                panic!("quest {quest_id}: giver {giver} omits it from quests_given")
            });
        assert_eq!(
            title_string_id,
            record["title_string_id"].as_u64(),
            "quest {quest_id}: giver {giver}'s quests_given title must match the quest record"
        );
        assert_eq!(record["job"].as_i64(), Some(i64::from(MEDIC)));
        let objectives = record["objectives"].as_array().unwrap();
        assert!(!objectives.is_empty(), "quest {quest_id} needs objectives");
        assert!(
            objectives.iter().all(|goal| goal["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "quest {quest_id}: every objective needs provenance"
        );
        let rewards = record["rewards"].as_array().unwrap();
        assert!(!rewards.is_empty(), "quest {quest_id} must pay");
        assert!(
            rewards.iter().all(|reward| reward["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "quest {quest_id}: every reward needs provenance"
        );
        assert!(
            rewards
                .iter()
                .all(|reward| reward["job_xp"]["11"].as_u64().is_some_and(|xp| xp > 0)),
            "quest {quest_id}: the reward must name Medic (profile 11) XP"
        );
        assert!(
            rewards.iter().all(|reward| reward["items"]
                .as_array()
                .is_some_and(|items| items.is_empty())),
            "quest {quest_id}: no invented item is granted"
        );
    }

    let exam = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(EXAM)))
        .unwrap();
    assert_eq!(exam["title_string_id"].as_u64(), Some(23_048));
    assert_eq!(exam["giver_npc_id"].as_u64(), Some(NAIA));
    assert_eq!(exam["prerequisite_quest_ids"], serde_json::json!([]));
    assert_eq!(exam["objectives"][0]["kind"], "talk_to_npc");
    assert_eq!(exam["objectives"][0]["npc_id"].as_u64(), Some(NAIA));
    assert_eq!(
        exam["rewards"][0]["coins"].as_u64(),
        Some(u64::from(EXAM_COINS))
    );
    assert_eq!(
        exam["rewards"][0]["job_xp"]["11"].as_u64(),
        Some(u64::from(EXAM_XP))
    );

    let house = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(HOUSE_CALL)))
        .unwrap();
    assert_eq!(house["title_string_id"].as_u64(), Some(22_989));
    assert_eq!(
        house["giver_npc_id"].as_u64(),
        Some(NAIA),
        "House Call's giver is Nurse Naia, not the unspawned 9000000419"
    );
    assert_eq!(house["prerequisite_quest_ids"], serde_json::json!([EXAM]));
    assert_eq!(house["objectives"][0]["npc_id"].as_u64(), Some(GREYLI));

    let vitamins = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(VITAMINS)))
        .unwrap();
    assert_eq!(vitamins["title_string_id"].as_u64(), Some(23_019));
    assert_eq!(vitamins["giver_npc_id"].as_u64(), Some(GORDON));
    assert_eq!(
        vitamins["prerequisite_quest_ids"],
        serde_json::json!([HOUSE_CALL])
    );
    assert_eq!(vitamins["objectives"][0]["npc_id"].as_u64(), Some(GREYLI));
    assert_eq!(
        vitamins["rewards"][0]["coins"].as_u64(),
        Some(u64::from(VITAMIN_COINS))
    );
    assert_eq!(
        vitamins["rewards"][0]["job_xp"]["11"].as_u64(),
        Some(u64::from(VITAMIN_XP))
    );

    // End-screen locale ids: the two completion lines are recovered client text.
    // The intro slots and the Take Your Vitamins end text stay 0; that is the
    // documented lead native-gap, asserted here and in the packet-level helpers.
    assert_eq!(
        exam["end_text_string_id"].as_u64(),
        Some(u64::from(EXAM_END_TEXT))
    );
    assert_eq!(
        house["end_text_string_id"].as_u64(),
        Some(u64::from(HOUSE_END_TEXT))
    );
    assert!(vitamins.get("end_text_string_id").is_none());
    for record in [exam, house, vitamins] {
        assert!(record.get("intro_text_string_id").is_none());
    }

    // T278: the pick-up trip is a server-side position check at Sunflower's row,
    // followed by the recovered hand-off to Nurse Naia. No gather/collect goal is
    // authored because harvesting is Chef-only and a Medic cannot perform it.
    let pills = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(PILLS)))
        .unwrap();
    assert_eq!(
        pills["title_string_id"].as_u64(),
        Some(u64::from(PILL_TITLE))
    );
    assert_eq!(pills["giver_npc_id"].as_u64(), Some(GORDON));
    assert_eq!(pills["job"].as_i64(), Some(i64::from(MEDIC)));
    assert!(
        pills.get("min_level").is_none(),
        "Palatable Pills has no level gate"
    );
    assert_eq!(
        pills["prerequisite_quest_ids"],
        serde_json::json!([HOUSE_CALL])
    );
    let pills_goals = pills["objectives"].as_array().unwrap();
    assert_eq!(pills_goals.len(), 2);
    assert_eq!(pills_goals[0]["kind"], "visit_position");
    assert_eq!(pills_goals[0]["zone_id"].as_u64(), Some(0));
    assert_eq!(pills_goals[0]["radius"].as_f64(), Some(8.0));
    assert_eq!(
        serde_json::from_value::<[f32; 3]>(pills_goals[0]["position"].clone()).unwrap(),
        content_position("sanctuary.json", SUNFLOWER),
        "the visit must use Sunflower's actual NPC row position, not a guessed centroid"
    );
    assert_eq!(pills_goals[1]["kind"], "talk_to_npc");
    assert_eq!(pills_goals[1]["npc_id"].as_u64(), Some(NAIA));
    assert!(
        pills_goals.iter().all(|goal| !matches!(
            goal["kind"].as_str(),
            Some("gather_resource" | "collect_item")
        )),
        "a Medic cannot harvest; the literal collection stays an explicit visit abstraction"
    );
    assert_eq!(
        pills["rewards"][0]["coins"].as_u64(),
        Some(u64::from(PILL_COINS))
    );
    assert_eq!(
        pills["rewards"][0]["job_xp"]["11"].as_u64(),
        Some(u64::from(PILL_XP))
    );
    assert_eq!(
        pills["intro_text_string_id"].as_u64(),
        Some(u64::from(PILL_INTRO_TEXT))
    );
    assert_eq!(
        pills["description_string_id"].as_u64(),
        Some(u64::from(PILL_DESCRIPTION))
    );
    assert_eq!(
        pills["end_text_string_id"].as_u64(),
        Some(u64::from(PILL_END_TEXT))
    );

    // T278: the level-five record is a real Medic rank-5 offer gate, not a client
    // claim; its only step is the recovered report back to Gordon.
    let level_five = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(LEVEL_FIVE)))
        .unwrap();
    assert_eq!(
        level_five["title_string_id"].as_u64(),
        Some(u64::from(LEVEL_TITLE))
    );
    assert_eq!(level_five["giver_npc_id"].as_u64(), Some(GORDON));
    assert_eq!(level_five["job"].as_i64(), Some(i64::from(MEDIC)));
    assert_eq!(level_five["min_level"].as_u64(), Some(5));
    assert_eq!(
        level_five["prerequisite_quest_ids"],
        serde_json::json!([PILLS, VITAMINS])
    );
    let level_goals = level_five["objectives"].as_array().unwrap();
    assert_eq!(level_goals.len(), 1);
    assert_eq!(level_goals[0]["kind"], "talk_to_npc");
    assert_eq!(level_goals[0]["npc_id"].as_u64(), Some(GORDON));
    assert_eq!(
        level_five["rewards"][0]["coins"].as_u64(),
        Some(u64::from(LEVEL_COINS))
    );
    assert_eq!(
        level_five["rewards"][0]["job_xp"]["11"].as_u64(),
        Some(u64::from(LEVEL_XP))
    );
    assert_eq!(
        level_five["intro_text_string_id"].as_u64(),
        Some(u64::from(LEVEL_INTRO_TEXT))
    );
    assert_eq!(
        level_five["description_string_id"].as_u64(),
        Some(u64::from(LEVEL_DESCRIPTION))
    );
    assert_eq!(
        level_five["end_text_string_id"].as_u64(),
        Some(u64::from(LEVEL_END_TEXT))
    );

    // The Bixie Hive, the repeatable contract 3090007 and every record from
    // Surgical Precision onward stay closed.
    for record in records {
        let id = record["id"].as_u64().unwrap();
        if id == u64::from(3_090_002_u32) || id >= 3_090_007 {
            assert_eq!(
                record["enabled"].as_bool(),
                Some(false),
                "quest {id} must stay disabled"
            );
        }
    }

    // Naia's trainer row teaches profile 11 through the ordinary menu.
    let trainers = read_json(&content_dir().join("job_trainers.json"));
    assert!(
        trainers["trainers"].as_array().unwrap().iter().any(|row| {
            row["npc"].as_u64() == Some(NAIA)
                && row["profile"].as_i64() == Some(i64::from(MEDIC))
                && row["label_string_id"].as_u64() == Some(u64::from(LEARN_LABEL))
        }),
        "Naia's trainer row drives the Learn menu option"
    );
}
