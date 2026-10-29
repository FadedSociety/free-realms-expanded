//! T665: the Snowhill job quests the evidence passes called live and wireable that the
//! server can serve today: Soccer Star "Meeting The Team" (3140005), Warrior "You Deserve
//! Better" (3150007), Ninja "Ice Troll Rampage" (3110023) and the Miner and Ninja level
//! gates (3100006, 3110024).
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`: the 26/8 ->
//! 26/10 -> 26/6 offer (49/1, decoded with the production reader), the 49/2 accept, Talk
//! steps, `realm level` for the rank goals, a position report for the Howling Hills goal, the
//! Talk that holds the 49/13, the 26/6 that releases it and the 49/14 turn-in. Job gates,
//! givers, card ids, end texts and payouts are read from the shipped records through
//! `tests/support/quest_records.rs`, never pinned here (gameplay values stay tunable).
//!
//! Client readers (IDA, 2014 client; no new wire): 49/13 `QuestEndPacket::ReadFields`
//! `0x00c7b990`, AddNpc reader `0x00921a10`; the rank goals reuse the T361 `job_level`
//! objective (49/9 reader `0x00c7af60`, 49/10 reader `0x00c7b070`).
//!
//! Left disabled on purpose (see `notes/Autopilot/T665.md`): the Dartmoor quests and the
//! rest of the wireable rows need a mechanic or a drawn NPC the server lacks.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, interact, objective_complete,
    offer, pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status, realm_level,
    saved, zone_player_with_quests,
};
use osfr_tools::{
    gameplay::Gameplay, quest_engine::QuestStatus, startup_world::parse_add_npc,
    world::NpcVisibility,
};
use quest_records::shipped_quest;

const SORIN: u64 = 9_000_000_472;
const MICHI: u64 = 9_000_000_151;
const STEELE: u64 = 9_000_000_208;
const LANDON: u64 = 9_000_000_140;
const EDDIE: u64 = 9_000_000_053;
const TERRENCE: u64 = 9_000_000_488;
const TUMBOLINA: u64 = 9_000_000_222;
const BIGS: u64 = 9_500_000_870;
const BOBO: u64 = 9_500_000_877;
const DEEDEE: u64 = 9_500_000_903;

const NINJA: i32 = 2;
const MINER: i32 = 14;
const WARRIOR: i32 = 32;
const BRAWLER: i32 = 43;
const SOCCER_STAR: i32 = 52;

/// The ungated Jobs-window record Landon offers any job (T542).
const BECOME_A_SOCCER_STAR: u32 = 3_140_001;
const SCRIMMAGE: u32 = 3_140_004;
const MEETING_THE_TEAM: u32 = 3_140_005;
const WARRIOR_W5: u32 = 3_150_005;
const YOU_DESERVE_BETTER: u32 = 3_150_007;
const MINER_LEVEL_5: u32 = 3_100_006;
const ICE_TROLL: u32 = 3_110_023;
const NINJA_LEVEL_20: u32 = 3_110_024;
const OTHER: u64 = CHARACTER + 1;

/// Snowhill rows by default; the recipients and Eddie live in the Wilds file and Tumbolina
/// in Merry Vale.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "snowhill.json",
    files: &[
        (TERRENCE, "wilds.json"),
        (BIGS, "wilds.json"),
        (BOBO, "wilds.json"),
        (DEEDEE, "wilds.json"),
        (EDDIE, "wilds.json"),
        (TUMBOLINA, "merry_vale.json"),
    ],
    runtime: &[],
};

/// The shipped record as the shared harness walks it: card, end text and payout come from
/// the data; `steps` are the ordered Talks before the hand-in at `turnin`.
fn quest(id: u32, steps: &'static [(u64, u32)], turnin: u64, profile: i32) -> ZoneQuest {
    let record = shipped_quest(id);
    ZoneQuest {
        steps,
        ..ZoneQuest::from_record(record, record.giver_npc_id, turnin, profile)
    }
}

/// A character on `active` with the `done` quests completed.
fn player(fixture: &Fixture, guid: u64, active: i32, done: &[(u32, &[u32])]) -> Gameplay {
    zone_player_with_quests(fixture, guid, active, &[], done)
}

fn soccer_star(fixture: &Fixture, guid: u64) -> Gameplay {
    player(fixture, guid, SOCCER_STAR, &[(SCRIMMAGE, &[1])])
}

fn warrior(fixture: &Fixture, guid: u64) -> Gameplay {
    player(fixture, guid, WARRIOR, &[(WARRIOR_W5, &[1])])
}

fn meeting_the_team() -> ZoneQuest {
    quest(
        MEETING_THE_TEAM,
        &[(BIGS, 0), (BOBO, 0), (DEEDEE, 0)],
        EDDIE,
        SOCCER_STAR,
    )
}

fn you_deserve_better() -> ZoneQuest {
    // Terrence speaks his bound greeting (73492, T298) when the step completes.
    quest(
        YOU_DESERVE_BETTER,
        &[(TERRENCE, 73_492)],
        TUMBOLINA,
        WARRIOR,
    )
}

#[test]
fn meeting_the_team_delivers_three_jerseys_and_ends_with_eddie() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = soccer_star(&fixture, CHARACTER);
    let quest = meeting_the_team();
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, SOCCER_STAR));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    // The recipients are ordered: Bobo before Bigs credits nothing and changes nothing.
    let bobo = MAP.at(CHARACTER, BOBO);
    assert_no_offer(&mut game, &bobo, BOBO, &mut now, "Bobo before Bigs");
    assert_eq!(counts(&game, MEETING_THE_TEAM), [0, 0, 0, 0]);

    for index in 0..quest.steps.len() {
        MAP.step(&mut game, CHARACTER, &quest, index, &mut now);
    }
    assert_eq!(pending_turnin(&game), None, "the hand-in is Eddie's");
    // Eddie's Talk completes the last goal and holds the end screen for him.
    MAP.turn_in(&mut game, CHARACTER, &quest, SOCCER_STAR, &mut now);

    assert!(quest.coins > 0 && quest.profile_xp > 0);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, SOCCER_STAR), xp_before + quest.profile_xp);
    assert_eq!(
        quest_status(&game, MEETING_THE_TEAM),
        QuestStatus::Completed
    );
    let landon = MAP.at(CHARACTER, LANDON);
    assert_packet_changes_nothing(
        &mut game,
        &landon,
        &quest_end_reply(MEETING_THE_TEAM),
        now,
        "a duplicate 49/14",
    );
}

#[test]
fn meeting_the_team_waits_for_the_scrimmage_and_the_job() {
    let fixture = Fixture::minimal(CHARACTER);
    let landon = MAP.at(CHARACTER, LANDON);
    let forged = quest_reply(MEETING_THE_TEAM, true);

    // A Soccer Star who has not played the scrimmage is offered the scrimmage, and a
    // forged accept for the jersey run changes nothing.
    let mut fresh = player(&fixture, CHARACTER, SOCCER_STAR, &[]);
    let now = 1_000;
    offer(&mut fresh, &landon, LANDON, SCRIMMAGE, now);
    assert_packet_changes_nothing(&mut fresh, &landon, &forged, now + 10, "a forged 49/2");

    // Another job is offered only the ungated Become a Soccer Star, even with the scrimmage
    // done, and a forged accept for the jersey run changes nothing.
    let mut brawler = player(&fixture, CHARACTER, BRAWLER, &[(SCRIMMAGE, &[1])]);
    offer(&mut brawler, &landon, LANDON, BECOME_A_SOCCER_STAR, now);
    assert_packet_changes_nothing(&mut brawler, &landon, &forged, now + 10, "a forged 49/2");
}

#[test]
fn meeting_the_team_survives_a_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = soccer_star(&fixture, CHARACTER);
    let quest = meeting_the_team();
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    MAP.step(&mut game, CHARACTER, &quest, 0, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, MEETING_THE_TEAM), QuestStatus::Active);
    assert_eq!(counts(&game, MEETING_THE_TEAM), [1, 0, 0, 0]);
    MAP.step(&mut game, CHARACTER, &quest, 1, &mut now);
}

#[test]
fn you_deserve_better_walks_steele_terrence_tumbolina() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = warrior(&fixture, CHARACTER);
    let quest = you_deserve_better();
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, WARRIOR));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    // Tumbolina cannot take the hand-in before Terrence: her Talk credits no goal. (She also
    // offers Distracted, T672, so this is not an "nothing offered" check.)
    let tumbolina = MAP.at(CHARACTER, TUMBOLINA);
    now += 10;
    interact(&mut game, &tumbolina, TUMBOLINA, now);
    assert_eq!(counts(&game, YOU_DESERVE_BETTER), [0, 0], "Tumbolina first");
    assert_packet_changes_nothing(
        &mut game,
        &tumbolina,
        &quest_end_reply(YOU_DESERVE_BETTER),
        now,
        "an early 49/14",
    );

    MAP.step(&mut game, CHARACTER, &quest, 0, &mut now);
    // The last goal is Tumbolina herself: her Talk completes it and holds the end screen.
    MAP.turn_in(&mut game, CHARACTER, &quest, WARRIOR, &mut now);
    assert_eq!(counts(&game, YOU_DESERVE_BETTER), [1, 1]);

    assert!(quest.coins > 0 && quest.profile_xp > 0);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, WARRIOR), xp_before + quest.profile_xp);
    let steele = MAP.at(CHARACTER, STEELE);
    assert_packet_changes_nothing(
        &mut game,
        &steele,
        &quest_end_reply(YOU_DESERVE_BETTER),
        now,
        "a duplicate 49/14",
    );
}

#[test]
fn you_deserve_better_needs_the_level_gate_and_the_warrior_job() {
    let fixture = Fixture::minimal(CHARACTER);
    let steele = MAP.at(CHARACTER, STEELE);
    let forged = quest_reply(YOU_DESERVE_BETTER, true);
    let mut now = 1_000;

    // A fresh Warrior (W4 and W5 not done) is offered nothing, and a forged accept for the
    // weapon errand changes nothing.
    let mut fresh = player(&fixture, CHARACTER, WARRIOR, &[]);
    assert_no_offer(
        &mut fresh,
        &steele,
        STEELE,
        &mut now,
        "a fresh Warrior at Steele",
    );
    assert_packet_changes_nothing(&mut fresh, &steele, &forged, now + 10, "a forged 49/2");

    // Gate done but another job active: nothing is offered.
    let mut brawler = player(&fixture, CHARACTER, BRAWLER, &[(WARRIOR_W5, &[1])]);
    assert_no_offer(
        &mut brawler,
        &steele,
        STEELE,
        &mut now,
        "a Brawler at Steele",
    );
    assert_packet_changes_nothing(&mut brawler, &steele, &forged, now + 10, "a forged 49/2");
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = warrior(&fixture, CHARACTER);
    let mut b = warrior(&fixture, OTHER);
    let quest = you_deserve_better();
    let mut now = 1_000;
    MAP.accept(&mut a, CHARACTER, &quest, &mut now);
    MAP.accept(&mut b, OTHER, &quest, &mut now);
    let b_before = saved(&b);
    MAP.step(&mut a, CHARACTER, &quest, 0, &mut now);
    assert_eq!(saved(&b), b_before, "A's talk changed B");
    assert_eq!(counts(&b, YOU_DESERVE_BETTER), [0, 0]);
}

/// The two level gates: (quest, job, completed prerequisites). Each ends with a Talk to its
/// giver after the rank goal.
const GATES: [(u32, i32, &[(u32, &[u32])]); 2] = [
    (MINER_LEVEL_5, MINER, &[]),
    (NINJA_LEVEL_20, NINJA, &[(ICE_TROLL, &[1])]),
];

#[test]
fn level_gates_complete_on_rank_and_pay_coins_once() {
    for (id, job, done) in GATES {
        // `realm level` is wired in the opening branch only, so this fixture keeps it.
        let fixture = Fixture::new();
        let mut game = player(&fixture, CHARACTER, job, done);
        let record = shipped_quest(id);
        let (giver, level) = (
            record.giver_npc_id,
            record.objectives[0]["level"].as_u64().unwrap() as u32,
        );
        let quest = quest(id, &[], giver, job);
        let giver_at = MAP.at(CHARACTER, giver);
        let coins_before = coins(&game);
        let mut now = 1_000;
        MAP.accept(&mut game, CHARACTER, &quest, &mut now);
        assert_eq!(
            counts(&game, id),
            [0, 0],
            "quest {id} starts below the gate"
        );

        // Below the level the giver changes nothing and offers nothing.
        assert_no_offer(
            &mut game,
            &giver_at,
            giver,
            &mut now,
            "the giver below the gate",
        );
        assert_eq!(pending_turnin(&game), None);
        realm_level(&mut game, &giver_at, level - 1, now);
        assert_eq!(counts(&game, id), [0, 0], "one rank short never credits");

        let reached = realm_level(&mut game, &giver_at, level, now + 10);
        assert!(
            objective_complete(&reached, id, 1),
            "quest {id} completes at rank {level}"
        );
        assert_eq!(counts(&game, id), [1, 0], "the return Talk is still open");
        now += 30;
        // The giver's Talk completes the last goal and holds the 49/13.
        MAP.turn_in(&mut game, CHARACTER, &quest, job, &mut now);

        assert!(quest.coins > 0, "quest {id} pays coins");
        assert_eq!(
            coins(&game),
            coins_before + quest.coins,
            "quest {id} pays once"
        );
        assert_packet_changes_nothing(
            &mut game,
            &giver_at,
            &quest_end_reply(id),
            now,
            "a duplicate 49/14",
        );
    }
}

#[test]
fn level_gates_are_offered_only_to_their_job() {
    for (id, _, done) in GATES {
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = player(&fixture, CHARACTER, BRAWLER, done);
        let giver = shipped_quest(id).giver_npc_id;
        let giver_at = MAP.at(CHARACTER, giver);
        let mut now = 1_000;
        assert_no_offer(
            &mut game,
            &giver_at,
            giver,
            &mut now,
            "another job at the giver",
        );
        assert_packet_changes_nothing(
            &mut game,
            &giver_at,
            &quest_reply(id, true),
            now + 10,
            "a forged 49/2",
        );
    }
}

#[test]
fn ice_troll_rampage_completes_at_the_howling_hills_and_unlocks_the_level_gate() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, NINJA, &[]);
    let record = shipped_quest(ICE_TROLL);
    let goal = &record.objectives[0];
    assert_eq!(goal["kind"], "reach_poi");
    let spot: Vec<f32> = goal["position"]
        .as_array()
        .unwrap()
        .iter()
        .map(|axis| axis.as_f64().unwrap() as f32)
        .collect();
    let radius = goal["radius"].as_f64().unwrap() as f32;
    let at = |metres: f32| character_at_for(CHARACTER, 0, [spot[0] + metres, spot[1], spot[2]]);
    let quest = quest(ICE_TROLL, &[], MICHI, NINJA);
    let michi = MAP.at(CHARACTER, MICHI);
    let mut now = 1_000;

    // The level gate waits for this quest: nothing at Michi but the Howling Hills errand.
    let forged_gate = quest_reply(NINJA_LEVEL_20, true);
    assert_packet_changes_nothing(&mut game, &michi, &forged_gate, now, "a forged 49/2");
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    // Outside the radius, or in another zone, credits nothing and changes nothing.
    let mut wrong_zone = at(0.0);
    wrong_zone.zone_id = 2;
    for (refused, label) in [(at(radius + 0.5), "just outside"), (wrong_zone, "zone 2")] {
        let before = saved(&game);
        let moved = game
            .on_location(&refused, now)
            .expect("a position is accepted");
        assert!(!moved.changed, "{label} credited the goal");
        assert_eq!(saved(&game), before, "{label} changed the save");
        now += 10;
    }
    assert_no_offer(&mut game, &michi, MICHI, &mut now, "Michi before the hills");
    assert_eq!(pending_turnin(&game), None);

    let arrived = game
        .on_location(&at(radius - 0.5), now)
        .expect("a position inside is accepted");
    assert!(arrived.changed);
    assert!(objective_complete(&arrived, ICE_TROLL, 1));
    assert_eq!(counts(&game, ICE_TROLL), [1]);
    now += 10;

    let coins_before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &quest, NINJA, &mut now);
    assert!(quest.coins > 0);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(quest_status(&game, ICE_TROLL), QuestStatus::Completed);
    assert_packet_changes_nothing(
        &mut game,
        &michi,
        &quest_end_reply(ICE_TROLL),
        now,
        "a duplicate 49/14",
    );
    // Michi now offers the level gate.
    offer(&mut game, &michi, MICHI, NINJA_LEVEL_20, now + 10);
}

#[test]
fn the_ninja_level_gate_waits_for_ice_troll_rampage() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, NINJA, &[]);
    let michi = MAP.at(CHARACTER, MICHI);
    let now = 1_000;
    // Michi offers the Howling Hills errand; a forged accept of the gate behind it changes
    // nothing.
    offer(&mut game, &michi, MICHI, ICE_TROLL, now);
    assert_packet_changes_nothing(
        &mut game,
        &michi,
        &quest_reply(NINJA_LEVEL_20, true),
        now + 10,
        "a forged 49/2",
    );
}

#[test]
fn sorin_and_eddie_are_drawn() {
    // Both rows are fork spawns (FreeRealms-Legacy FabledRealms.lua:2349 definition 3607 and
    // :2948 definition 4282), so the production visibility path draws them.
    for (file, npc, model, name) in [
        ("snowhill.json", SORIN, 382, 5_456),
        ("wilds.json", EDDIE, 1_456, 401_743),
    ] {
        let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
        let position = content_position(file, npc);
        assert!(
            visibility
                .teleport_targets(0)
                .iter()
                .any(|target| target.guid == npc && target.position == position),
            "the production spawn list holds {npc}"
        );
        let added = visibility
            .update(0, position)
            .expect("a player at the spawn gets the draw set")
            .iter()
            .filter_map(|packet| parse_add_npc(packet).ok())
            .find(|spawn| spawn.actor_id == npc)
            .expect("AddNpc for the row");
        assert_eq!((added.model_id, added.name_string_id), (model, name));
        assert_eq!(visibility.content_key(npc), Some(npc));
    }
}
