//! T675: zone quests round 2, Snowhill. Of the five rows in
//! `lab/references/quest-lines/wiring/zone-quests-matched/snowhill.json` three are wired: Blacksmith
//! "Bat Fighter" (3020007) and "Reach Level 5" (3020008) and Warrior "Asking Old Folks" (3150019).
//! The other two stay off behind a blocker this suite watches: "Warrior Weapon" (3020006) needs a
//! Valiant Guardian Cudgel recipe the recipe book does not hold, and "Steele is Useful" (3150020)
//! needs the weapon pieces and a Chillblaine fight in Frostfang Caverns, which has no encounter.
//!
//! Every walk drives the production paths in `tests/support/npc_quest_chain.rs` (`ZoneQuests`):
//! the 26/8 -> 26/10 -> 26/6 offer, the 49/2 accept, Talk steps, `realm level` for the rank goal,
//! the Talk that holds the 49/13, the 26/6 that releases it and the 49/14 turn-in. Givers, card
//! ids, end texts and payouts are read from the shipped records, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/13 `QuestEndPacket::ReadFields`
//! `0x00c7b990`, 49/9 reader `0x00c7af60` and 49/10 reader `0x00c7b070` for the rank goal (T361).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer, assert_packet_changes_nothing,
    content_dir, counts, objective_complete, offer, pending_turnin, profile_xp, quest_end_reply,
    quest_reply, quest_status, read_json, realm_level, saved, zone_player_with_quests,
};
use osfr_tools::{gameplay::Gameplay, quest_engine::QuestStatus, world::NpcVisibility};
use quest_records::shipped_quest;

const BRODY: u64 = 9_000_000_033;
const DERP: u64 = 9_000_000_069;
const WHITTINGTON: u64 = 9_000_000_205;
const STEELE: u64 = 9_000_000_208;
const MILO: u64 = 9_500_000_756;
const SILVERSNOW: u64 = 9_500_000_768;

const BLACKSMITH: i32 = 16;
const WARRIOR: i32 = 32;
const BRAWLER: i32 = 43;

const CURIOUS_CANCELATION: u32 = 3_020_005;
const WARRIOR_WEAPON: u32 = 3_020_006;
const BAT_FIGHTER: u32 = 3_020_007;
const REACH_LEVEL_5: u32 = 3_020_008;
const WARRIOR_W5: u32 = 3_150_005;
const YOU_DESERVE_BETTER: u32 = 3_150_007;
const WARRIOR_W16: u32 = 3_150_016;
const FASTVI_LEGEND: u32 = 3_150_018;
const ASKING_OLD_FOLKS: u32 = 3_150_019;
const STEELE_IS_USEFUL: u32 = 3_150_020;
const OTHER: u64 = CHARACTER + 1;

/// Snowhill rows by default; Sir Whittington's row lives in the Sanctuary file.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "snowhill.json",
    files: &[(WHITTINGTON, "sanctuary.json")],
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

fn blacksmith(fixture: &Fixture, guid: u64) -> Gameplay {
    // Bat Fighter follows Warrior Weapon again (the cudgel recipe is back).
    player(
        fixture,
        guid,
        BLACKSMITH,
        &[(CURIOUS_CANCELATION, &[1, 1]), (WARRIOR_WEAPON, &[1, 1, 1])],
    )
}

fn warrior(fixture: &Fixture, guid: u64) -> Gameplay {
    player(
        fixture,
        guid,
        WARRIOR,
        &[
            (WARRIOR_W5, &[1]),
            (YOU_DESERVE_BETTER, &[1, 1]),
            (WARRIOR_W16, &[1, 1]),
        ],
    )
}

fn bat_fighter() -> ZoneQuest {
    quest(BAT_FIGHTER, &[], DERP, BLACKSMITH)
}

/// Silversnow and Milo speak no bound line (their rows have no greeting), so the 26/3 body is
/// the stock "Hello!" (139503, rc-0136), never 0.
fn asking_old_folks() -> ZoneQuest {
    quest(
        ASKING_OLD_FOLKS,
        &[(SILVERSNOW, 139_503), (MILO, 139_503)],
        STEELE,
        WARRIOR,
    )
}

#[test]
fn bat_fighter_sends_the_blacksmith_to_derp_and_pays_once() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = blacksmith(&fixture, CHARACTER);
    let quest = bat_fighter();
    let (coins_before, xp_before) = (npc_quest_chain::coins(&game), profile_xp(&game, BLACKSMITH));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, BAT_FIGHTER), [0]);

    // Brody is the giver, not the hand-in: his Talk credits nothing and holds no end screen.
    let brody = MAP.at(CHARACTER, BRODY);
    assert_no_offer(&mut game, &brody, BRODY, &mut now, "Brody mid-quest");
    assert_packet_changes_nothing(
        &mut game,
        &brody,
        &quest_end_reply(BAT_FIGHTER),
        now,
        "an early 49/14",
    );

    // Derp's Talk completes the only goal, holds the end screen and pays on the 49/14.
    MAP.turn_in(&mut game, CHARACTER, &quest, BLACKSMITH, &mut now);
    assert_eq!(counts(&game, BAT_FIGHTER), [1]);
    assert!(quest.coins > 0 && quest.profile_xp > 0);
    assert_eq!(npc_quest_chain::coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, BLACKSMITH), xp_before + quest.profile_xp);
    assert_eq!(quest_status(&game, BAT_FIGHTER), QuestStatus::Completed);
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, DERP),
        &quest_end_reply(BAT_FIGHTER),
        now,
        "a duplicate 49/14",
    );
    // Brody now offers the level gate behind it.
    offer(&mut game, &brody, BRODY, REACH_LEVEL_5, now + 10);
}

#[test]
fn bat_fighter_waits_for_the_errand_before_it_and_the_job() {
    let fixture = Fixture::minimal(CHARACTER);
    let brody = MAP.at(CHARACTER, BRODY);
    let forged = quest_reply(BAT_FIGHTER, true);
    let mut now = 1_000;

    // A Blacksmith who has not finished Curious Cancelation is offered nothing, and a forged
    // accept changes nothing.
    let mut fresh = player(&fixture, CHARACTER, BLACKSMITH, &[]);
    assert_no_offer(&mut fresh, &brody, BRODY, &mut now, "a fresh Blacksmith");
    assert_packet_changes_nothing(&mut fresh, &brody, &forged, now + 10, "a forged 49/2");

    // The prerequisite done but another job active: nothing is offered either.
    let mut brawler = player(
        &fixture,
        CHARACTER,
        BRAWLER,
        &[(CURIOUS_CANCELATION, &[1, 1])],
    );
    assert_no_offer(&mut brawler, &brody, BRODY, &mut now, "a Brawler at Brody");
    assert_packet_changes_nothing(&mut brawler, &brody, &forged, now + 10, "a forged 49/2");
}

#[test]
fn reach_level_5_completes_on_rank_and_pays_coins_once() {
    // `realm level` is wired in the opening branch only, so this fixture keeps it.
    let fixture = Fixture::new();
    let mut game = player(
        &fixture,
        CHARACTER,
        BLACKSMITH,
        &[(CURIOUS_CANCELATION, &[1, 1]), (BAT_FIGHTER, &[1])],
    );
    let level = shipped_quest(REACH_LEVEL_5).job_level_goal();
    let quest = quest(REACH_LEVEL_5, &[], BRODY, BLACKSMITH);
    let brody = MAP.at(CHARACTER, BRODY);
    let coins_before = npc_quest_chain::coins(&game);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, REACH_LEVEL_5), [0, 0]);

    // Below the level Brody changes nothing and offers nothing.
    assert_no_offer(&mut game, &brody, BRODY, &mut now, "Brody below the gate");
    assert_eq!(pending_turnin(&game), None);
    realm_level(&mut game, &brody, level - 1, now);
    assert_eq!(
        counts(&game, REACH_LEVEL_5),
        [0, 0],
        "one rank short never credits"
    );

    let reached = realm_level(&mut game, &brody, level, now + 10);
    assert!(objective_complete(&reached, REACH_LEVEL_5, 1));
    assert_eq!(counts(&game, REACH_LEVEL_5), [1, 0], "the return is open");
    now += 30;
    // Brody's Talk completes the last goal and holds the 49/13.
    MAP.turn_in(&mut game, CHARACTER, &quest, BLACKSMITH, &mut now);

    assert!(quest.coins > 0);
    assert_eq!(npc_quest_chain::coins(&game), coins_before + quest.coins);
    assert_eq!(quest_status(&game, REACH_LEVEL_5), QuestStatus::Completed);
    assert_packet_changes_nothing(
        &mut game,
        &brody,
        &quest_end_reply(REACH_LEVEL_5),
        now,
        "a duplicate 49/14",
    );
}

#[test]
fn reach_level_5_waits_for_bat_fighter_and_the_blacksmith_job() {
    let fixture = Fixture::minimal(CHARACTER);
    let brody = MAP.at(CHARACTER, BRODY);
    let forged = quest_reply(REACH_LEVEL_5, true);
    let mut now = 1_000;

    // Brody offers the errand first; a forged accept of the gate behind it changes nothing.
    let mut game = blacksmith(&fixture, CHARACTER);
    offer(&mut game, &brody, BRODY, BAT_FIGHTER, now);
    assert_packet_changes_nothing(&mut game, &brody, &forged, now + 10, "a forged 49/2");

    // With the errand done another job is offered nothing.
    let mut brawler = player(
        &fixture,
        CHARACTER,
        BRAWLER,
        &[(CURIOUS_CANCELATION, &[1, 1]), (BAT_FIGHTER, &[1])],
    );
    assert_no_offer(&mut brawler, &brody, BRODY, &mut now, "a Brawler at Brody");
    assert_packet_changes_nothing(&mut brawler, &brody, &forged, now + 10, "a forged 49/2");
}

#[test]
fn asking_old_folks_walks_silversnow_milo_steele() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = warrior(&fixture, CHARACTER);
    let quest = asking_old_folks();
    let (coins_before, xp_before) = (npc_quest_chain::coins(&game), profile_xp(&game, WARRIOR));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    // The Talks are ordered: Milo and Steele before Silversnow credit nothing and change nothing.
    let milo = MAP.at(CHARACTER, MILO);
    assert_no_offer(&mut game, &milo, MILO, &mut now, "Milo before Silversnow");
    assert_eq!(counts(&game, ASKING_OLD_FOLKS), [0, 0, 0]);
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, STEELE),
        &quest_end_reply(ASKING_OLD_FOLKS),
        now,
        "an early 49/14",
    );

    for index in 0..quest.steps.len() {
        MAP.step(&mut game, CHARACTER, &quest, index, &mut now);
    }
    assert_eq!(pending_turnin(&game), None, "the hand-in is Steele's");
    // Steele's Talk completes the last goal and holds the end screen for him.
    MAP.turn_in(&mut game, CHARACTER, &quest, WARRIOR, &mut now);
    assert_eq!(counts(&game, ASKING_OLD_FOLKS), [1, 1, 1]);

    assert!(quest.coins > 0 && quest.profile_xp > 0);
    assert_eq!(npc_quest_chain::coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, WARRIOR), xp_before + quest.profile_xp);
    assert_eq!(
        quest_status(&game, ASKING_OLD_FOLKS),
        QuestStatus::Completed
    );
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, STEELE),
        &quest_end_reply(ASKING_OLD_FOLKS),
        now,
        "a duplicate 49/14",
    );
}

/// Steele also gives You Deserve Better; a Warrior who skipped it still hands this errand in
/// at him, and the Talk completes the goal instead of offering the other quest.
#[test]
fn steele_takes_the_hand_in_while_you_deserve_better_is_still_open() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(
        &fixture,
        CHARACTER,
        WARRIOR,
        &[(WARRIOR_W5, &[1]), (WARRIOR_W16, &[1, 1])],
    );
    let quest = asking_old_folks();
    let mut now = 1_000;
    MAP.run(&mut game, CHARACTER, &quest, WARRIOR, &mut now);
    assert_eq!(
        quest_status(&game, ASKING_OLD_FOLKS),
        QuestStatus::Completed
    );
}

#[test]
fn asking_old_folks_waits_for_the_level_gate_and_the_warrior_job() {
    let fixture = Fixture::minimal(CHARACTER);
    let whittington = MAP.at(CHARACTER, WHITTINGTON);
    let forged = quest_reply(ASKING_OLD_FOLKS, true);
    let mut now = 1_000;

    // A Warrior who has not done the level gate is offered the gate, and a forged accept of the
    // old-folks errand behind it changes nothing.
    let mut fresh = player(&fixture, CHARACTER, WARRIOR, &[]);
    offer(&mut fresh, &whittington, WHITTINGTON, WARRIOR_W16, now);
    assert_packet_changes_nothing(&mut fresh, &whittington, &forged, now + 10, "a forged 49/2");

    // The gate done but another job active: nothing is offered.
    let mut brawler = player(&fixture, CHARACTER, BRAWLER, &[(WARRIOR_W16, &[1, 1])]);
    assert_no_offer(
        &mut brawler,
        &whittington,
        WHITTINGTON,
        &mut now,
        "a Brawler at Whittington",
    );
    assert_packet_changes_nothing(
        &mut brawler,
        &whittington,
        &forged,
        now + 10,
        "a forged 49/2",
    );
}

#[test]
fn asking_old_folks_survives_a_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = warrior(&fixture, CHARACTER);
    let quest = asking_old_folks();
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    MAP.step(&mut game, CHARACTER, &quest, 0, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, ASKING_OLD_FOLKS), QuestStatus::Active);
    assert_eq!(counts(&game, ASKING_OLD_FOLKS), [1, 0, 0]);
    MAP.step(&mut game, CHARACTER, &quest, 1, &mut now);
    MAP.turn_in(&mut game, CHARACTER, &quest, WARRIOR, &mut now);
    assert_eq!(
        quest_status(&game, ASKING_OLD_FOLKS),
        QuestStatus::Completed
    );
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = warrior(&fixture, CHARACTER);
    let mut b = warrior(&fixture, OTHER);
    let quest = asking_old_folks();
    let mut now = 1_000;
    MAP.accept(&mut a, CHARACTER, &quest, &mut now);
    MAP.accept(&mut b, OTHER, &quest, &mut now);
    let b_before = saved(&b);
    MAP.step(&mut a, CHARACTER, &quest, 0, &mut now);
    assert_eq!(saved(&b), b_before, "A's talk changed B");
    assert_eq!(counts(&b, ASKING_OLD_FOLKS), [0, 0, 0]);
}

#[test]
fn the_talk_targets_are_drawn_by_the_production_spawn_list() {
    // Derp, Silversnow and Milo are fork spawns (FreeRealms-Legacy FabledRealms.lua:2029, :2321
    // and :2324) and Steele is a shipped extra spawn, so the visibility path draws all four.
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let targets = visibility.teleport_targets(0);
    for npc in [DERP, SILVERSNOW, MILO, STEELE] {
        assert!(
            targets.iter().any(|target| target.guid == npc),
            "the production spawn list holds {npc}"
        );
    }
}

/// Warrior Weapon is the forge step before Bat Fighter. It waited for the recipe book to hold the
/// Valiant Guardian Cudgel; the recipe was reconstructed (2026-11-07, `recipes.json`, same family
/// as the Stalwart Protector Hammer), so the quest is on again and back in front of Bat Fighter.
#[test]
fn warrior_weapon_stays_off_until_the_cudgel_can_be_forged() {
    let book = read_json(&content_dir().join("recipes.json"));
    assert!(
        book["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["name"] == "Valiant Guardian Cudgel"),
        "the recipe book holds the cudgel"
    );
    assert!(shipped_quest(WARRIOR_WEAPON).enabled);
    assert_eq!(
        shipped_quest(BAT_FIGHTER).prerequisite_quest_ids,
        [WARRIOR_WEAPON]
    );
}

/// Steele is Useful needs Fastvi's weapon pieces, four named Warrior moves on Chillblaine (quest
/// table: Fight Chillblaine; Use Kick, Spinning Blade, Quake and Commanding Shout on him) and a
/// Chillblaine fight in Frostfang Caverns. Fix F5 (2026-11-11) gave the caverns that fight: the
/// door runs `fights/14027.json`, whose main goal is Chillblaine himself. The quest still stays
/// off for what is missing, and each missing piece is asserted here, so this test fails (and asks
/// for the re-check) the day one of them lands:
/// - its record has no steps, and no item row or pickup places the pieces of Fastvi's weapon;
/// - the 2014 kits have Quake and Commanding Shout but no "Kick" and no "Spinning Blade".
/// The game-table lesson (3150018) has no caverns step: it is the "Weed Killing!" tower-defense
/// table (text 15135) next to Sir Whittington. L1207 put it on once `tower_defense.json` held the
/// table: game 353, "Weed Killing! - Easy" (text 14909), the quest's own goal line. Asking Old
/// Folks still follows W16 while Steele is Useful is off.
#[test]
fn steele_is_useful_stays_off_until_frostfang_caverns_has_an_encounter() {
    const CAVERNS_FIGHT: u64 = 14027;
    const CHILLBLAINE: u64 = 950_060;
    const WEED_KILLING_EASY: u64 = 353;
    assert!(!shipped_quest(STEELE_IS_USEFUL).enabled);
    let fastvi = shipped_quest(FASTVI_LEGEND);
    assert!(fastvi.enabled, "Legend of Fastville went live in L1207");

    // The caverns part is ready: the record maps onto the fight its door runs, and that fight's
    // main goal is Chillblaine.
    let caverns = read_json(&content_dir().join("dungeons/frostfang_caverns.json"));
    let goals = caverns["records"][0]["record"]["encounters"][0]["goals"]
        .as_array()
        .unwrap();
    assert!(
        goals
            .iter()
            .any(|g| g["fight_id"] == CAVERNS_FIGHT && g["goal"] == "main"),
        "Frostfang Caverns maps onto fight {CAVERNS_FIGHT}: {goals:?}"
    );
    let entrances = read_json(&content_dir().join("entrances.json"));
    let door = (entrances["entrances"].as_array().unwrap().iter())
        .find(|row| row["id"] == "frostfang_caverns")
        .expect("the caverns door");
    assert_eq!(door["fight"], CAVERNS_FIGHT, "the door runs the caverns fight");
    let fight = read_json(&content_dir().join("fights/14027.json"));
    assert_eq!(fight["objectives"][0]["id"], "main");
    assert_eq!(fight["objectives"][0]["target"]["actor"], "boss");
    let boss = fight["actors"]["boss"]["definition"].as_str().unwrap();
    assert_eq!(fight["kinds"][boss]["definition_id"], CHILLBLAINE);

    // What is still missing.
    assert!(
        shipped_quest(STEELE_IS_USEFUL).objectives.is_empty(),
        "Steele is Useful has steps now: re-check quest 3150020 (weapon pieces, moves, caverns)"
    );
    let kits = read_json(&content_dir().join("weapon_kits.json"));
    let moves: Vec<&str> = (kits["abilities"].as_object().unwrap().values())
        .filter_map(|ability| ability["name"].as_str())
        .collect();
    assert!(moves.contains(&"Quake") && moves.contains(&"Commanding Shout"));
    assert!(
        !moves.contains(&"Kick") && !moves.contains(&"Spinning Blade"),
        "the kits have Kick or Spinning Blade now: re-check quest 3150020"
    );
    // Fastville's table step names a game the tower-defense table holds.
    let tables = read_json(&content_dir().join("tower_defense.json"));
    assert!(
        (tables["games"].as_array().unwrap().iter()).any(|game| game["game_id"] == WEED_KILLING_EASY),
        "tower_defense.json holds Weed Killing! - Easy ({WEED_KILLING_EASY})"
    );
    let record = read_json(&content_dir().join("quests/warrior.json"));
    let fastvi_row = (record["records"].as_array().unwrap().iter())
        .find(|row| row["id"] == u64::from(FASTVI_LEGEND))
        .expect("the Fastville record");
    assert!(
        (fastvi_row["objectives"].as_array().unwrap().iter())
            .any(|goal| goal["kind"] == "finish_minigame" && goal["minigame_id"] == WEED_KILLING_EASY),
        "Legend of Fastville's table step is game {WEED_KILLING_EASY}"
    );
    assert_eq!(
        shipped_quest(ASKING_OLD_FOLKS).prerequisite_quest_ids,
        [WARRIOR_W16]
    );
}
