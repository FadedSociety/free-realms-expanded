//! The developer balance tools `realm sim` and `realm fightlog`, moved out of `gameplay.rs`
//! by the combat seam. Piece P4 (`combat-scale`) owns this file and extends the simulator.
use super::*;

impl Gameplay {
    /// `realm sim [battle]`: one line per simulated job level (battle 96 when none is named).
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
        for line in self.simulate_battle(battle, character)? {
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

    /// Balance simulator: a scripted player of the active job fights `battle`
    /// at several job levels on a copy of the engine (nothing is changed).
    /// The bot stands next to its target, swings on cooldown, uses First Aid
    /// below 60% health and Triage-like strikes when energy allows.
    fn simulate_battle(&self, battle: u32, character: &CharacterState) -> io::Result<Vec<String>> {
        let profile = self.profiles()?.active();
        let Some(base) = self.encounter.as_ref() else {
            return Ok(vec!["No encounter engine.".into()]);
        };
        if base.active() {
            return Ok(vec!["Finish or exit the current encounter first.".into()]);
        }
        if !self.battle_prizes_available(battle) {
            return Ok(vec![field_battle::UNAVAILABLE_BATTLE_PRIZE.into()]);
        }
        let mut lines = vec![format!("Battle {battle}, job {profile} (scripted bot):")];
        for level in [1_u32, 5, 10, 15, 20] {
            let mut engine = base.clone();
            // The scripted bot must not be helped by power-up hearts.
            engine.unload_power_ups();
            engine.set_player_job(profile, level);
            // Eligibility is per level; the eligible levels still run through
            // the same production `begin` guard.
            if let Some(required) = engine.battle_level_refusal(battle, level) {
                lines.push(format!(
                    "  level {level:>2}: needs level {required}; skipped"
                ));
                continue;
            }
            if let Err(error) = engine.begin(battle, character.clone(), profile) {
                return Ok(vec![format!("Cannot simulate battle {battle}: {error}")]);
            }
            engine.on_zone_ready(profile, 0)?;
            let abilities = self.abilities.as_ref();
            let slots: Vec<_> = (1..=7)
                .filter_map(|slot| {
                    abilities
                        .and_then(|a| a.job_slot(profile, slot, level))
                        .cloned()
                })
                .collect();
            let mut result = "timeout";
            let mut t = 0_u64;
            let mut lowest = engine.player_health().0;
            while t < 600_000 {
                t += 100;
                let standing = engine.standing_enemies();
                let Some(&(target, at, _)) =
                    standing.iter().min_by_key(|(_, _, shielded)| *shielded)
                else {
                    break;
                };
                let me = [at[0] + 1.5, at[1], at[2]];
                engine.tick(me, profile, t)?;
                let (hp, max) = engine.player_health();
                lowest = lowest.min(hp);
                if hp == 0 {
                    result = "lost";
                    break;
                }
                let heal = slots
                    .iter()
                    .find(|a| a.heal > 0 && a.damage_multiplier == 0.0);
                let strike = slots.iter().find(|a| a.damage_multiplier > 0.0);
                if hp * 10 < max * 6 {
                    if let Some(heal) = heal {
                        engine.job_ability(heal, 1, me, profile, t)?;
                    }
                }
                if let Some(strike) = strike {
                    engine.job_ability(strike, target, me, profile, t)?;
                }
                engine.attack(target, me, profile, 0, t)?;
                if engine
                    .state()
                    .active
                    .as_ref()
                    .is_some_and(|a| a.phase == crate::encounter::EncounterPhase::Won)
                {
                    result = "won";
                    break;
                }
            }
            let log = engine.fight_log();
            let dealt: i64 = log
                .iter()
                .filter(|e| matches!(e.kind, "hit" | "crit"))
                .map(|e| e.amount)
                .sum();
            let crits = log.iter().filter(|e| e.kind == "crit").count();
            let max = engine.player_health().1;
            lines.push(format!(
                "  level {level:>2}: {result} in {:.0}s, lowest health {lowest}/{max}, dealt {dealt} ({crits} crits)",
                t as f64 / 1000.0
            ));
        }
        Ok(lines)
    }
}
