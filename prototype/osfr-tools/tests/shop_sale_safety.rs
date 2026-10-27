//! T529: the coin store never sells an item the server cannot make work.
//!
//! The buy is refused inside `CoinShop::buy_once`, so the 165/4 direct buy, the 66/43 vendor buy
//! and the service `buy` entry all share the gate. Type policy:
//! - type 2 (pet adoption item): sellable only when the bundled pets content binds the id through
//!   `PetContent::definition_for_item`. The client adopts it with `PetsHandler::SendPetSummonByItemId`
//!   (`00b60110`, 53/32), and only the species binding can turn it into a pet.
//! - type 19 (ride): never for coins. Client item-use switch `00a1ec00` case 0x13 enters the
//!   mount path (`00b06db0` logs `SpawnMountByItemDefinitionId: item %d`); rides are Station Cash
//!   or earned only (T230).
//! - type 16 (house deed/lot): never. Every recorded deed is Type 16 (notes/Autopilot/L046.md)
//!   and OSFR's 9a8326f forces `ForceDisablePreview` on these after the housing preview
//!   soft-lock; housing is lead-owned.
//! - type 29 (furniture): stays on sale (R). OSFR keeps 185 coin furniture rows and the items
//!   wait in the bag until housing decoration lands.
//!
//! T794: a bought item counts for a "collect item" quest objective, exactly once per purchase, and a
//! refused buy, a sell or a buy-back counts for nothing (see `buy_credits_the_collect_objective`).
//!
//! Every request here goes through the real gateway dispatch: 26/8 + 26/10 open the shipped
//! Gloria vendor, 165/4 is the client's direct coin buy and 66/43 the vendor buy. A refusal must
//! change neither the snapshot/save nor the buy-back ledger.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/round70_content.rs"]
mod round70_content;
#[path = "common/vendor_fixture.rs"]
mod vendor_fixture;

use gateway_vendor::{
    NOW, buy_back_packet, buy_back_result, coin_buy_packet, completions, connect_with,
    count_definition, fixture_root, inner_packets, interact, inventory_of, merchant_status, select,
    sell_packet, send, vendor_buy_packet, wrap,
};
use osfr_tools::{
    coin_shop::{self, CoinShop, StoreCatalogue},
    game_store::Snapshot,
    gameplay::GameplayState,
    gateway::{Gateway, GatewayOutcome},
    inventory::Inventory,
    npc_interaction::decode_coin_merchant_list_165_10,
    pets::PetService,
    quest_engine::QuestState,
    station_cash::{DEED_ITEM_TYPE, Wallet},
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use vendor_fixture::{SEEDED_ITEM, VendorFixture};

/// Gloria's shipped vendor record: three type-2 Pet Pals with coin prices (bound pets since T454).
const GLORIA: u64 = 9_500_000_246;
const GLORIA_NPC_FILE: &str = "cobblestone_village.json";
/// A fixture-only Type-2 row: since T454 every client Type-2 item (528) is an adoption item, so the
/// unbound case is defined in the fixture copy by hand (see `unbound_pet_definition`).
const UNBOUND_PET: i32 = 1_999_999_002;
/// Fritz, one of the 16 adoption item ids `pets.json` binds.
const BOUND_PET: i32 = 4_324;
/// Skeletal Dragon (`mounts.json` mount 715, item 10,530).
const RIDE: i32 = 10_530;
/// Small Wilds House, a type-16 house deed.
const HOUSE_LOT: i32 = 2_213;
/// A type-29 furniture row that already has a coin price in the shipped stores.
const FURNITURE: i32 = 10_472;
/// An id no shipped catalogue defines; the unreadable-definition test gives it a truncated row.
const TRUNCATED: i32 = 1_999_999_001;
const START_COINS: u32 = 100_000;
/// T794: the synthetic collect quest the fixture installs.
const COLLECT_QUEST: u32 = 9_900_794;

/// Every vendor-stocked row the coin store refuses on the integration tip, with the reason it is
/// blocked. The content test fails if this list and the shipped `vendors.json` disagree in either
/// direction, so binding a Pet Pal in pets content must remove its row here.
/// T454 bound every Pet Pal Gloria sells, so no shipped vendor row is blocked any more.
const REVIEWED_UNSELLABLE: &[(i32, &str)] = &[];

fn data_content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("content file reads"))
        .unwrap_or_else(|error| panic!("{} parses: {error}", path.display()))
}

fn gloria_fixture(label: &str) -> VendorFixture {
    VendorFixture::for_vendor(&fixture_root(), label, GLORIA, GLORIA_NPC_FILE)
}

fn connect(fixture: &VendorFixture, state: GameplayState) -> Gateway {
    connect_with(
        fixture.reference(),
        fixture.character(),
        fixture.content(),
        state,
    )
}

/// The production shop entry: 26/8 opens the menu and 26/10 selects Gloria's Shop option.
fn open_shop(fixture: &VendorFixture, gateway: &mut Gateway) -> Vec<Vec<u8>> {
    send(gateway, &interact(GLORIA));
    send(gateway, &select(GLORIA, fixture.option()))
}

/// The item ids the real 165/10 merchant window advertises.
fn advertised_items(replies: &[Vec<u8>]) -> Vec<u32> {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
        .expect("the Shop choice sends the 165/10 coin merchant list");
    decode_coin_merchant_list_165_10(packet)
        .expect("165/10 matches the client reader")
        .entries
        .iter()
        .map(|entry| entry.item_id)
        .collect()
}

fn buy_165_4(transaction: u32, item: i32) -> Vec<u8> {
    coin_buy_packet(GLORIA, transaction, item, 1)
}

/// Sell the fixture's seeded item through the real 165/5 path, leaving one buy-back entry.
fn seed_buy_back(gateway: &mut Gateway) -> u32 {
    let replies = send(gateway, &sell_packet(0, 1, 1));
    let sale = completions(&replies)
        .into_iter()
        .next()
        .expect("the seeded item sells");
    assert_eq!(sale.result, coin_shop::RESULT_SUCCESS);
    sale.transaction
}

fn handled(gateway: &mut Gateway, packet: &[u8]) -> (Vec<Vec<u8>>, GatewayOutcome) {
    let outcome = gateway
        .handle_at(&wrap(packet), NOW)
        .expect("gateway handles the request");
    (inner_packets(&outcome), outcome)
}

fn assert_unbound_pet_item_is_still_unbound() {
    let pets = PetService::bundled_content().expect("the bundled pets content loads");
    assert!(
        pets.definition_for_item(UNBOUND_PET).is_none(),
        "pets content now binds item {UNBOUND_PET}, so the coin store sells it; pick another \
         unbound type-2 row for this refusal and update REVIEWED_UNSELLABLE"
    );
}

/// A readable Type-2 definition row, in the layout `Inventory::presentation` reads: the id at +0,
/// name id at +4, icon at +12, the model and texture counted strings, then GenderUsage and Type.
/// The loader requires the first word to be the definition id.
fn unbound_pet_definition() -> Vec<u8> {
    let mut bytes = vec![0u8; 46];
    bytes[0..4].copy_from_slice(&UNBOUND_PET.to_le_bytes());
    for _ in 0..2 {
        bytes.extend(0u32.to_le_bytes()); // empty model and texture strings
    }
    let mut fields = [0u8; 48];
    fields[4..8].copy_from_slice(&2i32.to_le_bytes()); // Type 2: a pet adoption item
    bytes.extend(fields);
    bytes.extend(0u32.to_le_bytes()); // empty tint alias
    bytes
}

/// Add the fixture-only type-2 row and price it in the fixture's own vendor copy.
fn stock_unbound_pet(fixture: &VendorFixture) {
    fixture.define_item(UNBOUND_PET, &unbound_pet_definition());
    fixture.stock_price_coins(&[(UNBOUND_PET, 10_000)]);
    let inventory = Inventory::load(fixture.content(), None).expect("fixture inventory loads");
    assert_eq!(
        inventory
            .presentation(UNBOUND_PET)
            .expect("the synthetic row is readable")
            .item_type,
        2,
        "the fixture-only row is a pet adoption item"
    );
}

fn assert_unchanged(gateway: &Gateway, before: &Snapshot, label: &str) {
    assert_eq!(
        &gateway.snapshot(),
        before,
        "{label}: the refused buy changed the snapshot"
    );
}

#[test]
fn an_unbound_pet_item_is_refused_through_the_real_165_4_buy() {
    assert_unbound_pet_item_is_still_unbound();
    let fixture = gloria_fixture("t529-a-165-4");
    // No shipped vendor stocks an unbound pet item, so this copy defines and stocks one.
    stock_unbound_pet(&fixture);
    let mut gateway = connect(&fixture, fixture.state(START_COINS, false));
    let replies = open_shop(&fixture, &mut gateway);
    assert!(
        !advertised_items(&replies).contains(&(UNBOUND_PET as u32)),
        "the merchant window advertised a pet item the shop refuses"
    );

    let sale = seed_buy_back(&mut gateway);
    let before = gateway.snapshot();
    let before_inventory = inventory_of(&gateway);
    let refused_transaction = 0x7000_0001;
    let (replies, outcome) = handled(&mut gateway, &buy_165_4(refused_transaction, UNBOUND_PET));
    assert!(
        !outcome.gameplay_changed,
        "a refused buy requested a gameplay save"
    );
    assert!(!outcome.state_changed, "a refused buy changed the session");
    let done = completions(&replies);
    assert_eq!(done.len(), 1, "exactly one 165/6 completion");
    assert_eq!(done[0].result, coin_shop::RESULT_INVALID_ITEM);
    assert_eq!(done[0].kind, 1, "the completion is a buy");
    assert_eq!(done[0].definition, UNBOUND_PET);
    assert_unchanged(&gateway, &before, "165/4");
    let after_inventory = inventory_of(&gateway);
    assert_eq!(after_inventory.coins, before_inventory.coins);
    assert_eq!(count_definition(&after_inventory, UNBOUND_PET), 0);

    // No ledger entry was added for the refused transaction, and the entry from the seed sale
    // is still readable.
    let probe = send(&mut gateway, &buy_back_packet(refused_transaction, 0));
    assert_eq!(buy_back_result(&probe), coin_shop::RESULT_FAILURE);
    let restored = send(&mut gateway, &buy_back_packet(sale, 0));
    assert_eq!(
        buy_back_result(&restored),
        coin_shop::RESULT_SUCCESS,
        "the refusal consumed an existing buy-back entry"
    );
    assert_eq!(count_definition(&inventory_of(&gateway), SEEDED_ITEM), 1);
}

#[test]
fn an_unbound_pet_item_is_refused_through_the_real_66_43_buy() {
    assert_unbound_pet_item_is_still_unbound();
    let fixture = gloria_fixture("t529-a-66-43");
    // No shipped vendor stocks an unbound pet item, so this copy defines and stocks one.
    stock_unbound_pet(&fixture);
    let mut gateway = connect(&fixture, fixture.state(START_COINS, false));
    open_shop(&fixture, &mut gateway);

    let sale = seed_buy_back(&mut gateway);
    let before = gateway.snapshot();
    let before_inventory = inventory_of(&gateway);
    let refused_transaction = 0x7100_0001;
    let packet = vendor_buy_packet(refused_transaction, UNBOUND_PET, 1);
    let (replies, outcome) = handled(&mut gateway, &packet);
    assert!(
        !outcome.gameplay_changed,
        "a refused buy requested a gameplay save"
    );
    assert!(!outcome.state_changed, "a refused buy changed the session");
    assert_eq!(replies.len(), 1, "one 66/44 answer");
    assert_eq!(merchant_status(&replies[0]), 0, "the client saw success");
    assert_unchanged(&gateway, &before, "66/43");
    assert_eq!(inventory_of(&gateway).coins, before_inventory.coins);
    assert_eq!(count_definition(&inventory_of(&gateway), UNBOUND_PET), 0);

    // `buy_vendor` does not cache refusals: the same transaction id retried is refused again and
    // still changes nothing.
    let (retry, _) = handled(&mut gateway, &packet);
    assert_eq!(merchant_status(&retry[0]), 0);
    assert_unchanged(&gateway, &before, "66/43 retry");

    let probe = send(&mut gateway, &buy_back_packet(refused_transaction, 0));
    assert_eq!(buy_back_result(&probe), coin_shop::RESULT_FAILURE);
    let restored = send(&mut gateway, &buy_back_packet(sale, 0));
    assert_eq!(buy_back_result(&restored), coin_shop::RESULT_SUCCESS);
}

#[test]
fn a_pet_item_bound_by_pets_content_is_sold() {
    let fixture = gloria_fixture("t529-b");
    fixture.stock_price_coins(&[(BOUND_PET, 432)]);
    let pets = PetService::bundled_content().expect("the bundled pets content loads");
    assert!(
        pets.definition_for_item(BOUND_PET).is_some(),
        "item {BOUND_PET} is no longer a pets.json adoption item; pick another bound id"
    );
    let inventory = Inventory::load(fixture.content(), None).expect("fixture inventory loads");
    assert_eq!(
        inventory
            .presentation(BOUND_PET)
            .expect("the bound pet item is defined")
            .item_type,
        2,
        "the bound adoption item is not a type-2 pet row"
    );

    let mut gateway = connect(&fixture, fixture.state(1_000, false));
    let replies = open_shop(&fixture, &mut gateway);
    assert!(
        advertised_items(&replies).contains(&(BOUND_PET as u32)),
        "the merchant window hid a bound pet item"
    );
    let (replies, outcome) = handled(&mut gateway, &buy_165_4(1, BOUND_PET));
    assert!(outcome.gameplay_changed, "a successful buy requests a save");
    let done = completions(&replies);
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].result, coin_shop::RESULT_SUCCESS);
    assert_eq!(done[0].definition, BOUND_PET);
    assert_eq!(done[0].price, 432);
    assert_eq!(inventory_of(&gateway).coins, 1_000 - 432);
    // T772: the bought item is adopted at once: a pet in the roster (and its 53/5), none in the bag.
    assert_eq!(count_definition(&inventory_of(&gateway), BOUND_PET), 0);
    assert_adopted(&gateway, &replies, 1);
}

/// T772: the saved roster's pets, from the gateway's save document.
fn roster(gateway: &Gateway) -> Vec<Value> {
    gateway
        .snapshot()
        .pets
        .and_then(|document| document["pets"].as_array().cloned())
        .unwrap_or_default()
}

/// T772: `count` pets adopted from `BOUND_PET`, and the reply carried the 53/5 roster.
fn assert_adopted(gateway: &Gateway, replies: &[Vec<u8>], count: usize) {
    let pets = roster(gateway);
    assert_eq!(pets.len(), count, "the roster holds every bought pet");
    assert!(
        pets.iter()
            .all(|pet| pet["adoption_item_definition_id"] == BOUND_PET),
        "each pet records the item it was adopted from"
    );
    assert!(
        replies.iter().any(|p| p.starts_with(&[53, 0, 5])),
        "the Pets window gets the 53/5 roster at once"
    );
}

#[test]
fn a_pet_item_bought_through_the_66_43_vendor_buy_is_adopted() {
    let fixture = gloria_fixture("t772-66-43");
    fixture.stock_price_coins(&[(BOUND_PET, 432)]);
    let mut gateway = connect(&fixture, fixture.state(1_000, false));
    open_shop(&fixture, &mut gateway);
    let (replies, outcome) = handled(&mut gateway, &vendor_buy_packet(7, BOUND_PET, 1));
    assert!(
        outcome.gameplay_changed,
        "the adoption is saved with the buy"
    );
    assert_eq!(inventory_of(&gateway).coins, 1_000 - 432);
    assert_eq!(count_definition(&inventory_of(&gateway), BOUND_PET), 0);
    assert_adopted(&gateway, &replies, 1);
}

#[test]
fn a_full_roster_refuses_a_pet_buy_before_money_moves() {
    let fixture = gloria_fixture("t772-full");
    fixture.stock_price_coins(&[(BOUND_PET, 10)]);
    let mut gateway = connect(&fixture, fixture.state(10_000, false));
    open_shop(&fixture, &mut gateway);
    // Fill the roster (50 pets) through the real buy.
    for n in 0..50 {
        let (replies, _) = handled(&mut gateway, &buy_165_4(n, BOUND_PET));
        assert_eq!(
            completions(&replies)[0].result,
            coin_shop::RESULT_SUCCESS,
            "buy {n}"
        );
    }
    assert_eq!(roster(&gateway).len(), 50);
    let before = gateway.snapshot();
    let (replies, outcome) = handled(&mut gateway, &buy_165_4(99, BOUND_PET));
    let done = completions(&replies);
    assert_eq!(done[0].result, coin_shop::RESULT_INVALID_QUANTITY);
    assert!(!outcome.gameplay_changed && !outcome.state_changed);
    assert_unchanged(&gateway, &before, "full roster");
    assert_eq!(inventory_of(&gateway).coins, 10_000 - 50 * 10);
    assert_eq!(count_definition(&inventory_of(&gateway), BOUND_PET), 0);
}

#[test]
fn rides_and_house_lots_are_refused_while_furniture_is_sold() {
    let fixture = gloria_fixture("t529-c");
    fixture.stock_price_coins(&[(RIDE, 650), (HOUSE_LOT, 700), (FURNITURE, 200)]);
    let inventory = Inventory::load(fixture.content(), None).expect("fixture inventory loads");
    for (item, expected) in [(RIDE, 19), (HOUSE_LOT, DEED_ITEM_TYPE), (FURNITURE, 29)] {
        assert_eq!(
            inventory
                .presentation(item)
                .expect("the fixture item is defined")
                .item_type,
            expected,
            "item {item} no longer has the type this test exercises"
        );
    }

    let mut gateway = connect(&fixture, fixture.state(1_000, false));
    let replies = open_shop(&fixture, &mut gateway);
    let advertised = advertised_items(&replies);
    assert!(!advertised.contains(&(RIDE as u32)));
    assert!(!advertised.contains(&(HOUSE_LOT as u32)));
    assert!(advertised.contains(&(FURNITURE as u32)));

    for (transaction, item) in [(11, RIDE), (12, HOUSE_LOT)] {
        let before = gateway.snapshot();
        let (replies, outcome) = handled(&mut gateway, &buy_165_4(transaction, item));
        assert!(
            !outcome.gameplay_changed,
            "a refused buy requested a gameplay save"
        );
        assert!(!outcome.state_changed);
        let done = completions(&replies);
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].result, coin_shop::RESULT_INVALID_ITEM);
        assert_unchanged(&gateway, &before, "type blocked buy");
        assert_eq!(inventory_of(&gateway).coins, 1_000);
        assert_eq!(count_definition(&inventory_of(&gateway), item), 0);
    }

    // Type-29 furniture stays on sale for coins (R).
    let (replies, outcome) = handled(&mut gateway, &buy_165_4(13, FURNITURE));
    assert!(outcome.gameplay_changed);
    assert_eq!(completions(&replies)[0].result, coin_shop::RESULT_SUCCESS);
    assert_eq!(inventory_of(&gateway).coins, 800);
    assert_eq!(count_definition(&inventory_of(&gateway), FURNITURE), 1);
}

/// The shipped content gate: every item any vendor stocks is either sellable for coins by the
/// current rules or listed above with its reason. Fails in both directions so binding a Pet Pal
/// in pets content, restocking a blocked row, or changing an item's type all need a deliberate
/// update here.
#[test]
fn every_shipped_vendor_item_is_sellable_or_reviewed_unsellable() {
    let data = data_content();
    let inventory_content = round70_content::source();
    let inventory = Inventory::load(&inventory_content, None).expect("inventory catalogue loads");
    let catalogue =
        StoreCatalogue::load(&data.join("stores.json")).expect("the shipped store catalogue loads");
    let shop = CoinShop::new(
        &inventory_content,
        inventory,
        catalogue,
        100,
        None,
        Wallet::default(),
    )
    .expect("the shop loads with the shipped content");

    let vendors = read_json(&data.join("vendors.json"));
    let mut blocked = BTreeMap::new();
    // The server stocks a vendor from `item_ids` (`npc_interaction.rs` `VendorRecord`), not from
    // the wiki-shaped `items` rows; NPCs 9000000024 and 9000000027 have an `items` row and no
    // `item_ids`, so only the loader's list counts here.
    for vendor in vendors["vendors"]
        .as_array()
        .expect("vendor records are an array")
    {
        for item in vendor["item_ids"]
            .as_array()
            .expect("vendor item_ids is an array")
        {
            let id = i32::try_from(item.as_i64().expect("vendor item ID is an integer"))
                .expect("vendor item ID fits i32");
            if let Some(reason) = shop.sale_block(id) {
                blocked.insert(id, reason);
            }
        }
    }
    let reviewed: BTreeMap<i32, &str> = REVIEWED_UNSELLABLE.iter().copied().collect();
    assert_eq!(
        blocked, reviewed,
        "the coin store's blocked vendor rows changed; a bound Pet Pal must leave \
         REVIEWED_UNSELLABLE and a newly blocked row must join it"
    );
}

/// A definition that is present but truncated before its Type field cannot be classified, so the
/// shop refuses it with the invalid-item reply instead of selling or erroring. The row is added
/// to the fixture's own `inventory.json` copy with a valid hash, so it loads like a shipped row.
#[test]
fn a_priced_item_with_an_unreadable_definition_is_refused_and_not_advertised() {
    let fixture = gloria_fixture("t529-d");
    let mut row = TRUNCATED.to_le_bytes().to_vec();
    row.extend([0; 8]); // far short of the model strings that precede Type
    fixture.define_item(TRUNCATED, &row);
    fixture.stock_price_coins(&[(TRUNCATED, 50), (FURNITURE, 200)]);
    let inventory = Inventory::load(fixture.content(), None).expect("fixture inventory loads");
    assert!(inventory.has_definition(TRUNCATED), "the row is defined");
    assert!(
        inventory.presentation(TRUNCATED).is_err(),
        "the row must be unreadable, not merely unknown"
    );

    let mut gateway = connect(&fixture, fixture.state(1_000, false));
    let advertised = advertised_items(&open_shop(&fixture, &mut gateway));
    assert!(!advertised.contains(&(TRUNCATED as u32)));
    assert!(
        advertised.contains(&(FURNITURE as u32)),
        "readable rows stay"
    );

    let before = gateway.snapshot();
    let (replies, outcome) = handled(&mut gateway, &buy_165_4(21, TRUNCATED));
    assert!(!outcome.gameplay_changed && !outcome.state_changed);
    let done = completions(&replies);
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].result, coin_shop::RESULT_INVALID_ITEM);
    assert_unchanged(&gateway, &before, "unreadable definition");
    assert_eq!(inventory_of(&gateway).coins, 1_000);
    assert_eq!(count_definition(&inventory_of(&gateway), TRUNCATED), 0);
}

/// T794: the collect objective's count, read from the gateway's save document.
fn collected(gateway: &Gateway) -> u32 {
    let state = gateway.snapshot().gameplay.expect("gameplay state exists");
    state.npc_quests.expect("the quest save exists").quests[&COLLECT_QUEST].counts[0]
}

/// T794: a Gloria gateway selling `stock` (item, price) and holding an accepted "collect 5 of
/// `item`" quest with `counted` already earned. The fixture owns the content directory the
/// gateway reads at every buy, so it is returned and must outlive the gateway.
fn collect_gateway(
    label: &str,
    item: i32,
    stock: (i32, u32),
    coins: u32,
    counted: u32,
) -> (VendorFixture, Gateway) {
    let fixture = gloria_fixture(label);
    fixture.stock_price_coins(&[stock]);
    let mut quests: QuestState = fixture.install_collect_quest(COLLECT_QUEST, GLORIA, item, 5);
    quests.quests.get_mut(&COLLECT_QUEST).unwrap().counts[0] = counted;
    let mut state = fixture.state(coins, false);
    state.npc_quests = Some(quests);
    let mut gateway = connect(&fixture, state);
    open_shop(&fixture, &mut gateway);
    (fixture, gateway)
}

/// The two buy requests the client sends: 165/4 (coin store) and 66/43 (vendor).
const BUYS: [(&str, fn(u32, i32, u32) -> Vec<u8>); 2] = [
    ("165-4", |transaction, item, quantity| {
        coin_buy_packet(GLORIA, transaction, item, quantity)
    }),
    ("66-43", vendor_buy_packet),
];

/// T794: both buy paths credit the objective by the bought quantity, once. (A second raise on the
/// same purchase would show here as double the quantity.)
#[test]
fn buy_credits_the_collect_objective_exactly_once() {
    for (label, buy) in BUYS {
        let (_fixture, mut gateway) = collect_gateway(
            &format!("t794-buy-{label}"),
            FURNITURE,
            (FURNITURE, 200),
            1_000,
            0,
        );
        let (_, outcome) = handled(&mut gateway, &buy(1, FURNITURE, 2));
        assert!(outcome.gameplay_changed, "{label}: the buy saves");
        assert_eq!(inventory_of(&gateway).coins, 600, "{label}: the buy paid");
        assert_eq!(collected(&gateway), 2, "{label}: two bought, two counted");
        handled(&mut gateway, &buy(2, FURNITURE, 1));
        assert_eq!(collected(&gateway), 3, "{label}: the next buy adds its own");
    }
}

/// T794: a refused buy (here: not enough coins) credits nothing and changes nothing. The player
/// already holds one of the objective's item (`SEEDED_ITEM`), so a count taken against a stale or
/// empty baseline would show as a credit.
#[test]
fn a_refused_buy_credits_nothing() {
    for (label, buy) in BUYS {
        let (_fixture, mut gateway) = collect_gateway(
            &format!("t794-refused-{label}"),
            SEEDED_ITEM,
            (SEEDED_ITEM, 200),
            150,
            0,
        );
        let before = gateway.snapshot();
        let (_, outcome) = handled(&mut gateway, &buy(1, SEEDED_ITEM, 1));
        assert!(
            !outcome.gameplay_changed && !outcome.state_changed,
            "{label}"
        );
        assert_unchanged(&gateway, &before, label);
        assert_eq!(collected(&gateway), 0, "{label}: a refused buy counted");
    }
}

/// T794: a bought pet item is adopted at once (T772), so the bag never holds it and nothing counts.
#[test]
fn a_bought_pet_item_credits_nothing() {
    for (label, buy) in BUYS {
        let (_fixture, mut gateway) = collect_gateway(
            &format!("t794-pet-{label}"),
            BOUND_PET,
            (BOUND_PET, 432),
            1_000,
            0,
        );
        let (replies, _) = handled(&mut gateway, &buy(1, BOUND_PET, 1));
        assert_adopted(&gateway, &replies, 1);
        assert_eq!(collected(&gateway), 0, "{label}: an adopted pet counted");
    }
}

/// T794: a sell lowers no count (`CollectItem` only counts gains), and a buy-back hands back what
/// the sale took at the sale price, so sell + buy-back must not count: it would loop the objective
/// for free. The quest starts at 1 of 5 with the fixture's seeded stack (never counted) in the bag.
#[test]
fn selling_and_buying_back_leave_the_count_alone() {
    let (_fixture, mut gateway) =
        collect_gateway("t794-sell", SEEDED_ITEM, (FURNITURE, 200), 1_000, 1);
    let sale = seed_buy_back(&mut gateway);
    assert_eq!(count_definition(&inventory_of(&gateway), SEEDED_ITEM), 0);
    assert_eq!(collected(&gateway), 1, "a sale lowered the count");
    let restored = send(&mut gateway, &buy_back_packet(sale, 0));
    assert_eq!(buy_back_result(&restored), coin_shop::RESULT_SUCCESS);
    assert_eq!(count_definition(&inventory_of(&gateway), SEEDED_ITEM), 1);
    assert_eq!(
        inventory_of(&gateway).coins,
        1_000,
        "the round trip is free"
    );
    assert_eq!(collected(&gateway), 1, "a buy-back counted as a new item");
}
