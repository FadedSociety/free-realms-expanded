//! F236/T197: the Medic entry chapter — So You Want to be a Medic? (3090001),
//! House Call (3090003) and Take Your Vitamins (3090005).
//!
//! The chapter is content-only; these tests drive the production in-world router
//! (`Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `QuestEngine`), the same
//! path the gateway calls for every in-world request, instead of engine internals.
//! Medic is learned through Nurse Naia's ordinary trainer menu and activated with
//! an ordinary 26/13 selection.
//!
//! Client request/reply layouts used here (verified in the client):
//! - 26/8 NPC interact: constructor `008b6bc0`; u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; u64 target + i32 choice.
//! - 26/6 dialogue response: constructor `009f5890`, sender `009f6740`; i32 row
//!   (301 is the quest row).
//! - 49/2 quest reply: constructor `00a86400`, sender `00a97d70`; u32 quest + u8 accept.
//! - 49/14 end reply: constructor `00a864c0`, sender `00a97ef0`; u32 quest.
//! 49/1 offers are read for their quest id at 6..10 and the intro locale id at
//! 10..14 (`QuestOffer` reader `00c7bb60`); 49/13 end screens name the speaker at
//! 6..14, the quest at 14..18 and the end-text locale id at 18..22
//! (`QuestEndPacket::ReadFields` `00c7b990`, resolved by `00984960` in
//! `ShowQuestEndScreen` `00a95420`).
//!
//! Giver reachability matters here: Naia (9000000161) and Greyli Dustywings
//! (9500000488) are legacy_data spawns. Doctor Gordon (9000000073) was an
//! unanchored row that `NpcVisibility` never spawned, so T197 corrected it
//! to its Legacy built-content spawn (FabledRealms.lua:1083, Npcs.json Id=2339);
//! the content test pins that.
//!
//! The final Talk of House Call and Take Your Vitamins is at Greyli. The engine
//! opens the 49/13 at the actor whose Talk completed the last objective
//! (`QuestEngine::on_event`, named-objective-target rule), so the 49/14 is sent
//! from there; the client text has no return-to-the-giver step for either quest.
//! This is the same recovered rule the T194 Archer Oath chapter uses at Hilforn.
//!
//! The shared fixture and packet helpers live in `tests/support/npc_quest_chain.rs`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

#[path = "support/medic_entry.rs"]
mod medic_entry;

use medic_entry::*;
use npc_quest_chain::*;
use osfr_tools::{
    character_store::CharacterState,
    game_store,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};

fn npc_character(npc_id: u64) -> CharacterState {
    npc_character_for(CHARACTER, npc_id)
}

fn npc_character_for(guid: u64, npc_id: u64) -> CharacterState {
    character_at_for(guid, SANCTUARY, content_position("sanctuary.json", npc_id))
}

fn send_error(
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = saved(game);
    let refused = game.handle_in_world(packet, character, now);
    assert!(
        refused
            .as_ref()
            .map_or(true, |out| out.replies.is_empty() && !out.changed),
        "{label}: the request must be refused"
    );
    assert_eq!(saved(game), before, "{label}: a refusal changed the save");
}

/// Wire dword 1 of a 49/1 offer: the quest-start intro locale id.
fn offer_intro_text_id(outcome: &Outcome, quest_id: u32) -> u32 {
    let offer = outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(quest_id)))
        .expect("49/1 offer");
    u32::from_le_bytes(offer[10..14].try_into().unwrap())
}

/// Wire dword 3 of a 49/13 end screen: the NPC end-text locale id.
fn end_text_id(packet: &[u8]) -> u32 {
    u32::from_le_bytes(packet[18..22].try_into().unwrap())
}

/// Every offer in this chapter currently carries intro 0: the T-1 locale slot is
/// empty and no unambiguous intro line is recovered. That is a documented lead
/// native-gap check, not an unasserted default.
fn offer_checked(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    quest_id: u32,
    now: u64,
) -> Outcome {
    let offered = offer(game, character, target, quest_id, now);
    assert_eq!(
        offer_intro_text_id(&offered, quest_id),
        0,
        "quest {quest_id}: the 49/1 intro is still the documented lead gap"
    );
    offered
}

fn learn_medic(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    let active_before = active_job(game);
    let menu = interact(game, character, NAIA, *now);
    let learn = menu_option(&menu, LEARN_LABEL);
    *now += 10;
    let learned = send(game, character, &npc_select(NAIA, learn), *now);
    assert!(learned.changed, "Naia's Learn choice must commit");
    assert!(
        game.state()
            .profiles
            .as_ref()
            .unwrap()
            .unlocked
            .contains(&MEDIC),
        "Naia's trainer row must unlock Medic"
    );
    assert_eq!(
        active_job(game),
        active_before,
        "learning a job must not switch to it"
    );
    *now += 10;
    let switched = send(game, character, &select_job(MEDIC), *now);
    assert!(
        switched.changed,
        "Medic selection must activate; its starter outfit has Chest and Legs"
    );
    assert_eq!(active_job(game), MEDIC, "the job gate needs Medic active");
}

fn start_exam(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    offer_checked(game, character, NAIA, EXAM, *now);
    *now += 10;
    accept(game, character, EXAM, *now);
}

fn complete_exam(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    start_exam(game, character, now);
    assert_eq!(medic_xp(game), 0);
    *now += 10;
    let talked = talk(game, character, NAIA, *now);
    assert!(talked.changed, "the exam talk must complete the objective");
    assert_eq!(counts(game, EXAM), vec![1]);
    assert_eq!(pending_turnin(game), Some(EXAM));
    assert!(
        !talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );
    *now += 10;
    release_end_screen(game, character, EXAM, NAIA, EXAM_END_TEXT, *now);
    *now += 10;
    finish_quest(game, character, EXAM, MEDIC, EXAM_COINS, EXAM_XP, *now);
}

fn complete_house_call(game: &mut Gameplay, now: &mut u64) {
    let naia = npc_character(NAIA);
    let greyli = npc_character(GREYLI);
    *now += 10;
    offer_checked(game, &naia, NAIA, HOUSE_CALL, *now);
    *now += 10;
    accept(game, &naia, HOUSE_CALL, *now);
    assert_eq!(medic_xp(game), EXAM_XP);

    *now += 10;
    let talked = talk(game, &greyli, GREYLI, *now);
    assert!(
        talked.changed,
        "the House Call talk must complete the objective"
    );
    assert_eq!(counts(game, HOUSE_CALL), vec![1]);
    assert_eq!(pending_turnin(game), Some(HOUSE_CALL));
    *now += 10;
    release_end_screen(game, &greyli, HOUSE_CALL, GREYLI, HOUSE_END_TEXT, *now);
    *now += 10;
    finish_quest(
        game,
        &greyli,
        HOUSE_CALL,
        MEDIC,
        HOUSE_COINS,
        HOUSE_XP,
        *now,
    );
}

fn complete_vitamins(game: &mut Gameplay, now: &mut u64) {
    let gordon = npc_character(GORDON);
    let greyli = npc_character(GREYLI);
    // T278 enables Palatable Pills (3090004), which Gordon offers before the
    // vitamins. Take that branch out of the offer queue; it stays active so this
    // helper does not pay its reward.
    *now += 10;
    offer(game, &gordon, GORDON, PILLS, *now);
    *now += 10;
    accept(game, &gordon, PILLS, *now);
    *now += 10;
    offer_checked(game, &gordon, GORDON, VITAMINS, *now);
    *now += 10;
    accept(game, &gordon, VITAMINS, *now);
    assert_eq!(medic_xp(game), EXAM_XP + HOUSE_XP);

    *now += 10;
    let talked = talk(game, &greyli, GREYLI, *now);
    assert!(
        talked.changed,
        "the vitamins talk must complete the objective"
    );
    assert_eq!(counts(game, VITAMINS), vec![1]);
    assert_eq!(pending_turnin(game), Some(VITAMINS));
    *now += 10;
    // No completion line is recovered for 3090005; the guide supplies none and
    // the locale block only has Greyli's in-dialogue thank-you. Keep 0 and mark
    // the gap rather than invent an end-screen id.
    release_end_screen(game, &greyli, VITAMINS, GREYLI, 0, *now);
    *now += 10;
    finish_quest(
        game,
        &greyli,
        VITAMINS,
        MEDIC,
        VITAMIN_COINS,
        VITAMIN_XP,
        *now,
    );
}

fn medic_xp(game: &Gameplay) -> u32 {
    profile_xp(game, MEDIC)
}

#[test]
fn naia_offers_the_medic_exam_only_to_medics() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 1_000;
    let naia = npc_character(NAIA);

    // Brawler active: the job 11 gate must hide the exam.
    assert_ne!(active_job(&game), BRAWLER);
    let switched = send(&mut game, &naia, &select_job(BRAWLER), now);
    assert!(switched.changed, "Brawler must be selectable");
    assert_eq!(active_job(&game), BRAWLER);
    assert_no_offer(&mut game, &naia, NAIA, &mut now, "Brawler at Naia");
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&EXAM),
        "a Brawler must not receive the Medic exam"
    );

    // A forged 49/2 accept for an offer the client never received changes nothing.
    let forged = send(&mut game, &naia, &quest_reply(EXAM, true), now + 1);
    assert!(!forged.changed);
    assert!(
        forged
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 3, Some(EXAM)))
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&EXAM)
    );

    // The ordinary NPC menu teaches and activates Medic.
    now += 10;
    learn_medic(&mut game, &naia, &mut now);
    offer_checked(&mut game, &naia, NAIA, EXAM, now + 10);

    // House Call stays behind the exam prerequisite.
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&HOUSE_CALL),
        "House Call must wait for the exam"
    );
}

#[test]
fn exam_and_house_call_complete_in_order() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 2_000;
    let naia = npc_character(NAIA);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);
    let coins_before = coins(&game);

    start_exam(&mut game, &naia, &mut now);
    assert_eq!(counts(&game, EXAM), vec![0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&HOUSE_CALL),
        "House Call must not be offered while the exam runs"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");

    // The exam talk then turn-in.
    now += 10;
    let talked = talk(&mut game, &naia, NAIA, now);
    assert!(talked.changed);
    assert_eq!(counts(&game, EXAM), vec![1]);
    assert_eq!(pending_turnin(&game), Some(EXAM));
    assert_eq!(coins(&game), coins_before);
    now += 10;
    release_end_screen(&mut game, &naia, EXAM, NAIA, EXAM_END_TEXT, now);
    now += 10;
    finish_quest(&mut game, &naia, EXAM, MEDIC, EXAM_COINS, EXAM_XP, now);
    assert_eq!(coins(&game), coins_before + EXAM_COINS);
    assert_eq!(medic_xp(&game), EXAM_XP);

    // House Call becomes offered; the objective is at Greyli.
    now += 10;
    offer_checked(&mut game, &naia, NAIA, HOUSE_CALL, now);
    now += 10;
    accept(&mut game, &naia, HOUSE_CALL, now);
    now += 10;
    let visited = talk(&mut game, &greyli, GREYLI, now);
    assert!(visited.changed);
    assert_eq!(counts(&game, HOUSE_CALL), vec![1]);
    assert_eq!(pending_turnin(&game), Some(HOUSE_CALL));
    assert_eq!(coins(&game), coins_before + EXAM_COINS);
    now += 10;
    release_end_screen(&mut game, &greyli, HOUSE_CALL, GREYLI, HOUSE_END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &greyli,
        HOUSE_CALL,
        MEDIC,
        HOUSE_COINS,
        HOUSE_XP,
        now,
    );

    assert_eq!(coins(&game), coins_before + EXAM_COINS + HOUSE_COINS);
    assert_eq!(medic_xp(&game), EXAM_XP + HOUSE_XP);

    // A duplicated 49/14 pays nothing.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &greyli,
        &quest_end_reply(HOUSE_CALL),
        now,
        "duplicate House Call turn-in",
    );
    assert_eq!(coins(&game), coins_before + EXAM_COINS + HOUSE_COINS);
    assert_eq!(medic_xp(&game), EXAM_XP + HOUSE_XP);
}

#[test]
fn take_your_vitamins_requires_house_call() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 3_000;
    let naia = npc_character(NAIA);
    let gordon = npc_character(GORDON);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);
    let coins_before = coins(&game);

    complete_exam(&mut game, &naia, &mut now);
    assert_eq!(coins(&game), coins_before + EXAM_COINS);

    // The prerequisite is missing: Gordon offers nothing.
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "Take Your Vitamins before House Call",
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&VITAMINS)
    );

    complete_house_call(&mut game, &mut now);
    let coins_after_house = coins(&game);
    let xp_after_house = medic_xp(&game);
    assert_eq!(coins_after_house, coins_before + EXAM_COINS + HOUSE_COINS);

    // Naia is not the vitamins giver: her menu offers nothing for the chapter.
    assert_no_offer(
        &mut game,
        &naia,
        NAIA,
        &mut now,
        "Take Your Vitamins at the wrong giver",
    );

    // T278 enables 3090004, offered first at Gordon; accepting it exposes the
    // vitamins branch without paying the pills reward.
    now += 10;
    offer(&mut game, &gordon, GORDON, PILLS, now);
    now += 10;
    accept(&mut game, &gordon, PILLS, now);

    // The completed House Call unlocks the vitamins at Doctor Gordon.
    now += 10;
    offer_checked(&mut game, &gordon, GORDON, VITAMINS, now);
    now += 10;
    accept(&mut game, &gordon, VITAMINS, now);

    // The client's only recovered step is the talk at Greyli; that Talk opens the
    // end screen there and the 49/14 settles the Gordon-issued quest.
    now += 10;
    let talked = talk(&mut game, &greyli, GREYLI, now);
    assert!(talked.changed);
    assert_eq!(counts(&game, VITAMINS), vec![1]);
    assert_eq!(pending_turnin(&game), Some(VITAMINS));
    assert_eq!(
        coins(&game),
        coins_after_house,
        "no reward before the turn-in"
    );
    now += 10;
    release_end_screen(&mut game, &greyli, VITAMINS, GREYLI, 0, now);
    now += 10;
    finish_quest(
        &mut game,
        &greyli,
        VITAMINS,
        MEDIC,
        VITAMIN_COINS,
        VITAMIN_XP,
        now,
    );

    assert_eq!(
        coins(&game),
        coins_before + EXAM_COINS + HOUSE_COINS + VITAMIN_COINS
    );
    assert_eq!(medic_xp(&game), xp_after_house + VITAMIN_XP);

    // Paid exactly once: a replayed 49/14 changes nothing.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &greyli,
        &quest_end_reply(VITAMINS),
        now,
        "duplicate Take Your Vitamins turn-in",
    );
    assert_eq!(
        coins(&game),
        coins_before + EXAM_COINS + HOUSE_COINS + VITAMIN_COINS
    );
    assert_eq!(medic_xp(&game), xp_after_house + VITAMIN_XP);
}

#[test]
fn medic_xp_goes_to_medic_not_the_active_job() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 4_000;
    let naia = npc_character(NAIA);
    let gordon = npc_character(GORDON);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);
    complete_exam(&mut game, &naia, &mut now);
    complete_house_call(&mut game, &mut now);
    let medic_before = medic_xp(&game);
    assert_eq!(medic_before, EXAM_XP + HOUSE_XP);

    // T278 enables 3090004, offered first at Gordon; accepting it exposes the
    // vitamins branch without paying the pills reward.
    now += 10;
    offer(&mut game, &gordon, GORDON, PILLS, now);
    now += 10;
    accept(&mut game, &gordon, PILLS, now);

    now += 10;
    offer_checked(&mut game, &gordon, GORDON, VITAMINS, now);
    now += 10;
    accept(&mut game, &gordon, VITAMINS, now);

    // Switch away before the turn-in: the accepted quest keeps its own reward map.
    now += 10;
    let brawler_xp_before = profile_xp(&game, BRAWLER);
    let switched = send(&mut game, &greyli, &select_job(BRAWLER), now);
    assert!(switched.changed, "Brawler must be selectable again");
    assert_eq!(active_job(&game), BRAWLER);

    now += 10;
    let talked = talk(&mut game, &greyli, GREYLI, now);
    assert!(talked.changed);
    now += 10;
    release_end_screen(&mut game, &greyli, VITAMINS, GREYLI, 0, now);
    now += 10;
    finish_quest(
        &mut game,
        &greyli,
        VITAMINS,
        MEDIC,
        VITAMIN_COINS,
        VITAMIN_XP,
        now,
    );

    assert_eq!(
        active_job(&game),
        BRAWLER,
        "the turn-in must not switch jobs"
    );
    assert_eq!(
        medic_xp(&game),
        medic_before + VITAMIN_XP,
        "profile 11 must receive the Medic XP"
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_xp_before,
        "the active job must not receive the Medic XP"
    );
}

#[test]
fn refusals_and_replay_change_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 5_000;
    let naia = npc_character(NAIA);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);

    // Wrong talk target: Greyli cannot advance the Naia exam objective.
    start_exam(&mut game, &naia, &mut now);
    let before = saved(&game);
    let wrong_npc = talk(&mut game, &greyli, GREYLI, now + 1);
    assert!(!wrong_npc.changed, "a wrong NPC must not advance the exam");
    assert_eq!(counts(&game, EXAM), vec![0]);
    assert_eq!(saved(&game), before, "the wrong NPC changed the save");

    // Forged accept for a quest offered by another giver changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &naia,
        &quest_reply(HOUSE_CALL, true),
        now + 2,
        "forged House Call accept",
    );
    assert_packet_changes_nothing(
        &mut game,
        &naia,
        &quest_reply(VITAMINS, true),
        now + 3,
        "forged Take Your Vitamins accept",
    );

    // Out of range: the menu and a formerly offered choice are both refused.
    let far = character_at(
        SANCTUARY,
        [naia.position[0] + 400.0, naia.position[1], naia.position[2]],
    );
    send_error(
        &mut game,
        &far,
        &npc_interact(NAIA),
        now + 4,
        "far interact",
    );
    now += 10;
    let menu = interact(&mut game, &naia, NAIA, now);
    let talk_choice = menu_option(&menu, TALK_LABEL);
    let offered = send(&mut game, &naia, &npc_select(NAIA, talk_choice), now + 1);
    assert!(
        offered.changed,
        "the exam objective is completed by the Naia talk"
    );
    assert_eq!(pending_turnin(&game), Some(EXAM));
    let held_bytes = saved(&game);
    send_error(
        &mut game,
        &far,
        &npc_select(NAIA, talk_choice),
        now + 2,
        "far menu select",
    );
    assert_eq!(
        saved(&game),
        held_bytes,
        "the refused choice changed the save"
    );

    // A replayed Talk at the same actor only re-sends the end screen.
    let replayed = send(&mut game, &naia, &npc_select(NAIA, talk_choice), now + 3);
    assert!(!replayed.changed, "a replayed Talk changed gameplay");
    assert_eq!(saved(&game), held_bytes, "a replayed Talk changed the save");
    assert_eq!(pending_turnin(&game), Some(EXAM));
    assert_eq!(counts(&game, EXAM), vec![1]);

    // Release and finish, then refuse every old packet again.
    now += 10;
    release_end_screen(&mut game, &naia, EXAM, NAIA, EXAM_END_TEXT, now);
    now += 10;
    finish_quest(&mut game, &naia, EXAM, MEDIC, EXAM_COINS, EXAM_XP, now);
    let finished = saved(&game);
    let finished_coins = coins(&game);
    let finished_xp = medic_xp(&game);
    for (label, packet) in [
        ("replayed exam end reply", quest_end_reply(EXAM)),
        ("replayed exam accept", quest_reply(EXAM, true)),
    ] {
        let outcome = send(&mut game, &naia, &packet, now + 10);
        assert!(!outcome.changed, "{label} changed gameplay");
        assert!(
            !outcome
                .replies
                .iter()
                .any(|reply| has_family(reply, 49, 3, None) || has_family(reply, 49, 4, None)),
            "{label} added or completed a quest"
        );
    }
    assert_eq!(saved(&game), finished, "old packets changed the save");
    assert_eq!(coins(&game), finished_coins);
    assert_eq!(medic_xp(&game), finished_xp);

    // A Talk at an unrelated NPC offers nothing and changes nothing.
    assert_no_offer(
        &mut game,
        &greyli,
        GREYLI,
        &mut now,
        "exam after completion",
    );

    // Naia's next chapter step is offered; the completed exam never is.
    now += 10;
    let menu = interact(&mut game, &naia, NAIA, now);
    let choice = menu_option(&menu, TALK_LABEL);
    let dialogue = send(&mut game, &naia, &npc_select(NAIA, choice), now + 1);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Naia must open a dialogue for the next chapter step"
    );
    let offered = send(&mut game, &naia, &dialogue_choice(QUEST_CHOICE), now + 2);
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(EXAM))),
        "the completed exam must not be offered again"
    );
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(HOUSE_CALL))),
        "House Call must follow the completed exam"
    );
}

/// A real reconnect goes through the `CharacterStore`/`game_store` envelope, not
/// an in-memory clone: a save-envelope change that drops `npc_quests` or the held
/// turn-in must fail here. The first reload happens with House Call active and
/// its 49/13 held for the 26/6 choice; the second happens after the whole chapter.
#[test]
fn relog_preserves_completed_history() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 6_000;
    let naia = npc_character(NAIA);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);
    complete_exam(&mut game, &naia, &mut now);

    // Start House Call and disconnect while the 49/13 is held.
    now += 10;
    offer_checked(&mut game, &naia, NAIA, HOUSE_CALL, now);
    now += 10;
    accept(&mut game, &naia, HOUSE_CALL, now);
    now += 10;
    let talked = talk(&mut game, &greyli, GREYLI, now);
    assert!(talked.changed);
    assert_eq!(pending_turnin(&game), Some(HOUSE_CALL));

    let save_path = fixture.path().join("gameplay-save.json");
    let character = naia.clone();
    let held = game.state();
    let snapshot = game_store::Snapshot {
        character: character.clone(),
        gameplay: Some(held.clone()),
        pets: None,
    };
    game_store::save(&save_path, &snapshot).unwrap();
    let loaded = game_store::load_or_create(&save_path, character).unwrap();
    assert_eq!(
        loaded.gameplay.as_ref(),
        Some(&held),
        "the save envelope changed the held turn-in state"
    );
    game = fixture.game(loaded.gameplay);
    assert_eq!(quest_status(&game, HOUSE_CALL), QuestStatus::Active);
    assert_eq!(counts(&game, HOUSE_CALL), vec![1]);
    assert_eq!(pending_turnin(&game), Some(HOUSE_CALL));
    assert_eq!(
        game.state()
            .npc_quests
            .as_ref()
            .unwrap()
            .pending_turnin_actor,
        Some(GREYLI),
        "the held end-screen speaker survives the envelope"
    );

    // Resume: the reconnect visibility batch re-sends the held 49/13 with the
    // speaker's 35/2, then the 49/14 settles the saved pending turn-in. The open
    // dialogue itself is session state and is correctly gone after a reconnect.
    let visibility_character = character_at(0, content_position("sanctuary.json", GREYLI));
    let packets = game
        .update_npc_visibility(&visibility_character)
        .expect("relog NPC visibility");
    let end = packets
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(HOUSE_CALL)))
        .expect("the held end screen is re-sent after relog");
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        GREYLI,
        "the re-sent end screen must name Greyli"
    );
    assert_eq!(
        end_text_id(end),
        HOUSE_END_TEXT,
        "the re-sent end screen keeps its recovered completion text"
    );
    now += 10;
    finish_quest(
        &mut game,
        &visibility_character,
        HOUSE_CALL,
        MEDIC,
        HOUSE_COINS,
        HOUSE_XP,
        now,
    );
    complete_vitamins(&mut game, &mut now);

    let final_bytes = saved(&game);
    let final_coins = coins(&game);
    let final_xp = medic_xp(&game);
    assert_eq!(
        final_coins,
        EXAM_COINS + HOUSE_COINS + VITAMIN_COINS,
        "the three chapter rewards"
    );
    assert_eq!(final_xp, EXAM_XP + HOUSE_XP + VITAMIN_XP);

    // Reconnect with the finished chapter: history and balances survive.
    let expected = game.state();
    let snapshot = game_store::Snapshot {
        character: naia.clone(),
        gameplay: Some(expected.clone()),
        pets: None,
    };
    game_store::save(&save_path, &snapshot).unwrap();
    let loaded = game_store::load_or_create(&save_path, naia.clone()).unwrap();
    assert_eq!(loaded.gameplay.as_ref(), Some(&expected));
    game = fixture.game(loaded.gameplay);
    assert_eq!(saved(&game), final_bytes, "reload changed the save bytes");
    for quest_id in [EXAM, HOUSE_CALL, VITAMINS] {
        assert_eq!(
            quest_status(&game, quest_id),
            QuestStatus::Completed,
            "quest {quest_id} history"
        );
    }
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), final_coins);
    assert_eq!(medic_xp(&game), final_xp);

    // None is offered again.
    assert_no_offer(
        &mut game,
        &naia,
        NAIA,
        &mut now,
        "completed exam after relog",
    );
    assert_no_offer(
        &mut game,
        &npc_character(GORDON),
        GORDON,
        &mut now,
        "completed vitamins after relog",
    );

    // Old packets pay nothing after the reload.
    for quest_id in [EXAM, HOUSE_CALL, VITAMINS] {
        let greyli = npc_character(GREYLI);
        assert_packet_changes_nothing(
            &mut game,
            &greyli,
            &quest_end_reply(quest_id),
            now + 1,
            "replayed turn-in after relog",
        );
        assert_packet_changes_nothing(
            &mut game,
            &greyli,
            &quest_reply(quest_id, true),
            now + 2,
            "replayed accept after relog",
        );
        now += 10;
    }
    assert_eq!(coins(&game), final_coins);
    assert_eq!(medic_xp(&game), final_xp);
}

#[test]
fn two_medics_progress_independently() {
    let fixture = Fixture::new();
    let first_character = npc_character(NAIA);
    let second_character = npc_character_for(SECOND_CHARACTER, NAIA);
    let mut first = fixture.game(None);
    let mut second = fixture.game_as(SECOND_CHARACTER, None);
    let mut now = 7_000;
    learn_medic(&mut first, &first_character, &mut now);
    learn_medic(&mut second, &second_character, &mut now);
    start_exam(&mut first, &first_character, &mut now);
    start_exam(&mut second, &second_character, &mut now);

    for (game, character) in [
        (&mut first, &first_character),
        (&mut second, &second_character),
    ] {
        let talked = talk(game, character, NAIA, now);
        assert!(talked.changed);
        assert_eq!(counts(game, EXAM), vec![1]);
        assert_eq!(pending_turnin(game), Some(EXAM));
        release_end_screen(game, character, EXAM, NAIA, EXAM_END_TEXT, now + 10);
        finish_quest(game, character, EXAM, MEDIC, EXAM_COINS, EXAM_XP, now + 20);
    }
    assert_eq!(counts(&first, EXAM), vec![1]);
    assert_eq!(counts(&second, EXAM), vec![1]);
    assert_eq!(
        coins(&first),
        EXAM_COINS,
        "the first character's exam pays once"
    );
    assert_eq!(
        coins(&second),
        EXAM_COINS,
        "the second character's exam pays once"
    );
    assert_eq!(medic_xp(&first), EXAM_XP);
    assert_eq!(medic_xp(&second), EXAM_XP);
}

#[test]
fn enabled_medic_chapter_records_are_complete_and_others_stay_disabled() {
    assert_enabled_medic_chapter_records();
}
