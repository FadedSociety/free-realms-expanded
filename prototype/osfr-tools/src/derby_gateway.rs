//! Optional ordinary-entry binding for the native demolition derby.
//!
//! The packet and match rules stay in `derby.rs`; this module binds one authenticated character to
//! one opt-in content package. The gate list is reconstructed policy (R), like the kart gates: an
//! NPC lists the derby only when named in `derby_gates.json`. Entry, phase acknowledgements and the
//! results hold use the existing native type-11 lifecycle.
//!
//! T219 pays a flat completion reward: the server-observed match reaching its results phase with
//! this character still in it. Every collision is a client `34/16` report, so the payout is
//! deliberately independent of contacts, damage and score; only finishing matters, and farming
//! finished matches is allowed. `paid_matches` records every paid match instance in the character
//! save, so a repeated results record or a reconnect cannot pay twice, and the instance counter
//! resumes above the ledger so a paid id is not reissued. F164's `reward` row and F117's
//! `reward_cooldown_s` stay accepted and ignored for F206 migration. No source-derived rule is
//! invented here; all tunable numbers come from `derby_entry.json`.

use crate::{
    character_store::CharacterState,
    derby::{
        DEMO_DERBY_LOBBY_NAME_ID, DerbyAction, DerbyError, DerbyPhase, DerbyPickup, DerbyRules,
        DerbyService, DerbyVehicleTemplate, ParticipantKey,
    },
    rewards::{RewardDisplay, RewardGrant},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read},
    path::Path,
    sync::Arc,
};

/// ORIGINAL: profile 49 is the Demo Derby Driver job (`Piece 041`).
pub const DEMO_DERBY_DRIVER_PROFILE: i32 = 49;

/// (R) The lobby label is used for the NPC option until a menu-specific label is recovered.
pub const DEFAULT_MENU_STRING_ID: u32 = DEMO_DERBY_LOBBY_NAME_ID as u32;

/// (R) Private instance ids start here, matching the kart entry convention.
pub const FIRST_DERBY_INSTANCE: u32 = 49_000_001;
pub const MAX_DERBY_GATES: usize = 16;

const MAX_CONTENT_BYTES: usize = 512 * 1024;
const MAX_GATES_BYTES: usize = 16 * 1024;
/// (R) Upper bound on each completion reward, so content cannot mint unlimited coins or XP.
pub const MAX_COMPLETION_REWARD: u32 = 10_000;
/// Save-size bound on the paid-match ledger; the newest rows are kept.
pub const MAX_PAID_MATCHES: usize = 32;
const ACTIVE_STATUS_MASK: u32 = (1 << crate::derby::DERBY_ACTIVE_STATUS_BIT) | 1;

/// One opt-in derby package. `vehicle`, `rules` and `pickups` are the service's existing typed
/// content; this wrapper adds the game id and the flat completion reward.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyGatewayContent {
    pub schema_version: u32,
    pub game_id: i32,
    pub vehicle: DerbyVehicleTemplate,
    pub rules: DerbyRules,
    #[serde(default)]
    pub pickups: Vec<DerbyPickup>,
    /// (R) Flat coins for finishing a match, paid once per match instance. A package with zero
    /// coins and zero XP pays nothing, so an operator file without these keys still loads.
    #[serde(default)]
    pub completion_reward_coins: u32,
    /// (R) Demo Derby Driver XP for the same completion; 0 pays no XP.
    #[serde(default)]
    pub completion_reward_xp: u32,
    /// Retired F117 payout row. Accepted and ignored only so an operator's F117-era
    /// `derby_entry.json` still loads; `completion_reward_coins`/`_xp` replaced it (F206/T219).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward: Option<serde_json::Value>,
}

/// (R) NPCs whose menu offers the derby. NPC existence is checked by `NpcInteraction`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyGates {
    pub schema_version: u32,
    #[serde(default = "default_menu_string_id")]
    pub menu_string_id: u32,
    pub npcs: Vec<u64>,
    /// Retired F117 paid-run cooldown. Accepted and ignored only so an operator's F117-era
    /// `derby_gates.json` still loads; the payout it paced no longer exists (F206).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward_cooldown_s: Option<u64>,
}

/// Derby save data. T023 wrote `previous_job` and F117 wrote `last_paid_unix`; both are accepted
/// and neither is written again. Current saves carry `paid_matches`, the completion ledger.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyGatewayState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_job: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_paid_unix: Option<u64>,
    /// Paid match instances, oldest first, at most the newest `MAX_PAID_MATCHES`. The save belongs
    /// to one character, so (this ledger, instance id) is the (character, match) key and another
    /// character's identical instance id is a different match. `next_instance` resumes above the
    /// newest row, so a pruned id is never reissued to a character that was paid for it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paid_matches: Vec<i32>,
}

fn default_menu_string_id() -> u32 {
    DEFAULT_MENU_STRING_ID
}

/// Why an ordinary derby entry was refused. The caller reports the message and changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerbyEntryDenied {
    NotAGate,
    Unavailable,
    AlreadyPlaying,
    Busy,
}

impl DerbyEntryDenied {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotAGate => "The derby isn't available here.",
            Self::Unavailable => "The derby isn't open yet.",
            Self::AlreadyPlaying => "You are already in a derby.",
            Self::Busy => "Finish the active activity before entering the derby.",
        }
    }
}

#[derive(Clone)]
pub struct DerbyGateway {
    content: Arc<DerbyGatewayContent>,
    gates: Option<Arc<DerbyGates>>,
    service: Option<DerbyService>,
    owner: Option<ParticipantKey>,
    next_instance: u32,
    previous_job: Option<i32>,
    /// Completion ledger, newest last; see `DerbyGatewayState::paid_matches`.
    paid_matches: Vec<i32>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

fn read_regular_limited(path: &Path, limit: usize, what: &str) -> io::Result<Option<Vec<u8>>> {
    let metadata = match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
        Ok(metadata) => metadata,
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(invalid(format!("{what} must be a regular file")));
    }
    if metadata.len() > limit as u64 {
        return Err(invalid(format!("{what} exceeds its size bound")));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(invalid(format!("{what} exceeds its size bound")));
    }
    Ok(Some(bytes))
}

fn load_gates(directory: &Path) -> io::Result<Option<DerbyGates>> {
    let path = directory.join("derby_gates.json");
    let Some(bytes) = read_regular_limited(&path, MAX_GATES_BYTES, "derby gates")? else {
        return Ok(None);
    };
    let gates: DerbyGates = serde_json::from_slice(&bytes).map_err(invalid)?;
    if gates.schema_version != 1
        || gates.menu_string_id == 0
        || gates.npcs.len() > MAX_DERBY_GATES
        || gates
            .npcs
            .iter()
            .try_fold(BTreeSet::new(), |mut seen, npc| {
                (*npc != 0 && seen.insert(*npc)).then_some(seen)
            })
            .is_none()
    {
        return Err(invalid("invalid derby gate list"));
    }
    Ok(Some(gates))
}

impl DerbyGateway {
    /// Load the content package, its optional NPC gate list and this character's completion
    /// ledger. Absent content leaves the route disabled; malformed present content or ledger
    /// rejects the world load instead of degrading silently.
    pub fn load(directory: &Path, saved: Option<&DerbyGatewayState>) -> io::Result<Option<Self>> {
        let path = directory.join("derby_entry.json");
        let Some(bytes) = read_regular_limited(&path, MAX_CONTENT_BYTES, "derby content")? else {
            return Ok(None);
        };
        let content: DerbyGatewayContent =
            serde_json::from_slice(&bytes).map_err(|error| invalid(error.to_string()))?;
        let mut gateway = Self::new(content, load_gates(directory)?)?;
        gateway.restore_ledger(saved)?;
        Ok(Some(gateway))
    }

    /// Restore the paid-match ledger and resume the instance counter above its newest row. A
    /// non-positive or oversized ledger fails the load rather than risking a second payout.
    fn restore_ledger(&mut self, saved: Option<&DerbyGatewayState>) -> io::Result<()> {
        let Some(saved) = saved else {
            return Ok(());
        };
        if saved.paid_matches.len() > MAX_PAID_MATCHES
            || saved.paid_matches.iter().any(|instance| *instance <= 0)
        {
            return Err(invalid("invalid derby paid-match ledger"));
        }
        self.paid_matches = saved.paid_matches.clone();
        let newest = self
            .paid_matches
            .iter()
            .map(|instance| *instance as u32)
            .max();
        self.next_instance = newest.map_or(FIRST_DERBY_INSTANCE, |newest| {
            newest.saturating_add(1).max(FIRST_DERBY_INSTANCE)
        });
        Ok(())
    }

    /// Build a gateway from typed content. This is also the strict content validator used by tests
    /// and by interactive content reloads.
    pub fn new(content: DerbyGatewayContent, gates: Option<DerbyGates>) -> io::Result<Self> {
        validate_content(&content)?;
        if let Some(gates) = &gates {
            if gates.schema_version != 1
                || gates.menu_string_id == 0
                || gates.npcs.len() > MAX_DERBY_GATES
                || gates.npcs.iter().any(|npc| *npc == 0)
                || gates.npcs.iter().collect::<BTreeSet<_>>().len() != gates.npcs.len()
            {
                return Err(invalid("invalid derby gate list"));
            }
        }
        Ok(Self {
            content: Arc::new(content),
            gates: gates.map(Arc::new),
            service: None,
            owner: None,
            next_instance: FIRST_DERBY_INSTANCE,
            previous_job: None,
            paid_matches: Vec::new(),
        })
    }

    /// NPC content keys whose menu may list the derby.
    pub fn gate_npcs(&self) -> &[u64] {
        self.gates
            .as_ref()
            .map_or(&[], |gates| gates.npcs.as_slice())
    }

    pub fn menu_string_id(&self) -> u32 {
        self.gates
            .as_ref()
            .map_or(DEFAULT_MENU_STRING_ID, |gates| gates.menu_string_id)
    }

    pub fn active(&self) -> bool {
        self.service.as_ref().is_some_and(|service| {
            !matches!(service.phase(), DerbyPhase::Closed | DerbyPhase::Aborted)
        })
    }

    pub fn phase(&self) -> Option<DerbyPhase> {
        self.service.as_ref().map(DerbyService::phase)
    }

    pub fn instance_id(&self) -> Option<i32> {
        self.service.as_ref().map(DerbyService::instance_id)
    }

    pub fn previous_job(&self) -> Option<i32> {
        self.previous_job
    }

    pub fn set_previous_job(&mut self, job: i32) {
        self.previous_job = Some(job);
    }

    /// Ordinary-entry admission. `busy` is computed by gameplay before any state is staged.
    pub fn admit(&self, npc_id: u64, busy: bool) -> Result<(), DerbyEntryDenied> {
        if !self
            .gates
            .as_ref()
            .is_some_and(|gates| gates.npcs.contains(&npc_id))
        {
            return Err(DerbyEntryDenied::NotAGate);
        }
        if self.active() {
            return Err(DerbyEntryDenied::AlreadyPlaying);
        }
        if busy {
            return Err(DerbyEntryDenied::Busy);
        }
        Ok(())
    }

    /// Start a fresh solo derby and return the native 39/16 offer. The client answers with 39/5.
    pub fn begin(&mut self, character: &CharacterState) -> io::Result<Vec<u8>> {
        self.begin_at(character, 0)
    }

    pub fn begin_at(&mut self, character: &CharacterState, now_ms: u64) -> io::Result<Vec<u8>> {
        if self.active() {
            return Err(invalid("derby is already active"));
        }
        if character.guid == 0 {
            return Err(invalid("derby entry requires an authenticated character"));
        }
        let instance = self.next_instance;
        if instance > i32::MAX as u32 {
            return Err(invalid("derby instance overflow"));
        }
        let owner = ParticipantKey(character.guid);
        let mut service = DerbyService::new(
            instance as i32,
            self.content.game_id,
            owner,
            self.content.vehicle.clone(),
            self.content.rules.clone(),
            self.content.pickups.clone(),
        )
        .map_err(|error| invalid(error.to_string()))?;
        service.set_lobby_deadline(now_ms);
        let offer = service
            .offer_packet()
            .map_err(|error| invalid(error.to_string()))?;
        self.next_instance = instance
            .checked_add(1)
            .ok_or_else(|| invalid("derby instance overflow"))?;
        self.service = Some(service);
        self.owner = Some(owner);
        self.previous_job = None;
        Ok(offer)
    }

    /// Handle the client's 39/5 Go request. A successful start also emits the recovered player
    /// status carrier with the type-11 active bit (R for the derby carrier; the bit itself is
    /// native `DERBY_ACTIVE_STATUS_BIT`).
    pub fn start(&mut self, inner: &[u8]) -> io::Result<Vec<Vec<u8>>> {
        let Some(service) = &self.service else {
            return Ok(Vec::new());
        };
        let mut staged = service.clone();
        let ack = staged
            .handle_minigame_packet(inner)
            .map_err(|error| invalid(error.to_string()))?;
        if ack.is_empty() {
            return Ok(Vec::new());
        }
        self.service = Some(staged);
        let owner = self
            .owner
            .ok_or_else(|| invalid("derby service has no authenticated owner"))?;
        Ok(vec![
            ack,
            crate::kart_gateway::player_status(owner.raw(), ACTIVE_STATUS_MASK),
        ])
    }

    /// Route one 34/* client record. Recipient filtering is explicit: this binding accepts one
    /// human, so an action addressed to any other session is an integration error, not silently
    /// dropped.
    pub fn handle(&mut self, inner: &[u8], now_ms: u64) -> io::Result<Vec<Vec<u8>>> {
        let Some(service) = &self.service else {
            return Ok(Vec::new());
        };
        let owner = self
            .owner
            .ok_or_else(|| invalid("derby service has no authenticated owner"))?;
        let mut staged = service.clone();
        let actions = staged
            .handle_client(owner, inner, now_ms)
            .map_err(|error| invalid(error.to_string()))?;
        let replies = packets_for_owner(&actions, owner)?;
        self.service = Some(staged);
        Ok(replies)
    }

    pub fn tick(&mut self, now_ms: u64) -> io::Result<Vec<Vec<u8>>> {
        let Some(service) = &self.service else {
            return Ok(Vec::new());
        };
        let owner = self
            .owner
            .ok_or_else(|| invalid("derby service has no authenticated owner"))?;
        let mut staged = service.clone();
        let actions = staged
            .tick(now_ms)
            .map_err(|error| invalid(error.to_string()))?;
        let replies = packets_for_owner(&actions, owner)?;
        self.service = Some(staged);
        Ok(replies)
    }

    /// Retire an offer or running match without a reward. Malformed, foreign and stale cancellation
    /// requests return no packets and leave the live service untouched.
    pub fn cancel(&mut self, packet: &[u8]) -> io::Result<Vec<Vec<u8>>> {
        let Some(service) = &self.service else {
            return Ok(Vec::new());
        };
        let owner = self
            .owner
            .ok_or_else(|| invalid("derby service has no authenticated owner"))?;
        let mut staged = service.clone();
        let Some(removal) = staged
            .cancel(owner, packet)
            .map_err(|error| invalid(error.to_string()))?
        else {
            return Ok(Vec::new());
        };
        self.service = Some(staged);
        Ok(vec![removal])
    }

    /// Accept the framework Close once the results screen has enabled it. The close retires the
    /// match and pays nothing: the completion reward is already settled at the results phase.
    pub fn close(&mut self, packet: &[u8]) -> io::Result<bool> {
        let Some(service) = &self.service else {
            return Ok(false);
        };
        let owner = self
            .owner
            .ok_or_else(|| invalid("derby service has no authenticated owner"))?;
        let mut staged = service.clone();
        let accepted = staged
            .handle_close(owner, packet)
            .map_err(|error| invalid(error.to_string()))?;
        if accepted {
            self.service = Some(staged);
        }
        Ok(accepted)
    }

    /// Take the finished match this session may be paid for: the live match is showing its results
    /// and the ledger has no row for its instance. Taking records the instance, so the same match
    /// never yields a second claim. `None` when the package grants nothing or the match is not
    /// finished.
    pub fn take_completion(&mut self) -> Option<i32> {
        if self.content.completion_reward_coins == 0 && self.content.completion_reward_xp == 0 {
            return None;
        }
        let instance = self
            .service
            .as_ref()
            .filter(|service| service.phase() == DerbyPhase::Results)
            .map(DerbyService::instance_id)?;
        if instance <= 0 || self.paid_matches.contains(&instance) {
            return None;
        }
        self.paid_matches.push(instance);
        if self.paid_matches.len() > MAX_PAID_MATCHES {
            self.paid_matches.remove(0);
        }
        Some(instance)
    }

    /// The one flat completion grant this package pays. It never carries items and never derives
    /// from a contact report, a score or a match duration; `display` is the ordinary reward toast.
    /// (R) The amounts are content, not recovered retail values.
    pub fn completion_grant(&self, guid: u64) -> RewardGrant {
        let job_xp = match self.content.completion_reward_xp {
            0 => BTreeMap::new(),
            xp => BTreeMap::from([(DEMO_DERBY_DRIVER_PROFILE, xp)]),
        };
        RewardGrant {
            coins: self.content.completion_reward_coins,
            items: Vec::new(),
            job_xp,
            display: RewardDisplay {
                source_guid: guid,
                player_guid: guid,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        }
    }

    /// The ledger to persist; `None` keeps a save with no paid match byte-identical.
    pub fn saved_state(&self) -> Option<DerbyGatewayState> {
        (!self.paid_matches.is_empty()).then(|| DerbyGatewayState {
            previous_job: None,
            last_paid_unix: None,
            paid_matches: self.paid_matches.clone(),
        })
    }

    /// Retire the live match and return the job to restore.
    pub fn exit(&mut self) -> Option<i32> {
        self.service = None;
        self.owner = None;
        self.previous_job.take()
    }
}

fn packets_for_owner(actions: &[DerbyAction], owner: ParticipantKey) -> io::Result<Vec<Vec<u8>>> {
    let mut replies = Vec::new();
    for action in actions {
        if action
            .recipients
            .iter()
            .any(|recipient| *recipient != owner)
        {
            return Err(invalid("derby action reached more than one human session"));
        }
        if action.recipients.is_empty() {
            return Err(invalid("derby action has no recipient"));
        }
        replies.push(action.packet.clone());
    }
    Ok(replies)
}

fn validate_content(content: &DerbyGatewayContent) -> io::Result<()> {
    if content.schema_version != 1
        || content.game_id <= 0
        || content.rules.score_target.is_none() && content.rules.time_limit_ms.is_none()
        || content.rules.score_target == Some(0)
        || content.rules.time_limit_ms == Some(0)
        || content.completion_reward_coins > MAX_COMPLETION_REWARD
        || content.completion_reward_xp > MAX_COMPLETION_REWARD
        || content
            .vehicle
            .start_transform_bits
            .iter()
            .any(|bits| !f32::from_bits(*bits).is_finite())
        || content.pickups.len() > 256
    {
        return Err(invalid("invalid derby content"));
    }
    DerbyService::new(
        FIRST_DERBY_INSTANCE as i32,
        content.game_id,
        ParticipantKey(1),
        content.vehicle.clone(),
        content.rules.clone(),
        content.pickups.clone(),
    )
    .map(|_| ())
    .map_err(|error: DerbyError| invalid(error.to_string()))
}
