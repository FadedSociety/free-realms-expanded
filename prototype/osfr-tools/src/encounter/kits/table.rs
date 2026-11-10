//! `weapon_kits.json`: the kit table, read once and resolved so a cast needs no lookups.
//! Every number here is data; the file marks the reconstructed ones (R).
use super::{Bar, BarSlot, Kit, WeaponFacts, status::StatusTable};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::Path,
    sync::{Arc, OnceLock},
};

const MAX_BYTES: u64 = 1024 * 1024;

fn bad(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// Read a bounded content file; `None` when it is absent.
pub(super) fn read_optional(path: &Path) -> io::Result<Option<Vec<u8>>> {
    use io::Read;
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(bad(format!("{} exceeds bounds", path.display())));
    }
    Ok(Some(bytes))
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Energy {
    pub max: i32,
    pub per_tick: i32,
    pub tick_ms: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Slots {
    pub basic: u8,
    #[serde(rename = "super")]
    pub super_: u8,
    pub third: u8,
    pub third_min_level: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Class {
    tier_damage: [u32; 5],
    cost: i32,
    recast_ms: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShapeRow {
    #[serde(default)]
    targets: usize,
    #[serde(default)]
    cone_deg: f32,
    #[serde(default)]
    radius: f32,
    #[serde(default)]
    jumps: usize,
    #[serde(default)]
    jump_range: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RiderDefaults {
    pub sometimes_chance: f32,
    pub extra_chance: f32,
    pub group_radius: f32,
    pub taunt_threat_basics: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRider {
    status: String,
    #[serde(default)]
    sometimes: bool,
    #[serde(default)]
    chance: Option<f32>,
    #[serde(default)]
    ms: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawExtra {
    #[serde(default)]
    sometimes: bool,
    #[serde(default)]
    chance: Option<f32>,
    power5: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBonus {
    when: String,
    power5: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHeal {
    who: String,
    power5: u32,
    class: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBuff {
    status: String,
    who: String,
    #[serde(default)]
    ms: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(in crate::encounter) struct RawDot {
    pub power5: u32,
    pub every_ms: u32,
    pub ms: u32,
    #[serde(default)]
    pub status: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnergyGain {
    amount: i32,
    chance: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSummon {
    count: u8,
    ms: u32,
    class: String,
    every_ms: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAbility {
    #[allow(dead_code)]
    name: String,
    name_id: i32,
    #[serde(default)]
    description_id: i32,
    ability_id: i32,
    #[serde(default)]
    tier_ids: BTreeMap<String, i32>,
    icon_id: i32,
    kind: String,
    job: u32,
    class: String,
    shape: String,
    range: f32,
    #[serde(default)]
    power5: u32,
    #[serde(default)]
    hits: Option<u8>,
    #[serde(default)]
    riders: Vec<RawRider>,
    #[serde(default)]
    extra: Option<RawExtra>,
    #[serde(default)]
    bonus: Option<RawBonus>,
    #[serde(default)]
    heal: Option<RawHeal>,
    #[serde(default)]
    buffs: Vec<RawBuff>,
    #[serde(default)]
    dot: Option<RawDot>,
    #[serde(default)]
    energy_gain: Option<RawEnergyGain>,
    #[serde(default)]
    summon: Option<RawSummon>,
    #[serde(default)]
    cost: Option<i32>,
    #[serde(default)]
    recast_ms: Option<u32>,
    #[serde(default)]
    animation: i32,
    #[serde(default)]
    effect: i32,
    #[serde(default)]
    wire: String,
    #[serde(default)]
    legacy_ids: Vec<i32>,
    #[serde(default)]
    #[allow(dead_code)]
    source: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawKit {
    job: u32,
    basic: String,
    #[serde(rename = "super")]
    super_: String,
    #[serde(default)]
    third: Option<String>,
    from_power: u8,
    min_level: u8,
    /// Item definition ids that carry this kit, by the item's Power rating.
    #[serde(default)]
    items: BTreeMap<String, Vec<i32>>,
    #[serde(default)]
    #[allow(dead_code)]
    source: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWeapons {
    /// Per profile: item-name suffix ("of Dragonstrike") to kit.
    #[serde(default)]
    by_suffix: BTreeMap<String, BTreeMap<String, String>>,
    default: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Passive {
    #[serde(default)]
    pub dealt: Option<f32>,
    #[serde(default)]
    pub taken: Option<f32>,
    #[serde(default)]
    pub crit: u32,
    #[serde(default)]
    pub crit_damage: f32,
    #[serde(default)]
    pub energy_per_tick: i32,
    /// Not applied yet: movement and dodge belong to the stat block.
    #[serde(default)]
    pub speed: Option<f32>,
    #[serde(default)]
    pub dodge: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TraitStatus {
    pub id: String,
    /// `target`, `attacker` or `near`.
    pub to: String,
    pub ms: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TraitStrike {
    pub class: String,
    pub targets: usize,
    pub radius: f32,
    #[serde(default)]
    pub factor: Option<f32>,
}

/// One passive trait of a job, granted at `level`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Trait {
    pub level: u8,
    pub id: String,
    pub name_id: i32,
    #[serde(default)]
    pub passive: Option<Passive>,
    /// `hit`, `crit`, `kill` or `struck`; empty for a pure passive.
    #[serde(default)]
    pub on: String,
    #[serde(default)]
    pub chance: f32,
    #[serde(default)]
    pub energy: i32,
    #[serde(default)]
    pub heal_pct: u32,
    #[serde(default)]
    pub group: bool,
    #[serde(default)]
    pub status: Option<TraitStatus>,
    #[serde(default)]
    pub strike: Option<TraitStrike>,
    #[serde(default)]
    #[allow(dead_code)]
    source: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema_version: u32,
    #[allow(dead_code)]
    source: serde_json::Value,
    energy: Energy,
    slots: Slots,
    classes: BTreeMap<String, Class>,
    shapes: BTreeMap<String, ShapeRow>,
    rider_defaults: RiderDefaults,
    abilities: BTreeMap<String, RawAbility>,
    kits: BTreeMap<String, RawKit>,
    weapons: RawWeapons,
    traits: BTreeMap<String, Vec<Trait>>,
}

/// Where an ability lands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// No enemy target: a heal, buff or summon.
    Own,
    Single,
    /// Enemies inside a cone in front of the caster, out to the ability's range.
    Front { cone_deg: f32, targets: usize },
    /// Enemies within `radius` of the caster ("near", "around").
    Circle { radius: f32, targets: usize },
    /// The target, then up to `jumps` more, each within `jump_range` of the last.
    Chain { jumps: usize, jump_range: f32 },
    /// Enemies within `radius` of the target.
    AtTarget { radius: f32, targets: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Basic,
    Super,
    Third,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rider {
    pub status: String,
    pub chance: f32,
    pub ms: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dot {
    pub damage: [u32; 5],
    pub every_ms: u32,
    pub ms: u32,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Summon {
    pub count: u8,
    pub ms: u32,
    pub damage: [u32; 5],
    pub every_ms: u32,
}

/// One ability, with every default already applied.
#[derive(Clone, Debug, PartialEq)]
pub struct Ability {
    pub key: String,
    pub name_id: i32,
    pub description_id: i32,
    /// Our id for the ability: what a quest step and a kill fact name.
    pub ability_id: i32,
    /// The live server's id per weapon tier, where a capture has it; the bar shows it.
    pub tier_ids: BTreeMap<u8, i32>,
    pub icon_id: i32,
    pub kind: Kind,
    pub job: u32,
    pub class: String,
    /// Damage of one hit per weapon tier.
    pub damage: [u32; 5],
    pub hits: u8,
    pub shape: Shape,
    pub range: f32,
    pub cost: i32,
    pub recast_ms: u32,
    pub riders: Vec<Rider>,
    pub extra: Option<(f32, [u32; 5])>,
    /// (`lone` or a status id, damage per tier).
    pub bonus: Option<(String, [u32; 5])>,
    /// (whole group, amount per tier).
    pub heal: Option<(bool, [u32; 5])>,
    /// (status, whole group, ms).
    pub buffs: Vec<(String, bool, u32)>,
    pub dot: Option<Dot>,
    /// (energy, chance) per target hit.
    pub energy_gain: Option<(i32, f32)>,
    pub summon: Option<Summon>,
    pub animation: i32,
    pub effect: i32,
    pub wire: String,
    pub legacy_ids: Vec<i32>,
    pub group_radius: f32,
}

impl Ability {
    /// The id the client's bar and definitions carry at `tier`.
    pub fn wire_id(&self, tier: u8) -> i32 {
        self.tier_ids.get(&tier).copied().unwrap_or(self.ability_id)
    }
}

struct KitRow {
    kit: Kit,
    from_power: u8,
    min_level: u8,
}

/// The loaded kit table.
pub struct KitTable {
    pub energy: Energy,
    pub slots: Slots,
    pub rider_defaults: RiderDefaults,
    /// A plain basic per tier: the unit taunt threat is counted in.
    pub plain_basic: [u32; 5],
    classes: BTreeMap<String, [u32; 5]>,
    abilities: BTreeMap<String, Arc<Ability>>,
    kits: BTreeMap<String, KitRow>,
    /// Item definition id to (kit, Power rating).
    by_item: BTreeMap<i32, (String, u8)>,
    by_suffix: BTreeMap<u32, Vec<(String, String)>>,
    default: BTreeMap<u32, String>,
    traits: BTreeMap<u32, Vec<Trait>>,
}

/// A value at each tier: the class ladder itself when `power5` is the class's own top step,
/// otherwise the ladder scaled to the tooltip's Power 5 number.
fn ladder(class: &[u32; 5], power5: u32) -> [u32; 5] {
    if power5 == class[4] {
        return *class;
    }
    let mut out = [0; 5];
    for (value, step) in out.iter_mut().zip(class) {
        *value = (f64::from(power5) * f64::from(*step) / f64::from(class[4].max(1))).round() as u32;
    }
    out
}

/// The first job level of each weapon tier (Power 1 to 5).
const TIER_LEVELS: [u8; 5] = [1, 4, 8, 12, 16];

/// The weapon tier a job level may use.
pub fn tier_of_level(level: u8) -> u8 {
    TIER_LEVELS.iter().filter(|&&first| first <= level).count().max(1) as u8
}

impl KitTable {
    /// The table shipped with the server, for callers that have no content directory.
    pub fn builtin() -> Option<&'static Arc<KitTable>> {
        static TABLE: OnceLock<Option<Arc<KitTable>>> = OnceLock::new();
        TABLE
            .get_or_init(|| {
                Self::parse(
                    include_bytes!("../../../data/content/weapon_kits.json"),
                    StatusTable::builtin(),
                )
                .ok()
                .map(Arc::new)
            })
            .as_ref()
    }

    /// `weapon_kits.json` in `directory`; `None` (kits dormant) when the file is absent.
    pub fn load(directory: &Path, statuses: &StatusTable) -> io::Result<Option<Arc<KitTable>>> {
        match read_optional(&directory.join("weapon_kits.json"))? {
            Some(bytes) => Ok(Some(Arc::new(Self::parse(&bytes, statuses)?))),
            None => Ok(None),
        }
    }

    pub fn parse(bytes: &[u8], statuses: &StatusTable) -> io::Result<KitTable> {
        let raw: Raw = serde_json::from_slice(bytes)
            .map_err(|e| bad(format!("weapon_kits.json: {e}")))?;
        if raw.schema_version != 1 {
            return Err(bad("weapon_kits.json: unsupported schema_version"));
        }
        if !(1..=1000).contains(&raw.energy.max)
            || !(0..=100).contains(&raw.energy.per_tick)
            || !(100..=60_000).contains(&raw.energy.tick_ms)
            || raw.slots.basic != 0
            || raw.slots.super_ != 1
            || !(2..8).contains(&raw.slots.third)
            || !(0.0..=1.0).contains(&raw.rider_defaults.sometimes_chance)
            || !(0.0..=1.0).contains(&raw.rider_defaults.extra_chance)
            || !(0.0..=100.0).contains(&raw.rider_defaults.group_radius)
        {
            return Err(bad("weapon_kits.json: energy, slots or rider_defaults out of range"));
        }
        let mut classes = BTreeMap::new();
        let mut costs = BTreeMap::new();
        for (name, class) in &raw.classes {
            if class.tier_damage.iter().any(|d| *d == 0 || *d > 1_000_000)
                || !(0..=1000).contains(&class.cost)
                || !(100..=600_000).contains(&class.recast_ms)
            {
                return Err(bad(format!("weapon_kits.json: class {name} out of range")));
            }
            classes.insert(name.clone(), class.tier_damage);
            costs.insert(name.clone(), (class.cost, class.recast_ms));
        }
        let plain_basic = *classes
            .get("basic_plain")
            .ok_or_else(|| bad("weapon_kits.json: class basic_plain is required"))?;
        let class_of = |name: &str, key: &str| {
            classes
                .get(name)
                .copied()
                .ok_or_else(|| bad(format!("weapon_kits.json: {key} names unknown class {name}")))
        };
        let status_ms = |id: &str, ms: Option<u32>, key: &str| -> io::Result<u32> {
            let row = statuses
                .row(id)
                .ok_or_else(|| bad(format!("weapon_kits.json: {key} names unknown status {id}")))?;
            let ms = ms.unwrap_or(row.default_ms);
            if !(100..=600_000).contains(&ms) {
                return Err(bad(format!("weapon_kits.json: {key} status time out of range")));
            }
            Ok(ms)
        };
        let who = |who: &str, key: &str| match who {
            "self" => Ok(false),
            "group" => Ok(true),
            _ => Err(bad(format!("weapon_kits.json: {key} who must be self or group"))),
        };
        let chance = |sometimes: bool, given: Option<f32>, default: f32, key: &str| {
            let chance = given.unwrap_or(if sometimes { default } else { 1.0 });
            if (0.0..=1.0).contains(&chance) {
                Ok(chance)
            } else {
                Err(bad(format!("weapon_kits.json: {key} chance out of range")))
            }
        };
        let mut abilities = BTreeMap::new();
        let mut ids = BTreeSet::new();
        for (key, a) in raw.abilities {
            let class = class_of(&a.class, &key)?;
            let (class_cost, class_recast) = costs[&a.class];
            let row = raw.shapes.get(&a.shape);
            let shape = match (a.shape.as_str(), row) {
                ("self", _) => Shape::Own,
                ("single", _) => Shape::Single,
                ("front", Some(r)) => Shape::Front { cone_deg: r.cone_deg, targets: r.targets },
                ("chain", Some(r)) => Shape::Chain { jumps: r.jumps, jump_range: r.jump_range },
                ("at_target", Some(r)) => Shape::AtTarget { radius: r.radius, targets: r.targets },
                (_, Some(r)) if r.radius > 0.0 => Shape::Circle { radius: r.radius, targets: r.targets },
                _ => return Err(bad(format!("weapon_kits.json: {key} names unknown shape {}", a.shape))),
            };
            let shape_ok = match shape {
                Shape::Own | Shape::Single => true,
                Shape::Front { cone_deg, targets } => (1.0..=360.0).contains(&cone_deg) && (1..=32).contains(&targets),
                Shape::Circle { radius, targets } | Shape::AtTarget { radius, targets } => {
                    (0.5..=50.0).contains(&radius) && (1..=32).contains(&targets)
                }
                Shape::Chain { jumps, jump_range } => jumps <= 16 && (0.5..=50.0).contains(&jump_range),
            };
            let kind = match a.kind.as_str() {
                "basic" => Kind::Basic,
                "super" => Kind::Super,
                "third" => Kind::Third,
                _ => return Err(bad(format!("weapon_kits.json: {key} kind"))),
            };
            let hits = a.hits.unwrap_or(1);
            let cost = a.cost.unwrap_or(class_cost);
            let recast_ms = a.recast_ms.unwrap_or(class_recast);
            let mut tier_ids = BTreeMap::new();
            for (tier, id) in &a.tier_ids {
                let tier: u8 = tier.parse().map_err(|_| bad(format!("weapon_kits.json: {key} tier_ids")))?;
                if !(1..=5).contains(&tier) || *id <= 0 || !ids.insert(*id) {
                    return Err(bad(format!("weapon_kits.json: {key} tier_ids invalid or duplicate")));
                }
                tier_ids.insert(tier, *id);
            }
            if !shape_ok
                || a.name_id <= 0
                || a.description_id < 0
                || a.ability_id <= 0
                || !ids.insert(a.ability_id)
                || a.icon_id <= 0
                || !(0.0..=30.0).contains(&a.range)
                || a.power5 > 1_000_000
                || !(1..=8).contains(&hits)
                || !(0..=1000).contains(&cost)
                || !(100..=600_000).contains(&recast_ms)
                || a.animation < 0
                || a.effect < 0
                || a.riders.len() > 4
                || a.buffs.len() > 4
                || a.legacy_ids.iter().any(|id| *id <= 0)
            {
                return Err(bad(format!("weapon_kits.json: ability {key} invalid or duplicate id")));
            }
            let mut riders = Vec::new();
            for r in &a.riders {
                riders.push(Rider {
                    chance: chance(r.sometimes, r.chance, raw.rider_defaults.sometimes_chance, &key)?,
                    ms: status_ms(&r.status, r.ms, &key)?,
                    status: r.status.clone(),
                });
            }
            let mut buffs = Vec::new();
            for b in &a.buffs {
                buffs.push((b.status.clone(), who(&b.who, &key)?, status_ms(&b.status, b.ms, &key)?));
            }
            let extra = match &a.extra {
                Some(e) => Some((
                    chance(e.sometimes, e.chance, raw.rider_defaults.extra_chance, &key)?,
                    ladder(&class, e.power5),
                )),
                None => None,
            };
            let bonus = match &a.bonus {
                Some(b) => {
                    if b.when != "lone" {
                        status_ms(&b.when, None, &key)?;
                    }
                    Some((b.when.clone(), ladder(&class, b.power5)))
                }
                None => None,
            };
            let heal = match &a.heal {
                Some(h) => Some((who(&h.who, &key)?, ladder(&class_of(&h.class, &key)?, h.power5))),
                None => None,
            };
            let dot = match &a.dot {
                Some(d) => {
                    if !(250..=60_000).contains(&d.every_ms) || !(250..=60_000).contains(&d.ms) {
                        return Err(bad(format!("weapon_kits.json: {key} dot timing out of range")));
                    }
                    if !d.status.is_empty() {
                        status_ms(&d.status, None, &key)?;
                    }
                    Some(Dot {
                        damage: ladder(&class, d.power5),
                        every_ms: d.every_ms,
                        ms: d.ms,
                        status: d.status.clone(),
                    })
                }
                None => None,
            };
            let summon = match &a.summon {
                Some(s) => {
                    if !(1..=8).contains(&s.count)
                        || !(1000..=120_000).contains(&s.ms)
                        || !(250..=60_000).contains(&s.every_ms)
                    {
                        return Err(bad(format!("weapon_kits.json: {key} summon out of range")));
                    }
                    Some(Summon {
                        count: s.count,
                        ms: s.ms,
                        damage: class_of(&s.class, &key)?,
                        every_ms: s.every_ms,
                    })
                }
                None => None,
            };
            let energy_gain = match &a.energy_gain {
                Some(g) if (1..=100).contains(&g.amount) && (0.0..=1.0).contains(&g.chance) => {
                    Some((g.amount, g.chance))
                }
                Some(_) => return Err(bad(format!("weapon_kits.json: {key} energy_gain out of range"))),
                None => None,
            };
            if a.power5 == 0
                && heal.is_none()
                && buffs.is_empty()
                && riders.is_empty()
                && dot.is_none()
                && summon.is_none()
            {
                return Err(bad(format!("weapon_kits.json: ability {key} does nothing")));
            }
            abilities.insert(
                key.clone(),
                Arc::new(Ability {
                    name_id: a.name_id,
                    description_id: a.description_id,
                    ability_id: a.ability_id,
                    tier_ids,
                    icon_id: a.icon_id,
                    kind,
                    job: a.job,
                    damage: if a.power5 == 0 { [0; 5] } else { ladder(&class, a.power5) },
                    class: a.class,
                    hits,
                    shape,
                    range: a.range,
                    cost,
                    recast_ms,
                    riders,
                    extra,
                    bonus,
                    heal,
                    buffs,
                    dot,
                    energy_gain,
                    summon,
                    animation: a.animation,
                    effect: a.effect,
                    wire: a.wire,
                    legacy_ids: a.legacy_ids,
                    group_radius: raw.rider_defaults.group_radius,
                    key,
                }),
            );
        }
        let mut kits = BTreeMap::new();
        let mut by_item = BTreeMap::new();
        for (id, k) in raw.kits {
            let find = |key: &str, kind: Kind| -> io::Result<Arc<Ability>> {
                abilities
                    .get(key)
                    .filter(|a| a.kind == kind && a.job == k.job)
                    .cloned()
                    .ok_or_else(|| bad(format!("weapon_kits.json: kit {id} names {key}, which is not its job's {kind:?}")))
            };
            let mut slots: [Option<Arc<Ability>>; 8] = Default::default();
            slots[usize::from(raw.slots.basic)] = Some(find(&k.basic, Kind::Basic)?);
            slots[usize::from(raw.slots.super_)] = Some(find(&k.super_, Kind::Super)?);
            if let Some(third) = &k.third {
                slots[usize::from(raw.slots.third)] = Some(find(third, Kind::Third)?);
            }
            if !(1..=5).contains(&k.from_power) || !(1..=20).contains(&k.min_level) {
                return Err(bad(format!("weapon_kits.json: kit {id} power or level out of range")));
            }
            for (power, items) in &k.items {
                let power: u8 = power.parse().map_err(|_| bad(format!("weapon_kits.json: kit {id} items")))?;
                for item in items {
                    if !(1..=5).contains(&power) || *item <= 0 || by_item.insert(*item, (id.clone(), power)).is_some() {
                        return Err(bad(format!("weapon_kits.json: item {item} is invalid or in two kits")));
                    }
                }
            }
            kits.insert(
                id.clone(),
                KitRow {
                    kit: Kit { id, job: k.job, slots },
                    from_power: k.from_power,
                    min_level: k.min_level,
                },
            );
        }
        let job_kit = |profile: &str, kit: &str| -> io::Result<u32> {
            let profile: u32 = profile.parse().map_err(|_| bad("weapon_kits.json: profile key"))?;
            match kits.get(kit) {
                Some(row) if row.kit.job == profile => Ok(profile),
                _ => Err(bad(format!("weapon_kits.json: weapons names {kit}, which is not a kit of job {profile}"))),
            }
        };
        let mut default = BTreeMap::new();
        for (profile, kit) in &raw.weapons.default {
            default.insert(job_kit(profile, kit)?, kit.clone());
        }
        let mut by_suffix = BTreeMap::new();
        for (profile, rows) in &raw.weapons.by_suffix {
            for (suffix, kit) in rows {
                let profile = job_kit(profile, kit)?;
                by_suffix.entry(profile).or_insert_with(Vec::new).push((suffix.clone(), kit.clone()));
            }
        }
        let mut traits = BTreeMap::new();
        for (profile, rows) in raw.traits {
            let profile: u32 = profile.parse().map_err(|_| bad("weapon_kits.json: trait profile key"))?;
            for t in &rows {
                let passive_ok = t.passive.as_ref().is_none_or(|p| {
                    p.dealt.is_none_or(|v| (0.1..=10.0).contains(&v))
                        && p.taken.is_none_or(|v| (0.1..=10.0).contains(&v))
                        && p.crit <= 100
                        && (-1.0..=5.0).contains(&p.crit_damage)
                        && (0..=100).contains(&p.energy_per_tick)
                });
                let status_ok = match &t.status {
                    Some(s) => {
                        status_ms(&s.id, Some(s.ms), &t.id)?;
                        matches!(s.to.as_str(), "target" | "attacker" | "near")
                    }
                    None => true,
                };
                let strike_ok = match &t.strike {
                    Some(s) => {
                        class_of(&s.class, &t.id)?;
                        (1..=16).contains(&s.targets) && (0.5..=50.0).contains(&s.radius)
                    }
                    None => true,
                };
                if !(1..=20).contains(&t.level)
                    || t.name_id <= 0
                    || !matches!(t.on.as_str(), "" | "hit" | "crit" | "kill" | "struck")
                    || !(0.0..=1.0).contains(&t.chance)
                    || !(0..=100).contains(&t.energy)
                    || t.heal_pct > 100
                    || !passive_ok
                    || !status_ok
                    || !strike_ok
                {
                    return Err(bad(format!("weapon_kits.json: trait {} out of range", t.id)));
                }
            }
            traits.insert(profile, rows);
        }
        Ok(KitTable {
            energy: raw.energy,
            slots: raw.slots,
            rider_defaults: raw.rider_defaults,
            plain_basic,
            classes,
            abilities,
            kits,
            by_item,
            by_suffix,
            default,
            traits,
        })
    }

    /// Whether the table gives `profile` a kit at all (a combat job).
    pub fn supports(&self, profile: u32) -> bool {
        self.default.contains_key(&profile)
    }

    pub fn ability(&self, key: &str) -> Option<&Arc<Ability>> {
        self.abilities.get(key)
    }

    pub fn abilities(&self) -> impl Iterator<Item = &Arc<Ability>> {
        self.abilities.values()
    }

    pub fn kit(&self, id: &str) -> Option<&Kit> {
        self.kits.get(id).map(|row| &row.kit)
    }

    pub fn kits(&self) -> impl Iterator<Item = &Kit> {
        self.kits.values().map(|row| &row.kit)
    }

    /// A class's damage ladder.
    pub fn class(&self, name: &str) -> Option<[u32; 5]> {
        self.classes.get(name).copied()
    }

    /// The traits `profile` has at `level`.
    pub fn traits(&self, profile: u32, level: u8) -> impl Iterator<Item = &Trait> {
        self.traits
            .get(&profile)
            .into_iter()
            .flatten()
            .filter(move |t| t.level <= level)
    }

    /// The kit `weapon` gives `profile`: by item id, then by the "of X" name suffix, then the
    /// job's default. A kit of another job never applies.
    pub fn kit_for(&self, profile: u32, weapon: &WeaponFacts) -> Option<&Kit> {
        let own = |id: &String| self.kits.get(id).filter(|row| row.kit.job == profile);
        self.by_item
            .get(&weapon.item_id)
            .and_then(|(id, _)| own(id))
            .or_else(|| {
                self.by_suffix
                    .get(&profile)
                    .into_iter()
                    .flatten()
                    .find(|(suffix, _)| !weapon.name.is_empty() && weapon.name.ends_with(suffix.as_str()))
                    .and_then(|(_, id)| own(id))
            })
            .or_else(|| self.default.get(&profile).and_then(own))
            .map(|row| &row.kit)
    }

    /// The weapon's Power: its own rating, else the item table's, else the kit's first tier.
    pub fn power(&self, weapon: &WeaponFacts, kit: &Kit) -> u8 {
        if (1..=5).contains(&weapon.power) {
            return weapon.power as u8;
        }
        self.by_item
            .get(&weapon.item_id)
            .map(|(_, power)| *power)
            .or_else(|| self.kits.get(&kit.id).map(|row| row.from_power))
            .unwrap_or(1)
    }

    /// The damage tier of a cast: the weapon's Power, capped by the tier the job level reaches.
    /// A weapon the table does not know (the default kit) follows the level alone.
    pub fn tier(&self, profile: u32, level: u8, weapon: &WeaponFacts) -> u8 {
        let cap = tier_of_level(level);
        match self.kit_for(profile, weapon) {
            Some(kit) if (1..=5).contains(&weapon.power) || self.by_item.contains_key(&weapon.item_id) => {
                self.power(weapon, kit).min(cap)
            }
            _ => cap,
        }
    }

    /// The level the kit's weapons first appear at (wiki), for tools.
    pub fn kit_min_level(&self, id: &str) -> Option<u8> {
        self.kits.get(id).map(|row| row.min_level)
    }

    /// Bar 1 as the weapon sets it: the basic, the super, and the third ability of a
    /// three-ability weapon once the job level allows it.
    pub fn bar(&self, profile: u32, level: u8, weapon: &WeaponFacts) -> Bar {
        let mut bar = Bar::default();
        let Some(kit) = self.kit_for(profile, weapon) else {
            return bar;
        };
        let tier = self.tier(profile, level, weapon);
        for (index, ability) in kit.slots.iter().enumerate() {
            let Some(ability) = ability else { continue };
            if ability.kind == Kind::Third && level < self.slots.third_min_level {
                continue;
            }
            bar.slots[index] = Some(BarSlot {
                ability_id: ability.wire_id(tier),
                energy_cost: ability.cost,
                icon_id: ability.icon_id,
                name_id: ability.name_id,
                range: ability.range,
            });
        }
        bar
    }
}
