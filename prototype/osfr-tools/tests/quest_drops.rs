//! Quest drops (combat design section 12): an enemy kind drops a quest's item only for a
//! player whose quest is on the step that collects it.
//!
//! Each test kills field enemies through the real 26/8 menu and 26/10 Attack path
//! (`kill_field_enemy`) over a scratch content copy: one synthetic quest in the production
//! split catalogue and `quest_drops` rows added to the copied `field_kill_rewards.json`. A
//! chance of 100 makes "it dropped" and "it did not" exact; the roll itself is the engine's
//! existing power-up random source.
#[path = "common/combat_scale.rs"]
mod combat_scale;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, FILLER_ITEM, Fixture, character_at_for, counts, fill_bag, kill_field_enemy,
    read_json,
};
use osfr_tools::{
    gameplay::{Gameplay, GameplayState},
    inventory::{InventoryItem, MAX_ITEMS},
    quest_engine::{CharacterEligibility, QuestEngine},
};
use serde_json::{Value, json};
use std::fs;

const A: u64 = CHARACTER;
const BRAWLER: i32 = 43;
const QUEST: u32 = 9_200_201;
const GIVER: u64 = 9_000_200_201;
const WOLF: u32 = 940_301;
const FIRST_GUID: u64 = 1_791_001;
/// A catalogued bag item (the fixture's bag filler).
const FANG: i32 = FILLER_ITEM;
/// An id no item catalogue defines: a counter with no bag item.
const TALLY: i32 = 987_654_321;

fn source() -> Value {
    json!([{"kind": "reconstructed", "evidence": "(R) scratch quest drop"}])
}

fn spot(index: u64) -> [f32; 3] {
    [index as f32 * 30.0, 0.0, 0.0]
}

fn wolf(index: u64) -> Value {
    json!({
        "guid": FIRST_GUID + index,
        "definition_id": WOLF,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic wolf",
        "texture_alias": "",
        "position": spot(index),
        "max_health": combat_scale::authored_for(combat_scale::strike(BRAWLER, 1), 1),
        "damage": 1,
        "boss": false,
        "level": 1,
        "respawn_seconds": 600,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode":"stationary","reach":2.5,"attack_ms":1000},
        "quest_ids": []
    })
}

/// `wolves` field wolves, a quest "defeat one wolf, then collect two of `item`", and one drop
/// row for it.
fn fixture(wolves: u64, item: i32, chance: u32, drop_quest: u32) -> Fixture {
    let enemies: Vec<Value> = (0..wolves).map(wolf).collect();
    let fixture = Fixture::for_character(A).with_field_enemies(&enemies);
    let catalogue = fixture.quest_catalogue();
    let record = json!({
        "id": QUEST,
        "title_string_id": 700_020,
        "giver_npc_id": GIVER,
        "prerequisite_quest_ids": [],
        "objectives": [
            {"kind": "defeat", "id": 1, "model_id": WOLF, "count": 1, "source": source()},
            {"kind": "collect_item", "id": 2, "item_id": item, "quantity": 2, "source": source()},
        ],
        "rewards": [{"coins": 5, "items": [], "job_xp": {}, "source": source()}],
        "source": source(),
    });
    fs::write(
        catalogue.join("quests/quest-drops.json"),
        serde_json::to_vec(&json!({"schema_version": 1, "records": [record]})).unwrap(),
    )
    .unwrap();
    let jobs_path = catalogue.join("jobs.json");
    let mut jobs = read_json(&jobs_path);
    jobs["records"][0]["quest_ids"]
        .as_array_mut()
        .unwrap()
        .push(json!(QUEST));
    fs::write(&jobs_path, serde_json::to_vec(&jobs).unwrap()).unwrap();

    let rewards_path = fixture.path().join("field_kill_rewards.json");
    let mut rewards = read_json(&rewards_path);
    rewards["quest_drops"] = json!([{
        "definition_id": WOLF, "item_id": item, "chance": chance, "quest_id": drop_quest,
        "source": source(),
    }]);
    fs::write(&rewards_path, serde_json::to_vec(&rewards).unwrap()).unwrap();
    fixture
}

fn state(fixture: &Fixture, with_quest: bool) -> GameplayState {
    let mut state = Gameplay::load(fixture.path(), None).unwrap().state();
    let eligibility = CharacterEligibility {
        player_level: 20,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
        ..Default::default()
    };
    let mut engine =
        QuestEngine::from_split_content(&fixture.quest_catalogue(), A, eligibility, None).unwrap();
    if with_quest {
        assert!(engine.offer(QUEST, GIVER).changed);
        assert!(engine.accept(QUEST).changed);
    }
    state.npc_quests = Some(engine.state().clone());
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = BRAWLER;
    profiles.unlocked.insert(BRAWLER);
    state
}

fn play(fixture: &Fixture, state: GameplayState) -> Gameplay {
    let mut game = fixture.game_as(A, Some(state));
    game.initialize_world(&character_at_for(A, 0, spot(0)), 0)
        .unwrap();
    game
}

fn kill(game: &mut Gameplay, index: u64, now: &mut u64) {
    assert!(kill_field_enemy(game, spot(index), FIRST_GUID + index, now));
}

fn held(game: &Gameplay, item: i32) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|row| row.definition_id == item)
        .map(|row| row.count)
        .sum()
}

#[test]
fn a_quest_item_drops_only_while_its_collect_step_is_open() {
    let fixture = fixture(4, FANG, 100, QUEST);
    let mut game = play(&fixture, state(&fixture, true));
    let mut now = 0;

    // The kill that finishes "defeat one" was made before the collect step: no fang.
    kill(&mut game, 0, &mut now);
    assert_eq!(counts(&game, QUEST), [1, 0]);
    assert_eq!(
        held(&game, FANG),
        0,
        "nothing drops before the collect step"
    );

    // Each kill on the collect step drops one fang into the bag, and the bag credits it.
    kill(&mut game, 1, &mut now);
    assert_eq!((held(&game, FANG), counts(&game, QUEST)), (1, vec![1, 1]));
    kill(&mut game, 2, &mut now);
    assert_eq!((held(&game, FANG), counts(&game, QUEST)), (2, vec![1, 2]));

    // The step is full: the same enemy drops nothing more.
    kill(&mut game, 3, &mut now);
    assert_eq!(held(&game, FANG), 2, "nothing drops after the collect step");
}

#[test]
fn a_player_without_the_quest_never_gets_the_drop() {
    let fixture = fixture(2, FANG, 100, QUEST);
    let mut game = play(&fixture, state(&fixture, false));
    let mut now = 0;
    kill(&mut game, 0, &mut now);
    kill(&mut game, 1, &mut now);
    assert_eq!(held(&game, FANG), 0);
}

#[test]
fn a_drop_row_for_another_quest_does_not_feed_this_one() {
    // The row names a quest this player does not have; their own collect step gets nothing.
    let fixture = fixture(3, FANG, 100, QUEST + 1);
    let mut game = play(&fixture, state(&fixture, true));
    let mut now = 0;
    for index in 0..3 {
        kill(&mut game, index, &mut now);
    }
    assert_eq!((held(&game, FANG), counts(&game, QUEST)), (0, vec![1, 0]));
}

#[test]
fn a_drop_with_no_bag_item_credits_the_step_directly() {
    let fixture = fixture(3, TALLY, 100, QUEST);
    let mut game = play(&fixture, state(&fixture, true));
    let mut now = 0;
    kill(&mut game, 0, &mut now);
    let bag_before = game.state().inventory.items.len();
    kill(&mut game, 1, &mut now);
    kill(&mut game, 2, &mut now);
    assert_eq!(
        counts(&game, QUEST),
        [1, 2],
        "the kills count as the collect"
    );
    assert_eq!(
        game.state().inventory.items.len(),
        bag_before,
        "and nothing is put in the bag"
    );
}

#[test]
fn a_full_bag_loses_the_drop_and_says_so_without_crediting() {
    // A catalogued item the character does not hold, so it cannot join a stack in the full bag.
    let probe = Fixture::for_character(A);
    let owned = Gameplay::load(probe.path(), None)
        .unwrap()
        .state()
        .inventory;
    let text = fs::read_to_string(probe.path().join("inventory.json")).unwrap();
    let other = text
        .split("\"definition_id\":")
        .skip(1)
        .filter_map(|rest| {
            let digits: String = rest
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse::<i32>().ok()
        })
        .find(|id| *id != FANG && owned.items.iter().all(|row| row.definition_id != *id))
        .expect("the fixture catalogue has an item the character does not hold");

    // Control: with room in the bag the same row drops and credits.
    let roomy = fixture(2, other, 100, QUEST);
    let mut game = play(&roomy, state(&roomy, true));
    let mut now = 0;
    kill(&mut game, 0, &mut now);
    kill(&mut game, 1, &mut now);
    assert_eq!((held(&game, other), counts(&game, QUEST)), (1, vec![1, 1]));

    let full = fixture(2, other, 100, QUEST);
    let mut saved = state(&full, true);
    // The bag's own ceiling is `inventory::MAX_ITEMS` rows: fill every one.
    fill_bag(&mut saved);
    let filler = saved.inventory.items.last().unwrap().clone();
    for id in saved.inventory.items.len()..MAX_ITEMS {
        saved.inventory.items.push(InventoryItem {
            id: 200_000 + id as i32,
            ..filler.clone()
        });
    }
    saved.inventory.next_instance_id = Some(200_000 + MAX_ITEMS as i64);
    let mut game = play(&full, saved);
    let mut now = 0;
    kill(&mut game, 0, &mut now);
    kill(&mut game, 1, &mut now);
    assert_eq!(counts(&game, QUEST), [1, 0], "no item, no credit");
    assert_eq!(held(&game, other), 0);
}

#[test]
fn bad_drop_rows_refuse_to_load() {
    for (chance, quest) in [(0, QUEST), (101, QUEST), (50, 0)] {
        let fixture = fixture(1, FANG, chance, quest);
        assert!(
            Gameplay::load(fixture.path(), None).is_err(),
            "chance {chance} for quest {quest} must not load"
        );
    }
}
