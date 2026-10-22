//! Raw split quest records: only the fields `QuestEngine::from_split_content`
//! reads but does not expose (the `enabled` flag, prerequisite and exclusion ids,
//! the job and level gate, objective target fields and the authored title id). The loader
//! stays the authority for rewards, bounds and the enabled/offer rules.
//!
//! Test-only. Include it next to `npc_quest_chain` (for `read_json`):
//! `enabled_quest_reachability.rs` and `enabled_quest_menu_offers.rs` do.
#![allow(dead_code)]

use super::npc_quest_chain::read_json;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|value| u32::try_from(value).ok())
}

/// One split quest record.
#[derive(Debug)]
pub struct RawQuest {
    pub id: u32,
    pub enabled: bool,
    pub giver_npc_id: u64,
    /// Authored title id before `definition_from_content` casts it to `i32`.
    pub title_string_id: u64,
    pub prerequisite_quest_ids: Vec<u32>,
    /// T465 mutually exclusive quests; the loader already refused bad ids.
    pub excludes_quest_ids: Vec<u32>,
    /// The `job` profile gate; `definition_from_content` applies `min_level` to
    /// that job's rank, or to the player level when there is no job.
    pub job: Option<i32>,
    pub min_level: Option<u32>,
    pub objectives: Vec<Value>,
}

pub fn sorted_json_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
                .path()
        })
        .collect();
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    paths
}

pub fn read_quests(root: &Path) -> Vec<RawQuest> {
    let mut quests = Vec::new();
    for path in sorted_json_paths(&root.join("quests")) {
        let document = read_json(&path);
        let records = document["records"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: no records array", path.display()));
        for record in records {
            let id = as_u32(&record["id"])
                .unwrap_or_else(|| panic!("{}: invalid quest id", path.display()));
            let enabled = match record.get("enabled") {
                None => true,
                Some(value) => value.as_bool().unwrap_or_else(|| {
                    panic!("{}: quest {id} enabled is not boolean", path.display())
                }),
            };
            let giver_npc_id = record["giver_npc_id"]
                .as_u64()
                .unwrap_or_else(|| panic!("{}: quest {id} has no giver", path.display()));
            let prerequisite_quest_ids = record["prerequisite_quest_ids"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|value| {
                    as_u32(value).unwrap_or_else(|| {
                        panic!("{}: quest {id} prerequisite id is invalid", path.display())
                    })
                })
                .collect();
            let objectives = record["objectives"]
                .as_array()
                .cloned()
                .unwrap_or_else(|| panic!("{}: quest {id} has no objectives", path.display()));
            quests.push(RawQuest {
                id,
                enabled,
                giver_npc_id,
                title_string_id: record["title_string_id"].as_u64().unwrap_or(0),
                prerequisite_quest_ids,
                excludes_quest_ids: record["excludes_quest_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(as_u32)
                    .collect(),
                job: record["job"]
                    .as_i64()
                    .and_then(|job| i32::try_from(job).ok()),
                min_level: as_u32(&record["min_level"]),
                objectives,
            });
        }
    }
    quests
}
