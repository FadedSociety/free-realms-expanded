//! T669: zone quests round 2, Briarwood and Wugachug. Of the thirteen quests in
//! `lab/references/quest-lines/wiring/zone-quests-matched/{briarwood,wugachug}.json`, three run end
//! to end on this tip and are enabled (`notes/Autopilot/T669.md` names the blocker of the other ten):
//!
//! * Brawler: Finding a Buyer (`3030024`): Petra (Blackspore) offers it, Andrew (Briarwood) is the
//!   one Talk and the turn-in;
//! * Miner: Growing Up So Quickly (`3100024`): Silverray (Blackspore) offers it, the player visits
//!   Pinkelug's spot at the Mother Lode and talks to her;
//! * Chef: The Food Reviewer (`3050026`): Cook Yumtumlot (Wugachug) offers it, Lucca De'Flor
//!   (Snowhill) is the one Talk and the turn-in.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`: the 26/8 -> 26/10 ->
//! 26/6 offer (49/1, decoded with the production reader), the 49/2 accept, the location event
//! (`Gameplay::on_location`) for the visit, the Talk that holds the 49/13, the 26/6 that releases
//! it and the 49/14 turn-in. Gate, card ids, turn-in and payout are read from the shipped records
//! through `tests/support/quest_records.rs`, so a retune in data needs no edit here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`, 49/13
//! `QuestEndPacket::ReadFields` `0x00c7b990`, 49/14 `0x00a864c0`; AddNpc `0x00921a10`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;
#[path = "support/quest_target_sets.rs"]
mod quest_target_sets;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, active_job, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, profile_xp, quest_end_reply,
    quest_reply, quest_status, reference_content, saved, seed_active_profile, seed_quest,
};
use osfr_tools::{
    gameplay::Gameplay,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use quest_records::shipped_quest;
use quest_target_sets::Targets;

const PETRA: u64 = 9_000_000_428;
const ANDREW: u64 = 9_000_000_009;
const SILVERRAY: u64 = 9_000_000_203;
const PINKELUG: u64 = 9_000_000_169;
const YUMTUMLOT: u64 = 9_000_000_061;
const LUCCA: u64 = 9_000_000_133;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
const CHEF: i32 = 45;
const OTHER: u64 = CHARACTER + 1;

/// Petra and Silverray live in Blackspore, Cook Yumtumlot in Wugachug and Lucca in Snowhill;
/// Andrew and Pinkelug are the default Briarwood file. Each is its own actor (a Legacy row, or an
/// extra spawn keyed to its row).
const MAP: ZoneQuests = ZoneQuests {
    default_file: "briarwood.json",
    files: &[
        (PETRA, "blackspore_swamp.json"),
        (SILVERRAY, "blackspore_swamp.json"),
        (YUMTUMLOT, "wugachug.json"),
        (LUCCA, "snowhill.json"),
    ],
    runtime: &[],
};

/// One enabled quest: its record, who offers it, the NPC its last Talk ends at and its job.
#[derive(Clone, Copy)]
struct Case {
    quest: u32,
    giver: u64,
    turnin: u64,
    job: i32,
}

const BUYER: Case = Case {
    quest: 3_030_024,
    giver: PETRA,
    turnin: ANDREW,
    job: BRAWLER,
};
const GROWING_UP: Case = Case {
    quest: 3_100_024,
    giver: SILVERRAY,
    turnin: PINKELUG,
    job: MINER,
};
const REVIEWER: Case = Case {
    quest: 3_050_026,
    giver: YUMTUMLOT,
    turnin: LUCCA,
    job: CHEF,
};
const CASES: [Case; 3] = [BUYER, GROWING_UP, REVIEWER];

impl Case {
    /// The quest as the shared harness walks it, every value from the shipped record.
    fn quest(self) -> ZoneQuest {
        ZoneQuest::from_record(shipped_quest(self.quest), self.giver, self.turnin, self.job)
    }

    /// The authored band gate: the job rank the quest waits for.
    fn gate(self) -> u32 {
        let rank = shipped_quest(self.quest)
            .min_level
            .expect("the quest keeps a band gate");
        assert!(
            rank > 1,
            "a gate of rank 1 would make the rank-short case empty"
        );
        rank
    }
}

/// A player with `active` selected and `case`'s job learned at `rank` (`Profiles::rank` is
/// `1 + xp / 100`), the quest's prerequisites done (Petra and Silverray offer their rank goal
/// before the follow-up; `zone_quests_blackspore` walks that order).
fn player(fixture: &Fixture, guid: u64, active: i32, case: Case, rank: u32) -> Gameplay {
    let job = case.job;
    let mut state = fixture.game_as(guid, None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER, job]);
    for &done in &shipped_quest(case.quest).prerequisite_quest_ids {
        let goals = vec![1; shipped_quest(done).objectives.len()];
        seed_quest(&mut state, done, QuestStatus::Completed, &goals);
    }
    state
        .profiles
        .as_mut()
        .expect("the fixture has profile content")
        .experience
        .insert(job, (rank - 1) * 100);
    fixture.game_as(guid, Some(state))
}

/// A player ready for `case`: its job active at the gate.
fn ready(fixture: &Fixture, guid: u64, case: Case) -> Gameplay {
    player(fixture, guid, case.job, case, case.gate())
}

/// The location event that completes Growing Up's visit: standing at Pinkelug's spot.
fn visit_pinkelug(game: &mut Gameplay, guid: u64, now: u64) -> bool {
    game.on_location(&MAP.at(guid, PINKELUG), now)
        .unwrap()
        .changed
}

/// Offer, accept and walk everything but the final Talk.
fn walk(game: &mut Gameplay, guid: u64, case: Case, now: &mut u64) {
    MAP.accept(game, guid, &case.quest(), now);
    if case.quest == GROWING_UP.quest {
        assert!(
            visit_pinkelug(game, guid, *now),
            "the visit at the Mother Lode"
        );
        *now += 10;
    }
}

#[test]
fn each_quest_is_offered_walked_and_paid_once() {
    for case in CASES {
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = ready(&fixture, CHARACTER, case);
        let quest = case.quest();
        let (coins_before, xp_before) = (coins(&game), profile_xp(&game, case.job));
        let mut now = 1_000;
        assert!(
            quest.coins > 0 && quest.profile_xp > 0,
            "the record pays something"
        );

        walk(&mut game, CHARACTER, case, &mut now);
        MAP.turn_in(&mut game, CHARACTER, &quest, case.job, &mut now);

        assert_eq!(quest_status(&game, case.quest), QuestStatus::Completed);
        assert_eq!(coins(&game), coins_before + quest.coins);
        assert_eq!(profile_xp(&game, case.job), xp_before + quest.profile_xp);

        // Done is done: the giver offers nothing again (Andrew, the Buyer's turn-in, now offers
        // IOU, the next quest of the band) and a repeated 49/14 pays nothing.
        let paid = coins(&game);
        assert_no_offer(
            &mut game,
            &MAP.at(CHARACTER, case.giver),
            MAP.actor(case.giver),
            &mut now,
            "the giver after the quest",
        );
        assert_packet_changes_nothing(
            &mut game,
            &MAP.at(CHARACTER, case.turnin),
            &quest_end_reply(case.quest),
            now,
            "a duplicate 49/14",
        );
        assert_eq!(coins(&game), paid);
    }
}

#[test]
fn the_gates_hold_for_rank_and_job() {
    for case in CASES {
        let fixture = Fixture::minimal(CHARACTER);
        let giver = MAP.at(CHARACTER, case.giver);
        let mut now = 1_000;

        // One rank short: the giver offers nothing and a forged 49/2 changes nothing.
        let mut short = player(&fixture, CHARACTER, case.job, case, case.gate() - 1);
        assert_no_offer(
            &mut short,
            &giver,
            MAP.actor(case.giver),
            &mut now,
            "rank short",
        );
        assert_packet_changes_nothing(
            &mut short,
            &giver,
            &quest_reply(case.quest, true),
            now,
            "a forged 49/2 below the gate",
        );
        assert!(
            !short
                .state()
                .npc_quests
                .as_ref()
                .unwrap()
                .quests
                .contains_key(&case.quest)
        );

        // At the gate with another job active: job-locked, not only level-locked.
        let mut other = player(&fixture, CHARACTER, ADVENTURER, case, case.gate());
        assert_ne!(active_job(&other), case.job);
        assert_no_offer(
            &mut other,
            &giver,
            MAP.actor(case.giver),
            &mut now,
            "another job active",
        );

        // The same Talk offers it once the gate is met.
        let mut open = ready(&fixture, CHARACTER, case);
        MAP.accept(&mut open, CHARACTER, &case.quest(), &mut now);
    }
}

#[test]
fn forged_early_or_out_of_order_requests_change_nothing() {
    for case in CASES {
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = ready(&fixture, CHARACTER, case);
        let mut now = 1_000;
        let turnin = MAP.at(CHARACTER, case.turnin);

        // Before the quest is accepted the hand-in NPC credits nothing.
        assert_packet_changes_nothing(
            &mut game,
            &turnin,
            &quest_end_reply(case.quest),
            now,
            "a 49/14 before the quest is accepted",
        );
        assert_packet_changes_nothing(
            &mut game,
            &turnin,
            &quest_reply(case.quest, true),
            now,
            "a forged 49/2 with nothing offered",
        );

        // Accepted, nothing done: an early 49/14 pays nothing. Growing Up's Talk before the
        // visit is the wrong order and moves no count.
        MAP.accept(&mut game, CHARACTER, &case.quest(), &mut now);
        assert_packet_changes_nothing(
            &mut game,
            &turnin,
            &quest_end_reply(case.quest),
            now,
            "an early 49/14",
        );
        if case.quest == GROWING_UP.quest {
            assert_no_offer(
                &mut game,
                &turnin,
                MAP.actor(PINKELUG),
                &mut now,
                "Pinkelug before the visit",
            );
            // A visit far from the Mother Lode (Silverray's camp) is not the visit.
            assert!(
                !game
                    .on_location(&MAP.at(CHARACTER, SILVERRAY), now + 1)
                    .unwrap()
                    .changed
            );
            assert_eq!(counts(&game, case.quest), [0, 0]);
        }
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Active);
    }
}

#[test]
fn progress_survives_a_reconnect_and_players_are_independent() {
    for case in CASES {
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = ready(&fixture, CHARACTER, case);
        let mut other = ready(&fixture, OTHER, case);
        let quest = case.quest();
        let mut now = 1_000;
        walk(&mut game, CHARACTER, case, &mut now);

        // A second player shares nothing with the first one's progress.
        MAP.accept(&mut other, OTHER, &quest, &mut now);
        assert!(counts(&other, case.quest).iter().all(|&count| count == 0));

        let mut game = fixture.reload_serialized(&game);
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Active);
        let other_before = saved(&other);
        MAP.turn_in(&mut game, CHARACTER, &quest, case.job, &mut now);
        assert_eq!(saved(&other), other_before, "A's turn-in changed B");

        // The held hand-in survives a reload too: the 49/14 pays after it.
        let game = fixture.reload_serialized(&game);
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Completed);
        assert_eq!(quest_status(&other, case.quest), QuestStatus::Active);
    }
}

#[test]
fn the_loader_binds_each_giver_and_turn_in() {
    let engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility::default(),
        None,
    )
    .expect("the shipped split catalogue loads");
    for case in CASES {
        assert!(engine.is_enabled(case.quest), "quest {}", case.quest);
        assert_eq!(engine.giver_npc_id(case.quest), Some(case.giver));
        // The formal turn-in drives the quest marker and the menu's turn-in row.
        assert_eq!(engine.turnin_npc_id(case.quest), Some(case.turnin));
    }
}

/// The ten quests of the two wiring lists that stay off, and the one blocker the server can
/// check: `Some(item)` is a gather target no server mining or harvesting game yields. The rest
/// are in `notes/Autopilot/T669.md`: a giver nobody draws, a client-reported step, an item the
/// catalogue lacks, a recipe without an id.
#[test]
fn the_other_ten_stay_off_until_their_blocker_lifts() {
    const OFF: [(u32, Option<u32>); 10] = [
        (3_000_043, None),
        (3_000_044, None),
        (3_050_025, None),
        (3_070_019, None),
        (3_090_019, None),
        (3_090_020, Some(77_549)),
        (3_090_023, None),
        (3_090_032, None),
        (3_100_025, Some(2_743)),
        (3_100_027, Some(2_747)),
    ];
    let targets = Targets::load(&content_dir(), &reference_content()).unwrap();
    for (quest, gather) in OFF {
        assert!(
            !shipped_quest(quest).enabled,
            "quest {quest} is enabled, not blocked"
        );
        if let Some(item) = gather {
            assert!(
                !targets.gather.contains(&item),
                "gather {item} is now produced: quest {quest} may be wireable"
            );
        }
    }
}
