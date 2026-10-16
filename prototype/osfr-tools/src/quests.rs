//! Persistent local quest counters and a reconstructed checkpoint activity.
//!
//! Original journal bytes stay in a private reference directory. Incremental
//! quest packet layouts are not established: status uses system chat and the
//! original journal counter is restored on reconnect. A zero enemy definition
//! explicitly disables an objective whose original target is not yet proven.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path};

const MAX_CONTENT: usize = 131_072;
const MAX_JOURNAL: usize = 1_048_576;
const LOCAL_COMPONENT_BOUND: f32 = 1_000_000.0;

fn invalid(text: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, text)
}

fn bounded_file(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit as u64 {
        return Err(invalid("quest input is not a bounded regular file"));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(invalid("quest input exceeded its bound"));
    }
    Ok(bytes)
}

fn valid_position(position: [f32; 3]) -> bool {
    position
        .iter()
        .all(|v| v.is_finite() && v.abs() <= LOCAL_COMPONENT_BOUND)
}

fn horizontal_squared(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestProgress {
    pub accepted: bool,
    pub counts: Vec<u32>,
    pub completed: bool,
    pub rewarded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestState {
    pub version: u32,
    pub quests: BTreeMap<u32, QuestProgress>,
    pub trial_best_ms: Option<u64>,
    pub trial_completions: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestReward {
    pub quest_id: u32,
    pub coins: u32,
    pub items: Vec<(i32, u32)>,
}

#[derive(Clone, Debug, Default)]
pub struct QuestOutcome {
    pub replies: Vec<Vec<u8>>,
    pub messages: Vec<String>,
    pub rewards: Vec<QuestReward>,
    pub changed: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct Goal {
    id: u32,
    text: String,
    required: u32,
    enemy_definition: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct Definition {
    id: u32,
    title: String,
    description: String,
    original: bool,
    initially_accepted: bool,
    giver_guid: u64,
    goals: Vec<Goal>,
    coins: u32,
    items: Vec<(i32, u32)>,
}

#[derive(Clone, Debug, Deserialize)]
struct Binding {
    quest_id: u32,
    objective_index: usize,
    offset: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct Journal {
    file: String,
    sha256: String,
    length: usize,
    offset: usize,
    bindings: Vec<Binding>,
}

#[derive(Clone, Debug, Deserialize)]
struct Trial {
    start: [f32; 3],
    checkpoints: Vec<[f32; 3]>,
    radius: f32,
    time_limit_ms: u64,
    cooldown_ms: u64,
    reward_coins: u32,
}

#[derive(Clone, Debug, Deserialize)]
struct Content {
    schema_version: u32,
    definitions: Vec<Definition>,
    journal: Journal,
    trial: Trial,
}

#[derive(Clone, Debug)]
struct ActiveTrial {
    start_ms: u64,
    last_ms: u64,
    last_position: [f32; 3],
    travelled: f32,
    checkpoint: usize,
}

#[derive(Clone, Debug)]
pub struct QuestEngine {
    content: Content,
    journal: Vec<u8>,
    state: QuestState,
    last_position: Option<[f32; 3]>,
    active_trial: Option<ActiveTrial>,
    next_trial_ms: u64,
}

impl QuestEngine {
    pub fn load(directory: &Path, state: Option<QuestState>) -> io::Result<Self> {
        let content: Content =
            serde_json::from_slice(&bounded_file(&directory.join("quests.json"), MAX_CONTENT)?)
                .map_err(|_| invalid("invalid quest content JSON"))?;
        if content.schema_version != 1
            || content.definitions.is_empty()
            || content.definitions.len() > 64
        {
            return Err(invalid("unsupported quest content"));
        }
        let mut ids = BTreeSet::new();
        for definition in &content.definitions {
            if definition.id == 0
                || !ids.insert(definition.id)
                || definition.title.is_empty()
                || definition.title.len() > 256
                || definition.description.len() > 4096
                || definition.goals.is_empty()
                || definition.goals.len() > 32
                || definition.coins > i32::MAX as u32
                || definition.items.len() > 32
                || definition
                    .items
                    .iter()
                    .any(|(id, quantity)| *id <= 0 || *quantity == 0 || *quantity > i32::MAX as u32)
            {
                return Err(invalid("invalid or duplicate quest definition"));
            }
            let mut goal_ids = BTreeSet::new();
            for goal in &definition.goals {
                if goal.id == 0
                    || !goal_ids.insert(goal.id)
                    || goal.required == 0
                    || goal.required > 1_000_000
                    || goal.text.len() > 2048
                    || goal.enemy_definition < 0
                {
                    return Err(invalid("invalid quest objective"));
                }
            }
        }
        let trial = &content.trial;
        if !valid_position(trial.start)
            || trial.checkpoints.len() < 2
            || trial.checkpoints.len() > 32
            || trial.checkpoints.iter().any(|p| !valid_position(*p))
            || !trial.radius.is_finite()
            || !(1.0..=20.0).contains(&trial.radius)
            || !(1000..=600_000).contains(&trial.time_limit_ms)
            || !(1000..=600_000).contains(&trial.cooldown_ms)
            || trial.reward_coins > i32::MAX as u32
        {
            return Err(invalid("invalid checkpoint activity"));
        }
        let path = Path::new(&content.journal.file);
        if path.components().count() != 1
            || !matches!(path.components().next(), Some(Component::Normal(_)))
        {
            return Err(invalid("quest journal must be a direct relative filename"));
        }
        let journal = bounded_file(&directory.join(path), MAX_JOURNAL)?;
        if journal.len() < 4
            || journal.len() != content.journal.length
            || format!("{:x}", Sha256::digest(&journal)) != content.journal.sha256
            || content.journal.offset < 13
            || content.journal.offset.checked_add(journal.len()).is_none()
            || content.journal.offset + journal.len() > MAX_JOURNAL
        {
            return Err(invalid("quest journal hash/length/offset mismatch"));
        }
        let mut offsets = BTreeSet::new();
        for binding in &content.journal.bindings {
            let definition = content
                .definitions
                .iter()
                .find(|d| d.id == binding.quest_id)
                .ok_or_else(|| invalid("journal binding references missing quest"))?;
            if !definition.original
                || binding.objective_index >= definition.goals.len()
                || binding.offset.checked_add(4).is_none()
                || binding.offset + 4 > journal.len()
                || !offsets.insert(binding.offset)
            {
                return Err(invalid("invalid quest journal binding"));
            }
            let initial = i32::from_le_bytes(
                journal[binding.offset..binding.offset + 4]
                    .try_into()
                    .unwrap(),
            );
            if initial < 0 || initial as u32 > definition.goals[binding.objective_index].required {
                return Err(invalid("invalid original objective progress"));
            }
        }
        let mut state = state.unwrap_or(QuestState {
            version: 1,
            quests: BTreeMap::new(),
            trial_best_ms: None,
            trial_completions: 0,
        });
        if state.version != 1
            || state.quests.len() > content.definitions.len()
            || state.quests.keys().any(|id| !ids.contains(id))
            || state
                .trial_best_ms
                .is_some_and(|ms| ms == 0 || ms > content.trial.time_limit_ms)
        {
            return Err(invalid("unsupported or inconsistent saved quests"));
        }
        for definition in &content.definitions {
            let progress = state
                .quests
                .entry(definition.id)
                .or_insert_with(|| QuestProgress {
                    accepted: definition.initially_accepted,
                    counts: vec![0; definition.goals.len()],
                    completed: false,
                    rewarded: false,
                });
            if progress.counts.len() != definition.goals.len()
                || progress
                    .counts
                    .iter()
                    .zip(&definition.goals)
                    .any(|(count, goal)| *count > goal.required)
                || (!progress.accepted
                    && (progress.completed
                        || progress.rewarded
                        || progress.counts.iter().any(|c| *c != 0)))
                || (progress.rewarded && !progress.completed)
                || progress.completed
                    != progress
                        .counts
                        .iter()
                        .zip(&definition.goals)
                        .all(|(count, goal)| *count == goal.required)
            {
                return Err(invalid("saved quest progress is inconsistent"));
            }
        }
        Ok(Self {
            content,
            journal,
            state,
            last_position: None,
            active_trial: None,
            next_trial_ms: 0,
        })
    }

    pub fn state(&self) -> &QuestState {
        &self.state
    }

    /// Gameplay acknowledges only after the complete inventory bundle succeeds.
    pub fn acknowledge_reward(&mut self, quest_id: u32) -> io::Result<()> {
        let progress = self
            .state
            .quests
            .get_mut(&quest_id)
            .ok_or_else(|| invalid("unknown reward quest"))?;
        if !progress.accepted || !progress.completed || progress.rewarded {
            return Err(invalid("quest reward is not pending"));
        }
        progress.rewarded = true;
        Ok(())
    }

    /// Clocks and an unfinished activity never survive a new network session.
    pub fn reset_session(&mut self) {
        self.active_trial = None;
        self.last_position = None;
        self.next_trial_ms = 0;
    }

    pub fn initialization(&self) -> io::Result<QuestOutcome> {
        Ok(QuestOutcome { messages: vec!["Local restoration: realm quest, realm quest accept 900001, realm race. Original quest counters appear in the journal after reconnect; the patrol and checkpoint activity are reconstructed.".into()], ..QuestOutcome::default() })
    }

    pub fn handle_command(&mut self, command: &str, now_ms: u64) -> io::Result<QuestOutcome> {
        if command.len() > 512 {
            return Err(invalid("quest command exceeds limit"));
        }
        let command = command
            .trim()
            .strip_prefix("realm ")
            .unwrap_or(command.trim());
        let words: Vec<_> = command.split_whitespace().collect();
        match words.as_slice() {
            ["quest"] | ["quests"] | ["status"] | ["quest", "list"] | ["quest", "status"] => {
                let messages = self.content.definitions.iter().map(|definition| {
                    let progress = &self.state.quests[&definition.id];
                    let label = if definition.original { "original record" } else { "reconstructed" };
                    let status = if progress.completed { "completed; reward granted" } else if progress.accepted { "active" } else { "available" };
                    let goals = definition.goals.iter().zip(&progress.counts).map(|(goal,count)| format!("{} {count}/{}{}", goal.text, goal.required,
                        if goal.enemy_definition == 0 { " (target definition not recovered)" } else { "" })).collect::<Vec<_>>().join("; ");
                    format!("{} [{}; {label}]: {} — {status}. {goals}", definition.id, if definition.original { "Free Realms" } else { "local" }, definition.title)
                }).collect();
                Ok(QuestOutcome { messages, ..QuestOutcome::default() })
            }
            ["quest", "accept", id] => match id.parse() {
                Ok(id) => self.accept(id),
                Err(_) => Ok(QuestOutcome {
                    messages: vec!["Invalid quest ID. Use realm quest to list available quests.".into()],
                    ..QuestOutcome::default()
                }),
            },
            ["race"] | ["race", "status"] => {
                let trial = &self.content.trial;
                let status = if let Some(active) = &self.active_trial {
                    let target = trial.checkpoints[active.checkpoint];
                    format!("checkpoint {}/{} at X={:.1}, Z={:.1}; elapsed {:.1}s",active.checkpoint+1,trial.checkpoints.len(),target[0],target[2],now_ms.saturating_sub(active.start_ms) as f64/1000.0)
                }
                    else { format!("start near X={:.1}, Z={:.1}; realm race start",trial.start[0],trial.start[2]) };
                Ok(QuestOutcome { messages:vec![format!("Reconstructed checkpoint activity: {status}. Limit {}s; reward {} coins; best {:?}ms.",trial.time_limit_ms/1000,trial.reward_coins,self.state.trial_best_ms)], ..QuestOutcome::default() })
            }
            ["race", "start"] => self.start_trial(now_ms),
            ["race", "cancel"] => { self.active_trial = None; Ok(QuestOutcome { messages: vec!["Checkpoint activity cancelled.".into()], ..QuestOutcome::default() }) }
            _ => Ok(QuestOutcome { messages:vec!["Use realm quest, realm quest accept <id>, realm race start, or realm race cancel.".into()], ..QuestOutcome::default() }),
        }
    }

    fn accept(&mut self, id: u32) -> io::Result<QuestOutcome> {
        let Some(definition) = self.content.definitions.iter().find(|d| d.id == id) else {
            return Ok(QuestOutcome {
                messages: vec![format!(
                    "Quest {id} is unavailable. Use realm quest to list available quests."
                )],
                ..QuestOutcome::default()
            });
        };
        let progress = self.state.quests.get_mut(&id).unwrap();
        if progress.accepted {
            return Ok(QuestOutcome {
                messages: vec![format!(
                    "{} is already {}.",
                    definition.title,
                    if progress.completed {
                        "completed"
                    } else {
                        "active"
                    }
                )],
                ..QuestOutcome::default()
            });
        }
        progress.accepted = true;
        Ok(QuestOutcome {
            changed: true,
            messages: vec![format!(
                "Accepted {}. {}",
                definition.title, definition.description
            )],
            ..QuestOutcome::default()
        })
    }

    pub fn interaction(&mut self, guid: u64) -> io::Result<QuestOutcome> {
        let id = self
            .content
            .definitions
            .iter()
            .find(|d| {
                d.giver_guid != 0 && d.giver_guid == guid && !self.state.quests[&d.id].accepted
            })
            .map(|d| d.id);
        match id {
            Some(id) => self.accept(id),
            None => Ok(QuestOutcome::default()),
        }
    }

    /// Called only for an authoritative, newly defeated enemy, not client claims.
    pub fn enemy_defeated(&mut self, definition_id: i32) -> io::Result<QuestOutcome> {
        if definition_id <= 0 {
            return Err(invalid("invalid defeated enemy definition"));
        }
        let mut outcome = QuestOutcome::default();
        for definition in &self.content.definitions {
            let progress = self.state.quests.get_mut(&definition.id).unwrap();
            if !progress.accepted || progress.rewarded {
                continue;
            }
            for (goal, count) in definition.goals.iter().zip(&mut progress.counts) {
                if goal.enemy_definition != 0
                    && goal.enemy_definition == definition_id
                    && *count < goal.required
                {
                    *count += 1;
                    outcome.changed = true;
                    outcome.messages.push(format!(
                        "{}: {} {count}/{}.",
                        definition.title, goal.text, goal.required
                    ));
                }
            }
            if progress
                .counts
                .iter()
                .zip(&definition.goals)
                .all(|(count, goal)| *count == goal.required)
            {
                progress.completed = true;
                outcome.rewards.push(QuestReward {
                    quest_id: definition.id,
                    coins: definition.coins,
                    items: definition.items.clone(),
                });
                outcome
                    .messages
                    .push(format!("Completed {}.", definition.title));
            }
        }
        Ok(outcome)
    }

    fn start_trial(&mut self, now_ms: u64) -> io::Result<QuestOutcome> {
        if self.active_trial.is_some() {
            return Err(invalid("checkpoint activity is already active"));
        }
        if now_ms < self.next_trial_ms {
            return Err(invalid("checkpoint activity is cooling down"));
        }
        let position = self
            .last_position
            .ok_or_else(|| invalid("position is not initialized"))?;
        let trial = &self.content.trial;
        if horizontal_squared(position, trial.start) > trial.radius.powi(2) {
            return Err(invalid("move to the checkpoint activity start"));
        }
        self.active_trial = Some(ActiveTrial {
            start_ms: now_ms,
            last_ms: now_ms,
            last_position: position,
            travelled: 0.0,
            checkpoint: 0,
        });
        let next = trial.checkpoints[0];
        Ok(QuestOutcome {
            messages: vec![format!(
                "Reconstructed checkpoint activity started. Checkpoint 1: X={:.1}, Z={:.1} (radius {:.0}).",
                next[0], next[2], trial.radius
            )],
            ..QuestOutcome::default()
        })
    }

    pub fn on_position(&mut self, position: [f32; 3], now_ms: u64) -> io::Result<QuestOutcome> {
        if !valid_position(position) {
            return Err(invalid("invalid activity position"));
        }
        self.last_position = Some(position);
        let Some(active) = self.active_trial.as_mut() else {
            return Ok(QuestOutcome::default());
        };
        let elapsed = now_ms
            .checked_sub(active.start_ms)
            .ok_or_else(|| invalid("activity clock moved backwards"))?;
        let delta = now_ms
            .checked_sub(active.last_ms)
            .ok_or_else(|| invalid("activity clock moved backwards"))?;
        let maximum_step = 20.0 * delta as f32 / 1000.0 + 2.0;
        let step = horizontal_squared(position, active.last_position).sqrt();
        let travelled = active.travelled + step;
        if elapsed > self.content.trial.time_limit_ms
            || step > maximum_step
            || travelled > 20.0 * elapsed as f32 / 1000.0 + 2.0
        {
            self.active_trial = None;
            return Ok(QuestOutcome {
                messages: vec![
                    "Checkpoint activity ended: time limit or implausible movement.".into(),
                ],
                ..QuestOutcome::default()
            });
        }
        active.last_ms = now_ms;
        active.last_position = position;
        active.travelled = travelled;
        let trial = &self.content.trial;
        if horizontal_squared(position, trial.checkpoints[active.checkpoint]) > trial.radius.powi(2)
        {
            return Ok(QuestOutcome::default());
        }
        active.checkpoint += 1;
        if active.checkpoint < trial.checkpoints.len() {
            let next = trial.checkpoints[active.checkpoint];
            return Ok(QuestOutcome {
                messages: vec![format!(
                    "Checkpoint reached. Next: X={:.1}, Z={:.1}.",
                    next[0], next[2]
                )],
                ..QuestOutcome::default()
            });
        }
        if elapsed == 0 {
            return Err(invalid("zero-duration activity completion"));
        }
        let completions = self
            .state
            .trial_completions
            .checked_add(1)
            .ok_or_else(|| invalid("activity completion overflow"))?;
        let next_trial_ms = now_ms
            .checked_add(trial.cooldown_ms)
            .ok_or_else(|| invalid("activity clock overflow"))?;
        self.state.trial_completions = completions;
        self.state.trial_best_ms = Some(
            self.state
                .trial_best_ms
                .map_or(elapsed, |old| old.min(elapsed)),
        );
        self.active_trial = None;
        self.next_trial_ms = next_trial_ms;
        Ok(QuestOutcome {
            changed: true,
            messages: vec![format!(
                "Checkpoint activity completed in {:.2}s. {} coins granted; cooldown {}s.",
                elapsed as f64 / 1000.0,
                trial.reward_coins,
                trial.cooldown_ms / 1000
            )],
            rewards: vec![QuestReward {
                quest_id: 0,
                coins: trial.reward_coins,
                items: Vec::new(),
            }],
            ..QuestOutcome::default()
        })
    }

    /// Preserve the complete original journal and all surrounding self fields.
    /// Apply this before an inventory splice, which can move the journal span.
    pub fn patch_self_template(&self, message: &[u8]) -> io::Result<Vec<u8>> {
        if message.len() < 13
            || message.len() > MAX_JOURNAL
            || message[..3] != [5, 0, 1]
            || message[7..9] != [12, 0]
            || u32::from_le_bytes(message[3..7].try_into().unwrap()) as usize != message.len() - 7
            || u32::from_le_bytes(message[9..13].try_into().unwrap()) as usize != message.len() - 13
        {
            return Err(invalid("invalid self packet for quest restoration"));
        }
        let journal = &self.content.journal;
        let end = journal
            .offset
            .checked_add(self.journal.len())
            .ok_or_else(|| invalid("journal range overflow"))?;
        if message.get(journal.offset..end) != Some(self.journal.as_slice()) {
            return Err(invalid(
                "self quest span differs from the verified reference; patch quests before inventory",
            ));
        }
        let mut bytes = message.to_vec();
        for binding in &journal.bindings {
            let progress = self.state.quests[&binding.quest_id].counts[binding.objective_index];
            let offset = journal.offset + binding.offset;
            bytes[offset..offset + 4].copy_from_slice(&(progress as i32).to_le_bytes());
        }
        Ok(bytes)
    }
}
