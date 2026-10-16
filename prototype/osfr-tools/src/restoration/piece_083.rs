//! 083: shops and purchases - the coin-store request packet, catalogue rows and the bounded
//! buy-from-merchant transaction.
//!
//! ORIGINAL facts this module is built on (provenance in
//! `notes/Research/Pieces/083 Shops and purchases.md`; the recovered inputs stay in the lab):
//! * Coin-store packets are family 165. `CoinStoreItemListPacket = 1`,
//!   `CoinStoreItemDefinitionsRequestPacket = 2`, `...ResponsePacket = 3`,
//!   `CoinStoreSellToClientRequestPacket = 4` (the client buying from a merchant),
//!   `CoinStoreBuyFromClientRequestPacket = 5` (selling, item 084's scope),
//!   `CoinStoreTransactionCompletePacket = 6`, `CoinStoreMerchantListPacket = 10`
//!   (`lab/decompiled/server/LibSOE/SOE/BaseCoinStorePackets.cs`). The supplied first-party
//!   client builds both the sell-to-client request and the transaction-complete packet
//!   (`CoinStoreSellToClientRequestPacket::vftable`,
//!   `CoinStoreTransactionCompletePacket::vftable` in
//!   `lab/decompiled/client/FreeRealms/pseudocode.c`).
//! * The request body is `MerchantGuid u64, MerchantUnknown i32, ItemRecord { Definition i32,
//!   Tint i32 }, Quantity i32` read with no trailing bytes
//!   (`lab/references/FreeRealms-Legacy/src/Sanctuary.Packet/BaseCoinStorePacket/`).
//! * The transaction-complete results are 1 success, 3 invalid item, 4 invalid quantity,
//!   5 invalid merchant, 6 invalid item tint, 7 balance not enough, 8 service unavailable,
//!   9 recipient not found, 10 not a member and 11 recipient cannot accept
//!   (`Sanctuary.Packet/BaseCoinStorePacket/CoinStoreTransactionCompletePacket.cs`), and a
//!   record carries type 1 (buy) / 2 (sell), the merchant guid, an id, the item record, the
//!   quantity, the price and the remaining quantity (`Sanctuary.Packet.Common/CoinStoreTransactionRecord.cs`).
//! * Catalogue rows are item-definition rows: `Cost`, `MemberDiscount`, `MembersOnly`,
//!   `MaxStackSize`, `IsTintable` and the icon tint. The member price is
//!   `(int)(Cost * (1.0f - MemberDiscount * 0.01f))` when `Cost > 0 && MemberDiscount <= 100`
//!   (`Sanctuary.Packet.Common/ClientItemDefinition.cs`), which matches the wiki's 10% member
//!   discount recorded for the housing lots. The rows themselves are pinned in
//!   `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`.
//! * `BuyDisabled`, `CoinStoreOnly` and `Hidden` are per-item store metadata the client
//!   receives in the coin-store item list (`Sanctuary.Packet.Common/ItemDefinitionMetaData.cs`).
//! * `MaxPerOrder`, `MaxPurchasable` and `QuantityAvailable` are first-party store-item fields:
//!   the client names them in its store data-source table
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`) and the UI reads the same field numbers
//!   (`lab/decompiled/client-ui/ScriptsBase.lua`). `GetMaxItemsPurchasable` and the native
//!   `CoinStoreScreen.CanBuyItem` gate the buy button client-side.
//!
//! RECONSTRUCTED here, labelled at each site: the catalogue document and its bounds, the
//! per-merchant binding of stock and order limits, the ownership cap, the check order, every
//! refusal and the staged-commit transaction. No original server code for a purchase was
//! recovered, so this module is not a native acceptance claim, and it never mints currency or
//! items of its own: coins come from the lead-owned economy layer and the granted units are
//! per-definition counts that item 081's inventory instances must back.
//!
//! Deliberate boundaries:
//! * selling, buy-back, gifts and the Station Cash marketplace are other packet paths and other
//!   work items (084, 085); this module refuses nothing outside sub-opcode 4 because it never
//!   parses those bodies;
//! * the client's native `CanBuyItem` predicate is unrecovered, so the ownership cap is
//!   catalogue data (`owned_limit`) rather than a reimplemented guess;
//! * a request whose tint is wrong for a non-tintable item is refused with 6. The pinned
//!   community handler instead rewrites the tint to the definition's icon tint; that rewrite is
//!   not copied because it silently changes what the client asked for;
//! * `quantity_remaining` stays `None` for an item whose stock is not tracked, because the
//!   wire encoding of "unlimited" is not recovered;
//! * the transaction timestamp and the completion packet's `ItemGuid` are not produced here:
//!   the timestamp encoding is unrecovered and the instance id belongs to item 081.
//!
//! ponytail: one catalogue map, one ledger, one staged clone for the commit. No trait objects,
//! no packet writer, no second index. Add a durable store when the lead owns one.

// RED evidence for the check order above: the first revision of this file mirrored the pinned
// community handler (look up, charge, write, hope). `tests/piece_083.rs` failed 5/13 on the
// missing merchant, quantity, stock, tint, membership, ownership and rollback rules; the
// retained log is `lab/reports/piece-083-20261009T042532Z/red-baseline.log`.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

pub const COIN_STORE_FAMILY: u16 = 165;
pub const COIN_STORE_ITEM_LIST: u16 = 1;
pub const COIN_STORE_ITEM_DEFINITIONS_REQUEST: u16 = 2;
pub const COIN_STORE_ITEM_DEFINITIONS_RESPONSE: u16 = 3;
pub const COIN_STORE_SELL_TO_CLIENT_REQUEST: u16 = 4;
pub const COIN_STORE_BUY_FROM_CLIENT_REQUEST: u16 = 5;
pub const COIN_STORE_TRANSACTION_COMPLETE: u16 = 6;
pub const COIN_STORE_MERCHANT_LIST: u16 = 10;

pub const COIN_STORE_RESULT_SUCCESS: i32 = 1;
pub const COIN_STORE_RESULT_INVALID_ITEM: i32 = 3;
pub const COIN_STORE_RESULT_INVALID_QUANTITY: i32 = 4;
pub const COIN_STORE_RESULT_INVALID_MERCHANT: i32 = 5;
pub const COIN_STORE_RESULT_INVALID_ITEM_TINT: i32 = 6;
pub const COIN_STORE_RESULT_BALANCE_NOT_ENOUGH: i32 = 7;
pub const COIN_STORE_RESULT_SERVICE_UNAVAILABLE: i32 = 8;
pub const COIN_STORE_RESULT_NOT_A_MEMBER: i32 = 10;
pub const COIN_STORE_TRANSACTION_TYPE_BUY: i32 = 1;

pub const COIN_STORE_REQUEST_LENGTH: usize = 28;
pub const SHOP_CATALOGUE_SCHEMA_VERSION: u32 = 1;
pub const MAX_CATALOGUE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_CATALOGUE_ITEMS: usize = 65_536;
pub const MAX_CATALOGUE_MERCHANTS: usize = 4_096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PurchaseRequest {
    pub merchant_guid: u64,
    pub merchant_unknown: i32,
    pub definition_id: i32,
    pub tint: i32,
    pub quantity: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestError {
    Length(usize),
    Family(u16),
    SubOpcode(u16),
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

impl PurchaseRequest {
    pub fn parse(bytes: &[u8]) -> Result<Self, RequestError> {
        if bytes.len() != COIN_STORE_REQUEST_LENGTH {
            return Err(RequestError::Length(bytes.len()));
        }
        let family = u16::from_le_bytes(bytes[..2].try_into().unwrap());
        if family != COIN_STORE_FAMILY {
            return Err(RequestError::Family(family));
        }
        let sub = u16::from_le_bytes(bytes[2..4].try_into().unwrap());
        if sub != COIN_STORE_SELL_TO_CLIENT_REQUEST {
            return Err(RequestError::SubOpcode(sub));
        }
        Ok(Self {
            merchant_guid: u64::from_le_bytes(bytes[4..12].try_into().unwrap()),
            merchant_unknown: read_i32(bytes, 12),
            definition_id: read_i32(bytes, 16),
            tint: read_i32(bytes, 20),
            quantity: read_i32(bytes, 24),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopItem {
    pub definition_id: i32,
    pub cost: u32,
    pub member_discount: u32,
    #[serde(default)]
    pub members_only: bool,
    #[serde(default)]
    pub buy_disabled: bool,
    pub is_tintable: bool,
    pub icon_tint: i32,
    #[serde(default)]
    pub quantity_available: Option<u32>,
    #[serde(default)]
    pub max_per_order: Option<u32>,
    #[serde(default)]
    pub owned_limit: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Merchant {
    pub guid: u64,
    pub items: Vec<ShopItem>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogueDocument {
    schema_version: u32,
    merchants: Vec<Merchant>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogueError {
    UnsupportedSchema(u32),
    TooLarge(usize),
    Corrupt(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShopCatalogue {
    merchants: BTreeSet<u64>,
    items: BTreeMap<(u64, i32), ShopItem>,
}

impl ShopCatalogue {
    pub fn from_json(bytes: &[u8]) -> Result<Self, CatalogueError> {
        if bytes.len() > MAX_CATALOGUE_BYTES {
            return Err(CatalogueError::TooLarge(bytes.len()));
        }
        let document: CatalogueDocument = serde_json::from_slice(bytes)
            .map_err(|error| CatalogueError::Corrupt(error.to_string()))?;
        if document.schema_version != SHOP_CATALOGUE_SCHEMA_VERSION {
            return Err(CatalogueError::UnsupportedSchema(document.schema_version));
        }
        if document.merchants.len() > MAX_CATALOGUE_MERCHANTS {
            return Err(CatalogueError::Corrupt("too many merchants".into()));
        }
        let mut merchants = BTreeSet::new();
        let mut items = BTreeMap::new();
        let mut total = 0usize;
        for merchant in document.merchants {
            if merchant.guid == 0 || !merchants.insert(merchant.guid) {
                return Err(CatalogueError::Corrupt("invalid merchant guid".into()));
            }
            for item in merchant.items {
                total += 1;
                if item.definition_id <= 0
                    || item.member_discount > 100
                    || item.icon_tint < 0
                    || item.cost > i32::MAX as u32
                    || items
                        .insert((merchant.guid, item.definition_id), item)
                        .is_some()
                {
                    return Err(CatalogueError::Corrupt("invalid catalogue row".into()));
                }
            }
        }
        if total > MAX_CATALOGUE_ITEMS {
            return Err(CatalogueError::Corrupt("too many items".into()));
        }
        Ok(Self { merchants, items })
    }

    pub fn merchants(&self) -> &BTreeSet<u64> {
        &self.merchants
    }

    pub fn item(&self, merchant_guid: u64, definition_id: i32) -> Option<&ShopItem> {
        self.items.get(&(merchant_guid, definition_id))
    }
}

/// ORIGINAL formula, reimplemented from the pinned
/// `ClientItemDefinition.GetMemberPurchasePrice()`: float32 discount factor, truncating cast.
pub fn member_unit_price(cost: u32, member_discount: u32) -> u32 {
    if cost > 0 && member_discount <= 100 {
        let discount_factor = member_discount as f32 * 0.01f32;
        let discount_multiplier = 1.0f32 - discount_factor;
        return (discount_multiplier * cost as f32) as u32;
    }
    cost
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buyer {
    pub character_guid: u64,
    pub member: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub transaction_type: i32,
    pub merchant_guid: u64,
    pub id: i32,
    pub definition_id: i32,
    pub tint: i32,
    pub quantity: u32,
    pub price: u32,
    pub quantity_remaining: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PurchaseReceipt {
    pub result: i32,
    pub character_guid: u64,
    pub unit_price: u32,
    pub total_price: u32,
    pub coins_remaining: u32,
    pub record: TransactionRecord,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    UnknownMerchant,
    UnknownItem,
    BuyDisabled,
    InvalidQuantity,
    OrderLimit {
        max: u32,
    },
    OutOfStock {
        available: u32,
    },
    TintNotAllowed {
        expected: i32,
    },
    MembersOnly,
    OwnershipLimit {
        limit: u32,
        owned: u32,
    },
    BalanceNotEnough {
        required: u32,
        available: u32,
    },
    /// The per-character transaction log reached the i32 wire id ceiling.
    TransactionLimit,
}

impl Refusal {
    pub fn result_code(self) -> i32 {
        match self {
            Refusal::UnknownMerchant => COIN_STORE_RESULT_INVALID_MERCHANT,
            Refusal::UnknownItem | Refusal::BuyDisabled => COIN_STORE_RESULT_INVALID_ITEM,
            Refusal::InvalidQuantity
            | Refusal::OrderLimit { .. }
            | Refusal::OutOfStock { .. }
            | Refusal::OwnershipLimit { .. } => COIN_STORE_RESULT_INVALID_QUANTITY,
            Refusal::TintNotAllowed { .. } => COIN_STORE_RESULT_INVALID_ITEM_TINT,
            Refusal::MembersOnly => COIN_STORE_RESULT_NOT_A_MEMBER,
            Refusal::BalanceNotEnough { .. } => COIN_STORE_RESULT_BALANCE_NOT_ENOUGH,
            Refusal::TransactionLimit => COIN_STORE_RESULT_SERVICE_UNAVAILABLE,
        }
    }
}

fn refused(reason: Refusal) -> PurchaseError {
    PurchaseError::Refused {
        result: reason.result_code(),
        reason,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PurchaseError {
    Refused { result: i32, reason: Refusal },
    PersistFailed(io::ErrorKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedStack {
    pub definition_id: i32,
    pub tint: i32,
    pub count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShopLedger {
    coins: BTreeMap<u64, u32>,
    stacks: BTreeMap<u64, Vec<OwnedStack>>,
    transactions: BTreeMap<u64, Vec<TransactionRecord>>,
    stock: BTreeMap<u64, BTreeMap<i32, u32>>,
}

impl ShopLedger {
    pub fn new(catalogue: &ShopCatalogue) -> Self {
        let mut stock: BTreeMap<u64, BTreeMap<i32, u32>> = BTreeMap::new();
        for (&(merchant, definition), item) in &catalogue.items {
            if let Some(available) = item.quantity_available {
                stock
                    .entry(merchant)
                    .or_default()
                    .insert(definition, available);
            }
        }
        Self {
            stock,
            ..Self::default()
        }
    }

    pub fn credit(&mut self, character_guid: u64, coins: u32) {
        *self.coins.entry(character_guid).or_default() = coins;
    }

    pub fn coins(&self, character_guid: u64) -> u32 {
        self.coins.get(&character_guid).copied().unwrap_or(0)
    }

    pub fn owned(&self, character_guid: u64, definition_id: i32) -> u32 {
        self.stacks
            .get(&character_guid)
            .map(|stacks| {
                stacks
                    .iter()
                    .filter(|stack| stack.definition_id == definition_id)
                    .map(|stack| stack.count)
                    .sum()
            })
            .unwrap_or(0)
    }

    pub fn stacks(&self, character_guid: u64, definition_id: i32, tint: i32) -> u32 {
        self.stacks
            .get(&character_guid)
            .map(|stacks| {
                stacks
                    .iter()
                    .filter(|stack| stack.definition_id == definition_id && stack.tint == tint)
                    .map(|stack| stack.count)
                    .sum()
            })
            .unwrap_or(0)
    }

    pub fn stock(&self, merchant_guid: u64, definition_id: i32) -> Option<u32> {
        self.stock
            .get(&merchant_guid)
            .and_then(|items| items.get(&definition_id))
            .copied()
    }

    pub fn transactions(&self, character_guid: u64) -> &[TransactionRecord] {
        self.transactions
            .get(&character_guid)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn purchase<F>(
        &mut self,
        catalogue: &ShopCatalogue,
        buyer: Buyer,
        request: &PurchaseRequest,
        persist: F,
    ) -> Result<PurchaseReceipt, PurchaseError>
    where
        F: FnOnce(&ShopLedger) -> io::Result<()>,
    {
        // 1. The merchant the request names must be a listed merchant (result 5).
        if !catalogue.merchants().contains(&request.merchant_guid) {
            return Err(refused(Refusal::UnknownMerchant));
        }
        // 2. The item must be in that merchant's catalogue and not marked buy-disabled (3).
        let item = match catalogue.item(request.merchant_guid, request.definition_id) {
            Some(item) => item,
            None => return Err(refused(Refusal::UnknownItem)),
        };
        if item.buy_disabled {
            return Err(refused(Refusal::BuyDisabled));
        }
        // 3. Quantity must be a positive count inside the order limit and remaining stock (4).
        if request.quantity < 1 {
            return Err(refused(Refusal::InvalidQuantity));
        }
        let quantity = request.quantity as u32;
        if let Some(max) = item.max_per_order {
            if quantity > max {
                return Err(refused(Refusal::OrderLimit { max }));
            }
        }
        if let Some(available) = self.stock(request.merchant_guid, request.definition_id) {
            if quantity > available {
                return Err(refused(Refusal::OutOfStock { available }));
            }
        }
        // 4. A non-tintable definition only accepts its own icon tint (6).
        if request.tint < 0 || (!item.is_tintable && request.tint != item.icon_tint) {
            return Err(refused(Refusal::TintNotAllowed {
                expected: item.icon_tint,
            }));
        }
        // 5. Membership-only rows refuse a non-member (10).
        if item.members_only && !buyer.member {
            return Err(refused(Refusal::MembersOnly));
        }
        // 6. The character's ownership of this definition must stay inside the row's cap (4).
        if let Some(limit) = item.owned_limit {
            let owned = self.owned(buyer.character_guid, request.definition_id);
            if u64::from(owned) + u64::from(quantity) > u64::from(limit) {
                return Err(refused(Refusal::OwnershipLimit { limit, owned }));
            }
        }
        // 7. The price is the list price, or the member price for a member (7 when short).
        let unit_price = if buyer.member {
            member_unit_price(item.cost, item.member_discount)
        } else {
            item.cost
        };
        let total_price = match unit_price.checked_mul(quantity) {
            Some(total) => total,
            None => return Err(refused(Refusal::InvalidQuantity)),
        };
        let balance = self.coins(buyer.character_guid);
        if balance < total_price {
            return Err(refused(Refusal::BalanceNotEnough {
                required: total_price,
                available: balance,
            }));
        }
        let Some(id) = (self.transactions(buyer.character_guid).len() as i64)
            .checked_add(1)
            .filter(|id| *id <= i32::MAX as i64)
        else {
            return Err(refused(Refusal::TransactionLimit));
        };

        // Commit in a staged copy: every check above has passed, the writer sees the finished
        // state, and only a successful write replaces `self`.
        let mut staged = self.clone();
        let coins_remaining = balance - total_price;
        staged.coins.insert(buyer.character_guid, coins_remaining);
        let stacks = staged.stacks.entry(buyer.character_guid).or_default();
        match stacks.iter_mut().find(|stack| {
            stack.definition_id == request.definition_id && stack.tint == request.tint
        }) {
            Some(stack) => match stack.count.checked_add(quantity) {
                Some(count) => stack.count = count,
                None => return Err(refused(Refusal::InvalidQuantity)),
            },
            None => stacks.push(OwnedStack {
                definition_id: request.definition_id,
                tint: request.tint,
                count: quantity,
            }),
        }
        let quantity_remaining = match staged
            .stock
            .get_mut(&request.merchant_guid)
            .and_then(|items| items.get_mut(&request.definition_id))
        {
            Some(available) => {
                *available -= quantity;
                Some(*available)
            }
            None => None,
        };
        let record = TransactionRecord {
            transaction_type: COIN_STORE_TRANSACTION_TYPE_BUY,
            merchant_guid: request.merchant_guid,
            id: id as i32,
            definition_id: request.definition_id,
            tint: request.tint,
            quantity,
            price: total_price,
            quantity_remaining,
        };
        staged
            .transactions
            .entry(buyer.character_guid)
            .or_default()
            .push(record);
        persist(&staged).map_err(|error| PurchaseError::PersistFailed(error.kind()))?;
        *self = staged;
        Ok(PurchaseReceipt {
            result: COIN_STORE_RESULT_SUCCESS,
            character_guid: buyer.character_guid,
            unit_price,
            total_price,
            coins_remaining,
            record,
        })
    }
}
