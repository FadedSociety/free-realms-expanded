//! The Marketplace panel on every minigame offer (P-games-window): which store bundles an offer
//! advertises, and the session's watch over the offer that is open.
//!
//! Client evidence (2014 `FreeRealms.exe`):
//! * 39/44 `MiniGameStoreAdvertisementWorldClient` (constructor `009a9f80`, dispatcher `009bfea0`
//!   case 0x2C). Reader `009b5f10`: the 15-byte minigame header, an i32, then `009b0700`'s counted
//!   list of i32 store bundle ids.
//! * `009b6fe0` hands the ids to the store data source `BaseClient.MiniGame.MarketplaceAd` and
//!   calls Lua `OnMarketplaceAd(7, the i32)` on the start handler; `MinigameStartScreen` then
//!   shows the bundles in its flyout, and BUY! opens the Marketplace on them
//!   (`MarketplaceHandler:ShowCustomView`). An encounter's own start packet does the same with
//!   referrer 8 (`00aa36c0`); that packet is not written here.
//! * The rows come from the client's store, so the bundles must be Marketplace listings the client
//!   already holds (66/5).
//!
//! Which bundles the live server advertised was never captured: the list is data
//! (`offer_boosts.json`, R).

use serde::Deserialize;
use std::{collections::BTreeMap, io, path::Path};

pub const FILE: &str = "offer_boosts.json";
/// The client's bundle id set holds 32 ids (`009b4060`).
pub const MAX_ADS: usize = 32;
const MAX_FILE_BYTES: u64 = 256 * 1024;
const MINIGAME: [u8; 2] = [39, 0];
const OFFER: u8 = 16;
const ADVERTISEMENT: u8 = 44;
/// 39/17 start, 39/18 game over, 39/19 end, 39/7 removal: the offer is no longer open.
const CLOSES_OFFER: [u8; 4] = [7, 17, 18, 19];
/// The client's own 39/5 (Go) and 39/6 (leave): the offer is no longer open, whatever the game
/// answers.
const CLIENT_CLOSES_OFFER: [u8; 2] = [5, 6];

/// When the Marketplace rows are pushed (R; see `store_push_note` in the data).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorePush {
    WorldReady,
    #[default]
    FirstOffer,
    Never,
}

/// One advertised bundle. Empty filters mean every game.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ad {
    pub item_id: u32,
    /// MiniGameInfo type ids (`RunSpec::type_id`).
    #[serde(default)]
    pub game_types: Vec<i32>,
    #[serde(default)]
    pub game_ids: Vec<i32>,
    #[serde(default)]
    pub source: String,
}

impl Ad {
    fn fits(&self, game: i32, kind: i32) -> bool {
        (self.game_types.is_empty() || self.game_types.contains(&kind))
            && (self.game_ids.is_empty() || self.game_ids.contains(&game))
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    schema_version: u32,
    #[serde(default)]
    source: String,
    pub enabled: bool,
    #[serde(default)]
    pub store_push: StorePush,
    #[serde(default)]
    store_push_note: String,
    #[serde(default)]
    pub use_when_bought_at_offer: bool,
    #[serde(default)]
    use_note: String,
    pub max_ads: usize,
    pub ads: Vec<Ad>,
    /// An event's own ads, by event key: the events framework's hook.
    #[serde(default)]
    pub event_ads: BTreeMap<String, Vec<Ad>>,
    #[serde(default)]
    event_note: String,
}

impl Table {
    /// `offer_boosts.json`, or `None` when the content set has none (no panel is sent).
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let path = directory.join(FILE);
        if !path.is_file() {
            return Ok(None);
        }
        if path.metadata()?.len() > MAX_FILE_BYTES {
            return Err(invalid("offer_boosts.json is too large"));
        }
        let table: Self = serde_json::from_slice(&std::fs::read(path)?).map_err(invalid)?;
        let ads = table.ads.iter().chain(table.event_ads.values().flatten());
        if table.schema_version != 1
            || !(1..=MAX_ADS).contains(&table.max_ads)
            || ads.clone().any(|ad| ad.item_id == 0)
            || ads.count() > 4096
        {
            return Err(invalid("invalid offer_boosts.json"));
        }
        let _ = (&table.source, &table.store_push_note, &table.use_note, &table.event_note);
        Ok(Some(table))
    }

    /// The bundles an offer of `game` (of MiniGameInfo type `kind`) advertises: the live events'
    /// first, then the standing list, each once.
    pub fn ads(&self, game: i32, kind: i32, events: &[String]) -> Vec<u32> {
        let mut ids = Vec::new();
        let event = events.iter().filter_map(|key| self.event_ads.get(key)).flatten();
        for ad in event.chain(&self.ads).filter(|ad| ad.fits(game, kind)) {
            if !ids.contains(&ad.item_id) {
                ids.push(ad.item_id);
            }
        }
        ids
    }
}

/// 39/44: the bundles `instance`'s start screen advertises. `context` reaches Lua as the
/// Marketplace's referrer context (R: the game id).
pub fn ad_packet(instance: i32, game: i32, context: i32, bundles: &[u32]) -> Vec<u8> {
    let mut bytes = crate::minigame::family39_header(ADVERTISEMENT, instance, game);
    bytes.extend(context.to_le_bytes());
    // At most MAX_ADS.
    bytes.extend((bundles.len() as i32).to_le_bytes());
    for id in bundles {
        bytes.extend(id.to_le_bytes());
    }
    bytes
}

fn word(bytes: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

/// One session's panel state. Not saved: a new session starts with no offer and no rows sent.
#[derive(Clone, Debug, Default)]
pub struct Watch {
    table: Option<std::sync::Arc<Table>>,
    events: Vec<String>,
    /// The open offer's instance and what it advertises.
    pending: Option<(i32, Vec<u32>)>,
    store_sent: bool,
}

impl Watch {
    pub fn load(directory: &Path) -> io::Result<Self> {
        Ok(Self {
            table: (Table::load(directory)?.filter(|table| table.enabled)).map(Into::into),
            ..Self::default()
        })
    }

    pub fn reset(&mut self) {
        self.pending = None;
        self.store_sent = false;
    }

    /// The events framework's hook: the keys of the events running now.
    pub fn set_events(&mut self, keys: Vec<String>) {
        self.events = keys;
    }

    pub fn events(&self) -> &[String] {
        &self.events
    }

    /// The open offer's instance.
    pub fn pending_instance(&self) -> Option<i32> {
        self.pending.as_ref().map(|pending| pending.0)
    }

    /// One packet from the client: its Go or leave for any run closes the offer at once, so a
    /// game that answers with none of the closing packets cannot leave a stale one behind.
    pub fn inbound(&mut self, inner: &[u8]) {
        if inner.get(..2) == Some(&MINIGAME[..])
            && inner.get(2).is_some_and(|sub| CLIENT_CLOSES_OFFER.contains(sub))
        {
            self.pending = None;
        }
    }

    /// Whether an offer is open and advertises every one of `bundles`.
    pub fn advertises(&self, mut bundles: impl Iterator<Item = u32>) -> bool {
        self.pending
            .as_ref()
            .is_some_and(|(_, ads)| !ads.is_empty() && bundles.all(|id| ads.contains(&id)))
    }

    pub fn use_when_bought(&self) -> bool {
        self.table.as_ref().is_some_and(|table| table.use_when_bought_at_offer)
    }

    /// Walk one commit's outgoing packets (each an inner packet after `head` wrapper bytes): every
    /// 39/16 offer is followed by its 39/44, and the Marketplace rows (`store`) go out once per
    /// session, ahead of the first panel: with the commit that entered the world (`entered_world`)
    /// or, failing that, just ahead of the first offer. `listed` says whether a bundle is a Marketplace listing.
    pub fn dress(
        &mut self,
        replies: &mut Vec<Vec<u8>>,
        head: usize,
        wrap: fn(&[u8]) -> Vec<u8>,
        entered_world: bool,
        listed: impl Fn(u32) -> bool,
        store: impl Fn() -> Vec<Vec<u8>>,
    ) {
        let Some(table) = self.table.clone() else {
            return;
        };
        let mut at = 0;
        while at < replies.len() {
            let inner = replies[at].get(head..).unwrap_or_default();
            at += 1;
            if inner.get(..2) != Some(&MINIGAME[..]) {
                continue;
            }
            let (Some(&sub), Some(instance)) = (inner.get(2), word(inner, 3)) else {
                continue;
            };
            if sub != OFFER {
                if CLOSES_OFFER.contains(&sub) && self.pending.as_ref().is_some_and(|p| p.0 == instance) {
                    self.pending = None;
                }
                continue;
            }
            // The header's game id, and the body's sixth dword (`minigame::info_body`).
            let (Some(game), Some(kind)) = (word(inner, 11), word(inner, 35)) else {
                continue;
            };
            let mut ads = table.ads(game, kind, &self.events);
            ads.retain(|id| listed(*id));
            ads.truncate(table.max_ads);
            if !ads.is_empty() {
                if !self.store_sent && table.store_push != StorePush::Never {
                    self.store_sent = true;
                    let rows: Vec<_> = store().iter().map(|p| wrap(p)).collect();
                    at += rows.len();
                    replies.splice(at - rows.len() - 1..at - rows.len() - 1, rows);
                }
                replies.insert(at, wrap(&ad_packet(instance, game, game, &ads)));
                at += 1;
            }
            self.pending = Some((instance, ads));
        }
        if entered_world && !self.store_sent && table.store_push == StorePush::WorldReady {
            self.store_sent = true;
            replies.extend(store().iter().map(|p| wrap(p)));
        }
    }
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
