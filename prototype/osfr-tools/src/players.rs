//! Live player sessions and deterministic area-of-interest message fanout.
//!
//! The 35/1 AddPc dispatch and field sequence use native client bytes plus a
//! pinned peer-schema crosswalk. The 35/3 remove and standalone 125 movement
//! layouts follow mapped client readers. Interest limits and fallback AddPc
//! presentation values are reconstructed policy (R).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const MAX_ADD_PC_MESSAGE_BYTES: usize = 64 * 1024; // (R) local snapshot/output bound.
const ADD_PC_FIXED_INNER_BYTES: usize = 195;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SessionId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ZoneInstance {
    pub zone_id: u32,
    pub instance_id: u64,
}

/// Public AddPc presentation built from authenticated character and account
/// state. `attachments_wire` is the existing inventory helper's count plus
/// encoded attachment rows; the registry validates it before storing it.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerPresentation {
    /// (R) Binds this snapshot to the authenticated character at registry join.
    pub character_guid: u64,
    pub name_ids: [i32; 3],
    pub first_name: String,
    pub last_name: String,
    pub model: i32,
    pub head: String,
    pub hair: String,
    pub hair_color: i32,
    pub eye_color: i32,
    pub skin_tone: String,
    pub face_paint: String,
    pub model_customization: String,
    pub max_movement_speed: f32,
    pub underage: bool,
    pub member: bool,
    pub referee: bool,
    pub active_profile: i32,
    pub attachments_wire: Vec<u8>,
    pub wield_type: i32,
}

impl Default for PlayerPresentation {
    fn default() -> Self {
        Self {
            character_guid: 0,
            name_ids: [0; 3], // (R) literal-name mode until identity metadata is supplied.
            first_name: String::new(),
            last_name: String::new(),
            model: 0, // (R) compatibility value when no presentation was supplied.
            head: String::new(),
            hair: String::new(),
            hair_color: 0, // (R) compatibility value when no presentation was supplied.
            eye_color: 0,  // (R) compatibility value when no presentation was supplied.
            skin_tone: String::new(),
            face_paint: String::new(),
            model_customization: String::new(),
            max_movement_speed: 10.0, // (R) matches the gateway's reconstructed base speed.
            underage: false,          // (R) display default only.
            member: false,            // (R) display default only.
            referee: false,           // (R) display default; never grants a server role.
            active_profile: 0, // (R) compatibility profile until gateway supplies the active one.
            attachments_wire: vec![0; 4], // (R) no active-profile attachments.
            wield_type: 0,     // (R) no equipped-weapon value supplied.
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Reconstructed area-of-interest policy (R): inclusive 3D radius and a
/// per-viewer cap ordered by distance, then session id.
pub struct InterestPolicy {
    radius: f32,
    max_visible_count: usize,
}

impl InterestPolicy {
    pub fn new(radius: f32, max_visible_count: usize) -> Result<Self, RegistryError> {
        if !radius.is_finite() || radius < 0.0 {
            return Err(RegistryError::InvalidPolicy);
        }
        Ok(Self {
            radius,
            max_visible_count,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerSession {
    pub session_id: SessionId,
    pub account_guid: u64,
    pub character_guid: u64,
    pub zone: ZoneInstance,
    pub position: [f32; 3],
    pub heading: f32,
    pub presentation: PlayerPresentation,
    pub movement_state: u8,
    pub movement_unknown: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageKind {
    Spawn,
    Position,
    Despawn,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboundMessage {
    pub recipient: SessionId,
    pub kind: MessageKind,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryError {
    InvalidPolicy,
    InvalidPose,
    InvalidPresentation,
    InvalidGuid,
    SessionAlreadyActive,
    GuidAlreadyBound,
    UnknownSession,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy => f.write_str("invalid player interest policy"),
            Self::InvalidPose => f.write_str("player pose contains a non-finite value"),
            Self::InvalidPresentation => f.write_str("invalid player presentation snapshot"),
            Self::InvalidGuid => f.write_str("account and character GUIDs must be nonzero"),
            Self::SessionAlreadyActive => f.write_str("session is already active"),
            Self::GuidAlreadyBound => f.write_str("GUID is already bound to another session"),
            Self::UnknownSession => f.write_str("session is not active"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[derive(Clone, Debug)]
pub struct PlayerRegistry {
    policy: InterestPolicy,
    sessions: BTreeMap<SessionId, PlayerSession>,
    // (R) At most one live session may own either its account or character GUID.
    guid_owners: BTreeMap<u64, SessionId>,
    // (R) Uniform 3D broad phase; exact distance and directed caps are unchanged.
    cells: BTreeMap<(ZoneInstance, [i64; 3]), BTreeSet<SessionId>>,
}

impl PlayerRegistry {
    pub fn new(policy: InterestPolicy) -> Self {
        Self {
            policy,
            sessions: BTreeMap::new(),
            guid_owners: BTreeMap::new(),
            cells: BTreeMap::new(),
        }
    }

    pub fn get(&self, session_id: SessionId) -> Option<&PlayerSession> {
        self.sessions.get(&session_id)
    }

    pub fn join(
        &mut self,
        session_id: SessionId,
        account_guid: u64,
        character_guid: u64,
        zone: ZoneInstance,
        position: [f32; 3],
        heading: f32,
    ) -> Result<Vec<OutboundMessage>, RegistryError> {
        let mut presentation = PlayerPresentation::default();
        presentation.character_guid = character_guid;
        self.join_with_presentation(
            session_id,
            account_guid,
            character_guid,
            zone,
            position,
            heading,
            presentation,
        )
    }

    /// Join with the authenticated snapshot used to build the subject's 35/1
    /// AddPc. The legacy `join` wrapper remains for callers awaiting gateway
    /// wiring and supplies only the explicit reconstructed fallback (R).
    pub fn join_with_presentation(
        &mut self,
        session_id: SessionId,
        account_guid: u64,
        character_guid: u64,
        zone: ZoneInstance,
        position: [f32; 3],
        heading: f32,
        presentation: PlayerPresentation,
    ) -> Result<Vec<OutboundMessage>, RegistryError> {
        validate_identity(account_guid, character_guid)?;
        validate_pose(position, heading)?;
        presentation.validate(character_guid)?;
        if self.sessions.contains_key(&session_id) {
            return Err(RegistryError::SessionAlreadyActive);
        }
        for guid in [account_guid, character_guid] {
            if self
                .guid_owners
                .get(&guid)
                .is_some_and(|owner| *owner != session_id)
            {
                return Err(RegistryError::GuidAlreadyBound);
            }
        }

        let mut affected = self.nearby(zone, position);
        affected.insert(session_id);
        let before_views = self.visible_sets(&affected);
        self.insert_cell(session_id, zone, position);
        self.sessions.insert(
            session_id,
            PlayerSession {
                session_id,
                account_guid,
                character_guid,
                zone,
                position,
                heading,
                presentation,
                movement_state: 0,   // (R) no accepted movement report yet.
                movement_unknown: 0, // (R) no accepted movement report yet.
            },
        );
        self.guid_owners.insert(account_guid, session_id);
        self.guid_owners.insert(character_guid, session_id);
        let after_views = self.visible_sets(&affected);
        Ok(self.diff_messages(None, &before_views, &after_views, None))
    }

    pub fn leave(&mut self, session_id: SessionId) -> Vec<OutboundMessage> {
        // (R) Repeated disconnect notifications are idempotent.
        let Some(removed) = self.sessions.get(&session_id).cloned() else {
            return Vec::new();
        };
        let affected = self.nearby(removed.zone, removed.position);
        let before_views = self.visible_sets(&affected);
        self.remove_cell(session_id, removed.zone, removed.position);
        self.sessions.remove(&session_id);
        for guid in [removed.account_guid, removed.character_guid] {
            if self.guid_owners.get(&guid) == Some(&session_id) {
                self.guid_owners.remove(&guid);
            }
        }
        let after_views = self.visible_sets(&affected);
        self.diff_messages(Some(&removed), &before_views, &after_views, None)
    }

    /// Change pose and/or zone while preserving accepted relay bytes. Interest
    /// radius and visible-count rules are reconstructed policy (R).
    pub fn update_pose(
        &mut self,
        session_id: SessionId,
        zone: ZoneInstance,
        position: [f32; 3],
        heading: f32,
    ) -> Result<Vec<OutboundMessage>, RegistryError> {
        let (state, unknown) = self
            .sessions
            .get(&session_id)
            .map(|player| (player.movement_state, player.movement_unknown))
            .unwrap_or((0, 0));
        self.update_pose_with_state(session_id, zone, position, heading, state, unknown)
    }

    /// Store bytes accepted for this session after GUID and pose validation.
    /// They are opaque protocol state; every u8 value is preserved as-is (R).
    pub fn update_pose_with_state(
        &mut self,
        session_id: SessionId,
        zone: ZoneInstance,
        position: [f32; 3],
        heading: f32,
        state: u8,
        unknown: u8,
    ) -> Result<Vec<OutboundMessage>, RegistryError> {
        validate_pose(position, heading)?;
        let Some(current) = self.sessions.get(&session_id) else {
            return Err(RegistryError::UnknownSession);
        };
        if current.zone == zone
            && current.position == position
            && current.heading == heading
            && current.movement_state == state
            && current.movement_unknown == unknown
        {
            return Ok(Vec::new());
        }

        let mut affected = self.nearby(current.zone, current.position);
        affected.extend(self.nearby(zone, position));
        affected.insert(session_id);
        let before_views = self.visible_sets(&affected);
        let old_cell = (current.zone, self.cell(current.position));
        let new_cell = (zone, self.cell(position));
        if old_cell != new_cell {
            let (old_zone, old_position) = (current.zone, current.position);
            self.remove_cell(session_id, old_zone, old_position);
            self.insert_cell(session_id, zone, position);
        }
        let changed = self.sessions.get_mut(&session_id).unwrap();
        changed.zone = zone;
        changed.position = position;
        changed.heading = heading;
        changed.movement_state = state;
        changed.movement_unknown = unknown;
        let after_views = self.visible_sets(&affected);
        Ok(self.diff_messages(None, &before_views, &after_views, Some(session_id)))
    }

    fn cell(&self, position: [f32; 3]) -> [i64; 3] {
        // Zero-radius policy only sees coincident positions. Unit cells avoid
        // division by zero. Saturating casts keep every finite pose valid.
        let width = if self.policy.radius == 0.0 {
            1.0
        } else {
            f64::from(self.policy.radius)
        };
        position.map(|value| (f64::from(value) / width).floor() as i64)
    }

    fn insert_cell(&mut self, id: SessionId, zone: ZoneInstance, position: [f32; 3]) {
        self.cells
            .entry((zone, self.cell(position)))
            .or_default()
            .insert(id);
    }

    fn remove_cell(&mut self, id: SessionId, zone: ZoneInstance, position: [f32; 3]) {
        let key = (zone, self.cell(position));
        if let Some(occupants) = self.cells.get_mut(&key) {
            occupants.remove(&id);
            if occupants.is_empty() {
                self.cells.remove(&key);
            }
        }
    }

    fn nearby(&self, zone: ZoneInstance, position: [f32; 3]) -> BTreeSet<SessionId> {
        let cell = self.cell(position);
        let mut nearby = BTreeSet::new();
        // One guard cell beyond the usual adjacent cells preserves the old
        // inclusive f64 radius when subtraction rounds at a cell boundary:
        // e.g. -f32::from_bits(1) and radius=100 are distance 100 in f64.
        // checked_add handles saturated coordinates without overflow/panics.
        for dx in -2..=2 {
            for dy in -2..=2 {
                for dz in -2..=2 {
                    let (Some(x), Some(y), Some(z)) = (
                        cell[0].checked_add(dx),
                        cell[1].checked_add(dy),
                        cell[2].checked_add(dz),
                    ) else {
                        continue;
                    };
                    if let Some(occupants) = self.cells.get(&(zone, [x, y, z])) {
                        nearby.extend(occupants);
                    }
                }
            }
        }
        // Only viewers within the old/new radius can change their directed
        // selection. Broad-phase false positives need no visibility rebuild.
        let radius_squared = f64::from(self.policy.radius).powi(2);
        nearby
            .retain(|id| distance_squared(position, self.sessions[id].position) <= radius_squared);
        nearby
    }

    fn visible_sets(
        &self,
        affected: &BTreeSet<SessionId>,
    ) -> BTreeMap<SessionId, BTreeSet<SessionId>> {
        let mut visible = BTreeMap::new();
        let radius_squared = f64::from(self.policy.radius).powi(2);
        // (R) Visibility is directed under a per-viewer cap. Distance ties use
        // session id, and the radius boundary is included.
        // ponytail: dense crowds still scan/sort local candidates per affected
        // viewer; incremental ranked views are a separate measured upgrade.
        for viewer in affected.iter().filter_map(|id| self.sessions.get(id)) {
            let mut candidates: Vec<(f64, SessionId)> = self
                .nearby(viewer.zone, viewer.position)
                .into_iter()
                .filter(|id| *id != viewer.session_id)
                .filter_map(|id| {
                    let candidate = &self.sessions[&id];
                    let distance_squared = distance_squared(viewer.position, candidate.position);
                    (distance_squared <= radius_squared).then_some((distance_squared, id))
                })
                .collect();
            candidates.sort_by(|left, right| {
                left.0
                    .total_cmp(&right.0)
                    .then_with(|| left.1.cmp(&right.1))
            });
            let set = candidates
                .into_iter()
                .take(self.policy.max_visible_count)
                .map(|(_, session)| session)
                .collect();
            visible.insert(viewer.session_id, set);
        }
        visible
    }

    fn diff_messages(
        &self,
        removed: Option<&PlayerSession>,
        before_views: &BTreeMap<SessionId, BTreeSet<SessionId>>,
        after_views: &BTreeMap<SessionId, BTreeSet<SessionId>>,
        updated_session: Option<SessionId>,
    ) -> Vec<OutboundMessage> {
        let mut messages = Vec::new();
        for (viewer, current_view) in after_views {
            let old_view = before_views.get(viewer).cloned().unwrap_or_default();
            for subject in old_view.difference(current_view) {
                if let Some(player) = self
                    .sessions
                    .get(subject)
                    .or_else(|| removed.filter(|player| player.session_id == *subject))
                {
                    messages.push(OutboundMessage {
                        recipient: *viewer,
                        kind: MessageKind::Despawn,
                        bytes: build_remove_player(player.character_guid),
                    });
                }
            }
            for subject in current_view.difference(&old_view) {
                if let Some(player) = self.sessions.get(subject) {
                    messages.push(OutboundMessage {
                        recipient: *viewer,
                        kind: MessageKind::Spawn,
                        bytes: build_add_pc(player),
                    });
                }
            }
            if let Some(updated) = updated_session {
                if current_view.contains(&updated) && old_view.contains(&updated) {
                    if let Some(player) = self.sessions.get(&updated) {
                        messages.push(OutboundMessage {
                            recipient: *viewer,
                            kind: MessageKind::Position,
                            bytes: build_position_update(player),
                        });
                    }
                }
            }
        }
        messages
    }
}

fn validate_identity(account_guid: u64, character_guid: u64) -> Result<(), RegistryError> {
    // (R) Zero is reserved as an invalid account or character identity.
    if account_guid == 0 || character_guid == 0 {
        return Err(RegistryError::InvalidGuid);
    }
    Ok(())
}

fn validate_pose(position: [f32; 3], heading: f32) -> Result<(), RegistryError> {
    if position.into_iter().any(|value| !value.is_finite()) || !heading.is_finite() {
        return Err(RegistryError::InvalidPose);
    }
    Ok(())
}

impl PlayerPresentation {
    fn validate(&self, character_guid: u64) -> Result<(), RegistryError> {
        if self.character_guid != character_guid
            || !self.max_movement_speed.is_finite()
            || self.max_movement_speed < 0.0
        {
            return Err(RegistryError::InvalidPresentation);
        }

        let mut message_bytes = ADD_PC_FIXED_INNER_BYTES + 7;
        for value in [
            &self.first_name,
            &self.last_name,
            &self.head,
            &self.hair,
            &self.skin_tone,
            &self.face_paint,
            &self.model_customization,
        ] {
            i32::try_from(value.len()).map_err(|_| RegistryError::InvalidPresentation)?;
            message_bytes = message_bytes
                .checked_add(value.len())
                .ok_or(RegistryError::InvalidPresentation)?;
        }

        let attachment_rows_bytes = validate_attachment_wire(&self.attachments_wire)?;
        message_bytes = message_bytes
            .checked_add(attachment_rows_bytes)
            .ok_or(RegistryError::InvalidPresentation)?;
        if message_bytes > MAX_ADD_PC_MESSAGE_BYTES {
            return Err(RegistryError::InvalidPresentation);
        }
        Ok(())
    }
}

fn validate_attachment_wire(bytes: &[u8]) -> Result<usize, RegistryError> {
    if bytes.len() > MAX_ADD_PC_MESSAGE_BYTES {
        return Err(RegistryError::InvalidPresentation);
    }
    let mut offset = 0;
    let count = usize::try_from(read_attachment_i32(bytes, &mut offset)?)
        .map_err(|_| RegistryError::InvalidPresentation)?;
    let available_rows = bytes.len().saturating_sub(4) / 24;
    if count > available_rows {
        return Err(RegistryError::InvalidPresentation);
    }

    for _ in 0..count {
        for _ in 0..3 {
            let length = usize::try_from(read_attachment_i32(bytes, &mut offset)?)
                .map_err(|_| RegistryError::InvalidPresentation)?;
            let start = offset;
            let end = offset
                .checked_add(length)
                .filter(|end| *end <= bytes.len())
                .ok_or(RegistryError::InvalidPresentation)?;
            std::str::from_utf8(&bytes[start..end])
                .map_err(|_| RegistryError::InvalidPresentation)?;
            offset = end;
        }
        offset = offset
            .checked_add(12)
            .filter(|end| *end <= bytes.len())
            .ok_or(RegistryError::InvalidPresentation)?;
    }
    if offset != bytes.len() {
        return Err(RegistryError::InvalidPresentation);
    }
    Ok(bytes.len() - 4)
}

fn read_attachment_i32(bytes: &[u8], offset: &mut usize) -> Result<i32, RegistryError> {
    let end = offset
        .checked_add(4)
        .filter(|end| *end <= bytes.len())
        .ok_or(RegistryError::InvalidPresentation)?;
    let value = i32::from_le_bytes(
        bytes[*offset..end]
            .try_into()
            .map_err(|_| RegistryError::InvalidPresentation)?,
    );
    *offset = end;
    Ok(value)
}

fn distance_squared(left: [f32; 3], right: [f32; 3]) -> f64 {
    let dx = f64::from(left[0]) - f64::from(right[0]);
    let dy = f64::from(left[1]) - f64::from(right[1]);
    let dz = f64::from(left[2]) - f64::from(right[2]);
    dx * dx + dy * dy + dz * dz
}

fn build_add_pc(player: &PlayerSession) -> Vec<u8> {
    let mut inner = Vec::with_capacity(195);
    put_u16(&mut inner, 35);
    put_u16(&mut inner, 1);
    put_u64(&mut inner, player.character_guid);

    let presentation = &player.presentation;
    for value in presentation.name_ids {
        put_i32(&mut inner, value);
    }
    put_string(&mut inner, &presentation.first_name);
    put_string(&mut inner, &presentation.last_name);
    put_i32(&mut inner, presentation.model);
    for _ in 0..3 {
        put_i32(&mut inner, 0); // chat bubble fields
    }
    for value in [
        player.position[0],
        player.position[1],
        player.position[2],
        1.0,
    ] {
        put_f32(&mut inner, value);
    }
    // Native 0092ff2c-0092ff44 derives yaw as atan2(X, Z); heading is radians (R).
    for value in [player.heading.sin(), 0.0, player.heading.cos(), 0.0] {
        put_f32(&mut inner, value);
    }
    inner.extend_from_slice(&presentation.attachments_wire);
    put_string(&mut inner, &presentation.head);
    put_string(&mut inner, &presentation.hair);
    put_i32(&mut inner, presentation.hair_color);
    put_i32(&mut inner, presentation.eye_color);
    put_i32(&mut inner, 0); // (R) Unknown3 has no authoritative source.
    put_string(&mut inner, &presentation.skin_tone);
    put_string(&mut inner, &presentation.face_paint);
    put_string(&mut inner, &presentation.model_customization);
    put_f32(&mut inner, presentation.max_movement_speed);
    inner.extend_from_slice(&[
        u8::from(presentation.underage),
        u8::from(presentation.member),
        u8::from(presentation.referee),
    ]);
    put_i32(&mut inner, 0); // (R) no temporary model override.
    put_i32(&mut inner, 0); // (R) guild runtime is not routed.
    put_i32(&mut inner, presentation.active_profile);
    for _ in 0..4 {
        put_i32(&mut inner, 0); // (R) title fields are not routed.
    }
    put_i32(&mut inner, 0); // (R) no public persistent-effect snapshot.
    put_u64(&mut inner, 0); // (R) mounts are not routed.
    put_i32(&mut inner, -1); // (R) unmounted seat sentinel.
    put_i32(&mut inner, -1); // (R) unmounted queue sentinel.
    put_f32(&mut inner, 0.0); // (R) no mount/nameplate lift.
    put_i32(&mut inner, presentation.wield_type);
    put_f32(&mut inner, 0.0); // (R) no VIP progression source.
    for _ in 0..3 {
        put_i32(&mut inner, 0); // (R) VIP icon/title and Unknown17.
    }
    tunnel(&inner)
}

fn build_position_update(player: &PlayerSession) -> Vec<u8> {
    let mut inner = Vec::with_capacity(36);
    put_u16(&mut inner, 125);
    put_u64(&mut inner, player.character_guid);
    for value in player.position {
        put_f32(&mut inner, value);
    }
    // Native 0095dae0 derives yaw as atan2(X, Z); heading is radians (R).
    for value in [player.heading.sin(), 0.0, player.heading.cos()] {
        put_f32(&mut inner, value);
    }
    inner.extend_from_slice(&[player.movement_state, player.movement_unknown]);
    tunnel(&inner)
}

fn build_remove_player(character_guid: u64) -> Vec<u8> {
    let mut inner = Vec::with_capacity(14);
    put_u16(&mut inner, 35);
    put_u16(&mut inner, 3);
    put_u16(&mut inner, 0); // (R) immediate removal submode
    put_u64(&mut inner, character_guid);
    tunnel(&inner)
}

fn tunnel(inner: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(7 + inner.len());
    out.extend_from_slice(&[5, 0, 1]);
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    put_i32(
        out,
        i32::try_from(value.len()).expect("string length exceeds i32"),
    );
    out.extend_from_slice(value.as_bytes());
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_bits().to_le_bytes());
}
