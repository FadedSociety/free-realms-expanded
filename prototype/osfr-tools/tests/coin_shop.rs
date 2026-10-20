use osfr_tools::{
    coin_shop::{self, CoinShop, StoreCatalogue},
    inventory::{Inventory, InventoryState},
};
use std::path::{Path, PathBuf};

fn inventory_content() -> PathBuf {
    std::env::var_os("OSFR_COIN_SHOP_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-gameplay")
        })
}

fn catalogue_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json")
}

fn inventory_with_coins(directory: &Path, coins: u32) -> Inventory {
    let loaded = Inventory::load(directory, None).expect("prepared item content loads");
    let mut state = loaded.state().clone();
    state.coins = coins;
    Inventory::load(directory, Some(state)).expect("coin balance is valid")
}

fn shop(directory: &Path, inventory: Inventory, capacity: usize) -> CoinShop {
    let catalogue = StoreCatalogue::load(&catalogue_path()).expect("store catalogue loads");
    CoinShop::new(directory, inventory, catalogue, capacity, None).expect("shop state loads")
}

fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

fn merchant_status(result: &coin_shop::CoinShopOutcome) -> u32 {
    let packet = result.replies.last().expect("vendor reply exists");
    assert_eq!(packet.len(), 56);
    assert_eq!(&packet[..4], &[66, 0, 44, 0]);
    u32::from_le_bytes(packet[48..52].try_into().unwrap())
}

#[test]
fn buy_debits_exact_cost_and_retry_is_idempotent() {
    let directory = inventory_content();
    let mut shop = shop(
        &directory,
        inventory_with_coins(&directory, 100),
        usize::MAX,
    );
    let original_count = count_definition(shop.inventory().state(), 8);

    let mut request = vec![165, 0, 4, 0];
    request.extend(0u64.to_le_bytes());
    request.extend(71001u32.to_le_bytes());
    request.extend(8i32.to_le_bytes());
    request.extend(1u32.to_le_bytes());
    request.extend(0u32.to_le_bytes());
    let bought = shop.handle(&request).unwrap();
    assert_eq!(bought.result_code, coin_shop::RESULT_SUCCESS);
    assert!(bought.changed);
    assert_eq!(shop.inventory().state().coins, 85);
    assert_eq!(
        count_definition(shop.inventory().state(), 8),
        original_count + 1
    );
    let once = shop.inventory().state().clone();

    let retry = shop.handle(&request).unwrap();
    assert_eq!(retry.result_code, coin_shop::RESULT_SUCCESS);
    assert!(!retry.changed);
    assert_eq!(shop.inventory().state(), &once);
}

#[test]
fn member_purchase_uses_catalogued_discounted_price() {
    let directory = inventory_content();
    let mut shop = shop(
        &directory,
        inventory_with_coins(&directory, 100),
        usize::MAX,
    );
    shop.set_member_status(true);

    let result = shop.buy(71005, 8, 1).unwrap();

    assert_eq!(result.result_code, coin_shop::RESULT_SUCCESS);
    assert_eq!(shop.inventory().state().coins, 87);
}

#[test]
fn insufficient_coins_returns_client_result_without_mutation() {
    let directory = inventory_content();
    let mut shop = shop(&directory, inventory_with_coins(&directory, 5), usize::MAX);
    let before = shop.inventory().state().clone();

    let result = shop.buy(71002, 8, 1).unwrap();

    assert_eq!(result.result_code, coin_shop::RESULT_INSUFFICIENT_COINS);
    assert!(!result.changed);
    assert_eq!(shop.inventory().state(), &before);
}

#[test]
fn full_bag_returns_service_result_without_mutation() {
    let directory = inventory_content();
    let loaded = Inventory::load(&directory, None).expect("prepared item content loads");
    let mut state = loaded.state().clone();
    state.coins = 100;
    state.items.clear();
    for equipped in &mut state.equipment {
        equipped.item_id = 0;
    }
    let inventory = Inventory::load(&directory, Some(state)).expect("empty bag is valid");
    let mut shop = shop(&directory, inventory, 0);
    let before = shop.inventory().state().clone();

    let result = shop.buy(71003, 8, 1).unwrap();

    assert_eq!(result.result_code, coin_shop::RESULT_SERVICE_UNAVAILABLE);
    assert!(!result.changed);
    assert_eq!(shop.inventory().state(), &before);
}

#[test]
fn unlisted_item_returns_invalid_item_without_mutation() {
    let directory = inventory_content();
    let mut shop = shop(
        &directory,
        inventory_with_coins(&directory, 100),
        usize::MAX,
    );
    let before = shop.inventory().state().clone();

    let result = shop.buy(71004, i32::MAX, 1).unwrap();

    assert_eq!(result.result_code, coin_shop::RESULT_INVALID_ITEM);
    assert!(!result.changed);
    assert_eq!(shop.inventory().state(), &before);
}

#[test]
fn sell_then_buy_back_restores_item_and_coin_balance() {
    let directory = inventory_content();
    let mut inventory = inventory_with_coins(&directory, 100);
    let mut state = inventory.state().clone();
    state.items.clear();
    for equipped in &mut state.equipment {
        equipped.item_id = 0;
    }
    inventory = Inventory::load(&directory, Some(state)).expect("empty inventory is valid");
    inventory.grant(0, 8, 1).expect("test item grant succeeds");
    let item_instance_id = inventory
        .state()
        .items
        .iter()
        .find(|item| item.definition_id == 8)
        .expect("catalogued item is owned")
        .id;
    let mut shop = shop(&directory, inventory, usize::MAX);

    let mut sell_request = vec![165, 0, 5, 0];
    sell_request.extend(0u64.to_le_bytes());
    sell_request.extend(item_instance_id.to_le_bytes());
    sell_request.extend(1u32.to_le_bytes());
    let sold = shop.handle(&sell_request).unwrap();
    assert_eq!(sold.result_code, coin_shop::RESULT_SUCCESS);
    assert_eq!(shop.inventory().state().coins, 107);
    assert_eq!(count_definition(shop.inventory().state(), 8), 0);

    let mut buy_back_request = vec![165, 0, 12, 0];
    buy_back_request.extend(sold.transaction_id.to_le_bytes());
    buy_back_request.extend(0u32.to_le_bytes());
    let bought_back = shop.handle(&buy_back_request).unwrap();
    assert_eq!(bought_back.result_code, coin_shop::RESULT_SUCCESS);
    assert_eq!(shop.inventory().state().coins, 100);
    assert_eq!(count_definition(shop.inventory().state(), 8), 1);
}

#[test]
fn marketplace_purchase_is_disabled_and_does_not_change_inventory() {
    let directory = inventory_content();
    let mut shop = shop(
        &directory,
        inventory_with_coins(&directory, 100),
        usize::MAX,
    );
    let before = shop.inventory().state().clone();
    let request = [66u16.to_le_bytes(), 3u16.to_le_bytes()].concat();

    let result = shop
        .handle(&request)
        .expect("disabled order request receives a reply");

    assert_eq!(result.result_code, coin_shop::MARKETPLACE_RESULT_DISABLED);
    assert!(!result.changed);
    assert_eq!(shop.inventory().state(), &before);
    let response = result.replies.first().expect("client receives failure");
    assert_eq!(u16::from_le_bytes(response[0..2].try_into().unwrap()), 66);
    assert_eq!(u16::from_le_bytes(response[2..4].try_into().unwrap()), 4);
    assert_eq!(i32::from_le_bytes(response[4..8].try_into().unwrap()), 0);
    assert!(String::from_utf8_lossy(response).contains("Not available in this restoration."));

    let fallback = shop.handle(&[66, 0, 43, 0]).unwrap();
    assert_eq!(fallback.result_code, coin_shop::MARKETPLACE_RESULT_DISABLED);
    assert_eq!(merchant_status(&fallback), 0);
}

#[test]
fn marketplace_handshake_replies_and_keeps_payment_sources_off() {
    let directory = inventory_content();
    let mut shop = shop(
        &directory,
        inventory_with_coins(&directory, 100),
        usize::MAX,
    );
    let initial = shop.marketplace_initial_packets();
    assert_eq!(initial[0], vec![66, 0, 24, 0, 1]);
    assert_eq!(initial[1], vec![66, 0, 33, 0, 0, 0]);

    for (request_opcode, reply_opcode) in [
        (10u16, 11u16),
        (12, 13),
        (14, 15),
        (16, 17),
        (18, 19),
        (20, 21),
        (22, 23),
        (25, 26),
    ] {
        let mut request = [66u16.to_le_bytes(), request_opcode.to_le_bytes()].concat();
        if matches!(request_opcode, 16 | 20 | 25) {
            // The client appends its locale as an i32 length and text (62 native traces).
            request.extend(5i32.to_le_bytes());
            request.extend(b"en_US");
        }
        let response = shop.handle(&request).unwrap();
        let packet = response.replies.first().expect("handshake response exists");
        assert_eq!(u16::from_le_bytes(packet[0..2].try_into().unwrap()), 66);
        assert_eq!(
            u16::from_le_bytes(packet[2..4].try_into().unwrap()),
            reply_opcode
        );
        assert!(!response.changed);
    }
}

/// Regression (2026-10-15 audit): the shop is created at login, but purchases
/// must use the live inventory, or coins/items earned since login are lost.
#[test]
fn purchase_uses_the_synced_live_inventory() {
    let directory = inventory_content();
    let mut shop = shop(&directory, inventory_with_coins(&directory, 5), usize::MAX);
    shop.sync_inventory(inventory_with_coins(&directory, 100));
    let result = shop.buy(71001, 8, 1).unwrap();
    assert_eq!(result.result_code, coin_shop::RESULT_SUCCESS);
    assert_eq!(shop.inventory().state().coins, 85);
}

#[test]
fn vendor_buy_debits_grants_and_retries_without_replaying() {
    let directory = inventory_content();
    let mut shop = shop(
        &directory,
        inventory_with_coins(&directory, 100),
        usize::MAX,
    );
    let before = count_definition(shop.inventory().state(), 8);

    let result = shop.buy_vendor(72001, 8, 1).unwrap();

    assert_eq!(result.result_code, coin_shop::RESULT_SUCCESS);
    assert!(result.changed);
    assert_eq!(merchant_status(&result), 1);
    assert_eq!(shop.inventory().state().coins, 85);
    assert_eq!(count_definition(shop.inventory().state(), 8), before + 1);
    let saved = shop.inventory().state().clone();

    let retry = shop.buy_vendor(72001, 8, 1).unwrap();
    assert_eq!(retry.result_code, coin_shop::RESULT_SUCCESS);
    assert!(!retry.changed);
    assert_eq!(retry.replies.len(), 1);
    assert_eq!(merchant_status(&retry), 1);
    assert_eq!(shop.inventory().state(), &saved);
}

#[test]
fn vendor_refusals_report_low_balance_unknown_item_and_full_stack() {
    let directory = inventory_content();
    let mut poor = shop(&directory, inventory_with_coins(&directory, 5), usize::MAX);
    let before = poor.inventory().state().clone();
    let low_balance = poor.buy_vendor(72002, 8, 1).unwrap();
    assert_eq!(
        low_balance.result_code,
        coin_shop::RESULT_INSUFFICIENT_COINS
    );
    assert_eq!(merchant_status(&low_balance), 0);
    assert!(!low_balance.changed);
    assert_eq!(poor.inventory().state(), &before);
    poor.sync_inventory(inventory_with_coins(&directory, 100));
    let recovered = poor.buy_vendor(72002, 8, 1).unwrap();
    assert_eq!(recovered.result_code, coin_shop::RESULT_SUCCESS);
    assert!(recovered.changed);
    assert_eq!(merchant_status(&recovered), 1);

    let mut stocked = shop(
        &directory,
        inventory_with_coins(&directory, 100),
        usize::MAX,
    );
    let unknown = stocked.buy_vendor(72003, i32::MAX, 1).unwrap();
    assert_eq!(unknown.result_code, coin_shop::RESULT_INVALID_ITEM);
    assert_eq!(merchant_status(&unknown), 0);
    assert!(!unknown.changed);

    let mut inventory = inventory_with_coins(&directory, 100);
    let mut state = inventory.state().clone();
    state.items.retain(|item| item.definition_id != 8);
    inventory = Inventory::load(&directory, Some(state)).expect("test inventory loads");
    inventory.grant(0, 8, 1).expect("max stack is filled");
    let mut full_stack = shop(&directory, inventory, usize::MAX);
    let before = full_stack.inventory().state().clone();
    let full = full_stack.buy_vendor(72004, 8, 2).unwrap();
    assert_eq!(full.result_code, coin_shop::RESULT_SERVICE_UNAVAILABLE);
    assert_eq!(merchant_status(&full), 0);
    assert!(!full.changed);
    assert_eq!(full_stack.inventory().state(), &before);
}
