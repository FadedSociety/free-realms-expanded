//! T242/F268: Fizzlesticks' first Wizard chapter, "Wizard: How to be a Wizard" (3160003)
//! and "Wizard: I Can't Read This!" (3160004). The chapter is content-only; these tests
//! drive the production request path (`Gameplay::handle_in_world` -> `gameplay/npc.rs` ->
//! `QuestEngine`) plus the production position event (`Gameplay::on_location`, the exact
//! call `gateway.rs` makes after movement validation).
//!
//! Request/reply layouts used here:
//! - 26/8 NPC interact: constructor `008b6bc0`; body is u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; body is u64 target + i32 choice.
//! - 26/6 dialogue response: reader `009f5890`; body is i32 response row (301 is the quest row).
//! - 26/13 job select: i32 profile id.
//! - 49/1 QuestInfo: reader `00c7bb60` -> `00c7b7a0` (six dwords: quest id, intro, card title,
//!   name, description, an unused field), decoded with the production
//!   `decode_quest_offer_49_1` rather than by hand.
//! - 49/2 quest reply: constructor `00a86400` / sender `00a97d70`; i32 quest + u8 accept.
//! - 49/3 QuestAdd: reader `00c7bd10`.
//! - 49/13 QuestEnd: writer/reader `00c7bbd0`/`00c7b990`; actor u64 at 6..14, quest i32 at
//!   14..18, end-text locale at 18..22, animation at 22..26, reward bundle from 26.
//! - 49/14 end reply: constructor `00a864c0`; i32 quest.
//! 35/2 AddNpc (constructor `0091cac0`) draws Fizzlesticks, Nurble and Squee from their
//! `legacy_data` regional rows; `enabled_wizard_chapter_records_are_complete_and_drawable`
//! pins that.
//!
//! The shared fixture and packet helpers live in `tests/support/npc_quest_chain.rs`; the
//! Wizard vocabulary and reward assertions live in `tests/support/wizard_entry.rs`.
//!
//! The chapter fixture is the same opening play package the other chapter tests use
//! (`lab/references/opening-anvil-placement-v1/content`); the repository's own
//! `data/content` supplies the quest, NPC and trainer catalogues, exactly as the bundled play
//! package does. The Wizard save is seeded the way the NINJA chain seeds a Ninja save: the
//! fixture's `profiles.json` has no Wizard starter outfit, so the 26/13 switch to Wizard is
//! not used here; the chapter asks only that Wizard is the active job.

#[path = "support/wizard_entry.rs"]
mod wizard_entry;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    quest_engine::QuestStatus,
};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use wizard_entry::*;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn pending_offer(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_offer
}

fn has_quest(game: &Gameplay, quest_id: u32) -> bool {
    game.state()
        .npc_quests
        .as_ref()
        .unwrap()
        .quests
        .contains_key(&quest_id)
}

/// `gateway.rs` sends the validated pose to `Gameplay::on_location`; that is the production
/// event a `visit_position` objective consumes.
fn visit(game: &mut Gameplay, position: [f32; 3], now: u64) -> Outcome {
    game.on_location(&character_at(OPEN_WORLD, position), now)
        .expect("location event")
}

fn assert_visit_changes_nothing(game: &mut Gameplay, position: [f32; 3], now: u64, label: &str) {
    let before = saved(game);
    let outcome = visit(game, position, now);
    assert!(!outcome.changed, "{label} changed gameplay");
    assert_eq!(saved(game), before, "{label} changed the save bytes");
}

#[test]
fn fizzlesticks_offers_the_first_wizard_chapter_to_an_eligible_wizard() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let mut now = 1_000;

    // An Adventurer sees no Wizard chapter; the open menu survives the refusal.
    let mut game = fixture.game(None);
    assert_no_offer(
        &mut game,
        &character,
        FIZZLESTICKS,
        &mut now,
        "Adventurer at Fizzlesticks",
    );

    // A character on another job sees no Wizard chapter, and the offer gate does not
    // depend on the menu being freshly opened.
    now += 10;
    let switched = send(&mut game, &character, &select_job(BRAWLER), now);
    assert!(switched.changed);
    assert_eq!(game.state().profiles.as_ref().unwrap().active, BRAWLER);
    assert_no_offer(
        &mut game,
        &character,
        FIZZLESTICKS,
        &mut now,
        "Brawler at Fizzlesticks",
    );

    // A Wizard the player has unlocked but is not playing is refused by the same job gate.
    let mut inactive = fixture.game(Some(unlocked_wizard_state(&fixture)));
    assert_no_offer(
        &mut inactive,
        &character,
        FIZZLESTICKS,
        &mut now,
        "inactive Wizard at Fizzlesticks",
    );

    // The active Wizard sees the first chapter, and only the first chapter.
    let mut game = wizard_game(&fixture);
    let offer = offered_card(&mut game, &character, FIZZLESTICKS, BOOK, now);
    assert_eq!(offer.intro_text_string_id, BOOK_INTRO);
    assert_eq!(offer.name_string_id, BOOK_TITLE);
    assert_eq!(offer.description_string_id, BOOK_DESCRIPTION);
    assert!(!offer.members_only);
    // The 49/1 card's reward bundle is the pre-existing empty 69-byte placeholder; the
    // exact coins and job XP are asserted on the 49/13 turn-in screen instead (F268 gap).
    assert_eq!(pending_offer(&game), Some(BOOK));
    assert_eq!(counts(&game, BOOK), vec![0, 0]);
    assert!(!has_quest(&game, GLASSES));

    // A second Talk while the offer is open starts nothing new and changes nothing.
    assert_no_offer(
        &mut game,
        &character,
        FIZZLESTICKS,
        &mut now,
        "open offer at Fizzlesticks",
    );
    assert_eq!(pending_offer(&game), Some(BOOK));

    // Accepting pays nothing yet.
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);
    now += 10;
    accept(&mut game, &character, BOOK, now);
    assert_eq!(pending_offer(&game), None);
    assert_eq!(counts(&game, BOOK), vec![0, 0]);
    assert_eq!(coins(&game), coins_before);
    assert_eq!(profile_xp(&game, WIZARD), xp_before);
    // The second quest stays gated behind the first.
    assert!(!has_quest(&game, GLASSES));
    assert_no_offer(
        &mut game,
        &character,
        FIZZLESTICKS,
        &mut now,
        "first chapter accepted",
    );
}

#[test]
fn the_spell_book_and_glasses_steps_advance_and_turn_in_once() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let book_search = content_position("wilds.json", SQUEE);
    let glasses_search = content_position("wilds.json", NURBLE);
    let mut now = 2_000;
    let mut game = wizard_game(&fixture);

    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, BOOK, now);
    now += 10;
    accept(&mut game, &character, BOOK, now);
    assert_eq!(counts(&game, BOOK), vec![0, 0]);

    // The other chapter's search area is out of range for this step.
    assert_visit_changes_nothing(&mut game, glasses_search, now + 1, "out-of-range visit");
    assert_eq!(counts(&game, BOOK), vec![0, 0]);

    // The camp search area completes the first ordered step only.
    now += 10;
    let searched = visit(&mut game, book_search, now);
    assert!(searched.changed);
    assert_eq!(counts(&game, BOOK), vec![1, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(
        !searched
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the search is not the turn-in"
    );
    // A duplicate visit changes nothing.
    assert_visit_changes_nothing(&mut game, book_search, now + 1, "duplicate visit");

    // A talk to the wrong Robgoblin Camp NPC changes nothing.
    now += 10;
    let wrong_before = saved(&game);
    let wrong = talk(&mut game, &avatar_at(NURBLE), NURBLE, now);
    assert!(!wrong.changed, "a talk to the wrong NPC changed gameplay");
    assert_eq!(
        saved(&game),
        wrong_before,
        "a talk to the wrong NPC changed the save bytes"
    );
    assert_eq!(counts(&game, BOOK), vec![1, 0]);

    // Returning to Fizzlesticks completes the ordered step and holds the end screen.
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);
    now += 10;
    let returned = talk(&mut game, &character, FIZZLESTICKS, now);
    assert!(returned.changed);
    assert_eq!(counts(&game, BOOK), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(BOOK));
    assert!(
        !returned
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");
    assert_eq!(profile_xp(&game, WIZARD), xp_before);

    now += 10;
    release_wizard_end_screen(&mut game, &character, BOOK, BOOK_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        BOOK,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), xp_before + REWARD_XP);

    // No duplicate payment on a replayed 49/14.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(BOOK),
        now,
        "duplicate turn-in",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);

    // The second chapter opens only now: the next quest choice offers "I Can't Read This!"
    // and never re-offers the completed first record.
    now += 10;
    let offer = offered_card(&mut game, &character, FIZZLESTICKS, GLASSES, now);
    assert_eq!(offer.intro_text_string_id, GLASSES_INTRO);
    assert_eq!(offer.name_string_id, GLASSES_TITLE);
    assert_eq!(offer.description_string_id, GLASSES_DESCRIPTION);
    now += 10;
    accept(&mut game, &character, GLASSES, now);
    assert_eq!(counts(&game, GLASSES), vec![0, 0]);

    // The spell-book search area is out of range for the junkpile step.
    assert_visit_changes_nothing(&mut game, book_search, now + 1, "wrong-area visit");
    assert_eq!(counts(&game, GLASSES), vec![0, 0]);

    now += 10;
    let searched = visit(&mut game, glasses_search, now);
    assert!(searched.changed);
    assert_eq!(counts(&game, GLASSES), vec![1, 0]);
    assert_visit_changes_nothing(
        &mut game,
        glasses_search,
        now + 1,
        "duplicate junkpile visit",
    );

    now += 10;
    let returned = talk(&mut game, &character, FIZZLESTICKS, now);
    assert!(returned.changed);
    assert_eq!(counts(&game, GLASSES), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(GLASSES));

    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);
    now += 10;
    release_wizard_end_screen(&mut game, &character, GLASSES, GLASSES_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        GLASSES,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), xp_before + REWARD_XP);
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(GLASSES),
        now,
        "duplicate glasses turn-in",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);

    // T303 enables the next record: after both chapters, Fizzlesticks offers
    // "Sticky Spell Book" and never re-offers the completed records.
    assert!(!has_quest(&game, STICKY));
    now += 10;
    let offer = offered_card(&mut game, &character, FIZZLESTICKS, STICKY, now);
    assert_eq!(offer.intro_text_string_id, STICKY_INTRO);
    assert_eq!(offer.name_string_id, STICKY_TITLE);
    assert_eq!(offer.description_string_id, STICKY_DESCRIPTION);
}

#[test]
fn forged_or_missing_objective_events_change_nothing() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let book_search = content_position("wilds.json", SQUEE);
    let glasses_search = content_position("wilds.json", NURBLE);
    let far_away = [book_search[0] + 60.0, book_search[1], book_search[2]];
    let mut now = 3_000;
    let mut game = wizard_game(&fixture);
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);

    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, BOOK, now);
    // A forged accept of the still-gated second quest must not consume the open offer.
    let before = saved(&game);
    let forged = send(&mut game, &character, &quest_reply(GLASSES, true), now + 1);
    assert!(!forged.changed);
    assert_eq!(
        saved(&game),
        before,
        "the forged accept changed the save bytes"
    );
    assert_eq!(pending_offer(&game), Some(BOOK));
    assert!(!has_quest(&game, GLASSES));

    now += 10;
    accept(&mut game, &character, BOOK, now);

    // A replayed 49/2 changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(BOOK, true),
        now + 1,
        "replayed accept",
    );
    // A premature 49/14 for the active quest pays nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(BOOK),
        now + 2,
        "premature turn-in",
    );
    // A forged 49/14 for the gated second quest pays nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(GLASSES),
        now + 3,
        "forged second-quest turn-in",
    );
    // A visit outside every search area changes nothing.
    assert_visit_changes_nothing(&mut game, far_away, now + 4, "missing-objective visit");
    // The other chapter's search area is out of range for this step.
    assert_visit_changes_nothing(&mut game, glasses_search, now + 5, "out-of-range visit");
    // A talk to a wrong NPC changes nothing.
    let wrong_before = saved(&game);
    let wrong = talk(&mut game, &avatar_at(NURBLE), NURBLE, now + 6);
    assert!(!wrong.changed);
    assert_eq!(
        saved(&game),
        wrong_before,
        "the wrong-NPC talk changed the save bytes"
    );
    assert_eq!(counts(&game, BOOK), vec![0, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), coins_before);
    assert_eq!(profile_xp(&game, WIZARD), xp_before);

    // Complete the chapter properly, then replay every old packet.
    now += 10;
    assert!(visit(&mut game, book_search, now).changed);
    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    now += 10;
    release_wizard_end_screen(&mut game, &character, BOOK, BOOK_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        BOOK,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    let completed_coins = coins(&game);
    let completed_xp = profile_xp(&game, WIZARD);
    for (index, packet) in [
        quest_end_reply(BOOK),
        quest_reply(BOOK, true),
        quest_end_reply(GLASSES),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = u64::try_from(index).unwrap();
        assert_packet_changes_nothing(
            &mut game,
            &character,
            &packet,
            now + 10 + offset,
            "replayed chapter packet after completion",
        );
    }
    assert_visit_changes_nothing(&mut game, book_search, now + 20, "post-completion visit");
    assert_eq!(coins(&game), completed_coins);
    assert_eq!(profile_xp(&game, WIZARD), completed_xp);
    assert_eq!(quest_status(&game, BOOK), QuestStatus::Completed);
    assert!(!has_quest(&game, GLASSES));
}

#[test]
fn a_mid_chapter_reconnect_resumes_the_saved_progress() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let book_search = content_position("wilds.json", SQUEE);
    let glasses_search = content_position("wilds.json", NURBLE);
    let mut now = 4_000;
    let mut game = wizard_game(&fixture);
    let starting_coins = coins(&game);
    let starting_xp = profile_xp(&game, WIZARD);

    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, BOOK, now);
    now += 10;
    accept(&mut game, &character, BOOK, now);
    now += 10;
    assert!(visit(&mut game, book_search, now).changed);
    assert_eq!(counts(&game, BOOK), vec![1, 0]);

    // Disconnect after the first objective.
    let mid_bytes = saved(&game);
    game = fixture.reload_serialized(&game);
    assert_eq!(
        saved(&game),
        mid_bytes,
        "reload changed the active quest state"
    );
    assert_eq!(quest_status(&game, BOOK), QuestStatus::Active);
    assert_eq!(counts(&game, BOOK), vec![1, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(!has_quest(&game, GLASSES));

    // The second quest is still gated until the first completes: a forged accept changes
    // nothing, and no second quest exists in the save.
    let before = saved(&game);
    let forged = send(&mut game, &character, &quest_reply(GLASSES, true), now + 1);
    assert!(!forged.changed);
    assert_eq!(saved(&game), before, "the gated accept changed the save");
    assert!(!has_quest(&game, GLASSES));

    // Resume the chapter: the talk step and the held end screen survive too.
    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    assert_eq!(counts(&game, BOOK), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(BOOK));
    let held_bytes = saved(&game);
    game = fixture.reload_serialized(&game);
    assert_eq!(saved(&game), held_bytes, "reload changed the held turn-in");
    assert_eq!(pending_turnin(&game), Some(BOOK));
    assert_eq!(coins(&game), starting_coins, "a disconnect pays nothing");
    assert_eq!(profile_xp(&game, WIZARD), starting_xp);

    // The relog visibility batch re-sends the saved end screen, and only after the speaker's
    // 35/2 AddNpc: `ShowQuestEndScreen` (`00a95420`) resolves the actor through the client
    // actor table and never retries.
    let packets = game
        .update_npc_visibility(&character)
        .expect("relog NPC visibility");
    let spawn = packets.iter().position(|packet| {
        osfr_tools::startup_world::parse_add_npc(packet)
            .is_ok_and(|npc| npc.actor_id == FIZZLESTICKS)
    });
    let end = packets
        .iter()
        .position(|packet| has_family(packet, 49, 13, Some(BOOK)))
        .expect("the held end screen is re-sent after relog");
    assert!(
        spawn.is_some_and(|spawn| spawn < end),
        "the speaker's 35/2 must precede its 49/13"
    );
    assert_eq!(
        u64::from_le_bytes(packets[end][6..14].try_into().unwrap()),
        FIZZLESTICKS,
        "the re-sent end screen must name the runtime speaker"
    );
    assert_eq!(
        u32::from_le_bytes(packets[end][18..22].try_into().unwrap()),
        BOOK_END_TEXT,
        "the re-sent end screen must carry the spell-book debrief text"
    );

    // The client answers the re-sent end screen directly after the reconnect.
    now += 10;
    finish_quest(
        &mut game,
        &character,
        BOOK,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&game), starting_coins + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), starting_xp + REWARD_XP);

    // The chain order is restored after the reconnect: the second quest opens now.
    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, GLASSES, now);
    now += 10;
    accept(&mut game, &character, GLASSES, now);
    now += 10;
    assert!(visit(&mut game, glasses_search, now).changed);
    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    now += 10;
    release_wizard_end_screen(&mut game, &character, GLASSES, GLASSES_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        GLASSES,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );

    let final_bytes = saved(&game);
    let final_coins = coins(&game);
    let final_xp = profile_xp(&game, WIZARD);
    game = fixture.reload_serialized(&game);
    assert_eq!(
        saved(&game),
        final_bytes,
        "reload changed completed history"
    );
    assert_eq!(final_coins, starting_coins + 2 * REWARD_COINS);
    assert_eq!(final_xp, starting_xp + 2 * REWARD_XP);
    for quest_id in [BOOK, GLASSES] {
        assert_eq!(quest_status(&game, quest_id), QuestStatus::Completed);
    }
    assert_eq!(pending_turnin(&game), None);
}

#[test]
fn a_malformed_wizard_record_fails_at_load() {
    let fixture = Fixture::new();

    // The control: a well-formed synthetic record binds through the production path.
    let control = record(3_999_001, vec![talk_goal(1, FIZZLESTICKS)], vec![]);
    let control_dir = synthetic_content(&control);
    let mut control_game = fixture.game(None);
    control_game
        .bind_npc_quest_catalogue(&control_dir, CHARACTER)
        .expect("the control record loads");
    let _ = fs::remove_dir_all(&control_dir);

    // A zero talk target.
    assert_load_refused(
        &fixture,
        record(3_999_001, vec![talk_goal(1, 0)], vec![]),
        "zero objective target",
        "invalid quest objective definition",
    );
    // An objective without provenance.
    let mut no_source = visit_goal(1, [0.0, 0.0, 0.0], 5.0);
    no_source["source"] = serde_json::json!([]);
    assert_load_refused(
        &fixture,
        record(3_999_001, vec![no_source], vec![]),
        "empty objective source",
        "runtime quest objective requires provenance",
    );
    // Overlapping objective ids inside one record.
    assert_load_refused(
        &fixture,
        record(
            3_999_001,
            vec![talk_goal(1, FIZZLESTICKS), talk_goal(1, NURBLE)],
            vec![],
        ),
        "overlapping objective ids",
        "invalid quest objective definition",
    );
    // A reward outside the wire bounds.
    let mut out_of_range = reward(5);
    out_of_range["coins"] = serde_json::json!(u32::MAX);
    assert_load_refused(
        &fixture,
        record(
            3_999_001,
            vec![talk_goal(1, FIZZLESTICKS)],
            vec![out_of_range],
        ),
        "out-of-range reward",
        "invalid quest definition",
    );
    // A job-XP row that names no known job profile.
    let mut unknown_profile = reward(5);
    unknown_profile["job_xp"] = serde_json::json!({"999": 25});
    assert_load_refused(
        &fixture,
        record(
            3_999_001,
            vec![talk_goal(1, FIZZLESTICKS)],
            vec![unknown_profile],
        ),
        "unknown job profile",
        "quest XP references unknown job profile",
    );
}

/// A corrupt reconnect must not consume the saved NPC quest progress: the bind is
/// transactional, and a retry with good content resumes the chapter.
#[test]
fn a_refused_rebind_keeps_the_bound_catalogue_and_saved_progress() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let book_search = content_position("wilds.json", SQUEE);
    let mut now = 6_000;
    let mut game = wizard_game(&fixture);
    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, BOOK, now);
    now += 10;
    accept(&mut game, &character, BOOK, now);
    now += 10;
    assert!(visit(&mut game, book_search, now).changed);
    let in_progress = saved(&game);

    // The corrupt record is read-only and reused for both refused binds.
    let directory = synthetic_content(&record(3_999_001, vec![talk_goal(1, 0)], vec![]));

    // The live catalogue is not replaced by a refused rebind, so its next step still works.
    let refused = game.bind_npc_quest_catalogue(&directory, CHARACTER);
    assert!(
        refused.is_err(),
        "a zero-target record must be refused at content load"
    );
    assert_eq!(
        saved(&game),
        in_progress,
        "the refused rebind changed the game"
    );
    now += 10;
    assert!(
        talk(&mut game, &character, FIZZLESTICKS, now).changed,
        "the still-bound catalogue must serve the next chapter step"
    );
    assert_eq!(pending_turnin(&game), Some(BOOK));

    // A successful rebind on a live game rebuilds from the bound engine (the saved
    // copy is consumed at load), so it keeps the in-flight chapter.
    let advanced = saved(&game);
    game.bind_npc_quest_catalogue(&content_dir(), CHARACTER)
        .expect("the live rebind binds the good catalogue");
    assert_eq!(
        saved(&game),
        advanced,
        "a live rebind discarded the bound chapter progress"
    );
    assert_eq!(pending_turnin(&game), Some(BOOK));

    // A reconnect loads the save before binding: a refused bind must keep the saved
    // progress for the retry instead of consuming it.
    let state: GameplayState = serde_json::from_slice(&in_progress).unwrap();
    let mut reconnecting = Gameplay::load(fixture.path(), Some(state)).expect("save loads");
    let refused = reconnecting.bind_npc_quest_catalogue(&directory, CHARACTER);
    assert!(
        refused.is_err(),
        "the corrupt reconnect content must be refused"
    );
    assert_eq!(
        saved(&reconnecting),
        in_progress,
        "the refused bind consumed the saved NPC quest progress"
    );
    let _ = fs::remove_dir_all(&directory);
    reconnecting
        .bind_npc_quest_catalogue(&content_dir(), CHARACTER)
        .expect("the retry binds the good catalogue");
    assert_eq!(
        saved(&reconnecting),
        in_progress,
        "the retry did not restore the saved progress"
    );
    assert_eq!(counts(&reconnecting, BOOK), vec![1, 0]);
    assert_eq!(quest_status(&reconnecting, BOOK), QuestStatus::Active);
    now += 10;
    assert!(
        talk(&mut reconnecting, &character, FIZZLESTICKS, now).changed,
        "the retried catalogue must serve the next chapter step"
    );
    assert_eq!(pending_turnin(&reconnecting), Some(BOOK));
}

fn talk_goal(id: u32, npc_id: u64) -> Value {
    serde_json::json!({
        "kind": "talk_to_npc",
        "id": id,
        "npc_id": npc_id,
        "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic input"}]
    })
}

fn visit_goal(id: u32, position: [f32; 3], radius: f32) -> Value {
    serde_json::json!({
        "kind": "visit_position",
        "id": id,
        "zone_id": 0,
        "position": position,
        "radius": radius,
        "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic input"}]
    })
}

fn reward(coins: u32) -> Value {
    serde_json::json!({
        "coins": coins,
        "items": [],
        "job_xp": {"12": 25},
        "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic input"}]
    })
}

fn record(id: u32, objectives: Vec<Value>, rewards: Vec<Value>) -> Value {
    serde_json::json!({
        "id": id,
        "job": WIZARD,
        "title_string_id": BOOK_TITLE,
        "giver_npc_id": FIZZLESTICKS,
        "prerequisite_quest_ids": [],
        "objectives": objectives,
        "rewards": rewards,
        "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic record"}]
    })
}

/// Write one synthetic split-content package and return its directory.
fn synthetic_content(quest: &Value) -> PathBuf {
    let id = quest["id"].as_u64().unwrap();
    let directory = std::env::temp_dir().join(format!(
        "osfr-wizard-malformed-{}-{}-{id}",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(directory.join("quests")).unwrap();
    fs::create_dir_all(directory.join("npcs")).unwrap();
    fs::write(
        directory.join("quests/wizard.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "records": [quest]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("npcs/wilds.json"),
        serde_json::to_vec(&serde_json::json!({"schema_version": 1, "records": []})).unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("jobs.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "records": [{
                "id": "wizard",
                "name": "Wizard",
                "profile_id": WIZARD,
                "category": "combat",
                "max_level": 20,
                "members_only": false,
                "trainer_npc_ids": [],
                "quest_ids": [id],
                "abilities": [],
                "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic job"}]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    directory
}

fn assert_load_refused(fixture: &Fixture, quest: Value, label: &str, expected: &str) {
    let directory = synthetic_content(&quest);
    let mut game = fixture.game(None);
    let result = game.bind_npc_quest_catalogue(&directory, CHARACTER);
    let _ = fs::remove_dir_all(&directory);
    let error = result
        .err()
        .unwrap_or_else(|| panic!("{label} must be refused at content load"));
    assert!(
        error.to_string().contains(expected),
        "{label} must fail for {expected:?}, got: {error}"
    );
    // The refusal left the previously bound catalogue in place and changed no save byte.
    assert_eq!(saved(&game), saved(&fixture.game(None)));
}

/// Content assertions: exactly the two chapter records are enabled, every target is a
/// drawable `legacy_data` Robgoblin Camp actor, and the search areas are anchored to real
/// spawn ground.
#[test]
fn enabled_wizard_chapter_records_are_complete_and_drawable() {
    let document = read_json(&content_dir().join("quests/wizard.json"));
    let records = document["records"].as_array().unwrap();
    let enabled: Vec<&Value> = records
        .iter()
        .filter(|record| record["enabled"].as_bool().unwrap_or(false))
        .collect();
    assert_eq!(
        enabled
            .iter()
            .map(|record| record["id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![u64::from(BOOK), u64::from(GLASSES), u64::from(STICKY)]
    );
    for record in &enabled {
        assert_eq!(record["giver_npc_id"].as_u64(), Some(FIZZLESTICKS));
        assert_eq!(record["job"].as_i64(), Some(i64::from(WIZARD)));
        let objectives = record["objectives"].as_array().unwrap();
        assert_eq!(
            objectives.len(),
            2,
            "an enabled quest needs real objectives"
        );
        assert!(
            objectives.iter().all(|goal| goal["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "every objective needs provenance"
        );
        let rewards = record["rewards"].as_array().unwrap();
        assert!(!rewards.is_empty(), "every enabled Wizard record pays");
        assert_eq!(rewards[0]["coins"].as_u64(), Some(u64::from(REWARD_COINS)));
        assert_eq!(
            rewards[0]["job_xp"]["12"].as_u64(),
            Some(u64::from(REWARD_XP))
        );
        assert_eq!(rewards[0]["items"], serde_json::json!([]));
        assert!(rewards.iter().all(|reward| {
            reward["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())
        }));
    }

    let book = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(BOOK)))
        .unwrap();
    assert_eq!(
        book["title_string_id"].as_u64(),
        Some(u64::from(BOOK_TITLE))
    );
    assert_eq!(
        book["description_string_id"].as_u64(),
        Some(u64::from(BOOK_DESCRIPTION))
    );
    assert_eq!(
        book["intro_text_string_id"].as_u64(),
        Some(u64::from(BOOK_INTRO))
    );
    assert_eq!(
        book["end_text_string_id"].as_u64(),
        Some(u64::from(BOOK_END_TEXT))
    );
    assert_eq!(book["prerequisite_quest_ids"], serde_json::json!([]));
    assert_eq!(book["objectives"][0]["kind"], "visit_position");
    assert_eq!(book["objectives"][0]["zone_id"].as_u64(), Some(0));
    let radius = book["objectives"][0]["radius"].as_f64().unwrap();
    assert!((4.0..=8.0).contains(&radius), "the camp search radius");
    assert_eq!(book["objectives"][1]["kind"], "talk_to_npc");
    assert_eq!(book["objectives"][1]["npc_id"].as_u64(), Some(FIZZLESTICKS));

    let glasses = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(GLASSES)))
        .unwrap();
    assert_eq!(
        glasses["title_string_id"].as_u64(),
        Some(u64::from(GLASSES_TITLE))
    );
    assert_eq!(
        glasses["description_string_id"].as_u64(),
        Some(u64::from(GLASSES_DESCRIPTION))
    );
    assert_eq!(
        glasses["intro_text_string_id"].as_u64(),
        Some(u64::from(GLASSES_INTRO))
    );
    assert_eq!(
        glasses["end_text_string_id"].as_u64(),
        Some(u64::from(GLASSES_END_TEXT))
    );
    assert_eq!(glasses["prerequisite_quest_ids"], serde_json::json!([BOOK]));
    assert_eq!(glasses["objectives"][0]["kind"], "visit_position");
    assert_eq!(glasses["objectives"][1]["kind"], "talk_to_npc");
    assert_eq!(
        glasses["objectives"][1]["npc_id"].as_u64(),
        Some(FIZZLESTICKS)
    );

    // The two search areas are anchored to the ground of drawable camp NPCs, and stay
    // outside the 12 m Fizzlesticks talk ring so neither step completes on acceptance.
    let npcs = read_json(&content_dir().join("npcs/wilds.json"));
    let regional = |content_key: u64| -> &Value {
        let rows: Vec<&Value> = npcs["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["npc_id"].as_u64() == Some(content_key))
            .collect();
        assert_eq!(
            rows.len(),
            1,
            "NPC {content_key} must have one regional row"
        );
        rows[0]
    };
    for (content_key, role) in [
        (FIZZLESTICKS, "trainer"),
        (NURBLE, "quest_giver"),
        (SQUEE, "quest_giver"),
    ] {
        let row = regional(content_key);
        assert_eq!(row["placement"].as_str(), Some("legacy_data"));
        assert_eq!(row["zone_id"].as_u64(), Some(16));
        assert!(
            row["roles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == role),
            "NPC {content_key} must be a drawable {role}"
        );
    }
    let fizzlesticks_ground = content_position("wilds.json", FIZZLESTICKS);
    let distance = |a: [f32; 3], b: [f32; 3]| -> f32 {
        (a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f32>()).sqrt()
    };
    for (record, anchor, name) in [(book, SQUEE, "spell-book"), (glasses, NURBLE, "glasses")] {
        let point: [f32; 3] =
            serde_json::from_value(record["objectives"][0]["position"].clone()).unwrap();
        let radius = record["objectives"][0]["radius"].as_f64().unwrap() as f32;
        let anchor_ground = content_position("wilds.json", anchor);
        assert!(
            distance(point, anchor_ground) < 0.001,
            "the {name} search area must sit on the {anchor} spawn ground"
        );
        assert!(
            distance(point, fizzlesticks_ground) > 12.0 + radius,
            "the {name} search area must be outside the talk ring plus its radius"
        );
    }

    // T303 enables the Nurble cleaning errand: two ordered Talks and the same 5/25
    // first-chapter reward as the two records before it.
    let sticky = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(STICKY)))
        .unwrap();
    assert_eq!(sticky["enabled"].as_bool(), Some(true));
    assert_eq!(
        sticky["title_string_id"].as_u64(),
        Some(u64::from(STICKY_TITLE))
    );
    assert_eq!(
        sticky["description_string_id"].as_u64(),
        Some(u64::from(STICKY_DESCRIPTION))
    );
    assert_eq!(
        sticky["intro_text_string_id"].as_u64(),
        Some(u64::from(STICKY_INTRO))
    );
    assert_eq!(
        sticky["end_text_string_id"].as_u64(),
        Some(u64::from(STICKY_END_TEXT))
    );
    assert_eq!(
        sticky["prerequisite_quest_ids"],
        serde_json::json!([GLASSES])
    );
    assert_eq!(sticky["objectives"][0]["kind"], "talk_to_npc");
    assert_eq!(sticky["objectives"][0]["npc_id"].as_u64(), Some(NURBLE));
    assert_eq!(sticky["objectives"][1]["kind"], "talk_to_npc");
    assert_eq!(
        sticky["objectives"][1]["npc_id"].as_u64(),
        Some(FIZZLESTICKS)
    );
}

#[test]
fn two_wizards_progress_independently() {
    let fixture = Fixture::new();
    let first_character = avatar_at(FIZZLESTICKS);
    let second_character = CharacterState {
        guid: SECOND_CHARACTER,
        ..first_character.clone()
    };
    let book_search = content_position("wilds.json", SQUEE);
    let mut now = 5_000;
    let mut first = wizard_game(&fixture);
    let mut second = fixture.game_as(SECOND_CHARACTER, Some(wizard_state(&fixture)));

    for (game, character) in [
        (&mut first, &first_character),
        (&mut second, &second_character),
    ] {
        now += 10;
        offer(game, character, FIZZLESTICKS, BOOK, now);
        now += 10;
        accept(game, character, BOOK, now);
    }
    for game in [&first, &second] {
        assert_eq!(counts(game, BOOK), vec![0, 0]);
    }

    // One player's search never enters the other's state.
    now += 10;
    assert!(visit(&mut first, book_search, now).changed);
    assert_eq!(counts(&first, BOOK), vec![1, 0]);
    assert_eq!(counts(&second, BOOK), vec![0, 0]);
    assert_eq!(pending_offer(&second), None);

    // The second player's own search and turn-in pay only that character.
    now += 10;
    assert!(visit(&mut second, book_search, now).changed);
    assert_eq!(counts(&second, BOOK), vec![1, 0]);
    let second_coins = coins(&second);
    assert!(
        talk(&mut second, &second_character, FIZZLESTICKS, now + 1).changed,
        "the second Wizard completes the same search step independently"
    );
    assert_eq!(counts(&second, BOOK), vec![1, 1]);
    assert_eq!(counts(&first, BOOK), vec![1, 0]);
    assert_eq!(pending_turnin(&second), Some(BOOK));
    assert_eq!(pending_turnin(&first), None);
    assert_eq!(coins(&second), second_coins, "no reward before the turn-in");
}
