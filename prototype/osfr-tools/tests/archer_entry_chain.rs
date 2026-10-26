//! T194/F217: Archer entry chapter, Path of the Archer (3010003) and Oath of Honor
//! (3010006). The chapter is content-only; these tests drive the production NPC
//! packet router (`Gameplay::handle_in_world` -> `gameplay/npc.rs` ->
//! `QuestEngine`) instead of calling engine internals.
//!
//! Request/reply layouts used here:
//! - 26/8 NPC interact: constructor `008b6bc0`; body is u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; body is u64 target + i32 choice.
//! - 26/6 dialogue response: constructor `009f5890`, sender `009f6740`; body is
//!   i32 response row (301 is the quest row).
//! - 49/2 quest reply: constructor `00a86400`, sender `00a97d70`; u32 quest + u8 accept.
//! - 49/14 end reply: constructor `00a864c0`, sender `00a97ef0`; u32 quest.
//! 49/1 offers are asserted only by their quest id at 6..10. 49/13 end screens
//! name the speaker at 6..14, the quest at 14..18 and the end-text locale id at
//! 18..22 (`QuestEndPacket::ReadFields` `00c7b990`; `ShowQuestEndScreen`
//! `00a95420` passes that third dword as the message id).
//!
//! The three recruit talk targets are existing unnamed Legacy Greenwood archer
//! actors (spawn GUIDs 100000003468-3470) bound to the recruit content keys in
//! `npc_spawns_extra.json`; their regional rows are unanchored scaffolding and
//! deliberately do not spawn a duplicate. The client's fourth Path step is
//! "Return to Lugabow" (locale 22083), so the turn-in is at the giver.

#[path = "support/archer_entry.rs"]
mod archer_entry;

use archer_entry::*;
use osfr_tools::quest_engine::QuestStatus;

#[test]
fn lugabow_offers_path_only_to_archers() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 1_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);

    activate_brawler(&mut game, &character, &mut now);
    assert_no_offer(
        &mut game,
        &character,
        LUGABOW,
        &mut now,
        "Brawler at Lugabow",
    );

    learn_archer(&mut game, &character, &mut now);
    offer(&mut game, &character, LUGABOW, PATH, now);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&OATH),
        "Oath of Honor stays locked behind Path of the Archer"
    );
}

#[test]
fn path_chapter_completes_in_order_and_pays_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 2_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    start_path(&mut game, &character, &mut now);
    let coins_before = coins(&game);
    let xp_before = archer_xp(&game);

    // Ordered: the second recruit cannot count before the first.
    let before_wrong_order = saved(&game);
    let (out_of_order, _, _) = talk_recruit(&mut game, HRUGAGU, &mut now);
    assert!(!out_of_order.changed);
    assert_eq!(counts(&game, PATH), vec![0, 0, 0, 0]);
    assert_eq!(saved(&game), before_wrong_order);

    let (first, _, _) = talk_recruit(&mut game, BOLLYLOG, &mut now);
    assert!(first.changed);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0, 0]);
    assert_eq!(pending_turnin(&game), None);

    let (second, _, _) = talk_recruit(&mut game, HRUGAGU, &mut now);
    assert!(second.changed);
    assert_eq!(counts(&game, PATH), vec![1, 1, 0, 0]);
    assert_eq!(pending_turnin(&game), None);

    let (third, _, _) = talk_recruit(&mut game, BRUGADUG, &mut now);
    assert!(third.changed);
    assert_eq!(counts(&game, PATH), vec![1, 1, 1, 0]);
    assert_eq!(
        pending_turnin(&game),
        None,
        "the third recruit is not the turn-in"
    );
    assert!(
        !third
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the client's last step returns to Lugabow, not the third recruit"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");
    assert_eq!(archer_xp(&game), xp_before);

    // Client step 4, "Return to Lugabow" (locale 22083): the giver opens the
    // end screen, held for the 26/6 dialogue choice.
    now += 10;
    let returned = talk(&mut game, &character, LUGABOW, now);
    assert!(returned.changed);
    assert_eq!(counts(&game, PATH), vec![1, 1, 1, 1]);
    assert_eq!(pending_turnin(&game), Some(PATH));
    assert!(
        !returned
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");
    assert_eq!(archer_xp(&game), xp_before);

    now += 10;
    let _ = release_end_screen(&mut game, &character, PATH, LUGABOW, PATH_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game, &character, PATH, ARCHER, PATH_COINS, PATH_XP, now,
    );
    assert_eq!(coins(&game), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&game), xp_before + PATH_XP);

    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(PATH),
        now,
        "duplicate turn-in",
    );
    assert_eq!(coins(&game), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&game), xp_before + PATH_XP);
}

#[test]
fn oath_chapter_requires_path_and_talks_to_hilforn() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 3_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    start_path(&mut game, &character, &mut now);

    // Active Path blocks Oath: the prerequisite is not completed yet.
    assert_no_offer(&mut game, &character, LUGABOW, &mut now, "Oath before Path");
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&OATH)
    );

    complete_path_via_recruits(&mut game, &mut now);
    assert_eq!(quest_status(&game, PATH), QuestStatus::Completed);
    let coins_after_path = coins(&game);
    let xp_after_path = archer_xp(&game);

    now += 10;
    offer(&mut game, &character, LUGABOW, OATH, now);
    now += 10;
    accept(&mut game, &character, OATH, now);
    complete_oath_via_hilforn(&mut game, &mut now);

    assert_eq!(coins(&game), coins_after_path + OATH_COINS);
    assert_eq!(archer_xp(&game), xp_after_path + OATH_XP);
    assert_eq!(quest_status(&game, OATH), QuestStatus::Completed);
    // Completed Path is not offered again.
    assert_no_offer(
        &mut game,
        &character,
        LUGABOW,
        &mut now,
        "completed Path after Oath",
    );
}

#[test]
fn wrong_npc_and_replay_do_not_advance_or_pay() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 4_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    start_path(&mut game, &character, &mut now);
    let coins_before = coins(&game);
    let xp_before = archer_xp(&game);

    for (npc_id, zone, file) in [
        (JONES, COBBLESTONE, "cobblestone_village.json"),
        (BARTLE, COBBLESTONE, "cobblestone_village.json"),
    ] {
        let position = content_position(file, npc_id);
        let wrong_character = character_at(zone, position);
        let before = saved(&game);
        let dialogue = talk(&mut game, &wrong_character, npc_id, now);
        now += 10;
        let closed = send(
            &mut game,
            &wrong_character,
            &dialogue_choice(QUEST_CHOICE),
            now,
        );
        now += 10;
        assert!(!dialogue.changed);
        assert!(
            !closed
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 1, 0]))
        );
        assert_eq!(counts(&game, PATH), vec![0, 0, 0, 0]);
        assert_eq!(saved(&game), before, "wrong NPC {npc_id} changed the save");
        assert_eq!(coins(&game), coins_before);
        assert_eq!(archer_xp(&game), xp_before);
    }

    // First objective, then replay the same Talk: ordered progress does not move.
    let (first, first_guid, first_position) = talk_recruit(&mut game, BOLLYLOG, &mut now);
    assert!(first.changed);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0, 0]);
    let after_first = saved(&game);
    let replay_character = character_at(GREENWOOD, first_position);
    let replayed = talk(&mut game, &replay_character, first_guid, now + 1);
    now += 10;
    assert!(!replayed.changed);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0, 0]);
    assert_eq!(
        saved(&game),
        after_first,
        "replaying a completed objective changed the save"
    );

    complete_path_via_recruits(&mut game, &mut now);
    assert_eq!(coins(&game), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&game), xp_before + PATH_XP);

    // A completed quest is not advanced or paid again by any old packet.
    let completed_bytes = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(PATH),
        now + 1,
        "replayed turn-in after completion",
    );
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(PATH, true),
        now + 2,
        "replayed accept after completion",
    );
    let bollylog_character = character_at(GREENWOOD, first_position);
    let after_completion = talk(&mut game, &bollylog_character, first_guid, now + 3);
    assert!(!after_completion.changed);
    assert_eq!(saved(&game), completed_bytes);
    assert_eq!(coins(&game), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&game), xp_before + PATH_XP);
}

#[test]
fn relog_keeps_completed_history_and_no_duplicate_reward() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 5_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    let starting_coins = coins(&game);
    let starting_xp = archer_xp(&game);
    start_path(&mut game, &character, &mut now);

    // Disconnect mid-activity after one objective.
    let (first, _, _) = talk_recruit(&mut game, BOLLYLOG, &mut now);
    assert!(first.changed);
    let mid_bytes = saved(&game);
    game = fixture.game(Some(game.state()));
    assert_eq!(
        saved(&game),
        mid_bytes,
        "reload changed the active quest state"
    );
    assert_eq!(quest_status(&game, PATH), QuestStatus::Active);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0, 0]);

    complete_path_via_recruits(&mut game, &mut now);
    now += 10;
    offer(&mut game, &character, LUGABOW, OATH, now);
    now += 10;
    accept(&mut game, &character, OATH, now);
    complete_oath_via_hilforn(&mut game, &mut now);

    let final_bytes = saved(&game);
    let final_coins = coins(&game);
    let final_xp = archer_xp(&game);
    assert_eq!(final_coins, starting_coins + PATH_COINS + OATH_COINS);
    assert_eq!(final_xp, starting_xp + PATH_XP + OATH_XP);

    // Reload the finished chapter: both quests stay in completed history.
    game = fixture.game(Some(game.state()));
    assert_eq!(
        saved(&game),
        final_bytes,
        "reload changed completed history"
    );
    for quest_id in [PATH, OATH] {
        assert_eq!(quest_status(&game, quest_id), QuestStatus::Completed);
    }
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), final_coins);
    assert_eq!(archer_xp(&game), final_xp);

    // Neither quest is offered again.
    assert_no_offer(
        &mut game,
        &character,
        LUGABOW,
        &mut now,
        "completed Path and Oath after relog",
    );
    // Replaying the old packets pays nothing and changes no save byte.
    for (index, quest_id) in [PATH, OATH].into_iter().enumerate() {
        let offset = u64::try_from(index).unwrap();
        assert_packet_changes_nothing(
            &mut game,
            &character,
            &quest_end_reply(quest_id),
            now + 10 + offset,
            "replayed turn-in after relog",
        );
        assert_packet_changes_nothing(
            &mut game,
            &character,
            &quest_reply(quest_id, true),
            now + 20 + offset,
            "replayed accept after relog",
        );
    }
    assert_eq!(coins(&game), final_coins);
    assert_eq!(archer_xp(&game), final_xp);
}

/// The client's fourth Path step was added after the first chapter shipped. A
/// save written by the old three-objective chapter cannot keep its counts:
/// `repair_counters` resets a changed objective list by design (R). Pin that
/// migration so the journal reset is visible and the chapter stays completable,
/// and prove the reset pays nothing.
#[test]
fn three_objective_path_save_resets_and_stays_completable() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 10_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    start_path(&mut game, &character, &mut now);
    let (first, _, _) = talk_recruit(&mut game, BOLLYLOG, &mut now);
    assert!(first.changed);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0, 0]);

    // Rewrite the progress as the old chapter saved it: three counts and the
    // held end screen at the third recruit.
    let mut old = game.state();
    let quests = old.npc_quests.as_mut().unwrap();
    let progress = quests.quests.get_mut(&PATH).unwrap();
    progress.counts = vec![1, 0, 0];
    quests.pending_turnin = Some(PATH);
    quests.pending_turnin_actor = Some(BRUGADUG);
    // Load-time migrations must not replace the saved economy with a stale
    // snapshot. Position is on `CharacterState`, not `GameplayState`.
    let inventory_before = old.inventory.clone();
    let coins_before = coins(&game);
    let xp_before = archer_xp(&game);

    let mut reloaded = fixture.game(Some(old));
    assert_eq!(
        counts(&reloaded, PATH),
        vec![0, 0, 0, 0],
        "a changed objective list resets the counters (content-migration policy)"
    );
    assert_eq!(quest_status(&reloaded, PATH), QuestStatus::Active);
    assert_eq!(pending_turnin(&reloaded), None);
    assert_eq!(
        reloaded
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .pending_turnin_actor,
        None,
        "the migration clears the held speaker too"
    );
    assert_eq!(coins(&reloaded), coins_before, "the reset pays nothing");
    assert_eq!(archer_xp(&reloaded), xp_before, "the reset grants no XP");
    assert_eq!(
        reloaded.state().inventory,
        inventory_before,
        "the reset must not replace the saved inventory"
    );

    complete_path_via_recruits(&mut reloaded, &mut now);
    assert_eq!(quest_status(&reloaded, PATH), QuestStatus::Completed);
    assert_eq!(coins(&reloaded), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&reloaded), xp_before + PATH_XP);
}

#[test]
fn relog_replays_the_held_end_screen_at_its_speaker() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 9_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    start_path(&mut game, &character, &mut now);

    for (content_key, expected) in [
        (BOLLYLOG, vec![1, 0, 0, 0]),
        (HRUGAGU, vec![1, 1, 0, 0]),
        (BRUGADUG, vec![1, 1, 1, 0]),
    ] {
        let (outcome, _, _) = talk_recruit(&mut game, content_key, &mut now);
        assert!(outcome.changed);
        assert_eq!(counts(&game, PATH), expected);
    }
    now += 10;
    let returned = talk(&mut game, &character, LUGABOW, now);
    assert!(returned.changed);
    assert_eq!(pending_turnin(&game), Some(PATH));

    // Disconnect while the 49/13 is held for the 26/6 choice.
    let held_bytes = saved(&game);
    game = fixture.game(Some(game.state()));
    assert_eq!(saved(&game), held_bytes, "reload changed the held state");
    assert_eq!(pending_turnin(&game), Some(PATH));
    assert_eq!(
        game.state()
            .npc_quests
            .as_ref()
            .unwrap()
            .pending_turnin_actor,
        Some(LUGABOW),
        "the saved pending actor is the content key that opened the screen"
    );

    // The relog visibility batch must spawn the speaker and re-send its 49/13.
    // Open-world NPC visibility uses the shared open-world zone label (0), not
    // the regional zone id used for range checks.
    let visibility_character = character_at(0, lugabow);
    let packets = game
        .update_npc_visibility(&visibility_character)
        .expect("relog NPC visibility");
    let spawn = packets.iter().position(|packet| {
        osfr_tools::startup_world::parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == LUGABOW)
    });
    let end = packets
        .iter()
        .position(|packet| has_family(packet, 49, 13, Some(PATH)))
        .expect("the held end screen is re-sent after relog");
    assert!(
        spawn.is_some_and(|spawn| spawn < end),
        "the speaker's 35/2 must precede its 49/13"
    );
    assert_eq!(
        u64::from_le_bytes(packets[end][6..14].try_into().unwrap()),
        LUGABOW,
        "the re-sent end screen must name the runtime speaker"
    );
    assert_eq!(
        u32::from_le_bytes(packets[end][18..22].try_into().unwrap()),
        PATH_END_TEXT,
        "the re-sent end screen must carry the Path debrief text"
    );

    now += 10;
    finish_quest(
        &mut game,
        &visibility_character,
        PATH,
        ARCHER,
        PATH_COINS,
        PATH_XP,
        now,
    );
}

#[test]
fn non_archer_is_refused_with_no_state_change() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 6_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    activate_brawler(&mut game, &character, &mut now);

    let before = saved(&game);
    let coins_before = coins(&game);
    let xp_before = archer_xp(&game);
    let dialogue = talk(&mut game, &character, LUGABOW, now + 1);
    now += 10;
    let closed = send(&mut game, &character, &dialogue_choice(QUEST_CHOICE), now);
    assert!(!dialogue.changed);
    assert!(
        !closed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
        "a Brawler must not receive Path of the Archer"
    );
    assert_eq!(
        saved(&game),
        before,
        "the non-Archer refusal changed the save"
    );
    assert_eq!(coins(&game), coins_before);
    assert_eq!(archer_xp(&game), xp_before);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&PATH)
    );
    assert_eq!(pending_turnin(&game), None);
}

#[test]
fn path_reward_xp_follows_the_archer_profile_after_a_job_switch() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 7_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    let archer_xp_before = archer_xp(&game);
    start_path(&mut game, &character, &mut now);

    // The accepted quest survives a job switch; its reward map still names 35.
    now += 10;
    let switched = send(&mut game, &character, &select_job(BRAWLER), now);
    assert!(switched.changed);
    assert_eq!(game.state().profiles.as_ref().unwrap().active, BRAWLER);
    let brawler_xp_before = profile_xp(&game, BRAWLER);
    complete_path_via_recruits(&mut game, &mut now);
    assert_eq!(archer_xp(&game), archer_xp_before + PATH_XP);
    assert_eq!(profile_xp(&game, BRAWLER), brawler_xp_before);
}

#[test]
fn two_archers_count_the_same_recruit_independently() {
    let fixture = Fixture::new();
    let mut first = fixture.game(None);
    let mut second = fixture.game(None);
    let mut now = 8_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut first, &character, &mut now);
    learn_archer(&mut second, &character, &mut now);
    start_path(&mut first, &character, &mut now);
    start_path(&mut second, &character, &mut now);

    for game in [&mut first, &mut second] {
        let (runtime_guid, position) = recruit_spawn(BOLLYLOG);
        let at_recruit = character_at(GREENWOOD, position);
        let outcome = talk(game, &at_recruit, runtime_guid, now);
        assert!(outcome.changed);
        assert_eq!(counts(game, PATH), vec![1, 0, 0, 0]);
    }
    assert_eq!(counts(&first, PATH), vec![1, 0, 0, 0]);
    assert_eq!(counts(&second, PATH), vec![1, 0, 0, 0]);
}

/// `lab/autopilot/protected-pass.txt` pins this name in this target. The content
/// assertions live in the shared support module.
#[test]
fn enabled_archer_chapter_records_are_complete_and_others_stay_disabled() {
    assert_enabled_archer_chapter_records();
}
