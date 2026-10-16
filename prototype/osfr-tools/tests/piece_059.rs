//! Piece 059 (dungeon definitions) tests: the recovered roster, difficulty policy, entrances
//! and the subset of dungeons whose zone definitions exist in the pinned pack catalogue.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (community/first-party inputs, hashes pinned in the lab bundle
//! `lab/reports/piece-059-20261009T054317Z/`): the wiki `Dungeons` page roster and the
//! per-dungeon entrance sentences, the locale achievement block 30439-30475 and the pack
//! catalogue's `*.gzne` / `*.gcnk` byte counts.
//!
//! RECONSTRUCTED (labelled in the module): [`entry_rule`] restates the wiki difficulty policy.
//!
//! NOT CLAIMED: encounters, spawns, waves, rewards, player counts, collision, or any native
//! dungeon run. 29 dungeons have no recovered zone definition.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact source file
// keeps this target compiling against the real artifact. Integration step for the lead: register
// the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_059.rs"]
#[allow(dead_code)]
mod piece_059;

use piece_059::{
    CatalogError, DIFFICULTY_MEMBERSHIP, DUNGEONS, MAX_DIFFICULTY, OBJECTIVE_LOCALE_FIRST,
    OBJECTIVE_LOCALE_LAST, REGIONS, ZONE_STUB_LIMIT, active, by_id, by_name, entry_rule,
    objective_locale, recovered_zone_maps, removed, validate, validate_recovered, with_difficulty,
};

#[test]
fn roster_shape_and_difficulty_membership_match_the_wiki_page() {
    assert_eq!(DUNGEONS.len(), 43);
    assert_eq!(active().len(), 41);
    assert_eq!(removed().len(), 2);
    for rank in 1..=MAX_DIFFICULTY {
        assert_eq!(
            with_difficulty(rank).len(),
            DIFFICULTY_MEMBERSHIP[rank as usize - 1]
        );
    }
    assert!(entry_rule(0).is_none());
    assert!(entry_rule(MAX_DIFFICULTY + 1).is_none());
}

#[test]
fn objective_rows_are_the_contiguous_ascending_achievement_block() {
    let mut covered = 0;
    let mut expected = OBJECTIVE_LOCALE_FIRST;
    for row in DUNGEONS.iter() {
        if let Some(locale) = row.objective_locale {
            assert_eq!(locale, expected, "{} is out of block order", row.name);
            assert!(locale <= OBJECTIVE_LOCALE_LAST);
            expected += 1;
            covered += 1;
        }
    }
    assert_eq!(covered, 37);
    assert_eq!(expected - 1, OBJECTIVE_LOCALE_LAST);
    assert_eq!(objective_locale(2), Some(30439));
    assert_eq!(objective_locale(43), Some(30475));
    assert_eq!(objective_locale(1), None);
    assert_eq!(objective_locale(999), None);
}

#[test]
fn objective_gaps_are_the_four_rows_the_block_does_not_cover() {
    let mut missing: Vec<&str> = DUNGEONS
        .iter()
        .filter(|row| row.objective_locale.is_none() && !row.removed)
        .map(|row| row.name)
        .collect();
    missing.sort_unstable();
    assert_eq!(
        missing,
        vec![
            "Bixie Hive",
            "Cursed Graveyard!",
            "Precursor Ruins",
            "Sandscale Oasis"
        ]
    );
}

#[test]
fn removed_rows_are_the_two_wiki_removals_without_achievements_or_zones() {
    let names: Vec<&str> = removed().iter().map(|row| row.name).collect();
    assert_eq!(names, vec!["Hewey's Escape", "King of the Ring!"]);
    for row in removed() {
        assert!(row.objective_locale.is_none());
        assert!(row.zone.is_none());
    }
}

#[test]
fn fourteen_zone_definitions_are_recovered_and_the_rest_are_gaps() {
    assert_eq!(recovered_zone_maps().len(), 14);
    let bixie = by_name("Bixie Hive").expect("Bixie Hive is on the roster");
    let zone = bixie
        .zone
        .expect("Bixie Hive has a recovered zone definition");
    assert_eq!(zone.stem, "sg_bixie_hive");
    assert_eq!(zone.zone_bytes, 1944);
    assert_eq!(zone.map_chunks, 9);
    assert_eq!(zone.map_bytes, 194_780);

    let spider = by_name("Arachnia's Lair").expect("Arachnia's Lair is on the roster");
    assert_eq!(spider.zone.expect("recovered zone").stem, "bw_spider_lair");

    let total: u32 = recovered_zone_maps()
        .iter()
        .map(|row| row.zone.expect("filtered").zone_bytes)
        .sum();
    assert_eq!(total, 43_071);

    let tavern = by_name("Tavern Cellar").expect("Tavern Cellar is on the roster");
    assert!(tavern.zone.is_none());
    assert!(tavern.stub_zone.is_none());
}

#[test]
fn header_only_stub_is_not_counted_as_a_recovered_map() {
    let den = by_name("Den of Secrets").expect("Den of Secrets is on the roster");
    assert!(den.zone.is_none());
    assert_eq!(den.stub_zone, Some("mv_den_of_secrets"));
    let summary = validate_recovered().expect("the recovered roster validates");
    assert_eq!(summary.recovered_zone_maps, 14);
    assert_eq!(summary.stub_zone_definitions, 1);
    assert_eq!(summary.missing_zone_maps, 29);
    assert_eq!(ZONE_STUB_LIMIT, 84);
}

#[test]
fn entrance_sentences_answer_with_place_trigger_and_region() {
    let bixie = by_name("Bixie Hive").expect("roster row");
    let entrance = bixie.entrance.expect("Bixie Hive has an entrance sentence");
    assert_eq!(entrance.trigger, "door");
    assert_eq!(entrance.area, Some("Queensfields"));
    assert_eq!(entrance.region, Some("Sanctuary"));
    assert!(REGIONS.contains(&"Sanctuary"));

    let sheep = by_name("Sheep Watch").expect("roster row");
    let sheep_entrance = sheep
        .entrance
        .expect("Sheep Watch has an entrance sentence");
    assert_eq!(sheep_entrance.place, "southwest Wilds, near Farnum's Farm");
    assert_eq!(sheep_entrance.region, Some("Wilds"));

    // The two pages without a usable entrance sentence stay explicit gaps.
    assert!(
        by_name("Precursor Ruins")
            .expect("roster row")
            .entrance
            .is_none()
    );
    assert!(
        by_name("Treeleaf's Retreat")
            .expect("roster row")
            .entrance
            .is_none()
    );
    let summary = validate_recovered().expect("the recovered roster validates");
    assert_eq!(summary.missing_entrances, 2);
}

#[test]
fn entry_rule_restates_the_wiki_difficulty_policy() {
    let rank1 = entry_rule(1).expect("rank 1 exists");
    assert_eq!(rank1.min_combat_level, None);
    assert!(rank1.recommended_only);
    let rank2 = entry_rule(2).expect("rank 2 exists");
    assert_eq!(rank2.min_combat_level, Some(3));
    assert!(rank2.recommended_only);
    assert_eq!(
        entry_rule(3).expect("rank 3 exists").min_combat_level,
        Some(5)
    );
    assert_eq!(
        entry_rule(4).expect("rank 4 exists").min_combat_level,
        Some(9)
    );
    let rank5 = entry_rule(5).expect("rank 5 exists");
    assert_eq!(rank5.min_combat_level, Some(13));
    assert!(!rank5.recommended_only);
}

#[test]
fn lookups_are_case_insensitive_and_bounded() {
    assert_eq!(by_id(1).expect("id 1 exists").name, "Bixie Hive");
    assert_eq!(by_id(43).expect("id 43 exists").name, "Trail of Betrayal");
    assert!(by_id(0).is_none());
    assert!(by_id(44).is_none());
    assert_eq!(
        by_name("cracked claw caverns")
            .expect("case-insensitive")
            .id,
        2
    );
    assert!(by_name("Briarheart").is_none());
}

#[test]
fn recovered_table_validates_and_counts_its_coverage() {
    let summary = validate_recovered().expect("the recovered roster validates");
    assert_eq!(summary.dungeons, 43);
    assert_eq!(summary.active, 41);
    assert_eq!(summary.removed, 2);
    assert_eq!(summary.objective_locales, 37);
    assert_eq!(summary.difficulty_membership, DIFFICULTY_MEMBERSHIP);
}

#[test]
fn malformed_tables_are_rejected() {
    let mut reordered = DUNGEONS.to_vec();
    reordered.swap(0, 1);
    assert_eq!(
        validate(&reordered).unwrap_err(),
        CatalogError::IdNotContiguous {
            expected: 1,
            found: 2
        }
    );

    let mut bad_difficulty = DUNGEONS.to_vec();
    bad_difficulty[0].difficulty = 6;
    assert_eq!(
        validate(&bad_difficulty).unwrap_err(),
        CatalogError::DifficultyOutOfRange {
            id: 1,
            difficulty: 6
        }
    );

    let mut duplicate = DUNGEONS.to_vec();
    duplicate[1].name = "Bixie Hive";
    assert_eq!(
        validate(&duplicate).unwrap_err(),
        CatalogError::DuplicateName("Bixie Hive")
    );

    let mut misaligned = DUNGEONS.to_vec();
    misaligned[1].objective_locale = Some(30445);
    assert_eq!(
        validate(&misaligned).unwrap_err(),
        CatalogError::ObjectiveLocaleMisaligned {
            id: 2,
            expected: 30439,
            found: 30445
        }
    );

    let mut removed_with_achievement = DUNGEONS.to_vec();
    removed_with_achievement[3].objective_locale = Some(30439);
    assert_eq!(
        validate(&removed_with_achievement).unwrap_err(),
        CatalogError::RemovedDungeonWithObjective("Hewey's Escape")
    );

    let mut duplicate_zone = DUNGEONS.to_vec();
    duplicate_zone[19].zone = duplicate_zone[0].zone;
    assert_eq!(
        validate(&duplicate_zone).unwrap_err(),
        CatalogError::DuplicateZoneStem("sg_bixie_hive")
    );

    let mut stub_and_zone = DUNGEONS.to_vec();
    stub_and_zone[0].stub_zone = Some("mv_den_of_secrets");
    assert_eq!(
        validate(&stub_and_zone).unwrap_err(),
        CatalogError::ZoneAndStubOnSameRow("Bixie Hive")
    );
}
