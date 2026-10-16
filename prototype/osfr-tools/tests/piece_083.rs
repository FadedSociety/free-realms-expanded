//! Piece 083 (shops and purchases): recovered coin-store request packet, catalogue
//! rows, price formula and the bounded purchase transaction.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL / recovered facts the vectors assert (provenance in
//! `notes/Research/Pieces/083 Shops and purchases.md` and in the lab bundle):
//! * Coin-store packet family 165 with sub-opcodes 1..18, including
//!   `CoinStoreSellToClientRequestPacket` = 4 and `CoinStoreTransactionCompletePacket` = 6
//!   (`lab/decompiled/server/LibSOE/SOE/BaseCoinStorePackets.cs`); the supplied first-party
//!   client builds both classes (`CoinStoreSellToClientRequestPacket::vftable`,
//!   `CoinStoreTransactionCompletePacket::vftable` in
//!   `lab/decompiled/client/FreeRealms/pseudocode.c`).
//! * The request body order `MerchantGuid u64, MerchantUnknown i32,
//!   ItemRecord { Definition i32, Tint i32 }, Quantity i32` comes from the pinned community
//!   reference `Sanctuary.Packet.BaseCoinStorePacket.CoinStoreSellToClientRequestPacket`
//!   (`lab/references/FreeRealms-Legacy`), cross-checked by the checker against that file.
//! * The transaction-complete result list (1 success, 3 invalid item, 4 invalid quantity,
//!   5 invalid merchant, 6 invalid tint, 7 balance, 8 unavailable, 9 recipient, 10 not a
//!   member, 11 recipient cannot accept) is the pinned community `CoinStoreTransactionCompletePacket`
//!   table.
//! * Item costs, member discounts, tintability and member-only flags are rows of the pinned
//!   community `Resources/ClientItemDefinitions.json`; the member price is the pinned
//!   `ClientItemDefinition.GetMemberPurchasePrice()` float32 formula.
//! * `QuantityAvailable`, `MaxPerOrder`, `MaxPurchasable` are first-party client store-item
//!   fields: the client names them in the store data-source table
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`) and the UI binds them
//!   (`lab/decompiled/client-ui/ScriptsBase.lua`).
//!
//! RECONSTRUCTED here: the ledger, the catalogue document, the per-merchant binding of stock
//! and limits, the ownership cap, every check order and every refusal mapping. No original
//! server code for a purchase was recovered, so nothing in this file is a native acceptance
//! claim and the module never mints currency or items on its own.

#[path = "../src/restoration/piece_083.rs"]
#[allow(dead_code)]
mod piece_083;

use piece_083::{
    Buyer, COIN_STORE_BUY_FROM_CLIENT_REQUEST, COIN_STORE_FAMILY, COIN_STORE_ITEM_LIST,
    COIN_STORE_MERCHANT_LIST, COIN_STORE_REQUEST_LENGTH, COIN_STORE_RESULT_BALANCE_NOT_ENOUGH,
    COIN_STORE_RESULT_INVALID_ITEM, COIN_STORE_RESULT_INVALID_ITEM_TINT,
    COIN_STORE_RESULT_INVALID_MERCHANT, COIN_STORE_RESULT_INVALID_QUANTITY,
    COIN_STORE_RESULT_NOT_A_MEMBER, COIN_STORE_RESULT_SUCCESS, COIN_STORE_SELL_TO_CLIENT_REQUEST,
    COIN_STORE_TRANSACTION_COMPLETE, COIN_STORE_TRANSACTION_TYPE_BUY, CatalogueError,
    PurchaseError, PurchaseRequest, Refusal, ShopCatalogue, ShopLedger, member_unit_price,
};

const MERCHANT: u64 = 402;

fn request(merchant: u64, definition: i32, tint: i32, quantity: i32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(COIN_STORE_REQUEST_LENGTH);
    bytes.extend(COIN_STORE_FAMILY.to_le_bytes());
    bytes.extend(COIN_STORE_SELL_TO_CLIENT_REQUEST.to_le_bytes());
    bytes.extend(merchant.to_le_bytes());
    bytes.extend(0i32.to_le_bytes());
    bytes.extend(definition.to_le_bytes());
    bytes.extend(tint.to_le_bytes());
    bytes.extend(quantity.to_le_bytes());
    bytes
}

/// Rows copied from the pinned `ClientItemDefinitions.json` (ids 1, 2 and 34) plus one
/// synthetic fixture row used only for the ownership and stock paths.
fn catalogue() -> ShopCatalogue {
    let bytes = br#"{
      "schema_version": 1,
      "merchants": [
        {
          "guid": 402,
          "items": [
            {
              "definition_id": 1,
              "cost": 5,
              "member_discount": 10,
              "is_tintable": false,
              "icon_tint": 231,
              "quantity_available": 4,
              "max_per_order": 2,
              "owned_limit": 1
            },
            {
              "definition_id": 2,
              "cost": 320,
              "member_discount": 10,
              "is_tintable": true,
              "icon_tint": 228
            },
            {
              "definition_id": 34,
              "cost": 206,
              "member_discount": 10,
              "members_only": true,
              "is_tintable": false,
              "icon_tint": 0,
              "quantity_available": 3
            },
            {
              "definition_id": 7001,
              "cost": 12,
              "member_discount": 10,
              "buy_disabled": true,
              "is_tintable": false,
              "icon_tint": 0
            }
          ]
        }
      ]
    }"#;
    ShopCatalogue::from_json(bytes).expect("fixture catalogue")
}

fn buyer(guid: u64, member: bool) -> Buyer {
    Buyer {
        character_guid: guid,
        member,
    }
}

fn persist_ok(_ledger: &ShopLedger) -> std::io::Result<()> {
    Ok(())
}

#[test]
fn recovered_coin_store_packet_constants() {
    assert_eq!(COIN_STORE_FAMILY, 165);
    assert_eq!(COIN_STORE_ITEM_LIST, 1);
    assert_eq!(COIN_STORE_SELL_TO_CLIENT_REQUEST, 4);
    assert_eq!(COIN_STORE_BUY_FROM_CLIENT_REQUEST, 5);
    assert_eq!(COIN_STORE_TRANSACTION_COMPLETE, 6);
    assert_eq!(COIN_STORE_MERCHANT_LIST, 10);
    assert_eq!(COIN_STORE_REQUEST_LENGTH, 28);
    assert_eq!(COIN_STORE_RESULT_SUCCESS, 1);
    assert_eq!(COIN_STORE_RESULT_INVALID_ITEM, 3);
    assert_eq!(COIN_STORE_RESULT_INVALID_QUANTITY, 4);
    assert_eq!(COIN_STORE_RESULT_INVALID_MERCHANT, 5);
    assert_eq!(COIN_STORE_RESULT_INVALID_ITEM_TINT, 6);
    assert_eq!(COIN_STORE_RESULT_BALANCE_NOT_ENOUGH, 7);
    assert_eq!(COIN_STORE_RESULT_NOT_A_MEMBER, 10);
    assert_eq!(COIN_STORE_TRANSACTION_TYPE_BUY, 1);
}

#[test]
fn request_layout_matches_the_recovered_serializer_order() {
    let bytes = request(MERCHANT, 2, 228, 1);
    let parsed = PurchaseRequest::parse(&bytes).expect("recovered request");
    assert_eq!(parsed.merchant_guid, MERCHANT);
    assert_eq!(parsed.merchant_unknown, 0);
    assert_eq!(parsed.definition_id, 2);
    assert_eq!(parsed.tint, 228);
    assert_eq!(parsed.quantity, 1);

    // Every field is exactly where the pinned deserializer reads it.
    assert_eq!(&bytes[..4], &[165, 0, 4, 0]);
    assert_eq!(&bytes[4..12], &MERCHANT.to_le_bytes());
    assert_eq!(&bytes[12..16], &0i32.to_le_bytes());
    assert_eq!(&bytes[16..20], &2i32.to_le_bytes());
    assert_eq!(&bytes[20..24], &228i32.to_le_bytes());
    assert_eq!(&bytes[24..28], &1i32.to_le_bytes());

    // Truncated, over-long and mis-routed bodies are refused instead of half-read.
    assert_eq!(
        PurchaseRequest::parse(&bytes[..27]),
        Err(piece_083::RequestError::Length(27))
    );
    let mut long = bytes.clone();
    long.push(0);
    assert_eq!(
        PurchaseRequest::parse(&long),
        Err(piece_083::RequestError::Length(29))
    );
    let mut wrong_family = bytes.clone();
    wrong_family[..2].copy_from_slice(&164u16.to_le_bytes());
    assert_eq!(
        PurchaseRequest::parse(&wrong_family),
        Err(piece_083::RequestError::Family(164))
    );
    let mut wrong_sub = bytes.clone();
    wrong_sub[2..4].copy_from_slice(&5u16.to_le_bytes());
    assert_eq!(
        PurchaseRequest::parse(&wrong_sub),
        Err(piece_083::RequestError::SubOpcode(5))
    );
}

#[test]
fn member_price_matches_the_recovered_definition_formula() {
    // Rows 1, 2 and 34 of the pinned catalogue: cost, MemberDiscount, member price.
    assert_eq!(member_unit_price(5, 10), 4);
    assert_eq!(member_unit_price(320, 10), 288);
    assert_eq!(member_unit_price(206, 10), 185);
    // The pinned formula multiplies in float32 and truncates. Cost 1 with a 10% discount
    // is 0, and a missing discount leaves the list price untouched.
    assert_eq!(member_unit_price(1, 10), 0);
    assert_eq!(member_unit_price(5, 0), 5);
    assert_eq!(member_unit_price(0, 10), 0);
}

#[test]
fn catalogue_document_is_validated_row_by_row() {
    let catalogue = catalogue();
    assert_eq!(
        catalogue.item(MERCHANT, 1).map(|i| i.cost),
        Some(5),
        "row 1 keeps the recovered cost"
    );
    assert_eq!(catalogue.item(MERCHANT, 2).map(|i| i.cost), Some(320));
    assert!(catalogue.item(MERCHANT, 999).is_none());
    assert!(catalogue.item(999, 1).is_none());

    let unsupported = br#"{"schema_version":2,"merchants":[]}"#;
    assert_eq!(
        ShopCatalogue::from_json(unsupported),
        Err(CatalogueError::UnsupportedSchema(2))
    );
    let duplicate = br#"{"schema_version":1,"merchants":[
      {"guid":7,"items":[{"definition_id":1,"cost":1,"member_discount":10,"is_tintable":false,"icon_tint":0},
                         {"definition_id":1,"cost":1,"member_discount":10,"is_tintable":false,"icon_tint":0}]}]}"#;
    assert!(matches!(
        ShopCatalogue::from_json(duplicate),
        Err(CatalogueError::Corrupt(_))
    ));
    let zero_merchant = br#"{"schema_version":1,"merchants":[{"guid":0,"items":[]}]}"#;
    assert!(matches!(
        ShopCatalogue::from_json(zero_merchant),
        Err(CatalogueError::Corrupt(_))
    ));
    let bad_discount = br#"{"schema_version":1,"merchants":[
      {"guid":7,"items":[{"definition_id":1,"cost":1,"member_discount":101,"is_tintable":false,"icon_tint":0}]}]}"#;
    assert!(matches!(
        ShopCatalogue::from_json(bad_discount),
        Err(CatalogueError::Corrupt(_))
    ));
}

#[test]
fn purchase_debits_coins_and_grants_the_definition_once() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 320);
    let parsed = PurchaseRequest::parse(&request(MERCHANT, 2, 228, 1)).unwrap();

    let receipt = ledger
        .purchase(&catalogue, buyer(11, false), &parsed, persist_ok)
        .expect("purchase");

    assert_eq!(receipt.result, COIN_STORE_RESULT_SUCCESS);
    assert_eq!(receipt.unit_price, 320);
    assert_eq!(receipt.total_price, 320);
    assert_eq!(
        receipt.record.transaction_type,
        COIN_STORE_TRANSACTION_TYPE_BUY
    );
    assert_eq!(receipt.record.merchant_guid, MERCHANT);
    assert_eq!(receipt.record.definition_id, 2);
    assert_eq!(receipt.record.tint, 228);
    assert_eq!(receipt.record.quantity, 1);
    assert_eq!(receipt.record.price, 320);
    assert_eq!(ledger.coins(11), 0);
    assert_eq!(ledger.owned(11, 2), 1);
    assert_eq!(ledger.stacks(11, 2, 228), 1);
    assert_eq!(ledger.transactions(11).len(), 1);
}

#[test]
fn member_purchase_uses_the_recovered_member_price() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 300);
    let parsed = PurchaseRequest::parse(&request(MERCHANT, 2, 228, 1)).unwrap();

    let receipt = ledger
        .purchase(&catalogue, buyer(11, true), &parsed, persist_ok)
        .unwrap();

    assert_eq!(receipt.unit_price, 288);
    assert_eq!(receipt.total_price, 288);
    assert_eq!(ledger.coins(11), 12);
}

#[test]
fn refusals_carry_the_recovered_result_codes() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 100_000);

    let unknown_merchant = PurchaseRequest::parse(&request(999, 2, 228, 1)).unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, unknown_merchant, buyer(11, false)),
        (COIN_STORE_RESULT_INVALID_MERCHANT, Refusal::UnknownMerchant)
    );
    let unknown_item = PurchaseRequest::parse(&request(MERCHANT, 4242, 0, 1)).unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, unknown_item, buyer(11, false)),
        (COIN_STORE_RESULT_INVALID_ITEM, Refusal::UnknownItem)
    );
    let disabled = PurchaseRequest::parse(&request(MERCHANT, 7001, 0, 1)).unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, disabled, buyer(11, false)),
        (COIN_STORE_RESULT_INVALID_ITEM, Refusal::BuyDisabled)
    );
    let members_only = PurchaseRequest::parse(&request(MERCHANT, 34, 0, 1)).unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, members_only, buyer(11, false)),
        (COIN_STORE_RESULT_NOT_A_MEMBER, Refusal::MembersOnly)
    );
    let wrong_tint = PurchaseRequest::parse(&request(MERCHANT, 1, 7, 1)).unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, wrong_tint, buyer(11, false)),
        (
            COIN_STORE_RESULT_INVALID_ITEM_TINT,
            Refusal::TintNotAllowed { expected: 231 }
        )
    );
}

#[test]
fn quantity_is_bounded_by_the_request_order_and_stock() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 100_000);
    for quantity in [0i32, -1] {
        let parsed = PurchaseRequest::parse(&request(MERCHANT, 2, 228, quantity)).unwrap();
        assert_eq!(
            refusal(&mut ledger, &catalogue, parsed, buyer(11, false)).0,
            COIN_STORE_RESULT_INVALID_QUANTITY
        );
    }
    // Row 1 carries both a per-order limit of 2 and four units of stock.
    let over_order = PurchaseRequest::parse(&request(MERCHANT, 1, 231, 3)).unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, over_order, buyer(11, false)),
        (
            COIN_STORE_RESULT_INVALID_QUANTITY,
            Refusal::OrderLimit { max: 2 }
        )
    );
    // Row 34 has three units and no per-order limit.
    let over_stock = PurchaseRequest::parse(&request(MERCHANT, 34, 0, 4)).unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, over_stock, buyer(11, true)),
        (
            COIN_STORE_RESULT_INVALID_QUANTITY,
            Refusal::OutOfStock { available: 3 }
        )
    );
}

#[test]
fn stock_is_consumed_once_and_then_exhausts() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 100_000);
    // Row 34 carries three units of stock and no per-order or ownership cap.
    let buy_two = PurchaseRequest::parse(&request(MERCHANT, 34, 0, 2)).unwrap();
    ledger
        .purchase(&catalogue, buyer(11, true), &buy_two, persist_ok)
        .unwrap();
    assert_eq!(ledger.stock(MERCHANT, 34), Some(1));
    assert_eq!(ledger.owned(11, 34), 2);

    let buy_two_again = PurchaseRequest::parse(&request(MERCHANT, 34, 0, 2)).unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, buy_two_again, buyer(11, true)),
        (
            COIN_STORE_RESULT_INVALID_QUANTITY,
            Refusal::OutOfStock { available: 1 }
        )
    );
    assert_eq!(
        ledger.stock(MERCHANT, 34),
        Some(1),
        "a refusal consumes no stock"
    );
}

#[test]
fn unique_ownership_limit_is_enforced_per_character() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 100);
    ledger.credit(12, 100);
    let first = PurchaseRequest::parse(&request(MERCHANT, 1, 231, 1)).unwrap();
    ledger
        .purchase(&catalogue, buyer(11, false), &first, persist_ok)
        .unwrap();
    assert_eq!(
        refusal(&mut ledger, &catalogue, first, buyer(11, false)),
        (
            COIN_STORE_RESULT_INVALID_QUANTITY,
            Refusal::OwnershipLimit { limit: 1, owned: 1 }
        )
    );
    // Ownership is per character: another character is not blocked by row 1's cap.
    assert!(
        ledger
            .purchase(&catalogue, buyer(12, false), &first, persist_ok)
            .is_ok()
    );
}

#[test]
fn insufficient_balance_refuses_without_touching_the_ledger() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 319);
    let parsed = PurchaseRequest::parse(&request(MERCHANT, 2, 228, 1)).unwrap();
    let before = ledger.clone();

    assert_eq!(
        refusal(&mut ledger, &catalogue, parsed, buyer(11, false)),
        (
            COIN_STORE_RESULT_BALANCE_NOT_ENOUGH,
            Refusal::BalanceNotEnough {
                required: 320,
                available: 319
            }
        )
    );
    assert_eq!(ledger, before, "a refused purchase changes nothing");
}

#[test]
fn a_failed_persist_rolls_the_whole_purchase_back() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 1000);
    let parsed = PurchaseRequest::parse(&request(MERCHANT, 1, 231, 1)).unwrap();
    let before = ledger.clone();

    let result = ledger.purchase(&catalogue, buyer(11, false), &parsed, |staged| {
        // The staged ledger really does hold the purchase when the writer runs.
        assert_eq!(staged.coins(11), 995);
        assert_eq!(staged.owned(11, 1), 1);
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "snapshot write refused",
        ))
    });

    assert_eq!(
        result,
        Err(PurchaseError::PersistFailed(
            std::io::ErrorKind::PermissionDenied
        ))
    );
    assert_eq!(
        ledger, before,
        "a failed write must not leave a purchase behind"
    );
    assert_eq!(ledger.coins(11), 1000);
    assert_eq!(ledger.owned(11, 1), 0);
    assert_eq!(ledger.stock(MERCHANT, 1), Some(4));
    assert!(ledger.transactions(11).is_empty());
}

#[test]
fn repeated_purchases_conserve_coins_stock_and_transaction_ids() {
    let catalogue = catalogue();
    let mut ledger = ShopLedger::new(&catalogue);
    ledger.credit(11, 1000);
    let mut spent = 0u32;

    for (definition, tint, quantity, member) in
        [(1, 231, 1, false), (2, 228, 1, false), (2, 228, 1, true)]
    {
        let parsed =
            PurchaseRequest::parse(&request(MERCHANT, definition, tint, quantity)).unwrap();
        let receipt = ledger
            .purchase(&catalogue, buyer(11, member), &parsed, persist_ok)
            .unwrap();
        spent += receipt.total_price;
    }

    assert_eq!(spent, 5 + 320 + 288);
    assert_eq!(ledger.coins(11), 1000 - spent);
    assert_eq!(ledger.owned(11, 2), 2);
    assert_eq!(
        ledger
            .transactions(11)
            .iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert_eq!(ledger.stock(MERCHANT, 1), Some(3));
    assert_eq!(
        ledger.stock(MERCHANT, 2),
        None,
        "untracked stock stays untracked"
    );
}

/// Run one request and return the refusal code and reason the module reports.
fn refusal(
    ledger: &mut ShopLedger,
    catalogue: &ShopCatalogue,
    parsed: PurchaseRequest,
    buyer: Buyer,
) -> (i32, Refusal) {
    match ledger.purchase(catalogue, buyer, &parsed, persist_ok) {
        Err(PurchaseError::Refused { result, reason }) => (result, reason),
        other => panic!("expected a refusal, got {other:?}"),
    }
}
