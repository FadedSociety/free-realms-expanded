//! T458: world events, part 1. An event in the content directory's `world_events.json` (beside
//! `mounts.json`) names its actors and a timeline of ops; an admin's `realm event start <key>`
//! runs it in the open world and `realm event stop <key>` ends it. The file is read at each
//! start, so an edit applies to the next run. Runs live in memory only (R): a restart mid-run
//! leaves nothing behind and nothing respawns.
//!
//! T460, part 2: `say`, `broadcast`, `move`, `area`, `cycle_anim` and `one_of`. Step times in a
//! `one_of` branch are offsets from the `one_of` step; a `cycle_anim`'s `until_ms` counts from
//! its own step (R). Every random choice (one_of at the run's start, cycle order and jitter)
//! comes from the run's `Rng`, seeded from the run serial and start time (R); tests pass a seed.
//!
//! Wire, every reader checked in the client:
//! * spawn: AddNpc (`startup_world::build_add_npc`, reader `0x00921a10`), then the optional poof
//!   as 35/16 PlayCompositeEffect (reader `0x008e5f50`).
//! * remove: 35/3 RemovePlayerGracefully carrying the poof (readers `0x008d69f0` and
//!   `0x008d6a60`; PlayClient plays a nonzero effect at `0x0093083b`).
//! * base_anim and play_anim: 35/8 with and without the base flag (reader `0x008e5dd0`).
//! * effect: 35/16. tag_on: 35/41 (reader `0x008d7ad0`) with a tag id this runner assigns and
//!   the actor as both guid and source; tag_off: 35/42 (reader `0x008d7bc0`) with that tag id.
//!
//! * say: 15/4 (`chat::string_say`, reader `0x00b0f680`), from the actor's guid, to the sessions
//!   this actor has been shown to within `LOCAL_CHAT_RADIUS` of it (3-D, as `social_chat`): the
//!   client looks the speaker up by guid, so a session without the AddNpc gets nothing.
//! * broadcast: 15/3 system line (`chat::system`, as T199's notice). The original probably used
//!   the undecoded 77/2 BroadcastPacketZone (R).
//! * move: opcode 125 (`piece_052::movement_packet`, reader `0x008e5940`), one per tick, and
//!   the base animation walk (2) while moving, then the actor's stand animation on arrival.
//! * area: opcode 89 from `ClientAreas::set_area` (T457) to every session in the run's instance.
//!
//! Visibility: each tick every live actor shows itself (AddNpc at its current position, current
//! base animation, live tags) to the registry's nearby sessions it has not been shown to, and
//! forgets sessions that left. Event guids never enter a session's `known_npcs`, so the
//! shared-field NPC budget is untouched. Cost per tick: one emptiness check when idle, else one
//! `nearby_sessions` query per live actor.
use crate::abilities::effect_packets::{
    encode_add_composite_effect, encode_play_composite_effect, encode_remove_composite_effect,
    encode_remove_gracefully,
};
use crate::client_areas::ClientAreas;
use crate::encounter::animation;
use crate::players::{PlayerRegistry, SessionId, ZoneInstance};
use crate::power_ups::Rng;
use crate::social_chat::LOCAL_CHAT_RADIUS;
use crate::startup_world::{NpcSpawn, build_add_npc};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::{
    io::{self, Read},
    path::Path,
    sync::Arc,
};

// Not registered in lib.rs (shared); include the one file here, as `social_chat` does.
#[path = "restoration/piece_052.rs"]
pub mod piece_052;
use piece_052::{ANIM_ID_WALK, MAX_PATROL_SPEED, movement_packet};

pub const FILE: &str = "world_events.json";
/// 'EVNT' in the high half; the run serial (bits 8..32) and the actor index fill the low half,
/// beside the other actor tags (mounts.rs, pets.rs, encounter.rs, harvest_nodes.rs, housing.rs).
/// The 35/3 header: the runner sends nothing else from 35/3 but a removal.
const REMOVAL_HEAD: [u8; 4] = [35, 0, 3, 0];
const GUID_TAG: u64 = 0x4556_4e54_0000_0000;
const SERIALS: u32 = 0xff_ffff;
// (R) Sanity bounds for hand-written content, not tuning values.
const MAX_EVENTS: usize = 32;
const MAX_ACTORS: usize = 32;
const MAX_STEPS: usize = 512;
const MAX_AT_MS: u64 = 2 * 60 * 60 * 1000;
const MAX_NAME_BYTES: usize = 64;
const MAX_BRANCHES: usize = 8;
const MAX_CYCLE_IDS: usize = 32;
const MIN_CYCLE_MS: u64 = 100;
const MAX_TEXT_BYTES: usize = 256;
const MAX_RADIUS_M: f32 = 10_000.0;
const MAX_COORD: f32 = 1_000_000.0;
const MIN_SPEED_MPS: f32 = 0.1;
/// The bound the sibling content loaders use (`abilities.rs`, `power_ups.rs`).
const MAX_JSON_BYTES: u64 = 1_048_576;

type Out = Vec<(SessionId, Vec<u8>)>;

/// An admin's `realm event start|stop <key>`, staged by gameplay. The gateway host runs it once
/// after the admin's own transaction commits (`bin/osfr_gateway/world_event_routing.rs`).
#[derive(Clone, Debug, PartialEq)]
pub enum WorldEventCommand {
    Start(String),
    Stop(String),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EventsFile {
    events: Vec<EventRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EventRow {
    key: String,
    actors: Vec<ActorRow>,
    timeline: Vec<StepRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActorRow {
    key: String,
    model_id: i32,
    /// Client locale text id for the nameplate; 0 shows no name, as `world::Placement`.
    name_string_id: i32,
    npc_name: String,
    position: [f32; 3],
    heading: f32,
    stand_animation_id: i32,
    hide_nameplate: bool,
    /// Provenance (script, capture or page) for authors; required, never sent.
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StepRow {
    at_ms: u64,
    op: OpRow,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Order {
    Random,
    Sequence,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum OpRow {
    Spawn {
        actor: String,
        effect_id: Option<i32>,
    },
    Remove {
        actor: String,
        effect_id: Option<i32>,
    },
    BaseAnim {
        actor: String,
        id: i32,
    },
    PlayAnim {
        actor: String,
        id: i32,
    },
    Effect {
        actor: String,
        id: i32,
    },
    TagOn {
        actor: String,
        effect_id: i32,
    },
    TagOff {
        actor: String,
        effect_id: i32,
    },
    Say {
        actor: String,
        string_id: i32,
    },
    /// The client locale line, copied as text; `locale_id` is its provenance and is never sent.
    Broadcast {
        text: String,
        locale_id: i32,
        radius_m: Option<f32>,
    },
    Move {
        actor: String,
        to: [f32; 3],
        speed_mps: f32,
    },
    Area {
        area_id: i32,
        on: bool,
    },
    CycleAnim {
        actors: Vec<String>,
        ids: Vec<i32>,
        every_ms: u64,
        jitter_ms: u64,
        until_ms: u64,
        order: Order,
    },
    OneOf {
        branches: Vec<Vec<StepRow>>,
    },
}

/// One validated event. Steps are in `at_ms` order; equal times keep file order.
#[derive(Debug, PartialEq)]
pub struct WorldEvent {
    pub key: String,
    actors: Vec<Actor>,
    /// Every step, each `one_of` branch's included (tagged with its `branch`).
    steps: Vec<Step>,
    cycles: Vec<Cycle>,
    broadcasts: Vec<Broadcast>,
    /// The branch count of each `one_of`, in file order.
    branches: Vec<usize>,
}

#[derive(Debug, PartialEq)]
struct Actor {
    /// The AddNpc without its per-run guid.
    spawn: NpcSpawn,
    position: [f32; 3],
    /// The poof of this actor's last `remove` step (0 for none); `stop` removes with it.
    remove_effect: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Step {
    at_ms: u64,
    /// The acting actor; 0 and unused for the ops that name none (broadcast, area, cycle).
    actor: usize,
    op: Op,
    /// (`one_of` index, branch index) for a step inside a branch: it runs only when that
    /// branch was drawn at the run's start.
    branch: Option<(usize, usize)>,
}

/// Each op's composite effect, animation or effect id; 0 is "no poof" for spawn and remove.
/// `Broadcast` and `Cycle` index the event's `broadcasts` and `cycles`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    Spawn(i32),
    Remove(i32),
    BaseAnim(i32),
    PlayAnim(i32),
    Effect(i32),
    TagOn(i32),
    TagOff(i32),
    Say(i32),
    Move { to: [f32; 3], speed: f32 },
    Broadcast(usize),
    Area(i32, bool),
    Cycle(usize),
}

#[derive(Debug, PartialEq)]
struct Cycle {
    actors: Vec<usize>,
    ids: Vec<i32>,
    every_ms: u64,
    jitter_ms: u64,
    until_ms: u64,
    sequence: bool,
}

#[derive(Debug, PartialEq)]
struct Broadcast {
    text: String,
    /// Metres on X/Z around the event's first actor; `None` is the whole instance.
    radius_m: Option<f32>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

/// Event keys are typed in chat, which gameplay lowercases.
fn valid_key(key: &str) -> bool {
    (1..=32).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Every event in `<dir>/world_events.json`, validated; a missing file is no events.
pub fn load(dir: &Path) -> io::Result<Vec<WorldEvent>> {
    let mut file = match std::fs::File::open(dir.join(FILE)) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        file => file?,
    };
    // Checked before reading: the load runs on the server thread inside an admin's commit.
    if file.metadata()?.len() > MAX_JSON_BYTES {
        return Err(invalid(format!(
            "{FILE} is larger than {MAX_JSON_BYTES} bytes"
        )));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let file: EventsFile = serde_json::from_slice(&bytes).map_err(invalid)?;
    if file.events.len() > MAX_EVENTS {
        return Err(invalid("at most 32 world events"));
    }
    let mut keys = BTreeSet::new();
    file.events
        .into_iter()
        .map(|row| match keys.insert(row.key.clone()) {
            true => compile(row),
            false => Err(invalid(format!("world event {} is listed twice", row.key))),
        })
        .collect()
}

fn compile(row: EventRow) -> io::Result<WorldEvent> {
    let EventRow {
        key,
        actors: rows,
        timeline,
    } = row;
    let fail = |what: String| invalid(format!("world event {key}: {what}"));
    if !valid_key(&key) || rows.len() > MAX_ACTORS || timeline.len() > MAX_STEPS {
        return Err(fail(
            "needs a key of 1-32 [a-z0-9_-], at most 32 actors and at most 512 steps".into(),
        ));
    }
    let mut index = BTreeMap::new();
    let mut actors = Vec::with_capacity(rows.len());
    for row in rows {
        let finite = row
            .position
            .iter()
            .chain([&row.heading])
            .all(|v| v.is_finite());
        if !valid_key(&row.key)
            || index.insert(row.key.clone(), actors.len()).is_some()
            || row.model_id <= 0
            || row.name_string_id < 0
            || row.stand_animation_id <= 0
            || !finite
            || row.npc_name.len() > MAX_NAME_BYTES
            || row.source.is_empty()
        {
            return Err(fail(format!(
                "actor {} needs a unique key, ids > 0, a finite pose, a name of at most 64 bytes and a source",
                row.key
            )));
        }
        let [x, y, z] = row.position;
        actors.push(Actor {
            spawn: NpcSpawn {
                name_string_id: row.name_string_id,
                model_id: row.model_id,
                npc_name: row.npc_name,
                hide_nameplate: u8::from(row.hide_nameplate),
                stand_animation_id: row.stand_animation_id,
                position: [x, y, z, 1.0],
                rotation: [row.heading.sin(), 0.0, row.heading.cos(), 0.0],
                // (R) Event actors are scenery: no interaction prompt, like rides (mounts.rs).
                is_interactable: 0,
                interact_range: 0,
                ..crate::world::content_spawn_template()
            },
            position: row.position,
            remove_effect: 0,
        });
    }
    let mut builder = Builder {
        index: &index,
        steps: Vec::with_capacity(timeline.len()),
        cycles: Vec::new(),
        broadcasts: Vec::new(),
        branches: Vec::new(),
    };
    for step in timeline {
        builder.add(step, 0, None).map_err(&fail)?;
    }
    let Builder {
        mut steps,
        cycles,
        broadcasts,
        branches,
        ..
    } = builder;
    steps.sort_by_key(|step| step.at_ms);
    for step in &steps {
        if let Op::Remove(effect) = step.op {
            actors[step.actor].remove_effect = effect;
        }
    }
    Ok(WorldEvent {
        key,
        actors,
        steps,
        cycles,
        broadcasts,
        branches,
    })
}

/// Turns timeline rows (and the rows inside `one_of` branches) into steps.
struct Builder<'a> {
    index: &'a BTreeMap<String, usize>,
    steps: Vec<Step>,
    cycles: Vec<Cycle>,
    broadcasts: Vec<Broadcast>,
    branches: Vec<usize>,
}

fn positive(id: i32) -> Result<i32, String> {
    (id > 0)
        .then_some(id)
        .ok_or_else(|| "every step needs ids > 0".into())
}

impl Builder<'_> {
    fn actor(&self, key: &str) -> Result<usize, String> {
        match self.index.get(key) {
            Some(&actor) => Ok(actor),
            None => Err(format!("a step names unknown actor {key}")),
        }
    }

    /// One row at `base` + its own `at_ms`; `branch` is set for a row inside a `one_of`.
    fn add(
        &mut self,
        row: StepRow,
        base: u64,
        branch: Option<(usize, usize)>,
    ) -> Result<(), String> {
        let at_ms = base.saturating_add(row.at_ms);
        if row.at_ms > MAX_AT_MS || at_ms > MAX_AT_MS || self.steps.len() >= MAX_STEPS {
            return Err("every step needs at_ms <= 2 h, and an event has at most 512 steps".into());
        }
        let optional = |id: Option<i32>| id.map_or(Ok(0), positive);
        let (actor, op) = match row.op {
            OpRow::Spawn { actor, effect_id } => {
                (self.actor(&actor)?, Op::Spawn(optional(effect_id)?))
            }
            OpRow::Remove { actor, effect_id } => {
                (self.actor(&actor)?, Op::Remove(optional(effect_id)?))
            }
            OpRow::BaseAnim { actor, id } => (self.actor(&actor)?, Op::BaseAnim(positive(id)?)),
            OpRow::PlayAnim { actor, id } => (self.actor(&actor)?, Op::PlayAnim(positive(id)?)),
            OpRow::Effect { actor, id } => (self.actor(&actor)?, Op::Effect(positive(id)?)),
            OpRow::TagOn { actor, effect_id } => {
                (self.actor(&actor)?, Op::TagOn(positive(effect_id)?))
            }
            OpRow::TagOff { actor, effect_id } => {
                (self.actor(&actor)?, Op::TagOff(positive(effect_id)?))
            }
            OpRow::Say { actor, string_id } => (self.actor(&actor)?, Op::Say(positive(string_id)?)),
            OpRow::Move {
                actor,
                to,
                speed_mps,
            } => {
                let target = to.iter().all(|v| v.is_finite() && v.abs() <= MAX_COORD);
                if !target || !(MIN_SPEED_MPS..=MAX_PATROL_SPEED).contains(&speed_mps) {
                    return Err(
                        "move needs a finite target within 1e6 m and a speed of 0.1-6.25 m/s"
                            .into(),
                    );
                }
                let speed = speed_mps;
                (self.actor(&actor)?, Op::Move { to, speed })
            }
            OpRow::Broadcast {
                text,
                locale_id,
                radius_m,
            } => {
                positive(locale_id)?;
                let radius = radius_m.is_none_or(|r| r > 0.0 && r <= MAX_RADIUS_M);
                let line = !text.trim().is_empty()
                    && text.len() <= MAX_TEXT_BYTES
                    && !text.chars().any(char::is_control);
                // A radius is measured from the first actor.
                if !(radius && line) || (radius_m.is_some() && self.index.is_empty()) {
                    return Err("broadcast needs a text line of at most 256 bytes without control characters, and a radius of 0-10000 m around a first actor".into());
                }
                self.broadcasts.push(Broadcast { text, radius_m });
                (0, Op::Broadcast(self.broadcasts.len() - 1))
            }
            OpRow::Area { area_id, on } => (0, Op::Area(positive(area_id)?, on)),
            OpRow::CycleAnim {
                actors,
                ids,
                every_ms,
                jitter_ms,
                until_ms,
                order,
            } => {
                let actors = actors
                    .iter()
                    .map(|key| self.actor(key))
                    .collect::<Result<Vec<_>, _>>()?;
                let ids = ids
                    .into_iter()
                    .map(positive)
                    .collect::<Result<Vec<_>, _>>()?;
                let sane = !actors.is_empty()
                    && actors.len() <= MAX_ACTORS
                    && !ids.is_empty()
                    && ids.len() <= MAX_CYCLE_IDS
                    && (MIN_CYCLE_MS..=MAX_AT_MS).contains(&every_ms)
                    && jitter_ms <= MAX_AT_MS
                    && until_ms <= MAX_AT_MS;
                if !sane {
                    return Err("cycle_anim needs actors, 1-32 ids, every_ms of 100 ms-2 h, and jitter_ms and until_ms of at most 2 h".into());
                }
                self.cycles.push(Cycle {
                    actors,
                    ids,
                    every_ms,
                    jitter_ms,
                    until_ms,
                    sequence: matches!(order, Order::Sequence),
                });
                (0, Op::Cycle(self.cycles.len() - 1))
            }
            OpRow::OneOf { branches } => {
                if branch.is_some() || !(1..=MAX_BRANCHES).contains(&branches.len()) {
                    return Err("one_of needs 1-8 branches and cannot nest".into());
                }
                let group = self.branches.len();
                self.branches.push(branches.len());
                for (number, rows) in branches.into_iter().enumerate() {
                    for row in rows {
                        self.add(row, at_ms, Some((group, number)))?;
                    }
                }
                return Ok(());
            }
        };
        self.steps.push(Step {
            at_ms,
            actor,
            op,
            branch,
        });
        Ok(())
    }
}

/// The live runs, at most one per event key, owned by the gateway host's shared world.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldEvents {
    runs: BTreeMap<String, Run>,
    /// Removals for the next tick, sent first: `stop`'s, and the ones a session refused.
    pending: Out,
    /// Areas a stopped run had on, switched off by the next tick.
    pending_off: Vec<(ZoneInstance, i32)>,
    serial: u32,
    tag: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    event: Arc<WorldEvent>,
    instance: ZoneInstance,
    started_ms: u64,
    serial: u32,
    /// This run's timeline: the event's steps without the `one_of` branches that were not drawn.
    steps: Vec<Step>,
    next_step: usize,
    actors: Vec<Shown>,
    rng: Rng,
    cycles: Vec<Playing>,
    /// Areas this run switched on and not yet off; `stop` and the run's end switch them off.
    areas: BTreeSet<i32>,
}

/// A `cycle_anim` that is running: its next play, its last allowed play and the plays so far.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Playing {
    cycle: usize,
    next_ms: u64,
    end_ms: u64,
    count: usize,
}

/// One actor's live state, reset when it is removed.
#[derive(Clone, Debug, Default, PartialEq)]
struct Shown {
    live: bool,
    base_anim: Option<i32>,
    /// Composite effect id -> the tag id its 35/41 used; the 35/42 sends that tag id back.
    tags: BTreeMap<i32, i32>,
    seen_by: BTreeSet<SessionId>,
    /// Where the actor is now: its spawn point until a `move` walks it. A viewer who arrives
    /// late gets its AddNpc here, not at the spawn point.
    position: [f32; 3],
    /// The AddNpc rotation, turned toward the target by `move`.
    rotation: [f32; 4],
    mover: Option<Mover>,
}

/// A `move` in progress: one opcode 125 step per tick toward `to`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Mover {
    to: [f32; 3],
    speed: f32,
    last_ms: u64,
}

/// What a step needs from the world besides its own run.
struct Env<'a> {
    players: &'a PlayerRegistry,
    areas: &'a mut ClientAreas,
    unix_ms: u64,
    tag: &'a mut i32,
}

/// Queue `packet()` for every session in `to`; with none, build and allocate nothing.
fn deliver(out: &mut Out, to: impl Iterator<Item = SessionId>, packet: impl FnOnce() -> Vec<u8>) {
    let mut to = to.peekable();
    if to.peek().is_some() {
        let packet = packet();
        out.extend(to.map(|session| (session, packet.clone())));
    }
}

/// Whether `b` is within `radius` metres of `a`, on X/Z only when `flat`.
fn within(a: [f32; 3], b: [f32; 3], radius: f32, flat: bool) -> bool {
    let d = |axis: usize| f64::from(a[axis]) - f64::from(b[axis]);
    let height = if flat { 0.0 } else { d(1) };
    d(0) * d(0) + height * height + d(2) * d(2) <= f64::from(radius).powi(2)
}

impl WorldEvents {
    /// Start `event` in `instance`; false, changing nothing, while a run of its key is live.
    pub fn start(&mut self, event: WorldEvent, instance: ZoneInstance, now_ms: u64) -> bool {
        self.begin(event, instance, now_ms, None)
    }

    /// `start` with the run's random stream fixed (`seed`), so a test can draw each branch.
    fn begin(
        &mut self,
        event: WorldEvent,
        instance: ZoneInstance,
        now_ms: u64,
        seed: Option<u64>,
    ) -> bool {
        if self.runs.contains_key(&event.key) {
            return false;
        }
        // A wrapped serial skips every live run's, so a live guid is never reused.
        let serial = loop {
            self.serial = self.serial % SERIALS + 1;
            if self.runs.values().all(|run| run.serial != self.serial) {
                break self.serial;
            }
        };
        let mut rng =
            Rng::seeded(seed.unwrap_or_else(|| Rng::mix(u64::from(serial) << 32 ^ now_ms)));
        let drawn: Vec<usize> = event
            .branches
            .iter()
            .map(|count| (rng.next_u64() % *count as u64) as usize)
            .collect();
        let steps = event
            .steps
            .iter()
            .filter(|step| {
                step.branch
                    .is_none_or(|(group, number)| drawn[group] == number)
            })
            .copied()
            .collect();
        let run = Run {
            actors: vec![Shown::default(); event.actors.len()],
            event: Arc::new(event),
            instance,
            started_ms: now_ms,
            serial,
            steps,
            next_step: 0,
            rng,
            cycles: Vec::new(),
            areas: BTreeSet::new(),
        };
        self.runs.insert(run.event.key.clone(), run);
        true
    }

    /// End `key`'s run: the next tick removes every live actor with its remove step's poof and
    /// switches off the areas the run had on, and the key may start again at once. False when
    /// it is not running.
    pub fn stop(&mut self, key: &str) -> bool {
        let Some(run) = self.runs.remove(key) else {
            return false;
        };
        for (index, shown) in run.actors.iter().enumerate() {
            let bytes =
                encode_remove_gracefully(run.guid(index), run.event.actors[index].remove_effect);
            self.pending.extend(
                shown
                    .seen_by
                    .iter()
                    .map(|session| (*session, bytes.clone())),
            );
        }
        self.pending_off
            .extend(run.areas.iter().map(|area| (run.instance, *area)));
        true
    }

    /// Run every step that fell due since the last tick, once each in `at_ms` order, then move
    /// the walking actors, play the animation cycles and show each live actor to the sessions
    /// that came near. A run ends once its last step has run and no actor is live; its areas
    /// are switched off then. `unix_ms` is the wall clock `ClientAreas` keeps its toggles by.
    /// Returns inner packets per session; idle, it allocates nothing.
    pub fn tick(
        &mut self,
        now_ms: u64,
        unix_ms: u64,
        players: &PlayerRegistry,
        areas: &mut ClientAreas,
    ) -> Out {
        let mut out = std::mem::take(&mut self.pending);
        let mut env = Env {
            players,
            areas: &mut *areas,
            unix_ms,
            tag: &mut self.tag,
        };
        for run in self.runs.values_mut() {
            while let Some(step) = run
                .steps
                .get(run.next_step)
                .copied()
                .filter(|step| run.started_ms.saturating_add(step.at_ms) <= now_ms)
            {
                run.next_step += 1;
                if let Err(reason) = run.apply(step, &mut env, &mut out) {
                    println!(
                        "{}",
                        serde_json::json!({"event": "world_event_step_skipped",
                            "key": run.event.key, "at_ms": step.at_ms, "reason": reason})
                    );
                }
            }
            run.play_cycles(now_ms, &mut out);
            for index in 0..run.actors.len() {
                if run.actors[index].live {
                    run.advance(index, now_ms, &mut out);
                    run.view(index, players, &mut out);
                }
            }
        }
        let mut off = std::mem::take(&mut self.pending_off);
        self.runs.retain(|_, run| {
            let done = run.next_step >= run.steps.len() && run.actors.iter().all(|a| !a.live);
            if done {
                off.extend(run.areas.iter().map(|area| (run.instance, *area)));
            }
            !done
        });
        for (instance, area) in off {
            if let Some(packet) = areas.set_area(area, false, unix_ms) {
                let sessions = players.sessions_in(instance).map(|s| s.session_id);
                deliver(&mut out, sessions, || packet);
            }
        }
        out
    }

    /// A session refused `undelivered`, its packets of one tick from the first refusal on, in
    /// order. The client must end up with no actor the runs no longer have, so:
    /// * a removal it missed stays owed: the next tick sends it again, first;
    /// * every actor it was shown is removed before a later tick shows it again, because the
    ///   client may hold the actor already (a refused base animation after an accepted AddNpc).
    ///   A 35/3 for a guid the client lacks is a no-op (its handler skips a missed lookup at
    ///   `0x00930808`), so removing an actor whose AddNpc was refused is harmless.
    pub fn refused(&mut self, session: SessionId, undelivered: Vec<Vec<u8>>) {
        let mut owed: Vec<_> = undelivered
            .into_iter()
            .filter(|packet| packet.starts_with(&REMOVAL_HEAD))
            .collect();
        for run in self.runs.values_mut() {
            for index in 0..run.actors.len() {
                if run.actors[index].seen_by.remove(&session) {
                    owed.push(encode_remove_gracefully(run.guid(index), 0));
                }
            }
        }
        for packet in owed {
            if !self
                .pending
                .iter()
                .any(|(to, held)| *to == session && *held == packet)
            {
                self.pending.push((session, packet));
            }
        }
    }
}

impl Run {
    fn guid(&self, index: usize) -> u64 {
        GUID_TAG | u64::from(self.serial) << 8 | index as u64
    }

    /// One step. An op the actor's state does not allow (anything but spawn before it is live,
    /// spawn while it is, a tag that is already on or not on) changes nothing, and neither does
    /// an op on an actor that was removed.
    fn apply(&mut self, step: Step, env: &mut Env, out: &mut Out) -> Result<(), &'static str> {
        let at = self.started_ms.saturating_add(step.at_ms);
        match step.op {
            Op::Broadcast(number) => {
                self.broadcast(number, env.players, out);
                return Ok(());
            }
            Op::Area(area, on) => {
                if on {
                    self.areas.insert(area);
                } else {
                    self.areas.remove(&area);
                }
                if let Some(packet) = env.areas.set_area(area, on, env.unix_ms) {
                    let sessions = env.players.sessions_in(self.instance);
                    deliver(out, sessions.map(|s| s.session_id), || packet);
                }
                return Ok(());
            }
            Op::Cycle(number) => {
                let until = self.event.cycles[number].until_ms;
                self.cycles.push(Playing {
                    cycle: number,
                    next_ms: at,
                    end_ms: at.saturating_add(until),
                    count: 0,
                });
                return Ok(());
            }
            _ => {}
        }
        let (index, guid) = (step.actor, self.guid(step.actor));
        let live = self.actors[index].live;
        if live == matches!(step.op, Op::Spawn(_)) {
            return Err(if live {
                "the actor is already live"
            } else {
                "the actor is not live"
            });
        }
        let shown = &mut self.actors[index];
        let packet = match step.op {
            Op::Spawn(effect) => {
                let actor = &self.event.actors[index];
                let (position, rotation) = (actor.position, actor.spawn.rotation);
                *shown = Shown {
                    live: true,
                    position,
                    rotation,
                    ..Shown::default()
                };
                self.view(index, env.players, out);
                if effect == 0 {
                    return Ok(());
                }
                encode_play_composite_effect(guid, effect, position)
            }
            Op::Remove(effect) => {
                let bytes = encode_remove_gracefully(guid, effect);
                let seen_by = std::mem::take(shown).seen_by;
                out.extend(seen_by.into_iter().map(|session| (session, bytes.clone())));
                return Ok(());
            }
            Op::BaseAnim(id) => {
                shown.base_anim = Some(id);
                animation(guid, id, true)
            }
            Op::PlayAnim(id) => animation(guid, id, false),
            Op::Effect(id) => encode_play_composite_effect(guid, id, shown.position),
            Op::TagOn(effect) => {
                if shown.tags.contains_key(&effect) {
                    return Err("the tag is already on");
                }
                // ponytail: one counter for every actor; a tag id repeats only after 2^31 tags.
                *env.tag = *env.tag % i32::MAX + 1;
                shown.tags.insert(effect, *env.tag);
                encode_add_composite_effect(guid, *env.tag, effect, guid)
            }
            Op::TagOff(effect) => match shown.tags.remove(&effect) {
                Some(tag) => encode_remove_composite_effect(guid, tag),
                None => return Err("the tag is not on"),
            },
            Op::Move { to, speed } => {
                shown.mover = Some(Mover {
                    to,
                    speed,
                    last_ms: at,
                });
                shown.base_anim = Some(ANIM_ID_WALK);
                animation(guid, ANIM_ID_WALK, true)
            }
            Op::Say(string_id) => {
                // Show first: a session that just came into range needs the speaker's AddNpc.
                self.view(index, env.players, out);
                let shown = &self.actors[index];
                let heard = shown.seen_by.iter().copied().filter(|session| {
                    env.players.get(*session).is_some_and(|player| {
                        within(player.position, shown.position, LOCAL_CHAT_RADIUS, false)
                    })
                });
                deliver(out, heard, || crate::chat::string_say(guid, string_id));
                return Ok(());
            }
            Op::Broadcast(_) | Op::Area(..) | Op::Cycle(_) => unreachable!("handled above"),
        };
        let seen_by = &self.actors[index].seen_by;
        out.extend(seen_by.iter().map(|session| (*session, packet.clone())));
        Ok(())
    }

    /// A 15/3 line to the instance, or to the sessions within the radius (on X/Z) of the first
    /// actor's current position (a radius needs a first actor: the loader checks).
    fn broadcast(&self, number: usize, players: &PlayerRegistry, out: &mut Out) {
        let Broadcast { text, radius_m } = &self.event.broadcasts[number];
        let around = radius_m.map(|radius| {
            let first = &self.actors[0];
            let center = if first.live {
                first.position
            } else {
                self.event.actors[0].position
            };
            (center, radius)
        });
        let heard = players.sessions_in(self.instance).filter(|player| {
            around.is_none_or(|(center, radius)| within(player.position, center, radius, true))
        });
        deliver(out, heard.map(|p| p.session_id), || {
            crate::chat::system(text)
        });
    }

    /// One opcode 125 step for a walking actor, at most once per tick, along the straight line
    /// to its target (no clamp: a stalled server catches up, and the last step lands on the
    /// target). Arrival sends the final position, then the actor's stand animation.
    fn advance(&mut self, index: usize, now_ms: u64, out: &mut Out) {
        let guid = self.guid(index);
        let stand = self.event.actors[index].spawn.stand_animation_id;
        let shown = &mut self.actors[index];
        let Some(Mover { to, speed, last_ms }) = shown.mover else {
            return;
        };
        let elapsed = now_ms.saturating_sub(last_ms);
        if elapsed == 0 {
            return;
        }
        let delta = [0, 1, 2].map(|axis| to[axis] - shown.position[axis]);
        let length = delta.iter().map(|d| d * d).sum::<f32>().sqrt();
        let step = speed * (elapsed as f32 / 1000.0);
        let flat = delta[0].hypot(delta[2]);
        if flat > f32::EPSILON {
            shown.rotation = [delta[0] / flat, 0.0, delta[2] / flat, 0.0];
        }
        // A non-finite length or step (checked at load, so unreachable) lands on the target.
        let arrived = !(length.is_finite() && step.is_finite()) || length <= step + 1e-3; // 1 mm of f32 drift
        shown.position = if arrived {
            to
        } else {
            [0, 1, 2].map(|axis| shown.position[axis] + delta[axis] * (step / length))
        };
        shown.mover = (!arrived).then_some(Mover {
            to,
            speed,
            last_ms: now_ms,
        });
        let heading = [shown.rotation[0], 0.0, shown.rotation[2]];
        let mut packets = vec![movement_packet(guid, shown.position, heading)];
        if arrived {
            shown.base_anim = Some(stand);
            packets.push(animation(guid, stand, true));
        }
        for session in &shown.seen_by {
            out.extend(packets.iter().map(|packet| (*session, packet.clone())));
        }
    }

    /// Play every cycle that is due, at most once each per tick: one animation per live actor
    /// of the cycle (the next id in sequence, or a random one each), then the next play after
    /// `every_ms` plus up to `jitter_ms`. A cycle ends after its last play inside `until_ms`.
    fn play_cycles(&mut self, now_ms: u64, out: &mut Out) {
        if self.cycles.is_empty() {
            return;
        }
        let event = Arc::clone(&self.event);
        let mut number = 0;
        while let Some(&playing) = self.cycles.get(number) {
            if playing.next_ms > now_ms {
                number += 1;
                continue;
            }
            let cycle = &event.cycles[playing.cycle];
            for &actor in &cycle.actors {
                if !self.actors[actor].live {
                    continue; // removed (or not yet spawned): nothing to animate
                }
                let pick = if cycle.sequence {
                    playing.count
                } else {
                    self.rng.next_u64() as usize
                };
                let id = cycle.ids[pick % cycle.ids.len()];
                let packet = animation(self.guid(actor), id, false);
                let seen_by = &self.actors[actor].seen_by;
                out.extend(seen_by.iter().map(|session| (*session, packet.clone())));
            }
            let jitter = match cycle.jitter_ms {
                0 => 0,
                jitter => self.rng.next_u64() % (jitter + 1),
            };
            let next_ms = playing
                .next_ms
                .saturating_add(cycle.every_ms)
                .saturating_add(jitter)
                .max(now_ms.saturating_add(1));
            if next_ms > playing.end_ms {
                self.cycles.remove(number);
            } else {
                self.cycles[number] = Playing {
                    next_ms,
                    count: playing.count + 1,
                    ..playing
                };
                number += 1;
            }
        }
    }

    /// Show actor `index` to nearby sessions it has not been shown to (AddNpc where it is now,
    /// its current base animation, its live tags) and forget sessions that left. One still in
    /// the instance gets the plain 35/3, so walking back shows the actor again.
    fn view(&mut self, index: usize, players: &PlayerRegistry, out: &mut Out) {
        let (guid, instance) = (self.guid(index), self.instance);
        let actor = &self.event.actors[index];
        let shown = &mut self.actors[index];
        let nearby = players.nearby_sessions(instance, shown.position);
        shown.seen_by.retain(|session| {
            let stays = nearby.contains(session);
            if !stays && players.get(*session).is_some_and(|p| p.zone == instance) {
                out.push((*session, encode_remove_gracefully(guid, 0)));
            }
            stays
        });
        if nearby.len() == shown.seen_by.len() {
            return; // seen_by is a subset of nearby: nobody new
        }
        let [x, y, z] = shown.position;
        let spawn = NpcSpawn {
            actor_id: guid,
            position: [x, y, z, 1.0],
            rotation: shown.rotation,
            ..actor.spawn.clone()
        };
        let mut packets: Vec<Vec<u8>> = crate::world::unframe_tunnel(&build_add_npc(&spawn))
            .into_iter()
            .collect();
        packets.extend(shown.base_anim.map(|id| animation(guid, id, true)));
        packets.extend(
            shown
                .tags
                .iter()
                .map(|(effect, tag)| encode_add_composite_effect(guid, *tag, *effect, guid)),
        );
        for session in nearby {
            if shown.seen_by.insert(session) {
                out.extend(packets.iter().map(|packet| (session, packet.clone())));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::players::InterestPolicy;
    use serde_json::json;

    const WORLD: ZoneInstance = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };

    const UNIX: u64 = 1_000_000;

    /// One tick with no client areas.
    fn tick(events: &mut WorldEvents, now_ms: u64, players: &PlayerRegistry) -> Out {
        events.tick(now_ms, UNIX, players, &mut ClientAreas::default())
    }

    fn actor(key: &str) -> serde_json::Value {
        json!({"key": key, "model_id": 1, "name_string_id": 0, "npc_name": "Event NPC",
            "position": [0.0, 0.0, 0.0], "heading": 0.0, "stand_animation_id": 1,
            "hide_nameplate": false, "source": "T458 test fixture"})
    }

    fn event(key: &str, timeline: serde_json::Value) -> serde_json::Value {
        json!({"key": key, "actors": [actor("a")], "timeline": timeline})
    }

    fn load_json(name: &str, file: &serde_json::Value) -> io::Result<Vec<WorldEvent>> {
        // One directory per test thread: tests run in parallel and share the helper names.
        let thread = format!("{:?}", std::thread::current().id()).replace(['(', ')'], "");
        let dir =
            std::env::temp_dir().join(format!("osfr-t458-{}-{thread}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE), file.to_string()).unwrap();
        let events = load(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        events
    }

    fn one(timeline: serde_json::Value) -> WorldEvent {
        let file = json!({"events": [event("show", timeline)]});
        load_json("one", &file).unwrap().pop().unwrap()
    }

    fn registry_with(session: u64, position: [f32; 3]) -> PlayerRegistry {
        let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
        players
            .join(SessionId(session), session, session, WORLD, position, 0.0)
            .unwrap();
        players
    }

    #[test]
    fn a_missing_file_is_no_events_and_the_shipped_file_loads() {
        let missing = std::env::temp_dir().join("osfr-t458-no-such-directory");
        assert!(load(&missing).unwrap().is_empty());
        let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        assert!(
            load(&shipped).unwrap().is_empty(),
            "the shipped file has no events"
        );
    }

    #[test]
    fn the_loader_refuses_everything_outside_the_bounds() {
        let spawn = json!([{"at_ms": 0, "op": {"spawn": {"actor": "a"}}}]);
        assert!(load_json("valid", &json!({"events": [event("show", spawn.clone())]})).is_ok());
        let many = |count: usize, item: serde_json::Value| vec![item; count];
        let steps = |count| json!(many(count, spawn[0].clone()));
        let actors = |count: usize| {
            json!(
                (0..count)
                    .map(|i| actor(&format!("a{i}")))
                    .collect::<Vec<_>>()
            )
        };
        let mut with_actors = event("show", json!([]));
        with_actors["actors"] = actors(33);
        let mut renamed = event("show", spawn.clone());
        renamed["actors"][0]["npc_name"] = json!("n".repeat(65));
        let mut no_source = event("show", spawn.clone());
        no_source["actors"][0]["source"] = json!("");
        let mut infinite = event("show", spawn.clone());
        infinite["actors"][0]["position"] = json!([1e39, 0.0, 0.0]);
        let mut model = event("show", spawn.clone());
        model["actors"][0]["model_id"] = json!(0);
        let mut extra = event("show", spawn.clone());
        extra["actors"][0]["scale"] = json!(1.0);
        let events_33: Vec<_> = (0..33)
            .map(|i| event(&format!("e{i}"), json!([])))
            .collect();
        let cases = [
            ("33 events", json!({"events": events_33})),
            ("33 actors", json!({"events": [with_actors]})),
            ("513 steps", json!({"events": [event("show", steps(513))]})),
            (
                "late step",
                json!({"events": [event("show", json!([{"at_ms": MAX_AT_MS + 1, "op": {"spawn": {"actor": "a"}}}]))]}),
            ),
            (
                "zero effect",
                json!({"events": [event("show", json!([{"at_ms": 0, "op": {"spawn": {"actor": "a", "effect_id": 0}}}]))]}),
            ),
            (
                "zero animation",
                json!({"events": [event("show", json!([{"at_ms": 0, "op": {"base_anim": {"actor": "a", "id": 0}}}]))]}),
            ),
            (
                "unknown actor",
                json!({"events": [event("show", json!([{"at_ms": 0, "op": {"spawn": {"actor": "b"}}}]))]}),
            ),
            (
                "unknown op field",
                json!({"events": [event("show", json!([{"at_ms": 0, "op": {"spawn": {"actor": "a", "id": 3}}}]))]}),
            ),
            (
                "unknown op",
                json!({"events": [event("show", json!([{"at_ms": 0, "op": {"walk": {"actor": "a"}}}]))]}),
            ),
            ("unknown actor field", json!({"events": [extra]})),
            ("model 0", json!({"events": [model]})),
            ("long name", json!({"events": [renamed]})),
            ("no source", json!({"events": [no_source]})),
            ("infinite position", json!({"events": [infinite]})),
            (
                "uppercase key",
                json!({"events": [event("Show", json!([]))]}),
            ),
            (
                "twice",
                json!({"events": [event("show", json!([])), event("show", json!([]))]}),
            ),
            ("no events field", json!({"event": []})),
        ];
        for (label, file) in cases {
            assert!(load_json(label, &file).is_err(), "{label} was accepted");
        }
        let at_bounds = json!({"events": [event("show", steps(512))]});
        assert_eq!(
            load_json("512 steps", &at_bounds).unwrap()[0].steps.len(),
            512
        );
    }

    /// An op on an actor that is not live (or a tag that is not on) is skipped and logged; the
    /// run keeps going and nothing is sent for it.
    #[test]
    fn an_op_the_actor_state_does_not_allow_is_skipped() {
        let event = one(json!([
            {"at_ms": 0, "op": {"base_anim": {"actor": "a", "id": 7}}},
            {"at_ms": 0, "op": {"tag_off": {"actor": "a", "effect_id": 50}}},
            {"at_ms": 0, "op": {"remove": {"actor": "a"}}},
            {"at_ms": 10, "op": {"spawn": {"actor": "a"}}},
            {"at_ms": 10, "op": {"spawn": {"actor": "a"}}},
            {"at_ms": 20, "op": {"tag_on": {"actor": "a", "effect_id": 50}}},
            {"at_ms": 20, "op": {"tag_on": {"actor": "a", "effect_id": 50}}},
            {"at_ms": 30, "op": {"remove": {"actor": "a"}}}
        ]));
        let players = registry_with(1, [1.0, 0.0, 0.0]);
        let mut events = WorldEvents::default();
        assert!(events.start(event, WORLD, 1_000));
        assert!(
            tick(&mut events, 1_000, &players).is_empty(),
            "nothing is live yet"
        );
        let spawned = tick(&mut events, 1_010, &players);
        assert_eq!(spawned.len(), 1, "one AddNpc, not two");
        assert_eq!(&spawned[0].1[..4], &[35, 0, 2, 0]);
        let tagged = tick(&mut events, 1_020, &players);
        assert_eq!(tagged.len(), 1, "one 35/41, not two");
        assert_eq!(&tagged[0].1[..4], &[35, 0, 41, 0]);
        assert_eq!(tick(&mut events, 1_030, &players).len(), 1, "the removal");
        assert_eq!(
            events,
            WorldEvents {
                serial: 1,
                tag: 1,
                ..WorldEvents::default()
            }
        );
    }

    #[test]
    fn a_wrapped_serial_never_reuses_a_live_guid() {
        let spawn = json!([{"at_ms": 0, "op": {"spawn": {"actor": "a"}}}]);
        let file = json!({"events": [event("one", spawn.clone()), event("two", spawn)]});
        let mut loaded = load_json("wrap", &file).unwrap();
        let (two, one) = (loaded.pop().unwrap(), loaded.pop().unwrap());
        let players = registry_with(1, [1.0, 0.0, 0.0]);
        let mut events = WorldEvents::default();
        assert!(events.start(one, WORLD, 0));
        let first = tick(&mut events, 0, &players);
        events.serial = SERIALS; // the next serial wraps to 1, which "one" holds
        assert!(events.start(two, WORLD, 0));
        let second = tick(&mut events, 0, &players);
        let guid = |packets: &Out| u64::from_le_bytes(packets[0].1[4..12].try_into().unwrap());
        assert_eq!(guid(&first), GUID_TAG | 1 << 8);
        assert_eq!(
            guid(&second),
            GUID_TAG | 2 << 8,
            "the wrap skipped the live serial"
        );
    }

    const GUID: u64 = GUID_TAG | 1 << 8;

    fn kinds(out: &Out) -> Vec<&[u8]> {
        out.iter().map(|(_, packet)| &packet[..4]).collect()
    }

    /// Finding 3: a session that took the AddNpc but refused the base animation is forgotten;
    /// it must get the removal first, never a second AddNpc for a guid it still holds.
    #[test]
    fn a_forgotten_session_is_removed_before_it_is_shown_again() {
        let event = one(json!([
            {"at_ms": 0, "op": {"spawn": {"actor": "a"}}},
            {"at_ms": 10, "op": {"base_anim": {"actor": "a", "id": 7}}}
        ]));
        let players = registry_with(1, [1.0, 0.0, 0.0]);
        let mut events = WorldEvents::default();
        assert!(events.start(event, WORLD, 0));
        assert_eq!(kinds(&tick(&mut events, 0, &players)), [[35, 0, 2, 0]]);
        let animated = tick(&mut events, 10, &players);
        assert_eq!(kinds(&animated), [[35, 0, 8, 0]]);
        events.refused(SessionId(1), vec![animated[0].1.clone()]);
        let again = tick(&mut events, 20, &players);
        assert_eq!(
            kinds(&again),
            [[35, 0, 3, 0], [35, 0, 2, 0], [35, 0, 8, 0]],
            "the removal, then the AddNpc and the animation"
        );
        assert_eq!(again[0].1, encode_remove_gracefully(GUID, 0));
        assert_eq!(&again[1].1[4..12], &GUID.to_le_bytes());
        assert!(
            tick(&mut events, 30, &players).is_empty(),
            "shown once, then quiet"
        );
    }

    /// Finding 3: a removal a session refused (a `remove` step, a `stop`, or the walk-out
    /// removal) is sent again until it is accepted, once per tick however often it is refused.
    #[test]
    fn a_refused_removal_stays_owed_until_it_is_delivered() {
        let event = one(json!([
            {"at_ms": 0, "op": {"spawn": {"actor": "a"}}},
            {"at_ms": 10, "op": {"remove": {"actor": "a", "effect_id": 5}}}
        ]));
        let players = registry_with(1, [1.0, 0.0, 0.0]);
        let mut events = WorldEvents::default();
        assert!(events.start(event, WORLD, 0));
        tick(&mut events, 0, &players);
        let removal = encode_remove_gracefully(GUID, 5);
        let sent = tick(&mut events, 10, &players);
        assert_eq!(sent, [(SessionId(1), removal.clone())]);
        // The run is over, yet the refused removal comes back, once, however often refused.
        events.refused(SessionId(1), sent.iter().map(|(_, p)| p.clone()).collect());
        events.refused(SessionId(1), vec![removal.clone()]);
        assert_eq!(
            tick(&mut events, 20, &players),
            [(SessionId(1), removal.clone())]
        );
        events.refused(SessionId(1), vec![removal.clone()]);
        assert_eq!(tick(&mut events, 30, &players), [(SessionId(1), removal)]);
        assert!(
            tick(&mut events, 40, &players).is_empty(),
            "delivered, so no longer owed"
        );
    }

    /// The same for `stop`: a refused removal is owed even though the run is gone.
    #[test]
    fn a_refused_stop_removal_is_not_lost() {
        let event = one(json!([{"at_ms": 0, "op": {"spawn": {"actor": "a"}}}]));
        let players = registry_with(1, [1.0, 0.0, 0.0]);
        let mut events = WorldEvents::default();
        assert!(events.start(event, WORLD, 0));
        tick(&mut events, 0, &players);
        assert!(events.stop("show"));
        let sent = tick(&mut events, 10, &players);
        assert_eq!(sent, [(SessionId(1), encode_remove_gracefully(GUID, 0))]);
        events.refused(SessionId(1), vec![sent[0].1.clone()]);
        assert_eq!(tick(&mut events, 20, &players), sent);
        assert!(tick(&mut events, 30, &players).is_empty());
    }

    /// The bound is checked on the file's size before it is read, and the error names the file.
    #[test]
    fn an_oversized_file_is_refused_by_name_before_it_is_read() {
        let dir = std::env::temp_dir().join(format!("osfr-t458-{}-big", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let padded = |extra: u64| format!(r#"{{"events": []}}{}"#, " ".repeat((extra) as usize));
        let pad = MAX_JSON_BYTES - 14; // `{"events": []}` is 14 bytes
        std::fs::write(dir.join(FILE), padded(pad)).unwrap();
        assert!(load(&dir).unwrap().is_empty(), "exactly the bound loads");
        std::fs::write(dir.join(FILE), padded(pad + 1)).unwrap();
        let error = load(&dir).unwrap_err().to_string();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(error.contains(FILE) && error.contains("larger"), "{error}");
    }

    /// With no events configured a tick sends nothing and allocates nothing.
    #[test]
    fn an_idle_tick_allocates_nothing() {
        let players = registry_with(1, [1.0, 0.0, 0.0]);
        let mut events = WorldEvents::default();
        let out = tick(&mut events, 5_000, &players);
        assert!(out.is_empty() && out.capacity() == 0);
    }

    // ---- T460: part 2 ----

    const OTHER: ZoneInstance = ZoneInstance {
        zone_id: 5,
        instance_id: 0,
    };
    fn registry(sessions: &[(u64, ZoneInstance, [f32; 3])]) -> PlayerRegistry {
        let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
        for &(id, zone, position) in sessions {
            players
                .join(SessionId(id), id, id, zone, position, 0.0)
                .unwrap();
        }
        players
    }

    /// One actor "a" at `position` and `timeline`.
    fn event_at(position: [f32; 3], timeline: serde_json::Value) -> WorldEvent {
        let mut row = event("show", timeline);
        row["actors"][0]["position"] = json!(position);
        let file = json!({"events": [row]});
        load_json("at", &file).unwrap().pop().unwrap()
    }

    fn to(out: &Out, session: u64) -> Vec<Vec<u8>> {
        out.iter()
            .filter(|(id, _)| id.0 == session)
            .map(|(_, packet)| packet.clone())
            .collect()
    }

    fn xyz(packet: &[u8]) -> [f32; 3] {
        let at = |n: usize| f32::from_le_bytes(packet[10 + 4 * n..14 + 4 * n].try_into().unwrap());
        [at(0), at(1), at(2)]
    }

    fn spawn_a() -> serde_json::Value {
        json!({"at_ms": 0, "op": {"spawn": {"actor": "a"}}})
    }

    /// An actor at 1.5 m/s covers 15 m in ten 1 s ticks along the straight line, faces its
    /// target, ends exactly on it and switches to its stand animation. A viewer who arrives
    /// halfway gets the AddNpc at the actor's current position.
    #[test]
    fn a_walking_actor_covers_its_path_one_tick_at_a_time_and_stands_on_arrival() {
        let walk = json!({"at_ms": 0, "op": {"move": {"actor": "a", "to": [115.0, 0.0, 200.0],
            "speed_mps": 1.5}}});
        let event = event_at([100.0, 0.0, 200.0], json!([spawn_a(), walk]));
        let mut players = registry(&[(1, WORLD, [101.0, 0.0, 200.0])]);
        let mut events = WorldEvents::default();
        assert!(events.start(event, WORLD, 0));
        let started = tick(&mut events, 0, &players);
        assert_eq!(started.len(), 2, "AddNpc, then the walk animation");
        assert_eq!(
            started[1].1,
            piece_052::animation_packet(GUID, ANIM_ID_WALK, true)
        );
        for second in 1..=10_u64 {
            if second == 5 {
                players
                    .join(SessionId(2), 2, 2, WORLD, [107.0, 0.0, 200.0], 0.0)
                    .unwrap();
            }
            let out = tick(&mut events, second * 1_000, &players);
            let steps = to(&out, 1);
            let expected = 100.0 + 1.5 * second as f32;
            assert!((xyz(&steps[0])[0] - expected).abs() < 1e-3, "tick {second}");
            assert_eq!(steps[0][..2], [125, 0], "an opcode 125 step");
            assert_eq!(
                steps[0][22..34],
                [1.0_f32, 0.0, 0.0]
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>(),
                "the heading points at the target"
            );
            let arrival = piece_052::animation_packet(GUID, 1, true);
            match second {
                10 => {
                    assert_eq!(xyz(&steps[0]), [115.0, 0.0, 200.0], "ends on the target");
                    assert_eq!(steps[1..], [arrival]);
                }
                5 => {
                    assert_eq!(steps.len(), 1);
                    let late = to(&out, 2);
                    let near = |value: f32| {
                        late[0].windows(4).any(|w| {
                            (f32::from_le_bytes(w.try_into().unwrap()) - value).abs() < 0.01
                        })
                    };
                    assert!(near(107.5) && !near(100.0), "the late AddNpc is at 107.5");
                    assert_eq!(
                        late[1],
                        piece_052::animation_packet(GUID, ANIM_ID_WALK, true)
                    );
                }
                _ => assert_eq!(steps.len(), 1),
            }
        }
        assert!(
            tick(&mut events, 11_000, &players).is_empty(),
            "no more steps"
        );
    }

    /// `say` reaches the sessions within 30 m (3-D) of the speaker and nobody else; with no
    /// session in range, nothing is sent and nothing allocated.
    #[test]
    fn say_reaches_only_the_sessions_within_thirty_metres() {
        let say = json!({"at_ms": 100, "op": {"say": {"actor": "a", "string_id": 432_431}}});
        let players = registry(&[
            (1, WORLD, [29.0, 0.0, 0.0]),
            (2, WORLD, [31.0, 0.0, 0.0]),
            (3, WORLD, [0.0, 20.0, 25.0]), // 25 m on the ground, 32 m in 3-D
            (4, OTHER, [1.0, 0.0, 0.0]),
        ]);
        let mut events = WorldEvents::default();
        assert!(events.start(
            event_at([0.0; 3], json!([spawn_a(), say.clone()])),
            WORLD,
            0
        ));
        assert_eq!(
            tick(&mut events, 0, &players).len(),
            3,
            "an AddNpc each for 1, 2 and 3"
        );
        let out = tick(&mut events, 100, &players);
        assert_eq!(
            out,
            [(SessionId(1), crate::chat::string_say(GUID, 432_431))]
        );
        assert_eq!(out[0].1[..4], [15, 0, 4, 0]);

        let far = registry(&[(1, WORLD, [500.0, 0.0, 0.0])]);
        let mut events = WorldEvents::default();
        assert!(events.start(event_at([0.0; 3], json!([spawn_a(), say])), WORLD, 0));
        for now in [0, 100] {
            let out = tick(&mut events, now, &far);
            assert!(
                out.is_empty() && out.capacity() == 0,
                "nothing sent or allocated"
            );
        }
    }

    /// `broadcast` without a radius reaches the event's instance only; with `radius_m` it
    /// reaches the sessions inside the radius on X/Z of the first actor.
    #[test]
    fn broadcast_reaches_the_instance_or_the_radius_on_the_ground_plane() {
        let line = "Let the Dance-off begin!";
        let say = |radius: serde_json::Value| {
            let mut op = json!({"text": line, "locale_id": 432_431});
            op["radius_m"] = radius;
            json!([{"at_ms": 0, "op": {"broadcast": op}}])
        };
        let players = registry(&[
            (1, WORLD, [10.0, 0.0, 0.0]),
            (2, WORLD, [500.0, 0.0, 0.0]),
            (3, WORLD, [20.0, 1_000.0, 0.0]), // 20 m on X/Z, 1 km up
            (4, OTHER, [10.0, 0.0, 0.0]),
        ]);
        let packet = crate::chat::system(line);
        for (radius, heard) in [(json!(null), vec![1, 2, 3]), (json!(50.0), vec![1, 3])] {
            let mut events = WorldEvents::default();
            assert!(events.start(event_at([0.0; 3], say(radius.clone())), WORLD, 0));
            let out = tick(&mut events, 0, &players);
            let expected: Vec<_> = heard
                .iter()
                .map(|id| (SessionId(*id), packet.clone()))
                .collect();
            assert_eq!(out, expected, "radius {radius}");
        }
    }

    fn areas_with(switchable: &[i32]) -> ClientAreas {
        let dir = std::env::temp_dir().join(format!("osfr-t460-areas-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let content = json!({"schema_version": 1,
            "switchable": {"area_ids": switchable, "source": "T460 test"}});
        std::fs::write(dir.join("client_areas.json"), content.to_string()).unwrap();
        let areas = ClientAreas::load(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        areas.unwrap()
    }

    fn area_packet(area: i32, on: bool, elapsed: i32) -> Vec<u8> {
        let mut packet = vec![89, 0];
        packet.extend(area.to_le_bytes());
        packet.push(u8::from(on));
        packet.extend(elapsed.to_le_bytes());
        packet
    }

    fn area_step(at_ms: u64, on: bool) -> serde_json::Value {
        json!({"at_ms": at_ms, "op": {"area": {"area_id": 4299, "on": on}}})
    }

    /// `area` on and off reach every session in the open world and nobody in another zone; a
    /// session that loads mid-song gets the elapsed time from `ClientAreas`; stopping early, or
    /// the run ending with the area still on, switches the area off.
    #[test]
    fn areas_reach_the_open_world_and_never_stay_on_after_the_run() {
        let players = registry(&[
            (1, WORLD, [1.0, 0.0, 0.0]),
            (2, WORLD, [500.0, 0.0, 0.0]),
            (3, OTHER, [1.0, 0.0, 0.0]),
        ]);
        let go = |events: &mut WorldEvents, areas: &mut ClientAreas, now: u64, unix: u64| {
            events.tick(now, unix, &players, areas)
        };
        let on = |elapsed| area_packet(4299, true, elapsed);
        let off = area_packet(4299, false, 0);
        let both = |packet: &Vec<u8>| {
            vec![
                (SessionId(1), packet.clone()),
                (SessionId(2), packet.clone()),
            ]
        };

        // On at 0, off at 1 s; a session that loads 30 s in gets the song's elapsed time.
        let mut areas = areas_with(&[4299]);
        let mut events = WorldEvents::default();
        assert!(events.start(
            event_at(
                [0.0; 3],
                json!([area_step(0, true), area_step(1_000, false)])
            ),
            WORLD,
            0
        ));
        assert_eq!(go(&mut events, &mut areas, 0, UNIX), both(&on(0)));
        assert_eq!(areas.world_ready_packets(UNIX + 30_000), [on(30_000)]);
        assert_eq!(go(&mut events, &mut areas, 1_000, UNIX + 1_000), both(&off));
        assert!(areas.world_ready_packets(UNIX + 30_000).is_empty());

        // Stopped early: the run's areas go off at the next tick, once.
        let keep = json!([area_step(0, true), spawn_a_at(100_000)]);
        assert!(events.start(event_at([0.0; 3], keep), WORLD, 5_000));
        assert_eq!(go(&mut events, &mut areas, 5_000, UNIX), both(&on(0)));
        assert!(events.stop("show"));
        assert_eq!(go(&mut events, &mut areas, 5_050, UNIX + 50), both(&off));
        assert!(go(&mut events, &mut areas, 5_100, UNIX + 100).is_empty());
        assert!(areas.world_ready_packets(UNIX + 100).is_empty());

        // Ends on its own with the area still on: switched off with the last actor.
        let timeline = json!([spawn_a(), area_step(0, true),
            {"at_ms": 500, "op": {"remove": {"actor": "a"}}}]);
        assert!(events.start(event_at([0.0; 3], timeline), WORLD, 9_000));
        go(&mut events, &mut areas, 9_000, UNIX);
        let out = go(&mut events, &mut areas, 9_500, UNIX + 500);
        assert_eq!(out.last(), Some(&(SessionId(2), off.clone())), "{out:?}");
        assert!(areas.world_ready_packets(UNIX + 500).is_empty());
        assert!(events.runs.is_empty());
    }

    fn spawn_a_at(at_ms: u64) -> serde_json::Value {
        json!({"at_ms": at_ms, "op": {"spawn": {"actor": "a"}}})
    }

    /// The anim ids session 1 sees, by tick time, for `timeline` run with `seed`.
    fn animations(timeline: serde_json::Value, seed: u64, until_ms: u64) -> Vec<(u64, i32)> {
        let players = registry(&[(1, WORLD, [1.0, 0.0, 0.0])]);
        let mut events = WorldEvents::default();
        assert!(events.begin(event_at([0.0; 3], timeline), WORLD, 0, Some(seed)));
        let mut seen = Vec::new();
        for now in (0..=until_ms).step_by(100) {
            for packet in to(&tick(&mut events, now, &players), 1) {
                if packet[..4] == [35, 0, 8, 0] && packet[20] == 0 {
                    seen.push((now, i32::from_le_bytes(packet[12..16].try_into().unwrap())));
                }
            }
        }
        seen
    }

    fn cycle(order: &str, jitter_ms: u64) -> serde_json::Value {
        json!([spawn_a(), {"at_ms": 0, "op": {"cycle_anim": {"actors": ["a"],
            "ids": [10, 11, 12], "every_ms": 1000, "jitter_ms": jitter_ms,
            "until_ms": 5000, "order": order}}}])
    }

    /// `cycle_anim`: in sequence it plays the ids in order and stops at `until_ms`; random with
    /// jitter is a pure function of the seed, and every gap is `every_ms` plus up to the jitter.
    #[test]
    fn cycle_anim_is_deterministic_for_a_fixed_seed() {
        let plays = animations(cycle("sequence", 0), 1, 8_000);
        assert_eq!(
            plays,
            [
                (0, 10),
                (1_000, 11),
                (2_000, 12),
                (3_000, 10),
                (4_000, 11),
                (5_000, 12)
            ]
        );
        let random = |seed| animations(cycle("random", 400), seed, 8_000);
        assert_eq!(random(7), random(7));
        assert_ne!(random(7), random(8), "another seed, another show");
        let times: Vec<u64> = random(7).iter().map(|(at, _)| *at).collect();
        assert!(times.len() >= 4 && times.iter().all(|at| *at <= 5_000));
        assert!(
            times
                .windows(2)
                .all(|w| (900..=1_500).contains(&(w[1] - w[0]))),
            "{times:?}"
        );
    }

    /// `one_of` draws one branch at the run's start from the seed; branch offsets count from the
    /// `one_of` step. Both branches run end to end, and a seed always draws the same one.
    #[test]
    fn one_of_runs_each_branch_end_to_end_and_is_stable_for_a_seed() {
        let branch =
            |id: i32| json!([{"at_ms": 500, "op": {"play_anim": {"actor": "a", "id": id}}}]);
        let timeline = json!([spawn_a(), {"at_ms": 1_000, "op": {"one_of": {
            "branches": [branch(21), branch(22), []]}}}]);
        let drawn = |seed| animations(timeline.clone(), seed, 3_000);
        let mut firsts = BTreeMap::new();
        for seed in 0..24 {
            let plays = drawn(seed);
            assert_eq!(
                plays,
                drawn(seed),
                "seed {seed} draws the same branch twice"
            );
            if let Some(&(at, id)) = plays.first() {
                assert_eq!((plays.len(), at), (1, 1_500), "offset from the one_of step");
                firsts.insert(id, seed);
            } else {
                firsts.insert(0, seed); // the empty branch
            }
        }
        assert_eq!(firsts.keys().copied().collect::<Vec<_>>(), [0, 21, 22]);
    }

    /// An op that names a removed actor (or one that never spawned) is skipped without a panic
    /// and sends nothing; the run goes on.
    #[test]
    fn ops_on_a_removed_actor_are_skipped() {
        let after = |at, op: serde_json::Value| json!({"at_ms": at, "op": op});
        let timeline = json!([
            after(
                0,
                json!({"move": {"actor": "a", "to": [5.0, 0.0, 5.0], "speed_mps": 1.0}})
            ),
            spawn_a_at(50),
            after(100, json!({"remove": {"actor": "a"}})),
            after(200, json!({"say": {"actor": "a", "string_id": 5}})),
            after(
                200,
                json!({"move": {"actor": "a", "to": [5.0, 0.0, 5.0], "speed_mps": 1.0}})
            ),
            after(
                200,
                json!({"cycle_anim": {"actors": ["a"], "ids": [3], "every_ms": 100,
                "jitter_ms": 0, "until_ms": 500, "order": "random"}})
            ),
        ]);
        let players = registry(&[(1, WORLD, [1.0, 0.0, 0.0])]);
        let mut events = WorldEvents::default();
        assert!(events.start(event_at([0.0; 3], timeline), WORLD, 0));
        let out = tick(&mut events, 10, &players);
        assert!(out.is_empty(), "a move before the spawn is skipped");
        assert_eq!(tick(&mut events, 50, &players).len(), 1, "the AddNpc");
        assert_eq!(tick(&mut events, 100, &players).len(), 1, "the removal");
        for now in (200..=1_000).step_by(50) {
            assert!(tick(&mut events, now, &players).is_empty(), "tick {now}");
        }
        assert!(events.runs.is_empty());
    }

    /// Everything outside the new ops' bounds is refused at load.
    #[test]
    fn the_loader_refuses_bad_part_two_ops() {
        let play = json!({"play_anim": {"actor": "a", "id": 1}});
        let step = |op: serde_json::Value| json!([{"at_ms": 0, "op": op}]);
        let walk = |to: serde_json::Value, speed: f64| {
            step(json!({"move": {"actor": "a", "to": to, "speed_mps": speed}}))
        };
        let say = |text: &str, locale: i32, radius: serde_json::Value| {
            step(json!({"broadcast": {"text": text, "locale_id": locale, "radius_m": radius}}))
        };
        let cycle = |every: u64, ids: serde_json::Value| {
            step(
                json!({"cycle_anim": {"actors": ["a"], "ids": ids, "every_ms": every,
                "jitter_ms": 0, "until_ms": 1000, "order": "random"}}),
            )
        };
        let one_of = |branches: serde_json::Value| step(json!({"one_of": {"branches": branches}}));
        let ok = [
            walk(json!([1.0, 0.0, 1.0]), 6.25),
            walk(json!([1.0, 0.0, 1.0]), 0.1),
            say("hi", 1, json!(null)),
            say("hi", 1, json!(9_000.0)),
            step(json!({"say": {"actor": "a", "string_id": 1}})),
            step(json!({"area": {"area_id": 1, "on": true}})),
            cycle(100, json!([1])),
            one_of(json!([[{"at_ms": 1, "op": play.clone()}], []])),
        ];
        for (number, timeline) in ok.into_iter().enumerate() {
            let file = json!({"events": [event("show", timeline)]});
            assert!(load_json("ok", &file).is_ok(), "valid case {number}");
        }
        let bad = [
            ("speed below", walk(json!([1.0, 0.0, 1.0]), 0.05)),
            ("speed above", walk(json!([1.0, 0.0, 1.0]), 6.5)),
            ("infinite target", walk(json!([1e39, 0.0, 1.0]), 1.0)),
            ("far target", walk(json!([2e6, 0.0, 1.0]), 1.0)),
            ("empty text", say("  ", 1, json!(null))),
            ("control text", say("a\nb", 1, json!(null))),
            ("long text", say(&"x".repeat(257), 1, json!(null))),
            ("no locale", say("hi", 0, json!(null))),
            ("zero radius", say("hi", 1, json!(0.0))),
            ("huge radius", say("hi", 1, json!(10_001.0))),
            (
                "say id",
                step(json!({"say": {"actor": "a", "string_id": 0}})),
            ),
            ("area id", step(json!({"area": {"area_id": 0, "on": true}}))),
            ("fast cycle", cycle(99, json!([1]))),
            ("no cycle ids", cycle(1000, json!([]))),
            ("zero cycle id", cycle(1000, json!([0]))),
            ("no branches", one_of(json!([]))),
            (
                "nested",
                one_of(json!([[{"at_ms": 0, "op": {"one_of": {"branches": [[]]}}}]])),
            ),
            (
                "branch past 2 h",
                json!([{"at_ms": 1, "op": {"one_of": {"branches": [[{"at_ms": MAX_AT_MS, "op": play}]]}}}]),
            ),
        ];
        for (label, timeline) in bad {
            let file = json!({"events": [event("show", timeline)]});
            assert!(load_json(label, &file).is_err(), "{label} accepted");
        }
        // A radius needs a first actor to measure from.
        let mut empty = event("show", say("hi", 1, json!(10.0)));
        empty["actors"] = json!([]);
        assert!(load_json("no actors", &json!({"events": [empty]})).is_err());
    }
}
