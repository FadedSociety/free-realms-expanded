//! Reconstructed transport-free queue and match coordination.
//!
//! Queue selection, readiness timeout, requeue, countdown, group preservation,
//! and match formation are server policy (R): the client does not disclose
//! those decisions. Wire codecs below are limited to layouts recovered from
//! the client readers/writers cited in `notes/Research/Matchmaking Service.md`.

use crate::minigame::{MinigameRuns, RunSpec};
use crate::players::SessionId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

const FAMILY_MATCHMAKING: u16 = 141;
const FAMILY_GROUP_LAUNCH: u16 = 175;
const MAX_WIRE_ROWS: usize = 4096;
const QUEUE_ROW_BYTES: usize = 19 * 4 + 2;
const OUTER_HEADER_BYTES: usize = 7;

/// Injected monotonic clock. Values must not move backwards (R).
pub trait Clock {
    fn now_ms(&self) -> u64;
}

/// Injected source for opaque queue request ids.
pub trait RandomSource {
    fn next_u64(&mut self) -> u64;
}

/// Queue policy/configuration. Queue ids and display fields are supplied by
/// the caller from the native 141/2 catalogue. Timing and formation choices
/// are reconstructed policy (R), not recovered retail server behavior.
#[derive(Clone, Debug)]
pub struct QueueDefinition {
    pub queue_id: u32,
    pub min_players: usize,
    pub max_players: usize,
    pub min_teams: usize,
    pub max_teams: usize,
    /// Optional per-team capacities. An empty list treats each joining party
    /// as one team; a nonempty list assigns whole parties FIFO to the first
    /// team with enough room (R).
    pub team_sizes: Vec<usize>,
    pub allow_solo: bool,
    pub allow_groups: bool,
    pub max_wait_ms: u64,
    pub ready_timeout_ms: u64,
    pub countdown_ms: u64,
    pub run_spec: RunSpec,
}

impl QueueDefinition {
    fn validate(&self) -> Result<(), MatchmakingError> {
        let team_capacity = self
            .team_sizes
            .iter()
            .try_fold(0usize, |total, size| total.checked_add(*size))
            .ok_or(MatchmakingError::InvalidQueue)?;
        if self.queue_id == 0
            || self.queue_id > i32::MAX as u32
            || self.min_players == 0
            || self.max_players < self.min_players
            || self.min_teams == 0
            || self.max_teams < self.min_teams
            || self.team_sizes.iter().any(|size| *size == 0)
            || (!self.team_sizes.is_empty()
                && (self.team_sizes.len() < self.min_teams
                    || self.team_sizes.len() > self.max_teams
                    || team_capacity > self.max_players))
        {
            return Err(MatchmakingError::InvalidQueue);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Participant {
    pub session_id: SessionId,
    pub character_guid: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueEntry {
    pub group_id: Option<u32>,
    pub leader: SessionId,
    pub members: Vec<Participant>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoinResult {
    Joined,
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JoinOutcome {
    pub result: JoinResult,
    pub outbound: Vec<SessionOutbound>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionOutbound {
    pub recipient: SessionId,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MatchmakingError {
    InvalidQueue,
    DuplicateQueue,
    UnknownQueue,
    InvalidEntry,
    SessionBusy,
    QueueFullForParty,
    UnknownMatch,
    InvalidPacket,
    PacketTooLarge,
    Lifecycle(String),
}

impl fmt::Display for MatchmakingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidQueue => f.write_str("invalid matchmaking queue definition"),
            Self::DuplicateQueue => f.write_str("duplicate matchmaking queue id"),
            Self::UnknownQueue => f.write_str("unknown matchmaking queue"),
            Self::InvalidEntry => f.write_str("invalid queue entry"),
            Self::SessionBusy => f.write_str("session already belongs to a queue or match"),
            Self::QueueFullForParty => f.write_str("queue capacity is smaller than the party"),
            Self::UnknownMatch => f.write_str("unknown match"),
            Self::InvalidPacket => f.write_str("malformed matchmaking packet"),
            Self::PacketTooLarge => f.write_str("matchmaking packet exceeds bounded size"),
            Self::Lifecycle(reason) => write!(f, "minigame lifecycle rejected match: {reason}"),
        }
    }
}

impl std::error::Error for MatchmakingError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Reservation {
    Waiting { queue_id: u32 },
    Ready { queue_id: u32 },
    Running { queue_id: u32, match_id: i32 },
}

#[derive(Clone, Debug)]
struct Party {
    request_id: u64,
    joined_at_ms: u64,
    entry: QueueEntry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReadyPhase {
    Checking,
    Countdown,
}

#[derive(Clone, Debug)]
struct ReadyCheck {
    launch_id: i32,
    deadline_ms: u64,
    phase: ReadyPhase,
    /// `(participant, team, ready)`; member ordering is formation order.
    members: Vec<(Participant, u32, bool)>,
    run_spec: RunSpec,
}

#[derive(Clone, Debug)]
struct ActiveMatch {
    participants: BTreeMap<SessionId, Participant>,
    lifecycle: MinigameRuns,
}

#[derive(Clone, Debug)]
struct QueueState {
    definition: QueueDefinition,
    waiting: VecDeque<Party>,
    ready: Option<ReadyCheck>,
}

/// Deterministic queue state machine. It owns no sockets or player registry;
/// the adapter resolves social group membership and routes `SessionOutbound`
/// to each session's reliable sender.
pub struct Matchmaking<C, R> {
    clock: C,
    rng: R,
    queues: BTreeMap<u32, QueueState>,
    reservations: BTreeMap<SessionId, Reservation>,
    matches: BTreeMap<i32, ActiveMatch>,
    used_request_ids: BTreeSet<u64>,
    next_instance_id: i32,
    next_launch_id: i32,
    last_now_ms: u64,
}

impl<C: Clock, R: RandomSource> Matchmaking<C, R> {
    pub fn new(
        definitions: impl IntoIterator<Item = QueueDefinition>,
        clock: C,
        rng: R,
        first_instance_id: i32,
    ) -> Result<Self, MatchmakingError> {
        if first_instance_id <= 0 {
            return Err(MatchmakingError::InvalidQueue);
        }
        let mut queues = BTreeMap::new();
        for definition in definitions {
            definition.validate()?;
            let id = definition.queue_id;
            if queues
                .insert(
                    id,
                    QueueState {
                        definition,
                        waiting: VecDeque::new(),
                        ready: None,
                    },
                )
                .is_some()
            {
                return Err(MatchmakingError::DuplicateQueue);
            }
        }
        let now = clock.now_ms();
        Ok(Self {
            clock,
            rng,
            queues,
            reservations: BTreeMap::new(),
            matches: BTreeMap::new(),
            used_request_ids: BTreeSet::new(),
            next_instance_id: first_instance_id,
            next_launch_id: first_instance_id,
            last_now_ms: now,
        })
    }

    pub fn queued_sessions(&self, queue_id: u32) -> Option<usize> {
        let queue = self.queues.get(&queue_id)?;
        Some(
            queue
                .waiting
                .iter()
                .map(|party| party.entry.members.len())
                .sum::<usize>()
                + queue.ready.as_ref().map_or(0, |ready| ready.members.len()),
        )
    }

    pub fn active_match_count(&self) -> usize {
        self.matches.len()
    }

    pub fn match_state(&self, match_id: i32) -> Option<crate::minigame::RunState> {
        self.matches
            .get(&match_id)
            .and_then(|active| active.lifecycle.state(match_id))
    }

    /// Atomically reserves every member of a solo or group entry. A repeated
    /// request with exactly the same current membership is inert (R).
    pub fn join(
        &mut self,
        queue_id: u32,
        entry: QueueEntry,
    ) -> Result<JoinOutcome, MatchmakingError> {
        let queue = self
            .queues
            .get(&queue_id)
            .ok_or(MatchmakingError::UnknownQueue)?;
        validate_entry(&queue.definition, &entry)?;
        let currently_reserved = entry
            .members
            .iter()
            .filter(|member| self.reservations.contains_key(&member.session_id))
            .count();
        if currently_reserved == entry.members.len()
            && entry.members.iter().all(|member| {
                matches!(
                    self.reservations.get(&member.session_id),
                    Some(Reservation::Waiting { queue_id: current })
                        | Some(Reservation::Ready { queue_id: current })
                        | Some(Reservation::Running { queue_id: current, .. })
                        if *current == queue_id
                )
            })
        {
            return Ok(JoinOutcome {
                result: JoinResult::Duplicate,
                outbound: Vec::new(),
            });
        }
        if currently_reserved != 0 {
            return Err(MatchmakingError::SessionBusy);
        }
        let members = entry.members.len();
        let max_players = queue.definition.max_players;
        if members > max_players {
            return Err(MatchmakingError::QueueFullForParty);
        }
        let request_id = self.unique_request_id();
        let now = self.now();
        let queue = self.queues.get_mut(&queue_id).unwrap();
        queue.waiting.push_back(Party {
            request_id,
            joined_at_ms: now,
            entry: entry.clone(),
        });
        for participant in entry.members {
            self.reservations
                .insert(participant.session_id, Reservation::Waiting { queue_id });
        }
        Ok(JoinOutcome {
            result: JoinResult::Joined,
            outbound: Vec::new(),
        })
    }

    /// Removes one member from a waiting party. Remaining party members stay
    /// together; this split-on-leave rule is reconstructed policy (R).
    pub fn leave(
        &mut self,
        session_id: SessionId,
    ) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        self.remove_session(session_id, true)
    }

    /// Disconnect uses the same state transition as leave but never emits to
    /// the disconnected session (R).
    pub fn disconnect(
        &mut self,
        session_id: SessionId,
    ) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        self.remove_session(session_id, false)
    }

    /// Marks a ready-check participant and fans out its 175/9 state. (R) The
    /// formation-order index is used as the member id because matchmaking-to-
    /// launch member-id mapping has not been recovered.
    pub fn set_ready(
        &mut self,
        session_id: SessionId,
        ready: bool,
    ) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        let Some(Reservation::Ready { queue_id }) = self.reservations.get(&session_id).copied()
        else {
            return Ok(Vec::new());
        };
        let now = self.now();
        let (launch_id, member_index, changed, recipients) = {
            let queue = self.queues.get_mut(&queue_id).unwrap();
            let Some(check) = queue.ready.as_mut() else {
                return Ok(Vec::new());
            };
            if check.phase != ReadyPhase::Checking {
                return Ok(Vec::new());
            }
            let Some(member_index) = check
                .members
                .iter()
                .position(|(participant, _, _)| participant.session_id == session_id)
            else {
                return Ok(Vec::new());
            };
            let changed = check.members[member_index].2 != ready;
            check.members[member_index].2 = ready;
            if ready && check.members.iter().all(|(_, _, is_ready)| *is_ready) {
                check.phase = ReadyPhase::Countdown;
                check.deadline_ms = now.saturating_add(queue.definition.countdown_ms);
            }
            (
                check.launch_id,
                member_index,
                changed,
                check
                    .members
                    .iter()
                    .map(|(participant, _, _)| participant.session_id)
                    .collect::<Vec<_>>(),
            )
        };
        if !changed {
            return Ok(Vec::new());
        }
        let member_id =
            u32::try_from(member_index).map_err(|_| MatchmakingError::PacketTooLarge)?;
        let packet = encode_member_state(
            launch_id,
            queue_id as i32,
            member_id,
            if ready { 0x02 } else { 0 },
        );
        Ok(recipients
            .into_iter()
            .map(|recipient| SessionOutbound {
                recipient,
                bytes: packet.clone(),
            })
            .collect())
    }

    /// Advances ready timeouts, queue delays and countdowns using the injected
    /// clock; returned bytes are ordered per match and then per recipient.
    pub fn tick(&mut self) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        let now = self.now();
        let queue_ids: Vec<u32> = self.queues.keys().copied().collect();
        let mut out = Vec::new();

        for queue_id in queue_ids.iter().copied() {
            let timed_out = self.queues[&queue_id].ready.as_ref().is_some_and(|check| {
                check.phase == ReadyPhase::Checking && now >= check.deadline_ms
            });
            if timed_out {
                out.extend(self.timeout_ready_check(queue_id)?);
            }
        }

        for queue_id in queue_ids.iter().copied() {
            let countdown_done = self.queues[&queue_id].ready.as_ref().is_some_and(|check| {
                check.phase == ReadyPhase::Countdown && now >= check.deadline_ms
            });
            if countdown_done {
                out.extend(self.start_match(queue_id)?);
            }
        }

        for queue_id in queue_ids {
            if self.queues[&queue_id].ready.is_some() {
                continue;
            }
            if let Some(candidate) = self.take_formation(queue_id, now) {
                out.extend(self.begin_ready_check(queue_id, candidate, now)?);
            }
        }
        Ok(out)
    }

    /// Releases a completed match after its activity reports terminal state.
    pub fn close_match(&mut self, match_id: i32) -> Result<(), MatchmakingError> {
        let active = self
            .matches
            .remove(&match_id)
            .ok_or(MatchmakingError::UnknownMatch)?;
        for session_id in active.participants.keys() {
            self.reservations.remove(session_id);
        }
        Ok(())
    }

    fn remove_session(
        &mut self,
        session_id: SessionId,
        emit_to_departing: bool,
    ) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        let Some(reservation) = self.reservations.remove(&session_id) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        match reservation {
            Reservation::Waiting { queue_id } => {
                let queue = self.queues.get_mut(&queue_id).unwrap();
                for party in &mut queue.waiting {
                    party
                        .entry
                        .members
                        .retain(|member| member.session_id != session_id);
                }
                queue
                    .waiting
                    .retain(|party| !party.entry.members.is_empty());
            }
            Reservation::Ready { queue_id } => {
                let check = self.queues.get_mut(&queue_id).unwrap().ready.take();
                if let Some(check) = check {
                    out.extend(self.cancel_ready_and_requeue(queue_id, check, Some(session_id))?);
                }
            }
            Reservation::Running { match_id, .. } => {
                let mut emptied = false;
                if let Some(active) = self.matches.get_mut(&match_id) {
                    active.participants.remove(&session_id);
                    emptied = active.participants.is_empty();
                    if emptied {
                        active.lifecycle.disconnect();
                    }
                }
                if emptied {
                    self.matches.remove(&match_id);
                }
            }
        }
        if emit_to_departing {
            // No response body is emitted for a local leave: native 141/5 and
            // 141/18 are server replies, while the request body is only partly
            // recovered. The gateway adapter sends its captured request.
        }
        Ok(out)
    }

    fn timeout_ready_check(
        &mut self,
        queue_id: u32,
    ) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        let check = self
            .queues
            .get_mut(&queue_id)
            .unwrap()
            .ready
            .take()
            .unwrap();
        self.cancel_ready_and_requeue(queue_id, check, None)
    }

    fn cancel_ready_and_requeue(
        &mut self,
        queue_id: u32,
        check: ReadyCheck,
        departing: Option<SessionId>,
    ) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        let mut out = Vec::new();
        let cancel = GroupLaunchPacket::header_only(16, check.launch_id, queue_id as i32)?;
        let now = self.now();
        let mut ready_members = Vec::new();
        for (participant, _team, is_ready) in check.members {
            if Some(participant.session_id) == departing {
                self.reservations.remove(&participant.session_id);
                continue;
            }
            out.push(SessionOutbound {
                recipient: participant.session_id,
                bytes: cancel.clone(),
            });
            if is_ready {
                ready_members.push(participant);
            } else {
                self.reservations.remove(&participant.session_id);
            }
        }
        // Ready timeout requeues only the ready players, as requested; the
        // split into solo entries and front-of-queue priority are (R).
        let request_ids: Vec<u64> = ready_members
            .iter()
            .map(|_| self.unique_request_id())
            .collect();
        let queue = self.queues.get_mut(&queue_id).unwrap();
        for (participant, request_id) in ready_members.into_iter().zip(request_ids).rev() {
            let entry = QueueEntry {
                group_id: None,
                leader: participant.session_id,
                members: vec![participant],
            };
            queue.waiting.push_front(Party {
                request_id,
                joined_at_ms: now,
                entry,
            });
            self.reservations
                .insert(participant.session_id, Reservation::Waiting { queue_id });
        }
        Ok(out)
    }

    fn take_formation(&mut self, queue_id: u32, now: u64) -> Option<Formation> {
        let queue = self.queues.get_mut(&queue_id)?;
        if queue.waiting.is_empty() {
            return None;
        }
        let definition = &queue.definition;
        let mut selected = Vec::new();
        let mut player_count = 0usize;
        let mut team_count = 0usize;
        let mut capacities = definition.team_sizes.clone();
        let mut used_teams = vec![false; capacities.len()];
        let mut oldest = u64::MAX;

        for party in &queue.waiting {
            let size = party.entry.members.len();
            if player_count + size > definition.max_players {
                break;
            }
            let team = if capacities.is_empty() {
                if team_count >= definition.max_teams {
                    break;
                }
                let id = team_count as u32;
                team_count += 1;
                id
            } else if let Some(index) = capacities.iter().position(|left| *left >= size) {
                capacities[index] -= size;
                if !used_teams[index] {
                    used_teams[index] = true;
                    team_count += 1;
                }
                index as u32
            } else {
                break;
            };
            player_count += size;
            oldest = oldest.min(party.joined_at_ms);
            selected.push((party.request_id, team, party.entry.members.clone()));
        }
        if player_count < definition.min_players || team_count < definition.min_teams {
            return None;
        }
        let full = player_count == definition.max_players
            || (!capacities.is_empty() && capacities.iter().all(|left| *left == 0));
        let delayed = now.saturating_sub(oldest) >= definition.max_wait_ms;
        if !full && !delayed {
            return None;
        }

        for (request_id, _, _) in &selected {
            if let Some(index) = queue
                .waiting
                .iter()
                .position(|party| party.request_id == *request_id)
            {
                queue.waiting.remove(index);
            }
        }
        let mut members = Vec::with_capacity(player_count);
        for (_, team, participants) in selected {
            for participant in participants {
                members.push((participant, team));
                self.reservations
                    .insert(participant.session_id, Reservation::Ready { queue_id });
            }
        }
        Some(Formation { members })
    }

    fn begin_ready_check(
        &mut self,
        queue_id: u32,
        formation: Formation,
        now: u64,
    ) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        let definition = &self.queues[&queue_id].definition;
        let launch_id = self.next_launch_id;
        self.next_launch_id = self
            .next_launch_id
            .checked_add(1)
            .ok_or(MatchmakingError::InvalidQueue)?;
        let members: Vec<_> = formation
            .members
            .into_iter()
            .map(|(participant, team)| (participant, team, false))
            .collect();
        let packet = GroupLaunchPacket::header_only(14, launch_id, queue_id as i32)?;
        let out = members
            .iter()
            .map(|(participant, _, _)| SessionOutbound {
                recipient: participant.session_id,
                bytes: packet.clone(),
            })
            .collect();
        let timeout = definition.ready_timeout_ms;
        let spec = definition.run_spec.clone();
        self.queues.get_mut(&queue_id).unwrap().ready = Some(ReadyCheck {
            launch_id,
            deadline_ms: now.saturating_add(timeout),
            phase: ReadyPhase::Checking,
            members,
            run_spec: spec,
        });
        Ok(out)
    }

    fn start_match(&mut self, queue_id: u32) -> Result<Vec<SessionOutbound>, MatchmakingError> {
        let check = self
            .queues
            .get_mut(&queue_id)
            .unwrap()
            .ready
            .take()
            .unwrap();
        let participants: Vec<_> = check.members.iter().map(|(p, _, _)| *p).collect();
        let instance_id = self.next_instance_id;
        let next_id = instance_id
            .checked_add(1)
            .ok_or(MatchmakingError::InvalidQueue)?;
        let mut lifecycle = MinigameRuns::new(instance_id)
            .map_err(|error| MatchmakingError::Lifecycle(error.to_string()))?;
        let offered = lifecycle
            .offer(check.run_spec)
            .map_err(|error| MatchmakingError::Lifecycle(error.to_string()))?;
        if !offered.accepted || offered.replies.len() != 1 {
            return Err(MatchmakingError::Lifecycle("offer was not accepted".into()));
        }
        let launch = GroupLaunchPacket::header_only(15, check.launch_id, queue_id as i32)?;
        let roster = encode_activity_launched(
            check.launch_id,
            queue_id as i32,
            &participants
                .iter()
                .map(|p| p.character_guid)
                .collect::<Vec<_>>(),
        )?;
        let mut out = Vec::new();
        let mut active_participants = BTreeMap::new();
        for participant in participants {
            out.push(SessionOutbound {
                recipient: participant.session_id,
                bytes: launch.clone(),
            });
            out.push(SessionOutbound {
                recipient: participant.session_id,
                bytes: roster.clone(),
            });
            out.push(SessionOutbound {
                recipient: participant.session_id,
                bytes: offered.replies[0].clone(),
            });
            active_participants.insert(participant.session_id, participant);
            self.reservations.insert(
                participant.session_id,
                Reservation::Running {
                    queue_id,
                    match_id: instance_id,
                },
            );
        }
        self.matches.insert(
            instance_id,
            ActiveMatch {
                participants: active_participants,
                lifecycle,
            },
        );
        self.next_instance_id = next_id;
        Ok(out)
    }

    fn unique_request_id(&mut self) -> u64 {
        let mut value = self.rng.next_u64();
        if value == 0 {
            value = 1;
        }
        while self.used_request_ids.contains(&value) {
            value = value.wrapping_add(1).max(1);
        }
        self.used_request_ids.insert(value);
        value
    }

    fn now(&mut self) -> u64 {
        let supplied = self.clock.now_ms();
        self.last_now_ms = supplied.max(self.last_now_ms);
        self.last_now_ms
    }
}

struct Formation {
    members: Vec<(Participant, u32)>,
}

fn validate_entry(
    definition: &QueueDefinition,
    entry: &QueueEntry,
) -> Result<(), MatchmakingError> {
    if entry.members.is_empty()
        || entry.members.len() > definition.max_players
        || !entry
            .members
            .iter()
            .any(|member| member.session_id == entry.leader)
        || entry
            .members
            .iter()
            .any(|member| member.session_id.0 == 0 || member.character_guid == 0)
        || entry
            .members
            .iter()
            .map(|member| member.session_id)
            .collect::<BTreeSet<_>>()
            .len()
            != entry.members.len()
        || (entry.members.len() == 1 && !definition.allow_solo)
        || (entry.members.len() > 1 && (!definition.allow_groups || entry.group_id.is_none()))
        || (entry.members.len() == 1 && entry.group_id.is_some())
    {
        return Err(MatchmakingError::InvalidEntry);
    }
    if !definition.team_sizes.is_empty()
        && !definition
            .team_sizes
            .iter()
            .any(|capacity| *capacity >= entry.members.len())
    {
        return Err(MatchmakingError::QueueFullForParty);
    }
    Ok(())
}

/// Captured world-tunnel wrapper. Its outer opcode and flag are preserved;
/// captured 141/1 and 141/2 frames use outer opcodes 6 and 5 respectively.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TunnelFrame {
    pub outer_opcode: u16,
    pub flag: u8,
    pub inner: Vec<u8>,
}

impl TunnelFrame {
    pub fn encode(&self) -> Result<Vec<u8>, MatchmakingError> {
        let length =
            u32::try_from(self.inner.len()).map_err(|_| MatchmakingError::PacketTooLarge)?;
        let mut bytes = Vec::with_capacity(OUTER_HEADER_BYTES + self.inner.len());
        bytes.extend(self.outer_opcode.to_le_bytes());
        bytes.push(self.flag);
        bytes.extend(length.to_le_bytes());
        bytes.extend(&self.inner);
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MatchmakingError> {
        if bytes.len() < OUTER_HEADER_BYTES {
            return Err(MatchmakingError::InvalidPacket);
        }
        let length = read_u32(bytes, 3)? as usize;
        if length > 1024 * 1024 || bytes.len() != OUTER_HEADER_BYTES + length {
            return Err(MatchmakingError::InvalidPacket);
        }
        Ok(Self {
            outer_opcode: read_u16(bytes, 0)?,
            flag: bytes[2],
            inner: bytes[OUTER_HEADER_BYTES..].to_vec(),
        })
    }
}

/// Generic family-141 envelope. Unknown bodies remain byte-exact instead of
/// assigning meanings to fields the native map has not resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Packet141 {
    pub sub_op: u16,
    pub body: Vec<u8>,
}

impl Packet141 {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.body.len());
        bytes.extend(FAMILY_MATCHMAKING.to_le_bytes());
        bytes.extend(self.sub_op.to_le_bytes());
        bytes.extend(&self.body);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MatchmakingError> {
        if bytes.len() < 4
            || bytes.len() > 1024 * 1024 + 4
            || read_u16(bytes, 0)? != FAMILY_MATCHMAKING
        {
            return Err(MatchmakingError::InvalidPacket);
        }
        Ok(Self {
            sub_op: read_u16(bytes, 2)?,
            body: bytes[4..].to_vec(),
        })
    }
}

/// 141/1: client queue catalogue request; body is the two-word session key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ListQueuesRequest {
    pub session_key: u64,
}

impl ListQueuesRequest {
    pub fn encode(self) -> Vec<u8> {
        Packet141 {
            sub_op: 1,
            body: self.session_key.to_le_bytes().to_vec(),
        }
        .encode()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MatchmakingError> {
        let packet = Packet141::decode(bytes)?;
        if packet.sub_op != 1 || packet.body.len() != 8 {
            return Err(MatchmakingError::InvalidPacket);
        }
        Ok(Self {
            session_key: read_u64(&packet.body, 0)?,
        })
    }
}

/// 141/6 client cancellation carries the request's two-word 64-bit id.
pub fn encode_cancel_match_request(request_id: u64) -> Vec<u8> {
    Packet141 {
        sub_op: 6,
        body: request_id.to_le_bytes().to_vec(),
    }
    .encode()
}

pub fn decode_cancel_match_request(bytes: &[u8]) -> Result<u64, MatchmakingError> {
    let packet = Packet141::decode(bytes)?;
    if packet.sub_op != 6 || packet.body.len() != 8 {
        return Err(MatchmakingError::InvalidPacket);
    }
    read_u64(&packet.body, 0)
}

/// 141/15 matchmaking service status has one boolean byte after the header.
pub fn encode_server_status(is_up: bool) -> Vec<u8> {
    Packet141 {
        sub_op: 15,
        body: vec![u8::from(is_up)],
    }
    .encode()
}

pub fn decode_server_status(bytes: &[u8]) -> Result<bool, MatchmakingError> {
    let packet = Packet141::decode(bytes)?;
    if packet.sub_op != 15 || packet.body.len() != 1 || packet.body[0] > 1 {
        return Err(MatchmakingError::InvalidPacket);
    }
    Ok(packet.body[0] != 0)
}

/// Native 141/2 fixed-width row, retaining all nineteen dwords and both flags.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueRow {
    pub wire_dwords: [u32; 19],
    pub member_only: u8,
    pub unused_flag: u8,
}

impl QueueRow {
    pub fn new(
        queue_id: u32,
        name_id: u32,
        match_type: u32,
        min_players: u32,
        max_players: u32,
        min_teams: u32,
        max_teams: u32,
        max_game_start_delay: u32,
        params: [u32; 7],
        encounter_description: u32,
        encounter_icon: u32,
        unknown: [u32; 2],
        member_only: u8,
        unused_flag: u8,
    ) -> Self {
        let mut wire_dwords = [0; 19];
        wire_dwords[0] = queue_id;
        wire_dwords[1] = name_id;
        wire_dwords[2] = match_type;
        wire_dwords[3] = min_players;
        wire_dwords[4] = max_players;
        wire_dwords[5] = min_teams;
        wire_dwords[6] = max_teams;
        wire_dwords[7] = max_game_start_delay;
        wire_dwords[8..15].copy_from_slice(&params);
        wire_dwords[15] = encounter_description;
        wire_dwords[16] = encounter_icon;
        wire_dwords[17..19].copy_from_slice(&unknown);
        Self {
            wire_dwords,
            member_only,
            unused_flag,
        }
    }

    pub fn queue_id(&self) -> u32 {
        self.wire_dwords[0]
    }

    fn encode_into(&self, bytes: &mut Vec<u8>) {
        for value in self.wire_dwords {
            bytes.extend(value.to_le_bytes());
        }
        bytes.push(self.member_only);
        bytes.push(self.unused_flag);
    }

    fn decode_from(bytes: &[u8]) -> Result<Self, MatchmakingError> {
        if bytes.len() < QUEUE_ROW_BYTES {
            return Err(MatchmakingError::InvalidPacket);
        }
        let mut wire_dwords = [0; 19];
        for (index, value) in wire_dwords.iter_mut().enumerate() {
            *value = read_u32(bytes, index * 4)?;
        }
        Ok(Self {
            wire_dwords,
            member_only: bytes[76],
            unused_flag: bytes[77],
        })
    }
}

/// 141/2 response: session key, counted fixed rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueListResponse {
    pub session_key: u64,
    pub rows: Vec<QueueRow>,
}

impl QueueListResponse {
    pub fn encode(&self) -> Result<Vec<u8>, MatchmakingError> {
        check_count(self.rows.len())?;
        let mut body = Vec::with_capacity(12 + self.rows.len() * QUEUE_ROW_BYTES);
        body.extend(self.session_key.to_le_bytes());
        body.extend((self.rows.len() as u32).to_le_bytes());
        for row in &self.rows {
            row.encode_into(&mut body);
        }
        Ok(Packet141 { sub_op: 2, body }.encode())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MatchmakingError> {
        let packet = Packet141::decode(bytes)?;
        if packet.sub_op != 2 || packet.body.len() < 12 {
            return Err(MatchmakingError::InvalidPacket);
        }
        let session_key = read_u64(&packet.body, 0)?;
        let count = checked_count(read_u32(&packet.body, 8)?)?;
        let expected = 12usize
            .checked_add(
                count
                    .checked_mul(QUEUE_ROW_BYTES)
                    .ok_or(MatchmakingError::PacketTooLarge)?,
            )
            .ok_or(MatchmakingError::PacketTooLarge)?;
        if packet.body.len() != expected {
            return Err(MatchmakingError::InvalidPacket);
        }
        let rows = packet.body[12..]
            .chunks_exact(QUEUE_ROW_BYTES)
            .map(QueueRow::decode_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { session_key, rows })
    }
}

/// 141/14 response has two count-prefixed u32 arrays; their meaning is the
/// UI's pending-request and average-wait columns, not names recovered by bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueStatsResponse {
    pub session_key: u64,
    pub values_a: Vec<u32>,
    pub values_b: Vec<u32>,
}

impl QueueStatsResponse {
    pub fn encode(&self) -> Result<Vec<u8>, MatchmakingError> {
        check_count(self.values_a.len())?;
        check_count(self.values_b.len())?;
        let mut body = Vec::new();
        body.extend(self.session_key.to_le_bytes());
        write_u32_array(&mut body, &self.values_a);
        write_u32_array(&mut body, &self.values_b);
        Ok(Packet141 { sub_op: 14, body }.encode())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MatchmakingError> {
        let packet = Packet141::decode(bytes)?;
        if packet.sub_op != 14 || packet.body.len() < 16 {
            return Err(MatchmakingError::InvalidPacket);
        }
        let session_key = read_u64(&packet.body, 0)?;
        let (values_a, cursor) = read_u32_array(&packet.body, 8)?;
        let (values_b, cursor) = read_u32_array(&packet.body, cursor)?;
        if cursor != packet.body.len() {
            return Err(MatchmakingError::InvalidPacket);
        }
        Ok(Self {
            session_key,
            values_a,
            values_b,
        })
    }
}

/// Exact header-only family-175 message. Semantics of the last two header
/// words are caller supplied; the client reads a 14-byte header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupLaunchPacket {
    pub sub_op: i32,
    pub request_word: i32,
    pub context_word: i32,
    pub body: Vec<u8>,
}

impl GroupLaunchPacket {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(14 + self.body.len());
        bytes.extend(FAMILY_GROUP_LAUNCH.to_le_bytes());
        bytes.extend(self.sub_op.to_le_bytes());
        bytes.extend(self.request_word.to_le_bytes());
        bytes.extend(self.context_word.to_le_bytes());
        bytes.extend(&self.body);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MatchmakingError> {
        if bytes.len() < 14
            || bytes.len() > 1024 * 1024 + 14
            || read_u16(bytes, 0)? != FAMILY_GROUP_LAUNCH
        {
            return Err(MatchmakingError::InvalidPacket);
        }
        Ok(Self {
            sub_op: read_i32(bytes, 2)?,
            request_word: read_i32(bytes, 6)?,
            context_word: read_i32(bytes, 10)?,
            body: bytes[14..].to_vec(),
        })
    }

    pub fn header_only(
        sub_op: i32,
        request_word: i32,
        context_word: i32,
    ) -> Result<Vec<u8>, MatchmakingError> {
        if !matches!(sub_op, 4 | 14 | 15 | 16 | 18 | 19 | 22 | 23) {
            return Err(MatchmakingError::InvalidPacket);
        }
        Ok(Self {
            sub_op,
            request_word,
            context_word,
            body: Vec::new(),
        }
        .encode())
    }
}

/// 175/5: counted participant GUID pairs, used for ActivityLaunched.
pub fn encode_activity_launched(
    request_word: i32,
    context_word: i32,
    character_guids: &[u64],
) -> Result<Vec<u8>, MatchmakingError> {
    check_count(character_guids.len())?;
    let mut body = Vec::with_capacity(4 + 8 * character_guids.len());
    body.extend((character_guids.len() as u32).to_le_bytes());
    for guid in character_guids {
        body.extend(guid.to_le_bytes());
    }
    Ok(GroupLaunchPacket {
        sub_op: 5,
        request_word,
        context_word,
        body,
    }
    .encode())
}

pub fn decode_activity_launched(bytes: &[u8]) -> Result<(i32, i32, Vec<u64>), MatchmakingError> {
    let packet = GroupLaunchPacket::decode(bytes)?;
    if packet.sub_op != 5 || packet.body.len() < 4 {
        return Err(MatchmakingError::InvalidPacket);
    }
    let count = checked_count(read_u32(&packet.body, 0)?)?;
    let expected = 4usize
        .checked_add(
            count
                .checked_mul(8)
                .ok_or(MatchmakingError::PacketTooLarge)?,
        )
        .ok_or(MatchmakingError::PacketTooLarge)?;
    if packet.body.len() != expected {
        return Err(MatchmakingError::InvalidPacket);
    }
    let mut guids = Vec::with_capacity(count);
    for index in 0..count {
        guids.push(read_u64(&packet.body, 4 + index * 8)?);
    }
    Ok((packet.request_word, packet.context_word, guids))
}

/// 175/9: member id plus state byte. The caller supplies the native member id;
/// no mapping from a Rust session id to that client id is recovered here.
pub fn encode_member_state(
    request_word: i32,
    context_word: i32,
    member_id: u32,
    state: u8,
) -> Vec<u8> {
    let mut body = Vec::with_capacity(5);
    body.extend(member_id.to_le_bytes());
    body.push(state);
    GroupLaunchPacket {
        sub_op: 9,
        request_word,
        context_word,
        body,
    }
    .encode()
}

/// 175/17 client invite reply: boolean accept byte followed by a 32-bit reason.
pub fn encode_invite_response(
    request_word: i32,
    context_word: i32,
    accepted: bool,
    reason: u32,
) -> Vec<u8> {
    let mut body = Vec::with_capacity(5);
    body.push(u8::from(accepted));
    body.extend(reason.to_le_bytes());
    GroupLaunchPacket {
        sub_op: 17,
        request_word,
        context_word,
        body,
    }
    .encode()
}

pub fn decode_invite_response(bytes: &[u8]) -> Result<(i32, i32, bool, u32), MatchmakingError> {
    let packet = GroupLaunchPacket::decode(bytes)?;
    if packet.sub_op != 17 || packet.body.len() != 5 || packet.body[0] > 1 {
        return Err(MatchmakingError::InvalidPacket);
    }
    Ok((
        packet.request_word,
        packet.context_word,
        packet.body[0] != 0,
        read_u32(&packet.body, 1)?,
    ))
}

/// Family-175 server invite result (sub-op 2 or 3). The byte string is kept
/// opaque to avoid assigning player-facing text semantics in this layer; the
/// final u32 is the native result code, whose mapping to UI text is unknown.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InviteMemberNotice {
    pub sub_op: i32,
    pub request_word: i32,
    pub context_word: i32,
    pub character_guid: u64,
    pub label_bytes: Vec<u8>,
    pub status: bool,
    pub result_code: u32,
}

impl InviteMemberNotice {
    pub fn encode(&self) -> Result<Vec<u8>, MatchmakingError> {
        if !matches!(self.sub_op, 2 | 3) || self.label_bytes.len() > 1024 * 1024 {
            return Err(MatchmakingError::InvalidPacket);
        }
        let length =
            u32::try_from(self.label_bytes.len()).map_err(|_| MatchmakingError::PacketTooLarge)?;
        let mut body = Vec::with_capacity(8 + 4 + self.label_bytes.len() + 5);
        body.extend(self.character_guid.to_le_bytes());
        body.extend(length.to_le_bytes());
        body.extend(&self.label_bytes);
        body.push(u8::from(self.status));
        body.extend(self.result_code.to_le_bytes());
        Ok(GroupLaunchPacket {
            sub_op: self.sub_op,
            request_word: self.request_word,
            context_word: self.context_word,
            body,
        }
        .encode())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MatchmakingError> {
        let packet = GroupLaunchPacket::decode(bytes)?;
        if !matches!(packet.sub_op, 2 | 3) || packet.body.len() < 17 {
            return Err(MatchmakingError::InvalidPacket);
        }
        let length = usize::try_from(read_u32(&packet.body, 8)?)
            .map_err(|_| MatchmakingError::PacketTooLarge)?;
        if length > 1024 * 1024 {
            return Err(MatchmakingError::PacketTooLarge);
        }
        let status_offset = 12usize
            .checked_add(length)
            .ok_or(MatchmakingError::PacketTooLarge)?;
        if packet.body.len() != status_offset + 5 || packet.body[status_offset] > 1 {
            return Err(MatchmakingError::InvalidPacket);
        }
        Ok(Self {
            sub_op: packet.sub_op,
            request_word: packet.request_word,
            context_word: packet.context_word,
            character_guid: read_u64(&packet.body, 0)?,
            label_bytes: packet.body[12..status_offset].to_vec(),
            status: packet.body[status_offset] != 0,
            result_code: read_u32(&packet.body, status_offset + 1)?,
        })
    }
}

fn write_u32_array(bytes: &mut Vec<u8>, values: &[u32]) {
    bytes.extend((values.len() as u32).to_le_bytes());
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
}

fn read_u32_array(bytes: &[u8], offset: usize) -> Result<(Vec<u32>, usize), MatchmakingError> {
    let count = checked_count(read_u32(bytes, offset)?)?;
    let start = offset
        .checked_add(4)
        .ok_or(MatchmakingError::PacketTooLarge)?;
    let end = start
        .checked_add(
            count
                .checked_mul(4)
                .ok_or(MatchmakingError::PacketTooLarge)?,
        )
        .ok_or(MatchmakingError::PacketTooLarge)?;
    if end > bytes.len() {
        return Err(MatchmakingError::InvalidPacket);
    }
    let values = (0..count)
        .map(|index| read_u32(bytes, start + index * 4))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((values, end))
}

fn check_count(count: usize) -> Result<(), MatchmakingError> {
    if count > MAX_WIRE_ROWS {
        Err(MatchmakingError::PacketTooLarge)
    } else {
        Ok(())
    }
}

fn checked_count(count: u32) -> Result<usize, MatchmakingError> {
    let count = usize::try_from(count).map_err(|_| MatchmakingError::PacketTooLarge)?;
    check_count(count)?;
    Ok(count)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, MatchmakingError> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or(MatchmakingError::InvalidPacket)?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, MatchmakingError> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or(MatchmakingError::InvalidPacket)?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, MatchmakingError> {
    Ok(read_u32(bytes, offset)? as i32)
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, MatchmakingError> {
    let slice = bytes
        .get(offset..offset + 8)
        .ok_or(MatchmakingError::InvalidPacket)?;
    Ok(u64::from_le_bytes([
        slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
    ]))
}
