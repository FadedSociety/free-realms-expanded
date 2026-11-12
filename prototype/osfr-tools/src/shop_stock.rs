//! B2 (owner 2026-11-11, "everything available"): limited shop stock.
//!
//! A `stores.json` row may carry `stock_per_event` (units one run of its `live_event` sells to
//! all players together), `per_player` (units one character may buy in that run) and
//! `stock_group` (rows that share both counters, such as every colour and gender row of one
//! outfit; a row without one counts alone). The counters are realm state, not content: one file
//! beside the account store (`shop-stock.json`, next to `redeem-codes.json`), read and written
//! under the account store's lock, so every gateway process shares them (R). A counter belongs
//! to one run of its event, named by the instant that run ends (`EventCalendar::running_until`;
//! 0 for a run held on by hand, or a row with no event). The next run has a new name, so the
//! counter starts again at zero when the event next starts.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io, path::Path};

pub const FILE: &str = "shop-stock.json";
/// (R) Bounds the file read into memory: one counter and one entry per buyer per group.
const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// One row group's limits, as `stores.json` gives them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limit {
    pub group: String,
    pub event: Option<String>,
    pub per_event: Option<u32>,
    pub per_player: Option<u32>,
}

/// Why a limited buy is refused; the shop tells the player in its own refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    SoldOut,
    PlayerLimit,
    /// The realm's counters file is held by another writer or unreadable: nothing is sold.
    Busy,
}

impl Refusal {
    pub fn message(self) -> &'static str {
        match self {
            Self::SoldOut => "That item is sold out until its event comes round again.",
            Self::PlayerLimit => "You have bought as many of that item as this event allows.",
            Self::Busy => "The shop is busy. Please try again.",
        }
    }
}

/// One group's sales in one run.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counter {
    pub run: u64,
    pub sold: u32,
    /// Character guid -> units bought this run.
    pub players: BTreeMap<u64, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    pub schema_version: u32,
    pub counters: BTreeMap<String, Counter>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            schema_version: 1,
            counters: BTreeMap::new(),
        }
    }
}

impl Ledger {
    /// The realm's counters; a missing file is an empty ledger.
    pub fn load(path: &Path) -> io::Result<Self> {
        match std::fs::metadata(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
            Ok(meta) if !meta.is_file() || meta.len() > MAX_FILE_BYTES => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "shop stock file is not a bounded regular file",
                ));
            }
            Ok(_) => {}
        }
        let ledger: Self =
            serde_json::from_slice(&std::fs::read(path)?).map_err(io::Error::other)?;
        if ledger.schema_version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "shop stock file needs schema_version 1",
            ));
        }
        Ok(ledger)
    }

    /// Whole-file replace, so a crash leaves the old counters or the new ones.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        crate::character_store::replace_bytes(path, &bytes)
    }

    fn counter(&self, limit: &Limit, run: u64) -> Option<&Counter> {
        (self.counters.get(&limit.group)).filter(|counter| counter.run == run)
    }

    /// Units the group has left this run; `None` when it has no per-event cap.
    pub fn remaining(&self, limit: &Limit, run: u64) -> Option<u32> {
        let sold = self.counter(limit, run).map_or(0, |counter| counter.sold);
        limit.per_event.map(|cap| cap.saturating_sub(sold))
    }

    /// Whether `buyer` may buy `quantity` more units this run.
    pub fn check(&self, limit: &Limit, run: u64, buyer: u64, quantity: u32) -> Result<(), Refusal> {
        if self
            .remaining(limit, run)
            .is_some_and(|left| left < quantity)
        {
            return Err(Refusal::SoldOut);
        }
        let mine = (self.counter(limit, run))
            .and_then(|counter| counter.players.get(&buyer).copied())
            .unwrap_or(0);
        if limit
            .per_player
            .is_some_and(|cap| mine.saturating_add(quantity) > cap)
        {
            return Err(Refusal::PlayerLimit);
        }
        Ok(())
    }

    /// Count a sale `check` allowed. A counter left from an earlier run starts again at zero.
    pub fn take(&mut self, limit: &Limit, run: u64, buyer: u64, quantity: u32) {
        let counter = self.counters.entry(limit.group.clone()).or_default();
        if counter.run != run {
            *counter = Counter {
                run,
                ..Counter::default()
            };
        }
        counter.sold = counter.sold.saturating_add(quantity);
        let mine = counter.players.entry(buyer).or_default();
        *mine = mine.saturating_add(quantity);
    }

    /// Undo a `take` whose sale did not go through.
    pub fn give_back(&mut self, limit: &Limit, run: u64, buyer: u64, quantity: u32) {
        let Some(counter) = (self.counters.get_mut(&limit.group)).filter(|c| c.run == run) else {
            return;
        };
        counter.sold = counter.sold.saturating_sub(quantity);
        if let Some(mine) = counter.players.get_mut(&buyer) {
            *mine = mine.saturating_sub(quantity);
            if *mine == 0 {
                counter.players.remove(&buyer);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limit(per_event: Option<u32>, per_player: Option<u32>) -> Limit {
        Limit {
            group: "outfit".into(),
            event: Some("halloween".into()),
            per_event,
            per_player,
        }
    }

    #[test]
    fn a_new_run_starts_the_counter_again() {
        let cap = limit(Some(2), None);
        let mut ledger = Ledger::default();
        ledger.take(&cap, 10, 1, 1);
        ledger.take(&cap, 10, 2, 1);
        assert_eq!(ledger.check(&cap, 10, 3, 1), Err(Refusal::SoldOut));
        assert_eq!(ledger.remaining(&cap, 20), Some(2), "next year's run");
        ledger.take(&cap, 20, 3, 1);
        assert_eq!(ledger.counters["outfit"].sold, 1);
        ledger.give_back(&cap, 20, 3, 1);
        assert_eq!(ledger.remaining(&cap, 20), Some(2));
    }

    #[test]
    fn a_player_cap_counts_one_buyer() {
        let cap = limit(None, Some(1));
        let mut ledger = Ledger::default();
        ledger.take(&cap, 0, 7, 1);
        assert_eq!(ledger.check(&cap, 0, 7, 1), Err(Refusal::PlayerLimit));
        assert_eq!(ledger.check(&cap, 0, 8, 1), Ok(()));
        assert_eq!(ledger.remaining(&cap, 0), None, "no shared cap");
    }
}
