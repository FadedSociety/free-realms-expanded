//! The real-content gateway purchase/refusal proof for the representative
//! selected-hub vendor. Kept out of `vendor_stock_coverage.rs` so that file
//! stays a content-coverage suite; the protected test name in that file is a
//! one-line wrapper around this function.
//!
//! The fixture is built from the shipped vendor record and the captured
//! catalogue. Only the NPC's zone/position is translated to zone 0 because
//! `Gateway::load_gameplay` without the opening campaign supports the open-world
//! zone. The stock row, prices, item definition and transaction path are the
//! shipped ones. The directory is removed on drop.

use crate::gateway_vendor::{
    NOW, count_definition, inner_packets, interact, inventory_of, login, merchant_status, select,
    sell_packet, send, vendor_buy_packet, wrap,
};
use crate::{REPRESENTATIVE_ITEM, REPRESENTATIVE_VENDOR, content_dir, prices, read_json};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    inventory::InventoryItem,
    npc_interaction::{NpcAction, NpcInteraction, decode_merchant_list_66_42},
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct GatewayFixture {
    content: PathBuf,
    reference: PathBuf,
    zone: u32,
    position: [f32; 3],
    option: u32,
    price: u32,
}

impl Drop for GatewayFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl GatewayFixture {
    fn new() -> Self {
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let reference = lab.join("references/rust-gateway-reference");
        let source = lab.join("reports/round70-native-20261016T061000Z/content");
        let content = lab.join("build/w178-vendor-gateway").join(format!(
            "f178-vendor-gateway-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(content.join("npcs")).expect("fixture content creates");
        for file in [
            "inventory.json",
            "quests.json",
            "quest-journal.bin",
            "combat.json",
            "combat-template.bin",
            "categories.json",
        ] {
            fs::copy(source.join(file), content.join(file))
                .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
        }
        let catalogue = content.join("item-catalogue.json");
        if fs::hard_link(source.join("item-catalogue.json"), &catalogue).is_err() {
            fs::copy(source.join("item-catalogue.json"), &catalogue)
                .expect("item catalogue copies");
        }
        for file in ["stores.json", "zones.json"] {
            fs::copy(content_dir().join(file), content.join(file))
                .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
        }

        let vendors = read_json(&content_dir().join("vendors.json"));
        let vendor = vendors["vendors"]
            .as_array()
            .expect("vendor records are an array")
            .iter()
            .find(|vendor| vendor["npc_id"].as_u64() == Some(REPRESENTATIVE_VENDOR))
            .expect("the representative vendor exists")
            .clone();
        fs::write(
            content.join("vendors.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "vendors": [vendor]}))
                .expect("fixture vendors serialize"),
        )
        .expect("fixture vendors write");

        let npcs = read_json(&content_dir().join("npcs/blackspore_swamp.json"));
        let mut npc = npcs["records"]
            .as_array()
            .expect("NPC records are an array")
            .iter()
            .find(|npc| npc["npc_id"].as_u64() == Some(REPRESENTATIVE_VENDOR))
            .expect("the representative NPC exists")
            .clone();
        npc["zone_id"] = json!(0);
        npc["zone"] = json!("FabledRealms");
        npc["position"] = json!([0.0, 0.0, 0.0]);
        npc["heading"] = json!(0.0);
        npc["placement"] = json!("legacy_data");
        fs::write(
            content.join("npcs/melninster.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "records": [npc]}))
                .expect("fixture NPC serializes"),
        )
        .expect("fixture NPC writes");
        fs::write(
            content.join("npc_spawns_extra.json"),
            br#"{"schema_version":1,"records":[]}"#,
        )
        .expect("fixture spawn overlay writes");

        let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .expect("fixture NPC content loads");
        let (zone, position) = service
            .location(REPRESENTATIVE_VENDOR)
            .expect("the representative vendor has a location");
        let menu = service
            .interact(REPRESENTATIVE_VENDOR, zone, position, None)
            .expect("the representative vendor opens");
        let option = u32::try_from(
            menu.options
                .iter()
                .find(|option| option.action == NpcAction::Shop)
                .expect("the representative vendor exposes a shop option")
                .id,
        )
        .expect("the shop option id fits the client field");
        let price = prices(&content_dir())
            .get(&(REPRESENTATIVE_ITEM as i64))
            .copied()
            .expect("the representative item has a positive price") as u32;
        Self {
            content,
            reference,
            zone,
            position,
            option,
            price,
        }
    }

    fn state(&self, coins: u32, full_bag: bool) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("fixture gameplay content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = coins;
        let item = |id: i32| InventoryItem {
            definition_id: 75_060,
            tint: 0,
            id,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        };
        if full_bag {
            for id in 100_000..100_100 {
                state.inventory.items.push(item(id));
            }
            state.inventory.next_instance_id = Some(100_100);
        } else {
            state.inventory.items.push(item(1));
            state.inventory.next_instance_id = Some(2);
        }
        state
    }

    fn character(&self) -> CharacterState {
        let mut character =
            Gateway::reference_state(&self.reference).expect("reference character loads");
        character.zone_id = self.zone;
        character.position = self.position;
        character.rotation = [0.0; 3];
        character
    }

    fn connect(&self, state: GameplayState) -> Gateway {
        self.connect_with(self.character(), state)
    }

    /// Reconnect through the saved character half of the save envelope, not a
    /// fresh reference character.
    fn connect_with(&self, character: CharacterState, state: GameplayState) -> Gateway {
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, Some(state))
                .expect("gateway loads the vendor content");
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway
                .handle_at(&packet, NOW)
                .expect("initialization message handles");
        }
        gateway
    }

    fn open_shop(&self, gateway: &mut Gateway) -> Vec<Vec<u8>> {
        send(gateway, &interact(REPRESENTATIVE_VENDOR));
        let replies = send(gateway, &select(REPRESENTATIVE_VENDOR, self.option));
        assert!(
            replies
                .iter()
                .any(|packet| packet.starts_with(&[66, 0, 42, 0])),
            "the gateway did not open the representative shop"
        );
        replies
    }
}

pub fn representative_new_stock_buys_once_and_refuses_atomically() {
    let fixture = GatewayFixture::new();
    let price = fixture.price;
    let mut gateway = fixture.connect(fixture.state(price + 1_000, false));
    let opened = fixture.open_shop(&mut gateway);
    let list = opened
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 42, 0]))
        .expect("shop list is present");
    // 66/42 is decoded exactly as the client reads it: dispatcher `00b48a10`
    // -> `00b40a60` -> `00b3d5f0` (header + nested record). Group reader
    // `00b33020` reads one scalar, two dword pairs and a final scalar (six
    // dwords), then `00b2e620` reads the entry count and rows via `00b29cd0`.
    let list = decode_merchant_list_66_42(list).expect("shop list matches the client reader");
    let entry = list
        .group
        .entries
        .iter()
        .find(|entry| entry.first_six_dwords[0] == REPRESENTATIVE_ITEM as u32)
        .expect("the gateway shop does not expose the new item");
    // (R) Local convention for the opaque row: dword 1 is the purchase price.
    assert_eq!(entry.first_six_dwords[1], price);

    let before = gateway.snapshot();
    let bought = gateway
        .handle_at(
            &wrap(&vendor_buy_packet(108_001, REPRESENTATIVE_ITEM, 1)),
            NOW,
        )
        .expect("vendor buy handles");
    assert!(bought.gameplay_changed, "a successful buy requests a save");
    let replies = inner_packets(&bought);
    assert_eq!(
        merchant_status(replies.last().expect("vendor buy reply")),
        1
    );
    let after = gateway.snapshot();
    let before_inventory = before
        .gameplay
        .as_ref()
        .expect("pre-buy gameplay exists")
        .inventory
        .clone();
    let after_inventory = after
        .gameplay
        .as_ref()
        .expect("post-buy gameplay exists")
        .inventory
        .clone();
    assert_eq!(after_inventory.coins, before_inventory.coins - price);
    assert_eq!(
        count_definition(&after_inventory, REPRESENTATIVE_ITEM),
        count_definition(&before_inventory, REPRESENTATIVE_ITEM) + 1
    );
    assert_eq!(
        count_definition(&after_inventory, 75_060),
        1,
        "the pre-login item survived the purchase"
    );
    assert_eq!(
        after.character, before.character,
        "a vendor purchase moved the character"
    );

    // Reconnect through both saved halves, not a fresh reference character.
    let saved_character: CharacterState = serde_json::from_slice(
        &serde_json::to_vec(&after.character).expect("character state serializes"),
    )
    .expect("character state parses");
    let saved: GameplayState = serde_json::from_slice(
        &serde_json::to_vec(
            &after
                .gameplay
                .clone()
                .expect("successful gameplay state serializes"),
        )
        .expect("gameplay state serializes"),
    )
    .expect("gameplay state parses");
    let mut reconnected = fixture.connect_with(saved_character.clone(), saved);
    let reconnect_snapshot = reconnected.snapshot();
    assert_eq!(
        reconnect_snapshot.character, saved_character,
        "the reconnect ignored the saved character half"
    );
    assert_eq!(inventory_of(&reconnected), after_inventory);

    // A reconnected player has no open merchant interaction: the same buy is
    // refused (66/44 status 0) and must not mutate the restored state.
    let replies = send(
        &mut reconnected,
        &vendor_buy_packet(108_005, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(
        merchant_status(replies.last().expect("closed-shop reply")),
        0,
        "a reconnected player has no open shop"
    );
    assert_eq!(
        reconnected.snapshot(),
        reconnect_snapshot,
        "the closed-shop refusal changed the reconnected state"
    );

    // Poor coins: the refusal changes nothing, and the shop survives it. Sell
    // the seeded item to top up, then the same buy lands.
    let mut poor = fixture.connect(fixture.state(price - 1, false));
    fixture.open_shop(&mut poor);
    let poor_before = poor.snapshot();
    let replies = send(
        &mut poor,
        &vendor_buy_packet(108_006, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(merchant_status(replies.last().expect("poor reply")), 0);
    assert_eq!(
        poor.snapshot(),
        poor_before,
        "insufficient-coins refusal changed the gateway state"
    );
    send(&mut poor, &sell_packet(0, 1, 1));
    let topped_up = inventory_of(&poor).coins;
    assert_eq!(
        topped_up,
        price - 1 + 375,
        "the poor-coins refusal consumed the open shop"
    );
    let replies = send(
        &mut poor,
        &vendor_buy_packet(108_020, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(
        merchant_status(replies.last().expect("poor retry reply")),
        1
    );
    assert_eq!(inventory_of(&poor).coins, topped_up - price);
    assert_eq!(
        count_definition(&inventory_of(&poor), REPRESENTATIVE_ITEM),
        1
    );

    // Full bag: the refusal changes nothing, and freeing one slot through the
    // real sell route lets the same buy land.
    let mut full = fixture.connect(fixture.state(price + 1_000, true));
    fixture.open_shop(&mut full);
    assert_eq!(
        inventory_of(&full)
            .items
            .iter()
            .filter(|item| item.count > 0)
            .count(),
        100
    );
    let full_before = full.snapshot();
    let replies = send(
        &mut full,
        &vendor_buy_packet(108_007, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(merchant_status(replies.last().expect("full-bag reply")), 0);
    assert_eq!(
        full.snapshot(),
        full_before,
        "full-bag refusal changed the gateway state"
    );
    send(&mut full, &sell_packet(0, 100_000, 1));
    let freed = inventory_of(&full);
    assert_eq!(
        freed.coins,
        price + 1_000 + 375,
        "the full-bag refusal consumed the open shop"
    );
    assert_eq!(freed.items.len(), 99);
    let replies = send(
        &mut full,
        &vendor_buy_packet(108_021, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(
        merchant_status(replies.last().expect("full-bag retry reply")),
        1
    );
    let purchased = inventory_of(&full);
    assert_eq!(purchased.coins, freed.coins - price);
    assert_eq!(purchased.items.len(), 100);
    assert_eq!(count_definition(&purchased, REPRESENTATIVE_ITEM), 1);
}
