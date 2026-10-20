//! Shared S6 fixture: content paths, the opening stage/actor selected by a
//! goal, the reply-text extractor and the owned reward count. Request bodies
//! live in `tests/common/gateway_packets.rs`. `tests/s6_characterization.rs`
//! wraps the state in a `Gateway`; `src/gameplay/s6_characterization.rs` wraps
//! it in a bare `Gameplay`.

use osfr_tools::{
    gameplay::{Gameplay, GameplayState},
    inventory::Inventory,
    opening::{OpeningActor, OpeningEngine, OpeningTrigger},
};
use std::path::PathBuf;

pub fn references() -> PathBuf {
    std::env::var_os("OSFR_S6_REFERENCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references")
        })
}

pub fn reference(content: &str) -> PathBuf {
    let path = references().join(content);
    assert!(
        path.join("opening.json").is_file(),
        "S6 requires the private content at {}",
        path.display()
    );
    path
}

#[allow(dead_code)] // The packet test root uses forging/smelting content only.
pub fn w104_content() -> PathBuf {
    let path = std::env::var_os("OSFR_S6_W104_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/build/w104/scratch/content")
        });
    assert!(
        path.join("opening.json").is_file() && path.join("harvesting.json").is_file(),
        "S6 requires the private W104 content at {}",
        path.display()
    );
    path
}

fn inner(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

/// System chat texts from either wrapped Gateway replies or bare Outcome
/// replies.
pub fn chat_texts(replies: &[Vec<u8>]) -> Vec<String> {
    let mut texts = Vec::new();
    for reply in replies {
        let packet = inner(reply);
        if packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8 {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            if let Some(body) = packet.get(8..8 + length)
                && let Ok(text) = std::str::from_utf8(body)
            {
                texts.push(text.to_owned());
            }
        }
    }
    texts
}

#[derive(Clone, Copy)]
pub enum TutorialGoal {
    Craft(u32),
    Gather(u32),
}

impl TutorialGoal {
    pub fn matches(self, trigger: &OpeningTrigger) -> bool {
        match (self, trigger) {
            (Self::Craft(recipe), OpeningTrigger::Craft { recipe_id }) => *recipe_id == recipe,
            (Self::Gather(resource), OpeningTrigger::Gather { resource_id }) => {
                *resource_id == resource
            }
            _ => false,
        }
    }
}

pub struct OpeningFixture {
    pub content: PathBuf,
    pub state: GameplayState,
    pub actor: OpeningActor,
    pub stage_id: u32,
    pub objective_index: usize,
}

/// The opening state for the stage that owns `goal`: every earlier stage is
/// complete, the target stage is accepted or pending with the earlier
/// objectives counted, and the requested profiles are unlocked.
pub fn opening_fixture(
    content_name: &str,
    goal: TutorialGoal,
    unlocked: &[i32],
    accepted: bool,
    active: i32,
    supplies: &[(i32, u32)],
) -> OpeningFixture {
    let content = reference(content_name);
    let base = Gameplay::load(&content, None).unwrap();
    let engine = OpeningEngine::load(&content, base.state().opening.clone()).unwrap();
    let (stage_id, objective_index) = engine
        .stages()
        .iter()
        .find_map(|stage| {
            stage
                .objectives
                .iter()
                .position(|objective| goal.matches(&objective.trigger))
                .map(|index| (stage.id, index))
        })
        .expect("tutorial opening objective");
    let mut state = base.state();
    if !supplies.is_empty() {
        // Grant through the production inventory so the fixture gets validated
        // stacks and instance ids instead of a hand-built item.
        let mut inventory = Inventory::load(&content, Some(state.inventory)).unwrap();
        for (definition, count) in supplies {
            inventory.grant(0, *definition, *count).unwrap();
        }
        state.inventory = inventory.state().clone();
    }
    for stage in engine.stages() {
        let progress = state
            .opening
            .as_mut()
            .unwrap()
            .stages
            .get_mut(&stage.id)
            .unwrap();
        if stage.id < stage_id {
            progress.accepted = true;
            progress.completed = true;
            progress.rewarded = true;
            progress.counts = stage
                .objectives
                .iter()
                .map(|objective| objective.required)
                .collect();
        } else if stage.id == stage_id {
            progress.accepted = accepted;
            progress.completed = false;
            progress.rewarded = false;
            progress.counts = if accepted {
                stage
                    .objectives
                    .iter()
                    .enumerate()
                    .map(|(index, objective)| {
                        if index < objective_index {
                            objective.required
                        } else {
                            0
                        }
                    })
                    .collect()
            } else {
                vec![0; stage.objectives.len()]
            };
        }
    }
    {
        let profiles = state.profiles.as_mut().unwrap();
        for profile in unlocked {
            profiles.unlocked.insert(*profile);
        }
        profiles.unlocked.insert(active);
        profiles.active = active;
    }
    let actor = engine
        .actors()
        .iter()
        .find(|actor| {
            actor
                .action
                .as_ref()
                .is_some_and(|action| goal.matches(action))
        })
        .expect("tutorial opening actor")
        .clone();
    OpeningFixture {
        content,
        state,
        actor,
        stage_id,
        objective_index,
    }
}
