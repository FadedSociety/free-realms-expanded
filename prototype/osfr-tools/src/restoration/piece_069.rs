//! 069: quest turn-in transactions for one restored server.
//!
//! This module is the exactly-once boundary between a completed quest objective and the reward
//! bundle that a turn-in grants. The quest engines in this crate flip `completed`/`rewarded` and
//! hand back a reward bundle that the caller then grants coin by coin and item by item
//! (`gameplay::apply_quest`, `gameplay::apply_opening`). A failure or a crash between those steps
//! leaves a rewarded quest with a partial or missing reward, and a repeated native turn-in request
//! has no recorded answer to replay. This module records a durable intent before anything is
//! granted, answers exact repeats from that record, and resolves a crash by looking at the
//! persisted quest state.
//!
//! ORIGINAL facts used here (all outside Git, pinned in the note):
//! * `lab/decompiled/server/LibSOE/SOE/BaseQuestPackets.cs` (supplied server) defines the quest
//!   family: `QuestAddPacket = 3`, `QuestCompletePacket = 4`, `QuestEndPacket = 13`,
//!   `QuestEndReplyPacket = 14`, `QuestObjectiveCompletePacket = 10`,
//!   `CompletedQuestCountUpdatePacket = 12`.
//! * `lab/runs/client-baseline-20261008T171251Z/analysis/native-cooking-finish-turnin-proof.json`
//!   shows one turn-in emitting exactly one `QuestCompletePacket` (frame 4071) and one
//!   `CompletedQuestCountUpdatePacket` (frame 4072, count 4) with one wallet delta 110->120 and
//!   one Chef XP delta 0->25, while the second Mae selection emitted no reward or progress packet.
//! * `lab/runs/client-baseline-20261008T193318Z/analysis-live/native-bartle-turnin-nextquest-proof.json`
//!   shows one choice frame, one wallet value, one XP value, one quest removal and one next-quest
//!   add with `reward_emitted_once: true`.
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`
//!   independently maps family `49` to these opcode names and family `50` to the reward packets;
//!   neither native turn-in above carries a family-50 packet, so the original reward-bundle path
//!   stays out of scope here.
//!
//! RECONSTRUCTED here: the ledger document, its record schema, the write-ahead ordering, every
//! rejection and every bound. No original server transaction source was recovered, and the native
//! client has not accepted anything this module produces. The earlier engines' acceptance runs are
//! evidence that the *observable* once-only reward exists, not that this ledger produced it.
//!
//! Known ceiling: a recorded quest answers only its exact request or a rewarded replay. A
//! repeatable quest that legitimately returns to an unrewarded state for a later attempt is
//! refused as `StaleAttempt` until that flow has captured evidence, because neither native proof
//! shows the same quest id turned in twice.
//!
//! Caller contract (the commit point stays outside this module):
//! * `consider` validates a turn-in request against the objective engine's progress and records a
//!   durable intent exactly once per `(quest_id, attempt)`.
//!   The caller resolves `progress` by `request.quest_id`; an unresolvable quest is `None` and is
//!   refused as `UnknownQuest` rather than judged against another quest's progress.
//! * `Apply` hands back the one staged bundle for that intent. The caller applies it and persists
//!   the whole gameplay snapshot with the existing atomic replace (`game_store::save`), which is
//!   the commit point: quest flags, items, currency and XP land together or not at all.
//! * `commit` records that the snapshot landed. `recover` is the only crash path: an unsettled
//!   intent either replays its bundle once (snapshot never landed) or settles without a grant
//!   (the persisted quest state already shows the turn-in).
//! * This module never mints currency, items or XP, and never writes the gameplay snapshot.
//!
//! ponytail: one append-only JSONL document, one linear scan, no second index and no background
//! writer. If a long-lived world ever needs per-account isolation, put one ledger beside each
//! account's snapshot instead of indexing here.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[path = "../quests.rs"]
#[allow(dead_code)]
mod quests;

// INTEGRATION (lead-owned): `quests` is already `pub mod quests;` in `src/lib.rs`, so when this
// module is registered, replace the `#[path]` include above with
// `pub use crate::quests::{QuestProgress, QuestState};` to keep the same public alias. Until then
// the owned test target includes this file by path exactly as `tests/piece_069.rs` does.
pub use quests::{QuestProgress, QuestState};

/// ORIGINAL: family byte observed on every native quest packet in the two turn-in proofs.
pub const QUEST_PACKET_FAMILY: u16 = 0x0031;
/// ORIGINAL: `BaseQuestPackets.QuestCompletePacket`.
pub const QUEST_COMPLETE_PACKET: u16 = 4;
/// ORIGINAL: `BaseQuestPackets.CompletedQuestCountUpdatePacket`.
pub const COMPLETED_QUEST_COUNT_UPDATE_PACKET: u16 = 12;
/// ORIGINAL: `BaseQuestPackets.QuestEndPacket` (the client's turn-in request).
pub const QUEST_END_PACKET: u16 = 13;
/// ORIGINAL: `BaseQuestPackets.QuestEndReplyPacket`.
pub const QUEST_END_REPLY_PACKET: u16 = 14;
/// ORIGINAL: `BaseQuestPackets.QuestAddPacket` (the next-quest accept in both proofs).
pub const QUEST_ADD_PACKET: u16 = 3;
/// ORIGINAL: `BaseQuestPackets.QuestObjectiveCompletePacket`.
pub const QUEST_OBJECTIVE_COMPLETE_PACKET: u16 = 10;
/// ORIGINAL order of the two packets a completed turn-in emitted in the Cooking proof
/// (frames 4071/4072). Wire serialization is item 070's; this module only pins the order.
pub const RECOVERED_TURN_IN_PACKETS: [u16; 2] =
    [QUEST_COMPLETE_PACKET, COMPLETED_QUEST_COUNT_UPDATE_PACKET];

/// RECONSTRUCTED ledger schema version.
pub const LEDGER_SCHEMA_VERSION: u32 = 1;
/// RECONSTRUCTED document bound, in the same spirit as the other owned snapshots.
pub const MAX_LEDGER_BYTES: usize = 262_144;
/// RECONSTRUCTED record bound. Halved by the intent/applied pair of one turn-in.
pub const MAX_LEDGER_RECORDS: usize = 4_096;
/// RECONSTRUCTED item-row bound for one turn-in bundle.
pub const MAX_TURN_IN_ITEMS: usize = 16;
/// RECONSTRUCTED experience bound for one turn-in bundle.
pub const MAX_TURN_IN_EXPERIENCE: u32 = 10_000_000;

fn invalid(text: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, text)
}

/// Every reward the caller must apply for one turn-in. RECONSTRUCTED container over content the
/// caller already owns; the numbers themselves stay in the quest content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnInReward {
    pub coins: u32,
    pub experience: u32,
    pub items: Vec<(i32, u32)>,
}

impl TurnInReward {
    fn is_valid(&self) -> bool {
        self.coins <= i32::MAX as u32
            && self.experience <= MAX_TURN_IN_EXPERIENCE
            && self.items.len() <= MAX_TURN_IN_ITEMS
            && self
                .items
                .iter()
                .all(|(id, quantity)| *id > 0 && *quantity > 0 && *quantity <= i32::MAX as u32)
            && (self.coins != 0 || !self.items.is_empty() || self.experience != 0)
    }
}

/// One native turn-in attempt. `attempt` is the objective engine's monotonic counter for the
/// quest; `request_id` identifies the client request that produced it. Both are caller-supplied
/// and only compared for equality and ordering here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurnInRequest {
    pub quest_id: u32,
    pub attempt: u32,
    pub request_id: u64,
}

/// Why a request was refused. RECONSTRUCTED classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnInRejection {
    /// Zero quest/attempt/request identity or an out-of-bound reward.
    InvalidRequest,
    /// The objective engine has no such quest.
    UnknownQuest,
    /// The quest exists but was never accepted.
    NotAccepted,
    /// Accepted, but the ordered objectives are not finished yet.
    ObjectivesIncomplete,
    /// The persisted state says the reward was already granted, but this ledger has no record of
    /// it. Refused instead of replayed: the ledger cannot prove what was granted.
    AlreadyRewarded,
    /// An older or reused attempt for a quest this ledger has already recorded.
    StaleAttempt { recorded: u32 },
}

/// A turn-in this ledger has recorded and that the caller must now apply exactly once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnInStaged {
    quest_id: u32,
    attempt: u32,
    request_id: u64,
    reward: TurnInReward,
}

impl TurnInStaged {
    pub fn quest_id(&self) -> u32 {
        self.quest_id
    }

    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    pub fn request_id(&self) -> u64 {
        self.request_id
    }

    pub fn reward(&self) -> &TurnInReward {
        &self.reward
    }
}

/// The recorded answer for one turn-in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurnInReceipt {
    pub quest_id: u32,
    pub attempt: u32,
    pub request_id: u64,
    /// True when recovery settled the receipt from the persisted quest state instead of from a
    /// `commit` call.
    pub settled_from_state: bool,
}

/// What the caller must do for a turn-in request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnInDecision {
    /// Apply this bundle, persist the whole snapshot, then call `commit`.
    Apply(TurnInStaged),
    /// Already turned in; grant nothing.
    Replay(TurnInReceipt),
    /// Refused; nothing changed.
    Reject(TurnInRejection),
}

/// What one unsettled intent needs after a restart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnInRecovery {
    /// The snapshot never landed: apply this bundle once, persist, then `commit`.
    Replay(TurnInStaged),
    /// The persisted state already shows the turn-in: grant nothing.
    Settled(TurnInReceipt),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LedgerKind {
    Intent,
    Applied,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LedgerRecord {
    schema: u32,
    kind: LedgerKind,
    quest_id: u32,
    attempt: u32,
    request_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reward: Option<TurnInReward>,
}

impl LedgerRecord {
    fn staged(&self) -> Option<TurnInStaged> {
        Some(TurnInStaged {
            quest_id: self.quest_id,
            attempt: self.attempt,
            request_id: self.request_id,
            reward: self.reward.clone()?,
        })
    }

    fn receipt(&self, settled_from_state: bool) -> TurnInReceipt {
        TurnInReceipt {
            quest_id: self.quest_id,
            attempt: self.attempt,
            request_id: self.request_id,
            settled_from_state,
        }
    }
}

/// Durable JSONL ledger of turn-in intents and their applied records, one turn-in per pair.
#[derive(Clone, Debug)]
pub struct TurnInLedger {
    path: PathBuf,
    records: Vec<LedgerRecord>,
}

impl TurnInLedger {
    /// Open or create the ledger. An unreadable, oversized, malformed or inconsistent document is
    /// an error; it is never truncated, repaired or silently replaced.
    pub fn open(path: &Path) -> io::Result<Self> {
        let records = match fs::symlink_metadata(path) {
            Ok(metadata) => {
                if !metadata.is_file() || metadata.file_type().is_symlink() {
                    return Err(invalid("ledger is not a bounded regular file"));
                }
                if metadata.len() > MAX_LEDGER_BYTES as u64 {
                    return Err(invalid("ledger exceeds its byte bound"));
                }
                let mut bytes = Vec::new();
                fs::File::open(path)?
                    .take(MAX_LEDGER_BYTES as u64 + 1)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > MAX_LEDGER_BYTES {
                    return Err(invalid("ledger exceeds its byte bound"));
                }
                parse_records(&bytes)?
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error),
        };
        Ok(Self {
            path: path.to_path_buf(),
            records,
        })
    }

    pub fn records(&self) -> usize {
        self.records.len()
    }

    /// The applied receipt for a quest, if this ledger has one.
    pub fn receipt(&self, quest_id: u32) -> Option<TurnInReceipt> {
        self.records
            .iter()
            .find(|record| record.kind == LedgerKind::Applied && record.quest_id == quest_id)
            .map(|record| record.receipt(false))
    }

    /// Validate one turn-in request and record its intent exactly once.
    pub fn consider(
        &mut self,
        request: TurnInRequest,
        progress: Option<&QuestProgress>,
        reward: &TurnInReward,
    ) -> io::Result<TurnInDecision> {
        if !reward.is_valid() {
            return Err(invalid("invalid turn-in reward bundle"));
        }
        if request.quest_id == 0 || request.attempt == 0 || request.request_id == 0 {
            return Ok(TurnInDecision::Reject(TurnInRejection::InvalidRequest));
        }
        let Some(progress) = progress else {
            return Ok(TurnInDecision::Reject(TurnInRejection::UnknownQuest));
        };
        if !progress.accepted {
            return Ok(TurnInDecision::Reject(TurnInRejection::NotAccepted));
        }
        if !progress.completed && !progress.rewarded {
            return Ok(TurnInDecision::Reject(
                TurnInRejection::ObjectivesIncomplete,
            ));
        }

        let recorded = self.recorded_attempt(request.quest_id);
        if let Some(applied) = self.records.iter().find(|record| {
            record.kind == LedgerKind::Applied && record.quest_id == request.quest_id
        }) {
            if progress.rewarded
                || (applied.attempt == request.attempt && applied.request_id == request.request_id)
            {
                return Ok(TurnInDecision::Replay(applied.receipt(false)));
            }
            return Ok(TurnInDecision::Reject(TurnInRejection::StaleAttempt {
                recorded,
            }));
        }
        if progress.rewarded {
            return Ok(TurnInDecision::Reject(TurnInRejection::AlreadyRewarded));
        }
        if let Some(intent) = self
            .records
            .iter()
            .find(|record| record.kind == LedgerKind::Intent && record.quest_id == request.quest_id)
        {
            if intent.attempt == request.attempt && intent.request_id == request.request_id {
                // Same intent, no second record: Apply stays the one staged bundle for it.
                return Ok(TurnInDecision::Apply(
                    intent
                        .staged()
                        .ok_or_else(|| invalid("intent record is missing its reward"))?,
                ));
            }
            return Ok(TurnInDecision::Reject(TurnInRejection::StaleAttempt {
                recorded,
            }));
        }
        if request.attempt <= recorded {
            return Ok(TurnInDecision::Reject(TurnInRejection::StaleAttempt {
                recorded,
            }));
        }

        let record = LedgerRecord {
            schema: LEDGER_SCHEMA_VERSION,
            kind: LedgerKind::Intent,
            quest_id: request.quest_id,
            attempt: request.attempt,
            request_id: request.request_id,
            reward: Some(reward.clone()),
        };
        self.append(&record)?;
        Ok(TurnInDecision::Apply(TurnInStaged {
            quest_id: request.quest_id,
            attempt: request.attempt,
            request_id: request.request_id,
            reward: reward.clone(),
        }))
    }

    /// Record that the caller's snapshot with this reward landed. Only a staged turn-in this
    /// ledger recorded can be committed, and only once.
    pub fn commit(&mut self, staged: TurnInStaged) -> io::Result<TurnInReceipt> {
        let intent = self.records.iter().find(|record| {
            record.kind == LedgerKind::Intent
                && record.quest_id == staged.quest_id
                && record.attempt == staged.attempt
                && record.request_id == staged.request_id
        });
        let Some(intent) = intent else {
            return Err(invalid("commit for a turn-in this ledger did not stage"));
        };
        if intent.reward.as_ref() != Some(&staged.reward) {
            return Err(invalid("committed bundle differs from the staged intent"));
        }
        if let Some(existing) = self.records.iter().find(|record| {
            record.kind == LedgerKind::Applied
                && record.quest_id == staged.quest_id
                && record.attempt == staged.attempt
        }) {
            // Idempotent: a repeated commit returns the record it already wrote.
            return Ok(existing.receipt(false));
        }
        let record = LedgerRecord {
            schema: LEDGER_SCHEMA_VERSION,
            kind: LedgerKind::Applied,
            quest_id: staged.quest_id,
            attempt: staged.attempt,
            request_id: staged.request_id,
            reward: None,
        };
        self.append(&record)?;
        Ok(record.receipt(false))
    }

    /// Resolve every intent whose snapshot has not been recorded yet. A caller runs this once
    /// after a restart, before accepting new turn-in requests.
    pub fn recover(&mut self, state: &QuestState) -> io::Result<Vec<TurnInRecovery>> {
        let pending: Vec<LedgerRecord> = self
            .records
            .iter()
            .filter(|record| record.kind == LedgerKind::Intent)
            .filter(|record| {
                !self.records.iter().any(|applied| {
                    applied.kind == LedgerKind::Applied
                        && applied.quest_id == record.quest_id
                        && applied.attempt == record.attempt
                })
            })
            .cloned()
            .collect();
        let mut actions = Vec::with_capacity(pending.len());
        for intent in pending {
            let rewarded = state
                .quests
                .get(&intent.quest_id)
                .is_some_and(|progress| progress.rewarded);
            if rewarded {
                let record = LedgerRecord {
                    schema: LEDGER_SCHEMA_VERSION,
                    kind: LedgerKind::Applied,
                    quest_id: intent.quest_id,
                    attempt: intent.attempt,
                    request_id: intent.request_id,
                    reward: None,
                };
                self.append(&record)?;
                actions.push(TurnInRecovery::Settled(record.receipt(true)));
            } else {
                let staged = intent
                    .staged()
                    .ok_or_else(|| invalid("intent record is missing its reward"))?;
                actions.push(TurnInRecovery::Replay(staged));
            }
        }
        Ok(actions)
    }

    fn recorded_attempt(&self, quest_id: u32) -> u32 {
        self.records
            .iter()
            .filter(|record| record.quest_id == quest_id)
            .map(|record| record.attempt)
            .max()
            .unwrap_or(0)
    }

    fn append(&mut self, record: &LedgerRecord) -> io::Result<()> {
        if self.records.len() >= MAX_LEDGER_RECORDS {
            return Err(invalid("ledger record bound reached"));
        }
        let mut line =
            serde_json::to_vec(record).map_err(|_| invalid("ledger record is not serializable"))?;
        if line.len() + self.records.len() > MAX_LEDGER_BYTES {
            return Err(invalid("ledger byte bound reached"));
        }
        line.push(b'\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(&line)?;
        file.flush()?;
        file.sync_data()?;
        self.records.push(record.clone());
        Ok(())
    }
}

fn parse_records(bytes: &[u8]) -> io::Result<Vec<LedgerRecord>> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("ledger is not UTF-8"))?;
    let mut records = Vec::new();
    let mut seen: BTreeMap<(u32, u32), LedgerKind> = BTreeMap::new();
    for line in text.split('\n') {
        if line.trim().is_empty() {
            continue;
        }
        if records.len() >= MAX_LEDGER_RECORDS {
            return Err(invalid("ledger record bound exceeded"));
        }
        let record: LedgerRecord =
            serde_json::from_str(line).map_err(|_| invalid("malformed ledger record"))?;
        if record.schema != LEDGER_SCHEMA_VERSION
            || record.quest_id == 0
            || record.attempt == 0
            || record.request_id == 0
        {
            return Err(invalid("unsupported ledger record"));
        }
        match record.kind {
            LedgerKind::Intent => {
                let reward = record
                    .reward
                    .as_ref()
                    .ok_or_else(|| invalid("intent record is missing its reward"))?;
                if !reward.is_valid() {
                    return Err(invalid("intent reward is out of bounds"));
                }
                if seen
                    .insert((record.quest_id, record.attempt), LedgerKind::Intent)
                    .is_some()
                {
                    return Err(invalid("duplicate ledger intent"));
                }
            }
            LedgerKind::Applied => {
                if record.reward.is_some() {
                    return Err(invalid("applied record carries a reward"));
                }
                match seen.get(&(record.quest_id, record.attempt)) {
                    Some(LedgerKind::Intent) => {
                        seen.insert((record.quest_id, record.attempt), LedgerKind::Applied);
                    }
                    _ => return Err(invalid("applied record without its intent")),
                }
            }
        }
        records.push(record);
    }
    Ok(records)
}
