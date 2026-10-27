//! T456: the 53/5 pet list record, decoded with a reader written from the client's record reader
//! `00912cf0` (and trick element reader `008ce1a0`), not from our writer.
#[path = "support/json.rs"]
mod json;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;

use osfr_tools::pets::{PetBusy, PetContent, PetService};
use pet_inventory::InventoryFixture;
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

const CHAR_ITEM: i32 = 70352;
const CRAB_ITEM: i32 = 7885;

/// One record as the client reads it; field names are the record offsets.
#[derive(Debug)]
struct Record {
    id: u32,
    sex_04: bool,
    icon_08: u32,
    members_30: bool,
    tricks_34: Vec<[u32; 9]>,
    classes_e4: Vec<[u32; 3]>,
    name: String,
    name_text_1c: u32,
    tint_100: u32,
    image_set_fc: u32,
    vanity_104: bool,
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn u8(&mut self) -> u8 {
        let (v, rest) = self.0.split_first().unwrap();
        self.0 = rest;
        *v
    }
    fn u32(&mut self) -> u32 {
        let (v, rest) = self.0.split_at(4);
        self.0 = rest;
        u32::from_le_bytes(v.try_into().unwrap())
    }
    fn string(&mut self) -> String {
        let len = self.u32() as usize;
        let (v, rest) = self.0.split_at(len);
        self.0 = rest;
        String::from_utf8(v.to_vec()).unwrap()
    }
    /// `00912cf0`, field by field.
    fn record(&mut self) -> Record {
        let id = self.u32();
        let sex_04 = self.u8() != 0;
        let icon_08 = self.u32();
        for _ in 0..4 {
            self.u32(); // needs +0x20..+0x2c
        }
        let members_30 = self.u8() != 0;
        let tricks_34 = (0..self.u32())
            .map(|_| {
                let mut row = [0; 9];
                row[0] = self.u32(); // list key
                for field in &mut row[1..] {
                    *field = self.u32(); // 008ce1a0: eight u32
                }
                let _known = self.u8();
                row
            })
            .collect();
        for _ in 0..self.u32() {
            self.u32(); // equipped ids +0xbc
        }
        let classes_e4 = (0..self.u32())
            .map(|_| [self.u32(), self.u32(), self.u32()])
            .collect();
        let name = self.string();
        let name_text_1c = self.u32();
        let tint_100 = self.u32();
        let _alias_108 = self.string();
        let image_set_fc = self.u32();
        let vanity_104 = self.u8() != 0;
        let _attachment_118 = self.u32();
        let _tutorial_11c = self.u8();
        for _ in 0..12 {
            self.u32(); // 4 favourites, 8 utilities
        }
        Record {
            id,
            sex_04,
            icon_08,
            members_30,
            tricks_34,
            classes_e4,
            name,
            name_text_1c,
            tint_100,
            image_set_fc,
            vanity_104,
        }
    }
}

/// The 53/5 list: u16 53, u8 5, count, (u32 key, record)*, then the active pair.
fn list(packet: &[u8]) -> Vec<Record> {
    assert_eq!(packet[..3], [53, 0, 5]);
    let mut r = Reader(&packet[3..]);
    let records = (0..r.u32())
        .map(|_| {
            r.u32();
            r.record()
        })
        .collect();
    r.u32();
    r.u32();
    r.u32(); // active pet id, actor guid (u64)
    assert!(r.0.is_empty(), "{} trailing bytes", r.0.len());
    records
}

fn pets_json() -> Value {
    serde_json::from_str(include_str!("../data/content/pets.json")).unwrap()
}

fn row<'a>(content: &'a Value, table: &str, key: &str, id: i64) -> &'a Value {
    content[table]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[key].as_i64() == Some(id))
        .unwrap()
}

fn locale() -> BTreeSet<u64> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/reports/locale-id-to-text.json");
    json::read(&path)
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_u64().unwrap())
        .collect()
}

/// Adopt one item through the real 53/32 path and return the owner's last 53/5 record.
fn adopt(item: i32, tint: i32) -> (PetService, Record) {
    let mut fixture = InventoryFixture::with_tinted_items(&[(10, item, 1, tint)]);
    let mut service = PetService::new(PetContent::bundled().unwrap()).unwrap();
    let wire = service
        .handle_with_inventory(
            &pet_wire::summon_by_item(item as u32, tint as u32),
            33,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            0,
        )
        .unwrap()
        .unwrap();
    assert!(wire.changed, "{:?}", wire.notice);
    let packet = wire
        .private
        .iter()
        .rev()
        .find(|p| p.starts_with(&[53, 0, 5]))
        .unwrap();
    let mut records = list(packet);
    (service, records.pop().unwrap())
}

#[test]
fn a_tinted_char_lists_its_icon_image_name_tint_and_named_tricks() {
    let content = pets_json();
    let item = row(&content, "items", "item_id", CHAR_ITEM.into());
    let model = row(
        &content,
        "models",
        "model_id",
        item["model_id"].as_i64().unwrap(),
    );
    let (_, record) = adopt(CHAR_ITEM, 282);
    assert_ne!(
        record.icon_08, 1_000_015,
        "+0x08 is the icon, not the species id"
    );
    assert_eq!(
        i64::from(record.icon_08),
        model["icon_id"].as_i64().unwrap()
    );
    assert_eq!(
        (record.image_set_fc, record.name_text_1c, record.tint_100),
        (5606, 24435, 282)
    );
    assert_eq!(record.members_30, item["members_only"].as_bool().unwrap());
    assert_eq!(record.vanity_104, model["vanity"].as_bool().unwrap());
    let names = locale();
    assert!(!record.tricks_34.is_empty());
    for trick in &record.tricks_34 {
        assert_eq!(trick[0], trick[1], "key and id");
        assert!(
            names.contains(&u64::from(trick[3])),
            "trick {} name {}",
            trick[1],
            trick[3]
        );
    }
}

#[test]
fn a_dog_lists_tricks_11_to_17_with_names_and_its_item_classes() {
    let content = pets_json();
    let model = row(&content, "models", "model_id", 364);
    let mut service = PetService::new(PetContent::bundled().unwrap()).unwrap();
    let definition = PetContent::bundled()
        .unwrap()
        .definition(1_000_015)
        .unwrap();
    let mut dog = definition.clone();
    dog.model = model["adr"].as_str().unwrap().into();
    let id = service.adopt_vendor_grant(1, &dog, 0).unwrap();
    let records = list(&service.list_packet());
    let record = records.iter().find(|r| r.id == id).unwrap();
    let ids: BTreeSet<u32> = record.tricks_34.iter().map(|t| t[1]).collect();
    assert!((11..=17).all(|id| ids.contains(&id)), "{ids:?}");
    let names = locale();
    assert!(
        record
            .tricks_34
            .iter()
            .all(|t| names.contains(&u64::from(t[3])))
    );
    assert_eq!(
        record.classes_e4.len(),
        model["item_classes"].as_array().unwrap().len()
    );
}

#[test]
fn a_crab_record_is_vanity_with_no_tricks() {
    let (_, record) = adopt(CRAB_ITEM, 0);
    assert!(record.vanity_104);
    assert!(record.tricks_34.is_empty());
}

/// F767: the sex column is the client's bool: Models.txt GENDER 2 (female) is true, 1 (male) and
/// none false. York (item 23930, model 364) is the recorded sample's female pet.
#[test]
fn the_sex_column_is_true_only_for_a_female_model() {
    assert!(!adopt(CHAR_ITEM, 0).1.sex_04, "Char, model 1756, GENDER 1");
    assert!(!adopt(CRAB_ITEM, 0).1.sex_04, "Crab, model 1667, no GENDER");
    assert!(adopt(23_930, 0).1.sex_04, "York, model 364, GENDER 2");
}

#[test]
fn an_old_roster_without_tints_loads_and_lists() {
    let content = PetContent::bundled().unwrap();
    let mut service = PetService::new(content.clone()).unwrap();
    let id = service
        .adopt_vendor_grant(1, &content.definition(1_000_015).unwrap(), 0)
        .unwrap();
    let mut document = service.document_value().unwrap();
    // A pre-T456 save: no `tint`, and learned trick 11.
    let pet = &mut document["pets"][0];
    pet.as_object_mut().unwrap().remove("tint");
    pet["learned_tricks"] = serde_json::json!([11]);
    let loaded = PetService::from_document(content, Some(document)).unwrap();
    let records = list(&loaded.list_packet());
    assert!(records.iter().any(|r| r.id == id && r.name.is_empty()));
}

#[test]
fn an_accepted_rename_reaches_viewers_and_a_rejection_only_the_owner() {
    let (mut service, _) = adopt(CHAR_ITEM, 0);
    let rename = |name: &str| {
        [
            vec![53, 0, 6],
            (name.len() as u32).to_le_bytes().to_vec(),
            name.as_bytes().to_vec(),
        ]
        .concat()
    };
    let accepted = service
        .handle(&rename("Admiral Ace"), 33, [0.0; 3], PetBusy::default(), 1)
        .unwrap()
        .unwrap();
    assert!(
        accepted.shared.iter().any(|p| p.starts_with(&[53, 0, 10])),
        "viewers get 53/10"
    );
    assert!(accepted.private.iter().any(|p| p.starts_with(&[53, 0, 10])));
    let rejected = service
        .handle(&rename("Qwxz Zyxq"), 33, [0.0; 3], PetBusy::default(), 2)
        .unwrap()
        .unwrap();
    assert!(rejected.shared.is_empty(), "a rejection stays private");
}

#[test]
fn a_plain_summon_ends_with_53_9_then_the_roster() {
    let (mut service, _) = adopt(CHAR_ITEM, 0);
    let dismiss = service
        .handle(
            &[53, 0, 4, 0, 0, 0, 0, 0],
            33,
            [0.0; 3],
            PetBusy::default(),
            1,
        )
        .unwrap()
        .unwrap();
    assert!(dismiss.private.iter().any(|p| p.starts_with(&[53, 0, 9])));
    let wire = service
        .handle(&pet_wire::summon(1), 33, [0.0; 3], PetBusy::default(), 2)
        .unwrap()
        .unwrap();
    let tail: Vec<_> = wire
        .private
        .iter()
        .rev()
        .take(2)
        .map(|p| p[..3].to_vec())
        .collect();
    assert_eq!(
        tail,
        vec![vec![53, 0, 5], vec![53, 0, 9]],
        "53/9 then 53/5 at the end"
    );
}
