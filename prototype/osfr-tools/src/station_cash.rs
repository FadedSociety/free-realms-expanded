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
//! * Place-order response 66/4 is read by `00b3b6d0`/`00b2ded0` (T453): a u32 order-tracking id
//!   (obj+0x18), a u32 result (obj+0x1C, 1 is success), a length-prefixed order id string, then
//!   two u32s (discount and total). The order completes only when the tracking id equals the
//!   pending order's (`00b3b6d0`, manager+0x3E8E8 -> +0x44), so every reply echoes it.
//! * Preview response 66/2 is read by `00b3b450`/`00b2de20`: four u32s, the tracking id (obj+0x18,
//!   unused by the handler), the result (obj+0x1C, 1 is success), discount and total.
//! * The catalogue is the 66/5 full-store message: a u32 inner type 1, a u32 store id, then the
//!   store (three u32 ids, two image strings and a keyed bundle map). The bundle record is read by
//!   `00b48700` -> `00b2dfd0`/`00b45340`/`00b43170`/`00b41860`/`00b3da50`/`00b370d0`; the field
//!   order below follows that reader. `00b43170` reads one map key then `00b41860` per entry;
//!   `00b41860` calls `00b3fa00` and then reads ten more fields (u32, u32, u8, u32, u32, u32, u8,
//!   u8, u32, u32), which are written as zeros (R). An earlier T230 writer stopped after the
//!   `00b3fa00` tail and was 31 bytes short per bundle. The field *names* (price, currency type,
//!   limits) are reconstructed from the community packet catalogue and the client's data-source
//!   column table `00997840` and are labelled (R).
//! * Category tree 66/7 (`00b48040`/`00b43090`), category groups 66/6 (`00b44d70`/`00b3ad20`) and
//!   bundle groups 66/9 (`00b45560`/`00b3d770`/`00b36bb0`) make the store navigable; the store
//!   screen sends 66/12 (server status), so the catalogue is pushed after the 66/13 status reply
//!   instead of at login.
//! * T526: a 66/7 node's parent id is record dword 3, the first `00b3b080` field; `00b43090`
//!   re-keys every node by it, `009a67c0`/`009a5eb0` build the tree from it and `0099d7e0` walks
//!   it, so a selected category lists its own bundles and its descendants'. The four ride
//!   sub-categories therefore name the Rides category as their parent.
//! * T657: the store-bundle data source `009a56d0` names each tile field: case 4 is the description
//!   id (bundle +164, the third dword `00b3da50` reads), case 5 the price (+176), cases 6-7 the
//!   image set and tint (`00b2ea50` stores the two strings at the image object +0x18 and the
//!   getters `atoi` them), case 9 IsOnSale (`00997300`: +243 != 0 or +228 != 0), case 10 SalePrice
//!   (+272), case 14/15 terms (+208) and case 2 the category group (+204). +243 is not a
//!   visibility byte; `0099f150` reads +243 as the sale filter and +264 as the hidden flag. The
//!   old writer sent 1 at +243, so every tile was on sale with SalePrice 0, and sent 0 at +164,
//!   so every tile showed `<STRING 0 NOT FOUND>`.
//! * T657: the client recomputes a 66/7 node's count from the bundles and adds one to every parent
//!   (`00c37e70`); the server writes that same descendant total. `0099f150` filters a bundle by its
//!   +204 group's 66/6 category, and `0099d7e0` accepts the selected category or any descendant.
//! * T744: `009a56d0` case 32 reads the members-only price from bundle +268; the client's
//!   `getMOItemPrice` uses it for a member (`MarketplaceDataSource.as`), so 0 is a 100% discount,
//!   not "no discount". A row without a member price writes its regular price there. The tile also
//!   stays on its loading placeholder while the bundle's item definitions are missing: the client
//!   asks for them with 66/27 (serializer `00b41a30`/`00b3dd80`; `00b47b20`/`00b48120`), and the
//!   66/28 reply (`00b40050`, reader `00b2e2d0`) clears the pending bundle only after the item
//!   definitions (35/37) arrived, so the reply sends the 35/37 batch first.
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
//! L046 (R): a listed lot deed (item Type 16) is paid like a ride but delivers the lot itself
//! through the caller's [`LotGrant`]; the deed item is never granted.
//!
//! T486: a paid order reports the stacks it granted (`OrderOutcome::granted`); the gameplay layer
//! turns a bought pet bundle into a pet through the same settlement the coin store uses.
//!
//! Slice 2 adds the earned source: `sc_rewards.json` names one verified completion's payout (kart
//! race, derby match, soccer match, chess game, quest turn-in, dungeon clear and the first
//! completed activity of the UTC day). `EarnTable` only decides an amount; the completion hook
//! spends it through the single reward applier (`rewards::apply_grant`, Station Cash as a reward
//! kind). Nothing here creates coins and no client value reaches an amount.

use crate::content::SourceTag;
use crate::inventory::{Inventory, InventoryState};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

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
const OP_ITEM_REQUIREMENTS: u16 = 39;
const OP_WALLET_RESPONSE: u16 = 11;
const OP_ENABLE_MARKETPLACE: u16 = 24;
const OP_CONTENT_REQUEST: u16 = 27;
const OP_CONTENT_RESPONSE: u16 = 28;
const OP_ENABLE_PAYMENT_SOURCES: u16 = 33;

/// Marketplace result 1 is success in the client reader `00b3b6d0`.
pub const ORDER_SUCCESS: u32 = 1;
/// `00cc5470` maps this to `marketplace_error_order_failure` (using it for every refusal is R).
pub const ORDER_FAILURE: u32 = 2;
/// `00cc5470` maps this to `marketplace_error_balance_not_enough`.
pub const ORDER_INSUFFICIENT_SC: u32 = 5;

/// A client order is tiny: six small strings, one detail and the preview's extra string.
pub const MAX_ORDER_PACKET: usize = 4096;
const MAX_ORDER_STRING: usize = 256;
const MAX_LOCALE_STRING: usize = 32;
const MAX_DETAIL_STRING: usize = 64;
const MAX_ORDER_DETAILS: usize = 16;
/// T744: the client streams two bundles per 66/27 request (`00b48120`); this bounds the entry
/// vector a hostile body can ask for.
const MAX_CONTENT_REQUESTS: usize = 8;
/// Sanity bounds for the data-configured values.
pub const MAX_PRICE_SC: u32 = 10_000_000;
const MAX_WALLET_BALANCE: u32 = i32::MAX as u32;
const MAX_ORDER_HISTORY: usize = 4096;
/// (R) Bounds for the T657 category table; the values themselves are content.
const MAX_ITEM_SHELVES: usize = 64;
const MAX_ITEM_SHELF_KEYS: usize = 256;

/// (R) Sanity bounds for `sc_rewards.json`; every amount is content, these only bound it.
pub const MAX_EARN_SC: u32 = 10_000;
const MAX_EARN_OVERRIDES: usize = 512;
const MAX_EARN_RIDES: usize = 64;
/// (R) Sanity bound for `chess_min_plies` (not a game-length limit).
const MAX_CHESS_MIN_PLIES: u32 = 400;
const MAX_EARN_FILE_BYTES: u64 = 65_536;
/// (R) The Station Cash "day" is the UTC day, like the chess ledger.
const ACTIVITY_DAY_SECONDS: u64 = 86_400;

/// One Marketplace category: its 66/7 node, the 66/6 group that holds it and the group key a
/// bundle's record names (`0099f150` matches a bundle to a category through that group).
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shelf {
    pub category_id: u32,
    pub category_name_id: u32,
    pub category_group_id: u32,
}

/// T526: the four final-era Marketplace ride sub-categories. The key is the `stores.json` row tag;
/// the client locale names are 5101131 Winged, 5101134 Non-Winged, 5101132 Exotic and 5101133
/// Mechanical, the ids Legacy `StoreBundleCategories.json` gives categories 266, 269, 268 and 267
/// under parent 139. The ids below are ours (R).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RideCategory {
    Winged,
    NonWinged,
    Exotic,
    Mechanical,
}

/// T526: one ride sub-category tree node. `category_id` doubles as the 66/6 group key of the
/// bundles it holds, the way Legacy gives groups 266-269 the category's own id.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideCategoryShelf {
    pub key: RideCategory,
    pub category_id: u32,
    pub category_name_id: u32,
    pub display_order: u32,
}

/// T657: a top-level Marketplace category for rows that are neither rides nor boomboxes.
/// `category_id` doubles as the 66/6 group key of the bundles it holds (the convention T526 uses
/// for the ride children). A row lists here when its `stores.json` item category is in
/// `item_category_ids`, or else when its item definition type is in `item_types`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemShelf {
    pub category_id: u32,
    pub category_name_id: u32,
    pub display_order: u32,
    #[serde(default)]
    pub item_category_ids: Vec<u32>,
    #[serde(default)]
    pub item_types: Vec<i32>,
}

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
    /// T495: the Boombox category beside Rides. Rows tagged `boombox` list there; none may be
    /// tagged while this is absent.
    pub boombox_shelf: Option<Shelf>,
    /// T526: the ride sub-categories under Rides. Empty keeps the pre-T526 single Rides category;
    /// non-empty must name all four, so every `ride_category` row has its child.
    pub ride_categories: Vec<RideCategoryShelf>,
    /// T657: the top-level categories for every other listing. A listed non-ride row that no shelf
    /// names is refused at load, so no non-ride row falls back to the Rides parent.
    pub item_shelves: Vec<ItemShelf>,
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
            boombox_shelf: None,
            ride_categories: Vec::new(),
            item_shelves: Vec::new(),
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
            || self.boombox_shelf.is_some_and(|shelf| {
                shelf.category_id == 0
                    || shelf.category_group_id == 0
                    || shelf.category_id == self.category_id
                    || shelf.category_group_id == self.category_group_id
            })
            || !self.ride_categories_valid()
            || !self.item_shelves_valid()
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

    /// A category id or group key owned by the Rides parent or the Boombox shelf. The ride
    /// children and the item shelves are checked against it and each other.
    fn base_reserved(&self, id: u32) -> bool {
        let boombox = self.boombox_shelf;
        id == self.category_id
            || id == self.category_group_id
            || boombox.is_some_and(|shelf| id == shelf.category_id || id == shelf.category_group_id)
    }

    /// T526: no sub-categories (the pre-T526 single Rides category), or exactly the four, each
    /// with its own id, order and name. A child id is also its 66/6 group key, so it may not
    /// collide with another shelf's.
    fn ride_categories_valid(&self) -> bool {
        if self.ride_categories.is_empty() {
            return true;
        }
        let mut keys = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut orders = BTreeSet::new();
        for shelf in &self.ride_categories {
            if !keys.insert(shelf.key)
                || !ids.insert(shelf.category_id)
                || !orders.insert(shelf.display_order)
                || shelf.category_id == 0
                || shelf.category_name_id == 0
                || shelf.display_order == 0
                || self.base_reserved(shelf.category_id)
                || self
                    .item_shelves
                    .iter()
                    .any(|item| item.category_id == shelf.category_id)
            {
                return false;
            }
        }
        self.ride_categories.len() == 4
    }

    /// T657: each item shelf needs its own id, order and name; every item category and item type
    /// names exactly one shelf, and no shelf reuses a ride/boombox/Rides id or group key.
    fn item_shelves_valid(&self) -> bool {
        if self.item_shelves.len() > MAX_ITEM_SHELVES {
            return false;
        }
        let mut ids = BTreeSet::new();
        // 1 and 2 are the Rides and Boombox top-level orders (`tree_nodes`).
        let mut orders = BTreeSet::from([1, 2]);
        let mut categories = BTreeSet::new();
        let mut types = BTreeSet::new();
        for shelf in &self.item_shelves {
            if !ids.insert(shelf.category_id)
                || !orders.insert(shelf.display_order)
                || shelf.category_id == 0
                || shelf.category_name_id == 0
                || shelf.display_order == 0
                || (shelf.item_category_ids.is_empty() && shelf.item_types.is_empty())
                || shelf.item_category_ids.len() + shelf.item_types.len() > MAX_ITEM_SHELF_KEYS
                || self.base_reserved(shelf.category_id)
                || self
                    .ride_categories
                    .iter()
                    .any(|ride| ride.category_id == shelf.category_id)
            {
                return false;
            }
            for category in &shelf.item_category_ids {
                if *category == 0 || !categories.insert(*category) {
                    return false;
                }
            }
            for item_type in &shelf.item_types {
                if *item_type <= 0 || !types.insert(*item_type) {
                    return false;
                }
            }
        }
        true
    }

    /// T526: the 66/6 group a ride row lists under: its sub-category, or the Rides parent.
    fn ride_group(&self, category: Option<RideCategory>) -> io::Result<u32> {
        let Some(category) = category else {
            return Ok(self.category_group_id);
        };
        self.ride_categories
            .iter()
            .find(|shelf| shelf.key == category)
            .map(|shelf| shelf.category_id)
            .ok_or_else(|| invalid("ride category is not configured"))
    }

    /// T657: the 66/6 group one row lists under. Boomboxes and tagged rides keep their T495/T526
    /// shelves; an untagged ride keeps the Rides parent so older content still lists it (checked
    /// before the item shelves, so a ride's own item category cannot move it); every other row
    /// takes the item shelf its category (then its item type) names, and one without a shelf is a
    /// content error, never a Rides fallback (the T657 bug).
    fn listing_group(
        &self,
        kind: RowKind,
        ride_category: Option<RideCategory>,
        category_id: Option<u32>,
        item_type: i32,
        ride: bool,
    ) -> io::Result<u32> {
        // A missing/off block promises a payment-free older content set; no catalogue is sent, so
        // the category table is not required there. Enabling Station Cash requires it again.
        if !self.enabled {
            return Ok(self.category_group_id);
        }
        match kind {
            // Never sent; only its price is read.
            RowKind::EarnOnly => Ok(self.category_group_id),
            RowKind::Boombox => self
                .boombox_shelf
                .map(|shelf| shelf.category_group_id)
                .ok_or_else(|| invalid("boombox row without a shelf")),
            RowKind::Ride => {
                if ride_category.is_some() {
                    return self.ride_group(ride_category);
                }
                // A ride keeps the Rides parent even when its row carries a shelved item category.
                if ride || item_type == RIDE_ITEM_TYPE {
                    return Ok(self.category_group_id);
                }
                if let Some(shelf) = self.item_shelf(category_id, item_type) {
                    return Ok(shelf.category_id);
                }
                Err(invalid("station cash row has no Marketplace category"))
            }
        }
    }

    /// The first item shelf naming this item category, or else this item definition type.
    fn item_shelf(&self, category_id: Option<u32>, item_type: i32) -> Option<&ItemShelf> {
        let by_category = category_id.filter(|id| *id > 0).and_then(|id| {
            self.item_shelves
                .iter()
                .find(|shelf| shelf.item_category_ids.contains(&id))
        });
        by_category.or_else(|| {
            self.item_shelves
                .iter()
                .find(|shelf| shelf.item_types.contains(&item_type))
        })
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

/// One server-verified completion that can pay Station Cash (R). The completion itself is never
/// decoded from a client message: each variant is raised by the owning activity's own accepted
/// completion record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EarnSource {
    KartRace,
    DerbyMatch,
    SoccerMatch,
    /// A settled chess game. It pays only if it reached a result through play (R): a win or a
    /// draw, or a loss after `EarnTable::chess_min_plies` plies, so an instant resign pays nothing.
    ChessGame {
        won_or_drew: bool,
        plies: u32,
    },
    Quest {
        id: u32,
    },
    DungeonClear {
        id: u32,
    },
}

/// A first-clear ride resolved against the live inventory and the Marketplace price.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RideOffer {
    pub item_id: i32,
    /// The Marketplace Station Cash value paid when the ride is already owned.
    pub sc_equivalent: u32,
    pub owned: bool,
    pub room: bool,
}

/// What one completion earns after the daily bonus and the first-clear ride decision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Earned {
    pub sc: u32,
    /// The first-clear ride item to grant, when this completion wins it.
    pub ride: Option<i32>,
    /// The first-clear ride is still unearned but the bag has no room; the caller tells the
    /// player to make room, and a later clear grants it.
    pub bag_full: bool,
}

impl Earned {
    /// The one atomic reward for this completion: the Station Cash kind plus the first-clear ride
    /// item, for the single reward applier (`rewards::apply_grant`). The display stays empty; the
    /// applier sends no reward bundle for it (client reader `008e7930` has no SC field).
    pub fn grant(self) -> crate::rewards::RewardGrant {
        crate::rewards::RewardGrant {
            coins: 0,
            items: self
                .ride
                .map(|definition_id| {
                    vec![crate::rewards::ItemGrant {
                        definition_id,
                        count: 1,
                    }]
                })
                .unwrap_or_default(),
            job_xp: Default::default(),
            sc: self.sc,
            display: crate::rewards::RewardDisplay {
                source_guid: 0,
                player_guid: 0,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        }
    }
}

/// The first clear of one dungeon grants one earn-only ride (R). It is granted once per character
/// save; an owned ride pays its Marketplace value instead, and a full bag defers it to a later
/// clear.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirstClearRide {
    pub dungeon_id: u32,
    pub item_id: i32,
}

/// A data-driven event ride. No seasonal event content exists yet (T418's live events only gate
/// quest offers), so a row must stay dormant: the loader refuses `enabled: true` rather than
/// carrying an unreachable trigger.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeasonalRide {
    pub item_id: i32,
    pub event: String,
    #[serde(default)]
    pub enabled: bool,
}

/// `data/content/sc_rewards.json`: the earn table. Missing keeps earning off, so a content set
/// without the file pays nothing (like the Marketplace policy). Every amount is (R) and tunable
/// without a rebuild; `MAX_EARN_SC` bounds a malicious or mistaken edit.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EarnTable {
    pub schema_version: u32,
    /// The whole table is off when false; a stored wallet is kept.
    pub enabled: bool,
    /// One completed race, match or game (the server-observed finish, win or loss).
    pub kart_race: u32,
    pub derby_match: u32,
    pub soccer_match: u32,
    pub chess_game: u32,
    /// (R) A lost chess game pays only after this many plies (half-moves); wins and draws always
    /// pay. Live-tunable; 0 pays every settled game. Sane bound 0..=400.
    #[serde(default = "default_chess_min_plies")]
    pub chess_min_plies: u32,
    /// One won combat run (the opening cave or any field battle), paid at the win transition;
    /// `hardest_dungeon_clear` replaces it for `hardest_dungeons`.
    pub dungeon_clear: u32,
    pub hardest_dungeon_clear: u32,
    /// Dungeon ids that pay `hardest_dungeon_clear` unless `dungeon_overrides` names them.
    pub hardest_dungeons: Vec<u32>,
    /// Per-dungeon amounts; a key here outranks the hardest/default tier.
    pub dungeon_overrides: BTreeMap<u32, u32>,
    /// One accepted NPC quest turn-in; `quest_overrides` replaces it per quest.
    pub quest_complete: u32,
    /// Per-quest amounts keyed by the quest's numeric id.
    pub quest_overrides: BTreeMap<u32, u32>,
    /// Added once per character per UTC day on the first qualifying completion.
    pub daily_first_activity: u32,
    pub first_clear_ride: Option<FirstClearRide>,
    /// Dormant event rides; every row must be disabled until a seasonal event trigger exists.
    pub seasonal_rides: Vec<SeasonalRide>,
    pub source: Vec<SourceTag>,
}

fn default_chess_min_plies() -> u32 {
    20
}

impl EarnTable {
    pub fn load(path: &Path) -> io::Result<Option<Self>> {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        if !metadata.is_file() || metadata.len() > MAX_EARN_FILE_BYTES {
            return Err(invalid("invalid Station Cash reward content"));
        }
        let bytes = fs::read(path)?;
        if bytes.len() as u64 > MAX_EARN_FILE_BYTES {
            return Err(invalid("invalid Station Cash reward content"));
        }
        let table: Self = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("invalid Station Cash reward content"))?;
        table.validate()?;
        Ok(Some(table))
    }

    fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1 {
            return Err(invalid("unsupported Station Cash reward schema"));
        }
        let amounts = [
            self.kart_race,
            self.derby_match,
            self.soccer_match,
            self.chess_game,
            self.dungeon_clear,
            self.hardest_dungeon_clear,
            self.quest_complete,
            self.daily_first_activity,
        ];
        if amounts.iter().any(|amount| *amount > MAX_EARN_SC)
            || self.hardest_dungeons.len() > MAX_EARN_OVERRIDES
            || self.dungeon_overrides.len() > MAX_EARN_OVERRIDES
            || self.quest_overrides.len() > MAX_EARN_OVERRIDES
            || self.seasonal_rides.len() > MAX_EARN_RIDES
            || self.chess_min_plies > MAX_CHESS_MIN_PLIES
        {
            return Err(invalid("Station Cash reward value outside bounds"));
        }
        let dungeons: std::collections::BTreeSet<u32> =
            self.hardest_dungeons.iter().copied().collect();
        if dungeons.len() != self.hardest_dungeons.len()
            || dungeons.contains(&0)
            || self
                .dungeon_overrides
                .iter()
                .any(|(id, sc)| *id == 0 || *sc > MAX_EARN_SC)
            || self
                .quest_overrides
                .iter()
                .any(|(id, sc)| *id == 0 || *sc > MAX_EARN_SC)
        {
            return Err(invalid("Station Cash reward key outside bounds"));
        }
        if let Some(ride) = self.first_clear_ride {
            if ride.item_id <= 0 || ride.dungeon_id == 0 || !dungeons.contains(&ride.dungeon_id) {
                return Err(invalid("invalid Station Cash first-clear ride"));
            }
        }
        let mut rides = std::collections::BTreeSet::new();
        for ride in &self.seasonal_rides {
            if ride.item_id <= 0
                || ride.event.trim().is_empty()
                || ride.event.len() > 64
                || ride.enabled
                || !rides.insert(ride.item_id)
            {
                return Err(invalid("invalid or active Station Cash seasonal ride"));
            }
        }
        if self.source.is_empty() {
            return Err(invalid("Station Cash reward content has no provenance"));
        }
        for tag in &self.source {
            let evidence = tag.evidence().trim();
            if evidence.is_empty() || (tag.is_reconstructed() && !evidence.starts_with("(R)")) {
                return Err(invalid("invalid Station Cash reward provenance"));
            }
        }
        Ok(())
    }

    /// The Station Cash one completion pays before the daily bonus.
    pub fn amount(&self, source: EarnSource) -> u32 {
        match source {
            EarnSource::KartRace => self.kart_race,
            EarnSource::DerbyMatch => self.derby_match,
            EarnSource::SoccerMatch => self.soccer_match,
            EarnSource::ChessGame { .. } => self.chess_game,
            EarnSource::Quest { id } => self
                .quest_overrides
                .get(&id)
                .copied()
                .unwrap_or(self.quest_complete),
            EarnSource::DungeonClear { id } => {
                self.dungeon_overrides.get(&id).copied().unwrap_or_else(|| {
                    if self.hardest_dungeons.contains(&id) {
                        self.hardest_dungeon_clear
                    } else {
                        self.dungeon_clear
                    }
                })
            }
        }
    }

    /// Whether the completion counts at all (R): only a chess game that reached a result through
    /// play does. A refused completion pays nothing, not even the daily bonus.
    fn counts(&self, source: EarnSource) -> bool {
        match source {
            EarnSource::ChessGame { won_or_drew, plies } => {
                won_or_drew || plies >= self.chess_min_plies
            }
            _ => true,
        }
    }

    /// Decide one completion against the staged wallet. `day` is the character's server day; the
    /// first completion of a day adds the bonus once. The first-clear ride is recorded in the
    /// wallet in the same staged transaction, so a failed grant records nothing. The once-only
    /// proof is `earned_rides`, never current ownership: rides trade (T429), and trading the ride
    /// away must not re-arm the award. A ride the bag cannot hold is not recorded, so a later
    /// clear with room still grants it.
    pub fn earned(
        &self,
        wallet: &mut Wallet,
        source: EarnSource,
        day: u64,
        ride: Option<RideOffer>,
    ) -> Earned {
        if !self.counts(source) {
            return Earned::default();
        }
        let mut sc = self.amount(source);
        if wallet.last_activity_day != Some(day) {
            wallet.last_activity_day = Some(day);
            sc = sc.saturating_add(self.daily_first_activity);
        }
        let mut earned = Earned {
            sc,
            ..Earned::default()
        };
        let ride = ride.filter(|offer| {
            self.first_clear_ride.is_some_and(|first| {
                offer.item_id == first.item_id
                    && matches!(source, EarnSource::DungeonClear { id } if id == first.dungeon_id)
            })
        });
        if let Some(offer) = ride {
            let item = u32::try_from(offer.item_id).unwrap_or(0);
            if item != 0 && !wallet.earned_rides.contains(&item) {
                if !offer.owned && !offer.room {
                    earned.bag_full = true;
                } else {
                    wallet.earned_rides.push(item);
                    if wallet.earned_rides.len() > MAX_EARN_RIDES {
                        let drop = wallet.earned_rides.len() - MAX_EARN_RIDES;
                        wallet.earned_rides.drain(..drop);
                    }
                    if offer.owned {
                        earned.sc = earned.sc.saturating_add(offer.sc_equivalent);
                    } else {
                        earned.ride = Some(offer.item_id);
                    }
                }
            }
        }
        earned
    }
}

/// (R) The wall-clock UTC day index used by the first-activity bonus; the gateway never reads a
/// client clock.
pub fn activity_day(now_seconds: u64) -> u64 {
    now_seconds / ACTIVITY_DAY_SECONDS
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

/// L046 (R): `ClientItemDefinition.Type` of a property deed. Every lot deed recorded in
/// `houses.json` has Type 16 in the item catalogue (the word `Inventory::presentation` reads after
/// the two strings); the client has no deed handling of its own (no `deed` string, none in
/// ScriptsBase.lua).
pub const DEED_ITEM_TYPE: i32 = 16;

/// T657 (R): `ClientItemDefinition.Type` of a ride, the client's ride-record action switch
/// (`00a1ec00` case 19, see `notes/Autopilot/T452.md`). An untagged ride row with this type still
/// lists under the Rides parent, so content without `mounts.json` keeps its rides.
pub const RIDE_ITEM_TYPE: i32 = 19;

/// One row the Marketplace lists: a store bundle equal to one ride item, or to one lot deed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Listing {
    pub bundle_id: u32,
    pub item_id: i32,
    pub name_id: u32,
    /// T657: the item definition's description locale id, or its name id when it has none (R).
    /// The client's `Description` column is bundle +164 (`009a56d0` case 4); a 0 here is the
    /// `<STRING 0 NOT FOUND>` the native run showed.
    pub description_id: u32,
    /// T657: the item definition's image-set id and tint, written as the bundle's two image
    /// strings. `00b2ea50` stores them and the client `atoi`s them for `ImageSetId` /
    /// `ImageTintValue` (`009a56d0` cases 6-7). 0 writes an empty string (no icon).
    pub icon_id: u32,
    pub icon_tint: u32,
    /// The regular Station Cash price, written at +176 (`009a56d0` case 5). The members-only price
    /// at +268 is written as the same value: no row carries a member SC price, and the client reads
    /// +268 as the member price, so 0 would be a 100% discount (T744).
    pub price: u32,
    /// The category group the bundle belongs to: a ride child, Rides, Boombox, or the T657 item
    /// shelf whose id is also its group key.
    pub group: u32,
    /// L046: the item is a lot deed. An order for it grants the lot through the caller's
    /// [`LotGrant`] and never puts the deed in the inventory.
    pub deed: bool,
    /// The item is a ride (`MountCatalog::mount_for_item` knows it, the rule the Rides roster
    /// uses). Only a ride is single-copy: every other listing sells again and again (T484 review).
    pub ride: bool,
}

/// T657: one `stores.json` row as the catalogue sees it. `category_id` is the row's item category
/// (the item definition's `CategoryId`), which the Marketplace category table maps to a shelf.
#[derive(Clone, Copy, Debug)]
pub struct StoreRow {
    pub item_id: i32,
    pub sc_price: u32,
    pub kind: RowKind,
    pub ride_category: Option<RideCategory>,
    pub category_id: Option<u32>,
}

impl Listing {
    /// The player already holds this ride, so a second order is refused and 66/39 marks it
    /// view-only. A deed is not a ride (it never enters the bag; its owner check is the lot
    /// grant), and a stackable listing such as a bag or a flair shard is never "owned".
    pub fn owned_by(&self, inventory: &InventoryState) -> bool {
        self.ride && owns_item(inventory, self.item_id)
    }
}

/// L046: the caller's lot grant for a paid deed. It receives the deed's item id (always from the
/// server-side listing, never from the client) and `commit`. It returns the lot's replies, or
/// `None` with nothing changed (no lot for the deed, the owner already has it, or no housing).
/// With `commit` false it only answers whether the grant would succeed and changes nothing.
pub type LotGrant<'a> = &'a mut dyn FnMut(i32, bool) -> Option<Vec<Vec<u8>>>;

/// How a `stores.json` row with an `sc_price` is sold (T495).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    /// Listed under the Rides category (every priced row that is not one of the others).
    Ride,
    /// Listed under the Boombox category.
    Boombox,
    /// Slice 2 reward row: never listed, never ordered.
    EarnOnly,
}

/// The validated Station Cash catalogue built from `stores.json` rows and the live item catalogue.
#[derive(Clone, Debug)]
pub struct Catalogue {
    config: Config,
    listings: Vec<Listing>,
    /// Ride rows that must never be listed (slice 2 rewards).
    earn_only: Vec<Listing>,
}

/// One 66/7 category node: its 66/6 group, its parent category (0 at the top level) and its
/// display order (T526).
#[derive(Clone, Copy)]
struct TreeNode {
    shelf: Shelf,
    parent_id: u32,
    display_order: u32,
}

impl Catalogue {
    pub fn build(
        config: Config,
        inventory: &Inventory,
        is_ride: impl Fn(i32) -> bool,
        rows: impl IntoIterator<Item = StoreRow>,
    ) -> io::Result<Self> {
        config.validate()?;
        let mut listings = Vec::new();
        let mut earn_only = Vec::new();
        for row in rows {
            let StoreRow {
                item_id,
                sc_price,
                kind,
                ride_category,
                category_id,
            } = row;
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
            let ride = is_ride(item_id);
            let name_id = u32::try_from(presentation.name_id).unwrap_or(0);
            let group = config.listing_group(
                kind,
                ride_category,
                category_id,
                presentation.item_type,
                ride,
            )?;
            let listing = Listing {
                bundle_id: u32::try_from(item_id)
                    .map_err(|_| invalid("invalid station cash id"))?,
                item_id,
                name_id,
                // (R) An item with no description id shows its name there: the client prints
                // `<STRING 0 NOT FOUND>` for id 0 (native run, 2026-10-25).
                description_id: u32::try_from(presentation.description_id)
                    .ok()
                    .filter(|id| *id != 0)
                    .unwrap_or(name_id),
                icon_id: u32::try_from(presentation.icon_id).unwrap_or(0),
                icon_tint: u32::try_from(presentation.icon_tint).unwrap_or(0),
                price: config.scaled_price(sc_price)?,
                group,
                deed: presentation.item_type == DEED_ITEM_TYPE,
                ride,
            };
            if kind == RowKind::EarnOnly {
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

    /// The 66/7 tree nodes the store shows: Rides, Boombox and the T657 item shelves at the top
    /// level, then the ride sub-categories under Rides (T526).
    fn tree_nodes(&self) -> Vec<TreeNode> {
        let config = &self.config;
        let rides = Shelf {
            category_id: config.category_id,
            category_name_id: config.category_name_id,
            category_group_id: config.category_group_id,
        };
        let item_shelves = config.item_shelves.iter().map(|shelf| TreeNode {
            shelf: Shelf {
                category_id: shelf.category_id,
                category_name_id: shelf.category_name_id,
                // T657: the shelf id is its own 66/6 group key, like the T526 ride children.
                category_group_id: shelf.category_id,
            },
            parent_id: 0,
            display_order: shelf.display_order,
        });
        let children = config.ride_categories.iter().map(|shelf| TreeNode {
            shelf: Shelf {
                category_id: shelf.category_id,
                category_name_id: shelf.category_name_id,
                // Legacy gives each ride group its category's own id (266-269).
                category_group_id: shelf.category_id,
            },
            parent_id: config.category_id,
            display_order: shelf.display_order,
        });
        std::iter::once(TreeNode {
            shelf: rides,
            parent_id: 0,
            display_order: 1,
        })
        .chain(config.boombox_shelf.map(|shelf| TreeNode {
            shelf,
            parent_id: 0,
            display_order: 2,
        }))
        .chain(item_shelves)
        .chain(children)
        .collect()
    }

    /// T657: the 66/7 count written per node. The client zeroes every node's count (`00c402b0`)
    /// and recomputes it from the bundles, walking each match up the parent chain (`00c37e70`), so
    /// only the transient tree before the bundles arrive sees this number; it is the same value
    /// (a parent counts its descendants too) and is kept so that tree is not empty.
    fn category_counts(&self, nodes: &[TreeNode]) -> Vec<u32> {
        let mut counts = vec![0u32; nodes.len()];
        let group_index: BTreeMap<u32, usize> = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.shelf.category_group_id, index))
            .collect();
        for listing in &self.listings {
            let Some(&leaf) = group_index.get(&listing.group) else {
                continue;
            };
            let mut at = Some(leaf);
            while let Some(index) = at {
                counts[index] = counts[index].saturating_add(1);
                let parent = nodes[index].parent_id;
                at = if parent == 0 {
                    None
                } else {
                    nodes
                        .iter()
                        .position(|node| node.shelf.category_id == parent)
                };
            }
        }
        counts
    }

    pub fn earn_only(&self) -> &[Listing] {
        &self.earn_only
    }

    pub fn listing(&self, bundle_id: u32) -> Option<&Listing> {
        self.listings
            .iter()
            .find(|listing| listing.bundle_id == bundle_id)
    }

    /// The Marketplace Station Cash value of an earn-only ride (never listed). This is the
    /// "SC equivalent" a first-clear award pays when the ride is already owned.
    pub fn earn_only_price(&self, item_id: i32) -> Option<u32> {
        self.earn_only
            .iter()
            .find(|listing| listing.item_id == item_id)
            .map(|listing| listing.price)
    }
}

/// One successfully paid order, persisted with the wallet so a reconnect cannot pay it twice.
///
/// The client's tracking id comes from a process-global counter that starts at 1 each time the
/// client runs (`00b28360`); a restarted client can legitimately reuse an id, so the replay key is
/// the whole order (id, store, bundle, quantity, tint), never the id alone.
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
/// so a disconnected or replayed order packet cannot be paid again. `earned_rides` records the
/// first-clear rides already awarded and `last_activity_day` the last day the first-completion
/// bonus was paid; both are written only by the reward applier's staged transaction.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wallet {
    #[serde(default)]
    pub balance: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub orders: Vec<OrderRecord>,
    /// Earn-only ride item ids whose first-clear award was already settled, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub earned_rides: Vec<u32>,
    /// UTC day index (`activity_day`) of the last paid first-completion bonus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_activity_day: Option<u64>,
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
        wallet.earned_rides.retain(|item| *item > 0);
        wallet.earned_rides.dedup();
        if wallet.earned_rides.len() > MAX_EARN_RIDES {
            let drop = wallet.earned_rides.len() - MAX_EARN_RIDES;
            wallet.earned_rides.drain(..drop);
        }
        wallet
    }

    pub fn is_empty(&self) -> bool {
        self.balance == 0
            && self.orders.is_empty()
            && self.earned_rides.is_empty()
            && self.last_activity_day.is_none()
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

#[derive(Default)]
pub struct OrderOutcome {
    pub result_code: u32,
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
    /// T486: the inventory stacks a paid order added, as `(instance id, units)`, so the caller can
    /// adopt a bought pet item at once (`PetService::adopt_granted`). A preview, refusal, deed or
    /// replayed order adds none.
    pub granted: Vec<(i32, u32)>,
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

/// One 66/27 store-bundle content request entry: the store and the bundle whose content the
/// client asked for. The client also sends the marketing item ids it read from that bundle's
/// entry list (`00b3dd80`/`00b37140`); they are consumed and discarded, because the reply is
/// driven by the server's listing, never by a client id (T744).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentRequest {
    pub store_id: u32,
    pub bundle_id: u32,
}

/// Read a 66/27 content request: a count, then per entry a store id, a bundle id, an item-id
/// count and that many ids. The client's serializer is `00b41a30` -> `00b3dd80` -> `00b37140`;
/// the Legacy reference names the same three fields (behaviour reference only). A count above
/// the bound, an entry that runs past the body or any trailing byte is refused.
pub fn decode_content_request(inner: &[u8]) -> io::Result<Vec<ContentRequest>> {
    if inner.len() < 8
        || inner.len() > MAX_ORDER_PACKET
        || inner[..2] != MARKETPLACE_FAMILY.to_le_bytes()
        || inner[2..4] != OP_CONTENT_REQUEST.to_le_bytes()
    {
        return Err(invalid("not a Marketplace content request"));
    }
    let mut reader = Reader {
        bytes: &inner[4..],
        at: 0,
    };
    let count = reader.u32()? as usize;
    if count > MAX_CONTENT_REQUESTS {
        return Err(invalid("too many Marketplace content requests"));
    }
    let mut requests = Vec::with_capacity(count);
    for _ in 0..count {
        let store_id = reader.u32()?;
        let bundle_id = reader.u32()?;
        // The ids are a hint: consume them so the frame must end exactly, but never keep them.
        for _ in 0..reader.u32()? {
            reader.u32()?;
        }
        requests.push(ContentRequest {
            store_id,
            bundle_id,
        });
    }
    reader.finish()?;
    Ok(requests)
}

/// 66/28: the counted `(store id, bundle id)` pairs whose item definitions the reply carries.
/// Client reader `00b2e2d0` reads the count and the eight-byte records into the response
/// object; `00b40050` then clears each bundle's pending-content entry. Send it only for a
/// non-empty set: the handler's base parse (`00b40050` -> `sub_B2DFD0`: two u16 then two u32)
/// needs 12 bytes, so a count-zero 8-byte reply is logged as a bad
/// `InGamePurchaseStoreBundlesBasePacket` and ignored.
pub fn content_response_packet(entries: &[(u32, u32)]) -> Vec<u8> {
    let mut bytes = header(OP_CONTENT_RESPONSE);
    put_i32(&mut bytes, entries.len() as i32);
    for (store_id, bundle_id) in entries {
        put_u32(&mut bytes, *store_id);
        put_u32(&mut bytes, *bundle_id);
    }
    bytes
}

/// Price and validate an order without changing anything; 66/2 reply. A new deed order also asks
/// `grant_lot` without committing, so a lot the order would refuse is refused here too.
pub fn preview(
    catalogue: &Catalogue,
    wallet: &Wallet,
    inventory: &InventoryState,
    order: &OrderRequest,
    grant_lot: LotGrant,
) -> OrderOutcome {
    let (result, total) = match price(catalogue, wallet, inventory, order) {
        Ok((listing, _, _, None))
            if listing.deed && grant_lot(listing.item_id, false).is_none() =>
        {
            (ORDER_FAILURE, 0)
        }
        Ok((_, _, total, _)) => (ORDER_SUCCESS, total),
        Err(result) => (result, 0),
    };
    OrderOutcome {
        result_code: result,
        replies: vec![preview_packet(order.tracking_id, result, total)],
        ..OrderOutcome::default()
    }
}

/// Debit Station Cash and grant the ride (or, for a deed, the lot) in one staged commit; 66/4
/// reply. The grant is the last step that can fail; the debit after it cannot.
pub fn place_order(
    catalogue: &Catalogue,
    wallet: &mut Wallet,
    inventory: &mut Inventory,
    order: &OrderRequest,
    bag_capacity: usize,
    busy: bool,
    grant_lot: LotGrant,
) -> OrderOutcome {
    if busy {
        return refusal(order, ORDER_FAILURE);
    }
    let (listing, detail, total, replay) = match price(catalogue, wallet, inventory.state(), order)
    {
        Ok(priced) => priced,
        Err(result) => return refusal(order, result),
    };
    // A replay of a fully paid order returns its result without touching state.
    if let Some(old_total) = replay {
        return OrderOutcome {
            result_code: ORDER_SUCCESS,
            replies: vec![
                wallet_packet(wallet.balance),
                order_packet(
                    order.tracking_id,
                    ORDER_SUCCESS,
                    &order.tracking_id.to_string(),
                    old_total,
                ),
            ],
            ..OrderOutcome::default()
        };
    }
    if wallet.balance < total {
        return refusal(order, ORDER_INSUFFICIENT_SC);
    }
    let (granted, added) = if listing.deed {
        // L046: a deed never enters the inventory; no lot for it means no sale.
        match grant_lot(listing.item_id, true) {
            Some(replies) => (replies, Vec::new()),
            None => return refusal(order, ORDER_FAILURE),
        }
    } else {
        let mut staged = inventory.clone();
        let granted = match staged.grant_with_tint(0, listing.item_id, 1, detail.tint) {
            Ok(granted) => granted,
            Err(_) => return refusal(order, ORDER_FAILURE),
        };
        if staged.state().items.len() > bag_capacity {
            return refusal(order, ORDER_FAILURE);
        }
        *inventory = staged;
        (granted.replies, granted.granted)
    };
    let balance = wallet.balance - total;
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
    replies.extend(granted);
    replies.push(order_packet(
        order.tracking_id,
        ORDER_SUCCESS,
        &order.tracking_id.to_string(),
        total,
    ));
    OrderOutcome {
        result_code: ORDER_SUCCESS,
        replies,
        changed: true,
        granted: added,
    }
}

/// Whether the player owns the item: an instance of that exact item id (R; a colour variant is
/// its own item id, so another tint is a separate purchase, see `notes/Autopilot/T484.md`) with a
/// positive count. A rental, expired or not, is not owned (R). The one "already owned" rule for
/// the Marketplace order path, its 66/39 flags and the earn-only ride award.
pub fn owns_item(inventory: &InventoryState, item_id: i32) -> bool {
    inventory
        .items
        .iter()
        .any(|item| item.definition_id == item_id && item.count > 0 && item.rental.is_none())
}

/// Checks run in this order: the listing, the replay (a paid duplicate still answers success),
/// ownership of a ride (a second copy would stack or duplicate it and waste the SC), the balance.
fn price<'a>(
    catalogue: &'a Catalogue,
    wallet: &Wallet,
    inventory: &InventoryState,
    order: &OrderRequest,
) -> Result<(&'a Listing, OrderDetail, u32, Option<u32>), u32> {
    if order.gift || order.coupon || order.details.len() != 1 {
        return Err(ORDER_FAILURE);
    }
    let detail = order.details[0];
    if detail.quantity != 1 || detail.store_id != catalogue.config.store_id {
        return Err(ORDER_FAILURE);
    }
    let listing = catalogue.listing(detail.bundle_id).ok_or(ORDER_FAILURE)?;
    let total = listing.price;
    let replay = wallet.replay(&detail, order.tracking_id);
    if replay.is_none() {
        if listing.owned_by(inventory) {
            return Err(ORDER_FAILURE);
        }
        if wallet.balance < total {
            return Err(ORDER_INSUFFICIENT_SC);
        }
    }
    Ok((listing, detail, total, replay))
}

fn refusal(order: &OrderRequest, result: u32) -> OrderOutcome {
    OrderOutcome {
        result_code: result,
        replies: vec![order_packet(
            order.tracking_id,
            result,
            &order.tracking_id.to_string(),
            0,
        )],
        ..OrderOutcome::default()
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

/// 66/2 (T453). Reader `00b2de20` stores tracking id, result, discount and total at
/// obj+0x18..+0x24; handler `00b3b450` treats the second dword (obj+0x1C) 1 as success.
pub(crate) fn preview_packet(tracking_id: u32, result: u32, total: u32) -> Vec<u8> {
    let mut bytes = header(OP_PREVIEW_RESPONSE);
    put_u32(&mut bytes, tracking_id);
    put_u32(&mut bytes, result);
    put_u32(&mut bytes, 0);
    put_u32(&mut bytes, total);
    bytes
}

/// 66/4 (T453). Reader `00b2ded0` stores the tracking id at obj+0x18 and the result at obj+0x1C,
/// then the order id string, discount (+0x4C) and total (+0x50). Handler `00b3b6d0` treats the
/// second dword 1 as success and completes the pending order only when the first matches it.
/// The order id is the tracking id in decimal (R) and the disabled reply's message.
pub(crate) fn order_packet(tracking_id: u32, result: u32, order_id: &str, total: u32) -> Vec<u8> {
    let mut bytes = header(OP_ORDER_RESPONSE);
    put_u32(&mut bytes, tracking_id);
    put_u32(&mut bytes, result);
    put_string(&mut bytes, order_id);
    put_u32(&mut bytes, 0);
    put_u32(&mut bytes, total);
    bytes
}

/// 66/39 (T484). Reader `00b3e490` takes a counted list (`00b32d00`: i32 count, then per entry a
/// u32 bundle id and a u8 flag, `00b2b590`) and stores `flag == 0` in each found bundle's byte
/// +242, the `IsViewOnly` column (`009a56d0` case 27). So flag 0 marks an owned ride view-only
/// and flag 1 leaves a listing purchasable (every non-ride listing, always). Every listing is
/// named, so a ride the player no longer owns is purchasable again at the next 66/38; the order
/// path (`price`) stays the authority.
pub fn requirements_packet(catalogue: &Catalogue, inventory: &InventoryState) -> Vec<u8> {
    let mut bytes = header(OP_ITEM_REQUIREMENTS);
    put_i32(&mut bytes, catalogue.listings.len() as i32);
    for listing in &catalogue.listings {
        put_u32(&mut bytes, listing.bundle_id);
        bytes.push(u8::from(!listing.owned_by(inventory)));
    }
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

/// 66/7 + category tree (`00b48040`/`00b43090`/`00b3fd70`/`00b3b080`): one node per shelf, each
/// counting its own bundles and its descendants' (the client walks the parent chain the same way,
/// `00c37e70`). `00b3b080` reads the parent as the first of its three record dwords and
/// `00b43090` re-keys every node by it, so a child must name the Rides category here.
fn category_tree_packet(catalogue: &Catalogue) -> Vec<u8> {
    let nodes = catalogue.tree_nodes();
    let counts = catalogue.category_counts(&nodes);
    let mut bytes = header(OP_CATEGORIES);
    put_i32(&mut bytes, nodes.len() as i32);
    for (node, count) in nodes.iter().zip(counts) {
        put_u32(&mut bytes, node.shelf.category_id);
        put_u32(&mut bytes, node.shelf.category_id);
        put_u32(&mut bytes, node.shelf.category_name_id);
        put_string(&mut bytes, "");
        put_string(&mut bytes, "");
        put_u32(&mut bytes, node.parent_id);
        put_u32(&mut bytes, count);
        put_u32(&mut bytes, node.display_order);
    }
    bytes.push(1);
    bytes
}

/// 66/6 + one category group per shelf, each holding its category (`00b44d70`/`00b3ad20`).
fn category_groups_packet(catalogue: &Catalogue) -> Vec<u8> {
    let nodes = catalogue.tree_nodes();
    let mut bytes = header(OP_CATEGORY_GROUPS);
    put_i32(&mut bytes, nodes.len() as i32);
    for node in &nodes {
        put_u32(&mut bytes, node.shelf.category_group_id);
        put_u32(&mut bytes, node.shelf.category_group_id);
        put_i32(&mut bytes, 1);
        put_u32(&mut bytes, node.shelf.category_id);
    }
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

/// One bundle record in the reader's order (`00b3da50` then `00b3fa00` then `00b41860`). The
/// field positions are the client's; the values T657 sets from data are the description, the two
/// image strings, the price and the sale flag.
fn bundle_record(bytes: &mut Vec<u8>, catalogue: &Catalogue, listing: &Listing, index: usize) {
    let config = &catalogue.config;
    put_u32(bytes, listing.bundle_id);
    put_u32(bytes, listing.name_id);
    // +164 Description (`009a56d0` case 4); never 0 (`Listing::description_id`).
    put_u32(bytes, listing.description_id);
    put_u32(bytes, 1);
    // The image object at +0x18 (`00b2ea50`): image set and tint strings, `atoi`'d by the client
    // for `ImageSetId`/`ImageTintValue` (`009a56d0` cases 6-7). An empty string means no icon.
    put_string(bytes, &decimal(listing.icon_id));
    put_string(bytes, &decimal(listing.icon_tint));
    bytes.push(0);
    put_string(bytes, "");
    put_u32(bytes, config.currency_type);
    // +176 Price (`009a56d0` case 5).
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
    put_u32(bytes, listing.group);
    // +243 IsOnSale, or +228 non-zero (`00997300`, `009a56d0` case 9). No shipped row is on sale;
    // the old writer's 1 here is why every tile carried the SALE! badge and showed SalePrice 0
    // (`009a56d0` case 10) instead of Price.
    bytes.push(0);
    // +208 is the terms locale id, not a visibility byte: `009a56d0` cases 14-15 read it as
    // `HasTermsAndConditions`/`TermsAndConditions`. The old 1 named locale 1.
    put_u32(bytes, 0);
    // +212 MaxPerOrder (`009a56d0` case 16). The order path accepts quantity 1 only, so is 1.
    put_u32(bytes, 1);
    put_u32(bytes, index as u32 + 1);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    bytes.push(0);
    bytes.push(0);
    bytes.push(0);
    // Client reader `00b41860` continues after the `00b3fa00` tail with ten more fields:
    // u32, u32, u8, u32, u32, u32, u8, u8, u32, u32 (31 bytes). Only the members-only price
    // (field 5, +268) and the sale price (field 6, +272) have recovered readers (`009a56d0`
    // cases 32 and 10); the rest stay zero (R). A short record makes the client consume the
    // next bundle's map key as these fields, misaligning every following entry.
    put_u32(bytes, 0);
    put_u32(bytes, 0);
    bytes.push(0);
    put_u32(bytes, 0);
    // +268 MembersOnlyPrice (`009a56d0` case 32). No shipped row has a member SC price, and the
    // client's member price is this field alone (`MarketplaceDataSource.getMOItemPrice`), so 0
    // is a 100% member discount. The regular price here means "no discount" (T744 native run).
    put_u32(bytes, listing.price);
    // +272 SalePrice (`009a56d0` case 10); IsOnSale is 0, so this field is never read as the price.
    put_u32(bytes, 0);
    bytes.push(0);
    bytes.push(0);
    put_u32(bytes, 0);
    put_u32(bytes, 0);
}

/// The decimal image/tint string the client `atoi`s; 0 means no icon and writes an empty string.
fn decimal(value: u32) -> String {
    if value == 0 {
        String::new()
    } else {
        value.to_string()
    }
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
