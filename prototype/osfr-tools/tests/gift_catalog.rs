//! gifts-codes: the shipped gift catalog (`data/content/gift_catalog.json`). Every row resolves
//! to client item rows (each item's pinned `name_id` is that row's NameId in the item table the
//! loader reads), every source record is a row or an unresolved entry with its reason, and the
//! owner's defaults hold: per account, the original codes kept, event rows on their event, the
//! rows the wiki says kept reappearing every 24 hours, the wiki's five tradeable offers.
//! Reads only repository files (no lab fixture), so it runs anywhere.
use osfr_tools::gifts::{Catalog, Per, Source};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn catalog() -> Catalog {
    Catalog::load(&root().join("data/content")).unwrap()
}

/// Item id -> NameId over every definition the inventory loads: the captured `inventory.json`
/// rows first (NameId is the second dword of each serialized definition), then the item table
/// and its supplement for the rest, in the inventory's own precedence.
fn name_ids() -> BTreeMap<i64, i64> {
    let mut out = BTreeMap::new();
    let path = root().join("data/release/inventory.json");
    let captured: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    for definition in captured["definitions"].as_array().unwrap() {
        let hex = definition["serialized_hex"].as_str().unwrap();
        let byte = |i: usize| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap();
        let name_id = i32::from_le_bytes([byte(4), byte(5), byte(6), byte(7)]);
        out.insert(definition["id"].as_i64().unwrap(), i64::from(name_id));
    }
    for file in ["item-table.json", "item-table-supplement.json"] {
        let path = root().join("data/release").join(file);
        let table: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let column = table["columns"].as_array().unwrap().iter().position(|c| c == "NameId").unwrap();
        for row in table["rows"].as_array().unwrap() {
            out.entry(row[0].as_i64().unwrap()).or_insert(row[1][column].as_i64().unwrap());
        }
    }
    out
}

#[test]
fn every_row_resolves_to_client_items_and_the_rest_say_why() {
    let catalog = catalog();
    let names = name_ids();
    for gift in &catalog.gifts {
        assert!(!gift.evidence.is_empty(), "{}: evidence", gift.key);
        for item in &gift.items {
            assert_eq!(
                names.get(&i64::from(item.item_id)),
                Some(&i64::from(item.name_id)),
                "{}: item {} must be the client row whose name was matched",
                gift.key,
                item.item_id
            );
        }
    }
    // Owner 2026-11-11: retired items stay available, so what cannot be given yet is missing work
    // (an item row to build, a source to find), never n/a. Only the TCG booster is the card game's.
    for row in &catalog.unresolved {
        assert!(!row.evidence.is_empty(), "{}", row.name);
        assert!(
            row.reason.starts_with("missing: ") || row.reason.starts_with("the card game's"),
            "{}: {}",
            row.name,
            row.reason
        );
    }
    let count = |source: Source| catalog.gifts.iter().filter(|g| g.source == source).count();
    let left = |source: Source| catalog.unresolved.iter().filter(|u| u.source == source).count();
    // Free Items (FR) lists 53 offers, every one a row now that the retired items have item rows
    // (L1170). Both Free Item Codes tables have 50 rows: the two with no published code ('Email
    // Code', a removed link) are standing offers, their notes naming the table's entry, and the TCG
    // booster is the card game's.
    assert_eq!((count(Source::Profile), left(Source::Profile)), (53, 0));
    assert_eq!((count(Source::Code), left(Source::Code)), (47, 1));
    let no_code: BTreeSet<&str> = (catalog.gifts.iter())
        .filter(|g| g.note.contains("no published code ('"))
        .map(|g| g.name.as_str())
        .collect();
    assert_eq!(no_code, BTreeSet::from(["Hot-To-Trot Tots", "Triceratops"]));
    assert!(catalog.gifts.iter().filter(|g| g.note.contains("no published code ('")).all(|g| g.source == Source::Promotional));
    assert_eq!(count(Source::Code) + left(Source::Code) + no_code.len(), 50);
    // fandom's Promotional, Code Flair and Code Weapons items no other list names, and the tie-ins,
    // login gifts and member packs the event audits list. What is left has no source naming it.
    assert_eq!((count(Source::Promotional), left(Source::Promotional)), (26, 1));
    assert_eq!((count(Source::Member), left(Source::Member)), (10, 1));
    let unique: BTreeSet<i32> = (catalog.gifts.iter())
        .flat_map(|gift| gift.items.iter().map(|item| item.item_id))
        .collect();
    assert_eq!(unique.len(), 164, "client items the catalog gives");
}

#[test]
fn the_defaults_follow_the_wiki_and_the_owner() {
    let catalog = catalog();
    let with = |pick: &dyn Fn(&osfr_tools::gifts::Gift) -> bool| -> BTreeSet<String> {
        catalog.gifts.iter().filter(|g| pick(g)).map(|g| g.name.clone()).collect()
    };
    assert!(catalog.gifts.iter().all(|g| g.per == Per::Account && g.enabled));
    let repeating = with(&|g| g.repeatable);
    let expected: BTreeSet<String> = [
        "Firefrog Legs", "Graveyard Flambe", "Gumball Machine", "Large Health Potion",
        "Prospector's Scroll", "Standard Hanging Plant", "Tiki Torch",
    ]
    .map(String::from)
    .into();
    assert_eq!(repeating, expected, "the rows that kept reappearing");
    assert!(catalog.gifts.iter().filter(|g| g.repeatable).all(|g| g.repeat_hours == Some(24)));
    let tradeable = with(&|g| g.source == Source::Profile && g.tradeable);
    let expected: BTreeSet<String> =
        ["Fuzzy Squirrel", "Gumball Machine", "Standard Hanging Plant", "Tiki Torch", "Turtle"]
            .map(String::from)
            .into();
    assert_eq!(tradeable, expected);
    let halloween = with(&|g| g.live_event.as_deref() == Some("halloween"));
    assert_eq!(halloween.len(), 7, "{halloween:?}");
    assert!(halloween.contains("Witch Pail") && halloween.contains("Super Spooktacular 2009 Member Pack"));
    assert!(halloween.contains("October 2013 Member Pack"), "Pinkeye and Toeny");
    let snow_days = with(&|g| g.live_event.as_deref() == Some("snow_days"));
    assert_eq!(snow_days.len(), 8, "{snow_days:?}");
    let birthday = with(&|g| g.live_event.as_deref() == Some("birthday_bash"));
    assert_eq!(birthday.len(), 3, "{birthday:?}");
    // Every member pack sits on its event when it had one, and pays its coins.
    let packs = with(&|g| g.source == Source::Member);
    assert_eq!(packs.len(), 10, "{packs:?}");
    let december = catalog.get("member-december-2011").unwrap();
    assert_eq!((december.coins, december.live_event.as_deref()), (10_000, Some("snow_days")));
    assert_eq!(december.items.len(), 6, "the Snow Globe and the pack's five other pieces");
    assert!(december.items.iter().any(|i| i.item_id == 27943 && i.count == 2), "two Snowhill Lodge Chairs");
    // A pack is its pieces, never the box row (the Spring Chicken Pack's box has no contents).
    let chicken = catalog.get("member-spring-chicken-pack").unwrap();
    let pieces: Vec<i32> = chicken.items.iter().map(|i| i.item_id).collect();
    assert_eq!((pieces, chicken.live_event.as_deref()), (vec![911275, 911276, 911308], Some("birthday_bash")));
    // The limited-stock fields are in every row (unset: no limit), for the Studio to show.
    assert!(catalog.gifts.iter().all(|g| g.stock_per_event.is_none() && g.per_player.is_none() && g.stock_group.is_none()));
    // Every event key is a calendar event.
    let calendar: Value =
        serde_json::from_slice(&std::fs::read(root().join("data/content/event_calendar.json")).unwrap()).unwrap();
    let keys: BTreeSet<&str> = calendar["events"].as_array().unwrap().iter().map(|e| e["key"].as_str().unwrap()).collect();
    assert!(catalog.gifts.iter().filter_map(|g| g.live_event.as_deref()).all(|key| keys.contains(key)));
    // The original codes work again, each for its own item.
    for (code, name) in [
        ("KN78CH2", "Striped Shoes"),
        ("FACEBOOKFANS01", "Veggie Surprise"),
        ("GETSPOOKY", "Robgoblin Costume"),
        ("SNOWDAYS", "Snowman"),
        ("ROSESFORMOM", "Mother's Day Bouquet"),
    ] {
        assert_eq!(catalog.by_code(code).map(|g| g.name.as_str()), Some(name), "{code}");
    }
    assert_eq!(catalog.gifts.iter().filter(|g| g.code.is_some()).count(), 47, "published codes");
    // Where the wiki's word and the client item's own trade flag disagree, the row says so (the
    // trade window follows the client's flag): the five spheres of September 2010 and the two
    // poster rows B3 built tradeable.
    let disagree = with(&|g| g.note.contains("the trade window follows the client"));
    assert_eq!(disagree.len(), 7, "{disagree:?}");
    assert_eq!(disagree.iter().filter(|name| name.ends_with("Sphere")).count(), 5);
    assert!(disagree.contains("Hoodwinked Poster Pack") && disagree.contains("ThunderCats Poster"));
}
