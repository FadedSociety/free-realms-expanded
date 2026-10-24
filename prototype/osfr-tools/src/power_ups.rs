//! Combat power-up drop tables: content schema, bounded loader, pure roll.
//!
//! Slice 1 of `notes/Research/Combat Power-ups.md` (W153). Nothing in the
//! client holds drop chances, weights or effect sizes, so every number in
//! `data/content/power_ups.json` is reconstructed policy (R) except the 15 %
//! and 50 % heals and the 100 energy read from the locale dev notes. Values
//! stay live-tunable in the file; the bounds below are sanity limits that stop
//! a bad edit from breaking a fight, not pins on the tuning. Spawning,
//! pickup and use are later slices and live in the encounter code.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

const MAX_JSON_BYTES: u64 = 1_048_576;
const MAX_KINDS: usize = 64;
const MAX_TABLES: usize = 32;
const MAX_POOL: usize = 64;
const MAX_GUARANTEED: usize = 8;
/// Battle tiers 1 to 5 (dungeon / field battle `difficulty`).
pub const TIERS: usize = 5;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Area {
    pub radius: f64,
    pub max_targets: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Burn {
    pub multiplier: f64,
    pub duration_ms: u32,
    pub interval_ms: u32,
}

/// One power-up kind. Optional fields are the effects the kind has; which are
/// present decides what the later slices do with it.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Kind {
    pub model_id: u32,
    /// Applied on pickup and never held (health, energy).
    pub auto: bool,
    pub name_id: Option<u32>,
    pub icon_id: Option<u32>,
    /// Pickup model scale.
    pub scale: Option<f64>,
    pub heal_pct_max: Option<u32>,
    #[serde(default)]
    pub group: bool,
    pub energy: Option<u32>,
    pub area: Option<Area>,
    pub damage_by_tier: Option<Vec<f64>>,
    pub burn: Option<Burn>,
    pub stun_ms: Option<u32>,
    pub duration_ms: Option<u32>,
    #[serde(default)]
    pub invulnerable: bool,
    pub speed_pct: Option<u32>,
    pub source: String,
}

/// One independent roll: succeeds with `chance`, then picks a pool entry by weight.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Roll {
    pub chance: f64,
    pub pool: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub health_roll: Option<Roll>,
    pub other_roll: Option<Roll>,
    #[serde(default)]
    pub guaranteed: Vec<String>,
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pickup {
    pub ttl_ms: u32,
    pub radius: f64,
    pub max_alive_per_instance: u32,
    pub source: String,
}

/// Zero-based job-toolbar index a held power-up occupies and the client sends
/// when the player presses its key: the third slot, `ActionBarSlot3` (default
/// `3`; binding table 0x1b4198c row 21 keysym 0x33). The regular-controller
/// input dispatcher `00b02880` case 21 calls `DispatchActionBarSlot`
/// `008a7eb0` with slot 2, and the `AbilityProcessor` sends 36/10 bar 1 slot 2
/// (writer `00a342e0`). The client has no separate combat power-up input: the
/// `UsePowerup` action (row 15, default Control_L) belongs to the demolition
/// derby/minigame controllers and is never dispatched by `00b02880`. The
/// locale dev note "power ups - add super shield to bar" makes the slot
/// server-owned, so the held kind uses its own key here. (T222 research.)
pub const SLOT: i32 = 2;

/// Base of the synthetic 36/13 definition id a held kind is shown under,
/// clear of the authored job definitions (3784..24097) the client also
/// caches. (R) The original server's definition id is unrecovered.
pub const DEFINITION_BASE: i32 = 90_000_000;

/// One held kind as the client's own key slot needs it: the synthetic
/// definition id, the client name id and the icon id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub definition_id: i32,
    pub name_id: i32,
    pub icon_id: i32,
}

/// Who shares a kill (R, W158 slice 2b): every player who dealt damage to the
/// enemy within `window_ms` (or was its target) gets their own roll. When more
/// than one player took part, each roll's chances are multiplied by
/// `group_chance_multiplier` (1.0 = a full roll each) so a crowd does not flood
/// the ground. Guaranteed drops are not scaled.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Participation {
    pub window_ms: u32,
    pub group_chance_multiplier: f64,
    pub source: String,
}

/// Used when `power_ups.json` has no `participation` block.
const DEFAULT_WINDOW_MS: u32 = 30_000;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PowerUps {
    pub schema_version: u32,
    pub kinds: BTreeMap<String, Kind>,
    pub tables: BTreeMap<String, Table>,
    pub pickup: Pickup,
    #[serde(default)]
    pub participation: Option<Participation>,
}

impl PowerUps {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_JSON_BYTES + 1)
            .read_to_end(&mut bytes)?;
        Self::from_json(&bytes)
    }

    /// Parse and validate. An `Err` means the caller keeps its previous tables.
    pub fn from_json(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() as u64 > MAX_JSON_BYTES {
            return Err(invalid("power-up file exceeds size limit"));
        }
        let file: Self =
            serde_json::from_slice(bytes).map_err(|error| invalid(&error.to_string()))?;
        file.validate()?;
        Ok(file)
    }

    pub fn table(&self, id: &str) -> Option<&Table> {
        self.tables.get(id)
    }

    pub fn kind(&self, id: &str) -> Option<&Kind> {
        self.kinds.get(id)
    }

    /// The held kind's own key-slot record. `None` for an unknown id or a kind
    /// that cannot be shown (validation requires a stored kind to carry a name
    /// and icon, so an accepted file always resolves one).
    pub fn slot(&self, id: &str) -> Option<Slot> {
        let kind = self.kind(id)?;
        Some(Slot {
            definition_id: DEFINITION_BASE.checked_add(i32::try_from(kind.name_id?).ok()?)?,
            name_id: i32::try_from(kind.name_id?).ok()?,
            icon_id: i32::try_from(kind.icon_id?).ok()?,
        })
    }

    /// How long after its last hit a player still shares a kill.
    pub fn participation_window_ms(&self) -> u64 {
        u64::from(
            self.participation
                .as_ref()
                .map_or(DEFAULT_WINDOW_MS, |p| p.window_ms),
        )
    }

    /// Roll-chance factor when several players share a kill.
    pub fn group_chance_multiplier(&self) -> f64 {
        self.participation
            .as_ref()
            .map_or(1.0, |p| p.group_chance_multiplier)
    }

    fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1
            || self.kinds.len() > MAX_KINDS
            || self.tables.len() > MAX_TABLES
        {
            return Err(invalid("unsupported or oversized power-up file"));
        }
        let p = &self.pickup;
        if !(5_000..=120_000).contains(&p.ttl_ms)
            || !in_range(p.radius, 0.5, 5.0)
            || !(1..=32).contains(&p.max_alive_per_instance)
            || !sourced(&p.source)
        {
            return Err(invalid("pickup settings out of bounds"));
        }
        if let Some(share) = &self.participation {
            if !(1_000..=300_000).contains(&share.window_ms)
                || !in_range(share.group_chance_multiplier, 0.05, 1.0)
                || !sourced(&share.source)
            {
                return Err(invalid("participation settings out of bounds"));
            }
        }
        for (id, kind) in &self.kinds {
            if id.trim().is_empty() || !kind.valid() {
                return Err(invalid(&format!("power-up kind {id:?} is invalid")));
            }
        }
        for (id, table) in &self.tables {
            if id.trim().is_empty() || !table.valid(&self.kinds) {
                return Err(invalid(&format!("power-up table {id:?} is invalid")));
            }
        }
        Ok(())
    }
}

impl Kind {
    fn valid(&self) -> bool {
        let area = self
            .area
            .as_ref()
            .is_none_or(|a| in_range(a.radius, 0.5, 30.0) && (1..=16).contains(&a.max_targets));
        let burn = self.burn.as_ref().is_none_or(|b| {
            in_range(b.multiplier, 0.01, 10.0)
                && (500..=30_000).contains(&b.duration_ms)
                && (500..=b.duration_ms).contains(&b.interval_ms)
        });
        let tiers = self
            .damage_by_tier
            .as_ref()
            .is_none_or(|t| t.len() == TIERS && t.iter().all(|&x| in_range(x, 0.01, 1000.0)));
        self.model_id > 0
            && sourced(&self.source)
            // A stored kind is shown on the client's key slot, so it needs a
            // name and an icon; automatic kinds are applied at the pickup.
            && (self.auto
                || (self.name_id.is_some_and(|v| v > 0)
                    && self.icon_id.is_some_and(|v| v > 0)))
            && self.name_id.is_none_or(|v| i32::try_from(v).is_ok())
            && self.icon_id.is_none_or(|v| i32::try_from(v).is_ok())
            && self.scale.is_none_or(|s| in_range(s, 0.25, 4.0))
            && self.heal_pct_max.is_none_or(|v| (1..=100).contains(&v))
            && self.energy.is_none_or(|v| (1..=100).contains(&v))
            && self.stun_ms.is_none_or(|v| (500..=10_000).contains(&v))
            && self.duration_ms.is_none_or(|v| (500..=30_000).contains(&v))
            && self.speed_pct.is_none_or(|v| v <= 100)
            && area
            && burn
            && tiers
    }
}

impl Table {
    fn valid(&self, kinds: &BTreeMap<String, Kind>) -> bool {
        sourced(&self.source)
            && self.guaranteed.len() <= MAX_GUARANTEED
            && self.guaranteed.iter().all(|k| kinds.contains_key(k))
            && [&self.health_roll, &self.other_roll]
                .into_iter()
                .flatten()
                .all(|r| r.valid(kinds))
    }
}

impl Roll {
    fn valid(&self, kinds: &BTreeMap<String, Kind>) -> bool {
        in_range(self.chance, 0.0, 0.5)
            && self.pool.len() <= MAX_POOL
            && self.pool.keys().all(|k| kinds.contains_key(k))
            && self.pool.values().all(|&w| w <= 1000)
            && self.pool.values().any(|&w| w > 0)
    }
}

/// Small seedable generator (SplitMix64) shared by drop rolls and the chess
/// computer's move choice; both need a reproducible stream in tests. This is
/// not a CSPRNG: in chess it is only an anti-replay stream (an observer that
/// recovered the state could predict the computer's replies, and the bounded
/// daily coin cap limits what that is worth), and drops are not an economy
/// secret. The seed comes from the OS in production, so a new game is not
/// predictable before its first replies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng(u64);

impl Rng {
    pub fn seeded(seed: u64) -> Self {
        Self(seed)
    }

    pub fn from_entropy() -> io::Result<Self> {
        let mut seed = [0u8; 8];
        getrandom::getrandom(&mut seed).map_err(|_| invalid("no OS randomness"))?;
        Ok(Self(u64::from_le_bytes(seed)))
    }

    /// SplitMix64 finalizer. `next_u64` applies it to each draw; chess also
    /// uses it to derive one table seed from the process seed.
    pub fn mix(mut value: u64) -> u64 {
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        Self::mix(self.0)
    }

    /// Uniform in `[0, 1)` with 53 bits.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Kinds a defeated enemy drops: the guaranteed list, then the health roll,
/// then the other roll (R: two independent rolls, mirroring the two original
/// reactive procs). Pure: the same table and RNG state give the same drops.
pub fn roll<'a>(table: &'a Table, rng: &mut Rng) -> Vec<&'a str> {
    roll_scaled(table, rng, 1.0)
}

/// `roll` with every roll chance multiplied by `chance_scale` (shared kills).
pub fn roll_scaled<'a>(table: &'a Table, rng: &mut Rng, chance_scale: f64) -> Vec<&'a str> {
    let mut drops: Vec<&str> = table.guaranteed.iter().map(String::as_str).collect();
    for r in [&table.health_roll, &table.other_roll]
        .into_iter()
        .flatten()
    {
        if rng.next_f64() < r.chance * chance_scale {
            drops.extend(pick(&r.pool, rng));
        }
    }
    drops
}

/// Weighted pick. `None` only for an all-zero pool, which validation rejects.
fn pick<'a>(pool: &'a BTreeMap<String, u32>, rng: &mut Rng) -> Option<&'a str> {
    let total: u64 = pool.values().map(|&w| u64::from(w)).sum();
    if total == 0 {
        return None;
    }
    let mut at = rng.next_u64() % total;
    for (kind, &weight) in pool {
        let weight = u64::from(weight);
        if at < weight {
            return Some(kind);
        }
        at -= weight;
    }
    None
}

fn in_range(value: f64, low: f64, high: f64) -> bool {
    value.is_finite() && (low..=high).contains(&value)
}

fn sourced(text: &str) -> bool {
    !text.trim().is_empty() && text.len() <= 512 && !text.chars().any(char::is_control)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}
