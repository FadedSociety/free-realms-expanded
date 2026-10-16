//! Work item 014: recovered blacksmith pattern and equipment restrictions.
//!
//! Recovered original facts, all copied from pinned sources listed in
//! `notes/Research/Pieces/014 Forging recipes/content.md`:
//!
//! * `PATTERNS` are the 28 recoverable `Class 58` records of
//!   `ClientItemDefinitions.json`. `Param1` is the record's own recipe index;
//!   the `Recipe` suffix, `Type 5`, single stack and no-sale flag are the
//!   recovered pattern-item shape. Record 2506 carries recovered description
//!   id 1587, whose locale text names the Spunky Scrapper Hammer as its
//!   product, so the class 58 to class 71 link is not a name guess.
//! * The recovered locale names fix the class meanings used below:
//!   `Blacksmith Shards` (58), `Blacksmith Tools` (34),
//!   `Brawler 1-Handed Hammers` (71) and `Brawler Gear` (25).
//! * `EQUIPMENT` are the recovered hammer/tool definitions the opening
//!   forging path grants or compares against. The ten `30280..=30289`
//!   records are one genuine definition with ten tint variants.
//! * `SLOTS` and `CLASS_OWNERS` come from the supplied client
//!   `EquipmentSlotDefinitions.txt` and the pinned `ItemClasses.txt`.
//! * `PROFILES` merges the pinned `Profiles.json` class allowlists with the
//!   slot spools of the recovered character's own profile records.
//!
//! The rule enforced by `check_equip` is reconstructed policy: a profile may
//! equip a definition only when the recovered class allowlist contains the
//! definition's class and the profile's slot spool carries the definition's
//! slot. Which ten-tint variant a historical forge produced remains unknown.

/// Recovered Blacksmith job profile.
pub const BLACKSMITH: i32 = 16;
/// Recovered Brawler job profile, the class-71 hammer owner.
pub const BRAWLER: i32 = 43;
/// Recovered equipment slot 7, `Weapon`.
pub const SLOT_WEAPON: i32 = 7;
/// Recovered `Class 58` pattern record count.
pub const PATTERN_COUNT: usize = 28;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub id: i32,
    pub name: &'static str,
    pub param1: Option<i32>,
    pub category_id: Option<i32>,
    pub name_id: i32,
    pub description_id: i32,
    pub type_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Equipment {
    pub id: i32,
    pub name: &'static str,
    pub class_id: i32,
    pub slot: i32,
    pub category_id: i32,
    pub model: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: i32,
    pub name: &'static str,
    pub is_weapon: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassOwner {
    pub class_id: i32,
    pub profile_name_id: i32,
    pub profile_id: i32,
    pub wield_type: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProfileRules {
    pub profile_id: i32,
    pub name_id: i32,
    pub name: &'static str,
    pub allowed_classes: &'static [i32],
    pub slots: &'static [i32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipRejection {
    UnknownDefinition,
    UnknownProfile,
    ClassNotAllowed { class_id: i32 },
    SlotNotAllowed { slot: i32 },
}

pub const PATTERNS: &[Pattern] = &[
    Pattern {
        id: 838,
        name: "Anata's Tempered Circle Shard",
        param1: None,
        category_id: Some(124),
        name_id: 3409,
        description_id: 3419,
        type_id: 1,
    },
    Pattern {
        id: 869,
        name: "Chocolate Bumbleberry Mousse Recipe",
        param1: Some(59),
        category_id: Some(17),
        name_id: 1536,
        description_id: 1563,
        type_id: 5,
    },
    Pattern {
        id: 2505,
        name: "Quick Shot Bow Recipe",
        param1: Some(10),
        category_id: Some(17),
        name_id: 1564,
        description_id: 1586,
        type_id: 5,
    },
    Pattern {
        id: 2506,
        name: "Spunky Scrapper Hammer Recipe",
        param1: Some(11),
        category_id: Some(17),
        name_id: 1565,
        description_id: 1587,
        type_id: 5,
    },
    Pattern {
        id: 2507,
        name: "First Aid Scalpel Recipe",
        param1: Some(12),
        category_id: Some(17),
        name_id: 1566,
        description_id: 1588,
        type_id: 5,
    },
    Pattern {
        id: 2508,
        name: "Dancing Monkey Bokken Recipe",
        param1: Some(13),
        category_id: Some(17),
        name_id: 1567,
        description_id: 1589,
        type_id: 5,
    },
    Pattern {
        id: 2509,
        name: "Power Charmed Wand Recipe",
        param1: Some(14),
        category_id: Some(17),
        name_id: 1568,
        description_id: 1590,
        type_id: 5,
    },
    Pattern {
        id: 2510,
        name: "Fast Blast Bow Recipe",
        param1: Some(15),
        category_id: Some(17),
        name_id: 1569,
        description_id: 1591,
        type_id: 5,
    },
    Pattern {
        id: 2512,
        name: "Field Support Hammer Recipe",
        param1: Some(17),
        category_id: Some(17),
        name_id: 1571,
        description_id: 1593,
        type_id: 5,
    },
    Pattern {
        id: 2513,
        name: "Biting Mongoose Blade Recipe",
        param1: Some(18),
        category_id: Some(17),
        name_id: 1572,
        description_id: 1594,
        type_id: 5,
    },
    Pattern {
        id: 2514,
        name: "Lunar Enchanted Wand Recipe",
        param1: Some(19),
        category_id: Some(17),
        name_id: 1573,
        description_id: 1595,
        type_id: 5,
    },
    Pattern {
        id: 2515,
        name: "Alley Fighter Hammer Recipe",
        param1: Some(20),
        category_id: Some(17),
        name_id: 1574,
        description_id: 1596,
        type_id: 5,
    },
    Pattern {
        id: 2516,
        name: "Anguish Aid Bonesaw Recipe",
        param1: Some(21),
        category_id: Some(17),
        name_id: 1575,
        description_id: 1597,
        type_id: 5,
    },
    Pattern {
        id: 2517,
        name: "Diving Hawk Scythe Recipe",
        param1: Some(22),
        category_id: Some(17),
        name_id: 1576,
        description_id: 1598,
        type_id: 5,
    },
    Pattern {
        id: 2519,
        name: "Strong Scrapper Hammer Recipe",
        param1: Some(24),
        category_id: Some(17),
        name_id: 1578,
        description_id: 1600,
        type_id: 5,
    },
    Pattern {
        id: 2520,
        name: "Pain Relief Bonesaw Recipe",
        param1: Some(25),
        category_id: Some(17),
        name_id: 1579,
        description_id: 1601,
        type_id: 5,
    },
    Pattern {
        id: 2521,
        name: "Stalking Panther Scythe Recipe",
        param1: Some(26),
        category_id: Some(17),
        name_id: 1580,
        description_id: 1602,
        type_id: 5,
    },
    Pattern {
        id: 2522,
        name: "Stalwart Protector Hammer Recipe",
        param1: Some(27),
        category_id: Some(17),
        name_id: 1581,
        description_id: 1603,
        type_id: 5,
    },
    Pattern {
        id: 2523,
        name: "Frenzy Fighter Hammer Recipe",
        param1: Some(28),
        category_id: Some(17),
        name_id: 1582,
        description_id: 1604,
        type_id: 5,
    },
    Pattern {
        id: 2524,
        name: "Trauma Support Shockrod Recipe",
        param1: Some(29),
        category_id: Some(17),
        name_id: 1583,
        description_id: 1605,
        type_id: 5,
    },
    Pattern {
        id: 2525,
        name: "Striking Serpent Sword Recipe",
        param1: Some(30),
        category_id: Some(17),
        name_id: 1584,
        description_id: 1606,
        type_id: 5,
    },
    Pattern {
        id: 2526,
        name: "Intrepid Protector Axe Recipe",
        param1: Some(31),
        category_id: Some(17),
        name_id: 1585,
        description_id: 1607,
        type_id: 5,
    },
    Pattern {
        id: 23872,
        name: "Dust Storm Flair Shard Recipe",
        param1: Some(155),
        category_id: Some(17),
        name_id: 26636,
        description_id: 26663,
        type_id: 5,
    },
    Pattern {
        id: 23882,
        name: "Manguana Sunstone Recipe",
        param1: Some(165),
        category_id: Some(17),
        name_id: 26646,
        description_id: 26673,
        type_id: 5,
    },
    Pattern {
        id: 49382,
        name: "Savage Sting Bow Recipe",
        param1: Some(65),
        category_id: Some(17),
        name_id: 384403,
        description_id: 384400,
        type_id: 5,
    },
    Pattern {
        id: 49383,
        name: "Brutal Blast Bow Recipe",
        param1: Some(67),
        category_id: Some(17),
        name_id: 384404,
        description_id: 384401,
        type_id: 5,
    },
    Pattern {
        id: 49384,
        name: "Void Woven Wand Recipe",
        param1: Some(68),
        category_id: Some(17),
        name_id: 384405,
        description_id: 384402,
        type_id: 5,
    },
    Pattern {
        id: 49385,
        name: "Eclipse Infused Wand Recipe",
        param1: Some(64),
        category_id: Some(17),
        name_id: 384393,
        description_id: 384392,
        type_id: 5,
    },
];

pub const EQUIPMENT: &[Equipment] = &[
    Equipment {
        id: 30280,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30281,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30282,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30283,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30284,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30285,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30286,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30287,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30288,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 30289,
        name: "Spunky Scrapper Hammer",
        class_id: 71,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 75030,
        name: "Brawler's Mallet of Sweeps",
        class_id: 25,
        slot: 7,
        category_id: 207,
        model: "hammer_ar_ag_weapon_logjammer.adr",
    },
    Equipment {
        id: 6459,
        name: "Student Blacksmith Hammer",
        class_id: 34,
        slot: 7,
        category_id: 107,
        model: "tool_ar_ag_weapon_hammer.adr",
    },
];

pub const SLOTS: &[Slot] = &[Slot {
    id: 7,
    name: "Weapon",
    is_weapon: true,
}];

pub const CLASS_OWNERS: &[ClassOwner] = &[
    ClassOwner {
        class_id: 25,
        profile_name_id: 2873,
        profile_id: 43,
        wield_type: 0,
    },
    ClassOwner {
        class_id: 34,
        profile_name_id: 2614,
        profile_id: 16,
        wield_type: 0,
    },
    ClassOwner {
        class_id: 58,
        profile_name_id: 2614,
        profile_id: 16,
        wield_type: 0,
    },
    ClassOwner {
        class_id: 71,
        profile_name_id: 2873,
        profile_id: 43,
        wield_type: 13,
    },
];

pub const PROFILES: &[ProfileRules] = &[
    ProfileRules {
        profile_id: 16,
        name_id: 2614,
        name: "Blacksmith",
        allowed_classes: &[
            14, 34, 48, 49, 58, 60, 67, 69, 144, 145, 146, 150, 153, 154, 155, 156, 158, 161, 162,
            168, 173, 999,
        ],
        slots: &[1, 2, 3, 4, 5, 7, 13],
    },
    ProfileRules {
        profile_id: 43,
        name_id: 2873,
        name: "Brawler",
        allowed_classes: &[
            5, 25, 42, 50, 52, 69, 71, 144, 145, 146, 150, 152, 153, 154, 155, 156, 158, 161, 162,
            168, 173, 181, 182, 183, 999,
        ],
        slots: &[1, 2, 3, 4, 5, 7, 8, 12, 6],
    },
];

pub fn patterns() -> &'static [Pattern] {
    PATTERNS
}

pub fn pattern(id: i32) -> Option<&'static Pattern> {
    PATTERNS.iter().find(|p| p.id == id)
}

pub fn equipment() -> &'static [Equipment] {
    EQUIPMENT
}

pub fn equipment_by_id(id: i32) -> Option<&'static Equipment> {
    EQUIPMENT.iter().find(|e| e.id == id)
}

pub fn slot(id: i32) -> Option<&'static Slot> {
    SLOTS.iter().find(|s| s.id == id)
}

pub fn slot_count() -> usize {
    SLOTS.len()
}

pub fn class_owner(class_id: i32) -> Option<&'static ClassOwner> {
    CLASS_OWNERS.iter().find(|c| c.class_id == class_id)
}

pub fn profile(profile_id: i32) -> Option<&'static ProfileRules> {
    PROFILES.iter().find(|p| p.profile_id == profile_id)
}

/// Reconstructed policy: the class allowlist and the slot spool must both
/// admit the definition.
pub fn allows(rules: &ProfileRules, definition: &Equipment) -> Result<(), EquipRejection> {
    if !rules.allowed_classes.contains(&definition.class_id) {
        return Err(EquipRejection::ClassNotAllowed {
            class_id: definition.class_id,
        });
    }
    if !rules.slots.contains(&definition.slot) {
        return Err(EquipRejection::SlotNotAllowed {
            slot: definition.slot,
        });
    }
    Ok(())
}

/// Reconstructed policy: only a recovered profile whose class allowlist and
/// slot spool both admit the definition may equip it.
pub fn check_equip(
    profile_id: i32,
    definition_id: i32,
) -> Result<&'static Equipment, EquipRejection> {
    let definition = equipment_by_id(definition_id).ok_or(EquipRejection::UnknownDefinition)?;
    let rules = profile(profile_id).ok_or(EquipRejection::UnknownProfile)?;
    allows(rules, definition)?;
    Ok(definition)
}
