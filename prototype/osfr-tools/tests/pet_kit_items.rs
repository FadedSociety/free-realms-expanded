//! T797: the real client pet kit items. Every client item definition with `Type` 4 (the pet kit)
//! is recorded in `pets.json` with its client `Slot`, `Param2` and locale name, stocked at the two
//! pet merchants, and accepted by the care path in its slot: a real bowl fits, and a feed with a
//! real bowl and food spends one food, raises the food need and plays the client `pet_feed`
//! animation on the pet actor.
//!
//! The client facts come from the pinned data the catalogue is built from
//! (`lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`), the runtime item
//! definitions (`lab/reports/round70-.../content/item-catalogue.json`, read through `Inventory`) and
//! the client locale (`lab/reports/locale-id-to-text.json`). The slot is the client care-panel tab
//! (`BaseClient.PetInventory` `ShampooData`, `ConditionerData`, `BrushData`, `FoodBowlData`,
//! `FoodData`, `VitaminData`); `00c077a0` maps it to the utility slot. A public checkout has no
//! private lab: the client-side gate reports a skip there and the shipped-content checks still run.

#[path = "support/pet_care.rs"]
mod care;
#[path = "support/json.rs"]
mod json;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;
#[path = "common/round70_content.rs"]
mod round70_content;

use care::{
    OWNER, T0, content_dir, fit, lab, locale, notify_kinds, pets_json, send, statuses, words,
};
use osfr_tools::{
    inventory::Inventory,
    pets::{PetEquipmentSlot, PetService},
};
use pet_inventory::InventoryFixture;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

/// The six client care-panel slots: shampoo, conditioner, brush, food bowl, food and vitamins.
const KIT_SLOTS: [i64; 6] = [200, 201, 203, 204, 205, 206];
/// The two pet merchants: Gloria (Cobblestone Village) and Florina (Sanctuary, `Pet supplies`).
const PET_MERCHANTS: [u64; 2] = [9_500_000_246, 9_500_000_482];
/// A real bowl and the food the care path has always used: item 14 "Red Daisies Bowl" (slot 204)
/// and 1835 "Simple Pet Food" (slot 205, Param2 120).
const BOWL_DEF: i32 = 14;
const FOOD_DEF: i32 = 1835;
const SHAMPOO_DEF: i32 = 1836;
/// A client Type-4 food that is the fishing suppliers' stock, not a pet-shop item.
const BAIT_DEF: i64 = 57012;
const BOWL_INSTANCE: i32 = 101;
const FOOD_INSTANCE: i32 = 102;
const SHAMPOO_INSTANCE: i32 = 103;
/// The client `Resources/AnimationTypes.xml` slot `pet_feed`.
const PET_FEED_ANIMATION: i32 = 5_507;

fn kit_rows() -> Vec<Value> {
    pets_json()["care_items"]
        .as_array()
        .expect("care_items is an array")
        .clone()
}

/// The pinned client item definitions file: absent in a public checkout.
fn client_items_path() -> PathBuf {
    lab("references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json")
}

/// Whether every private table the client-side gate reads is present.
fn client_tables() -> bool {
    client_items_path().is_file()
        && lab("reports/locale-id-to-text.json").is_file()
        && round70_content::source().is_dir()
}

fn client_kit_items() -> Vec<Value> {
    json::read(&client_items_path())
        .as_array()
        .expect("client item definitions are an array")
        .iter()
        .filter(|row| row["Type"].as_i64() == Some(4))
        .cloned()
        .collect()
}

fn catalogue() -> Inventory {
    Inventory::load(&round70_content::source(), None).expect("the runtime catalogue loads")
}

fn int(row: &Value, key: &str) -> i64 {
    row[key]
        .as_i64()
        .unwrap_or_else(|| panic!("{key} in {row}"))
}

fn is_test(row: &Value) -> bool {
    row["label"]
        .as_str()
        .is_some_and(|label| label.contains("(TEST)"))
}

/// A bag with the real bowl, food and shampoo definitions (their installed slots and classes).
fn real_kit() -> InventoryFixture {
    InventoryFixture::with_kit_items(
        &[
            (BOWL_INSTANCE, BOWL_DEF, 1),
            (FOOD_INSTANCE, FOOD_DEF, 2),
            (SHAMPOO_INSTANCE, SHAMPOO_DEF, 2),
        ],
        &[
            (BOWL_DEF, 204, 73),
            (FOOD_DEF, 205, 73),
            (SHAMPOO_DEF, 200, 73),
        ],
    )
}

/// The guid/animation/flags of every 35/8 animation packet: `u16 35, u16 8, u64 guid,
/// i32 animation, i32 unknown, u8 flags` (native reader `00908a50`).
fn animations(packets: &[Vec<u8>]) -> Vec<(u64, i32, u8)> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 8, 0]))
        .map(|packet| {
            assert_eq!(packet.len(), 21, "one animation frame");
            (
                u64::from_le_bytes(packet[4..12].try_into().unwrap()),
                i32::from_le_bytes(packet[12..16].try_into().unwrap()),
                packet[20],
            )
        })
        .collect()
}

#[test]
fn every_client_pet_kit_item_is_recorded_with_its_slot_name_and_param() {
    if !client_tables() {
        eprintln!("skipped: the private client kit tables are absent");
        return;
    }
    let catalogue = catalogue();
    let locale = locale();
    let rows = kit_rows();
    let mut by_id = BTreeMap::new();
    for row in &rows {
        assert!(
            by_id
                .insert(int(row, "definition_id"), row.clone())
                .is_none(),
            "duplicate kit row {row}"
        );
    }
    let client = client_kit_items();
    assert_eq!(client.len(), 170, "the client Type-4 count changed");
    assert_eq!(
        rows.len(),
        client.len(),
        "every client kit item is recorded"
    );
    let mut classes = BTreeSet::new();
    for item in &client {
        let id = int(item, "Id");
        let row = by_id
            .get(&id)
            .unwrap_or_else(|| panic!("client kit item {id} has no pets.json row"));
        let slot = int(item, "Slot");
        let name_id = int(item, "NameId");
        assert!(KIT_SLOTS.contains(&slot), "item {id} slot {slot}");
        assert_eq!(int(row, "slot"), slot, "item {id} slot");
        assert_eq!(int(row, "name_id"), name_id, "item {id} name id");
        assert_eq!(
            row["label"].as_str(),
            Some(locale[&name_id].as_str()),
            "item {id} name is its own client locale text"
        );
        let param2 = item["Param2"].as_i64().unwrap_or(0);
        assert_eq!(
            row["restore"].as_f64(),
            Some(param2 as f64),
            "item {id} Param2"
        );
        if slot == 205 {
            assert!(param2 > 0, "food {id} has a restore");
        }
        // The runtime definition the client receives: type 4, the recorded slot and a class a pet
        // model lists (so the fit gate can accept it).
        let (item_type, installed_slot, class) = catalogue
            .kit_facts(id as i32)
            .unwrap_or_else(|| panic!("item {id} has no runtime definition"));
        assert_eq!(item_type, 4, "item {id} type");
        assert_eq!(i64::from(installed_slot), slot, "item {id} runtime slot");
        assert_eq!(i64::from(class), int(item, "Class"), "item {id} class");
        classes.insert(i64::from(class));
        assert_eq!(
            catalogue.presentation(id as i32).unwrap().name_id,
            name_id as i32,
            "item {id} runtime name id"
        );
    }
    let models = pets_json();
    let model_classes: BTreeSet<i64> = models["models"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|model| model["item_classes"].as_array().into_iter().flatten())
        .map(|class| int(class, "id"))
        .collect();
    for class in classes {
        assert!(
            model_classes.contains(&class),
            "kit class {class} is on no pet model"
        );
    }
}

#[test]
fn both_pet_merchants_stock_every_non_test_kit_item_at_its_client_cost() {
    let rows = kit_rows();
    let by_id: BTreeMap<i64, &Value> = rows
        .iter()
        .map(|row| (int(row, "definition_id"), row))
        .collect();
    let stores = json::read(&content_dir().join("stores.json"));
    let prices: BTreeMap<i64, &Value> = stores["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (int(row, "item_id"), row))
        .collect();
    let vendors = json::read(&content_dir().join("vendors.json"));
    // Every stocked kit item carries the client Cost, or the 5-coin (R) floor where that Cost is
    // 0. A public checkout has no client table, so this half of the gate reports a skip and the
    // shipped-content checks below still run.
    let client_costs: Option<BTreeMap<i64, i64>> = client_items_path().is_file().then(|| {
        client_kit_items()
            .iter()
            .map(|row| (int(row, "Id"), row["Cost"].as_i64().unwrap_or(0)))
            .collect()
    });
    let expected: BTreeSet<i64> = rows
        .iter()
        .filter(|row| !is_test(row) && int(row, "definition_id") != BAIT_DEF)
        .map(|row| int(row, "definition_id"))
        .collect();
    assert_eq!(expected.len(), 163, "the stocked kit count changed");
    // Bait is a client kit row (Type 4, slot 205) but it is the fishing vendors' unpriced stock;
    // pricing it would make their vendor rows sellable, so the pet merchants do not stock it.
    assert!(
        rows.iter()
            .any(|row| int(row, "definition_id") == BAIT_DEF && int(row, "slot") == 205),
        "Bait stays recorded as a kit row"
    );
    for npc_id in PET_MERCHANTS {
        let vendor = vendors["vendors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|vendor| vendor["npc_id"].as_u64() == Some(npc_id))
            .unwrap_or_else(|| panic!("pet merchant {npc_id}"));
        let item_ids: Vec<i64> = vendor["item_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_i64().unwrap())
            .collect();
        let items: Vec<&Value> = vendor["items"].as_array().unwrap().iter().collect();
        assert_eq!(
            item_ids,
            items
                .iter()
                .map(|item| int(item, "item_id"))
                .collect::<Vec<_>>(),
            "pet merchant {npc_id} advertises exactly its item rows"
        );
        let mut stocked_kit = BTreeSet::new();
        let mut stocked_test = BTreeSet::new();
        for item in &items {
            let id = int(item, "item_id");
            let Some(row) = by_id.get(&id) else {
                continue; // Gloria also sells pet pals; only the recorded kit rows are checked here.
            };
            let slot = int(row, "slot");
            assert!(
                KIT_SLOTS.contains(&slot),
                "stocked item {id} slot {slot} is not a care-panel slot"
            );
            let name_id = int(row, "name_id");
            let label = row["label"].as_str().unwrap_or_default();
            assert_eq!(
                item["item_name_id"].as_i64(),
                Some(name_id),
                "stocked item {id} vendor name id"
            );
            assert_eq!(
                item["item_source"]["english_text"].as_str(),
                Some(label),
                "stocked item {id} english text"
            );
            let price = prices
                .get(&id)
                .unwrap_or_else(|| panic!("stocked item {id} has no store price"));
            assert!(
                price["purchase_price_coins"]
                    .as_u64()
                    .is_some_and(|p| p > 0),
                "stocked item {id} has no positive price"
            );
            if let Some(costs) = &client_costs {
                let cost = costs
                    .get(&id)
                    .unwrap_or_else(|| panic!("stocked item {id} is not a client Type-4 row"));
                let expected = if *cost > 0 { *cost } else { 5 };
                assert_eq!(
                    price["purchase_price_coins"].as_i64(),
                    Some(expected),
                    "stocked item {id} is not at its client Cost ({cost})"
                );
            }
            for field in [
                "purchase_price_coins",
                "member_price_coins",
                "sell_price_coins",
            ] {
                assert_eq!(item[field], price[field], "item {id} {field}");
            }
            assert_eq!(item["runtime_eligible"].as_bool(), Some(true), "item {id}");
            if is_test(row) {
                stocked_test.insert(id);
            } else {
                stocked_kit.insert(id);
            }
        }
        assert_eq!(
            stocked_kit, expected,
            "pet merchant {npc_id} stocks every non-test client kit item"
        );
        assert!(
            stocked_test.is_empty(),
            "pet merchant {npc_id} stocks test items {stocked_test:?}"
        );
    }
}

#[test]
fn a_real_bowl_and_real_food_fit_feed_and_play_the_pet_feed_animation() {
    let (mut service, pet) = care::dog();
    let mut fixture = real_kit();
    // The client's fit request (53/16) for the real bowl and food.
    for (slot, instance) in [(5, BOWL_INSTANCE), (6, FOOD_INSTANCE)] {
        let wire = send(&mut service, &mut fixture, &fit(pet, slot, instance), T0);
        assert!(wire.changed, "fitting {instance} into slot {slot}");
    }
    send(&mut service, &mut fixture, &pet_wire::summon(pet), T0);
    let before = service.pet(pet).unwrap().needs.food;

    let started = send(&mut service, &mut fixture, &pet_wire::feed(pet), T0 + 1_000);
    assert_eq!(notify_kinds(&started.private), vec![1]);
    assert!(
        animations(&started.private).is_empty(),
        "the feed has begun only"
    );

    let early = service.care_tick(&mut fixture.inventory, T0 + 1_500);
    assert!(animations(&early.private).is_empty());
    assert_eq!(care::count(&fixture, FOOD_INSTANCE), 2);

    let applied = service.care_tick(&mut fixture.inventory, T0 + 1_900);
    assert!(applied.changed);
    let expected = vec![(PetService::pet_guid(OWNER), PET_FEED_ANIMATION, 0)];
    assert_eq!(
        animations(&applied.private),
        expected,
        "the owner sees the eat"
    );
    assert_eq!(animations(&applied.shared), expected, "viewers see the eat");
    assert_eq!(care::count(&fixture, FOOD_INSTANCE), 1, "one food is eaten");
    assert_eq!(care::count(&fixture, BOWL_INSTANCE), 1, "the bowl is kept");
    let status = statuses(&applied.private);
    assert_eq!(status.len(), 1);
    let food = f32::from_le_bytes(status[0][7..11].try_into().unwrap());
    assert!(food > before && food <= 100.0, "{before} -> {food}");

    let ended = service.care_tick(&mut fixture.inventory, T0 + 7_000);
    assert_eq!(notify_kinds(&ended.private), vec![2]);
    assert!(animations(&ended.private).is_empty(), "the eat plays once");
}

#[test]
fn a_non_bowl_in_the_bowl_slot_is_refused_and_changes_nothing() {
    let (mut service, pet) = care::dog();
    let mut fixture = real_kit();
    // An old save (or a non-client path) may hold a shampoo instance in the bowl's slot.
    service
        .equip_item(
            pet,
            SHAMPOO_INSTANCE,
            PetEquipmentSlot::Utility(5),
            &fixture.inventory,
        )
        .unwrap();
    let fitted = send(&mut service, &mut fixture, &fit(pet, 6, FOOD_INSTANCE), T0);
    assert!(fitted.changed);
    send(&mut service, &mut fixture, &pet_wire::summon(pet), T0);
    let roster = service.document_value();
    let inventory = fixture.inventory.state().clone();

    let wire = send(&mut service, &mut fixture, &pet_wire::feed(pet), T0 + 1_000);
    assert_eq!(notify_kinds(&wire.private), vec![6], "an unlock refusal");
    assert_eq!(
        wire.private,
        vec![words(&[53, 0, 20], &[pet, 6, 0, 0, 0, 0])]
    );
    let tick = service.care_tick(&mut fixture.inventory, T0 + 2_000);
    assert!(!tick.changed);
    assert!(animations(&tick.private).is_empty());
    assert_eq!(service.document_value(), roster);
    assert_eq!(fixture.inventory.state(), &inventory);
    assert_eq!(care::count(&fixture, FOOD_INSTANCE), 2);
}
