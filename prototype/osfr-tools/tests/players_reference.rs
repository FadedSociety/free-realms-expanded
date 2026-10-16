//! W96 frozen pre-grid algorithm, for valid-presentation differential/load tests.
//! Original project players.rs, 2026-10-16. No upstream/Legacy source copied.
//! Presentation validation remains covered by tests/players.rs; wire builders
//! and all-session clone/visibility/diff logic below are the pre-change code.
#![allow(dead_code)]
use osfr_tools::players::{
    MessageKind, OutboundMessage, PlayerPresentation, PlayerSession, RegistryError, SessionId,
    ZoneInstance,
};
use std::collections::{BTreeMap, BTreeSet};
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
                presentation,
                movement_state: 0,   // (R) no accepted movement report yet.
                movement_unknown: 0, // (R) no accepted movement report yet.
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

        let before_sessions = self.sessions.clone();
        let before_views = self.visible_sets();
        let changed = self.sessions.get_mut(&session_id).unwrap();
        changed.zone = zone;
        changed.position = position;
        changed.heading = heading;
        changed.movement_state = state;
        changed.movement_unknown = unknown;
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
        updated_session: Option<SessionId>,
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

impl PlayerRegistry {
    // Benchmark setup only; excludes cubic sequential-join setup from timings.
    pub fn seeded(policy: InterestPolicy, players: impl Iterator<Item = PlayerSession>) -> Self {
        let mut registry = Self::new(policy);
        for player in players {
            registry
                .guid_owners
                .insert(player.account_guid, player.session_id);
            registry
                .guid_owners
                .insert(player.character_guid, player.session_id);
            registry.sessions.insert(player.session_id, player);
        }
        registry
    }
}
