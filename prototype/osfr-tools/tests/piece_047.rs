//! Work item 047 acceptance: recovered gear attributes and profile restrictions.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/references/Sanctuary/src/Sanctuary.Packet.Common/CharacterStat/CharacterStatId.cs`
//!   (pinned revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, AGPL-3.0, inspected as
//!   facts) - the recovered 69 value character stat enum, ids 0..68.
//! * The same tree's `CharacterStat/CharacterStats.cs` - the int/float default literal of
//!   each stat row, which is the recovered wire kind of that stat id.
//! * The same tree's `BaseItemStatDefinition.cs` and `ClientItemStatDefinition.cs` - the
//!   item stat wire shape `Id`, `Type`, `Value`, `Unknown`; `Type 0` writes a 32 bit int,
//!   `Type 1` writes a 32 bit float.
//! * `lab/references/Sanctuary/src/Resources/ClientItemDefinitions.json` - the two item
//!   rows that carry a `Stats.<key>.Id`/`Stats.<key>.Value` vector: 18092 Parachute and
//!   78397 The Fire Hawk. Both decode as IEEE-754 float32 bit patterns.
//! * The same tree's `Resources/Mounts.json` - mount 13459 The Fire Hawk carries the same
//!   glide values as item 78397, which is the independent second source for that vector.
//! * The same tree's `Resources/ItemClasses.txt` and `ItemClassDefinition.cs` - the class
//!   rows `Id^NameId^IconId^WieldType^StatId^ProfileNameId`.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - native `BaseItemStatDefinition` and
//!   `ItemStatDefinition` vtables and a `HashListMap<int,ItemStatDefinition,71,-1>`
//!   instantiation, so the original client stat container declares 71 slots while 69 ids
//!   are recovered.
//!
//! `equip_gate` is reconstructed policy over the recovered rows: the class row's owning
//! profile (when the profile name id names a job), the recovered `MinProfileRank` gate and
//! the recovered `MembersOnly` flag. Nothing here grants items, coins or experience, and
//! no native acceptance is claimed.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the
//! lead registers `pub mod piece_047;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_047.rs"]
#[allow(dead_code)]
mod piece_047;

use piece_047::{
    CHARACTER_STAT_ID_COUNT, CHARACTER_STAT_IDS, EquipDecision, FIRE_HAWK_MOUNT_ID,
    GEAR_STAT_ITEMS, GearAttributeError, INT_WIRE_TYPE, NATIVE_ITEM_STAT_CONTAINER_CAPACITY,
    StatKind, decode_item_stat, equip_gate, gear_class, gear_stat_item, stat_row,
};

fn decoded(item_id: i32) -> Vec<(u32, f32)> {
    let item = gear_stat_item(item_id).unwrap();
    item.raw_stats
        .iter()
        .map(|(id, bits)| (id.to_owned(), decode_item_stat(*id, *bits).unwrap().value))
        .collect()
}

#[test]
fn stat_table_is_contiguous_and_matches_the_recovered_enum() {
    assert_eq!(CHARACTER_STAT_IDS.len(), CHARACTER_STAT_ID_COUNT);
    assert_eq!(CHARACTER_STAT_ID_COUNT, 69);
    for (index, row) in CHARACTER_STAT_IDS.iter().enumerate() {
        assert_eq!(row.id as usize, index);
        assert_eq!(stat_row(row.id).map(|row| row.name), Some(row.name));
    }

    let max_health = stat_row(1).unwrap();
    assert_eq!(max_health.name, "MaxHealth");
    assert_eq!(max_health.kind, StatKind::Int);
    let default_speed = stat_row(59).unwrap();
    assert_eq!(default_speed.name, "GlideDefaultForwardSpeed");
    assert_eq!(default_speed.kind, StatKind::Float);
    let glide_enabled = stat_row(64).unwrap();
    assert_eq!(glide_enabled.name, "GlideEnabled");
    assert_eq!(glide_enabled.kind, StatKind::Int);
    let jump_height = stat_row(68).unwrap();
    assert_eq!(jump_height.name, "JumpHeight");
    assert_eq!(jump_height.kind, StatKind::Float);
    assert!(stat_row(69).is_none());
}

#[test]
fn parachute_and_fire_hawk_vectors_decode_to_the_recovered_values() {
    assert_eq!(GEAR_STAT_ITEMS.len(), 2);
    assert_eq!(
        decoded(18092),
        vec![
            (59, 8.0),
            (60, 2.0),
            (61, 18.0),
            (62, 0.75),
            (63, 4.0),
            (64, 1.0),
            (67, 4.0),
        ]
    );
    assert_eq!(
        decoded(78397),
        vec![
            (59, 12.0),
            (60, 6.0),
            (61, 22.0),
            (62, 0.75),
            (63, 3.0),
            (64, 1.0),
            (67, 4.0),
            (68, 5.0),
        ]
    );
}

#[test]
fn glide_vectors_keep_the_recovered_min_default_max_ordering() {
    for item in GEAR_STAT_ITEMS {
        let values = decoded(item.id);
        let value = |id: u32| values.iter().find(|(key, _)| *key == id).unwrap().1;
        assert!(value(60) >= 0.0, "{} minimum forward speed", item.id);
        assert!(value(60) <= value(59), "{} min <= default", item.id);
        assert!(value(59) <= value(61), "{} default <= max", item.id);
        assert!(value(62) > 0.0, "{} fall time", item.id);
        assert!(
            value(64) == 0.0 || value(64) == 1.0,
            "{} glide enabled is a flag",
            item.id
        );
    }
}

#[test]
fn int_kind_stats_must_decode_to_integral_values() {
    let enabled = decode_item_stat(64, 0x3f80_0000).unwrap();
    assert_eq!(enabled.kind, StatKind::Int);
    assert_eq!(enabled.value, 1.0);
    assert_eq!(enabled.raw_bits, 0x3f80_0000);

    assert_eq!(
        decode_item_stat(64, 0x3f8c_cccd),
        Err(GearAttributeError::NonIntegralIntStat {
            id: 64,
            value: 1.100000023841858
        })
    );
    assert_eq!(
        decode_item_stat(999, 0x4000_0000),
        Err(GearAttributeError::UnknownStat(999))
    );
}

#[test]
fn fire_hawk_item_binds_to_the_recovered_mount_row() {
    let item = gear_stat_item(78397).unwrap();
    assert_eq!(item.param1, Some(FIRE_HAWK_MOUNT_ID));
    assert_eq!(item.class_id, 172);
    // The pinned Fire Hawk item row carries no `Slot` field, so its slot is unrecovered.
    assert_eq!(item.slot, None);
    assert_eq!(item.min_profile_rank, None);
    assert!(!item.members_only);
    let class = gear_class(172).unwrap();
    assert_eq!(class.profile_name, "Ride");
    assert_eq!(class.owner_profile, None);
}

#[test]
fn recovered_classes_name_their_owning_profile_or_shared_scope() {
    let ninja = gear_class(2).unwrap();
    assert_eq!(ninja.name, "Ninja Gear");
    assert_eq!(ninja.owner_profile, Some(2));
    let all_jobs = gear_class(158).unwrap();
    assert_eq!(all_jobs.name, "Parachute");
    assert_eq!(all_jobs.profile_name, "All Jobs");
    assert_eq!(all_jobs.owner_profile, None);
    assert_eq!(gear_class(50).unwrap().profile_name, "Any Combat Job");
    assert_eq!(gear_class(69).unwrap().profile_name, "Any Job");
    assert!(gear_class(999).is_none());
}

#[test]
fn equip_gate_applies_the_recovered_profile_scope_rank_and_member_flags() {
    let ninja = gear_class(2).unwrap();
    assert_eq!(
        equip_gate(Some(ninja), None, false, 2, 1, true),
        EquipDecision::Allowed
    );
    assert_eq!(
        equip_gate(Some(ninja), None, false, 12, 20, true),
        EquipDecision::WrongProfile
    );
    assert_eq!(
        equip_gate(None, None, false, 2, 20, true),
        EquipDecision::UnknownClass
    );

    let shared = gear_class(158).unwrap();
    assert_eq!(
        equip_gate(Some(shared), Some(20), false, 45, 5, true),
        EquipDecision::RankTooLow
    );
    assert_eq!(
        equip_gate(Some(shared), Some(10), false, 45, 10, true),
        EquipDecision::Allowed
    );
    assert_eq!(
        equip_gate(Some(shared), None, true, 45, 20, false),
        EquipDecision::MembersOnly
    );
    assert_eq!(
        equip_gate(Some(shared), None, true, 45, 20, true),
        EquipDecision::Allowed
    );
}

#[test]
fn native_container_capacity_stays_above_the_recovered_id_space() {
    assert_eq!(NATIVE_ITEM_STAT_CONTAINER_CAPACITY, 71);
    assert!(NATIVE_ITEM_STAT_CONTAINER_CAPACITY >= CHARACTER_STAT_ID_COUNT);
    assert_eq!((INT_WIRE_TYPE, piece_047::FLOAT_WIRE_TYPE), (0, 1));
}
