//! T727/F741: "Demo Derby: So Long, Betty!" (`3060011`) and "Demo Derby: Tucker's Troubles"
//! (`3060013`), the two Thunder Falls quests of the round-3 wiring list that a shipped
//! server can run end to end, enabled in `data/content/quests/demo_derby_driver.json`. Crash
//! Davis (the drawn Wilds `legacy_data` row 9000000289) offers both; each ends with one Talk:
//! Merri De'Flor (9000000146, drawn at Lakeshore by the extra spawn T697 keyed to her row) for
//! Betty, Tucker Thunderjolt (9000000495, a `legacy_data` row T698 placed in the Wilds) for
//! Tucker's Troubles.
//!
//! The other six rows of the list stay disabled (reasons in `notes/Autopilot/T727.md`): each
//! ends in a `finish_minigame` id no server path emits (`tests/support/quest_target_sets.rs`,
//! gated by `enabled_quest_reachability.rs`).
//!
//! Every test drives the shared `ZoneQuests` runner in `tests/support/npc_quest_chain.rs`: the
//! 26/8 -> 26/10 -> 26/6 offer (49/1, decoded with the production reader), the 49/2 accept,
//! the Talk that holds the 49/13, the 26/6 that releases it and the 49/14 turn-in. Rewards are
//! read from the quest data, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`,
//! 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`, end text
//! `18..22`, one dword, then the reward bundle `0x008e7930`), 49/14 `0x00a864c0`; AddNpc
//! reader `0x00921a10` (`startup_world::parse_add_npc`).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer, assert_packet_changes_nothing,
    coins, content_dir, counts, offer, pending_turnin, profile_xp, quest_end_reply, quest_reply,
    quest_status, read_json, saved, seed_active_profile, talk,
};
use osfr_tools::{
    gameplay::Gameplay,
    quest_engine::{CharacterEligibility, QuestEngine, QuestProgress, QuestStatus},
    world::NpcVisibility,
};

const CRASH: u64 = 9_000_000_289;
const RULES_OF_THE_ROAD: u32 = 3_060_004;
const LICENSE_TO_DRIVE: u32 = 3_060_005;
const SEEKING_A_SPONSOR: u32 = 3_060_016;
const DERBY_DRIVER: i32 = 49;
const ADVENTURER: i32 = 1;

/// Crash and Tucker live in `wilds.json`; Merri (drawn by the T697 extra spawn keyed to her
/// row, so her actor guid is her content id) lives in `lakeshore.json`.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "wilds.json",
    files: &[(9_000_000_146, "lakeshore.json")],
    runtime: &[],
};

/// One enabled quest: its record id, the quests a player has finished before Crash offers
/// it and the one NPC its Talk (and turn-in) ends at.
struct Case {
    quest: u32,
    before: &'static [u32],
    target: u64,
}

const BETTY: Case = Case {
    quest: 3_060_011,
    before: &[RULES_OF_THE_ROAD, LICENSE_TO_DRIVE],
    target: 9_000_000_146,
};
const TUCKER: Case = Case {
    quest: 3_060_013,
    before: &[RULES_OF_THE_ROAD, LICENSE_TO_DRIVE, 3_060_011],
    target: 9_000_000_495,
};

impl Case {
    /// The quest as the shared runner walks it, with card ids, end text and reward read
    /// from the shipped record so a retune in data needs no edit here.
    fn quest(&self) -> ZoneQuest {
        let file = read_json(&content_dir().join("quests/demo_derby_driver.json"));
        let record = file["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"].as_u64() == Some(u64::from(self.quest)))
            .expect("the quest record ships");
        let id = |key: &str| record[key].as_u64().unwrap_or(0) as u32;
        let reward = &record["rewards"][0];
        let xp = reward["job_xp"][DERBY_DRIVER.to_string()].as_u64().unwrap() as u32;
        let quest = ZoneQuest {
            id: self.quest,
            giver: CRASH,
            card: [
                id("title_string_id"),
                id("intro_text_string_id"),
                id("description_string_id"),
            ],
            steps: &[],
            turnin: self.target,
            end_text: id("end_text_string_id"),
            coins: reward["coins"].as_u64().unwrap() as u32,
            xp,
            profile_xp: xp,
        };
        assert!(quest.coins > 0 && quest.xp > 0, "the quest pays something");
        quest
    }
}

/// A character with `active` selected and the quests in `done` already completed.
fn player(fixture: &Fixture, active: i32, done: &[u32]) -> Gameplay {
    let mut state = fixture.game_as(CHARACTER, None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER, DERBY_DRIVER]);
    for id in done {
        state.npc_quests.as_mut().unwrap().quests.insert(
            *id,
            QuestProgress {
                status: QuestStatus::Completed,
                counts: Vec::new(),
            },
        );
    }
    fixture.game_as(CHARACTER, Some(state))
}

/// An active derby driver who has finished everything Crash needs before offering `case`.
fn ready_for(fixture: &Fixture, case: &Case) -> Gameplay {
    player(fixture, DERBY_DRIVER, case.before)
}

#[test]
fn each_quest_is_offered_by_crash_ended_by_its_npc_and_pays_once() {
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    for case in [&BETTY, &TUCKER] {
        assert_eq!(
            visibility.content_key(case.target),
            Some(case.target),
            "quest {}: the target is a drawn, talkable actor",
            case.quest
        );
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = ready_for(&fixture, case);
        let quest = case.quest();
        let (coins_before, xp_before) = (coins(&game), profile_xp(&game, DERBY_DRIVER));
        let mut now = 1_000;

        MAP.accept(&mut game, CHARACTER, &quest, &mut now);
        assert_eq!(counts(&game, case.quest), [0]);
        MAP.turn_in(&mut game, CHARACTER, &quest, DERBY_DRIVER, &mut now);

        assert_eq!(coins(&game), coins_before + quest.coins);
        assert_eq!(
            profile_xp(&game, DERBY_DRIVER),
            xp_before + quest.profile_xp
        );
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Completed);

        // Replays and a second Talk change nothing.
        let target = MAP.at(CHARACTER, case.target);
        assert_packet_changes_nothing(
            &mut game,
            &target,
            &quest_end_reply(case.quest),
            now + 100,
            "a duplicate 49/14",
        );
        now += 200;
        // Tucker is also the giver of Seeking a Sponsor (T674), which follows Tucker's Troubles.
        if case.quest == TUCKER.quest {
            offer(&mut game, &target, case.target, SEEKING_A_SPONSOR, now);
        } else {
            assert_no_offer(
                &mut game,
                &target,
                case.target,
                &mut now,
                "the target after",
            );
        }
    }
}

#[test]
fn tuckers_troubles_waits_for_betty_and_each_credits_only_its_own_npc() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = ready_for(&fixture, &BETTY);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &BETTY.quest(), &mut now);

    // Betty open: a forged accept of Tucker's Troubles is refused.
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, CRASH),
        &quest_reply(TUCKER.quest, true),
        now,
        "a forged 49/2 before Betty is done",
    );
    MAP.turn_in(&mut game, CHARACTER, &BETTY.quest(), DERBY_DRIVER, &mut now);

    // Betty done: Crash offers Tucker's Troubles next, and Tucker's Talk completes only it.
    MAP.accept(&mut game, CHARACTER, &TUCKER.quest(), &mut now);
    MAP.turn_in(
        &mut game,
        CHARACTER,
        &TUCKER.quest(),
        DERBY_DRIVER,
        &mut now,
    );
    assert_eq!(quest_status(&game, BETTY.quest), QuestStatus::Completed);
    assert_eq!(quest_status(&game, TUCKER.quest), QuestStatus::Completed);
}

#[test]
fn only_a_licensed_derby_driver_is_offered_them_and_refusals_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let crash = MAP.at(CHARACTER, CRASH);
    let mut now = 1_000;

    // Another job active: nothing is offered and a forged accept changes nothing.
    let mut game = player(&fixture, ADVENTURER, TUCKER.before);
    assert_no_offer(&mut game, &crash, CRASH, &mut now, "derby job not active");
    for case in [&BETTY, &TUCKER] {
        now += 100;
        assert_packet_changes_nothing(
            &mut game,
            &crash,
            &quest_reply(case.quest, true),
            now,
            "a forged 49/2 with another job active",
        );
    }

    // The derby job without the license test: Crash's next step is the license, and a
    // forged accept of either quest changes nothing.
    let mut game = player(&fixture, DERBY_DRIVER, &[RULES_OF_THE_ROAD]);
    for case in [&BETTY, &TUCKER] {
        now += 100;
        assert_packet_changes_nothing(
            &mut game,
            &crash,
            &quest_reply(case.quest, true),
            now,
            "a forged 49/2 before the license test",
        );
    }
    // `offer` itself asserts the one card is the license test.
    offer(&mut game, &crash, CRASH, LICENSE_TO_DRIVE, now + 10);

    // Positive controls: with the job active and the earlier quests done Crash does offer
    // each, so a regression that disables either quest cannot hide behind the refusals above.
    for case in [&BETTY, &TUCKER] {
        now += 100;
        offer(
            &mut ready_for(&fixture, case),
            &crash,
            CRASH,
            case.quest,
            now,
        );
    }
}

#[test]
fn the_target_credits_nothing_before_the_quest_and_the_hand_in_waits_for_it() {
    for case in [&BETTY, &TUCKER] {
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = ready_for(&fixture, case);
        let target = MAP.at(CHARACTER, case.target);
        let mut now = 1_000;
        // Neither target offers a quest of its own.
        assert_no_offer(
            &mut game,
            &target,
            case.target,
            &mut now,
            "the target early",
        );

        MAP.accept(&mut game, CHARACTER, &case.quest(), &mut now);
        let before = saved(&game);
        assert_packet_changes_nothing(
            &mut game,
            &target,
            &quest_end_reply(case.quest),
            now + 10,
            "a forged 49/14 before the visit",
        );
        assert_packet_changes_nothing(
            &mut game,
            &MAP.at(CHARACTER, CRASH),
            &quest_reply(case.quest, true),
            now + 20,
            "a replayed 49/2",
        );
        assert_eq!(saved(&game), before);
        assert_eq!(counts(&game, case.quest), [0]);
        assert_eq!(pending_turnin(&game), None);
    }
}

#[test]
fn progress_and_the_held_end_screen_survive_a_reconnect() {
    for case in [&BETTY, &TUCKER] {
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = ready_for(&fixture, case);
        let quest = case.quest();
        let mut now = 1_000;
        MAP.accept(&mut game, CHARACTER, &quest, &mut now);

        let mut game = fixture.reload_serialized(&game);
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Active);
        let talked = talk(&mut game, &MAP.at(CHARACTER, case.target), case.target, now);
        assert!(talked.changed, "the Talk did not credit the visit");
        assert_eq!(counts(&game, case.quest), [1]);
        assert_eq!(pending_turnin(&game), Some(case.quest));

        // The held hand-in also survives a save: a second Talk keeps it pending (it credits
        // nothing new), and the 26/6 then releases the 49/13 and the 49/14 pays.
        let mut game = fixture.reload_serialized(&game);
        assert_eq!(pending_turnin(&game), Some(case.quest));
        now += 100;
        talk(&mut game, &MAP.at(CHARACTER, case.target), case.target, now);
        assert_eq!(pending_turnin(&game), Some(case.quest));
        MAP.pay_held_turnin(&mut game, CHARACTER, &quest, DERBY_DRIVER, &mut now);
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Completed);
    }
}

#[test]
fn the_loader_binds_crash_as_giver_and_each_target_as_turn_in() {
    let engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility::default(),
        None,
    )
    .expect("the shipped split catalogue loads");
    for case in [&BETTY, &TUCKER] {
        assert!(engine.is_enabled(case.quest));
        assert_eq!(engine.giver_npc_id(case.quest), Some(CRASH));
        // The formal turn-in drives the target's turn-in menu row and the quest marker.
        assert_eq!(engine.turnin_npc_id(case.quest), Some(case.target));
    }
    // Locale 402845 is the completion line the answers tie to Tucker's Troubles; it is the
    // 49/13 end text. Betty has no attested completion line, so hers stays 0.
    assert_eq!(TUCKER.quest().end_text, 402_845);
    assert_eq!(BETTY.quest().end_text, 0);
}
