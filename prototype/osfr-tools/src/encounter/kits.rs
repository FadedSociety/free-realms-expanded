//! The 2014 weapon kits: the action bar comes from the weapon, and one resolver turns a slot
//! press into a strike (combat design section 6). Piece P2 (`combat-kits`) owns the bodies and
//! the loader for `weapon_kits.json`.
//!
//! Frozen by the combat seam: `bar_for` and `resolve` and the fields the seam listed. `Kit` and
//! `Strike` are this piece's to extend. Without `weapon_kits.json` in the content directory the
//! kits are dormant and the engine's older job skills (`skills.rs`) run as before.
use super::*;

mod cast;
pub mod status;
pub mod table;

pub use self::status::{StatusRow, StatusTable};
pub use self::table::{Ability, Kind, KitTable, Shape};

/// What the bar needs to know about the equipped weapon.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeaponFacts {
    pub item_id: i32,
    /// The item's name, for the "of X" suffix rule.
    pub name: String,
    /// The weapon's Power rating; the tier follows it, capped by the job level.
    pub power: u32,
}

/// The cast being resolved. P2's cast path sets `KitState::cast` around its strikes; every
/// `PlayerHit` built meanwhile carries it (`player_hit_packets`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cast {
    pub ability: i32,
    /// The hit effect the client plays on the target, if the ability has one.
    pub effect: Option<i32>,
}

/// The engine field `kit_state`: the loaded kits, the equipped weapons and the cast in
/// progress. It outlives `reset_session`. P2 may add fields; `weapons` and `cast` are frozen.
#[derive(Clone, Debug, Default)]
pub struct KitState {
    /// Each job's equipped weapon by profile id, as gameplay last reported it (`equip_weapon`).
    pub weapons: BTreeMap<i32, WeaponFacts>,
    pub cast: Option<Cast>,
}

/// One filled action-bar slot, as packet 36/5 lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct BarSlot {
    pub ability_id: i32,
    pub energy_cost: i32,
    pub icon_id: i32,
    pub name_id: i32,
    pub range: f32,
}

/// The eight slots of bar 1: slot 0 the basic, slot 1 the super.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bar {
    pub slots: [Option<BarSlot>; 8],
}

/// One weapon kit row of `weapon_kits.json`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Kit {
    pub id: String,
    /// Profile id of the job the kit belongs to.
    pub job: u32,
    /// Bar 1 by slot: 0 the basic, 1 the super, and the third ability where the table puts it.
    pub slots: [Option<Arc<Ability>>; 8],
}

impl Kit {
    pub fn slot(&self, slot: u8) -> Option<&Arc<Ability>> {
        self.slots.get(usize::from(slot)).and_then(Option::as_ref)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Caster {
    pub guid: u64,
    pub position: [f32; 3],
    /// Unit heading on the ground plane, for cones. All zero means "facing the target".
    pub heading: [f32; 3],
    pub profile: u32,
    pub level: u8,
    pub tier: u8,
    pub energy: i32,
    pub now_ms: u64,
}

/// A body an ability can land on.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub guid: u64,
    /// Enemy definition id; 0 for a player.
    pub kind: u32,
    pub position: [f32; 3],
    pub health: u32,
    pub max_health: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Targets<'a> {
    /// The client's selected target, if any.
    pub selected: Option<u64>,
    pub enemies: &'a [Body],
    /// The caster's group, the caster included.
    pub allies: &'a [Body],
}

/// What an accepted cast does; the engine applies it and sends the packets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Strike {
    pub ability_id: i32,
    pub energy_cost: i32,
    pub recast_ms: u32,
    /// (target, damage) per hit, in order.
    pub hits: Vec<(u64, u32)>,
    /// (target, amount).
    pub heals: Vec<(u64, u32)>,
    /// (target, status id in `status_effects.json`, ms).
    pub statuses: Vec<(u64, String, u32)>,
    /// The enemy the cast was aimed at, if it had one.
    pub primary: Option<u64>,
    /// (target, status the target must have, damage): a hit that lands only on such a target.
    pub conditional: Vec<(u64, String, u32)>,
    /// Damage over time started on a target.
    pub dots: Vec<StrikeDot>,
    /// Energy the caster gets back (drains).
    pub energy_gain: i32,
    /// Health for the rest of the caster's group; each member's own engine applies it.
    pub group_heal: u32,
    /// Helpers that fight beside the caster: (count, ms, damage per strike, ms between strikes).
    pub summon: Option<(u8, u32, u32, u32)>,
    pub animation: i32,
    pub effect: i32,
    /// Name of the 36/4 template in `combat_wire.json`, when one exists.
    pub wire: String,
    /// Our id of the ability and the older ids a quest step may still name.
    pub credit_ids: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StrikeDot {
    pub target: u64,
    pub damage: u32,
    pub every_ms: u32,
    pub ms: u32,
    /// Marker status put on the target while it burns; empty for none.
    pub status: String,
}

/// The bar a job at `level` gets from its weapon, by the shipped kit table.
pub fn bar_for(profile: u32, level: u8, weapon: &WeaponFacts) -> Bar {
    KitTable::builtin().map_or_else(Bar::default, |table| table.bar(profile, level, weapon))
}

/// A repeatable roll in 0..1 from the cast itself, so `resolve` stays a pure function.
fn roll(caster: &Caster, ability: i32, salt: u64) -> f32 {
    let mut x = caster
        .now_ms
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(caster.guid.rotate_left(17))
        .wrapping_add((ability as u64) << 32)
        .wrapping_add(salt.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 40) as f32 / (1u64 << 24) as f32
}

fn flat(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn by_distance(from: [f32; 3]) -> impl Fn(&&Body, &&Body) -> std::cmp::Ordering {
    move |a, b| {
        (flat(from, a.position), a.guid)
            .partial_cmp(&(flat(from, b.position), b.guid))
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Move `first` to the front of `hit` if it is there.
fn lead_with(hit: &mut Vec<&Body>, first: Option<u64>) {
    if let Some(i) = first.and_then(|guid| hit.iter().position(|b| b.guid == guid)) {
        let chosen = hit.remove(i);
        hit.insert(0, chosen);
    }
}

/// Resolve bar slot `slot` of `kit`, or the 36/1 refusal string id: 285 not enough energy,
/// 286 cannot be used, 288 no target in reach. (287, not ready, is the engine's: it owns the
/// clock.)
pub fn resolve(kit: &Kit, slot: u8, caster: &Caster, targets: &Targets) -> Result<Strike, i32> {
    let ability = kit.slot(slot).ok_or(286)?;
    if caster.energy < ability.cost {
        return Err(285);
    }
    let tier = usize::from(caster.tier.clamp(1, 5)) - 1;
    let id = ability.ability_id;
    let mut alive: Vec<&Body> = targets.enemies.iter().filter(|b| b.health > 0).collect();
    alive.sort_by(by_distance(caster.position));
    let within = |b: &Body, reach: f32| flat(caster.position, b.position) <= reach;
    // The aimed-at enemy: the selected one if it is in reach (a selected enemy out of reach
    // fails instead of silently hitting another), else the nearest in reach.
    let aimed = |reach: f32| -> Result<&Body, i32> {
        let selected = targets
            .selected
            .and_then(|guid| alive.iter().copied().find(|b| b.guid == guid));
        match selected {
            Some(b) if within(b, reach) => Ok(b),
            Some(_) => Err(288),
            None => alive.iter().copied().find(|b| within(b, reach)).ok_or(288),
        }
    };
    let mut primary = None;
    let hit: Vec<&Body> = match ability.shape {
        Shape::Own => Vec::new(),
        Shape::Single => {
            let b = aimed(ability.range)?;
            primary = Some(b.guid);
            vec![b]
        }
        Shape::Circle { radius, targets: cap } => {
            let mut hit: Vec<&Body> = alive.iter().copied().filter(|b| within(b, radius)).collect();
            lead_with(&mut hit, targets.selected);
            hit.truncate(cap);
            primary = hit.first().map(|b| b.guid);
            hit
        }
        Shape::Front { cone_deg, targets: cap } => {
            let facing = if flat(caster.heading, [0.0; 3]) > 0.001 {
                caster.heading
            } else {
                let b = aimed(ability.range)?;
                primary = Some(b.guid);
                [b.position[0] - caster.position[0], 0.0, b.position[2] - caster.position[2]]
            };
            let length = flat(facing, [0.0; 3]);
            let limit = (cone_deg.to_radians() / 2.0).cos();
            let mut hit: Vec<&Body> = alive
                .iter()
                .copied()
                .filter(|b| within(b, ability.range))
                .filter(|b| {
                    let to = [b.position[0] - caster.position[0], 0.0, b.position[2] - caster.position[2]];
                    let d = flat(to, [0.0; 3]);
                    // Standing on the caster counts as in front; so does a zero-length facing.
                    d < 0.01
                        || length < 0.001
                        || (to[0] * facing[0] + to[2] * facing[2]) / (d * length) >= limit - 1e-4
                })
                .collect();
            lead_with(&mut hit, primary);
            hit.truncate(cap);
            primary = primary.or(hit.first().map(|b| b.guid));
            hit
        }
        Shape::Chain { jumps, jump_range } => {
            let first = aimed(ability.range)?;
            primary = Some(first.guid);
            let mut hit = vec![first];
            let mut last = first.position;
            for _ in 0..jumps {
                let next = alive
                    .iter()
                    .copied()
                    .filter(|b| !hit.iter().any(|h| h.guid == b.guid) && flat(last, b.position) <= jump_range)
                    .min_by(by_distance(last));
                match next {
                    Some(b) => {
                        last = b.position;
                        hit.push(b);
                    }
                    None => break,
                }
            }
            hit
        }
        Shape::AtTarget { radius, targets: cap } => {
            let first = aimed(ability.range)?;
            primary = Some(first.guid);
            let mut hit: Vec<&Body> = alive
                .iter()
                .copied()
                .filter(|b| flat(first.position, b.position) <= radius)
                .collect();
            hit.sort_by(by_distance(first.position));
            lead_with(&mut hit, primary);
            hit.truncate(cap);
            hit
        }
    };
    let helps = ability.heal.is_some() || !ability.buffs.is_empty() || ability.summon.is_some();
    if hit.is_empty() && !helps {
        return Err(288);
    }
    let mut strike = Strike {
        ability_id: id,
        energy_cost: ability.cost,
        recast_ms: ability.recast_ms,
        primary,
        animation: ability.animation,
        effect: ability.effect,
        wire: ability.wire.clone(),
        credit_ids: std::iter::once(id)
            .chain(ability.legacy_ids.iter().copied())
            .collect(),
        ..Default::default()
    };
    let lone = alive.len() == 1;
    for (index, b) in hit.iter().enumerate() {
        let salt = (index as u64) << 8;
        if ability.damage[tier] > 0 {
            for _ in 0..ability.hits {
                strike.hits.push((b.guid, ability.damage[tier]));
            }
        }
        if let Some((chance, damage)) = ability.extra
            && roll(caster, id, salt | 1) < chance
        {
            strike.hits.push((b.guid, damage[tier]));
        }
        match &ability.bonus {
            Some((when, damage)) if when == "lone" => {
                if lone {
                    strike.hits.push((b.guid, damage[tier]));
                }
            }
            Some((status, damage)) => {
                strike.conditional.push((b.guid, status.clone(), damage[tier]));
            }
            None => {}
        }
        for (n, rider) in ability.riders.iter().enumerate() {
            if roll(caster, id, salt | (2 + n as u64)) < rider.chance {
                strike.statuses.push((b.guid, rider.status.clone(), rider.ms));
            }
        }
        if let Some(dot) = &ability.dot {
            strike.dots.push(StrikeDot {
                target: b.guid,
                damage: dot.damage[tier],
                every_ms: dot.every_ms,
                ms: dot.ms,
                status: dot.status.clone(),
            });
        }
        if let Some((amount, chance)) = ability.energy_gain
            && roll(caster, id, salt | 7) < chance
        {
            strike.energy_gain += amount;
        }
    }
    let group = |whole: bool| -> Vec<u64> {
        let mut members = vec![caster.guid];
        if whole {
            members.extend(
                targets
                    .allies
                    .iter()
                    .filter(|b| {
                        b.guid != caster.guid
                            && b.health > 0
                            && flat(caster.position, b.position) <= ability.group_radius
                    })
                    .map(|b| b.guid),
            );
        }
        members
    };
    if let Some((whole, amount)) = ability.heal {
        strike.heals = group(whole).into_iter().map(|guid| (guid, amount[tier])).collect();
        if whole {
            strike.group_heal = amount[tier];
        }
    }
    for (status, whole, ms) in &ability.buffs {
        for guid in group(*whole) {
            strike.statuses.push((guid, status.clone(), *ms));
        }
    }
    if let Some(summon) = &ability.summon {
        strike.summon = Some((summon.count, summon.ms, summon.damage[tier], summon.every_ms));
    }
    Ok(strike)
}

/// The engine's kit data and per-session kit state; it lives in `SkillEffects`. (The equipped
/// weapons and the cast in progress are in the seam's `KitState`.)
#[derive(Clone, Default)]
pub(super) struct KitRun {
    /// `None` while `weapon_kits.json` is absent: the engine's older job skills run instead.
    pub(super) table: Option<Arc<KitTable>>,
    pub(super) statuses: Arc<StatusTable>,
    pub(super) book: status::Book,
    /// Critical hits rolled, so a cast can tell whether one of its own hits was critical.
    pub(super) crits: u32,
    /// Crit chance added by the target of the hit in flight.
    pub(super) crit_against: u32,
    pub(super) passive: cast::Passives,
    pub(super) taunt_threat: u32,
    pub(super) helpers: Option<cast::Helpers>,
    pub(super) last_health: Option<u32>,
    /// Where the player last cast from: the centre for helpers and trait effects.
    pub(super) position: Option<[f32; 3]>,
}

impl KitRun {
    /// A new job or a new run: session state goes, content and the weapon stay.
    pub(super) fn reset(&mut self) {
        self.book.clear();
        self.crit_against = 0;
        self.helpers = None;
        self.last_health = None;
        self.position = None;
    }
}

impl EncounterEngine {
    /// Engine load: `weapon_kits.json` and `status_effects.json` from `directory`. Without
    /// the kit file the kits stay dormant; the status table then still serves battle items
    /// and enemy attacks if its own file is there.
    pub(super) fn load_kits(&mut self, directory: &Path) -> io::Result<()> {
        let statuses = StatusTable::load(directory)?.unwrap_or_default();
        self.skills.kit.table = KitTable::load(directory, &statuses)?;
        self.skills.kit.statuses = Arc::new(statuses);
        Ok(())
    }

    /// Gameplay reports the weapon `profile` has equipped: at every job sync (world load, job
    /// switch, level-up) and after every committed inventory request (`Gameplay::sync_weapon`).
    /// The cast follows it; a weapon the table does not know means the job's default kit.
    pub fn equip_weapon(&mut self, profile: i32, weapon: WeaponFacts) {
        self.kit_state.weapons.insert(profile, weapon);
    }

    /// Whether the weapon kits are live in this engine.
    pub fn kits_live(&self) -> bool {
        self.skills.kit.table.is_some()
    }

    /// Kill-fact fields this piece knows (the ability or item that made the kill).
    pub(super) fn stamp_kill_kit(&self, fact: &mut KillFact) {
        fact.ability = self.kit_state.cast.as_ref().map(|cast| cast.ability);
    }

    /// A basic attack request (36/10, bar 1 slot 0) on `guid`. The original toolbar names
    /// the player itself for a self-centred attack; the server then picks the enemy.
    pub(super) fn basic_cast(
        &mut self,
        guid: u64,
        player_guid: u64,
        position: [f32; 3],
        profile_id: i32,
        slot: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        if self.kit_applies(profile_id) {
            return self.kit_cast(0, guid, position, profile_id, now_ms);
        }
        if guid == player_guid {
            // The original basic toolbar requests a self-centered attack.
            // This supported slice resolves one nearby enemy on the server;
            // original multi-target sweep effects remain unrecovered.
            let states: Vec<&EnemyState> = match &self.state.active {
                Some(a) => a.enemies.iter().collect(),
                None => self
                    .state
                    .world_enemies
                    .iter()
                    .chain(
                        self.field_sim
                            .enemies
                            .iter()
                            .filter(|f| f.present)
                            .map(|f| &f.state),
                    )
                    .collect(),
            };
            let range = self.scale.profile_stats(profile_id, slot).0;
            let nearest = states
                .into_iter()
                .filter(|s| {
                    self.enemy(s.guid).is_some()
                        && s.health > 0
                        && distance(position, s.position) <= range
                })
                // An unshielded enemy in reach first, then the nearest.
                .min_by(|a, b| {
                    (self.shielded(a.guid), distance(position, a.position))
                        .partial_cmp(&(self.shielded(b.guid), distance(position, b.position)))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|s| s.guid);
            return match nearest {
                Some(target) => self.attack(target, position, profile_id, slot, now_ms),
                None => Ok(EncounterOutcome::default()),
            };
        }
        self.attack(guid, position, profile_id, slot, now_ms)
    }

    /// One accepted basic attack on `guid`: reach and damage of the job's slot, then the hit.
    pub(super) fn basic_strike(
        &mut self,
        guid: u64,
        position: [f32; 3],
        profile_id: i32,
        slot: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        if slot == 0 && self.kit_applies(profile_id) {
            return self.kit_cast(0, guid, position, profile_id, now_ms);
        }
        let (range, damage) = self.scale.profile_stats(profile_id, slot);
        let damage = (damage as f32 * self.damage_scale).round() as u32;
        self.strike(guid, position, range, damage, now_ms)
    }
}
