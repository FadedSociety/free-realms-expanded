//! T672: zone quests round 2, Merry Vale. Three of the rows in
//! `lab/references/quest-lines/wiring/zone-quests-matched/merry-vale.json` run end to end on this
//! tip and are enabled: Chef Jarvie's `Chef: Reach Level 10` (3050023, quest-lines/quests.json
//! q3050023) and `Chef: Chugawhat?` (3050024, q3050024, taken in by Cook Yumtumlot in Wugachug),
//! and Tumbolina's `Warrior: Distracted` (3150008, q3150008, via Braldan Stoneaxe). The Brawler
//! and Card Duelist quests stay off by the lead's decision
//! (`tests/tcg_and_brawler_npc_placements.rs` guards it); `the_other_merry_vale_rows_stay_off`
//! re-checks the rest.
//!
//! Every packet goes through the production router (`Gameplay::handle_in_world`) via the shared
//! walkers in `tests/support/npc_quest_chain.rs`: 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with
//! the production reader), 49/2 accept, the Talk steps, the developer `realm level` command (the
//! server's own rank change) and the 26/6 -> 49/13 -> 49/14 turn-in. Job gates, givers, card ids,
//! end text and payouts are read from the shipped records, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`, 49/13
//! `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`, end text `18..22`,
//! one dword, then the reward bundle `0x008e7930`), 49/14 `0x00a864c0`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, counts, objective_complete, pending_turnin, profile_xp,
    quest_end_reply, quest_reply, quest_status, realm_level, saved, zone_player_with_quests,
};
use osfr_tools::{gameplay::Gameplay, quest_engine::QuestStatus};
use quest_records::shipped_quest;

const CHEF: i32 = 45;
const WARRIOR: i32 = 32;
const JARVIE: u64 = 9_000_000_044;
const YUMTUMLOT: u64 = 9_000_000_061;
const TUMBOLINA: u64 = 9_000_000_222;
const BRALDAN: u64 = 9_000_000_030;
const CHEF_10: u32 = 3_050_023;
const CHUGAWHAT: u32 = 3_050_024;
const DISTRACTED: u32 = 3_150_008;
/// Each quest's prerequisite: the last enabled quest before it in its line.
const CHEF_5: u32 = 3_050_020;
const WARRIOR_5: u32 = 3_150_005;
/// Jarvie, Tumbolina and Braldan live in `merry_vale.json`, Cook Yumtumlot in `wugachug.json`.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "merry_vale.json",
    files: &[(YUMTUMLOT, "wugachug.json")],
    runtime: &[],
};

fn chef_ten() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(CHEF_10), JARVIE, JARVIE, CHEF)
}

/// Jarvie gives it; Cook Yumtumlot's one Talk completes it and takes it in.
fn chugawhat() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(CHUGAWHAT), JARVIE, YUMTUMLOT, CHEF)
}

/// Tumbolina gives it and takes it back; Braldan is the first step, never the turn-in.
fn distracted() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(DISTRACTED), TUMBOLINA, TUMBOLINA, WARRIOR)
}

/// A Chef with `done` quests completed.
fn chef(fixture: &Fixture, done: &[(u32, &[u32])]) -> Gameplay {
    zone_player_with_quests(fixture, CHARACTER, CHEF, &[ADVENTURER, CHEF], done)
}

fn warrior(fixture: &Fixture, done: &[(u32, &[u32])]) -> Gameplay {
    zone_player_with_quests(fixture, CHARACTER, WARRIOR, &[ADVENTURER, WARRIOR], done)
}

fn goal_level(id: u32) -> u32 {
    let goals = &shipped_quest(id).objectives;
    let goal = goals
        .iter()
        .find(|goal| goal["kind"] == "job_level")
        .expect("a rank goal");
    quest_records::as_u32(&goal["level"]).unwrap()
}

#[test]
fn jarvie_chef_level_ten_is_offered_walked_and_paid_once() {
    let record = shipped_quest(CHEF_10);
    assert!(record.enabled);
    assert_eq!(record.job, Some(CHEF), "a Chef-only quest");
    let quest = chef_ten();
    assert!(quest.coins > 0, "the reward is authored, not empty");

    let fixture = Fixture::new();
    let mut game = chef(&fixture, &[(CHEF_5, &[1, 1])]);
    let jarvie = MAP.at(CHARACTER, JARVIE);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, CHEF_10), [0, 0]);

    // The rank goal is open: his Talk and an early hand-in change nothing.
    assert_no_offer(&mut game, &jarvie, JARVIE, &mut now, "rank goal open");
    assert_eq!(pending_turnin(&game), None);
    let early = quest_end_reply(CHEF_10);
    assert_packet_changes_nothing(&mut game, &jarvie, &early, now, "an early 49/14");
    assert_eq!(quest_status(&game, CHEF_10), QuestStatus::Active);

    // One rank short never credits it; the gate rank does.
    let gate = goal_level(CHEF_10);
    realm_level(&mut game, &jarvie, gate - 1, now + 20);
    assert_eq!(counts(&game, CHEF_10), [0, 0], "one rank short");
    let reached = realm_level(&mut game, &jarvie, gate, now + 30);
    assert!(objective_complete(&reached, CHEF_10, 1));
    assert_eq!(counts(&game, CHEF_10), [1, 0]);
    assert_eq!(
        pending_turnin(&game),
        None,
        "a level-up opens no end screen"
    );

    now += 40;
    let before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &quest, CHEF, &mut now);
    assert_eq!(quest_status(&game, CHEF_10), QuestStatus::Completed);
    assert_eq!(coins(&game), before + quest.coins);

    // A replayed hand-in and accept pay and add nothing, and he offers nothing more of it.
    let duplicate = quest_end_reply(CHEF_10);
    assert_packet_changes_nothing(&mut game, &jarvie, &duplicate, now, "a duplicate 49/14");
    let replay = quest_reply(CHEF_10, true);
    assert_packet_changes_nothing(&mut game, &jarvie, &replay, now + 5, "a replayed 49/2");
    assert_eq!(coins(&game), before + quest.coins);
}

#[test]
fn chugawhat_follows_level_ten_and_cook_yumtumlot_takes_it_in() {
    let record = shipped_quest(CHUGAWHAT);
    assert!(record.enabled);
    assert_eq!(record.job, Some(CHEF), "a Chef-only quest");
    let (level_ten, quest) = (chef_ten(), chugawhat());
    assert!(quest.coins > 0 && quest.xp > 0, "the reward is authored");

    let fixture = Fixture::new();
    let mut game = chef(&fixture, &[(CHEF_5, &[1, 1])]);
    let (jarvie, cook) = (MAP.at(CHARACTER, JARVIE), MAP.at(CHARACTER, YUMTUMLOT));
    let mut now = 1_000;

    // Not offered until Reach Level 10 is done, and the cook's Talk is inert without the quest.
    let before = saved(&game);
    assert!(
        !MAP.talk(&mut game, CHARACTER, YUMTUMLOT, now).changed,
        "the cook's Talk without the quest is inert"
    );
    assert_eq!(saved(&game), before);
    assert_packet_changes_nothing(
        &mut game,
        &jarvie,
        &quest_reply(CHUGAWHAT, true),
        now + 5,
        "a forged accept before Reach Level 10",
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&CHUGAWHAT),
        "Chugawhat must not exist before Reach Level 10"
    );

    MAP.accept(&mut game, CHARACTER, &level_ten, &mut now);
    realm_level(&mut game, &jarvie, goal_level(CHEF_10), now);
    MAP.turn_in(&mut game, CHARACTER, &level_ten, CHEF, &mut now);

    // Jarvie now offers Chugawhat; his own Talk is not its turn-in.
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, CHUGAWHAT), [0]);
    assert_no_offer(&mut game, &jarvie, JARVIE, &mut now, "back at Jarvie");
    assert_packet_changes_nothing(
        &mut game,
        &jarvie,
        &quest_end_reply(CHUGAWHAT),
        now,
        "an early 49/14",
    );

    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, CHEF));
    MAP.turn_in(&mut game, CHARACTER, &quest, CHEF, &mut now);
    assert_eq!(quest_status(&game, CHUGAWHAT), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, CHEF), xp_before + quest.profile_xp);
    let duplicate = quest_end_reply(CHUGAWHAT);
    assert_packet_changes_nothing(&mut game, &cook, &duplicate, now, "a duplicate 49/14");
}

#[test]
fn tumbolina_distracted_runs_through_braldan_and_survives_a_reconnect() {
    let record = shipped_quest(DISTRACTED);
    assert!(record.enabled);
    assert_eq!(record.job, Some(WARRIOR), "a Warrior-only quest");
    let quest = distracted();
    assert!(quest.coins > 0 && quest.xp > 0, "the reward is authored");

    let fixture = Fixture::new();
    let mut game = warrior(&fixture, &[(WARRIOR_5, &[1])]);
    let tumbolina = MAP.at(CHARACTER, TUMBOLINA);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, DISTRACTED), [0, 0]);

    // Braldan comes first: Tumbolina has nothing to take in yet.
    assert_no_offer(&mut game, &tumbolina, TUMBOLINA, &mut now, "before Braldan");
    let early = quest_end_reply(DISTRACTED);
    assert_packet_changes_nothing(&mut game, &tumbolina, &early, now, "an early 49/14");

    let talked = MAP.talk(&mut game, CHARACTER, BRALDAN, now);
    assert!(talked.changed, "Braldan's Talk did not credit the step");
    assert!(objective_complete(&talked, DISTRACTED, 1));
    assert_eq!(counts(&game, DISTRACTED), [1, 0]);
    assert_eq!(pending_turnin(&game), None, "Braldan is not the turn-in");

    // A save and load between the steps keeps the Talk; Braldan again changes nothing.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(counts(&game, DISTRACTED), [1, 0]);
    now += 10;
    let braldan = MAP.at(CHARACTER, BRALDAN);
    assert_no_offer(&mut game, &braldan, BRALDAN, &mut now, "Braldan twice");
    assert_eq!(pending_turnin(&game), None);

    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, WARRIOR));
    MAP.turn_in(&mut game, CHARACTER, &quest, WARRIOR, &mut now);
    assert_eq!(quest_status(&game, DISTRACTED), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, WARRIOR), xp_before + quest.profile_xp);
    let replay = quest_reply(DISTRACTED, true);
    assert_packet_changes_nothing(&mut game, &tumbolina, &replay, now, "a replayed 49/2");
    assert_no_offer(&mut game, &tumbolina, TUMBOLINA, &mut now, "after");
}

#[test]
fn the_wrong_job_and_a_missing_prerequisite_change_nothing() {
    let fixture = Fixture::new();
    let mut now = 1_000;
    let adventurer = |done: &[(u32, &[u32])]| {
        zone_player_with_quests(&fixture, CHARACTER, ADVENTURER, &[], done)
    };
    // A Chef who has not done Reach Level 5, a Warrior who has not done
    // Achieve Level 5, and each quest's giver for another job: nothing is offered and a forged
    // accept adds nothing.
    let cases = [
        (
            CHEF_10,
            JARVIE,
            "another job",
            adventurer(&[(CHEF_5, &[1, 1])]),
        ),
        (CHEF_10, JARVIE, "no Reach Level 5", chef(&fixture, &[])),
        (
            DISTRACTED,
            TUMBOLINA,
            "another job",
            adventurer(&[(WARRIOR_5, &[1])]),
        ),
        (
            DISTRACTED,
            TUMBOLINA,
            "no Achieve Level 5",
            warrior(&fixture, &[]),
        ),
    ];
    for (id, giver, label, mut game) in cases {
        let at = MAP.at(CHARACTER, giver);
        assert_no_offer(&mut game, &at, giver, &mut now, label);
        assert_packet_changes_nothing(&mut game, &at, &quest_reply(id, true), now, label);
        assert!(
            !game
                .state()
                .npc_quests
                .as_ref()
                .unwrap()
                .quests
                .contains_key(&id),
            "{label}: quest {id} must not exist"
        );
    }
}

/// Merry Vale rows that stay disabled: the Card Duelist and Postman quests of
/// `zone-quests-matched/merry-vale.json` `still_blocked`, each needing a mechanic the server
/// lacks (see `notes/Autopilot/T672.md`). A row that gains objectives has to be re-checked.
#[test]
fn the_other_merry_vale_rows_stay_off() {
    for id in [3_040_026, 3_130_012, 3_130_013, 3_130_015] {
        let quest = shipped_quest(id);
        assert!(!quest.enabled, "quest {id} is enabled, not blocked");
        assert!(
            quest.objectives.is_empty(),
            "{id} grew objectives; re-check it"
        );
    }
}
