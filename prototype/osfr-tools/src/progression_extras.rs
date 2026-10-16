//! Collections, achievements, titles and claim-window state.
//!
//! Client facts are cited in `notes/Research/Collections Achievements Titles.md`.
//! Event bindings, reward timing, title selection propagation, and opaque claim
//! row serialization are reconstructed policy (R). Where definitions are
//! server-sent and have no client table, local IDs use the documented (R)
//! namespaces in the content files; they are not recovered historical IDs.

use crate::rewards::{self, RewardBundleRow, RewardGrant, RewardState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io;

#[path = "restoration/piece_074.rs"]
mod collection_ledger;
pub use collection_ledger::{
    CollectOutcome, CollectionCatalogue, CollectionDefinition, CollectionLedger, CollectionProgress,
};

const MAX_COUNTER: u32 = 1_000_000;
const MAX_PACKET_BYTES: usize = 1024 * 1024;
const MAX_CLAIM_ITEMS: usize = 256;
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
                let (next, mut emitted) = apply_award(
                    &staged_rewards,
                    &rule.reward,
                    &format!("achievement:{}", rule.key),
                )?;
                staged_rewards = next;
                reward_packets.append(&mut emitted);
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

    pub fn contains(&self, title_id: u32) -> bool {
        self.owned.contains_key(&title_id)
    }

    pub fn add(&mut self, fields: [u32; 4]) -> io::Result<Vec<u8>> {
        let id = fields[0];
        if id == 0 || self.owned.contains_key(&id) {
            return Err(invalid("invalid or duplicate owned title"));
        }
        let mut fields = fields;
        fields[3] = 0;
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
        let mut packet = packet_u16_u32(152, 3);
        put_u32(&mut packet, count); // R: list count width; row width is client evidence.
        for (&id, fields) in &self.owned {
            let mut row = *fields;
            row[0] = id;
            row[3] = if self.active == Some(id) { 1 } else { 0 };
            for field in row {
                put_u32(&mut packet, field);
            }
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

// Claim records are supplied as opaque client-reader bytes. The client row
// reader's middle fields are not fully recovered; this module never invents them.

#[derive(Clone, Debug)]
struct PendingClaim {
    grant: RewardGrant,
    client_record: Vec<u8>,
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
    pub fn issue_server_grant(
        &mut self,
        claim_id: u64,
        grant: RewardGrant,
        client_record: Vec<u8>,
    ) -> io::Result<()> {
        validate_reward_rows(&grant.display.rows)?;
        if claim_id == 0
            || self.pending.contains_key(&claim_id)
            || self.claimed.contains(&claim_id)
            || client_record.is_empty()
            || client_record.len() > MAX_PACKET_BYTES
            || self.pending.len() >= MAX_CLAIM_ITEMS
        {
            return Err(invalid("invalid or duplicate pending claim"));
        }
        self.pending.insert(
            claim_id,
            PendingClaim {
                grant,
                client_record,
            },
        );
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

    pub fn claimed_ids(&self) -> &BTreeSet<u64> {
        &self.claimed
    }

    pub fn list_request() -> Vec<u8> {
        packet_u16_u16(108, 1)
    }

    pub fn list_response(&self) -> io::Result<Vec<u8>> {
        let records: Vec<_> = self
            .pending
            .values()
            .map(|item| item.client_record.as_slice())
            .collect();
        let mut packet = packet_u16_u16(108, 2);
        packet.push(1);
        packet.extend(encode_claim_array(&records)?);
        Ok(packet)
    }

    pub fn handle_claim_request(
        &mut self,
        request: &[u8],
        reward_state: &RewardState,
    ) -> io::Result<ClaimTransition> {
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
            || self.trackers.contains_key(&tracker_id)
            || ids.iter().copied().collect::<BTreeSet<_>>().len() != ids.len()
            || ids.iter().any(|id| !self.pending.contains_key(id))
        {
            return Ok(reject());
        }

        let grants: Vec<_> = ids.iter().map(|id| &self.pending[id].grant).collect();
        let grant = merge_grants(&grants)?;
        let grant_key = claim_grant_key(&ids);
        let (next_rewards, reward_packets) = apply_award(reward_state, &grant, &grant_key)?;
        let records: Vec<_> = ids
            .iter()
            .map(|id| self.pending[id].client_record.as_slice())
            .collect();
        let mut response = packet_u16_u16(108, 4);
        put_u32(&mut response, tracker_id);
        response.push(1);
        response.extend(encode_claim_array(&records)?);

        let item_ids: BTreeSet<u32> = grant
            .items
            .iter()
            .filter_map(|item| u32::try_from(item.definition_id).ok())
            .collect();
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

    pub fn handle_definitions_request(
        &mut self,
        request: &[u8],
        definitions_blob: &[u8],
    ) -> io::Result<Vec<u8>> {
        let (tracker_id, ids) = parse_definitions_request(request)?;
        let expected = self
            .trackers
            .get(&tracker_id)
            .ok_or_else(|| invalid("unknown claim tracker"))?;
        if ids.is_empty() || ids.iter().any(|id| !expected.contains(id)) {
            return Err(invalid("claim tracker does not own requested definitions"));
        }
        if definitions_blob.len() > MAX_PACKET_BYTES - 13 {
            return Err(invalid("claim definitions exceed packet bound"));
        }
        let len = i32::try_from(definitions_blob.len())
            .map_err(|_| invalid("claim definitions exceed wire length"))?;
        let mut response = packet_u16_u16(108, 6);
        put_u32(&mut response, tracker_id);
        response.push(1);
        response.extend(len.to_le_bytes());
        response.extend_from_slice(definitions_blob);
        self.trackers.remove(&tracker_id);
        Ok(response)
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
