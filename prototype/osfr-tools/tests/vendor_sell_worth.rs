//! T434: the coin store's Sell tab lists what 165/5 buys, at the price it pays.
//!
//! Client evidence: the Sell tab lists an owned item only when its definition has NoSale
//! (`+0x35`) clear and ResellValue (`+0xf0`) above 0 (`CanSellItem` `00cf08d0`) and shows
//! ResellValue as its Worth (`00a6e5f0` case 27; case 28 reads NoSale). `008e3760` reads the row
//! up to ClientEquipReqSetId and its caller `00919e10` reads the next dword into `+0xf0`, which is
//! `piece_096` `ItemDefinition.resell_value`. A 35/37 row for a cached id replaces it
//! (`00a3c290`). 165/5 carries the merchant, item instance and quantity only (`00c76dd0` ->
//! `00c75a70`), so the server alone decides the price. Every check below decodes the rows with
//! the 35/37 reader layout and sells through the real gateway path (26/8, 26/10, 165/5, 165/6).
//! The prices come from the shipped `stores.json` and item catalogue; the stores.json-versus-Worth
//! policy is (R).

#[allow(dead_code)]
#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[allow(dead_code)]
#[path = "../src/restoration/piece_096.rs"]
mod piece_096;
#[path = "common/resale.rs"]
mod resale;
#[allow(dead_code)]
#[path = "common/roosey_shop.rs"]
mod roosey_shop;
#[path = "common/round70_content.rs"]
mod round70_content;

use gateway_vendor::{
    NOW, completions, count_definition, inner_packets, inventory_of, sell_packet, send, wrap,
};
use osfr_tools::{
    coin_shop::{CoinShop, RESULT_SUCCESS, StoreCatalogue},
    gameplay::{Gameplay, GameplayState},
    inventory::{Inventory, InventoryItem},
    inventory_v2_fixes::encode_definition_batch_v2,
    npc_interaction::decode_coin_merchant_list_165_10,
    profiles::Profiles,
    station_cash::Wallet,
};
use piece_096::{InitMessage, ItemDefinition, TypedWriter, decode_payload};
use resale::{purchase_price, resale, rows};
use roosey_shop::{Fixture, data, read_json};
use std::collections::{BTreeMap, BTreeSet};

/// `stores.json` pays Cost / 10 (T633) for it; its definition shows a different Worth (3).
const RING: i32 = 75_732;
/// `stores.json` buys it for 750 and pays Cost / 10 for it; its generated row shows 0, so the
/// tab hides it.
const NECKLACE: i32 = 75_250;
/// A job starter Roosey stocks: it pays 0, its definition shows 7 and her price patch applies.
const MALLET: i32 = 75_030;
/// Copper Ore: no store row, definition Worth 1.
const ORE: i32 = 2_739;
/// A job starter with no store row and definition Worth 75.
const STARTER: i32 = 62_473;
/// Scaled Dragon, a ride: no store row, definition NoSale set.
const DRAGON: i32 = 7_961;
/// A ride whose store row is no_sale.
const STORE_RIDE: i32 = 76_856;

/// Add one stack of each `(definition, count)` with fresh instance ids.
fn give(state: &mut GameplayState, items: &[(i32, u32)]) {
    for (n, (definition_id, count)) in (900_001..).zip(items) {
        state.inventory.items.push(InventoryItem {
            definition_id: *definition_id,
            tint: 0,
            id: n,
            count: *count,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
    }
    state.inventory.next_instance_id = Some(901_000);
}

/// Every 35/37 row in `packets`, decoded with the reader layout.
fn decoded(packets: &[Vec<u8>]) -> BTreeMap<i32, ItemDefinition> {
    let mut rows = BTreeMap::new();
    for packet in packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 37, 0]))
    {
        let Ok(InitMessage::ItemDefinitions(batch)) =
            decode_payload(TypedWriter::ItemDefinitions.slot(), packet)
        else {
            panic!("the 35/37 batch must match the reader layout");
        };
        rows.extend(batch.items.into_iter().map(|row| (row.id, row)));
    }
    rows
}

/// The rows the client cached at login for `ids` (the catalogue's own records).
fn login_rows(inventory: &Inventory, ids: &BTreeSet<i32>) -> BTreeMap<i32, ItemDefinition> {
    let records: Vec<Vec<u8>> = ids
        .iter()
        .map(|id| {
            inventory
                .definition_record(*id)
                .expect("an owned item has a definition")
                .to_vec()
        })
        .collect();
    decoded(&[encode_definition_batch_v2(&records, &[]).expect("login rows encode")])
}

/// The client's Sell-tab Worth: `Some` only for a row `00cf08d0` lists.
fn listed(row: &ItemDefinition) -> Option<u32> {
    (!row.no_sale && row.resell_value > 0).then_some(row.resell_value as u32)
}

#[test]
fn shop_open_shows_each_owned_item_at_what_165_5_pays() {
    let fixture = Fixture::new(&["profiles.json"]);
    let inventory = Inventory::load(&fixture.content, None).expect("fixture inventory loads");
    let mut state = fixture.state();
    state.inventory.coins = 10_000;
    give(
        &mut state,
        &[(RING, 1), (MALLET, 1), (ORE, 3), (STARTER, 1), (DRAGON, 1)],
    );
    let mut gateway = fixture.connect(state);

    // Buy the generated necklace through 165/4 (`00c75ed0` field order), then open the shop
    // again: the Worth rows describe the bag at open.
    let opened = fixture.open_shop(&mut gateway);
    let list = opened
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
        .expect("the Shop choice sends the 165/10 coin merchant list");
    let group = decode_coin_merchant_list_165_10(list).expect("165/10 matches the client reader");
    let mut buy = vec![165, 0, 4, 0];
    buy.extend((u64::from(group.fields[3]) | (u64::from(group.fields[4]) << 32)).to_le_bytes());
    buy.extend(group.fields[0].to_le_bytes());
    buy.extend(NECKLACE.to_le_bytes());
    buy.extend(0u32.to_le_bytes());
    buy.extend(1u32.to_le_bytes());
    let bought = send(&mut gateway, &buy);
    assert!(
        bought
            .iter()
            .any(|packet| packet.starts_with(&[165, 0, 6, 0]) && packet[4..8] == [1, 0, 0, 0]),
        "the necklace buy succeeds"
    );
    assert_eq!(inventory_of(&gateway).coins, 10_000 - 750);

    let replies = fixture.open_shop(&mut gateway);
    let owned = inventory_of(&gateway);
    let ids = owned.items.iter().map(|item| item.definition_id).collect();
    let login = login_rows(&inventory, &ids);
    let pushed = decoded(&replies);
    let mut client = login.clone();
    client.extend(pushed.clone());

    // The shipped content still disagrees with the server price (sanity, not a pin).
    let (ring_paid, necklace_paid) = (resale(RING), resale(NECKLACE));
    assert!(listed(&login[&RING]).is_some_and(|worth| worth != ring_paid));
    assert_eq!(listed(&login[&NECKLACE]), None);
    assert_eq!(listed(&login[&MALLET]), Some(7));
    assert_eq!(listed(&login[&STARTER]), Some(75));

    // (a) and (e): the pushed rows show what 165/5 pays, and change nothing else.
    for (id, worth) in [
        (RING, ring_paid as i32),
        (NECKLACE, necklace_paid as i32),
        (MALLET, 0),
        (STARTER, 0),
    ] {
        assert_eq!(client[&id].resell_value, worth, "item {id} shows {worth}");
    }
    for id in [RING, NECKLACE, STARTER] {
        let mut expected = login[&id].clone();
        expected.resell_value = pushed[&id].resell_value;
        assert_eq!(pushed[&id], expected, "item {id} differs beyond its Worth");
    }
    // Roosey stocks the mallet: one row carries her stores.json price and the Worth.
    let mut expected = login[&MALLET].clone();
    expected.cost = 750;
    expected.resell_value = 0;
    assert_eq!(pushed[&MALLET], expected);
    assert!(
        !pushed.contains_key(&ORE) && !pushed.contains_key(&DRAGON),
        "rows that already show what 165/5 pays are not pushed"
    );

    // Shown == paid: a listed stack sells for exactly its Worth; an unlisted one pays nothing.
    for item in &owned.items {
        let worth = listed(&client[&item.definition_id]);
        let coins = inventory_of(&gateway).coins;
        let done = completions(&send(&mut gateway, &sell_packet(0, item.id, 1)));
        assert_eq!(done.len(), 1, "one 165/6 per sell");
        let paid = (done[0].result == RESULT_SUCCESS).then_some(done[0].price);
        match worth {
            Some(worth) => assert_eq!(paid, Some(worth), "item {} shown", item.definition_id),
            None => assert!(
                paid.is_none_or(|paid| paid == 0),
                "unlisted item {} paid {paid:?}",
                item.definition_id
            ),
        }
        assert_eq!(inventory_of(&gateway).coins, coins + paid.unwrap_or(0));
    }
    assert_eq!(
        inventory_of(&gateway).coins,
        10_000 - 750 + ring_paid + necklace_paid + 1,
        "ring, necklace and one ore paid; the starters paid 0"
    );
}

/// T633: a weapon, a clothing item and a consumable each pay a tenth of their buy price (the live
/// wiki tooltips: 85, 118 and 50 for 850, 1182 and 500), not the old Cost / 2. This canary and
/// `shipped_sell_prices_follow_the_tenth_rule_with_room_to_retune` are the only places the rule
/// is written down; every other suite reads the row's own `sell_price_coins`.
#[test]
fn weapon_clothing_and_consumable_sell_for_a_tenth_of_their_price() {
    const WAND: i32 = 75_154;
    const CLOAK: i32 = 43;
    const BANDAGE: i32 = 3_998;
    let fixture = Fixture::new(&["profiles.json"]);
    let mut state = fixture.state();
    give(&mut state, &[(WAND, 1), (CLOAK, 1), (BANDAGE, 1)]);
    let mut gateway = fixture.connect(state);
    fixture.open_shop(&mut gateway);

    for (instance, definition) in (900_001..).zip([WAND, CLOAK, BANDAGE]) {
        let coins = inventory_of(&gateway).coins;
        let rule = (purchase_price(definition) / 10).max(1);
        assert!(rule > 1, "item {definition} is a priced row");
        let done = completions(&send(&mut gateway, &sell_packet(0, instance, 1)));
        assert_eq!(done.len(), 1, "one 165/6 per sell");
        assert_eq!(done[0].result, RESULT_SUCCESS, "item {definition} sells");
        assert_eq!(done[0].price, rule, "item {definition} pays a tenth");
        assert_eq!(inventory_of(&gateway).coins, coins + rule);
    }
}

/// T640: the shipped rows still follow the T633 tenth rule, but a hand retune of a few rows is
/// data, not a regression: only a whole-rule regression (more than 1% of the priced rows off the
/// rule) fails. Free-issuance zeros (F228) and rows without a sell price are counted as off-rule.
#[test]
fn shipped_sell_prices_follow_the_tenth_rule_with_room_to_retune() {
    let priced: Vec<_> = rows()
        .iter()
        .filter(|row| row["no_sale"] == false && row["purchase_price_coins"].is_u64())
        .collect();
    assert!(
        priced.len() > 10_000,
        "the shipped store has its priced rows"
    );
    let off_rule = priced
        .iter()
        .filter(|row| {
            let rule = (row["purchase_price_coins"].as_u64().unwrap() / 10).max(1);
            row["sell_price_coins"].as_u64() != Some(rule)
        })
        .count();
    assert!(
        off_rule * 100 <= priced.len(),
        "{off_rule} of {} priced rows are off the tenth rule",
        priced.len()
    );
}

#[test]
fn granted_ore_sells_for_its_worth_and_survives_a_relog() {
    let fixture = Fixture::new(&[]);
    let inventory = Inventory::load(&fixture.content, None).expect("fixture inventory loads");
    let worth = listed(&login_rows(&inventory, &BTreeSet::from([ORE]))[&ORE])
        .expect("the client lists Copper Ore");
    let mut state = fixture.state();
    give(&mut state, &[(ORE, 2)]);
    let mut gateway = fixture.connect(state);
    fixture.open_shop(&mut gateway);

    let outcome = gateway
        .handle_at(&wrap(&sell_packet(0, 900_001, 1)), NOW)
        .expect("the ore sale handles");
    assert!(outcome.gameplay_changed, "a sale requests a save");
    let done = completions(&inner_packets(&outcome));
    assert_eq!(done.len(), 1);
    assert_eq!(
        done[0].result, RESULT_SUCCESS,
        "no store row, but the client's Worth"
    );
    assert_eq!(done[0].price, worth);
    let after = inventory_of(&gateway);
    assert_eq!(after.coins, 1_000 + worth);
    assert_eq!(count_definition(&after, ORE), 1);

    // Relog through the saved form: the coins and the remaining ore hold.
    let saved = serde_json::to_vec(&gateway.snapshot().gameplay.expect("gameplay state"))
        .expect("the save serializes");
    let state: GameplayState = serde_json::from_slice(&saved).expect("the save loads");
    let relogged = inventory_of(&fixture.connect(state));
    assert_eq!(relogged.coins, 1_000 + worth);
    assert_eq!(count_definition(&relogged, ORE), 1);
}

#[test]
fn rides_no_sale_rows_and_worn_items_are_refused_without_change() {
    let fixture = Fixture::new(&[]);
    let inventory = Inventory::load(&fixture.content, None).expect("fixture inventory loads");
    // Keep one worn item the Sell tab would otherwise list, so only the equipped rule refuses it.
    let worn = Gameplay::load(&fixture.content, None)
        .expect("fixture gameplay content loads")
        .state()
        .inventory;
    let (link, item) = worn
        .equipment
        .iter()
        .filter(|link| link.item_id != 0)
        .find_map(|link| {
            let item = worn.items.iter().find(|item| item.id == link.item_id)?;
            let row = login_rows(&inventory, &BTreeSet::from([item.definition_id]));
            listed(&row[&item.definition_id]).map(|_| (link.clone(), item.clone()))
        })
        .expect("a worn item the Sell tab lists");
    let mut state = fixture.state();
    state.inventory.items.push(item.clone());
    state
        .inventory
        .equipment
        .iter_mut()
        .find(|entry| entry.profile_id == link.profile_id && entry.slot == link.slot)
        .expect("the worn slot exists")
        .item_id = item.id;
    give(&mut state, &[(DRAGON, 1), (STORE_RIDE, 1)]);
    let mut gateway = fixture.connect(state);
    fixture.open_shop(&mut gateway);

    for instance in [900_001, 900_002, item.id] {
        let before = gateway.snapshot();
        let done = completions(&send(&mut gateway, &sell_packet(0, instance, 1)));
        assert_eq!(done.len(), 1, "a refused sell still answers 165/6");
        assert_ne!(done[0].result, RESULT_SUCCESS, "instance {instance} sold");
        assert_eq!(gateway.snapshot(), before, "a refused sell changed state");
    }
}

#[test]
fn no_priced_item_sells_above_its_lowest_buy_price() {
    let fixture = Fixture::new(&["profiles.json"]);
    let inventory = Inventory::load(&fixture.content, None).expect("fixture inventory loads");
    let profiles = Profiles::load(&fixture.content, None).expect("fixture profiles load");
    let stores = data().join("stores.json");
    let shop = CoinShop::new(
        &fixture.content,
        inventory.clone(),
        StoreCatalogue::load(&stores).expect("stores.json loads"),
        100,
        Some(&profiles),
        Wallet::default(),
    )
    .expect("the shop builds");
    let stocked: BTreeSet<i64> = read_json(&data().join("vendors.json"))["vendors"]
        .as_array()
        .expect("vendor records are an array")
        .iter()
        .flat_map(|vendor| vendor["items"].as_array().cloned().unwrap_or_default())
        .filter_map(|item| item["item_id"].as_i64())
        .collect();
    let rows = read_json(&stores);
    let rows = rows["items"].as_array().expect("store rows are an array");
    let (mut priced, mut sellable) = (0, 0);
    for row in rows {
        let id = row["item_id"].as_i64().expect("store row id") as i32;
        let Some(buy) = ["purchase_price_coins", "member_price_coins"]
            .iter()
            .filter_map(|field| row[*field].as_u64())
            .min()
        else {
            continue;
        };
        if let Some(sell) = shop.sell_unit_price(id) {
            assert!(u64::from(sell) <= buy, "item {id} sells for {sell} > {buy}");
            sellable += usize::from(stocked.contains(&i64::from(id)));
        }
        priced += usize::from(stocked.contains(&i64::from(id)));
    }
    // Sanity bounds from the 2026-10-22 measurement (1,012 priced stock rows, 992 sellable).
    assert!(
        priced > 500 && sellable > 500,
        "{priced} priced, {sellable} sellable"
    );

    // The Worth fallback buys what the client lists and stores.json never priced (442 rows
    // measured), and refuses every ride: they all carry NoSale (T429 keeps byte 45).
    let store_priced: BTreeSet<i64> = rows
        .iter()
        .filter(|row| !row["sell_price_coins"].is_null())
        .filter_map(|row| row["item_id"].as_i64())
        .collect();
    let fallback = inventory
        .definition_ids()
        .filter(|id| !store_priced.contains(&i64::from(*id)))
        .filter(|id| shop.sell_unit_price(*id).is_some_and(|price| price > 0))
        .count();
    assert!(fallback > 200, "{fallback} items paid by the client Worth");
    for ride in [DRAGON, STORE_RIDE] {
        assert_eq!(shop.sell_unit_price(ride), None, "ride {ride} is sellable");
    }
}
