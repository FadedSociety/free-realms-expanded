//! The real-content fixture for the representative selected-hub vendor, shared by
//! `tests/common/vendor_purchase.rs` and the gateway binary's L011 save-failure test.
//!
//! The pack is built from the shipped vendor record and the captured catalogue. Only the
//! NPC's zone/position is translated to zone 0 because `Gateway::load_gameplay` without the
//! opening campaign supports the open-world zone. The stock row, prices, item definition and
//! transaction path are the shipped ones. The output directory is the one parameter: the
//! integration suite builds under `lab/build`, the binary test under its scratch directory.
//! This module has no `crate::` items so `#[path]`-inclusion works in both hosts. The
//! directory is removed on drop.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    inventory::InventoryItem,
    npc_interaction::{NpcAction, NpcInteraction},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "round70_content.rs"]
mod round70_content;

/// The shipped representative selected-hub vendor and its added stock row.
pub const REPRESENTATIVE_VENDOR: u64 = 9_500_000_045;
pub const REPRESENTATIVE_ITEM: i32 = 76_427;
/// The fixture's seeded pre-login item, which the buy-back probe sells and restores.
pub const SEEDED_ITEM: i32 = 75_060;

pub struct VendorFixture {
    content: PathBuf,
    reference: PathBuf,
    zone: u32,
    position: [f32; 3],
    option: u32,
    price: u32,
    sell_price: u32,
}

impl Drop for VendorFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl VendorFixture {
    /// Build one single-vendor pack under `root`; `label` names it for debugging.
    pub fn new(root: &Path, label: &str) -> Self {
        let reference = round70_content::reference();
        let content = round70_content::build(root, label, &[]);
        let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        for file in ["stores.json", "zones.json"] {
            fs::copy(data.join(file), content.join(file))
                .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
        }

        let vendors = read_json(&data.join("vendors.json"));
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

        let npcs = read_json(&data.join("npcs/blackspore_swamp.json"));
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
        let price = store_price(
            &data.join("stores.json"),
            REPRESENTATIVE_ITEM,
            "purchase_price_coins",
        );
        let sell_price = store_price(&data.join("stores.json"), SEEDED_ITEM, "sell_price_coins");
        Self {
            content,
            reference,
            zone,
            position,
            option,
            price,
            sell_price,
        }
    }

    pub fn content(&self) -> &Path {
        &self.content
    }

    pub fn reference(&self) -> &Path {
        &self.reference
    }

    /// The shop menu option the fixture vendor exposes (client 26/10 choice).
    pub fn option(&self) -> u32 {
        self.option
    }

    /// The authoritative purchase price of `REPRESENTATIVE_ITEM` from `stores.json`.
    pub fn price(&self) -> u32 {
        self.price
    }

    /// The authoritative resale price of `SEEDED_ITEM` from `stores.json`.
    pub fn sell_price(&self) -> u32 {
        self.sell_price
    }

    pub fn character(&self) -> CharacterState {
        let mut character =
            Gateway::reference_state(&self.reference).expect("reference character loads");
        character.zone_id = self.zone;
        character.position = self.position;
        character.rotation = [0.0; 3];
        character
    }

    pub fn state(&self, coins: u32, full_bag: bool) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("fixture gameplay content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = coins;
        let item = |id: i32| InventoryItem {
            definition_id: SEEDED_ITEM,
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
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("content file reads"))
        .unwrap_or_else(|error| panic!("{} parses: {error}", path.display()))
}

/// The authoritative Coin price for `item` from `stores.json` (`purchase_price_coins`
/// or `sell_price_coins`).
fn store_price(stores: &Path, item: i32, field: &str) -> u32 {
    let document = read_json(stores);
    let price = document["items"]
        .as_array()
        .expect("store items are an array")
        .iter()
        .find(|row| row["item_id"].as_i64() == Some(i64::from(item)))
        .and_then(|row| row[field].as_u64())
        .unwrap_or_else(|| panic!("item {item} has {field}"));
    u32::try_from(price).expect("the store price fits a Coin balance")
}
