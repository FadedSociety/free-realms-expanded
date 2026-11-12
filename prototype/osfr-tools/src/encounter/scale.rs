//! Combat numbers by rank and tier: the one place kits, the brain and the director ask for
//! health and damage (combat design section 5), and the enemy catalogue (section 7). Piece P4
//! (`combat-scale`) owns this file.
//!
//! Frozen by the combat seam: the first six signatures below do not change. Every number
//! they answer comes from the shipped `combat_scale.json` (`combat_scale.rs` reads and checks
//! it). Engine code asks its own loaded scale instead (`self.scale.enemy_health(..)` and the
//! like), which a content directory may retune and which holds the catalogue.
use super::combat_scale::CombatScale;
pub use super::combat_scale::StatInput;
use super::*;

mod catalogue;
pub use self::catalogue::{Catalogue, Definition, Speeds};

fn tables() -> Option<Arc<CombatScale>> {
    CombatScale::shipped()
}

/// Health of an enemy: plain basics of the tier times the rank's count, times the group
/// factor (a share more per member after the first).
pub fn enemy_health(rank: Rank, tier: u8, group: u8) -> u32 {
    tables().map_or(0, |t| t.enemy_health(rank, tier, group))
}

/// An enemy's normal hit, raw (before the target's Defense).
pub fn enemy_hit(rank: Rank, tier: u8) -> u32 {
    tables().map_or(0, |t| t.enemy_hit(rank, tier))
}

/// An enemy's own stated health (a fight row's or kind's), with the owner's health multiplier.
pub fn live_health(health: u32) -> u32 {
    tables().map_or(health, |t| t.live_health(health as f32))
}

/// An enemy's own stated raw hit, with the owner's damage multiplier.
pub fn live_hit(hit: u32) -> u32 {
    tables().map_or(hit, |t| t.live_hit(hit as f32))
}

/// Damage of a kit ability class (`combat_scale.json` `ability_damage`) at a weapon tier; 0
/// for a class the table does not hold.
pub fn ability_damage(class: &str, tier: u8) -> u32 {
    tables().map_or(0, |t| t.ability_damage(class, tier))
}

/// Damage a player takes from a raw hit after Defense and then Absorb.
pub fn mitigate(raw: u32, defense: u32, absorb: u32, tier: u8) -> u32 {
    tables().map_or(raw, |t| t.mitigate(raw, defense, absorb, tier))
}

/// A player's base maximum health at a job level, the same for every job.
pub fn player_health(level: u8) -> u32 {
    tables().map_or(0, |t| t.player_health(level))
}

/// The weapon tier (1-based) a job level reaches.
pub fn tier_of_level(level: u8) -> u8 {
    tables().map_or(1, |t| t.tier_of_level(level))
}

// What follows is new with P4; the six above are the frozen seam.

/// A special hit made from a normal one, and the time between an enemy's hits.
pub fn special_hit(raw: u32) -> u32 {
    tables().map_or(raw, |t| t.special_hit(raw))
}

pub fn enemy_hit_interval_ms() -> u32 {
    tables().map_or(0, |t| t.enemy_hit_interval_ms())
}

/// Health an enemy of the rank regains per second of a fight (0 for a rank that does not).
pub fn enemy_regen(rank: Rank, max_health: u32) -> u32 {
    tables().map_or(0, |t| t.enemy_regen(rank, max_health))
}

/// Walk, run and return-home speed of a body class (`small`, `pack`, `large`).
pub fn speeds(class: &str) -> Option<[f32; 3]> {
    tables().and_then(|t| t.speeds(class))
}

/// The combat rows of a player's 38/7 stat block, ready for `wire::stats`.
pub fn stat_rows(of: StatInput) -> Vec<(i32, wire::StatValue)> {
    tables().map_or_else(Vec::new, |t| t.stat_rows(of))
}

/// One line of the kill table: a job at a level against one enemy.
#[derive(Clone, Debug, PartialEq)]
pub struct KillLine {
    pub level: u8,
    pub tier: u8,
    /// What one basic attack deals.
    pub strike: u32,
    pub enemy_health: u32,
    pub hits: u32,
    pub seconds: f32,
    /// The enemy's raw hit and how many of them empty a player with no Defense.
    pub enemy_hit: u32,
    pub hits_to_fall: u32,
    pub seconds_to_fall: f32,
}

impl EncounterEngine {
    /// The post-load hook of the combat seam. The catalogue is not read here but inside
    /// `CombatScale::load`: this hook runs after the world and dungeon rows have been scaled
    /// and the saved enemies restored against them, and those rows need the catalogue.
    pub(super) fn load_scale(&mut self, _directory: &Path) -> io::Result<()> {
        Ok(())
    }

    /// Kill-fact fields this piece knows (the catalogue's rank and tags of the kind). A kind
    /// without a rank keeps what the engine saw (boss or normal).
    pub(super) fn stamp_kill_scale(&self, fact: &mut KillFact) {
        let kind = i32::try_from(fact.kind).unwrap_or(0);
        let Some(definition) = self.scale.catalogue().by_kind(kind) else {
            return;
        };
        if let Some(rank) = definition.rank {
            fact.rank = rank;
        }
        for tag in &definition.tags {
            if !fact.tags.contains(tag) {
                fact.tags.push(tag.clone());
            }
        }
    }

    /// The kill table of the balance simulator: a player of `profile` at each of `levels`
    /// swinging basics at the gate's rhythm against an enemy of `rank`. The enemy stands at
    /// `tier`, or at the player's own tier when none is given; `health` and `hit` replace the
    /// rank's numbers (a catalogue kind that states its own). Nothing here touches a fight.
    pub fn kill_table(
        &self,
        profile: i32,
        rank: Rank,
        tier: Option<u8>,
        own: (Option<u32>, Option<u32>),
        levels: &[u8],
    ) -> Vec<KillLine> {
        let scale = &self.scale;
        let gate = self.rules.basic_gate_ms as f32 / 1000.0;
        let between_hits = scale.enemy_hit_interval_ms() as f32 / 1000.0;
        (levels.iter())
            .map(|&level| {
                let own_tier = scale.tier_of_level(level);
                let tier = tier.unwrap_or(own_tier);
                let basic = scale.basic_damage(profile) as f32;
                let strike = (basic * scale.tier_factor(u32::from(level))).round() as u32;
                let enemy_health = own.0.unwrap_or_else(|| scale.enemy_health(rank, tier, 1));
                let enemy_hit = own.1.unwrap_or_else(|| scale.enemy_hit(rank, tier));
                let hits = enemy_health.div_ceil(strike.max(1));
                let health = scale.level_health(u32::from(level));
                let hits_to_fall = health.div_ceil(enemy_hit.max(1)) * enemy_hit.min(1);
                KillLine {
                    level,
                    tier,
                    strike,
                    enemy_health,
                    hits,
                    seconds: hits as f32 * gate,
                    enemy_hit,
                    hits_to_fall,
                    seconds_to_fall: hits_to_fall as f32 * between_hits,
                }
            })
            .collect()
    }

    /// The lowest and highest seconds the scale file allows one player for a rank.
    pub fn solo_seconds(&self, rank: Rank) -> [f32; 2] {
        self.scale.solo_seconds(rank)
    }

    /// The catalogue's kind by `id` with its health and raw hit at a tier (the kind's own
    /// when `None`) for a group of `group`; `None` for an unknown kind or one without a rank.
    pub fn catalogue_kind(
        &self,
        id: &str,
        tier: Option<u8>,
        group: u8,
    ) -> Option<(Definition, (u32, u32))> {
        let definition = self.scale.catalogue().by_id(id)?;
        let numbers = self.scale.kind_numbers(definition, tier, group)?;
        Some((definition.clone(), numbers))
    }

    /// Every kind of the loaded catalogue.
    pub fn catalogue_kinds(&self) -> &[Definition] {
        self.scale.catalogue().kinds()
    }

    /// The job levels and the weapon tiers of the loaded scale (the simulator's sweep).
    pub fn scale_span(&self) -> (u8, u8) {
        (self.scale.max_level() as u8, self.scale.tiers())
    }
}
