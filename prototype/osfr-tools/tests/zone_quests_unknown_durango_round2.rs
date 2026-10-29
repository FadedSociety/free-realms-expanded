//! T677: zone quests round 2 for the rows with no settled zone and for Durango Zoo
//! (`lab/references/quest-lines/wiring/zone-quests-matched/{unknown,durango-zoo}.json`). One of
//! the 15 `wireable_by_match` rows runs end to end on this tip and is enabled: Silverray's
//! `Miner: Minor Mix-Up` (3100021, quest-lines/quests.json q3100021), which sends the player to
//! Ramberry twice and back to Silverray. The other fourteen need a mechanic or an NPC the server
//! lacks (the Brawler pair by the lead's decision); `the_other_rows_stay_off` pins them so a row
//! that gains objectives has to be re-checked (reasons in `notes/Autopilot/T677.md`).
//!
//! Every packet goes through the production router (`Gameplay::handle_in_world`) via the shared
//! walkers in `tests/support/npc_quest_chain.rs`: 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with
//! the production reader), 49/2 accept, the Talk steps and the 26/6 -> 49/13 -> 49/14 turn-in. The
//! job gate, giver, card ids, end text and payout are read from the shipped record, never pinned.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`, 49/13
//! `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`, end text `18..22`,
//! one dword, then the reward bundle `0x008e7930`), 49/14 `0x00a864c0`, 26/3 payload reader
//! `0x00a9ef10` (first dword = the dialogue body).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, counts, dialogue_body, objective_complete,
    pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status, saved,
    zone_player_with_quests,
};
use osfr_tools::{gameplay::Gameplay, quest_engine::QuestStatus};
use quest_records::{as_u32, shipped_quest};

const MINER: i32 = 14;
const SILVERRAY: u64 = 9_000_000_203;
const RAMBERRY: u64 = 9_500_000_059;
const MIX_UP: u32 = 3_100_021;
/// Silverray's ungated Miner rank-20 gate, seeded done so his menu offers nothing else.
const RANK_TWENTY: u32 = 3_100_023;
/// Silverray's errand that sends the player to him: the quest's prerequisite.
const ERRAND: u32 = 3_100_020;
/// Both rows live in `blackspore_swamp.json`.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "blackspore_swamp.json",
    files: &[],
    runtime: &[],
};

/// Silverray gives it and takes it in; Ramberry is the two steps, never the turn-in.
fn mix_up() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(MIX_UP), SILVERRAY, SILVERRAY, MINER)
}

/// A Miner who finished the errand, plus whatever else `done` seeds.
fn miner(fixture: &Fixture, done: &[(u32, &[u32])]) -> Gameplay {
    zone_player_with_quests(fixture, CHARACTER, MINER, &[ADVENTURER, MINER], done)
}

#[test]
fn minor_mix_up_runs_through_ramberry_twice_and_pays_once() {
    let record = shipped_quest(MIX_UP);
    assert!(record.enabled);
    assert_eq!(record.job, Some(MINER), "a Miner-only quest");
    assert_eq!(record.prerequisite_quest_ids, [ERRAND]);
    let quest = mix_up();
    assert!(
        quest.coins > 0 && quest.profile_xp > 0,
        "the reward is authored"
    );
    let first_line = as_u32(&record.objectives[0]["dialogue_string_id"]).expect("Ramberry's line");

    let fixture = Fixture::new();
    let mut game = miner(&fixture, &[(ERRAND, &[1]), (RANK_TWENTY, &[1])]);
    let (silverray, ramberry) = (MAP.at(CHARACTER, SILVERRAY), MAP.at(CHARACTER, RAMBERRY));
    let mut now = 1_000;

    // Ramberry has nothing to say about it before the quest is taken.
    let before = saved(&game);
    assert!(
        !MAP.talk(&mut game, CHARACTER, RAMBERRY, now).changed,
        "Ramberry's Talk without the quest is inert"
    );
    assert_eq!(saved(&game), before);

    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, MIX_UP), [0, 0, 0]);

    // Silverray has nothing to take in yet: his Talk and an early 49/14 change nothing.
    assert_no_offer(
        &mut game,
        &silverray,
        SILVERRAY,
        &mut now,
        "before Ramberry",
    );
    let early = quest_end_reply(MIX_UP);
    assert_packet_changes_nothing(&mut game, &silverray, &early, now, "an early 49/14");

    // One click credits one step, in order; Ramberry is not the turn-in.
    let first = MAP.talk(&mut game, CHARACTER, RAMBERRY, now);
    assert!(objective_complete(&first, MIX_UP, 1));
    assert_eq!(dialogue_body(&first), first_line, "Ramberry's line");
    assert_eq!(counts(&game, MIX_UP), [1, 0, 0]);
    now += 10;
    let second = MAP.talk(&mut game, CHARACTER, RAMBERRY, now);
    assert!(objective_complete(&second, MIX_UP, 2));
    assert_eq!(counts(&game, MIX_UP), [1, 1, 0]);
    assert_eq!(pending_turnin(&game), None, "Ramberry is not the turn-in");

    // A third click credits nothing, and the step progress survives a save and load.
    now += 10;
    assert_no_offer(
        &mut game,
        &ramberry,
        RAMBERRY,
        &mut now,
        "Ramberry a third time",
    );
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(counts(&game, MIX_UP), [1, 1, 0]);

    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, MINER));
    MAP.turn_in(&mut game, CHARACTER, &quest, MINER, &mut now);
    assert_eq!(quest_status(&game, MIX_UP), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, MINER), xp_before + quest.profile_xp);

    // A replayed hand-in and accept pay and add nothing.
    let after = coins(&game);
    let duplicate = quest_end_reply(MIX_UP);
    assert_packet_changes_nothing(&mut game, &silverray, &duplicate, now, "a duplicate 49/14");
    let replay = quest_reply(MIX_UP, true);
    assert_packet_changes_nothing(&mut game, &silverray, &replay, now + 5, "a replayed 49/2");
    assert_eq!(coins(&game), after);
}

#[test]
fn the_wrong_job_and_a_missing_errand_change_nothing() {
    let fixture = Fixture::new();
    let mut now = 1_000;
    let silverray = MAP.at(CHARACTER, SILVERRAY);

    // Another job with the errand done: nothing is offered.
    let mut other =
        zone_player_with_quests(&fixture, CHARACTER, ADVENTURER, &[], &[(ERRAND, &[1])]);
    assert_no_offer(&mut other, &silverray, SILVERRAY, &mut now, "another job");

    // Both cases: a forged 49/2 adds nothing and the quest never exists.
    let cases = [("another job", other), ("no errand", miner(&fixture, &[]))];
    for (label, mut game) in cases {
        let before = saved(&game);
        assert_packet_changes_nothing(
            &mut game,
            &silverray,
            &quest_reply(MIX_UP, true),
            now,
            label,
        );
        assert_eq!(saved(&game), before);
        assert!(
            !game
                .state()
                .npc_quests
                .as_ref()
                .unwrap()
                .quests
                .contains_key(&MIX_UP),
            "{label}: Minor Mix-Up must not exist"
        );
    }
}

/// The rows of the two matched files that stay disabled, each with the reason it cannot run on
/// this tip (`notes/Autopilot/T677.md`): Brawler by the lead's decision; the rest need a mechanic
/// or NPC row the server lacks. A row that gains objectives has to be re-checked.
#[test]
fn the_other_rows_stay_off() {
    for id in [
        3_030_022, 3_030_036, // Brawler contracts (lead decision)
        3_060_007, 3_060_017, // Demo Derby: Duncan has no NPC row; no objectives
        3_070_020, 3_070_023, 3_070_045, // Fisherman: no fungus, disguise or treasure catch
        3_090_024, // Medic: rescue targets
        3_110_021, // Ninja: the hermit's book
        3_140_021, 3_140_027, 3_140_030, // Soccer Star: no mechanic, no game 328
        3_010_022, 3_010_023, // Archer, Durango Zoo: smuggler trigger, Chef-only harvest
    ] {
        let quest = shipped_quest(id);
        assert!(!quest.enabled, "quest {id} is enabled, not blocked");
        assert!(
            quest.objectives.is_empty(),
            "{id} grew objectives; re-check it"
        );
    }
}
