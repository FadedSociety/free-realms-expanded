//! Work item 047: recovered gear attributes and item profile restrictions.
//!
//! Recovered original facts, copied from the pinned sources listed in
//! `notes/Research/Pieces/047 Gear attributes.md`:
//!
//! * `CHARACTER_STAT_IDS` is the recovered character stat id space: 69 ordered values,
//!   ids 0..68, from `CharacterStatId.cs`. `InvalidStat` (0) is the invalid default and
//!   has no row in `CharacterStats.cs`; every other id has one default literal, and that
//!   literal (`0` versus `0f`) is the recovered int/float wire kind of the stat.
//! * `INT_WIRE_TYPE`/`FLOAT_WIRE_TYPE` are the recovered `BaseItemStatDefinition::Type`
//!   codes: the item stat serializer writes a 32 bit int for type 0 and a 32 bit float for
//!   type 1, then the `Unknown` bool. The community class comment calls those types
//!   "additive" and "multiplicative"; that meaning is reconstruction commentary, so the
//!   constants here only name the serialization behaviour that the code shows.
//! * `GEAR_STAT_ITEMS` are the only two item rows in the pinned client item table with a
//!   `Stats.<key>.Id`/`Stats.<key>.Value` vector. Their values are IEEE-754 float32 bit
//!   patterns; the key equals the stat id in every recovered row.
//! * Item 78397 The Fire Hawk carries `Param1 = 13459`, the recovered mount row id whose
//!   glide fields carry the same eight values, so that vector has an independent second
//!   source. The parachute vector has one source and only the internal glide ordering
//!   check in this piece.
//! * `GEAR_CLASSES` are recovered `ItemClasses.txt` rows
//!   (`Id^NameId^IconId^WieldType^StatId^ProfileNameId`, field names from
//!   `ItemClassDefinition.cs`), resolved against the preserved locale export. A class
//!   whose profile name id names a job profile is owned by that profile; `All Jobs`,
//!   `Any Combat Job` and `Any Job` are class scopes without a profile row, so
//!   `owner_profile` stays `None` for them rather than guessing a restriction.
//! * `NATIVE_ITEM_STAT_CONTAINER_CAPACITY` is the container size declared by the native
//!   client pseudocode (`HashListMap<int,ItemStatDefinition,71,-1>`). It is two slots wider
//!   than the recovered id space; which ids, if any, fill those slots is not recovered.
//!
//! `equip_gate` is reconstructed policy over those recovered rows: class ownership, the
//! recovered `MinProfileRank` gate and the recovered `MembersOnly` flag. Nothing here
//! grants items, coins or experience, and no native acceptance is claimed.

/// Recovered number of character stat ids (0..68).
pub const CHARACTER_STAT_ID_COUNT: usize = 69;

/// Recovered item stat wire type that serializes a 32 bit integer value.
pub const INT_WIRE_TYPE: i32 = 0;

/// Recovered item stat wire type that serializes a 32 bit float value.
pub const FLOAT_WIRE_TYPE: i32 = 1;

/// Native `HashListMap<int,ItemStatDefinition,71,-1>` capacity, two above the recovered ids.
pub const NATIVE_ITEM_STAT_CONTAINER_CAPACITY: usize = 71;

/// Recovered parachute item row, one of the two rows carrying an item stat vector.
pub const PARACHUTE_ITEM_ID: i32 = 18092;

/// Recovered Fire Hawk item row, the vector with an independent mount row.
pub const FIRE_HAWK_ITEM_ID: i32 = 78397;

/// Recovered Fire Hawk mount row (`Mounts.json`), bound by the item's `Param1`.
pub const FIRE_HAWK_MOUNT_ID: i32 = 13459;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatKind {
    Int,
    Float,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StatIdRow {
    pub id: u32,
    pub name: &'static str,
    pub kind: StatKind,
}

/// Recovered `CharacterStatId` order with the `CharacterStats` default literal kind.
pub const CHARACTER_STAT_IDS: &[StatIdRow] = &[
    StatIdRow {
        id: 0,
        name: "InvalidStat",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 1,
        name: "MaxHealth",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 2,
        name: "MaxMovementSpeed",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 3,
        name: "WeaponRange",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 4,
        name: "HitPointRegen",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 5,
        name: "MaxMana",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 6,
        name: "ManaRegen",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 7,
        name: "Defense",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 8,
        name: "MeleeAvoidance",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 9,
        name: "MeleeCriticalHitChance",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 10,
        name: "MeleeCriticalHitMultiplier",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 11,
        name: "MeleeChanceToHit",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 12,
        name: "MeleeWeaponDamageMultiplier",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 13,
        name: "MeleeHandToHandDamage",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 14,
        name: "EquippedMeleeWeaponDamage",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 15,
        name: "MeleeAttackIntervalMs",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 16,
        name: "DamageReductionAmount",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 17,
        name: "ExperienceBoostPercent",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 18,
        name: "DamageReductionPercent",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 19,
        name: "DamageAddition",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 20,
        name: "DamageMultiplier",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 21,
        name: "HealingAddition",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 22,
        name: "HealingMultiplier",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 23,
        name: "M3CollectableSpawnRate",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 24,
        name: "M3SpecialSpawnRate",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 25,
        name: "M3DetrementalSpawnRate",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 26,
        name: "M3Magnitude",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 27,
        name: "M3Timer",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 28,
        name: "MimicRadius",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 29,
        name: "MimicCuttingMagnitude",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 30,
        name: "MimicPowerMagnitude",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 31,
        name: "MimicGreenRange",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 32,
        name: "MimicSpeed",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 33,
        name: "AbilityCriticalHitChance",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 34,
        name: "AbilityCriticalHitMultiplier",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 35,
        name: "Luck",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 36,
        name: "HeadInflationPercent",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 37,
        name: "GoldBoostPercent",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 38,
        name: "M3PerMatchProc",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 39,
        name: "SoccerKickPower",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 40,
        name: "SoccerFootwork",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 41,
        name: "SoccerSpeed",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 42,
        name: "SoccerToughness",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 43,
        name: "SoccerTacklePower",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 44,
        name: "FishingCastingSkill",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 45,
        name: "FishingCastingStrength",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 46,
        name: "FishingLineStrength",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 47,
        name: "FishingReelingSpeed",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 48,
        name: "FishingLuck",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 49,
        name: "FishingPerfectCastSkill",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 50,
        name: "Toughness",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 51,
        name: "AbilityCritVulnerability",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 52,
        name: "MeleeCritVulnerability",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 53,
        name: "RangeMultiplier",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 54,
        name: "MaxShields",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 55,
        name: "ShieldsRegen",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 56,
        name: "FactoryProductionModifier",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 57,
        name: "FactoryYieldModifier",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 58,
        name: "PlayerCastIllusionImmunity",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 59,
        name: "GlideDefaultForwardSpeed",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 60,
        name: "GlideMinForwardSpeed",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 61,
        name: "GlideMaxForwardSpeed",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 62,
        name: "GlideFallTime",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 63,
        name: "GlideFallSpeed",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 64,
        name: "GlideEnabled",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 65,
        name: "InCombatHitPointRegen",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 66,
        name: "InCombatManaRegen",
        kind: StatKind::Int,
    },
    StatIdRow {
        id: 67,
        name: "GlideAccel",
        kind: StatKind::Float,
    },
    StatIdRow {
        id: 68,
        name: "JumpHeight",
        kind: StatKind::Float,
    },
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecodedItemStat {
    pub id: u32,
    pub raw_bits: u32,
    pub value: f32,
    pub kind: StatKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GearAttributeError {
    UnknownStat(u32),
    NotFinite(u32),
    NonIntegralIntStat { id: u32, value: f32 },
}

pub fn stat_row(id: u32) -> Option<&'static StatIdRow> {
    CHARACTER_STAT_IDS.get(id as usize)
}

/// Decodes one recovered item stat value.
///
/// The pinned item table stores the value as a 32 bit pattern, and the item stat
/// serializer writes that pattern as a float when the definition type is 1. An int kind
/// stat therefore has to decode to an integral float; anything else means the pattern was
/// read or written with the wrong width.
pub fn decode_item_stat(id: u32, raw_bits: u32) -> Result<DecodedItemStat, GearAttributeError> {
    let row = stat_row(id).ok_or(GearAttributeError::UnknownStat(id))?;
    let value = f32::from_bits(raw_bits);
    if !value.is_finite() {
        return Err(GearAttributeError::NotFinite(id));
    }
    if row.kind == StatKind::Int && value.fract() != 0.0 {
        return Err(GearAttributeError::NonIntegralIntStat { id, value });
    }
    Ok(DecodedItemStat {
        id,
        raw_bits,
        value,
        kind: row.kind,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GearClassRow {
    pub class_id: i32,
    pub name_id: i32,
    pub name: &'static str,
    pub wield_type: i32,
    pub stat_id: i32,
    pub profile_name_id: i32,
    pub profile_name: &'static str,
    /// Owning job profile id when the class profile name id names a job profile.
    pub owner_profile: Option<i32>,
}

/// Recovered `ItemClasses.txt` rows for the item classes this piece needs.
pub const GEAR_CLASSES: &[GearClassRow] = &[
    GearClassRow {
        class_id: 2,
        name_id: 2922,
        name: "Ninja Gear",
        wield_type: 0,
        stat_id: 16,
        profile_name_id: 3100,
        profile_name: "Ninja",
        owner_profile: Some(2),
    },
    GearClassRow {
        class_id: 50,
        name_id: 2235,
        name: "All Combat Job Item Job",
        wield_type: 0,
        stat_id: 0,
        profile_name_id: 17372,
        profile_name: "Any Combat Job",
        owner_profile: None,
    },
    GearClassRow {
        class_id: 69,
        name_id: 2610,
        name: "All Job Shards",
        wield_type: 0,
        stat_id: 0,
        profile_name_id: 17373,
        profile_name: "Any Job",
        owner_profile: None,
    },
    GearClassRow {
        class_id: 158,
        name_id: 440560,
        name: "Parachute",
        wield_type: 27,
        stat_id: 16,
        profile_name_id: 113125,
        profile_name: "All Jobs",
        owner_profile: None,
    },
    GearClassRow {
        class_id: 172,
        name_id: 5101171,
        name: "Ride",
        wield_type: 0,
        stat_id: 0,
        profile_name_id: 5101171,
        profile_name: "Ride",
        owner_profile: None,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GearStatItem {
    pub id: i32,
    pub comment: &'static str,
    pub class_id: i32,
    /// Recovered `Slot`; `None` when the pinned row has no `Slot` field.
    pub slot: Option<i32>,
    /// Recovered `MinProfileRank`; `None` when the pinned row has no such field.
    pub min_profile_rank: Option<u8>,
    /// Recovered `MembersOnly` flag; `false` includes the absent field.
    pub members_only: bool,
    /// Recovered `Param1`, which binds the Fire Hawk item to its mount row.
    pub param1: Option<i32>,
    /// `(stat id, stored 32 bit value)` pairs in recovered `Stats.<key>` order.
    pub raw_stats: &'static [(u32, u32)],
}

/// The two recovered item rows carrying an item stat vector.
pub const GEAR_STAT_ITEMS: &[GearStatItem] = &[
    GearStatItem {
        id: 18092,
        comment: "Parachute",
        class_id: 158,
        slot: Some(6),
        min_profile_rank: None,
        members_only: false,
        param1: None,
        raw_stats: &[
            (59, 1090519040),
            (60, 1073741824),
            (61, 1099956224),
            (62, 1061158912),
            (63, 1082130432),
            (64, 1065353216),
            (67, 1082130432),
        ],
    },
    GearStatItem {
        id: 78397,
        comment: "The Fire Hawk",
        class_id: 172,
        slot: None,
        min_profile_rank: None,
        members_only: false,
        param1: Some(13459),
        raw_stats: &[
            (59, 1094713344),
            (60, 1086324736),
            (61, 1102053376),
            (62, 1061158912),
            (63, 1077936128),
            (64, 1065353216),
            (67, 1082130432),
            (68, 1084227584),
        ],
    },
];

pub fn gear_class(class_id: i32) -> Option<&'static GearClassRow> {
    GEAR_CLASSES.iter().find(|row| row.class_id == class_id)
}

pub fn gear_stat_item(id: i32) -> Option<&'static GearStatItem> {
    GEAR_STAT_ITEMS.iter().find(|row| row.id == id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipDecision {
    Allowed,
    UnknownClass,
    WrongProfile,
    RankTooLow,
    MembersOnly,
}

/// Reconstructed equip policy over the recovered restriction rows.
///
/// The class's owning profile must match the requesting profile, the recovered
/// `MinProfileRank` gate must be met, and a recovered `MembersOnly` item needs an active
/// membership. Classes whose profile name id is a shared scope have no owner and stay
/// open; classes with no recovered row are refused instead of assumed usable.
pub fn equip_gate(
    class: Option<&GearClassRow>,
    min_profile_rank: Option<u8>,
    members_only: bool,
    profile_id: i32,
    profile_rank: u8,
    member: bool,
) -> EquipDecision {
    let Some(class) = class else {
        return EquipDecision::UnknownClass;
    };
    if let Some(owner) = class.owner_profile {
        if owner != profile_id {
            return EquipDecision::WrongProfile;
        }
    }
    if let Some(required) = min_profile_rank {
        if profile_rank < required {
            return EquipDecision::RankTooLow;
        }
    }
    if members_only && !member {
        return EquipDecision::MembersOnly;
    }
    EquipDecision::Allowed
}
