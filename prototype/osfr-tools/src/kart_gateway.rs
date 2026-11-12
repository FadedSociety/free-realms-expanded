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
    kart_rows::KartRows,
    kart_service::KartServiceContent,
    kart_tracks::{self, TrackPackage},
    le,
    vehicle_parts::{self, GarageContent, Loadouts, Looks, Vehicles},
    zone_transition,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read},
    path::Path,
    sync::Arc,
};

/// The per-track packages, keyed by scene (`kart_tracks/<scene>.json`).
type Tracks = BTreeMap<String, Arc<TrackPackage>>;

/// Bytes of a start pose in the captured bodies: a 3 x 3 row-major basis then the translation.
const POSE_BYTES: usize = 48;
/// Where the captured 41/114 details carry the MiniGameData row id, counted from the end (R: the
/// word before the Games window row 62 and its `62/14` pair; `initialize` clears a later word).
const DETAILS_ROW_FROM_END: usize = 43;

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
    /// L097: the client `MiniGameData.txt` row the shipped race is (28 = Wildwood Kart Racing for
    /// the `sg_kt_track` scene); the Games window row with that ContentId starts the race too.
    #[serde(default)]
    pub minigame_id: Option<u32>,
    /// P-kart-racing: the speedway row each gate starts (NPC content key -> MiniGameData row); a
    /// gate absent here starts `minigame_id`. A gate whose row has no track package refuses with
    /// "Racing isn't open yet" instead of starting another speedway's race (sheet B5).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub gate_rows: BTreeMap<u64, u32>,
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
    /// L930: the MiniGameData row this race runs as; a finish credits it. Absent in saves that
    /// predate rows (the gates' default row then).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<u32>,
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
    /// The character's karts and derby cars, with what is fitted and which is chosen.
    #[serde(default, skip_serializing_if = "crate::vehicle_parts::Loadouts::is_empty")]
    pub loadouts: crate::vehicle_parts::Loadouts,
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
            loadouts,
        } = self;
        version != after.version
            || content_sha256 != after.content_sha256
            || next_instance != after.next_instance
            || active != after.active
            || last_paid_unix != after.last_paid_unix
            || loadouts != after.loadouts
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
    /// L930: the MiniGameData row the race ran as (`kart_rows.json`), not the speedway id the
    /// reward registry is keyed by.
    pub minigame_id: Option<u32>,
    /// (R) T787: the finished race's own elapsed time.
    pub elapsed_seconds: Option<u32>,
    /// C6: the place among the AI field (1 = first); `None` when the race ran without a field.
    pub placement: Option<u8>,
    /// C6: whether the racer beat the row's named rival; `None` on a row with no rival.
    pub beat_rival: Option<bool>,
}

impl KartRewardClaim {
    /// The settled facts the quest engine matches a `require` against: the elapsed time, the place
    /// and, on a rival row, whether the rival was beaten (`won`).
    pub fn result(&self) -> crate::quest_engine::MinigameResult {
        crate::quest_engine::MinigameResult {
            won: self.beat_rival.unwrap_or(true),
            placement: self.placement,
            ..crate::quest_engine::MinigameResult::finished_at(self.elapsed_seconds)
        }
    }
}

#[derive(Clone)]
pub struct KartGateway {
    content: Arc<KartGatewayContent>,
    state: KartGatewayState,
    entry: Option<KartEntryHandler>,
    race: Option<crate::kart_service::KartService>,
    rewards: Arc<crate::kart_rewards::KartRewardRegistry>,
    gates: Option<Arc<KartGates>>,
    /// L930: the rows the race serves; `None` means the one default row.
    rows: Option<Arc<crate::kart_rows::KartRows>>,
    /// P-kart-racing: the race packages of the other track scenes.
    tracks: Arc<Tracks>,
    /// Where the captured 41/114 details carry the MiniGameInfo texts [name, icon, description,
    /// difficulty], so a quest row's offer shows its own title (R: found by the Wildwood row's
    /// texts; `None` leaves the frames as captured).
    texts_offset: Option<usize>,
    /// The row the next [`Self::begin`] runs, set by [`Self::choose_row`]; never saved.
    next_row: Option<u32>,
    /// `None` when the content has no vehicle table: logins keep the template's list.
    vehicles: Option<Arc<Vehicles>>,
    /// `None` without `vehicle_looks.json`: every car keeps the race's or derby's default look.
    looks: Option<Arc<Looks>>,
    /// The garage's own values; `None` without `kart_garage.json` (no doors, no first cars).
    garage_content: Option<Arc<GarageContent>>,
    /// C6: the AI racers' tunables; `None` without `kart_ai.json` (a solo time trial as before).
    ai: Option<Arc<crate::kart_ai::KartAi>>,
}

/// The grid pose (3 x 3 basis then translation) opens the captured 33/5 body (`client_data`,
/// `kart-motion-layout` report) and follows the owner guid in the kart's 33/11 block, so it moves
/// with the definition when another look is applied.
const CLIENT_DATA_POSE: usize = 0;
const VEHICLE_POSE_AFTER_GUID: usize = 8;

/// True when `bytes` carry a pose at `offset` whose translation is `start`.
fn pose_at(bytes: &[u8], offset: usize, start: [f32; 3]) -> bool {
    bytes.len() >= offset + POSE_BYTES
        && (0..3).all(|axis| le::f32(bytes, offset + 36 + axis * 4) == Some(start[axis]))
}

/// Writes a grid pose (basis rows then translation) at `offset`.
fn write_pose(bytes: &mut [u8], offset: usize, basis: [f32; 9], translation: [f32; 3]) {
    for (index, value) in basis.iter().chain(translation.iter()).enumerate() {
        let at = offset + index * 4;
        if let Some(slot) = bytes.get_mut(at..at + 4) {
            slot.copy_from_slice(&value.to_le_bytes());
        }
    }
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

/// Same recovered 35/20 layout as the isolated piece056 codec, now usable
/// by race entry and return without registering its unrelated status ledger.
pub(crate) fn player_status(guid: u64, flags: u32) -> Vec<u8> {
    let mut bytes = vec![35, 0, 20, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.extend(flags.to_le_bytes());
    bytes
}

/// The race package row `row` runs: the table's rules over the package of the row's track (the
/// Wildwood `base` or a `kart_tracks` package), the base itself for a save that predates rows or a
/// package with no table.
fn race_for(
    rows: Option<&KartRows>,
    tracks: &Tracks,
    base: &KartServiceContent,
    row: Option<u32>,
) -> io::Result<KartServiceContent> {
    match (rows, row) {
        (Some(rows), Some(id)) => {
            let track = rows.track(id).and_then(|scene| tracks.get(scene));
            rows.race(id, track.map_or(base, |track| &track.race))
        }
        _ => Ok(base.clone()),
    }
}

/// The track package row `row` races on, when it is not the Wildwood package.
fn track_for<'a>(
    rows: Option<&KartRows>,
    tracks: &'a Tracks,
    row: Option<u32>,
) -> Option<&'a Arc<TrackPackage>> {
    tracks.get(rows?.track(row?)?)
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
    let mut seen = BTreeSet::new();
    if gates.schema_version != 1
        || gates.menu_string_id == 0
        || gates.npcs.len() > MAX_GATES
        || gates.reward_cooldown_s > MAX_REWARD_COOLDOWN_S
        || !gates.npcs.iter().all(|npc| *npc != 0 && seen.insert(*npc))
        || !gates
            .gate_rows
            .iter()
            .all(|(npc, row)| seen.contains(npc) && *row != 0)
    {
        return Err(invalid("invalid kart gates"));
    }
    Ok(Some(gates))
}

/// What the garage borrows from a race: its captured offers, its kart and its scene.
#[derive(Clone, Debug, PartialEq)]
pub struct GarageParts {
    pub encounter_id: u32,
    pub loading: Vec<u8>,
    pub ready: Vec<u8>,
    pub vehicle: Vec<u8>,
    pub scene: zone_transition::ZoneTransition,
}

impl KartGateway {
    /// The race's parts for the garage; `load` keeps exactly one kart. The vehicle stops at
    /// `vehicle_guid_offset`: what follows is 33/11's (owner, positions, flags), not the definition,
    /// and the client drops a 139/2 that does not parse to its last byte.
    pub fn garage_parts(&self) -> Option<GarageParts> {
        let mut vehicle = self.content.entry.entry_vehicles.first()?.clone();
        vehicle.attribute_block.truncate(self.content.vehicle_guid_offset);
        Some(GarageParts {
            encounter_id: self.content.context.encounter_id,
            loading: self.content.details_loading.clone(),
            ready: self.content.details_ready.clone(),
            vehicle: vehicle.encode(),
            scene: self.content.context.scene.clone(),
        })
    }

    /// The vehicle table and this character's loadouts.
    pub fn garage(&mut self) -> Option<(Arc<Vehicles>, &mut Loadouts)> {
        Some((self.vehicles.clone()?, &mut self.state.loadouts))
    }

    pub fn garage_content(&self) -> Option<&GarageContent> {
        self.garage_content.as_deref()
    }

    /// The vehicle table, when the content has one.
    pub fn vehicles(&self) -> Option<&Vehicles> {
        self.vehicles.as_deref()
    }

    /// 139/5 from a race's offer: the open entry takes the newly chosen kart while nobody has
    /// registered, as 33/11 goes out at registration. A pick after that waits for the next race.
    pub fn refresh_look(&mut self) -> io::Result<()> {
        let waiting = (self.entry.as_ref())
            .is_some_and(|entry| matches!(entry.phase(), crate::kart_entry::EntryPhase::Waiting));
        if waiting && self.state.active.is_some() {
            self.rebuild_entry()?;
        }
        Ok(())
    }

    /// The open entry's content, after this visit's rewrites (`None` between races).
    pub fn entry_content(&self) -> Option<&RaceEntryContent> {
        Some(self.entry.as_ref()?.content())
    }

    /// The definition of the chosen car of a type (1 kart, 2 derby), when `vehicle_looks.json`
    /// knows its look; `None` keeps the race's or derby's default car.
    pub fn chosen_look(&self, kind: u32) -> Option<Vec<u8>> {
        let item = self.state.loadouts.chosen(kind)?.item;
        self.looks.as_ref()?.definition(item).map(<[u8]>::to_vec)
    }

    /// The list and chosen ids the self packet carries.
    pub fn login_vehicles(&self) -> Option<(Vec<u8>, [u32; 2])> {
        let table = self.vehicles.as_ref()?;
        Some((self.state.loadouts.encode(table), self.state.loadouts.active))
    }

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
        let mut content: KartGatewayContent = serde_json::from_slice(&bytes).map_err(invalid)?;
        if content.schema_version != 1 {
            return Err(invalid("unsupported kart entry content"));
        }
        // L927: the Kart Driver's `activity_xp` per race (`job_xp_curve.json`) replaces the price
        // row's XP; the race and the reward registry below both read the one price.
        let rate = crate::profiles::load_curves(directory)?
            .get(&crate::kart_rewards::KART_DRIVER_JOB_ID)
            .and_then(crate::profiles::RankCurve::activity_xp);
        if let Some(price) = content.race.as_mut().and_then(|race| race.reward.as_mut())
            && let Some(xp) = rate
        {
            price.xp = xp;
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
                || le::u32(frame, 4) != Some(content.context.encounter_id)
                || le::u32(frame, 8) != Some(content.context.instance_id)
                || le::u32(frame, 12) != Some(content.context.encounter_id)
                || le::u32(frame, 16) != Some(state)
                || le::u64(frame, 32) != Some(content.context.session)
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
        if le::u64(
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
            loadouts: Loadouts::default(),
        });
        if state.version != 1
            || state.content_sha256 != hash
            || !(48_000_001..=i32::MAX as u32).contains(&state.next_instance)
        {
            return Err(invalid("kart entry snapshot content differs"));
        }
        let gates = load_gates(directory)?.map(Arc::new);
        // Every track pays the one registered price (the base package's, with the Kart Driver's
        // `activity_xp` already applied above) and reports the base speedway to the registry.
        let mut tracks = kart_tracks::load(directory)?;
        if let Some(base) = &content.race {
            for track in tracks.values_mut() {
                track.race.reward = base.reward;
                track.race.speedway = base.speedway;
            }
        }
        let tracks: Arc<Tracks> = Arc::new(
            tracks
                .into_iter()
                .map(|(scene, track)| (scene, Arc::new(track)))
                .collect(),
        );
        let scenes: BTreeSet<String> = std::iter::once(content.context.scene.scene.clone())
            .chain(tracks.keys().cloned())
            .collect();
        let rows = KartRows::load(directory, scenes)?.map(Arc::new);
        // A saved race on a track whose package is gone (content moved on) keeps its entry so
        // the login abandon returns the player (`Gameplay::abandon_kart_for_reconnect`), but its
        // race is dropped: nothing can judge or pay it.
        let orphaned = state.active.as_ref().is_some_and(|active| {
            active.context.scene.scene != content.context.scene.scene
                && track_for(rows.as_deref(), &tracks, active.row).is_none()
        });
        if let Some(active) = &state.active {
            let mut expected = content.context.clone();
            if let Some(track) = track_for(rows.as_deref(), &tracks, active.row) {
                expected.scene = track.scene.clone();
            } else if orphaned {
                zone_transition::encode(&active.context.scene)?;
                expected.scene = active.context.scene.clone();
            }
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
        if let (Some(rows), Some(base)) = (&rows, &content.race) {
            // Every row that can run must build a valid race, and the gates' row must be one.
            for id in rows.playable_ids() {
                let race = race_for(Some(rows), &tracks, base, Some(id))?;
                crate::kart_service::KartService::new(race).map_err(invalid)?;
            }
            if let Some(id) = gates.as_ref().and_then(|g| g.minigame_id)
                && !rows.is_playable(id)
            {
                return Err(invalid("the kart gates' row cannot run"));
            }
        }
        // Races on another track move the kart to that track's grid, so the captured bodies must
        // carry the Wildwood grid where `rebuild_entry` writes: the 33/5 body's head and the 33/11
        // block right after the owner guid. The MiniGameInfo texts of the captured details are
        // found by the default row's texts; without them the frames stay as captured.
        if !tracks.is_empty()
            && !content.race.as_ref().is_some_and(|race| {
                pose_at(&content.entry.client_data, CLIENT_DATA_POSE, race.start_position)
                    && pose_at(
                        &content.entry.entry_vehicles[0].attribute_block,
                        content.vehicle_guid_offset + VEHICLE_POSE_AFTER_GUID,
                        race.start_position,
                    )
            })
        {
            return Err(invalid("kart track packages need the package's grid pose"));
        }
        let texts_offset = gates
            .as_ref()
            .and_then(|g| g.minigame_id)
            .and_then(|id| rows.as_ref()?.texts(id))
            .and_then(|texts| {
                let frame = &content.details_loading;
                (0..frame.len().saturating_sub(16)).find(|&at| {
                    le::u32(frame, at) == Some(texts.name_id)
                        && le::u32(frame, at + 4) == Some(texts.icon_id)
                        && le::u32(frame, at + 8) == Some(texts.description_id)
                        && le::u32(frame, at + 12) == Some(texts.difficulty)
                })
            });
        let mut state = state;
        if orphaned {
            state.race = None;
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
                let row = state.active.as_ref().and_then(|a| a.row);
                Some(
                    crate::kart_service::KartService::restore(
                        race_for(rows.as_deref(), &tracks, config, row)?,
                        saved.clone(),
                    )
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
        let prices = content
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
            crate::kart_rewards::KartRewardRegistry::new(prices)
                .map_err(|e| invalid(format!("invalid kart reward registration: {e:?}")))?,
        );
        let mut result = Self {
            content: Arc::new(content),
            state,
            entry: None,
            race,
            rewards,
            gates,
            rows,
            tracks,
            texts_offset,
            next_row: None,
            vehicles: Vehicles::load(directory)?,
            looks: Looks::load(directory)?,
            garage_content: GarageContent::load(directory)?,
            ai: crate::kart_ai::KartAi::load(directory)?.map(Arc::new),
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
    /// L097: the MiniGameData row the Games window lists for this race, when the race runs.
    pub fn minigame_id(&self) -> Option<u32> {
        self.content.race.as_ref()?;
        self.gates.as_ref()?.minigame_id
    }
    /// L930: every MiniGameData row a race can run as, sorted: the default row and the table's
    /// playable rows. Empty without race content. A row on a track we have no content for is
    /// absent, so its Games window row stays closed.
    pub fn minigame_ids(&self) -> Vec<u32> {
        if self.content.race.is_none() {
            return Vec::new();
        }
        let mut ids: Vec<u32> = self.minigame_id().into_iter().collect();
        ids.extend(self.rows.iter().flat_map(|rows| rows.playable_ids()));
        ids.sort_unstable();
        ids.dedup();
        ids
    }
    /// L930: picks the row the next [`Self::begin`] runs: the first of `wanted` (the player's open
    /// quest objectives) that can run, else `entered` (the Games window row played), else the
    /// default row. Call it just before `begin`, on the same staged copy.
    pub fn choose_row(&mut self, wanted: &[u32], entered: Option<u32>) {
        let runs = |id: &u32| self.rows.as_ref().is_some_and(|rows| rows.is_playable(*id));
        // P-kart-racing: a gate or Games row names a speedway; a quest row may stand in for it
        // only on that speedway's track, so Mustang Alli never zones a Wildwood Sprint to Wildwood.
        let track = |id: u32| self.rows.as_ref().and_then(|rows| rows.track(id));
        let entered_track = entered.and_then(track);
        self.next_row = wanted
            .iter()
            .copied()
            .filter(|id| entered_track.is_none_or(|scene| track(*id) == Some(scene)))
            .find(runs)
            .or_else(|| entered.filter(runs));
    }
    /// C6: whether a finish of row `id` reports a place (the row's track has an AI field).
    pub fn reports_placement(&self, id: u32) -> bool {
        self.ai_field(Some(id)).is_some()
    }
    /// The row a race with no choice runs as: the gates' `minigame_id`.
    fn default_row(&self) -> Option<u32> {
        self.gates.as_ref()?.minigame_id
    }
    /// P-kart-racing: the speedway row gate `npc_id` starts when no quest row is wanted
    /// (`kart_gates.json` `gate_rows`); `None` falls to the default row.
    pub fn gate_row(&self, npc_id: u64) -> Option<u32> {
        self.gates.as_ref()?.gate_rows.get(&npc_id).copied()
    }
    /// The track package the active race (or the row about to start) races on, when it is not
    /// the Wildwood package.
    fn track(&self, row: Option<u32>) -> Option<&Arc<TrackPackage>> {
        track_for(self.rows.as_deref(), &self.tracks, row)
    }
    /// Every track scene a race can zone into, the package's first.
    pub fn scenes(&self) -> Vec<&str> {
        std::iter::once(self.content.context.scene.scene.as_str())
            .chain(self.tracks.keys().map(String::as_str))
            .collect()
    }
    /// The row the active race runs as; saves that predate rows run the default one.
    pub fn row(&self) -> Option<u32> {
        let active = self.state.active.as_ref()?;
        active.row.or_else(|| self.default_row())
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
        // A gate bound to a speedway whose row the table lists but cannot run refuses rather than
        // starting the Wildwood race in its place; a package with no row table has one race only.
        if let (Some(row), Some(rows)) = (self.gate_row(npc_id), self.rows.as_ref())
            && !rows.is_playable(row)
        {
            return Err(EntryDenied::NoRace);
        }
        self.admit_play_at(busy, now_unix)
    }

    /// L097: admission for Play on the race's Games window row: every gate rule but the gate
    /// itself (the portal already proved the row is this race).
    pub fn admit_play(&self, busy: bool) -> Result<(), EntryDenied> {
        self.admit_play_at(busy, now_unix())
    }

    fn admit_play_at(&self, busy: bool, now_unix: u64) -> Result<(), EntryDenied> {
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
        Ok(self.state.active.as_ref().is_some_and(|a| {
            le::u32(inner, 3) == Some(a.context.encounter_id) && inner[7..15] == [255; 8]
        }))
    }
    /// ORIGINAL mid-race quit (T392). The title-bar Exit runs `Ui.QuitMiniGame`
    /// (`00939100`): the race's MiniGameState is in progress (`+0x61`, set by the
    /// 41/114 path `009beb70`), so it sends EncounterParticipantRequestExit 41/109
    /// (ctor `008a31a0`): a zero word, then the 41/107 instance it stored at
    /// `client+0xc10` (dispatcher `00aa36c0` case 0x6b). The retail racing capture
    /// sent exactly that mid-lap (frame 42898) and the server answered 41/121
    /// EncounterParticipantTerminateEncounter: header + flag byte 0 (frame 42925;
    /// case 0x79 -> `00938fe0` raises `MiniGameHandler:EncounterCancelled`, then
    /// `008d2730` drops the client state). Returns that answer when the quit names
    /// this race; the caller then runs the ordinary exit (no payout).
    pub fn quit_reply(&self, inner: &[u8]) -> io::Result<Option<Vec<u8>>> {
        if !inner.starts_with(&[41, 0, 109, 0]) {
            return Ok(None);
        }
        if inner.len() != 12 {
            return Err(invalid("invalid kart quit width"));
        }
        let Some(a) = &self.state.active else {
            return Ok(None);
        };
        if le::u32(inner, 4) != Some(0) || le::u32(inner, 8) != Some(a.context.instance_id) {
            return Ok(None);
        }
        let mut reply = vec![41, 0, 121, 0];
        reply.extend(a.context.encounter_id.to_le_bytes());
        reply.extend(a.context.instance_id.to_le_bytes());
        reply.push(0);
        Ok(Some(reply))
    }
    pub fn scene(&self) -> Option<&zone_transition::ZoneTransition> {
        self.state.active.as_ref().map(|a| &a.context.scene)
    }
    pub fn supports_location(&self, character: &CharacterState) -> bool {
        self.state.active.as_ref().is_some_and(|a| {
            character.guid == a.return_location.guid && character.zone_id == a.context.instance_id
        })
    }
    /// C6: the AI field row `row` races against on its track, when `kart_ai.json` names the track.
    fn ai_field(&self, row: Option<u32>) -> Option<crate::kart_ai::AiField> {
        let ai = self.ai.as_ref()?;
        let base = self.content.race.as_ref()?;
        let race = race_for(self.rows.as_deref(), &self.tracks, base, row).ok()?;
        match track_for(self.rows.as_deref(), &self.tracks, row) {
            Some(track) => ai.field(row, &track.scene.scene, &race, track.start_basis()),
            None => ai.field(
                row,
                &self.content.context.scene.scene,
                &race,
                [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            ),
        }
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
        let mut start = self.content.vehicle_guid_offset;
        // The chosen kart's look (`vehicle_looks.json`) replaces the captured definition; the
        // 33/11 tail after it stays, so the owner guid moves with the definition's length. A
        // look that does not apply leaves the default kart: a saved race must always load.
        if let Some(look) = self.chosen_look(1) {
            match vehicle_parts::apply_look(&mut content.entry_vehicles[0], &look, start) {
                Some(moved) => start = moved,
                None => eprintln!("kart look is not a definition; the default kart rides"),
            }
        }
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
        // P-kart-racing: a race on another track puts the kart on that track's grid, in the 33/5
        // body and the 33/11 block alike (the captured bodies carry the Wildwood grid). The 33/11
        // pose follows the owner guid, so it moves with `start` when a look replaces the definition.
        if let Some(track) = self.track(self.row()).cloned() {
            let basis = track.start_basis();
            write_pose(
                &mut content.client_data,
                CLIENT_DATA_POSE,
                basis,
                track.race.start_position,
            );
            write_pose(
                &mut content.entry_vehicles[0].attribute_block,
                start + VEHICLE_POSE_AFTER_GUID,
                basis,
                track.race.start_position,
            );
        }
        // C6: the client waits for as many NPC racers as `33/4` names; the field's introduction
        // frames follow the kart's registration (`handle`).
        let row = active.row.or_else(|| self.default_row());
        if let Some(field) = self.ai_field(row) {
            content.npc_count = field.count();
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
        // P-kart-racing: the offer names the scene the race zones into and the row it runs as.
        for axis in 0..3 {
            bytes[51 + axis * 4..55 + axis * 4]
                .copy_from_slice(&active.context.scene.position[axis].to_le_bytes());
        }
        if let (Some(row), Some(at)) = (self.row(), self.texts_offset)
            && let Some(texts) = self.rows.as_ref().and_then(|rows| rows.texts(row))
        {
            for (index, value) in [
                texts.name_id,
                texts.icon_id,
                texts.description_id,
                texts.difficulty,
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(slot) = bytes.get_mut(at + index * 4..at + index * 4 + 4) {
                    slot.copy_from_slice(&value.to_le_bytes());
                }
            }
            let id_at = bytes.len().saturating_sub(DETAILS_ROW_FROM_END);
            if le::u32(&bytes, id_at) == self.default_row() {
                bytes[id_at..id_at + 4].copy_from_slice(&row.to_le_bytes());
            }
        }
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
        let row = self.next_row.take().or_else(|| self.default_row());
        let mut staged = self.clone();
        let mut context = self.content.context.clone();
        if let Some(track) = self.track(row) {
            context.scene = track.scene.clone();
        }
        context.session = character.guid;
        context.instance_id = staged.state.next_instance;
        context.scene.id = context.instance_id as i32;
        let scene = context.scene.clone();
        staged.state.active = Some(ActiveKartEntry {
            context,
            return_location: character.clone(),
            previous_job: None,
            row,
        });
        staged.state.next_instance = next;
        staged.rebuild_entry()?;
        if let Some(base) = &staged.content.race {
            let mut config = race_for(staged.rows.as_deref(), &staged.tracks, base, row)?;
            let field = staged.ai_field(row);
            if let Some(field) = &field {
                crate::kart_ai::set_npc_count(&mut config, field.count());
            }
            let mut race = crate::kart_service::KartService::new(config).map_err(invalid)?;
            if let Some(field) = field {
                race.set_ai(field);
            }
            race.set_gate(EntryGate::Enabled);
            staged.race = Some(race);
            staged.sync_race();
        }
        let replies = vec![staged.details(true)?];
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
            if le::u32(inner, 4) != Some(context.encounter_id)
                || le::u32(inner, 8) != Some(context.instance_id)
                || le::u64(inner, 12) != Some(context.session)
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
            let mut replies = self.handle_entry(inner)?;
            if subtype == 9 && !replies.is_empty() {
                if let (Some(race), Some(active)) = (&mut self.race, &self.state.active) {
                    let context = &active.context;
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
                    // C6: the AI racers join the grid right after the kart, as the capture's did.
                    replies.extend(race.ai_intro_frames());
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
        Ok(self
            .claim_reward_with_event(inventory, profiles, Default::default())?
            .replies)
    }
    /// Claim a finished race once and report the race id for quest events.
    pub fn claim_reward_with_event(
        &mut self,
        inventory: &mut crate::inventory::Inventory,
        profiles: &mut crate::profiles::Profiles,
        boost: crate::consumable_effects::Boost,
    ) -> io::Result<KartRewardClaim> {
        if !self.race.as_ref().is_some_and(|race| race.finish_flag()) {
            return Ok(KartRewardClaim::default());
        }
        // Claiming consumes the finish offer. Stage that flag with both banks so
        // an overflow or refused price leaves a retryable, unpaid finish.
        let mut staged = self.clone();
        let mut staged_inventory = inventory.clone();
        let mut staged_profiles = profiles.clone();
        let Some(race) = staged.race.as_mut() else {
            return Ok(KartRewardClaim::default());
        };
        if !race.reward_priced() {
            return Ok(KartRewardClaim::default());
        }
        let Some(guid) = race.player_guid() else {
            return Ok(KartRewardClaim::default());
        };
        // T787: read the elapsed fact from the session's own finish time before the
        // one-shot handover consumes the finish offer.
        let elapsed_seconds = race
            .session()
            .and_then(crate::kart_racing::RaceSession::finish_elapsed_seconds)
            .filter(|seconds| *seconds <= crate::content::MAX_MINIGAME_ELAPSED_SECONDS);
        let (placement, beat_rival) = (race.placement(), race.beat_rival());
        let Some(finished) = race.take_finished_session() else {
            return Ok(KartRewardClaim::default());
        };
        let payout = match crate::kart_rewards::claim_race_reward(
            &staged.rewards,
            finished,
            &mut staged_inventory,
            &mut staged_profiles,
            guid,
            guid,
            boost,
        ) {
            Ok(payout) => payout,
            // A bank/XP limit is a refused payout, not a transport failure.
            // Keep the unpaid finish intact and release its progress frame.
            Err(_) => return Ok(KartRewardClaim::default()),
        };
        staged.state.last_paid_unix = Some(now_unix());
        staged.sync_race();
        let minigame_id = staged.row();
        *self = staged;
        *inventory = staged_inventory;
        *profiles = staged_profiles;
        Ok(KartRewardClaim {
            replies: payout.replies,
            minigame_id,
            elapsed_seconds,
            placement,
            beat_rival,
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
