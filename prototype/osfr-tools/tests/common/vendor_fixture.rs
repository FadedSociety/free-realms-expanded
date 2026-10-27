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
    quest_engine::{CharacterEligibility, QuestEngine, QuestState},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "item_catalogue.rs"]
mod item_catalogue;
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
    /// The repo `data/content/stores.json` copy this fixture's pack was built from.
    stores: PathBuf,
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
        Self::for_vendor(root, label, REPRESENTATIVE_VENDOR, "blackspore_swamp.json")
    }

    /// T529: the same pack for another shipped vendor record and the NPC file that holds it.
    /// Only the NPC's zone/position is translated; the stock rows are the shipped ones until
    /// `stock_price_coins` replaces them.
    pub fn for_vendor(root: &Path, label: &str, npc_id: u64, npc_file: &str) -> Self {
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
            .find(|record| record["npc_id"].as_u64() == Some(npc_id))
            .expect("the fixture vendor exists")
            .clone();
        fs::write(
            content.join("vendors.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "vendors": [vendor]}))
                .expect("fixture vendors serialize"),
        )
        .expect("fixture vendors write");

        let npcs = read_json(&data.join("npcs").join(npc_file));
        let mut npc = npcs["records"]
            .as_array()
            .expect("NPC records are an array")
            .iter()
            .find(|record| record["npc_id"].as_u64() == Some(npc_id))
            .expect("the fixture NPC exists")
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
            .location(npc_id)
            .expect("the fixture vendor has a location");
        let menu = service
            .interact(npc_id, zone, position, None)
            .expect("the fixture vendor opens");
        let option = u32::try_from(
            menu.options
                .iter()
                .find(|option| option.action == NpcAction::Shop)
                .expect("the fixture vendor exposes a shop option")
                .id,
        )
        .expect("the shop option id fits the client field");
        let stores = data.join("stores.json");
        let price = store_price(&stores, REPRESENTATIVE_ITEM, "purchase_price_coins");
        let sell_price = store_price(&stores, SEEDED_ITEM, "sell_price_coins");
        Self {
            content,
            reference,
            stores,
            zone,
            position,
            option,
            price,
            sell_price,
        }
    }

    /// T529: replace this fixture vendor's stock with `rows` (`item_id`, coin price) and give
    /// every row a coin-only `stores.json` entry in the fixture copy. The shipped content files
    /// are never written.
    pub fn stock_price_coins(&self, rows: &[(i32, u32)]) {
        let vendors = read_json(&self.content.join("vendors.json"));
        let mut vendor = vendors["vendors"][0].clone();
        let ids: Vec<i32> = rows.iter().map(|(item, _)| *item).collect();
        vendor["item_ids"] = json!(ids);
        fs::write(
            self.content.join("vendors.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "vendors": [vendor]}))
                .expect("fixture vendors serialize"),
        )
        .expect("fixture vendors write");

        let mut stores = read_json(&self.content.join("stores.json"));
        let items = stores["items"]
            .as_array_mut()
            .expect("store items are an array");
        for (item, price) in rows {
            let row = json!({
                "item_id": item,
                "purchase_price_coins": price,
                "member_discount_percent": null,
                "member_price_coins": null,
                "sell_price_coins": null,
                "no_sale": true
            });
            match items
                .iter_mut()
                .find(|row| row["item_id"].as_i64() == Some(i64::from(*item)))
            {
                Some(existing) => *existing = row,
                None => items.push(row),
            }
        }
        fs::write(
            self.content.join("stores.json"),
            serde_json::to_vec(&stores).expect("fixture stores serialize"),
        )
        .expect("fixture stores write");
    }

    /// Add a definition row (`item_catalogue::row`, the shared row schema) with the raw
    /// `serialized` bytes to this fixture's own `inventory.json`, so it loads like a shipped row. The shipped file is
    /// never written.
    pub fn define_item(&self, id: i32, serialized: &[u8]) {
        let path = self.content.join("inventory.json");
        let mut content = read_json(&path);
        content["definitions"]
            .as_array_mut()
            .expect("definitions are an array")
            .push(item_catalogue::row(id, serialized.to_vec()));
        fs::write(
            path,
            serde_json::to_vec(&content).expect("fixture inventory serializes"),
        )
        .expect("fixture inventory writes");
    }

    /// T794: add one accepted "collect `quantity` of `item`" quest, given by `giver`, to this
    /// fixture's own content and return the quest save that holds it active (put it in
    /// `GameplayState::npc_quests`). The gateway then binds the fixture's split catalogue instead
    /// of the shipped one. The shipped content files are never written.
    pub fn install_collect_quest(
        &self,
        quest: u32,
        giver: u64,
        item: i32,
        quantity: u32,
    ) -> QuestState {
        let source =
            json!([{"kind": "reconstructed", "evidence": "(R) T794 synthetic collect objective"}]);
        let write = |path: &str, document: serde_json::Value| {
            fs::write(
                self.content.join(path),
                serde_json::to_vec(&document).expect("fixture quest content serializes"),
            )
            .expect("fixture quest content writes");
        };
        fs::create_dir_all(self.content.join("quests")).expect("fixture quests dir creates");
        write(
            "quests/t794_collect.json",
            json!({"schema_version": 1, "records": [{
                "id": quest,
                "title_string_id": 1,
                "giver_npc_id": giver,
                "prerequisite_quest_ids": [],
                "objectives": [{
                    "kind": "collect_item", "id": 1, "item_id": item,
                    "quantity": quantity, "source": source.clone()
                }],
                "rewards": [],
                "source": source.clone()
            }]}),
        );
        write(
            "jobs.json",
            json!({"schema_version": 1, "records": [{
                "id": "t794_synthetic", "name": "Synthetic", "profile_id": null,
                "category": "freestyle", "max_level": 20, "members_only": false,
                "trainer_npc_ids": [], "quest_ids": [quest], "abilities": [], "source": source
            }]}),
        );
        let mut engine = QuestEngine::from_split_content(
            &self.content,
            self.character().guid,
            CharacterEligibility {
                player_level: 1,
                member: true,
                member_quest_limit: 10,
                ..CharacterEligibility::default()
            },
            None,
        )
        .expect("fixture quest catalogue loads");
        assert!(engine.offer(quest, giver).changed);
        assert!(engine.accept(quest).changed);
        engine.state().clone()
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

    /// The authoritative resale price of any stocked item from `stores.json`.
    pub fn sell_price_of(&self, item: i32) -> u32 {
        store_price(&self.stores, item, "sell_price_coins")
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
