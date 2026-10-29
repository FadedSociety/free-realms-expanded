//! T660: the Blackspore job rank gates the lab's fixed objective specs let the server run
//! today, enabled in `data/content/quests/{blacksmith,miner,brawler}.json`: Silverray's
//! "Blacksmith: Reach Level 20" (3020024) and "Miner: Reach Level 20" (3100023, live "unsure",
//! so (R)), and Petra's "Brawler: Achieve Level 5" (3030021, a rank gate then a Talk back at
//! her). T668 adds Chloe's "Ninja: Achieve Level 5" (3110007, behind the stealth referral 3110004).
//! The other Blackspore rows stay disabled (notes/Autopilot/T660.md and T668.md list each blocker).
//!
//! Every test drives the production in-world router through `tests/support/npc_quest_chain.rs`:
//! the 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with the production reader), the 49/2 accept,
//! the `job_level` goal (T361, completed by a real `realm level`), the Talk that holds the 49/13,
//! the 26/6 that releases it and the 49/14 turn-in. Cards, end texts, gates and payouts are read
//! from the shipped records through `ZoneQuest::from_record`; no reward is pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`,
//! 49/13 `QuestEndPacket::ReadFields` `0x00c7b990`, 49/14 `0x00a864c0`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, counts, objective_complete, pending_turnin, profile_xp,
    quest_end_reply, quest_reply, quest_status, realm_level, talk, zone_player,
    zone_player_with_quests,
};
use osfr_tools::{gameplay::Gameplay, quest_engine::QuestStatus};
use quest_records::{RawQuest, as_u32, shipped_quest};

const SILVERRAY: u64 = 9_000_000_203;
const PETRA: u64 = 9_000_000_428;
const CHLOE: u64 = 9_000_000_047;
const BLACKSMITH: i32 = 16;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
const NINJA: i32 = 2;
const BLACKSMITH_20: u32 = 3_020_024;
const MINER_20: u32 = 3_100_023;
const BRAWLER_5: u32 = 3_030_021;
const NINJA_5: u32 = 3_110_007;
/// Quest, giver (also the hand-in) and the job whose active rank it gates.
const GATES: [(u32, u64, i32); 4] = [
    (BLACKSMITH_20, SILVERRAY, BLACKSMITH),
    (MINER_20, SILVERRAY, MINER),
    (BRAWLER_5, PETRA, BRAWLER),
    (NINJA_5, CHLOE, NINJA),
];
/// Both givers' rows are in `blackspore_swamp.json`; Petra's drawn body is her
/// `npc_spawns_extra.json` spawn, whose runtime guid is her content key.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "blackspore_swamp.json",
    files: &[],
    runtime: &[],
};
const OTHER: u64 = CHARACTER + 1;

/// A player on `job` who finished the record's shipped prerequisites, so its offer gate is open.
fn player(fixture: &Fixture, guid: u64, job: i32, id: u32) -> Gameplay {
    let done: Vec<(u32, Vec<u32>)> = shipped_quest(id)
        .prerequisite_quest_ids
        .iter()
        .map(|&pre| (pre, vec![1; shipped_quest(pre).objectives.len()]))
        .collect();
    let done: Vec<(u32, &[u32])> = done.iter().map(|(id, n)| (*id, n.as_slice())).collect();
    zone_player_with_quests(fixture, guid, job, &[ADVENTURER], &done)
}

/// The rank the record's first goal asks for, from the data.
fn gate(record: &RawQuest) -> u32 {
    let goal = &record.objectives[0];
    assert_eq!(goal["kind"], "job_level", "quest {}: goal 1", record.id);
    let level = as_u32(&goal["level"]).unwrap();
    assert!((1..=20).contains(&level), "quest {}: rank gate", record.id);
    level
}

fn walk_record(id: u32, giver: u64, job: i32) -> ZoneQuest {
    let record = shipped_quest(id);
    assert_eq!(record.job, Some(job), "quest {id}: offered to its job only");
    assert!(record.enabled, "quest {id} ships enabled");
    let turnin = record.objectives.last().map_or(giver, |goal| {
        goal["npc_id"]
            .as_u64()
            .filter(|_| goal["kind"] == "talk_to_npc")
            .unwrap_or(giver)
    });
    let quest = ZoneQuest::from_record(record, giver, turnin, job);
    assert!(quest.coins > 0, "quest {id} pays a reward");
    quest
}

#[test]
fn each_gate_is_offered_waits_for_the_rank_and_is_paid_once() {
    for (id, giver, job) in GATES {
        let fixture = Fixture::new();
        let mut game = player(&fixture, CHARACTER, job, id);
        let quest = walk_record(id, giver, job);
        let level = gate(shipped_quest(id));
        let (at_giver, at_turnin) = (MAP.at(CHARACTER, giver), MAP.at(CHARACTER, quest.turnin));
        let mut now = 1_000;

        MAP.accept(&mut game, CHARACTER, &quest, &mut now);
        assert!(counts(&game, id).iter().all(|&count| count == 0));

        // Short of the gate nothing completes, hands in or pays, and a forged 49/14 is refused.
        assert_no_offer(&mut game, &at_turnin, quest.turnin, &mut now, "the hand-in");
        assert_eq!(pending_turnin(&game), None);
        assert_packet_changes_nothing(
            &mut game,
            &at_turnin,
            &quest_end_reply(id),
            now,
            "an early 49/14",
        );
        now += 100;
        realm_level(&mut game, &at_giver, level - 1, now);
        assert_eq!(counts(&game, id)[0], 0, "quest {id}: one rank short");
        assert_eq!(quest_status(&game, id), QuestStatus::Active);

        now += 100;
        let reached = realm_level(&mut game, &at_giver, level, now);
        assert!(
            objective_complete(&reached, id, 1),
            "quest {id}: reaching the rank sends the existing 49/10"
        );
        assert_eq!(counts(&game, id)[0], 1);

        now += 100;
        let (coins_before, xp_before) = (coins(&game), profile_xp(&game, job));
        MAP.turn_in(&mut game, CHARACTER, &quest, job, &mut now);
        assert_eq!(coins(&game), coins_before + quest.coins, "quest {id}");
        assert_eq!(profile_xp(&game, job), xp_before + quest.profile_xp);
        assert_eq!(quest_status(&game, id), QuestStatus::Completed);

        // A replayed hand-in, a replayed accept and a second offer change nothing.
        now += 100;
        assert_packet_changes_nothing(
            &mut game,
            &at_turnin,
            &quest_end_reply(id),
            now,
            "a duplicate 49/14",
        );
        assert_packet_changes_nothing(
            &mut game,
            &at_giver,
            &quest_reply(id, true),
            now + 5,
            "a replayed 49/2",
        );
        assert_no_offer(&mut game, &at_giver, giver, &mut now, "the giver after");
    }
}

#[test]
fn only_the_active_job_is_offered_and_forged_accepts_change_nothing() {
    for (id, giver, _) in GATES {
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
        let at_giver = MAP.at(CHARACTER, giver);
        let mut now = 1_000;
        assert_no_offer(&mut game, &at_giver, giver, &mut now, "another job");
        assert_packet_changes_nothing(
            &mut game,
            &at_giver,
            &quest_reply(id, true),
            now,
            "a forged 49/2 with another job active",
        );
    }
}

#[test]
fn silverray_offers_each_active_job_only_its_own_gate() {
    for (id, other, job) in [
        (BLACKSMITH_20, MINER_20, BLACKSMITH),
        (MINER_20, BLACKSMITH_20, MINER),
    ] {
        let fixture = Fixture::new();
        let mut game = zone_player(&fixture, CHARACTER, job);
        let quest = walk_record(id, SILVERRAY, job);
        let at_giver = MAP.at(CHARACTER, SILVERRAY);
        let mut now = 1_000;
        // The sibling gate's forged accept is refused while this job's gate is the offered one.
        assert_packet_changes_nothing(
            &mut game,
            &at_giver,
            &quest_reply(other, true),
            now,
            "the other job's gate",
        );
        MAP.accept(&mut game, CHARACTER, &quest, &mut now);
        assert_eq!(quest_status(&game, id), QuestStatus::Active);
        assert!(
            game.state()
                .npc_quests
                .unwrap()
                .quests
                .get(&other)
                .is_none(),
            "the other job's gate never started"
        );
    }
}

#[test]
fn petra_holds_the_rank_goal_before_her_talk_and_progress_survives_a_reconnect() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, BRAWLER);
    let mut other = zone_player(&fixture, OTHER, BRAWLER);
    let quest = walk_record(BRAWLER_5, PETRA, BRAWLER);
    let level = gate(shipped_quest(BRAWLER_5));
    let petra = MAP.at(CHARACTER, PETRA);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    MAP.accept(&mut other, OTHER, &quest, &mut now);

    // The Talk goal is second: Petra before the rank does not advance it.
    let early = talk(&mut game, &petra, PETRA, now + 10);
    assert!(
        !early.changed,
        "Petra's Talk before the rank advanced the quest"
    );
    assert_eq!(counts(&game, BRAWLER_5), [0, 0]);

    now += 100;
    realm_level(&mut game, &petra, level, now);
    assert_eq!(counts(&game, BRAWLER_5), [1, 0]);
    let mut game = fixture.save_and_reload(&game, &petra);
    assert_eq!(quest_status(&game, BRAWLER_5), QuestStatus::Active);
    assert_eq!(counts(&game, BRAWLER_5), [1, 0]);

    // The held end screen (shown, 49/14 not yet sent) also survives a save.
    now += 100;
    assert!(talk(&mut game, &petra, PETRA, now).changed);
    let mut game = fixture.save_and_reload(&game, &petra);
    assert_eq!(pending_turnin(&game), Some(BRAWLER_5));
    now += 100;
    talk(&mut game, &petra, PETRA, now);
    MAP.pay_held_turnin(&mut game, CHARACTER, &quest, BRAWLER, &mut now);
    assert_eq!(quest_status(&game, BRAWLER_5), QuestStatus::Completed);

    // The second player shares nothing with the first one's progress.
    assert_eq!(quest_status(&other, BRAWLER_5), QuestStatus::Active);
    assert_eq!(counts(&other, BRAWLER_5), [0, 0]);
}

#[test]
fn chloes_level_five_gate_waits_for_the_stealth_referral() {
    let fixture = Fixture::new();
    let chloe = MAP.at(CHARACTER, CHLOE);
    let mut now = 1_000;
    // A Ninja who has not finished the entry line is offered nothing and cannot forge the accept.
    let mut early = zone_player(&fixture, CHARACTER, NINJA);
    assert_no_offer(
        &mut early,
        &chloe,
        CHLOE,
        &mut now,
        "Chloe before the referral",
    );
    assert_packet_changes_nothing(
        &mut early,
        &chloe,
        &quest_reply(NINJA_5, true),
        now,
        "a forged 49/2 before the referral",
    );
    // Once the referral is done she offers the gate.
    let mut ready = player(&fixture, CHARACTER, NINJA, NINJA_5);
    let quest = walk_record(NINJA_5, CHLOE, NINJA);
    MAP.accept(&mut ready, CHARACTER, &quest, &mut now);
    assert_eq!(quest_status(&ready, NINJA_5), QuestStatus::Active);
}
