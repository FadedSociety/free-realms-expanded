//! Shared Wizard chapter vocabulary, fixture state and reward assertions.
//!
//! `wizard_entry_chain.rs` includes this with
//! `#[path = "support/wizard_entry.rs"] mod wizard_entry;` and re-exports it. Test-only;
//! production never copies content. The generic chapter harness lives in
//! `tests/support/npc_quest_chain.rs`, shared with the Archer and Medic chapter tests.
#![allow(dead_code)]

#[path = "npc_quest_chain.rs"]
mod npc_quest_chain;

pub use npc_quest_chain::*;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    npc_interaction::{QuestOfferPacket49_1, decode_quest_offer_49_1},
    profiles::ProfileState,
    quest_engine::QuestStatus,
};
use std::collections::{BTreeMap, BTreeSet};

/// A second character id for the per-character ledger test.
pub const SECOND_CHARACTER: u64 = 194_018;
pub const FIZZLESTICKS: u64 = 9_000_000_324;
pub const NURBLE: u64 = 9_000_000_415;
/// Drawable Robgoblin Camp NPC whose ground spot is the spell-book search area.
pub const SQUEE: u64 = 9_500_000_949;
pub const BOOK: u32 = 3_160_003;
pub const GLASSES: u32 = 3_160_004;
/// "Sticky Spell Book" (the Nurble cleaning errand), enabled by T303.
pub const STICKY: u32 = 3_160_005;
pub const BOOK_TITLE: u32 = 20_859;
pub const BOOK_DESCRIPTION: u32 = 20_860;
pub const BOOK_INTRO: u32 = 24_947;
pub const BOOK_END_TEXT: u32 = 9_797;
pub const GLASSES_TITLE: u32 = 22_188;
pub const GLASSES_DESCRIPTION: u32 = 22_189;
/// The guide's row-2 pitch (raw locale hash 2073760606); its match is wording-only (R).
pub const GLASSES_INTRO: u32 = 24_948;
pub const GLASSES_END_TEXT: u32 = 38_882;
pub const STICKY_TITLE: u32 = 22_193;
pub const STICKY_DESCRIPTION: u32 = 22_194;
/// The guide's row-3 pitch (raw locale hash 3719837419); its match is wording-only (R).
pub const STICKY_INTRO: u32 = 24_949;
/// The guide's row-3 hand-in line (raw locale hash 2823854753).
pub const STICKY_END_TEXT: u32 = 22_373;
pub const REWARD_COINS: u32 = 5;
pub const REWARD_XP: u32 = 25;
/// T542's Jobs-window record: Fizzlesticks is its giver, its one talk target and the
/// turn-in.
pub const BECOME_A_WIZARD: u32 = 3_160_035;
pub const WIZARD: i32 = 12;
pub const BRAWLER: i32 = 43;
/// The open continent is world zone 0; regional zone ids are content labels.
pub const OPEN_WORLD: u32 = 0;

/// The player standing in front of `npc_id` in the open continent.
pub fn avatar_at(npc_id: u64) -> CharacterState {
    character_at(OPEN_WORLD, content_position("wilds.json", npc_id))
}

/// A saved Wizard: profile 12 unlocked and active at rank 1 (no experience). The client's
/// level-20 Medic unlock is the trainer row's prerequisite, not a quest gate.
pub fn wizard_state(fixture: &Fixture) -> GameplayState {
    let mut state = Gameplay::load(fixture.path(), None)
        .expect("opening fixture loads")
        .state();
    state.profiles = Some(ProfileState {
        version: 1,
        active: WIZARD,
        unlocked: BTreeSet::from([1, WIZARD, BRAWLER]),
        experience: BTreeMap::new(),
    });
    state
}

/// A Wizard job the player has unlocked but is not playing.
pub fn unlocked_wizard_state(fixture: &Fixture) -> GameplayState {
    let mut state = wizard_state(fixture);
    state.profiles = Some(ProfileState {
        version: 1,
        active: BRAWLER,
        unlocked: BTreeSet::from([1, WIZARD, BRAWLER]),
        experience: BTreeMap::new(),
    });
    state
}

/// T542: Fizzlesticks' menu also offers the ungated Become a Wizard (3160035). A chapter
/// test completes it in the saved state, so his Talk offers only the chapter quests.
pub fn chapter_game_for(fixture: &Fixture, guid: u64, state: GameplayState) -> Gameplay {
    let mut state = fixture.game_as(guid, Some(state)).state();
    seed_quest(&mut state, BECOME_A_WIZARD, QuestStatus::Completed, &[1]);
    fixture.game_as(guid, Some(state))
}

pub fn wizard_game(fixture: &Fixture) -> Gameplay {
    chapter_game_for(fixture, CHARACTER, wizard_state(fixture))
}

/// Talk to the giver and decode the single 49/1 offer with the production reader.
pub fn offered_card(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    quest_id: u32,
    now: u64,
) -> QuestOfferPacket49_1 {
    let offered = offer(game, character, target, quest_id, now);
    let packet = offered
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .expect("the offer contains the 49/1");
    let decoded = decode_quest_offer_49_1(packet).expect("49/1 decodes");
    assert_eq!(decoded.quest_id, quest_id);
    decoded
}

pub fn quest_title(quest_id: u32) -> u32 {
    match quest_id {
        BOOK => BOOK_TITLE,
        GLASSES => GLASSES_TITLE,
        STICKY => STICKY_TITLE,
        other => panic!("quest {other} has no title in this chapter"),
    }
}

/// Release the held 49/13 and assert the client-visible rows: actor and debrief text
/// (shared helper), the (R) zero animation dword, then the reward bundle that
/// `encode_quest_reward_bundle` builds from the quest definition.
pub fn release_wizard_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_end_text: u32,
    now: u64,
) -> Vec<u8> {
    let end = release_end_screen(
        game,
        character,
        quest_id,
        FIZZLESTICKS,
        expected_end_text,
        now,
    );
    assert_eq!(
        u32::from_le_bytes(end[22..26].try_into().unwrap()),
        0,
        "the animation dword is the (R) default"
    );
    assert_reward_rows(&end, quest_id, character.guid);
    end
}

/// The 49/13 reward bundle rows: success byte, coins i32, stars i32, ..., display
/// source guid, player guid, icon, name (offsets per client reader `00c7b990`).
fn assert_reward_rows(end: &[u8], quest_id: u32, player_guid: u64) {
    assert_eq!(end[26], 1, "the reward bundle success byte");
    assert_eq!(
        i32::from_le_bytes(end[27..31].try_into().unwrap()),
        REWARD_COINS as i32,
        "the 49/13 reward bundle carries the coin row"
    );
    assert_eq!(
        i32::from_le_bytes(end[31..35].try_into().unwrap()),
        REWARD_XP as i32,
        "the 49/13 reward bundle carries the job-XP row"
    );
    assert_eq!(
        u32::from_le_bytes(end[83..87].try_into().unwrap()),
        quest_title(quest_id),
        "the reward display names the quest title"
    );
    assert_eq!(
        u64::from_le_bytes(end[63..71].try_into().unwrap()),
        FIZZLESTICKS,
        "the reward display source is the quest giver"
    );
    assert_eq!(
        u64::from_le_bytes(end[71..79].try_into().unwrap()),
        player_guid,
        "the reward display names the character"
    );
}
