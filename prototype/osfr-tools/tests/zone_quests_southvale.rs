//! T729: the Southvale Miner quests the fixed objective specs let the server run today,
//! enabled in `data/content/quests/miner.json`: "Reach Level 10" (3100012) and "Finding
//! Southvale" (3100013) at Derp Axemaker, and "Reach Level 15" (3100019) and "See For
//! Yourself" (3100020) at Grumpore.
//!
//! Every test drives the production in-world router through `tests/support/npc_quest_chain.rs`:
//! the 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with the production reader), the 49/2 accept,
//! the `job_level` goal (T361, completed by a real `realm level`), the Talk that holds the
//! 49/13, the 26/6 that releases it and the 49/14 turn-in. Rewards, text ids and level targets
//! are read from the quest data, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`,
//! 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`, end text
//! `18..22`, one dword, then the reward bundle `0x008e7930`), 49/14 `0x00a864c0`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, accept, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, end_screen_reward,
    finish_quest, objective_complete, offer, pending_turnin, profile_xp, quest_end_reply,
    quest_reply, quest_status, read_json, release_end_screen, seed_active_profile, send, talk,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    npc_interaction::decode_quest_offer_49_1,
    quest_engine::QuestStatus,
};
use serde_json::Value;

const DERP: u64 = 9_000_000_069;
const MILI: u64 = 9_000_000_405;
const GRUMPORE: u64 = 9_000_000_337;
const SILVERRAY: u64 = 9_000_000_203;
const LEVEL_TEN: u32 = 3_100_012;
const SOUTHVALE: u32 = 3_100_013;
const LEVEL_FIFTEEN: u32 = 3_100_019;
const SEE_FOR_YOURSELF: u32 = 3_100_020;
const MINER: i32 = 14;
const OTHER: u64 = CHARACTER + 1;

/// The shipped quest record, for the values the tests must not pin.
fn record(quest: u32) -> Value {
    read_json(&content_dir().join("quests/miner.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(quest)))
        .cloned()
        .unwrap_or_else(|| panic!("quest {quest} ships in miner.json"))
}

fn reward(quest: u32) -> (u32, u32) {
    let row = &record(quest)["rewards"][0];
    (
        row["coins"].as_u64().unwrap() as u32,
        row["job_xp"][MINER.to_string()].as_u64().unwrap_or(0) as u32,
    )
}

/// The rank a `job_level` goal asks for.
fn target_level(quest: u32) -> u32 {
    record(quest)["objectives"][0]["level"].as_u64().unwrap() as u32
}

fn derp(guid: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("snowhill.json", DERP))
}

fn mili(guid: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("greenwood_forest.json", MILI))
}

fn silverray(guid: u64) -> CharacterState {
    character_at_for(
        guid,
        0,
        content_position("blackspore_swamp.json", SILVERRAY),
    )
}

fn grumpore(guid: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("greenwood_forest.json", GRUMPORE))
}

/// A character with `active` selected and both jobs learned. The offer gate reads the
/// *active* job (`Gameplay::character_eligibility`), so the Miner job must be the active one.
fn player(fixture: &Fixture, guid: u64, active: i32) -> Gameplay {
    let mut state = fixture.game_as(guid, None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER, MINER]);
    fixture.game_as(guid, Some(state))
}

fn realm_level(game: &mut Gameplay, who: &CharacterState, level: u32, now: u64) -> Outcome {
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    send(
        game,
        who,
        &chat::local_request(&format!("realm level {level}")),
        now,
    )
}

/// Offer, check the 49/1 card against the record, accept.
fn accepted(game: &mut Gameplay, who: &CharacterState, npc: u64, quest: u32, now: u64) {
    let offered = offer(game, who, npc, quest, now);
    let card = offered
        .replies
        .iter()
        .find_map(|packet| decode_quest_offer_49_1(packet).ok())
        .expect("the 49/1 card decodes");
    let record = record(quest);
    let field = |key: &str| record[key].as_u64().unwrap_or(0) as u32;
    assert_eq!(
        [
            card.name_string_id,
            card.intro_text_string_id,
            card.description_string_id
        ],
        [
            field("title_string_id"),
            field("intro_text_string_id"),
            field("description_string_id")
        ],
        "49/1 title, intro and description come from the record"
    );
    accept(game, who, quest, now + 2);
}

/// The final Talk, the end screen and the 49/14: coins and Miner XP from the data.
fn turn_in(game: &mut Gameplay, who: &CharacterState, npc: u64, quest: u32, now: u64) {
    let (coins, xp) = reward(quest);
    assert!(coins > 0, "quest {quest} pays something");
    talk(game, who, npc, now);
    assert_eq!(pending_turnin(game), Some(quest));
    let end_text = record(quest)["end_text_string_id"].as_u64().unwrap() as u32;
    let end = release_end_screen(game, who, quest, npc, end_text, now + 2);
    assert_eq!(end_screen_reward(&end), (coins, xp));
    finish_quest(game, who, quest, MINER, coins, xp, now + 4);
}

#[test]
fn a_miner_walks_the_level_ten_gate_then_finds_mili_and_is_paid_once_each() {
    let fixture = Fixture::new();
    let mut game = player(&fixture, CHARACTER, MINER);
    let (derp, mili) = (derp(CHARACTER), mili(CHARACTER));
    let mut now = 1_000;

    // The Southvale errand waits behind the gate: a forged accept before it changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &derp,
        &quest_reply(SOUTHVALE, true),
        now,
        "a forged 49/2 for Finding Southvale before the gate",
    );

    accepted(&mut game, &derp, DERP, LEVEL_TEN, now);
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
    let level = realm_level(&mut game, &derp, target_level(LEVEL_TEN), now);
    assert!(
        objective_complete(&level, LEVEL_TEN, 1),
        "reaching the rank sends the existing 49/10"
    );
    assert_eq!(counts(&game, LEVEL_TEN), [1]);

    now += 100;
    let coins_before = coins(&game);
    turn_in(&mut game, &derp, DERP, LEVEL_TEN, now);
    assert_eq!(coins(&game), coins_before + reward(LEVEL_TEN).0);

    // The gate is done, so Derp now sends the Miner south.
    now += 100;
    accepted(&mut game, &derp, DERP, SOUTHVALE, now);
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
    turn_in(&mut game, &mili, MILI, SOUTHVALE, now);
    let (paid, xp) = reward(SOUTHVALE);
    assert_eq!(coins(&game), coins_before + paid);
    assert_eq!(profile_xp(&game, MINER), xp_before + xp);

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
    let mut game = player(&fixture, CHARACTER, MINER);
    let grumpore = grumpore(CHARACTER);
    let mut now = 1_000;
    accepted(&mut game, &grumpore, GRUMPORE, LEVEL_FIFTEEN, now);
    assert_eq!(counts(&game, LEVEL_FIFTEEN), [0]);

    // One rank short never credits, and Grumpore has nothing to hand in or offer.
    now += 100;
    realm_level(&mut game, &grumpore, target_level(LEVEL_FIFTEEN) - 1, now);
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
    let level = realm_level(&mut game, &grumpore, target_level(LEVEL_FIFTEEN), now);
    assert!(objective_complete(&level, LEVEL_FIFTEEN, 1));
    assert_eq!(counts(&game, LEVEL_FIFTEEN), [1]);

    now += 100;
    let coins_before = coins(&game);
    turn_in(&mut game, &grumpore, GRUMPORE, LEVEL_FIFTEEN, now);
    assert_eq!(coins(&game), coins_before + reward(LEVEL_FIFTEEN).0);

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
    accepted(&mut game, &grumpore, GRUMPORE, SEE_FOR_YOURSELF, now);
    assert_no_offer(
        &mut game,
        &grumpore,
        GRUMPORE,
        &mut now,
        "Grumpore with the errand open",
    );
    now += 100;
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, MINER));
    turn_in(
        &mut game,
        &silverray(CHARACTER),
        SILVERRAY,
        SEE_FOR_YOURSELF,
        now,
    );
    let (paid, xp) = reward(SEE_FOR_YOURSELF);
    assert_eq!(coins(&game), coins_before + paid);
    assert_eq!(profile_xp(&game, MINER), xp_before + xp);
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
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    assert_no_offer(
        &mut game,
        &derp(CHARACTER),
        DERP,
        &mut now,
        "Derp for an Adventurer",
    );
    assert_no_offer(
        &mut game,
        &grumpore(CHARACTER),
        GRUMPORE,
        &mut now,
        "Grumpore for an Adventurer",
    );
    for (quest, who) in [
        (LEVEL_TEN, derp(CHARACTER)),
        (LEVEL_FIFTEEN, grumpore(CHARACTER)),
    ] {
        assert_packet_changes_nothing(
            &mut game,
            &who,
            &quest_reply(quest, true),
            now,
            "a forged 49/2 with another job active",
        );
    }
}

#[test]
fn progress_survives_a_reconnect_and_players_are_independent() {
    let fixture = Fixture::new();
    let mut game = player(&fixture, CHARACTER, MINER);
    let mut other = player(&fixture, OTHER, MINER);
    let (grumpore, other_grumpore) = (grumpore(CHARACTER), grumpore(OTHER));
    let mut now = 1_000;
    accepted(&mut game, &grumpore, GRUMPORE, LEVEL_FIFTEEN, now);
    accepted(&mut other, &other_grumpore, GRUMPORE, LEVEL_FIFTEEN, now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, LEVEL_FIFTEEN), QuestStatus::Active);
    now += 100;
    realm_level(&mut game, &grumpore, target_level(LEVEL_FIFTEEN), now);
    assert_eq!(counts(&game, LEVEL_FIFTEEN), [1]);
    assert_eq!(
        counts(&other, LEVEL_FIFTEEN),
        [0],
        "the other player is untouched"
    );

    // The held end screen (shown, 49/14 not yet sent) also survives a save.
    now += 100;
    talk(&mut game, &grumpore, GRUMPORE, now);
    let end_text = record(LEVEL_FIFTEEN)["end_text_string_id"]
        .as_u64()
        .unwrap() as u32;
    release_end_screen(
        &mut game,
        &grumpore,
        LEVEL_FIFTEEN,
        GRUMPORE,
        end_text,
        now + 2,
    );
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(pending_turnin(&game), Some(LEVEL_FIFTEEN));
    let (coins, xp) = reward(LEVEL_FIFTEEN);
    finish_quest(
        &mut game,
        &grumpore,
        LEVEL_FIFTEEN,
        MINER,
        coins,
        xp,
        now + 10,
    );
    assert_eq!(quest_status(&other, LEVEL_FIFTEEN), QuestStatus::Active);
    assert_eq!(counts(&other, LEVEL_FIFTEEN), [0]);
}
