//! T662: the one Lakeshore/Greenwood/Sacred Grove zone quest the evidence calls live and the
//! server can run now, enabled in `data/content/quests/ninja.json`: Ninja "Achieve Level 10"
//! (`3110013`, Ty). Card Duelist "Cardie Party" (`3040014`) stays disabled on purpose
//! (`tests/tcg_and_brawler_npc_placements.rs` guards it: card duel results do not reach quests).
//!
//! Content only, no new wire and no Rust change. Every packet goes through
//! `Gameplay::handle_in_world` / `Gameplay::on_location`, the routers the gateway calls:
//! 26/8 -> 26/10 -> 26/6 offers (49/1, decoded with the production reader), 49/2 accepts,
//! Talk and position steps, the 26/6 that releases the held 49/13, and the 49/14 turn-in.
//! Client readers (unchanged): 26/3 `0x00a9ef10`; 49/13 `QuestEndPacket::ReadFields`
//! `0x00c7b990` (actor `6..14`, quest `14..18`, end text `18..22`, reward bundle
//! `0x008e7930`); 49/1 `0x00c7bb60`; AddNpc `0x00921a10`.
//!
//! Rewards are read from the shipped record, never pinned here (an (R) value the lead tunes
//! in data). The prerequisite is the shipped one: Achieve Level 10 follows `3110004`, the last
//! enabled Ninja step.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, active_job, assert_no_offer,
    assert_packet_changes_nothing, content_dir, counts, offer, pending_turnin, quest_end_reply,
    quest_reply, quest_status, realm_level, seed_active_profile, seed_quest,
};
use osfr_tools::{
    gameplay::Gameplay,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use quest_records::shipped_quest;
use std::collections::BTreeMap;

const TY: u64 = 9_000_000_223;
const BRY: u64 = 9_000_000_035;
const NINJA: i32 = 2;
const BRAWLER: i32 = 43;
const LEVEL_TEN: u32 = 3_110_013;
/// The shipped predecessor of the level gate: Silence is Golden... and Deadly.
const STEALTH: u32 = 3_110_004;
/// T542's Job Browser starter, also offered by Ty with no gate; every save here has it done so
/// the quests under test are the only offers left.
const BECOME_A_NINJA: u32 = 3_110_026;
/// The shipped Ninja entry line, done: The Ninja Path, Stolen Away, the stealth referral.
const ENTRY_LINE: [(u32, usize); 3] = [(3_110_001, 1), (3_110_002, 1), (STEALTH, 4)];
const MAP: ZoneQuests = ZoneQuests {
    default_file: "lakeshore.json",
    files: &[],
    runtime: &[],
};

/// The quest as the shared harness walks it, every value from the shipped record.
fn quest(id: u32, giver: u64, profile: i32) -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(id), giver, giver, profile)
}

/// A save for `active` with `xp` on that profile and the quests in `done` completed.
fn save_for(fixture: &Fixture, active: i32, xp: u32, done: &[(u32, usize)]) -> Gameplay {
    let mut state = fixture.game(None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER, NINJA, BRAWLER]);
    state
        .profiles
        .as_mut()
        .unwrap()
        .experience
        .insert(active, xp);
    for (id, goals) in done.iter().chain(&[(BECOME_A_NINJA, 1)]) {
        seed_quest(&mut state, *id, QuestStatus::Completed, &vec![1; *goals]);
    }
    fixture.game(Some(state))
}

#[test]
fn level_ten_gate_is_offered_climbs_with_the_rank_and_pays_from_data() {
    let gate = quest(LEVEL_TEN, TY, NINJA);
    assert!(
        gate.coins + gate.profile_xp > 0,
        "the end screen carries a reward"
    );
    let fixture = Fixture::new();
    // Rank 9 (800 XP on the 100-XP curve), the entry line done through the stealth referral.
    let mut game = save_for(&fixture, NINJA, 800, &ENTRY_LINE);
    let ty = MAP.at(CHARACTER, TY);
    let mut now = 1_000;

    MAP.accept(&mut game, CHARACTER, &gate, &mut now);
    assert_eq!(counts(&game, LEVEL_TEN), vec![0], "rank 9 is short of 10");

    // Short of rank 10 the giver changes nothing (no end screen, no payout).
    assert_no_offer(&mut game, &ty, TY, &mut now, "Ty before rank 10");
    assert_eq!(pending_turnin(&game), None);

    now += 10;
    realm_level(&mut game, &ty, 10, now);
    assert_eq!(
        counts(&game, LEVEL_TEN),
        vec![1],
        "rank 10 completes the goal"
    );
    assert_eq!(quest_status(&game, LEVEL_TEN), QuestStatus::Active);

    // Ty opens the turn-in and the data decides the payout (the rank-10 command's own XP is
    // already in the save; the helper checks only the gate's delta).
    now += 10;
    MAP.turn_in(&mut game, CHARACTER, &gate, NINJA, &mut now);

    // Paid once: a duplicate 49/14 changes nothing and Ty offers nothing more.
    assert_packet_changes_nothing(
        &mut game,
        &ty,
        &quest_end_reply(LEVEL_TEN),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &ty, TY, &mut now, "Ty after the gate");
}

#[test]
fn level_ten_gate_is_refused_to_the_wrong_job_and_the_wrong_point_in_the_line() {
    let fixture = Fixture::new();
    let ty = MAP.at(CHARACTER, TY);
    let mut now = 2_000;
    let forged = quest_reply(LEVEL_TEN, true);

    // A Brawler who finished the referral is not offered the Ninja gate.
    let mut brawler = save_for(&fixture, BRAWLER, 900, &[(STEALTH, 4)]);
    assert_no_offer(&mut brawler, &ty, TY, &mut now, "Brawler at Ty");
    assert_packet_changes_nothing(&mut brawler, &ty, &forged, now, "a forged 49/2");
    assert_eq!(active_job(&brawler), BRAWLER);

    // A Ninja one step earlier gets the stealth referral, not the gate, and cannot
    // forge the gate's accept.
    let mut early = save_for(&fixture, NINJA, 900, &ENTRY_LINE[..2]);
    offer(&mut early, &ty, TY, STEALTH, now + 10);
    assert_packet_changes_nothing(&mut early, &ty, &forged, now + 20, "a forged 49/2");
    assert_eq!(quest_status(&early, STEALTH), QuestStatus::Offered);
    assert!(
        !early
            .state()
            .npc_quests
            .unwrap()
            .quests
            .contains_key(&LEVEL_TEN)
    );
}

#[test]
fn level_gate_waits_for_the_finished_stealth_referral() {
    let eligibility = CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::from([("ninja".to_string(), 10)]),
        member: true,
        nonmember_quest_limit: 20,
        member_quest_limit: 20,
    };
    let fixture = Fixture::new();
    let engine = |done: &[(u32, usize)], open: Option<u32>| {
        let mut state = fixture.game(None).state();
        for (id, goals) in done {
            seed_quest(&mut state, *id, QuestStatus::Completed, &vec![1; *goals]);
        }
        if let Some(id) = open {
            seed_quest(&mut state, id, QuestStatus::Active, &[0; 4]);
        }
        QuestEngine::from_split_content(
            &content_dir(),
            CHARACTER,
            eligibility.clone(),
            state.npc_quests,
        )
        .expect("the shipped catalogue loads")
    };
    // The referral is open or untouched: the gate is not offerable. Finished: it is.
    assert!(!engine(&ENTRY_LINE[..2], Some(STEALTH)).can_offer(LEVEL_TEN, TY));
    assert!(!engine(&ENTRY_LINE[..2], None).can_offer(LEVEL_TEN, TY));
    assert!(engine(&ENTRY_LINE, None).can_offer(LEVEL_TEN, TY));
    // Only its giver offers it.
    assert!(!engine(&ENTRY_LINE, None).can_offer(LEVEL_TEN, BRY));
}
