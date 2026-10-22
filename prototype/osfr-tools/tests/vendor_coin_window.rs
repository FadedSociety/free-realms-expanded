//! T362: Roosey's 165/10 coin merchant window lists her stock at the server prices.
//!
//! Client evidence: `00c78fb0` reads the 165/10 body (header, the six-dword merchant record
//! `00b33020`, count, then 38-byte rows `00b29cd0`), stores it in the merchant stock data source
//! and raises `HandlerMerchant:Show("CoinStore")` (`00c78c10`). The `NpcMerchantV2` window's
//! CoinStore mode displays that data source (`ScriptsBase.lua` `NpcMerchantV2:setMode`). The
//! row's coin price is the resolved item definition's `+0x24` (`00cff140` case 27; serialized
//! dword 7), not a row field, so the server pushes a 35/37 definition update whenever the loaded
//! definition differs from `stores.json`.

#[allow(dead_code)]
#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/round70_content.rs"]
mod round70_content;

use gateway_vendor::{
    NOW, count_definition, inner_packets, interact, inventory_of, login, select, send, wrap,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    inventory::Inventory,
    npc_interaction::{NpcAction, NpcInteraction, decode_coin_merchant_list_165_10},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const ROOSEY: u64 = 9_500_000_255;
const RING: i32 = 75_260;
const RING_PRICE: u32 = 75;
const STOCK: [i32; 6] = [75_030, 75_250, 75_260, 75_278, 75_294, 75_297];

static NEXT: AtomicU64 = AtomicU64::new(0);

fn data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("t362-roosey")
}

struct Fixture {
    content: PathBuf,
    reference: PathBuf,
    option: u32,
    position: [f32; 3],
    prices: BTreeMap<i32, u32>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl Fixture {
    fn new() -> Self {
        let reference = round70_content::reference();
        let content = round70_content::build(
            &fixture_root(),
            &format!("t362-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
            &[],
        );
        let data = data();
        for file in ["stores.json", "zones.json"] {
            fs::copy(data.join(file), content.join(file)).expect("fixture content file copies");
        }
        let vendors = read_json(&data.join("vendors.json"));
        let vendor = vendors["vendors"]
            .as_array()
            .expect("vendor records are an array")
            .iter()
            .find(|vendor| vendor["npc_id"].as_u64() == Some(ROOSEY))
            .expect("Roosey's vendor row exists")
            .clone();
        write_json(
            &content.join("vendors.json"),
            &json!({"schema_version": 1, "vendors": [vendor]}),
        );
        let npcs = read_json(&data.join("npcs/cobblestone_village.json"));
        let npc = npcs["records"]
            .as_array()
            .expect("NPC records are an array")
            .iter()
            .find(|npc| npc["npc_id"].as_u64() == Some(ROOSEY))
            .expect("Roosey's NPC record exists")
            .clone();
        write_json(
            &content.join("npcs/roosey.json"),
            &json!({"schema_version": 1, "records": [npc]}),
        );
        write_json(
            &content.join("npc_spawns_extra.json"),
            &json!({"schema_version": 1, "records": []}),
        );

        let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .expect("fixture NPC content loads");
        let (zone, position) = service.location(ROOSEY).expect("Roosey has a location");
        let menu = service
            .interact(ROOSEY, zone, position, None)
            .expect("Roosey opens her menu");
        let option = u32::try_from(
            menu.options
                .iter()
                .find(|option| option.action == NpcAction::Shop)
                .expect("Roosey exposes the Shop option")
                .id,
        )
        .expect("the shop option id fits the wire field");
        let stores = read_json(&data.join("stores.json"));
        let prices = stores["items"]
            .as_array()
            .expect("store rows are an array")
            .iter()
            .filter_map(|row| {
                Some((
                    row["item_id"].as_i64()? as i32,
                    row["purchase_price_coins"].as_u64()? as u32,
                ))
            })
            .filter(|(id, _)| STOCK.contains(id))
            .collect();
        Self {
            content,
            reference,
            option,
            position,
            prices,
        }
    }

    fn character(&self) -> CharacterState {
        let mut character =
            Gateway::reference_state(&self.reference).expect("reference character loads");
        character.zone_id = 0;
        character.position = self.position;
        character.rotation = [0.0; 3];
        character
    }

    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("fixture gameplay content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = 1_000;
        state.inventory.next_instance_id = Some(1);
        state
    }

    fn connect(&self, state: GameplayState) -> Gateway {
        let mut gateway = Gateway::load_gameplay(
            &self.reference,
            self.character(),
            &self.content,
            Some(state),
        )
        .expect("gateway loads the Roosey content");
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        let guid = gateway.state().guid;
        for packet in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway
                .handle_at(&packet, NOW)
                .expect("initialization message handles");
        }
        gateway
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("fixture JSON reads"))
        .expect("fixture JSON parses")
}

fn write_json(path: &Path, value: &Value) {
    fs::write(
        path,
        serde_json::to_vec(value).expect("fixture JSON serializes"),
    )
    .expect("fixture JSON writes");
}

/// The production path: 26/8 opens the menu and 26/10 selects the Shop option.
fn open_shop(gateway: &mut Gateway, fixture: &Fixture) -> Vec<Vec<u8>> {
    let menu = send(gateway, &interact(ROOSEY));
    assert!(
        menu.iter().any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "26/8 must answer with the 26/9 menu"
    );
    send(gateway, &select(ROOSEY, fixture.option))
}

/// The 35/37 batch the shop pushes, when any loaded definition price differs.
fn definition_batch(replies: &[Vec<u8>]) -> Option<&[u8]> {
    replies
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 37, 0]))
        .map(Vec::as_slice)
}

/// The 165/6 completion (`00c74ef0`): result at offset 4, then the transaction record.
fn coin_completion(packets: &[Vec<u8>]) -> &[u8] {
    packets
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 6, 0]))
        .map(Vec::as_slice)
        .expect("a 165/6 transaction completion follows a 165/4 buy")
}

fn word(packet: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(packet[at..at + 4].try_into().expect("packet field"))
}

#[test]
fn roosey_coin_window_lists_her_stock_at_server_prices() {
    let fixture = Fixture::new();
    let inventory = Inventory::load(&fixture.content, None).expect("fixture inventory loads");
    let mut gateway = fixture.connect(fixture.state());
    let replies = open_shop(&mut gateway, &fixture);

    let list = replies
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
        .expect("the Shop choice sends the 165/10 coin merchant list");
    assert!(
        !replies
            .iter()
            .any(|packet| packet.starts_with(&[66, 0, 42, 0])),
        "a coin vendor must not open the 66/42 Station Cash window"
    );
    let group = decode_coin_merchant_list_165_10(list).expect("165/10 matches the client reader");

    // The record carries the merchant identity the client copies into 165/4 and 165/5. The
    // first field is the per-shop context; there is no time-left dword in this message.
    assert_eq!(group.fields[0], NOW as u32, "the record starts at byte 4");
    assert_eq!(
        u64::from(group.fields[3]) | (u64::from(group.fields[4]) << 32),
        ROOSEY,
        "the record merchant GUID is Roosey"
    );
    assert_eq!(group.entries.len(), STOCK.len(), "Roosey's six stock rows");
    let ids: Vec<i32> = group
        .entries
        .iter()
        .map(|entry| entry.item_id as i32)
        .collect();
    for id in STOCK {
        assert!(ids.contains(&id), "row {id} is missing: {ids:?}");
    }

    // The row's display data comes from the definition; the client reads it directly.
    for entry in &group.entries {
        let presentation = inventory
            .presentation(entry.item_id as i32)
            .expect("the stock row resolves an item definition");
        assert_eq!(entry.name_id, presentation.name_id as u32);
        assert_eq!(entry.description_id, presentation.description_id as u32);
        assert_eq!(entry.icon_id, presentation.icon_id as u32);
        assert_eq!(entry.tint_value, presentation.icon_tint as u32);
        assert_eq!(entry.quantity_available, 0, "unlimited server stock");
        assert!(!entry.members_only);
        assert!(entry.can_buy);
        assert_eq!(entry.available_tint_group_id, 0);
    }

    // The displayed coin price is the definition's coin purchase price. A row the login
    // catalogue did not carry, or whose loaded definition price differs from stores.json, is
    // pushed through the paired 35/37 batch.
    let mut updates = 0;
    for id in STOCK {
        let price = fixture.prices[&id];
        let loaded = inventory
            .definition_coin_price(id)
            .expect("the definition carries a coin price");
        let raw = inventory
            .definition_record(id)
            .expect("the row has a definition");
        assert_eq!(
            &raw[28..32],
            &loaded.to_le_bytes(),
            "loaded price wire offset"
        );
        let captured = inventory.definition_was_captured(id);
        if captured && loaded == price {
            continue;
        }
        updates += 1;
        let record = inventory
            .definition_record_with_coin_price(id, price)
            .expect("the definition can carry the corrected price");
        // Reader-derived pin: `008e3760` stores serialized dword 7 (bytes 28..32) at `+0x24`,
        // the column `00cff140` case 27 displays.
        assert_eq!(&record[28..32], &price.to_le_bytes(), "price wire offset");
        let batch = definition_batch(&replies)
            .unwrap_or_else(|| panic!("row {id} needs a 35/37 definition update"));
        assert!(
            batch.windows(record.len()).any(|window| window == record),
            "the 35/37 batch does not carry item {id} at {price} coins"
        );
    }
    if updates == 0 {
        assert!(
            definition_batch(&replies).is_none(),
            "no definition needs an update, so no 35/37 batch should be sent"
        );
    }

    // Buy the 75-coin ring through the native 165/4 field order (`00c77230` -> the serializer
    // `00c75ed0`): merchant u64, merchant-record context, definition id, tint, then quantity.
    // Every buy from one open shop carries the same context, so buying the ring again is a
    // second purchase, not a retry.
    let merchant = u64::from(group.fields[3]) | (u64::from(group.fields[4]) << 32);
    let mut buy = vec![165, 0, 4, 0];
    buy.extend(merchant.to_le_bytes());
    buy.extend(group.fields[0].to_le_bytes());
    buy.extend(RING.to_le_bytes());
    buy.extend(0u32.to_le_bytes());
    buy.extend(1u32.to_le_bytes());
    for bought in 1..=2 {
        let outcome = gateway
            .handle_at(&wrap(&buy), NOW + 1 + u64::from(bought))
            .expect("the ring buy handles");
        assert!(outcome.gameplay_changed, "a successful buy requests a save");
        let done = inner_packets(&outcome);
        let completion = coin_completion(&done);
        assert_eq!(word(completion, 4), 1, "the 165/6 result is success");
        assert_eq!(word(completion, 12), 1, "kind 1 = buy");
        assert_eq!(word(completion, 44), 1, "quantity");
        assert_eq!(word(completion, 48), RING_PRICE, "the server unit price");
        let after = inventory_of(&gateway);
        assert_eq!(
            after.coins,
            1_000 - RING_PRICE * bought,
            "server price, not a client value"
        );
        assert_eq!(count_definition(&after, RING), bought);
    }

    // A refused row changes nothing and leaves the open shop usable.
    let before = gateway.snapshot();
    let mut refused = vec![165, 0, 4, 0];
    refused.extend(merchant.to_le_bytes());
    refused.extend(group.fields[0].to_le_bytes());
    refused.extend(999_999i32.to_le_bytes());
    refused.extend(0u32.to_le_bytes());
    refused.extend(1u32.to_le_bytes());
    let refused = gateway
        .handle_at(&wrap(&refused), NOW + 4)
        .expect("the session survives an unknown item");
    let refused = inner_packets(&refused);
    assert!(
        !refused
            .iter()
            .any(|packet| packet.starts_with(&[165, 0, 6, 0])),
        "a non-stock item must not produce a purchase completion"
    );
    assert_eq!(gateway.snapshot(), before, "a refused buy changed state");
}
