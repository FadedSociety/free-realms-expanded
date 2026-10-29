//! T786: a Talk step that hands items to an NPC takes them from the bag and credits the step
//! in one transaction (`gameplay/npc.rs` `talk_to_npc` -> `gameplay/quest_delivery.rs` ->
//! `quest_engine::QuestEvent::DeliverItem`).
//!
//! Every runtime test drives the production request path (`Gameplay::handle_in_world`, the
//! gateway's game handler, reached here through the shared 26/8 -> 26/10 Talk helpers) against
//! the shipped Blacksmith record 3020021, copied into the opening fixture and enabled there.
//! The shipped record is `enabled: false` because its other steps and its reward wait for the
//! data pass; the fixture enables it exactly as the pet-trick suite enables its disabled record.
//!
//! The shipped step names item 30480 (the wand the objective spec names) at NPC 9000000203
//! (Silverray, the quest's giver and its turn-in row). The reference opening inventory defines
//! no 30480, so the runtime tests rewrite the copied step's item id to definitions the opening
//! bag really holds; `shipped_record_names_the_wand` reads and binds the shipped row itself.
//!
//! The one rollback the public path cannot reach (the content loader bounds every event field,
//! so a Talk can never produce an invalid delivery event) is unit-tested beside the production
//! transaction in `src/gameplay/quest_delivery.rs`:
//! `failed_event_after_staged_consume_leaves_items_and_progress`.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[allow(dead_code)] // Only the shared reply-text extractor is used here.
#[path = "common/s6.rs"]
mod s6;

use gateway_vendor::count_definition;
use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, TALK_LABEL, character_at_for, coins, content_dir,
    content_position, counts, finish_quest, gateway_send, gateway_talk, has_family, interact,
    menu_option, menu_option_bytes, npc_interact, npc_select, pending_turnin, quest_abandon,
    quest_status, read_json, release_end_screen, saved, seed_quest, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    inventory::RentalState,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use serde_json::{Value, json};
use std::fs;

/// The shipped Blacksmith record under test (`data/content/quests/blacksmith.json`).
const QUEST: u32 = 3_020_021;
/// Silverray, the record's giver, its turn-in row and the delivery step's NPC.
const GIVER: u64 = 9_000_000_203;
/// The item the shipped step names: the wand its craft step points at (30480).
const WAND: i32 = 30_480;
/// A real opening-inventory definition (`slot 7`, uncapped) the reference bag can hold.
const FIXTURE_ITEM: i32 = 75_060;
/// A real opening-inventory definition the state validator accepts as equipped on profile 1
/// slot 3 (`slot 3`, class 1, `max_stack` 1).
const EQUIPPABLE_ITEM: i32 = 24_084;
/// Instance ids the fixture hands out, clear of the opening save's own 1..7 rows.
const ITEM_INSTANCE_BASE: i32 = 90_000;
const NOW: u64 = 1_000;

/// A fixture that carries the shipped NPC rows (its own opening copy wins for the bag) so the
/// Blacksmith giver can be talked to where the content places it.
fn fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.overlay_content(&content_dir());
    // The reference opening bag defines none of the harvesting board's reward rows and this
    // suite never gathers, so drop the overlayed opt-in board (the T503/T788 fixture rule).
    let harvesting = fixture.path().join("harvesting.json");
    if harvesting.is_file() {
        fs::remove_file(&harvesting).expect("drop the unused harvesting board");
    }
    fixture
}

/// The content NPC's authored position; every imported regional quest is in open-world zone 0.
fn at(npc_id: u64) -> CharacterState {
    character_at_for(
        CHARACTER,
        0,
        content_position("blackspore_swamp.json", npc_id),
    )
}

/// Enable the shipped record in the fixture's copied catalogue and let a test rewrite it.
fn patch_record(scratch: &Fixture, edit: impl FnOnce(&mut Value)) {
    scratch.patch_quest("blacksmith.json", QUEST, |record| {
        record["enabled"] = json!(true);
        edit(record);
    });
}

/// Point the shipped delivery step at a definition the opening bag can hold and (optionally)
/// change its required units.
fn rewrite_step(record: &mut Value, item: i32, quantity: u32) {
    let goals = record["objectives"]
        .as_array_mut()
        .expect("the shipped record has the delivery step");
    assert_eq!(goals.len(), 1, "the shipped record carries one step");
    goals[0]["item_id"] = json!(item);
    goals[0]["quantity"] = json!(quantity);
}

/// One bag stack; `rental` makes it a stack a delivery must ignore.
fn add_item(
    state: &mut GameplayState,
    definition_id: i32,
    count: u32,
    instance: i32,
    rental: bool,
) {
    let mut stack = pet_inventory::item(instance, definition_id, count);
    stack.rental = rental.then_some(RentalState {
        expiration: i64::MAX,
        upsells: 0,
        bundle_id: 0,
    });
    state.inventory.items.push(stack);
    let next = state.inventory.next_instance_id.unwrap_or(1);
    state.inventory.next_instance_id = Some(next.max(i64::from(instance) + 1));
}

/// The saved state with the record's one delivery step active and the given stacks in the bag.
/// `edit` runs on it before it is loaded.
fn armed_state(
    scratch: &Fixture,
    item: i32,
    quantity: u32,
    stacks: &[(u32, bool)],
    edit: impl FnOnce(&mut GameplayState),
) -> GameplayState {
    patch_record(scratch, |record| rewrite_step(record, item, quantity));
    let mut state = scratch.game(None).state();
    seed_quest(&mut state, QUEST, QuestStatus::Active, &[0]);
    for (index, (count, rental)) in stacks.iter().enumerate() {
        add_item(
            &mut state,
            item,
            *count,
            ITEM_INSTANCE_BASE + index as i32,
            *rental,
        );
    }
    edit(&mut state);
    state
}

/// [`armed_state`], loaded as a bare `Gameplay`.
fn armed_with(
    scratch: &Fixture,
    item: i32,
    quantity: u32,
    stacks: &[(u32, bool)],
    edit: impl FnOnce(&mut GameplayState),
) -> Gameplay {
    scratch.game(Some(armed_state(scratch, item, quantity, stacks, edit)))
}

fn armed(scratch: &Fixture, item: i32, quantity: u32, stacks: &[(u32, bool)]) -> Gameplay {
    armed_with(scratch, item, quantity, stacks, |_| {})
}

/// The Talk at the delivery NPC.
fn hand_in(game: &mut Gameplay, now: u64) -> osfr_tools::gameplay::Outcome {
    talk(game, &at(GIVER), GIVER, now)
}

#[test]
fn hand_in_takes_the_items_and_credits_the_step() {
    let fixture = fixture();
    let mut game = armed(&fixture, FIXTURE_ITEM, 1, &[(2, false)]);
    let who = at(GIVER);

    let handed = hand_in(&mut game, NOW);
    assert!(handed.changed, "the hand-over saves the player");
    assert_eq!(
        count_definition(&game.state().inventory, FIXTURE_ITEM),
        1,
        "one unit leaves the stack"
    );
    assert_eq!(counts(&game, QUEST), [1], "the step is credited");
    assert_eq!(
        pending_turnin(&game),
        Some(QUEST),
        "the same Talk opens the held end screen"
    );
    assert!(
        handed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 3, 0])),
        "the reduced stack is re-sent (38/3 item update)"
    );
    assert!(
        handed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 9])),
        "the client gets the counted 49/9 update"
    );
    assert!(
        handed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 10])),
        "the one-count step completes"
    );

    // The record's reward rows are unrecovered: the end screen pays nothing, but the turn-in
    // path still completes the quest end to end.
    let end = release_end_screen(&mut game, &who, QUEST, GIVER, 0, NOW + 2);
    assert!(
        has_family(&end, 49, 13, Some(QUEST)),
        "the held 49/13 names the quest"
    );
    let before_coins = coins(&game);
    finish_quest(&mut game, &who, QUEST, ADVENTURER, 0, 0, NOW + 4);
    assert_eq!(coins(&game), before_coins, "no coin reward is invented");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}

#[test]
fn short_hand_in_changes_nothing() {
    let fixture = fixture();
    // The step needs two units; the bag holds one real stack and a rental stack, so neither
    // can satisfy it: a rental is never handed in.
    let mut game = armed(&fixture, FIXTURE_ITEM, 2, &[(1, false), (2, true)]);
    let before = saved(&game);
    let menu = interact(&mut game, &at(GIVER), GIVER, NOW);
    let choice = menu_option(&menu, TALK_LABEL);
    let refused = send(&mut game, &at(GIVER), &npc_select(GIVER, choice), NOW + 1);

    assert!(!refused.changed, "a short hand-in is not a save");
    assert_eq!(
        saved(&game),
        before,
        "a short hand-in changed the bag or the progress"
    );
    assert_eq!(
        count_definition(&game.state().inventory, FIXTURE_ITEM),
        3,
        "neither stack was touched"
    );
    assert_eq!(counts(&game, QUEST), [0], "the step was not credited");
    let lines = s6::chat_texts(&refused.replies);
    assert_eq!(lines.len(), 1, "exactly one chat line: {lines:?}");
    assert!(
        lines[0].contains("items this step needs"),
        "the line is the delivery refusal: {lines:?}"
    );
    assert!(
        refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the Talk still opens its dialogue"
    );
    assert!(
        !refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 9, 0])),
        "a short hand-in sends no objective update"
    );

    // The refusal did not consume the open NPC menu: the same Talk row still answers.
    let again = send(&mut game, &at(GIVER), &npc_select(GIVER, choice), NOW + 2);
    assert!(
        again
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "a refused hand-in consumed the open menu"
    );
    assert_eq!(saved(&game), before);
}

#[test]
fn quantity_spans_stacks() {
    let fixture = fixture();
    let mut game = armed(&fixture, FIXTURE_ITEM, 4, &[(2, false), (3, false)]);
    let handed = hand_in(&mut game, NOW);
    assert!(handed.changed);
    assert_eq!(
        count_definition(&game.state().inventory, FIXTURE_ITEM),
        1,
        "four units come out of two stacks"
    );
    assert_eq!(
        game.state()
            .inventory
            .items
            .iter()
            .filter(|item| item.definition_id == FIXTURE_ITEM)
            .count(),
        1,
        "the emptied stack is removed"
    );
    assert_eq!(counts(&game, QUEST), [1]);
}

#[test]
fn second_hand_in_takes_nothing() {
    let fixture = fixture();
    let mut game = armed(&fixture, FIXTURE_ITEM, 1, &[(1, false)]);
    hand_in(&mut game, NOW);
    assert_eq!(count_definition(&game.state().inventory, FIXTURE_ITEM), 0);
    let after = saved(&game);

    let again = hand_in(&mut game, NOW + 10);
    assert!(!again.changed, "the second Talk marks no save");
    assert_eq!(
        count_definition(&game.state().inventory, FIXTURE_ITEM),
        0,
        "a completed step takes no second item"
    );
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(saved(&game), after, "the second Talk changed quest state");
}

#[test]
fn equipped_item_is_unequipped_and_taken() {
    let fixture = fixture();
    let mut game = armed_with(&fixture, EQUIPPABLE_ITEM, 1, &[(1, false)], |state| {
        let slot = state
            .inventory
            .equipment
            .iter_mut()
            .find(|entry| entry.profile_id == 1 && entry.slot == 3)
            .expect("the opening save has profile 1 slot 3");
        slot.item_id = ITEM_INSTANCE_BASE;
    });
    assert!(
        game.state()
            .inventory
            .equipment
            .iter()
            .any(|entry| entry.profile_id == 1 && entry.slot == 3 && entry.item_id > 0),
        "the item starts equipped"
    );

    let handed = hand_in(&mut game, NOW);
    assert!(handed.changed);
    assert_eq!(
        count_definition(&game.state().inventory, EQUIPPABLE_ITEM),
        0
    );
    assert!(
        game.state()
            .inventory
            .equipment
            .iter()
            .all(|entry| entry.item_id != ITEM_INSTANCE_BASE),
        "the equipped slot was cleared"
    );
    assert!(
        handed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 6])),
        "the cleared slot is sent as 38/6"
    );
    assert!(
        handed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 4])),
        "the removed instance is sent as 38/4"
    );
}

#[test]
fn reload_after_hand_in_keeps_both() {
    let fixture = fixture();
    let mut game = armed(&fixture, FIXTURE_ITEM, 2, &[(3, false)]);
    let who = at(GIVER);
    hand_in(&mut game, NOW);
    assert_eq!(count_definition(&game.state().inventory, FIXTURE_ITEM), 1);
    assert_eq!(counts(&game, QUEST), [1]);

    // The real save envelope, not a state proxy: both the bag and the progress survive.
    let reloaded = fixture.save_and_reload(&game, &who);
    assert_eq!(
        count_definition(&reloaded.state().inventory, FIXTURE_ITEM),
        1,
        "the bag kept the hand-over"
    );
    assert_eq!(
        counts(&reloaded, QUEST),
        [1],
        "the credit is saved progress"
    );
    assert_eq!(pending_turnin(&reloaded), Some(QUEST));
}

#[test]
fn abandon_after_hand_in_does_not_refund() {
    let fixture = fixture();
    let mut game = armed(&fixture, FIXTURE_ITEM, 1, &[(2, false)]);
    hand_in(&mut game, NOW);
    assert_eq!(count_definition(&game.state().inventory, FIXTURE_ITEM), 1);

    let abandoned = send(&mut game, &at(GIVER), &quest_abandon(QUEST), NOW + 10);
    assert!(abandoned.changed, "the abandon was accepted");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Abandoned);
    assert_eq!(
        count_definition(&game.state().inventory, FIXTURE_ITEM),
        1,
        "(R) a spent hand-in is not refunded on abandon"
    );
    assert_eq!(pending_turnin(&game), None);
}

#[test]
fn inactive_delivery_step_takes_nothing() {
    let fixture = fixture();
    // Ordered [talk, deliver]: only the open step may act, so the first Talk credits the talk
    // step and must leave the bag alone even though the NPC is also the delivery target.
    patch_record(&fixture, |record| {
        record["objectives"] = json!([
            {
                "kind": "talk_to_npc",
                "id": 1,
                "npc_id": GIVER,
                "source": [{"kind": "reconstructed", "evidence": "(R) T786 scratch talk step"}],
            },
            {
                "kind": "deliver_item",
                "id": 2,
                "npc_id": GIVER,
                "item_id": FIXTURE_ITEM,
                "quantity": 1,
                "source": [{"kind": "reconstructed", "evidence": "(R) T786 scratch delivery step"}],
            },
        ]);
    });
    let mut state = fixture.game(None).state();
    seed_quest(&mut state, QUEST, QuestStatus::Active, &[0, 0]);
    add_item(&mut state, FIXTURE_ITEM, 2, ITEM_INSTANCE_BASE, false);
    let mut game = fixture.game(Some(state));

    let first = hand_in(&mut game, NOW);
    assert!(first.changed, "the talk step is credited");
    assert_eq!(counts(&game, QUEST), [1, 0]);
    assert_eq!(
        count_definition(&game.state().inventory, FIXTURE_ITEM),
        2,
        "an inactive delivery step takes nothing"
    );

    // Once the delivery step opens, the same Talk takes its unit.
    let second = hand_in(&mut game, NOW + 10);
    assert!(second.changed);
    assert_eq!(counts(&game, QUEST), [1, 1]);
    assert_eq!(count_definition(&game.state().inventory, FIXTURE_ITEM), 1);
    assert_eq!(pending_turnin(&game), Some(QUEST));
}

#[test]
fn out_of_range_quantity_is_rejected_at_load() {
    for bad in [0u32, 1_000_001] {
        let fixture = fixture();
        patch_record(&fixture, |record| rewrite_step(record, FIXTURE_ITEM, bad));
        let mut game = Gameplay::load(fixture.path(), None).expect("the opening content loads");
        assert!(
            game.bind_npc_quest_catalogue(&fixture.quest_catalogue(), CHARACTER)
                .is_err(),
            "quantity {bad} must be refused by the split loader"
        );
    }
}

#[test]
fn hand_in_touches_only_the_talking_player() {
    let fixture = fixture();
    let mut mine = armed(&fixture, FIXTURE_ITEM, 1, &[(1, false)]);
    let other = CHARACTER + 1;
    let mut other_state = fixture.game_as(other, None).state();
    seed_quest(&mut other_state, QUEST, QuestStatus::Active, &[0]);
    add_item(
        &mut other_state,
        FIXTURE_ITEM,
        1,
        ITEM_INSTANCE_BASE + 10,
        false,
    );
    let theirs = fixture.game_as(other, Some(other_state));
    let their_before = saved(&theirs);

    hand_in(&mut mine, NOW);
    assert_eq!(count_definition(&mine.state().inventory, FIXTURE_ITEM), 0);
    assert_eq!(
        count_definition(&theirs.state().inventory, FIXTURE_ITEM),
        1,
        "the other bag is untouched"
    );
    assert_eq!(counts(&theirs, QUEST), [0]);
    assert_eq!(
        saved(&theirs),
        their_before,
        "the other player's save changed"
    );
}

/// A gateway session standing at the delivery NPC with `state` loaded: the gateway stages the
/// game per packet and commits the outcome (`Gateway::handle_at`), the path a hand-in really takes.
fn session_at_giver(fixture: &Fixture, state: GameplayState) -> Gateway {
    let position = content_position("blackspore_swamp.json", GIVER);
    fixture.session(CHARACTER, position, Some(state))
}

#[test]
fn gateway_hand_in_commits_items_and_progress() {
    let fixture = fixture();
    let state = armed_state(&fixture, FIXTURE_ITEM, 1, &[(2, false)], |_| {});
    let mut gateway = session_at_giver(&fixture, state);

    let replies = gateway_talk(&mut gateway, GIVER, NOW);
    assert!(
        replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 9, 0])),
        "the hand-in sends the counted objective update"
    );
    let game = gateway.gameplay().expect("the session has gameplay");
    assert_eq!(
        count_definition(&game.state().inventory, FIXTURE_ITEM),
        1,
        "the committed bag lost a unit"
    );
    assert_eq!(counts(game, QUEST), [1], "the committed step is credited");
    assert_eq!(pending_turnin(game), Some(QUEST));
}

#[test]
fn gateway_short_hand_in_commits_nothing_and_keeps_the_menu() {
    let fixture = fixture();
    let state = armed_state(&fixture, FIXTURE_ITEM, 2, &[(1, false), (2, true)], |_| {});
    let mut gateway = session_at_giver(&fixture, state);
    let before = saved(gateway.gameplay().unwrap());

    let menu = gateway_send(&mut gateway, &npc_interact(GIVER), NOW);
    let menu = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 answers the 26/9 menu");
    let choice = menu_option_bytes(menu, TALK_LABEL);
    for now in [NOW + 1, NOW + 2] {
        let refused = gateway_send(&mut gateway, &npc_select(GIVER, choice), now);
        assert_eq!(
            s6::chat_texts(&refused).len(),
            1,
            "one refusal line per attempt, and the second select proves the menu stayed open"
        );
        assert!(
            refused
                .iter()
                .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
            "the Talk still opens its dialogue"
        );
        assert_eq!(
            saved(gateway.gameplay().unwrap()),
            before,
            "a short hand-in committed a change"
        );
    }
}

/// The shipped record itself: the wand id, the delivery NPC and the unit count are the data the
/// mechanism reads, and the production split loader binds the record (still disabled, because the
/// rest of the line and its reward wait for the data pass).
#[test]
fn shipped_record_names_the_wand() {
    let document = read_json(&content_dir().join("quests/blacksmith.json"));
    let record = document["records"]
        .as_array()
        .expect("blacksmith records")
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(QUEST)))
        .expect("the Wand Replacement record exists");
    assert!(
        !record["enabled"].as_bool().unwrap_or(true),
        "the record ships disabled until the id pass completes the line"
    );
    let goals = record["objectives"]
        .as_array()
        .expect("the record carries its delivery step");
    assert_eq!(goals.len(), 1, "only the delivery step is wired");
    assert_eq!(goals[0]["kind"].as_str(), Some("deliver_item"));
    assert_eq!(goals[0]["npc_id"].as_u64(), Some(GIVER));
    assert_eq!(goals[0]["item_id"].as_i64(), Some(i64::from(WAND)));
    assert_eq!(goals[0]["quantity"].as_u64(), Some(1));

    let eligibility = CharacterEligibility {
        player_level: 20,
        ..CharacterEligibility::default()
    };
    let engine = QuestEngine::from_split_content(&content_dir(), CHARACTER, eligibility, None)
        .expect("the shipped split catalogue binds");
    assert!(
        !engine.is_enabled(QUEST),
        "a disabled record is never offered"
    );
}
