//! Piece 074 (collections and collectibles) tests: the recovered category table,
//! per-entry progress, exactly-once completion and restart persistence.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client inputs, hashes and CRCs pinned in the lab report):
//! * `OSFR Client/Resources/CollectionCategories.txt` — 10 rows, header
//!   `#ID^NAME_ID^ICON_ID^`; row 1 is `1^93636^9535^`, row 10 is
//!   `10^392298^1319^`. The resolved names are Exploration, Job Specific,
//!   Flora, Toys, Insects, Robgoblin Junk, Archaic, Aquatic, Quests, Minigames.
//! * `decompiled/client-ui/ScriptsBase.lua` `CollectionData_DS` — the client's
//!   collection data source is `BaseClient.CollectionData` with `EntryCount` at
//!   column 6, `CollectionCategoryID` at 8 and exactly eight entry flags
//!   `EntryCollected1..8` at columns 9..16, so a collection row carries 17
//!   columns. `CollectionCategory_DS` is `BaseClient.CollectionCategory` with
//!   `CollectionCategoryID` 0, `CollectionCategoryNAME` 1, `CollectionCategoryICON_ID` 2.
//! * `decompiled/client/FreeRealms/pseudocode.c` — `FUN_009fce20` returns 3 for
//!   the category row width and `FUN_009fce30` names those three columns
//!   ("Collection Category ID", "Collection Category NAME",
//!   "Collection Category ICON_ID"); `FUN_009fd050` buckets a category with
//!   `param_2 % 10`; the client builds `ClientUpdatePacketCollectionStart`,
//!   `ClientUpdatePacketCollectionAddEntry`, `ClientUpdatePacketCollectionRemoveEntry`
//!   and `ClientUpdatePacketCollectionRemove`, and notifies
//!   `CollectionStarted` / `UpdateCollection` / `CollectionCompleted`.
//!
//! COMMUNITY (catalogue shape only, never a first-party claim): the wiki corpus
//! lists eight entries for a common Toys collection such as Army Men, and the
//! pinned community server decodes a collection row with an entry list carrying
//! `Index`, `Collected` and a reward bundle.
//!
//! RECONSTRUCTED here: the catalogue/ledger split, every entry identifier used
//! in these vectors (they are synthetic — no first-party entry ids were
//! recovered), the durable document and every error. Node placement, radii,
//! reward item ids and reward amounts were not recovered and are not invented.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_074.rs"]
#[allow(dead_code)]
mod piece_074;

use piece_074::{
    COLLECTION_CATEGORIES, COLLECTION_CATEGORY_COUNT, CollectOutcome, CollectionCatalogue,
    CollectionDefinition, CollectionError, CollectionLedger,
};

const COLLECTION_CATEGORY_COLUMNS: usize = 3;
const COLLECTION_ROW_COLUMNS: usize = 17;
const COLLECTION_VIEW_ENTRY_FLAGS: usize = 8;
const COLLECTION_CATEGORY_HASH_BUCKETS: u32 = 10;

fn category_bucket(category_id: i32) -> Option<u32> {
    if category_id <= 0 {
        return None;
    }
    Some((category_id as u32) % COLLECTION_CATEGORY_HASH_BUCKETS)
}

/// COMMUNITY: the Toys category row, `CollectionCategories.txt` id 4.
const CATEGORY_TOYS: i32 = 4;
/// COMMUNITY: "Army Men" is a common Toys collection whose wiki page lists
/// exactly eight entries; the identifiers below are synthetic.
const ARMY_MEN: i32 = 20_074;
const ARMY_MEN_ENTRIES: [i32; 8] = [
    74_001, 74_002, 74_003, 74_004, 74_005, 74_006, 74_007, 74_008,
];
/// COMMUNITY: "Tops" is a second common Toys collection, also eight entries.
const TOPS: i32 = 20_075;
const TOPS_ENTRIES: [i32; 8] = [
    74_101, 74_102, 74_103, 74_104, 74_105, 74_106, 74_107, 74_108,
];

fn toys_catalogue() -> CollectionCatalogue {
    let mut catalogue = CollectionCatalogue::new();
    catalogue
        .insert(CollectionDefinition {
            id: ARMY_MEN,
            category_id: CATEGORY_TOYS,
            entries: ARMY_MEN_ENTRIES.to_vec(),
        })
        .expect("Army Men definition is well formed");
    catalogue
        .insert(CollectionDefinition {
            id: TOPS,
            category_id: CATEGORY_TOYS,
            entries: TOPS_ENTRIES.to_vec(),
        })
        .expect("Tops definition is well formed");
    catalogue
}

fn collect_all(
    ledger: &mut CollectionLedger,
    catalogue: &CollectionCatalogue,
    entries: &[i32],
) -> Vec<CollectOutcome> {
    entries
        .iter()
        .map(|entry| {
            ledger
                .collect(catalogue, *entry)
                .expect("a catalogued entry is collectable")
        })
        .collect()
}

#[test]
fn category_table_is_the_ten_recovered_client_rows() {
    assert_eq!(COLLECTION_CATEGORY_COUNT, 10);
    assert_eq!(COLLECTION_CATEGORY_COUNT, COLLECTION_CATEGORIES.len());
    for (index, category) in COLLECTION_CATEGORIES.iter().enumerate() {
        assert_eq!(
            category.id,
            index as i32 + 1,
            "the client rows are numbered 1..10 in file order"
        );
    }
    assert_eq!(
        COLLECTION_CATEGORIES[0],
        piece_074::CollectionCategory {
            id: 1,
            name_id: 93636,
            icon_id: 9535,
        }
    );
    assert_eq!(
        COLLECTION_CATEGORIES[9],
        piece_074::CollectionCategory {
            id: 10,
            name_id: 392_298,
            icon_id: 1319,
        }
    );
    // The category row is three columns wide; the collection row is the
    // recovered `BaseClient.CollectionData` width.
    assert_eq!(COLLECTION_CATEGORY_COLUMNS, 3);
    assert_eq!(COLLECTION_ROW_COLUMNS, 17);
    assert_eq!(COLLECTION_VIEW_ENTRY_FLAGS, 8);
    assert_eq!(COLLECTION_CATEGORY_HASH_BUCKETS, 10);
}

#[test]
fn category_bucket_is_the_recovered_modulo() {
    assert_eq!(category_bucket(1), Some(1));
    assert_eq!(category_bucket(10), Some(0));
    assert_eq!(
        category_bucket(11),
        Some(1),
        "FUN_009fd050 hashes with % 10"
    );
    assert_eq!(
        category_bucket(0),
        None,
        "category 0 is not one of the ten recovered rows"
    );
}

#[test]
fn collecting_every_entry_completes_the_collection_once() {
    let catalogue = toys_catalogue();
    let mut ledger = CollectionLedger::new();

    let outcomes = collect_all(&mut ledger, &catalogue, &ARMY_MEN_ENTRIES);
    for outcome in &outcomes[..ARMY_MEN_ENTRIES.len() - 1] {
        assert!(outcome.newly_collected);
        assert_eq!(outcome.collection_id, ARMY_MEN);
        assert!(
            outcome.completion.is_none(),
            "an incomplete collection must not report a completion"
        );
    }

    let last = outcomes.last().expect("the run collected something");
    assert!(last.newly_collected);
    let completion = last
        .completion
        .as_ref()
        .expect("the eighth entry completes Army Men");
    assert_eq!(completion.collection_id, ARMY_MEN);
    assert_eq!(completion.entry_count, COLLECTION_VIEW_ENTRY_FLAGS);

    let progress = ledger
        .progress(&catalogue, ARMY_MEN)
        .expect("Army Men is catalogued");
    assert_eq!(progress.collected, COLLECTION_VIEW_ENTRY_FLAGS);
    assert_eq!(progress.total, COLLECTION_VIEW_ENTRY_FLAGS);
    assert!(progress.complete);
    assert!(ledger.is_completed(ARMY_MEN));
    assert!(
        !ledger.is_completed(TOPS),
        "completing one collection must not complete its neighbour"
    );
}

#[test]
fn replaying_a_collected_entry_does_not_re_emit_a_completion() {
    let catalogue = toys_catalogue();
    let mut ledger = CollectionLedger::new();
    collect_all(&mut ledger, &catalogue, &ARMY_MEN_ENTRIES);
    assert!(ledger.is_completed(ARMY_MEN));

    let replay = ledger
        .collect(&catalogue, ARMY_MEN_ENTRIES[0])
        .expect("a catalogued entry stays collectable");
    assert!(
        !replay.newly_collected,
        "the entry was already in the ledger"
    );
    assert!(
        replay.completion.is_none(),
        "a replay must not re-emit the completion, or the reward is granted twice"
    );
    assert_eq!(ledger.collected_count(), COLLECTION_VIEW_ENTRY_FLAGS);
    assert_eq!(ledger.completed_count(), 1);
}

#[test]
fn restart_preserves_progress_and_the_completion_marker() {
    let catalogue = toys_catalogue();
    let mut ledger = CollectionLedger::new();
    collect_all(&mut ledger, &catalogue, &ARMY_MEN_ENTRIES);
    collect_all(&mut ledger, &catalogue, &TOPS_ENTRIES[..3]);

    let bytes = ledger.to_bytes().expect("the ledger serializes");
    let mut restored = CollectionLedger::from_bytes(&bytes).expect("the ledger reloads");

    assert_eq!(restored.collected_count(), ledger.collected_count());
    assert_eq!(restored.completed_count(), 1);
    assert!(restored.is_completed(ARMY_MEN));
    assert!(!restored.is_completed(TOPS));
    assert_eq!(
        restored.progress(&catalogue, TOPS).expect("Tops").collected,
        3
    );

    let replay = restored
        .collect(&catalogue, ARMY_MEN_ENTRIES[3])
        .expect("a collected entry stays collectable after a restart");
    assert!(!replay.newly_collected);
    assert!(
        replay.completion.is_none(),
        "a restart must not re-open an already completed collection"
    );

    // Completing the second collection after the restart still works, and the
    // first collection's marker is untouched.
    let outcomes = collect_all(&mut restored, &catalogue, &TOPS_ENTRIES[3..]);
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| outcome.completion.is_some())
            .count(),
        1,
        "only the entry that completed Tops reports a completion"
    );
    assert!(restored.is_completed(TOPS));
    assert_eq!(restored.completed_count(), 2);
}

#[test]
fn malformed_definitions_and_entries_are_refused() {
    let mut catalogue = CollectionCatalogue::new();
    assert_eq!(
        catalogue.insert(CollectionDefinition {
            id: ARMY_MEN,
            category_id: 11,
            entries: vec![1],
        }),
        Err(CollectionError::UnknownCategory(11))
    );
    assert_eq!(
        catalogue.insert(CollectionDefinition {
            id: ARMY_MEN,
            category_id: CATEGORY_TOYS,
            entries: vec![],
        }),
        Err(CollectionError::EmptyCollection(ARMY_MEN))
    );
    assert_eq!(
        catalogue.insert(CollectionDefinition {
            id: ARMY_MEN,
            category_id: CATEGORY_TOYS,
            entries: vec![7, 7],
        }),
        Err(CollectionError::DuplicateEntry(7))
    );
    catalogue
        .insert(CollectionDefinition {
            id: ARMY_MEN,
            category_id: CATEGORY_TOYS,
            entries: ARMY_MEN_ENTRIES.to_vec(),
        })
        .expect("the repaired definition is accepted");
    assert_eq!(
        catalogue.insert(CollectionDefinition {
            id: ARMY_MEN,
            category_id: CATEGORY_TOYS,
            entries: ARMY_MEN_ENTRIES.to_vec(),
        }),
        Err(CollectionError::DuplicateCollection(ARMY_MEN))
    );

    let mut ledger = CollectionLedger::new();
    assert_eq!(
        ledger.collect(&catalogue, 999_999),
        Err(CollectionError::UnknownEntry(999_999))
    );
    assert_eq!(ledger.collected_count(), 0);
}

#[test]
fn a_corrupt_or_unsupported_ledger_is_refused() {
    assert!(matches!(
        CollectionLedger::from_bytes(b"not a ledger"),
        Err(CollectionError::Corrupt(_))
    ));
    assert!(matches!(
        CollectionLedger::from_bytes(br#"{"schema_version":99,"collected":[],"completed":[]}"#),
        Err(CollectionError::Corrupt(_))
    ));
    assert!(matches!(
        CollectionLedger::from_bytes(
            br#"{"schema_version":1,"collected":[1],"completed":[],"extra":1}"#
        ),
        Err(CollectionError::Corrupt(_))
    ));
    let oversized = vec![b' '; piece_074::MAX_LEDGER_BYTES + 1];
    assert!(matches!(
        CollectionLedger::from_bytes(&oversized),
        Err(CollectionError::Corrupt(_))
    ));
}
