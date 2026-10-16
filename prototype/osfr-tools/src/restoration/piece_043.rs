//! Piece 043 - pet trainer and card (TCG) progression: recovered profile rows,
//! the job level-up notification contract and the milestone unlock table.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/043 Pet trainer and card progression.md`):
//!
//! * `decompiled/client-ui/ScriptsBase.lua`, `NotificationHandler:JobLevelUp`:
//!   supported badge image sets `{10,11,12,13,19,20,32,1783,1785,1999,46,4842,4843}`,
//!   movies `t[1783] = "levelup_tcg"` and `t[1785] = "levelup_pettrainer"`,
//!   sounds `s[1783] = {[1]=16114,[4]=16115,[8]=16116,[12]=16117,[16]=16118}` and
//!   `s[1785] = {[1]=16119,[4]=16120,[8]=16121,[12]=16122,[16]=16123}`,
//!   message locale 919922 for level 1 and 919923 otherwise.
//! * Locale milestone blocks 34070-34074 / 34150-34154 (Card Duelist) and
//!   34110-34114 / 34190-34194 (Pet Trainer) at job levels 4, 8, 12, 16 and 20.
//! * Recorded `PacketSendSelfToClient.json` profile rows: Card Duelist is job 120
//!   (name 2410, description 2411) and the 16 recorded rows match the pinned
//!   community `Profiles.json` catalogue 16/16. No Pet Trainer row exists in the
//!   recorded profile, so its job id stays unrecovered.
//! * `FUN_0089b240` installs `PacketSendSelfToClient::vftable` with opcode 12;
//!   the native `JobLevelUp` dispatch passes five values.
//! * Asset manifest: `levelup_pettrainer.gfx` / `levelup_tcg.gfx` and the per-job
//!   audio rows `levelup_{pettrainer,cardduelist}_level{1,5,10,15,20,_misc}.mp3`.
//!
//! RECONSTRUCTED, labelled at each call site: the once-per-instance outcome
//! ledger, the caller-supplied XP curve (the real thresholds are unrecovered) and
//! the non-member level-4 cap, which is community-documented rather than
//! first-party. Award amounts are never invented here; the owning activity module
//! passes the XP value of an outcome it has already validated.

use std::collections::BTreeSet;

/// Highest job level the recovered achievement block covers (level 20 rows).
pub const MAX_JOB_LEVEL: u32 = 20;
/// Job levels that carry a recovered achievement milestone row.
pub const ACHIEVEMENT_MILESTONES: [u32; 5] = [4, 8, 12, 16, 20];
/// Job levels with a recovered level-up sound row in the Lua table.
pub const LEVEL_UP_SOUND_LEVELS: [u32; 5] = [1, 4, 8, 12, 16];
/// Badge image sets the Lua level-up handler accepts (`goodIndices`).
pub const SUPPORTED_LEVEL_UP_BADGE_SETS: [u32; 13] =
    [10, 11, 12, 13, 19, 20, 32, 1783, 1785, 1999, 46, 4842, 4843];
/// Locale id shown when the first level of a job is reached.
pub const LEVEL_UP_MESSAGE_LEVEL_ONE: u32 = 919922;
/// Locale id shown for every other job level-up.
pub const LEVEL_UP_MESSAGE_OTHER_LEVELS: u32 = 919923;

/// Card Duelist profile job id from the recorded profile sample.
pub const CARD_DUELIST_JOB_ID: u32 = 120;
/// Card Duelist localized job name id (`JobName` in the profile record).
pub const CARD_DUELIST_JOB_NAME_LOCALE: u32 = 2410;
/// Card Duelist localized job description id.
pub const CARD_DUELIST_JOB_DESCRIPTION_LOCALE: u32 = 2411;
/// Card Duelist badge image set; also the level-up table key in the client Lua.
pub const CARD_DUELIST_BADGE_IMAGE_SET: u32 = 1783;

/// Pet Trainer localized job name id. This is the only recovered Pet Trainer
/// identity row; the job is absent from the recorded profile sample.
pub const PET_TRAINER_JOB_NAME_LOCALE: u32 = 17419;
/// Pet Trainer badge image set; also the level-up table key in the client Lua.
pub const PET_TRAINER_BADGE_IMAGE_SET: u32 = 1785;
/// No recorded profile row and no shipped catalogue row carries a Pet Trainer
/// job id, so it stays explicitly unrecovered instead of being guessed.
pub const PET_TRAINER_PROFILE_JOB_ID: Option<u32> = None;

/// A job level-up milestone from the recovered locale block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MilestoneUnlock {
    pub level: u32,
    pub title_locale: u32,
    pub description_locale: u32,
}

/// Recovered level-up presentation row for one job badge image set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelUpPresentation {
    pub badge_image_set: u32,
    pub movie: &'static str,
    /// Sound ids parallel to [`LEVEL_UP_SOUND_LEVELS`].
    pub sounds: [u32; 5],
}

impl LevelUpPresentation {
    /// Sound id for the exact level, mirroring the Lua table lookup. Levels the
    /// recovered table does not key are `None`; the client still plays the movie.
    pub fn sound_for_level(&self, level: u32) -> Option<u32> {
        LEVEL_UP_SOUND_LEVELS
            .iter()
            .position(|candidate| *candidate == level)
            .map(|index| self.sounds[index])
    }
}

/// Card Duelist level-up row (`t[1783]`, `s[1783]`).
pub const CARD_DUELIST_LEVEL_UP: LevelUpPresentation = LevelUpPresentation {
    badge_image_set: CARD_DUELIST_BADGE_IMAGE_SET,
    movie: "levelup_tcg",
    sounds: [16114, 16115, 16116, 16117, 16118],
};

/// Pet Trainer level-up row (`t[1785]`, `s[1785]`).
pub const PET_TRAINER_LEVEL_UP: LevelUpPresentation = LevelUpPresentation {
    badge_image_set: PET_TRAINER_BADGE_IMAGE_SET,
    movie: "levelup_pettrainer",
    sounds: [16119, 16120, 16121, 16122, 16123],
};

const CARD_DUELIST_MILESTONES: [MilestoneUnlock; 5] = [
    MilestoneUnlock {
        level: 4,
        title_locale: 34070,
        description_locale: 34150,
    },
    MilestoneUnlock {
        level: 8,
        title_locale: 34071,
        description_locale: 34151,
    },
    MilestoneUnlock {
        level: 12,
        title_locale: 34072,
        description_locale: 34152,
    },
    MilestoneUnlock {
        level: 16,
        title_locale: 34073,
        description_locale: 34153,
    },
    MilestoneUnlock {
        level: 20,
        title_locale: 34074,
        description_locale: 34154,
    },
];

const PET_TRAINER_MILESTONES: [MilestoneUnlock; 5] = [
    MilestoneUnlock {
        level: 4,
        title_locale: 34110,
        description_locale: 34190,
    },
    MilestoneUnlock {
        level: 8,
        title_locale: 34111,
        description_locale: 34191,
    },
    MilestoneUnlock {
        level: 12,
        title_locale: 34112,
        description_locale: 34192,
    },
    MilestoneUnlock {
        level: 16,
        title_locale: 34113,
        description_locale: 34193,
    },
    MilestoneUnlock {
        level: 20,
        title_locale: 34114,
        description_locale: 34194,
    },
];

/// Jobs owned by this piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressionJob {
    CardDuelist,
    PetTrainer,
}

impl ProgressionJob {
    pub fn name_locale(self) -> u32 {
        match self {
            ProgressionJob::CardDuelist => CARD_DUELIST_JOB_NAME_LOCALE,
            ProgressionJob::PetTrainer => PET_TRAINER_JOB_NAME_LOCALE,
        }
    }

    pub fn badge_image_set(self) -> u32 {
        match self {
            ProgressionJob::CardDuelist => CARD_DUELIST_BADGE_IMAGE_SET,
            ProgressionJob::PetTrainer => PET_TRAINER_BADGE_IMAGE_SET,
        }
    }

    pub fn level_up(self) -> &'static LevelUpPresentation {
        match self {
            ProgressionJob::CardDuelist => &CARD_DUELIST_LEVEL_UP,
            ProgressionJob::PetTrainer => &PET_TRAINER_LEVEL_UP,
        }
    }

    pub fn milestones(self) -> &'static [MilestoneUnlock; 5] {
        match self {
            ProgressionJob::CardDuelist => &CARD_DUELIST_MILESTONES,
            ProgressionJob::PetTrainer => &PET_TRAINER_MILESTONES,
        }
    }
}

/// Levels the client resolves by `jobBadgeImageSetId`; the other supported sets
/// belong to the sibling progression items and are deliberately not claimed here.
fn presentation_for(badge_image_set: u32) -> Option<&'static LevelUpPresentation> {
    let row = if badge_image_set == CARD_DUELIST_BADGE_IMAGE_SET {
        &CARD_DUELIST_LEVEL_UP
    } else if badge_image_set == PET_TRAINER_BADGE_IMAGE_SET {
        &PET_TRAINER_LEVEL_UP
    } else {
        return None;
    };
    if SUPPORTED_LEVEL_UP_BADGE_SETS.contains(&badge_image_set) {
        Some(row)
    } else {
        None
    }
}

/// What the client shows when a job level is reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelUpNotification {
    pub badge_image_set: u32,
    pub level: u32,
    pub movie: &'static str,
    /// `None` for levels the recovered sound table does not key; the movie still
    /// plays in that case, exactly like the original handler.
    pub sound_id: Option<u32>,
    pub message_locale: u32,
}

/// Build the level-up presentation for a badge image set and level.
///
/// Returns `None` when the badge image set is not one of the sets the recovered
/// handler accepts; the original code plays nothing at all in that case.
pub fn level_up_notification(badge_image_set: u32, level: u32) -> Option<LevelUpNotification> {
    let row = presentation_for(badge_image_set)?;
    Some(LevelUpNotification {
        badge_image_set,
        level,
        movie: row.movie,
        sound_id: row.sound_for_level(level),
        message_locale: if level == 1 {
            LEVEL_UP_MESSAGE_LEVEL_ONE
        } else {
            LEVEL_UP_MESSAGE_OTHER_LEVELS
        },
    })
}

/// Which activity produced a validated outcome. Trick rounds and card matches
/// keep separate instance spaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutcomeSource {
    PetTrick,
    TcgMatch,
}

/// Result of one activity instance, as validated by the owning activity module.
///
/// The XP value is carried by the activity module because the original
/// per-outcome award amounts are unrecovered; this module never invents one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeResult {
    Completed { xp: u32 },
    Abandoned,
    Lost,
}

/// One activity outcome that the owning module (piece 011 trick rounds, piece 010
/// card matches) has already validated.
///
/// Construct these only from a validated completion; a client-reported score or
/// packet on its own is not a validated outcome and must never reach this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidatedOutcome {
    source: OutcomeSource,
    instance_id: u64,
    result: OutcomeResult,
}

impl ValidatedOutcome {
    pub fn pet_trick(instance_id: u64, result: OutcomeResult) -> Self {
        Self {
            source: OutcomeSource::PetTrick,
            instance_id,
            result,
        }
    }

    pub fn tcg_match(instance_id: u64, result: OutcomeResult) -> Self {
        Self {
            source: OutcomeSource::TcgMatch,
            instance_id,
            result,
        }
    }

    pub fn source(&self) -> OutcomeSource {
        self.source
    }

    pub fn instance_id(&self) -> u64 {
        self.instance_id
    }

    pub fn result(&self) -> OutcomeResult {
        self.result
    }
}

/// Caller-supplied XP curve: `thresholds[level - 1]` is the XP a character must
/// accumulate inside `level` to reach `level + 1`.
///
/// The original thresholds are not recovered, so this module never ships one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelCurve {
    thresholds: Vec<u32>,
}

impl LevelCurve {
    pub fn new(thresholds: Vec<u32>) -> Self {
        Self { thresholds }
    }

    pub fn required_for(&self, level: u32) -> Result<u32, ProgressionError> {
        let index = level.saturating_sub(1) as usize;
        self.thresholds
            .get(index)
            .copied()
            .ok_or(ProgressionError::CurveTooShort(level))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressionError {
    /// The same validated instance was already credited.
    DuplicateOutcome,
    /// The outcome belongs to a different job than the ledger tracks.
    JobMismatch,
    /// Instance id 0 is not a real activity instance.
    InvalidInstanceId,
    InvalidLevel(u32),
    /// Saved level exceeds the cap for the membership state.
    InvalidLevelForMembership(u32),
    /// A saved unlock level is not one of the recovered milestones.
    UnknownMilestone(u32),
    /// A saved unlock level is above the saved job level.
    MilestoneNotReached(u32),
    /// The supplied curve has no threshold for this level.
    CurveTooShort(u32),
}

impl std::fmt::Display for ProgressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgressionError::DuplicateOutcome => write!(f, "outcome instance already credited"),
            ProgressionError::JobMismatch => write!(f, "outcome belongs to another job"),
            ProgressionError::InvalidInstanceId => write!(f, "instance id 0 is not an instance"),
            ProgressionError::InvalidLevel(level) => write!(f, "invalid job level {level}"),
            ProgressionError::InvalidLevelForMembership(level) => {
                write!(f, "job level {level} exceeds the membership cap")
            }
            ProgressionError::UnknownMilestone(level) => {
                write!(f, "level {level} is not a recovered milestone")
            }
            ProgressionError::MilestoneNotReached(level) => {
                write!(f, "milestone {level} is above the saved job level")
            }
            ProgressionError::CurveTooShort(level) => {
                write!(f, "xp curve has no threshold for level {level}")
            }
        }
    }
}

impl std::error::Error for ProgressionError {}

/// Saved progression, mirroring the recovered profile record's `JobLevel`,
/// `JobLevelXP` and `JobLevelTotalXP` fields plus the emitted milestone levels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressionSnapshot {
    pub job: ProgressionJob,
    pub level: u32,
    /// `JobLevelXP` - XP inside the current level.
    pub xp_in_level: u32,
    /// `JobLevelTotalXP` - career XP for the job.
    pub xp_total: u32,
    pub unlocked_levels: Vec<u32>,
}

/// What one validated outcome produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedGrants {
    /// XP credited by this outcome as reported by the owning activity module.
    pub xp: u32,
    /// Level reached by this outcome, when a level-up happened.
    pub leveled_to: Option<u32>,
    /// Milestones crossed by this outcome, in ascending level order.
    pub unlocks: Vec<MilestoneUnlock>,
    /// Presentation for the final level reached, when a level-up happened.
    pub notification: Option<LevelUpNotification>,
}

/// Bounded progression ledger for the two jobs owned by this piece.
#[derive(Clone, Debug)]
pub struct ProgressionLedger {
    job: ProgressionJob,
    member: bool,
    level: u32,
    xp_in_level: u32,
    xp_total: u32,
    unlocked_levels: Vec<u32>,
    credited: BTreeSet<(OutcomeSource, u64)>,
}

impl ProgressionLedger {
    pub fn new(job: ProgressionJob, member: bool) -> Self {
        Self {
            job,
            member,
            level: 1,
            xp_in_level: 0,
            xp_total: 0,
            unlocked_levels: Vec::new(),
            credited: BTreeSet::new(),
        }
    }

    pub fn from_snapshot(
        snapshot: ProgressionSnapshot,
        member: bool,
    ) -> Result<Self, ProgressionError> {
        if snapshot.level == 0 || snapshot.level > MAX_JOB_LEVEL {
            return Err(ProgressionError::InvalidLevel(snapshot.level));
        }
        let cap = if member {
            MAX_JOB_LEVEL
        } else {
            NON_MEMBER_JOB_LEVEL_CAP
        };
        if snapshot.level > cap {
            return Err(ProgressionError::InvalidLevelForMembership(snapshot.level));
        }
        for level in &snapshot.unlocked_levels {
            if !ACHIEVEMENT_MILESTONES.contains(level) {
                return Err(ProgressionError::UnknownMilestone(*level));
            }
            if *level > snapshot.level {
                return Err(ProgressionError::MilestoneNotReached(*level));
            }
        }
        Ok(Self {
            job: snapshot.job,
            member,
            level: snapshot.level,
            xp_in_level: snapshot.xp_in_level,
            xp_total: snapshot.xp_total,
            unlocked_levels: snapshot.unlocked_levels,
            credited: BTreeSet::new(),
        })
    }

    pub fn snapshot(&self) -> ProgressionSnapshot {
        ProgressionSnapshot {
            job: self.job,
            level: self.level,
            xp_in_level: self.xp_in_level,
            xp_total: self.xp_total,
            unlocked_levels: self.unlocked_levels.clone(),
        }
    }

    pub fn job(&self) -> ProgressionJob {
        self.job
    }

    pub fn level(&self) -> u32 {
        self.level
    }

    pub fn xp_in_level(&self) -> u32 {
        self.xp_in_level
    }

    pub fn xp_total(&self) -> u32 {
        self.xp_total
    }

    pub fn unlocked_levels(&self) -> &[u32] {
        &self.unlocked_levels
    }

    /// Non-members cap at level 4. This rule is community-documented (fandom
    /// wiki, "Jobs") rather than first-party, so it is isolated here.
    pub fn member_level_cap(&self) -> u32 {
        if self.member {
            MAX_JOB_LEVEL
        } else {
            NON_MEMBER_JOB_LEVEL_CAP
        }
    }

    pub fn max_level(&self) -> u32 {
        MAX_JOB_LEVEL
    }

    /// Credit one validated outcome.
    ///
    /// Only `Completed` results grant XP; `Abandoned` and `Lost` credit nothing
    /// and do not consume the instance id, so a later valid completion of the
    /// same instance can still be credited. A replayed completion is refused.
    pub fn apply(
        &mut self,
        outcome: &ValidatedOutcome,
        curve: &LevelCurve,
    ) -> Result<AppliedGrants, ProgressionError> {
        if outcome.job_source_job() != self.job {
            return Err(ProgressionError::JobMismatch);
        }
        if outcome.instance_id == 0 {
            return Err(ProgressionError::InvalidInstanceId);
        }
        let OutcomeResult::Completed { xp } = outcome.result else {
            return Ok(AppliedGrants {
                xp: 0,
                leveled_to: None,
                unlocks: Vec::new(),
                notification: None,
            });
        };
        if !self.credited.insert((outcome.source, outcome.instance_id)) {
            return Err(ProgressionError::DuplicateOutcome);
        }

        self.xp_total = self.xp_total.saturating_add(xp);
        self.xp_in_level = self.xp_in_level.saturating_add(xp);
        let cap = self.member_level_cap();
        let mut unlocks = Vec::new();
        let mut reached = None;
        while self.level < cap {
            let required = curve.required_for(self.level)?;
            if self.xp_in_level < required {
                break;
            }
            self.xp_in_level -= required;
            self.level += 1;
            reached = Some(self.level);
            let milestone = self
                .job
                .milestones()
                .iter()
                .find(|milestone| milestone.level == self.level);
            if let Some(milestone) = milestone {
                if !self.unlocked_levels.contains(&self.level) {
                    self.unlocked_levels.push(self.level);
                    unlocks.push(*milestone);
                }
            }
        }

        let notification =
            reached.and_then(|level| level_up_notification(self.job.badge_image_set(), level));

        Ok(AppliedGrants {
            xp,
            leveled_to: reached,
            unlocks,
            notification,
        })
    }
}

impl ValidatedOutcome {
    fn job_source_job(&self) -> ProgressionJob {
        match self.source {
            // Trick rounds feed the Pet Trainer job; card matches feed Card Duelist.
            OutcomeSource::PetTrick => ProgressionJob::PetTrainer,
            OutcomeSource::TcgMatch => ProgressionJob::CardDuelist,
        }
    }
}

/// Community-documented non-member cap (fandom wiki, "Jobs"); not first-party.
pub const NON_MEMBER_JOB_LEVEL_CAP: u32 = 4;
