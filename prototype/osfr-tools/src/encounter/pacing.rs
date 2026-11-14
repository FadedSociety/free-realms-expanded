//! Instance pacing (the owner, rc-0136, 2026-11-11). The instance fights a player enters
//! from the overworld (battle starters and field-battle markers; never a dungeon door) take
//! 60 to 150 seconds solo at their recommended level. `instance_pacing.json` holds a row per
//! fight id:
//!
//! - `waves`: how many waves come in all. A fight file's own waves are cycled after its last
//!   one (`Fight::pace`); a `field_battles.json` row's roster comes back as a new round.
//! - `wave_gap_ms`: a wave that waits on another's clearing (or the next round) comes this
//!   long after the clear, so the player catches breath.
//! - `max_attackers`: at most this many enemies swing at one player at once; the rest hold.
//! - `attack_ms`: an enemy here swings at most this often. The captured rhythm is 2.3 s
//!   between bites (`combat_scale.json` `enemy_hit`), the behaviour presets' 1 s is not.
//! - `heals`: the battle's health power-ups drop more often, never without limit: a chance per
//!   kill, a cooldown between hearts and a per-run budget of health they may hold in all. The
//!   kind comes from the `standard` table's own health pool (small and big hearts).
//!
//! Every value is data (Studio, `studio-fields.md`); a fight with no row keeps its own pace.
use super::*;

pub const FILE: &str = "instance_pacing.json";
const MAX_BYTES: u64 = 64 * 1024;
const MAX_ROWS: usize = 128;
/// The drop table whose health pool a paced heart is drawn from.
const HEART_TABLE: &str = "standard";

/// One fight's pacing.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pacing {
    /// Waves in all (the fight's own count or fewer keeps its own).
    #[serde(default)]
    pub waves: u32,
    /// The pause between a wave's clearing and the next wave (or round).
    #[serde(default)]
    pub wave_gap_ms: u64,
    /// Enemies that may swing at one player at once (0: no cap).
    #[serde(default)]
    pub max_attackers: u32,
    /// The shortest time between one enemy's swings (0: its own).
    #[serde(default)]
    pub attack_ms: u64,
    #[serde(default)]
    pub heals: Option<HealDrops>,
    pub source: String,
}

/// A paced fight's heart drops.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HealDrops {
    /// Chance a kill drops a heart.
    pub chance: f64,
    /// Health the hearts of one run may hold in all, in percent of the player's maximum.
    pub budget_pct: u32,
    /// No heart drops within this long of the last one.
    pub cooldown_ms: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema_version: u32,
    #[allow(dead_code)]
    source: Vec<serde_json::Value>,
    fights: BTreeMap<u32, Pacing>,
}

/// The loaded table: fight id to its pacing.
#[derive(Clone, Debug, Default)]
pub struct PacingTable(BTreeMap<u32, Pacing>);

impl PacingTable {
    /// `instance_pacing.json` of a content directory; none there is an empty table.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join(FILE);
        if !path.is_file() {
            return Ok(Self::default());
        }
        let refuse = |why: &str| invalid(&format!("{FILE}: {why}"));
        if fs::metadata(&path)?.len() > MAX_BYTES {
            return Err(refuse("file too large"));
        }
        let file: File =
            serde_json::from_slice(&fs::read(&path)?).map_err(|e| refuse(&e.to_string()))?;
        if file.schema_version != 1 || file.fights.len() > MAX_ROWS {
            return Err(refuse("schema_version 1 and at most 128 rows"));
        }
        for (id, p) in &file.fights {
            let heals_hold = p.heals.as_ref().is_none_or(|h| {
                (0.0..=1.0).contains(&h.chance) && h.budget_pct <= 1_000 && h.cooldown_ms <= 600_000
            });
            if p.waves as usize > director::fight::MAX_WAVES
                || p.wave_gap_ms > 120_000
                || p.max_attackers > 16
                || p.attack_ms > 60_000
                || !heals_hold
                || !p.source.contains("(R)")
            {
                return Err(refuse(&format!(
                    "{id}: a value outside its bounds, or no (R) source"
                )));
            }
        }
        Ok(Self(file.fights))
    }

    pub fn get(&self, id: u32) -> Option<&Pacing> {
        self.0.get(&id)
    }

    pub fn ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.0.keys().copied()
    }
}

/// What a paced run has spent so far (session only, reset with the arena).
#[derive(Clone, Debug, Default)]
pub(super) struct PaceRun {
    /// Health the dropped hearts hold, in percent of max health, and when the next may drop.
    heal_given_pct: u32,
    heal_ready_ms: u64,
    /// Rounds a `field_battles.json` row has brought past its first, and the next one's time.
    rounds: u32,
    next_round_ms: Option<u64>,
}

impl EncounterEngine {
    /// The pacing of the run in progress, if its fight has a row: its own, or that of the fight
    /// an entrance runs it as (an entrance that runs a battle row goes by its own id).
    pub(super) fn pacing(&self) -> Option<&Pacing> {
        let id = self.state.active.as_ref()?.encounter_id;
        let table = &self.fight_state.pacing;
        table
            .get(id)
            .or_else(|| table.get(self.fight_state.doors.fight(id)))
    }

    /// The paced cap on enemies swinging at one player at once.
    pub(super) fn max_attackers(&self) -> Option<u32> {
        self.pacing().map(|p| p.max_attackers).filter(|&k| k > 0)
    }

    /// The paced floor on the time between one enemy's swings (0: none).
    pub(super) fn paced_attack_ms(&self) -> u64 {
        self.pacing().map_or(0, |p| p.attack_ms)
    }

    /// Whether a drop table's heart of `heal` percent still fits a paced fight's heal budget
    /// (always outside one, and for anything that heals nothing).
    pub(super) fn paced_heart_fits(&self, heal: u32) -> bool {
        let budget = self.pacing().and_then(|p| p.heals.as_ref()).map(|h| h.budget_pct);
        heal == 0
            || budget.is_none_or(|b| self.arena.pace.heal_given_pct.saturating_add(heal) <= b)
    }

    /// Count a placed drop-table heart against a paced fight's heal budget.
    pub(super) fn charge_paced_heart(&mut self, heal: u32) {
        if self.pacing().is_some_and(|p| p.heals.is_some()) {
            let spent = &mut self.arena.pace;
            spent.heal_given_pct = spent.heal_given_pct.saturating_add(heal);
        }
    }

    /// A kill in a paced fight may drop a heart: its chance, unless the cooldown runs or the
    /// run's heal budget is spent. A big heart only drops while the budget still holds it.
    pub(super) fn pace_heal_drop(
        &mut self,
        enemy: u64,
        at: [f32; 3],
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let Some(heals) = self.pacing().and_then(|p| p.heals.clone()) else {
            return;
        };
        let (Some(power_ups), Some(rng)) = (self.power_ups.clone(), self.power_up_rng.as_mut())
        else {
            return;
        };
        let spent = &self.arena.pace;
        if now_ms < spent.heal_ready_ms || spent.heal_given_pct >= heals.budget_pct {
            return;
        }
        if rng.next_f64() >= heals.chance {
            return;
        }
        let left = heals.budget_pct - spent.heal_given_pct;
        let pool = (power_ups.table(HEART_TABLE))
            .and_then(|t| t.health_roll.as_ref())
            .map(|roll| roll.pool.clone())
            .unwrap_or_default();
        let heal = |id: &String| power_ups.kind(id).and_then(|k| k.heal_pct_max).unwrap_or(0);
        let hearts: Vec<(&String, u32, u32)> = (pool.iter())
            .map(|(id, weight)| (id, *weight, heal(id)))
            .filter(|heart| heart.2 > 0)
            .collect();
        // The hearts the budget still holds; past them, only the smallest.
        let smallest = hearts.iter().map(|heart| heart.2).min();
        let affordable: Vec<(&String, u32)> = (hearts.iter())
            .filter(|heart| heart.2 <= left || Some(heart.2) == smallest)
            .map(|heart| (heart.0, heart.1))
            .collect();
        let Some(kind) = power_ups::pick_weighted(affordable.iter().copied(), rng).cloned() else {
            return;
        };
        if self.place_pickup(&power_ups, &kind, 0, (enemy, now_ms), at, now_ms, out) {
            let spent = &mut self.arena.pace;
            spent.heal_given_pct = spent.heal_given_pct.saturating_add(heal(&kind));
            spent.heal_ready_ms = now_ms.saturating_add(heals.cooldown_ms);
        }
    }

    /// When a paced row's next round comes, if one is waiting.
    pub(super) fn round_due(&self) -> Option<u64> {
        self.arena.pace.next_round_ms
    }

    /// A `field_battles.json` row's last kill: true when the pacing brings its roster back for
    /// another round (after the wave gap) instead of the win.
    pub(super) fn next_round(&mut self, now_ms: u64) -> bool {
        let rounds = self.pacing().map_or(1, |p| p.waves.max(1));
        let gap = self.pacing().map_or(0, |p| p.wave_gap_ms);
        let spent = &mut self.arena.pace;
        if spent.next_round_ms.is_some() {
            return true;
        }
        if spent.rounds + 1 >= rounds {
            return false;
        }
        spent.next_round_ms = Some(now_ms.saturating_add(gap));
        true
    }

    /// The next round of a paced `field_battles.json` row, once its gap has run: every enemy
    /// stands again at its home with full health, as the arena load spawned it.
    pub(super) fn tick_rounds(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        if self.arena.pace.next_round_ms.is_none_or(|due| now_ms < due) {
            return;
        }
        let Some(d) = self.dungeon().cloned() else {
            return;
        };
        let Some(active) = self.state.active.as_mut() else {
            return;
        };
        if active.phase != EncounterPhase::Fighting {
            return;
        }
        let pace = &mut self.arena.pace;
        pace.next_round_ms = None;
        pace.rounds += 1;
        let mut standing = Vec::new();
        for (a, s) in d.enemies.iter().zip(active.enemies.iter_mut()) {
            *s = enemy_state(a);
            if !s.dormant {
                standing.push((a.clone(), s.position, s.health));
            }
        }
        let mut guids = Vec::new();
        for (a, at, health) in standing {
            self.arena.brains.remove(&a.guid);
            self.arena.attacks.remove(&a.guid);
            self.arena.fallen.remove(&a.guid);
            out.replies.push(remove(a.guid));
            out.replies.push(self.spawn(&a, at, 0));
            out.replies
                .extend(enemy_health(a.guid, health, a.max_health));
            guids.push(a.guid);
        }
        out.replies.push(instance_relevance(&guids, false));
        out.replies.push(objective(2, d.objective_id, 0));
        out.changed = true;
    }
}
