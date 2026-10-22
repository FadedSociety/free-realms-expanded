//! Event helpers for the scavenger hunt (3000200) suites.
//!
//! `tests/scavenger_hunt_quest.rs` (one player) and
//! `tests/scavenger_hunt_two_players.rs` (two players) both include this module.
//! It re-exports the canonical fixture and packet helpers from
//! `tests/support/npc_quest_chain.rs` and adds only the hunt-specific steps: the
//! fresh Adventurer character, the host's content position, the ordered stop
//! visits and the refusal snapshots that compare the whole state *and* its
//! serialized save bytes. Never copy these helpers into a suite.
//!
//! Client evidence (2014 client, checked with `lab/autopilot/ida_query.py`):
//! - 26/8 NPC interact constructor `008b6bc0`: two GUID dwords + one bool.
//! - 26/10 menu select constructor `00a36f80`: two GUID dwords + i32 choice.
//! - 26/6 dialogue response `009f5890`: i32 row (301 is the quest row).
//! - 49/2 QuestReply constructor `00a86400`: quest id + accept byte.
//! - 49/14 QuestEndReply constructor `00a864c0`: one quest id.
//! - 49/13 QuestEnd reader `00c7b990`: u64 actor at 6..14, quest 14..18, end text
//!   18..22, animation 22..26, reward bundle (coins 27..31, XP 31..35).
//! - 49/9 ObjectiveUpdate reader `00c7af60`: quest, objective, current, progress
//!   float, bool, trailing dword.
//! - 49/10 ObjectiveComplete reader `00c7b070`: four dwords then the bool.
//! No packet shape is added here; every builder below is the shared
//! `npc_quest_chain` one.
#![allow(dead_code)]

#[path = "npc_quest_chain.rs"]
mod npc_quest_chain;

pub use npc_quest_chain::*;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    npc_interaction::NpcInteraction,
    profiles::ProfileState,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// The fixed admin event quest (T247), hosted by Jacque since T281.
pub const QUEST: u32 = 3_000_200;
/// Event host and return target. His `legacy_data` Sanctuary row at Jacque's Cafe
/// has no quest bindings, so one Talk offers the hunt directly.
pub const JACQUE: u64 = 9_500_000_497;
/// Stop 1: Pappy Farnum's farm.
pub const PAPPY: u64 = 9_000_000_165;
/// Stop 2: the Cobblestone brawling ring (Carlos Brazenfist).
pub const CARLOS: u64 = 9_000_000_042;
/// Stop 3: Highroad Junction (Merigold).
pub const MERIGOLD: u64 = 9_000_000_402;
/// The reward row names Adventurer (profile 1) explicitly.
pub const ADVENTURER: i32 = 1;
/// Used only by the reward-profile assertion; the hunt itself has no job gate.
pub const BRAWLER: i32 = 43;
/// The three ordered stops in hunt order.
pub const STOPS: [u64; 3] = [PAPPY, CARLOS, MERIGOLD];

/// `events.json` with the hunt open, as `realm event open 3000200` writes it
/// (T418; `tests/live_events.rs` drives the command itself).
pub const HUNT_OPEN: &str = r#"{"schema_version":1,"open":[3000200]}"#;

/// A fresh character with the hunt open in the fixture's own live folder. The
/// hunt is a live event (T418), so the suites written before it start with the
/// admin's open already done. Sessions rebuilt later with `Fixture::reload` or
/// `game_as` have no live folder, so the hunt is closed for them; they only
/// continue a hunt already accepted, which a closed event keeps.
pub fn fresh(fixture: &Fixture, character_id: u64) -> Gameplay {
    let live = fixture.open_live_events([QUEST]);
    fresh_in(fixture, character_id, &live)
}

/// A fresh character with the Adventurer profile active, bound to the real
/// bundled split quest content exactly as production binds it, whose live world
/// (`--world`) is `live`. The Brawler profile is unlocked so the turn-in tests
/// can switch the active job.
pub fn fresh_in(fixture: &Fixture, character_id: u64, live: &Path) -> Gameplay {
    let mut state = fixture.game_as(character_id, None).state();
    state.profiles = Some(ProfileState {
        version: 1,
        active: ADVENTURER,
        unlocked: BTreeSet::from([ADVENTURER, BRAWLER]),
        experience: BTreeMap::new(),
    });
    let mut game = fixture.game_as(character_id, Some(state));
    game.set_world_dir(Some(live.to_path_buf()))
        .expect("the live world folder opens");
    game
}

pub fn npc_position(npc_id: u64) -> [f32; 3] {
    let content = content_dir();
    NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("NPC content loads")
        .location(npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} is a positioned content spawn"))
        .1
}

/// `subtype` is 8 (activated), 9 (update) or 10 (complete); each carries the
/// quest id at 6..10 and the objective id at 10..14.
pub fn has_objective(replies: &[Vec<u8>], subtype: u16, objective_id: u32) -> bool {
    replies.iter().any(|packet| {
        packet.len() >= 14
            && packet[..6] == [49, 0, subtype as u8, 0, 0, 0]
            && packet[6..10] == QUEST.to_le_bytes()
            && packet[10..14] == objective_id.to_le_bytes()
    })
}

/// Jacque has no competing quest offer, so one Talk offers the hunt directly
/// through the production Talk/26/6 path.
pub fn offer_hunt(game: &mut Gameplay, jacque: &CharacterState, now: &mut u64) -> Outcome {
    let offered = offer(game, jacque, JACQUE, QUEST, *now);
    *now += 4;
    assert_eq!(pending_offer(game), Some(QUEST));
    offered
}

pub fn accept_hunt(game: &mut Gameplay, jacque: &CharacterState, now: &mut u64) {
    let accepted = accept(game, jacque, QUEST, *now);
    *now += 2;
    let add = accepted
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 3, Some(QUEST)))
        .expect("accept sends the 49/3 quest record");
    // 49/3 packet: six-byte header, 103-byte fixed prefix and reward bundle,
    // then the objective count at 109..113 and one 107-byte record each. Client
    // reader chain 0x0093c840 (fixed prefix and count) -> 0x0092b050 (count and
    // per-objective loop; it reads each record's leading id dword) -> 0x008fd770
    // (the rest of the 107-byte record).
    let objective_count = i32::from_le_bytes(add[109..113].try_into().unwrap());
    assert_eq!(objective_count, 4, "the hunt has four objectives");
    let ids: Vec<u32> = (0..usize::try_from(objective_count).unwrap())
        .map(|index| {
            let offset = 113 + index * 107;
            u32::from_le_bytes(add[offset..offset + 4].try_into().unwrap())
        })
        .collect();
    assert_eq!(ids, [1, 2, 3, 4], "the objectives are in hunt order");
    assert!(
        has_objective(&accepted.replies, 8, 1),
        "accept activates the first ordered objective"
    );
    assert_eq!(counts(game, QUEST), [0, 0, 0, 0]);
    assert_eq!(pending_offer(game), None);
}

/// Visit stop `index` (0-based) and assert the ordered counters after it.
pub fn visit_stop(game: &mut Gameplay, guid: u64, index: usize, now: &mut u64) {
    let mut expected = [0, 0, 0, 0];
    for value in expected.iter_mut().take(index + 1) {
        *value = 1;
    }
    visit(game, guid, npc_position(STOPS[index]), now, expected);
}

/// Walk one character through the first `count` stops in order.
pub fn walk_stops(game: &mut Gameplay, guid: u64, count: usize, now: &mut u64) {
    for index in 0..count {
        visit_stop(game, guid, index, now);
    }
}

pub fn visit(
    game: &mut Gameplay,
    guid: u64,
    position: [f32; 3],
    now: &mut u64,
    expected: [u32; 4],
) -> Outcome {
    *now += 1;
    // Runtime movement is the one open-world zone; npcs/*.json zone ids are
    // content region labels (src/npc_interaction.rs check_range).
    let outcome = game
        .on_location(&character_at_for(guid, 0, position), *now)
        .expect("movement is accepted");
    assert!(outcome.changed, "the landmark must advance the hunt");
    assert_eq!(counts(game, QUEST), expected, "landmark counters");
    // The increment itself is a 49/9 `ObjectiveUpdate` (client reader
    // 0x00c7af60: quest 6..10, objective 10..14, current 14..18); 49/10 and the
    // next 49/8 are asserted by the caller.
    let advanced = expected.iter().take_while(|count| **count == 1).count();
    assert!(
        objective_update(&outcome, QUEST, advanced as u32, 1),
        "the visit sends the 49/9 update for objective {advanced}"
    );
    outcome
}

pub fn assert_move_changes_nothing(
    game: &mut Gameplay,
    guid: u64,
    zone_id: u32,
    position: [f32; 3],
    now: u64,
    label: &str,
) {
    let before_state = game.state();
    let before_bytes = saved(game);
    let at = character_at_for(guid, zone_id, position);
    let outcome = game.on_location(&at, now).expect("movement is accepted");
    assert!(!outcome.changed, "{label} changed gameplay");
    assert_eq!(game.state(), before_state, "{label} changed the state");
    assert_eq!(saved(game), before_bytes, "{label} changed the save bytes");
}

/// A refused packet changes neither the whole `GameplayState` nor its serialized
/// save bytes. The shared helper owns the packet/reply checks; this wrapper adds
/// the state snapshot the event's refusal gates require.
pub fn refused(
    game: &mut Gameplay,
    speaker: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before_state = game.state();
    let before_bytes = saved(game);
    assert_packet_changes_nothing(game, speaker, packet, now, label);
    assert_eq!(game.state(), before_state, "{label} changed the state");
    assert_eq!(saved(game), before_bytes, "{label} changed the save bytes");
}

/// An out-of-range 26/8 is refused before any menu, offer or save change.
pub fn refused_interaction(
    game: &mut Gameplay,
    speaker: &CharacterState,
    target: u64,
    now: u64,
    label: &str,
) {
    let before_state = game.state();
    let before_bytes = saved(game);
    assert!(
        game.handle_in_world(&npc_interact(target), speaker, now)
            .is_err(),
        "{label} must be refused"
    );
    assert_eq!(game.state(), before_state, "{label} changed the state");
    assert_eq!(saved(game), before_bytes, "{label} changed the save bytes");
}
