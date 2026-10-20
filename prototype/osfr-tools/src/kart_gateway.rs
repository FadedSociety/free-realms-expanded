//! Optional race entry binding. The reviewed playtest binding is tracked in
//! `data/content/kart_entry.json`; private captures remain in the lab.
//! Entry alone never awards anything; the race authority owns completion.
//!
//! Ordinary entry (W149, brief #3): a player starts a race by choosing the race option in the
//! 26/9 menu of a gate NPC, which the client answers with 26/10 (`npc_interaction`). The gate NPCs
//! come from the optional `kart_gates.json` beside `kart_entry.json`; that file is deliberately
//! outside the content hash, so editing the gate list never invalidates a saved race. 102/1 stays
//! a catalogue request and never starts a race. Every gate rule is reconstructed policy (R).
use crate::{
    character_store::CharacterState,
    kart_entry::{EntryGate, EntryOutcome, KartEntryHandler, RaceEntryContent, RaceEntryContext},
    zone_transition,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read},
    path::Path,
    sync::Arc,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KartGatewayContent {
    pub schema_version: u32,
    pub entry: RaceEntryContent,
    pub context: RaceEntryContext,
    /// Inner 41/114 frames, not tunnel envelopes.
    pub details_loading: Vec<u8>,
    pub details_ready: Vec<u8>,
    /// Proven GUID slot in the first (owned) vehicle's attribute block.
    pub vehicle_guid_offset: usize,
    #[serde(default)]
    pub race: Option<crate::kart_service::KartServiceContent>,
}

/// (R) Client locale 3052 is the exact text "Kart Race Lobby"; the menu label for the race option
/// until a better recovered label exists. Overridable in `kart_gates.json`.
pub const DEFAULT_MENU_STRING_ID: u32 = 3052;
/// (R) Upper bound on gate NPCs.
pub const MAX_GATES: usize = 16;
const MAX_GATES_FILE: usize = 16 * 1024;

/// (R) The NPCs whose menu offers the race. Ids are NPC content keys, like `npc_interaction`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KartGates {
    pub schema_version: u32,
    #[serde(default = "default_menu_string_id")]
    pub menu_string_id: u32,
    pub npcs: Vec<u64>,
    /// (R) Seconds after a paid race before a gate NPC starts another; 0 is off.
    /// Solo races pay on every finish, so without it racing can be farmed.
    #[serde(default)]
    pub reward_cooldown_s: u64,
}

/// (R) Sanity bound on the gate cooldown: one day.
pub const MAX_REWARD_COOLDOWN_S: u64 = 86_400;

fn default_menu_string_id() -> u32 {
    DEFAULT_MENU_STRING_ID
}

/// Why an ordinary entry was refused. Nothing moved; the player is told with [`Self::message`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryDenied {
    /// The chosen NPC is not a listed gate (a forged choice, or the list changed).
    NotAGate,
    /// The package has no race content, so a started race could never finish or pay.
    NoRace,
    AlreadyRacing,
    /// Another activity (crafting, gathering, a battle) is running.
    Busy,
    /// The gate cooldown after a paid race has not passed (R).
    Cooldown,
}

impl EntryDenied {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotAGate => "You can't start a race here.",
            Self::NoRace => "Racing isn't open yet.",
            Self::AlreadyRacing => "You are already in a race.",
            Self::Busy => "Finish the active activity before racing.",
            Self::Cooldown => "The track is being reset. Come back for another race soon.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActiveKartEntry {
    pub context: RaceEntryContext,
    pub return_location: CharacterState,
    /// The job before the race made the player a Kart Driver; restored on exit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_job: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KartGatewayState {
    pub version: u32,
    pub content_sha256: String,
    pub next_instance: u32,
    pub active: Option<ActiveKartEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub race: Option<crate::kart_service::ServiceSnapshot>,
    /// Unix seconds of the last paid race, for the gate cooldown (saved, so a
    /// relog does not reset it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_paid_unix: Option<u64>,
}

impl KartGatewayState {
    /// Whether a committed kart frame must write the character save. Three session-local fields
    /// are excluded: the weapon tick clock (re-stamped every racing tick), the observation flood
    /// budget and the last weapon pose (both move on accepted `33/6`/`33/17` bulk frames). They
    /// are persisted and restored, but they ride the next real change and the unconditional
    /// end-of-session save. Every other field is named in the destructure, so a field added later
    /// is compared by default.
    pub fn changed_for_save(self, after: &Self) -> bool {
        let Self {
            version,
            content_sha256,
            next_instance,
            active,
            race,
            last_paid_unix,
        } = self;
        version != after.version
            || content_sha256 != after.content_sha256
            || next_instance != after.next_instance
            || active != after.active
            || last_paid_unix != after.last_paid_unix
            || match (race, &after.race) {
                (Some(before), Some(after)) => !before.save_decision_eq(after),
                (None, None) => false,
                _ => true,
            }
    }
}

/// A paid finish and the stable race identity a quest may credit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KartRewardClaim {
    pub replies: Vec<Vec<u8>>,
    /// (R) The speedway id is the stable, content-authored race identity.
    pub minigame_id: Option<u32>,
}

#[derive(Clone)]
pub struct KartGateway {
    content: Arc<KartGatewayContent>,
    state: KartGatewayState,
    entry: Option<KartEntryHandler>,
    race: Option<crate::kart_service::KartService>,
    rewards: Arc<crate::kart_rewards::KartRewardRegistry>,
    gates: Option<Arc<KartGates>>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}
fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn guid(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?,
    ))
}

/// Same recovered 35/20 layout as the isolated piece056 codec, now usable
/// by race entry and return without registering its unrelated status ledger.
pub(crate) fn player_status(guid: u64, flags: u32) -> Vec<u8> {
    let mut bytes = vec![35, 0, 20, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.extend(flags.to_le_bytes());
    bytes
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Reads the optional gate list. Absent means no NPC offers the race.
fn load_gates(directory: &Path) -> io::Result<Option<KartGates>> {
    let path = directory.join("kart_gates.json");
    let metadata = match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
        Ok(m) => m,
    };
    if !metadata.is_file() {
        return Err(invalid("kart gates must be a regular file"));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((MAX_GATES_FILE + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_GATES_FILE {
        return Err(invalid("kart gates exceed 16 KiB"));
    }
    let gates: KartGates = serde_json::from_slice(&bytes).map_err(invalid)?;
    let mut seen = std::collections::BTreeSet::new();
    if gates.schema_version != 1
        || gates.menu_string_id == 0
        || gates.npcs.len() > MAX_GATES
        || gates.reward_cooldown_s > MAX_REWARD_COOLDOWN_S
        || !gates.npcs.iter().all(|npc| *npc != 0 && seen.insert(*npc))
    {
        return Err(invalid("invalid kart gates"));
    }
    Ok(Some(gates))
}

impl KartGateway {
    pub fn load(directory: &Path, saved: Option<KartGatewayState>) -> io::Result<Option<Self>> {
        let path = directory.join("kart_entry.json");
        let metadata = match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound && saved.is_none() => return Ok(None),
            Err(e) => return Err(e),
            Ok(m) => m,
        };
        const MAX: usize = 512 * 1024;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(invalid("kart entry content must be a regular file"));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take((MAX + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX {
            return Err(invalid("kart entry content exceeds 512 KiB"));
        }
        let content: KartGatewayContent = serde_json::from_slice(&bytes).map_err(invalid)?;
        if content.schema_version != 1 {
            return Err(invalid("unsupported kart entry content"));
        }
        if content.race.as_ref().is_some_and(|race| {
            !race
                .schedule
                .iter()
                .any(|step| step.frame == crate::kart_service::ScheduledFrame::Countdown(0))
        }) {
            return Err(invalid("kart race schedule never reaches countdown zero"));
        }
        KartEntryHandler::new(content.entry.clone()).map_err(invalid)?;
        // This starter binding owns one kart. AI racers need separate progress,
        // transforms and identities before they can be included here.
        if content.entry.entry_vehicles.len() != 1 {
            return Err(invalid(
                "kart starter entry requires exactly one owned vehicle",
            ));
        }
        zone_transition::encode(&content.context.scene)?;
        for (frame, state) in [(&content.details_loading, 2), (&content.details_ready, 4)] {
            if !(65..=4096).contains(&frame.len())
                || frame[..4] != [41, 0, 114, 0]
                || word(frame, 4) != content.context.encounter_id
                || word(frame, 8) != content.context.instance_id
                || word(frame, 12) != content.context.encounter_id
                || word(frame, 16) != state
                || guid(frame, 32) != Some(content.context.session)
            {
                return Err(invalid("kart encounter template proof differs"));
            }
            for index in 0..3 {
                if frame[51 + index * 4..55 + index * 4]
                    != content.context.scene.position[index].to_le_bytes()
                {
                    return Err(invalid("kart scene and encounter spawn differ"));
                }
            }
        }
        if guid(
            &content.entry.entry_vehicles[0].attribute_block,
            content.vehicle_guid_offset,
        ) != Some(content.context.session)
            || !content.entry.progress.iter().any(|p| {
                p.racer_guid == content.context.session && p.tail_0x29 == content.context.session
            })
        {
            return Err(invalid("kart vehicle or progress ownership proof differs"));
        }
        let hash = format!("{:x}", Sha256::digest(&bytes));
        let state = saved.unwrap_or(KartGatewayState {
            version: 1,
            content_sha256: hash.clone(),
            next_instance: 48_000_001,
            active: None,
            race: None,
            last_paid_unix: None,
        });
        if state.version != 1
            || state.content_sha256 != hash
            || !(48_000_001..=i32::MAX as u32).contains(&state.next_instance)
        {
            return Err(invalid("kart entry snapshot content differs"));
        }
        if let Some(active) = &state.active {
            let mut expected = content.context.clone();
            expected.session = active.return_location.guid;
            expected.instance_id = active.context.instance_id;
            expected.scene.id = active.context.instance_id as i32;
            if active.context.instance_id.checked_add(1) != Some(state.next_instance)
                || active.context.session != active.return_location.guid
                || active.context != expected
                || active.return_location.guid == 0
                || active
                    .return_location
                    .position
                    .iter()
                    .chain(&active.return_location.rotation)
                    .any(|v| !v.is_finite())
            {
                return Err(invalid("invalid active kart entry snapshot"));
            }
        }
        let mut race = match (&content.race, &state.race) {
            (Some(config), Some(saved)) => {
                if saved.reward_taken != saved.session.reward_claimed {
                    return Err(invalid("kart payout markers differ"));
                }
                if saved.context.player_guid == 0
                    || saved.context.session != saved.context.player_guid
                    || saved.context.encounter_id != content.context.encounter_id
                    || saved.context.instance_id.checked_add(1) != Some(state.next_instance)
                    || state.active.as_ref().is_some_and(|a| {
                        a.context.session != saved.context.session
                            || a.context.instance_id != saved.context.instance_id
                    })
                {
                    return Err(invalid("kart race snapshot ownership differs"));
                }
                Some(
                    crate::kart_service::KartService::restore(config.clone(), saved.clone())
                        .map_err(invalid)?,
                )
            }
            (Some(config), None) => {
                Some(crate::kart_service::KartService::new(config.clone()).map_err(invalid)?)
            }
            (None, Some(_)) => return Err(invalid("kart race snapshot requires race content")),
            (None, None) => None,
        };
        if let Some(race) = &mut race {
            race.set_gate(EntryGate::Enabled);
        }
        // Content is the sole source of the registered price. Save state may
        // not override it; its SHA and service layout are checked above.
        let rows = content
            .race
            .as_ref()
            .and_then(|r| {
                r.reward.map(|price| {
                    crate::kart_rewards::RewardRegistration::reconstructed(
                        r.speedway,
                        price.coins,
                        price.xp,
                    )
                })
            })
            .into_iter()
            .collect();
        let rewards = Arc::new(
            crate::kart_rewards::KartRewardRegistry::new(rows)
                .map_err(|e| invalid(format!("invalid kart reward registration: {e:?}")))?,
        );
        let mut result = Self {
            content: Arc::new(content),
            state,
            entry: None,
            race,
            rewards,
            gates: load_gates(directory)?.map(Arc::new),
        };
        if result.state.active.is_some() {
            result.rebuild_entry()?;
        }
        Ok(Some(result))
    }
    pub fn state(&self) -> &KartGatewayState {
        &self.state
    }
    /// NPC content keys whose menu may list the race. Empty unless the package carries race
    /// content: an NPC must never offer a race that cannot finish.
    pub fn gate_npcs(&self) -> &[u64] {
        match (&self.gates, &self.content.race) {
            (Some(gates), Some(_)) => &gates.npcs,
            _ => &[],
        }
    }
    /// Locale id of the race option's menu label.
    pub fn menu_string_id(&self) -> u32 {
        self.gates
            .as_ref()
            .map_or(DEFAULT_MENU_STRING_ID, |g| g.menu_string_id)
    }
    /// Ordinary-entry admission for a 26/10 race choice at `npc_id` (already range-checked by the
    /// NPC service). `busy` is true while another activity is running. Mutates nothing; on `Ok`
    /// the caller runs the same job switch + [`Self::begin`] + [`Self::set_previous_job`] as the
    /// developer command.
    pub fn admit(&self, npc_id: u64, busy: bool) -> Result<(), EntryDenied> {
        self.admit_at(npc_id, busy, now_unix())
    }

    /// [`Self::admit`] at a given time (unix seconds), for tests.
    pub fn admit_at(&self, npc_id: u64, busy: bool, now_unix: u64) -> Result<(), EntryDenied> {
        if !self
            .gates
            .as_ref()
            .is_some_and(|g| g.npcs.contains(&npc_id))
        {
            return Err(EntryDenied::NotAGate);
        }
        if self.content.race.is_none() {
            return Err(EntryDenied::NoRace);
        }
        if self.active() {
            return Err(EntryDenied::AlreadyRacing);
        }
        if busy {
            return Err(EntryDenied::Busy);
        }
        let cooldown = self.gates.as_ref().map_or(0, |g| g.reward_cooldown_s);
        if self
            .state
            .last_paid_unix
            .is_some_and(|paid| now_unix < paid.saturating_add(cooldown))
        {
            return Err(EntryDenied::Cooldown);
        }
        Ok(())
    }
    fn sync_race(&mut self) {
        self.state.race = self.race.as_ref().and_then(|r| r.snapshot());
    }
    pub fn active(&self) -> bool {
        self.state.active.is_some()
    }
    /// ORIGINAL ScoreScreen.ExitScreen -> Ui.LeaveMiniGameOrEncounter:
    /// opcode39, byte subtype7, encounter id, two -1 words (15 inner bytes).
    pub fn accepts_close(&self, inner: &[u8]) -> io::Result<bool> {
        if !inner.starts_with(&[39, 0, 7]) {
            return Ok(false);
        }
        if inner.len() != 15 {
            return Err(invalid("invalid kart close width"));
        }
        Ok(self
            .state
            .active
            .as_ref()
            .is_some_and(|a| word(inner, 3) == a.context.encounter_id && inner[7..15] == [255; 8]))
    }
    pub fn scene(&self) -> Option<&zone_transition::ZoneTransition> {
        self.state.active.as_ref().map(|a| &a.context.scene)
    }
    pub fn supports_location(&self, character: &CharacterState) -> bool {
        self.state.active.as_ref().is_some_and(|a| {
            character.guid == a.return_location.guid && character.zone_id == a.context.instance_id
        })
    }
    fn rebuild_entry(&mut self) -> io::Result<()> {
        let active = self
            .state
            .active
            .as_ref()
            .ok_or_else(|| invalid("no active kart entry"))?;
        let mut content = self.content.entry.clone();
        // The captured kart transform is a fixed track spawn, not the avatar's
        // prior world location. Its translation is also present in 33/11.
        // 33/5 has a slot, not a GUID; register words are rebound by on_register.
        let old = self.content.context.session;
        let owned = active.context.session;
        let start = self.content.vehicle_guid_offset;
        content.entry_vehicles[0].attribute_block[start..start + 8]
            .copy_from_slice(&owned.to_le_bytes());
        for progress in &mut content.progress {
            if progress.racer_guid == old {
                progress.racer_guid = owned;
            }
            if progress.tail_0x29 == old {
                progress.tail_0x29 = owned;
            }
        }
        let mut entry = KartEntryHandler::new(content).map_err(invalid)?;
        entry.set_gate(EntryGate::Enabled);
        entry.open(active.context.clone()).map_err(invalid)?;
        self.entry = Some(entry);
        Ok(())
    }
    fn details(&self, loading: bool) -> io::Result<Vec<u8>> {
        let active = self
            .state
            .active
            .as_ref()
            .ok_or_else(|| invalid("no active kart entry"))?;
        let mut bytes = if loading {
            self.content.details_loading.clone()
        } else {
            self.content.details_ready.clone()
        };
        bytes[8..12].copy_from_slice(&active.context.instance_id.to_le_bytes());
        bytes[32..40].copy_from_slice(&active.context.session.to_le_bytes());
        Ok(bytes)
    }
    pub fn begin(
        &mut self,
        character: &CharacterState,
    ) -> io::Result<(Vec<Vec<u8>>, zone_transition::ZoneTransition)> {
        if self.active() {
            return Err(invalid("kart entry already active"));
        }
        let next = self
            .state
            .next_instance
            .checked_add(1)
            .filter(|n| *n <= i32::MAX as u32)
            .ok_or_else(|| invalid("kart instance overflow"))?;
        let mut staged = self.clone();
        let mut context = self.content.context.clone();
        context.session = character.guid;
        context.instance_id = staged.state.next_instance;
        context.scene.id = context.instance_id as i32;
        staged.state.active = Some(ActiveKartEntry {
            context,
            return_location: character.clone(),
            previous_job: None,
        });
        staged.state.next_instance = next;
        staged.rebuild_entry()?;
        if let Some(config) = &staged.content.race {
            let mut race =
                crate::kart_service::KartService::new(config.clone()).map_err(invalid)?;
            race.set_gate(EntryGate::Enabled);
            staged.race = Some(race);
            staged.sync_race();
        }
        let replies = vec![staged.details(true)?];
        let scene = staged.scene().unwrap().clone();
        *self = staged;
        Ok((replies, scene))
    }
    pub fn initialize(&self) -> io::Result<Vec<Vec<u8>>> {
        let context = &self
            .state
            .active
            .as_ref()
            .ok_or_else(|| invalid("no active kart entry"))?
            .context;
        let mut enter = vec![41, 0, 2, 0];
        enter.extend(context.encounter_id.to_le_bytes());
        enter.extend(context.instance_id.to_le_bytes());
        enter.extend([0; 9]);
        // ORIGINAL frame15451 (also15496): PlayerUpdate/20 sets the local
        // actor mask to0x401. Bit10 activates VehicleRaceProcessor(1), whose
        // start branch sends33/9; zoning's reset(0) only tears it down.
        let status = player_status(context.session, 0x401);
        let details = self.details(false)?;
        // ORIGINAL frames15449/15450 carry two launch responses. The second
        // clears MiniGameInfo.ActivityId (the u32 fifteen bytes from the end).
        // It reaches the readiness path after the first creates MiniGameState.
        let mut ready = details.clone();
        let activity = ready.len() - 15;
        ready[activity..activity + 4].fill(0);
        Ok(vec![details, enter, ready, status])
    }
    /// ORIGINAL frame21639 follows ClientFinishedLoading10: the same
    /// encounter-enter packet now carries the local player's GUID.
    pub fn loaded(&self) -> io::Result<Vec<Vec<u8>>> {
        let context = &self
            .state
            .active
            .as_ref()
            .ok_or_else(|| invalid("no active kart entry"))?
            .context;
        let mut enter = vec![41, 0, 2, 0];
        enter.extend(context.encounter_id.to_le_bytes());
        enter.extend(context.instance_id.to_le_bytes());
        enter.extend(context.session.to_le_bytes());
        enter.push(0);
        // EncounterZoneIsReady (41/107) is a bare encounter header. Native
        // case0x6b calls HandlerMiniGameStart:setReady after decoding it.
        // The local direct-entry command also needs this readiness signal.
        let mut ready = vec![41, 0, 107, 0];
        ready.extend(context.encounter_id.to_le_bytes());
        ready.extend(context.instance_id.to_le_bytes());
        Ok(vec![ready, enter])
    }
    pub fn handle_entry(&mut self, inner: &[u8]) -> io::Result<Vec<Vec<u8>>> {
        let Some(entry) = &mut self.entry else {
            return Ok(Vec::new());
        };
        match entry.handle_inner(inner).map_err(invalid)? {
            EntryOutcome::Replies(replies) => Ok(replies),
            EntryOutcome::Refused(_) => Ok(Vec::new()),
        }
    }
    pub fn handle(&mut self, inner: &[u8], now_ms: u64) -> io::Result<Vec<Vec<u8>>> {
        // ORIGINAL native Go emits EncounterStart (41/108), the encounter
        // header followed by the owned player GUID. Registration happens while
        // assets are still loading; it must not start the countdown clock.
        if inner.starts_with(&[41, 0, 108, 0]) {
            if inner.len() != 20 {
                return Err(invalid("invalid kart start width"));
            }
            let Some(active) = &self.state.active else {
                return Ok(Vec::new());
            };
            let context = &active.context;
            if word(inner, 4) != context.encounter_id
                || word(inner, 8) != context.instance_id
                || guid(inner, 12) != Some(context.session)
                || !self.entry.as_ref().is_some_and(|e| {
                    matches!(
                        e.phase(),
                        crate::kart_entry::EntryPhase::Started
                            | crate::kart_entry::EntryPhase::Joined
                    )
                })
            {
                return Ok(Vec::new());
            }
            if let Some(race) = &mut self.race {
                if race.phase() != crate::kart_service::RacePhase::Preparing {
                    return Ok(Vec::new());
                }
                race.start_countdown(now_ms).map_err(invalid)?;
            }
            let id = context.encounter_id;
            self.sync_race();
            // Native op39/u8 subtype17 calls MiniGame:OnGameStarted using
            // StateId. Group/Game are unused by that arm; this private race
            // has no group or catalogue game id.
            let mut started = vec![39, 0, 17];
            for word in [id as i32, -1, -1] {
                started.extend(word.to_le_bytes());
            }
            return Ok(vec![started]);
        }
        let (subtype, _) = crate::kart_entry::decode_inner(inner).map_err(invalid)?;
        let replies = if matches!(subtype, 9 | 2) {
            let replies = self.handle_entry(inner)?;
            if subtype == 9 && !replies.is_empty() {
                if let Some(race) = &mut self.race {
                    let context = &self.state.active.as_ref().unwrap().context;
                    race.prepare(
                        crate::kart_service::RaceContext {
                            session: context.session,
                            player_guid: context.session,
                            instance_id: context.instance_id,
                            encounter_id: context.encounter_id,
                        },
                        now_ms,
                    )
                    .map_err(invalid)?;
                }
            }
            replies
        } else if let Some(race) = &mut self.race {
            race.handle_inner(inner, now_ms)
                .map_err(invalid)?
                .replies()
                .to_vec()
        } else {
            Vec::new()
        };
        self.sync_race();
        Ok(replies)
    }
    pub fn tick(&mut self, now_ms: u64) -> io::Result<Vec<Vec<u8>>> {
        let replies = if let Some(race) = &mut self.race {
            race.tick(now_ms).map_err(invalid)?.replies().to_vec()
        } else {
            Vec::new()
        };
        self.sync_race();
        Ok(replies)
    }
    pub fn claim_reward(
        &mut self,
        inventory: &mut crate::inventory::Inventory,
        profiles: &mut crate::profiles::Profiles,
    ) -> io::Result<Vec<Vec<u8>>> {
        Ok(self.claim_reward_with_event(inventory, profiles)?.replies)
    }
    /// Claim a finished race once and report the race id for quest events.
    pub fn claim_reward_with_event(
        &mut self,
        inventory: &mut crate::inventory::Inventory,
        profiles: &mut crate::profiles::Profiles,
    ) -> io::Result<KartRewardClaim> {
        if !self.race.as_ref().is_some_and(|race| race.finish_flag()) {
            return Ok(KartRewardClaim::default());
        }
        // Claiming consumes the finish offer. Stage that flag with both banks so
        // an overflow or refused price leaves a retryable, unpaid finish.
        let mut staged = self.clone();
        let mut staged_inventory = inventory.clone();
        let mut staged_profiles = profiles.clone();
        let race = staged.race.as_mut().unwrap();
        if !race.reward_priced() {
            return Ok(KartRewardClaim::default());
        }
        let guid = race.player_guid().unwrap();
        let finished = race.take_finished_session().unwrap();
        let payout = match crate::kart_rewards::claim_race_reward(
            &staged.rewards,
            finished,
            &mut staged_inventory,
            &mut staged_profiles,
            guid,
            guid,
        ) {
            Ok(payout) => payout,
            // A bank/XP limit is a refused payout, not a transport failure.
            // Keep the unpaid finish intact and release its progress frame.
            Err(_) => return Ok(KartRewardClaim::default()),
        };
        staged.state.last_paid_unix = Some(now_unix());
        staged.sync_race();
        *self = staged;
        *inventory = staged_inventory;
        *profiles = staged_profiles;
        Ok(KartRewardClaim {
            replies: payout.replies,
            minigame_id: Some(u32::from(payout.speedway)),
        })
    }
    pub fn race_ready(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let entry = self
            .entry
            .as_mut()
            .ok_or_else(|| invalid("no active kart entry"))?;
        match entry.on_race_ready().map_err(invalid)? {
            EntryOutcome::Replies(replies) => Ok(replies),
            EntryOutcome::Refused(_) => Ok(Vec::new()),
        }
    }
    pub fn set_previous_job(&mut self, job: i32) {
        if let Some(active) = &mut self.state.active {
            active.previous_job = Some(job);
        }
    }
    pub fn previous_job(&self) -> Option<i32> {
        self.state.active.as_ref().and_then(|a| a.previous_job)
    }
    pub fn exit(&mut self) -> Option<CharacterState> {
        if let Some(race) = &mut self.race {
            race.abort();
        }
        self.sync_race();
        self.entry = None;
        self.state.active.take().map(|a| a.return_location)
    }
}
