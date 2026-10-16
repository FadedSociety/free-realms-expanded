//! Work item 032 acceptance: Brawler progression definitions and unlock / reward boundaries.
//!
//! ORIGINAL evidence behind these vectors (none of it is authored here):
//! * `Profiles.json` (community conversion of the supplied client profile table, AGPL-3.0,
//!   `raisingkaines/FreeRealms-Legacy`) — the Brawler row `Id` 43, `NameId` 2873,
//!   `DescriptionId` 3234, `Type` 2 (combat), `Icon` 1337, `Hint` 384195, 25 item classes and the
//!   seven-item creation kit `DefaultItems` `7676 7536 7495 7939 7571 7781 75030`.
//! * `Resources/ItemClasses.txt` of the supplied client — `ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^
//!   PROFILE_NAME_ID`; rows 5, 25, 42, 52 and 71 carry `PROFILE_NAME_ID` 2873.
//! * `ClientItemDefinitions.json` — every equipment row's `Class` and `MinProfileRank`, the rank
//!   boundary this item unlocks at. Rank 0 rows carry no rank requirement.
//! * `Resources/Requirements.txt`, `RequirementGroups.txt`, `RequirementSets.txt` of the supplied
//!   client — the recovered `CheckRankLastProfileByType` rows and the AND groups that combine them.
//! * `Resources/ItemFilterOptions.txt` — row 11 `2873 -> 43` binds the Brawler locale name id to the
//!   Brawler profile id.
//! * `lab/reports/locale-id-to-text.json` (unmodified locale export of the supplied client) — the
//!   trait name block 420938..420941 and the trait text block 420962..420965.
//!
//! COMMUNITY evidence, kept separate: `notes` wiki revision 48172 lists the Brawler equipment levels
//! 1, 3, 4, 5, 8, 11, 12, 13, 15, 16, 17, 19, 20 and the trait ranks 5, 10, 15, 20.
//!
//! UNRECOVERED, and therefore asserted as gaps rather than numbers: the per-rank experience
//! thresholds. No preserved table carries them; the client receives `Rank` and `RankPercent` from the
//! server. `JobProgression::experience_gaps` names exactly what is missing.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod restoration::piece_032;` in `src/lib.rs`.

#[path = "../src/restoration/piece_032.rs"]
#[allow(dead_code)]
mod piece_032;

use piece_032::{
    BRAWLER_DESCRIPTION_ID, BRAWLER_ICON_ID, BRAWLER_NAME_ID, BRAWLER_PROFILE_ID,
    BRAWLER_TRAIT_NAME_IDS, BRAWLER_TRAIT_RANKS, COMBAT_PROFILE_TYPE, FilterOptionRow,
    ItemClassRow, ItemRow, JobProgression, MAX_JOB_RANK, ProfileRow, ProgressionError,
    RankComparator, RankGate, RankWindow, RequirementRow, parse_id_pairs, parse_item_class_table,
    parse_item_filter_options, parse_requirement_table,
};
use std::collections::{BTreeMap, BTreeSet};

const CLASS_HEADER: &str = "#ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^PROFILE_NAME_ID^\n";

fn profile_row() -> ProfileRow {
    let json = r#"{
        "Comment": "Brawler",
        "Id": 43,
        "NameId": 2873,
        "DescriptionId": 3234,
        "Type": 2,
        "Icon": 1337,
        "BadgeImageSet": 10,
        "ButtonImageSet": 1,
        "AbilityBgImageSet": 4,
        "Hint": 384195,
        "MembersOnly": false,
        "Trial": false,
        "Unknown": 0,
        "Unknown2": 0,
        "ItemClasses": {
            "5": {"Class": 5, "Unknown": 5},
            "25": {"Class": 25, "Unknown": 5},
            "71": {"Class": 71, "Unknown": 5}
        },
        "DefaultItems": [7676, 75030]
    }"#;
    serde_json::from_str(json).expect("shipped profile row shape parses")
}

fn class_table() -> Vec<ItemClassRow> {
    parse_item_class_table(&format!(
        "{CLASS_HEADER}5^2925^10^13^19^2873^\n25^23266^29^0^16^2873^\n42^2247^55^13^0^2873^\n\
         71^2870^55^13^0^2873^\n"
    ))
    .expect("class table parses")
}

fn item(id: i32, comment: &str, class_id: i32, slot: i32, rank: i32) -> ItemRow {
    ItemRow {
        id,
        comment: comment.to_owned(),
        class_id: Some(class_id),
        min_profile_rank: rank,
        slot: Some(slot),
        name_id: Some(id),
        profile_override: None,
    }
}

/// Two live weapons (rank 1 and rank 15), one rank-5 development row, one rank-0 always-available
/// weapon and a creation-kit piece, mirroring the shipped shapes.
fn items() -> Vec<ItemRow> {
    vec![
        item(75030, "Brawler's Mallet of Sweeps", 25, 7, 1),
        item(8050, "Pro Brawler Hammer", 5, 7, 1),
        item(46000, "Strong Scrapper Hammer", 5, 7, 15),
        item(70300, "Level 5 Brawler Test Weapon", 5, 7, 5),
        item(78197, "Brawler's Power Fist", 71, 7, 0),
        item(7676, "Student Brawler Cap", 25, 1, 1),
    ]
}

#[test]
fn shipped_profile_row_parses_into_the_recovered_shape() {
    let row = profile_row();
    assert_eq!(row.id, 43);
    assert_eq!(row.name_id, 2873);
    assert_eq!(row.description_id, 3234);
    assert_eq!(row.profile_type, COMBAT_PROFILE_TYPE);
    assert_eq!(row.hint, 384195);
    assert!(!row.members_only);
    assert!(!row.trial);
    assert_eq!(row.default_items, vec![7676, 75030]);
    assert_eq!(row.item_classes.len(), 3);
}

#[test]
fn class_table_parses_and_rejects_a_wrong_header() {
    let rows = class_table();
    assert_eq!(rows[0].id, 5);
    assert_eq!(rows[0].wield_type, 13);
    assert_eq!(rows[0].profile_name_id, BRAWLER_NAME_ID);
    assert_eq!(rows[1].id, 25);

    let wrong = parse_item_class_table("#ID^NAME_ID^\n5^2925^\n");
    assert!(matches!(
        wrong,
        Err(ProgressionError::HeaderMismatch { table }) if table == "ItemClasses"
    ));
}

#[test]
fn requirement_table_keeps_only_check_rank_rows_as_gates() {
    let text = "#client_requirements.ID^client_requirements.TYPE^ref_client_req_types.CODE_FACTORY_NAME^client_requirements.OPERATOR^client_requirements.PARAM_1^client_requirements.PARAM_2^client_requirements.PARAM_3^client_requirements.STRING_ID^client_requirements.PARAM_4^client_requirements.PARAM_5^\n\
                3^3^CheckRankLastProfileByType^6^2^5^0^0^^^\n\
                25^4^AlwaysFails^1^0^0^0^0^^^\n";
    let rows = parse_requirement_table(text).expect("requirement table parses");
    assert_eq!(rows.len(), 2);

    let gates: Vec<RankGate> = rows.iter().filter_map(RankGate::from_row).collect();
    assert_eq!(gates.len(), 1);
    assert_eq!(
        gates[0],
        RankGate {
            requirement_id: 3,
            profile_type: 2,
            comparator: RankComparator::AtLeast,
            rank: 5,
        }
    );
}

#[test]
fn gate_comparators_are_inclusive_at_the_boundary() {
    let at_least = RankGate {
        requirement_id: 3,
        profile_type: COMBAT_PROFILE_TYPE,
        comparator: RankComparator::AtLeast,
        rank: 5,
    };
    let at_most = RankGate {
        requirement_id: 4,
        profile_type: COMBAT_PROFILE_TYPE,
        comparator: RankComparator::AtMost,
        rank: 8,
    };

    assert!(!at_least.matches(4));
    assert!(at_least.matches(5));
    assert!(at_most.matches(8));
    assert!(!at_most.matches(9));
    assert!(
        !at_least.matches_for(1, 20),
        "another job type must not satisfy a combat gate"
    );
}

#[test]
fn development_rows_are_reported_separately_from_the_live_rank_boundaries() {
    let progression = JobProgression::build(&profile_row(), &class_table(), &items())
        .expect("progression builds");

    assert_eq!(progression.development_items, vec![70300]);
    assert_eq!(progression.development_ranks(), BTreeSet::from([5]));
    assert!(
        !progression.observed_ranks().contains(&5),
        "a rank whose only weapon row is a development row is not a live unlock boundary"
    );
    assert_eq!(progression.observed_ranks(), BTreeSet::from([1, 15]));
    assert_eq!(
        progression.unlocked_at(1),
        vec![75030, 8050, 78197],
        "rank 0 equipment is always available"
    );
    assert_eq!(progression.unlocked_at(15).len(), 4);
    assert_eq!(progression.unlocked_at(20).len(), 4);
}

#[test]
fn unlock_curve_is_monotone_over_the_recovered_rank_axis() {
    let progression = JobProgression::build(&profile_row(), &class_table(), &items())
        .expect("progression builds");
    let curve = progression.unlock_curve();
    assert_eq!(curve.len(), MAX_JOB_RANK as usize);
    for pair in curve.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "unlocks must not shrink as rank grows: {curve:?}"
        );
    }
    assert_eq!(curve[0], 3);
    assert_eq!(*curve.last().expect("curve"), 4);
}

#[test]
fn equipment_above_the_recovered_rank_ceiling_is_a_violation() {
    let mut rows = items();
    rows.push(item(999999, "Future Brawler Hammer", 5, 7, 21));
    let progression =
        JobProgression::build(&profile_row(), &class_table(), &rows).expect("progression builds");

    assert_eq!(progression.violations.len(), 1);
    assert_eq!(progression.violations[0].item_id, 999999);
    assert_eq!(progression.violations[0].rank, 21);
    assert!(
        !progression.observed_ranks().contains(&21),
        "an out-of-range row must not create a live unlock boundary"
    );
}

#[test]
fn equipment_from_another_jobs_class_is_not_attributed_to_this_job() {
    let rows = vec![
        item(75030, "Brawler's Mallet of Sweeps", 25, 7, 1),
        item(1234, "Student Archer Bow", 15, 7, 1),
    ];
    let progression =
        JobProgression::build(&profile_row(), &class_table(), &rows).expect("progression builds");

    // Class 15 is another profile's weapon class; it must not become a Brawler boundary.
    assert!(progression.violations.is_empty());
    assert_eq!(progression.unlocked_at(20), vec![75030]);
}

#[test]
fn declared_community_levels_are_cross_checked_against_the_original_rows() {
    let progression = JobProgression::build(&profile_row(), &class_table(), &items())
        .expect("progression builds");
    let declared = BTreeSet::from([1, 5, 15, 19]);
    let check = progression.cross_check_declared_ranks(&declared);

    assert_eq!(check.missing, BTreeSet::from([19]));
    assert_eq!(check.unexpected, BTreeSet::new());
    assert_eq!(check.development_only, BTreeSet::from([5]));

    let strict = progression.cross_check_declared_ranks(&BTreeSet::from([1, 15]));
    assert!(strict.missing.is_empty());
    assert!(strict.unexpected.is_empty());
}

#[test]
fn the_creation_kit_is_within_the_profile_classes_and_rank_floor() {
    let mut rows = items();
    rows.push(item(7676, "Student Brawler Cap", 25, 1, 1));
    let progression =
        JobProgression::build(&profile_row(), &class_table(), &rows).expect("progression builds");
    let kit = progression.creation_kit(&profile_row());

    assert_eq!(kit.len(), 2);
    assert!(
        kit.iter()
            .all(|entry| entry.class_wieldable && entry.rank <= 1)
    );
}

#[test]
fn combat_rank_windows_pair_the_recovered_gate_rows() {
    let requirements = vec![
        requirement(3, 6, 2, 5),
        requirement(4, 4, 2, 12),
        requirement(5, 6, 2, 9),
    ];
    let groups = vec![(3, 3), (3, 4), (4, 5)];
    let sets: Vec<(i32, i32)> = vec![(3, 3), (4, 4)];

    let windows = RankWindow::for_sets(&requirements, &groups, &sets, COMBAT_PROFILE_TYPE)
        .expect("windows build");
    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0].lower.expect("lower bound").1, 5);
    assert_eq!(windows[0].upper.expect("upper bound").1, 12);
    assert!(windows[0].contains(5));
    assert!(windows[0].contains(12));
    assert!(!windows[0].contains(13));
    assert_eq!(windows[1].lower.expect("lower bound").1, 9);
    assert!(windows[1].upper.is_none());
}

#[test]
fn gate_rows_for_a_different_profile_type_do_not_form_a_window() {
    let requirements = vec![requirement(3, 6, 2, 5), requirement(13, 6, 10, 9)];
    let groups = vec![(3, 3), (3, 13)];
    let sets = vec![(3, 3)];

    let error = RankWindow::for_sets(&requirements, &groups, &sets, COMBAT_PROFILE_TYPE)
        .expect_err("mixed profile types are rejected");
    assert!(matches!(
        error,
        ProgressionError::MixedGateTypes { group_id: 3 }
    ));
}

#[test]
fn profile_identity_is_confirmed_by_the_client_filter_option_row() {
    let text = "#ID^NAME_ID^HAS_RANGE^VALUE^VALUE_RANGE_START^VALUE_RANGE_END^\n\
                11^2873^0^43^0^0^\n\
                8^2836^0^1^0^0^\n";
    let rows: Vec<FilterOptionRow> = parse_item_filter_options(text).expect("filter table parses");
    assert_eq!(rows[0].name_id, BRAWLER_NAME_ID);
    assert_eq!(rows[0].value, BRAWLER_PROFILE_ID);
    assert_eq!(rows[1].name_id, 2836);
    assert_eq!(rows[1].value, 1);
}

#[test]
fn id_pair_tables_parse_and_reject_unknown_headers() {
    let rows = parse_id_pairs(
        "#REQUIREMENT_GROUP_ID^REQUIREMENT_ID^\n3^3^\n3^4^\n",
        ["REQUIREMENT_GROUP_ID", "REQUIREMENT_ID"],
        "RequirementGroups",
    )
    .expect("groups parse");
    assert_eq!(rows, vec![(3, 3), (3, 4)]);

    let bad = parse_id_pairs(
        "#NOPE^REQUIREMENT_ID^\n3^3^\n",
        ["A", "B"],
        "RequirementGroups",
    );
    assert!(bad.is_err());
}

#[test]
fn recovered_constants_keep_original_and_community_labels_apart() {
    assert_eq!(BRAWLER_PROFILE_ID, 43);
    assert_eq!(BRAWLER_NAME_ID, 2873);
    assert_eq!(BRAWLER_DESCRIPTION_ID, 3234);
    assert_eq!(BRAWLER_ICON_ID, 1337);
    assert_eq!(MAX_JOB_RANK, 20);
    assert_eq!(BRAWLER_TRAIT_NAME_IDS, [420938, 420939, 420940, 420941]);
    // COMMUNITY wiki mapping, not an original table; the module must not present it as recovered.
    assert_eq!(BRAWLER_TRAIT_RANKS, [5, 10, 15, 20]);
}

fn requirement(id: i32, operator: i32, profile_type: i32, rank: i32) -> RequirementRow {
    RequirementRow {
        id,
        requirement_type: 3,
        factory: "CheckRankLastProfileByType".to_owned(),
        operator,
        param_1: profile_type,
        param_2: rank,
        param_3: 0,
    }
}

fn read_env(name: &str) -> std::path::PathBuf {
    std::env::var_os(name)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| panic!("{name} must point at the preserved table"))
}

#[test]
#[ignore = "requires the preserved client tables under lab/references and lab/runs"]
fn private_brawler_tables_match_the_preserved_lab_inputs() {
    let profiles_path = read_env("OSFR_PIECE_032_PROFILES");
    let items_path = read_env("OSFR_PIECE_032_ITEMS");
    let classes_path = read_env("OSFR_PIECE_032_CLASSES");
    let requirements_path = read_env("OSFR_PIECE_032_REQUIREMENTS");
    let groups_path = read_env("OSFR_PIECE_032_GROUPS");
    let sets_path = read_env("OSFR_PIECE_032_SETS");
    let filters_path = read_env("OSFR_PIECE_032_FILTERS");

    let profiles: Vec<ProfileRow> =
        serde_json::from_slice(&std::fs::read(&profiles_path).expect("readable Profiles.json"))
            .expect("Profiles.json parses");
    let items: Vec<ItemRow> = serde_json::from_slice(
        &std::fs::read(&items_path).expect("readable ClientItemDefinitions.json"),
    )
    .expect("ClientItemDefinitions.json parses");
    let classes = parse_item_class_table(
        &std::fs::read_to_string(&classes_path).expect("readable ItemClasses.txt"),
    )
    .expect("class table parses");
    let requirements = parse_requirement_table(
        &std::fs::read_to_string(&requirements_path).expect("readable Requirements.txt"),
    )
    .expect("requirement table parses");
    let groups = parse_id_pairs(
        &std::fs::read_to_string(&groups_path).expect("readable RequirementGroups.txt"),
        ["REQUIREMENT_GROUP_ID", "REQUIREMENT_ID"],
        "RequirementGroups",
    )
    .expect("groups parse");
    let sets = parse_id_pairs(
        &std::fs::read_to_string(&sets_path).expect("readable RequirementSets.txt"),
        ["REQUIREMENT_SET_ID", "REQUIREMENT_GROUP_ID"],
        "RequirementSets",
    )
    .expect("sets parse");
    let filters = parse_item_filter_options(
        &std::fs::read_to_string(&filters_path).expect("readable ItemFilterOptions.txt"),
    )
    .expect("filter options parse");

    // The client filter table independently binds the recovered name id to the profile id.
    let brawler_filter = filters
        .iter()
        .find(|row| row.name_id == BRAWLER_NAME_ID)
        .expect("Brawler filter option row");
    assert_eq!(brawler_filter.value, BRAWLER_PROFILE_ID);

    let profile = profiles
        .iter()
        .find(|row| row.name_id == BRAWLER_NAME_ID)
        .expect("Brawler profile row");
    assert_eq!(profile.id, BRAWLER_PROFILE_ID);
    assert_eq!(profile.profile_type, COMBAT_PROFILE_TYPE);
    assert_eq!(profile.item_classes.len(), 25);
    assert_eq!(
        profile.default_items,
        vec![7676, 7536, 7495, 7939, 7571, 7781, 75030]
    );

    let progression = JobProgression::build(profile, &classes, &items).expect("progression builds");
    assert!(
        progression.violations.is_empty(),
        "no shipped Brawler row may break a boundary: {:?}",
        progression.violations
    );

    // The five classes the class table marks as Brawler's own.
    let owned: BTreeSet<i32> = progression
        .wield_classes
        .iter()
        .filter(|class| class.owned)
        .map(|class| class.id)
        .collect();
    assert_eq!(owned, BTreeSet::from([5, 25, 42, 52, 71]));
    assert_eq!(progression.wield_classes.len(), 25);

    // The Brawler creation kit is class 25 across slots 1..7 at rank 1.
    let kit = progression.creation_kit(profile);
    assert_eq!(kit.len(), 7);
    assert!(
        kit.iter()
            .all(|entry| entry.class_wieldable && entry.rank == 1)
    );
    let kit_slots: BTreeSet<i32> = kit.iter().filter_map(|entry| entry.slot).collect();
    assert_eq!(kit_slots, BTreeSet::from([1, 2, 3, 4, 5, 6, 7]));

    // Weapon unlock boundaries: three ranks carry nothing but development rows, and the remaining
    // ranks are exactly the levels the community wiki lists for Brawler weapons.
    let declared = BTreeSet::from([1, 3, 4, 5, 8, 11, 12, 13, 15, 16, 17, 19, 20]);
    let check = progression.cross_check_declared_ranks(&declared);
    assert_eq!(check.missing, BTreeSet::new());
    assert_eq!(check.unexpected, BTreeSet::new());
    assert_eq!(check.development_only, BTreeSet::from([2, 10, 18]));
    assert_eq!(progression.observed_ranks(), declared);
    assert_eq!(progression.equipment[&20].len(), 11);
    assert_eq!(progression.equipment[&1].len(), 447);
    assert_eq!(progression.equipment.get(&7), None);
    assert_eq!(progression.equipment.get(&9), None);
    assert_eq!(
        progression.unrestricted.len(),
        6,
        "the shipped rank-0 weapons carry no rank requirement"
    );
    assert_eq!(progression.development_items.len(), 17);
    assert_eq!(
        progression
            .equipment
            .values()
            .map(|bucket| bucket.len())
            .sum::<usize>(),
        907
    );

    // Combat rank gates: eight recovered rows in six AND groups, all on profile type 2.
    let gates: Vec<RankGate> = requirements.iter().filter_map(RankGate::from_row).collect();
    assert_eq!(gates.len(), 24);
    let combat: Vec<RankGate> = gates
        .iter()
        .filter(|gate| gate.profile_type == COMBAT_PROFILE_TYPE)
        .copied()
        .collect();
    assert_eq!(combat.len(), 8);
    let windows = RankWindow::for_sets(&requirements, &groups, &sets, COMBAT_PROFILE_TYPE)
        .expect("combat windows build");
    assert_eq!(windows.len(), 6);
    let bounds: Vec<(Option<i32>, Option<i32>)> = windows
        .iter()
        .map(|window| {
            (
                window.lower.map(|(_, rank)| rank),
                window.upper.map(|(_, rank)| rank),
            )
        })
        .collect();
    assert_eq!(
        bounds,
        vec![
            (Some(10), None),
            (None, Some(8)),
            (Some(5), Some(12)),
            (Some(9), Some(16)),
            (Some(13), None),
            (Some(17), None),
        ]
    );

    // No rank axis in the shipped catalogue exceeds the recovered ceiling.
    let max_rank = items
        .iter()
        .map(|row| row.min_profile_rank)
        .max()
        .expect("item rows");
    assert_eq!(max_rank, MAX_JOB_RANK);

    // The experience curve is not in any preserved table; the gap stays explicit.
    let gaps = progression.experience_gaps();
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].contains("RankPercent"));

    // Bounded-map note: the shipped file is 58,795 rows, so the vectors stay well inside the cap.
    assert!(items.len() < 65_536);
    assert!(BTreeMap::from([(1, 1)]).contains_key(&1));
}
