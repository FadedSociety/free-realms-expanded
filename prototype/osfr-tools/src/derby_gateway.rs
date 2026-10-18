//! Optional ordinary-entry binding for the native demolition derby.
//!
//! The packet and match rules stay in `derby.rs`; this module binds one authenticated character to
//! one opt-in content package. The gate list is reconstructed policy (R), like the kart gates: an
//! NPC lists the derby only when named in `derby_gates.json`. Entry, phase acknowledgements, the
//! results hold and payout use the existing native type-11 lifecycle. No source-derived rule is
//! invented here; all tunable numbers come from `derby_entry.json`.

use crate::{
    character_store::CharacterState,
    derby::{
        DEMO_DERBY_LOBBY_NAME_ID, DerbyAction, DerbyCloseOutcome, DerbyError, DerbyPhase,
        DerbyPickup, DerbyRules, DerbyService, DerbyVehicleTemplate, ParticipantKey,
    },
    rewards::{RewardDisplay, RewardGrant, RewardState},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
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
/// (R) Ten minutes between paid runs unless the gate package tunes it.
pub const DEFAULT_REWARD_COOLDOWN_S: u64 = 600;

const MAX_CONTENT_BYTES: usize = 512 * 1024;
const MAX_GATES_BYTES: usize = 16 * 1024;
const ACTIVE_STATUS_MASK: u32 = (1 << crate::derby::DERBY_ACTIVE_STATUS_BIT) | 1;

/// One configured derby payout. The values are content, not save state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyReward {
    pub coins: u32,
    #[serde(default)]
    pub job_xp: std::collections::BTreeMap<i32, u32>,
    #[serde(default)]
    pub reward_kind: i32,
    #[serde(default)]
    pub icon_id: i32,
    #[serde(default)]
    pub name_id: i32,
}

/// One opt-in derby package. `vehicle`, `rules` and `pickups` are the service's existing typed
/// content; this wrapper adds only the game id and payout row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyGatewayContent {
    pub schema_version: u32,
    pub game_id: i32,
    pub vehicle: DerbyVehicleTemplate,
    pub rules: DerbyRules,
    #[serde(default)]
    pub pickups: Vec<DerbyPickup>,
    pub reward: DerbyReward,
}

/// (R) NPCs whose menu offers the derby. NPC existence is checked by `NpcInteraction`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyGates {
    pub schema_version: u32,
    #[serde(default = "default_menu_string_id")]
    pub menu_string_id: u32,
    pub npcs: Vec<u64>,
    /// (R) Seconds after a paid derby before a gate starts another; 0 is off.
    #[serde(default = "default_reward_cooldown_s")]
    pub reward_cooldown_s: u64,
}

/// The only derby data a save needs: the last paid-run epoch for the cross-session cooldown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyGatewayState {
    pub last_paid_unix: u64,
}

fn default_menu_string_id() -> u32 {
    DEFAULT_MENU_STRING_ID
}

fn default_reward_cooldown_s() -> u64 {
    DEFAULT_REWARD_COOLDOWN_S
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

/// Why an ordinary derby entry was refused. The caller reports the message and changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerbyEntryDenied {
    NotAGate,
    Unavailable,
    AlreadyPlaying,
    Busy,
    Cooldown,
}

impl DerbyEntryDenied {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotAGate => "The derby isn't available here.",
            Self::Unavailable => "The derby isn't open yet.",
            Self::AlreadyPlaying => "You are already in a derby.",
            Self::Busy => "Finish the active activity before entering the derby.",
            Self::Cooldown => "The arena is being reset. Come back for another derby soon.",
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
    last_paid_unix: Option<u64>,
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
        || gates.reward_cooldown_s > crate::kart_gateway::MAX_REWARD_COOLDOWN_S
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
    /// Load the content package and its optional NPC gate list. Absent content leaves the route
    /// disabled; malformed present content rejects the world load instead of degrading silently.
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        Self::load_with_state(directory, None)
    }

    pub fn load_with_state(
        directory: &Path,
        saved: Option<DerbyGatewayState>,
    ) -> io::Result<Option<Self>> {
        let path = directory.join("derby_entry.json");
        let Some(bytes) = read_regular_limited(&path, MAX_CONTENT_BYTES, "derby content")? else {
            return Ok(None);
        };
        let content: DerbyGatewayContent =
            serde_json::from_slice(&bytes).map_err(|error| invalid(error.to_string()))?;
        let mut gateway = Self::new(content, load_gates(directory)?)?;
        if let Some(saved) = saved {
            if saved.last_paid_unix == 0 {
                return Err(invalid("invalid saved derby cooldown"));
            }
            gateway.last_paid_unix = Some(saved.last_paid_unix);
        }
        Ok(Some(gateway))
    }

    /// Build a gateway from typed content. This is also the strict content validator used by tests
    /// and by interactive content reloads.
    pub fn new(content: DerbyGatewayContent, gates: Option<DerbyGates>) -> io::Result<Self> {
        validate_content(&content)?;
        if let Some(gates) = &gates {
            if gates.schema_version != 1
                || gates.menu_string_id == 0
                || gates.reward_cooldown_s > crate::kart_gateway::MAX_REWARD_COOLDOWN_S
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
            last_paid_unix: None,
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

    pub fn snapshot(&self) -> Option<DerbyGatewayState> {
        self.last_paid_unix
            .map(|last_paid_unix| DerbyGatewayState { last_paid_unix })
    }

    pub fn set_previous_job(&mut self, job: i32) {
        self.previous_job = Some(job);
    }

    /// Ordinary-entry admission. `busy` is computed by gameplay before any state is staged.
    pub fn admit(&self, npc_id: u64, busy: bool) -> Result<(), DerbyEntryDenied> {
        self.admit_at(npc_id, busy, now_unix())
    }

    pub fn admit_at(&self, npc_id: u64, busy: bool, now_unix: u64) -> Result<(), DerbyEntryDenied> {
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
        let cooldown = self
            .gates
            .as_ref()
            .map_or(0, |gates| gates.reward_cooldown_s);
        if self
            .last_paid_unix
            .is_some_and(|paid| now_unix < paid.saturating_add(cooldown))
        {
            return Err(DerbyEntryDenied::Cooldown);
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

    /// Apply the final Close payout to a staged reward state. The caller only commits the returned
    /// reward state when `accepted` is true, preserving the all-or-nothing inventory contract.
    pub fn close(&mut self, packet: &[u8], rewards: &RewardState) -> io::Result<DerbyCloseOutcome> {
        self.close_at(packet, rewards, now_unix())
    }

    pub fn close_at(
        &mut self,
        packet: &[u8],
        rewards: &RewardState,
        now_unix: u64,
    ) -> io::Result<DerbyCloseOutcome> {
        if now_unix == 0 {
            return Err(invalid("derby payout clock is unavailable"));
        }
        let Some(service) = &self.service else {
            return Ok(refused_close(DerbyPhase::Closed));
        };
        let owner = self
            .owner
            .ok_or_else(|| invalid("derby service has no authenticated owner"))?;
        let mut staged = service.clone();
        let grant = self.grant(owner);
        let outcome = staged
            .handle_close(owner, packet, rewards, &grant)
            .map_err(|error| invalid(error.to_string()))?;
        if outcome.accepted {
            self.service = Some(staged);
            self.last_paid_unix = Some(now_unix);
        }
        Ok(outcome)
    }

    /// Retire the live match and return the job to restore.
    pub fn exit(&mut self) -> Option<i32> {
        self.service = None;
        self.owner = None;
        self.previous_job.take()
    }

    fn grant(&self, owner: ParticipantKey) -> RewardGrant {
        RewardGrant {
            coins: self.content.reward.coins,
            items: Vec::new(),
            job_xp: self.content.reward.job_xp.clone(),
            display: RewardDisplay {
                source_guid: owner.raw(),
                player_guid: owner.raw(),
                reward_kind: self.content.reward.reward_kind,
                icon_id: self.content.reward.icon_id,
                name_id: self.content.reward.name_id,
                rows: Vec::new(),
            },
        }
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

fn refused_close(phase: DerbyPhase) -> DerbyCloseOutcome {
    DerbyCloseOutcome {
        accepted: false,
        phase,
        reward_state: None,
        reward_packets: Vec::new(),
    }
}

fn validate_content(content: &DerbyGatewayContent) -> io::Result<()> {
    if content.schema_version != 1
        || content.game_id <= 0
        || content.rules.score_target.is_none() && content.rules.time_limit_ms.is_none()
        || content.rules.score_target == Some(0)
        || content.rules.time_limit_ms == Some(0)
        || content
            .vehicle
            .start_transform_bits
            .iter()
            .any(|bits| !f32::from_bits(*bits).is_finite())
        || content.pickups.len() > 256
    {
        return Err(invalid("invalid derby content"));
    }
    if content.reward.coins > i32::MAX as u32
        || content.reward.job_xp.keys().any(|profile| *profile <= 0)
        || content
            .reward
            .job_xp
            .values()
            .any(|xp| *xp > i32::MAX as u32)
        || (content.reward.coins == 0 && content.reward.job_xp.values().all(|xp| *xp == 0))
    {
        return Err(invalid("invalid derby reward"));
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
