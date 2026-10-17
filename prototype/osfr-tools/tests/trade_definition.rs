//! W163: the two trade facts the inventory reads from a definition row (NO_TRADE, max stack).
use osfr_tools::inventory::Inventory;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// A definition row with the recovered layout: eleven words, then NO_TRADE and NO_SALE flags
/// (`piece_096` `ItemDefinition`), so NO_TRADE is the byte at +44.
fn row(id: i32, no_trade: bool) -> Vec<u8> {
    let mut bytes = vec![0u8; 147];
    bytes[..4].copy_from_slice(&id.to_le_bytes());
    bytes[44] = u8::from(no_trade);
    bytes
}

fn inventory() -> (PathBuf, Inventory) {
    let path = std::env::temp_dir().join(format!(
        "osfr-trade-definition-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    let state = json!({"coins":0,"items":[],"equipment":[]});
    let span = |kind: &str, offset: usize, bytes: &[u8]| json!({"kind":kind,"offset":offset,"length":bytes.len(),"profile_id":i32::from(kind == "equipment"),"sha256":digest(bytes)});
    let zero = 0i32.to_le_bytes();
    let spans = [
        span("coins", 13, &zero),
        span("equipment", 17, &zero),
        span("items", 21, &zero),
    ];
    // (row, max_stack in the content file)
    let rows = [
        (row(300, false), 7),
        (row(301, true), 7),
        (row(302, false), -1),
        (
            8i32.to_le_bytes()
                .iter()
                .chain(&[0u8; 4])
                .copied()
                .collect(),
            7,
        ), // id 8: too short
    ];
    let definitions: Vec<_> = rows
        .iter()
        .map(|(bytes, max_stack)| {
            let id = i32::from_le_bytes(bytes[..4].try_into().unwrap());
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            json!({"id":id,"slot":3,"class_id":1,"default_tint":0,"max_stack":max_stack,"model":"m.adr","texture":"t","tint_alias":"a","effect":0,"wield_type":0,"serialized_hex":hex,"sha256":digest(bytes)})
        })
        .collect();
    let content = json!({"schema_version":1,"player_guid":1,"default_state":state,"profiles":[{"id":1,"allowed_classes":[1],"slots":[3]}],"spans":spans,"definitions":definitions});
    fs::write(
        path.join("inventory.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .unwrap();
    let inventory = Inventory::load(&path, None).unwrap();
    (path, inventory)
}

#[test]
fn no_trade_and_max_stack_come_from_the_definition_row() {
    let (path, inventory) = inventory();
    assert_eq!(inventory.trade_flags(300), Some((false, Some(7))));
    assert_eq!(inventory.trade_flags(301), Some((true, Some(7))));
    // A max stack of zero or less means uncapped (R).
    assert_eq!(inventory.trade_flags(302), Some((false, None)));
    // A row too short to carry the flag, and an unknown id, have no entry: they cannot be offered.
    assert_eq!(inventory.trade_flags(8), None);
    assert_eq!(inventory.trade_flags(999), None);
    let _ = fs::remove_dir_all(path);
}
