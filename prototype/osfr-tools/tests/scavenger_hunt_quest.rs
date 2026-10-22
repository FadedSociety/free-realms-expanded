//! T247/F272/T281: the playable starting-road scavenger hunt (content-only).
//!
//! The tests drive the production in-world NPC router (`Gameplay::handle_in_world`
//! -> `gameplay/npc.rs` -> `QuestEngine`), the real `Gameplay::on_location`
//! movement path and the real accept/turn-in packets. They never call the quest
//! engine's private helpers as a shortcut. The shared fixture and packet helpers
//! live in `tests/support/npc_quest_chain.rs`; the hunt steps both this suite and
//! the two-player suite drive live in `tests/support/scavenger_hunt.rs`; this
//! file adds only the single-player and content-identity checks.
//!
//! Client evidence (2014 client, checked with `lab/autopilot/ida_query.py`):
//! - 26/8 NPC interact constructor `008b6bc0`: two GUID dwords + one bool.
//! - 26/10 menu select constructor `00a36f80`: two GUID dwords + i32 choice.
//! - 26/6 dialogue response `009f5890`: i32 row (301 is the quest row).
//! - 49/2 QuestReply constructor `00a86400`: quest id + accept byte.
//! - 49/14 QuestEndReply constructor `00a864c0`: one quest id.
//! - 49/1 QuestInfo reader `00c7bb60` -> `00c7b7a0`: quest id, five screen
//!   dwords, bool, reward bundle, u64, f32-bits dword, two bools.
//! - 49/3 QuestAdd reader `0093c840` (fixed prefix and objective count) ->
//!   `0092b050` (count and per-objective loop) -> `008fd770` (the 107-byte
//!   record; its leading id dword is read in `0092b050`). `00c7bd10` is only an
//!   apply entry that calls `009225d0`, not the record layout.
//! - 49/13 QuestEnd reader `00c7b990`: u64 actor, three dwords, reward bundle,
//!   trailing dword.
//! - 35/2 AddNpc reader `00921a10` (`0091cac0` is its caller-side constructor):
//!   Jacque's existing `legacy_data` row supplies the actor; no new actor or
//!   spawn packet is added.
//! `visit_position` is a server-side objective; the client only receives the
//! ordinary 49/9/49/10 objective updates and the 49/8 activation, so there is no
//! new wire shape to match.

#[path = "support/scavenger_hunt.rs"]
mod scavenger_hunt;

use osfr_tools::{
    character_store::CharacterState,
    game_store,
    gameplay::Gameplay,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
    world::NpcVisibility,
};
use scavenger_hunt::{
    ADVENTURER, BRAWLER, CARLOS, Fixture, JACQUE, MERIGOLD, PAPPY, QUEST, accept, accept_hunt,
    active_job, assert_move_changes_nothing, assert_no_offer, character_at_for as character, coins,
    content_dir, counts, finish_quest, fresh, has_family, has_objective, npc_position,
    objective_update, offer, offer_hunt, pending_offer, pending_turnin, profile_xp,
    quest_end_reply, quest_reply, quest_status, read_json, refused, refused_interaction,
    release_end_screen, saved, select_job, send, talk, visit,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CHARACTER: u64 = 281_001;
const OTHER_CHARACTER: u64 = 281_002;
/// Samantha's existing Cobblestone referral. It is unchanged; Ricky Danger is its
/// Talk target, so a player carrying it resolves it at Ricky, not at the hunt host.
const REFERRAL: u32 = 3_000_021;
/// Ricky Danger's pre-existing original chain. Both records name him as giver; the
/// hunt must never be inserted ahead of them.
const CALL_THE_CREW: u32 = 3_000_059;
const BACK_FOR_MORE: u32 = 3_000_060;
const SAMANTHA: u64 = 9_000_000_189;
const RICKY: u64 = 9_500_000_254;
/// The generic stock "Scavenger Hunt " label. The Greenwood Forest minigame title
/// (429342) belongs to a different retail activity and is not reused.
const TITLE_ID: u32 = 3_374;
/// A positioned Cobblestone pet vendor with no quest binding; a non-target NPC.
const GLORIA: u64 = 9_500_000_246;
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        member: false,
        nonmember_quest_limit: 100,
        member_quest_limit: 100,
        ..CharacterEligibility::default()
    }
}

/// One complete hunt through the final Talk and the held 49/13 end screen, so
/// a test can exercise the 49/14 turn-in (or refuse it) at that exact point.
fn open_hunt_end_screen(game: &mut Gameplay, jacque: &CharacterState, now: &mut u64) {
    offer_hunt(game, jacque, now);
    accept_hunt(game, jacque, now);
    for (position, expected) in [
        (npc_position(PAPPY), [1, 0, 0, 0]),
        (npc_position(CARLOS), [1, 1, 0, 0]),
        (npc_position(MERIGOLD), [1, 1, 1, 0]),
    ] {
        visit(game, jacque.guid, position, now, expected);
    }
    let final_talk = talk(game, jacque, JACQUE, *now);
    *now += 2;
    assert!(has_objective(&final_talk.replies, 10, 4));
    release_end_screen(game, jacque, QUEST, JACQUE, 0, *now);
    *now += 2;
    assert_eq!(pending_turnin(game), Some(QUEST));
}

#[test]
fn jacque_offers_the_hunt_first_to_a_new_character() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let jacque = character(CHARACTER, 0, npc_position(JACQUE));
    let mut now = 1_000;

    let offered = offer_hunt(&mut game, &jacque, &mut now);
    let offers: Vec<_> = offered
        .replies
        .iter()
        .filter(|packet| has_family(packet, 49, 1, None))
        .collect();
    assert_eq!(offers.len(), 1, "one Talk offers exactly one quest");
    let offer = offers[0];
    // 49/1 layout (reader 0x00c7b7a0): quest at 6..10, intro at 10..14, the
    // five ShowQuestStartScreen dwords after it, with the client-verified card
    // name at 18..22 and description at 22..26. The builder also mirrors the
    // title into wire dwords 2 (14..18) and 5 (26..30); both meanings are
    // unresolved (R), so this test does not pin those server-side choices.
    assert_eq!(
        u32::from_le_bytes(offer[6..10].try_into().unwrap()),
        QUEST,
        "the first offer is the hunt, not the Samantha referral"
    );
    assert_eq!(
        u32::from_le_bytes(offer[10..14].try_into().unwrap()),
        0,
        "intro text is not authored; it stays 0 under the hollow-text policy"
    );
    assert_eq!(
        u32::from_le_bytes(offer[18..22].try_into().unwrap()),
        TITLE_ID,
        "the client-verified card name is the stock hunt title"
    );
    assert_eq!(
        u32::from_le_bytes(offer[22..26].try_into().unwrap()),
        0,
        "description text is not authored; it stays 0 under the hollow-text policy"
    );
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(REFERRAL))),
        "Jacque must not offer Samantha's referral"
    );
    assert_eq!(pending_offer(&game), Some(QUEST));
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Offered);
    assert_eq!(counts(&game, QUEST), [0, 0, 0, 0]);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&REFERRAL),
        "a fresh character has not received Samantha's referral"
    );

    // A pending offer is not repeated and the save stays equal.
    assert_no_offer(&mut game, &jacque, JACQUE, &mut now, "a pending hunt offer");

    accept_hunt(&mut game, &jacque, &mut now);
    // The active hunt is not re-offered either.
    assert_no_offer(&mut game, &jacque, JACQUE, &mut now, "an active hunt");
}

#[test]
fn the_four_stops_complete_and_pay_once() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let jacque = character(CHARACTER, 0, npc_position(JACQUE));
    let mut now = 3_000;
    offer_hunt(&mut game, &jacque, &mut now);
    accept_hunt(&mut game, &jacque, &mut now);

    // An early return Talk neither completes the final objective nor opens the
    // end screen, and changes nothing.
    assert_no_offer(&mut game, &jacque, JACQUE, &mut now, "an early return Talk");
    assert_eq!(counts(&game, QUEST), [0, 0, 0, 0]);

    // Stop 1: Pappy Farnum's farm.
    let pappy = visit(
        &mut game,
        CHARACTER,
        npc_position(PAPPY),
        &mut now,
        [1, 0, 0, 0],
    );
    assert!(has_objective(&pappy.replies, 10, 1), "goal 1 completes");
    assert!(has_objective(&pappy.replies, 8, 2), "goal 2 activates");
    // Re-entering the completed stop changes nothing.
    assert_move_changes_nothing(
        &mut game,
        CHARACTER,
        0,
        npc_position(PAPPY),
        now + 1,
        "a replayed visit",
    );
    // Stop 2: the Cobblestone brawling ring.
    let ring = visit(
        &mut game,
        CHARACTER,
        npc_position(CARLOS),
        &mut now,
        [1, 1, 0, 0],
    );
    assert!(has_objective(&ring.replies, 10, 2), "goal 2 completes");
    assert!(has_objective(&ring.replies, 8, 3), "goal 3 activates");
    // Stop 3: Highroad Junction / Merigold.
    let highroad = visit(
        &mut game,
        CHARACTER,
        npc_position(MERIGOLD),
        &mut now,
        [1, 1, 1, 0],
    );
    assert!(has_objective(&highroad.replies, 10, 3), "goal 3 completes");
    assert!(
        has_objective(&highroad.replies, 8, 4),
        "the return Talk activates"
    );

    // The reward row names Adventurer (profile 1) even when another profile is
    // active at turn-in. Switch through the production 26/13 path first.
    let switched = send(&mut game, &jacque, &select_job(BRAWLER), now);
    now += 2;
    assert!(switched.changed, "the job switch commits");
    assert_eq!(active_job(&game), BRAWLER);
    let brawler_xp_before = profile_xp(&game, BRAWLER);

    // Return to Jacque: the final Talk completes goal 4 and holds the end
    // screen until the client chooses the dialogue row.
    let final_talk = talk(&mut game, &jacque, JACQUE, now);
    now += 2;
    assert!(has_objective(&final_talk.replies, 10, 4));
    assert!(objective_update(&final_talk, QUEST, 4, 1));
    assert!(
        !final_talk
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the end screen is released by the 26/6 choice"
    );
    assert_eq!(counts(&game, QUEST), [1, 1, 1, 1]);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    let screen = release_end_screen(&mut game, &jacque, QUEST, JACQUE, 0, now);
    now += 2;
    assert_eq!(screen.len(), 99);
    assert_eq!(i32::from_le_bytes(screen[27..31].try_into().unwrap()), 50);
    assert_eq!(i32::from_le_bytes(screen[31..35].try_into().unwrap()), 25);

    let before_payout = game.state();
    let before_payout_bytes = saved(&game);
    let inventory_before = before_payout.inventory.clone();
    let coins_before = before_payout.inventory.coins;
    let xp_before = profile_xp(&game, ADVENTURER);
    let completed = finish_quest(&mut game, &jacque, QUEST, ADVENTURER, 50, 25, now);
    now += 2;
    let count = completed
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 12, None))
        .expect("the completed count follows");
    assert_eq!(i32::from_le_bytes(count[6..10].try_into().unwrap()), 1);
    // The payout is one atomic state change: the quest becomes Completed, the
    // pending 49/13 state clears and the reward adds exactly 50 coins/25 XP.
    // Compare the whole state and its serialized save bytes, so a partial save
    // or an unexpected item grant fails here.
    let mut expected = before_payout.clone();
    expected.inventory.coins = coins_before.checked_add(50).unwrap();
    expected
        .profiles
        .as_mut()
        .unwrap()
        .experience
        .insert(ADVENTURER, xp_before.checked_add(25).unwrap());
    let quests = expected.npc_quests.as_mut().unwrap();
    quests.quests.get_mut(&QUEST).unwrap().status = QuestStatus::Completed;
    quests.pending_turnin = None;
    quests.pending_turnin_actor = None;
    assert_eq!(
        game.state(),
        expected,
        "the turn-in must change only the quest, the pending screen and the reward"
    );
    assert_eq!(
        saved(&game),
        serde_json::to_vec(&expected).unwrap(),
        "the payout save must be exactly the expected envelope"
    );
    assert_ne!(
        saved(&game),
        before_payout_bytes,
        "the payout must be persisted"
    );
    assert_eq!(
        game.state().inventory.items,
        inventory_before.items,
        "the hunt rewards no items"
    );
    // The real game-save envelope, not only the in-memory serialization, must
    // carry the exact post-payout state across a reload.
    let save_path = fixture.path().join("gameplay-save.json");
    let snapshot = game_store::Snapshot {
        character: jacque.clone(),
        gameplay: Some(expected.clone()),
        pets: None,
    };
    game_store::save(&save_path, &snapshot).unwrap();
    let reloaded = game_store::load_or_create(&save_path, jacque.clone()).unwrap();
    assert_eq!(
        reloaded.gameplay.as_ref(),
        Some(&expected),
        "the save envelope changed the expected payout state"
    );
    let reconnected = fixture.game(reloaded.gameplay);
    assert_eq!(reconnected.state(), expected);
    assert_eq!(coins(&game), coins_before + 50);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 25);
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_xp_before,
        "the active job receives no XP from the hunt's explicit profile-1 row"
    );
    assert_eq!(active_job(&game), BRAWLER);

    // A post-completion Talk offers nothing and pays nothing.
    let paid = game.state();
    let paid_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &jacque,
        JACQUE,
        &mut now,
        "a post-completion Talk",
    );
    assert_eq!(game.state(), paid);
    assert_eq!(saved(&game), paid_bytes);

    // A replayed 49/14, 49/2 or visit after the payout changes nothing.
    refused(
        &mut game,
        &jacque,
        &quest_end_reply(QUEST),
        now + 1,
        "a replayed turn-in",
    );
    refused(
        &mut game,
        &jacque,
        &quest_reply(QUEST, true),
        now + 2,
        "a replayed accept",
    );
    assert_move_changes_nothing(
        &mut game,
        CHARACTER,
        0,
        npc_position(PAPPY),
        now + 3,
        "a post-payout visit",
    );
    assert_eq!(game.state(), paid);
    assert_eq!(saved(&game), paid_bytes);
    assert_eq!(coins(&game), coins_before + 50);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 25);
}

#[test]
fn samantha_still_offers_introduce_yourself_and_the_hunt_does_not_cross() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = fresh(&fixture, CHARACTER);
    let a_samantha = character(CHARACTER, 0, npc_position(SAMANTHA));
    let a_ricky = character(CHARACTER, 0, npc_position(RICKY));
    let a_jacque = character(CHARACTER, 0, npc_position(JACQUE));
    let mut now = 20_000;

    // Samantha's referral record is unchanged: one Talk target, Ricky Danger.
    // `offer` asserts the first (and only) 49/1 is the referral, not the hunt, so
    // an original quest keeps priority at its giver.
    let referral = offer(&mut a, &a_samantha, SAMANTHA, REFERRAL, now);
    now += 4;
    assert!(
        referral
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 1, Some(QUEST))),
        "Samantha must not offer the hunt"
    );
    assert_eq!(quest_status(&a, REFERRAL), QuestStatus::Offered);
    accept(&mut a, &a_samantha, REFERRAL, now);
    now += 4;
    assert_eq!(quest_status(&a, REFERRAL), QuestStatus::Active);

    // The active referral (target Ricky) does not suppress the hunt at Jacque.
    offer_hunt(&mut a, &a_jacque, &mut now);
    accept_hunt(&mut a, &a_jacque, &mut now);
    assert_eq!(quest_status(&a, REFERRAL), QuestStatus::Active);
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Active);

    // Complete the referral at its actual target; the active hunt is untouched
    // and the finished referral is not re-offered.
    let referral_talk = talk(&mut a, &a_ricky, RICKY, now);
    now += 2;
    assert!(referral_talk.changed);
    assert_eq!(counts(&a, REFERRAL), [1]);
    assert_eq!(pending_turnin(&a), Some(REFERRAL));
    release_end_screen(&mut a, &a_ricky, REFERRAL, RICKY, 0, now);
    now += 2;
    finish_quest(&mut a, &a_ricky, REFERRAL, ADVENTURER, 5, 20, now);
    now += 2;
    assert_eq!(quest_status(&a, REFERRAL), QuestStatus::Completed);
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Active);
    assert_eq!(counts(&a, QUEST), [0, 0, 0, 0]);
    assert_no_offer(
        &mut a,
        &a_samantha,
        SAMANTHA,
        &mut now,
        "Samantha after the referral is completed",
    );

    // An active hunt does not suppress the still-unaccepted referral at
    // Samantha for a second character.
    let mut b = fresh(&fixture, OTHER_CHARACTER);
    let b_jacque = character(OTHER_CHARACTER, 0, npc_position(JACQUE));
    let b_samantha = character(OTHER_CHARACTER, 0, npc_position(SAMANTHA));
    let mut b_now = 30_000;
    offer_hunt(&mut b, &b_jacque, &mut b_now);
    accept_hunt(&mut b, &b_jacque, &mut b_now);
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Active);
    let referral_offer = offer(&mut b, &b_samantha, SAMANTHA, REFERRAL, b_now);
    assert!(
        referral_offer
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(REFERRAL))),
        "Samantha still offers Introduce Yourself"
    );
    assert!(
        referral_offer
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 1, Some(QUEST))),
        "Samantha does not offer the active hunt"
    );
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Active);
    assert_eq!(counts(&b, QUEST), [0, 0, 0, 0]);
}

#[test]
fn refusals_and_reconnect_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let jacque = character(CHARACTER, 0, npc_position(JACQUE));
    let mut now = 40_000;

    // The accepted T247/F272 decline behavior: a 49/2 decline is a client choice,
    // so it clears the server's pending offer and removes the offered quest, and
    // that change is persisted. Only a repeated stale decline is inert.
    offer_hunt(&mut game, &jacque, &mut now);
    let offered_state = game.state();
    let offered_bytes = saved(&game);
    let declined = send(&mut game, &jacque, &quest_reply(QUEST, false), now);
    now += 2;
    assert!(declined.changed, "declining clears the pending offer");
    assert_eq!(pending_offer(&game), None);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&QUEST),
        "a declined offer is removed"
    );
    let mut expected = offered_state.clone();
    expected.npc_quests.as_mut().unwrap().pending_offer = None;
    assert!(
        expected
            .npc_quests
            .as_mut()
            .unwrap()
            .quests
            .remove(&QUEST)
            .is_some(),
        "the pre-decline snapshot contains the offered quest"
    );
    assert_eq!(
        game.state(),
        expected,
        "a decline changes only the pending offer and the offered quest"
    );
    assert_ne!(
        saved(&game),
        offered_bytes,
        "the declined offer is removed from the save"
    );
    let declined_state = game.state();
    let declined_bytes = saved(&game);
    refused(
        &mut game,
        &jacque,
        &quest_reply(QUEST, false),
        now,
        "a repeated decline",
    );
    assert_eq!(game.state(), declined_state);
    assert_eq!(saved(&game), declined_bytes);

    // Re-offer, then forge a duplicate accept and a 49/14 with no end screen.
    offer_hunt(&mut game, &jacque, &mut now);
    accept_hunt(&mut game, &jacque, &mut now);
    refused(
        &mut game,
        &jacque,
        &quest_reply(QUEST, true),
        now,
        "a duplicate accept",
    );
    refused(
        &mut game,
        &jacque,
        &quest_end_reply(QUEST),
        now + 1,
        "a 49/14 with no end screen",
    );

    // Wrong NPC: a positioned Cobblestone pet vendor with no quest binding.
    let gloria = character(CHARACTER, 0, npc_position(GLORIA));
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &gloria,
        GLORIA,
        &mut now,
        "the non-target NPC conversation",
    );
    assert_eq!(game.state(), before_state);
    assert_eq!(saved(&game), before_bytes);

    // Out-of-range 26/8, an outside-radius visit and a wrong-zone visit are
    // all refused without state or save bytes.
    let mut far = npc_position(JACQUE);
    far[2] += 100.0;
    refused_interaction(
        &mut game,
        &character(CHARACTER, 0, far),
        JACQUE,
        now + 1,
        "an out-of-range 26/8",
    );
    let mut outside = npc_position(PAPPY);
    outside[2] += 30.0;
    assert_move_changes_nothing(&mut game, CHARACTER, 0, outside, now + 2, "outside radius");
    assert_move_changes_nothing(
        &mut game,
        CHARACTER,
        1,
        npc_position(PAPPY),
        now + 3,
        "wrong zone",
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0, 0, 0, 0]);
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, ADVENTURER), 0);

    // Two stops, disconnect, reconnect: the saved counters survive and no
    // reward is paid early.
    visit(
        &mut game,
        CHARACTER,
        npc_position(PAPPY),
        &mut now,
        [1, 0, 0, 0],
    );
    visit(
        &mut game,
        CHARACTER,
        npc_position(CARLOS),
        &mut now,
        [1, 1, 0, 0],
    );
    let mid_state = game.state();
    let mid_bytes = saved(&game);
    game = fixture.reload(&game);
    assert_eq!(
        game.state(),
        mid_state,
        "reconnect changed the mid-hunt state"
    );
    assert_eq!(
        saved(&game),
        mid_bytes,
        "reconnect changed the mid-hunt save"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [1, 1, 0, 0]);
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, ADVENTURER), 0);

    // Finish the hunt through the real paths.
    visit(
        &mut game,
        CHARACTER,
        npc_position(MERIGOLD),
        &mut now,
        [1, 1, 1, 0],
    );
    let final_talk = talk(&mut game, &jacque, JACQUE, now);
    now += 2;
    assert!(has_objective(&final_talk.replies, 10, 4));
    release_end_screen(&mut game, &jacque, QUEST, JACQUE, 0, now);
    now += 2;
    finish_quest(&mut game, &jacque, QUEST, ADVENTURER, 50, 25, now);
    now += 2;
    let paid_state = game.state();
    let paid_bytes = saved(&game);

    // Reconnect after the successful turn-in: the payout is saved once and a
    // replayed 49/14 does not pay again.
    game = fixture.reload(&game);
    assert_eq!(game.state(), paid_state, "reconnect changed the paid state");
    assert_eq!(saved(&game), paid_bytes, "reconnect changed the paid save");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&game), 50);
    assert_eq!(profile_xp(&game, ADVENTURER), 25);
    refused(
        &mut game,
        &jacque,
        &quest_end_reply(QUEST),
        now + 1,
        "a turn-in replay after reconnect",
    );
    assert_eq!(coins(&game), 50);
    assert_eq!(profile_xp(&game, ADVENTURER), 25);
}

#[test]
fn a_second_player_does_not_share_hunt_progress() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = fresh(&fixture, CHARACTER);
    let mut b = fresh(&fixture, OTHER_CHARACTER);
    let a_jacque = character(CHARACTER, 0, npc_position(JACQUE));
    let b_jacque = character(OTHER_CHARACTER, 0, npc_position(JACQUE));
    let mut a_now = 50_000;
    let mut b_now = 60_000;

    offer_hunt(&mut a, &a_jacque, &mut a_now);
    accept_hunt(&mut a, &a_jacque, &mut a_now);
    offer_hunt(&mut b, &b_jacque, &mut b_now);
    assert_eq!(pending_offer(&b), Some(QUEST));
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Offered);
    assert_eq!(counts(&b, QUEST), [0, 0, 0, 0]);

    visit(
        &mut a,
        CHARACTER,
        npc_position(PAPPY),
        &mut a_now,
        [1, 0, 0, 0],
    );
    // B sees none of A's progress or ledger.
    assert_eq!(counts(&b, QUEST), [0, 0, 0, 0]);
    assert_eq!(
        b.state().npc_quests.as_ref().unwrap().character_id,
        OTHER_CHARACTER
    );
    assert_eq!(coins(&b), 0);
    assert_eq!(profile_xp(&b, ADVENTURER), 0);

    // A's stale/out-of-range request fails for A; B's session and hunt continue.
    let mut far = npc_position(JACQUE);
    far[2] += 100.0;
    refused_interaction(
        &mut a,
        &character(CHARACTER, 0, far),
        JACQUE,
        a_now + 1,
        "A's out-of-range 26/8",
    );
    assert_eq!(counts(&b, QUEST), [0, 0, 0, 0]);
    accept_hunt(&mut b, &b_jacque, &mut b_now);
    visit(
        &mut b,
        OTHER_CHARACTER,
        npc_position(PAPPY),
        &mut b_now,
        [1, 0, 0, 0],
    );
    assert_eq!(counts(&a, QUEST), [1, 0, 0, 0]);
    assert_eq!(counts(&b, QUEST), [1, 0, 0, 0]);

    // A finishes first; B stays unpaid at its first stop.
    let ring = visit(
        &mut a,
        CHARACTER,
        npc_position(CARLOS),
        &mut a_now,
        [1, 1, 0, 0],
    );
    assert!(has_objective(&ring.replies, 10, 2));
    let highroad = visit(
        &mut a,
        CHARACTER,
        npc_position(MERIGOLD),
        &mut a_now,
        [1, 1, 1, 0],
    );
    assert!(has_objective(&highroad.replies, 10, 3));
    let a_final = talk(&mut a, &a_jacque, JACQUE, a_now);
    a_now += 2;
    assert!(has_objective(&a_final.replies, 10, 4));
    release_end_screen(&mut a, &a_jacque, QUEST, JACQUE, 0, a_now);
    a_now += 2;
    finish_quest(&mut a, &a_jacque, QUEST, ADVENTURER, 50, 25, a_now);
    a_now += 2;
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&a), 50);
    assert_eq!(profile_xp(&a, ADVENTURER), 25);
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Active);
    assert_eq!(counts(&b, QUEST), [1, 0, 0, 0]);
    assert_eq!(coins(&b), 0);
    assert_eq!(profile_xp(&b, ADVENTURER), 0);

    // B completes independently and is paid once too.
    visit(
        &mut b,
        OTHER_CHARACTER,
        npc_position(CARLOS),
        &mut b_now,
        [1, 1, 0, 0],
    );
    visit(
        &mut b,
        OTHER_CHARACTER,
        npc_position(MERIGOLD),
        &mut b_now,
        [1, 1, 1, 0],
    );
    let b_final = talk(&mut b, &b_jacque, JACQUE, b_now);
    b_now += 2;
    assert!(has_objective(&b_final.replies, 10, 4));
    release_end_screen(&mut b, &b_jacque, QUEST, JACQUE, 0, b_now);
    b_now += 2;
    finish_quest(&mut b, &b_jacque, QUEST, ADVENTURER, 50, 25, b_now);
    b_now += 2;
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&b), 50);
    assert_eq!(profile_xp(&b, ADVENTURER), 25);
    assert_eq!(coins(&a), 50);
    assert_eq!(profile_xp(&a, ADVENTURER), 25);

    // Each payout is once only; a replayed 49/14 changes neither player.
    refused(
        &mut a,
        &a_jacque,
        &quest_end_reply(QUEST),
        a_now,
        "A's turn-in replay",
    );
    refused(
        &mut b,
        &b_jacque,
        &quest_end_reply(QUEST),
        b_now,
        "B's turn-in replay",
    );
    assert_eq!(coins(&a), 50);
    assert_eq!(profile_xp(&a, ADVENTURER), 25);
    assert_eq!(coins(&b), 50);
    assert_eq!(profile_xp(&b, ADVENTURER), 25);
}

/// (R) Original quests keep priority. The hunt is not bound to Samantha's NPC
/// row, so a new character talking to her receives her original referral as the
/// first (and only) 49/1. This is the guard for the rejected design that put the
/// custom event ahead of an original quest at a shared giver.
#[test]
fn the_hunt_never_hides_an_original_quest_at_its_giver() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let samantha = character(CHARACTER, 0, npc_position(SAMANTHA));
    let now = 75_000;

    // `offer` asserts exactly one 49/1 and that it decodes to the referral.
    let offered = offer(&mut game, &samantha, SAMANTHA, REFERRAL, now);
    assert!(
        offered
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 1, Some(QUEST))),
        "the hunt must not hide Samantha's original referral"
    );
    assert_eq!(quest_status(&game, REFERRAL), QuestStatus::Offered);
    assert_eq!(pending_offer(&game), Some(REFERRAL));
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&QUEST),
        "talking to an original giver must not start the hunt"
    );
}

/// A wallet at the coin cap refuses the hunt's 50-coin grant for that player
/// only, atomically: no coins, XP or save bytes change and the reward stays
/// pending. The other player completes normally.
#[test]
fn an_over_cap_reward_is_refused_for_that_player_only() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = fresh(&fixture, CHARACTER);
    let mut b = fresh(&fixture, OTHER_CHARACTER);
    let a_jacque = character(CHARACTER, 0, npc_position(JACQUE));
    let b_jacque = character(OTHER_CHARACTER, 0, npc_position(JACQUE));
    let mut a_now = 80_000;
    let mut b_now = 90_000;
    open_hunt_end_screen(&mut a, &a_jacque, &mut a_now);
    open_hunt_end_screen(&mut b, &b_jacque, &mut b_now);

    let mut state = a.state();
    state.inventory.coins = i32::MAX as u32 - 49;
    a = fixture.game_as(CHARACTER, Some(state));
    let before_state = a.state();
    let before_bytes = saved(&a);
    // T382: a per-player denial, not a session-ending error.
    assert!(
        !a.handle_in_world(&quest_end_reply(QUEST), &a_jacque, a_now)
            .expect("an over-cap coin grant is a refusal, not an error")
            .changed,
        "an over-cap coin grant is refused for A"
    );
    assert_eq!(
        a.state(),
        before_state,
        "A's over-cap refusal changed state"
    );
    assert_eq!(
        saved(&a),
        before_bytes,
        "A's over-cap refusal changed save bytes"
    );
    assert_eq!(pending_turnin(&a), Some(QUEST), "A's reward stays pending");
    assert_eq!(coins(&a), i32::MAX as u32 - 49);
    assert_eq!(profile_xp(&a, ADVENTURER), 0);

    // B's normal turn-in still pays; A's refusal does not affect B.
    finish_quest(&mut b, &b_jacque, QUEST, ADVENTURER, 50, 25, b_now);
    assert_eq!(coins(&b), 50);
    assert_eq!(profile_xp(&b, ADVENTURER), 25);

    // A retry still refuses and changes nothing.
    let before_state = a.state();
    let before_bytes = saved(&a);
    assert!(
        !a.handle_in_world(&quest_end_reply(QUEST), &a_jacque, a_now + 1)
            .expect("the over-cap retry is a refusal, not an error")
            .changed,
        "the over-cap retry is refused for A"
    );
    assert_eq!(a.state(), before_state);
    assert_eq!(saved(&a), before_bytes);
    assert_eq!(coins(&a), i32::MAX as u32 - 49);
    assert_eq!(profile_xp(&a, ADVENTURER), 0);
}

/// (R) Pre-existing behavior pinned by F272 review finding 7: 49/14 is keyed only
/// by `pending_turnin`; it does not recheck the speaker or the player's range.
/// Enforcing a turn-in range is a separate, lead-approved change, so this test
/// records the current result rather than silently claiming a refusal.
#[test]
fn a_completed_final_talk_is_turned_in_away_from_the_giver_today() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let jacque = character(CHARACTER, 0, npc_position(JACQUE));
    let mut now = 70_000;
    open_hunt_end_screen(&mut game, &jacque, &mut now);

    // Walk 100 m away and send the turn-in from that position.
    let mut far = npc_position(JACQUE);
    far[0] += 100.0;
    let walked = game
        .on_location(&character(CHARACTER, 0, far), now)
        .expect("movement is accepted");
    assert!(!walked.changed, "walking away advances no objective");
    now += 2;
    finish_quest(
        &mut game,
        &character(CHARACTER, 0, far),
        QUEST,
        ADVENTURER,
        50,
        25,
        now,
    );
}

/// Content identity for the hunt (kept under its protected T281 test name).
/// The operator broadcast script lives in the task note, not in content, so
/// this test does not read note prose (F345 review finding 6). Host row
/// drawability and the malformed-record loader case are separate tests.
#[test]
fn the_hunt_identity_and_admin_script_are_recorded() {
    let content = content_dir();
    let quest_dir = content.join("quests");
    let mut files: Vec<PathBuf> = fs::read_dir(&quest_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();

    let mut seen = BTreeSet::new();
    let mut host_givers = Vec::new();
    for path in &files {
        let document = read_json(path);
        for record in document["records"].as_array().unwrap() {
            let id = record["id"].as_u64().unwrap() as u32;
            assert!(id > 0, "{}: quest ids are positive", path.display());
            assert!(seen.insert(id), "duplicate quest id {id}");
            if record["giver_npc_id"] == json!(JACQUE) {
                host_givers.push(id);
            }
        }
    }
    assert_eq!(
        host_givers,
        vec![QUEST],
        "the hunt is Jacque's only giver record"
    );

    let hunt_path = quest_dir.join("scavenger_hunt.json");
    let hunt = read_json(&hunt_path);
    let records = hunt["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record["id"], json!(QUEST));
    assert_eq!(record["title_string_id"], json!(TITLE_ID));
    assert_eq!(record["giver_npc_id"], json!(JACQUE));
    assert_eq!(record["prerequisite_quest_ids"], json!([]));
    assert_ne!(record["enabled"], json!(false));

    // The loader requires the quest to be listed by a jobs.json quest_ids
    // list; the giver offer itself comes from giver_npc_id. This is a loader
    // structural invariant, not a job gate: the record has no `job` field and is
    // not Adventurer content.
    let jobs = read_json(&content.join("jobs.json"));
    let job_refs: Vec<&str> = jobs["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|job| {
            job["quest_ids"]
                .as_array()
                .is_some_and(|ids| ids.contains(&json!(QUEST)))
        })
        .map(|job| job["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        job_refs,
        vec!["adventurer"],
        "the hunt has one loader-required job reference"
    );

    // No NPC row binds the hunt at all. The offer comes only from the record's
    // giver_npc_id through QuestEngine::quest_ids_for_giver, so no row's explicit
    // first-offer list can reorder a pre-existing quest.
    let mut binders = Vec::new();
    for path in fs::read_dir(content.join("npcs")).unwrap() {
        let path = path.unwrap().path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let document = read_json(&path);
        for npc in document["records"].as_array().unwrap() {
            let npc_id = npc["npc_id"].as_u64().unwrap();
            for field in ["quest_ids_given", "quests_given", "quests_accepted"] {
                let bound = npc[field].as_array().is_some_and(|rows| {
                    rows.iter()
                        .any(|row| row["quest_id"] == json!(QUEST) || row == &json!(QUEST))
                });
                if bound {
                    binders.push((npc_id, field));
                }
            }
        }
    }
    assert!(
        binders.is_empty(),
        "no NPC row may bind the hunt; found {binders:?}"
    );

    // The host row, Ricky's untouched original chain and Samantha's referral
    // are checked by `the_hunt_host_is_drawable_in_sanctuary`.

    // The three stops copy the cited rows verbatim and use the open-world zone.
    let objectives = record["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 4);
    for (index, (file, npc_id, id)) in [
        ("farnum_farmstead.json", PAPPY, 1u32),
        ("cobblestone_village.json", CARLOS, 2),
        ("highroad_junction.json", MERIGOLD, 3),
    ]
    .into_iter()
    .enumerate()
    {
        let objective = &objectives[index];
        assert_eq!(objective["kind"], json!("visit_position"));
        assert_eq!(objective["id"], json!(id));
        assert_eq!(objective["zone_id"], json!(0));
        assert_eq!(objective["radius"], json!(8.0));
        let npc = npc_row(file, npc_id);
        assert_eq!(
            objective["position"], npc["position"],
            "stop {id} must copy the {file} row position verbatim"
        );
        let position: [f32; 3] = serde_json::from_value(objective["position"].clone()).unwrap();
        assert!(position.iter().all(|value| value.is_finite()));
        let radius = objective["radius"].as_f64().unwrap();
        assert!(
            (1.0..=50.0).contains(&radius),
            "the local hunt radius stays inside its authored sanity bounds"
        );
    }
    assert_eq!(objectives[3]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[3]["npc_id"], json!(JACQUE));

    // Every record/objective/reward carries non-empty, correctly labelled
    // provenance; a reconstructed tag starts with (R).
    let check_sources = |value: &Value, label: &str| {
        let tags = value["source"]
            .as_array()
            .unwrap_or_else(|| panic!("{label} has provenance"));
        assert!(!tags.is_empty(), "{label} has non-empty provenance");
        for tag in tags {
            let evidence = tag["evidence"].as_str().unwrap_or("");
            assert!(!evidence.trim().is_empty(), "{label} evidence is non-empty");
            if tag["kind"] == json!("reconstructed") {
                assert!(
                    evidence.starts_with("(R)"),
                    "{label} reconstructed evidence starts with (R)"
                );
            }
        }
    };
    check_sources(record, "record");
    for (index, objective) in objectives.iter().enumerate() {
        check_sources(objective, &format!("objective {index}"));
    }
    for (index, reward) in record["rewards"].as_array().unwrap().iter().enumerate() {
        check_sources(reward, &format!("reward {index}"));
    }
    let rewards = record["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(50));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"1": 25}));

    // The operator broadcast script is recorded in `notes/Autopilot/T281.md`.
    // The suite deliberately does not read note prose: a note edit is not a
    // gameplay change and a checkout without `notes/` must still pass (F345
    // review finding 6).
}

/// The host NPC rows are unchanged: Jacque is a drawable `legacy_data` quest
/// giver with no explicit quest bindings, Ricky's original chain still names
/// him, and Samantha still gives the referral that targets Ricky.
#[test]
fn the_hunt_host_is_drawable_in_sanctuary() {
    let content = content_dir();
    let jacque = npc_row("sanctuary.json", JACQUE);
    assert_eq!(jacque["placement"], json!("legacy_data"));
    assert_eq!(jacque["model_id"], json!(134));
    assert_eq!(jacque["name_string_id"], json!(7_575));
    assert_eq!(jacque["greeting_string_id"], json!(15_500));
    assert_eq!(jacque["quest_ids_given"], json!([]));
    assert_eq!(jacque["quests_given"], json!([]));
    assert_eq!(jacque["quests_accepted"], json!([]));
    assert!(
        jacque["roles"]
            .as_array()
            .unwrap()
            .contains(&json!("quest_giver")),
        "Jacque is a quest giver"
    );
    let position: [f32; 3] = serde_json::from_value(jacque["position"].clone()).unwrap();
    assert_eq!(position, [-1212.878, -13.95695, 463.8414]);
    assert!(position.iter().all(|value| value.is_finite()));
    let spawned: BTreeMap<u64, [f32; 3]> = NpcVisibility::load(&content)
        .unwrap()
        .teleport_targets(0)
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect();
    assert_eq!(
        spawned.get(&JACQUE),
        Some(&position),
        "Jacque is spawned by the production visibility loader"
    );

    // Ricky Danger's row stays byte-identical: his original chain still names
    // him as giver and the hunt is not bound to him.
    let original_quests = read_json(&content.join("quests/adventurer.json"));
    let original_records = original_quests["records"].as_array().unwrap();
    for quest_id in [CALL_THE_CREW, BACK_FOR_MORE] {
        let original = original_records
            .iter()
            .find(|record| record["id"] == json!(quest_id))
            .expect("Ricky's original quest is in adventurer.json");
        assert_eq!(original["giver_npc_id"], json!(RICKY));
        assert_ne!(original["enabled"], json!(false));
    }
    let ricky = npc_row("cobblestone_village.json", RICKY);
    assert_eq!(ricky["placement"], json!("legacy_data"));
    assert_eq!(ricky["model_id"], json!(960));
    assert_eq!(ricky["name_string_id"], json!(94_007));
    assert_eq!(ricky["greeting_string_id"], json!(94_388));
    assert_eq!(ricky["quest_ids_given"], json!([]));
    assert_eq!(ricky["quests_given"], json!([]));
    assert_eq!(ricky["quests_accepted"], json!([]));
    assert!(
        ricky["roles"]
            .as_array()
            .unwrap()
            .contains(&json!("quest_giver")),
        "Ricky is a quest giver"
    );
    let ricky_position: [f32; 3] = serde_json::from_value(ricky["position"].clone()).unwrap();
    assert_eq!(ricky_position, [-1909.511, -44.3937, 462.83]);

    // Samantha's referral is untouched: she still gives 3000021, which targets
    // Ricky, and the hunt did not move onto her.
    let samantha = npc_row("cobblestone_village.json", SAMANTHA);
    assert!(
        samantha["quest_ids_given"]
            .as_array()
            .unwrap()
            .contains(&json!(REFERRAL)),
        "Samantha still gives the referral"
    );
    let referral = original_quests["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"] == json!(REFERRAL))
        .expect("Introduce Yourself is in adventurer.json");
    assert_eq!(referral["giver_npc_id"], json!(SAMANTHA));
    assert_ne!(referral["enabled"], json!(false));
    assert_eq!(referral["objectives"][0]["npc_id"], json!(RICKY));
}

/// A malformed hunt record is a bounded `InvalidData` load error, never a
/// panic; the unmodified copy in the same scratch directory is the control.
#[test]
fn a_malformed_hunt_record_is_a_load_error() {
    let scratch = Scratch::new("scavenger-hunt-malformed");
    copy_split_content(&scratch.0);
    assert!(
        QuestEngine::from_split_content(&scratch.0, 1, eligibility(), None).is_ok(),
        "the control copy must load"
    );
    let path = scratch.0.join("quests/scavenger_hunt.json");
    let mut corrupted = read_json(&path);
    corrupted["records"][0]["objectives"][0]["radius"] = json!(0.0);
    fs::write(&path, serde_json::to_vec(&corrupted).unwrap()).unwrap();
    let error = match QuestEngine::from_split_content(&scratch.0, 1, eligibility(), None) {
        Ok(_) => panic!("a zero hunt radius must be a load error"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}

/// (R) The split loader requires the quest catalogue to equal the union of every
/// jobs.json `quest_ids` list (`quest_engine.rs from_split_content`). That rule is
/// why the event quest has an Adventurer reference; it is not a job gate. This
/// test pins the rule so a future standalone-event loader change is deliberate.
#[test]
fn a_quest_that_no_job_lists_is_rejected_by_the_split_loader() {
    let scratch = Scratch::new("standalone-quest");
    copy_split_content(&scratch.0);
    let jobs_path = scratch.0.join("jobs.json");
    let mut jobs = read_json(&jobs_path);
    for job in jobs["records"].as_array_mut().unwrap() {
        if let Some(ids) = job["quest_ids"].as_array_mut() {
            ids.retain(|id| id != &json!(QUEST));
        }
    }
    fs::write(&jobs_path, serde_json::to_vec(&jobs).unwrap()).unwrap();
    let error = match QuestEngine::from_split_content(&scratch.0, 1, eligibility(), None) {
        Ok(_) => panic!("a quest that no job lists must be rejected"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(
        error.to_string().contains("jobs"),
        "the error names the jobs.json union rule: {error}"
    );
}

fn npc_row(file: &str, npc_id: u64) -> Value {
    let document = read_json(&content_dir().join("npcs").join(file));
    document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"] == json!(npc_id))
        .unwrap_or_else(|| panic!("{file} has NPC {npc_id}"))
        .clone()
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-t281-{name}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_split_content(target: &Path) {
    let content = content_dir();
    for folder in ["quests", "npcs"] {
        fs::create_dir_all(target.join(folder)).unwrap();
        for entry in fs::read_dir(content.join(folder)).unwrap() {
            let path = entry.unwrap().path();
            fs::copy(&path, target.join(folder).join(path.file_name().unwrap())).unwrap();
        }
    }
    fs::copy(content.join("jobs.json"), target.join("jobs.json")).unwrap();
}
