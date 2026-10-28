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
//! T464 adds `face` (turn in place) and ships the Halloween dance-off as the first scheduled event.
//!
//! T538, participation: an event row's optional `dance_off` block names a Voting Box actor, the
//! two costume teams (client item name ids), their vote rows, the winning cakes and the lines
//! they show. A character standing in the block's area in a team's costume joins that team for
//! the run (keyed by character guid, so a disconnect keeps it); a vote is one per character; the
//! side with more votes wins the run's `one_of` ending, and a tie or no votes keeps the draw the
//! run made at its start. The winning cake answers one 15/4 line per participant, once. The
//! interaction and vote packets are answered by the bin's event-guid route; `gameplay/npc.rs`
//! answers a 26/6 only when it sent the dialogue, so it never closes an event actor's prompt.
//!
//! T462, part 3: an event row's optional `schedule` starts it on wall-clock slots from the world
//! tick (`schedule.rs` holds the slot rules): season gate, overlap skips, a kickoff line before
//! the start. Admin start and stop stay as they were; a slot whose event is already running
//! (admin-started, say) is skipped, not queued.
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
//! * face (T464): the `move` arrival packet without the walk: opcode 125 at the actor's current
//!   position with the heading toward the target (R: the original script turns a crew on arrival).
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
use crate::quest_engine::DanceTeam;
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
mod schedule;
use schedule::{Due, Entry, Schedule, ScheduleRow};

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
/// T538: at most this many costume name ids per team.
const MAX_COSTUME_IDS: usize = 16;
/// T538: bounds on the content's join scan cadence (server tuning, not a client value).
const MIN_JOIN_SCAN_MS: u64 = 50;
const MAX_JOIN_SCAN_MS: u64 = 10_000;
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
    /// Rows stay JSON until compiled, so `load_scheduled` can drop one broken row alone.
    events: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EventRow {
    key: String,
    actors: Vec<ActorRow>,
    timeline: Vec<StepRow>,
    schedule: Option<ScheduleRow>,
    /// T538: the run's costume teams, Voting Box, cakes and lines.
    #[serde(default)]
    dance_off: Option<DanceOffRow>,
}

/// One row's dance-off block: the area, the Voting Box, the prompt and refusal lines, and the
/// two teams with their costume name ids and cakes.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DanceOffRow {
    /// The area a costume joins in; the run's instance is the event's instance.
    center: [f32; 3],
    radius_m: f32,
    /// The interactable actor that offers the vote.
    voting_box: String,
    /// How often the world tick scans this run's area for costume joins (server tuning, not a
    /// client value; the loader bounds it).
    join_scan_ms: u64,
    /// Client locale id of the 26/3 prompt ("Which side..?").
    prompt_string_id: i32,
    /// Client locale id of the 15/4 line for a second vote.
    already_voted_string_id: i32,
    /// Client locale id of the 15/4 line for a character who never joined a team.
    not_participant_string_id: i32,
    /// Client locale id of the 15/4 line for a second cake use.
    used_cake_string_id: i32,
    teams: Vec<DanceTeamRow>,
    /// Provenance for the block (client ids, notes, evidence); required, never sent.
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DanceTeamRow {
    team: DanceTeam,
    /// Client item name ids (`ClientItemDefinition.NameId`) that put a character on this team.
    costume_name_ids: Vec<i32>,
    /// The interactable cake actor; its spawn in a `one_of` branch is this team's ending.
    cake_actor: String,
    /// The client 26/6 row id this team's vote row answers with (R: above the NPC rows;
    /// content data, not a protocol constant).
    vote_response_id: i32,
    /// Client locale id of the vote row's label.
    vote_label_string_id: i32,
    /// Client locale id of the 15/4 line one cake use answers.
    cake_say_string_id: i32,
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
    /// The line, copied as text; `locale_id` is its client locale provenance and is never sent.
    /// Left out, the line is a reconstruction (R) with no client locale entry (T493).
    Broadcast {
        text: String,
        locale_id: Option<i32>,
        radius_m: Option<f32>,
    },
    Move {
        actor: String,
        to: [f32; 3],
        speed_mps: f32,
    },
    /// Turn in place toward `to` (on X/Z).
    Face {
        actor: String,
        to: [f32; 3],
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
    schedule: Option<Schedule>,
    /// T538: `None` for an event nobody joins.
    dance_off: Option<DanceOff>,
}

/// One compiled dance-off. Teams are indexed by [`DanceTeam::index`].
#[derive(Debug, PartialEq)]
struct DanceOff {
    center: [f32; 3],
    radius_m: f32,
    voting_box: usize,
    join_scan_ms: u64,
    prompt_string_id: i32,
    already_voted_string_id: i32,
    not_participant_string_id: i32,
    used_cake_string_id: i32,
    teams: [TeamCostume; 2],
}

#[derive(Debug, PartialEq)]
struct TeamCostume {
    team: DanceTeam,
    costume_name_ids: Vec<i32>,
    cake: usize,
    vote_response_id: i32,
    vote_label_string_id: i32,
    cake_say_string_id: i32,
    /// The `(one_of group, branch)` that spawns `cake`; the other team's cake is in the
    /// sibling branch of the same group.
    branch: (usize, usize),
}

/// T538: one run's participation. Every map is keyed by character guid, never by session, so a
/// disconnect keeps the character's team, vote and used cake for the run.
#[derive(Clone, Debug, Default, PartialEq)]
struct LiveDanceOff {
    teams: BTreeMap<u64, DanceTeam>,
    votes: BTreeMap<u64, DanceTeam>,
    /// Characters with an open vote prompt and the run serial that owns it, so an answer counts
    /// in the run whose prompt the client is looking at.
    prompted: BTreeMap<u64, u32>,
    used_cake: BTreeSet<u64>,
    /// The side the ending went to; votes and joins stop once it is set.
    decided: Option<DanceTeam>,
    /// The per-side vote count at the ending, recorded when the draw is decided (the decision
    /// is where a miscount matters; tests pin it directly).
    tally: Option<[usize; 2]>,
    /// The next tick that scans for costume joins (the row's `join_scan_ms`): a spectator in
    /// the area is not rescanned at the full tick rate.
    next_scan_ms: u64,
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
    Move {
        to: [f32; 3],
        speed: f32,
    },
    Face([f32; 3]),
    Broadcast(usize),
    Area(i32, bool),
    Cycle(usize),
    /// The `one_of` decision point: runs at its own `at_ms`, before the branch steps it
    /// covers, and lets a dance-off replace the branch drawn at the run's start.
    OneOf(usize),
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

/// The file's rows, checked as a whole; a missing file is no events.
fn read(dir: &Path) -> io::Result<EventsFile> {
    let mut file = match std::fs::File::open(dir.join(FILE)) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(EventsFile { events: Vec::new() });
        }
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
    Ok(file)
}

/// Each row compiled alone: a row that fails is an `Err` in its place.
fn compile_rows(file: EventsFile) -> impl Iterator<Item = io::Result<WorldEvent>> {
    let mut keys = BTreeSet::new();
    file.events.into_iter().map(move |value| {
        let row: EventRow = serde_json::from_value(value).map_err(invalid)?;
        match keys.insert(row.key.clone()) {
            true => compile(row),
            false => Err(invalid(format!("world event {} is listed twice", row.key))),
        }
    })
}

/// Every event in `<dir>/world_events.json`, validated; a missing file is no events.
pub fn load(dir: &Path) -> io::Result<Vec<WorldEvent>> {
    compile_rows(read(dir)?).collect()
}

/// The events that carry a `schedule`. Unlike `load`, a row that is broken is logged and left
/// out, so one bad event never stops another's schedule; a file that cannot be read at all is
/// still an error.
pub fn load_scheduled(dir: &Path) -> io::Result<Vec<WorldEvent>> {
    let mut events = Vec::new();
    for row in compile_rows(read(dir)?) {
        match row {
            Ok(event) if event.schedule.is_some() => events.push(event),
            Ok(_) => {}
            Err(error) => println!(
                "{}",
                serde_json::json!({"event": "world_event_schedule_refused",
                    "reason": error.to_string()})
            ),
        }
    }
    Ok(events)
}

fn compile(row: EventRow) -> io::Result<WorldEvent> {
    let EventRow {
        key,
        actors: rows,
        timeline,
        schedule,
        dance_off,
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
    let schedule = schedule
        .map(|row| Schedule::compile(row, &key))
        .transpose()
        .map_err(&fail)?;
    for step in &steps {
        if let Op::Remove(effect) = step.op {
            actors[step.actor].remove_effect = effect;
        }
    }
    let dance_off = dance_off
        .map(|row| compile_dance_off(row, &index, &steps, &branches, &mut actors))
        .transpose()
        .map_err(&fail)?;
    Ok(WorldEvent {
        key,
        actors,
        steps,
        cycles,
        broadcasts,
        branches,
        schedule,
        dance_off,
    })
}

/// T538: one row's `dance_off` block. The two cakes must be the two branches of one `one_of`;
/// that pairing is the only mapping from a vote to the run's ending, so a content edit cannot
/// leave a second flag to keep in sync. Role actors become interactable; every other event actor
/// stays scenery.
fn compile_dance_off(
    row: DanceOffRow,
    index: &BTreeMap<String, usize>,
    steps: &[Step],
    branches: &[usize],
    actors: &mut [Actor],
) -> Result<DanceOff, String> {
    if !row
        .center
        .iter()
        .all(|value| value.is_finite() && value.abs() <= MAX_COORD)
        || !(row.radius_m > 0.0 && row.radius_m <= MAX_RADIUS_M)
        || !(MIN_JOIN_SCAN_MS..=MAX_JOIN_SCAN_MS).contains(&row.join_scan_ms)
        || row.source.is_empty()
        || row.source.len() > 4096
    {
        return Err(
            "dance_off needs a finite center, a radius of 1-10000 m, join_scan_ms, and a source"
                .into(),
        );
    }
    let positive_id = |id: i32| positive(id).map_err(|_| "dance_off ids must be > 0".to_string());
    let prompt_string_id = positive_id(row.prompt_string_id)?;
    let already_voted_string_id = positive_id(row.already_voted_string_id)?;
    let not_participant_string_id = positive_id(row.not_participant_string_id)?;
    let used_cake_string_id = positive_id(row.used_cake_string_id)?;
    let voting_box = *index
        .get(&row.voting_box)
        .ok_or_else(|| "dance_off names an unknown Voting Box actor".to_string())?;
    if !steps.iter().any(|step| {
        step.actor == voting_box && step.branch.is_none() && matches!(step.op, Op::Spawn(_))
    }) {
        return Err("the Voting Box must be spawned outside a one_of".into());
    }
    if row.teams.len() != 2 {
        return Err("dance_off needs exactly two teams".into());
    }
    let mut teams: [Option<TeamCostume>; 2] = [None, None];
    for team in row.teams {
        let slot = &mut teams[team.team.index()];
        if slot.is_some() {
            return Err("dance_off names a team twice".into());
        }
        let cake = *index
            .get(&team.cake_actor)
            .ok_or_else(|| format!("dance_off names an unknown cake actor {}", team.cake_actor))?;
        if cake == voting_box {
            return Err("the Voting Box cannot be a cake".into());
        }
        if team.costume_name_ids.is_empty()
            || team.costume_name_ids.len() > MAX_COSTUME_IDS
            || team.costume_name_ids.iter().any(|id| *id <= 0)
        {
            return Err(format!(
                "team {} needs 1-{MAX_COSTUME_IDS} costume name ids > 0",
                team.cake_actor
            ));
        }
        let vote_response_id = positive_id(team.vote_response_id)?;
        let vote_label_string_id = positive_id(team.vote_label_string_id)?;
        let cake_say_string_id = positive_id(team.cake_say_string_id)?;
        // The ending is the branch that spawns this team's cake; a cake spawned anywhere else
        // would make the vote-to-ending mapping ambiguous.
        if steps.iter().any(|step| {
            step.actor == cake && step.branch.is_none() && matches!(step.op, Op::Spawn(_))
        }) {
            return Err(format!(
                "cake {} must be spawned inside a one_of branch",
                team.cake_actor
            ));
        }
        let mut spawns = steps.iter().filter(|step| {
            step.actor == cake && step.branch.is_some() && matches!(step.op, Op::Spawn(_))
        });
        let Some(branch) = spawns.next().and_then(|step| step.branch) else {
            return Err(format!("cake {} has no one_of spawn", team.cake_actor));
        };
        if spawns.next().is_some() {
            return Err(format!(
                "cake {} is spawned in more than one branch",
                team.cake_actor
            ));
        }
        *slot = Some(TeamCostume {
            team: team.team,
            costume_name_ids: team.costume_name_ids,
            cake,
            vote_response_id,
            vote_label_string_id,
            cake_say_string_id,
            branch,
        });
    }
    let [Some(vampire), Some(werewolf)] = teams else {
        return Err("dance_off needs one vampire and one werewolf team".into());
    };
    let teams = [vampire, werewolf];
    if teams[0].vote_response_id == teams[1].vote_response_id {
        return Err("the two teams need distinct vote response ids".into());
    }
    if teams[0]
        .costume_name_ids
        .iter()
        .any(|id| teams[1].costume_name_ids.contains(id))
    {
        return Err("the two teams cannot share a costume name id".into());
    }
    if teams[0].branch.0 != teams[1].branch.0 || teams[0].branch.1 == teams[1].branch.1 {
        return Err("the two cakes must be the two branches of one one_of".into());
    }
    let group = teams[0].branch.0;
    if branches.get(group).copied() != Some(2) {
        return Err("the cakes' one_of needs exactly the two branches".into());
    }
    let template = crate::world::content_spawn_template();
    for index in [voting_box, teams[0].cake, teams[1].cake] {
        actors[index].spawn.is_interactable = 1;
        actors[index].spawn.interact_range = template.interact_range;
    }
    Ok(DanceOff {
        center: row.center,
        radius_m: row.radius_m,
        voting_box,
        join_scan_ms: row.join_scan_ms,
        prompt_string_id,
        already_voted_string_id,
        not_participant_string_id,
        used_cake_string_id,
        teams,
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
            OpRow::Face { actor, to } => {
                if !to.iter().all(|v| v.is_finite() && v.abs() <= MAX_COORD) {
                    return Err("face needs a finite target within 1e6 m".into());
                }
                (self.actor(&actor)?, Op::Face(to))
            }
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
                locale_id.map(positive).transpose()?;
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
                // The decision point runs before the branch rows it covers (stable sort keeps
                // this order among equal `at_ms`), so a dance-off's votes replace the draw
                // before any branch step is due.
                self.steps.push(Step {
                    at_ms,
                    actor: 0,
                    op: Op::OneOf(group),
                    branch: None,
                });
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
    /// The scheduled events (T462) and the earliest unix ms any of them needs the tick.
    schedule: Vec<Entry>,
    wake_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    event: Arc<WorldEvent>,
    instance: ZoneInstance,
    started_ms: u64,
    serial: u32,
    /// This run's timeline: every event step, the branch steps skipped when their branch is not
    /// `drawn`. A dance-off may replace one draw at its `one_of` step, so the branch steps stay.
    steps: Vec<Step>,
    next_step: usize,
    /// The branch each `one_of` group runs; drawn at the start and possibly re-decided.
    drawn: Vec<usize>,
    actors: Vec<Shown>,
    rng: Rng,
    cycles: Vec<Playing>,
    /// Areas this run switched on and not yet off; `stop` and the run's end switch them off.
    areas: BTreeSet<i32>,
    /// T538: `None` for an event with no `dance_off` row.
    dance_off: Option<LiveDanceOff>,
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

/// `rotation` turned toward the offset `delta` on X/Z; unchanged when there is no horizontal offset.
fn facing(rotation: [f32; 4], delta: [f32; 3]) -> [f32; 4] {
    let flat = delta[0].hypot(delta[2]);
    match flat > f32::EPSILON {
        true => [delta[0] / flat, 0.0, delta[2] / flat, 0.0],
        false => rotation,
    }
}

/// Whether `b` is within `radius` metres of `a`, on X/Z only when `flat`.
fn within(a: [f32; 3], b: [f32; 3], radius: f32, flat: bool) -> bool {
    let d = |axis: usize| f64::from(a[axis]) - f64::from(b[axis]);
    let height = if flat { 0.0 } else { d(1) };
    d(0) * d(0) + height * height + d(2) * d(2) <= f64::from(radius).powi(2)
}

/// T538: an event guid's run serial and actor index, from the low half `Run::guid` writes.
fn event_guid_parts(guid: u64) -> Option<(u32, usize)> {
    if !WorldEvents::is_event_guid(guid) {
        return None;
    }
    let serial = (guid as u32) >> 8;
    (serial != 0).then_some((serial, (guid & 0xff) as usize))
}

/// T538: a team's vote row is content (`DanceTeamRow::vote_response_id`); the 26/3 prompt row and
/// the 26/6 answer lookup read the same field, so they cannot drift. (R) The rows sit above the
/// NPC conversation's `1..=5` and the greeter/quest rows so a vote click cannot advance an NPC
/// dialogue the server still had open; the client echoes the row's own id (`009f6aa0`).
///
/// T538: the 26/3 vote prompt (reader `00a9ef10`). The body is the client's "Which side..?"
/// text id and the dialog camera targets the Voting Box; the rows carry the teams' labels.
fn vote_prompt_packet(actor: u64, dance: &DanceOff) -> Option<Vec<u8>> {
    let choices = dance
        .teams
        .iter()
        .map(|team| {
            crate::npc_interaction::dialogue_choice_row(
                team.vote_response_id,
                team.vote_label_string_id as u32,
            )
        })
        .collect();
    crate::npc_interaction::encode_dialogue_26_3(&crate::npc_interaction::DialoguePacket26_3 {
        initial_dwords: [
            dance.prompt_string_id as u32,
            0,
            actor as u32,
            (actor >> 32) as u32,
        ],
        initial_flag: false,
        initial_float_bits: 0,
        choices,
        choice_vectors: [[0; 4]; 2],
        vector_flag: false,
        final_vector: [0; 4],
        float_after_vector_bits: 0,
        final_flags: [false; 3],
        final_float_bits: 0,
        final_dword: 0,
    })
    .ok()
}

impl WorldEvents {
    /// Start `event` in `instance`; false, changing nothing, while a run of its key is live.
    pub fn start(&mut self, event: WorldEvent, instance: ZoneInstance, now_ms: u64) -> bool {
        self.begin(Arc::new(event), instance, now_ms, None)
    }

    /// Replace the schedule with the scheduled events of `events`, running in `instance`. Each
    /// event's next slot comes from `unix_ms` alone, so a restart never runs a slot late.
    pub fn schedule(
        &mut self,
        events: Vec<WorldEvent>,
        instance: ZoneInstance,
        unix_ms: u64,
        areas: &ClientAreas,
    ) {
        self.schedule = Schedule::entries(events, instance);
        for entry in &mut self.schedule {
            entry.begin(unix_ms, areas);
        }
        self.wake_ms = self
            .schedule
            .iter()
            .map(Entry::wake_ms)
            .min()
            .unwrap_or_default(); // nothing scheduled: `run_schedule` never reads it
    }

    /// Start each scheduled event that is due and send each kickoff line that is due. A start
    /// for an event that is already running does nothing. With nothing due, one comparison.
    fn run_schedule(
        &mut self,
        now_ms: u64,
        unix_ms: u64,
        players: &PlayerRegistry,
        areas: &ClientAreas,
        out: &mut Out,
    ) {
        if self.schedule.is_empty() || unix_ms < self.wake_ms {
            return;
        }
        let mut entries = std::mem::take(&mut self.schedule);
        for entry in &mut entries {
            match entry.step(unix_ms, areas) {
                Due::Idle => {}
                Due::Start => {
                    self.begin(Arc::clone(&entry.event), entry.instance, now_ms, None);
                }
                Due::Announce(line) => {
                    let heard = players.sessions_in(entry.instance).map(|p| p.session_id);
                    deliver(out, heard, || crate::chat::system(&line));
                }
            }
        }
        self.wake_ms = entries.iter().map(Entry::wake_ms).min().unwrap_or(u64::MAX);
        self.schedule = entries;
    }

    /// `start` with the run's random stream fixed (`seed`), so a test can draw each branch.
    fn begin(
        &mut self,
        event: Arc<WorldEvent>,
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
        let dance_off = event.dance_off.as_ref().map(|_| LiveDanceOff::default());
        // Every step stays: the `one_of` marker skips the branch that was not drawn, and a
        // dance-off may replace the draw at that marker.
        let steps = event.steps.clone();
        let run = Run {
            actors: vec![Shown::default(); event.actors.len()],
            event,
            instance,
            started_ms: now_ms,
            serial,
            steps,
            next_step: 0,
            drawn,
            rng,
            cycles: Vec::new(),
            areas: BTreeSet::new(),
            dance_off,
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
        self.run_schedule(now_ms, unix_ms, players, areas, &mut out);
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
                // A branch step waits for its group's draw; the marker before it decided.
                if step
                    .branch
                    .is_some_and(|(group, number)| run.drawn.get(group) != Some(&number))
                {
                    continue;
                }
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

    /// T538: whether `guid` is an event actor's guid (the `EVNT` tag). The bin routes these
    /// interactions here; no content NPC or player guid uses the range.
    pub fn is_event_guid(guid: u64) -> bool {
        guid >> 32 == GUID_TAG >> 32
    }

    /// T538: the world tick's per-run scan cadence: true at most every row's `join_scan_ms` for
    /// an open dance-off, so a spectator standing in the area is not rescanned at the full tick
    /// rate. False when no dance-off is live.
    pub fn dance_off_scan_due(&mut self, now_ms: u64) -> bool {
        let mut due = false;
        for run in self.runs.values_mut() {
            let Some(scan_ms) = run
                .event
                .dance_off
                .as_ref()
                .map(|dance| dance.join_scan_ms)
                .filter(|_| {
                    run.dance_off
                        .as_ref()
                        .is_some_and(|live| live.decided.is_none())
                })
            else {
                continue;
            };
            let Some(live) = run.dance_off.as_mut() else {
                continue;
            };
            if now_ms >= live.next_scan_ms {
                live.next_scan_ms = now_ms.saturating_add(scan_ms);
                due = true;
            }
        }
        due
    }

    /// T538: the `(instance, serial)` of every open dance-off; the world tick's join pass starts
    /// here. A file without one costs one run scan and no player scan.
    pub fn open_dance_offs(&self) -> Vec<(ZoneInstance, u32)> {
        let mut open: Vec<_> = self
            .runs
            .values()
            .filter(|run| {
                run.dance_off
                    .as_ref()
                    .is_some_and(|live| live.decided.is_none())
            })
            .map(|run| (run.instance, run.serial))
            .collect();
        open.sort_unstable();
        open
    }

    /// T538: whether run `serial` still accepts a join and `position` is inside its area: the
    /// area test the join pass needs before it reads any equipment. The position is the
    /// committed registry pose, never a raw client report.
    pub fn dance_off_area(&self, serial: u32, position: [f32; 3]) -> bool {
        self.runs.values().any(|run| {
            run.serial == serial
                && run
                    .dance_off
                    .as_ref()
                    .is_some_and(|live| live.decided.is_none())
                && run
                    .event
                    .dance_off
                    .as_ref()
                    .is_some_and(|dance| within(position, dance.center, dance.radius_m, false))
        })
    }

    /// T538: the team `character` would join in run `serial` by standing in the area in a team's
    /// costume. `name_ids` are the equipped whole-body costume `NameId`s. Read-only: the caller
    /// queues the quest hook and records the join with `join_dance_off`.
    pub fn dance_off_team(
        &self,
        serial: u32,
        character: u64,
        name_ids: &[i32],
    ) -> Option<DanceTeam> {
        let run = self.runs.values().find(|run| {
            run.serial == serial
                && run.dance_off.as_ref().is_some_and(|live| {
                    live.decided.is_none() && !live.teams.contains_key(&character)
                })
        })?;
        run.event
            .dance_off
            .as_ref()?
            .teams
            .iter()
            .find(|team| team.costume_name_ids.iter().any(|id| name_ids.contains(id)))
            .map(|team| team.team)
    }

    /// T538: record the join `dance_off_team` detected. False, changing nothing, when the run
    /// ended, was decided or already has the character. Keyed by character guid, so a
    /// disconnect keeps the team for the run.
    pub fn join_dance_off(&mut self, serial: u32, character: u64, team: DanceTeam) -> bool {
        let Some(run) = self.runs.values_mut().find(|run| run.serial == serial) else {
            return false;
        };
        let Some(live) = run.dance_off.as_mut().filter(|live| live.decided.is_none()) else {
            return false;
        };
        if live.teams.contains_key(&character) {
            return false;
        }
        live.teams.insert(character, team);
        true
    }

    /// T538: the team run `serial` recorded for `character`, which is what the cake serves and
    /// what the join pass re-checks after a reconnect.
    pub fn dance_off_participant(&self, serial: u32, character: u64) -> Option<DanceTeam> {
        self.runs
            .values()
            .find(|run| run.serial == serial)?
            .dance_off
            .as_ref()?
            .teams
            .get(&character)
            .copied()
    }

    /// T538: one client 26/8 click on an event actor, answered in the bin's event-guid route.
    /// The Voting Box opens the 26/3 vote prompt; a live cake answers its 15/4 line. Every
    /// other event actor, an out-of-reach click, a stale guid and a decided run answer nothing.
    pub fn interact(
        &mut self,
        target: u64,
        character: u64,
        instance: ZoneInstance,
        position: [f32; 3],
    ) -> Vec<Vec<u8>> {
        let Some((serial, index)) = event_guid_parts(target) else {
            return Vec::new();
        };
        let Some(run) = self.runs.values_mut().find(|run| run.serial == serial) else {
            return Vec::new();
        };
        let Some(live) = run.dance_off.as_mut() else {
            return Vec::new();
        };
        if run.instance != instance || !run.actors.get(index).is_some_and(|actor| actor.live) {
            return Vec::new();
        }
        let event = Arc::clone(&run.event);
        let Some(dance) = event.dance_off.as_ref() else {
            return Vec::new();
        };
        let reach = crate::npc_interaction::INTERACTION_RADIUS as f32;
        if !within(position, run.actors[index].position, reach, false) {
            return Vec::new();
        }
        if index == dance.voting_box {
            if live.decided.is_some() {
                return Vec::new();
            }
            let Some(packet) = vote_prompt_packet(target, dance) else {
                return Vec::new();
            };
            // Re-opening replaces any prompt already open; a refused click above left it alone.
            live.prompted.insert(character, run.serial);
            return vec![packet];
        }
        let Some(team) = dance.teams.iter().find(|team| team.cake == index) else {
            return Vec::new();
        };
        if !live.teams.contains_key(&character) {
            return vec![crate::chat::string_say(
                target,
                dance.not_participant_string_id,
            )];
        }
        if !live.used_cake.insert(character) {
            return vec![crate::chat::string_say(target, dance.used_cake_string_id)];
        }
        vec![crate::chat::string_say(target, team.cake_say_string_id)]
    }

    /// T538: the client's 26/6 answer to an open vote prompt, owned by the run whose prompt the
    /// client is looking at. `None` when no prompt is open or the row is not one of that run's
    /// vote rows, so another dialogue's answer stays with the NPC dialogue path. Only a counted
    /// vote changes the run and closes the prompt; a refused answer sends nothing, so the client
    /// keeps the prompt and the player can step back into reach and answer.
    pub fn vote(
        &mut self,
        character: u64,
        response_id: i32,
        instance: ZoneInstance,
        position: [f32; 3],
    ) -> Option<Vec<Vec<u8>>> {
        let serial = self
            .runs
            .values()
            .find_map(|run| run.dance_off.as_ref()?.prompted.get(&character).copied())?;
        let run = self.runs.values_mut().find(|run| run.serial == serial)?;
        let event = Arc::clone(&run.event);
        let dance = event.dance_off.as_ref()?;
        // Only the row a team's vote offers counts; any other 26/6 belongs to another dialogue.
        let team = dance
            .teams
            .iter()
            .find(|team| team.vote_response_id == response_id)
            .map(|team| team.team)?;
        let instance_ok = run.instance == instance;
        let box_guid = run.guid(dance.voting_box);
        let box_live = run
            .actors
            .get(dance.voting_box)
            .is_some_and(|actor| actor.live);
        let box_at = run.actors[dance.voting_box].position;
        let live = run.dance_off.as_mut()?;
        let reach = crate::npc_interaction::INTERACTION_RADIUS as f32;
        if !instance_ok || !box_live || !within(position, box_at, reach, false) {
            // A refusal changes nothing: no close, so the client keeps the prompt.
            return Some(Vec::new());
        }
        if live.decided.is_some() {
            return Some(Vec::new());
        }
        if live.votes.contains_key(&character) {
            return Some(vec![
                crate::chat::string_say(box_guid, dance.already_voted_string_id),
                crate::npc_interaction::encode_dialogue_close_26_4(),
            ]);
        }
        live.prompted.remove(&character);
        live.votes.insert(character, team);
        Some(vec![crate::npc_interaction::encode_dialogue_close_26_4()])
    }

    /// T538: a click on another actor ends any open vote prompt, so a later 26/6 cannot be
    /// answered by a prompt the player abandoned. A run's end clears its own prompts.
    pub fn clear_vote_prompt(&mut self, character: u64) {
        for run in self.runs.values_mut() {
            if let Some(live) = &mut run.dance_off {
                live.prompted.remove(&character);
            }
        }
    }
}

impl Run {
    fn guid(&self, index: usize) -> u64 {
        GUID_TAG | u64::from(self.serial) << 8 | index as u64
    }

    /// T538: the `one_of` at `group` is due, so its draw is final. A dance-off whose cakes are
    /// that group's branches gives the ending to the side with more votes; a tie or no votes
    /// keeps the draw the run made at its start. Every group then drops the steps of the branch
    /// it will not take, so the run ends with its drawn branch instead of waiting for a skipped
    /// step far later (the marker sorts before the branch rows it covers).
    fn decide(&mut self, group: usize) {
        let Some(&drawn) = self.drawn.get(group) else {
            return;
        };
        let mut chosen = drawn;
        let mut team = None;
        let mut tally = None;
        if let (Some(live), Some(data)) = (self.dance_off.as_ref(), self.event.dance_off.as_ref())
            && group == data.teams[0].branch.0
            && live.decided.is_none()
        {
            let mut counted = [0usize; 2];
            for voted in live.votes.values() {
                counted[voted.index()] += 1;
            }
            team = match counted[0].cmp(&counted[1]) {
                std::cmp::Ordering::Greater => Some(DanceTeam::Vampire),
                std::cmp::Ordering::Less => Some(DanceTeam::Werewolf),
                std::cmp::Ordering::Equal => data
                    .teams
                    .iter()
                    .find(|team| team.branch.1 == drawn)
                    .map(|team| team.team),
            };
            if let Some(side) = team {
                chosen = data.teams[side.index()].branch.1;
            }
            tally = Some(counted);
        }
        self.drawn[group] = chosen;
        if let Some(live) = self.dance_off.as_mut()
            && let Some(counted) = tally
        {
            if let Some(side) = team {
                live.decided = Some(side);
                // The box closes with the ending: a prompt still open answers nothing after it.
                live.prompted.clear();
            }
            live.tally = Some(counted);
        }
        let mut next_step = 0;
        let mut kept = Vec::with_capacity(self.steps.len());
        for (index, step) in self.steps.iter().copied().enumerate() {
            if step
                .branch
                .is_some_and(|(branch_group, branch)| branch_group == group && branch != chosen)
            {
                continue;
            }
            if index < self.next_step {
                next_step += 1;
            }
            kept.push(step);
        }
        self.steps = kept;
        self.next_step = next_step;
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
            Op::OneOf(group) => {
                self.decide(group);
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
            Op::Face(to) => {
                // A walker turns toward its own target again on its next step.
                shown.rotation =
                    facing(shown.rotation, [0, 1, 2].map(|a| to[a] - shown.position[a]));
                movement_packet(
                    guid,
                    shown.position,
                    [shown.rotation[0], 0.0, shown.rotation[2]],
                )
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
            Op::Broadcast(_) | Op::Area(..) | Op::Cycle(_) | Op::OneOf(_) => {
                unreachable!("handled above")
            }
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
        shown.rotation = facing(shown.rotation, delta);
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
        assert!(load(&shipped).is_ok(), "the shipped file loads");
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

    /// `face` turns a standing actor toward its target on X/Z with one opcode 125 packet at the
    /// actor's own position; a viewer who arrives later gets the AddNpc already turned, a target
    /// straight above keeps the heading, and a target that is not finite is refused at load.
    #[test]
    fn face_turns_the_actor_in_place_and_late_viewers_see_it_turned() {
        let face =
            |at_ms, to: [f32; 3]| json!({"at_ms": at_ms, "op": {"face": {"actor": "a", "to": to}}});
        let turner = event_at(
            [10.0, 5.0, 10.0],
            json!([
                spawn_a(),
                face(100, [30.0, 9.0, 10.0]),
                face(200, [10.0, 0.0, 10.0])
            ]),
        );
        let mut players = registry(&[(1, WORLD, [11.0, 0.0, 10.0])]);
        let mut events = WorldEvents::default();
        assert!(events.start(turner, WORLD, 0));
        tick(&mut events, 0, &players);
        let heading = |packet: &[u8]| -> Vec<u8> { packet[22..34].to_vec() };
        let toward = |x: f32, z: f32| {
            [x, 0.0, z]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<u8>>()
        };
        let turned = to(&tick(&mut events, 100, &players), 1);
        assert_eq!(turned.len(), 1);
        assert_eq!(
            (turned[0][..2].to_vec(), xyz(&turned[0])),
            (vec![125, 0], [10.0, 5.0, 10.0])
        );
        assert_eq!(heading(&turned[0]), toward(1.0, 0.0), "faces +x, not up");
        let same = to(&tick(&mut events, 200, &players), 1);
        assert_eq!(
            heading(&same[0]),
            toward(1.0, 0.0),
            "no horizontal offset: unchanged"
        );
        players
            .join(SessionId(2), 2, 2, WORLD, [11.0, 0.0, 10.0], 0.0)
            .unwrap();
        let late = to(&tick(&mut events, 300, &players), 2);
        assert!(
            late[0]
                .windows(12)
                .any(|w| w == toward(1.0, 0.0).as_slice()),
            "the late AddNpc carries the turned rotation"
        );
        let bad = json!({"events": [event("show", json!([{"at_ms": 0, "op": {"face": {"actor": "a", "to": [1e39, 0.0, 0.0]}}}]))]});
        assert!(load_json("face", &bad).is_err());
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
        assert!(events.begin(Arc::new(event_at([0.0; 3], timeline)), WORLD, 0, Some(seed)));
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
            // T493: a reconstructed line has no client locale entry, so the field may be left out.
            step(json!({"broadcast": {"text": "hi"}})),
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

    /// T538: three actors and one ending `one_of` whose branches spawn the two teams' cakes.
    fn dance() -> serde_json::Value {
        json!({
            "key": "dance",
            "actors": [actor("voting_box"), actor("cake_vampire"), actor("cake_werewolf")],
            "timeline": [
                {"at_ms": 0, "op": {"spawn": {"actor": "voting_box"}}},
                {"at_ms": 1000, "op": {"one_of": {"branches": [
                    [{"at_ms": 100, "op": {"spawn": {"actor": "cake_vampire"}}}],
                    [{"at_ms": 100, "op": {"spawn": {"actor": "cake_werewolf"}}}]]}}},
                {"at_ms": 5000, "op": {"remove": {"actor": "voting_box", "effect_id": 21}}},
                {"at_ms": 5000, "op": {"remove": {"actor": "cake_vampire", "effect_id": 21}}},
                {"at_ms": 5000, "op": {"remove": {"actor": "cake_werewolf", "effect_id": 21}}}
            ],
            "dance_off": dance_off()
        })
    }

    fn dance_off() -> serde_json::Value {
        json!({
            "center": [0.0, 0.0, 0.0],
            "radius_m": 30.0,
            "voting_box": "voting_box",
            "join_scan_ms": 500,
            "prompt_string_id": 432429,
            "already_voted_string_id": 5252,
            "not_participant_string_id": 432505,
            "used_cake_string_id": 432430,
            "teams": [
                {"team": "vampire", "costume_name_ids": [14688], "cake_actor": "cake_vampire",
                 "vote_response_id": 401, "vote_label_string_id": 432207,
                 "cake_say_string_id": 432434},
                {"team": "werewolf", "costume_name_ids": [14690], "cake_actor": "cake_werewolf",
                 "vote_response_id": 402, "vote_label_string_id": 432208,
                 "cake_say_string_id": 432435}
            ],
            "source": "T538 unit fixture"
        })
    }

    /// One run started from a fixed seed, so a test can draw the branch itself.
    fn dance_run(seed: u64) -> WorldEvents {
        let event = load_json("dance", &json!({"events": [dance()]}))
            .unwrap()
            .pop()
            .unwrap();
        let mut events = WorldEvents::default();
        assert!(events.begin(Arc::new(event), WORLD, 0, Some(seed)));
        events
    }

    /// The prompt's response row a team answers with.
    fn response_for(team: DanceTeam) -> i32 {
        match team {
            DanceTeam::Vampire => 401,
            DanceTeam::Werewolf => 402,
        }
    }

    /// Opens the vote prompt and answers it for `character`.
    fn cast(events: &mut WorldEvents, character: u64, team: DanceTeam) -> Vec<Vec<u8>> {
        let box_guid = events.runs["dance"].guid(0);
        assert_eq!(
            events.interact(box_guid, character, WORLD, [0.0; 3]).len(),
            1,
            "the prompt"
        );
        events
            .vote(character, response_for(team), WORLD, [0.0; 3])
            .unwrap_or_else(|| panic!("vote {team:?} had no open prompt"))
    }

    /// The world tick's two steps for one character: the area gate, then detect the team from
    /// the outfit and record the join. `None` when the area, costume or run refuses the join.
    fn join(
        events: &mut WorldEvents,
        position: [f32; 3],
        character: u64,
        names: &[i32],
    ) -> Option<DanceTeam> {
        let serial = events.runs["dance"].serial;
        if !events.dance_off_area(serial, position) {
            return None;
        }
        let team = events.dance_off_team(serial, character, names)?;
        assert!(
            events.join_dance_off(serial, character, team),
            "the join was refused right after detection"
        );
        Some(team)
    }

    #[test]
    fn the_dance_off_loader_refuses_bad_rows_and_makes_the_roles_interactable() {
        let ok = json!({"events": [dance()]});
        let compiled = load_json("dance-ok", &ok).unwrap().pop().unwrap();
        assert!(compiled.dance_off.is_some());
        assert_eq!(compiled.actors[0].spawn.is_interactable, 1);
        assert_eq!(
            compiled.actors[0].spawn.interact_range,
            crate::world::content_spawn_template().interact_range
        );
        assert_eq!(compiled.actors[1].spawn.is_interactable, 1);
        assert_eq!(compiled.actors[2].spawn.is_interactable, 1);
        let mut bad = Vec::new();
        let mut one_team = dance();
        one_team["dance_off"]["teams"].as_array_mut().unwrap().pop();
        bad.push(("one team", one_team));
        let mut twice = dance();
        twice["dance_off"]["teams"][1]["team"] = json!("vampire");
        bad.push(("team twice", twice));
        let mut shared = dance();
        shared["dance_off"]["teams"][1]["costume_name_ids"] = json!([14688]);
        bad.push(("shared costume", shared));
        let mut zero_name = dance();
        zero_name["dance_off"]["teams"][0]["costume_name_ids"] = json!([0]);
        bad.push(("costume 0", zero_name));
        let mut no_names = dance();
        no_names["dance_off"]["teams"][0]["costume_name_ids"] = json!([]);
        bad.push(("no costume", no_names));
        let mut unknown_cake = dance();
        unknown_cake["dance_off"]["teams"][0]["cake_actor"] = json!("nope");
        bad.push(("unknown cake", unknown_cake));
        let mut unknown_box = dance();
        unknown_box["dance_off"]["voting_box"] = json!("nope");
        bad.push(("unknown box", unknown_box));
        let mut box_is_cake = dance();
        box_is_cake["dance_off"]["voting_box"] = json!("cake_vampire");
        bad.push(("box is a cake", box_is_cake));
        let mut outside = dance();
        outside["timeline"]
            .as_array_mut()
            .unwrap()
            .push(json!({"at_ms": 0, "op": {"spawn": {"actor": "cake_vampire"}}}));
        bad.push(("cake outside a branch", outside));
        let mut same_branch = dance();
        same_branch["timeline"][1]["op"]["one_of"]["branches"][1][0]["op"]["spawn"]["actor"] =
            json!("cake_vampire");
        bad.push(("one cake branch", same_branch));
        let mut third = dance();
        third["timeline"][1]["op"]["one_of"]["branches"]
            .as_array_mut()
            .unwrap()
            .push(json!([]));
        bad.push(("three branches", third));
        let mut prompt = dance();
        prompt["dance_off"]["prompt_string_id"] = json!(0);
        bad.push(("prompt 0", prompt));
        let mut label = dance();
        label["dance_off"]["teams"][0]["vote_label_string_id"] = json!(0);
        bad.push(("label 0", label));
        let mut radius = dance();
        radius["dance_off"]["radius_m"] = json!(0.0);
        bad.push(("radius 0", radius));
        let mut cadence = dance();
        cadence["dance_off"]["join_scan_ms"] = json!(0);
        bad.push(("cadence 0", cadence));
        let mut slow = dance();
        slow["dance_off"]["join_scan_ms"] = json!(MAX_JOIN_SCAN_MS + 1);
        bad.push(("cadence over the bound", slow));
        let mut row = dance();
        row["dance_off"]["teams"][0]["vote_response_id"] = json!(0);
        bad.push(("vote row 0", row));
        let mut shared_row = dance();
        shared_row["dance_off"]["teams"][1]["vote_response_id"] = json!(401);
        bad.push(("shared vote row", shared_row));
        let mut far = dance();
        far["dance_off"]["center"] = json!([1e39, 0.0, 0.0]);
        bad.push(("infinite center", far));
        let mut source = dance();
        source["dance_off"]["source"] = json!("");
        bad.push(("no source", source));
        let mut extra = dance();
        extra["dance_off"]["winner"] = json!("vampire");
        bad.push(("unknown field", extra));
        for (label, row) in bad {
            let file = json!({"events": [row]});
            assert!(load_json(label, &file).is_err(), "{label} accepted");
        }
        // The shipped dance-off compiles with its roles interactable.
        let shipped = load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap();
        let event = shipped
            .iter()
            .find(|event| event.key == "halloween_dance_off")
            .expect("the shipped dance-off");
        let dance_off = event.dance_off.as_ref().expect("its dance_off block");
        let voting = &event.actors[dance_off.voting_box];
        assert_eq!(
            (voting.spawn.is_interactable, voting.spawn.interact_range),
            (1, crate::world::content_spawn_template().interact_range)
        );
        assert_eq!(voting.spawn.npc_name, "Voting Box");
        assert_eq!(dance_off.teams[0].team, DanceTeam::Vampire);
        assert_eq!(dance_off.teams[0].costume_name_ids, [14688, 119187, 119191]);
        assert_eq!(dance_off.teams[1].costume_name_ids, [14690, 119208]);
        assert_eq!(
            dance_off
                .teams
                .iter()
                .map(|team| (team.team, team.vote_response_id))
                .collect::<Vec<_>>(),
            [(DanceTeam::Vampire, 401), (DanceTeam::Werewolf, 402)]
        );
        assert_eq!(dance_off.join_scan_ms, 500);
    }

    /// T538 (review finding 5): the join pass scans an open dance-off at most every row's
    /// `join_scan_ms`, so a spectator in the area is not rescanned at the full 20 Hz; the open
    /// list is what the bin's pass starts from and is empty after the ending.
    #[test]
    fn the_costume_scan_is_cadenced_and_lists_the_open_dance_offs() {
        let players = registry_with(1, [0.0; 3]);
        let mut events = dance_run(2);
        let serial = events.runs["dance"].serial;
        let scan = events.runs["dance"]
            .event
            .dance_off
            .as_ref()
            .unwrap()
            .join_scan_ms;
        assert_eq!(scan, 500);
        assert_eq!(events.open_dance_offs(), [(WORLD, serial)]);
        assert!(events.dance_off_scan_due(0));
        assert!(!events.dance_off_scan_due(scan - 1));
        assert!(events.dance_off_scan_due(scan));
        tick(&mut events, 1_000, &players);
        assert!(events.open_dance_offs().is_empty());
        assert!(!events.dance_off_scan_due(scan * 2));
    }

    #[test]
    fn a_costume_in_the_area_joins_one_team_for_the_whole_run() {
        let mut events = dance_run(1);
        let serial = events.runs["dance"].serial;
        assert!(events.dance_off_area(serial, [0.0; 3]));
        assert_eq!(
            join(&mut events, [1.0, 0.0, 1.0], 11, &[14690]),
            Some(DanceTeam::Werewolf)
        );
        assert_eq!(
            events.dance_off_participant(serial, 11),
            Some(DanceTeam::Werewolf)
        );
        // A joined character reads no equipment again: the team lookup refuses and the join is
        // not offered a second time.
        assert_eq!(
            events.dance_off_team(serial, 11, &[14690]),
            None,
            "a joined character is not scanned again"
        );
        assert_eq!(
            events.open_dance_offs(),
            [(WORLD, serial)],
            "only the run's own instance is scanned"
        );
        assert_eq!(
            join(&mut events, [0.0; 3], 12, &[14688]),
            Some(DanceTeam::Vampire)
        );
        // Two costumes at once: the first team in content order wins (R).
        assert_eq!(
            join(&mut events, [0.0; 3], 13, &[14690, 14688]),
            Some(DanceTeam::Vampire)
        );
        // No costume, outside the radius, another serial and an existing member do nothing.
        assert_eq!(join(&mut events, [0.0; 3], 14, &[]), None);
        assert_eq!(join(&mut events, [100.0, 0.0, 0.0], 15, &[14690]), None);
        assert_eq!(events.dance_off_team(serial + 1, 16, &[14690]), None);
        assert_eq!(join(&mut events, [0.0; 3], 11, &[14690]), None);
        // The area gate applies before any equipment is read.
        assert!(!events.dance_off_area(serial, [100.0, 0.0, 0.0]));
        // The ending closes joining, even for a character never seen before.
        let players = registry_with(1, [0.0; 3]);
        tick(&mut events, 1_000, &players);
        assert_eq!(join(&mut events, [0.0; 3], 17, &[14690]), None);
        assert!(
            !events.join_dance_off(serial, 17, DanceTeam::Werewolf),
            "the record refuses after the ending"
        );
        assert!(
            !events.dance_off_area(serial, [0.0; 3]),
            "the ending closes joins"
        );
        let live = events.runs["dance"].dance_off.as_ref().unwrap();
        assert_eq!(live.teams.len(), 3);
        assert!(live.decided.is_some(), "the ending closed the run");
    }

    #[test]
    fn votes_decide_the_ending_and_a_tie_or_no_votes_keeps_the_draw() {
        let players = registry_with(1, [0.0; 3]);
        // More votes for one team: that team's cake is the ending, whatever the draw was. The
        // recorded tally is asserted too, so a vote counted twice cannot pass unnoticed (the
        // brief's canary doubles every count, which preserves a majority).
        let mut events = dance_run(7);
        tick(&mut events, 0, &players);
        cast(&mut events, 41, DanceTeam::Vampire);
        cast(&mut events, 42, DanceTeam::Vampire);
        cast(&mut events, 43, DanceTeam::Werewolf);
        tick(&mut events, 1_000, &players);
        let live = events.runs["dance"].dance_off.as_ref().unwrap();
        assert_eq!(live.decided, Some(DanceTeam::Vampire));
        assert_eq!(live.tally, Some([2, 1]), "the counted votes, per side");
        let out = tick(&mut events, 1_100, &players);
        let cake = events.runs["dance"].guid(1);
        assert!(
            out.iter()
                .any(|(_, packet)| packet.starts_with(&[35, 0, 2, 0])
                    && packet[4..12] == cake.to_le_bytes()),
            "the vampire cake spawned"
        );
        // A tie keeps the branch drawn at the start.
        let mut events = dance_run(7);
        tick(&mut events, 0, &players);
        cast(&mut events, 51, DanceTeam::Vampire);
        cast(&mut events, 52, DanceTeam::Werewolf);
        let drawn = events.runs["dance"].drawn[0];
        tick(&mut events, 1_000, &players);
        let live = events.runs["dance"].dance_off.as_ref().unwrap();
        assert_eq!(live.tally, Some([1, 1]));
        assert_eq!(
            live.decided,
            Some(if drawn == 0 {
                DanceTeam::Vampire
            } else {
                DanceTeam::Werewolf
            })
        );
        // No votes keeps it too.
        let mut events = dance_run(7);
        let drawn = events.runs["dance"].drawn[0];
        tick(&mut events, 1_100, &players);
        let live = events.runs["dance"].dance_off.as_ref().unwrap();
        assert_eq!(live.tally, Some([0, 0]));
        assert_eq!(
            live.decided,
            Some(if drawn == 0 {
                DanceTeam::Vampire
            } else {
                DanceTeam::Werewolf
            })
        );
    }

    /// T538 (review finding 7): two dance-offs in one instance are keyed by their run serial, so
    /// a join, a prompt and a vote always land in the run they were made for.
    #[test]
    fn two_dance_offs_in_one_instance_keep_their_own_joins_and_votes() {
        let mut first = dance();
        first["key"] = json!("dance_a");
        let mut second = dance();
        second["key"] = json!("dance_b");
        // The second run's box and area are 50 m away, so a click reaches one run's box only.
        second["actors"][0]["position"] = json!([50.0, 0.0, 0.0]);
        second["dance_off"]["center"] = json!([50.0, 0.0, 0.0]);
        let mut events = WorldEvents::default();
        for (seed, row) in [first, second].into_iter().enumerate() {
            let event = load_json("two runs", &json!({"events": [row]}))
                .unwrap()
                .pop()
                .unwrap();
            assert!(events.begin(Arc::new(event), WORLD, 0, Some(seed as u64 + 1)));
        }
        let serial_a = events.runs["dance_a"].serial;
        let serial_b = events.runs["dance_b"].serial;
        assert_ne!(serial_a, serial_b);
        assert_eq!(
            events.open_dance_offs(),
            [(WORLD, serial_a), (WORLD, serial_b)]
        );
        // A costume join names its run: no lookup picks "the first dance-off".
        assert!(events.dance_off_area(serial_a, [0.0; 3]));
        assert!(!events.dance_off_area(serial_a, [50.0, 0.0, 0.0]));
        assert_eq!(
            events.dance_off_team(serial_b, 1, &[14688]),
            Some(DanceTeam::Vampire)
        );
        assert!(events.join_dance_off(serial_b, 1, DanceTeam::Vampire));
        assert_eq!(
            events.dance_off_participant(serial_b, 1),
            Some(DanceTeam::Vampire)
        );
        assert_eq!(
            events.dance_off_participant(serial_a, 1),
            None,
            "the join did not leak into the other run"
        );
        // A prompt answers in the run whose box the player clicked: the other run's box is out
        // of reach, so that answer is refused and leaves its prompt open.
        let players = registry_with(1, [0.0; 3]);
        tick(&mut events, 0, &players);
        let box_a = events.runs["dance_a"].guid(0);
        let box_b = events.runs["dance_b"].guid(0);
        let box_a = events.runs["dance_a"].guid(0);
        let box_b = events.runs["dance_b"].guid(0);
        let at_b = [50.0, 0.0, 0.0];
        assert_eq!(events.interact(box_a, 2, WORLD, [0.0; 3]).len(), 1);
        assert_eq!(events.interact(box_b, 3, WORLD, at_b).len(), 1);
        // Character 2's prompt is run A's, so an answer at run B's box is out of reach and is
        // refused without consuming it; the answer at A counts.
        assert_eq!(events.vote(2, 401, WORLD, at_b), Some(Vec::new()));
        assert!(events.vote(2, 401, WORLD, [0.0; 3]).is_some());
        assert!(events.vote(3, 401, WORLD, at_b).is_some());
        assert_eq!(
            events.runs["dance_a"]
                .dance_off
                .as_ref()
                .unwrap()
                .votes
                .get(&2),
            Some(&DanceTeam::Vampire)
        );
        assert_eq!(
            events.runs["dance_b"]
                .dance_off
                .as_ref()
                .unwrap()
                .votes
                .get(&3),
            Some(&DanceTeam::Vampire)
        );
        assert!(
            !events.runs["dance_a"]
                .dance_off
                .as_ref()
                .unwrap()
                .votes
                .contains_key(&3),
            "the B prompt's answer did not count in A"
        );
    }

    #[test]
    fn one_vote_per_character_and_only_an_open_prompt_counts() {
        let players = registry_with(1, [0.0; 3]);
        let mut events = dance_run(3);
        tick(&mut events, 0, &players);
        let box_guid = events.runs["dance"].guid(0);
        // A 26/6 with no prompt belongs to the NPC dialogue path.
        assert!(events.vote(61, 1, WORLD, [0.0; 3]).is_none());
        assert!(
            events
                .interact(box_guid, 61, WORLD, [100.0, 0.0, 0.0])
                .is_empty()
        );
        cast(&mut events, 61, DanceTeam::Vampire);
        let again = {
            assert_eq!(events.interact(box_guid, 61, WORLD, [0.0; 3]).len(), 1);
            events.vote(61, 402, WORLD, [0.0; 3]).unwrap()
        };
        assert_eq!(
            again,
            [
                crate::chat::string_say(box_guid, 5252),
                crate::npc_interaction::encode_dialogue_close_26_4()
            ],
            "a second vote answers the already-voted line and changes no tally"
        );
        let live = events.runs["dance"].dance_off.as_ref().unwrap();
        assert_eq!(live.votes.get(&61), Some(&DanceTeam::Vampire));
        assert_eq!(live.votes.len(), 1);
        // A prompt opened before the ending is closed by the ending: its answer is refused and
        // counts nothing.
        assert_eq!(events.interact(box_guid, 62, WORLD, [0.0; 3]).len(), 1);
        assert_eq!(events.interact(box_guid, 63, WORLD, [0.0; 3]).len(), 1);
        tick(&mut events, 1_000, &players);
        assert_eq!(events.vote(62, 401, WORLD, [0.0; 3]), None);
        assert!(
            events.runs["dance"]
                .dance_off
                .as_ref()
                .unwrap()
                .votes
                .get(&62)
                .is_none()
        );
        // The box opens nothing after the ending, and a close drops a prompt left open.
        assert!(events.interact(box_guid, 64, WORLD, [0.0; 3]).is_empty());
        events.clear_vote_prompt(63);
        assert_eq!(events.vote(63, 1, WORLD, [0.0; 3]), None);
    }

    /// T538 (review finding 4 and the attempt-2 blocker): a refused click or vote leaves the
    /// prompt open and answers nothing, so the client keeps its dialog and the player can step
    /// back and answer. Only a counted vote consumes the prompt.
    #[test]
    fn a_refused_click_or_vote_keeps_the_prompt_open() {
        let players = registry_with(1, [0.0; 3]);
        let mut events = dance_run(9);
        tick(&mut events, 0, &players);
        let box_guid = events.runs["dance"].guid(0);
        let close = crate::npc_interaction::encode_dialogue_close_26_4();
        assert_eq!(events.interact(box_guid, 61, WORLD, [0.0; 3]).len(), 1);
        // Out of reach: the click and the answer are refused without a reply or a close, and
        // the prompt stays.
        assert!(
            events
                .interact(box_guid, 61, WORLD, [100.0, 0.0, 0.0])
                .is_empty()
        );
        assert_eq!(
            events.vote(61, 401, WORLD, [100.0, 0.0, 0.0]),
            Some(Vec::new()),
            "an out-of-reach answer sends nothing, so the client keeps the prompt"
        );
        assert!(
            events.runs["dance"]
                .dance_off
                .as_ref()
                .unwrap()
                .prompted
                .contains_key(&61),
            "the out-of-reach refusal kept the prompt"
        );
        // A row that is not a vote belongs to another dialogue: the event route leaves it
        // alone (and the prompt open).
        assert_eq!(events.vote(61, 1, WORLD, [0.0; 3]), None);
        assert!(
            events.runs["dance"]
                .dance_off
                .as_ref()
                .unwrap()
                .prompted
                .contains_key(&61)
        );
        // Back in reach the same prompt counts the vote once and consumes it.
        assert_eq!(events.vote(61, 401, WORLD, [0.0; 3]), Some(vec![close]));
        let live = events.runs["dance"].dance_off.as_ref().unwrap();
        assert_eq!(live.votes.get(&61), Some(&DanceTeam::Vampire));
        assert!(
            !live.prompted.contains_key(&61),
            "the counted vote consumed it"
        );
        // One content field drives the prompt rows and the answer rows, so they cannot drift.
        let dance = events.runs["dance"].event.dance_off.as_ref().unwrap();
        assert_eq!(
            dance
                .teams
                .iter()
                .map(|team| (team.team, team.vote_response_id))
                .collect::<Vec<_>>(),
            [(DanceTeam::Vampire, 401), (DanceTeam::Werewolf, 402)]
        );
    }

    /// T538 (review finding 7): the `one_of` marker drops the branch the run did not draw, so
    /// the run ends with the drawn branch's last step instead of a skipped step far later.
    #[test]
    fn a_one_of_ends_with_its_drawn_branch_not_a_skipped_late_step() {
        let branch = |id: i32, anim_at: u64| {
            json!([
                {"at_ms": 0, "op": {"spawn": {"actor": "a"}}},
                {"at_ms": 1000, "op": {"remove": {"actor": "a"}}},
                {"at_ms": anim_at, "op": {"play_anim": {"actor": "a", "id": id}}}
            ])
        };
        // Branch 0 ends at 2000; branch 1 has a step at 7000.
        let event = one(json!([
            {"at_ms": 1000, "op": {"one_of": {"branches": [branch(21, 1000), branch(22, 6000)]}}}
        ]));
        let mut seed = 0;
        loop {
            let mut rng = Rng::seeded(seed);
            if rng.next_u64() % 2 == 0 {
                break;
            }
            seed += 1;
            assert!(seed < 64, "a seed draws the short branch");
        }
        let mut events = WorldEvents::default();
        assert!(events.begin(Arc::new(event), WORLD, 0, Some(seed)));
        assert_eq!(events.runs["show"].drawn[0], 0);
        let players = registry_with(1, [1.0, 0.0, 0.0]);
        tick(&mut events, 1_000, &players);
        tick(&mut events, 2_100, &players);
        assert!(
            events.runs.is_empty(),
            "the skipped branch's 7000 ms step must not keep the run alive: {:?}",
            events.runs.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn the_winning_cake_serves_each_participant_once_and_refuses_the_rest() {
        let players = registry_with(1, [0.0; 3]);
        let mut events = dance_run(11);
        tick(&mut events, 0, &players);
        assert_eq!(
            join(&mut events, [0.0; 3], 71, &[14688]),
            Some(DanceTeam::Vampire)
        );
        assert_eq!(
            join(&mut events, [0.0; 3], 72, &[14690]),
            Some(DanceTeam::Werewolf)
        );
        cast(&mut events, 71, DanceTeam::Vampire);
        cast(&mut events, 72, DanceTeam::Vampire);
        tick(&mut events, 1_000, &players);
        tick(&mut events, 1_100, &players);
        let cake = events.runs["dance"].guid(1);
        let loser = events.runs["dance"].guid(2);
        assert!(events.interact(loser, 71, WORLD, [0.0; 3]).is_empty());
        assert_eq!(
            events.interact(cake, 73, WORLD, [0.0; 3]),
            [crate::chat::string_say(cake, 432505)],
            "a non-participant is refused with the participant line"
        );
        assert_eq!(
            events.interact(cake, 71, WORLD, [0.0; 3]),
            [crate::chat::string_say(cake, 432434)]
        );
        assert_eq!(
            events.interact(cake, 72, WORLD, [0.0; 3]),
            [crate::chat::string_say(cake, 432434)],
            "the losing team's participant may use the winning cake; the cake speaks"
        );
        assert_eq!(
            events.interact(cake, 71, WORLD, [0.0; 3]),
            [crate::chat::string_say(cake, 432430)],
            "the second use is refused"
        );
        assert_eq!(
            events.interact(cake, 74, WORLD, [100.0, 0.0, 0.0]),
            Vec::<Vec<u8>>::new(),
            "out of reach"
        );
        let live = events.runs["dance"].dance_off.as_ref().unwrap();
        assert_eq!(
            live.used_cake.iter().copied().collect::<BTreeSet<_>>(),
            BTreeSet::from([71, 72])
        );
        assert_eq!(live.teams.len(), 2);
    }

    #[test]
    fn the_prompt_is_the_client_26_3_and_the_lines_are_the_client_15_4() {
        let mut events = dance_run(5);
        let players = registry_with(1, [0.0; 3]);
        tick(&mut events, 0, &players);
        let box_guid = events.runs["dance"].guid(0);
        let replies = events.interact(box_guid, 81, WORLD, [0.0; 3]);
        assert_eq!(replies.len(), 1);
        let mut expected = vec![26, 0, 3, 0];
        for value in [432429u32, 0, box_guid as u32, (box_guid >> 32) as u32] {
            expected.extend(value.to_le_bytes());
        }
        expected.push(0);
        expected.extend(0u32.to_le_bytes());
        expected.extend(2u32.to_le_bytes());
        for row in [[401u32, 17, 432207, 0, 18], [402, 17, 432208, 0, 18]] {
            for value in row {
                expected.extend(value.to_le_bytes());
            }
        }
        expected.extend([0u8; 32]); // two zero float4s
        expected.push(0);
        expected.extend([0u8; 16]); // the final vector
        expected.extend(0u32.to_le_bytes());
        expected.extend([0, 0, 0]);
        expected.extend(0u32.to_le_bytes());
        expected.extend(0u32.to_le_bytes());
        assert_eq!(replies[0], expected, "reader 00a9ef10, 93 + 20*n bytes");
        assert_eq!(
            events.vote(81, 401, WORLD, [0.0; 3]),
            Some(vec![crate::npc_interaction::encode_dialogue_close_26_4()])
        );
        assert_eq!(events.interact(box_guid, 82, WORLD, [0.0; 3]).len(), 1);
        events.clear_vote_prompt(82);
        assert!(events.vote(82, 1, WORLD, [0.0; 3]).is_none());
    }

    #[test]
    fn participation_votes_and_cake_use_are_kept_by_character_guid() {
        let mut events = dance_run(13);
        let players = registry_with(1, [0.0; 3]);
        tick(&mut events, 0, &players);
        assert_eq!(
            join(&mut events, [0.0; 3], 91, &[14690]),
            Some(DanceTeam::Werewolf)
        );
        cast(&mut events, 91, DanceTeam::Werewolf);
        // The run's state names no session: a tick with a different registry, as after a
        // disconnect and a reconnect, keeps the character's team and vote.
        let other = registry_with(2, [50.0, 0.0, 0.0]);
        tick(&mut events, 200, &other);
        let live = events.runs["dance"].dance_off.as_ref().unwrap();
        assert_eq!(live.teams.get(&91), Some(&DanceTeam::Werewolf));
        assert_eq!(live.votes.get(&91), Some(&DanceTeam::Werewolf));
        assert_eq!(join(&mut events, [0.0; 3], 91, &[14690]), None);
        let box_guid = events.runs["dance"].guid(0);
        assert_eq!(events.interact(box_guid, 91, WORLD, [0.0; 3]).len(), 1);
        let again = events.vote(91, 401, WORLD, [0.0; 3]).unwrap();
        assert_eq!(again[0], crate::chat::string_say(box_guid, 5252));
        assert_eq!(
            events.runs["dance"].dance_off.as_ref().unwrap().votes.len(),
            1
        );
    }
}
