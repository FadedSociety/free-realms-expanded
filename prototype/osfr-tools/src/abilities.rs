//! Small, data-driven ability resolver. Packets here are INNER gateway messages.
//!
//! The client readers establish the wire layouts; damage, costs, timers, failure routing,
//! status stacking and cast completion timing remain reconstructed policy (R).

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
};

#[allow(dead_code)]
#[path = "restoration/piece_056.rs"]
pub(crate) mod effect_packets;

const MAX_JSON_BYTES: u64 = 1_048_576;
const MAX_ABILITIES: usize = 512;
const MAX_EFFECTS_PER_ABILITY: usize = 32;
const KNOCKED_OUT_BIT: i32 = 1 << 7;

/// (R) Require the job's weapon class in native Weapon slot 7. No supported
/// combat cast is weapon-free. Classes come from pinned Sanctuary
/// Resources/ItemClasses.txt (reference); the client supplies wield families.
/// per-skill requirements are not recovered from AbilityDefinition. The join
/// lives on `Inventory::equipped_weapon_for_class`, so the T521 toolbar label
/// and this cast gate read the same live definition bytes.
pub fn equipped_weapon_matches(
    inventory: &crate::inventory::Inventory,
    profile: i32,
    required_class: Option<i32>,
) -> bool {
    required_class
        .filter(|class| *class > 0)
        .is_some_and(|class| {
            inventory
                .equipped_weapon_for_class(profile, class)
                .is_some()
        })
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sourced<T> {
    pub value: T,
    pub source: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    SelfOnly,
    Ally,
    SelfOrAlly,
    Hostile,
    Any,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    Damage,
    Heal,
    ApplyEffect,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectRecipient {
    Caster,
    Target,
}

/// Extra effects consumed by the routed encounter executor. All numeric
/// tuning and stacking rules are reconstructed (R), documented in content.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum JobEffect {
    AreaDamage {
        radius: f32,
        max_targets: usize,
        centered_on_caster: bool,
    },
    DamageOverTime {
        multiplier: f32,
        duration_ms: u64,
        interval_ms: u64,
    },
    Stun {
        duration_ms: u64,
    },
    Buff {
        recipient: EffectRecipient,
        duration_ms: u64,
        damage_multiplier: f32,
        damage_taken_multiplier: f32,
        crit_bonus: u32,
        crit_damage_bonus: f32,
    },
}

impl JobEffect {
    pub fn valid(&self) -> bool {
        match *self {
            Self::AreaDamage {
                radius,
                max_targets,
                ..
            } => (0.1..=30.0).contains(&radius) && (1..=16).contains(&max_targets),
            Self::DamageOverTime {
                multiplier,
                duration_ms,
                interval_ms,
            } => {
                (0.01..=10.0).contains(&multiplier)
                    && (500..=30_000).contains(&duration_ms)
                    && (500..=duration_ms).contains(&interval_ms)
            }
            Self::Stun { duration_ms } => (1..=10_000).contains(&duration_ms),
            Self::Buff {
                duration_ms,
                damage_multiplier,
                damage_taken_multiplier,
                crit_bonus,
                crit_damage_bonus,
                ..
            } => {
                (1..=30_000).contains(&duration_ms)
                    && (0.1..=3.0).contains(&damage_multiplier)
                    && (0.1..=3.0).contains(&damage_taken_multiplier)
                    && crit_bonus <= 50
                    && (0.0..=1.0).contains(&crit_damage_bonus)
            }
        }
    }

    pub fn needs_target(&self) -> bool {
        !matches!(
            self,
            Self::Buff {
                recipient: EffectRecipient::Caster,
                ..
            }
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AbilityEffect {
    pub kind: Sourced<EffectKind>,
    pub recipient: Sourced<EffectRecipient>,
    #[serde(default)]
    pub amount: Option<Sourced<i32>>,
    #[serde(default)]
    pub effect_id: Option<Sourced<i32>>,
    #[serde(default)]
    pub duration_ms: Option<Sourced<u64>>,
    #[serde(default)]
    pub icon_id: Option<Sourced<i32>>,
    #[serde(default)]
    pub name_id: Option<Sourced<i32>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AbilityDefinition {
    pub id: Sourced<i32>,
    pub name: Sourced<String>,
    pub job_profile: Sourced<i32>,
    /// Zero-based action-bar slot.
    pub slot: Sourced<i32>,
    pub target_kind: Sourced<TargetKind>,
    pub range: Sourced<f32>,
    pub cost: Sourced<i32>,
    pub cooldown_ms: Sourced<u64>,
    pub cast_time_ms: Sourced<u64>,
    pub effects: Vec<AbilityEffect>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AbilityFile {
    schema_version: Sourced<u32>,
    abilities: Vec<AbilityDefinition>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AbilityCatalog {
    abilities: BTreeMap<i32, AbilityDefinition>,
}

impl AbilityCatalog {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_JSON_BYTES + 1)
            .read_to_end(&mut bytes)?;
        Self::from_json(&bytes)
    }

    pub fn from_json(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() as u64 > MAX_JSON_BYTES {
            return Err(invalid("ability catalogue exceeds size limit"));
        }
        let file: AbilityFile =
            serde_json::from_slice(bytes).map_err(|error| invalid(&error.to_string()))?;
        if file.schema_version.value != 1
            || !source_ok(&file.schema_version)
            || file.abilities.len() > MAX_ABILITIES
        {
            return Err(invalid("unsupported or oversized ability catalogue"));
        }

        let mut abilities = BTreeMap::new();
        for ability in file.abilities {
            validate_definition(&ability)?;
            if abilities.insert(ability.id.value, ability).is_some() {
                return Err(invalid("duplicate ability id"));
            }
        }
        Ok(Self { abilities })
    }

    pub fn get(&self, id: i32) -> Option<&AbilityDefinition> {
        self.abilities.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &AbilityDefinition> {
        self.abilities.values()
    }
}

fn validate_definition(ability: &AbilityDefinition) -> io::Result<()> {
    if ability.id.value <= 0
        || ability.job_profile.value <= 0
        || !(0..8).contains(&ability.slot.value)
        || !ability.range.value.is_finite()
        || ability.range.value < 0.0
        || ability.cost.value < 0
        || ability.cooldown_ms.value > 86_400_000
        || ability.cast_time_ms.value > 86_400_000
        || ability.effects.len() > MAX_EFFECTS_PER_ABILITY
        || !source_ok(&ability.id)
        || !source_ok(&ability.name)
        || ability.name.value.trim().is_empty()
        || !source_ok(&ability.job_profile)
        || !source_ok(&ability.slot)
        || !source_ok(&ability.target_kind)
        || !source_ok(&ability.range)
        || !source_ok(&ability.cost)
        || !source_ok(&ability.cooldown_ms)
        || !source_ok(&ability.cast_time_ms)
    {
        return Err(invalid("invalid ability definition"));
    }

    for effect in &ability.effects {
        if !source_ok(&effect.kind) || !source_ok(&effect.recipient) {
            return Err(invalid("ability effect is missing provenance"));
        }
        match effect.kind.value {
            EffectKind::Damage | EffectKind::Heal => {
                effect
                    .amount
                    .as_ref()
                    .filter(|value| source_ok(value) && value.value > 0)
                    .ok_or_else(|| invalid("damage/heal effect needs a positive amount"))?;
            }
            EffectKind::ApplyEffect => {
                let id = effect
                    .effect_id
                    .as_ref()
                    .filter(|value| source_ok(value) && value.value > 0)
                    .ok_or_else(|| invalid("apply-effect needs a positive effect id"))?;
                let duration = effect
                    .duration_ms
                    .as_ref()
                    .filter(|value| source_ok(value) && (1..=86_400_000).contains(&value.value))
                    .ok_or_else(|| invalid("apply-effect needs a bounded duration"))?;
                let icon = effect
                    .icon_id
                    .as_ref()
                    .filter(|value| source_ok(value) && value.value >= 0)
                    .ok_or_else(|| invalid("apply-effect needs an icon id"))?;
                let name = effect
                    .name_id
                    .as_ref()
                    .filter(|value| source_ok(value) && value.value >= 0)
                    .ok_or_else(|| invalid("apply-effect needs a name id"))?;
                let _ = (id, duration, icon, name);
            }
        }
    }
    Ok(())
}

fn source_ok<T>(value: &Sourced<T>) -> bool {
    !value.source.trim().is_empty()
        && value.source.len() <= 512
        && !value.source.chars().any(char::is_control)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorKind {
    Player,
    Hostile,
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RespawnWindow {
    pub timer_ms: i32,
    pub revive_cost: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveEffect {
    pub tag_id: i32,
    pub ability_id: i32,
    pub expires_at_ms: u64,
}

/// Mutable state for one actor. Cooldown keys are zero-based toolbar slots.
#[derive(Clone, Debug, PartialEq)]
pub struct CombatState {
    pub guid: u64,
    pub kind: ActorKind,
    pub job_profile: i32,
    pub position: [f32; 3],
    pub health: i32,
    pub max_health: i32,
    pub energy: i32,
    pub max_energy: i32,
    pub owned_abilities: BTreeSet<i32>,
    pub slots: BTreeMap<i32, i32>,
    pub cooldown_until_ms: BTreeMap<i32, u64>,
    pub casting_until_ms: Option<u64>,
    pub active_effects: BTreeMap<i32, ActiveEffect>,
    /// Encounter values come from the instance policy (R); `None` means no encounter window.
    pub respawn_window: Option<RespawnWindow>,
}

impl CombatState {
    pub fn new(
        guid: u64,
        kind: ActorKind,
        job_profile: i32,
        position: [f32; 3],
        health: i32,
        max_health: i32,
        energy: i32,
        max_energy: i32,
    ) -> Self {
        Self {
            guid,
            kind,
            job_profile,
            position,
            health,
            max_health,
            energy,
            max_energy,
            owned_abilities: BTreeSet::new(),
            slots: BTreeMap::new(),
            cooldown_until_ms: BTreeMap::new(),
            casting_until_ms: None,
            active_effects: BTreeMap::new(),
            respawn_window: None,
        }
    }
}

pub enum AbilityTarget<'a> {
    SelfTarget,
    Actor(&'a mut CombatState),
    Missing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbilityFailure {
    NotOwned,
    InvalidDefinition,
    SlotMismatch,
    WrongJob,
    CasterKnockedOut,
    MissingTarget,
    InvalidTarget,
    OutOfRange,
    InsufficientEnergy,
    Cooldown,
    AlreadyCasting,
}

impl AbilityFailure {
    /// Client locale ids are recovered; mapping server-only failures to the generic line is (R).
    pub const fn client_message_id(self) -> i32 {
        match self {
            Self::InsufficientEnergy => 285, // locale: "You don't have enough energy for that ability!"
            Self::Cooldown => 287,           // locale: "That ability isn't ready yet..."
            Self::MissingTarget => 288,      // locale: "You need to choose a target!"
            Self::NotOwned
            | Self::InvalidDefinition
            | Self::SlotMismatch
            | Self::WrongJob
            | Self::CasterKnockedOut
            | Self::InvalidTarget
            | Self::OutOfRange
            | Self::AlreadyCasting => 286, // locale: "You can't use abilities right now." (R mapping)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AbilityOutcome {
    pub failure: Option<AbilityFailure>,
    pub client_failure_code: Option<i32>,
    pub packets: Vec<Vec<u8>>,
}

impl AbilityOutcome {
    fn failed(failure: AbilityFailure) -> Self {
        let code = failure.client_message_id();
        Self {
            failure: Some(failure),
            client_failure_code: Some(code),
            packets: vec![build_failure(code)],
        }
    }

    fn succeeded(packets: Vec<Vec<u8>>) -> Self {
        Self {
            failure: None,
            client_failure_code: None,
            packets,
        }
    }
}

pub fn use_ability(
    caster: &mut CombatState,
    target: AbilityTarget<'_>,
    ability: &AbilityDefinition,
    now_ms: u64,
) -> AbilityOutcome {
    if validate_definition(ability).is_err() {
        return AbilityOutcome::failed(AbilityFailure::InvalidDefinition);
    }
    let target_view = match &target {
        AbilityTarget::SelfTarget => Some(&*caster),
        AbilityTarget::Actor(actor) => Some(&**actor),
        AbilityTarget::Missing => None,
    };
    let self_target = matches!(&target, AbilityTarget::SelfTarget);
    if let Some(failure) = validate_use(caster, target_view, self_target, ability, now_ms) {
        return AbilityOutcome::failed(failure);
    }

    let caster_guid = caster.guid;
    let target_guid = target_view.map_or(caster.guid, |actor| actor.guid);
    let mut packets = Vec::new();
    if ability.cast_time_ms.value > 0 {
        packets.push(build_cast_start(caster.guid, target_guid, ability));
    }

    // (R) This slice's response order is cast, combat/health, energy, then effect-tag adds.
    // (R) Charge and begin cooldown at request acceptance; delayed cast completion is not modeled here.
    caster.energy -= ability.cost.value;
    caster.cooldown_until_ms.insert(
        ability.slot.value,
        now_ms.saturating_add(ability.cooldown_ms.value),
    );
    caster.casting_until_ms =
        (ability.cast_time_ms.value > 0).then(|| now_ms.saturating_add(ability.cast_time_ms.value));

    let (results, effect_adds) = match target {
        AbilityTarget::SelfTarget => apply_effects_to_one(caster, caster_guid, ability, now_ms),
        AbilityTarget::Actor(actor) => apply_effects_to_pair(caster, actor, ability, now_ms),
        AbilityTarget::Missing => unreachable!("validated target"),
    };
    packets.extend(results);
    if ability.cost.value > 0 {
        packets.push(build_energy(caster.energy, caster.max_energy));
    }
    packets.extend(effect_adds);
    AbilityOutcome::succeeded(packets)
}

fn validate_use(
    caster: &CombatState,
    target: Option<&CombatState>,
    self_target: bool,
    ability: &AbilityDefinition,
    now_ms: u64,
) -> Option<AbilityFailure> {
    // (R) The client guide requires these checks, but the original server validation order is absent.
    if !caster.owned_abilities.contains(&ability.id.value) {
        return Some(AbilityFailure::NotOwned);
    }
    if caster.slots.get(&ability.slot.value) != Some(&ability.id.value) {
        return Some(AbilityFailure::SlotMismatch);
    }
    if caster.job_profile != ability.job_profile.value {
        return Some(AbilityFailure::WrongJob);
    }
    if caster.health <= 0 {
        return Some(AbilityFailure::CasterKnockedOut);
    }
    let Some(target) = target else {
        return Some(AbilityFailure::MissingTarget);
    };
    if target.health <= 0
        || target.guid == 0
        || (!self_target && target.guid == caster.guid)
        || !valid_target_kind(ability.target_kind.value, target.kind, self_target)
    {
        return Some(AbilityFailure::InvalidTarget);
    }
    if !in_range(caster, target, self_target, ability.range.value) {
        return Some(AbilityFailure::OutOfRange);
    }
    if caster.energy < ability.cost.value {
        return Some(AbilityFailure::InsufficientEnergy);
    }
    if caster
        .cooldown_until_ms
        .get(&ability.slot.value)
        // (R) Server deadlines use monotonic milliseconds and expire at the exact deadline.
        .is_some_and(|until| now_ms < *until)
    {
        return Some(AbilityFailure::Cooldown);
    }
    if caster.casting_until_ms.is_some_and(|until| now_ms < until) {
        return Some(AbilityFailure::AlreadyCasting);
    }
    None
}

fn valid_target_kind(kind: TargetKind, actor: ActorKind, self_target: bool) -> bool {
    // (R) This compact actor-kind mapping is not an original selector table.
    match kind {
        TargetKind::SelfOnly => self_target,
        TargetKind::Ally => !self_target && actor == ActorKind::Player,
        TargetKind::SelfOrAlly => self_target || actor == ActorKind::Player,
        TargetKind::Hostile => !self_target && actor == ActorKind::Hostile,
        TargetKind::Any => true,
    }
}

fn in_range(caster: &CombatState, target: &CombatState, self_target: bool, range: f32) -> bool {
    // (R) Euclidean 3D distance is the local range rule; native target selectors are only partly decoded.
    if self_target {
        return true;
    }
    if !range.is_finite()
        || caster
            .position
            .iter()
            .chain(target.position.iter())
            .any(|v| !v.is_finite())
    {
        return false;
    }
    let distance_squared = caster
        .position
        .iter()
        .zip(target.position)
        .map(|(from, to)| {
            let delta = f64::from(*from) - f64::from(to);
            delta * delta
        })
        .sum::<f64>();
    distance_squared <= f64::from(range) * f64::from(range)
}

fn apply_effects_to_one(
    actor: &mut CombatState,
    caster_guid: u64,
    ability: &AbilityDefinition,
    now_ms: u64,
) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let mut results = Vec::new();
    let mut additions = Vec::new();
    for effect in &ability.effects {
        apply_one(
            actor,
            caster_guid,
            ability.id.value,
            effect,
            now_ms,
            &mut results,
            &mut additions,
        );
    }
    (results, additions)
}

fn apply_effects_to_pair(
    caster: &mut CombatState,
    target: &mut CombatState,
    ability: &AbilityDefinition,
    now_ms: u64,
) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let mut results = Vec::new();
    let mut additions = Vec::new();
    let caster_guid = caster.guid;
    for effect in &ability.effects {
        let actor = match effect.recipient.value {
            EffectRecipient::Caster => &mut *caster,
            EffectRecipient::Target => &mut *target,
        };
        apply_one(
            actor,
            caster_guid,
            ability.id.value,
            effect,
            now_ms,
            &mut results,
            &mut additions,
        );
    }
    (results, additions)
}

fn apply_one(
    actor: &mut CombatState,
    caster_guid: u64,
    ability_id: i32,
    effect: &AbilityEffect,
    now_ms: u64,
    results: &mut Vec<Vec<u8>>,
    additions: &mut Vec<Vec<u8>>,
) {
    match effect.kind.value {
        EffectKind::Damage => {
            let damage = effect.amount.as_ref().expect("catalogue validated").value;
            let previous = actor.health;
            // (R) Direct damage subtracts a flat authored amount and clamps at zero.
            actor.health = actor.health.saturating_sub(damage).max(0);
            let applied = previous - actor.health;
            if applied > 0 {
                results.push(build_attack_processed(
                    caster_guid,
                    actor.guid,
                    applied,
                    actor.health,
                    actor.max_health,
                ));
                if actor.kind == ActorKind::Player && actor.health == 0 {
                    // The client uses bit 7 for knockout; zero-health transition and window policy are (R).
                    results.push(build_character_state(actor.guid, KNOCKED_OUT_BIT));
                    results.push(build_health(actor.health, actor.max_health));
                    if let Some(window) = actor.respawn_window {
                        results.push(build_respawn_window(window));
                    }
                } else {
                    results.push(build_actor_health(actor));
                }
            }
        }
        EffectKind::Heal => {
            let amount = effect.amount.as_ref().expect("catalogue validated").value;
            let previous = actor.health;
            // (R) Flat healing clamps at the actor's maximum.
            actor.health = actor.health.saturating_add(amount).min(actor.max_health);
            let applied = actor.health - previous;
            if applied > 0 {
                results.push(build_hitpoint_delta(caster_guid, actor.guid, applied));
                results.push(build_actor_health(actor));
            }
        }
        EffectKind::ApplyEffect if actor.health > 0 => {
            let effect_id = effect
                .effect_id
                .as_ref()
                .expect("catalogue validated")
                .value;
            let duration_ms = effect
                .duration_ms
                .as_ref()
                .expect("catalogue validated")
                .value;
            let icon_id = effect.icon_id.as_ref().expect("catalogue validated").value;
            let name_id = effect.name_id.as_ref().expect("catalogue validated").value;
            // R: a definition id doubles as the stable per-actor tag id; reapplication refreshes it.
            actor.active_effects.insert(
                effect_id,
                ActiveEffect {
                    tag_id: effect_id,
                    ability_id,
                    expires_at_ms: now_ms.saturating_add(duration_ms),
                },
            );
            additions.push(build_effect_tag(effect_id, duration_ms, icon_id, name_id));
        }
        EffectKind::ApplyEffect => {}
    }
}

/// Remove expired tags and return their ordered 38/17 inner messages.
/// (R) Expiry is a one-shot removal when `now_ms` reaches the stored deadline.
pub fn tick_effects(actor: &mut CombatState, now_ms: u64) -> Vec<Vec<u8>> {
    let expired: Vec<i32> = actor
        .active_effects
        .iter()
        .filter_map(|(&tag_id, effect)| (now_ms >= effect.expires_at_ms).then_some(tag_id))
        .collect();
    expired
        .into_iter()
        .filter_map(|tag_id| {
            actor.active_effects.remove(&tag_id)?;
            Some(effect_packets::encode_remove_effect_tag(tag_id))
        })
        .collect()
}

fn build_cast_start(caster: u64, target: u64, ability: &AbilityDefinition) -> Vec<u8> {
    // Client family-36 case 3 parses through `00a31aa0`/`00a2fc70`.
    let mut bytes = header(36, 3);
    bytes.extend(caster.to_le_bytes());
    // The second identifier's target role is a local assignment pending native call-site proof (R).
    bytes.extend(target.to_le_bytes());
    // 36/3 carries four dwords; ability id, animation and effect ids are (R) placeholders.
    put_i32(&mut bytes, ability.id.value);
    put_i32(&mut bytes, 0); // (R) no recovered animation mapping for this authored ability
    put_i32(&mut bytes, 0); // (R) no recovered composite-effect mapping
    bytes.extend((ability.cast_time_ms.value as f32 / 1000.0).to_le_bytes());
    bytes.push(0); // (R) trailing flag
    bytes
}

pub(crate) fn build_failure(message_id: i32) -> Vec<u8> {
    // Client family-36 case 1 reads the string id with `00a31470`.
    let mut bytes = header(36, 1);
    put_i32(&mut bytes, message_id);
    bytes
}

/// (R) Client string 286 ("You can't use abilities right now.") answers any
/// cast this server cannot serve; the existing refusals already use it.
const UNSUPPORTED_CAST_MESSAGE_ID: i32 = 286;
/// Length of every 36/13 definition the server sends (see `OpeningAbilities::activation`).
const DEFINITION_LEN: usize = 160;

pub fn unsupported_cast_failure() -> Vec<u8> {
    build_failure(UNSUPPORTED_CAST_MESSAGE_ID)
}

/// T889: what a job without a basic attack hears when it tries to fight, on the solo path and on
/// the shared field alike. The field menu's Attack choice (26/10) gets a chat line (R); a 36/10
/// cast gets the client's own cast failure.
pub fn non_combat_refusal(from_menu: bool) -> Vec<u8> {
    if from_menu {
        crate::chat::system("Switch to a combat job before attacking.")
    } else {
        unsupported_cast_failure()
    }
}

/// Parse a client 36/12 request: header plus one `i32` ability id. The native
/// writer `00a34410` (via `00a32eb0`) emits exactly the int16 pair and a dword
/// on a definition-cache miss (`00a34d10`). Any other length is malformed.
pub fn parse_definition_request(inner: &[u8]) -> io::Result<i32> {
    match inner {
        [36, 0, 12, 0, a, b, c, d] => Ok(i32::from_le_bytes([*a, *b, *c, *d])),
        _ => Err(invalid("malformed ability definition request")),
    }
}

/// Answer a 36/12 request from the 36/13 definitions already built for the
/// player's toolbar (`OpeningAbilities::activation`, native-accepted layout
/// `00a32930`). At most one message is returned; an id that is not in `loaded`
/// (unknown, another job's, or above the player's level) yields `None`, so a
/// forged id costs one scan of a handful of messages and leaks nothing. (R)
/// Silence on a miss: the client queues the id once and never retries
/// (`00a34d10`), and a failure string would show on a mere tooltip hover.
pub fn definition_reply(loaded: &[Vec<u8>], id: i32) -> Option<Vec<u8>> {
    if id <= 0 {
        return None;
    }
    loaded
        .iter()
        .find(|m| {
            m.len() == DEFINITION_LEN
                && m.starts_with(&[36, 0, 13, 0])
                && m[4..8] == id.to_le_bytes()
        })
        .cloned()
}

/// Classify a client 36/10 start-ability request this server cannot serve.
/// Writer `00a342e0`: bar, slot, then a mode dword; mode 0 carries one `u64`
/// target id (24 bytes in all), any other mode four dwords (ground/pulse
/// position, 32 bytes). (R) Only mode 0 on bar 1, slots 0..=7 can be served;
/// ground modes and other bars (the item bar is a separate build) get the
/// failure packet instead of silence. Pure: it never touches combat state, so
/// a refusal costs no energy, cooldown or damage. `Ok(None)` means "not 36/10
/// or supported shape: dispatch as before"; a wrong length is malformed.
pub fn unsupported_cast(inner: &[u8]) -> io::Result<Option<Vec<u8>>> {
    if !inner.starts_with(&[36, 0, 10, 0]) {
        return Ok(None);
    }
    let word = |at: usize| {
        inner
            .get(at..at + 4)
            .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
            .ok_or_else(|| invalid("truncated ability cast request"))
    };
    let (bar, slot, mode) = (word(4)?, word(8)?, word(12)?);
    if inner.len() != if mode == 0 { 24 } else { 32 } {
        return Err(invalid("malformed ability cast request"));
    }
    let served = mode == 0 && bar == 1 && (0..=7).contains(&slot);
    Ok((!served).then(unsupported_cast_failure))
}

fn build_attack_processed(
    attacker: u64,
    target: u64,
    damage: i32,
    target_health: i32,
    target_max_health: i32,
) -> Vec<u8> {
    // Client reader 00a2a910 and handler 00a2ba40 consume this result shape.
    let mut bytes = header(32, 7);
    for guid in [attacker, attacker, target] {
        bytes.extend(guid.to_le_bytes());
    }
    for value in [damage, target_max_health, 0] {
        put_i32(&mut bytes, value); // effect id zero is (R), no mapping in this slice
    }
    bytes.extend([0, 0]); // (R) critical and reaction flags are clear
    put_i32(&mut bytes, 0); // (R) uninterpreted client result field
    put_i32(&mut bytes, target_health);
    bytes
}

fn build_actor_health(actor: &CombatState) -> Vec<u8> {
    if actor.kind == ActorKind::Player {
        build_health(actor.health, actor.max_health)
    } else {
        // Client family-35 sub-5 reader `008d6f10` consumes actor id and three integers.
        let mut bytes = header(35, 5);
        bytes.extend(actor.guid.to_le_bytes());
        for value in [actor.health, actor.max_health, 0] {
            put_i32(&mut bytes, value); // final zero remains (R)
        }
        bytes
    }
}

fn build_health(current: i32, maximum: i32) -> Vec<u8> {
    // Family-38 dispatcher `009392c0`, sub 1 reader `0089d2b0`.
    let mut bytes = header(38, 1);
    put_i32(&mut bytes, current);
    put_i32(&mut bytes, maximum);
    bytes
}

pub(crate) fn build_energy(current: i32, maximum: i32) -> Vec<u8> {
    // Family-38 dispatcher `009392c0`, sub 13 reader `0089d360`.
    let mut bytes = header(38, 13);
    put_i32(&mut bytes, current);
    put_i32(&mut bytes, maximum); // (R) the second dword's meaning is not decoded; this model sends max energy
    bytes.push(0); // (R) client flag is not semantically decoded in this slice
    bytes
}

pub(crate) fn build_hitpoint_delta(source: u64, target: u64, delta: i32) -> Vec<u8> {
    // Client family-35 reader `008d6c50` consumes two ids, flag, three integers, flag.
    let mut bytes = header(35, 35);
    bytes.extend(source.to_le_bytes());
    bytes.extend(target.to_le_bytes());
    bytes.push(1); // The client requires the display flag for healing/damage-over-time numbers.
    put_i32(&mut bytes, delta); // (R) this slice places the signed delta in the first integer field
    put_i32(&mut bytes, 0); // (R) secondary delta fields
    put_i32(&mut bytes, 0);
    bytes.push(0); // (R) trailing flag
    bytes
}

fn build_character_state(guid: u64, flags: i32) -> Vec<u8> {
    // Client family-35 dispatcher `0092f460`, sub 20, reads actor id and flag dword.
    let mut bytes = header(35, 20);
    bytes.extend(guid.to_le_bytes());
    put_i32(&mut bytes, flags);
    bytes
}

fn build_respawn_window(window: RespawnWindow) -> Vec<u8> {
    // Client family-41 dispatcher `0090d830` routes sub 125 to timer/cost display `00904300`.
    let mut bytes = header(41, 125);
    put_i32(&mut bytes, window.timer_ms);
    put_i32(&mut bytes, window.revive_cost);
    bytes
}

pub(crate) fn build_effect_tag(
    tag_id: i32,
    duration_ms: u64,
    icon_id: i32,
    name_id: i32,
) -> Vec<u8> {
    // Client family-38 dispatcher `009392c0`, sub 16 reader `008d8870` reads the tag id, then an
    // i32 length and the 85-byte tag blob (`008bfc00`). Unknown tag members stay zero (R).
    let duration_seconds = duration_ms.saturating_add(999) / 1000; // (R) round up for client display
    let tag = effect_packets::EffectTagPayload {
        unknown_1: tag_id, // the client's instance id; one buff-bar icon per tag
        unknown_2: tag_id, // (R) carries the authored effect-definition id
        unknown_3: 0,
        unknown_4: 0,
        unknown_5: 0,
        unknown_6: 0,
        unknown_7: i32::try_from(duration_seconds).unwrap_or(i32::MAX),
        unknown_8: false,
        unknown_9: 0,
        unknown_10: 0,
        unknown_11: 0,
        unknown_12: 0,
        composite_effect_id: 0, // (R) no composite effect assigned
        unknown_14: 0,
        unknown_15: 0,
        unknown_16: 0,
        unknown_17: false,
        unknown_18: false,
        unknown_19: false,
        unknown_20: icon_id,
        unknown_21: false,
        unknown_22: name_id,
        unknown_23: 0,
    };
    effect_packets::encode_add_effect_tag(tag_id, &tag)
}

fn header(family: u16, sub: u16) -> Vec<u8> {
    [family.to_le_bytes(), sub.to_le_bytes()].concat()
}

fn put_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Client writer `00a342e0`: header, i32 bar, i32 slot, i32 mode, then a
    /// u64 target (mode 0, 24 bytes) or four f32 (other modes, 32 bytes).
    fn cast(bar: i32, slot: i32, mode: i32, tail: usize) -> Vec<u8> {
        let mut packet = vec![36, 0, 10, 0];
        for value in [bar, slot, mode] {
            packet.extend(value.to_le_bytes());
        }
        packet.extend(std::iter::repeat_n(0, tail));
        packet
    }

    #[test]
    fn unsupported_cast_rejects_the_malformed_20_byte_shape() {
        // Mode 0 on the job bar is served by the combat path, not refused.
        assert_eq!(unsupported_cast(&cast(1, 0, 0, 8)).unwrap(), None);
        // Other bars and ground modes get the client failure string.
        assert_eq!(
            unsupported_cast(&cast(2, 0, 0, 8)).unwrap(),
            Some(unsupported_cast_failure())
        );
        assert_eq!(
            unsupported_cast(&cast(1, 0, 1, 16)).unwrap(),
            Some(unsupported_cast_failure())
        );
        // 20 bytes is neither the 24- nor the 32-byte client shape.
        assert_eq!(
            unsupported_cast(&cast(1, 0, 0, 4)).unwrap_err().to_string(),
            "malformed ability cast request"
        );
        // A truncated mode dword fails before the length check.
        assert_eq!(
            unsupported_cast(&[36, 0, 10, 0, 1, 0, 0, 0])
                .unwrap_err()
                .to_string(),
            "truncated ability cast request"
        );
        // Another family/subtype is dispatched as before.
        assert_eq!(unsupported_cast(&[36, 0, 12, 0]).unwrap(), None);
    }
}
