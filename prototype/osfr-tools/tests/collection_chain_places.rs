//! Collections piece: chained sets, exploration spots, entry-only node rows and the reward
//! items the item table gained.
//!
//! - A finished set records its entry in the next set: the paid reward item (wiki: "The reward
//!   for this collection ... is used in the ... collection"), or the entry its captured Token
//!   row stands for (R, by name).
//! - Walking into an exploration spot (`collection_places.json`) records its entry once, through
//!   the real movement route `Gameplay::on_location`.
//! - A node row with no item (`collection_entry_id`, item 0) fills its entry through the real
//!   26/8 click and puts nothing in the bag.

#[path = "support/json.rs"]
mod json;
#[path = "support/pickup_fixture.rs"]
mod pickup_fixture;

use osfr_tools::{
    collection_catalogue::CollectionCatalogueContent, gameplay::Gameplay,
    harvest_nodes::collection_node_guid, npc_interaction::NpcInteractRequest,
};
use pickup_fixture::{PickupFixture, at, bundled, shipped_nodes};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path};

const FORKS: i32 = 33;
const WOODEN_SPOON_ENTRY: i32 = 205;
const STILLWATER_CROSSING: i32 = 263;
const CURSED_GRAVEYARD: i32 = 1928;
const STILLWATER_NEWBIE: i32 = 1932;
const NEWBIE_EXPLORER: i32 = 264;
const NEWBIE_EXPLORER_COMPLETE: i32 = 1901;

fn catalogue() -> CollectionCatalogueContent {
    CollectionCatalogueContent::load(&bundled())
        .expect("catalogue load")
        .expect("the catalogue file")
}

fn ledger(game: &Gameplay) -> Value {
    let extras = game.state().progression_extras.expect("extras");
    serde_json::to_value(&extras.collections).unwrap()
}

fn collected(game: &Gameplay, entry: i32) -> bool {
    game.state()
        .progression_extras
        .is_some_and(|extras| extras.collections.is_collected(entry))
}

fn completed(game: &Gameplay, collection: i32) -> bool {
    ledger(game)["completed"]
        .as_array()
        .unwrap()
        .contains(&json!(collection))
}

fn place(entry: i32) -> [f32; 3] {
    let file = json::read(&bundled().join("collection_places.json"));
    let row = file["places"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["entry_id"] == entry)
        .expect("a place row");
    serde_json::from_value(row["position"].clone()).unwrap()
}

fn with_ledger(fixture: &PickupFixture, collected: &[i32]) -> Gameplay {
    let mut state = fixture.load(None).state();
    let mut extras = state.progression_extras.take().unwrap_or_default();
    extras.collections =
        serde_json::from_value(json!({"collected": collected, "completed": []})).unwrap();
    state.progression_extras = Some(extras);
    fixture.load(Some(state))
}

#[test]
fn a_reward_item_and_a_token_each_name_the_next_sets_entry() {
    let catalogue = catalogue();
    // Forks pays the Wooden Spoon, which is an entry of Spoons.
    assert_eq!(catalogue.chain_entries(FORKS), [WOODEN_SPOON_ENTRY]);
    assert_eq!(
        catalogue.chain_entries(STILLWATER_CROSSING),
        [STILLWATER_NEWBIE]
    );
    assert_eq!(
        catalogue.chain_entries(NEWBIE_EXPLORER),
        [NEWBIE_EXPLORER_COMPLETE]
    );
    let mut tokens = 0;
    for collection in catalogue.collections() {
        let chain = catalogue.chain_entries(collection.id);
        for entry in chain {
            let (_, owner) = catalogue.entry(*entry).expect("a chained entry exists");
            assert_ne!(owner.id, collection.id, "a set never chains into itself");
        }
        assert_eq!(
            chain.iter().collect::<BTreeSet<_>>().len(),
            chain.len(),
            "collection {} lists a chained entry twice",
            collection.id
        );
        if !collection.reward.tokens.is_empty() {
            tokens += 1;
            assert!(
                !chain.is_empty(),
                "token set {} chains nowhere",
                collection.id
            );
        }
    }
    assert!(tokens >= 40, "the captured token rows are still read");
}

#[test]
fn every_reward_item_has_a_definition_in_the_shipped_item_tables() {
    let release = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release");
    let mut defined = BTreeSet::new();
    for file in ["item-table.json", "item-table-supplement.json"] {
        for row in json::read(&release.join(file))["rows"].as_array().unwrap() {
            defined.insert(row[0].as_i64().unwrap() as i32);
        }
    }
    // The Skeletal T-Rex (79446, captured as 12410) is defined by the mounts piece's supplement
    // row, so it may be absent here until both have merged; nothing else may be.
    let unpayable = catalogue().unpayable_reward_items(|id| defined.contains(&id));
    assert!(
        unpayable.iter().all(|row| *row == (313, 12410)),
        "a collection reward names an item no table defines: {unpayable:?}"
    );
    // With no table at all, the rows that only carry a captured id are among the unpayable.
    let none = catalogue().unpayable_reward_items(|_| false);
    assert!(none.contains(&(311, 38817)) && none.contains(&(312, 38816)));
}

#[test]
fn walking_into_a_spot_records_it_once_and_a_finished_set_chains_on() {
    let fixture = PickupFixture::new("collect-place");
    // Everything but the graveyard, and the two other Newbie Explorer entries.
    let mut game = with_ledger(&fixture, &[1927, 1929, 1930, 1931]);
    let spot = place(CURSED_GRAVEYARD);

    let far = at([spot[0] + 500.0, spot[1], spot[2]]);
    let out = game.on_location(&far, 0).expect("movement is served");
    assert!(!collected(&game, CURSED_GRAVEYARD));
    assert!(!out.replies.iter().any(|p| p.starts_with(&[38, 0, 10, 0])));

    let here = at([spot[0] + 3.0, spot[1], spot[2] - 3.0]);
    let out = game.on_location(&here, 1_000).expect("movement is served");
    assert!(out.changed, "the save must be written");
    let adds = out
        .replies
        .iter()
        .filter(|p| p.starts_with(&[38, 0, 10, 0]))
        .count();
    assert_eq!(
        adds, 3,
        "the spot, its Newbie entry and the Wilds Explorer entry"
    );
    assert!(collected(&game, CURSED_GRAVEYARD) && completed(&game, STILLWATER_CROSSING));
    assert!(collected(&game, STILLWATER_NEWBIE) && completed(&game, NEWBIE_EXPLORER));
    assert!(collected(&game, NEWBIE_EXPLORER_COMPLETE));

    // Standing there again does nothing, and a relog keeps it.
    let before = ledger(&game);
    let again = game.on_location(&here, 2_000).expect("movement is served");
    assert!(!again.replies.iter().any(|p| p.starts_with(&[38, 0, 10, 0])));
    assert_eq!(ledger(&game), before);
    let mut relog = fixture.load(Some(game.state()));
    assert!(collected(&relog, CURSED_GRAVEYARD) && completed(&relog, NEWBIE_EXPLORER));
    let again = relog.on_location(&here, 3_000).expect("movement is served");
    assert!(!again.replies.iter().any(|p| p.starts_with(&[38, 0, 10, 0])));
}

#[test]
fn every_spot_names_a_catalogue_entry_on_the_exploration_tab() {
    let catalogue = catalogue();
    let file = json::read(&bundled().join("collection_places.json"));
    let mut seen = BTreeSet::new();
    for row in file["places"].as_array().unwrap() {
        let entry = row["entry_id"].as_i64().unwrap() as i32;
        let (_, owner) = catalogue.entry(entry).expect("a place names a real entry");
        assert_eq!(
            owner.category_id, 1,
            "entry {entry} is not an exploration entry"
        );
        assert!(seen.insert(entry), "entry {entry} has two spots");
        assert!(row["source"].as_str().is_some_and(|s| !s.is_empty()));
    }
    for row in file["gaps"].as_array().unwrap() {
        let entry = row["entry_id"].as_i64().unwrap() as i32;
        assert!(catalogue.entry(entry).is_some() && !seen.contains(&entry));
    }
}

#[test]
fn an_entry_only_row_fills_its_entry_and_leaves_the_bag_alone() {
    // A placed node whose table holds a row with no item, and a seed that draws that row.
    let probe = shipped_nodes();
    let nodes = json::read(&bundled().join("collection_nodes.json"));
    let pool = nodes["pools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pool| {
            pool["drop_table"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["item_definition_id"] == 0)
        })
        .expect("a pool with an entry-only row");
    let spawn = nodes["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|spawn| spawn["pool_key"] == pool["key"])
        .map(|spawn| spawn["id"].as_u64().unwrap() as u32)
        .find(|id| probe.is_collect(collection_node_guid(*id)))
        .expect("a placed node of that pool");
    let guid = collection_node_guid(spawn);
    let (seed, entry) = (0..10_000u64)
        .find_map(|seed| {
            let mut nodes = shipped_nodes();
            nodes.seed_rolls(seed);
            match nodes.collect_drop(guid) {
                Some((0, Some(entry))) => Some((seed, entry)),
                _ => None,
            }
        })
        .expect("a seed that draws an entry-only row");
    let position = probe
        .actors()
        .find(|actor| actor.guid == guid)
        .unwrap()
        .position;

    let fixture = PickupFixture::new("collect-entry-only");
    let mut game = fixture.load(None);
    game.seed_collection_rolls(seed);
    let who = at(position);
    game.update_npc_visibility(&who)
        .expect("the view is served");
    let bag = game.state().inventory.items;
    let request = NpcInteractRequest {
        target: guid,
        trailing_flag: false,
    };
    let out = game
        .handle_in_world(&request.encode(), &who, 0)
        .expect("the 26/8 request is accepted");
    assert!(out.changed && collected(&game, entry));
    assert!(out.replies.iter().any(|p| p.starts_with(&[38, 0, 10, 0])));
    assert_eq!(
        game.state().inventory.items,
        bag,
        "no item for an entry-only row"
    );
    // The node is gone: a second click is refused and records nothing new.
    let before = ledger(&game);
    game.handle_in_world(&request.encode(), &who, 1)
        .expect("the 26/8 request is accepted");
    assert_eq!(ledger(&game), before);
}

#[test]
fn a_set_finished_before_chains_existed_pays_its_chain_on_the_next_movement() {
    // An older save: Stillwater Crossing finished (and the two other Newbie entries held), but
    // its Newbie Explorer entry was never recorded.
    let fixture = PickupFixture::new("collect-backfill");
    let mut state = fixture.load(None).state();
    let mut extras = state.progression_extras.take().unwrap_or_default();
    extras.collections = serde_json::from_value(json!({
        "collected": [1927, 1928, 1929, 1930, 1931],
        "completed": [STILLWATER_CROSSING],
    }))
    .unwrap();
    state.progression_extras = Some(extras);
    let mut game = fixture.load(Some(state));
    assert!(!collected(&game, STILLWATER_NEWBIE));

    // Anywhere in the world, far from every spot: one owed entry per movement.
    let who = at([5_000.0, 0.0, 5_000.0]);
    let first = game.on_location(&who, 0).expect("movement is served");
    assert!(first.changed && collected(&game, STILLWATER_NEWBIE));
    assert!(completed(&game, NEWBIE_EXPLORER) && collected(&game, NEWBIE_EXPLORER_COMPLETE));
    let before = ledger(&game);
    let again = game.on_location(&who, 1_000).expect("movement is served");
    assert!(!again.replies.iter().any(|p| p.starts_with(&[38, 0, 10, 0])));
    assert_eq!(ledger(&game), before, "the chain is paid once");
}

#[test]
fn an_entry_only_row_whose_entry_is_held_is_left_out_of_the_draw() {
    let nodes = json::read(&bundled().join("collection_nodes.json"));
    let probe = shipped_nodes();
    let guid = nodes["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|spawn| spawn["pool_key"] == "southern-wilds-robgoblin-junk")
        .map(|spawn| collection_node_guid(spawn["id"].as_u64().unwrap() as u32))
        .find(|guid| probe.is_collect(*guid))
        .expect("a placed junkpile");
    let mut entry_only = 0;
    for seed in 0..2_000u64 {
        let mut nodes = shipped_nodes();
        nodes.seed_rolls(seed);
        entry_only += usize::from(matches!(nodes.collect_drop(guid), Some((0, _))));
        let (item, _) = nodes
            .collect_drop_skipping(guid, |_| true)
            .expect("item rows remain");
        assert_ne!(item, 0, "seed {seed} drew a row that gives nothing");
    }
    assert!(entry_only > 0, "the pool no longer holds an entry-only row");
}
