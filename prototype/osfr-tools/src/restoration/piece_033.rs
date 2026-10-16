//! Work item 033: recovered Warrior job, skill, ownership and level restrictions.
//!
//! Recovered original facts, all copied from pinned sources listed in
//! `notes/Research/Pieces/033 Warrior progression.md`:
//!
//! * `WARRIOR_PROFILE_ID` and the header constants are the pinned Legacy
//!   `Profiles.json` record 32, which is also what the prepared profile packet
//!   in `references/rust-opening/profiles.json` serializes.
//! * `WARRIOR_CLASSES` are the four `ItemClasses.txt` rows whose owner name id
//!   is 4664: `Warrior Gear` (15), `Warrior Axes & Hammers` (17),
//!   `Warrior Jewelry` (45) and `Warrior Shards` (55). Brawler classes 25 and
//!   71 are owned by name id 2873 and are deliberately absent.
//! * `WARRIOR_CREATION_KIT` is the recovered `DefaultItems` list of profile 32
//!   resolved against `ClientItemDefinitions.json`.
//! * `WARRIOR_LEVEL_TEST_EQUIPMENT`, the two level-gate ladders and the slot
//!   names come from the same pinned item table and the supplied client
//!   `EquipmentSlotDefinitions.txt`.
//! * `WARRIOR_SKILLS` names and rank damage ladders are recovered from the
//!   preserved `locale-id-to-text.json` map (rank text ids 24541/24544/24546/
//!   24548/24550, 6163/47414/47415/47416/47417, 24695/24697/24699/24701/24705,
//!   24708/24710/24712/24714/24716). The unlock levels are the community wiki
//!   Warrior table, corroborated by the recovered "Achieve Level 5/10/15 as a
//!   Warrior" and "Warrior: Achieve Level 5/10/15/20" locale text.
//! * `WARRIOR_WEAPON_ABILITIES` are recovered locale names and tier text.
//!
//! The rule enforced by `check_warrior_equip` is reconstructed policy: an item
//! may be equipped by the Warrior profile only when its class is Warrior-owned,
//! its slot is in the recovered Warrior slot spool, and both the item and slot
//! level gates are at or below the current job level.

/// Recovered Warrior profile id.
pub const WARRIOR_PROFILE_ID: i32 = 32;
/// Recovered Warrior profile `NameId`; locale 4664 resolves to `Warrior`.
pub const WARRIOR_NAME_ID: i32 = 4664;
/// Recovered Warrior profile `DescriptionId`; locale 4665 is the job blurb.
pub const WARRIOR_DESCRIPTION_ID: i32 = 4665;
/// Recovered Warrior profile `Hint`; locale 384191 is the job hint.
pub const WARRIOR_HINT_ID: i32 = 384191;
/// Recovered Warrior profile icon id.
pub const WARRIOR_ICON_ID: i32 = 21594;
/// Recovered Warrior profile `Type`; 2 is the combat job type.
pub const WARRIOR_PROFILE_TYPE: i32 = 2;
/// Recovered Warrior `BadgeImageSet`.
pub const WARRIOR_BADGE_IMAGE_SET: i32 = 10;
/// Recovered Warrior `ButtonImageSet`.
pub const WARRIOR_BUTTON_IMAGE_SET: i32 = 1;
/// Recovered Warrior `AbilityBgImageSet`.
pub const WARRIOR_ABILITY_BG_IMAGE_SET: i32 = 31;
/// Recovered Warrior `MembersOnly` flag.
pub const WARRIOR_MEMBERS_ONLY: bool = false;
/// Recovered Warrior `Trial` flag.
pub const WARRIOR_TRIAL: bool = true;

/// Recovered job name from locale 4664.
pub const WARRIOR_JOB_NAME: &str = "Warrior";
/// Recovered job description from locale 4665.
pub const WARRIOR_JOB_DESCRIPTION: &str =
    "Warriors are hardy Brawlers that can take and dish out damage.";
/// Recovered job hint from locale 384191.
pub const WARRIOR_JOB_HINT: &str = "A true warrior does not fear harsh weather.";

/// Recovered `Warrior Gear` class id (armour).
pub const WARRIOR_GEAR_CLASS: i32 = 15;
/// Recovered `Warrior Axes & Hammers` class id (weapons).
pub const WARRIOR_WEAPON_CLASS: i32 = 17;
/// Recovered `Warrior Jewelry` class id.
pub const WARRIOR_JEWELRY_CLASS: i32 = 45;
/// Recovered `Warrior Shards` class id.
pub const WARRIOR_SHARD_CLASS: i32 = 55;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemClass {
    pub id: i32,
    pub name: &'static str,
    pub name_id: i32,
    pub icon_id: i32,
    pub wield_type: i32,
    pub stat_id: i32,
    pub owner_name_id: i32,
}

/// The four recovered `ItemClasses.txt` rows whose owner name id is 4664.
pub const WARRIOR_CLASSES: &[ItemClass] = &[
    ItemClass {
        id: 15,
        name: "Warrior Gear",
        name_id: 2935,
        icon_id: 497,
        wield_type: 0,
        stat_id: 16,
        owner_name_id: 4664,
    },
    ItemClass {
        id: 17,
        name: "Warrior Axes & Hammers",
        name_id: 2937,
        icon_id: 498,
        wield_type: 12,
        stat_id: 19,
        owner_name_id: 4664,
    },
    ItemClass {
        id: 45,
        name: "Warrior Jewelry",
        name_id: 2250,
        icon_id: 55,
        wield_type: 12,
        stat_id: 0,
        owner_name_id: 4664,
    },
    ItemClass {
        id: 55,
        name: "Warrior Shards",
        name_id: 2318,
        icon_id: 55,
        wield_type: 12,
        stat_id: 0,
        owner_name_id: 4664,
    },
];

/// The recovered class allowlist of the profile 32 packet.
pub const WARRIOR_CLASS_ALLOWLIST: &[i32] = &[
    15, 17, 45, 50, 55, 69, 144, 145, 146, 150, 151, 153, 154, 155, 156, 158, 161, 162, 168, 173,
    183, 999,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WarriorEquipment {
    pub id: i32,
    pub name: &'static str,
    pub class_id: i32,
    pub slot: i32,
    pub min_profile_rank: i32,
}

/// The recovered `DefaultItems` of profile 32, resolved to item records.
pub const WARRIOR_CREATION_KIT: &[WarriorEquipment] = &[
    WarriorEquipment {
        id: 46644,
        name: "Student Warrior Helm",
        class_id: 15,
        slot: 1,
        min_profile_rank: 1,
    },
    WarriorEquipment {
        id: 46389,
        name: "Student Warrior Gauntlets",
        class_id: 15,
        slot: 2,
        min_profile_rank: 1,
    },
    WarriorEquipment {
        id: 46137,
        name: "Student Warrior Tunic",
        class_id: 15,
        slot: 3,
        min_profile_rank: 1,
    },
    WarriorEquipment {
        id: 46732,
        name: "Student Warrior Breeches",
        class_id: 15,
        slot: 4,
        min_profile_rank: 1,
    },
    WarriorEquipment {
        id: 46224,
        name: "Student Warrior Hightops",
        class_id: 15,
        slot: 5,
        min_profile_rank: 1,
    },
    WarriorEquipment {
        id: 47019,
        name: "Student Warrior Pauldrons",
        class_id: 15,
        slot: 6,
        min_profile_rank: 1,
    },
    WarriorEquipment {
        id: 29963,
        name: "Student Warrior Cudgel",
        class_id: 17,
        slot: 7,
        min_profile_rank: 1,
    },
];

/// The recovered `Level N Warrior Test Chest` family; each record carries the
/// level it tests in `MinProfileRank`.
pub const WARRIOR_LEVEL_TEST_EQUIPMENT: &[WarriorEquipment] = &[
    WarriorEquipment {
        id: 70054,
        name: "Level 2 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 2,
    },
    WarriorEquipment {
        id: 70060,
        name: "Level 5 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 5,
    },
    WarriorEquipment {
        id: 70066,
        name: "Level 8 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 8,
    },
    WarriorEquipment {
        id: 70072,
        name: "Level 10 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 10,
    },
    WarriorEquipment {
        id: 70078,
        name: "Level 13 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 13,
    },
    WarriorEquipment {
        id: 70084,
        name: "Level 15 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 15,
    },
    WarriorEquipment {
        id: 70090,
        name: "Level 16 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 16,
    },
    WarriorEquipment {
        id: 70096,
        name: "Level 17 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 17,
    },
    WarriorEquipment {
        id: 70102,
        name: "Level 18 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 18,
    },
    WarriorEquipment {
        id: 70108,
        name: "Level 19 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 19,
    },
    WarriorEquipment {
        id: 70114,
        name: "Level 20 Warrior Test Chest",
        class_id: 15,
        slot: 3,
        min_profile_rank: 20,
    },
];

/// Distinct recovered `MinProfileRank` values across `Warrior Axes & Hammers`
/// (class 17) records.
pub const WARRIOR_WEAPON_LEVEL_GATES: &[i32] =
    &[1, 2, 3, 4, 5, 8, 10, 11, 12, 13, 15, 16, 17, 18, 19, 20];
/// Distinct recovered `MinProfileRank` values across `Warrior Gear` (class 15)
/// records.
pub const WARRIOR_GEAR_LEVEL_GATES: &[i32] = &[
    1, 2, 3, 4, 5, 6, 8, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotRule {
    pub slot: i32,
    pub name: &'static str,
    pub min_profile_rank: i32,
    pub is_weapon: bool,
}

/// The recovered Warrior slot spool `[1, 2, 3, 4, 5, 6, 7, 8, 12]` resolved
/// against the supplied client `EquipmentSlotDefinitions.txt`.
pub const WARRIOR_SLOTS: &[SlotRule] = &[
    SlotRule {
        slot: 1,
        name: "Head",
        min_profile_rank: 0,
        is_weapon: false,
    },
    SlotRule {
        slot: 2,
        name: "Hands",
        min_profile_rank: 0,
        is_weapon: false,
    },
    SlotRule {
        slot: 3,
        name: "Chest",
        min_profile_rank: 0,
        is_weapon: false,
    },
    SlotRule {
        slot: 4,
        name: "Legs",
        min_profile_rank: 0,
        is_weapon: false,
    },
    SlotRule {
        slot: 5,
        name: "Feet",
        min_profile_rank: 0,
        is_weapon: false,
    },
    SlotRule {
        slot: 6,
        name: "Shoulders",
        min_profile_rank: 0,
        is_weapon: false,
    },
    SlotRule {
        slot: 7,
        name: "Weapon",
        min_profile_rank: 0,
        is_weapon: true,
    },
    SlotRule {
        slot: 8,
        name: "Accessory1",
        min_profile_rank: 0,
        is_weapon: false,
    },
    SlotRule {
        slot: 12,
        name: "SquareRune",
        min_profile_rank: 0,
        is_weapon: false,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skill {
    pub name_id: i32,
    pub name: &'static str,
    pub unlock_level: i32,
    pub rank_damage: &'static [i32],
}

/// Recovered Warrior skills. Names and rank ladders are original locale facts;
/// the unlock levels are the community wiki table, separate from the ranks.
pub const WARRIOR_SKILLS: &[Skill] = &[
    Skill {
        name_id: 24540,
        name: "Kick",
        unlock_level: 1,
        rank_damage: &[133, 150, 167, 183, 200],
    },
    Skill {
        name_id: 4908,
        name: "Spinning Blade",
        unlock_level: 5,
        rank_damage: &[50, 56, 63, 69, 75],
    },
    Skill {
        name_id: 24694,
        name: "Quake",
        unlock_level: 10,
        rank_damage: &[83, 94, 104, 115, 125],
    },
    Skill {
        name_id: 24706,
        name: "Commanding Shout",
        unlock_level: 15,
        rank_damage: &[50, 56, 63, 69, 75],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponAbility {
    pub name_id: i32,
    pub name: &'static str,
    pub tier_text_id: i32,
    pub tier_text: &'static str,
}

/// Recovered Warrior weapon ability names and tier text from locale.
pub const WARRIOR_WEAPON_ABILITIES: &[WeaponAbility] = &[
    WeaponAbility {
        name_id: 420252,
        name: "Sweeping Slash",
        tier_text_id: 420253,
        tier_text: " Warrior Tier 1 - Spammable Ability - Sweeping Slash",
    },
    WeaponAbility {
        name_id: 442458,
        name: "Spinning Attack",
        tier_text_id: 420255,
        tier_text: " Warrior Tier - Ultra Ability - Spinning Attack",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipRejection {
    NotWarriorClass { class_id: i32 },
    SlotNotInWarriorSpool { slot: i32 },
    LevelTooLow { required: i32, level: i32 },
}

pub fn warrior_class(class_id: i32) -> Option<&'static ItemClass> {
    WARRIOR_CLASSES.iter().find(|record| record.id == class_id)
}

pub fn warrior_slot(slot: i32) -> Option<&'static SlotRule> {
    WARRIOR_SLOTS.iter().find(|record| record.slot == slot)
}

pub fn warrior_skill(name_id: i32) -> Option<&'static Skill> {
    WARRIOR_SKILLS
        .iter()
        .find(|record| record.name_id == name_id)
}

/// A skill is available when the Warrior job level reached its unlock level.
pub fn skill_unlocked(job_level: i32, name_id: i32) -> bool {
    warrior_skill(name_id).is_some_and(|skill| job_level >= skill.unlock_level)
}

/// The recovered skill name ids available at a job level, in table order.
pub fn unlocked_skill_name_ids(job_level: i32) -> Vec<i32> {
    WARRIOR_SKILLS
        .iter()
        .filter(|skill| job_level >= skill.unlock_level)
        .map(|skill| skill.name_id)
        .collect()
}

/// Reconstructed Warrior equip policy over the recovered restrictions:
/// Warrior-owned class, Warrior slot spool, and both level gates satisfied.
pub fn check_warrior_equip(
    job_level: i32,
    class_id: i32,
    slot: i32,
    item_min_profile_rank: i32,
) -> Result<(), EquipRejection> {
    if warrior_class(class_id).is_none() {
        return Err(EquipRejection::NotWarriorClass { class_id });
    }
    let Some(slot_rule) = warrior_slot(slot) else {
        return Err(EquipRejection::SlotNotInWarriorSpool { slot });
    };
    let required = item_min_profile_rank.max(slot_rule.min_profile_rank);
    if job_level < required {
        return Err(EquipRejection::LevelTooLow {
            required,
            level: job_level,
        });
    }
    Ok(())
}
