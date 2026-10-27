//! T732: the Wildwood round-3 quests whose fixed objective specs the server can run.
//!
//! Two records in `data/content/quests/kart_driver.json` are enabled:
//! - `3080007` Kart Driver: Challenging T.R. (quest-lines/quests.json q3080007): giver
//!   Linx Redline 9500000164, one Talk to T.R. Walker Jr. 9500000950. The 49/13 end
//!   screen opens at T.R. because he is the only objective's named target.
//! - `3080009` Kart Driver: Exploder's League Sponsorship (q3080009): giver and only
//!   target Blammo 9500000872.
//!
//! The other three rows in `wiring/round3/wildwood.json` stay disabled. `3080004`,
//! `3080011` and `3080012` wait on `finish_minigame` ids 628, 632 and 633, which are
//! client MiniGameData race rows; the only kart producer credits the content
//! speedway 31 (`KartGateway::claim_reward_with_event` -> `KartRewardClaim.minigame_id`
//! -> `Gameplay::tick_world`), so no server event can match those ids. `3080004` also
//! has no evidenced giver, and `3080011`'s Big Paulie has no content row (only a
//! reference spawn). The reasons are in `notes/Autopilot/T732.md`.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`: the
//! 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with the production reader), 49/2
//! accept, the Talk that holds the 49/13, the 26/6 that releases it and the 49/14
//! turn-in. Card ids, end text and rewards are read from the shipped record, never
//! pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2
//! `0x00a86400`, 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest
//! `14..18`, end text `18..22`, one dword, then the reward bundle `0x008e7930`), 49/14
//! `0x00a864c0`, 26/3 body reader `0x00a9ef10`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, end_screen_reward, finish_quest,
    pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status, read_json,
    release_end_screen, saved, talk, zone_player,
};
use osfr_tools::quest_engine::QuestStatus;
use quest_records::shipped_quest;
use serde_json::json;

const KART: i32 = 48;
const TR: u32 = 3_080_007;
const LEAGUE: u32 = 3_080_009;
const LINX: u64 = 9_500_000_164;
const TR_WALKER: u64 = 9_500_000_950;
const BLAMMO: u64 = 9_500_000_872;
const VERVE: u64 = 9_500_000_956;
const OTHER: u64 = CHARACTER + 1;

/// Every NPC row is in `wilds.json` except Linx, whose row sits in Briarwood.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "wilds.json",
    files: &[(LINX, "briarwood.json")],
    runtime: &[],
};

fn challenge() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(TR), LINX, TR_WALKER, KART)
}

fn sponsorship() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(LEAGUE), BLAMMO, BLAMMO, KART)
}

#[test]
fn challenging_tr_is_offered_by_linx_and_paid_once() {
    let quest = challenge();
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, KART);
    let (linx, walker) = (MAP.at(CHARACTER, LINX), MAP.at(CHARACTER, TR_WALKER));
    let mut now = 1_000;
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, KART));
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, TR), [0]);
    assert_eq!(
        shipped_quest(TR).objectives[0]["npc_id"].as_u64(),
        Some(TR_WALKER)
    );

    // T.R. has no hand-in before the challenge is spoken.
    assert_packet_changes_nothing(
        &mut game,
        &walker,
        &quest_end_reply(TR),
        now,
        "an early 49/14",
    );
    // The only objective is a Talk to T.R., so his Talk also opens the end screen.
    MAP.turn_in(&mut game, CHARACTER, &quest, KART, &mut now);
    assert_eq!(counts(&game, TR), [1]);
    assert_eq!(quest_status(&game, TR), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, KART), xp_before + quest.profile_xp);

    assert_packet_changes_nothing(
        &mut game,
        &walker,
        &quest_end_reply(TR),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &linx, LINX, &mut now, "Linx after");
    assert_no_offer(&mut game, &walker, TR_WALKER, &mut now, "T.R. after");
}

#[test]
fn exploders_league_sponsorship_is_offered_and_paid_once() {
    let quest = sponsorship();
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, KART);
    let blammo = MAP.at(CHARACTER, BLAMMO);
    let mut now = 1_000;
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, KART));
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, LEAGUE), [0]);

    assert_packet_changes_nothing(
        &mut game,
        &blammo,
        &quest_end_reply(LEAGUE),
        now,
        "an early 49/14",
    );
    MAP.turn_in(&mut game, CHARACTER, &quest, KART, &mut now);
    assert_eq!(counts(&game, LEAGUE), [1]);
    assert_eq!(quest_status(&game, LEAGUE), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + quest.coins);
    assert_eq!(profile_xp(&game, KART), xp_before + quest.profile_xp);

    assert_packet_changes_nothing(
        &mut game,
        &blammo,
        &quest_end_reply(LEAGUE),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &blammo, BLAMMO, &mut now, "Blammo after");
}

#[test]
fn only_the_active_kart_driver_is_offered_and_forged_packets_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    // The line is Kart Driver only: neither giver offers while another job is active,
    // and a forged 49/2 is refused without touching the save.
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, LINX),
        LINX,
        &mut now,
        "Linx as Adventurer",
    );
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, BLAMMO),
        BLAMMO,
        &mut now,
        "Blammo as Adventurer",
    );
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, LINX),
        &quest_reply(TR, true),
        now,
        "a forged 49/2 for Challenging T.R.",
    );
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, BLAMMO),
        &quest_reply(LEAGUE, true),
        now + 1,
        "a forged 49/2 for the sponsorship",
    );

    // The last accepted offer may be replayed, and the end screen cannot be forced.
    let mut game = zone_player(&fixture, CHARACTER, KART);
    let walker = MAP.at(CHARACTER, TR_WALKER);
    now += 100;
    MAP.accept(&mut game, CHARACTER, &challenge(), &mut now);
    now += 100;
    assert_packet_changes_nothing(
        &mut game,
        &walker,
        &quest_reply(TR, true),
        now,
        "a replayed 49/2",
    );
    assert_packet_changes_nothing(
        &mut game,
        &walker,
        &quest_end_reply(TR),
        now + 1,
        "an early 49/14",
    );
    // Verve gives only a disabled quest, so this Talk advances nothing.
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, VERVE),
        VERVE,
        &mut now,
        "Verve while the challenge is open",
    );
}

#[test]
fn progress_survives_reconnect_and_players_are_independent() {
    let (challenge, sponsorship) = (challenge(), sponsorship());
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = zone_player(&fixture, CHARACTER, KART);
    let mut b = zone_player(&fixture, OTHER, KART);
    let mut now = 1_000;
    MAP.accept(&mut a, CHARACTER, &challenge, &mut now);
    MAP.accept(&mut b, OTHER, &sponsorship, &mut now);
    assert_eq!(counts(&a, TR), [0]);
    let b_before = saved(&b);

    // A disconnect after accepting keeps the quest open; the challenge still works.
    let mut a = fixture.reload_serialized(&a);
    assert_eq!(quest_status(&a, TR), QuestStatus::Active);
    assert_eq!(counts(&a, TR), [0]);
    let walker = MAP.at(CHARACTER, TR_WALKER);
    let talked = talk(&mut a, &walker, TR_WALKER, now);
    assert!(talked.changed);
    now += 10;
    // The held end screen is save state: reconnect, reopen it at T.R. and pay. This is
    // `ZoneQuests::turn_in` minus its first Talk, which already happened before the drop.
    let mut a = fixture.reload_serialized(&a);
    assert_eq!(pending_turnin(&a), Some(TR));
    talk(&mut a, &walker, TR_WALKER, now);
    let end = release_end_screen(&mut a, &walker, TR, TR_WALKER, challenge.end_text, now + 2);
    assert_eq!(end_screen_reward(&end), (challenge.coins, challenge.xp));
    finish_quest(
        &mut a,
        &walker,
        TR,
        KART,
        challenge.coins,
        challenge.profile_xp,
        now + 4,
    );
    now += 10;
    let mut a = fixture.reload_serialized(&a);
    assert_eq!(quest_status(&a, TR), QuestStatus::Completed);
    assert_no_offer(
        &mut a,
        &MAP.at(CHARACTER, LINX),
        LINX,
        &mut now,
        "Linx after",
    );

    // A's run never touched B, and B's own quest still pays once.
    assert_eq!(saved(&b), b_before, "A's run changed B");
    let paid = coins(&b);
    MAP.turn_in(&mut b, OTHER, &sponsorship, KART, &mut now);
    assert_eq!(quest_status(&b, LEAGUE), QuestStatus::Completed);
    assert_eq!(coins(&b), paid + sponsorship.coins);
}

/// The enabled records match the checked objective specs, the dialog ids and a
/// non-empty payout read from the same data the runtime loader reads. The three
/// rows whose `finish_minigame` race ids have no server producer stay disabled.
#[test]
fn shipped_records_keep_the_checked_objectives_and_payouts() {
    let checked: [(u32, u64, u64, u32, u32, u32); 2] = [
        (TR, LINX, TR_WALKER, 394_235, 101_178, 101_160),
        (LEAGUE, BLAMMO, BLAMMO, 394_484, 394_486, 394_485),
    ];
    for (quest, giver, target, title, intro, description) in checked {
        let record = shipped_quest(quest);
        assert!(record.enabled, "quest {quest} is enabled");
        assert_eq!(record.job, Some(KART));
        assert_eq!(record.giver_npc_id, giver);
        assert_eq!(record.title_string_id, u64::from(title));
        assert_eq!(record.intro_text_string_id, intro);
        assert_eq!(record.description_string_id, description);
        assert!(record.prerequisite_quest_ids.is_empty());
        assert_eq!(record.objectives.len(), 1);
        assert_eq!(record.objectives[0]["kind"], json!("talk_to_npc"));
        assert_eq!(record.objectives[0]["npc_id"].as_u64(), Some(target));
        let (coins, xp) = record.reward_totals(KART);
        assert!(coins > 0 && xp > 0, "quest {quest} has a real payout");
        let raw = read_json(&content_dir().join("quests/kart_driver.json"));
        let row = raw["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"].as_u64() == Some(u64::from(quest)))
            .expect("the raw quest row");
        assert!(
            row["rewards"][0]["items"]
                .as_array()
                .is_some_and(|items| items.is_empty()),
            "quest {quest} grants no item"
        );
        assert!(
            row["source"]
                .as_array()
                .unwrap()
                .iter()
                .any(|tag| tag["evidence"].as_str().unwrap_or("").contains("(R) T732")),
            "quest {quest} cites T732"
        );
    }
    for disabled in [3_080_004, 3_080_011, 3_080_012] {
        let record = shipped_quest(disabled);
        assert!(
            !record.enabled && record.objectives.is_empty(),
            "quest {disabled} waits on a race id no server path emits"
        );
    }

    // The rows bind the giver and the T.R. end-screen actor; the loader reads both.
    let briarwood = read_json(&content_dir().join("npcs/briarwood.json"));
    let linx = briarwood["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(LINX))
        .expect("Linx's placed row");
    assert!(
        linx["quest_ids_given"]
            .as_array()
            .unwrap()
            .contains(&json!(TR)),
        "Linx gives Challenging T.R."
    );
    let wilds = read_json(&content_dir().join("npcs/wilds.json"));
    let rows = wilds["records"].as_array().unwrap();
    let blammo = rows
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(BLAMMO))
        .expect("Blammo's placed row");
    assert!(
        blammo["quest_ids_given"]
            .as_array()
            .unwrap()
            .contains(&json!(LEAGUE)),
        "Blammo gives the sponsorship"
    );
    let walker = rows
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(TR_WALKER))
        .expect("T.R.'s placed row");
    assert!(
        walker["quests_accepted"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["quest_id"].as_u64() == Some(u64::from(TR))),
        "T.R. is the challenge's end-screen actor"
    );
}
