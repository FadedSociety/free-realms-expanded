//! Work item 038 acceptance: Adventurer exploration/collection progression.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` — the original client binary. The four
//!   `ClientUpdatePacketCollection*` constructors pass 8, 9, 10 and 11 to `FUN_0089d180`, and the
//!   reward-bundle entry classes return type ids 10 / 11 with the names `CollectionAdd` and
//!   `CollectionEntryAdd`.
//! * the same decompile holds the UI action/event strings (`StartCollection`, `UpdateCollection`,
//!   `CompleteCollection`, `CollectionStarted`, `CollectionCompleted`), the category data source
//!   columns (`Collection Category ID/NAME/ICON_ID`) and the two counters (`CollectionsCompleted`,
//!   `CollectionCount`).
//! * `lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json` — the supplied
//!   emulator's initialization payload, accepted by the original client. It carries 253 per-profile
//!   collection rows with the collected flag on every entry.
//!
//! COMMUNITY evidence used by these vectors:
//! * `lab/references/freerealms-wiki/wiki.sqlite`, pages `Collections`, `Exploration (Collections)`,
//!   `List of Collections`, `Adventurer` — the exploration/rare-exploration/explorer taxonomy and the
//!   Adventurer unlock rule.
//!
//! RECONSTRUCTED here: the reconciliation rules (once-only entry, once-only completion, profile
//! isolation, caller-supplied proximity range). No distance, reward value, currency or item is
//! invented by this module.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_038;` under `src/restoration/`/`src/lib.rs`.
#[path = "../src/restoration/piece_038.rs"]
#[allow(dead_code)]
mod piece_038;

use piece_038::{
    ABOUT_ME_COLLECTIONS_COMPLETED, ACTION_COMPLETE_COLLECTION, ACTION_START_COLLECTION,
    ACTION_UPDATE_COLLECTION, AdventurerJob, CATEGORY_COLUMN_ICON_ID, CATEGORY_COLUMN_ID,
    CATEGORY_COLUMN_NAME, CLIENT_UPDATE_COLLECTION_ADD_ENTRY, CLIENT_UPDATE_COLLECTION_REMOVE,
    CLIENT_UPDATE_COLLECTION_REMOVE_ENTRY, CLIENT_UPDATE_COLLECTION_START, CollectionKind,
    CollectionUpdateOp, EVENT_COLLECTION_COMPLETED, EVENT_COLLECTION_STARTED,
    PROFILE_COLLECTION_COUNT, ProfileCollections, ProgressError, REWARD_BUNDLE_COLLECTION_ADD,
    REWARD_BUNDLE_COLLECTION_ENTRY_ADD, RewardBundleCollectionEntry, TutorialExit, UnlockOutcome,
    UpdateOutcome,
};

fn briarwood_water_samples() -> ProfileCollections {
    let mut profile = ProfileCollections::new(1);
    // Community `Collections.json` row 10 in `lab/references/Sanctuary` and captured emulator row 10
    // agree on collection 10 with entries 41..=48.
    profile.define(10, CollectionKind::Item, &[41, 42, 43, 44, 45, 46, 47, 48]);
    profile
}

#[test]
fn update_opcodes_are_the_recovered_client_constructors() {
    assert_eq!(CLIENT_UPDATE_COLLECTION_START, 8);
    assert_eq!(CLIENT_UPDATE_COLLECTION_REMOVE, 9);
    assert_eq!(CLIENT_UPDATE_COLLECTION_ADD_ENTRY, 10);
    assert_eq!(CLIENT_UPDATE_COLLECTION_REMOVE_ENTRY, 11);
    assert_eq!(CollectionUpdateOp::Start.opcode(), 8);
    assert_eq!(CollectionUpdateOp::Remove.opcode(), 9);
    assert_eq!(CollectionUpdateOp::AddEntry.opcode(), 10);
    assert_eq!(CollectionUpdateOp::RemoveEntry.opcode(), 11);
    assert_eq!(
        CollectionUpdateOp::from_opcode(10),
        Some(CollectionUpdateOp::AddEntry)
    );
    // 12 is the next constructor in the same run and is a different packet.
    assert_eq!(CollectionUpdateOp::from_opcode(12), None);
    assert_eq!(CollectionUpdateOp::from_opcode(0), None);
}

#[test]
fn reward_bundle_types_are_the_recovered_collection_entries() {
    assert_eq!(REWARD_BUNDLE_COLLECTION_ADD, 10);
    assert_eq!(REWARD_BUNDLE_COLLECTION_ENTRY_ADD, 11);
    let completed = RewardBundleCollectionEntry::Completed;
    assert_eq!(completed.type_id(), 10);
    assert_eq!(completed.name(), "CollectionAdd");
    assert_eq!(completed.parameter_count(), 1);
    let entry = RewardBundleCollectionEntry::EntryAdded;
    assert_eq!(entry.type_id(), 11);
    assert_eq!(entry.name(), "CollectionEntryAdd");
    assert_eq!(entry.parameter_count(), 4);
}

#[test]
fn ui_action_and_event_names_are_the_recovered_strings() {
    assert_eq!(ACTION_START_COLLECTION, "StartCollection");
    assert_eq!(ACTION_UPDATE_COLLECTION, "UpdateCollection");
    assert_eq!(ACTION_COMPLETE_COLLECTION, "CompleteCollection");
    assert_eq!(EVENT_COLLECTION_STARTED, "CollectionStarted");
    assert_eq!(EVENT_COLLECTION_COMPLETED, "CollectionCompleted");
}

#[test]
fn category_columns_and_profile_counters_are_the_recovered_names() {
    assert_eq!(CATEGORY_COLUMN_ID, "Collection Category ID");
    assert_eq!(CATEGORY_COLUMN_NAME, "Collection Category NAME");
    assert_eq!(CATEGORY_COLUMN_ICON_ID, "Collection Category ICON_ID");
    assert_eq!(ABOUT_ME_COLLECTIONS_COMPLETED, "CollectionsCompleted");
    assert_eq!(PROFILE_COLLECTION_COUNT, "CollectionCount");
}

#[test]
fn a_collection_completes_once_when_its_last_entry_arrives() {
    let mut profile = briarwood_water_samples();
    for entry in 41..=47 {
        assert_eq!(
            profile.apply(1, CollectionUpdateOp::AddEntry, 10, entry),
            Ok(UpdateOutcome::EntryAdded)
        );
        assert!(!profile.is_complete(10));
    }
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::AddEntry, 10, 48),
        Ok(UpdateOutcome::Completed)
    );
    assert!(profile.is_complete(10));
    assert_eq!(profile.completed_count(), 1);
}

#[test]
fn a_replayed_add_entry_is_ignored_and_never_re_completes() {
    let mut profile = briarwood_water_samples();
    for entry in 41..=48 {
        profile
            .apply(1, CollectionUpdateOp::AddEntry, 10, entry)
            .unwrap();
    }
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::AddEntry, 10, 48),
        Ok(UpdateOutcome::DuplicateIgnored)
    );
    assert_eq!(profile.collected_entry_count(10), 8);
    assert_eq!(profile.completed_count(), 1);
}

#[test]
fn an_update_addressed_to_another_profile_is_refused_and_changes_nothing() {
    let mut profile = briarwood_water_samples();
    assert_eq!(
        profile.apply(2, CollectionUpdateOp::AddEntry, 10, 41),
        Err(ProgressError::ForeignProfile)
    );
    assert_eq!(profile.collected_entry_count(10), 0);
    assert_eq!(profile.completed_count(), 0);
}

#[test]
fn unknown_collection_or_entry_is_refused() {
    let mut profile = briarwood_water_samples();
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::AddEntry, 999, 41),
        Err(ProgressError::UnknownCollection)
    );
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::AddEntry, 10, 99),
        Err(ProgressError::UnknownEntry)
    );
}

#[test]
fn remove_entry_and_remove_collection_clear_progress_without_losing_the_definition() {
    let mut profile = briarwood_water_samples();
    for entry in 41..=48 {
        profile
            .apply(1, CollectionUpdateOp::AddEntry, 10, entry)
            .unwrap();
    }
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::RemoveEntry, 10, 48),
        Ok(UpdateOutcome::EntryRemoved)
    );
    assert!(!profile.is_complete(10));
    assert_eq!(profile.completed_count(), 0);
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::RemoveEntry, 10, 48),
        Ok(UpdateOutcome::DuplicateIgnored)
    );
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::Remove, 10, 0),
        Ok(UpdateOutcome::CollectionRemoved)
    );
    assert_eq!(profile.collected_entry_count(10), 0);
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::Start, 10, 0),
        Ok(UpdateOutcome::Started)
    );
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::Start, 10, 0),
        Ok(UpdateOutcome::DuplicateIgnored)
    );
    // The definition survives, so the collection can be re-earned.
    assert_eq!(
        profile.apply(1, CollectionUpdateOp::AddEntry, 10, 41),
        Ok(UpdateOutcome::EntryAdded)
    );
}

#[test]
fn exploration_entry_unlocks_inside_the_caller_supplied_range() {
    let mut profile = ProfileCollections::new(1);
    profile.define(20, CollectionKind::Exploration, &[1, 2]);
    assert_eq!(
        profile.approach(20, 1, 12.0, 12.0),
        Ok(UnlockOutcome::Unlocked)
    );
    assert_eq!(
        profile.approach(20, 2, 12.01, 12.0),
        Ok(UnlockOutcome::OutOfRange)
    );
    assert_eq!(profile.collected_entry_count(20), 1);
}

#[test]
fn proximity_never_unlocks_a_rare_coin_entry() {
    let mut profile = ProfileCollections::new(1);
    profile.define(21, CollectionKind::RareExploration, &[1, 2]);
    assert_eq!(
        profile.approach(21, 1, 0.0, 12.0),
        Ok(UnlockOutcome::RequiresInteraction)
    );
    assert_eq!(profile.collect(21, 1), Ok(UnlockOutcome::Unlocked));
    assert_eq!(profile.collect(21, 1), Ok(UnlockOutcome::AlreadyCollected));
    assert_eq!(profile.collected_entry_count(21), 1);
}

#[test]
fn invalid_range_or_distance_is_rejected() {
    let mut profile = ProfileCollections::new(1);
    profile.define(20, CollectionKind::Exploration, &[1]);
    assert_eq!(
        profile.approach(20, 1, 1.0, -1.0),
        Err(ProgressError::InvalidRange)
    );
    assert_eq!(
        profile.approach(20, 1, f32::NAN, 12.0),
        Err(ProgressError::InvalidRange)
    );
    assert_eq!(
        profile.approach(20, 1, f32::INFINITY, 12.0),
        Err(ProgressError::InvalidRange)
    );
    assert_eq!(profile.collected_entry_count(20), 0);
}

#[test]
fn explorer_collection_completes_only_after_every_child_is_complete() {
    let mut profile = ProfileCollections::new(1);
    profile.define(30, CollectionKind::Exploration, &[1]);
    profile.define(31, CollectionKind::Exploration, &[1]);
    profile.define(32, CollectionKind::Explorer, &[30, 31]);
    // An explorer entry is its child collection, so it can never be collected directly.
    assert_eq!(
        profile.approach(32, 30, 0.0, 5.0),
        Ok(UnlockOutcome::CompletedByChildren)
    );
    profile.approach(30, 1, 0.0, 5.0).unwrap();
    assert_eq!(profile.resolve_explorer_unlocks(), Vec::<i32>::new());
    profile.approach(31, 1, 0.0, 5.0).unwrap();
    assert_eq!(profile.resolve_explorer_unlocks(), vec![32]);
    assert_eq!(profile.resolve_explorer_unlocks(), Vec::<i32>::new());
    assert_eq!(profile.completed_count(), 3);
}

#[test]
fn adventurer_unlock_is_granted_at_most_once() {
    let mut job = AdventurerJob::locked();
    assert!(!job.is_unlocked());
    assert!(job.apply(TutorialExit::FirstQuestCompleted));
    assert!(job.is_unlocked());
    assert!(!job.apply(TutorialExit::FirstQuestCompleted));
    assert!(!job.apply(TutorialExit::Skipped));

    let mut skipped = AdventurerJob::locked();
    assert!(skipped.apply(TutorialExit::Skipped));
    assert!(skipped.is_unlocked());
}

#[test]
fn replayed_updates_reproduce_the_same_progress_in_a_stable_order() {
    let mut profile = briarwood_water_samples();
    // A reconnect can replay the same update packets; the ledger must not drift or double count.
    for _ in 0..2 {
        for entry in [41, 43, 45] {
            profile
                .apply(1, CollectionUpdateOp::AddEntry, 10, entry)
                .unwrap();
        }
    }
    let snapshot = profile.snapshot();
    assert_eq!(
        snapshot,
        vec![
            (10, 41, true),
            (10, 42, false),
            (10, 43, true),
            (10, 44, false),
            (10, 45, true),
            (10, 46, false),
            (10, 47, false),
            (10, 48, false),
        ]
    );
    assert_eq!(profile.collected_entry_count(10), 3);
    assert_eq!(profile.completed_count(), 0);

    // The rest of the captured row then completes it exactly once.
    for entry in 41..=48 {
        profile
            .apply(1, CollectionUpdateOp::AddEntry, 10, entry)
            .unwrap();
    }
    assert!(profile.is_complete(10));
    assert_eq!(profile.completed_count(), 1);
}
