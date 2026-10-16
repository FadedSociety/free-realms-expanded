//! Work item 039: original Miner and Farmer gathering-job progression.
//!
//! ORIGINAL facts implemented here, with the recovered source of each:
//! * the farm/ranch rank ladder is the original client table `Resources/FactoryRanks.txt`
//!   (`HOUSING_INSTANCE_ID^EXPERIENCE_POINTS_REQUIRED^NAME_STRING_ID^RANK^
//!   GAIN_RANK_COMPOSITE_EFFECT^FACTORY_TYPE^YIELD_MULTIPLIER^REWARD_SET_ID^ITEM_CAP_MODIFIER^
//!   MEMBERS_ONLY`). FARMING instances 58 and 63 and RANCHING instance 60 each carry ranks 1..20
//!   with the same `EXPERIENCE_POINTS_REQUIRED` ladder; the two farm instances differ only in
//!   `REWARD_SET_ID`, and every row shares `NAME_STRING_ID` 0, `GAIN_RANK_COMPOSITE_EFFECT` 15263
//!   and `YIELD_MULTIPLIER` 0.0. `MEMBERS_ONLY` turns on at rank 5;
//! * `Resources/FactoryPlots.txt` attaches the rank tables to the player plots: plot 3 uses
//!   FARMING 58, plot 5 uses FARMING 63 and plot 7 uses RANCHING 60. Instances 57, 59, 62 and 64
//!   have no rank rows at all, so they are not progression tracks;
//! * the Miner vein gates are the original locale rows 1663..1666: tin requires rank 4, iron 8,
//!   silver 12 and gold 16. Copper carries no gate string anywhere in the client locale;
//! * the Miner rank gates are the original client requirement rows 17..20
//!   (`CheckRankLastProfileByType`, operator 6, profile type 9) at ranks 5, 9, 13 and 17;
//! * the rank wire contract the client reads: `Profile_DS` columns 4 `Rank`, 5 `StarsAvailable`,
//!   11 `StarsEarned` and 12 `RankPercent`; `ClientPcProfile`/`ClientPcData` fields `JobLevel`,
//!   `JobLevelXP`, `JobLevelTotalXP`; and the native `ClientUpdatePacketUpdateProfileRank`
//!   (length 0x12) with its `RankExperience` / `RankMaxExperience` tooltip fields.
//!
//! INFERRED here, never presented as shipped: that `EXPERIENCE_POINTS_REQUIRED` is the
//! cumulative total needed for its rank row (rank 1 requires 0 and the column name is the only
//! shipped semantics), and that `RankPercent` is the floored percentage of the current rank.
//! Both live in the helper functions, not in the data tables.
//!
//! Explicit gaps, do not imply these:
//! * the Miner per-rank xp thresholds are NOT recovered. The client ships no profile rank table
//!   (it loads `FactoryRanks.txt` and the other `Resources` tables only) and instead reads
//!   `RankExperience`/`RankMaxExperience` from the server, so `rank_of` answers
//!   [`ProgressionGap::XpThresholdsUnrecovered`] for a profile track;
//! * which physical farm each instance serves is not established: `FactoryPlots.txt` gives the
//!   instance ids and the community `Houses.json` gives property 33 `Briarwood Farm` and 34
//!   `Wilds Farm`, but no preserved row joins the two namespaces;
//! * reward-set contents (4274..4289, 4381..4396), item-cap semantics and the xp granted per
//!   mining/harvesting outcome are server content;
//! * [`NON_MEMBER_RANK_CAP`] is community wiki documentation, not a client or captured-server row.
//!
//! The duplicate-grant guard lives in [`GrantLedger`]: mining (work item 001) authorises one
//! award per finished `Type8Run::instance_id`, harvesting (002) authorises one award per finished
//! harvesting instance, and a replayed outcome is answered with [`GrantDecision::Duplicate`] so
//! the caller never applies xp twice. The ledger stores the authorised amounts only; it does not
//! grant currency or items and it does not replace `crate::profiles` as the profile store.
use std::{collections::BTreeMap, fmt};

/// Highest rank in every recovered rank table.
pub const MAX_RANK: u32 = 20;
/// COMMUNITY only (wiki `Jobs` page): most jobs cap at rank 4 without a membership. No client or
/// captured-server row carries this value, so it is exported as documentation.
pub const NON_MEMBER_RANK_CAP: u32 = 4;
/// The FARMING factory instances with a rank table.
pub const FARM_INSTANCE_IDS: [i32; 2] = [58, 63];
/// The RANCHING factory instance with a rank table.
pub const RANCH_INSTANCE_ID: i32 = 60;
/// ORIGINAL: `MEMBERS_ONLY` 1 from this rank on, in every recovered row.
pub const FARM_MEMBERS_ONLY_FROM_RANK: u32 = 5;
/// ORIGINAL: `GAIN_RANK_COMPOSITE_EFFECT` in every recovered row.
pub const GAIN_RANK_COMPOSITE_EFFECT: i32 = 15263;
/// ORIGINAL: `NAME_STRING_ID` in every recovered row. The rank has no localized name.
pub const FARM_NAME_STRING_ID: u32 = 0;
/// ORIGINAL: `YIELD_MULTIPLIER` in every recovered row.
pub const FARM_YIELD_MULTIPLIER: f32 = 0.0;

/// One ORIGINAL `FactoryRanks.txt` row. `reward_set` keeps the table's own sentinel `0`, which
/// means "no reward set on this rank"; [`Self::reward_set`] converts it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FarmRankRow {
    pub rank: u32,
    pub xp_required: u32,
    pub item_cap: u32,
    pub reward_set: i32,
}

impl FarmRankRow {
    /// ORIGINAL `MEMBERS_ONLY` column: 1 from rank 5 on, in every recovered row.
    pub fn members_only(&self) -> bool {
        self.rank >= FARM_MEMBERS_ONLY_FROM_RANK
    }

    /// `Some(id)` when the original row carries a reward set, `None` for the `0` sentinel.
    pub fn reward_set(&self) -> Option<i32> {
        if self.reward_set == 0 {
            None
        } else {
            Some(self.reward_set)
        }
    }
}

const fn row(rank: u32, xp_required: u32, item_cap: u32, reward_set: i32) -> FarmRankRow {
    FarmRankRow {
        rank,
        xp_required,
        item_cap,
        reward_set,
    }
}

/// ORIGINAL `FactoryRanks.txt`, FARMING housing instance 58 (player plot 3).
pub const FARM_RANKS_58: [FarmRankRow; 20] = [
    row(1, 0, 3, 0),
    row(2, 180, 4, 0),
    row(3, 540, 5, 4274),
    row(4, 990, 6, 0),
    row(5, 1530, 7, 4275),
    row(6, 2370, 8, 0),
    row(7, 3330, 9, 4276),
    row(8, 4410, 10, 4277),
    row(9, 5910, 12, 4278),
    row(10, 7710, 14, 4279),
    row(11, 10230, 16, 4280),
    row(12, 13110, 18, 4281),
    row(13, 16890, 21, 4282),
    row(14, 21930, 24, 4283),
    row(15, 28410, 27, 4284),
    row(16, 35700, 31, 4285),
    row(17, 45000, 35, 4286),
    row(18, 57600, 39, 4287),
    row(19, 72810, 44, 4288),
    row(20, 90000, 50, 4289),
];

/// ORIGINAL `FactoryRanks.txt`, FARMING housing instance 63 (player plot 5). The xp ladder and
/// item caps are identical to instance 58; only the reward-set ids differ.
pub const FARM_RANKS_63: [FarmRankRow; 20] = [
    row(1, 0, 3, 0),
    row(2, 180, 4, 0),
    row(3, 540, 5, 4381),
    row(4, 990, 6, 0),
    row(5, 1530, 7, 4382),
    row(6, 2370, 8, 0),
    row(7, 3330, 9, 4383),
    row(8, 4410, 10, 4384),
    row(9, 5910, 12, 4385),
    row(10, 7710, 14, 4386),
    row(11, 10230, 16, 4387),
    row(12, 13110, 18, 4388),
    row(13, 16890, 21, 4389),
    row(14, 21930, 24, 4390),
    row(15, 28410, 27, 4391),
    row(16, 35700, 31, 4392),
    row(17, 45000, 35, 4393),
    row(18, 57600, 39, 4394),
    row(19, 72810, 44, 4395),
    row(20, 90000, 50, 4396),
];

/// ORIGINAL `FactoryRanks.txt`, RANCHING housing instance 60 (player plot 7). Same xp ladder and
/// the same rank-5 members-only turn; its own item caps and no reward sets at all.
pub const RANCH_RANKS_60: [FarmRankRow; 20] = [
    row(1, 0, 1, 0),
    row(2, 180, 1, 0),
    row(3, 540, 1, 0),
    row(4, 990, 1, 0),
    row(5, 1530, 2, 0),
    row(6, 2370, 2, 0),
    row(7, 3330, 2, 0),
    row(8, 4410, 2, 0),
    row(9, 5910, 3, 0),
    row(10, 7710, 3, 0),
    row(11, 10230, 3, 0),
    row(12, 13110, 4, 0),
    row(13, 16890, 4, 0),
    row(14, 21930, 4, 0),
    row(15, 28410, 5, 0),
    row(16, 35700, 5, 0),
    row(17, 45000, 5, 0),
    row(18, 57600, 6, 0),
    row(19, 72810, 6, 0),
    row(20, 90000, 6, 0),
];

/// Rows for a recovered factory instance, or `None` for an instance without a rank table
/// (57, 59, 62, 64) and for every unknown id.
pub fn farm_ranks(instance_id: i32) -> Option<&'static [FarmRankRow; 20]> {
    match instance_id {
        58 => Some(&FARM_RANKS_58),
        63 => Some(&FARM_RANKS_63),
        60 => Some(&RANCH_RANKS_60),
        _ => None,
    }
}

/// Rank progress derived from a recovered ladder. `rank_experience` and `rank_max_experience`
/// mirror the recovered native fields `RankExperience` and `RankMaxExperience`; `rank_max_experience`
/// is `0` at the cap because no further threshold is shipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RankProgress {
    pub rank: u32,
    pub xp: u64,
    pub rank_experience: u64,
    pub rank_max_experience: u64,
    pub capped: bool,
}

impl RankProgress {
    /// INFERRED display percentage: the floored share of the current rank's span, `100` at cap.
    pub fn percent(&self) -> u32 {
        if self.capped || self.rank_max_experience == 0 {
            return 100;
        }
        ((self.rank_experience * 100) / self.rank_max_experience) as u32
    }
}

/// INFERRED reading of the original column: `EXPERIENCE_POINTS_REQUIRED` is the cumulative total
/// needed to hold that rank, so the first row whose requirement is `<= xp` wins.
pub fn rank_for_xp(rows: &[FarmRankRow; 20], xp: u64) -> u32 {
    let mut rank = 1;
    for entry in rows {
        if u64::from(entry.xp_required) <= xp {
            rank = entry.rank;
        } else {
            break;
        }
    }
    rank
}

pub fn progress(rows: &[FarmRankRow; 20], xp: u64) -> RankProgress {
    let rank = rank_for_xp(rows, xp);
    let index = (rank as usize).saturating_sub(1).min(rows.len() - 1);
    let floor = u64::from(rows[index].xp_required);
    let capped = rank >= MAX_RANK;
    let next = if capped {
        None
    } else {
        Some(u64::from(rows[(index + 1).min(rows.len() - 1)].xp_required))
    };
    RankProgress {
        rank,
        xp,
        rank_experience: xp.saturating_sub(floor),
        rank_max_experience: next.map(|next| next.saturating_sub(floor)).unwrap_or(0),
        capped,
    }
}

/// A progression track: a job profile (Miner 14) or a recovered farm/ranch factory instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProgressionTrack {
    Profile(i32),
    Farm(i32),
    Ranch(i32),
}

impl ProgressionTrack {
    /// The mining job profile. Mining minigame stars travel to the Miner job (wiki `Stars`).
    pub const MINER: Self = Self::Profile(MINER_PROFILE_ID);
}

/// The rank table behind a track, or the reason there is none.
#[derive(Clone, Copy, Debug)]
pub enum Curve {
    Recovered(&'static [FarmRankRow; 20]),
    Unrecovered(&'static str),
}

/// Why a track has no rank table. The gap is data, not an error inside a recovered table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressionGap {
    XpThresholdsUnrecovered(&'static str),
    UnknownFarmInstance(i32),
}

impl fmt::Display for ProgressionGap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XpThresholdsUnrecovered(job) => write!(
                formatter,
                "{job} per-rank xp thresholds are not in the preserved client data"
            ),
            Self::UnknownFarmInstance(id) => {
                write!(
                    formatter,
                    "factory instance {id} has no recovered rank table"
                )
            }
        }
    }
}

impl std::error::Error for ProgressionGap {}

pub fn curve(track: ProgressionTrack) -> Curve {
    match track {
        ProgressionTrack::Profile(id) if id == MINER_PROFILE_ID => {
            Curve::Unrecovered(MINER_XP_THRESHOLDS_GAP)
        }
        ProgressionTrack::Profile(_) => Curve::Unrecovered(JOB_XP_THRESHOLDS_GAP),
        ProgressionTrack::Farm(58) => Curve::Recovered(&FARM_RANKS_58),
        ProgressionTrack::Farm(63) => Curve::Recovered(&FARM_RANKS_63),
        ProgressionTrack::Ranch(60) => Curve::Recovered(&RANCH_RANKS_60),
        ProgressionTrack::Farm(_) | ProgressionTrack::Ranch(_) => {
            Curve::Unrecovered(FARM_INSTANCE_TABLE_GAP)
        }
    }
}

/// Rank for a track, or the explicit data gap that blocks it.
pub fn rank_of(track: ProgressionTrack, xp: u64) -> Result<RankProgress, ProgressionGap> {
    match curve(track) {
        Curve::Recovered(rows) => Ok(progress(rows, xp)),
        Curve::Unrecovered(_) => match track {
            ProgressionTrack::Profile(MINER_PROFILE_ID) => {
                Err(ProgressionGap::XpThresholdsUnrecovered("Miner"))
            }
            ProgressionTrack::Profile(_) => Err(ProgressionGap::XpThresholdsUnrecovered("job")),
            ProgressionTrack::Farm(id) | ProgressionTrack::Ranch(id) => {
                Err(ProgressionGap::UnknownFarmInstance(id))
            }
        },
    }
}

pub const MINER_XP_THRESHOLDS_GAP: &str = "Miner xp thresholds: the client ships no profile rank table; RankExperience and \
     RankMaxExperience arrive from the server";
pub const JOB_XP_THRESHOLDS_GAP: &str =
    "job xp thresholds: no profile rank table is shipped with the client";
pub const FARM_INSTANCE_TABLE_GAP: &str = "factory instance has no FactoryRanks.txt rows (57, 59, 62 and 64 are plot instances without \
     a rank table)";

// ---------------------------------------------------------------------------
// Miner facts.
// ---------------------------------------------------------------------------

pub const MINER_PROFILE_ID: i32 = 14;
/// ORIGINAL profile type of Miner 14 in the captured `Profiles.json` rows.
pub const MINER_PROFILE_TYPE: i32 = 9;
pub const MINER_NAME_STRING_ID: u32 = 2470;
pub const MINER_DESCRIPTION_STRING_ID: u32 = 2471;

/// One ORIGINAL vein gate. `required_rank` is `None` when the client ships no gate string; that is
/// the case for copper, the starting ore.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OreVeinGate {
    pub ore: &'static str,
    pub required_rank: Option<u32>,
    pub locale_id: Option<u32>,
}

/// ORIGINAL locale rows 1663..1666 plus the ungated copper ore.
pub const MINER_ORE_GATES: [OreVeinGate; 5] = [
    OreVeinGate {
        ore: "copper",
        required_rank: None,
        locale_id: None,
    },
    OreVeinGate {
        ore: "tin",
        required_rank: Some(4),
        locale_id: Some(1663),
    },
    OreVeinGate {
        ore: "iron",
        required_rank: Some(8),
        locale_id: Some(1664),
    },
    OreVeinGate {
        ore: "silver",
        required_rank: Some(12),
        locale_id: Some(1665),
    },
    OreVeinGate {
        ore: "gold",
        required_rank: Some(16),
        locale_id: Some(1666),
    },
];

pub fn ore_gate(ore: &str) -> Option<&'static OreVeinGate> {
    MINER_ORE_GATES.iter().find(|gate| gate.ore == ore)
}

/// ORIGINAL `Requirements.txt` rows 17..20 (`CheckRankLastProfileByType`, operator 6, profile
/// type 9) gate content on the Miner reaching these ranks.
pub const MINER_RANK_REQUIREMENT_GATES: [u32; 4] = [5, 9, 13, 17];

// ---------------------------------------------------------------------------
// Duplicate-grant guard for the mining (001) and harvesting (002) outcomes.
// ---------------------------------------------------------------------------

/// Where an authorised award came from. The source is part of the ledger key, so a mining and a
/// harvesting outcome that happen to share a numeric id stay distinct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GrantSource {
    Mining,
    Harvesting,
    Farm,
    Quest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GrantKey {
    pub track: ProgressionTrack,
    pub source: GrantSource,
    pub outcome_id: i64,
}

impl GrantKey {
    pub fn new(
        track: ProgressionTrack,
        source: GrantSource,
        outcome_id: i64,
    ) -> Result<Self, GrantError> {
        if outcome_id <= 0 {
            return Err(GrantError::InvalidOutcomeId(outcome_id));
        }
        Ok(Self {
            track,
            source,
            outcome_id,
        })
    }

    /// Work item 001: one key per finished `Type8Run::instance_id`. Mining stars belong to the
    /// Miner job, so the track is fixed to [`ProgressionTrack::MINER`].
    pub fn mining(instance_id: i32) -> Result<Self, GrantError> {
        Self::new(
            ProgressionTrack::MINER,
            GrantSource::Mining,
            i64::from(instance_id),
        )
    }

    /// Work item 002: one key per finished harvesting instance. The rewarded job is the
    /// minigame's required profile type, which is server content, so the caller names the track.
    pub fn harvesting(track: ProgressionTrack, instance_id: i32) -> Result<Self, GrantError> {
        Self::new(track, GrantSource::Harvesting, i64::from(instance_id))
    }

    pub fn farm(track: ProgressionTrack, outcome_id: i64) -> Result<Self, GrantError> {
        Self::new(track, GrantSource::Farm, outcome_id)
    }

    pub fn quest(track: ProgressionTrack, quest_id: i32) -> Result<Self, GrantError> {
        Self::new(track, GrantSource::Quest, i64::from(quest_id))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrantDecision {
    /// First time this outcome authorises an award; apply the xp exactly once.
    Granted,
    /// The outcome was already authorised; apply nothing.
    Duplicate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrantError {
    ZeroAmount,
    InvalidOutcomeId(i64),
}

impl fmt::Display for GrantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroAmount => formatter.write_str("an award must carry a positive xp amount"),
            Self::InvalidOutcomeId(id) => {
                write!(formatter, "outcome id {id} is not a positive instance id")
            }
        }
    }
}

impl std::error::Error for GrantError {}

/// Records which activity outcomes already authorised a job-xp award. Applying the same outcome
/// twice answers [`GrantDecision::Duplicate`] and changes nothing, which is what keeps the mining
/// (001) and harvesting (002) results from granting twice when a completion message is replayed.
#[derive(Debug, Default)]
pub struct GrantLedger {
    awarded: BTreeMap<GrantKey, u32>,
}

impl GrantLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn authorize(&mut self, key: GrantKey, amount: u32) -> Result<GrantDecision, GrantError> {
        if amount == 0 {
            return Err(GrantError::ZeroAmount);
        }
        if self.awarded.contains_key(&key) {
            return Ok(GrantDecision::Duplicate);
        }
        self.awarded.insert(key, amount);
        Ok(GrantDecision::Granted)
    }

    pub fn awarded(&self, key: GrantKey) -> Option<u32> {
        self.awarded.get(&key).copied()
    }

    /// Total authorised xp for one track, ready to hand to the profile store.
    pub fn total_awarded(&self, track: ProgressionTrack) -> u64 {
        self.awarded
            .iter()
            .filter(|(key, _)| key.track == track)
            .map(|(_, amount)| u64::from(*amount))
            .sum()
    }

    pub fn len(&self) -> usize {
        self.awarded.len()
    }

    pub fn is_empty(&self) -> bool {
        self.awarded.is_empty()
    }
}
