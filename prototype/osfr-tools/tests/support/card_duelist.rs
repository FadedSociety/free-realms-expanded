//! T505/T545: the Card Duelist chain's shared fixtures.
//!
//! Include with `#[path = "support/card_duelist.rs"] mod card_duelist;`. It re-exports
//! `npc_quest_chain`, so the trainer, the original-quests and the duel-credit suites share one
//! copy of the fixture, packet and reader helpers. Test-only.

#![allow(dead_code)]

#[path = "npc_quest_chain.rs"]
mod npc_quest_chain;

pub use npc_quest_chain::*;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
};
use serde_json::Value;
use std::fs;

pub const SAM: u64 = 9_000_000_187;
pub const POE: u64 = 9_000_000_170;
pub const DOMINIC: u64 = 9_000_000_074;
pub const ESTHER: u64 = 9_000_000_082;
pub const POES_BEST_DECK: u32 = 3_040_011;
pub const ESTHER_THE_BRAINS: u32 = 3_040_034;
pub const CARD_DUELIST: i32 = 120;
pub const BECOME_A_CARD_DUELIST: u32 = 392_269;

/// The character state at `npc`'s recovered spawn: `sanctuary.json` holds Sam and Poe,
/// `briarwood.json` holds Dominic and Esther.
pub fn at(npc: u64) -> CharacterState {
    at_for(CHARACTER, npc)
}

pub fn at_for(guid: u64, npc: u64) -> CharacterState {
    let file = if npc == POE || npc == SAM {
        "sanctuary.json"
    } else {
        "briarwood.json"
    };
    character_at_for(guid, 0, content_position(file, npc))
}

/// The shared fixture: a private copy of the shipped content with the two Card Duelist quests
/// switched on or off. Callers that need more content (an item-catalogue row) extend it.
pub fn fixture_with(enabled: &[u32]) -> Fixture {
    let fixture = Fixture::new();
    fixture.overlay_content(&content_dir());
    // As `quest_item_reward`: the opening inventory lacks the farming rewards.
    let _ = fs::remove_file(fixture.path().join("harvesting.json"));
    for quest in [POES_BEST_DECK, ESTHER_THE_BRAINS] {
        fixture.patch_quest("card_duelist.json", quest, |record| {
            record["enabled"] = Value::Bool(enabled.contains(&quest));
        });
    }
    fixture
}

/// Sam Potts teaches Card Duelist through the real 26/9 menu (T467).
pub fn learn_card_duelist(game: &mut Gameplay, who: &CharacterState, now: u64) -> Outcome {
    let menu = interact(game, who, SAM, now);
    let choice = menu_option(&menu, BECOME_A_CARD_DUELIST);
    send(game, who, &npc_select(SAM, choice), now + 1)
}

/// Seed profile 120 active in `game`'s save. Card Duelist cannot be selected through 26/13 yet
/// (it has no starter outfit), so the save is seeded active, as `seed_active_profile` documents.
pub fn activate_card_duelist(fixture: &Fixture, game: &Gameplay, guid: u64) -> Gameplay {
    let mut state = game.state();
    seed_active_profile(&mut state, CARD_DUELIST, &[]);
    fixture.game_as(guid, Some(state))
}

/// A Card Duelist as Sam teaches it, with profile 120 active.
pub fn card_duelist_as(fixture: &Fixture, guid: u64) -> Gameplay {
    let who = at_for(guid, SAM);
    let mut game = fixture.game_as(guid, None);
    let learned = learn_card_duelist(&mut game, &who, 1_000);
    assert!(learned.changed, "Sam teaches Card Duelist");
    assert!(
        game.state()
            .profiles
            .expect("profiles")
            .unlocked
            .contains(&CARD_DUELIST)
    );
    activate_card_duelist(fixture, &game, guid)
}

pub fn card_duelist(fixture: &Fixture) -> Gameplay {
    card_duelist_as(fixture, CHARACTER)
}
