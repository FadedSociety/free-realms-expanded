//! T545: the admin-only `realm duel <minigame_id>` credit proves the Card Duelist quest chain
//! before the card game produces a result itself.
//!
//! The command credits exactly one `FinishMinigame` event for a client TYPE 16 `MiniGameData`
//! row and pays nothing: the quest's own 49/14 turn-in pays the cap and the job XP. A Player or
//! Builder gets the existing role refusal (`developer_command_denied`), and every refusal leaves
//! the production character save byte-equal and sends no quest packet. The reconnect steps use
//! `Fixture::save_and_reload`, the `game_store` save envelope and load a login performs.
//!
//! Client evidence: `Resources/MiniGameData.txt` header `#ID^TYPE^...`; rows `504^16`, `507^16`
//! and `668^16` (Poe's Trick Deck, Poe's Tournament Deck, Esther's Friendly Deck), while `18^8`
//! (kart), `39^11` (derby) and `727^23` (Treasure War) are not card duels. The chain itself is
//! T505's: 3040011 targets 507 at Poe; 3040034 starts at Dominic, meets Esther at 9000000082 and
//! targets 668. Shipped content keeps both quests disabled.

#[path = "support/card_duelist.rs"]
mod card_duelist;
#[path = "common/item_catalogue.rs"]
mod item_catalogue;

use card_duelist::{
    ADVENTURER, CARD_DUELIST, CHARACTER, DOMINIC, ESTHER, ESTHER_THE_BRAINS, Fixture, POE,
    POES_BEST_DECK, accept, at, at_for, card_duelist_as, coins, content_dir, counts,
    end_screen_reward, finish_quest, fixture_with, objective_complete, offer, pending_turnin,
    profile_xp, quest_end_reply, quest_status, release_end_screen, said, seed_active_profile, send,
    talk,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};
use serde_json::Value;
use std::fs;

const POES_TRICK_DECK: u32 = 504;
const POES_TOURNAMENT_DECK: u32 = 507;
const ESTHERS_FRIENDLY_DECK: u32 = 668;
const DUELIST_CAP: i32 = 51_187;
const POES_BEST_DECK_END_TEXT: u32 = 103_746;
const ESTHER_THE_BRAINS_END_TEXT: u32 = 407_021;

/// The shared fixture plus the cap's catalogue row. As `quest_item_reward`: the opening
/// inventory lacks the farming rewards.
fn duel_fixture(enabled: &[u32]) -> Fixture {
    let fixture = fixture_with(enabled);
    item_catalogue::write(
        fixture.path(),
        &[item_catalogue::sized_row(DUELIST_CAP as u32, 64)],
    );
    fixture
}

/// The production account-store shape: the role, not `--dev-tools`, authorizes the command.
fn admin(game: &mut Gameplay) {
    game.set_role_commands(true);
    game.set_role(AccountRole::Admin);
}

fn duel(game: &mut Gameplay, who: &CharacterState, argument: &str, now: u64) -> Outcome {
    send(
        game,
        who,
        &chat::local_request(&format!("realm duel {argument}")),
        now,
    )
}

fn caps(game: &Gameplay) -> usize {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == DUELIST_CAP)
        .count()
}

#[test]
fn only_an_admin_can_credit_a_duel() {
    let fixture = duel_fixture(&[POES_BEST_DECK]);
    let mut game = card_duelist_as(&fixture, CHARACTER);
    let poe = at(POE);
    offer(&mut game, &poe, POE, POES_BEST_DECK, 2_000);
    accept(&mut game, &poe, POES_BEST_DECK, 2_010);
    game.set_role_commands(true);
    let before = fixture.saved_bytes(&game, &poe);

    for role in [AccountRole::Player, AccountRole::Builder] {
        game.set_role(role);
        let denied = duel(&mut game, &poe, "507", 3_000);
        assert!(
            !denied.changed,
            "{role:?}: a refused credit changed gameplay"
        );
        assert!(denied.dev_command_denied, "{role:?}: not the role refusal");
        assert!(
            said(&denied, "your account role does not allow this command"),
            "{role:?}: the existing role refusal must answer"
        );
        assert!(
            !said(&denied, "Credited card duel"),
            "{role:?}: claimed a credit"
        );
        assert_eq!(counts(&game, POES_BEST_DECK), [0], "{role:?}");
        assert_eq!(
            fixture.saved_bytes(&game, &poe),
            before,
            "{role:?}: a refusal changed the saved character"
        );
    }

    // The control: the same command and character succeed once the role is admin, so the
    // refusal above was the role and not the fixture.
    game.set_role(AccountRole::Admin);
    let credited = duel(&mut game, &poe, "507", 3_010);
    assert!(credited.changed, "the admin credit is the control");
    assert_eq!(counts(&game, POES_BEST_DECK), [1]);
}

#[test]
fn non_duel_ids_are_refused_and_change_nothing() {
    let fixture = duel_fixture(&[POES_BEST_DECK]);
    let mut game = card_duelist_as(&fixture, CHARACTER);
    let poe = at(POE);
    offer(&mut game, &poe, POE, POES_BEST_DECK, 2_000);
    accept(&mut game, &poe, POES_BEST_DECK, 2_010);
    admin(&mut game);
    let before = fixture.saved_bytes(&game, &poe);

    for (argument, refusal) in [
        ("0", "Usage: realm duel"),
        ("abc", "Usage: realm duel"),
        ("18", "not a client TYPE 16"),
        ("39", "not a client TYPE 16"),
        ("727", "not a client TYPE 16"),
        ("4294967295", "not a client TYPE 16"),
    ] {
        let refused = duel(&mut game, &poe, argument, 3_000);
        assert!(!refused.changed, "{argument}: a refusal changed gameplay");
        assert!(
            said(&refused, refusal),
            "{argument}: the refusal line must say why"
        );
        assert!(
            !said(&refused, "Credited card duel"),
            "{argument}: a refusal claimed a credit"
        );
        assert!(
            !refused
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0])),
            "{argument}: a refusal sent a quest packet"
        );
        assert_eq!(
            counts(&game, POES_BEST_DECK),
            [0],
            "{argument}: the objective moved"
        );
        assert_eq!(
            fixture.saved_bytes(&game, &poe),
            before,
            "{argument}: a refusal changed the saved character"
        );
    }

    // A missing or extra argument is a refusal too, not the realm fallback.
    for argument in ["", "507 1"] {
        let refused = duel(&mut game, &poe, argument, 3_100);
        assert!(!refused.changed, "{argument:?}");
        assert!(said(&refused, "Usage: realm duel"), "{argument:?}");
        assert_eq!(
            fixture.saved_bytes(&game, &poe),
            before,
            "{argument:?}: a refusal changed the saved character"
        );
    }
}

#[test]
fn poes_best_deck_pays_the_cap_and_card_duelist_xp_once() {
    let fixture = duel_fixture(&[POES_BEST_DECK]);
    let mut game = card_duelist_as(&fixture, CHARACTER);
    let poe = at(POE);
    let now = 2_000;
    offer(&mut game, &poe, POE, POES_BEST_DECK, now);
    accept(&mut game, &poe, POES_BEST_DECK, now + 10);
    admin(&mut game);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, CARD_DUELIST));
    assert_eq!(caps(&game), 0, "no cap before the turn-in");

    // Poe's Trick Deck is a TYPE 16 card duel, so the command credits it, but it is not the
    // objective's row: the accepted credit changes nothing.
    let before = fixture.saved_bytes(&game, &poe);
    let trick = duel(&mut game, &poe, &POES_TRICK_DECK.to_string(), now + 20);
    assert!(!trick.changed, "504 is not quest 3040011's target");
    assert!(said(&trick, "Credited card duel 504"));
    assert_eq!(counts(&game, POES_BEST_DECK), [0]);
    assert_eq!(
        fixture.saved_bytes(&game, &poe),
        before,
        "504 changed the saved character"
    );

    // The quest's own row completes its one objective; the credit pays nothing yet.
    let won = duel(&mut game, &poe, &POES_TOURNAMENT_DECK.to_string(), now + 30);
    assert!(won.changed, "507 completes the objective");
    assert!(objective_complete(&won, POES_BEST_DECK, 1));
    assert_eq!(counts(&game, POES_BEST_DECK), [1]);
    assert_eq!(quest_status(&game, POES_BEST_DECK), QuestStatus::Active);
    assert_eq!(coins(&game), coins_before);
    assert_eq!(profile_xp(&game, CARD_DUELIST), xp_before);
    assert_eq!(caps(&game), 0);

    // Reconnect between the credit and the turn-in through the production save envelope (not
    // a `GameplayState` proxy): the completed objective survives.
    let game = fixture.save_and_reload(&game, &poe);
    assert_eq!(counts(&game, POES_BEST_DECK), [1]);
    assert_eq!(quest_status(&game, POES_BEST_DECK), QuestStatus::Active);

    // Switch to another job before the turn-in: the record still pays profile 120 (T202).
    let mut state = game.state();
    seed_active_profile(&mut state, ADVENTURER, &[]);
    let mut game = fixture.game(Some(state));
    assert_eq!(
        game.state().profiles.as_ref().unwrap().active,
        ADVENTURER,
        "the switch is in the saved state"
    );
    assert_eq!(counts(&game, POES_BEST_DECK), [1]);

    let poe = at(POE);
    let talked = talk(&mut game, &poe, POE, now + 100);
    assert!(talked.changed, "the final Talk holds the end screen");
    assert_eq!(pending_turnin(&game), Some(POES_BEST_DECK));
    let end = release_end_screen(
        &mut game,
        &poe,
        POES_BEST_DECK,
        POE,
        POES_BEST_DECK_END_TEXT,
        now + 102,
    );
    assert_eq!(end_screen_reward(&end), (0, 300));
    let coins_at_turnin = coins(&game);
    finish_quest(
        &mut game,
        &poe,
        POES_BEST_DECK,
        CARD_DUELIST,
        0,
        300,
        now + 104,
    );
    assert_eq!(coins(&game), coins_at_turnin, "the record pays no coins");
    assert_eq!(profile_xp(&game, CARD_DUELIST), xp_before + 300);
    assert_eq!(caps(&game), 1, "exactly one Duelist Nature Cap");
    assert_eq!(
        game.state()
            .inventory
            .items
            .iter()
            .find(|item| item.definition_id == DUELIST_CAP)
            .map(|item| item.count),
        Some(1)
    );

    // A second credit and a replayed 49/14 pay nothing, and the completed quest stays completed.
    let paid = fixture.saved_bytes(&game, &poe);
    let again = duel(
        &mut game,
        &poe,
        &POES_TOURNAMENT_DECK.to_string(),
        now + 200,
    );
    assert!(
        !again.changed,
        "a completed objective is not credited twice"
    );
    assert_eq!(
        fixture.saved_bytes(&game, &poe),
        paid,
        "the second credit changed the saved character"
    );
    let replay = send(&mut game, &poe, &quest_end_reply(POES_BEST_DECK), now + 210);
    assert!(!replay.changed, "a replayed 49/14 is refused");
    assert_eq!(
        fixture.saved_bytes(&game, &poe),
        paid,
        "the replayed 49/14 changed the saved character"
    );
    assert_eq!(caps(&game), 1);
    assert_eq!(coins(&game), coins_at_turnin);
    assert_eq!(profile_xp(&game, CARD_DUELIST), xp_before + 300);

    // Save and reload keep the completed quest, the cap and the XP.
    let reloaded = fixture.save_and_reload(&game, &poe);
    assert_eq!(
        quest_status(&reloaded, POES_BEST_DECK),
        QuestStatus::Completed
    );
    assert_eq!(profile_xp(&reloaded, CARD_DUELIST), xp_before + 300);
    assert_eq!(caps(&reloaded), 1);
}

#[test]
fn esther_the_brains_runs_dominic_then_esther() {
    let fixture = duel_fixture(&[ESTHER_THE_BRAINS]);
    let mut game = card_duelist_as(&fixture, CHARACTER);
    let now = 2_000;
    let dominic = at(DOMINIC);
    offer(&mut game, &dominic, DOMINIC, ESTHER_THE_BRAINS, now);
    accept(&mut game, &dominic, ESTHER_THE_BRAINS, now + 10);
    admin(&mut game);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, CARD_DUELIST));

    // The quest is ordered: the duel does not advance it before Esther is met.
    let before = fixture.saved_bytes(&game, &dominic);
    let early = duel(
        &mut game,
        &dominic,
        &ESTHERS_FRIENDLY_DECK.to_string(),
        now + 20,
    );
    assert!(!early.changed, "668 before meeting Esther advances nothing");
    assert!(said(&early, "Credited card duel 668"));
    assert_eq!(counts(&game, ESTHER_THE_BRAINS), [0, 0]);
    assert_eq!(
        fixture.saved_bytes(&game, &dominic),
        before,
        "668 before Esther changed the saved character"
    );

    let esther = at(ESTHER);
    let met = talk(&mut game, &esther, ESTHER, now + 100);
    assert!(met.changed && objective_complete(&met, ESTHER_THE_BRAINS, 1));
    assert_eq!(counts(&game, ESTHER_THE_BRAINS), [1, 0]);

    let won = duel(
        &mut game,
        &esther,
        &ESTHERS_FRIENDLY_DECK.to_string(),
        now + 110,
    );
    assert!(won.changed && objective_complete(&won, ESTHER_THE_BRAINS, 2));
    assert_eq!(counts(&game, ESTHER_THE_BRAINS), [1, 1]);

    // Turn in at Esther: +3 coins and +100 profile-120 XP, no item.
    let items_before = game.state().inventory.items.clone();
    let talked = talk(&mut game, &esther, ESTHER, now + 120);
    assert!(talked.changed, "the final Talk holds the end screen");
    assert_eq!(pending_turnin(&game), Some(ESTHER_THE_BRAINS));
    let end = release_end_screen(
        &mut game,
        &esther,
        ESTHER_THE_BRAINS,
        ESTHER,
        ESTHER_THE_BRAINS_END_TEXT,
        now + 122,
    );
    assert_eq!(end_screen_reward(&end), (3, 100));
    finish_quest(
        &mut game,
        &esther,
        ESTHER_THE_BRAINS,
        CARD_DUELIST,
        3,
        100,
        now + 124,
    );
    assert_eq!(coins(&game), coins_before + 3);
    assert_eq!(profile_xp(&game, CARD_DUELIST), xp_before + 100);
    assert_eq!(
        game.state().inventory.items,
        items_before,
        "the record grants no item"
    );
}

#[test]
fn a_credit_touches_only_the_issuing_character() {
    const OTHER: u64 = CHARACTER + 1;
    let fixture = duel_fixture(&[POES_BEST_DECK]);
    let mut issuing = card_duelist_as(&fixture, CHARACTER);
    let mut other = card_duelist_as(&fixture, OTHER);
    let one = at(POE);
    let two = at_for(OTHER, POE);

    offer(&mut issuing, &one, POE, POES_BEST_DECK, 2_000);
    accept(&mut issuing, &one, POES_BEST_DECK, 2_010);
    offer(&mut other, &two, POE, POES_BEST_DECK, 2_000);
    accept(&mut other, &two, POES_BEST_DECK, 2_010);
    admin(&mut issuing);
    let other_before = fixture.saved_bytes(&other, &two);

    let credited = duel(&mut issuing, &one, &POES_TOURNAMENT_DECK.to_string(), 3_000);
    assert!(
        credited.changed,
        "the issuing character's objective completes"
    );
    assert_eq!(counts(&issuing, POES_BEST_DECK), [1]);
    assert_eq!(counts(&other, POES_BEST_DECK), [0]);
    assert_eq!(
        fixture.saved_bytes(&other, &two),
        other_before,
        "another player's saved character changed"
    );
}

#[test]
fn shipped_content_is_unchanged_by_the_command() {
    let shipped_path = content_dir().join("quests/card_duelist.json");
    let before = fs::read(&shipped_path).unwrap();
    let document: Value = serde_json::from_slice(&before).unwrap();
    for quest in [POES_BEST_DECK, ESTHER_THE_BRAINS] {
        let record = document["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["id"] == quest)
            .unwrap_or_else(|| panic!("quest {quest} is in the shipped file"));
        assert_eq!(
            record["enabled"],
            Value::Bool(false),
            "quest {quest} must stay disabled"
        );
    }

    // The TYPE 16 whitelist is content, and the cards the chain names are on it while the
    // other activities' rows are not.
    let cards = osfr_tools::card_game::Content::load_optional(&content_dir())
        .unwrap()
        .expect("card_game.json ships");
    for id in [POES_TRICK_DECK, POES_TOURNAMENT_DECK, ESTHERS_FRIENDLY_DECK] {
        assert!(cards.is_duel_row(id), "MiniGameData {id} is TYPE 16");
    }
    for id in [18, 39, 727] {
        assert!(!cards.is_duel_row(id), "MiniGameData {id} is not TYPE 16");
    }

    // Run the real realm dispatch with the shipped split quest catalogue bound: both quests
    // are disabled there, so the credit moves nothing and writes neither the quest file nor
    // the whitelist. The session's other modules come from the play package (`data/content`
    // alone has no `opening.json`, and the realm router lives behind the opening module), so
    // the shipped quest data is what the command is bound to. The reachability producer pin is
    // a separate gate (`enabled_quest_reachability::shipped_minigame_producers_are_pinned`)
    // and was not touched, because no client or NPC path emits a card duel.
    let fixture = fixture_with(&[]);
    let cards_before = fs::read(content_dir().join("card_game.json")).unwrap();
    let mut shipped = Gameplay::load(fixture.path(), None).expect("the play package loads");
    shipped
        .bind_npc_quest_catalogue(&content_dir(), CHARACTER)
        .expect("the shipped catalogue binds");
    admin(&mut shipped);
    let poe = at(POE);
    let credited = duel(&mut shipped, &poe, &POES_TOURNAMENT_DECK.to_string(), 3_000);
    assert!(
        !credited.changed,
        "both Card Duelist quests are disabled in the shipped tree"
    );
    assert!(said(&credited, "Credited card duel 507"));
    assert_eq!(
        fs::read(&shipped_path).unwrap(),
        before,
        "the command wrote to the shipped quest content"
    );
    assert_eq!(
        fs::read(content_dir().join("card_game.json")).unwrap(),
        cards_before,
        "the command wrote to the shipped whitelist"
    );
}
