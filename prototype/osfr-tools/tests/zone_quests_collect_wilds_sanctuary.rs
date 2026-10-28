//! T539: three collect-node fork quests imported as data only:
//! Tools for Smashing (`3000342`), Box O' Rocks (`3000343`) in the Forest Troll Village
//! quarry line, and Hasti's Homework Heist (`3000302`) in Sanctuary.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`: the
//! split quest loader through `Fixture::game_as`, the 26/8 -> 26/10 -> 26/6 offer
//! (49/1), the 49/2 accept, the 26/8 pickup click through `Gameplay::handle_in_world`,
//! the 49/8/49/9/49/10 objective updates, and the 26/6 -> 49/13 -> 49/14 turn-in. The
//! node tables are `data/content/quest_nodes.json`, read by the production
//! `GatheringNodes::load` through the fixture.
//!
//! Client wire, unchanged (the pickup adds no packet):
//! - 26/8 interact constructor `0x008b6bc0` (two caller words = u64 target, one flag
//!   byte), sender `0x0092efe0`; decoded by `decode_npc_interact_26_8`.
//! - 49/9 `QuestObjectiveUpdatePacket` reader `0x00c7af60`: quest, objective, current
//!   counter, progress float, a byte and one dword; encoded by
//!   `QuestMessage::encode_exact`.
//! - 49/8 `ObjectiveActivated` reader `0x00c7aeb0` and 49/10 `GoalComplete` reader
//!   `0x00c7b070`; 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (reward bundle
//!   `0x008e7930`), already pinned by T202/T498/T503.
//! - 35/2 add and 35/3 remove: the existing node visibility path
//!   (`NpcVisibility::refresh`).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, accept, assert_no_offer,
    assert_packet_changes_nothing, character_at_for, click_from, coins, collect, content_dir,
    counts, npc_interact, objective_activated, objective_complete, objective_update, offer, pick,
    pickups, profile_xp, quest_end_reply, quest_status, removed, said, saved, send,
    zone_player_with_quests,
};
use osfr_tools::{
    gameplay::Gameplay,
    harvest_nodes::{HarvestNodesContent, NodeActor},
    quest_engine::QuestStatus,
};

const BURGLZ: u64 = 9_500_001_007;
const VERGOSH: u64 = 9_500_000_955;
const NOMI: u64 = 9_500_000_509;
const QUARRY: u32 = 3_000_341;
const LITTLE_BROTHER: u32 = 3_000_301;
const TOOLS: u32 = 3_000_342;
const BOX: u32 = 3_000_343;
const HOMEWORK: u32 = 3_000_302;
const OTHER: u64 = CHARACTER + 1;

/// Every NPC this suite talks to lives in its zone file and spawns as its content key.
const WILDS: ZoneQuests = ZoneQuests {
    default_file: "wilds.json",
    files: &[],
    runtime: &[],
};
const SANCTUARY: ZoneQuests = ZoneQuests {
    default_file: "sanctuary.json",
    files: &[],
    runtime: &[],
};

const TOOLS_QUEST: ZoneQuest = ZoneQuest {
    id: TOOLS,
    giver: BURGLZ,
    card: [70_218, 70_299, 70_219],
    steps: &[],
    turnin: BURGLZ,
    end_text: 70_196,
    coins: 30,
    xp: 25,
    profile_xp: 25,
};
const BOX_QUEST: ZoneQuest = ZoneQuest {
    id: BOX,
    giver: BURGLZ,
    card: [70_201, 70_505, 70_202],
    steps: &[],
    turnin: VERGOSH,
    end_text: 70_202,
    coins: 30,
    xp: 30,
    profile_xp: 30,
};
const HOMEWORK_QUEST: ZoneQuest = ZoneQuest {
    id: HOMEWORK,
    giver: NOMI,
    card: [74_437, 74_436, 74_456],
    steps: &[],
    turnin: NOMI,
    end_text: 72_906,
    coins: 40,
    xp: 25,
    profile_xp: 25,
};

/// A private copy of the shipped node tables plus the NPC and quest catalogue bindings,
/// without the opening campaign (no opening position can pay anything in these tests).
fn fixture() -> Fixture {
    let fixture = Fixture::minimal(CHARACTER);
    fixture.overlay_content(&content_dir());
    fixture
}

/// A player with the prerequisite quests already completed, so the offer gate is the
/// production prereq rule rather than a test-only bypass.
fn player(fixture: &Fixture, guid: u64, completed: &[(u32, &[u32])]) -> Gameplay {
    zone_player_with_quests(fixture, guid, ADVENTURER, &[ADVENTURER], completed)
}

/// Click a node the open quest does not need: one chat line, no packet, and not one
/// persisted byte changes. The location and visibility updates run before the snapshot,
/// so this compares the refusal itself.
fn refused_without_change(game: &mut Gameplay, player: u64, node: &NodeActor, now: u64) {
    let character = character_at_for(player, 0, node.position);
    game.on_location(&character, now)
        .expect("the location report is accepted");
    game.update_npc_visibility(&character)
        .expect("the NPC view update is served");
    let before = saved(game);
    let refused = send(game, &character, &npc_interact(node.guid), now + 1);
    assert!(said(&refused, "You don't need that right now."));
    assert!(!refused.changed, "the refusal changed gameplay");
    assert!(!removed(&refused, node.guid), "the refusal hid the node");
    assert_eq!(saved(game), before, "the refusal changed the save");
}

#[test]
fn follow_ups_wait_for_their_prerequisite() {
    let fixture = fixture();

    // Burglz offers neither quarry follow-up before Question at the Quarry is done.
    let mut fresh = player(&fixture, CHARACTER, &[]);
    let burglz = WILDS.at(CHARACTER, BURGLZ);
    let mut now = 1_000;
    assert_no_offer(
        &mut fresh,
        &burglz,
        BURGLZ,
        &mut now,
        "Burglz before the Quarry",
    );

    // Nomi offers Little Brother now; Hasti's Homework waits until it is done.
    let nomi = SANCTUARY.at(CHARACTER, NOMI);
    now += 10;
    offer(&mut fresh, &nomi, NOMI, LITTLE_BROTHER, now);
    now += 3;
    accept(&mut fresh, &nomi, LITTLE_BROTHER, now);
    now += 10;
    assert_no_offer(
        &mut fresh,
        &nomi,
        NOMI,
        &mut now,
        "Hasti's Homework before Little Brother is done",
    );

    // With the Quarry done, Burglz offers exactly Tools for Smashing; Box O' Rocks waits
    // until Tools is done.
    let mut quarry_done = player(&fixture, CHARACTER, &[(QUARRY, &[1])]);
    let mut now = 1_000;
    WILDS.accept(&mut quarry_done, CHARACTER, &TOOLS_QUEST, &mut now);
    assert_eq!(quest_status(&quarry_done, TOOLS), QuestStatus::Active);
    assert_no_offer(
        &mut quarry_done,
        &WILDS.at(CHARACTER, BURGLZ),
        BURGLZ,
        &mut now,
        "Box O' Rocks before Tools is done",
    );
}

#[test]
fn quarry_tools_then_box_o_rocks() {
    let fixture = fixture();
    let mut game = player(&fixture, CHARACTER, &[(QUARRY, &[1])]);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;

    WILDS.accept(&mut game, CHARACTER, &TOOLS_QUEST, &mut now);
    assert_eq!(counts(&game, TOOLS), [0, 0, 0]);
    // An early turn-in is refused and changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &WILDS.at(CHARACTER, BURGLZ),
        &quest_end_reply(TOOLS),
        now,
        "an early 49/14",
    );
    collect(
        &mut game,
        CHARACTER,
        TOOLS,
        1,
        "smashing-hammers",
        5,
        2,
        &mut now,
    );
    collect(
        &mut game,
        CHARACTER,
        TOOLS,
        2,
        "smashing-chisels",
        5,
        3,
        &mut now,
    );
    WILDS.turn_in(&mut game, CHARACTER, &TOOLS_QUEST, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, TOOLS), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 30);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 25);

    // Box O' Rocks follows at the same giver and is handed in to Vergosh.
    WILDS.accept(&mut game, CHARACTER, &BOX_QUEST, &mut now);
    assert_eq!(counts(&game, BOX), [0, 0]);
    collect(&mut game, CHARACTER, BOX, 1, "box-o-rocks", 3, 2, &mut now);
    WILDS.turn_in(&mut game, CHARACTER, &BOX_QUEST, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, BOX), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 60);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 55);
}

#[test]
fn hasti_homework_heist() {
    let fixture = fixture();
    let mut game = player(&fixture, CHARACTER, &[(LITTLE_BROTHER, &[1, 1])]);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;

    SANCTUARY.accept(&mut game, CHARACTER, &HOMEWORK_QUEST, &mut now);
    assert_eq!(counts(&game, HOMEWORK), [0, 0]);
    collect(
        &mut game,
        CHARACTER,
        HOMEWORK,
        1,
        "hastis-homework",
        8,
        2,
        &mut now,
    );
    SANCTUARY.turn_in(&mut game, CHARACTER, &HOMEWORK_QUEST, ADVENTURER, &mut now);

    assert_eq!(quest_status(&game, HOMEWORK), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 40);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 25);
}

#[test]
fn pickups_count_only_for_their_own_type() {
    let fixture = fixture();
    let mut game = player(&fixture, CHARACTER, &[(QUARRY, &[1])]);
    let mut now = 1_000;
    WILDS.accept(&mut game, CHARACTER, &TOOLS_QUEST, &mut now);

    // Only the hammer objective is open; a chisel click is refused and changes nothing.
    let chisel = pickups("smashing-chisels").remove(0);
    refused_without_change(&mut game, CHARACTER, &chisel, now + 10);
    assert_eq!(counts(&game, TOOLS), [0, 0, 0]);
    now += 20;

    // The same state still accepts a hammer: the chisel click consumed no attempt.
    let hammer = pickups("smashing-hammers").remove(0);
    let clicked = pick(&mut game, CHARACTER, &hammer, &mut now);
    assert!(removed(&clicked, hammer.guid));
    assert!(objective_update(&clicked, TOOLS, 1, 1));
    assert_eq!(counts(&game, TOOLS), [1, 0, 0]);
}

#[test]
fn pickups_refused_before_the_quest() {
    let fixture = fixture();
    let mut game = player(&fixture, CHARACTER, &[(QUARRY, &[1])]);
    let hammer = pickups("smashing-hammers").remove(0);
    refused_without_change(&mut game, CHARACTER, &hammer, 1_000);
}

#[test]
fn counts_survive_reconnect() {
    let fixture = fixture();
    let mut game = player(&fixture, CHARACTER, &[(QUARRY, &[1])]);
    let mut now = 1_000;
    WILDS.accept(&mut game, CHARACTER, &TOOLS_QUEST, &mut now);
    let hammers = pickups("smashing-hammers");
    for node in hammers.iter().take(3) {
        let clicked = pick(&mut game, CHARACTER, node, &mut now);
        assert!(removed(&clicked, node.guid));
    }
    assert_eq!(counts(&game, TOOLS), [3, 0, 0]);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, TOOLS), QuestStatus::Active);
    assert_eq!(
        counts(&game, TOOLS),
        [3, 0, 0],
        "the count is saved progress"
    );

    // Nodes are not saved: the reloaded session shows every node again, including one
    // this character picked before.
    let first = &hammers[0];
    let clicked = click_from(&mut game, CHARACTER, first.guid, 0, first.position, now);
    assert!(removed(&clicked, first.guid));
    assert!(objective_update(&clicked, TOOLS, 1, 4));
    assert_eq!(counts(&game, TOOLS), [4, 0, 0]);
    now += 10;

    // Finish the hammer step at the last unopened spawn, then the chisel step.
    let fifth = &hammers[4];
    let clicked = pick(&mut game, CHARACTER, fifth, &mut now);
    assert!(objective_update(&clicked, TOOLS, 1, 5));
    assert!(objective_complete(&clicked, TOOLS, 1));
    assert!(objective_activated(&clicked, TOOLS, 2));
    collect(
        &mut game,
        CHARACTER,
        TOOLS,
        2,
        "smashing-chisels",
        5,
        3,
        &mut now,
    );
    WILDS.turn_in(&mut game, CHARACTER, &TOOLS_QUEST, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, TOOLS), QuestStatus::Completed);
}

#[test]
fn two_players_are_independent() {
    let fixture = fixture();
    let mut a = player(&fixture, CHARACTER, &[(QUARRY, &[1])]);
    let mut b = player(&fixture, OTHER, &[(QUARRY, &[1])]);
    let mut now = 1_000;
    WILDS.accept(&mut a, CHARACTER, &TOOLS_QUEST, &mut now);
    WILDS.accept(&mut b, OTHER, &TOOLS_QUEST, &mut now);
    let node = pickups("smashing-hammers").remove(0);

    let b_before = saved(&b);
    let clicked = pick(&mut a, CHARACTER, &node, &mut now);
    assert!(removed(&clicked, node.guid));
    assert_eq!(counts(&a, TOOLS), [1, 0, 0]);
    assert_eq!(counts(&b, TOOLS), [0, 0, 0], "A's pickup credited B");
    assert_eq!(saved(&b), b_before, "A's pickup changed B");

    // The node is still shown for B: B's own click is admitted and counts for B only.
    let b_clicked = click_from(&mut b, OTHER, node.guid, 0, node.position, now);
    assert!(removed(&b_clicked, node.guid));
    assert!(objective_update(&b_clicked, TOOLS, 1, 1));
    assert_eq!(counts(&b, TOOLS), [1, 0, 0]);
    assert_eq!(counts(&a, TOOLS), [1, 0, 0], "B's pickup credited A");
}

/// The four T539 node types and their pools are pinned against the client's prop table:
/// `Resources/Models.txt` maps 197 to `sh_shovel_01.adr`, 196 to `sh_pick_01.adr`, 84 to
/// `sh_crate_01.adr` and 584 to `sg_papers_01.adr`, matching locale 70221/70227
/// ("shovels"/"picks"). A transposed fork row would spawn the wrong prop and pass every
/// other test.
#[test]
fn node_types_and_pools_pin_the_client_props() {
    let content = HarvestNodesContent::load_from(content_dir().join("quest_nodes.json"))
        .expect("the shipped quest node table loads");
    let find = |key: &str| {
        content
            .node_types
            .iter()
            .find(|node_type| node_type.key == key)
            .unwrap_or_else(|| panic!("quest node type {key}"))
    };
    for (key, name, model_id) in [
        ("smashing-hammers", "Shovel", 197),
        ("smashing-chisels", "Pick", 196),
        ("box-o-rocks", "Rock", 84),
        ("hastis-homework", "Hasti's Homework", 584),
    ] {
        let node_type = find(key);
        assert_eq!(node_type.name, name, "{key} name");
        assert_eq!(node_type.model_id, model_id, "{key} model");
        assert_eq!(node_type.scale, 1.0, "{key} scale");
        assert_eq!(node_type.interact_range, 12, "{key} interact range");
        assert_eq!(node_type.composite_effect_id, 5386, "{key} effect");
        assert_eq!(node_type.cursor_id, 17, "{key} cursor");
        assert_eq!(node_type.placement_y_offset, 0.0, "{key} y offset");
        assert!(node_type.drop_table.is_empty(), "{key} grants an item");
    }

    for (key, node_type, max_active_nodes, spawns) in [
        ("briarwood-smashing-hammers", "smashing-hammers", 5, 5),
        ("briarwood-smashing-chisels", "smashing-chisels", 5, 5),
        ("briarwood-box-o-rocks", "box-o-rocks", 3, 3),
        ("briarwood-hastis-homework", "hastis-homework", 8, 8),
    ] {
        let pool = content
            .pools
            .iter()
            .find(|pool| pool.key == key)
            .unwrap_or_else(|| panic!("quest node pool {key}"));
        assert_eq!(pool.node_type, node_type, "{key} type");
        assert_eq!(pool.max_active_nodes, max_active_nodes, "{key} cap");
        assert_eq!(pool.respawn_seconds, 30, "{key} respawn");
        assert_eq!(
            content
                .spawns
                .iter()
                .filter(|spawn| spawn.pool_key == key)
                .count(),
            spawns,
            "{key} spawns"
        );
    }
}
