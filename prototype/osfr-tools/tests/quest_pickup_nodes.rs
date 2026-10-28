//! T503: quest-only pickup nodes with a counted `collect_node` goal, proved end to end
//! by A Booming Problem (`3000361`, Heddy Heathcliff, 5 tin ore).
//!
//! Every test drives the production router: a 26/8 click through `Gameplay::handle_in_world`,
//! the split quest loader through `Fixture::game_as`, the offer/accept/turn-in packets of
//! `tests/support/npc_quest_chain.rs` (49/1, 49/2, 26/3/26/6, 49/13, 49/14) and the 49/9
//! objective update. The node content is `data/content/quest_nodes.json`, copied into the
//! private fixture so `GatheringNodes::load` reads it the way a session does.
//!
//! Client wire, unchanged (the pickup adds no packet):
//! - 26/8 interact: constructor `0x008b6bc0` (two caller words = u64 target, one flag byte),
//!   sender `0x0092efe0`; decoded by `decode_npc_interact_26_8`.
//! - 49/9 objective update: reader `0x00c7af60` reads quest, objective and current counters,
//!   then the progress float, a notification byte and one unresolved dword; encoded by
//!   `QuestMessage::encode_exact`.
//! - 35/2 add and 35/3 remove: the existing node visibility path (`NpcVisibility::refresh`).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, added, character_at_for, click_from, coins, collect, content_dir,
    content_position, counts, end_screen_reward, finish_quest, objective_update, offer,
    pending_turnin, pick, pickups, profile_xp, quest_status, release_end_screen, removed, said,
    saved, seed_active_profile, shipped_nodes, talk, zone_player,
};
use osfr_tools::{
    gameplay::{Gameplay, Outcome},
    harvest_nodes::{GatheringNodes, NodeActor, NodeChange},
    mining::MINER_PROFILE_ID,
    quest_engine::QuestStatus,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const HEDDY: u64 = 9_500_000_588;
const QUEST: u32 = 3_000_361;
const ADVENTURER: i32 = 1;
/// A Booming Problem's fork reward (R): 40 coins and the T202 collect rung of 25 XP.
const COINS: u32 = 40;
const XP: u32 = 25;
const END_TEXT: u32 = 400_436;
const RESPAWN_MS: u64 = 30_000;
const OTHER: u64 = CHARACTER + 1;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn fixture() -> Fixture {
    // The opening content is required by `Gameplay::load` whenever mining.json is
    // present (the busy case starts a real mining run), so this is `new`, not `minimal`.
    let fixture = Fixture::new();
    fixture.overlay_content(&content_dir());
    // This slice starts no Type8 harvesting, and the opening fixture's inventory has no
    // harvesting reward rows; drop the overlayed opt-in board instead of fabricating items.
    let harvesting = fixture.path().join("harvesting.json");
    if harvesting.is_file() {
        fs::remove_file(&harvesting).expect("drop the unused harvesting board");
    }
    fixture
}

fn node_named(name: &str) -> NodeActor {
    shipped_nodes()
        .actors()
        .find(|actor| actor.name == name)
        .unwrap_or_else(|| panic!("a {name} node is shipped"))
        .clone()
}

fn at(player: u64, npc: u64) -> osfr_tools::character_store::CharacterState {
    character_at_for(player, 0, content_position("seaside.json", npc))
}

/// Offer and accept A Booming Problem at Heddy.
fn accept_booming(game: &mut Gameplay, player: u64, now: &mut u64) {
    let heddy = at(player, HEDDY);
    offer(game, &heddy, HEDDY, QUEST, *now);
    accept(game, &heddy, QUEST, *now + 2);
    *now += 20;
}

fn tick(game: &mut Gameplay, now: u64) -> Outcome {
    game.tick(&character_at_for(CHARACTER, 0, [0.0, 0.0, 0.0]), now)
        .unwrap_or_else(|error| panic!("tick at {now}: {error}"))
}

fn mining_active(game: &Gameplay) -> bool {
    serde_json::to_value(game.state()).unwrap()["mining"]["active"].is_object()
}

fn turn_in(game: &mut Gameplay, player: u64, now: u64) {
    let heddy = at(player, HEDDY);
    let talked = talk(game, &heddy, HEDDY, now);
    assert!(talked.changed, "the final Talk advances the talk step");
    assert_eq!(pending_turnin(game), Some(QUEST));
    let end = release_end_screen(game, &heddy, QUEST, HEDDY, END_TEXT, now + 2);
    assert_eq!(end_screen_reward(&end), (COINS, XP));
    finish_quest(game, &heddy, QUEST, ADVENTURER, COINS, XP, now + 4);
}

#[test]
fn a_booming_problem_end_to_end() {
    let fixture = fixture();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    accept_booming(&mut game, CHARACTER, &mut now);
    assert_eq!(counts(&game, QUEST), [0, 0]);

    collect(&mut game, CHARACTER, QUEST, 1, "tin-ore", 5, 2, &mut now);
    turn_in(&mut game, CHARACTER, now);

    assert_eq!(coins(&game), coins_before + COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + XP);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}

#[test]
fn pickup_refused_when_not_needed() {
    let fixture = fixture();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    let nodes = pickups("tin-ore");
    let first = &nodes[0];

    // 1. No quest at all: one chat line, the node stays, the save is unchanged.
    let before = saved(&game);
    let refused = click_from(&mut game, CHARACTER, first.guid, 0, first.position, now);
    assert!(said(&refused, "You don't need that right now."));
    assert!(!removed(&refused, first.guid), "a refusal hid the node");
    assert_eq!(saved(&game), before, "the refusal changed the save");
    now += 10;

    // The refusal left the node usable: the same click succeeds once the quest is active.
    accept_booming(&mut game, CHARACTER, &mut now);
    let picked = pick(&mut game, CHARACTER, first, &mut now);
    assert!(removed(&picked, first.guid));
    assert_eq!(counts(&game, QUEST)[0], 1);

    // 2. The count is already 5 (the objective is full): a regrown node is refused.
    for (index, node) in nodes.iter().enumerate().skip(1) {
        let clicked = pick(&mut game, CHARACTER, node, &mut now);
        assert!(
            objective_update(&clicked, QUEST, 1, index as u32 + 1),
            "pickup {} was not counted",
            index + 1
        );
    }
    assert_eq!(counts(&game, QUEST), [5, 0]);
    let regrown = tick(&mut game, now + RESPAWN_MS);
    assert!(added(&regrown, first.guid), "the picked node regrows");
    let before = saved(&game);
    let full = click_from(
        &mut game,
        CHARACTER,
        first.guid,
        0,
        first.position,
        now + RESPAWN_MS + 10,
    );
    assert!(said(&full, "You don't need that right now."));
    assert!(!removed(&full, first.guid));
    assert_eq!(
        saved(&game),
        before,
        "the full-count refusal changed the save"
    );
    now += RESPAWN_MS + 20;

    // 3. The quest is completed: a shown node is refused the same way. The NotNeeded
    // line also proves the node reached admission as shown (Gone is checked first).
    turn_in(&mut game, CHARACTER, now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    let second = &nodes[1];
    let before = saved(&game);
    let refused = click_from(
        &mut game,
        CHARACTER,
        second.guid,
        0,
        second.position,
        now + 30 + RESPAWN_MS,
    );
    assert!(said(&refused, "You don't need that right now."));
    assert!(!removed(&refused, second.guid));
    assert_eq!(
        saved(&game),
        before,
        "the completed-quest refusal changed the save"
    );
}

#[test]
fn pickup_refused_out_of_range_or_busy() {
    let fixture = fixture();
    let mut state = fixture.game_as(CHARACTER, None).state();
    seed_active_profile(&mut state, MINER_PROFILE_ID, &[ADVENTURER]);
    let mut game = fixture.game_as(CHARACTER, Some(state));
    let mut now = 1_000;
    accept_booming(&mut game, CHARACTER, &mut now);
    let node = pickups("tin-ore").remove(0);

    // Beyond interact_range 12.
    let far = [node.position[0] + 30.0, node.position[1], node.position[2]];
    let before = saved(&game);
    let refused = click_from(&mut game, CHARACTER, node.guid, 0, far, now);
    assert!(said(&refused, "Move closer to use that."));
    assert!(!removed(&refused, node.guid));
    assert_eq!(
        saved(&game),
        before,
        "an out-of-range refusal changed the save"
    );

    // Outside the open world (zone 0).
    let refused = click_from(&mut game, CHARACTER, node.guid, 2, node.position, now + 10);
    assert!(said(&refused, "Move closer to use that."));
    assert!(!removed(&refused, node.guid));
    assert_eq!(
        saved(&game),
        before,
        "a wrong-zone refusal changed the save"
    );

    // While another activity runs: a mining run starts on a copper vein (the same
    // production click path) and closes the shared busy gate.
    let vein = node_named("Copper Vein");
    let started = click_from(&mut game, CHARACTER, vein.guid, 0, vein.position, now + 20);
    assert!(started.changed, "the copper vein starts its Type8 run");
    assert!(mining_active(&game), "the mining run is the busy activity");
    let before = saved(&game);
    let refused = click_from(&mut game, CHARACTER, node.guid, 0, node.position, now + 30);
    assert!(said(&refused, "Finish what you are doing first."));
    assert!(!removed(&refused, node.guid));
    assert_eq!(saved(&game), before, "a busy refusal changed the save");
    assert!(
        mining_active(&game),
        "the refusal left the activity running"
    );
}

#[test]
fn picked_node_regrows_after_respawn() {
    let fixture = fixture();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    accept_booming(&mut game, CHARACTER, &mut now);
    let node = pickups("tin-ore").remove(0);

    let picked_at = now + 1;
    let picked = pick(&mut game, CHARACTER, &node, &mut now);
    assert!(removed(&picked, node.guid));
    assert_eq!(counts(&game, QUEST)[0], 1);

    // A second click in the same moment is Gone, and changes nothing.
    let before = saved(&game);
    let again = click_from(&mut game, CHARACTER, node.guid, 0, node.position, now);
    assert!(said(&again, "already been gathered"));
    assert!(!removed(&again, node.guid));
    assert_eq!(saved(&game), before);

    // Not back one millisecond before the pool's respawn_seconds; back at it.
    let early = tick(&mut game, picked_at + RESPAWN_MS - 1);
    assert!(!added(&early, node.guid), "the node regrew too early");
    let back = tick(&mut game, picked_at + RESPAWN_MS);
    assert!(added(&back, node.guid), "the node regrows after 30 s");

    let second = pick(&mut game, CHARACTER, &node, &mut now);
    assert!(removed(&second, node.guid));
    assert!(objective_update(&second, QUEST, 1, 2));
    assert_eq!(counts(&game, QUEST), [2, 0]);
}

#[test]
fn count_survives_reconnect() {
    let fixture = fixture();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    accept_booming(&mut game, CHARACTER, &mut now);
    let nodes = pickups("tin-ore");
    for node in nodes.iter().take(3) {
        pick(&mut game, CHARACTER, node, &mut now);
    }
    assert_eq!(counts(&game, QUEST), [3, 0]);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [3, 0], "the count is saved progress");

    // Nodes are not saved: a fresh login sees every node, including one this
    // character already picked in its previous session.
    let picked_again = pick(&mut game, CHARACTER, &nodes[0], &mut now);
    assert!(removed(&picked_again, nodes[0].guid));
    assert_eq!(counts(&game, QUEST), [4, 0]);
    pick(&mut game, CHARACTER, &nodes[3], &mut now);
    assert_eq!(counts(&game, QUEST), [5, 0]);

    turn_in(&mut game, CHARACTER, now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}

#[test]
fn two_players_do_not_share_nodes() {
    let fixture = fixture();
    let mut a = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut b = zone_player(&fixture, OTHER, ADVENTURER);
    let mut now = 1_000;
    accept_booming(&mut a, CHARACTER, &mut now);
    accept_booming(&mut b, OTHER, &mut now);
    let node = pickups("tin-ore").remove(0);

    let b_before = saved(&b);
    let picked = pick(&mut a, CHARACTER, &node, &mut now);
    assert!(removed(&picked, node.guid));
    assert_eq!(counts(&a, QUEST), [1, 0]);
    assert_eq!(counts(&b, QUEST), [0, 0], "A's pickup credited B");
    assert_eq!(saved(&b), b_before, "A's pickup changed B");

    // The node is still shown for B: B's own click is admitted and counts.
    let b_clicked = click_from(&mut b, OTHER, node.guid, 0, node.position, now);
    assert!(removed(&b_clicked, node.guid));
    assert!(objective_update(&b_clicked, QUEST, 1, 1));
    assert_eq!(counts(&b, QUEST), [1, 0]);
    assert_eq!(counts(&a, QUEST), [1, 0], "B's pickup credited A");
}

/// A scratch content directory holding only `quest_nodes.json`.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-t503-quest-nodes-{}-{name}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("scratch content directory");
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, content: &Value) -> io::Result<()> {
        self.write_file("quest_nodes.json", content)
    }

    fn write_file(&self, file: &str, content: &Value) -> io::Result<()> {
        fs::write(
            self.0.join(file),
            serde_json::to_vec(content).expect("node JSON"),
        )
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn node_file(drop_table: Value, pool_key: &str) -> Value {
    json!({
        "schema_version": 1,
        "node_types": [{
            "key": "tin-ore",
            "name": "Tin Ore",
            "model_id": 611,
            "scale": 1.0,
            "composite_effect_id": 5386,
            "interact_range": 12,
            "cursor_id": 17,
            "placement_y_offset": 0.0,
            "drop_table": drop_table,
        }],
        "pools": [{
            "key": "fixture-pool",
            "zone_definition_id": 1,
            "node_type": "tin-ore",
            "max_active_nodes": 1,
            "respawn_seconds": 30,
        }],
        "spawns": [{
            "id": 1,
            "pool_key": pool_key,
            "zone_definition_id": 1,
            "position": [0.0, 0.0, 0.0],
            "heading": 0.0,
        }],
    })
}

fn load_error(scratch: &Scratch, content: &Value) -> String {
    scratch.write(content).expect("scratch quest_nodes.json");
    GatheringNodes::load(scratch.path())
        .err()
        .expect("the bad quest node file must be refused")
        .to_string()
}

/// A spawn id is unique per node file, not across files: the tag byte gives each file its
/// own guid space. Before the base's tag byte was cleared, mining (2) and quest (3) spawn 1
/// both OR-ed to the same guid and `GatheringNodes::load` refused world entry.
#[test]
fn same_spawn_id_in_the_mining_and_quest_files_stays_distinct() {
    let mut mining = node_file(json!([]), "fixture-pool");
    mining["node_types"][0]["key"] = json!("copper");
    mining["node_types"][0]["name"] = json!("Copper Vein");
    mining["node_types"][0]["drop_table"] = json!([{"item_definition_id": 2739, "weight": 100}]);
    mining["pools"][0]["node_type"] = json!("copper");
    mining["spawns"][0]["position"] = json!([5.0, 0.0, 0.0]);
    let dir = Scratch::new("tag-space");
    dir.write_file("mining_nodes.json", &mining)
        .expect("scratch mining_nodes.json");
    dir.write(&node_file(json!([]), "fixture-pool"))
        .expect("scratch quest_nodes.json");
    let nodes = GatheringNodes::load(dir.path())
        .expect("a mining spawn 1 and a quest spawn 1 must not collide")
        .expect("both files place a node");
    assert_eq!(nodes.actors().count(), 2);
    let guids: BTreeSet<u64> = nodes.actors().map(|actor| actor.guid).collect();
    assert_eq!(guids.len(), 2, "two files' spawn 1 share one guid");
    let quest = nodes
        .actors()
        .find(|actor| actor.name == "Tin Ore")
        .expect("the quest pickup is placed")
        .guid;
    let mining = nodes
        .actors()
        .find(|actor| actor.name == "Copper Vein")
        .expect("the mining node is placed")
        .guid;
    assert_eq!(
        nodes.pickup_kind(quest),
        Some("tin-ore"),
        "the quest spawn is the pickup"
    );
    assert_eq!(
        nodes.pickup_kind(mining),
        None,
        "the mining spawn is a gather site"
    );
}

/// A pick of a node that is already hidden is "no change": it must not push the node's
/// return past the deadline set by the pick that actually hid it.
#[test]
fn repeated_pick_does_not_extend_the_hide() {
    let dir = Scratch::new("repeat-pick");
    dir.write(&node_file(json!([]), "fixture-pool"))
        .expect("scratch quest_nodes.json");
    let mut nodes = GatheringNodes::load(dir.path())
        .expect("valid quest nodes load")
        .expect("one quest node is placed");
    let guid = nodes.actors().next().expect("placed node").guid;
    assert!(matches!(
        nodes.sync(0, false).as_slice(),
        [NodeChange::Show(_)]
    ));
    assert!(matches!(
        nodes.consume(guid, 1_000),
        Some(NodeChange::Hide {
            guid: hidden,
            companion: None
        }) if hidden == guid
    ));
    assert!(
        nodes.consume(guid, 20_000).is_none(),
        "a repeat pick reported a change"
    );
    assert!(
        nodes.sync(29_999, false).is_empty(),
        "the node regrew early"
    );
    let back = nodes.sync(31_000, false);
    assert!(
        matches!(back.as_slice(), [NodeChange::Show(actor)] if actor.guid == guid),
        "the repeat pick pushed the return past the pool's respawn time"
    );
}

#[test]
fn quest_nodes_load_rejects_bad_rows() {
    // Control: the same shape loads one placed pickup.
    let good = Scratch::new("good");
    good.write(&node_file(json!([]), "fixture-pool"))
        .expect("scratch quest_nodes.json");
    let nodes = GatheringNodes::load(good.path())
        .expect("valid quest nodes load")
        .expect("one quest node is placed");
    assert_eq!(
        nodes.pickup_kinds().into_iter().collect::<BTreeSet<_>>(),
        BTreeSet::from(["tin-ore"])
    );
    assert_eq!(nodes.actors().count(), 1);

    // A drop table would be a second, unaudited reward path.
    let drop = Scratch::new("drop");
    let error = load_error(
        &drop,
        &node_file(
            json!([{"item_definition_id": 11082, "weight": 100}]),
            "fixture-pool",
        ),
    );
    assert!(error.contains("drop table"), "{error}");

    // A spawn naming a pool the file does not define.
    let pool = Scratch::new("pool");
    let error = load_error(&pool, &node_file(json!([]), "missing-pool"));
    assert!(error.contains("unknown pool"), "{error}");

    // A pool naming an unknown node type.
    let kind = Scratch::new("kind");
    let mut content = node_file(json!([]), "fixture-pool");
    content["pools"][0]["node_type"] = json!("missing-node");
    let error = load_error(&kind, &content);
    assert!(error.contains("unknown node type"), "{error}");

    // The loader's 1 MiB file bound.
    let big = Scratch::new("big");
    fs::write(
        big.path().join("quest_nodes.json"),
        vec![b' '; (1 << 20) + 1],
    )
    .expect("scratch oversized file");
    let error = GatheringNodes::load(big.path())
        .err()
        .expect("an oversized quest node file must be refused")
        .to_string();
    assert!(error.contains("1 MiB"), "{error}");
}
