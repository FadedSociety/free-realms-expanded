//! T524: sparkling collection nodes through the real 26/8 route (`Gameplay::handle_in_world`).
//!
//! A jobless player clicks (26/8) a placed `collection_nodes.json` node in range: one item
//! from the pool's weighted table is granted, that item's collection entry is recorded through
//! the shared T450 ledger path (the 38/10 follows the item grant), and the node hides until the
//! pool's respawn. Every refusal changes nothing. The suite also pins the AddNpc sparkle fields,
//! the water companion actor, relog and the gateway dispatch.
//!
//! Node guids are `harvest_nodes`' "NOD" base, tag 5 (`collection_nodes.json`) and the content
//! spawn id; the water companion is tag 4. The gameplay content is the private opening package
//! (its item definitions are the ones the campaign ships) plus the shipped Mining18 board for
//! the busy case; node, catalogue and coin content are the repository's `data/content`.

#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "support/json.rs"]
mod json;

use osfr_tools::{
    character_store::CharacterState,
    collection_catalogue::CollectionCatalogueContent,
    gameplay::{Gameplay, GameplayState, Outcome},
    gateway::{Gateway, GatewayOutcome},
    harvest_nodes::{GatheringNodes, NodeActor},
    harvesting::CHEF_PROFILE_ID,
    inventory::InventoryItem,
    mining::{self, MINER_PROFILE_ID},
    npc_interaction::NpcInteractRequest,
    startup_world::parse_add_npc,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// `harvest_nodes::NODE_GUID_BASE`; the loader owns the value, the test only builds clicks.
const NODE_GUID_BASE: u64 = 0x4e4f_4400_0000_0000;
/// Sparkling water nodes: spawns 186 and 840 share a pool (for the second-find gate).
const WATER_NODE: u32 = 151;
const SAME_ITEM_A: u32 = 186;
const SAME_ITEM_B: u32 = 840;
/// The rare-node subtitle locale row (`Rare Collection`) on every rare collection type.
const RARE_SUBTITLE: i32 = -2_118_222_202;
const FILLER: i32 = 1195;
const PLAYER: u64 = 1;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn bundled() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn collect_guid(spawn_id: u32) -> u64 {
    NODE_GUID_BASE | 5 << 32 | u64::from(spawn_id)
}

fn shipped_nodes() -> GatheringNodes {
    GatheringNodes::load(&bundled())
        .expect("the shipped node tables load")
        .expect("the shipped node tables are present")
}

/// The roll of `WATER_NODE` for the test player at `now_ms`: the seed is the player guid,
/// node guid and click time, so a click's item is pinned by the click's `now_ms`.
fn water_item(now_ms: u64) -> i32 {
    shipped_nodes()
        .collect_item(collect_guid(WATER_NODE), PLAYER, now_ms)
        .expect("a collect roll") as i32
}

/// The item definitions of the fixture's inventory content (the private opening package
/// defines far fewer than the shipped item catalogue, so most pool items are undefined here).
fn defined_items() -> BTreeSet<i32> {
    let inventory = json::read(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/opening-journal-text-v2/content/inventory.json"),
    );
    inventory["definitions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|definition| definition["id"].as_i64().unwrap() as i32)
        .collect()
}

/// The first click time at or after `from_ms` whose `WATER_NODE` roll the fixture defines.
fn water_click_at(from_ms: u64) -> u64 {
    let defined = defined_items();
    (from_ms..)
        .find(|now_ms| defined.contains(&water_item(*now_ms)))
        .expect("some roll lands on a defined item")
}

fn node(spawn_id: u32) -> NodeActor {
    let guid = collect_guid(spawn_id);
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

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: PLAYER,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
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

fn removed(outcome: &Outcome, guid: u64) -> bool {
    outcome
        .replies
        .iter()
        .any(|packet| packet.starts_with(&[35, 0, 3, 0]) && has_guid(packet, guid))
}

fn added(outcome: &Outcome, guid: u64) -> bool {
    outcome
        .replies
        .iter()
        .any(|packet| packet.starts_with(&[35, 0, 2, 0]) && has_guid(packet, guid))
}

fn has_guid(packet: &[u8], guid: u64) -> bool {
    packet.windows(8).any(|window| window == guid.to_le_bytes())
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
fn counts(state: &GameplayState) -> BTreeMap<i32, u32> {
    let mut counts = BTreeMap::new();
    for item in &state.inventory.items {
        if item.rental.is_none() {
            *counts.entry(item.definition_id).or_default() += item.count;
        }
    }
    counts
}

/// The item a pickup can change, one entry per definition.
fn delta(before: &GameplayState, after: &GameplayState) -> Vec<(i32, u32)> {
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

fn collected(game: &Gameplay, entry: i32) -> bool {
    game.state()
        .progression_extras
        .is_some_and(|extras| extras.collections.is_collected(entry))
}

/// What a refusal must leave alone: the save and the visibility revision.
fn snapshot(game: &Gameplay) -> (GameplayState, u64) {
    (game.state(), game.npc_visibility_revision())
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

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-collect-node-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let opening = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/opening-journal-text-v2/content");
        content_copy::copy_tree(&opening, &dir);
        // The busy case starts a real Mining18 run; its board and outputs are shipped.
        fs::copy(
            bundled().join(mining::CONTENT_FILE),
            dir.join(mining::CONTENT_FILE),
        )
        .unwrap();
        Self { dir }
    }

    fn load(&self, state: Option<GameplayState>) -> Gameplay {
        Gameplay::load(&self.dir, state).expect("gameplay content loads")
    }

    /// The opening save: neither gather job is unlocked, so a collect click that ran the
    /// gather admission would answer NoJob. The snapshot requires the active job to stay
    /// unlocked, so the minimum valid save keeps job 1 and no Miner/Chef.
    fn jobless(&self) -> GameplayState {
        self.load(None).state()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn fill_inventory(state: &mut GameplayState) {
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
fn a_jobless_player_gets_one_item_and_its_entry_through_the_real_click() {
    let fixture = Fixture::new();
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
    let now_ms = water_click_at(1_000);
    let out = click(&mut game, node.guid, &who, now_ms);

    assert!(out.changed, "the pickup is a saved change");
    // Exactly one item, pinned by the click's time, and one the node's pool can yield.
    let item = water_item(now_ms);
    let gained = delta(&before, &game.state());
    assert_eq!(
        gained,
        vec![(item, 1)],
        "exactly one item definition changed"
    );
    assert!(
        pool_drops(WATER_NODE).contains(&(item as u32)),
        "item {item} is not in the pool table"
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
        .entry_for_item(item)
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
    assert!(removed(&out, node.guid), "the node was not hidden");
}

#[test]
fn a_second_node_with_the_same_item_grants_it_and_records_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.load(Some(fixture.jobless()));
    let first = node(SAME_ITEM_A);
    let second = node(SAME_ITEM_B);
    let nodes = shipped_nodes();
    let defined = defined_items();
    let first_at = (1_000..)
        .find(|now_ms| {
            nodes
                .collect_item(first.guid, PLAYER, *now_ms)
                .is_some_and(|item| defined.contains(&(item as i32)))
        })
        .expect("a defined first roll");
    let item = nodes
        .collect_item(first.guid, PLAYER, first_at)
        .expect("first roll");
    // The second click's time is chosen so the shared pool rolls the same item again.
    let second_at = (first_at + 1..)
        .find(|now_ms| nodes.collect_item(second.guid, PLAYER, *now_ms) == Some(item))
        .expect("the shared pool can repeat an item");

    let before = game.state();
    let first_out = click(&mut game, first.guid, &at(first.position), first_at);
    assert!(
        entry_packet(&first_out).is_some(),
        "the first find records its entry"
    );
    let entry = catalogue().entry_for_item(item as i32).expect("entry");
    assert!(collected(&game, entry));

    let after_first = game.state();
    let second_out = click(&mut game, second.guid, &at(second.position), second_at);
    assert_eq!(delta(&after_first, &game.state()), vec![(item as i32, 1)]);
    assert!(
        entry_packet(&second_out).is_none(),
        "a known entry was recorded twice"
    );
    assert!(collected(&game, entry));
    let ledger = game.state().progression_extras.unwrap().collections;
    assert_eq!(ledger.collected_count(), 1, "one entry was recorded");
    assert_eq!(delta(&before, &game.state()), vec![(item as i32, 2)]);
}

#[test]
fn the_node_hides_now_and_returns_after_its_pool_respawn() {
    let fixture = Fixture::new();
    let mut game = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    let first_at = water_click_at(1_000);
    let out = click(&mut game, node.guid, &who, first_at);
    assert!(removed(&out, node.guid), "the click hides the node");
    // Water pools respawn in 60 s.
    assert!(!added(
        &game.tick(&who, first_at + 59_999).unwrap(),
        node.guid
    ));
    let back = game.tick(&who, first_at + 60_000).unwrap();
    assert!(added(&back, node.guid), "the node did not grow back");

    // The returned node rolls afresh: the item is the one pinned by this click's time.
    let again_at = water_click_at(first_at + 60_100);
    let before = game.state();
    click(&mut game, node.guid, &who, again_at);
    assert_eq!(
        delta(&before, &game.state()),
        vec![(water_item(again_at), 1)]
    );
}

#[test]
fn every_click_rolls_afresh_from_the_pools_table() {
    let nodes = shipped_nodes();
    let spawn = busiest_pool_spawn();
    let guid = collect_guid(spawn);
    let table = pool_drops(spawn);
    let items: BTreeSet<u32> = (0..200u64)
        .map(|click| {
            let now_ms = 1_000 + click * 61_000;
            let item = nodes.collect_item(guid, PLAYER, now_ms).expect("a roll");
            assert_eq!(nodes.collect_item(guid, PLAYER, now_ms), Some(item));
            item
        })
        .collect();
    assert!(
        items.iter().all(|item| table.contains(item)),
        "a roll left the pool's table"
    );
    assert!(
        items.len() > 40,
        "200 clicks on one node of a {}-drop pool reached only {} items",
        table.len(),
        items.len()
    );
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
    let fixture = Fixture::new();
    let mut state = fixture.jobless();
    fill_inventory(&mut state);
    let mut game = fixture.load(Some(state));
    let who = at(node.position);
    view(&mut game, &who);
    let before = snapshot(&game);
    let now_ms = water_click_at(1_000);
    let out = raw_click(&mut game, node.guid, &who, now_ms);
    assert!(said(&out, "Your bag is full. Make room before gathering."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
    assert!(!collected(
        &game,
        catalogue().entry_for_item(water_item(now_ms)).unwrap()
    ));

    // A roll the running content does not define refuses with the generic line and keeps
    // the node.
    let defined = defined_items();
    let undefined_at = (1_000..)
        .find(|now_ms| !defined.contains(&water_item(*now_ms)))
        .expect("some roll lands on an undefined item");
    let mut game = fixture.load(Some(fixture.jobless()));
    view(&mut game, &who);
    let before = snapshot(&game);
    let out = raw_click(&mut game, node.guid, &who, undefined_at);
    assert!(said(&out, "You cannot gather that right now."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);

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
        let out = raw_click(&mut game, node.guid, &who, 1_000);
        assert!(said(&out, "Move closer to use that."));
        assert!(!out.changed);
        assert_eq!(snapshot(&game), before);
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
    let out = raw_click(&mut game, node.guid, &who, 1_100);
    assert!(said(&out, "Finish what you are doing first."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);

    // A second click on the hidden node gets the existing Gone text and no grant.
    let mut game = fixture.load(Some(fixture.jobless()));
    let who = at(node.position);
    let first_at = water_click_at(1_000);
    assert!(click(&mut game, node.guid, &who, first_at).changed);
    let before = snapshot(&game);
    let out = raw_click(&mut game, node.guid, &who, first_at + 1_000);
    assert!(said(&out, "That has already been gathered."));
    assert!(!out.changed);
    assert_eq!(snapshot(&game), before);
}

#[test]
fn the_actor_packets_carry_the_types_sparkle_and_the_water_companion() {
    let fixture = Fixture::new();
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
    assert!(companion(&packets, flowers.guid).is_none());

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
    let companion = companion(&packets, water.guid).expect("the water companion");
    assert_eq!(companion.model_id, 271, "invisible cube");
    assert_eq!(companion.composite_effect_id, 5742);
    assert_eq!(companion.is_interactable, 0);
    assert_eq!(
        companion.position,
        [water.position[0], water.position[1], water.position[2], 1.0]
    );

    // Chef mushrooms start sparkling too; their Type8 flow is unchanged (gathering_nodes).
    let mushroom = nodes
        .actors()
        .find(|actor| actor.name == "Mushrooms")
        .expect("a mushroom node")
        .clone();
    let packets = game.update_npc_visibility(&at(mushroom.position)).unwrap();
    assert_eq!(parsed(&packets, mushroom.guid).composite_effect_id, 5742);
    let ore = nodes
        .actors()
        .find(|actor| actor.name == "Copper Vein")
        .expect("a copper vein")
        .clone();
    let packets = game.update_npc_visibility(&at(ore.position)).unwrap();
    assert_eq!(parsed(&packets, ore.guid).composite_effect_id, 0);
}

fn parsed(packets: &[Vec<u8>], guid: u64) -> osfr_tools::startup_world::NpcSpawn {
    packets
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|spawn| spawn.actor_id == guid)
        .unwrap_or_else(|| panic!("no AddNpc for {guid:#x}"))
}

/// The tag-4 companion of a collection node, when it has one.
fn companion(packets: &[Vec<u8>], node_guid: u64) -> Option<osfr_tools::startup_world::NpcSpawn> {
    let companion = node_guid - (1 << 32);
    packets
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|spawn| spawn.actor_id == companion)
}

#[test]
fn relog_keeps_the_entry_and_shows_every_node_again() {
    let fixture = Fixture::new();
    let mut game = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    let now_ms = water_click_at(1_000);
    assert!(click(&mut game, node.guid, &who, now_ms).changed);
    let entry = catalogue().entry_for_item(water_item(now_ms)).unwrap();
    assert!(collected(&game, entry));

    let mut relog = fixture.load(Some(game.state()));
    assert!(collected(&relog, entry), "the saved ledger lost the entry");
    // Nodes are personal and never saved: every one is shown again.
    let packets = relog.update_npc_visibility(&who).unwrap();
    assert!(
        packets
            .iter()
            .any(|packet| { parse_add_npc(packet).is_ok_and(|spawn| spawn.actor_id == node.guid) })
    );
    assert!(click(&mut relog, node.guid, &who, water_click_at(2_000)).changed);
    assert!(collected(&relog, entry));
}

#[test]
fn a_pickup_is_personal_to_the_player() {
    let fixture = Fixture::new();
    let mut first = fixture.load(Some(fixture.jobless()));
    let mut second = fixture.load(Some(fixture.jobless()));
    let node = node(WATER_NODE);
    let who = at(node.position);
    let now_ms = water_click_at(1_000);
    assert!(click(&mut first, node.guid, &who, now_ms).changed);
    let out = click(&mut second, node.guid, &who, now_ms);
    assert!(out.changed, "the second player could not pick the node");
    assert!(entry_packet(&out).is_some(), "the second ledger was empty");
}

/// The gateway dispatch: 26/8 reaches the collect handler and the outcome is saved.
#[test]
fn the_gateway_click_route_grants_and_records() {
    let fixture = Fixture::new();
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    let node = node(WATER_NODE);
    let mut character: CharacterState = Gateway::reference_state(&reference).unwrap();
    character.zone_id = 0;
    character.position = node.position;
    let mut gateway =
        Gateway::load_gameplay(&reference, character, &fixture.dir, Some(fixture.jobless()))
            .unwrap();
    login(&mut gateway);
    tick_gateway(&mut gateway, 1_500);
    let click_at = water_click_at(2_000);
    let entry = catalogue().entry_for_item(water_item(click_at)).unwrap();
    let out = click_gateway(&mut gateway, node.guid, click_at);
    assert!(
        out.gameplay_changed,
        "the gateway did not persist the pickup"
    );
    let gameplay = gateway.snapshot().gameplay.unwrap();
    assert!(
        gameplay
            .progression_extras
            .is_some_and(|extras| extras.collections.is_collected(entry)),
        "the gateway route did not record the entry"
    );
    assert!(scroll(&out, entry), "the 38/10 did not reach the client");
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn inner(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

fn send(gateway: &mut Gateway, packet: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(packet), now_ms)
        .expect("gateway frame")
}

fn tick_gateway(gateway: &mut Gateway, now_ms: u64) -> GatewayOutcome {
    gateway.tick_at(now_ms).expect("tick")
}

fn login(gateway: &mut Gateway) -> Vec<GatewayOutcome> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if index == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    vec![
        gateway.handle_at(&bytes, 0).unwrap(),
        send(gateway, &13u16.to_le_bytes(), 1_000),
        send(gateway, &10u16.to_le_bytes(), 1_000),
    ]
}

fn click_gateway(gateway: &mut Gateway, guid: u64, now_ms: u64) -> GatewayOutcome {
    let request = NpcInteractRequest {
        target: guid,
        trailing_flag: false,
    };
    send(gateway, &request.encode(), now_ms)
}

fn scroll(outcome: &GatewayOutcome, entry: i32) -> bool {
    let expected = add_entry(entry);
    outcome
        .replies
        .iter()
        .any(|reply| inner(reply) == expected.as_slice())
        && outcome
            .replies
            .iter()
            .any(|reply| inner(reply).starts_with(&[35, 0, 3, 0]))
}
