//! Earned pets (`pets::earn`): the shipped `pet_rewards.json`, the counters, the claim, the saved
//! state, the bag search for reward pet items, and the real path through `Gameplay`: accepted
//! pet tricks (client 53/7) earn the first trick pet on the pet tick.
//!
//! The table's numbers are tunable data and are not pinned here; the tests read them.
#[path = "support/pet_care.rs"]
mod care;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "support/pet_wire.rs"]
mod pet_wire;

use care::{ADOPT_DEF, ADOPT_ITEM, content_dir, dog, trick_row};
use npc_quest_chain::{CHARACTER, Fixture};
use osfr_tools::{
    character_store::CharacterState,
    chat,
    gameplay::Gameplay,
    pets::{EARN_FILE, EarnRules, PetContent, PetService},
};
use pet_inventory::InventoryFixture;
use pet_wire::{perform_trick, summon};
use serde_json::{Value, json};
use std::collections::BTreeSet;

const START: u64 = 1_000_000;
const TRICK: u32 = 11;
/// Two pet items of the bundled content: Char (red) and a Balloon Animal.
const CHAR: i32 = 70_352;
const BALLOON: i32 = 17_841;

fn content() -> PetContent {
    PetContent::bundled().unwrap()
}

fn shipped() -> Value {
    json::read(&content_dir().join(EARN_FILE))
}

fn rules(rows: Value) -> EarnRules {
    let table = json!({ "schema_version": 1, "enabled": true, "rows": rows });
    EarnRules::from_json(&serde_json::to_vec(&table).unwrap(), &content()).unwrap()
}

fn row(key: &str, counter: &str, at: u32, item: i32) -> Value {
    json!({
        "key": key, "counter": counter, "at": at, "item_id": item, "name": key,
        "goal": format!("{counter} {at}"), "kind": "reconstructed", "basis": "test"
    })
}

fn service(rows: Value) -> PetService {
    let mut service = PetService::new(content()).unwrap();
    service.set_earn_rules(rules(rows));
    service
}

fn pets_in(document: Option<Value>) -> Vec<Value> {
    document
        .and_then(|roster| roster["pets"].as_array().cloned())
        .unwrap_or_default()
}

#[test]
fn the_shipped_table_loads_and_gives_every_pet_item_a_source_or_a_reason() {
    let table = shipped();
    let rules = EarnRules::from_json(&serde_json::to_vec(&table).unwrap(), &content()).unwrap();
    assert!(rules.is_enabled());
    assert!(!rules.rows().is_empty());
    assert!(rules.provenance()["generator"].is_string());

    let mut covered: BTreeSet<i64> = rules.rows().iter().map(|r| i64::from(r.item_id)).collect();
    for entry in rules.unwired() {
        assert!(entry["reason"].is_string(), "every unwired entry says why");
        covered.extend(entry["item_id"].as_i64());
        covered.extend(
            entry["item_ids"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_i64),
        );
    }
    let missing: Vec<i64> = care::pets_json()["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["item_id"].as_i64())
        .filter(|id| !covered.contains(id))
        .collect();
    assert!(
        missing.is_empty(),
        "pet items with no source and no reason: {missing:?}"
    );

    // One pet item is earned once: two rows for one item would hand out a duplicate.
    let mut items = BTreeSet::new();
    for row in rules.rows() {
        assert!(
            items.insert(row.item_id),
            "item {} has two rows",
            row.item_id
        );
    }
    assert!(
        rules
            .rows()
            .iter()
            .any(|row| i64::from(row.item_id) == i64::from(BALLOON)),
        "the earn-only Balloon Animal has a source"
    );
}

/// A `quest:<id>` row is only reachable when that quest exists and is enabled.
#[test]
fn every_quest_row_names_an_enabled_quest() {
    let mut enabled = BTreeSet::new();
    for file in std::fs::read_dir(content_dir().join("quests")).unwrap() {
        for record in json::read(&file.unwrap().path())["records"]
            .as_array()
            .unwrap()
        {
            if record["enabled"] == json!(true) {
                enabled.extend(record["id"].as_u64());
            }
        }
    }
    for row in shipped()["rows"].as_array().unwrap() {
        if let Some(id) = row["counter"]
            .as_str()
            .and_then(|c| c.strip_prefix("quest:"))
        {
            assert!(
                enabled.contains(&id.parse::<u64>().unwrap()),
                "row {} needs quest {id}, which is not enabled",
                row["key"]
            );
        }
    }
}

#[test]
fn a_counter_that_reaches_its_goal_gives_the_pet_once_and_survives_a_reload() {
    let rows = json!([
        row("first", "quest", 2, CHAR),
        row("second", "quest", 3, BALLOON)
    ]);
    let mut service = service(rows.clone());
    assert!(
        service.claim_earned(10).earned.is_empty(),
        "nothing is due at zero"
    );

    service.earn_bump("quest", 1);
    assert!(
        service.claim_earned(10).earned.is_empty(),
        "one short of the goal"
    );
    service.earn_bump("quest", 1);
    let claim = service.claim_earned(10);
    assert_eq!(claim.earned.len(), 1);
    assert_eq!(claim.earned[0].name, "first");
    assert_eq!(
        claim.earned[0].next,
        Some(("second".into(), "quest 3".into(), 2, 3)),
        "the next pet on the same counter, with the progress"
    );
    assert_eq!(
        claim.replies,
        vec![service.list_packet()],
        "the 53/5 roster follows"
    );
    let pet = service
        .pet(claim.earned[0].pet_id)
        .expect("the pet is in the roster");
    assert_eq!(pet.adoption_item_definition_id, Some(CHAR));
    assert_eq!(pet.tint, Some(282), "the item row's own tint is the coat");
    assert!(service.has_earned("first") && !service.has_earned("second"));

    // Paid once: more of the same counter never pays the row again.
    service.earn_bump("quest", 1);
    let again = service.claim_earned(11);
    assert_eq!(again.earned.len(), 1, "only the second row");
    assert_eq!(again.earned[0].name, "second");
    assert_eq!(again.earned[0].next, None);
    service.earn_bump("quest", 5);
    assert!(service.claim_earned(12).earned.is_empty());

    // The counters and the paid rows ride in the roster document.
    let document = service.document_value().expect("a roster with pets");
    let mut reloaded = PetService::from_document(content(), Some(document.clone())).unwrap();
    reloaded.set_earn_rules(rules(rows));
    assert_eq!(reloaded.earn_count("quest"), 8);
    assert!(
        reloaded.claim_earned(13).earned.is_empty(),
        "a reload pays nothing twice"
    );
    assert_eq!(reloaded.document_value(), Some(document));
}

#[test]
fn counters_are_saved_before_the_first_pet() {
    let mut service = service(json!([row("far", "kill", 100, CHAR)]));
    assert_eq!(
        service.document_value(),
        None,
        "an untouched roster keeps its saved value"
    );
    service.earn_bump("kill", 40);
    let document = service
        .document_value()
        .expect("the counter alone is worth saving");
    let reloaded = PetService::from_document(content(), Some(document)).unwrap();
    assert_eq!(reloaded.earn_count("kill"), 40);
}

#[test]
fn without_a_table_nothing_is_counted_and_unnamed_counters_are_not_stored() {
    let mut dormant = PetService::new(content()).unwrap();
    dormant.earn_bump("quest", 50);
    dormant.earn_day(7);
    assert_eq!(dormant.earn_count("quest"), 0);
    assert!(dormant.claim_earned(1).earned.is_empty());
    assert_eq!(dormant.document_value(), None);

    let mut service = service(json!([row("only", "quest", 1, CHAR)]));
    service.earn_bump("minigame:999", 3);
    service.earn_reach("job_level", 20);
    assert_eq!(
        service.document_value(),
        None,
        "counters no row names are never stored"
    );
}

#[test]
fn a_job_level_is_a_high_water_mark_and_a_day_counts_once() {
    let mut service = service(json!([
        row("rank", "job_level", 10, CHAR),
        row("week", "active_day", 2, BALLOON)
    ]));
    service.earn_reach("job_level", 9);
    service.earn_reach("job_level", 4);
    assert_eq!(
        service.earn_count("job_level"),
        9,
        "a lower rank of another job changes nothing"
    );
    service.earn_day(100);
    service.earn_day(100);
    assert_eq!(service.earn_count("active_day"), 1, "one day counts once");
    assert!(service.claim_earned(1).earned.is_empty());
    service.earn_reach("job_level", 10);
    service.earn_day(101);
    let names: Vec<String> = service
        .claim_earned(2)
        .earned
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(names, ["week", "rank"]);
}

#[test]
fn a_full_roster_keeps_the_pet_due_and_says_so_once() {
    let mut service = service(json!([row("late", "quest", 1, CHAR)]));
    let definition = content().definition(1_000_015).unwrap();
    let mut grant = 1;
    while service.room() > 0 {
        service.adopt_vendor_grant(grant, &definition, 0).unwrap();
        grant += 1;
    }
    let before = service.roster_snapshot();
    service.earn_bump("quest", 1);
    let claim = service.claim_earned(5);
    assert!(claim.earned.is_empty() && claim.replies.is_empty());
    assert!(claim.roster_full, "the owner is told");
    assert!(!service.has_earned("late"), "the row stays due");
    assert_eq!(
        service
            .roster_snapshot()
            .map(|r| serde_json::to_value(r).unwrap()["pets"].clone()),
        before.map(|r| serde_json::to_value(r).unwrap()["pets"].clone()),
        "no pet changed"
    );
    service.earn_bump("quest", 1);
    assert!(!service.claim_earned(6).roster_full, "told once a session");
}

#[test]
fn a_bad_table_is_refused() {
    let content = content();
    let load = |table: Value| EarnRules::from_json(&serde_json::to_vec(&table).unwrap(), &content);
    let good = row("a", "quest", 1, CHAR);
    assert!(load(json!({ "schema_version": 1, "enabled": true, "rows": [good.clone()] })).is_ok());
    assert!(load(json!({ "schema_version": 2, "enabled": true, "rows": [] })).is_err());
    let mut twice = vec![good.clone(), good.clone()];
    assert!(load(json!({ "schema_version": 1, "enabled": true, "rows": twice })).is_err());
    twice = vec![row("zero", "quest", 0, CHAR)];
    assert!(load(json!({ "schema_version": 1, "enabled": true, "rows": twice })).is_err());
    twice = vec![row("no-pet", "quest", 1, 41)];
    assert!(
        load(json!({ "schema_version": 1, "enabled": true, "rows": twice })).is_err(),
        "an item that is no pet adoption item"
    );
}

/// A reward applier puts a pet item in the bag without a grant list (a collection reward, a
/// quest reward, a treasure chest). The pet tick's bag search adopts it.
#[test]
fn a_pet_item_a_reward_left_in_the_bag_becomes_a_pet() {
    let mut service = PetService::new(content()).unwrap();
    let mut fixture = InventoryFixture::with_items(&[(ADOPT_ITEM, ADOPT_DEF, 1), (40, 912, 1)]);
    let replies = service.adopt_bag_items(&mut fixture.inventory, 20);
    assert_eq!(
        replies.last(),
        Some(&service.list_packet()),
        "the roster closes the replies"
    );
    assert_eq!(pets_in(service.document_value()).len(), 1);
    let left: Vec<i32> = fixture
        .inventory
        .state()
        .items
        .iter()
        .map(|i| i.definition_id)
        .collect();
    assert_eq!(left, [912], "the pet item is consumed, the brush stays");
    assert!(
        service
            .adopt_bag_items(&mut fixture.inventory, 21)
            .is_empty()
    );
}

fn who() -> CharacterState {
    CharacterState {
        guid: CHARACTER,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}

/// One accepted trick through the real route, played to its end. Returns every reply.
fn do_trick(game: &mut Gameplay, at: u64) -> Vec<Vec<u8>> {
    let who = who();
    let row = trick_row(TRICK);
    let mut replies = game
        .handle_in_world(&perform_trick(TRICK), &who, at)
        .unwrap()
        .replies;
    for step in [
        row.start_delay_ms as u64,
        (row.start_delay_ms + row.duration_ms) as u64,
    ] {
        replies.extend(
            game.pet_tick(&who, at + step)
                .map(|out| out.replies)
                .unwrap_or_default(),
        );
    }
    replies
}

/// The real entry path: a content directory with the shipped table, a summoned dog, and the
/// client's 53/7 trick requests through `Gameplay::handle_in_world`. The goal of the first
/// trick-ladder row is read from the table, so tuning it does not move this test.
#[test]
fn accepted_tricks_through_gameplay_earn_the_first_trick_pet_and_it_is_saved() {
    let first = shipped()["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["counter"] == json!("pet_trick"))
        .min_by_key(|row| row["at"].as_u64())
        .cloned()
        .expect("the shipped table has a trick ladder");
    let goal = first["at"].as_u64().unwrap();

    let fixture = Fixture::minimal(CHARACTER);
    fixture.copy_content(EARN_FILE);
    let (pets, pet) = dog();
    let mut game = Gameplay::load(fixture.path(), None).unwrap();
    game.adopt_pets(pets.document_value()).unwrap();
    game.initialize_world(&who(), START).unwrap();
    game.handle_in_world(&summon(pet), &who(), START + 1_000)
        .unwrap();

    let earned_line = chat::system(&format!(
        "You earned a new pet: {}! ({}) Open your Pets window to meet it.",
        first["name"].as_str().unwrap(),
        first["goal"].as_str().unwrap()
    ));
    let mut at = START + 2_000;
    for trick in 1..goal {
        let replies = do_trick(&mut game, at);
        assert!(
            !replies.contains(&earned_line),
            "trick {trick} of {goal} earns nothing yet"
        );
        at += 60_000;
    }
    assert_eq!(pets_in(game.pets_document()).len(), 1);

    let replies = do_trick(&mut game, at);
    assert!(
        replies.contains(&earned_line),
        "the goal's trick earns the pet and says so"
    );
    let roster = pets_in(game.pets_document());
    assert_eq!(roster.len(), 2, "the new pet is in the roster");
    assert_eq!(roster[1]["adoption_item_definition_id"], first["item_id"]);
    assert!(
        replies.iter().any(|packet| packet.starts_with(&[53, 0, 5])),
        "the client gets the new 53/5 roster"
    );

    // Saved and reloaded: the pet is there, and the same count pays nothing again.
    let document = game.pets_document();
    let mut next = Gameplay::load(fixture.path(), None).unwrap();
    next.adopt_pets(document.clone()).unwrap();
    next.initialize_world(&who(), START).unwrap();
    let tick = next
        .pet_tick(&who(), START + 5_000)
        .map(|out| out.replies)
        .unwrap_or_default();
    assert!(!tick.contains(&earned_line));
    assert_eq!(pets_in(next.pets_document()).len(), 2);
    assert_eq!(next.pets_document(), document);
}

/// A content directory without the table earns nothing (dormant by default).
#[test]
fn a_content_directory_without_the_table_earns_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let (pets, pet) = dog();
    let mut game = Gameplay::load(fixture.path(), None).unwrap();
    game.adopt_pets(pets.document_value()).unwrap();
    game.initialize_world(&who(), START).unwrap();
    game.handle_in_world(&summon(pet), &who(), START + 1_000)
        .unwrap();
    let before = game.pets_document();
    let mut at = START + 2_000;
    for _ in 0..3 {
        do_trick(&mut game, at);
        at += 60_000;
    }
    assert_eq!(game.pets_document(), before, "no counter, no pet");
}

/// The saved paid-once record is the row key, so the shipped key is the pet item alone: moving
/// a row to another counter or goal must never pay an owned pet again.
#[test]
fn a_shipped_row_key_is_its_pet_item_and_survives_a_retuned_ladder() {
    for row in shipped()["rows"].as_array().unwrap() {
        assert_eq!(row["key"], json!(format!("pet#{}", row["item_id"])));
    }
    let mut service = service(json!([row("pet#70352", "quest", 1, CHAR)]));
    service.earn_bump("quest", 1);
    assert_eq!(service.claim_earned(1).earned.len(), 1);
    // The same pet retuned onto another counter and goal: already paid.
    service.set_earn_rules(rules(json!([row("pet#70352", "dungeon", 3, CHAR)])));
    service.earn_bump("dungeon", 3);
    assert!(service.claim_earned(2).earned.is_empty());
    assert_eq!(pets_in(service.document_value()).len(), 1);
}

/// A capped counter grows by at most its cap per activity day; the cap is saved, so a relog on
/// the same day does not reopen it, and the next day does.
#[test]
fn a_daily_cap_limits_a_counter_per_day_and_is_saved() {
    let table = json!({
        "schema_version": 1, "enabled": true, "daily_caps": { "pet_trick": 3 },
        "rows": [row("coat", "pet_trick", 5, CHAR), row("free", "kill", 5, BALLOON)]
    });
    let load = || EarnRules::from_json(&serde_json::to_vec(&table).unwrap(), &content()).unwrap();
    let mut service = PetService::new(content()).unwrap();
    service.set_earn_rules(load());
    service.set_earn_day(100);
    for _ in 0..20 {
        service.earn_bump("pet_trick", 1);
    }
    assert_eq!(service.earn_count("pet_trick"), 3, "the day's cap");
    service.earn_bump("kill", 20);
    assert_eq!(service.earn_count("kill"), 20, "a counter without a cap is not limited");

    let mut relogged = PetService::from_document(content(), service.document_value()).unwrap();
    relogged.set_earn_rules(load());
    relogged.set_earn_day(100);
    relogged.earn_bump("pet_trick", 1);
    assert_eq!(relogged.earn_count("pet_trick"), 3, "a relog does not reopen the day");
    relogged.set_earn_day(101);
    relogged.earn_bump("pet_trick", 5);
    assert_eq!(relogged.earn_count("pet_trick"), 6, "the next day allows the cap again");

    let zero = json!({
        "schema_version": 1, "enabled": true, "daily_caps": { "pet_trick": 0 }, "rows": []
    });
    assert!(EarnRules::from_json(&serde_json::to_vec(&zero).unwrap(), &content()).is_err());
}

/// The shipped pet-care counters are capped, so tricks alone cannot fill the roster in a day.
#[test]
fn the_shipped_pet_care_counters_have_a_daily_cap() {
    let table = shipped();
    let rules = EarnRules::from_json(&serde_json::to_vec(&table).unwrap(), &content()).unwrap();
    for counter in ["pet_trick", "pet_feed", "pet_groom"] {
        assert!(rules.daily_cap(counter).is_some(), "{counter} has no daily cap");
    }
}
