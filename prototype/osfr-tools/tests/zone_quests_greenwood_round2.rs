//! T671: zone quests round 2, Greenwood Forest and Lakeshore. Four quests whose NPCs the data
//! already draws are enabled (`quest-lines/quests.json` q3010009, q3010012, q3020019, q3160001):
//! Hilforn's Archer "Achieve Level 5" (3010009) and "Archer Departure" (3010012), Mili Ironcall's
//! Blacksmith "Reach Level 15" (3020019) and Fizzlesticks' "Being a Better Wizard" (3160001).
//! Every Brawler and Card Duelist row stays disabled by the lead's decision
//! (`tests/tcg_and_brawler_npc_placements.rs` guards it).
//!
//! Every packet goes through the production router (`Gameplay::handle_in_world`) via the shared
//! walkers in `tests/support/npc_quest_chain.rs`: 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with
//! the production reader), 49/2 accept, `Gameplay::on_location` for the visit goal, the Talk
//! steps, the developer `realm level` command (the server's own rank change) and the
//! 26/6 -> 49/13 -> 49/14 turn-in. Giver, job gate, objectives and payouts are read from the
//! shipped records, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`, 49/13
//! `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`, end text `18..22`,
//! then the reward bundle `0x008e7930`), 49/14 `0x00a864c0`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, character_at_for, coins, counts, objective_complete,
    pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status, realm_level,
    seed_quest, talk, zone_player_with_quests,
};
use osfr_tools::{gameplay::Gameplay, quest_engine::QuestStatus};
use quest_records::shipped_quest;

const ARCHER: i32 = 35;
const BLACKSMITH: i32 = 16;
const WIZARD: i32 = 12;
const HILFORN: u64 = 9_000_000_106;
const VICKI: u64 = 9_000_000_230;
const MILI: u64 = 9_000_000_405;
const FIZZLESTICKS: u64 = 9_000_000_324;
const ARCHER_5: u32 = 3_010_009;
const DEPARTURE: u32 = 3_010_012;
const BLACKSMITH_15: u32 = 3_020_019;
const BETTER_WIZARD: u32 = 3_160_001;
/// Oath of Honor, the last enabled step of the Hilforn chain: Archer 5 follows it.
const OATH: u32 = 3_010_006;
/// Fizzlesticks' other ungated entry quest (T542); a save has it done so only the quest under
/// test is left to offer a player who is not a Wizard.
const BECOME_A_WIZARD: u32 = 3_160_035;
/// Mili's ungated Miner starter; a save has it done so her Talk offers only the quest under test.
const BECOME_A_MINER: u32 = 3_100_001;
/// Hilforn and Mili live in `greenwood_forest.json`; Vicki and Fizzlesticks in their own zone files.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "greenwood_forest.json",
    files: &[
        (VICKI, "farnum_farmstead.json"),
        (FIZZLESTICKS, "wilds.json"),
    ],
    runtime: &[],
};

fn quest(id: u32, giver: u64, turnin: u64, profile: i32) -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(id), giver, turnin, profile)
}

/// A save with `active` selected, `unlocked` jobs learned and `done` quests completed.
fn player(fixture: &Fixture, active: i32, unlocked: &[i32], done: &[(u32, &[u32])]) -> Gameplay {
    zone_player_with_quests(fixture, CHARACTER, active, unlocked, done)
}

/// An Archer who finished the Oath and holds Archer 5 unfinished, so Hilforn offers nothing.
fn archer_with_gate_open(fixture: &Fixture) -> Gameplay {
    let mut state = player(fixture, ARCHER, &[ADVENTURER], &[(OATH, &[1])]).state();
    seed_quest(&mut state, ARCHER_5, QuestStatus::Active, &[0, 0]);
    fixture.game_as(CHARACTER, Some(state))
}

/// Offer, accept, climb the rank (one short never credits, the gate does), turn in at `giver`
/// and prove it is paid once and the giver offers nothing more.
fn run_level_gate(id: u32, giver: u64, profile: i32, unlocked: &[i32], done: &[(u32, &[u32])]) {
    let record = shipped_quest(id);
    assert!(record.enabled);
    assert_eq!(record.job, Some(profile), "a job-gated level quest");
    let gate = quest(id, giver, giver, profile);
    assert!(gate.coins > 0, "the reward is authored, not empty");

    let fixture = Fixture::new();
    let mut game = player(&fixture, profile, unlocked, done);
    let at = MAP.at(CHARACTER, giver);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &gate, &mut now);
    assert_eq!(counts(&game, id), [0, 0]);

    // The rank goal is open: the giver's Talk and an early hand-in change nothing.
    assert_no_offer(&mut game, &at, giver, &mut now, "rank goal open");
    assert_eq!(pending_turnin(&game), None);
    assert_packet_changes_nothing(&mut game, &at, &quest_end_reply(id), now, "an early 49/14");

    realm_level(
        &mut game,
        &at,
        shipped_quest(id).job_level_goal() - 1,
        now + 20,
    );
    assert_eq!(counts(&game, id), [0, 0], "one rank short");
    let reached = realm_level(&mut game, &at, shipped_quest(id).job_level_goal(), now + 30);
    assert!(objective_complete(&reached, id, 1));
    assert_eq!(counts(&game, id), [1, 0]);
    assert_eq!(
        pending_turnin(&game),
        None,
        "a level-up opens no end screen"
    );

    now += 40;
    let before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &gate, profile, &mut now);
    assert_eq!(quest_status(&game, id), QuestStatus::Completed);
    assert_eq!(coins(&game), before + gate.coins);

    // Paid once: a duplicate hand-in and a replayed accept change nothing.
    let duplicate = quest_end_reply(id);
    assert_packet_changes_nothing(&mut game, &at, &duplicate, now, "a duplicate 49/14");
    let replay = quest_reply(id, true);
    assert_packet_changes_nothing(&mut game, &at, &replay, now + 5, "a replayed 49/2");
    assert_eq!(coins(&game), before + gate.coins);
}

#[test]
fn hilforn_archer_level_five_gate_runs_and_pays_once() {
    run_level_gate(
        ARCHER_5,
        HILFORN,
        ARCHER,
        &[ADVENTURER, ARCHER],
        &[(OATH, &[1])],
    );
}

#[test]
fn mili_blacksmith_level_fifteen_gate_runs_and_pays_once() {
    run_level_gate(
        BLACKSMITH_15,
        MILI,
        BLACKSMITH,
        &[ADVENTURER, BLACKSMITH],
        &[(BECOME_A_MINER, &[1])],
    );
}

#[test]
fn archer_departure_visits_the_farm_then_finds_vicki_and_survives_a_reconnect() {
    let record = shipped_quest(DEPARTURE);
    assert!(record.enabled);
    assert_eq!(record.job, Some(ARCHER));
    let departure = quest(DEPARTURE, HILFORN, VICKI, ARCHER);
    assert!(departure.coins > 0 && departure.profile_xp > 0);
    let farm: [f32; 3] = serde_json::from_value(record.objectives[0]["position"].clone()).unwrap();

    let fixture = Fixture::new();
    let mut game = player(
        &fixture,
        ARCHER,
        &[ADVENTURER, ARCHER],
        &[(OATH, &[1]), (ARCHER_5, &[1, 1])],
    );
    let (hilforn, vicki) = (MAP.at(CHARACTER, HILFORN), MAP.at(CHARACTER, VICKI));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &departure, &mut now);
    assert_eq!(counts(&game, DEPARTURE), [0, 0]);

    // Ordered: Vicki's Talk before the visit counts nothing, and neither does standing at Hilforn.
    let before = talk(&mut game, &vicki, VICKI, now);
    assert!(!before.changed, "Vicki has nothing before the visit");
    assert!(!game.on_location(&hilforn, now + 5).unwrap().changed);
    assert_eq!(counts(&game, DEPARTURE), [0, 0]);

    let visited = game
        .on_location(&character_at_for(CHARACTER, 0, farm), now + 10)
        .unwrap();
    assert!(visited.changed, "the farm completes the visit");
    assert_eq!(counts(&game, DEPARTURE), [1, 0]);
    assert_eq!(pending_turnin(&game), None);

    // Reconnect mid-quest, then find Vicki: she holds the end screen and pays the record's reward.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(counts(&game, DEPARTURE), [1, 0]);
    now += 100;
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ARCHER));
    MAP.turn_in(&mut game, CHARACTER, &departure, ARCHER, &mut now);
    assert_eq!(quest_status(&game, DEPARTURE), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + departure.coins);
    assert_eq!(profile_xp(&game, ARCHER), xp_before + departure.profile_xp);
    assert_packet_changes_nothing(
        &mut game,
        &vicki,
        &quest_end_reply(DEPARTURE),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &hilforn, HILFORN, &mut now, "Hilforn after");
}

#[test]
fn fizzlesticks_offers_being_a_better_wizard_to_any_job_once() {
    let record = shipped_quest(BETTER_WIZARD);
    assert!(record.enabled);
    assert_eq!(record.giver_npc_id, FIZZLESTICKS);
    assert_eq!(
        record.job, None,
        "ungated: the client shows it on another job"
    );
    let entry = quest(BETTER_WIZARD, FIZZLESTICKS, FIZZLESTICKS, ADVENTURER);
    assert!(entry.coins > 0, "the reward is authored, not empty");
    assert_eq!(entry.xp, 0, "no job XP for a job not learned");

    let fixture = Fixture::new();
    // An Adventurer who has not learned Wizard: the Wizard-gated line stays hidden, Become a
    // Wizard is done, so the entry quest is the one offer.
    let mut game = player(&fixture, ADVENTURER, &[], &[(BECOME_A_WIZARD, &[1])]);
    let at = MAP.at(CHARACTER, FIZZLESTICKS);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &entry, &mut now);
    assert_eq!(counts(&game, BETTER_WIZARD), [0]);
    let before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &entry, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, BETTER_WIZARD), QuestStatus::Completed);
    assert_eq!(coins(&game), before + entry.coins);

    // Paid once: nothing is offered again and a replay changes nothing.
    assert_no_offer(&mut game, &at, FIZZLESTICKS, &mut now, "after");
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_reply(BETTER_WIZARD, true),
        now,
        "a replayed 49/2",
    );
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(BETTER_WIZARD),
        now + 5,
        "a duplicate 49/14",
    );
    assert_eq!(coins(&game), before + entry.coins);
}

#[test]
fn wrong_job_or_point_in_the_line_is_refused_and_changes_nothing() {
    let fixture = Fixture::new();
    let mut now = 2_000;
    let (hilforn, mili, vicki) = (
        MAP.at(CHARACTER, HILFORN),
        MAP.at(CHARACTER, MILI),
        MAP.at(CHARACTER, VICKI),
    );
    // Archer 5 needs the Archer job and the Oath; Departure needs Archer 5 done; the Blacksmith
    // gate needs the Blacksmith job. A forged accept at the right NPC adds nothing either.
    let cases = [
        (
            ARCHER_5,
            HILFORN,
            &hilforn,
            "an Adventurer at Hilforn",
            player(&fixture, ADVENTURER, &[ARCHER], &[(OATH, &[1])]),
        ),
        (
            ARCHER_5,
            HILFORN,
            &hilforn,
            "an Archer without the Oath",
            player(&fixture, ARCHER, &[ADVENTURER], &[]),
        ),
        (
            DEPARTURE,
            HILFORN,
            &hilforn,
            "an Archer with Archer 5 still open",
            archer_with_gate_open(&fixture),
        ),
        (
            BLACKSMITH_15,
            MILI,
            &mili,
            "a Wizard at Mili",
            player(&fixture, WIZARD, &[ADVENTURER], &[(BECOME_A_MINER, &[1])]),
        ),
    ];
    for (id, giver, at, label, mut game) in cases {
        assert_no_offer(&mut game, at, giver, &mut now, label);
        assert_packet_changes_nothing(&mut game, at, &quest_reply(id, true), now, label);
        assert!(
            !game
                .state()
                .npc_quests
                .as_ref()
                .unwrap()
                .quests
                .contains_key(&id),
            "{label}: the forged accept added the quest"
        );
    }

    // Vicki is only the hand-in: with Departure not taken she has nothing to take in.
    let mut game = player(
        &fixture,
        ARCHER,
        &[ADVENTURER],
        &[(OATH, &[1]), (ARCHER_5, &[1, 1])],
    );
    assert_packet_changes_nothing(
        &mut game,
        &vicki,
        &quest_end_reply(DEPARTURE),
        now,
        "a forged 49/14",
    );
}
