//! T633/T640: what the shipped `stores.json` row pays for one unit, read off the row's own
//! `sell_price_coins` so retuning a row never needs a new literal in a test. The T633 rule
//! (a tenth of the buy price, at least 1) is checked in `vendor_sell_worth.rs` only.

use serde_json::Value;
use std::{fs, path::PathBuf, sync::OnceLock};

/// Every shipped store row.
#[allow(dead_code)] // only the rule checks in vendor_sell_worth.rs walk all rows
pub fn rows() -> &'static [Value] {
    static ROWS: OnceLock<Vec<Value>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json");
        let mut stores: Value = serde_json::from_slice(&fs::read(path).expect("stores.json reads"))
            .expect("stores.json parses");
        match stores["items"].take() {
            Value::Array(rows) => rows,
            other => panic!("store rows are an array, got {other}"),
        }
    })
}

fn row(definition: i32) -> &'static Value {
    rows()
        .iter()
        .find(|row| row["item_id"] == definition)
        .unwrap_or_else(|| panic!("store row {definition}"))
}

/// What the shipped store row pays for one unit (its own `sell_price_coins`).
pub fn resale(definition: i32) -> u32 {
    let price = row(definition)["sell_price_coins"]
        .as_u64()
        .unwrap_or_else(|| panic!("store row {definition} has a sell price"));
    u32::try_from(price).expect("sell price fits a Coin balance")
}

/// The row's regular coin buy price.
#[allow(dead_code)] // the T633 canary in vendor_sell_worth.rs is the only reader
pub fn purchase_price(definition: i32) -> u32 {
    let price = row(definition)["purchase_price_coins"]
        .as_u64()
        .unwrap_or_else(|| panic!("store row {definition} has a buy price"));
    u32::try_from(price).expect("buy price fits a Coin balance")
}
