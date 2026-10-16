//! Work item 084 acceptance: vendor selling and buyback transaction semantics.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - the shipped client's own packet
//!   constructors call `FUN_00c6e960(<opcode>)` under BaseCoinStore (165): 1, 2, 3, 4, 5, 6, 7, 8,
//!   9, 10, 11, 12 (`CoinStoreBuyBackRequestPacket`), 13 (`CoinStoreBuyBackResponsePacket`).
//!   `FUN_00c76dd0`/`FUN_00c77030` build the sell request (opcode 5) as two GUID words, one item
//!   GUID word and one quantity word; `FUN_00c76c60` builds the buyback request (opcode 12) as two
//!   integer words. `FUN_00c78a10` logs `"Buy Back response - tid: %d, result: %d."` and only
//!   treats result 1 as success.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` - `CoinStore_TransactionHistory_DS` columns
//!   (Id 0, Type 1, MerchantGuid 2, ItemId 3, Quantity 4, TintId 5, Price 6, BoughtBack 7);
//!   `GetValidBuyBackItems` drops every row whose `BoughtBack` is `1`; `NpcMerchantV2:BuyBackItem`
//!   forwards the transaction id alone; `CoinStore_PlayerSell_DS` carries `ItemGuid`, `Quantity`,
//!   `Price` and `NoSale`.
//! * `lab/decompiled/server/Gateway/Gateway.Player/PlayerCode.cs` - the original server writes
//!   `ClientItemDefinition.ResellValue` into the item definition packet, and
//!   `lab/decompiled/server/Gateway/Gateway/ClientItemDefinition.cs` names that field.
//! * `lab/decompiled/server/LibSOE/SOE/BaseCoinStorePackets.cs` - the recovered opcode enum where
//!   `CoinStoreBuyFromClientRequestPacket = 5` and `CoinStoreSellToClientRequestPacket = 4`.
//!
//! BOUNDARIES: the ledger below is a reconstruction of the server-side transaction semantics. The
//! sell payout rule (unit resell value x quantity) and the buyback price source (the price stored on
//! the recorded transaction) are reconstructed; a `max(1, cost / 10)` formula reproduced by one
//! community corpus is explicitly *not* used, because two items in the read-only wiki and the
//! community catalogue disagree with it (`Zipped Leather Jacket`: cost 400, resell 27).
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_084;` under `src/restoration/`.
#[path = "../src/restoration/piece_084.rs"]
#[allow(dead_code)]
mod piece_084;

use piece_084::{
    BASE_COIN_STORE_PACKET, BUYBACK_RESULT_REJECTED, BUYBACK_ROWS_FILTER_RECOVERED, FieldSpec,
    FieldWidth, OP_BUY_REQUEST, OP_BUYBACK_REQUEST, OP_BUYBACK_RESPONSE,
    OP_CLEAR_TRANSACTION_HISTORY, OP_SELL_REQUEST, OP_TRANSACTION_COMPLETE,
    RESULT_BALANCE_NOT_ENOUGH, RESULT_INVALID_ITEM, RESULT_INVALID_QUANTITY,
    RESULT_SERVICE_UNAVAILABLE, RESULT_SUCCESS, SELL_REQUEST_FIELDS, TRANSACTION_RECORD_FIELDS,
    TRANSACTION_TYPE_BUY, TRANSACTION_TYPE_SELL, VendorError, VendorLedger, buyback_cost,
    sell_payout, sell_request_len,
};

const MERCHANT: u64 = 0x0000_1234_5678_9abc;

fn ledger() -> VendorLedger {
    let mut ledger = VendorLedger::new(1_000);
    // A five-unit stack priced straight from the recovered `Zipped Leather Jacket` row
    // (cost 400, resell 27) in the pinned community catalogue.
    let guid = ledger.grant(79_388, 0, 5, 27, false);
    assert_eq!(guid, 1);
    ledger
}

// --- recovered wire contract -------------------------------------------------------------

#[test]
fn wire_opcode_table_matches_the_original_client() {
    assert_eq!(BASE_COIN_STORE_PACKET, 165);
    assert_eq!(OP_BUY_REQUEST, 4);
    assert_eq!(OP_SELL_REQUEST, 5);
    assert_eq!(OP_TRANSACTION_COMPLETE, 6);
    assert_eq!(OP_CLEAR_TRANSACTION_HISTORY, 11);
    assert_eq!(OP_BUYBACK_REQUEST, 12);
    assert_eq!(OP_BUYBACK_RESPONSE, 13);
    assert_eq!(RESULT_SUCCESS, 1);
}

#[test]
fn sell_request_layout_is_pinned_to_the_native_constructor() {
    assert_eq!(
        SELL_REQUEST_FIELDS,
        [
            FieldSpec::new("merchant_guid", FieldWidth::U64),
            FieldSpec::new("item_guid", FieldWidth::I32),
            FieldSpec::new("quantity", FieldWidth::I32),
        ]
    );
    assert_eq!(sell_request_len(), 16);
    assert_eq!(
        TRANSACTION_RECORD_FIELDS,
        [
            "type",
            "merchant_guid",
            "id",
            "timestamp",
            "item_record",
            "quantity",
            "price",
            "quantity_remaining",
        ]
    );
    assert_eq!(TRANSACTION_TYPE_BUY, 1);
    assert_eq!(TRANSACTION_TYPE_SELL, 2);
    assert_eq!(BUYBACK_ROWS_FILTER_RECOVERED, true);
}

// --- sell pricing ------------------------------------------------------------------------

#[test]
fn sell_payout_is_unit_resell_value_times_quantity() {
    assert_eq!(sell_payout(27, 3), Ok(81));
    assert_eq!(sell_payout(75, 1), Ok(75));
    assert_eq!(sell_payout(0, 4), Ok(0));
    assert_eq!(sell_payout(i32::MAX, 1), Ok(i64::from(i32::MAX)));
    // Two recovered 32-bit amounts can never overflow i64; the product is exact, not saturating.
    assert_eq!(sell_payout(i32::MAX, 3), Ok(6_442_450_941));
    assert_eq!(sell_payout(27, 0), Err(VendorError::InvalidQuantity));
    assert_eq!(sell_payout(-1, 1), Err(VendorError::InvalidQuantity));
    assert_eq!(sell_payout(27, -2), Err(VendorError::InvalidQuantity));
    assert_eq!(
        VendorError::Overflow.wire_result_code(),
        RESULT_SERVICE_UNAVAILABLE
    );
}

#[test]
fn a_coin_balance_that_cannot_hold_the_payout_is_refused_whole() {
    let mut ledger = VendorLedger::new(i64::MAX - 10);
    ledger.grant(5_000, 0, 100, i32::MAX, false);

    let refused = ledger.sell(MERCHANT, 1, 100).unwrap_err();
    assert_eq!(refused, VendorError::Overflow);
    assert_eq!(refused.wire_result_code(), RESULT_SERVICE_UNAVAILABLE);
    assert_eq!(ledger.coins(), i64::MAX - 10);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(100));
    assert_eq!(ledger.transactions().len(), 0);
}

// --- sell transactions -------------------------------------------------------------------

#[test]
fn sell_moves_the_item_and_credits_coins_once() {
    let mut ledger = ledger();

    let sale = ledger.sell(MERCHANT, 1, 3).expect("sell succeeds");
    assert_eq!(sale.transaction_id, 1);
    assert_eq!(sale.coins_paid, 81);
    assert_eq!(sale.quantity, 3);
    assert_eq!(ledger.coins(), 1_081);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(2));
    assert_eq!(ledger.transactions().len(), 1);

    let record = ledger.transactions()[0].clone();
    assert_eq!(record.id, 1);
    assert_eq!(record.transaction_type, TRANSACTION_TYPE_SELL);
    assert_eq!(record.merchant_guid, MERCHANT);
    assert_eq!(record.definition, 79_388);
    assert_eq!(record.item_guid, 1);
    assert_eq!(record.unit_price, 27);
    assert_eq!(record.quantity, 3);
    assert_eq!(record.quantity_remaining, 3);
    assert!(!record.bought_back);
    assert_eq!(ledger.buyback_rows().len(), 1);
}

#[test]
fn a_no_sale_item_is_refused_with_the_invalid_item_result() {
    let mut ledger = VendorLedger::new(0);
    ledger.grant(1, 0, 1, 5, true);

    let refused = ledger.sell(MERCHANT, 1, 1).unwrap_err();
    assert_eq!(refused, VendorError::NoSale);
    assert_eq!(refused.wire_result_code(), RESULT_INVALID_ITEM);
    assert_eq!(ledger.coins(), 0);
    assert_eq!(ledger.transactions().len(), 0);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(1));
}

#[test]
fn a_replayed_sell_request_cannot_duplicate_coins() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 3).expect("first sale succeeds");

    let replay = ledger.sell(MERCHANT, 1, 3).unwrap_err();
    assert_eq!(replay, VendorError::InsufficientQuantity);
    assert_eq!(replay.wire_result_code(), RESULT_INVALID_QUANTITY);
    assert_eq!(ledger.coins(), 1_081);
    assert_eq!(ledger.transactions().len(), 1);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(2));

    let unknown = ledger.sell(MERCHANT, 99, 1).unwrap_err();
    assert_eq!(unknown, VendorError::UnknownItem);
    assert_eq!(unknown.wire_result_code(), RESULT_INVALID_ITEM);
}

// --- buyback -----------------------------------------------------------------------------

#[test]
fn buyback_charges_the_recorded_price_and_returns_the_same_instance() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 3).expect("sale");

    let buyback = ledger.buy_back(1, 0).expect("buyback succeeds");
    assert_eq!(buyback.transaction_id, 1);
    assert_eq!(buyback.coins_charged, 81);
    assert_eq!(buyback.quantity, 3);
    assert_eq!(buyback.item_guid, 1);
    assert_eq!(ledger.coins(), 1_000);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(5));
    assert_eq!(ledger.items().len(), 1);
    assert!(ledger.transactions()[0].bought_back);
    assert!(ledger.buyback_rows().is_empty());
}

#[test]
fn a_replayed_buyback_is_refused_without_duplication() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 3).expect("sale");
    ledger.buy_back(1, 0).expect("first buyback");

    let replay = ledger.buy_back(1, 0).unwrap_err();
    assert_eq!(replay, VendorError::AlreadyBoughtBack);
    assert_eq!(replay.buyback_result_code(), BUYBACK_RESULT_REJECTED);
    assert_eq!(ledger.coins(), 1_000);
    assert_eq!(ledger.items().len(), 1);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(5));
}

#[test]
fn partial_buyback_tracks_the_remaining_quantity() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 4).expect("sale");

    let first = ledger.buy_back(1, 1).expect("partial buyback");
    assert_eq!(first.coins_charged, 27);
    assert_eq!(first.quantity, 1);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(2));
    assert_eq!(ledger.transactions()[0].quantity_remaining, 3);
    assert!(!ledger.transactions()[0].bought_back);
    assert_eq!(ledger.buyback_rows().len(), 1);

    let too_many = ledger.buy_back(1, 5).unwrap_err();
    assert_eq!(too_many, VendorError::InvalidQuantity);
    assert_eq!(ledger.transactions()[0].quantity_remaining, 3);
    assert_eq!(ledger.coins(), 1_081);

    let rest = ledger.buy_back(1, 3).expect("second buyback");
    assert_eq!(rest.coins_charged, 81);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(5));
    assert!(ledger.transactions()[0].bought_back);
    assert_eq!(
        ledger.buy_back(1, 1).unwrap_err(),
        VendorError::AlreadyBoughtBack
    );
}

#[test]
fn buyback_needs_enough_coins_and_leaves_state_untouched_when_short() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 5).expect("sale");
    assert_eq!(ledger.coins(), 1_135);
    assert_eq!(buyback_cost(27, 5), Ok(135));
    assert_eq!(
        buyback_cost(i32::MAX, i32::MAX),
        Ok(4_611_686_014_132_420_609)
    );

    ledger.adjust_coins(-1_120).expect("external sink");
    assert_eq!(ledger.coins(), 15);

    let short = ledger.buy_back(1, 5).unwrap_err();
    assert_eq!(short, VendorError::NotEnoughCoins);
    assert_eq!(short.wire_result_code(), RESULT_BALANCE_NOT_ENOUGH);
    assert_eq!(ledger.coins(), 15);
    assert_eq!(ledger.transactions()[0].quantity_remaining, 5);
    assert_eq!(ledger.item(1), None);
}

#[test]
fn buyback_ignores_purchase_history_rows() {
    let mut ledger = ledger();
    let purchase_id = ledger.record_purchase(MERCHANT, 5_000, 3, 40, 2);
    assert_eq!(purchase_id, 1);
    assert_eq!(
        ledger.transactions()[0].transaction_type,
        TRANSACTION_TYPE_BUY
    );
    assert!(ledger.buyback_rows().is_empty());

    let refused = ledger.buy_back(purchase_id, 1).unwrap_err();
    assert_eq!(refused, VendorError::UnknownTransaction);
    assert_eq!(ledger.transactions().len(), 1);

    let unknown = ledger.buy_back(77, 1).unwrap_err();
    assert_eq!(unknown, VendorError::UnknownTransaction);
}

#[test]
fn clearing_transaction_history_ends_buyback_eligibility() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 2).expect("sale");
    assert_eq!(ledger.buyback_rows().len(), 1);

    ledger.clear_transaction_history();
    assert!(ledger.buyback_rows().is_empty());
    assert_eq!(
        ledger.buy_back(1, 2).unwrap_err(),
        VendorError::UnknownTransaction
    );
    assert_eq!(ledger.coins(), 1_054);
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(3));
}

// --- restart, identity and persistence ---------------------------------------------------

#[test]
fn a_restart_cannot_replay_a_completed_buyback() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 3).expect("sale");
    ledger.buy_back(1, 0).expect("buyback");

    let snapshot = ledger.snapshot();
    let mut restored = VendorLedger::restore(&snapshot).expect("snapshot restores");
    assert_eq!(restored.coins(), ledger.coins());
    assert_eq!(restored.items(), ledger.items());
    assert_eq!(restored.transactions(), ledger.transactions());

    assert_eq!(
        restored.buy_back(1, 0).unwrap_err(),
        VendorError::AlreadyBoughtBack
    );
    assert_eq!(restored.coins(), 1_000);
    assert_eq!(restored.items().len(), 1);
    assert_eq!(restored.item(1).map(|item| item.quantity), Some(5));
}

#[test]
fn a_restart_preserves_a_partial_buyback_and_the_next_ids() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 5).expect("sale");
    ledger.buy_back(1, 2).expect("partial buyback");
    ledger.sell(MERCHANT, 1, 1).expect("second sale");

    let restored = VendorLedger::restore(&ledger.snapshot()).expect("snapshot restores");
    assert_eq!(restored.transactions().len(), 2);
    assert_eq!(restored.transactions()[0].quantity_remaining, 3);
    assert_eq!(restored.transactions()[1].id, 2);

    let mut restored = restored;
    let next = restored.sell(MERCHANT, 1, 1).expect("third sale");
    assert_eq!(next.transaction_id, 3, "transaction ids are never reused");
    let guid = restored.grant(79_389, 0, 1, 27, false);
    assert_eq!(guid, 2, "item guids are never reused");

    assert!(VendorLedger::restore("not json").is_err());
}

#[test]
fn a_fully_sold_instance_comes_back_with_its_original_guid() {
    let mut ledger = ledger();
    ledger.sell(MERCHANT, 1, 5).expect("full stack sale");
    assert_eq!(ledger.item(1), None);

    ledger.buy_back(1, 0).expect("buyback");
    assert_eq!(ledger.item(1).map(|item| item.quantity), Some(5));
    assert_eq!(ledger.item(1).map(|item| item.resell_value), Some(27));
    assert_eq!(ledger.grant(79_388, 0, 1, 27, false), 2);
}
