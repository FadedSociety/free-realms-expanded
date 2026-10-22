//! T449: gates on the committed coin -> collection entry map. Only the canaries and the totals
//! are pinned; the 83 id-order pairs stay tunable one row at a time.

use osfr_tools::collection_catalogue::CollectionCatalogueContent;
use osfr_tools::collection_coins::{
    CoinsFile, CollectionCoins, FILE_NAME, Pairing, SCHEMA_VERSION, Tier,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn catalogue() -> CollectionCatalogueContent {
    CollectionCatalogueContent::load(&content_dir())
        .unwrap()
        .expect("committed catalogue")
}

fn committed_text() -> String {
    fs::read_to_string(content_dir().join(FILE_NAME)).unwrap()
}

fn committed() -> CollectionCoins {
    CollectionCoins::load(&content_dir(), &catalogue())
        .unwrap()
        .expect("committed coins")
}

/// Loads `text` from a private temp directory (or an empty one when `None`).
fn load_text(name: &str, text: Option<&str>) -> io::Result<Option<CollectionCoins>> {
    let dir = std::env::temp_dir().join(format!("osfr-t449-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    if let Some(text) = text {
        fs::write(dir.join(FILE_NAME), text).unwrap();
    }
    let result = CollectionCoins::load(&dir, &catalogue());
    fs::remove_dir_all(&dir).unwrap();
    result
}

/// Edits a copy of the committed file and returns the loader's refusal.
fn refusal(name: &str, edit: impl FnOnce(&mut Value)) -> String {
    let mut document: Value = serde_json::from_str(&committed_text()).unwrap();
    edit(&mut document);
    match load_text(name, Some(&serde_json::to_string(&document).unwrap())) {
        Ok(_) => panic!("{name}: the loader accepted it"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn every_placed_coin_is_bound_once_with_its_own_tier() {
    let coins = committed();
    assert_eq!(coins.iter().count(), 118);
    let spawns: Value = serde_json::from_str(
        &fs::read_to_string(content_dir().join("npc_spawns_extra.json")).unwrap(),
    )
    .unwrap();
    let rows: BTreeMap<u64, &Value> = spawns["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["name_string_id"] == 386170 || row["name_string_id"] == 4412)
        .map(|row| (row["spawn_guid"].as_u64().unwrap(), row))
        .collect();
    assert_eq!(
        rows.keys().copied().collect::<Vec<_>>(),
        coins.iter().map(|(guid, _)| guid).collect::<Vec<_>>(),
        "every coin row is bound, and nothing else"
    );
    let mut entries = BTreeSet::new();
    for (guid, coin) in coins.iter() {
        let row = rows[&guid];
        assert_eq!(row["legacy_npc_id"], json!(coin.legacy_npc_id), "{guid}");
        let tier = if row["name_string_id"] == 386170 {
            Tier::Rare
        } else {
            Tier::Elite
        };
        assert_eq!(coin.tier, tier, "{guid}");
        assert!(
            entries.insert(coin.entry_id),
            "entry {} twice",
            coin.entry_id
        );
        if let Some(id) = coin.legacy_npc_id {
            let id = format!("Id={id}");
            assert!(coin.source[0].evidence.contains(&id), "{guid}: {coin:?}");
        }
        let checked = coin.source.iter().any(|source| source.kind == "playtest");
        assert_eq!(checked, coin.pairing == Pairing::Playtest, "{guid}");
    }
    let playtest = coins
        .iter()
        .filter(|(_, coin)| coin.pairing == Pairing::Playtest)
        .count();
    assert_eq!(playtest, 35);
    // Tunable (R): a sanity bound only, so retuning the reach stays a one-value data edit.
    assert!((1.0..=50.0).contains(&coins.pickup_range()));
}

#[test]
fn canary_pairs_hold() {
    let coins = committed();
    for (legacy, collection, entry, pairing) in [
        (1946, 267, 1937, Pairing::Playtest),
        (2097, 267, 1958, Pairing::Playtest), // re-paired in Lxzxrus 301380a
        (2112, 288, 2041, Pairing::Playtest), // Lxzxrus 65ec688 override
        (4421, 269, 1966, Pairing::Playtest), // Lxzxrus 65ec688 override
        (4438, 284, 2015, Pairing::IdOrder),  // stays Elite; Sanctuary e3deee7 renamed it
        (4445, 311, 2188, Pairing::IdOrder),  // Rare-named Sunstone: Sanctuary's swap undone
        (4447, 312, 2196, Pairing::IdOrder),  // Elite-named Sunstone
    ] {
        let coin = coins.by_guid(100_000_000_000 + legacy).expect("coin");
        assert_eq!(
            (
                coin.legacy_npc_id,
                coin.collection_id,
                coin.entry_id,
                coin.pairing
            ),
            (Some(legacy as u32), collection, entry, pairing)
        );
    }
}

#[test]
fn gaps_are_every_unbound_exploration_entry() {
    let catalogue = catalogue();
    let coins = committed();
    let bound: BTreeSet<i32> = coins.iter().map(|(_, coin)| coin.entry_id).collect();
    let unbound: Vec<(i32, i32)> = catalogue
        .collections()
        .iter()
        .filter(|collection| Tier::of_collection(collection.id).is_some())
        .flat_map(|collection| collection.entries.iter().map(|e| (collection.id, e.id)))
        .filter(|(_, entry)| !bound.contains(entry))
        .collect();
    let file: CoinsFile = serde_json::from_str(&committed_text()).unwrap();
    assert_eq!(file.schema_version, SCHEMA_VERSION);
    let gaps: Vec<(i32, i32)> = file
        .gaps
        .entries_without_coin
        .iter()
        .map(|gap| (gap.collection_id, gap.entry_id))
        .collect();
    assert_eq!(gaps, unbound);
    let ids: Vec<i32> = gaps.iter().map(|&(_, entry)| entry).collect();
    assert_eq!(ids, [1949, 1950, 1954, 1955, 1985, 2038, 2195, 2203]);
    for gap in &file.gaps.entries_without_coin {
        let expected = match gap.entry_id {
            2038 | 2195 | 2203 => "placed by coin-spawn-additions: ",
            1949 => "no coin left for it: ", // Sanctuary pairs Elite coin 4438 here as Rare
            _ => "no coin in any source",
        };
        assert!(gap.reason.starts_with(expected), "{gap:?}");
    }
    let [note] = file.gaps.notes.as_slice() else {
        panic!("one note: {:?}", file.gaps.notes)
    };
    assert_eq!((note.entry_id, note.spawn_guid), (2039, 100_000_002_115));
    assert!(note.note.contains("cannot be reached on foot"), "{note:?}");
}

#[test]
fn loader_refuses_bad_files() {
    assert!(load_text("absent", None).unwrap().is_none());
    assert!(
        load_text("control", Some(&committed_text()))
            .unwrap()
            .is_some()
    );
    let duplicate_guid = refusal("duplicate-guid", |doc| {
        doc["records"][1]["spawn_guid"] = doc["records"][0]["spawn_guid"].clone();
    });
    assert!(duplicate_guid.contains("listed twice"), "{duplicate_guid}");
    let duplicate_entry = refusal("duplicate-entry", |doc| {
        let first = doc["records"][0].clone();
        for key in ["tier", "collection_id", "entry_id"] {
            doc["records"][1][key] = first[key].clone();
        }
    });
    assert!(duplicate_entry.contains("is bound to"), "{duplicate_entry}");
    let unknown = refusal("unknown-entry", |doc| {
        doc["records"][0]["entry_id"] = json!(999_999)
    });
    assert!(unknown.contains("not in the catalogue"), "{unknown}");
    let owner = refusal("wrong-owner", |doc| {
        doc["records"][0]["collection_id"] = json!(268)
    });
    assert!(owner.contains("belongs to"), "{owner}");
    let tier = refusal("tier-mismatch", |doc| {
        let flipped = if doc["records"][0]["tier"] == "rare" {
            "elite"
        } else {
            "rare"
        };
        doc["records"][0]["tier"] = json!(flipped);
    });
    assert!(tier.contains("coin in collection"), "{tier}");
    let field = refusal("unknown-field", |doc| doc["records"][0]["extra"] = json!(1));
    assert!(field.contains("unknown field"), "{field}");
    // 1e39 overflows f32 to infinity.
    for (name, range) in [
        ("range-0", json!(0.0)),
        ("range-negative", json!(-1.0)),
        ("range-inf", json!(1e39)),
    ] {
        let error = refusal(name, |doc| doc["pickup_range"] = range);
        assert!(error.contains("pickup_range"), "{name}: {error}");
    }
    // JSON has no NaN: json! writes it as null, which fails to parse before the bound check.
    let nan = refusal("range-nan", |doc| doc["pickup_range"] = json!(f64::NAN));
    assert!(nan.contains("null"), "{nan}");
}
