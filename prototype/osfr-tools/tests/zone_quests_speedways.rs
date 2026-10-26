//! T726: "Kart Driver: Seaside Board of Tourism Sponsorship" (3080019), the one
//! Speedways quest of the round-3 list the server can run end to end today.
//!
//! The record is content only (`data/content/quests/kart_driver.json`): Petal
//! Oceanmist (9500000613, a `legacy_data` row T698 drew at the Frost Ridge gates)
//! offers it, one `talk_to_npc` objective names her again, and the final Talk is the
//! 49/13 turn-in. No packet, handler or selector is new; the wire readers are the
//! ones `tests/support/npc_quest_chain.rs` cites: 49/1 `0x00c7bb60`, 49/2
//! `0x00a86400`, 49/13 `QuestEndPacket::ReadFields` `0x00c7b990`, 49/14
//! `0x00a864c0`, AddNpc `0x00921a10`.
//!
//! The other seven quests of the list stay disabled: their `finish_minigame` ids
//! (635, 640, 593, 650, 651, 654, and the derby tutorial's row 289) are rows no server
//! path emits (`tests/support/quest_target_sets.rs`: only kart race 31 credits a kart
//! finish), so `enabled_quest_reachability.rs` refuses them. Every reward is read from
//! the shipped data, never pinned here.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, content_position, counts, finish_quest,
    pending_offer, pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status,
    release_end_screen, saved, talk, zone_player,
};
use osfr_tools::{quest_engine::QuestStatus, startup_world::parse_add_npc, world::NpcVisibility};
use quest_records::read_quests;

const PETAL: u64 = 9_500_000_613;
const QUEST: u32 = 3_080_019;
const KART_DRIVER: i32 = 48;
/// Title, intro (the offer line) and description (the objective hint) the 49/1 card carries.
const CARD: [u32; 3] = [394_663, 394_665, 394_664];
/// Petal's reply on the turn-in Talk (client locale, quest_dialog tagged to this title).
const END_TEXT: u32 = 394_666;

const MAP: ZoneQuests = ZoneQuests {
    default_file: "seaside.json",
    files: &[],
    runtime: &[],
};

/// The quest with its reward read from the shipped record.
fn sponsorship() -> ZoneQuest {
    let record = read_quests(&content_dir())
        .into_iter()
        .find(|quest| quest.id == QUEST)
        .expect("the record ships");
    assert!(record.enabled, "the sponsorship quest is enabled");
    let (coins, xp) = record.reward_totals();
    assert!(coins > 0 && xp > 0, "the quest pays coins and job XP");
    ZoneQuest {
        id: QUEST,
        giver: PETAL,
        card: CARD,
        steps: &[],
        turnin: PETAL,
        end_text: END_TEXT,
        coins,
        xp,
    }
}

#[test]
fn a_kart_driver_is_offered_the_sponsorship_by_petal_and_paid_once() {
    let quest = sponsorship();
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, KART_DRIVER);
    let petal = MAP.at(CHARACTER, PETAL);
    let mut now = 1_000;
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, KART_DRIVER));

    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, QUEST), [0]);
    MAP.turn_in(&mut game, CHARACTER, &quest, KART_DRIVER, &mut now);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, KART_DRIVER), xp_before + quest.xp);

    // Done once: no second offer, and a replayed 49/14 or 49/2 changes nothing.
    assert_no_offer(&mut game, &petal, PETAL, &mut now, "Petal after the quest");
    assert_packet_changes_nothing(
        &mut game,
        &petal,
        &quest_end_reply(QUEST),
        now,
        "a replayed 49/14",
    );
    assert_packet_changes_nothing(
        &mut game,
        &petal,
        &quest_reply(QUEST, true),
        now,
        "a replayed 49/2",
    );
    assert_eq!(coins(&game), coins_before + quest.coins);
}

#[test]
fn a_player_without_the_kart_job_is_not_offered_the_sponsorship() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let petal = MAP.at(CHARACTER, PETAL);
    let mut now = 1_000;
    assert_no_offer(
        &mut game,
        &petal,
        PETAL,
        &mut now,
        "Petal for an Adventurer",
    );
    let before = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &petal,
        &quest_reply(QUEST, true),
        now,
        "a forged 49/2",
    );
    assert_eq!(saved(&game), before);
}

#[test]
fn an_early_turn_in_changes_nothing_until_petal_is_talked_to() {
    let quest = sponsorship();
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, KART_DRIVER);
    let petal = MAP.at(CHARACTER, PETAL);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    // Accepted, not yet talked to: nothing is held for a turn-in, so a forged
    // 49/14 and a repeated 49/2 are refused without touching the save.
    assert_eq!(pending_turnin(&game), None);
    assert_packet_changes_nothing(
        &mut game,
        &petal,
        &quest_end_reply(QUEST),
        now,
        "an early 49/14",
    );
    assert_packet_changes_nothing(
        &mut game,
        &petal,
        &quest_reply(QUEST, true),
        now,
        "a second 49/2",
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0]);
    assert_eq!(pending_offer(&game), None);

    // With the end screen held, a stray 49/2 neither drops it nor starts anything.
    now += 10;
    talk(&mut game, &petal, PETAL, now);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    assert_packet_changes_nothing(
        &mut game,
        &petal,
        &quest_reply(QUEST, true),
        now + 1,
        "a 49/2 mid turn-in",
    );
    assert_eq!(pending_turnin(&game), Some(QUEST));
    release_end_screen(&mut game, &petal, QUEST, PETAL, END_TEXT, now + 2);
}

#[test]
fn the_sponsorship_survives_a_reconnect() {
    let quest = sponsorship();
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, KART_DRIVER);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    // The held end screen (Talk done, 49/14 not yet sent) also survives a save.
    let petal = MAP.at(CHARACTER, PETAL);
    talk(&mut game, &petal, PETAL, now);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    finish_quest(
        &mut game,
        &petal,
        QUEST,
        KART_DRIVER,
        quest.coins,
        quest.xp,
        now + 4,
    );
}

#[test]
fn petal_is_drawn_with_her_client_name_at_the_frost_ridge_gates() {
    let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let position = content_position("seaside.json", PETAL);
    assert_eq!(visibility.content_key(PETAL), Some(PETAL));
    let added = visibility
        .update(0, position)
        .expect("a player at the spawn gets the draw set")
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == PETAL)
        .expect("AddNpc for Petal");
    assert_eq!(added.position[..3], position);
    // fairy_f_valinda.adr (model 36) and the client locale name 'Petal Oceanmist' (394904).
    assert_eq!((added.model_id, added.name_string_id), (36, 394_904));
}
