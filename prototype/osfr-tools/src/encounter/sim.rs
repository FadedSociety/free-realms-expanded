//! The simulate-a-clear the balance tools and the pacing test share (rc-0136, enemy tuning): a
//! scripted solo player of a job and level plays one fight on a copy of the engine with the
//! job's real rotation, and reports how it ended, how long it took and the health left.
//! `realm sim` and `tests/instance_pacing.rs` both run it; nothing it does reaches a save.
use super::*;
use crate::opening_abilities::JobAbility;

/// The longest one simulated fight may run.
pub const LIMIT_MS: u64 = 600_000;
const STEP_MS: u64 = 100;

/// How a simulated fight ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimEnd {
    Won,
    Lost,
    Timeout,
}

/// One simulated fight.
#[derive(Clone, Debug, PartialEq)]
pub struct SimClear {
    pub end: SimEnd,
    pub seconds: f32,
    /// Health at the end, the lowest it fell to, and the player's maximum.
    pub health: u32,
    pub lowest: u32,
    pub max_health: u32,
    pub dealt: i64,
    pub crits: usize,
    /// Health power-ups the player stepped onto.
    pub hearts: u32,
    /// Times the player went down (a fight file lets it get up again; a battle row ends).
    pub knockouts: u32,
}

impl EncounterEngine {
    /// A scripted solo player of `profile` at job `level` fights encounter `id` on a copy of
    /// this engine. The rotation is the job's own, from its bar `slots`: a basic swing on the
    /// gate, its strike ability whenever its energy allows, First Aid (its heal) below 60 %
    /// health. It stands beside the nearest standing enemy (the unshielded first), steps onto
    /// a dropped heart below 85 % health, and walks out through a fight file's exit door once
    /// its goal opens it (the door counts a player who was clear of it while it stood open).
    /// Knocked out, it waits to get up as a player does. Power-up hearts help it as they help
    /// a player. The run's own phase says how it ended.
    pub fn simulate_clear(
        &self,
        id: u32,
        level: u32,
        profile: i32,
        slots: &[JobAbility],
        home: CharacterState,
    ) -> io::Result<SimClear> {
        let mut e = self.clone();
        // The drops roll on a generator of their own; seeded per fight and level, every run of
        // the simulator plays the same fight.
        if e.power_up_rng.is_some() {
            e.power_up_rng = Some(Rng::seeded(u64::from(id) << 8 | u64::from(level)));
        }
        e.set_player_job(profile, level);
        e.begin(id, home.clone(), profile)?;
        e.on_zone_ready(profile, 0)?;
        let heal = (slots.iter()).find(|a| a.heal > 0 && a.damage_multiplier == 0.0);
        let strike = slots.iter().find(|a| a.damage_multiplier > 0.0);
        let mut me = e.dungeon().map_or(home.position, |d| d.zone.position);
        let (mut t, mut lowest, mut hearts) = (0_u64, e.player_health().0, 0_u32);
        let (mut knockouts, mut down, mut clear_of_door) = (0_u32, false, false);
        let phase = |e: &Self| e.state.active.as_ref().map(|a| a.phase.clone());
        let mut end = SimEnd::Timeout;
        while t < LIMIT_MS {
            t += STEP_MS;
            match phase(&e) {
                Some(EncounterPhase::Won) => end = SimEnd::Won,
                Some(EncounterPhase::Lost) | None => end = SimEnd::Lost,
                _ => {}
            }
            if end != SimEnd::Timeout {
                break;
            }
            let (hp, max) = e.player_health();
            let target =
                (e.standing_enemies().into_iter()).min_by_key(|(_, _, shielded)| *shielded);
            let pickups = e.health_pickups();
            let heart = (hp * 20 < max * 17)
                .then(|| {
                    pickups
                        .iter()
                        .copied()
                        .min_by(|a, b| distance(me, *a).total_cmp(&distance(me, *b)))
                })
                .flatten();
            me = match (heart, target, e.open_exit()) {
                (Some(at), ..) => at,
                (None, Some((_, at, _)), _) => [at[0] + 1.5, at[1], at[2]],
                (None, None, Some(door)) if clear_of_door => door,
                (None, None, Some(_)) => {
                    clear_of_door = true;
                    me
                }
                (None, None, None) => me,
            };
            e.tick(me, profile, t)?;
            hearts += pickups.len().saturating_sub(e.health_pickups().len()) as u32;
            let (hp, max) = e.player_health();
            lowest = lowest.min(hp);
            knockouts += u32::from(hp == 0 && !down);
            down = hp == 0;
            if down {
                continue;
            }
            if let Some(heal) = heal.filter(|_| hp * 10 < max * 6) {
                e.job_ability(heal, 1, me, profile, t)?;
            }
            if let Some((target, ..)) = target {
                if let Some(strike) = strike {
                    e.job_ability(strike, target, me, profile, t)?;
                }
                e.attack(target, me, profile, 0, t)?;
            }
        }
        if end == SimEnd::Timeout && phase(&e) == Some(EncounterPhase::Won) {
            end = SimEnd::Won;
        }
        let log = e.fight_log();
        let dealt = (log.iter())
            .filter(|ev| matches!(ev.kind, "hit" | "crit"))
            .map(|ev| ev.amount)
            .sum();
        let (health, max_health) = e.player_health();
        Ok(SimClear {
            end,
            seconds: t as f32 / 1000.0,
            health,
            lowest,
            max_health,
            dealt,
            crits: log.iter().filter(|ev| ev.kind == "crit").count(),
            hearts,
            knockouts,
        })
    }

    /// The open exit door of a fight file's run, where a player walks out.
    pub fn open_exit(&self) -> Option<[f32; 3]> {
        let run = (self.run.run.as_ref()).filter(|run| !run.legacy && run.exit_is_open())?;
        Some(run.def.exit_spot())
    }
}
