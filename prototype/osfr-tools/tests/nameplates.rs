//! Nameplates piece: AddNpc (35/2) `hide_nameplate` as the 2014 servers set it
//! (`src/nameplate.rs`, `data/content/nameplate_rules.json`).
//!
//! Every check reads the 35/2 the production path sends (reader `0x00921a10`, the switch is the
//! byte after the literal name): `NpcVisibility::load` for content rows, `apply_node_changes`
//! for nodes, and `Gameplay::update_npc_visibility` for coins, which keep their names (2014).
//! A hidden name keeps the actor clickable (`is_interactable`) and its 35/12 hover cursor.
//!
//! The coin test loads the private opening package (`support/pickup_fixture.rs`): run it on
//! this PC (`BUILD_SLOT_LOCAL=1`).
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
#[path = "support/pickup_fixture.rs"]
mod pickup_fixture;

use npc_visibility::{content_dir, seen_from};
use osfr_tools::{
    harvest_nodes::{NodeChange, apply_node_changes},
    nameplate::{self, Kind, Rules},
    startup_world::{NpcSpawn, parse_add_npc},
    world::NpcVisibility,
};
use pickup_fixture::{PickupFixture, at, shipped_nodes};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

const SNOWBERRY_PLANT: (u64, [f32; 3]) = (100_000_033_088, [-1548.1401, -31.54154, 354.50082]);
const RABBIT: (u64, [f32; 3]) = (100_000_033_100, [-1430.057, -31.778555, 217.45319]);
/// Seaside Rare Exploration coin 1946 (`collection_coins.json`).
const COIN: (u64, [f32; 3]) = (100_000_001_946, [-507.1435, -27.20111, -1201.367]);
const QUEEN_VALERIAN: (u64, [f32; 3]) = (9_500_000_519, [-1054.135, 4.629629, 305.7209]);
/// A Blackspore quest giver on a model every 2014 row hid (3295).
const LON_BLACKFOREST: (u64, [f32; 3]) = (9_500_000_038, [-2449.0, -31.7, 1188.0]);
/// A quest giver whose 2014 name was always hidden (the extra spawn that draws him).
const ROSCOE: u64 = 9_000_000_184;
/// Boddington, a placed Blackspore vendor (`npcs/`, `vendors.json`).
const BODDINGTON: (u64, [f32; 3]) = (9_500_000_007, [-2496.12, -23.96224, 1084.769]);
/// Cragara, the Seaside pet shop seller (`npc_spawns_extra.json`, content key 9500000569).
const CRAGARA_SEASIDE: [f32; 3] = [-631.06, 9.61, -1037.52];

fn spawn_of(spawns: &[NpcSpawn], guid: u64) -> &NpcSpawn {
    spawns
        .iter()
        .find(|npc| npc.actor_id == guid)
        .unwrap_or_else(|| panic!("{guid}: no 35/2 in the view"))
}

/// The 35/12 hover cursor id the view sends for `guid` (count, then guid, has-cursor, id, flag).
fn cursor_of(packets: &[Vec<u8>], guid: u64) -> Option<u8> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 12, 0]))
        .flat_map(|packet| packet[8..].chunks(11))
        .find(|entry| entry[..8] == guid.to_le_bytes())
        .map(|entry| entry[9])
}

#[test]
fn the_rules_file_loads_and_every_block_cites_its_source() {
    Rules::parse(nameplate::SHIPPED).expect("the shipped rules parse");
    let file: Value = serde_json::from_str(nameplate::SHIPPED).unwrap();
    let blocks: Vec<_> = file
        .as_object()
        .unwrap()
        .iter()
        .filter(|(key, value)| value.is_object() && *key != "provenance")
        .collect();
    assert_eq!(blocks.len(), 7);
    for (key, block) in &blocks {
        let source = block["source"].as_str().unwrap_or_default();
        assert!(!source.trim().is_empty(), "{key} has no source");
        let mut blank = file.clone();
        blank[key.as_str()]["source"] = Value::from(" ");
        assert!(
            Rules::parse(&blank.to_string()).is_err(),
            "{key} without a source loads"
        );
    }
    // The spec's 2014 evidence: Sanctuary@10f129e Npcs.json, 140 names and 85 models always hidden.
    assert_eq!(file["hidden_names"]["names"].as_array().unwrap().len(), 140);
    assert_eq!(
        file["hidden_models"]["model_ids"].as_array().unwrap().len(),
        85
    );
    let mut extra = file.clone();
    extra["surprise"] = Value::from(1);
    assert!(
        Rules::parse(&extra.to_string()).is_err(),
        "an unknown block loads"
    );
}

#[test]
fn plants_butterflies_and_rabbits_are_hidden_but_stay_clickable() {
    let table = NpcVisibility::load(&content_dir()).unwrap();
    for (guid, here) in [SNOWBERRY_PLANT, RABBIT] {
        let packets = table.clone().update(0, here).unwrap();
        let spawns: Vec<_> = packets
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .map(|packet| parse_add_npc(packet).unwrap())
            .collect();
        let npc = spawn_of(&spawns, guid);
        assert_eq!((npc.hide_nameplate, npc.is_interactable), (1, 1), "{guid}");
    }
    // The plant still answers a hover with its use cursor (`npc_presentation.json`, 17).
    let packets = table.clone().update(0, SNOWBERRY_PLANT.1).unwrap();
    assert_eq!(cursor_of(&packets, SNOWBERRY_PLANT.0), Some(17));

    // Quest pick-up nodes: the Butterflies and the Snowberry (`quest_nodes.json`).
    let nodes = shipped_nodes();
    for name in ["Butterflies", "Snowberry", "Rabbit"] {
        let node = nodes
            .actors()
            .find(|actor| actor.name == name)
            .unwrap_or_else(|| panic!("no {name} node"))
            .clone();
        let mut visibility = NpcVisibility::default();
        apply_node_changes(&mut visibility, vec![NodeChange::Show(node.clone())]);
        let spawns: Vec<_> = (visibility.update(0, node.position).unwrap().iter())
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .map(|packet| parse_add_npc(packet).unwrap())
            .collect();
        let npc = spawn_of(&spawns, node.guid);
        assert_eq!((npc.hide_nameplate, npc.is_interactable), (1, 1), "{name}");
    }
}

#[test]
fn a_coin_keeps_its_name_through_the_live_session_as_2014_did() {
    // The 2014 table hides none of its 121 Exploration Coin rows (lead, 2026-11-11).
    let fixture = PickupFixture::new("nameplate-coin");
    let mut game = fixture.load(None);
    let packets = game.update_npc_visibility(&at(COIN.1)).unwrap();
    let spawns: Vec<_> = packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .map(|packet| parse_add_npc(packet).unwrap())
        .collect();
    let coin = spawn_of(&spawns, COIN.0);
    assert_eq!((coin.hide_nameplate, coin.is_interactable), (0, 1));
    // The same row through the bare NPC table keeps the 2014 table's answer for its name.
    let bare = seen_from(&NpcVisibility::load(&content_dir()).unwrap(), COIN.1);
    assert_eq!(spawn_of(&bare, COIN.0).hide_nameplate, 0);
}

#[test]
fn queen_valerian_quest_givers_a_vendor_and_a_player_keep_their_names() {
    let table = NpcVisibility::load(&content_dir()).unwrap();
    for (guid, here) in [QUEEN_VALERIAN, LON_BLACKFOREST, BODDINGTON] {
        let npc = spawn_of(&seen_from(&table, here), guid).clone();
        assert_eq!(npc.hide_nameplate, 0, "{guid}");
    }
    let roscoe = table
        .teleport_targets(0)
        .into_iter()
        .find(|target| target.guid == ROSCOE)
        .expect("Roscoe is drawn");
    assert_eq!(
        spawn_of(&seen_from(&table, roscoe.position), ROSCOE).hide_nameplate,
        0
    );
    // Without the quest, the same model and name would be hidden.
    assert!(nameplate::hidden(3295, "Lon Blackforest", Kind::Npc, None));
    assert!(nameplate::hidden(184, "Roscoe", Kind::Npc, None));
    // A player rides AddPc (35/1), which this rule never writes; as a character it is never
    // hidden, even named like a critter and flagged.
    assert!(!nameplate::hidden(1, "Rabbit", Kind::Character, Some(true)));
    assert!(nameplate::hidden(1, "Rabbit", Kind::Npc, None));
    // A row's own flag wins over the rules, and a pick-up hides unless its row says otherwise.
    assert!(!nameplate::hidden(403, "Rabbit", Kind::Npc, Some(false)));
    assert!(nameplate::hidden(
        1,
        "Queen Valerian",
        Kind::Npc,
        Some(true)
    ));
    assert!(!nameplate::hidden(
        342,
        "Butterflies",
        Kind::Pickup,
        Some(false)
    ));
}

/// Every giver, and every talk or delivery NPC of an enabled quest, read straight from
/// `quests/*.json` (an `any_of` names its leaves).
fn quest_npcs() -> BTreeSet<u64> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(content_dir().join("quests")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let file: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        for quest in file["records"].as_array().unwrap() {
            ids.insert(quest["giver_npc_id"].as_u64().unwrap());
            if quest["enabled"].as_bool() == Some(false) {
                continue;
            }
            for goal in quest["objectives"].as_array().into_iter().flatten() {
                let leaves = goal["alternatives"]
                    .as_array()
                    .map_or(vec![goal], |alternatives| alternatives.iter().collect());
                for leaf in leaves {
                    let talk =
                        matches!(leaf["kind"].as_str(), Some("talk_to_npc" | "deliver_item"));
                    ids.extend(leaf["npc_id"].as_u64().filter(|_| talk));
                }
            }
        }
    }
    ids
}

#[test]
fn every_live_quests_giver_and_talk_targets_keep_their_names() {
    let quest_npcs = quest_npcs();
    let mut table = NpcVisibility::load(&content_dir()).unwrap();
    // Walk the open world from every drawn actor, keeping each 35/2 the walk is sent.
    let mut seen = BTreeMap::new();
    for target in table.teleport_targets(0) {
        for packet in table.update(0, target.position).unwrap() {
            if packet.starts_with(&[35, 0, 2, 0]) {
                let npc = parse_add_npc(&packet).unwrap();
                seen.insert(npc.actor_id, npc);
            }
        }
    }
    let mut checked = 0;
    for (guid, npc) in &seen {
        let key = table.content_key(*guid).unwrap_or(*guid);
        if quest_npcs.contains(&key) {
            assert_eq!(
                npc.hide_nameplate, 0,
                "quest NPC {key} (actor {guid}) is hidden"
            );
            checked += 1;
        }
    }
    let hidden = seen.values().filter(|npc| npc.hide_nameplate == 1).count();
    println!(
        "open world: {} actors drawn, {hidden} hidden, {checked} quest NPCs all named",
        seen.len()
    );
    assert!(checked > 500, "only {checked} quest NPCs drawn");
}

/// The 35/10 overhead image keys the view sends for `guid` (entry: guid, compact byte, kind,
/// state, then the image key, `npc_notifications.rs`).
fn icons_of(packets: &[Vec<u8>], guid: u64) -> Vec<i32> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 10, 0]))
        .flat_map(|packet| packet[8..].chunks(39))
        .filter(|entry| entry.len() == 39 && entry[..8] == guid.to_le_bytes())
        .map(|entry| i32::from_le_bytes(entry[17..21].try_into().unwrap()))
        .collect()
}

fn view(visibility: &NpcVisibility, at: [f32; 3]) -> Vec<Vec<u8>> {
    visibility.clone().update(0, at).unwrap()
}

/// rc-0136 F4 (q-mining, the vendor and SC tags): an actor that carries an overhead icon keeps
/// its nameplate shown, or the client drops the icon with the name. A copper vein shows its
/// pick (image 340) and is sent shown; a plant with no icon still hides its name. The Striped
/// and Armored T-Rex displays now get the SC tag (171) every other SC display has, and each
/// vendor icon's NPC is sent shown.
#[test]
fn actors_with_an_overhead_icon_keep_their_nameplate() {
    // Every placed vein: the W171 copper veins beside Therin (model 259, which had no pick
    // before rc-0136), the Snowhill world veins and the rich veins.
    let nodes = shipped_nodes();
    let veins: Vec<_> = nodes
        .actors()
        .filter(|actor| actor.name.ends_with(" Vein"))
        .cloned()
        .collect();
    assert!(veins.iter().any(|vein| vein.model_id == 259), "the Therin veins are placed");
    for vein in veins {
        let mut visibility = NpcVisibility::default();
        apply_node_changes(&mut visibility, vec![NodeChange::Show(vein.clone())]);
        let packets = view(&visibility, vein.position);
        let spawns: Vec<_> = (packets.iter())
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .map(|packet| parse_add_npc(packet).unwrap())
            .collect();
        let drawn = spawn_of(&spawns, vein.guid);
        let shown = (drawn.hide_nameplate, drawn.is_interactable);
        assert_eq!(shown, (0, 1), "{}: the vein's icon needs its plate", vein.guid);
        assert_eq!(icons_of(&packets, vein.guid), [340], "{} shows its pick", vein.guid);
    }
    // A Snowberry Plant has no icon: its name stays hidden (2014).
    let table = NpcVisibility::load(&content_dir()).unwrap();
    let plant = view(&table, SNOWBERRY_PLANT.1);
    assert!(icons_of(&plant, SNOWBERRY_PLANT.0).is_empty());
    let plant_spawns: Vec<_> = (plant.iter())
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .map(|packet| parse_add_npc(packet).unwrap())
        .collect();
    assert_eq!(spawn_of(&plant_spawns, SNOWBERRY_PLANT.0).hide_nameplate, 1);
    // SC displays and the pet shop seller: each has its tag and is sent shown.
    for (guid, at, image) in [
        (100_000_004_661, [-908.5041, -34.22184, -380.331], 171),
        (100_000_004_662, [-894.3927, -33.65656, -385.3986], 171),
        (100_000_001_738, [-586.6817, 19.42487, -1049.762], 171),
        (9_500_000_569, CRAGARA_SEASIDE, 381),
    ] {
        let packets = view(&table, at);
        assert_eq!(icons_of(&packets, guid), [image], "{guid}");
        let spawns: Vec<_> = (packets.iter())
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .map(|packet| parse_add_npc(packet).unwrap())
            .collect();
        assert_eq!(spawn_of(&spawns, guid).hide_nameplate, 0, "{guid}");
    }
}
