//! Bounded native soccer lifecycle/wire service for the MMO005 restoration.
//!
//! This module is the callable seam between a gateway session and the
//! client-reader-derived soccer wire codec, `soccer_match` (server-owned
//! authority) and the frozen private
//! sub-14 state adapter
//! (`lab/reports/soccer-state-adapter-20261009T174320Z/src/adapter.rs`, sha256
//! `FROZEN_ADAPTER_SHA256`, 13 lead + 14 builder tests). The adapter code below
//! is copied verbatim apart from doc-comment corrections and its import paths
//! (`crate::` here, so the module compiles both inside this crate and as the
//! owned `#[path]` test module); its equations are not rewritten here.
//!
//! Evidence split, kept explicit on purpose:
//!
//! - **Client facts.** The native readers establish the family-76 envelope and
//!   layouts for Soccer sub-14, sub-16 and sub-13. Sub-14 vecA is the action
//!   direction and aux is quantized charge; its `u32` is a client state-change
//!   counter, not transport replay state. See the Soccer build guide and the
//!   accepted native-map labels it cites.
//! - **Reconstructed policy (R).** Every roster limit, team assignment, bot,
//!   clock, physics value, impulse output mapping, scoreboard relay, lifecycle
//!   transition, result and reward in this service. Client readers establish
//!   packet shape, not historical server values or behavior.
//! - **Open evidence.** The native client path does not establish the original
//!   match launch, result exchange, reward table, or authoritative server
//!   emission of sub-13. Those remain unverified.
//!
//! The service serializes a bounded set of candidate outputs. Only sub-13's
//! body layout in that output set has a cited client reader here; all output
//! packet choices, field values and acceptance by a retail Soccer client remain
//! R/open. Opaque position/ball tails, game-state words and unrecovered
//! statistics are not authored.
//!
//! This is a bounded server slice. It makes no native playability claim, sends
//! no packet and touches no shared gateway, content or save file.

#![allow(dead_code)] // Module is exercised by an owned `#[path]` test until the lead registers it.

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use crate::minigame::{
    Activity, ActivityFinish, MinigameRuns, RunMode, RunOutcome, RunSpec, RunState,
};
use crate::rewards::{RewardGrant, RewardState};
use crate::soccer::{self, Evidence, Message, SoccerPacket, ZONE};
use crate::soccer_match::{
    Authority, ClientPacket, Intent, KickKind, MatchConfig, MatchPhase, Outcome, PlayerState,
    StateAction, StateRequest, TEAM_SLOT_COUNT, classify_state, client_packet,
};
use crate::zone_transition::{self, ZoneTransition};

/// Frozen private sub-14 adapter source pin (lead + builder verified).
pub const FROZEN_ADAPTER_SHA256: &str =
    "11d291c2284922e3cde7549632a969b85261a2c12a7ba59464178281c2de5dd7";

/// R fixture spawn position; it is not a native client fact.
pub const R_ENTRY_POSITION: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
/// R fixture spawn rotation; it is not a native client fact.
pub const R_ENTRY_ROTATION: [f32; 4] = [0.0, 0.0, 1.0, 0.0];
/// R candidate for the begin-zoning envelope flag.
pub const ENTRY_FLAG: u8 = 1;
/// R candidate begin-zoning type.
pub const ENTRY_TYPE: i32 = 2;
/// R candidate sky definition.
pub const ENTRY_SKY: &str = "sky_wilds24.xml";
/// R candidate begin-zoning byte.
pub const ENTRY_UNKNOWN: u8 = 1;
/// R candidate geometry id.
pub const ENTRY_GEOMETRY_ID: i32 = 107;
/// R candidate server registration marker.
pub const REGISTER_MARKER: [u8; 2] = [0, 1];
/// R candidate possession flag.
pub const POSSESSION_FLAG: u8 = 0;
/// R candidate free-ball scale.
pub const FREE_BALL_SCALE: f32 = 1.0;
/// R sentinel for "no previous holder".
pub const NO_HOLDER: u64 = 0;
/// R candidate value for the sub-13 leading word.
pub const IMPULSE_WORD: u32 = 0;
/// Sub-13 fourth vector component: unnamed, emitted as the reconstructed zero.
pub const IMPULSE_W: f32 = 0.0;
/// R shot action code; the client groups state 1 against states 0/2/3.
pub const SHOT_ACTION: u32 = 0;
/// R pass action code.
pub const PASS_ACTION: u32 = 1;

const BOT_ID_BASE: u64 = u64::MAX - 7;

#[derive(Clone)]
struct ParticipantRun {
    runs: MinigameRuns,
    instance_id: i32,
    team: u32,
    rewards: RewardState,
    grant: RewardGrant,
}

/// Generic lifecycle plus soccer cleanup from a 39/* client packet.
pub struct LifecycleChange {
    pub run: RunOutcome,
    pub soccer_events: Vec<ServiceEvent>,
}

/// R result adapter: report team goals as score, award one star to a win,
/// treat ties as losses for the per-player generic result, and round elapsed
/// server time to whole seconds.
struct SoccerActivity<'a> {
    authority: &'a Authority,
    team: u32,
}

impl Activity for SoccerActivity<'_> {
    fn score(&self) -> u32 {
        self.authority.score()[self.team as usize]
    }

    fn finish(&self) -> Option<ActivityFinish> {
        let outcome = self.authority.outcome()?;
        let won = outcome.winner == Some(self.team);
        Some(ActivityFinish {
            won,
            stars: u32::from(won),
            elapsed_seconds: self.authority.elapsed().round() as u32,
        })
    }
}

/// Validate the RECONSTRUCTED candidate policy before it shapes any intent:
/// the two scales must be finite and positive, and the charge floor must be a
/// finite fraction in `0..=1`. Copied from the frozen adapter.
fn validate_policy(policy: &AdapterPolicy) -> io::Result<()> {
    if !policy.max_power.is_finite() || policy.max_power <= 0.0 {
        return Err(invalid(
            "soccer adapter policy max_power must be finite and positive",
        ));
    }
    if !policy.min_direction_norm.is_finite() || policy.min_direction_norm <= 0.0 {
        return Err(invalid(
            "soccer adapter policy min_direction_norm must be finite and positive",
        ));
    }
    if !policy.charge_floor.is_finite() || policy.charge_floor < 0.0 || policy.charge_floor > 1.0 {
        return Err(invalid(
            "soccer adapter policy charge_floor must be finite and in 0..=1",
        ));
    }
    Ok(())
}

/// RECONSTRUCTED service policy, identical to the frozen adapter's:
/// `max_power` scales the proven `aux / 255` charge into intent power and the
/// authority's own `max_kick_power` bounds the resulting value; `charge_floor`
/// is the explicit refusal for zero/near-zero charge; `min_direction_norm`
/// rejects a degenerate vecA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdapterPolicy {
    pub max_power: f32,
    pub charge_floor: f32,
    pub min_direction_norm: f32,
}

/// Proven native charge scaling: `aux / 255` (`DAT_017ed9d0 = 255.0`,
/// `DAT_017f1ae4 = 1/255`).
pub fn charge_from_aux(aux: u8) -> f32 {
    f32::from(aux) / 255.0
}

/// Build the server-owned kick intent for one typed sub-14 state request.
///
/// Copied from the frozen private adapter; the authority's own guards still
/// gate application. `target` is the caller-supplied, server-verified teammate
/// id; it is never guessed from the unnamed `player+0x1C == 3` selector.
pub fn state_request_intent(
    request: &StateRequest,
    policy: &AdapterPolicy,
    target: Option<u64>,
) -> io::Result<Intent> {
    validate_policy(policy)?;
    if match request.action {
        StateAction::Kick => !matches!(classify_state(request.state), Some(StateAction::Kick)),
        StateAction::Pass => !matches!(classify_state(request.state), Some(StateAction::Pass)),
    } {
        return Err(invalid(
            "soccer state action does not match the classified state",
        ));
    }
    let vec_a = request.action_direction;
    let direction = [vec_a[0], vec_a[1], vec_a[2]];
    if !direction.iter().all(|value| value.is_finite()) {
        return Err(invalid("soccer state direction is not finite"));
    }
    let norm = direction
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        .sqrt();
    if norm < f64::from(policy.min_direction_norm) {
        return Err(invalid("soccer state direction is degenerate"));
    }

    let charge = charge_from_aux(request.aux);
    let power = charge * policy.max_power;
    // The candidate refuses below its reconstructed floor. A non-positive
    // power is left to the authority's own `(0, max_kick_power]` bound, which
    // already refuses zero power before any mutation.
    if charge < policy.charge_floor || !power.is_finite() {
        return Err(invalid(
            "soccer state power below the reconstructed candidate floor (native aux 0 may be a normal uncharged kick)",
        ));
    }

    let kind = match request.action {
        StateAction::Kick => {
            if target.is_some() {
                return Err(invalid("soccer kick state must not carry a pass target"));
            }
            KickKind::Shot
        }
        StateAction::Pass => {
            let target = target.ok_or_else(|| {
                invalid("soccer pass state needs an explicit server-verified target")
            })?;
            if target == request.player {
                return Err(invalid("soccer pass target is the kicker"));
            }
            KickKind::Pass
        }
    };

    Ok(Intent::Kick {
        player: request.player,
        sequence: request.sequence,
        kind,
        direction,
        power,
        target: match kind {
            KickKind::Shot => None,
            KickKind::Pass => target,
        },
    })
}

/// Validate and apply one sub-14 state request through the existing
/// `Authority::apply` contract. Copied from the frozen private adapter.
///
/// `sender` is the authenticated session guid and `now` the authoritative
/// server time in seconds; both are gateway inputs, never payload fields. All
/// validation runs before the authority is touched, so every refusal leaves
/// the match snapshot unchanged.
pub fn apply_state_request(
    authority: &mut Authority,
    sender: u64,
    now: f32,
    request: &StateRequest,
    policy: &AdapterPolicy,
    target: Option<u64>,
) -> io::Result<()> {
    if sender != request.player {
        return Err(invalid(
            "soccer state sender does not match the payload player",
        ));
    }
    let state = authority
        .player(request.player)
        .ok_or_else(|| invalid("soccer state from an unregistered player"))?;
    if request.sequence <= state.sequence {
        return Err(invalid("stale soccer state request"));
    }
    if classify_state(request.state) != Some(request.action) {
        return Err(invalid(
            "soccer state action does not match the classified state",
        ));
    }
    if request.action == StateAction::Pass {
        let target = target
            .ok_or_else(|| invalid("soccer pass state needs an explicit server-verified target"))?;
        let target_state: &PlayerState = authority
            .player(target)
            .ok_or_else(|| invalid("soccer pass target is unregistered"))?;
        if target_state.team != state.team {
            return Err(invalid("soccer pass target is not a teammate"));
        }
    }
    let intent = state_request_intent(request, policy, target)?;
    authority.apply(sender, now, intent)
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn wrap(packet: SoccerPacket) -> Vec<u8> {
    soccer::encode(&Message {
        outer: 5,
        flag: true,
        packet,
    })
}

/// One server event and its R-selected packet mapping. Only sub-13 has a
/// cited client reader in this service's emitted set; original server output
/// layouts and field values for these events remain open evidence.
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceEvent {
    /// R server-assigned roster entry.
    Registered { player: u64, marker: [u8; 2] },
    /// R server-assigned team/slot for one player.
    TeamAssigned { player: u64, team: u32, slot: u32 },
    /// R possession relay; `previous` is the last reported holder (`0` none).
    Possession { holder: u64, previous: u64 },
    /// R free-ball relay at `position`; `previous` is the last reported holder.
    FreeBall {
        position: [f32; 3],
        scale: f32,
        previous: u64,
    },
    /// R scoreboard step; the candidate wire carries `[a, b, a + b]`.
    Goal { score: [u32; 2] },
    /// R sub-13 impulse candidate for an accepted kick/pass. `vector` is the
    /// server unit direction (`w` zero); `charge` uses the client-proven
    /// `aux / 255`; `power`, `action` and `value_c` mapping remain R.
    Impulse {
        player: u64,
        kind: KickKind,
        vector: [f32; 4],
        charge: f32,
        power: f32,
        action: u32,
        value_c: f32,
    },
    /// R match clock.
    GameTime { remaining: f32, elapsed: f32 },
}

impl ServiceEvent {
    /// The codec packet layout selected by this R event.
    pub fn packet(&self) -> SoccerPacket {
        match self {
            ServiceEvent::Registered { player, marker } => SoccerPacket::RegisterPlayer {
                player: *player,
                marker: *marker,
            },
            ServiceEvent::TeamAssigned { player, team, slot } => SoccerPacket::SetPlayerTeam {
                player: *player,
                team: *team,
                slot: *slot,
            },
            ServiceEvent::Possession { holder, previous } => SoccerPacket::AcquireBall {
                flag: POSSESSION_FLAG,
                holder: *holder,
                previous: *previous,
            },
            ServiceEvent::FreeBall {
                position,
                scale,
                previous,
            } => SoccerPacket::FreeBall {
                flag: POSSESSION_FLAG,
                position: *position,
                scale: *scale,
                previous: *previous,
            },
            ServiceEvent::Goal { score } => SoccerPacket::GoalMade {
                fields: [score[0], score[1], score[0] + score[1]],
            },
            ServiceEvent::Impulse {
                vector,
                charge,
                power,
                action,
                value_c,
                ..
            } => SoccerPacket::ImpulseBall {
                word: IMPULSE_WORD,
                vector: *vector,
                value_a: *charge,
                value_b: *power,
                action: *action,
                value_c: *value_c,
            },
            ServiceEvent::GameTime { remaining, elapsed } => SoccerPacket::UpdateGameTime {
                remaining: *remaining,
                elapsed: *elapsed,
            },
        }
    }

    /// The family-76 envelope around this R event's payload.
    pub fn wire(&self) -> Vec<u8> {
        wrap(self.packet())
    }

    /// The inner family-76 body for `Outcome::replies` (the gateway adds the
    /// tunnel envelope itself).
    pub fn inner(&self) -> Vec<u8> {
        soccer::encode_inner(&self.packet())
    }
}

/// Codec gate for the service's bounded R output set. It refuses variants with
/// opaque tails or unsupported fields; this does not establish client acceptance.
pub fn emit_packet(packet: &SoccerPacket) -> io::Result<Vec<u8>> {
    match packet {
        SoccerPacket::RegisterPlayer { .. }
        | SoccerPacket::SetPlayerTeam { .. }
        | SoccerPacket::GoalMade { .. }
        | SoccerPacket::AcquireBall { .. }
        | SoccerPacket::FreeBall { .. }
        | SoccerPacket::ImpulseBall { .. }
        | SoccerPacket::UpdateGameTime { .. } => Ok(wrap(packet.clone())),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "soccer packet {other:?} is not emittable by this service: {}",
                unsupported_reason(other)
            ),
        )),
    }
}

fn unsupported_reason(packet: &SoccerPacket) -> &'static str {
    match packet {
        SoccerPacket::UpdatePlayerPos { .. } | SoccerPacket::UpdateSoccerBall { .. } => {
            "its trailing block is not established by the native reader map"
        }
        SoccerPacket::UpdateGameState { .. } | SoccerPacket::BallBounce { .. } => {
            "its word block has no recovered meaning"
        }
        SoccerPacket::SetClientConfig { .. }
        | SoccerPacket::TeamColours { .. }
        | SoccerPacket::SpawnPickUp { .. }
        | SoccerPacket::AcquirePickUp { .. } => {
            "its native reader layout is outside this task's recovered set"
        }
        SoccerPacket::UpdatePlayerState { .. } | SoccerPacket::UpdatePlayerControls { .. } => {
            "it is a client-authored relay packet, not server authority"
        }
        SoccerPacket::UpdatePlayerStats { .. }
        | SoccerPacket::PlayerVariableStats { .. }
        | SoccerPacket::SetSoccerPlayerOneTimeStats { .. } => {
            "its statistic values are not recovered"
        }
        SoccerPacket::HighArcBall { .. } => "its trailing bytes are opaque",
        SoccerPacket::Unknown { .. } => "its payload layout was never recovered",
        _ => "it is not in this slice's emission set",
    }
}

/// A joined session's server-assigned roster entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SessionEntry {
    team: u32,
    slot: u32,
}

/// Bounded soccer session service: roster entry/exit, kickoff/tick lifecycle and
/// client family-76 processing over one `Authority`.
///
/// The session guid is the authenticated identity guid used by the gateway,
/// which is also the authority's player id, so a client packet can never name
/// another session; [`SoccerService::handle_client`] takes the authenticated
/// sender separately from the payload and the authority refuses a mismatch.
#[derive(Clone)]
pub struct SoccerService {
    authority: Authority,
    policy: AdapterPolicy,
    sessions: BTreeMap<u64, SessionEntry>,
    runs: BTreeMap<u64, ParticipantRun>,
    bots: BTreeSet<u64>,
    next_instance_id: i32,
    bots_filled: bool,
    cancelled: bool,
    /// R relay bookkeeping: last holder reported on the service wire (`0` none).
    last_holder: u64,
}

impl SoccerService {
    /// Create the service from caller-supplied match bounds and adapter policy.
    /// Field dimensions and all physics values are R configuration.
    pub fn new(config: MatchConfig, policy: AdapterPolicy) -> io::Result<Self> {
        validate_policy(&policy)?;
        Ok(Self {
            authority: Authority::new(config)?,
            policy,
            sessions: BTreeMap::new(),
            runs: BTreeMap::new(),
            bots: BTreeSet::new(),
            next_instance_id: 1,
            bots_filled: false,
            cancelled: false,
            last_holder: NO_HOLDER,
        })
    }

    pub fn authority(&self) -> &Authority {
        &self.authority
    }

    pub fn policy(&self) -> AdapterPolicy {
        self.policy
    }

    pub fn phase(&self) -> MatchPhase {
        self.authority.phase()
    }

    pub fn score(&self) -> [u32; 2] {
        self.authority.score()
    }

    pub fn remaining(&self) -> f32 {
        self.authority.remaining()
    }

    pub fn elapsed(&self) -> f32 {
        self.authority.elapsed()
    }

    pub fn revision(&self) -> u64 {
        self.authority.revision()
    }

    pub fn ball_position(&self) -> [f32; 3] {
        self.authority.ball_position()
    }

    pub fn ball_velocity(&self) -> [f32; 3] {
        self.authority.ball_velocity()
    }

    pub fn possession(&self) -> Option<u64> {
        self.authority.possession()
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.authority.outcome()
    }

    pub fn player(&self, session: u64) -> Option<&PlayerState> {
        self.authority.player(session)
    }

    pub fn session_team(&self, session: u64) -> Option<(u32, u32)> {
        self.sessions
            .get(&session)
            .map(|entry| (entry.team, entry.slot))
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn bot_count(&self) -> usize {
        self.bots.len()
    }

    pub fn cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn run_state(&self, session: u64) -> Option<RunState> {
        let run = self.runs.get(&session)?;
        run.runs.state(run.instance_id)
    }

    pub fn instance_id(&self, session: u64) -> Option<i32> {
        self.runs.get(&session).map(|run| run.instance_id)
    }

    pub fn reward_state(&self, session: u64) -> Option<&RewardState> {
        self.runs.get(&session).map(|run| &run.rewards)
    }

    /// Server-side roster entry. Emits R registration and team assignment
    /// relays for the joining session. Duplicate joins, an
    /// out-of-range team and a taken slot are refused before any mutation.
    pub fn join(&mut self, session: u64, team: u32, slot: u32) -> io::Result<Vec<ServiceEvent>> {
        if self.cancelled || self.bots_filled || session == 0 || session >= BOT_ID_BASE {
            return Err(invalid("soccer session cannot join this match"));
        }
        if self.sessions.contains_key(&session) {
            return Err(invalid("soccer session is already joined"));
        }
        self.authority.register(session, team, slot)?;
        self.sessions.insert(session, SessionEntry { team, slot });
        Ok(vec![
            ServiceEvent::Registered {
                player: session,
                marker: REGISTER_MARKER,
            },
            ServiceEvent::TeamAssigned {
                player: session,
                team,
                slot,
            },
        ])
    }

    /// R: fill every open 4v4 roster slot with passive server bots for solo
    /// testing. Bots have no AI, never submit client intents, and never score.
    pub fn fill_bots_for_solo(&mut self, session: u64) -> io::Result<Vec<ServiceEvent>> {
        if self.cancelled
            || self.authority.phase() != MatchPhase::Kickoff
            || self.sessions.len() != 1
            || !self.sessions.contains_key(&session)
            || self.bots_filled
        {
            return Err(invalid("soccer solo bot fill is not available"));
        }
        let mut fill = Vec::new();
        for team in 0..2 {
            for slot in 0..TEAM_SLOT_COUNT {
                if self
                    .sessions
                    .get(&session)
                    .is_some_and(|entry| entry.team == team && entry.slot == slot)
                {
                    continue;
                }
                let bot = BOT_ID_BASE + (team * TEAM_SLOT_COUNT + slot) as u64;
                if self.authority.player(bot).is_some() {
                    return Err(invalid("soccer bot identity collision"));
                }
                fill.push((bot, team, slot));
            }
        }
        let mut events = Vec::with_capacity(fill.len() * 2);
        for (bot, team, slot) in fill {
            self.authority.register(bot, team, slot)?;
            self.bots.insert(bot);
            events.push(ServiceEvent::Registered {
                player: bot,
                marker: REGISTER_MARKER,
            });
            events.push(ServiceEvent::TeamAssigned {
                player: bot,
                team,
                slot,
            });
        }
        self.bots_filled = true;
        Ok(events)
    }

    /// Offer the type-12 Soccer run through the shared lifecycle. `game_id`,
    /// `encounter_id`, the 39/16 defaults and the reward grant are caller data
    /// or R policy; the native soccer completion/reward exchange is unknown.
    pub fn offer_minigame(
        &mut self,
        session: u64,
        game_id: i32,
        encounter_id: i32,
        rewards: RewardState,
        grant: RewardGrant,
    ) -> io::Result<RunOutcome> {
        if self.cancelled
            || self.authority.phase() != MatchPhase::Kickoff
            || !self.sessions.contains_key(&session)
            || self.runs.contains_key(&session)
            || game_id <= 0
            || encounter_id <= 0
        {
            return Err(invalid("soccer minigame offer is invalid"));
        }
        let instance_id = self.next_instance_id;
        let next_instance_id = instance_id
            .checked_add(1)
            .ok_or_else(|| invalid("soccer minigame instance counter exhausted"))?;
        let mut runs = MinigameRuns::new(instance_id)?;
        let offered = runs.offer(RunSpec::empty(
            game_id,
            soccer::SOCCER_TYPE_ID,
            RunMode::Encounter {
                encounter_id,
                player_guid: session,
            },
        ))?;
        if !offered.accepted {
            return Err(invalid("soccer minigame offer was refused"));
        }
        let team = self.sessions[&session].team;
        self.next_instance_id = next_instance_id;
        self.runs.insert(
            session,
            ParticipantRun {
                runs,
                instance_id,
                team,
                rewards,
                grant,
            },
        );
        Ok(offered)
    }

    /// Route shared 39/* lifecycle messages to MinigameRuns. A cancellation
    /// also removes the soccer roster entry and releases any held ball.
    pub fn handle_minigame_lifecycle(
        &mut self,
        session: u64,
        packet: &[u8],
    ) -> io::Result<LifecycleChange> {
        let outcome = self
            .runs
            .get_mut(&session)
            .ok_or_else(|| invalid("soccer minigame session is not offered"))?
            .runs
            .handle_client(packet)?;
        let soccer_events = if outcome.state == Some(RunState::Cancelled) {
            self.leave(session)?
        } else {
            Vec::new()
        };
        Ok(LifecycleChange {
            run: outcome,
            soccer_events,
        })
    }

    /// R adapter: complete the generic start handshake. The match begins when
    /// all connected humans have reached Playing; local bots need no run.
    pub fn mark_started(&mut self, session: u64) -> io::Result<Vec<ServiceEvent>> {
        if self.cancelled {
            return Err(invalid("soccer match is cancelled"));
        }
        let participant = self
            .runs
            .get_mut(&session)
            .ok_or_else(|| invalid("soccer minigame session is not offered"))?;
        let started = participant.runs.started(participant.instance_id);
        if !started.changed {
            return Err(invalid("soccer minigame is not starting"));
        }
        let all_ready = self.sessions.keys().all(|guid| {
            self.runs
                .get(guid)
                .is_some_and(|run| run.runs.state(run.instance_id) == Some(RunState::Playing))
        });
        if all_ready {
            self.start()
        } else {
            Ok(Vec::new())
        }
    }

    /// Settle the terminal score through MinigameRuns and rewards. The
    /// configured win grant, per-team outcome mapping and tie-as-loss rule are
    /// R; no historical Soccer reward amount is known.
    pub fn finish_minigame(&mut self, session: u64) -> io::Result<RunOutcome> {
        let run = self
            .runs
            .get_mut(&session)
            .ok_or_else(|| invalid("soccer minigame session is not offered"))?;
        if self.authority.phase() != MatchPhase::Finished {
            return Ok(RunOutcome::default());
        }
        let activity = SoccerActivity {
            authority: &self.authority,
            team: run.team,
        };
        let settlement = self.authority.outcome().and_then(|outcome| {
            (outcome.winner == Some(run.team)).then_some((&run.rewards, &run.grant))
        });
        let outcome = run.runs.finish(run.instance_id, &activity, settlement)?;
        if let Some(next) = outcome.reward_state.as_ref() {
            run.rewards = next.clone();
        }
        Ok(outcome)
    }

    /// Session departure. The roster entry is removed; if the leaver held the
    /// ball it is released at the authoritative position. The relay fields are
    /// R because the client reader does not establish server output values.
    pub fn leave(&mut self, session: u64) -> io::Result<Vec<ServiceEvent>> {
        if self.sessions.remove(&session).is_none() {
            return Err(invalid("soccer session is not joined"));
        }
        if let Some(run) = self.runs.get_mut(&session) {
            let _ = run.runs.disconnect();
        }
        let released = self.authority.possession() == Some(session);
        let ball = self.authority.ball_position();
        self.authority.unregister(session)?;
        if self.last_holder == session {
            self.last_holder = NO_HOLDER;
        }
        if released {
            self.cancelled = self.sessions.is_empty();
            Ok(vec![ServiceEvent::FreeBall {
                position: ball,
                scale: FREE_BALL_SCALE,
                previous: session,
            }])
        } else {
            self.cancelled = self.sessions.is_empty();
            Ok(Vec::new())
        }
    }

    /// The gateway owns the timeout threshold (R) and calls this on expiry.
    pub fn timeout(&mut self, session: u64) -> io::Result<Vec<ServiceEvent>> {
        self.disconnect(session)
    }

    /// Disconnect cancels that player's run without reward and removes them
    /// from authority. A solo match is cancelled when no human remains.
    pub fn disconnect(&mut self, session: u64) -> io::Result<Vec<ServiceEvent>> {
        self.leave(session)
    }

    /// R: kick off from `Kickoff` or a goal `Restart` after every joined human
    /// completes the shared minigame start handshake.
    pub fn start(&mut self) -> io::Result<Vec<ServiceEvent>> {
        if self.cancelled {
            return Err(invalid("soccer match is cancelled"));
        }
        if self.sessions.is_empty()
            || self.sessions.keys().any(|guid| {
                self.runs.get(guid).map_or(true, |run| {
                    run.runs.state(run.instance_id) != Some(RunState::Playing)
                })
            })
        {
            return Err(invalid("soccer minigame start handshake is incomplete"));
        }
        self.authority.kickoff()?;
        Ok(vec![self.game_time()])
    }

    /// Start the second half from the interval and report the clock.
    pub fn second_half(&mut self) -> io::Result<Vec<ServiceEvent>> {
        if self.cancelled {
            return Err(invalid("soccer match is cancelled"));
        }
        self.authority.start_second_half()?;
        Ok(vec![self.game_time()])
    }

    /// Advance the deterministic R simulation and report its candidate events:
    /// a goal, out-of-play free-ball relay, possession change, then the clock.
    /// Invalid steps are refused and
    /// leave the match untouched.
    pub fn tick(&mut self, dt: f32) -> io::Result<Vec<ServiceEvent>> {
        if self.cancelled {
            return Err(invalid("soccer match is cancelled"));
        }
        let before_phase = self.authority.phase();
        let before_score = self.authority.score();
        let before_possession = self.authority.possession();
        let previous_holder = self.last_holder;
        self.authority.advance(dt)?;

        if self.authority.score() != before_score {
            for participant in self.runs.values_mut() {
                let activity = SoccerActivity {
                    authority: &self.authority,
                    team: participant.team,
                };
                participant.runs.score(participant.instance_id, &activity);
            }
        }

        let mut events = Vec::new();
        if self.authority.score() != before_score {
            events.push(ServiceEvent::Goal {
                score: self.authority.score(),
            });
            self.last_holder = NO_HOLDER;
        } else if before_phase == MatchPhase::Running
            && self.authority.phase() == MatchPhase::Restart
        {
            // R out-of-play return: reset at the configured spawn and report
            // the last holder using the candidate codec layout.
            events.push(ServiceEvent::FreeBall {
                position: self.authority.ball_position(),
                scale: FREE_BALL_SCALE,
                previous: before_possession.unwrap_or(previous_holder),
            });
            self.last_holder = NO_HOLDER;
        } else if self.authority.phase() == MatchPhase::Running {
            let after_possession = self.authority.possession();
            if after_possession != before_possession {
                match after_possession {
                    Some(holder) => {
                        events.push(ServiceEvent::Possession {
                            holder,
                            previous: previous_holder,
                        });
                        self.last_holder = holder;
                    }
                    None => {
                        if previous_holder != NO_HOLDER {
                            events.push(ServiceEvent::FreeBall {
                                position: self.authority.ball_position(),
                                scale: FREE_BALL_SCALE,
                                previous: previous_holder,
                            });
                        }
                        self.last_holder = NO_HOLDER;
                    }
                }
            }
        }
        events.push(self.game_time());
        Ok(events)
    }

    /// Process one decoded client family-76 packet.
    ///
    /// Position and control intents go through the authority's own bounds; a
    /// sub-14 kick/pass state goes through the frozen adapter, and an accepted
    /// kick emits the reconstructed sub-13 impulse. Read-only client packets
    /// (registration, non kick/pass states) are refused with their actual
    /// reason instead of being silently swallowed, and every refusal leaves the
    /// match untouched.
    pub fn handle_client(
        &mut self,
        sender: u64,
        now: f32,
        bytes: &[u8],
        sequence: u64,
        pass_target: Option<u64>,
    ) -> io::Result<Vec<ServiceEvent>> {
        if self.cancelled || !self.sessions.contains_key(&sender) {
            return Err(invalid("soccer sender is not joined"));
        }
        match client_packet(bytes, sequence)? {
            ClientPacket::Intent(intent) => {
                self.authority.apply(sender, now, intent)?;
                Ok(Vec::new())
            }
            ClientPacket::State(request) => {
                let intent = state_request_intent(&request, &self.policy, pass_target)?;
                let (kind, power) = match intent {
                    Intent::Kick { kind, power, .. } => (kind, power),
                    _ => return Err(invalid("soccer state adapter returned a non-kick intent")),
                };
                // Validate and simulate on a probe clone first, so the emitted
                // impulse direction is the frozen authority's own accepted
                // result and every refusal still happens before the live match
                // is touched. An accepted kick always leaves a non-degenerate
                // velocity (power is in `(0, max_kick_power]`).
                let mut probe = self.authority.clone();
                apply_state_request(&mut probe, sender, now, &request, &self.policy, pass_target)?;
                let direction = unit(probe.ball_velocity())?;
                apply_state_request(
                    &mut self.authority,
                    sender,
                    now,
                    &request,
                    &self.policy,
                    pass_target,
                )?;
                self.last_holder = request.player;
                Ok(vec![ServiceEvent::Impulse {
                    player: request.player,
                    kind,
                    vector: [direction[0], direction[1], direction[2], IMPULSE_W],
                    charge: charge_from_aux(request.aux),
                    power,
                    action: match request.action {
                        StateAction::Kick => SHOT_ACTION,
                        StateAction::Pass => PASS_ACTION,
                    },
                    value_c: 0.0,
                }])
            }
            ClientPacket::ReadOnly(SoccerPacket::RegisterPlayer { .. }) => Err(invalid(
                "client registration input is read-only; the roster entry is server-assigned",
            )),
            ClientPacket::ReadOnly(_) => {
                Err(invalid("client state is read-only for the authority"))
            }
        }
    }

    /// Reconnect save seam: an exact clone of the authoritative state.
    pub fn snapshot(&self) -> Authority {
        self.authority.snapshot()
    }

    /// Restore a snapshot from the same configuration. The frozen authority
    /// rules apply: configuration mismatch, corrupt state and stale revisions
    /// are refused, and the receiver's clock domain is preserved.
    pub fn restore(&mut self, snapshot: &Authority) -> io::Result<()> {
        self.authority.restore(snapshot)
    }

    fn game_time(&self) -> ServiceEvent {
        ServiceEvent::GameTime {
            remaining: self.authority.remaining(),
            elapsed: self.authority.elapsed(),
        }
    }
}

/// Unit direction with the same acceptance threshold as `soccer_match`'s own
/// normalize step (`length > 1.0e-6`), evaluated before any mutation.
fn unit(value: [f32; 3]) -> io::Result<[f32; 3]> {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    if !length.is_finite() || length <= 1.0e-6 {
        return Err(invalid("soccer impulse direction has no usable length"));
    }
    Ok([value[0] / length, value[1] / length, value[2] / length])
}

/// Encode a candidate opcode-31 `ClientBeginZoning` frame. The native layout
/// is shared; scene/config values below are R candidates, not client facts.
pub fn entry_scene(position: [f32; 4], rotation: [f32; 4]) -> io::Result<ZoneTransition> {
    let scene = ZoneTransition {
        envelope_flag: ENTRY_FLAG,
        scene: ZONE.to_string(),
        type_id: ENTRY_TYPE,
        position,
        rotation,
        sky: ENTRY_SKY.to_string(),
        tutorial: false,
        unknown: ENTRY_UNKNOWN,
        id: 0,
        update_radius: 0.0,
        geometry_id: ENTRY_GEOMETRY_ID,
        override_update_radius: true,
        wait_for_zone_ready: false,
    };
    // Validate through the registered native codec before the caller stages a
    // zone change: bounds, finite floats and string limits.
    zone_transition::encode(&scene)?;
    Ok(scene)
}

/// Encode a candidate opcode-31 `ClientBeginZoning` frame. The native layout
/// is shared; scene/config values below are R candidates, not client facts.
pub fn entry_frame(position: [f32; 4], rotation: [f32; 4]) -> io::Result<Vec<u8>> {
    zone_transition::encode(&entry_scene(position, rotation)?)
}

/// Rebuild the family-76 gateway envelope around an application body that the
/// gateway already stripped (`u16 76 | u16 sub | payload`). The soccer codec's
/// own `decode` keeps its captured-envelope contract; this is the live route's
/// adapter to it.
pub fn wire_from_inner(inner: &[u8]) -> io::Result<Vec<u8>> {
    if inner.len() < 4 || u16::from_le_bytes([inner[0], inner[1]]) != soccer::TUNNEL_OPCODE {
        return Err(invalid("not a soccer application body"));
    }
    let mut bytes = Vec::with_capacity(inner.len() + 7);
    bytes.extend(5u16.to_le_bytes());
    bytes.push(1);
    bytes.extend(
        u32::try_from(inner.len())
            .map_err(|_| invalid("soccer application body too large"))?
            .to_le_bytes(),
    );
    bytes.extend(inner);
    Ok(bytes)
}

/// Result packet and reward semantics remain open native evidence.
pub const COMPLETION_EVIDENCE: Evidence = Evidence::Missing;
