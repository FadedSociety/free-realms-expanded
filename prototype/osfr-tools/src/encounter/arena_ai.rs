//! One battle run's enemy simulation: each enemy's brain memory, attack clocks, walk set and
//! attacks in flight, and the solo and the shared enemy tick. Split from `arena.rs` by the
//! combat seam so the brain (P3) and the run lifecycle (P5, `arena.rs`) never share a file.
//!
//! Every 250 ms decision goes through `brain::step`. The tick moves and swings by the stance
//! the brain holds; everything else the brain decides is applied by `acts.rs`.
use super::brain::{Act, Attack, AttackKind, Memory, Move, TargetRule, View, Who};
use super::*;

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
    /// Fleeing actors: running or resting, until a time.
    pub(super) kiting: BTreeMap<u64, Kite>,
    /// Shared runs only: enemy GUID to the member who last damaged it (its target).
    pub(super) last_hit: BTreeMap<u64, u64>,
    /// The member whose action holds a shared run's sim; `None` for a solo run (and so for
    /// `ArenaSim::default()` after every `reset_session`, which also detaches a member).
    pub(super) acting: Option<u64>,
    /// What each enemy's brain remembers.
    pub(super) brains: BTreeMap<u64, Memory>,
    /// Acts the tick's decisions left for the engine (`apply_act`), in order.
    pub(super) decided: Vec<(u64, Act)>,
    /// Signals every brain hears at this tick, and those raised for the next one.
    pub(super) signals: Vec<String>,
    pub(super) raised: Vec<String>,
    /// Attacks with a wind-up or a flight, not landed yet.
    pub(super) landings: Vec<Landing>,
    /// Tags a `summon` gave to the helpers it raised.
    pub(super) tagged: BTreeMap<u64, Vec<String>>,
    /// What the `died` rules running right now decided for their enemy (`acts::Death`).
    pub(super) verdict: Option<acts::Death>,
    /// `leave`: (due, enemy).
    pub(super) leaving: Vec<(u64, u64)>,
    /// Where each player was at its last tick (solo: player 1).
    pub(super) seen: BTreeMap<u64, [f32; 3]>,
    /// When each enemy fell, so a `summon` can raise a fallen helper again.
    pub(super) fallen: BTreeMap<u64, u64>,
    /// (R) Generator for special hits and relocation spots.
    pub(super) rolls: u64,
    /// What a paced run spent of its hearts, and its rounds (`pacing.rs`).
    pub(super) pace: super::pacing::PaceRun,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Kite {
    Run(u64),
    Rest(u64),
}

/// An attack on its way: it lands at `due` on `target` if that player is then in its area.
#[derive(Clone, Debug)]
pub(super) struct Landing {
    due: u64,
    guid: u64,
    target: u64,
    point: [f32; 3],
    attack: Attack,
    /// The attacker's normal hit.
    base: u32,
}

/// What one arena tick reads from the engine; the sim itself is `&mut`.
pub(super) struct ArenaTick<'a> {
    pub(super) position: [f32; 3],
    /// The player these enemies fight (1 in a solo run).
    pub(super) target: u64,
    pub(super) ground: &'a Ground,
    pub(super) behaviours: &'a BTreeMap<u64, Behaviour>,
    pub(super) skills: &'a SkillEffects,
    pub(super) mitigation: f32,
    pub(super) player_health: u32,
    pub(super) max_health: u32,
    pub(super) now_ms: u64,
    pub(super) elapsed: u64,
    /// Living, woken actors per tag, over the whole roster.
    pub(super) tags_alive: &'a BTreeMap<String, u32>,
    /// A paced fight's cap on enemies swinging at this player at once (`pacing.rs`).
    pub(super) max_attackers: Option<u32>,
    /// A paced fight's floor on the time between one enemy's swings (0: none).
    pub(super) attack_ms: u64,
    /// A paced fight (one entered from the overworld) has no notice radius: as captured in the
    /// Frostfang Growlers battle (p12), every enemy goes for the player from wherever it spawned
    /// (owner, rc-0137: the hooligans "don't all come at the player").
    pub(super) battle: bool,
    /// The player stood up from a knockout moments ago: nothing swings at them or lands on
    /// them yet (`combat_rules.json` `revive.fight_grace_ms`, owner 2026-11-13).
    pub(super) guarded: bool,
}

/// Health as the whole percent the rules compare (the old summon ladder's arithmetic).
pub(super) fn health_pct(health: u32, max_health: u32) -> u32 {
    (u64::from(health) * 100 / u64::from(max_health.max(1))) as u32
}

/// The tags of enemy `guid`: its behaviour's and those a summon gave it.
pub(super) fn tags_of<'a>(
    guid: u64,
    behaviours: &'a BTreeMap<u64, Behaviour>,
    tagged: &'a BTreeMap<u64, Vec<String>>,
) -> impl Iterator<Item = &'a String> {
    let own = behaviours.get(&guid).map(|b| b.brain.tags.as_slice());
    let given = tagged.get(&guid).map(Vec::as_slice);
    own.unwrap_or_default()
        .iter()
        .chain(given.unwrap_or_default())
}

/// Living, woken actors per tag: `states` is the whole roster.
pub(super) fn tags_alive(
    states: &[EnemyState],
    behaviours: &BTreeMap<u64, Behaviour>,
    tagged: &BTreeMap<u64, Vec<String>>,
) -> BTreeMap<String, u32> {
    let mut alive = BTreeMap::new();
    for s in states.iter().filter(|s| s.health > 0 && !s.dormant) {
        let tags: BTreeSet<&String> = tags_of(s.guid, behaviours, tagged).collect();
        for tag in tags {
            *alive.entry(tag.clone()).or_default() += 1;
        }
    }
    alive
}

impl ArenaSim {
    /// A defeated enemy leaves the walk-away, target and brain books.
    pub(super) fn forget(&mut self, guid: u64) {
        self.kiting.remove(&guid);
        self.last_hit.remove(&guid);
        self.brains.remove(&guid);
    }

    /// 0 to 1, the same generator as the combat crit rolls.
    pub(super) fn roll(&mut self) -> f32 {
        self.rolls = self
            .rolls
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.rolls >> 40) as f32 / 16_777_216.0
    }

    /// Signals raised since the last tick become the ones every brain hears now.
    pub(super) fn turn_signals(&mut self) {
        self.signals = std::mem::take(&mut self.raised);
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
        self.seen.insert(t.target, position);
        // Attacks that were on their way to this player.
        let (landed, flying): (Vec<_>, Vec<_>) = std::mem::take(&mut self.landings)
            .into_iter()
            .partition(|l| l.due <= now_ms && l.target == t.target);
        self.landings = flying;
        for l in landed {
            let from = states.iter().find(|s| s.guid == l.guid).map(|s| s.position);
            let within = match l.attack.kind {
                // (R) One metre of grace: the swing was already on its way.
                AttackKind::Melee => {
                    from.is_some_and(|from| distance(position, from) <= l.attack.reach + 1.0)
                }
                _ => distance(position, l.point) <= l.attack.radius,
            };
            if l.attack.land_effect != 0 {
                out.replies
                    .push(composite_effect(l.guid, l.attack.land_effect, l.point));
            }
            if within && !t.guarded {
                let dealt = self.land(l.guid, &l.attack, l.base, t, total_damage, out);
                total_damage = total_damage.saturating_add(dealt);
            }
        }
        // A paced fight lets only the enemies nearest the player swing; the rest hold.
        let front: Option<BTreeSet<u64>> = t.max_attackers.map(|most| {
            let mut near: Vec<(f32, u64)> = (actors.iter())
                .filter_map(|a| states.iter().find(|s| s.guid == a.guid))
                .filter(|s| s.health > 0 && !s.dormant)
                .map(|s| (distance(position, s.position), s.guid))
                .collect();
            near.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            near.into_iter().take(most as usize).map(|(_, guid)| guid).collect()
        });
        let plain = Behaviour::default();
        for actor in actors {
            let guid = actor.guid;
            let Some(i) = states.iter().position(|s| s.guid == guid) else {
                continue;
            };
            if states[i].health == 0 || states[i].dormant {
                continue;
            }
            if t.skills.stunned(guid, now_ms) {
                continue;
            }
            let b = t.behaviours.get(&guid).unwrap_or(&plain);
            // L092: a scared enemy runs straight away from its scarer and strikes nobody.
            if let Some(from) = t.skills.fleeing_from(guid, now_ms) {
                let s = &mut states[i];
                let step = b.move_speed * elapsed.min(250) as f32 / 1000.0;
                let unit = super::battle_items::away_from(s.position, from, 1.0);
                let away = [unit[0] - s.position[0], 0.0, unit[2] - s.position[2]];
                let mut next = super::battle_items::away_from(s.position, from, step);
                t.ground.snap(&mut next);
                s.position = next;
                out.replies.push(movement(guid, next, away));
                if self.moving.insert(guid) {
                    out.replies.push(animation(guid, 3, true));
                }
                out.changed = true;
                continue;
            }
            // The decision.
            let program = b.brain.program.clone();
            let delta = distance(position, states[i].position);
            let memory = self
                .brains
                .entry(guid)
                .or_insert_with(|| Memory::seeded(guid));
            // Given up, or knocked down and waiting to stand up again.
            if memory.yielded || now_ms < memory.down_until {
                continue;
            }
            memory.spawned(now_ms);
            // An enemy engages only once its target is inside its chase limit, as the engine
            // before the brain did: a far party leaves it idle, and its `every` rules do not
            // start (lead 2026-11-10: they cast at players standing at the entry and flooded
            // every member of a shared run with packets).
            let chase_limit = if t.battle { f32::INFINITY } else { b.chase_limit };
            if delta > chase_limit && !memory.is_engaged() {
                if self.moving.remove(&guid) {
                    out.replies.push(animation(guid, 1, true));
                }
                continue;
            }
            memory.engaged(now_ms);
            let view = View {
                now_ms,
                health_pct: health_pct(states[i].health, actor.max_health),
                position: states[i].position,
                home: actor.position,
                target: Some(t.target),
                target_distance: Some(delta),
                nearest_player: Some(delta),
                tags_alive: t.tags_alive,
                signals: &self.signals,
                phase: memory.phase(),
                night: false,
            };
            let mut acts = Vec::new();
            brain::step(&program, memory, &view, &mut acts);
            for act in acts {
                match act {
                    Act::Stance(_) | Act::Phase(_) => {}
                    Act::Attack(id) => memory.ordered = Some(id),
                    other => self.decided.push((guid, other)),
                }
            }
            let stance = memory.stance(&program).clone();
            let ordered = memory.ordered.take();
            let speed = memory.flags.speed.unwrap_or(b.move_speed);
            let attack = ordered
                .as_deref()
                .or(stance.attack.as_deref())
                .and_then(|id| program.attack(id))
                .cloned();
            let reach = attack.as_ref().map_or(b.reach, Attack::reach);
            // Fleeing: run, rest, run again, striking nobody.
            if let Move::Flee { ms, speed } = stance.movement {
                self.kite_turn(&actor, &mut states[i], t, b, u64::from(ms), speed, out);
                continue;
            }
            self.kiting.remove(&guid);
            if delta > chase_limit {
                if self.moving.remove(&guid) {
                    out.replies.push(animation(guid, 1, true));
                }
                continue;
            }
            let pace = speed * elapsed.min(250) as f32 / 1000.0;
            let rooted = t.skills.rooted(guid, now_ms);
            let mut stands = true;
            match &stance.movement {
                Move::Chase if delta > reach && !rooted => {
                    let s = &mut states[i];
                    let heading = [
                        (position[0] - s.position[0]) / delta,
                        0.0,
                        (position[2] - s.position[2]) / delta,
                    ];
                    // Stop inside reach so f32 rounding at world coordinates cannot
                    // strand a pursuer just outside the attack threshold.
                    let step = pace.min(delta - (reach - 0.1));
                    for (coord, to) in s.position.iter_mut().zip(position) {
                        *coord += (to - *coord) / delta * step;
                    }
                    t.ground.snap(&mut s.position);
                    out.replies.push(movement(guid, s.position, heading));
                    if self.moving.insert(guid) {
                        out.replies.push(animation(guid, 3, true));
                    }
                    out.changed = true;
                }
                Move::KeepAway { min, max } if !rooted => {
                    stands = false;
                    if delta < *min {
                        if !self.run_from(&actor, &mut states[i], t, b.leash, pace, out) {
                            self.stop(guid, out);
                        }
                    } else if delta > *max {
                        let step = pace.min(delta - (max - 0.1));
                        self.walk_to(guid, &mut states[i], position, step, t, out);
                    } else {
                        self.stop(guid, out);
                    }
                }
                Move::Behind { tag, m } if !rooted => {
                    stands = false;
                    // Behind the tagged allies: `m` past their middle, on the far side
                    // from the player.
                    let allies: Vec<[f32; 3]> = states
                        .iter()
                        .filter(|s| s.guid != guid && s.health > 0 && !s.dormant)
                        .filter(|s| tags_of(s.guid, t.behaviours, &self.tagged).any(|x| x == tag))
                        .map(|s| s.position)
                        .collect();
                    let goal = (!allies.is_empty()).then(|| {
                        let n = allies.len() as f32;
                        let mid = [
                            allies.iter().map(|p| p[0]).sum::<f32>() / n,
                            states[i].position[1],
                            allies.iter().map(|p| p[2]).sum::<f32>() / n,
                        ];
                        super::battle_items::away_from(mid, position, *m)
                    });
                    match goal {
                        Some(goal) if distance(states[i].position, goal) > 0.5 => {
                            self.walk_to(guid, &mut states[i], goal, pace, t, out);
                        }
                        _ => self.stop(guid, out),
                    }
                }
                Move::Home if !rooted => {
                    stands = false;
                    if distance(states[i].position, actor.position) > 0.5 {
                        self.walk_to(guid, &mut states[i], actor.position, pace, t, out);
                    } else {
                        self.stop(guid, out);
                    }
                }
                _ => {}
            }
            let Some(attack) = attack else {
                continue;
            };
            // An ordered burst around the enemy goes off wherever the player stands.
            let anywhere =
                ordered.is_some() && attack.kind == AttackKind::Burst && !attack.at_target();
            if distance(position, states[i].position) > reach && !anywhere {
                if let (Some(id), Some(memory)) = (ordered, self.brains.get_mut(&guid)) {
                    // Keep the order until the target is in reach.
                    memory.ordered = Some(id);
                }
                continue;
            }
            if stands {
                self.stop(guid, out);
            }
            if t.guarded || front.as_ref().is_some_and(|front| !front.contains(&guid)) {
                continue;
            }
            if ordered.is_none()
                && self
                    .attacks
                    .get(&guid)
                    .is_some_and(|last| {
                        now_ms.saturating_sub(*last) < attack.interval_ms.max(t.attack_ms)
                    })
            {
                continue;
            }
            self.attacks.insert(guid, now_ms);
            out.replies.push(animation(guid, attack.animation, false));
            let wait = attack.windup_ms.saturating_add(attack.travel_ms);
            if wait == 0 {
                let dealt = self.land(guid, &attack, actor.damage, t, total_damage, out);
                total_damage = total_damage.saturating_add(dealt);
                continue;
            }
            let point = match attack.kind {
                AttackKind::Burst if !attack.at_target() => states[i].position,
                _ => position,
            };
            if attack.projectile_effect != 0 {
                out.replies
                    .push(composite_effect(guid, attack.projectile_effect, point));
            }
            if self.landings.len() < 256 {
                self.landings.push(Landing {
                    due: now_ms.saturating_add(wait),
                    guid,
                    target: t.target,
                    point,
                    attack,
                    base: actor.damage,
                });
            }
        }
        total_damage
    }

    /// An attack lands on the tick's player: the hit packet, and its riders left for
    /// `apply_act`. Returns the damage before the player's remaining health caps it.
    fn land(
        &mut self,
        guid: u64,
        attack: &Attack,
        base: u32,
        t: &ArenaTick,
        taken: u32,
        out: &mut EncounterOutcome,
    ) -> u32 {
        let special = match &attack.special {
            Some(special) if self.roll() < special.chance => Some(special),
            _ => None,
        };
        let hit = special.map_or(attack.hit, |s| s.hit);
        let attack_damage = (base as f32 * hit * t.mitigation).round() as u32;
        let left = t.player_health.saturating_sub(taken);
        let damage = attack_damage.min(left);
        out.replies.push(wire::enemy_hit(&wire::EnemyHit {
            attacker: guid,
            target: 1,
            damage,
            target_max: t.max_health,
            target_current: left.saturating_sub(damage),
            effect: special.map_or(attack.effect, |s| s.effect),
            special: special.is_some(),
            knockout: damage > 0 && damage == left,
        }));
        for rider in &attack.riders {
            self.decided.push((
                guid,
                Act::Status {
                    id: rider.status.clone(),
                    to: Who::Target,
                    ms: rider.ms,
                },
            ));
        }
        attack_damage
    }

    /// Stand still: end the walk animation if one is playing.
    fn stop(&mut self, guid: u64, out: &mut EncounterOutcome) {
        if self.moving.remove(&guid) {
            out.replies.push(animation(guid, 1, true));
        }
    }

    /// One step of `step` metres toward `goal`.
    fn walk_to(
        &mut self,
        guid: u64,
        s: &mut EnemyState,
        goal: [f32; 3],
        step: f32,
        t: &ArenaTick,
        out: &mut EncounterOutcome,
    ) {
        let left = distance(s.position, goal);
        if left <= 0.001 || step <= 0.0 {
            return;
        }
        let heading = [
            (goal[0] - s.position[0]) / left,
            0.0,
            (goal[2] - s.position[2]) / left,
        ];
        let step = step.min(left);
        for (coord, to) in s.position.iter_mut().zip(goal) {
            *coord += (to - *coord) / left * step;
        }
        t.ground.snap(&mut s.position);
        out.replies.push(movement(guid, s.position, heading));
        if self.moving.insert(guid) {
            out.replies.push(animation(guid, 3, true));
        }
        out.changed = true;
    }

    /// One step straight away from the player, or along the leash edge (either tangent)
    /// when cornered. `false` when there is nowhere to go.
    fn run_from(
        &mut self,
        actor: &Actor,
        s: &mut EnemyState,
        t: &ArenaTick,
        leash: f32,
        step: f32,
        out: &mut EncounterOutcome,
    ) -> bool {
        let player = t.position;
        let delta = distance(player, s.position).max(0.01);
        let away = [
            (s.position[0] - player[0]) / delta,
            0.0,
            (s.position[2] - player[2]) / delta,
        ];
        let directions = [away, [away[2], 0.0, -away[0]], [-away[2], 0.0, away[0]]];
        let next = directions.iter().find_map(|d| {
            let next = t.ground.snapped([
                s.position[0] + d[0] * step,
                player[1],
                s.position[2] + d[2] * step,
            ]);
            let flat = [next[0], actor.position[1], next[2]];
            (distance(flat, actor.position) <= leash).then_some((next, *d))
        });
        let Some((next, heading)) = next else {
            return false;
        };
        s.position = next;
        out.replies.push(movement(actor.guid, s.position, heading));
        if self.moving.insert(actor.guid) {
            out.replies.push(animation(actor.guid, 3, true));
        }
        out.changed = true;
        true
    }

    /// One tick of a fleeing actor: run from the player for `run_ms` at `run_speed`, then
    /// rest (no attacks), then run again. The rest, the leash and the distance it starts to
    /// run at are the behaviour's own numbers.
    #[allow(clippy::too_many_arguments)]
    fn kite_turn(
        &mut self,
        actor: &Actor,
        s: &mut EnemyState,
        t: &ArenaTick,
        b: &Behaviour,
        run_ms: u64,
        run_speed: f32,
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
            .or_insert(Kite::Run(now_ms.saturating_add(run_ms)));
        match phase {
            Kite::Rest(until) if now_ms < until => {}
            Kite::Rest(_) => {
                self.kiting
                    .insert(actor.guid, Kite::Run(now_ms.saturating_add(run_ms)));
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
                let step = run_speed * elapsed.min(250) as f32 / 1000.0;
                self.run_from(actor, s, t, b.leash, step, out);
            }
        }
    }
}

/// An enemy's target (R): its last attacker while that member is standing in the fight, else
/// the nearest standing member. `players` holds only members who can be hit.
#[cfg_attr(not(test), allow(dead_code))] // the plain rule, kept for the run lifecycle's tests
pub(super) fn target_of(
    last_hit: &BTreeMap<u64, u64>,
    guid: u64,
    at: [f32; 3],
    players: &[FieldParticipant],
) -> Option<u64> {
    target_by(last_hit, None, guid, at, players)
}

/// `target_of`, unless a `target` rule of `nearest` or `lowest_health` picks that way instead.
pub(super) fn target_by(
    last_hit: &BTreeMap<u64, u64>,
    rule: Option<TargetRule>,
    guid: u64,
    at: [f32; 3],
    players: &[FieldParticipant],
) -> Option<u64> {
    let nearest = || {
        players
            .iter()
            .min_by(|a, b| distance(a.position, at).total_cmp(&distance(b.position, at)))
            .map(|p| p.guid)
    };
    match rule {
        Some(TargetRule::Nearest) => nearest(),
        Some(TargetRule::LowestHealth) => players
            .iter()
            .min_by_key(|p| (p.health, p.guid))
            .map(|p| p.guid),
        // (R) No taunt exists yet: a taunter rule falls back to the attacker.
        _ => last_hit
            .get(&guid)
            .filter(|hitter| players.iter().any(|p| p.guid == **hitter))
            .copied()
            .or_else(nearest),
    }
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
        let mut upkeep = EncounterOutcome::default();
        self.arena_upkeep(now_ms, &mut upkeep);
        let ground = self.ground();
        let (max_attackers, attack_ms) = (self.max_attackers(), self.paced_attack_ms());
        let battle = self.pacing().is_some();
        let Some(active) = &mut self.state.active else {
            return Vec::new();
        };
        let states = &mut active.enemies;
        let alive = tags_alive(states, &self.behaviour_of, &self.arena.tagged);
        let targets: BTreeMap<u64, u64> = actors
            .iter()
            .filter_map(|a| {
                let s = states.iter().find(|s| s.guid == a.guid)?;
                if s.health == 0 || s.dormant {
                    return None;
                }
                let rule = self.arena.brains.get(&a.guid).and_then(|m| m.target_rule);
                let target = target_by(&self.arena.last_hit, rule, a.guid, s.position, players)?;
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
                target: p.guid,
                ground: &ground,
                behaviours: &self.behaviour_of,
                skills: &self.skills,
                mitigation: p.mitigation,
                player_health: p.health,
                max_health: p.max_health,
                now_ms,
                elapsed,
                tags_alive: &alive,
                max_attackers,
                attack_ms,
                battle,
                guarded: now_ms < p.grace_until,
            };
            let mut out = EncounterOutcome::default();
            let damage = self.arena.tick_enemies(mine, states, &tick, &mut out);
            hits.push((p.guid, damage, out.replies));
        }
        // What the brains decided besides moving and swinging; every member sees it.
        self.apply_decided(now_ms, &mut upkeep);
        if !upkeep.replies.is_empty() {
            match hits.first_mut() {
                Some((_, _, packets)) => packets.append(&mut upkeep.replies),
                None => {
                    if let Some(p) = players.first() {
                        hits.push((p.guid, 0, upkeep.replies));
                    }
                }
            }
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
        // A shared run's enemies tick once per instance in the driver (`tick_arena_enemies`).
        if self.arena.acting.is_some() {
            return 0;
        }
        self.arena_upkeep(now_ms, out);
        let mitigation = self.skills.mitigation();
        let ground = self.ground();
        let (max_attackers, attack_ms) = (self.max_attackers(), self.paced_attack_ms());
        let battle = self.pacing().is_some();
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
        let alive = tags_alive(states, &self.behaviour_of, &self.arena.tagged);
        let tick = ArenaTick {
            position,
            target: 1,
            ground: &ground,
            behaviours: &self.behaviour_of,
            skills: &self.skills,
            mitigation,
            player_health: self.state.player_health,
            max_health: self.max_health,
            now_ms,
            elapsed,
            tags_alive: &alive,
            max_attackers,
            attack_ms,
            battle,
            guarded: now_ms < self.fight_state.guarded_until,
        };
        let damage = self.arena.tick_enemies(actors, states, &tick, out);
        self.apply_decided(now_ms, out);
        damage
    }

    /// An enemy takes no damage: a rule made it invulnerable or untargetable, it gave up,
    /// or an actor with its shield tag stands (the old summoner behind its minions; The
    /// owner: "you kill those, then keep attacking the summoner") (R).
    pub(super) fn shielded(&self, guid: u64) -> bool {
        let memory = self
            .arena
            .brains
            .get(&guid)
            .or_else(|| self.field_sim.brains.get(&guid));
        if memory.is_some_and(|m| {
            m.yielded
                || m.down_until > self.last_tick.unwrap_or(0)
                || m.flags.invulnerable == Some(true)
                || m.flags.untargetable == Some(true)
        }) {
            return true;
        }
        let Some(tag) = self
            .behaviour_of
            .get(&guid)
            .and_then(|b| b.brain.shield.as_ref())
        else {
            return false;
        };
        let (Some(_), Some(active)) = (self.dungeon(), self.state.active.as_ref()) else {
            return false;
        };
        active.enemies.iter().any(|s| {
            s.guid != guid
                && s.health > 0
                && !s.dormant
                && tags_of(s.guid, &self.behaviour_of, &self.arena.tagged).any(|x| x == tag)
        })
    }
}
