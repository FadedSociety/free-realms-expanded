//! T357: the enabled split quest catalogue and the giver NPC rows agree.
//!
//! The shipped content is loaded through the production quest loader
//! (`QuestEngine::from_split_content`): it owns the enabled default (a record
//! without `enabled` is enabled), the giver match (a row id whose quest names a
//! different `giver_npc_id` is a load error) and the enabled set. The NPC rows
//! are the shipped `data/content/npcs/*.json` rows that `NpcInteraction::load`
//! parses; only the row's explicit `quest_ids_given`/`quests_given` lists are
//! read here because no production accessor exposes them.
//!
//! The Talk path (`NpcInteraction::offerable_quest_for_talk`) consults the row
//! list first and then the loader's giver union (`QuestEngine::quest_ids_for_giver`),
//! so an unlisted quest is still offerable today. That union is why three ids are
//! deliberately left unbound: `tests/scavenger_hunt_quest.rs` pins Jacque's and
//! Ricky Danger's rows empty to prove the union and the hunt host's clean first
//! Talk. They are named in `UNBOUND_UNION_CONTROLS`; any other enabled quest
//! without a row binding fails this gate.
//!
//! Counted on the T377 tree: 655 split quest records, 48 enabled, 45 of them
//! bound by a giver row, 3 union controls. The gate reads the counts, so a
//! content change reports its new numbers instead of silently re-pinning them.
//!
//! Every listed id must resolve to an authored quest and must name that quest's
//! giver; a repeated entry inside one row is refused even though the loader's
//! set union tolerates it. The disabled-listing side of the brief's negative
//! check is reported, not failed: 372 pre-existing row listings name disabled
//! quests, a separate content cleanup outside this cross-reference fix.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{content_dir, read_json};
use osfr_tools::{
    npc_interaction::NpcInteraction,
    quest_engine::{CharacterEligibility, QuestEngine},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const CHARACTER: u64 = 7;

/// Enabled quests whose giver rows stay deliberately empty, so the offer union
/// (not a row) is what makes them reachable. `tests/scavenger_hunt_quest.rs`
/// pins these rows empty: `3000200` is the Scavenger Hunt host Jacque
/// (9500000497, a clean first Talk) and `3000059`/`3000060` are Ricky Danger's
/// original Cobblestone chain (9500000254). A new unbound quest is a failure.
const UNBOUND_UNION_CONTROLS: [u32; 3] = [3_000_059, 3_000_060, 3_000_200];

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 20,
        member_quest_limit: 20,
    }
}

/// One authored NPC row's explicit quest lists.
struct Row {
    file: String,
    npc_id: u64,
    quest_ids_given: Vec<u32>,
    quests_given: Vec<u32>,
}

impl Row {
    /// How many times this row lists `quest_id` in `quest_ids_given` and in
    /// `quests_given`. The two fields of one row are one binding (the loader
    /// takes their union), but a repeated entry in one field is a duplicate.
    fn listings(&self, quest_id: u32) -> (usize, usize) {
        (
            self.quest_ids_given
                .iter()
                .filter(|id| **id == quest_id)
                .count(),
            self.quests_given
                .iter()
                .filter(|id| **id == quest_id)
                .count(),
        )
    }

    fn binds(&self, quest_id: u32) -> bool {
        let (ids, objects) = self.listings(quest_id);
        ids + objects > 0
    }
}

fn rows(root: &Path) -> Vec<Row> {
    let mut paths: Vec<_> = fs::read_dir(root.join("npcs"))
        .unwrap_or_else(|error| panic!("{}: {error}", root.display()))
        .map(|entry| entry.expect("NPC directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    paths.sort();
    let mut rows = Vec::new();
    for path in paths {
        let document = read_json(&path);
        let file = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();
        let records = document["records"]
            .as_array()
            .unwrap_or_else(|| panic!("{file}: no records array"));
        for record in records {
            let npc_id = record["npc_id"]
                .as_u64()
                .unwrap_or_else(|| panic!("{file}: NPC row has no id"));
            let quest_ids_given = record["quest_ids_given"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|value| {
                    value
                        .as_u64()
                        .and_then(|id| u32::try_from(id).ok())
                        .unwrap_or_else(|| {
                            panic!("{file}: NPC {npc_id} has a non-numeric quest id")
                        })
                })
                .collect();
            let quests_given = record["quests_given"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    entry["quest_id"]
                        .as_u64()
                        .and_then(|id| u32::try_from(id).ok())
                })
                .collect();
            rows.push(Row {
                file: file.clone(),
                npc_id,
                quest_ids_given,
                quests_given,
            });
        }
    }
    rows
}

/// The split quest catalogue as the production loader sees it.
struct QuestRows {
    engine: QuestEngine,
    /// Every authored record: id -> (enabled flag with the absent default true,
    /// authored giver).
    records: BTreeMap<u32, (bool, u64)>,
}

impl QuestRows {
    fn load(root: &Path) -> Self {
        let engine = QuestEngine::from_split_content(root, CHARACTER, eligibility(), None)
            .expect("the shipped split catalogue loads");
        let mut paths: Vec<_> = fs::read_dir(root.join("quests"))
            .unwrap_or_else(|error| panic!("{}: {error}", root.display()))
            .map(|entry| entry.expect("quest directory entry").path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect();
        paths.sort();
        let mut records = BTreeMap::new();
        for path in paths {
            let document = read_json(&path);
            let records_array = document["records"]
                .as_array()
                .unwrap_or_else(|| panic!("{}: no records array", path.display()));
            for record in records_array {
                let id = record["id"]
                    .as_u64()
                    .and_then(|id| u32::try_from(id).ok())
                    .unwrap_or_else(|| panic!("{}: invalid quest id", path.display()));
                // The loader's own rule: an absent `enabled` is enabled.
                let enabled = record.get("enabled").is_none_or(|value| {
                    value.as_bool().unwrap_or_else(|| {
                        panic!("{}: quest {id} enabled is not boolean", path.display())
                    })
                });
                let giver = record["giver_npc_id"]
                    .as_u64()
                    .unwrap_or_else(|| panic!("{}: quest {id} has no giver", path.display()));
                assert!(
                    records.insert(id, (enabled, giver)).is_none(),
                    "{}: duplicate quest id {id}",
                    path.display()
                );
            }
        }
        Self { engine, records }
    }

    fn enabled(&self) -> impl Iterator<Item = (u32, u64)> + '_ {
        self.records
            .iter()
            .filter(|(_, (enabled, _))| *enabled)
            .map(|(id, (_, giver))| (*id, *giver))
    }
}

#[test]
fn every_enabled_quest_is_bound_to_its_giver_row_or_a_named_union_control() {
    let root = content_dir();
    let quests = QuestRows::load(&root);
    let rows = rows(&root);
    // The production NPC loader must accept the rows this gate reads; a minimal
    // `quests_given` binding is valid because every `NpcQuest` field is optional.
    NpcInteraction::load(&root.join("npcs"), &root.join("stores.json"))
        .expect("the shipped NPC content loads");

    // The loader is the authority for both halves of a binding: every authored
    // record keeps its giver, and every raw-enabled record is enabled in the
    // loader (not disabled, and offering a non-empty objective list).
    for (id, (enabled, giver)) in &quests.records {
        assert_eq!(
            quests.engine.giver_npc_id(*id),
            Some(*giver),
            "quest {id}: the loader's giver disagrees with the authored record"
        );
        assert_eq!(
            quests.engine.is_enabled(*id),
            *enabled,
            "quest {id}: the loader's enabled view disagrees with the authored flag"
        );
    }

    // A quest id may appear in at most one NPC row and in each of that row's two
    // lists at most once. The loader's set union tolerates a duplicate; the
    // catalogue does not, because the row is what a Talk menu reads.
    let mut duplicate_bindings = Vec::new();
    let mut wrong_givers = Vec::new();
    let mut missing_definitions = Vec::new();
    for row in &rows {
        let listed: BTreeSet<u32> = row
            .quest_ids_given
            .iter()
            .copied()
            .chain(row.quests_given.iter().copied())
            .collect();
        for quest_id in listed {
            let Some((_, giver)) = quests.records.get(&quest_id) else {
                missing_definitions.push((quest_id, row.npc_id, row.file.clone()));
                continue;
            };
            if *giver != row.npc_id {
                wrong_givers.push((quest_id, row.npc_id, row.file.clone()));
            }
            let (ids, objects) = row.listings(quest_id);
            if ids > 1 || objects > 1 {
                duplicate_bindings.push((quest_id, row.npc_id, row.file.clone()));
            }
        }
    }
    // A missing definition is a loader error, repeated here so a row typo is
    // named with its file; the wrong-giver and duplicate checks are this gate's
    // own contract.
    assert!(
        missing_definitions.is_empty(),
        "NPC rows list quest ids with no authored record: {missing_definitions:?}"
    );
    assert!(
        wrong_givers.is_empty(),
        "NPC rows bind a quest that names a different giver: {wrong_givers:?}"
    );
    assert!(
        duplicate_bindings.is_empty(),
        "NPC rows list a quest more than once in a field: {duplicate_bindings:?}"
    );

    // Bindings by row; a quest in both lists of the same row is one binding.
    let controls: BTreeSet<u32> = UNBOUND_UNION_CONTROLS.iter().copied().collect();
    let mut unbound = Vec::new();
    let mut bound = BTreeSet::new();
    for (quest_id, giver) in quests.enabled() {
        if controls.contains(&quest_id) {
            assert!(
                !rows.iter().any(|row| row.binds(quest_id)),
                "union control {quest_id} must stay unbound (tests/scavenger_hunt_quest.rs pins the row)"
            );
        }
        let binders: Vec<&Row> = rows.iter().filter(|row| row.binds(quest_id)).collect();
        match binders.as_slice() {
            [] => unbound.push(quest_id),
            [row] if row.npc_id == giver => {
                bound.insert(quest_id);
            }
            _ => {
                let named: Vec<String> = binders
                    .iter()
                    .map(|row| format!("{quest_id}->{} ({})", row.npc_id, row.file))
                    .collect();
                panic!("quest {quest_id}: expected exactly one giver row {giver}, found {named:?}")
            }
        }
    }
    // Sorted: `unbound` follows the quest-id map and the controls are named.
    let unexpected: Vec<u32> = unbound
        .iter()
        .copied()
        .filter(|id| !controls.contains(id))
        .collect();
    assert!(
        unexpected.is_empty(),
        "enabled quests with no giver NPC row binding (bind the row, or add a documented \
         UNBOUND_UNION_CONTROLS entry): {unexpected:?}"
    );
    for control in &controls {
        assert!(
            quests
                .records
                .get(control)
                .is_some_and(|(enabled, _)| *enabled),
            "union control {control} must stay an enabled quest record"
        );
    }

    // Pre-existing gap, reported rather than failed: rows may list a disabled
    // quest (the loader accepts it). T357 does not unlist those; a queued
    // content cleanup owns them.
    let disabled_listed: BTreeSet<u32> = rows
        .iter()
        .flat_map(|row| {
            row.quest_ids_given
                .iter()
                .copied()
                .chain(row.quests_given.iter().copied())
        })
        .filter(|id| quests.records.get(id).is_some_and(|(enabled, _)| !enabled))
        .collect();
    println!(
        "split quest records: {} ({} enabled); enabled quests bound to their giver row: {}; \
         union controls: {}; disabled quest ids still listed by NPC rows: {}",
        quests.records.len(),
        quests.enabled().count(),
        bound.len(),
        controls.len(),
        disabled_listed.len(),
    );
}
