//! The developer balance tools `realm sim` and `realm fightlog`, moved out of `gameplay.rs`
//! by the combat seam. Piece P4 (`combat-scale`) owns this file and extends the simulator.
use super::*;
use crate::encounter::sim::{SimClear, SimEnd};

impl Gameplay {
    /// `realm sim [battle] [level]`: one line per simulated job level (battle 96 when none is
    /// named; levels 1, 5, 10, 15 and 20 when no level is).
    /// `realm sim rank <rank> [tier]` and `realm sim kind <enemies.json id> [tier]` print the
    /// kill table instead: hits and seconds per job level against one enemy of that rank or
    /// kind, at the given tier or at the player's own.
    pub(super) fn developer_sim(
        &self,
        rest: &[&str],
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let say = |out: &mut Outcome, text: String| out.replies.push(chat::system(&text));
        if let [what @ ("rank" | "kind"), name, tier @ ..] = rest {
            let tier = tier.first().and_then(|t| t.parse::<u8>().ok());
            for line in self.kill_table_lines(what, name, tier)? {
                say(out, line);
            }
            return Ok(());
        }
        let battle = rest
            .first()
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(96);
        let level = rest.get(1).and_then(|n| n.parse::<u32>().ok());
        for line in self.simulate_battle(battle, level, character)? {
            say(out, line);
        }
        Ok(())
    }

    /// `realm fightlog`: totals of the current (or last) fight's log.
    pub(super) fn developer_fightlog(&self, out: &mut Outcome) {
        let say = |out: &mut Outcome, text: String| out.replies.push(chat::system(&text));
        let log = self
            .encounter
            .as_ref()
            .map(|e| e.fight_log().to_vec())
            .unwrap_or_default();
        let dealt: i64 = log
            .iter()
            .filter(|e| e.source == "you" && matches!(e.kind, "hit" | "crit"))
            .map(|e| e.amount)
            .sum();
        let taken: i64 = log
            .iter()
            .filter(|e| e.kind == "taken")
            .map(|e| e.amount)
            .sum();
        let healed: i64 = log
            .iter()
            .filter(|e| e.kind == "heal")
            .map(|e| e.amount)
            .sum();
        let crits = log.iter().filter(|e| e.kind == "crit").count();
        say(
            out,
            format!(
                "Fight log: {} events; dealt {dealt} ({crits} crits), taken {taken}, healed {healed}.",
                log.len()
            ),
        );
    }

    /// The kill table of the active job at levels 1, 5, 10, 15 and the last, from the scale
    /// tables alone: basics at the gate's rhythm against one enemy, and that enemy's hits
    /// against a player with no Defense.
    fn kill_table_lines(&self, what: &str, name: &str, tier: Option<u8>) -> io::Result<Vec<String>> {
        let profile = self.profiles()?.active();
        let Some(engine) = self.encounter.as_ref() else {
            return Ok(vec!["No encounter engine.".into()]);
        };
        let (rank, own, title) = if what == "kind" {
            let Some((kind, _)) = engine.catalogue_kind(name, tier, 1) else {
                return Ok(vec![format!("No ranked enemy kind '{name}' in enemies.json.")]);
            };
            let rank = kind.rank.unwrap_or_default();
            ((rank, kind.tier), (kind.health, kind.hit), kind.kind)
        } else {
            let Ok(rank) = serde_json::from_value(serde_json::Value::String(name.to_owned()))
            else {
                return Ok(vec![format!("No rank '{name}' (minion, normal, tough, elite, boss ...).")]);
            };
            ((rank, None), (None, None), name.to_owned())
        };
        let last = engine.scale_span().0;
        let levels: Vec<u8> = (1..=last).filter(|l| *l == 1 || l % 5 == 0).collect();
        let tier = tier.or(rank.1.filter(|_| what == "kind"));
        let [low, high] = engine.solo_seconds(rank.0);
        let mut lines = vec![format!(
            "{title} ({:?}), job {profile}; this rank should take one player {low:.1}-{high:.1} s:",
            rank.0
        )];
        for line in engine.kill_table(profile, rank.0, tier, own, &levels) {
            lines.push(format!(
                "  level {:>2}: tier {} enemy {} health, {} a hit: {} hits, {:.1} s; it hits {} and needs {} hits, {:.0} s",
                line.level,
                line.tier,
                line.enemy_health,
                line.strike,
                line.hits,
                line.seconds,
                line.enemy_hit,
                line.hits_to_fall,
                line.seconds_to_fall
            ));
        }
        Ok(lines)
    }

    /// Balance simulator: a scripted player of the active job fights `battle` at several job
    /// levels (or at `only`) through [`EncounterEngine::simulate_clear`], on a copy of the
    /// engine (nothing is changed).
    fn simulate_battle(
        &self,
        battle: u32,
        only: Option<u32>,
        character: &CharacterState,
    ) -> io::Result<Vec<String>> {
        // A realm without the opening content has neither: say so, not nothing.
        let (Some(_), Some(profiles)) = (&self.encounter, &self.profiles) else {
            return Ok(vec!["No encounter engine.".into()]);
        };
        let profile = profiles.active();
        let mut lines = vec![format!("Battle {battle}, job {profile} (scripted bot):")];
        for level in only.map_or_else(|| vec![1_u32, 5, 10, 15, 20], |level| vec![level]) {
            let line = match self.simulate_fight(battle, profile, level, character)? {
                Err(why) if why.starts_with("needs level") => {
                    format!("  level {level:>2}: {why}; skipped")
                }
                Err(why) => return Ok(vec![why]),
                Ok(sim) => format!(
                    "  level {level:>2}: {} in {:.0}s, health left {}/{}, lowest {}, dealt {} ({} crits), {} hearts, {} knockouts",
                    match sim.end {
                        SimEnd::Won => "won",
                        SimEnd::Lost => "lost",
                        SimEnd::Timeout => "timeout",
                    },
                    sim.seconds,
                    sim.health,
                    sim.max_health,
                    sim.lowest,
                    sim.dealt,
                    sim.crits,
                    sim.hearts,
                    sim.knockouts
                ),
            };
            lines.push(line);
        }
        Ok(lines)
    }

    /// One simulated solo clear of `battle` by job `profile` at `level`, with its real
    /// rotation (its bar's strike ability and First Aid). `Err` says why it cannot run: no
    /// engine, a fight in progress, a prize that cannot be granted, or the level gate.
    pub fn simulate_fight(
        &self,
        battle: u32,
        profile: i32,
        level: u32,
        character: &CharacterState,
    ) -> io::Result<Result<SimClear, String>> {
        let Some(engine) = self.encounter.as_ref() else {
            return Ok(Err("No encounter engine.".into()));
        };
        if engine.active() {
            return Ok(Err("Finish or exit the current encounter first.".into()));
        }
        if !self.battle_prizes_available(battle) {
            return Ok(Err(field_battle::UNAVAILABLE_BATTLE_PRIZE.into()));
        }
        if let Some(required) = engine.battle_level_refusal(battle, level) {
            return Ok(Err(format!("needs level {required}")));
        }
        let slots: Vec<_> = (1..=7)
            .filter_map(|slot| {
                (self.abilities.as_ref())
                    .and_then(|a| a.job_slot(profile, slot, level))
                    .cloned()
            })
            .collect();
        match engine.simulate_clear(battle, level, profile, &slots, character.clone()) {
            Ok(sim) => Ok(Ok(sim)),
            Err(error) => Ok(Err(format!("Cannot simulate battle {battle}: {error}"))),
        }
    }
}
