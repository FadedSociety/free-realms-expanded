//! T667: zone quests, Wilds part 2. Of the Wilds rows in
//! `lab/references/quest-lines/wiring/zone-quests/wilds.json` that ship disabled, two run end to
//! end on this tip and are enabled: Glerga's `Chef: Reach Level 20` (3050031, quest-lines/quests.json
//! q3050031) and Fizzlesticks' `Wizard: Reach Level 5` (3160010, q3160010). Every other row stays
//! off; `the_other_wilds_rows_stay_off_with_a_blocker` re-checks each mechanical blocker and fails
//! when one lifts, so the next builder re-checks the row before wiring it.
//!
//! Every packet goes through the production router (`Gameplay::handle_in_world`) via the shared
//! walkers in `tests/support/npc_quest_chain.rs`: 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with
//! the production reader), 49/2 accept, the Talk steps, the developer `realm level` command (the
//! server's own rank change) and the 26/6 -> 49/13 -> 49/14 turn-in. The job gate, giver, card
//! ids, objectives, end text and payout are read from the shipped records, never pinned here.
//! A `job_level` step after a Talk step is credited when the Talk opens it and on every later
//! rank change (T743, `tests/zone_quests_seaside_round3.rs`).
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`, 49/9
//! `0x00c7af60`, 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`,
//! end text `18..22`, one dword, then the reward bundle `0x008e7930`), 49/14 `0x00a864c0`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/placement.rs"]
mod placement;
#[path = "support/quest_records.rs"]
mod quest_records;
#[path = "support/quest_target_sets.rs"]
mod quest_target_sets;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, objective_activated,
    objective_complete, pending_turnin, quest_end_reply, quest_reply, quest_status, realm_level,
    reference_content, saved, seed_active_profile, seed_quest, talk, zone_player_with_quests,
};
use osfr_tools::{gameplay::Gameplay, quest_engine::QuestStatus, world::NpcVisibility};
use quest_records::shipped_quest;
use quest_target_sets::Targets;
use std::collections::BTreeSet;

const CHEF: i32 = 45;
const WIZARD: i32 = 12;
const GLERGA: u64 = 9_000_000_332;
const FIZZLESTICKS: u64 = 9_000_000_324;
const NATALI: u64 = 9_500_000_360;
const CHEF_20: u32 = 3_050_031;
const WIZARD_5: u32 = 3_160_010;
/// Each quest's prerequisite: the last enabled quest before it in its line.
const CHEF_5: u32 = 3_050_020;
/// The wizard first-chapter quests Fizzlesticks offers before Reach Level 5, so a Talk at him
/// offers nothing else: Become a Wizard (3160035), the three Robgoblin Camp quests and the
/// ungated Being a Better Wizard (3160001, T671).
const WIZARD_OPENING: [(u32, &[u32]); 5] = [
    (3_160_035, &[1]),
    (3_160_001, &[1]),
    (3_160_003, &[1, 1]),
    (3_160_004, &[1, 1]),
    (3_160_005, &[1, 1]),
];
/// Glerga and Fizzlesticks live in `wilds.json`, Natali in `lakeshore.json` (a reference row the
/// loader draws from `npc_spawns_extra.json` under its own content key).
const MAP: ZoneQuests = ZoneQuests {
    default_file: "wilds.json",
    files: &[(NATALI, "lakeshore.json")],
    runtime: &[],
};

fn chef_quest() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(CHEF_20), GLERGA, GLERGA, CHEF)
}

/// Fizzlesticks gives it and takes it back; Natali is the first step, never the turn-in.
fn wizard_quest() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(WIZARD_5), FIZZLESTICKS, FIZZLESTICKS, WIZARD)
}

/// A Chef with `done` quests completed.
fn chef(fixture: &Fixture, active: i32, done: &[(u32, &[u32])]) -> Gameplay {
    let unlocked = [ADVENTURER, CHEF];
    zone_player_with_quests(fixture, CHARACTER, active, &unlocked, done)
}

/// A Wizard whose opening quests are done, so Fizzlesticks has only Reach Level 5 to offer.
fn wizard(fixture: &Fixture, active: i32) -> Gameplay {
    let unlocked = [ADVENTURER, WIZARD];
    zone_player_with_quests(fixture, CHARACTER, active, &unlocked, &WIZARD_OPENING)
}

#[test]
fn glerga_chef_level_twenty_is_offered_walked_and_paid_once() {
    let record = shipped_quest(CHEF_20);
    assert!(record.enabled);
    assert_eq!(record.job, Some(CHEF), "a Chef-only quest");
    let quest = chef_quest();
    assert!(quest.coins > 0, "the reward is authored, not empty");

    let fixture = Fixture::new();
    let mut game = chef(&fixture, CHEF, &[(CHEF_5, &[1, 1])]);
    let glerga = MAP.at(CHARACTER, GLERGA);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, CHEF_20), [0, 0]);

    // The rank goal is open: her Talk and an early hand-in change nothing.
    assert_no_offer(&mut game, &glerga, GLERGA, &mut now, "rank goal open");
    assert_eq!(pending_turnin(&game), None);
    let early = quest_end_reply(CHEF_20);
    assert_packet_changes_nothing(&mut game, &glerga, &early, now, "an early 49/14");
    assert_eq!(quest_status(&game, CHEF_20), QuestStatus::Active);

    // One rank short never credits it; the gate rank does.
    let gate = shipped_quest(CHEF_20).job_level_goal();
    realm_level(&mut game, &glerga, gate - 1, now + 20);
    assert_eq!(counts(&game, CHEF_20), [0, 0], "one rank short");
    let reached = realm_level(&mut game, &glerga, gate, now + 30);
    assert!(objective_complete(&reached, CHEF_20, 1));
    assert_eq!(counts(&game, CHEF_20), [1, 0]);
    assert_eq!(
        pending_turnin(&game),
        None,
        "a level-up opens no end screen"
    );

    now += 40;
    let before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &quest, CHEF, &mut now);
    assert_eq!(quest_status(&game, CHEF_20), QuestStatus::Completed);
    assert_eq!(coins(&game), before + quest.coins);

    // A replayed hand-in and accept pay and add nothing, and she offers nothing more.
    let duplicate = quest_end_reply(CHEF_20);
    assert_packet_changes_nothing(&mut game, &glerga, &duplicate, now, "a duplicate 49/14");
    let replay = quest_reply(CHEF_20, true);
    assert_packet_changes_nothing(&mut game, &glerga, &replay, now + 5, "a replayed 49/2");
    assert_no_offer(&mut game, &glerga, GLERGA, &mut now, "after");
    assert_eq!(coins(&game), before + quest.coins);
}

#[test]
fn fizzlesticks_wizard_level_five_runs_in_the_clients_order() {
    let record = shipped_quest(WIZARD_5);
    assert!(record.enabled);
    assert_eq!(record.job, Some(WIZARD), "a Wizard-only quest");
    let quest = wizard_quest();
    assert!(quest.coins > 0, "the reward is authored, not empty");

    let fixture = Fixture::new();
    let mut game = wizard(&fixture, WIZARD);
    let (fizzlesticks, natali) = (MAP.at(CHARACTER, FIZZLESTICKS), MAP.at(CHARACTER, NATALI));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, WIZARD_5), [0, 0, 0]);

    // Natali comes first: the rank goal is not open yet, and Fizzlesticks has nothing to take in.
    now += 10;
    realm_level(
        &mut game,
        &fizzlesticks,
        shipped_quest(WIZARD_5).job_level_goal(),
        now,
    );
    assert_eq!(
        counts(&game, WIZARD_5),
        [0, 0, 0],
        "the rank goal is not open yet"
    );
    assert_no_offer(
        &mut game,
        &fizzlesticks,
        FIZZLESTICKS,
        &mut now,
        "before Natali",
    );
    let early = quest_end_reply(WIZARD_5);
    assert_packet_changes_nothing(&mut game, &fizzlesticks, &early, now, "an early 49/14");

    // Natali's Talk opens the rank step, and the rank already reached completes it at once.
    now += 10;
    let talked = talk(&mut game, &natali, NATALI, now);
    assert!(objective_complete(&talked, WIZARD_5, 1));
    assert!(objective_complete(&talked, WIZARD_5, 2));
    assert_eq!(counts(&game, WIZARD_5), [1, 1, 0]);
    assert_eq!(pending_turnin(&game), None, "Natali is not the turn-in");

    // Back at Natali: still no end screen there. Fizzlesticks takes it in and pays.
    now += 10;
    talk(&mut game, &natali, NATALI, now);
    assert_eq!(
        pending_turnin(&game),
        None,
        "Natali must not hold the end screen"
    );
    now += 10;
    let before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &quest, WIZARD, &mut now);
    assert_eq!(quest_status(&game, WIZARD_5), QuestStatus::Completed);
    assert_eq!(coins(&game), before + quest.coins);
    assert_no_offer(&mut game, &fizzlesticks, FIZZLESTICKS, &mut now, "after");
}

#[test]
fn a_new_wizard_levels_up_after_natali_and_progress_survives_a_reconnect() {
    let quest = wizard_quest();
    let fixture = Fixture::new();
    let mut game = wizard(&fixture, WIZARD);
    let natali = MAP.at(CHARACTER, NATALI);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    now += 10;
    let talked = talk(&mut game, &natali, NATALI, now);
    assert!(objective_complete(&talked, WIZARD_5, 1));
    assert!(objective_activated(&talked, WIZARD_5, 2));
    assert_eq!(counts(&game, WIZARD_5), [1, 0, 0]);

    // A save and load between the steps keeps the Talk, and the rank step is still open.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, WIZARD_5), QuestStatus::Active);
    assert_eq!(counts(&game, WIZARD_5), [1, 0, 0]);
    now += 10;
    let gate = shipped_quest(WIZARD_5).job_level_goal();
    realm_level(&mut game, &natali, gate - 1, now);
    assert_eq!(counts(&game, WIZARD_5), [1, 0, 0], "one rank short");
    let reached = realm_level(&mut game, &natali, gate, now + 10);
    assert!(objective_complete(&reached, WIZARD_5, 2));
    assert_eq!(counts(&game, WIZARD_5), [1, 1, 0]);

    now += 30;
    let before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &quest, WIZARD, &mut now);
    assert_eq!(quest_status(&game, WIZARD_5), QuestStatus::Completed);
    assert_eq!(coins(&game), before + quest.coins);
}

#[test]
fn natali_before_the_offer_the_wrong_job_and_a_missing_prerequisite_change_nothing() {
    let fixture = Fixture::new();
    let mut now = 1_000;

    // Natali's Talk without the quest is inert.
    let mut game = wizard(&fixture, WIZARD);
    let before = saved(&game);
    let talked = talk(&mut game, &MAP.at(CHARACTER, NATALI), NATALI, now);
    assert!(!talked.changed, "Natali's Talk without the quest is inert");
    assert_eq!(saved(&game), before);

    // Another job active, or the prerequisite undone (Wizards still on the opening quests, a
    // Chef who has not done Reach Level 5): nothing is offered and a forged accept adds nothing.
    let adventurer = |done: &[(u32, &[u32])]| {
        zone_player_with_quests(&fixture, CHARACTER, ADVENTURER, &[], done)
    };
    // The second opening quest is still in progress: without the prerequisite Fizzlesticks
    // would already offer Reach Level 5.
    let unfinished = {
        let mut state = fixture.game_as(CHARACTER, None).state();
        seed_active_profile(&mut state, WIZARD, &[ADVENTURER, WIZARD]);
        for (id, counters) in &WIZARD_OPENING[..3] {
            seed_quest(&mut state, *id, QuestStatus::Completed, counters);
        }
        seed_quest(&mut state, 3_160_004, QuestStatus::Active, &[0, 0]);
        fixture.game_as(CHARACTER, Some(state))
    };
    let cases = [
        (
            CHEF_20,
            GLERGA,
            "another job",
            adventurer(&[(CHEF_5, &[1, 1])]),
        ),
        (
            WIZARD_5,
            FIZZLESTICKS,
            "another job",
            adventurer(&WIZARD_OPENING),
        ),
        (
            CHEF_20,
            GLERGA,
            "no Reach Level 5",
            chef(&fixture, CHEF, &[]),
        ),
        (WIZARD_5, FIZZLESTICKS, "no Sticky Spell Book", unfinished),
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

/// Wilds part 2 rows that stay disabled, each with the blockers this suite can check. The rest
/// of the 33 rows need a mechanic or a fact the server lacks (see `notes/Autopilot/T667.md`).
#[test]
fn the_other_wilds_rows_stay_off_with_a_blocker() {
    /// Quests whose giver 9000000058 / 9000000281 / 9000000055 (Coach Eddie) has no drawn row.
    const NO_GIVER: [u32; 5] = [3_140_006, 3_140_010, 3_140_011, 3_140_023, 3_140_026];
    /// Quests whose `finish_minigame` id has no server path that reports it.
    const NO_MINIGAME: [(u32, u32); 1] = [(3_040_025, 527)];
    /// Quests behind a prerequisite that is itself disabled.
    const NO_PREREQUISITE: [u32; 2] = [3_160_008, 3_160_009];
    let root = content_dir();
    let visibility = NpcVisibility::load(&root).unwrap();
    let drawn: BTreeSet<u64> = placement::drawn_npcs(&visibility)
        .into_iter()
        .filter_map(|npc| npc.content_key)
        .collect();
    let targets = Targets::load(&root, &reference_content()).unwrap();
    let off = |id: u32| {
        let quest = shipped_quest(id);
        assert!(!quest.enabled, "quest {id} is enabled, not blocked");
        quest
    };
    for id in NO_GIVER {
        let giver = off(id).giver_npc_id;
        assert!(!drawn.contains(&giver), "{id}: giver {giver} is drawn now");
    }
    for (id, game) in NO_MINIGAME {
        off(id);
        assert!(
            !targets.minigames.contains(&game),
            "{id}: minigame {game} has an emitter now"
        );
    }
    for id in NO_PREREQUISITE {
        let disabled = off(id)
            .prerequisite_quest_ids
            .iter()
            .any(|p| !shipped_quest(*p).enabled);
        assert!(disabled, "{id}: its prerequisites are enabled now");
    }
    // The rest of the list: still stubs nobody has wired.
    for id in [3_060_022, 3_090_010, 3_160_006, 3_160_007, 3_160_012] {
        assert!(
            off(id).objectives.is_empty(),
            "{id} grew objectives; re-check it"
        );
    }
}
