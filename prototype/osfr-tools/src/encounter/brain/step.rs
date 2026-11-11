//! One decision of one enemy: what it remembers between decisions and the pure function that
//! turns a view into acts (combat design 8.3). No packets and no engine types.
use super::program::{Cond, Rule, Trigger};
use super::{Act, Flags, Program, Stance, TargetRule, View};

#[derive(Clone, Debug, Default, PartialEq)]
struct RuleMemory {
    fired: u32,
    /// Cooldown end.
    ready_at: u64,
    /// `hit`: hits counted toward the next firing.
    hits: u32,
    /// `every`: when the timer next rings.
    due: Option<u64>,
    /// Edge triggers: whether the watched thing held at the last decision.
    level: Option<bool>,
    /// `tag_dead`: the tag has had more living actors than `left` at some point.
    seen_above: bool,
    /// `spawned`: already answered.
    spawned: bool,
}

/// What one enemy (or one fight script) remembers between steps: phase, rule fire counts and
/// next-allowed times, the hit counter, crossed thresholds, the stance and pending timed acts.
/// Events (spawned, engaged, hit, died, talked) are written here by the owner before a step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memory {
    rules: Vec<RuleMemory>,
    stance: Option<Stance>,
    phase: u8,
    /// Acts waiting for their time (`wait_ms`, the attack of a telegraph).
    pending: Vec<(u64, Act)>,
    seed: u64,
    spawned_at: Option<u64>,
    engaged_at: Option<u64>,
    /// Health when the memory was started for an enemy already in a fight (a relog):
    /// thresholds above it count as crossed long ago and do not fire again.
    resumed_pct: Option<u32>,
    now_engaged: bool,
    new_hits: u32,
    now_died: bool,
    now_talked: bool,
    now_target_down: bool,
    /// Neighbours that got a target since the last step: distance and their tags.
    allies: Vec<(f32, Vec<String>)>,
    /// What `set` changed on this enemy (the engine reads it; `step` does not).
    pub flags: Flags,
    /// An attack ordered by a rule or the fight, used at the enemy's next move.
    pub ordered: Option<String>,
    /// The last `target` rule.
    pub target_rule: Option<TargetRule>,
    /// It gave up (`yield`): it stands as an NPC and decides nothing more.
    pub yielded: bool,
    /// `revive`: it lies down until then, deciding nothing and taking no damage.
    pub down_until: u64,
}

impl Memory {
    /// A memory whose chance rolls follow `seed`, so the same fight replays the same way.
    pub fn seeded(seed: u64) -> Self {
        Self {
            seed,
            ..Self::default()
        }
    }

    /// Start the memory of an enemy that is already at `health_pct` (the run was resumed):
    /// `health_below` rules it has passed stay quiet.
    pub fn resumed(seed: u64, health_pct: u32) -> Self {
        Self {
            seed,
            resumed_pct: Some(health_pct),
            ..Self::default()
        }
    }

    /// The enemy appeared (kept from the first call only).
    pub fn spawned(&mut self, now_ms: u64) {
        self.spawned_at.get_or_insert(now_ms);
    }

    /// The enemy got a target; only the first call since it last had none counts.
    pub fn engaged(&mut self, now_ms: u64) {
        if self.engaged_at.is_none() {
            self.engaged_at = Some(now_ms);
            self.now_engaged = true;
        }
    }

    pub fn is_engaged(&self) -> bool {
        self.engaged_at.is_some()
    }

    /// A player hit landed on it.
    pub fn hit(&mut self) {
        self.new_hits = self.new_hits.saturating_add(1);
    }

    /// It reached zero health.
    pub fn died(&mut self) {
        self.now_died = true;
    }

    /// A player interacted with it.
    pub fn talked(&mut self) {
        self.now_talked = true;
    }

    /// Its target was knocked out.
    pub fn target_down(&mut self) {
        self.now_target_down = true;
    }

    /// A neighbour `distance` metres away, carrying `tags`, got a target.
    pub fn ally_engaged(&mut self, distance: f32, tags: &[String]) {
        if self.allies.len() < 8 {
            self.allies.push((distance, tags.to_vec()));
        }
    }

    /// The stance in force: the last one a rule set, else the program's.
    pub fn stance<'a>(&'a self, program: &'a Program) -> &'a Stance {
        self.stance.as_ref().unwrap_or(&program.stance)
    }

    pub fn set_stance(&mut self, stance: Stance) {
        self.stance = Some(stance);
    }

    pub fn phase(&self) -> u8 {
        self.phase
    }

    pub fn set_phase(&mut self, phase: u8) {
        self.phase = phase;
    }

    /// Queue an act for `due_ms` (the fight's telegraphs use this).
    pub fn defer(&mut self, due_ms: u64, act: Act) {
        if self.pending.len() < 64 {
            self.pending.push((due_ms, act));
        }
    }

    /// (R) Same generator as the combat crit rolls; 0 to 1.
    fn roll(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.seed >> 40) as f32 / 16_777_216.0
    }
}

/// Apply what an act changes in the memory and hand it on. A telegraph becomes its line now
/// and its attack after the wait.
fn emit(m: &mut Memory, act: Act, now: u64, out: &mut Vec<Act>) {
    match &act {
        Act::Stance(stance) => m.stance = Some(stance.clone()),
        Act::Phase(n) => m.phase = *n,
        Act::Telegraph { say, ms, attack } => {
            out.push(Act::Say(say.clone()));
            m.defer(now.saturating_add(u64::from(*ms)), Act::Attack(attack.clone()));
            return;
        }
        _ => {}
    }
    out.push(act);
}

/// A level that fires only on the step it starts to hold.
fn rising(memory: &mut RuleMemory, holds: bool, first: bool) -> bool {
    let before = memory.level.replace(holds).unwrap_or(first);
    holds && !before
}

struct Events<'a> {
    engaged: bool,
    hits: u32,
    died: bool,
    talked: bool,
    target_down: bool,
    allies: &'a [(f32, Vec<String>)],
    spawned_at: u64,
    engaged_at: Option<u64>,
    resumed_pct: Option<u32>,
}

fn triggered(rule: &Rule, memory: &mut RuleMemory, e: &Events, v: &View) -> bool {
    let now = v.now_ms;
    match &rule.when {
        Trigger::Spawned { after_ms } => {
            let due = !memory.spawned && now >= e.spawned_at.saturating_add(*after_ms);
            memory.spawned |= due;
            due
        }
        Trigger::Engaged {} => e.engaged,
        Trigger::Hit { count } => {
            // Hits during the cooldown are not saved up for later.
            if now < memory.ready_at {
                memory.hits = 0;
                return false;
            }
            memory.hits = memory.hits.saturating_add(e.hits);
            let enough = memory.hits >= *count;
            if enough {
                memory.hits -= count;
            }
            enough
        }
        Trigger::HealthBelow { pct } => {
            let passed = e.resumed_pct.is_some_and(|at| at < *pct);
            rising(memory, v.health_pct < *pct, passed)
        }
        Trigger::Every { ms, first_ms } => {
            let Some(start) = e.engaged_at else {
                memory.due = None;
                return false;
            };
            let due = *memory
                .due
                .get_or_insert(start.saturating_add(first_ms.unwrap_or(*ms)));
            if now >= due {
                memory.due = Some(now.saturating_add(*ms));
            }
            now >= due
        }
        Trigger::PlayerWithin { m } => {
            rising(memory, v.nearest_player.is_some_and(|d| d <= *m), false)
        }
        Trigger::PlayerBeyond { m } => {
            let beyond = v.nearest_player.is_some_and(|d| d > *m);
            // Starting out far away is not "getting farther".
            rising(memory, beyond, beyond)
        }
        Trigger::AllyEngaged { m, tag } => e.allies.iter().any(|(distance, tags)| {
            distance <= m && tag.as_ref().is_none_or(|tag| tags.contains(tag))
        }),
        Trigger::TagDead { tag, left } => {
            let alive = v.tags_alive.get(tag).copied().unwrap_or(0);
            if alive > *left {
                memory.seen_above = true;
                memory.level = Some(false);
                return false;
            }
            // Never fires for a group that was never there (a pack still asleep).
            memory.seen_above && rising(memory, true, false)
        }
        Trigger::TargetDown {} => e.target_down,
        Trigger::Died {} => e.died,
        Trigger::Talked {} => e.talked,
        Trigger::Signal { name } => v.signals.iter().any(|s| s == name),
    }
}

fn holds(cond: &Cond, m: &mut Memory, v: &View) -> bool {
    match cond {
        Cond::HealthBelow(pct) => v.health_pct < *pct,
        Cond::HealthAbove(pct) => v.health_pct > *pct,
        Cond::TagAlive { tag, min, max } => {
            let alive = v.tags_alive.get(tag).copied().unwrap_or(0);
            (*min..=*max).contains(&alive)
        }
        Cond::Chance(p) => m.roll() < *p,
        Cond::Phase(n) => v.phase == *n,
        Cond::TargetWithin(metres) => v.target_distance.is_some_and(|d| d <= *metres),
        Cond::TargetBeyond(metres) => v.target_distance.is_some_and(|d| d > *metres),
        Cond::Night(night) => v.night == *night,
    }
}

pub(super) fn run(p: &Program, m: &mut Memory, v: &View, out: &mut Vec<Act>) {
    let now = v.now_ms;
    let spawned_at = *m.spawned_at.get_or_insert(now);
    if m.rules.len() != p.rules.len() {
        m.rules.resize(p.rules.len(), RuleMemory::default());
    }
    // Acts whose wait is over, oldest first.
    if m.pending.iter().any(|(due, _)| *due <= now) {
        let (mut due, later): (Vec<_>, Vec<_>) =
            m.pending.drain(..).partition(|(due, _)| *due <= now);
        m.pending = later;
        due.sort_by_key(|(due, _)| *due);
        for (_, act) in due {
            emit(m, act, now, out);
        }
    }
    let allies = std::mem::take(&mut m.allies);
    let events = Events {
        engaged: std::mem::take(&mut m.now_engaged),
        hits: std::mem::take(&mut m.new_hits),
        died: std::mem::take(&mut m.now_died),
        talked: std::mem::take(&mut m.now_talked),
        target_down: std::mem::take(&mut m.now_target_down),
        allies: &allies,
        spawned_at,
        engaged_at: m.engaged_at,
        resumed_pct: m.resumed_pct,
    };
    for (index, rule) in p.rules.iter().enumerate() {
        // The trigger is always looked at, so an edge passed during a cooldown is spent.
        if !triggered(rule, &mut m.rules[index], &events, v) {
            continue;
        }
        let memory = &m.rules[index];
        if rule.once && memory.fired > 0
            || rule.max > 0 && memory.fired >= rule.max
            || now < memory.ready_at
        {
            continue;
        }
        if !rule.conds.iter().all(|cond| holds(cond, m, v)) {
            continue;
        }
        let memory = &mut m.rules[index];
        memory.fired = memory.fired.saturating_add(1);
        memory.ready_at = now.saturating_add(rule.cooldown_ms);
        let mut wait = 0_u64;
        for (after, act) in &rule.acts {
            wait = wait.saturating_add(*after);
            if wait == 0 {
                emit(m, act.clone(), now, out);
            } else {
                m.defer(now.saturating_add(wait), act.clone());
            }
        }
    }
}
