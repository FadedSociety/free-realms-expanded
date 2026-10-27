//! T303: "Sticky Spell Book" (3160005), Fizzlesticks' third Wizard chapter.
//!
//! The chapter is content-only: the record sends the Wizard to Junkmaster Nurble and back.
//! These tests drive the production request path (`Gameplay::handle_in_world` ->
//! `gameplay/npc.rs` -> `QuestEngine`) and the production NPC visibility loader
//! (`NpcVisibility::load`). Every reply layout used here was re-checked against the 2014
//! client this run:
//! - 26/8 NPC interact: constructor `008b6bc0`; body is u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; body is u64 target + i32 choice.
//! - 26/6 dialogue response: constructor `009f5890`; one i32 response row (301 is the quest row).
//! - 49/1 QuestInfo: reader `00c7bb60` -> `00c7b7a0` (six dwords: quest id, intro, an unresolved
//!   field, the card title/name, the description, an unresolved field), decoded with the
//!   production `decode_quest_offer_49_1`.
//! - 49/2 quest reply: constructor `00a86400` / sender `00a97d70`; i32 quest + u8 accept.
//! - 49/13 QuestEnd: reader `00c7b990`; actor u64 at 6..14, quest i32 at 14..18, end-text
//!   locale at 18..22, animation at 22..26, reward bundle from 26 (69-byte body via `008e7930`,
//!   row count at 87..91 of the packet).
//! - 49/14 end reply: constructor `00a864c0`; i32 quest.
//!
//! The chapter fixture and the 26/8/26/10/26/6/49/2/49/14 helpers are the shared
//! `tests/support/npc_quest_chain.rs` harness; Wizard vocabulary and reward assertions live in
//! `tests/support/wizard_entry.rs`. The first two chapters are played through the same
//! production path, not seeded, so the 3160004 prerequisite is a real completion.

#[path = "support/wizard_entry.rs"]
mod wizard_entry;

use osfr_tools::{
    character_store::CharacterState,
    encounter::EncounterEngine,
    game_store,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
    startup_world::parse_add_npc,
    world::NpcVisibility,
};
use serde_json::Value;
use wizard_entry::*;

/// The fixture's recovered dungeon encounter (same id `tests/type8_gateway.rs` begins).
const ENCOUNTER_ID: u32 = 411;

fn has_quest(game: &Gameplay, quest_id: u32) -> bool {
    game.state()
        .npc_quests
        .as_ref()
        .unwrap()
        .quests
        .contains_key(&quest_id)
}

/// `gateway.rs` sends the validated pose to `Gameplay::on_location`; the first two chapters
/// consume their `visit_position` objectives through it. The character, not the module-level
/// default, names whose location moved.
fn visit(game: &mut Gameplay, character: &CharacterState, position: [f32; 3], now: u64) -> Outcome {
    game.on_location(&character_at_for(character.guid, OPEN_WORLD, position), now)
        .expect("location event")
}

/// Play one T242 chapter through the real offer/accept/visit/talk/end-screen/turn-in path.
fn play_chapter(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    search: [f32; 3],
    end_text: u32,
    now: &mut u64,
) {
    *now += 10;
    offer(game, character, FIZZLESTICKS, quest_id, *now);
    *now += 10;
    accept(game, character, quest_id, *now);
    *now += 10;
    assert!(
        visit(game, character, search, *now).changed,
        "quest {quest_id}'s search must advance"
    );
    *now += 10;
    assert!(
        talk(game, character, FIZZLESTICKS, *now).changed,
        "quest {quest_id}'s hand-in must advance"
    );
    *now += 10;
    release_wizard_end_screen(game, character, quest_id, end_text, *now);
    *now += 10;
    finish_quest(
        game,
        character,
        quest_id,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        *now,
    );
}

/// A Wizard who has completed 3160003 and 3160004, standing at Fizzlesticks.
fn sticky_ready(fixture: &Fixture, character: &CharacterState) -> (Gameplay, u64) {
    let mut game = fixture.game_as(character.guid, Some(wizard_state(fixture)));
    let mut now = 1_000;
    play_chapter(
        &mut game,
        character,
        BOOK,
        content_position("wilds.json", SQUEE),
        BOOK_END_TEXT,
        &mut now,
    );
    play_chapter(
        &mut game,
        character,
        GLASSES,
        content_position("wilds.json", NURBLE),
        GLASSES_END_TEXT,
        &mut now,
    );
    assert_eq!(quest_status(&game, GLASSES), QuestStatus::Completed);
    (game, now)
}

fn accept_sticky(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    offered_card(game, character, FIZZLESTICKS, STICKY, *now);
    *now += 10;
    accept(game, character, STICKY, *now);
}

/// Save through the production `game_store` envelope and reload through the same bind the
/// gateway's reconnect uses.
fn save_reload(fixture: &Fixture, game: &Gameplay, character: &CharacterState) -> Gameplay {
    let path = fixture.path().join("sticky-spell-book-save.json");
    let snapshot = game_store::Snapshot {
        character: character.clone(),
        gameplay: Some(game.state()),
        pets: None,
    };
    game_store::save(&path, &snapshot).expect("game_store saves the character");
    let loaded = game_store::load_or_create(&path, character.clone()).expect("game_store loads");
    assert_eq!(
        loaded.character, *character,
        "the character survived the save"
    );
    fixture.game_as(
        character.guid,
        Some(loaded.gameplay.expect("the save carries gameplay")),
    )
}

/// A real conversation that must leave the quest, menu and save untouched.
fn assert_talk_changes_nothing(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: u64,
    label: &str,
) {
    let before = saved(game);
    let talked = talk(game, character, target, now);
    assert!(!talked.changed, "{label} changed gameplay");
    assert!(
        !talked
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None) || has_family(packet, 49, 13, None)),
        "{label} offered or turned in a quest"
    );
    let closed = send(game, character, &dialogue_choice(QUEST_CHOICE), now + 2);
    assert!(
        !closed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None) || has_family(packet, 49, 13, None)),
        "{label} released an offer or end screen on close"
    );
    assert_eq!(saved(game), before, "{label} changed the save bytes");
}

/// Every content row and objective/reward needs provenance; reconstructed tags are marked.
fn assert_provenance(value: &Value, label: &str) {
    let source = value["source"]
        .as_array()
        .unwrap_or_else(|| panic!("{label} has provenance"));
    assert!(!source.is_empty(), "{label} has provenance");
    for tag in source {
        let evidence = tag["evidence"]
            .as_str()
            .unwrap_or_else(|| panic!("{label} evidence is a string"));
        assert!(!evidence.is_empty(), "{label} evidence is non-empty");
        if tag["kind"] == "reconstructed" {
            assert!(
                evidence.starts_with("(R)"),
                "{label} reconstructed evidence starts with (R): {evidence}"
            );
        }
    }
}

#[test]
fn nurble_then_fizzlesticks_completes_and_pays_once() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let (mut game, mut now) = sticky_ready(&fixture, &character);
    let at_nurble = avatar_at(NURBLE);
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);

    now += 10;
    let card = offered_card(&mut game, &character, FIZZLESTICKS, STICKY, now);
    assert_eq!(card.intro_text_string_id, STICKY_INTRO);
    assert_eq!(card.name_string_id, STICKY_TITLE);
    assert_eq!(card.description_string_id, STICKY_DESCRIPTION);
    assert_eq!(pending_offer(&game), Some(STICKY));
    assert_eq!(counts(&game, STICKY), vec![0, 0]);

    now += 10;
    let accepted = accept(&mut game, &character, STICKY, now);
    assert_eq!(pending_offer(&game), None);
    assert!(
        objective_activated(&accepted, STICKY, 1),
        "accept activates the Nurble objective"
    );
    assert!(
        !objective_activated(&accepted, STICKY, 2),
        "an ordered quest activates only the first objective"
    );

    // The return talk is out of order until Nurble has the book.
    assert_talk_changes_nothing(
        &mut game,
        &character,
        FIZZLESTICKS,
        now + 1,
        "return talk before Nurble",
    );
    assert_eq!(counts(&game, STICKY), vec![0, 0]);
    assert_eq!(pending_turnin(&game), None);

    // Nurble advances exactly one ordered counter and does not end the quest.
    now += 10;
    let to_nurble = talk(&mut game, &at_nurble, NURBLE, now);
    assert!(to_nurble.changed);
    assert_eq!(counts(&game, STICKY), vec![1, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(objective_update(&to_nurble, STICKY, 1, 1));
    assert!(objective_complete(&to_nurble, STICKY, 1));
    assert!(
        objective_activated(&to_nurble, STICKY, 2),
        "the second objective opens after the first completes"
    );
    assert_eq!(
        to_nurble
            .replies
            .iter()
            .filter(|packet| has_family(packet, 49, 9, Some(STICKY)))
            .count(),
        1,
        "the Nurble Talk advances exactly one counter"
    );
    assert!(
        !to_nurble
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the first step is not the turn-in"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");
    assert_eq!(profile_xp(&game, WIZARD), xp_before);

    assert_talk_changes_nothing(
        &mut game,
        &at_nurble,
        NURBLE,
        now + 1,
        "duplicate Nurble talk",
    );

    // Returning to Fizzlesticks completes the second counter and holds the 49/13.
    now += 10;
    let to_fizzlesticks = talk(&mut game, &character, FIZZLESTICKS, now);
    assert!(to_fizzlesticks.changed);
    assert_eq!(counts(&game, STICKY), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(STICKY));
    assert!(objective_update(&to_fizzlesticks, STICKY, 2, 1));
    assert!(objective_complete(&to_fizzlesticks, STICKY, 2));
    assert!(
        !to_fizzlesticks
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the end screen is held for the 26/6 dialogue choice"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");

    now += 10;
    release_wizard_end_screen(&mut game, &character, STICKY, STICKY_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        STICKY,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), xp_before + REWARD_XP);
    assert_eq!(quest_status(&game, STICKY), QuestStatus::Completed);

    // A replayed end reply and accept pay nothing, and no Talk re-offers the record.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(STICKY),
        now + 1,
        "replayed turn-in",
    );
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(STICKY, true),
        now + 2,
        "replayed accept",
    );
    assert_talk_changes_nothing(
        &mut game,
        &character,
        FIZZLESTICKS,
        now + 3,
        "talk after completion",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), xp_before + REWARD_XP);
}

#[test]
fn the_wizard_xp_goes_to_profile_12_even_after_a_job_switch() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let (mut game, mut now) = sticky_ready(&fixture, &character);
    accept_sticky(&mut game, &character, &mut now);
    now += 10;
    assert!(talk(&mut game, &avatar_at(NURBLE), NURBLE, now).changed);

    // Switch the active job after accepting and after the delivery step.
    now += 10;
    assert!(
        send(&mut game, &character, &select_job(BRAWLER), now).changed,
        "the active job switch is accepted"
    );
    assert_eq!(active_job(&game), BRAWLER);
    let brawler_xp = profile_xp(&game, BRAWLER);
    let wizard_xp = profile_xp(&game, WIZARD);

    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    assert_eq!(pending_turnin(&game), Some(STICKY));
    now += 10;
    // The 49/13 star total is the quest's own job XP, not the active job's.
    release_wizard_end_screen(&mut game, &character, STICKY, STICKY_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        STICKY,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(profile_xp(&game, WIZARD), wizard_xp + REWARD_XP);
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_xp,
        "the active job's XP must not change"
    );
}

#[test]
fn progress_and_payment_survive_a_reload_and_a_replay_pays_nothing() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let (mut game, mut now) = sticky_ready(&fixture, &character);
    let starting_coins = coins(&game);
    let starting_xp = profile_xp(&game, WIZARD);
    accept_sticky(&mut game, &character, &mut now);
    now += 10;
    assert!(talk(&mut game, &avatar_at(NURBLE), NURBLE, now).changed);

    // Disconnect after the Nurble Talk: the counter is in the character save.
    let mid = saved(&game);
    game = save_reload(&fixture, &game, &character);
    assert_eq!(saved(&game), mid, "the reload changed the active quest");
    assert_eq!(quest_status(&game, STICKY), QuestStatus::Active);
    assert_eq!(counts(&game, STICKY), vec![1, 0]);
    assert_eq!(coins(&game), starting_coins, "a disconnect pays nothing");
    assert_eq!(profile_xp(&game, WIZARD), starting_xp);

    // Return to Fizzlesticks on the live session, release the held end screen, and pay once.
    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    assert_eq!(pending_turnin(&game), Some(STICKY));
    assert_eq!(
        coins(&game),
        starting_coins,
        "the reconnect and the held end screen pay nothing"
    );
    now += 10;
    release_wizard_end_screen(&mut game, &character, STICKY, STICKY_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        STICKY,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&game), starting_coins + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), starting_xp + REWARD_XP);

    // Reload the paid state: Completed, the exact wallet, and no replay payment.
    let paid = saved(&game);
    game = save_reload(&fixture, &game, &character);
    assert_eq!(saved(&game), paid, "the reload changed completed history");
    assert_eq!(quest_status(&game, STICKY), QuestStatus::Completed);
    assert_eq!(coins(&game), starting_coins + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), starting_xp + REWARD_XP);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(STICKY),
        now + 1,
        "replayed turn-in after reload",
    );
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(STICKY, true),
        now + 2,
        "replayed accept after reload",
    );
    assert_eq!(coins(&game), starting_coins + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), starting_xp + REWARD_XP);
}

#[test]
fn refusals_and_objective_order_change_nothing() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let (mut game, mut now) = sticky_ready(&fixture, &character);
    accept_sticky(&mut game, &character, &mut now);
    let at_nurble = avatar_at(NURBLE);
    let at_squee = avatar_at(SQUEE);

    // An out-of-range 26/8 is rejected before the menu opens and changes no byte.
    let far = CharacterState {
        position: [
            character.position[0] + 200.0,
            character.position[1],
            character.position[2],
        ],
        ..character.clone()
    };
    let before = saved(&game);
    assert!(
        game.handle_in_world(&npc_interact(FIZZLESTICKS), &far, now + 1)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an out-of-range interaction must be refused"
    );
    assert_eq!(
        saved(&game),
        before,
        "the out-of-range request changed the save"
    );
    assert_eq!(pending_offer(&game), None);

    // Forged replies before any progress.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(STICKY, true),
        now + 2,
        "forged accept while active",
    );
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(STICKY),
        now + 3,
        "forged turn-in with no pending screen",
    );

    // Wrong NPC and the out-of-order return talk.
    assert_talk_changes_nothing(&mut game, &at_squee, SQUEE, now + 4, "wrong NPC");
    assert_talk_changes_nothing(
        &mut game,
        &character,
        FIZZLESTICKS,
        now + 5,
        "return talk before Nurble",
    );
    assert_eq!(counts(&game, STICKY), vec![0, 0]);

    now += 10;
    assert!(talk(&mut game, &at_nurble, NURBLE, now).changed);
    assert_eq!(counts(&game, STICKY), vec![1, 0]);
    assert_talk_changes_nothing(
        &mut game,
        &at_nurble,
        NURBLE,
        now + 1,
        "duplicate Nurble talk",
    );
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(STICKY),
        now + 2,
        "premature turn-in",
    );

    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    assert_eq!(pending_turnin(&game), Some(STICKY));
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(STICKY, true),
        now + 1,
        "replayed accept with a held end screen",
    );
    now += 10;
    release_wizard_end_screen(&mut game, &character, STICKY, STICKY_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        STICKY,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    let paid_coins = coins(&game);
    let paid_xp = profile_xp(&game, WIZARD);

    // Replayed accept and turn-in after completion change nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(STICKY),
        now + 1,
        "replayed turn-in after completion",
    );
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(STICKY, true),
        now + 2,
        "replayed accept after completion",
    );
    assert_talk_changes_nothing(
        &mut game,
        &character,
        FIZZLESTICKS,
        now + 3,
        "post-completion talk",
    );
    assert_eq!(coins(&game), paid_coins);
    assert_eq!(profile_xp(&game, WIZARD), paid_xp);
    assert_eq!(quest_status(&game, STICKY), QuestStatus::Completed);
}

#[test]
fn a_non_wizard_or_missing_prerequisite_is_not_offered_and_changes_nothing() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);

    // A non-Wizard with both prior chapters complete gets no offer at Fizzlesticks.
    let (mut game, mut now) = sticky_ready(&fixture, &character);
    now += 10;
    assert!(
        send(&mut game, &character, &select_job(BRAWLER), now).changed,
        "the active job switch is accepted"
    );
    assert_eq!(active_job(&game), BRAWLER);
    assert_no_offer(
        &mut game,
        &character,
        FIZZLESTICKS,
        &mut now,
        "non-Wizard with the prerequisite complete",
    );
    assert!(!has_quest(&game, STICKY));

    // A Wizard whose 3160004 is still in progress is not offered the next record.
    let mut game = fixture.game_as(CHARACTER, Some(wizard_state(&fixture)));
    let mut now = 2_000;
    play_chapter(
        &mut game,
        &character,
        BOOK,
        content_position("wilds.json", SQUEE),
        BOOK_END_TEXT,
        &mut now,
    );
    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, GLASSES, now);
    now += 10;
    accept(&mut game, &character, GLASSES, now);
    assert_no_offer(
        &mut game,
        &character,
        FIZZLESTICKS,
        &mut now,
        "3160004 still in progress",
    );
    assert!(!has_quest(&game, STICKY));
}

#[test]
fn two_wizards_progress_independently() {
    let fixture = Fixture::new();
    let first_character = avatar_at(FIZZLESTICKS);
    let second_character = CharacterState {
        guid: SECOND_CHARACTER,
        ..first_character.clone()
    };
    let first_at_nurble = avatar_at(NURBLE);
    let second_at_nurble = CharacterState {
        guid: SECOND_CHARACTER,
        ..first_at_nurble.clone()
    };
    let (mut first, mut now) = sticky_ready(&fixture, &first_character);
    let (mut second, _) = sticky_ready(&fixture, &second_character);
    let first_starting_coins = coins(&first);
    let second_starting_coins = coins(&second);

    // Each offer is that character's own.
    now += 10;
    offered_card(&mut first, &first_character, FIZZLESTICKS, STICKY, now);
    assert_eq!(pending_offer(&first), Some(STICKY));
    assert_eq!(pending_offer(&second), None);
    now += 10;
    accept(&mut first, &first_character, STICKY, now);
    now += 10;
    offered_card(&mut second, &second_character, FIZZLESTICKS, STICKY, now);
    assert_eq!(pending_offer(&second), Some(STICKY));
    assert_eq!(pending_offer(&first), None);
    now += 10;
    accept(&mut second, &second_character, STICKY, now);

    // The first player's Nurble Talk never touches the second player's state.
    now += 10;
    assert!(talk(&mut first, &first_at_nurble, NURBLE, now).changed);
    assert_eq!(counts(&first, STICKY), vec![1, 0]);
    assert_eq!(counts(&second, STICKY), vec![0, 0]);
    assert_eq!(pending_turnin(&second), None);

    now += 10;
    assert!(talk(&mut second, &second_at_nurble, NURBLE, now).changed);
    assert_eq!(counts(&second, STICKY), vec![1, 0]);
    now += 10;
    assert!(talk(&mut first, &first_character, FIZZLESTICKS, now).changed);
    assert_eq!(pending_turnin(&first), Some(STICKY));
    assert_eq!(pending_turnin(&second), None);
    assert_eq!(counts(&second, STICKY), vec![1, 0]);

    // Both turn in on their own ledgers.
    now += 10;
    release_wizard_end_screen(&mut first, &first_character, STICKY, STICKY_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut first,
        &first_character,
        STICKY,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&first), first_starting_coins + REWARD_COINS);
    assert_eq!(
        coins(&second),
        second_starting_coins,
        "the first payout is not shared"
    );

    now += 10;
    assert!(talk(&mut second, &second_character, FIZZLESTICKS, now).changed);
    now += 10;
    release_wizard_end_screen(&mut second, &second_character, STICKY, STICKY_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut second,
        &second_character,
        STICKY,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&second), second_starting_coins + REWARD_COINS);
    assert_eq!(coins(&first), first_starting_coins + REWARD_COINS);
    for game in [&first, &second] {
        assert_eq!(quest_status(game, STICKY), QuestStatus::Completed);
        assert_eq!(pending_turnin(game), None);
    }
}

#[test]
fn an_active_movie_or_encounter_refuses_the_talk_before_the_offer() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let (game, now) = sticky_ready(&fixture, &character);

    // One live dungeon encounter; the production `begin` is the same call
    // `tests/type8_gateway.rs` makes. The shared busy table treats it as a start gate.
    let mut state = game.state().clone();
    let mut encounter = EncounterEngine::load(fixture.path(), None).expect("encounter content");
    encounter
        .begin(ENCOUNTER_ID, character.clone(), WIZARD)
        .expect("the fixture encounter begins");
    assert!(encounter.active());
    state.encounter = Some(encounter.state().clone());
    let mut game = fixture.game_as(character.guid, Some(state));

    // The offer is otherwise reachable: the same Talk without the encounter would offer it
    // in `nurble_then_fizzlesticks_completes_and_pays_once`. The busy refusal changes nothing.
    let before = saved(&game);
    let refused = send(&mut game, &character, &npc_interact(FIZZLESTICKS), now + 1);
    assert!(!refused.changed, "an active encounter refuses the Talk");
    assert!(
        !refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "no NPC menu opens during the encounter"
    );
    assert!(
        refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[15, 0, 3, 0])),
        "the busy refusal is a chat message"
    );
    assert_eq!(saved(&game), before, "the busy refusal changed the save");
    assert_eq!(pending_offer(&game), None);
    assert!(!has_quest(&game, STICKY));
}

#[test]
fn the_reward_grants_no_item_and_is_paid_once() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let (mut game, mut now) = sticky_ready(&fixture, &character);
    let items_before = game.state().inventory.items.clone();
    let equipment_before = game.state().inventory.equipment.clone();
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);
    accept_sticky(&mut game, &character, &mut now);
    now += 10;
    assert!(talk(&mut game, &avatar_at(NURBLE), NURBLE, now).changed);
    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);

    // The 49/13 reward bundle's display row count is zero: coins and stars only.
    now += 10;
    let end = release_wizard_end_screen(&mut game, &character, STICKY, STICKY_END_TEXT, now);
    assert_eq!(
        i32::from_le_bytes(end[87..91].try_into().unwrap()),
        0,
        "the reward bundle has no item rows"
    );
    assert_eq!(
        game.state().inventory.items,
        items_before,
        "the held end screen grants no item"
    );
    assert_eq!(
        game.state().inventory.equipment,
        equipment_before,
        "the held end screen changes no equipment"
    );
    now += 10;
    finish_quest(
        &mut game,
        &character,
        STICKY,
        WIZARD,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(
        game.state().inventory.items,
        items_before,
        "the turn-in grants no item"
    );
    assert_eq!(
        game.state().inventory.equipment,
        equipment_before,
        "the turn-in changes no equipment"
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), xp_before + REWARD_XP);

    // The only grant is the content row, once; a replay adds nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(STICKY),
        now + 1,
        "replayed turn-in for the reward",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), xp_before + REWARD_XP);
    assert_eq!(game.state().inventory.items, items_before);
    assert_eq!(game.state().inventory.equipment, equipment_before);
}

#[test]
fn the_quest_text_and_targets_resolve() {
    let wizard = read_json(&content_dir().join("quests/wizard.json"));
    let records = wizard["records"].as_array().expect("wizard records");
    let sticky = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(STICKY)))
        .expect("the Sticky Spell Book record");
    assert_eq!(sticky["enabled"].as_bool(), Some(true));
    assert_eq!(sticky["job"].as_i64(), Some(i64::from(WIZARD)));
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
        sticky["giver_npc_id"].as_u64(),
        Some(FIZZLESTICKS),
        "the giver is Fizzlesticks"
    );
    assert_eq!(
        sticky["prerequisite_quest_ids"],
        serde_json::json!([GLASSES])
    );
    assert!(
        sticky.get("turnin_npc_id").map_or(true, Value::is_null),
        "the turn-in defaults to the giver, never Nurble"
    );
    assert_provenance(sticky, "the Sticky Spell Book record");

    let objectives = sticky["objectives"].as_array().expect("objectives");
    assert_eq!(objectives.len(), 2, "the delivery and the return");
    for (goal, (npc_id, objective_id)) in objectives
        .iter()
        .zip([(NURBLE, 1_u64), (FIZZLESTICKS, 2_u64)])
    {
        assert_eq!(goal["kind"], "talk_to_npc");
        assert_eq!(goal["id"].as_u64(), Some(objective_id));
        assert_eq!(goal["npc_id"].as_u64(), Some(npc_id));
        assert_provenance(goal, "a Sticky Spell Book objective");
    }

    let rewards = sticky["rewards"].as_array().expect("rewards");
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"].as_u64(), Some(u64::from(REWARD_COINS)));
    assert_eq!(
        rewards[0]["job_xp"]["12"].as_u64(),
        Some(u64::from(REWARD_XP))
    );
    assert_eq!(rewards[0]["items"], serde_json::json!([]));
    assert_provenance(&rewards[0], "the Sticky Spell Book reward");

    // Both actors are drawable `legacy_data` rows at finite Robgoblin Camp positions.
    let npcs = read_json(&content_dir().join("npcs/wilds.json"));
    let regional = |npc_id: u64| -> &Value {
        let rows: Vec<&Value> = npcs["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["npc_id"].as_u64() == Some(npc_id))
            .collect();
        assert_eq!(rows.len(), 1, "NPC {npc_id} has exactly one row");
        rows[0]
    };
    let fizzlesticks = regional(FIZZLESTICKS);
    let nurble = regional(NURBLE);
    assert_eq!(fizzlesticks["placement"].as_str(), Some("legacy_data"));
    assert_eq!(nurble["placement"].as_str(), Some("legacy_data"));
    assert_eq!(fizzlesticks["model_id"].as_u64(), Some(191));
    assert_eq!(nurble["model_id"].as_u64(), Some(189));
    assert_eq!(fizzlesticks["name_string_id"].as_u64(), Some(20_517));
    assert_eq!(nurble["name_string_id"].as_u64(), Some(20_518));
    for row in [fizzlesticks, nurble] {
        assert!(
            row["roles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|role| role == "quest_giver"),
            "the actor is a drawable quest giver"
        );
    }
    let fizzlesticks_position: [f32; 3] =
        serde_json::from_value(fizzlesticks["position"].clone()).unwrap();
    let nurble_position: [f32; 3] = serde_json::from_value(nurble["position"].clone()).unwrap();
    assert_eq!(fizzlesticks_position, [-468.6923, 9.669079, 763.1784]);
    assert_eq!(nurble_position, [-475.6702, 9.8823, 781.8755]);
    assert!(fizzlesticks_position.iter().all(|value| value.is_finite()));
    assert!(nurble_position.iter().all(|value| value.is_finite()));
    assert_eq!(
        objectives[0]["npc_id"].as_u64(),
        Some(NURBLE),
        "the first objective targets the drawable Nurble row"
    );
    assert_eq!(
        objectives[1]["npc_id"].as_u64(),
        Some(FIZZLESTICKS),
        "the return objective targets the drawable Fizzlesticks row"
    );

    // `NpcVisibility::load` spawns both `legacy_data` rows into the runtime open-world zone 0
    // (the row's zone 16 is a regional content label), and the runtime actor id is the key the
    // quest binds.
    let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let packets = visibility
        .update(OPEN_WORLD, fizzlesticks_position)
        .expect("the open-world visibility update");
    let mut actors = Vec::new();
    for packet in &packets {
        if let Ok(npc) = parse_add_npc(packet) {
            actors.push(npc.actor_id);
        }
    }
    assert!(actors.contains(&NURBLE), "Nurble is drawn near the camp");
    assert!(
        actors.contains(&FIZZLESTICKS),
        "Fizzlesticks is drawn near the camp"
    );
}
