//! Mining adapter for the shared minigame run lifecycle.
//!
//! `Type8Engine` owns the recovered MatchThree board protocol. This adapter
//! owns shared start/result/leave/close state and stages successful rewards
//! through `rewards::apply_grant`.

use osfr_tools::{
    minigame::{Activity, ActivityFinish, MinigameRuns, RunMode, RunOutcome, RunSpec, RunState},
    mining::{
        MINER_PROFILE_ID, MINING_DESCRIPTION_ID, MINING_GAME_ID, MINING_ICON_ID, MINING_NAME_ID,
        Type8Engine, Type8Phase, Type8State, Type8Transaction,
    },
    quest_engine::QuestEvent,
    rewards::{ItemGrant, RewardDisplay, RewardGrant, RewardState},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io, path::Path};

/// (R) Mining XP and its original drop table were not recovered. This XP
/// award reuses the documented Miner progression amount as a local starting
/// rule; item outputs come from the reconstructed Mining content spec.
pub const RECONSTRUCTED_MINER_XP_PER_WIN: u32 = 25;

const MINING_TYPE_ID: i32 = 8;
const MAX_REWARD_ITEMS: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiningLifecycleSnapshot {
    version: u8,
    pub activity: Type8State,
    pub runs: MinigameRuns,
    player_guid: u64,
}

/// Staged Mining change. The caller saves activity, run, reward and quest
/// state together before sending either `replies` or `reward_packets`.
#[derive(Default)]
pub struct MiningActivityOutcome {
    pub accepted: bool,
    pub changed: bool,
    pub state: Option<RunState>,
    pub replies: Vec<Vec<u8>>,
    pub reward_packets: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
    pub quest_events: Vec<QuestEvent>,
}

pub struct MiningLifecycle {
    engine: Type8Engine,
    runs: MinigameRuns,
    player_guid: u64,
}

struct MiningActivity<'a>(&'a Type8Engine);

impl Activity for MiningActivity<'_> {
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
            // (R) Mining's original star calculation was not recovered.
            stars: 0,
            elapsed_seconds,
        })
    }
}

impl MiningLifecycle {
    pub fn load(
        directory: &Path,
        content_file: &str,
        game_id: i32,
        first_instance_id: i32,
        player_guid: u64,
        snapshot: Option<MiningLifecycleSnapshot>,
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
                if saved.version != 1 || saved.player_guid != player_guid {
                    return Err(invalid("invalid Mining lifecycle snapshot"));
                }
                Self::restore(engine, saved.runs, player_guid)
            }
            None => Self::new(engine, player_guid),
        }
    }

    pub fn new(engine: Type8Engine, player_guid: u64) -> io::Result<Self> {
        if engine.state().active.is_some() {
            return Err(invalid("active Mining state needs its lifecycle snapshot"));
        }
        let runs = MinigameRuns::new(engine.state().next_instance_id)?;
        Self::restore(engine, runs, player_guid)
    }

    pub fn restore(engine: Type8Engine, runs: MinigameRuns, player_guid: u64) -> io::Result<Self> {
        if player_guid == 0 {
            return Err(invalid("Mining player guid must be positive"));
        }
        runs.validate()?;
        validate_pair(&engine, &runs)?;
        Ok(Self {
            engine,
            runs,
            player_guid,
        })
    }

    pub fn snapshot(&self) -> MiningLifecycleSnapshot {
        MiningLifecycleSnapshot {
            version: 1,
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

    pub fn begin(&mut self, profile_id: i32) -> io::Result<MiningActivityOutcome> {
        let mut engine = self.engine.clone();
        let activity = engine.begin(profile_id)?;
        let id = engine
            .state()
            .active
            .as_ref()
            .map(|run| run.instance_id)
            .ok_or_else(|| invalid("Mining offer did not create an activity run"))?;
        let mut runs = self.runs.clone();
        let offered = runs.offer(run_spec(&engine))?;
        if !offered.accepted || runs.current_instance_id() != Some(id) {
            return Err(invalid("Mining and minigame instance ids diverged"));
        }
        self.engine = engine;
        self.runs = runs;

        let mut out = from_run(offered);
        // Shared 39/16 is byte-compatible with the engine's former writer.
        // Keep only the shared lifecycle response to avoid duplicate offers.
        out.accepted = true;
        out.changed |= activity.changed;
        Ok(out)
    }

    /// Handle the ordinary shared lifecycle controls and Mining's 39/14 movie
    /// messages. `reward_state` is required only when the server proves a win.
    pub fn handle(
        &mut self,
        packet: &[u8],
        reward_state: Option<&RewardState>,
    ) -> io::Result<MiningActivityOutcome> {
        if packet.len() >= 3 && packet[..2] == 39_u16.to_le_bytes() {
            match packet[2] {
                5 | 6 | 7 => return self.handle_control(packet),
                14 => return self.handle_movie(packet, reward_state),
                _ => return Ok(MiningActivityOutcome::default()),
            }
        }
        if packet == [26, 0, 42, 0] {
            return self.close();
        }
        Ok(MiningActivityOutcome::default())
    }

    pub fn tick(&mut self, elapsed_ms: u32) -> io::Result<MiningActivityOutcome> {
        let mut engine = self.engine.clone();
        let activity = engine.tick(elapsed_ms)?;
        let mut runs = self.runs.clone();
        let mut out = MiningActivityOutcome {
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
                let adapter = MiningActivity(&engine);
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

    pub fn disconnect(&mut self) -> MiningActivityOutcome {
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

    fn handle_control(&mut self, packet: &[u8]) -> io::Result<MiningActivityOutcome> {
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
        // Start and quit replies come from MinigameRuns; the engine's copies
        // are deliberately discarded.
        Ok(out)
    }

    fn close(&mut self) -> io::Result<MiningActivityOutcome> {
        let mut engine = self.engine.clone();
        let mut runs = self.runs.clone();
        let closed = runs.handle_client(&[26, 0, 42, 0])?;
        let mut removal = Vec::new();
        if closed.changed {
            let cancelled = engine.cancel();
            // Native offer cancel is `26/42` followed by the instance removal `39/19`
            // (review Native Crafting Quit 005748). Closing a finished run needs no reply:
            // the results sequence already sent one.
            if closed.state == Some(RunState::Cancelled) {
                removal = cancelled.replies;
            }
            validate_pair(&engine, &runs)?;
            self.engine = engine;
            self.runs = runs;
        }
        let mut out = from_run(closed);
        out.replies.extend(removal);
        Ok(out)
    }

    fn handle_movie(
        &mut self,
        packet: &[u8],
        reward_state: Option<&RewardState>,
    ) -> io::Result<MiningActivityOutcome> {
        if let Some(id) = self.runs.current_instance_id() {
            if self
                .runs
                .state(id)
                .is_some_and(|state| matches!(state, RunState::Won | RunState::Lost))
                && movie_command(packet)? != "FRServer_GameClose"
            {
                return Ok(MiningActivityOutcome::default());
            }
        }
        let mut engine = self.engine.clone();
        let activity = engine.handle(packet)?;
        let mut runs = self.runs.clone();
        let mut out = MiningActivityOutcome {
            accepted: activity.changed,
            changed: activity.changed,
            replies: activity.replies,
            ..Default::default()
        };

        let id = runs.current_instance_id();
        if let Some(id) = id {
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
            let adapter = MiningActivity(&engine);
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
                        .ok_or_else(|| invalid("Mining win requires staged reward state"))?;
                    Some((
                        base,
                        reward_grant(activity.transaction.as_ref(), self.player_guid)?,
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
                out.replies.clear();
                out.replies.extend(finished.replies);
                out.reward_packets = finished.reward_packets;
                out.reward_state = finished.reward_state;
                if won && out.reward_state.is_some() {
                    out.quest_events = grant_quest_events(
                        &grant.as_ref().expect("winning Mining run has a grant").1,
                    );
                }
            } else if runs.state(id).is_some_and(|state| {
                matches!(state, RunState::Won | RunState::Lost | RunState::Closed)
            }) {
                // Terminal callbacks can arrive again after a save/reload.
                // Type8's old result writer is suppressed by the shared ledger.
                out.replies.clear();
            }
        }

        validate_pair(&engine, &runs)?;
        self.engine = engine;
        self.runs = runs;
        Ok(out)
    }
}

fn run_spec(engine: &Type8Engine) -> RunSpec {
    let mut spec = RunSpec::empty(
        MINING_GAME_ID,
        MINING_TYPE_ID,
        RunMode::Flash {
            movie: engine.launch_movie().to_owned(),
        },
    );
    spec.name_id = MINING_NAME_ID;
    spec.icon_id = MINING_ICON_ID;
    spec.description_id = MINING_DESCRIPTION_ID;
    spec.profile_type = engine.profile_id();
    spec.preload_file = engine.preload_file().to_owned();
    spec
}

/// Build the shared Mining payout from the engine's server-owned transaction.
/// The live node adapter and this lifecycle use the same item and job-XP rules.
pub fn reward_grant(
    transaction: Option<&Type8Transaction>,
    player_guid: u64,
) -> io::Result<RewardGrant> {
    let mut quantities = BTreeMap::<i32, u32>::new();
    if let Some(transaction) = transaction {
        if !transaction.consumes.is_empty() {
            return Err(invalid("Mining payout cannot consume materials"));
        }
        if transaction.grants.len() > MAX_REWARD_ITEMS {
            return Err(invalid("Mining payout has too many ore rows"));
        }
        for &(definition_id, count) in &transaction.grants {
            let quantity = quantities.entry(definition_id).or_default();
            *quantity = quantity
                .checked_add(count)
                .ok_or_else(|| invalid("Mining ore quantity overflow"))?;
        }
    }
    Ok(RewardGrant {
        coins: 0,
        items: quantities
            .into_iter()
            .map(|(definition_id, count)| ItemGrant {
                definition_id,
                count,
            })
            .collect(),
        job_xp: BTreeMap::from([(MINER_PROFILE_ID, RECONSTRUCTED_MINER_XP_PER_WIN)]),
        display: RewardDisplay {
            source_guid: player_guid,
            player_guid,
            reward_kind: 0,
            icon_id: MINING_ICON_ID,
            name_id: MINING_NAME_ID,
            rows: Vec::new(),
        },
    })
}

fn grant_quest_events(grant: &RewardGrant) -> Vec<QuestEvent> {
    let mut events: Vec<_> = grant
        .items
        .iter()
        .map(|item| QuestEvent::CollectItem {
            item_id: item.definition_id,
            quantity: item.count,
        })
        .collect();
    events.push(QuestEvent::FinishMinigame {
        minigame_id: MINING_GAME_ID as u32,
    });
    events
}

fn from_run(run: RunOutcome) -> MiningActivityOutcome {
    MiningActivityOutcome {
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
        return Err(invalid("Mining lifecycle counters diverged"));
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
                return Err(invalid("Mining activity and lifecycle phases diverged"));
            }
        }
        (None, Some(id))
            if runs
                .state(id)
                .is_some_and(|state| matches!(state, RunState::Won | RunState::Lost)) => {}
        (None, None) => {}
        _ => return Err(invalid("Mining activity and lifecycle instances diverged")),
    }
    Ok(())
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn movie_command(packet: &[u8]) -> io::Result<&str> {
    if packet.len() < 20 || packet[..3] != [39, 0, 14] {
        return Err(invalid("malformed Mining movie packet"));
    }
    let size = i32::from_le_bytes(packet[15..19].try_into().unwrap());
    if size < 2 || size as usize != packet.len() - 19 || packet.last() != Some(&0) {
        return Err(invalid("malformed Mining movie payload"));
    }
    let body = std::str::from_utf8(&packet[19..packet.len() - 1])
        .map_err(|_| invalid("Mining movie payload is not UTF-8"))?;
    Ok(body.split('\t').next().unwrap_or_default())
}
