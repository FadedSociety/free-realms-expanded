//! P-shops-marketplace: the welcome splash's Popular Items and the Marketplace's Hot New Items.
//!
//! Client path: `WelcomeHandler:Show` (ScriptsBase.lua) requests the wallet (66/10, `00b43bc0`),
//! then `InGamePurchaseStoreScreen.SelectStoreBundleGroup(6)`; `StoreBundleGroupContents__Refresh`
//! (`009a3a50`) looks group 6 up in the store's 66/9 group map and lists its bundles, and the
//! `StoreItemGroupContents` data-source update calls `WelcomeHandler:AddHotNewItems`, which fills
//! the panel from each bundle's name, description, price and image. Before this piece the 66/9
//! packet carried one group keyed 1 (the client's Miniclip affiliate group), so group 6 was empty
//! and the panel showed "undefined" (ROADMAP item 3, owner 2026-10-26). The Marketplace's own
//! "Hot New Items!" category (`MarketplaceCategoryIds.NEW_ITEMS`, `EnableNewStoreBundleFilter`)
//! lists the bundles whose IsNewItem byte (+241, `009a56d0` case 26) is set.

use osfr_tools::{
    coin_shop::{CoinShop, StoreCatalogue},
    inventory::Inventory,
    station_cash::{BundleGroup, Config, Wallet},
};
use std::path::PathBuf;

fn inventory_content() -> PathBuf {
    std::env::var_os("OSFR_STATION_CASH_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/reports/round70-native-20261016T061000Z/content")
        })
}

fn shipped_shop() -> CoinShop {
    let directory = inventory_content();
    let inventory = Inventory::load(&directory, None).expect("item content loads");
    let stores = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json");
    let catalogue = StoreCatalogue::load(&stores).expect("store catalogue loads");
    CoinShop::new(&directory, inventory, catalogue, usize::MAX, None, Wallet::default())
        .expect("shop loads")
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("four bytes"))
}

/// The 66/9 groups as (id, name id, bundle ids), read the way `00b45560` -> `00b3d770` does.
fn bundle_groups(packet: &[u8]) -> Vec<(u32, u32, Vec<u32>)> {
    assert_eq!(&packet[..4], &[66, 0, 9, 0], "66/9 header");
    let mut at = 4;
    let count = u32_at(packet, at) as usize;
    at += 4;
    let mut groups = Vec::with_capacity(count);
    for _ in 0..count {
        let key = u32_at(packet, at);
        let id = u32_at(packet, at + 4);
        let name_id = u32_at(packet, at + 8);
        assert_eq!(key, id, "group map key is its id");
        at += 16;
        for _ in 0..2 {
            let length = u32_at(packet, at) as usize;
            at += 4 + length;
        }
        at += 1 + 4;
        let entries = u32_at(packet, at) as usize;
        at += 4;
        let mut bundles = Vec::with_capacity(entries);
        for _ in 0..entries {
            assert_eq!(u32_at(packet, at), u32_at(packet, at + 4), "entry key is its bundle id");
            bundles.push(u32_at(packet, at));
            at += 12;
        }
        groups.push((id, name_id, bundles));
    }
    assert_eq!(at, packet.len(), "66/9 trailing bytes");
    groups
}

#[test]
fn the_welcome_splash_wallet_request_brings_the_popular_items_group() {
    let mut shop = shipped_shop();
    let outcome = shop.handle(&[66, 0, 10, 0]).expect("wallet request");
    let packet = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 9, 0]))
        .expect("the 66/10 reply carries the 66/9 bundle groups");
    let groups = bundle_groups(packet);
    let (_, name_id, bundles) = groups
        .iter()
        .find(|(id, _, _)| *id == 6)
        .expect("group 6, the one WelcomeHandler:Show selects");
    assert_eq!(*name_id, 156_763, "Legacy StoreBundleGroups.json Top Sellers");
    assert!(bundles.len() >= 4, "enough tiles for the carousel: {bundles:?}");
    for bundle in bundles {
        let listing = shop
            .sc_catalogue()
            .listing(*bundle)
            .expect("a popular item is a listed bundle");
        assert!(listing.price > 0 && listing.name_id > 0 && listing.description_id > 0);
    }
    assert!(
        groups.iter().all(|(id, _, _)| !(1..=3).contains(id)),
        "no group uses the client's Miniclip/Bigpoint/Members-Only ids: {groups:?}"
    );
}

#[test]
fn new_items_are_flagged_and_everything_else_is_not() {
    let shop = shipped_shop();
    let config = shop.sc_catalogue().config();
    assert!(!config.new_item_ids.is_empty(), "shipped data names new items");
    let flagged: Vec<i32> = shop
        .sc_catalogue()
        .listings()
        .iter()
        .filter(|listing| listing.new_item)
        .map(|listing| listing.item_id)
        .collect();
    let mut expected = config.new_item_ids.clone();
    expected.sort_unstable();
    assert_eq!(flagged, expected, "every new row is listed and flagged, nothing else is");
}

#[test]
fn bundle_group_data_is_refused_when_malformed() {
    let base = shipped_shop().sc_catalogue().config().clone();
    let group = |id: u32, name_id: u32, item_ids: Vec<i32>| BundleGroup {
        id,
        name_id,
        description_id: 0,
        item_ids,
    };
    let cases: [(&str, Vec<BundleGroup>, Vec<i32>); 5] = [
        ("duplicate id", vec![group(6, 1, vec![1]), group(6, 2, vec![2])], vec![]),
        ("zero id", vec![group(0, 1, vec![1])], vec![]),
        ("no name", vec![group(6, 0, vec![1])], vec![]),
        ("empty group", vec![group(6, 1, vec![])], vec![]),
        ("bad new item", vec![], vec![0]),
    ];
    for (what, bundle_groups, new_item_ids) in cases {
        let config = Config {
            bundle_groups,
            new_item_ids,
            ..base.clone()
        };
        assert!(config.validate().is_err(), "{what} must be refused");
    }
    assert!(base.validate().is_ok(), "the shipped policy loads");
}
