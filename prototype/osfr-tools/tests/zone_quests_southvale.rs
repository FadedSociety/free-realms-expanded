//! T729/F746: the Southvale Miner quests the fixed objective specs let the server
//! run today, enabled in `data/content/quests/miner.json`: "Reach Level 10"
//! (3100012) and "Finding Southvale" (3100013) at Derp Axemaker, and "Reach Level
//! 15" (3100019) and "See For Yourself" (3100020) at Grumpore.
//!
//! Every test drives the production in-world router through
//! `tests/support/npc_quest_chain.rs`: the 26/8 -> 26/10 -> 26/6 offer (49/1,
//! decoded with the production reader), the 49/2 accept, the `job_level` goal
//! (T361, completed by a real `realm level`), the Talk that holds the 49/13, the
//! 26/6 that releases it and the 49/14 turn-in. The quest records, cards, end
//! texts and payouts come from `tests/support/quest_records.rs`, so the reward sum
//! is the loader's (every row, every job-XP profile), not one row's. The only
//! pinned values are the two level gates the client text names.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2
//! `0x00a86400`, 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`,
//! quest `14..18`, end text `18..22`, one dword, then the reward bundle
//! `0x008e7930`), 49/14 `0x00a864c0`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, objective_complete, pending_turnin,
    profile_xp, quest_end_reply, quest_reply, quest_status, realm_level, talk, zone_player,
};
use osfr_tools::quest_engine::QuestStatus;
use quest_records::{RawQuest, as_u32, read_quests};
use std::sync::OnceLock;

const DERP: u64 = 9_000_000_069;
const MILI: u64 = 9_000_000_405;
const GRUMPORE: u64 = 9_000_000_337;
const SILVERRAY: u64 = 9_000_000_203;
const LEVEL_TEN: u32 = 3_100_012;
const SOUTHVALE: u32 = 3_100_013;
const LEVEL_FIFTEEN: u32 = 3_100_019;
const SEE_FOR_YOURSELF: u32 = 3_100_020;
/// The client text's Miner ranks: 387507 "Reach Level 10 ..." and 387518
/// "Reach Level 15 ...". Pinned so a data edit to 11 or 16 fails here.
const LEVEL_TEN_GATE: u32 = 10;
const LEVEL_FIFTEEN_GATE: u32 = 15;
const MINER: i32 = 14;
const OTHER: u64 = CHARACTER + 1;

/// Every NPC row is in `greenwood_forest.json` except Derp (Snowhill) and
/// Silverray (Blackspore).
const MAP: ZoneQuests = ZoneQuests {
    default_file: "greenwood_forest.json",
    files: &[
        (DERP, "snowhill.json"),
        (SILVERRAY, "blackspore_swamp.json"),
    ],
    runtime: &[],
};

/// The shipped quest records, read once through the shared record reader.
fn shipped_quests() -> &'static [RawQuest] {
    static RECORDS: OnceLock<Vec<RawQuest>> = OnceLock::new();
    RECORDS.get_or_init(|| read_quests(&content_dir()))
}

fn record(id: u32) -> &'static RawQuest {
    shipped_quests()
        .iter()
        .find(|quest| quest.id == id)
        .unwrap_or_else(|| panic!("the shipped quest record {id}"))
}

/// The quest with its card, end text and payout read from the shipped record.
fn quest(id: u32, giver: u64, turnin: u64) -> ZoneQuest {
    let shipped = record(id);
    let (coins, xp) = shipped.reward_totals();
    assert!(coins > 0, "quest {id} has a real payout");
    ZoneQuest {
        id,
        giver,
        card: [
            u32::try_from(shipped.title_string_id).expect("quest title id fits u32"),
            shipped.intro_text_string_id,
            shipped.description_string_id,
        ],
        steps: &[],
        turnin,
        end_text: shipped.end_text_string_id,
        coins,
        xp,
    }
}

#[test]
fn a_miner_walks_the_level_ten_gate_then_finds_mili_and_is_paid_once_each() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, MINER);
    let level_ten = quest(LEVEL_TEN, DERP, DERP);
    let southvale = quest(SOUTHVALE, DERP, MILI);
    let (derp, mili) = (MAP.at(CHARACTER, DERP), MAP.at(CHARACTER, MILI));
    let mut now = 1_000;

    assert_eq!(
        as_u32(&record(LEVEL_TEN).objectives[0]["level"]),
        Some(LEVEL_TEN_GATE),
        "the shipped level gate is the client text's level 10"
    );

    // The Southvale errand waits behind the gate: a forged accept before it changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &derp,
        &quest_reply(SOUTHVALE, true),
        now,
        "a forged 49/2 for Finding Southvale before the gate",
    );

    MAP.accept(&mut game, CHARACTER, &level_ten, &mut now);
    assert_eq!(counts(&game, LEVEL_TEN), [0], "rank 1 is short of the gate");
    now += 100;
    assert_packet_changes_nothing(
        &mut game,
        &derp,
        &quest_end_reply(LEVEL_TEN),
        now,
        "a forged 49/14 before the gate",
    );

    now += 100;
    let level = realm_level(&mut game, &derp, LEVEL_TEN_GATE, now);
    assert!(
        objective_complete(&level, LEVEL_TEN, 1),
        "reaching the rank sends the existing 49/10"
    );
    assert_eq!(counts(&game, LEVEL_TEN), [1]);

    now += 100;
    let coins_before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &level_ten, MINER, &mut now);
    assert_eq!(coins(&game), coins_before + level_ten.coins);

    // The gate is done, so Derp now sends the Miner south.
    now += 100;
    MAP.accept(&mut game, CHARACTER, &southvale, &mut now);
    assert_eq!(counts(&game, SOUTHVALE), [0]);
    now += 100;
    assert_no_offer(
        &mut game,
        &derp,
        DERP,
        &mut now,
        "Derp with the errand open",
    );
    assert_eq!(pending_turnin(&game), None, "Derp has no hand-in for it");

    // Mili is the named target: the Talk at her camp opens the end screen there.
    now += 100;
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, MINER));
    MAP.turn_in(&mut game, CHARACTER, &southvale, MINER, &mut now);
    assert_eq!(coins(&game), coins_before + southvale.coins);
    assert_eq!(profile_xp(&game, MINER), xp_before + southvale.xp);

    // Replays and a second offer change nothing.
    now += 100;
    assert_packet_changes_nothing(
        &mut game,
        &mili,
        &quest_end_reply(SOUTHVALE),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &derp, DERP, &mut now, "Derp after the errand");
    assert_eq!(quest_status(&game, SOUTHVALE), QuestStatus::Completed);
}

#[test]
fn grumpore_holds_the_level_fifteen_gate_until_the_rank_is_real() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, MINER);
    let level_fifteen = quest(LEVEL_FIFTEEN, GRUMPORE, GRUMPORE);
    let see_for_yourself = quest(SEE_FOR_YOURSELF, GRUMPORE, SILVERRAY);
    let grumpore = MAP.at(CHARACTER, GRUMPORE);
    let mut now = 1_000;

    assert_eq!(
        as_u32(&record(LEVEL_FIFTEEN).objectives[0]["level"]),
        Some(LEVEL_FIFTEEN_GATE),
        "the shipped level gate is the client text's level 15"
    );

    MAP.accept(&mut game, CHARACTER, &level_fifteen, &mut now);
    assert_eq!(counts(&game, LEVEL_FIFTEEN), [0]);

    // One rank short never credits, and Grumpore has nothing to hand in or offer.
    now += 100;
    realm_level(&mut game, &grumpore, LEVEL_FIFTEEN_GATE - 1, now);
    assert_eq!(counts(&game, LEVEL_FIFTEEN), [0]);
    assert_no_offer(
        &mut game,
        &grumpore,
        GRUMPORE,
        &mut now,
        "Grumpore one rank short",
    );
    assert_eq!(pending_turnin(&game), None);
    assert_packet_changes_nothing(
        &mut game,
        &grumpore,
        &quest_end_reply(LEVEL_FIFTEEN),
        now + 10,
        "an early 49/14",
    );
    assert_eq!(quest_status(&game, LEVEL_FIFTEEN), QuestStatus::Active);

    now += 100;
    let level = realm_level(&mut game, &grumpore, LEVEL_FIFTEEN_GATE, now);
    assert!(objective_complete(&level, LEVEL_FIFTEEN, 1));
    assert_eq!(counts(&game, LEVEL_FIFTEEN), [1]);

    now += 100;
    let coins_before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &level_fifteen, MINER, &mut now);
    assert_eq!(coins(&game), coins_before + level_fifteen.coins);

    now += 100;
    assert_packet_changes_nothing(
        &mut game,
        &grumpore,
        &quest_end_reply(LEVEL_FIFTEEN),
        now,
        "a duplicate 49/14",
    );

    // The gate opens Grumpore's next errand, a Talk at Silverray in Blackspore.
    now += 100;
    MAP.accept(&mut game, CHARACTER, &see_for_yourself, &mut now);
    assert_no_offer(
        &mut game,
        &grumpore,
        GRUMPORE,
        &mut now,
        "Grumpore with the errand open",
    );
    now += 100;
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, MINER));
    MAP.turn_in(&mut game, CHARACTER, &see_for_yourself, MINER, &mut now);
    assert_eq!(coins(&game), coins_before + see_for_yourself.coins);
    assert_eq!(profile_xp(&game, MINER), xp_before + see_for_yourself.xp);
    assert_eq!(pending_turnin(&game), None);
    now += 100;
    assert_no_offer(
        &mut game,
        &grumpore,
        GRUMPORE,
        &mut now,
        "Grumpore after both",
    );
}

#[test]
fn only_the_active_miner_is_offered_the_gates() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, DERP),
        DERP,
        &mut now,
        "Derp for an Adventurer",
    );
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, GRUMPORE),
        GRUMPORE,
        &mut now,
        "Grumpore for an Adventurer",
    );
    for (quest, npc) in [(LEVEL_TEN, DERP), (LEVEL_FIFTEEN, GRUMPORE)] {
        assert_packet_changes_nothing(
            &mut game,
            &MAP.at(CHARACTER, npc),
            &quest_reply(quest, true),
            now,
            "a forged 49/2 with another job active",
        );
    }
}

#[test]
fn progress_survives_a_reconnect_and_players_are_independent() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, MINER);
    let mut other = zone_player(&fixture, OTHER, MINER);
    let level_fifteen = quest(LEVEL_FIFTEEN, GRUMPORE, GRUMPORE);
    let grumpore = MAP.at(CHARACTER, GRUMPORE);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &level_fifteen, &mut now);
    MAP.accept(&mut other, OTHER, &level_fifteen, &mut now);

    let mut game = fixture.save_and_reload(&game, &grumpore);
    assert_eq!(quest_status(&game, LEVEL_FIFTEEN), QuestStatus::Active);
    now += 100;
    realm_level(&mut game, &grumpore, LEVEL_FIFTEEN_GATE, now);
    assert_eq!(counts(&game, LEVEL_FIFTEEN), [1]);
    assert_eq!(
        counts(&other, LEVEL_FIFTEEN),
        [0],
        "the other player is untouched"
    );

    // The held end screen (shown, 49/14 not yet sent) also survives a save.
    now += 100;
    talk(&mut game, &grumpore, GRUMPORE, now);
    let mut game = fixture.save_and_reload(&game, &grumpore);
    assert_eq!(pending_turnin(&game), Some(LEVEL_FIFTEEN));
    // A reconnect has no in-memory 26/6 choice, so the client Talks again; the saved
    // actor must still reopen the 49/13.
    now += 100;
    talk(&mut game, &grumpore, GRUMPORE, now);
    MAP.pay_held_turnin(&mut game, CHARACTER, &level_fifteen, MINER, &mut now);
    assert_eq!(quest_status(&game, LEVEL_FIFTEEN), QuestStatus::Completed);
    assert_eq!(quest_status(&other, LEVEL_FIFTEEN), QuestStatus::Active);
    assert_eq!(counts(&other, LEVEL_FIFTEEN), [0]);
}

/// Finding Southvale's only objective is a Talk at Mili, a named target rather
/// than the quest's fallback turn-in NPC (Derp). A save made with the end screen
/// held must keep the actor: the 49/13 after a reconnect opens at Mili.
#[test]
fn finding_southvale_reopens_at_mili_after_the_real_save_envelope() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, MINER);
    let level_ten = quest(LEVEL_TEN, DERP, DERP);
    let southvale = quest(SOUTHVALE, DERP, MILI);
    let (derp, mili) = (MAP.at(CHARACTER, DERP), MAP.at(CHARACTER, MILI));
    let mut now = 1_000;

    MAP.accept(&mut game, CHARACTER, &level_ten, &mut now);
    now += 10;
    realm_level(&mut game, &derp, LEVEL_TEN_GATE, now);
    now += 10;
    MAP.turn_in(&mut game, CHARACTER, &level_ten, MINER, &mut now);

    MAP.accept(&mut game, CHARACTER, &southvale, &mut now);
    now += 10;
    talk(&mut game, &mili, MILI, now);
    assert_eq!(pending_turnin(&game), Some(SOUTHVALE));

    let mut game = fixture.save_and_reload(&game, &mili);
    assert_eq!(pending_turnin(&game), Some(SOUTHVALE));
    // A reconnect has no in-memory 26/6 choice: the second Talk at Mili must reopen the
    // 49/13 from the saved actor. If the actor were lost, the Derp fallback would refuse
    // the Talk and pay_held_turnin would find no 49/13 to release.
    now += 10;
    talk(&mut game, &mili, MILI, now);
    MAP.pay_held_turnin(&mut game, CHARACTER, &southvale, MINER, &mut now);
    assert_eq!(quest_status(&game, SOUTHVALE), QuestStatus::Completed);
}
