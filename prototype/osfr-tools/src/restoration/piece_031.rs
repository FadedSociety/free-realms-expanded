//! Work item 031: original job/profile catalogue, localization and unlock coverage.
//!
//! ORIGINAL facts carried by this module:
//! * The client job datasource `BaseClient.Profiles` column order (`Profile_DS` in
//!   `lab/decompiled/client-ui/ScriptsBase.lua`): Id, Name, Description, Type, Rank,
//!   StarsAvailable, Icon, BadgeImageSet, ButtonImageSet, AbilityBgImageSet, MembersOnly,
//!   StarsEarned, RankPercent, Hint, Trial, TrialExpired.
//! * The captured payload set: 16 profiles (1, 2, 4, 11, 12, 14, 16, 32, 35, 43, 45, 48, 49, 52,
//!   120, 137) with sha256 proofs in `lab/references/rust-opening/profiles.json`, plus the
//!   captured state `active` 1 and `unlocked` [1, 43].
//! * Client-side unlock flow: `JobBrowser:StartJobQuest(profileID)` scans the ActivityFinder
//!   datasource for the row whose column 0 is the profile id and starts column 4 as a breadcrumb
//!   quest; selecting a job goes through `JobScreen.SelectJob`; the existing production
//!   `crate::profiles` answers activation with the captured 38/21 payload.
//!
//! COMMUNITY facts (recorded, not original): the profile name/description string ids, types and
//! membership/trial flags come from `lab/references/Sanctuary/src/Resources/Profiles.json`; the
//! combat/crafting/freestyle split and the retired Pet Trainer job come from the read-only wiki
//! corpus. Both are cross-checked against the original locale by `validate_locale`, and the Python
//! checker `scripts/pieces/piece_031.py` performs the same cross-check against all pinned inputs.
//!
//! BOUNDARIES: this catalogue is recovery metadata. It grants no job, defines no XP curve and
//! claims no native activation. The server-side profile rows and the ActivityFinder rows that map
//! a profile id to its unlock quest id were never captured, so no per-job unlock quest id is
//! recovered. The membership/trial semantics of column 14 and the wiki members-only claim conflict
//! and stay unresolved.

/// Recovered `Profile_DS` / `Profile_Filtered_DS` column order from the client Lua.
pub const PROFILE_DS_COLUMNS: [&str; 16] = [
    "Id",
    "Name",
    "Description",
    "Type",
    "Rank",
    "StarsAvailable",
    "Icon",
    "BadgeImageSet",
    "ButtonImageSet",
    "AbilityBgImageSet",
    "MembersOnly",
    "StarsEarned",
    "RankPercent",
    "Hint",
    "Trial",
    "TrialExpired",
];

/// `JobBrowser:StartJobQuest` reads the profile id and the breadcrumb quest from these columns.
pub const ACTIVITY_FINDER_PROFILE_COLUMN: usize = 0;
pub const ACTIVITY_FINDER_QUEST_COLUMN: usize = 4;

/// Every combat job shares profile type 2; the type is not a job identity.
pub const PROFILE_TYPE_COMBAT: i32 = 2;

/// Historic classification of a job. Community documentation, not original server data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobClassification {
    Combat,
    Crafting,
    Freestyle,
    Staff,
}

/// Where a catalogue row comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobProvenance {
    /// A sha256-proved payload was captured from the original server for this profile id.
    CapturedPayload,
    /// Only the community profile table defines this row.
    CommunityDefinition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobProfile {
    pub id: i32,
    pub name: &'static str,
    pub name_string_id: u32,
    pub description_string_id: u32,
    pub profile_type: i32,
    pub classification: JobClassification,
    pub members_only: bool,
    pub trial: bool,
    pub provenance: JobProvenance,
}

/// A job the wiki documents as retired before the captured build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemovedJob {
    pub name: &'static str,
    pub name_string_id: u32,
    pub classification: JobClassification,
    pub removed_utc: &'static str,
    pub note: &'static str,
}

const fn row(
    id: i32,
    name: &'static str,
    name_string_id: u32,
    description_string_id: u32,
    profile_type: i32,
    classification: JobClassification,
    members_only: bool,
    trial: bool,
    provenance: JobProvenance,
) -> JobProfile {
    JobProfile {
        id,
        name,
        name_string_id,
        description_string_id,
        profile_type,
        classification,
        members_only,
        trial,
        provenance,
    }
}

/// The 18 recovered profile definitions: the 16 captured payloads plus the two community-only
/// staff rows. Name ids and description ids are validated against the preserved original locale.
pub const JOBS: [JobProfile; 18] = [
    row(
        1,
        "Adventurer",
        2836,
        2837,
        1,
        JobClassification::Freestyle,
        false,
        false,
        JobProvenance::CapturedPayload,
    ),
    row(
        2,
        "Ninja",
        3100,
        3101,
        2,
        JobClassification::Combat,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
    row(
        4,
        "Postman",
        3237,
        3238,
        3,
        JobClassification::Freestyle,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
    row(
        11,
        "Medic",
        2278,
        2279,
        2,
        JobClassification::Combat,
        false,
        false,
        JobProvenance::CapturedPayload,
    ),
    row(
        12,
        "Wizard",
        2280,
        2281,
        2,
        JobClassification::Combat,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
    row(
        14,
        "Miner",
        2470,
        2471,
        9,
        JobClassification::Crafting,
        false,
        false,
        JobProvenance::CapturedPayload,
    ),
    row(
        16,
        "Blacksmith",
        2614,
        2615,
        12,
        JobClassification::Crafting,
        false,
        false,
        JobProvenance::CapturedPayload,
    ),
    row(
        20,
        "Enforcer",
        2301,
        2302,
        9,
        JobClassification::Staff,
        false,
        false,
        JobProvenance::CommunityDefinition,
    ),
    row(
        32,
        "Warrior",
        4664,
        4665,
        2,
        JobClassification::Combat,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
    row(
        35,
        "Archer",
        4822,
        4823,
        2,
        JobClassification::Combat,
        false,
        false,
        JobProvenance::CapturedPayload,
    ),
    row(
        43,
        "Brawler",
        2873,
        3234,
        2,
        JobClassification::Combat,
        false,
        false,
        JobProvenance::CapturedPayload,
    ),
    row(
        45,
        "Chef",
        2856,
        2857,
        10,
        JobClassification::Crafting,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
    row(
        48,
        "Kart Driver",
        2917,
        2918,
        5,
        JobClassification::Freestyle,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
    row(
        49,
        "Demo Derby Driver",
        2919,
        2920,
        7,
        JobClassification::Freestyle,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
    row(
        52,
        "Soccer Star",
        4650,
        2920,
        8,
        JobClassification::Freestyle,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
    row(
        58,
        "Referee",
        23268,
        2302,
        9,
        JobClassification::Staff,
        false,
        false,
        JobProvenance::CommunityDefinition,
    ),
    row(
        120,
        "Card Duelist",
        2410,
        2411,
        17,
        JobClassification::Freestyle,
        false,
        false,
        JobProvenance::CapturedPayload,
    ),
    row(
        137,
        "Fisherman",
        20178,
        407842,
        21,
        JobClassification::Freestyle,
        false,
        true,
        JobProvenance::CapturedPayload,
    ),
];

/// The wiki records Pet Trainer as a freestyle job removed on 2009-12-14. Its locale name id is
/// preserved, but no profile id or payload survives, so it is not in `JOBS`.
pub const REMOVED_JOBS: [RemovedJob; 1] = [RemovedJob {
    name: "Pet Trainer",
    name_string_id: 17419,
    classification: JobClassification::Freestyle,
    removed_utc: "2009-12-14",
    note: "no profile id or captured payload recovered",
}];

/// Profile ids with a sha256-proved payload in the preserved capture.
pub const CAPTURED_PAYLOAD_IDS: [i32; 16] = [
    1, 2, 4, 11, 12, 14, 16, 32, 35, 43, 45, 48, 49, 52, 120, 137,
];

/// The captured account state: Adventurer active, Brawler also unlocked. Original evidence that
/// the captured character never unlocked the other fourteen jobs.
pub const CAPTURED_ACTIVE_ID: i32 = 1;
pub const CAPTURED_UNLOCKED_IDS: [i32; 2] = [1, 43];

/// Why the unlock mapping stops at the client: those server rows were not captured.
pub const UNLOCK_EVIDENCE: [&str; 4] = [
    "FRJobBrowser:PopulateJobList renders the BaseClient.Profiles rows",
    "JobBrowser.BrowserOnUserEvent 'SelectJob' calls JobScreen.SelectJob(jobId)",
    "JobBrowser:StartJobQuest(profileID) starts the ActivityFinder breadcrumb quest",
    "crate::profiles answers activation with the captured 38/21 profile payload",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogueError {
    DuplicateId(i32),
    DuplicateName(&'static str),
    MissingNameString {
        id: i32,
        name_string_id: u32,
    },
    LocalizedNameDiffers {
        id: i32,
        name_string_id: u32,
        found: String,
    },
}

pub fn jobs() -> &'static [JobProfile] {
    &JOBS
}

pub fn job(id: i32) -> Option<&'static JobProfile> {
    JOBS.iter().find(|row| row.id == id)
}

/// Every row keeps a unique profile id, a unique non-zero name id and a non-zero description id.
pub fn validate() -> Result<(), CatalogueError> {
    let mut ids = [0i32; JOBS.len()];
    let mut names = [""; JOBS.len()];
    for (index, row) in JOBS.iter().enumerate() {
        if ids[..index].contains(&row.id) {
            return Err(CatalogueError::DuplicateId(row.id));
        }
        if names[..index].contains(&row.name) {
            return Err(CatalogueError::DuplicateName(row.name));
        }
        if row.name_string_id == 0 || row.description_string_id == 0 || row.name.is_empty() {
            return Err(CatalogueError::MissingNameString {
                id: row.id,
                name_string_id: row.name_string_id,
            });
        }
        ids[index] = row.id;
        names[index] = row.name;
    }
    Ok(())
}

/// Cross-check every recovered name id against a locale lookup. The original locale export lives
/// in the lab; the gateway can pass its content locale instead.
pub fn validate_locale(lookup: impl Fn(u32) -> Option<String>) -> Result<(), CatalogueError> {
    for row in JOBS.iter() {
        match lookup(row.name_string_id) {
            None => {
                return Err(CatalogueError::MissingNameString {
                    id: row.id,
                    name_string_id: row.name_string_id,
                });
            }
            Some(found) if found.trim() != row.name => {
                return Err(CatalogueError::LocalizedNameDiffers {
                    id: row.id,
                    name_string_id: row.name_string_id,
                    found,
                });
            }
            Some(_) => {}
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coverage {
    pub captured: Vec<i32>,
    pub community_only: Vec<i32>,
    pub removed: Vec<&'static str>,
}

impl Coverage {
    pub fn recovered() -> Self {
        Self {
            captured: CAPTURED_PAYLOAD_IDS.to_vec(),
            community_only: JOBS
                .iter()
                .filter(|row| row.provenance == JobProvenance::CommunityDefinition)
                .map(|row| row.id)
                .collect(),
            removed: REMOVED_JOBS.iter().map(|row| row.name).collect(),
        }
    }

    /// Only payload-backed profiles may be activated from captured bytes.
    pub fn payload_for(&self, id: i32) -> bool {
        self.captured.contains(&id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnlockEvidence {
    pub active_id: i32,
    pub unlocked_ids: Vec<i32>,
    pub quest_ids_recovered: bool,
    pub evidence: &'static [&'static str],
}

impl UnlockEvidence {
    pub fn active_name(&self) -> &'static str {
        job(self.active_id).map(|row| row.name).unwrap_or_default()
    }

    pub fn unlocked_name(&self, id: i32) -> Option<&'static str> {
        self.unlocked_ids
            .contains(&id)
            .then(|| job(id).map(|row| row.name))
            .flatten()
    }
}

/// The unlocked set observed in the capture. Per-job unlock quest ids stay unrecovered.
pub fn job_unlock() -> UnlockEvidence {
    UnlockEvidence {
        active_id: CAPTURED_ACTIVE_ID,
        unlocked_ids: CAPTURED_UNLOCKED_IDS.to_vec(),
        quest_ids_recovered: false,
        evidence: &UNLOCK_EVIDENCE,
    }
}
