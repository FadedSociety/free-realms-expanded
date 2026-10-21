//! Drawability coverage for every priced vendor in the shipped content.
//!
//! T372 adds the 25 `npc_spawns_extra.json` actors that were missing from the
//! playtest package. The loader path is `world::NpcVisibility::load`; the
//! interaction path is the real 26/8 -> 26/9 -> 26/10 gateway route. Client
//! evidence: 35/2 AddNpc reader `0x00921a10`; 26/8 constructor `0x008b6bc0`
//! (u64 target + bool); 26/9 chain `0x00aa2360 -> 0x00aa2220 -> 0x00aa1600 ->
//! 0x00aa04d0` (option reader `0x00a9dca0` reads seven dwords; the 32-byte
//! object keeps one unused dword); 26/10 constructor `0x00a36f80` (u64 target +
//! i32 choice). 66/42 is the existing merchant window decoder.
//!
//! The position is the NPC row's reconstructed POI/zone-centre placement; the
//! model is a same-race/gender/job client body chosen from the 2014
//! `Resources/Models.txt` because these late-era NPC rows carry no model
//! binding. Both choices are tagged `(R)` in the spawn row. No live transform,
//! model identity or server GUID is claimed as captured.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;

use gateway_vendor::{inner_packets, interact, login, select, send, wrap};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    game_store::{self, Snapshot},
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    npc_interaction::{NpcAction, NpcInteraction, NpcSelection, decode_merchant_list_66_42},
    world::NpcVisibility,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// The 25 priced vendors that had no runtime actor at the T372 tip. The
/// coverage test also derives the set from content and asserts it is exactly
/// this list, so a new unspawned priced vendor fails instead of passing.
const T372_SPAWNS: &[u64] = &[
    9_500_000_746,
    9_500_000_032,
    9_500_000_067,
    9_500_000_115,
    9_500_000_691,
    9_500_000_741,
    9_500_000_771,
    9_500_000_772,
    9_500_000_775,
    9_500_000_166,
    9_500_000_339,
    9_500_000_348,
    9_500_000_349,
    9_500_000_742,
    9_500_000_888,
    9_500_000_355,
    9_500_000_482,
    9_500_000_676,
    9_500_000_344,
    9_500_000_543,
    9_500_000_354,
    9_500_000_763,
    9_500_000_769,
    9_500_000_648,
    9_500_000_845,
];

/// The brief's representative subset; each row opens its real 66/42 window.
const REPRESENTATIVE: &[u64] = &[
    9_500_000_482,
    9_500_000_746,
    9_500_000_339,
    9_500_000_543,
    9_500_000_166,
];

const NOW: u64 = 10_000;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn lab_dir() -> PathBuf {
    manifest_dir().join("../../../lab")
}

fn content_dir() -> PathBuf {
    manifest_dir().join("data/content")
}

fn gameplay_dir() -> PathBuf {
    lab_dir().join("reports/round70-native-20261016T061000Z/content")
}

fn reference_dir() -> PathBuf {
    lab_dir().join("references/rust-gateway-reference")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

struct NpcRow {
    placement: String,
    name: String,
    position: [f32; 3],
    model_id: Option<u64>,
    model_file_name: Option<String>,
    name_string_id: Option<u64>,
}

fn npc_rows(content: &Path) -> BTreeMap<u64, NpcRow> {
    let mut rows = BTreeMap::new();
    let npc_dir = content.join("npcs");
    for entry in fs::read_dir(&npc_dir).expect("NPC directory reads") {
        let path = entry.expect("NPC entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let document = read_json(&path);
        for row in document["records"].as_array().expect("NPC records") {
            let npc_id = row["npc_id"].as_u64().expect("NPC id");
            let position = row["position"]
                .as_array()
                .expect("NPC position")
                .iter()
                .map(|value| value.as_f64().expect("NPC position component") as f32)
                .collect::<Vec<_>>()
                .try_into()
                .expect("NPC position is XYZ");
            rows.insert(
                npc_id,
                NpcRow {
                    placement: row["placement"].as_str().unwrap_or_default().to_string(),
                    name: row["name"].as_str().expect("NPC name").to_string(),
                    position,
                    model_id: row["model_id"].as_u64(),
                    model_file_name: row["model_file_name"].as_str().map(str::to_string),
                    name_string_id: row["name_string_id"].as_u64(),
                },
            );
        }
    }
    rows
}

struct PricedVendor {
    npc_id: u64,
    item_ids: BTreeSet<i32>,
}

/// Every vendor with at least one item whose server price is positive.
fn priced_vendors(content: &Path) -> Vec<PricedVendor> {
    read_json(&content.join("vendors.json"))["vendors"]
        .as_array()
        .expect("vendor records")
        .iter()
        .filter(|vendor| {
            vendor["items"]
                .as_array()
                .expect("vendor items")
                .iter()
                .any(|item| {
                    item["purchase_price_coins"]
                        .as_u64()
                        .is_some_and(|price| price > 0)
                })
        })
        .map(|vendor| PricedVendor {
            npc_id: vendor["npc_id"].as_u64().expect("vendor NPC id"),
            item_ids: vendor["item_ids"]
                .as_array()
                .expect("vendor item ids")
                .iter()
                .map(|id| {
                    i32::try_from(id.as_i64().expect("vendor item id"))
                        .expect("vendor item id fits i32")
                })
                .collect(),
        })
        .collect()
}

struct ExtraRow {
    content_key: Option<u64>,
    model_id: u64,
    model_file_name: String,
    name: String,
    name_string_id: Option<u64>,
    position: [f32; 3],
    capture_match: Value,
    position_w: Value,
    legacy_npc_id: Value,
    source: Vec<Value>,
}

fn extra_rows(content: &Path) -> BTreeMap<u64, ExtraRow> {
    let mut rows = BTreeMap::new();
    for row in read_json(&content.join("npc_spawns_extra.json"))["records"]
        .as_array()
        .expect("extra spawn records")
    {
        let position = row["position"]
            .as_array()
            .expect("extra position")
            .iter()
            .map(|value| value.as_f64().expect("extra position component") as f32)
            .collect::<Vec<_>>()
            .try_into()
            .expect("extra position is XYZ");
        rows.insert(
            row["spawn_guid"].as_u64().expect("spawn guid"),
            ExtraRow {
                content_key: row["content_key"].as_u64(),
                model_id: row["model_id"].as_u64().unwrap_or_default(),
                model_file_name: row["model_file_name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                name: row["name"].as_str().unwrap_or_default().to_string(),
                name_string_id: row["name_string_id"].as_u64(),
                position,
                capture_match: row["capture_match"].clone(),
                position_w: row["position_w"].clone(),
                legacy_npc_id: row["legacy_npc_id"].clone(),
                source: row["source"].as_array().cloned().unwrap_or_default(),
            },
        );
    }
    rows
}

fn locale_texts() -> &'static HashMap<u64, String> {
    static LOCALE: OnceLock<HashMap<u64, String>> = OnceLock::new();
    LOCALE.get_or_init(|| {
        read_json(&lab_dir().join("reports/locale-id-to-text.json"))
            .as_array()
            .expect("locale rows")
            .iter()
            .filter_map(|row| {
                Some((
                    row["id"].as_u64()?,
                    row["text"].as_str().unwrap_or_default().to_string(),
                ))
            })
            .collect()
    })
}

fn position_matches(actual: [f32; 3], expected: [f32; 3]) -> bool {
    (0..3).all(|axis| (actual[axis] - expected[axis]).abs() < 1e-3)
}

fn source_kind(row: &ExtraRow, kind: &str) -> bool {
    row.source
        .iter()
        .any(|source| source["kind"].as_str() == Some(kind))
}

fn connect(character: CharacterState, state: GameplayState) -> Gateway {
    let guid = character.guid;
    let mut gateway =
        Gateway::load_gameplay(&reference_dir(), character, &gameplay_dir(), Some(state))
            .expect("the gateway loads the prepared gameplay content");
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    for packet in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway
            .handle_at(&packet, NOW)
            .expect("gateway initialization handles");
    }
    gateway
}

fn character_at(position: [f32; 3]) -> CharacterState {
    let mut character = Gateway::reference_state(&reference_dir()).expect("reference character");
    character.zone_id = 0;
    character.position = position;
    character.rotation = [1.0, 0.0, 0.0];
    character
}

fn base_state() -> &'static GameplayState {
    static BASE: OnceLock<GameplayState> = OnceLock::new();
    BASE.get_or_init(|| {
        Gameplay::load(&gameplay_dir(), None)
            .expect("the prepared gameplay content loads")
            .state()
    })
}

fn menu_packet<'a>(replies: &'a [Vec<u8>]) -> &'a [u8] {
    replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/9 menu")
}

fn merchant_packet<'a>(replies: &'a [Vec<u8>]) -> &'a [u8] {
    replies
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 42, 0]))
        .expect("66/42 merchant window")
}

/// The 26/9 option ids are seven little-endian dwords per choice after the
/// u64 NPC id, the boolean byte and the i32 count (`0x00a9dca0`).
fn menu_option_ids(packet: &[u8]) -> Vec<i32> {
    assert_eq!(&packet[..4], &[26, 0, 9, 0]);
    let count = i32::from_le_bytes(packet[13..17].try_into().unwrap());
    assert!(count >= 0, "negative menu count");
    (0..count as usize)
        .map(|index| {
            let offset = 17 + index * 28;
            i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
        })
        .collect()
}

fn shop_choice(content: &Path, npc_id: u64) -> i32 {
    let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("the production NPC loader accepts the vendor content");
    let (zone, position) = service
        .location(npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} has a location"));
    let menu = service
        .interact(npc_id, zone, position, None)
        .unwrap_or_else(|error| panic!("NPC {npc_id} interaction: {error:?}"));
    menu.options
        .iter()
        .find(|option| option.action == NpcAction::Shop)
        .unwrap_or_else(|| panic!("NPC {npc_id} exposes no shop option"))
        .id
}

fn shop_items(packet: &[u8]) -> BTreeSet<i32> {
    let list = decode_merchant_list_66_42(packet).expect("66/42 matches the client reader");
    list.group
        .entries
        .iter()
        .map(|entry| {
            assert!(
                entry.first_six_dwords[1] > 0,
                "shop entry {} has no positive price",
                entry.first_six_dwords[0]
            );
            i32::try_from(entry.first_six_dwords[0]).expect("shop item id fits i32")
        })
        .collect()
}

#[test]
fn every_priced_vendor_is_drawable() {
    let content = content_dir();
    let npcs = npc_rows(&content);
    let vendors = priced_vendors(&content);
    let extras = extra_rows(&content);
    let locale = locale_texts();
    let visibility =
        NpcVisibility::load(&content).expect("the production NPC visibility loader loads");
    let mut seen = BTreeSet::new();
    let targets: BTreeMap<u64, _> = visibility
        .teleport_targets(0)
        .into_iter()
        .map(|target| {
            assert!(seen.insert(target.guid), "duplicate actor {}", target.guid);
            (target.guid, target)
        })
        .collect();

    let expected: BTreeSet<u64> = T372_SPAWNS.iter().copied().collect();
    assert_eq!(expected.len(), T372_SPAWNS.len(), "duplicate T372 spawn id");
    let non_legacy_priced: BTreeSet<u64> = vendors
        .iter()
        .filter(|vendor| {
            npcs.get(&vendor.npc_id)
                .is_some_and(|row| row.placement != "legacy_data")
        })
        .map(|vendor| vendor.npc_id)
        .collect();
    assert_eq!(
        non_legacy_priced, expected,
        "the priced vendors that are not legacy_data changed; T372's 25-row set is no longer exact"
    );

    let missing: BTreeSet<u64> = vendors
        .iter()
        .filter(|vendor| npcs.contains_key(&vendor.npc_id))
        .filter(|vendor| visibility.content_key(vendor.npc_id) != Some(vendor.npc_id))
        .map(|vendor| vendor.npc_id)
        .collect();
    assert!(
        missing.is_empty(),
        "priced vendors with no runtime actor: {missing:?}"
    );

    for vendor in &vendors {
        let Some(npc) = npcs.get(&vendor.npc_id) else {
            // A priced vendor with no NPC row is outside this slice's target set.
            continue;
        };
        let target = targets.get(&vendor.npc_id).unwrap_or_else(|| {
            panic!(
                "priced vendor {} ({}) is not in the 35/2 spawn list",
                vendor.npc_id, npc.name
            )
        });
        assert!(
            position_matches(target.position, npc.position),
            "vendor {} spawn position differs from the NPC row",
            vendor.npc_id
        );
        if npc.placement == "legacy_data" {
            assert!(
                npc.model_id.is_some_and(|model| model > 0),
                "legacy vendor {} has no model",
                vendor.npc_id
            );
            assert!(
                npc.model_file_name
                    .as_deref()
                    .is_some_and(|name| !name.is_empty()),
                "legacy vendor {} has no model file",
                vendor.npc_id
            );
            assert!(
                npc.name_string_id.is_some_and(|name| name > 0),
                "legacy vendor {} has no name id",
                vendor.npc_id
            );
        } else {
            let extra = extras.get(&vendor.npc_id).unwrap_or_else(|| {
                panic!(
                    "non-legacy priced vendor {} has no extra spawn",
                    vendor.npc_id
                )
            });
            assert_eq!(
                extra.content_key,
                Some(vendor.npc_id),
                "vendor {} extra content key",
                vendor.npc_id
            );
            assert_eq!(extra.name, npc.name, "vendor {} spine name", vendor.npc_id);
            assert!(
                position_matches(extra.position, npc.position),
                "vendor {} extra position differs from its NPC row",
                vendor.npc_id
            );
            assert!(
                extra.model_id > 0 && !extra.model_file_name.is_empty(),
                "vendor {} spawn has no drawable model",
                vendor.npc_id
            );
            let name_id = extra.name_string_id.unwrap_or_else(|| {
                panic!("vendor {} spawn has no normalized name id", vendor.npc_id)
            });
            assert_eq!(
                locale.get(&name_id).map(String::as_str),
                Some(npc.name.as_str()),
                "vendor {} name id does not resolve to its row name",
                vendor.npc_id
            );
            assert_eq!(extra.legacy_npc_id, Value::Null);
            assert_eq!(extra.capture_match, Value::Null);
            assert_eq!(extra.position_w, Value::Null);
            assert!(
                source_kind(extra, "legacy-data"),
                "vendor {} spawn lacks a legacy-data source",
                vendor.npc_id
            );
            assert!(
                source_kind(extra, "reconstructed"),
                "vendor {} spawn lacks a reconstructed source",
                vendor.npc_id
            );
            let reconstruction = extra
                .source
                .iter()
                .filter(|source| source["kind"] == "reconstructed")
                .filter_map(|source| source["evidence"].as_str())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(
                reconstruction.starts_with("(R)")
                    && reconstruction.contains("not a captured transform")
                    && reconstruction.to_ascii_lowercase().contains("position"),
                "vendor {} reconstruction does not state the reconstructed placement",
                vendor.npc_id
            );
        }
    }
}

#[test]
fn representative_vendors_open_the_real_shop_menu() {
    let content = content_dir();
    let npcs = npc_rows(&content);
    let vendors: BTreeMap<u64, PricedVendor> = priced_vendors(&content)
        .into_iter()
        .map(|vendor| (vendor.npc_id, vendor))
        .collect();
    let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("the production NPC loader accepts the vendor content");
    for &npc_id in REPRESENTATIVE {
        let npc = npcs
            .get(&npc_id)
            .unwrap_or_else(|| panic!("representative vendor {npc_id} has no NPC row"));
        let vendor = vendors
            .get(&npc_id)
            .unwrap_or_else(|| panic!("representative vendor {npc_id} has no priced stock"));
        let (zone, position) = service
            .location(npc_id)
            .unwrap_or_else(|| panic!("vendor {npc_id} has a location"));
        let menu = service
            .interact(npc_id, zone, position, None)
            .unwrap_or_else(|error| panic!("vendor {npc_id} interaction: {error:?}"));
        let choice = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Shop)
            .unwrap_or_else(|| panic!("vendor {npc_id} exposes no shop option"))
            .id;
        let selected = service
            .select(&select(npc_id, choice as u32), zone, position, None)
            .unwrap_or_else(|error| panic!("vendor {npc_id} shop selection: {error:?}"));
        let NpcSelection::Shop(window) = selected else {
            panic!("vendor {npc_id} selected a non-shop action");
        };
        let service_items: BTreeSet<i32> = window.items.iter().map(|item| item.item_id).collect();
        assert_eq!(
            service_items, vendor.item_ids,
            "vendor {npc_id} service window does not list its priced stock"
        );

        let mut gateway = connect(character_at(npc.position), base_state().clone());
        let menu = send(&mut gateway, &interact(npc_id));
        let packet = menu_packet(&menu);
        assert!(
            menu_option_ids(packet).contains(&choice),
            "vendor {npc_id} 26/9 menu has no shop choice {choice}"
        );
        let window = send(&mut gateway, &select(npc_id, choice as u32));
        let items = shop_items(merchant_packet(&window));
        assert_eq!(
            items, vendor.item_ids,
            "vendor {npc_id} 66/42 window does not list its priced stock"
        );
    }
}

#[test]
fn a_far_or_wrong_zone_click_changes_nothing() {
    const FLORINA: u64 = 9_500_000_482;
    let content = content_dir();
    let npcs = npc_rows(&content);
    let npc = npcs.get(&FLORINA).expect("Florina row");

    let mut far = connect(
        character_at([npc.position[0] + 100.0, npc.position[1], npc.position[2]]),
        base_state().clone(),
    );
    let before = far.snapshot();
    let outcome = far
        .handle_at(&wrap(&interact(FLORINA)), NOW)
        .expect("an out-of-range click is a per-session refusal");
    assert!(
        inner_packets(&outcome).is_empty(),
        "an out-of-range click sent a reply"
    );
    assert_eq!(
        far.snapshot(),
        before,
        "an out-of-range click changed state"
    );

    // A non-zero character zone cannot load through `Gateway::load_gameplay` in
    // this prepared pack (its gameplay supports zone 0), so drive the same
    // `Gameplay::handle_in_world` entry the gateway calls with a zone-1
    // character. The NPC row is in zone 10, so `check_range` refuses the click
    // before any menu.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    let mut wrong_zone_character = character_at(npc.position);
    wrong_zone_character.zone_id = 1;
    let before = game.state();
    let outcome = game.handle_in_world(&interact(FLORINA), &wrong_zone_character, NOW);
    assert!(outcome.is_err(), "a wrong-zone click must be refused");
    assert_eq!(game.state(), before, "a wrong-zone click changed state");
}

#[test]
fn two_players_see_and_open_the_same_vendor() {
    const FLORINA: u64 = 9_500_000_482;
    let content = content_dir();
    let npcs = npc_rows(&content);
    let npc = npcs.get(&FLORINA).expect("Florina row");
    let choice = shop_choice(&content, FLORINA);

    let mut first = connect(character_at(npc.position), base_state().clone());
    let mut second = connect(character_at(npc.position), base_state().clone());
    let first_menu = send(&mut first, &interact(FLORINA));
    let second_menu = send(&mut second, &interact(FLORINA));
    assert_eq!(
        menu_packet(&first_menu),
        menu_packet(&second_menu),
        "two sessions see different 26/9 menus"
    );
    assert!(menu_option_ids(menu_packet(&second_menu)).contains(&choice));

    // One session's refused choice leaves its menu and the other session alone.
    let second_before = second.snapshot();
    let refused = second
        .handle_at(&wrap(&select(FLORINA, u32::MAX)), NOW)
        .expect("an unoffered choice is a per-session refusal");
    assert!(
        inner_packets(&refused).is_empty(),
        "an unoffered choice sent a reply"
    );
    assert_eq!(
        second.snapshot(),
        second_before,
        "a refused choice changed the session"
    );

    // A third, out-of-range player's refusal cannot consume either menu.
    let mut far = connect(
        character_at([npc.position[0] + 100.0, npc.position[1], npc.position[2]]),
        base_state().clone(),
    );
    let far_before = far.snapshot();
    let outcome = far
        .handle_at(&wrap(&interact(FLORINA)), NOW)
        .expect("the far click is a per-session refusal");
    assert!(inner_packets(&outcome).is_empty());
    assert_eq!(far.snapshot(), far_before);

    let first_window = send(&mut first, &select(FLORINA, choice as u32));
    let second_window = send(&mut second, &select(FLORINA, choice as u32));
    assert_eq!(
        shop_items(merchant_packet(&first_window)),
        shop_items(merchant_packet(&second_window)),
        "two sessions see different shop stock"
    );
}

struct SaveDir(PathBuf);

impl Drop for SaveDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn save_dir(label: &str) -> SaveDir {
    let path = lab_dir()
        .join("build/t372-priced-vendors")
        .join(format!("{label}-{}", std::process::id()));
    fs::create_dir_all(&path).expect("T372 save directory creates");
    SaveDir(path)
}

#[test]
fn a_reconnect_keeps_the_vendor_drawable_and_shop_openable() {
    const FLADNAG: u64 = 9_500_000_746;
    let content = content_dir();
    let npcs = npc_rows(&content);
    let npc = npcs.get(&FLADNAG).expect("Fladnag row");
    let choice = shop_choice(&content, FLADNAG);
    let save = save_dir("reconnect");
    let path = save.0.join("snapshot.json");

    let mut gateway = connect(character_at(npc.position), base_state().clone());
    send(&mut gateway, &interact(FLADNAG));
    let opened = send(&mut gateway, &select(FLADNAG, choice as u32));
    let before = shop_items(merchant_packet(&opened));
    let snapshot = gateway.snapshot();
    game_store::save(&path, &snapshot).expect("the character snapshot saves");

    let loaded = game_store::load_or_create(&path, snapshot.character.clone())
        .expect("the character snapshot reloads");
    assert_eq!(loaded.character, snapshot.character);
    assert_eq!(loaded.gameplay, snapshot.gameplay);
    let visibility =
        NpcVisibility::load(&content).expect("the production NPC visibility loader reloads");
    assert_eq!(visibility.content_key(FLADNAG), Some(FLADNAG));
    let target = visibility
        .teleport_targets(0)
        .into_iter()
        .find(|target| target.guid == FLADNAG)
        .expect("Fladnag is still spawned after content reload");
    assert!(position_matches(target.position, npc.position));

    let mut reconnected = Gateway::load_gameplay(
        &reference_dir(),
        loaded.character.clone(),
        &gameplay_dir(),
        loaded.gameplay.clone(),
    )
    .expect("the reconnected gateway loads");
    reconnected.set_dev_tools(false);
    reconnected.set_role(AccountRole::Player);
    for packet in [login(loaded.character.guid), wrap(&[13, 0]), wrap(&[10, 0])] {
        reconnected
            .handle_at(&packet, NOW)
            .expect("gateway initialization handles");
    }
    let menu = send(&mut reconnected, &interact(FLADNAG));
    assert!(menu_option_ids(menu_packet(&menu)).contains(&choice));
    let window = send(&mut reconnected, &select(FLADNAG, choice as u32));
    assert_eq!(
        shop_items(merchant_packet(&window)),
        before,
        "the relogged shop stock differs"
    );
    assert_eq!(
        reconnected.snapshot().character,
        loaded.character,
        "the relog moved the character"
    );
    // The save is still a valid envelope after the relog.
    let reloaded: Snapshot = game_store::load_or_create(&path, loaded.character.clone())
        .expect("the snapshot remains readable");
    assert_eq!(reloaded, loaded);
}
