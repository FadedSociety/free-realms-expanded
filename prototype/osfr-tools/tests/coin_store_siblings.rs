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
//! The fixture and packet helpers are copied from `vendor_sell.rs`; some go unused here.
#![allow(dead_code)]

use osfr_tools::{
    account_store::AccountRole,
    coin_shop::{self, CoinShop, CoinShopOutcome, StoreCatalogue},
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState},
    npc_interaction::{NpcAction, NpcInteraction, NpcInteractionError, NpcSelection},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const VENDOR_ID: u64 = 9_500_000_339;
/// Default-state fixtures from `lab/references/rust-gameplay/inventory.json`.
const SELLABLE_INSTANCE: i32 = 14; // definition 75060, sell price 375, not equipped
const EQUIPPED_INSTANCE: i32 = 34; // definition 21683, sell price 5, equipped
const UNCATALOGUED_INSTANCE: i32 = 4; // definition 19522, no store row
const NO_SALE_DEFINITION: i32 = 1180; // store row flagged no_sale
const CATALOGUE_ONLY_DEFINITION: i32 = 910_500;
const RESULT_OK: u32 = coin_shop::RESULT_SUCCESS;

fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn definition_row(id: i32) -> Value {
    definition_row_sized(id, 8)
}

/// One synthetic row of exactly `size` wire bytes; `size` is bounded by the inventory decoder
/// at 65,536 bytes.
fn definition_row_sized(id: i32, size: usize) -> Value {
    let mut bytes = id.to_le_bytes().to_vec();
    bytes.resize(size.max(8), 0x5a);
    json!({
        "id": id,
        "slot": 0,
        "class_id": 1,
        "default_tint": 0,
        "max_stack": 10,
        "model": "",
        "texture": "",
        "tint_alias": "",
        "effect": 0,
        "wield_type": 0,
        "serialized_hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "sha256": format!("{:x}", Sha256::digest(&bytes))
    })
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

fn sell_packet(merchant: u64, instance: i32, quantity: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 5, 0];
    packet.extend(merchant.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet
}

fn buy_packet(merchant: u64, transaction: u32, item: i32, quantity: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 4, 0];
    packet.extend(merchant.to_le_bytes());
    packet.extend(transaction.to_le_bytes());
    packet.extend(item.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet.extend(0u32.to_le_bytes());
    packet
}

/// The 165/6 record as `00c74ef0`/`00c71f30` read it. Field names beyond the result are (R).
#[derive(Debug, PartialEq)]
struct Completion {
    result: u32,
    leading_item: i32,
    kind: u32,
    merchant: u64,
    transaction: u32,
    timestamp: u64,
    definition: i32,
    tint: i32,
    quantity: u32,
    price: u32,
    remaining: u32,
}

fn parse_completion(packet: &[u8]) -> Completion {
    // header 4 + result 4 + item 4 + record (4+8+4+8+4+4+4+4+4) = 56: the client reader
    // overruns on anything shorter, which is how the earlier 52-byte frame failed.
    assert_eq!(packet.len(), 56, "completion must match the reader size");
    assert_eq!(&packet[..4], &[165, 0, 6, 0]);
    let u32_at = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    let u64_at = |at: usize| u64::from_le_bytes(packet[at..at + 8].try_into().unwrap());
    Completion {
        result: u32_at(4),
        leading_item: u32_at(8) as i32,
        kind: u32_at(12),
        merchant: u64_at(16),
        transaction: u32_at(24),
        timestamp: u64_at(28),
        definition: u32_at(36) as i32,
        tint: u32_at(40) as i32,
        quantity: u32_at(44),
        price: u32_at(48),
        remaining: u32_at(52),
    }
}

fn completions(replies: &[Vec<u8>]) -> Vec<Completion> {
    replies
        .iter()
        .filter(|packet| packet.starts_with(&[165, 0, 6, 0]))
        .map(|packet| parse_completion(packet))
        .collect()
}

fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
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
        let shop = CoinShop::new(&directory, inventory, catalogue, usize::MAX, None).unwrap();
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

const NOW: u64 = 10_000;
const NPC: u64 = 700;
const STACK_MANY: i32 = 900_001; // definition 75060 x3, sell 375 each
const STACK_NO_SALE: i32 = 900_003; // definition 1180
const STACK_ONE: i32 = 900_004; // definition 8 x1, sell 7
static NEXT: AtomicU64 = AtomicU64::new(0);

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn bundle(messages: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for message in messages {
        assert!(message.len() < 255);
        bytes.push(message.len() as u8);
        bytes.extend(message);
    }
    bytes
}

/// A bundle that uses the client's 16-bit child length, needed once a 165/2 list passes 255 bytes.
fn large_bundle(messages: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for message in messages {
        assert!(message.len() <= u16::MAX as usize);
        match message.len() {
            len if len < 255 => bytes.push(len as u8),
            // 255 is the escape itself, so that length needs the 32-bit form below.
            255 => {
                bytes.push(255);
                bytes.extend(255u16.to_be_bytes());
                bytes.extend(255u32.to_be_bytes());
            }
            len => {
                bytes.push(255);
                bytes.extend((len as u16).to_be_bytes());
            }
        }
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
        definitions.push(definition_row(CATALOGUE_ONLY_DEFINITION));
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
        write_json(
            &content.join("item-catalogue.json"),
            &json!({
                "schema_version": 1,
                "definitions": definitions
            }),
        );
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
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.position = [0.; 3];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, Some(state)).unwrap();
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway.handle_at(&packet, NOW).unwrap();
        }
        gateway
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
            inner.iter().any(|p| p.starts_with(&[66, 0, 42, 0])),
            "the shop did not open"
        );
        (gateway, inner)
    }
}

/// Tunnel payloads of every reply, with the 7-byte application wrapper removed.
fn inner_packets(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome.replies.iter().map(|p| p[7..].to_vec()).collect()
}

fn inventory_of(gateway: &Gateway) -> InventoryState {
    gateway.snapshot().gameplay.unwrap().inventory
}

fn send(gateway: &mut Gateway, packet: &[u8]) -> Vec<Vec<u8>> {
    inner_packets(&gateway.handle_at(&wrap(packet), NOW).unwrap())
}

// ---------------------------------------------------------------------------------------------
// Packet builders and readers for the sibling requests
// ---------------------------------------------------------------------------------------------

fn buy_back_packet(transaction: u32, context: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 12, 0];
    packet.extend(transaction.to_le_bytes());
    packet.extend(context.to_le_bytes());
    packet
}

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

/// One-record 35/37 batch. Rows have no per-record length, so the test fixture retains the
/// expected record separately and compares the concatenated payload.
fn parse_definition_batch(packet: &[u8]) -> (i32, Vec<u8>) {
    assert_eq!(&packet[..4], &[35, 0, 37, 0]);
    let payload_len = i32::from_le_bytes(packet[4..8].try_into().unwrap());
    let count = i32::from_le_bytes(packet[8..12].try_into().unwrap());
    assert_eq!(payload_len as usize + 8, packet.len());
    assert_eq!(count, 1);
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
    assert_eq!(after_sale.coins, 100 + 375);
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
    assert_eq!(counter.state().coins, 10 * 375);
    assert_eq!(count_definition(&counter.state(), 75060), original - 10);

    let partial = counter.send(&buy_back_packet(id, 3)).unwrap();
    assert!(partial.changed);
    assert_eq!(buy_backs(&partial.replies), vec![(id, RESULT_OK, 3)]);
    assert_eq!(counter.state().coins, 7 * 375);
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

    set_coins(&mut counter, 374);
    let before = counter.state();
    let short = counter.send(&buy_back_packet(id, 0)).unwrap();
    assert!(!short.changed);
    assert_eq!(
        buy_backs(&short.replies),
        vec![(id, coin_shop::RESULT_FAILURE, 0)]
    );
    assert_eq!(counter.state(), before);

    set_coins(&mut counter, 375);
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
    assert_eq!(paid, 11 * 375);

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
    assert_eq!(counter.state().coins, paid - 2 * 375);
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
    let mut shop = CoinShop::new(&inventory_path, inventory, catalogue, usize::MAX, None).unwrap();
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
        assert_eq!(outcome.replies, vec![expected], "{label}");
    }
    assert_eq!(counter.state(), before);
}

#[test]
fn marketplace_payment_like_rows_stay_refused_or_free() {
    let mut counter = Counter::new(100, |_| {});
    let before = counter.state();
    // Order preview and place-order: never enabled, with the configured message (R).
    let preview = counter.shop.handle(&marketplace(1)).unwrap();
    assert_eq!(preview.result_code, coin_shop::MARKETPLACE_RESULT_DISABLED);
    assert_eq!(
        preview.replies,
        vec![[vec![66, 0, 2, 0], vec![0; 16]].concat()]
    );
    let order = counter.shop.handle(&marketplace(3)).unwrap();
    assert_eq!(order.result_code, coin_shop::MARKETPLACE_RESULT_DISABLED);
    assert!(order.replies[0].starts_with(&[66, 0, 4, 0, 0, 0, 0, 0]));
    assert!(String::from_utf8_lossy(&order.replies[0]).contains("Not available"));
    // The wallet is zero and both payment sources start off.
    let wallet = counter.shop.handle(&marketplace(10)).unwrap();
    assert_eq!(&wallet.replies[0][9..17], &[0; 8]);
    assert_eq!(
        counter.shop.marketplace_initial_packets()[1],
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
    // Bundle content (27) and item requirements (38): no catalogue, no reply, no state.
    for sub in [27, 38] {
        let outcome = counter.shop.handle(&marketplace(sub)).unwrap();
        assert!(outcome.replies.is_empty() && !outcome.changed, "66/{sub}");
    }
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
        .map(|id| definition_row_sized(*id as i32, size))
        .collect();
    (rows, ids)
}

#[test]
fn multiple_definition_batches_follow_request_order() {
    let (rows, ids) = large_definitions(2, 40 * 1024);
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
    assert_eq!(parse_definitions(&replies[2]), (true, ids));
    assert_eq!(
        gateway.snapshot(),
        before,
        "definition delivery is read-only"
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

    // Canary: sixty rows stay under the budget and are each delivered before the answer.
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
        &large_bundle(&[request.clone(), request.clone(), request.clone()]),
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

    // Canary: two identical children stay inside the budget and are both answered.
    let answered = gateway
        .handle_at(&large_bundle(&[request.clone(), request]), NOW)
        .unwrap();
    assert_eq!(answered.replies.len(), 2 * (ids.len() + 1));
    assert_eq!(gateway.snapshot(), before);
}
