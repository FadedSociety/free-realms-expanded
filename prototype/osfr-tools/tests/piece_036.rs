//! Work item 036 acceptance: Wizard progression - spells, resources, effects and a validated cast
//! slice that rejects invalid targets and repeated casts.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs` - the recovered ability packet ids.
//!   `AbilityPacketFailed = 1`, `AbilityPacketStartCasting = 3`, `AbilityPacketSetDefinition = 5`,
//!   `AbilityPacketClientRequestStartAbility = 10`, `AbilityPacketAbilityDefinition = 13`.
//! * `lab/decompiled/server/Gateway/Gateway.Player/UpdateStats.cs` - the opening mana update is
//!   tunneled packet `0x0026` sub `13` carrying `100`, `100`, `true`, and the set-definition literal
//!   `240005000100000008000000...` decodes to opcode 36 sub 5, profile 1, slot count 8, eight
//!   all-zero (Type 0) slots.
//!
//! COMMUNITY-RECOVERED data (pinned local references, not original assets):
//! * `lab/references/Sanctuary/src/Resources/CombatJobs.json` profile 12 "Wizard" - ranged reach 20,
//!   wield type 10, slot definitions 4895/4899, basic recast 600 ms, fallback basic 4010, energy
//!   100 max / 4 per second, Student Wizard Wand 4914 -> basic 4011 / special 4028.
//! * `lab/references/Sanctuary/src/Resources/CombatAbilities.json` - 4010 (bare-hand fallback, 200),
//!   4011 Zap (279), 4028 Lightning Blast (1143, energy 100, cast effect 16305).
//! * `lab/references/Sanctuary/src/Resources/ClientItemDefinitions.json` - item 4914 "Student Wizard
//!   Wand", `Class` 10, slot 7, NameId 11056.
//! * `lab/references/freerealms-wiki/wiki.sqlite` - the Student Wizard Wand page states Zap deals 279
//!   and Lightning Blast 1143; the Wizard page lists Zap/Fireball/Ice Nova/Chaos at job levels
//!   1/5/10/15.
//! * `lab/references/Sanctuary-combat-pr-research/pr-77/.../AbilityPacketStartCasting.cs` - cast
//!   feedback fields (caster guid, target guid, composite effect, animation, ability id, action time,
//!   progress bar) and `AbilityPacketClientRequestStartAbility` - action bar id, slot, target flag,
//!   then a `Vector4` position when the flag is 1 or a guid otherwise.
//!
//! Nothing here claims native acceptance, hit timing, projectile travel, AoE resolution or historical
//! rewards. The cast slice is a reconstruction of server-side validation only.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_036;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_036.rs"]
#[allow(dead_code)]
mod piece_036;

use piece_036::{
    ABILITY_BASE_OPCODE, ABILITY_OP_ABILITY_DEFINITION, ABILITY_OP_CLIENT_REQUEST_START_ABILITY,
    ABILITY_OP_FAILED, ABILITY_OP_SET_DEFINITION, ABILITY_OP_START_CASTING, AbilityEffect,
    CastDenial, CastTargeting, LIGHTNING_BLAST_ABILITY_ID, MANA_UPDATE_OPCODE,
    MANA_UPDATE_SUBOPCODE, STUDENT_WIZARD_WAND_DEFINITION_ID,
    STUDENT_WIZARD_WAND_LIGHTNING_BLAST_DAMAGE, STUDENT_WIZARD_WAND_ZAP_DAMAGE, WIZARD_ABILITIES,
    WIZARD_AUTO_TARGET_REACH, WIZARD_BASIC_RECAST_MS, WIZARD_BASIC_SLOT_DEFINITION_ID,
    WIZARD_ENERGY_MAX, WIZARD_ENERGY_REGEN_PER_SECOND, WIZARD_FALLBACK_BASIC_ABILITY_ID,
    WIZARD_PROFILE_ID, WIZARD_SPECIAL_SLOT_DEFINITION_ID, WIZARD_WIELD_TYPE, WizardCastState,
    WizardSlot, WizardTarget, WizardToolbar, ZAP_ABILITY_ID, wizard_ability,
};

const WAND: i32 = STUDENT_WIZARD_WAND_DEFINITION_ID;

fn hostile_target(guid: u64, position: [f32; 3]) -> WizardTarget {
    WizardTarget {
        guid,
        position,
        dead: false,
        disposition: 0,
    }
}

fn wizard_toolbar() -> WizardToolbar {
    WizardToolbar::for_weapon(WAND)
}

/// One valid Zap cast against the recovered 20 unit reach, locked for 500 ms of action time.
fn zap_once(state: &mut WizardCastState, now_ms: u64) -> Result<piece_036::WizardCast, CastDenial> {
    let target = hostile_target(0x1000, [10.0, 0.0, 0.0]);
    state.cast(
        WIZARD_PROFILE_ID,
        &wizard_toolbar(),
        WizardSlot::Basic,
        CastTargeting::Guid(0x1000),
        [0.0, 0.0, 0.0],
        Some(&target),
        now_ms,
        500,
    )
}

#[test]
fn ability_packet_ids_are_the_recovered_original_values() {
    assert_eq!(ABILITY_BASE_OPCODE, 36);
    assert_eq!(ABILITY_OP_FAILED, 1);
    assert_eq!(ABILITY_OP_START_CASTING, 3);
    assert_eq!(ABILITY_OP_SET_DEFINITION, 5);
    assert_eq!(ABILITY_OP_CLIENT_REQUEST_START_ABILITY, 10);
    assert_eq!(ABILITY_OP_ABILITY_DEFINITION, 13);
    assert_eq!(MANA_UPDATE_OPCODE, 38);
    assert_eq!(MANA_UPDATE_SUBOPCODE, 13);
}

#[test]
fn wizard_job_facts_are_the_community_recovered_kit_values() {
    assert_eq!(WIZARD_PROFILE_ID, 12);
    assert_eq!(WIZARD_AUTO_TARGET_REACH, 20.0);
    assert_eq!(WIZARD_WIELD_TYPE, 10);
    assert_eq!(WIZARD_BASIC_RECAST_MS, 600);
    assert_eq!(WIZARD_ENERGY_MAX, 100);
    assert_eq!(WIZARD_ENERGY_REGEN_PER_SECOND, 4);
    assert_eq!(WIZARD_BASIC_SLOT_DEFINITION_ID, 4895);
    assert_eq!(WIZARD_SPECIAL_SLOT_DEFINITION_ID, 4899);
    assert_eq!(WIZARD_FALLBACK_BASIC_ABILITY_ID, 4010);
    assert_eq!(STUDENT_WIZARD_WAND_DEFINITION_ID, 4914);
}

#[test]
fn the_three_recovered_wizard_abilities_carry_their_real_effects() {
    assert_eq!(WIZARD_ABILITIES.len(), 3);

    let fallback = wizard_ability(WIZARD_FALLBACK_BASIC_ABILITY_ID).expect("fallback");
    assert_eq!(fallback.effect, AbilityEffect::SweepDamage);
    assert_eq!(fallback.damage, 200);
    assert_eq!(fallback.energy_cost, 0);
    assert_eq!(fallback.animation_id, 1099);
    assert_eq!(fallback.hit_effect_id, 7);
    assert_eq!(fallback.cast_effect_id, 0);
    assert_eq!(fallback.target_animation_id, 1120);
    assert_eq!(fallback.icon_id, 280);
    assert_eq!(fallback.name_id, 0);

    let zap = wizard_ability(ZAP_ABILITY_ID).expect("zap");
    assert_eq!(zap.effect, AbilityEffect::SweepDamage);
    assert_eq!(zap.damage, STUDENT_WIZARD_WAND_ZAP_DAMAGE);
    assert_eq!(zap.damage, 279);
    assert_eq!(zap.energy_cost, 0);
    assert_eq!(zap.animation_id, 1099);
    assert_eq!(zap.hit_effect_id, 7);
    assert_eq!(zap.icon_id, 14434);
    assert_eq!(zap.name_id, 17071);

    let blast = wizard_ability(LIGHTNING_BLAST_ABILITY_ID).expect("lightning blast");
    assert_eq!(blast.damage, STUDENT_WIZARD_WAND_LIGHTNING_BLAST_DAMAGE);
    assert_eq!(blast.damage, 1143);
    assert_eq!(blast.energy_cost, WIZARD_ENERGY_MAX);
    assert_eq!(blast.animation_id, 1138);
    assert_eq!(blast.cast_effect_id, 16305);
    assert_eq!(blast.hit_effect_id, 0);
    assert_eq!(blast.target_animation_id, 1120);
    assert_eq!(blast.icon_id, 294);
    assert_eq!(blast.name_id, 442459);

    assert!(wizard_ability(4012).is_none());
}

#[test]
fn the_student_wand_resolves_to_zap_and_lightning_blast() {
    let toolbar = wizard_toolbar();
    assert_eq!(toolbar.profile_id, WIZARD_PROFILE_ID);
    assert_eq!(toolbar.ability_id(WizardSlot::Basic), Some(ZAP_ABILITY_ID));
    assert_eq!(
        toolbar.ability_id(WizardSlot::Special),
        Some(LIGHTNING_BLAST_ABILITY_ID)
    );
    assert_eq!(
        toolbar.ability(WizardSlot::Basic).map(|ability| ability.id),
        Some(ZAP_ABILITY_ID)
    );
    assert_eq!(
        toolbar.slot_definition_id(WizardSlot::Basic),
        Some(WIZARD_BASIC_SLOT_DEFINITION_ID)
    );
    assert_eq!(
        toolbar.slot_definition_id(WizardSlot::Special),
        Some(WIZARD_SPECIAL_SLOT_DEFINITION_ID)
    );
}

#[test]
fn an_unmapped_weapon_falls_back_to_the_bare_hand_basic_only() {
    let toolbar = WizardToolbar::for_weapon(4915);
    assert_eq!(
        toolbar.ability_id(WizardSlot::Basic),
        Some(WIZARD_FALLBACK_BASIC_ABILITY_ID)
    );
    assert_eq!(toolbar.ability_id(WizardSlot::Special), None);
    assert_eq!(toolbar.slot_definition_id(WizardSlot::Special), None);
    assert_eq!(
        wizard_ability(WIZARD_FALLBACK_BASIC_ABILITY_ID)
            .unwrap()
            .damage,
        200
    );
}

#[test]
fn the_first_zap_costs_no_energy_and_returns_the_recovered_effects() {
    let mut state = WizardCastState::new(0x42);
    let cast = zap_once(&mut state, 0).expect("first zap");
    assert_eq!(cast.ability_id, ZAP_ABILITY_ID);
    assert_eq!(cast.damage, STUDENT_WIZARD_WAND_ZAP_DAMAGE);
    assert_eq!(cast.energy_cost, 0);
    assert_eq!(cast.energy_after, WIZARD_ENERGY_MAX);
    assert_eq!(cast.animation_id, 1099);
    assert_eq!(cast.hit_effect_id, 7);
    assert_eq!(cast.cast_effect_id, 0);
    assert_eq!(cast.target_animation_id, 1120);
    assert_eq!(cast.targeting, CastTargeting::Guid(0x1000));
    assert_eq!(state.energy(), WIZARD_ENERGY_MAX);
}

#[test]
fn repeated_casts_are_rejected_while_the_previous_cast_is_locked() {
    let mut state = WizardCastState::new(0x42);
    zap_once(&mut state, 1_000).expect("first zap");
    assert_eq!(zap_once(&mut state, 1_100), Err(CastDenial::CastInProgress));
    assert_eq!(zap_once(&mut state, 1_499), Err(CastDenial::CastInProgress));
    // The 500 ms action time has elapsed but the 600 ms basic recast window has not: still refused.
    assert_eq!(zap_once(&mut state, 1_500), Err(CastDenial::Recast));
    assert!(zap_once(&mut state, 1_600).is_ok());
}

#[test]
fn a_basic_cast_inside_the_recovered_600ms_recast_window_is_rejected() {
    let mut state = WizardCastState::new(0x42);
    let target = hostile_target(0x1000, [10.0, 0.0, 0.0]);
    let toolbar = wizard_toolbar();
    let cast = |state: &mut WizardCastState, now_ms| {
        state.cast(
            WIZARD_PROFILE_ID,
            &toolbar,
            WizardSlot::Basic,
            CastTargeting::Guid(0x1000),
            [0.0, 0.0, 0.0],
            Some(&target),
            now_ms,
            0,
        )
    };
    assert!(cast(&mut state, 5_000).is_ok());
    assert_eq!(cast(&mut state, 5_599), Err(CastDenial::Recast));
    assert!(cast(&mut state, 5_600).is_ok());
}

#[test]
fn lightning_blast_spends_the_recovered_hundred_energy_exactly_once() {
    let mut state = WizardCastState::new(0x42);
    let toolbar = wizard_toolbar();
    let target = hostile_target(0x2000, [12.0, 0.0, 0.0]);
    let blast = state
        .cast(
            WIZARD_PROFILE_ID,
            &toolbar,
            WizardSlot::Special,
            CastTargeting::Guid(0x2000),
            [0.0, 0.0, 0.0],
            Some(&target),
            0,
            1_000,
        )
        .expect("first blast");
    assert_eq!(blast.ability_id, LIGHTNING_BLAST_ABILITY_ID);
    assert_eq!(blast.damage, STUDENT_WIZARD_WAND_LIGHTNING_BLAST_DAMAGE);
    assert_eq!(blast.energy_cost, WIZARD_ENERGY_MAX);
    assert_eq!(blast.cast_effect_id, 16305);
    assert_eq!(blast.energy_after, 0);
    assert_eq!(state.energy(), 0);

    // A repeated request while the first blast still locks the slot is refused.
    let no_energy = state.cast(
        WIZARD_PROFILE_ID,
        &toolbar,
        WizardSlot::Special,
        CastTargeting::Guid(0x2000),
        [0.0, 0.0, 0.0],
        Some(&target),
        0,
        1_000,
    );
    assert_eq!(no_energy, Err(CastDenial::CastInProgress));
    let after_lock = state.cast(
        WIZARD_PROFILE_ID,
        &toolbar,
        WizardSlot::Special,
        CastTargeting::Guid(0x2000),
        [0.0, 0.0, 0.0],
        Some(&target),
        2_000,
        1_000,
    );
    assert_eq!(after_lock, Err(CastDenial::InsufficientEnergy));
}

#[test]
fn energy_regenerates_four_per_whole_second_and_carries_the_remainder() {
    let mut state = WizardCastState::new(0x42);
    let toolbar = wizard_toolbar();
    let target = hostile_target(0x2000, [12.0, 0.0, 0.0]);
    state
        .cast(
            WIZARD_PROFILE_ID,
            &toolbar,
            WizardSlot::Special,
            CastTargeting::Guid(0x2000),
            [0.0, 0.0, 0.0],
            Some(&target),
            0,
            0,
        )
        .expect("blast");
    assert_eq!(state.energy(), 0);
    state.regenerate(999);
    assert_eq!(state.energy(), 0);
    state.regenerate(1_000);
    assert_eq!(state.energy(), 4);
    state.regenerate(3_500);
    assert_eq!(state.energy(), 12);
    state.regenerate(4_000);
    assert_eq!(state.energy(), 16);
    // The pool saturates at the recovered maximum and never wraps.
    state.regenerate(100_000);
    assert_eq!(state.energy(), WIZARD_ENERGY_MAX);
}

#[test]
fn an_out_of_reach_target_is_rejected_at_the_recovered_twenty_units() {
    let mut state = WizardCastState::new(0x42);
    let toolbar = wizard_toolbar();
    let cast_at = |state: &mut WizardCastState, position: [f32; 3]| {
        let target = hostile_target(0x3000, position);
        state.cast(
            WIZARD_PROFILE_ID,
            &toolbar,
            WizardSlot::Basic,
            CastTargeting::Guid(0x3000),
            [0.0, 0.0, 0.0],
            Some(&target),
            0,
            0,
        )
    };
    assert_eq!(
        cast_at(&mut state, [20.1, 0.0, 0.0]),
        Err(CastDenial::OutOfReach)
    );
    assert!(cast_at(&mut state, [0.0, 0.0, 20.0]).is_ok());
    assert_eq!(
        cast_at(&mut state, [0.0, 20.5, 0.0]),
        Err(CastDenial::OutOfReach)
    );
}

#[test]
fn dead_self_and_non_hostile_targets_are_rejected() {
    let mut state = WizardCastState::new(0x42);
    let toolbar = wizard_toolbar();
    let mut dead = hostile_target(0x4000, [5.0, 0.0, 0.0]);
    dead.dead = true;
    let mut ally = hostile_target(0x5000, [5.0, 0.0, 0.0]);
    ally.disposition = 1;
    let cast = |state: &mut WizardCastState, targeting: CastTargeting, target: &WizardTarget| {
        state.cast(
            WIZARD_PROFILE_ID,
            &toolbar,
            WizardSlot::Basic,
            targeting,
            [0.0, 0.0, 0.0],
            Some(target),
            0,
            0,
        )
    };
    assert_eq!(
        cast(&mut state, CastTargeting::Guid(0x4000), &dead),
        Err(CastDenial::DeadTarget)
    );
    assert_eq!(
        cast(&mut state, CastTargeting::Guid(0x5000), &ally),
        Err(CastDenial::NotHostileTarget)
    );
    assert_eq!(
        cast_at_self(&mut state, &toolbar),
        Err(CastDenial::SelfTarget)
    );
}

fn cast_at_self(
    state: &mut WizardCastState,
    toolbar: &WizardToolbar,
) -> Result<piece_036::WizardCast, CastDenial> {
    let facts = hostile_target(0x42, [1.0, 0.0, 0.0]);
    state.cast(
        WIZARD_PROFILE_ID,
        toolbar,
        WizardSlot::Basic,
        CastTargeting::Guid(0x42),
        [0.0, 0.0, 0.0],
        Some(&facts),
        0,
        0,
    )
}

#[test]
fn unresolved_malformed_and_non_finite_requests_are_rejected() {
    let mut state = WizardCastState::new(0x42);
    let toolbar = wizard_toolbar();
    let missing = state.cast(
        WIZARD_PROFILE_ID,
        &toolbar,
        WizardSlot::Basic,
        CastTargeting::Guid(0x9000),
        [0.0, 0.0, 0.0],
        None,
        0,
        0,
    );
    assert_eq!(missing, Err(CastDenial::UnknownTarget));

    let mismatched = hostile_target(0x9001, [1.0, 0.0, 0.0]);
    let wrong_guid = state.cast(
        WIZARD_PROFILE_ID,
        &toolbar,
        WizardSlot::Basic,
        CastTargeting::Guid(0x9000),
        [0.0, 0.0, 0.0],
        Some(&mismatched),
        0,
        0,
    );
    assert_eq!(wrong_guid, Err(CastDenial::UnknownTarget));

    let zero_guid = state.cast(
        WIZARD_PROFILE_ID,
        &toolbar,
        WizardSlot::Basic,
        CastTargeting::Guid(0),
        [0.0, 0.0, 0.0],
        Some(&mismatched),
        0,
        0,
    );
    assert_eq!(zero_guid, Err(CastDenial::UnknownTarget));

    let non_finite = state.cast(
        WIZARD_PROFILE_ID,
        &toolbar,
        WizardSlot::Basic,
        CastTargeting::Position([f32::NAN, 0.0, 0.0, 1.0]),
        [0.0, 0.0, 0.0],
        None,
        0,
        0,
    );
    assert_eq!(non_finite, Err(CastDenial::NonFiniteTarget));
}

#[test]
fn a_ground_targeted_request_keeps_its_position_and_skips_actor_checks() {
    let mut state = WizardCastState::new(0x42);
    let toolbar = wizard_toolbar();
    let cast = state
        .cast(
            WIZARD_PROFILE_ID,
            &toolbar,
            WizardSlot::Special,
            CastTargeting::Position([1.0, 2.0, 3.0, 1.0]),
            [0.0, 0.0, 0.0],
            None,
            0,
            0,
        )
        .expect("ground cast");
    assert_eq!(
        cast.targeting,
        CastTargeting::Position([1.0, 2.0, 3.0, 1.0])
    );
    assert_eq!(cast.ability_id, LIGHTNING_BLAST_ABILITY_ID);
}

#[test]
fn wrong_profile_empty_slot_and_unknown_ability_are_rejected() {
    let mut state = WizardCastState::new(0x42);
    let toolbar = wizard_toolbar();
    let target = hostile_target(0x6000, [4.0, 0.0, 0.0]);
    let wrong_profile = state.cast(
        1,
        &toolbar,
        WizardSlot::Basic,
        CastTargeting::Guid(0x6000),
        [0.0, 0.0, 0.0],
        Some(&target),
        0,
        0,
    );
    assert_eq!(wrong_profile, Err(CastDenial::NotWizardProfile));

    let fallback = WizardToolbar::for_weapon(4915);
    let empty = state.cast(
        WIZARD_PROFILE_ID,
        &fallback,
        WizardSlot::Special,
        CastTargeting::Guid(0x6000),
        [0.0, 0.0, 0.0],
        Some(&target),
        0,
        0,
    );
    assert_eq!(empty, Err(CastDenial::EmptySlot));

    let bogus = WizardToolbar {
        profile_id: WIZARD_PROFILE_ID,
        basic_ability_id: 7,
        special_ability_id: None,
    };
    let unknown = state.cast(
        WIZARD_PROFILE_ID,
        &bogus,
        WizardSlot::Basic,
        CastTargeting::Guid(0x6000),
        [0.0, 0.0, 0.0],
        Some(&target),
        0,
        0,
    );
    assert_eq!(unknown, Err(CastDenial::UnknownAbility));
}

#[test]
fn a_non_finite_caster_position_is_rejected_before_any_energy_is_spent() {
    let mut state = WizardCastState::new(0x42);
    let toolbar = wizard_toolbar();
    let target = hostile_target(0x7000, [4.0, 0.0, 0.0]);
    let denied = state.cast(
        WIZARD_PROFILE_ID,
        &toolbar,
        WizardSlot::Special,
        CastTargeting::Guid(0x7000),
        [f32::INFINITY, 0.0, 0.0],
        Some(&target),
        0,
        0,
    );
    assert_eq!(denied, Err(CastDenial::NonFiniteCaster));
    assert_eq!(state.energy(), WIZARD_ENERGY_MAX);
}
