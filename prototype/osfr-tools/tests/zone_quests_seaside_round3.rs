//! T743: Seaside round 3. Doctor Callahan's Medic: Achieve Level 20 (`3090027`) runs in the
//! client's order: Beth Callahan's Talk, then the rank-20 goal, turned in at Callahan (the
//! quest's giver), never at Beth. A `job_level` step after a Talk step is credited when the
//! step opens (a rank-20 Medic completes it at once) and on every later rank change.
//!
//! Every packet goes through the production router (`Gameplay::handle_in_world`) via the
//! shared `tests/support/npc_quest_chain.rs` walkers: 26/8 -> 26/10 -> 26/6 offer (49/1),
//! 49/2 accept, the Talk step, the developer `realm level` command (the server's own rank
//! change) and the 26/6 -> 49/13 -> 49/14 turn-in. Client readers (IDA, 2014 client, no new
//! wire): 26/3 `0x00a9ef10`, 49/1 `0x00c7bb60`, 49/8 `0x00c7aeb0`, 49/10 `0x00c7b070`,
//! 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (reward bundle `0x008e7930`).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, coins, content_dir, counts, objective_activated,
    objective_complete, pending_turnin, profile_xp, quest_reply, quest_status, read_json, saved,
    seed_active_profile, send, talk,
};
use osfr_tools::{
    account_store::AccountRole,
    chat,
    gameplay::{Gameplay, Outcome},
    quest_engine::{QuestProgress, QuestStatus},
};

const CALLAHAN: u64 = 9_000_000_072;
const BETH: u64 = 9_000_000_023;
const MEDIC: i32 = 11;
const LEVEL_FIVE: u32 = 3_090_006;
const LEVEL_TWENTY: u32 = 3_090_027;
/// Beth's Talk and the rank goal, in the record's order.
const TALK_STEP: u32 = 1;
const RANK_STEP: u32 = 2;
/// The Medic rank curve is one rank per 100 XP (`job_level_quest.rs`).
const RANK_FIFTEEN_XP: u32 = 1_400;
const RANK_TWENTY_XP: u32 = 1_900;

/// Callahan lives in `seaside.json`, Beth in `sanctuary.json` (both drawn bodies are
/// their own content keys).
const MAP: ZoneQuests = ZoneQuests {
    default_file: "seaside.json",
    files: &[(BETH, "sanctuary.json")],
    runtime: &[],
};

/// The reward the record authors; read from data so tuning it does not break the test.
fn gate() -> ZoneQuest {
    let document = read_json(&content_dir().join("quests/medic.json"));
    let record = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"] == LEVEL_TWENTY)
        .expect("the level-20 record is authored");
    assert_eq!(record["enabled"], true, "the quest ships enabled");
    let reward = &record["rewards"][0];
    let xp = reward["job_xp"]["11"].as_u64().unwrap() as u32;
    ZoneQuest {
        id: LEVEL_TWENTY,
        giver: CALLAHAN,
        // Title, intro (his offer speech) and description ids of the record.
        card: [387_577, 387_576, 387_578],
        // The goals (Beth's Talk, then the rank) are driven by hand below.
        steps: &[],
        turnin: CALLAHAN,
        end_text: 387_579,
        coins: reward["coins"].as_u64().unwrap() as u32,
        xp,
        profile_xp: xp,
    }
}

/// A player whose active job is `active`, with Medic at `xp` and Achieve Level 5 done.
fn player(fixture: &Fixture, active: i32, xp: u32, level_five_done: bool) -> Gameplay {
    let mut state = fixture.game_as(CHARACTER, None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER, MEDIC]);
    state
        .profiles
        .as_mut()
        .unwrap()
        .experience
        .insert(MEDIC, xp);
    if level_five_done {
        state.npc_quests.as_mut().unwrap().quests.insert(
            LEVEL_FIVE,
            QuestProgress {
                status: QuestStatus::Completed,
                counts: vec![1],
            },
        );
    }
    fixture.game_as(CHARACTER, Some(state))
}

/// The developer level command: the server's own rank change, as `job_level_quest.rs` uses.
/// It needs the opening engine, so these tests use `Fixture::new`, not `Fixture::minimal`.
fn realm_level_twenty(game: &mut Gameplay, now: u64) -> Outcome {
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    send(
        game,
        &MAP.at(CHARACTER, CALLAHAN),
        &chat::local_request("realm level 20"),
        now,
    )
}

/// Callahan turns the finished quest in and pays; Beth never held the end screen.
fn turn_in_at_callahan(game: &mut Gameplay, quest: &ZoneQuest, now: &mut u64) {
    let (coins_before, xp_before) = (coins(game), profile_xp(game, MEDIC));
    MAP.turn_in(game, CHARACTER, quest, MEDIC, now);
    assert_eq!(quest_status(game, LEVEL_TWENTY), QuestStatus::Completed);
    assert_eq!(coins(game), coins_before + quest.coins);
    assert_eq!(profile_xp(game, MEDIC), xp_before + quest.profile_xp);
}

#[test]
fn a_rank_fifteen_medic_levels_up_after_beth_and_turns_in_at_callahan() {
    let quest = gate();
    let fixture = Fixture::new();
    let mut game = player(&fixture, MEDIC, RANK_FIFTEEN_XP, true);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, LEVEL_TWENTY), [0, 0]);

    // Callahan has nothing to take in yet.
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, CALLAHAN),
        CALLAHAN,
        &mut now,
        "before Beth",
    );

    // Beth's Talk opens the rank step; rank 15 is short of it, so it stays open.
    let beth = MAP.at(CHARACTER, BETH);
    now += 10;
    let talked = talk(&mut game, &beth, BETH, now);
    assert!(objective_complete(&talked, LEVEL_TWENTY, TALK_STEP));
    assert!(objective_activated(&talked, LEVEL_TWENTY, RANK_STEP));
    assert_eq!(counts(&game, LEVEL_TWENTY), [1, 0]);
    assert_eq!(pending_turnin(&game), None, "Beth is not the turn-in");

    now += 10;
    let leveled = realm_level_twenty(&mut game, now);
    assert!(objective_complete(&leveled, LEVEL_TWENTY, RANK_STEP));
    assert_eq!(counts(&game, LEVEL_TWENTY), [1, 1]);
    assert_eq!(profile_xp(&game, MEDIC), RANK_TWENTY_XP);
    assert_eq!(
        pending_turnin(&game),
        None,
        "a level-up opens no end screen"
    );

    // Back at Beth after the quest is ready: still no end screen there.
    now += 10;
    talk(&mut game, &beth, BETH, now);
    assert_eq!(
        pending_turnin(&game),
        None,
        "Beth must not hold the end screen"
    );

    now += 10;
    turn_in_at_callahan(&mut game, &quest, &mut now);
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, CALLAHAN),
        CALLAHAN,
        &mut now,
        "after",
    );
}

#[test]
fn a_rank_twenty_medic_completes_the_rank_step_the_moment_beth_opens_it() {
    let quest = gate();
    let fixture = Fixture::new();
    let mut game = player(&fixture, MEDIC, RANK_TWENTY_XP, true);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(counts(&game, LEVEL_TWENTY), [0, 0], "Beth comes first");

    let talked = talk(&mut game, &MAP.at(CHARACTER, BETH), BETH, now);
    assert!(objective_complete(&talked, LEVEL_TWENTY, RANK_STEP));
    assert_eq!(counts(&game, LEVEL_TWENTY), [1, 1]);
    assert_eq!(pending_turnin(&game), None, "Beth is not the turn-in");
    now += 10;
    turn_in_at_callahan(&mut game, &quest, &mut now);
}

#[test]
fn a_rank_reached_before_beth_is_credited_when_beth_opens_the_step() {
    let quest = gate();
    let fixture = Fixture::new();
    let mut game = player(&fixture, MEDIC, RANK_FIFTEEN_XP, true);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    // The rank goal is not open yet, so levelling up first credits nothing.
    now += 10;
    realm_level_twenty(&mut game, now);
    assert_eq!(counts(&game, LEVEL_TWENTY), [0, 0]);

    now += 10;
    talk(&mut game, &MAP.at(CHARACTER, BETH), BETH, now);
    assert_eq!(counts(&game, LEVEL_TWENTY), [1, 1]);
    now += 10;
    turn_in_at_callahan(&mut game, &quest, &mut now);
}

#[test]
fn refusals_change_nothing() {
    let fixture = Fixture::new();
    let mut now = 1_000;
    // Below the offer rank, without the prerequisite, and with another job active.
    for (label, active, xp, done) in [
        ("rank 14", MEDIC, RANK_FIFTEEN_XP - 100, true),
        ("no Achieve Level 5", MEDIC, RANK_TWENTY_XP, false),
        ("Adventurer active", ADVENTURER, RANK_TWENTY_XP, true),
    ] {
        let mut game = player(&fixture, active, xp, done);
        let callahan = MAP.at(CHARACTER, CALLAHAN);
        assert_no_offer(&mut game, &callahan, CALLAHAN, &mut now, label);
        // A forged accept for a quest never offered adds nothing.
        assert_packet_changes_nothing(
            &mut game,
            &callahan,
            &quest_reply(LEVEL_TWENTY, true),
            now + 50,
            label,
        );
        assert!(
            !game
                .state()
                .npc_quests
                .as_ref()
                .unwrap()
                .quests
                .contains_key(&LEVEL_TWENTY),
            "{label}: the quest must not exist"
        );
    }
}

#[test]
fn beth_talk_before_accept_credits_nothing() {
    let fixture = Fixture::new();
    let mut game = player(&fixture, MEDIC, RANK_FIFTEEN_XP, true);
    let before = saved(&game);
    let talked = talk(&mut game, &MAP.at(CHARACTER, BETH), BETH, 1_000);
    assert!(!talked.changed, "Beth's Talk without the quest is inert");
    assert_eq!(saved(&game), before);
}
