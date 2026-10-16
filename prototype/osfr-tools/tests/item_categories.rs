use osfr_tools::item_categories::ItemCategories;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn integer(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}

struct Fixture {
    directory: PathBuf,
    content: Value,
    expected: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-categories-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let mut expected = vec![44, 0, 2, 0];
        integer(&mut expected, 2);
        for (id, name, icon, sort, visible, display) in [
            (10, 962, 1007, 1200, true, 1),
            (104, 23322, 5141, 1360, false, 0),
        ] {
            for field in [id, id, name, icon, sort] {
                integer(&mut expected, field);
            }
            expected.push(u8::from(visible));
            integer(&mut expected, display);
        }
        integer(&mut expected, 3);
        // The third source row rejoins group8 and serializes before group44.
        for fields in [[8, 8, 10], [8, 8, 58], [44, 44, 104]] {
            for field in fields {
                integer(&mut expected, field);
            }
        }
        let content = json!({"schema_version":1,"categories":[
            {"id":10,"name_id":962,"icon_id":1007,"unknown":1200,"unknown2":true,"display":1},
            {"id":104,"name_id":23322,"icon_id":5141,"unknown":1360,"unknown2":false,"display":0}],
            "groups":[{"id":8,"category_id":10},{"id":44,"category_id":104},{"id":8,"category_id":58}],
            "packet_sha256":format!("{:x}",Sha256::digest(&expected))});
        let fixture = Self {
            directory,
            content,
            expected,
        };
        fixture.write();
        fixture
    }
    fn write(&self) {
        fs::write(
            self.directory.join("categories.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("categories.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}

#[test]
fn dictionary_keys_values_booleans_and_group_order_match_evidenced_layout() {
    let fixture = Fixture::new();
    let categories = ItemCategories::load(&fixture.directory).unwrap();
    assert_eq!(categories.initialization(), fixture.expected);
    assert_eq!(categories.clone().initialization(), fixture.expected);
    assert_eq!(fixture.expected.len(), 98);
}

#[test]
fn conflicting_duplicate_or_invalid_rows_and_changed_proof_reject() {
    for mutation in 0..7 {
        let mut fixture = Fixture::new();
        match mutation {
            0 => fixture.content["categories"][1]["id"] = json!(10),
            1 => fixture.content["groups"][2]["category_id"] = json!(10),
            2 => fixture.content["categories"][0]["display"] = json!(-1),
            3 => fixture.content["groups"][0]["id"] = json!(0),
            4 => fixture.content["packet_sha256"] = json!("00".repeat(32)),
            5 => fixture.content["schema_version"] = json!(2),
            _ => fixture.content["categories"] = json!([]),
        }
        fixture.write();
        assert!(ItemCategories::load(&fixture.directory).is_err());
    }
}

#[test]
fn oversized_and_malformed_private_content_reject() {
    let fixture = Fixture::new();
    fs::write(
        fixture.directory.join("categories.json"),
        vec![b' '; 256 * 1024 + 1],
    )
    .unwrap();
    assert!(ItemCategories::load(&fixture.directory).is_err());
    fs::write(fixture.directory.join("categories.json"), b"invalid-json").unwrap();
    assert!(ItemCategories::load(&fixture.directory).is_err());
}

#[test]
#[ignore = "requires prepared private categories and unmodified community serializer output"]
fn private_community_serializer_packet_matches_exactly() {
    let directory =
        PathBuf::from(std::env::var_os("OSFR_GAMEPLAY_CONTENT").expect("private content required"));
    let oracle = PathBuf::from(
        std::env::var_os("OSFR_CATEGORY_ORACLE").expect("private C# oracle output required"),
    );
    let categories = ItemCategories::load(&directory).unwrap();
    let packet = categories.initialization();
    assert_eq!(packet, fs::read(oracle).unwrap());
    assert_eq!(&packet[..8], &[44, 0, 2, 0, 71, 0, 0, 0]);
    assert_eq!(packet.len(), 2675);
    println!("71 categories and74 group associations match unmodified C# serializer byte for byte");
}
