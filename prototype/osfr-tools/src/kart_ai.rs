//! C6: server-driven AI racers for the kart race (`data/content/kart_ai.json`).
//!
//! ORIGINAL facts this module implements (the retail capture `racing-*-op33-sub*.bin` and the client's
//! own readers; nothing else is named or invented):
//!
//! * the live server filled a race with seven AI racers: a `35/2` AddNpc (the driver's name and body),
//!   then `33/11` CreateProxiedVehicleRace, `33/5` and `33/14` per racer, ahead of `33/4` NpcCount 7 and
//!   `33/1` state 4 (the dispatcher `00ba74d0` case 0xb accepts a creation only while the state is
//!   below 4, and state 4 waits for as many loaded NPC racers as `33/4` names). The captured seven
//!   35/2 bodies (frames 16794-17349) name Lily Leadfoot 19358, Zoomer Rex 19359, Jet Swiftpass 19363,
//!   Layla Octane 19361, Mai Redline 19362, Connor Rush 19364 and Screamin' Steven 19344; they differ
//!   only in actor guid, name, model and grid position, so one captured body is the template.
//! * the AI vehicle block ends with `u64 racer guid | 3 x 3 f32 basis | 3 f32 position | u8 | u8 slot`
//!   (the tail is 58 bytes in all six captured blocks); the racer guid is `(slot * 0x10) << 32 |
//!   0x01700652`.
//! * a `33/5` pose of a non-local racer names its racer by the slot in bits 3-5 of its control byte
//!   (case 5 of `00ba74d0`); the captured AI frames carry `slot << 3 | 0x40` and a monotone clock tail.
//! * the place is not computed by the client: every `33/14` carries it. `i32 @0x08` is the racer's place
//!   (0 = first), `f32 @0x0c` its gap to the leader in metres, `f32 @0x10` its distance still to run
//!   (the racer's `+0x1614`, which the results sort reads), `f32 @0x1d` the racer's current-lap clock
//!   (it restarts when a lap closes) and the four
//!   `u64` at `@0x29..` the four leading racer guids in order. The local racer's card is read from its
//!   own `33/14`: the top three from the guid list, the rest from `i32 @0x08` (`00ba1ea0`).
//! * the live server sent all eight racers' `33/14` in place order about every 50 ms.
//! * the results board lists a racer only once `laps >= lap limit`, ordered by laps, then total time
//!   (`00b9abe0`, `00ba2700`, Lua `RaceResults`).
//!
//! RECONSTRUCTED (R) here, all tunable in `kart_ai.json`: the AI follows the track's ordered gate
//! waypoints as a closed loop at `loop length / lap_seconds * pace`; a racer starts on its grid slot
//! (the captured grid offsets) and joins the loop over `join_ms`; the lap times per track, the per-slot
//! pace, the rival rows, and the order of the guessed coverage-track gates. The Wildwood loop length
//! (1805 m) matches the capture's lap (the leader's 5388 m to run over three laps).
//!
//! ponytail: one pure function of the race clock per racer; the only state is what was already sent.
use crate::{
    content::{SourceTag, check_sources},
    kart_entry::{self, ProxiedVehicle},
    kart_progress_native::{self, ProgressPayload},
    kart_service::{GateRule, KartServiceContent, ScheduledFrame},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read},
    path::Path,
    sync::Arc,
};

pub const FILE: &str = "kart_ai.json";
const MAX_FILE: usize = 256 * 1024;
/// ORIGINAL: the client keeps eight racer slots; slot 0 is the local racer.
pub const MAX_AI: usize = 7;
const MAX_BLOCK: usize = 8192;
/// ORIGINAL: bytes after the owner guid in a 33/11 block: guid + pose (48) + `u8` + `u8 slot`.
const BLOCK_TAIL: usize = 58;
const GUID_LOW: u64 = 0x0170_0652;
/// ORIGINAL: `FLT_MAX`, the captured not-yet-finished `33/14` distance and best lap.
const FLT_MAX_BITS: u32 = 0x7f7f_ffff;
/// (R) Base of the AI pose clock tail; the capture's is a client clock in the 2.88e9 range.
const CLOCK_BASE: u32 = 2_881_000_000;
const BIG: f64 = 1.0e9;

fn d_pose() -> u64 {
    50
}
fn d_board() -> u64 {
    500
}
fn d_join() -> u64 {
    6_000
}
fn d_true() -> bool {
    true
}

/// One AI racer's tunables. Slot 1..=7; the grid offset is metres to the right of and ahead of the
/// local racer's grid slot (the captured grid).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRacerSpec {
    pub slot: u8,
    pub name_string_id: i32,
    pub model_id: i32,
    /// Fraction of the track's top lap speed.
    pub pace: f32,
    pub right_m: f32,
    pub ahead_m: f32,
    /// Index into `vehicles`: which captured kart body this racer drives.
    pub look: usize,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackPace {
    /// Seconds the pace-1.0 racer needs for one loop of the track's gate waypoints.
    pub lap_seconds: f32,
}

/// A rival a row names ("Place higher than ..."): it replaces the driver of `slot` for that row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rival {
    pub slot: u8,
    pub name_string_id: i32,
    pub model_id: i32,
    pub pace: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRow {
    pub game_id: u32,
    /// Multiplies every AI pace on this row; unset is 1.
    #[serde(default)]
    pub pace_scale: Option<f32>,
    #[serde(default)]
    pub rival: Option<Rival>,
}

/// One captured AI kart body: the `33/11` name, its first attribute and the verbatim block.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiVehicleSpec {
    pub body_asset: String,
    pub first_attribute: f32,
    pub block_hex: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AiFile {
    schema_version: u32,
    #[serde(default = "d_pose")]
    pose_interval_ms: u64,
    #[serde(default = "d_board")]
    board_interval_ms: u64,
    #[serde(default = "d_join")]
    join_ms: u64,
    /// Send each racer's 35/2 AddNpc first (the driver's name); (R) from the capture's frame gaps.
    #[serde(default = "d_true")]
    driver_npcs: bool,
    /// One captured 35/2 AddNpc (334 bytes, the inner frame); a driver is this body with its guid,
    /// name, model and grid position replaced.
    driver_npc_hex: String,
    racers: Vec<AiRacerSpec>,
    tracks: BTreeMap<String, TrackPace>,
    rows: Vec<AiRow>,
    vehicles: Vec<AiVehicleSpec>,
    source: Vec<SourceTag>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(text.get(i * 2..i * 2 + 2)?, 16).ok())
        .collect()
}

/// The ORIGINAL racer guid of a slot.
pub fn racer_guid(slot: u8) -> u64 {
    ((u64::from(slot) * 0x10) << 32) | GUID_LOW
}

/// The loaded table: dormant (`None`) without `kart_ai.json`.
#[derive(Clone, Debug)]
pub struct KartAi {
    file: AiFile,
    shared: Arc<Shared>,
}

/// The captured bytes every field of a race reuses; shared, never copied per race.
#[derive(Debug)]
struct Shared {
    /// (body asset, first attribute) per captured kart.
    vehicles: Vec<(String, f32)>,
    blocks: Vec<Vec<u8>>,
    driver: Vec<u8>,
}

impl KartAi {
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let path = directory.join(FILE);
        let metadata = match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
            Ok(m) => m,
        };
        if !metadata.is_file() {
            return Err(invalid("kart ai must be a regular file"));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take((MAX_FILE + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_FILE {
            return Err(invalid("kart ai exceeds 256 KiB"));
        }
        let file: AiFile = serde_json::from_slice(&bytes).map_err(invalid)?;
        let finite = |v: f32| v.is_finite();
        let mut slots = BTreeSet::new();
        if file.schema_version != 1
            || file.racers.is_empty()
            || file.racers.len() > MAX_AI
            || file.vehicles.is_empty()
            || file.vehicles.len() > MAX_AI
            || !(10..=5_000).contains(&file.pose_interval_ms)
            || !(20..=10_000).contains(&file.board_interval_ms)
            || file.join_ms > 60_000
            || file.rows.len() > 128
            || file.tracks.is_empty()
            || file.tracks.len() > 16
        {
            return Err(invalid("invalid kart ai file"));
        }
        for racer in &file.racers {
            if !(1..=MAX_AI as u8).contains(&racer.slot)
                || !slots.insert(racer.slot)
                || !(0.2..=1.5).contains(&racer.pace)
                || !finite(racer.right_m)
                || !finite(racer.ahead_m)
                || racer.right_m.abs() > 200.0
                || racer.ahead_m.abs() > 500.0
                || racer.look >= file.vehicles.len()
                || racer.name_string_id <= 0
                || racer.model_id <= 0
            {
                return Err(invalid(format!("invalid kart ai racer {}", racer.slot)));
            }
        }
        for pace in file.tracks.values() {
            if !(5.0..=600.0).contains(&pace.lap_seconds) {
                return Err(invalid("invalid kart ai lap seconds"));
            }
        }
        let mut rows = BTreeSet::new();
        for row in &file.rows {
            let scale_ok = row.pace_scale.is_none_or(|s| (0.1..=2.0).contains(&s));
            let rival_ok = row.rival.as_ref().is_none_or(|r| {
                slots.contains(&r.slot)
                    && (0.2..=1.5).contains(&r.pace)
                    && r.name_string_id > 0
                    && r.model_id > 0
            });
            if row.game_id == 0 || !rows.insert(row.game_id) || !scale_ok || !rival_ok {
                return Err(invalid(format!("invalid kart ai row {}", row.game_id)));
            }
        }
        let mut blocks = Vec::new();
        for vehicle in &file.vehicles {
            let block = unhex(&vehicle.block_hex)
                .filter(|b| (BLOCK_TAIL..=MAX_BLOCK).contains(&b.len()))
                .ok_or_else(|| invalid("kart ai vehicle block"))?;
            if vehicle.body_asset.is_empty()
                || vehicle.body_asset.len() > 128
                || !vehicle.first_attribute.is_finite()
            {
                return Err(invalid("kart ai vehicle"));
            }
            blocks.push(block);
        }
        let driver = unhex(&file.driver_npc_hex)
            .filter(|d| d.len() >= 53 && d.len() <= 1024 && d[..4] == [35, 0, 2, 0])
            .ok_or_else(|| invalid("kart ai driver body"))?;
        let vehicles = file
            .vehicles
            .iter()
            .map(|v| (v.body_asset.clone(), v.first_attribute))
            .collect();
        check_sources("kart ai", &file.source).map_err(invalid)?;
        let shared = Arc::new(Shared {
            vehicles,
            blocks,
            driver,
        });
        Ok(Some(Self { file, shared }))
    }

    /// The field row `row` races against on `scene`, when the file names that track. `basis` is the
    /// track's start basis (row-major 3 x 3, forward in row 2) and `race.start_position` the local grid.
    pub fn field(
        &self,
        row: Option<u32>,
        scene: &str,
        race: &KartServiceContent,
        basis: [f32; 9],
    ) -> Option<AiField> {
        let pace = self.file.tracks.get(scene)?;
        let row_cfg = row.and_then(|id| self.file.rows.iter().find(|r| r.game_id == id));
        let scale = f64::from(row_cfg.and_then(|r| r.pace_scale).unwrap_or(1.0));
        let points: Vec<[f64; 3]> = race
            .gates
            .iter()
            .map(|g| g.position.map(f64::from))
            .collect();
        let path = Path3::new(points)?;
        let len = path.len;
        let start = race.start_position.map(f64::from);
        let s_grid = path.project([start[0], start[2]]);
        let finish_index = match race.gate_rule {
            Some(GateRule::Coverage { finish_gate, .. }) => (finish_gate as usize).saturating_sub(1),
            None => path.pts.len() - 1,
        }
        .min(path.pts.len() - 1);
        let s_finish = path.cum[finish_index];
        // Metres of the first lap: grid to the finish gate along the loop, a full loop at least
        // half long (a coverage lap cannot close within the first leaves).
        let mut d0 = (s_finish - s_grid).rem_euclid(len);
        if d0 < 0.5 * len {
            d0 += len;
        }
        let laps = race.laps.max(1);
        let total = d0 + f64::from(laps - 1) * len;
        let top_speed = len / f64::from(pace.lap_seconds);
        let right = [f64::from(basis[0]), f64::from(basis[2])];
        let fwd = [f64::from(basis[6]), f64::from(basis[8])];
        let mut racers = Vec::new();
        for spec in &self.file.racers {
            let rival = row_cfg
                .and_then(|r| r.rival.as_ref())
                .filter(|r| r.slot == spec.slot);
            let (name_string_id, model_id, spec_pace) = match rival {
                Some(r) => (r.name_string_id, r.model_id, f64::from(r.pace)),
                None => (spec.name_string_id, spec.model_id, f64::from(spec.pace) * scale),
            };
            let (r_m, a_m) = (f64::from(spec.right_m), f64::from(spec.ahead_m));
            let grid = [
                start[0] + right[0] * r_m + fwd[0] * a_m,
                start[1],
                start[2] + right[1] * r_m + fwd[1] * a_m,
            ];
            racers.push(Racer {
                slot: spec.slot,
                guid: racer_guid(spec.slot),
                name_string_id,
                model_id,
                speed: top_speed * spec_pace,
                ahead: a_m,
                grid,
                look: spec.look,
                rival: rival.is_some(),
            });
        }
        racers.sort_by_key(|r| r.slot);
        Some(AiField {
            shared: Arc::clone(&self.shared),
            driver_npcs: self.file.driver_npcs,
            racers,
            path,
            s_grid,
            d0,
            total,
            laps,
            basis,
            join_s: self.file.join_ms as f64 / 1000.0,
            pose_interval_ms: self.file.pose_interval_ms,
            board_interval_ms: self.file.board_interval_ms,
            started_ms: None,
            next_pose_ms: 0,
            next_board_ms: 0,
            p_local: 0.0,
            laps_done: 0,
            local_lap_start_s: 0.0,
            finished: None,
        })
    }
}

#[derive(Clone, Debug)]
struct Path3 {
    pts: Vec<[f64; 3]>,
    /// Arc length at each point.
    cum: Vec<f64>,
    len: f64,
}

impl Path3 {
    fn new(pts: Vec<[f64; 3]>) -> Option<Self> {
        if pts.len() < 3 || pts.iter().flatten().any(|v| !v.is_finite()) {
            return None;
        }
        let mut cum = vec![0.0];
        for i in 1..pts.len() {
            let last = *cum.last()?;
            cum.push(last + dist2(pts[i - 1], pts[i]));
        }
        let len = cum.last()? + dist2(*pts.last()?, pts[0]);
        (len > 10.0).then_some(Self { pts, cum, len })
    }

    fn seg_end(&self, i: usize) -> [f64; 3] {
        self.pts[(i + 1) % self.pts.len()]
    }

    /// The point `arc` metres along the loop.
    fn at(&self, arc: f64) -> [f64; 3] {
        let arc = arc.rem_euclid(self.len);
        let i = match self.cum.binary_search_by(|c| c.partial_cmp(&arc).unwrap_or(std::cmp::Ordering::Less)) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let a = self.pts[i];
        let b = self.seg_end(i);
        let seg = dist2(a, b).max(1e-6);
        let t = ((arc - self.cum[i]) / seg).clamp(0.0, 1.0);
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    }

    /// The arc coordinate of the loop point nearest to the ground position `xz`.
    fn project(&self, xz: [f64; 2]) -> f64 {
        let mut best = (f64::MAX, 0.0);
        for i in 0..self.pts.len() {
            let (a, b) = (self.pts[i], self.seg_end(i));
            let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
            let l2 = (dx * dx + dz * dz).max(1e-9);
            let t = (((xz[0] - a[0]) * dx + (xz[1] - a[2]) * dz) / l2).clamp(0.0, 1.0);
            let (px, pz) = (a[0] + t * dx, a[2] + t * dz);
            let d = (xz[0] - px).hypot(xz[1] - pz);
            if d < best.0 {
                best = (d, self.cum[i] + t * l2.sqrt());
            }
        }
        best.1
    }
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

#[derive(Clone, Debug)]
struct Racer {
    slot: u8,
    guid: u64,
    name_string_id: i32,
    model_id: i32,
    /// Metres per second along the loop.
    speed: f64,
    /// Metres ahead of the local grid slot at the start.
    ahead: f64,
    grid: [f64; 3],
    look: usize,
    rival: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Who {
    Local,
    Ai(usize),
}

/// The local racer's result once its last lap is in.
#[derive(Clone, Copy, Debug)]
struct LocalFinish {
    at_ms: u64,
    place: u8,
}

/// What the local racer's own `33/14` carries beyond its lap data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalBoard {
    /// Place minus one.
    pub slot: i32,
    pub gap_m: f32,
    pub remaining_m: f32,
    pub clock_s: f32,
    pub leaders: [u64; 4],
}

/// One race's AI racers. Session-local: every pose is a pure function of the race clock.
#[derive(Clone, Debug)]
pub struct AiField {
    shared: Arc<Shared>,
    driver_npcs: bool,
    racers: Vec<Racer>,
    path: Path3,
    /// The arc coordinate of the loop point nearest the local grid slot.
    s_grid: f64,
    /// Metres of the first lap.
    d0: f64,
    /// Metres of the whole race for the local racer.
    total: f64,
    laps: u32,
    basis: [f32; 9],
    join_s: f64,
    pose_interval_ms: u64,
    board_interval_ms: u64,
    started_ms: Option<u64>,
    next_pose_ms: u64,
    next_board_ms: u64,
    /// The local racer's progress in metres since its grid slot.
    p_local: f64,
    laps_done: u32,
    /// Race second the local racer's current lap began.
    local_lap_start_s: f64,
    finished: Option<LocalFinish>,
}

fn f32le(out: &mut Vec<u8>, values: &[f32]) {
    for v in values {
        out.extend(v.to_le_bytes());
    }
}

impl AiField {
    /// Racers; the `33/4` count.
    pub fn count(&self) -> u32 {
        self.racers.len() as u32
    }

    /// The row's rival slot and name, when the row names one.
    pub fn rival_slot(&self) -> Option<u8> {
        self.racers.iter().find(|r| r.rival).map(|r| r.slot)
    }

    fn ai_total(&self, r: &Racer) -> f64 {
        (self.total - r.ahead).max(1.0)
    }

    /// Race seconds at which AI `r` crosses the line for the last time.
    fn finish_s(&self, r: &Racer) -> f64 {
        self.ai_total(r) / r.speed
    }

    fn progress(&self, r: &Racer, t_s: f64) -> f64 {
        r.ahead + r.speed * t_s.max(0.0)
    }

    /// Laps an AI has closed by progress `p`.
    fn laps_of(&self, p: f64) -> u32 {
        if p < self.d0 {
            0
        } else {
            (((p - self.d0) / self.path.len + 1e-9).floor() as u32 + 1).min(self.laps)
        }
    }

    /// (best lap seconds, total seconds) of an AI at race second `t_s`.
    fn times(&self, r: &Racer, t_s: f64) -> (f64, f64) {
        let t_s = t_s.min(self.finish_s(r));
        let p = self.progress(r, t_s);
        let done = self.laps_of(p);
        if done == 0 {
            return (0.0, 0.0);
        }
        let crossing = |k: u32| (self.d0 + f64::from(k - 1) * self.path.len - r.ahead) / r.speed;
        let mut best = crossing(1).max(0.0);
        for k in 2..=done {
            best = best.min(crossing(k) - crossing(k - 1));
        }
        (best, crossing(done).max(0.0))
    }

    /// Seconds into the racer's current lap at race second `t_s` (all of it on lap 1); a finished
    /// racer shows its last lap.
    fn lap_clock(&self, r: &Racer, t_s: f64) -> f64 {
        let t_s = t_s.min(self.finish_s(r));
        let done = self.laps_of(self.progress(r, t_s));
        let crossing = |k: u32| (self.d0 + f64::from(k - 1) * self.path.len - r.ahead) / r.speed;
        if done == 0 {
            t_s
        } else if done >= self.laps {
            if done >= 2 {
                crossing(done) - crossing(done - 1)
            } else {
                crossing(1)
            }
        } else {
            (t_s - crossing(done)).max(0.0)
        }
    }

    /// (key, progress, finished) of each racer at race second `t_s`: the sort key puts a finished
    /// racer ahead of every unfinished one, earlier finish first.
    fn standings(&self, t_s: f64) -> Vec<(Who, f64, f64)> {
        let mut rows: Vec<(Who, f64, f64)> = self
            .racers
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let p = self.progress(r, t_s);
                let key = if t_s >= self.finish_s(r) {
                    BIG - self.finish_s(r)
                } else {
                    p
                };
                (Who::Ai(i), key, p.min(self.total))
            })
            .collect();
        let (key, p) = match self.finished {
            Some(done) => (BIG - (done.at_ms as f64 / 1000.0).max(0.0), self.total),
            None => (self.p_local, self.p_local.min(self.total)),
        };
        rows.push((Who::Local, key, p));
        // Descending key; the slot orders ties so the board is stable.
        rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        rows
    }

    fn guid_of(&self, who: Who, local_guid: u64) -> u64 {
        match who {
            Who::Local => local_guid,
            Who::Ai(i) => self.racers[i].guid,
        }
    }

    fn race_s(&self, now_ms: u64) -> f64 {
        self.started_ms
            .map_or(0.0, |s| now_ms.saturating_sub(s) as f64 / 1000.0)
    }

    /// The vehicle, grid pose and driver frames of the field, in the capture's order: for each racer
    /// the 35/2 AddNpc, 33/11, 33/5 and 33/14.
    pub fn intro_frames(&self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let basis = self.basis;
        for r in &self.racers {
            if self.driver_npcs {
                out.push(self.driver_frame(r));
            }
            let look = r.look.min(self.shared.vehicles.len() - 1);
            let spec = &self.shared.vehicles[look];
            let mut block = self.shared.blocks[look.min(self.shared.blocks.len() - 1)].clone();
            let at = block.len() - BLOCK_TAIL;
            block[at..at + 8].copy_from_slice(&r.guid.to_le_bytes());
            let mut pose = Vec::with_capacity(48);
            f32le(&mut pose, &basis);
            f32le(&mut pose, &[r.grid[0] as f32, r.grid[1] as f32, r.grid[2] as f32]);
            block[at + 8..at + 56].copy_from_slice(&pose);
            let n = block.len();
            block[n - 2] = 0;
            block[n - 1] = r.slot;
            let vehicle = ProxiedVehicle {
                body_asset: spec.0.clone(),
                first_attribute: spec.1,
                attribute_block: block,
            };
            out.push(kart_entry::encode_inner(
                kart_entry::CREATE_VEHICLE_SUBTYPE,
                &vehicle.encode(),
            ));
            out.push(kart_entry::encode_inner(
                kart_entry::CLIENT_DATA_SUBTYPE,
                &self.pose_body(r.grid, basis, [0.0; 3], r.slot, 0, false),
            ));
            out.push(kart_entry::encode_inner(
                kart_progress_native::PROGRESS_SUBTYPE,
                &kart_progress_native::encode_progress(&ProgressPayload {
                    racer_guid: r.guid,
                    slot: 0,
                    unknown_0c_bits: 0,
                    unknown_10_bits: FLT_MAX_BITS,
                    laps_completed: 0,
                    local_racer: 0,
                    lap_limit: self.laps as i32,
                    unknown_1d_bits: 0,
                    best_lap_bits: FLT_MAX_BITS,
                    total_time_bits: 0,
                    tail_0x29: 0,
                    tail_0x31: 0,
                    tail_0x39: 0,
                    tail_0x41: 0,
                }),
            ));
        }
        out
    }

    /// The captured AddNpc body with this racer's guid, name, model and grid position.
    fn driver_frame(&self, r: &Racer) -> Vec<u8> {
        let mut frame = self.shared.driver.clone();
        frame[4..12].copy_from_slice(&r.guid.to_le_bytes());
        frame[12..16].copy_from_slice(&r.name_string_id.to_le_bytes());
        frame[16..20].copy_from_slice(&r.model_id.to_le_bytes());
        for (axis, value) in r.grid.iter().enumerate() {
            let at = 37 + axis * 4;
            frame[at..at + 4].copy_from_slice(&(*value as f32).to_le_bytes());
        }
        frame
    }

    /// An 85-byte `33/5` pose; the control byte carries the slot in bits 3-5 and 0x40 while moving.
    fn pose_body(
        &self,
        pos: [f64; 3],
        basis: [f32; 9],
        velocity: [f32; 3],
        slot: u8,
        clock_ms: u64,
        moving: bool,
    ) -> Vec<u8> {
        let mut body = Vec::with_capacity(85);
        f32le(&mut body, &basis);
        f32le(&mut body, &[pos[0] as f32, pos[1] as f32, pos[2] as f32]);
        f32le(&mut body, &velocity);
        f32le(&mut body, &[0.0, 0.0, 0.0, 0.0]);
        body.push(slot << 3 | if moving { 0x40 } else { 0 });
        body.extend(CLOCK_BASE.wrapping_add(clock_ms as u32).to_le_bytes());
        body.extend(1u32.to_le_bytes());
        body
    }

    /// Go: the clock starts and the first poses and board go out on the next tick.
    pub fn start(&mut self, now_ms: u64) {
        self.started_ms = Some(now_ms);
        self.next_pose_ms = now_ms;
        self.next_board_ms = now_ms;
        self.p_local = 0.0;
        self.laps_done = 0;
        self.local_lap_start_s = 0.0;
        self.finished = None;
    }

    /// The racer's pose at race second `t_s`: on the loop, plus the grid offset that fades over
    /// `join_s`, heading along the loop.
    fn pose_at(&self, r: &Racer, t_s: f64) -> ([f64; 3], [f32; 9], [f32; 3]) {
        let arc = self.s_grid + self.progress(r, t_s);
        let on_loop = self.path.at(arc);
        let on_grid = self.path.at(self.s_grid + r.ahead);
        let fade = if self.join_s <= 0.0 {
            0.0
        } else {
            let x = (t_s / self.join_s).clamp(0.0, 1.0);
            1.0 - x * x * (3.0 - 2.0 * x)
        };
        let pos = [
            on_loop[0] + (r.grid[0] - on_grid[0]) * fade,
            on_loop[1] + (r.grid[1] - on_grid[1]) * fade,
            on_loop[2] + (r.grid[2] - on_grid[2]) * fade,
        ];
        let (a, b) = (self.path.at(arc - 6.0), self.path.at(arc + 6.0));
        let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
        let n = dx.hypot(dz).max(1e-9);
        let (sin, cos) = (dx / n, dz / n);
        let basis = [
            cos as f32, 0.0, -sin as f32, 0.0, 1.0, 0.0, sin as f32, 0.0, cos as f32,
        ];
        let v = r.speed as f32;
        (pos, basis, [sin as f32 * v, 0.0, cos as f32 * v])
    }

    /// Feeds the local racer's latest position and closed laps. Progress is the loop distance since
    /// the grid, on the branch nearest to the last one, held between the closed-lap bounds.
    pub fn observe_local(&mut self, position: [f32; 3], laps_done: u32, now_ms: u64) {
        if self.finished.is_some() {
            return;
        }
        if laps_done.min(self.laps) > self.laps_done {
            self.local_lap_start_s = self.race_s(now_ms);
        }
        let len = self.path.len;
        let s = self.path.project([f64::from(position[0]), f64::from(position[2])]);
        // Progress candidates differ by whole loops; take the one nearest the last.
        let base = (s - self.s_grid).rem_euclid(len);
        let m = ((self.p_local - base) / len).round();
        let mut p = base + m * len;
        let k = laps_done.min(self.laps);
        if k >= 1 {
            p = p.max(self.d0 + f64::from(k - 1) * len);
        }
        if k < self.laps {
            p = p.min(self.d0 + f64::from(k) * len - 0.5);
        }
        self.laps_done = k;
        self.p_local = p.max(0.0);
    }

    /// The place of the local racer and its row of the board at race time `now_ms`.
    pub fn local_board(&self, now_ms: u64, local_guid: u64) -> LocalBoard {
        let t = self.race_s(now_ms);
        let rows = self.standings(t);
        let place = rows.iter().position(|r| r.0 == Who::Local).unwrap_or(0);
        let leader_p = rows.first().map_or(0.0, |r| r.2);
        let own_p = rows.get(place).map_or(0.0, |r| r.2);
        let leaders = std::array::from_fn(|i| {
            rows.get(i).map_or(0, |r| self.guid_of(r.0, local_guid))
        });
        LocalBoard {
            slot: place as i32,
            gap_m: (leader_p - own_p).max(0.0) as f32,
            remaining_m: (self.total - own_p).max(0.0) as f32,
            clock_s: (t - self.local_lap_start_s).max(0.0) as f32,
            leaders,
        }
    }

    /// The racer rows of a board update at `now_ms`.
    fn board_frames(&self, now_ms: u64, local_guid: u64) -> Vec<Vec<u8>> {
        let t = self.race_s(now_ms);
        let rows = self.standings(t);
        let leader_p = rows.first().map_or(0.0, |r| r.2);
        let leaders: [u64; 4] =
            std::array::from_fn(|i| rows.get(i).map_or(0, |r| self.guid_of(r.0, local_guid)));
        let mut out = Vec::new();
        for (place, (who, _, p)) in rows.iter().enumerate() {
            let Who::Ai(i) = *who else { continue };
            let r = &self.racers[i];
            let (best, total) = self.times(r, t);
            out.push(self.ai_progress(
                r,
                place as i32,
                (leader_p - p).max(0.0),
                (self.total - p).max(0.0),
                self.lap_clock(r, t),
                best,
                total,
                self.laps_of(self.progress(r, t)),
                leaders,
            ));
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn ai_progress(
        &self,
        r: &Racer,
        slot: i32,
        gap: f64,
        remaining: f64,
        clock_s: f64,
        best: f64,
        total: f64,
        laps: u32,
        leaders: [u64; 4],
    ) -> Vec<u8> {
        kart_entry::encode_inner(
            kart_progress_native::PROGRESS_SUBTYPE,
            &kart_progress_native::encode_progress(&ProgressPayload {
                racer_guid: r.guid,
                slot,
                unknown_0c_bits: (gap as f32).to_bits(),
                unknown_10_bits: (remaining as f32).to_bits(),
                laps_completed: laps as i32,
                local_racer: 0,
                lap_limit: self.laps as i32,
                unknown_1d_bits: (clock_s as f32).to_bits(),
                best_lap_bits: (best as f32).to_bits(),
                total_time_bits: (total as f32).to_bits(),
                tail_0x29: leaders[0],
                tail_0x31: leaders[1],
                tail_0x39: leaders[2],
                tail_0x41: leaders[3],
            }),
        )
    }

    /// Due poses and AI board rows at `now_ms`, and whether a board is due (the caller adds the
    /// local racer's own row). Silent until [`Self::start`].
    pub fn tick(&mut self, now_ms: u64, local_guid: u64, boards: bool) -> (Vec<Vec<u8>>, bool) {
        let mut out = Vec::new();
        let Some(started) = self.started_ms else {
            return (out, false);
        };
        let mut board_due = false;
        if now_ms >= self.next_pose_ms {
            self.next_pose_ms = now_ms.saturating_add(self.pose_interval_ms);
            let t = now_ms.saturating_sub(started) as f64 / 1000.0;
            let clock = now_ms.saturating_sub(started);
            for r in &self.racers {
                let (pos, basis, vel) = self.pose_at(r, t);
                let body = self.pose_body(pos, basis, vel, r.slot, clock, true);
                out.push(kart_entry::encode_inner(kart_entry::CLIENT_DATA_SUBTYPE, &body));
            }
        }
        if boards && now_ms >= self.next_board_ms {
            self.next_board_ms = now_ms.saturating_add(self.board_interval_ms);
            out.extend(self.board_frames(now_ms, local_guid));
            board_due = true;
        }
        (out, board_due)
    }

    /// The local racer closed its last lap at `now_ms`: every AI's final `33/14` (an AI still
    /// driving shows its projected total, which its deterministic line fixes), and the local place
    /// (1-based) with whether it beat the row's rival.
    pub fn finish(&mut self, now_ms: u64, local_guid: u64) -> (Vec<Vec<u8>>, u8, Option<bool>) {
        let elapsed_ms = self.started_ms.map_or(0, |s| now_ms.saturating_sub(s));
        self.finished = Some(LocalFinish {
            at_ms: elapsed_ms,
            place: 0,
        });
        let local_s = elapsed_ms as f64 / 1000.0;
        let ahead: Vec<usize> = (0..self.racers.len())
            .filter(|i| self.finish_s(&self.racers[*i]) < local_s)
            .collect();
        let place = ahead.len() as u8 + 1;
        let beat = self
            .racers
            .iter()
            .find(|r| r.rival)
            .map(|rival| self.finish_s(rival) >= local_s);
        if let Some(done) = &mut self.finished {
            done.place = place;
        }
        // Final order: finish time, the local racer among them.
        let mut order: Vec<(Who, f64)> = (0..self.racers.len())
            .map(|i| (Who::Ai(i), self.finish_s(&self.racers[i])))
            .collect();
        order.push((Who::Local, local_s));
        order.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let leaders: [u64; 4] =
            std::array::from_fn(|i| order.get(i).map_or(0, |r| self.guid_of(r.0, local_guid)));
        let mut out = Vec::new();
        for (place, (who, finish)) in order.iter().enumerate() {
            let Who::Ai(i) = *who else { continue };
            let r = &self.racers[i];
            let (best, total) = self.times(r, *finish + 1.0e-6);
            out.push(self.ai_progress(
                r,
                place as i32,
                0.0,
                0.0,
                self.lap_clock(r, *finish),
                best,
                total,
                self.laps,
                leaders,
            ));
        }
        (out, place, beat)
    }

    /// The local racer's place (1-based) once it has finished.
    pub fn placement(&self) -> Option<u8> {
        self.finished.map(|f| f.place)
    }

    /// Whether the local racer beat the row's rival, once finished; `None` on a row with no rival.
    pub fn beat_rival(&self) -> Option<bool> {
        let done = self.finished?;
        let rival = self.racers.iter().find(|r| r.rival)?;
        Some(self.finish_s(rival) >= done.at_ms as f64 / 1000.0)
    }

    /// Applies the field's place to the local racer's `33/14`.
    pub fn apply_local(&self, payload: &mut ProgressPayload, now_ms: u64) {
        let board = match self.finished {
            Some(done) => {
                let local_s = done.at_ms as f64 / 1000.0;
                let mut order: Vec<(Who, f64)> = (0..self.racers.len())
                    .map(|i| (Who::Ai(i), self.finish_s(&self.racers[i])))
                    .collect();
                order.push((Who::Local, local_s));
                order.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                let leaders = std::array::from_fn(|i| {
                    order.get(i).map_or(0, |r| self.guid_of(r.0, payload.racer_guid))
                });
                LocalBoard {
                    slot: i32::from(done.place) - 1,
                    gap_m: 0.0,
                    remaining_m: 0.0,
                    clock_s: (local_s - self.local_lap_start_s).max(0.0) as f32,
                    leaders,
                }
            }
            None => self.local_board(now_ms, payload.racer_guid),
        };
        payload.slot = board.slot;
        payload.unknown_0c_bits = board.gap_m.to_bits();
        payload.unknown_10_bits = board.remaining_m.to_bits();
        payload.unknown_1d_bits = board.clock_s.to_bits();
        payload.tail_0x29 = board.leaders[0];
        payload.tail_0x31 = board.leaders[1];
        payload.tail_0x39 = board.leaders[2];
        payload.tail_0x41 = board.leaders[3];
    }
}

/// Sets every `33/4` step of the race schedule to the field's racer count, so the client waits for
/// exactly the racers the entry creates (the shipped schedule says 0).
pub fn set_npc_count(race: &mut KartServiceContent, count: u32) {
    for step in &mut race.schedule {
        if let ScheduledFrame::NpcCount(value) = &mut step.frame {
            *value = count;
        }
    }
}
