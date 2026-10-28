//! Versioned offline content records. Loader cross-reference checks are (R)
//! packaging invariants; they do not claim historical server policy.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    fmt, fs,
    path::{Path, PathBuf},
};

pub const SCHEMA_VERSION: u32 = 1;

/// The server's bundled authored content directory.
pub fn bundled_content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// The NPC content set `Gameplay::load` binds: the save's own directory when it
/// carries the NPC package, else the bundled content.
pub fn npc_content(directory: &Path) -> PathBuf {
    if directory.join("npc_spawns_extra.json").is_file() && directory.join("npcs").is_dir() {
        directory.to_path_buf()
    } else {
        bundled_content()
    }
}

/// Whether that set has what the NPC interaction layer (and its coin shop)
/// needs. `Gameplay::load` binds both from this one rule.
pub fn npc_shop_enabled(content: &Path) -> bool {
    content.join("npcs").is_dir()
        && content.join("stores.json").is_file()
        && content.join("vendors.json").is_file()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    content = "evidence",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SourceTag {
    Client(String),
    Video(String),
    Wiki(String),
    Reconstructed(String),
    #[serde(rename = "legacy-data")]
    LegacyData(String),
}

impl SourceTag {
    pub(crate) fn evidence(&self) -> &str {
        match self {
            Self::Client(value)
            | Self::Video(value)
            | Self::Wiki(value)
            | Self::Reconstructed(value)
            | Self::LegacyData(value) => value,
        }
    }

    pub(crate) fn is_reconstructed(&self) -> bool {
        matches!(self, Self::Reconstructed(_))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionedFile<T> {
    pub schema_version: u32,
    pub records: Vec<T>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    pub id: u32,
    pub name: String,
    pub geometry_id: Option<u32>,
    pub zone_type: Option<i32>,
    pub tutorial: Option<bool>,
    pub sky: Option<String>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Rotation {
    Heading(f32),
    Quaternion([f32; 4]),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Transform {
    pub position: [f32; 3],
    /// (R) Preserves startup NPC float4's fourth component without naming its meaning.
    pub position_w: Option<f32>,
    pub rotation: Rotation,
    pub scale: Option<f32>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NpcRole {
    Vendor,
    QuestGiver,
    Ambient,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NpcSpawn {
    pub actor_id: u64,
    pub zone_id: u32,
    pub model_id: u32,
    pub name_string_id: u32,
    pub transform: Transform,
    /// One spawn may serve multiple roles; each role still needs provenance.
    pub role: Vec<NpcRole>,
    pub quest_ids: Vec<u32>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PointOfInterest {
    pub id: u32,
    pub zone_id: u32,
    pub name_string_id: u32,
    pub transform: Transform,
    pub icon_id: Option<u32>,
    /// Startup opcode 57 crosswalk fields; meanings remain unresolved where noted.
    pub location_id: Option<u32>,
    pub notification_type: Option<u32>,
    pub sub_name_string_id: Option<u32>,
    pub unknown_field: Option<u32>,
    pub breadcrumb_quest_id: Option<u32>,
    pub teleport_location_id: Option<u32>,
    pub atlas_name: Option<String>,
    pub unknown_trailing_field: Option<u32>,
    pub source: Vec<SourceTag>,
}

/// Content-facing item data. `startup_catalogues::ItemDefinition` is a borrowed
/// wire-record view, so it is not suitable as a deserialized catalogue row.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: i32,
    pub name_string_id: u32,
    pub description_string_id: Option<u32>,
    pub category_id: Option<i32>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ItemGrant {
    pub definition_id: i32,
    pub count: u32,
    pub source: Vec<SourceTag>,
}

/// Content-facing grant, shaped for conversion to `rewards::RewardGrant`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RewardGrant {
    pub coins: u32,
    pub items: Vec<ItemGrant>,
    /// Keys are the client profile IDs used by `rewards::RewardGrant::job_xp`.
    pub job_xp: BTreeMap<i32, u32>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContentGoal {
    Defeat {
        id: u32,
        model_id: u32,
        count: u32,
        source: Vec<SourceTag>,
    },
    TalkToNpc {
        id: u32,
        npc_id: u64,
        /// Locale id the NPC says when this step is done (Call the Crew!: Shakey's
        /// "Tell Ricky I'll come fix his car", 94258).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dialogue_string_id: Option<u32>,
        /// T543: the step's multi-line conversation. A step without lines keeps the
        /// single `dialogue_string_id` behavior.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        lines: Vec<crate::quest_dialogue::DialogueLine>,
        /// T543: the Talk starts the conversation but does not complete the step; the
        /// finished conversation does (`QuestEvent::DialogueFinished`).
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        complete_after_dialogue: bool,
        source: Vec<SourceTag>,
    },
    CollectItem {
        id: u32,
        item_id: i32,
        quantity: u32,
        source: Vec<SourceTag>,
    },
    ReachPoi {
        id: u32,
        poi_id: u32,
        // W104 explicit reach geometry (quest_engine::RuntimeGoal::ReachPoi).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zone_id: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        position: Option<[f32; 3]>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        radius: Option<f32>,
        source: Vec<SourceTag>,
    },
    VisitZone {
        id: u32,
        zone_id: u32,
        source: Vec<SourceTag>,
    },
    /// (R) The character's job rank reaches `level`: one authoritative server fact
    /// from the `Profiles::rank` 100-XP curve. `profile_id` is a jobs.json profile id.
    JobLevel {
        id: u32,
        profile_id: i32,
        level: u8,
        source: Vec<SourceTag>,
    },
    // W104 runtime kinds; same fields as quest_engine::RuntimeGoal.
    GatherResource {
        id: u32,
        resource_id: u32,
        quantity: u32,
        source: Vec<SourceTag>,
    },
    /// (R) T503: pick up `quantity` quest-only world nodes of one `quest_nodes.json` type.
    CollectNode {
        id: u32,
        node_type: String,
        quantity: u32,
        source: Vec<SourceTag>,
    },
    CraftRecipe {
        id: u32,
        recipe_id: u32,
        quantity: u32,
        source: Vec<SourceTag>,
    },
    FinishMinigame {
        id: u32,
        minigame_id: u32,
        count: u32,
        source: Vec<SourceTag>,
    },
    /// (R) T788: the owner's pet performs a trick the client names by id. `trick_id` is a
    /// `pets.json` `tricks[].id`; the event is the server accepting that 53/7, never a grade.
    PerformPetTrick {
        id: u32,
        trick_id: u32,
        count: u32,
        source: Vec<SourceTag>,
    },
    EquipItem {
        id: u32,
        definition_id: i32,
        source: Vec<SourceTag>,
    },
    VisitPosition {
        id: u32,
        zone_id: u32,
        position: [f32; 3],
        radius: f32,
        source: Vec<SourceTag>,
    },
    // W175: a goal only the client judges (look, UI event, first movement). The
    // client reports it with 45/7; `client_id` is that wire id.
    ClientReported {
        id: u32,
        client_id: u32,
        trigger: crate::quest_engine::ClientTrigger,
        source: Vec<SourceTag>,
    },
    /// (R) An accepted, non-talk interaction with one exact runtime actor.
    InteractActor {
        id: u32,
        actor_guid: u64,
        source: Vec<SourceTag>,
    },
    /// (R) The authoritative terminal win of one encounter or field battle.
    CompleteDungeon {
        id: u32,
        encounter_id: u32,
        source: Vec<SourceTag>,
    },
    /// T789: (R) one accepted job-ability cast from the toolbar. `target_actor_kind_id` is an
    /// `enemies.json` `definition_id`; absent means any target (and none for a self cast).
    UseAbility {
        id: u32,
        ability_id: i32,
        count: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_actor_kind_id: Option<u32>,
        source: Vec<SourceTag>,
    },
    /// T789: (R) one single-use item actually spent from the item bar.
    UseItem {
        id: u32,
        item_id: i32,
        count: u32,
        source: Vec<SourceTag>,
    },
}

impl ContentGoal {
    fn id(&self) -> u32 {
        match self {
            Self::Defeat { id, .. }
            | Self::TalkToNpc { id, .. }
            | Self::CollectItem { id, .. }
            | Self::ReachPoi { id, .. }
            | Self::VisitZone { id, .. }
            | Self::JobLevel { id, .. }
            | Self::GatherResource { id, .. }
            | Self::CollectNode { id, .. }
            | Self::CraftRecipe { id, .. }
            | Self::FinishMinigame { id, .. }
            | Self::PerformPetTrick { id, .. }
            | Self::EquipItem { id, .. }
            | Self::VisitPosition { id, .. }
            | Self::ClientReported { id, .. }
            | Self::InteractActor { id, .. }
            | Self::CompleteDungeon { id, .. }
            | Self::UseAbility { id, .. }
            | Self::UseItem { id, .. } => *id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub id: u32,
    pub title_string_id: Option<u32>,
    pub giver_npc_id: u64,
    pub prerequisite_quest_ids: Vec<u32>,
    /// (R) Authored offer gate; with `job`, applies to that job's level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_level: Option<u8>,
    /// Numeric profile id from jobs.json; no inferred gate from the quest file name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job: Option<i32>,
    /// W104 publication flag: hollow quests are `false` and never offered.
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
    /// T418: offered only while an admin has the event open (`realm event open`,
    /// `gameplay/live_events.rs`). A closed event stays enabled.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub live_event: bool,
    /// T465: quests this one is mutually exclusive with. While any of them is
    /// offered, active or completed, this one is neither offered nor accepted
    /// (`QuestEngine::requirement_denial`); an abandon reopens it (R). The split
    /// loader requires at most 16 distinct other quests, each listing this one back.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excludes_quest_ids: Vec<u32>,
    pub objectives: Vec<ContentGoal>,
    pub rewards: Vec<RewardGrant>,
    pub source: Vec<SourceTag>,
}

fn enabled_by_default() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobCategory {
    Combat,
    Crafting,
    Freestyle,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JobAbility {
    pub level: u8,
    pub ability_id: u32,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    /// Stable slug, for example `archer`; not a recovered client numeric id.
    pub id: String,
    pub name: String,
    pub profile_id: Option<i32>,
    pub category: JobCategory,
    pub max_level: u8,
    pub members_only: bool,
    pub trainer_npc_ids: Vec<u64>,
    pub quest_ids: Vec<u32>,
    pub abilities: Vec<JobAbility>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EntryRequirement {
    QuestComplete {
        quest_id: u32,
        source: Vec<SourceTag>,
    },
    JobLevel {
        job_id: String,
        level: u8,
        source: Vec<SourceTag>,
    },
    PlayerLevel {
        level: u8,
        source: Vec<SourceTag>,
    },
    MembersOnly {
        source: Vec<SourceTag>,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Encounter {
    pub id: String,
    pub goals: Vec<ContentGoal>,
    pub rewards: Vec<RewardGrant>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Dungeon {
    pub id: String,
    pub name_string_id: Option<u32>,
    pub zone_id: Option<u32>,
    pub entry_requirements: Vec<EntryRequirement>,
    pub encounters: Vec<Encounter>,
    pub rewards: Vec<RewardGrant>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MinigameRow {
    pub id: u32,
    pub game_type_id: u32,
    pub name_string_id: Option<u32>,
    pub dungeon_id: Option<String>,
    pub rewards: Vec<RewardGrant>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContentDb {
    pub zones: Vec<Zone>,
    pub npcs: Vec<NpcSpawn>,
    pub pois: Vec<PointOfInterest>,
    pub items: Vec<Item>,
    pub quests: Vec<Quest>,
    pub jobs: Vec<Job>,
    pub dungeons: Vec<Dungeon>,
    pub minigames: Vec<MinigameRow>,
}

#[derive(Debug, PartialEq)]
pub enum ContentError {
    Io {
        file: String,
        message: String,
    },
    InvalidJson {
        file: String,
        message: String,
    },
    MissingProvenance {
        file: String,
        record_index: usize,
    },
    UnsupportedSchemaVersion {
        file: String,
        found: u32,
        expected: u32,
    },
    DuplicateId {
        kind: String,
        id: String,
    },
    DanglingReference {
        owner: String,
        field: String,
        target: String,
    },
    InvalidProvenance {
        record: String,
        detail: String,
    },
    InvalidRecord {
        record: String,
        detail: String,
    },
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { file, message } => write!(f, "{file}: {message}"),
            Self::InvalidJson { file, message } => write!(f, "{file}: {message}"),
            Self::MissingProvenance { file, record_index } => {
                write!(
                    f,
                    "{file}: record {record_index} is missing source provenance"
                )
            }
            Self::UnsupportedSchemaVersion {
                file,
                found,
                expected,
            } => write!(f, "{file}: schema_version {found}, expected {expected}"),
            Self::DuplicateId { kind, id } => write!(f, "duplicate {kind} id {id}"),
            Self::DanglingReference {
                owner,
                field,
                target,
            } => write!(f, "{owner}.{field} references missing id {target}"),
            Self::InvalidProvenance { record, detail } => {
                write!(f, "{record}: invalid source provenance: {detail}")
            }
            Self::InvalidRecord { record, detail } => write!(f, "{record}: {detail}"),
        }
    }
}

impl std::error::Error for ContentError {}

fn read_records<T: DeserializeOwned>(dir: &Path, file: &str) -> Result<Vec<T>, ContentError> {
    let path = dir.join(file);
    let raw = fs::read_to_string(&path).map_err(|error| ContentError::Io {
        file: path.display().to_string(),
        message: error.to_string(),
    })?;
    let value: Value = serde_json::from_str(&raw).map_err(|error| ContentError::InvalidJson {
        file: file.to_owned(),
        message: error.to_string(),
    })?;
    let version_value = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| ContentError::InvalidJson {
            file: file.to_owned(),
            message: "missing or invalid schema_version".to_owned(),
        })?;
    let version = u32::try_from(version_value).map_err(|_| ContentError::InvalidJson {
        file: file.to_owned(),
        message: "schema_version is outside the u32 range".to_owned(),
    })?;
    if version != SCHEMA_VERSION {
        return Err(ContentError::UnsupportedSchemaVersion {
            file: file.to_owned(),
            found: version,
            expected: SCHEMA_VERSION,
        });
    }
    let records = value
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| ContentError::InvalidJson {
            file: file.to_owned(),
            message: "missing or invalid records array".to_owned(),
        })?;
    for (record_index, record) in records.iter().enumerate() {
        if record.get("source").is_none() {
            return Err(ContentError::MissingProvenance {
                file: file.to_owned(),
                record_index,
            });
        }
    }
    let document: VersionedFile<T> =
        serde_json::from_value(value).map_err(|error| ContentError::InvalidJson {
            file: file.to_owned(),
            message: error.to_string(),
        })?;
    Ok(document.records)
}

fn unique_ids(
    kind: &str,
    ids: impl IntoIterator<Item = String>,
) -> Result<HashSet<String>, ContentError> {
    let mut seen = HashSet::new();
    for id in ids {
        if id.trim().is_empty() {
            return Err(ContentError::InvalidRecord {
                record: kind.to_owned(),
                detail: "id must not be empty".to_owned(),
            });
        }
        if !seen.insert(id.clone()) {
            return Err(ContentError::DuplicateId {
                kind: kind.to_owned(),
                id,
            });
        }
    }
    Ok(seen)
}

pub(crate) fn check_sources(record: &str, sources: &[SourceTag]) -> Result<(), ContentError> {
    if sources.is_empty() {
        return Err(ContentError::InvalidProvenance {
            record: record.to_owned(),
            detail: "source must contain at least one tag".to_owned(),
        });
    }
    for source in sources {
        let evidence = source.evidence().trim();
        if evidence.is_empty() {
            return Err(ContentError::InvalidProvenance {
                record: record.to_owned(),
                detail: "evidence must not be empty".to_owned(),
            });
        }
        if source.is_reconstructed() && !evidence.starts_with("(R)") {
            return Err(ContentError::InvalidProvenance {
                record: record.to_owned(),
                detail: "reconstructed evidence must start with (R)".to_owned(),
            });
        }
    }
    Ok(())
}

fn reference(
    owner: &str,
    field: &str,
    target: impl ToString,
    exists: bool,
) -> Result<(), ContentError> {
    if exists {
        Ok(())
    } else {
        Err(ContentError::DanglingReference {
            owner: owner.to_owned(),
            field: field.to_owned(),
            target: target.to_string(),
        })
    }
}

impl ContentDb {
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ContentError> {
        let dir = dir.as_ref();
        let db = Self {
            zones: read_records(&dir, "zones.json")?,
            npcs: read_records(&dir, "npcs.json")?,
            pois: read_records(&dir, "pois.json")?,
            items: read_records(&dir, "items.json")?,
            quests: read_records(&dir, "quests.json")?,
            jobs: read_records(&dir, "jobs.json")?,
            dungeons: read_records(&dir, "dungeons.json")?,
            minigames: read_records(&dir, "minigames.json")?,
        };
        db.validate()?;
        Ok(db)
    }

    fn validate(&self) -> Result<(), ContentError> {
        for row in &self.zones {
            check_sources(&format!("zone {}", row.id), &row.source)?;
        }
        for row in &self.npcs {
            let owner = format!("npc {}", row.actor_id);
            check_sources(&owner, &row.source)?;
            if row.role.is_empty() {
                return Err(ContentError::InvalidRecord {
                    record: owner,
                    detail: "role must contain at least one value".to_owned(),
                });
            }
            if row.role.iter().copied().collect::<HashSet<_>>().len() != row.role.len() {
                return Err(ContentError::InvalidRecord {
                    record: owner,
                    detail: "role contains a duplicate value".to_owned(),
                });
            }
        }
        for row in &self.pois {
            check_sources(&format!("poi {}", row.id), &row.source)?;
        }
        for row in &self.items {
            check_sources(&format!("item {}", row.id), &row.source)?;
        }
        for row in &self.quests {
            let owner = format!("quest {}", row.id);
            check_sources(&owner, &row.source)?;
            if row
                .min_level
                .is_some_and(|level| !(1..=20).contains(&level))
                || row.job.is_some_and(|profile| profile <= 0)
            {
                return Err(ContentError::InvalidRecord {
                    record: owner,
                    detail: "quest requirements need min_level 1..=20 and a positive job profile"
                        .into(),
                });
            }
            for goal in &row.objectives {
                check_goal_sources(&owner, goal)?;
            }
            for reward in &row.rewards {
                check_reward_sources(&owner, reward)?;
            }
        }
        for row in &self.jobs {
            let owner = format!("job {}", row.id);
            check_sources(&owner, &row.source)?;
            for ability in &row.abilities {
                check_sources(
                    &format!("{owner} ability {}", ability.ability_id),
                    &ability.source,
                )?;
            }
        }
        for row in &self.dungeons {
            let owner = format!("dungeon {}", row.id);
            check_sources(&owner, &row.source)?;
            for requirement in &row.entry_requirements {
                check_requirement_sources(&owner, requirement)?;
            }
            for encounter in &row.encounters {
                let encounter_owner = format!("{owner} encounter {}", encounter.id);
                check_sources(&encounter_owner, &encounter.source)?;
                for goal in &encounter.goals {
                    check_goal_sources(&encounter_owner, goal)?;
                }
                for reward in &encounter.rewards {
                    check_reward_sources(&encounter_owner, reward)?;
                }
            }
            for reward in &row.rewards {
                check_reward_sources(&owner, reward)?;
            }
        }
        for row in &self.minigames {
            let owner = format!("minigame {}", row.id);
            check_sources(&owner, &row.source)?;
            for reward in &row.rewards {
                check_reward_sources(&owner, reward)?;
            }
        }

        let zones = unique_ids("zone", self.zones.iter().map(|row| row.id.to_string()))?;
        let npcs = unique_ids("npc", self.npcs.iter().map(|row| row.actor_id.to_string()))?;
        let pois = unique_ids("poi", self.pois.iter().map(|row| row.id.to_string()))?;
        let items = unique_ids("item", self.items.iter().map(|row| row.id.to_string()))?;
        let quests = unique_ids("quest", self.quests.iter().map(|row| row.id.to_string()))?;
        let jobs = unique_ids("job", self.jobs.iter().map(|row| row.id.clone()))?;
        let job_profiles = unique_ids(
            "job profile",
            self.jobs
                .iter()
                .filter_map(|row| row.profile_id.map(|id| id.to_string())),
        )?;
        let dungeons = unique_ids("dungeon", self.dungeons.iter().map(|row| row.id.clone()))?;
        unique_ids(
            "minigame",
            self.minigames.iter().map(|row| row.id.to_string()),
        )?;

        for row in &self.npcs {
            let owner = format!("npc {}", row.actor_id);
            reference(
                &owner,
                "zone_id",
                row.zone_id,
                zones.contains(&row.zone_id.to_string()),
            )?;
            for quest_id in &row.quest_ids {
                reference(
                    &owner,
                    "quest_ids",
                    quest_id,
                    quests.contains(&quest_id.to_string()),
                )?;
            }
        }
        for row in &self.pois {
            reference(
                &format!("poi {}", row.id),
                "zone_id",
                row.zone_id,
                zones.contains(&row.zone_id.to_string()),
            )?;
        }
        for row in &self.quests {
            let owner = format!("quest {}", row.id);
            if let Some(profile) = row.job {
                reference(
                    &owner,
                    "job",
                    profile,
                    job_profiles.contains(&profile.to_string()),
                )?;
            }
            reference(
                &owner,
                "giver_npc_id",
                row.giver_npc_id,
                npcs.contains(&row.giver_npc_id.to_string()),
            )?;
            unique_ids(
                "quest objective",
                row.objectives.iter().map(|goal| goal.id().to_string()),
            )?;
            for quest_id in &row.prerequisite_quest_ids {
                reference(
                    &owner,
                    "prerequisite_quest_ids",
                    quest_id,
                    quests.contains(&quest_id.to_string()),
                )?;
            }
            for goal in &row.objectives {
                validate_goal(&owner, goal, &zones, &npcs, &pois, &items, &job_profiles)?;
            }
            for reward in &row.rewards {
                validate_reward(&owner, reward, &items, &job_profiles)?;
            }
        }
        for row in &self.jobs {
            let owner = format!("job {}", row.id);
            for npc_id in &row.trainer_npc_ids {
                reference(
                    &owner,
                    "trainer_npc_ids",
                    npc_id,
                    npcs.contains(&npc_id.to_string()),
                )?;
            }
            for quest_id in &row.quest_ids {
                reference(
                    &owner,
                    "quest_ids",
                    quest_id,
                    quests.contains(&quest_id.to_string()),
                )?;
            }
        }
        for row in &self.dungeons {
            let owner = format!("dungeon {}", row.id);
            if let Some(zone_id) = row.zone_id {
                reference(
                    &owner,
                    "zone_id",
                    zone_id,
                    zones.contains(&zone_id.to_string()),
                )?;
            }
            for requirement in &row.entry_requirements {
                validate_requirement(&owner, requirement, &quests, &jobs)?;
            }
            unique_ids(
                &format!("encounter in dungeon {}", row.id),
                row.encounters.iter().map(|encounter| encounter.id.clone()),
            )?;
            for encounter in &row.encounters {
                let encounter_owner = format!("{owner} encounter {}", encounter.id);
                unique_ids(
                    "encounter goal",
                    encounter.goals.iter().map(|goal| goal.id().to_string()),
                )?;
                for goal in &encounter.goals {
                    validate_goal(
                        &encounter_owner,
                        goal,
                        &zones,
                        &npcs,
                        &pois,
                        &items,
                        &job_profiles,
                    )?;
                }
                for reward in &encounter.rewards {
                    validate_reward(&encounter_owner, reward, &items, &job_profiles)?;
                }
            }
            for reward in &row.rewards {
                validate_reward(&owner, reward, &items, &job_profiles)?;
            }
        }
        for row in &self.minigames {
            let owner = format!("minigame {}", row.id);
            if let Some(dungeon_id) = &row.dungeon_id {
                reference(
                    &owner,
                    "dungeon_id",
                    dungeon_id,
                    dungeons.contains(dungeon_id),
                )?;
            }
            for reward in &row.rewards {
                validate_reward(&owner, reward, &items, &job_profiles)?;
            }
        }
        Ok(())
    }
}

fn check_goal_sources(owner: &str, goal: &ContentGoal) -> Result<(), ContentError> {
    let (id, source) = match goal {
        ContentGoal::Defeat { id, source, .. }
        | ContentGoal::TalkToNpc { id, source, .. }
        | ContentGoal::CollectItem { id, source, .. }
        | ContentGoal::ReachPoi { id, source, .. }
        | ContentGoal::VisitZone { id, source, .. }
        | ContentGoal::JobLevel { id, source, .. }
        | ContentGoal::GatherResource { id, source, .. }
        | ContentGoal::CollectNode { id, source, .. }
        | ContentGoal::CraftRecipe { id, source, .. }
        | ContentGoal::FinishMinigame { id, source, .. }
        | ContentGoal::PerformPetTrick { id, source, .. }
        | ContentGoal::EquipItem { id, source, .. }
        | ContentGoal::VisitPosition { id, source, .. }
        | ContentGoal::ClientReported { id, source, .. }
        | ContentGoal::InteractActor { id, source, .. }
        | ContentGoal::CompleteDungeon { id, source, .. }
        | ContentGoal::UseAbility { id, source, .. }
        | ContentGoal::UseItem { id, source, .. } => (id, source),
    };
    check_sources(&format!("{owner} goal {id}"), source)
}

fn check_reward_sources(owner: &str, reward: &RewardGrant) -> Result<(), ContentError> {
    check_sources(owner, &reward.source)?;
    for item in &reward.items {
        check_sources(
            &format!("{owner} item grant {}", item.definition_id),
            &item.source,
        )?;
    }
    Ok(())
}

fn check_requirement_sources(
    owner: &str,
    requirement: &EntryRequirement,
) -> Result<(), ContentError> {
    let source = match requirement {
        EntryRequirement::QuestComplete { source, .. }
        | EntryRequirement::JobLevel { source, .. }
        | EntryRequirement::PlayerLevel { source, .. }
        | EntryRequirement::MembersOnly { source } => source,
    };
    check_sources(&format!("{owner} entry requirement"), source)
}

fn validate_goal(
    owner: &str,
    goal: &ContentGoal,
    zones: &HashSet<String>,
    npcs: &HashSet<String>,
    pois: &HashSet<String>,
    items: &HashSet<String>,
    job_profiles: &HashSet<String>,
) -> Result<(), ContentError> {
    match goal {
        ContentGoal::Defeat { .. } => Ok(()),
        ContentGoal::TalkToNpc {
            npc_id,
            lines,
            complete_after_dialogue,
            ..
        } => {
            crate::quest_dialogue::validate(lines).map_err(|detail| {
                ContentError::InvalidRecord {
                    record: owner.to_owned(),
                    detail: detail.to_owned(),
                }
            })?;
            if *complete_after_dialogue && lines.is_empty() {
                return Err(ContentError::InvalidRecord {
                    record: owner.to_owned(),
                    detail: "complete_after_dialogue requires lines".to_owned(),
                });
            }
            reference(
                owner,
                "goal.npc_id",
                npc_id,
                npcs.contains(&npc_id.to_string()),
            )
        }
        ContentGoal::CollectItem { item_id, .. } => reference(
            owner,
            "goal.item_id",
            item_id,
            items.contains(&item_id.to_string()),
        ),
        ContentGoal::ReachPoi { poi_id, .. } => reference(
            owner,
            "goal.poi_id",
            poi_id,
            pois.contains(&poi_id.to_string()),
        ),
        ContentGoal::VisitZone { zone_id, .. } => reference(
            owner,
            "goal.zone_id",
            zone_id,
            zones.contains(&zone_id.to_string()),
        ),
        ContentGoal::JobLevel {
            profile_id, level, ..
        } => {
            reference(
                owner,
                "goal.profile_id",
                profile_id,
                *profile_id > 0 && job_profiles.contains(&profile_id.to_string()),
            )?;
            if !(1..=20).contains(level) {
                return Err(ContentError::InvalidRecord {
                    record: owner.to_owned(),
                    detail: "goal.level must be 1..=20".to_owned(),
                });
            }
            Ok(())
        }
        ContentGoal::EquipItem { definition_id, .. } => reference(
            owner,
            "goal.definition_id",
            definition_id,
            items.contains(&definition_id.to_string()),
        ),
        ContentGoal::CollectNode {
            node_type,
            quantity,
            ..
        } => {
            if !valid_node_type(node_type) {
                return Err(ContentError::InvalidRecord {
                    record: owner.to_owned(),
                    detail: format!("goal.node_type must be 1..={MAX_NODE_TYPE_CHARS} characters"),
                });
            }
            if !(1..=MAX_COLLECT_NODE_QUANTITY).contains(quantity) {
                return Err(ContentError::InvalidRecord {
                    record: owner.to_owned(),
                    detail: format!("goal.quantity must be 1..={MAX_COLLECT_NODE_QUANTITY}"),
                });
            }
            // The node type resolves against quest_nodes.json in the enabled-quest
            // reachability gate, not in this id-reference pass.
            Ok(())
        }
        // T788: the trick id must be a real row; the enabled-quest reachability gate
        // resolves it in pets.json (the quest engine does not load pet content).
        ContentGoal::PerformPetTrick {
            trick_id, count, ..
        } => {
            if *trick_id == 0 || !(1..=crate::quest_engine::MAX_COUNT).contains(count) {
                return Err(ContentError::InvalidRecord {
                    record: owner.to_owned(),
                    detail: format!(
                        "goal.trick_id must be positive and goal.count 1..={}",
                        crate::quest_engine::MAX_COUNT
                    ),
                });
            }
            Ok(())
        }
        // The quest engine validates resource, recipe, minigame and geometry bindings.
        ContentGoal::GatherResource { .. }
        | ContentGoal::CraftRecipe { .. }
        | ContentGoal::FinishMinigame { .. }
        | ContentGoal::VisitPosition { .. }
        | ContentGoal::ClientReported { .. } => Ok(()),
        ContentGoal::InteractActor { actor_guid, .. } => {
            positive_goal_field(owner, "goal.actor_guid", *actor_guid)
        }
        ContentGoal::CompleteDungeon { encounter_id, .. } => {
            positive_goal_field(owner, "goal.encounter_id", u64::from(*encounter_id))
        }
        // T789: sanity bounds only here. The enabled-quest reachability gate resolves the
        // ability id in `job_abilities.json`, the target in `enemies.json` and the item in the
        // shipped catalogue, because the quest engine loads none of those tables.
        ContentGoal::UseAbility {
            ability_id,
            count,
            target_actor_kind_id,
            ..
        } => {
            positive_goal_field(
                owner,
                "goal.ability_id",
                u64::try_from(*ability_id).unwrap_or(0),
            )?;
            use_goal_count(owner, count)?;
            if target_actor_kind_id.is_some_and(|target| target == 0) {
                return Err(ContentError::InvalidRecord {
                    record: owner.to_owned(),
                    detail: "goal.target_actor_kind_id must be positive or absent".to_owned(),
                });
            }
            Ok(())
        }
        ContentGoal::UseItem { item_id, count, .. } => {
            positive_goal_field(owner, "goal.item_id", u64::try_from(*item_id).unwrap_or(0))?;
            use_goal_count(owner, count)
        }
    }
}

fn use_goal_count(owner: &str, count: &u32) -> Result<(), ContentError> {
    if !(1..=crate::quest_engine::MAX_COUNT).contains(count) {
        return Err(ContentError::InvalidRecord {
            record: owner.to_owned(),
            detail: format!("goal.count must be 1..={}", crate::quest_engine::MAX_COUNT),
        });
    }
    Ok(())
}

fn positive_goal_field(owner: &str, field: &str, value: u64) -> Result<(), ContentError> {
    if value == 0 {
        return Err(ContentError::InvalidRecord {
            record: owner.to_owned(),
            detail: format!("{field} must be positive"),
        });
    }
    Ok(())
}

/// (R) T503: one `collect_node` node-type key, 1..=64 characters with no control
/// characters. Recovered fork keys are short words; the bound keeps a content typo from
/// becoming an unbounded key in the save and in the reachability gate.
const MAX_NODE_TYPE_CHARS: usize = 64;
/// (R) T503: one `collect_node` count (the fork goals are single digits).
pub(crate) const MAX_COLLECT_NODE_QUANTITY: u32 = 100;

pub(crate) fn valid_node_type(node_type: &str) -> bool {
    (1..=MAX_NODE_TYPE_CHARS).contains(&node_type.chars().count())
        && !node_type.chars().any(char::is_control)
}

/// (R) Job XP may only name a positive profile id declared by `jobs.json`. The split
/// quest loader (`quest_engine::definition_from_content`) shares this rule, so the two
/// content paths cannot drift; `scripts/content_check.py` audits the authored JSON.
pub(crate) fn job_xp_profile_unknown(
    job_xp: &BTreeMap<i32, u32>,
    profile_known: impl Fn(i32) -> bool,
) -> Option<i32> {
    job_xp
        .keys()
        .copied()
        .find(|profile| *profile <= 0 || !profile_known(*profile))
}

fn validate_reward(
    owner: &str,
    reward: &RewardGrant,
    items: &HashSet<String>,
    job_profiles: &HashSet<String>,
) -> Result<(), ContentError> {
    for item in &reward.items {
        reference(
            owner,
            "reward.items.definition_id",
            item.definition_id,
            items.contains(&item.definition_id.to_string()),
        )?;
    }
    if let Some(profile_id) = job_xp_profile_unknown(&reward.job_xp, |profile_id| {
        job_profiles.contains(&profile_id.to_string())
    }) {
        reference(
            owner,
            "reward.job_xp.profile_id",
            profile_id,
            job_profiles.contains(&profile_id.to_string()),
        )?;
    }
    Ok(())
}

fn validate_requirement(
    owner: &str,
    requirement: &EntryRequirement,
    quests: &HashSet<String>,
    jobs: &HashSet<String>,
) -> Result<(), ContentError> {
    match requirement {
        EntryRequirement::QuestComplete { quest_id, .. } => reference(
            owner,
            "entry_requirement.quest_id",
            quest_id,
            quests.contains(&quest_id.to_string()),
        ),
        EntryRequirement::JobLevel { job_id, .. } => reference(
            owner,
            "entry_requirement.job_id",
            job_id,
            jobs.contains(job_id),
        ),
        EntryRequirement::PlayerLevel { .. } | EntryRequirement::MembersOnly { .. } => Ok(()),
    }
}
