//! The behaviour language as loaded data: one stance, a list of when / if / do rules and the
//! attacks they name (combat design 8.1, 8.2), with the load-time limits of 8.3. Everything
//! here is checked once at load so `step` never has to refuse anything.
use super::{Act, Flags, Move, Place, Stance, TargetRule, Who};
use crate::encounter::wire::{How, Line};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Load-time limits (design 8.3); Studio checks the same numbers.
pub struct Limits;

impl Limits {
    pub const RULES: usize = 32;
    pub const ACTIONS: usize = 8;
    pub const SUMMONS: u8 = 16;
    /// The decision cadence: no timer may be shorter.
    pub const TIMER_MS: u64 = 250;
    /// The attack id every behaviour has: the plain swing built from the row's own reach and
    /// attack time. `attacks.json` may not redefine it.
    pub const MELEE: &'static str = "melee";
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttackKind {
    /// Hits the target inside `reach`.
    #[default]
    Melee,
    /// Thrown at where the target stands; lands `travel_ms` later on whoever is within `radius`.
    Lob,
    /// An area around the enemy (or its target with `"at": "target"`), after `windup_ms`.
    Burst,
}

/// The bigger hit an attack sometimes lands (design 5.3: 1.5 times, effect 5622).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Special {
    pub chance: f32,
    pub hit: f32,
    #[serde(default)]
    pub effect: i32,
}

/// A status the attack leaves on whoever it hits.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rider {
    pub status: String,
    pub ms: u32,
    #[serde(default)]
    pub tick_hit: f32,
}

/// One enemy attack from `attacks.json`. `hit` multiplies the enemy's own normal hit.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Attack {
    pub kind: AttackKind,
    pub reach: f32,
    pub range: f32,
    pub radius: f32,
    pub hit: f32,
    pub interval_ms: u64,
    pub windup_ms: u64,
    pub travel_ms: u64,
    pub animation: i32,
    pub effect: i32,
    pub projectile_effect: i32,
    pub land_effect: i32,
    /// A burst's centre: `self` (the default) or `target`.
    pub at: Option<String>,
    pub special: Option<Special>,
    pub riders: Vec<Rider>,
    pub note: Option<String>,
}

impl Default for Attack {
    fn default() -> Self {
        Self {
            kind: AttackKind::Melee,
            reach: 2.5,
            range: 0.0,
            radius: 0.0,
            hit: 1.0,
            interval_ms: 2300,
            windup_ms: 0,
            travel_ms: 0,
            animation: 1001,
            effect: 0,
            projectile_effect: 0,
            land_effect: 0,
            at: None,
            special: None,
            riders: Vec::new(),
            note: None,
        }
    }
}

impl Attack {
    /// The plain swing of a row: its own reach and attack time.
    pub fn melee(reach: f32, interval_ms: u64) -> Self {
        Self {
            reach,
            interval_ms,
            ..Self::default()
        }
    }

    /// A burst centred on the target instead of the enemy.
    pub fn at_target(&self) -> bool {
        self.at.as_deref() == Some("target")
    }

    /// How close the target must be before the enemy starts this attack.
    pub fn reach(&self) -> f32 {
        match self.kind {
            AttackKind::Melee => self.reach,
            AttackKind::Lob => self.range,
            AttackKind::Burst if self.at_target() => self.range.max(self.radius),
            AttackKind::Burst => self.radius,
        }
    }

    fn check(&self, id: &str) -> Result<(), String> {
        let lengths = [self.reach, self.range, self.radius, self.hit];
        if lengths.iter().any(|v| !v.is_finite() || !(0.0..=1000.0).contains(v)) {
            return Err(format!("attack {id}: a distance or hit is outside 0..1000"));
        }
        if self.reach() <= 0.0 {
            return Err(format!("attack {id}: it can reach nobody"));
        }
        if self.kind != AttackKind::Melee && self.radius <= 0.0 {
            return Err(format!("attack {id}: a lob or burst needs a radius"));
        }
        if self.interval_ms < Limits::TIMER_MS {
            return Err(format!("attack {id}: interval_ms is under {}", Limits::TIMER_MS));
        }
        if self.windup_ms > 60_000 || self.travel_ms > 60_000 {
            return Err(format!("attack {id}: wind-up or travel is over a minute"));
        }
        if !matches!(self.at.as_deref(), None | Some("self" | "target")) {
            return Err(format!("attack {id}: at must be self or target"));
        }
        if let Some(special) = &self.special {
            if !(0.0..=1.0).contains(&special.chance) || !(0.0..=1000.0).contains(&special.hit) {
                return Err(format!("attack {id}: special chance or hit is out of range"));
            }
        }
        if self.riders.len() > 4 || self.riders.iter().any(|r| r.status.is_empty()) {
            return Err(format!("attack {id}: at most 4 riders, each with a status"));
        }
        Ok(())
    }
}

/// Read the `attacks` object of `attacks.json`.
pub fn parse_attacks(attacks: &Value) -> Result<BTreeMap<String, Attack>, String> {
    let rows: BTreeMap<String, Attack> =
        serde_json::from_value(attacks.clone()).map_err(|e| format!("attacks: {e}"))?;
    if rows.len() > 512 {
        return Err("attacks: more than 512 rows".into());
    }
    for (id, attack) in &rows {
        if id == Limits::MELEE {
            return Err(format!("attacks: '{}' is built in", Limits::MELEE));
        }
        attack.check(id)?;
    }
    Ok(rows)
}

/// What a rule waits for (design 8.2): exactly one per rule.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    /// Once, that long after the enemy appears.
    Spawned {
        #[serde(default)]
        after_ms: u64,
    },
    /// When it first has a target.
    Engaged {},
    /// Every `count` hits taken.
    Hit {
        #[serde(default = "one")]
        count: u32,
    },
    /// Each time health crosses down through `pct`.
    HealthBelow { pct: u32 },
    /// On a timer while engaged.
    Every {
        ms: u64,
        #[serde(default)]
        first_ms: Option<u64>,
    },
    /// When the nearest player comes inside `m`.
    PlayerWithin { m: f32 },
    /// When the nearest player gets farther than `m`.
    PlayerBeyond { m: f32 },
    /// When a neighbour within `m` (with `tag`, when given) gets a target.
    AllyEngaged {
        #[serde(default = "join_radius")]
        m: f32,
        #[serde(default)]
        tag: Option<String>,
    },
    /// When the living actors with `tag` fall to `left`.
    TagDead {
        tag: String,
        #[serde(default)]
        left: u32,
    },
    /// When its target is knocked out.
    TargetDown {},
    /// At zero health, before removal.
    Died {},
    /// When a player interacts with it.
    Talked {},
    /// When a rule or the fight raised `name` (heard on the next decision).
    Signal { name: String },
}

fn one() -> u32 {
    1
}

fn join_radius() -> f32 {
    6.0
}

fn no_cap() -> u32 {
    u32::MAX
}

/// What must also hold for a rule to fire: all of a rule's conditions.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Cond {
    HealthBelow(u32),
    HealthAbove(u32),
    TagAlive {
        tag: String,
        #[serde(default)]
        min: u32,
        #[serde(default = "no_cap")]
        max: u32,
    },
    Chance(f32),
    Phase(u8),
    TargetWithin(f32),
    TargetBeyond(f32),
    Night(bool),
}

/// One rule: when the trigger fires and every condition holds, do the acts in order, each
/// after its own `wait_ms` (waits add up along the list).
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub id: String,
    pub when: Trigger,
    pub conds: Vec<Cond>,
    pub acts: Vec<(u64, Act)>,
    pub once: bool,
    pub cooldown_ms: u64,
    pub max: u32,
}

/// A loaded behaviour: its stance, its rules and the attacks they name.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub stance: Stance,
    pub rules: Vec<Rule>,
    pub attacks: BTreeMap<String, Attack>,
}

impl Default for Program {
    /// Chase and swing with the engine's original constants.
    fn default() -> Self {
        Self {
            stance: Stance {
                movement: Move::Chase,
                attack: Some(Limits::MELEE.into()),
                leash: true,
            },
            rules: Vec::new(),
            attacks: BTreeMap::from([(Limits::MELEE.to_string(), Attack::melee(2.5, 1000))]),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleJson {
    #[serde(default)]
    id: Option<String>,
    when: Trigger,
    #[serde(default, rename = "if")]
    conds: Vec<Cond>,
    #[serde(rename = "do")]
    acts: Vec<Value>,
    #[serde(default)]
    once: bool,
    #[serde(default)]
    cooldown_ms: u64,
    #[serde(default)]
    max: u32,
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StanceJson {
    #[serde(rename = "move")]
    movement: String,
    min: Option<f32>,
    max: Option<f32>,
    ms: Option<u32>,
    speed: Option<f32>,
    tag: Option<String>,
    m: Option<f32>,
    id: Option<String>,
    points: Option<String>,
    #[serde(default)]
    attack: Option<String>,
    leash: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SayJson {
    text_id: Option<i32>,
    text: Option<String>,
    how: Option<String>,
    #[serde(default)]
    text_ids: Vec<i32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FlagsJson {
    invulnerable: Option<bool>,
    untargetable: Option<bool>,
    speed: Option<f32>,
    scale: Option<f32>,
    name_id: Option<i32>,
    model_id: Option<i32>,
    disposition: Option<i32>,
}

fn field<T: DeserializeOwned>(body: &Value, key: &str, what: &str) -> Result<Option<T>, String> {
    match body.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => serde_json::from_value(v.clone())
            .map(Some)
            .map_err(|e| format!("{what}: {key} {e}")),
    }
}

fn need<T: DeserializeOwned>(body: &Value, key: &str, what: &str) -> Result<T, String> {
    field(body, key, what)?.ok_or(format!("{what}: {key} is missing"))
}

/// Refuse keys the action does not have, so a typo is an error and not a silent default.
fn only(body: &Value, keys: &[&str], what: &str) -> Result<(), String> {
    match body {
        Value::Null => Ok(()),
        Value::Object(map) => match map.keys().find(|k| !keys.contains(&k.as_str())) {
            Some(extra) => Err(format!("{what}: unknown key '{extra}'")),
            None => Ok(()),
        },
        _ => Err(format!("{what}: must be an object")),
    }
}

fn distance(v: f32, what: &str) -> Result<f32, String> {
    if v.is_finite() && (0.0..=1000.0).contains(&v) {
        Ok(v)
    } else {
        Err(format!("{what}: {v} must be 0..1000"))
    }
}

fn percent(v: u32, what: &str) -> Result<u32, String> {
    if (1..=100).contains(&v) {
        Ok(v)
    } else {
        Err(format!("{what}: {v} must be 1..100"))
    }
}

fn timer(ms: u64, what: &str) -> Result<u64, String> {
    if ms >= Limits::TIMER_MS && ms <= 3_600_000 {
        Ok(ms)
    } else {
        Err(format!(
            "{what}: {ms} ms must be {}..3600000",
            Limits::TIMER_MS
        ))
    }
}

fn place(name: &str, what: &str) -> Result<Place, String> {
    Ok(match name {
        "self" | "own" => Place::Own,
        "target" => Place::Target,
        "away" => Place::Away,
        "points" => Place::Points,
        "ring" => Place::Ring,
        "behind_allies" => Place::BehindAllies,
        other => match other.strip_prefix("point:") {
            Some(point) if !point.is_empty() => Place::Point(point.into()),
            _ => return Err(format!("{what}: unknown place '{other}'")),
        },
    })
}

fn who(name: &str, what: &str) -> Result<Who, String> {
    Ok(match name {
        "self" | "own" => Who::Own,
        "target" => Who::Target,
        "all_players" => Who::AllPlayers,
        "boss" => Who::Boss,
        other => match other.strip_prefix("tag:") {
            Some(tag) if !tag.is_empty() => Who::Tag(tag.into()),
            _ => return Err(format!("{what}: unknown receiver '{other}'")),
        },
    })
}

fn stance(value: &Value, what: &str) -> Result<Stance, String> {
    let s: StanceJson =
        serde_json::from_value(value.clone()).map_err(|e| format!("{what}: stance {e}"))?;
    let movement = match s.movement.as_str() {
        "hold" => Move::Hold,
        "chase" => Move::Chase,
        "keep_away" => {
            let min = distance(s.min.ok_or(format!("{what}: keep_away needs min"))?, what)?;
            let max = distance(s.max.ok_or(format!("{what}: keep_away needs max"))?, what)?;
            if min >= max {
                return Err(format!("{what}: keep_away min must be under max"));
            }
            Move::KeepAway { min, max }
        }
        "flee" => Move::Flee {
            ms: timer(u64::from(s.ms.unwrap_or(4000)), what)? as u32,
            speed: distance(s.speed.unwrap_or(3.5), what)?,
        },
        "behind" => Move::Behind {
            tag: s.tag.clone().ok_or(format!("{what}: behind needs tag"))?,
            m: distance(s.m.unwrap_or(6.0), what)?,
        },
        "route" => Move::Route(s.id.clone().ok_or(format!("{what}: route needs id"))?),
        "perch" => Move::Perch(
            s.points
                .clone()
                .ok_or(format!("{what}: perch needs points"))?,
        ),
        "roam" => Move::Roam(distance(s.m.unwrap_or(4.0), what)?),
        "home" => Move::Home,
        other => return Err(format!("{what}: unknown move '{other}'")),
    };
    Ok(Stance {
        movement,
        attack: s.attack,
        leash: s.leash.unwrap_or(true),
    })
}

fn line(value: &Value, what: &str) -> Result<Line, String> {
    let s: SayJson = serde_json::from_value(value.clone()).map_err(|e| format!("{what}: {e}"))?;
    let how = match s.how.as_deref() {
        None | Some("bubble") => How::Bubble,
        Some("red") => How::Red,
        Some("system") => How::System,
        Some(other) => return Err(format!("{what}: unknown how '{other}'")),
    };
    if s.text_id.is_none() && s.text_ids.is_empty() && s.text.as_deref().is_none_or(str::is_empty) {
        return Err(format!("{what}: say needs text_id or text"));
    }
    if s.text.as_ref().is_some_and(|t| t.len() > 200) {
        return Err(format!("{what}: say text is over 200 bytes"));
    }
    if s.text_ids.len() > 16 || s.text_ids.iter().any(|id| *id <= 0) {
        return Err(format!("{what}: say text_ids holds 1..=16 positive ids"));
    }
    Ok(Line {
        text_id: s.text_id,
        text: s.text,
        how,
        text_ids: s.text_ids,
    })
}

/// One `do` entry: `"yield"`, or an object with exactly one action key. Returns its wait
/// and the act.
fn action(value: &Value, what: &str) -> Result<(u64, Act), String> {
    let (name, mut body) = match value {
        Value::String(name) => (name.clone(), Value::Null),
        Value::Object(map) if map.len() == 1 => {
            let (name, body) = map.iter().next().ok_or(format!("{what}: empty action"))?;
            (name.clone(), body.clone())
        }
        _ => return Err(format!("{what}: an action is one name or one {{name: body}}")),
    };
    let what = &format!("{what} {name}");
    let wait = match body.as_object_mut().and_then(|map| map.remove("wait_ms")) {
        None => 0,
        Some(v) => match v.as_u64() {
            Some(0) => 0,
            Some(ms) => timer(ms, what)?,
            None => return Err(format!("{what}: wait_ms must be a whole number")),
        },
    };
    let text = |key: &str| need::<String>(&body, key, what);
    let act = match name.as_str() {
        "stance" => Act::Stance(stance(&body, what)?),
        "attack" => {
            only(&body, &["id"], what)?;
            Act::Attack(text("id")?)
        }
        "say" => Act::Say(line(&body, what)?),
        "telegraph" => {
            only(&body, &["say", "ms", "attack"], what)?;
            Act::Telegraph {
                say: line(body.get("say").unwrap_or(&Value::Null), what)?,
                ms: timer(need::<u64>(&body, "ms", what)?, what)? as u32,
                attack: text("attack")?,
            }
        }
        "summon" => {
            only(&body, &["kind", "count", "at", "radius", "tag", "visual"], what)?;
            let count = field::<u8>(&body, "count", what)?.unwrap_or(1);
            if count == 0 || count > Limits::SUMMONS {
                return Err(format!("{what}: count must be 1..{}", Limits::SUMMONS));
            }
            let visual = field::<String>(&body, "visual", what)?;
            if !matches!(visual.as_deref(), None | Some("mound")) {
                return Err(format!("{what}: the only visual is mound"));
            }
            Act::Summon {
                kind: text("kind")?,
                count,
                at: place(&field::<String>(&body, "at", what)?.unwrap_or("self".into()), what)?,
                radius: distance(field(&body, "radius", what)?.unwrap_or(0.0), what)?,
                tag: field(&body, "tag", what)?.unwrap_or_default(),
                mound: visual.is_some(),
            }
        }
        "wake" => {
            only(&body, &["tag"], what)?;
            Act::Wake(text("tag")?)
        }
        "relocate" => {
            only(&body, &["to", "min", "max", "effect"], what)?;
            let min = distance(field(&body, "min", what)?.unwrap_or(0.0), what)?;
            let max = distance(field(&body, "max", what)?.unwrap_or(min), what)?;
            if min > max {
                return Err(format!("{what}: min is over max"));
            }
            Act::Relocate {
                to: place(&field::<String>(&body, "to", what)?.unwrap_or("away".into()), what)?,
                min,
                max,
                effect: field(&body, "effect", what)?,
            }
        }
        "heal" => {
            only(&body, &["who", "pct"], what)?;
            Act::Heal {
                who: who(&field::<String>(&body, "who", what)?.unwrap_or("self".into()), what)?,
                pct: percent(need(&body, "pct", what)?, what)?,
            }
        }
        "status" => {
            only(&body, &["id", "to", "ms"], what)?;
            Act::Status {
                id: text("id")?,
                to: who(&field::<String>(&body, "to", what)?.unwrap_or("target".into()), what)?,
                ms: timer(need::<u64>(&body, "ms", what)?, what)? as u32,
            }
        }
        "set" => {
            let f: FlagsJson =
                serde_json::from_value(body.clone()).map_err(|e| format!("{what}: {e}"))?;
            for v in [f.speed, f.scale].into_iter().flatten() {
                if !v.is_finite() || !(0.0..=50.0).contains(&v) {
                    return Err(format!("{what}: speed and scale must be 0..50"));
                }
            }
            Act::Set(Flags {
                invulnerable: f.invulnerable,
                untargetable: f.untargetable,
                speed: f.speed,
                scale: f.scale,
                name_id: f.name_id,
                model_id: f.model_id,
                disposition: f.disposition,
            })
        }
        "phase" => {
            only(&body, &["n"], what)?;
            Act::Phase(need(&body, "n", what)?)
        }
        "become" => {
            only(&body, &["definition", "health_pct"], what)?;
            Act::Become {
                definition: text("definition")?,
                health_pct: percent(field(&body, "health_pct", what)?.unwrap_or(100), what)?,
            }
        }
        "signal" => {
            only(&body, &["name"], what)?;
            Act::Signal(text("name")?)
        }
        "drop" => {
            only(&body, &["table", "pickup"], what)?;
            let table = field::<String>(&body, "table", what)?;
            let pickup = field::<String>(&body, "pickup", what)?;
            Act::Drop(
                table
                    .or(pickup)
                    .ok_or(format!("{what}: needs table or pickup"))?,
            )
        }
        "yield" => {
            only(&body, &[], what)?;
            Act::Yield
        }
        "leave" => {
            only(&body, &["ms"], what)?;
            Act::Leave(field(&body, "ms", what)?.unwrap_or(0))
        }
        "revive" => {
            only(&body, &["pct", "after_ms"], what)?;
            Act::Revive {
                pct: percent(field(&body, "pct", what)?.unwrap_or(100), what)?,
                after_ms: field(&body, "after_ms", what)?.unwrap_or(0),
            }
        }
        "target" => {
            only(&body, &["rule"], what)?;
            Act::Target(match text("rule")?.as_str() {
                "attacker" => TargetRule::Attacker,
                "nearest" => TargetRule::Nearest,
                "lowest_health" => TargetRule::LowestHealth,
                "taunter" => TargetRule::Taunter,
                other => return Err(format!("{what}: unknown rule '{other}'")),
            })
        }
        other => return Err(format!("{what}: unknown action '{other}'")),
    };
    Ok((wait, act))
}

fn check_trigger(when: &Trigger, what: &str) -> Result<(), String> {
    match when {
        Trigger::Hit { count } if *count == 0 => Err(format!("{what}: hit count must be 1 or more")),
        Trigger::HealthBelow { pct } => percent(*pct, what).map(drop),
        Trigger::Every { ms, .. } => timer(*ms, what).map(drop),
        Trigger::PlayerWithin { m } | Trigger::PlayerBeyond { m } | Trigger::AllyEngaged { m, .. } => {
            distance(*m, what).map(drop)
        }
        Trigger::TagDead { tag, .. } if tag.is_empty() => Err(format!("{what}: tag_dead needs a tag")),
        Trigger::Signal { name } if name.is_empty() => Err(format!("{what}: signal needs a name")),
        _ => Ok(()),
    }
}

fn check_cond(cond: &Cond, what: &str) -> Result<(), String> {
    match cond {
        Cond::HealthBelow(pct) | Cond::HealthAbove(pct) if *pct > 100 => {
            Err(format!("{what}: health percent {pct} is over 100"))
        }
        Cond::Chance(p) if !(0.0..=1.0).contains(p) => Err(format!("{what}: chance {p} must be 0..1")),
        Cond::TargetWithin(m) | Cond::TargetBeyond(m) => distance(*m, what).map(drop),
        Cond::TagAlive { tag, min, max } if tag.is_empty() || min > max => {
            Err(format!("{what}: tag_alive needs a tag and min at most max"))
        }
        _ => Ok(()),
    }
}

impl Rule {
    /// Something stops this rule from firing on every decision.
    fn braked(&self) -> bool {
        self.once || self.max > 0 || self.cooldown_ms >= Limits::TIMER_MS
    }
}

impl Program {
    /// Read `{"stance": {...}, "rules": [...]}` with the default stance and swing.
    pub fn from_json(behaviour: &Value, attacks: &BTreeMap<String, Attack>) -> Result<Self, String> {
        only(behaviour, &["stance", "rules"], "behaviour")?;
        let rules = match behaviour.get("rules") {
            None => Vec::new(),
            Some(Value::Array(rules)) => rules.clone(),
            Some(_) => return Err("behaviour: rules must be a list".into()),
        };
        let base = Self::default();
        Self::build(
            behaviour.get("stance"),
            &rules,
            base.stance,
            Attack::melee(2.5, 1000),
            attacks,
        )
    }

    /// Build and check a program. `fallback` is the stance when `stance` is absent and
    /// `melee` the row's own swing; only the attacks the program names are kept.
    pub fn build(
        stance_json: Option<&Value>,
        rules: &[Value],
        fallback: Stance,
        melee: Attack,
        attacks: &BTreeMap<String, Attack>,
    ) -> Result<Self, String> {
        if rules.len() > Limits::RULES {
            return Err(format!("more than {} rules", Limits::RULES));
        }
        let stance = match stance_json {
            None | Some(Value::Null) => fallback,
            Some(value) => stance(value, "stance")?,
        };
        let mut built = Vec::with_capacity(rules.len());
        let mut ids = BTreeSet::new();
        for (index, value) in rules.iter().enumerate() {
            let rule: RuleJson = serde_json::from_value(value.clone())
                .map_err(|e| format!("rule {}: {e}", index + 1))?;
            let id = rule.id.unwrap_or(format!("rule{}", index + 1));
            let what = &format!("rule {id}");
            if id.is_empty() || !ids.insert(id.clone()) {
                return Err(format!("{what}: ids must be unique and not empty"));
            }
            check_trigger(&rule.when, what)?;
            if rule.conds.len() > 8 {
                return Err(format!("{what}: more than 8 conditions"));
            }
            for cond in &rule.conds {
                check_cond(cond, what)?;
            }
            if rule.acts.is_empty() || rule.acts.len() > Limits::ACTIONS {
                return Err(format!("{what}: needs 1..{} actions", Limits::ACTIONS));
            }
            if rule.cooldown_ms != 0 {
                timer(rule.cooldown_ms, what)?;
            }
            let acts = rule
                .acts
                .iter()
                .map(|a| action(a, what))
                .collect::<Result<Vec<_>, _>>()?;
            built.push(Rule {
                id,
                when: rule.when,
                conds: rule.conds,
                acts,
                once: rule.once,
                cooldown_ms: rule.cooldown_ms,
                max: rule.max,
            });
        }
        let mut program = Self {
            stance,
            rules: built,
            attacks: BTreeMap::new(),
        };
        program.loops()?;
        // Keep the attacks it names; an unknown name is refused.
        let mut named: Vec<&str> = program.stance.attack.as_deref().into_iter().collect();
        for (_, act) in program.rules.iter().flat_map(|r| &r.acts) {
            match act {
                Act::Stance(s) => named.extend(s.attack.as_deref()),
                Act::Attack(id) => named.push(id),
                Act::Telegraph { attack, .. } => named.push(attack),
                _ => {}
            }
        }
        let mut kept = BTreeMap::new();
        for id in named {
            let attack = if id == Limits::MELEE {
                melee.clone()
            } else {
                attacks
                    .get(id)
                    .cloned()
                    .ok_or(format!("unknown attack '{id}'"))?
            };
            kept.insert(id.to_string(), attack);
        }
        program.attacks = kept;
        Ok(program)
    }

    /// Refuse rules that could fire on every decision for ever: a chain of rules that hear a
    /// signal and raise one, coming back to where it started, with no `once`, `max` or
    /// cooldown anywhere on the way.
    fn loops(&self) -> Result<(), String> {
        let mut edges: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::new();
        for rule in self.rules.iter().filter(|r| !r.braked()) {
            let Trigger::Signal { name } = &rule.when else {
                continue;
            };
            for (_, act) in &rule.acts {
                if let Act::Signal(raised) = act {
                    edges
                        .entry(name.as_str())
                        .or_default()
                        .push((raised.as_str(), rule.id.as_str()));
                }
            }
        }
        for start in edges.keys() {
            let mut seen = BTreeSet::new();
            let mut open = vec![*start];
            while let Some(at) = open.pop() {
                for (next, rule) in edges.get(at).map(Vec::as_slice).unwrap_or_default() {
                    if next == start {
                        return Err(format!(
                            "rule {rule}: signal '{start}' loops back to itself with no cooldown, once or max"
                        ));
                    }
                    if seen.insert(*next) {
                        open.push(next);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn attack(&self, id: &str) -> Option<&Attack> {
        self.attacks.get(id)
    }

    /// Some rule can fire for an enemy that has no target yet, so idle enemies must be
    /// stepped too.
    pub fn wakes_idle(&self) -> bool {
        self.rules.iter().any(|r| {
            matches!(
                r.when,
                Trigger::Spawned { .. }
                    | Trigger::PlayerWithin { .. }
                    | Trigger::PlayerBeyond { .. }
                    | Trigger::AllyEngaged { .. }
                    | Trigger::Talked {}
                    | Trigger::Signal { .. }
            )
        })
    }
}
