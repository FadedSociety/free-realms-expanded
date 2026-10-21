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
    content_position, counts, end_screen_reward, finish_quest, has_family, offer, pending_turnin,
    profile_xp, quest_end_reply, quest_status, saved, send, talk_step,
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
/// `[1, 43]` (pinned by [`a_locked_postman_profile_is_paid_nothing`]), and the
/// reward's profile-4 XP must be grantable (T249's unlock rule).
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
    // turn-in through the serialized save state, not only the live clone.
    let held = saved(&game);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(
        saved(&game),
        held,
        "reload changed the saved pending turn-in"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), Some(QUEST));

    // The production relog batch re-sends the held 49/13 after its speaker's
    // 35/2 (`gateway.rs` -> `update_npc_visibility` ->
    // `append_pending_turnin_screen`); `ShowQuestEndScreen` (`00a95420`)
    // resolves the actor through the client table and never retries.
    let packets = game
        .update_npc_visibility(&target)
        .expect("relog NPC visibility");
    let spawn = packets.iter().position(|packet| {
        osfr_tools::startup_world::parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == TARGET)
    });
    let end = packets
        .iter()
        .position(|packet| has_family(packet, 49, 13, Some(QUEST)))
        .expect("the held end screen is re-sent after relog");
    assert!(
        spawn.is_some_and(|spawn| spawn < end),
        "the speaker's 35/2 must precede its 49/13"
    );
    assert_eq!(
        u64::from_le_bytes(packets[end][6..14].try_into().unwrap()),
        TARGET,
        "the re-sent end screen must name the runtime speaker"
    );
    assert_eq!(
        u32::from_le_bytes(packets[end][18..22].try_into().unwrap()),
        END_TEXT,
        "the re-sent end screen must carry the quest's debrief text"
    );
    assert_eq!(
        packets[end].len(),
        99,
        "the no-display-row reward bundle keeps the client-reader length"
    );
    assert_eq!(end_screen_reward(&packets[end]), (REWARD_COINS, REWARD_XP));

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

    // The completed ledger and the paid reward survive the serialized reconnect;
    // the reward is not paid again on load.
    let reloaded = fixture.reload_serialized(&game);
    assert_eq!(
        saved(&reloaded),
        finished,
        "reload changed completed history"
    );
    assert_eq!(quest_status(&reloaded, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&reloaded), REWARD_COINS);
    assert_eq!(profile_xp(&reloaded, POSTMAN), REWARD_XP);
}

/// The reward names profile 4, so a character without Postman is refused at
/// turn-in by the F249 reward gate (`QuestDenial::Job`): the end screen clears,
/// nothing is paid, and the quest stays active for a later Talk. The content has
/// no job gate on `3130032` (pre-existing T314 gap named in `notes/Autopilot/
/// F365.md`); the first test unlocks Postman as its setup.
#[test]
fn a_locked_postman_profile_is_paid_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    assert!(
        !game
            .state()
            .profiles
            .as_ref()
            .expect("the opening fixture stores profiles")
            .unlocked
            .contains(&POSTMAN),
        "the opening fixture starts without Postman"
    );
    let giver = at(GIVER, "shrouded_glade.json");
    let target = at(TARGET, "wugachug.json");
    let mut now = 2_000;

    offer(&mut game, &giver, GIVER, QUEST, now);
    now += 10;
    accept(&mut game, &giver, QUEST, now);
    now += 10;
    talk_step(&mut game, &target, TARGET, now, "Postman: Trail of Mail");
    assert_eq!(pending_turnin(&game), Some(QUEST));
    let coins_before = coins(&game);

    now += 10;
    let settled = send(&mut game, &target, &quest_end_reply(QUEST), now);
    assert!(
        settled.changed,
        "the locked-profile refusal still releases the end screen"
    );
    assert!(
        !settled
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 4, Some(QUEST))),
        "a locked profile must not report quest completion"
    );
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), coins_before);
    assert_eq!(profile_xp(&game, POSTMAN), 0);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [1]);
}
