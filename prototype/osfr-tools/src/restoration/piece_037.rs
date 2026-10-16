//! Medic progression: healing, revive, resource and target rules (work item 037).
//!
//! ORIGINAL facts implemented here, with the recovered source of each:
//! * the Medic job is the client `WieldType` 14 (`Resources/WieldTypeAnimationMappings.txt`,
//!   `14^Medic^1099^0`) and the job strings are client locale 2278/2279/384185
//!   ("Medics use medicine to keep allies healthy as well as debilitate opponents.");
//! * the two combat resources are client stat 1 Health (locale 22485) and stat 5 Energy (22489),
//!   declared in `Resources/CharacterStatDefinitions.txt`;
//! * Triage is described by locale 422736/442468 as "Sorts out friend from foe with surgical
//!   precision, healing you and your group and damaging all nearby opponents." Incision is locale
//!   422746, "Strikes with surgical precision, damaging all opponents in front of you." The job's
//!   own heal is First Aid (3784) whose five rank texts are locale 3813 and 47279-47282;
//! * Triage's recovered client effects are 16162 `PFX_heal_red_hands_cast_starburst` (cast) and
//!   16161 `PFX_heal_red_explosion_lg_AOE` (hit) in
//!   `Resources/ActorCompositeEffectDefinitions.xml`;
//! * a knocked-out actor cannot act: locale 280 "You can't use abilities when you are knocked out!",
//!   and locale 37466 says the knock-out lasts "a few seconds" before the player gets back up;
//! * an ability whose energy cost is not met reports locale 285, "You don't have enough energy for
//!   that ability!";
//! * the revive window is `Main.wndRespawn` -> `UI\revive_window.swf` (`wndRespawn.xml`). The
//!   decompiled `revive_window.gfx` disables its paid button when `m_coinWallet < m_reviveCost`,
//!   labels the alternative "Revive at safe location: Free" and emits `respawn`,
//!   `OnPaidRevive( 1 )` for the paid button and `OnPaidRevive( 0 )` for the safe button. The
//!   client Lua `RespawnWindow:DisplayRespawn(timer, isOverworld, reviveCost)` shows the two-coin
//!   dialog only when `isOverworld == 1` and otherwise starts a countdown; `Ui.RevivePlayer()` and
//!   `Ui.OnPaidRevive(b)` are the revive entry points;
//! * a revived character receives the post-revival invulnerability ability, which locale 22112
//!   states has "2 effects"; the recovered resurrect visual is effect 4002 `PFX_magic-resurrect`.
//!
//! RECONSTRUCTED here, never presented as historical:
//! * the target matrix. Only the friendly/opponent split above is original; the concrete
//!   accept/reject pairs, their denial variants and the ordering of the checks are a bounded server
//!   policy built on that text;
//! * `ReviveRequiresKnockedOutTarget` for a living ally, and the decision that an ability revive
//!   needs another friendly actor. Self-revive is the `wndRespawn` flow, not an ability.
//!
//! Explicit gaps, do not imply these:
//! * behaviour when the *target* of damage is already knocked out is unrecovered. This module adds
//!   no restriction there, and `damage_on_a_knocked_out_opponent_keeps_no_recovered_restriction`
//!   records that as a deliberate open point;
//! * only locale 285 and 280 have recovered client text. Every other denial carries no string
//!   because the real one was not recovered;
//! * the numeric ability ids 3010/3011/3024 and the energy cost/radius/damage figures are community
//!   mappings from `lab/references/Sanctuary` used only to cross-reference the recovered locale
//!   names. No balance number here is claimed as original;
//! * heals-over-time, critical healing, range and line of sight are unrecovered.

/// ORIGINAL: `Resources/WieldTypeAnimationMappings.txt` line `14^Medic^1099^0`.
pub const MEDIC_WIELD_TYPE: u32 = 14;
/// ORIGINAL: `Resources/CharacterStatDefinitions.txt` id 1 -> locale 22485 "Health".
pub const HEALTH_STAT_ID: u32 = 1;
/// ORIGINAL: `Resources/CharacterStatDefinitions.txt` id 5 -> locale 22489 "Energy".
pub const ENERGY_STAT_ID: u32 = 5;
/// COMMUNITY mapping (Sanctuary `CombatAbilities.json`): unarmed Medic fallback ability 3010.
pub const ABILITY_MEDIC_FALLBACK: u32 = 3010;
/// COMMUNITY mapping: the Medic basic ability "Incision" (locale 420174) is id 3011.
pub const ABILITY_INCISION: u32 = 3011;
/// COMMUNITY mapping: the Medic special ability "Triage" (locale 420186/442456) is id 3024.
pub const ABILITY_TRIAGE: u32 = 3024;
/// COMMUNITY mapping plus ORIGINAL name: Triage's cast effect 16162 is
/// `PFX_heal_red_hands_cast_starburst`.
pub const TRIAGE_CAST_EFFECT_ID: u32 = 16162;
/// COMMUNITY mapping plus ORIGINAL name: Triage's hit effect 16161 is
/// `PFX_heal_red_explosion_lg_AOE`.
pub const TRIAGE_HIT_EFFECT_ID: u32 = 16161;
/// ORIGINAL: `PFX_magic-resurrect` is effect id 4002 in the client effect table.
pub const RESURRECT_EFFECT_ID: u32 = 4002;
/// ORIGINAL: the five First Aid rank texts, locale 3813 then 47279-47282. Rank 1 heals 250 and the
/// last text ("Heals yourself and any ally near you for 375 health.") carries no next rank.
pub const FIRST_AID_HEAL_LADDER: [u32; 5] = [250, 281, 313, 344, 375];
/// ORIGINAL: locale 22112 - "This ability (with 2 effects) is placed on the PCs after they revive".
pub const POST_REVIVAL_INVULNERABILITY_EFFECT_COUNT: u32 = 2;
/// ORIGINAL: locale 285.
pub const NOT_ENOUGH_ENERGY_DENIAL: &str = "You don't have enough energy for that ability!";
/// ORIGINAL: locale 280.
pub const KNOCKED_OUT_ACTION_DENIAL: &str = "You can't use abilities when you are knocked out!";
/// ORIGINAL: the client Lua `RespawnWindow:DisplayRespawn` coin-revive template.
pub const PAID_REVIVE_LABEL_TEMPLATE: &str = "Revive here:\n %1 coins";
/// ORIGINAL: the decompiled `revive_window.gfx` safe button label.
pub const SAFE_REVIVE_LABEL: &str = "Revive at safe location: Free";
/// ORIGINAL: locale 408279, the countdown window's recover-now button.
pub const RECOVER_NOW_LABEL: &str = "Recover Now!";

/// Vital state of an actor or target. ORIGINAL naming: the client calls this being "knocked out"
/// (locale 280/37466/408276), not dead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VitalState {
    Alive,
    KnockedOut,
}

/// Who the target is relative to the caster. ORIGINAL split: Medic text distinguishes "you and your
/// group"/"any ally near you" from "all nearby opponents". `Ally` is a friendly actor outside the
/// caster's group, which locale 3813 ("yourself and any ally near you") covers on the heal side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetRelation {
    SelfPlayer,
    GroupMember,
    Ally,
    Opponent,
}

impl TargetRelation {
    /// A friendly actor other than the caster. Ability revives need one of these.
    pub fn is_other_friendly(&self) -> bool {
        matches!(self, Self::GroupMember | Self::Ally)
    }

    /// Friend or self, the side the recovered Medic text heals.
    pub fn is_friendly(&self) -> bool {
        !matches!(self, Self::Opponent)
    }
}

/// The three recovered ability shapes: opponent damage, group/ally heal and ally revive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbilityKind {
    Damage,
    Heal,
    Revive,
}

/// RECONSTRUCTED policy outcomes. Only `ActorKnockedOut` and `NotEnoughEnergy` have recovered
/// client text; the rest are server-side classifications with no recovered string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastDenial {
    ActorKnockedOut,
    NotEnoughEnergy,
    HealRequiresLivingTarget,
    HealRequiresFriendlyTarget,
    DamageRequiresOpponent,
    ReviveRequiresKnockedOutTarget,
    ReviveRequiresFriendlyTarget,
}

impl CastDenial {
    /// ORIGINAL text where one was recovered; `None` means the client string is a known gap.
    pub fn client_string(&self) -> Option<&'static str> {
        match self {
            Self::ActorKnockedOut => Some(KNOCKED_OUT_ACTION_DENIAL),
            Self::NotEnoughEnergy => Some(NOT_ENOUGH_ENERGY_DENIAL),
            Self::HealRequiresLivingTarget
            | Self::HealRequiresFriendlyTarget
            | Self::DamageRequiresOpponent
            | Self::ReviveRequiresKnockedOutTarget
            | Self::ReviveRequiresFriendlyTarget => None,
        }
    }
}

impl std::fmt::Display for CastDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::ActorKnockedOut => "actor is knocked out",
            Self::NotEnoughEnergy => "energy cost is not covered",
            Self::HealRequiresLivingTarget => "healing needs a living target",
            Self::HealRequiresFriendlyTarget => "healing needs a friendly target",
            Self::DamageRequiresOpponent => "damage needs an opponent",
            Self::ReviveRequiresKnockedOutTarget => "revive needs a knocked-out target",
            Self::ReviveRequiresFriendlyTarget => "revive needs a friendly target",
        })
    }
}

impl std::error::Error for CastDenial {}

/// RECONSTRUCTED server policy: a knocked-out actor cannot cast (ORIGINAL locale 280) and an
/// ability whose energy cost exceeds the current energy is refused (ORIGINAL locale 285 text).
pub fn can_use_ability(vital: VitalState, energy: u32, cost: u32) -> Result<(), CastDenial> {
    if vital == VitalState::KnockedOut {
        return Err(CastDenial::ActorKnockedOut);
    }
    if energy < cost {
        return Err(CastDenial::NotEnoughEnergy);
    }
    Ok(())
}

/// RECONSTRUCTED target policy on top of the recovered friendly/opponent text.
pub fn target_allowed(
    kind: AbilityKind,
    relation: TargetRelation,
    vital: VitalState,
) -> Result<(), CastDenial> {
    match kind {
        AbilityKind::Heal => {
            if !relation.is_friendly() {
                return Err(CastDenial::HealRequiresFriendlyTarget);
            }
            if vital == VitalState::KnockedOut {
                return Err(CastDenial::HealRequiresLivingTarget);
            }
            Ok(())
        }
        AbilityKind::Damage => {
            if relation != TargetRelation::Opponent {
                return Err(CastDenial::DamageRequiresOpponent);
            }
            // No recovered rule restricts damage against a knocked-out opponent; see the module gap.
            Ok(())
        }
        AbilityKind::Revive => {
            if !relation.is_other_friendly() {
                return Err(CastDenial::ReviveRequiresFriendlyTarget);
            }
            if vital != VitalState::KnockedOut {
                return Err(CastDenial::ReviveRequiresKnockedOutTarget);
            }
            Ok(())
        }
    }
}

/// ORIGINAL: the First Aid heal per rank from the recovered rank texts. `None` outside ranks 1..=5.
pub fn first_aid_heal(rank: u8) -> Option<u32> {
    let index = usize::from(rank).checked_sub(1)?;
    FIRST_AID_HEAL_LADDER.get(index).copied()
}

/// ORIGINAL fields of `RespawnWindow:DisplayRespawn(timer, isOverworld, reviveCost)`. The cost and
/// timer are server-supplied; nothing here invents a value for them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviveOffer {
    pub timer_seconds: u32,
    pub is_overworld: bool,
    pub revive_cost_coins: u32,
}

/// ORIGINAL buttons. `Countdown` is the non-overworld recover window, `PaidRevive` and `SafeRevive`
/// are the two buttons the overworld dialog shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviveChoice {
    Countdown { seconds: u32 },
    PaidRevive { cost_coins: u32 },
    SafeRevive,
}

impl ReviveChoice {
    /// ORIGINAL label text each button shows.
    pub fn label_template(&self) -> &'static str {
        match self {
            Self::Countdown { .. } => RECOVER_NOW_LABEL,
            Self::PaidRevive { .. } => PAID_REVIVE_LABEL_TEMPLATE,
            Self::SafeRevive => SAFE_REVIVE_LABEL,
        }
    }
}

impl ReviveOffer {
    /// ORIGINAL branching: `isOverworld == 1` shows the coin dialog, everything else counts down.
    pub fn choices(&self) -> Vec<ReviveChoice> {
        if self.is_overworld {
            vec![
                ReviveChoice::PaidRevive {
                    cost_coins: self.revive_cost_coins,
                },
                ReviveChoice::SafeRevive,
            ]
        } else {
            vec![ReviveChoice::Countdown {
                seconds: self.timer_seconds,
            }]
        }
    }

    /// ORIGINAL arithmetic from `showTwoButtons_lua`: the paid button is enabled only when
    /// `m_coinWallet < m_reviveCost` is false.
    pub fn paid_revive_enabled(&self, coin_wallet: u32) -> bool {
        coin_wallet >= self.revive_cost_coins
    }
}

/// ORIGINAL client commands: the countdown/respawn button emits `respawn`
/// (`Ui.RevivePlayer()`), and both coin buttons emit `OnPaidRevive(1)`/`OnPaidRevive(0)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviveEvent {
    Respawn,
    PaidRevive { paid: bool },
}

pub fn revive_event(choice: ReviveChoice) -> ReviveEvent {
    match choice {
        ReviveChoice::Countdown { .. } => ReviveEvent::Respawn,
        ReviveChoice::PaidRevive { .. } => ReviveEvent::PaidRevive { paid: true },
        ReviveChoice::SafeRevive => ReviveEvent::PaidRevive { paid: false },
    }
}
