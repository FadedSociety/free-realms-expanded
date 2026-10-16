//! Private W84 oracle. No captured content is embedded in the repository.
#[allow(dead_code)]
#[path = "../src/inventory_v2_fixes.rs"]
mod inventory_v2_fixes;

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

fn hex(value: &str) -> Vec<u8> {
    assert!(value.is_ascii() && value.len() % 2 == 0);
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
        .collect()
}

fn batch(records: &[Vec<u8>]) -> Vec<u8> {
    inventory_v2_fixes::encode_definition_batch_v2(records, &[]).unwrap()
}

#[test]
#[ignore = "requires OSFR_ITEM_CATALOGUE pointing at private W84 output"]
fn captured_rows_roundtrip_and_generated_catalogue_use_existing_batch_encoder() {
    let directory = std::env::var_os("OSFR_ITEM_CATALOGUE").expect("private output required");
    let directory = Path::new(&directory);
    let report: Value =
        serde_json::from_slice(&fs::read(directory.join("report.json")).unwrap()).unwrap();
    let source = fs::read(report["inputs"]["inventory"]["path"].as_str().unwrap()).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&source)),
        report["inputs"]["inventory"]["sha256"].as_str().unwrap()
    );
    let source: Value = serde_json::from_slice(&source).unwrap();
    let roundtrip: Value =
        serde_json::from_slice(&fs::read(directory.join("roundtrip.json")).unwrap()).unwrap();
    let original = source["definitions"].as_array().unwrap();
    let regenerated = roundtrip.as_array().unwrap();
    assert_eq!(
        original.len(),
        1703,
        "review a changed private oracle explicitly"
    );
    assert_eq!(regenerated.len(), original.len());
    let mut records = Vec::new();
    for (before, after) in original.iter().zip(regenerated) {
        assert_eq!(before["id"], after["id"]);
        let expected = hex(before["serialized_hex"].as_str().unwrap());
        assert_eq!(format!("{:x}", Sha256::digest(&expected)), before["sha256"]);
        let actual = hex(after["serialized_hex"].as_str().unwrap());
        assert_eq!(expected, actual, "row {} differs", before["id"]);
        records.push(actual);
    }
    let packet = batch(&records);
    assert_eq!(&packet[..4], &[35, 0, 37, 0]);
    assert_eq!(
        i32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize,
        packet.len() - 8
    );
    assert_eq!(i32::from_le_bytes(packet[8..12].try_into().unwrap()), 1649);

    let catalogue: Value =
        serde_json::from_slice(&fs::read(directory.join("catalogue.json")).unwrap()).unwrap();
    let mut by_id = BTreeMap::new();
    for row in catalogue["definitions"].as_array().unwrap() {
        let record = hex(row["serialized_hex"].as_str().unwrap());
        assert_eq!(
            i32::from_le_bytes(record[..4].try_into().unwrap()) as i64,
            row["id"].as_i64().unwrap()
        );
        assert_eq!(format!("{:x}", Sha256::digest(&record)), row["sha256"]);
        assert!(by_id.insert(row["id"].as_i64().unwrap(), record).is_none());
    }
    assert_eq!(
        by_id.len() as u64,
        report["generated_unique_ids"].as_u64().unwrap()
    );
    let generated = batch(&by_id.into_values().collect::<Vec<_>>());
    assert_eq!(
        i32::from_le_bytes(generated[4..8].try_into().unwrap()) as usize,
        generated.len() - 8
    );

    // Canary: changing one source byte must be observable in the batch proof.
    let mut corrupted = records.clone();
    corrupted[0][4] ^= 1;
    assert_ne!(batch(&corrupted), packet);
    println!(
        "roundtrip: {}/{} rows; unique=1649; generated={}; framing and mutation canary passed",
        regenerated.len(),
        original.len(),
        report["generated_unique_ids"]
    );
}
