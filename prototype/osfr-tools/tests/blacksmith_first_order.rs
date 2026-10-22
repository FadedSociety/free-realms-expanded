//! T274: the Blacksmith's first playable orders, "Smithy Merchandise" (3020003) and
//! "Curious Cancelation" (3020005), and Bruno Battlerock's recovered crafting stock.
//!
//! Content-only slice: the quests are in `data/content/quests/blacksmith.json` and Bruno's
//! stock is in `data/content/vendors.json` / `stores.json`. These tests drive the production
//! in-world router (`Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `QuestEngine`), the
//! production position event (`Gameplay::on_location`, called by `gateway.rs` after movement
//! validation), and the real 165/10 -> 66/43 -> 66/44 vendor route. They do not call
//! `QuestOutcome` or `CoinShop::buy_vendor` directly.
//!
//! Client evidence used here:
//! - 26/8 NPC interact: constructor `008b6bc0`; u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; u64 target + i32 choice.
//! - 26/6 dialogue response: reader `009f5890`; i32 row (301 is the quest row).
//! - 49/1 offer: reader `00c7bb60`; 49/2 accept: constructor `00a86400`.
//! - 49/13 end screen: `00c7bbd0`/`00c7b990` (actor 6..14, quest 14..18, end text 18..22,
//!   reward bundle from 26; `008e7930` reads the coin word at 27..31 and the star total at
//!   31..35); 49/14 end reply: constructor `00a864c0`.
//! - 165/10 coin list: `00c78fb0` (shared row reader `00b29cd0`); 66/43 buy: serializer `00b3ab20`;
//!   66/44 result: reader `00b32430`, status dword at wire offset 48.
//!
//! The shared quest fixture and packet helpers live in `tests/support/npc_quest_chain.rs`; the
//! vendor packet and result decoders live in `tests/common/gateway_vendor.rs`.
#[allow(dead_code)] // The shared vendor kit carries helpers this suite does not send.
#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use gateway_vendor::{count_definition, merchant_status, vendor_buy_packet};
use npc_quest_chain::{
    CHARACTER, Fixture, accept, assert_no_offer, assert_packet_changes_nothing, character_at_for,
    coins, content_dir, content_position, counts, end_screen_reward, finish_quest, has_family,
    interact, menu_option, npc_interact, npc_select, offer, pending_offer, pending_turnin,
    profile_xp, quest_end_reply, quest_reply, quest_status, read_json, release_end_screen, saved,
    select_job, send, talk_step,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    inventory::{Inventory, InventoryItem},
    npc_interaction::{NpcAction, NpcInteraction, NpcSelection, decode_coin_merchant_list_165_10},
    profiles::ProfileState,
    quest_engine::QuestStatus,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------------------------
// Production content keys. NPC ids are `data/content/npcs/snowhill.json` rows; quest ids are
// `data/content/quests/blacksmith.json`.
// ---------------------------------------------------------------------------------------------
const SMITTY: u64 = 9_000_000_206;
const TRIXI: u64 = 9_000_000_220;
const BRUNO: u64 = 9_000_000_034;
const BLACKSMITH: i32 = 16;
const BRAWLER: i32 = 43;
const MERCHANDISE: u32 = 3_020_003;
const CANCELATION: u32 = 3_020_005;
const WOODEN_PLANKS: i32 = 55;
const THORNY_FASTENERS: i32 = 56;
const SINGING_COAL: i32 = 879;
/// Legacy `ClientItemDefinitions.Cost` for item 55/56/879, the reference price in `stores.json`.
const UNIT_PRICE: u32 = 12;
const OPEN_WORLD: u32 = 0;
/// `job_trainers.json` Smitty row for "Become a Blacksmith" (profile 16).
const LEARN_BLACKSMITH_LABEL: u32 = 412_314;
/// `npc_interaction::SHOP_STRING_ID`, the 26/9 menu label whose action is `Shop`.
const SHOP_LABEL: u32 = 432_156;
const MERCHANDISE_END_TEXT: u32 = 90_073;
const CANCELATION_END_TEXT: u32 = 90_737;

fn spot(guid: u64, npc_id: u64) -> CharacterState {
    character_at_for(guid, OPEN_WORLD, content_position("snowhill.json", npc_id))
}

fn blacksmith_state(fixture: &Fixture, guid: u64, coins: u32) -> GameplayState {
    let mut state = fixture.game_as(guid, None).state();
    state.profiles = Some(ProfileState {
        version: 1,
        active: BLACKSMITH,
        unlocked: [1, BLACKSMITH, BRAWLER].into_iter().collect(),
        experience: BTreeMap::new(),
    });
    state.inventory.coins = coins;
    state
}

fn blacksmith_game(fixture: &Fixture, guid: u64, coins: u32) -> Gameplay {
    fixture.game_as(guid, Some(blacksmith_state(fixture, guid, coins)))
}

/// Accept 3020003 at Smitty through the real 26/8 -> 26/10 -> 26/6 -> 49/1 -> 49/2 route.
fn accept_merchandise(game: &mut Gameplay, smitty: &CharacterState, now: &mut u64) {
    *now += 10;
    offer(game, smitty, SMITTY, MERCHANDISE, *now);
    *now += 10;
    accept(game, smitty, MERCHANDISE, *now);
    assert_eq!(quest_status(game, MERCHANDISE), QuestStatus::Active);
}

/// The production position event: `gateway.rs` forwards a validated pose to `on_location`.
fn visit_bruno(game: &mut Gameplay, bruno: &CharacterState, now: u64) -> Outcome {
    game.on_location(bruno, now).expect("location event")
}

/// Read the real 165/10 coin merchant list the client opens from the 26/10 Shop choice.
fn merchant_list(game: &mut Gameplay, target: &CharacterState, now: u64) -> Vec<u8> {
    let menu = interact(game, target, BRUNO, now);
    let choice = menu_option(&menu, SHOP_LABEL);
    let opened = send(game, target, &npc_select(BRUNO, choice), now + 1);
    let list = opened
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
        .expect("the Shop choice sends the 165/10 coin merchant list")
        .clone();
    assert_eq!(&list[..4], &[165, 0, 10, 0]);
    list
}

fn merchant_ids(list: &[u8]) -> Vec<i32> {
    decode_coin_merchant_list_165_10(list)
        .expect("165/10 matches the shared client row reader")
        .entries
        .iter()
        .map(|entry| entry.item_id as i32)
        .collect()
}

fn buy(
    game: &mut Gameplay,
    target: &CharacterState,
    transaction: u32,
    item_id: i32,
    quantity: u32,
    now: u64,
) -> Outcome {
    send(
        game,
        target,
        &vendor_buy_packet(transaction, item_id, quantity),
        now,
    )
}

fn result_status(outcome: &Outcome) -> u32 {
    merchant_status(
        outcome
            .replies
            .last()
            .expect("a 66/43 request always answers with 66/44"),
    )
}

/// A refused 66/43 must answer 66/44 false, report no change, and leave the full typed state
/// equal. `!outcome.changed` is what keeps the gateway from writing a save for the refusal.
fn assert_buy_refused(
    game: &mut Gameplay,
    target: &CharacterState,
    transaction: u32,
    item_id: i32,
    quantity: u32,
    now: u64,
    label: &str,
) {
    let before = snapshot(game);
    let refused = buy(game, target, transaction, item_id, quantity, now);
    assert!(!refused.changed, "{label}: the refusal reported a change");
    assert_eq!(
        result_status(&refused),
        0,
        "{label}: the client was told the buy succeeded"
    );
    assert_unchanged(game, &before, label);
}

/// A refusal must leave the full typed `GameplayState` equal. `saved` serializes the same
/// state for cross-session comparisons; it is not the character-store envelope.
fn snapshot(game: &Gameplay) -> GameplayState {
    game.state()
}

fn assert_unchanged(game: &Gameplay, before: &GameplayState, label: &str) {
    assert_eq!(
        &game.state(),
        before,
        "{label}: the full gameplay state changed"
    );
}

fn assert_no_offer_without_change(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: &mut u64,
    label: &str,
) {
    let before = snapshot(game);
    assert_no_offer(game, character, target, now, label);
    assert_unchanged(game, &before, label);
}

fn assert_packet_refused_without_change(
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = snapshot(game);
    assert_packet_changes_nothing(game, character, packet, now, label);
    assert_unchanged(game, &before, label);
}

/// Finish 3020003 after the visit: return to Smitty through 26/8 -> 26/10 -> 26/6, release the
/// held 49/13, then send 49/14. The helper asserts the exact coin and job-XP delta.
fn finish_merchandise(game: &mut Gameplay, smitty: &CharacterState, now: &mut u64) {
    *now += 10;
    talk_step(game, smitty, SMITTY, *now, "Smithy Merchandise return");
    assert_eq!(pending_turnin(game), Some(MERCHANDISE));
    *now += 10;
    let end = release_end_screen(
        game,
        smitty,
        MERCHANDISE,
        SMITTY,
        MERCHANDISE_END_TEXT,
        *now,
    );
    assert_eq!(
        end_screen_reward(&end),
        (5, 20),
        "the end screen displays the configured 5 coins and 20 Blacksmith job-XP total"
    );
    *now += 10;
    finish_quest(game, smitty, MERCHANDISE, BLACKSMITH, 5, 20, *now);
}

#[test]
fn smitty_offers_the_merchandise_lesson_only_to_blacksmiths() {
    let fixture = Fixture::minimal(CHARACTER);
    let smitty = spot(CHARACTER, SMITTY);
    let mut now = 1_000;

    // An Adventurer sees no Blacksmith order, and the open menu is untouched by the refusal.
    let mut adventurer = fixture.game(None);
    assert_no_offer_without_change(
        &mut adventurer,
        &smitty,
        SMITTY,
        &mut now,
        "Adventurer at Smitty",
    );
    let menu = interact(&mut adventurer, &smitty, SMITTY, now + 1);
    menu_option(&menu, LEARN_BLACKSMITH_LABEL);
    assert!(
        !menu
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None)),
        "a non-Blacksmith must not be offered the Blacksmith lesson"
    );

    // A real Blacksmith (profile 16 unlocked and active) gets exactly one 49/1 for 3020003,
    // without the disabled 3020002 unlock quest being completed.
    let mut blacksmith = blacksmith_game(&fixture, CHARACTER, 0);
    let offered = offer(&mut blacksmith, &smitty, SMITTY, MERCHANDISE, now + 10);
    assert_eq!(
        offered
            .replies
            .iter()
            .filter(|packet| has_family(packet, 49, 1, Some(MERCHANDISE)))
            .count(),
        1,
        "the lesson must be offered once"
    );
    assert_eq!(
        pending_offer(&blacksmith),
        Some(MERCHANDISE),
        "the offer is pending for the 49/2 reply"
    );
    assert!(
        !blacksmith
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&3_020_002),
        "3020002 stays disabled and is not required as a prerequisite"
    );
}

#[test]
fn the_supply_visit_and_the_vendor_purchase_pay_once() {
    let fixture = Fixture::minimal(CHARACTER);
    let smitty = spot(CHARACTER, SMITTY);
    let bruno = spot(CHARACTER, BRUNO);
    let mut game = blacksmith_game(&fixture, CHARACTER, 100);
    let mut now = 2_000;

    accept_merchandise(&mut game, &smitty, &mut now);

    // The production position event advances the visit objective inside the 8 m stall radius.
    now += 10;
    let visited = visit_bruno(&mut game, &bruno, now);
    assert!(visited.changed, "the supply visit must advance 3020003");
    assert_eq!(counts(&game, MERCHANDISE), [1, 0]);

    // The Shop choice opens the client-readable 165/10 coin component list.
    now += 10;
    let list = merchant_list(&mut game, &bruno, now);

    // Buy exactly four fasteners and four planks through 66/43; each answer is a successful
    // 66/44 and the wallet/stack deltas are exact.
    now += 10;
    let bought_fasteners = buy(&mut game, &bruno, 91_001, THORNY_FASTENERS, 4, now);
    assert_eq!(result_status(&bought_fasteners), 1);
    assert_eq!(coins(&game), 100 - 4 * UNIT_PRICE);
    assert_eq!(
        count_definition(&game.state().inventory, THORNY_FASTENERS),
        4
    );
    now += 10;
    let bought_planks = buy(&mut game, &bruno, 91_002, WOODEN_PLANKS, 4, now);
    assert_eq!(result_status(&bought_planks), 1);
    assert_eq!(coins(&game), 100 - 8 * UNIT_PRICE);
    assert_eq!(count_definition(&game.state().inventory, WOODEN_PLANKS), 4);
    assert_eq!(
        merchant_ids(&list),
        vec![WOODEN_PLANKS, THORNY_FASTENERS, SINGING_COAL],
        "the 165/10 list is exactly Bruno's three recovered components"
    );

    // A locked profile is refused by the reward path; the XP belongs to profile 16 even when
    // the character has switched to Brawler before the turn-in.
    now += 10;
    let switched = send(&mut game, &smitty, &select_job(BRAWLER), now);
    assert!(switched.changed);
    assert_eq!(game.state().profiles.as_ref().unwrap().active, BRAWLER);

    let coins_before = coins(&game);
    finish_merchandise(&mut game, &smitty, &mut now);
    assert_eq!(coins(&game), coins_before + 5);
    assert_eq!(profile_xp(&game, BLACKSMITH), 20);
    assert_eq!(profile_xp(&game, BRAWLER), 0, "XP stays on profile 16");
    assert_eq!(counts(&game, MERCHANDISE), [1, 1]);
    assert_eq!(
        count_definition(&game.state().inventory, THORNY_FASTENERS),
        4,
        "the bought components are not consumed by the turn-in"
    );
    assert_eq!(count_definition(&game.state().inventory, WOODEN_PLANKS), 4);

    // A replayed 49/14 pays nothing and changes no gameplay state.
    now += 10;
    assert_packet_refused_without_change(
        &mut game,
        &smitty,
        &quest_end_reply(MERCHANDISE),
        now,
        "replayed 49/14",
    );
}

#[test]
fn the_cancelation_talk_chain_pays_once() {
    let fixture = Fixture::minimal(CHARACTER);
    let smitty = spot(CHARACTER, SMITTY);
    let bruno = spot(CHARACTER, BRUNO);
    let trixi = spot(CHARACTER, TRIXI);
    let mut game = blacksmith_game(&fixture, CHARACTER, 100);
    let mut now = 3_000;

    accept_merchandise(&mut game, &smitty, &mut now);

    // 3020005's prerequisite is 3020003: while 3020003 is only active, Smitty must not offer
    // the later order. This throws away the earlier accept gate, so the offer check itself
    // must refuse (a cleared prerequisite would fail here).
    assert_no_offer_without_change(
        &mut game,
        &smitty,
        SMITTY,
        &mut now,
        "3020005 before 3020003 completes",
    );

    now += 10;
    visit_bruno(&mut game, &bruno, now);
    finish_merchandise(&mut game, &smitty, &mut now);

    // 3020005 is offered only after 3020003 completes, and both steps are ordered Talks.
    now += 10;
    offer(&mut game, &smitty, SMITTY, CANCELATION, now);
    now += 10;
    accept(&mut game, &smitty, CANCELATION, now);
    now += 10;
    talk_step(&mut game, &trixi, TRIXI, now, "Curious Cancelation: Trixi");
    assert_eq!(counts(&game, CANCELATION), [1, 0]);
    now += 10;
    talk_step(
        &mut game,
        &smitty,
        SMITTY,
        now,
        "Curious Cancelation: Smitty",
    );
    assert_eq!(counts(&game, CANCELATION), [1, 1]);
    assert_eq!(pending_turnin(&game), Some(CANCELATION));

    let coins_before = coins(&game);
    now += 10;
    release_end_screen(
        &mut game,
        &smitty,
        CANCELATION,
        SMITTY,
        CANCELATION_END_TEXT,
        now,
    );
    now += 10;
    finish_quest(&mut game, &smitty, CANCELATION, BLACKSMITH, 5, 20, now);
    assert_eq!(coins(&game), coins_before + 5);
    assert_eq!(profile_xp(&game, BLACKSMITH), 40);
    assert_eq!(quest_status(&game, CANCELATION), QuestStatus::Completed);

    now += 10;
    assert_packet_refused_without_change(
        &mut game,
        &smitty,
        &quest_end_reply(CANCELATION),
        now,
        "replayed Curious Cancelation turn-in",
    );
}

#[test]
fn refusals_and_reconnect_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let smitty = spot(CHARACTER, SMITTY);
    let bruno = spot(CHARACTER, BRUNO);
    let trixi = spot(CHARACTER, TRIXI);
    let mut now = 4_000;

    // Non-Blacksmith: no offer and no forged accept.
    let mut adventurer = fixture.game(None);
    assert_no_offer_without_change(
        &mut adventurer,
        &smitty,
        SMITTY,
        &mut now,
        "non-Blacksmith offer",
    );
    assert_packet_refused_without_change(
        &mut adventurer,
        &smitty,
        &quest_reply(MERCHANDISE, true),
        now,
        "forged 49/2 as a non-Blacksmith",
    );

    // Missing 3020003: the later order and a turn-in with no end screen are refused.
    let mut missing = blacksmith_game(&fixture, CHARACTER + 1, 100);
    assert_packet_refused_without_change(
        &mut missing,
        &smitty,
        &quest_reply(CANCELATION, true),
        now,
        "3020005 without 3020003",
    );
    assert_packet_refused_without_change(
        &mut missing,
        &smitty,
        &quest_end_reply(MERCHANDISE),
        now + 1,
        "49/14 with no end screen",
    );

    // Wrong NPC: the return Talk to Trixi neither advances nor offers anything.
    let mut wrong_npc = blacksmith_game(&fixture, CHARACTER + 2, 100);
    accept_merchandise(&mut wrong_npc, &smitty, &mut now);
    assert_no_offer_without_change(
        &mut wrong_npc,
        &trixi,
        TRIXI,
        &mut now,
        "wrong NPC on the return Talk",
    );

    // Out-of-range 26/8 is refused before any state change.
    let at_smitty = spot(CHARACTER + 3, SMITTY);
    let far = CharacterState {
        guid: at_smitty.guid,
        position: [
            at_smitty.position[0] + 400.0,
            at_smitty.position[1],
            at_smitty.position[2],
        ],
        ..at_smitty
    };
    let mut out_of_range = blacksmith_game(&fixture, CHARACTER + 3, 100);
    let before = snapshot(&out_of_range);
    assert!(
        out_of_range
            .handle_in_world(&npc_interact(SMITTY), &far, now)
            .is_err(),
        "an out-of-range 26/8 must be refused"
    );
    assert_unchanged(&out_of_range, &before, "out-of-range 26/8");

    // Insufficient coins: buying two costs 24 against a 20-coin wallet, so the 66/44 refusal
    // answers and changes nothing. The same open shop then sells one for 12, proving the
    // refusal did not consume the shop.
    let mut poor = blacksmith_game(&fixture, CHARACTER + 4, 20);
    accept_merchandise(&mut poor, &smitty, &mut now);
    poor.on_location(&bruno, now).expect("location event");
    merchant_list(&mut poor, &bruno, now + 1);
    assert_buy_refused(
        &mut poor,
        &bruno,
        91_010,
        THORNY_FASTENERS,
        2,
        now + 2,
        "insufficient coins",
    );
    let bought = buy(&mut poor, &bruno, 91_015, THORNY_FASTENERS, 1, now + 3);
    assert_eq!(result_status(&bought), 1, "the refusal closed the shop");
    assert_eq!(coins(&poor), 20 - UNIT_PRICE);
    assert_eq!(
        count_definition(&poor.state().inventory, THORNY_FASTENERS),
        1
    );

    // Full bag: 100 stacks refuse a new stack, but the open shop still merges a stack of the
    // same item, proving the refusal did not consume it.
    let mut full_state = blacksmith_state(&fixture, CHARACTER + 5, 100);
    full_state.inventory.items.clear();
    for entry in &mut full_state.inventory.equipment {
        entry.item_id = 0;
    }
    for id in 100_000..100_100 {
        full_state.inventory.items.push(InventoryItem {
            id,
            definition_id: if id == 100_000 {
                THORNY_FASTENERS
            } else {
                21_496
            },
            tint: 0,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
    }
    full_state.inventory.next_instance_id = Some(100_100);
    let mut full = fixture.game_as(CHARACTER + 5, Some(full_state));
    accept_merchandise(&mut full, &smitty, &mut now);
    full.on_location(&bruno, now).expect("location event");
    merchant_list(&mut full, &bruno, now + 1);
    assert_buy_refused(
        &mut full,
        &bruno,
        91_011,
        WOODEN_PLANKS,
        1,
        now + 2,
        "full bag",
    );
    let merged = buy(&mut full, &bruno, 91_016, THORNY_FASTENERS, 1, now + 3);
    assert_eq!(result_status(&merged), 1, "the refusal closed the shop");
    assert_eq!(
        count_definition(&full.state().inventory, THORNY_FASTENERS),
        2
    );

    // Unknown item, zero quantity and an oversized quantity are refused through the same
    // open shop; a valid buy afterwards proves the shop survived all three.
    let buyer = spot(CHARACTER + 6, BRUNO);
    let mut game = blacksmith_game(&fixture, CHARACTER + 6, 100);
    accept_merchandise(&mut game, &smitty, &mut now);
    game.on_location(&buyer, now).expect("location event");
    merchant_list(&mut game, &buyer, now + 1);
    assert_buy_refused(
        &mut game,
        &buyer,
        91_012,
        i32::MAX,
        1,
        now + 2,
        "unknown item",
    );
    assert_buy_refused(
        &mut game,
        &buyer,
        91_013,
        THORNY_FASTENERS,
        0,
        now + 3,
        "zero quantity",
    );
    assert_buy_refused(
        &mut game,
        &buyer,
        91_017,
        THORNY_FASTENERS,
        u32::MAX,
        now + 4,
        "oversized quantity",
    );
    let valid = buy(&mut game, &buyer, 91_018, THORNY_FASTENERS, 1, now + 5);
    assert_eq!(result_status(&valid), 1, "the refusal closed the shop");
    assert_eq!(coins(&game), 100 - UNIT_PRICE);
    assert_eq!(
        count_definition(&game.state().inventory, THORNY_FASTENERS),
        1
    );

    // A mid-quest reconnect keeps the visit counter and the bought components, and the quest
    // still pays exactly once after the reload.
    let mut reconnect = blacksmith_game(&fixture, CHARACTER + 7, 100);
    accept_merchandise(&mut reconnect, &smitty, &mut now);
    let bruno_at = spot(CHARACTER + 7, BRUNO);
    reconnect
        .on_location(&bruno_at, now)
        .expect("location event");
    merchant_list(&mut reconnect, &bruno_at, now + 1);
    let bought = buy(
        &mut reconnect,
        &bruno_at,
        91_014,
        THORNY_FASTENERS,
        1,
        now + 2,
    );
    assert_eq!(result_status(&bought), 1);
    let reconnect = fixture.game_as(CHARACTER + 7, Some(reconnect.state()));
    assert_eq!(counts(&reconnect, MERCHANDISE), [1, 0]);
    assert_eq!(
        count_definition(&reconnect.state().inventory, THORNY_FASTENERS),
        1
    );
    let mut reconnect = reconnect;
    let coins_before = coins(&reconnect);
    let smitty_at = spot(CHARACTER + 7, SMITTY);
    finish_merchandise(&mut reconnect, &smitty_at, &mut now);
    assert_eq!(coins(&reconnect), coins_before + 5);
    assert_eq!(profile_xp(&reconnect, BLACKSMITH), 20);
}

#[test]
fn bruno_sells_exactly_the_three_recovered_components() {
    let fixture = Fixture::new();
    let content = content_dir();
    let vendors = read_json(&content.join("vendors.json"));
    let record = vendors["vendors"]
        .as_array()
        .expect("vendors are an array")
        .iter()
        .find(|vendor| vendor["npc_id"].as_u64() == Some(BRUNO))
        .expect("Bruno Battlerock has one vendor record");
    assert_eq!(record["item_ids"], json!([55, 56, 879]));
    assert_eq!(
        record["stock_completeness"],
        "all_named_evidence_items_mapped"
    );
    let items = record["items"].as_array().expect("items are an array");
    assert_eq!(items.len(), 3, "no fourth component is invented");

    // The NPC row must agree with the vendor record instead of claiming no stock (T274 MINOR 2).
    let snowhill = read_json(&content.join("npcs/snowhill.json"));
    let bruno = snowhill["records"]
        .as_array()
        .expect("NPC records are an array")
        .iter()
        .find(|npc| npc["npc_id"].as_u64() == Some(BRUNO))
        .expect("Bruno is a positioned Snowhill NPC");
    let stock = bruno["vendor_stock"]
        .as_str()
        .expect("Bruno's record carries the recovered stock, not null");
    for name in ["Thorny Fasteners", "Wooden Planks", "Singing Coal"] {
        assert!(stock.contains(name), "vendor_stock names {name}");
    }
    assert!(bruno["vendor_stock_reason"].is_null());

    let catalogue = Inventory::load(fixture.path(), None).expect("the runtime catalogue loads");
    let stores = read_json(&content.join("stores.json"));
    let prices: BTreeMap<i64, &Value> = stores["items"]
        .as_array()
        .expect("store items are an array")
        .iter()
        .map(|row| {
            (
                row["item_id"]
                    .as_i64()
                    .expect("store item id is an integer"),
                row,
            )
        })
        .collect();
    for (index, item) in items.iter().enumerate() {
        let id = item["item_id"].as_i64().expect("item id is an integer");
        assert_eq!(record["item_ids"][index], id, "runtime ids follow the rows");
        let definition = i32::try_from(id).expect("vendor item id fits i32");
        assert!(
            catalogue.has_definition(definition),
            "item {id} resolves in the runtime catalogue"
        );
        assert_eq!(item["runtime_eligible"], true);
        let price = prices
            .get(&id)
            .unwrap_or_else(|| panic!("item {id} has a matching store row"));
        assert!(
            price["purchase_price_coins"]
                .as_u64()
                .is_some_and(|value| value > 0),
            "item {id} has a positive purchase price"
        );
        for field in [
            "purchase_price_coins",
            "member_price_coins",
            "sell_price_coins",
        ] {
            assert_eq!(item[field], price[field], "item {id} price field {field}");
        }
        assert_eq!(item["price_source"]["kind"], "stores.json");
    }

    // The same three ids are the client-readable coin component shop, opened through the
    // production interaction service.
    let mut interaction = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("the production NPC loader accepts Bruno");
    let (zone, position) = interaction
        .location(BRUNO)
        .expect("Bruno has a positioned spawn");
    let menu = interaction
        .interact(BRUNO, zone, position, None)
        .expect("Bruno opens his menu");
    let choice = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Shop)
        .expect("Bruno exposes the Shop action");
    let NpcSelection::Shop(window) = interaction
        .select(&npc_select(BRUNO, choice.id), zone, position, None)
        .expect("the Shop choice opens the merchant window")
    else {
        panic!("Bruno selected a non-shop action");
    };
    let ids: BTreeSet<i32> = window.items.iter().map(|item| item.item_id).collect();
    assert_eq!(ids, [WOODEN_PLANKS, THORNY_FASTENERS, SINGING_COAL].into());
    assert_eq!(window.items.len(), 3, "no fourth item in the coin window");
    assert!(
        window
            .stock_description
            .as_deref()
            .is_some_and(|text| text.contains("Thorny Fasteners")),
        "the coin window carries Bruno's recovered stock description"
    );
    for item in window.items {
        assert_eq!(item.purchase_price_coins, Some(UNIT_PRICE));
    }
}

#[test]
fn a_second_blacksmith_does_not_share_quest_or_vendor_state() {
    let fixture = Fixture::minimal(CHARACTER);
    let other = CHARACTER + 50;
    let smitty_a = spot(CHARACTER, SMITTY);
    let smitty_b = spot(other, SMITTY);
    let bruno_a = spot(CHARACTER, BRUNO);
    let bruno_b = spot(other, BRUNO);
    let mut a = blacksmith_game(&fixture, CHARACTER, 100);
    let mut b = blacksmith_game(&fixture, other, 100);
    let mut now = 5_000;

    accept_merchandise(&mut a, &smitty_a, &mut now);
    accept_merchandise(&mut b, &smitty_b, &mut now);
    now += 10;
    a.on_location(&bruno_a, now).expect("location event");
    b.on_location(&bruno_b, now).expect("location event");
    assert_eq!(counts(&a, MERCHANDISE), [1, 0]);
    assert_eq!(counts(&b, MERCHANDISE), [1, 0]);

    // B buys one of each; A's coins, items and counters must not move.
    merchant_list(&mut b, &bruno_b, now + 1);
    let a_before = saved(&a);
    let b_fastener = buy(&mut b, &bruno_b, 92_001, THORNY_FASTENERS, 1, now + 2);
    let b_plank = buy(&mut b, &bruno_b, 92_002, WOODEN_PLANKS, 1, now + 3);
    assert_eq!(result_status(&b_fastener), 1);
    assert_eq!(result_status(&b_plank), 1);
    assert_eq!(saved(&a), a_before, "B's sale moved A's state");

    // A buys four of each; B's state must not move.
    merchant_list(&mut a, &bruno_a, now + 4);
    let b_before = saved(&b);
    assert_eq!(
        result_status(&buy(&mut a, &bruno_a, 92_003, THORNY_FASTENERS, 4, now + 5)),
        1
    );
    assert_eq!(
        result_status(&buy(&mut a, &bruno_a, 92_004, WOODEN_PLANKS, 4, now + 6)),
        1
    );
    assert_eq!(saved(&b), b_before, "A's sale moved B's state");
    assert_eq!(coins(&a), 100 - 8 * UNIT_PRICE);
    assert_eq!(coins(&b), 100 - 2 * UNIT_PRICE);
    assert_eq!(count_definition(&a.state().inventory, THORNY_FASTENERS), 4);
    assert_eq!(count_definition(&b.state().inventory, THORNY_FASTENERS), 1);

    // A turns in; B's active quest and wallet are untouched.
    let mut a_now = now + 10;
    let b_active = saved(&b);
    finish_merchandise(&mut a, &smitty_a, &mut a_now);
    assert_eq!(saved(&b), b_active, "A's turn-in moved B's state");
    assert_eq!(quest_status(&b, MERCHANDISE), QuestStatus::Active);
    assert_eq!(coins(&b), 100 - 2 * UNIT_PRICE);

    // Each session pays its own single reward.
    let mut b_now = now + 20;
    finish_merchandise(&mut b, &smitty_b, &mut b_now);
    assert_eq!(profile_xp(&a, BLACKSMITH), 20);
    assert_eq!(profile_xp(&b, BLACKSMITH), 20);
    assert_eq!(coins(&a), 100 - 8 * UNIT_PRICE + 5);
    assert_eq!(coins(&b), 100 - 2 * UNIT_PRICE + 5);
}

#[test]
fn a_failing_quest_event_for_one_player_does_not_stop_the_other() {
    let fixture = Fixture::minimal(CHARACTER);
    let other = CHARACTER + 60;
    let smitty_a = spot(CHARACTER, SMITTY);
    let smitty_b = spot(other, SMITTY);
    let bruno_a = spot(CHARACTER, BRUNO);
    let bruno_b = spot(other, BRUNO);
    let mut a = blacksmith_game(&fixture, CHARACTER, 100);
    let mut b = blacksmith_game(&fixture, other, 100);
    let mut now = 6_000;

    accept_merchandise(&mut a, &smitty_a, &mut now);
    accept_merchandise(&mut b, &smitty_b, &mut now);
    now += 10;
    a.on_location(&bruno_a, now).expect("location event");
    b.on_location(&bruno_b, now).expect("location event");

    // A's stale events fail for A only: a turn-in with no end screen, a stale accept and a
    // malformed NPC request. A's full state is unchanged after every refusal.
    let before = snapshot(&a);
    let stale_turnin = send(&mut a, &smitty_a, &quest_end_reply(MERCHANDISE), now + 1);
    assert!(!stale_turnin.changed);
    assert_packet_refused_without_change(
        &mut a,
        &smitty_a,
        &quest_reply(MERCHANDISE, true),
        now + 2,
        "A stale accept",
    );
    assert!(
        a.handle_in_world(&[26, 0, 8, 0], &smitty_a, now + 3)
            .is_err(),
        "A malformed 26/8 must fail for A"
    );
    assert_unchanged(&a, &before, "A's refusals");

    // B still buys and turns in normally.
    merchant_list(&mut b, &bruno_b, now + 4);
    assert_eq!(
        result_status(&buy(&mut b, &bruno_b, 93_001, THORNY_FASTENERS, 4, now + 5)),
        1
    );
    assert_eq!(
        result_status(&buy(&mut b, &bruno_b, 93_002, WOODEN_PLANKS, 4, now + 6)),
        1
    );
    let mut b_now = now + 10;
    finish_merchandise(&mut b, &smitty_b, &mut b_now);
    assert_eq!(quest_status(&b, MERCHANDISE), QuestStatus::Completed);
    assert_eq!(profile_xp(&b, BLACKSMITH), 20);
    assert_eq!(coins(&b), 100 - 8 * UNIT_PRICE + 5);
}
