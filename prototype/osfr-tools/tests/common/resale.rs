//! T633 (R): the sell-back rule the shipped `stores.json` rows follow, read off the data so a
//! retuned row never needs a new literal in a test. Wiki tooltips pay a tenth of the buy price.

use serde_json::Value;
use std::{fs, path::PathBuf};

/// What the shipped store row pays for one unit: a tenth of its regular price, at least 1.
pub fn resale(definition: i32) -> u32 {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json");
    let stores: Value = serde_json::from_slice(&fs::read(path).expect("stores.json reads"))
        .expect("stores.json parses");
    let row = stores["items"]
        .as_array()
        .expect("store rows are an array")
        .iter()
        .find(|row| row["item_id"] == definition)
        .unwrap_or_else(|| panic!("store row {definition}"));
    (row["purchase_price_coins"].as_u64().expect("priced row") / 10).max(1) as u32
}
