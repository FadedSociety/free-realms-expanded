//! T458: world events, part 1. An event in the content directory's `world_events.json` (beside
//! `mounts.json`) names its actors and a timeline of ops; an admin's `realm event start <key>`
//! runs it in the open world and `realm event stop <key>` ends it. The file is read at each
//! start, so an edit applies to the next run. Runs live in memory only (R): a restart mid-run
//! leaves nothing behind and nothing respawns.
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
//! Visibility: each tick every live actor shows itself (AddNpc, current base animation, live
//! tags) to the registry's nearby sessions it has not been shown to, and forgets sessions that
//! left. Event guids never enter a session's `known_npcs`, so the shared-field NPC budget is
//! untouched. Cost per tick: one emptiness check when idle, else one `nearby_sessions` query
//! per live actor.
use crate::abilities::effect_packets::{
    encode_add_composite_effect, encode_play_composite_effect, encode_remove_composite_effect,
    encode_remove_gracefully,
};
use crate::encounter::animation;
use crate::players::{PlayerRegistry, SessionId, ZoneInstance};
use crate::startup_world::{NpcSpawn, build_add_npc};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::{
    io::{self, Read},
    path::Path,
    sync::Arc,
};

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
}

impl OpRow {
    /// The actor key, the op, and the id it carries (which must be > 0); spawn and remove may
    /// carry none, which is effect 0 on the wire.
    fn split(self) -> (String, Op, Option<i32>) {
        match self {
            Self::Spawn { actor, effect_id } => {
                (actor, Op::Spawn(effect_id.unwrap_or(0)), effect_id)
            }
            Self::Remove { actor, effect_id } => {
                (actor, Op::Remove(effect_id.unwrap_or(0)), effect_id)
            }
            Self::BaseAnim { actor, id } => (actor, Op::BaseAnim(id), Some(id)),
            Self::PlayAnim { actor, id } => (actor, Op::PlayAnim(id), Some(id)),
            Self::Effect { actor, id } => (actor, Op::Effect(id), Some(id)),
            Self::TagOn { actor, effect_id } => (actor, Op::TagOn(effect_id), Some(effect_id)),
            Self::TagOff { actor, effect_id } => (actor, Op::TagOff(effect_id), Some(effect_id)),
        }
    }
}

/// One validated event. Steps are in `at_ms` order; equal times keep file order.
#[derive(Debug, PartialEq)]
pub struct WorldEvent {
    pub key: String,
    actors: Vec<Actor>,
    steps: Vec<Step>,
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
    actor: usize,
    op: Op,
}

/// Each op's composite effect, animation or effect id; 0 is "no poof" for spawn and remove.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    Spawn(i32),
    Remove(i32),
    BaseAnim(i32),
    PlayAnim(i32),
    Effect(i32),
    TagOn(i32),
    TagOff(i32),
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
    let mut steps = Vec::with_capacity(timeline.len());
    for StepRow { at_ms, op } in timeline {
        let (actor, op, id) = op.split();
        let Some(&actor) = index.get(&actor) else {
            return Err(fail(format!("a step names unknown actor {actor}")));
        };
        if at_ms > MAX_AT_MS || id.is_some_and(|id| id <= 0) {
            return Err(fail("every step needs at_ms <= 2 h and ids > 0".into()));
        }
        steps.push(Step { at_ms, actor, op });
    }
    steps.sort_by_key(|step| step.at_ms);
    for step in &steps {
        if let Op::Remove(effect) = step.op {
            actors[step.actor].remove_effect = effect;
        }
    }
    Ok(WorldEvent { key, actors, steps })
}

/// The live runs, at most one per event key, owned by the gateway host's shared world.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldEvents {
    runs: BTreeMap<String, Run>,
    /// Removals for the next tick, sent first: `stop`'s, and the ones a session refused.
    pending: Out,
    serial: u32,
    tag: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct Run {
    event: Arc<WorldEvent>,
    instance: ZoneInstance,
    started_ms: u64,
    serial: u32,
    next_step: usize,
    actors: Vec<Shown>,
}

/// One actor's live state, reset when it is removed.
#[derive(Clone, Debug, Default, PartialEq)]
struct Shown {
    live: bool,
    base_anim: Option<i32>,
    /// Composite effect id -> the tag id its 35/41 used; the 35/42 sends that tag id back.
    tags: BTreeMap<i32, i32>,
    seen_by: BTreeSet<SessionId>,
}

impl WorldEvents {
    /// Start `event` in `instance`; false, changing nothing, while a run of its key is live.
    pub fn start(&mut self, event: WorldEvent, instance: ZoneInstance, now_ms: u64) -> bool {
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
        let run = Run {
            actors: vec![Shown::default(); event.actors.len()],
            event: Arc::new(event),
            instance,
            started_ms: now_ms,
            serial,
            next_step: 0,
        };
        self.runs.insert(run.event.key.clone(), run);
        true
    }

    /// End `key`'s run: the next tick removes every live actor with its remove step's poof, and
    /// the key may start again at once. False when it is not running.
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
        true
    }

    /// Run every step that fell due since the last tick, once each in `at_ms` order, then show
    /// each live actor to the sessions that came near. A run ends once its last step has run
    /// and no actor is live. Returns inner packets per session; idle, it allocates nothing.
    pub fn tick(&mut self, now_ms: u64, players: &PlayerRegistry) -> Out {
        let mut out = std::mem::take(&mut self.pending);
        for run in self.runs.values_mut() {
            while let Some(step) = run
                .event
                .steps
                .get(run.next_step)
                .copied()
                .filter(|step| run.started_ms.saturating_add(step.at_ms) <= now_ms)
            {
                run.next_step += 1;
                if let Err(reason) = run.apply(step, players, &mut self.tag, &mut out) {
                    println!(
                        "{}",
                        serde_json::json!({"event": "world_event_step_skipped",
                            "key": run.event.key, "at_ms": step.at_ms, "reason": reason})
                    );
                }
            }
            for index in 0..run.actors.len() {
                if run.actors[index].live {
                    run.view(index, players, &mut out);
                }
            }
        }
        self.runs.retain(|_, run| {
            run.next_step < run.event.steps.len() || run.actors.iter().any(|shown| shown.live)
        });
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
    /// spawn while it is, a tag that is already on or not on) changes nothing.
    fn apply(
        &mut self,
        step: Step,
        players: &PlayerRegistry,
        tag: &mut i32,
        out: &mut Out,
    ) -> Result<(), &'static str> {
        let (index, guid) = (step.actor, self.guid(step.actor));
        let position = self.event.actors[index].position;
        let shown = &mut self.actors[index];
        if shown.live == matches!(step.op, Op::Spawn(_)) {
            return Err(if shown.live {
                "the actor is already live"
            } else {
                "the actor is not live"
            });
        }
        let packet = match step.op {
            Op::Spawn(effect) => {
                shown.live = true;
                self.view(index, players, out);
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
            Op::Effect(id) => encode_play_composite_effect(guid, id, position),
            Op::TagOn(effect) => {
                if shown.tags.contains_key(&effect) {
                    return Err("the tag is already on");
                }
                // ponytail: one counter for every actor; a tag id repeats only after 2^31 tags.
                *tag = *tag % i32::MAX + 1;
                shown.tags.insert(effect, *tag);
                encode_add_composite_effect(guid, *tag, effect, guid)
            }
            Op::TagOff(effect) => match shown.tags.remove(&effect) {
                Some(tag) => encode_remove_composite_effect(guid, tag),
                None => return Err("the tag is not on"),
            },
        };
        let seen_by = &self.actors[index].seen_by;
        out.extend(seen_by.iter().map(|session| (*session, packet.clone())));
        Ok(())
    }

    /// Show actor `index` to nearby sessions it has not been shown to (AddNpc, its current base
    /// animation, its live tags) and forget sessions that left. One still in the instance gets
    /// the plain 35/3, so walking back shows the actor again.
    fn view(&mut self, index: usize, players: &PlayerRegistry, out: &mut Out) {
        let (guid, instance) = (self.guid(index), self.instance);
        let actor = &self.event.actors[index];
        let shown = &mut self.actors[index];
        let nearby = players.nearby_sessions(instance, actor.position);
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
        let spawn = NpcSpawn {
            actor_id: guid,
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
            events.tick(1_000, &players).is_empty(),
            "nothing is live yet"
        );
        let spawned = events.tick(1_010, &players);
        assert_eq!(spawned.len(), 1, "one AddNpc, not two");
        assert_eq!(&spawned[0].1[..4], &[35, 0, 2, 0]);
        let tagged = events.tick(1_020, &players);
        assert_eq!(tagged.len(), 1, "one 35/41, not two");
        assert_eq!(&tagged[0].1[..4], &[35, 0, 41, 0]);
        assert_eq!(events.tick(1_030, &players).len(), 1, "the removal");
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
        let first = events.tick(0, &players);
        events.serial = SERIALS; // the next serial wraps to 1, which "one" holds
        assert!(events.start(two, WORLD, 0));
        let second = events.tick(0, &players);
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
        assert_eq!(kinds(&events.tick(0, &players)), [[35, 0, 2, 0]]);
        let animated = events.tick(10, &players);
        assert_eq!(kinds(&animated), [[35, 0, 8, 0]]);
        events.refused(SessionId(1), vec![animated[0].1.clone()]);
        let again = events.tick(20, &players);
        assert_eq!(
            kinds(&again),
            [[35, 0, 3, 0], [35, 0, 2, 0], [35, 0, 8, 0]],
            "the removal, then the AddNpc and the animation"
        );
        assert_eq!(again[0].1, encode_remove_gracefully(GUID, 0));
        assert_eq!(&again[1].1[4..12], &GUID.to_le_bytes());
        assert!(
            events.tick(30, &players).is_empty(),
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
        events.tick(0, &players);
        let removal = encode_remove_gracefully(GUID, 5);
        let sent = events.tick(10, &players);
        assert_eq!(sent, [(SessionId(1), removal.clone())]);
        // The run is over, yet the refused removal comes back, once, however often refused.
        events.refused(SessionId(1), sent.iter().map(|(_, p)| p.clone()).collect());
        events.refused(SessionId(1), vec![removal.clone()]);
        assert_eq!(events.tick(20, &players), [(SessionId(1), removal.clone())]);
        events.refused(SessionId(1), vec![removal.clone()]);
        assert_eq!(events.tick(30, &players), [(SessionId(1), removal)]);
        assert!(
            events.tick(40, &players).is_empty(),
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
        events.tick(0, &players);
        assert!(events.stop("show"));
        let sent = events.tick(10, &players);
        assert_eq!(sent, [(SessionId(1), encode_remove_gracefully(GUID, 0))]);
        events.refused(SessionId(1), vec![sent[0].1.clone()]);
        assert_eq!(events.tick(20, &players), sent);
        assert!(events.tick(30, &players).is_empty());
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
        let out = events.tick(5_000, &players);
        assert!(out.is_empty() && out.capacity() == 0);
    }
}
