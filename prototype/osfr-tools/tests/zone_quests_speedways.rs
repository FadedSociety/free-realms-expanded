//! T726: "Kart Driver: Seaside Board of Tourism Sponsorship" (3080019), the one
//! Speedways quest of the round-3 list the server can run end to end today.
//!
//! The record is content only (`data/content/quests/kart_driver.json`): Petal
//! Oceanmist (9500000613, a `legacy_data` row T698 drew for the Frost Ridge
//! Speedway) offers it, one `talk_to_npc` objective names her again, and the final
//! Talk is the 49/13 turn-in. No packet, handler or selector is new; the wire
//! readers are the ones `tests/support/npc_quest_chain.rs` cites: 49/1
//! `0x00c7bb60`, 49/2 `0x00a86400`, 49/13 `QuestEndPacket::ReadFields`
//! `0x00c7b990`, 49/14 `0x00a864c0`, AddNpc `0x00921a10`. The 49/3 goal row's name
//! and description locale ids are resolved by the journal (`0x00cf7fd0` case 0 =
//! name, case 1 = description), which is why the objective text is the client's
//! direct Talk line 394910, not the narrative hint 394664.
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
    ADVENTURER, CHARACTER, Fixture, TALK_LABEL, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, content_position, counts, finish_quest,
    has_family, interact, menu_option, npc_select, pending_offer, pending_turnin, profile_xp,
    quest_end_reply, quest_reply, quest_status, read_json, release_end_screen, saved, send, talk,
    zone_player,
};
use osfr_tools::{quest_engine::QuestStatus, startup_world::parse_add_npc, world::NpcVisibility};
use quest_records::read_quests;

const PETAL: u64 = 9_500_000_613;
/// The POI-anchored Wilds row for the same Petal at the Frost Ridge Speedway POI
/// (`placement_candidates`: poi_id 33); the seaside row is the legacy Snowhill spawn.
const SPEEDWAY_POI: u64 = 9_500_000_930;
const QUEST: u32 = 3_080_019;
const KART_DRIVER: i32 = 48;
/// Title, intro (the offer line) and description (the objective hint) the 49/1 card carries.
const CARD: [u32; 3] = [394_663, 394_665, 394_664];
/// Petal's reply on the turn-in Talk (client locale, quest_dialog tagged to this title).
const END_TEXT: u32 = 394_666;
/// The client's direct objective line ("Talk to Petal Oceanmist at Frost Ridge
/// Speedway to see if she will offer you a sponsorship."); 394911 is its short form.
const OBJECTIVE_TEXT: u32 = 394_910;

const MAP: ZoneQuests = ZoneQuests {
    default_file: "seaside.json",
    files: &[],
    runtime: &[],
};

/// The quest with its reward read from the shipped record. The grant is the named
/// Kart Driver profile, not the sum of every profile's XP (the 49/13 star total is
/// that sum, which is why the one-profile shape is asserted first).
fn sponsorship() -> ZoneQuest {
    let record = read_quests(&content_dir())
        .into_iter()
        .find(|quest| quest.id == QUEST)
        .expect("the record ships");
    assert!(record.enabled, "the sponsorship quest is enabled");
    assert_eq!(record.rewards.len(), 1, "the quest pays one reward row");
    assert_eq!(
        record.rewards[0]["job_xp"]
            .as_object()
            .expect("job XP map")
            .len(),
        1,
        "one profile pays, so the 49/13 star total is that profile's grant"
    );
    let (coins, xp) = record.reward_totals(KART_DRIVER);
    assert!(coins > 0 && xp > 0, "the quest pays coins and job XP");
    assert_eq!(
        record.objectives[0]["text_string_id"].as_u64(),
        Some(u64::from(OBJECTIVE_TEXT)),
        "the tracker goal is the client's Talk line, not the narrative hint 394664"
    );
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
fn petal_row_binds_the_sponsorship_as_given_and_accepted() {
    // The NPC row is the loader's explicit giver/turn-in mapping. Both lists name
    // the quest with its title and provenance so a content audit (and the 26/9
    // turn-in option) sees the hand-in instead of a stale "no hand-in" note.
    let document = read_json(&content_dir().join("npcs/seaside.json"));
    let row = document["records"]
        .as_array()
        .expect("records")
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(PETAL))
        .expect("Petal's row ships");
    assert_eq!(row["quest_ids_given"], serde_json::json!([QUEST]));
    for list in ["quests_given", "quests_accepted"] {
        let entries = row[list].as_array().expect(list);
        let entry = entries
            .iter()
            .find(|entry| entry["quest_id"].as_u64() == Some(u64::from(QUEST)))
            .unwrap_or_else(|| panic!("{list} binds the sponsorship"));
        assert_eq!(
            entry["title_string_id"].as_u64(),
            Some(394_663),
            "{list}: the quest title id"
        );
        assert!(
            entry["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty()),
            "{list}: the binding carries its provenance"
        );
    }
}

#[test]
fn a_kart_driver_accept_without_an_offer_changes_nothing() {
    // The accept gate is the engine's pending offer, not the job: a Kart Driver
    // who never talked to Petal cannot jump the 49/1 offer with a bare 49/2.
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, KART_DRIVER);
    let petal = MAP.at(CHARACTER, PETAL);
    assert_packet_changes_nothing(
        &mut game,
        &petal,
        &quest_reply(QUEST, true),
        1_000,
        "a 49/2 accept with no offer",
    );
    assert_eq!(pending_offer(&game), None);
    assert_eq!(pending_turnin(&game), None);
}

#[test]
fn a_turn_in_menu_option_cannot_skip_the_talk_objective() {
    // The row's `quests_accepted` binding (the explicit hand-in) puts a 26/9
    // TurnInQuest option, labelled with the quest title, on Petal's menu while the
    // quest is active. Selecting it before the Talk objective is complete must be
    // refused, leave the save untouched, and keep the open menu usable.
    let quest = sponsorship();
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, KART_DRIVER);
    let petal = MAP.at(CHARACTER, PETAL);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    let menu = interact(&mut game, &petal, PETAL, now);
    let turn_in = menu_option(&menu, CARD[0]);
    let talk = menu_option(&menu, TALK_LABEL);
    let before = saved(&game);
    let refused = send(&mut game, &petal, &npc_select(PETAL, turn_in), now + 1);
    assert!(!refused.changed, "an incomplete turn-in changed gameplay");
    assert!(
        !refused
            .replies
            .iter()
            .any(|reply| has_family(reply, 49, 13, None)),
        "an incomplete turn-in opened the 49/13 end screen"
    );
    assert_eq!(
        saved(&game),
        before,
        "the incomplete turn-in changed the serialized state"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), None);
    // The rejected selection did not consume the menu: its Talk option still runs.
    let talked = send(&mut game, &petal, &npc_select(PETAL, talk), now + 2);
    assert!(
        talked.changed,
        "the refused turn-in option consumed the open NPC menu"
    );
    assert_eq!(pending_turnin(&game), Some(QUEST));
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
    let petal = MAP.at(CHARACTER, PETAL);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    // The real character save envelope, not a `GameplayState` serde round trip.
    let mut game = fixture.save_and_reload(&game, &petal);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    // The held end screen (Talk done, 49/14 not yet sent) also survives a save.
    talk(&mut game, &petal, PETAL, now);
    let mut game = fixture.save_and_reload(&game, &petal);
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
    // The seaside row is the legacy Snowhill spawn T698 kept. The POI-anchored
    // Wilds row for the same Petal sits on the Frost Ridge Speedway POI (its
    // `placement_candidates` name poi_id 33), so a bounded distance proves the
    // draw is at the speedway rather than trusting the row's own position.
    let poi = content_position("wilds.json", SPEEDWAY_POI);
    let (dx, dz) = (
        f64::from(position[0]) - f64::from(poi[0]),
        f64::from(position[2]) - f64::from(poi[2]),
    );
    let distance = (dx * dx + dz * dz).sqrt();
    assert!(
        distance < 40.0,
        "Petal's row is {distance:.1} m from the Frost Ridge Speedway POI row {SPEEDWAY_POI}"
    );
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
