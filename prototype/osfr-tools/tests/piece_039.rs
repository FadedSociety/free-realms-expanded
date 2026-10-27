//! Work item 039 acceptance: Miner and Farmer (farm) gathering-job progression.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/working/OSFR.Client/OSFR Client/Resources/FactoryRanks.txt` — the original client rank
//!   table. Columns `HOUSING_INSTANCE_ID^EXPERIENCE_POINTS_REQUIRED^NAME_STRING_ID^RANK^
//!   GAIN_RANK_COMPOSITE_EFFECT^FACTORY_TYPE^YIELD_MULTIPLIER^REWARD_SET_ID^ITEM_CAP_MODIFIER^
//!   MEMBERS_ONLY`. FARMING instances 58 and 63 and RANCHING instance 60 carry twenty ranks each
//!   with the same xp ladder.
//! * `lab/working/OSFR.Client/OSFR Client/Resources/FactoryPlots.txt` — plot 3 uses FARMING
//!   instance 58, plot 5 uses FARMING instance 63 and plot 7 uses RANCHING instance 60; instances
//!   59/62/64 have no rank rows.
//! * `en_us_data.dat` locale rows 1663..1666 — the verbatim vein gate strings: tin 4, iron 8,
//!   silver 12, gold 16. Copper carries no gate string.
//! * `lab/working/OSFR.Client/OSFR Client/Resources/Requirements.txt` — `CheckRankLastProfileByType`
//!   rows 17..20 gate on profile type 9 (Miner) at ranks 5, 9, 13 and 17.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` `Profile_DS` — rank columns 4 `Rank`,
//!   5 `StarsAvailable`, 11 `StarsEarned`, 12 `RankPercent`, the fields the farm/farm-owner
//!   progression travels in.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` — `ClientUpdatePacketUpdateProfileRank`
//!   (`FUN_0089e350`, length 0x12, two fields) and the `RankExperience` / `RankMaxExperience`
//!   tooltip fields behind `FUN_00a75d50`; `lab/decompiled/server/Gateway/Gateway.Login/
//!   ClientPcData.cs` carries `JobLevel`, `JobLevelXP` and `JobLevelTotalXP`.
//!
//! The Miner xp *thresholds* are an explicit gap: the client ships no profile rank table (it
//! loads `FactoryRanks.txt` and reads `RankExperience`/`RankMaxExperience` from the server), so
//! only the farm/ranch ladder is recoverable. the private recovery checker `piece_039.py` records the same gap.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_039;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_039.rs"]
#[allow(dead_code)]
mod piece_039;

use piece_039::{
    Curve, FARM_INSTANCE_IDS, FARM_MEMBERS_ONLY_FROM_RANK, FARM_NAME_STRING_ID, FARM_RANKS_58,
    FARM_RANKS_63, FARM_YIELD_MULTIPLIER, FarmRankRow, GAIN_RANK_COMPOSITE_EFFECT, GrantDecision,
    GrantError, GrantKey, GrantLedger, GrantSource, MAX_RANK, MINER_DESCRIPTION_STRING_ID,
    MINER_NAME_STRING_ID, MINER_ORE_GATES, MINER_PROFILE_ID, MINER_PROFILE_TYPE,
    MINER_RANK_REQUIREMENT_GATES, NON_MEMBER_RANK_CAP, OreVeinGate, ProgressionGap,
    ProgressionTrack, RANCH_INSTANCE_ID, RANCH_RANKS_60, curve, farm_ranks, ore_gate, progress,
    rank_for_xp, rank_of,
};

const FARM_XP_REQUIRED: [u32; 20] = [
    0, 180, 540, 990, 1530, 2370, 3330, 4410, 5910, 7710, 10230, 13110, 16890, 21930, 28410, 35700,
    45000, 57600, 72810, 90000,
];
const FARM_ITEM_CAP: [u32; 20] = [
    3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 18, 21, 24, 27, 31, 35, 39, 44, 50,
];
const RANCH_ITEM_CAP: [u32; 20] = [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6];

fn xp_ladder(rows: &[FarmRankRow; 20]) -> Vec<u32> {
    rows.iter().map(|row| row.xp_required).collect()
}

fn caps(rows: &[FarmRankRow; 20]) -> Vec<u32> {
    rows.iter().map(|row| row.item_cap).collect()
}

fn reward_sets(rows: &[FarmRankRow; 20]) -> Vec<i32> {
    rows.iter().map(|row| row.reward_set).collect()
}

#[test]
fn farm_ladder_is_the_recovered_twenty_rank_table() {
    assert_eq!(MAX_RANK, 20);
    assert_eq!(FARM_INSTANCE_IDS, [58, 63]);
    assert_eq!(RANCH_INSTANCE_ID, 60);
    assert_eq!(GAIN_RANK_COMPOSITE_EFFECT, 15263);
    assert_eq!(FARM_NAME_STRING_ID, 0);
    assert_eq!(FARM_YIELD_MULTIPLIER, 0.0);
    assert_eq!(FARM_MEMBERS_ONLY_FROM_RANK, 5);
    assert_eq!(NON_MEMBER_RANK_CAP, 4);
    assert_eq!(xp_ladder(&FARM_RANKS_58), FARM_XP_REQUIRED);
    assert_eq!(xp_ladder(&FARM_RANKS_63), FARM_XP_REQUIRED);
    assert_eq!(xp_ladder(&RANCH_RANKS_60), FARM_XP_REQUIRED);
    for (index, entry) in FARM_RANKS_58.iter().enumerate() {
        assert_eq!(entry.rank, index as u32 + 1);
    }
    assert!(FARM_XP_REQUIRED.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn both_farm_instances_share_the_ladder_and_differ_in_reward_sets() {
    assert_eq!(caps(&FARM_RANKS_58), FARM_ITEM_CAP);
    assert_eq!(caps(&FARM_RANKS_63), FARM_ITEM_CAP);
    assert!(
        FARM_RANKS_58
            .iter()
            .zip(FARM_RANKS_63.iter())
            .all(|(left, right)| left.xp_required == right.xp_required
                && left.item_cap == right.item_cap
                && left.rank == right.rank)
    );
    assert_ne!(reward_sets(&FARM_RANKS_58), reward_sets(&FARM_RANKS_63));
    assert_eq!(
        FARM_RANKS_58
            .iter()
            .filter(|row| row.reward_set != 0)
            .count(),
        16
    );
}

#[test]
fn ranch_rows_reuse_the_ladder_with_their_own_item_caps() {
    assert_eq!(caps(&RANCH_RANKS_60), RANCH_ITEM_CAP);
    assert!(RANCH_RANKS_60.iter().all(|row| row.reward_set == 0));
}

#[test]
fn members_only_starts_at_the_recovered_rank_five() {
    for instance_id in FARM_INSTANCE_IDS {
        let rows = farm_ranks(instance_id).expect("farm ranks");
        for entry in rows {
            assert_eq!(
                entry.members_only(),
                entry.rank >= FARM_MEMBERS_ONLY_FROM_RANK
            );
        }
    }
    let ranch = farm_ranks(RANCH_INSTANCE_ID).expect("ranch ranks");
    assert!(ranch[3].members_only() == false);
    assert!(ranch[4].members_only());
}

#[test]
fn reward_sets_are_absent_below_rank_three_and_per_instance() {
    assert_eq!(FARM_RANKS_58[0].reward_set, 0);
    assert_eq!(FARM_RANKS_58[1].reward_set, 0);
    assert_eq!(FARM_RANKS_58[2].reward_set, 4274);
    assert_eq!(FARM_RANKS_58[3].reward_set, 0);
    assert_eq!(FARM_RANKS_58[5].reward_set, 0);
    assert_eq!(FARM_RANKS_58[6].reward_set, 4276);
    assert_eq!(FARM_RANKS_58[19].reward_set, 4289);
    assert_eq!(FARM_RANKS_63[2].reward_set, 4381);
    assert_eq!(FARM_RANKS_63[5].reward_set, 0);
    assert_eq!(FARM_RANKS_63[6].reward_set, 4383);
    assert_eq!(FARM_RANKS_63[19].reward_set, 4396);
}

#[test]
fn farm_lookup_rejects_instances_without_a_rank_table() {
    assert!(farm_ranks(58).is_some());
    assert!(farm_ranks(63).is_some());
    assert!(farm_ranks(60).is_some());
    for missing in [57, 59, 62, 64] {
        assert!(farm_ranks(missing).is_none(), "instance {missing}");
    }
}

#[test]
fn rank_boundaries_follow_the_cumulative_table() {
    assert_eq!(rank_for_xp(&FARM_RANKS_58, 0), 1);
    assert_eq!(rank_for_xp(&FARM_RANKS_58, 179), 1);
    assert_eq!(rank_for_xp(&FARM_RANKS_58, 180), 2);
    assert_eq!(rank_for_xp(&FARM_RANKS_58, 539), 2);
    assert_eq!(rank_for_xp(&FARM_RANKS_58, 540), 3);
    assert_eq!(rank_for_xp(&FARM_RANKS_58, 89_999), 19);
    assert_eq!(rank_for_xp(&FARM_RANKS_58, 90_000), 20);
    assert_eq!(rank_for_xp(&FARM_RANKS_58, u64::MAX), 20);
}

#[test]
fn progress_reports_rank_experience_and_max() {
    let at_rank_three = progress(&FARM_RANKS_58, 700);
    assert_eq!(at_rank_three.rank, 3);
    assert_eq!(at_rank_three.xp, 700);
    assert_eq!(at_rank_three.rank_experience, 160);
    assert_eq!(at_rank_three.rank_max_experience, 450);
    assert_eq!(at_rank_three.percent(), 35);
    assert!(!at_rank_three.capped);

    let fresh = progress(&FARM_RANKS_58, 0);
    assert_eq!(fresh.rank, 1);
    assert_eq!(fresh.rank_experience, 0);
    assert_eq!(fresh.rank_max_experience, 180);
    assert_eq!(fresh.percent(), 0);
}

#[test]
fn progress_is_capped_at_rank_twenty() {
    let capped = progress(&FARM_RANKS_58, 90_000);
    assert_eq!(capped.rank, MAX_RANK);
    assert_eq!(capped.rank_experience, 0);
    assert_eq!(capped.rank_max_experience, 0);
    assert_eq!(capped.percent(), 100);
    assert!(capped.capped);

    let beyond = progress(&RANCH_RANKS_60, 100_000);
    assert_eq!(beyond.rank, MAX_RANK);
    assert_eq!(beyond.rank_experience, 10_000);
    assert!(beyond.capped);
}

#[test]
fn miner_vein_gates_are_the_recovered_locale_values() {
    assert_eq!(MINER_PROFILE_ID, 14);
    assert_eq!(MINER_PROFILE_TYPE, 9);
    assert_eq!(MINER_NAME_STRING_ID, 2470);
    assert_eq!(MINER_DESCRIPTION_STRING_ID, 2471);
    assert_eq!(
        MINER_ORE_GATES.map(|gate| (gate.ore, gate.required_rank)),
        [
            ("copper", None),
            ("tin", Some(4)),
            ("iron", Some(8)),
            ("silver", Some(12)),
            ("gold", Some(16)),
        ]
    );
    assert_eq!(
        MINER_ORE_GATES.map(|gate| gate.locale_id),
        [None, Some(1663), Some(1664), Some(1665), Some(1666)]
    );
    let OreVeinGate {
        ore,
        required_rank,
        locale_id,
    } = *ore_gate("gold").expect("gold gate");
    assert_eq!(
        (ore, required_rank, locale_id),
        ("gold", Some(16), Some(1666))
    );
    assert!(ore_gate("mithril").is_none());
}

#[test]
fn miner_rank_requirement_gates_are_monotonic() {
    assert_eq!(MINER_RANK_REQUIREMENT_GATES, [5, 9, 13, 17]);
    assert!(
        MINER_RANK_REQUIREMENT_GATES
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    assert!(
        MINER_RANK_REQUIREMENT_GATES
            .iter()
            .all(|rank| *rank <= MAX_RANK)
    );
}

#[test]
fn miner_xp_curve_is_an_explicit_gap() {
    assert!(matches!(
        curve(ProgressionTrack::Profile(MINER_PROFILE_ID)),
        Curve::Unrecovered(_)
    ));
    assert_eq!(
        rank_of(ProgressionTrack::Profile(MINER_PROFILE_ID), 50_000),
        Err(ProgressionGap::XpThresholdsUnrecovered("Miner"))
    );
    assert!(matches!(
        curve(ProgressionTrack::Farm(58)),
        Curve::Recovered(_)
    ));
    assert_eq!(
        rank_of(ProgressionTrack::Farm(59), 0),
        Err(ProgressionGap::UnknownFarmInstance(59))
    );
    assert_eq!(
        rank_of(ProgressionTrack::Ranch(RANCH_INSTANCE_ID), 100_000)
            .expect("ranch rank")
            .rank,
        MAX_RANK
    );
}

#[test]
fn ledger_authorizes_a_mining_outcome_once() {
    let mut ledger = GrantLedger::new();
    let key = GrantKey::mining(18_000_001).expect("mining key");
    assert_eq!(key.track, ProgressionTrack::Profile(MINER_PROFILE_ID));
    assert_eq!(key.source, GrantSource::Mining);
    assert_eq!(ledger.authorize(key, 30), Ok(GrantDecision::Granted));
    assert_eq!(ledger.awarded(key), Some(30));
    assert_eq!(ledger.authorize(key, 30), Ok(GrantDecision::Duplicate));
    assert_eq!(ledger.awarded(key), Some(30));
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.total_awarded(key.track), 30);
}

#[test]
fn ledger_keys_mining_and_harvesting_separately() {
    let mut ledger = GrantLedger::new();
    let mining = GrantKey::mining(7).expect("mining key");
    let harvesting =
        GrantKey::harvesting(ProgressionTrack::Profile(45), 7).expect("harvesting key");
    assert_eq!(mining.source, GrantSource::Mining);
    assert_eq!(harvesting.source, GrantSource::Harvesting);
    // The harvesting minigame rewards the job named by its required profile type, which is
    // server content; the caller supplies that track.
    assert_eq!(harvesting.track, ProgressionTrack::Profile(45));
    assert_ne!(mining, harvesting);
    assert_eq!(ledger.authorize(mining, 10), Ok(GrantDecision::Granted));
    assert_eq!(ledger.authorize(harvesting, 4), Ok(GrantDecision::Granted));
    assert_eq!(ledger.len(), 2);
}

#[test]
fn ledger_totals_xp_per_track_across_outcomes() {
    let mut ledger = GrantLedger::new();
    let farm = ProgressionTrack::Farm(58);
    for outcome in 1..=3 {
        let key = GrantKey::farm(farm, outcome).expect("farm key");
        assert_eq!(ledger.authorize(key, 15), Ok(GrantDecision::Granted));
    }
    assert_eq!(ledger.total_awarded(farm), 45);
    assert_eq!(
        rank_of(farm, ledger.total_awarded(farm))
            .expect("farm rank")
            .rank,
        1
    );
    assert_eq!(
        ledger.total_awarded(ProgressionTrack::Profile(MINER_PROFILE_ID)),
        0
    );
}

#[test]
fn ledger_rejects_zero_amounts_and_bad_outcome_ids() {
    let mut ledger = GrantLedger::new();
    let key = GrantKey::mining(18_000_001).expect("mining key");
    assert_eq!(ledger.authorize(key, 0), Err(GrantError::ZeroAmount));
    assert!(ledger.is_empty());
    assert_eq!(GrantKey::mining(0), Err(GrantError::InvalidOutcomeId(0)));
    assert_eq!(
        GrantKey::harvesting(ProgressionTrack::Profile(45), -4),
        Err(GrantError::InvalidOutcomeId(-4))
    );
    assert!(GrantKey::quest(ProgressionTrack::Profile(MINER_PROFILE_ID), 12).is_ok());
}
