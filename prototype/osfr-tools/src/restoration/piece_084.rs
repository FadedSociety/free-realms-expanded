//! Work item 084 - vendor selling and buyback.
//!
//! Recovered original material (pinned, hashed and re-checked by the private recovery checker `piece_084.py`):
//!
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` is the shipped client. Its packet
//!   constructors call `FUN_00c6e960(<opcode>)` under `BaseCoinStore` (165), which fixes the
//!   opcode table used below. `FUN_00c76dd0` (called by `FUN_00c77030`) fills the sell request
//!   with four 32-bit words: two merchant GUID words, one item GUID word and one quantity word.
//!   `FUN_00c76c60` fills the buyback request with two 32-bit words. `FUN_00c78a10` parses the
//!   buyback response and logs `"Buy Back response - tid: %d, result: %d."`, treating only
//!   `result == 1` as success.
//! * `lab/references/Sanctuary/src/Sanctuary.Packet/BaseCoinStorePacket/...` pins the same sell
//!   request layout (`MerchantGuid`, `ItemGuid`, `Quantity`) and the transaction-complete fields
//!   (`Result`, `ItemGuid`, `TransactionRecord`).
//! * `lab/decompiled/client-ui/ScriptsBase.lua` exposes the transaction history to script as
//!   `Id, Type, MerchantGuid, ItemId, Quantity, TintId, Price, BoughtBack`, and
//!   `GetValidBuyBackItems` drops every row whose `BoughtBack` is `1`. The merchant window's only
//!   buyback action forwards the transaction id alone, so a request with quantity 0 means "the
//!   remaining quantity of that transaction".
//! * `lab/decompiled/server/Gateway/Gateway.Player/PlayerCode.cs` writes
//!   `ClientItemDefinition.ResellValue` into the item definition packet the client receives, which
//!   is the `CoinSellPrice` column of `ItemDefinitionSelect_DS` (column 26).
//!
//! Reconstructed policy (explicitly *not* original fact):
//!
//! * The payout for a sale is `ResellValue * quantity` and the buyback price is the unit price
//!   recorded on the sale, both in exact `i64` coins. A `max(1, cost / 10)` formula reproduces
//!   1229 of 1230 priced rows in `lab/references/OpenSourceFreeRealms/.../ClientItemDefinitions.json`,
//!   but the read-only wiki and the pinned community catalogue agree on `Zipped Leather Jacket` at
//!   cost 400 / resell 27, so the ledger takes the per-item value as data instead of a formula.
//! * Rejection codes for the transaction-complete packet are the community-recorded list; the
//!   buyback response only distinguishes `1` from "not 1", so a rejection uses
//!   `BUYBACK_RESULT_REJECTED`.
//! * Transaction ids and item instance GUIDs are monotonic and never reused, including across a
//!   transaction-history clear and a snapshot restore. That is what makes a replayed or restarted
//!   request unable to duplicate coins or items.
//!
//! This module owns no shared state: the lead registers it and routes real inventory/coin writes
//! through [`VendorLedger`] or its own economy layer.

use serde::{Deserialize, Serialize};

/// Recovered base packet code for every coin-store message.
pub const BASE_COIN_STORE_PACKET: u16 = 165;

/// Recovered sub-opcodes of [`BASE_COIN_STORE_PACKET`], read from the shipped client's own packet
/// constructors.
pub const OP_ITEM_LIST: u16 = 1;
pub const OP_ITEM_DEFINITIONS_REQUEST: u16 = 2;
pub const OP_ITEM_DEFINITIONS_RESPONSE: u16 = 3;
/// `CoinStoreSellToClientRequestPacket`: the server sells an item to the player (a purchase).
pub const OP_BUY_REQUEST: u16 = 4;
/// `CoinStoreBuyFromClientRequestPacket`: the server buys an item from the player (a sale).
pub const OP_SELL_REQUEST: u16 = 5;
pub const OP_TRANSACTION_COMPLETE: u16 = 6;
pub const OP_OPEN: u16 = 7;
pub const OP_DYNAMIC_LIST_UPDATE_REQUEST: u16 = 8;
pub const OP_DYNAMIC_LIST_UPDATE_RESPONSE: u16 = 9;
pub const OP_MERCHANT_LIST: u16 = 10;
pub const OP_CLEAR_TRANSACTION_HISTORY: u16 = 11;
pub const OP_BUYBACK_REQUEST: u16 = 12;
pub const OP_BUYBACK_RESPONSE: u16 = 13;
/// `CoinStoreSellToClientAndGiftRequestPacket`, recovered from the same client enum.
pub const OP_SELL_AND_GIFT_REQUEST: u16 = 14;

/// `Type` values recovered from `CoinStoreTransactionRecord`.
pub const TRANSACTION_TYPE_BUY: i32 = 1;
pub const TRANSACTION_TYPE_SELL: i32 = 2;

/// Transaction-complete result codes, as recorded by the pinned community handler.
pub const RESULT_SUCCESS: i32 = 1;
pub const RESULT_INVALID_ITEM: i32 = 3;
pub const RESULT_INVALID_QUANTITY: i32 = 4;
pub const RESULT_INVALID_MERCHANT: i32 = 5;
pub const RESULT_INVALID_TINT: i32 = 6;
pub const RESULT_BALANCE_NOT_ENOUGH: i32 = 7;
pub const RESULT_SERVICE_UNAVAILABLE: i32 = 8;
pub const RESULT_RECIPIENT_NOT_FOUND: i32 = 9;
pub const RESULT_NOT_A_MEMBER: i32 = 10;
pub const RESULT_RECIPIENT_CANNOT_ACCEPT: i32 = 11;

/// The client only treats a buyback result of `1` as success; every other value follows the
/// "failed to void transaction on client" path. `2` is this module's rejection value.
pub const BUYBACK_RESULT_SUCCESS: i32 = 1;
pub const BUYBACK_RESULT_REJECTED: i32 = 2;

/// `GetValidBuyBackItems` drops rows whose `BoughtBack` column is `1`. Pinned so the checker and
/// the Rust ledger cannot drift from the recovered client filter.
pub const BUYBACK_ROWS_FILTER_RECOVERED: bool = true;

/// Width of a wire field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldWidth {
    U64,
    I32,
}

impl FieldWidth {
    pub const fn size(self) -> usize {
        match self {
            FieldWidth::U64 => 8,
            FieldWidth::I32 => 4,
        }
    }
}

/// One recovered field of a coin-store request, in wire order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldSpec {
    pub name: &'static str,
    pub width: FieldWidth,
}

impl FieldSpec {
    pub const fn new(name: &'static str, width: FieldWidth) -> Self {
        Self { name, width }
    }
}

/// `CoinStoreBuyFromClientRequestPacket` (opcode 5), matching the client's constructor.
pub const SELL_REQUEST_FIELDS: [FieldSpec; 3] = [
    FieldSpec::new("merchant_guid", FieldWidth::U64),
    FieldSpec::new("item_guid", FieldWidth::I32),
    FieldSpec::new("quantity", FieldWidth::I32),
];

/// `CoinStoreBuyBackRequestPacket` (opcode 12): a transaction id and a quantity.
pub const BUYBACK_REQUEST_FIELDS: [FieldSpec; 2] = [
    FieldSpec::new("transaction_id", FieldWidth::I32),
    FieldSpec::new("quantity", FieldWidth::I32),
];

/// `CoinStoreBuyBackResponsePacket` (opcode 13): the id the response is about plus a result.
pub const BUYBACK_RESPONSE_FIELDS: [FieldSpec; 2] = [
    FieldSpec::new("transaction_id", FieldWidth::I32),
    FieldSpec::new("result", FieldWidth::I32),
];

/// Serialized order of the `TransactionRecord` carried by the transaction-complete packet.
pub const TRANSACTION_RECORD_FIELDS: [&str; 8] = [
    "type",
    "merchant_guid",
    "id",
    "timestamp",
    "item_record",
    "quantity",
    "price",
    "quantity_remaining",
];

const fn field_len(fields: &[FieldSpec]) -> usize {
    let mut total = 0;
    let mut index = 0;
    while index < fields.len() {
        total += fields[index].width.size();
        index += 1;
    }
    total
}

/// Wire length of the sell request as the shipped client builds it.
pub const fn sell_request_len() -> usize {
    field_len(&SELL_REQUEST_FIELDS)
}

/// Wire length of the buyback request.
pub const fn buyback_request_len() -> usize {
    field_len(&BUYBACK_REQUEST_FIELDS)
}

/// Why a vendor transaction was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VendorError {
    /// The item definition carries `NoSale`.
    NoSale,
    /// The instance GUID is not in the ledger's inventory.
    UnknownItem,
    /// Quantity was zero/negative, or larger than the instance holds.
    InvalidQuantity,
    /// The instance holds fewer units than requested.
    InsufficientQuantity,
    /// The ledger does not hold enough coins.
    NotEnoughCoins,
    /// The transaction id is unknown, or the record is not a sale.
    UnknownTransaction,
    /// The transaction was already bought back in full (or the history was cleared).
    AlreadyBoughtBack,
    /// A checked coin computation overflowed.
    Overflow,
}

impl VendorError {
    /// Result code for `CoinStoreTransactionCompletePacket`.
    pub const fn wire_result_code(self) -> i32 {
        match self {
            VendorError::NoSale | VendorError::UnknownItem => RESULT_INVALID_ITEM,
            VendorError::InvalidQuantity | VendorError::InsufficientQuantity => {
                RESULT_INVALID_QUANTITY
            }
            VendorError::NotEnoughCoins => RESULT_BALANCE_NOT_ENOUGH,
            VendorError::UnknownTransaction
            | VendorError::AlreadyBoughtBack
            | VendorError::Overflow => RESULT_SERVICE_UNAVAILABLE,
        }
    }

    /// Result code for `CoinStoreBuyBackResponsePacket`.
    pub const fn buyback_result_code(self) -> i32 {
        BUYBACK_RESULT_REJECTED
    }
}

/// Coins a sale pays out: the item's recovered resell value per unit.
pub fn sell_payout(unit_value: i32, quantity: i32) -> Result<i64, VendorError> {
    if unit_value < 0 || quantity < 1 {
        return Err(VendorError::InvalidQuantity);
    }
    i64::from(unit_value)
        .checked_mul(i64::from(quantity))
        .ok_or(VendorError::Overflow)
}

/// Coins a buyback charges: the unit price recorded when the stack was sold.
pub fn buyback_cost(unit_price: i32, quantity: i32) -> Result<i64, VendorError> {
    if unit_price < 0 || quantity < 1 {
        return Err(VendorError::InvalidQuantity);
    }
    i64::from(unit_price)
        .checked_mul(i64::from(quantity))
        .ok_or(VendorError::Overflow)
}

/// An instance the ledger owns.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldItem {
    pub guid: i32,
    pub definition: i32,
    pub tint: i32,
    pub quantity: i32,
    pub resell_value: i32,
    pub no_sale: bool,
}

/// A row of `BaseClient.CoinStore.TransactionHistory`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i32,
    pub transaction_type: i32,
    pub merchant_guid: u64,
    pub item_guid: i32,
    pub definition: i32,
    pub tint: i32,
    pub unit_price: i32,
    pub quantity: i32,
    pub quantity_remaining: i32,
    pub bought_back: bool,
}

/// Outcome of an accepted sale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaleOutcome {
    pub transaction_id: i32,
    pub coins_paid: i64,
    pub quantity: i32,
}

/// Outcome of an accepted buyback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuyBackOutcome {
    pub transaction_id: i32,
    pub coins_charged: i64,
    pub quantity: i32,
    pub item_guid: i32,
}

/// Coin and transaction authority for one character's vendor interaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorLedger {
    coins: i64,
    next_item_guid: i32,
    next_transaction_id: i32,
    items: Vec<HeldItem>,
    transactions: Vec<Transaction>,
}

impl VendorLedger {
    pub fn new(coins: i64) -> Self {
        Self {
            coins,
            next_item_guid: 1,
            next_transaction_id: 1,
            items: Vec::new(),
            transactions: Vec::new(),
        }
    }

    pub fn coins(&self) -> i64 {
        self.coins
    }

    pub fn items(&self) -> &[HeldItem] {
        &self.items
    }

    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    pub fn item(&self, guid: i32) -> Option<&HeldItem> {
        self.items.iter().find(|item| item.guid == guid)
    }

    /// Rows the recovered client would render: sales whose `BoughtBack` column is still `0`.
    pub fn buyback_rows(&self) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|record| {
                record.transaction_type == TRANSACTION_TYPE_SELL && !record.bought_back
            })
            .collect()
    }

    /// Grants a fresh instance. The caller owns the item-grant path (rewards, shops, loot);
    /// this only allocates the identity the vendor transactions bind to.
    pub fn grant(
        &mut self,
        definition: i32,
        tint: i32,
        quantity: i32,
        resell_value: i32,
        no_sale: bool,
    ) -> i32 {
        assert!(quantity >= 1, "grant quantity must be positive");
        let guid = self.next_item_guid;
        self.next_item_guid += 1;
        self.items.push(HeldItem {
            guid,
            definition,
            tint,
            quantity,
            resell_value,
            no_sale,
        });
        guid
    }

    /// Applies an external coin source or sink (quest reward, shop purchase, trainer fee).
    pub fn adjust_coins(&mut self, delta: i64) -> Result<i64, VendorError> {
        let updated = self.coins.checked_add(delta).ok_or(VendorError::Overflow)?;
        if updated < 0 {
            return Err(VendorError::NotEnoughCoins);
        }
        self.coins = updated;
        Ok(updated)
    }

    /// Sells `quantity` units of one instance to a vendor.
    pub fn sell(
        &mut self,
        merchant_guid: u64,
        item_guid: i32,
        quantity: i32,
    ) -> Result<SaleOutcome, VendorError> {
        if quantity < 1 {
            return Err(VendorError::InvalidQuantity);
        }
        let index = self
            .items
            .iter()
            .position(|item| item.guid == item_guid)
            .ok_or(VendorError::UnknownItem)?;
        let item = &self.items[index];
        if item.no_sale {
            return Err(VendorError::NoSale);
        }
        if item.quantity < quantity {
            return Err(VendorError::InsufficientQuantity);
        }

        let payout = sell_payout(item.resell_value, quantity)?;
        let updated_coins = self
            .coins
            .checked_add(payout)
            .ok_or(VendorError::Overflow)?;
        let transaction_id = self.next_transaction_id;
        let record = Transaction {
            id: transaction_id,
            transaction_type: TRANSACTION_TYPE_SELL,
            merchant_guid,
            item_guid,
            definition: item.definition,
            tint: item.tint,
            unit_price: item.resell_value,
            quantity,
            quantity_remaining: quantity,
            bought_back: false,
        };

        if item.quantity == quantity {
            self.items.remove(index);
        } else {
            self.items[index].quantity -= quantity;
        }
        self.coins = updated_coins;
        self.next_transaction_id += 1;
        self.transactions.push(record);

        Ok(SaleOutcome {
            transaction_id,
            coins_paid: payout,
            quantity,
        })
    }

    /// Buys back `quantity` units of a recorded sale. A quantity of `0` means every unit still
    /// outstanding on that transaction, which is what the recovered merchant window asks for.
    pub fn buy_back(
        &mut self,
        transaction_id: i32,
        quantity: i32,
    ) -> Result<BuyBackOutcome, VendorError> {
        if quantity < 0 {
            return Err(VendorError::InvalidQuantity);
        }
        let index = self
            .transactions
            .iter()
            .position(|record| record.id == transaction_id)
            .ok_or(VendorError::UnknownTransaction)?;
        let record = &self.transactions[index];
        if record.transaction_type != TRANSACTION_TYPE_SELL {
            return Err(VendorError::UnknownTransaction);
        }
        if record.bought_back || record.quantity_remaining <= 0 {
            return Err(VendorError::AlreadyBoughtBack);
        }

        let wanted = if quantity == 0 {
            record.quantity_remaining
        } else {
            quantity
        };
        if wanted > record.quantity_remaining {
            return Err(VendorError::InvalidQuantity);
        }

        let cost = buyback_cost(record.unit_price, wanted)?;
        if self.coins < cost {
            return Err(VendorError::NotEnoughCoins);
        }

        let item_guid = record.item_guid;
        let definition = record.definition;
        let tint = record.tint;
        let unit_price = record.unit_price;

        self.coins -= cost;
        let record = &mut self.transactions[index];
        record.quantity_remaining -= wanted;
        record.bought_back = record.quantity_remaining == 0;

        match self.items.iter_mut().find(|item| item.guid == item_guid) {
            Some(item) => item.quantity += wanted,
            None => self.items.push(HeldItem {
                guid: item_guid,
                definition,
                tint,
                quantity: wanted,
                resell_value: unit_price,
                // The stack was accepted by a vendor, so its definition cannot be NoSale.
                no_sale: false,
            }),
        }

        Ok(BuyBackOutcome {
            transaction_id,
            coins_charged: cost,
            quantity: wanted,
            item_guid,
        })
    }

    /// Records a purchase in the same history the client renders. Item and coin effects of a
    /// purchase belong to the shop owner; the ledger only keeps the history row so a buyback can
    /// tell a sale apart from a purchase.
    pub fn record_purchase(
        &mut self,
        merchant_guid: u64,
        definition: i32,
        tint: i32,
        unit_price: i32,
        quantity: i32,
    ) -> i32 {
        let id = self.next_transaction_id;
        self.next_transaction_id += 1;
        self.transactions.push(Transaction {
            id,
            transaction_type: TRANSACTION_TYPE_BUY,
            merchant_guid,
            item_guid: 0,
            definition,
            tint,
            unit_price,
            quantity,
            quantity_remaining: 0,
            bought_back: false,
        });
        id
    }

    /// Applies `CoinStoreClearTransactionHistoryPacket`. Ids keep counting up so a cleared
    /// history cannot hand a stale id to a fresh transaction.
    pub fn clear_transaction_history(&mut self) {
        self.transactions.clear();
    }

    /// Serializes the ledger for a save file.
    pub fn snapshot(&self) -> String {
        serde_json::to_string(self).expect("ledger snapshot serializes")
    }

    /// Restores a ledger written by [`VendorLedger::snapshot`].
    pub fn restore(snapshot: &str) -> Result<Self, String> {
        serde_json::from_str(snapshot).map_err(|error| error.to_string())
    }
}
