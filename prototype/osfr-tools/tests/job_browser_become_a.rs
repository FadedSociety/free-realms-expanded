//! T542: each trained job's original Become-a quest now exists as one enabled regional
//! quest, mapped from the 147/1 Jobs-window row so the client's Start quest (49/15) works.
//!
//! Data-only slice. The shipped `job_browser.json` maps every row's original quest id to a
//! regional record; the trainer in `job_trainers.json` is the giver, the one talk target and
//! the turn-in (the original was started from the Jobs window, and the 49/15 offer path needs
//! a giver: `gameplay/npc_quests.rs::register_breadcrumb_quests`). A finished quest pays the
//! W110 five-coin fallback and no job XP, so a locked profile can never refuse the turn-in
//! (`tests/quest_job_xp.rs::locked_profile_turnin_is_refused_cleanly`).
//!
//! Every request here runs the production path in `tests/support/npc_quest_chain.rs`: the
//! 49/15 breadcrumb start (`quest_journal`), the 49/1 offer decoded with the production
//! reader, 49/2 accept, the Talk that credits the step and holds the 49/13, the 26/6 that
//! releases it, and the 49/14 turn-in. Rows are read only through the shipped records
//! (`tests/support/quest_records.rs`) and `data/content`.
//!
//! Client readers (2014 client, no new wire): the 147/1 ActivityProfileList reader
//! `0x00be6f20` (Jobs window sender `0x00c18aa0`) builds the row whose QuestId is the
//! original id; 49/15 `QuestStartBreadcrumbPacket` writer `0x00a91730`; 49/1
//! `QuestInfoPacket` `0x00c7b7a0`; 49/13 `QuestEndPacket::ReadFields` `0x00c7b990`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, accept, breadcrumb_start, coins,
    content_dir, counts, interact, menu_option, npc_select, pending_offer, pending_turnin,
    profile_xp, quest_status, read_json, saved, send, zone_player,
};
use osfr_tools::{
    chat,
    gameplay::Gameplay,
    npc_interaction::decode_quest_offer_49_1,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
    quest_journal::BREADCRUMB_REFUSED,
};
use quest_records::{RawQuest, read_quests, shipped_quest};
use serde_json::Value;
use std::collections::BTreeMap;

/// One mapped Jobs-window row: profile, original id, regional Become-a record, and the
/// trainer the regional quest names (`job_trainers.json`).
struct Mapped {
    profile: i32,
    original: i32,
    regional: u32,
    trainer: u64,
}

/// The rows in `job_browser.json` order.
const MAPPED: [Mapped; 12] = [
    Mapped {
        profile: 137,
        original: 1968,
        regional: 3_070_001,
        trainer: 9_000_000_181,
    },
    Mapped {
        profile: 52,
        original: 1965,
        regional: 3_140_001,
        trainer: 9_000_000_140,
    },
    Mapped {
        profile: 49,
        original: 1960,
        regional: 3_060_047,
        trainer: 9_000_000_391,
    },
    Mapped {
        profile: 48,
        original: 1961,
        regional: 3_080_050,
        trainer: 9_000_000_039,
    },
    Mapped {
        profile: 45,
        original: 1978,
        regional: 3_050_034,
        trainer: 9_000_000_467,
    },
    Mapped {
        profile: 35,
        original: 1952,
        regional: 3_010_035,
        trainer: 9_000_000_387,
    },
    Mapped {
        profile: 32,
        original: 1966,
        regional: 3_150_025,
        trainer: 9_000_000_076,
    },
    Mapped {
        profile: 14,
        original: 1979,
        regional: 3_100_001,
        trainer: 9_000_000_405,
    },
    Mapped {
        profile: 12,
        original: 1967,
        regional: 3_160_035,
        trainer: 9_000_000_324,
    },
    Mapped {
        profile: 11,
        original: 1962,
        regional: 3_090_034,
        trainer: 9_000_000_161,
    },
    Mapped {
        profile: 4,
        original: 1964,
        regional: 3_130_001,
        trainer: 9_000_000_091,
    },
    Mapped {
        profile: 2,
        original: 1963,
        regional: 3_110_026,
        trainer: 9_000_000_223,
    },
];

/// The Fisherman row: the walk this suite drives end to end.
const FISHERMAN: usize = 0;

/// Every trainer row these tests click, and where it lives. Every trainer spawns as its own
/// content key (nine `legacy_data` rows; Dewey's and Landon's `npc_spawns_extra` rows use the
/// same guid), so the client's clicked actor is the content id.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "stillwater_crossing.json",
    files: &[
        (9_000_000_181, "stillwater_crossing.json"),
        (9_000_000_140, "snowhill.json"),
        (9_000_000_391, "wilds.json"),
        (9_000_000_039, "briarwood.json"),
        (9_000_000_467, "sanctuary.json"),
        (9_000_000_387, "greenwood_forest.json"),
        (9_000_000_076, "snowhill.json"),
        (9_000_000_405, "greenwood_forest.json"),
        (9_000_000_324, "wilds.json"),
        (9_000_000_161, "sanctuary.json"),
        (9_000_000_091, "sanctuary.json"),
        (9_000_000_223, "lakeshore.json"),
    ],
    runtime: &[],
};

const OTHER: u64 = CHARACTER + 1;

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 20,
        member_quest_limit: 20,
    }
}

/// The record as the shared harness walks it: giver, turn-in and the one talk target are the
/// trainer; the card ids and the payout come from the shipped record.
fn quest(mapped: &Mapped) -> ZoneQuest {
    let shipped = shipped_quest(mapped.regional);
    ZoneQuest::from_record(shipped, mapped.trainer, mapped.trainer, ADVENTURER)
}

/// The 49/15 Start quest for `mapped`, then the 49/2 accept.
fn start(fixture: &Fixture, mapped: &Mapped) -> Gameplay {
    let mut game = zone_player(fixture, CHARACTER, ADVENTURER);
    let trainer = MAP.at(CHARACTER, mapped.trainer);
    let started = send(
        &mut game,
        &trainer,
        &breadcrumb_start(mapped.original),
        1_000,
    );
    assert!(
        started.changed,
        "quest {}: the 49/15 must offer",
        mapped.regional
    );
    let offers: Vec<Vec<u8>> = started
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .cloned()
        .collect();
    assert_eq!(
        offers.len(),
        1,
        "quest {}: one 49/1 for the mapped id",
        mapped.regional
    );
    let offer = decode_quest_offer_49_1(&offers[0]).expect("the 49/1 decodes");
    assert_eq!(offer.quest_id, mapped.regional);
    assert_eq!(
        offer.name_string_id,
        u32::try_from(shipped_quest(mapped.regional).title_string_id).expect("title id fits u32"),
        "quest {}: the 49/1 title is the record's",
        mapped.regional
    );
    assert_eq!(pending_offer(&game), Some(mapped.regional));
    accept(&mut game, &trainer, mapped.regional, 1_010);
    game
}

#[test]
fn start_quest_offers_and_completes_become_a() {
    let fixture = Fixture::minimal(CHARACTER);
    let mapped = &MAPPED[FISHERMAN];
    let mut game = start(&fixture, mapped);
    let quest = quest(mapped);
    assert_eq!(quest_status(&game, mapped.regional), QuestStatus::Active);

    // Walk to the trainer and talk: the step completes, the held 49/13 shows the five-coin
    // bundle and the 49/14 turn-in pays it once, with no job XP.
    let xp_before = profile_xp(&game, ADVENTURER);
    let coins_before = coins(&game);
    let mut now = 2_000;
    MAP.turn_in(&mut game, CHARACTER, &quest, ADVENTURER, &mut now);
    assert_eq!(coins(&game), coins_before + 5, "five coins");
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before, "no job XP");
    assert_eq!(quest_status(&game, mapped.regional), QuestStatus::Completed);
    assert_eq!(pending_offer(&game), None);
    assert_eq!(pending_turnin(&game), None);
}

#[test]
fn each_mapped_row_reaches_an_enabled_quest() {
    let root = content_dir();
    let engine = QuestEngine::from_split_content(&root, CHARACTER, eligibility(), None)
        .expect("the shipped split catalogue loads");
    let browser = read_json(&root.join("job_browser.json"));
    let trainers = read_json(&root.join("job_trainers.json"));
    let trainer_by_profile: BTreeMap<i64, u64> = trainers["trainers"]
        .as_array()
        .expect("job_trainers.json trainers")
        .iter()
        .filter_map(|row| Some((row["profile"].as_i64()?, row["npc"].as_u64()?)))
        .collect();
    let records: BTreeMap<u32, RawQuest> = read_quests(&root)
        .into_iter()
        .map(|row| (row.id, row))
        .collect();
    let mut mapped = 0usize;
    for row in browser["records"]
        .as_array()
        .expect("job_browser.json records")
    {
        let profile = row["profile_id"].as_i64().expect("a profile id");
        let trainer = *trainer_by_profile.get(&profile).unwrap_or_else(|| {
            panic!("profile {profile}: the Jobs window row has no job_trainers.json trainer")
        });
        let regional = u32::try_from(row["regional_quest_id"].as_u64().unwrap_or_else(|| {
            panic!("profile {profile}: the Jobs window row maps no regional quest")
        }))
        .expect("a u32 quest id");
        assert!(
            engine.is_enabled(regional),
            "profile {profile}: quest {regional} is not offered by the engine"
        );
        assert_eq!(
            engine.giver_npc_id(regional),
            Some(trainer),
            "profile {profile}: quest {regional} giver"
        );
        assert_eq!(
            engine.turnin_npc_id(regional),
            Some(trainer),
            "profile {profile}: quest {regional} turn-in"
        );
        let shipped = records
            .get(&regional)
            .unwrap_or_else(|| panic!("profile {profile}: quest {regional} has no shipped record"));
        assert_eq!(shipped.giver_npc_id, trainer);
        assert!(shipped.enabled, "quest {regional} is enabled");
        assert!(
            shipped.prerequisite_quest_ids.is_empty(),
            "quest {regional}: no prerequisite"
        );
        assert_eq!(shipped.job, None, "quest {regional}: no job gate");
        assert_eq!(shipped.min_level, None, "quest {regional}: no level gate");
        assert_eq!(
            shipped.objectives.len(),
            1,
            "quest {regional}: one objective"
        );
        assert_eq!(shipped.objectives[0]["kind"], Value::from("talk_to_npc"));
        assert_eq!(
            shipped.objectives[0]["npc_id"].as_u64(),
            Some(trainer),
            "quest {regional}: the talk target is the trainer"
        );
        assert!(
            shipped.objectives[0].get("dialogue_string_id").is_none(),
            "quest {regional}: no dialogue line"
        );
        let (rewards, xp) = shipped.reward_totals(profile as i32);
        assert_eq!(rewards, 5, "quest {regional}: five coins");
        assert_eq!(xp, 0, "quest {regional}: no job XP");
        assert!(
            shipped
                .rewards
                .iter()
                .all(|row| row["job_xp"].as_object().is_some_and(|map| map.is_empty()))
        );
        mapped += 1;
    }
    assert_eq!(mapped, MAPPED.len(), "one enabled Become-a quest per row");
    // Every row this suite walks is the mapping the shipped table carries.
    for expected in &MAPPED {
        let row = records
            .get(&expected.regional)
            .unwrap_or_else(|| panic!("quest {} has no record", expected.regional));
        assert_eq!(
            row.giver_npc_id, expected.trainer,
            "profile {}",
            expected.profile
        );
        assert_eq!(row.objectives[0]["npc_id"].as_u64(), Some(expected.trainer));
    }
}

#[test]
fn completed_become_a_is_refused_again() {
    let fixture = Fixture::minimal(CHARACTER);
    let mapped = &MAPPED[FISHERMAN];
    let mut game = start(&fixture, mapped);
    let quest = quest(mapped);
    let mut now = 3_000;
    MAP.turn_in(&mut game, CHARACTER, &quest, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, mapped.regional), QuestStatus::Completed);

    // A second Start quest for the same original id: the refusal line, no state at all.
    let trainer = MAP.at(CHARACTER, mapped.trainer);
    let before = saved(&game);
    let refused = send(
        &mut game,
        &trainer,
        &breadcrumb_start(mapped.original),
        now + 10,
    );
    assert!(!refused.changed, "a completed quest was started again");
    assert_eq!(refused.replies, vec![chat::system(BREADCRUMB_REFUSED)]);
    assert_eq!(saved(&game), before, "the refused 49/15 changed the save");
    assert_eq!(pending_offer(&game), None);
    assert_eq!(quest_status(&game, mapped.regional), QuestStatus::Completed);
}

#[test]
fn the_trainer_still_teaches_the_job() {
    // The trainer menu is the only job unlock; the Become-a quest grants nothing but coins.
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mapped = &MAPPED[FISHERMAN];
    assert!(
        !game
            .state()
            .profiles
            .as_ref()
            .expect("the fixture has profiles")
            .unlocked
            .contains(&mapped.profile),
        "the fixture starts without Fisherman"
    );
    let trainer = MAP.at(CHARACTER, mapped.trainer);
    let menu = interact(&mut game, &trainer, mapped.trainer, 4_000);
    // `job_trainers.json` label for Reed's row: the same client id as the quest title.
    let learn = menu_option(&menu, 412_490);
    let learned = send(
        &mut game,
        &trainer,
        &npc_select(mapped.trainer, learn),
        4_001,
    );
    assert!(
        learned.changed,
        "the trainer's Become a Fisherman must commit"
    );
    assert!(
        game.state()
            .profiles
            .as_ref()
            .expect("the fixture has profiles")
            .unlocked
            .contains(&mapped.profile),
        "the trainer menu unlocked Fisherman"
    );
    // Learning the job did not start the quest: the Jobs window's Start still does.
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .expect("the fixture bound the quest catalogue")
            .quests
            .contains_key(&mapped.regional),
        "the trainer menu must not start the Become-a quest"
    );
}

#[test]
fn two_players_are_independent() {
    let first = Fixture::minimal(CHARACTER);
    let second = Fixture::minimal(OTHER);
    let mapped = &MAPPED[FISHERMAN];
    let quest = quest(mapped);

    let mut one = start(&first, mapped);
    let mut now = 5_000;
    MAP.turn_in(&mut one, CHARACTER, &quest, ADVENTURER, &mut now);
    assert_eq!(quest_status(&one, mapped.regional), QuestStatus::Completed);

    // The other character never saw the first one's start or completion.
    let mut two = zone_player(&second, OTHER, ADVENTURER);
    assert!(
        !two.state()
            .npc_quests
            .as_ref()
            .expect("the fixture bound the quest catalogue")
            .quests
            .contains_key(&mapped.regional)
    );
    let trainer = MAP.at(OTHER, mapped.trainer);
    let started = send(
        &mut two,
        &trainer,
        &breadcrumb_start(mapped.original),
        5_100,
    );
    assert!(started.changed, "the second player's 49/15 must offer");
    assert_eq!(pending_offer(&two), Some(mapped.regional));
    accept(&mut two, &trainer, mapped.regional, 5_110);
    assert_eq!(quest_status(&two, mapped.regional), QuestStatus::Active);
    assert_eq!(quest_status(&one, mapped.regional), QuestStatus::Completed);
}

#[test]
fn progress_survives_a_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mapped = &MAPPED[FISHERMAN];
    let game = start(&fixture, mapped);
    let quest = quest(mapped);

    // Reconnect mid-quest: the accepted state and its empty counter reload as saved.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, mapped.regional), QuestStatus::Active);
    assert_eq!(counts(&game, mapped.regional), [0]);
    assert_eq!(coins(&game), 0);

    let mut now = 6_000;
    MAP.turn_in(&mut game, CHARACTER, &quest, ADVENTURER, &mut now);
    assert_eq!(coins(&game), 5);
    assert_eq!(quest_status(&game, mapped.regional), QuestStatus::Completed);

    // The completion is durable: another reconnect keeps it and pays nothing more.
    let game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, mapped.regional), QuestStatus::Completed);
    assert_eq!(coins(&game), 5);
}
