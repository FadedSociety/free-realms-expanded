//! Work item 033: recovered Warrior job, skill, ownership and level restrictions.
//!
//! The module is owned by item 033 and is included here until the lead
//! registers it. Expected values are the recovered records from the pinned
//! Legacy `Profiles.json`, `ItemClasses.txt` and `ClientItemDefinitions.json`,
//! the supplied client `EquipmentSlotDefinitions.txt`, the preserved
//! `reports/locale-id-to-text.json` locale map and the read-only wiki corpus.
//! None of the Warrior numbers below are Brawler values.
#[path = "../src/restoration/piece_033.rs"]
mod piece_033;

use piece_033::{
    EquipRejection, WARRIOR_ABILITY_BG_IMAGE_SET, WARRIOR_BADGE_IMAGE_SET,
    WARRIOR_BUTTON_IMAGE_SET, WARRIOR_CLASS_ALLOWLIST, WARRIOR_CLASSES, WARRIOR_CREATION_KIT,
    WARRIOR_DESCRIPTION_ID, WARRIOR_GEAR_CLASS, WARRIOR_GEAR_LEVEL_GATES, WARRIOR_HINT_ID,
    WARRIOR_ICON_ID, WARRIOR_JEWELRY_CLASS, WARRIOR_JOB_DESCRIPTION, WARRIOR_JOB_HINT,
    WARRIOR_JOB_NAME, WARRIOR_LEVEL_TEST_EQUIPMENT, WARRIOR_MEMBERS_ONLY, WARRIOR_NAME_ID,
    WARRIOR_PROFILE_ID, WARRIOR_PROFILE_TYPE, WARRIOR_SHARD_CLASS, WARRIOR_SKILLS, WARRIOR_TRIAL,
    WARRIOR_WEAPON_ABILITIES, WARRIOR_WEAPON_CLASS, WARRIOR_WEAPON_LEVEL_GATES,
    check_warrior_equip, skill_unlocked, unlocked_skill_name_ids, warrior_class, warrior_skill,
    warrior_slot,
};

#[test]
fn warrior_job_identity_is_recovered() {
    assert_eq!(WARRIOR_PROFILE_ID, 32);
    assert_eq!(WARRIOR_NAME_ID, 4664);
    assert_eq!(WARRIOR_DESCRIPTION_ID, 4665);
    assert_eq!(WARRIOR_HINT_ID, 384191);
    assert_eq!(WARRIOR_ICON_ID, 21594);
    assert_eq!(WARRIOR_PROFILE_TYPE, 2);
    assert_eq!(WARRIOR_BADGE_IMAGE_SET, 10);
    assert_eq!(WARRIOR_BUTTON_IMAGE_SET, 1);
    assert_eq!(WARRIOR_ABILITY_BG_IMAGE_SET, 31);
    assert!(!WARRIOR_MEMBERS_ONLY);
    assert!(WARRIOR_TRIAL);

    assert_eq!(WARRIOR_JOB_NAME, "Warrior");
    assert_eq!(
        WARRIOR_JOB_DESCRIPTION,
        "Warriors are hardy Brawlers that can take and dish out damage."
    );
    assert_eq!(
        WARRIOR_JOB_HINT,
        "A true warrior does not fear harsh weather."
    );
}

#[test]
fn warrior_classes_are_owned_by_warrior_not_brawler() {
    assert_eq!(WARRIOR_CLASSES.len(), 4);
    // The four recovered `ItemClasses.txt` rows whose owner name id is 4664.
    assert_eq!(WARRIOR_GEAR_CLASS, 15);
    assert_eq!(WARRIOR_WEAPON_CLASS, 17);
    assert_eq!(WARRIOR_JEWELRY_CLASS, 45);
    assert_eq!(WARRIOR_SHARD_CLASS, 55);

    let gear = warrior_class(15).expect("recovered Warrior Gear class");
    assert_eq!(gear.name, "Warrior Gear");
    assert_eq!(gear.name_id, 2935);
    assert_eq!(gear.icon_id, 497);
    assert_eq!(gear.wield_type, 0);
    assert_eq!(gear.stat_id, 16);
    assert_eq!(gear.owner_name_id, WARRIOR_NAME_ID);

    let weapons = warrior_class(17).expect("recovered Warrior Axes & Hammers class");
    assert_eq!(weapons.name, "Warrior Axes & Hammers");
    assert_eq!(weapons.name_id, 2937);
    assert_eq!(weapons.wield_type, 12);
    assert_eq!(weapons.stat_id, 19);
    assert_eq!(weapons.owner_name_id, WARRIOR_NAME_ID);

    let jewelry = warrior_class(45).expect("recovered Warrior Jewelry class");
    assert_eq!(jewelry.name, "Warrior Jewelry");
    assert_eq!(jewelry.name_id, 2250);
    assert_eq!(jewelry.wield_type, 12);

    let shards = warrior_class(55).expect("recovered Warrior Shards class");
    assert_eq!(shards.name, "Warrior Shards");
    assert_eq!(shards.name_id, 2318);
    assert_eq!(shards.wield_type, 12);

    assert!(warrior_class(15).is_some());
    assert!(warrior_class(17).is_some());
    assert!(warrior_class(45).is_some());
    assert!(warrior_class(55).is_some());

    // Brawler classes 25 (`Brawler Gear`) and 71 (`Brawler 1-Handed Hammers`)
    // are owned by name id 2873 and are absent from the Warrior allowlist.
    assert!(warrior_class(25).is_none());
    assert!(warrior_class(71).is_none());
    assert!(!WARRIOR_CLASS_ALLOWLIST.contains(&25));
    assert!(!WARRIOR_CLASS_ALLOWLIST.contains(&71));
}

#[test]
fn warrior_class_allowlist_matches_recovered_profile_packet() {
    assert_eq!(
        WARRIOR_CLASS_ALLOWLIST,
        &[
            15, 17, 45, 50, 55, 69, 144, 145, 146, 150, 151, 153, 154, 155, 156, 158, 161, 162,
            168, 173, 183, 999,
        ]
    );
    for class in [15, 17, 45, 55] {
        assert!(WARRIOR_CLASS_ALLOWLIST.contains(&class));
    }
}

#[test]
fn warrior_creation_kit_is_the_recovered_default_item_list() {
    let ids: Vec<i32> = WARRIOR_CREATION_KIT.iter().map(|item| item.id).collect();
    assert_eq!(ids, vec![46644, 46389, 46137, 46732, 46224, 47019, 29963]);

    let helm = &WARRIOR_CREATION_KIT[0];
    assert_eq!(helm.name, "Student Warrior Helm");
    assert_eq!(helm.class_id, WARRIOR_GEAR_CLASS);
    assert_eq!(helm.slot, 1);
    assert_eq!(helm.min_profile_rank, 1);

    let cudgel = &WARRIOR_CREATION_KIT[6];
    assert_eq!(cudgel.name, "Student Warrior Cudgel");
    assert_eq!(cudgel.class_id, WARRIOR_WEAPON_CLASS);
    assert_eq!(cudgel.slot, 7);
    assert_eq!(cudgel.min_profile_rank, 1);
    assert!(
        WARRIOR_CREATION_KIT
            .iter()
            .all(|item| item.min_profile_rank == 1)
    );
}

#[test]
fn warrior_level_test_equipment_and_gates_are_recovered() {
    let bands: Vec<(i32, i32)> = WARRIOR_LEVEL_TEST_EQUIPMENT
        .iter()
        .map(|item| (item.id, item.min_profile_rank))
        .collect();
    assert_eq!(
        bands,
        vec![
            (70054, 2),
            (70060, 5),
            (70066, 8),
            (70072, 10),
            (70078, 13),
            (70084, 15),
            (70090, 16),
            (70096, 17),
            (70102, 18),
            (70108, 19),
            (70114, 20),
        ]
    );
    assert!(
        WARRIOR_LEVEL_TEST_EQUIPMENT
            .iter()
            .all(|item| item.class_id == WARRIOR_GEAR_CLASS && item.slot == 3)
    );

    assert_eq!(
        WARRIOR_WEAPON_LEVEL_GATES,
        &[1, 2, 3, 4, 5, 8, 10, 11, 12, 13, 15, 16, 17, 18, 19, 20]
    );
    assert_eq!(
        WARRIOR_GEAR_LEVEL_GATES,
        &[
            1, 2, 3, 4, 5, 6, 8, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20
        ]
    );
}

#[test]
fn warrior_equip_checks_ownership_slot_and_level() {
    // Recovered creation kit item: Student Warrior Cudgel, class 17, slot 7.
    assert_eq!(check_warrior_equip(1, 17, 7, 1), Ok(()));
    assert_eq!(check_warrior_equip(20, 15, 3, 20), Ok(()));

    // Brawler's Mallet of Sweeps (75030) is class 25 and must be rejected.
    assert_eq!(
        check_warrior_equip(20, 25, 7, 1),
        Err(EquipRejection::NotWarriorClass { class_id: 25 })
    );
    assert_eq!(
        check_warrior_equip(20, 71, 7, 1),
        Err(EquipRejection::NotWarriorClass { class_id: 71 })
    );

    // Pro Warrior Hammer (7074) carries recovered MinProfileRank 8.
    assert_eq!(
        check_warrior_equip(7, 17, 7, 8),
        Err(EquipRejection::LevelTooLow {
            required: 8,
            level: 7
        })
    );
    assert_eq!(check_warrior_equip(8, 17, 7, 8), Ok(()));

    // Slot 13 (DiamondRune) is outside the recovered Warrior slot spool.
    assert_eq!(
        check_warrior_equip(20, 15, 13, 1),
        Err(EquipRejection::SlotNotInWarriorSpool { slot: 13 })
    );
}

#[test]
fn warrior_slots_carry_the_recovered_client_names() {
    assert_eq!(warrior_slot(1).expect("head slot").name, "Head");
    assert_eq!(warrior_slot(3).expect("chest slot").name, "Chest");
    assert_eq!(warrior_slot(7).expect("weapon slot").name, "Weapon");
    assert!(warrior_slot(7).expect("weapon slot").is_weapon);
    assert_eq!(warrior_slot(8).expect("accessory slot").name, "Accessory1");
    assert_eq!(warrior_slot(12).expect("rune slot").name, "SquareRune");
    assert_eq!(warrior_slot(1).expect("head slot").min_profile_rank, 0);
    assert!(warrior_slot(9).is_none());
}

#[test]
fn warrior_skills_unlock_at_the_recovered_job_levels() {
    assert_eq!(WARRIOR_SKILLS.len(), 4);

    let kick = warrior_skill(24540).expect("recovered Kick skill");
    assert_eq!(kick.name, "Kick");
    assert_eq!(kick.unlock_level, 1);
    assert_eq!(kick.rank_damage, &[133, 150, 167, 183, 200]);

    let spinning_blade = warrior_skill(4908).expect("recovered Spinning Blade skill");
    assert_eq!(spinning_blade.name, "Spinning Blade");
    assert_eq!(spinning_blade.unlock_level, 5);
    assert_eq!(spinning_blade.rank_damage, &[50, 56, 63, 69, 75]);

    let quake = warrior_skill(24694).expect("recovered Quake skill");
    assert_eq!(quake.name, "Quake");
    assert_eq!(quake.unlock_level, 10);
    assert_eq!(quake.rank_damage, &[83, 94, 104, 115, 125]);

    let shout = warrior_skill(24706).expect("recovered Commanding Shout skill");
    assert_eq!(shout.name, "Commanding Shout");
    assert_eq!(shout.unlock_level, 15);
    assert_eq!(shout.rank_damage, &[50, 56, 63, 69, 75]);

    assert_eq!(unlocked_skill_name_ids(1), vec![24540]);
    assert_eq!(unlocked_skill_name_ids(4), vec![24540]);
    assert_eq!(unlocked_skill_name_ids(5), vec![24540, 4908]);
    assert_eq!(unlocked_skill_name_ids(10), vec![24540, 4908, 24694]);
    assert_eq!(unlocked_skill_name_ids(15), vec![24540, 4908, 24694, 24706]);
    assert_eq!(unlocked_skill_name_ids(20).len(), 4);
    assert!(!skill_unlocked(4, 4908));
    assert!(skill_unlocked(5, 4908));
    assert!(!skill_unlocked(20, 999_999));
}

#[test]
fn warrior_weapon_abilities_are_recovered_locale_facts() {
    assert_eq!(WARRIOR_WEAPON_ABILITIES.len(), 2);

    let sweeping = &WARRIOR_WEAPON_ABILITIES[0];
    assert_eq!(sweeping.name_id, 420252);
    assert_eq!(sweeping.name, "Sweeping Slash");
    assert_eq!(sweeping.tier_text_id, 420253);
    assert_eq!(
        sweeping.tier_text,
        " Warrior Tier 1 - Spammable Ability - Sweeping Slash"
    );

    let spinning = &WARRIOR_WEAPON_ABILITIES[1];
    assert_eq!(spinning.name_id, 442458);
    assert_eq!(spinning.name, "Spinning Attack");
    assert_eq!(spinning.tier_text_id, 420255);
    assert_eq!(
        spinning.tier_text,
        " Warrior Tier - Ultra Ability - Spinning Attack"
    );
}
