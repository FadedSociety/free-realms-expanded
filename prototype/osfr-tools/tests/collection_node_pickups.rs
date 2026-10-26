//! T524: sparkling collection nodes through the real 26/8 route (`Gameplay::handle_in_world`).
//!
//! A jobless player clicks (26/8) a placed `collection_nodes.json` node in range: one item
//! from the pool's weighted table is granted, that item's collection entry is recorded through
//! the shared T450 ledger path (the 38/10 follows the item grant), and the node hides until the
//! pool's respawn. Every refusal changes nothing. The suite also pins the AddNpc sparkle fields
//! (scoped to the collection nodes and the Chef mushrooms), the water companion actor, the
//! content gaps that must refuse, relog and the gateway dispatch.
//!
//! Node guids come from `harvest_nodes::collection_node_guid`; the roll is a per-player OS
//! entropy stream in production, so a test that pins an item seeds the stream with
//! `Gameplay::seed_collection_rolls` (the same seed on a probe `GatheringNodes` computes the
//! expected item without going through the request). The gameplay content is the private
//! opening package plus a definition for every water-pool drop; node, catalogue and coin
//! content are the repository's `data/content` unless a test builds a minimal NPC package.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/item_catalogue.rs"]
mod item_catalogue;
#[path = "support/json.rs"]
mod json;
#[path = "support/pickup_fixture.rs"]
mod pickup_fixture;

use osfr_tools::{
    character_store::CharacterState,
    collection_catalogue::CollectionCatalogueContent,
    gameplay::{Gameplay, Outcome},
    gateway::Gateway,
    harvest_nodes::{
        GatheringNodes, NodeActor, NodeChange, apply_node_changes, collection_node_guid,
    },
    harvesting::CHEF_PROFILE_ID,
    inventory::InventoryItem,
    mining::MINER_PROFILE_ID,
    npc_interaction::NpcInteractRequest,
    startup_world::parse_add_npc,
    world::NpcVisibility,
};
use pickup_fixture::{PickupFixture, added, at, bundled, removed, snapshot};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// Sparkling water: spawn 151 in the 18-drop `stillwater-sparkling-water` pool.
const WATER_NODE: u32 = 151;
/// Two nodes in different pools with the same 18-item table (Stillwater and Sanctuary
/// central), for the "entry already collected" case.
const SAME_ITEM_A: u32 = 186;
const SAME_ITEM_B: u32 = 840;
/// The rare-node subtitle locale row (`Rare Collection`) on every rare collection type.
const RARE_SUBTITLE: i32 = -2_118_222_202;
const FILLER: i32 = 1195;
/// The seeded click: `seed_collection_rolls(ROLL_SEED)` makes the next roll `WATER_ITEM`.
/// `WATER_ITEM` is the pinned first draw of SplitMix64 seed 7 over the water pool's weights
/// (18 rows of weight 100, in `collection_nodes.json` order); it is in the middle of the table
/// so an "always the first drop" bug cannot pass.
const ROLL_SEED: u64 = 7;
const WATER_ITEM: i32 = 23_179;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn shipped_nodes() -> GatheringNodes {
    GatheringNodes::load(&bundled())
        .expect("the shipped node tables load")
        .expect("the shipped node tables are present")
}

/// The first draw of `seed` on `spawn_id`'s pool, computed on a probe loader (no request).
fn seeded_item(seed: u64, spawn_id: u32) -> i32 {
    let mut nodes = shipped_nodes();
    nodes.seed_rolls(seed);
    nodes
        .collect_item(collection_node_guid(spawn_id))
        .expect("a collect roll") as i32
}

fn node(spawn_id: u32) -> NodeActor {
    let guid = collection_node_guid(spawn_id);
    shipped_nodes()
        .actors()
        .find(|actor| actor.guid == guid)
        .unwrap_or_else(|| panic!("collection spawn {spawn_id} is placed"))
        .clone()
}

fn pool_drops(spawn_id: u32) -> Vec<u32> {
    let content = json::read(&bundled().join("collection_nodes.json"));
    let spawn = content["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|spawn| spawn["id"] == spawn_id)
        .expect("spawn row");
    let key = spawn["pool_key"].as_str().unwrap();
    content["pools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pool| pool["key"] == key)
        .expect("pool row")["drop_table"]
        .as_array()
        .unwrap()
        .iter()
        .map(|drop| drop["item_definition_id"].as_u64().unwrap() as u32)
        .collect()
}

fn catalogue() -> CollectionCatalogueContent {
    CollectionCatalogueContent::load(&bundled())
        .expect("catalogue load")
        .expect("the catalogue file")
}

/// Define every drop of the water pool in the fixture pack, so a click can never refuse for an
/// undefined item (the gateway test cannot seed the roll; the pinned test still knows its item).
fn define_water_drops(fixture: &PickupFixture) {
    let rows: Vec<Value> = pool_drops(WATER_NODE)
        .into_iter()
        .map(|item| item_catalogue::sized_row(item, 64))
        .collect();
    item_catalogue::write(fixture.dir(), &rows);
}

fn click(game: &mut Gameplay, target: u64, who: &CharacterState, now_ms: u64) -> Outcome {
    // The real client reports its pose, which refreshes the NPC view, before the click.
    view(game, who);
    raw_click(game, target, who, now_ms)
}

fn raw_click(game: &mut Gameplay, target: u64, who: &CharacterState, now_ms: u64) -> Outcome {
    let request = NpcInteractRequest {
        target,
        trailing_flag: false,
    };
    game.handle_in_world(&request.encode(), who, now_ms)
        .expect("the 26/8 request is accepted")
}

fn view(game: &mut Gameplay, who: &CharacterState) {
    game.update_npc_visibility(who)
        .expect("the NPC view update is served");
}

fn said(outcome: &Outcome, needle: &str) -> bool {
    outcome.replies.iter().any(|packet| {
        if !packet.starts_with(&[15, 0, 3, 0]) || packet.len() < 9 {
            return false;
        }
        let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
        packet
            .get(8..8 + length)
            .is_some_and(|text| String::from_utf8_lossy(text).contains(needle))
    })
}

/// Definition counts that a pickup could change; rentals are not stack rows.
fn counts(items: &[InventoryItem]) -> BTreeMap<i32, u32> {
    let mut counts = BTreeMap::new();
    for item in items.iter().filter(|item| item.rental.is_none()) {
        *counts.entry(item.definition_id).or_default() += item.count;
    }
    counts
}

/// The items a pickup can change, one entry per definition.
fn delta(before: &[InventoryItem], after: &[InventoryItem]) -> Vec<(i32, u32)> {
    let before = counts(before);
    let after = counts(after);
    let mut changed = Vec::new();
    for definition in before.keys().chain(after.keys()).collect::<BTreeSet<_>>() {
        let from = before.get(definition).copied().unwrap_or(0);
        let to = after.get(definition).copied().unwrap_or(0);
        if from != to {
            changed.push((*definition, to.saturating_sub(from)));
        }
    }
    changed.sort();
    changed
}

fn state_delta(
    before: &osfr_tools::gameplay::GameplayState,
    after: &osfr_tools::gameplay::GameplayState,
) -> Vec<(i32, u32)> {
    delta(&before.inventory.items, &after.inventory.items)
}

fn collected(game: &Gameplay, entry: i32) -> bool {
    game.state()
        .progression_extras
        .is_some_and(|extras| extras.collections.is_collected(entry))
}

fn add_entry(entry_id: i32) -> Vec<u8> {
    let catalogue = catalogue();
    let (entry, collection) = catalogue.entry(entry_id).expect("entry row");
    let mut packet = vec![38, 0, 10, 0];
    for value in [
        entry.id as u32,
        entry.icon_id as u32,
        entry.icon_tint_id as u32,
        entry.name_id as u32,
        collection.id as u32,
        entry.index,
        0,
    ] {
        packet.extend(value.to_le_bytes());
    }
    packet.push(1);
    packet
}

fn entry_packet(outcome: &Outcome) -> Option<&Vec<u8>> {
    outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[38, 0, 10, 0]))
}

/// The first item grant (38/2 add or 38/3 count update) after the request.
fn item_packet(outcome: &Outcome) -> Option<usize> {
    outcome
        .replies
        .iter()
        .position(|packet| packet.starts_with(&[38, 0]) && !packet.starts_with(&[38, 0, 10, 0]))
}

fn entry_packet_index(outcome: &Outcome) -> Option<usize> {
    outcome
        .replies
        .iter()
        .position(|packet| packet.starts_with(&[38, 0, 10, 0]))
}

fn fill_inventory(state: &mut osfr_tools::gameplay::GameplayState) {
    let items = &mut state.inventory.items;
    let mut next = items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    while items.len() < 4096 {
        items.push(InventoryItem {
            id: next,
            definition_id: FILLER,
            tint: 0,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        next += 1;
    }
    state.inventory.next_instance_id = Some(i64::from(next));
}

#[test]
fn a_jobless_player_gets_the_pinned_item_and_its_entry_through_the_real_click() {
    let fixture = PickupFixture::new("collect-node");
    define_water_drops(&fixture);
    let state = fixture.jobless();
    let profiles = state.profiles.as_ref().expect("the opening profiles");
    assert!(
        !profiles.unlocked.contains(&MINER_PROFILE_ID)
            && !profiles.unlocked.contains(&CHEF_PROFILE_ID),
        "the canary needs neither gather job unlocked"
    );
    let mut game = fixture.load(Some(state));
    let node = node(WATER_NODE);
    let who = at(node.position);
    let before = game.state();
    assert_eq!(
        seeded_item(ROLL_SEED, WATER_NODE),
        WATER_ITEM,
        "the pinned seed's first draw moved"
    );
    game.seed_collection_rolls(ROLL_SEED);
    let out = click(&mut game, node.guid, &who, 1_000);

    assert!(out.changed, "the pickup is a saved change");
    // Exactly one item, the one the seeded stream pinned.
    assert_eq!(
        state_delta(&before, &game.state()),
        vec![(WATER_ITEM, 1)],
        "exactly one item definition changed"
    );
    assert!(
        pool_drops(WATER_NODE).contains(&(WATER_ITEM as u32)),
        "item {WATER_ITEM} is not in the pool table"
    );
    // The item grant precedes the entry tick (38/10).
    let item_at = item_packet(&out).expect("the item grant packet");
    let entry_at = entry_packet_index(&out).expect("the 38/10 entry packet");
    assert!(
        item_at < entry_at,
        "the 38/10 did not follow the item grant"
    );
    // 38/10 carries the catalogue row for the item's entry, exactly once.
    let entry = catalogue()
        .entry_for_item(WATER_ITEM)
        .expect("every shipped drop item maps to one entry");
    assert!(collected(&game, entry));
    let mut add_entry_out = out
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[38, 0, 10, 0]));
    assert_eq!(add_entry_out.next(), Some(&add_entry(entry)));
    assert!(add_entry_out.next().is_none(), "two 38/10 packets");
    assert!(!said(&out, "You need to be a"), "a job gate ran");
    assert!(
        !said(&out, "You don't need that"),
        "the pickup fell through"
    );
    // The node hides for this player in the same click.
    assert!(removed(&out.replies, node.guid), "the node was not hidden");
}

#[test]
fn a_second_node_with_the_same_item_grants_it_and_records_nothing() {
    let fixture = PickupFixture::new("collect-node-repeat");
    define_water_drops(&fixture);
    let mut game = fixture.load(Some(fixture.jobless()));
    let first = node(SAME_ITEM_A);
    let second = node(SAME_ITEM_B);
    // The two pools hold the same 18-item table, so the same seed draws the same item on both.
    let first_item = seeded_item(ROLL_SEED, SAME_ITEM_A);
    assert_eq!(first_item, seeded_item(ROLL_SEED, SAME_ITEM_B));

    let before = game.state();
    game.seed_collection_rolls(ROLL_SEED);
    let first_out = click(&mut game, first.guid, &at(first.position), 1_000);
    assert!(
        entry_packet(&first_out).is_some(),
        "the first find records its entry"
    );
    let entry = catalogue().entry_for_item(first_item).expect("entry");
    assert!(collected(&game, entry));

    let after_first = game.state();
    game.seed_collection_rolls(ROLL_SEED);
    let second_out = click(&mut game, second.guid, &at(second.position), 2_000);
    assert_eq!(
        state_delta(&after_first, &game.state()),
        vec![(first_item, 1)]
    );
    assert!(
        entry_packet(&second_out).is_none(),
        "a known entry was recorded twice"
    );
    assert!(collected(&game, entry));
    let ledger = game.state().progression_extras.unwrap().collections;
    assert_eq!(ledger.collected_count(), 1, "one entry was recorded");
    assert_eq!(state_delta(&before, &game.state()), vec![(first_item, 2)]);
}

#[test]
fn the_node_hides_now_and_returns_after_its_pool_respawn() {
    let fixture = PickupFixture::new("collect-node-respawn");
    define_water_drops(&fixture);
    let mut game = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    game.seed_collection_rolls(ROLL_SEED);
    let out = click(&mut game, node.guid, &who, 1_000);
    assert!(removed(&out.replies, node.guid), "the click hides the node");
    // Water pools respawn in 60 s.
    assert!(!added(
        &game.tick(&who, 1_000 + 59_999).unwrap().replies,
        node.guid
    ));
    let back = game.tick(&who, 1_000 + 60_000).unwrap();
    assert!(
        added(&back.replies, node.guid),
        "the node did not grow back"
    );

    // The returned node rolls afresh: the next seeded draw is granted.
    game.seed_collection_rolls(ROLL_SEED + 1);
    let expected = seeded_item(ROLL_SEED + 1, WATER_NODE);
    let before = game.state();
    click(&mut game, node.guid, &who, 1_000 + 60_100);
    assert_eq!(state_delta(&before, &game.state()), vec![(expected, 1)]);
}

#[test]
fn every_click_draws_afresh_from_the_pools_table() {
    let mut nodes = shipped_nodes();
    let spawn = busiest_pool_spawn();
    let guid = collection_node_guid(spawn);
    let table = pool_drops(spawn);
    nodes.seed_rolls(9);
    assert!(!nodes.sync(0, false).is_empty());
    let mut now = 1_000;
    let mut items = BTreeSet::new();
    for _ in 0..200 {
        let item = nodes.collect_item(guid).expect("a roll");
        assert!(table.contains(&item), "a roll {item} left the pool's table");
        assert!(
            nodes.consume(guid, now).is_some(),
            "the click did not hide the node"
        );
        now += 61_000;
        nodes.sync(now, false);
        items.insert(item);
    }
    assert!(
        items.len() > 40,
        "200 draws on one node of a {}-drop pool reached only {} items",
        table.len(),
        items.len()
    );
}

/// A scratch directory holding just the node files a test writes.
struct Scratch(PathBuf);

impl Scratch {
    fn new(prefix: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-{prefix}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, file: &str, value: Value) {
        fs::write(self.path().join(file), serde_json::to_vec(&value).unwrap()).unwrap();
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// One synthetic node file: `type_key`'s type, one pool and one spawn at `position`.
fn node_file(
    type_key: &str,
    name: &str,
    model_id: u32,
    effect: u32,
    extra: &[u32],
    drops: &[(u32, u32)],
    spawn_id: u32,
) -> Value {
    json!({
        "schema_version": 1,
        "node_types": [{
            "key": type_key,
            "name": name,
            "model_id": model_id,
            "scale": 1.0,
            "composite_effect_id": effect,
            "extra_composite_effect_ids": extra,
            "interact_range": 12,
            "cursor_id": 0,
            "placement_y_offset": 0.0,
            "drop_table": [],
        }],
        "pools": [{
            "key": "pool",
            "zone_definition_id": 1,
            "node_type": type_key,
            "max_active_nodes": 1,
            "respawn_seconds": 60,
            "drop_table": drops
                .iter()
                .map(|(item, weight)| json!({"item_definition_id": item, "weight": weight}))
                .collect::<Vec<_>>(),
        }],
        "spawns": [{
            "id": spawn_id,
            "pool_key": "pool",
            "zone_definition_id": 1,
            "position": [0.0, 0.0, 0.0],
            "heading": 0.0,
        }],
    })
}

#[test]
fn the_collection_roll_follows_the_pools_weights() {
    let scratch = Scratch::new("collect-weights");
    scratch.write(
        "collection_nodes.json",
        node_file(
            "synthetic",
            "Synthetic node",
            1,
            5742,
            &[],
            &[(11, 1), (12, 2), (13, 7)],
            900,
        ),
    );
    let mut nodes = GatheringNodes::load(scratch.path())
        .unwrap()
        .expect("the synthetic collection node loads");
    let guid = collection_node_guid(900);
    let mut counts = BTreeMap::new();
    for seed in 0..20_000u64 {
        nodes.seed_rolls(seed);
        let item = nodes.collect_item(guid).expect("a roll");
        *counts.entry(item).or_insert(0u32) += 1;
    }
    for (item, weight) in [(11u32, 1u32), (12, 2), (13, 7)] {
        let share = f64::from(counts[&item]) / 20_000.0;
        let expected = f64::from(weight) / 10.0;
        assert!(
            (share - expected).abs() < 0.02,
            "item {item}: {share} vs {expected}"
        );
    }
}

/// A spawn of the pool with the most drops (the junk pools, 246+ items on a few nodes).
fn busiest_pool_spawn() -> u32 {
    let content = json::read(&bundled().join("collection_nodes.json"));
    let pool = content["pools"]
        .as_array()
        .unwrap()
        .iter()
        .max_by_key(|pool| pool["drop_table"].as_array().unwrap().len())
        .expect("a pool");
    content["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|spawn| spawn["pool_key"] == pool["key"])
        .expect("a spawn of the pool")["id"]
        .as_u64()
        .unwrap() as u32
}

#[test]
fn refusals_change_nothing() {
    let node = node(WATER_NODE);

    // A full bag refuses with the gathering text and keeps the item and the ledger.
    let fixture = PickupFixture::new("collect-refuse-bag");
    define_water_drops(&fixture);
    let mut state = fixture.jobless();
    fill_inventory(&mut state);
    let mut game = fixture.load(Some(state));
    let who = at(node.position);
    view(&mut game, &who);
    let before = snapshot(&game);
    game.seed_collection_rolls(ROLL_SEED);
    let out = raw_click(&mut game, node.guid, &who, 1_000);
    assert!(said(&out, "Your bag is full. Make room before gathering."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
    assert!(
        !removed(&out.replies, node.guid),
        "a refused click hid the node"
    );
    assert!(!collected(
        &game,
        catalogue().entry_for_item(WATER_ITEM).unwrap()
    ));

    // Out of range and off the open world refuse with the shared texts.
    let mut game = fixture.load(Some(fixture.jobless()));
    for who in [
        at([node.position[0] + 40.0, node.position[1], node.position[2]]),
        CharacterState {
            zone_id: 5,
            ..at(node.position)
        },
    ] {
        view(&mut game, &who);
        let before = snapshot(&game);
        game.seed_collection_rolls(ROLL_SEED);
        let out = raw_click(&mut game, node.guid, &who, 1_000);
        assert!(said(&out, "Move closer to use that."));
        assert!(!out.changed);
        assert_eq!(snapshot(&game), before);
        assert!(!removed(&out.replies, node.guid));
    }
    // One player's busy run refuses another activity, unchanged.
    let mut state = fixture.jobless();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.unlocked.insert(MINER_PROFILE_ID);
    profiles.active = MINER_PROFILE_ID;
    let mut game = fixture.load(Some(state));
    let vein = shipped_nodes()
        .actors()
        .find(|actor| actor.name == "Copper Vein")
        .expect("a copper vein")
        .clone();
    assert!(
        click(&mut game, vein.guid, &at(vein.position), 1_000).changed,
        "the mining run did not start"
    );
    let who = at(node.position);
    view(&mut game, &who);
    let before = snapshot(&game);
    game.seed_collection_rolls(ROLL_SEED);
    let out = raw_click(&mut game, node.guid, &who, 1_100);
    assert!(said(&out, "Finish what you are doing first."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);

    // A second click on the hidden node gets the existing Gone text and no grant.
    let mut game = fixture.load(Some(fixture.jobless()));
    let who = at(node.position);
    game.seed_collection_rolls(ROLL_SEED);
    assert!(click(&mut game, node.guid, &who, 1_000).changed);
    let before = snapshot(&game);
    let out = raw_click(&mut game, node.guid, &who, 2_000);
    assert!(said(&out, "That has already been gathered."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
}

/// The fixture folder is the session's NPC content root, so the click sees only the node file
/// copied here and the collection files this test supplies.
fn npc_package_with_node_file(prefix: &str, catalogue: bool) -> PickupFixture {
    let fixture = PickupFixture::npc_package(prefix);
    fs::copy(
        bundled().join("collection_nodes.json"),
        fixture.dir().join("collection_nodes.json"),
    )
    .unwrap();
    if catalogue {
        for file in ["collection_catalogue.json", "collection_coins.json"] {
            fs::copy(bundled().join(file), fixture.dir().join(file)).unwrap();
        }
    }
    fixture
}

#[test]
fn a_missing_collection_catalogue_refuses_before_any_grant() {
    let fixture = npc_package_with_node_file("collect-no-catalogue", false);
    let mut game = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    view(&mut game, &who);
    let before = snapshot(&game);
    let out = raw_click(&mut game, node.guid, &who, 1_000);
    assert!(said(&out, "You cannot gather that right now."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
    assert!(!removed(&out.replies, node.guid), "the node was hidden");
    assert!(state_delta(&before.0, &game.state()).is_empty());
}

/// Rewrite every pool's drop table to the one item, so the click's roll is fixed.
fn retune_drops(fixture: &PickupFixture, item: u32) {
    let path = fixture.dir().join("collection_nodes.json");
    let mut content = json::read(&path);
    for pool in content["pools"].as_array_mut().unwrap() {
        pool["drop_table"] = json!([{"item_definition_id": item, "weight": 1}]);
    }
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
}

#[test]
fn a_drop_without_a_catalogue_entry_refuses_before_any_grant() {
    let fixture = npc_package_with_node_file("collect-no-entry", true);
    // Defined by the fixture's item catalogue but linked to no collection entry.
    item_catalogue::write(fixture.dir(), &[item_catalogue::sized_row(9_999_999, 64)]);
    retune_drops(&fixture, 9_999_999);
    let mut game = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    view(&mut game, &who);
    let before = snapshot(&game);
    let out = raw_click(&mut game, node.guid, &who, 1_000);
    assert!(
        catalogue().entry_for_item(9_999_999).is_none(),
        "the fixture item must not have an entry"
    );
    assert!(said(&out, "You cannot gather that right now."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
    assert!(!removed(&out.replies, node.guid), "the node was hidden");
    assert!(state_delta(&before.0, &game.state()).is_empty());
}

#[test]
fn an_undefined_drop_refuses_before_any_grant() {
    let fixture = npc_package_with_node_file("collect-undefined", true);
    retune_drops(&fixture, 9_999_998);
    let mut game = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    view(&mut game, &who);
    let before = snapshot(&game);
    let out = raw_click(&mut game, node.guid, &who, 1_000);
    assert!(said(&out, "You cannot gather that right now."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
    assert!(!removed(&out.replies, node.guid), "the node was hidden");
    assert!(state_delta(&before.0, &game.state()).is_empty());
}

#[test]
fn the_actor_packets_carry_the_types_sparkle_and_the_water_companion() {
    let fixture = PickupFixture::new("collect-actor-packets");
    let mut game = fixture.load(None);
    let nodes = shipped_nodes();
    let find = |name: &str| {
        nodes
            .actors()
            .find(|actor| nodes.is_collect(actor.guid) && actor.name == name)
            .unwrap_or_else(|| panic!("a {name} collect node"))
            .clone()
    };

    // Flowers: the common pixie dust on a normal interactable actor.
    let flowers = find("Pile of Flowers");
    let packets = game.update_npc_visibility(&at(flowers.position)).unwrap();
    let spawn = parsed(&packets, flowers.guid);
    assert_eq!(spawn.model_id, 248);
    assert_eq!(spawn.composite_effect_id, 5742, "PFX_pixie_dust_small");
    assert_eq!(spawn.interact_range, 12);
    assert_eq!(spawn.sub_text_name_string_id, -1);
    assert_eq!(spawn.is_interactable, 1);
    assert!(companion(&packets, &flowers).is_none());

    // Rare flowers: the big effect and the Rare Collection subtitle.
    let rare = find("Rare Pile of Flowers");
    let packets = game.update_npc_visibility(&at(rare.position)).unwrap();
    let spawn = parsed(&packets, rare.guid);
    assert_eq!(spawn.model_id, 670);
    assert_eq!(spawn.composite_effect_id, 5740);
    assert_eq!(spawn.sub_text_name_string_id, RARE_SUBTITLE);

    // Sparkling water: the water effect on the node, pixie dust on an invisible
    // non-interactable companion (tag 4) at the same spot.
    let water = find("Sparkling Water");
    let packets = game.update_npc_visibility(&at(water.position)).unwrap();
    let spawn = parsed(&packets, water.guid);
    assert_eq!(spawn.model_id, 271);
    assert_eq!(spawn.composite_effect_id, 15381, "water bubbles");
    let companion = companion(&packets, &water).expect("the water companion");
    assert_eq!(companion.model_id, 271, "invisible cube");
    assert_eq!(companion.composite_effect_id, 5742);
    assert_eq!(companion.is_interactable, 0);
    assert_eq!(
        companion.position,
        [water.position[0], water.position[1], water.position[2], 1.0]
    );

    // T524 review: the Chef mushrooms only gain their sparkle; their AddNpc range and
    // subtitle stay the captured template's. Mining actors keep the whole template.
    let mushroom = nodes
        .actors()
        .find(|actor| actor.name == "Mushrooms")
        .expect("a mushroom node")
        .clone();
    let packets = game.update_npc_visibility(&at(mushroom.position)).unwrap();
    let spawn = parsed(&packets, mushroom.guid);
    assert_eq!(spawn.composite_effect_id, 5742);
    assert_eq!(spawn.interact_range, 100);
    assert_eq!(spawn.sub_text_name_string_id, -1);
    let ore = nodes
        .actors()
        .find(|actor| actor.name == "Copper Vein")
        .expect("a copper vein")
        .clone();
    let packets = game.update_npc_visibility(&at(ore.position)).unwrap();
    let spawn = parsed(&packets, ore.guid);
    assert_eq!(spawn.composite_effect_id, 0);
    assert_eq!(spawn.interact_range, 100);
}

fn parsed(packets: &[Vec<u8>], guid: u64) -> osfr_tools::startup_world::NpcSpawn {
    packets
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|spawn| spawn.actor_id == guid)
        .unwrap_or_else(|| panic!("no AddNpc for {guid:#x}"))
}

/// The tag-4 companion of a collection node, when it has one.
fn companion(packets: &[Vec<u8>], node: &NodeActor) -> Option<osfr_tools::startup_world::NpcSpawn> {
    let companion = node.companion_guid()?;
    packets
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|spawn| spawn.actor_id == companion)
}

#[test]
fn hiding_a_non_collect_node_keeps_the_water_companion_visible() {
    // A quest node and a water collection node share spawn id 900, so their tag-4 companion
    // guid is the same; hiding the quest node must not remove the water companion.
    let scratch = Scratch::new("collect-companion");
    scratch.write(
        "quest_nodes.json",
        node_file("tin-ore", "Tin Ore", 611, 5386, &[], &[], 900),
    );
    scratch.write(
        "collection_nodes.json",
        node_file(
            "sparkling-water",
            "Sparkling Water",
            271,
            15381,
            &[5742],
            &[(11, 1)],
            900,
        ),
    );
    let mut nodes = GatheringNodes::load(scratch.path())
        .unwrap()
        .expect("both synthetic node files load");
    let quest = nodes
        .actors()
        .find(|actor| actor.name == "Tin Ore")
        .expect("the quest node")
        .guid;
    let water = nodes
        .actors()
        .find(|actor| actor.name == "Sparkling Water")
        .expect("the water node")
        .clone();
    let companion = water.companion_guid().expect("the water companion");

    let mut visibility = NpcVisibility::default();
    apply_node_changes(&mut visibility, nodes.sync(0, false));
    let packets = visibility.update(0, [0.0, 0.0, 0.0]).unwrap();
    assert!(added(&packets, companion), "the companion was not drawn");

    let hide = nodes
        .consume(quest, 1_000)
        .expect("the quest node is shown");
    assert!(
        matches!(
            hide,
            NodeChange::Hide {
                companion: None,
                ..
            }
        ),
        "a quest node carried a companion"
    );
    apply_node_changes(&mut visibility, vec![hide]);
    let after = visibility.update(0, [0.0, 0.0, 0.0]).unwrap();
    assert!(removed(&after, quest), "the quest node did not hide");
    assert!(
        !removed(&after, companion),
        "hiding the quest node removed the water companion"
    );
    assert!(
        visibility.visible().contains(&companion),
        "the water companion is no longer tracked"
    );
}

#[test]
fn relog_keeps_the_entry_and_shows_every_node_again() {
    let fixture = PickupFixture::new("collect-node-relog");
    define_water_drops(&fixture);
    let mut game = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    game.seed_collection_rolls(ROLL_SEED);
    assert!(click(&mut game, node.guid, &who, 1_000).changed);
    let entry = catalogue().entry_for_item(WATER_ITEM).unwrap();
    assert!(collected(&game, entry));

    let mut relog = fixture.load(Some(game.state()));
    assert!(collected(&relog, entry), "the saved ledger lost the entry");
    // Nodes are personal and never saved: every one is shown again.
    let packets = relog.update_npc_visibility(&who).unwrap();
    assert!(added(&packets, node.guid));
    relog.seed_collection_rolls(ROLL_SEED);
    assert!(click(&mut relog, node.guid, &who, 2_000).changed);
    assert!(collected(&relog, entry));
}

#[test]
fn a_pickup_is_personal_to_the_player() {
    let fixture = PickupFixture::new("collect-node-personal");
    define_water_drops(&fixture);
    let mut first = fixture.load(Some(fixture.jobless()));
    let mut second = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    first.seed_collection_rolls(ROLL_SEED);
    assert!(click(&mut first, node.guid, &who, 1_000).changed);
    second.seed_collection_rolls(ROLL_SEED);
    let out = click(&mut second, node.guid, &who, 1_000);
    assert!(out.changed, "the second player could not pick the node");
    assert!(entry_packet(&out).is_some(), "the second ledger was empty");
}

/// The gateway dispatch: 26/8 reaches the collect handler and the outcome is saved. The roll is
/// the server's own entropy stream here (no gateway seed hook), so the item is read back from
/// the save and checked against the pool/catalogue rather than pinned.
#[test]
fn the_gateway_click_route_grants_and_records() {
    let fixture = PickupFixture::new("collect-gateway");
    define_water_drops(&fixture);
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    let node = node(WATER_NODE);
    let mut character: CharacterState = Gateway::reference_state(&reference).unwrap();
    character.zone_id = 0;
    character.position = node.position;
    let mut gateway =
        gateway_vendor::connect_with(&reference, character, fixture.dir(), fixture.jobless());
    let before = gateway.snapshot().gameplay.expect("gameplay").inventory;
    let outcome = gateway
        .handle_at(
            &gateway_vendor::wrap(&gateway_vendor::interact(node.guid)),
            11_000,
        )
        .expect("the gateway handles the click");
    assert!(
        outcome.gameplay_changed,
        "the gateway did not persist the pickup"
    );
    let after = gateway.snapshot().gameplay.expect("gameplay");
    let gained = delta(&before.items, &after.inventory.items);
    assert_eq!(gained.len(), 1, "one definition changed: {gained:?}");
    let (item, count) = gained[0];
    assert_eq!(count, 1);
    assert!(
        pool_drops(WATER_NODE).contains(&(item as u32)),
        "item {item} is not in the click's pool table"
    );
    let entry = catalogue()
        .entry_for_item(item)
        .expect("shipped drop entry");
    assert!(
        after
            .progression_extras
            .is_some_and(|extras| extras.collections.is_collected(entry)),
        "the gateway route did not record the entry"
    );
    let replies = gateway_vendor::inner_packets(&outcome);
    assert!(
        replies.iter().any(|packet| packet == &add_entry(entry)),
        "the 38/10 did not reach the client"
    );
    assert!(
        removed(&replies, node.guid),
        "the node removal did not reach the client"
    );
}

/// T524 review: a mining node type that declares an extra effect is refused, because only the
/// collection kind draws a tag-4 companion.
#[test]
fn a_non_collect_extra_effect_is_refused_at_load() {
    let scratch = Scratch::new("collect-extra-effect");
    scratch.write(
        "mining_nodes.json",
        node_file("copper-vein", "Copper Vein", 259, 0, &[5742], &[], 1),
    );
    let error = GatheringNodes::load(scratch.path())
        .err()
        .expect("a mining type must not declare an extra effect")
        .to_string();
    assert!(error.contains("extra composite effect"), "{error}");
}
