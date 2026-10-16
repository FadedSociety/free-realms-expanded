//! Work item 034 acceptance: Ninja progression - recovered job/equipment data, the weapon to
//! ability toolbar, and a bounded server-side validation of ability use (job, rank, cooldown and
//! energy) that answers with the recovered client denial payloads.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs` - `AbilityPacketFailed = 1`,
//!   `AbilityPacketStartCasting = 3`, `AbilityPacketClientRequestStartAbility = 10`, all on the
//!   recovered ability opcode 36.
//! * `lab/reports/locale-id-to-text.json` - the original denial phrases: 285 ("You don't have
//!   enough energy for that ability!"), 287 ("That ability isn't ready yet..."), 288 ("You need to
//!   choose a target!"), and 3079, the community gateway's blanket refusal.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - the client's `NoCast*` cast-precondition
//!   ladder, whose last condition (`NoCastRecastTimer`) is the recast check.
//! * `lab/references/Sanctuary/src/Resources/Profiles.json` - profile 2 "Ninja", name id 3100,
//!   description id 3101, ability background set 3, hint 384179, `Trial: true`, and the creation
//!   kit `[19522, 19431, 19197, 19787, 19253, 75091]`.
//! * `lab/references/Sanctuary/src/Resources/ClientItemDefinitions.json` - the 30 named Ninja
//!   weapons 75090-75119: `MinProfileRank` 1/5/8/12/16 and `PowerRating` 1..5.
//! * `lab/references/freerealms-wiki/wiki.sqlite` - the weapon pages' damage ladder (254/889,
//!   488/1554, 853/2716, 1357/4750, 2372/8302) and the members-only level-4 trial restriction.
//! * `lab/references/Sanctuary/src/Resources/CombatJobs.json` and `CombatAbilities.json` -
//!   COMMUNITY-RECOVERED: reaches 4/7, basic recast 600 ms, energy 100 at 4 per second, the
//!   level-1 weapon 75091 mapping to basic 2154 and special 2050.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_034;` under `src/restoration/`. Integration step for the lead: register
//! the module and drop the include.

#[path = "../src/restoration/piece_034.rs"]
#[allow(dead_code)]
mod piece_034;

use piece_034::{
    ABILITY_BASE_OPCODE, ABILITY_OP_CLIENT_REQUEST_START_ABILITY, ABILITY_OP_FAILED,
    ABILITY_OP_START_CASTING, DENIAL_ABILITY_NOT_READY, DENIAL_CANNOT_USE_NOW,
    DENIAL_INSUFFICIENT_ENERGY, LIGHTNING_STRIKE_ABILITY_ID, NINJA_FALLBACK_BASIC_ABILITY_ID,
    NINJA_PROFILE_ID, NINJA_TRAINING_SWORD_1000_STORMS, NINJA_TRAINING_SWORD_DRAGONSTRIKE,
    NinjaCast, NinjaProgression, NinjaRefusal, NinjaSlot, NinjaState, THOUSAND_STORMS_ABILITY_ID,
};

const NINJA_BLADE_1000_STORMS: i32 = 75093;
const STUDENT_WIZARD_WAND: i32 = 4914;

/// A trimmed copy of the emitted `ninja-progression.json`: the fields the validator consumes keep
/// the recovered values, and the unrelated evidence sections are omitted.
const CATALOGUE: &str = r#"{
  "schema_version": 1,
  "job": {
    "profile_id": 2,
    "name_id": 3100,
    "name": "Ninja",
    "description_id": 3101,
    "description": "Ninjas are deadly Brawlers with dual swords and shadow skills.",
    "ability_bg_image_set": 3,
    "badge_image_set": 10,
    "button_image_set": 1,
    "hint_id": 384179,
    "hint": "Somewhere near calm water, a ninja trainer resides.",
    "members_only": false,
    "trial": true,
    "combat_members_only": true,
    "free_sample_level": 4,
    "default_item_ids": [19522, 19431, 19197, 19787, 19253, 75091]
  },
  "weapons": [
    {"item_definition_id": 75090, "name_id": 131974, "name": "Ninja's Training Sword of Dragonstrike",
     "min_profile_rank": 1, "power_rating": 1, "cost": 750},
    {"item_definition_id": 75091, "name_id": 131975, "name": "Ninja's Training Sword of 1000 Storms",
     "min_profile_rank": 1, "power_rating": 1, "cost": 750},
    {"item_definition_id": 75093, "name_id": 131977, "name": "Ninja's Blade of 1000 Storms",
     "min_profile_rank": 5, "power_rating": 2, "cost": 850}
  ],
  "abilities": [
    {"ability_id": 2010, "name": "", "name_id": 0, "damage": 150, "energy_cost": 0,
     "slot": "basic", "cooldown_ms": null, "cooldown_source": "unrecovered",
     "animation_id": 1099, "cast_effect_id": 0},
    {"ability_id": 2154, "name": "Lightning Strike", "name_id": 20217, "damage": 254,
     "energy_cost": 0, "slot": "basic", "cooldown_ms": 600,
     "cooldown_source": "community_reconstruction: CombatJobs.json BasicRecastMs",
     "animation_id": 1099, "cast_effect_id": 0},
    {"ability_id": 2050, "name": "1000 storms", "name_id": 419521, "damage": 889,
     "energy_cost": 100, "slot": "special", "cooldown_ms": null,
     "cooldown_source": "unrecovered", "animation_id": 1033, "cast_effect_id": 16088}
  ],
  "combat": {
    "auto_target_reach": 7,
    "basic_auto_target_reach": 4,
    "wield_type": 9,
    "basic_slot_definition_id": 4895,
    "special_slot_definition_id": 4899,
    "basic_recast_ms": 600,
    "fallback_basic_ability_id": 2010,
    "energy_max": 100,
    "energy_regen_per_second": 4,
    "weapon_abilities": [
      {"weapon_definition_ids": [75091], "basic_ability_id": 2154, "special_ability_id": 2050}
    ]
  },
  "refusals": [
    {"string_id": 285, "key": "NoCastInsufficientMana",
     "text": "You don't have enough energy for that ability!", "provenance": "original"},
    {"string_id": 287, "key": "NoCastRecastTimer",
     "text": "That ability isn't ready yet...", "provenance": "original"},
    {"string_id": 288, "key": null, "text": "You need to choose a target!", "provenance": "original"},
    {"string_id": 3079, "key": "StartAbilityRefused",
     "text": "You can't use that ability right now.", "provenance": "community"}
  ]
}"#;

fn progression() -> NinjaProgression {
    NinjaProgression::from_json(CATALOGUE.as_bytes()).expect("the recovered catalogue loads")
}

/// The recovered creation kit: rank 1, member trial, training sword of 1000 Storms equipped.
fn recruited() -> NinjaProgression {
    let mut ninja = progression();
    ninja.set_rank(1);
    ninja
        .equip(NINJA_TRAINING_SWORD_1000_STORMS)
        .expect("the rank 1 training sword is equippable");
    ninja
}

fn cast_basic(ninja: &mut NinjaProgression, now_ms: u64) -> Result<NinjaCast, NinjaRefusal> {
    ninja.cast(NINJA_PROFILE_ID, NinjaSlot::Basic, now_ms)
}

fn cast_special(ninja: &mut NinjaProgression, now_ms: u64) -> Result<NinjaCast, NinjaRefusal> {
    ninja.cast(NINJA_PROFILE_ID, NinjaSlot::Special, now_ms)
}

#[test]
fn recovered_ability_packet_ids_are_the_original_values() {
    assert_eq!(ABILITY_BASE_OPCODE, 36);
    assert_eq!(ABILITY_OP_FAILED, 1);
    assert_eq!(ABILITY_OP_START_CASTING, 3);
    assert_eq!(ABILITY_OP_CLIENT_REQUEST_START_ABILITY, 10);
    assert_eq!(DENIAL_INSUFFICIENT_ENERGY, 285);
    assert_eq!(DENIAL_ABILITY_NOT_READY, 287);
    assert_eq!(DENIAL_CANNOT_USE_NOW, 3079);
    assert_eq!(NINJA_PROFILE_ID, 2);
}

#[test]
fn a_corrupt_or_incomplete_catalogue_is_refused() {
    assert!(NinjaProgression::from_json(b"not json").is_err());

    let mut document: serde_json::Value = serde_json::from_str(CATALOGUE).unwrap();
    document["schema_version"] = serde_json::json!(99);
    assert!(NinjaProgression::from_json(document.to_string().as_bytes()).is_err());

    // A damaging ability that lost its damage cannot be trusted.
    let mut document: serde_json::Value = serde_json::from_str(CATALOGUE).unwrap();
    document["abilities"][2]["damage"] = serde_json::json!(0);
    assert!(NinjaProgression::from_json(document.to_string().as_bytes()).is_err());

    // A catalogue that fabricates a cooldown without a source is refused.
    let mut document: serde_json::Value = serde_json::from_str(CATALOGUE).unwrap();
    document["abilities"][1]["cooldown_source"] = serde_json::json!("");
    assert!(NinjaProgression::from_json(document.to_string().as_bytes()).is_err());
}

#[test]
fn a_foreign_weapon_cannot_be_equipped() {
    let mut ninja = progression();
    ninja.set_rank(16);
    assert_eq!(
        ninja.equip(STUDENT_WIZARD_WAND),
        Err(NinjaRefusal::UnknownWeapon {
            item_definition_id: STUDENT_WIZARD_WAND
        }),
        "the Wizard wand is not a recovered Ninja item definition"
    );
}

#[test]
fn a_weapon_tier_needs_its_recovered_rank() {
    let mut ninja = progression();
    ninja.set_rank(1);
    assert_eq!(
        ninja.equip(NINJA_BLADE_1000_STORMS),
        Err(NinjaRefusal::RankTooLow {
            item_definition_id: NINJA_BLADE_1000_STORMS,
            required: 5,
            rank: 1
        })
    );
    ninja.set_rank(5);
    let weapon = ninja
        .equip(NINJA_BLADE_1000_STORMS)
        .expect("rank 5 unlocks the power 2 blade");
    assert_eq!(weapon.power_rating, 2);
    assert_eq!(weapon.min_profile_rank, 5);
    assert_eq!(ninja.weapon_definition_id(), NINJA_BLADE_1000_STORMS);
}

#[test]
fn the_training_sword_resolves_the_recovered_ability_pair() {
    let ninja = recruited();
    assert_eq!(
        ninja.toolbar().unwrap(),
        vec![LIGHTNING_STRIKE_ABILITY_ID, THOUSAND_STORMS_ABILITY_ID]
    );
    let basic = ninja.ability(NinjaSlot::Basic).unwrap();
    assert_eq!(basic.damage, 254);
    assert_eq!(basic.cooldown_ms, Some(600));
    let special = ninja.ability(NinjaSlot::Special).unwrap();
    assert_eq!(special.damage, 889);
    assert_eq!(special.energy_cost, 100);
}

#[test]
fn an_unmapped_weapon_falls_back_to_the_basic_only() {
    let mut ninja = progression();
    ninja.set_rank(1);
    ninja
        .equip(NINJA_TRAINING_SWORD_DRAGONSTRIKE)
        .expect("the Dragonstrike sword is a recovered Ninja weapon");
    assert_eq!(
        ninja.toolbar().unwrap(),
        vec![NINJA_FALLBACK_BASIC_ABILITY_ID],
        "no ability ids are invented for a weapon family without a recovered mapping"
    );
    assert_eq!(
        cast_special(&mut ninja, 0),
        Err(NinjaRefusal::UnknownAbility {
            slot: NinjaSlot::Special
        })
    );
    assert_eq!(cast_basic(&mut ninja, 0).unwrap().ability_id, 2010);
}

#[test]
fn a_non_ninja_profile_cannot_cast() {
    let mut ninja = recruited();
    assert_eq!(
        ninja.cast(12, NinjaSlot::Basic, 0),
        Err(NinjaRefusal::NotNinjaJob { profile_id: 12 })
    );
    assert_eq!(
        ninja.denial_string_id(&NinjaRefusal::NotNinjaJob { profile_id: 12 }),
        Some(DENIAL_CANNOT_USE_NOW)
    );
}

#[test]
fn the_basic_recast_window_refuses_a_second_cast() {
    let mut ninja = recruited();
    let first = cast_basic(&mut ninja, 1_000).expect("the first basic cast is accepted");
    assert_eq!(first.ability_id, LIGHTNING_STRIKE_ABILITY_ID);
    assert_eq!(first.cooldown_ms, Some(600));

    assert_eq!(
        cast_basic(&mut ninja, 1_599),
        Err(NinjaRefusal::NotReady {
            ability_id: LIGHTNING_STRIKE_ABILITY_ID,
            remaining_ms: 1
        }),
        "the 600 ms community recast window still applies at 599 ms"
    );
    assert_eq!(
        ninja.denial_string_id(&NinjaRefusal::NotReady {
            ability_id: LIGHTNING_STRIKE_ABILITY_ID,
            remaining_ms: 1
        }),
        Some(DENIAL_ABILITY_NOT_READY)
    );

    let second = cast_basic(&mut ninja, 1_600).expect("the cast is ready at exactly 600 ms");
    assert_eq!(second.ability_id, LIGHTNING_STRIKE_ABILITY_ID);

    // A stale or replayed timestamp must not clear the window.
    assert_eq!(
        cast_basic(&mut ninja, 900),
        Err(NinjaRefusal::NotReady {
            ability_id: LIGHTNING_STRIKE_ABILITY_ID,
            remaining_ms: 1_300
        })
    );
}

#[test]
fn the_special_has_no_invented_recast() {
    let mut ninja = recruited();
    let first = cast_special(&mut ninja, 5_000).expect("the special is off cooldown");
    assert_eq!(first.cooldown_ms, None);
    assert_eq!(first.cooldown_source, "unrecovered");
    ninja.regenerate(30_000);
    let second = cast_special(&mut ninja, 30_000)
        .expect("no cooldown is fabricated for an unrecovered recast");
    assert_eq!(second.ability_id, THOUSAND_STORMS_ABILITY_ID);
}

#[test]
fn energy_is_spent_once_and_regenerates() {
    let mut ninja = recruited();
    assert_eq!(ninja.energy(), 100);
    let cast = cast_special(&mut ninja, 0).expect("the recovered energy pool pays for the special");
    assert_eq!(cast.energy_cost, 100);
    assert_eq!(cast.energy_after, 0);
    assert_eq!(
        cast_special(&mut ninja, 0),
        Err(NinjaRefusal::InsufficientEnergy {
            required: 100,
            available: 0
        })
    );
    assert_eq!(
        ninja.denial_string_id(&NinjaRefusal::InsufficientEnergy {
            required: 100,
            available: 0
        }),
        Some(DENIAL_INSUFFICIENT_ENERGY)
    );

    // COMMUNITY-RECOVERED regeneration: 4 per second, saturating at 100.
    ninja.regenerate(25_000);
    assert_eq!(ninja.energy(), 100);
    assert!(cast_special(&mut ninja, 25_000).is_ok());
}

#[test]
fn a_refused_cast_does_not_change_the_state() {
    let mut ninja = recruited();
    cast_basic(&mut ninja, 1_000).unwrap();
    let before = ninja.state().clone();
    assert!(cast_basic(&mut ninja, 1_100).is_err());
    assert_eq!(
        *ninja.state(),
        before,
        "a refusal must not spend energy or move a cooldown"
    );
}

#[test]
fn denial_packets_use_the_recovered_string_ids() {
    let ninja = recruited();
    let not_ready = ninja
        .denial_packet(&NinjaRefusal::NotReady {
            ability_id: LIGHTNING_STRIKE_ABILITY_ID,
            remaining_ms: 1,
        })
        .expect("the recast denial has a recovered client string");
    assert_eq!(
        not_ready,
        vec![36, 0, 1, 0, 0x1f, 0x01, 0x00, 0x00],
        "36/1 followed by string id 287 as a little-endian int32"
    );

    let energy = ninja
        .denial_packet(&NinjaRefusal::InsufficientEnergy {
            required: 100,
            available: 0,
        })
        .unwrap();
    assert_eq!(energy, vec![36, 0, 1, 0, 0x1d, 0x01, 0x00, 0x00]);

    assert!(
        ninja
            .denial_packet(&NinjaRefusal::RankTooLow {
                item_definition_id: NINJA_BLADE_1000_STORMS,
                required: 5,
                rank: 1,
            })
            .is_none(),
        "no denial string id is recovered for a rank gate, so no payload is fabricated"
    );
}

#[test]
fn start_casting_packet_matches_the_recovered_wire() {
    let mut ninja = recruited();
    let cast = cast_special(&mut ninja, 0).unwrap();
    let packet = ninja.start_casting_packet(0x1122_3344_5566_7788, 0x99, &cast, 750);

    let mut expected = vec![36, 0, 3, 0];
    expected.extend(0x1122_3344_5566_7788_u64.to_le_bytes());
    expected.extend(0x99_u64.to_le_bytes());
    expected.extend(16088_i32.to_le_bytes()); // recovered cast effect for 1000 Storms
    expected.extend(1033_i32.to_le_bytes()); // recovered animation for 1000 Storms
    expected.extend(THOUSAND_STORMS_ABILITY_ID.to_le_bytes());
    expected.extend(0.75_f32.to_le_bytes());
    expected.push(1);
    assert_eq!(packet, expected);
}

#[test]
fn the_cast_state_survives_a_restart() {
    let mut ninja = recruited();
    cast_basic(&mut ninja, 1_000).unwrap();
    let saved = serde_json::to_vec(ninja.state()).unwrap();
    let state: NinjaState = serde_json::from_slice(&saved).unwrap();

    let mut restarted =
        NinjaProgression::from_json_with_state(CATALOGUE.as_bytes(), state).unwrap();
    assert_eq!(restarted.energy(), 100);
    assert_eq!(
        cast_basic(&mut restarted, 1_100),
        Err(NinjaRefusal::NotReady {
            ability_id: LIGHTNING_STRIKE_ABILITY_ID,
            remaining_ms: 500
        }),
        "a restarted server still knows the last basic cast"
    );
    assert!(cast_basic(&mut restarted, 1_600).is_ok());
}

/// Reads the real catalogue emitted by `scripts/pieces/piece_034.py` and validates the recovered
/// job/equipment facts end to end. The lab is private state, so the check is skipped without the
/// environment variable.
#[test]
fn the_emitted_ninja_catalogue_validates() {
    let Some(path) = std::env::var_os("OSFR_PIECE_034_CATALOGUE") else {
        return;
    };
    let bytes = std::fs::read(path).expect("the emitted catalogue is readable");
    let mut ninja = NinjaProgression::from_json(&bytes).expect("the emitted document validates");
    assert_eq!(ninja.profile_id(), NINJA_PROFILE_ID);
    assert_eq!(ninja.job_name(), "Ninja");
    assert_eq!(ninja.free_sample_level(), 4);
    assert!(ninja.combat_members_only());
    assert_eq!(
        ninja.weapon_count(),
        71,
        "71 named Ninja weapons are recovered"
    );
    assert_eq!(
        ninja.canonical_tiers(),
        vec![(1, 1), (5, 2), (8, 3), (12, 4), (16, 5)]
    );

    ninja.set_rank(16);
    ninja
        .equip(75111)
        .expect("the rank 16 Shadow Blade of 1000 Storms");
    assert_eq!(
        ninja.toolbar().unwrap(),
        vec![NINJA_FALLBACK_BASIC_ABILITY_ID],
        "only the level 1 kit weapon has a recovered ability-id mapping"
    );
    assert!(ninja.ability(NinjaSlot::Special).is_none());
    assert_eq!(
        ninja.ability(NinjaSlot::Basic).unwrap().damage,
        150,
        "an unmapped weapon keeps the recovered bare-hand fallback"
    );

    ninja
        .equip(NINJA_TRAINING_SWORD_1000_STORMS)
        .expect("the recovered creation weapon is equippable from rank 1");
    assert_eq!(
        ninja.toolbar().unwrap(),
        vec![LIGHTNING_STRIKE_ABILITY_ID, THOUSAND_STORMS_ABILITY_ID]
    );
    let special = ninja.ability(NinjaSlot::Special).unwrap();
    assert_eq!(special.damage, 889);
    assert_eq!(
        special.cooldown_ms, None,
        "the special recast stays unrecovered"
    );
}
