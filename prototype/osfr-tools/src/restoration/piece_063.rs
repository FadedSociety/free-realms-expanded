//! Dungeon party rewards: completion fan-out, per-member parcels and replay rejection.
//!
//! Piece 063, owned paths only. Evidence labels follow the other restoration modules:
//! ORIGINAL is read from the supplied client, the pinned community sources or original capture
//! bytes; RECONSTRUCTED is a bounded local policy the lead must replace with native evidence;
//! MISSING is an explicit gap.
//!
//! ORIGINAL facts implemented here:
//! * the minigame family is opcode **39** (`0x27`). `FUN_008a4420(param_1, sub, a, b, c)` writes
//!   `param_1[1] = 0x27` and `param_1[2] = sub` before installing `BaseMiniGamePacket::vftable`
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`).
//! * the reward flow inside that family is recovered from the client constructors:
//!   `MiniGameRewardStatusPacket` = 22, `MiniGameLootWheelSetItemToLandOnPacket` = 45,
//!   `MiniGameLootWheelOnRotationStoppedPacket` = 46 and `MiniGameUpdateRewardPacket` = 68.
//!   The client dispatcher decodes 45 and 68 (their cases call the constructors and then read the
//!   body), while 46 is the client's own report. The pinned community enum and reader carry the same
//!   four sub-opcodes.
//! * the minigame header is `u8 sub | i32 state | i32 group | i32 game`. Three original retail
//!   capture frames satisfy it byte-for-byte, so 45/46 carry the same group and game identity as the
//!   encounter they belong to.
//! * the original score screen (decompiled ActionScript) sends `OnLootWheelSpinRequest`, applies the
//!   landing slice the server already chose through `StopLootWheelAt`, and then sends
//!   `OnLootWheelRotationStopped`. The prize is therefore server-decided and the stop report is a
//!   claim that must be granted at most once.
//!
//! RECONSTRUCTED here (no original server reward code was recovered):
//! * every party rule: a run is opened with an explicit participant roster; any participant may
//!   report the win; the win completes the run for every participant at once and entitles each of
//!   them to one parcel; each member claims their own parcel once, at most.
//! * the parcel contents themselves. The caller supplies one [`RewardBundle`] for the run and this
//!   module deliberately applies no per-member share, drop table or membership bonus arithmetic.
//!
//! MISSING / explicit gaps:
//! * the per-message bodies of 39/22, 39/45, 39/46 and 39/68 were not recovered: no captured
//!   family-39 reward traffic and no original serializer exist, so only the header is parsed here.
//! * no original dungeon reward table, coin/item split or per-member payout rule was recovered.
//! * membership (`MembersOnly`) bonuses, disconnect/reconnect credit and mid-run join credit are
//!   owned by the instance-lifecycle and membership items, not by this module.
//!
//! ponytail: one map of runs, one set of claimed players, three validation passes. No trait objects,
//! no packet writer for an unrecovered body, no reward arithmetic. Grow the body codec only when a
//! captured family-39 reward frame or an original serializer exists; grow the share rules only when
//! an original reward table exists.

use std::collections::{BTreeMap, BTreeSet};

/// A character guid. The wire uses 64-bit character guids throughout the original captures.
pub type PlayerId = u64;

/// ORIGINAL: `BaseMiniGamePacket`, read from the client constructor `FUN_008a4420`.
pub const MINIGAME_FAMILY: u16 = 0x27;

/// ORIGINAL: the recovered minigame header, in bytes, after the family `u16`.
pub const MINIGAME_HEADER_LEN: usize = 1 + 3 * 4;

/// Who sends a recovered reward message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardFlow {
    /// Server to client.
    ServerToClient,
    /// Client to server.
    ClientToServer,
}

/// ORIGINAL: the reward surface of `BaseMiniGamePacket` (family 39), read from the client
/// constructors and cross-checked against the pinned community enum and reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiniGameRewardMessage {
    /// 39/22 - reward state for the results screen.
    RewardStatus,
    /// 39/45 - the landing prize the server already chose for the winner's loot wheel.
    LootWheelSetItemToLandOn,
    /// 39/46 - the client reports that the wheel stopped; the server grants the prize.
    LootWheelOnRotationStopped,
    /// 39/68 - a reward update for the results screen.
    UpdateReward,
}

impl MiniGameRewardMessage {
    /// Every recovered reward sub-opcode.
    pub const ALL: [MiniGameRewardMessage; 4] = [
        MiniGameRewardMessage::RewardStatus,
        MiniGameRewardMessage::LootWheelSetItemToLandOn,
        MiniGameRewardMessage::LootWheelOnRotationStopped,
        MiniGameRewardMessage::UpdateReward,
    ];

    pub fn sub_opcode(self) -> u8 {
        match self {
            MiniGameRewardMessage::RewardStatus => 22,
            MiniGameRewardMessage::LootWheelSetItemToLandOn => 45,
            MiniGameRewardMessage::LootWheelOnRotationStopped => 46,
            MiniGameRewardMessage::UpdateReward => 68,
        }
    }

    /// The class the supplied client constructs for this sub-opcode.
    pub fn class_name(self) -> &'static str {
        match self {
            MiniGameRewardMessage::RewardStatus => "MiniGameRewardStatusPacket",
            MiniGameRewardMessage::LootWheelSetItemToLandOn => {
                "MiniGameLootWheelSetItemToLandOnPacket"
            }
            MiniGameRewardMessage::LootWheelOnRotationStopped => {
                "MiniGameLootWheelOnRotationStoppedPacket"
            }
            MiniGameRewardMessage::UpdateReward => "MiniGameUpdateRewardPacket",
        }
    }

    pub fn flow(self) -> RewardFlow {
        match self {
            MiniGameRewardMessage::LootWheelOnRotationStopped => RewardFlow::ClientToServer,
            _ => RewardFlow::ServerToClient,
        }
    }

    pub fn from_sub(sub: u8) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|message| message.sub_opcode() == sub)
    }
}

/// A parsed minigame header. `extra` stays opaque: the per-message bodies were not recovered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MiniGameHeader {
    pub sub: u8,
    pub state_id: i32,
    /// The instance/group identity carried by every minigame packet.
    pub group_id: i32,
    /// The activity/game identity.
    pub game_id: i32,
    /// Every byte after the recovered header, uninterpreted.
    pub extra: Vec<u8>,
}

/// Parse the recovered minigame header from `bytes`, which start at the sub-opcode (that is,
/// immediately after the family `u16`).
pub fn parse_minigame_header(bytes: &[u8]) -> Result<MiniGameHeader, RewardError> {
    if bytes.len() < MINIGAME_HEADER_LEN {
        return Err(RewardError::MalformedHeader);
    }
    Ok(MiniGameHeader {
        sub: bytes[0],
        state_id: i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]),
        group_id: i32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
        game_id: i32::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]),
        extra: bytes[MINIGAME_HEADER_LEN..].to_vec(),
    })
}

/// RECONSTRUCTED: the parcel shape the caller supplies and this module fans out.
///
/// It matches the existing encounter reward shape (`coins` plus item-definition/quantity rows) so
/// the lead's economy layer can apply it unchanged. No original dungeon reward table was recovered,
/// so every value here is caller-supplied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardBundle {
    pub coins: u32,
    pub items: Vec<(i32, u32)>,
}

/// What the caller should tell the clients after a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Opened { instance_id: i32, game_id: i32 },
}

/// The result of a completed run: the shared parcel and every member it is owed to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinishOutcome {
    pub instance_id: i32,
    pub game_id: i32,
    /// Every member entitled to the parcel, in roster order.
    pub eligible: Vec<PlayerId>,
    /// Every member whose completion was recorded, in roster order. Identical to `eligible` under
    /// the reconstructed policy.
    pub completed: Vec<PlayerId>,
    pub shared: RewardBundle,
}

/// Every refusal is explicit; nothing here silently succeeds or invents a payout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardError {
    /// The bytes are shorter than the recovered minigame header.
    MalformedHeader,
    /// The instance id is not a positive run identity.
    InvalidInstance,
    /// A run was opened with no participants.
    EmptyRoster,
    /// The same player appears twice in one roster.
    DuplicateParticipant,
    /// A run is already open for that instance id.
    AlreadyOpen,
    /// That instance id has already completed; it can never pay out again.
    AlreadyFinished,
    /// The run has not completed yet.
    NotFinished,
    /// The player is not on the run's participant roster.
    NotEligible,
    /// This player already claimed their parcel.
    AlreadyClaimed,
    /// No run exists for that instance id.
    UnknownRun,
}

#[derive(Clone, Debug)]
struct RunRecord {
    game_id: i32,
    participants: Vec<PlayerId>,
    bundle: RewardBundle,
    finished: bool,
    claimed: BTreeSet<PlayerId>,
}

/// A bounded, evidence-labelled dungeon party reward ledger.
///
/// RECONSTRUCTED policy summary: the owner of a run supplies its participant roster and shared
/// parcel; any participant may report the completion; one completion completes the run for every
/// participant and entitles each of them to the parcel; each participant claims their own parcel at
/// most once; a finished instance id is refused forever, so a replayed finish or a stale instance id
/// can never pay out twice.
#[derive(Clone, Debug, Default)]
pub struct DungeonRewardLedger {
    runs: BTreeMap<i32, RunRecord>,
    finished: BTreeSet<i32>,
}

impl DungeonRewardLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one dungeon run with its participant roster and shared parcel.
    pub fn open(
        &mut self,
        instance_id: i32,
        game_id: i32,
        participants: &[PlayerId],
        bundle: RewardBundle,
    ) -> Result<Reply, RewardError> {
        if instance_id <= 0 {
            return Err(RewardError::InvalidInstance);
        }
        if participants.is_empty() {
            return Err(RewardError::EmptyRoster);
        }
        if has_duplicates(participants) {
            return Err(RewardError::DuplicateParticipant);
        }
        if self.finished.contains(&instance_id) {
            return Err(RewardError::AlreadyFinished);
        }
        if self.runs.contains_key(&instance_id) {
            return Err(RewardError::AlreadyOpen);
        }
        self.runs.insert(
            instance_id,
            RunRecord {
                game_id,
                participants: participants.to_vec(),
                bundle,
                finished: false,
                claimed: BTreeSet::new(),
            },
        );
        Ok(Reply::Opened {
            instance_id,
            game_id,
        })
    }

    /// Record the run's completion once and fan the shared parcel out to every participant.
    ///
    /// A second completion for the same instance id is refused forever, including after the roster
    /// has been dropped from the live map.
    pub fn finish(
        &mut self,
        instance_id: i32,
        actor: PlayerId,
    ) -> Result<FinishOutcome, RewardError> {
        let run = self.runs.get(&instance_id).ok_or(RewardError::UnknownRun)?;
        if !run.participants.contains(&actor) {
            return Err(RewardError::NotEligible);
        }
        if run.finished {
            return Err(RewardError::AlreadyFinished);
        }
        let run = self.runs.get_mut(&instance_id).expect("run checked above");
        run.finished = true;
        let outcome = FinishOutcome {
            instance_id,
            game_id: run.game_id,
            eligible: run.participants.clone(),
            completed: run.participants.clone(),
            shared: run.bundle.clone(),
        };
        self.finished.insert(instance_id);
        Ok(outcome)
    }

    /// Whether the given member still owes a claim on this run.
    pub fn pending(&self, instance_id: i32, player: PlayerId) -> Result<bool, RewardError> {
        let run = self.runs.get(&instance_id).ok_or(RewardError::UnknownRun)?;
        if !run.participants.contains(&player) {
            return Err(RewardError::NotEligible);
        }
        if !run.finished {
            return Err(RewardError::NotFinished);
        }
        Ok(!run.claimed.contains(&player))
    }

    /// Claim one member's parcel. This is the grant the 39/46 rotation-stopped report triggers, so a
    /// repeated report for the same member is refused instead of granting again.
    pub fn claim(
        &mut self,
        instance_id: i32,
        player: PlayerId,
    ) -> Result<RewardBundle, RewardError> {
        let run = self.runs.get(&instance_id).ok_or(RewardError::UnknownRun)?;
        if !run.participants.contains(&player) {
            return Err(RewardError::NotEligible);
        }
        if !run.finished {
            return Err(RewardError::NotFinished);
        }
        if run.claimed.contains(&player) {
            return Err(RewardError::AlreadyClaimed);
        }
        let run = self.runs.get_mut(&instance_id).expect("run checked above");
        run.claimed.insert(player);
        Ok(run.bundle.clone())
    }

    /// Whether the run has completed. Finished ids stay refused even after the run is dropped.
    pub fn is_finished(&self, instance_id: i32) -> bool {
        self.finished.contains(&instance_id)
    }

    /// Every member whose completion was recorded for this run.
    pub fn completed(&self, instance_id: i32) -> Result<Vec<PlayerId>, RewardError> {
        let run = self.runs.get(&instance_id).ok_or(RewardError::UnknownRun)?;
        if !run.finished {
            return Err(RewardError::NotFinished);
        }
        Ok(run.participants.clone())
    }

    /// A stable JSON snapshot in the same shape as the other restoration modules.
    pub fn state(&self) -> serde_json::Value {
        let runs: Vec<serde_json::Value> = self
            .runs
            .iter()
            .map(|(instance_id, run)| {
                serde_json::json!({
                    "instance_id": instance_id,
                    "game_id": run.game_id,
                    "finished": run.finished,
                    "coins": run.bundle.coins,
                    "items": run.bundle.items,
                    "participants": run.participants,
                    "claimed": run.claimed.iter().collect::<Vec<_>>(),
                })
            })
            .collect();
        let finished: Vec<i32> = self.finished.iter().copied().collect();
        serde_json::json!({ "runs": runs, "finished": finished })
    }
}

fn has_duplicates(participants: &[PlayerId]) -> bool {
    let mut seen = BTreeSet::new();
    participants.iter().any(|player| !seen.insert(*player))
}
