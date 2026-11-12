//! Coin Shop purchases and the Station Cash Marketplace handshake.
//!
//! Client packet readers establish the family/opcode headers and transaction result fields.
//! Numeric result-code meanings, resale pricing, buy-back window, merchant authorization and
//! Marketplace defaults are reconstructed policy (R). No persistence or packet transport is
//! performed here; callers save the returned state before sending replies.
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
//! and the list. T230 adds the per-character Station Cash wallet and ride orders; wallet funding,
//! membership and real-money orders stay refused (R).

use crate::inventory::Inventory;
use crate::le;
use crate::shop_stock;
use crate::station_cash::{self, RowKind, StoreRow, Wallet};
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
    /// T230 Station Cash policy. A missing block keeps the Marketplace payment-free.
    #[serde(default)]
    station_cash: station_cash::Config,
    marketplace: MarketplaceConfig,
    items: Vec<StoreItem>,
    /// F228/L040 free issuance (job starter outfits, the Condo starter kit), set by
    /// `CoinShop::new`: every copy resells for 0 wherever the player got it (`sell_unit_price`).
    #[serde(skip)]
    free_items: BTreeSet<i32>,
    /// T484: the ride catalogue beside `stores.json` (`mounts.json`, the one the Rides roster
    /// uses); a listing it knows is a ride, the only single-copy listing. Absent (a content set
    /// without rides, or a fixture directory) means no listing is a ride.
    #[serde(skip)]
    rides: Option<Arc<crate::mounts::MountCatalog>>,
    /// B2: item -> the `live_event` that alone sells it, built by `load`.
    #[serde(skip)]
    seasonal: BTreeMap<i32, String>,
    /// B2: item -> its limited-stock counters (`shop_stock`), built by `load`.
    #[serde(skip)]
    limits: BTreeMap<i32, shop_stock::Limit>,
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
    /// T230: original Station Cash price (unscaled). Marketplace-only rows cannot also be
    /// bought or sold for coins, so the two currencies never meet in one row.
    #[serde(default)]
    sc_price: Option<u32>,
    /// T230: never listed in the Marketplace (slice 2 reward rides such as the Chrome Drake).
    #[serde(default)]
    earn_only: bool,
    /// T495: listed under the Marketplace's Boombox category instead of Rides.
    #[serde(default)]
    boombox: bool,
    /// T526: the Marketplace ride sub-category this row lists under. Absent keeps the row under
    /// the Rides parent; an unknown value is refused by `StoreCatalogue::load`.
    #[serde(default)]
    ride_category: Option<station_cash::RideCategory>,
    /// T657: the row's item category (the item definition's `CategoryId`). The category table
    /// maps it to a Marketplace shelf; absent or 0 falls back to the item type.
    #[serde(default)]
    category_id: Option<u32>,
    /// B2: sold only while this `event_calendar.json` event runs (the event-repeat contract's
    /// `live_event`); every coin, vendor and Marketplace path asks [`CoinShop::sale_block`] or
    /// the in-season Marketplace view.
    #[serde(default)]
    live_event: Option<String>,
    /// B2: units one run of `live_event` sells to everyone, shared by the row's `stock_group`.
    #[serde(default)]
    stock_per_event: Option<u32>,
    /// B2: units one character may buy per run (per row group).
    #[serde(default)]
    per_player: Option<u32>,
    /// B2: rows sharing one stock counter (every colour and gender row of an outfit).
    #[serde(default)]
    stock_group: Option<String>,
    /// B2: the ride family (`station_cash.ride_families` key) the row lists under; a key no family
    /// names is refused at load.
    #[serde(default)]
    family: Option<String>,
    /// B2: the row's own colour or variant label inside its family (content for the Studio and
    /// for a client that draws a colour picker; the stock client shows each row's own name).
    #[serde(default)]
    variant: Option<String>,
}

/// B2: the season tags and the limited-stock groups of `items`, refusing a malformed one: a key
/// is 1..=32 ASCII graphic bytes, a cap is at least 1, a per-event cap needs an event, a group
/// needs a cap, and every row of one group carries the same event and caps.
fn event_index(
    items: &[StoreItem],
) -> io::Result<(BTreeMap<i32, String>, BTreeMap<i32, shop_stock::Limit>)> {
    let key_ok =
        |key: &str| (1..=32).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_graphic());
    let mut seasonal = BTreeMap::new();
    let mut limits = BTreeMap::new();
    let mut groups: BTreeMap<String, shop_stock::Limit> = BTreeMap::new();
    for item in items {
        let event = item.live_event.clone();
        let limited = item.stock_per_event.is_some() || item.per_player.is_some();
        if event.as_deref().is_some_and(|key| !key_ok(key))
            || item.stock_per_event == Some(0)
            || item.per_player == Some(0)
            || (item.stock_per_event.is_some() && event.is_none())
            || item
                .stock_group
                .as_deref()
                .is_some_and(|group| !limited || !key_ok(group))
        {
            return Err(invalid("invalid store item event or stock fields"));
        }
        if let Some(key) = &event {
            seasonal.insert(item.item_id, key.clone());
        }
        if !limited {
            continue;
        }
        let limit = shop_stock::Limit {
            group: (item.stock_group.clone()).unwrap_or_else(|| format!("item:{}", item.item_id)),
            event,
            per_event: item.stock_per_event,
            per_player: item.per_player,
        };
        if groups
            .entry(limit.group.clone())
            .or_insert_with(|| limit.clone())
            != &limit
        {
            return Err(invalid("store rows of one stock group disagree"));
        }
        limits.insert(item.item_id, limit);
    }
    Ok((seasonal, limits))
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
        let mut catalogue: Self = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        catalogue.station_cash.validate()?;
        let mounts = path.with_file_name("mounts.json");
        if mounts.is_file() {
            catalogue.rides = Some(Arc::new(crate::mounts::MountCatalog::load(mounts)?));
        }
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
                || item
                    .sc_price
                    .is_some_and(|price| price == 0 || price > station_cash::MAX_PRICE_SC)
                || (item.earn_only && item.sc_price.is_none())
                || (item.boombox && (item.earn_only || item.sc_price.is_none()))
                || (item.sc_price.is_some()
                    && (item.purchase_price_coins.is_some()
                        || item.member_price_coins.is_some()
                        || item.sell_price_coins.is_some()
                        || !item.no_sale))
            {
                return Err(invalid("invalid store item entry"));
            }
        }
        (catalogue.seasonal, catalogue.limits) = event_index(&catalogue.items)?;
        let config = &catalogue.station_cash;
        if (catalogue.items.iter()).any(|item| {
            (item.family.as_deref()).is_some_and(|key| config.family_index(key).is_none())
                || (item.variant.as_deref())
                    .is_some_and(|label| label.is_empty() || label.len() > 64)
        }) {
            return Err(invalid(
                "store row names an unknown ride family or a bad variant",
            ));
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
    /// T772: the inventory stacks this buy added, as `(instance id, units)`, so the caller can
    /// adopt pet adoption items at once. A refusal, sell or cached retry adds none.
    pub granted: Vec<(i32, u32)>,
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
    /// T230: the per-character Station Cash wallet. `GameplayState::sc_wallet` is its save form.
    sc_wallet: Wallet,
    /// T230: the validated ride catalogue, shared with every staged clone.
    sc_catalogue: Arc<station_cash::Catalogue>,
    /// T229: the earn table beside the campaign content; absent keeps earnings off.
    sc_earn: Option<station_cash::EarnTable>,
    /// T529: store-row item ids the bundled pets content binds to a species, built once with
    /// the catalogue. A type-2 row outside this set has no pet the server can create, so the
    /// coin store refuses it; a later pets-content binding makes the row sellable with no code
    /// change.
    pet_items: BTreeSet<i32>,
    /// T772: how many more pets the session's roster holds; a bought pet item is adopted at once,
    /// so a buy of more is refused before money moves. `None` until the session announces it, and
    /// then a bound pet buy fails closed rather than skipping the full-roster gate.
    pet_room: Option<usize>,
    vendor_transactions: BTreeMap<u32, (u32, Vec<u8>)>,
    sales: VecDeque<SaleRecord>,
    next_transaction_id: u32,
    /// F3: the login 165/1 lists the client holds, shared by every staged clone.
    store_lists: Arc<StoreLists>,
    /// F3: the stock of the merchant window last opened (165/10), which its 165/8 asks for.
    open_stock: Vec<i32>,
    /// F3: the Marketplace bundles of the Station Cash window last opened (66/42, a display).
    open_bundles: Vec<u32>,
    /// B2: every Marketplace listing, in season or not; `sc_catalogue` is the in-season view
    /// every order, list, display and offer reads.
    sc_all: Arc<station_cash::Catalogue>,
    /// B2: the running events among the store's `live_event` keys, each with its run (the
    /// instant it ends, `shop_stock`), as the session's calendar last said (`follow_events`).
    /// `None` until it says: a shop no calendar drives (a bare fixture) sells everything.
    open_events: Option<BTreeMap<String, u64>>,
    /// B2: the character buying, for `per_player` limits (`set_buyer`).
    buyer: u64,
    /// B2: where the limited-stock counters live; `None` uses the realm file beside the account
    /// store, or this session's memory when the realm has none (R).
    stock_file: Option<PathBuf>,
    stock_memory: shop_stock::Ledger,
}

/// F3: what the client's coin store already knows from the login 165/1 (`00c75fc0`): the static
/// ids and the dynamic records (id, three dwords, three flags), kept for the 165/9 answer.
#[derive(Debug, Default)]
pub struct StoreLists {
    pub static_ids: BTreeSet<i32>,
    pub dynamic: Vec<(i32, [u32; 3], [u8; 3])>,
}

impl CoinShop {
    pub fn new(
        inventory_content: &Path,
        inventory: Inventory,
        mut catalogue: StoreCatalogue,
        bag_capacity: usize,
        profiles: Option<&crate::profiles::Profiles>,
        wallet: Wallet,
    ) -> io::Result<Self> {
        // F228: a starter outfit item is granted free whenever its job is learned or reselected,
        // so the store pays 0 for it however the player got a copy. Reuse the session's loaded
        // profiles: a second parse would also reject a package whose saved progression is valid
        // but whose unused `default_state` is not. Purchase prices are untouched.
        // L040: the Condo starter kit in this content's houses.json is free issuance too.
        catalogue.free_items = profiles.map_or_else(Default::default, |p| p.all_starter_items());
        catalogue.free_items.extend(
            crate::housing_service::HousingContent::kit_items(inventory_content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))?,
        );
        // T529: one bundled-pets parse per shop construction; a buy is only a set lookup. This
        // is the same species binding `PetContent::definition_for_item` resolves at adoption, so
        // a type-2 row becomes coin-sellable the moment pets content binds it.
        let pets = crate::pets::PetService::bundled_content()?;
        let pet_items = catalogue
            .items
            .iter()
            .filter(|item| pets.definition_for_item(item.item_id).is_some())
            .map(|item| item.item_id)
            .collect();
        let sc_catalogue = station_cash::Catalogue::build(
            catalogue.station_cash.clone(),
            &inventory,
            |item_id| {
                catalogue
                    .rides
                    .as_ref()
                    .is_some_and(|rides| rides.mount_for_item(item_id).is_some())
            },
            catalogue.items.iter().filter_map(|item| {
                let kind = match (item.earn_only, item.boombox) {
                    (true, _) => RowKind::EarnOnly,
                    (false, true) => RowKind::Boombox,
                    (false, false) => RowKind::Ride,
                };
                item.sc_price.map(|sc_price| StoreRow {
                    item_id: item.item_id,
                    sc_price,
                    kind,
                    ride_category: item.ride_category,
                    family: (item.family.as_deref())
                        .and_then(|key| catalogue.station_cash.family_index(key)),
                    category_id: item.category_id,
                })
            }),
        )?;
        let sc_wallet = Wallet::restore(Some(wallet), sc_catalogue.config());
        let sc_all = Arc::new(sc_catalogue);
        Ok(Self {
            inventory_content: inventory_content.to_path_buf(),
            inventory,
            catalogue: Arc::new(catalogue),
            bag_capacity,
            // (R) The caller must set the verified account flag after construction.
            is_member: false,
            sc_wallet,
            sc_catalogue: sc_all.clone(),
            sc_all,
            open_events: None,
            buyer: 0,
            stock_file: None,
            stock_memory: shop_stock::Ledger::default(),
            // `load` fills this from the NPC content; `new` alone keeps earning off.
            sc_earn: None,
            pet_items,
            pet_room: None,
            vendor_transactions: BTreeMap::new(),
            sales: VecDeque::new(),
            next_transaction_id: 1,
            store_lists: Arc::default(),
            open_stock: Vec::new(),
            open_bundles: Vec::new(),
        })
    }

    /// F3: the login 165/1 lists this session's client was sent (the gateway's startup snapshot).
    pub fn set_store_lists(&mut self, lists: StoreLists) {
        self.store_lists = Arc::new(lists);
    }

    /// B2: follow the session's event calendar. `run_of(key)` is the running run of event
    /// `key` (`EventCalendar::is_running`, named by the instant it ends) or `None` when it is not
    /// running. A closed event's stock leaves the coin shop, every vendor window and the
    /// Marketplace view (a store screen sees it at its next 66/10). True when anything moved.
    pub fn follow_events(&mut self, run_of: impl Fn(&str) -> Option<u64>) -> bool {
        let keys: BTreeSet<&String> = self.catalogue.seasonal.values().collect();
        let open: BTreeMap<String, u64> = (keys.into_iter())
            .filter_map(|key| run_of(key).map(|run| (key.clone(), run)))
            .collect();
        if self.open_events.as_ref() == Some(&open) {
            return false;
        }
        self.open_events = Some(open);
        self.sc_catalogue = Arc::new(self.sc_all.without(|item| !self.in_season(item)));
        true
    }

    /// B2: whether `item_id` may be sold now: untagged, or its event is running (or no calendar
    /// drives this shop).
    pub fn in_season(&self, item_id: i32) -> bool {
        match (self.catalogue.seasonal.get(&item_id), &self.open_events) {
            (Some(key), Some(open)) => open.contains_key(key),
            _ => true,
        }
    }

    /// B2: the character whose buys `per_player` limits count.
    pub fn set_buyer(&mut self, character_guid: u64) {
        self.buyer = character_guid;
    }

    /// B2: keep the limited-stock counters in `path` (tests; the realm default is the file
    /// beside the account store).
    pub fn set_stock_file(&mut self, path: PathBuf) {
        self.stock_file = Some(path);
    }

    fn stock_path(&self) -> Option<PathBuf> {
        (self.stock_file.clone()).or_else(|| {
            crate::redeem_codes::store().map(|codes| codes.with_file_name(shop_stock::FILE))
        })
    }

    /// B2: `item_id`'s limit and the run it counts in, `None` for an unlimited row.
    fn limit_now(&self, item_id: i32) -> Option<(shop_stock::Limit, u64)> {
        let limit = self.catalogue.limits.get(&item_id)?.clone();
        let run = (limit.event.as_ref())
            .and_then(|key| self.open_events.as_ref()?.get(key).copied())
            .unwrap_or(0);
        Some((limit, run))
    }

    /// B2: why `quantity` more of `item_id` may not be sold to this buyer now (stock or share
    /// spent), read without taking anything.
    pub fn stock_refusal(
        &self,
        item_id: i32,
        quantity: u32,
    ) -> io::Result<Option<shop_stock::Refusal>> {
        let Some((limit, run)) = self.limit_now(item_id) else {
            return Ok(None);
        };
        let ledger = match self.stock_path() {
            Some(path) => shop_stock::Ledger::load(&path)?,
            None => self.stock_memory.clone(),
        };
        Ok(ledger.check(&limit, run, self.buyer, quantity).err())
    }

    /// B2: one sale of `quantity` units of `item_id` under its stock limit. An unlimited row runs
    /// `sell` as it is. A limited one holds the counters' lock across the sale: a spent stock or
    /// share answers `refused` and sells nothing; otherwise the units are counted (and saved)
    /// before `sell` runs and handed back when it reports no sale, so a crash never oversells.
    fn limited<T>(
        &mut self,
        item_id: i32,
        quantity: u32,
        refused: impl FnOnce(&mut Self, shop_stock::Refusal) -> T,
        sell: impl FnOnce(&mut Self) -> io::Result<(T, bool)>,
    ) -> io::Result<T> {
        let Some((limit, run)) = self.limit_now(item_id) else {
            return Ok(sell(self)?.0);
        };
        let path = self.stock_path();
        // A counters file another writer holds, or cannot be read or written, sells nothing: the
        // buy is refused as busy (the session goes on), never sold uncounted.
        let opened: io::Result<_> = (path.as_deref())
            .map(|path| {
                let lock = crate::account_store::AccountStoreLock::acquire_within(
                    path,
                    crate::account_store::STAFF_LOCK_WAIT,
                )?;
                Ok((lock, shop_stock::Ledger::load(path)?))
            })
            .transpose();
        let (_lock, mut ledger) = match opened {
            Ok(Some((lock, ledger))) => (Some(lock), ledger),
            Ok(None) => (None, std::mem::take(&mut self.stock_memory)),
            Err(_) => return Ok(refused(self, shop_stock::Refusal::Busy)),
        };
        let buyer = self.buyer;
        let outcome = match ledger.check(&limit, run, buyer, quantity) {
            Err(refusal) => Ok(refused(self, refusal)),
            Ok(()) => {
                ledger.take(&limit, run, buyer, quantity);
                match path.as_deref().map_or(Ok(()), |path| ledger.save(path)) {
                    Err(_) => {
                        ledger.give_back(&limit, run, buyer, quantity);
                        Ok(refused(self, shop_stock::Refusal::Busy))
                    }
                    Ok(()) => match sell(self) {
                        Ok((out, true)) => Ok(out),
                        other => {
                            ledger.give_back(&limit, run, buyer, quantity);
                            // ponytail: a give-back that cannot be saved leaves the counter one
                            // unit high (it never oversells); an operator can fix the file.
                            if let Some(Err(error)) = path.as_deref().map(|path| ledger.save(path))
                            {
                                eprintln!(
                                    "shop stock give-back for item {item_id} not saved: {error}"
                                );
                            }
                            other.map(|(out, _)| out)
                        }
                    },
                }
            }
        };
        if path.is_none() {
            self.stock_memory = ledger;
        }
        outcome
    }

    /// B2: one Station Cash order (66/3, or a display window's 66/43) under its bundle's stock
    /// limit; `Err` is the spent stock or share, with nothing placed. Every listing's bundle id is
    /// its item id (`station_cash::Catalogue::build`), and an order holds one unit.
    fn limited_order(
        &mut self,
        order: &station_cash::OrderRequest,
        place: impl FnOnce(&mut Self) -> station_cash::OrderOutcome,
    ) -> io::Result<Result<station_cash::OrderOutcome, shop_stock::Refusal>> {
        let item_id = (order.details.first())
            .and_then(|detail| i32::try_from(detail.bundle_id).ok())
            .unwrap_or(0);
        self.limited(
            item_id,
            1,
            |_, refusal| Err(refusal),
            |shop| {
                let outcome = place(shop);
                let sold = outcome.changed && outcome.result_code == station_cash::ORDER_SUCCESS;
                Ok((Ok(outcome), sold))
            },
        )
    }

    /// F3: a merchant window (165/10) listing `stock` was just sent; its 165/8 follows.
    pub fn open_merchant(&mut self, stock: Vec<i32>) {
        self.open_stock = stock;
        self.open_bundles.clear();
    }

    /// F3: a Station Cash window (66/42) listing `bundles` was just sent, a display's shop.
    pub fn open_sc_merchant(&mut self, bundles: Vec<u32>) {
        self.open_bundles = bundles;
        self.open_stock.clear();
    }

    /// F3: the 66/43 buy from the open Station Cash window (`00b43310` puts the window row's
    /// bundle first). It is paid as the Marketplace order 66/3 would be (`place_order`: listed,
    /// priced, wallet, bag, replay window), then answered with the window's own 66/44 result and
    /// the new balance (66/11). `None` when the bundle is not on the open window.
    pub fn buy_open_bundle(
        &mut self,
        bundle: u32,
        quantity: u32,
        tracking_id: u32,
        busy: bool,
        grant_lot: station_cash::LotGrant,
    ) -> Option<CoinShopOutcome> {
        if !self.open_bundles.contains(&bundle) {
            return None;
        }
        let store_id = self.sc_catalogue.config().store_id;
        let order = station_cash::OrderRequest {
            tracking_id,
            details: vec![station_cash::OrderDetail {
                bundle_id: bundle,
                store_id,
                quantity,
                tint: 0,
            }],
            coupon: false,
            gift: false,
        };
        // B2: a spent stock says so in chat before the window's own 66/44 result; a counters lock
        // the realm cannot take sells nothing, like any other refused order.
        let (outcome, said) = match self.limited_order(&order, |shop| {
            station_cash::place_order(
                &shop.sc_catalogue,
                &mut shop.sc_wallet,
                &mut shop.inventory,
                &order,
                shop.bag_capacity,
                busy,
                grant_lot,
            )
        }) {
            Ok(Ok(outcome)) => (outcome, None),
            Ok(Err(refusal)) => (
                Default::default(),
                Some(crate::chat::system(refusal.message())),
            ),
            Err(_) => (Default::default(), None),
        };
        let paid = outcome.changed && outcome.result_code == station_cash::ORDER_SUCCESS;
        Some(CoinShopOutcome {
            result_code: if paid { RESULT_SUCCESS } else { RESULT_FAILURE },
            replies: (said.into_iter())
                .chain([
                    merchant_buy_result_packet(paid),
                    station_cash::wallet_packet(self.sc_wallet.balance),
                ])
                .collect(),
            transaction_id: tracking_id,
            changed: outcome.changed,
            granted: outcome.granted,
        })
    }

    /// F3: the 165/9 dynamic list update (`00c76100`, records read by `00c74200`/`00c71510`). The
    /// client swaps its whole dynamic list for this one, and the merchant window's CanBuy column
    /// (`00cff140` case 31) is 0 for an item in neither list or whose first flag is set
    /// (`00c70fe0`), so the Buy button stays grey. So: the login dynamic records, each the open
    /// merchant stocks made buyable, then the merchant's other items the static list lacks, with
    /// the third flag set so the gamedock store keeps them unlisted (`00cf1ac0`), as the 18 such
    /// static rows are (R). A static id is never repeated: the next update unlinks by id.
    /// rc-0137: `own` are rows another service keeps in the same list (the bank's upgrades, with
    /// their own lock); they go first and replace a login row of the same id, since this one
    /// update is the client's whole dynamic list.
    pub fn dynamic_list_update(&self, own: &[(i32, [u32; 3], [u8; 3])]) -> Vec<u8> {
        let lists = &self.store_lists;
        let stocks = |id: i32| self.open_stock.contains(&id);
        let owned = |id: i32| own.iter().any(|record| record.0 == id);
        let mut records: Vec<(i32, [u32; 3], [u8; 3])> = own.to_vec();
        records.extend((lists.dynamic.iter()).filter(|record| !owned(record.0)).map(
            |&(id, words, [first, second, third])| {
                (id, words, [u8::from(first != 0 && !stocks(id)), second, third])
            },
        ));
        records.extend(self.window_event_rows().filter(|record| !owned(record.0)));
        let known: BTreeSet<i32> = records.iter().map(|record| record.0).collect();
        let fresh = |id: &&i32| !lists.static_ids.contains(id) && !known.contains(id);
        let extra: BTreeSet<i32> = self.open_stock.iter().filter(fresh).copied().collect();
        records.extend(extra.into_iter().map(|id| (id, [id.cast_unsigned(), 0, 0], [0, 0, 1])));
        let mut reply = header(COIN_FAMILY, 9);
        put_i32(&mut reply, records.len() as i32);
        for (id, words, flags) in records {
            put_i32(&mut reply, id);
            words.iter().for_each(|word| put_u32(&mut reply, *word));
            reply.extend(flags);
        }
        reply
    }

    /// C1: in-season event stock (`live_event`) with a coin price that the login lists lack, as
    /// window rows: words (item, category, 0) like the shipped 165/1 rows, every flag clear. The
    /// gamedock Coin Shop asks for the dynamic list when it opens (`CoinStoreHandler:Init` ->
    /// `CoinStoreScreen.OnCoinStoreOpened` `00c003c0` -> 165/8 `00c76d20`), and every 165/9
    /// rebuilds the window's rows (`00c76100` -> `RefreshCoinStoreFilteredRows` `00cf2120`) from
    /// the store nodes whose third flag (`+0xe`) is clear (`RebuildFilteredNodeList` `00cf1ac0`).
    /// So these rows are listed only while their event runs; the 21 L1172 priced in coins were
    /// in no list and at no vendor. (R) A shop no calendar drives (a bare fixture) lists none:
    /// only the session's calendar can say an event runs.
    fn window_event_rows(&self) -> impl Iterator<Item = (i32, [u32; 3], [u8; 3])> + '_ {
        let lists = &self.store_lists;
        let listed = |id: i32| {
            lists.static_ids.contains(&id) || lists.dynamic.iter().any(|record| record.0 == id)
        };
        let driven = self.open_events.is_some();
        (self.catalogue.items.iter())
            .filter(move |item| {
                driven
                    && self.catalogue.seasonal.contains_key(&item.item_id)
                    && item.purchase_price_coins.is_some()
                    && self.in_season(item.item_id)
                    && !listed(item.item_id)
            })
            .map(|item| {
                let id = item.item_id;
                (id, [id.cast_unsigned(), item.category_id.unwrap_or(0), 0], [0, 0, 0])
            })
    }

    /// C1: whether the gamedock Coin Shop window lists `item_id`, so its buy (165/4 with the null
    /// merchant and no record) may be sold without a vendor: a static row, a login dynamic row
    /// with the third flag clear (`00cf1ac0`), or in-season event stock it is sent
    /// (`window_event_rows`). A vendor-only row (third flag set) or a row in no list is not.
    pub fn window_lists(&self, item_id: i32) -> bool {
        let lists = &self.store_lists;
        lists.static_ids.contains(&item_id)
            || (lists.dynamic.iter()).any(|(id, _, flags)| *id == item_id && flags[2] == 0)
            || self.window_event_rows().any(|record| record.0 == item_id)
    }

    /// Build the session's shop when the NPC package provides one. A saved Station Cash wallet
    /// with no Marketplace to spend it in is refused, or the next save would drop it.
    pub fn load(
        inventory_content: &Path,
        npc_content: &Path,
        shop_enabled: bool,
        inventory: Inventory,
        profiles: Option<&crate::profiles::Profiles>,
        wallet: Option<Wallet>,
    ) -> io::Result<Option<Self>> {
        let shop = if shop_enabled {
            let catalogue = StoreCatalogue::load(&npc_content.join("stores.json"))?;
            // (R) Local bag capacity is 100 occupied stacks until an account capacity is wired.
            let mut shop = Self::new(
                inventory_content,
                inventory,
                catalogue,
                100,
                profiles,
                wallet.clone().unwrap_or_default(),
            )?;
            // T229: the earn table ships beside the NPC/store content (the same fallback the
            // Marketplace catalogue uses). A missing file keeps earning off.
            shop.sc_earn = station_cash::EarnTable::load(&npc_content.join("sc_rewards.json"))?;
            Some(shop)
        } else {
            None
        };
        station_cash::require_wallet_owner(&wallet, shop.is_some())?;
        Ok(shop)
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    /// Trade against the player's live inventory: coins and items earned
    /// since the shop was created must not be lost or ignored on a purchase.
    pub fn sync_inventory(&mut self, inventory: Inventory) {
        self.inventory = inventory;
    }

    /// T772: the session's free pet roster slots, set beside `sync_inventory` before a buy.
    pub fn sync_pet_room(&mut self, room: usize) {
        self.pet_room = Some(room);
    }

    pub fn set_member_status(&mut self, is_member: bool) {
        self.is_member = is_member;
    }

    /// T230: read the live Station Cash wallet (the save field is `GameplayState::sc_wallet`).
    pub fn sc_wallet(&self) -> &Wallet {
        &self.sc_wallet
    }

    /// The wallet as it belongs in the save; `None` when a fresh character has no Station Cash.
    pub fn saved_wallet(&self) -> Option<Wallet> {
        (!self.sc_wallet.is_empty()).then(|| self.sc_wallet.clone())
    }

    pub fn sc_catalogue(&self) -> &station_cash::Catalogue {
        &self.sc_catalogue
    }

    /// T229: the earn table read beside the campaign content; `None` keeps earnings off.
    pub fn sc_earn(&self) -> Option<&station_cash::EarnTable> {
        self.sc_earn.as_ref()
    }

    /// T229: adopt the Station Cash wallet from one staged reward transaction. The reward path
    /// is the only writer; the order path still debits the same field.
    pub fn set_sc_wallet(&mut self, wallet: Wallet) {
        self.sc_wallet = wallet;
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
                ..CoinShopOutcome::default()
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
                ..CoinShopOutcome::default()
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
            // The hub passes the shared busy-table answer and the session's lot grant to
            // `handle_marketplace`; a direct service call (tests, older callers) is not inside an
            // activity and has no housing, so a deed order is refused (L046).
            MARKETPLACE_FAMILY => self.handle_marketplace(inner, false, &mut |_, _| None),
            _ => Err(invalid("not a store packet")),
        }
    }

    fn handle_coin_store(&mut self, inner: &[u8]) -> io::Result<CoinShopOutcome> {
        let opcode = read_u16(inner, 2)?;
        match opcode {
            // Client writer 0x00c75ed0 (BuyItem 0x00c77230, called by 0x00c77db0): header,
            // merchant u64, the 165/10 record scalar as the context (`00cf10d0`), definition id,
            // tint, quantity (offset 24). The context is the same for every buy from one open
            // shop and nothing else is per request, so a repeated 165/4 is a second purchase;
            // transport retransmits are already dropped by the SOE layer.
            4 if inner.len() == 28 => self.buy_once(
                read_u32(inner, 12)?,
                read_i32(inner, 16)?,
                read_u32(inner, 24)?,
                read_u64(inner, 4)?,
            ),
            5 if inner.len() == 20 => self.sell_with_merchant(
                read_i32(inner, 12)?,
                read_u32(inner, 16)?,
                read_u64(inner, 4)?,
            ),
            2 => self.item_definitions(inner),
            // F3: the client clears and refills its dynamic list from 165/9; the open merchant's
            // stock goes in, or its Buy stays grey (`dynamic_list_update`).
            8 if inner.len() == 4 => Ok(CoinShopOutcome {
                replies: vec![self.dynamic_list_update(&[])],
                result_code: RESULT_SUCCESS,
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
        self.buy_once(client_transaction_id, item_id, quantity, 0)
    }

    /// T529: why the coin store must not sell this item, or `None` when it may. The item `Type`
    /// is the client definition field `Inventory::presentation` reads after the model and texture
    /// strings (`008e3760`). Type 19 is a ride: the item-use switch `00a1ec00` handles case 0x13
    /// and `00b06db0` owns the mount path (it logs `SpawnMountByItemDefinitionId: item %d`,
    /// notes/Autopilot/T452.md).
    /// Type 16 is a house deed/lot: every recorded deed is Type 16 (notes/Autopilot/L046.md) and
    /// OSFR's 9a8326f forces `ForceDisablePreview` on them after the housing preview soft-lock;
    /// housing is lead-owned. Type 2 is a pet adoption item, which only the species binding can
    /// turn into a pet (`PetsHandler::SendPetSummonByItemId` `00b60110`, 53/32). Mapping type to
    /// policy is (R); the field layout and the type values are pinned by the shipped content.
    ///
    /// A definition that exists but cannot be read is refused here instead of becoming an error.
    /// The reason stays server-side: `buy_once` answers with the existing invalid-item reply.
    pub fn sale_block(&self, item_id: i32) -> Option<&'static str> {
        // B2: event stock out of season leaves the window and the buy alike.
        if !self.in_season(item_id) {
            return Some("event stock is out of season");
        }
        let item_type = match self.inventory.presentation(item_id) {
            Ok(presentation) => presentation.item_type,
            Err(_) => return Some("item definition type is unreadable"),
        };
        match item_type {
            2 => (!self.pet_items.contains(&item_id))
                .then_some("pet item has no pets.json species binding"),
            station_cash::DEED_ITEM_TYPE => Some("house lot (housing preview soft-lock)"),
            19 => Some("rides are Station Cash or earned only"),
            _ => None,
        }
    }

    /// Every coin buy (165/4, the 66/43 vendor buy, `buy`), under the row's stock limit (B2). A
    /// spent stock or share is the invalid-quantity failure, with a chat line first that says so
    /// (the 66/43 path swaps the last reply for its own result).
    fn buy_once(
        &mut self,
        transaction_id: u32,
        item_id: i32,
        quantity: u32,
        merchant_id: u64,
    ) -> io::Result<CoinShopOutcome> {
        self.limited(
            item_id,
            quantity,
            |shop, refusal| {
                let code = match refusal {
                    shop_stock::Refusal::Busy => RESULT_SERVICE_UNAVAILABLE,
                    _ => RESULT_INVALID_QUANTITY,
                };
                let mut out =
                    shop.failure(1, code, transaction_id, item_id, quantity, 0, merchant_id);
                out.replies
                    .insert(0, crate::chat::system(refusal.message()));
                out
            },
            |shop| {
                let out = shop.buy_unlimited(transaction_id, item_id, quantity, merchant_id)?;
                let sold = out.changed && out.result_code == RESULT_SUCCESS;
                Ok((out, sold))
            },
        )
    }

    fn buy_unlimited(
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
        // T529: what the item is matters before any price or quantity work. The reply and the
        // unchanged state are the existing invalid-item failure; nothing is debited or granted.
        if self.sale_block(item_id).is_some() {
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
        // T772: a bought pet item becomes a pet at once, so more than the roster holds is refused
        // like any impossible quantity, before money moves; an unknown room fails closed.
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
        if self.pet_items.contains(&item_id) {
            let Some(room) = self.pet_room else {
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
            if quantity as usize > room {
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
        }
        let Some(item) = self.catalogue.item(item_id) else {
            return Ok(self.failure(
                1,
                RESULT_INVALID_ITEM,
                transaction_id,
                item_id,
                quantity,
                0,
                merchant_id,
            ));
        };
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
        let Some(coins) = self.inventory.state().coins.checked_sub(total_price) else {
            return Ok(self.failure(
                1,
                RESULT_INSUFFICIENT_COINS,
                transaction_id,
                item_id,
                quantity,
                unit_price,
                merchant_id,
            ));
        };
        let mut next_state = self.inventory.state().clone();
        next_state.coins = coins;
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
        if staged.state().bag_stacks() > self.bag_capacity {
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
            granted: granted.granted,
        })
    }

    /// Client-readable 165/6 refusal for a 165/5 sell the NPC layer rejected before the shop saw
    /// it (closed shop, range, merchant). Echoes the request's fields when it is well formed.
    /// Nothing is read from or changed in the inventory; the refusal is not recorded as a sale (R).
    pub fn refuse_sell(&mut self, inner: &[u8], result_code: u32) -> CoinShopOutcome {
        let (merchant, item, quantity) = inner
            .strip_prefix(&[0xa5, 0, 5, 0])
            .filter(|rest| rest.len() == 16)
            .and_then(|rest| Some((le::u64(rest, 0)?, le::i32(rest, 8)?, le::u32(rest, 12)?)))
            .unwrap_or((0, 0, 0));
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
        for chunk in inner[8..].as_chunks::<4>().0 {
            let id = u32::from_le_bytes(*chunk);
            let defined = i32::try_from(id).ok().and_then(|definition_id| {
                Some((
                    definition_id,
                    self.inventory.definition_record(definition_id)?,
                ))
            });
            complete &= defined.is_some();
            if let Some((definition_id, record)) = defined
                && seen.insert(id)
            {
                known.push(id);
                if !self.inventory.definition_was_captured(definition_id) {
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
        let (transaction, context) = inner
            .strip_prefix(&[0xa5, 0, 12, 0])
            .filter(|rest| rest.len() == 8)
            .and_then(|rest| Some((le::u32(rest, 0)?, le::u32(rest, 4)?)))
            .unwrap_or((0, 0));
        buy_back_result(transaction, result_code, context)
    }

    pub fn sell(&mut self, item_instance_id: i32, quantity: u32) -> io::Result<CoinShopOutcome> {
        self.sell_with_merchant(item_instance_id, quantity, 0)
    }

    /// T434: what 165/5 pays for one unit of `definition_id`, or `None` when the shop does not
    /// buy it. The one sale price: a sale records it and buy-back refunds that record. In order:
    /// a `stores.json` no_sale row is never bought; free issuance pays 0, with or without a row;
    /// a `stores.json` sell price is authoritative; otherwise the client's own Worth, when its
    /// Sell tab lists the item (NoSale clear, ResellValue > 0, `00cf08d0`), is paid (R). That
    /// fallback covers the loot, gathered materials and potions `stores.json` never priced.
    pub fn sell_unit_price(&self, definition_id: i32) -> Option<u32> {
        let row = self.catalogue.item(definition_id);
        if row.is_some_and(|row| row.no_sale) {
            return None;
        }
        if self.catalogue.free_items.contains(&definition_id) {
            return Some(0);
        }
        row.and_then(|row| row.sell_price_coins).or_else(|| {
            let (no_sale, worth) = self.inventory.sale_flags(definition_id)?;
            u32::try_from(worth)
                .ok()
                .filter(|worth| !no_sale && *worth > 0)
        })
    }

    /// T434: the Worth a 35/37 update must carry so the client's Sell tab shows what 165/5 pays
    /// for `definition_id`: the paid price where the definition's ResellValue differs, or 0 to
    /// drop a listed item the shop pays nothing for off the tab (`00cf08d0`). `None` when the
    /// client's definition already agrees. The client cache keeps the update for the session
    /// (`00a3c290` overwrites by id), so it is sent on every shop open (R).
    pub fn sell_worth_override(&self, definition_id: i32) -> Option<u32> {
        let (no_sale, worth) = self.inventory.sale_flags(definition_id)?;
        match self.sell_unit_price(definition_id) {
            Some(paid) if paid > 0 => (i64::from(paid) != i64::from(worth)).then_some(paid),
            _ => (!no_sale && worth > 0).then_some(0),
        }
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
        let Some(unit_price) = self.sell_unit_price(owned.definition_id) else {
            // (R) An item the store has no row for is invalid; a row it refuses is unavailable.
            let result = match self.catalogue.item(owned.definition_id) {
                Some(_) => RESULT_SERVICE_UNAVAILABLE,
                None => RESULT_INVALID_ITEM,
            };
            return Ok(self.failure(
                2,
                result,
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
            ..CoinShopOutcome::default()
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
        let Some(coins) = self.inventory.state().coins.checked_sub(total_price) else {
            return Ok(buy_back_result(
                sale_transaction_id,
                RESULT_FAILURE,
                client_context,
            ));
        };
        let mut next_state = self.inventory.state().clone();
        next_state.coins = coins;
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
        if staged.state().bag_stacks() > self.bag_capacity {
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
            granted: restored.granted,
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
            ..CoinShopOutcome::default()
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

    /// The Marketplace handshake (66/24, 66/33) and the store rows (66/7/6/9/5) when both the
    /// Station Cash package and the Marketplace handshake are enabled; empty otherwise. The
    /// native store screen asks for its wallet (66/10, `00b43bc0`/`RequestWalletInfo`) and never
    /// for the 66/12 status, so the 66/10 reply carries this set; 66/12 answers a client that
    /// does ask.
    pub fn marketplace_packets(&self) -> Vec<Vec<u8>> {
        if self.marketplace_enabled() {
            station_cash::catalogue_packets(&self.sc_catalogue)
        } else {
            Vec::new()
        }
    }

    /// Whether the Marketplace lists `bundle` (an offer may advertise only what can be bought).
    pub fn lists_bundle(&self, bundle: u32) -> bool {
        self.marketplace_enabled() && self.sc_catalogue.listing(bundle).is_some()
    }

    /// F3 (rc-0136 m-booster): a new character had no Station Cash for a booster until activities
    /// paid some. The earn table's welcome grant is paid once per character, at the first wallet
    /// read, and the wallet remembers it (`Wallet::welcomed`). Returns whether it was paid now.
    fn welcome(&mut self) -> bool {
        let config = self.sc_catalogue.config();
        let grant = (self.sc_earn.as_ref())
            .filter(|table| table.enabled)
            .map_or(0, |table| table.welcome_grant);
        if grant == 0 || !config.enabled || self.sc_wallet.welcomed {
            return false;
        }
        self.sc_wallet.welcomed = true;
        let balance = self.sc_wallet.balance.saturating_add(grant);
        self.sc_wallet.balance = balance.min(config.max_balance);
        true
    }

    fn marketplace_enabled(&self) -> bool {
        self.sc_catalogue.config().enabled && self.catalogue.marketplace.enabled_on_handshake
    }

    /// Answer gateway Marketplace requests. A Station Cash order debits the wallet and grants the
    /// ride in one staged commit; wallet funding, membership and paid orders stay off (R).
    /// `busy` is the shared busy table's answer and only gates the order request. `grant_lot`
    /// delivers a paid lot deed, and a preview asks it without committing (L046).
    pub fn handle_marketplace(
        &mut self,
        inner: &[u8],
        busy: bool,
        grant_lot: station_cash::LotGrant,
    ) -> io::Result<CoinShopOutcome> {
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
        let mut changed = false;
        let mut transaction_id = 0;
        let mut granted = Vec::new();
        let (result_code, replies) = match opcode {
            1 => {
                if !self.sc_catalogue.config().enabled {
                    // T453: echo the request's tracking id; a body that does not decode gets 0.
                    let tracking_id = station_cash::decode_order(inner, true)
                        .map_or(0, |order| order.tracking_id);
                    let reply =
                        station_cash::preview_packet(tracking_id, MARKETPLACE_RESULT_DISABLED, 0);
                    (MARKETPLACE_RESULT_DISABLED, vec![reply])
                } else {
                    let order = station_cash::decode_order(inner, true)?;
                    transaction_id = order.tracking_id;
                    let outcome = station_cash::preview(
                        &self.sc_catalogue,
                        &self.sc_wallet,
                        self.inventory.state(),
                        &order,
                        grant_lot,
                    );
                    // B2: a spent stock or share fails the preview too, and says why.
                    let bundle = order.details.first().map_or(0, |detail| detail.bundle_id);
                    let item = i32::try_from(bundle).unwrap_or(0);
                    let refusal =
                        (self.stock_refusal(item, 1)).unwrap_or(Some(shop_stock::Refusal::Busy));
                    match refusal {
                        Some(refusal) if outcome.result_code == station_cash::ORDER_SUCCESS => (
                            station_cash::ORDER_FAILURE,
                            vec![
                                crate::chat::system(refusal.message()),
                                station_cash::preview_packet(
                                    order.tracking_id,
                                    station_cash::ORDER_FAILURE,
                                    0,
                                ),
                            ],
                        ),
                        _ => (outcome.result_code, outcome.replies),
                    }
                }
            }
            3 => {
                if !self.sc_catalogue.config().enabled {
                    let tracking_id = station_cash::decode_order(inner, false)
                        .map_or(0, |order| order.tracking_id);
                    let reply = station_cash::order_packet(
                        tracking_id,
                        MARKETPLACE_RESULT_DISABLED,
                        &self.catalogue.marketplace.not_available_message,
                        0,
                    );
                    (MARKETPLACE_RESULT_DISABLED, vec![reply])
                } else {
                    let order = station_cash::decode_order(inner, false)?;
                    transaction_id = order.tracking_id;
                    // B2: a spent stock or share is the order's failure, its text saying why.
                    let outcome = match self.limited_order(&order, |shop| {
                        station_cash::place_order(
                            &shop.sc_catalogue,
                            &mut shop.sc_wallet,
                            &mut shop.inventory,
                            &order,
                            shop.bag_capacity,
                            busy,
                            grant_lot,
                        )
                    })? {
                        Ok(outcome) => outcome,
                        Err(refusal) => station_cash::OrderOutcome {
                            result_code: station_cash::ORDER_FAILURE,
                            replies: vec![
                                crate::chat::system(refusal.message()),
                                station_cash::order_packet(
                                    order.tracking_id,
                                    station_cash::ORDER_FAILURE,
                                    refusal.message(),
                                    0,
                                ),
                            ],
                            ..Default::default()
                        },
                    };
                    changed = outcome.changed;
                    granted = outcome.granted;
                    let mut replies = outcome.replies;
                    if changed && self.marketplace_enabled() {
                        // T484: grey the bought ride out without reopening the store.
                        replies.push(station_cash::requirements_packet(
                            &self.sc_catalogue,
                            self.inventory.state(),
                        ));
                    }
                    (outcome.result_code, replies)
                }
            }
            10 => {
                changed = self.welcome();
                let mut replies = vec![station_cash::wallet_packet(self.sc_wallet.balance)];
                // The native store screen sends this wallet request when it opens
                // (`00b43bc0`, Lua `RequestWalletInfo`) and never the 66/12 status
                // request, so the reply carries the handshake and the catalogue; the
                // client dispatches 66/x by subtype (`00b48a10`).
                replies.extend(self.marketplace_packets());
                (1, replies)
            }
            12 => {
                // Answered for a client that does ask for the server status; the shipped
                // screen uses 66/10 instead.
                let enabled = self.catalogue.marketplace.enabled_on_handshake;
                let mut replies = vec![station_cash::bool_packet(13, enabled)];
                replies.extend(self.marketplace_packets());
                (1, replies)
            }
            14 => (1, vec![empty_product_list(15)]),
            // Status 1 with an empty list is the client's own "no data was available" path
            // (`00b460c0`, `00b46a50`, `00b46200`, `00b45ba0`).
            16 => (1, vec![empty_list_with_text(17)]),
            18 => (1, vec![empty_product_list(19)]),
            20 => (1, vec![empty_list_with_text(21)]),
            22 => (1, vec![empty_list_with_text(23)]),
            25 => (1, vec![account_info_packet()]),
            // T380: 66/38 is the store screen's `Opening` request (`00c047a0`). Its 66/39 reply
            // (`00b3e490`) refreshes StoreItemRoot, and that refresh is what fills the bundle
            // list: StoreItems only copies the store once the commerce processor is ready
            // (`0099aa60`), one client tick after a 66/5 arrives (`00b335f0`). T484: the list
            // carries each listing's owned flag.
            38 if self.marketplace_enabled() => (
                1,
                vec![station_cash::requirements_packet(
                    &self.sc_catalogue,
                    self.inventory.state(),
                )],
            ),
            // T744: 66/27 is the store screen's request for bundle content definitions. The client
            // asked for these bundles because its local item-definition map lacks their items; the
            // 35/37 batch must arrive before 66/28, because the 66/28 handler (`00b40050`) clears a
            // bundle's pending entry only once every content item resolves. A reply is driven by the
            // server's listing and catalogue, never by the client's item ids (hostile input note in
            // T744.md).
            27 if self.marketplace_enabled() => {
                let requests = station_cash::decode_content_request(inner)?;
                let store_id = self.sc_catalogue.config().store_id;
                let mut seen = BTreeSet::new();
                let mut confirmed = BTreeSet::new();
                let mut records = Vec::new();
                for request in &requests {
                    if request.store_id != store_id {
                        continue;
                    }
                    let Some(listing) = self.sc_catalogue.listing(request.bundle_id) else {
                        continue;
                    };
                    let Some(record) = self.inventory.definition_record(listing.item_id) else {
                        continue;
                    };
                    if seen.insert(listing.item_id) {
                        records.push(record.to_vec());
                    }
                    confirmed.insert((request.store_id, request.bundle_id));
                }
                let mut replies = Vec::new();
                if !records.is_empty() {
                    // The canonical 35/37 encoder writes the startup catalogue's framing.
                    replies.push(
                        crate::inventory_v2_fixes::encode_definition_batch_v2(&records, &[])
                            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
                    );
                }
                // A count-zero 66/28 (8 bytes) is rejected by the client's 12-byte base parse
                // (`sub_B40050` -> `sub_B2DFD0`), so an empty set gets no reply at all.
                let confirmed = confirmed.into_iter().collect::<Vec<_>>();
                if !confirmed.is_empty() {
                    replies.push(station_cash::content_response_packet(&confirmed));
                }
                (1, replies)
            }
            // Bundle content with the Marketplace off, and 66/38 with it off: no reply.
            27 | 38 => return Ok(CoinShopOutcome::default()),
            // NpcMerchantV2 buy reply: every result other than 1 is a client-visible refusal.
            43 => (
                MARKETPLACE_RESULT_DISABLED,
                vec![merchant_buy_result_packet(false)],
            ),
            _ => return Ok(CoinShopOutcome::default()),
        };
        Ok(CoinShopOutcome {
            result_code,
            transaction_id,
            replies,
            changed,
            granted,
        })
    }
}

pub(crate) fn transaction_complete(
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
        ..CoinShopOutcome::default()
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

fn account_info_packet() -> Vec<u8> {
    // 00b2e080 reads status, two length-prefixed strings, and a final boolean.
    let mut bytes = header(MARKETPLACE_FAMILY, 26);
    put_u32(&mut bytes, 1);
    put_text(&mut bytes, "US");
    put_text(&mut bytes, "USD");
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

/// Observed client requests (62 native traces): 66/10, 66/18 and 66/38 (`00c03920` builds only
/// the header) are the bare header; 66/16, 20 and 25 add an `i32` length and that many locale
/// bytes (`en_US`). Rows with no observed sender are only length-bounded (R).
fn marketplace_request_is_well_formed(opcode: u16, inner: &[u8]) -> bool {
    match (opcode, le::i32(inner, 4)) {
        // The order body is parsed strictly by `station_cash::decode_order`; 4 KiB bounds the
        // six small strings, one detail and the preview's extra string before allocation.
        (1 | 3, _) => inner.len() <= station_cash::MAX_ORDER_PACKET,
        (10 | 18 | 38, _) => inner.len() == 4,
        (16 | 20 | 25, Some(length)) => {
            usize::try_from(length).is_ok_and(|n| n <= 32 && inner.len() == 8 + n)
        }
        (16 | 20 | 25, None) => false,
        _ => inner.len() <= MAX_MARKETPLACE_REQUEST,
    }
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
        ..CoinShopOutcome::default()
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
    le::u16(bytes, offset).ok_or_else(|| invalid("truncated store field"))
}

fn read_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    le::u32(bytes, offset).ok_or_else(|| invalid("truncated store field"))
}

fn read_i32(bytes: &[u8], offset: usize) -> io::Result<i32> {
    le::i32(bytes, offset).ok_or_else(|| invalid("truncated store field"))
}

fn read_u64(bytes: &[u8], offset: usize) -> io::Result<u64> {
    le::u64(bytes, offset).ok_or_else(|| invalid("truncated store field"))
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
