//! Native soccer entry through the activity portal and the shared minigame lifecycle.
//!
//! Recovered client facts reused here:
//!
//! * The activity portal row is played with `167/1/2` and the minigame records
//!   `42`/`43`/`44` are the three Soccer activities (type 12) from the shipped
//!   `MiniGameData` rows; category 22 and `Soccer_Briarwood.dds` come from the
//!   client `Resources/ActivityCategories.txt`.
//! * Family 76 input layouts and the shared `39/*` lifecycle are owned by the
//!   already-reviewed `soccer`, `soccer_match` and `soccer_service` modules.
//! * The scene name `sg_soccer` and the opcode-31 entry frame are the captured
//!   candidate in `soccer_service`; its spawn, sky, geometry and match values
//!   stay reconstructed (R).
//!
//! Everything in this module is the gateway adapter around those modules:
//! content admission, roster setup, packet wrapping, auto-resume/second-half
//! transitions, and settling the shared result's grant on the caller's live
//! reward state. The original soccer launch/result exchange is not recovered,
//! so the shared lifecycle remains a reconstructed compatibility adapter (R).

use crate::activity_portal::Activity;
use crate::character_store::CharacterState;
use crate::minigame::RunState;
use crate::rewards::{RewardDisplay, RewardGrant, RewardState};
use crate::soccer_match::{FieldBounds, HalfPolicy, MatchConfig, MatchPhase, PhysicsConfig};
use crate::soccer_service::{
    AdapterPolicy, ServiceEvent, SoccerService, entry_scene, wire_from_inner,
};
use crate::zone_transition::ZoneTransition;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::Path,
};

/// Bundled/content filename; loaded from the gameplay directory when present,
/// otherwise from the server's bundled `data/content`.
pub const CONTENT_FILE: &str = "soccer.json";
pub const SCHEMA_VERSION: u32 = 1;
const MAX_FILE: u64 = 1 << 20;
const MAX_GAMES: usize = 8;

/// One published soccer activity row.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Game {
    /// Portal activity id; equal to the shipped MiniGameData row (42/43/44).
    pub id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
    /// R level floor. The original Soccer Star gate is not recovered.
    #[serde(default)]
    pub required_level: u32,
    /// R encounter identity used by the shared `RunMode::Encounter` start.
    pub encounter_id: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BoundsSettings {
    half_x: f32,
    half_z: f32,
    goal_half_width: f32,
    goal_height: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct PhysicsSettings {
    gravity: f32,
    max_position_step: f32,
    max_speed: f32,
    control_radius: f32,
    max_kick_power: f32,
    pass_radius: f32,
    max_tick: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum HalfPolicySetting {
    SwitchSides,
    KeepSides,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdapterSettings {
    max_power: f32,
    charge_floor: f32,
    min_direction_norm: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct MatchSettings {
    bounds: BoundsSettings,
    physics: PhysicsSettings,
    half_seconds: f32,
    kickoff_spawn: [f32; 3],
    half_policy: HalfPolicySetting,
    adapter: AdapterSettings,
    entry_position: [f32; 4],
    entry_rotation: [f32; 4],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RewardSettings {
    coins: u32,
    /// R job XP; empty unless an authored reward references an unlocked profile.
    #[serde(default)]
    job_xp: BTreeMap<i32, u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Content {
    schema_version: u32,
    #[serde(default, rename = "source")]
    _source: String,
    games: Vec<Game>,
    match_config: MatchSettings,
    reward: RewardSettings,
}

impl Content {
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_FILE + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE {
            return Err(invalid("soccer content too large"));
        }
        let content: Self = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        content.validate()?;
        Ok(content)
    }

    pub fn games(&self) -> &[Game] {
        &self.games
    }

    pub fn game(&self, id: u32) -> Option<&Game> {
        self.games.iter().find(|game| game.id == id)
    }

    /// Portal rows for this file; admission keeps its own per-player checks.
    pub fn portal_rows(&self) -> Vec<Activity> {
        self.games
            .iter()
            .map(|game| Activity {
                id: game.id,
                name_id: game.name_id,
                description_id: game.description_id,
                icon_id: game.icon_id,
                difficulty: game.difficulty,
                required_level: game.required_level,
                kind: crate::activity_portal::Kind::Soccer,
            })
            .collect()
    }

    pub fn entry_scene(&self) -> io::Result<ZoneTransition> {
        entry_scene(
            self.match_config.entry_position,
            self.match_config.entry_rotation,
        )
    }

    pub fn config(&self) -> MatchConfig {
        let settings = self.match_config;
        MatchConfig {
            bounds: FieldBounds {
                half_x: settings.bounds.half_x,
                half_z: settings.bounds.half_z,
                goal_half_width: settings.bounds.goal_half_width,
                goal_height: settings.bounds.goal_height,
            },
            physics: PhysicsConfig {
                gravity: settings.physics.gravity,
                max_position_step: settings.physics.max_position_step,
                max_speed: settings.physics.max_speed,
                control_radius: settings.physics.control_radius,
                max_kick_power: settings.physics.max_kick_power,
                pass_radius: settings.physics.pass_radius,
                max_tick: settings.physics.max_tick,
            },
            half_seconds: settings.half_seconds,
            kickoff_spawn: settings.kickoff_spawn,
            half_policy: match settings.half_policy {
                HalfPolicySetting::SwitchSides => HalfPolicy::SwitchSides,
                HalfPolicySetting::KeepSides => HalfPolicy::KeepSides,
            },
        }
    }

    pub fn policy(&self) -> AdapterPolicy {
        AdapterPolicy {
            max_power: self.match_config.adapter.max_power,
            charge_floor: self.match_config.adapter.charge_floor,
            min_direction_norm: self.match_config.adapter.min_direction_norm,
        }
    }

    fn grant(&self, game: &Game, player_guid: u64) -> RewardGrant {
        RewardGrant {
            coins: self.reward.coins,
            items: Vec::new(),
            job_xp: self.reward.job_xp.clone(),
            display: RewardDisplay {
                source_guid: 0,
                player_guid,
                reward_kind: 0,
                icon_id: i32::try_from(game.icon_id).unwrap_or(0),
                name_id: i32::try_from(game.name_id).unwrap_or(0),
                rows: Vec::new(),
            },
        }
    }

    fn validate(&self) -> io::Result<()> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(invalid("soccer content schema version is not supported"));
        }
        if self.games.is_empty() || self.games.len() > MAX_GAMES {
            return Err(invalid("soccer content needs 1..=8 games"));
        }
        for (index, game) in self.games.iter().enumerate() {
            if game.id == 0
                || game.id > i32::MAX as u32
                || game.encounter_id <= 0
                || self.games[..index].iter().any(|other| other.id == game.id)
            {
                return Err(invalid("soccer game ids/encounters are invalid"));
            }
        }
        // The service validates the match configuration and adapter policy.
        SoccerService::new(self.config(), self.policy())?;
        self.entry_scene()?;
        if self.reward.coins > i32::MAX as u32
            || self
                .reward
                .job_xp
                .iter()
                .any(|(profile, xp)| *profile <= 0 || *xp == 0)
        {
            return Err(invalid("soccer reward is invalid"));
        }
        Ok(())
    }
}

/// One adapted match. It is session-only: `GameplayState` intentionally does
/// not persist it, and a reconnect releases the run without a grant.
#[derive(Clone)]
pub struct Session {
    service: SoccerService,
    game: Game,
    human: u64,
    pending: Vec<ServiceEvent>,
    offer: Vec<Vec<u8>>,
    /// The server-owned arena scene this match entered; the gateway stages it
    /// and reports it as the runtime zone while the session lives.
    scene: ZoneTransition,
    sequence: u64,
    last_tick_ms: Option<u64>,
    /// True once the terminal score was settled; a settled match never pays
    /// again, and the flag is session-only like the match itself.
    settled: bool,
    return_zone: u32,
    return_position: [f32; 3],
    return_rotation: [f32; 3],
}

#[derive(Default)]
pub struct LifecycleResult {
    pub replies: Vec<Vec<u8>>,
    pub state: Option<RunState>,
}

/// One tick's staged output. `settlement_due` means full time was reached and
/// the caller must call [`Session::settle`] with its live reward state.
#[derive(Default)]
pub struct TickResult {
    pub replies: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
    pub changed: bool,
    pub finished: bool,
    pub settlement_due: bool,
}

impl Session {
    /// Join the authenticated player, fill the solo roster with passive bots,
    /// and create the shared type-12 offer. The caller stages the scene change
    /// and sends [`Self::loaded_replies`] after the client's Loaded packet.
    /// No reward snapshot is taken here: the payout reads the caller's live
    /// inventory when the match settles.
    pub fn begin(
        content: &Content,
        activity_id: u32,
        character: &CharacterState,
    ) -> io::Result<Self> {
        let game = content
            .game(activity_id)
            .cloned()
            .ok_or_else(|| invalid("soccer activity is not configured"))?;
        let scene = content.entry_scene()?;
        let mut service = SoccerService::new(content.config(), content.policy())?;
        let mut pending = service.join(character.guid, 0, 0)?;
        pending.extend(service.fill_bots_for_solo(character.guid)?);
        let offered = service.offer_minigame(
            character.guid,
            game.id as i32,
            game.encounter_id,
            content.grant(&game, character.guid),
        )?;
        if !offered.accepted {
            return Err(invalid("soccer offer was refused"));
        }
        Ok(Self {
            service,
            game,
            human: character.guid,
            pending,
            offer: offered.replies,
            scene,
            sequence: 0,
            last_tick_ms: None,
            settled: false,
            return_zone: character.zone_id,
            return_position: character.position,
            return_rotation: character.rotation,
        })
    }

    /// Roster, team and 39/16 offer already staged for the client after Loaded.
    pub fn loaded_replies(&mut self) -> Vec<Vec<u8>> {
        let mut replies: Vec<_> = self.pending.drain(..).map(|event| event.inner()).collect();
        replies.append(&mut self.offer);
        replies
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn phase(&self) -> MatchPhase {
        self.service.phase()
    }

    pub fn score(&self) -> [u32; 2] {
        self.service.score()
    }

    pub fn run_state(&self) -> Option<RunState> {
        self.service.run_state(self.human)
    }

    /// The server-owned arena scene the gateway stages on entry.
    pub fn scene(&self) -> &ZoneTransition {
        &self.scene
    }

    pub fn return_location(&self) -> (u32, [f32; 3], [f32; 3]) {
        (self.return_zone, self.return_position, self.return_rotation)
    }

    /// L012: the character back at the pose it left for the match. A match belongs to one
    /// connection, so saves and reconnects use this pose, never the arena scene.
    pub fn return_character(&self, character: &CharacterState) -> CharacterState {
        CharacterState {
            zone_id: self.return_zone,
            position: self.return_position,
            rotation: self.return_rotation,
            ..character.clone()
        }
    }

    /// The shared `39/*`/`41/109`/`26/42` controls. A pre-win quit cancels the
    /// run; a terminal result close retires it without another grant.
    pub fn handle_lifecycle(
        &mut self,
        guid: u64,
        inner: &[u8],
        now_ms: u64,
    ) -> io::Result<LifecycleResult> {
        let mut service = self.service.clone();
        let change = service.handle_minigame_lifecycle(guid, inner)?;
        let mut result = LifecycleResult {
            replies: change.run.replies,
            state: change.run.state,
        };
        result
            .replies
            .extend(change.soccer_events.iter().map(ServiceEvent::inner));
        if result.state == Some(RunState::Starting) {
            let started = service.mark_started(guid)?;
            result
                .replies
                .extend(started.iter().map(ServiceEvent::inner));
            result.state = service.run_state(guid);
            if result.state == Some(RunState::Playing) {
                self.last_tick_ms = Some(now_ms);
            }
        }
        self.service = service;
        Ok(result)
    }

    /// One family-76 application body. `pass_target` is deliberately absent:
    /// the client payload does not establish one, so pass states stay refused
    /// until a server-verified target source exists.
    pub fn handle_client(
        &mut self,
        guid: u64,
        inner: &[u8],
        now_ms: u64,
    ) -> io::Result<Vec<Vec<u8>>> {
        let wire = wire_from_inner(inner)?;
        let sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| invalid("soccer input sequence exhausted"))?;
        let mut service = self.service.clone();
        let events = service.handle_client(guid, now_ms as f32 / 1000.0, &wire, sequence, None)?;
        self.service = service;
        self.sequence = sequence;
        Ok(events.iter().map(ServiceEvent::inner).collect())
    }

    /// Advance the authoritative clock. Restart and interval transitions are
    /// automatic (R): the client has no recovered resume packet, and the shared
    /// service exposes those transitions only as server calls. Full time only
    /// reports `settlement_due`; the caller settles on its live reward state via
    /// [`Self::settle`] so no entry snapshot can replace later changes.
    pub fn tick(&mut self, now_ms: u64) -> io::Result<TickResult> {
        let mut result = TickResult::default();
        let mut service = self.service.clone();
        if service.cancelled() {
            return Ok(result);
        }
        let mut last_tick_ms = self.last_tick_ms;
        if service.phase() == MatchPhase::Running {
            let previous = self.last_tick_ms.unwrap_or(now_ms);
            let elapsed = now_ms.saturating_sub(previous).min(250);
            last_tick_ms = Some(now_ms);
            if elapsed > 0 {
                let events = service.tick(elapsed as f32 / 1000.0)?;
                result
                    .replies
                    .extend(events.iter().map(ServiceEvent::inner));
                result.changed |= !events.is_empty();
            }
        }
        match service.phase() {
            MatchPhase::Restart => {
                let events = service.start()?;
                result
                    .replies
                    .extend(events.iter().map(ServiceEvent::inner));
                result.changed |= !events.is_empty();
            }
            MatchPhase::HalfTime => {
                let events = service.second_half()?;
                result
                    .replies
                    .extend(events.iter().map(ServiceEvent::inner));
                result.changed |= !events.is_empty();
            }
            MatchPhase::Finished if !self.settled => {
                result.settlement_due = true;
            }
            _ => {}
        }
        self.service = service;
        self.last_tick_ms = last_tick_ms;
        Ok(result)
    }

    /// Settle the terminal score on the caller's live inventory and profiles.
    ///
    /// The win grant is applied on top of those live values, never to a
    /// snapshot captured at entry: a claim or trade made during the match stays
    /// in place. A loss pays nothing; a duplicate call is inert.
    pub fn settle(&mut self, guid: u64, live: &RewardState) -> io::Result<TickResult> {
        let mut result = TickResult::default();
        let mut service = self.service.clone();
        let outcome = service.finish_minigame(guid, live)?;
        if outcome.accepted {
            self.settled = true;
            result.replies.extend(outcome.replies);
            result.replies.extend(outcome.reward_packets);
            result.reward_state = outcome.reward_state;
            result.changed = true;
            result.finished = true;
        }
        self.service = service;
        Ok(result)
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
