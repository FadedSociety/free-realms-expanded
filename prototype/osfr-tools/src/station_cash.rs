//! Station Cash wallet, Marketplace catalogue and ride orders (T230).
//!
//! Client evidence for every field this module writes (read from the 2014 client with IDA; all
//! addresses are in the shipped `FreeRealms.exe`):
//!
//! * Wallet request 66/10 is the bare header. Wallet response 66/11 is read by `00b3e120` through
//!   `00b3a6e0`/`00b2b450`: a u32 status, then a record of a bool, a u32 balance, a u32
//!   credit-card id, two length-prefixed strings and a bool. The balance (`00b3e120` at
//!   `00b3e1c3`-`00b3e1ca`) goes through `00d5c770` to wallet field +0x1f0 and is passed to the
//!   Lua `OnWalletInfo` event as its third argument, so it is the Station Cash shown to the
//!   player. The record's first u32 after the status is therefore the balance, not a credit-card
//!   id as the community reference names the fields; the client reader wins.
//! * Place-order 66/3 is an order object serialized by `00b3fc60`: two length-prefixed strings, a
//!   u32 order-tracking id, four more strings, a u32 detail count, then per detail a u32
//!   store-bundle id, a u32 store id, a u32 quantity, a tint string and a second string
//!   (`00b3af80`, shared with the preview request). Preview 66/1 is the same order followed by one
//!   extra string.
//! * Place-order response 66/4 is read by `00b3b6d0`/`00b2ded0`: a u32 result (1 is success), a
//!   u32 order-tracking id, a length-prefixed order id string, then two u32s (discount and total).
//! * Preview response 66/2 is read by `00b3b450`/`00b2de20`: four u32s, the first compared with 1.
//! * The catalogue is the 66/5 full-store message: a u32 inner type 1, a u32 store id, then the
//!   store (three u32 ids, two image strings and a keyed bundle map). The bundle record is read by
//!   `00b48700` -> `00b2dfd0`/`00b45340`/`00b43170`/`00b41860`/`00b3da50`/`00b370d0`; the field
//!   order below follows that reader. Its field *names* (price, currency type, limits) are
//!   reconstructed from the community packet catalogue and the client's data-source column table
//!   `00997840` and are labelled (R).
//! * Category tree 66/7 (`00b48040`/`00b43090`), category groups 66/6 (`00b44d70`/`00b3ad20`) and
//!   bundle groups 66/9 (`00b45560`/`00b3d770`/`00b36bb0`) make the store navigable; the store
//!   screen sends 66/12 (server status), so the catalogue is pushed after the 66/13 status reply
//!   instead of at login.
//!
//! Reconstructed policy (R): Station Cash is a per-character balance (the original wallet was an
//! account wallet funded by real money; this revival has no payment path, so the save file is the
//! atomic unit). The balance is never negative, is capped by data, and only an order debits it.
//! An order is validated, priced and granted on clones and commits both the wallet and the
//! inventory together. Replays are keyed by the full order (tracking id, store, bundle, quantity,
//! tint) in a bounded persisted history, so a duplicated packet is paid at most once. The ceiling
//! is the data-configured `order_history`; a packet older than that window is outside this slice's
//! proof. Rides marked `earn_only` are never listed and can never be ordered here.
//!
//! This module grants no Station Cash: slice 2 adds the earned source. Nothing here creates coins.

use crate::inventory::Inventory;
use serde::{Deserialize, Serialize};
use std::io;

/// `PacketBaseInGamePurchase` family.
const MARKETPLACE_FAMILY: u16 = 66;
const OP_PREVIEW: u16 = 1;
const OP_PREVIEW_RESPONSE: u16 = 2;
const OP_ORDER: u16 = 3;
const OP_ORDER_RESPONSE: u16 = 4;
const OP_STORE: u16 = 5;
const OP_CATEGORY_GROUPS: u16 = 6;
const OP_CATEGORIES: u16 = 7;
const OP_BUNDLE_GROUPS: u16 = 9;
const OP_WALLET_RESPONSE: u16 = 11;
const OP_ENABLE_MARKETPLACE: u16 = 24;
const OP_ENABLE_PAYMENT_SOURCES: u16 = 33;

/// Marketplace result 1 is success in the client reader `00b3b6d0`.
pub const ORDER_SUCCESS: u32 = 1;
/// The client maps this to `marketplace_error_order_failure` (R).
pub const ORDER_FAILURE: u32 = 2;
/// The client maps this to `marketplace_error_balance_not_enough` (R).
pub const ORDER_INSUFFICIENT_SC: u32 = 5;

/// A client order is tiny: six small strings, one detail and the preview's extra string.
pub const MAX_ORDER_PACKET: usize = 4096;
const MAX_ORDER_STRING: usize = 256;
const MAX_LOCALE_STRING: usize = 32;
const MAX_DETAIL_STRING: usize = 64;
const MAX_ORDER_DETAILS: usize = 16;
/// Sanity bounds for the data-configured values.
pub const MAX_PRICE_SC: u32 = 10_000_000;
const MAX_WALLET_BALANCE: u32 = i32::MAX as u32;
const MAX_ORDER_HISTORY: usize = 4096;

/// `stores.json.station_cash`: the tunable Marketplace policy. A missing block keeps the feature
/// off, so older content sets and fixtures keep their payment-free behaviour.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub enabled: bool,
    pub store_id: u32,
    pub store_name_id: u32,
    pub store_description_id: u32,
    pub category_id: u32,
    pub category_name_id: u32,
    pub category_group_id: u32,
    pub sc_price_scale: f64,
    pub min_price_sc: u32,
    pub max_price_sc: u32,
    pub max_balance: u32,
    pub order_history: usize,
    pub currency_type: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            store_id: 1,
            store_name_id: 0,
            store_description_id: 0,
            category_id: 1,
            category_name_id: 0,
            category_group_id: 1,
            sc_price_scale: 1.0,
            min_price_sc: 1,
            max_price_sc: 100_000,
            max_balance: 1_000_000,
            order_history: 256,
            currency_type: 0,
        }
    }
}

impl Config {
    pub fn validate(&self) -> io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        if self.store_id == 0
            || self.category_id == 0
            || self.category_group_id == 0
            || !self.sc_price_scale.is_finite()
            || self.sc_price_scale <= 0.0
            || self.sc_price_scale > 10.0
            || self.min_price_sc == 0
            || self.min_price_sc > self.max_price_sc
            || self.max_price_sc > MAX_PRICE_SC
            || self.max_balance == 0
            || self.max_balance > MAX_WALLET_BALANCE
            || self.order_history == 0
            || self.order_history > MAX_ORDER_HISTORY
        {
            return Err(invalid("invalid station_cash policy"));
        }
        Ok(())
    }

    /// `round(sc_price * sc_price_scale)`, refused outside the data bounds.
    fn scaled_price(&self, sc_price: u32) -> io::Result<u32> {
        let raw = f64::from(sc_price) * self.sc_price_scale;
        if !raw.is_finite() || raw < 0.0 || raw > f64::from(u32::MAX) {
            return Err(invalid("station cash price overflow"));
        }
        let rounded = raw.round() as u32;
        if rounded < self.min_price_sc || rounded > self.max_price_sc {
            return Err(invalid("station cash price outside bounds"));
        }
        Ok(rounded)
    }
}

/// A saved wallet must have a Marketplace to spend it in; otherwise the next save would silently
/// drop it. Called while the character loads.
pub fn require_wallet_owner(wallet: &Option<Wallet>, shop_present: bool) -> io::Result<()> {
    if shop_present || wallet.as_ref().is_none_or(Wallet::is_empty) {
        Ok(())
    } else {
        Err(invalid(
            "saved Station Cash wallet requires Marketplace content",
        ))
    }
}

/// One row the Marketplace lists: a store bundle equal to one ride item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Listing {
    pub bundle_id: u32,
    pub item_id: i32,
    pub name_id: u32,
    pub price: u32,
}

/// The validated Station Cash catalogue built from `stores.json` rows and the live item catalogue.
#[derive(Clone, Debug)]
pub struct Catalogue {
    config: Config,
    listings: Vec<Listing>,
    /// Ride rows that must never be listed (slice 2 rewards).
    earn_only: Vec<Listing>,
}

impl Catalogue {
    pub fn build(
        config: Config,
        inventory: &Inventory,
        rows: impl IntoIterator<Item = (i32, u32, bool)>,
    ) -> io::Result<Self> {
        config.validate()?;
        let mut listings = Vec::new();
        let mut earn_only = Vec::new();
        for (item_id, sc_price, reward_only) in rows {
            if item_id <= 0 || sc_price == 0 || sc_price > MAX_PRICE_SC {
                return Err(invalid("invalid station cash row"));
            }
            // A reduced test inventory may not define every shipped ride. Such a row cannot be
            // granted, so it is not listed; the content gate (`tests/ride_sources.rs`) proves the
            // shipped catalogue's definitions instead of making every small fixture load every
            // ride.
            let Ok(presentation) = inventory.presentation(item_id) else {
                continue;
            };
            let listing = Listing {
                bundle_id: u32::try_from(item_id)
                    .map_err(|_| invalid("invalid station cash id"))?,
                item_id,
                name_id: u32::try_from(presentation.name_id).unwrap_or(0),
                price: config.scaled_price(sc_price)?,
            };
            if reward_only {
                earn_only.push(listing);
            } else {
                listings.push(listing);
            }
        }
        listings.sort_by_key(|listing| listing.bundle_id);
        earn_only.sort_by_key(|listing| listing.bundle_id);
        // A reward-only row with the same bundle id as a listed row would alias it.
        if listings.iter().any(|listing| {
            earn_only
                .iter()
                .any(|other| other.bundle_id == listing.bundle_id)
        }) {
            return Err(invalid("station cash bundle id aliases a reward row"));
        }
        Ok(Self {
            config,
            listings,
            earn_only,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn listings(&self) -> &[Listing] {
        &self.listings
    }

    pub fn earn_only(&self) -> &[Listing] {
        &self.earn_only
    }

    pub fn listing(&self, bundle_id: u32) -> Option<&Listing> {
        self.listings
            .iter()
            .find(|listing| listing.bundle_id == bundle_id)
    }
}

/// One successfully paid order, persisted with the wallet so a reconnect cannot pay it twice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderRecord {
    pub tracking_id: u32,
    pub store_id: u32,
    pub bundle_id: u32,
    pub quantity: u32,
    pub tint: i32,
    pub total: u32,
}

/// The per-character Station Cash wallet. `orders` is the replay window; it is part of the save,
/// so a disconnected or replayed order packet cannot be paid again.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wallet {
    #[serde(default)]
    pub balance: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orders: Vec<OrderRecord>,
}

impl Wallet {
    /// Restore a saved wallet under the current data ceiling, dropping corrupt replay rows. A
    /// package with the feature off keeps the saved wallet byte-for-byte: an unrelated content
    /// change must not clamp or prune a currency it does not currently serve.
    pub fn restore(saved: Option<Self>, config: &Config) -> Self {
        let mut wallet = saved.unwrap_or_default();
        if !config.enabled {
            return wallet;
        }
        wallet.balance = wallet.balance.min(config.max_balance);
        wallet.orders.retain(|order| {
            order.store_id > 0 && order.bundle_id > 0 && order.quantity > 0 && order.total > 0
        });
        if wallet.orders.len() > config.order_history {
            wallet
                .orders
                .drain(..wallet.orders.len() - config.order_history);
        }
        wallet
    }

    pub fn is_empty(&self) -> bool {
        self.balance == 0 && self.orders.is_empty()
    }

    fn replay(&self, order: &OrderDetail, tracking_id: u32) -> Option<u32> {
        self.orders
            .iter()
            .rev()
            .find(|record| {
                record.tracking_id == tracking_id
                    && record.store_id == order.store_id
                    && record.bundle_id == order.bundle_id
                    && record.quantity == order.quantity
                    && record.tint == order.tint
            })
            .map(|record| record.total)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderDetail {
    pub bundle_id: u32,
    pub store_id: u32,
    pub quantity: u32,
    pub tint: i32,
}

/// A decoded 66/1 or 66/3 request. Strings that can steer a grant are reduced to decisions here;
/// the rest are length-validated and discarded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderRequest {
    pub tracking_id: u32,
    pub details: Vec<OrderDetail>,
    pub coupon: bool,
    /// The recipient string names someone other than the customer (a gift order).
    pub gift: bool,
}

pub struct OrderOutcome {
    pub result_code: u32,
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
}

/// Read one Marketplace order. `preview` expects the extra trailing string of 66/1.
pub fn decode_order(inner: &[u8], preview: bool) -> io::Result<OrderRequest> {
    if inner.len() < 4 || inner.len() > MAX_ORDER_PACKET {
        return Err(invalid("invalid Marketplace order length"));
    }
    let opcode = u16::from_le_bytes([inner[2], inner[3]]);
    if inner[..2] != MARKETPLACE_FAMILY.to_le_bytes()
        || opcode != if preview { OP_PREVIEW } else { OP_ORDER }
    {
        return Err(invalid("not a Marketplace order"));
    }
    let mut reader = Reader {
        bytes: &inner[4..],
        at: 0,
    };
    let customer = reader.string(MAX_ORDER_STRING)?;
    let recipient = reader.string(MAX_ORDER_STRING)?;
    let tracking_id = reader.u32()?;
    reader.string(MAX_LOCALE_STRING)?;
    let coupon = reader.string(MAX_ORDER_STRING)?;
    reader.string(MAX_ORDER_STRING)?;
    reader.string(MAX_ORDER_STRING)?;
    let count = reader.u32()?;
    if count as usize > MAX_ORDER_DETAILS {
        return Err(invalid("too many Marketplace order details"));
    }
    let mut details = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let bundle_id = reader.u32()?;
        let store_id = reader.u32()?;
        let quantity = reader.u32()?;
        let tint = reader.string(MAX_DETAIL_STRING)?;
        reader.string(MAX_ORDER_STRING)?;
        let tint = if tint.is_empty() {
            0
        } else {
            let text =
                std::str::from_utf8(tint).map_err(|_| invalid("invalid Marketplace order tint"))?;
            // (R) -1 is the client's "use the item's default tint"; grant_with_tint takes >= 0.
            text.parse::<i32>()
                .map_err(|_| invalid("invalid Marketplace order tint"))?
                .max(0)
        };
        details.push(OrderDetail {
            bundle_id,
            store_id,
            quantity,
            tint,
        });
    }
    if preview && !reader.finished() {
        // The recovered preview template carries one extra string; tolerate its absence so a
        // client that sends the bare order still previews. Anything after that string is invalid.
        reader.string(MAX_ORDER_STRING)?;
    }
    reader.finish()?;
    Ok(OrderRequest {
        tracking_id,
        details,
        coupon: !coupon.is_empty(),
        gift: !recipient.is_empty() && recipient != customer,
    })
}

/// Price and validate an order without changing anything; 66/2 reply.
pub fn preview(catalogue: &Catalogue, wallet: &Wallet, order: &OrderRequest) -> OrderOutcome {
    let (result, total) = match price(catalogue, wallet, order) {
        Ok((_, _, total)) => (ORDER_SUCCESS, total),
        Err(result) => (result, 0),
    };
    OrderOutcome {
        result_code: result,
        replies: vec![preview_packet(result, order.tracking_id, total)],
        changed: false,
    }
}

/// Debit Station Cash and grant the ride in one staged commit; 66/4 reply.
pub fn place_order(
    catalogue: &Catalogue,
    wallet: &mut Wallet,
    inventory: &mut Inventory,
    order: &OrderRequest,
    bag_capacity: usize,
    busy: bool,
) -> OrderOutcome {
    if busy {
        return refusal(order, ORDER_FAILURE);
    }
    let (listing, detail, total) = match price(catalogue, wallet, order) {
        Ok(priced) => priced,
        Err(result) => return refusal(order, result),
    };
    // A replay of a fully paid order returns its result without touching state.
    if let Some(old_total) = wallet.replay(&detail, order.tracking_id) {
        return OrderOutcome {
            result_code: ORDER_SUCCESS,
            replies: vec![
                wallet_packet(wallet.balance),
                order_packet(ORDER_SUCCESS, order.tracking_id, old_total),
            ],
            changed: false,
        };
    }
    if wallet.balance < total {
        return refusal(order, ORDER_INSUFFICIENT_SC);
    }
    let mut staged = inventory.clone();
    let granted = match staged.grant_with_tint(0, listing.item_id, 1, detail.tint) {
        Ok(granted) => granted,
        Err(_) => return refusal(order, ORDER_FAILURE),
    };
    if staged.state().items.len() > bag_capacity {
        return refusal(order, ORDER_FAILURE);
    }
    let balance = wallet.balance - total;
    *inventory = staged;
    wallet.balance = balance;
    wallet.orders.push(OrderRecord {
        tracking_id: order.tracking_id,
        store_id: detail.store_id,
        bundle_id: detail.bundle_id,
        quantity: detail.quantity,
        tint: detail.tint,
        total,
    });
    if wallet.orders.len() > catalogue.config.order_history {
        let drop = wallet.orders.len() - catalogue.config.order_history;
        wallet.orders.drain(..drop);
    }
    let mut replies = vec![wallet_packet(balance)];
    replies.extend(granted.replies);
    replies.push(order_packet(ORDER_SUCCESS, order.tracking_id, total));
    OrderOutcome {
        result_code: ORDER_SUCCESS,
        replies,
        changed: true,
    }
}

fn price<'a>(
    catalogue: &'a Catalogue,
    wallet: &Wallet,
    order: &OrderRequest,
) -> Result<(&'a Listing, OrderDetail, u32), u32> {
    if order.gift || order.coupon || order.details.len() != 1 {
        return Err(ORDER_FAILURE);
    }
    let detail = order.details[0];
    if detail.quantity != 1 || detail.store_id != catalogue.config.store_id {
        return Err(ORDER_FAILURE);
    }
    let listing = catalogue.listing(detail.bundle_id).ok_or(ORDER_FAILURE)?;
    let total = listing.price;
    if wallet.balance < total && wallet.replay(&detail, order.tracking_id).is_none() {
        return Err(ORDER_INSUFFICIENT_SC);
    }
    Ok((listing, detail, total))
}

fn refusal(order: &OrderRequest, result: u32) -> OrderOutcome {
    OrderOutcome {
        result_code: result,
        replies: vec![order_packet(result, order.tracking_id, 0)],
        changed: false,
    }
}

/// 66/11. The client reader `00b3a6e0`/`00b2b450` consumes exactly this record.
pub fn wallet_packet(balance: u32) -> Vec<u8> {
    let mut bytes = header(OP_WALLET_RESPONSE);
    put_u32(&mut bytes, 1);
    bytes.push(0);
    put_u32(&mut bytes, balance);
    put_u32(&mut bytes, 0);
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.push(0);
    bytes
}

/// 66/2. Reader `00b3b450` treats the first dword 1 as success.
fn preview_packet(result: u32, tracking_id: u32, total: u32) -> Vec<u8> {
    let mut bytes = header(OP_PREVIEW_RESPONSE);
    put_u32(&mut bytes, result);
    put_u32(&mut bytes, tracking_id);
    put_u32(&mut bytes, 0);
    put_u32(&mut bytes, total);
    bytes
}

/// 66/4. Reader `00b3b6d0` treats the first dword 1 as success and `00b2ded0` reads the rest.
fn order_packet(result: u32, tracking_id: u32, total: u32) -> Vec<u8> {
    let mut bytes = header(OP_ORDER_RESPONSE);
    put_u32(&mut bytes, result);
    put_u32(&mut bytes, tracking_id);
    put_string(&mut bytes, &tracking_id.to_string());
    put_u32(&mut bytes, 0);
    put_u32(&mut bytes, total);
    bytes
}

/// The catalogue packets the store screen needs, in the order it receives them.
pub fn catalogue_packets(catalogue: &Catalogue) -> Vec<Vec<u8>> {
    vec![
        bool_packet(OP_ENABLE_MARKETPLACE, true),
        payment_sources_packet(),
        category_tree_packet(catalogue),
        category_groups_packet(catalogue),
        bundle_groups_packet(catalogue),
        store_packet(catalogue),
    ]
}

fn payment_sources_packet() -> Vec<u8> {
    let mut bytes = header(OP_ENABLE_PAYMENT_SOURCES);
    bytes.extend_from_slice(&[0, 0]);
    bytes
}

pub(crate) fn bool_packet(opcode: u16, value: bool) -> Vec<u8> {
    let mut bytes = header(opcode);
    bytes.push(u8::from(value));
    bytes
}

/// 66/7 + category tree (`00b48040`/`00b43090`/`00b3fd70`/`00b3b080`).
fn category_tree_packet(catalogue: &Catalogue) -> Vec<u8> {
    let config = &catalogue.config;
    let mut bytes = header(OP_CATEGORIES);
    put_i32(&mut bytes, 1); // one category
    put_u32(&mut bytes, config.category_id);
    put_u32(&mut bytes, config.category_id);
    put_u32(&mut bytes, config.category_name_id);
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    put_u32(&mut bytes, 0);
    put_u32(&mut bytes, catalogue.listings.len() as u32);
    put_u32(&mut bytes, 1);
    bytes.push(1);
    bytes
}

/// 66/6 + one category group holding the category (`00b44d70`/`00b3ad20`).
fn category_groups_packet(catalogue: &Catalogue) -> Vec<u8> {
    let config = &catalogue.config;
    let mut bytes = header(OP_CATEGORY_GROUPS);
    put_i32(&mut bytes, 1); // one group
    put_u32(&mut bytes, config.category_group_id);
    put_u32(&mut bytes, config.category_group_id);
    put_i32(&mut bytes, 1);
    put_u32(&mut bytes, config.category_id);
    bytes
}

/// 66/9 + one bundle group (`00b45560`/`00b3d770`/`00b36bb0`/`00b33350`/`00b2a090`).
fn bundle_groups_packet(catalogue: &Catalogue) -> Vec<u8> {
    let config = &catalogue.config;
    let mut bytes = header(OP_BUNDLE_GROUPS);
    put_i32(&mut bytes, 1); // one bundle group
    put_u32(&mut bytes, config.category_group_id);
    put_u32(&mut bytes, config.category_group_id);
    put_u32(&mut bytes, config.category_name_id);
    put_u32(&mut bytes, 0);
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.push(0);
    put_u32(&mut bytes, 1);
    put_i32(&mut bytes, catalogue.listings.len() as i32);
    for (index, listing) in catalogue.listings.iter().enumerate() {
        // Keyed map entry: map key, the entry's own bundle id, then its display order
        // (client reader `00b33350` -> `00b2a090` reads both value words).
        put_u32(&mut bytes, listing.bundle_id);
        put_u32(&mut bytes, listing.bundle_id);
        put_u32(&mut bytes, index as u32 + 1);
    }
    bytes
}

/// 66/5 full store (`00b48700` -> `00b45340`/`00b43170`/`00b41860`).
fn store_packet(catalogue: &Catalogue) -> Vec<u8> {
    let config = &catalogue.config;
    let mut bytes = header(OP_STORE);
    put_u32(&mut bytes, 1);
    put_u32(&mut bytes, config.store_id);
    put_u32(&mut bytes, config.store_id);
    put_u32(&mut bytes, config.store_name_id);
    put_u32(&mut bytes, config.store_description_id);
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    put_i32(&mut bytes, catalogue.listings.len() as i32);
    for (index, listing) in catalogue.listings.iter().enumerate() {
        put_u32(&mut bytes, listing.bundle_id);
        bundle_record(&mut bytes, catalogue, listing, index);
    }
    bytes
}

/// One bundle record in the reader's order. Names are (R); the positions are the client's.
fn bundle_record(bytes: &mut Vec<u8>, catalogue: &Catalogue, listing: &Listing, index: usize) {
    let config = &catalogue.config;
    put_u32(bytes, listing.bundle_id);
    put_u32(bytes, listing.name_id);
    put_u32(bytes, 0);
    put_u32(bytes, 1);
    put_string(bytes, "");
    put_string(bytes, "");
    bytes.push(0);
    put_string(bytes, "");
    put_u32(bytes, config.currency_type);
    put_u32(bytes, listing.price);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    put_u64(bytes, 0);
    put_u64(bytes, 0);
    put_u32(bytes, 0);
    bytes.push(0);
    put_u64(bytes, 0);
    put_u64(bytes, 0);
    put_i32(bytes, 1);
    put_u32(bytes, 1);
    put_u32(bytes, listing.item_id as u32);
    put_u32(bytes, listing.item_id as u32);
    put_string(bytes, "");
    put_u32(bytes, config.store_id);
    put_u32(bytes, config.category_group_id);
    bytes.push(1);
    put_u32(bytes, 1);
    put_u32(bytes, index as u32 + 1);
    put_u32(bytes, index as u32 + 1);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    bytes.push(0);
    bytes.push(0);
    bytes.push(0);
}

fn header(opcode: u16) -> Vec<u8> {
    let mut bytes = Vec::new();
    put_u16(&mut bytes, MARKETPLACE_FAMILY);
    put_u16(&mut bytes, opcode);
    bytes
}

fn put_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    put_i32(bytes, value.len() as i32);
    bytes.extend_from_slice(value.as_bytes());
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn u32(&mut self) -> io::Result<u32> {
        let end = self
            .at
            .checked_add(4)
            .ok_or_else(|| invalid("order overflow"))?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| invalid("truncated Marketplace order"))?;
        self.at = end;
        Ok(u32::from_le_bytes(
            bytes.try_into().map_err(|_| invalid("truncated order"))?,
        ))
    }

    fn string(&mut self, max: usize) -> io::Result<&'a [u8]> {
        let length = self.u32()? as usize;
        if length > max {
            return Err(invalid("oversized Marketplace order string"));
        }
        let end = self
            .at
            .checked_add(length)
            .ok_or_else(|| invalid("order overflow"))?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| invalid("truncated Marketplace order string"))?;
        self.at = end;
        Ok(bytes)
    }

    fn finished(&self) -> bool {
        self.at == self.bytes.len()
    }

    fn finish(&self) -> io::Result<()> {
        if self.finished() {
            Ok(())
        } else {
            Err(invalid("trailing Marketplace order bytes"))
        }
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
