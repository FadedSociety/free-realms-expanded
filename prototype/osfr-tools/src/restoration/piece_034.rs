//! Ninja progression (MMO work item 034): the recovered job and equipment tables, the weapon to
//! ability toolbar, and a bounded server-side validation of ability use.
//!
//! ORIGINAL facts this module is built on (all read outside this file and listed in
//! `notes/Research/Pieces/034 Ninja progression.md`):
//! * ability packets ride opcode 36 (`BaseAbilityPackets.AbilityPacketFailed = 1`,
//!   `AbilityPacketStartCasting = 3`, `AbilityPacketClientRequestStartAbility = 10`).
//! * the original denial phrases are locale ids 285 ("You don't have enough energy for that
//!   ability!"), 287 ("That ability isn't ready yet...") and 288 ("You need to choose a target!").
//!   The pinned community gateway answers an ability it will not service with string id 3079.
//! * the client's cast-precondition ladder ends in `NoCastRecastTimer`, which is the recast check
//!   this module enforces.
//! * profile 2 is "Ninja" (name id 3100, description id 3101, ability background set 3, hint
//!   384179, `Trial: true`), the creation kit is `[19522, 19431, 19197, 19787, 19253, 75091]`, and
//!   the character-creation weapon 75091 grants Lightning Strike (2154) and 1000 Storms (2050).
//! * the named Ninja weapons 75090-75119 gate on `MinProfileRank` 1/5/8/12/16 with `PowerRating`
//!   1..5, and the wiki weapon pages carry the damage ladder 254/889, 488/1554, 853/2716,
//!   1357/4750, 2372/8302.
//!
//! COMMUNITY-RECOVERED (pinned community resources, never original game data): the 4/7 unit
//! reaches, the 600 ms basic recast, the 100 energy pool regenerating 4 per second, and the
//! level-1 weapon mapping.
//!
//! RECONSTRUCTED here: the check order, the denial mapping, the per-ability cooldown bookkeeping
//! and the packet writers. Nothing here has been accepted by the native client.
//!
//! Explicit gaps, do not imply these:
//! * no original recast duration is recovered. Only the community 600 ms basic recast is applied;
//!   an ability without a recovered cooldown keeps `cooldown_ms: None` and the cast reports that
//!   its cooldown policy is unrecovered instead of inventing a timer.
//! * no original energy pool size or regeneration rate is recovered; both are community values.
//! * no original rank experience curve, trait effect or reward boundary is recovered.
//! * no denial string id is recovered for a rank gate or an unknown ability beyond the community
//!   blanket refusal, so those refusals carry no fabricated phrase id unless noted.
//! * only the level-1 weapon family has a recovered ability-id mapping; every other weapon falls
//!   back to the bare-hand basic and an empty special slot.
//! * cone, radius, line of sight, hit timing and damage application are outside this slice; the
//!   caller owns them.
//!
//! ponytail: one catalogue, one state map, no trait objects and no persistence layer. Add a
//! snapshot when the lead persists characters.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Schema version of the recovered catalogue document this module accepts.
pub const NINJA_CATALOGUE_SCHEMA_VERSION: u32 = 1;
/// ORIGINAL: the base opcode for every recovered ability packet.
pub const ABILITY_BASE_OPCODE: u16 = 36;
/// ORIGINAL: `BaseAbilityPackets.AbilityPacketFailed`.
pub const ABILITY_OP_FAILED: u16 = 1;
/// ORIGINAL: `BaseAbilityPackets.AbilityPacketStartCasting`.
pub const ABILITY_OP_START_CASTING: u16 = 3;
/// ORIGINAL: `BaseAbilityPackets.AbilityPacketClientRequestStartAbility`.
pub const ABILITY_OP_CLIENT_REQUEST_START_ABILITY: u16 = 10;
/// ORIGINAL: locale id for "You don't have enough energy for that ability!".
pub const DENIAL_INSUFFICIENT_ENERGY: i32 = 285;
/// ORIGINAL: locale id for "That ability isn't ready yet...".
pub const DENIAL_ABILITY_NOT_READY: i32 = 287;
/// ORIGINAL: locale id for "You need to choose a target!".  The recovered client ladder has no key
/// for it, so the module records the id without a precondition position.
pub const DENIAL_NEED_TARGET: i32 = 288;
/// COMMUNITY-RECOVERED: the pinned gateway's blanket refusal for an ability it will not service.
pub const DENIAL_CANNOT_USE_NOW: i32 = 3079;
/// ORIGINAL: profile 2 is Ninja in `Profiles.json`.
pub const NINJA_PROFILE_ID: i32 = 2;
/// ORIGINAL: the character-creation weapon of the Ninja kit.
pub const NINJA_TRAINING_SWORD_1000_STORMS: i32 = 75091;
/// ORIGINAL: the sibling rank 1 weapon whose ability pair has no recovered mapping.
pub const NINJA_TRAINING_SWORD_DRAGONSTRIKE: i32 = 75090;
/// COMMUNITY-RECOVERED: the bare-hand fallback basic for an unmapped weapon.
pub const NINJA_FALLBACK_BASIC_ABILITY_ID: i32 = 2010;
/// COMMUNITY-RECOVERED: the recovered basic ability of weapon 75091.
pub const LIGHTNING_STRIKE_ABILITY_ID: i32 = 2154;
/// COMMUNITY-RECOVERED: the recovered special ability of weapon 75091.
pub const THOUSAND_STORMS_ABILITY_ID: i32 = 2050;

/// The two recovered toolbar positions. The client's toolbar is positional: index 0 is the basic
/// attack, index 1 the special.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NinjaSlot {
    Basic,
    Special,
}

impl NinjaSlot {
    /// The positional toolbar index of this slot.
    pub fn index(self) -> u8 {
        match self {
            NinjaSlot::Basic => 0,
            NinjaSlot::Special => 1,
        }
    }

    /// The slot at a toolbar index, if any.
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(NinjaSlot::Basic),
            1 => Some(NinjaSlot::Special),
            _ => None,
        }
    }
}

/// A refusal from the bounded validation. Every variant is a local policy decision; only the
/// variants with a recovered phrase answer the client with a denial packet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NinjaRefusal {
    /// The acting profile is not the recovered Ninja profile id.
    NotNinjaJob { profile_id: i32 },
    /// The item definition is not a recovered Ninja weapon.
    UnknownWeapon { item_definition_id: i32 },
    /// The weapon's recovered `MinProfileRank` gate is above the acting rank.
    RankTooLow {
        item_definition_id: i32,
        required: i32,
        rank: i32,
    },
    /// The slot has no ability for the equipped weapon.
    UnknownAbility { slot: NinjaSlot },
    /// The ability is still inside its recovered recast window.
    NotReady { ability_id: i32, remaining_ms: u64 },
    /// The pool cannot pay the ability's recovered energy cost.
    InsufficientEnergy { required: i32, available: i32 },
    /// The catalogue the lead supplied is unusable.
    Corrupt(String),
}

impl NinjaRefusal {
    /// The recovered client phrase id for this refusal, when one exists.
    ///
    /// An unusable request (wrong job, unknown weapon, unknown ability) returns
    /// [`DENIAL_CANNOT_USE_NOW`], the phrase the pinned gateway answers with when it will not
    /// service an ability request. A rank gate carries no id at all, because no phrase was
    /// recovered for it.
    pub fn denial_string_id(&self) -> Option<i32> {
        match self {
            NinjaRefusal::NotReady { .. } => Some(DENIAL_ABILITY_NOT_READY),
            NinjaRefusal::InsufficientEnergy { .. } => Some(DENIAL_INSUFFICIENT_ENERGY),
            NinjaRefusal::NotNinjaJob { .. }
            | NinjaRefusal::UnknownWeapon { .. }
            | NinjaRefusal::UnknownAbility { .. } => Some(DENIAL_CANNOT_USE_NOW),
            NinjaRefusal::RankTooLow { .. } | NinjaRefusal::Corrupt(_) => None,
        }
    }
}

/// The accepted consequence of one ability use. Damage application, hit resolution and rewards
/// stay with the owning subsystems.
#[derive(Clone, Debug, PartialEq)]
pub struct NinjaCast {
    pub ability_id: i32,
    pub damage: i32,
    pub energy_cost: i32,
    pub energy_after: i32,
    /// The recast that will be enforced, or `None` when none is recovered.
    pub cooldown_ms: Option<u64>,
    /// Where the recast value came from.
    pub cooldown_source: String,
    /// ORIGINAL: the recovered animation id of the ability.
    pub animation_id: i32,
    /// ORIGINAL: the recovered cast effect id of the ability.
    pub cast_effect_id: i32,
}

/// The durable part of the progression state. It is plain data so the lead can persist it next to
/// a character without this module owning a file format.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NinjaState {
    /// ORIGINAL: the job rank the equipment gates compare against.
    pub rank: i32,
    /// The equipped weapon item definition id.
    pub weapon_definition_id: i32,
    /// COMMUNITY-RECOVERED pool, reconstructed bookkeeping.
    pub energy: i32,
    pub last_energy_ms: u64,
    /// The last accepted cast of each ability, in milliseconds.
    pub last_cast_ms: BTreeMap<i32, u64>,
}

#[derive(Clone, Debug, Deserialize)]
struct JobRow {
    profile_id: i32,
    name: String,
    #[serde(default)]
    name_id: i32,
    #[serde(default)]
    description_id: i32,
    #[serde(default)]
    hint_id: i32,
    #[serde(default)]
    members_only: bool,
    #[serde(default)]
    trial: bool,
    #[serde(default)]
    combat_members_only: bool,
    #[serde(default)]
    free_sample_level: i32,
}

/// ORIGINAL: one recovered Ninja weapon row. The level gate is the item definition's own
/// `MinProfileRank` and the tier is its `PowerRating`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct WeaponRow {
    pub item_definition_id: i32,
    #[serde(default)]
    pub name: String,
    pub min_profile_rank: i32,
    pub power_rating: i32,
}

/// ORIGINAL/COMMUNITY-RECOVERED: one recovered Ninja ability row.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct AbilityRow {
    pub ability_id: i32,
    #[serde(default)]
    pub name: String,
    pub damage: i32,
    #[serde(default)]
    pub energy_cost: i32,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub cooldown_ms: Option<u64>,
    #[serde(default)]
    pub cooldown_source: String,
    #[serde(default)]
    pub animation_id: i32,
    #[serde(default)]
    pub cast_effect_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct WeaponAbilities {
    weapon_definition_ids: Vec<i32>,
    basic_ability_id: i32,
    #[serde(default)]
    special_ability_id: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
struct CombatRow {
    #[serde(default)]
    auto_target_reach: f32,
    #[serde(default)]
    basic_auto_target_reach: Option<f32>,
    #[serde(default)]
    wield_type: i32,
    #[serde(default)]
    basic_recast_ms: u64,
    fallback_basic_ability_id: i32,
    energy_max: i32,
    energy_regen_per_second: i32,
    #[serde(default)]
    weapon_abilities: Vec<WeaponAbilities>,
}

#[derive(Clone, Debug, Deserialize)]
struct TierRow {
    min_profile_rank: i32,
    power_rating: i32,
}

#[derive(Deserialize)]
struct Document {
    schema_version: u32,
    job: JobRow,
    weapons: Vec<WeaponRow>,
    abilities: Vec<AbilityRow>,
    combat: CombatRow,
    #[serde(default)]
    canonical_weapon_tiers: Vec<TierRow>,
}

/// The validated Ninja catalogue plus the mutable progression state.
#[derive(Debug)]
pub struct NinjaProgression {
    schema_version: u32,
    job: JobRow,
    weapons: BTreeMap<i32, WeaponRow>,
    abilities: BTreeMap<i32, AbilityRow>,
    combat: CombatRow,
    canonical_tiers: Vec<(i32, i32)>,
    state: NinjaState,
}

fn invalid(message: impl Into<String>) -> NinjaRefusal {
    NinjaRefusal::Corrupt(message.into())
}

impl NinjaProgression {
    /// Parse and validate a recovered Ninja catalogue, starting at rank 1 with no weapon equipped
    /// and the community-recovered full energy pool.
    ///
    /// A document that does not match the recovered rules is refused rather than trusted.
    pub fn from_json(bytes: &[u8]) -> Result<Self, NinjaRefusal> {
        Self::parse(bytes, None)
    }

    /// Parse and validate a recovered Ninja catalogue and restore a saved state.
    pub fn from_json_with_state(bytes: &[u8], state: NinjaState) -> Result<Self, NinjaRefusal> {
        Self::parse(bytes, Some(state))
    }

    fn parse(bytes: &[u8], state: Option<NinjaState>) -> Result<Self, NinjaRefusal> {
        let document: Document = serde_json::from_slice(bytes)
            .map_err(|error| invalid(format!("unreadable catalogue: {error}")))?;
        if document.schema_version != NINJA_CATALOGUE_SCHEMA_VERSION {
            return Err(invalid(format!(
                "unsupported catalogue version {}",
                document.schema_version
            )));
        }
        if document.job.profile_id != NINJA_PROFILE_ID || document.job.name != "Ninja" {
            return Err(invalid(format!(
                "profile {} ({}) is not the recovered Ninja job",
                document.job.profile_id, document.job.name
            )));
        }
        if document.combat.energy_max <= 0 || document.combat.energy_regen_per_second <= 0 {
            return Err(invalid("the recovered energy pool is not positive"));
        }
        if document.weapons.is_empty() || document.abilities.is_empty() {
            return Err(invalid("the catalogue carries no weapons or abilities"));
        }

        let mut weapons = BTreeMap::new();
        for weapon in document.weapons {
            if weapon.item_definition_id <= 0
                || weapon.min_profile_rank < 1
                || weapon.power_rating < 1
            {
                return Err(invalid(format!(
                    "weapon {} carries an invalid rank or power",
                    weapon.item_definition_id
                )));
            }
            if weapons.insert(weapon.item_definition_id, weapon).is_some() {
                return Err(invalid("a weapon definition is repeated"));
            }
        }

        let mut abilities = BTreeMap::new();
        for ability in document.abilities {
            if ability.damage <= 0 {
                return Err(invalid(format!(
                    "ability {} carries no damage",
                    ability.ability_id
                )));
            }
            if ability.energy_cost < 0 || ability.energy_cost > document.combat.energy_max {
                return Err(invalid(format!(
                    "ability {} costs {} outside the recovered pool",
                    ability.ability_id, ability.energy_cost
                )));
            }
            if ability.cooldown_ms.is_some() && ability.cooldown_source.is_empty() {
                return Err(invalid(format!(
                    "ability {} carries a cooldown without a recovered source",
                    ability.ability_id
                )));
            }
            if abilities.insert(ability.ability_id, ability).is_some() {
                return Err(invalid("an ability is repeated"));
            }
        }

        for mapping in &document.combat.weapon_abilities {
            for weapon_id in &mapping.weapon_definition_ids {
                if !weapons.contains_key(weapon_id) {
                    return Err(invalid(format!(
                        "the ability mapping references unknown weapon {weapon_id}"
                    )));
                }
            }
            if !abilities.contains_key(&mapping.basic_ability_id) {
                return Err(invalid(
                    "the ability mapping references an unknown basic ability",
                ));
            }
            if let Some(special) = mapping.special_ability_id {
                if !abilities.contains_key(&special) {
                    return Err(invalid(
                        "the ability mapping references an unknown special ability",
                    ));
                }
            }
        }
        if !abilities.contains_key(&document.combat.fallback_basic_ability_id) {
            return Err(invalid("the recovered fallback basic ability is missing"));
        }

        let canonical_tiers = document
            .canonical_weapon_tiers
            .iter()
            .map(|tier| (tier.min_profile_rank, tier.power_rating))
            .collect::<Vec<_>>();

        let state = state.unwrap_or(NinjaState {
            rank: 1,
            weapon_definition_id: 0,
            energy: document.combat.energy_max,
            last_energy_ms: 0,
            last_cast_ms: BTreeMap::new(),
        });
        if state.rank < 1 || state.energy < 0 || state.energy > document.combat.energy_max {
            return Err(invalid("the saved state is outside the recovered bounds"));
        }
        if state.weapon_definition_id != 0 && !weapons.contains_key(&state.weapon_definition_id) {
            return Err(invalid("the saved state equips an unknown weapon"));
        }

        Ok(Self {
            schema_version: document.schema_version,
            job: document.job,
            weapons,
            abilities,
            combat: document.combat,
            canonical_tiers,
            state,
        })
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn profile_id(&self) -> i32 {
        self.job.profile_id
    }

    pub fn job_name(&self) -> &str {
        &self.job.name
    }

    pub fn name_id(&self) -> i32 {
        self.job.name_id
    }

    pub fn description_id(&self) -> i32 {
        self.job.description_id
    }

    pub fn hint_id(&self) -> i32 {
        self.job.hint_id
    }

    /// ORIGINAL: the profile row's own members-only flag (false for Ninja).
    pub fn members_only(&self) -> bool {
        self.job.members_only
    }

    /// ORIGINAL: the profile row's trial flag (true for Ninja).
    pub fn trial(&self) -> bool {
        self.job.trial
    }

    /// ORIGINAL: the wiki restriction - members-only for combat, free to sample through level 4.
    pub fn combat_members_only(&self) -> bool {
        self.job.combat_members_only
    }

    pub fn free_sample_level(&self) -> i32 {
        self.job.free_sample_level
    }

    pub fn weapon_count(&self) -> usize {
        self.weapons.len()
    }

    /// The recovered canonical equipment ladder as (rank, power) pairs.
    pub fn canonical_tiers(&self) -> Vec<(i32, i32)> {
        self.canonical_tiers.clone()
    }

    pub fn rank(&self) -> i32 {
        self.state.rank
    }

    /// The job rank the equipment gates compare against. The lead owns how a rank is earned.
    pub fn set_rank(&mut self, rank: i32) {
        self.state.rank = rank.max(1);
    }

    pub fn energy(&self) -> i32 {
        self.state.energy
    }

    pub fn state(&self) -> &NinjaState {
        &self.state
    }

    pub fn weapon_definition_id(&self) -> i32 {
        self.state.weapon_definition_id
    }

    /// Equip a recovered Ninja weapon. The recovered gate is the item definition's own
    /// `MinProfileRank`; a weapon that is not a recovered Ninja item is refused.
    pub fn equip(&mut self, item_definition_id: i32) -> Result<&WeaponRow, NinjaRefusal> {
        let weapon = self
            .weapons
            .get(&item_definition_id)
            .ok_or(NinjaRefusal::UnknownWeapon { item_definition_id })?;
        if self.state.rank < weapon.min_profile_rank {
            return Err(NinjaRefusal::RankTooLow {
                item_definition_id,
                required: weapon.min_profile_rank,
                rank: self.state.rank,
            });
        }
        self.state.weapon_definition_id = item_definition_id;
        Ok(self
            .weapons
            .get(&item_definition_id)
            .expect("the weapon was just resolved"))
    }

    /// The ability ids the equipped weapon resolves to: always a basic, plus a special when the
    /// weapon has a recovered mapping. An unmapped weapon falls back to the bare-hand basic.
    pub fn toolbar(&self) -> Result<Vec<i32>, NinjaRefusal> {
        let mut toolbar = vec![self.ability_id_for_slot(NinjaSlot::Basic)?];
        if let Some(special) = self.ability_id_for_slot(NinjaSlot::Special).ok() {
            toolbar.push(special);
        }
        Ok(toolbar)
    }

    fn ability_id_for_slot(&self, slot: NinjaSlot) -> Result<i32, NinjaRefusal> {
        let mapping = self.combat.weapon_abilities.iter().find(|mapping| {
            mapping
                .weapon_definition_ids
                .contains(&self.state.weapon_definition_id)
        });
        match slot {
            NinjaSlot::Basic => Ok(mapping
                .map(|mapping| mapping.basic_ability_id)
                .unwrap_or(self.combat.fallback_basic_ability_id)),
            NinjaSlot::Special => mapping
                .and_then(|mapping| mapping.special_ability_id)
                .ok_or(NinjaRefusal::UnknownAbility { slot }),
        }
    }

    /// The recovered ability behind a toolbar slot of the equipped weapon.
    pub fn ability(&self, slot: NinjaSlot) -> Option<&AbilityRow> {
        let ability_id = self.ability_id_for_slot(slot).ok()?;
        self.abilities.get(&ability_id)
    }

    /// The recovered basic reach / special reach in world units.
    pub fn reaches(&self) -> (f32, f32) {
        (
            self.combat
                .basic_auto_target_reach
                .unwrap_or(self.combat.auto_target_reach),
            self.combat.auto_target_reach,
        )
    }

    /// COMMUNITY-RECOVERED regeneration: whole seconds only, saturating at the recovered maximum.
    /// A non-monotonic timestamp grants nothing.
    pub fn regenerate(&mut self, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.state.last_energy_ms);
        let whole_seconds = elapsed / 1_000;
        if whole_seconds == 0 {
            return;
        }
        self.state.last_energy_ms += whole_seconds * 1_000;
        let gained = (whole_seconds as i64) * (self.combat.energy_regen_per_second as i64);
        self.state.energy =
            (self.state.energy as i64 + gained).min(self.combat.energy_max as i64) as i32;
    }

    /// Validate one ability use and, if accepted, spend its energy and stamp its recast.
    ///
    /// Checks run before the first mutation, so a refusal leaves the state untouched. Only the
    /// recovered job/rank/cooldown/energy gates are applied; targeting, reach and damage stay with
    /// the caller.
    pub fn cast(
        &mut self,
        profile_id: i32,
        slot: NinjaSlot,
        now_ms: u64,
    ) -> Result<NinjaCast, NinjaRefusal> {
        if profile_id != self.job.profile_id {
            return Err(NinjaRefusal::NotNinjaJob { profile_id });
        }
        let ability_id = self.ability_id_for_slot(slot)?;
        let (damage, energy_cost, cooldown_ms, cooldown_source, animation_id, cast_effect_id) = {
            let ability = self
                .abilities
                .get(&ability_id)
                .ok_or(NinjaRefusal::UnknownAbility { slot })?;
            (
                ability.damage,
                ability.energy_cost,
                ability.cooldown_ms,
                ability.cooldown_source.clone(),
                ability.animation_id,
                ability.cast_effect_id,
            )
        };
        self.regenerate(now_ms);
        if self.state.energy < energy_cost {
            return Err(NinjaRefusal::InsufficientEnergy {
                required: energy_cost,
                available: self.state.energy,
            });
        }
        // ORIGINAL: the client's precondition ladder ends in `NoCastRecastTimer`, so the recast
        // check runs last. An ability whose recast is unrecovered has no timer to check.
        if let Some(cooldown) = cooldown_ms {
            if let Some(last) = self.state.last_cast_ms.get(&ability_id) {
                let ready_at = last.saturating_add(cooldown);
                if now_ms < ready_at {
                    return Err(NinjaRefusal::NotReady {
                        ability_id,
                        remaining_ms: ready_at - now_ms,
                    });
                }
            }
        }

        self.state.energy -= energy_cost;
        if cooldown_ms.is_some() {
            self.state.last_cast_ms.insert(ability_id, now_ms);
        }
        Ok(NinjaCast {
            ability_id,
            damage,
            energy_cost,
            energy_after: self.state.energy,
            cooldown_ms,
            cooldown_source,
            animation_id,
            cast_effect_id,
        })
    }

    /// The recovered denial payload for a refusal, or `None` when no phrase id is recovered.
    pub fn denial_packet(&self, refusal: &NinjaRefusal) -> Option<Vec<u8>> {
        let string_id = refusal.denial_string_id()?;
        Some(denial_payload(string_id))
    }

    /// The recovered phrase id this refusal answers with, if any.
    pub fn denial_string_id(&self, refusal: &NinjaRefusal) -> Option<i32> {
        refusal.denial_string_id()
    }

    /// `AbilityPacketStartCasting`: the recovered cast feedback. `action_time_ms` is the caller's
    /// cast duration, which is not recovered.
    pub fn start_casting_packet(
        &self,
        caster_guid: u64,
        target_guid: u64,
        cast: &NinjaCast,
        action_time_ms: u64,
    ) -> Vec<u8> {
        let mut bytes = header(ABILITY_BASE_OPCODE, ABILITY_OP_START_CASTING);
        bytes.extend(caster_guid.to_le_bytes());
        bytes.extend(target_guid.to_le_bytes());
        bytes.extend(cast.cast_effect_id.to_le_bytes());
        bytes.extend(cast.animation_id.to_le_bytes());
        bytes.extend(cast.ability_id.to_le_bytes());
        bytes.extend((action_time_ms as f32 / 1_000.0).to_le_bytes());
        bytes.push(u8::from(action_time_ms > 0));
        bytes
    }
}

/// `AbilityPacketFailed`: the ability opcode, sub opcode 1 and the recovered phrase id.
pub fn denial_payload(string_id: i32) -> Vec<u8> {
    let mut bytes = header(ABILITY_BASE_OPCODE, ABILITY_OP_FAILED);
    bytes.extend(string_id.to_le_bytes());
    bytes
}

fn header(opcode: u16, sub: u16) -> Vec<u8> {
    [opcode.to_le_bytes(), sub.to_le_bytes()].concat()
}
