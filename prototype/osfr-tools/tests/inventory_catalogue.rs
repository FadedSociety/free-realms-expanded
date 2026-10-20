//! W88: synthetic public checks; private W84 rows are read only in the ignored gate.
use osfr_tools::inventory::Inventory;
#[path = "common/item_catalogue.rs"]
#[allow(dead_code)]
mod item_catalogue;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn row(id: i32, marker: i32) -> Value {
    let bytes = [id.to_le_bytes(), marker.to_le_bytes()].concat();
    item_catalogue::row(id, bytes)
}

fn request(id: i32) -> Vec<u8> {
    [vec![35, 0, 34, 0], id.to_le_bytes().to_vec(), vec![0; 4]].concat()
}

fn word(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/build/w88/fixtures")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        let zero = format!("{:x}", Sha256::digest([0; 4]));
        let manifest = json!({"schema_version":1,"player_guid":1,
            "default_state":{"coins":0,"items":[],"equipment":[]},"profiles":[],
            "spans":[{"kind":"coins","offset":13,"length":4,"sha256":zero,"profile_id":0},
                {"kind":"items","offset":17,"length":4,"sha256":zero,"profile_id":0}],
            "definitions":[row(100, 7)]});
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        Self(path)
    }
    fn catalogue(&self, rows: Vec<Value>) {
        item_catalogue::write(&self.0, &rows);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn generated_grant_sends_batch_first_and_reload_includes_owned_rows_only() {
    let f = Fixture::new();
    f.catalogue(vec![row(200, 8), row(300, 9)]);
    let mut inventory = Inventory::load(&f.0, None).unwrap();
    assert!(
        inventory.has_definition(200),
        "generated-only definition must load"
    );
    assert_eq!(word(&inventory.definitions_packet(), 8), 1);
    let granted = inventory.grant(0, 200, 1).unwrap();
    assert_eq!(granted.replies.len(), 2);
    assert_eq!(
        granted.replies[0],
        [
            vec![35, 0, 37, 0],
            12i32.to_le_bytes().to_vec(),
            1i32.to_le_bytes().to_vec(),
            200i32.to_le_bytes().to_vec(),
            8i32.to_le_bytes().to_vec()
        ]
        .concat()
    );
    assert_eq!(&granted.replies[1][..4], &[38, 0, 2, 0]);
    assert_eq!(
        &granted.replies[1][granted.replies[1].len() - 8..],
        &granted.replies[0][12..]
    );
    let stacked = inventory.grant(0, 200, 1).unwrap();
    assert_eq!(&stacked.replies[0][..4], &[35, 0, 37, 0]);
    assert_eq!(&stacked.replies[1][..4], &[38, 0, 3, 0]);
    let loaded = Inventory::load(&f.0, Some(inventory.state().clone())).unwrap();
    assert_eq!(loaded.state(), inventory.state());
    let packet = loaded.definitions_packet();
    assert_eq!(word(&packet, 8), 2);
    assert_eq!(word(&packet, 12), 100);
    assert_eq!(word(&packet, 20), 200);
}

#[test]
fn captured_rows_win_and_optional_catalogue_fails_closed() {
    let f = Fixture::new();
    f.catalogue(vec![row(100, 999), row(200, 8)]);
    let mut inventory = Inventory::load(&f.0, None).unwrap();
    assert_eq!(
        word(&inventory.handle(&request(100)).unwrap().replies[0], 16),
        7
    );
    assert!(inventory.has_definition(200));
    f.catalogue(vec![row(200, 8), row(200, 8)]);
    assert!(
        Inventory::load(&f.0, None).is_err(),
        "duplicate generated IDs rejected"
    );
    let mut bad = row(200, 8);
    bad["sha256"] = json!("bad");
    f.catalogue(vec![bad]);
    assert!(Inventory::load(&f.0, None).is_err());
    fs::write(
        f.0.join("item-catalogue.json"),
        b"{\"schema_version\":2,\"definitions\":[]}",
    )
    .unwrap();
    assert!(Inventory::load(&f.0, None).is_err());
    fs::write(
        f.0.join("item-catalogue.json"),
        b"{\"schema_version\":1,\"definitions\":[]}x",
    )
    .unwrap();
    assert!(Inventory::load(&f.0, None).is_err());
}

#[test]
fn definition_requests_reject_malformed_and_oversized_input_without_mutating_state() {
    let f = Fixture::new();
    let mut inventory = Inventory::load(&f.0, None).unwrap();
    let before = inventory.state().clone();
    let valid = request(100);
    for length in 0..valid.len() {
        assert!(inventory.handle(&valid[..length]).is_err());
        assert!(inventory.definition_replies(&valid[..length]).is_err());
    }
    let mut invalid = vec![request(0), request(-1), request(i32::MIN)];
    for length in [1i32, -1, i32::MAX] {
        let mut packet = valid.clone();
        packet[8..12].copy_from_slice(&length.to_le_bytes());
        invalid.push(packet);
    }
    // Native carries one ID, not a count or concatenated requests.
    invalid.push([valid.clone(), request(100)].concat());
    invalid.push([valid.clone(), vec![0]].concat());
    invalid.push(vec![0; 65_537]);
    for packet in invalid {
        assert!(inventory.definition_replies(&packet).is_err());
        assert!(inventory.handle(&packet).is_err());
    }
    for id in [200, i32::MAX] {
        let outcome = inventory.handle(&request(id)).unwrap();
        assert!(outcome.replies.is_empty());
        assert!(!outcome.changed);
    }
    assert_eq!(inventory.state(), &before);
    assert_eq!(
        inventory.definition_replies(&valid).unwrap(),
        inventory.handle(&valid).unwrap().replies
    );
    let mut wrong_header = valid.clone();
    wrong_header[2] = 37;
    assert!(inventory.definition_replies(&wrong_header).is_err());
}

#[test]
fn catalogue_file_row_count_and_row_size_have_independent_limits() {
    use std::io::Write;
    let f = Fixture::new();
    let mut oversized = row(200, 8);
    let mut bytes = vec![0; 65_537];
    bytes[..4].copy_from_slice(&200i32.to_le_bytes());
    oversized["serialized_hex"] =
        json!(bytes.iter().map(|b| format!("{b:02x}")).collect::<String>());
    oversized["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    f.catalogue(vec![oversized]);
    assert!(
        Inventory::load(&f.0, None)
            .err()
            .unwrap()
            .to_string()
            .contains("hex size")
    );
    let path = f.0.join("item-catalogue.json");
    let mut file = std::io::BufWriter::new(fs::File::create(&path).unwrap());
    file.write_all(b"{\"schema_version\":1,\"definitions\":[")
        .unwrap();
    for id in 1..=65_537 {
        if id != 1 {
            file.write_all(b",").unwrap();
        }
        serde_json::to_writer(&mut file, &row(id, 8)).unwrap();
    }
    file.write_all(b"]}").unwrap();
    drop(file);
    assert!(
        Inventory::load(&f.0, None)
            .err()
            .unwrap()
            .to_string()
            .contains("row limit")
    );
    let mut file = std::io::BufWriter::new(fs::File::create(&path).unwrap());
    file.write_all(b"{\"schema_version\":1,\"definitions\":[")
        .unwrap();
    for id in 1i32..=513 {
        if id != 1 {
            file.write_all(b",").unwrap();
        }
        let mut bytes = vec![0; 65_536];
        bytes[..4].copy_from_slice(&id.to_le_bytes());
        let mut definition = row(id, 8);
        definition["serialized_hex"] =
            json!(bytes.iter().map(|b| format!("{b:02x}")).collect::<String>());
        definition["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
        serde_json::to_writer(&mut file, &definition).unwrap();
    }
    file.write_all(b"]}").unwrap();
    drop(file);
    assert!(
        Inventory::load(&f.0, None)
            .err()
            .unwrap()
            .to_string()
            .contains("byte limit")
    );
    fs::File::create(&path)
        .unwrap()
        .set_len(128 * 1024 * 1024 + 1)
        .unwrap();
    assert!(
        Inventory::load(&f.0, None)
            .err()
            .unwrap()
            .to_string()
            .contains("128 MiB")
    );
}

fn private_fixture() -> (Fixture, PathBuf) {
    let directory =
        PathBuf::from(std::env::var_os("OSFR_ITEM_CATALOGUE").expect("W84 output required"));
    let report: Value =
        serde_json::from_slice(&fs::read(directory.join("report.json")).unwrap()).unwrap();
    let source = PathBuf::from(report["inputs"]["inventory"]["path"].as_str().unwrap());
    let input = fs::read(&source).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&input)),
        report["inputs"]["inventory"]["sha256"]
    );
    let f = Fixture::new();
    fs::write(f.0.join("inventory.json"), input).unwrap();
    fs::copy(
        directory.join("catalogue.json"),
        f.0.join("item-catalogue.json"),
    )
    .unwrap();
    (f, directory)
}

#[test]
#[ignore = "requires OSFR_ITEM_CATALOGUE; separate process for startup memory sampling"]
fn private_startup_measurement() {
    let (f, _) = private_fixture();
    let start = std::time::Instant::now();
    let inventory = Inventory::load(&f.0, None).unwrap();
    let elapsed = start.elapsed();
    assert!(inventory.has_definition(4914));
    println!(
        "W88 startup_ms={:.3} startup_packet_bytes={} owned_items={}",
        elapsed.as_secs_f64() * 1000.0,
        inventory.definitions_packet().len(),
        inventory.state().items.len()
    );
    // Allow the external, read-only process-memory sampler to observe residency.
    std::thread::sleep(std::time::Duration::from_millis(300));
    std::hint::black_box(inventory);
}

#[test]
#[ignore = "requires OSFR_ITEM_CATALOGUE pointing at private W84 output"]
fn private_w84_rows_grant_and_reply_byte_exactly() {
    let (f, directory) = private_fixture();
    let catalogue: Value =
        serde_json::from_slice(&fs::read(directory.join("catalogue.json")).unwrap()).unwrap();
    let mut inventory = Inventory::load(&f.0, None).unwrap();
    let before = inventory.state().clone();
    let rows = catalogue["definitions"].as_array().unwrap();
    assert_eq!(rows.len(), 58_835);
    let hex = |row: &Value| {
        row["serialized_hex"]
            .as_str()
            .unwrap()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>()
    };
    for row in rows {
        let id = row["id"].as_i64().unwrap() as i32;
        let expected = hex(row);
        let reply = inventory.definition_replies(&request(id)).unwrap();
        assert_eq!(reply.len(), 1);
        assert_eq!(word(&reply[0], 4), id);
        assert_eq!(word(&reply[0], 8) as usize, expected.len());
        assert_eq!(&reply[0][12..], expected, "W84 row {id}");
    }
    assert_eq!(inventory.state(), &before);
    let generated = rows.iter().find(|row| row["id"] == 4914).unwrap();
    let expected = hex(generated);
    let grant = inventory.grant(0, 4914, 1).unwrap();
    let batch = grant
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 37, 0]))
        .unwrap();
    assert_eq!(word(batch, 8), 1);
    assert_eq!(&batch[12..], expected);
    let item = grant
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[38, 0, 2, 0]))
        .unwrap();
    assert_eq!(&item[item.len() - expected.len()..], expected);
    let loaded = Inventory::load(&f.0, Some(inventory.state().clone())).unwrap();
    assert_eq!(loaded.state(), inventory.state());
    println!(
        "W84 exact replies={}; generated-only grant=4914; reload passed",
        rows.len()
    );
}
