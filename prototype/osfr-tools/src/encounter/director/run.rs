//! One fight in progress, as a pure state machine (combat design sections 10 and 11): it reads
//! facts (`on_fact`) and the clock (`tick`) and answers with what the engine must do. No
//! packets and no engine types, so a whole fight replays in a test from a list of facts.
use super::fight::*;
use super::*;

/// Where one roster actor stands in the run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Life {
    /// Not in the fight yet: a later wave, or dormant until woken.
    #[default]
    Waiting,
    Alive,
    Dead,
    /// Freed, yielded or left: gone without being defeated.
    Gone,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Goal {
    pub state: GoalState,
    pub done: u32,
    pub of: u32,
    /// When it became active.
    pub since: u64,
    /// Progress a `credit` action added.
    credited: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct WaveRun {
    /// Firings so far and when the last one was.
    fired: u32,
    last: u64,
    /// The trigger held on the last look (it fires on the change).
    held: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuleRun {
    fired: u32,
    ready_at: u64,
    held: bool,
    /// `every`: when it is next due.
    next: Option<u64>,
}

/// What happened since the last look, for the triggers that fire on an event.
#[derive(Clone, Debug, PartialEq)]
enum Event {
    Signal(String),
    Died(usize),
    Talked(usize),
    Room(String),
}

/// The points one finished run is worth, row by row (`Score` holds the values).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScoreCard {
    pub enemies: u32,
    pub enemy_points: u32,
    /// Seconds from the start of the fight to its end (the Time Bonus row's clock).
    pub seconds: u32,
    pub time_bonus: u32,
    pub knockouts_left: u32,
    pub knockouts_allowed: u32,
    pub knockout_points: u32,
    pub goals: u32,
    pub goal_points: u32,
    pub bonus_goals: u32,
    pub bonus_points: u32,
    pub total: u32,
}

impl ScoreCard {
    /// The points of a card whose counts are filled in. Fixed by the live results cards: 300 an
    /// enemy, 100,000 less 500 a second after the first, 25,000 for knockouts unused (Seaside
    /// Swoopers: 17 = 5100, 3:07 = 7000, 3/3 = 25,000, total 42,600; video archive, evidence-
    /// enemies.md). (R) the penalty per knockout used and the goal award. A lost or unfinished
    /// run keeps what was done and earns neither bonus.
    pub fn scored(mut self, s: &Score, won: bool) -> Self {
        let late = self.seconds.saturating_sub(s.quick_time_s);
        let used = self.knockouts_allowed.saturating_sub(self.knockouts_left);
        self.enemy_points = self.enemies.saturating_mul(s.per_enemy);
        self.goal_points = self.goals.saturating_mul(s.per_objective);
        self.bonus_points = self.bonus_goals.saturating_mul(s.per_objective);
        (self.time_bonus, self.knockout_points) = if won {
            (
                (s.quick_value).saturating_sub(late.saturating_mul(s.quick_loss_per_s)),
                (s.per_knockout_left).saturating_sub(used.saturating_mul(s.knockout_penalty)),
            )
        } else {
            (0, 0)
        };
        self.total = [
            self.enemy_points,
            self.time_bonus,
            self.knockout_points,
            self.goal_points,
            self.bonus_points,
        ]
        .into_iter()
        .fold(0, u32::saturating_add);
        self
    }
}

/// A run as it is saved with the character. Every field has a default, so an older save loads;
/// the clock is left out of the comparison that decides when to write.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Saved {
    fight: u32,
    elapsed_ms: u64,
    /// 0 waiting, 1 alive, 2 dead, 3 gone, per roster slot.
    lives: Vec<u8>,
    knockouts: u32,
    kills: u32,
    /// (0 waiting, 1 active, 2 done, 3 failed; credited) per goal.
    goals: Vec<(u8, u32)>,
    waves: Vec<u32>,
    rules: Vec<u32>,
    exit_open: bool,
    phase: u8,
    collected: BTreeMap<String, u32>,
}

impl PartialEq for Saved {
    fn eq(&self, other: &Self) -> bool {
        let key = |s: &Self| {
            (
                s.fight,
                s.lives.clone(),
                s.knockouts,
                s.kills,
                s.goals.clone(),
                s.waves.clone(),
                s.rules.clone(),
                s.exit_open,
                s.phase,
                s.collected.clone(),
            )
        };
        key(self) == key(other)
    }
}

/// What the engine side keeps with a run, so a group's shared run carries it too.
#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::encounter) struct Bind {
    /// The door and the allies clients have been sent.
    pub shown: BTreeSet<u64>,
    /// Named clocks (ward pressure per goal, the last drive).
    pub clocks: BTreeMap<String, u64>,
    /// Where each escorted actor stands, and the route point it walks to.
    pub spots: BTreeMap<usize, ([f32; 3], usize)>,
    /// A player has stood clear of the open exit door (walking back in then leaves).
    pub clear_of_exit: bool,
}

/// One fight in progress: actors, waves, goal progress, knockouts, script memory.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Run {
    pub fight: u32,
    pub(in crate::encounter) def: Arc<Fight>,
    /// A `field_battles.json` or cave row with no fight file: the engine's own single goal
    /// stays in charge and this run only reports.
    pub(in crate::encounter) legacy: bool,
    /// Party size when the run began; enemy health follows it (`scale::enemy_health`).
    pub group: u8,
    pub(in crate::encounter) started: Option<u64>,
    pub(in crate::encounter) now: u64,
    pub(in crate::encounter) lives: Vec<Life>,
    /// Health of each roster actor, 0..=100, as the engine last reported it.
    pub(in crate::encounter) health: Vec<u8>,
    waves: Vec<WaveRun>,
    pub(in crate::encounter) goals: Vec<Goal>,
    rules: Vec<RuleRun>,
    /// Knockouts the group has taken.
    pub knockouts: u32,
    /// Enemies defeated.
    pub kills: u32,
    pub(in crate::encounter) exit_open: bool,
    /// `Some(true)` won, `Some(false)` lost.
    pub(in crate::encounter) ended: Option<bool>,
    pub(in crate::encounter) ended_at: u64,
    phase: u8,
    /// Acts waiting for their time: (due, target tag, action).
    queue: Vec<(u64, Option<String>, Action)>,
    events: Vec<Event>,
    /// Answers to facts, handed over on the next tick.
    carried: Vec<DirectorAct>,
    collected: BTreeMap<String, u32>,
    /// Points and rooms a player stands in.
    inside: BTreeSet<String>,
    seed: u64,
    pub(in crate::encounter) bind: Bind,
}

impl Run {
    pub(super) fn new(fight: &Fight, party: &Party) -> Self {
        let def = Arc::new(fight.clone());
        let mut run = Self {
            fight: fight.id,
            group: party.members.len().clamp(1, usize::from(u8::MAX)) as u8,
            lives: def
                .roster
                .iter()
                .map(|slot| {
                    if slot.start {
                        Life::Alive
                    } else {
                        Life::Waiting
                    }
                })
                .collect(),
            health: vec![100; def.roster.len()],
            waves: vec![WaveRun::default(); def.waves.len()],
            goals: vec![Goal::default(); def.objectives.len()],
            rules: vec![RuleRun::default(); def.script.len()],
            seed: 0x5eed_f167_0000_0000 ^ u64::from(fight.id),
            def,
            ..Self::default()
        };
        for i in 0..run.goals.len() {
            let (done, of, _) = run.progress(i);
            run.goals[i].done = done;
            run.goals[i].of = of;
        }
        run
    }

    /// A run of a row that has no fight file; its clock times the results card.
    pub(in crate::encounter) fn legacy(fight: u32, now_ms: u64) -> Self {
        Self {
            fight,
            legacy: true,
            group: 1,
            started: Some(now_ms),
            ..Self::default()
        }
    }

    /// The run as it is saved.
    pub fn save(&self) -> Saved {
        Saved {
            fight: self.fight,
            elapsed_ms: self.elapsed(),
            lives: self.lives.iter().map(|life| *life as u8).collect(),
            knockouts: self.knockouts,
            kills: self.kills,
            goals: (self.goals.iter())
                .map(|goal| (goal.state as u8, goal.credited))
                .collect(),
            waves: self.waves.iter().map(|wave| wave.fired).collect(),
            rules: self.rules.iter().map(|rule| rule.fired).collect(),
            exit_open: self.exit_open,
            phase: self.phase,
            collected: self.collected.clone(),
        }
    }

    /// A run of `fight` picked up from `saved` at `now_ms`; `None` when the save is of another
    /// fight or another shape (the file changed since). Clocks that were running start again:
    /// a wave's "or after" and a timed goal count from the return.
    pub fn restore(fight: &Fight, saved: &Saved, now_ms: u64) -> Option<Self> {
        let mut run = Self::new(fight, &Party::default());
        if saved.fight != fight.id
            || saved.lives.len() != run.lives.len()
            || saved.goals.len() != run.goals.len()
            || saved.waves.len() != run.waves.len()
            || saved.rules.len() != run.rules.len()
        {
            return None;
        }
        run.now = now_ms;
        run.started = Some(now_ms.saturating_sub(saved.elapsed_ms));
        for (life, saved) in run.lives.iter_mut().zip(&saved.lives) {
            *life = match saved {
                0 => Life::Waiting,
                1 => Life::Alive,
                2 => Life::Dead,
                _ => Life::Gone,
            };
        }
        for (goal, (state, credited)) in run.goals.iter_mut().zip(&saved.goals) {
            goal.state = match state {
                0 => GoalState::Waiting,
                1 => GoalState::Active,
                2 => GoalState::Done,
                _ => GoalState::Failed,
            };
            goal.credited = *credited;
            goal.since = now_ms;
        }
        for (wave, fired) in run.waves.iter_mut().zip(&saved.waves) {
            wave.fired = *fired;
            wave.last = now_ms;
        }
        run.knockouts = saved.knockouts;
        run.kills = saved.kills;
        run.exit_open = saved.exit_open;
        run.phase = saved.phase;
        run.collected.clone_from(&saved.collected);
        for i in 0..run.goals.len() {
            let (done, of, _) = run.progress(i);
            run.goals[i].done = if run.goals[i].state == GoalState::Done {
                of
            } else {
                done
            };
            run.goals[i].of = of;
        }
        // What holds now has already been answered: only a new change fires a rule or a wave.
        let def = run.def.clone();
        for (w, wave) in def.waves.iter().enumerate() {
            run.waves[w].held = run.holds(&wave.when);
        }
        for (r, rule) in def.script.iter().enumerate() {
            run.rules[r].fired = saved.rules[r];
            run.rules[r].held = run.holds(&rule.when);
        }
        Some(run)
    }

    pub fn definition(&self) -> &Fight {
        &self.def
    }

    /// `Some(true)` once won, `Some(false)` once lost.
    pub fn outcome(&self) -> Option<bool> {
        self.ended
    }

    pub fn exit_is_open(&self) -> bool {
        self.exit_open
    }

    /// A player stands in the named point or room.
    pub fn is_inside(&self, place: &str) -> bool {
        self.inside.contains(place)
    }

    pub fn knockouts_left(&self) -> u32 {
        self.def.knockouts().saturating_sub(self.knockouts)
    }

    /// (state, done, of) of goal `id`.
    pub fn goal(&self, id: &str) -> Option<(GoalState, u32, u32)> {
        let i = self.def.objectives.iter().position(|o| o.id == id)?;
        let goal = &self.goals[i];
        Some((goal.state, goal.done, goal.of))
    }

    pub fn life(&self, guid: u64) -> Option<Life> {
        self.slot_of(guid).map(|i| self.lives[i])
    }

    /// Living actors that carry `tag`.
    pub fn alive(&self, tag: &str) -> u32 {
        self.tagged(tag)
            .filter(|&i| self.lives[i] == Life::Alive)
            .count() as u32
    }

    /// Living actors per tag, as an enemy's rules read them (`brain::View::tags_alive`).
    pub fn tags_alive(&self) -> BTreeMap<String, u32> {
        let mut counts = BTreeMap::new();
        for (slot, life) in self.def.roster.iter().zip(&self.lives) {
            if *life == Life::Alive {
                for tag in &slot.tags {
                    *counts.entry(tag.clone()).or_insert(0) += 1;
                }
            }
        }
        counts
    }

    /// The engine's latest reading of one actor's health, 0..=100.
    pub fn set_health(&mut self, guid: u64, pct: u32) {
        if let Some(i) = self.slot_of(guid) {
            self.health[i] = pct.min(100) as u8;
        }
    }

    pub(in crate::encounter) fn slot_of(&self, guid: u64) -> Option<usize> {
        let i = guid.checked_sub(self.def.guid(None))?.checked_sub(1)? as usize;
        (i < self.def.roster.len() && self.def.guid(Some(i)) == guid).then_some(i)
    }

    fn tagged<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = usize> + 'a {
        self.def
            .roster
            .iter()
            .enumerate()
            .filter(move |(_, slot)| slot.tags.iter().any(|t| t == tag))
            .map(|(i, _)| i)
    }

    fn elapsed(&self) -> u64 {
        self.now.saturating_sub(self.started.unwrap_or(self.now))
    }

    fn roll(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.seed >> 40) as f32) / ((1_u64 << 24) as f32)
    }

    /// The score of the run as it stands at `now` (R: the arithmetic; the values are data).
    pub fn score(&self, now_ms: u64) -> ScoreCard {
        let end = if self.ended.is_some() {
            self.ended_at
        } else {
            now_ms
        };
        let count = |bonus: bool| {
            self.def
                .objectives
                .iter()
                .zip(&self.goals)
                .filter(|(o, g)| o.bonus == bonus && g.state == GoalState::Done)
                .count() as u32
        };
        ScoreCard {
            enemies: self.kills,
            seconds: (end.saturating_sub(self.started.unwrap_or(end)) / 1000) as u32,
            knockouts_left: self.knockouts_left(),
            knockouts_allowed: self.def.knockouts(),
            goals: count(false),
            bonus_goals: count(true),
            ..ScoreCard::default()
        }
        .scored(&self.def.rules.score, self.ended == Some(true))
    }

    /// (done, of, failed) of goal `i` as the run stands.
    fn progress(&self, i: usize) -> (u32, u32, bool) {
        let goal = &self.def.objectives[i];
        let state = &self.goals[i];
        let set = self.def.targets(&goal.target);
        let down = |pool: &[usize]| {
            pool.iter()
                .filter(|&&s| self.lives[s] == Life::Dead)
                .count() as u32
        };
        let active_for = self.now.saturating_sub(state.since);
        let lost = |key: &Option<String>| {
            set.iter().any(|&s| {
                Some(&self.def.roster[s].key) == key.as_ref() && self.lives[s] == Life::Dead
            })
        };
        let (done, of, failed) = match goal.kind {
            GoalKind::Defeat | GoalKind::Destroy => {
                let gone = set
                    .iter()
                    .filter(|&&s| matches!(self.lives[s], Life::Dead | Life::Gone))
                    .count() as u32;
                (gone, goal.count.unwrap_or(set.len() as u32), false)
            }
            GoalKind::Free => {
                let freed = set.iter().filter(|&&s| self.lives[s] == Life::Gone).count() as u32;
                let of = goal.count.unwrap_or(set.len() as u32);
                // A captive lost for good can leave too few to free.
                let left = set.iter().filter(|&&s| self.lives[s] != Life::Dead).count() as u32;
                (freed, of, left < of)
            }
            GoalKind::Collect => {
                let have = goal
                    .target
                    .pickup
                    .as_ref()
                    .and_then(|p| self.collected.get(p))
                    .copied()
                    .unwrap_or(0);
                (have, goal.count.unwrap_or(1), false)
            }
            GoalKind::Reach => {
                let there = goal
                    .target
                    .point
                    .as_ref()
                    .is_some_and(|p| self.inside.contains(p));
                (
                    u32::from(there && state.state == GoalState::Active),
                    1,
                    false,
                )
            }
            GoalKind::Survive => (
                u32::from(state.state == GoalState::Active && active_for >= goal.ms),
                1,
                false,
            ),
            GoalKind::Defend => {
                let failed = lost(&goal.target.actor);
                match goal.until.as_ref().map(|u| &u.defeated) {
                    Some(until) => {
                        let pool = self.def.targets(&Target {
                            tag: until.tag.clone(),
                            definition: until.definition.clone(),
                            ..Target::default()
                        });
                        (down(&pool).min(until.count), until.count, failed)
                    }
                    None => (
                        u32::from(state.state == GoalState::Active && active_for >= goal.ms),
                        1,
                        failed,
                    ),
                }
            }
            GoalKind::Escort => {
                let arrived = goal
                    .target
                    .actor
                    .as_ref()
                    .is_some_and(|a| self.inside.contains(&format!("escort:{a}")));
                (u32::from(arrived), 1, lost(&goal.target.actor))
            }
        };
        ((done + state.credited).min(of), of, failed)
    }

    fn fact(&self, i: usize) -> FightFact {
        FightFact::Goal {
            fight: self.fight,
            goal: self.def.objectives[i].id.clone(),
            done: self.goals[i].done,
            of: self.goals[i].of,
        }
    }

    fn end(&mut self, won: bool, out: &mut Vec<DirectorAct>) {
        if self.ended.is_none() {
            self.ended = Some(won);
            self.ended_at = self.now;
            out.push(if won {
                DirectorAct::Win
            } else {
                DirectorAct::Lose
            });
        }
    }

    fn open_exit(&mut self, out: &mut Vec<DirectorAct>) {
        if self.exit_open || self.ended.is_some() {
            return;
        }
        if self.def.instanced() {
            self.exit_open = true;
            out.push(DirectorAct::OpenExit);
        } else {
            // The open world has no door: the last goal ends the fight.
            self.end(true, out);
        }
    }

    /// Bring the primary chain and the bonus goals up to date.
    fn goals_step(&mut self, out: &mut Vec<DirectorAct>) {
        let def = self.def.clone();
        loop {
            let mut changed = false;
            // The first primary goal not yet done is the active one.
            let next = def
                .objectives
                .iter()
                .zip(&self.goals)
                .position(|(o, g)| !o.bonus && g.state != GoalState::Done);
            for (i, goal) in def.objectives.iter().enumerate() {
                let before = self.goals[i].clone();
                if before.state == GoalState::Waiting && (goal.bonus || next == Some(i)) {
                    self.goals[i].state = GoalState::Active;
                    self.goals[i].since = self.now;
                }
                if matches!(self.goals[i].state, GoalState::Done | GoalState::Failed) {
                    continue;
                }
                let (done, of, failed) = self.progress(i);
                self.goals[i].done = done;
                self.goals[i].of = of;
                if self.goals[i].state == GoalState::Active {
                    if failed {
                        self.goals[i].state = GoalState::Failed;
                    } else if done >= of {
                        self.goals[i].state = GoalState::Done;
                    }
                }
                if self.goals[i] != before {
                    changed = true;
                    if self.goals[i].state != GoalState::Waiting {
                        out.push(DirectorAct::Fact(self.fact(i)));
                    }
                    if self.goals[i].state == GoalState::Failed && !goal.bonus {
                        self.end(false, out);
                    }
                }
            }
            if !changed {
                break;
            }
        }
        if def
            .objectives
            .iter()
            .zip(&self.goals)
            .all(|(o, g)| o.bonus || g.state == GoalState::Done)
        {
            self.open_exit(out);
        }
    }

    /// Whether a trigger holds now (state triggers) or just happened (event triggers).
    fn holds(&self, when: &Trigger) -> bool {
        match when {
            Trigger::Manual | Trigger::Every { .. } => false,
            Trigger::FightStarted { after_ms } => {
                self.started.is_some() && self.elapsed() >= *after_ms
            }
            Trigger::Clock { ms } => self.started.is_some() && self.elapsed() >= *ms,
            Trigger::WaveLeft { wave, left } => {
                let fired = self
                    .def
                    .waves
                    .iter()
                    .position(|w| w.id == *wave)
                    .is_some_and(|w| self.waves[w].fired > 0);
                fired && self.alive(&format!("wave:{wave}")) <= *left
            }
            Trigger::ObjectiveDone { id } => self
                .goal(id)
                .is_some_and(|(state, _, _)| state == GoalState::Done),
            Trigger::PlayerWithin { point, .. } => self.inside.contains(point),
            Trigger::TagDead { tag, left } => {
                // Counted only once every actor with the tag has had its chance to appear.
                self.tagged(tag).all(|i| self.lives[i] != Life::Waiting) && self.alive(tag) <= *left
            }
            Trigger::HealthBelow { tag, pct } => self
                .tagged(tag)
                .any(|i| self.lives[i] == Life::Alive && u32::from(self.health[i]) < *pct),
            Trigger::RoomEntered { id } => self.events.contains(&Event::Room(id.clone())),
            Trigger::Signal { name } => self.events.contains(&Event::Signal(name.clone())),
            Trigger::Died { tag } => self
                .tagged(tag)
                .any(|i| self.events.contains(&Event::Died(i))),
            Trigger::Talked { tag } => self
                .tagged(tag)
                .any(|i| self.events.contains(&Event::Talked(i))),
        }
    }

    fn fire_wave(&mut self, w: usize, out: &mut Vec<DirectorAct>) {
        let firing = self.waves[w].fired;
        self.waves[w].fired += 1;
        self.waves[w].last = self.now;
        for (i, slot) in self.def.roster.iter().enumerate() {
            if slot.wave == Some((w, firing)) && self.lives[i] == Life::Waiting {
                self.lives[i] = Life::Alive;
            }
        }
        out.push(DirectorAct::Wave(self.def.waves[w].id.clone()));
    }

    fn waves_step(&mut self, out: &mut Vec<DirectorAct>) {
        let def = self.def.clone();
        for (w, wave) in def.waves.iter().enumerate() {
            let firings = wave.repeat.as_ref().map_or(1, |r| r.times);
            if self.waves[w].fired >= firings {
                continue;
            }
            if self.waves[w].fired > 0 {
                // A repeating wave comes again on its own clock.
                let every = wave.repeat.as_ref().map_or(u64::MAX, |r| r.every_ms);
                if self.now >= self.waves[w].last.saturating_add(every) {
                    self.fire_wave(w, out);
                }
                continue;
            }
            let held = self.holds(&wave.when);
            let edge = held && !self.waves[w].held;
            self.waves[w].held = held;
            // "Or after": counted from the wave it waits on, else from the start.
            let armed = match &wave.when {
                Trigger::WaveLeft { wave: on, .. } => def
                    .waves
                    .iter()
                    .position(|other| other.id == *on)
                    .filter(|&o| self.waves[o].fired > 0)
                    .map(|o| self.waves[o].last),
                _ => self.started,
            };
            let late = wave
                .or_after_ms
                .zip(armed)
                .is_some_and(|(ms, since)| self.now >= since.saturating_add(ms));
            if edge || late {
                self.fire_wave(w, out);
            }
        }
    }

    fn rules_step(&mut self) {
        let def = self.def.clone();
        for (r, rule) in def.script.iter().enumerate() {
            let fired = match &rule.when {
                Trigger::Every { ms, first_ms } => {
                    let start = self.started.unwrap_or(self.now);
                    let due = *self.rules[r].next.get_or_insert(start + first_ms);
                    let hit = self.now >= due;
                    if hit {
                        self.rules[r].next = Some(due.max(self.now.saturating_sub(*ms)) + ms);
                    }
                    hit
                }
                when => {
                    let held = self.holds(when);
                    let event = matches!(
                        when,
                        Trigger::RoomEntered { .. }
                            | Trigger::Signal { .. }
                            | Trigger::Died { .. }
                            | Trigger::Talked { .. }
                    );
                    let edge = held && (event || !self.rules[r].held);
                    self.rules[r].held = held;
                    edge
                }
            };
            let memory = &self.rules[r];
            if !fired
                || (rule.once && memory.fired > 0)
                || (rule.max > 0 && memory.fired >= rule.max)
                || self.now < memory.ready_at
            {
                continue;
            }
            let passes = rule.conditions.iter().all(|cond| match cond {
                Cond::TagAlive { tag, min, max } => (*min..=*max).contains(&self.alive(tag)),
                Cond::Phase(n) => self.phase == *n,
                Cond::Chance(p) => self.roll() < *p,
            });
            if !passes {
                continue;
            }
            self.rules[r].fired += 1;
            self.rules[r].ready_at = self.now + rule.cooldown_ms;
            let mut due = self.now;
            for step in &rule.steps {
                due += step.wait_ms;
                self.queue.push((due, step.on.clone(), step.action.clone()));
            }
        }
    }

    /// Carry out every queued act that is due. Acts the fight itself performs and that an
    /// enemy's rule could also perform go to `plain`; the rest to `out`.
    fn drain(&mut self, out: &mut Vec<DirectorAct>, mut plain: Option<&mut Vec<Act>>) -> bool {
        let mut did = false;
        let mut i = 0;
        while i < self.queue.len() {
            if self.queue[i].0 > self.now {
                i += 1;
                continue;
            }
            let (_, on, action) = self.queue.remove(i);
            did = true;
            match action {
                Action::SpawnWave(id) => {
                    if let Some(w) = self.def.waves.iter().position(|w| w.id == id) {
                        let firings = self.def.waves[w].repeat.as_ref().map_or(1, |r| r.times);
                        if self.waves[w].fired < firings {
                            self.fire_wave(w, out);
                        }
                    }
                }
                Action::OpenExit => self.open_exit(out),
                Action::Fail => self.end(false, out),
                Action::Win => self.end(true, out),
                Action::SetObjective { id, state } => {
                    if let Some(g) = self.def.objectives.iter().position(|o| o.id == id) {
                        if self.goals[g].state != state {
                            self.goals[g].state = state;
                            if state == GoalState::Done {
                                self.goals[g].done = self.goals[g].of;
                            }
                            if state == GoalState::Active {
                                self.goals[g].since = self.now;
                            }
                            out.push(DirectorAct::Fact(self.fact(g)));
                            if state == GoalState::Failed && !self.def.objectives[g].bonus {
                                self.end(false, out);
                            }
                        }
                    }
                }
                Action::Credit { id, n } => {
                    if let Some(g) = self.def.objectives.iter().position(|o| o.id == id) {
                        self.goals[g].credited = self.goals[g].credited.saturating_add(n);
                    }
                }
                Action::DropPowerUps { table, .. } => {
                    out.push(DirectorAct::Actor {
                        guid: 0,
                        act: Act::Drop(table),
                    });
                }
                Action::Brain(act) => {
                    match &act {
                        Act::Signal(name) => self.events.push(Event::Signal(name.clone())),
                        Act::Phase(n) => self.phase = *n,
                        Act::Wake(tag) => {
                            let woken: Vec<usize> = self
                                .tagged(tag)
                                .filter(|&s| self.lives[s] == Life::Waiting)
                                .collect();
                            for s in woken {
                                self.lives[s] = Life::Alive;
                            }
                        }
                        _ => {}
                    }
                    // A phase and a wake by tag change the run itself; nobody acts them out.
                    if on.is_none() && matches!(act, Act::Phase(_) | Act::Wake(_)) {
                        continue;
                    }
                    match (on, &mut plain) {
                        (Some(tag), _) => {
                            // A captive just freed may still say its thanks as it leaves.
                            let says = matches!(act, Act::Say(_));
                            let targets: Vec<u64> = self
                                .tagged(&tag)
                                .filter(|&s| {
                                    self.lives[s] == Life::Alive
                                        || (says && self.lives[s] == Life::Gone)
                                })
                                .map(|s| self.def.guid(Some(s)))
                                .collect();
                            out.extend(targets.into_iter().map(|guid| DirectorAct::Actor {
                                guid,
                                act: act.clone(),
                            }));
                        }
                        (None, Some(plain)) => plain.push(act),
                        (None, None) => out.push(match act {
                            Act::Signal(name) => DirectorAct::Signal(name),
                            act => DirectorAct::Actor { guid: 0, act },
                        }),
                    }
                }
            }
        }
        did
    }

    /// Settle the run at the current time: waves, goals, rules and due acts, until nothing
    /// more follows.
    fn settle(&mut self, out: &mut Vec<DirectorAct>, mut plain: Option<&mut Vec<Act>>) {
        if self.legacy || self.started.is_none() {
            return;
        }
        for _ in 0..8 {
            if self.ended.is_some() {
                break;
            }
            let before = (out.len(), self.queue.len());
            self.waves_step(out);
            self.goals_step(out);
            self.rules_step();
            self.events.clear();
            let did = self.drain(out, plain.as_mut().map(|acts| &mut **acts));
            if !did && before == (out.len(), self.queue.len()) {
                break;
            }
        }
        self.events.clear();
        let limit = u64::from(self.def.rules.time_limit_s) * 1000;
        if limit > 0 && self.elapsed() >= limit {
            self.end(false, out);
        }
    }

    pub(super) fn fact_in(&mut self, fact: &Signal, plain: &mut Vec<Act>) {
        if self.legacy || self.ended.is_some() {
            return;
        }
        let def = self.def.clone();
        let mut out = Vec::new();
        match fact {
            Signal::Named(name) => {
                if let Some(place) = name.strip_prefix("arrived:") {
                    self.inside.insert(format!("escort:{place}"));
                }
                self.events.push(Event::Signal(name.clone()));
            }
            Signal::Died { guid, .. } => {
                if let Some(i) = self.slot_of(*guid) {
                    if self.lives[i] == Life::Alive {
                        self.lives[i] = Life::Dead;
                        self.health[i] = 0;
                        if def.roster[i].side == Side::Enemy {
                            self.kills += 1;
                        }
                        self.events.push(Event::Died(i));
                    }
                }
            }
            Signal::Talked { guid } => {
                if *guid == def.guid(None) {
                    if self.exit_open {
                        self.end(true, &mut out);
                    }
                } else if let Some(i) = self.slot_of(*guid) {
                    self.events.push(Event::Talked(i));
                    let slot = &def.roster[i];
                    let captive = def
                        .objectives
                        .iter()
                        .any(|o| o.kind == GoalKind::Free && def.targets(&o.target).contains(&i));
                    let guarded = slot
                        .guarded_by
                        .as_ref()
                        .is_some_and(|tag| self.alive(tag) > 0);
                    if captive && self.lives[i] == Life::Alive && !guarded {
                        self.lives[i] = Life::Gone;
                        out.push(DirectorAct::Fact(FightFact::Freed {
                            kind: slot.definition_id,
                            guid: *guid,
                        }));
                    }
                }
            }
            Signal::PlayerDown { .. } => {
                self.knockouts += 1;
                if self.knockouts > def.knockouts() {
                    self.end(false, &mut out);
                }
            }
            Signal::Reached { place } => {
                if place == "exit" {
                    if self.exit_open {
                        self.end(true, &mut out);
                    }
                } else if let Some(left) = place.strip_prefix("left:") {
                    self.inside.remove(left);
                } else if self.inside.insert(place.clone()) {
                    self.events.push(Event::Room(place.clone()));
                }
            }
            Signal::Collected { pickup } => {
                *self.collected.entry(pickup.clone()).or_insert(0) += 1;
            }
        }
        self.settle(&mut out, Some(plain));
        // What the engine must still do waits for the next tick, in order.
        self.carried.extend(out);
    }

    pub(super) fn tick_to(&mut self, now_ms: u64, out: &mut Vec<DirectorAct>) {
        if self.legacy {
            return;
        }
        self.now = self.now.max(now_ms);
        self.started.get_or_insert(self.now);
        out.append(&mut self.carried);
        if self.ended.is_none() {
            self.settle(out, None);
        }
    }
}
