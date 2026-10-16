//! Piece 011 - pet training activities: recovered trick gestures, validated round
//! completion and the pet-trick wire family.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/011 Pet training activities.md`):
//!
//! * `Resources/PetTrickGesturePoints.txt` - `PET_TRICK_ID^SEQUENCE_ID^X^Y`, waypoints
//!   on a normalised plane; the shipped file has 23 trick ids.
//! * `Resources/PetTrickGestureSettings.txt` - `DRAW_DIST 0.1`, `SAMPLE_DRAW_RATE
//!   0.0005`, `NUM_SEGMENTS 128`, effects 5511/5525/5526/5527,
//!   `HIT_NODE_DIST_SQR 0.0025`, `TIMEOUT_SECONDS 10`, `FORCE_END_MILLISECONDS 20000`
//!   and the locale strings 90155/90157/90161/90163 (results) and 94617/94618
//!   (get ready / draw).
//! * Packet family 53 (`PetBasePackets`): 7 PetPerformTrickPacket, 8
//!   PetTrickGestureResult, 12 PetTrickSkillLevelPacket, 13 PetTrickAddPacket,
//!   14 PetTrickFavoriteSetPacket, 15 PetTrickNotifyPacket.
//! * Client pseudocode: subtype 7 and 8 carry one 32-bit value each
//!   (`FUN_00b5ff50`, `FUN_00b5ffe0`); the favourite request rejects slots outside
//!   `param_2 - 1U < 4` and writes pet, slot-1 and trick (`FUN_00b5f670`).
//! * Community profile serializer: each pet trick is a 32-bit key plus eight 32-bit
//!   fields and one boolean flag.
//!
//! RECONSTRUCTED, labelled at each call site: nodes are credited only in order, and a
//! round completion is once-only per instance id. The score-to-result-string thresholds
//! are NOT recovered and are deliberately absent: only the four string ids are kept.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Inner opcode of the recovered pet packet family (`PetBasePackets`).
pub const PET_BASE_OPCODE: u16 = 53;
/// `PetPerformTrickPacket` - one recovered 32-bit payload field.
pub const SUB_PERFORM_TRICK: u8 = 7;
/// `PetTrickGestureResult` - one recovered 32-bit payload field.
pub const SUB_GESTURE_RESULT: u8 = 8;
/// `PetTrickSkillLevelPacket`.
pub const SUB_SKILL_LEVEL: u8 = 12;
/// `PetTrickAddPacket`.
pub const SUB_TRICK_ADD: u8 = 13;
/// `PetTrickFavoriteSetPacket` - three recovered 32-bit payload fields.
pub const SUB_FAVORITE_SET: u8 = 14;
/// `PetTrickNotifyPacket`.
pub const SUB_NOTIFY: u8 = 15;
/// Player-facing favourite slots, recovered from the `param_2 - 1U < 4` guard.
pub const FAVORITE_SLOT_COUNT: u32 = 4;
/// 32-bit fields per trick in the client profile record.
pub const PROFILE_TRICK_FIELDS: usize = 8;
/// Recovered profile trick record: key + eight fields + flag.
pub const TRICK_RECORD_LEN: usize = 4 + PROFILE_TRICK_FIELDS * 4 + 1;

const REQUIRED_KEYS: [&str; 16] = [
    "DRAW_DIST",
    "SAMPLE_DRAW_RATE",
    "NUM_SEGMENTS",
    "SAMPLE_DRAW_EFFECT",
    "PLAYER_DRAW_EFFECT",
    "SAMPLE_NODE_EFFECT",
    "PLAYER_NODE_EFFECT",
    "HIT_NODE_DIST_SQR",
    "TIMEOUT_SECONDS",
    "RESULT_BAD_STRING",
    "RESULT_OK_STRING",
    "RESULT_GOOD_STRING",
    "RESULT_GREAT_STRING",
    "GET_READY_STRING",
    "START_DRAW_STRING",
    "FORCE_END_MILLISECONDS",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GestureNode {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GestureTrick {
    pub trick_id: u32,
    pub nodes: Vec<GestureNode>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GestureSettings {
    pub draw_dist: f32,
    pub sample_draw_rate: f32,
    pub num_segments: u32,
    pub sample_draw_effect: u32,
    pub player_draw_effect: u32,
    pub sample_node_effect: u32,
    pub player_node_effect: u32,
    pub hit_node_dist_sqr: f32,
    pub timeout_seconds: f32,
    pub force_end_milliseconds: u32,
    pub result_bad_string: u32,
    pub result_ok_string: u32,
    pub result_good_string: u32,
    pub result_great_string: u32,
    pub get_ready_string: u32,
    pub start_draw_string: u32,
}

impl GestureSettings {
    /// The recovered file stores the squared radius; a drawn sample must land within
    /// this distance of the pending waypoint.
    pub fn hit_radius(&self) -> f32 {
        self.hit_node_dist_sqr.sqrt()
    }

    pub fn timeout_milliseconds(&self) -> u32 {
        (self.timeout_seconds * 1000.0).round() as u32
    }

    pub fn max_samples(&self) -> usize {
        self.num_segments as usize
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsError {
    MissingKey(&'static str),
    BadNumber { key: String },
    MalformedRow { line: usize },
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey(key) => write!(f, "missing settings key {key}"),
            Self::BadNumber { key } => write!(f, "unparseable settings value for {key}"),
            Self::MalformedRow { line } => write!(f, "malformed settings row at line {line}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GestureError {
    Empty,
    MalformedRow {
        line: usize,
    },
    BadTrickId {
        trick_id: u32,
    },
    NonContiguous {
        trick_id: u32,
        expected: u32,
        got: u32,
    },
    WaypointOutOfRange {
        trick_id: u32,
        sequence: u32,
    },
}

impl fmt::Display for GestureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "gesture file has no waypoints"),
            Self::MalformedRow { line } => write!(f, "malformed gesture row at line {line}"),
            Self::BadTrickId { trick_id } => write!(f, "invalid trick id {trick_id}"),
            Self::NonContiguous {
                trick_id,
                expected,
                got,
            } => {
                write!(
                    f,
                    "trick {trick_id} expected sequence {expected}, got {got}"
                )
            }
            Self::WaypointOutOfRange { trick_id, sequence } => {
                write!(
                    f,
                    "trick {trick_id} sequence {sequence} leaves the unit plane"
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireError {
    LengthMismatch { expected: usize, got: usize },
    InvalidFavoriteSlot(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundError {
    UnknownTrick(u32),
    Incomplete {
        hit_nodes: usize,
        total_nodes: usize,
    },
    TimedOut {
        elapsed_milliseconds: u32,
    },
    PastForceEnd {
        limit_milliseconds: u32,
    },
    TooManySamples {
        limit: usize,
    },
    TimeWentBackwards,
    ReplayedInstance(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundOutcome {
    pub hit_nodes: usize,
    pub total_nodes: usize,
    pub elapsed_milliseconds: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrickRecord {
    pub trick_id: u32,
    pub fields: [u32; PROFILE_TRICK_FIELDS],
    pub flag: bool,
}

impl TrickRecord {
    /// Field 2 of the recovered profile sample holds the trick's locale name id
    /// (trick 14 Handstand, 15 Play Dead, 16 Peek-a-boo, 17 Speak). This mapping is
    /// inferred from that sample, not proven by the serializer.
    pub fn name_id_candidate(&self) -> Option<u32> {
        match self.fields[2] {
            0 => None,
            value => Some(value),
        }
    }

    /// Field 7 tracks the per-pet skill level in the recovered sample (1/4/5 seen).
    pub fn skill_level_candidate(&self) -> u32 {
        self.fields[7]
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(TRICK_RECORD_LEN);
        out.extend_from_slice(&self.trick_id.to_le_bytes());
        for value in self.fields {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.push(u8::from(self.flag));
        out
    }
}

/// Parse the shipped `#CONSTANT^VALUE` settings file.
pub fn parse_settings(text: &str) -> Result<GestureSettings, SettingsError> {
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('^').collect();
        if fields.len() < 2 || fields[0].is_empty() {
            return Err(SettingsError::MalformedRow { line: index + 1 });
        }
        values.insert(fields[0].to_string(), fields[1].to_string());
    }

    for key in REQUIRED_KEYS {
        if !values.contains_key(key) {
            return Err(SettingsError::MissingKey(key));
        }
    }

    let float = |key: &'static str| -> Result<f32, SettingsError> {
        let raw = values.get(key).ok_or(SettingsError::MissingKey(key))?;
        raw.parse::<f32>().map_err(|_| SettingsError::BadNumber {
            key: key.to_string(),
        })
    };
    let unsigned = |key: &'static str| -> Result<u32, SettingsError> {
        let raw = values.get(key).ok_or(SettingsError::MissingKey(key))?;
        raw.parse::<u32>().map_err(|_| SettingsError::BadNumber {
            key: key.to_string(),
        })
    };

    Ok(GestureSettings {
        draw_dist: float("DRAW_DIST")?,
        sample_draw_rate: float("SAMPLE_DRAW_RATE")?,
        num_segments: unsigned("NUM_SEGMENTS")?,
        sample_draw_effect: unsigned("SAMPLE_DRAW_EFFECT")?,
        player_draw_effect: unsigned("PLAYER_DRAW_EFFECT")?,
        sample_node_effect: unsigned("SAMPLE_NODE_EFFECT")?,
        player_node_effect: unsigned("PLAYER_NODE_EFFECT")?,
        hit_node_dist_sqr: float("HIT_NODE_DIST_SQR")?,
        timeout_seconds: float("TIMEOUT_SECONDS")?,
        force_end_milliseconds: unsigned("FORCE_END_MILLISECONDS")?,
        result_bad_string: unsigned("RESULT_BAD_STRING")?,
        result_ok_string: unsigned("RESULT_OK_STRING")?,
        result_good_string: unsigned("RESULT_GOOD_STRING")?,
        result_great_string: unsigned("RESULT_GREAT_STRING")?,
        get_ready_string: unsigned("GET_READY_STRING")?,
        start_draw_string: unsigned("START_DRAW_STRING")?,
    })
}

/// Parse `#PET_TRICK_ID^SEQUENCE_ID^X^Y^` into per-trick waypoint paths.
pub fn parse_gesture_points(text: &str) -> Result<Vec<GestureTrick>, GestureError> {
    let mut order: Vec<u32> = Vec::new();
    let mut tricks: BTreeMap<u32, GestureTrick> = BTreeMap::new();

    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('^').collect();
        if fields.len() < 4 {
            return Err(GestureError::MalformedRow { line: index + 1 });
        }
        let malformed = || GestureError::MalformedRow { line: index + 1 };
        let trick_id: u32 = fields[0].trim().parse().map_err(|_| malformed())?;
        let sequence: u32 = fields[1].trim().parse().map_err(|_| malformed())?;
        let point = GestureNode {
            x: fields[2].trim().parse().map_err(|_| malformed())?,
            y: fields[3].trim().parse().map_err(|_| malformed())?,
        };

        if trick_id == 0 {
            return Err(GestureError::BadTrickId { trick_id });
        }
        if !(0.0..=1.0).contains(&point.x) || !(0.0..=1.0).contains(&point.y) {
            return Err(GestureError::WaypointOutOfRange { trick_id, sequence });
        }

        if !tricks.contains_key(&trick_id) {
            order.push(trick_id);
            tricks.insert(
                trick_id,
                GestureTrick {
                    trick_id,
                    nodes: Vec::new(),
                },
            );
        }
        let trick = tricks.get_mut(&trick_id).expect("inserted above");
        // Waypoints may repeat: the shipped trick 103 returns to an earlier node, so
        // revisiting a point is real data rather than a malformed row.
        if sequence != trick.nodes.len() as u32 {
            return Err(GestureError::NonContiguous {
                trick_id,
                expected: trick.nodes.len() as u32,
                got: sequence,
            });
        }
        trick.nodes.push(point);
    }

    if order.is_empty() {
        return Err(GestureError::Empty);
    }
    Ok(order
        .into_iter()
        .map(|id| tricks.remove(&id).expect("trick present"))
        .collect())
}

/// One live gesture round. Samples are credited in order against the pending waypoint.
#[derive(Clone, Debug, PartialEq)]
pub struct TrickRound {
    instance_id: u64,
    trick_id: u32,
    nodes: Vec<GestureNode>,
    started_milliseconds: i64,
    last_milliseconds: i64,
    samples: usize,
    hit_nodes: usize,
}

impl TrickRound {
    pub fn instance_id(&self) -> u64 {
        self.instance_id
    }

    pub fn trick_id(&self) -> u32 {
        self.trick_id
    }

    pub fn hit_nodes(&self) -> usize {
        self.hit_nodes
    }

    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn samples(&self) -> usize {
        self.samples
    }

    /// Record one drawn sample and return how many waypoints have been credited.
    pub fn record_sample(
        &mut self,
        x: f32,
        y: f32,
        at_milliseconds: i64,
        settings: GestureSettings,
    ) -> Result<usize, RoundError> {
        if at_milliseconds < self.last_milliseconds {
            return Err(RoundError::TimeWentBackwards);
        }
        let elapsed = at_milliseconds - self.started_milliseconds;
        if elapsed > i64::from(settings.force_end_milliseconds) {
            return Err(RoundError::PastForceEnd {
                limit_milliseconds: settings.force_end_milliseconds,
            });
        }
        if self.samples >= settings.max_samples() {
            return Err(RoundError::TooManySamples {
                limit: settings.max_samples(),
            });
        }
        self.samples += 1;
        self.last_milliseconds = at_milliseconds;

        if let Some(pending) = self.nodes.get(self.hit_nodes) {
            let dx = x - pending.x;
            let dy = y - pending.y;
            if dx * dx + dy * dy <= settings.hit_node_dist_sqr {
                self.hit_nodes += 1;
            }
        }
        Ok(self.hit_nodes)
    }

    /// Accept the round only when every waypoint was credited inside the recovered
    /// timeout. Grade thresholds are not recovered, so the caller receives counts and
    /// timings rather than a guessed result string.
    pub fn finish(
        &self,
        now_milliseconds: i64,
        settings: GestureSettings,
    ) -> Result<RoundOutcome, RoundError> {
        let elapsed = (now_milliseconds - self.started_milliseconds).max(0) as u32;
        if elapsed > settings.timeout_milliseconds() {
            return Err(RoundError::TimedOut {
                elapsed_milliseconds: elapsed,
            });
        }
        if self.hit_nodes < self.nodes.len() {
            return Err(RoundError::Incomplete {
                hit_nodes: self.hit_nodes,
                total_nodes: self.nodes.len(),
            });
        }
        Ok(RoundOutcome {
            hit_nodes: self.hit_nodes,
            total_nodes: self.nodes.len(),
            elapsed_milliseconds: elapsed,
        })
    }
}

/// Reconstructed once-only completion gate over the recovered gesture library.
pub struct TrickRoundLedger {
    tricks: BTreeMap<u32, Vec<GestureNode>>,
    completed: BTreeSet<u64>,
}

impl TrickRoundLedger {
    pub fn new(tricks: &[GestureTrick]) -> Self {
        Self {
            tricks: tricks
                .iter()
                .map(|trick| (trick.trick_id, trick.nodes.clone()))
                .collect(),
            completed: BTreeSet::new(),
        }
    }

    pub fn begin(
        &self,
        instance_id: u64,
        trick_id: u32,
        started_milliseconds: i64,
        _settings: GestureSettings,
    ) -> Result<TrickRound, RoundError> {
        if self.completed.contains(&instance_id) {
            return Err(RoundError::ReplayedInstance(instance_id));
        }
        let nodes = self
            .tricks
            .get(&trick_id)
            .ok_or(RoundError::UnknownTrick(trick_id))?;
        Ok(TrickRound {
            instance_id,
            trick_id,
            nodes: nodes.clone(),
            started_milliseconds,
            last_milliseconds: started_milliseconds,
            samples: 0,
            hit_nodes: 0,
        })
    }

    /// Validate and retire one round. A repeated instance id is refused, so a replayed
    /// completion cannot grant a second result.
    pub fn complete(
        &mut self,
        round: &TrickRound,
        now_milliseconds: i64,
        settings: GestureSettings,
    ) -> Result<RoundOutcome, RoundError> {
        if self.completed.contains(&round.instance_id) {
            return Err(RoundError::ReplayedInstance(round.instance_id));
        }
        let outcome = round.finish(now_milliseconds, settings)?;
        self.completed.insert(round.instance_id);
        Ok(outcome)
    }
}

/// `PetPerformTrickPacket` body: the trick id as one little-endian 32-bit value.
pub fn encode_perform_trick(trick_id: u32) -> [u8; 4] {
    trick_id.to_le_bytes()
}

pub fn decode_perform_trick(body: &[u8]) -> Result<u32, WireError> {
    decode_u32(body)
}

/// `PetTrickGestureResult` body: one little-endian 32-bit value.
pub fn encode_gesture_result(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

pub fn decode_gesture_result(body: &[u8]) -> Result<u32, WireError> {
    decode_u32(body)
}

/// `PetTrickFavoriteSetPacket` body: pet, zero-based favourite slot and trick.
pub fn encode_favorite_set(pet_id: u32, slot: u32, trick_id: u32) -> Result<[u8; 12], WireError> {
    if slot == 0 || slot > FAVORITE_SLOT_COUNT {
        return Err(WireError::InvalidFavoriteSlot(slot));
    }
    let mut out = [0u8; 12];
    out[..4].copy_from_slice(&pet_id.to_le_bytes());
    out[4..8].copy_from_slice(&(slot - 1).to_le_bytes());
    out[8..].copy_from_slice(&trick_id.to_le_bytes());
    Ok(out)
}

pub fn decode_favorite_set(body: &[u8]) -> Result<(u32, u32, u32), WireError> {
    if body.len() != 12 {
        return Err(WireError::LengthMismatch {
            expected: 12,
            got: body.len(),
        });
    }
    Ok((
        u32::from_le_bytes(body[..4].try_into().expect("length checked")),
        u32::from_le_bytes(body[4..8].try_into().expect("length checked")),
        u32::from_le_bytes(body[8..].try_into().expect("length checked")),
    ))
}

/// Decode one pet trick record from the client profile payload.
pub fn decode_trick_record(body: &[u8]) -> Result<TrickRecord, WireError> {
    if body.len() != TRICK_RECORD_LEN {
        return Err(WireError::LengthMismatch {
            expected: TRICK_RECORD_LEN,
            got: body.len(),
        });
    }
    let mut fields = [0u32; PROFILE_TRICK_FIELDS];
    for (index, slot) in fields.iter_mut().enumerate() {
        let start = 4 + index * 4;
        *slot = u32::from_le_bytes(body[start..start + 4].try_into().expect("length checked"));
    }
    Ok(TrickRecord {
        trick_id: u32::from_le_bytes(body[..4].try_into().expect("length checked")),
        fields,
        flag: body[TRICK_RECORD_LEN - 1] != 0,
    })
}

fn decode_u32(body: &[u8]) -> Result<u32, WireError> {
    if body.len() != 4 {
        return Err(WireError::LengthMismatch {
            expected: 4,
            got: body.len(),
        });
    }
    Ok(u32::from_le_bytes(body.try_into().expect("length checked")))
}
