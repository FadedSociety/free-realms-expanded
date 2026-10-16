//! Bounded, persistent opening-story progression from private recovered content.
//! The local quest IDs, bindings, interaction substitutes and reward economy are
//! explicitly reconstructed. Original text/assets are loaded outside the repo.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn valid_position(position: [f32; 3]) -> bool {
    position
        .iter()
        .all(|v| v.is_finite() && v.abs() <= 1_000_000.0)
}

fn within(a: [f32; 3], b: [f32; 3], radius: f32) -> bool {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f32>() <= radius * radius
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningProgress {
    pub accepted: bool,
    pub counts: Vec<u32>,
    pub completed: bool,
    pub rewarded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningState {
    pub version: u32,
    pub content_sha256: String,
    pub stages: BTreeMap<u32, OpeningProgress>,
    pub discoveries: BTreeSet<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningReward {
    pub stage_id: u32,
    pub coins: u32,
    pub items: Vec<(i32, u32)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpeningOutcome {
    pub changed: bool,
    pub messages: Vec<String>,
    pub rewards: Vec<OpeningReward>,
    pub unlock_profiles: Vec<i32>,
    pub completed_opening: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OpeningTrigger {
    Talk {
        actor_guid: u64,
    },
    Interact {
        actor_guid: u64,
    },
    Gather {
        resource_id: u32,
    },
    Craft {
        recipe_id: u32,
    },
    Visit {
        zone_id: u32,
        position: [f32; 3],
        radius: f32,
    },
    Defeat {
        definition_id: i32,
    },
    ClassSelected {
        profile_id: i32,
    },
    CosmeticEquipped {
        definition_id: i32,
    },
    DungeonCompleted {
        encounter_id: u32,
    },
    AbilityUsed {
        ability_id: i32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningObjective {
    pub id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub description: String,
    pub required: u32,
    pub trigger: OpeningTrigger,
    pub recovery: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningActor {
    pub guid: u64,
    pub definition_id: i32,
    pub name_id: u32,
    pub name: String,
    pub model_id: i32,
    pub model_file_name: String,
    pub zone_id: u32,
    pub position: [f32; 3],
    pub heading: f32,
    pub interaction_radius: f32,
    pub kind: String,
    pub action: Option<OpeningTrigger>,
    pub required_profile: Option<i32>,
    pub provenance: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningStage {
    pub id: u32,
    pub original_quest_id: Option<u32>,
    pub title_id: u32,
    pub description_id: u32,
    pub title: String,
    pub description: String,
    pub giver_guid: u64,
    pub turnin_guid: u64,
    pub ordered: bool,
    pub objectives: Vec<OpeningObjective>,
    pub reward: OpeningReward,
    pub unlock_profiles: Vec<i32>,
    pub provenance: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpeningDiscovery {
    pub id: u32,
    pub name_id: u32,
    pub name: String,
    pub zone_id: u32,
    pub position: [f32; 3],
    pub radius: f32,
    pub reward: OpeningReward,
    pub provenance: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OpeningContent {
    schema_version: u32,
    scope: String,
    reconstruction_notice: String,
    initial_profiles: Vec<i32>,
    actors: Vec<OpeningActor>,
    discoveries: Vec<OpeningDiscovery>,
    stages: Vec<OpeningStage>,
    provenance: serde_json::Value,
}

#[derive(Clone, Copy, Debug)]
pub enum OpeningEvent {
    Talk { actor_id: u64 },
    Interact { actor_id: u64 },
    Gather { resource_id: u32, quantity: u32 },
    Craft { recipe_id: u32, quantity: u32 },
    Visit { zone_id: u32, position: [f32; 3] },
    Defeat { definition_id: i32 },
    ClassSelected { profile_id: i32 },
    CosmeticEquipped { definition_id: i32 },
    DungeonCompleted { encounter_id: u32 },
    AbilityUsed { ability_id: i32 },
}

#[derive(Clone, Debug)]
pub struct OpeningJournalObjective {
    pub id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub description: String,
    pub required: u32,
    pub count: u32,
}

#[derive(Clone, Debug)]
pub struct OpeningJournalEntry {
    pub quest_id: u32,
    pub title_id: u32,
    pub description_id: u32,
    pub title: String,
    pub description: String,
    pub accepted: bool,
    pub completed: bool,
    pub rewarded: bool,
    pub objectives: Vec<OpeningJournalObjective>,
}

#[derive(Clone, Debug)]
pub struct OpeningEngine {
    content: OpeningContent,
    state: OpeningState,
    location: Option<(u32, [f32; 3])>,
    selected_profile: Option<i32>,
}

fn reward_valid(reward: &OpeningReward) -> bool {
    reward.coins <= i32::MAX as u32
        && reward.items.len() <= 16
        && reward
            .items
            .iter()
            .all(|(id, n)| *id > 0 && *n > 0 && *n <= i32::MAX as u32)
}

impl OpeningEngine {
    pub fn load(content_directory: &Path, state: Option<OpeningState>) -> io::Result<Self> {
        let path = content_directory.join("opening.json");
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 524_288 {
            return Err(invalid("opening content must be a bounded regular file"));
        }
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(524_289)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 524_288 {
            return Err(invalid("opening content exceeds size bound"));
        }
        let content: OpeningContent =
            serde_json::from_slice(&bytes).map_err(|_| invalid("invalid opening content JSON"))?;
        Self::validate_content(&content)?;
        let hash = format!("{:x}", Sha256::digest(&bytes));
        let state = state.unwrap_or_else(|| OpeningState {
            version: 1,
            content_sha256: hash.clone(),
            discoveries: BTreeSet::new(),
            stages: content
                .stages
                .iter()
                .map(|stage| {
                    (
                        stage.id,
                        OpeningProgress {
                            accepted: false,
                            counts: vec![0; stage.objectives.len()],
                            completed: false,
                            rewarded: false,
                        },
                    )
                })
                .collect(),
        });
        if state.version != 1
            || state.content_sha256 != hash
            || state.stages.len() != content.stages.len()
        {
            return Err(invalid("opening save does not match this content version"));
        }
        let mut earlier_complete = true;
        for stage in &content.stages {
            let progress = state
                .stages
                .get(&stage.id)
                .ok_or_else(|| invalid("opening save has unknown stages"))?;
            let all = progress.counts.len() == stage.objectives.len()
                && progress
                    .counts
                    .iter()
                    .zip(&stage.objectives)
                    .all(|(count, goal)| *count <= goal.required);
            let finished = progress
                .counts
                .iter()
                .zip(&stage.objectives)
                .all(|(count, goal)| *count == goal.required);
            if !all
                || (progress.accepted && !earlier_complete)
                || (!progress.accepted && progress.counts.iter().any(|n| *n != 0))
                || progress.rewarded != progress.completed
                || (progress.completed && (!progress.accepted || !finished))
            {
                return Err(invalid("inconsistent opening progression save"));
            }
            earlier_complete = progress.completed;
        }
        if state
            .discoveries
            .iter()
            .any(|id| !content.discoveries.iter().any(|d| d.id == *id))
        {
            return Err(invalid("opening save contains an unknown discovery"));
        }
        Ok(Self {
            content,
            state,
            location: None,
            selected_profile: None,
        })
    }

    fn validate_content(content: &OpeningContent) -> io::Result<()> {
        if content.schema_version != 1
            || content.stages.is_empty()
            || content.stages.len() > 32
            || content.actors.is_empty()
            || content.actors.len() > 128
            || content.discoveries.len() > 64
            || content.scope.len() > 2048
            || content.reconstruction_notice.len() > 2048
        {
            return Err(invalid("unsupported or oversized opening content"));
        }
        let actor_ids: BTreeSet<u64> = content.actors.iter().map(|a| a.guid).collect();
        let stage_ids: BTreeSet<u32> = content.stages.iter().map(|s| s.id).collect();
        let discovery_ids: BTreeSet<u32> = content.discoveries.iter().map(|d| d.id).collect();
        if actor_ids.len() != content.actors.len()
            || actor_ids.contains(&0)
            || stage_ids.len() != content.stages.len()
            || stage_ids.contains(&0)
            || discovery_ids.len() != content.discoveries.len()
            || discovery_ids.contains(&0)
        {
            return Err(invalid("duplicate or zero opening content identity"));
        }
        for actor in &content.actors {
            if !valid_position(actor.position)
                || !actor.heading.is_finite()
                || !actor.interaction_radius.is_finite()
                || !(0.1..=100.0).contains(&actor.interaction_radius)
                || actor.name.len() > 256
                || actor.definition_id <= 0
                || actor.model_id <= 0
                || actor.model_file_name.len() > 256
            {
                return Err(invalid("invalid opening actor"));
            }
        }
        for stage in &content.stages {
            if !actor_ids.contains(&stage.giver_guid)
                || !actor_ids.contains(&stage.turnin_guid)
                || stage.objectives.is_empty()
                || stage.objectives.len() > 32
                || stage.title.len() > 256
                || stage.description.len() > 4096
                || !reward_valid(&stage.reward)
                || stage.reward.stage_id != stage.id
                || stage.unlock_profiles.len() > 16
                || stage.unlock_profiles.iter().any(|id| *id <= 0)
            {
                return Err(invalid("invalid opening stage"));
            }
            let ids: BTreeSet<u32> = stage.objectives.iter().map(|o| o.id).collect();
            if ids.len() != stage.objectives.len() || ids.contains(&0) {
                return Err(invalid("invalid opening objective identity"));
            }
            for goal in &stage.objectives {
                if goal.required == 0 || goal.required > 10_000 || goal.description.len() > 4096 {
                    return Err(invalid("invalid opening objective"));
                }
                match goal.trigger {
                    OpeningTrigger::Talk { actor_guid }
                    | OpeningTrigger::Interact { actor_guid }
                        if !actor_ids.contains(&actor_guid) =>
                    {
                        return Err(invalid("opening objective actor missing"));
                    }
                    OpeningTrigger::Visit {
                        position, radius, ..
                    } if !valid_position(position)
                        || !radius.is_finite()
                        || !(0.1..=1000.0).contains(&radius) =>
                    {
                        return Err(invalid("invalid opening objective position"));
                    }
                    _ => {}
                }
            }
        }
        for discovery in &content.discoveries {
            if !valid_position(discovery.position)
                || !discovery.radius.is_finite()
                || !(0.1..=1000.0).contains(&discovery.radius)
                || discovery.name.len() > 256
                || !reward_valid(&discovery.reward)
            {
                return Err(invalid("invalid opening discovery"));
            }
        }
        Ok(())
    }

    pub fn state(&self) -> &OpeningState {
        &self.state
    }
    pub fn actors(&self) -> &[OpeningActor] {
        &self.content.actors
    }
    pub fn discoveries(&self) -> &[OpeningDiscovery] {
        &self.content.discoveries
    }
    pub fn initial_profiles(&self) -> &[i32] {
        &self.content.initial_profiles
    }
    pub fn active_stage(&self) -> Option<&OpeningStage> {
        self.content
            .stages
            .iter()
            .find(|s| !self.state.stages[&s.id].completed)
    }

    /// Read-only view of every content stage, including completed ones. Callers
    /// that must reason about stages which are no longer active (for example
    /// the completed-cave replay gate) use this instead of re-reading
    /// `opening.json`.
    pub fn stages(&self) -> &[OpeningStage] {
        &self.content.stages
    }

    pub fn objectives(&self) -> &[OpeningObjective] {
        self.active_stage()
            .map(|s| s.objectives.as_slice())
            .unwrap_or(&[])
    }
    pub fn reset_session(&mut self) {
        self.location = None;
        self.selected_profile = None;
    }
    pub fn complete(&self) -> bool {
        self.active_stage().is_none()
            && self.state.discoveries.len() == self.content.discoveries.len()
    }

    pub fn journal_entries(&self) -> Vec<OpeningJournalEntry> {
        self.content
            .stages
            .iter()
            .map(|stage| {
                let progress = &self.state.stages[&stage.id];
                OpeningJournalEntry {
                    quest_id: stage.id,
                    title_id: stage.title_id,
                    description_id: stage.description_id,
                    title: stage.title.clone(),
                    description: stage.description.clone(),
                    accepted: progress.accepted,
                    completed: progress.completed,
                    rewarded: progress.rewarded,
                    objectives: stage
                        .objectives
                        .iter()
                        .zip(&progress.counts)
                        .map(|(goal, count)| OpeningJournalObjective {
                            id: goal.id,
                            name_id: goal.name_id,
                            description_id: goal.description_id,
                            description: goal.description.clone(),
                            required: goal.required,
                            count: *count,
                        })
                        .collect(),
                }
            })
            .collect()
    }

    pub fn initialization(&self) -> OpeningOutcome {
        let mut outcome = self.status();
        outcome
            .messages
            .insert(0, self.content.reconstruction_notice.clone());
        outcome
    }

    pub fn status(&self) -> OpeningOutcome {
        let mut outcome = OpeningOutcome::default();
        let completed = self.state.stages.values().filter(|s| s.completed).count();
        outcome.messages.push(format!(
            "Opening: {completed}/{} steps, {}/{} discoveries.",
            self.content.stages.len(),
            self.state.discoveries.len(),
            self.content.discoveries.len()
        ));
        if let Some(stage) = self.active_stage() {
            let progress = &self.state.stages[&stage.id];
            outcome.messages.push(format!(
                "{}: {}",
                stage.title,
                if progress.accepted {
                    "active"
                } else {
                    "talk to its guide to accept"
                }
            ));
            if let Some((goal, count)) = stage
                .objectives
                .iter()
                .zip(&progress.counts)
                .find(|(g, n)| **n < g.required)
            {
                outcome
                    .messages
                    .push(format!("{} ({count}/{})", goal.description, goal.required));
            } else {
                outcome
                    .messages
                    .push("Objectives complete. Return to the guide to turn in this step.".into());
            }
            let guid = if progress.accepted
                && progress
                    .counts
                    .iter()
                    .zip(&stage.objectives)
                    .all(|(n, g)| *n == g.required)
            {
                stage.turnin_guid
            } else {
                stage.giver_guid
            };
            if let Some(actor) = self.content.actors.iter().find(|a| a.guid == guid) {
                outcome.messages.push(format!(
                    "Guide: {} at {:.1}, {:.1}, {:.1}.",
                    actor.name, actor.position[0], actor.position[1], actor.position[2]
                ));
            }
        } else {
            outcome
                .messages
                .push("Opening story complete. Your progress and rewards are saved.".into());
            outcome.completed_opening = self.complete();
        }
        outcome
    }

    pub fn handle_command(&mut self, command: &str) -> OpeningOutcome {
        let normalized = command.trim().to_ascii_lowercase();
        let words: Vec<&str> = normalized.split_whitespace().collect();
        let command = words.last().copied().unwrap_or("status");
        if matches!(command, "accept" | "turnin") {
            if let Some(stage) = self.active_stage() {
                let accepted = self.state.stages[&stage.id].accepted;
                if command == "accept" && accepted {
                    return self.status();
                }
                if command == "turnin" && !accepted {
                    return OpeningOutcome {
                        messages: vec!["Accept the current step from its guide first.".into()],
                        ..OpeningOutcome::default()
                    };
                }
                let actor = if command == "accept" {
                    stage.giver_guid
                } else {
                    stage.turnin_guid
                };
                return self
                    .on_interaction(actor)
                    .unwrap_or_else(|_| OpeningOutcome {
                        messages: vec![
                            "Move within talking distance of the current guide first.".into(),
                        ],
                        ..OpeningOutcome::default()
                    });
            }
        }
        let mut outcome = self.status();
        if command == "help" {
            outcome.messages.push("Talk to nearby guides to accept and turn in steps. Use real resource objects, equipment, classes and combat; chat cannot complete objectives.".into());
        }
        outcome
    }

    fn near_actor(&self, guid: u64) -> bool {
        let Some((zone, position)) = self.location else {
            return false;
        };
        self.content.actors.iter().any(|a| {
            a.guid == guid
                && a.zone_id == zone
                && within(position, a.position, a.interaction_radius)
        })
    }

    pub fn on_position(&mut self, zone_id: u32, position: [f32; 3]) -> io::Result<OpeningOutcome> {
        if !valid_position(position) {
            return Err(invalid("invalid opening player position"));
        }
        self.location = Some((zone_id, position));
        self.on_event(OpeningEvent::Visit { zone_id, position })
    }

    /// Move the remembered interaction origin without evaluating position
    /// events. Developer relocation uses this so a teleport cannot grant
    /// quests, discoveries or rewards by itself; later real client movement
    /// still follows the normal rules.
    pub fn set_location_without_events(
        &mut self,
        zone_id: u32,
        position: [f32; 3],
    ) -> io::Result<()> {
        if !valid_position(position) {
            return Err(invalid("invalid opening player position"));
        }
        self.location = Some((zone_id, position));
        Ok(())
    }

    pub fn on_interaction(&mut self, actor_guid: u64) -> io::Result<OpeningOutcome> {
        if !self.near_actor(actor_guid) {
            return Ok(OpeningOutcome {
                messages: vec!["Move closer to that opening guide or resource.".into()],
                ..OpeningOutcome::default()
            });
        }
        let actor = self
            .content
            .actors
            .iter()
            .find(|a| a.guid == actor_guid)
            .cloned()
            .ok_or_else(|| invalid("unknown opening actor"))?;
        if actor.required_profile.is_some() && actor.required_profile != self.selected_profile {
            let job = match actor.required_profile.unwrap() {
                1 => "Adventurer",
                14 => "Miner",
                16 => "Blacksmith",
                43 => "Brawler",
                45 => "Chef",
                11 => "Medic",
                35 => "Archer",
                _ => "the required job",
            };
            return Ok(OpeningOutcome {
                messages: vec![format!("Switch to {job} before using {}.", actor.name)],
                ..OpeningOutcome::default()
            });
        }
        if let Some(action) = actor.action {
            let event = match action {
                OpeningTrigger::Gather { resource_id } => OpeningEvent::Gather {
                    resource_id,
                    quantity: 1,
                },
                OpeningTrigger::Craft { recipe_id } => OpeningEvent::Craft {
                    recipe_id,
                    quantity: 1,
                },
                _ => OpeningEvent::Interact {
                    actor_id: actor_guid,
                },
            };
            self.on_event(event)
        } else {
            self.on_event(OpeningEvent::Talk {
                actor_id: actor_guid,
            })
        }
    }

    pub fn on_event(&mut self, event: OpeningEvent) -> io::Result<OpeningOutcome> {
        match event {
            OpeningEvent::Gather {
                resource_id,
                quantity,
            } if resource_id == 0 || !(1..=10_000).contains(&quantity) => {
                return Err(invalid("invalid opening resource event"));
            }
            OpeningEvent::Craft {
                recipe_id,
                quantity,
            } if recipe_id == 0 || !(1..=10_000).contains(&quantity) => {
                return Err(invalid("invalid opening craft event"));
            }
            OpeningEvent::ClassSelected { profile_id } if profile_id <= 0 => {
                return Err(invalid("invalid opening profile event"));
            }
            OpeningEvent::CosmeticEquipped { definition_id }
            | OpeningEvent::Defeat { definition_id }
                if definition_id <= 0 =>
            {
                return Err(invalid("invalid opening definition event"));
            }
            _ => {}
        }
        let mut outcome = OpeningOutcome::default();
        if matches!(
            event,
            OpeningEvent::Gather { .. } | OpeningEvent::Craft { .. }
        ) {
            let authorized = self.content.actors.iter().any(|actor| {
                self.near_actor(actor.guid)
                    && (actor.required_profile.is_none() || actor.required_profile == self.selected_profile)
                    && matches!((&actor.action, &event),
                        (Some(OpeningTrigger::Gather { resource_id: a }), OpeningEvent::Gather { resource_id: b, .. }) if a == b)
                    || self.near_actor(actor.guid)
                        && (actor.required_profile.is_none() || actor.required_profile == self.selected_profile)
                        && matches!((&actor.action, &event),
                            (Some(OpeningTrigger::Craft { recipe_id: a }), OpeningEvent::Craft { recipe_id: b, .. }) if a == b)
            });
            if !authorized {
                return Ok(outcome);
            }
        }
        if let OpeningEvent::Visit { zone_id, position } = event {
            if !valid_position(position) {
                return Err(invalid("invalid opening visit position"));
            }
            self.location = Some((zone_id, position));
            for discovery in &self.content.discoveries {
                if zone_id == discovery.zone_id
                    && within(position, discovery.position, discovery.radius)
                    && self.state.discoveries.insert(discovery.id)
                {
                    outcome.changed = true;
                    outcome
                        .messages
                        .push(format!("Discovered: {}.", discovery.name));
                    if discovery.reward.coins != 0 || !discovery.reward.items.is_empty() {
                        outcome.rewards.push(discovery.reward.clone());
                    }
                }
            }
        }
        if let OpeningEvent::ClassSelected { profile_id } = event {
            self.selected_profile = Some(profile_id);
        }
        let Some(stage) = self.active_stage().cloned() else {
            outcome.completed_opening = self.complete();
            return Ok(outcome);
        };
        let talking_guid = match event {
            OpeningEvent::Talk { actor_id } => Some(actor_id),
            _ => None,
        };
        if talking_guid.is_some_and(|guid| !self.near_actor(guid)) {
            return Ok(outcome);
        }
        if let OpeningEvent::Interact { actor_id } = event {
            if !self.near_actor(actor_id) {
                return Ok(outcome);
            }
        }
        let progress = self.state.stages.get_mut(&stage.id).unwrap();
        let newly_accepted = !progress.accepted && talking_guid == Some(stage.giver_guid);
        if newly_accepted {
            progress.accepted = true;
            outcome.changed = true;
            outcome.unlock_profiles = stage.unlock_profiles.clone();
            outcome
                .messages
                .push(format!("Accepted: {}. {}", stage.title, stage.description));
        }
        if !progress.accepted {
            return Ok(outcome);
        }
        let next = progress
            .counts
            .iter()
            .zip(&stage.objectives)
            .position(|(n, g)| *n < g.required);
        for (i, goal) in stage.objectives.iter().enumerate() {
            if progress.counts[i] >= goal.required || (stage.ordered && Some(i) != next) {
                continue;
            }
            let quantity = match (&goal.trigger, &event) {
                (OpeningTrigger::Talk { actor_guid }, OpeningEvent::Talk { actor_id })
                    if actor_guid == actor_id =>
                {
                    1
                }
                (OpeningTrigger::Interact { actor_guid }, OpeningEvent::Interact { actor_id })
                    if actor_guid == actor_id =>
                {
                    1
                }
                (
                    OpeningTrigger::Gather { resource_id: a },
                    OpeningEvent::Gather {
                        resource_id: b,
                        quantity,
                    },
                ) if a == b => *quantity,
                (
                    OpeningTrigger::Craft { recipe_id: a },
                    OpeningEvent::Craft {
                        recipe_id: b,
                        quantity,
                    },
                ) if a == b => *quantity,
                (
                    OpeningTrigger::Visit {
                        zone_id: a,
                        position: target,
                        radius,
                    },
                    OpeningEvent::Visit {
                        zone_id: b,
                        position,
                    },
                ) if a == b && within(*target, *position, *radius) => 1,
                (
                    OpeningTrigger::Defeat { definition_id: a },
                    OpeningEvent::Defeat { definition_id: b },
                ) if a == b => 1,
                (
                    OpeningTrigger::ClassSelected { profile_id: a },
                    OpeningEvent::ClassSelected { profile_id: b },
                ) if a == b => 1,
                (
                    OpeningTrigger::CosmeticEquipped { definition_id: a },
                    OpeningEvent::CosmeticEquipped { definition_id: b },
                ) if a == b => 1,
                (
                    OpeningTrigger::DungeonCompleted { encounter_id: a },
                    OpeningEvent::DungeonCompleted { encounter_id: b },
                ) if a == b => 1,
                (
                    OpeningTrigger::AbilityUsed { ability_id: a },
                    OpeningEvent::AbilityUsed { ability_id: b },
                ) if a == b => 1,
                _ => 0,
            };
            if quantity > 0 {
                progress.counts[i] = progress.counts[i]
                    .saturating_add(quantity)
                    .min(goal.required);
                outcome.changed = true;
                outcome.messages.push(format!(
                    "{}: {}/{}.",
                    goal.description, progress.counts[i], goal.required
                ));
            }
        }
        let done = progress
            .counts
            .iter()
            .zip(&stage.objectives)
            .all(|(n, g)| *n == g.required);
        if done && talking_guid == Some(stage.turnin_guid) && !newly_accepted {
            progress.completed = true;
            progress.rewarded = true;
            outcome.changed = true;
            if stage.reward.coins != 0 || !stage.reward.items.is_empty() {
                outcome.rewards.push(stage.reward);
            }
            outcome
                .messages
                .push(format!("Completed: {}.", stage.title));
            if let Some(next) = self.active_stage() {
                outcome
                    .messages
                    .push(format!("Next: {}. Talk to its guide.", next.title));
            } else {
                outcome.completed_opening = self.complete();
                outcome.messages.push(
                    "Opening story complete. Explore your discoveries and return safely.".into(),
                );
            }
        }
        Ok(outcome)
    }
}
