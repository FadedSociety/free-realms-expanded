//! T676: zone quests round 2 for Southvale, Shrouded Glade, Sunstone Valley, Wildwood and the
//! speedways (`lab/references/quest-lines/wiring/zone-quests-matched/`). Of the 15
//! `wireable_by_match` rows one record runs end to end on this tip and is enabled:
//! `3130031` Postman: Achieve Level 20 (quest-lines/quests.json q3130031). Brin Galeglow offers
//! it, the Postman rank-20 goal and her Talk complete it, and she takes it in. Two rows were
//! already enabled before this task (`3020014` by T729, `3080009` by T732).
//!
//! The other twelve stay disabled, each for a reason `notes/Autopilot/T676.md` lists;
//! `the_other_round_two_rows_stay_off` pins them, and
//! `the_pet_trainer_rows_wait_for_a_pet_trainer_profile` pins the Pet Trainer ones whose blocker
//! is a missing content row (profile 121) and a client fact (Box has no gesture).
//!
//! Every packet goes through the production router (`Gameplay::handle_in_world`) via the shared
//! walkers in `tests/support/npc_quest_chain.rs`: 26/8 -> 26/10 -> 26/6 offer (49/1, decoded
//! with the production reader), 49/2 accept, the developer `realm level` command (the server's
//! own rank change) and the 26/6 -> 49/13 -> 49/14 turn-in. Givers, card ids, end text and
//! payouts are read from the shipped record, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`, 49/13
//! `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`, end text `18..22`,
//! one dword, then the reward bundle `0x008e7930`), 49/14 `0x00a864c0`; 53/7 writer
//! `0x00b5ff50`, sent by `BeginPetGestureRound` `0x00cc9330` only when the trick has a gesture
//! record.

#[path = "support/json.rs"]
mod json;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;
#[path = "support/quest_target_sets.rs"]
mod quest_target_sets;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, objective_complete, pending_turnin,
    quest_end_reply, quest_reply, quest_status, realm_level, reference_content,
    zone_player_unlocked,
};
use osfr_tools::quest_engine::QuestStatus;
use quest_records::{as_u32, shipped_quest};
use quest_target_sets::Targets;

const POSTMAN: i32 = 4;
const PET_TRAINER: u32 = 121;
const BRIN: u64 = 9_000_000_031;
const LEVEL_TWENTY: u32 = 3_130_031;
/// The client's Postman cap text: 387969 "Reach level 20 as a postman." Pinned so a data edit
/// to another rank fails here.
const POSTMAN_GATE: u32 = 20;

/// Brin lives in `shrouded_glade.json`.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "shrouded_glade.json",
    files: &[],
    runtime: &[],
};

fn quest() -> ZoneQuest {
    ZoneQuest::from_record(shipped_quest(LEVEL_TWENTY), BRIN, BRIN, POSTMAN)
}

#[test]
fn brin_postman_level_twenty_is_offered_walked_and_paid_once() {
    let record = shipped_quest(LEVEL_TWENTY);
    assert!(record.enabled);
    assert_eq!(record.job, Some(POSTMAN), "a Postman-only quest");
    assert_eq!(
        as_u32(&record.objectives[0]["level"]),
        Some(POSTMAN_GATE),
        "the shipped rank goal is the client text's level 20"
    );
    let quest = quest();
    assert!(quest.coins > 0, "the reward is authored, not empty");

    let fixture = Fixture::new();
    let mut game = zone_player_unlocked(&fixture, CHARACTER, POSTMAN, &[ADVENTURER, POSTMAN]);
    let brin = MAP.at(CHARACTER, BRIN);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, LEVEL_TWENTY), [0, 0]);

    // The rank goal is open: her Talk and an early hand-in change nothing.
    assert_no_offer(&mut game, &brin, BRIN, &mut now, "rank goal open");
    assert_eq!(pending_turnin(&game), None);
    let early = quest_end_reply(LEVEL_TWENTY);
    assert_packet_changes_nothing(&mut game, &brin, &early, now, "an early 49/14");
    assert_eq!(quest_status(&game, LEVEL_TWENTY), QuestStatus::Active);

    // One rank short never credits it; the gate rank does, and a level-up opens no end screen.
    realm_level(&mut game, &brin, POSTMAN_GATE - 1, now + 20);
    assert_eq!(counts(&game, LEVEL_TWENTY), [0, 0], "one rank short");
    let reached = realm_level(&mut game, &brin, POSTMAN_GATE, now + 30);
    assert!(objective_complete(&reached, LEVEL_TWENTY, 1));
    assert_eq!(counts(&game, LEVEL_TWENTY), [1, 0]);
    assert_eq!(pending_turnin(&game), None);

    now += 40;
    let before = coins(&game);
    MAP.turn_in(&mut game, CHARACTER, &quest, POSTMAN, &mut now);
    assert_eq!(quest_status(&game, LEVEL_TWENTY), QuestStatus::Completed);
    assert_eq!(coins(&game), before + quest.coins);

    // A replayed hand-in and accept pay and add nothing, and she offers nothing more of it.
    let duplicate = quest_end_reply(LEVEL_TWENTY);
    assert_packet_changes_nothing(&mut game, &brin, &duplicate, now, "a duplicate 49/14");
    let replay = quest_reply(LEVEL_TWENTY, true);
    assert_packet_changes_nothing(&mut game, &brin, &replay, now + 5, "a replayed 49/2");
    assert_eq!(coins(&game), before + quest.coins);
    assert_no_offer(&mut game, &brin, BRIN, &mut now, "after");
}

/// An Adventurer who never learned Postman, and one who learned it but plays another job, are
/// offered nothing and a forged accept adds nothing.
#[test]
fn the_wrong_job_changes_nothing() {
    let fixture = Fixture::new();
    let mut now = 1_000;
    for (label, unlocked) in [
        ("another job", &[ADVENTURER][..]),
        ("job learned, not active", &[ADVENTURER, POSTMAN][..]),
    ] {
        let mut game = zone_player_unlocked(&fixture, CHARACTER, ADVENTURER, unlocked);
        let brin = MAP.at(CHARACTER, BRIN);
        assert_no_offer(&mut game, &brin, BRIN, &mut now, label);
        let forged = quest_reply(LEVEL_TWENTY, true);
        assert_packet_changes_nothing(&mut game, &brin, &forged, now, label);
        assert!(
            !game
                .state()
                .npc_quests
                .as_ref()
                .unwrap()
                .quests
                .contains_key(&LEVEL_TWENTY),
            "{label}: quest {LEVEL_TWENTY} must not exist"
        );
    }
}

/// The Pet Trainer rows of the Shrouded Glade file: no profile content exists for the job
/// (profile 121), so no player can make it active and a job gate could never be met; leaving the
/// gate off would offer a members-only job's line to everyone (the T723 `3120010` blocker). The
/// Box trick of `3120013` also has no gesture row, so the client never sends its 53/7.
/// The test fails when the profile gains content, so the next builder re-checks the rows.
#[test]
fn the_pet_trainer_rows_wait_for_a_pet_trainer_profile() {
    let targets = Targets::load(&content_dir(), &reference_content()).unwrap();
    assert!(
        !targets.profiles.contains(&PET_TRAINER),
        "profile 121 gained content; re-check the Pet Trainer rows before wiring them"
    );
    for id in [3_120_011, 3_120_012, 3_120_013, 3_120_015] {
        assert!(!shipped_quest(id).enabled, "quest {id} is enabled");
    }
    // pets.json records that the Box trick (3003) has no PetTrickGesturePoints.txt row, the only
    // way the client sends 53/7 (BeginPetGestureRound 0x00cc9330); Roll Over and Peek-a-boo do.
    let pets = json::pets_json();
    let trick = |id: u64| {
        pets["tricks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|trick| trick["id"] == id)
            .unwrap_or_else(|| panic!("pets.json lists trick {id}"))
    };
    assert!(
        trick(3_003)["source"][0]["evidence"]
            .as_str()
            .is_some_and(|text| text.contains("no PetTrickGesturePoints.txt row"))
    );
    assert!(trick(12)["waypoint_count"].as_u64().unwrap() > 0);
}

/// The rows of the five matched files that stay disabled (reasons in `notes/Autopilot/T676.md`).
/// A row that gains objectives has to be re-checked.
#[test]
fn the_other_round_two_rows_stay_off() {
    for id in [
        // Southvale: Bolstering the Bows, Local Bonesaw Shortage (no blacksmith recipe),
        // Gems For Good Luck, Marriage Proposal (no tourmaline, iron or malachite gather).
        3_020_015, 3_020_017, 3_100_014, 3_100_015,
        // Sunstone Valley: Grunt Work (Sunstone ore), Can We Build It? (tin ore, forging).
        3_020_033, 3_020_036,
        // Wildwood: An Incomplete Set, Wildwood Derby 1000 (no objectives in any source).
        3_060_031, 3_060_044,
        // Speedways: the Demolition Derby Tutorial (its minigame row 289 has no server producer).
        3_060_001,
    ] {
        assert!(!shipped_quest(id).enabled, "quest {id} is enabled");
    }
}
