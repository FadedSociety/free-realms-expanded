//! One battle run's enemy simulation: attack clocks, walk set and skittish kiting, the solo
//! and the shared enemy tick, and the summon ladder. Split from `arena.rs` by the combat seam
//! so the brain (P3) and the run lifecycle (P5, `arena.rs`) never share a file.
use super::*;

/// Summon presentation (R): the arm-swirl cast and the client's troll spawn
/// puff. troll_shaman.adr has com_cast_01/02 and com_spawn but no
/// com_cast_special_* (the first native run looped on the missing 1131), so
/// the summon uses com_cast_02, new minions play com_spawn, and the puff is
/// PFX_bones_yellow_troll-spawn (ActorCompositeEffectDefinitions.xml 16337).
/// The live game's choices are not recovered.
const SUMMON_ANIMATION: i32 = 1112; // com_cast_02
const SPAWN_ANIMATION: i32 = 1404; // com_spawn
const SUMMON_EFFECT: i32 = 16337;

/// (R) The one arena-simulation value. The engine owns exactly one `ArenaSim`;
/// a group's shared run (`SharedArena`) copies its own into the acting member's
/// engine for one action and copies it back (`enter_arena`, `leave_arena`).
/// Never add a second engine field that would have to be copied separately.
///
/// Runtime clocks only: none of it is saved (`ActiveEncounter` still holds the
/// enemies and phase), so every load, begin and `reset_session` starts it
/// empty. Keys are arena enemy GUIDs, and the opening stage's `world_enemies`
/// share the same tick loop and so the same maps. Field enemy GUIDs never
/// appear here: the encounter, field-enemy and field-battle loaders refuse an
/// actor GUID that another set already uses (`arena_and_field_guids_are_disjoint`).
#[derive(Clone, Default)]
pub(super) struct ArenaSim {
    /// Enemy GUID to its last attack time.
    pub(super) attacks: BTreeMap<u64, u64>,
    /// Enemy GUIDs whose walk animation is active.
    pub(super) moving: BTreeSet<u64>,
    /// Skittish actors fighting alone: running or resting, until a time.
    pub(super) kiting: BTreeMap<u64, Kite>,
    /// Shared runs only: enemy GUID to the member who last damaged it (its target).
    pub(super) last_hit: BTreeMap<u64, u64>,
    /// The member whose action holds a shared run's sim; `None` for a solo run (and so for
    /// `ArenaSim::default()` after every `reset_session`, which also detaches a member).
    pub(super) acting: Option<u64>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Kite {
    Run(u64),
    Rest(u64),
}

/// What one arena tick reads from the engine; the sim itself is `&mut`.
pub(super) struct ArenaTick<'a> {
    pub(super) position: [f32; 3],
    pub(super) ground: &'a Ground,
    pub(super) behaviours: &'a BTreeMap<u64, Behaviour>,
    pub(super) skills: &'a SkillEffects,
    pub(super) mitigation: f32,
    pub(super) player_health: u32,
    pub(super) max_health: u32,
    pub(super) now_ms: u64,
    pub(super) elapsed: u64,
    /// Summoned minions still stand (the summoner fights beside them instead of kiting);
    /// read off the whole roster, so a driver's per-member subset cannot change it.
    pub(super) backup: bool,
}

/// Whether a summoner has minions standing: `actors` is the whole roster.
pub(super) fn summoner_backup(actors: &[Actor], states: &[EnemyState]) -> bool {
    actors.iter().any(|a| {
        a.summon_below_percent.is_some()
            && states
                .iter()
                .any(|s| s.guid == a.guid && s.health > 0 && !s.dormant)
    })
}

impl ArenaSim {
    /// A defeated enemy leaves the walk-away and target books.
    pub(super) fn forget(&mut self, guid: u64) {
        self.kiting.remove(&guid);
        self.last_hit.remove(&guid);
    }

    /// Move, chase and swing every standing enemy for one tick; returns the
    /// damage the player takes (the caller applies it).
    pub(super) fn tick_enemies(
        &mut self,
        actors: Vec<Actor>,
        states: &mut [EnemyState],
        t: &ArenaTick,
        out: &mut EncounterOutcome,
    ) -> u32 {
        let (position, now_ms, elapsed) = (t.position, t.now_ms, t.elapsed);
        let mut total_damage = 0_u32;
        for actor in actors {
            let Some(s) = states.iter_mut().find(|s| s.guid == actor.guid) else {
                continue;
            };
            if s.health == 0 || s.dormant {
                continue;
            }
            if t.skills.stunned(actor.guid, now_ms) {
                continue;
            }
            let b = t.behaviours.get(&actor.guid).cloned().unwrap_or_default();
            // L092: a scared enemy runs straight away from its scarer and strikes nobody.
            if let Some(from) = t.skills.fleeing_from(actor.guid, now_ms) {
                let step = b.move_speed * elapsed.min(250) as f32 / 1000.0;
                let unit = super::battle_items::away_from(s.position, from, 1.0);
                let away = [unit[0] - s.position[0], 0.0, unit[2] - s.position[2]];
                let mut next = super::battle_items::away_from(s.position, from, step);
                t.ground.snap(&mut next);
                s.position = next;
                out.replies.push(movement(actor.guid, next, away));
                if self.moving.insert(actor.guid) {
                    out.replies.push(animation(actor.guid, 3, true));
                }
                out.changed = true;
                continue;
            }
            if b.mode == Mode::Skittish {
                if t.backup {
                    self.kiting.remove(&actor.guid);
                } else {
                    self.kite_turn(&actor, s, t, &b, out);
                    continue;
                }
            }
            let delta = distance(position, s.position);
            if delta > b.chase_limit {
                if self.moving.remove(&actor.guid) {
                    out.replies.push(animation(actor.guid, 1, true));
                }
                continue;
            }
            if delta > b.reach && b.mode != Mode::Stationary && !t.skills.rooted(actor.guid, now_ms)
            {
                let heading = [
                    (position[0] - s.position[0]) / delta,
                    0.0,
                    (position[2] - s.position[2]) / delta,
                ];
                // Stop inside reach so f32 rounding at world coordinates cannot
                // strand a pursuer just outside the attack threshold.
                let step =
                    (b.move_speed * elapsed.min(250) as f32 / 1000.0).min(delta - (b.reach - 0.1));
                for (coord, to) in s.position.iter_mut().zip(position) {
                    *coord += (to - *coord) / delta * step;
                }
                t.ground.snap(&mut s.position);
                out.replies.push(movement(actor.guid, s.position, heading));
                if self.moving.insert(actor.guid) {
                    out.replies.push(animation(actor.guid, 3, true));
                }
                out.changed = true;
            }
            if distance(position, s.position) <= b.reach {
                if self.moving.remove(&actor.guid) {
                    out.replies.push(animation(actor.guid, 1, true));
                }
                if self
                    .attacks
                    .get(&actor.guid)
                    .is_some_and(|last| now_ms.saturating_sub(*last) < b.attack_ms)
                {
                    continue;
                }
                self.attacks.insert(actor.guid, now_ms);
                out.replies.push(animation(actor.guid, 1001, false));
                let attack_damage = (actor.damage as f32 * t.mitigation).round() as u32;
                let damage = attack_damage.min(t.player_health.saturating_sub(total_damage));
                out.replies.push(attack_processed(
                    actor.guid,
                    1,
                    damage,
                    t.player_health
                        .saturating_sub(total_damage)
                        .saturating_sub(damage),
                    t.max_health,
                    false,
                ));
                total_damage = total_damage.saturating_add(attack_damage);
            }
        }
        total_damage
    }

    /// One tick of a skittish actor fighting alone: run from the player, then
    /// rest (no attacks), then run again.
    fn kite_turn(
        &mut self,
        actor: &Actor,
        s: &mut EnemyState,
        t: &ArenaTick,
        b: &Behaviour,
        out: &mut EncounterOutcome,
    ) {
        let (player, now_ms, elapsed) = (t.position, t.now_ms, t.elapsed);
        let delta = distance(player, s.position).max(0.01);
        let away = [
            (s.position[0] - player[0]) / delta,
            0.0,
            (s.position[2] - player[2]) / delta,
        ];
        let phase = *self
            .kiting
            .entry(actor.guid)
            .or_insert(Kite::Run(now_ms.saturating_add(b.run_ms)));
        match phase {
            Kite::Rest(until) if now_ms < until => {}
            Kite::Rest(_) => {
                self.kiting
                    .insert(actor.guid, Kite::Run(now_ms.saturating_add(b.run_ms)));
            }
            Kite::Run(until) if now_ms >= until => {
                self.kiting
                    .insert(actor.guid, Kite::Rest(now_ms.saturating_add(b.rest_ms)));
                self.moving.remove(&actor.guid);
                // Turn to face the player and catch its breath.
                out.replies
                    .push(movement(actor.guid, s.position, [-away[0], 0.0, -away[2]]));
                out.replies.push(animation(actor.guid, 1, true));
                out.replies
                    .push(animation(actor.guid, b.rest_animation, false));
                out.changed = true;
            }
            Kite::Run(until) if delta > b.run_from => {
                self.kiting
                    .insert(actor.guid, Kite::Run(until.saturating_add(elapsed)));
                if self.moving.remove(&actor.guid) {
                    out.replies
                        .push(movement(actor.guid, s.position, [-away[0], 0.0, -away[2]]));
                    out.replies.push(animation(actor.guid, 1, true));
                    out.changed = true;
                }
            }
            Kite::Run(_) => {
                let step = b.run_speed * elapsed.min(250) as f32 / 1000.0;
                // Straight away, or along the leash edge (either tangent) when cornered.
                let directions = [away, [away[2], 0.0, -away[0]], [-away[2], 0.0, away[0]]];
                let next = directions.iter().find_map(|d| {
                    let next = t.ground.snapped([
                        s.position[0] + d[0] * step,
                        player[1],
                        s.position[2] + d[2] * step,
                    ]);
                    let flat = [next[0], actor.position[1], next[2]];
                    (distance(flat, actor.position) <= b.leash).then_some((next, *d))
                });
                if let Some((next, heading)) = next {
                    s.position = next;
                    out.replies.push(movement(actor.guid, s.position, heading));
                    if self.moving.insert(actor.guid) {
                        out.replies.push(animation(actor.guid, 3, true));
                    }
                    out.changed = true;
                }
            }
        }
    }
}

/// An enemy's target (R): its last attacker while that member is standing in the fight, else
/// the nearest standing member. `players` holds only members who can be hit.
pub(super) fn target_of(
    last_hit: &BTreeMap<u64, u64>,
    guid: u64,
    at: [f32; 3],
    players: &[FieldParticipant],
) -> Option<u64> {
    last_hit
        .get(&guid)
        .filter(|hitter| players.iter().any(|p| p.guid == **hitter))
        .copied()
        .or_else(|| {
            players
                .iter()
                .min_by(|a, b| distance(a.position, at).total_cmp(&distance(b.position, at)))
                .map(|p| p.guid)
        })
}

impl EncounterEngine {
    /// One tick of a shared run's enemies, on the arena's enemies (call it between `enter_arena`
    /// and `leave_arena`). Each enemy moves and swings once, at its own target; returns, per
    /// member hit, the damage it takes and the packets that tick made (a member's own damage
    /// popups are among them).
    pub fn tick_arena_enemies(
        &mut self,
        players: &[FieldParticipant],
        now_ms: u64,
        elapsed: u64,
    ) -> Vec<(u64, u32, Vec<Vec<u8>>)> {
        let Some(actors) = self.dungeon().map(|d| d.enemies.clone()) else {
            return Vec::new();
        };
        let ground = self.ground();
        let Some(active) = &mut self.state.active else {
            return Vec::new();
        };
        let states = &mut active.enemies;
        let backup = summoner_backup(&actors, states);
        let targets: BTreeMap<u64, u64> = actors
            .iter()
            .filter_map(|a| {
                let s = states.iter().find(|s| s.guid == a.guid)?;
                if s.health == 0 || s.dormant {
                    return None;
                }
                let target = target_of(&self.arena.last_hit, a.guid, s.position, players)?;
                Some((a.guid, target))
            })
            .collect();
        let mut hits = Vec::new();
        for p in players {
            let mine: Vec<Actor> = actors
                .iter()
                .filter(|a| targets.get(&a.guid) == Some(&p.guid))
                .cloned()
                .collect();
            if mine.is_empty() {
                continue;
            }
            let tick = ArenaTick {
                position: p.position,
                ground: &ground,
                behaviours: &self.behaviour_of,
                skills: &self.skills,
                mitigation: p.mitigation,
                player_health: p.health,
                max_health: p.max_health,
                now_ms,
                elapsed,
                backup,
            };
            let mut out = EncounterOutcome::default();
            let damage = self.arena.tick_enemies(mine, states, &tick, &mut out);
            hits.push((p.guid, damage, out.replies));
        }
        hits
    }

    /// One tick of this player's own arena (or opening-stage) enemies; returns the damage the
    /// player takes. A shared run's enemies tick once per instance in the driver
    /// (`tick_arena_enemies`) instead.
    pub(super) fn tick_solo_arena(
        &mut self,
        position: [f32; 3],
        now_ms: u64,
        elapsed: u64,
        out: &mut EncounterOutcome,
    ) -> u32 {
        let mitigation = self.skills.mitigation();
        let ground = self.ground();
        let actors: Vec<Actor> = if let Some(d) = self.dungeon() {
            d.enemies.clone()
        } else {
            self.content
                .world_enemies
                .iter()
                .filter(|a| a.stages.contains(&self.state.world_stage))
                .cloned()
                .collect()
        };
        let states = if let Some(a) = &mut self.state.active {
            &mut a.enemies
        } else {
            &mut self.state.world_enemies
        };
        let tick = ArenaTick {
            position,
            ground: &ground,
            behaviours: &self.behaviour_of,
            skills: &self.skills,
            mitigation,
            player_health: self.state.player_health,
            max_health: self.max_health,
            now_ms,
            elapsed,
            backup: summoner_backup(&actors, states),
        };
        // A shared run's enemies tick once per instance in the driver (`tick_arena_enemies`).
        if self.arena.acting.is_none() {
            self.arena.tick_enemies(actors, states, &tick, out)
        } else {
            0
        }
    }

    /// The summoner was hit down to `hp`: every dormant minion whose
    /// threshold is now crossed appears around it in a puff, while the
    /// summoner plays its cast.
    pub(super) fn summon(
        &mut self,
        summoner: u64,
        hp: u32,
        max_health: u32,
        out: &mut EncounterOutcome,
    ) {
        let Some(d) = self.dungeon() else {
            return;
        };
        let Some(home) = d
            .enemies
            .iter()
            .find(|a| a.guid == summoner && a.summon_below_percent.is_none())
            .map(|a| a.position)
        else {
            return;
        };
        if !d.enemies.iter().any(|a| a.summon_below_percent.is_some()) {
            return;
        }
        let percent = (u64::from(hp) * 100 / u64::from(max_health.max(1))) as u32;
        let content = self.content.clone();
        let found = content.dungeons.iter().find(|x| x.id == d.id);
        let (Some(d), Some(active)) = (found, self.state.active.as_mut()) else {
            return;
        };
        let at = active
            .enemies
            .iter()
            .find(|s| s.guid == summoner)
            .map_or(home, |s| s.position);
        let mut called = Vec::new();
        for (a, s) in d.enemies.iter().zip(active.enemies.iter_mut()) {
            if s.dormant && a.summon_below_percent.is_some_and(|below| percent < below) {
                s.dormant = false;
                // Keep the authored offset from the summoner, around where it
                // stands now.
                s.position = [
                    at[0] + (a.position[0] - home[0]),
                    at[1],
                    at[2] + (a.position[2] - home[2]),
                ];
                called.push((a.clone(), s.position));
            }
        }
        if called.is_empty() {
            return;
        }
        out.replies
            .push(animation(summoner, SUMMON_ANIMATION, false));
        let mut guids = Vec::new();
        for (a, position) in &called {
            out.replies
                .push(composite_effect(a.guid, SUMMON_EFFECT, *position));
            out.replies.push(self.spawn(a, *position, 0));
            out.replies
                .push(enemy_hp(a.guid, a.max_health, a.max_health));
            out.replies.push(animation(a.guid, SPAWN_ANIMATION, false));
            guids.push(a.guid);
        }
        out.replies.push(relevance(&guids, false));
    }

    /// The encounter's summoner while any of its summoned minions stands
    /// (The owner: "you kill those, then keep attacking the summoner") (R).
    pub(super) fn shielded(&self, guid: u64) -> bool {
        let (Some(d), Some(active)) = (self.dungeon(), self.state.active.as_ref()) else {
            return false;
        };
        d.enemies.iter().any(|a| a.guid == guid && a.skittish)
            && d.enemies
                .iter()
                .zip(&active.enemies)
                .any(|(a, s)| a.summon_below_percent.is_some() && s.health > 0 && !s.dormant)
    }
}
