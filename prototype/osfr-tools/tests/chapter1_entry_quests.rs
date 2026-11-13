//! L1195: the chapter 1 entry quests that send the player to Queen Valerian end with the same
//! palace tail (find the Royal Palace, speak to the palace guard, speak to Quills) and turn in
//! at Quills; Take a Number then runs from Quills through Pux and the Chugawug to the Queen.
//!
//! Sources (opus/L1195-chapter1-entry.md): the 2014 client blocks (390297, 390299, 391023,
//! 140647-140685), the live wiki pages and video 8Mo5c_FKQJg (2011). Every route is driven
//! through the production in-world router (`Gameplay::handle_in_world`, `on_location`); spots
//! and ids come from the shipped data.

#[allow(dead_code)] // Only `recruit_spawn`: a content key's drawn spawn (guid, spot).
#[path = "support/archer_entry.rs"]
mod archer_entry;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use archer_entry::recruit_spawn;
use npc_quest_chain::*;
use osfr_tools::{character_store::CharacterState, gameplay::Gameplay, quest_engine::QuestStatus};

const QUILLS: u64 = 9_500_000_520;
const PALACE_GUARD: u64 = 9_500_003_915;
const PUX: u64 = 9_500_000_517;
const BERTABOO: u64 = 9_500_004_865;
const QUEEN: u64 = 9_500_000_519;
const TAKE_A_NUMBER: u32 = 3_000_028;
const ADVENTURER: i32 = 1;

/// Each entry quest this item switched on or built: id, giver, the npcs file the giver stands
/// in. Bravery is in Demand (3955536) already had the tail; its giver Sorin is the Miner trainer
/// and its intro follows his mine-cave work, so an opening character is not offered it here.
const ENTRY: [(u32, u64, &str); 3] = [
    (3_000_025, 9_000_000_450, "blackspore_swamp.json"), // Sal's Suggestion
    (3_000_027, 9_000_000_477, "wilds.json"),            // Sharing Your Talents
    (3_000_062, 9_500_000_658, "shrouded_glade.json"),   // Druidic Direction
];

fn record(quest: u32) -> serde_json::Value {
    for entry in std::fs::read_dir(content_dir().join("quests")).unwrap() {
        let file = read_json(&entry.unwrap().path());
        if let Some(row) = file["records"]
            .as_array()
            .and_then(|rows| rows.iter().find(|row| row["id"] == quest))
        {
            return row.clone();
        }
    }
    panic!("quest {quest} is not shipped")
}

fn at(guid: u64, file: &str, npc: u64) -> CharacterState {
    character_at_for(guid, 0, content_position(file, npc))
}

/// Talk to a quest NPC that is bound to a drawn spawn (the palace guard, Bertaboo): the client
/// clicks the spawn's runtime guid, standing at the spawn.
fn talk_bound(game: &mut Gameplay, guid: u64, content_key: u64, now: u64) -> bool {
    let (runtime, spot) = recruit_spawn(content_key);
    talk(game, &character_at_for(guid, 0, spot), runtime, now).changed
}

/// Walk an accepted entry quest's palace tail and turn it in at Quills.
fn walk_tail_and_turn_in(game: &mut Gameplay, guid: u64, quest: u32, now: &mut u64) {
    let row = record(quest);
    let goals = row["objectives"].as_array().unwrap();
    let visit = goals.iter().find(|goal| goal["kind"] == "visit_position").expect("a palace visit");
    let palace: [f32; 3] = serde_json::from_value(visit["position"].clone()).unwrap();
    *now += 10;
    let visited = game.on_location(&character_at_for(guid, 0, palace), *now).unwrap();
    assert!(visited.changed, "quest {quest}: reaching the palace advances the visit");
    *now += 10;
    assert!(talk_bound(game, guid, PALACE_GUARD, *now), "quest {quest}: the palace guard");
    *now += 10;
    let quills = at(guid, "sanctuary.json", QUILLS);
    assert!(talk(game, &quills, QUILLS, *now).changed, "quest {quest}: Quills");
    assert!(counts(game, quest).iter().all(|&count| count == 1), "quest {quest}: every goal done");
    *now += 10;
    let end = u32::try_from(row["end_text_string_id"].as_u64().expect("an end line")).unwrap();
    release_end_screen(game, &quills, quest, QUILLS, end, *now);
    *now += 10;
    let coins = u32::try_from(row["rewards"][0]["coins"].as_u64().unwrap()).unwrap();
    finish_quest(game, &quills, quest, ADVENTURER, coins, 0, *now);
    assert_eq!(quest_status(game, quest), QuestStatus::Completed);
}

#[test]
fn every_entry_quest_ends_at_quills_after_the_palace_guard() {
    let fixture = Fixture::new();
    for (index, (quest, giver, file)) in ENTRY.into_iter().enumerate() {
        let guid = 500 + index as u64;
        let mut game = fixture.game_as(guid, None);
        let mut now = 1_000;
        let start = at(guid, file, giver);
        offer(&mut game, &start, giver, quest, now);
        accept(&mut game, &start, quest, now + 10);
        now += 20;
        walk_tail_and_turn_in(&mut game, guid, quest, &mut now);
    }
}

#[test]
fn take_a_number_follows_an_entry_quest_and_ends_with_the_queen() {
    let fixture = Fixture::new();
    let guid = 600;
    let mut game = fixture.game_as(guid, None);
    let mut now = 1_000;
    let quills = at(guid, "sanctuary.json", QUILLS);

    // Before any entry quest, Quills has nothing to offer for the Queen.
    assert_no_offer(&mut game, &quills, QUILLS, &mut now, "Take a Number before an entry quest");

    // Sharing Your Talents ends at Quills; then Quills offers Take a Number.
    let (quest, giver, file) = ENTRY[1];
    let start = at(guid, file, giver);
    offer(&mut game, &start, giver, quest, now);
    accept(&mut game, &start, quest, now + 10);
    now += 20;
    walk_tail_and_turn_in(&mut game, guid, quest, &mut now);
    now += 10;
    offer(&mut game, &quills, QUILLS, TAKE_A_NUMBER, now);
    accept(&mut game, &quills, TAKE_A_NUMBER, now + 10);
    now += 20;

    // Pux, the Chugawug by Jacque's cafe (bound to her drawn spawn), Pux again, Quills, the Queen.
    for (step, npc) in [PUX, BERTABOO, PUX, QUILLS, QUEEN].into_iter().enumerate() {
        now += 10;
        let changed = if npc == BERTABOO {
            talk_bound(&mut game, guid, npc, now)
        } else {
            talk(&mut game, &at(guid, "sanctuary.json", npc), npc, now).changed
        };
        assert!(changed, "Take a Number step {}", step + 1);
        assert_eq!(counts(&game, TAKE_A_NUMBER)[step], 1, "step {} is done", step + 1);
    }
    let queen = at(guid, "sanctuary.json", QUEEN);
    now += 10;
    release_end_screen(&mut game, &queen, TAKE_A_NUMBER, QUEEN, 140_650, now);
    now += 10;
    finish_quest(&mut game, &queen, TAKE_A_NUMBER, ADVENTURER, 30, 0, now);
    assert_eq!(quest_status(&game, TAKE_A_NUMBER), QuestStatus::Completed);
}
