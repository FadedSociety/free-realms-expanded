//! T670: zone quests round 2, Cobblestone Village, Farnum Farmstead, Stillwater Crossing and
//! Highroad Junction. Of the 15 rows in `lab/references/quest-lines/wiring/zone-quests-matched/`
//! (cobblestone-village, farnum-farmstead, stillwater-crossing, highroad-junction `.json`),
//! two run end to end on this tip and are enabled: Simone's `Chef: Delivery Derailed` (3050007,
//! quest-lines/quests.json q3050007; find Jonelle) and Helena's `Chef: Gavin Gone Missing`
//! (3050016, q3050016; find Gavin Greenpetal). Each target is a drawn `legacy_data` row, so the
//! Talk at the found NPC completes the goal and that NPC holds the end screen. Every other row
//! stays off (`the_other_round_two_rows_stay_off_with_a_blocker` re-checks each blocker).
//!
//! Every packet goes through the production router via the shared walkers in
//! `tests/support/npc_quest_chain.rs`: 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with the
//! production reader), 49/2 accept, the Talk that credits the goal and holds the 49/13, and the
//! 26/6 -> 49/13 -> 49/14 turn-in. Job gate, card ids, end text and payout come from the shipped
//! records, never from literals here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`,
//! 49/13 `QuestEndPacket::ReadFields` `0x00c7b990`, 49/14 `0x00a864c0`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, counts, pending_turnin, profile_xp, quest_end_reply,
    quest_reply, quest_status, saved, talk, zone_player_with_quests,
};
use osfr_tools::{gameplay::Gameplay, quest_engine::QuestStatus};
use quest_records::shipped_quest;

const CHEF: i32 = 45;
const SIMONE: u64 = 9_000_000_467;
const JONELLE: u64 = 9_000_000_120;
const HELENA: u64 = 9_000_000_103;
const GAVIN: u64 = 9_000_000_096;
const DELIVERY_DERAILED: u32 = 3_050_007;
const GAVIN_GONE_MISSING: u32 = 3_050_016;
/// "Become a Chef", the enabled Simone quest that stands in for her disabled first quest.
const BECOME_A_CHEF: u32 = 3_050_034;

/// Simone's row is in `sanctuary.json`, Jonelle's in `cobblestone_village.json`.
const COBBLESTONE: ZoneQuests = ZoneQuests {
    default_file: "sanctuary.json",
    files: &[(JONELLE, "cobblestone_village.json")],
    runtime: &[],
};
/// Helena and Gavin share `stillwater_crossing.json`.
const STILLWATER: ZoneQuests = ZoneQuests {
    default_file: "stillwater_crossing.json",
    files: &[],
    runtime: &[],
};

fn delivery_derailed() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(DELIVERY_DERAILED), SIMONE, JONELLE, CHEF)
}

fn gavin_gone_missing() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(GAVIN_GONE_MISSING), HELENA, GAVIN, CHEF)
}

/// A Chef with `done` quests completed.
fn chef(fixture: &Fixture, active: i32, done: &[(u32, &[u32])]) -> Gameplay {
    zone_player_with_quests(fixture, CHARACTER, active, &[ADVENTURER, CHEF], done)
}

fn has_quest(game: &Gameplay, id: u32) -> bool {
    game.state()
        .npc_quests
        .as_ref()
        .unwrap()
        .quests
        .contains_key(&id)
}

#[test]
fn simone_offers_delivery_derailed_after_become_a_chef_and_jonelle_pays_it_once() {
    let record = shipped_quest(DELIVERY_DERAILED);
    assert!(record.enabled);
    assert_eq!(record.job, Some(CHEF), "a Chef-only quest");
    assert_eq!(record.prerequisite_quest_ids, [BECOME_A_CHEF]);
    let quest = delivery_derailed();
    assert!(quest.profile_xp > 0, "the reward is authored, not empty");

    let fixture = Fixture::new();
    let mut game = chef(&fixture, CHEF, &[(BECOME_A_CHEF, &[1])]);
    let mut now = 1_000;
    let xp_before = profile_xp(&game, CHEF);
    COBBLESTONE.run(&mut game, CHARACTER, &quest, CHEF, &mut now);
    assert_eq!(counts(&game, quest.id), [1]);
    assert_eq!(profile_xp(&game, CHEF), xp_before + quest.profile_xp);

    // A replayed hand-in and accept pay nothing, and neither NPC offers it again.
    let jonelle = COBBLESTONE.at(CHARACTER, JONELLE);
    let (paid_coins, paid_xp) = (coins(&game), profile_xp(&game, CHEF));
    assert_packet_changes_nothing(
        &mut game,
        &jonelle,
        &quest_end_reply(DELIVERY_DERAILED),
        now,
        "a replayed 49/14",
    );
    let simone = COBBLESTONE.at(CHARACTER, SIMONE);
    assert_packet_changes_nothing(
        &mut game,
        &simone,
        &quest_reply(DELIVERY_DERAILED, true),
        now + 5,
        "a replayed 49/2",
    );
    assert_no_offer(&mut game, &simone, SIMONE, &mut now, "Simone after");
    assert_no_offer(&mut game, &jonelle, JONELLE, &mut now, "Jonelle after");
    assert_eq!(
        (coins(&game), profile_xp(&game, CHEF)),
        (paid_coins, paid_xp)
    );
}

#[test]
fn helena_offers_gavin_gone_missing_and_gavin_pays_it_once() {
    let record = shipped_quest(GAVIN_GONE_MISSING);
    assert!(record.enabled);
    assert_eq!(record.job, Some(CHEF), "a Chef-only quest");
    let quest = gavin_gone_missing();
    assert!(quest.profile_xp > 0, "the reward is authored, not empty");

    let fixture = Fixture::new();
    let mut game = chef(&fixture, CHEF, &[]);
    let mut now = 1_000;
    let xp_before = profile_xp(&game, CHEF);
    STILLWATER.run(&mut game, CHARACTER, &quest, CHEF, &mut now);
    assert_eq!(counts(&game, quest.id), [1]);
    assert_eq!(profile_xp(&game, CHEF), xp_before + quest.profile_xp);

    let gavin = STILLWATER.at(CHARACTER, GAVIN);
    let helena = STILLWATER.at(CHARACTER, HELENA);
    let (paid_coins, paid_xp) = (coins(&game), profile_xp(&game, CHEF));
    assert_packet_changes_nothing(
        &mut game,
        &gavin,
        &quest_end_reply(GAVIN_GONE_MISSING),
        now,
        "a replayed 49/14",
    );
    assert_no_offer(&mut game, &helena, HELENA, &mut now, "Helena after");
    assert_no_offer(&mut game, &gavin, GAVIN, &mut now, "Gavin after");
    assert_eq!(
        (coins(&game), profile_xp(&game, CHEF)),
        (paid_coins, paid_xp)
    );
}

#[test]
fn every_refusal_changes_nothing() {
    let fixture = Fixture::new();
    let mut now = 1_000;
    let (derailed, missing) = (delivery_derailed(), gavin_gone_missing());

    // Another job active: neither giver offers, a forged accept adds nothing.
    let cases = [
        (
            &COBBLESTONE,
            SIMONE,
            DELIVERY_DERAILED,
            "Simone as an Adventurer",
        ),
        (
            &STILLWATER,
            HELENA,
            GAVIN_GONE_MISSING,
            "Helena as an Adventurer",
        ),
    ];
    for (zone, giver, id, label) in cases {
        let mut game = zone_player_with_quests(
            &fixture,
            CHARACTER,
            ADVENTURER,
            &[ADVENTURER, CHEF],
            &[(BECOME_A_CHEF, &[1])],
        );
        let at = zone.at(CHARACTER, giver);
        assert_no_offer(&mut game, &at, giver, &mut now, label);
        assert_packet_changes_nothing(&mut game, &at, &quest_reply(id, true), now, label);
        assert!(!has_quest(&game, id), "{label}: quest {id} must not exist");
    }

    // A Chef who has not done Become a Chef: a forged accept at Simone adds nothing.
    let mut game = chef(&fixture, CHEF, &[]);
    let simone = COBBLESTONE.at(CHARACTER, SIMONE);
    assert_packet_changes_nothing(
        &mut game,
        &simone,
        &quest_reply(DELIVERY_DERAILED, true),
        now,
        "no Become a Chef",
    );
    assert!(!has_quest(&game, DELIVERY_DERAILED), "no Become a Chef");

    // The found NPC's Talk without the quest is inert, and an early hand-in settles nothing.
    for (zone, quest, target, label) in [
        (&COBBLESTONE, &derailed, JONELLE, "Jonelle"),
        (&STILLWATER, &missing, GAVIN, "Gavin"),
    ] {
        let mut game = chef(&fixture, CHEF, &[(BECOME_A_CHEF, &[1])]);
        let before = saved(&game);
        let talked = talk(&mut game, &zone.at(CHARACTER, target), target, now);
        assert!(!talked.changed, "{label}'s Talk without the quest is inert");
        assert_eq!(saved(&game), before, "{label}");

        zone.accept(&mut game, CHARACTER, quest, &mut now);
        let at = zone.at(CHARACTER, target);
        assert_packet_changes_nothing(
            &mut game,
            &at,
            &quest_end_reply(quest.id),
            now,
            "an early 49/14",
        );
        assert_eq!(pending_turnin(&game), None);
        assert_eq!(counts(&game, quest.id), [0]);
    }
}

#[test]
fn progress_survives_a_reconnect_and_players_are_independent() {
    let fixture = Fixture::new();
    let quest = gavin_gone_missing();
    let mut a = chef(&fixture, CHEF, &[]);
    let mut now = 1_000;
    STILLWATER.accept(&mut a, CHARACTER, &quest, &mut now);

    // A save and load before the Talk keeps the open goal; another player is untouched.
    let mut a = fixture.reload_serialized(&a);
    assert_eq!(quest_status(&a, GAVIN_GONE_MISSING), QuestStatus::Active);
    assert_eq!(counts(&a, GAVIN_GONE_MISSING), [0]);
    let other = CHARACTER + 1;
    let b = zone_player_with_quests(&fixture, other, CHEF, &[ADVENTURER, CHEF], &[]);
    let b_before = saved(&b);

    STILLWATER.turn_in(&mut a, CHARACTER, &quest, CHEF, &mut now);
    assert_eq!(saved(&b), b_before, "A's run changed B");

    let a = fixture.reload_serialized(&a);
    assert_eq!(quest_status(&a, GAVIN_GONE_MISSING), QuestStatus::Completed);
}

/// The other 13 rows of the four matched lists, less the two already enabled (3000021, 3030008)
/// and the Brawler 3030011 that `card_duelist_and_brawler_quests_stay_disabled` guards. Each
/// check fails when its blocker lifts, so the next builder re-checks the row before wiring it.
#[test]
fn the_other_round_two_rows_stay_off_with_a_blocker() {
    let off = |id: u32| {
        let quest = shipped_quest(id);
        assert!(!quest.enabled, "quest {id} is enabled, not blocked");
        quest
    };
    // Opening-arc records behind a disabled predecessor (the tutorial arc runs in its own stage
    // engine; a second copy would double an opening stage).
    for id in [3_000_011, 3_000_015] {
        let blocked = off(id)
            .prerequisite_quest_ids
            .iter()
            .any(|p| !shipped_quest(*p).enabled);
        assert!(blocked, "{id}: its prerequisites are enabled now");
    }
    // Stubs with no goal: a mechanic (artifact clicks, bixie and hooligan kills, a basket and
    // five pickups, a prank and balls to nab) or a fact (no source gives the goal) is missing.
    for id in [
        3_000_012, 3_000_018, 3_010_010, 3_050_006, 3_050_017, 3_060_018, 3_070_002, 3_140_013,
    ] {
        assert!(
            off(id).objectives.is_empty(),
            "{id} grew objectives; re-check it"
        );
    }
}
