//! T661: "Demo Derby: Think of the Kittens" (`3060024`), the one Cobblestone Village,
//! Farnum Farmstead, Stillwater Crossing or Highroad Junction quest from the zone-quest
//! wiring lists that a shipped server can run end to end, enabled in
//! `data/content/quests/demo_derby_driver.json` at Kanan's existing Stillwater Crossing
//! `legacy_data` row.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`: the
//! 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with the production reader), the 49/2 accept,
//! `Gameplay::on_location` for the visit goal, the Talk that holds the 49/13, the 26/6 that
//! releases it and the 49/14 turn-in. Rewards and the visit anchor are read from the quest
//! data, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`,
//! 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`, end text
//! `18..22`, one dword, then the reward bundle `0x008e7930`), 49/14 `0x00a864c0`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, assert_no_offer, assert_packet_changes_nothing, character_at_for,
    coins, content_dir, content_position, counts, end_screen_reward, finish_quest, offer,
    pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status, read_json,
    release_end_screen, saved, seed_active_profile, talk,
};
use osfr_tools::{
    character_store::CharacterState, gameplay::Gameplay, npc_interaction::decode_quest_offer_49_1,
    quest_engine::QuestStatus,
};
use serde_json::Value;

const KANAN: u64 = 9_000_000_124;
const QUEST: u32 = 3_060_024;
const DERBY_DRIVER: i32 = 49;
const ADVENTURER: i32 = 1;
const OTHER: u64 = CHARACTER + 1;

/// The shipped quest record, for the values the tests must not pin.
fn record() -> Value {
    read_json(&content_dir().join("quests/demo_derby_driver.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(QUEST)))
        .cloned()
        .expect("the Think of the Kittens record")
}

fn reward() -> (u32, u32) {
    let row = &record()["rewards"][0];
    (
        row["coins"].as_u64().unwrap() as u32,
        row["job_xp"][DERBY_DRIVER.to_string()].as_u64().unwrap() as u32,
    )
}

fn visit_anchor() -> [f32; 3] {
    serde_json::from_value(record()["objectives"][0]["position"].clone()).unwrap()
}

fn kanan(guid: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("stillwater_crossing.json", KANAN))
}

fn at_anchor(guid: u64) -> CharacterState {
    character_at_for(guid, 0, visit_anchor())
}

/// A character with `active` selected and both jobs learned. The quest's offer gate reads
/// the active job (`Gameplay::character_eligibility`), so the derby job must be the active one.
fn player(fixture: &Fixture, guid: u64, active: i32) -> Gameplay {
    let mut state = fixture.game_as(guid, None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER, DERBY_DRIVER]);
    fixture.game_as(guid, Some(state))
}

fn accepted(game: &mut Gameplay, guid: u64, now: u64) {
    let offered = offer(game, &kanan(guid), KANAN, QUEST, now);
    let card = offered
        .replies
        .iter()
        .find_map(|packet| decode_quest_offer_49_1(packet).ok())
        .expect("the 49/1 card decodes");
    let record = record();
    assert_eq!(
        [
            card.name_string_id,
            card.intro_text_string_id,
            card.description_string_id
        ],
        [
            "title_string_id",
            "intro_text_string_id",
            "description_string_id"
        ]
        .map(|key| record[key].as_u64().unwrap() as u32),
        "49/1 title, intro and description come from the record"
    );
    accept(game, &kanan(guid), QUEST, now + 2);
}

fn turn_in(game: &mut Gameplay, guid: u64, now: u64) {
    let (coins, xp) = reward();
    talk(game, &kanan(guid), KANAN, now);
    let end_text = record()["end_text_string_id"].as_u64().unwrap_or(0) as u32;
    let end = release_end_screen(game, &kanan(guid), QUEST, KANAN, end_text, now + 2);
    assert_eq!(end_screen_reward(&end), (coins, xp));
    assert_eq!(pending_turnin(game), Some(QUEST));
    finish_quest(game, &kanan(guid), QUEST, DERBY_DRIVER, coins, xp, now + 4);
}

#[test]
fn the_flier_run_is_offered_walked_and_paid_once() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, DERBY_DRIVER);
    let mut now = 1_000;
    accepted(&mut game, CHARACTER, now);
    assert_eq!(counts(&game, QUEST), [0]);

    // Standing at Kanan is not the flier run; only the town's anchor is.
    now += 100;
    let before = saved(&game);
    let near_kanan = game.on_location(&kanan(CHARACTER), now).unwrap();
    assert!(!near_kanan.changed, "Kanan's spot is outside the anchor");
    assert_eq!(saved(&game), before, "a far visit changes nothing");
    assert_eq!(counts(&game, QUEST), [0]);

    now += 100;
    let visited = game.on_location(&at_anchor(CHARACTER), now).unwrap();
    assert!(visited.changed, "the anchor completes the goal");
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, DERBY_DRIVER));
    now += 100;
    turn_in(&mut game, CHARACTER, now);
    let (paid_coins, paid_xp) = reward();
    assert_eq!(coins(&game), coins_before + paid_coins);
    assert_eq!(profile_xp(&game, DERBY_DRIVER), xp_before + paid_xp);

    // Replays and a second offer change nothing.
    now += 100;
    assert_packet_changes_nothing(
        &mut game,
        &kanan(CHARACTER),
        &quest_end_reply(QUEST),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &kanan(CHARACTER), KANAN, &mut now, "Kanan after");
}

#[test]
fn only_the_active_derby_driver_is_offered_it_and_forged_packets_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    assert_no_offer(
        &mut game,
        &kanan(CHARACTER),
        KANAN,
        &mut now,
        "derby job not active",
    );
    assert_packet_changes_nothing(
        &mut game,
        &kanan(CHARACTER),
        &quest_reply(QUEST, true),
        now,
        "a forged 49/2 with another job active",
    );

    // With the job active: accepted, but not yet done.
    let mut game = player(&fixture, CHARACTER, DERBY_DRIVER);
    now += 100;
    accepted(&mut game, CHARACTER, now);
    now += 100;
    let waiting = talk(&mut game, &kanan(CHARACTER), KANAN, now);
    assert!(!waiting.changed, "Kanan has no hand-in before the visit");
    assert_eq!(pending_turnin(&game), None);
    assert_packet_changes_nothing(
        &mut game,
        &kanan(CHARACTER),
        &quest_end_reply(QUEST),
        now + 10,
        "a forged 49/14 before the visit",
    );
    assert_packet_changes_nothing(
        &mut game,
        &kanan(CHARACTER),
        &quest_reply(QUEST, true),
        now + 20,
        "a replayed 49/2",
    );
}

#[test]
fn progress_survives_reconnect_and_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, DERBY_DRIVER);
    let mut other = player(&fixture, OTHER, DERBY_DRIVER);
    let mut now = 1_000;
    accepted(&mut game, CHARACTER, now);
    now += 100;
    assert!(
        game.on_location(&at_anchor(CHARACTER), now)
            .unwrap()
            .changed
    );

    // The second player is untouched by the first one's visit.
    accepted(&mut other, OTHER, now);
    assert_eq!(counts(&other, QUEST), [0]);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(counts(&game, QUEST), [1], "the visit survived the reload");
    now += 100;
    turn_in(&mut game, CHARACTER, now);
    assert_eq!(counts(&other, QUEST), [0]);
    assert_eq!(quest_status(&other, QUEST), QuestStatus::Active);
}
