//! T229: the Station Cash a completion is expected to pay, read from the shipped earn table, so
//! tuning `data/content/sc_rewards.json` moves the expectation instead of failing a pinned value.

use osfr_tools::station_cash::{EarnSource, EarnTable};
use std::path::PathBuf;

pub fn table() -> EarnTable {
    EarnTable::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/sc_rewards.json"))
        .expect("the shipped earn table parses")
        .expect("the shipped earn table exists")
}

/// One completion that is the character's first of the UTC day (base amount plus the bonus).
pub fn first_of_day(source: EarnSource) -> u32 {
    let table = table();
    table.amount(source) + table.daily_first_activity
}
