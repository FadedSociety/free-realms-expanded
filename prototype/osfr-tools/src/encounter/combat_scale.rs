//! The combat scale: how hard players hit, how tough they are, which levels
//! open each weapon tier, and how enemy rows follow. Everything comes from
//! `combat_scale.json` (`notes/Research/Damage Scale.md`); a missing or invalid
//! file refuses the load, there is no built-in default.
use super::*;

const FILE: &str = "combat_scale.json";
/// Every value carries a source row naming it.
const FIELDS: [&str; 6] = [
    "basic_damage",
    "tier_min_levels",
    "tier_ratio",
    "player_base_health",
    "enemy_health_scale",
    "enemy_damage_scale",
];

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
    source: Vec<SourceRow>,
}

impl CombatScale {
    /// The content directory's own file wins; otherwise the bundled one, the
    /// same rule gameplay uses for the field content. Neither is an error.
    pub(super) fn load(directory: &Path) -> io::Result<Self> {
        let mut path = directory.join(FILE);
        if !path.is_file() {
            path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("data/content")
                .join(FILE);
        }
        let refuse = |e: &dyn std::fmt::Display| invalid(&format!("{}: {e}", path.display()));
        if fs::metadata(&path).map_err(|e| refuse(&e))?.len() > 16 * 1024 {
            return Err(refuse(&"file too large"));
        }
        let scale: Self = serde_json::from_slice(&fs::read(&path).map_err(|e| refuse(&e))?)
            .map_err(|e| refuse(&e))?;
        let scales = [scale.enemy_health_scale, scale.enemy_damage_scale];
        let tiers = &scale.tier_min_levels;
        if scale.schema_version != 1
            || !scale.basic_damage.contains_key(&0)
            || scale
                .basic_damage
                .values()
                .any(|d| !(1..=100_000).contains(d))
            || tiers.first() != Some(&1)
            || tiers.windows(2).any(|w| w[0] >= w[1])
            || tiers.last().is_some_and(|&l| l > 20)
            || !(1.0..=10.0).contains(&scale.tier_ratio)
            || !(100..=100_000).contains(&scale.player_base_health)
            || scales.iter().any(|s| !(0.01..=100.0).contains(s))
            || FIELDS
                .iter()
                .any(|f| !scale.source.iter().any(|s| s.field == *f))
        {
            return Err(refuse(&"value outside its sanity bounds"));
        }
        Ok(scale)
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

    /// Final-era scaling (the owner chose the final era, 2026-10-15): health grows
    /// x`tier_ratio` every four levels (videos: Medic 10 = 2081, Ninja/Archer/
    /// Medic 14 = 3309-3639, Archer 16 = 4476-4883, level 20 = 7116-9821) from
    /// the level-1 base 500 the HUD frames show (Damage Scale.md Q3).
    pub(super) fn health_factor(&self, level: u32) -> f32 {
        self.tier_ratio.powf((level.clamp(1, 20) - 1) as f32 / 4.0)
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
    /// Damage stays as authored; no video measures enemy damage.
    pub(super) fn scale_world_enemy(&self, a: &mut Actor) {
        self.scale_enemy(a, self.health_factor(a.level), 1.0);
    }

    /// A loaded field enemy: its content level scales health on the player
    /// curve and its weapon-tier damage (R, T737).
    pub(super) fn scale_field_enemy(&self, a: &mut Actor) {
        self.scale_enemy(a, self.health_factor(a.level), self.tier_factor(a.level));
    }
}
