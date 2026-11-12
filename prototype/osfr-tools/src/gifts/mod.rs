//! Gifts and codes (gifts-codes piece, owner 2026-11-11): the original's free items, from the
//! website profile page's ad offers and from published codes, handed out through the client's
//! own Claim window, and the GM panel's way to give an event winner a gift or a code.
//!
//! - **Catalog** (`data/content/gift_catalog.json`): one row per gift with the fields the Studio
//!   edits (`repeatable`/`repeat_hours`/`live_event` as in the event-repeat contract, `tradeable`,
//!   `enabled`). A missing file is an empty catalog; a malformed one is an error naming it.
//! - **Gift box**: per account, `<account store dir>/gifts/<account id>.json`, beside the realm's
//!   mail. Pending gifts wait there until a character claims one; `taken` records when each
//!   catalog row was last claimed, so a once-only row never comes back and a repeating one comes
//!   back after its hours. Every write is a locked read-modify-write through the atomic
//!   `character_store::replace_bytes`, as mail's are.
//! - **Standing offers**: the profile rows. When enabled (the catalog, or the panel's toggle in
//!   `gifts/offers.json`) and their event runs (`EventCalendar::is_running`), every account finds
//!   them in its Claim window at login and when the event starts.
//! - **Codes**: `realm redeem <code>` takes a catalog row's original code (once per account) or
//!   an admin code (`redeem_codes`, with its uses, expiry and binding) and puts the gift in the
//!   Claim window. The client's own Redeem Code window does not reach the gateway: it posts the
//!   code over HTTP to the commerce service (`RedeemCodeRequest::Validate` 0x00f7a7e0, sent by
//!   `DispatchUramRequestByType` 0x00d6d830 from `Ui.RedeemCode` 0x00c02230).
//!
//! The Claim window's packets are in `claim`; the panel's actions in `admin`.

pub mod admin;
pub mod claim;

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, PoisonError},
    time::{SystemTime, UNIX_EPOCH},
};

pub const CATALOG_FILE: &str = "gift_catalog.json";
const SCHEMA_VERSION: u32 = 1;
/// (R) Bounds the catalog read into memory (the shipped one is about 80 KB).
const MAX_CATALOG_BYTES: u64 = 4 << 20;
/// (R) A gift box at its pending cap is about 60 KB; this bounds a corrupt or foreign file.
const MAX_BOX_BYTES: u64 = 1 << 20;
/// (R) Pending gifts one account may hold; a send past it is refused.
pub const MAX_PENDING: usize = 200;
/// (R) Item rows one gift may carry (`redeem_codes::MAX_ITEMS`, a code's bound).
pub const MAX_GIFT_ITEMS: usize = crate::redeem_codes::MAX_ITEMS;
/// (R) A staff note on a gift.
pub const MAX_NOTE_CHARS: usize = 200;
/// (R) Coins one gift may pay (mail's attachment cap).
pub const MAX_GIFT_COINS: u32 = 1_000_000;
/// (R) The top half of a gift's claim id: "GIFT", clear of the character claims' sha-derived
/// ids (`progression_extras::achievement_claim_id`) and of every id the client invents.
pub const CLAIM_TAG: u64 = 0x4749_4654 << 32;

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// One item a gift gives. `name_id` is the item row's client NameId, pinned when the row was
/// resolved by name (`tests/gift_catalog.rs` checks it against the item table).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GiftItem {
    pub item_id: i32,
    pub count: u32,
    pub name_id: i32,
}

/// Where a catalog row came from: the profile page's offers, a published code, a promotion or
/// tie-in with no code, or a monthly member pack. Every row but a code is a standing offer
/// (owner 2026-11-11: everything stays available, retired items included).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Profile,
    Code,
    Promotional,
    Member,
}

/// Who a once-only row is once for.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Per {
    Account,
    Character,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Gift {
    pub key: String,
    pub name: String,
    pub items: Vec<GiftItem>,
    pub source: Source,
    pub code: Option<String>,
    pub per: Per,
    pub repeatable: bool,
    /// Hours from a claim until the row is offered again; required when `repeatable`.
    pub repeat_hours: Option<u32>,
    /// The `event_calendar.json` key the row runs during, or none.
    pub live_event: Option<String>,
    pub tradeable: bool,
    pub enabled: bool,
    /// Coins the gift pays with its items (a member pack's coins).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub coins: u32,
    /// Claims the whole realm may make per event run (per all time for a row with no event);
    /// `None` is unlimited. B2's shop field, so the Studio shares one control.
    #[serde(default)]
    pub stock_per_event: Option<u32>,
    /// Claims one player (`per`) may make per event run (all time with no event); `None` is
    /// unlimited beyond `repeatable`.
    #[serde(default)]
    pub per_player: Option<u32>,
    /// Rows with the same group share one counter for both limits (the colours and genders of
    /// one LIMITED EDITION outfit); `None` counts the row on its own. B2's shop field.
    #[serde(default)]
    pub stock_group: Option<String>,
    pub note: String,
    pub evidence: String,
}

impl Gift {
    pub fn items(&self) -> Vec<(i32, u32)> {
        self.items.iter().map(|item| (item.item_id, item.count)).collect()
    }

    /// The run a claim counts against: the event's current run, or all time.
    fn run(&self, events: &Events) -> String {
        (self.live_event.as_deref())
            .and_then(|key| events.get(key))
            .cloned()
            .unwrap_or_else(|| "always".into())
    }

    /// The `taken` key a claim of this row records.
    /// The counter a limit counts on: the row's stock group, else the row.
    fn counter(&self) -> &str {
        self.stock_group.as_deref().unwrap_or(&self.key)
    }

    /// The per-player counter key (`counter@guid` for a per-character row).
    fn limit_key(&self, character: u64) -> String {
        match self.per {
            Per::Account => self.counter().to_owned(),
            Per::Character => format!("{}@{character}", self.counter()),
        }
    }

    fn ledger(&self, character: u64) -> String {
        match self.per {
            Per::Account => self.key.clone(),
            Per::Character => format!("{}@{character}", self.key),
        }
    }
}

/// A source item that cannot be given yet, with the reason: missing work (an item row to build,
/// a source to find), never "not applicable".
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Unresolved {
    pub name: String,
    pub source: Source,
    pub code: Option<String>,
    /// The client locale row of its name, when the client has one (the item row's NameId to be).
    #[serde(default)]
    pub name_id: Option<i32>,
    #[serde(default)]
    pub live_event: Option<String>,
    pub reason: String,
    pub evidence: String,
}

/// The calendar events running now: event key to its current run (the local date it ends, or
/// `held` while a hand toggle holds it).
pub type Events = BTreeMap<String, String>;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub sources: serde_json::Value,
    pub gifts: Vec<Gift>,
    #[serde(default)]
    pub unresolved: Vec<Unresolved>,
}

fn invalid(text: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, text.into())
}

fn read_bounded(path: &Path, max: u64) -> io::Result<Option<Vec<u8>>> {
    match fs::metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
        Ok(meta) if !meta.is_file() || meta.len() > max => Err(invalid(format!(
            "{} is not a regular file of at most {max} bytes",
            path.display()
        ))),
        Ok(_) => fs::read(path).map(Some),
    }
}

impl Catalog {
    /// `gift_catalog.json` in `content_dir`; a missing file is an empty catalog.
    // ponytail: read on use (a list request, a redeem, a login sync, a panel call), no cache:
    // the file is small and Studio edits then need no reload. Cache by mtime if a profile shows it.
    pub fn load(content_dir: &Path) -> io::Result<Self> {
        let path = content_dir.join(CATALOG_FILE);
        let Some(bytes) = read_bounded(&path, MAX_CATALOG_BYTES)? else {
            return Ok(Self {
                schema_version: SCHEMA_VERSION,
                ..Self::default()
            });
        };
        let catalog: Self = serde_json::from_slice(&bytes)
            .map_err(|error| invalid(format!("{}: {error}", path.display())))?;
        catalog
            .validate()
            .map_err(|error| invalid(format!("{}: {error}", path.display())))?;
        Ok(catalog)
    }

    /// Unique keys and codes, item rows inside their bounds, and the contract's rule that a
    /// repeating row names its hours.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION {
            return Err("needs schema_version 1".into());
        }
        let mut keys = BTreeSet::new();
        let mut codes = BTreeSet::new();
        for gift in &self.gifts {
            let key = &gift.key;
            if key.is_empty() || key.len() > 64 || !keys.insert(key.as_str()) {
                return Err(format!("gift key {key:?} is empty, too long or repeated"));
            }
            if let Some(code) = &gift.code
                && (!crate::redeem_codes::valid_code(code) || !codes.insert(normalize(code)))
            {
                return Err(format!("gift {key}: code {code:?} is malformed or repeated"));
            }
            if gift.items.is_empty()
                || gift.items.len() > MAX_GIFT_ITEMS
                || gift.items.iter().any(|item| item.item_id <= 0 || item.count == 0)
            {
                return Err(format!("gift {key}: 1 to {MAX_GIFT_ITEMS} items, each id and count positive"));
            }
            match (gift.repeatable, gift.repeat_hours) {
                (true, Some(hours)) if hours >= 1 => {}
                (false, None) => {}
                _ => return Err(format!("gift {key}: repeat_hours is required (at least 1) exactly when repeatable")),
            }
            if gift.coins > MAX_GIFT_COINS
                || gift.stock_per_event == Some(0)
                || gift.stock_group.as_deref().is_some_and(|group| group.is_empty() || group.len() > 64)
                || gift.per_player == Some(0)
            {
                return Err(format!(
                    "gift {key}: coins at most {MAX_GIFT_COINS}; stock_per_event and per_player at least 1 when set"
                ));
            }
            if gift.live_event.as_deref().is_some_and(str::is_empty) {
                return Err(format!("gift {key}: live_event is an event key or absent"));
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Gift> {
        self.gifts.iter().find(|gift| gift.key == key)
    }

    pub fn by_code(&self, code: &str) -> Option<&Gift> {
        let code = normalize(code);
        self.gifts
            .iter()
            .find(|gift| gift.code.as_deref().is_some_and(|own| normalize(own) == code))
    }
}

pub fn normalize(code: &str) -> String {
    crate::redeem_codes::normalize(code)
}

/// How a pending gift arrived.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Via {
    /// A standing offer: listed only while its row is enabled and its event runs.
    Offer,
    /// A staff member sent it from the panel.
    Staff,
    /// A redeemed code.
    Code,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Pending {
    pub id: u32,
    /// The catalog row, when the gift is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub name: String,
    pub items: Vec<(i32, u32)>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub coins: u32,
    pub from: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// Unix seconds.
    pub sent: u64,
    pub via: Via,
    /// Set for a per-character row: only that character sees it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<u64>,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

impl Pending {
    pub fn claim_id(&self) -> u64 {
        CLAIM_TAG | u64::from(self.id)
    }
}

/// One account's gifts.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GiftBox {
    pub schema_version: u32,
    pub next_id: u32,
    pub pending: Vec<Pending>,
    /// Catalog key (`key@guid` for a per-character row) to the unix second it was last claimed.
    #[serde(default)]
    pub taken: BTreeMap<String, u64>,
    /// The same keys to this player's claims in the row's current run (`per_player`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub claims: BTreeMap<String, RunCount>,
}

impl Default for GiftBox {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            next_id: 1,
            pending: Vec::new(),
            taken: BTreeMap::new(),
            claims: BTreeMap::new(),
        }
    }
}

/// Claims of one row in one run: a player's (`GiftBox::claims`) or the realm's (`Stock`).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunCount {
    pub run: String,
    pub count: u32,
}

fn in_run(counts: &BTreeMap<String, RunCount>, key: &str, run: &str) -> u32 {
    counts.get(key).filter(|c| c.run == run).map_or(0, |c| c.count)
}

fn bump(counts: &mut BTreeMap<String, RunCount>, key: &str, run: &str) {
    let count = in_run(counts, key, run).saturating_add(1);
    counts.insert(key.to_owned(), RunCount { run: run.to_owned(), count });
}

/// The realm's claims of its limited rows (`gifts/stock.json`), against `stock_per_event`.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Stock {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub taken: BTreeMap<String, RunCount>,
}

impl Stock {
    pub fn path(store: &Path) -> PathBuf {
        gifts_dir(store).join("stock.json")
    }

    /// The counts as stored; a missing file is none. An unreadable one is an error, so a
    /// limited row is never given past its stock because its counter could not be read.
    pub fn read(store: &Path) -> io::Result<Self> {
        match read_bounded(&Self::path(store), MAX_BOX_BYTES)? {
            None => Ok(Self::default()),
            Some(bytes) => serde_json::from_slice(&bytes).map_err(|error| invalid(error.to_string())),
        }
    }
}

/// A refusal sentence (no trailing stop) or a storage failure.
#[derive(Debug)]
pub enum GiftError {
    Refused(String),
    Io(io::Error),
}

impl fmt::Display for GiftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused(text) => f.write_str(text),
            Self::Io(error) => write!(f, "gift storage failed ({error})"),
        }
    }
}

impl std::error::Error for GiftError {}

impl From<io::Error> for GiftError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub(crate) fn refused(text: impl Into<String>) -> GiftError {
    GiftError::Refused(text.into())
}

/// The realm's gift folder, beside its account store.
pub fn gifts_dir(account_store: &Path) -> PathBuf {
    account_store.with_file_name("gifts")
}

pub fn box_path(account_store: &Path, account: u64) -> PathBuf {
    gifts_dir(account_store).join(format!("{account}.json"))
}

/// The admin codes file the gateway also hands `redeem_codes::set_store`.
pub fn codes_path(account_store: &Path) -> PathBuf {
    account_store.with_file_name("redeem-codes.json")
}

// ponytail: one process lock, as mail's: only the gateway writes gift files.
static LOCK: Mutex<()> = Mutex::new(());

fn guard() -> MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A gift box as stored; a missing file is an empty box.
pub fn load(path: &Path) -> io::Result<GiftBox> {
    let Some(bytes) = read_bounded(path, MAX_BOX_BYTES)? else {
        return Ok(GiftBox::default());
    };
    let gifts: GiftBox = serde_json::from_slice(&bytes).map_err(|error| invalid(error.to_string()))?;
    if gifts.schema_version != SCHEMA_VERSION || gifts.pending.len() > MAX_PENDING {
        return Err(invalid("unsupported or oversized gift box"));
    }
    Ok(gifts)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(value).map_err(io::Error::other)? + "\n";
    crate::character_store::replace_bytes(path, text.as_bytes())
}

/// Read, change and atomically replace one gift box under the gift lock. A refusal or a failed
/// write leaves the file as it was.
pub fn update<T>(
    path: &Path,
    change: impl FnOnce(&mut GiftBox) -> Result<T, GiftError>,
) -> Result<T, GiftError> {
    let _guard = guard();
    let mut gifts = load(path)?;
    let result = change(&mut gifts)?;
    write_json(path, &gifts)?;
    Ok(result)
}

/// What a sender puts in a box; the box assigns the id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parcel {
    pub key: Option<String>,
    pub name: String,
    pub items: Vec<(i32, u32)>,
    pub coins: u32,
    pub from: String,
    pub note: String,
    pub via: Via,
    pub character: Option<u64>,
}

impl Parcel {
    /// A catalog row as a parcel.
    pub fn of(gift: &Gift, from: &str, via: Via, character: u64) -> Self {
        Self {
            key: Some(gift.key.clone()),
            name: gift.name.clone(),
            items: gift.items(),
            coins: gift.coins,
            from: from.to_owned(),
            note: String::new(),
            via,
            character: (gift.per == Per::Character).then_some(character),
        }
    }
}

fn push(gifts: &mut GiftBox, parcel: Parcel, now: u64) -> Result<u32, GiftError> {
    if gifts.pending.len() >= MAX_PENDING {
        return Err(refused("that account's Claim window is full"));
    }
    let id = gifts.next_id;
    if id == 0 || id == u32::MAX {
        return Err(refused("that account has no gift ids left"));
    }
    gifts.next_id = id + 1;
    gifts.pending.push(Pending {
        id,
        key: parcel.key,
        name: parcel.name,
        items: parcel.items,
        coins: parcel.coins,
        from: parcel.from,
        note: parcel.note,
        sent: now,
        via: parcel.via,
        character: parcel.character,
    });
    Ok(id)
}

/// Put one gift in an account's box (a staff send, an admin code), and tell the tick so an
/// online player hears of it. Returns the gift id.
pub fn deliver(store: &Path, account: u64, parcel: Parcel) -> Result<u32, GiftError> {
    // A Claim window row shows its first item, so a gift carries at least one.
    if parcel.items.is_empty() || parcel.items.len() > MAX_GIFT_ITEMS {
        return Err(refused(format!("a gift carries 1 to {MAX_GIFT_ITEMS} items")));
    }
    let id = update(&box_path(store, account), |gifts| push(gifts, parcel, unix_now()))?;
    mark(&DELIVERED, store, account);
    Ok(id)
}

/// The panel's offer toggles (`gifts/offers.json`): catalog key to enabled, over the catalog.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Toggles {
    pub schema_version: u32,
    pub enabled: BTreeMap<String, bool>,
}

impl Toggles {
    pub fn path(store: &Path) -> PathBuf {
        gifts_dir(store).join("offers.json")
    }

    /// The toggles as stored; a missing or unreadable file is none (the catalog decides).
    pub fn load(store: &Path) -> Self {
        read_bounded(&Self::path(store), MAX_BOX_BYTES)
            .ok()
            .flatten()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn enabled(&self, gift: &Gift) -> bool {
        self.enabled.get(&gift.key).copied().unwrap_or(gift.enabled)
    }

    /// Set one row's toggle; returns the effective value before.
    pub fn set(store: &Path, gift: &Gift, enabled: bool) -> io::Result<bool> {
        let _guard = guard();
        let mut toggles = Self::load(store);
        let before = toggles.enabled(gift);
        toggles.schema_version = SCHEMA_VERSION;
        toggles.enabled.insert(gift.key.clone(), enabled);
        write_json(&Self::path(store), &toggles)?;
        Ok(before)
    }
}

/// What decides whether a row shows now: the catalog, the toggles, the running events and the
/// realm's limited stock.
pub struct Rules<'a> {
    pub catalog: &'a Catalog,
    pub toggles: &'a Toggles,
    pub events: &'a Events,
    pub stock: &'a Stock,
}

impl Rules<'_> {
    /// A row is live while enabled, while its event (if any) runs, and while its stock lasts.
    pub fn live(&self, gift: &Gift) -> bool {
        self.toggles.enabled(gift)
            && (gift.live_event.as_deref()).is_none_or(|key| self.events.contains_key(key))
            && (gift.stock_per_event)
                .is_none_or(|cap| in_run(&self.stock.taken, gift.counter(), &gift.run(self.events)) < cap)
    }

    /// Whether `character` (its account, for a per-account row) may claim `gift` again this run.
    fn player_left(&self, gift: &Gift, gifts: &GiftBox, character: u64) -> bool {
        let run = gift.run(self.events);
        (gift.per_player).is_none_or(|cap| in_run(&gifts.claims, &gift.limit_key(character), &run) < cap)
    }

    /// Whether `pending` is in `character`'s Claim window now: its own character's, and a
    /// standing offer only while its row is live.
    pub fn shows(&self, pending: &Pending, character: u64) -> bool {
        pending.character.is_none_or(|own| own == character)
            && (pending.via != Via::Offer
                || pending
                    .key
                    .as_deref()
                    .and_then(|key| self.catalog.get(key))
                    .is_some_and(|gift| self.live(gift)))
    }

    /// Whether standing offer `gift` is due in `gifts` for `character` at `now`: live, not
    /// already waiting, and never claimed, or (repeating) claimed at least its hours ago.
    pub fn offer_due(&self, gift: &Gift, gifts: &GiftBox, character: u64, now: u64) -> bool {
        let waiting = gifts.pending.iter().any(|pending| {
            pending.via == Via::Offer
                && pending.key.as_deref() == Some(gift.key.as_str())
                && pending.character.is_none_or(|own| own == character)
        });
        let again = |last: &u64| {
            gift.repeatable
                && gift
                    .repeat_hours
                    .is_some_and(|hours| now >= last.saturating_add(u64::from(hours) * 3600))
        };
        gift.source != Source::Code
            && self.live(gift)
            && !waiting
            && self.player_left(gift, gifts, character)
            && gifts.taken.get(&gift.ledger(character)).is_none_or(again)
    }
}

/// Add every due standing offer to `account`'s box; returns how many were added.
pub fn sync_offers(
    store: &Path,
    account: u64,
    character: u64,
    rules: &Rules,
    now: u64,
) -> Result<usize, GiftError> {
    let path = box_path(store, account);
    // A read first, so a login with nothing due writes no file.
    let current = load(&path)?;
    if !rules
        .catalog
        .gifts
        .iter()
        .any(|gift| rules.offer_due(gift, &current, character, now))
    {
        return Ok(0);
    }
    update(&path, |gifts| {
        let due: Vec<&Gift> = (rules.catalog.gifts.iter())
            .filter(|gift| rules.offer_due(gift, gifts, character, now))
            .collect();
        for gift in &due {
            push(gifts, Parcel::of(gift, "Free Realms", Via::Offer, character), now)?;
        }
        Ok(due.len())
    })
}

/// A catalog row's original code, once per account (or character): refused while one waits or
/// once claimed (a repeating row again after its hours).
pub fn redeem_classic(
    store: &Path,
    account: u64,
    character: u64,
    gift: &Gift,
    rules: &Rules,
) -> Result<u32, GiftError> {
    if !rules.live(gift) {
        return Err(refused("That code is not active right now."));
    }
    let now = unix_now();
    let id = update(&box_path(store, account), |gifts| {
        let waiting = gifts
            .pending
            .iter()
            .any(|pending| pending.key.as_deref() == Some(gift.key.as_str()) && pending.via == Via::Code);
        let last = gifts.taken.get(&gift.ledger(character)).copied();
        let again = last.is_none_or(|last| {
            gift.repeatable
                && gift
                    .repeat_hours
                    .is_some_and(|hours| now >= last.saturating_add(u64::from(hours) * 3600))
        });
        if waiting || !again {
            return Err(refused("You already redeemed that code."));
        }
        push(gifts, Parcel::of(gift, "Free Realms", Via::Code, character), now)
    })?;
    mark(&DELIVERED, store, account);
    Ok(id)
}

/// Take pending gifts `ids` out of `account`'s box for `character` (the caller has granted them
/// on a trial copy and commits it only after this succeeds), recording each catalog row's claim
/// against the player's `per_player` and the realm's `stock_per_event`. Staff sends are outside
/// both limits. One lock covers the box and the realm's stock; the stock is written first, so a
/// failed box write can only over-count a claim, never give one past the stock.
pub fn take(
    store: &Path,
    account: u64,
    ids: &[u32],
    character: u64,
    rules: &Rules,
    now: u64,
) -> Result<Vec<Pending>, GiftError> {
    let path = box_path(store, account);
    let _guard = guard();
    let mut gifts = load(&path)?;
    let mut stock = Stock::read(store)?;
    let mut taken = Vec::with_capacity(ids.len());
    for id in ids {
        let index = (gifts.pending.iter())
            .position(|pending| pending.id == *id && rules.shows(pending, character))
            .ok_or_else(|| refused("that gift is no longer waiting"))?;
        taken.push(gifts.pending.remove(index));
    }
    let mut limited = false;
    for pending in taken.iter().filter(|pending| pending.via != Via::Staff) {
        let Some(gift) = pending.key.as_deref().and_then(|key| rules.catalog.get(key)) else {
            continue;
        };
        let run = gift.run(rules.events);
        if !rules.player_left(gift, &gifts, character) {
            return Err(refused("you have claimed that gift as many times as allowed"));
        }
        if let Some(cap) = gift.stock_per_event {
            if in_run(&stock.taken, gift.counter(), &run) >= cap {
                return Err(refused("that gift has run out for this event"));
            }
            bump(&mut stock.taken, gift.counter(), &run);
            limited = true;
        }
        bump(&mut gifts.claims, &gift.limit_key(character), &run);
        gifts.taken.insert(gift.ledger(character), now);
    }
    if limited {
        stock.schema_version = SCHEMA_VERSION;
        write_json(&Stock::path(store), &stock)?;
    }
    write_json(&path, &gifts)?;
    Ok(taken)
}

// The tick's hand-offs: accounts that were sent a gift (tell them), and accounts whose list
// moved under a character claim (send the list again). Keyed by account store, so test
// gateways with their own stores never see each other's.
static DELIVERED: Mutex<BTreeSet<(PathBuf, u64)>> = Mutex::new(BTreeSet::new());
static RESEND: Mutex<BTreeSet<(PathBuf, u64)>> = Mutex::new(BTreeSet::new());

fn mark(set: &Mutex<BTreeSet<(PathBuf, u64)>>, store: &Path, account: u64) {
    set.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert((store.to_path_buf(), account));
}

fn take_marked(set: &Mutex<BTreeSet<(PathBuf, u64)>>, store: &Path) -> BTreeSet<u64> {
    let mut set = set.lock().unwrap_or_else(PoisonError::into_inner);
    let (mine, rest): (BTreeSet<_>, BTreeSet<_>) =
        std::mem::take(&mut *set).into_iter().partition(|(path, _)| path == store);
    *set = rest;
    mine.into_iter().map(|(_, account)| account).collect()
}

/// Accounts sent a gift since the last call (for this store).
pub fn take_delivered(store: &Path) -> BTreeSet<u64> {
    take_marked(&DELIVERED, store)
}

/// Accounts whose Claim window list must be sent again (for this store).
pub fn take_resend(store: &Path) -> BTreeSet<u64> {
    take_marked(&RESEND, store)
}

pub(crate) fn mark_resend(store: &Path, account: u64) {
    mark(&RESEND, store, account);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("osfr-gifts-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("accounts.json")
    }

    fn gift(key: &str) -> Gift {
        Gift {
            key: key.into(),
            name: key.into(),
            items: vec![GiftItem {
                item_id: 70430,
                count: 1,
                name_id: 1,
            }],
            source: Source::Profile,
            code: None,
            per: Per::Account,
            repeatable: false,
            repeat_hours: None,
            live_event: None,
            tradeable: false,
            enabled: true,
            coins: 0,
            stock_per_event: None,
            per_player: None,
            stock_group: None,
            note: String::new(),
            evidence: String::new(),
        }
    }

    fn catalog(gifts: Vec<Gift>) -> Catalog {
        Catalog {
            schema_version: 1,
            gifts,
            ..Catalog::default()
        }
    }

    #[test]
    fn the_shipped_catalog_loads_and_keeps_the_contract() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let shipped = Catalog::load(&dir).unwrap();
        assert!(shipped.gifts.len() >= 90, "{}", shipped.gifts.len());
        assert!(shipped.by_code("getspooky").is_some_and(|g| g.name == "Robgoblin Costume"));
        let mut bad = catalog(vec![gift("a")]);
        bad.gifts[0].repeatable = true;
        assert!(bad.validate().is_err(), "repeatable needs its hours");
        bad.gifts[0].repeat_hours = Some(24);
        bad.validate().unwrap();
        bad.gifts.push(gift("a"));
        assert!(bad.validate().is_err(), "keys are unique");
    }

    #[test]
    fn offers_come_once_repeat_after_their_hours_and_follow_events() {
        let store = scratch("offers");
        let mut daily = gift("daily");
        daily.repeatable = true;
        daily.repeat_hours = Some(24);
        let mut spooky = gift("spooky");
        spooky.live_event = Some("halloween".into());
        let once = gift("once");
        let catalog = catalog(vec![once, daily, spooky]);
        let toggles = Toggles::default();
        let (off, stock) = (Events::new(), Stock::default());
        let rules = Rules { catalog: &catalog, toggles: &toggles, events: &off, stock: &stock };
        assert_eq!(sync_offers(&store, 7, 70, &rules, 1_000).unwrap(), 2);
        assert_eq!(sync_offers(&store, 7, 70, &rules, 1_000).unwrap(), 0, "already waiting");
        let path = box_path(&store, 7);
        let ids: Vec<u32> = load(&path).unwrap().pending.iter().map(|p| p.id).collect();
        assert_eq!(take(&store, 7, &ids, 70, &rules, 1_000).unwrap().len(), 2);
        assert!(take(&store, 7, &ids, 70, &rules, 1_000).is_err(), "claimed once");
        assert_eq!(sync_offers(&store, 7, 70, &rules, 1_000 + 3600).unwrap(), 0);
        assert_eq!(sync_offers(&store, 7, 70, &rules, 1_000 + 24 * 3600).unwrap(), 1, "daily again");
        let on = Events::from([("halloween".to_owned(), "2026-11-12".to_owned())]);
        let rules = Rules { catalog: &catalog, toggles: &toggles, events: &on, stock: &stock };
        assert_eq!(sync_offers(&store, 7, 70, &rules, 1_000 + 24 * 3600).unwrap(), 1, "the event's row");
        let spooky = load(&path).unwrap().pending.into_iter().find(|p| p.key.as_deref() == Some("spooky")).unwrap();
        assert!(rules.shows(&spooky, 70));
        let rules = Rules { catalog: &catalog, toggles: &toggles, events: &off, stock: &stock };
        assert!(!rules.shows(&spooky, 70), "hidden once the event ends");
        let _ = fs::remove_dir_all(gifts_dir(&store));
    }

    #[test]
    fn a_classic_code_is_once_per_account_and_a_toggle_hides_its_row() {
        let store = scratch("classic");
        let mut code = gift("code-snowdays");
        code.source = Source::Code;
        code.code = Some("SNOWDAYS".into());
        let catalog = catalog(vec![code.clone()]);
        let toggles = Toggles::default();
        let (off, stock) = (Events::new(), Stock::default());
        let rules = Rules { catalog: &catalog, toggles: &toggles, events: &off, stock: &stock };
        assert_eq!(sync_offers(&store, 3, 30, &rules, 5).unwrap(), 0, "codes are no offers");
        redeem_classic(&store, 3, 30, &code, &rules).unwrap();
        assert!(redeem_classic(&store, 3, 31, &code, &rules).is_err(), "per account");
        assert_eq!(take_delivered(&store), BTreeSet::from([3]));
        Toggles::set(&store, &code, false).unwrap();
        let toggles = Toggles::load(&store);
        let rules = Rules { catalog: &catalog, toggles: &toggles, events: &off, stock: &stock };
        assert!(redeem_classic(&store, 4, 40, &code, &rules).is_err(), "a disabled row's code");
        let _ = fs::remove_dir_all(gifts_dir(&store));
    }

    /// A limited row (the LIMITED EDITION kind): `stock_per_event` claims per event run for the
    /// whole realm, `per_player` per player per run; a new run starts both again.
    #[test]
    fn limited_rows_keep_their_stock_and_per_player_counts_per_run() {
        let store = scratch("limited");
        let mut wolf = gift("werewolf");
        wolf.live_event = Some("halloween".into());
        wolf.repeatable = true;
        wolf.repeat_hours = Some(1);
        wolf.stock_per_event = Some(2);
        wolf.per_player = Some(1);
        let catalog = catalog(vec![wolf]);
        let toggles = Toggles::default();
        let run = |end: &str| Events::from([("halloween".to_owned(), end.to_owned())]);
        let october = run("2026-11-12");
        let claim = |account: u64, now: u64, events: &Events| {
            let stock = Stock::read(&store).unwrap();
            let rules = Rules { catalog: &catalog, toggles: &toggles, events, stock: &stock };
            sync_offers(&store, account, account * 10, &rules, now).unwrap();
            let waiting = load(&box_path(&store, account)).unwrap().pending;
            let ids: Vec<u32> = waiting.iter().map(|p| p.id).collect();
            (ids.len(), take(&store, account, &ids, account * 10, &rules, now))
        };
        assert!(matches!(claim(1, 10, &october), (1, Ok(_))));
        // Its repeat hours pass, but one per player per run.
        assert!(matches!(claim(1, 10 + 7200, &october), (0, Ok(_))));
        assert!(matches!(claim(2, 10, &october), (1, Ok(_))), "the second of two");
        assert!(matches!(claim(3, 10, &october), (0, Ok(_))), "out of stock: not offered");
        assert_eq!(Stock::read(&store).unwrap().taken["werewolf"].count, 2);
        // Next October is a new run: stock and the player's count start again.
        let next = run("2027-11-12");
        assert!(matches!(claim(3, 10, &next), (1, Ok(_))));
        assert!(matches!(claim(1, 10 + 7200, &next), (1, Ok(_))));
        let _ = fs::remove_dir_all(gifts_dir(&store));
    }

    /// Rows with one `stock_group` share their counters: two colours, one stock of 2.
    #[test]
    fn rows_of_one_stock_group_share_one_stock() {
        let store = scratch("group");
        let colour = |key: &str| {
            let mut row = gift(key);
            row.stock_per_event = Some(2);
            row.stock_group = Some("werewolf".into());
            row
        };
        let catalog = catalog(vec![colour("brown"), colour("grey")]);
        let toggles = Toggles::default();
        let events = Events::new();
        let offered = |account: u64| {
            let stock = Stock::read(&store).unwrap();
            let rules = Rules { catalog: &catalog, toggles: &toggles, events: &events, stock: &stock };
            sync_offers(&store, account, account, &rules, 5).unwrap();
            let ids: Vec<u32> = load(&box_path(&store, account)).unwrap().pending.iter().map(|p| p.id).collect();
            (ids.len(), take(&store, account, &ids, account, &rules, 5).is_ok())
        };
        // The first player takes one of each colour: the group's stock of 2 is gone.
        assert_eq!(offered(1), (2, true));
        assert_eq!(Stock::read(&store).unwrap().taken["werewolf"].count, 2);
        assert_eq!(offered(2), (0, true), "neither colour is left");
        let _ = fs::remove_dir_all(gifts_dir(&store));
    }
}
