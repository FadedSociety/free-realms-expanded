//! T724: zone quests round 3, Briarwood. Brawler: IOU! (`3030027`) is the one quest of the
//! round-3 list that the server can run end to end today; the other nine stay disabled
//! (`notes/Autopilot/T724.md` names each blocker).
//!
//! Andrew (`9000000009`, Briarwood, a Legacy spawn) offers it; Nurble (`9000000415`, Wilds, a
//! Legacy spawn) is the talk target and the turn-in. Every test drives the production paths in
//! `tests/support/npc_quest_chain.rs`: 26/8 -> 26/10 -> 26/6 offers (49/1, decoded with the
//! production reader), 49/2 accepts, the 26/6 that releases the held 49/13, and the 49/14
//! turn-in. Client readers (IDA, 2014 client; no new wire): 49/13 `QuestEndPacket::ReadFields`
//! `0x00c7b990`, 26/3 payload reader `0x00a9ef10`, AddNpc `0x00921a10`.
//!
//! Rewards and the rank gate are read from the quest record, so a retune in data needs no edit
//! here; the tests assert that the record pays what the 49/13 shows and the save receives.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, active_job, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, profile_xp, quest_end_reply,
    quest_reply, quest_status, read_json, saved, seed_active_profile,
};
use osfr_tools::{
    gameplay::Gameplay,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use serde_json::Value;
use std::collections::BTreeMap;

const ANDREW: u64 = 9_000_000_009;
const NURBLE: u64 = 9_000_000_415;
const IOU: u32 = 3_030_027;
const BRAWLER: i32 = 43;
const OTHER: u64 = CHARACTER + 1;

/// Andrew's row is in `briarwood.json`; Nurble's is in `wilds.json`.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "briarwood.json",
    files: &[(NURBLE, "wilds.json")],
    runtime: &[],
};

fn record() -> Value {
    read_json(&content_dir().join("quests/brawler.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|quest| quest["id"].as_u64() == Some(u64::from(IOU)))
        .expect("Brawler: IOU! is in brawler.json")
        .clone()
}

/// The quest as the harness walks it, with the card ids and rewards read from the record.
fn iou() -> ZoneQuest {
    let record = record();
    let reward = &record["rewards"][0];
    let xp = reward["job_xp"][BRAWLER.to_string()].as_u64().unwrap() as u32;
    ZoneQuest {
        id: IOU,
        giver: ANDREW,
        card: [
            record["title_string_id"].as_u64().unwrap() as u32,
            record["intro_text_string_id"].as_u64().unwrap() as u32,
            record["description_string_id"].as_u64().unwrap() as u32,
        ],
        steps: &[],
        turnin: NURBLE,
        end_text: record["end_text_string_id"].as_u64().unwrap() as u32,
        coins: reward["coins"].as_u64().unwrap() as u32,
        xp,
        profile_xp: xp,
    }
}

/// `Profiles::rank` is `1 + xp / 100`.
fn xp_for_rank(rank: u64) -> u32 {
    ((rank - 1) * 100) as u32
}

/// The authored band gate; a record with none (or rank 1) would make the rank-short case below
/// meaningless, so that fails here instead of underflowing.
fn min_rank() -> u64 {
    let rank = record()["min_level"].as_u64().expect("the band gate");
    assert!(rank > 1, "IOU keeps a band gate above rank 1");
    rank
}

/// A player with Brawler active at `xp` Brawler XP.
fn brawler(fixture: &Fixture, guid: u64, xp: u32) -> Gameplay {
    let mut state = fixture.game_as(guid, None).state();
    seed_active_profile(&mut state, BRAWLER, &[ADVENTURER]);
    state
        .profiles
        .as_mut()
        .expect("the fixture has profile content")
        .experience
        .insert(BRAWLER, xp);
    fixture.game_as(guid, Some(state))
}

#[test]
fn iou_is_offered_by_andrew_and_handed_in_to_nurble() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = brawler(&fixture, CHARACTER, xp_for_rank(min_rank()));
    let quest = iou();
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, BRAWLER));
    let mut now = 1_000;
    assert!(quest.coins > 0 && quest.xp > 0, "the record pays something");

    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, IOU), [0], "one Talk goal, not yet done");
    MAP.turn_in(&mut game, CHARACTER, &quest, BRAWLER, &mut now);

    assert_eq!(quest_status(&game, IOU), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, BRAWLER), xp_before + quest.profile_xp);

    // Done is done: neither NPC offers it again and a repeated 49/14 pays nothing.
    let coins_done = coins(&game);
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, ANDREW),
        ANDREW,
        &mut now,
        "Andrew after IOU",
    );
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, NURBLE),
        NURBLE,
        &mut now,
        "Nurble after IOU",
    );
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, NURBLE),
        &quest_end_reply(IOU),
        now,
        "a duplicate 49/14 for IOU",
    );
    assert_eq!(coins(&game), coins_done);
}

#[test]
fn iou_waits_for_the_band_gate_and_the_brawler_job() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut now = 1_000;
    let andrew = MAP.at(CHARACTER, ANDREW);

    // One rank short of the authored gate: Andrew offers nothing, the save is byte-equal.
    let mut game = brawler(&fixture, CHARACTER, xp_for_rank(min_rank() - 1));
    assert_no_offer(&mut game, &andrew, ANDREW, &mut now, "a rank-short Brawler");
    assert_packet_changes_nothing(
        &mut game,
        &andrew,
        &quest_reply(IOU, true),
        now,
        "a forged 49/2 below the gate",
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&IOU)
    );

    // At the gate with another job active: the quest is job-locked, not just level-locked.
    let mut state = fixture.game_as(CHARACTER, None).state();
    seed_active_profile(&mut state, ADVENTURER, &[BRAWLER]);
    state
        .profiles
        .as_mut()
        .unwrap()
        .experience
        .insert(BRAWLER, xp_for_rank(min_rank()));
    let mut other_job = fixture.game_as(CHARACTER, Some(state));
    assert_ne!(active_job(&other_job), BRAWLER);
    assert_no_offer(
        &mut other_job,
        &andrew,
        ANDREW,
        &mut now,
        "an Adventurer-active player with a ranked Brawler",
    );

    // The same Talk offers it once the gate is met.
    let mut ready = brawler(&fixture, CHARACTER, xp_for_rank(min_rank()));
    MAP.accept(&mut ready, CHARACTER, &iou(), &mut now);
}

#[test]
fn forged_or_early_requests_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = brawler(&fixture, CHARACTER, xp_for_rank(min_rank()));
    let mut now = 1_000;
    let nurble = MAP.at(CHARACTER, NURBLE);

    // Nurble before the quest is accepted: nothing eligible, nothing credited.
    assert_no_offer(
        &mut game,
        &nurble,
        NURBLE,
        &mut now,
        "Nurble before IOU is accepted",
    );
    assert_packet_changes_nothing(
        &mut game,
        &nurble,
        &quest_end_reply(IOU),
        now,
        "a 49/14 before IOU is accepted",
    );
    assert_packet_changes_nothing(
        &mut game,
        &nurble,
        &quest_reply(IOU, true),
        now,
        "a forged 49/2 with nothing offered",
    );

    // Accepted but not handed in: an early 49/14 and a Talk at the wrong NPC credit nothing.
    MAP.accept(&mut game, CHARACTER, &iou(), &mut now);
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, ANDREW),
        &quest_end_reply(IOU),
        now,
        "a 49/14 before the Talk at Nurble",
    );
    assert_eq!(counts(&game, IOU), [0]);
    assert_eq!(quest_status(&game, IOU), QuestStatus::Active);
}

#[test]
fn progress_survives_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = brawler(&fixture, CHARACTER, xp_for_rank(min_rank()));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &iou(), &mut now);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, IOU), QuestStatus::Active);
    MAP.turn_in(&mut game, CHARACTER, &iou(), BRAWLER, &mut now);
    let game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, IOU), QuestStatus::Completed);
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = brawler(&fixture, CHARACTER, xp_for_rank(min_rank()));
    let mut b = brawler(&fixture, OTHER, xp_for_rank(min_rank()));
    let quest = iou();
    let mut now = 1_000;
    MAP.accept(&mut a, CHARACTER, &quest, &mut now);
    MAP.accept(&mut b, OTHER, &quest, &mut now);
    let b_before = saved(&b);

    MAP.turn_in(&mut a, CHARACTER, &quest, BRAWLER, &mut now);
    assert_eq!(saved(&b), b_before, "A's turn-in changed B");
    assert_eq!(quest_status(&b, IOU), QuestStatus::Active);

    let b_coins = coins(&b);
    MAP.turn_in(&mut b, OTHER, &quest, BRAWLER, &mut now);
    assert_eq!(coins(&b), b_coins + quest.coins);
}

#[test]
fn the_loader_binds_andrew_as_giver_and_nurble_as_turn_in() {
    let engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility {
            player_level: 20,
            job_levels: BTreeMap::new(),
            member: true,
            nonmember_quest_limit: 20,
            member_quest_limit: 20,
        },
        None,
    )
    .expect("the shipped split catalogue loads");
    assert!(engine.is_enabled(IOU));
    assert_eq!(engine.giver_npc_id(IOU), Some(ANDREW));
    // The formal turn-in drives the quest marker and the menu's turn-in row.
    assert_eq!(engine.turnin_npc_id(IOU), Some(NURBLE));
}
