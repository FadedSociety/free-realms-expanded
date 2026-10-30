//! T556: placed birthday cakes, part 1 of 2. The cake table (`placed_cakes.json`) and a pure
//! registry of live cakes: no I/O, no clock, no items. The caller owns the clock and the
//! recipient lists, sends what `place` and `tick` return, and applies the [`Spec`] a `click`
//! returns (consumable_effects, T473). Part 2 wires the item, its cooldown and the gateway.
//!
//! # Recovered client facts
//!
//! * The cake is an ordinary actor: 35/2 AddNpc, reader `00921a10` (the native field sequence is
//!   `startup_world::NpcSpawn`'s). The reader has no cursor field; the hover cursor rides the
//!   35/12 NpcRelevance entry. Dispatcher case 12 (`0092f460`) reads each entry with `008db5e0`: u64
//!   guid, u8 has-cursor, a signed byte cursor id only when that flag is set, u8 flag; it then calls
//!   `00962f30` on the actor, which stores the id (or -1) at +0x708. Cursor 5 is `cursor_talk`
//!   (`Cursors.txt`); the NPC guide entry (`encounter.rs`) is the same `[1, 5, 0]`.
//! * 35/16 plays a composite effect (`008e5f50`), 35/8 sets an animation (`00908a50`), 35/3 removes
//!   an actor (`008d69f0`, `008d6a60`); the writers are the shared `effect_packets` ones.
//! * Client rows behind each cake (model, name, effects, animation) are cited in the row's
//!   `source`.
//!
//! # Reconstructed (R)
//!
//! * The 60 s lifetime, 5 s click window, 1.5 m spawn offset, 10-20 s one-shot cadence and its
//!   3 s length are Sanctuary's `CakeItemDefinition` defaults; the 5 m click range is this repo's
//!   (R). All are data, bounded here as sanity limits.
//! * One-shot and its return are 35/8 with the base flag set, as Sanctuary sends both (flags 1).
//!   Whether the client plays `amb_oneshot_01` once on the base path is a native check.
//! * The roll is uniform over the row's outcomes, never the cake's previous outcome when it has two
//!   or more (Sanctuary `RollExcluding`). It is a pure hash of the cake, the clicker and the time,
//!   so the registry holds no random state and stays comparable.
//! * Guids: the high word is 'CAKE' (`0x43414b45`), the low word counts live cakes. Characters
//!   count up from 1 (`account_store`), content NPCs sit in 9e9..1e10, pets are
//!   `owner ^ 0x4000.. ^ (pet id << 32)`, rides `owner ^ 0xc000..`; `tests/placed_cakes.rs`
//!   asserts the disjointness.

use crate::{
    abilities::effect_packets::{encode_play_composite_effect, encode_remove_gracefully},
    consumable_effects::{self, Kind, Spec},
    encounter::animation,
    npc_presentation::{Cursor, encode_cursors},
    players::ZoneInstance,
    power_ups::Rng,
    startup_world::{self, NpcSpawn},
    world,
};
use serde::Deserialize;
use std::{fs, io, path::Path, sync::Arc};

const FILE: &str = "placed_cakes.json";
const MAX_FILE: usize = 256 * 1024;
const MAX_ROWS: usize = 64;
const MAX_OUTCOMES: usize = 8;
const MAX_SPAWN_EFFECTS: usize = 8;
const MAX_LIFETIME_MS: u64 = 600_000;
const MAX_WINDOW_MS: u64 = 60_000;
const MAX_COOLDOWN_MS: u64 = 3_600_000;
const MAX_LIVE: usize = 256;
/// The one-shot cadence stays inside this range (R, from the brief).
const SHOT_INTERVAL_MS: std::ops::RangeInclusive<u64> = 10_000..=20_000;
const MAX_SHOT_MS: u64 = 10_000;
/// 'CAKE' in the high word of every cake guid; the low word counts cakes.
pub const GUID_TAG: u64 = 0x4341_4b45_0000_0000;
const GUID_TAG_MASK: u64 = 0xffff_ffff_0000_0000;

/// What a session asks of the shared registry. The host applies it after the commit (T557):
/// `Place` at the placer's registry pose, `Click` for the clicker's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The placer pressed the cake item `row` (a [`Row::item_id`]).
    Place { row: i32 },
    /// The player clicked the actor `guid`; only a live cake answers.
    Click { guid: u64 },
}

/// Whether `guid` is in the cake range ('CAKE' in the high word), live or not.
pub fn is_cake(guid: u64) -> bool {
    guid & GUID_TAG_MASK == GUID_TAG
}

/// What one click does, resolved from the client's own ability tables.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Transform { model_id: i32, poof_effect_id: i32 },
    Aura { composite_effect_id: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub effect: Effect,
    pub duration_ms: u64,
}

/// A boss cake's ambient animation: every `min..=max` ms it plays `animation_id` for
/// `animation_ms`, then returns to the row's own animation.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OneShot {
    pub animation_id: i32,
    pub animation_ms: u64,
    pub min_interval_ms: u64,
    pub max_interval_ms: u64,
}

/// One cake item. `item_cooldown_ms` is part 2's (the item bar); this module only stores it.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub item_id: i32,
    pub name_id: i32,
    pub model_id: i32,
    pub cursor_id: i32,
    pub animation_id: i32,
    pub spawn_effect_ids: Vec<i32>,
    pub lifetime_ms: u64,
    pub item_cooldown_ms: u64,
    pub click_window_ms: u64,
    pub click_range_m: f32,
    pub spawn_offset_m: f32,
    /// The buff-bar icon of the effect a click starts.
    pub icon_id: i32,
    pub one_shot: Option<OneShot>,
    pub outcomes: Vec<Outcome>,
    pub source: String,
}

impl Row {
    fn spec(&self, outcome: &Outcome) -> Spec {
        let kind = match outcome.effect {
            Effect::Transform {
                model_id,
                poof_effect_id,
            } => Kind::Transform {
                model_id,
                poof_effect_id,
            },
            Effect::Aura {
                composite_effect_id,
            } => Kind::Aura {
                composite_effect_id,
                delay_ms: 0,
            },
        };
        Spec {
            kind,
            duration_ms: outcome.duration_ms,
            icon_id: self.icon_id,
            name_id: self.name_id,
        }
    }

    /// Sanity bounds, not tuned values.
    fn valid(&self) -> bool {
        let shot_ok = self.one_shot.is_none_or(|s| {
            s.animation_id > 0
                && (1..=MAX_SHOT_MS).contains(&s.animation_ms)
                && SHOT_INTERVAL_MS.contains(&s.min_interval_ms)
                && SHOT_INTERVAL_MS.contains(&s.max_interval_ms)
                && s.min_interval_ms <= s.max_interval_ms
        });
        self.item_id > 0
            && self.name_id > 0
            && self.model_id > 0
            && (1..=127).contains(&self.cursor_id)
            && self.animation_id > 0
            && (1..=MAX_SPAWN_EFFECTS).contains(&self.spawn_effect_ids.len())
            && self.spawn_effect_ids.iter().all(|id| *id > 0)
            && (1..=MAX_LIFETIME_MS).contains(&self.lifetime_ms)
            && self.item_cooldown_ms <= MAX_COOLDOWN_MS
            && self.click_window_ms <= MAX_WINDOW_MS
            && (0.5..=50.0).contains(&self.click_range_m)
            && (0.0..=10.0).contains(&self.spawn_offset_m)
            && self.icon_id > 0
            && shot_ok
            && (1..=MAX_OUTCOMES).contains(&self.outcomes.len())
            && self
                .outcomes
                .iter()
                .all(|o| consumable_effects::admits(&self.spec(o)).is_ok())
            && !self.source.is_empty()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema_version: u32,
    max_live: usize,
    cakes: Vec<Row>,
}

/// Every cake row, read-only and shared.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalogue {
    rows: Vec<Arc<Row>>,
    max_live: usize,
}

impl Catalogue {
    /// `placed_cakes.json` from the content directory; a missing file is an empty catalogue.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let bytes = match fs::read(directory.join(FILE)) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let invalid = |what: String| io::Error::new(io::ErrorKind::InvalidData, what);
        if bytes.len() > MAX_FILE {
            return Err(invalid("placed_cakes.json exceeds 256 KiB".into()));
        }
        let file: File = serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
        if file.schema_version != 1
            || file.cakes.len() > MAX_ROWS
            || !(1..=MAX_LIVE).contains(&file.max_live)
        {
            return Err(invalid("unsupported placed_cakes.json".into()));
        }
        for (index, row) in file.cakes.iter().enumerate() {
            if !row.valid() || file.cakes[..index].iter().any(|r| r.item_id == row.item_id) {
                return Err(invalid(format!("invalid cake row {}", row.item_id)));
            }
        }
        Ok(Self {
            rows: file.cakes.into_iter().map(Arc::new).collect(),
            max_live: file.max_live,
        })
    }

    pub fn get(&self, item_id: i32) -> Option<&Arc<Row>> {
        self.rows.iter().find(|row| row.item_id == item_id)
    }

    /// The server-wide cap on live cakes, for [`PlacedCakes::new`].
    pub fn max_live(&self) -> usize {
        self.max_live
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A position or heading that is not finite.
    Position,
    /// The server already holds its cap of live cakes.
    Full,
    /// The AddNpc could not be built.
    Spawn,
    /// No live cake has this guid (never placed, or already despawned).
    Unknown,
    /// The clicker stands in another zone instance.
    Instance,
    /// The cake's lifetime is over; the next tick removes it.
    Expired,
    Range,
    /// The click window since the last accepted click is still closed.
    Window,
    /// A caller-built row with no outcome.
    NoOutcome,
}

/// A fresh cake: its guid and the packets for every viewer, in order: AddNpc, the cursor
/// relevance entry, then one 35/16 per spawn effect.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub guid: u64,
    pub spawn: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Shot {
    at_ms: u64,
    playing: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct Cake {
    guid: u64,
    row: Arc<Row>,
    instance: ZoneInstance,
    position: [f32; 3],
    ends_ms: u64,
    last_click_ms: Option<u64>,
    last_outcome: Option<usize>,
    shot: Option<Shot>,
}

/// The live cakes of one server. Session-only: nothing here is saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlacedCakes {
    live: Vec<Cake>,
    next: u32,
    max_live: usize,
}

/// One hash of the cake, who acted and when: every random choice below is a pure function of it.
fn seed(guid: u64, who: u64, now_ms: u64) -> u64 {
    Rng::mix(guid ^ Rng::mix(who ^ Rng::mix(now_ms)))
}

/// Uniform over `0..len`, never `last` when `len >= 2`. Sanctuary's `RollExcluding`.
pub fn roll_excluding(len: usize, last: Option<usize>, seed: u64) -> usize {
    let hash = Rng::mix(seed);
    match last.filter(|l| *l < len && len >= 2) {
        Some(l) => {
            let pick = (hash % (len as u64 - 1)) as usize;
            pick + usize::from(pick >= l)
        }
        None => (hash % len.max(1) as u64) as usize,
    }
}

/// 35/12 with one entry: the cake's hover cursor. Wire in the module doc.
fn cursor_relevance(guid: u64, cursor_id: i32) -> Vec<u8> {
    let id = cursor_id as u8;
    encode_cursors(&[(guid, Cursor { id, flag: false })])
}

impl Cake {
    /// The wait before the next one-shot, drawn in `min..=max` from the cake, `who` and the time.
    fn interval_ms(&self, shot: &OneShot, who: u64, now_ms: u64) -> u64 {
        let span = shot.max_interval_ms - shot.min_interval_ms + 1;
        shot.min_interval_ms + seed(self.guid, who, now_ms) % span
    }

    /// The one-shot packet due at `now_ms`, if any; one step per call.
    fn step_one_shot(&mut self, now_ms: u64) -> Option<Vec<u8>> {
        let row = Arc::clone(&self.row);
        let shot = row.one_shot.as_ref()?;
        let state = self.shot?;
        if now_ms < state.at_ms {
            return None;
        }
        let (packet, next) = if state.playing {
            (
                animation(self.guid, row.animation_id, true),
                Shot {
                    at_ms: now_ms.saturating_add(self.interval_ms(shot, 0, now_ms)),
                    playing: false,
                },
            )
        } else {
            (
                animation(self.guid, shot.animation_id, true),
                Shot {
                    at_ms: now_ms.saturating_add(shot.animation_ms),
                    playing: true,
                },
            )
        };
        self.shot = Some(next);
        Some(packet)
    }
}

impl PlacedCakes {
    pub fn new(max_live: usize) -> Self {
        Self {
            live: Vec::new(),
            next: 0,
            max_live,
        }
    }

    pub fn len(&self) -> usize {
        self.live.len()
    }

    pub fn is_empty(&self) -> bool {
        self.live.is_empty()
    }

    /// The next counter value whose guid no live cake has, counting up through the low word and
    /// wrapping, and that guid. `place` stores the counter only once the cake is built.
    fn free_guid(&self) -> (u32, u64) {
        let mut counter = self.next;
        loop {
            counter = counter.wrapping_add(1);
            let guid = GUID_TAG | u64::from(counter);
            if !self.live.iter().any(|c| c.guid == guid) {
                return (counter, guid);
            }
        }
    }

    /// Puts a cake `row.spawn_offset_m` in front of the placer along `heading`. A refusal
    /// changes nothing.
    pub fn place(
        &mut self,
        row: &Arc<Row>,
        placer: u64,
        instance: ZoneInstance,
        position: [f32; 3],
        heading: f32,
        now_ms: u64,
    ) -> Result<Placed, Refusal> {
        if !position.iter().chain([&heading]).all(|v| v.is_finite()) {
            return Err(Refusal::Position);
        }
        if self.live.len() >= self.max_live {
            return Err(Refusal::Full);
        }
        let (sin, cos) = heading.sin_cos();
        let at = [
            position[0] + sin * row.spawn_offset_m,
            position[1],
            position[2] + cos * row.spawn_offset_m,
        ];
        let (counter, guid) = self.free_guid();
        let npc = NpcSpawn {
            actor_id: guid,
            name_string_id: row.name_id,
            model_id: row.model_id,
            position: [at[0], at[1], at[2], 1.0],
            rotation: [sin, 0.0, cos, 0.0],
            animation_id: row.animation_id,
            ..world::content_spawn_template()
        };
        let add = world::unframe_tunnel(&startup_world::build_add_npc(&npc))
            .map_err(|_| Refusal::Spawn)?;
        let mut spawn = vec![add, cursor_relevance(guid, row.cursor_id)];
        spawn.extend(
            row.spawn_effect_ids
                .iter()
                .map(|id| encode_play_composite_effect(guid, *id, at)),
        );
        let mut cake = Cake {
            guid,
            row: Arc::clone(row),
            instance,
            position: at,
            ends_ms: now_ms.saturating_add(row.lifetime_ms),
            last_click_ms: None,
            last_outcome: None,
            shot: None,
        };
        cake.shot = row.one_shot.as_ref().map(|s| Shot {
            at_ms: now_ms.saturating_add(cake.interval_ms(s, placer, now_ms)),
            playing: false,
        });
        self.next = counter;
        self.live.push(cake);
        Ok(Placed { guid, spawn })
    }

    /// Despawns cakes whose lifetime is over (each exactly once: the cake leaves the registry
    /// with its removal) and advances every boss cake's ambient animation. Each entry is the
    /// cake's instance and position (the caller sends to the sessions near it) and its packets.
    /// Nothing allocates while nothing is due.
    pub fn tick(&mut self, now_ms: u64) -> Vec<(ZoneInstance, [f32; 3], Vec<Vec<u8>>)> {
        let mut out = Vec::new();
        self.live.retain_mut(|cake| {
            if now_ms >= cake.ends_ms {
                let removal = vec![encode_remove_gracefully(cake.guid, 0)];
                out.push((cake.instance, cake.position, removal));
                return false;
            }
            if let Some(packet) = cake.step_one_shot(now_ms) {
                out.push((cake.instance, cake.position, vec![packet]));
            }
            true
        });
        out
    }

    /// One click: the same instance, within range, the window open. The first of two clicks in a
    /// window wins and the second is refused. A refusal changes nothing.
    pub fn click(
        &mut self,
        guid: u64,
        clicker: u64,
        instance: ZoneInstance,
        clicker_position: [f32; 3],
        now_ms: u64,
    ) -> Result<Spec, Refusal> {
        let cake = self
            .live
            .iter_mut()
            .find(|c| c.guid == guid)
            .ok_or(Refusal::Unknown)?;
        let row = Arc::clone(&cake.row);
        if cake.instance != instance {
            return Err(Refusal::Instance);
        }
        if now_ms >= cake.ends_ms {
            return Err(Refusal::Expired);
        }
        let dx = clicker_position[0] - cake.position[0];
        let dz = clicker_position[2] - cake.position[2];
        // Written so a NaN fails the comparison.
        if !(dx.hypot(dz) <= row.click_range_m) {
            return Err(Refusal::Range);
        }
        if cake
            .last_click_ms
            .is_some_and(|last| now_ms < last.saturating_add(row.click_window_ms))
        {
            return Err(Refusal::Window);
        }
        let pick = roll_excluding(
            row.outcomes.len(),
            cake.last_outcome,
            seed(guid, clicker, now_ms),
        );
        let spec = row
            .outcomes
            .get(pick)
            .map(|o| row.spec(o))
            .ok_or(Refusal::NoOutcome)?;
        cake.last_click_ms = Some(now_ms);
        cake.last_outcome = Some(pick);
        Ok(spec)
    }
}
