//! Kart pickup set: native `33/15` create, `33/16` destroy and `33/18` grant, plus the bounded
//! positional authority a caller runs *after* its own claim validation.
//!
//! ORIGINAL facts (nothing here is named or invented):
//!
//! * `33/15` `VehicleRacePacketCreateProxiedVehicleRacePickup` (reader `FUN_00ba1380` ->
//!   `FUN_00ba0070`) carries `u32 handle | u32 model | 4 x f32 (x, y, z, w) | f32 | u32`: 32 bytes.
//!   The client spawns only when the handle is unknown (`FUN_00b9e130`), so repeats are idempotent.
//! * `33/16` `VehicleRacePacketDestroyProxiedVehicleRacePickup` carries one `u32 handle`.
//! * `33/18` `VehicleRacePacketPickupHitResponse` carries one `u32` item type routed into the same
//!   client queue as the boost award (`FUN_00fe9410`).
//! * the `33/17` claim names the *claiming player* (the same `racer+0xe60 | racer+0xe64` pair as the
//!   gate hit) plus the vehicle position; it carries no handle, so attribution is positional.
//!
//! Capture constants, kept verbatim and deliberately unlabelled: every create in
//! `lab/reports/original-minigame-packets` uses model `497`, `w = 1.0`, `+0x30 = 1.0` and
//! `+0x34 = 1324`. The community resource table resolves `497` to
//! `sg_icon_random_pickup_anim_bbe.adr`; the last two words have no recovered label and are written
//! back unchanged rather than prettified as physics.
//!
//! RECONSTRUCTED content: the collection radius (the capture's largest claim-to-box distance is
//! 2.525 m, so [`DEFAULT_RADIUS_M`] is 3 m and content may choose up to [`MAX_RADIUS_M`]), the drop
//! type and the optional timed respawn. The typed item foundation accepts exactly the four captured
//! types (1 missile, 3 mine, 4 shield, 5 boost); the client applies each type's own callback. No
//! weapon effect is implemented or claimed here.
//!
//! RESPAWN. [`PickupRespawn`] is opt-in and defaults to absent, so content and saves written before
//! it behave exactly as before: a collected box is permanent. When content opts in, a collection
//! schedules the box's return `delay_ms` later (bounded to 1..=60 s, at most
//! [`MAX_RESPAWNS_PER_PICKUP`] per definition per race) and [`PickupSet::due_respawns`] announces it
//! once the caller's clock passes the due time. The native capture shows 13 destroyed boxes
//! returning 5.033..=8.037 s later **with the same handle**, so the definition handle is reused:
//! inventing a generation protocol would buy nothing, because `33/17` carries no handle at all and
//! staleness is bounded by transport dedup, the timed live state and the caller's owner/movement
//! guards. Nothing here respawns before the caller announces the spawn, after the race leaves
//! `Racing`, past the window or across a clock regression - those edges belong to the service.
//!
//! AUTHORITY. This module never touches economy, gates, phases or saves. It answers one question -
//! which live box did this owned claim position touch - and consumes that box exactly once, after
//! the caller stages its own grant. A refused claim changes nothing, including the pending
//! respawn schedule. `reset` is for a new race only.

use crate::kart_entry;
use serde::{Deserialize, Serialize};
use std::fmt;

/// ORIGINAL: client `VehicleRacePacketCreateProxiedVehicleRacePickup`.
pub const CREATE_SUBTYPE: u16 = 15;
/// ORIGINAL: client `VehicleRacePacketDestroyProxiedVehicleRacePickup`.
pub const DESTROY_SUBTYPE: u16 = 16;
/// ORIGINAL: client `VehicleRacePacketPickupHitRequest` (the claim this set answers).
pub const CLAIM_SUBTYPE: u16 = 17;
/// ORIGINAL: client `VehicleRacePacketPickupHitResponse`.
pub const GRANT_SUBTYPE: u16 = 18;
/// ORIGINAL class mapping: `FUN_00b99b40` builds `VehicleRacePacketItemExpired` with subtype
/// `0x15`; the client's dispatcher case `0x15` reads one u32 and toggles the matching effect off.
pub const EXPIRE_SUBTYPE: u16 = 21;
/// ORIGINAL raw PE: case `0x15` at 0xba88c1/0xba88c6 subtracts 2 twice, then
/// calls `FUN_00c57610(0)` at 0xba88cc for selector 4. Selector 1 is ignored;
/// the pointer-typed decompiler comparison obscures the second subtraction.
pub const SHIELD_EXPIRE_VALUE: u32 = 4;
/// ORIGINAL: the only box model in the captured race, `sg_icon_random_pickup_anim_bbe.adr`.
pub const PICKUP_MODEL: u32 = 497;
/// ORIGINAL: the recovered item types. `FUN_00ba74d0` case `0x14` applies type 5 as the boost
/// (`vehicle+0x118 = DAT_017ec67c`, `vehicle+0x64 = 1`) and the captured `33/18` grants carry
/// 1, 3, 4 and 5; the community icon table names the family attack/boost/health/sheild/ultimate,
/// and the lead's chronology audit maps 1 missile, 3 mine, 4 shield, 5 boost. The client applies
/// each type's own callback - no weapon effect is implemented or claimed on this side.
pub const MISSILE_ITEM_TYPE: u32 = 1;
pub const MINE_ITEM_TYPE: u32 = 3;
pub const SHIELD_ITEM_TYPE: u32 = 4;
pub const BOOST_ITEM_TYPE: u32 = 5;

/// True for the four item types the captures prove; anything else is refused as unsupported.
pub fn is_recovered_item_type(item_type: u32) -> bool {
    matches!(
        item_type,
        MISSILE_ITEM_TYPE | MINE_ITEM_TYPE | SHIELD_ITEM_TYPE | BOOST_ITEM_TYPE
    )
}

/// RECONSTRUCTED opt-in timed respawn. Absent content keeps the legacy permanent consumption.
/// The native capture shows the same handle recreated 5.033..=8.037 s after its destroy, so the
/// definition handle is reused instead of inventing a generation protocol, and a `33/17` claim
/// carries no handle at all: staleness is bounded by transport dedup, the timed live state and the
/// owner/movement guards, never by handle freshness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PickupRespawn {
    pub delay_ms: u64,
    pub max_respawns: u32,
}

/// RECONSTRUCTED floor: a shorter delay would let one collection return almost immediately.
pub const MIN_RESPAWN_DELAY_MS: u64 = 1_000;
/// RECONSTRUCTED ceiling: longer than a captured minute-scale respawn is content error.
pub const MAX_RESPAWN_DELAY_MS: u64 = 60_000;
/// RECONSTRUCTED bound: respawns per definition per race, so a race cannot flood the track.
pub const MAX_RESPAWNS_PER_PICKUP: u32 = 16;
/// ORIGINAL capture constant: the fourth create float.
pub const CREATE_W: f32 = 1.0;
/// ORIGINAL capture constant (`+0x30`, NaN-checked by the reader): label unrecovered.
pub const CREATE_SCALAR_0X30: f32 = 1.0;
/// ORIGINAL capture constant (`+0x34`): label unrecovered.
pub const CREATE_TAIL_0X34: u32 = 1324;
/// ORIGINAL: `33/15` payload width.
pub const CREATE_PAYLOAD_LEN: usize = 32;
/// ORIGINAL: `33/16` payload width.
pub const DESTROY_PAYLOAD_LEN: usize = 4;
/// ORIGINAL: `33/18` payload width.
pub const GRANT_PAYLOAD_LEN: usize = 4;
/// RECONSTRUCTED bound: content definitions per race.
pub const MAX_PICKUPS: usize = 64;
/// RECONSTRUCTED content: collection radius when a definition does not choose one.
pub const DEFAULT_RADIUS_M: f32 = 3.0;
/// RECONSTRUCTED bound: a radius beyond this would swallow a track section.
pub const MAX_RADIUS_M: f32 = 6.0;

/// One authored pickup definition. `radius` and `item_type` are reconstructed content; `handle`,
/// `model` and `position` are the fields the original `33/15` frame carries.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PickupSpec {
    pub handle: u32,
    pub model: u32,
    pub position: [f32; 3],
    pub radius: f32,
    pub item_type: u32,
}

impl PickupSpec {
    /// The captured retail box: model 497, the reconstructed default radius, boost drop.
    pub fn native(handle: u32, position: [f32; 3]) -> Self {
        Self {
            handle,
            model: PICKUP_MODEL,
            position,
            radius: DEFAULT_RADIUS_M,
            item_type: BOOST_ITEM_TYPE,
        }
    }
}

/// Typed definition/snapshot failure: nothing was created or consumed.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PickupError {
    TooMany {
        count: usize,
        max: usize,
    },
    ZeroHandle,
    DuplicateHandle {
        handle: u32,
    },
    NonFinitePosition {
        handle: u32,
    },
    BadRadius {
        handle: u32,
        radius: f32,
    },
    UnsupportedItemType {
        handle: u32,
        item_type: u32,
    },
    UnknownHandle {
        handle: u32,
    },
    DuplicateConsumed {
        handle: u32,
    },
    /// A respawn policy outside the reconstructed bounds.
    BadRespawnPolicy {
        field: &'static str,
    },
    /// A saved respawn entry is inconsistent with the definitions, consumed marks or window.
    IncoherentRespawn {
        handle: u32,
    },
}

impl fmt::Display for PickupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PickupError::TooMany { count, max } => {
                write!(f, "content needs 0..={max} pickups, got {count}")
            }
            PickupError::ZeroHandle => write!(f, "pickup handle must be non-zero"),
            PickupError::DuplicateHandle { handle } => {
                write!(f, "pickup handle {handle} is defined twice")
            }
            PickupError::NonFinitePosition { handle } => {
                write!(f, "pickup {handle} position is not finite")
            }
            PickupError::BadRadius { handle, radius } => write!(
                f,
                "pickup {handle} radius {radius} must be positive, finite and <= {MAX_RADIUS_M}"
            ),
            PickupError::UnsupportedItemType { handle, item_type } => write!(
                f,
                "pickup {handle} item type {item_type} is not one of the recovered types 1, 3, 4, 5"
            ),
            PickupError::UnknownHandle { handle } => {
                write!(f, "pickup handle {handle} is not part of this set")
            }
            PickupError::DuplicateConsumed { handle } => {
                write!(
                    f,
                    "pickup handle {handle} is consumed twice in the snapshot"
                )
            }
            PickupError::BadRespawnPolicy { field } => {
                write!(f, "pickup respawn policy {field} is outside its bounds")
            }
            PickupError::IncoherentRespawn { handle } => {
                write!(f, "pickup respawn entry {handle} is incoherent")
            }
        }
    }
}

impl std::error::Error for PickupError {}

/// Typed refusal for one claim; nothing moved.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PickupRefusal {
    ForeignGuid { offered: u64, owned: u64 },
    NonFinitePosition,
    NoPickupInRange,
    UnknownHandle { handle: u32 },
    AlreadyConsumed { handle: u32 },
}

impl fmt::Display for PickupRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PickupRefusal::ForeignGuid { offered, owned } => {
                write!(f, "claim guid {offered:016x} is not the owned {owned:016x}")
            }
            PickupRefusal::NonFinitePosition => write!(f, "claim position is not finite"),
            PickupRefusal::NoPickupInRange => write!(f, "no live pickup is inside the radius"),
            PickupRefusal::UnknownHandle { handle } => {
                write!(f, "pickup handle {handle} is not part of this set")
            }
            PickupRefusal::AlreadyConsumed { handle } => {
                write!(f, "pickup {handle} was already consumed")
            }
        }
    }
}

/// A claim that staged cleanly, handed back as an opaque token: the caller may read it, run its own
/// grant, then [`PickupSet::commit`]. The fields are private because `commit` resolves the granted
/// type from the authoritative definition - a token can never name a type the content did not
/// define.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StagedClaim {
    handle: u32,
    item_type: u32,
}

impl StagedClaim {
    /// The definition handle this claim staged.
    pub fn handle(&self) -> u32 {
        self.handle
    }

    /// The type the matching definition declares (the grant `commit` will actually emit).
    pub fn item_type(&self) -> u32 {
        self.item_type
    }
}

/// A committed claim: exactly one `33/18` grant followed by the matching `33/16` destroy.
#[derive(Clone, Debug, PartialEq)]
pub struct ClaimedPickup {
    pub handle: u32,
    pub item_type: u32,
    pub replies: Vec<Vec<u8>>,
}

/// One scheduled return: the collected definition handle and the server clock it returns at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingRespawn {
    pub handle: u32,
    pub due_ms: u64,
}

/// Persisted state. Every field defaults, so a save written before pickups or respawns existed
/// restores as "nothing collected, nothing pending" and never conjures a box.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PickupSnapshot {
    /// Collected definition handles that are still off the track, sorted and unique.
    #[serde(default)]
    pub consumed: Vec<u32>,
    /// One entry per respawn already fired this race (bounded by the policy).
    #[serde(default)]
    pub respawned: Vec<u32>,
    /// Returns that are scheduled but not yet due, sorted by `(due_ms, handle)`.
    #[serde(default)]
    pub pending: Vec<PendingRespawn>,
}

/// Exact `33/15` payload (32 bytes) for one definition.
pub fn encode_create_payload(spec: &PickupSpec) -> Vec<u8> {
    let mut payload = spec.handle.to_le_bytes().to_vec();
    payload.extend(spec.model.to_le_bytes());
    payload.extend(spec.position[0].to_le_bytes());
    payload.extend(spec.position[1].to_le_bytes());
    payload.extend(spec.position[2].to_le_bytes());
    payload.extend(CREATE_W.to_le_bytes());
    payload.extend(CREATE_SCALAR_0X30.to_le_bytes());
    payload.extend(CREATE_TAIL_0X34.to_le_bytes());
    debug_assert_eq!(payload.len(), CREATE_PAYLOAD_LEN);
    payload
}

/// Inner `33/15` frame for one definition.
pub fn encode_create(spec: &PickupSpec) -> Vec<u8> {
    kart_entry::encode_inner(CREATE_SUBTYPE, &encode_create_payload(spec))
}

/// Inner `33/16` frame for one handle.
pub fn encode_destroy(handle: u32) -> Vec<u8> {
    kart_entry::encode_inner(DESTROY_SUBTYPE, &handle.to_le_bytes())
}

/// Inner `33/18` frame for one item type.
pub fn encode_grant(item_type: u32) -> Vec<u8> {
    kart_entry::encode_inner(GRANT_SUBTYPE, &item_type.to_le_bytes())
}

/// Inner `33/21` `VehicleRacePacketItemExpired` frame for one effect value.
pub fn encode_expire(value: u32) -> Vec<u8> {
    kart_entry::encode_inner(EXPIRE_SUBTYPE, &value.to_le_bytes())
}

fn finite(position: [f32; 3]) -> bool {
    position.iter().all(|value| value.is_finite())
}

/// A bounded set of authored pickups plus the running race's collection and respawn state.
#[derive(Clone, Debug, PartialEq)]
pub struct PickupSet {
    /// Sorted by handle so nearest-box ties always resolve to the lowest handle.
    specs: Vec<PickupSpec>,
    consumed: Vec<u32>,
    /// Opt-in timed respawn; `None` keeps every collected box permanent.
    respawn: Option<PickupRespawn>,
    /// One entry per respawn already fired this race.
    respawned: Vec<u32>,
    /// Returns that are scheduled but not yet due, sorted by `(due_ms, handle)`.
    pending: Vec<PendingRespawn>,
}

impl PickupSet {
    /// Validates and sorts the definitions with no respawn: the pre-respawn behavior.
    pub fn new(specs: Vec<PickupSpec>) -> Result<Self, PickupError> {
        Self::with_respawn(specs, None)
    }

    /// Validates and sorts the definitions plus the optional respawn policy. Handles are unique and
    /// non-zero, positions finite, radii positive and bounded, and only the four recovered item
    /// types are accepted. `None` keeps every collected box permanent.
    pub fn with_respawn(
        mut specs: Vec<PickupSpec>,
        respawn: Option<PickupRespawn>,
    ) -> Result<Self, PickupError> {
        if specs.len() > MAX_PICKUPS {
            return Err(PickupError::TooMany {
                count: specs.len(),
                max: MAX_PICKUPS,
            });
        }
        specs.sort_by_key(|spec| spec.handle);
        for (index, spec) in specs.iter().enumerate() {
            if spec.handle == 0 {
                return Err(PickupError::ZeroHandle);
            }
            if specs[..index]
                .iter()
                .any(|other| other.handle == spec.handle)
            {
                return Err(PickupError::DuplicateHandle {
                    handle: spec.handle,
                });
            }
            if !finite(spec.position) {
                return Err(PickupError::NonFinitePosition {
                    handle: spec.handle,
                });
            }
            if !spec.radius.is_finite() || spec.radius <= 0.0 || spec.radius > MAX_RADIUS_M {
                return Err(PickupError::BadRadius {
                    handle: spec.handle,
                    radius: spec.radius,
                });
            }
            if !is_recovered_item_type(spec.item_type) {
                return Err(PickupError::UnsupportedItemType {
                    handle: spec.handle,
                    item_type: spec.item_type,
                });
            }
        }
        if let Some(policy) = respawn {
            if policy.delay_ms < MIN_RESPAWN_DELAY_MS || policy.delay_ms > MAX_RESPAWN_DELAY_MS {
                return Err(PickupError::BadRespawnPolicy { field: "delay_ms" });
            }
            if policy.max_respawns == 0 || policy.max_respawns > MAX_RESPAWNS_PER_PICKUP {
                return Err(PickupError::BadRespawnPolicy {
                    field: "max_respawns",
                });
            }
        }
        Ok(Self {
            specs,
            consumed: Vec::new(),
            respawn,
            respawned: Vec::new(),
            pending: Vec::new(),
        })
    }

    /// The opt-in respawn policy, or `None` for permanent consumption.
    pub fn respawn(&self) -> Option<PickupRespawn> {
        self.respawn
    }

    /// The read-only definitions, in handle order.
    pub fn specs(&self) -> &[PickupSpec] {
        &self.specs
    }

    /// Every `33/15` spawn for a still-live definition, in handle order. A fresh set returns every
    /// definition; a restored set omits the boxes the snapshot already consumed, so a late
    /// re-announce cannot resurrect them.
    pub fn create_frames(&self) -> Vec<Vec<u8>> {
        self.live_create_frames()
    }

    /// Explicit name for [`PickupSet::create_frames`]: live definitions only.
    pub fn live_create_frames(&self) -> Vec<Vec<u8>> {
        self.specs
            .iter()
            .filter(|spec| !self.consumed.contains(&spec.handle))
            .map(encode_create)
            .collect()
    }

    pub fn is_live(&self, handle: u32) -> bool {
        self.specs.iter().any(|spec| spec.handle == handle) && !self.consumed.contains(&handle)
    }

    pub fn live_count(&self) -> usize {
        self.specs.len() - self.consumed.len()
    }

    /// True while a definition exists and its box is still on the track.
    pub fn is_consumed(&self, handle: u32) -> bool {
        self.consumed.contains(&handle)
    }

    /// Fresh race: the definitions stay and every collection, respawn count and pending return
    /// clears. Call only on a new race.
    pub fn reset(&mut self) {
        self.consumed.clear();
        self.respawned.clear();
        self.pending.clear();
    }

    /// Nearest live box for an owned claim. Read-only, so the caller can run its own grant first
    /// and only then [`PickupSet::commit`]; a refusal changes nothing.
    pub fn stage_claim(
        &self,
        claimed_guid: u64,
        owned_guid: u64,
        position: [f32; 3],
    ) -> Result<StagedClaim, PickupRefusal> {
        if claimed_guid != owned_guid {
            return Err(PickupRefusal::ForeignGuid {
                offered: claimed_guid,
                owned: owned_guid,
            });
        }
        if !finite(position) {
            return Err(PickupRefusal::NonFinitePosition);
        }
        let mut nearest: Option<(f32, &PickupSpec)> = None;
        for spec in &self.specs {
            if self.consumed.contains(&spec.handle) {
                continue;
            }
            let dx = position[0] - spec.position[0];
            let dy = position[1] - spec.position[1];
            let dz = position[2] - spec.position[2];
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            if distance > spec.radius {
                continue;
            }
            // Strict `<` keeps the first (lowest) handle on a tie, so equidistant boxes are stable.
            if nearest.is_none_or(|(best, _)| distance < best) {
                nearest = Some((distance, spec));
            }
        }
        nearest
            .map(|(_, spec)| StagedClaim {
                handle: spec.handle,
                item_type: spec.item_type,
            })
            .ok_or(PickupRefusal::NoPickupInRange)
    }

    /// Consumes the staged box once and returns the original reply order: `33/18` grant, then
    /// `33/16` destroy. The granted type is resolved from the matching definition, never from the
    /// token, so a forged token cannot emit an unported type. A second commit of the same claim is
    /// refused.
    ///
    /// When a respawn policy is set and the definition still has budget, the collection schedules
    /// its return at `now_ms + delay_ms` **only when that due time still falls inside the caller's
    /// race window** (`race_end_ms`); a collection too late in the race stays permanent instead of
    /// parking a return that could never fire. The definition handle is reused exactly as the
    /// captured native server does (13 destroyed boxes returned 5.033..=8.037 s later with the same
    /// handle).
    pub fn commit(
        &mut self,
        staged: StagedClaim,
        now_ms: u64,
        race_end_ms: u64,
    ) -> Result<Vec<Vec<u8>>, PickupRefusal> {
        let Some(spec) = self.specs.iter().find(|spec| spec.handle == staged.handle) else {
            return Err(PickupRefusal::UnknownHandle {
                handle: staged.handle,
            });
        };
        if self.consumed.contains(&staged.handle) {
            return Err(PickupRefusal::AlreadyConsumed {
                handle: staged.handle,
            });
        }
        let item_type = spec.item_type;
        self.consumed.push(staged.handle);
        if let Some(policy) = self.respawn {
            if self.respawns_used(staged.handle) < policy.max_respawns {
                let due_ms = now_ms.saturating_add(policy.delay_ms);
                if due_ms <= race_end_ms {
                    self.pending.push(PendingRespawn {
                        handle: staged.handle,
                        due_ms,
                    });
                    self.pending
                        .sort_by_key(|pending| (pending.due_ms, pending.handle));
                }
            }
        }
        Ok(vec![encode_grant(item_type), encode_destroy(staged.handle)])
    }

    /// Returns every pending box whose delay has elapsed: `33/15` frames in due order, each with
    /// the definition's position and its original handle. The caller decides whether the race is
    /// live (in phase, inside the window), so this is a pure state transition - nothing else moves.
    pub fn due_respawns(&mut self, now_ms: u64) -> Vec<Vec<u8>> {
        let mut replies = Vec::new();
        let mut index = 0;
        while index < self.pending.len() {
            if self.pending[index].due_ms > now_ms {
                index += 1;
                continue;
            }
            let pending = self.pending.remove(index);
            self.respawned.push(pending.handle);
            self.consumed.retain(|handle| *handle != pending.handle);
            if let Some(spec) = self.specs.iter().find(|spec| spec.handle == pending.handle) {
                replies.push(encode_create(spec));
            }
        }
        replies
    }

    /// How many times this definition has already returned this race.
    fn respawns_used(&self, handle: u32) -> u32 {
        self.respawned
            .iter()
            .filter(|respawned| **respawned == handle)
            .count() as u32
    }

    /// Stage and commit in one call for callers that have nothing to persist between the two.
    pub fn claim(
        &mut self,
        claimed_guid: u64,
        owned_guid: u64,
        position: [f32; 3],
        now_ms: u64,
        race_end_ms: u64,
    ) -> Result<ClaimedPickup, PickupRefusal> {
        let staged = self.stage_claim(claimed_guid, owned_guid, position)?;
        let replies = self.commit(staged, now_ms, race_end_ms)?;
        Ok(ClaimedPickup {
            handle: staged.handle,
            item_type: staged.item_type,
            replies,
        })
    }

    /// Sorted, unique state: consumed handles, fired respawns and pending returns.
    pub fn snapshot(&self) -> PickupSnapshot {
        let mut consumed = self.consumed.clone();
        consumed.sort_unstable();
        let mut respawned = self.respawned.clone();
        respawned.sort_unstable();
        let mut pending = self.pending.clone();
        pending.sort_by_key(|pending| (pending.due_ms, pending.handle));
        PickupSnapshot {
            consumed,
            respawned,
            pending,
        }
    }

    /// Restores collection and respawn state against the same definitions and policy.
    ///
    /// A save written before respawns (all fields default) restores as "nothing collected, nothing
    /// pending". Duplicated or unknown handles, a pending return that is not collected, more than
    /// one return per definition at a time, a fired-plus-pending count beyond the policy, and a
    /// `due_ms` outside `race_started_ms..=race_end_ms` are all refused rather than guessed.
    pub fn restore(
        specs: Vec<PickupSpec>,
        respawn: Option<PickupRespawn>,
        snapshot: PickupSnapshot,
        race_started_ms: u64,
        race_end_ms: u64,
    ) -> Result<Self, PickupError> {
        let mut set = Self::with_respawn(specs, respawn)?;
        for (index, handle) in snapshot.consumed.iter().enumerate() {
            if snapshot.consumed[..index].contains(handle) {
                return Err(PickupError::DuplicateConsumed { handle: *handle });
            }
            if !set.specs.iter().any(|spec| spec.handle == *handle) {
                return Err(PickupError::UnknownHandle { handle: *handle });
            }
        }
        let Some(policy) = respawn else {
            // Without a policy nothing can ever have fired or been scheduled.
            if let Some(handle) = snapshot
                .respawned
                .first()
                .copied()
                .or_else(|| snapshot.pending.first().map(|pending| pending.handle))
            {
                return Err(PickupError::IncoherentRespawn { handle });
            }
            let mut consumed = snapshot.consumed;
            consumed.sort_unstable();
            set.consumed = consumed;
            return Ok(set);
        };
        for handle in snapshot
            .respawned
            .iter()
            .copied()
            .chain(snapshot.pending.iter().map(|pending| pending.handle))
        {
            if !set.specs.iter().any(|spec| spec.handle == handle) {
                return Err(PickupError::UnknownHandle { handle });
            }
            let used = snapshot
                .respawned
                .iter()
                .filter(|respawned| **respawned == handle)
                .count() as u32
                + snapshot
                    .pending
                    .iter()
                    .filter(|pending| pending.handle == handle)
                    .count() as u32;
            if used > policy.max_respawns {
                return Err(PickupError::IncoherentRespawn { handle });
            }
        }
        for (index, pending) in snapshot.pending.iter().enumerate() {
            if !snapshot.consumed.contains(&pending.handle) {
                // A scheduled return exists only while its box is off the track.
                return Err(PickupError::IncoherentRespawn {
                    handle: pending.handle,
                });
            }
            if snapshot.pending[..index]
                .iter()
                .any(|other| other.handle == pending.handle)
            {
                // At most one live return per definition.
                return Err(PickupError::IncoherentRespawn {
                    handle: pending.handle,
                });
            }
            if pending.due_ms < race_started_ms || pending.due_ms > race_end_ms {
                return Err(PickupError::IncoherentRespawn {
                    handle: pending.handle,
                });
            }
        }
        let mut consumed = snapshot.consumed;
        consumed.sort_unstable();
        let mut respawned = snapshot.respawned;
        respawned.sort_unstable();
        let mut pending = snapshot.pending;
        pending.sort_by_key(|pending| (pending.due_ms, pending.handle));
        set.consumed = consumed;
        set.respawned = respawned;
        set.pending = pending;
        Ok(set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A token is opaque outside this module; forged inside it, the granted type must still come
    /// from the definition, never from the token.
    #[test]
    fn forged_staged_token_cannot_emit_an_unsupported_grant() {
        let spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
        let mut set = PickupSet::new(vec![spec]).unwrap();
        let forged = StagedClaim {
            handle: 1,
            item_type: 1,
        };
        let replies = set.commit(forged, 0, 0).unwrap();
        assert_eq!(
            replies
                .iter()
                .map(|frame| u16::from_le_bytes([frame[2], frame[3]]))
                .collect::<Vec<_>>(),
            vec![GRANT_SUBTYPE, DESTROY_SUBTYPE]
        );
        assert_eq!(
            u32::from_le_bytes(replies[0][4..8].try_into().unwrap()),
            BOOST_ITEM_TYPE,
            "the grant resolves the definition type, not the token"
        );

        let mut fresh = PickupSet::new(vec![spec]).unwrap();
        assert_eq!(
            fresh
                .commit(
                    StagedClaim {
                        handle: 9,
                        item_type: BOOST_ITEM_TYPE,
                    },
                    0,
                    0,
                )
                .unwrap_err(),
            PickupRefusal::UnknownHandle { handle: 9 }
        );
    }
}
