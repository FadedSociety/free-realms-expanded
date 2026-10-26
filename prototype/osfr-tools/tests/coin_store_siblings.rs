//! W172: the Coin Store and Marketplace sibling requests (165/2, 165/8, 165/12, 165/14 and the
//! Marketplace rows 66/1..38), at the service layer and routed through the gateway. Synthetic
//! content only; no player data, names or chat.
//!
//! Recovered client facts are cited at `coin_shop.rs`' module comment: 165/12 is the sale's
//! transaction id plus a quantity, 165/3 is a flag plus an id set, and the Marketplace list
//! replies all lead with a status dword. Everything else asserted here is server policy (R):
//! a data-configured buy-back window, buy-back price equal to the sale price, gifts and orders
//! refused, Station Cash and membership lists empty.
//!
//! The fixture is copied from `vendor_sell.rs`; the packet builders and the 165/6 parser are the
//! shared `tests/common/gateway_vendor.rs` ones.
#![allow(dead_code)]

use gateway_vendor::{
    NOW, buy_back_packet, completions, count_definition, inner_packets, inventory_of, sell_packet,
    send, wrap,
};
use osfr_tools::{
    account_store::AccountRole,
    chat,
    coin_shop::{self, CoinShop, CoinShopOutcome, StoreCatalogue},
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState},
    npc_interaction::{NpcAction, NpcInteraction, NpcInteractionError, NpcSelection},
    station_cash::{self, Wallet},
};
use resale::resale;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/item_catalogue.rs"]
#[allow(dead_code)]
mod item_catalogue;
#[path = "common/resale.rs"]
mod resale;

const VENDOR_ID: u64 = 9_500_000_339;
/// Default-state fixtures from `lab/references/rust-gameplay/inventory.json`.
const SELLABLE_INSTANCE: i32 = 14; // definition 75060 (bought for 750), not equipped
const EQUIPPED_INSTANCE: i32 = 34; // definition 21683, sell price 5, equipped
const UNCATALOGUED_INSTANCE: i32 = 4; // definition 19522, no store row
const NO_SALE_DEFINITION: i32 = 1180; // store row flagged no_sale
const CATALOGUE_ONLY_DEFINITION: i32 = 910_500;
const RESULT_OK: u32 = coin_shop::RESULT_SUCCESS;

fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn store_row(id: i32, buy: Option<u32>, sell: Option<u32>, no_sale: bool) -> Value {
    json!({
        "item_id": id,
        "purchase_price_coins": buy,
        "member_discount_percent": null,
        "member_price_coins": null,
        "sell_price_coins": sell,
        "no_sale": no_sale
    })
}

fn inventory_content() -> PathBuf {
    std::env::var_os("OSFR_COIN_SHOP_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-gameplay")
        })
}

/// Instance ids whose stack differs between two states (added, removed or modified).
fn changed_stacks(before: &InventoryState, after: &InventoryState) -> Vec<i32> {
    let ids: BTreeSet<i32> = before
        .items
        .iter()
        .chain(&after.items)
        .map(|item| item.id)
        .collect();
    ids.into_iter()
        .filter(|id| {
            before.items.iter().find(|item| item.id == *id)
                != after.items.iter().find(|item| item.id == *id)
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Service layer: NpcInteraction + CoinShop
// ---------------------------------------------------------------------------------------------

struct Counter {
    service: NpcInteraction,
    shop: CoinShop,
    zone: u32,
    position: [f32; 3],
}

impl Counter {
    fn new(coins: u32, tweak: impl FnOnce(&mut InventoryState)) -> Self {
        let directory = inventory_content();
        let mut state = Inventory::load(&directory, None)
            .expect("inventory definitions load")
            .state()
            .clone();
        state.coins = coins;
        tweak(&mut state);
        let inventory = Inventory::load(&directory, Some(state)).expect("test state is valid");
        let catalogue = StoreCatalogue::load(&content_dir().join("stores.json")).unwrap();
        let shop = CoinShop::new(
            &directory,
            inventory,
            catalogue,
            usize::MAX,
            None,
            Wallet::default(),
        )
        .unwrap();
        let data = content_dir();
        let mut service =
            NpcInteraction::load(&data.join("npcs"), &data.join("stores.json")).unwrap();
        // (R) Item 8 is an explicit fixture mapping for the vendor's stock.
        service.set_vendor_stock_items(VENDOR_ID, &[8]).unwrap();
        let (zone, position) = service.location(VENDOR_ID).expect("content vendor exists");
        Self {
            service,
            shop,
            zone,
            position,
        }
    }

    fn open(&mut self) {
        let menu = self
            .service
            .interact(VENDOR_ID, self.zone, self.position, None)
            .unwrap();
        let option = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Shop)
            .expect("vendor exposes a shop option");
        let mut request = vec![26, 0, 10, 0];
        request.extend(VENDOR_ID.to_le_bytes());
        request.extend(option.id.to_le_bytes());
        let selected = self
            .service
            .select(&request, self.zone, self.position, None)
            .unwrap();
        assert!(matches!(selected, NpcSelection::Shop(_)));
    }

    fn state(&self) -> InventoryState {
        self.shop.inventory().state().clone()
    }

    fn send(&mut self, packet: &[u8]) -> Result<CoinShopOutcome, NpcInteractionError> {
        self.service
            .handle_coin_store(&mut self.shop, packet, self.zone, self.position)
    }
}

// ---------------------------------------------------------------------------------------------
// Routed fixture (the real gameplay.rs path)
// ---------------------------------------------------------------------------------------------

const NPC: u64 = 700;
const STACK_MANY: i32 = 900_001; // definition 75060 x3, sell 375 each
const STACK_NO_SALE: i32 = 900_003; // definition 1180
const STACK_ONE: i32 = 900_004; // definition 8 x1, sell 7
static NEXT: AtomicU64 = AtomicU64::new(0);

fn bundle(messages: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for message in messages {
        assert!(message.len() < u16::MAX as usize);
        // The client's two-byte length form: 0xff then a 16-bit big-endian length; only
        // `FF FF FF` selects a 32-bit length (GetVariableValue 0x010daea0).
        bytes.push(255);
        bytes.extend((message.len() as u16).to_be_bytes());
        bytes.extend(message);
    }
    bytes
}

fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["w172-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

fn npc_request(sub: u16, choice: Option<u32>) -> Vec<u8> {
    let mut bytes = [26u16.to_le_bytes(), sub.to_le_bytes()].concat();
    bytes.extend(NPC.to_le_bytes());
    match choice {
        Some(choice) => bytes.extend(choice.to_le_bytes()),
        None => bytes.push(0),
    }
    bytes
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

struct Fixture {
    content: PathBuf,
    reference: PathBuf,
    /// An item instance that is equipped in the prepared default state.
    equipped: i32,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl Fixture {
    fn new() -> Self {
        Self::with_definitions(Vec::new())
    }

    /// A fixture whose unowned catalogue carries `definitions` in addition to the one store row.
    fn with_definitions(mut definitions: Vec<Value>) -> Self {
        definitions.push(item_catalogue::sized_row(
            CATALOGUE_ONLY_DEFINITION as u32,
            8,
        ));
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let source = std::env::var_os("OSFR_APP_FUZZ_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab.join("reports/round59-native-20261015T124443Z/content"));
        let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab.join("references/rust-gateway-reference"));
        assert!(
            source.join("inventory.json").is_file(),
            "prepared content fixture missing: {}",
            source.display()
        );
        let content = lab.join("build/w172-fixtures").join(format!(
            "vendor-sell-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&content).unwrap();
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file()
                && matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("json" | "bin")
                )
            {
                fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
            }
        }
        fs::create_dir(content.join("npcs")).unwrap();
        write_json(
            &content.join("npcs/synthetic.json"),
            &json!({"schema_version":1,"records":[
                {"npc_id":NPC,"name":"Synthetic NPC","zone_id":0,"position":[0.,0.,0.],
                 "placement":"legacy_data","model_id":1,"roles":["vendor"],"vendor_stock":"synthetic"}
            ]}),
        );
        write_json(
            &content.join("npc_spawns_extra.json"),
            &json!({"schema_version":1,"records":[]}),
        );
        write_json(
            &content.join("zones.json"),
            &json!({"schema_version":1,"records":[{"id":0,"name":"Synthetic zone"}]}),
        );
        write_json(
            &content.join("vendors.json"),
            &json!({"schema_version":1,"vendors":[{"npc_id":NPC,"item_ids":[8]}]}),
        );
        // The quests/jobs files the gameplay loader expects, kept empty of this test's concerns.
        fs::create_dir(content.join("quests")).unwrap();
        let provenance = json!([{"kind":"reconstructed","evidence":"W172 synthetic test policy"}]);
        write_json(
            &content.join("quests/synthetic.json"),
            &json!({"schema_version":1,"records":[{
                "id":701,"title_string_id":1,"giver_npc_id":NPC,"prerequisite_quest_ids":[],
                "objectives":[{"kind":"talk_to_npc","id":702,"npc_id":NPC,"source":provenance}],
                "rewards":[{"coins":7,"items":[],"job_xp":{},"source":provenance}],"source":provenance
            }]}),
        );
        write_json(
            &content.join("jobs.json"),
            &json!({"schema_version":1,"records":[{
                "id":"synthetic","name":"Synthetic job","profile_id":null,"category":"freestyle",
                "max_level":20,"members_only":false,"trainer_npc_ids":[],"quest_ids":[701],"abilities":[],"source":provenance
            }]}),
        );
        item_catalogue::write(&content, &definitions);
        // Pick an equipped item from the state the gameplay loader actually produces.
        let loaded = Gameplay::load(&content, None).unwrap().state().inventory;
        let (equipped, equipped_definition) = loaded
            .equipment
            .iter()
            .filter(|entry| entry.item_id != 0)
            .find_map(|entry| {
                let item = loaded.items.iter().find(|item| item.id == entry.item_id)?;
                Some((item.id, item.definition_id))
            })
            .expect("the prepared state has an equipped item");
        write_json(
            &content.join("stores.json"),
            &json!({"schema_version":1,"marketplace":{
            "enabled_on_handshake":false,"orders_enabled":false,
            "payment_sources_enabled":false,"not_available_message":"disabled"},
            "items":[
                store_row(8, Some(15), Some(7), false),
                store_row(75060, Some(750), Some(375), false),
                store_row(equipped_definition, Some(10), Some(5), false),
                store_row(NO_SALE_DEFINITION, Some(50), None, true),
                store_row(CATALOGUE_ONLY_DEFINITION, Some(20), Some(10), false),
            ]}),
        );
        Self {
            content,
            reference,
            equipped,
        }
    }

    /// T230: enable the Station Cash Marketplace with one listed ride (item 14289, defined by the
    /// round59 fixture) at 100 SC and one earn-only row (item 19522). The normal fixture keeps the
    /// feature off, so the older "orders disabled" tests still exercise that path.
    fn with_sc(self) -> Self {
        let path = self.content.join("stores.json");
        let mut stores = read_json(&path);
        stores["marketplace"]["enabled_on_handshake"] = json!(true);
        stores["station_cash"] = json!({
            "enabled": true,
            "store_id": 1,
            "store_name_id": 0,
            "store_description_id": 0,
            "category_id": 1,
            "category_name_id": 0,
            "category_group_id": 1,
            // T526: the four ride children, so the gateway emits the child tree.
            "ride_categories": [
                {"key":"winged","category_id":2,"category_name_id":5101131,"display_order":10},
                {"key":"non_winged","category_id":3,"category_name_id":5101134,"display_order":20},
                {"key":"exotic","category_id":4,"category_name_id":5101132,"display_order":30},
                {"key":"mechanical","category_id":5,"category_name_id":5101133,"display_order":40}
            ],
            "sc_price_scale": 1.0,
            "min_price_sc": 1,
            "max_price_sc": 100_000,
            "max_balance": 1_000,
            "order_history": 8,
            "currency_type": 0
        });
        for row in [
            json!({
                "item_id": 14289,
                "purchase_price_coins": null,
                "member_discount_percent": null,
                "member_price_coins": null,
                "sell_price_coins": null,
                "no_sale": true,
                "sc_price": 100,
                // T526: Wolf is a Non-Winged ride, so the bundle lists under child 3.
                "ride_category": "non_winged"
            }),
            json!({
                "item_id": 19522,
                "purchase_price_coins": null,
                "member_discount_percent": null,
                "member_price_coins": null,
                "sell_price_coins": null,
                "no_sale": true,
                "sc_price": 50,
                "earn_only": true
            }),
        ] {
            stores["items"].as_array_mut().unwrap().push(row);
        }
        write_json(&path, &stores);
        // F656: only a ride is single-copy, and `StoreCatalogue::load` finds the rides in the
        // `mounts.json` beside `stores.json`: the shipped one, cut down to the listed ride.
        let mut mounts =
            read_json(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/mounts.json"));
        mounts["mounts"]
            .as_array_mut()
            .unwrap()
            .retain(|mount| mount["item_definition_id_candidates"] == json!([SC_RIDE_ITEM]));
        assert_eq!(mounts["mounts"].as_array().unwrap().len(), 1);
        write_json(&self.content.join("mounts.json"), &mounts);
        self
    }

    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None).unwrap().state();
        let equipped: BTreeSet<_> = state
            .inventory
            .equipment
            .iter()
            .map(|entry| entry.item_id)
            .collect();
        assert!(equipped.contains(&self.equipped));
        state
            .inventory
            .items
            .retain(|item| equipped.contains(&item.id));
        state.inventory.coins = 100;
        for (id, definition_id, count) in [
            (STACK_MANY, 75060, 3),
            (STACK_NO_SALE, NO_SALE_DEFINITION, 1),
            (STACK_ONE, 8, 1),
        ] {
            state.inventory.items.push(InventoryItem {
                id,
                definition_id,
                tint: 0,
                count,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: 0,
                rental: None,
                ability_count: 0,
                activate_enabled: false,
            });
        }
        state.inventory.next_instance_id = Some(900_005);
        state
    }

    fn connect(&self, state: GameplayState) -> Gateway {
        self.enter(state).0
    }

    /// Enter the world and return the initialization (Ready) reply stream, so a test can pin
    /// that world entry itself pushes no Marketplace packet.
    fn enter(&self, state: GameplayState) -> (Gateway, Vec<Vec<u8>>) {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.position = [0.; 3];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, Some(state)).unwrap();
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        gateway.handle_at(&login(), NOW).unwrap();
        let ready = gateway.handle_at(&wrap(&[13, 0]), NOW).unwrap();
        gateway.handle_at(&wrap(&[10, 0]), NOW).unwrap();
        (gateway, inner_packets(&ready))
    }

    /// A fresh player standing at the vendor with the shop window open.
    fn at_open_shop(&self) -> (Gateway, Vec<Vec<u8>>) {
        let mut gateway = self.connect(self.state());
        gateway
            .handle_at(&wrap(&npc_request(8, None)), NOW)
            .unwrap();
        let opened = gateway
            .handle_at(&wrap(&npc_request(10, Some(2))), NOW)
            .unwrap();
        let inner = inner_packets(&opened);
        assert!(
            inner.iter().any(|p| p.starts_with(&[165, 0, 10, 0])),
            "the shop did not open"
        );
        (gateway, inner)
    }
}

// ---------------------------------------------------------------------------------------------
// Packet builders and readers for the sibling requests
// ---------------------------------------------------------------------------------------------

/// 165/13 as the client reads it (`00c71e40`): three dwords after the header.
fn parse_buy_back(packet: &[u8]) -> (u32, u32, u32) {
    assert_eq!(packet.len(), 16, "165/13 is the header and three dwords");
    assert_eq!(&packet[..4], &[165, 0, 13, 0]);
    let word = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    (word(4), word(8), word(12))
}

fn buy_backs(replies: &[Vec<u8>]) -> Vec<(u32, u32, u32)> {
    replies
        .iter()
        .filter(|packet| packet.starts_with(&[165, 0, 13, 0]))
        .map(|packet| parse_buy_back(packet))
        .collect()
}

fn definitions_packet(ids: &[u32]) -> Vec<u8> {
    let mut packet = vec![165, 0, 2, 0];
    packet.extend((ids.len() as i32).to_le_bytes());
    ids.iter().for_each(|id| packet.extend(id.to_le_bytes()));
    packet
}

/// 165/3 as the client reads it (`00c74e40`, `008e8230`): a flag byte, a count, then the ids.
fn parse_definitions(packet: &[u8]) -> (bool, Vec<u32>) {
    assert_eq!(&packet[..4], &[165, 0, 3, 0]);
    assert!(packet[4] <= 1, "the flag is a boolean byte");
    let count = i32::from_le_bytes(packet[5..9].try_into().unwrap());
    assert_eq!(packet.len(), 9 + 4 * count as usize, "no unread bytes");
    let ids = packet[9..]
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();
    (packet[4] == 1, ids)
}

/// A 35/37 batch. Rows have no per-record length, so the test fixture retains the expected
/// records separately and compares the concatenated payload.
fn parse_definition_batch(packet: &[u8]) -> (i32, Vec<u8>) {
    assert_eq!(&packet[..4], &[35, 0, 37, 0]);
    let payload_len = i32::from_le_bytes(packet[4..8].try_into().unwrap());
    let count = i32::from_le_bytes(packet[8..12].try_into().unwrap());
    assert_eq!(payload_len as usize + 8, packet.len());
    assert!(count >= 1, "an empty batch is never emitted");
    (count, packet[12..].to_vec())
}

fn gift_packet() -> Vec<u8> {
    let mut packet = vec![165, 0, 14, 0];
    packet.extend(VENDOR_ID.to_le_bytes());
    packet.extend(1u64.to_le_bytes());
    packet.extend(1i32.to_le_bytes());
    packet.extend(8u32.to_le_bytes());
    packet.extend(1u32.to_le_bytes());
    packet
}

fn marketplace(sub: u16) -> Vec<u8> {
    [66u16.to_le_bytes(), sub.to_le_bytes()].concat()
}

const SC_RIDE_ITEM: i32 = 14_289;
const SC_EARN_ONLY_ITEM: i32 = 19_522;
const SC_RIDE_PRICE: u32 = 100;

fn sc_put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// The client order serializer `00b3fc60`/`00b3af80` (66/3), with one detail.
fn sc_order(tracking: u32, bundle: u32, quantity: u32) -> Vec<u8> {
    let mut bytes = vec![66, 0, 3, 0];
    sc_put_string(&mut bytes, "");
    sc_put_string(&mut bytes, "");
    bytes.extend(tracking.to_le_bytes());
    sc_put_string(&mut bytes, "en_US");
    sc_put_string(&mut bytes, "");
    sc_put_string(&mut bytes, "");
    sc_put_string(&mut bytes, "");
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(bundle.to_le_bytes());
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(quantity.to_le_bytes());
    sc_put_string(&mut bytes, "0");
    sc_put_string(&mut bytes, "");
    bytes
}

fn sc_preview(tracking: u32, bundle: u32, quantity: u32) -> Vec<u8> {
    let mut bytes = sc_order(tracking, bundle, quantity);
    bytes[2] = 1;
    sc_put_string(&mut bytes, "");
    bytes
}

/// The 66/4 result: the second dword after the header (`00b2ded0` obj+0x1C, T453).
fn order_result(packets: &[Vec<u8>]) -> u32 {
    packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 4, 0]))
        .map(|packet| u32::from_le_bytes(packet[8..12].try_into().unwrap()))
        .expect("order response exists")
}

fn wallet_balance(packets: &[Vec<u8>]) -> u32 {
    packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 11, 0]))
        .map(|packet| u32::from_le_bytes(packet[9..13].try_into().unwrap()))
        .expect("wallet response exists")
}

fn locale_request(sub: u16, locale: &str) -> Vec<u8> {
    let mut packet = marketplace(sub);
    packet.extend((locale.len() as i32).to_le_bytes());
    packet.extend(locale.as_bytes());
    packet
}

/// Status 1 and an empty counted list: the shape `00b415c0` and `00b41800` read.
fn status_and_empty_list(opcode: u8) -> Vec<u8> {
    vec![66, 0, opcode, 0, 1, 0, 0, 0, 0, 0, 0, 0]
}

/// Status 1, an empty text and an empty list: `00b416c0`, `00b41760` and `00b41620`.
fn status_text_and_empty_list(opcode: u8) -> Vec<u8> {
    vec![66, 0, opcode, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
}

/// The login-time and sibling Marketplace requests with the exact reply bytes the recovered
/// client readers consume with nothing left over. `enabled` is the store config's
/// `enabled_on_handshake`, which 66/12 repeats (the real content enables it, the fixture does not).
fn marketplace_table(enabled: bool) -> Vec<(&'static str, Vec<u8>, Vec<u8>)> {
    let mut wallet = vec![66, 0, 11, 0, 1, 0, 0, 0, 0];
    wallet.extend([0; 16]); // two dwords, two empty texts (`00b2b450`)
    wallet.push(0);
    let mut account = vec![66, 0, 26, 0, 1, 0, 0, 0];
    account.extend([2, 0, 0, 0, b'U', b'S', 3, 0, 0, 0, b'U', b'S', b'D', 0]);
    vec![
        ("wallet 66/10", marketplace(10), wallet),
        (
            "status 66/12",
            marketplace(12),
            vec![66, 0, 13, 0, u8::from(enabled)],
        ),
        (
            "station cash skus 66/14",
            marketplace(14),
            status_and_empty_list(15),
        ),
        (
            "currency 66/16",
            locale_request(16, "en_US"),
            status_text_and_empty_list(17),
        ),
        ("states 66/18", marketplace(18), status_and_empty_list(19)),
        (
            "countries 66/20",
            locale_request(20, "en_US"),
            status_text_and_empty_list(21),
        ),
        (
            "membership skus 66/22",
            marketplace(22),
            status_text_and_empty_list(23),
        ),
        ("account 66/25", locale_request(25, "en_US"), account),
    ]
}

// ---------------------------------------------------------------------------------------------
// Service layer: sell then buy back
// ---------------------------------------------------------------------------------------------

#[test]
fn sell_then_buy_back_once_restores_the_stack_and_the_exact_coins() {
    let mut counter = Counter::new(100, |_| {});
    counter.open();
    let start = counter.state();
    let stack_before = count_definition(&start, 75060);
    assert!(stack_before > 0, "fixture owns a sellable stack");

    let sold = counter.send(&sell_packet(0, SELLABLE_INSTANCE, 1)).unwrap();
    assert!(sold.changed);
    let id = completions(&sold.replies)[0].transaction;
    let after_sale = counter.state();
    assert_eq!(after_sale.coins, 100 + resale(75060));
    assert_eq!(count_definition(&after_sale, 75060), stack_before - 1);

    let back = counter.send(&buy_back_packet(id, 0)).unwrap();
    assert!(back.changed);
    assert_eq!(buy_backs(&back.replies), vec![(id, RESULT_OK, 0)]);
    let restored = counter.state();
    // (R) The buy-back price is the sale price, so the round trip is exactly neutral.
    assert_eq!(restored.coins, 100);
    assert_eq!(count_definition(&restored, 75060), stack_before);

    // A second try for the same sale is a clean failure: nothing is charged or granted.
    for _ in 0..3 {
        let again = counter.send(&buy_back_packet(id, 0)).unwrap();
        assert!(!again.changed);
        assert_eq!(
            buy_backs(&again.replies),
            vec![(id, coin_shop::RESULT_FAILURE, 0)]
        );
        assert_eq!(counter.state(), restored);
    }
}

#[test]
fn partial_buy_back_consumes_only_the_requested_quantity() {
    const STACK: i32 = 900_020;
    let mut counter = Counter::new(0, |state| {
        state.items.push(InventoryItem {
            id: STACK,
            definition_id: 75060,
            tint: 0,
            count: 10,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        state.next_instance_id = Some(900_100);
    });
    counter.open();
    let start = counter.state();
    let original = count_definition(&start, 75060);

    let sold = counter.send(&sell_packet(0, STACK, 10)).unwrap();
    let id = completions(&sold.replies)[0].transaction;
    assert_eq!(counter.state().coins, 10 * resale(75060));
    assert_eq!(count_definition(&counter.state(), 75060), original - 10);

    let partial = counter.send(&buy_back_packet(id, 3)).unwrap();
    assert!(partial.changed);
    assert_eq!(buy_backs(&partial.replies), vec![(id, RESULT_OK, 3)]);
    assert_eq!(counter.state().coins, 7 * resale(75060));
    assert_eq!(count_definition(&counter.state(), 75060), original - 7);

    // Eight exceeds the seven left on the sale. Nothing changes, including its remainder.
    let after_partial = counter.state();
    let excessive = counter.send(&buy_back_packet(id, 8)).unwrap();
    assert!(!excessive.changed);
    assert_eq!(
        buy_backs(&excessive.replies),
        vec![(id, coin_shop::RESULT_FAILURE, 8)]
    );
    assert_eq!(counter.state(), after_partial);

    // Zero means the seven still outstanding, which proves the failed request kept the history.
    let rest = counter.send(&buy_back_packet(id, 0)).unwrap();
    assert!(rest.changed);
    assert_eq!(buy_backs(&rest.replies), vec![(id, RESULT_OK, 0)]);
    assert_eq!(counter.state().coins, 0);
    assert_eq!(count_definition(&counter.state(), 75060), original);

    let replay = counter.send(&buy_back_packet(id, 0)).unwrap();
    assert!(!replay.changed);
    assert_eq!(
        buy_backs(&replay.replies),
        vec![(id, coin_shop::RESULT_FAILURE, 0)]
    );
}

#[test]
fn an_unaffordable_buy_back_keeps_the_sale_available() {
    let mut counter = Counter::new(0, |_| {});
    counter.open();
    let sold = counter.send(&sell_packet(0, SELLABLE_INSTANCE, 1)).unwrap();
    let id = completions(&sold.replies)[0].transaction;
    let set_coins = |counter: &mut Counter, coins: u32| {
        let mut state = counter.state();
        state.coins = coins;
        let inventory = Inventory::load(&inventory_content(), Some(state)).unwrap();
        counter.shop.sync_inventory(inventory);
    };

    let unit = resale(75060);
    set_coins(&mut counter, unit - 1);
    let before = counter.state();
    let short = counter.send(&buy_back_packet(id, 0)).unwrap();
    assert!(!short.changed);
    assert_eq!(
        buy_backs(&short.replies),
        vec![(id, coin_shop::RESULT_FAILURE, 0)]
    );
    assert_eq!(counter.state(), before);

    set_coins(&mut counter, unit);
    let paid = counter.send(&buy_back_packet(id, 0)).unwrap();
    assert_eq!(buy_backs(&paid.replies), vec![(id, RESULT_OK, 0)]);
    assert_eq!(counter.state().coins, 0);
}

#[test]
fn only_the_ten_most_recent_sales_can_be_bought_back() {
    const STACK: i32 = 900_010;
    let mut counter = Counter::new(0, |state| {
        state.items.push(InventoryItem {
            id: STACK,
            definition_id: 75060,
            tint: 0,
            count: 12,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        state.next_instance_id = Some(900_100);
    });
    counter.open();
    let ids: Vec<u32> = (0..11)
        .map(|_| {
            let sold = counter.send(&sell_packet(0, STACK, 1)).unwrap();
            completions(&sold.replies)[0].transaction
        })
        .collect();
    let paid = counter.state().coins;
    assert_eq!(paid, 11 * resale(75060));

    // The oldest sale fell out of the window: refused, nothing charged.
    let evicted = counter.send(&buy_back_packet(ids[0], 0)).unwrap();
    assert!(!evicted.changed);
    assert_eq!(
        buy_backs(&evicted.replies),
        vec![(ids[0], coin_shop::RESULT_FAILURE, 0)]
    );
    assert_eq!(counter.state().coins, paid);

    for id in [ids[1], ids[10]] {
        let back = counter.send(&buy_back_packet(id, 0)).unwrap();
        assert_eq!(buy_backs(&back.replies), vec![(id, RESULT_OK, 0)]);
    }
    assert_eq!(counter.state().coins, paid - 2 * resale(75060));
}

#[test]
fn buy_back_window_is_bounded_store_data() {
    fn store_file(window: Option<u32>) -> Value {
        let mut document = json!({
            "schema_version": 1,
            "marketplace": {
                "enabled_on_handshake": false,
                "orders_enabled": false,
                "payment_sources_enabled": false,
                "not_available_message": "disabled"
            },
            "items": [store_row(75060, Some(750), Some(375), false)]
        });
        if let Some(window) = window {
            document["buy_back_window"] = json!(window);
        }
        document
    }

    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/build/w172-fixtures");
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join(format!(
        "store-window-{}-{}.json",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));

    // Omitting the policy keeps the ten-sale default; the existing window test covers it.
    write_json(&path, &store_file(None));
    StoreCatalogue::load(&path).expect("default window loads");

    write_json(&path, &store_file(Some(3)));
    let catalogue = StoreCatalogue::load(&path).expect("custom window loads");
    let inventory_path = inventory_content();
    let mut state = Inventory::load(&inventory_path, None)
        .unwrap()
        .state()
        .clone();
    state.coins = 0;
    state.items.push(InventoryItem {
        id: 900_030,
        definition_id: 75060,
        tint: 0,
        count: 4,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    state.next_instance_id = Some(900_100);
    let inventory = Inventory::load(&inventory_path, Some(state)).unwrap();
    let mut shop = CoinShop::new(
        &inventory_path,
        inventory,
        catalogue,
        usize::MAX,
        None,
        Wallet::default(),
    )
    .unwrap();
    let ids: Vec<u32> = (0..4)
        .map(|_| shop.sell(900_030, 1).unwrap().transaction_id)
        .collect();
    assert!(
        shop.buy_back(ids[0]).unwrap().result_code != RESULT_OK,
        "the fourth-oldest sale falls out of a three-sale window"
    );
    assert_eq!(shop.buy_back(ids[3]).unwrap().result_code, RESULT_OK);

    for invalid in [0, 101] {
        write_json(&path, &store_file(Some(invalid)));
        assert!(
            StoreCatalogue::load(&path).is_err(),
            "window {invalid} must be outside the validated range"
        );
    }
    let _ = fs::remove_file(path);
}

#[test]
fn buy_back_keeps_the_open_shop_and_range_checks() {
    let mut counter = Counter::new(100, |_| {});
    // No shop open yet.
    let before = counter.state();
    let closed = counter.send(&buy_back_packet(1, 9)).unwrap();
    assert!(!closed.changed);
    assert_eq!(
        buy_backs(&closed.replies),
        vec![(1, coin_shop::RESULT_INVALID_MERCHANT, 9)]
    );
    assert_eq!(counter.state(), before);

    counter.open();
    let sold = counter.send(&sell_packet(0, SELLABLE_INSTANCE, 1)).unwrap();
    let id = completions(&sold.replies)[0].transaction;
    let after_sale = counter.state();

    // Out of range: refused, and the sale stays available for the right place.
    let far = [
        counter.position[0] + 1.0e6,
        counter.position[1],
        counter.position[2],
    ];
    let refused = counter
        .service
        .handle_coin_store(
            &mut counter.shop,
            &buy_back_packet(id, 0),
            counter.zone,
            far,
        )
        .unwrap();
    assert!(!refused.changed);
    assert_eq!(
        buy_backs(&refused.replies),
        vec![(id, coin_shop::RESULT_INVALID_MERCHANT, 0)]
    );
    assert_eq!(counter.state(), after_sale);

    // Malformed lengths: a failure reply that echoes nothing, no change.
    for length in [4, 8, 11, 13, 20] {
        let mut packet = buy_back_packet(id, 0);
        packet.resize(length, 0);
        let outcome = counter.send(&packet).unwrap();
        assert!(!outcome.changed, "length {length}");
        assert_eq!(
            buy_backs(&outcome.replies),
            vec![(0, coin_shop::RESULT_FAILURE, 0)],
            "length {length}"
        );
        assert_eq!(counter.state(), after_sale, "length {length}");
    }

    // The shop closes: refused again, then a fresh window can still buy it back.
    counter.service.close(VENDOR_ID).unwrap();
    let closed = counter.send(&buy_back_packet(id, 0)).unwrap();
    assert_eq!(
        buy_backs(&closed.replies),
        vec![(id, coin_shop::RESULT_INVALID_MERCHANT, 0)]
    );
    assert_eq!(counter.state(), after_sale);
    counter.open();
    let back = counter.send(&buy_back_packet(id, 0)).unwrap();
    assert_eq!(buy_backs(&back.replies), vec![(id, RESULT_OK, 0)]);
    assert_eq!(counter.state().coins, 100);
}

// ---------------------------------------------------------------------------------------------
// Service layer: definitions, list update, gifts
// ---------------------------------------------------------------------------------------------

#[test]
fn item_definitions_are_answered_from_the_item_catalogue() {
    let mut counter = Counter::new(100, |_| {});
    let before = counter.state();
    let unknown = 0x7fff_fff0;
    let cases: Vec<(&str, Vec<u8>, bool, Vec<u32>)> = vec![
        (
            "all known",
            definitions_packet(&[8, 75060]),
            true,
            vec![8, 75060],
        ),
        (
            "an unknown id makes it incomplete",
            definitions_packet(&[8, unknown, 75060]),
            false,
            vec![8, 75060],
        ),
        (
            "duplicates collapse",
            definitions_packet(&[75060, 8, 75060, 8]),
            true,
            vec![75060, 8],
        ),
        (
            "ids above i32 are unknown",
            definitions_packet(&[u32::MAX]),
            false,
            vec![],
        ),
        ("empty request", definitions_packet(&[]), true, vec![]),
        (
            "only unknown",
            definitions_packet(&[unknown]),
            false,
            vec![],
        ),
        // Malformed or oversized lists get an empty incomplete answer, never silence.
        (
            "count past the cap",
            definitions_packet(&vec![8; 4097]),
            false,
            vec![],
        ),
        (
            "negative count",
            [vec![165, 0, 2, 0], (-1i32).to_le_bytes().to_vec()].concat(),
            false,
            vec![],
        ),
        (
            "count longer than the body",
            [
                vec![165, 0, 2, 0],
                3i32.to_le_bytes().to_vec(),
                8u32.to_le_bytes().to_vec(),
            ]
            .concat(),
            false,
            vec![],
        ),
        (
            "trailing bytes",
            [definitions_packet(&[8]), vec![0]].concat(),
            false,
            vec![],
        ),
        ("header only", vec![165, 0, 2, 0], false, vec![]),
    ];
    for (label, packet, complete, ids) in cases {
        let outcome = counter.send(&packet).unwrap();
        assert!(!outcome.changed, "{label}");
        assert_eq!(outcome.replies.len(), 1, "{label}: one reply per request");
        assert_eq!(
            parse_definitions(&outcome.replies[0]),
            (complete, ids),
            "{label}"
        );
        assert_eq!(counter.state(), before, "{label}: state changed");
    }
    // The largest accepted list is answered in full and stays bounded.
    let all_eight = counter.send(&definitions_packet(&vec![8; 4096])).unwrap();
    assert_eq!(parse_definitions(&all_eight.replies[0]), (true, vec![8]));
}

#[test]
fn the_dynamic_list_request_gets_the_empty_list_and_malformed_ones_error() {
    let mut counter = Counter::new(100, |_| {});
    let before = counter.state();
    let outcome = counter.send(&[165, 0, 8, 0]).unwrap();
    assert!(!outcome.changed);
    assert_eq!(outcome.replies, vec![vec![165, 0, 9, 0, 0, 0, 0, 0]]);
    for packet in [vec![165, 0, 8, 0, 0], vec![165, 0, 8, 0, 0, 0, 0, 0]] {
        assert!(counter.send(&packet).is_err());
    }
    assert_eq!(counter.state(), before);
}

#[test]
fn gifts_are_refused_visibly_and_change_nothing() {
    let mut counter = Counter::new(1_000_000, |_| {});
    let before = counter.state();
    // The refusal does not depend on an open shop; it never touches either account.
    for open in [false, true] {
        if open {
            counter.open();
        }
        let outcome = counter.send(&gift_packet()).unwrap();
        assert!(!outcome.changed);
        assert_eq!(outcome.result_code, coin_shop::RESULT_SERVICE_UNAVAILABLE);
        assert_eq!(outcome.replies.len(), 1);
        let reply = &outcome.replies[0];
        assert_eq!(&reply[..4], &[165, 0, 18, 0]);
        assert_eq!(
            u32::from_le_bytes(reply[4..8].try_into().unwrap()),
            coin_shop::RESULT_SERVICE_UNAVAILABLE
        );
        assert_eq!(counter.state(), before);
    }
    // Malformed gift lists and absurd counts are errors, not grants.
    let mut huge = gift_packet();
    huge[20..24].copy_from_slice(&i32::MAX.to_le_bytes());
    for packet in [
        huge,
        gift_packet()[..23].to_vec(),
        [gift_packet(), vec![0]].concat(),
    ] {
        assert!(counter.send(&packet).is_err());
    }
    assert_eq!(counter.state(), before);
}

// ---------------------------------------------------------------------------------------------
// Service layer: Marketplace rows
// ---------------------------------------------------------------------------------------------

#[test]
fn marketplace_replies_match_the_recovered_client_readers_byte_for_byte() {
    let mut counter = Counter::new(100, |_| {});
    let before = counter.state();
    for (label, request, expected) in marketplace_table(true) {
        let outcome = counter.shop.handle(&request).unwrap();
        assert!(!outcome.changed, "{label}");
        assert_eq!(outcome.result_code, 1, "{label}");
        assert_eq!(
            outcome.replies.first(),
            Some(&expected),
            "{label}: first reply"
        );
        if !matches!(label, "wallet 66/10" | "status 66/12") {
            // The wallet and store-status answers also push the catalogue after their reply;
            // its bytes are asserted by the gateway and shipped-row tests.
            assert_eq!(outcome.replies, vec![expected], "{label}");
        }
    }
    assert_eq!(counter.state(), before);
}

#[test]
fn marketplace_payment_like_rows_stay_refused_or_free() {
    let mut counter = Counter::new(100, |_| {});
    let before = counter.state();
    // T230: the shipped content enables Station Cash orders, but item 8 has no Station Cash row,
    // so both preview and place-order are refused without touching state. The bare rows are
    // malformed now that an order body is parsed.
    assert!(counter.shop.handle(&marketplace(1)).is_err());
    assert!(counter.shop.handle(&marketplace(3)).is_err());
    let preview = counter
        .shop
        .handle(&sc_preview(21, 8, 1))
        .expect("unlisted preview parses");
    assert_eq!(preview.result_code, station_cash::ORDER_FAILURE);
    assert_eq!(preview.replies[0][..4], [66, 0, 2, 0]);
    assert!(!preview.changed);
    let order = counter
        .shop
        .handle(&sc_order(22, 8, 1))
        .expect("unlisted order parses");
    assert_eq!(order.result_code, station_cash::ORDER_FAILURE);
    assert_eq!(order.replies[0][..4], [66, 0, 4, 0]);
    assert_eq!(
        u32::from_le_bytes(order.replies[0][4..8].try_into().unwrap()),
        22
    );
    assert_eq!(
        u32::from_le_bytes(order.replies[0][8..12].try_into().unwrap()),
        station_cash::ORDER_FAILURE
    );
    assert!(!order.changed);
    // The wallet is zero and both payment sources start off.
    let wallet = counter.shop.handle(&marketplace(10)).unwrap();
    assert_eq!(&wallet.replies[0][9..17], &[0; 8]);
    assert_eq!(
        counter.shop.marketplace_packets()[1],
        vec![66, 0, 33, 0, 0, 0]
    );
    // SKU lists (Station Cash and membership) are explicitly empty.
    for request in [marketplace(14), marketplace(22)] {
        let reply = &counter.shop.handle(&request).unwrap().replies[0];
        assert_eq!(&reply[4..8], &1u32.to_le_bytes());
        assert_eq!(&reply[reply.len() - 4..], &[0; 4]);
    }
    assert_eq!(counter.state(), before);
}

#[test]
fn unrecovered_marketplace_rows_and_malformed_requests_change_nothing() {
    let mut counter = Counter::new(100, |_| {});
    let before = counter.state();
    // Bundle content (27): no reply, no state.
    let outcome = counter.shop.handle(&marketplace(27)).unwrap();
    assert!(outcome.replies.is_empty() && !outcome.changed, "66/27");
    let mut long = marketplace(14);
    long.resize(65, 0);
    let bad_locales = [
        [locale_request(16, "en_US"), vec![0]].concat(),
        locale_request(16, "en_US")[..12].to_vec(),
        locale_request(20, &"x".repeat(33)),
        {
            let mut negative = marketplace(25);
            negative.extend((-1i32).to_le_bytes());
            negative
        },
        marketplace(16),
        marketplace(25),
    ];
    let bare_with_tail = [
        [marketplace(10), vec![0]].concat(),
        [marketplace(18), vec![0]].concat(),
        [marketplace(38), vec![0]].concat(),
    ];
    for packet in bad_locales.into_iter().chain(bare_with_tail).chain([long]) {
        assert!(
            counter.shop.handle(&packet).is_err(),
            "must be refused: {packet:02x?}"
        );
    }
    assert_eq!(counter.state(), before);
}

// ---------------------------------------------------------------------------------------------
// Routed through the gateway
// ---------------------------------------------------------------------------------------------

fn traced(fixture: &Fixture, state: GameplayState) -> Gateway {
    let mut gateway = fixture.connect(state);
    gateway.set_trace_enabled(true);
    gateway
}

fn deliver(gateway: &mut Gateway, packet: &[u8]) -> GatewayOutcome {
    gateway.handle_at(&wrap(packet), NOW).unwrap()
}

#[test]
fn the_login_time_marketplace_requests_each_get_one_reply_through_the_gateway() {
    let fixture = Fixture::new();
    let mut gateway = traced(&fixture, fixture.state());
    let before = gateway.snapshot();
    for (label, request, expected) in marketplace_table(false) {
        let outcome = deliver(&mut gateway, &request);
        assert_eq!(inner_packets(&outcome), vec![expected], "{label}");
        // Nothing changes, so nothing is saved and no world state moves.
        assert!(
            !outcome.state_changed && !outcome.gameplay_changed,
            "{label}"
        );
        assert!(
            !outcome.began_zoning && outcome.nearby.is_empty(),
            "{label}"
        );
        assert_eq!(gateway.snapshot(), before, "{label}");
        let event = outcome.trace.last().expect("trace is enabled");
        assert_eq!(event.handler, "merchant", "{label}");
        assert_eq!(event.reason, None, "{label}");
    }
}

fn contains_u32(bytes: &[u8], value: u32) -> bool {
    bytes
        .windows(4)
        .any(|window| u32::from_le_bytes(window.try_into().unwrap()) == value)
}

/// T230: a fixture content package with no `station_cash` block (feature off) answers the
/// wallet, preview, order and store-status rows with the disabled shapes and changes nothing.
#[test]
fn disabled_station_cash_policy_answers_every_marketplace_row_without_changes() {
    let fixture = Fixture::new();
    let mut gateway = fixture.connect(fixture.state());
    let before = gateway.snapshot();
    assert!(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .and_then(|gameplay| gameplay.sc_wallet.as_ref())
            .is_none()
    );

    // 66/10 wallet: success status and a zero balance.
    let wallet = send(&mut gateway, &marketplace(10));
    assert_eq!(wallet.len(), 1);
    assert_eq!(&wallet[0][..4], &[66, 0, 11, 0]);
    assert_eq!(wallet_balance(&wallet), 0);

    // T453: both disabled replies echo the request's tracking id before result 0 (`00b2de20`,
    // `00b2ded0`); a body that does not decode still gets the disabled reply, with tracking 0.
    // 66/1 preview: tracking id, result 0, discount 0, total 0.
    let disabled_preview = |tracking: u8| {
        vec![vec![
            66, 0, 2, 0, tracking, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]]
    };
    let packets = send(&mut gateway, &sc_preview(31, 8, 1));
    assert_eq!(packets, disabled_preview(31));
    assert_eq!(send(&mut gateway, &marketplace(1)), disabled_preview(0));

    // 66/3 order: tracking id, result 0, the configured unavailable message, discount, total.
    let disabled_order = |tracking: u8| {
        let mut bytes = vec![66, 0, 4, 0, tracking, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0];
        bytes.extend_from_slice(b"disabled");
        bytes.extend([0; 8]);
        vec![bytes]
    };
    let packets = send(&mut gateway, &sc_order(32, 8, 1));
    assert_eq!(packets, disabled_order(32));
    assert_eq!(send(&mut gateway, &marketplace(3)), disabled_order(0));

    // 66/12 store status: the handshake flag is off, so no catalogue is pushed.
    let packets = send(&mut gateway, &marketplace(12));
    assert_eq!(packets, vec![vec![66, 0, 13, 0, 0]]);

    assert_eq!(gateway.snapshot(), before);
}

/// F387: the real store screen asks for its wallet (66/10, native sender `00b43bc0`, Lua
/// `RequestWalletInfo`) and never for the catalogue (66/12), so the wallet reply carries the
/// 66/24 handshake and the ride catalogue. World entry itself pushes nothing; the lead owns
/// that stream. `00b48700` is the 66/5 reader.
#[test]
fn wallet_reply_carries_the_marketplace_handshake_and_the_ride_catalogue() {
    let fixture = Fixture::new().with_sc();
    let (mut gateway, ready) = fixture.enter(fixture.state());
    assert!(
        ready.iter().all(|packet| !packet.starts_with(&[66, 0])),
        "world entry must not push the Marketplace"
    );
    let before = gateway.snapshot();
    let outcome = deliver(&mut gateway, &marketplace(10));
    let packets = inner_packets(&outcome);
    assert_eq!(&packets[0][..4], &[66, 0, 11, 0], "the wallet reply leads");
    assert_eq!(wallet_balance(&packets), 0);
    let market: Vec<&Vec<u8>> = packets
        .iter()
        .filter(|packet| packet.starts_with(&[66, 0]))
        .skip(1)
        .collect();
    let heads: Vec<&[u8]> = market.iter().map(|packet| &packet[..4]).collect();
    assert_eq!(
        heads,
        vec![
            &[66, 0, 24, 0][..],
            &[66, 0, 33, 0],
            &[66, 0, 7, 0],
            &[66, 0, 6, 0],
            &[66, 0, 9, 0],
            &[66, 0, 5, 0],
        ],
        "the handshake and the four catalogue rows follow the wallet"
    );
    assert_eq!(market[0][4], 1, "the Marketplace is enabled");
    // The fixture lists one ride. The bundle record is the reader's 185 bytes plus the two image
    // strings T657 now fills from the item definition (Wolf 14289: image set 6873, tint 121, so
    // seven payload bytes), the map key is the item id and the price sits at the `00b3da50` price
    // position.
    let store = market[5];
    let image_bytes = "6873".len() + "121".len();
    assert_eq!(
        store.len(),
        36 + 185 + image_bytes,
        "one 66/5 bundle with its image strings"
    );
    assert_eq!(
        u32::from_le_bytes(store[36..40].try_into().unwrap()),
        SC_RIDE_ITEM as u32,
        "the bundle map key is the ride item"
    );
    assert_eq!(
        u32::from_le_bytes(
            store[73 + image_bytes..77 + image_bytes]
                .try_into()
                .unwrap()
        ),
        SC_RIDE_PRICE,
        "the listed SC price"
    );
    // T526: the gateway emits the four ride children, and the bundle names its child's 66/6 group
    // (`00b3fa00` CategoryGroupId at record offset 110), so Wolf lists under Non-Winged.
    let tree = market[2];
    for name_id in [5_101_131u32, 5_101_134, 5_101_132, 5_101_133] {
        assert!(
            tree.windows(4)
                .any(|window| u32::from_le_bytes(window.try_into().unwrap()) == name_id),
            "66/7 carries the ride child name {name_id}"
        );
    }
    assert_eq!(
        u32::from_le_bytes(
            store[150 + image_bytes..154 + image_bytes]
                .try_into()
                .unwrap()
        ),
        3,
        "the bundle's category group is the Non-Winged child"
    );
    assert!(!outcome.state_changed && !outcome.gameplay_changed);
    assert_eq!(
        gateway.snapshot(),
        before,
        "the wallet reply changes nothing"
    );
}

/// F387: a package without Station Cash keeps the pre-T329 replies: no Marketplace packet at
/// world entry or in the wallet reply, even with the Marketplace handshake flag on, and 66/12
/// still answers 66/13 only.
#[test]
fn wallet_reply_sends_no_marketplace_packet_when_station_cash_is_disabled() {
    let fixture = Fixture::new();
    let path = fixture.content.join("stores.json");
    let mut stores = read_json(&path);
    stores["marketplace"]["enabled_on_handshake"] = json!(true);
    write_json(&path, &stores);
    let (mut gateway, ready) = fixture.enter(fixture.state());
    assert!(!ready.is_empty(), "the world-entry stream is present");
    assert!(
        ready.iter().all(|packet| !packet.starts_with(&[66, 0])),
        "world entry pushes no Marketplace packet"
    );
    let packets = send(&mut gateway, &marketplace(10));
    assert_eq!(
        packets.len(),
        1,
        "the disabled wallet reply carries no catalogue"
    );
    assert_eq!(&packets[0][..4], &[66, 0, 11, 0]);
    let packets = send(&mut gateway, &marketplace(12));
    assert_eq!(packets, vec![vec![66, 0, 13, 0, 1]]);
    // T380: the store screen's 66/38 still gets no reply.
    assert!(send(&mut gateway, &marketplace(38)).is_empty());
}

/// T380: the store screen sends the bare 66/38 when it opens (Lua `Opening`, `00c047a0` ->
/// `00c03920`). Its reply 66/39 is read by `00b3e490`: a counted list (`00b32d00`) of a u32
/// bundle id and a purchase byte (`00b2b590`). After a clean read the client refreshes
/// StoreItemRoot (`0099a790`), and only that refresh fills the bundle list once its commerce
/// processor is ready (`0099aa60`). T484: the list names every listing (the earn-only row is
/// absent), purchasable until owned; nothing is saved.
#[test]
fn store_screen_opening_gets_the_item_requirements_reply() {
    let fixture = Fixture::new().with_sc();
    let mut gateway = traced(&fixture, fixture.state());
    let before = gateway.snapshot();
    let outcome = deliver(&mut gateway, &marketplace(38));
    let mut expected = vec![66, 0, 39, 0, 1, 0, 0, 0];
    expected.extend((SC_RIDE_ITEM as u32).to_le_bytes());
    expected.push(1);
    assert_eq!(inner_packets(&outcome), vec![expected]);
    assert_eq!(outcome.trace.last().unwrap().handler, "merchant");
    assert!(!outcome.state_changed && !outcome.gameplay_changed);
    let refused = deliver(&mut gateway, &[marketplace(38), vec![0]].concat());
    assert!(refused.replies.is_empty());
    assert_eq!(refused.trace.last().unwrap().outcome, "rejected");
    assert_eq!(gateway.snapshot(), before);
}

/// T484: the shop is built with the login-time inventory, so the NPC route hands it the live bag
/// before a 66/38, a preview or an order reads ownership. A ride given in the session (the
/// production `realm give`) shows as owned in the next 66/39 and is refused as a second order.
#[test]
fn a_ride_given_in_the_session_is_owned_at_the_next_store_open() {
    let fixture = Fixture::new().with_sc();
    let mut state = fixture.state();
    state.sc_wallet = Some(Wallet {
        balance: 500,
        ..Wallet::default()
    });
    let mut gateway = fixture.connect(state);
    let flag = |packets: &[Vec<u8>]| {
        assert_eq!(packets.len(), 1);
        *packets[0].last().unwrap()
    };
    assert_eq!(flag(&send(&mut gateway, &marketplace(38))), 1);
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);
    send(
        &mut gateway,
        &chat::local_request(&format!("realm give {SC_RIDE_ITEM}")),
    );
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    assert_eq!(count_definition(&inventory_of(&gateway), SC_RIDE_ITEM), 1);
    assert_eq!(flag(&send(&mut gateway, &marketplace(38))), 0);

    let before = gateway.snapshot();
    let preview = send(&mut gateway, &sc_preview(1, SC_RIDE_ITEM as u32, 1));
    assert_eq!(&preview[0][..4], &[66, 0, 2, 0]);
    assert_eq!(
        u32::from_le_bytes(preview[0][8..12].try_into().unwrap()),
        station_cash::ORDER_FAILURE
    );
    let order = send(&mut gateway, &sc_order(2, SC_RIDE_ITEM as u32, 1));
    assert_eq!(order_result(&order), station_cash::ORDER_FAILURE);
    assert_eq!(gateway.snapshot(), before, "refusals change nothing");
}

/// F387: 66/29 (the store's close statistics, `PacketInGamePurchaseStoreClientStatistics` from
/// `00b23fc0`) and 117/41 (the one-time commerce session request, `0089b510`) stay unknown
/// traffic. The gateway drops both before the shared application router, sends no reply and
/// changes nothing; making them accepted traffic is a lead-owned routing decision.
#[test]
fn store_statistics_and_one_time_session_requests_stay_unrouted_and_change_nothing() {
    let fixture = Fixture::new().with_sc();
    let mut gateway = traced(&fixture, fixture.state());
    let before = gateway.snapshot();
    let mut statistics = marketplace(29);
    statistics.extend([0u8; 65]); // the observed 69-byte close report
    for packet in [statistics, vec![117, 0, 41, 0, 0, 0]] {
        let outcome = deliver(&mut gateway, &packet);
        assert!(inner_packets(&outcome).is_empty(), "{packet:02x?}");
        assert!(!outcome.state_changed && !outcome.gameplay_changed);
        assert!(
            outcome.applications.is_empty(),
            "unknown traffic must not reach the shared router: {packet:02x?}"
        );
        let event = outcome.trace.last().expect("trace is enabled");
        assert_eq!(event.handler, "gateway.application", "{packet:02x?}");
        assert_eq!(event.outcome, "rejected", "{packet:02x?}");
        assert_eq!(
            event.reason.as_deref(),
            Some("unknown application message"),
            "{packet:02x?}"
        );
    }
    assert_eq!(gateway.snapshot(), before);
}

/// T230: the real gateway path shows the wallet, debits exactly the listed price and grants the
/// ride once; a replay and an insufficient order leave the snapshot untouched.
#[test]
fn gateway_sc_wallet_and_ride_order_are_atomic_and_replay_once() {
    let fixture = Fixture::new().with_sc();
    let mut state = fixture.state();
    state.sc_wallet = Some(Wallet {
        balance: 150,
        ..Wallet::default()
    });
    let mut gateway = fixture.connect(state);
    assert_eq!(wallet_balance(&send(&mut gateway, &marketplace(10))), 150);
    let item_before = count_definition(&inventory_of(&gateway), SC_RIDE_ITEM);
    let coins_before = inventory_of(&gateway).coins;

    let outcome = gateway
        .handle_at(&wrap(&sc_order(1, SC_RIDE_ITEM as u32, 1)), NOW)
        .unwrap();
    let packets = inner_packets(&outcome);
    assert_eq!(order_result(&packets), station_cash::ORDER_SUCCESS);
    assert_eq!(wallet_balance(&packets), 150 - SC_RIDE_PRICE);
    assert!(outcome.state_changed);
    let after = inventory_of(&gateway);
    assert_eq!(count_definition(&after, SC_RIDE_ITEM), item_before + 1);
    assert_eq!(after.coins, coins_before, "an SC order creates no coins");
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .expect("gameplay")
            .sc_wallet
            .expect("wallet")
            .balance,
        150 - SC_RIDE_PRICE
    );

    let once = gateway.snapshot();
    let replay = gateway
        .handle_at(&wrap(&sc_order(1, SC_RIDE_ITEM as u32, 1)), NOW)
        .unwrap();
    assert_eq!(
        order_result(&inner_packets(&replay)),
        station_cash::ORDER_SUCCESS
    );
    assert!(!replay.state_changed);
    assert_eq!(gateway.snapshot(), once);

    // T484: the ride is now owned, which outranks the short balance (the balance reply for an
    // unowned ride is pinned in `marketplace_owned_rides`).
    let short = gateway
        .handle_at(&wrap(&sc_order(2, SC_RIDE_ITEM as u32, 1)), NOW)
        .unwrap();
    assert_eq!(
        order_result(&inner_packets(&short)),
        station_cash::ORDER_FAILURE
    );
    assert!(!short.state_changed);
    assert_eq!(gateway.snapshot(), once);
}

/// T230: opening the store (66/12) pushes the catalogue; the listed ride carries the scaled
/// price and the earn-only row is absent.
#[test]
fn gateway_sc_catalogue_lists_the_ride_and_skips_earn_only() {
    let fixture = Fixture::new().with_sc();
    let mut gateway = fixture.connect(fixture.state());
    let packets = send(&mut gateway, &marketplace(12));
    assert_eq!(&packets[0][..4], &[66, 0, 13, 0]);
    assert_eq!(packets[0][4], 1, "marketplace enabled");
    for opcode in [6u8, 7, 9, 24, 33] {
        assert!(
            packets
                .iter()
                .any(|packet| packet.starts_with(&[66, 0, opcode, 0])),
            "missing catalogue push 66/{opcode}"
        );
    }
    let store = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 5, 0]))
        .expect("store bundle packet");
    assert!(contains_u32(store, SC_RIDE_ITEM as u32));
    assert!(contains_u32(store, SC_RIDE_PRICE));
    assert!(!contains_u32(store, SC_EARN_ONLY_ITEM as u32));
}

/// T230: the wallet and the granted ride survive a reconnect through the character snapshot,
/// and the persisted replay window still refuses the old packet.
#[test]
fn gateway_sc_balance_and_ride_survive_reload() {
    let fixture = Fixture::new().with_sc();
    let mut state = fixture.state();
    state.sc_wallet = Some(Wallet {
        balance: 150,
        ..Wallet::default()
    });
    let mut gateway = fixture.connect(state);
    let bought = gateway
        .handle_at(&wrap(&sc_order(5, SC_RIDE_ITEM as u32, 1)), NOW)
        .unwrap();
    assert_eq!(
        order_result(&inner_packets(&bought)),
        station_cash::ORDER_SUCCESS
    );
    let bought_count = count_definition(&inventory_of(&gateway), SC_RIDE_ITEM);
    let saved = gateway.snapshot().gameplay.expect("gameplay snapshot");
    // The wallet is a save field: encode and decode the character state before reconnecting.
    let encoded = serde_json::to_vec(&saved).expect("character state serializes");
    let saved: GameplayState = serde_json::from_slice(&encoded).expect("character state parses");
    assert_eq!(
        saved.sc_wallet.as_ref().expect("wallet is saved").balance,
        50
    );
    assert_eq!(saved.sc_wallet.as_ref().unwrap().orders.len(), 1);
    let mut again = fixture.connect(saved);
    let after_reload = send(&mut again, &marketplace(10));
    assert_eq!(wallet_balance(&after_reload), 50);
    assert!(
        after_reload
            .iter()
            .any(|packet| packet.starts_with(&[66, 0, 24, 0])),
        "the catalogue is rebuilt after a reconnect"
    );
    assert_eq!(
        count_definition(&inventory_of(&again), SC_RIDE_ITEM),
        bought_count
    );
    let replay = again
        .handle_at(&wrap(&sc_order(5, SC_RIDE_ITEM as u32, 1)), NOW)
        .unwrap();
    assert_eq!(
        order_result(&inner_packets(&replay)),
        station_cash::ORDER_SUCCESS
    );
    assert!(!replay.state_changed);
    assert_eq!(wallet_balance(&inner_packets(&replay)), 50);
}

/// T230: a full bag refuses the order before the wallet or inventory changes.
#[test]
fn gateway_sc_full_bag_is_refused_atomically() {
    let fixture = Fixture::new().with_sc();
    let mut state = fixture.state();
    state.sc_wallet = Some(Wallet {
        balance: 150,
        ..Wallet::default()
    });
    let mut next = 1_000_000i32;
    while state.inventory.items.len() < 100 {
        state.inventory.items.push(InventoryItem {
            id: next,
            definition_id: 8,
            tint: 0,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        next += 1;
    }
    state.inventory.next_instance_id = Some(i64::from(next));
    let mut gateway = fixture.connect(state);
    let before = gateway.snapshot();
    let refused = gateway
        .handle_at(&wrap(&sc_order(6, SC_RIDE_ITEM as u32, 1)), NOW)
        .unwrap();
    assert_eq!(
        order_result(&inner_packets(&refused)),
        station_cash::ORDER_FAILURE
    );
    assert!(!refused.state_changed);
    assert_eq!(gateway.snapshot(), before);
}

#[test]
fn unanswered_marketplace_rows_are_known_traffic_and_malformed_ones_are_rejected() {
    let fixture = Fixture::new();
    let mut gateway = traced(&fixture, fixture.state());
    let before = gateway.snapshot();
    for sub in [27, 38] {
        let outcome = deliver(&mut gateway, &marketplace(sub));
        assert!(outcome.replies.is_empty());
        let event = outcome.trace.last().unwrap();
        assert_eq!(event.handler, "merchant");
        assert_ne!(event.reason.as_deref(), Some("unknown application message"));
    }
    for packet in [[marketplace(10), vec![0]].concat(), marketplace(16)] {
        let outcome = deliver(&mut gateway, &packet);
        assert!(outcome.replies.is_empty());
        assert_eq!(outcome.trace.last().unwrap().outcome, "rejected");
    }
    assert_eq!(gateway.snapshot(), before);
}

#[test]
fn gateway_sell_then_buy_back_once_then_replays_pay_nothing() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let start = inventory_of(&gateway);
    let stack = |state: &InventoryState| {
        state
            .items
            .iter()
            .find(|item| item.id == STACK_MANY)
            .map_or(0, |item| item.count)
    };

    let done = completions(&send(&mut gateway, &sell_packet(0, STACK_MANY, 2)));
    assert_eq!(done[0].result, RESULT_OK);
    let id = done[0].transaction;
    let sold = inventory_of(&gateway);
    assert_eq!(sold.coins, start.coins + 750);
    assert_eq!(stack(&sold), 1);

    let replies = send(&mut gateway, &buy_back_packet(id, 0));
    assert_eq!(buy_backs(&replies), vec![(id, RESULT_OK, 0)]);
    let back = inventory_of(&gateway);
    assert_eq!(back.coins, start.coins);
    assert_eq!(
        count_definition(&back, 75060),
        count_definition(&start, 75060)
    );

    let settled = gateway.snapshot();
    for _ in 0..3 {
        let replies = send(&mut gateway, &buy_back_packet(id, 0));
        assert_eq!(
            buy_backs(&replies),
            vec![(id, coin_shop::RESULT_FAILURE, 0)]
        );
        assert_eq!(gateway.snapshot(), settled, "a replay must change nothing");
    }

    // After a relog the sale history is gone and the shop is closed: refused, nothing charged.
    let mut again = fixture.connect(gateway.snapshot().gameplay.unwrap());
    let relogged = again.snapshot();
    let replies = send(&mut again, &buy_back_packet(id, 0));
    assert_eq!(
        buy_backs(&replies),
        vec![(id, coin_shop::RESULT_INVALID_MERCHANT, 0)]
    );
    assert_eq!(again.snapshot(), relogged);
}

#[test]
fn gateway_bundle_failure_rolls_back_buy_back_history() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let sold = completions(&send(&mut gateway, &sell_packet(0, STACK_MANY, 2)));
    let id = sold[0].transaction;
    let sold_state = gateway.snapshot();

    // A malformed 66/16 follows the valid buy-back. The shared sale history must not escape the
    // rejected bundle even though gameplay snapshots intentionally omit session-only shop state.
    let buy_back = buy_back_packet(id, 0);
    let failed = gateway.handle_at(&bundle(&[wrap(&buy_back), wrap(&marketplace(16))]), NOW);
    assert!(
        failed.is_err(),
        "the malformed child must reject the bundle"
    );
    assert_eq!(gateway.snapshot(), sold_state);

    let replies = send(&mut gateway, &buy_back);
    assert_eq!(buy_backs(&replies), vec![(id, RESULT_OK, 0)]);
    assert_eq!(
        count_definition(&inventory_of(&gateway), 75060),
        count_definition(&fixture.state().inventory, 75060)
    );
}

#[test]
fn gateway_buy_back_refusals_answer_the_client_and_change_nothing() {
    let fixture = Fixture::new();
    // A player with no shop open.
    let mut gateway = fixture.connect(fixture.state());
    let before = gateway.snapshot();
    let replies = send(&mut gateway, &buy_back_packet(1, 3));
    assert_eq!(
        buy_backs(&replies),
        vec![(1, coin_shop::RESULT_INVALID_MERCHANT, 3)]
    );
    assert_eq!(gateway.snapshot(), before);

    // An open shop, an id that was never a sale, and a malformed length.
    let (mut gateway, _) = fixture.at_open_shop();
    let before = gateway.snapshot();
    let replies = send(&mut gateway, &buy_back_packet(424_242, 0));
    assert_eq!(
        buy_backs(&replies),
        vec![(424_242, coin_shop::RESULT_FAILURE, 0)]
    );
    let replies = send(&mut gateway, &buy_back_packet(1, 0)[..11]);
    assert_eq!(buy_backs(&replies), vec![(0, coin_shop::RESULT_FAILURE, 0)]);
    assert_eq!(gateway.snapshot(), before);
    // Canary: the window still works.
    let done = completions(&send(&mut gateway, &sell_packet(0, STACK_ONE, 1)));
    assert_eq!(done[0].result, RESULT_OK);
}

#[test]
fn gateway_definitions_list_update_and_gifts_are_answered_without_state_change() {
    let fixture = Fixture::new();
    // No shop open: these three are session-free.
    let mut gateway = fixture.connect(fixture.state());
    let before = gateway.snapshot();

    let replies = send(&mut gateway, &definitions_packet(&[8, 75060, 0x7fff_fff0]));
    assert_eq!(replies.len(), 1);
    assert_eq!(parse_definitions(&replies[0]), (false, vec![8, 75060]));

    let replies = send(&mut gateway, &[165, 0, 8, 0]);
    assert_eq!(replies, vec![vec![165, 0, 9, 0, 0, 0, 0, 0]]);

    let replies = send(&mut gateway, &gift_packet());
    assert_eq!(replies.len(), 1);
    assert_eq!(&replies[0][..4], &[165, 0, 18, 0]);
    assert_eq!(
        u32::from_le_bytes(replies[0][4..8].try_into().unwrap()),
        coin_shop::RESULT_SERVICE_UNAVAILABLE
    );
    assert_eq!(gateway.snapshot(), before);
}

#[test]
fn catalogue_only_definition_is_delivered_before_the_coin_store_ack() {
    let fixture = Fixture::new();
    let record = Inventory::load(&fixture.content, None)
        .unwrap()
        .definition_record(CATALOGUE_ONLY_DEFINITION)
        .expect("generated catalogue row exists")
        .to_vec();
    let mut gateway = fixture.connect(fixture.state());
    let before = gateway.snapshot();

    let replies = send(
        &mut gateway,
        &definitions_packet(&[CATALOGUE_ONLY_DEFINITION as u32]),
    );
    assert_eq!(replies.len(), 2, "definition batch, then 165/3");
    assert_eq!(
        parse_definition_batch(&replies[0]),
        (1, record),
        "the uncaptured row precedes the acknowledgement"
    );
    assert_eq!(
        parse_definitions(&replies[1]),
        (true, vec![CATALOGUE_ONLY_DEFINITION as u32])
    );
    assert_eq!(gateway.snapshot(), before, "definition read changed state");
}

const LARGE_DEFINITION_BASE: i32 = 920_000;

/// Synthetic unowned catalogue rows of a chosen wire size, plus their requested ids.
fn large_definitions(count: usize, size: usize) -> (Vec<Value>, Vec<u32>) {
    let ids: Vec<u32> = (0..count)
        .map(|index| (LARGE_DEFINITION_BASE + index as i32) as u32)
        .collect();
    let rows = ids
        .iter()
        .map(|id| item_catalogue::sized_row(*id, size))
        .collect();
    (rows, ids)
}

#[test]
fn multiple_definition_batches_follow_request_order() {
    let (rows, mut ids) = large_definitions(2, 40 * 1024);
    // Descending ids: sorting the request would show in both the batches and the 165/3 list.
    ids.reverse();
    let fixture = Fixture::with_definitions(rows);
    let inventory = Inventory::load(&fixture.content, None).unwrap();
    let mut gateway = fixture.connect(fixture.state());
    let before = gateway.snapshot();

    let replies = send(&mut gateway, &definitions_packet(&ids));
    assert_eq!(
        replies.len(),
        3,
        "two 40 KiB rows cannot share a 60 KiB batch"
    );
    for (index, reply) in replies[..2].iter().enumerate() {
        let (_, payload) = parse_definition_batch(reply);
        assert_eq!(
            payload,
            inventory.definition_record(ids[index] as i32).unwrap(),
            "batch {index} must carry the requested row in order"
        );
    }
    assert_eq!(parse_definitions(&replies[2]), (true, ids.clone()));
    assert_eq!(
        gateway.snapshot(),
        before,
        "definition delivery is read-only"
    );
}

#[test]
fn one_definition_batch_sorts_rows_but_the_id_list_follows_request_order() {
    let (rows, mut ids) = large_definitions(2, 8);
    ids.reverse();
    let fixture = Fixture::with_definitions(rows);
    let inventory = Inventory::load(&fixture.content, None).unwrap();
    let mut gateway = fixture.connect(fixture.state());

    let replies = send(&mut gateway, &definitions_packet(&ids));
    assert_eq!(replies.len(), 2, "two tiny rows share one 35/37 batch");
    let (count, payload) = parse_definition_batch(&replies[0]);
    assert_eq!(count, 2);
    // The existing encoder writes records in id order inside one batch (record ids are the
    // definition ids); the batch sequence and the 165/3 list follow the request order.
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    let expected: Vec<u8> = sorted
        .iter()
        .flat_map(|id| {
            inventory
                .definition_record(*id as i32)
                .unwrap()
                .iter()
                .copied()
        })
        .collect();
    assert_eq!(
        payload, expected,
        "the batch keeps id order inside the batch"
    );
    assert_eq!(parse_definitions(&replies[1]), (true, ids));
}

#[test]
fn a_refused_bundle_leaves_the_npc_menu_closed() {
    // 26/8 opens the menu, then three large definition reads exceed the bundle budget. The
    // staged Gameplay owns the menu, so the refusal must discard it with the rest of the bundle.
    let (rows, ids) = large_definitions(60, 65_536);
    let fixture = Fixture::with_definitions(rows);
    let mut gateway = fixture.connect(fixture.state());
    let before = gateway.snapshot();

    let request = wrap(&definitions_packet(&ids));
    let refused = gateway.handle_at(
        &bundle(&[
            wrap(&npc_request(8, None)),
            request.clone(),
            request.clone(),
            request,
        ]),
        NOW,
    );
    assert!(refused.is_err(), "the bundle must be refused at the budget");
    assert_eq!(
        gateway.snapshot(),
        before,
        "the refused bundle changed state"
    );

    let outcome = gateway
        .handle_at(&wrap(&npc_request(10, Some(2))), NOW)
        .unwrap();
    assert!(
        !outcome
            .replies
            .iter()
            .any(|reply| reply[7..].starts_with(&[165, 0, 10, 0])),
        "a refused bundle must not leave the NPC menu open"
    );
    assert_eq!(gateway.snapshot(), before);

    // Positive control: the same interaction outside the refused bundle opens the shop.
    gateway
        .handle_at(&wrap(&npc_request(8, None)), NOW)
        .unwrap();
    let opened = gateway
        .handle_at(&wrap(&npc_request(10, Some(2))), NOW)
        .unwrap();
    assert!(
        inner_packets(&opened)
            .iter()
            .any(|packet| packet.starts_with(&[165, 0, 10, 0])),
        "the fixture must still be able to open the shop"
    );
}

#[test]
fn definition_reply_over_budget_is_refused_before_records_are_copied() {
    // Sixty-five maximum-size rows exceed the 4 MiB single-answer budget by construction.
    let (rows, ids) = large_definitions(65, 65_536);
    let fixture = Fixture::with_definitions(rows);
    let mut gateway = fixture.connect(fixture.state());
    let before = gateway.snapshot();

    let replies = send(&mut gateway, &definitions_packet(&ids));
    assert_eq!(
        replies.len(),
        1,
        "an empty 165/3 answer and no 35/37 payloads"
    );
    assert_eq!(parse_definitions(&replies[0]), (false, vec![]));
    assert_eq!(gateway.snapshot(), before, "the refusal changed state");

    // Positive control: sixty rows stay under the budget and are each delivered before the answer.
    let under = &ids[..60];
    let replies = send(&mut gateway, &definitions_packet(under));
    assert_eq!(replies.len(), under.len() + 1);
    assert_eq!(
        parse_definitions(replies.last().unwrap()),
        (true, under.to_vec())
    );
    assert_eq!(gateway.snapshot(), before);
}

#[test]
fn gateway_rejects_a_bundle_that_repeats_large_definition_requests() {
    // Three ~3.9 MiB answers fit the single-request budget but not the 8 MiB bundle budget.
    let (rows, ids) = large_definitions(60, 65_536);
    let fixture = Fixture::with_definitions(rows);
    let mut gateway = fixture.connect(fixture.state());
    let before = gateway.snapshot();

    let request = wrap(&definitions_packet(&ids));
    let refused = gateway.handle_at(
        &bundle(&[request.clone(), request.clone(), request.clone()]),
        NOW,
    );
    let error = match refused {
        Ok(_) => panic!("the bundle must not stage twelve MiB of replies"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("outbound budget"),
        "expected the reply budget refusal, got {error}"
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "the refused bundle changed state"
    );

    // Positive control: two identical children stay inside the budget and are both answered.
    let answered = gateway
        .handle_at(&bundle(&[request.clone(), request]), NOW)
        .unwrap();
    assert_eq!(answered.replies.len(), 2 * (ids.len() + 1));
    assert_eq!(gateway.snapshot(), before);
}
