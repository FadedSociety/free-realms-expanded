//! Server-owned soccer match authority for the MMO005 restoration.
//!
//! Evidence split:
//!
//! - **Client facts.** The native map supports the sub-14 body, state-change
//!   counter, aux charge and vecA direction semantics, plus the sub-16 controls
//!   body. It does not establish authoritative match rules.
//! - **Reconstructed policy (R).** Team capacity, clock, bounds, motion,
//!   control/pass radii, kick-power scale, goal crossing, restart, kickoff and
//!   second-half side changes are configurable server behavior.
//! - **Open evidence.** The native match completion/reward route and the
//!   authoritative meaning of sub-13 output remain unresolved.
//!
//! The client can never assert a goal: it submits bounded position, control
//! and kick/pass intents, and the authority owns the ball, possession, the
//! clock and the score. This is a small deterministic slice - no terrain
//! collision, no native acceptance, no physical authority claim.
//!
//! Reconstructed server policy is explicit in the configuration and comments.
//! [`Authority::apply`] binds requests to an authenticated sender and server
//! time, limits movement, and keeps the ball, possession and score server-owned.
//! The native state counter is never used for transport replay protection.

use std::collections::BTreeMap;
use std::io;

use crate::soccer::{Evidence, SoccerPacket, decode};

/// R: second-half attack-side policy. No client rule was recovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HalfPolicy {
    /// Teams swap ends at the interval (reconstructed default).
    SwitchSides,
    /// Teams keep their first-half ends for the whole match.
    KeepSides,
}

/// R: four slots per team for local solo fill. The client packet shape exposes
/// a slot field but does not establish this capacity.
pub const TEAM_SLOT_COUNT: u32 = 4;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn finite3(value: [f32; 3]) -> bool {
    value.iter().all(|v| v.is_finite())
}

fn length(value: [f32; 3]) -> f32 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    length([a[0] - b[0], a[1] - b[1], a[2] - b[2]])
}

fn normalize(value: [f32; 3]) -> io::Result<[f32; 3]> {
    let len = length(value);
    if !len.is_finite() || len <= 1e-6 {
        return Err(invalid("soccer direction has no usable length"));
    }
    Ok([value[0] / len, value[1] / len, value[2] / len])
}

/// Caller-supplied pitch bounds (R); no dimensions are recovered from the
/// client. Team 0 attacks `+z`, team 1 `-z` (R).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldBounds {
    pub half_x: f32,
    pub half_z: f32,
    pub goal_half_width: f32,
    pub goal_height: f32,
}

impl FieldBounds {
    pub fn validate(&self) -> io::Result<()> {
        if !finite3([self.half_x, self.half_z, self.goal_half_width])
            || !self.goal_height.is_finite()
        {
            return Err(invalid("soccer field bounds must be finite"));
        }
        if self.half_x <= 0.0 || self.half_z <= 0.0 {
            return Err(invalid("soccer field half extents must be positive"));
        }
        if self.goal_half_width <= 0.0 || self.goal_height <= 0.0 {
            return Err(invalid("soccer goal bounds must be positive"));
        }
        if self.goal_half_width > self.half_x {
            return Err(invalid("soccer goal is wider than the field"));
        }
        Ok(())
    }

    fn inside(&self, position: [f32; 3]) -> bool {
        position[0].abs() <= self.half_x && position[2].abs() <= self.half_z && position[1] >= 0.0
    }
}

/// Reconstructed tuning values; overridable by the caller. None of these is a
/// historical constant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsConfig {
    /// Downward acceleration; 0.0 disables it.
    pub gravity: f32,
    /// Largest accepted player displacement for one position intent.
    pub max_position_step: f32,
    /// Player speed cap used by the server-time movement budget (units/s).
    pub max_speed: f32,
    /// Distance within which a player may hold or receive the ball.
    pub control_radius: f32,
    /// Largest accepted kick power.
    pub max_kick_power: f32,
    /// Largest accepted kicker-to-teammate distance for a pass.
    pub pass_radius: f32,
    /// Largest accepted simulation step for one `advance` call.
    pub max_tick: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            max_position_step: 10.0,
            max_speed: 40.0,
            control_radius: 2.0,
            max_kick_power: 30.0,
            pass_radius: 40.0,
            max_tick: 0.25,
        }
    }
}

/// Caller-supplied configuration for one match.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatchConfig {
    pub bounds: FieldBounds,
    pub physics: PhysicsConfig,
    /// R: configurable half length. `soccer::HALF_SECONDS` is only a fallback.
    pub half_seconds: f32,
    pub kickoff_spawn: [f32; 3],
    /// Caller-owned second-half side policy.
    pub half_policy: HalfPolicy,
}

impl MatchConfig {
    /// Defaults: reconstructed physics, an R 180-second fallback, and an R
    /// origin kickoff. Geometry and spawn supplied by callers are also R until
    /// client evidence establishes otherwise.
    pub fn new(bounds: FieldBounds) -> Self {
        Self {
            bounds,
            physics: PhysicsConfig::default(),
            half_seconds: crate::soccer::HALF_SECONDS,
            kickoff_spawn: [0.0, 0.0, 0.0],
            half_policy: HalfPolicy::SwitchSides,
        }
    }

    pub fn validate(&self) -> io::Result<()> {
        self.bounds.validate()?;
        if !self.half_seconds.is_finite() || self.half_seconds <= 0.0 {
            return Err(invalid("soccer half length must be positive"));
        }
        let p = self.physics;
        if !p.gravity.is_finite() || p.gravity < 0.0 {
            return Err(invalid("soccer gravity must be finite and non-negative"));
        }
        for value in [
            p.max_position_step,
            p.max_speed,
            p.control_radius,
            p.max_kick_power,
            p.pass_radius,
            p.max_tick,
        ] {
            if !value.is_finite() || value <= 0.0 {
                return Err(invalid("soccer physics values must be finite and positive"));
            }
        }
        if !finite3(self.kickoff_spawn) || !self.bounds.inside(self.kickoff_spawn) {
            return Err(invalid(
                "soccer kickoff spawn must be finite and inside the field",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KickKind {
    Shot,
    Pass,
}

/// Bounded player intent. The client never sends a goal or a ball transform;
/// those packet ids are refused by [`client_intent`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intent {
    Position {
        player: u64,
        sequence: u64,
        position: [f32; 3],
    },
    Controls {
        player: u64,
        sequence: u64,
        /// Two advisory direction components (rev 3); never kick or pass.
        direction: [f32; 2],
        /// Raw control bits preserved verbatim; no bit is named by the client.
        controls: u32,
    },
    Kick {
        player: u64,
        sequence: u64,
        kind: KickKind,
        /// Validated for finiteness; a `Pass` derives its direction from the
        /// verified target position instead.
        direction: [f32; 3],
        power: f32,
        /// Pass target; must be a registered teammate for `Pass`.
        target: Option<u64>,
    },
}

impl Intent {
    pub fn player(&self) -> u64 {
        match self {
            Intent::Position { player, .. }
            | Intent::Controls { player, .. }
            | Intent::Kick { player, .. } => *player,
        }
    }

    pub fn sequence(&self) -> u64 {
        match self {
            Intent::Position { sequence, .. }
            | Intent::Controls { sequence, .. }
            | Intent::Kick { sequence, .. } => *sequence,
        }
    }
}

/// State classification used by this service is R: the native map establishes
/// the state byte field but not the full kick/pass state table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateAction {
    Kick,
    Pass,
}

/// A typed sub-14 kick/pass request. vecA is the client's action direction,
/// aux is quantized charge, and the u32 is a state-change counter rather than
/// a transport sequence. The server still validates and simulates the action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateRequest {
    pub player: u64,
    pub sequence: u64,
    pub state: u8,
    pub aux: u8,
    /// Incremented on client state changes (`player+0xB0`); not a replay clock.
    pub state_counter: u32,
    pub action: StateAction,
    /// vecA, wire offset 0x0E: action direction.
    pub action_direction: [f32; 4],
    /// vecB, wire offset 0x1E: steering/movement input; never ball authority.
    pub steering_vector: [f32; 4],
    /// vecC, wire offset 0x2E: position-like input; never ball authority.
    pub position_vector: [f32; 4],
}

/// R candidate allowlist: 16-26 and 31-34 are treated as kicks; 27-30 as passes.
pub fn classify_state(state: u8) -> Option<StateAction> {
    match state {
        16..=26 | 31..=34 => Some(StateAction::Kick),
        27..=30 => Some(StateAction::Pass),
        _ => None,
    }
}

/// One decoded client packet candidate.
#[derive(Clone, Debug, PartialEq)]
pub enum ClientPacket {
    /// Ids 1 and 16: an intent the authority can bound and apply.
    Intent(Intent),
    /// Id 14 with a named kick/pass state: bounded client input, never a
    /// client-authored ball transform.
    State(StateRequest),
    /// Ids 3 and 14 (non kick/pass states): decoded read-only by this authority.
    ReadOnly(SoccerPacket),
}

/// Decode one client packet using the mapped layouts available to this codec.
///
/// The authority accepts the input classes it has a validated path for. Goal
/// and ball-state packets cannot enter as client authority. Sub 13's layout is
/// known, but its emission semantics are not. `sequence` comes from the
/// reliable gateway, not from the client's state counter.
pub fn client_packet(bytes: &[u8], sequence: u64) -> io::Result<ClientPacket> {
    let message = decode(bytes)?;
    match &message.packet {
        SoccerPacket::UpdatePlayerPos {
            player, position, ..
        } => Ok(ClientPacket::Intent(Intent::Position {
            player: *player,
            sequence,
            position: *position,
        })),
        SoccerPacket::UpdatePlayerControls {
            player,
            direction,
            controls,
        } => Ok(ClientPacket::Intent(Intent::Controls {
            player: *player,
            sequence,
            direction: *direction,
            controls: *controls,
        })),
        SoccerPacket::UpdatePlayerState {
            player,
            state,
            aux,
            state_counter,
            vectors,
        } => match classify_state(*state) {
            Some(action) => Ok(ClientPacket::State(StateRequest {
                player: *player,
                sequence,
                state: *state,
                aux: *aux,
                state_counter: *state_counter,
                action,
                action_direction: vectors[0],
                steering_vector: vectors[1],
                position_vector: vectors[2],
            })),
            None => Ok(ClientPacket::ReadOnly(message.packet.clone())),
        },
        SoccerPacket::RegisterPlayer { .. } => Ok(ClientPacket::ReadOnly(message.packet.clone())),
        SoccerPacket::ImpulseBall { .. } => Err(invalid(
            "soccer impulse 13 is server-authored in the located evidence; no client author path was located",
        )),
        SoccerPacket::GoalMade { .. } => Err(invalid("client may not assert a soccer goal")),
        _ => Err(invalid(
            "packet id is not a supported soccer client request",
        )),
    }
}

/// Intent-only view of [`client_packet`]. A kick/pass still needs the service
/// charge policy and a server-verified pass target.
pub fn client_intent(bytes: &[u8], sequence: u64) -> io::Result<Intent> {
    match client_packet(bytes, sequence)? {
        ClientPacket::Intent(intent) => Ok(intent),
        ClientPacket::State(_) => Err(invalid(
            "soccer kick/pass state needs configured charge policy and a server-verified pass target",
        )),
        ClientPacket::ReadOnly(SoccerPacket::RegisterPlayer { .. }) => Err(invalid(
            "client registration is read-only for the authority",
        )),
        ClientPacket::ReadOnly(_) => Err(invalid(
            "client player state is read-only for the authority",
        )),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchPhase {
    Kickoff,
    Running,
    /// R: clock hold after a goal; only `kickoff` resumes play.
    Restart,
    HalfTime,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerState {
    pub team: u32,
    pub slot: u32,
    /// Last verified position; `None` until the first bounded position intent.
    pub position: Option<[f32; 3]>,
    /// Authoritative server time when `position` was last accepted. Kept in
    /// f64 so a clock-domain rebase preserves elapsed allowances exactly.
    pub position_at: f64,
    pub sequence: u64,
}

/// Once-only R outcome. `reward` stays [`Evidence::Missing`] because the native
/// client does not establish a soccer reward table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outcome {
    pub score: [u32; 2],
    pub winner: Option<u32>,
    pub half_seconds: f32,
    pub half_seconds_evidence: Evidence,
    pub reward: Evidence,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Authority {
    config: MatchConfig,
    players: BTreeMap<u64, PlayerState>,
    ball_position: [f32; 3],
    ball_velocity: [f32; 3],
    possession: Option<u64>,
    live_pass: Option<u64>,
    last_touch: Option<u64>,
    /// False while the last toucher may not immediately re-take the ball.
    toucher_clear: bool,
    score: [u32; 2],
    phase: MatchPhase,
    half: u8,
    remaining: f32,
    elapsed: f32,
    /// Authoritative server clock (f64 for exact rebases); only `advance` and
    /// `restore` move it.
    server_time: f64,
    revision: u64,
    outcome: Option<Outcome>,
}

impl Authority {
    pub fn new(config: MatchConfig) -> io::Result<Self> {
        config.validate()?;
        Ok(Self {
            ball_position: config.kickoff_spawn,
            config,
            players: BTreeMap::new(),
            ball_velocity: [0.0; 3],
            possession: None,
            live_pass: None,
            last_touch: None,
            toucher_clear: true,
            score: [0, 0],
            phase: MatchPhase::Kickoff,
            half: 1,
            remaining: config.half_seconds,
            elapsed: 0.0,
            server_time: 0.0,
            revision: 0,
            outcome: None,
        })
    }

    pub fn config(&self) -> &MatchConfig {
        &self.config
    }

    pub fn phase(&self) -> MatchPhase {
        self.phase
    }

    pub fn half(&self) -> u8 {
        self.half
    }

    pub fn score(&self) -> [u32; 2] {
        self.score
    }

    pub fn remaining(&self) -> f32 {
        self.remaining
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Authoritative monotonic server time in seconds.
    pub fn server_time(&self) -> f32 {
        self.server_time as f32
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn ball_position(&self) -> [f32; 3] {
        self.ball_position
    }

    pub fn ball_velocity(&self) -> [f32; 3] {
        self.ball_velocity
    }

    pub fn possession(&self) -> Option<u64> {
        self.possession
    }

    pub fn last_touch(&self) -> Option<u64> {
        self.last_touch
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    pub fn player(&self, player: u64) -> Option<&PlayerState> {
        self.players.get(&player)
    }

    /// Server-side roster assignment. Client bytes can never reach this.
    pub fn register(&mut self, player: u64, team: u32, slot: u32) -> io::Result<()> {
        if team > 1 || slot >= TEAM_SLOT_COUNT {
            return Err(invalid("soccer team slot out of range"));
        }
        if self.players.contains_key(&player) {
            return Err(invalid("soccer player already registered"));
        }
        if self
            .players
            .values()
            .any(|state| state.team == team && state.slot == slot)
        {
            return Err(invalid("soccer team slot already taken"));
        }
        self.players.insert(
            player,
            PlayerState {
                team,
                slot,
                position: None,
                position_at: 0.0,
                sequence: 0,
            },
        );
        self.revision += 1;
        Ok(())
    }

    /// Reset the ball and start play. Valid from `Kickoff` and `Restart`.
    pub fn kickoff(&mut self) -> io::Result<()> {
        if !matches!(self.phase, MatchPhase::Kickoff | MatchPhase::Restart) {
            return Err(invalid("soccer kickoff is not available in this phase"));
        }
        self.reset_ball();
        self.phase = MatchPhase::Running;
        self.revision += 1;
        Ok(())
    }

    /// Remove one player (session departure). Server-side only; client bytes
    /// can never reach this. The roster entry is dropped, any possession held
    /// by the player is released in place and any live pass naming them as the
    /// target is cleared, so a left session can never keep the ball or be
    /// handed one. Unknown players are refused before any mutation.
    pub fn unregister(&mut self, player: u64) -> io::Result<()> {
        if self.players.remove(&player).is_none() {
            return Err(invalid("soccer player is not registered"));
        }
        if self.possession == Some(player) {
            self.possession = None;
        }
        if self.live_pass == Some(player) {
            self.live_pass = None;
        }
        if self.last_touch == Some(player) {
            self.last_touch = None;
            self.toucher_clear = true;
        }
        self.revision += 1;
        Ok(())
    }

    /// Start the second half from the interval. Caller-supplied phase gate.
    pub fn start_second_half(&mut self) -> io::Result<()> {
        if self.phase != MatchPhase::HalfTime || self.half != 1 {
            return Err(invalid("soccer second half is not available in this phase"));
        }
        self.half = 2;
        self.remaining = self.config.half_seconds;
        self.reset_ball();
        self.phase = MatchPhase::Running;
        self.revision += 1;
        Ok(())
    }

    /// Apply one bounded client intent.
    ///
    /// `sender` is the authenticated session guid and `now` the authoritative
    /// monotonic server time in seconds; both are gateway inputs, never
    /// payload fields. A sender that does not match the payload player is
    /// refused before any player state is read or written, so a spoofed packet
    /// cannot touch another player's sequence or position. Rejection leaves
    /// the match untouched.
    ///
    /// Trusted-clock contract: `now` must come from the gateway's clock. It is
    /// clamped to the authority's own `server_time`, which only [`advance`]
    /// moves, so intents can never advance the clock, a forward spike grants
    /// no extra movement budget and poisons no later timestamp, and a lagging
    /// timestamp simply grants less budget. Client packets never carry time.
    pub fn apply(&mut self, sender: u64, now: f32, intent: Intent) -> io::Result<()> {
        if self.phase != MatchPhase::Running {
            return Err(invalid("soccer intents are only accepted while running"));
        }
        if sender != intent.player() {
            return Err(invalid(
                "soccer intent sender does not match the payload player",
            ));
        }
        if !now.is_finite() || now < 0.0 {
            return Err(invalid("invalid soccer intent time"));
        }
        let now = now as f64;
        let trusted_now = if now > self.server_time {
            self.server_time
        } else {
            now
        };
        let player = intent.player();
        let sequence = intent.sequence();
        let state = self
            .players
            .get(&player)
            .ok_or_else(|| invalid("soccer intent from an unregistered player"))?;
        if sequence <= state.sequence {
            return Err(invalid("stale soccer intent"));
        }

        match intent {
            Intent::Position {
                player: _,
                sequence: _,
                position,
            } => {
                if !finite3(position) {
                    return Err(invalid("non-finite soccer position"));
                }
                if !self.config.bounds.inside(position) {
                    return Err(invalid("soccer position outside the field"));
                }
                let (previous, previous_at) = match self.players.get(&player) {
                    Some(state) => (state.position, state.position_at),
                    None => (None, 0.0),
                };
                if let Some(previous) = previous {
                    let step = distance(previous, position);
                    if step > self.config.physics.max_position_step {
                        return Err(invalid("soccer position step too large"));
                    }
                    // Movement also costs authoritative server time: the
                    // budget is max_speed * elapsed on the server clock, with
                    // no epsilon, so equal-timestamp spam cannot accumulate
                    // movement and any nonzero step at dt 0 is refused.
                    let budget =
                        (self.config.physics.max_speed as f64) * (trusted_now - previous_at);
                    if (step as f64) > budget {
                        return Err(invalid("soccer position exceeds the server time budget"));
                    }
                } else if distance(self.config.kickoff_spawn, position)
                    > self.config.physics.max_position_step
                {
                    // A first position is bounded from the kickoff spawn too.
                    return Err(invalid("soccer position step too large"));
                }
                let state = self.players.get_mut(&player).expect("checked above");
                state.position = Some(position);
                state.position_at = trusted_now;
                state.sequence = sequence;
            }
            Intent::Controls {
                player: _,
                sequence: _,
                direction,
                controls: _,
            } => {
                if !direction.iter().all(|value| value.is_finite()) {
                    return Err(invalid("non-finite soccer control direction"));
                }
                // Controls are recorded for ordering only; the raw bits stay
                // advisory input and are never decoded as kick/pass.
                self.players
                    .get_mut(&player)
                    .expect("checked above")
                    .sequence = sequence;
            }
            Intent::Kick {
                player: _,
                sequence: _,
                kind,
                direction,
                power,
                target,
            } => {
                if !finite3(direction) {
                    return Err(invalid("non-finite soccer kick direction"));
                }
                if !power.is_finite() || power <= 0.0 || power > self.config.physics.max_kick_power
                {
                    return Err(invalid("soccer kick power out of range"));
                }
                if self.possession != Some(player) {
                    return Err(invalid("soccer kick without possession"));
                }
                let kicker = self.players[&player];
                let kicker_position = kicker
                    .position
                    .ok_or_else(|| invalid("soccer kicker has no verified position"))?;
                if distance(kicker_position, self.ball_position)
                    > self.config.physics.control_radius
                {
                    return Err(invalid("soccer kicker is not near the ball"));
                }
                match kind {
                    KickKind::Shot => {
                        self.ball_velocity = normalize(direction)?;
                        self.ball_velocity = [
                            self.ball_velocity[0] * power,
                            self.ball_velocity[1] * power,
                            self.ball_velocity[2] * power,
                        ];
                        self.live_pass = None;
                    }
                    KickKind::Pass => {
                        let target =
                            target.ok_or_else(|| invalid("soccer pass without a target"))?;
                        if target == player {
                            return Err(invalid("soccer pass target is the kicker"));
                        }
                        let target_state = self
                            .players
                            .get(&target)
                            .ok_or_else(|| invalid("soccer pass target is unregistered"))?;
                        if target_state.team != kicker.team {
                            return Err(invalid("soccer pass target is not a teammate"));
                        }
                        let target_position = target_state.position.ok_or_else(|| {
                            invalid("soccer pass target has no verified position")
                        })?;
                        if distance(self.ball_position, target_position)
                            > self.config.physics.pass_radius
                        {
                            return Err(invalid("soccer pass target out of range"));
                        }
                        let toward = normalize([
                            target_position[0] - self.ball_position[0],
                            target_position[1] - self.ball_position[1],
                            target_position[2] - self.ball_position[2],
                        ])?;
                        self.ball_velocity =
                            [toward[0] * power, toward[1] * power, toward[2] * power];
                        self.live_pass = Some(target);
                    }
                }
                self.possession = None;
                self.last_touch = Some(player);
                self.toucher_clear = false;
                self.players
                    .get_mut(&player)
                    .expect("checked above")
                    .sequence = sequence;
            }
        }
        self.revision += 1;
        Ok(())
    }

    /// Advance the deterministic simulation by `dt` seconds. The equations are
    /// reconstructed, not recorded: position += velocity * dt, then
    /// velocity.y -= gravity * dt. The caller supplies the step and is bounded
    /// by `max_tick`.
    pub fn advance(&mut self, dt: f32) -> io::Result<()> {
        if self.phase != MatchPhase::Running {
            return Err(invalid("soccer advance requires a running match"));
        }
        if !dt.is_finite() || dt <= 0.0 || dt > self.config.physics.max_tick {
            return Err(invalid("soccer advance step out of range"));
        }

        // Match clock first: a half cannot end with a ball still in flight.
        self.remaining -= dt;
        self.elapsed += dt;
        self.server_time += dt as f64;
        if self.remaining <= 0.0 {
            self.remaining = 0.0;
            self.ball_velocity = [0.0; 3];
            self.possession = None;
            self.live_pass = None;
            if self.half == 1 {
                self.phase = MatchPhase::HalfTime;
            } else {
                self.phase = MatchPhase::Finished;
                self.outcome = Some(self.build_outcome());
            }
            self.revision += 1;
            return Ok(());
        }

        if let Some(holder) = self.possession {
            if let Some(position) = self.players.get(&holder).and_then(|state| state.position) {
                self.ball_position = position;
                self.ball_velocity = [0.0; 3];
                self.revision += 1;
                return Ok(());
            }
        }

        let previous = self.ball_position;
        let step = [
            self.ball_velocity[0] * dt,
            self.ball_velocity[1] * dt,
            self.ball_velocity[2] * dt,
        ];
        let mut next = [
            previous[0] + step[0],
            previous[1] + step[1],
            previous[2] + step[2],
        ];
        if self.config.physics.gravity > 0.0 {
            self.ball_velocity[1] -= self.config.physics.gravity * dt;
        }
        if next[1] < 0.0 {
            next[1] = 0.0;
            self.ball_velocity[1] = 0.0;
        }
        self.ball_position = next;

        if !self.toucher_clear {
            let clear = match self
                .last_touch
                .and_then(|player| self.players.get(&player))
                .and_then(|state| state.position)
            {
                Some(position) => distance(position, next) > self.config.physics.control_radius,
                None => true,
            };
            if clear {
                self.toucher_clear = true;
            }
        }

        if let Some(team) = self.goal_crossing(previous, next) {
            self.score[team as usize] += 1;
            self.phase = MatchPhase::Restart;
            self.ball_velocity = [0.0; 3];
            self.possession = None;
            self.live_pass = None;
            self.revision += 1;
            return Ok(());
        }

        if !self.config.bounds.inside(next) {
            // R out-of-play restart:
            // the ball returns to the kickoff spawn and play resumes through
            // `kickoff()`, so a miss cannot strand the match.
            self.reset_ball();
            self.phase = MatchPhase::Restart;
            self.revision += 1;
            return Ok(());
        }

        if let Some(target) = self.live_pass {
            if let Some(position) = self.players.get(&target).and_then(|state| state.position) {
                if distance(position, next) <= self.config.physics.control_radius {
                    self.possession = Some(target);
                    self.ball_position = position;
                    self.ball_velocity = [0.0; 3];
                    self.live_pass = None;
                    self.revision += 1;
                    return Ok(());
                }
            }
        } else if self.possession.is_none() && self.toucher_clear {
            let control_radius = self.config.physics.control_radius;
            let winner = self
                .players
                .iter()
                .filter_map(|(player, state)| {
                    let position = state.position?;
                    let distance = distance(position, next);
                    (distance <= control_radius).then_some((*player, position, distance))
                })
                .min_by(|a, b| a.2.total_cmp(&b.2));
            if let Some((player, position, _)) = winner {
                self.possession = Some(player);
                self.ball_position = position;
                self.ball_velocity = [0.0; 3];
            }
        }

        self.revision += 1;
        Ok(())
    }

    /// Reconnect save: an exact clone of the authority state.
    pub fn snapshot(&self) -> Authority {
        self.clone()
    }

    /// Restore a snapshot from the same configuration. Older revisions are
    /// rejected, so a stale save cannot roll the match back.
    ///
    /// Clock domains: the receiver's `server_time` is kept and the snapshot's
    /// movement clock is rebased onto it (`position_at` shifts by the same
    /// delta), so every player's elapsed movement allowance survives a
    /// reconnect or a process restart whose monotonic base differs. The match
    /// clock (`remaining`/`elapsed`) and `revision` are restored verbatim.
    ///
    /// Corrupt snapshots are not constructible through this API today (the
    /// fields are private and there is no deserialization path); the finiteness
    /// and clock-monotonicity checks below are fail-closed defense for any
    /// future save format.
    pub fn restore(&mut self, snapshot: &Authority) -> io::Result<()> {
        if snapshot.config != self.config {
            return Err(invalid("soccer snapshot configuration mismatch"));
        }
        snapshot.validate_snapshot()?;
        if snapshot.revision < self.revision {
            return Err(invalid("stale soccer snapshot"));
        }
        let delta = self.server_time - snapshot.server_time;
        *self = snapshot.clone();
        self.server_time = snapshot.server_time + delta;
        for state in self.players.values_mut() {
            state.position_at += delta;
        }
        Ok(())
    }

    /// Fail-closed snapshot validation. Corrupt snapshots are not reachable
    /// through the public API; these checks exist for a future save format and
    /// reject non-finite or non-monotonic clock state before adoption.
    fn validate_snapshot(&self) -> io::Result<()> {
        if !self.server_time.is_finite() || self.server_time < 0.0 {
            return Err(invalid("soccer snapshot clock is not finite"));
        }
        if !self.remaining.is_finite() || self.remaining < 0.0 {
            return Err(invalid("soccer snapshot match clock is not finite"));
        }
        if !self.elapsed.is_finite() || self.elapsed < 0.0 {
            return Err(invalid("soccer snapshot match elapsed is not finite"));
        }
        if !finite3(self.ball_position) || !finite3(self.ball_velocity) {
            return Err(invalid("soccer snapshot ball state is not finite"));
        }
        for state in self.players.values() {
            if !state.position_at.is_finite() || state.position_at > self.server_time {
                return Err(invalid("soccer snapshot player clock is not monotonic"));
            }
            if let Some(position) = state.position {
                if !finite3(position) {
                    return Err(invalid("soccer snapshot player position is not finite"));
                }
            }
        }
        Ok(())
    }

    fn reset_ball(&mut self) {
        self.ball_position = self.config.kickoff_spawn;
        self.ball_velocity = [0.0; 3];
        self.possession = None;
        self.live_pass = None;
        self.last_touch = None;
        self.toucher_clear = true;
    }

    fn build_outcome(&self) -> Outcome {
        let winner = if self.score[0] > self.score[1] {
            Some(0)
        } else if self.score[1] > self.score[0] {
            Some(1)
        } else {
            None
        };
        Outcome {
            score: self.score,
            winner,
            half_seconds: self.config.half_seconds,
            half_seconds_evidence: Evidence::Reconstructed,
            reward: Evidence::Missing,
            revision: self.revision,
        }
    }

    /// Attack direction sign for a team in the current half. First-half
    /// convention: team 0 attacks `+z`, team 1 `-z` (reconstructed). The
    /// caller-owned [`HalfPolicy`] decides whether the sides swap at the
    /// interval; the side-switch behavior remains R policy.
    fn attack_sign(&self, team: u32) -> f32 {
        let first_half = if team == 0 { 1.0 } else { -1.0 };
        let switched = self.config.half_policy == HalfPolicy::SwitchSides && self.half == 2;
        if switched { -first_half } else { first_half }
    }

    /// Deterministic goal test: a signed crossing of the team's attack goal
    /// line inside the caller-supplied mouth.
    fn goal_crossing(&self, previous: [f32; 3], next: [f32; 3]) -> Option<u32> {
        for team in 0..2u32 {
            let sign = self.attack_sign(team);
            let line = sign * self.config.bounds.half_z;
            let crossed = if sign > 0.0 {
                previous[2] <= line && next[2] > line
            } else {
                previous[2] >= line && next[2] < line
            };
            if !crossed {
                continue;
            }
            let t = (line - previous[2]) / (next[2] - previous[2]);
            let x = previous[0] + (next[0] - previous[0]) * t;
            let y = previous[1] + (next[1] - previous[1]) * t;
            if x.abs() <= self.config.bounds.goal_half_width
                && y >= 0.0
                && y <= self.config.bounds.goal_height
            {
                return Some(team);
            }
        }
        None
    }
}
