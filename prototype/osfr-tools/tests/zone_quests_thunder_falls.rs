//! T727: "Demo Derby: So Long, Betty!" (`3060011`) and "Demo Derby: Tucker's Troubles"
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
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`: the
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
    CHARACTER, Fixture, accept, assert_no_offer, assert_packet_changes_nothing, character_at_for,
    coins, content_dir, content_position, counts, end_screen_reward, finish_quest, offer,
    pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status, read_json,
    release_end_screen, saved, seed_active_profile, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::decode_quest_offer_49_1,
    quest_engine::{QuestProgress, QuestStatus},
    world::NpcVisibility,
};
use serde_json::Value;

const CRASH: u64 = 9_000_000_289;
const RULES_OF_THE_ROAD: u32 = 3_060_004;
const LICENSE_TO_DRIVE: u32 = 3_060_005;
const DERBY_DRIVER: i32 = 49;
const ADVENTURER: i32 = 1;

/// One enabled quest: its record id, the quests a player has finished before Crash offers it
/// and the one NPC (and content file) its Talk ends at.
struct Case {
    quest: u32,
    before: &'static [u32],
    target: u64,
    target_file: &'static str,
}

const BETTY: Case = Case {
    quest: 3_060_011,
    before: &[RULES_OF_THE_ROAD, LICENSE_TO_DRIVE],
    target: 9_000_000_146,
    target_file: "lakeshore.json",
};
const TUCKER: Case = Case {
    quest: 3_060_013,
    before: &[RULES_OF_THE_ROAD, LICENSE_TO_DRIVE, 3_060_011],
    target: 9_000_000_495,
    target_file: "wilds.json",
};

impl Case {
    /// The shipped quest record, for the values the tests must not pin.
    fn record(&self) -> Value {
        read_json(&content_dir().join("quests/demo_derby_driver.json"))["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"].as_u64() == Some(u64::from(self.quest)))
            .cloned()
            .expect("the quest record ships")
    }

    fn reward(&self) -> (u32, u32) {
        let record = self.record();
        let row = &record["rewards"][0];
        let (coins, xp) = (
            row["coins"].as_u64().unwrap() as u32,
            row["job_xp"][DERBY_DRIVER.to_string()].as_u64().unwrap() as u32,
        );
        assert!(coins > 0 && xp > 0, "the quest pays something");
        (coins, xp)
    }

    fn target_at(&self) -> CharacterState {
        character_at_for(
            CHARACTER,
            0,
            content_position(self.target_file, self.target),
        )
    }

    /// Crash's offer (49/1 card pinned to the record) and the player's accept.
    fn accepted(&self, game: &mut Gameplay, now: u64) {
        let offered = offer(game, &crash(), CRASH, self.quest, now);
        let card = offered
            .replies
            .iter()
            .find_map(|packet| decode_quest_offer_49_1(packet).ok())
            .expect("the 49/1 card decodes");
        let record = self.record();
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
            .map(|key| record[key].as_u64().unwrap_or(0) as u32),
            "49/1 title, intro and description come from the record"
        );
        accept(game, &crash(), self.quest, now + 2);
    }

    /// The Talk at the target that completes the one goal and holds the 49/13.
    fn visit(&self, game: &mut Gameplay, now: u64) {
        let talked = talk(game, &self.target_at(), self.target, now);
        assert!(talked.changed, "the Talk did not credit the visit");
        assert_eq!(counts(game, self.quest), [1]);
        assert_eq!(pending_turnin(game), Some(self.quest));
    }

    /// The 26/6 that releases the end screen at the target and the 49/14 that pays.
    fn turn_in(&self, game: &mut Gameplay, now: u64) {
        let (coins, xp) = self.reward();
        let end_text = self.record()["end_text_string_id"].as_u64().unwrap_or(0) as u32;
        let end = release_end_screen(
            game,
            &self.target_at(),
            self.quest,
            self.target,
            end_text,
            now,
        );
        assert_eq!(end_screen_reward(&end), (coins, xp));
        finish_quest(
            game,
            &self.target_at(),
            self.quest,
            DERBY_DRIVER,
            coins,
            xp,
            now + 2,
        );
    }
}

fn crash() -> CharacterState {
    character_at_for(CHARACTER, 0, content_position("wilds.json", CRASH))
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
        let (coins_before, xp_before) = (coins(&game), profile_xp(&game, DERBY_DRIVER));
        let mut now = 1_000;
        case.accepted(&mut game, now);
        assert_eq!(counts(&game, case.quest), [0]);

        now += 100;
        case.visit(&mut game, now);
        now += 100;
        case.turn_in(&mut game, now);

        let (paid_coins, paid_xp) = case.reward();
        assert_eq!(coins(&game), coins_before + paid_coins);
        assert_eq!(profile_xp(&game, DERBY_DRIVER), xp_before + paid_xp);
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Completed);

        // Replays and a second Talk change nothing.
        now += 100;
        assert_packet_changes_nothing(
            &mut game,
            &case.target_at(),
            &quest_end_reply(case.quest),
            now,
            "a duplicate 49/14",
        );
        assert_no_offer(
            &mut game,
            &case.target_at(),
            case.target,
            &mut now,
            "the target after",
        );
    }
}

#[test]
fn tuckers_troubles_waits_for_betty_and_each_credits_only_its_own_npc() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = ready_for(&fixture, &BETTY);
    let mut now = 1_000;
    BETTY.accepted(&mut game, now);

    // Betty open: a forged accept of Tucker's Troubles is refused and Merri's Talk is Betty's.
    now += 100;
    assert_packet_changes_nothing(
        &mut game,
        &crash(),
        &quest_reply(TUCKER.quest, true),
        now,
        "a forged 49/2 before Betty is done",
    );
    now += 100;
    BETTY.visit(&mut game, now);
    now += 100;
    BETTY.turn_in(&mut game, now);

    // Betty done: Crash offers Tucker's Troubles next, and Tucker's Talk completes only it.
    now += 100;
    TUCKER.accepted(&mut game, now);
    now += 100;
    TUCKER.visit(&mut game, now);
    now += 100;
    TUCKER.turn_in(&mut game, now);
    assert_eq!(quest_status(&game, BETTY.quest), QuestStatus::Completed);
    assert_eq!(quest_status(&game, TUCKER.quest), QuestStatus::Completed);
}

#[test]
fn only_a_licensed_derby_driver_is_offered_them_and_refusals_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut now = 1_000;

    // Another job active: nothing is offered and a forged accept changes nothing.
    let mut game = player(&fixture, ADVENTURER, TUCKER.before);
    assert_no_offer(&mut game, &crash(), CRASH, &mut now, "derby job not active");
    for case in [&BETTY, &TUCKER] {
        now += 100;
        assert_packet_changes_nothing(
            &mut game,
            &crash(),
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
            &crash(),
            &quest_reply(case.quest, true),
            now,
            "a forged 49/2 before the license test",
        );
    }
    // `offer` itself asserts the one card is the license test.
    offer(&mut game, &crash(), CRASH, LICENSE_TO_DRIVE, now + 10);
}

#[test]
fn the_target_credits_nothing_before_the_quest_and_the_hand_in_waits_for_it() {
    for case in [&BETTY, &TUCKER] {
        let fixture = Fixture::minimal(CHARACTER);
        let mut game = ready_for(&fixture, case);
        let mut now = 1_000;
        // Neither target has an enabled quest of its own to offer.
        assert_no_offer(
            &mut game,
            &case.target_at(),
            case.target,
            &mut now,
            "the target early",
        );

        case.accepted(&mut game, now);
        now += 100;
        let before = saved(&game);
        assert_packet_changes_nothing(
            &mut game,
            &case.target_at(),
            &quest_end_reply(case.quest),
            now + 10,
            "a forged 49/14 before the visit",
        );
        assert_packet_changes_nothing(
            &mut game,
            &crash(),
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
        let mut now = 1_000;
        case.accepted(&mut game, now);

        let mut game = fixture.reload_serialized(&game);
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Active);
        now += 100;
        case.visit(&mut game, now);

        // The visit's held hand-in also survives a save; a second Talk reopens the screen.
        let mut game = fixture.reload_serialized(&game);
        assert_eq!(pending_turnin(&game), Some(case.quest));
        now += 100;
        talk(&mut game, &case.target_at(), case.target, now);
        case.turn_in(&mut game, now + 10);
        assert_eq!(quest_status(&game, case.quest), QuestStatus::Completed);
    }
}
