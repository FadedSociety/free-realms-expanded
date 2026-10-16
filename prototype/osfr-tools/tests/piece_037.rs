//! Work item 037 acceptance: Medic healing, revive, resource and target rules.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/reports/locale-id-to-text.json` (sha256
//!   `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`) - the exported client
//!   locale. Medic job name 2278, description 2279, ability names/descriptions 420174/422746
//!   (Incision), 420186+442456/422736+442468 (Triage), 3810/420497/422737 (Target Vitals),
//!   3784/3813/47279-47282 (First Aid and its five rank texts), 3800/73908/73914 (Shock Paddles),
//!   22485/22489 (Health/Energy stat names), 92026/2492/2983 (heal, energy and damage effect
//!   tooltips), 285 ("You don't have enough energy for that ability!"), 280 ("You can't use
//!   abilities when you are knocked out!"), 37466 (knock-out recovery), 22111/22112 (post-revival
//!   invulnerability "with 2 effects"), 20463 (resurrection scroll).
//! * `lab/working/OSFR.Client/OSFR Client/Resources/WieldTypeAnimationMappings.txt` - `14^Medic`.
//! * `lab/working/OSFR.Client/OSFR Client/Resources/CharacterStatDefinitions.txt` - stat id 1 ->
//!   22485 Health, stat id 5 -> 22489 Energy.
//! * `lab/working/OSFR.Client/OSFR Client/Resources/AbilityEffectTypeSettings.txt` - effect type 3
//!   carries the `Param1TunedLow..High` heal/energy tooltips (92026, 2492); type 1 carries damage
//!   (2983); type 2 carries the flat `Param2TunedLow` stat text.
//! * `lab/working/OSFR.Client/OSFR Client/Resources/ActorCompositeEffectDefinitions.xml` - 16161
//!   `PFX_heal_red_explosion_lg_AOE`, 16162 `PFX_heal_red_hands_cast_starburst`, 4002
//!   `PFX_magic-resurrect`.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` - `RespawnWindow:DisplayRespawn(timer, isOverworld,
//!   reviveCost)`, the two-button paid revive string and `Ui.RevivePlayer()`/`Ui.OnPaidRevive(b)`.
//! * `lab/reports/piece-037-20261009T035019Z/revive-window/scripts/frame_1/DoAction.as` - the
//!   decompiled original `revive_window.gfx`. `setCoinAmount_lua`/`setReviveCost_lua`,
//!   `showTwoButtons_lua` disabling the paid button when `m_coinWallet < m_reviveCost`, the safe
//!   button label "Revive at safe location: Free" and the `FR_event` commands
//!   `respawn`, `OnPaidRevive( 1 )`, `OnPaidRevive( 0 )`.
//! * `lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs` and `BaseCombatPackets.cs` - original
//!   server opcode enums (`AbilityPacketFailed` = 1, `AbilityPacketStartCasting` = 3,
//!   `AbilityPacketClientRequestStartAbility` = 10, `CombatPacketSingleAttackTarget` = 3,
//!   `CombatPacketAttackTargetDamage` = 4).
//!
//! COMMUNITY mapping, never presented as original: the ability ids 3010/3011/3024 against those
//! locale names and effect ids come from `lab/references/Sanctuary/src/Resources/CombatAbilities.json`
//! and `CombatJobs.json` (profile 11, wield 14, energy 100, `AoeDamageHeal`). Those numbers are
//! carried for cross-reference only; no balance value here is claimed as historical.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_037;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_037.rs"]
#[allow(dead_code)]
mod piece_037;

use piece_037::{
    ABILITY_INCISION, ABILITY_MEDIC_FALLBACK, ABILITY_TRIAGE, AbilityKind, CastDenial,
    ENERGY_STAT_ID, FIRST_AID_HEAL_LADDER, HEALTH_STAT_ID, KNOCKED_OUT_ACTION_DENIAL,
    MEDIC_WIELD_TYPE, NOT_ENOUGH_ENERGY_DENIAL, PAID_REVIVE_LABEL_TEMPLATE,
    POST_REVIVAL_INVULNERABILITY_EFFECT_COUNT, RECOVER_NOW_LABEL, RESURRECT_EFFECT_ID,
    ReviveChoice, ReviveEvent, ReviveOffer, SAFE_REVIVE_LABEL, TRIAGE_CAST_EFFECT_ID,
    TRIAGE_HIT_EFFECT_ID, TargetRelation, VitalState, can_use_ability, first_aid_heal,
    revive_event, target_allowed,
};

#[test]
fn medic_identity_is_the_recovered_client_wield_type() {
    assert_eq!(MEDIC_WIELD_TYPE, 14);
    assert_eq!(HEALTH_STAT_ID, 1);
    assert_eq!(ENERGY_STAT_ID, 5);
    assert_eq!(ABILITY_MEDIC_FALLBACK, 3010);
    assert_eq!(ABILITY_INCISION, 3011);
    assert_eq!(ABILITY_TRIAGE, 3024);
    assert_eq!(TRIAGE_CAST_EFFECT_ID, 16162);
    assert_eq!(TRIAGE_HIT_EFFECT_ID, 16161);
    assert_eq!(RESURRECT_EFFECT_ID, 4002);
}

#[test]
fn first_aid_ladder_is_the_recovered_five_rank_text() {
    assert_eq!(FIRST_AID_HEAL_LADDER, [250, 281, 313, 344, 375]);
    assert_eq!(first_aid_heal(1), Some(250));
    assert_eq!(first_aid_heal(2), Some(281));
    assert_eq!(first_aid_heal(5), Some(375));
    assert_eq!(first_aid_heal(0), None);
    assert_eq!(first_aid_heal(6), None);
}

#[test]
fn heal_accepts_self_group_member_and_ally_targets() {
    for relation in [
        TargetRelation::SelfPlayer,
        TargetRelation::GroupMember,
        TargetRelation::Ally,
    ] {
        assert_eq!(
            target_allowed(AbilityKind::Heal, relation, VitalState::Alive),
            Ok(()),
            "{relation:?}"
        );
    }
}

#[test]
fn heal_rejects_opponents_and_knocked_out_targets() {
    assert_eq!(
        target_allowed(
            AbilityKind::Heal,
            TargetRelation::Opponent,
            VitalState::Alive
        ),
        Err(CastDenial::HealRequiresFriendlyTarget)
    );
    assert_eq!(
        target_allowed(
            AbilityKind::Heal,
            TargetRelation::Ally,
            VitalState::KnockedOut
        ),
        Err(CastDenial::HealRequiresLivingTarget)
    );
}

#[test]
fn damage_accepts_only_opponents() {
    assert_eq!(
        target_allowed(
            AbilityKind::Damage,
            TargetRelation::Opponent,
            VitalState::Alive
        ),
        Ok(())
    );
    for relation in [
        TargetRelation::SelfPlayer,
        TargetRelation::GroupMember,
        TargetRelation::Ally,
    ] {
        assert_eq!(
            target_allowed(AbilityKind::Damage, relation, VitalState::Alive),
            Err(CastDenial::DamageRequiresOpponent),
            "{relation:?}"
        );
    }
}

#[test]
fn revive_accepts_only_knocked_out_friendlies() {
    assert_eq!(
        target_allowed(
            AbilityKind::Revive,
            TargetRelation::Ally,
            VitalState::KnockedOut
        ),
        Ok(())
    );
    assert_eq!(
        target_allowed(
            AbilityKind::Revive,
            TargetRelation::GroupMember,
            VitalState::KnockedOut
        ),
        Ok(())
    );
    assert_eq!(
        target_allowed(AbilityKind::Revive, TargetRelation::Ally, VitalState::Alive),
        Err(CastDenial::ReviveRequiresKnockedOutTarget)
    );
    assert_eq!(
        target_allowed(
            AbilityKind::Revive,
            TargetRelation::Opponent,
            VitalState::KnockedOut
        ),
        Err(CastDenial::ReviveRequiresFriendlyTarget)
    );
    // A knocked-out actor is revived through `wndRespawn`/`Ui.RevivePlayer()`, not by casting an
    // ability on itself, so an ability revive always needs another friendly actor.
    assert_eq!(
        target_allowed(
            AbilityKind::Revive,
            TargetRelation::SelfPlayer,
            VitalState::KnockedOut
        ),
        Err(CastDenial::ReviveRequiresFriendlyTarget)
    );
}

#[test]
fn damage_on_a_knocked_out_opponent_keeps_no_recovered_restriction() {
    // Open point, recorded rather than invented: the recovered text only forbids a knocked-out actor
    // from casting. Nothing recovered restricts damaging an already knocked-out opponent.
    assert_eq!(
        target_allowed(
            AbilityKind::Damage,
            TargetRelation::Opponent,
            VitalState::KnockedOut
        ),
        Ok(())
    );
}

#[test]
fn knocked_out_actors_cannot_use_abilities() {
    assert_eq!(
        can_use_ability(VitalState::KnockedOut, 100, 0),
        Err(CastDenial::ActorKnockedOut)
    );
    assert_eq!(
        CastDenial::ActorKnockedOut.client_string(),
        Some(KNOCKED_OUT_ACTION_DENIAL)
    );
    assert_eq!(
        KNOCKED_OUT_ACTION_DENIAL,
        "You can't use abilities when you are knocked out!"
    );
}

#[test]
fn energy_cost_is_reported_with_the_recovered_denial_string() {
    assert_eq!(
        can_use_ability(VitalState::Alive, 99, 100),
        Err(CastDenial::NotEnoughEnergy)
    );
    assert_eq!(can_use_ability(VitalState::Alive, 100, 100), Ok(()));
    assert_eq!(can_use_ability(VitalState::Alive, 0, 0), Ok(()));
    assert_eq!(
        CastDenial::NotEnoughEnergy.client_string(),
        Some(NOT_ENOUGH_ENERGY_DENIAL)
    );
    assert_eq!(
        NOT_ENOUGH_ENERGY_DENIAL,
        "You don't have enough energy for that ability!"
    );
}

#[test]
fn paid_revive_is_disabled_below_the_server_supplied_cost() {
    let offer = ReviveOffer {
        timer_seconds: 30,
        is_overworld: true,
        revive_cost_coins: 1000,
    };
    assert!(offer.paid_revive_enabled(1000));
    assert!(offer.paid_revive_enabled(1001));
    assert!(!offer.paid_revive_enabled(999));
}

#[test]
fn overworld_offer_matches_the_two_button_dialog() {
    let offer = ReviveOffer {
        timer_seconds: 30,
        is_overworld: true,
        revive_cost_coins: 1000,
    };
    assert_eq!(
        offer.choices(),
        vec![
            ReviveChoice::PaidRevive { cost_coins: 1000 },
            ReviveChoice::SafeRevive,
        ]
    );
    assert_eq!(SAFE_REVIVE_LABEL, "Revive at safe location: Free");
    assert_eq!(ReviveChoice::SafeRevive.label_template(), SAFE_REVIVE_LABEL);
    assert_eq!(
        ReviveChoice::PaidRevive { cost_coins: 1000 }.label_template(),
        PAID_REVIVE_LABEL_TEMPLATE
    );
}

#[test]
fn encounter_offer_counts_down_without_a_coin_option() {
    let offer = ReviveOffer {
        timer_seconds: 30,
        is_overworld: false,
        revive_cost_coins: 1000,
    };
    assert_eq!(
        offer.choices(),
        vec![ReviveChoice::Countdown { seconds: 30 }]
    );
    assert_eq!(
        ReviveChoice::Countdown { seconds: 30 }.label_template(),
        RECOVER_NOW_LABEL
    );
    assert_eq!(RECOVER_NOW_LABEL, "Recover Now!");
}

#[test]
fn revive_events_match_the_recovered_client_commands() {
    assert_eq!(
        revive_event(ReviveChoice::PaidRevive { cost_coins: 1000 }),
        ReviveEvent::PaidRevive { paid: true }
    );
    assert_eq!(
        revive_event(ReviveChoice::SafeRevive),
        ReviveEvent::PaidRevive { paid: false }
    );
    assert_eq!(
        revive_event(ReviveChoice::Countdown { seconds: 30 }),
        ReviveEvent::Respawn
    );
}

#[test]
fn post_revival_invulnerability_declares_two_effects() {
    assert_eq!(POST_REVIVAL_INVULNERABILITY_EFFECT_COUNT, 2);
}

#[test]
fn ability_denials_carry_no_client_text_when_none_was_recovered() {
    assert_eq!(CastDenial::HealRequiresFriendlyTarget.client_string(), None);
    assert_eq!(CastDenial::DamageRequiresOpponent.client_string(), None);
    assert_eq!(
        CastDenial::ReviveRequiresFriendlyTarget.client_string(),
        None
    );
    assert_eq!(
        CastDenial::ReviveRequiresKnockedOutTarget.client_string(),
        None
    );
    assert_eq!(CastDenial::HealRequiresLivingTarget.client_string(), None);
}
