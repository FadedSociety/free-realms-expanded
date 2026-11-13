//! Combat rules that are timing and chance, not scale: the attack gate, energy, critical hits,
//! job identity, the knockout timers and the death delay. They come from `combat_rules.json`
//! in the content directory. The shipped file is compiled in and is the only copy of the
//! defaults: a directory without the file gets it whole, and a file that names only some keys
//! gets the shipped value for the rest. A file that is present but bad refuses the load.
//! Piece P4 (`combat-scale`) owns this file and the data file.
use super::*;

const FILE: &str = "combat_rules.json";
/// The shipped file, the one copy of the default numbers.
const SHIPPED: &str = include_str!("../../data/content/combat_rules.json");

/// Energy for job abilities (R): the HUD shows 100/100; regeneration of 2 per 0.5 s follows
/// the community reference's 4 per second.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Energy {
    pub max: i32,
    pub step: i32,
    pub step_ms: u64,
}

/// Critical hits deal x1.5 (video: an 83 hit critted for 124; client "50 Crit Mult %").
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Crit {
    pub numerator: u32,
    pub denominator: u32,
}

/// Combat job identity (R; the owner's design, 2026-10-15): Brawler is the berserker tank
/// (video level 20: Brawler 9821 vs Archer 8540 health), Ninja hits quick and crits often,
/// Medic and Wizard have small health pools (Medic heals, Wizard hits hard from range).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Job {
    /// Multiplier on the level's base health.
    pub health: f32,
    pub crit_percent: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Revive {
    /// (R) Overworld knockout window: the ten-second recovery of the wiki Combat page
    /// (research piece 058). The client counts it down only when it does not show the
    /// two-button dialog, and then sends the free revive itself.
    pub timer_ms: i32,
    /// (R) Field enemies ignore a freshly revived or recovered player this long.
    pub field_grace_ms: u64,
    /// Inside a fight, enemies cannot hurt a player who stood up from a knockout this long
    /// (owner 2026-11-13: "3-4 seconds of invincibility after a combat knockout"). A `revive`
    /// block without it (an older content file) has none.
    #[serde(default)]
    pub fight_grace_ms: u64,
    /// (R) Server fallback: an overworld knockout with no 41/122 by then is revived at the
    /// safe location anyway, so a client that never shows the window (its 41/125 field order
    /// is inferred, not captured) cannot leave a player down.
    pub auto_ms: u64,
}

/// Delay of the graceful remove (35/3) that follows a defeat.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DeathDelay {
    pub field: u32,
    pub battle: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CombatRules {
    pub schema_version: u32,
    /// Least time between two accepted basic attacks, in ms.
    pub basic_gate_ms: u64,
    pub energy: Energy,
    pub crit: Crit,
    /// By profile id: 2 Ninja, 11 Medic, 12 Wizard, 32 Warrior, 35 Archer, 43 Brawler. A
    /// profile without a row has health x1 and never crits.
    pub jobs: BTreeMap<i32, Job>,
    pub revive: Revive,
    pub death_delay_ms: DeathDelay,
    /// P1's switch: `true` sends a player hit in the live shape (36/3, 36/4, 35/35, 36/14),
    /// `false` the 35/8, 32/7, 35/5 the server has always sent.
    pub player_hit_wire: bool,
    /// Evidence rows for the content checker.
    pub source: Vec<serde_json::Value>,
}

impl CombatRules {
    /// The shipped rows with the top-level keys of `file` laid over them.
    fn parse(file: Option<&[u8]>) -> Result<Self, serde_json::Error> {
        let mut rows: serde_json::Map<String, serde_json::Value> = serde_json::from_str(SHIPPED)?;
        if let Some(file) = file {
            rows.extend(serde_json::from_slice::<serde_json::Map<_, _>>(file)?);
        }
        serde_json::from_value(rows.into())
    }

    /// The content directory's file over the shipped one, or the shipped one alone.
    pub(super) fn load(directory: &Path) -> io::Result<Arc<Self>> {
        let path = directory.join(FILE);
        let refuse = |e: &dyn std::fmt::Display| invalid(&format!("{}: {e}", path.display()));
        let file = if path.is_file() {
            if fs::metadata(&path).map_err(|e| refuse(&e))?.len() > 16 * 1024 {
                return Err(refuse(&"file too large"));
            }
            Some(fs::read(&path).map_err(|e| refuse(&e))?)
        } else {
            None
        };
        let rules = Self::parse(file.as_deref()).map_err(|e| refuse(&e))?;
        let minute = 60_000;
        if rules.schema_version != 1
            || rules.basic_gate_ms > minute
            || !(1..=10_000).contains(&rules.energy.max)
            || !(0..=rules.energy.max).contains(&rules.energy.step)
            || !(50..=minute).contains(&rules.energy.step_ms)
            || rules.crit.denominator == 0
            || rules.crit.numerator < rules.crit.denominator
            || rules.crit.numerator > rules.crit.denominator.saturating_mul(100)
            || rules
                .jobs
                .values()
                .any(|job| !(0.1..=10.0).contains(&job.health) || job.crit_percent > 100)
            || !(0..=600_000).contains(&rules.revive.timer_ms)
            || rules.revive.field_grace_ms > 10 * minute
            || rules.revive.fight_grace_ms > minute
            || !(1_000..=10 * minute).contains(&rules.revive.auto_ms)
            || u64::from(rules.death_delay_ms.field.max(rules.death_delay_ms.battle)) > minute
        {
            return Err(refuse(&"value outside its sanity bounds"));
        }
        Ok(Arc::new(rules))
    }

    /// (health multiplier, crit chance %) of a profile.
    pub(super) fn job_identity(&self, profile: i32) -> (f32, u32) {
        self.jobs
            .get(&profile)
            .map_or((1.0, 0), |job| (job.health, job.crit_percent))
    }

    /// What a critical hit multiplies damage by, before skill bonuses.
    pub(super) fn crit_multiplier(&self) -> f64 {
        f64::from(self.crit.numerator) / f64::from(self.crit.denominator)
    }

    pub(super) fn death_delay(&self, field: bool) -> u32 {
        if field {
            self.death_delay_ms.field
        } else {
            self.death_delay_ms.battle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(name: &str, file: Option<&str>) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("combat-rules-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        if let Some(text) = file {
            fs::write(dir.join(FILE), text).unwrap();
        }
        dir
    }

    /// The shipped file is the defaults. These are the 2014 values the captures settle
    /// (gate 650 ms, energy +4 a second, one health curve, crit 3 %); the knockout timers and
    /// death delays are still the engine's constants from before the combat seam.
    #[test]
    fn the_shipped_file_equals_the_built_in_defaults() {
        let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut rules = CombatRules::load(&shipped).unwrap().as_ref().clone();
        assert_eq!(rules, CombatRules::parse(None).unwrap());
        assert!(!rules.source.is_empty(), "every value names its source");
        rules.source.clear();
        let job = |health, crit_percent| Job {
            health,
            crit_percent,
        };
        let before_the_seam = CombatRules {
            schema_version: 1,
            basic_gate_ms: 650,
            energy: Energy {
                max: 100,
                step: 4,
                step_ms: 1_000,
            },
            crit: Crit {
                numerator: 3,
                denominator: 2,
            },
            jobs: BTreeMap::from([
                (43, job(1.0, 3)),
                (32, job(1.0, 3)),
                (35, job(1.0, 3)),
                (2, job(1.0, 3)),
                (11, job(1.0, 3)),
                (12, job(1.0, 3)),
            ]),
            revive: Revive {
                timer_ms: 10_000,
                field_grace_ms: 8_000,
                fight_grace_ms: 3_500,
                auto_ms: 15_000,
            },
            death_delay_ms: DeathDelay {
                field: 0,
                battle: 0,
            },
            player_hit_wire: false,
            source: Vec::new(),
        };
        assert_eq!(rules, before_the_seam);
    }

    #[test]
    fn a_missing_file_is_the_defaults_and_a_bad_file_refuses_the_load() {
        let none = directory("none", None);
        assert_eq!(
            *CombatRules::load(&none).unwrap(),
            CombatRules::parse(None).unwrap()
        );
        let changed = directory("changed", Some(r#"{"basic_gate_ms": 700}"#));
        let rules = CombatRules::load(&changed).unwrap();
        assert_eq!((rules.basic_gate_ms, rules.energy.max), (700, 100));
        for (name, bad) in [
            ("json", "not json"),
            ("key", r#"{"basic_gate": 650}"#),
            ("version", r#"{"schema_version": 2}"#),
            ("crit", r#"{"crit": {"numerator": 1, "denominator": 0}}"#),
            (
                "energy",
                r#"{"energy": {"max": 0, "step": 2, "step_ms": 500}}"#,
            ),
        ] {
            let dir = directory(name, Some(bad));
            assert!(CombatRules::load(&dir).is_err(), "{name}");
            let _ = fs::remove_dir_all(dir);
        }
        for dir in [none, changed] {
            let _ = fs::remove_dir_all(dir);
        }
    }
}
