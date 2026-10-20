//! Coin Shop purchases and a payment-free Marketplace handshake.
//!
//! Client packet readers establish the family/opcode headers and transaction result fields.
//! Numeric result-code meanings, buy idempotency's caller field, resale pricing, buy-back window,
//! merchant authorization and Marketplace defaults are reconstructed policy (R). No persistence
//! or packet transport is performed here; callers save the returned state before sending replies.
//!
//! Sibling requests (W172). Recovered from the client: 165/2 is `i32 count` then `count x u32`
//! item ids (sender `00c76b10`, writers `00c75580`/`00c75620`) and is answered by 165/3, which the
//! client reads (`00c74e40`, `008e8230`) as `u8 complete` then `i32 count` then `count x u32` ids;
//! it carries no definition records, so an uncaptured catalogue row is delivered first in 35/37.
//! 165/12 is `u32 transaction id` then `u32 quantity` (`00c76600`, `00c759c0`), where the id is
//! the dword after the first u64 of the sale's 165/6 record (`00c71f30`, record +0x10). Native
//! `00c78a10` passes the third 165/13 response word to `01087060`, which subtracts that quantity
//! from client history; quantity zero clears the remainder. The Marketplace list replies 66/17,
//! 19, 21 and 23 all start `u32 status` and, on status 1, carry a counted list (readers
//! `00b416c0`, `00b41800`, `00b41760`, `00b41620`); 17, 21 and 23 put a text between the status
//! and the list. Station Cash, wallet funding and membership stay refused or empty (R).

use crate::inventory::Inventory;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

const COIN_FAMILY: u16 = 0xa5;
const MARKETPLACE_FAMILY: u16 = 0x42;
const MAX_CATALOGUE_BYTES: usize = 16 * 1024 * 1024;
const MAX_PACKET_BYTES: usize = 64 * 1024;
const MAX_GIFT_ITEMS: usize = 256;
const MAX_CLIENT_TRANSACTIONS: usize = 4096;
/// (R) Store data may keep more history, but a session never retains an unbounded sale list.
const MAX_BUY_BACK_WINDOW: usize = 100;
/// (R) The client keeps a longer list; ten is the deployed server policy when data omits a limit.
const DEFAULT_BUY_BACK_WINDOW: usize = 10;
/// (R) Server policy, not a recovered client limit. The client's static item table is a
/// `HashList<ItemDefinitionMetaData,4096,1024>` (vtable `01833bd8`), but the 165/2 reader does not
/// cap the requested list; 4,096 ids keeps one request bounded.
const MAX_DEFINITION_IDS: usize = 4096;
/// Keep each 35/37 batch below the inbound packet cap. A single decoded row is at most 65,536
/// bytes, so an exceptionally large row is sent alone.
const MAX_DEFINITION_BATCH_BYTES: usize = 60 * 1024;
/// (R) Server memory policy, not a recovered client limit (lead-accepted 2026-10-19, F179):
/// bound one 165/2 answer before any record is copied. The prepared catalogue's largest row is
/// 319 bytes, so 4,096 ids are about 1.3 MiB; four MiB leaves threefold headroom without letting
/// a hostile list of maximum-size rows (65,536 bytes each) allocate the whole 32 MiB catalogue.
const MAX_DEFINITION_REPLY_BYTES: usize = 4 * 1024 * 1024;
/// (R) Marketplace requests are tiny (4 bytes, or a 13-byte locale string); anything longer is refused.
const MAX_MARKETPLACE_REQUEST: usize = 64;

// The client dispatches codes 1..=11 through one virtual call. The names appear in the same order
// in the native result function, but the numeric-to-name association is not proven; mapping is (R).
pub const RESULT_SUCCESS: u32 = 1;
pub const RESULT_FAILURE: u32 = 2;
pub const RESULT_INVALID_ITEM: u32 = 3;
pub const RESULT_INVALID_QUANTITY: u32 = 4;
pub const RESULT_INVALID_MERCHANT: u32 = 5;
pub const RESULT_INVALID_TINT: u32 = 6;
pub const RESULT_INSUFFICIENT_COINS: u32 = 7;
pub const RESULT_SERVICE_UNAVAILABLE: u32 = 8;
pub const RESULT_RECIPIENT_NOT_FOUND: u32 = 9;
pub const RESULT_NOT_A_MEMBER: u32 = 10;
pub const RESULT_RECIPIENT_CANNOT_ACCEPT: u32 = 11;

// Marketplace place-order result 1 is success in client reader 00b3b6d0; zero is failure.
pub const MARKETPLACE_RESULT_DISABLED: u32 = 0;

#[derive(Clone, Deserialize)]
pub struct StoreCatalogue {
    schema_version: u32,
    #[serde(default = "default_buy_back_window")]
    buy_back_window: usize,
    marketplace: MarketplaceConfig,
    items: Vec<StoreItem>,
}

fn default_buy_back_window() -> usize {
    DEFAULT_BUY_BACK_WINDOW
}

#[derive(Clone, Deserialize)]
struct MarketplaceConfig {
    enabled_on_handshake: bool,
    orders_enabled: bool,
    payment_sources_enabled: bool,
    not_available_message: String,
}

#[derive(Clone, Deserialize)]
struct StoreItem {
    item_id: i32,
    purchase_price_coins: Option<u32>,
    member_discount_percent: Option<u32>,
    member_price_coins: Option<u32>,
    sell_price_coins: Option<u32>,
    no_sale: bool,
}

impl StoreCatalogue {
    pub fn load(path: &Path) -> io::Result<Self> {
        let file = std::fs::File::open(path)?;
        if !file.metadata()?.is_file() || file.metadata()?.len() > MAX_CATALOGUE_BYTES as u64 {
            return Err(invalid("store catalogue is not a bounded regular file"));
        }
        let mut bytes = Vec::new();
        file.take((MAX_CATALOGUE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_CATALOGUE_BYTES {
            return Err(invalid("store catalogue exceeds 16 MiB"));
        }
        let catalogue: Self = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if catalogue.schema_version != 1
            || catalogue.items.len() > 100_000
            || !(1..=MAX_BUY_BACK_WINDOW).contains(&catalogue.buy_back_window)
            || catalogue.marketplace.orders_enabled
            || catalogue.marketplace.payment_sources_enabled
            || catalogue.marketplace.not_available_message.is_empty()
        {
            return Err(invalid("unsupported or payment-enabled store catalogue"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for item in &catalogue.items {
            if item.item_id <= 0
                || !ids.insert(item.item_id)
                || item
                    .purchase_price_coins
                    .into_iter()
                    .chain(item.member_price_coins)
                    .chain(item.sell_price_coins)
                    .any(|price| price > i32::MAX as u32)
                || item
                    .member_price_coins
                    .zip(item.purchase_price_coins)
                    .is_some_and(|(member, regular)| member > regular)
                || item
                    .member_discount_percent
                    .is_some_and(|discount| discount > 100)
                || (item.no_sale && item.sell_price_coins.is_some())
            {
                return Err(invalid("invalid store item entry"));
            }
        }
        Ok(catalogue)
    }

    fn item(&self, item_id: i32) -> Option<&StoreItem> {
        self.items.iter().find(|item| item.item_id == item_id)
    }

    /// F228: a job starter outfit item is free issuance, so its sell price is 0. A row with
    /// no sell price stays unsellable (`None`); the purchase price is unchanged.
    fn zero_starter_sell_prices(&mut self, starter_items: &BTreeSet<i32>) {
        for item in &mut self.items {
            item.sell_price_coins =
                starter_sell_price(item.item_id, item.sell_price_coins, starter_items);
        }
    }
}

/// F228: a job starter outfit item is free issuance, so every copy resells for 0 wherever the
/// player got it. The Coin Shop catalogue and the 66/42 merchant window both apply this one
/// rule. A row with no sell price stays unsellable (`None`).
pub fn starter_sell_price(
    item_id: i32,
    sell_price: Option<u32>,
    starter_items: &BTreeSet<i32>,
) -> Option<u32> {
    if starter_items.contains(&item_id) {
        sell_price.map(|_| 0)
    } else {
        sell_price
    }
}

/// Inner application messages and the state-change indicator. Persist state before sending replies.
#[derive(Clone, Debug, Default)]
pub struct CoinShopOutcome {
    pub result_code: u32,
    pub transaction_id: u32,
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
}

#[derive(Clone)]
struct SaleRecord {
    transaction_id: u32,
    item_id: i32,
    tint: i32,
    unit_price: u32,
    remaining: u32,
}

#[derive(Clone)]
pub struct CoinShop {
    inventory_content: PathBuf,
    inventory: Inventory,
    catalogue: Arc<StoreCatalogue>,
    bag_capacity: usize,
    is_member: bool,
    client_transactions: BTreeMap<u32, (u32, Vec<u8>)>,
    vendor_transactions: BTreeMap<u32, (u32, Vec<u8>)>,
    sales: VecDeque<SaleRecord>,
    next_transaction_id: u32,
}

impl CoinShop {
    pub fn new(
        inventory_content: &Path,
        inventory: Inventory,
        mut catalogue: StoreCatalogue,
        bag_capacity: usize,
        profiles: Option<&crate::profiles::Profiles>,
    ) -> io::Result<Self> {
        // F228: a starter outfit item is granted free whenever its job is learned or reselected,
        // so the store pays 0 for it however the player got a copy. Reuse the session's loaded
        // profiles: a second parse would also reject a package whose saved progression is valid
        // but whose unused `default_state` is not. Purchase prices are untouched.
        if let Some(profiles) = profiles {
            catalogue.zero_starter_sell_prices(&profiles.all_starter_items());
        }
        Ok(Self {
            inventory_content: inventory_content.to_path_buf(),
            inventory,
            catalogue: Arc::new(catalogue),
            bag_capacity,
            // (R) The caller must set the verified account flag after construction.
            is_member: false,
            client_transactions: BTreeMap::new(),
            vendor_transactions: BTreeMap::new(),
            sales: VecDeque::new(),
            next_transaction_id: 1,
        })
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    /// Trade against the player's live inventory: coins and items earned
    /// since the shop was created must not be lost or ignored on a purchase.
    pub fn sync_inventory(&mut self, inventory: Inventory) {
        self.inventory = inventory;
    }

    pub fn set_member_status(&mut self, is_member: bool) {
        self.is_member = is_member;
    }

    /// Apply one 66/43 vendor buy using the store catalogue's server-side price.
    /// The request field mapping and transaction id are reconstructed policy (R).
    pub fn buy_vendor(
        &mut self,
        client_transaction_id: u32,
        item_id: i32,
        quantity: u32,
    ) -> io::Result<CoinShopOutcome> {
        if let Some((result_code, response)) = self.vendor_transactions.get(&client_transaction_id)
        {
            return Ok(CoinShopOutcome {
                result_code: *result_code,
                transaction_id: client_transaction_id,
                replies: vec![response.clone()],
                changed: false,
            });
        }
        // (R) Keep the session idempotency cache bounded under client-controlled IDs.
        if self.vendor_transactions.len() >= MAX_CLIENT_TRANSACTIONS {
            return Ok(vendor_refusal(
                client_transaction_id,
                RESULT_SERVICE_UNAVAILABLE,
            ));
        }

        let mut outcome = self.buy_once(client_transaction_id, item_id, quantity, 0)?;
        outcome.replies.pop();
        let response = merchant_buy_result_packet(outcome.result_code == RESULT_SUCCESS);
        outcome.replies.push(response.clone());
        outcome.transaction_id = client_transaction_id;
        if outcome.changed {
            self.vendor_transactions
                .insert(client_transaction_id, (outcome.result_code, response));
        }
        Ok(outcome)
    }

    /// (R) Refusals are retryable; return the native-visible 66/44 response without caching them.
    pub fn refuse_vendor_buy(
        &mut self,
        client_transaction_id: u32,
        result_code: u32,
    ) -> CoinShopOutcome {
        if let Some((result_code, response)) = self.vendor_transactions.get(&client_transaction_id)
        {
            return CoinShopOutcome {
                result_code: *result_code,
                transaction_id: client_transaction_id,
                replies: vec![response.clone()],
                changed: false,
            };
        }
        vendor_refusal(client_transaction_id, result_code)
    }

    /// Handle one inner client packet. Unsupported messages remain outside this service.
    pub fn handle(&mut self, inner: &[u8]) -> io::Result<CoinShopOutcome> {
        if inner.len() < 4 || inner.len() > MAX_PACKET_BYTES {
            return Err(invalid("invalid store packet length"));
        }
        match read_u16(inner, 0)? {
            COIN_FAMILY => self.handle_coin_store(inner),
            MARKETPLACE_FAMILY => self.handle_marketplace(inner),
            _ => Err(invalid("not a store packet")),
        }
    }

    fn handle_coin_store(&mut self, inner: &[u8]) -> io::Result<CoinShopOutcome> {
        let opcode = read_u16(inner, 2)?;
        match opcode {
            // (R) The first undocumented caller dword is used as the client idempotency key.
            // The client writes two caller values, but their semantics are unresolved.
            4 if inner.len() == 28 => self.buy_with_merchant(
                read_u32(inner, 12)?,
                read_i32(inner, 16)?,
                read_u32(inner, 20)?,
                read_u64(inner, 4)?,
            ),
            5 if inner.len() == 20 => self.sell_with_merchant(
                read_i32(inner, 12)?,
                read_u32(inner, 16)?,
                read_u64(inner, 4)?,
            ),
            2 => self.item_definitions(inner),
            // (R) The dynamic list is always empty; the client clears and refills it from 165/9.
            8 if inner.len() == 4 => Ok(CoinShopOutcome {
                replies: vec![counted_empty(COIN_FAMILY, 9)],
                ..CoinShopOutcome::default()
            }),
            12 if inner.len() == 12 => {
                self.buy_back_with_context(read_u32(inner, 4)?, read_u32(inner, 8)?)
            }
            8 | 12 => Err(invalid("malformed Coin Store request")),
            14 => self.refuse_gift(inner),
            _ => Ok(CoinShopOutcome::default()),
        }
    }

    pub fn buy(
        &mut self,
        client_transaction_id: u32,
        item_id: i32,
        quantity: u32,
    ) -> io::Result<CoinShopOutcome> {
        self.buy_with_merchant(client_transaction_id, item_id, quantity, 0)
    }

    fn buy_with_merchant(
        &mut self,
        client_transaction_id: u32,
        item_id: i32,
        quantity: u32,
        merchant_id: u64,
    ) -> io::Result<CoinShopOutcome> {
        if let Some((result_code, completion)) =
            self.client_transactions.get(&client_transaction_id)
        {
            return Ok(CoinShopOutcome {
                result_code: *result_code,
                transaction_id: client_transaction_id,
                replies: vec![completion.clone()],
                changed: false,
            });
        }
        if self.client_transactions.len() >= MAX_CLIENT_TRANSACTIONS {
            return Ok(self.failure(
                1,
                RESULT_SERVICE_UNAVAILABLE,
                client_transaction_id,
                item_id,
                quantity,
                0,
                0,
            ));
        }

        let outcome = self.buy_once(client_transaction_id, item_id, quantity, merchant_id)?;
        let completion = outcome
            .replies
            .last()
            .cloned()
            .expect("buy outcome always carries a transaction response");
        self.client_transactions
            .insert(client_transaction_id, (outcome.result_code, completion));
        Ok(outcome)
    }

    fn buy_once(
        &mut self,
        transaction_id: u32,
        item_id: i32,
        quantity: u32,
        merchant_id: u64,
    ) -> io::Result<CoinShopOutcome> {
        if item_id <= 0
            || self.catalogue.item(item_id).is_none()
            || !self.inventory.has_definition(item_id)
        {
            return Ok(self.failure(
                1,
                RESULT_INVALID_ITEM,
                transaction_id,
                item_id,
                quantity,
                0,
                merchant_id,
            ));
        }
        if quantity == 0 || quantity > i32::MAX as u32 {
            return Ok(self.failure(
                1,
                RESULT_INVALID_QUANTITY,
                transaction_id,
                item_id,
                quantity,
                0,
                merchant_id,
            ));
        }
        let item = self.catalogue.item(item_id).expect("item was checked");
        let unit_price = if self.is_member {
            item.member_price_coins.or(item.purchase_price_coins)
        } else {
            item.purchase_price_coins
        };
        let Some(unit_price) = unit_price else {
            return Ok(self.failure(
                1,
                RESULT_SERVICE_UNAVAILABLE,
                transaction_id,
                item_id,
                quantity,
                0,
                merchant_id,
            ));
        };
        // The account membership flag must be supplied by the caller; member price arithmetic is (R).
        let Some(total_price) = unit_price.checked_mul(quantity) else {
            return Ok(self.failure(
                1,
                RESULT_INVALID_QUANTITY,
                transaction_id,
                item_id,
                quantity,
                unit_price,
                merchant_id,
            ));
        };
        if self.inventory.state().coins < total_price {
            return Ok(self.failure(
                1,
                RESULT_INSUFFICIENT_COINS,
                transaction_id,
                item_id,
                quantity,
                unit_price,
                merchant_id,
            ));
        }
        let mut next_state = self.inventory.state().clone();
        next_state.coins -= total_price;
        let mut staged = Inventory::load(&self.inventory_content, Some(next_state))?;
        let granted = match staged.grant(0, item_id, quantity) {
            Ok(granted) => granted,
            Err(_) => {
                return Ok(self.failure(
                    1,
                    RESULT_SERVICE_UNAVAILABLE,
                    transaction_id,
                    item_id,
                    quantity,
                    unit_price,
                    merchant_id,
                ));
            }
        };
        if staged.state().items.len() > self.bag_capacity {
            return Ok(self.failure(
                1,
                RESULT_SERVICE_UNAVAILABLE,
                transaction_id,
                item_id,
                quantity,
                unit_price,
                merchant_id,
            ));
        }

        let mut replies = vec![coin_balance_packet(staged.state().coins)];
        replies.extend(granted.replies);
        let completion = transaction_complete(
            1,
            RESULT_SUCCESS,
            transaction_id,
            item_id,
            0,
            quantity,
            unit_price,
            merchant_id,
        );
        replies.push(completion);
        self.inventory = staged;
        Ok(CoinShopOutcome {
            result_code: RESULT_SUCCESS,
            transaction_id,
            replies,
            changed: true,
        })
    }

    /// Client-readable 165/6 refusal for a 165/5 sell the NPC layer rejected before the shop saw
    /// it (closed shop, range, merchant). Echoes the request's fields when it is well formed.
    /// Nothing is read from or changed in the inventory; the refusal is not recorded as a sale (R).
    pub fn refuse_sell(&mut self, inner: &[u8], result_code: u32) -> CoinShopOutcome {
        let (merchant, item, quantity) = match inner {
            [0xa5, 0, 5, 0, rest @ ..] if rest.len() == 16 => (
                u64::from_le_bytes(rest[..8].try_into().unwrap()),
                i32::from_le_bytes(rest[8..12].try_into().unwrap()),
                u32::from_le_bytes(rest[12..].try_into().unwrap()),
            ),
            _ => (0, 0, 0),
        };
        let transaction_id = self.allocate_transaction_id();
        self.failure(2, result_code, transaction_id, item, quantity, 0, merchant)
    }

    /// 165/2: deliver uncaptured requested definitions in bounded 35/37 batches, then answer with
    /// the requested ids the item catalogue defines, in request order and without duplicates. The
    /// complete flag is true when every id was known. A malformed, oversized or over-budget list
    /// gets an empty incomplete answer, because the client waits for one reply per request (its
    /// search stays pending otherwise, `00c76e70`). The record budget is decided before a single
    /// row is copied, so a repeated bundled request cannot accumulate a definition payload.
    /// Read-only (R).
    fn item_definitions(&self, inner: &[u8]) -> io::Result<CoinShopOutcome> {
        let valid = read_i32(inner, 4)
            .ok()
            .and_then(|count| usize::try_from(count).ok())
            .is_some_and(|count| count <= MAX_DEFINITION_IDS && inner.len() == 8 + 4 * count);
        if !valid {
            return Ok(definition_outcome(false, &[]));
        }
        let (mut known, mut seen) = (Vec::new(), BTreeSet::new());
        // References only: the request is refused before any record bytes are copied.
        let mut pending = Vec::new();
        let mut complete = true;
        let mut deliver_bytes = 0usize;
        for chunk in inner[8..].chunks_exact(4) {
            let id = u32::from_le_bytes(chunk.try_into().unwrap());
            let defined = i32::try_from(id).is_ok_and(|id| self.inventory.has_definition(id));
            complete &= defined;
            if defined && seen.insert(id) {
                known.push(id);
                if !self.inventory.definition_was_captured(id as i32) {
                    let record = self
                        .inventory
                        .definition_record(id as i32)
                        .expect("definition existence was checked");
                    deliver_bytes = deliver_bytes.saturating_add(record.len());
                    pending.push(record);
                }
            }
        }
        // (R) Refuse before copying records; the client still gets its one 165/3 answer.
        if deliver_bytes > MAX_DEFINITION_REPLY_BYTES {
            return Ok(definition_outcome(false, &[]));
        }
        let mut batches = Vec::new();
        let mut batch = Vec::new();
        let mut batch_bytes = 0usize;
        for record in pending {
            if !batch.is_empty()
                && batch_bytes.saturating_add(record.len()) > MAX_DEFINITION_BATCH_BYTES
            {
                batches.push(
                    crate::inventory_v2_fixes::encode_definition_batch_v2(&batch, &[])
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
                );
                batch.clear();
                batch_bytes = 0;
            }
            batch_bytes += record.len();
            batch.push(record.to_vec());
        }
        if !batch.is_empty() {
            batches.push(
                crate::inventory_v2_fixes::encode_definition_batch_v2(&batch, &[])
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
            );
        }
        let mut outcome = definition_outcome(complete, &known);
        outcome.replies.splice(0..0, batches);
        Ok(outcome)
    }

    /// Client-readable 165/13 refusal for a 165/12 the NPC layer rejected before the shop saw it
    /// (closed shop, range). Echoes the request's id and quantity; changes nothing (R).
    pub fn refuse_buy_back(&self, inner: &[u8], result_code: u32) -> CoinShopOutcome {
        let (transaction, context) = match inner {
            [0xa5, 0, 12, 0, rest @ ..] if rest.len() == 8 => (
                u32::from_le_bytes(rest[..4].try_into().unwrap()),
                u32::from_le_bytes(rest[4..].try_into().unwrap()),
            ),
            _ => (0, 0),
        };
        buy_back_result(transaction, result_code, context)
    }

    pub fn sell(&mut self, item_instance_id: i32, quantity: u32) -> io::Result<CoinShopOutcome> {
        self.sell_with_merchant(item_instance_id, quantity, 0)
    }

    fn sell_with_merchant(
        &mut self,
        item_instance_id: i32,
        quantity: u32,
        merchant_id: u64,
    ) -> io::Result<CoinShopOutcome> {
        let transaction_id = self.allocate_transaction_id();
        if item_instance_id <= 0 {
            return Ok(self.failure(
                2,
                RESULT_INVALID_ITEM,
                transaction_id,
                item_instance_id,
                quantity,
                0,
                merchant_id,
            ));
        }
        if quantity == 0 || quantity > i32::MAX as u32 {
            return Ok(self.failure(
                2,
                RESULT_INVALID_QUANTITY,
                transaction_id,
                item_instance_id,
                quantity,
                0,
                merchant_id,
            ));
        }
        let Some(owned) = self
            .inventory
            .state()
            .items
            .iter()
            .find(|item| item.id == item_instance_id)
            .cloned()
        else {
            return Ok(self.failure(
                2,
                RESULT_INVALID_ITEM,
                transaction_id,
                item_instance_id,
                quantity,
                0,
                merchant_id,
            ));
        };
        // (R) Only copies that are neither consumed nor stored away can be sold, matching the
        // trade availability rule; selling into them left stored_count above the stack count.
        let available = owned
            .count
            .saturating_sub(owned.consumed_count.saturating_add(owned.stored_count));
        if quantity > available {
            return Ok(self.failure(
                2,
                RESULT_INVALID_QUANTITY,
                transaction_id,
                owned.definition_id,
                quantity,
                0,
                merchant_id,
            ));
        }
        if owned.rental.is_some()
            || self
                .inventory
                .state()
                .equipment
                .iter()
                .any(|entry| entry.item_id == item_instance_id)
        {
            return Ok(self.failure(
                2,
                RESULT_SERVICE_UNAVAILABLE,
                transaction_id,
                owned.definition_id,
                quantity,
                0,
                merchant_id,
            ));
        }
        let Some(item) = self.catalogue.item(owned.definition_id) else {
            return Ok(self.failure(
                2,
                RESULT_INVALID_ITEM,
                transaction_id,
                owned.definition_id,
                quantity,
                0,
                merchant_id,
            ));
        };
        if item.no_sale {
            return Ok(self.failure(
                2,
                RESULT_SERVICE_UNAVAILABLE,
                transaction_id,
                owned.definition_id,
                quantity,
                0,
                merchant_id,
            ));
        }
        let Some(unit_price) = item.sell_price_coins else {
            return Ok(self.failure(
                2,
                RESULT_SERVICE_UNAVAILABLE,
                transaction_id,
                owned.definition_id,
                quantity,
                0,
                merchant_id,
            ));
        };
        let Some(total_price) = unit_price.checked_mul(quantity) else {
            return Ok(self.failure(
                2,
                RESULT_INVALID_QUANTITY,
                transaction_id,
                owned.definition_id,
                quantity,
                unit_price,
                merchant_id,
            ));
        };
        let mut staged = Inventory::load(
            &self.inventory_content,
            Some(self.inventory.state().clone()),
        )?;
        let removed = match staged.consume(item_instance_id, quantity) {
            Ok(removed) => removed,
            Err(_) => {
                return Ok(self.failure(
                    2,
                    RESULT_SERVICE_UNAVAILABLE,
                    transaction_id,
                    owned.definition_id,
                    quantity,
                    unit_price,
                    merchant_id,
                ));
            }
        };
        if staged.grant_coins(total_price).is_err() {
            return Ok(self.failure(
                2,
                RESULT_SERVICE_UNAVAILABLE,
                transaction_id,
                owned.definition_id,
                quantity,
                unit_price,
                merchant_id,
            ));
        }

        let mut sold_item = owned.clone();
        sold_item.count = quantity;
        sold_item.consumed_count = sold_item.consumed_count.min(quantity);
        self.sales.push_back(SaleRecord {
            transaction_id,
            item_id: owned.definition_id,
            tint: owned.tint,
            unit_price,
            remaining: quantity,
        });
        while self.sales.len() > self.catalogue.buy_back_window {
            self.sales.pop_front();
        }

        let mut replies = removed.replies;
        replies.push(coin_balance_packet(staged.state().coins));
        replies.push(transaction_complete(
            2,
            RESULT_SUCCESS,
            transaction_id,
            sold_item.definition_id,
            sold_item.tint,
            quantity,
            unit_price,
            merchant_id,
        ));
        self.inventory = staged;
        Ok(CoinShopOutcome {
            result_code: RESULT_SUCCESS,
            transaction_id,
            replies,
            changed: true,
        })
    }

    pub fn buy_back(&mut self, sale_transaction_id: u32) -> io::Result<CoinShopOutcome> {
        self.buy_back_with_context(sale_transaction_id, 0)
    }

    fn buy_back_with_context(
        &mut self,
        sale_transaction_id: u32,
        client_context: u32,
    ) -> io::Result<CoinShopOutcome> {
        let Some(index) = self
            .sales
            .iter()
            .position(|sale| sale.transaction_id == sale_transaction_id && sale.remaining > 0)
        else {
            return Ok(buy_back_result(
                sale_transaction_id,
                RESULT_FAILURE,
                client_context,
            ));
        };
        let sale = self.sales[index].clone();
        let wanted = match client_context {
            0 => sale.remaining,
            quantity if quantity <= sale.remaining => quantity,
            _ => {
                return Ok(buy_back_result(
                    sale_transaction_id,
                    RESULT_FAILURE,
                    client_context,
                ));
            }
        };
        let Some(total_price) = sale.unit_price.checked_mul(wanted) else {
            return Ok(buy_back_result(
                sale_transaction_id,
                RESULT_FAILURE,
                client_context,
            ));
        };
        if self.inventory.state().coins < total_price {
            return Ok(buy_back_result(
                sale_transaction_id,
                RESULT_FAILURE,
                client_context,
            ));
        }
        let mut next_state = self.inventory.state().clone();
        next_state.coins -= total_price;
        let mut staged = Inventory::load(&self.inventory_content, Some(next_state))?;
        let restored = match staged.grant_with_tint(0, sale.item_id, wanted, sale.tint) {
            Ok(restored) => restored,
            Err(_) => {
                return Ok(buy_back_result(
                    sale_transaction_id,
                    RESULT_SERVICE_UNAVAILABLE,
                    client_context,
                ));
            }
        };
        if staged.state().items.len() > self.bag_capacity {
            return Ok(buy_back_result(
                sale_transaction_id,
                RESULT_SERVICE_UNAVAILABLE,
                client_context,
            ));
        }
        self.sales[index].remaining -= wanted;
        let mut replies = vec![coin_balance_packet(staged.state().coins)];
        replies.extend(restored.replies);
        replies.push(buy_back_packet(
            sale_transaction_id,
            RESULT_SUCCESS,
            client_context,
        ));
        self.inventory = staged;
        Ok(CoinShopOutcome {
            result_code: RESULT_SUCCESS,
            transaction_id: sale_transaction_id,
            replies,
            changed: true,
        })
    }

    fn allocate_transaction_id(&mut self) -> u32 {
        let id = self.next_transaction_id.max(1);
        self.next_transaction_id = id.wrapping_add(1).max(1);
        id
    }

    fn failure(
        &self,
        kind: u32,
        result_code: u32,
        transaction_id: u32,
        item_id: i32,
        quantity: u32,
        unit_price: u32,
        merchant_id: u64,
    ) -> CoinShopOutcome {
        CoinShopOutcome {
            result_code,
            transaction_id,
            replies: vec![transaction_complete(
                kind,
                result_code,
                transaction_id,
                item_id,
                0,
                quantity,
                unit_price,
                merchant_id,
            )],
            changed: false,
        }
    }

    fn refuse_gift(&self, inner: &[u8]) -> io::Result<CoinShopOutcome> {
        if inner.len() < 24 {
            return Err(invalid("truncated coin gift request"));
        }
        let count = read_i32(inner, 20)?;
        if !(0..=MAX_GIFT_ITEMS as i32).contains(&count) || inner.len() != 24 + count as usize * 8 {
            return Err(invalid("invalid coin gift item list"));
        }
        // The client has a Coin Shop gift path (sub 14/18), but cross-account atomic persistence
        // is not available at this layer. Refuse visibly (R); do not debit or grant either account.
        let mut reply = header(COIN_FAMILY, 18);
        put_u32(&mut reply, RESULT_SERVICE_UNAVAILABLE);
        put_u64(&mut reply, 0);
        put_u64(&mut reply, 0);
        put_i32(&mut reply, 0);
        Ok(CoinShopOutcome {
            result_code: RESULT_SERVICE_UNAVAILABLE,
            replies: vec![reply],
            changed: false,
            ..CoinShopOutcome::default()
        })
    }

    /// Startup enables the Marketplace shell while disabling both payment sources (R).
    pub fn marketplace_initial_packets(&self) -> Vec<Vec<u8>> {
        vec![
            vec![
                66,
                0,
                24,
                0,
                u8::from(self.catalogue.marketplace.enabled_on_handshake),
            ],
            vec![66, 0, 33, 0, 0, 0],
        ]
    }

    /// Answer gateway Marketplace requests without enabling any paid order (R).
    pub fn handle_marketplace(&mut self, inner: &[u8]) -> io::Result<CoinShopOutcome> {
        if inner.len() < 4 || inner.len() > MAX_PACKET_BYTES {
            return Err(invalid("invalid Marketplace packet length"));
        }
        if read_u16(inner, 0)? != MARKETPLACE_FAMILY {
            return Err(invalid("not a Marketplace packet"));
        }
        let opcode = read_u16(inner, 2)?;
        if !marketplace_request_is_well_formed(opcode, inner) {
            return Err(invalid("malformed Marketplace request"));
        }
        let (result_code, response) = match opcode {
            1 => (MARKETPLACE_RESULT_DISABLED, preview_disabled_packet()),
            3 => (
                MARKETPLACE_RESULT_DISABLED,
                order_disabled_packet(&self.catalogue.marketplace.not_available_message),
            ),
            10 => (1, wallet_packet()),
            12 => (
                1,
                marketplace_bool_packet(13, self.catalogue.marketplace.enabled_on_handshake),
            ),
            14 => (1, empty_product_list(15)),
            // Status 1 with an empty list is the client's own "no data was available" path
            // (`00b460c0`, `00b46a50`, `00b46200`, `00b45ba0`). A header-only reply read as an
            // overrun and logged a serialize failure instead.
            16 => (1, empty_list_with_text(17)),
            18 => (1, empty_product_list(19)),
            20 => (1, empty_list_with_text(21)),
            22 => (1, empty_list_with_text(23)),
            25 => (1, account_info_packet()),
            // Bundle content (27) and item requirements (38): the Station Cash catalogue is empty
            // (R), so the client has no bundle to ask about; the layouts of both requests are
            // unrecovered. Refused with no reply and no state change.
            27 | 38 => return Ok(CoinShopOutcome::default()),
            // NpcMerchantV2 buy reply: every result other than 1 is a client-visible refusal.
            43 => (
                MARKETPLACE_RESULT_DISABLED,
                merchant_buy_result_packet(false),
            ),
            _ => return Ok(CoinShopOutcome::default()),
        };
        Ok(CoinShopOutcome {
            result_code,
            replies: vec![response],
            changed: false,
            ..CoinShopOutcome::default()
        })
    }
}

fn transaction_complete(
    kind: u32,
    result: u32,
    transaction_id: u32,
    item_id: i32,
    tint: i32,
    quantity: u32,
    unit_price: u32,
    merchant_id: u64,
) -> Vec<u8> {
    // Client reader `00c74ef0`/`00c71f30` (handler `00c78220`): two dwords, then a record. The first
    // dword is the result: the handler tests it against 1 and feeds it to the result dispatch
    // `00c6f300`, so a sell completion that led with its kind (2) read as a failure. Record order:
    // dword, u64, dword, u64, item pair (`008cb540`: definition, tint), then quantity, price and
    // quantity remaining (`[0xc]`, `[0xe]`, `[0xd]`). The size pattern is recovered; the field
    // meanings (kind, merchant, id, timestamp) follow the DS column names and the community packet
    // order and are (R). The second leading dword is the item the transaction concerns (R); the
    // timestamp is zero (R).
    let remaining = if kind == 2 && result == RESULT_SUCCESS {
        quantity
    } else {
        0
    };
    let mut bytes = header(COIN_FAMILY, 6);
    put_u32(&mut bytes, result);
    put_i32(&mut bytes, item_id);
    put_u32(&mut bytes, kind);
    put_u64(&mut bytes, merchant_id);
    put_u32(&mut bytes, transaction_id);
    put_u64(&mut bytes, 0);
    put_i32(&mut bytes, item_id);
    put_i32(&mut bytes, tint);
    put_u32(&mut bytes, quantity);
    put_u32(&mut bytes, unit_price);
    put_u32(&mut bytes, remaining);
    bytes
}

fn buy_back_result(transaction_id: u32, result: u32, context: u32) -> CoinShopOutcome {
    CoinShopOutcome {
        result_code: result,
        transaction_id,
        replies: vec![buy_back_packet(transaction_id, result, context)],
        changed: false,
    }
}

fn buy_back_packet(transaction_id: u32, result: u32, context: u32) -> Vec<u8> {
    let mut bytes = header(COIN_FAMILY, 13);
    put_u32(&mut bytes, transaction_id);
    put_u32(&mut bytes, result);
    put_u32(&mut bytes, context);
    bytes
}

fn coin_balance_packet(balance: u32) -> Vec<u8> {
    let mut bytes = header(38, 19);
    put_i32(&mut bytes, balance as i32);
    bytes
}

/// 165/3 as the client reads it (`00c74e40`, `008e8230`): flag, count, then the known ids.
fn definition_outcome(complete: bool, known: &[u32]) -> CoinShopOutcome {
    let mut reply = header(COIN_FAMILY, 3);
    reply.push(u8::from(complete));
    put_i32(&mut reply, known.len() as i32);
    known.iter().for_each(|id| put_u32(&mut reply, *id));
    CoinShopOutcome {
        result_code: if complete {
            RESULT_SUCCESS
        } else {
            RESULT_FAILURE
        },
        replies: vec![reply],
        ..CoinShopOutcome::default()
    }
}

fn counted_empty(family: u16, opcode: u16) -> Vec<u8> {
    let mut bytes = header(family, opcode);
    put_i32(&mut bytes, 0);
    bytes
}

fn account_info_packet() -> Vec<u8> {
    // 00b2e080 reads status, two length-prefixed strings, and a final boolean.
    let mut bytes = header(MARKETPLACE_FAMILY, 26);
    put_u32(&mut bytes, 1);
    put_text(&mut bytes, "US");
    put_text(&mut bytes, "USD");
    bytes.push(0);
    bytes
}

fn wallet_packet() -> Vec<u8> {
    // Wallet balance is zero and no payment action is available (R); match the client's fixed reader.
    let mut bytes = header(MARKETPLACE_FAMILY, 11);
    put_u32(&mut bytes, 1);
    bytes.push(0);
    put_u32(&mut bytes, 0);
    put_u32(&mut bytes, 0);
    put_text(&mut bytes, "");
    put_text(&mut bytes, "");
    bytes.push(0);
    bytes
}

/// Status 1 and an empty counted list: Station Cash SKUs (15, `00b415c0`) and state codes (19,
/// `00b41800`).
fn empty_product_list(opcode: u16) -> Vec<u8> {
    let mut bytes = header(MARKETPLACE_FAMILY, opcode);
    put_u32(&mut bytes, 1);
    put_i32(&mut bytes, 0);
    bytes
}

/// Status 1, an empty text and an empty counted list: currency codes (17, `00b416c0`), country
/// codes (21, `00b41760`) and membership SKUs (23, `00b41620`).
fn empty_list_with_text(opcode: u16) -> Vec<u8> {
    let mut bytes = header(MARKETPLACE_FAMILY, opcode);
    put_u32(&mut bytes, 1);
    put_i32(&mut bytes, 0);
    put_i32(&mut bytes, 0);
    bytes
}

/// Observed client requests (62 native traces): 66/10 and 66/18 are the bare header; 66/16, 20
/// and 25 add an `i32` length and that many locale bytes (`en_US`). Rows with no observed sender
/// are only length-bounded (R).
fn marketplace_request_is_well_formed(opcode: u16, inner: &[u8]) -> bool {
    match (opcode, inner.get(4..8)) {
        (10 | 18, _) => inner.len() == 4,
        (16 | 20 | 25, Some(length)) => {
            let length = i32::from_le_bytes(length.try_into().unwrap());
            usize::try_from(length).is_ok_and(|n| n <= 32 && inner.len() == 8 + n)
        }
        (16 | 20 | 25, None) => false,
        _ => inner.len() <= MAX_MARKETPLACE_REQUEST,
    }
}

fn marketplace_bool_packet(opcode: u16, enabled: bool) -> Vec<u8> {
    let mut bytes = header(MARKETPLACE_FAMILY, opcode);
    bytes.push(u8::from(enabled));
    bytes
}

fn preview_disabled_packet() -> Vec<u8> {
    let mut bytes = header(MARKETPLACE_FAMILY, 2);
    for _ in 0..4 {
        put_u32(&mut bytes, MARKETPLACE_RESULT_DISABLED);
    }
    bytes
}

fn order_disabled_packet(message: &str) -> Vec<u8> {
    // Reader 00b2ded0 consumes two dwords, a length-prefixed string, then two dwords.
    let mut bytes = header(MARKETPLACE_FAMILY, 4);
    put_u32(&mut bytes, MARKETPLACE_RESULT_DISABLED);
    put_u32(&mut bytes, 0);
    put_text(&mut bytes, message);
    put_u32(&mut bytes, 0);
    put_u32(&mut bytes, 0);
    bytes
}

/// 66/44 response. Native reader `00b32430` consumes seven dwords, an empty
/// length-prefixed string, then five dwords; object field +0x58 (wire offset 48)
/// is the status compared with 1 by dispatcher `00b48a10`. Other fields are R.
pub fn merchant_buy_result_packet(success: bool) -> Vec<u8> {
    let mut bytes = header(MARKETPLACE_FAMILY, 44);
    for _ in 0..7 {
        put_u32(&mut bytes, 0);
    }
    put_i32(&mut bytes, 0);
    for field in 0..5 {
        put_u32(&mut bytes, u32::from(success && field == 3));
    }
    bytes
}

fn vendor_refusal(transaction_id: u32, result_code: u32) -> CoinShopOutcome {
    CoinShopOutcome {
        result_code,
        transaction_id,
        replies: vec![merchant_buy_result_packet(false)],
        changed: false,
    }
}

fn header(family: u16, opcode: u16) -> Vec<u8> {
    [family.to_le_bytes(), opcode.to_le_bytes()].concat()
}

fn put_text(bytes: &mut Vec<u8>, value: &str) {
    put_i32(bytes, value.len() as i32);
    bytes.extend_from_slice(value.as_bytes());
}

fn put_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> io::Result<u16> {
    Ok(u16::from_le_bytes(
        bytes
            .get(offset..offset + 2)
            .ok_or_else(|| invalid("truncated store field"))?
            .try_into()
            .unwrap(),
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| invalid("truncated store field"))?
            .try_into()
            .unwrap(),
    ))
}

fn read_i32(bytes: &[u8], offset: usize) -> io::Result<i32> {
    Ok(i32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| invalid("truncated store field"))?
            .try_into()
            .unwrap(),
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> io::Result<u64> {
    Ok(u64::from_le_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or_else(|| invalid("truncated store field"))?
            .try_into()
            .unwrap(),
    ))
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
