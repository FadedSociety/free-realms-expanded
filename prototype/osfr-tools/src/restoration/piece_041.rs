//! Work item 041: racer and derby progression (Kart Driver 48, Demo Derby Driver 49).
//!
//! The module carries the recovered job, rank and vehicle-unlock definitions for the two driving
//! jobs and links a *validated* race/derby outcome to them exactly once. It grants nothing else:
//! no coins, no items, no vehicle ownership transfer.
//!
//! ORIGINAL facts recovered from the pinned corpus (see
//! `notes/Research/Pieces/041 Racer and derby progression.md` for paths and hashes):
//! * Job identity: `PacketSendSelfToClient.json` → `ClientPcProfiles` rows with `JobGUID` 48
//!   ("Start of Kart Driver", name id 2917, description id 2918, badge 7494) and 49 ("Start of
//!   Demo Derby Driver", name id 2919, description id 2920, badge 7494); both names resolve in the
//!   supplied locale export. Community `Profiles.json` agrees on id, name id and description id.
//! * Lobby game types 1 (Kart Race) and 2 (Demo Derby) and the tunnel inner opcodes 33
//!   (`BaseVehicleRacePacket`) and 34 (`BaseCehicleDemolitionDerbyPacket`) from the unchanged
//!   community packet enum. The racing/derby captures carry exactly those streams.
//! * Rank model: the client Lua caps the job rank display at 20 (`if tonumber(rank) == 20 then
//!   exp = 100 end`), the captured character's two driving profiles sit at `JobLevel` 20, and the
//!   recovered gear catalogue gates items on ranks 1..20 for both jobs.
//! * Unlock definitions: `ItemClasses.txt` row 24 = "Kart Driver Gear" (profile name id 2917) and
//!   row 22 = "Demolition Derby Driver Gear" (profile name id 17401). Their catalogue rows carry
//!   `MinProfileRank` gates: 1,842 Kart Driver rows over ranks 1..20 and 1,647 derby rows over
//!   ranks 1..8 and 10..20 (no rank 9 row exists).
//! * Vehicles: item `Type` 11 with `CategoryId` 127 holds the 70-entry vehicle catalogue, and item
//!   classes 75 ("Racing Kart", part `Type` 9) and 76 ("Demo Derby Kart", part `Type` 10) hold the
//!   131-row part catalogues each (engine, turbo, steering, traction, mod A, mod B, decals,
//!   chassis). Rank gates there are 1 and 5 only.
//! * Ownership schema: the captured `VehicleStruct` carries six loadouts; each loadout's item id,
//!   name id, description id and model index match a catalogue row exactly (see
//!   `CAPTURED_VEHICLES`).
//! * Outcome link: `MiniGameStarsEarnedPacket` (inner opcode 21) carries a single byte, and the
//!   client increments its star counter from the minigame datasource `EarnedStars` column; the job
//!   rank itself arrives from the server (`JobLevelUp` call handler, `Profile_DS` rank/percent
//!   columns).
//!
//! RECONSTRUCTED (labelled, local policy - not historical behaviour):
//! * `DriverProgression` credits at most one award per outcome id for the job that owns the
//!   activity, records an unpriced finish (0 stars) without inventing a number, and refuses to
//!   advance once the profile rank is at the cap.
//! * `rank_cap` mirrors the community wiki's 20 / 4 split. The client only proves the 20 cap.
//!
//! NOT RECOVERED, deliberately absent:
//! * The star/XP threshold table that turns an award total into the next job rank. The thresholds
//!   were server-side; the captured corpus holds no such table, so the rank is supplied by the
//!   caller (the profile layer) and this module never derives one.
//! * Payout amounts for a race or derby win, vehicle price/ownership semantics beyond the captured
//!   rows, and any client-authoritative score.

use std::collections::BTreeMap;

/// Captured profile id of the Kart Driver job.
pub const KART_DRIVER_JOB: i32 = 48;
/// Captured profile id of the Demo Derby Driver job.
pub const DEMO_DERBY_DRIVER_JOB: i32 = 49;

pub const KART_DRIVER_NAME_ID: u32 = 2917;
pub const KART_DRIVER_DESCRIPTION_ID: u32 = 2918;
pub const DEMO_DERBY_DRIVER_NAME_ID: u32 = 2919;
pub const DEMO_DERBY_DRIVER_DESCRIPTION_ID: u32 = 2920;

/// Original client cap: the job window pins the rank bar to 100% at rank 20.
pub const MAX_JOB_RANK: u8 = 20;
/// COMMUNITY policy (wiki revision 43870), not proven by the captured client: most jobs stop at
/// rank 4 without a membership.
pub const NON_MEMBER_RANK_CAP: u8 = 4;

/// `ItemClasses.txt` row 24: "Kart Driver Gear" (profile name id 2917).
pub const KART_DRIVER_GEAR_CLASS: i32 = 24;
/// `ItemClasses.txt` row 22: "Demolition Derby Driver Gear" (profile name id 17401).
pub const DEMO_DERBY_DRIVER_GEAR_CLASS: i32 = 22;
pub const DEMO_DERBY_DRIVER_GEAR_PROFILE_NAME_ID: u32 = 17401;

/// `ItemClasses.txt` rows 75/76: the per-job kart part classes.
pub const RACING_KART_PART_CLASS: i32 = 75;
pub const DEMO_DERBY_KART_PART_CLASS: i32 = 76;
pub const RACING_KART_PART_CLASS_NAME_ID: u32 = 3722;
pub const DEMO_DERBY_KART_PART_CLASS_NAME_ID: u32 = 3723;
/// Item type carried by the racing-kart parts and by the derby-kart parts.
pub const RACING_KART_PART_TYPE: i32 = 9;
pub const DEMO_DERBY_KART_PART_TYPE: i32 = 10;
/// Item type and category of the vehicle catalogue rows themselves.
pub const VEHICLE_ITEM_TYPE: i32 = 11;
pub const VEHICLE_CATEGORY_ID: i32 = 127;

/// The largest star award one outcome can carry: `MiniGameStarsEarnedPacket` stores a single byte.
pub const MAX_STARS_PER_OUTCOME: u8 = u8::MAX;

/// One of the two original vehicle-race activities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DriverActivity {
    KartRace,
    DemoDerby,
}

impl DriverActivity {
    /// The job that owns this activity's progression. Recovered job ids only.
    pub const fn owning_job(self) -> i32 {
        match self {
            DriverActivity::KartRace => KART_DRIVER_JOB,
            DriverActivity::DemoDerby => DEMO_DERBY_DRIVER_JOB,
        }
    }

    /// Original lobby game type: Sanctuary's unchanged `LobbyGameDefinition` comment.
    pub const fn lobby_game_type(self) -> i32 {
        match self {
            DriverActivity::KartRace => 1,
            DriverActivity::DemoDerby => 2,
        }
    }

    /// Original tunnel inner opcode of the activity's packet family.
    pub const fn tunnel_inner_opcode(self) -> u16 {
        match self {
            DriverActivity::KartRace => 33,
            DriverActivity::DemoDerby => 34,
        }
    }
}

/// A rank-gated catalogue row (representative row per rank: the lowest item id at that rank).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RankUnlock {
    pub rank: u8,
    pub item_id: i32,
    pub name_id: u32,
    pub label: &'static str,
    /// Catalogue rows at this rank, including colour and race variants.
    pub catalogue_rows: u16,
}

/// Kart Driver gear rows, `ItemClasses.txt` class 24.
pub const KART_DRIVER_GEAR: [RankUnlock; 20] = [
    rank_unlock(1, 1780, 1618, "Steady Pace Jacket", 450),
    rank_unlock(2, 14097, 27957, "Amateur Kart Driver Shoes", 5),
    rank_unlock(3, 43222, 123200, "Great Velocity Jersey", 45),
    rank_unlock(4, 15035, 28869, "Amateur Kart Driver Pants", 5),
    rank_unlock(5, 1789, 1617, "Extreme Celerity Jacket", 104),
    rank_unlock(6, 76280, 133288, "Pit Stop Kart Driver Jacket", 4),
    rank_unlock(7, 14604, 28447, "Amateur Kart Driver Helmet", 1),
    rank_unlock(8, 14154, 28014, "Rookie Kart Driver Shoes", 50),
    rank_unlock(9, 13400, 27275, "Rookie Kart Driver Jacket", 2),
    rank_unlock(10, 1792, 1628, "High Velocity Jacket", 265),
    rank_unlock(11, 1791, 1629, "Intense Quickness Jacket", 64),
    rank_unlock(12, 15094, 28820, "Pro Kart Driver Pants", 5),
    rank_unlock(13, 13475, 27347, "Pro Kart Driver Jacket", 46),
    rank_unlock(14, 76296, 133304, "Switchback Kart Driver Jacket", 4),
    rank_unlock(15, 1800, 1632, "Top Speed Jacket", 60),
    rank_unlock(16, 15004, 28839, "Expert Kart Driver Pants", 5),
    rank_unlock(17, 13463, 27335, "Expert Kart Driver Jacket", 46),
    rank_unlock(18, 14788, 28625, "Expert Kart Driver Helmet", 5),
    rank_unlock(19, 1822, 1655, "Ultimate Celerity Jacket", 67),
    rank_unlock(20, 1783, 1617, "Extreme Celerity Jacket", 609),
];

/// Demo Derby Driver gear rows, `ItemClasses.txt` class 22. Rank 9 has no catalogue row.
pub const DEMO_DERBY_DRIVER_GEAR: [RankUnlock; 19] = [
    rank_unlock(1, 595, 1109, "Boom Bump Helmet", 586),
    rank_unlock(2, 14015, 27876, "Amateur Demo Derby Boots", 5),
    rank_unlock(3, 41696, 121680, "Crazy Crunch Shirt", 45),
    rank_unlock(4, 622, 69389, "Amateur Demo Derby Pants", 5),
    rank_unlock(5, 642, 28506, "Wonky Ram Helmet", 70),
    rank_unlock(6, 602, 69381, "Amateur Demo Derby Helmet", 5),
    rank_unlock(7, 14038, 27898, "Rookie Demo Derby Boots", 1),
    rank_unlock(8, 15157, 28990, "Rookie Demo Derby Pants", 50),
    rank_unlock(10, 685, 1129, "Bang Bash Helmet", 184),
    rank_unlock(11, 13541, 27412, "Wild Crash Jacket", 48),
    rank_unlock(12, 13969, 27830, "Pro Demo Derby Boots", 5),
    rank_unlock(13, 41856, 121840, "Whacking Crunch Jacket", 45),
    rank_unlock(14, 13546, 27417, "Pro Demo Derby Jacket", 6),
    rank_unlock(15, 780, 28504, "All-Star Demo Derby Helmet", 52),
    rank_unlock(16, 14619, 28462, "Expert Demo Derby Helmet", 5),
    rank_unlock(17, 13523, 27394, "Expert Demo Derby Jacket", 47),
    rank_unlock(18, 13962, 27823, "All-Star Demo Derby Boots", 6),
    rank_unlock(19, 13568, 27438, "Raging Ram Jacket", 47),
    rank_unlock(20, 599, 69379, "Elite Pink Wacky Crash Helmet", 435),
];

const fn rank_unlock(
    rank: u8,
    item_id: i32,
    name_id: u32,
    label: &'static str,
    catalogue_rows: u16,
) -> RankUnlock {
    RankUnlock {
        rank,
        item_id,
        name_id,
        label,
        catalogue_rows,
    }
}

/// A vehicle the supplied capture proves the character owned: a `VehicleStruct` loadout joined to
/// the catalogue row on item id, name id, description id and model index (`Param1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapturedVehicle {
    pub item_id: i32,
    pub name_id: u32,
    pub description_id: u32,
    pub model_index: u16,
    pub power_rating: i16,
    pub label: &'static str,
}

pub const CAPTURED_VEHICLES: [CapturedVehicle; 6] = [
    captured_vehicle(48316, 141872, 383250, 6, 1, "Roadster Racer"),
    captured_vehicle(48317, 141873, 383249, 9, 1, "Ripper Wrecker"),
    captured_vehicle(48318, 141874, 141875, 7, 2, "Cardie Racer"),
    captured_vehicle(61000, 7160, 7172, 45, 5, "Arrow Runner"),
    captured_vehicle(61006, 7166, 7178, 51, 5, "Splash Zipper"),
    captured_vehicle(56995, 39239, 39269, 40, 5, "Barbed Crusher"),
];

const fn captured_vehicle(
    item_id: i32,
    name_id: u32,
    description_id: u32,
    model_index: u16,
    power_rating: i16,
    label: &'static str,
) -> CapturedVehicle {
    CapturedVehicle {
        item_id,
        name_id,
        description_id,
        model_index,
        power_rating,
        label,
    }
}

/// COMMUNITY policy (wiki revision 43870): rank 20, or rank 4 for a non-member. The 4 cap is not
/// reproduced by the captured client and stays a labelled policy, never a recovered fact.
pub const fn rank_cap(member: bool) -> u8 {
    if member {
        MAX_JOB_RANK
    } else {
        NON_MEMBER_RANK_CAP
    }
}

/// A race or derby outcome that already passed its own session contract.
///
/// Build one only from a validated session result: the kart-racing contract's granted reward
/// outcome or the derby contract's granted reward outcome. Raw packets, scoreboards and arbitrary
/// client numbers must never be turned into one; `stars_earned` is the award the caller has
/// evidence for, or `0` when the payout is unknown (racing reward numbers are unrecovered).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidatedOutcome {
    pub activity: DriverActivity,
    /// Session-scoped outcome id from the owning contract. Used only for once-only credit.
    pub outcome_id: u64,
    pub stars_earned: u8,
}

impl ValidatedOutcome {
    pub const fn new(activity: DriverActivity, outcome_id: u64, stars_earned: u8) -> Self {
        Self {
            activity,
            outcome_id,
            stars_earned,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressionError {
    /// The profile id is not one of the two recovered driving jobs.
    NotADrivingJob(i32),
}

/// The single result of crediting an outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditOutcome {
    /// First sight of this outcome for this job.
    Credited { stars: u8, total_stars: u32 },
    /// This outcome id was already credited; nothing changed.
    Duplicate,
    /// The activity belongs to the other driving job; nothing changed.
    WrongJob { expected: i32, got: i32 },
    /// Credited once, but the profile rank is already at the cap.
    AtMaxRank { rank: u8, total_stars: u32 },
}

/// Per-job progression ledger. It records validated outcomes and star awards; it produces no
/// currency or item grant of any kind.
#[derive(Clone, Debug)]
pub struct DriverProgression {
    job: i32,
    total_stars: u32,
    credited: BTreeMap<u64, DriverActivity>,
}

impl DriverProgression {
    pub fn new(job: i32) -> Result<Self, ProgressionError> {
        match job {
            KART_DRIVER_JOB | DEMO_DERBY_DRIVER_JOB => Ok(Self {
                job,
                total_stars: 0,
                credited: BTreeMap::new(),
            }),
            other => Err(ProgressionError::NotADrivingJob(other)),
        }
    }

    pub const fn job(&self) -> i32 {
        self.job
    }

    pub const fn total_stars(&self) -> u32 {
        self.total_stars
    }

    pub fn credited_outcomes(&self) -> usize {
        self.credited.len()
    }

    /// Credit one validated outcome against `profile_rank`, which the caller owns because the
    /// star/XP threshold table was never recovered. Replays never move the total.
    pub fn credit(&mut self, outcome: ValidatedOutcome, profile_rank: u8) -> CreditOutcome {
        let expected = self.job;
        let got = outcome.activity.owning_job();
        if got != expected {
            return CreditOutcome::WrongJob { expected, got };
        }
        if self.credited.contains_key(&outcome.outcome_id) {
            return CreditOutcome::Duplicate;
        }
        self.credited.insert(outcome.outcome_id, outcome.activity);
        self.total_stars = self
            .total_stars
            .saturating_add(u32::from(outcome.stars_earned));
        if profile_rank >= MAX_JOB_RANK {
            return CreditOutcome::AtMaxRank {
                rank: MAX_JOB_RANK,
                total_stars: self.total_stars,
            };
        }
        CreditOutcome::Credited {
            stars: outcome.stars_earned,
            total_stars: self.total_stars,
        }
    }

    /// The recovered unlock rows this job knows about.
    pub fn unlock_table(&self) -> &'static [RankUnlock] {
        match self.job {
            KART_DRIVER_JOB => &KART_DRIVER_GEAR,
            _ => &DEMO_DERBY_DRIVER_GEAR,
        }
    }

    /// Every recovered unlock row whose gate is open at `profile_rank`.
    pub fn unlocked(&self, profile_rank: u8) -> Vec<&'static RankUnlock> {
        self.unlock_table()
            .iter()
            .filter(|row| row.rank <= profile_rank)
            .collect()
    }

    /// The rows that open exactly at `profile_rank` (what a level-up to that rank reveals).
    pub fn unlocks_at(&self, profile_rank: u8) -> Vec<&'static RankUnlock> {
        self.unlock_table()
            .iter()
            .filter(|row| row.rank == profile_rank)
            .collect()
    }
}

/// Every recovered definition of one driving job, for the catalogue/cross-check path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverJobDefinition {
    pub job: i32,
    pub name_id: u32,
    pub description_id: u32,
    pub activity: DriverActivity,
    pub gear_class: i32,
    pub part_class: i32,
    pub part_class_name_id: u32,
}

pub const DRIVER_JOBS: [DriverJobDefinition; 2] = [
    DriverJobDefinition {
        job: KART_DRIVER_JOB,
        name_id: KART_DRIVER_NAME_ID,
        description_id: KART_DRIVER_DESCRIPTION_ID,
        activity: DriverActivity::KartRace,
        gear_class: KART_DRIVER_GEAR_CLASS,
        part_class: RACING_KART_PART_CLASS,
        part_class_name_id: RACING_KART_PART_CLASS_NAME_ID,
    },
    DriverJobDefinition {
        job: DEMO_DERBY_DRIVER_JOB,
        name_id: DEMO_DERBY_DRIVER_NAME_ID,
        description_id: DEMO_DERBY_DRIVER_DESCRIPTION_ID,
        activity: DriverActivity::DemoDerby,
        gear_class: DEMO_DERBY_DRIVER_GEAR_CLASS,
        part_class: DEMO_DERBY_KART_PART_CLASS,
        part_class_name_id: DEMO_DERBY_KART_PART_CLASS_NAME_ID,
    },
];

pub fn driver_job(job: i32) -> Option<&'static DriverJobDefinition> {
    DRIVER_JOBS.iter().find(|row| row.job == job)
}

/// Structure checks over the recovered tables. The full corpus cross-check lives in
/// the private recovery checker `piece_041.py`; this keeps the constants self-consistent in-tree.
pub fn validate() -> Result<(), ProgressionError> {
    for job in DRIVER_JOBS {
        if job.activity.owning_job() != job.job {
            return Err(ProgressionError::NotADrivingJob(job.job));
        }
    }
    Ok(())
}

// ponytail: constants plus one ledger. No trait, no rank curve, no vehicle-purchase model - the
// threshold table and the purchase semantics are unrecovered, so the caller owns both.
