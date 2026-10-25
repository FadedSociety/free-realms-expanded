//! T447: gates on the committed collection catalogue (no private data), plus its generator and
//! drift oracle (ignored; needs the private login template).

use osfr_tools::collection_catalogue::{
    CatalogueFile, Collection, CollectionCatalogueContent, Entry, FILE_NAME, Gaps, ItemLink,
    ItemLinkNote, Reward, RewardItem, RewardToken, RewardXp, SCHEMA_VERSION,
};
use osfr_tools::inventory::Inventory;
use osfr_tools::startup_selfdata::{CollectionRecord, parse_self};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn committed_text() -> String {
    fs::read_to_string(content_dir().join(FILE_NAME)).unwrap()
}

fn committed() -> CollectionCatalogueContent {
    CollectionCatalogueContent::load(&content_dir())
        .unwrap()
        .expect("committed catalogue")
}

/// Loads `text` from a private temp directory (or an empty one when `None`).
fn load_text(name: &str, text: Option<&str>) -> io::Result<Option<CollectionCatalogueContent>> {
    let dir = std::env::temp_dir().join(format!("osfr-t447-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    if let Some(text) = text {
        fs::write(dir.join(FILE_NAME), text).unwrap();
    }
    let result = CollectionCatalogueContent::load(&dir);
    fs::remove_dir_all(&dir).unwrap();
    result
}

/// Edits a copy of the committed file and returns the loader's refusal.
fn refusal(name: &str, edit: impl FnOnce(&mut Value)) -> String {
    let mut document: Value = serde_json::from_str(&committed_text()).unwrap();
    edit(&mut document);
    let text = serde_json::to_string(&document).unwrap();
    match load_text(name, Some(&text)) {
        Ok(_) => panic!("{name}: the loader accepted it"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn committed_catalogue_holds_every_captured_collection() {
    let content = committed();
    assert_eq!(content.catalogue().len(), 253);
    let entries: Vec<(&Collection, &Entry)> = content
        .collections()
        .iter()
        .flat_map(|collection| {
            collection
                .entries
                .iter()
                .map(move |entry| (collection, entry))
        })
        .collect();
    assert_eq!(entries.len(), 1815);
    // The load went through `CollectionCatalogue::insert`, which refuses a category `category()`
    // does not know (see `loader_refuses_bad_files`).
    assert!(
        content
            .collections()
            .iter()
            .all(|c| (1..=10).contains(&c.category_id))
    );
    assert!(
        entries
            .iter()
            .filter(|(collection, _)| collection.category_id == 1)
            .all(|(_, entry)| entry.item_definition_id.is_none())
    );
    let mut links = BTreeMap::new();
    let mut notes = BTreeMap::new();
    for (_, entry) in &entries {
        assert_eq!(
            entry.item_definition_id.is_some(),
            entry.item_link.is_some()
        );
        assert_eq!(entry.item_link.is_none(), entry.item_link_note.is_some());
        if let Some(link) = entry.item_link {
            *links.entry(link).or_insert(0) += 1;
        }
        if let Some(note) = entry.item_link_note {
            *notes.entry(note).or_insert(0) += 1;
        }
    }
    assert_eq!(
        notes,
        BTreeMap::from([
            (ItemLinkNote::NoItem, 299),
            (ItemLinkNote::AmbiguousName, 100)
        ])
    );
    assert_eq!(
        links,
        BTreeMap::from([
            (ItemLink::Sanctuary, 1087),
            (ItemLink::LegacyUniqueName, 329)
        ])
    );
    let file: CatalogueFile = serde_json::from_str(&committed_text()).unwrap();
    assert_eq!(file.gaps.null_item_links, notes);
    assert_eq!(
        file.gaps.reward_items_missing_from_legacy,
        [12410, 38816, 38817]
    );
}

#[test]
fn collection_10_matches_the_capture() {
    let content = committed();
    let collection = content.collection(10).unwrap();
    assert_eq!((collection.name_id, collection.category_id), (17054, 3));
    let rows: Vec<(i32, u32)> = collection.entries.iter().map(|e| (e.id, e.index)).collect();
    assert_eq!(rows, (41..=48).zip(1..=8).collect::<Vec<_>>());
    let (entry, owner) = content.entry(41).unwrap();
    assert_eq!(owner.id, 10);
    assert_eq!(
        entry,
        &Entry {
            id: 41,
            index: 1,
            name_id: 17056,
            icon_id: 2124,
            icon_tint_id: 99,
            item_definition_id: Some(11081),
            item_link: Some(ItemLink::Sanctuary),
            item_link_note: None,
        }
    );
    assert_eq!(
        collection.reward,
        Reward {
            coins: 9,
            xp: vec![RewardXp {
                profile_id: 11,
                amount: 50
            }],
            ..Reward::default()
        }
    );
}

#[test]
fn exploration_sets_carry_their_captured_rows() {
    let content = committed();
    let ids = |collection: &Collection| collection.entries.iter().map(|e| e.id).collect::<Vec<_>>();
    let set_265 = content.collection(265).unwrap();
    assert_eq!(ids(set_265), [1935, 1947, 1948, 1949, 1950]);
    assert_eq!(
        set_265.reward.items,
        [RewardItem {
            item_definition_id: Some(18040),
            count: 1,
            female_item_definition_id: Some(16529),
            captured_item_definition_id: None,
        }]
    );
    let set_284 = &content.collection(284).unwrap().reward;
    assert_eq!(set_284.items[0].item_definition_id, Some(70341));
    assert_eq!(
        set_284.tokens,
        [RewardToken {
            token_id: 520,
            count: 1
        }]
    );
    assert_eq!(set_284.collection_add, [309]);
    assert_eq!(content.collection(311).unwrap().name_id, 3795);
    assert_eq!(content.collection(312).unwrap().name_id, 3797);
    assert_eq!(ids(content.collection(313).unwrap()), [2204, 2205, 2206]);
}

#[test]
fn entry_for_item_answers_only_unique_links() {
    let content = committed();
    assert_eq!(content.entry_for_item(11081), Some(41));
    let linking_2868 = content
        .collections()
        .iter()
        .flat_map(|c| &c.entries)
        .filter(|e| e.item_definition_id == Some(2868))
        .count();
    assert_eq!(linking_2868, 2, "2868 is linked by two entries");
    assert_eq!(content.entry_for_item(2868), None);
    assert_eq!(content.entry_for_item(-1), None);
}

/// T630: every item the catalogue binds (entry links, reward items, female variants) has a
/// definition in the runtime item catalogue, so the playtest bundle's item gate passes.
#[test]
fn every_bound_item_has_a_runtime_definition() {
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports/round70-native-20261016T061000Z/content");
    let inventory = Inventory::load(&runtime, None).expect("the round70 item catalogue loads");
    let content = committed();
    let undefined: BTreeSet<i32> = content
        .collections()
        .iter()
        .flat_map(|c| {
            let entries = c.entries.iter().map(|e| e.item_definition_id);
            let rewards = c
                .reward
                .items
                .iter()
                .flat_map(|item| [item.item_definition_id, item.female_item_definition_id]);
            entries.chain(rewards).flatten()
        })
        .filter(|&id| !inventory.has_definition(id))
        .collect();
    assert!(undefined.is_empty(), "bound but undefined: {undefined:?}");

    // The three captured ids no definition source has: the "Skeletal T-Rex" row (NameId 19042)
    // grants 79446, ride 13575's item; the "Sunstone Street Lamp" and "Skullz Tent" rows grant
    // no item, and their collections and tokens stand.
    let reward = |id: i32| content.collection(id).unwrap().reward.clone();
    let item = |granted, captured| RewardItem {
        item_definition_id: granted,
        count: 1,
        female_item_definition_id: None,
        captured_item_definition_id: Some(captured),
    };
    assert_eq!(reward(313).items, [item(Some(79446), 12410)]);
    assert_eq!(reward(313).coins, 12);
    for (id, captured, token) in [(311, 38817, 654), (312, 38816, 655)] {
        assert_eq!(reward(id).items, [item(None, captured)]);
        assert_eq!(
            reward(id).tokens,
            [RewardToken {
                token_id: token,
                count: 1
            }]
        );
    }
}

#[test]
fn loader_refuses_bad_files() {
    assert!(load_text("absent", None).unwrap().is_none());
    assert!(
        load_text("control", Some(&committed_text()))
            .unwrap()
            .is_some()
    );
    let first_entry = |doc: &mut Value| doc["collections"][0]["entries"][0].clone();
    let duplicate = refusal("duplicate-entry", |doc| {
        let id = first_entry(doc)["id"].clone();
        doc["collections"][1]["entries"][0]["id"] = id;
    });
    assert!(duplicate.contains("listed twice"), "{duplicate}");
    for index in [0, 9] {
        let error = refusal(&format!("index-{index}"), |doc| {
            doc["collections"][0]["entries"][0]["index"] = json!(index);
        });
        assert!(error.contains("collection 10: entry indexes"), "{error}");
    }
    let category = refusal("category-11", |doc| {
        doc["collections"][0]["category_id"] = json!(11)
    });
    assert!(
        category.contains("unknown collection category 11"),
        "{category}"
    );
    let unknown = refusal("unknown-field", |doc| {
        doc["collections"][0]["extra"] = json!(1)
    });
    assert!(unknown.contains("unknown field"), "{unknown}");
    for (name, id, pointer, value) in [
        ("coins", 10, "/reward/coins", 100_001),
        ("xp", 10, "/reward/xp/0/amount", 0),
        ("item-count", 265, "/reward/items/0/count", 101),
        ("token-count", 284, "/reward/tokens/0/count", 0),
    ] {
        let error = refusal(name, |doc| {
            let collections = doc["collections"].as_array_mut().unwrap();
            let collection = collections.iter_mut().find(|c| c["id"] == id).unwrap();
            *collection.pointer_mut(pointer).unwrap() = json!(value);
        });
        let expected = format!("collection {id}: reward value out of bounds");
        assert!(error.contains(&expected), "{name}: {error}");
    }
    let oversized = committed_text() + &" ".repeat(1 << 20);
    let error = load_text("oversized", Some(&oversized))
        .err()
        .expect("oversized refused");
    assert!(error.to_string().contains("over 1 MiB"), "{error}");
}

/// T522: a target that becomes a source of another accepted target is a deeper chain; its pair is
/// dropped at load, so a completion cascade stops at two levels and cannot loop.
#[test]
fn meta_map_drops_a_chain_whose_source_is_another_target() {
    let mut document: Value = serde_json::from_str(&committed_text()).unwrap();
    let collections = document["collections"].as_array_mut().unwrap();
    let mut target = collections
        .iter()
        .find(|collection| collection["id"] == 309)
        .cloned()
        .expect("309");
    target["id"] = json!(10_000);
    target["name_id"] = json!(419_999);
    target["entries"] = json!([{
        "id": 21_000, "index": 1, "name_id": 419_914, "icon_id": 2236,
        "icon_tint_id": 0, "item_definition_id": null, "item_link": null
    }]);
    target["reward"] = json!({
        "coins": 0, "items": [], "xp": [], "tokens": [], "collection_add": [], "quest_add": []
    });
    collections.push(target);
    collections
        .iter_mut()
        .find(|collection| collection["id"] == 309)
        .unwrap()["reward"]["collection_add"] = json!([10_000]);
    let text = serde_json::to_string(&document).unwrap();
    let content = load_text("meta-chain", Some(&text))
        .unwrap()
        .expect("the chain still loads");
    assert_eq!(content.meta_pairs().len(), 14);
    assert_eq!(content.meta_entry(284), Some(2170));
    assert_eq!(content.meta_entry(309), None);
}

// Generator: the join inputs are AGPL data rows from pinned reference checkouts (no code copied).
const LEGACY_ITEMS: (&str, &str) = (
    "FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json",
    "9e082025391177d5a33ee18c714ddd06b457a695",
);
const SANCTUARY: (&str, &str) = (
    "Sanctuary-main-20261019/src/Resources/Collections.json",
    "10f129e7f3e2094304d20ccee6e3f7136a5e889d",
);
const LXZXRUS: (&str, &str) = (
    "Lxzxrus-Free-Realms-Server/src/Resources/Collections.json",
    "37ef17587d43e9b33bfb10e5484518f61a82c6be",
);

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LegacyItem {
    id: i32,
    #[serde(default)]
    name_id: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SanctuaryCollection {
    entries: Vec<SanctuaryEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SanctuaryEntry {
    id: i32,
    name_id: i32,
    #[serde(default)]
    item_definition_id: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LxzxrusCollection {
    id: i32,
    #[serde(default)]
    reward_item_ids: Vec<i32>,
    #[serde(default)]
    reward_item_ids_girls: Vec<i32>,
}

type Link = (Option<i32>, Option<ItemLink>, Option<ItemLinkNote>);

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn dwords<const N: usize>(bytes: &[u8]) -> [u32; N] {
    std::array::from_fn(|i| u32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap()))
}

fn int(value: u32) -> i32 {
    i32::try_from(value).expect("captured id fits i32")
}

fn join_input<T: DeserializeOwned>(
    references: &Path,
    (path, commit): (&str, &str),
    key: &str,
    provenance: &mut BTreeMap<String, String>,
) -> T {
    let bytes = fs::read(references.join(path)).unwrap();
    provenance.insert(key.into(), format!("lab/references/{path} @ {commit}"));
    provenance.insert(format!("{key}_sha256"), sha256(&bytes));
    serde_json::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes)).unwrap()
}

/// One captured record. Entry payload in stream order (`008dacc0`): DefinitionId, Index,
/// CollectionId, NameId, the icon pair (`008caaa0`), an unknown dword, Collected (not kept).
/// Reward rows' common dwords (`0103c460`): icon pair, +0x10 (display NameId, `00c33430`
/// column 1), +0x20 (count/amount), +0x24 (the item, profile or token id), +0x28.
/// `reward_item(item, name)` is the item a kind-1 row grants.
fn collection(
    record: &CollectionRecord,
    link: &impl Fn(i32, i32) -> Link,
    reward_item: &impl Fn(i32, i32) -> Option<i32>,
    female: &BTreeMap<(i32, i32), i32>,
) -> Collection {
    let [
        id,
        name_id,
        description_id,
        category_id,
        icon_id,
        icon_tint_id,
        flag6,
    ] = record.prefix;
    let entries = record
        .keyed_rows
        .iter()
        .map(|row| {
            let [
                entry_id,
                index,
                owner,
                entry_name,
                entry_icon,
                entry_tint,
                unknown,
            ] = dwords(&row.payload);
            assert_eq!(
                (row.key, owner, unknown),
                (entry_id, id, 0),
                "entry {entry_id}"
            );
            let (item_definition_id, item_link, item_link_note) =
                link(int(entry_id), int(entry_name));
            Entry {
                id: int(entry_id),
                index,
                name_id: int(entry_name),
                icon_id: int(entry_icon),
                icon_tint_id: int(entry_tint),
                item_definition_id,
                item_link,
                item_link_note,
            }
        })
        .collect();
    let mut reward = Reward {
        coins: dwords::<1>(&record.reward.fixed_fields)[0],
        ..Reward::default()
    };
    for row in &record.reward.entries {
        let [_, _, name, count, target, zero] = dwords(&row.common_fields);
        assert_eq!(zero, 0, "collection {id} reward row");
        let tail = || int(dwords::<1>(&row.kind_fields)[0]);
        match row.kind {
            1 => {
                let granted = reward_item(int(target), int(name));
                reward.items.push(RewardItem {
                    item_definition_id: granted,
                    count,
                    female_item_definition_id: female.get(&(int(id), int(target))).copied(),
                    captured_item_definition_id: (granted != Some(int(target)))
                        .then_some(int(target)),
                })
            }
            3 => reward.xp.push(RewardXp {
                profile_id: int(target),
                amount: count,
            }),
            12 => reward.tokens.push(RewardToken {
                token_id: int(target),
                count,
            }),
            10 => reward.collection_add.push(tail()),
            6 => reward.quest_add.push(tail()),
            kind => panic!("collection {id}: reward kind {kind} has no catalogue field"),
        }
    }
    Collection {
        id: int(id),
        name_id: int(name_id),
        description_id: int(description_id),
        category_id: int(category_id),
        icon_id: int(icon_id),
        icon_tint_id: int(icon_tint_id),
        flag6,
        entries,
        reward,
    }
}

fn generate(packet: &[u8], references: &Path) -> String {
    let mut provenance: BTreeMap<String, String> = [
        ("template", "lab/references/rust-gateway-reference/0007-login.bin"),
        (
            "template_note",
            "recorded from the FreeRealms-Legacy gateway (AGPL-derived data) replaying the 2014 SendSelf",
        ),
        ("license", "AGPL data rows only, no code copied"),
        (
            "client_readers",
            "collection 009099d0, entry row 008dacc0, reward bundle 008e7930, reward kinds 0103c070",
        ),
        (
            "item_link_rule",
            "Sanctuary Entries[].ItemDefinitionId when non-zero, else the single Legacy item with the entry NameId, else null",
        ),
        (
            "generator",
            "tests/collection_catalogue.rs regenerate_or_check_collection_catalogue",
        ),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value.into()))
    .collect();
    provenance.insert("template_sha256".into(), sha256(packet));
    let legacy: Vec<LegacyItem> =
        join_input(references, LEGACY_ITEMS, "legacy_items", &mut provenance);
    let sanctuary: Vec<SanctuaryCollection> = join_input(
        references,
        SANCTUARY,
        "sanctuary_collections",
        &mut provenance,
    );
    let lxzxrus: Vec<LxzxrusCollection> =
        join_input(references, LXZXRUS, "lxzxrus_collections", &mut provenance);

    let legacy_ids: BTreeSet<i32> = legacy.iter().map(|item| item.id).collect();
    let mut by_name: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
    for item in &legacy {
        if let Some(name_id) = item.name_id {
            by_name.entry(name_id).or_default().push(item.id);
        }
    }
    let sanctuary: BTreeMap<i32, &SanctuaryEntry> = sanctuary
        .iter()
        .flat_map(|collection| &collection.entries)
        .filter(|entry| entry.item_definition_id != 0)
        .map(|entry| (entry.id, entry))
        .collect();
    let mut female = BTreeMap::new();
    for row in lxzxrus
        .iter()
        .filter(|row| !row.reward_item_ids_girls.is_empty())
    {
        assert_eq!(row.reward_item_ids.len(), row.reward_item_ids_girls.len());
        for (&male, &girl) in row.reward_item_ids.iter().zip(&row.reward_item_ids_girls) {
            female.insert((row.id, male), girl);
        }
    }
    let link = |entry_id: i32, name_id: i32| -> Link {
        if let Some(row) = sanctuary.get(&entry_id) {
            assert_eq!(
                row.name_id, name_id,
                "Sanctuary entry {entry_id} disagrees with the capture"
            );
            return (
                Some(row.item_definition_id),
                Some(ItemLink::Sanctuary),
                None,
            );
        }
        match by_name.get(&name_id).map(Vec::as_slice) {
            Some([item]) => (Some(*item), Some(ItemLink::LegacyUniqueName), None),
            Some([_, _, ..]) => (None, None, Some(ItemLinkNote::AmbiguousName)),
            _ => (None, None, Some(ItemLinkNote::NoItem)),
        }
    };
    // T630 (R): a captured reward item Legacy does not define cannot be granted; the row's own
    // display name picks the single Legacy item with that NameId (12410 "Skeletal T-Rex" ->
    // 79446), and with none the row grants no item (38816, 38817).
    let reward_item = |item: i32, name_id: i32| {
        if legacy_ids.contains(&item) {
            return Some(item);
        }
        match by_name.get(&name_id).map(Vec::as_slice) {
            Some([only]) => Some(*only),
            _ => None,
        }
    };
    let collections: Vec<Collection> = parse_self(packet)
        .collections
        .iter()
        .map(|record| collection(record, &link, &reward_item, &female))
        .collect();

    let reward_items = || collections.iter().flat_map(|c| &c.reward.items);
    let joined_female = reward_items().filter(|item| item.female_item_definition_id.is_some());
    assert_eq!(
        joined_female.count(),
        female.len(),
        "every Lxzxrus female item joins a reward"
    );
    let reward_items_missing_from_legacy = reward_items()
        .flat_map(|item| {
            [
                item.captured_item_definition_id.or(item.item_definition_id),
                item.female_item_definition_id,
            ]
        })
        .flatten()
        .filter(|id| !legacy_ids.contains(id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut null_item_links = BTreeMap::new();
    for note in collections
        .iter()
        .flat_map(|c| &c.entries)
        .filter_map(|e| e.item_link_note)
    {
        *null_item_links.entry(note).or_insert(0) += 1;
    }
    let captured: BTreeSet<i32> = collections.iter().map(|c| c.id).collect();
    let collection_add_not_captured = collections
        .iter()
        .flat_map(|c| &c.reward.collection_add)
        .filter(|id| !captured.contains(id))
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let file = CatalogueFile {
        schema_version: SCHEMA_VERSION,
        provenance,
        collections,
        gaps: Gaps {
            reward_items_missing_from_legacy,
            null_item_links,
            collection_add_not_captured,
        },
    };
    serde_json::to_string_pretty(&file).unwrap() + "\n"
}

/// Generator and drift oracle. `OSFR_REGENERATE_COLLECTIONS=1` rewrites the committed file;
/// otherwise the committed file must equal what the template and joins produce.
#[test]
#[ignore = "requires the private login template via OSFR_GATEWAY_REFERENCE"]
fn regenerate_or_check_collection_catalogue() {
    let reference = PathBuf::from(
        std::env::var("OSFR_GATEWAY_REFERENCE")
            .expect("private reference path required for the catalogue generator"),
    );
    let packet = fs::read(reference.join("0007-login.bin")).unwrap();
    let text = generate(&packet, &reference.join(".."));
    let path = content_dir().join(FILE_NAME);
    if std::env::var("OSFR_REGENERATE_COLLECTIONS").as_deref() == Ok("1") {
        fs::write(&path, &text).unwrap();
    } else {
        assert!(
            fs::read_to_string(&path).unwrap() == text,
            "{FILE_NAME} drifted from the template or its joins; rerun with OSFR_REGENERATE_COLLECTIONS=1"
        );
    }
}
