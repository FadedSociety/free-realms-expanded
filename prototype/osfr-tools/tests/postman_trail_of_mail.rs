//! F343: the live one-Talk turn-in for Postmaster Lane's "Trail of Mail"
//! (3130032), the row T314 gave a reward.
//!
//! The quest is content-only: Postmaster Lane (9000000173) offers it, one Talk
//! to Postchug Runalug (9000000172) completes the only objective and opens the
//! 49/13 end screen, and the 49/14 reply settles 5 coins + 5 Postman (profile
//! 4) XP. These tests drive the production in-world router
//! (`Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `QuestEngine`); they do
//! not call `QuestOutcome` or `finish_turnin` directly.
//!
//! Client wire evidence used here is the shared NPC quest chain:
//! - 26/8 interact constructor `0x008b6bc0`; 26/10 menu-select constructor
//!   `0x00a36f80`; 26/6 dialogue-response constructor `0x009f5890`.
//! - 49/1 offer reader `0x00c7bb60`; 49/2 accept constructor `0x00a86400`;
//!   49/13 end-screen reader `0x00c7b990` (actor `6..14`, quest `14..18`, end
//!   text `18..22`, reward bundle `0x008e7930`); 49/14 reply constructor
//!   `0x00a864c0`.
//!
//! The shared fixture and packet builders live in
//! `tests/support/npc_quest_chain.rs`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, assert_packet_changes_nothing, character_at_for, coins,
    content_position, counts, end_screen_reward, finish_quest, offer, pending_turnin, profile_xp,
    quest_end_reply, quest_status, release_end_screen, saved, talk, talk_step,
};
use osfr_tools::{character_store::CharacterState, gameplay::Gameplay, quest_engine::QuestStatus};

const QUEST: u32 = 3_130_032;
const GIVER: u64 = 9_000_000_173;
const TARGET: u64 = 9_000_000_172;
const END_TEXT: u32 = 141_831;
const POSTMAN: i32 = 4;
const REWARD_COINS: u32 = 5;
const REWARD_XP: u32 = 5;

fn at(npc_id: u64, file: &str) -> CharacterState {
    // The open-world zone resolves every regional label by distance (npc_interaction
    // `check_range`), so the character stands on the authored content position.
    character_at_for(CHARACTER, 0, content_position(file, npc_id))
}

/// The opening fixture with Postman unlocked: its starter profile set is
/// `[1, 43]`, and the reward's profile-4 XP must be grantable (T249's unlock rule).
fn postman_game(fixture: &Fixture) -> Gameplay {
    let mut state = fixture.game(None).state();
    state
        .profiles
        .as_mut()
        .expect("the opening fixture stores profiles")
        .unlocked
        .insert(POSTMAN);
    fixture.game(Some(state))
}

#[test]
fn postmaster_lane_pays_trail_of_mail_once() {
    let fixture = Fixture::new();
    let mut game = postman_game(&fixture);
    let giver = at(GIVER, "shrouded_glade.json");
    let target = at(TARGET, "wugachug.json");
    let mut now = 1_000;

    offer(&mut game, &giver, GIVER, QUEST, now);
    now += 10;
    accept(&mut game, &giver, QUEST, now);
    now += 10;
    talk_step(&mut game, &target, TARGET, now, "Postman: Trail of Mail");
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(pending_turnin(&game), Some(QUEST));

    // A reconnect between the final Talk and 49/14 must keep the pending
    // turn-in; the same production bind reloads the saved state.
    let mut game = fixture.reload(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), Some(QUEST));

    // The held 49/13 packet is session memory; the production recovery is a
    // fresh Talk at the actor that opened the screen, which re-sends it.
    now += 10;
    talk(&mut game, &target, TARGET, now);
    now += 10;
    let end = release_end_screen(&mut game, &target, QUEST, TARGET, END_TEXT, now);
    assert_eq!(
        end.len(),
        99,
        "the no-display-row reward bundle keeps the client-reader length"
    );
    assert_eq!(end_screen_reward(&end), (REWARD_COINS, REWARD_XP));

    now += 10;
    finish_quest(
        &mut game,
        &target,
        QUEST,
        POSTMAN,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&game), REWARD_COINS);
    assert_eq!(profile_xp(&game, POSTMAN), REWARD_XP);

    // A second 49/14 must not pay twice or change any saved state.
    let finished = saved(&game);
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &target,
        &quest_end_reply(QUEST),
        now,
        "duplicate Trail of Mail turn-in",
    );
    assert_eq!(coins(&game), REWARD_COINS);
    assert_eq!(profile_xp(&game, POSTMAN), REWARD_XP);
    assert_eq!(saved(&game), finished);

    // The completed ledger and the paid reward survive the same production bind
    // a reconnect uses; the reward is not paid again on load.
    let reloaded = fixture.reload(&game);
    assert_eq!(quest_status(&reloaded, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&reloaded), REWARD_COINS);
    assert_eq!(profile_xp(&reloaded, POSTMAN), REWARD_XP);
}
