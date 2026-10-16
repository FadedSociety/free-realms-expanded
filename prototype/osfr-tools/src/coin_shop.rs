//! Coin Shop purchases and a payment-free Marketplace handshake.
//!
//! Client packet readers establish the family/opcode headers and transaction result fields.
//! Numeric result-code meanings, buy idempotency's caller field, resale pricing, buy-back window,
//! merchant authorization and Marketplace defaults are reconstructed policy (R). No persistence
//! or packet transport is performed here; callers save the returned state before sending replies.

use crate::inventory::Inventory;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, VecDeque},
    io::{self, Read},
    path::{Path, PathBuf},
};

const COIN_FAMILY: u16 = 0xa5;
const MARKETPLACE_FAMILY: u16 = 0x42;
const MAX_CATALOGUE_BYTES: usize = 16 * 1024 * 1024;
const MAX_PACKET_BYTES: usize = 64 * 1024;
const MAX_GIFT_ITEMS: usize = 256;
const MAX_CLIENT_TRANSACTIONS: usize = 4096;
const BUY_BACK_WINDOW: usize = 20;

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
    marketplace: MarketplaceConfig,
    items: Vec<StoreItem>,
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
    quantity: u32,
    buy_back_price: u32,
    available: bool,
}

pub struct CoinShop {
    inventory_content: PathBuf,
    inventory: Inventory,
    catalogue: StoreCatalogue,
    bag_capacity: usize,
    is_member: bool,
    client_transactions: BTreeMap<u32, (u32, Vec<u8>)>,
    sales: VecDeque<SaleRecord>,
    next_transaction_id: u32,
}

impl CoinShop {
    pub fn new(
        inventory_content: &Path,
        inventory: Inventory,
        catalogue: StoreCatalogue,
        bag_capacity: usize,
    ) -> io::Result<Self> {
        Ok(Self {
            inventory_content: inventory_content.to_path_buf(),
            inventory,
            catalogue,
            bag_capacity,
            // (R) The caller must set the verified account flag after construction.
            is_member: false,
            client_transactions: BTreeMap::new(),
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
            8 if inner.len() == 4 => Ok(CoinShopOutcome {
                replies: vec![counted_empty(COIN_FAMILY, 9)],
                ..CoinShopOutcome::default()
            }),
            12 if inner.len() == 12 => {
                self.buy_back_with_context(read_u32(inner, 4)?, read_u32(inner, 8)?)
            }
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
        let granted = match staged.grant_bundle(0, &[(item_id, quantity)]) {
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
        if quantity > owned.count {
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
            quantity,
            buy_back_price: total_price,
            available: true,
        });
        while self.sales.len() > BUY_BACK_WINDOW {
            self.sales.pop_front();
        }

        let mut replies = removed.replies;
        replies.push(coin_balance_packet(staged.state().coins));
        replies.push(transaction_complete(
            2,
            RESULT_SUCCESS,
            transaction_id,
            sold_item.definition_id,
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
            .position(|sale| sale.transaction_id == sale_transaction_id && sale.available)
        else {
            return Ok(buy_back_result(
                sale_transaction_id,
                RESULT_FAILURE,
                client_context,
            ));
        };
        let sale = self.sales[index].clone();
        if self.inventory.state().coins < sale.buy_back_price {
            return Ok(buy_back_result(
                sale_transaction_id,
                RESULT_FAILURE,
                client_context,
            ));
        }
        let mut next_state = self.inventory.state().clone();
        next_state.coins -= sale.buy_back_price;
        let mut staged = Inventory::load(&self.inventory_content, Some(next_state))?;
        let restored = match staged.grant_with_tint(0, sale.item_id, sale.quantity, sale.tint) {
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
        self.sales[index].available = false;
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
        let (result_code, response) = match opcode {
            1 => (MARKETPLACE_RESULT_DISABLED, preview_disabled_packet()),
            3 => (
                MARKETPLACE_RESULT_DISABLED,
                order_disabled_packet(&self.catalogue.marketplace.not_available_message),
            ),
            10 => (1, wallet_packet()),
            12 => (1, marketplace_bool_packet(13, true)),
            14 => (1, empty_product_list(15)),
            // The native currency reader accepts an empty response and logs "no data available".
            16 => (1, header(MARKETPLACE_FAMILY, 17)),
            18 => (1, counted_empty(MARKETPLACE_FAMILY, 19)),
            20 => (1, counted_empty(MARKETPLACE_FAMILY, 21)),
            22 => (1, empty_product_list(23)),
            25 => (1, account_info_packet()),
            // NpcMerchantV2 buy reply: every result other than 1 is a client-visible refusal.
            43 => (MARKETPLACE_RESULT_DISABLED, merchant_buy_disabled_packet()),
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
    quantity: u32,
    unit_price: u32,
    merchant_id: u64,
) -> Vec<u8> {
    // Header and two leading dwords precede the reader's record: kind, u64, u32, u64, block, 3 u32.
    // The zero-length nested block and field meanings are R because no store traffic was captured.
    let mut bytes = header(COIN_FAMILY, 6);
    put_u32(&mut bytes, kind);
    put_u32(&mut bytes, result);
    put_u32(&mut bytes, kind);
    put_u64(&mut bytes, u64::from(transaction_id));
    put_i32(&mut bytes, item_id);
    put_u64(&mut bytes, merchant_id);
    put_i32(&mut bytes, 0);
    put_u32(&mut bytes, quantity);
    put_u32(&mut bytes, unit_price);
    put_u32(&mut bytes, result);
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

fn empty_product_list(opcode: u16) -> Vec<u8> {
    let mut bytes = header(MARKETPLACE_FAMILY, opcode);
    put_u32(&mut bytes, 1);
    put_i32(&mut bytes, 0);
    bytes
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

fn merchant_buy_disabled_packet() -> Vec<u8> {
    let mut bytes = header(MARKETPLACE_FAMILY, 44);
    put_u32(&mut bytes, MARKETPLACE_RESULT_DISABLED);
    bytes
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
