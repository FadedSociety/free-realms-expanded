//! Work item 014: recovered anvil pattern catalogue and equipment restrictions.
//!
//! The module is owned by item 014 and is included here until the lead
//! registers it. Expected values are the recovered records from the pinned
//! Legacy `ClientItemDefinitions.json`, `ItemClasses.txt`, `Profiles.json`
//! and the supplied client `EquipmentSlotDefinitions.txt`.
#[path = "../src/restoration/piece_014.rs"]
mod piece_014;

use piece_014::{
    BLACKSMITH, BRAWLER, EquipRejection, PATTERN_COUNT, ProfileRules, SLOT_WEAPON, allows,
    check_equip, class_owner, equipment, equipment_by_id, pattern, patterns, profile, slot,
    slot_count,
};

#[test]
fn pattern_catalogue_matches_recovered_class58_records() {
    assert_eq!(patterns().len(), PATTERN_COUNT);
    assert_eq!(patterns().len(), 28);

    let spunky = pattern(2506).expect("recovered Spunky Scrapper pattern");
    assert_eq!(spunky.name, "Spunky Scrapper Hammer Recipe");
    assert_eq!(spunky.param1, Some(11));
    assert_eq!(spunky.category_id, Some(17));
    assert_eq!(spunky.name_id, 1565);
    assert_eq!(spunky.description_id, 1587);
    assert_eq!(spunky.type_id, 5);

    // The Chef recipe and the one non-category-17 class58 record stay distinct.
    let mousse = pattern(869).expect("recovered Chef pattern");
    assert_eq!(mousse.name, "Chocolate Bumbleberry Mousse Recipe");
    assert_eq!(mousse.param1, Some(59));
    let shard = pattern(838).expect("recovered flair shard record");
    assert_eq!(shard.param1, None);
    assert_eq!(shard.category_id, Some(124));
    assert_eq!(shard.type_id, 1);

    assert!(pattern(2504).is_none());
}

#[test]
fn spunky_family_is_one_weapon_definition_with_ten_tints() {
    assert_eq!(equipment().len(), 12);
    let variants: Vec<_> = (30280..=30289)
        .map(|id| equipment_by_id(id).expect("recovered hammer variant"))
        .collect();
    assert_eq!(variants.len(), 10);
    for variant in &variants {
        assert_eq!(variant.name, "Spunky Scrapper Hammer");
        assert_eq!(variant.class_id, 71);
        assert_eq!(variant.slot, SLOT_WEAPON);
        assert_eq!(variant.category_id, 207);
        assert_eq!(variant.model, "hammer_ar_ag_weapon_logjammer.adr");
    }

    // The supplied substitute and the Blacksmith's own tool are the adjacent
    // recovered records the opening campaign depends on.
    let mallet = equipment_by_id(75030).expect("recovered substitute");
    assert_eq!(mallet.name, "Brawler's Mallet of Sweeps");
    assert_eq!(mallet.class_id, 25);
    let tool = equipment_by_id(6459).expect("recovered Blacksmith tool");
    assert_eq!(tool.name, "Student Blacksmith Hammer");
    assert_eq!(tool.class_id, 34);
    assert_eq!(tool.category_id, 107);
}

#[test]
fn item_class_owners_follow_the_recovered_profile_column() {
    let pattern_class = class_owner(58).expect("recipe class");
    assert_eq!(pattern_class.profile_id, BLACKSMITH);
    assert_eq!(pattern_class.profile_name_id, 2614);
    let hammer_class = class_owner(71).expect("hammer class");
    assert_eq!(hammer_class.profile_id, BRAWLER);
    assert_eq!(hammer_class.profile_name_id, 2873);
    assert_eq!(hammer_class.wield_type, 13);
    assert_eq!(class_owner(34).expect("tool class").profile_id, BLACKSMITH);
    assert_eq!(class_owner(25).expect("mallet class").profile_id, BRAWLER);
    assert!(class_owner(999).is_none());
}

#[test]
fn blacksmith_crafts_the_pattern_but_only_brawler_equips_the_hammer() {
    // The Blacksmith's own weapon slot holds the class34 tool, not class71.
    assert!(check_equip(BLACKSMITH, 6459).is_ok());
    assert_eq!(
        check_equip(BLACKSMITH, 30280).unwrap_err(),
        EquipRejection::ClassNotAllowed { class_id: 71 }
    );
    assert!(check_equip(BRAWLER, 30280).is_ok());
    assert_eq!(
        check_equip(BRAWLER, 6459).unwrap_err(),
        EquipRejection::ClassNotAllowed { class_id: 34 }
    );

    let blacksmith = profile(BLACKSMITH).expect("recovered Blacksmith rules");
    assert_eq!(blacksmith.name_id, 2614);
    assert!(blacksmith.allowed_classes.contains(&58));
    assert!(!blacksmith.allowed_classes.contains(&71));
    let brawler = profile(BRAWLER).expect("recovered Brawler rules");
    assert_eq!(brawler.name_id, 2873);
    assert!(brawler.allowed_classes.contains(&71));
    assert!(!brawler.allowed_classes.contains(&58));
}

#[test]
fn slot_seven_is_the_recovered_weapon_slot() {
    let weapon = slot(SLOT_WEAPON).expect("recovered weapon slot");
    assert_eq!(weapon.name, "Weapon");
    assert!(weapon.is_weapon);
    assert_eq!(slot_count(), 1);
    assert!(slot(1).is_none());
    // The recovered client profile records carry these slot spools.
    assert_eq!(profile(BLACKSMITH).unwrap().slots, &[1, 2, 3, 4, 5, 7, 13]);
    assert!(profile(BRAWLER).unwrap().slots.contains(&8));
    assert!(!profile(BLACKSMITH).unwrap().slots.contains(&8));
}

#[test]
fn unknown_definitions_profiles_and_slots_are_rejected() {
    assert_eq!(
        check_equip(BLACKSMITH, 2506).unwrap_err(),
        EquipRejection::UnknownDefinition
    );
    assert_eq!(
        check_equip(1, 30280).unwrap_err(),
        EquipRejection::UnknownProfile
    );
    assert_eq!(
        check_equip(BRAWLER, 999_999).unwrap_err(),
        EquipRejection::UnknownDefinition
    );
}

#[test]
fn a_profile_without_the_weapon_slot_rejects_the_hammer() {
    let no_weapon = ProfileRules {
        profile_id: 999,
        name_id: 0,
        name: "Fixture",
        allowed_classes: &[71],
        slots: &[1],
    };
    let hammer = equipment_by_id(30280).unwrap();
    assert_eq!(
        allows(&no_weapon, hammer).unwrap_err(),
        EquipRejection::SlotNotAllowed { slot: SLOT_WEAPON }
    );
    assert!(allows(profile(BRAWLER).unwrap(), hammer).is_ok());
}
