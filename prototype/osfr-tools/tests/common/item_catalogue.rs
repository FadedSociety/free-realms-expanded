//! Shared synthetic `item-catalogue.json` rows for the gateway bin's budget tests and the
//! `inventory_catalogue` integration fixture. Test-only; production never includes this file.
//! Include with `#[path = ".../tests/common/item_catalogue.rs"] mod item_catalogue;`.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

/// One catalogue row whose serialized definition record is `bytes`.
pub fn row(id: i32, bytes: Vec<u8>) -> Value {
    json!({
        "id": id, "slot": 0, "class_id": 1, "default_tint": 0, "max_stack": 10,
        "model": "", "texture": "", "tint_alias": "", "effect": 0, "wield_type": 0,
        "serialized_hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "sha256": format!("{:x}", Sha256::digest(&bytes))
    })
}

/// One row whose serialized definition is `id` followed by `0x5a` padding to `size` bytes.
pub fn sized_row(id: u32, size: usize) -> Value {
    let mut bytes = (id as i32).to_le_bytes().to_vec();
    bytes.resize(size, 0x5a);
    row(id as i32, bytes)
}

/// Write one `{"schema_version":1,"definitions":[...]}` catalogue into `directory`.
pub fn write(directory: &Path, rows: &[Value]) {
    fs::write(
        directory.join("item-catalogue.json"),
        serde_json::to_vec(&json!({"schema_version":1,"definitions":rows})).unwrap(),
    )
    .unwrap();
}
