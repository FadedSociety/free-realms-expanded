//! Collections, achievements, titles and claim-window state.
//!
//! Client facts are cited in `notes/Research/Collections Achievements Titles.md`.
//! Event bindings, reward timing, title selection propagation, and opaque claim
//! row serialization are reconstructed policy (R). Where definitions are
//! server-sent and have no client table, local IDs use the documented (R)
//! namespaces in the content files; they are not recovered historical IDs.

use crate::{
    inventory::Inventory,
    rewards::{self, RewardBundleRow, RewardGrant, RewardState},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

#[path = "restoration/piece_074.rs"]
mod collection_ledger;
pub use collection_ledger::{
    CollectOutcome, CollectionCatalogue, CollectionDefinition, CollectionLedger, CollectionProgress,
};

// Local safety bounds, not recovered historical limits (R).
const MAX_COUNTER: u32 = 1_000_000;
const MAX_PACKET_BYTES: usize = 1024 * 1024;
const MAX_CLAIM_ITEMS: usize = 256;
const MAX_CLAIM_HISTORY: usize = 4096;
const MAX_TITLES: usize = 256;
const FORBIDDEN_BUNDLE_ENTRY_TYPES: [u32; 6] = [2, 4, 5, 9, 16, 21];
const SUPPORTED_BUNDLE_ENTRY_TYPES: [u32; 17] = [
    1, 3, 6, 7, 8, 10, 11, 12, 13, 14, 15, 17, 18, 19, 20, 22, 23,
];

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn packet_u16_u16(family: u16, subtype: u16) -> Vec<u8> {
    [family.to_le_bytes(), subtype.to_le_bytes()].concat()
}

fn packet_u16_u32(family: u16, subtype: u32) -> Vec<u8> {
    let mut packet = family.to_le_bytes().to_vec();
    packet.extend(subtype.to_le_bytes());
    packet
}

/// Reject bundle rows the client cannot instantiate. Reward presentation must
/// never contain ids 2, 4, 5, 9, 16 or 21 (client reader `0103c070`).
pub fn validate_reward_rows(rows: &[RewardBundleRow]) -> io::Result<()> {
    for row in rows {
        if FORBIDDEN_BUNDLE_ENTRY_TYPES.contains(&row.type_id)
            || !SUPPORTED_BUNDLE_ENTRY_TYPES.contains(&row.type_id)
            || row.body.len() < 34
        {
            return Err(invalid("unsupported or truncated reward bundle entry"));
        }
    }
    Ok(())
}

fn apply_award(
    state: &RewardState,
    grant: &RewardGrant,
    key: &str,
) -> io::Result<(RewardState, Vec<Vec<u8>>)> {
    validate_reward_rows(&grant.display.rows)?;
    rewards::apply_grant(state, grant, key)
}

// Content readers. Unknown client ids and server bindings are optional by design.

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionContent {
    pub schema_version: u32,
    pub categories: Vec<CollectionCategoryContent>,
    pub definitions: Vec<CollectionDefinitionContent>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionCategoryContent {
    pub id: u32,
    pub name_id: u32,
    pub icon_id: u32,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionDefinitionContent {
    pub key: String,
    pub client_id: Option<u32>,
    pub category_id: Option<u32>,
    #[serde(default)]
    pub wiki_category_id: Option<u32>,
    pub name: String,
    pub name_id: Option<u32>,
    pub description: Option<String>,
    pub description_id: Option<u32>,
    pub image_id: Option<u32>,
    #[serde(default)]
    pub collection_type: Option<u32>,
    #[serde(default)]
    pub icon_tint_id: Option<u32>,
    #[serde(default)]
    pub header_metadata: Option<u32>,
    #[serde(default)]
    pub reward_metadata: Option<u32>,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
    #[serde(default)]
    pub legacy_icon_id: Option<u32>,
    #[serde(default)]
    pub entries_known: Option<bool>,
    #[serde(default)]
    pub entry_count: Option<u32>,
    pub entries: Vec<CollectionEntryContent>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CollectionEntryContent {
    pub name: String,
    pub client_entry_id: Option<u32>,
    #[serde(default)]
    pub name_id: Option<u32>,
    #[serde(default)]
    pub item_definition_id: Option<u32>,
    #[serde(default)]
    pub icon_id: Option<u32>,
    #[serde(default)]
    pub legacy_icon_id: Option<u32>,
    #[serde(default)]
    pub icon_tint_id: Option<u32>,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AchievementContent {
    pub schema_version: u32,
    pub client_categories: Vec<serde_json::Value>,
    pub trophies: Vec<serde_json::Value>,
    pub definitions: Vec<AchievementDefinitionContent>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AchievementDefinitionContent {
    pub key: String,
    pub client_id: Option<u32>,
    pub name: String,
    pub description: Option<String>,
    pub name_id: Option<u32>,
    pub description_id: Option<u32>,
    pub image_id: Option<u32>,
    pub image_set_id: Option<u32>,
    pub counter_key: Option<String>,
    pub threshold: Option<u32>,
    #[serde(default)]
    pub reward: Option<AchievementRewardContent>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AchievementRewardContent {
    #[serde(default)]
    pub title_names: Vec<String>,
    #[serde(default)]
    pub client_trophy_id: Option<u32>,
    #[serde(default)]
    pub trophy_points: u32,
    #[serde(default)]
    pub item_ids: Option<Vec<u32>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TitleContent {
    pub schema_version: u32,
    pub definitions: Vec<TitleDefinitionContent>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TitleDefinitionContent {
    pub key: String,
    pub client_id: Option<u32>,
    pub name: String,
    pub name_id: Option<u32>,
    pub description_id: Option<u32>,
    pub image_id: Option<u32>,
    pub image_set_id: Option<u32>,
    #[serde(default)]
    pub title_type: Option<u32>,
    #[serde(default)]
    pub legacy_icon_id: Option<u32>,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

pub fn parse_collection_content(bytes: &[u8]) -> io::Result<CollectionContent> {
    serde_json::from_slice(bytes).map_err(|_| invalid("invalid collection content"))
}

pub fn parse_achievement_content(bytes: &[u8]) -> io::Result<AchievementContent> {
    serde_json::from_slice(bytes).map_err(|_| invalid("invalid achievement content"))
}

pub fn parse_title_content(bytes: &[u8]) -> io::Result<TitleContent> {
    serde_json::from_slice(bytes).map_err(|_| invalid("invalid title content"))
}

#[derive(Clone, Debug, Default)]
pub struct AchievementBindings {
    rules: Vec<AchievementRule>,
    title_bindings: Vec<AchievementTitleBinding>,
    claim_bindings: Vec<AchievementClaimBinding>,
    job_slugs: BTreeMap<i32, String>,
}

impl AchievementBindings {
    /// Load achievement counters and exact name joins to the shipped title
    /// definitions. Missing optional content yields no bindings.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let bundled = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let source = [directory, bundled.as_path()]
            .into_iter()
            .find(|path| path.join("achievements.json").is_file());
        let Some(source) = source else {
            return Ok(Self::default());
        };
        let achievements = parse_achievement_content(&fs::read(source.join("achievements.json"))?)?;
        let titles = if source.join("titles.json").is_file() {
            parse_title_content(&fs::read(source.join("titles.json"))?)?
        } else {
            TitleContent {
                schema_version: 1,
                definitions: Vec::new(),
            }
        };
        let jobs = if source.join("jobs.json").is_file() {
            serde_json::from_slice::<JobsContent>(&fs::read(source.join("jobs.json"))?)
                .map_err(|_| invalid("invalid jobs content"))?
        } else {
            JobsContent {
                records: Vec::new(),
            }
        };

        let mut title_by_name = BTreeMap::new();
        for title in &titles.definitions {
            if let Some(id) = title.client_id {
                title_by_name.entry(title.name.as_str()).or_insert([
                    id,
                    title.title_type.unwrap_or(0),
                    title.name_id.unwrap_or(0),
                    title.image_id.unwrap_or(0),
                ]);
            }
        }
        let mut rules = Vec::new();
        let mut title_bindings = Vec::new();
        let mut claim_bindings = Vec::new();
        for definition in achievements.definitions {
            let (Some(client_id), Some(counter_key), Some(threshold)) = (
                definition.client_id,
                definition.counter_key,
                definition.threshold,
            ) else {
                continue;
            };
            let reward = definition.reward.unwrap_or_default();
            let item_ids: BTreeSet<_> = reward
                .item_ids
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect();
            rules.push(AchievementRule {
                key: definition.key.clone(),
                client_id: Some(client_id),
                counter_key,
                objective_id: 1,
                threshold,
                trophy_id: reward.client_trophy_id,
                reward: empty_grant(),
            });
            for name in reward.title_names {
                if let Some(&title) = title_by_name.get(name.as_str()) {
                    title_bindings.push(AchievementTitleBinding {
                        achievement_key: definition.key.clone(),
                        title,
                    });
                }
            }
            let items: Vec<_> = item_ids
                .into_iter()
                .map(|id| {
                    i32::try_from(id).map(|definition_id| rewards::ItemGrant {
                        definition_id,
                        count: 1,
                    })
                })
                .collect::<Result<_, _>>()
                .map_err(|_| invalid("achievement item reward exceeds i32"))?;
            if !items.is_empty() {
                claim_bindings.push(AchievementClaimBinding {
                    achievement_key: definition.key.clone(),
                    claim_id: achievement_claim_id(&definition.key),
                    grant: RewardGrant {
                        coins: 0,
                        items,
                        job_xp: BTreeMap::new(),
                        display: rewards::RewardDisplay {
                            source_guid: 0,
                            player_guid: 0,
                            reward_kind: 0,
                            icon_id: 0,
                            name_id: 0,
                            rows: Vec::new(),
                        },
                    },
                });
            }
        }
        let mut job_slugs = BTreeMap::new();
        for job in jobs.records {
            if job.profile_id > 0
                && !job.id.is_empty()
                && job_slugs.insert(job.profile_id, job.id).is_some()
            {
                return Err(invalid("duplicate achievement job binding"));
            }
        }
        AchievementProgression::new(rules.clone(), AchievementState::default())?;
        Ok(Self {
            rules,
            title_bindings,
            claim_bindings,
            job_slugs,
        })
    }

    pub fn rules(&self) -> &[AchievementRule] {
        &self.rules
    }

    pub fn title_bindings(&self) -> &[AchievementTitleBinding] {
        &self.title_bindings
    }

    pub fn claim_bindings(&self) -> &[AchievementClaimBinding] {
        &self.claim_bindings
    }

    pub fn job_slug(&self, profile_id: i32) -> Option<&str> {
        self.job_slugs.get(&profile_id).map(String::as_str)
    }
}

/// An achievement whose recovered reward contains item definitions queues a
/// server-owned claim rather than granting inventory before the player claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AchievementClaimBinding {
    pub achievement_key: String,
    pub claim_id: u64,
    pub grant: RewardGrant,
}

fn achievement_claim_id(key: &str) -> u64 {
    let digest = Sha256::digest(format!("achievement-claim:{key}").as_bytes());
    let mut id = u64::from_le_bytes(digest[..8].try_into().unwrap());
    id |= 1 << 63;
    id
}

#[derive(Deserialize)]
struct JobsContent {
    records: Vec<JobRow>,
}

#[derive(Deserialize)]
struct JobRow {
    id: String,
    profile_id: i32,
}

// World collection wire messages use signed 16-bit family/subtype headers.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionEntryUpdate {
    pub collection_id: u32,
    pub id_pair: [u32; 2],
    /// Four entry fields are retained as wire values; their meanings are not
    /// fully recovered. `collected=true` is reconstructed event policy (R).
    pub fields: [u32; 4],
    pub collected: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectionMessage {
    /// `record` is the already serialized client record inside the length blob.
    Start {
        record: Vec<u8>,
    },
    /// The subtype-9 body was not recovered; callers must supply known bytes.
    Remove {
        body: Vec<u8>,
    },
    AddEntry(CollectionEntryUpdate),
    RemoveEntry {
        collection_id: u32,
        entry_id: u32,
    },
}

impl CollectionMessage {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let (subtype, body) = match self {
            Self::Start { record } => {
                let len = i32::try_from(record.len())
                    .map_err(|_| invalid("collection record exceeds wire length"))?;
                let mut body = len.to_le_bytes().to_vec();
                body.extend_from_slice(record);
                (8, body)
            }
            Self::Remove { body } => (9, body.clone()),
            Self::AddEntry(update) => {
                let mut body = Vec::with_capacity(29);
                put_u32(&mut body, update.collection_id);
                for value in update.id_pair.into_iter().chain(update.fields) {
                    put_u32(&mut body, value);
                }
                body.push(u8::from(update.collected));
                (10, body)
            }
            Self::RemoveEntry {
                collection_id,
                entry_id,
            } => {
                let mut body = Vec::with_capacity(8);
                put_u32(&mut body, *collection_id);
                put_u32(&mut body, *entry_id);
                (11, body)
            }
        };
        let mut packet = packet_u16_u16(38, subtype);
        packet.extend(body);
        if packet.len() > MAX_PACKET_BYTES {
            return Err(invalid("collection packet exceeds bound"));
        }
        Ok(packet)
    }
}

/// One of the gameplay sources that can supply a newly collected entry (R).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectionEventOrigin {
    HarvestNode,
    QuestReward,
    CollectionBundleEntry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionEvent {
    pub entry_id: i32,
    pub origin: CollectionEventOrigin,
    pub wire_update: Option<CollectionEntryUpdate>,
}

#[derive(Clone)]
pub struct CollectionTransition {
    pub outcome: CollectOutcome,
    pub collection_packets: Vec<Vec<u8>>,
    pub reward_state: RewardState,
    pub reward_packets: Vec<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct CollectionProgression {
    catalogue: CollectionCatalogue,
    ledger: CollectionLedger,
}

impl CollectionProgression {
    pub fn new(catalogue: CollectionCatalogue, ledger: CollectionLedger) -> Self {
        Self { catalogue, ledger }
    }

    pub fn progress(&self, collection_id: i32) -> Option<CollectionProgress> {
        self.ledger.progress(&self.catalogue, collection_id)
    }

    pub fn from_ledger_bytes(catalogue: CollectionCatalogue, bytes: &[u8]) -> io::Result<Self> {
        let ledger = CollectionLedger::from_bytes(bytes)
            .map_err(|_| invalid("invalid collection ledger"))?;
        Ok(Self { catalogue, ledger })
    }

    pub fn ledger_bytes(&self) -> io::Result<Vec<u8>> {
        self.ledger
            .to_bytes()
            .map_err(|_| invalid("collection ledger serialization failed"))
    }

    pub fn collect(
        &mut self,
        event: CollectionEvent,
        completion_reward: &RewardGrant,
        reward_state: &RewardState,
    ) -> io::Result<CollectionTransition> {
        let mut staged = self.ledger.clone();
        let outcome = staged
            .collect(&self.catalogue, event.entry_id)
            .map_err(|_| invalid("collection event references unknown entry"))?;
        if !outcome.newly_collected {
            return Ok(CollectionTransition {
                outcome,
                collection_packets: Vec::new(),
                reward_state: reward_state.clone(),
                reward_packets: Vec::new(),
            });
        }

        let mut collection_packets = Vec::new();
        if let Some(update) = event.wire_update {
            if update.collection_id != u32::try_from(outcome.collection_id).unwrap_or(0)
                || !update.collected
            {
                return Err(invalid("collection wire update does not match event"));
            }
            collection_packets.push(CollectionMessage::AddEntry(update).encode()?);
        }

        let (next_rewards, reward_packets) = if outcome.completion.is_some() {
            apply_award(
                reward_state,
                completion_reward,
                &format!("collection:{}", outcome.collection_id),
            )?
        } else {
            (reward_state.clone(), Vec::new())
        };
        // R: update progress before its single completion reward. Caller saves
        // the ledger and RewardState in the same character transaction.
        self.ledger = staged;
        Ok(CollectionTransition {
            outcome,
            collection_packets,
            reward_state: next_rewards,
            reward_packets,
        })
    }
}

// Achievement wire headers and body widths are recovered; field meanings and
// event-to-counter bindings remain reconstructed policy (R).

#[derive(Clone, Debug)]
pub struct AchievementRule {
    pub key: String,
    pub client_id: Option<u32>,
    pub counter_key: String,
    pub objective_id: u32,
    pub threshold: u32,
    pub trophy_id: Option<u32>,
    pub reward: RewardGrant,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AchievementState {
    pub counters: BTreeMap<String, u32>,
    pub completed: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub struct AchievementProgression {
    rules: Vec<AchievementRule>,
    state: AchievementState,
}

#[derive(Clone)]
pub struct AchievementTransition {
    pub state: AchievementState,
    pub packets: Vec<Vec<u8>>,
    pub reward_state: RewardState,
    pub reward_packets: Vec<Vec<u8>>,
    pub newly_completed: Vec<String>,
}

impl AchievementProgression {
    pub fn new(rules: Vec<AchievementRule>, state: AchievementState) -> io::Result<Self> {
        let mut keys = BTreeSet::new();
        if rules.len() > 4096
            || rules.iter().any(|rule| {
                rule.key.is_empty()
                    || rule.counter_key.is_empty()
                    || rule.threshold == 0
                    || rule.threshold > MAX_COUNTER
                    || !keys.insert(rule.key.clone())
            })
        {
            return Err(invalid("invalid achievement rules"));
        }
        Ok(Self { rules, state })
    }

    pub fn state(&self) -> &AchievementState {
        &self.state
    }

    pub fn on_event(
        &mut self,
        counter_key: &str,
        amount: u32,
        reward_state: &RewardState,
    ) -> io::Result<AchievementTransition> {
        if amount == 0 || amount > MAX_COUNTER {
            return Err(invalid("invalid achievement event amount"));
        }
        let mut staged_state = self.state.clone();
        let mut staged_rewards = reward_state.clone();
        let mut packets = Vec::new();
        let mut reward_packets = Vec::new();
        let mut newly_completed = Vec::new();

        for rule in self.rules.iter().filter(|r| r.counter_key == counter_key) {
            if staged_state.completed.contains(&rule.key) {
                continue;
            }
            let current = staged_state
                .counters
                .get(&rule.key)
                .copied()
                .unwrap_or(0)
                .checked_add(amount)
                .filter(|value| *value <= MAX_COUNTER)
                .ok_or_else(|| invalid("achievement counter overflow"))?;
            staged_state.counters.insert(rule.key.clone(), current);

            if let Some(id) = rule.client_id {
                packets.push(achievement_progress_packet(
                    id,
                    rule.objective_id,
                    current,
                    rule.threshold,
                ));
            }
            if current >= rule.threshold {
                staged_state.completed.insert(rule.key.clone());
                if let Some(id) = rule.client_id {
                    packets.push(achievement_complete_packet(id, rule.trophy_id.unwrap_or(0)));
                }
                if !grant_is_empty(&rule.reward) {
                    let (next, mut emitted) = apply_award(
                        &staged_rewards,
                        &rule.reward,
                        &format!("achievement:{}", rule.key),
                    )?;
                    staged_rewards = next;
                    reward_packets.append(&mut emitted);
                }
                newly_completed.push(rule.key.clone());
            }
        }

        self.state = staged_state.clone();
        Ok(AchievementTransition {
            state: staged_state,
            packets,
            reward_state: staged_rewards,
            reward_packets,
            newly_completed,
        })
    }

    /// Set a counter to an authoritative absolute value. This is used for
    /// values already stored by gameplay (job rank, completed quest count).
    pub fn observe(
        &mut self,
        counter_key: &str,
        value: u32,
        reward_state: &RewardState,
    ) -> io::Result<AchievementTransition> {
        if value > MAX_COUNTER {
            return Err(invalid("invalid achievement counter"));
        }
        let mut staged_state = self.state.clone();
        let mut staged_rewards = reward_state.clone();
        let mut packets = Vec::new();
        let mut reward_packets = Vec::new();
        let mut newly_completed = Vec::new();

        for rule in self.rules.iter().filter(|r| r.counter_key == counter_key) {
            let current = value.min(MAX_COUNTER);
            if staged_state.counters.get(&rule.key).copied().unwrap_or(0) == current
                && (current < rule.threshold || staged_state.completed.contains(&rule.key))
            {
                continue;
            }
            staged_state.counters.insert(rule.key.clone(), current);
            if staged_state.completed.contains(&rule.key) {
                continue;
            }
            if let Some(id) = rule.client_id {
                packets.push(achievement_progress_packet(
                    id,
                    rule.objective_id,
                    current,
                    rule.threshold,
                ));
            }
            if current >= rule.threshold {
                staged_state.completed.insert(rule.key.clone());
                if let Some(id) = rule.client_id {
                    packets.push(achievement_complete_packet(id, rule.trophy_id.unwrap_or(0)));
                }
                if !grant_is_empty(&rule.reward) {
                    let (next, mut emitted) = apply_award(
                        &staged_rewards,
                        &rule.reward,
                        &format!("achievement:{}", rule.key),
                    )?;
                    staged_rewards = next;
                    reward_packets.append(&mut emitted);
                }
                newly_completed.push(rule.key.clone());
            }
        }

        self.state = staged_state.clone();
        Ok(AchievementTransition {
            state: staged_state,
            packets,
            reward_state: staged_rewards,
            reward_packets,
            newly_completed,
        })
    }
}

fn grant_is_empty(grant: &RewardGrant) -> bool {
    grant.coins == 0
        && grant.items.is_empty()
        && grant.job_xp.is_empty()
        && grant.display.rows.is_empty()
}

fn empty_grant() -> RewardGrant {
    RewardGrant {
        coins: 0,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        display: rewards::RewardDisplay {
            source_guid: 0,
            player_guid: 0,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn achievement_progress_packet(
    achievement_id: u32,
    objective_id: u32,
    current: u32,
    threshold: u32,
) -> Vec<u8> {
    let mut packet = packet_u16_u32(150, 7);
    for value in [achievement_id, objective_id, current] {
        put_u32(&mut packet, value);
    }
    // R: normalized objective progress; the native constructor stores an f32.
    let progress = (current as f32 / threshold as f32).min(1.0);
    packet.extend(progress.to_le_bytes());
    packet
}

fn achievement_complete_packet(achievement_id: u32, trophy_id: u32) -> Vec<u8> {
    let mut packet = packet_u16_u32(150, 3);
    put_u32(&mut packet, achievement_id);
    put_u32(&mut packet, trophy_id);
    packet.push(1); // R: normal-completion flag; field meaning from ctor `00ba98c0`.
    packet
}

// Owned titles use the four client packet dwords. The selected flag's position
// in that record is reconstructed from the Lua data-source column (R).

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TitleState {
    owned: BTreeMap<u32, [u32; 4]>,
    active: Option<u32>,
}

impl TitleState {
    pub fn active(&self) -> Option<u32> {
        self.active
    }

    pub fn is_empty(&self) -> bool {
        self.owned.is_empty()
    }

    pub fn contains(&self, title_id: u32) -> bool {
        self.owned.contains_key(&title_id)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.owned.len() > MAX_TITLES
            || self.owned.keys().any(|id| *id == 0)
            || self.active.is_some_and(|id| !self.owned.contains_key(&id))
        {
            return Err(invalid("invalid title state"));
        }
        Ok(())
    }

    pub fn add(&mut self, fields: [u32; 4]) -> io::Result<Vec<u8>> {
        let id = fields[0];
        if id == 0 || self.owned.contains_key(&id) || self.owned.len() >= MAX_TITLES {
            return Err(invalid("invalid or duplicate owned title"));
        }
        self.owned.insert(id, fields);
        let mut packet = packet_u16_u32(152, 1);
        for field in fields {
            put_u32(&mut packet, field);
        }
        Ok(packet)
    }

    pub fn remove(&mut self, title_id: u32) -> io::Result<Vec<u8>> {
        if self.owned.remove(&title_id).is_none() {
            return Err(invalid("title is not owned"));
        }
        if self.active == Some(title_id) {
            self.active = None;
        }
        let mut packet = packet_u16_u32(152, 2);
        put_u32(&mut packet, title_id);
        Ok(packet)
    }

    pub fn handle_select_request(&mut self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        if bytes.len() != 10
            || bytes[..2] != 152u16.to_le_bytes()
            || bytes[2..6] != 4u32.to_le_bytes()
        {
            return Err(invalid("invalid title select request"));
        }
        let id = u32::from_le_bytes(bytes[6..10].try_into().unwrap());
        if !self.owned.contains_key(&id) {
            return Err(invalid("cannot select an unowned title"));
        }
        self.active = Some(id);
        self.update_all_packet()
    }

    pub fn update_all_packet(&self) -> io::Result<Vec<u8>> {
        let count =
            u32::try_from(self.owned.len()).map_err(|_| invalid("too many owned titles"))?;
        let mut blob = Vec::with_capacity(8 + self.owned.len() * 16);
        put_u32(&mut blob, count);
        for fields in self.owned.values() {
            for field in fields {
                put_u32(&mut blob, *field);
            }
        }
        // The last dword is the selected title id (ClientPcData PlayerTitleStruct
        // and native reader 0090a890); selection is not a per-row flag.
        put_u32(&mut blob, self.active.unwrap_or(0));
        let mut packet = packet_u16_u32(152, 3);
        packet.extend(
            i32::try_from(blob.len())
                .map_err(|_| invalid("title list exceeds wire length"))?
                .to_le_bytes(),
        );
        packet.extend(blob);
        if packet.len() > MAX_PACKET_BYTES {
            return Err(invalid("title list exceeds packet bound"));
        }
        Ok(packet)
    }
}

pub fn title_select_request(title_id: u32) -> Vec<u8> {
    let mut packet = packet_u16_u32(152, 4);
    put_u32(&mut packet, title_id);
    packet
}

pub fn title_refresh_request() -> Vec<u8> {
    packet_u16_u32(152, 5)
}

/// One achievement-to-title binding. The caller supplies only titles emitted by
/// `AchievementTransition::newly_completed`; title ids are not client-authoritative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AchievementTitleBinding {
    pub achievement_key: String,
    pub title: [u32; 4],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimItemEntry {
    /// Two leading dwords consumed into entry+4/+8. The second is the item
    /// definition id used by native predicate `00a012a0`.
    pub field_4: u32,
    pub item_id: u32,
    /// The dword consumed into entry+0x14 by native reader `00a033e0`.
    pub field_14: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimItem {
    pub claim_ids: Vec<u64>,
    pub name_id: u32,
    pub description_id: u32,
    pub icon: [u32; 2],
    pub entries: Vec<ClaimItemEntry>,
    pub field_150: [u32; 2],
}

impl ClaimItem {
    pub fn for_grant(claim_id: u64, grant: &RewardGrant) -> io::Result<Self> {
        let entries = grant
            .items
            .iter()
            .map(|item| {
                let item_id = u32::try_from(item.definition_id)
                    .map_err(|_| invalid("claim item definition id exceeds u32"))?;
                Ok(ClaimItemEntry {
                    // (R) The native +4 field is not named; use the stack count.
                    field_4: item.count,
                    item_id,
                    field_14: 0,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        let item = Self {
            claim_ids: vec![claim_id],
            name_id: 0,
            description_id: 0,
            icon: [0; 2],
            entries,
            field_150: [0; 2],
        };
        item.validate()?;
        Ok(item)
    }

    /// Exact row consumed by native reader `00a040e0`.
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = Vec::new();
        put_u32(
            &mut bytes,
            u32::try_from(self.claim_ids.len()).map_err(|_| invalid("too many claim ids"))?,
        );
        for id in &self.claim_ids {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        put_u32(&mut bytes, self.name_id);
        put_u32(&mut bytes, self.description_id);
        for value in self.icon {
            put_u32(&mut bytes, value);
        }
        put_u32(
            &mut bytes,
            u32::try_from(self.entries.len()).map_err(|_| invalid("too many claim entries"))?,
        );
        for entry in &self.entries {
            for value in [entry.field_4, entry.item_id, entry.field_14] {
                put_u32(&mut bytes, value);
            }
        }
        for value in self.field_150 {
            put_u32(&mut bytes, value);
        }
        if bytes.len() > MAX_PACKET_BYTES - 9 {
            return Err(invalid("claim row exceeds packet bound"));
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8], cursor: &mut usize) -> io::Result<Self> {
        let claim_count = take_u32(bytes, cursor)? as usize;
        if claim_count == 0 || claim_count > MAX_CLAIM_ITEMS {
            return Err(invalid("invalid claim id count"));
        }
        let mut claim_ids = Vec::with_capacity(claim_count);
        for _ in 0..claim_count {
            let id = u64::from_le_bytes(
                take(bytes, cursor, 8)?
                    .try_into()
                    .map_err(|_| invalid("invalid claim id"))?,
            );
            if id == 0 || claim_ids.last().is_some_and(|last| *last >= id) {
                return Err(invalid("claim ids are not sorted and unique"));
            }
            claim_ids.push(id);
        }
        let name_id = take_u32(bytes, cursor)?;
        let description_id = take_u32(bytes, cursor)?;
        let icon = [take_u32(bytes, cursor)?, take_u32(bytes, cursor)?];
        let entry_count = take_u32(bytes, cursor)? as usize;
        if entry_count > MAX_CLAIM_ITEMS {
            return Err(invalid("too many claim entries"));
        }
        let entries = (0..entry_count)
            .map(|_| {
                Ok(ClaimItemEntry {
                    field_4: take_u32(bytes, cursor)?,
                    item_id: take_u32(bytes, cursor)?,
                    field_14: take_u32(bytes, cursor)?,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        let field_150 = [take_u32(bytes, cursor)?, take_u32(bytes, cursor)?];
        let item = Self {
            claim_ids,
            name_id,
            description_id,
            icon,
            entries,
            field_150,
        };
        item.validate()?;
        Ok(item)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.claim_ids.is_empty()
            || self.claim_ids.len() > MAX_CLAIM_ITEMS
            || self.claim_ids.contains(&0)
            || self.claim_ids.windows(2).any(|ids| ids[0] >= ids[1])
            || self.entries.len() > MAX_CLAIM_ITEMS
            || self.entries.iter().any(|entry| entry.item_id == 0)
        {
            return Err(invalid("invalid claim item"));
        }
        Ok(())
    }

    fn definition_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.entries.iter().map(|entry| entry.item_id)
    }
}

fn take<'a>(bytes: &'a [u8], cursor: &mut usize, count: usize) -> io::Result<&'a [u8]> {
    let end = cursor
        .checked_add(count)
        .ok_or_else(|| invalid("claim cursor overflow"))?;
    let slice = bytes
        .get(*cursor..end)
        .ok_or_else(|| invalid("truncated claim row"))?;
    *cursor = end;
    Ok(slice)
}

fn take_u32(bytes: &[u8], cursor: &mut usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        take(bytes, cursor, 4)?
            .try_into()
            .map_err(|_| invalid("invalid claim dword"))?,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingClaimState {
    claim_id: u64,
    grant: RewardGrant,
    record: ClaimItem,
}

#[derive(Clone, Debug)]
struct PendingClaim {
    grant: RewardGrant,
    record: ClaimItem,
}

#[derive(Clone, Debug, Default)]
pub struct ClaimWindow {
    pending: BTreeMap<u64, PendingClaim>,
    claimed: BTreeSet<u64>,
    trackers: BTreeMap<u32, BTreeSet<u32>>,
}

#[derive(Clone)]
pub struct ClaimTransition {
    pub accepted: bool,
    pub tracker_id: u32,
    pub claimed_ids: Vec<u64>,
    pub response_packet: Vec<u8>,
    pub reward_state: RewardState,
    pub reward_packets: Vec<Vec<u8>>,
}

impl ClaimWindow {
    /// R: only trusted server grant code calls this method to queue a claim.
    pub fn issue_server_grant(&mut self, claim_id: u64, grant: RewardGrant) -> io::Result<()> {
        validate_reward_rows(&grant.display.rows)?;
        let record = ClaimItem::for_grant(claim_id, &grant)?;
        let record_bytes = record.encode()?.len();
        let mut pending_bytes = 0usize;
        for item in self.pending.values() {
            pending_bytes = pending_bytes
                .checked_add(item.record.encode()?.len())
                .ok_or_else(|| invalid("claim record total overflow"))?;
        }
        if claim_id == 0
            || self.pending.contains_key(&claim_id)
            || self.claimed.contains(&claim_id)
            || self.pending.len() >= MAX_CLAIM_ITEMS
            || pending_bytes
                .checked_add(record_bytes)
                .is_none_or(|total| total > MAX_PACKET_BYTES - 13)
        {
            return Err(invalid("invalid or duplicate pending claim"));
        }
        self.pending
            .insert(claim_id, PendingClaim { grant, record });
        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn with_claimed_ids(claimed: BTreeSet<u64>) -> Self {
        Self {
            pending: BTreeMap::new(),
            claimed,
            trackers: BTreeMap::new(),
        }
    }

    fn from_saved(claimed: BTreeSet<u64>, pending: Vec<PendingClaimState>) -> io::Result<Self> {
        if claimed.len() > MAX_CLAIM_HISTORY
            || claimed.contains(&0)
            || pending.len() > MAX_CLAIM_ITEMS
        {
            return Err(invalid("invalid claim history"));
        }
        let mut window = Self {
            pending: BTreeMap::new(),
            claimed,
            trackers: BTreeMap::new(),
        };
        let mut bytes = 0usize;
        for saved in pending {
            validate_reward_rows(&saved.grant.display.rows)?;
            if saved.claim_id == 0
                || saved.record.claim_ids.as_slice() != [saved.claim_id]
                || ClaimItem::for_grant(saved.claim_id, &saved.grant)? != saved.record
                || window.claimed.contains(&saved.claim_id)
            {
                return Err(invalid("invalid saved pending claim"));
            }
            bytes = bytes
                .checked_add(saved.record.encode()?.len())
                .ok_or_else(|| invalid("saved claim record total overflow"))?;
            if bytes > MAX_PACKET_BYTES - 13 {
                return Err(invalid("saved claim records exceed packet bound"));
            }
            if window
                .pending
                .insert(
                    saved.claim_id,
                    PendingClaim {
                        grant: saved.grant,
                        record: saved.record,
                    },
                )
                .is_some()
            {
                return Err(invalid("duplicate saved pending claim"));
            }
        }
        if window.pending.len().saturating_add(window.claimed.len()) > MAX_CLAIM_HISTORY {
            return Err(invalid("claim history is full"));
        }
        Ok(window)
    }

    fn pending_state(&self) -> Vec<PendingClaimState> {
        self.pending
            .iter()
            .map(|(&claim_id, item)| PendingClaimState {
                claim_id,
                grant: item.grant.clone(),
                record: item.record.clone(),
            })
            .collect()
    }

    pub fn claimed_ids(&self) -> &BTreeSet<u64> {
        &self.claimed
    }

    pub fn list_request() -> Vec<u8> {
        packet_u16_u16(108, 1)
    }

    pub fn list_response(&self) -> io::Result<Vec<u8>> {
        // 108/2 reader 00a04950; each row uses 00a040e0.
        let records = self
            .pending
            .values()
            .map(|item| item.record.encode())
            .collect::<io::Result<Vec<_>>>()?;
        let mut packet = packet_u16_u16(108, 2);
        packet.push(1);
        packet.extend(encode_claim_array(
            &records.iter().map(Vec::as_slice).collect::<Vec<_>>(),
        )?);
        Ok(packet)
    }

    pub fn handle_claim_request(
        &mut self,
        request: &[u8],
        reward_state: &RewardState,
    ) -> io::Result<ClaimTransition> {
        // 108/4 reader 00a049b0; rows are read by 00a040e0.
        let (tracker_id, ids) = parse_claim_request(request)?;
        let reject = || {
            let mut response = packet_u16_u16(108, 4);
            put_u32(&mut response, tracker_id);
            response.push(0);
            response.extend(0u32.to_le_bytes());
            ClaimTransition {
                accepted: false,
                tracker_id,
                claimed_ids: Vec::new(),
                response_packet: response,
                reward_state: reward_state.clone(),
                reward_packets: Vec::new(),
            }
        };
        if ids.is_empty()
            || ids.len() > MAX_CLAIM_ITEMS
            || tracker_id == 0
            || self.trackers.contains_key(&tracker_id)
            || self.trackers.len() >= MAX_CLAIM_ITEMS
            || self.claimed.len().saturating_add(ids.len()) > MAX_CLAIM_HISTORY
            || ids.iter().copied().collect::<BTreeSet<_>>().len() != ids.len()
            || ids.iter().any(|id| !self.pending.contains_key(id))
        {
            return Ok(reject());
        }

        let grants: Vec<_> = ids.iter().map(|id| &self.pending[id].grant).collect();
        let grant = match merge_grants(&grants) {
            Ok(grant) => grant,
            Err(_) => return Ok(reject()),
        };
        let grant_key = claim_grant_key(&ids);
        let (next_rewards, reward_packets) = match apply_award(reward_state, &grant, &grant_key) {
            Ok(result) => result,
            Err(_) => return Ok(reject()),
        };
        let records = ids
            .iter()
            .map(|id| self.pending[id].record.encode())
            .collect::<io::Result<Vec<_>>>()?;
        let mut response = packet_u16_u16(108, 4);
        put_u32(&mut response, tracker_id);
        response.push(1);
        response.extend(encode_claim_array(
            &records.iter().map(Vec::as_slice).collect::<Vec<_>>(),
        )?);

        let item_ids: BTreeSet<u32> = grant
            .items
            .iter()
            .filter_map(|item| u32::try_from(item.definition_id).ok())
            .collect();
        // Mark the ledger before the caller adopts the reward state. The caller
        // saves both snapshots together before sending a reply.
        for id in &ids {
            self.pending.remove(id);
            self.claimed.insert(*id);
        }
        self.trackers.insert(tracker_id, item_ids);
        Ok(ClaimTransition {
            accepted: true,
            tracker_id,
            claimed_ids: ids,
            response_packet: response,
            reward_state: next_rewards,
            reward_packets,
        })
    }

    pub fn definitions_response(
        &mut self,
        request: &[u8],
        inventory: &Inventory,
    ) -> io::Result<Vec<u8>> {
        // 108/6 reader 00a02ac0; both opener paths send 108/5 first
        // (00a04ea0 and 00a04f40).
        let (tracker_id, ids) = parse_definitions_request(request)?;
        if tracker_id == 0 {
            return Err(invalid("invalid claim tracker"));
        }
        let requested: BTreeSet<_> = ids.iter().copied().collect();
        if requested.len() != ids.len() {
            return Err(invalid("claim tracker does not own requested definitions"));
        }
        let pending_ids: BTreeSet<_> = self
            .pending
            .values()
            .flat_map(|item| item.record.definition_ids())
            .collect();
        let mut trackers = self.trackers.clone();
        if let Some(expected) = trackers.get(&tracker_id) {
            if ids.iter().any(|id| !expected.contains(id)) {
                return Err(invalid("claim tracker does not own requested definitions"));
            }
        } else {
            if ids.iter().any(|id| !pending_ids.contains(id)) || trackers.len() >= MAX_CLAIM_ITEMS {
                return Err(invalid("claim tracker does not own requested definitions"));
            }
            trackers.insert(tracker_id, requested);
        }
        let definitions_blob = self.definition_blob(&ids, inventory)?;
        let len = i32::try_from(definitions_blob.len())
            .map_err(|_| invalid("claim definitions exceed wire length"))?;
        let mut response = packet_u16_u16(108, 6);
        put_u32(&mut response, tracker_id);
        response.push(1);
        response.extend(len.to_le_bytes());
        response.extend_from_slice(&definitions_blob);
        trackers.remove(&tracker_id);
        self.trackers = trackers;
        Ok(response)
    }

    fn definition_blob(&self, ids: &[u32], inventory: &Inventory) -> io::Result<Vec<u8>> {
        let count = u32::try_from(ids.len()).map_err(|_| invalid("too many claim definitions"))?;
        let mut blob = Vec::new();
        put_u32(&mut blob, count);
        for &id in ids {
            let definition_id =
                i32::try_from(id).map_err(|_| invalid("claim definition id exceeds i32"))?;
            let record = inventory
                .definition_record(definition_id)
                .ok_or_else(|| invalid("claim definition is missing from inventory"))?;
            if blob
                .len()
                .checked_add(record.len())
                .is_none_or(|total| total > MAX_PACKET_BYTES - 13)
            {
                return Err(invalid("claim definitions exceed packet bound"));
            }
            blob.extend_from_slice(record);
        }
        Ok(blob)
    }
}

/// Persisted titles, achievement counters and the claim inbox. Definition
/// trackers remain per-session.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionExtrasState {
    #[serde(default)]
    pub titles: TitleState,
    #[serde(default)]
    pub claimed_ids: BTreeSet<u64>,
    #[serde(default)]
    achievements: AchievementState,
    #[serde(default)]
    pending_claims: Vec<PendingClaimState>,
}

#[derive(Clone, Default)]
pub struct ProgressionOutcome {
    /// False when another gameplay route owns the request.
    pub handled: bool,
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
    pub reward_state: Option<RewardState>,
}

/// Character-owned title and claim state. Only trusted server methods can add
/// titles or queue claim grants.
#[derive(Clone, Debug, Default)]
pub struct ProgressionExtras {
    titles: TitleState,
    claims: ClaimWindow,
    achievements: AchievementState,
}

impl ProgressionExtras {
    pub fn load(saved: Option<ProgressionExtrasState>) -> io::Result<Self> {
        let Some(saved) = saved else {
            return Ok(Self::default());
        };
        saved.titles.validate()?;
        if saved
            .achievements
            .counters
            .values()
            .any(|value| *value > MAX_COUNTER)
            || saved.achievements.counters.len() > MAX_CLAIM_HISTORY
            || saved.achievements.completed.len() > MAX_CLAIM_HISTORY
        {
            return Err(invalid("invalid achievement state"));
        }
        Ok(Self {
            titles: saved.titles,
            claims: ClaimWindow::from_saved(saved.claimed_ids, saved.pending_claims)?,
            achievements: saved.achievements,
        })
    }

    pub fn state(&self) -> ProgressionExtrasState {
        ProgressionExtrasState {
            titles: self.titles.clone(),
            claimed_ids: self.claims.claimed_ids().clone(),
            achievements: self.achievements.clone(),
            pending_claims: self.claims.pending_state(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.titles.is_empty()
            && self.claims.claimed_ids().is_empty()
            && self.claims.pending_count() == 0
            && self.achievements == AchievementState::default()
    }

    pub fn titles(&self) -> &TitleState {
        &self.titles
    }

    pub fn pending_count(&self) -> usize {
        self.claims.pending_count()
    }

    pub fn achievement_state(&self) -> AchievementState {
        self.achievements.clone()
    }

    pub fn set_achievement_state(&mut self, state: AchievementState) {
        self.achievements = state;
    }

    /// Trusted server/store code queues a claim. No client request can name or
    /// synthesize the reward.
    pub fn issue_server_grant(&mut self, claim_id: u64, grant: RewardGrant) -> io::Result<()> {
        if self
            .claims
            .claimed_ids()
            .len()
            .saturating_add(self.claims.pending_count())
            >= MAX_CLAIM_HISTORY
        {
            return Err(invalid("claim history is full"));
        }
        self.claims.issue_server_grant(claim_id, grant)
    }

    /// Award only the titles bound to achievements completed by the existing
    /// achievement engine during this transition.
    pub fn award_completed_achievement_titles(
        &mut self,
        newly_completed: &[String],
        bindings: &[AchievementTitleBinding],
    ) -> io::Result<Vec<Vec<u8>>> {
        let completed: BTreeSet<&str> = newly_completed.iter().map(String::as_str).collect();
        let mut staged = self.titles.clone();
        let mut packets = Vec::new();
        for binding in bindings {
            if !completed.contains(binding.achievement_key.as_str())
                || staged.contains(binding.title[0])
            {
                continue;
            }
            packets.push(staged.add(binding.title)?);
        }
        self.titles = staged;
        Ok(packets)
    }

    /// Route the client's title and claim requests. Forged ids are silent
    /// refusals; claim failures with a valid request receive 108/4 result 0.
    pub fn handle(
        &mut self,
        request: &[u8],
        reward_state: &RewardState,
    ) -> io::Result<ProgressionOutcome> {
        let mut outcome = ProgressionOutcome {
            handled: true,
            ..ProgressionOutcome::default()
        };
        if request == ClaimWindow::list_request() {
            outcome.replies.push(self.claims.list_response()?);
            return Ok(outcome);
        }
        if request == title_refresh_request() {
            outcome.replies.push(self.titles.update_all_packet()?);
            return Ok(outcome);
        }
        if request.len() == 10
            && request[..2] == 152u16.to_le_bytes()
            && request[2..6] == 4u32.to_le_bytes()
        {
            let mut staged = self.titles.clone();
            let reply = match staged.handle_select_request(request) {
                Ok(reply) => reply,
                Err(_) => return Ok(outcome),
            };
            outcome.changed = staged != self.titles;
            self.titles = staged;
            outcome.replies.push(reply);
            return Ok(outcome);
        }
        if request.len() >= 4 && request[..4] == [108, 0, 3, 0] {
            let mut staged = self.claims.clone();
            let transition = match staged.handle_claim_request(request, reward_state) {
                Ok(transition) => transition,
                Err(_) => return Ok(outcome),
            };
            if transition.accepted {
                self.claims = staged;
                outcome.changed = true;
                outcome.reward_state = Some(transition.reward_state);
            }
            outcome.replies.push(transition.response_packet);
            outcome.replies.extend(transition.reward_packets);
            return Ok(outcome);
        }
        if request.len() >= 4 && request[..4] == [108, 0, 5, 0] {
            let reply = match self
                .claims
                .definitions_response(request, &reward_state.inventory)
            {
                Ok(reply) => reply,
                Err(_) => return Ok(outcome),
            };
            outcome.replies.push(reply);
            return Ok(outcome);
        }
        outcome.handled = false;
        Ok(outcome)
    }
}

pub fn claim_request(tracker_id: u32, ids: &[u64]) -> io::Result<Vec<u8>> {
    if ids.is_empty() || ids.len() > MAX_CLAIM_ITEMS {
        return Err(invalid("invalid claim id count"));
    }
    let mut packet = packet_u16_u16(108, 3);
    put_u32(&mut packet, tracker_id);
    put_u32(
        &mut packet,
        u32::try_from(ids.len()).map_err(|_| invalid("too many claim ids"))?,
    );
    for id in ids {
        packet.extend_from_slice(&id.to_le_bytes());
    }
    Ok(packet)
}

pub fn definitions_request(tracker_id: u32, ids: &[u32]) -> io::Result<Vec<u8>> {
    if ids.is_empty() || ids.len() > MAX_CLAIM_ITEMS {
        return Err(invalid("invalid claim definition count"));
    }
    let mut packet = packet_u16_u16(108, 5);
    put_u32(&mut packet, tracker_id);
    put_u32(
        &mut packet,
        u32::try_from(ids.len()).map_err(|_| invalid("too many claim definitions"))?,
    );
    for id in ids {
        put_u32(&mut packet, *id);
    }
    Ok(packet)
}

fn parse_claim_request(bytes: &[u8]) -> io::Result<(u32, Vec<u64>)> {
    if bytes.len() < 12 || bytes[..2] != 108u16.to_le_bytes() || bytes[2..4] != 3u16.to_le_bytes() {
        return Err(invalid("invalid claim request header"));
    }
    let tracker = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if count > MAX_CLAIM_ITEMS || 12usize.checked_add(count * 8) != Some(bytes.len()) {
        return Err(invalid("invalid claim request length"));
    }
    let ids = bytes[12..]
        .chunks_exact(8)
        .map(|row| u64::from_le_bytes(row.try_into().unwrap()))
        .collect();
    Ok((tracker, ids))
}

fn parse_definitions_request(bytes: &[u8]) -> io::Result<(u32, Vec<u32>)> {
    if bytes.len() < 12 || bytes[..2] != 108u16.to_le_bytes() || bytes[2..4] != 5u16.to_le_bytes() {
        return Err(invalid("invalid claim definitions header"));
    }
    let tracker = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if count > MAX_CLAIM_ITEMS || 12usize.checked_add(count * 4) != Some(bytes.len()) {
        return Err(invalid("invalid claim definitions length"));
    }
    let ids = bytes[12..]
        .chunks_exact(4)
        .map(|row| u32::from_le_bytes(row.try_into().unwrap()))
        .collect();
    Ok((tracker, ids))
}

fn encode_claim_array(records: &[&[u8]]) -> io::Result<Vec<u8>> {
    let count = u32::try_from(records.len()).map_err(|_| invalid("too many claim records"))?;
    let size = records.iter().try_fold(4usize, |size, row| {
        size.checked_add(row.len())
            .ok_or_else(|| invalid("claim array size overflow"))
    })?;
    if size > MAX_PACKET_BYTES - 9 {
        return Err(invalid("claim array exceeds packet bound"));
    }
    let mut bytes = Vec::with_capacity(size);
    put_u32(&mut bytes, count); // R: count width; per-item bytes remain opaque.
    for record in records {
        bytes.extend_from_slice(record);
    }
    Ok(bytes)
}

fn merge_grants(grants: &[&RewardGrant]) -> io::Result<RewardGrant> {
    let first = grants.first().ok_or_else(|| invalid("empty claim batch"))?;
    let mut merged = (*first).clone();
    for grant in &grants[1..] {
        merged.coins = merged
            .coins
            .checked_add(grant.coins)
            .ok_or_else(|| invalid("claim coin total overflow"))?;
        merged.items.extend(grant.items.iter().cloned());
        for (&profile, &amount) in &grant.job_xp {
            let current = merged.job_xp.get(&profile).copied().unwrap_or(0);
            merged.job_xp.insert(
                profile,
                current
                    .checked_add(amount)
                    .ok_or_else(|| invalid("claim XP total overflow"))?,
            );
        }
        merged
            .display
            .rows
            .extend(grant.display.rows.iter().cloned());
    }
    Ok(merged)
}

fn claim_grant_key(ids: &[u64]) -> String {
    let mut sorted = ids.to_vec();
    sorted.sort_unstable();
    let mut bytes = Vec::with_capacity(sorted.len() * 8);
    for id in sorted {
        bytes.extend(id.to_le_bytes());
    }
    let digest = Sha256::digest(bytes);
    let mut key = String::from("claim-batch:");
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(key, "{byte:02x}");
    }
    key
}

/// Promotional opcode 114 is store-originated. This preserves an already
/// serialized subtype-1/2 body; it does not implement code redemption (R).
pub fn promotional_packet(subtype: u16, body: &[u8]) -> io::Result<Vec<u8>> {
    if !matches!(subtype, 1 | 2) || body.len() > MAX_PACKET_BYTES {
        return Err(invalid("unsupported promotional packet"));
    }
    let mut packet = packet_u16_u16(114, subtype);
    packet.extend_from_slice(body);
    Ok(packet)
}
