//! Live player sessions and deterministic area-of-interest message fanout.
//!
//! The 35/1 AddPc dispatch and field sequence use native client bytes plus a
//! pinned peer-schema crosswalk. The 35/3 remove and standalone 125 movement
//! layouts follow mapped client readers. Interest limits and minimal AddPc
//! appearance values are reconstructed policy (R).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SessionId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ZoneInstance {
    pub zone_id: u32,
    pub instance_id: u64,
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
}

impl PlayerRegistry {
    pub fn new(policy: InterestPolicy) -> Self {
        Self {
            policy,
            sessions: BTreeMap::new(),
            guid_owners: BTreeMap::new(),
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
        validate_identity(account_guid, character_guid)?;
        validate_pose(position, heading)?;
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

        let before_sessions = self.sessions.clone();
        let before_views = self.visible_sets();
        self.sessions.insert(
            session_id,
            PlayerSession {
                session_id,
                account_guid,
                character_guid,
                zone,
                position,
                heading,
            },
        );
        self.guid_owners.insert(account_guid, session_id);
        self.guid_owners.insert(character_guid, session_id);
        let after_views = self.visible_sets();
        Ok(self.diff_messages(&before_sessions, &before_views, &after_views, None))
    }

    pub fn leave(&mut self, session_id: SessionId) -> Vec<OutboundMessage> {
        // (R) Repeated disconnect notifications are idempotent.
        let Some(removed) = self.sessions.get(&session_id).cloned() else {
            return Vec::new();
        };
        let before_sessions = self.sessions.clone();
        let before_views = self.visible_sets();
        self.sessions.remove(&session_id);
        for guid in [removed.account_guid, removed.character_guid] {
            if self.guid_owners.get(&guid) == Some(&session_id) {
                self.guid_owners.remove(&guid);
            }
        }
        let after_views = self.visible_sets();
        self.diff_messages(&before_sessions, &before_views, &after_views, None)
    }

    /// Change pose and/or zone. Interest radius and visible-count rules are
    /// reconstructed policy (R); zone equality includes the instance id.
    pub fn update_pose(
        &mut self,
        session_id: SessionId,
        zone: ZoneInstance,
        position: [f32; 3],
        heading: f32,
    ) -> Result<Vec<OutboundMessage>, RegistryError> {
        validate_pose(position, heading)?;
        let Some(current) = self.sessions.get(&session_id) else {
            return Err(RegistryError::UnknownSession);
        };
        if current.zone == zone && current.position == position && current.heading == heading {
            return Ok(Vec::new());
        }

        let before_sessions = self.sessions.clone();
        let before_views = self.visible_sets();
        let changed = self.sessions.get_mut(&session_id).unwrap();
        changed.zone = zone;
        changed.position = position;
        changed.heading = heading;
        let after_views = self.visible_sets();
        Ok(self.diff_messages(
            &before_sessions,
            &before_views,
            &after_views,
            Some(session_id),
        ))
    }

    fn visible_sets(&self) -> BTreeMap<SessionId, BTreeSet<SessionId>> {
        let mut visible = BTreeMap::new();
        let radius_squared = f64::from(self.policy.radius).powi(2);
        // (R) Visibility is directed under a per-viewer cap. Distance ties use
        // session id, and the radius boundary is included.
        for viewer in self.sessions.values() {
            let mut candidates: Vec<(f64, SessionId)> = self
                .sessions
                .values()
                .filter(|candidate| {
                    candidate.session_id != viewer.session_id && candidate.zone == viewer.zone
                })
                .filter_map(|candidate| {
                    let distance_squared = distance_squared(viewer.position, candidate.position);
                    (distance_squared <= radius_squared)
                        .then_some((distance_squared, candidate.session_id))
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
        before_sessions: &BTreeMap<SessionId, PlayerSession>,
        before_views: &BTreeMap<SessionId, BTreeSet<SessionId>>,
        after_views: &BTreeMap<SessionId, BTreeSet<SessionId>>,
        moved_session: Option<SessionId>,
    ) -> Vec<OutboundMessage> {
        let mut messages = Vec::new();
        for (viewer, current_view) in after_views {
            let old_view = before_views.get(viewer).cloned().unwrap_or_default();
            for subject in old_view.difference(current_view) {
                if let Some(player) = before_sessions.get(subject) {
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
            if let Some(moved) = moved_session {
                if current_view.contains(&moved) && old_view.contains(&moved) {
                    if let Some(player) = self.sessions.get(&moved) {
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

    // (R) Presentation, membership, guild, title and mount data are not in the
    // registry inputs. Emit empty/zero defaults until account data is wired in.
    for _ in 0..3 {
        put_i32(&mut inner, 0);
    }
    put_string(&mut inner, "");
    put_string(&mut inner, "");
    put_i32(&mut inner, 0); // model
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
    // (R) AddPc carries a quaternion. The recovered movement packet carries a
    // horizontal direction vector; this yaw quaternion is the spawn crosswalk.
    let half_heading = player.heading * 0.5;
    for value in [0.0, half_heading.sin(), 0.0, half_heading.cos()] {
        put_f32(&mut inner, value);
    }
    put_i32(&mut inner, 0); // attachment count
    put_string(&mut inner, ""); // head
    put_string(&mut inner, ""); // hair
    for _ in 0..3 {
        put_i32(&mut inner, 0); // hair color, eye color, unknown field
    }
    for _ in 0..3 {
        put_string(&mut inner, ""); // skin tone, face paint, customization
    }
    put_f32(&mut inner, 0.0); // (R) movement speed; profile data not yet supplied
    inner.extend_from_slice(&[0, 0, 0]); // (R) underage, member, referee
    put_i32(&mut inner, 0); // temporary appearance
    put_i32(&mut inner, 0); // guild count
    put_i32(&mut inner, 0); // active profile
    for _ in 0..4 {
        put_i32(&mut inner, 0); // title fields
    }
    put_i32(&mut inner, 0); // effect-tag count
    put_u64(&mut inner, 0); // mount GUID
    put_i32(&mut inner, 0); // mount seat
    put_i32(&mut inner, 0); // mount queue position
    put_f32(&mut inner, 0.0); // name vertical offset
    put_i32(&mut inner, 0); // wield type
    put_f32(&mut inner, 0.0); // VIP rank
    for _ in 0..3 {
        put_i32(&mut inner, 0); // VIP icon, VIP title, unknown field
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
    inner.extend_from_slice(&[0, 0]); // (R) state and unknown bytes
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
