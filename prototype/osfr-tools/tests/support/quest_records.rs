//! Raw split quest records: only the fields `QuestEngine::from_split_content`
//! reads but does not expose (the `enabled` flag, prerequisite and exclusion ids,
//! the job and level gate, objective target fields and the authored title id). The loader
//! stays the authority for rewards, bounds and the enabled/offer rules.
//!
//! [`shipped_quests`]/[`find_quest`]/[`shipped_quest`] are the one shipped-record finder
//! every zone suite uses, and [`ZoneQuest::from_record`] is the one `RawQuest` ->
//! `ZoneQuest` card/payout mapping (it lives here because this module already depends
//! on `npc_quest_chain` for `read_json`; the reverse would not compile in the suites
//! that include `npc_quest_chain` alone).
//!
//! Test-only. Include it next to `npc_quest_chain` (for `read_json`):
//! `enabled_quest_reachability.rs` and `enabled_quest_menu_offers.rs` do.
#![allow(dead_code)]

use super::npc_quest_chain::{ZoneQuest, content_dir, read_json};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

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
    /// The raw reward rows [`RawQuest::reward_totals`] reads; the loader validates them.
    pub rewards: Vec<Value>,
    /// The 49/1 card text ids and the 49/13 end text (absent means 0, the
    /// loader's default), before the loader's `i32` cast.
    pub intro_text_string_id: u32,
    pub description_string_id: u32,
    pub end_text_string_id: u32,
    /// The record-level `source` notes' evidence text (provenance, `(R)` rules, stay-off reasons).
    pub source_notes: Vec<String>,
}

impl RawQuest {
    /// The (coins, `profile` XP) the reward rows pay this profile. The 49/13 star
    /// word is the sum of *every* profile's XP (`end_screen_reward`), so a test that
    /// compares that word with one profile's grant must pass the profile here and
    /// assert the one-profile shape; summing every profile would hide a wrong-profile
    /// grant.
    pub fn reward_totals(&self, profile: i32) -> (u32, u32) {
        let key = profile.to_string();
        let sum = |row: &Value| -> (u64, u64) {
            (
                row["coins"].as_u64().unwrap_or(0),
                row["job_xp"]
                    .as_object()
                    .and_then(|xp| xp.get(&key))
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            )
        };
        let (coins, xp) = self
            .rewards
            .iter()
            .map(sum)
            .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
        (coins as u32, xp as u32)
    }

    /// The level of the quest's `job_level` goal (the rank a level gate waits for).
    pub fn job_level_goal(&self) -> u32 {
        let goal = self
            .objectives
            .iter()
            .find(|goal| goal["kind"] == "job_level")
            .unwrap_or_else(|| panic!("quest {} has a rank goal", self.id));
        as_u32(&goal["level"]).expect("a rank goal has a level")
    }

    /// The 49/13 star word: every profile's job XP summed, the way production's
    /// `encode_quest_reward_bundle` builds the bundle. [`Self::reward_totals`] is one
    /// profile's share; a multi-profile record has a larger star word.
    pub fn reward_stars(&self) -> u32 {
        let stars = self
            .rewards
            .iter()
            .flat_map(|row| row["job_xp"].as_object().into_iter().flatten())
            .filter_map(|(_, amount)| amount.as_u64())
            .sum::<u64>();
        u32::try_from(stars).expect("the quest star total fits the 49/13 wire field")
    }
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
                rewards: record["rewards"].as_array().cloned().unwrap_or_default(),
                intro_text_string_id: as_u32(&record["intro_text_string_id"]).unwrap_or(0),
                description_string_id: as_u32(&record["description_string_id"]).unwrap_or(0),
                end_text_string_id: as_u32(&record["end_text_string_id"]).unwrap_or(0),
                source_notes: record["source"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|tag| tag["evidence"].as_str().map(str::to_owned))
                    .collect(),
            });
        }
    }
    quests
}

/// The shipped records, read once per test binary.
pub fn shipped_quests() -> &'static [RawQuest] {
    static RECORDS: OnceLock<Vec<RawQuest>> = OnceLock::new();
    RECORDS.get_or_init(|| read_quests(&content_dir()))
}

/// The record `id` in `quests`, for a suite that patched its own private catalogue.
pub fn find_quest(quests: &[RawQuest], id: u32) -> &RawQuest {
    quests
        .iter()
        .find(|quest| quest.id == id)
        .unwrap_or_else(|| panic!("the shipped quest record {id}"))
}

/// The shipped record `id`.
pub fn shipped_quest(id: u32) -> &'static RawQuest {
    find_quest(shipped_quests(), id)
}

impl ZoneQuest {
    /// The shared zone-quest record for one shipped quest. `giver` and `turnin` are
    /// the clicked NPC rows (a record carries the giver and objectives, not the
    /// engine's named end-screen actor); `profile` names the job-XP profile whose
    /// share `finish_quest` credits. The 49/13 star word (`xp`) is every profile's
    /// XP, so a record paying two profiles has `xp > profile_xp`.
    pub fn from_record(record: &RawQuest, giver: u64, turnin: u64, profile: i32) -> Self {
        let (coins, profile_xp) = record.reward_totals(profile);
        Self {
            id: record.id,
            giver,
            card: [
                u32::try_from(record.title_string_id).expect("quest title id fits u32"),
                record.intro_text_string_id,
                record.description_string_id,
            ],
            steps: &[],
            turnin,
            end_text: record.end_text_string_id,
            coins,
            xp: record.reward_stars(),
            profile_xp,
        }
    }
}
