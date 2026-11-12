//! One fight file (`data/content/fights/<id>.json`, combat design section 10): its shape, the
//! load-time checks and the roster every run of it uses. Nothing here touches the engine: the
//! runtime (`run.rs`) and the engine side (`bind.rs`) both read a checked `Fight`.
use super::super::brain::{Act, Flags, Move, Place as ActPlace, Stance, TargetRule, Who};
use super::super::wire::{How, Line};
use super::super::*;
use serde_json::Value;

/// Load limits (design 8.3 and 10). A file past one of them is refused, never trimmed.
pub const MAX_FILE_BYTES: u64 = 256 * 1024;
pub const MAX_FIGHTS: usize = 512;
pub const MAX_ACTORS: usize = 64;
pub const MAX_WAVES: usize = 32;
pub const MAX_OBJECTIVES: usize = 16;
pub const MAX_RULES: usize = 32;
pub const MAX_ACTIONS: usize = 8;
pub const MIN_TIMER_MS: u64 = 250;
/// Knockouts a group may take before the fight is lost (wiki combat guide: 10 in a wandering
/// battle, 15 in a dungeon; a fight's `rules.knockouts` overrides it).
pub const BATTLE_KNOCKOUTS: u32 = 10;
pub const DUNGEON_KNOCKOUTS: u32 = 15;
/// The exit door as the live server sent it (capture p12 t=546306.7: name id 4826 "Exit",
/// model 846, disposition 0 then SetDisposition 1).
pub const EXIT_NAME_ID: i32 = 4826;
pub const EXIT_MODEL_ID: i32 = 846;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
    Battle,
    Dungeon,
    WorldBoss,
    QuestFight,
    Duel,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    #[default]
    Enemy,
    /// A ward, an escort or a captive: players cannot hit it.
    Ally,
}

/// A fixed spot, or the name of a list in `points` (a wave walks the list).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Spot {
    At([f32; 3]),
    Points(String),
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Starter {
    pub definition: String,
    pub at: [f32; 3],
    pub roam: f32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Interact {
    pub npc_guid: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Entry {
    pub starter: Option<Starter>,
    /// The starter, the start screen and the wheel preview are the row of the same id in
    /// `field_battles.json`; this file replaces that row's roster and goal when a run begins.
    pub battle_row: bool,
    pub games_row: u32,
    pub min_combat_level: u32,
    pub quest_ids: Vec<u32>,
    pub interact: Option<Interact>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Room {
    pub id: String,
    /// min x, y, z then max x, y, z.
    #[serde(rename = "box")]
    pub bounds: [f32; 6],
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Place {
    pub zone: u32,
    pub arena: String,
    pub sky: String,
    pub geometry_id: i32,
    pub arrive: [f32; 3],
    pub heading: [f32; 3],
    /// Where the exit door stands; absent, the door stands at `arrive`.
    pub exit_door: Option<[f32; 3]>,
    pub rooms: Vec<Room>,
    pub open_world: bool,
    pub centre: [f32; 3],
    pub radius: f32,
}

/// The values of the encounter settings record (packet 62) as the client names them
/// (field-name list at 0x01050DB0): EncounterNpcKOValue, EncounterKnockoutRewardValue,
/// EncounterKnockoutPenalty, EncounterQuickCompletionValue / Loss / Time,
/// EncounterObjectiveMultiplier. (R) how the server turned them into a score.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Score {
    pub per_enemy: u32,
    pub per_knockout_left: u32,
    pub knockout_penalty: u32,
    pub quick_value: u32,
    pub quick_loss_per_s: u32,
    pub quick_time_s: u32,
    pub objective_multiplier: f32,
    /// Points per finished goal (the 45/3 award the engine has always sent).
    pub per_objective: u32,
}

/// The score values a packet-62 record carries, in its order: quick-completion time, value and
/// loss, then per enemy, knockout reward and penalty (C16's battle record: 1, 100,000, 500, 300,
/// 25,000, 5,000 for a row with no fight file).
impl From<&crate::encounter::wire::Settings> for Score {
    fn from(s: &crate::encounter::wire::Settings) -> Self {
        let [time, value, loss, enemy, reward, penalty] = s.ints.map(|v| v.max(0) as u32);
        Self {
            per_enemy: enemy,
            per_knockout_left: reward,
            knockout_penalty: penalty,
            quick_value: value,
            quick_loss_per_s: loss,
            quick_time_s: time,
            objective_multiplier: s.float_b,
            ..Self::default()
        }
    }
}

impl Default for Score {
    fn default() -> Self {
        Self {
            per_enemy: 0,
            per_knockout_left: 0,
            knockout_penalty: 0,
            quick_value: 0,
            quick_loss_per_s: 0,
            quick_time_s: 0,
            objective_multiplier: 1.0,
            per_objective: 5000,
        }
    }
}

/// Packets no native run has proven yet; each stays off until one has.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct WireSwitches {
    /// Send packet 62 with this fight's score values (captured layout, C16).
    pub settings: bool,
    /// Send 35/18 with the knockouts used and allowed (R: the badge's source).
    pub lives: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Rules {
    pub knockouts: Option<u32>,
    pub group_max: u32,
    pub time_limit_s: u32,
    pub retry: bool,
    pub solo: bool,
    pub can_use_abilities: bool,
    pub can_use_consumables: bool,
    pub score: Score,
    /// A held power-up every player enters with (the seasonal bosses' borrowed weapon).
    pub power_up: Option<String>,
    /// (R) Seconds a knocked-out player waits before standing up inside the fight.
    pub recover_s: u32,
    pub wire: WireSwitches,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            knockouts: None,
            group_max: MAX_MEMBERS as u32,
            time_limit_s: 0,
            retry: true,
            solo: false,
            can_use_abilities: true,
            can_use_consumables: true,
            score: Score::default(),
            power_up: None,
            recover_s: 10,
            wire: WireSwitches::default(),
        }
    }
}

/// An enemy kind written in the fight file itself. A name not found here is looked up in
/// `enemies.json`.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct KindRow {
    pub definition_id: i32,
    pub name: String,
    pub name_id: i32,
    pub model_id: i32,
    pub texture_alias: String,
    pub rank: Rank,
    /// 0 follows the fight's difficulty.
    pub level: u32,
    /// Absent, health and damage follow rank and tier (`scale`).
    pub max_health: Option<u32>,
    pub damage: Option<u32>,
    pub scale: Option<f32>,
    pub behaviour: Option<Value>,
    pub power_up_table: Option<String>,
    pub tags: Vec<String>,
    pub side: Side,
    pub boss: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ActorRow {
    pub definition: String,
    pub at: Option<Spot>,
    pub room: Option<String>,
    pub heading: f32,
    pub boss: bool,
    pub dormant: bool,
    pub tags: Vec<String>,
    pub behaviour: Option<Value>,
    pub scale: Option<f32>,
    pub max_health: Option<u32>,
    /// A captive: it cannot be freed while an actor with this tag lives.
    pub guarded_by: Option<String>,
    pub side: Option<Side>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Repeat {
    pub times: u32,
    pub every_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SpawnRow {
    #[serde(default)]
    pub definition: Option<String>,
    /// A named actor of `actors` that enters with this wave.
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default = "one")]
    pub count: u32,
    pub at: Spot,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn ten() -> f32 {
    10.0
}
fn many() -> u32 {
    u32::MAX
}

/// What starts a wave or a fight rule: exactly one per rule.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    /// Only a `spawn_wave` action fires it.
    #[default]
    Manual,
    FightStarted {
        #[serde(default)]
        after_ms: u64,
    },
    WaveLeft {
        wave: String,
        #[serde(default)]
        left: u32,
    },
    ObjectiveDone {
        id: String,
    },
    RoomEntered {
        id: String,
    },
    PlayerWithin {
        point: String,
        #[serde(default = "ten")]
        m: f32,
    },
    Clock {
        ms: u64,
    },
    Every {
        ms: u64,
        #[serde(default)]
        first_ms: u64,
    },
    Signal {
        name: String,
    },
    TagDead {
        tag: String,
        #[serde(default)]
        left: u32,
    },
    /// Any living actor with the tag is below this share of its health.
    HealthBelow {
        tag: String,
        pct: u32,
    },
    Died {
        tag: String,
    },
    Talked {
        tag: String,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Cond {
    TagAlive {
        tag: String,
        #[serde(default)]
        min: u32,
        #[serde(default = "many")]
        max: u32,
    },
    Chance(f32),
    Phase(u8),
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalState {
    /// A later link of the primary chain.
    #[default]
    Waiting,
    Active,
    Done,
    Failed,
}

/// What a fight rule does. The actions an enemy's own rules have (design 8.2) come through as
/// `Brain`; the rest exist only at fight level.
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    SpawnWave(String),
    OpenExit,
    SetObjective { id: String, state: GoalState },
    Credit { id: String, n: u32 },
    Fail,
    Win,
    DropPowerUps { table: String, at: Option<String> },
    Brain(Act),
}

/// One action of a rule: `{"<action>": {...}, "wait_ms": 0, "on": "<tag>"}`. `on` names the
/// actors that carry out a brain action; without it the fight itself does.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(try_from = "Value")]
pub struct Step {
    pub wait_ms: u64,
    pub on: Option<String>,
    pub action: Action,
}

impl TryFrom<Value> for Step {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, String> {
        let object = v.as_object().ok_or("an action must be an object")?;
        let mut step = None;
        let (mut wait_ms, mut on) = (0, None);
        for (key, body) in object {
            match key.as_str() {
                "wait_ms" => wait_ms = body.as_u64().ok_or("wait_ms must be a number")?,
                "on" => on = Some(body.as_str().ok_or("on must be a tag")?.to_owned()),
                name if step.is_none() => step = Some(action(name, body)?),
                _ => return Err("an action holds exactly one action".into()),
            }
        }
        Ok(Step {
            wait_ms,
            on,
            action: step.ok_or("an action holds exactly one action")?,
        })
    }
}

fn line(v: &Value) -> Result<Line, String> {
    if let Some(text) = v.as_str() {
        return Ok(Line {
            text: Some(text.to_owned()),
            ..Line::default()
        });
    }
    let how = match v.get("how").and_then(Value::as_str) {
        None | Some("bubble") => How::Bubble,
        Some("red") => How::Red,
        Some("system") => How::System,
        Some(other) => return Err(format!("unknown say how '{other}'")),
    };
    let text_id = v
        .get("text_id")
        .and_then(Value::as_i64)
        .filter(|id| *id > 0)
        .map(|id| id as i32);
    let text = v.get("text").and_then(Value::as_str).map(str::to_owned);
    if text_id.is_none() && text.is_none() {
        return Err("say needs text_id or text".into());
    }
    Ok(Line { text_id, text, how })
}

fn who(v: Option<&Value>) -> Result<Who, String> {
    Ok(match v.and_then(Value::as_str) {
        None | Some("self") => Who::Own,
        Some("target") => Who::Target,
        Some("all_players") => Who::AllPlayers,
        Some("boss") => Who::Boss,
        Some(tag) => Who::Tag(tag.to_owned()),
    })
}

fn place(v: Option<&Value>) -> ActPlace {
    match v.and_then(Value::as_str) {
        None | Some("self") => ActPlace::Own,
        Some("target") => ActPlace::Target,
        Some("away") => ActPlace::Away,
        Some("points") => ActPlace::Points,
        Some("ring") => ActPlace::Ring,
        Some("behind_allies") => ActPlace::BehindAllies,
        Some(point) => ActPlace::Point(point.to_owned()),
    }
}

fn movement(v: &Value) -> Result<Move, String> {
    let f = |body: &Value, key: &str| body.get(key).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let text = |body: &Value, key: &str| {
        body.get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(format!("move needs {key}"))
    };
    if let Some(name) = v.as_str() {
        return match name {
            "hold" => Ok(Move::Hold),
            "chase" => Ok(Move::Chase),
            "home" => Ok(Move::Home),
            other => Err(format!("unknown move '{other}'")),
        };
    }
    let (name, body) = v
        .as_object()
        .and_then(|o| o.iter().next())
        .ok_or("move must be a name or one object")?;
    Ok(match name.as_str() {
        "keep_away" => Move::KeepAway {
            min: f(body, "min"),
            max: f(body, "max"),
        },
        "flee" => Move::Flee {
            ms: f(body, "ms") as u32,
            speed: f(body, "speed"),
        },
        "behind" => Move::Behind {
            tag: text(body, "tag")?,
            m: f(body, "m"),
        },
        "route" => Move::Route(text(body, "id")?),
        "perch" => Move::Perch(text(body, "points")?),
        "roam" => Move::Roam(f(body, "m")),
        other => return Err(format!("unknown move '{other}'")),
    })
}

fn action(name: &str, v: &Value) -> Result<Action, String> {
    let text = |key: &str| {
        v.get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(format!("{name} needs {key}"))
    };
    let number = |key: &str, default: f64| v.get(key).and_then(Value::as_f64).unwrap_or(default);
    let flag = |key: &str| v.get(key).and_then(Value::as_bool);
    let int = |key: &str| v.get(key).and_then(Value::as_i64).map(|n| n as i32);
    Ok(match name {
        "spawn_wave" => Action::SpawnWave(text("id")?),
        "open_exit" => Action::OpenExit,
        "fail" => Action::Fail,
        "win" => Action::Win,
        "set_objective" => Action::SetObjective {
            id: text("id")?,
            state: match v.get("state") {
                None => GoalState::Done,
                Some(state) => serde_json::from_value(state.clone()).map_err(|e| e.to_string())?,
            },
        },
        "credit" => Action::Credit {
            id: text("id")?,
            n: number("n", 1.0) as u32,
        },
        "drop_power_ups" => Action::DropPowerUps {
            table: text("table")?,
            at: v.get("at").and_then(Value::as_str).map(str::to_owned),
        },
        "say" => Action::Brain(Act::Say(line(v)?)),
        "wake" => Action::Brain(Act::Wake(text("tag")?)),
        "signal" => Action::Brain(Act::Signal(text("name")?)),
        "phase" => Action::Brain(Act::Phase(number("n", 0.0) as u8)),
        "heal" => Action::Brain(Act::Heal {
            who: who(v.get("who"))?,
            pct: number("pct", 100.0) as u32,
        }),
        "status" => Action::Brain(Act::Status {
            id: text("id")?,
            to: who(v.get("to"))?,
            ms: number("ms", 0.0) as u32,
        }),
        "set" => Action::Brain(Act::Set(Flags {
            invulnerable: flag("invulnerable"),
            untargetable: flag("untargetable"),
            speed: v.get("speed").and_then(Value::as_f64).map(|n| n as f32),
            scale: v.get("scale").and_then(Value::as_f64).map(|n| n as f32),
            name_id: int("name_id"),
            model_id: int("model_id"),
            disposition: int("disposition"),
        })),
        "drop" => Action::Brain(Act::Drop(text("table").or_else(|_| text("pickup"))?)),
        "stance" => Action::Brain(Act::Stance(Stance {
            movement: movement(v.get("move").unwrap_or(&Value::String("chase".into())))?,
            attack: v.get("attack").and_then(Value::as_str).map(str::to_owned),
            leash: flag("leash").unwrap_or(false),
        })),
        "attack" => Action::Brain(Act::Attack(text("id")?)),
        "telegraph" => Action::Brain(Act::Telegraph {
            say: line(v.get("say").ok_or("telegraph needs say")?)?,
            ms: number("ms", 0.0) as u32,
            attack: text("attack")?,
        }),
        "summon" => Action::Brain(Act::Summon {
            kind: text("kind")?,
            count: number("count", 1.0) as u8,
            at: place(v.get("at")),
            radius: number("radius", 0.0) as f32,
            tag: v
                .get("tag")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            mound: v.get("visual").and_then(Value::as_str) == Some("mound"),
        }),
        "relocate" => Action::Brain(Act::Relocate {
            to: place(v.get("to")),
            min: number("min", 0.0) as f32,
            max: number("max", 0.0) as f32,
            effect: v.get("effect").and_then(Value::as_str).map(str::to_owned),
        }),
        "become" => Action::Brain(Act::Become {
            definition: text("definition")?,
            health_pct: number("health_pct", 100.0) as u32,
        }),
        "yield" => Action::Brain(Act::Yield),
        "leave" => Action::Brain(Act::Leave(number("ms", 0.0) as u32)),
        "revive" => Action::Brain(Act::Revive {
            pct: number("pct", 100.0) as u32,
            after_ms: number("after_ms", 0.0) as u32,
        }),
        "target" => Action::Brain(Act::Target(match text("rule")?.as_str() {
            "attacker" => TargetRule::Attacker,
            "nearest" => TargetRule::Nearest,
            "lowest_health" => TargetRule::LowestHealth,
            "taunter" => TargetRule::Taunter,
            other => return Err(format!("unknown target rule '{other}'")),
        })),
        other => return Err(format!("unknown action '{other}'")),
    })
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Wave {
    pub id: String,
    #[serde(default)]
    pub when: Trigger,
    /// The wave also comes this long after the wave it waits on appeared (or after the start).
    #[serde(default)]
    pub or_after_ms: Option<u64>,
    #[serde(default)]
    pub repeat: Option<Repeat>,
    pub spawn: Vec<SpawnRow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub when: Trigger,
    #[serde(default, rename = "if")]
    pub conditions: Vec<Cond>,
    #[serde(rename = "do")]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub once: bool,
    #[serde(default)]
    pub cooldown_ms: u64,
    /// Most times it fires; 0 is no limit.
    #[serde(default)]
    pub max: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalKind {
    #[default]
    Defeat,
    Destroy,
    Free,
    Collect,
    Reach,
    Survive,
    Defend,
    Escort,
}

/// Which actors (or pickup, or place) a goal is about. Several keys add up.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Target {
    pub all_waves: bool,
    /// Every enemy of the fight.
    pub all: bool,
    pub waves: Vec<String>,
    pub tag: Option<String>,
    pub actor: Option<String>,
    pub definition: Option<String>,
    pub pickup: Option<String>,
    pub point: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Defeated {
    pub definition: Option<String>,
    pub tag: Option<String>,
    pub count: u32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Until {
    pub defeated: Defeated,
}

/// (R) How enemies wear a ward down: each living enemy within `within` metres of it takes
/// `damage` off every `every_ms`. No recovered rule exists; the live AI aimed at players.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pressure {
    pub within: f32,
    pub damage: u32,
    pub every_ms: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Objective {
    pub id: String,
    pub kind: GoalKind,
    /// The client string of the goal row; 0 shows no row (the goal still counts).
    pub text_id: i32,
    pub text: String,
    pub target: Target,
    pub count: Option<u32>,
    pub primary: bool,
    pub bonus: bool,
    pub ms: u64,
    pub radius: f32,
    pub until: Option<Until>,
    pub pressure: Option<Pressure>,
    /// Escort: the point list walked, the metres per second, and how near a player must be.
    pub route: Option<String>,
    pub to: Option<String>,
    pub speed: f32,
    pub follow_m: f32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Purse {
    pub coins: u32,
    pub job_xp: u32,
    pub items: Vec<(i32, u32)>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Rewards {
    pub coins: u32,
    pub job_xp: Option<u32>,
    pub wheel: Vec<Value>,
    /// The wheel's coin slot (the wiki's "N Coins").
    pub wheel_coins: u32,
    pub drops: Vec<Value>,
    /// Paid on top when every bonus goal is done.
    pub bonus: Purse,
    /// Once per player for taking part in an open-world boss.
    pub participation_chest: Option<Purse>,
}

/// A seasonal world boss's timetable and announcement.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Schedule {
    pub every_s: u32,
    pub towns: Vec<String>,
    /// The point list the boss walks.
    pub route: Option<String>,
    pub banner_text_id: i32,
    pub banner: String,
}

/// One actor of a run, in roster order: named actors first, then every wave firing's spawns.
/// An enemy's place in the roster fixes its guid (`Fight::guid`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Slot {
    pub key: String,
    pub kind: String,
    pub definition_id: u32,
    pub spot: [f32; 3],
    /// (wave index, firing) for an actor a wave brings.
    pub wave: Option<(usize, u32)>,
    /// Standing when the fight begins.
    pub start: bool,
    pub tags: Vec<String>,
    pub boss: bool,
    pub side: Side,
    pub rank: Rank,
    pub guarded_by: Option<String>,
}

/// One loaded fight file.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Fight {
    pub schema_version: u32,
    pub id: u32,
    pub kind: Kind,
    /// `false` keeps a checked file out of play (a conversion waiting for its native run).
    pub enabled: Option<bool>,
    pub title_id: i32,
    pub description_id: i32,
    pub icon_id: Option<i32>,
    pub difficulty: i32,
    pub entry: Entry,
    pub place: Place,
    pub rules: Rules,
    pub schedule: Option<Schedule>,
    pub kinds: BTreeMap<String, KindRow>,
    pub actors: BTreeMap<String, ActorRow>,
    pub points: BTreeMap<String, Vec<[f32; 3]>>,
    pub waves: Vec<Wave>,
    pub objectives: Vec<Objective>,
    pub script: Vec<Rule>,
    pub rewards: Rewards,
    pub source: Vec<Value>,
    /// Built by `check`: every actor a run can hold.
    #[serde(skip)]
    pub roster: Vec<Slot>,
    /// Kinds the roster uses, local rows and catalogue rows alike.
    #[serde(skip)]
    pub resolved: BTreeMap<String, KindRow>,
    /// Its row of `instance_pacing.json`, when it has one (`Fight::pace`).
    #[serde(skip)]
    pub pacing: Option<crate::encounter::pacing::Pacing>,
}

/// `enemies.json` definitions by id, read leniently: only the fields a fight needs, so the
/// catalogue's owner can add columns freely.
#[derive(Clone, Debug, Default)]
pub struct Catalogue(pub BTreeMap<String, KindRow>);

impl Catalogue {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join("enemies.json");
        if !path.is_file() {
            return Ok(Self::default());
        }
        let file: Value = serde_json::from_slice(&fs::read(&path)?)
            .map_err(|e| invalid(&format!("enemies.json: {e}")))?;
        let mut rows = BTreeMap::new();
        for row in file["definitions"].as_array().into_iter().flatten() {
            let Some(id) = row["id"].as_str() else {
                continue;
            };
            let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
            let boss = row["boss"].as_bool().unwrap_or(false);
            rows.insert(
                id.to_owned(),
                KindRow {
                    definition_id: int("definition_id"),
                    name: row["kind"].as_str().unwrap_or(id).to_owned(),
                    name_id: int("name_string_id"),
                    model_id: int("model_id"),
                    texture_alias: row["texture_alias"].as_str().unwrap_or_default().to_owned(),
                    rank: serde_json::from_value(row["rank"].clone()).unwrap_or(if boss {
                        Rank::Boss
                    } else {
                        Rank::Normal
                    }),
                    level: row["level"].as_u64().unwrap_or(0) as u32,
                    max_health: row["max_health"].as_u64().map(|n| n as u32),
                    damage: row["damage"].as_u64().map(|n| n as u32),
                    scale: row["scale"].as_f64().map(|n| n as f32),
                    boss,
                    ..KindRow::default()
                },
            );
        }
        Ok(Self(rows))
    }
}

/// Every source row names its evidence, and at least one reconstructed row says so with `(R)`
/// (the rule `field_enemies.json` rows follow).
fn check_sources(sources: &[Value]) -> Result<(), String> {
    let mut reconstructed = false;
    for source in sources {
        let evidence = source["evidence"].as_str().unwrap_or_default();
        let marked = evidence.starts_with("(R)");
        if evidence.trim().is_empty() || (source["kind"] == "reconstructed" && !marked) {
            return Err("/source: every row needs evidence, a reconstructed one its (R)".into());
        }
        reconstructed |= source["kind"] == "reconstructed";
    }
    reconstructed
        .then_some(())
        .ok_or("/source: needs a reconstructed row".into())
}

fn name_ok(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn spot_ok(p: [f32; 3]) -> bool {
    p.iter().all(|v| v.is_finite() && v.abs() <= 100_000.0)
}

impl Fight {
    /// Parse and check one file. `name` is what a refusal calls it; every refusal is
    /// `<name>: <json pointer>: <reason>`.
    pub fn parse(bytes: &[u8], catalogue: &Catalogue, name: &str) -> io::Result<Self> {
        Self::parse_paced(bytes, catalogue, name, &Default::default())
    }

    /// [`Self::parse`] with the fight's pacing row applied before the checks.
    pub fn parse_paced(
        bytes: &[u8],
        catalogue: &Catalogue,
        name: &str,
        pacing: &crate::encounter::pacing::PacingTable,
    ) -> io::Result<Self> {
        let mut fight: Fight =
            serde_json::from_slice(bytes).map_err(|e| invalid(&format!("{name}: {e}")))?;
        if let Some(row) = pacing.get(fight.id) {
            fight
                .pace(row)
                .map_err(|e| invalid(&format!("{name}: {e}")))?;
        }
        fight
            .check(catalogue)
            .map_err(|e| invalid(&format!("{name}: {e}")))?;
        Ok(fight)
    }

    /// The pacing row's waves (the owner, rc-0136): after the fight's own last wave its waves
    /// come again in order, each once the one before has cleared, until `waves` have come. A
    /// rule that waited on the own last wave (the boss's wake) now waits on the new last one,
    /// and a goal over the own waves counts the new ones too.
    pub fn pace(&mut self, row: &crate::encounter::pacing::Pacing) -> Result<(), String> {
        self.pacing = Some(row.clone());
        let (own, total) = (self.waves.len(), row.waves as usize);
        if own == 0 || total <= own {
            return Ok(());
        }
        if total > MAX_WAVES {
            return Err(format!("pacing: at most {MAX_WAVES} waves"));
        }
        let last = self.waves[own - 1].id.clone();
        let mut previous = last.clone();
        let mut added = Vec::new();
        for n in own..total {
            let mut wave = self.waves[n % own].clone();
            wave.id = format!("{}_{}", wave.id, n / own + 1);
            if self.waves.iter().chain(&added).any(|w: &Wave| w.id == wave.id) {
                return Err(format!("pacing: wave id {} is taken", wave.id));
            }
            wave.when = Trigger::WaveLeft {
                wave: previous.clone(),
                left: 0,
            };
            (wave.or_after_ms, wave.repeat) = (None, None);
            previous.clone_from(&wave.id);
            added.push(wave);
        }
        let own_ids: Vec<String> = self.waves.iter().map(|w| w.id.clone()).collect();
        let new_ids: Vec<String> = added.iter().map(|w| w.id.clone()).collect();
        self.waves.extend(added);
        for rule in &mut self.script {
            if let Trigger::WaveLeft { wave, .. } = &mut rule.when {
                if *wave == last {
                    wave.clone_from(&previous);
                }
            }
        }
        for goal in &mut self.objectives {
            if goal.target.waves.iter().any(|w| own_ids.contains(w)) {
                goal.target.waves.extend(new_ids.iter().cloned());
            }
        }
        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    /// Runs in its own instance (the open-world kinds do not).
    pub fn instanced(&self) -> bool {
        !self.place.open_world
    }

    pub fn knockouts(&self) -> u32 {
        self.rules.knockouts.unwrap_or(match self.kind {
            Kind::Battle => BATTLE_KNOCKOUTS,
            Kind::Dungeon => DUNGEON_KNOCKOUTS,
            _ => 0,
        })
    }

    pub fn group_max(&self) -> usize {
        if self.rules.solo {
            1
        } else {
            (self.rules.group_max as usize).clamp(1, MAX_MEMBERS)
        }
    }

    /// The job level the fight is tuned for.
    pub fn level(&self) -> u32 {
        self.entry
            .min_combat_level
            .max(difficulty_level(self.difficulty))
    }

    /// The guid of roster slot `index`; `None` is the exit door. One block per fight, clear
    /// of every authored guid (the loader checks) and of the pickup range.
    pub fn guid(&self, index: Option<usize>) -> u64 {
        0x4647_0000_0000_0000 | (u64::from(self.id) << 8) | index.map_or(0, |i| i as u64 + 1)
    }

    pub fn exit_spot(&self) -> [f32; 3] {
        self.place.exit_door.unwrap_or(self.place.arrive)
    }

    /// Every tag a rule, goal or guard may name.
    pub fn tags(&self) -> BTreeSet<&str> {
        self.roster
            .iter()
            .flat_map(|slot| slot.tags.iter().map(String::as_str))
            .collect()
    }

    /// Roster slots a goal target names.
    pub fn targets(&self, target: &Target) -> Vec<usize> {
        self.roster
            .iter()
            .enumerate()
            .filter(|(_, slot)| {
                let wave = slot.wave.map(|(w, _)| self.waves[w].id.as_str());
                (target.all && slot.side == Side::Enemy)
                    || (target.all_waves && wave.is_some())
                    || wave.is_some_and(|id| target.waves.iter().any(|w| w == id))
                    || target.tag.as_ref().is_some_and(|t| slot.tags.contains(t))
                    || target.actor.as_ref().is_some_and(|a| *a == slot.key)
                    || target.definition.as_ref().is_some_and(|d| *d == slot.kind)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn point(&self, list: &str, n: usize) -> Option<[f32; 3]> {
        let points = self.points.get(list)?;
        points.get(n % points.len().max(1)).copied()
    }

    fn check(&mut self, catalogue: &Catalogue) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("/schema_version: must be 1".into());
        }
        if self.id == 0 || self.id >= 1 << 24 {
            return Err("/id: must be 1..16777215".into());
        }
        if !(0..=5).contains(&self.difficulty) {
            return Err("/difficulty: must be 0..5".into());
        }
        check_sources(&self.source)?;
        if self.place.open_world {
            if !(self.place.radius > 0.0 && self.place.radius <= 500.0)
                || !spot_ok(self.place.centre)
            {
                return Err("/place: an open-world fight needs a centre and a radius".into());
            }
            if matches!(self.kind, Kind::Battle | Kind::Dungeon) {
                return Err("/place/open_world: a battle or dungeon runs in an arena".into());
            }
        } else {
            let zone = self.zone();
            validate_zone(&zone).map_err(|_| "/place: needs a valid arena".to_string())?;
            if zone.zone_id == 0 || !self.place.exit_door.is_none_or(spot_ok) {
                return Err("/place: needs a zone and a finite exit door".into());
            }
        }
        for (i, room) in self.place.rooms.iter().enumerate() {
            let b = room.bounds;
            if !name_ok(&room.id)
                || b.iter().any(|v| !v.is_finite())
                || (0..3).any(|a| b[a] >= b[a + 3])
            {
                return Err(format!("/place/rooms/{i}: bad room box"));
            }
        }
        if self.rules.knockouts.is_some_and(|n| n > 99)
            || self.rules.group_max == 0
            || self.rules.group_max as usize > MAX_MEMBERS
            || !(1..=600).contains(&self.rules.recover_s)
        {
            return Err(format!(
                "/rules: knockouts 0..99, group_max 1..{MAX_MEMBERS}, recover_s 1..600"
            ));
        }
        if self.kind == Kind::Duel && !self.rules.solo {
            return Err("/rules/solo: a duel is one on one".into());
        }
        if self.schedule.as_ref().is_some_and(|s| {
            self.kind != Kind::WorldBoss
                || s.every_s < 60
                || s.route
                    .as_ref()
                    .is_some_and(|r| !self.points.contains_key(r))
        }) {
            return Err("/schedule: world bosses only, every_s 60 or more, a known route".into());
        }
        for (list, points) in &self.points {
            if !name_ok(list)
                || points.is_empty()
                || points.len() > 64
                || !points.iter().all(|p| spot_ok(*p))
            {
                return Err(format!("/points/{list}: 1..64 finite points"));
            }
        }
        self.build_roster(catalogue)?;
        self.check_waves()?;
        self.check_objectives()?;
        self.check_script()
    }

    fn kind_row(&self, name: &str, catalogue: &Catalogue) -> Option<KindRow> {
        self.kinds
            .get(name)
            .or_else(|| catalogue.0.get(name))
            .cloned()
    }

    fn build_roster(&mut self, catalogue: &Catalogue) -> Result<(), String> {
        if self.waves.len() > MAX_WAVES {
            return Err(format!("/waves: at most {MAX_WAVES}"));
        }
        let mut roster = Vec::new();
        let mut resolved = BTreeMap::new();
        let mut resolve = |name: &str, at: &str| -> Result<KindRow, String> {
            let row = self
                .kind_row(name, catalogue)
                .ok_or(format!("{at}: unknown definition '{name}'"))?;
            if row.model_id <= 0 || row.name.is_empty() || row.name.len() > 120 || row.name_id < 0 {
                return Err(format!(
                    "{at}: definition '{name}' needs a name and a model"
                ));
            }
            resolved.insert(name.to_owned(), row.clone());
            Ok(row)
        };
        // Named actors a wave brings in: (wave index, spawn row index).
        let mut entering = BTreeMap::new();
        for (w, wave) in self.waves.iter().enumerate() {
            for (s, row) in wave.spawn.iter().enumerate() {
                if let Some(actor) = &row.actor {
                    if entering.insert(actor.clone(), (w, s)).is_some()
                        || wave.repeat.is_some()
                        || row.count != 1
                    {
                        return Err(format!(
                            "/waves/{w}/spawn/{s}: actor '{actor}' enters once, in one wave"
                        ));
                    }
                }
            }
        }
        let mut named = BTreeMap::new();
        for (key, row) in &self.actors {
            let at = format!("/actors/{key}");
            if !name_ok(key) {
                return Err(format!("{at}: names are a-z, 0-9 and _"));
            }
            let kind = resolve(&row.definition, &at)?;
            let side = row.side.unwrap_or(kind.side);
            let wave = entering.get(key).copied();
            let spot = match (&row.at, wave) {
                (Some(Spot::At(p)), _) => *p,
                (Some(Spot::Points(list)), _) => self
                    .point(list, roster.len())
                    .ok_or(format!("{at}/at: unknown points '{list}'"))?,
                (None, Some((w, s))) => match &self.waves[w].spawn[s].at {
                    Spot::At(p) => *p,
                    Spot::Points(list) => self
                        .point(list, w * 7 + s)
                        .ok_or(format!("/waves/{w}/spawn/{s}/at: unknown points '{list}'"))?,
                },
                (None, None) => return Err(format!("{at}: needs `at`, or a wave that brings it")),
            };
            if !spot_ok(spot) {
                return Err(format!("{at}/at: not a finite position"));
            }
            if let Some(room) = &row.room {
                if !self.place.rooms.iter().any(|r| r.id == *room) {
                    return Err(format!("{at}/room: unknown room '{room}'"));
                }
            }
            let mut tags = vec![format!("actor:{key}"), format!("kind:{}", row.definition)];
            tags.extend(row.room.iter().map(|room| format!("room:{room}")));
            if let Some((w, _)) = wave {
                tags.push(format!("wave:{}", self.waves[w].id));
            }
            if row.boss || kind.boss {
                tags.push("boss".into());
            }
            tags.extend(kind.tags.iter().chain(&row.tags).cloned());
            named.insert(key.clone(), roster.len());
            roster.push(Slot {
                key: key.clone(),
                kind: row.definition.clone(),
                definition_id: u32::try_from(kind.definition_id).unwrap_or(0),
                spot,
                wave: wave.map(|(w, _)| (w, 0)),
                start: wave.is_none() && !row.dormant,
                tags,
                boss: row.boss || kind.boss,
                side,
                rank: kind.rank,
                guarded_by: row.guarded_by.clone(),
            });
        }
        let mut ids = BTreeSet::new();
        for (w, wave) in self.waves.iter().enumerate() {
            let at = format!("/waves/{w}");
            if !name_ok(&wave.id) || !ids.insert(wave.id.as_str()) {
                return Err(format!("{at}/id: missing, malformed or used twice"));
            }
            let firings = match &wave.repeat {
                None => 1,
                Some(r) if (1..=32).contains(&r.times) && r.every_ms >= MIN_TIMER_MS => r.times,
                Some(_) => {
                    return Err(format!(
                        "{at}/repeat: times 1..32, every_ms {MIN_TIMER_MS} or more"
                    ));
                }
            };
            if wave.spawn.is_empty() || wave.or_after_ms.is_some_and(|ms| ms < MIN_TIMER_MS) {
                return Err(format!(
                    "{at}: needs spawns, and or_after_ms {MIN_TIMER_MS} or more"
                ));
            }
            for firing in 0..firings {
                for (s, row) in wave.spawn.iter().enumerate() {
                    let at = format!("{at}/spawn/{s}");
                    match (&row.definition, &row.actor) {
                        (None, Some(actor)) => {
                            if !named.contains_key(actor) {
                                return Err(format!("{at}: unknown actor '{actor}'"));
                            }
                        }
                        (Some(definition), None) => {
                            let kind = resolve(definition, &at)?;
                            if !(1..=32).contains(&row.count) {
                                return Err(format!("{at}/count: 1..32"));
                            }
                            for n in 0..row.count as usize {
                                let spot = match &row.at {
                                    Spot::At(p) => *p,
                                    Spot::Points(list) => self
                                        .point(list, w * 7 + firing as usize * 3 + roster.len() + n)
                                        .ok_or(format!("{at}/at: unknown points '{list}'"))?,
                                };
                                if !spot_ok(spot) {
                                    return Err(format!("{at}/at: not a finite position"));
                                }
                                let mut tags =
                                    vec![format!("wave:{}", wave.id), format!("kind:{definition}")];
                                if kind.boss {
                                    tags.push("boss".into());
                                }
                                tags.extend(kind.tags.iter().chain(&row.tags).cloned());
                                roster.push(Slot {
                                    key: format!("{}.{firing}.{s}.{n}", wave.id),
                                    kind: definition.clone(),
                                    definition_id: u32::try_from(kind.definition_id).unwrap_or(0),
                                    spot,
                                    wave: Some((w, firing)),
                                    start: false,
                                    tags,
                                    boss: kind.boss,
                                    side: kind.side,
                                    rank: kind.rank,
                                    guarded_by: None,
                                });
                            }
                        }
                        _ => return Err(format!("{at}: names a definition or an actor, not both")),
                    }
                }
            }
        }
        if roster.is_empty() || roster.len() > MAX_ACTORS {
            return Err(format!("/actors: a fight holds 1..{MAX_ACTORS} actors"));
        }
        let known: BTreeSet<&str> = roster
            .iter()
            .flat_map(|slot| slot.tags.iter().map(String::as_str))
            .collect();
        for slot in &roster {
            if let Some(guard) = &slot.guarded_by {
                if !known.contains(guard.as_str()) {
                    return Err(format!(
                        "/actors/{}/guarded_by: unknown tag '{guard}'",
                        slot.key
                    ));
                }
            }
        }
        self.roster = roster;
        self.resolved = resolved;
        Ok(())
    }

    fn tag_known(&self, tag: &str) -> bool {
        self.roster
            .iter()
            .any(|slot| slot.tags.iter().any(|t| t == tag))
    }

    /// A trigger's names must exist; `Err` carries the reason.
    fn check_trigger(&self, when: &Trigger) -> Result<(), String> {
        let tag = |tag: &str| {
            self.tag_known(tag)
                .then_some(())
                .ok_or(format!("unknown tag '{tag}'"))
        };
        match when {
            Trigger::WaveLeft { wave, .. } => self
                .waves
                .iter()
                .any(|w| w.id == *wave)
                .then_some(())
                .ok_or(format!("unknown wave '{wave}'")),
            Trigger::ObjectiveDone { id } => self
                .objectives
                .iter()
                .any(|o| o.id == *id)
                .then_some(())
                .ok_or(format!("unknown objective '{id}'")),
            Trigger::RoomEntered { id } => self
                .place
                .rooms
                .iter()
                .any(|r| r.id == *id)
                .then_some(())
                .ok_or(format!("unknown room '{id}'")),
            Trigger::PlayerWithin { point, m } => {
                (self.points.contains_key(point) && *m > 0.0 && m.is_finite())
                    .then_some(())
                    .ok_or(format!("unknown point '{point}' or bad distance"))
            }
            Trigger::Every { ms, .. } if *ms < MIN_TIMER_MS => {
                Err(format!("a timer is {MIN_TIMER_MS} ms or more"))
            }
            Trigger::TagDead { tag: t, .. }
            | Trigger::HealthBelow { tag: t, .. }
            | Trigger::Died { tag: t }
            | Trigger::Talked { tag: t } => tag(t),
            _ => Ok(()),
        }
    }

    /// Every wave must be able to fire: its trigger names real things, and a chain of
    /// `wave_left` leads back to a wave something starts.
    fn check_waves(&self) -> Result<(), String> {
        let spawned: BTreeSet<&str> = self
            .script
            .iter()
            .flat_map(|rule| &rule.steps)
            .filter_map(|step| match &step.action {
                Action::SpawnWave(id) => Some(id.as_str()),
                _ => None,
            })
            .collect();
        let mut fires = BTreeSet::new();
        for (w, wave) in self.waves.iter().enumerate() {
            self.check_trigger(&wave.when)
                .map_err(|e| format!("/waves/{w}/when: {e}"))?;
            if !matches!(wave.when, Trigger::WaveLeft { .. } | Trigger::Manual)
                || spawned.contains(wave.id.as_str())
            {
                fires.insert(wave.id.as_str());
            }
        }
        loop {
            let before = fires.len();
            for wave in &self.waves {
                if let Trigger::WaveLeft { wave: on, .. } = &wave.when {
                    if fires.contains(on.as_str()) && *on != wave.id {
                        fires.insert(wave.id.as_str());
                    }
                }
            }
            if fires.len() == before {
                break;
            }
        }
        match self
            .waves
            .iter()
            .position(|w| !fires.contains(w.id.as_str()))
        {
            Some(w) => Err(format!(
                "/waves/{w}: wave '{}' can never fire",
                self.waves[w].id
            )),
            None => Ok(()),
        }
    }

    /// Whether roster slot `i` can ever stand in the fight.
    fn appears(&self, i: usize) -> bool {
        let slot = &self.roster[i];
        slot.start
            || slot.wave.is_some()
            || self.script.iter().flat_map(|rule| &rule.steps).any(|step| {
                matches!(&step.action, Action::Brain(Act::Wake(tag)) if slot.tags.contains(tag))
            })
    }

    fn check_objectives(&self) -> Result<(), String> {
        if self.objectives.is_empty() || self.objectives.len() > MAX_OBJECTIVES {
            return Err(format!("/objectives: 1..{MAX_OBJECTIVES} goals"));
        }
        if self.objectives.iter().all(|o| o.bonus) {
            return Err("/objectives: at least one primary goal".into());
        }
        let mut ids = BTreeSet::new();
        for (i, goal) in self.objectives.iter().enumerate() {
            let at = format!("/objectives/{i}");
            let unreachable = |why: &str| Err(format!("{at}: unreachable objective: {why}"));
            if !name_ok(&goal.id) || !ids.insert(goal.id.as_str()) {
                return Err(format!("{at}/id: missing, malformed or used twice"));
            }
            if goal.primary && goal.bonus {
                return Err(format!("{at}: primary or bonus, not both"));
            }
            if goal.text_id < 0 || goal.text.len() > 200 {
                return Err(format!("{at}: bad text"));
            }
            if let Some(tag) = &goal.target.tag {
                if !self.tag_known(tag) {
                    return Err(format!("{at}/target/tag: unknown tag '{tag}'"));
                }
            }
            for wave in &goal.target.waves {
                if !self.waves.iter().any(|w| w.id == *wave) {
                    return Err(format!("{at}/target/waves: unknown wave '{wave}'"));
                }
            }
            let set = self.targets(&goal.target);
            match goal.kind {
                GoalKind::Defeat | GoalKind::Destroy | GoalKind::Free => {
                    let want = goal.kind == GoalKind::Free;
                    if set.is_empty() {
                        return unreachable("it names no actor");
                    }
                    if set
                        .iter()
                        .any(|&s| (self.roster[s].side == Side::Ally) != want)
                    {
                        return unreachable("enemies are defeated, allies are freed");
                    }
                    if let Some(s) = set.iter().find(|&&s| !self.appears(s)) {
                        return unreachable(&format!(
                            "actor '{}' never appears",
                            self.roster[*s].key
                        ));
                    }
                    if goal.count.is_some_and(|n| n == 0 || n as usize > set.len()) {
                        return unreachable("its count is more than the actors it names");
                    }
                }
                GoalKind::Collect => {
                    if goal.target.pickup.as_ref().is_none_or(|p| p.is_empty()) {
                        return unreachable("it names no pickup");
                    }
                }
                GoalKind::Reach => {
                    let place = goal.target.point.as_deref().unwrap_or_default();
                    if !self.points.contains_key(place)
                        && !self.place.rooms.iter().any(|r| r.id == place)
                    {
                        return unreachable(&format!("unknown point or room '{place}'"));
                    }
                }
                GoalKind::Survive => {
                    if goal.ms < MIN_TIMER_MS {
                        return unreachable("it has no time to survive");
                    }
                }
                GoalKind::Defend | GoalKind::Escort => {
                    let ward = set
                        .iter()
                        .copied()
                        .find(|&s| goal.target.actor.as_ref() == Some(&self.roster[s].key));
                    let Some(ward) = ward.filter(|&s| self.appears(s)) else {
                        return unreachable("it names no actor that appears");
                    };
                    if self.roster[ward].side != Side::Ally {
                        return unreachable("the actor to keep safe must be `side: ally`");
                    }
                    if goal.kind == GoalKind::Defend {
                        let until = goal.until.as_ref().map(|u| &u.defeated);
                        let enough = until.is_none_or(|d| {
                            let pool = self.targets(&Target {
                                tag: d.tag.clone(),
                                definition: d.definition.clone(),
                                ..Target::default()
                            });
                            d.count > 0 && pool.len() >= d.count as usize
                        });
                        if (goal.ms == 0 && until.is_none()) || !enough {
                            return unreachable("it needs `ms`, or `until` with enough enemies");
                        }
                        if goal.pressure.as_ref().is_some_and(|p| {
                            p.every_ms < MIN_TIMER_MS || !(p.within > 0.0) || p.damage == 0
                        }) {
                            return Err(format!("{at}/pressure: bad values"));
                        }
                    } else {
                        let route = goal.route.as_deref().unwrap_or_default();
                        if !self.points.contains_key(route)
                            || !(goal.speed > 0.0 && goal.speed <= 20.0)
                        {
                            return unreachable("an escort needs a route and a speed up to 20");
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn check_script(&self) -> Result<(), String> {
        if self.script.len() > MAX_RULES {
            return Err(format!("/script: at most {MAX_RULES} rules"));
        }
        let mut ids = BTreeSet::new();
        for (r, rule) in self.script.iter().enumerate() {
            let at = format!("/script/{r}");
            if !name_ok(&rule.id) || !ids.insert(rule.id.as_str()) {
                return Err(format!("{at}/id: missing, malformed or used twice"));
            }
            if rule.steps.is_empty() || rule.steps.len() > MAX_ACTIONS {
                return Err(format!("{at}/do: 1..{MAX_ACTIONS} actions"));
            }
            if matches!(rule.when, Trigger::Manual) {
                return Err(format!("{at}/when: a rule needs a trigger"));
            }
            self.check_trigger(&rule.when)
                .map_err(|e| format!("{at}/when: {e}"))?;
            for cond in &rule.conditions {
                match cond {
                    Cond::TagAlive { tag, .. } if !self.tag_known(tag) => {
                        return Err(format!("{at}/if: unknown tag '{tag}'"));
                    }
                    Cond::Chance(p) if !(0.0..=1.0).contains(p) => {
                        return Err(format!("{at}/if: chance is 0..1"));
                    }
                    _ => {}
                }
            }
            for (s, step) in rule.steps.iter().enumerate() {
                let at = format!("{at}/do/{s}");
                if step.wait_ms > 600_000 {
                    return Err(format!("{at}/wait_ms: ten minutes at most"));
                }
                if let Some(tag) = &step.on {
                    if !self.tag_known(tag) {
                        return Err(format!("{at}/on: unknown tag '{tag}'"));
                    }
                }
                let goal = |id: &str| {
                    self.objectives
                        .iter()
                        .any(|o| o.id == id)
                        .then_some(())
                        .ok_or(format!("{at}: unknown objective '{id}'"))
                };
                match &step.action {
                    Action::SpawnWave(id) if !self.waves.iter().any(|w| w.id == *id) => {
                        return Err(format!("{at}: unknown wave '{id}'"));
                    }
                    Action::SetObjective { id, .. } | Action::Credit { id, .. } => goal(id)?,
                    Action::DropPowerUps {
                        at: Some(point), ..
                    } if !self.points.contains_key(point) => {
                        return Err(format!("{at}: unknown point '{point}'"));
                    }
                    Action::Brain(Act::Wake(tag)) if !self.tag_known(tag) => {
                        return Err(format!("{at}: unknown tag '{tag}'"));
                    }
                    Action::Brain(Act::Heal {
                        who: Who::Tag(tag), ..
                    })
                    | Action::Brain(Act::Status {
                        to: Who::Tag(tag), ..
                    }) if !self.tag_known(tag) => {
                        return Err(format!("{at}: unknown tag '{tag}'"));
                    }
                    _ => {}
                }
            }
            // A rule a signal starts, that raises the same signal at once and may fire
            // without limit, would run every tick.
            if let Trigger::Signal { name } = &rule.when {
                let loops = rule.steps.iter().any(|step| {
                    step.wait_ms < MIN_TIMER_MS
                        && matches!(&step.action, Action::Brain(Act::Signal(raised)) if raised == name)
                });
                if loops && !rule.once && rule.max == 0 && rule.cooldown_ms < MIN_TIMER_MS {
                    return Err(format!(
                        "{at}: raises its own signal with no cooldown; add cooldown_ms, once or max"
                    ));
                }
            }
        }
        Ok(())
    }

    /// The arena as the engine's zoning record.
    pub(in crate::encounter) fn zone(&self) -> ZoneTransition {
        ZoneTransition {
            zone_id: self.place.zone,
            name: self.place.arena.clone(),
            sky: self.place.sky.clone(),
            geometry_id: self.place.geometry_id,
            position: self.place.arrive,
            rotation: self.place.heading,
            tutorial: false,
        }
    }
}
