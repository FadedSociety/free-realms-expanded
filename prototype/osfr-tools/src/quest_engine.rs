//! Definition-driven quest state and protocol sequence candidate.
//!
//! Native packet layouts and ordering citations live beside each message builder.
//! Eligibility, progress bindings, silent flags, unknown trailing values, turn-in
//! order, and grant keys are reconstructed server policy (R).

use crate::{
    content::{ContentDb, ContentGoal},
    opening::{OpeningEngine, OpeningTrigger},
    rewards::{
        self, GrantFailure, ItemGrant, RewardBundleRow, RewardDisplay, RewardGrant, RewardState,
    },
    startup_messages, startup_world,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

const MAX_OBJECTIVES: usize = 64;
const MAX_QUESTS: usize = 1024;
const MAX_COUNT: u32 = 1_000_000;
/// Local bound on a registered UI event name (R; the native limit is not recovered).
const MAX_UI_EVENT_BYTES: usize = 64;
const MAX_SPLIT_CONTENT_BYTES: u64 = 8 * 1024 * 1024;
const MAX_SPLIT_TOTAL_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Deserialize)]
struct SplitNpcFile {
    schema_version: u32,
    records: Vec<SplitNpcRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SplitQuestFile {
    schema_version: u32,
    records: Vec<Value>,
}

// (R) Additional server objective bindings. The client consumes counters,
// not these event identities; authors must supply provenance in `source`.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RuntimeGoal {
    GatherResource {
        id: u32,
        resource_id: u32,
        quantity: u32,
        source: Vec<crate::content::SourceTag>,
    },
    CraftRecipe {
        id: u32,
        recipe_id: u32,
        quantity: u32,
        source: Vec<crate::content::SourceTag>,
    },
    FinishMinigame {
        id: u32,
        minigame_id: u32,
        count: u32,
        source: Vec<crate::content::SourceTag>,
    },
    EquipItem {
        id: u32,
        definition_id: i32,
        source: Vec<crate::content::SourceTag>,
    },
    VisitPosition {
        id: u32,
        zone_id: u32,
        position: [f32; 3],
        radius: f32,
        source: Vec<crate::content::SourceTag>,
    },
    ClientReported {
        id: u32,
        client_id: u32,
        trigger: ClientTrigger,
        source: Vec<crate::content::SourceTag>,
    },
    // Explicit reach geometry avoids guessing a radius from a POI id.
    ReachPoi {
        id: u32,
        poi_id: u32,
        zone_id: u32,
        position: [f32; 3],
        radius: f32,
        source: Vec<crate::content::SourceTag>,
    },
}

fn runtime_objective(
    value: Value,
) -> io::Result<(QuestObjective, Option<(u32, u32, [f32; 3], f32)>)> {
    let goal: RuntimeGoal =
        serde_json::from_value(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut location = None;
    let (id, required, kind, source) = match goal {
        RuntimeGoal::GatherResource {
            id,
            resource_id,
            quantity,
            source,
        } => (
            id,
            quantity,
            ObjectiveKind::GatherResource { resource_id },
            source,
        ),
        RuntimeGoal::CraftRecipe {
            id,
            recipe_id,
            quantity,
            source,
        } => (
            id,
            quantity,
            ObjectiveKind::CraftRecipe { recipe_id },
            source,
        ),
        RuntimeGoal::FinishMinigame {
            id,
            minigame_id,
            count,
            source,
        } => (
            id,
            count,
            ObjectiveKind::FinishMinigame { minigame_id },
            source,
        ),
        RuntimeGoal::EquipItem {
            id,
            definition_id,
            source,
        } => (id, 1, ObjectiveKind::EquipItem { definition_id }, source),
        RuntimeGoal::ClientReported {
            id,
            client_id,
            trigger,
            source,
        } => (
            id,
            1,
            ObjectiveKind::ClientReported { client_id, trigger },
            source,
        ),
        RuntimeGoal::VisitPosition {
            id,
            zone_id,
            position,
            radius,
            source,
        } => (
            id,
            1,
            ObjectiveKind::VisitPosition {
                zone_id,
                position,
                radius,
            },
            source,
        ),
        RuntimeGoal::ReachPoi {
            id,
            poi_id,
            zone_id,
            position,
            radius,
            source,
        } => {
            if !valid_objective_kind(&ObjectiveKind::VisitPosition {
                zone_id,
                position,
                radius,
            }) {
                return Err(invalid("invalid reach geometry"));
            }
            location = Some((poi_id, zone_id, position, radius));
            (id, 1, ObjectiveKind::ReachPoi { poi_id }, source)
        }
    };
    if source.is_empty() {
        return Err(invalid("runtime quest objective requires provenance"));
    }
    Ok((
        QuestObjective {
            id,
            required,
            description: format!("Objective {id}"),
            kind,
        },
        location,
    ))
}

#[derive(Clone, Copy, Debug, Default)]
struct QuestLocaleFields {
    intro_text_string_id: Option<u32>,
    description_string_id: Option<u32>,
    /// Giver's turn-in line (locale block slot T+2; scripts/quest_text_backfill.py).
    end_text_string_id: Option<u32>,
}

fn take_optional_locale_id(record: &mut Value, field: &str) -> io::Result<Option<u32>> {
    let value = record
        .as_object_mut()
        .ok_or_else(|| invalid("split quest record is not an object"))?
        .remove(field);
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    let id = value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .filter(|id| *id != 0)
        .ok_or_else(|| invalid("split quest locale ID is invalid"))?;
    Ok(Some(id))
}

#[derive(Deserialize)]
struct SplitNpcRecord {
    npc_id: u64,
    #[serde(default)]
    quest_ids_given: Vec<u32>,
    #[serde(default)]
    quests_given: Vec<SplitNpcQuest>,
    #[serde(default)]
    quests_accepted: Vec<SplitNpcQuest>,
}

#[derive(Deserialize)]
struct SplitNpcQuest {
    #[serde(default)]
    quest_id: Option<u32>,
}

fn read_split_file<T: DeserializeOwned>(path: &Path, total: &mut u64) -> io::Result<T> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_SPLIT_CONTENT_BYTES {
        return Err(invalid("split quest input is not a bounded regular file"));
    }
    *total = total.saturating_add(metadata.len());
    if *total > MAX_SPLIT_TOTAL_BYTES {
        return Err(invalid("split quest content exceeds 32 MiB"));
    }
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {error}", path.display()),
        )
    })
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CharacterEligibility {
    /// Caller supplies the real active job level, never a client claim.
    pub player_level: u8,
    /// (R) Jobs eligible for offers and their real levels. Supply only the active job
    /// when enforcing a switch-job requirement; the engine does not infer profiles.
    pub job_levels: BTreeMap<String, u8>,
    pub member: bool,
    pub nonmember_quest_limit: usize,
    pub member_quest_limit: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuestRequirements {
    pub player_level: Option<u8>,
    pub job: Option<(String, u8)>,
    pub members_only: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectiveKind {
    TalkToNpc {
        npc_id: u64,
    },
    KillActorKind {
        actor_kind_id: u32,
    },
    CollectItem {
        item_id: i32,
    },
    EnterArea {
        area_id: u32,
    },
    ReachPoi {
        poi_id: u32,
    },
    FinishMinigame {
        minigame_id: u32,
    },
    UseItem {
        item_id: i32,
    },
    InteractNpc {
        npc_id: u64,
    },
    GatherResource {
        resource_id: u32,
    },
    CraftRecipe {
        recipe_id: u32,
    },
    SelectJob {
        profile_id: i32,
    },
    /// A job-rank gate (Warrior W5). The event must name the same profile and
    /// carry at least `level`; the goal completes once and never regresses (R).
    JobLevel {
        profile_id: i32,
        level: u8,
    },
    EquipItem {
        definition_id: i32,
    },
    CompleteDungeon {
        encounter_id: u32,
    },
    UseAbility {
        ability_id: i32,
    },
    VisitPosition {
        zone_id: u32,
        position: [f32; 3],
        radius: f32,
    },
    /// A goal only the client can judge (a look, a UI event, a first movement). The client
    /// reports it with 45/7; `client_id` names it on the wire and is unique in the
    /// catalogue (R, W175).
    ClientReported {
        client_id: u32,
        trigger: ClientTrigger,
    },
}

/// What the client is told to watch for: 45/6 LookAt, 45/10 UIEvent or 45/11 FirstMovement
/// (piece 072, readers `00c436b0`, `00c437a0`, `00c43870`).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientTrigger {
    /// Actor target. The world-position branch needs the client's private sentinel GUID,
    /// which is not recovered, so content must name an actor (R).
    LookAt {
        guid: u64,
        /// Vertical offset in tenths of a world unit: the client reads an i32 and
        /// scales it by 0.1 (reader `009b2760`, dispatcher `009be130`).
        offset_tenths: i32,
        range: u32,
        angle_degrees: u32,
    },
    UiEvent {
        event: String,
    },
    FirstMovement {
        mouse: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuestObjective {
    pub id: u32,
    pub required: u32,
    pub description: String,
    pub kind: ObjectiveKind,
}

#[derive(Clone, Debug)]
pub struct QuestDefinition {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub giver_npc_id: u64,
    /// NPC that validates `begin_turnin` and presents the 49/13 end screen.
    /// NPC content binds it with a `quests_accepted` row; it defaults to the giver.
    pub turnin_npc_id: u64,
    pub prerequisite_quest_ids: Vec<u32>,
    pub requirements: QuestRequirements,
    pub ordered: bool,
    pub objectives: Vec<QuestObjective>,
    pub reward: RewardGrant,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestStatus {
    Offered,
    Active,
    Completed,
    Abandoned,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestProgress {
    pub status: QuestStatus,
    pub counts: Vec<u32>,
}

/// Character-owned quest state. Persist with the reward state in one save commit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestState {
    pub version: u32,
    pub character_id: u64,
    pub quests: BTreeMap<u32, QuestProgress>,
    pub pending_offer: Option<u32>,
    /// The completed quest whose 49/13 end screen waits for the client's 49/14. Only a paid
    /// turn-in, the locked-profile denial (F249), the quest's own 26/23 abandon or the load-time
    /// save repair of that quest clears it; any other refused reward keeps it so the player can
    /// retry (T382).
    pub pending_turnin: Option<u32>,
    /// Content NPC key that opened `pending_turnin`; it may be a Talk objective target
    /// rather than the formal `turnin_npc_id`. Missing on older saves: the definition's
    /// turn-in NPC is the fallback. Cleared with `pending_turnin`.
    #[serde(default)]
    pub pending_turnin_actor: Option<u64>,
    pub selected_quest_id: Option<u32>,
    pub auto_switch_selected: bool,
}

impl QuestState {
    /// Open a completed quest's end-screen step, recording the actor that opened it so a
    /// reconnect or a second Talk can name that actor in 49/13.
    fn open_pending_turnin(&mut self, quest_id: u32, actor_id: u64) {
        self.pending_turnin = Some(quest_id);
        self.pending_turnin_actor = Some(actor_id);
    }

    fn clear_pending_turnin(&mut self) {
        self.pending_turnin = None;
        self.pending_turnin_actor = None;
    }

    /// The actor that opened the pending screen; older saves fall back to the definition's
    /// formal turn-in NPC.
    fn turnin_actor(&self, turnin_npc_id: u64) -> u64 {
        self.pending_turnin_actor.unwrap_or(turnin_npc_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestDenial {
    WrongGiver,
    Prerequisite,
    Level,
    Job,
    Membership,
    Limit,
    Unavailable,
    Incomplete,
    WrongTurnin,
    NoPendingTurnin,
    /// The bag cannot hold the reward; the end screen stays for a retry (T382).
    NeedSpace,
    /// A content or server fault in the reward row; the end screen stays and 26/23 abandon is
    /// the player's exit (T382).
    RewardData,
    /// A coin or XP amount would pass its wire field; the end screen stays (T382).
    RewardLimit,
}

impl QuestDenial {
    /// The one chat line a reward denial speaks (R wording). The other denials stay silent, as
    /// they were before T382.
    pub fn message(&self) -> Option<&'static str> {
        match self {
            Self::NeedSpace => Some("Your bag is full. Make room and turn the quest in again."),
            Self::RewardData => {
                Some("This quest's reward cannot be paid. You can abandon it from your journal.")
            }
            Self::RewardLimit => {
                Some("You cannot receive this reward right now. Turn the quest in again later.")
            }
            _ => None,
        }
    }
}

/// Semantic quest message. `encode_exact` only returns bytes where the body is
/// decoded by the reviewed guide; 49/1, 49/3 and 49/13 remain typed intents.
/// See Build Guides/Gameplay/Quests and Objectives.md ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â§Network for reader/writer
/// provenance: 49/1 `00c7bb60`/`00a94be0`; 49/3 `00c7cc80`/`00c7ccd0`;
/// 49/8 `00c7b570`/`00c7a910`; 49/9 `00c7b5e0`/`00c7a990`;
/// 49/10 `00c7b650`/`00c7aa10`; 49/13 `00c7bbd0`/`00c7c030`;
/// 49/4 `00c7b420`; 49/6 `00c7b500`; 49/12 `00c7b730`; 47/15 `00a91bf0`.
#[derive(Clone, Debug, PartialEq)]
pub enum QuestMessage {
    QuestInfo {
        quest_id: u32,
    },
    QuestAdd {
        quest_id: u32,
    },
    ObjectiveActivated {
        quest_id: u32,
        objective_id: u32,
        required: u32,
    },
    ObjectiveUpdate {
        quest_id: u32,
        objective_id: u32,
        current: u32,
        progress: f32,
    },
    ObjectiveComplete {
        quest_id: u32,
        objective_id: u32,
        progress: f32,
    },
    QuestEnd {
        quest_id: u32,
        actor_id: u64,
    },
    QuestComplete {
        quest_id: u32,
    },
    CompletedQuestCount {
        count: u32,
    },
    QuestAbandoned {
        quest_id: u32,
    },
    QuestLimit {
        member: bool,
    },
}

impl QuestMessage {
    /// Family and subtype, matching the headers in Quests and Objectives.md ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â§Network.
    pub fn opcode(&self) -> (u16, u32) {
        match self {
            Self::QuestInfo { .. } => (49, 1),
            Self::QuestAdd { .. } => (49, 3),
            Self::ObjectiveActivated { .. } => (49, 8),
            Self::ObjectiveUpdate { .. } => (49, 9),
            Self::ObjectiveComplete { .. } => (49, 10),
            Self::QuestEnd { .. } => (49, 13),
            Self::QuestComplete { .. } => (49, 4),
            Self::CompletedQuestCount { .. } => (49, 12),
            Self::QuestAbandoned { .. } => (49, 6),
            Self::QuestLimit { .. } => (47, 15),
        }
    }

    /// Encodes decoded bodies only. `None` means the guide leaves this body
    /// incomplete or unresolved; callers must not treat an opcode as a packet.
    pub fn encode_exact(&self) -> Option<Vec<u8>> {
        match self {
            Self::ObjectiveActivated {
                quest_id,
                objective_id,
                required,
            } => {
                let mut bytes = header49(8);
                for value in [*quest_id, *objective_id, *required] {
                    put_i32(&mut bytes, value.try_into().ok()?);
                }
                bytes.push(0); // (R) visible notification flag; required field meaning is unresolved.
                Some(bytes)
            }
            Self::ObjectiveUpdate {
                quest_id,
                objective_id,
                current,
                progress,
            } => {
                if !progress.is_finite() || !(0.0..=1.0).contains(progress) {
                    return None;
                }
                let mut bytes = header49(9);
                for value in [*quest_id, *objective_id, *current] {
                    put_i32(&mut bytes, value.try_into().ok()?);
                }
                bytes.extend(progress.to_le_bytes());
                bytes.push(0); // (R) Show the update notification; bool polarity is guide-derived.
                put_i32(&mut bytes, 0); // (R) unresolved trailing dword.
                Some(bytes)
            }
            Self::ObjectiveComplete {
                quest_id,
                objective_id,
                progress,
            } => {
                if !progress.is_finite() || !(0.0..=1.0).contains(progress) {
                    return None;
                }
                let mut bytes = header49(10);
                put_i32(&mut bytes, (*quest_id).try_into().ok()?);
                put_i32(&mut bytes, (*objective_id).try_into().ok()?);
                bytes.extend(progress.to_le_bytes());
                put_i32(&mut bytes, 0); // (R) unresolved dword.
                bytes.push(0); // (R) visible completion notification.
                Some(bytes)
            }
            Self::QuestComplete { quest_id } => Some(id_packet49(4, *quest_id)?),
            Self::CompletedQuestCount { count } => Some(id_packet49(12, *count)?),
            Self::QuestAbandoned { quest_id } => {
                // Reader `0x00c7ae40` (via `0x00c7b500`, exact length): header, u32 id, bool.
                // Without the bool the client drops the packet and keeps the quest listed.
                let mut bytes = id_packet49(6, *quest_id)?;
                bytes.push(0); // (R) 0 = show the abandon chat line (`0x00c7cd40` case 6).
                Some(bytes)
            }
            Self::QuestLimit { member } => Some(vec![47, 0, 15, if *member { 1 } else { 2 }]),
            Self::QuestInfo { .. } | Self::QuestAdd { .. } | Self::QuestEnd { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuestRewardSummary {
    pub quest_id: u32,
    pub coins: u32,
    pub items: Vec<(i32, u32)>,
}

#[derive(Clone, Debug, Default)]
pub struct QuestOutcome {
    pub changed: bool,
    pub messages: Vec<String>,
    /// Ordered server output. Reward bytes come directly from `apply_grant`.
    pub outbound: Vec<OutboundMessage>,
    pub rewards: Vec<QuestRewardSummary>,
    pub denial: Option<QuestDenial>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutboundMessage {
    Quest(QuestMessage),
    Reward(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum QuestEvent {
    TalkToNpc {
        npc_id: u64,
    },
    KillActorKind {
        actor_kind_id: u32,
        count: u32,
    },
    CollectItem {
        item_id: i32,
        quantity: u32,
    },
    EnterArea {
        area_id: u32,
    },
    ReachPoi {
        poi_id: u32,
    },
    FinishMinigame {
        minigame_id: u32,
    },
    UseItem {
        item_id: i32,
        quantity: u32,
    },
    InteractNpc {
        npc_id: u64,
    },
    /// An authoritative job rank after a level change. The match is profile-exact
    /// and completes when this rank is at least the objective level (R).
    JobLevel {
        profile_id: i32,
        level: u8,
    },
    GatherResource {
        resource_id: u32,
        quantity: u32,
    },
    CraftRecipe {
        recipe_id: u32,
        quantity: u32,
    },
    SelectJob {
        profile_id: i32,
    },
    EquipItem {
        definition_id: i32,
    },
    CompleteDungeon {
        encounter_id: u32,
    },
    UseAbility {
        ability_id: i32,
    },
    VisitPosition {
        zone_id: u32,
        position: [f32; 3],
    },
    ClientReported {
        client_id: u32,
    },
}

impl From<OpeningTrigger> for ObjectiveKind {
    fn from(value: OpeningTrigger) -> Self {
        match value {
            OpeningTrigger::Talk { actor_guid } => Self::TalkToNpc { npc_id: actor_guid },
            OpeningTrigger::Interact { actor_guid } => Self::InteractNpc { npc_id: actor_guid },
            OpeningTrigger::Gather { resource_id } => Self::GatherResource { resource_id },
            OpeningTrigger::Craft { recipe_id } => Self::CraftRecipe { recipe_id },
            OpeningTrigger::Visit {
                zone_id,
                position,
                radius,
            } => Self::VisitPosition {
                zone_id,
                position,
                radius,
            },
            OpeningTrigger::Defeat { definition_id } => Self::KillActorKind {
                actor_kind_id: definition_id as u32,
            },
            OpeningTrigger::ClassSelected { profile_id } => Self::SelectJob { profile_id },
            OpeningTrigger::CosmeticEquipped { definition_id } => Self::EquipItem { definition_id },
            OpeningTrigger::DungeonCompleted { encounter_id } => {
                Self::CompleteDungeon { encounter_id }
            }
            OpeningTrigger::AbilityUsed { ability_id } => Self::UseAbility { ability_id },
        }
    }
}

impl From<OpeningTrigger> for QuestEvent {
    fn from(value: OpeningTrigger) -> Self {
        match ObjectiveKind::from(value) {
            ObjectiveKind::TalkToNpc { npc_id } => Self::TalkToNpc { npc_id },
            ObjectiveKind::InteractNpc { npc_id } => Self::InteractNpc { npc_id },
            ObjectiveKind::GatherResource { resource_id } => Self::GatherResource {
                resource_id,
                quantity: 1,
            },
            ObjectiveKind::CraftRecipe { recipe_id } => Self::CraftRecipe {
                recipe_id,
                quantity: 1,
            },
            ObjectiveKind::VisitPosition {
                zone_id, position, ..
            } => Self::VisitPosition { zone_id, position },
            ObjectiveKind::KillActorKind { actor_kind_id } => Self::KillActorKind {
                actor_kind_id,
                count: 1,
            },
            ObjectiveKind::SelectJob { profile_id } => Self::SelectJob { profile_id },
            ObjectiveKind::EquipItem { definition_id } => Self::EquipItem { definition_id },
            ObjectiveKind::CompleteDungeon { encounter_id } => {
                Self::CompleteDungeon { encounter_id }
            }
            ObjectiveKind::UseAbility { ability_id } => Self::UseAbility { ability_id },
            _ => unreachable!("opening trigger adapter only creates opening objective kinds"),
        }
    }
}

#[derive(Clone, Debug)]
struct EngineOptions {
    opening_order: Option<Vec<u32>>,
}

#[derive(Clone, Debug)]
pub struct QuestEngine {
    character_id: u64,
    eligibility: CharacterEligibility,
    definitions: BTreeMap<u32, QuestDefinition>,
    split_locale_fields: BTreeMap<u32, QuestLocaleFields>,
    /// (quest id, objective id) -> what the NPC says when that talk step is done.
    step_dialogue: BTreeMap<(u32, u32), u32>,
    /// (quest id, objective id) -> goal line locale id (content `text_string_id`).
    objective_text: BTreeMap<(u32, u32), u32>,
    job_profile_ids: BTreeMap<String, i32>,
    state: QuestState,
    options: EngineOptions,
    /// Quest offered by the developer `realm offer` command; its accept skips
    /// the prerequisite, level, job and membership checks. It belongs to the
    /// current `pending_offer` only: a successful accept, a decline or a later
    /// ordinary offer clears it, so it can never bypass a later offer's gates.
    dev_offer: Option<u32>,
    disabled: BTreeSet<u32>,
    /// T418: `live_event` quests and whether an admin has them open
    /// (`gameplay/live_events.rs`). A closed one is neither offered nor
    /// accepted, but it is not `disabled`: saves, progress and turn-ins keep it.
    live_events: BTreeMap<u32, bool>,
    location_quests: BTreeMap<u32, BTreeSet<u32>>,
    poi_locations: BTreeMap<u32, (u32, [f32; 3], f32)>,
}

/// 49/13 fields before the reward bundle: the turn-in actor as an 8-byte value, then three
/// dwords. `ShowQuestEndScreen` (`00a95420`) receives the actor pair first, the quest id
/// the "Thanks!" button sends back in 49/14 second, the NPC end-text locale id third, and
/// an animation fourth. The client resolves the actor through its actor table (`00984960`),
/// so `actor_id` must be the runtime actor GUID. A Talk event names the content NPC key;
/// when that key is bound to a different extra-spawn actor, `gameplay/npc.rs` patches the
/// held end screen with the clicked runtime GUID when 26/6 releases it.
fn put_end_screen_ids(bytes: &mut Vec<u8>, quest_id: i32, actor_id: u64, end_text: u32) {
    bytes.extend(actor_id.to_le_bytes());
    put_i32(bytes, quest_id);
    bytes.extend(end_text.to_le_bytes());
    bytes.extend([0; 4]);
}

impl QuestEngine {
    /// Build the regional quest catalogue from the split authored data. The
    /// files provide definitions and NPC bindings; eligibility defaults and
    /// missing client fields are reconstructed server policy (R).
    pub fn from_split_content(
        directory: &Path,
        character_id: u64,
        eligibility: CharacterEligibility,
        state: Option<QuestState>,
    ) -> io::Result<Self> {
        let quest_dir = directory.join("quests");
        let npc_dir = directory.join("npcs");
        if !quest_dir.is_dir() || !npc_dir.is_dir() {
            return Err(invalid("split quest content requires quests/ and npcs/"));
        }
        let mut quest_paths: Vec<PathBuf> = fs::read_dir(&quest_dir)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<_>>()?;
        quest_paths.retain(|path| path.extension().is_some_and(|ext| ext == "json"));
        quest_paths.sort();
        let mut total_bytes = 0u64;
        let mut quests = Vec::new();
        let mut split_locale_fields = BTreeMap::new();
        let mut objective_text = BTreeMap::new();
        let mut disabled = BTreeSet::new();
        let mut runtime_goals = BTreeMap::new();
        let mut poi_locations = BTreeMap::new();
        for path in quest_paths {
            let file: SplitQuestFile = read_split_file(&path, &mut total_bytes)?;
            if file.schema_version != crate::content::SCHEMA_VERSION {
                return Err(invalid("unsupported split quest schema"));
            }
            for mut record in file.records {
                let object = record
                    .as_object_mut()
                    .ok_or_else(|| invalid("split quest record is not an object"))?;
                let enabled = object.remove("enabled").unwrap_or(Value::Bool(true));
                let enabled = enabled
                    .as_bool()
                    .ok_or_else(|| invalid("quest enabled must be boolean"))?;
                let mut goals = object
                    .remove("objectives")
                    .ok_or_else(|| invalid("quest objectives missing"))?;
                let goals = goals
                    .as_array_mut()
                    .ok_or_else(|| invalid("quest objectives must be a list"))?;
                if goals.len() > MAX_OBJECTIVES {
                    return Err(invalid("too many quest objectives"));
                }
                let mut standard = Vec::new();
                let mut extras = BTreeMap::new();
                let mut goal_text = Vec::new();
                for (index, goal) in goals.iter_mut().enumerate() {
                    let id = goal.get("id").and_then(Value::as_u64);
                    // Stripped before the typed goal parse, like the quest-level locale ids.
                    if let Some(text) = take_optional_locale_id(goal, "text_string_id")? {
                        let id = id.and_then(|id| u32::try_from(id).ok());
                        goal_text.push((id.ok_or_else(|| invalid("invalid objective id"))?, text));
                    }
                    match goal.get("kind").and_then(Value::as_str) {
                        Some(
                            "gather_resource" | "craft_recipe" | "finish_minigame" | "equip_item"
                            | "visit_position" | "reach_poi" | "client_reported",
                        ) => {
                            let (objective, location) = runtime_objective(goal.clone())?;
                            if let Some((id, zone, position, radius)) = location {
                                if poi_locations
                                    .insert(id, (zone, position, radius))
                                    .is_some_and(|old| old != (zone, position, radius))
                                {
                                    return Err(invalid("conflicting quest POI geometry"));
                                }
                            }
                            extras.insert(index, objective);
                        }
                        _ => standard.push(goal.clone()),
                    }
                }
                object.insert("objectives".into(), Value::Array(standard));
                let locale_fields = QuestLocaleFields {
                    intro_text_string_id: take_optional_locale_id(
                        &mut record,
                        "intro_text_string_id",
                    )?,
                    description_string_id: take_optional_locale_id(
                        &mut record,
                        "description_string_id",
                    )?,
                    end_text_string_id: take_optional_locale_id(&mut record, "end_text_string_id")?,
                };
                let quest: crate::content::Quest =
                    serde_json::from_value(record).map_err(|error| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("{}: {error}", path.display()),
                        )
                    })?;
                if !enabled {
                    disabled.insert(quest.id);
                }
                objective_text.extend(
                    goal_text
                        .into_iter()
                        .map(|(id, text)| ((quest.id, id), text)),
                );
                if !extras.is_empty() {
                    runtime_goals.insert(quest.id, extras);
                }
                if locale_fields.intro_text_string_id.is_some()
                    || locale_fields.description_string_id.is_some()
                    || locale_fields.end_text_string_id.is_some()
                {
                    split_locale_fields.insert(quest.id, locale_fields);
                }
                quests.push(quest);
            }
            if quests.len() > MAX_QUESTS {
                return Err(invalid("split quest catalogue exceeds 1,024 records"));
            }
        }
        if quests.is_empty() {
            return Err(invalid("split quest catalogue is empty"));
        }
        let quest_ids: BTreeSet<u32> = quests.iter().map(|quest| quest.id).collect();
        if quest_ids.len() != quests.len() {
            return Err(invalid("duplicate split quest id"));
        }
        let jobs: crate::content::VersionedFile<crate::content::Job> =
            read_split_file(&directory.join("jobs.json"), &mut total_bytes)?;
        if jobs.schema_version != crate::content::SCHEMA_VERSION {
            return Err(invalid("unsupported split job schema"));
        }
        let job_profile_ids: BTreeMap<String, i32> = jobs
            .records
            .iter()
            .filter_map(|job| {
                job.profile_id
                    .map(|profile_id| (job.id.clone(), profile_id))
            })
            .collect();
        let job_quests: BTreeSet<u32> = jobs
            .records
            .iter()
            .flat_map(|job| job.quest_ids.iter().copied())
            .collect();
        if job_quests.iter().any(|id| !quest_ids.contains(id))
            || quests.iter().any(|quest| !job_quests.contains(&quest.id))
        {
            return Err(invalid("split jobs reference missing quest definitions"));
        }

        let mut known_npcs = BTreeSet::new();
        let mut givers = BTreeMap::new();
        let mut turnins = BTreeMap::new();
        let mut npc_paths: Vec<PathBuf> = fs::read_dir(&npc_dir)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<_>>()?;
        npc_paths.retain(|path| path.extension().is_some_and(|ext| ext == "json"));
        npc_paths.sort();
        for path in npc_paths {
            let file: SplitNpcFile = read_split_file(&path, &mut total_bytes)?;
            if file.schema_version != 1 {
                return Err(invalid("unsupported split NPC schema"));
            }
            for npc in file.records {
                if npc.npc_id == 0 || !known_npcs.insert(npc.npc_id) {
                    return Err(invalid("duplicate or zero split NPC id"));
                }
                let giver_ids: BTreeSet<u32> = npc
                    .quest_ids_given
                    .iter()
                    .copied()
                    .chain(npc.quests_given.iter().filter_map(|quest| quest.quest_id))
                    .collect();
                for quest_id in giver_ids {
                    if !quest_ids.contains(&quest_id) {
                        return Err(invalid("split NPC references missing quest definition"));
                    }
                    if givers
                        .insert(quest_id, npc.npc_id)
                        .is_some_and(|previous| previous != npc.npc_id)
                    {
                        return Err(invalid("quest has multiple split giver NPCs"));
                    }
                }
                for quest_id in npc
                    .quests_accepted
                    .iter()
                    .filter_map(|quest| quest.quest_id)
                {
                    if !quest_ids.contains(&quest_id) {
                        return Err(invalid("split NPC references missing quest definition"));
                    }
                    if turnins.insert(quest_id, npc.npc_id).is_some() {
                        return Err(invalid("quest has multiple split turn-in NPCs"));
                    }
                }
            }
        }
        // NPC relationship rows are an independently authored mapping. Keep
        // their explicit bindings consistent with each typed quest record.
        for (quest_id, npc_id) in givers {
            let quest = quests
                .iter()
                .find(|quest| quest.id == quest_id)
                .expect("givers were validated against quest_ids");
            if quest.giver_npc_id != npc_id {
                return Err(invalid("split quest and NPC giver bindings disagree"));
            }
        }
        let mut definitions = quests
            .iter()
            .map(|quest| definition_from_content(quest, character_id, &job_profile_ids))
            .collect::<io::Result<Vec<_>>>()?;
        for definition in &mut definitions {
            if let Some(extras) = runtime_goals.remove(&definition.id) {
                let mut standard = definition.objectives.drain(..);
                let total = standard.len() + extras.len();
                let mut extras = extras;
                let objectives = (0..total)
                    .map(|index| {
                        extras
                            .remove(&index)
                            .unwrap_or_else(|| standard.next().expect("validated objective order"))
                    })
                    .collect();
                drop(standard);
                definition.objectives = objectives;
            }
            if let Some(npc_id) = turnins.get(&definition.id) {
                definition.turnin_npc_id = *npc_id;
            }
        }
        let step_dialogue = quests
            .iter()
            .flat_map(|quest| {
                quest.objectives.iter().filter_map(move |goal| match goal {
                    crate::content::ContentGoal::TalkToNpc {
                        id,
                        dialogue_string_id: Some(text),
                        ..
                    } => Some(((quest.id, *id), *text)),
                    _ => None,
                })
            })
            .collect();
        let mut engine =
            Self::new_with_disabled(character_id, definitions, eligibility, state, disabled)?;
        engine.poi_locations = poi_locations;
        engine.live_events = quests
            .iter()
            .filter(|quest| quest.live_event)
            .map(|quest| (quest.id, false))
            .collect();
        engine.index_location_quests();
        if let Some(id) = engine
            .state
            .pending_offer
            .filter(|id| !engine.is_enabled(*id))
        {
            engine.decline(id);
        }
        engine.job_profile_ids = job_profile_ids;
        engine.split_locale_fields = split_locale_fields;
        engine.step_dialogue = step_dialogue;
        engine.objective_text = objective_text;
        Ok(engine)
    }

    pub fn from_content_db(
        content: &ContentDb,
        character_id: u64,
        eligibility: CharacterEligibility,
        state: Option<QuestState>,
    ) -> io::Result<Self> {
        let job_profile_ids = content
            .jobs
            .iter()
            .filter_map(|job| job.profile_id.map(|profile| (job.id.clone(), profile)))
            .collect();
        let definitions = content
            .quests
            .iter()
            .map(|quest| definition_from_content(quest, character_id, &job_profile_ids))
            .collect::<io::Result<Vec<_>>>()?;
        Self::new(character_id, definitions, eligibility, state)
    }

    /// Transitional adapter for the reviewed 17-stage opening content. The old
    /// engine remains the parity oracle until the lead retires its bindings.
    pub fn from_opening(
        opening: &OpeningEngine,
        character_id: u64,
        eligibility: CharacterEligibility,
        state: Option<QuestState>,
    ) -> io::Result<Self> {
        let stages = opening.stages();
        let mut definitions = Vec::with_capacity(stages.len());
        for (index, stage) in stages.iter().enumerate() {
            let objectives = stage
                .objectives
                .iter()
                .map(|objective| QuestObjective {
                    id: objective.id,
                    required: objective.required,
                    description: objective.description.clone(),
                    kind: objective.trigger.clone().into(),
                })
                .collect();
            let prerequisite_quest_ids = stages[..index]
                .last()
                .map(|previous| vec![previous.id])
                .unwrap_or_default();
            let item_grants = stage
                .reward
                .items
                .iter()
                .map(|(definition_id, count)| ItemGrant {
                    definition_id: *definition_id,
                    count: *count,
                })
                .collect();
            let reward = RewardGrant {
                coins: stage.reward.coins,
                items: item_grants,
                job_xp: BTreeMap::new(),
                display: RewardDisplay {
                    source_guid: stage.giver_guid,
                    player_guid: character_id,
                    reward_kind: 0,
                    icon_id: 0,
                    name_id: stage.title_id.try_into().unwrap_or(0),
                    rows: Vec::<RewardBundleRow>::new(),
                },
            };
            definitions.push(QuestDefinition {
                id: stage.id,
                title: stage.title.clone(),
                description: stage.description.clone(),
                giver_npc_id: stage.giver_guid,
                turnin_npc_id: stage.turnin_guid,
                prerequisite_quest_ids,
                requirements: QuestRequirements::default(),
                ordered: stage.ordered,
                objectives,
                reward,
            });
        }
        let mut engine = Self::new(character_id, definitions, eligibility, state)?;
        engine.options.opening_order = Some(stages.iter().map(|stage| stage.id).collect());
        Ok(engine)
    }

    pub fn new(
        character_id: u64,
        definitions: Vec<QuestDefinition>,
        eligibility: CharacterEligibility,
        state: Option<QuestState>,
    ) -> io::Result<Self> {
        Self::new_with_disabled(
            character_id,
            definitions,
            eligibility,
            state,
            BTreeSet::new(),
        )
    }

    fn new_with_disabled(
        character_id: u64,
        definitions: Vec<QuestDefinition>,
        eligibility: CharacterEligibility,
        state: Option<QuestState>,
        disabled: BTreeSet<u32>,
    ) -> io::Result<Self> {
        if character_id == 0 || definitions.is_empty() || definitions.len() > MAX_QUESTS {
            return Err(invalid(
                "unsupported quest engine identity or definition count",
            ));
        }
        let mut indexed = BTreeMap::new();
        for definition in definitions {
            validate_definition(&definition)?;
            if indexed.insert(definition.id, definition).is_some() {
                return Err(invalid("duplicate quest definition"));
            }
        }
        for definition in indexed.values() {
            if definition
                .prerequisite_quest_ids
                .iter()
                .any(|id| !indexed.contains_key(id) || *id == definition.id)
            {
                return Err(invalid(
                    "quest prerequisite references unknown or self quest",
                ));
            }
        }
        // (R) A 45/7 claim must name exactly one objective, so client ids are unique.
        let mut client_ids = BTreeSet::new();
        for objective in indexed.values().flat_map(|d| &d.objectives) {
            if let ObjectiveKind::ClientReported { client_id, .. } = &objective.kind {
                if !client_ids.insert(*client_id) {
                    return Err(invalid("duplicate client objective id"));
                }
            }
        }
        let mut state = state.unwrap_or_else(|| QuestState {
            version: 1,
            character_id,
            quests: BTreeMap::new(),
            pending_offer: None,
            pending_turnin: None,
            pending_turnin_actor: None,
            selected_quest_id: None,
            auto_switch_selected: false,
        });
        if state.version != 1
            || state.character_id != character_id
            || state.quests.len() > MAX_QUESTS
        {
            return Err(invalid("saved quest identity, version or count is invalid"));
        }
        repair_counters(&mut state, &indexed, &disabled);
        validate_state(&state, character_id, &indexed, &disabled)?;
        let mut engine = Self {
            character_id,
            eligibility,
            definitions: indexed,
            split_locale_fields: BTreeMap::new(),
            step_dialogue: BTreeMap::new(),
            objective_text: BTreeMap::new(),
            job_profile_ids: BTreeMap::new(),
            dev_offer: None,
            disabled,
            live_events: BTreeMap::new(),
            location_quests: BTreeMap::new(),
            poi_locations: BTreeMap::new(),
            state,
            options: EngineOptions {
                opening_order: None,
            },
        };
        engine.index_location_quests();
        Ok(engine)
    }

    fn index_location_quests(&mut self) {
        self.location_quests.clear();
        for definition in self.definitions.values() {
            for objective in &definition.objectives {
                let zone = match &objective.kind {
                    ObjectiveKind::EnterArea { area_id } => Some(*area_id),
                    ObjectiveKind::VisitPosition { zone_id, .. } => Some(*zone_id),
                    ObjectiveKind::ReachPoi { poi_id } => {
                        self.poi_locations.get(poi_id).map(|p| p.0)
                    }
                    _ => None,
                };
                if let Some(zone) = zone {
                    self.location_quests
                        .entry(zone)
                        .or_default()
                        .insert(definition.id);
                }
            }
        }
    }

    pub fn state(&self) -> &QuestState {
        &self.state
    }

    pub fn eligibility(&self) -> &CharacterEligibility {
        &self.eligibility
    }

    pub fn set_eligibility(&mut self, eligibility: CharacterEligibility) {
        self.eligibility = eligibility;
    }

    pub fn job_profile_ids(&self) -> impl Iterator<Item = (&str, i32)> {
        self.job_profile_ids
            .iter()
            .map(|(job_id, profile_id)| (job_id.as_str(), *profile_id))
    }

    /// Build only the empty-reward 49/13 case. The native reader consumes five dwords, the
    /// 69-byte empty bundle base, and a float; meanings/defaults beyond the quest ID are (R).
    pub fn empty_reward_quest_end_packet(&self, quest_id: u32, actor_id: u64) -> Option<Vec<u8>> {
        let reward = &self.definitions.get(&quest_id)?.reward;
        if reward.coins != 0
            || !reward.items.is_empty()
            || reward.job_xp.values().any(|amount| *amount != 0)
            || !reward.display.rows.is_empty()
        {
            return None;
        }
        let mut bytes = header49(13);
        put_end_screen_ids(
            &mut bytes,
            quest_id.try_into().ok()?,
            actor_id,
            self.end_text_string_id(quest_id).unwrap_or(0),
        );
        bytes.extend([0; 69]);
        bytes.extend(0.0f32.to_le_bytes());
        Some(bytes)
    }

    /// The 49/1/49/13 ClientRewardBundle body for this quest's authored reward.
    ///
    /// Both screens call the client bundle reader `sub_8E7930` (`0x008e7930`): the 49/1
    /// reader `0x00c7b7a0` at packet offset 31 and the 49/13 reader `0x00c7b990` at
    /// packet offset 26. They share `encode_quest_reward_bundle`, so the displayed coins,
    /// summed job-XP stars and counted display rows cannot drift. The bundle is
    /// display-only: sending it grants nothing.
    pub fn quest_reward_bundle(&self, quest_id: u32) -> io::Result<Vec<u8>> {
        let reward = &self
            .definitions
            .get(&quest_id)
            .ok_or_else(|| invalid("unknown quest reward"))?
            .reward;
        encode_quest_reward_bundle(reward)
    }

    /// Build the populated quest end screen from the saved definition (R field defaults).
    pub fn quest_end_packet(&self, quest_id: u32, actor_id: u64) -> io::Result<Vec<u8>> {
        let reward = &self
            .definitions
            .get(&quest_id)
            .ok_or_else(|| invalid("unknown quest end definition"))?
            .reward;
        if let Some(packet) = self.empty_reward_quest_end_packet(quest_id, actor_id) {
            return Ok(packet);
        }

        let mut bytes = header49(13);
        put_end_screen_ids(
            &mut bytes,
            quest_id
                .try_into()
                .map_err(|_| invalid("quest id exceeds i32"))?,
            actor_id,
            self.end_text_string_id(quest_id).unwrap_or(0),
        );
        bytes.extend(encode_quest_reward_bundle(reward)?);
        bytes.extend(0.0f32.to_le_bytes());
        Ok(bytes)
    }

    /// Login journal order required by the client: 209/1 definitions, then 209/2 set.
    /// Field meanings and membership/self-data blocks remain separate integration work.
    pub fn journal_login_packets(
        definitions: &startup_world::JournalDefinitions,
        entries: &[(u32, u32)],
    ) -> Vec<Vec<u8>> {
        vec![
            startup_world::build_journal_definitions(definitions),
            startup_messages::build_journal_set(entries),
        ]
    }

    pub fn progress(&self, quest_id: u32) -> Option<&QuestProgress> {
        self.state.quests.get(&quest_id)
    }

    /// Unfinished goals of an active quest that can be worked on now: all of them, or only
    /// the first one when the quest is ordered (R; the same rule as `accept`). Empty for a
    /// quest that is not active, which keeps a forged or stale selection from tracking.
    pub fn current_objectives(&self, quest_id: u32) -> Vec<&QuestObjective> {
        let (Some(definition), Some(progress)) = (
            self.definitions.get(&quest_id),
            self.state.quests.get(&quest_id),
        ) else {
            return Vec::new();
        };
        if progress.status != QuestStatus::Active {
            return Vec::new();
        }
        let open = definition
            .objectives
            .iter()
            .zip(&progress.counts)
            .filter(|(objective, count)| **count < objective.required)
            .map(|(objective, _)| objective);
        if definition.ordered {
            open.take(1).collect()
        } else {
            open.collect()
        }
    }

    /// Zone and position of a reach-POI objective's target.
    pub fn poi_location(&self, poi_id: u32) -> Option<(u32, [f32; 3])> {
        self.poi_locations
            .get(&poi_id)
            .map(|(zone, position, _)| (*zone, *position))
    }

    pub fn title_string_id(&self, quest_id: u32) -> Option<u32> {
        self.definitions
            .get(&quest_id)
            .map(|definition| definition.reward.display.name_id)
            .filter(|id| *id != 0)
            .map(|id| id as u32)
    }

    pub fn intro_text_string_id(&self, quest_id: u32) -> Option<u32> {
        self.split_locale_fields
            .get(&quest_id)
            .and_then(|fields| fields.intro_text_string_id)
    }

    /// What the NPC says for a talk step this outcome just completed, if any.
    pub fn step_dialogue_for(&self, outcome: &QuestOutcome) -> Option<u32> {
        outcome.outbound.iter().find_map(|message| match message {
            OutboundMessage::Quest(QuestMessage::ObjectiveComplete {
                quest_id,
                objective_id,
                ..
            }) => self.step_dialogue.get(&(*quest_id, *objective_id)).copied(),
            _ => None,
        })
    }

    pub fn end_text_string_id(&self, quest_id: u32) -> Option<u32> {
        self.split_locale_fields
            .get(&quest_id)
            .and_then(|fields| fields.end_text_string_id)
    }

    pub fn description_string_id(&self, quest_id: u32) -> Option<u32> {
        self.split_locale_fields
            .get(&quest_id)
            .and_then(|fields| fields.description_string_id)
    }

    pub fn quest_ids_for_giver(&self, npc_id: u64) -> impl Iterator<Item = u32> + '_ {
        self.definitions
            .values()
            .filter(move |definition| definition.giver_npc_id == npc_id)
            .map(|definition| definition.id)
    }

    /// Serialize the client-reader 49/3 quest record (`0093c840`, `00c7cc80`).
    /// Locale fields absent from this content set and reward display rows use
    /// zero/empty placeholders (R); authoritative reward grants remain typed.
    pub fn quest_add_packet(&self, quest_id: u32) -> io::Result<Vec<u8>> {
        let definition = self
            .definitions
            .get(&quest_id)
            .ok_or_else(|| invalid("unknown quest add definition"))?;
        let progress = self
            .state
            .quests
            .get(&quest_id)
            .ok_or_else(|| invalid("quest add has no saved progress"))?;
        let mut bytes = header49(3);
        put_i32(
            &mut bytes,
            quest_id
                .try_into()
                .map_err(|_| invalid("quest id exceeds i32"))?,
        );
        put_i32(&mut bytes, definition.reward.display.name_id);
        put_i32(&mut bytes, 0);
        put_i32(&mut bytes, 0);
        bytes.push(0);
        bytes.extend(0i64.to_le_bytes());
        put_i32(&mut bytes, 0);
        bytes.push(0);
        bytes.extend(0.0f32.to_le_bytes());
        bytes.extend([0; 69]);
        put_i32(&mut bytes, definition.objectives.len() as i32);
        // (R) A goal without authored text shows the quest description, else its title.
        let fallback_text = self
            .description_string_id(quest_id)
            .or(self.title_string_id(quest_id))
            .unwrap_or(0);
        for (index, objective) in definition.objectives.iter().enumerate() {
            let text = self
                .objective_text
                .get(&(quest_id, objective.id))
                .copied()
                .unwrap_or(fallback_text);
            // Key, definition id, name and description locale ids (`008fd770`). The client
            // rekeys objectives by definition id (`00baf380` -> `00bab950`) and 49/8/9/10 find
            // them by it (`00c7a910`), so it is the objective id. The helper's goal line is
            // the name (`00cf7fd0` column 0; id 0 prints "<STRING 0 NOT FOUND>"); description
            // repeats it (R), as the opening journal's goal labels do.
            for value in [objective.id, objective.id, text, text] {
                put_i32(
                    &mut bytes,
                    value
                        .try_into()
                        .map_err(|_| invalid("objective id exceeds i32"))?,
                );
            }
            bytes.push(0);
            bytes.extend([0; 69]);
            for value in [0, progress.counts[index], objective.required, 0] {
                put_i32(
                    &mut bytes,
                    value
                        .try_into()
                        .map_err(|_| invalid("objective count exceeds i32"))?,
                );
            }
            bytes.push(0);
            put_i32(&mut bytes, 0);
        }
        put_i32(&mut bytes, 0);
        bytes.push(0);
        bytes.push(0);
        bytes.push(u8::from(self.state.selected_quest_id == Some(quest_id)));
        Ok(bytes)
    }

    /// Rehydrate every active regional record using the accept-path QuestAdd and objective wire
    /// sequence. `excluded` contains opening IDs, whose journal owns their login synchronization.
    pub fn active_login_packets(&self, excluded: &BTreeSet<u32>) -> io::Result<Vec<Vec<u8>>> {
        let active: Vec<u32> = self
            .state
            .quests
            .iter()
            .filter_map(|(&id, progress)| {
                (progress.status == QuestStatus::Active && !excluded.contains(&id)).then_some(id)
            })
            .collect();
        let mut packets = Vec::new();
        for quest_id in active {
            let definition = &self.definitions[&quest_id];
            let progress = &self.state.quests[&quest_id];
            packets.push(self.quest_add_packet(quest_id)?);

            // (R) This mirrors accept(): all unordered goals or the first unfinished ordered goal
            // become active. The full 49/3 record already carries each saved current count.
            let next_ordered = definition
                .ordered
                .then(|| {
                    progress
                        .counts
                        .iter()
                        .zip(&definition.objectives)
                        .position(|(count, objective)| *count < objective.required)
                })
                .flatten();
            for (index, objective) in definition.objectives.iter().enumerate() {
                if progress.counts[index] >= objective.required
                    || (definition.ordered && next_ordered != Some(index))
                {
                    continue;
                }
                packets.push(
                    QuestMessage::ObjectiveActivated {
                        quest_id,
                        objective_id: objective.id,
                        required: objective.required,
                    }
                    .encode_exact()
                    .ok_or_else(|| invalid("quest objective activation cannot be encoded"))?,
                );
            }
            for (index, objective) in definition.objectives.iter().enumerate() {
                let current = progress.counts[index];
                if current == 0 {
                    continue;
                }
                let percent = overall_progress(definition, &progress.counts);
                packets.push(
                    QuestMessage::ObjectiveUpdate {
                        quest_id,
                        objective_id: objective.id,
                        current,
                        progress: percent,
                    }
                    .encode_exact()
                    .ok_or_else(|| invalid("quest objective progress cannot be encoded"))?,
                );
                if current == objective.required {
                    packets.push(
                        QuestMessage::ObjectiveComplete {
                            quest_id,
                            objective_id: objective.id,
                            progress: percent,
                        }
                        .encode_exact()
                        .ok_or_else(|| invalid("quest objective completion cannot be encoded"))?,
                    );
                }
            }
        }
        Ok(packets)
    }

    /// The actor that must be visible before the pending 49/13 can open, if one is pending.
    pub fn pending_turnin_actor(&self) -> Option<u64> {
        let quest_id = self.state.pending_turnin?;
        let fallback = self.definitions.get(&quest_id)?.turnin_npc_id;
        Some(self.state.turnin_actor(fallback))
    }

    /// Persist the client-selected active quest after decoding 47/12 (R policy).
    pub fn select_quest(&mut self, quest_id: u32) -> QuestOutcome {
        if self
            .state
            .quests
            .get(&quest_id)
            .is_none_or(|progress| progress.status != QuestStatus::Active)
        {
            return denied(QuestDenial::Unavailable);
        }
        if self.state.selected_quest_id == Some(quest_id) {
            return QuestOutcome::default();
        }
        self.state.selected_quest_id = Some(quest_id);
        QuestOutcome {
            changed: true,
            ..QuestOutcome::default()
        }
    }

    /// Persist the client's auto-switch flag (`manager+0x10f8`). The 47/13 wire byte is its
    /// inverse (`00c7c5d0` sends `sete`), so `quest_journal::handle` passes `!locked` (R policy).
    pub fn set_auto_switch_selected(&mut self, auto_switch: bool) -> QuestOutcome {
        if self.state.auto_switch_selected == auto_switch {
            return QuestOutcome::default();
        }
        self.state.auto_switch_selected = auto_switch;
        QuestOutcome {
            changed: true,
            ..QuestOutcome::default()
        }
    }

    /// Offer only after server-side prerequisites, level/job, membership and limit checks (R).
    pub fn offer(&mut self, quest_id: u32, giver_npc_id: u64) -> QuestOutcome {
        if let Some(denial) = self.offer_denial(quest_id, giver_npc_id) {
            let mut outcome = denied(denial.clone());
            if denial == QuestDenial::Limit {
                outcome
                    .outbound
                    .push(OutboundMessage::Quest(QuestMessage::QuestLimit {
                        member: self.eligibility.member,
                    }));
            }
            return outcome;
        }
        let definition = &self.definitions[&quest_id];
        self.state.quests.insert(
            quest_id,
            QuestProgress {
                status: QuestStatus::Offered,
                counts: vec![0; definition.objectives.len()],
            },
        );
        self.state.pending_offer = Some(quest_id);
        self.dev_offer = None;
        QuestOutcome {
            changed: true,
            outbound: vec![OutboundMessage::Quest(QuestMessage::QuestInfo { quest_id })],
            ..QuestOutcome::default()
        }
    }

    /// Developer `realm offer` only: make a quest acceptable without the giver
    /// and prerequisite checks, so a test can start mid-chain quests directly.
    pub fn dev_mark_offered(&mut self, quest_id: u32) -> bool {
        let Some(definition) = self.definitions.get(&quest_id) else {
            return false;
        };
        if definition.objectives.is_empty() || self.disabled.contains(&quest_id) {
            return false;
        }
        if self.state.quests.get(&quest_id).is_some_and(|progress| {
            matches!(
                progress.status,
                QuestStatus::Active | QuestStatus::Completed
            )
        }) {
            return false;
        }
        self.state.quests.insert(
            quest_id,
            QuestProgress {
                status: QuestStatus::Offered,
                counts: vec![0; definition.objectives.len()],
            },
        );
        self.state.pending_offer = Some(quest_id);
        self.dev_offer = Some(quest_id);
        true
    }

    pub fn can_offer(&self, quest_id: u32, giver_npc_id: u64) -> bool {
        self.offer_denial(quest_id, giver_npc_id).is_none()
    }

    /// The NPC that gives `quest_id`; a breadcrumb start offers through this giver (R).
    pub fn giver_npc_id(&self, quest_id: u32) -> Option<u64> {
        self.definitions.get(&quest_id).map(|d| d.giver_npc_id)
    }

    /// The NPC that accepts `quest_id`'s completed turn-in.
    pub fn turnin_npc_id(&self, quest_id: u32) -> Option<u64> {
        self.definitions.get(&quest_id).map(|d| d.turnin_npc_id)
    }

    pub fn is_enabled(&self, quest_id: u32) -> bool {
        !self.disabled.contains(&quest_id)
            && self
                .definitions
                .get(&quest_id)
                .is_some_and(|d| !d.objectives.is_empty())
    }

    /// T418: every `live_event` quest and whether it is open.
    pub fn live_events(&self) -> &BTreeMap<u32, bool> {
        &self.live_events
    }

    /// T418: open exactly the live events in `open` (the live world's
    /// `events.json`). Returns the ids that are not live events; they are ignored.
    pub fn set_open_events(&mut self, open: &BTreeSet<u32>) -> Vec<u32> {
        for (id, is_open) in &mut self.live_events {
            *is_open = open.contains(id);
        }
        open.iter()
            .copied()
            .filter(|id| !self.live_events.contains_key(id))
            .collect()
    }

    fn event_closed(&self, quest_id: u32) -> bool {
        self.live_events.get(&quest_id) == Some(&false)
    }

    fn offer_denial(&self, quest_id: u32, giver_npc_id: u64) -> Option<QuestDenial> {
        let Some(definition) = self.definitions.get(&quest_id) else {
            return Some(QuestDenial::Unavailable);
        };
        // (R) Incomplete content stays available to editors and saved state,
        // but cannot be offered, even when `enabled` is accidentally true.
        // T418: neither is a live event an admin has not opened.
        if definition.objectives.is_empty()
            || self.disabled.contains(&quest_id)
            || self.event_closed(quest_id)
        {
            return Some(QuestDenial::Unavailable);
        }
        if definition.giver_npc_id != giver_npc_id {
            return Some(QuestDenial::WrongGiver);
        }
        if self
            .state
            .pending_offer
            .is_some_and(|pending| pending != quest_id)
            || self.state.quests.get(&quest_id).is_some_and(|progress| {
                matches!(
                    progress.status,
                    QuestStatus::Offered | QuestStatus::Active | QuestStatus::Completed
                )
            })
        {
            return Some(QuestDenial::Unavailable);
        }
        if let Some(denial) = self.requirement_denial(definition) {
            return Some(denial);
        }
        let limit = if self.eligibility.member {
            self.eligibility.member_quest_limit
        } else {
            self.eligibility.nonmember_quest_limit
        };
        let occupied = self
            .state
            .quests
            .values()
            .filter(|progress| {
                matches!(progress.status, QuestStatus::Offered | QuestStatus::Active)
            })
            .count();
        (occupied >= limit).then_some(QuestDenial::Limit)
    }

    /// Accept a matching 49/2 reply after a pending 49/1 offer.
    pub fn accept(&mut self, quest_id: u32) -> QuestOutcome {
        if self.state.pending_offer != Some(quest_id) || !self.is_enabled(quest_id) {
            return denied(QuestDenial::Unavailable);
        }
        let developer_offer = self.dev_offer == Some(quest_id);
        // T418: an offer shown before the admin closed the event is not accepted.
        // The client forgets its offer once it sends this accept (0x00a97d70
        // sets the offer id to -1), so the refusal withdraws the offer like a
        // decline; a kept pending offer would block every later offer.
        if self.event_closed(quest_id) && !developer_offer {
            let mut outcome = self.decline(quest_id);
            outcome.denial = Some(QuestDenial::Unavailable);
            return outcome;
        }
        let definition = &self.definitions[&quest_id];
        if let Some(denial) = self
            .requirement_denial(definition)
            .filter(|_| !developer_offer)
        {
            return denied(denial);
        }
        let active_limit = if self.eligibility.member {
            self.eligibility.member_quest_limit
        } else {
            self.eligibility.nonmember_quest_limit
        };
        if self
            .state
            .quests
            .values()
            .filter(|p| matches!(p.status, QuestStatus::Active))
            .count()
            >= active_limit
        {
            let mut outcome = denied(QuestDenial::Limit);
            outcome
                .outbound
                .push(OutboundMessage::Quest(QuestMessage::QuestLimit {
                    member: self.eligibility.member,
                }));
            return outcome;
        }
        self.dev_offer = None;
        self.state.pending_offer = None;
        let progress = self
            .state
            .quests
            .get_mut(&quest_id)
            .expect("offered quest exists");
        progress.status = QuestStatus::Active;
        let active_ids: Vec<_> = definition
            .objectives
            .iter()
            .enumerate()
            .filter(|(index, _)| !definition.ordered || *index == 0)
            .map(|(_, objective)| objective.id)
            .collect();
        let mut packets = vec![QuestMessage::QuestAdd { quest_id }];
        packets.extend(active_ids.into_iter().map(|objective_id| {
            let objective = definition
                .objectives
                .iter()
                .find(|item| item.id == objective_id)
                .unwrap();
            QuestMessage::ObjectiveActivated {
                quest_id,
                objective_id,
                required: objective.required,
            }
        }));
        let mut outcome = QuestOutcome {
            changed: true,
            outbound: packets.into_iter().map(OutboundMessage::Quest).collect(),
            ..QuestOutcome::default()
        };
        if self.options.opening_order.is_some() {
            outcome.messages.push(format!(
                "Accepted: {}. {}",
                definition.title, definition.description
            ));
        }
        outcome
    }

    /// Decline is client-to-server 49/2 with accept=0; this clears the pending offer silently.
    pub fn decline(&mut self, quest_id: u32) -> QuestOutcome {
        if self.state.pending_offer != Some(quest_id) {
            return denied(QuestDenial::Unavailable);
        }
        self.state.pending_offer = None;
        self.dev_offer = None;
        self.state.quests.remove(&quest_id);
        QuestOutcome {
            changed: true,
            ..QuestOutcome::default()
        }
    }

    /// Progress only from authoritative server events. NPC range/ownership and event authenticity
    /// remain the caller's responsibility (R).
    pub fn on_event(&mut self, event: QuestEvent) -> io::Result<QuestOutcome> {
        validate_event(&event)?;
        let mut outcome = QuestOutcome::default();
        let ids: Vec<u32> = self
            .state
            .quests
            .iter()
            .filter(|(_, progress)| progress.status == QuestStatus::Active)
            .filter(|(id, _)| match &event {
                QuestEvent::VisitPosition { zone_id, .. } => self
                    .location_quests
                    .get(zone_id)
                    .is_some_and(|ids| ids.contains(id)),
                _ => true,
            })
            .map(|(id, _)| *id)
            .collect();
        for quest_id in ids {
            let definition = self.definitions[&quest_id].clone();
            let Some(progress) = self.state.quests.get_mut(&quest_id) else {
                continue;
            };
            if progress.status != QuestStatus::Active {
                continue;
            }
            let next_ordered = if definition.ordered {
                progress
                    .counts
                    .iter()
                    .zip(&definition.objectives)
                    .position(|(count, objective)| *count < objective.required)
            } else {
                None
            };
            let mut activate_next = None;
            for (index, objective) in definition.objectives.iter().enumerate() {
                if progress.counts[index] >= objective.required
                    || (definition.ordered && next_ordered != Some(index))
                {
                    continue;
                }
                let increment = match (&objective.kind, &event) {
                    (
                        ObjectiveKind::EnterArea { area_id },
                        QuestEvent::VisitPosition { zone_id, .. },
                    ) if area_id == zone_id => 1,
                    (
                        ObjectiveKind::ReachPoi { poi_id },
                        QuestEvent::VisitPosition { zone_id, position },
                    ) => u32::from(self.poi_locations.get(poi_id).is_some_and(
                        |(zone, target, radius)| {
                            zone == zone_id && within(*target, *position, *radius)
                        },
                    )),
                    _ => event_matches(&objective.kind, &event),
                };
                if increment == 0 {
                    continue;
                }
                progress.counts[index] = progress.counts[index]
                    .saturating_add(increment)
                    .min(objective.required);
                outcome.changed = true;
                let percent = overall_progress(&definition, &progress.counts);
                outcome.messages.push(format!(
                    "{}: {}/{}.",
                    objective.description, progress.counts[index], objective.required
                ));
                outcome
                    .outbound
                    .push(OutboundMessage::Quest(QuestMessage::ObjectiveUpdate {
                        quest_id,
                        objective_id: objective.id,
                        current: progress.counts[index],
                        progress: percent,
                    }));
                if progress.counts[index] == objective.required {
                    outcome.outbound.push(OutboundMessage::Quest(
                        QuestMessage::ObjectiveComplete {
                            quest_id,
                            objective_id: objective.id,
                            progress: percent,
                        },
                    ));
                    if definition.ordered {
                        activate_next = definition
                            .objectives
                            .iter()
                            .enumerate()
                            .find(|(candidate, goal)| {
                                *candidate > index && progress.counts[*candidate] < goal.required
                            })
                            .map(|(_, goal)| (goal.id, goal.required));
                    }
                }
            }
            if let Some((objective_id, required)) = activate_next {
                outcome
                    .outbound
                    .push(OutboundMessage::Quest(QuestMessage::ObjectiveActivated {
                        quest_id,
                        objective_id,
                        required,
                    }));
            }
            let ready = !definition.objectives.is_empty()
                && progress
                    .counts
                    .iter()
                    .zip(&definition.objectives)
                    .all(|(count, objective)| *count == objective.required);
            let named_objective_target = match &event {
                QuestEvent::TalkToNpc { npc_id } => definition.objectives.iter().any(|objective| {
                    matches!(
                        &objective.kind,
                        ObjectiveKind::TalkToNpc { npc_id: target } if target == npc_id
                    )
                }),
                _ => false,
            };
            if ready
                && self.state.pending_turnin.is_none()
                && (event_is_talk_to(&event, definition.turnin_npc_id) || named_objective_target)
            {
                // Both conditions are Talk-only, so this records the NPC the player talked to.
                if let QuestEvent::TalkToNpc { npc_id: actor_id } = &event {
                    self.state.open_pending_turnin(quest_id, *actor_id);
                    outcome
                        .outbound
                        .push(OutboundMessage::Quest(QuestMessage::QuestEnd {
                            quest_id,
                            actor_id: *actor_id,
                        }));
                }
            }
        }
        Ok(outcome)
    }

    /// Start the 49/13 end-screen step. The 49/14 reply must call `finish_turnin`.
    /// Repeating it at the actor that opened an already pending screen re-sends the
    /// end screen without changing state (end-screen recovery).
    pub fn begin_turnin(&mut self, quest_id: u32, npc_id: u64) -> QuestOutcome {
        let Some(definition) = self.definitions.get(&quest_id) else {
            return denied(QuestDenial::Unavailable);
        };
        if npc_id != definition.turnin_npc_id {
            return denied(QuestDenial::WrongTurnin);
        }
        if self.state.pending_turnin.is_some() {
            return self.reopen_turnin(quest_id, npc_id);
        }
        self.begin_ready_turnin(quest_id, npc_id)
    }

    /// Begin a completed quest turn-in after NPC content identifies the named target (R policy).
    /// A Talk that already opened the screen goes through [`Self::on_event`] alone; a second
    /// explicit begin must not queue another 49/13.
    pub fn begin_named_target_turnin(&mut self, quest_id: u32, npc_id: u64) -> QuestOutcome {
        if self.state.pending_turnin.is_some() {
            return denied(QuestDenial::Unavailable);
        }
        self.begin_ready_turnin(quest_id, npc_id)
    }

    /// Re-send the already pending screen for `quest_id` when `npc_id` is the actor that
    /// opened it. A different actor or quest returns a denial with no outbound (recovery).
    pub fn reopen_turnin(&self, quest_id: u32, npc_id: u64) -> QuestOutcome {
        if self.state.pending_turnin != Some(quest_id) {
            return denied(QuestDenial::Unavailable);
        }
        let Some(actor_id) = self.pending_turnin_actor() else {
            return denied(QuestDenial::Unavailable);
        };
        if npc_id != actor_id {
            return denied(QuestDenial::Unavailable);
        }
        QuestOutcome {
            outbound: vec![OutboundMessage::Quest(QuestMessage::QuestEnd {
                quest_id,
                actor_id,
            })],
            ..QuestOutcome::default()
        }
    }

    fn begin_ready_turnin(&mut self, quest_id: u32, npc_id: u64) -> QuestOutcome {
        if self.state.pending_turnin.is_some() {
            return denied(QuestDenial::Unavailable);
        }
        let Some(definition) = self.definitions.get(&quest_id) else {
            return denied(QuestDenial::Unavailable);
        };
        let Some(progress) = self.state.quests.get(&quest_id) else {
            return denied(QuestDenial::Unavailable);
        };
        if definition.objectives.is_empty()
            || progress.status != QuestStatus::Active
            || !is_complete(definition, &progress.counts)
        {
            return denied(QuestDenial::Incomplete);
        }
        self.state.open_pending_turnin(quest_id, npc_id);
        QuestOutcome {
            changed: true,
            outbound: vec![OutboundMessage::Quest(QuestMessage::QuestEnd {
                quest_id,
                actor_id: npc_id,
            })],
            ..QuestOutcome::default()
        }
    }

    /// Settle one pending 49/14 turn-in through the idempotent rewards transaction (R key policy).
    pub fn finish_turnin(
        &mut self,
        quest_id: u32,
        reward_state: &RewardState,
    ) -> io::Result<(QuestOutcome, RewardState)> {
        if self.state.pending_turnin != Some(quest_id) {
            return Ok((denied(QuestDenial::NoPendingTurnin), reward_state.clone()));
        }
        let definition = self
            .definitions
            .get(&quest_id)
            .ok_or_else(|| invalid("pending quest definition disappeared"))?;
        let grant_key = format!("quest:{}:{}", self.character_id, quest_id);
        let (next_reward_state, reward_packets) =
            match rewards::apply_grant(reward_state, &definition.reward, &grant_key) {
                Ok(granted) => granted,
                Err(failure) => {
                    return Ok((self.refuse_turnin(quest_id, failure), reward_state.clone()));
                }
            };
        self.state.clear_pending_turnin();
        self.state
            .quests
            .get_mut(&quest_id)
            .expect("pending quest exists")
            .status = QuestStatus::Completed;
        if self.state.selected_quest_id == Some(quest_id) {
            self.state.selected_quest_id = None;
        }
        let completed_count = self
            .state
            .quests
            .values()
            .filter(|progress| progress.status == QuestStatus::Completed)
            .count() as u32;
        let mut outcome = QuestOutcome {
            changed: true,
            outbound: reward_packets
                .into_iter()
                .map(OutboundMessage::Reward)
                .chain([
                    OutboundMessage::Quest(QuestMessage::QuestComplete { quest_id }),
                    OutboundMessage::Quest(QuestMessage::CompletedQuestCount {
                        count: completed_count,
                    }),
                ])
                .collect(),
            rewards: vec![reward_summary(quest_id, &definition.reward)],
            messages: vec![format!("Completed: {}.", definition.title)],
            ..QuestOutcome::default()
        };
        if let Some(order) = &self.options.opening_order {
            let next = order
                .iter()
                .skip_while(|id| **id != quest_id)
                .nth(1)
                .copied();
            if let Some(next_id) = next {
                outcome.messages.push(format!(
                    "Next: {}. Talk to its guide.",
                    self.definitions[&next_id].title
                ));
            } else {
                outcome.messages.push(
                    "Opening story complete. Explore your discoveries and return safely.".into(),
                );
            }
        }
        Ok((outcome, next_reward_state))
    }

    /// A refused grant pays nothing and completes nothing; it is a per-player denial, never a
    /// session-ending error (T382). Only the locked profile releases the end screen; every other
    /// refusal keeps `pending_turnin` for a retry or a 26/23 abandon.
    fn refuse_turnin(&mut self, quest_id: u32, failure: GrantFailure) -> QuestOutcome {
        let denial = match failure {
            // (R) The job gate is an offer gate, so a save written before this reward row gained
            // its job XP can hold an active quest whose reward names a profile the character has
            // not unlocked. Release the screen so the player can unlock the job and retry, or
            // abandon (F249 review).
            GrantFailure::LockedProfile => {
                self.state.clear_pending_turnin();
                return QuestOutcome {
                    changed: true,
                    denial: Some(QuestDenial::Job),
                    ..QuestOutcome::default()
                };
            }
            GrantFailure::BagFull => QuestDenial::NeedSpace,
            // Always the player's wallet or XP: `validate_definition` refuses a reward row past
            // the wire field when the engine loads.
            GrantFailure::WireRange => QuestDenial::RewardLimit,
            GrantFailure::UnknownItem
            | GrantFailure::BadItem
            | GrantFailure::Shape
            | GrantFailure::InvalidKey => {
                // Not the player's fault: one bounded line for the operator, like
                // `quest_save_repair` (fixed text, a u32 id and the class; no identity or chat).
                eprintln!("quest_turnin_reward_refused quest={quest_id} reason={failure:?}");
                QuestDenial::RewardData
            }
        };
        denied(denial)
    }

    /// 26/0x17 command result: remove active quest without any grant. A pending end screen for
    /// this quest goes with it, so a reward that cannot be paid never traps the quest (T382).
    pub fn abandon(&mut self, quest_id: u32) -> QuestOutcome {
        let Some(progress) = self.state.quests.get_mut(&quest_id) else {
            return denied(QuestDenial::Unavailable);
        };
        if progress.status != QuestStatus::Active {
            return denied(QuestDenial::Unavailable);
        }
        progress.status = QuestStatus::Abandoned;
        if self.state.pending_turnin == Some(quest_id) {
            self.state.clear_pending_turnin();
        }
        if self.state.selected_quest_id == Some(quest_id) {
            self.state.selected_quest_id = None;
        }
        QuestOutcome {
            changed: true,
            outbound: vec![OutboundMessage::Quest(QuestMessage::QuestAbandoned {
                quest_id,
            })],
            ..QuestOutcome::default()
        }
    }

    fn requirement_denial(&self, definition: &QuestDefinition) -> Option<QuestDenial> {
        if definition.prerequisite_quest_ids.iter().any(|id| {
            self.state
                .quests
                .get(id)
                .is_none_or(|progress| progress.status != QuestStatus::Completed)
        }) {
            return Some(QuestDenial::Prerequisite);
        }
        if definition
            .requirements
            .player_level
            .is_some_and(|level| self.eligibility.player_level < level)
        {
            return Some(QuestDenial::Level);
        }
        if let Some((job, level)) = &definition.requirements.job {
            if self.eligibility.job_levels.get(job).copied().unwrap_or(0) < *level {
                return Some(QuestDenial::Job);
            }
        }
        if definition.requirements.members_only && !self.eligibility.member {
            return Some(QuestDenial::Membership);
        }
        None
    }
}

fn definition_from_content(
    quest: &crate::content::Quest,
    character_id: u64,
    job_profile_ids: &BTreeMap<String, i32>,
) -> io::Result<QuestDefinition> {
    if quest
        .min_level
        .is_some_and(|level| !(1..=20).contains(&level))
    {
        return Err(invalid("quest min_level must be in 1..=20"));
    }
    // (R) An explicit job gate uses that job's level, not another job/avatar level.
    let job = quest
        .job
        .map(|profile| {
            job_profile_ids
                .iter()
                .find(|(_, id)| **id == profile && profile > 0)
                .map(|(job, _)| (job.clone(), quest.min_level.unwrap_or(1)))
                .ok_or_else(|| invalid("quest job must reference a known positive profile id"))
        })
        .transpose()?;
    for goal in &quest.objectives {
        match goal {
            ContentGoal::InteractActor {
                actor_guid, source, ..
            } => {
                if *actor_guid == 0 || source.is_empty() {
                    return Err(invalid(
                        "interact_actor objective requires a positive actor and provenance",
                    ));
                }
            }
            ContentGoal::CompleteDungeon {
                encounter_id,
                source,
                ..
            } => {
                if *encounter_id == 0 || source.is_empty() {
                    return Err(invalid(
                        "complete_dungeon objective requires a positive encounter and provenance",
                    ));
                }
            }
            ContentGoal::JobLevel {
                profile_id,
                level,
                source,
                ..
            } => {
                if *profile_id <= 0
                    || !job_profile_ids.values().any(|known| known == profile_id)
                    || !(1..=20).contains(level)
                    || source.is_empty()
                {
                    return Err(invalid(
                        "job_level objective requires a known positive profile, level 1..=20 and provenance",
                    ));
                }
            }
            _ => {}
        }
    }
    let objectives = quest
        .objectives
        .iter()
        .map(|goal| match goal {
            ContentGoal::Defeat {
                id,
                model_id,
                count,
                ..
            } => QuestObjective {
                id: *id,
                required: *count,
                description: format!("Objective {}", id),
                // (R) Content currently supplies model_id; caller must bind it to its actor-kind event.
                kind: ObjectiveKind::KillActorKind {
                    actor_kind_id: *model_id,
                },
            },
            ContentGoal::TalkToNpc { id, npc_id, .. } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::TalkToNpc { npc_id: *npc_id },
            },
            ContentGoal::CollectItem {
                id,
                item_id,
                quantity,
                ..
            } => QuestObjective {
                id: *id,
                required: *quantity,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::CollectItem { item_id: *item_id },
            },
            ContentGoal::ReachPoi { id, poi_id, .. } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::ReachPoi { poi_id: *poi_id },
            },
            ContentGoal::VisitZone { id, zone_id, .. } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::EnterArea { area_id: *zone_id },
            },
            // W104 kinds, mapped exactly as `runtime_objective` maps them.
            ContentGoal::GatherResource {
                id,
                resource_id,
                quantity,
                ..
            } => QuestObjective {
                id: *id,
                required: *quantity,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::GatherResource {
                    resource_id: *resource_id,
                },
            },
            ContentGoal::CraftRecipe {
                id,
                recipe_id,
                quantity,
                ..
            } => QuestObjective {
                id: *id,
                required: *quantity,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::CraftRecipe {
                    recipe_id: *recipe_id,
                },
            },
            ContentGoal::FinishMinigame {
                id,
                minigame_id,
                count,
                ..
            } => QuestObjective {
                id: *id,
                required: *count,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::FinishMinigame {
                    minigame_id: *minigame_id,
                },
            },
            ContentGoal::EquipItem {
                id, definition_id, ..
            } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::EquipItem {
                    definition_id: *definition_id,
                },
            },
            ContentGoal::VisitPosition {
                id,
                zone_id,
                position,
                radius,
                ..
            } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::VisitPosition {
                    zone_id: *zone_id,
                    position: *position,
                    radius: *radius,
                },
            },
            ContentGoal::ClientReported {
                id,
                client_id,
                trigger,
                ..
            } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::ClientReported {
                    client_id: *client_id,
                    trigger: trigger.clone(),
                },
            },
            ContentGoal::InteractActor { id, actor_guid, .. } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::InteractNpc {
                    npc_id: *actor_guid,
                },
            },
            ContentGoal::CompleteDungeon {
                id, encounter_id, ..
            } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::CompleteDungeon {
                    encounter_id: *encounter_id,
                },
            },
            ContentGoal::JobLevel {
                id,
                profile_id,
                level,
                ..
            } => QuestObjective {
                id: *id,
                required: 1,
                description: format!("Objective {}", id),
                kind: ObjectiveKind::JobLevel {
                    profile_id: *profile_id,
                    level: *level,
                },
            },
        })
        .collect::<Vec<_>>();
    let mut reward = RewardGrant {
        coins: 0,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        display: RewardDisplay {
            source_guid: quest.giver_npc_id,
            player_guid: character_id,
            reward_kind: 0,
            icon_id: 0,
            name_id: quest.title_string_id.map(|id| id as i32).unwrap_or(0),
            rows: Vec::new(),
        },
    };
    for row in &quest.rewards {
        reward.coins = reward
            .coins
            .checked_add(row.coins)
            .ok_or_else(|| invalid("quest coin reward overflow"))?;
        for item in &row.items {
            reward.items.push(ItemGrant {
                definition_id: item.definition_id,
                count: item.count,
            });
        }
        for (profile, amount) in &row.job_xp {
            let total = reward.job_xp.entry(*profile).or_default();
            *total = total
                .checked_add(*amount)
                .ok_or_else(|| invalid("quest XP reward overflow"))?;
        }
    }
    if let Some(profile) = crate::content::job_xp_profile_unknown(&reward.job_xp, |profile| {
        job_profile_ids.values().any(|id| *id == profile)
    }) {
        // (R) The content-DB path checks the same key rule in
        // `content::validate_reward`; `scripts/content_check.py` audits authored JSON.
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("quest XP references unknown job profile {profile}"),
        ));
    }
    Ok(QuestDefinition {
        id: quest.id,
        title: quest
            .title_string_id
            .map(|id| format!("Quest {id}"))
            .unwrap_or_else(|| format!("Quest {}", quest.id)),
        description: String::new(),
        giver_npc_id: quest.giver_npc_id,
        turnin_npc_id: quest.giver_npc_id,
        prerequisite_quest_ids: quest.prerequisite_quest_ids.clone(),
        requirements: QuestRequirements {
            player_level: quest.min_level.filter(|_| quest.job.is_none()),
            job,
            members_only: false,
        },
        // Steps run in order, one active at a time, as the quest helper shows them.
        // Unordered, "report back to the giver" steps (Call the Crew!: Shakey, then
        // Ricky) would complete on the accepting conversation.
        ordered: true,
        objectives,
        reward,
    })
}

fn validate_definition(definition: &QuestDefinition) -> io::Result<()> {
    if definition.id == 0
        || definition.giver_npc_id == 0
        || definition.turnin_npc_id == 0
        || definition.objectives.len() > MAX_OBJECTIVES
        || definition.title.len() > 512
        || definition.description.len() > 4096
        || definition.reward.coins > i32::MAX as u32
        || definition
            .reward
            .items
            .iter()
            .any(|item| item.definition_id <= 0 || item.count == 0 || item.count > i32::MAX as u32)
        || definition
            .reward
            .job_xp
            .values()
            .try_fold(0u32, |total, amount| total.checked_add(*amount))
            .is_none_or(|total| total > i32::MAX as u32)
        || definition
            .requirements
            .player_level
            .is_some_and(|level| level == 0)
        || definition
            .requirements
            .job
            .as_ref()
            .is_some_and(|(job, level)| job.is_empty() || *level == 0)
    {
        return Err(invalid("invalid quest definition"));
    }
    let ids: BTreeSet<_> = definition
        .objectives
        .iter()
        .map(|objective| objective.id)
        .collect();
    if ids.len() != definition.objectives.len()
        || ids.contains(&0)
        || definition.objectives.iter().any(|objective| {
            objective.required == 0
                || objective.required > MAX_COUNT
                || objective.description.len() > 4096
                || !valid_objective_kind(&objective.kind)
        })
        // F576: a job-level goal is checked at accept and on a rank change, never when an
        // earlier ordered step completes, so in an ordered quest it may only be the first step.
        || (definition.ordered
            && definition
                .objectives
                .iter()
                .skip(1)
                .any(|objective| matches!(objective.kind, ObjectiveKind::JobLevel { .. })))
    {
        return Err(invalid("invalid quest objective definition"));
    }
    Ok(())
}

/// (R) Content-edit migration, including W92's formerly hollow records. No grants
/// run here: completed status is authoritative history, never a new reward claim.
fn repair_counters(
    state: &mut QuestState,
    definitions: &BTreeMap<u32, QuestDefinition>,
    disabled: &BTreeSet<u32>,
) {
    state.quests.retain(|id, progress| {
        let available = definitions.get(id).filter(|_| !disabled.contains(id));
        let reason = if let Some(definition) = available {
            if progress.counts.len() == definition.objectives.len()
                && !progress
                    .counts
                    .iter()
                    .zip(&definition.objectives)
                    .any(|(count, goal)| {
                        *count > goal.required
                            || (progress.status == QuestStatus::Completed
                                && *count != goal.required)
                    })
            {
                return true;
            }
            progress.counts = definition
                .objectives
                .iter()
                .map(|goal| {
                    if progress.status == QuestStatus::Completed {
                        goal.required
                    } else {
                        0
                    }
                })
                .collect();
            "reset_counters"
        } else {
            if progress.status == QuestStatus::Completed && progress.counts.is_empty() {
                return true;
            }
            progress.counts.clear();
            "drop_unavailable_counters"
        };
        // Only fixed text and a u32 id: one bounded line per repaired quest, no identity/chat.
        eprintln!("quest_save_repair quest={id} action={reason}");
        if state.pending_offer == Some(*id) && available.is_none() {
            state.pending_offer = None;
        }
        if state.pending_turnin == Some(*id) {
            state.pending_turnin = None;
        }
        if state.selected_quest_id == Some(*id) && available.is_none() {
            state.selected_quest_id = None;
        }
        available.is_some() || progress.status == QuestStatus::Completed
    });
    if state.pending_turnin.is_none() {
        state.pending_turnin_actor = None;
    }
}

fn validate_state(
    state: &QuestState,
    character_id: u64,
    definitions: &BTreeMap<u32, QuestDefinition>,
    disabled: &BTreeSet<u32>,
) -> io::Result<()> {
    if state.version != 1 || state.character_id != character_id || state.quests.len() > MAX_QUESTS {
        return Err(invalid(
            "saved quest state does not match definitions or character",
        ));
    }
    for (id, progress) in &state.quests {
        let Some(definition) = definitions.get(id).filter(|_| !disabled.contains(id)) else {
            if progress.status != QuestStatus::Completed || !progress.counts.is_empty() {
                return Err(invalid("unavailable quest has nonhistorical progress"));
            }
            continue;
        };
        if progress.counts.len() != definition.objectives.len()
            || progress
                .counts
                .iter()
                .zip(&definition.objectives)
                .any(|(count, objective)| *count > objective.required)
            || (progress.status == QuestStatus::Offered
                && progress.counts.iter().any(|count| *count != 0))
            || (progress.status == QuestStatus::Completed
                && !is_complete(definition, &progress.counts))
        {
            return Err(invalid("saved quest counters are inconsistent"));
        }
    }
    let offered_count = state
        .quests
        .values()
        .filter(|progress| progress.status == QuestStatus::Offered)
        .count();
    if state.pending_offer.is_some_and(|id| {
        state
            .quests
            .get(&id)
            .is_none_or(|progress| progress.status != QuestStatus::Offered)
    }) || offered_count > 1
        || (state.pending_offer.is_some() != (offered_count == 1))
        || state.selected_quest_id.is_some_and(|id| {
            state
                .quests
                .get(&id)
                .is_none_or(|progress| progress.status != QuestStatus::Active)
        })
        || state.pending_turnin.is_some_and(|id| {
            state.quests.get(&id).is_none_or(|progress| {
                progress.status != QuestStatus::Active
                    || definitions
                        .get(&id)
                        .is_none_or(|definition| !is_complete(definition, &progress.counts))
            })
        })
    {
        return Err(invalid("saved quest handshake is inconsistent"));
    }
    Ok(())
}

fn valid_objective_kind(kind: &ObjectiveKind) -> bool {
    match kind {
        ObjectiveKind::TalkToNpc { npc_id } | ObjectiveKind::InteractNpc { npc_id } => *npc_id != 0,
        ObjectiveKind::KillActorKind { actor_kind_id } => *actor_kind_id != 0,
        ObjectiveKind::CollectItem { item_id } | ObjectiveKind::UseItem { item_id } => *item_id > 0,
        ObjectiveKind::EnterArea { .. } => true,
        ObjectiveKind::ReachPoi { poi_id } => *poi_id != 0,
        ObjectiveKind::FinishMinigame { minigame_id } => *minigame_id != 0,
        ObjectiveKind::GatherResource { resource_id } => *resource_id != 0,
        ObjectiveKind::CraftRecipe { recipe_id } => *recipe_id != 0,
        ObjectiveKind::SelectJob { profile_id } => *profile_id > 0,
        ObjectiveKind::JobLevel { profile_id, level } => {
            *profile_id > 0 && (1..=20).contains(level)
        }
        ObjectiveKind::EquipItem { definition_id } => *definition_id > 0,
        ObjectiveKind::CompleteDungeon { encounter_id } => *encounter_id != 0,
        ObjectiveKind::UseAbility { ability_id } => *ability_id > 0,
        ObjectiveKind::ClientReported { client_id, trigger } => {
            (1..=i32::MAX as u32).contains(client_id)
                && match trigger {
                    ClientTrigger::LookAt {
                        guid,
                        offset_tenths,
                        range,
                        angle_degrees,
                    } => {
                        *guid != 0
                            && (-10_000_000..=10_000_000).contains(offset_tenths)
                            && (1..=i32::MAX as u32).contains(range)
                            && (1..=360).contains(angle_degrees)
                    }
                    ClientTrigger::UiEvent { event } => {
                        (1..=MAX_UI_EVENT_BYTES).contains(&event.len())
                            && event.bytes().all(|byte| byte.is_ascii_graphic())
                    }
                    ClientTrigger::FirstMovement { .. } => true,
                }
        }
        ObjectiveKind::VisitPosition {
            zone_id: _,
            position,
            radius,
        } => {
            position
                .iter()
                .all(|value| value.is_finite() && value.abs() <= 1_000_000.0)
                && radius.is_finite()
                && (0.1..=1000.0).contains(radius)
        }
    }
}

fn validate_event(event: &QuestEvent) -> io::Result<()> {
    let valid = match event {
        QuestEvent::TalkToNpc { npc_id } | QuestEvent::InteractNpc { npc_id } => *npc_id != 0,
        QuestEvent::KillActorKind {
            actor_kind_id,
            count,
        } => *actor_kind_id != 0 && (1..=MAX_COUNT).contains(count),
        QuestEvent::CollectItem { item_id, quantity }
        | QuestEvent::UseItem { item_id, quantity } => {
            *item_id > 0 && (1..=MAX_COUNT).contains(quantity)
        }
        QuestEvent::EnterArea { .. } => true,
        QuestEvent::ReachPoi { poi_id } => *poi_id != 0,
        QuestEvent::FinishMinigame { minigame_id } => *minigame_id != 0,
        QuestEvent::GatherResource {
            resource_id,
            quantity,
        } => *resource_id != 0 && (1..=MAX_COUNT).contains(quantity),
        QuestEvent::CraftRecipe {
            recipe_id,
            quantity,
        } => *recipe_id != 0 && (1..=MAX_COUNT).contains(quantity),
        QuestEvent::SelectJob { profile_id }
        | QuestEvent::UseAbility {
            ability_id: profile_id,
        } => *profile_id > 0,
        QuestEvent::JobLevel { profile_id, level } => *profile_id > 0 && (1..=20).contains(level),
        QuestEvent::EquipItem { definition_id } => *definition_id > 0,
        QuestEvent::CompleteDungeon { encounter_id } => *encounter_id != 0,
        QuestEvent::ClientReported { client_id } => (1..=i32::MAX as u32).contains(client_id),
        QuestEvent::VisitPosition { position, .. } => position
            .iter()
            .all(|value| value.is_finite() && value.abs() <= 1_000_000.0),
    };
    if valid {
        Ok(())
    } else {
        Err(invalid("invalid authoritative quest event"))
    }
}

fn event_matches(kind: &ObjectiveKind, event: &QuestEvent) -> u32 {
    match (kind, event) {
        (ObjectiveKind::TalkToNpc { npc_id: a }, QuestEvent::TalkToNpc { npc_id: b }) if a == b => {
            1
        }
        (
            ObjectiveKind::KillActorKind { actor_kind_id: a },
            QuestEvent::KillActorKind {
                actor_kind_id: b,
                count,
            },
        ) if a == b => *count,
        (
            ObjectiveKind::CollectItem { item_id: a },
            QuestEvent::CollectItem {
                item_id: b,
                quantity,
            },
        ) if a == b => *quantity,
        (ObjectiveKind::EnterArea { area_id: a }, QuestEvent::EnterArea { area_id: b })
            if a == b =>
        {
            1
        }
        (ObjectiveKind::ReachPoi { poi_id: a }, QuestEvent::ReachPoi { poi_id: b }) if a == b => 1,
        (
            ObjectiveKind::FinishMinigame { minigame_id: a },
            QuestEvent::FinishMinigame { minigame_id: b },
        ) if a == b => 1,
        (
            ObjectiveKind::UseItem { item_id: a },
            QuestEvent::UseItem {
                item_id: b,
                quantity,
            },
        ) if a == b => *quantity,
        (ObjectiveKind::InteractNpc { npc_id: a }, QuestEvent::InteractNpc { npc_id: b })
            if a == b =>
        {
            1
        }
        (
            ObjectiveKind::GatherResource { resource_id: a },
            QuestEvent::GatherResource {
                resource_id: b,
                quantity,
            },
        ) if a == b => *quantity,
        (
            ObjectiveKind::CraftRecipe { recipe_id: a },
            QuestEvent::CraftRecipe {
                recipe_id: b,
                quantity,
            },
        ) if a == b => *quantity,
        (ObjectiveKind::SelectJob { profile_id: a }, QuestEvent::SelectJob { profile_id: b })
            if a == b =>
        {
            1
        }
        (
            ObjectiveKind::EquipItem { definition_id: a },
            QuestEvent::EquipItem { definition_id: b },
        ) if a == b => 1,
        (
            ObjectiveKind::CompleteDungeon { encounter_id: a },
            QuestEvent::CompleteDungeon { encounter_id: b },
        ) if a == b => 1,
        (ObjectiveKind::UseAbility { ability_id: a }, QuestEvent::UseAbility { ability_id: b })
            if a == b =>
        {
            1
        }
        (
            ObjectiveKind::VisitPosition {
                zone_id: a,
                position: target,
                radius,
            },
            QuestEvent::VisitPosition {
                zone_id: b,
                position,
            },
        ) if a == b && within(*target, *position, *radius) => 1,
        (
            ObjectiveKind::ClientReported { client_id: a, .. },
            QuestEvent::ClientReported { client_id: b },
        ) if a == b => 1,
        (
            ObjectiveKind::JobLevel {
                profile_id: a,
                level: objective_level,
            },
            QuestEvent::JobLevel {
                profile_id: b,
                level: event_level,
            },
        ) if a == b && event_level >= objective_level => 1,
        _ => 0,
    }
}

/// Client 45/7 `ClientComplete` (`00cffba0` builds it, `00cffa70` sends it; piece 072):
/// `u16 45`, `u8 7`, `i32 objective id` = 7 bytes. `Ok(None)` means another packet. The length
/// must be exact and the id positive (R). The id only names a goal; whether that goal is open
/// is the engine's call.
pub fn decode_client_complete(inner: &[u8]) -> io::Result<Option<u32>> {
    let [45, 0, 7, rest @ ..] = inner else {
        return Ok(None);
    };
    let bytes: [u8; 4] = rest
        .try_into()
        .map_err(|_| invalid("client objective completion must be 7 bytes"))?;
    u32::try_from(i32::from_le_bytes(bytes))
        .ok()
        .filter(|id| *id != 0)
        .map(Some)
        .ok_or_else(|| invalid("client objective id out of range"))
}

impl QuestEngine {
    /// The client-reported goals the client should be watching right now: the open goals of
    /// active quests (R).
    pub fn open_client_objectives(&self) -> Vec<(u32, &ClientTrigger)> {
        self.state
            .quests
            .keys()
            .flat_map(|quest_id| self.current_objectives(*quest_id))
            .filter_map(|objective| match &objective.kind {
                ObjectiveKind::ClientReported { client_id, trigger } => Some((*client_id, trigger)),
                _ => None,
            })
            .collect()
    }

    /// The open job-level goals of the character's active quests. `current_objectives`
    /// already limits an ordered quest to its first unfinished step, so a later goal
    /// cannot be credited early; `validate_definition` refuses a job-level goal that is
    /// not an ordered quest's first step, so none waits behind an earlier one.
    pub fn open_job_level_objectives(&self) -> Vec<(i32, u8)> {
        self.state
            .quests
            .keys()
            .flat_map(|quest_id| self.current_objectives(*quest_id))
            .filter_map(|objective| match &objective.kind {
                ObjectiveKind::JobLevel { profile_id, level } => Some((*profile_id, *level)),
                _ => None,
            })
            .collect()
    }
}

/// What this session has told the client about client-reported goals; not saved. The client
/// keeps registrations and never de-duplicates them (`00c43140`), so each id is registered
/// once (45/6, 45/10 or 45/11) and a goal that opens again is re-armed with 45/9 (R).
#[derive(Clone, Debug, Default)]
pub struct ClientObjectives {
    registered: BTreeSet<u32>,
    armed: BTreeSet<u32>,
}

impl ClientObjectives {
    /// The packets that tell the client about goals opened since the last call.
    pub fn update(&mut self, engine: &QuestEngine) -> Vec<Vec<u8>> {
        let open = engine.open_client_objectives();
        let mut packets = Vec::new();
        for (client_id, trigger) in &open {
            if self.armed.contains(client_id) {
                continue;
            }
            let (sub, tail) = match trigger {
                _ if !self.registered.insert(*client_id) => (9, Vec::new()),
                ClientTrigger::LookAt {
                    guid,
                    offset_tenths,
                    range,
                    angle_degrees,
                } => {
                    // 45/6 actor form read by 009b2760 and dispatched by 009be130:
                    // id, u64 GUID, i32 offset in tenths, i32 range, i32 degrees.
                    let mut tail = guid.to_le_bytes().to_vec();
                    tail.extend(offset_tenths.to_le_bytes());
                    tail.extend(range.to_le_bytes());
                    tail.extend(angle_degrees.to_le_bytes());
                    (6, tail)
                }
                ClientTrigger::UiEvent { event } => {
                    let mut tail = (event.len() as u32).to_le_bytes().to_vec();
                    tail.extend(event.as_bytes());
                    (10, tail)
                }
                ClientTrigger::FirstMovement { mouse } => (11, vec![u8::from(*mouse)]),
            };
            let mut packet = vec![45, 0, sub];
            packet.extend(client_id.to_le_bytes());
            packet.extend(tail);
            packets.push(packet);
        }
        self.armed = open.into_iter().map(|(client_id, _)| client_id).collect();
        packets
    }
}

fn event_is_talk_to(event: &QuestEvent, npc_id: u64) -> bool {
    matches!(event, QuestEvent::TalkToNpc { npc_id: id } if *id == npc_id)
}

fn within(a: [f32; 3], b: [f32; 3], radius: f32) -> bool {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f32>() <= radius * radius
}

fn is_complete(definition: &QuestDefinition, counts: &[u32]) -> bool {
    counts.len() == definition.objectives.len()
        && counts
            .iter()
            .zip(&definition.objectives)
            .all(|(count, objective)| *count == objective.required)
}

fn overall_progress(definition: &QuestDefinition, counts: &[u32]) -> f32 {
    (counts
        .iter()
        .zip(&definition.objectives)
        .map(|(count, objective)| *count as f64 / objective.required as f64)
        .sum::<f64>()
        / definition.objectives.len() as f64) as f32
}

fn reward_summary(quest_id: u32, grant: &RewardGrant) -> QuestRewardSummary {
    QuestRewardSummary {
        quest_id,
        coins: grant.coins,
        items: grant
            .items
            .iter()
            .map(|item| (item.definition_id, item.count))
            .collect(),
    }
}

fn denied(reason: QuestDenial) -> QuestOutcome {
    QuestOutcome {
        denial: Some(reason),
        ..QuestOutcome::default()
    }
}

fn encode_quest_reward_bundle(reward: &RewardGrant) -> io::Result<Vec<u8>> {
    let stars = reward.job_xp.values().try_fold(0u32, |total, amount| {
        total
            .checked_add(*amount)
            .ok_or_else(|| invalid("quest reward XP display total overflow"))
    })?;
    let coins =
        i32::try_from(reward.coins).map_err(|_| invalid("quest coins exceed wire field"))?;
    let stars = i32::try_from(stars).map_err(|_| invalid("quest XP exceeds wire field"))?;
    let row_count = i32::try_from(reward.display.rows.len())
        .map_err(|_| invalid("quest reward bundle has too many rows"))?;
    if reward.display.rows.len() > 1024
        || reward.display.rows.iter().any(|row| {
            !matches!(
                row.type_id,
                1 | 3 | 6 | 7 | 8 | 10 | 11 | 12 | 13 | 14 | 15 | 17 | 18 | 19 | 20 | 22 | 23
            ) || row.body.len() < 34
                || row.body.len() > 65_536
        })
    {
        return Err(invalid("unsupported quest reward display row"));
    }

    // This is the 69-byte body consumed by 008e7930, without family 50/subtype 1.
    // Success/default fields follow the existing reward writer and remain reconstructed (R).
    let mut bytes = Vec::new();
    bytes.push(1);
    for value in [coins, stars, 0, reward.display.reward_kind, 0, 0] {
        put_i32(&mut bytes, value);
    }
    bytes.extend(1.0f32.to_le_bytes());
    put_i32(&mut bytes, 0);
    put_i32(&mut bytes, 0);
    bytes.extend(reward.display.source_guid.to_le_bytes());
    bytes.extend(reward.display.player_guid.to_le_bytes());
    put_i32(&mut bytes, reward.display.icon_id);
    put_i32(&mut bytes, reward.display.name_id);
    put_i32(&mut bytes, row_count);
    for row in &reward.display.rows {
        bytes.extend(row.type_id.to_le_bytes());
        bytes.extend(&row.body);
    }
    put_i32(&mut bytes, 0);
    Ok(bytes)
}

fn header49(subtype: i32) -> Vec<u8> {
    let mut bytes = 49u16.to_le_bytes().to_vec();
    put_i32(&mut bytes, subtype);
    bytes
}

fn id_packet49(subtype: i32, id: u32) -> Option<Vec<u8>> {
    let mut bytes = header49(subtype);
    put_i32(&mut bytes, id.try_into().ok()?);
    Some(bytes)
}

fn put_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}
