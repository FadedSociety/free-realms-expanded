//! T247/F272: the playable starting-road scavenger hunt (content-only).
//!
//! The tests drive the production in-world NPC router (`Gameplay::handle_in_world`
//! -> `gameplay/npc.rs` -> `QuestEngine`), the real `Gameplay::on_location`
//! movement path and the real accept/turn-in packets. They never call the quest
//! engine's private helpers as a shortcut. The shared fixture and packet helpers
//! live in `tests/support/npc_quest_chain.rs`; this file adds only the event
//! content checks.
//!
//! Client evidence (2014 client, checked with `lab/autopilot/ida_query.py`):
//! - 26/8 NPC interact constructor `008b6bc0`: u64 target + one bool.
//! - 26/10 menu select constructor `00a36f80`: two GUID dwords + i32 choice.
//! - 26/6 dialogue response `009f5890`: i32 row (301 is the quest row).
//! - 49/2 QuestReply constructor `00a86400`: quest id + accept byte.
//! - 49/14 QuestEndReply constructor `00a864c0`: one quest id.
//! - 49/1 QuestInfo reader `00c7bb60` / `00c7b7a0`: quest id, five screen dwords,
//!   bool, reward bundle, u64, f32-bits dword, two bools.
//! - 49/3 QuestAdd reader `0093c840` (fixed prefix and objective count) ->
//!   `0092b050` (count and per-objective loop) -> `008fd770` (the 107-byte
//!   record; its leading id dword is read in `0092b050`). `00c7bd10` is only an
//!   apply entry that calls `009225d0`, not the record layout.
//! - 49/13 QuestEnd reader `00c7b990`: u64 actor, three dwords, reward bundle,
//!   trailing dword.
//! `visit_position` is a server-side objective; the client only receives the
//! ordinary 49/9/49/10 objective updates and the 49/8 activation, so there is no
//! new wire shape to match.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    Fixture, accept, assert_no_offer, assert_packet_changes_nothing, character_at_for as character,
    coins, content_dir, counts, finish_quest, has_family, offer, pending_offer, pending_turnin,
    profile_xp, quest_end_reply, quest_reply, quest_status, read_json, release_end_screen, saved,
    send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    npc_interaction::NpcInteraction,
    profiles::ProfileState,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CHARACTER: u64 = 247_001;
const OTHER_CHARACTER: u64 = 247_002;
const QUEST: u32 = 3_000_200;
/// Samantha's existing Cobblestone referral; it is her first 49/1 offer, so the
/// tests accept it through the real menu before the hunt becomes the next offer.
const REFERRAL: u32 = 3_000_021;
const SAMANTHA: u64 = 9_000_000_189;
/// The generic stock "Scavenger Hunt " label. The Greenwood Forest minigame title
/// (429342) belongs to a different retail activity and is not reused.
const TITLE_ID: u32 = 3_374;
const HUNT_OBJECTIVE_BYTES: usize = 107;
const PAPPY: u64 = 9_000_000_165;
const CARLOS: u64 = 9_000_000_042;
const MERIGOLD: u64 = 9_000_000_402;
/// A positioned Cobblestone vendor with no quest binding; a nearby non-target NPC.
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

/// A fresh character with the Adventurer profile active, bound to the real
/// bundled split quest content exactly as production binds it.
fn fresh(fixture: &Fixture, character_id: u64) -> Gameplay {
    let mut state = fixture.game_as(character_id, None).state();
    state.profiles = Some(ProfileState {
        version: 1,
        active: 1,
        unlocked: BTreeSet::from([1]),
        experience: BTreeMap::new(),
    });
    fixture.game_as(character_id, Some(state))
}

fn npc_position(npc_id: u64) -> [f32; 3] {
    let content = content_dir();
    NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("NPC content loads")
        .location(npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} is a positioned content spawn"))
        .1
}

fn has_objective(replies: &[Vec<u8>], subtype: u16, objective_id: u32) -> bool {
    replies.iter().any(|packet| {
        packet.len() >= 14
            && packet[..6] == [49, 0, subtype as u8, 0, 0, 0]
            && packet[6..10] == QUEST.to_le_bytes()
            && packet[10..14] == objective_id.to_le_bytes()
    })
}

/// Accept Samantha's existing Cobblestone referral, then obtain the hunt's 49/1
/// offer. The referral is her first offer (`quest_ids_given` is consulted before
/// `QuestEngine::quest_ids_for_giver`), so accepting it is the production order
/// that makes the hunt the next offer, not a test-only shortcut.
fn offer_hunt(game: &mut Gameplay, samantha: &CharacterState, now: &mut u64) -> Outcome {
    offer(game, samantha, SAMANTHA, REFERRAL, *now);
    *now += 4;
    accept(game, samantha, REFERRAL, *now);
    *now += 4;
    let offered = offer(game, samantha, SAMANTHA, QUEST, *now);
    *now += 4;
    assert_eq!(pending_offer(game), Some(QUEST));
    offered
}

fn accept_hunt(game: &mut Gameplay, samantha: &CharacterState, now: &mut u64) {
    let accepted = accept(game, samantha, QUEST, *now);
    *now += 2;
    let add = accepted
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 3, Some(QUEST)))
        .expect("accept sends the 49/3 quest record");
    // 49/3 packet: six-byte header, 103-byte fixed prefix and reward bundle,
    // then the objective count at 109..113 and one 107-byte record each. Client
    // reader chain 0x0093c840 (fixed prefix and count) -> 0x0092b050 (count and
    // per-objective loop; it reads each record's leading id dword) -> 0x008fd770
    // (the rest of the 107-byte record).
    let objective_count = i32::from_le_bytes(add[109..113].try_into().unwrap());
    assert_eq!(objective_count, 4, "the hunt has four objectives");
    let ids: Vec<u32> = (0..usize::try_from(objective_count).unwrap())
        .map(|index| {
            let offset = 113 + index * HUNT_OBJECTIVE_BYTES;
            u32::from_le_bytes(add[offset..offset + 4].try_into().unwrap())
        })
        .collect();
    assert_eq!(ids, [1, 2, 3, 4], "the objectives are in hunt order");
    assert!(
        has_objective(&accepted.replies, 8, 1),
        "accept activates the first ordered objective"
    );
    assert_eq!(counts(game, QUEST), [0, 0, 0, 0]);
    assert_eq!(pending_offer(game), None);
}

fn visit(
    game: &mut Gameplay,
    guid: u64,
    position: [f32; 3],
    now: &mut u64,
    expected: [u32; 4],
) -> Outcome {
    *now += 1;
    // Runtime movement is the one open-world zone; npcs/*.json zone ids are
    // content region labels (src/npc_interaction.rs check_range).
    let outcome = game
        .on_location(&character(guid, 0, position), *now)
        .expect("movement is accepted");
    assert!(outcome.changed, "the landmark must advance the hunt");
    assert_eq!(counts(game, QUEST), expected, "landmark counters");
    outcome
}

fn assert_move_changes_nothing(
    game: &mut Gameplay,
    guid: u64,
    zone_id: u32,
    position: [f32; 3],
    now: u64,
    label: &str,
) {
    let before_state = game.state();
    let before_bytes = saved(game);
    let at = character(guid, zone_id, position);
    let outcome = game.on_location(&at, now).expect("movement is accepted");
    assert!(!outcome.changed, "{label} changed gameplay");
    assert_eq!(game.state(), before_state, "{label} changed the state");
    assert_eq!(saved(game), before_bytes, "{label} changed the save bytes");
}

#[test]
fn samantha_offers_her_referral_then_the_hunt() {
    let record = read_json(&content_dir().join("quests/scavenger_hunt.json"))["records"][0].clone();
    assert_eq!(record["objectives"].as_array().unwrap().len(), 4);
    assert_eq!(record["rewards"][0]["coins"], json!(50));
    assert_eq!(record["rewards"][0]["job_xp"], json!({"1": 25}));

    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let samantha = character(CHARACTER, 0, npc_position(SAMANTHA));
    let mut now = 1_000;

    let offer = offer_hunt(&mut game, &samantha, &mut now);
    let offers: Vec<_> = offer
        .replies
        .iter()
        .filter(|packet| has_family(packet, 49, 1, Some(QUEST)))
        .collect();
    assert_eq!(offers.len(), 1, "the hunt offer appears once");
    let offer = offers[0];
    // 49/1 layout (reader 0x00c7b7a0): quest, intro, wire dword 2, name,
    // description, wire dword 5. The builder puts the stock title in 2/name/5.
    assert_eq!(u32::from_le_bytes(offer[6..10].try_into().unwrap()), QUEST);
    assert_eq!(u32::from_le_bytes(offer[10..14].try_into().unwrap()), 0);
    assert_eq!(
        u32::from_le_bytes(offer[14..18].try_into().unwrap()),
        TITLE_ID
    );
    assert_eq!(
        u32::from_le_bytes(offer[18..22].try_into().unwrap()),
        TITLE_ID
    );
    assert_eq!(
        u32::from_le_bytes(offer[22..26].try_into().unwrap()),
        0,
        "description text is not authored; it stays 0 under the hollow-text policy"
    );
    assert_eq!(pending_offer(&game), Some(QUEST));
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Offered);
    assert_eq!(counts(&game, QUEST), [0, 0, 0, 0]);

    // A pending offer is not repeated and the save stays equal.
    assert_no_offer(
        &mut game,
        &samantha,
        SAMANTHA,
        &mut now,
        "a pending hunt offer",
    );

    accept_hunt(&mut game, &samantha, &mut now);
    let first = game.state();
    let first_bytes = saved(&game);

    // A second character independently resolves the same giver, referral and
    // hunt offer; the first character's state is untouched.
    let mut other = fresh(&fixture, OTHER_CHARACTER);
    let other_samantha = character(OTHER_CHARACTER, 0, npc_position(SAMANTHA));
    let mut other_now = 2_000;
    offer_hunt(&mut other, &other_samantha, &mut other_now);
    assert_eq!(pending_offer(&other), Some(QUEST));
    assert_eq!(quest_status(&other, QUEST), QuestStatus::Offered);
    assert_eq!(
        other.state().npc_quests.as_ref().unwrap().character_id,
        OTHER_CHARACTER
    );
    assert_eq!(game.state(), first);
    assert_eq!(saved(&game), first_bytes);
}

#[test]
fn a_declined_referral_is_reoffered_before_the_hunt() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let samantha = character(CHARACTER, 0, npc_position(SAMANTHA));
    let mut now = 11_000;

    offer(&mut game, &samantha, SAMANTHA, REFERRAL, now);
    now += 4;
    let declined = send(&mut game, &samantha, &quest_reply(REFERRAL, false), now);
    assert!(declined.changed, "declining clears the pending offer");
    assert_eq!(pending_offer(&game), None);
    now += 4;

    // (R) Pre-existing offer order: a declined quest is removed and is eligible
    // again, so the next Talk re-offers the referral and the hunt stays hidden.
    // The event runbook therefore tells players to accept and talk again.
    offer(&mut game, &samantha, SAMANTHA, REFERRAL, now);
    now += 4;
    accept(&mut game, &samantha, REFERRAL, now);
    now += 4;
    let hunt = offer(&mut game, &samantha, SAMANTHA, QUEST, now);
    assert!(
        hunt.replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(QUEST))),
        "the hunt follows the accepted referral"
    );
}

#[test]
fn the_three_landmarks_and_return_complete_the_hunt_once() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let samantha = character(CHARACTER, 0, npc_position(SAMANTHA));
    let mut now = 3_000;
    offer_hunt(&mut game, &samantha, &mut now);
    accept_hunt(&mut game, &samantha, &mut now);

    // A Talk at Samantha before the three stops neither completes the final
    // objective nor opens the end screen, and changes nothing.
    assert_no_offer(
        &mut game,
        &samantha,
        SAMANTHA,
        &mut now,
        "an early return Talk",
    );
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

    // Return to Samantha: the final Talk completes goal 4 and holds the end
    // screen until the client chooses the dialogue row.
    let final_talk = talk(&mut game, &samantha, SAMANTHA, now);
    now += 2;
    assert!(has_objective(&final_talk.replies, 10, 4));
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

    let screen = release_end_screen(&mut game, &samantha, QUEST, SAMANTHA, 0, now);
    now += 2;
    assert_eq!(screen.len(), 99);
    assert_eq!(i32::from_le_bytes(screen[27..31].try_into().unwrap()), 50);
    assert_eq!(i32::from_le_bytes(screen[31..35].try_into().unwrap()), 25);

    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, 1);
    let completed = finish_quest(&mut game, &samantha, QUEST, 1, 50, 25, now);
    now += 2;
    let count = completed
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 12, None))
        .expect("the completed count follows");
    assert_eq!(i32::from_le_bytes(count[6..10].try_into().unwrap()), 1);

    // A post-completion Talk offers nothing and pays nothing.
    let paid = game.state();
    let paid_bytes = saved(&game);
    assert_no_offer(
        &mut game,
        &samantha,
        SAMANTHA,
        &mut now,
        "a post-completion Talk",
    );
    assert_eq!(game.state(), paid);
    assert_eq!(saved(&game), paid_bytes);

    // A repeated 49/14 (or 49/2) after the payout changes nothing.
    let paid = game.state();
    let paid_bytes = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &samantha,
        &quest_end_reply(QUEST),
        now + 1,
        "repeated turn-in",
    );
    assert_packet_changes_nothing(
        &mut game,
        &samantha,
        &quest_reply(QUEST, true),
        now + 2,
        "repeated accept",
    );
    assert_eq!(game.state(), paid);
    assert_eq!(saved(&game), paid_bytes);
    assert_eq!(coins(&game), coins_before + 50);
    assert_eq!(profile_xp(&game, 1), xp_before + 25);
}

#[test]
fn a_wrong_position_or_npc_changes_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let samantha = character(CHARACTER, 0, npc_position(SAMANTHA));
    let mut now = 5_000;
    offer_hunt(&mut game, &samantha, &mut now);
    accept_hunt(&mut game, &samantha, &mut now);

    // Outside the 8 m search radius, same zone and landmark height.
    let mut outside = npc_position(PAPPY);
    outside[2] += 30.0;
    assert_move_changes_nothing(&mut game, CHARACTER, 0, outside, now + 1, "outside radius");

    // Inside the copied position but in a different zone.
    assert_move_changes_nothing(
        &mut game,
        CHARACTER,
        1,
        npc_position(PAPPY),
        now + 2,
        "wrong zone",
    );

    // A nearby positioned NPC that is not a hunt target and gives no quest: the
    // whole real conversation leaves the saved state equal.
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

    // Forged accept without a new offer, and a forged 49/14 without a pending
    // end screen, are both refused without a state change.
    assert_packet_changes_nothing(
        &mut game,
        &samantha,
        &quest_reply(QUEST, true),
        now + 1,
        "forged 49/2",
    );
    assert_packet_changes_nothing(
        &mut game,
        &samantha,
        &quest_end_reply(QUEST),
        now + 2,
        "forged 49/14",
    );

    // Out-of-range interaction is refused and mutates nothing.
    let mut far = npc_position(SAMANTHA);
    far[2] += 100.0;
    let before_state = game.state();
    let before_bytes = saved(&game);
    assert!(
        game.handle_in_world(
            &npc_quest_chain::npc_interact(SAMANTHA),
            &character(CHARACTER, 0, far),
            now + 3
        )
        .is_err(),
        "an out-of-range 26/8 must be refused"
    );
    assert_eq!(game.state(), before_state);
    assert_eq!(saved(&game), before_bytes);

    // The hunt is still Active with no progress and no payout.
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0, 0, 0, 0]);
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, 1), 0);
}

/// (R) Pre-existing behavior pinned by F272 review finding 7: 49/14 is keyed only
/// by `pending_turnin`; it does not recheck the speaker or the player's range.
/// Enforcing a turn-in range is a separate, lead-approved change, so this test
/// records the current result rather than silently claiming a refusal.
#[test]
fn a_completed_final_talk_is_turned_in_away_from_the_giver_today() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let samantha = character(CHARACTER, 0, npc_position(SAMANTHA));
    let mut now = 13_000;
    offer_hunt(&mut game, &samantha, &mut now);
    accept_hunt(&mut game, &samantha, &mut now);
    for position in [
        npc_position(PAPPY),
        npc_position(CARLOS),
        npc_position(MERIGOLD),
    ] {
        now += 1;
        game.on_location(&character(CHARACTER, 0, position), now)
            .expect("movement is accepted");
    }
    let final_talk = talk(&mut game, &samantha, SAMANTHA, now);
    now += 2;
    assert!(has_objective(&final_talk.replies, 10, 4));
    release_end_screen(&mut game, &samantha, QUEST, SAMANTHA, 0, now);
    now += 2;
    assert_eq!(pending_turnin(&game), Some(QUEST));

    // Walk 100 m away and send the turn-in from that position.
    let mut far = npc_position(SAMANTHA);
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
        1,
        50,
        25,
        now,
    );
}

#[test]
fn a_mid_hunt_reconnect_keeps_progress_and_pays_no_extra() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = fresh(&fixture, CHARACTER);
    let samantha = character(CHARACTER, 0, npc_position(SAMANTHA));
    let mut now = 7_000;
    offer_hunt(&mut game, &samantha, &mut now);
    accept_hunt(&mut game, &samantha, &mut now);

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
    let mid = saved(&game);
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, 1), 0);

    // Disconnect after two landmarks: the counters are in the save and are
    // restored without a payout.
    game = fixture.reload(&game);
    assert_eq!(saved(&game), mid, "reload changed the mid-hunt save");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [1, 1, 0, 0]);
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, 1), 0);

    visit(
        &mut game,
        CHARACTER,
        npc_position(MERIGOLD),
        &mut now,
        [1, 1, 1, 0],
    );
    let final_talk = talk(&mut game, &samantha, SAMANTHA, now);
    now += 2;
    assert!(has_objective(&final_talk.replies, 10, 4));
    release_end_screen(&mut game, &samantha, QUEST, SAMANTHA, 0, now);
    now += 2;
    finish_quest(&mut game, &samantha, QUEST, 1, 50, 25, now);
    now += 2;
    let paid = saved(&game);

    // Reconnect after the turn-in does not pay again and a replayed 49/14
    // changes nothing.
    game = fixture.reload(&game);
    assert_eq!(saved(&game), paid, "reload changed the paid save");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&game), 50);
    assert_eq!(profile_xp(&game, 1), 25);
    assert_packet_changes_nothing(
        &mut game,
        &samantha,
        &quest_end_reply(QUEST),
        now + 1,
        "turn-in replay after reconnect",
    );
    assert_eq!(coins(&game), 50);
    assert_eq!(profile_xp(&game, 1), 25);
}

#[test]
fn the_quest_loads_with_real_targets_and_no_collision() {
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
    assert_eq!(files.len(), 18, "the hunt is one new quest file");

    let mut seen = BTreeSet::new();
    let mut total_records = 0usize;
    for path in &files {
        let document = read_json(path);
        for record in document["records"].as_array().unwrap() {
            let id = record["id"].as_u64().unwrap() as u32;
            assert!(id > 0, "{}: quest ids are positive", path.display());
            assert!(seen.insert(id), "duplicate quest id {id}");
            total_records += 1;
        }
    }
    assert_eq!(total_records, 654, "653 existing records plus the hunt");

    let hunt_path = quest_dir.join("scavenger_hunt.json");
    let hunt = read_json(&hunt_path);
    let records = hunt["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record["id"], json!(QUEST));
    assert_eq!(record["title_string_id"], json!(TITLE_ID));
    assert_eq!(record["giver_npc_id"], json!(SAMANTHA));
    assert_eq!(record["prerequisite_quest_ids"], json!([]));
    assert_ne!(record["enabled"], json!(false));

    // The loader requires the quest to be listed by a jobs.json quest_ids
    // list; the giver offer itself comes from the record's giver_npc_id. This is
    // a loader structural invariant, not a job gate: the record has no `job`
    // field and is not Adventurer content.
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

    // No NPC content row was edited to bind the quest; the giver is the quest
    // record's field and the menu adds QuestEngine::quest_ids_for_giver.
    for path in fs::read_dir(content.join("npcs")).unwrap() {
        let path = path.unwrap().path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let document = read_json(&path);
        for npc in document["records"].as_array().unwrap() {
            for field in ["quest_ids_given", "quests_given", "quests_accepted"] {
                let bound = npc[field].as_array().is_some_and(|rows| {
                    rows.iter()
                        .any(|row| row["quest_id"] == json!(QUEST) || row == &json!(QUEST))
                });
                assert!(!bound, "{}: NPC {field} binds the hunt", path.display());
            }
        }
    }

    // The giver is spawned and drawable in the bundled NPC content.
    let samantha = npc_row("cobblestone_village.json", SAMANTHA);
    assert!(
        samantha["roles"]
            .as_array()
            .unwrap()
            .contains(&json!("quest_giver")),
        "Samantha is a quest giver"
    );
    assert!(samantha["model_id"].as_u64().unwrap() > 0);
    let position: [f32; 3] = serde_json::from_value(samantha["position"].clone()).unwrap();
    assert!(position.iter().all(|value| value.is_finite()));
    assert!(
        NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .unwrap()
            .location(SAMANTHA)
            .is_some(),
        "Samantha has a positioned content spawn"
    );

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
    assert_eq!(objectives[3]["npc_id"], json!(SAMANTHA));

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

    // A malformed synthetic copy is a bounded load error, never a panic; the
    // unmodified copy in the same scratch directory loads as a control.
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
            "osfr-t247-{name}-{}-{}",
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
