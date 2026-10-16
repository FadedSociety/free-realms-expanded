//! Work item 031 acceptance: original job/profile catalogue, localization and unlock coverage.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client-ui/ScriptsBase.lua` — the decompiled client Lua. `Profile_DS`
//!   (`BaseClient.Profiles`) declares the 16 job columns in order: Id, Name, Description, Type,
//!   Rank, StarsAvailable, Icon, BadgeImageSet, ButtonImageSet, AbilityBgImageSet, MembersOnly,
//!   StarsEarned, RankPercent, Hint, Trial, TrialExpired. `Profile_Filtered_DS` repeats the same
//!   order. `JobBrowser:StartJobQuest(profileID)` scans the ActivityFinder datasource for a row
//!   whose column 0 equals the profile id and starts column 4 as a breadcrumb quest.
//! * `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json` — the preserved original
//!   locale table (85,932 rows). Every recovered `name_string_id`/`description_string_id` resolves
//!   there; ids 4419971..4420001 are the "Master of Levels" list of the 16 level-20 jobs.
//! * `lab/references/rust-opening/profiles.json` — private captured profile payloads. 16 profiles
//!   (1, 2, 4, 11, 12, 14, 16, 32, 35, 43, 45, 48, 49, 52, 120, 137) with per-profile sha256
//!   proofs; captured state `active` 1 and `unlocked` [1, 43].
//! * `lab/references/Sanctuary/src/Resources/Profiles.json` — community profile table (id,
//!   NameId, DescriptionId, Type, icon and badge sets, DefaultItems). Community content that adds
//!   staff profiles 58 Referee and 20 Enforcer, which have no captured payload.
//! * `lab/references/freerealms-wiki/wiki.sqlite` — read-only wiki corpus. "Jobs" classifies the
//!   historic roster; "Pet Trainer" records the job removed on 2009-12-14.
//!
//! The catalogue is recovery metadata. It does not grant jobs, does not define XP and does not
//! claim native activation: the server-side profile rows and unlock quest ids were not captured.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_031;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_031.rs"]
#[allow(dead_code)]
mod piece_031;

use piece_031::{
    ACTIVITY_FINDER_PROFILE_COLUMN, ACTIVITY_FINDER_QUEST_COLUMN, CAPTURED_ACTIVE_ID,
    CAPTURED_PAYLOAD_IDS, CAPTURED_UNLOCKED_IDS, CatalogueError, Coverage, JobClassification,
    JobProvenance, PROFILE_DS_COLUMNS, PROFILE_TYPE_COMBAT, REMOVED_JOBS, job, job_unlock, jobs,
    validate, validate_locale,
};
use std::collections::BTreeMap;

/// The recovered original locale values for every catalogue name id.
fn recovered_names() -> BTreeMap<u32, String> {
    [
        (2836u32, "Adventurer"),
        (3100, "Ninja"),
        (3237, "Postman"),
        (2278, "Medic"),
        (2280, "Wizard"),
        (2470, "Miner"),
        (2614, "Blacksmith"),
        (4664, "Warrior"),
        (4822, "Archer"),
        (2873, "Brawler"),
        (2856, "Chef"),
        (2917, "Kart Driver"),
        (2919, "Demo Derby Driver"),
        (4650, "Soccer Star"),
        (2410, "Card Duelist"),
        (20178, "Fisherman"),
        (23268, "Referee"),
        (2301, "Enforcer"),
    ]
    .into_iter()
    .map(|(id, text)| (id, text.to_string()))
    .collect()
}

#[test]
fn catalogue_holds_the_sixteen_captured_profiles_plus_the_two_staff_rows() {
    assert_eq!(jobs().len(), 18);
    let captured: Vec<i32> = jobs()
        .iter()
        .filter(|row| row.provenance == JobProvenance::CapturedPayload)
        .map(|row| row.id)
        .collect();
    assert_eq!(captured, CAPTURED_PAYLOAD_IDS);
    let community: Vec<i32> = jobs()
        .iter()
        .filter(|row| row.provenance == JobProvenance::CommunityDefinition)
        .map(|row| row.id)
        .collect();
    assert_eq!(community, vec![20, 58]);
}

#[test]
fn every_row_keeps_a_distinct_recovered_name_string_id() {
    let mut ids: Vec<u32> = jobs().iter().map(|row| row.name_string_id).collect();
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count, "name string ids must be unique");
    for row in jobs() {
        assert!(!row.name.is_empty());
        assert_ne!(row.name_string_id, 0);
        assert_ne!(row.description_string_id, 0);
    }
    assert!(validate().is_ok());
}

#[test]
fn localization_validation_uses_the_name_string_ids() {
    let locale = recovered_names();
    assert!(validate_locale(|id| locale.get(&id).cloned()).is_ok());
    let mut drifted = recovered_names();
    drifted.insert(2873, "Boxer".to_string());
    let error = validate_locale(|id| drifted.get(&id).cloned()).unwrap_err();
    assert_eq!(
        error,
        CatalogueError::LocalizedNameDiffers {
            id: 43,
            name_string_id: 2873,
            found: "Boxer".to_string(),
        }
    );
    let error = validate_locale(|id| if id == 2836 { None } else { Some("x".into()) }).unwrap_err();
    assert_eq!(
        error,
        CatalogueError::MissingNameString {
            id: 1,
            name_string_id: 2836
        }
    );
}

#[test]
fn combat_rows_are_exactly_the_recovered_type_two_jobs() {
    let by_type: Vec<i32> = jobs()
        .iter()
        .filter(|row| row.profile_type == PROFILE_TYPE_COMBAT)
        .map(|row| row.id)
        .collect();
    let by_class: Vec<i32> = jobs()
        .iter()
        .filter(|row| row.classification == JobClassification::Combat)
        .map(|row| row.id)
        .collect();
    assert_eq!(by_type, vec![2, 11, 12, 32, 35, 43]);
    assert_eq!(by_type, by_class);
}

#[test]
fn classifications_match_the_wiki_roster_split() {
    let count = |class: JobClassification| {
        jobs()
            .iter()
            .filter(|row| row.classification == class)
            .count()
    };
    assert_eq!(count(JobClassification::Combat), 6);
    assert_eq!(count(JobClassification::Crafting), 3);
    assert_eq!(count(JobClassification::Freestyle), 7);
    assert_eq!(count(JobClassification::Staff), 2);
    for id in [14, 16, 45] {
        assert_eq!(job(id).unwrap().classification, JobClassification::Crafting);
    }
    for id in [48, 49, 52, 120, 137] {
        assert_eq!(
            job(id).unwrap().classification,
            JobClassification::Freestyle
        );
    }
}

#[test]
fn coverage_reports_captured_community_only_and_removed_rows() {
    let coverage = Coverage::recovered();
    assert_eq!(coverage.captured, CAPTURED_PAYLOAD_IDS.to_vec());
    assert_eq!(coverage.community_only, vec![20, 58]);
    assert_eq!(coverage.removed, vec!["Pet Trainer"]);
    assert!(coverage.payload_for(1));
    assert!(coverage.payload_for(137));
    assert!(!coverage.payload_for(58));
    assert!(!coverage.payload_for(20));
    assert_eq!(REMOVED_JOBS.len(), 1);
    assert_eq!(REMOVED_JOBS[0].name_string_id, 17419);
    assert_eq!(REMOVED_JOBS[0].classification, JobClassification::Freestyle);
    assert_eq!(REMOVED_JOBS[0].removed_utc, "2009-12-14");
}

#[test]
fn client_profile_datasource_columns_keep_the_recovered_order() {
    assert_eq!(PROFILE_DS_COLUMNS.len(), 16);
    assert_eq!(PROFILE_DS_COLUMNS[0], "Id");
    assert_eq!(PROFILE_DS_COLUMNS[1], "Name");
    assert_eq!(PROFILE_DS_COLUMNS[2], "Description");
    assert_eq!(PROFILE_DS_COLUMNS[3], "Type");
    assert_eq!(PROFILE_DS_COLUMNS[13], "Hint");
    assert_eq!(PROFILE_DS_COLUMNS[14], "Trial");
    assert_eq!(PROFILE_DS_COLUMNS[15], "TrialExpired");
}

#[test]
fn unlock_evidence_stays_client_side_and_names_the_captured_state() {
    assert_eq!(ACTIVITY_FINDER_PROFILE_COLUMN, 0);
    assert_eq!(ACTIVITY_FINDER_QUEST_COLUMN, 4);
    assert_eq!(CAPTURED_ACTIVE_ID, 1);
    assert_eq!(CAPTURED_UNLOCKED_IDS, [1, 43]);
    let unlock = job_unlock();
    assert_eq!(unlock.active_id, 1);
    assert_eq!(unlock.unlocked_ids, vec![1, 43]);
    assert_eq!(unlock.active_name(), "Adventurer");
    assert_eq!(unlock.unlocked_name(43), Some("Brawler"));
    // Server-side job rows and unlock quest ids were never captured.
    assert!(!unlock.quest_ids_recovered);
}
