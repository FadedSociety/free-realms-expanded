//! The combat scale: how hard players hit, how tough they are, which levels
//! open each weapon tier, and how enemy rows follow. Everything comes from
//! `combat_scale.json` (`notes/Research/Damage Scale.md`, combat design section 5); a
//! content directory without the file gets the shipped one, compiled in (the same rule as
//! `combat_rules.rs`); an invalid file refuses the load. No number lives in code.
//!
//! Two enemy scales live side by side. A kind the catalogue (`enemies.json`) gives a `rank`
//! takes its health and hit from the rank tables: plain basics of the tier times the rank's
//! count, the way the three captured kinds measure out. Every other row keeps the older
//! rule: its authored numbers are level-one baselines, grown on the player curve and by the
//! two global enemy scales. Ranking a kind is one key in its definition.
use super::scale::{Catalogue, Definition};
use super::*;
use std::sync::OnceLock;

const FILE: &str = "combat_scale.json";
/// The shipped file: for a content directory that has none, and for `scale`'s free functions.
const BUNDLED: &str = include_str!("../../data/content/combat_scale.json");
/// Every value carries a source row naming it; `ranks` and `enemy_hit` carry two, what was
/// captured and what was filled in.
const FIELDS: [&str; 20] = [
    "basic_damage",
    "tier_min_levels",
    "tier_ratio",
    "player_base_health",
    "enemy_health_scale",
    "enemy_damage_scale",
    "max_level",
    "health_step_levels",
    "ability_damage",
    "plain_basic",
    "ranks",
    "group_step",
    "enemy_hit",
    "enemy_regen_percent",
    "defense",
    "player",
    "speeds",
    "stats",
    "ranks",
    "enemy_hit",
];

/// One rank of enemy (`ranks` in the file).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RankRow {
    /// Health, counted in plain basic attacks of the enemy's tier.
    hp_basics: f32,
    /// Multiplier on the tier's normal hit.
    hit: f32,
    /// Whether the rank regains health during a fight.
    regen: bool,
    /// The seconds one player at the enemy's tier should need, lowest and highest; the
    /// simulator test holds the table to it.
    solo_seconds: [f32; 2],
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemyHit {
    /// A normal hit is this share of a player's base health at the tier's first level.
    floor_health_share: f32,
    interval_ms: u32,
    special_factor: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Defense {
    /// One clothing piece's Defense per tier.
    piece: Vec<f32>,
    pieces: f32,
    /// Defense that halves a hit, in full sets of the tier.
    half_sets: f32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CritRow {
    percent: f32,
    factor: f32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Regen {
    fight: f32,
    idle: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlayerRow {
    damage_multiplier: f32,
    ability_crit: CritRow,
    melee_crit: CritRow,
    regen_percent: Regen,
    weapon_range: f32,
    weapon_speed_ms: u32,
    to_hit: u32,
}

/// One row of the 38/7 stat block: the client's stat id and where its value comes from.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StatRow {
    id: i32,
    #[allow(dead_code)]
    name: String,
    from: String,
}

/// What a player brings to the stat block besides the job level.
#[derive(Clone, Copy, Debug, Default)]
pub struct StatInput {
    pub level: u8,
    pub defense: u32,
    pub absorb: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRow {
    field: String,
    #[allow(dead_code)]
    kind: String,
    #[allow(dead_code)]
    evidence: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CombatScale {
    schema_version: u32,
    /// Level-1 basic attack per profile id; key 0 answers every other profile.
    basic_damage: BTreeMap<i32, u32>,
    /// First job level of each weapon tier.
    tier_min_levels: Vec<u32>,
    /// Damage growth per tier, and health growth per four levels.
    tier_ratio: f32,
    player_base_health: u32,
    enemy_health_scale: f32,
    enemy_damage_scale: f32,
    max_level: u32,
    /// Levels per `tier_ratio` step of player health.
    health_step_levels: u32,
    /// Damage per kit ability class and weapon tier.
    ability_damage: BTreeMap<String, Vec<u32>>,
    /// The class enemy health is counted in.
    plain_basic: String,
    ranks: BTreeMap<Rank, RankRow>,
    /// Extra health per group member after the first, as a share.
    group_step: f32,
    enemy_hit: EnemyHit,
    /// Health a regenerating enemy regains per second of a fight, in percent.
    enemy_regen_percent: f32,
    defense: Defense,
    player: PlayerRow,
    /// Walk, run and return-home speed per body class.
    speeds: BTreeMap<String, [f32; 3]>,
    stats: Vec<StatRow>,
    source: Vec<SourceRow>,
    /// The enemy kinds of `enemies.json`; empty when the directory has none.
    #[serde(skip)]
    catalogue: Arc<Catalogue>,
}

impl CombatScale {
    /// The content directory's own file wins; otherwise the shipped one, which
    /// is compiled in, so a server away from its source tree still loads.
    pub(super) fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join(FILE);
        let refuse = |e: &dyn std::fmt::Display| invalid(&format!("{}: {e}", path.display()));
        let bytes = if path.is_file() {
            if fs::metadata(&path).map_err(|e| refuse(&e))?.len() > 16 * 1024 {
                return Err(refuse(&"file too large"));
            }
            fs::read(&path).map_err(|e| refuse(&e))?
        } else {
            BUNDLED.as_bytes().to_vec()
        };
        let mut scale: Self = serde_json::from_slice(&bytes).map_err(|e| refuse(&e))?;
        if !scale.valid() {
            return Err(refuse(&"value outside its sanity bounds"));
        }
        scale.catalogue = Arc::new(Catalogue::load(directory)?);
        Ok(scale)
    }

    /// The shipped file's scale, with no catalogue: what `scale`'s free functions read. The
    /// engine's own paths use the scale it loaded (`self.scale`), which a content directory
    /// may replace; a process-wide "last loaded" would race between engines.
    pub(super) fn shipped() -> Option<Arc<Self>> {
        static SHIPPED: OnceLock<Option<Arc<CombatScale>>> = OnceLock::new();
        let shipped = || serde_json::from_str::<Self>(BUNDLED).ok();
        (SHIPPED.get_or_init(|| shipped().filter(Self::valid).map(Arc::new))).clone()
    }

    fn valid(&self) -> bool {
        let scale = self;
        let scales = [scale.enemy_health_scale, scale.enemy_damage_scale];
        let tiers = &scale.tier_min_levels;
        let count = tiers.len();
        let positive = |v: f32, most: f32| v.is_finite() && v > 0.0 && v <= most;
        let share = |v: f32| (0.0..=100.0).contains(&v);
        let crit = |c: CritRow| share(c.percent) && (1.0..=10.0).contains(&c.factor);
        let (hit, player) = (&scale.enemy_hit, &scale.player);
        let new_rows_hold = (1..=100).contains(&scale.max_level)
            && (1..=100).contains(&scale.health_step_levels)
            && scale.ability_damage.len() <= 32
            && scale.ability_damage.contains_key(&scale.plain_basic)
            && (scale.ability_damage.values())
                .all(|l| l.len() == count && l.iter().all(|d| (1..=1_000_000).contains(d)))
            && scale.ranks.len() == Rank::WorldBoss as usize + 1
            && scale.ranks.values().all(|r| {
                positive(r.hp_basics, 100_000.0)
                    && share(r.hit)
                    && positive(r.solo_seconds[0], r.solo_seconds[1])
                    && r.solo_seconds[1] <= 86_400.0
            })
            && (0.0..=10.0).contains(&scale.group_step)
            && positive(hit.floor_health_share, 1.0)
            && (100..=60_000).contains(&hit.interval_ms)
            && (1.0..=10.0).contains(&hit.special_factor)
            && share(scale.enemy_regen_percent)
            && scale.defense.piece.len() == count
            && scale.defense.piece.iter().all(|p| positive(*p, 100_000.0))
            && positive(scale.defense.pieces, 100.0)
            && positive(scale.defense.half_sets, 100.0)
            && positive(player.damage_multiplier, 100.0)
            && crit(player.ability_crit)
            && crit(player.melee_crit)
            && share(player.regen_percent.fight)
            && share(player.regen_percent.idle)
            && positive(player.weapon_range, 1_000.0)
            && scale.speeds.len() <= 16
            && scale.speeds.values().flatten().all(|s| positive(*s, 100.0))
            && scale.stats.len() <= 71
            && (scale.stats.iter()).all(|row| self.stat(&row.from, StatInput::default()).is_some());
        !(scale.schema_version != 2
            || !new_rows_hold
            || !scale.basic_damage.contains_key(&0)
            || scale
                .basic_damage
                .values()
                .any(|d| !(1..=100_000).contains(d))
            || tiers.first() != Some(&1)
            || tiers.windows(2).any(|w| w[0] >= w[1])
            || tiers.last().is_some_and(|&l| l > scale.max_level)
            || !(1.0..=10.0).contains(&scale.tier_ratio)
            || !(100..=100_000).contains(&scale.player_base_health)
            || scales.iter().any(|s| !(0.01..=100.0).contains(s))
            || FIELDS.iter().any(|f| {
                let rows = scale.source.iter().filter(|s| s.field == *f).count();
                rows < FIELDS.iter().filter(|other| other == &f).count()
            }))
    }

    /// Level-1 basic attack of a job; the engine doubles it for the second slot.
    pub(super) fn basic_damage(&self, profile: i32) -> u32 {
        *self
            .basic_damage
            .get(&profile)
            .unwrap_or(&self.basic_damage[&0])
    }

    /// Weapon tier a job level can wield (1-based).
    fn tier(&self, level: u32) -> i32 {
        self.tier_min_levels
            .iter()
            .filter(|&&l| l <= level)
            .count()
            .max(1) as i32
    }

    /// Damage and heal multiplier of that tier.
    pub(super) fn tier_factor(&self, level: u32) -> f32 {
        self.tier_ratio.powi(self.tier(level) - 1)
    }

    /// Final-era scaling (The owner chose the final era, 2026-10-15): health grows
    /// x`tier_ratio` every four levels (videos: Medic 10 = 2081, Ninja/Archer/
    /// Medic 14 = 3309-3639, Archer 16 = 4476-4883, level 20 = 7116-9821) from
    /// the level-1 base 500 the HUD frames show (Damage Scale.md Q3).
    pub(super) fn health_factor(&self, level: u32) -> f32 {
let steps = (level.clamp(1, self.max_level) - 1) as f32 / self.health_step_levels as f32;
        self.tier_ratio.powf(steps)
    }

    /// A player's base maximum health at a job level (before job identity).
    pub(super) fn level_health(&self, level: u32) -> u32 {
        (self.player_base_health as f32 * self.health_factor(level)).round() as u32
    }

    /// Reach and strike of a job's attack slot (reach is (R) code: a Wizard
    /// and an Archer shoot, the rest are melee; the second slot hits twice).
    pub(super) fn profile_stats(&self, profile: i32, slot: i32) -> (f32, u32) {
        let range = match profile {
            35 => 30.0,
            12 => 20.0,
            _ => 3.0,
        };
        let damage = self.basic_damage(profile);
        (range, if slot == 0 { damage } else { damage * 2 })
    }

    pub(super) fn player_base_health(&self) -> u32 {
        self.player_base_health
    }

    /// Scale an authored enemy row (level-one baselines) onto the live scale:
    /// its own level or difficulty factors first, then the global enemy
    /// scales, rounded once at the end.
    pub(super) fn scale_enemy(&self, a: &mut Actor, health_factor: f32, damage_factor: f32) {
        let health = a.max_health as f32 * health_factor * self.enemy_health_scale;
        let damage = a.damage as f32 * damage_factor * self.enemy_damage_scale;
        a.max_health = health.round() as u32;
        a.damage = damage.round() as u32;
    }

    /// A loaded world or dungeon row (R, T737): its authored health is a
    /// level-one baseline, so its own content level scales it like a player's.
    /// Damage stays as authored; no video measures enemy damage. A ranked kind
    /// takes its rank's numbers at the level's tier instead.
    pub(super) fn scale_world_enemy(&self, a: &mut Actor) {
        if !self.rank_enemy(a, self.tier(a.level) as u8) {
            self.scale_enemy(a, self.health_factor(a.level), 1.0);
        }
    }

    /// A loaded field enemy: its content level scales health on the player
    /// curve and its weapon-tier damage (R, T737); a ranked kind as above.
    pub(super) fn scale_field_enemy(&self, a: &mut Actor) {
        if !self.rank_enemy(a, self.tier(a.level) as u8) {
            self.scale_enemy(a, self.health_factor(a.level), self.tier_factor(a.level));
        }
    }

    /// A battle's enemy: a ranked kind fights at the battle's difficulty as its tier (battles
    /// are not scaled to the player); the rest scale like a player at the difficulty's level
    /// gate, so each fight keeps the same share of danger (R).
    pub(super) fn scale_battle_enemy(&self, a: &mut Actor, difficulty: i32) {
        let tier = difficulty.clamp(1, i32::from(self.tiers())) as u8;
        if !self.rank_enemy(a, tier) {
            let factor = self.health_factor(difficulty_level(difficulty));
            self.scale_enemy(a, factor, factor);
        }
    }

    /// Give a row of a ranked kind its catalogue numbers, or a row that names its own `rank`
    /// that rank's numbers at the tier; false when neither has a rank.
    fn rank_enemy(&self, a: &mut Actor, tier: u8) -> bool {
        let own = |rank| (self.enemy_health(rank, tier, 1), self.enemy_hit(rank, tier));
        let Some((health, hit)) = (self.catalogue.by_kind(a.definition_id))
            .and_then(|definition| self.kind_numbers(definition, Some(tier), 1))
            .or_else(|| a.rank.map(own))
        else {
            return false;
        };
        (a.max_health, a.damage) = (health.max(1), hit);
        true
    }

    pub(super) fn catalogue(&self) -> &Catalogue {
        &self.catalogue
    }

    /// Health and raw hit of a catalogue kind at a tier (its own when none is given) for a
    /// group of `group`; `None` for a kind without a rank. A number the definition states
    /// outright wins over the rank's.
    pub(super) fn kind_numbers(
        &self,
        definition: &Definition,
        tier: Option<u8>,
        group: u8,
    ) -> Option<(u32, u32)> {
        let rank = definition.rank?;
        let tier = tier.or(definition.tier).unwrap_or(1);
        let health = match definition.health {
            Some(own) => (own as f32 * self.group_factor(group)).round() as u32,
            None => self.enemy_health(rank, tier, group),
        };
        let hit = definition.hit;
        Some((health, hit.unwrap_or_else(|| self.enemy_hit(rank, tier))))
    }

    fn tier_index(&self, tier: u8) -> usize {
        usize::from(tier).clamp(1, self.tier_min_levels.len()) - 1
    }

    fn group_factor(&self, group: u8) -> f32 {
        1.0 + self.group_step * f32::from(group.max(1) - 1)
    }

    /// Health of an enemy: plain basics of the tier times the rank's count, times the group
    /// factor.
    pub(super) fn enemy_health(&self, rank: Rank, tier: u8, group: u8) -> u32 {
        let plain = self.ability_damage(&self.plain_basic, tier) as f32;
        let basics = self.ranks.get(&rank).map_or(0.0, |row| row.hp_basics);
        (plain * basics * self.group_factor(group)).round() as u32
    }

    /// A normal hit of the tier before the rank's multiplier: the share of a player's base
    /// health at the tier's first level. A kind whose own hit was captured states it.
    fn tier_hit(&self, tier: u8) -> f32 {
        let floor = self.tier_min_levels[self.tier_index(tier)];
        (self.level_health(floor) as f32 * self.enemy_hit.floor_health_share).round()
    }

    /// An enemy's normal hit, raw (before the target's Defense).
    pub(super) fn enemy_hit(&self, rank: Rank, tier: u8) -> u32 {
        let by_rank = self.ranks.get(&rank).map_or(0.0, |row| row.hit);
        (self.tier_hit(tier) * by_rank).round() as u32
    }

    /// A special hit made from a normal one (the wire marks it with its own effect).
    pub(super) fn special_hit(&self, raw: u32) -> u32 {
        (raw as f32 * self.enemy_hit.special_factor).round() as u32
    }

    pub(super) fn enemy_hit_interval_ms(&self) -> u32 {
        self.enemy_hit.interval_ms
    }

    /// Health a rank regains per second of a fight.
    pub(super) fn enemy_regen(&self, rank: Rank, max_health: u32) -> u32 {
        if !self.ranks.get(&rank).is_some_and(|row| row.regen) {
            return 0;
        }
        (max_health as f32 * self.enemy_regen_percent / 100.0).round() as u32
    }

    /// The seconds one player should need for a rank at his own tier, lowest and highest.
    pub(super) fn solo_seconds(&self, rank: Rank) -> [f32; 2] {
        (self.ranks.get(&rank)).map_or([0.0; 2], |row| row.solo_seconds)
    }

    /// Damage of a kit ability class at a weapon tier; 0 for a class the table does not hold.
    pub(super) fn ability_damage(&self, class: &str, tier: u8) -> u32 {
        (self.ability_damage.get(class)).map_or(0, |ladder| ladder[self.tier_index(tier)])
    }

    /// Damage a player takes from a raw hit: Defense removes `D / (D + K)` of it, where K is
    /// `half_sets` full clothing sets of the tier, then Absorb comes off flat.
    pub(super) fn mitigate(&self, raw: u32, defense: u32, absorb: u32, tier: u8) -> u32 {
        let set = self.defense.pieces * self.defense.piece[self.tier_index(tier)];
        let (k, defense) = (self.defense.half_sets * set, defense as f32);
        let taken = (raw as f32 * (1.0 - defense / (defense + k))).round() as u32;
        taken.saturating_sub(absorb)
    }

    /// Walk, run and return-home speed of a body class (`small`, `pack`, `large`).
    pub(super) fn speeds(&self, class: &str) -> Option<[f32; 3]> {
        self.speeds.get(class).copied()
    }

    /// The weapon tier (1-based) a job level reaches.
    pub(super) fn tier_of_level(&self, level: u8) -> u8 {
        self.tier(u32::from(level)) as u8
    }

    /// A player's base maximum health at a job level, the same for every job.
    pub(super) fn player_health(&self, level: u8) -> u32 {
        self.level_health(u32::from(level))
    }

    pub(super) fn max_level(&self) -> u32 {
        self.max_level
    }

    /// How many weapon tiers the file holds.
    pub(super) fn tiers(&self) -> u8 {
        self.tier_min_levels.len() as u8
    }

    fn stat(&self, from: &str, of: StatInput) -> Option<wire::StatValue> {
        use wire::StatValue::{Float, Int};
        let player = &self.player;
        let health = self.level_health(u32::from(of.level.max(1))) as f32;
        let percent = |share: f32| Int((health * share / 100.0).round() as i32);
        Some(match from {
            "health" => Int(health as i32),
            "defense" => Int(of.defense as i32),
            "absorb" => Int(of.absorb as i32),
            "weapon_range" => Float(player.weapon_range),
            "weapon_speed_ms" => Int(player.weapon_speed_ms as i32),
            "to_hit" => Int(player.to_hit as i32),
            "damage_multiplier" => Float(player.damage_multiplier),
            "melee_crit_percent" => Int(player.melee_crit.percent.round() as i32),
            "melee_crit_factor" => Float(player.melee_crit.factor),
            "ability_crit_percent" => Int(player.ability_crit.percent.round() as i32),
            "ability_crit_factor" => Float(player.ability_crit.factor),
            "regen_fight" => percent(player.regen_percent.fight),
            "regen_idle" => percent(player.regen_percent.idle),
            _ => return None,
        })
    }

    /// The combat rows of a player's 38/7 stat block, ready for `wire::stats`.
    pub(super) fn stat_rows(&self, of: StatInput) -> Vec<(i32, wire::StatValue)> {
        (self.stats.iter())
            .filter_map(|row| Some((row.id, self.stat(&row.from, of)?)))
            .collect()
    }
}
