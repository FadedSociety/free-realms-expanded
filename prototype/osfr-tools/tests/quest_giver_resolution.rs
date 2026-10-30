//! T898: every quest giver that is not an NPC row is an `npcs_unresolved.json` slot that says
//! why it is still open, and every slot the research resolved points at a real NPC row.
//!
//! `resolution.npc_id` on a slot is the census row the giver was named as; the quest records
//! carry that row id, so a resolved slot is never a quest's giver any more. A slot a quest
//! still points at has `resolution.npc_id: null` and the evidence tried in `basis`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{content_dir, read_json};
use quest_records::{shipped_quests, sorted_json_paths};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The `resolution` of every unresolved slot that has one, by slot id.
fn slot_resolutions() -> BTreeMap<u64, Value> {
    let slots = read_json(&content_dir().join("npcs_unresolved.json"));
    slots["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|slot| !slot["resolution"].is_null())
        .map(|slot| (slot["ref_id"].as_u64().unwrap(), slot["resolution"].clone()))
        .collect()
}

fn npc_row_ids() -> BTreeSet<u64> {
    sorted_json_paths(&content_dir().join("npcs"))
        .iter()
        .flat_map(|path| {
            read_json(path)["records"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["npc_id"].as_u64().unwrap())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn a_resolved_slot_names_a_real_npc_row_and_no_quest_still_points_at_the_slot() {
    let rows = npc_row_ids();
    let resolutions = slot_resolutions();
    for (slot, resolution) in &resolutions {
        let Some(row) = resolution["npc_id"].as_u64() else {
            continue;
        };
        assert!(
            rows.contains(&row),
            "slot {slot} resolves to missing row {row}"
        );
        let basis = resolution["basis"].as_str().unwrap_or_default();
        assert!(basis.len() > 20, "slot {slot} resolves without a basis");
        let stale: Vec<u32> = shipped_quests()
            .iter()
            .filter(|quest| quest.giver_npc_id == *slot)
            .map(|quest| quest.id)
            .collect();
        assert!(
            stale.is_empty(),
            "slot {slot} resolves to {row} but quests {stale:?} still name the slot"
        );
    }
}

#[test]
fn a_giver_left_on_an_unresolved_slot_carries_the_evidence_tried() {
    let rows = npc_row_ids();
    let resolutions = slot_resolutions();
    let slots: BTreeSet<u64> = read_json(&content_dir().join("npcs_unresolved.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|slot| slot["ref_id"].as_u64().unwrap())
        .collect();
    for quest in shipped_quests() {
        let giver = quest.giver_npc_id;
        if rows.contains(&giver) || !slots.contains(&giver) {
            continue;
        }
        let resolution = resolutions
            .get(&giver)
            .unwrap_or_else(|| panic!("quest {} names slot {giver} with no resolution", quest.id));
        assert!(
            resolution["npc_id"].is_null(),
            "quest {} names slot {giver}, which resolves to a row",
            quest.id
        );
        let basis = resolution["basis"].as_str().unwrap_or_default();
        assert!(
            basis.len() > 20,
            "slot {giver} (quest {}) states no evidence tried",
            quest.id
        );
    }
}
