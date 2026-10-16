//! Harvesting adapter for the shared minigame run lifecycle.
//!
//! Type-8 owns the recovered Farming movie state machine. This adapter owns
//! ordinary offer/start/result/leave/close state and stages one winning grant.

use osfr_tools::{
    harvesting::{CHEF_PROFILE_ID, HARVESTING_ICON_ID},
    minigame::{Activity, ActivityFinish, MinigameRuns, RunMode, RunOutcome, RunSpec, RunState},
    mining::{Type8Engine, Type8Phase, Type8State, Type8Transaction},
    quest_engine::QuestEvent,
    rewards::{ItemGrant, RewardDisplay, RewardGrant, RewardState},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io, path::Path};

/// (R) Original Harvesting XP has not been recovered; use a small local award.
pub const RECONSTRUCTED_CHEF_XP_PER_WIN: u32 = 25;

const HARVESTING_TYPE_ID: i32 = 8;
const MAX_REWARD_ITEMS: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarvestingLifecycleSnapshot {
    version: u8,
    game_id: i32,
    pub activity: Type8State,
    pub runs: MinigameRuns,
    player_guid: u64,
}

/// Staged change. The caller persists activity, run, reward and quest state
/// together before sending either reply list.
#[derive(Default)]
pub struct HarvestingActivityOutcome {
    pub accepted: bool,
    pub changed: bool,
    pub state: Option<RunState>,
    pub replies: Vec<Vec<u8>>,
    pub reward_packets: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
    pub quest_events: Vec<QuestEvent>,
}

pub struct HarvestingLifecycle {
    engine: Type8Engine,
    runs: MinigameRuns,
    game_id: i32,
    player_guid: u64,
}

struct HarvestingActivity<'a>(&'a Type8Engine);

impl Activity for HarvestingActivity<'_> {
    fn score(&self) -> u32 {
        self.0.state().active.as_ref().map_or(0, |run| run.score)
    }

    fn finish(&self) -> Option<ActivityFinish> {
        let run = self.0.state().active.as_ref()?;
        let won = match run.phase {
            Type8Phase::Won => true,
            Type8Phase::Lost => false,
            _ => return None,
        };
        let elapsed_seconds = self
            .0
            .spec()
            .duration_seconds
            .saturating_mul(1000)
            .saturating_sub(run.remaining_ms)
            / 1000;
        Some(ActivityFinish {
            won,
            // (R) Harvesting's original star calculation was not recovered.
            stars: 0,
            elapsed_seconds,
        })
    }
}

impl HarvestingLifecycle {
    pub fn load(
        directory: &Path,
        content_file: &str,
        game_id: i32,
        first_instance_id: i32,
        player_guid: u64,
        snapshot: Option<HarvestingLifecycleSnapshot>,
    ) -> io::Result<Self> {
        let activity_state = snapshot.as_ref().map(|saved| saved.activity.clone());
        let engine = Type8Engine::load(
            directory,
            content_file,
            game_id,
            first_instance_id,
            activity_state,
        )?;
        match snapshot {
            Some(saved) => {
                if saved.version != 1
                    || saved.game_id != game_id
                    || saved.player_guid != player_guid
                {
                    return Err(invalid("invalid Harvesting lifecycle snapshot"));
                }
                Self::restore(engine, saved.runs, game_id, player_guid)
            }
            None => Self::new(engine, game_id, player_guid),
        }
    }

    pub fn new(engine: Type8Engine, game_id: i32, player_guid: u64) -> io::Result<Self> {
        if engine.state().active.is_some() {
            return Err(invalid(
                "active Harvesting state needs its lifecycle snapshot",
            ));
        }
        let runs = MinigameRuns::new(engine.state().next_instance_id)?;
        Self::restore(engine, runs, game_id, player_guid)
    }

    pub fn restore(
        engine: Type8Engine,
        runs: MinigameRuns,
        game_id: i32,
        player_guid: u64,
    ) -> io::Result<Self> {
        if player_guid == 0 || super::region(game_id).is_none() {
            return Err(invalid("invalid Harvesting player or region"));
        }
        if engine.profile_id() != CHEF_PROFILE_ID {
            return Err(invalid("Harvesting content must use the Chef profile"));
        }
        runs.validate()?;
        validate_pair(&engine, &runs)?;
        Ok(Self {
            engine,
            runs,
            game_id,
            player_guid,
        })
    }

    pub fn snapshot(&self) -> HarvestingLifecycleSnapshot {
        HarvestingLifecycleSnapshot {
            version: 1,
            game_id: self.game_id,
            activity: self.engine.state().clone(),
            runs: self.runs.clone(),
            player_guid: self.player_guid,
        }
    }

    pub fn activity_state(&self) -> &Type8State {
        self.engine.state()
    }

    pub fn run_state(&self) -> Option<RunState> {
        self.runs
            .current_instance_id()
            .and_then(|id| self.runs.state(id))
    }

    pub fn begin(&mut self, profile_id: i32) -> io::Result<HarvestingActivityOutcome> {
        let mut engine = self.engine.clone();
        let activity = engine.begin(profile_id)?;
        let id = engine
            .state()
            .active
            .as_ref()
            .map(|run| run.instance_id)
            .ok_or_else(|| invalid("Harvesting offer did not create an activity run"))?;
        let mut runs = self.runs.clone();
        let offered = runs.offer(run_spec(&engine, self.game_id)?)?;
        if !offered.accepted || runs.current_instance_id() != Some(id) {
            return Err(invalid("Harvesting and minigame instance ids diverged"));
        }
        self.engine = engine;
        self.runs = runs;

        let mut out = from_run(offered);
        out.accepted = true;
        out.changed |= activity.changed;
        Ok(out)
    }

    /// Handle shared lifecycle controls and Harvesting's type-8 movie messages.
    /// A win requires the caller's staged reward state.
    pub fn handle(
        &mut self,
        packet: &[u8],
        reward_state: Option<&RewardState>,
    ) -> io::Result<HarvestingActivityOutcome> {
        if packet.len() >= 3 && packet[..2] == 39_u16.to_le_bytes() {
            match packet[2] {
                5 | 6 | 7 => return self.handle_control(packet),
                14 => return self.handle_movie(packet, reward_state),
                _ => return Ok(HarvestingActivityOutcome::default()),
            }
        }
        if packet == [26, 0, 42, 0] {
            return self.close();
        }
        Ok(HarvestingActivityOutcome::default())
    }

    pub fn tick(&mut self, elapsed_ms: u32) -> io::Result<HarvestingActivityOutcome> {
        let mut engine = self.engine.clone();
        let activity = engine.tick(elapsed_ms)?;
        let mut runs = self.runs.clone();
        let mut out = HarvestingActivityOutcome {
            accepted: activity.changed,
            changed: activity.changed,
            replies: activity.replies,
            ..Default::default()
        };
        if engine
            .state()
            .active
            .as_ref()
            .is_some_and(|run| run.phase == Type8Phase::Lost)
        {
            if let Some(id) = runs.current_instance_id() {
                let adapter = HarvestingActivity(&engine);
                let finished = runs.finish(id, &adapter, None)?;
                out.accepted |= finished.accepted;
                out.changed |= finished.changed;
                out.state = finished.state;
                out.replies.extend(finished.replies);
            }
        }
        self.engine = engine;
        self.runs = runs;
        Ok(out)
    }

    pub fn disconnect(&mut self) -> HarvestingActivityOutcome {
        let mut runs = self.runs.clone();
        let cancelled = runs.disconnect();
        if cancelled.changed {
            let mut engine = self.engine.clone();
            let _ = engine.cancel();
            self.engine = engine;
            self.runs = runs;
        }
        from_run(cancelled)
    }

    fn handle_control(&mut self, packet: &[u8]) -> io::Result<HarvestingActivityOutcome> {
        let mut engine = self.engine.clone();
        let activity = engine.handle(packet)?;
        let mut runs = self.runs.clone();
        let lifecycle = runs.handle_client(packet)?;
        if lifecycle.changed {
            if lifecycle.state == Some(RunState::Cancelled) {
                let _ = engine.cancel();
            }
            validate_pair(&engine, &runs)?;
            self.engine = engine;
            self.runs = runs;
        }
        let mut out = from_run(lifecycle);
        out.accepted |= activity.changed;
        out.changed |= activity.changed;
        Ok(out)
    }

    fn close(&mut self) -> io::Result<HarvestingActivityOutcome> {
        let mut engine = self.engine.clone();
        let mut runs = self.runs.clone();
        let closed = runs.handle_client(&[26, 0, 42, 0])?;
        if closed.changed {
            let _ = engine.cancel();
            validate_pair(&engine, &runs)?;
            self.engine = engine;
            self.runs = runs;
        }
        Ok(from_run(closed))
    }

    fn handle_movie(
        &mut self,
        packet: &[u8],
        reward_state: Option<&RewardState>,
    ) -> io::Result<HarvestingActivityOutcome> {
        if let Some(id) = self.runs.current_instance_id() {
            if self
                .runs
                .state(id)
                .is_some_and(|state| matches!(state, RunState::Won | RunState::Lost))
                && movie_command(packet)? != "FRServer_GameClose"
            {
                return Ok(HarvestingActivityOutcome::default());
            }
        }
        let mut engine = self.engine.clone();
        let activity = engine.handle(packet)?;
        let mut runs = self.runs.clone();
        let mut out = HarvestingActivityOutcome {
            accepted: activity.changed,
            changed: activity.changed,
            replies: activity.replies,
            ..Default::default()
        };

        if let Some(id) = runs.current_instance_id() {
            if engine
                .state()
                .active
                .as_ref()
                .is_some_and(|run| run.phase == Type8Phase::Playing)
                && runs.state(id) == Some(RunState::Starting)
            {
                let started = runs.started(id);
                out.accepted |= started.accepted;
                out.changed |= started.changed;
                out.state = started.state;
            }
            let adapter = HarvestingActivity(&engine);
            let scored = runs.score(id, &adapter);
            out.accepted |= scored.accepted;
            out.changed |= scored.changed;
            out.state = scored.state.or(out.state);

            let terminal = engine
                .state()
                .active
                .as_ref()
                .is_some_and(|run| matches!(run.phase, Type8Phase::Won | Type8Phase::Lost));
            if terminal && runs.state(id) == Some(RunState::Playing) {
                let won = engine
                    .state()
                    .active
                    .as_ref()
                    .is_some_and(|run| run.phase == Type8Phase::Won);
                let grant = if won {
                    let base = reward_state
                        .ok_or_else(|| invalid("Harvesting win requires staged reward state"))?;
                    let transaction = activity.transaction.as_ref();
                    Some((
                        base,
                        reward_grant(transaction, self.game_id, self.player_guid)?,
                    ))
                } else {
                    None
                };
                let finished = match grant.as_ref() {
                    Some((base, grant)) => runs.finish(id, &adapter, Some((base, grant)))?,
                    None => runs.finish(id, &adapter, None)?,
                };
                out.accepted |= finished.accepted;
                out.changed |= finished.changed;
                out.state = finished.state.or(out.state);
                out.replies = finished.replies;
                out.reward_packets = finished.reward_packets;
                out.reward_state = finished.reward_state;
                if won && out.reward_state.is_some() {
                    out.quest_events = grant_quest_events(
                        &grant
                            .as_ref()
                            .expect("winning Harvesting run has a grant")
                            .1,
                        self.game_id,
                    );
                }
            } else if runs.state(id).is_some_and(|state| {
                matches!(state, RunState::Won | RunState::Lost | RunState::Closed)
            }) {
                out.replies.clear();
            }
        }

        validate_pair(&engine, &runs)?;
        self.engine = engine;
        self.runs = runs;
        Ok(out)
    }
}

fn run_spec(engine: &Type8Engine, game_id: i32) -> io::Result<RunSpec> {
    let content = super::content(game_id).map_err(|_| invalid("invalid Harvesting region"))?;
    let mut spec = RunSpec::empty(
        game_id,
        HARVESTING_TYPE_ID,
        RunMode::Flash {
            movie: engine.launch_movie().to_owned(),
        },
    );
    spec.name_id = content.name_id;
    spec.icon_id = content.icon_id;
    spec.description_id = content.description_id;
    spec.profile_type = engine.profile_id();
    spec.preload_file = engine.preload_file().to_owned();
    Ok(spec)
}

fn reward_grant(
    transaction: Option<&Type8Transaction>,
    game_id: i32,
    player_guid: u64,
) -> io::Result<RewardGrant> {
    let transaction =
        transaction.ok_or_else(|| invalid("Harvesting win has no produce transaction"))?;
    if !transaction.consumes.is_empty() {
        return Err(invalid("Harvesting payout cannot consume materials"));
    }
    if transaction.grants.is_empty() || transaction.grants.len() > MAX_REWARD_ITEMS {
        return Err(invalid(
            "Harvesting payout has an invalid produce row count",
        ));
    }
    let mut quantities = BTreeMap::<i32, u32>::new();
    for &(definition_id, count) in &transaction.grants {
        let quantity = quantities.entry(definition_id).or_default();
        *quantity = quantity
            .checked_add(count)
            .ok_or_else(|| invalid("Harvesting produce quantity overflow"))?;
    }
    let region = super::region(game_id).ok_or_else(|| invalid("unknown Harvesting region"))?;
    // (R) The engine's bundle count comes from the local content spec. The
    // original reward set and XP amount were not present in client assets.
    Ok(RewardGrant {
        coins: 0,
        items: quantities
            .into_iter()
            .map(|(definition_id, count)| ItemGrant {
                definition_id,
                count,
            })
            .collect(),
        job_xp: BTreeMap::from([(CHEF_PROFILE_ID, RECONSTRUCTED_CHEF_XP_PER_WIN)]),
        display: RewardDisplay {
            source_guid: player_guid,
            player_guid,
            reward_kind: 0,
            icon_id: HARVESTING_ICON_ID,
            name_id: region.name_id,
            rows: Vec::new(),
        },
    })
}

fn grant_quest_events(grant: &RewardGrant, game_id: i32) -> Vec<QuestEvent> {
    // (R) Forward staged item grants and the region id to the local quest engine.
    let mut events: Vec<_> = grant
        .items
        .iter()
        .map(|item| QuestEvent::CollectItem {
            item_id: item.definition_id,
            quantity: item.count,
        })
        .collect();
    events.push(QuestEvent::FinishMinigame {
        minigame_id: game_id as u32,
    });
    events
}

fn from_run(run: RunOutcome) -> HarvestingActivityOutcome {
    HarvestingActivityOutcome {
        accepted: run.accepted,
        changed: run.changed,
        state: run.state,
        replies: run.replies,
        reward_packets: run.reward_packets,
        reward_state: run.reward_state,
        quest_events: Vec::new(),
    }
}

fn validate_pair(engine: &Type8Engine, runs: &MinigameRuns) -> io::Result<()> {
    if engine.state().next_instance_id != runs.next_instance_id() {
        return Err(invalid("Harvesting lifecycle counters diverged"));
    }
    match (&engine.state().active, runs.current_instance_id()) {
        (Some(activity), Some(id)) if activity.instance_id == id => {
            let expected = match activity.phase {
                Type8Phase::Offered => RunState::Offered,
                Type8Phase::Launched | Type8Phase::Connected | Type8Phase::LevelOffered => {
                    RunState::Starting
                }
                Type8Phase::Playing | Type8Phase::LevelComplete => RunState::Playing,
                Type8Phase::Won => RunState::Won,
                Type8Phase::Lost => RunState::Lost,
            };
            if runs.state(id) != Some(expected) {
                return Err(invalid("Harvesting activity and lifecycle phases diverged"));
            }
        }
        (None, Some(id))
            if runs
                .state(id)
                .is_some_and(|state| matches!(state, RunState::Won | RunState::Lost)) => {}
        (None, None) => {}
        _ => {
            return Err(invalid(
                "Harvesting activity and lifecycle instances diverged",
            ));
        }
    }
    Ok(())
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn movie_command(packet: &[u8]) -> io::Result<&str> {
    if packet.len() < 20 || packet[..3] != [39, 0, 14] {
        return Err(invalid("malformed Harvesting movie packet"));
    }
    let size = i32::from_le_bytes(packet[15..19].try_into().unwrap());
    if size < 2 || size as usize != packet.len() - 19 || packet.last() != Some(&0) {
        return Err(invalid("malformed Harvesting movie payload"));
    }
    let body = std::str::from_utf8(&packet[19..packet.len() - 1])
        .map_err(|_| invalid("Harvesting movie payload is not UTF-8"))?;
    Ok(body.split('\t').next().unwrap_or_default())
}
