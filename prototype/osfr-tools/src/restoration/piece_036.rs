//! Wizard progression - spells, resources, effects and a validated cast slice (work item 036).
//!
//! ORIGINAL facts implemented here, with the recovered source of each:
//! * ability packet ids, from the decompiled original server enum
//!   `lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs`:
//!   `AbilityPacketFailed = 1`, `AbilityPacketStartCasting = 3`, `AbilityPacketSetDefinition = 5`,
//!   `AbilityPacketClientRequestStartAbility = 10`, `AbilityPacketAbilityDefinition = 13`. All of
//!   them ride the recovered ability opcode `0x0024` = 36.
//! * the opening mana update is tunneled packet `0x0026` = 38 sub `13` carrying `100`, `100` then a
//!   boolean (`Gateway/Gateway.Player/UpdateStats.cs`, `SendClientUpdatePacketMana`), and the
//!   recovered toolbar literal `240005000100000008000000…` decodes to 36/5, profile 1, slot count 8
//!   and eight all-zero (Type 0) slots. The community reconstruction agrees: eight slots, positional.
//!
//! COMMUNITY-RECOVERED data, pinned under `lab/references` and verified by
//! the private recovery checker `piece_036.py`:
//! * `Sanctuary/src/Resources/CombatJobs.json` profile 12 "Wizard": ranged, auto-target reach 20,
//!   wield type 10, basic/special slot definitions 4895/4899, basic recast 600 ms, fallback basic
//!   4010, energy 100 max with 4 per second, and Student Wizard Wand 4914 -> basic 4011 Zap /
//!   special 4028 Lightning Blast.
//! * `Sanctuary/src/Resources/CombatAbilities.json`: 4010 (bare-hand fallback, 200 damage, no cost),
//!   4011 Zap (279, no cost, animation 1099, hit effect 7, target animation 1120) and 4028 Lightning
//!   Blast (1143, energy 100, animation 1138, cast effect 16305, target animation 1120).
//! * `Sanctuary/src/Resources/ClientItemDefinitions.json` item 4914 "Student Wizard Wand": Class 10
//!   (the Wizard wield type), slot 7, NameId 11056, icon 3158/234, power rating 1.
//! * `freerealms-wiki/wiki.sqlite`: the Student Wizard Wand page states Zap 279 and Lightning Blast
//!   1143, matching the ability table; the Wizard page lists Zap/Fireball/Ice Nova/Chaos at job
//!   levels 1/5/10/15.
//! * `Sanctuary-combat-pr-research/pr-77/.../AbilityPacketStartCasting.cs` and
//!   `Sanctuary.Packet/BaseAbilityPacket/AbilityPacketClientRequestStartAbility.cs`: the cast
//!   feedback fields and the client request shape `[36][10][int32 action bar][int32 slot][int32
//!   target][uchar-type payload]`, where target 1 carries a `Vector4` position and target 0 carries a
//!   guid. The local sandbox `src/combat.rs` already parses the same 24/32 byte lengths.
//!
//! RECONSTRUCTED here (policy, clearly not historical): the validation order, the denial set, the
//! single in-flight cast lock and the once-only energy spend. Non-finite guard values and the
//! inclusive 20 unit reach boundary are local arithmetic, not recovered tolerances.
//!
//! Explicit gaps, do not imply these:
//! * `ActionTime` (cast duration / slot lock) is not recovered for any ability; it is a caller input.
//! * no denial string id is recovered, so `AbilityPacketFailed` payloads are not fabricated.
//! * hit timing, projectile travel, the `SweepDamage` cone and area shape, and line of sight are
//!   unrecovered; this module validates a request and returns effects for the caller to play.
//! * AoE/positional resolution for a `target == 1` request is unrecovered; such a request is only
//!   checked for finite coordinates and carries no actor checks.
//! * job level, experience, unlocks and quest rewards are outside this slice.

/// ORIGINAL: the base opcode for every recovered ability packet.
pub const ABILITY_BASE_OPCODE: u16 = 36;
/// ORIGINAL: `BaseAbilityPackets.AbilityPacketFailed`.
pub const ABILITY_OP_FAILED: u16 = 1;
/// ORIGINAL: `BaseAbilityPackets.AbilityPacketStartCasting`.
pub const ABILITY_OP_START_CASTING: u16 = 3;
/// ORIGINAL: `BaseAbilityPackets.AbilityPacketSetDefinition`.
pub const ABILITY_OP_SET_DEFINITION: u16 = 5;
/// ORIGINAL: `BaseAbilityPackets.AbilityPacketClientRequestStartAbility`.
pub const ABILITY_OP_CLIENT_REQUEST_START_ABILITY: u16 = 10;
/// ORIGINAL: `BaseAbilityPackets.AbilityPacketAbilityDefinition`.
pub const ABILITY_OP_ABILITY_DEFINITION: u16 = 13;
/// ORIGINAL: the mana/resource update opcode from `SendClientUpdatePacketMana`.
pub const MANA_UPDATE_OPCODE: u16 = 38;
/// ORIGINAL: the mana/resource update sub opcode.
pub const MANA_UPDATE_SUBOPCODE: u16 = 13;

/// COMMUNITY-RECOVERED: `CombatJobs.json` profile 12 is Wizard.
pub const WIZARD_PROFILE_ID: i32 = 12;
/// COMMUNITY-RECOVERED: Wizard ranged auto-target reach in world units.
pub const WIZARD_AUTO_TARGET_REACH: f32 = 20.0;
/// COMMUNITY-RECOVERED: Wizard wield type, matching item 4914 `Class` 10.
pub const WIZARD_WIELD_TYPE: i32 = 10;
/// COMMUNITY-RECOVERED: basic ability recast window.
pub const WIZARD_BASIC_RECAST_MS: u64 = 600;
/// COMMUNITY-RECOVERED: maximum energy; the original opening mana update also carries 100.
pub const WIZARD_ENERGY_MAX: i32 = 100;
/// COMMUNITY-RECOVERED: energy regeneration per second.
pub const WIZARD_ENERGY_REGEN_PER_SECOND: i32 = 4;
/// COMMUNITY-RECOVERED: the basic toolbar slot definition id.
pub const WIZARD_BASIC_SLOT_DEFINITION_ID: i32 = 4895;
/// COMMUNITY-RECOVERED: the special toolbar slot definition id.
pub const WIZARD_SPECIAL_SLOT_DEFINITION_ID: i32 = 4899;
/// COMMUNITY-RECOVERED: bare-hand fallback used when no mapped wand is equipped.
pub const WIZARD_FALLBACK_BASIC_ABILITY_ID: i32 = 4010;
/// COMMUNITY-RECOVERED: Student Wizard Wand item definition id.
pub const STUDENT_WIZARD_WAND_DEFINITION_ID: i32 = 4914;
/// COMMUNITY-RECOVERED: Student Wizard Wand basic ability, Zap.
pub const ZAP_ABILITY_ID: i32 = 4011;
/// COMMUNITY-RECOVERED: Student Wizard Wand special ability, Lightning Blast.
pub const LIGHTNING_BLAST_ABILITY_ID: i32 = 4028;
/// COMMUNITY-RECOVERED: Zap damage, also stated on the wiki wand page.
pub const STUDENT_WIZARD_WAND_ZAP_DAMAGE: i32 = 279;
/// COMMUNITY-RECOVERED: Lightning Blast damage, also stated on the wiki wand page.
pub const STUDENT_WIZARD_WAND_LIGHTNING_BLAST_DAMAGE: i32 = 1143;

/// COMMUNITY-RECOVERED effect families. Every Wizard ability in the recovered table is
/// `SweepDamage`; the other families (AoE damage, AoE damage/heal) belong to jobs outside this item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbilityEffect {
    SweepDamage,
}

/// COMMUNITY-RECOVERED ability definition fields used by the cast slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WizardAbility {
    pub id: i32,
    pub effect: AbilityEffect,
    pub damage: i32,
    pub energy_cost: i32,
    pub animation_id: i32,
    pub hit_effect_id: i32,
    pub cast_effect_id: i32,
    pub target_animation_id: i32,
    pub icon_id: i32,
    pub name_id: i32,
    pub description_id: i32,
}

/// COMMUNITY-RECOVERED: the three abilities reachable from the Wizard toolbar.
pub const WIZARD_ABILITIES: [WizardAbility; 3] = [
    WizardAbility {
        id: WIZARD_FALLBACK_BASIC_ABILITY_ID,
        effect: AbilityEffect::SweepDamage,
        damage: 200,
        energy_cost: 0,
        animation_id: 1099,
        hit_effect_id: 7,
        cast_effect_id: 0,
        target_animation_id: 1120,
        icon_id: 280,
        name_id: 0,
        description_id: 0,
    },
    WizardAbility {
        id: ZAP_ABILITY_ID,
        effect: AbilityEffect::SweepDamage,
        damage: STUDENT_WIZARD_WAND_ZAP_DAMAGE,
        energy_cost: 0,
        animation_id: 1099,
        hit_effect_id: 7,
        cast_effect_id: 0,
        target_animation_id: 1120,
        icon_id: 14434,
        name_id: 17071,
        description_id: 0,
    },
    WizardAbility {
        id: LIGHTNING_BLAST_ABILITY_ID,
        effect: AbilityEffect::SweepDamage,
        damage: STUDENT_WIZARD_WAND_LIGHTNING_BLAST_DAMAGE,
        energy_cost: WIZARD_ENERGY_MAX,
        animation_id: 1138,
        hit_effect_id: 0,
        cast_effect_id: 16305,
        target_animation_id: 1120,
        icon_id: 294,
        name_id: 442459,
        description_id: 0,
    },
];

/// The recovered ability with this definition id, if it belongs to Wizard.
pub fn wizard_ability(id: i32) -> Option<&'static WizardAbility> {
    WIZARD_ABILITIES.iter().find(|ability| ability.id == id)
}

/// COMMUNITY-RECOVERED toolbar positions. The set definition packet is positional: index 0 is the
/// basic slot definition, index 1 the special slot definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WizardSlot {
    Basic,
    Special,
}

/// The resolved Wizard toolbar for one equipped weapon.
///
/// The wire carries the toolbar slot definition ids (4895/4899) in the set definition packet and the
/// matching ability definition packet; the client's cast request references the action-bar slot. This
/// struct keeps the mapping explicit so a caller cannot invent an ability the job does not own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WizardToolbar {
    pub profile_id: i32,
    pub basic_ability_id: i32,
    pub special_ability_id: Option<i32>,
}

impl WizardToolbar {
    /// RECONSTRUCTED mapping of the community kit: the Student Wizard Wand resolves to Zap and
    /// Lightning Blast, any other or missing weapon falls back to the bare-hand basic and an empty
    /// special slot.
    pub fn for_weapon(weapon_definition_id: i32) -> Self {
        if weapon_definition_id == STUDENT_WIZARD_WAND_DEFINITION_ID {
            Self {
                profile_id: WIZARD_PROFILE_ID,
                basic_ability_id: ZAP_ABILITY_ID,
                special_ability_id: Some(LIGHTNING_BLAST_ABILITY_ID),
            }
        } else {
            Self {
                profile_id: WIZARD_PROFILE_ID,
                basic_ability_id: WIZARD_FALLBACK_BASIC_ABILITY_ID,
                special_ability_id: None,
            }
        }
    }

    pub fn ability_id(&self, slot: WizardSlot) -> Option<i32> {
        match slot {
            WizardSlot::Basic => Some(self.basic_ability_id),
            WizardSlot::Special => self.special_ability_id,
        }
    }

    pub fn ability(&self, slot: WizardSlot) -> Option<&'static WizardAbility> {
        self.ability_id(slot).and_then(wizard_ability)
    }

    /// The toolbar slot definition id that the set definition packet carries for this slot. An empty
    /// special slot has no definition id.
    pub fn slot_definition_id(&self, slot: WizardSlot) -> Option<i32> {
        match slot {
            WizardSlot::Basic => Some(WIZARD_BASIC_SLOT_DEFINITION_ID),
            WizardSlot::Special => self
                .special_ability_id
                .map(|_| WIZARD_SPECIAL_SLOT_DEFINITION_ID),
        }
    }
}

/// ORIGINAL target rule from the recoverable combat audit (`notes/Research/Native Combat Controls.md`):
/// target selection requires hostile `Disposition` 0 at actor +0x178 and rejects the dead flag at
/// +0x6f8 bit 7. The fields here are the caller's view of the candidate actor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WizardTarget {
    pub guid: u64,
    pub position: [f32; 3],
    pub dead: bool,
    pub disposition: i32,
}

/// The two recovered client request modes: a guid target, or a `Vector4` ground position when the
/// request's target flag is 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CastTargeting {
    Guid(u64),
    Position([f32; 4]),
}

/// RECONSTRUCTED denial set. No denial string id is recovered, so nothing here fabricates an
/// `AbilityPacketFailed` payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastDenial {
    /// The active profile is not Wizard.
    NotWizardProfile,
    /// The requested slot has no ability for the equipped weapon.
    EmptySlot,
    /// The slot's ability id is not a recovered Wizard ability.
    UnknownAbility,
    /// The guid is zero or does not resolve to the candidate actor the caller supplied.
    UnknownTarget,
    /// The targeting payload carries a non-finite coordinate.
    NonFiniteTarget,
    /// The candidate actor carries the recovered dead flag.
    DeadTarget,
    /// The candidate actor is not hostile (`Disposition` != 0).
    NotHostileTarget,
    /// The request targets the caster.
    SelfTarget,
    /// The candidate actor is farther than the recovered 20 unit reach.
    OutOfReach,
    /// The caster position is not finite.
    NonFiniteCaster,
    /// The pool cannot pay the recovered energy cost.
    InsufficientEnergy,
    /// A basic cast inside the recovered 600 ms recast window.
    Recast,
    /// Another cast is still in its action-time window.
    CastInProgress,
}

/// The once-only consequence of an accepted cast: the effects for the caller to play and the energy
/// actually spent. Damage application, hit resolution and rewards stay with the owning subsystems.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WizardCast {
    pub ability_id: i32,
    pub targeting: CastTargeting,
    pub damage: i32,
    pub energy_cost: i32,
    pub animation_id: i32,
    pub cast_effect_id: i32,
    pub hit_effect_id: i32,
    pub target_animation_id: i32,
    pub energy_after: i32,
}

/// RECONSTRUCTED server-side cast state: the energy pool, the basic recast stamp and the single
/// in-flight cast lock. The client locks the pressed action-bar slot for `ActionTime`; modelling one
/// in-flight cast per caster is a local policy, not a recovered rule.
#[derive(Debug)]
pub struct WizardCastState {
    caster_guid: u64,
    energy: i32,
    last_energy_ms: u64,
    last_basic_ms: Option<u64>,
    lock_until_ms: u64,
}

impl WizardCastState {
    /// A fresh caster starts at the recovered maximum energy.
    pub fn new(caster_guid: u64) -> Self {
        Self {
            caster_guid,
            energy: WIZARD_ENERGY_MAX,
            last_energy_ms: 0,
            last_basic_ms: None,
            lock_until_ms: 0,
        }
    }

    pub fn energy(&self) -> i32 {
        self.energy
    }

    /// COMMUNITY-RECOVERED regeneration: 4 energy per whole second, saturating at 100. Partial
    /// seconds carry; a non-monotonic timestamp grants nothing.
    pub fn regenerate(&mut self, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.last_energy_ms);
        let whole_seconds = elapsed / 1_000;
        if whole_seconds == 0 {
            return;
        }
        self.last_energy_ms += whole_seconds * 1_000;
        let gained = (whole_seconds as i64) * (WIZARD_ENERGY_REGEN_PER_SECOND as i64);
        self.energy = (self.energy as i64 + gained).min(WIZARD_ENERGY_MAX as i64) as i32;
    }

    /// Validate one recovered client cast request and, if accepted, spend its energy exactly once.
    ///
    /// `action_time_ms` is the unrecovered cast duration the caller currently uses for the
    /// `AbilityPacketStartCasting` payload; it is also the window in which a second cast is refused.
    #[allow(clippy::too_many_arguments)]
    pub fn cast(
        &mut self,
        profile_id: i32,
        toolbar: &WizardToolbar,
        slot: WizardSlot,
        targeting: CastTargeting,
        caster_position: [f32; 3],
        target: Option<&WizardTarget>,
        now_ms: u64,
        action_time_ms: u64,
    ) -> Result<WizardCast, CastDenial> {
        self.regenerate(now_ms);
        if profile_id != WIZARD_PROFILE_ID || toolbar.profile_id != profile_id {
            return Err(CastDenial::NotWizardProfile);
        }
        let ability_id = toolbar.ability_id(slot).ok_or(CastDenial::EmptySlot)?;
        let ability = wizard_ability(ability_id).ok_or(CastDenial::UnknownAbility)?;

        if !caster_position.iter().all(|value| value.is_finite()) {
            return Err(CastDenial::NonFiniteCaster);
        }
        match targeting {
            CastTargeting::Guid(guid) => {
                if guid == 0 {
                    return Err(CastDenial::UnknownTarget);
                }
                if guid == self.caster_guid {
                    return Err(CastDenial::SelfTarget);
                }
                let target = match target {
                    Some(target) if target.guid == guid => target,
                    _ => return Err(CastDenial::UnknownTarget),
                };
                if !target.position.iter().all(|value| value.is_finite()) {
                    return Err(CastDenial::NonFiniteTarget);
                }
                if target.dead {
                    return Err(CastDenial::DeadTarget);
                }
                if target.disposition != 0 {
                    return Err(CastDenial::NotHostileTarget);
                }
                if squared_distance(caster_position, target.position)
                    > WIZARD_AUTO_TARGET_REACH * WIZARD_AUTO_TARGET_REACH
                {
                    return Err(CastDenial::OutOfReach);
                }
            }
            CastTargeting::Position(position) => {
                if !position.iter().all(|value| value.is_finite()) {
                    return Err(CastDenial::NonFiniteTarget);
                }
            }
        }

        if now_ms < self.lock_until_ms {
            return Err(CastDenial::CastInProgress);
        }
        if slot == WizardSlot::Basic {
            if let Some(last) = self.last_basic_ms {
                if now_ms.saturating_sub(last) < WIZARD_BASIC_RECAST_MS {
                    return Err(CastDenial::Recast);
                }
            }
        }
        if self.energy < ability.energy_cost {
            return Err(CastDenial::InsufficientEnergy);
        }

        self.energy -= ability.energy_cost;
        if slot == WizardSlot::Basic {
            self.last_basic_ms = Some(now_ms);
        }
        self.lock_until_ms = now_ms.saturating_add(action_time_ms);
        Ok(WizardCast {
            ability_id: ability.id,
            targeting,
            damage: ability.damage,
            energy_cost: ability.energy_cost,
            animation_id: ability.animation_id,
            cast_effect_id: ability.cast_effect_id,
            hit_effect_id: ability.hit_effect_id,
            target_animation_id: ability.target_animation_id,
            energy_after: self.energy,
        })
    }
}

fn squared_distance(from: [f32; 3], to: [f32; 3]) -> f32 {
    from.iter()
        .zip(to.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum()
}
