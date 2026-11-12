//! A boys' or girls' quest reward piece goes only to that wearer.
//!
//! The wiki lists gendered rewards as a pair (Chapter 1: The Missing Sisters pays the girls'
//! Rolled-Sleeve Hoodie 190 and the boys' Hoodie 199), and the quest rows keep both. The turn-in
//! grant (`rewards::apply_grant`, given the character's gender by `gameplay/npc.rs`) pays the
//! row whose item-table GenderUsage (1 boys, 2 girls) fits the character and every unisex
//! row; `Inventory::reward_fits_gender` is the rule.
//!
//! The tests drive the production router with the client's own packets (26/8, 26/10, 26/6,
//! 49/2, 49/14; layouts in `quest_job_xp.rs`) on Chef: Ranger Requisition, a one-talk quest,
//! with its reward row replaced by the gendered pair plus its own unisex shard.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    Fixture, accept, character_at_for, finish_quest, npc_location, offer, quest_status,
    release_end_screen, seed_active_profile, talk_step,
};
use osfr_tools::{
    gameplay::Gameplay,
    identity::{CharacterIdentity, NativeCreation},
    quest_engine::QuestStatus,
};
use serde_json::{Value, json};

const CHARACTER: u64 = 235_012;
const CHEF: i32 = 45;
/// QF: Ranger Requisition's giver is Merigold, the wiki's (Ricardo takes the hand-in).
const MERIGOLD: u64 = 9_000_000_402;
const RICARDO: u64 = 9_000_000_178;
const RANGER_REQUISITION: u32 = 3_050_011;
const RANGER_END_TEXT: u32 = 104_136;
const RANGER_XP: u32 = 100;
/// Item-table GenderUsage 2: the girls' Rolled-Sleeve Hoodie of The Missing Sisters.
const GIRLS_HOODIE: i32 = 190;
/// Item-table GenderUsage 1: the boys' Hoodie of The Missing Sisters.
const BOYS_HOODIE: i32 = 199;
/// Item-table GenderUsage 0: the quest's own Chef's Power Shard of Timing I.
const UNISEX_SHARD: i32 = 75_665;

fn fixture(reward: &[i32]) -> Fixture {
    let fixture = Fixture::for_character(CHARACTER);
    fixture.patch_quest("chef.json", RANGER_REQUISITION, |quest| {
        let template = quest["rewards"][0]["items"][0].clone();
        assert_eq!(template["definition_id"], UNISEX_SHARD);
        let items: Vec<Value> = reward
            .iter()
            .map(|definition| {
                let mut item = template.clone();
                item["definition_id"] = json!(definition);
                item
            })
            .collect();
        quest["rewards"][0]["items"] = Value::Array(items);
    });
    fixture
}

/// A Chef; `gender` 1 or 2 arrives the way the client sends it, in the character identity.
/// `None` leaves the session without an identity (gender 0).
fn chef(fixture: &Fixture, gender: Option<i32>) -> Gameplay {
    let mut state = fixture.game(None).state();
    seed_active_profile(&mut state, CHEF, &[1, CHEF]);
    let mut game = fixture.game(Some(state));
    if let Some(gender) = gender {
        let identity = CharacterIdentity {
            guid: 1,
            first_name: "Ari".into(),
            last_name: "Vale".into(),
            model: 1,
            head: "head.adr".into(),
            hair: "hair.adr".into(),
            model_customization: String::new(),
            face_paint: "none".into(),
            skin_tone: "skin".into(),
            eye_color: 262,
            hair_color: 116,
            gender,
            creation: NativeCreation {
                first_name: "Ari".into(),
                last_name: "Vale".into(),
                temporary_first_name: String::new(),
                temporary_last_name: String::new(),
                choices: [0; 21],
            },
            attachments: Vec::new(),
        };
        assert!(game.apply_identity(&identity).unwrap());
    }
    game
}

fn owned(game: &Gameplay, definition: i32) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition)
        .map(|item| item.count)
        .sum()
}

/// Offer, accept, the one talk and the turn-in; returns what the bag gained of each id.
fn turn_in(game: &mut Gameplay) -> [u32; 3] {
    let ids = [GIRLS_HOODIE, BOYS_HOODIE, UNISEX_SHARD];
    let before = ids.map(|id| owned(game, id));
    let at = |npc| {
        let (zone, position) = npc_location(npc);
        character_at_for(CHARACTER, zone, position)
    };
    let (merigold, ricardo) = (at(MERIGOLD), at(RICARDO));
    offer(game, &merigold, MERIGOLD, RANGER_REQUISITION, 7_000);
    accept(game, &merigold, RANGER_REQUISITION, 7_010);
    talk_step(game, &ricardo, RICARDO, 7_020, "Chef: Ranger Requisition");
    release_end_screen(
        game,
        &ricardo,
        RANGER_REQUISITION,
        RICARDO,
        RANGER_END_TEXT,
        7_030,
    );
    finish_quest(
        game,
        &ricardo,
        RANGER_REQUISITION,
        CHEF,
        0,
        RANGER_XP,
        7_040,
    );
    assert_eq!(
        quest_status(game, RANGER_REQUISITION),
        QuestStatus::Completed
    );
    let after = ids.map(|id| owned(game, id));
    [0, 1, 2].map(|i| after[i] - before[i])
}

#[test]
fn a_girl_receives_the_girls_piece_and_every_unisex_item() {
    let fixture = fixture(&[GIRLS_HOODIE, BOYS_HOODIE, UNISEX_SHARD]);
    let mut game = chef(&fixture, Some(2));
    assert_eq!(turn_in(&mut game), [1, 0, 1]);
}

#[test]
fn a_boy_receives_the_boys_piece_and_every_unisex_item() {
    let fixture = fixture(&[GIRLS_HOODIE, BOYS_HOODIE, UNISEX_SHARD]);
    let mut game = chef(&fixture, Some(1));
    assert_eq!(turn_in(&mut game), [0, 1, 1]);
}

/// (R) As for the starter kits (L953), a session whose identity has not arrived is not checked.
#[test]
fn an_unknown_gender_is_not_filtered() {
    let fixture = fixture(&[GIRLS_HOODIE, BOYS_HOODIE, UNISEX_SHARD]);
    let mut game = chef(&fixture, None);
    assert_eq!(turn_in(&mut game), [1, 1, 1]);
}

/// A reward that holds only the other wearer's piece still completes and pays the rest
/// (the job XP `finish_quest` checks); it grants no item and raises no error.
#[test]
fn a_reward_of_only_the_other_genders_piece_completes_without_it() {
    let fixture = fixture(&[BOYS_HOODIE]);
    let mut game = chef(&fixture, Some(2));
    assert_eq!(turn_in(&mut game), [0, 0, 0]);
}

/// The shipped pairs the filter is for: each side of a pair is one boys' and one girls' row.
#[test]
fn the_shipped_pairs_are_one_row_for_each_wearer() {
    let table = npc_quest_chain::read_json(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release/item-table.json"),
    );
    let column = table["columns"]
        .as_array()
        .unwrap()
        .iter()
        .position(|name| name == "GenderUsage")
        .expect("the item table has GenderUsage");
    let usage = |id: i64| {
        table["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row[0] == id)
            .unwrap_or_else(|| panic!("item {id} is in the table"))[1][column]
            .as_i64()
            .unwrap()
    };
    // The Missing Sisters (and Battle Bry), Mail Marksman's Flat Rate shorts, then the three
    // Winterwear quests (boys' Subarctic, girls' Alpine).
    for (boys, girls) in [
        (199, 190),
        (75_967, 75_968),
        (76_589, 76_604),
        (76_590, 76_605),
        (76_591, 76_606),
    ] {
        assert_eq!((usage(boys), usage(girls)), (1, 2), "{boys}/{girls}");
    }
}
