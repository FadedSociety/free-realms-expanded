//! T367: prove the admin scavenger hunt (3000200) for two players.
//!
//! T247 built the fixed hunt and T281 moved it to Jacque, but only a
//! single-player suite proved it. Playtest night is two or more friends: each
//! must see the offer, keep independent landmark progress, turn in once, keep
//! the 50 coins / 25 Adventurer XP across a relog, and never be affected by
//! another player's refusal, replay or disconnect.
//!
//! Every test drives the production `Gameplay::handle_in_world` NPC/quest
//! router and the production `Gameplay::on_location` movement path, against the
//! real bundled `data/content` hunt record, through the shared fixture in
//! `tests/support/npc_quest_chain.rs`. No production code or content changes:
//! this suite only proves the existing engine.
//!
//! Client evidence (2014 client, `lab/autopilot/ida_query.py`; no new wire
//! shape is built here): 26/8 `008b6bc0` (target u64 + bool), 26/10 `00a36f80`
//! (target u64 + i32 choice), 26/6 `009f5890` (i32 row), 49/1
//! `00c7bb60` -> `00c7b7a0` (quest id, five screen dwords, bool, reward bundle,
//! u64, f32-bits dword, two bools), 49/2 `00a86400` (quest id + accept byte),
//! 49/3 `0093c840` -> `0092b050` -> `008fd770` (count then 107-byte records),
//! 49/9 `00c7af60` (quest, objective, current, progress, bool, dword), 49/10
//! `00c7b070` (four dwords then the bool), 49/13 `00c7b990` (actor 6..14,
//! quest 14..18, end text 18..22, animation 22..26, reward bundle: coins
//! 27..31, XP 31..35), 49/14 `00a864c0` (one quest id). The reward is verified
//! from the 49/13 bundle bytes, so a field slip fails here.

#[path = "support/scavenger_hunt.rs"]
mod scavenger_hunt;

use osfr_tools::{
    character_store::CharacterState,
    chat, game_store,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};
use scavenger_hunt::{
    ADVENTURER, BRAWLER, CARLOS, Fixture, JACQUE, MERIGOLD, PAPPY, QUEST, accept_hunt, active_job,
    assert_move_changes_nothing, assert_no_offer, character_at_for as character, coins,
    content_dir, counts, end_screen_reward, finish_quest, fresh, has_family, has_objective,
    npc_position, offer_hunt, pending_offer, pending_turnin, profile_xp, quest_end_reply,
    quest_reply, quest_status, read_json, refused, refused_interaction, release_end_screen, saved,
    select_job, send, talk, visit_stop, walk_stops,
};
use serde_json::json;
use std::path::Path;

const A: u64 = 367_001;
const B: u64 = 367_002;
const C: u64 = 367_003;
/// A positioned Cobblestone pet vendor with no quest binding; a non-target NPC.
const GLORIA: u64 = 9_500_000_246;
/// A quest id the hunt record does not define; every reply naming it must refuse.
const UNKNOWN_QUEST: u32 = QUEST + 1;

fn at_jacque(character_id: u64) -> CharacterState {
    character(character_id, 0, npc_position(JACQUE))
}

/// A fresh character that has accepted the hunt: one Talk offer, then 49/2.
fn hunt_ready(fixture: &Fixture, character_id: u64, now: &mut u64) -> (Gameplay, CharacterState) {
    let mut game = fresh(fixture, character_id);
    let at = at_jacque(character_id);
    offer_hunt(&mut game, &at, now);
    accept_hunt(&mut game, &at, now);
    (game, at)
}

/// The return Talk completes the fourth ordered goal and holds the 49/13 until
/// the client's 26/6 dialogue choice releases it.
fn return_talk(game: &mut Gameplay, at: &CharacterState, now: &mut u64) {
    let final_talk = talk(game, at, JACQUE, *now);
    *now += 2;
    assert!(
        has_objective(&final_talk.replies, 10, 4),
        "the return Talk completes goal 4"
    );
    assert!(
        !final_talk
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the end screen is held until the 26/6 choice"
    );
    assert_eq!(counts(game, QUEST), [1, 1, 1, 1]);
    assert_eq!(pending_turnin(game), Some(QUEST));
}

fn release_end(game: &mut Gameplay, at: &CharacterState, now: &mut u64) -> Vec<u8> {
    let screen = release_end_screen(game, at, QUEST, JACQUE, 0, *now);
    *now += 2;
    screen
}

/// The 49/12 completed-quest count that follows a turn-in.
fn paid_count(outcome: &Outcome) -> i32 {
    let count = outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 12, None))
        .expect("the completed count follows a turn-in");
    i32::from_le_bytes(count[6..10].try_into().unwrap())
}

/// Save through the real `game_store` envelope, load it back, prove the envelope
/// round-trips the whole `GameplayState`, and rebind it for `character_id`.
fn reload_via_store(
    fixture: &Fixture,
    path: &Path,
    at: &CharacterState,
    game: &Gameplay,
    character_id: u64,
) -> Gameplay {
    let snapshot = game_store::Snapshot {
        character: at.clone(),
        gameplay: Some(game.state()),
    };
    game_store::save(path, &snapshot).expect("the character save commits");
    let loaded = game_store::load_or_create(path, at.clone()).expect("the character save loads");
    assert_eq!(
        loaded.gameplay.as_ref(),
        snapshot.gameplay.as_ref(),
        "the game_store envelope changed the saved gameplay state"
    );
    fixture.game_as(character_id, loaded.gameplay)
}

#[test]
fn two_characters_are_each_offered_and_accept_the_hunt() {
    let fixture = Fixture::minimal(A);
    let mut a = fresh(&fixture, A);
    let mut b = fresh(&fixture, B);
    let a_at = at_jacque(A);
    let b_at = at_jacque(B);
    let mut a_now = 100_000;
    let mut b_now = 200_000;

    // A is offered first: the pending offer and its offered quest are
    // character-owned, so B has neither.
    offer_hunt(&mut a, &a_at, &mut a_now);
    assert_eq!(pending_offer(&a), Some(QUEST));
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Offered);
    assert_eq!(pending_offer(&b), None);
    assert!(
        b.state().npc_quests.as_ref().unwrap().quests.is_empty(),
        "B has no quest before its own offer"
    );
    let a_offered = a.state();
    let a_offered_bytes = saved(&a);

    // A forged 49/2 with no server offer is refused and creates nothing for B.
    refused(
        &mut b,
        &b_at,
        &quest_reply(QUEST, true),
        b_now,
        "B's unoffered accept",
    );
    assert_eq!(pending_offer(&b), None);
    assert!(b.state().npc_quests.as_ref().unwrap().quests.is_empty());
    assert_eq!(a.state(), a_offered, "B's refusal changed A");
    assert_eq!(saved(&a), a_offered_bytes);

    // B is offered independently while A's offer stays open.
    offer_hunt(&mut b, &b_at, &mut b_now);
    assert_eq!(pending_offer(&a), Some(QUEST), "A's offer survives B's");
    assert_eq!(pending_offer(&b), Some(QUEST));
    assert_eq!(a.state(), a_offered, "B's offer changed A");

    // Each accepts its own offer, one after the other.
    accept_hunt(&mut a, &a_at, &mut a_now);
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Active);
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Offered);
    accept_hunt(&mut b, &b_at, &mut b_now);
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Active);
    assert_eq!(counts(&a, QUEST), [0, 0, 0, 0]);
    assert_eq!(counts(&b, QUEST), [0, 0, 0, 0]);
    assert_eq!(pending_offer(&a), None);
    assert_eq!(pending_offer(&b), None);
    assert_eq!(coins(&a), 0);
    assert_eq!(coins(&b), 0);

    // A replays its accept and forges an unknown id: nothing changes and no
    // quest is created for B.
    refused(
        &mut a,
        &a_at,
        &quest_reply(QUEST, true),
        a_now + 1,
        "A's replayed accept",
    );
    refused(
        &mut a,
        &a_at,
        &quest_reply(UNKNOWN_QUEST, true),
        a_now + 2,
        "A's forged accept",
    );
    assert_eq!(counts(&b, QUEST), [0, 0, 0, 0]);
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Active);
    assert_eq!(coins(&b), 0);
}

#[test]
fn each_character_advances_the_three_stops_independently() {
    let fixture = Fixture::minimal(A);
    let mut a_now = 110_000;
    let mut b_now = 120_000;
    let (mut a, _) = hunt_ready(&fixture, A, &mut a_now);
    let (mut b, _) = hunt_ready(&fixture, B, &mut b_now);

    // A visits only the farm.
    visit_stop(&mut a, A, 0, &mut a_now);
    assert_eq!(counts(&a, QUEST), [1, 0, 0, 0]);
    assert_eq!(counts(&b, QUEST), [0, 0, 0, 0]);

    // B visits the farm and the ring; A's whole state and save bytes do not
    // change.
    let a_state = a.state();
    let a_bytes = saved(&a);
    visit_stop(&mut b, B, 0, &mut b_now);
    visit_stop(&mut b, B, 1, &mut b_now);
    assert_eq!(counts(&b, QUEST), [1, 1, 0, 0]);
    assert_eq!(counts(&a, QUEST), [1, 0, 0, 0], "B's moves advanced A");
    assert_eq!(a.state(), a_state, "B's moves changed A's whole state");
    assert_eq!(saved(&a), a_bytes, "B's moves changed A's save bytes");

    // A's next legitimate move advances A only.
    let b_state = b.state();
    let b_bytes = saved(&b);
    visit_stop(&mut a, A, 1, &mut a_now);
    assert_eq!(counts(&a, QUEST), [1, 1, 0, 0]);
    assert_eq!(counts(&b, QUEST), [1, 1, 0, 0]);
    assert_eq!(b.state(), b_state, "A's move changed B's whole state");
    assert_eq!(saved(&b), b_bytes, "A's move changed B's save bytes");

    // A point 30 m outside the next stop and the exact stop in the wrong zone
    // change nothing.
    let mut outside = npc_position(MERIGOLD);
    outside[2] += 30.0;
    assert_move_changes_nothing(&mut a, A, 0, outside, a_now + 1, "30 m outside the stop");
    assert_move_changes_nothing(
        &mut a,
        A,
        1,
        npc_position(MERIGOLD),
        a_now + 2,
        "the wrong-zone stop",
    );
    assert_eq!(counts(&a, QUEST), [1, 1, 0, 0]);
    assert_eq!(counts(&b, QUEST), [1, 1, 0, 0]);
    assert_eq!(b.state(), b_state);
    assert_eq!(saved(&b), b_bytes);
}

#[test]
fn both_return_talks_hold_their_own_49_13_and_turn_in_once() {
    let fixture = Fixture::minimal(A);
    let mut a_now = 130_000;
    let mut b_now = 140_000;
    let (mut a, a_at) = hunt_ready(&fixture, A, &mut a_now);
    let (mut b, b_at) = hunt_ready(&fixture, B, &mut b_now);
    walk_stops(&mut a, A, 3, &mut a_now);
    walk_stops(&mut b, B, 3, &mut b_now);

    // B switches its active job through the production 26/13 path. The hunt's
    // reward row names Adventurer (profile 1), so the switch must not move the XP.
    let switched = send(&mut b, &b_at, &select_job(BRAWLER), b_now);
    b_now += 2;
    assert!(switched.changed, "the job switch commits");
    assert_eq!(active_job(&b), BRAWLER);
    let brawler_xp_before = profile_xp(&b, BRAWLER);

    // Each return Talk completes goal 4 and holds its own 49/13. The held
    // screen pays nothing until the 49/14 reply.
    return_talk(&mut a, &a_at, &mut a_now);
    return_talk(&mut b, &b_at, &mut b_now);
    let a_screen = release_end(&mut a, &a_at, &mut a_now);
    let b_screen = release_end(&mut b, &b_at, &mut b_now);
    assert_eq!(end_screen_reward(&a_screen), (50, 25));
    assert_eq!(end_screen_reward(&b_screen), (50, 25));
    assert_eq!(pending_turnin(&a), Some(QUEST));
    assert_eq!(pending_turnin(&b), Some(QUEST));
    assert_eq!(coins(&a), 0, "the 49/13 display is not a payout");
    assert_eq!(coins(&b), 0);

    // A live delta between accept and turn-in survives: the payout applies to
    // the live inventory/profiles, never to an accept-time snapshot.
    let mut a_state = a.state();
    a_state.inventory.coins += 7;
    a_state
        .profiles
        .as_mut()
        .unwrap()
        .experience
        .insert(ADVENTURER, 3);
    a = fixture.game_as(A, Some(a_state));
    let a_coins_before = coins(&a);
    let a_xp_before = profile_xp(&a, ADVENTURER);
    let a_items_before = a.state().inventory.items.clone();

    // A turns in first: exactly +50/+25 on top of the live delta, 49/4/49/12,
    // Completed, no item; B's held turn-in is untouched.
    let b_held = b.state();
    let b_held_bytes = saved(&b);
    let a_paid = finish_quest(&mut a, &a_at, QUEST, ADVENTURER, 50, 25, a_now + 1);
    a_now += 3;
    assert_eq!(paid_count(&a_paid), 1);
    assert_eq!(coins(&a), a_coins_before + 50);
    assert_eq!(profile_xp(&a, ADVENTURER), a_xp_before + 25);
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Completed);
    assert_eq!(
        a.state().inventory.items,
        a_items_before,
        "the hunt rewards no items"
    );
    assert_eq!(b.state(), b_held, "A's payout changed B's held state");
    assert_eq!(saved(&b), b_held_bytes);

    // B turns in its own copy; profile 1 receives the XP, not the active job.
    let a_paid_state = a.state();
    let a_paid_bytes = saved(&a);
    let b_paid = finish_quest(&mut b, &b_at, QUEST, ADVENTURER, 50, 25, b_now + 1);
    b_now += 3;
    assert_eq!(paid_count(&b_paid), 1);
    assert_eq!(coins(&b), 50);
    assert_eq!(profile_xp(&b, ADVENTURER), 25);
    assert_eq!(profile_xp(&b, BRAWLER), brawler_xp_before);
    assert_eq!(active_job(&b), BRAWLER);
    assert_eq!(a.state(), a_paid_state, "B's payout changed A");
    assert_eq!(saved(&a), a_paid_bytes);
    assert_eq!(coins(&a), a_coins_before + 50);

    // A replay of 49/14 or 49/2 pays nothing and never reaches the other player.
    let a_paid_state = a.state();
    let a_paid_bytes = saved(&a);
    let b_paid_state = b.state();
    let b_paid_bytes = saved(&b);
    refused(
        &mut a,
        &a_at,
        &quest_end_reply(QUEST),
        a_now + 1,
        "A's replayed turn-in",
    );
    refused(
        &mut a,
        &a_at,
        &quest_reply(QUEST, true),
        a_now + 2,
        "A's replayed accept",
    );
    refused(
        &mut b,
        &b_at,
        &quest_end_reply(QUEST),
        b_now + 1,
        "B's replayed turn-in",
    );
    assert_eq!(a.state(), a_paid_state);
    assert_eq!(saved(&a), a_paid_bytes);
    assert_eq!(b.state(), b_paid_state);
    assert_eq!(saved(&b), b_paid_bytes);
    assert_eq!(coins(&a), a_coins_before + 50);
    assert_eq!(coins(&b), 50);
}

#[test]
fn a_refusal_by_one_player_does_not_change_the_other() {
    let fixture = Fixture::minimal(A);
    let mut a_now = 150_000;
    let mut b_now = 160_000;
    let (mut a, a_at) = hunt_ready(&fixture, A, &mut a_now);
    let (mut b, _) = hunt_ready(&fixture, B, &mut b_now);
    let b_before = b.state();
    let b_before_bytes = saved(&b);

    // A's premature turn-in has no pending end screen; forged ids and a forged
    // accept are refused too. Each refusal leaves the whole state and its
    // serialized save bytes equal.
    let before_state = a.state();
    let before_bytes = saved(&a);
    refused(
        &mut a,
        &a_at,
        &quest_end_reply(QUEST),
        a_now + 1,
        "a premature turn-in",
    );
    refused(
        &mut a,
        &a_at,
        &quest_end_reply(UNKNOWN_QUEST),
        a_now + 2,
        "a forged turn-in id",
    );
    refused(
        &mut a,
        &a_at,
        &quest_reply(UNKNOWN_QUEST, true),
        a_now + 3,
        "a forged accept id",
    );

    // A full conversation with a nearby non-target vendor offers and changes
    // nothing; the open menu stays usable (assert_no_offer re-selects Talk).
    let gloria = character(A, 0, npc_position(GLORIA));
    assert_no_offer(&mut a, &gloria, GLORIA, &mut a_now, "the wrong-NPC Talk");

    // An out-of-range 26/8 at the real host is refused before any menu.
    let mut far = npc_position(JACQUE);
    far[2] += 100.0;
    refused_interaction(
        &mut a,
        &character(A, 0, far),
        JACQUE,
        a_now + 1,
        "the out-of-range 26/8",
    );

    // A position 30 m outside a stop and the same stop in another zone change
    // nothing.
    let mut outside = npc_position(PAPPY);
    outside[2] += 30.0;
    assert_move_changes_nothing(&mut a, A, 0, outside, a_now + 2, "30 m outside the stop");
    assert_move_changes_nothing(
        &mut a,
        A,
        1,
        npc_position(PAPPY),
        a_now + 3,
        "the wrong-zone stop",
    );
    assert_eq!(a.state(), before_state, "A's refusals changed A's state");
    assert_eq!(
        saved(&a),
        before_bytes,
        "A's refusals changed A's save bytes"
    );
    assert_eq!(counts(&a, QUEST), [0, 0, 0, 0]);
    assert_eq!(coins(&a), 0);

    // B never moved through any of A's refusals, and B's next legitimate move
    // still advances.
    assert_eq!(counts(&b, QUEST), [0, 0, 0, 0]);
    assert_eq!(b.state(), b_before, "A's refusals changed B");
    assert_eq!(saved(&b), b_before_bytes);
    visit_stop(&mut b, B, 0, &mut b_now);
    visit_stop(&mut b, B, 1, &mut b_now);
    assert_eq!(counts(&b, QUEST), [1, 1, 0, 0]);
    assert_eq!(counts(&a, QUEST), [0, 0, 0, 0]);
    assert_eq!(a.state(), before_state);
    assert_eq!(saved(&a), before_bytes);
}

#[test]
fn both_keep_progress_and_reward_across_reload() {
    let fixture = Fixture::minimal(A);
    let a_path = fixture.path().join("a-gameplay-save.json");
    let b_path = fixture.path().join("b-gameplay-save.json");
    let mut a_now = 170_000;
    let mut b_now = 180_000;
    let (mut a, a_at) = hunt_ready(&fixture, A, &mut a_now);
    let (mut b, b_at) = hunt_ready(&fixture, B, &mut b_now);
    walk_stops(&mut a, A, 2, &mut a_now);
    walk_stops(&mut b, B, 2, &mut b_now);

    // Round trip both saves through the real game_store envelope: the two
    // stops survive a relog, and no reward is paid early.
    a = reload_via_store(&fixture, &a_path, &a_at, &a, A);
    b = reload_via_store(&fixture, &b_path, &b_at, &b, B);
    assert_eq!(counts(&a, QUEST), [1, 1, 0, 0]);
    assert_eq!(counts(&b, QUEST), [1, 1, 0, 0]);
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Active);
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Active);
    refused(
        &mut a,
        &a_at,
        &quest_end_reply(QUEST),
        a_now + 1,
        "A's turn-in after the mid-hunt reload",
    );
    refused(
        &mut b,
        &b_at,
        &quest_end_reply(QUEST),
        b_now + 1,
        "B's turn-in after the mid-hunt reload",
    );
    assert_eq!(coins(&a), 0);
    assert_eq!(coins(&b), 0);

    // Third stop and return Talk: the pending end screen survives a relog and
    // the held 49/13 is replayed when the opener is talked to again.
    visit_stop(&mut a, A, 2, &mut a_now);
    visit_stop(&mut b, B, 2, &mut b_now);
    return_talk(&mut a, &a_at, &mut a_now);
    return_talk(&mut b, &b_at, &mut b_now);
    a = reload_via_store(&fixture, &a_path, &a_at, &a, A);
    b = reload_via_store(&fixture, &b_path, &b_at, &b, B);
    assert_eq!(pending_turnin(&a), Some(QUEST));
    assert_eq!(pending_turnin(&b), Some(QUEST));
    talk(&mut a, &a_at, JACQUE, a_now);
    a_now += 2;
    talk(&mut b, &b_at, JACQUE, b_now);
    b_now += 2;
    assert_eq!(
        end_screen_reward(&release_end(&mut a, &a_at, &mut a_now)),
        (50, 25)
    );
    assert_eq!(
        end_screen_reward(&release_end(&mut b, &b_at, &mut b_now)),
        (50, 25)
    );

    // Each turns in once and keeps the reward.
    let a_paid = finish_quest(&mut a, &a_at, QUEST, ADVENTURER, 50, 25, a_now + 1);
    a_now += 3;
    let b_paid = finish_quest(&mut b, &b_at, QUEST, ADVENTURER, 50, 25, b_now + 1);
    b_now += 3;
    assert_eq!(paid_count(&a_paid), 1);
    assert_eq!(paid_count(&b_paid), 1);
    a = reload_via_store(&fixture, &a_path, &a_at, &a, A);
    b = reload_via_store(&fixture, &b_path, &b_at, &b, B);
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Completed);
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&a), 50);
    assert_eq!(coins(&b), 50);
    assert_eq!(profile_xp(&a, ADVENTURER), 25);
    assert_eq!(profile_xp(&b, ADVENTURER), 25);

    // A replayed 49/14 after the paid reload pays nothing for either player.
    refused(
        &mut a,
        &a_at,
        &quest_end_reply(QUEST),
        a_now + 1,
        "A's replayed turn-in after reload",
    );
    refused(
        &mut b,
        &b_at,
        &quest_end_reply(QUEST),
        b_now + 1,
        "B's replayed turn-in after reload",
    );
    assert_eq!(coins(&a), 50);
    assert_eq!(coins(&b), 50);
    assert_eq!(profile_xp(&a, ADVENTURER), 25);
    assert_eq!(profile_xp(&b, ADVENTURER), 25);
}

#[test]
fn the_event_uses_no_dev_commands_and_no_other_player_credit() {
    let fixture = Fixture::minimal(A);
    let mut a_now = 190_000;
    let mut b_now = 200_000;
    let (mut a, a_at) = hunt_ready(&fixture, A, &mut a_now);
    let (mut b, b_at) = hunt_ready(&fixture, B, &mut b_now);

    // A third character in the same content never receives the hunt's state.
    let c_at = at_jacque(C);
    let mut c = fresh(&fixture, C);
    let c_before = c.state();
    let c_bytes = saved(&c);

    walk_stops(&mut a, A, 3, &mut a_now);
    walk_stops(&mut b, B, 3, &mut b_now);
    // Accepting and walking the stops pays nothing; only the 49/14 turn-in does.
    assert_eq!(coins(&a), 0);
    assert_eq!(coins(&b), 0);
    assert_eq!(profile_xp(&a, ADVENTURER), 0);
    assert_eq!(profile_xp(&b, ADVENTURER), 0);
    return_talk(&mut a, &a_at, &mut a_now);
    return_talk(&mut b, &b_at, &mut b_now);
    let a_screen = release_end(&mut a, &a_at, &mut a_now);
    let b_screen = release_end(&mut b, &b_at, &mut b_now);
    assert_eq!(end_screen_reward(&a_screen), (50, 25));
    assert_eq!(end_screen_reward(&b_screen), (50, 25));
    assert_eq!(coins(&a), 0, "the 49/13 display is not a payout");

    // A's turn-in credits A only; B is holding its own screen and is unchanged.
    let b_held = b.state();
    let b_held_bytes = saved(&b);
    let a_paid = finish_quest(&mut a, &a_at, QUEST, ADVENTURER, 50, 25, a_now + 1);
    assert_eq!(paid_count(&a_paid), 1);
    assert_eq!(b.state(), b_held, "A's payout changed B");
    assert_eq!(saved(&b), b_held_bytes);
    assert_eq!(coins(&b), 0);
    assert_eq!(profile_xp(&b, ADVENTURER), 0);

    // The third character has no offer, no quest and no reward; every forged
    // packet it sends is refused and changes neither its state nor its save.
    assert_eq!(pending_offer(&c), None);
    assert!(
        c.state().npc_quests.as_ref().unwrap().quests.is_empty(),
        "the hunt is never offered to a character that did not talk to the host"
    );
    assert_eq!(coins(&c), 0);
    refused(
        &mut c,
        &c_at,
        &quest_reply(QUEST, true),
        300_000,
        "C's forged accept",
    );
    refused(
        &mut c,
        &c_at,
        &quest_end_reply(QUEST),
        300_001,
        "C's forged turn-in",
    );
    assert_eq!(c.state(), c_before);
    assert_eq!(saved(&c), c_bytes);

    // The event needs no 15/x chat line: a `realm broadcast` request from an
    // ordinary player is refused by the production developer-command gate and
    // starts, completes or pays nothing (the 15/1 builder is production
    // `chat::local_request`, not a hand-written shape).
    let realm = chat::local_request("realm broadcast synthetic probe");
    let before_state = c.state();
    let before_bytes = saved(&c);
    let refusal = c
        .handle_in_world(&realm, &c_at, 300_002)
        .expect("the command gate answers a refused developer command");
    assert!(
        refusal.dev_command_denied,
        "the realm command is denied for an ordinary player"
    );
    assert!(!refusal.changed, "a refused command changes no gameplay");
    assert!(
        !refusal
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, None)
                || has_family(packet, 49, 4, None)
                || has_family(packet, 50, 1, None)),
        "a refused command may not add, complete or pay a quest"
    );
    assert_eq!(c.state(), before_state);
    assert_eq!(saved(&c), before_bytes);

    // B is paid its own copy, and A keeps exactly its own 50/25.
    let a_paid_state = a.state();
    let a_paid_bytes = saved(&a);
    let b_paid = finish_quest(&mut b, &b_at, QUEST, ADVENTURER, 50, 25, b_now + 1);
    assert_eq!(paid_count(&b_paid), 1);
    assert_eq!(coins(&b), 50);
    assert_eq!(profile_xp(&b, ADVENTURER), 25);
    assert_eq!(a.state(), a_paid_state, "B's payout changed A");
    assert_eq!(saved(&a), a_paid_bytes);
    assert_eq!(coins(&a), 50);
    assert_eq!(profile_xp(&a, ADVENTURER), 25);

    // Values live in `data/content/quests/scavenger_hunt.json`: the stops, the
    // 8 m radius and the 50/25 profile-1 reward are the shipped record's, not
    // compiled event constants.
    let hunt = read_json(&content_dir().join("quests/scavenger_hunt.json"));
    let record = &hunt["records"][0];
    assert_eq!(record["id"], json!(QUEST));
    assert_eq!(record["giver_npc_id"], json!(JACQUE));
    let reward = &record["rewards"][0];
    assert_eq!(reward["coins"], json!(50));
    assert_eq!(reward["job_xp"], json!({"1": 25}));
    assert_eq!(reward["items"], json!([]));
    let objectives = record["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 4);
    for (index, npc_id) in [PAPPY, CARLOS, MERIGOLD].into_iter().enumerate() {
        let objective = &objectives[index];
        assert_eq!(objective["kind"], json!("visit_position"));
        assert_eq!(objective["zone_id"], json!(0));
        assert_eq!(objective["radius"], json!(8.0));
        let position: [f32; 3] =
            serde_json::from_value(objective["position"].clone()).expect("a stop position");
        assert_eq!(
            position,
            npc_position(npc_id),
            "stop {index} position is data"
        );
    }
    assert_eq!(objectives[3]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[3]["npc_id"], json!(JACQUE));
}

#[test]
fn a_disconnect_mid_hunt_does_not_stop_the_other() {
    let fixture = Fixture::minimal(A);
    let mut a_now = 210_000;
    let mut b_now = 220_000;
    let (mut a, _) = hunt_ready(&fixture, A, &mut a_now);
    let (mut b, b_at) = hunt_ready(&fixture, B, &mut b_now);
    walk_stops(&mut a, A, 2, &mut a_now);
    walk_stops(&mut b, B, 2, &mut b_now);

    // A drops after two stops; the saved gameplay reloads unchanged through the
    // fixture's reconnect path.
    let a_before = a.state();
    let a_bytes = saved(&a);
    a = fixture.reload(&a);
    assert_eq!(a.state(), a_before, "the reconnect changed A's state");
    assert_eq!(saved(&a), a_bytes, "the reconnect changed A's save");
    assert_eq!(counts(&a, QUEST), [1, 1, 0, 0]);
    assert_eq!(counts(&b, QUEST), [1, 1, 0, 0], "A's reconnect changed B");

    // B, never disconnected, finishes the third stop and its turn-in.
    visit_stop(&mut b, B, 2, &mut b_now);
    return_talk(&mut b, &b_at, &mut b_now);
    let screen = release_end(&mut b, &b_at, &mut b_now);
    assert_eq!(end_screen_reward(&screen), (50, 25));
    let paid = finish_quest(&mut b, &b_at, QUEST, ADVENTURER, 50, 25, b_now + 1);
    assert_eq!(paid_count(&paid), 1);
    assert_eq!(coins(&b), 50);
    assert_eq!(profile_xp(&b, ADVENTURER), 25);

    // A's state and wallet are untouched by B's completion; no production call
    // above returned Err or panicked.
    assert_eq!(a.state(), a_before);
    assert_eq!(saved(&a), a_bytes);
    assert_eq!(counts(&a, QUEST), [1, 1, 0, 0]);
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Active);
    assert_eq!(coins(&a), 0);
    assert_eq!(profile_xp(&a, ADVENTURER), 0);
}
