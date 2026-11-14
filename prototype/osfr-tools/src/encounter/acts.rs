//! The engine side of the brain: applying acts, and the points where the engine tells the
//! brain that something happened. Piece P3 (`combat-brain`) owns the bodies.
//!
//! Frozen by the combat seam: the names and signatures of `apply_act`, `load_brains`,
//! `enemies_spawned`, `enemy_hit_taken` and `enemy_died`, and the `Death` answer;
//! `encounter.rs` calls each hook once.
//!
//! What an act can reach today:
//! - A battle's roster is fixed, so `summon` and `become` take their actors from the fight's
//!   own sleeping (dormant) actors, found by kind (the name as a slug, or a tag). A `summon`
//!   may raise a fallen helper again, which is how a summoner repeats without end.
//! - `status` on a player has no status table to land in yet (P2): it is written to the fight
//!   log only. A stun on an enemy uses the engine's stun.
//! - At zero health a `died` rule decides the death (`Death`): `revive` keeps the enemy (down
//!   for `after_ms`, then up at `pct`), `become` hands over to the next form, `yield` counts
//!   as the kill but leaves the actor standing. A `yield` above zero health only stops the
//!   fighting; the kill is credited by the hub's death path, which it does not reach.
//! - A signal goes to every brain of the run at its next decision and to the director. The
//!   fight raises one for the brains alone with `raise_signal`.
use super::arena_ai::{health_pct, tags_alive, tags_of};
use super::behaviour::slug;
use super::brain::{Act, Flags, Memory, Place, TargetRule, View, Who};
use super::*;

/// (R) The disposition of an enemy that gave up: the talk-to value the cave guide spawns with.
const YIELDED_DISPOSITION: i32 = 2;
/// (R) What a knocked-down enemy plays while it waits to stand up again (com_stunned).
const DOWN_ANIMATION: i32 = 1401;

/// The engine field `brain_state`: what `behaviours.json` and `attacks.json` hold besides
/// the presets (attacks, looks, the open-world rules, `attach`). It outlives `reset_session`;
/// per-run memory belongs in `ArenaSim`. No other piece looks inside.
#[derive(Clone, Debug, Default)]
pub struct BrainState {
    pub(super) data: Arc<super::behaviour::BrainData>,
}

/// What `enemy_died` decides for an enemy that reached zero health.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Death {
    /// Dies as before: removal, kill credit, goal progress, drops.
    Die,
    /// `yield`: credited as a kill with everything that follows, but no removal is sent; the
    /// actor stays as an NPC.
    Yield,
    /// `become`, `revive`: not a death. Nothing of the death path runs; the brain has already
    /// put the enemy's health back and sent its own packets.
    Survive,
}

impl EncounterEngine {
    /// Carry out one act of enemy `guid` (arena and field enemies alike).
    pub(super) fn apply_act(
        &mut self,
        guid: u64,
        act: &brain::Act,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let field = self.brain_in_field(guid);
        match act {
            Act::Stance(stance) => self.memory(guid, field).set_stance(stance.clone()),
            Act::Phase(n) => self.memory(guid, field).set_phase(*n),
            Act::Attack(id) => self.memory(guid, field).ordered = Some(id.clone()),
            Act::Say(line) => out.replies.push(wire::say(guid, &line.pick(guid, now_ms))),
            Act::Telegraph { say, ms, attack } => {
                out.replies.push(wire::say(guid, say));
                self.memory(guid, field).defer(
                    now_ms.saturating_add(u64::from(*ms)),
                    Act::Attack(attack.clone()),
                );
            }
            Act::Signal(name) => {
                self.raise_signal(name);
                if let Some(run) = self.run.run.as_mut() {
                    let mut answers = Vec::new();
                    director::on_fact(run, &director::Signal::Named(name.clone()), &mut answers);
                    self.arena
                        .decided
                        .extend(answers.into_iter().map(|a| (guid, a)));
                }
            }
            Act::Target(rule) => {
                self.memory(guid, field).target_rule = Some(*rule);
                if *rule == TargetRule::Nearest {
                    self.arena.last_hit.remove(&guid);
                    if let Some(f) = self.field_enemy_mut(guid) {
                        f.threat.clear();
                    }
                }
            }
            Act::Set(flags) => self.set_flags(guid, flags, field, out),
            Act::Heal { who, pct } => self.heal(guid, who, *pct, field, out),
            Act::Status { id, to, ms } => self.status(guid, id, to, *ms, now_ms, field, out),
            Act::Relocate {
                to,
                min,
                max,
                effect,
            } => self.relocate(guid, to, *min, *max, effect.as_deref(), field, out),
            Act::Wake(tag) if field => self.wake_field(guid, tag),
            Act::Wake(tag) => self.wake(guid, tag, out),
            Act::Yield => {
                if self.at_zero(guid) {
                    self.arena.verdict = Some(Death::Yield);
                }
                self.memory(guid, field).yielded = true;
                self.arena.moving.remove(&guid);
                self.field_sim.moving.remove(&guid);
                out.replies.push(animation(guid, 1, true));
                out.replies
                    .push(wire::disposition(guid, YIELDED_DISPOSITION));
            }
            Act::Drop(table) => {
                if let Some(at) = self.enemy(guid).map(|(_, s)| s.position) {
                    self.drop_power_ups(table, (guid, now_ms), at, now_ms, 1, out);
                }
            }
            // The rest needs a roster to draw from: battles only.
            _ if field => {}
            Act::Summon {
                kind,
                count,
                at,
                radius,
                tag,
                mound,
            } => self.summon(guid, kind, *count, at, *radius, tag, *mound, now_ms, out),
            Act::Become {
                definition,
                health_pct,
            } => {
                let dying = self.at_zero(guid);
                if self.become_kind(guid, definition, *health_pct, out) && dying {
                    self.arena.verdict = Some(Death::Survive);
                }
            }
            Act::Leave(ms) => self
                .arena
                .leaving
                .push((now_ms.saturating_add(u64::from(*ms)), guid)),
            Act::Revive { pct, after_ms } => {
                if self.stand_up_later(guid, *pct, *after_ms, now_ms, out) {
                    self.arena.verdict = Some(Death::Survive);
                }
            }
        }
        out.changed = true;
    }

    /// Engine load: read this piece's own content from `directory` (absent files leave the
    /// feature dormant). The brain's content is `behaviours.json` with `attacks.json` beside
    /// it; both load through `load_behaviours`, which is also the hot reload, so there is
    /// nothing more to read here.
    pub(super) fn load_brains(&mut self, _directory: &Path) -> io::Result<()> {
        Ok(())
    }

    /// A run's actors were sent to the client (`guids` holds every actor shown, the guide too).
    /// Each enemy's memory starts here; one that is already hurt belongs to a resumed run and
    /// keeps quiet about the health thresholds it passed before. Nothing is decided yet: a
    /// group's members each get this call on their own engine, so the first decision (and a
    /// rule that answers the spawn itself) waits for the first swing or tick, which a shared
    /// run takes on its one shared state.
    pub(super) fn enemies_spawned(
        &mut self,
        guids: &[u64],
        now_ms: u64,
        _out: &mut EncounterOutcome,
    ) {
        let Some(d) = self.dungeon() else {
            return;
        };
        let actors = d.enemies.clone();
        for actor in actors.iter().filter(|a| guids.contains(&a.guid)) {
            let Some(health) = self.enemy(actor.guid).map(|(_, s)| s.health) else {
                continue;
            };
            let mut memory = if health < actor.max_health {
                Memory::resumed(actor.guid, health_pct(health, actor.max_health))
            } else {
                Memory::seeded(actor.guid)
            };
            memory.spawned(now_ms);
            self.arena.brains.insert(actor.guid, memory);
        }
    }

    /// A player hit landed on enemy `guid` for `dealt`, leaving it `hp` of `max_health`.
    /// The brain answers at once (a summon ladder shows in the same reply as the hit).
    pub(super) fn enemy_hit_taken(
        &mut self,
        guid: u64,
        _dealt: u32,
        hp: u32,
        _max_health: u32,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let field = self.brain_in_field(guid);
        let memory = self.memory(guid, field);
        memory.hit();
        memory.alert |= !field;
        if hp > 0 && !field {
            self.think(guid, now_ms, out);
        }
    }

    /// Enemy `guid` reached zero health; called before its removal is sent. Its `died` rules
    /// run now and the answer decides whether it dies (`Death`).
    pub(super) fn enemy_died(
        &mut self,
        guid: u64,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> Death {
        if self.brain_in_field(guid) {
            // A field enemy starts afresh when it respawns.
            self.field_sim.brains.remove(&guid);
            self.field_sim.homing.remove(&guid);
            return Death::Die;
        }
        self.arena.verdict = None;
        self.arena
            .brains
            .entry(guid)
            .or_insert_with(|| Memory::seeded(guid))
            .died();
        self.think(guid, now_ms, out);
        let death = self.arena.verdict.take().unwrap_or(Death::Die);
        if death != Death::Survive {
            self.arena.fallen.insert(guid, now_ms);
            self.arena.tagged.remove(&guid);
        }
        death
    }

    /// The enemy is at zero health right now (its `died` rules are running).
    fn at_zero(&self, guid: u64) -> bool {
        self.states().iter().any(|s| s.guid == guid && s.health == 0)
    }

    /// `revive`, for an enemy at zero health: it is not dead. It lies down for `after_ms`
    /// taking no damage, then fights on at `pct` of its health. `false` when it was not at
    /// zero (nothing to revive).
    fn stand_up_later(
        &mut self,
        guid: u64,
        pct: u32,
        after_ms: u32,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> bool {
        let Some(max_health) = self
            .roster()
            .iter()
            .find(|a| a.guid == guid)
            .map(|a| a.max_health)
        else {
            return false;
        };
        let Some(s) = self.states_mut().iter_mut().find(|s| s.guid == guid) else {
            return false;
        };
        if s.health > 0 {
            return false;
        }
        s.health = (u64::from(max_health) * u64::from(pct) / 100).max(1) as u32;
        let health = s.health;
        self.memory(guid, false).down_until = now_ms.saturating_add(u64::from(after_ms));
        self.arena.moving.remove(&guid);
        self.arena.kiting.remove(&guid);
        out.replies.push(animation(guid, 1, true));
        if after_ms > 0 {
            out.replies.push(animation(guid, DOWN_ANIMATION, false));
        }
        out.replies.extend(enemy_health(guid, health, max_health));
        true
    }

    /// A player interacted with enemy `guid`: its `talked` rules answer. Not called by the hub
    /// yet; the director calls it for stage hosts.
    pub fn brain_talked(&mut self, guid: u64, now_ms: u64) -> EncounterOutcome {
        let mut out = EncounterOutcome::default();
        let field = self.brain_in_field(guid);
        self.memory(guid, field).talked();
        if !field {
            self.think(guid, now_ms, &mut out);
        }
        out
    }

    /// Raise a signal for every brain of the run; they hear it at their next decision.
    pub(super) fn raise_signal(&mut self, name: &str) {
        if self.arena.raised.len() < 64 && !self.arena.raised.iter().any(|s| s == name) {
            self.arena.raised.push(name.to_string());
        }
    }

    /// Apply what the last tick's decisions left for the engine.
    pub(super) fn apply_decided(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        for (guid, act) in std::mem::take(&mut self.arena.decided) {
            self.apply_act(guid, &act, now_ms, out);
        }
    }

    /// Start of an arena tick: turn the signals over, and carry out the departures that
    /// are due.
    pub(super) fn arena_upkeep(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        self.arena.turn_signals();
        let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut self.arena.leaving)
            .into_iter()
            .partition(|(at, _)| *at <= now_ms);
        self.arena.leaving = later;
        for (_, guid) in due {
            let Some(s) = self.states_mut().iter_mut().find(|s| s.guid == guid) else {
                continue;
            };
            if s.health == 0 || s.dormant {
                continue;
            }
            s.dormant = true;
            self.arena.moving.remove(&guid);
            self.arena.forget(guid);
            out.replies.push(remove(guid));
            out.changed = true;
        }
    }

    /// One decision of arena enemy `guid` outside the tick (a hit, its death, a talk): the
    /// same `step`, with the acts applied at once.
    fn think(&mut self, guid: u64, now_ms: u64, out: &mut EncounterOutcome) {
        let Some(program) = self.behaviour_of.get(&guid).map(|b| b.brain.program.clone()) else {
            return;
        };
        if program.rules.is_empty() {
            return;
        }
        let Some(actor) = self.roster().into_iter().find(|a| a.guid == guid) else {
            return;
        };
        let states = self.states();
        let Some(s) = states.iter().find(|s| s.guid == guid) else {
            return;
        };
        if s.dormant {
            return;
        }
        let alive = tags_alive(states, &self.behaviour_of, &self.arena.tagged);
        let (health, position) = (s.health, s.position);
        let target = self.target_guid(guid);
        let target_distance = self
            .arena
            .seen
            .get(&target)
            .map(|at| distance(*at, position));
        let memory = self
            .arena
            .brains
            .entry(guid)
            .or_insert_with(|| Memory::seeded(guid));
        if memory.yielded {
            return;
        }
        let view = View {
            now_ms,
            health_pct: health_pct(health, actor.max_health),
            position,
            home: actor.position,
            target: Some(target),
            target_distance,
            nearest_player: target_distance,
            tags_alive: &alive,
            signals: &[],
            phase: memory.phase(),
            night: false,
        };
        let mut acts = Vec::new();
        brain::step(&program, memory, &view, &mut acts);
        for act in acts {
            self.apply_act(guid, &act, now_ms, out);
        }
    }

    /// The enemy is one of the open world's (its memory lives with the field simulation).
    fn brain_in_field(&self, guid: u64) -> bool {
        !self.active() && self.field_sim.enemies.iter().any(|f| f.actor.guid == guid)
    }

    fn memory(&mut self, guid: u64, field: bool) -> &mut Memory {
        let brains = if field {
            &mut self.field_sim.brains
        } else {
            &mut self.arena.brains
        };
        brains.entry(guid).or_insert_with(|| Memory::seeded(guid))
    }

    fn field_enemy_mut(&mut self, guid: u64) -> Option<&mut FieldEnemy> {
        if self.active() {
            return None;
        }
        self.field_sim
            .enemies
            .iter_mut()
            .find(|f| f.actor.guid == guid)
    }

    /// The actors of the run (or of the opening stage outside one).
    fn roster(&self) -> Vec<Actor> {
        match self.dungeon() {
            Some(d) => d.enemies.clone(),
            None => self.content.world_enemies.clone(),
        }
    }

    fn states(&self) -> &[EnemyState] {
        match &self.state.active {
            Some(active) => &active.enemies,
            None => &self.state.world_enemies,
        }
    }

    fn states_mut(&mut self) -> &mut Vec<EnemyState> {
        match &mut self.state.active {
            Some(active) => &mut active.enemies,
            None => &mut self.state.world_enemies,
        }
    }

    /// Who arena enemy `guid` fights: its last attacker, else the acting member, else the
    /// solo player.
    fn target_guid(&self, guid: u64) -> u64 {
        self.arena
            .last_hit
            .get(&guid)
            .copied()
            .or(self.arena.acting)
            .unwrap_or(1)
    }

    /// Where the enemy stands, its home, how far it may go from home and where its target is.
    fn bearings(&self, guid: u64, field: bool) -> Option<([f32; 3], [f32; 3], f32, Option<[f32; 3]>)> {
        let leash = self.behaviour_of.get(&guid).map_or(12.0, |b| b.leash);
        if field {
            let f = self.field_sim.enemies.iter().find(|f| f.actor.guid == guid)?;
            let target = f.target.and_then(|t| self.field_sim.seen.get(&t).copied());
            return Some((
                f.state.position,
                f.actor.position,
                f.leash_radius.unwrap_or(leash),
                target,
            ));
        }
        let actor = self.roster().into_iter().find(|a| a.guid == guid)?;
        let s = self.states().iter().find(|s| s.guid == guid)?;
        let target = self.arena.seen.get(&self.target_guid(guid)).copied();
        Some((s.position, actor.position, leash, target))
    }

    fn set_position(&mut self, guid: u64, field: bool, position: [f32; 3]) {
        if field {
            if let Some(f) = self.field_enemy_mut(guid) {
                f.state.position = position;
                f.roam_goal = None;
            }
        } else if let Some(s) = self.states_mut().iter_mut().find(|s| s.guid == guid) {
            s.position = position;
        }
    }

    /// `relocate`: be somewhere else at once, facing the target. The spot stays within the
    /// enemy's leash of its home so it cannot leave the arena floor (or its camp).
    #[allow(clippy::too_many_arguments)]
    fn relocate(
        &mut self,
        guid: u64,
        to: &Place,
        min: f32,
        max: f32,
        effect: Option<&str>,
        field: bool,
        out: &mut EncounterOutcome,
    ) {
        let Some((from, home, leash, target)) = self.bearings(guid, field) else {
            return;
        };
        let ground = self.ground();
        let around = target.unwrap_or(from);
        let inside = |p: [f32; 3]| horizontal_distance(p, home) <= leash;
        let spot = match to {
            Place::Own => from,
            Place::Target => super::battle_items::away_from(around, from, -2.0),
            Place::Away => {
                // A spot `min` to `max` from the target, inside the leash; failing that, the
                // leash edge on the far side of home from the target.
                let mut found = None;
                for _ in 0..12 {
                    let angle = self.arena.roll() * std::f32::consts::TAU;
                    let far = min + self.arena.roll() * (max - min);
                    let p = [
                        around[0] + far * angle.cos(),
                        from[1],
                        around[2] + far * angle.sin(),
                    ];
                    if inside(p) && distance(p, from) > 1.0 {
                        found = Some(p);
                        break;
                    }
                }
                found.unwrap_or_else(|| super::battle_items::away_from(home, around, leash))
            }
            Place::BehindAllies => {
                let allies: Vec<[f32; 3]> = self
                    .states()
                    .iter()
                    .filter(|s| s.guid != guid && s.health > 0 && !s.dormant)
                    .map(|s| s.position)
                    .collect();
                if allies.is_empty() || field {
                    super::battle_items::away_from(from, around, max.max(3.0))
                } else {
                    let n = allies.len() as f32;
                    let mid = [
                        allies.iter().map(|p| p[0]).sum::<f32>() / n,
                        from[1],
                        allies.iter().map(|p| p[2]).sum::<f32>() / n,
                    ];
                    super::battle_items::away_from(mid, around, max.max(3.0))
                }
            }
            Place::Ring | Place::Points => {
                let angle = self.arena.roll() * std::f32::consts::TAU;
                [
                    home[0] + leash * angle.cos(),
                    from[1],
                    home[2] + leash * angle.sin(),
                ]
            }
            // Named points belong to the fight file (the director); without one it stays.
            Place::Point(_) => from,
        };
        let spot = if inside(spot) {
            spot
        } else {
            // Pull an outside spot back onto the leash circle.
            let d = horizontal_distance(spot, home).max(0.001);
            [
                home[0] + (spot[0] - home[0]) / d * leash,
                spot[1],
                home[2] + (spot[2] - home[2]) / d * leash,
            ]
        };
        let spot = if field {
            match ground.field_position(spot) {
                Some(spot) => spot,
                None => return,
            }
        } else {
            ground.snapped(spot)
        };
        if let Some(id) = effect.and_then(|name| self.brain_data().visuals.effects.get(name).copied())
        {
            out.replies.push(composite_effect(guid, id, from));
        }
        self.set_position(guid, field, spot);
        self.arena.kiting.remove(&guid);
        let d = distance(around, spot).max(0.01);
        let facing = [(around[0] - spot[0]) / d, 0.0, (around[2] - spot[2]) / d];
        out.replies.push(movement(guid, spot, facing));
        if self.arena.moving.remove(&guid) | self.field_sim.moving.remove(&guid) {
            out.replies.push(animation(guid, 1, true));
        }
    }

    /// `wake`: the sleeping actors with `tag` appear around the caller, each keeping its
    /// authored offset from the caller's home, in a puff while the caller plays its cast.
    fn wake(&mut self, caller: u64, tag: &str, out: &mut EncounterOutcome) {
        let roster = self.roster();
        let home = roster.iter().find(|a| a.guid == caller).map(|a| a.position);
        let at = self
            .states()
            .iter()
            .find(|s| s.guid == caller)
            .map(|s| s.position);
        let mut called = Vec::new();
        for a in &roster {
            if !tags_of(a.guid, &self.behaviour_of, &self.arena.tagged).any(|x| x == tag) {
                continue;
            }
            let Some(s) = self.states_mut().iter_mut().find(|s| s.guid == a.guid) else {
                continue;
            };
            if !s.dormant || s.health == 0 {
                continue;
            }
            s.dormant = false;
            if let (Some(home), Some(at)) = (home, at) {
                s.position = [
                    at[0] + (a.position[0] - home[0]),
                    at[1],
                    at[2] + (a.position[2] - home[2]),
                ];
            }
            called.push((a.clone(), s.position));
        }
        self.show_raised(caller, &called, true, out);
    }

    /// The packets of helpers appearing: the caller's cast, then per helper its puff (when
    /// it rises from a mound), its spawn, its full bar and its rise.
    fn show_raised(
        &mut self,
        caller: u64,
        called: &[(Actor, [f32; 3])],
        mound: bool,
        out: &mut EncounterOutcome,
    ) {
        if called.is_empty() {
            return;
        }
        let visuals = self.brain_data().visuals.clone();
        out.replies
            .push(animation(caller, visuals.summon_animation, false));
        let mut guids = Vec::new();
        for (a, position) in called {
            if mound {
                out.replies
                    .push(composite_effect(a.guid, visuals.mound_effect, *position));
            }
            out.replies.push(self.spawn(a, *position, 0));
            out.replies
                .extend(enemy_health(a.guid, a.max_health, a.max_health));
            out.replies
                .push(animation(a.guid, visuals.spawn_animation, false));
            guids.push(a.guid);
        }
        out.replies.push(relevance(&guids, false));
    }

    /// `summon`: raise `count` helpers of `kind` from the fight's sleeping actors, or from
    /// helpers that fell long enough ago, and give them `tag`.
    #[allow(clippy::too_many_arguments)]
    fn summon(
        &mut self,
        caller: u64,
        kind: &str,
        count: u8,
        at: &Place,
        radius: f32,
        tag: &str,
        mound: bool,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let Some((from, home, leash, target)) = self.bearings(caller, false) else {
            return;
        };
        let reuse_after = self.brain_data().visuals.reuse_after_ms;
        let roster = self.roster();
        // Sleeping actors first, then the helpers that fell first. Only helpers (actors that
        // are not part of the goal) are ever raised twice.
        let mut pool: Vec<(u64, &Actor)> = Vec::new();
        for a in roster.iter().filter(|a| a.guid != caller) {
            let named = slug(&a.name) == kind
                || tags_of(a.guid, &self.behaviour_of, &self.arena.tagged).any(|x| x == kind);
            let Some(s) = self.states().iter().find(|s| s.guid == a.guid) else {
                continue;
            };
            if !named {
                continue;
            }
            if s.dormant && s.health > 0 {
                pool.push((0, a));
            } else if s.health == 0 && a.summon_below_percent.is_some() {
                if let Some(fell) = self.arena.fallen.get(&a.guid) {
                    if now_ms.saturating_sub(*fell) >= reuse_after {
                        pool.push((1 + *fell, a));
                    }
                }
            }
        }
        pool.sort_by_key(|(order, a)| (*order, a.guid));
        pool.truncate(usize::from(count));
        if pool.is_empty() {
            return;
        }
        let centre = match at {
            Place::Target => target.unwrap_or(from),
            Place::Ring | Place::Points => home,
            _ => from,
        };
        let spread = match at {
            Place::Ring | Place::Points if radius <= 0.0 => leash,
            _ if radius <= 0.0 => 1.5,
            _ => radius,
        };
        let ground = self.ground();
        let turn = self.arena.roll() * std::f32::consts::TAU;
        let n = pool.len() as f32;
        let mut called = Vec::new();
        for (i, (_, a)) in pool.iter().enumerate() {
            let angle = turn + std::f32::consts::TAU * i as f32 / n;
            let position = ground.snapped([
                centre[0] + spread * angle.cos(),
                centre[1],
                centre[2] + spread * angle.sin(),
            ]);
            let Some(s) = self.states_mut().iter_mut().find(|s| s.guid == a.guid) else {
                continue;
            };
            s.dormant = false;
            s.health = a.max_health;
            s.position = position;
            self.arena.fallen.remove(&a.guid);
            self.arena.forget(a.guid);
            self.arena.attacks.remove(&a.guid);
            if tag.is_empty() {
                self.arena.tagged.remove(&a.guid);
            } else {
                self.arena.tagged.insert(a.guid, vec![tag.to_string()]);
            }
            called.push(((*a).clone(), position));
        }
        self.show_raised(caller, &called, mound, out);
    }

    /// `become`: the sleeping actor of kind `definition` takes this enemy's place at
    /// `health_pct` of its own health. This enemy leaves the fight as fallen (it counts as
    /// down for the goal) without the kill being credited: the next form carries the fight.
    /// `false` when the fight has no such actor asleep.
    fn become_kind(
        &mut self,
        guid: u64,
        definition: &str,
        health_pct: u32,
        out: &mut EncounterOutcome,
    ) -> bool {
        let roster = self.roster();
        let Some(position) = self
            .states()
            .iter()
            .find(|s| s.guid == guid)
            .map(|s| s.position)
        else {
            return false;
        };
        let next = roster.iter().find(|a| {
            a.guid != guid
                && (slug(&a.name) == definition
                    || tags_of(a.guid, &self.behaviour_of, &self.arena.tagged)
                        .any(|x| x == definition))
                && self
                    .states()
                    .iter()
                    .any(|s| s.guid == a.guid && s.dormant && s.health > 0)
        });
        let Some(next) = next else {
            return false;
        };
        let health = (u64::from(next.max_health) * u64::from(health_pct) / 100).max(1) as u32;
        for s in self.states_mut().iter_mut() {
            if s.guid == next.guid {
                s.dormant = false;
                s.health = health;
                s.position = position;
            } else if s.guid == guid {
                s.dormant = true;
                s.health = 0;
            }
        }
        out.replies.push(remove(guid));
        if roster.iter().any(|a| a.guid == guid && a.boss) {
            out.replies.push(boss_display(guid, false));
        }
        self.arena.moving.remove(&guid);
        self.arena.forget(guid);
        out.replies.push(self.spawn(next, position, 0));
        out.replies
            .extend(enemy_health(next.guid, health, next.max_health));
        if next.boss {
            out.replies.push(boss_display(next.guid, true));
        }
        out.replies.push(relevance(&[next.guid], false));
        true
    }

    /// `wake` in the open world: idle neighbours with the tag take up the caller's target.
    fn wake_field(&mut self, caller: u64, tag: &str) {
        let Some((at, target)) = self
            .field_sim
            .enemies
            .iter()
            .find(|f| f.actor.guid == caller)
            .map(|f| (f.state.position, f.target))
        else {
            return;
        };
        let Some(target) = target else {
            return;
        };
        for f in &mut self.field_sim.enemies {
            let tagged = self
                .behaviour_of
                .get(&f.actor.guid)
                .is_some_and(|b| b.brain.tags.iter().any(|x| x == tag));
            // (R) 30 m: a pack is a camp, not the whole zone.
            if tagged
                && f.present
                && f.state.health > 0
                && f.motion == FieldMotion::Idle
                && distance(f.state.position, at) <= 30.0
            {
                f.threat.add(target, 0);
            }
        }
    }

    /// `heal`: a share of each receiver's own full health. Players are never healed by an
    /// enemy's act.
    fn heal(&mut self, guid: u64, who: &Who, pct: u32, field: bool, out: &mut EncounterOutcome) {
        if field {
            if let (Who::Own, Some(f)) = (who, self.field_enemy_mut(guid)) {
                let max = f.actor.max_health;
                if f.state.health > 0 && f.state.health < max {
                    let gain = (u64::from(max) * u64::from(pct) / 100) as u32;
                    let before = f.state.health;
                    f.state.health = before.saturating_add(gain).min(max);
                    let now = f.state.health;
                    out.replies.push(wire::hitpoints(
                        guid,
                        guid,
                        true,
                        max as i32,
                        now as i32,
                        (now - before) as i32,
                        false,
                    ));
                    out.replies.push(enemy_hp(guid, now, max));
                }
            }
            return;
        }
        let roster = self.roster();
        let receivers: Vec<&Actor> = roster
            .iter()
            .filter(|a| match who {
                Who::Own => a.guid == guid,
                Who::Boss => a.boss,
                Who::Tag(tag) => {
                    tags_of(a.guid, &self.behaviour_of, &self.arena.tagged).any(|x| x == tag)
                }
                Who::Target | Who::AllPlayers => false,
            })
            .collect();
        for a in receivers {
            let Some(s) = self.states_mut().iter_mut().find(|s| s.guid == a.guid) else {
                continue;
            };
            if s.health == 0 || s.dormant || s.health >= a.max_health {
                continue;
            }
            let gain = (u64::from(a.max_health) * u64::from(pct) / 100) as u32;
            let before = s.health;
            s.health = before.saturating_add(gain).min(a.max_health);
            let now = s.health;
            out.replies.push(wire::hitpoints(
                guid,
                a.guid,
                true,
                a.max_health as i32,
                now as i32,
                (now - before) as i32,
                false,
            ));
            out.replies.push(enemy_hp(a.guid, now, a.max_health));
        }
    }

    /// `status`: a stun on enemies uses the engine's stun. Anything on a player waits for
    /// the shared status table (P2) and is written to the fight log so it can be seen.
    #[allow(clippy::too_many_arguments)]
    fn status(
        &mut self,
        guid: u64,
        id: &str,
        to: &Who,
        ms: u32,
        now_ms: u64,
        field: bool,
        out: &mut EncounterOutcome,
    ) {
        let name = self
            .enemy(guid)
            .map(|(a, _)| a.name.clone())
            .unwrap_or_default();
        match to {
            Who::Target | Who::AllPlayers => {
                self.log_fight(now_ms, "status", &name, id, i64::from(ms));
            }
            _ if id != "stun" || field => {
                self.log_fight(now_ms, "status", &name, id, i64::from(ms));
            }
            _ => {
                let targets: Vec<u64> = self
                    .roster()
                    .iter()
                    .filter(|a| match to {
                        Who::Own => a.guid == guid,
                        Who::Boss => a.boss,
                        Who::Tag(tag) => tags_of(a.guid, &self.behaviour_of, &self.arena.tagged)
                            .any(|x| x == tag),
                        _ => false,
                    })
                    .map(|a| a.guid)
                    .collect();
                self.apply_stun(&targets, u64::from(ms), now_ms, out);
            }
        }
    }

    /// `set`: only the named fields change. Speed and disposition have their own packets;
    /// a new size, name or model is shown by spawning the actor again in place (R).
    fn set_flags(&mut self, guid: u64, set: &Flags, field: bool, out: &mut EncounterOutcome) {
        let flags = &mut self.memory(guid, field).flags;
        macro_rules! take {
            ($($name:ident),*) => {$(
                if set.$name.is_some() {
                    flags.$name = set.$name;
                }
            )*};
        }
        take!(invulnerable, untargetable, speed, scale, name_id, model_id, disposition);
        let flags = flags.clone();
        if let Some(speed) = set.speed {
            out.replies.push(wire::speed(guid, speed));
        }
        if set.scale.is_some() || set.name_id.is_some() || set.model_id.is_some() {
            if let Some((actor, s)) = self.enemy(guid) {
                let (mut shown, position, health) = (actor.clone(), s.position, s.health);
                shown.scale = flags.scale.or(shown.scale);
                shown.name_id = flags.name_id.unwrap_or(shown.name_id);
                shown.model_id = flags.model_id.unwrap_or(shown.model_id);
                out.replies.push(remove(guid));
                out.replies
                    .push(self.spawn(&shown, position, flags.disposition.unwrap_or(0)));
                out.replies.extend(enemy_health(guid, health, shown.max_health));
                if shown.boss {
                    out.replies.push(boss_display(guid, true));
                }
                out.replies.push(relevance(&[guid], false));
            }
        } else if let Some(value) = set.disposition {
            out.replies.push(wire::disposition(guid, value));
        }
    }
}
