//! Optional ordinary-entry binding for the native demolition derby.
//!
//! The packet and match rules stay in `derby.rs`; this module binds one authenticated character to
//! one opt-in content package. The gate list is reconstructed policy (R), like the kart gates: an
//! NPC lists the derby only when named in `derby_gates.json`. Entry, phase acknowledgements and the
//! results hold use the existing native type-11 lifecycle.
//!
//! T791 hosts more than the default MiniGameData row. `derby_rows.json` adds rows that inherit every
//! unset field (vehicle, rules, pickups, rewards) from `derby_entry.json`. `derby_minigames.json`
//! lists the MiniGameData rows the package hosts (read by the content layer's
//! `read_minigame_rows`, so it has the same provenance and unique-id rules as any minigame table)
//! and proves each hosted id, the default row included, is TYPE 11 before the world loads. A missing
//! `derby_rows.json` still means the one default row.
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
    content::{MinigameRow, SourceTag, check_sources, read_minigame_rows},
    derby::{
        DEMO_DERBY_LOBBY_NAME_ID, DEMO_DERBY_TYPE_ID, DerbyAction, DerbyError,
        DerbyParticipantResult, DerbyPhase, DerbyPickup, DerbyRules, DerbyService,
        DerbyVehicleTemplate, NpcDriving, ParticipantKey,
    },
    rewards::{RewardDisplay, RewardGrant},
    zone_transition::ZoneTransition,
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
/// (R) Sanity bound on the extra MiniGameData rows one world may host.
pub const MAX_DERBY_ROWS: usize = 32;
/// Save-size bound on the paid-match ledger; the newest rows are kept.
pub const MAX_PAID_MATCHES: usize = 32;
const MAX_ROWS_BYTES: usize = 64 * 1024;
/// The hosted-row proof beside `derby_entry.json`; see the module docs.
pub const MINIGAMES_FILE: &str = "derby_minigames.json";
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
    /// coins and zero XP pays nothing, so an operator file without these keys still loads; its
    /// finished matches still enter the ledger and still credit `finish_minigame`.
    #[serde(default)]
    pub completion_reward_coins: u32,
    /// (R) Demo Derby Driver XP for the same completion; 0 pays no XP.
    #[serde(default)]
    pub completion_reward_xp: u32,
    /// L931: the arena the match zones into (the live op31 `sg_dm_arena` frame); none plays in place.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<ZoneTransition>,
    /// L931: the AI cars by slot, slot 1 first (the live 34/10 bodies and start transforms); a
    /// slot past the end reuses `vehicle`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub npc_vehicles: Vec<DerbyVehicleTemplate>,
    /// L959: the AI drivers by slot, slot 1 first (the live 35/2 names and bodies); none names no
    /// driver.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub npc_drivers: Vec<crate::derby::DerbyDriver>,
    /// Retired F117 payout row. Accepted and ignored only so an operator's F117-era
    /// `derby_entry.json` still loads; `completion_reward_coins`/`_xp` replaced it (F206/T219).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward: Option<serde_json::Value>,
}

/// One extra hosted row in `derby_rows.json`. Every unset field inherits `derby_entry.json`; a
/// row can therefore play exactly like the default package without restating it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DerbyRowContent {
    game_id: i32,
    npcs: Vec<u64>,
    /// The rule keys this row changes (L931: a partial object merged over the default rules; a full
    /// `DerbyRules` object replaces every key as before).
    #[serde(default)]
    rules: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    pickups: Option<Vec<DerbyPickup>>,
    #[serde(default)]
    completion_reward_coins: Option<u32>,
    #[serde(default)]
    completion_reward_xp: Option<u32>,
    /// L931 (R): chase-and-bump NPC drivers for this row; unset keeps inert placeholder NPCs.
    #[serde(default)]
    npc_driving: Option<NpcDriving>,
    source: Vec<SourceTag>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct DerbyRowsFile {
    schema_version: u32,
    /// L931 (R): the same for the default row (`derby_entry.json`), which has no row of its own.
    #[serde(default)]
    default_npc_driving: Option<NpcDriving>,
    rows: Vec<DerbyRowContent>,
}

/// One row this gateway hosts: the effective package (the default package with the row's
/// overrides applied) and the NPCs whose menu may enter it.
#[derive(Clone, Debug)]
pub struct DerbyRow {
    package: Arc<DerbyGatewayContent>,
    npcs: Vec<u64>,
    npc_driving: Option<NpcDriving>,
}

impl DerbyRow {
    /// The MiniGameData id the 39/16 offer names and a finished match credits.
    pub fn game_id(&self) -> i32 {
        self.package.game_id
    }

    /// The effective package; inheritance was resolved at load.
    pub fn package(&self) -> &DerbyGatewayContent {
        &self.package
    }

    /// L931 (R): the chase-and-bump NPC drivers this row runs, `None` for inert placeholder NPCs.
    pub fn npc_driving(&self) -> Option<NpcDriving> {
        self.npc_driving
    }
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
    /// Finished (paid and credited) match instances, oldest first, at most the newest
    /// `MAX_PAID_MATCHES`; a zero-reward package records its finishes too. The row it was played in
    /// is deliberately absent: instance ids are per match, so one ledger covers every hosted row.
    /// The save belongs to one character, so (this ledger, instance id) is the (character, match)
    /// key and another character's identical instance id is a different match. `next_instance`
    /// resumes above the newest row, so a pruned id is never reissued to a character that was paid
    /// for it.
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
    /// The default package's row first, then `derby_rows.json` in file order.
    rows: Vec<DerbyRow>,
    service: Option<DerbyService>,
    owner: Option<ParticipantKey>,
    next_instance: u32,
    previous_job: Option<i32>,
    /// Completion ledger, newest last; see `DerbyGatewayState::paid_matches`.
    paid_matches: Vec<i32>,
    /// L931: the arena visit of the live match.
    arena: Option<ArenaVisit>,
}

/// L931: a match played in its arena scene, as a fishing hole (`gameplay/fishing_scene.rs`).
#[derive(Clone, Debug)]
struct ArenaVisit {
    /// The world pose kept at entry; every way out zones back here.
    return_to: CharacterState,
    /// The 39/16 offer and its ready record, sent once the arena has loaded.
    pending: Vec<Vec<u8>>,
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

fn load_rows(directory: &Path) -> io::Result<DerbyRowsFile> {
    let path = directory.join("derby_rows.json");
    let Some(bytes) = read_regular_limited(&path, MAX_ROWS_BYTES, "derby rows")? else {
        return Ok(DerbyRowsFile::default());
    };
    let rows: DerbyRowsFile = serde_json::from_slice(&bytes).map_err(invalid)?;
    if rows.schema_version != 1 || rows.rows.len() > MAX_DERBY_ROWS {
        return Err(invalid("invalid derby rows file"));
    }
    Ok(rows)
}

fn load_minigames(directory: &Path) -> io::Result<Option<Vec<MinigameRow>>> {
    if !directory.join(MINIGAMES_FILE).exists() {
        return Ok(None);
    }
    read_minigame_rows(directory, MINIGAMES_FILE)
        .map(Some)
        .map_err(|error| invalid(error.to_string()))
}

/// Refuse a row's NPC driving numbers outside the service's bounds, at load rather than at entry.
fn check_driving(driving: Option<NpcDriving>, rules: &DerbyRules, game_id: i32) -> io::Result<()> {
    match driving {
        Some(driving) if !driving.is_valid(rules) => {
            Err(invalid(format!("derby row {game_id}: invalid npc_driving")))
        }
        _ => Ok(()),
    }
}

/// Refuse a hosted id the MiniGameData table does not list as a demolition-derby (TYPE 11) row.
fn require_derby_row(minigames: &[MinigameRow], game_id: i32, owner: &str) -> io::Result<()> {
    minigames
        .iter()
        .any(|entry| {
            i64::from(entry.id) == i64::from(game_id)
                && entry.game_type_id == DEMO_DERBY_TYPE_ID as u32
        })
        .then_some(())
        .ok_or_else(|| {
            invalid(format!(
                "{owner} {game_id} is not a {MINIGAMES_FILE} type-11 row"
            ))
        })
}

/// Resolve one extra row over the default package and enforce every bounds check. The caller has
/// already rejected a duplicate or non-positive id.
fn resolve_row(
    row: DerbyRowContent,
    default: &DerbyGatewayContent,
    gate_npcs: &[u64],
    minigames: Option<&[MinigameRow]>,
) -> io::Result<DerbyRow> {
    if row.npcs.is_empty() || row.npcs.len() > MAX_DERBY_GATES {
        return Err(invalid(format!(
            "derby row {} needs one to {MAX_DERBY_GATES} NPCs",
            row.game_id
        )));
    }
    let mut seen = BTreeSet::new();
    for npc in &row.npcs {
        if *npc == 0 || !seen.insert(*npc) {
            return Err(invalid(format!(
                "derby row {} lists NPC {npc} twice or as zero",
                row.game_id
            )));
        }
        if !gate_npcs.contains(npc) {
            return Err(invalid(format!(
                "derby row {} lists NPC {npc} outside derby_gates.json",
                row.game_id
            )));
        }
    }
    if let Some(minigames) = minigames {
        require_derby_row(minigames, row.game_id, "derby row")?;
    }
    check_sources(&format!("derby row {}", row.game_id), &row.source)
        .map_err(|error| invalid(error.to_string()))?;
    let rules = match row.rules {
        None => default.rules.clone(),
        Some(changes) => {
            let mut merged = serde_json::to_value(&default.rules).map_err(invalid)?;
            let keys =
                (merged.as_object_mut()).ok_or_else(|| invalid("derby rules are not an object"))?;
            keys.extend(changes);
            serde_json::from_value(merged)
                .map_err(|error| invalid(format!("derby row {}: {error}", row.game_id)))?
        }
    };
    let package = DerbyGatewayContent {
        schema_version: default.schema_version,
        game_id: row.game_id,
        vehicle: default.vehicle.clone(),
        scene: default.scene.clone(),
        npc_vehicles: default.npc_vehicles.clone(),
        npc_drivers: default.npc_drivers.clone(),
        rules,
        pickups: row.pickups.unwrap_or_else(|| default.pickups.clone()),
        completion_reward_coins: row
            .completion_reward_coins
            .unwrap_or(default.completion_reward_coins),
        completion_reward_xp: row
            .completion_reward_xp
            .unwrap_or(default.completion_reward_xp),
        reward: None,
    };
    validate_content(&package)
        .map_err(|error| invalid(format!("derby row {}: {error}", row.game_id)))?;
    check_driving(row.npc_driving, &package.rules, row.game_id)?;
    Ok(DerbyRow {
        package: Arc::new(package),
        npcs: row.npcs,
        npc_driving: row.npc_driving,
    })
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
        let rows = load_rows(directory)?;
        let minigames = load_minigames(directory)?;
        let mut gateway = Self::build(
            content,
            load_gates(directory)?,
            rows.default_npc_driving,
            rows.rows,
            minigames.as_deref(),
        )?;
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
    /// and by interactive content reloads. It hosts the default row only.
    pub fn new(content: DerbyGatewayContent, gates: Option<DerbyGates>) -> io::Result<Self> {
        Self::build(content, gates, None, Vec::new(), None)
    }

    /// Validate the default package and its gates, resolve every extra row, and host them in file
    /// order behind the default row. A world that lists extra rows without `derby_minigames.json`
    /// is refused: an untyped row could credit a non-derby MiniGameData id. When the table is
    /// present the default row must be listed in it too.
    fn build(
        content: DerbyGatewayContent,
        gates: Option<DerbyGates>,
        default_npc_driving: Option<NpcDriving>,
        rows: Vec<DerbyRowContent>,
        minigames: Option<&[MinigameRow]>,
    ) -> io::Result<Self> {
        validate_content(&content)?;
        check_driving(default_npc_driving, &content.rules, content.game_id)?;
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
        if !rows.is_empty() && minigames.is_none() {
            return Err(invalid(format!(
                "derby rows require {MINIGAMES_FILE} to prove their MiniGameData type"
            )));
        }
        if let Some(minigames) = minigames {
            require_derby_row(minigames, content.game_id, "derby default row")?;
        }
        let gate_npcs = gates
            .as_ref()
            .map(|gates| gates.npcs.clone())
            .unwrap_or_default();
        let mut game_ids = BTreeSet::from([content.game_id]);
        let mut hosted = Vec::with_capacity(rows.len() + 1);
        for row in rows {
            if row.game_id <= 0 {
                return Err(invalid(format!(
                    "invalid derby row game id {}",
                    row.game_id
                )));
            }
            if !game_ids.insert(row.game_id) {
                return Err(invalid(format!(
                    "duplicate derby row game id {}",
                    row.game_id
                )));
            }
            hosted.push(resolve_row(row, &content, &gate_npcs, minigames)?);
        }
        let content = Arc::new(content);
        hosted.insert(
            0,
            DerbyRow {
                package: Arc::clone(&content),
                npcs: gate_npcs,
                npc_driving: default_npc_driving,
            },
        );
        Ok(Self {
            content,
            gates: gates.map(Arc::new),
            rows: hosted,
            service: None,
            owner: None,
            next_instance: FIRST_DERBY_INSTANCE,
            previous_job: None,
            paid_matches: Vec::new(),
            arena: None,
        })
    }

    /// Every hosted row, the default package first and extras in file order.
    pub fn rows(&self) -> &[DerbyRow] {
        &self.rows
    }

    /// Rows `npc_id` may enter, in row order. A non-gate NPC yields nothing.
    pub fn rows_for(&self, npc_id: u64) -> impl Iterator<Item = &DerbyRow> {
        self.rows
            .iter()
            .filter(move |row| row.npcs.contains(&npc_id))
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

    /// L931: where a car of the live match is now (its last pose).
    pub fn driver_position(&self, key: crate::derby::ParticipantKey) -> Option<[f32; 3]> {
        self.service.as_ref()?.driver_position(key)
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
        self.admit_play(busy)
    }

    /// L097: admission for Play on the default derby's Games window row (ContentId
    /// [`Self::game_id`]): every gate rule but the gate itself.
    pub fn admit_play(&self, busy: bool) -> Result<(), DerbyEntryDenied> {
        if self.active() {
            return Err(DerbyEntryDenied::AlreadyPlaying);
        }
        if busy {
            return Err(DerbyEntryDenied::Busy);
        }
        Ok(())
    }

    /// Start the default row of a fresh solo derby and return the native 39/16 offer.
    pub fn begin(&mut self, character: &CharacterState) -> io::Result<Vec<u8>> {
        self.begin_at(character, 0)
    }

    pub fn begin_at(&mut self, character: &CharacterState, now_ms: u64) -> io::Result<Vec<u8>> {
        self.begin_row(character, now_ms, None)
    }

    /// T889: the 41/107 ready for the offer `begin_row` just returned; `None` with no match.
    pub fn ready_packet(&self) -> Option<Vec<u8>> {
        self.service.as_ref().map(DerbyService::ready_packet)
    }

    /// Start the named hosted row (`None` is the default row) and return its native 39/16 offer.
    /// The 39/16 game id, the running match and the completion credit all come from this package.
    pub fn begin_row(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
        game_id: Option<i32>,
    ) -> io::Result<Vec<u8>> {
        if self.active() {
            return Err(invalid("derby is already active"));
        }
        if character.guid == 0 {
            return Err(invalid("derby entry requires an authenticated character"));
        }
        let row = match game_id {
            Some(id) => self.rows.iter().find(|row| row.game_id() == id),
            None => self.rows.first(),
        }
        .ok_or_else(|| match game_id {
            Some(id) => invalid(format!("derby row {id} is not hosted")),
            None => invalid("derby has no hosted row"),
        })?;
        let (package, npc_driving) = (Arc::clone(&row.package), row.npc_driving);
        let instance = self.next_instance;
        if instance > i32::MAX as u32 {
            return Err(invalid("derby instance overflow"));
        }
        let owner = ParticipantKey(character.guid);
        let mut service = DerbyService::new(
            instance as i32,
            package.game_id,
            owner,
            package.vehicle.clone(),
            package.rules.clone(),
            package.pickups.clone(),
        )
        .map_err(|error| invalid(error.to_string()))?;
        service
            .set_npc_driving(npc_driving)
            .map_err(|error| invalid(error.to_string()))?;
        service
            .set_npc_vehicles(package.npc_vehicles.clone())
            .map_err(|error| invalid(error.to_string()))?;
        service
            .set_npc_drivers(package.npc_drivers.clone())
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

    /// L958: a quit in the race, ended through the results screen (`DerbyService::forfeit`).
    /// `None` when the packet is not a forfeit.
    pub fn forfeit(&mut self, inner: &[u8], now_ms: u64) -> io::Result<Option<Vec<Vec<u8>>>> {
        let Some(service) = &self.service else {
            return Ok(None);
        };
        let owner = self
            .owner
            .ok_or_else(|| invalid("derby service has no authenticated owner"))?;
        let mut staged = service.clone();
        let Some(actions) = staged
            .forfeit(owner, inner, now_ms)
            .map_err(|error| invalid(error.to_string()))?
        else {
            return Ok(None);
        };
        let replies = packets_for_owner(&actions, owner)?;
        self.service = Some(staged);
        Ok(Some(replies))
    }

    /// Retire an offer or a match before its race without a reward (a quit in the race is
    /// [`Self::forfeit`]). Malformed, foreign and stale cancellation
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

    /// Take the finished match this session may be paid and credited for: the live match is
    /// showing its results and the ledger has no row for its instance. Taking records the
    /// instance, so the same match never yields a second claim. `None` when the match is not
    /// finished or was already taken; the reward amount plays no part (see `completion_grant`).
    pub fn take_completion(&mut self) -> Option<i32> {
        let instance = self
            .service
            .as_ref()
            // L958: a forfeited race shows the results but is not a completion.
            .filter(|service| service.phase() == DerbyPhase::Results && !service.forfeited())
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

    /// (R) T787: the owner's own result facts once the match is showing Results.
    /// `None` before Results or for a foreign key; the quest credit then carries no
    /// score, placement or knock-out claim.
    pub fn completion_result(&self, guid: u64) -> Option<DerbyParticipantResult> {
        let key = ParticipantKey(guid);
        if self.owner != Some(key) {
            return None;
        }
        self.service.as_ref()?.participant_result(key)
    }

    /// The flat completion grant the entered row pays. It never carries items and never derives
    /// from a contact report, a score or a match duration; `display` is the ordinary reward toast.
    /// (R) The amounts are content, not recovered retail values. `None` for a 0 coin / 0 XP
    /// package, which pays nothing and sends no toast. L927: `activity_xp` is the Demo Derby
    /// Driver's rate per match (`job_xp_curve.json`); it replaces the package's XP when present.
    pub fn completion_grant(&self, guid: u64, activity_xp: Option<u32>) -> Option<RewardGrant> {
        let content = self.active_package();
        let job_xp = match activity_xp.unwrap_or(content.completion_reward_xp) {
            0 if content.completion_reward_coins == 0 => return None,
            0 => BTreeMap::new(),
            xp => BTreeMap::from([(DEMO_DERBY_DRIVER_PROFILE, xp)]),
        };
        Some(RewardGrant {
            coins: content.completion_reward_coins,
            items: Vec::new(),
            job_xp,
            sc: 0,
            display: RewardDisplay {
                source_guid: guid,
                player_guid: guid,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        })
    }

    /// The entered row's MiniGameData id (`derby_entry.json` is the default row; a live match uses
    /// the row it started). This is the game id the 39/16 offer names and the id a finished match
    /// credits to `finish_minigame`. `validate_content` keeps every row positive.
    pub fn game_id(&self) -> u32 {
        self.service
            .as_ref()
            .map_or(self.content.game_id, DerbyService::game_id) as u32
    }

    /// The entered row's effective package; the default package when no match is live.
    fn active_package(&self) -> &DerbyGatewayContent {
        self.service
            .as_ref()
            .and_then(|service| {
                self.rows
                    .iter()
                    .find(|row| row.game_id() == service.game_id())
            })
            .map_or(self.content.as_ref(), DerbyRow::package)
    }

    /// The ledger to persist; `None` keeps a save with no finished match byte-identical.
    pub fn saved_state(&self) -> Option<DerbyGatewayState> {
        (!self.paid_matches.is_empty()).then(|| DerbyGatewayState {
            previous_job: None,
            last_paid_unix: None,
            paid_matches: self.paid_matches.clone(),
        })
    }

    /// L931: the entered row's arena scene; `None` plays the match where the player stands.
    pub fn scene(&self) -> Option<&ZoneTransition> {
        self.active_package().scene.as_ref()
    }

    /// L931: zone into the arena; `pending` waits for its Loaded (`arena_loaded`).
    pub fn enter_arena(&mut self, return_to: CharacterState, pending: Vec<Vec<u8>>) {
        self.arena = Some(ArenaVisit { return_to, pending });
    }

    /// L931: the held start records once the arena has loaded; a second load resends nothing.
    pub fn arena_loaded(&mut self) -> Option<Vec<Vec<u8>>> {
        Some(std::mem::take(&mut self.arena.as_mut()?.pending))
    }

    /// L931: the arena scene while a visit lasts.
    pub fn arena_scene(&self) -> Option<&ZoneTransition> {
        self.arena.as_ref().and(self.scene())
    }

    /// L931: the world pose a save inside the arena stores.
    pub fn arena_return(&self) -> Option<&CharacterState> {
        Some(&self.arena.as_ref()?.return_to)
    }

    /// L931: end the visit; the world pose to zone back to.
    pub fn leave_arena(&mut self) -> Option<CharacterState> {
        self.arena.take().map(|visit| visit.return_to)
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
        // L959: driver NPCs are only cleared by the zone change back from an arena; a package
        // that plays in place would leave them standing in the world.
        || content.scene.is_none() && !content.npc_drivers.is_empty()
    {
        return Err(invalid("invalid derby content"));
    }
    let mut service = DerbyService::new(
        FIRST_DERBY_INSTANCE as i32,
        content.game_id,
        ParticipantKey(1),
        content.vehicle.clone(),
        content.rules.clone(),
        content.pickups.clone(),
    )
    .map_err(|error: DerbyError| invalid(error.to_string()))?;
    // The AI cars and drivers are checked at load too, not first at a player's entry.
    service
        .set_npc_vehicles(content.npc_vehicles.clone())
        .and_then(|()| service.set_npc_drivers(content.npc_drivers.clone()))
        .map_err(|error| invalid(error.to_string()))
}
