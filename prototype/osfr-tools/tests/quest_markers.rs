//! T499: quest markers on the visibility and quest paths. Every step is a client request through
//! `Gameplay::handle_in_world` or the NPC-view calls the gateway makes: `update_npc_visibility`
//! (per player) and `shared_npc_view` plus `quest_marker_packets` (shared field).
//!
//! Wire model, from the client readers cited in `src/quest_markers.rs`: a 35/10 is
//! u16 35, u16 10, an i32 count, then 39-byte elements (u64 guid at 0, image id at 17,
//! `008db310`); a 35/11 is the same header, a count, then 16-byte elements (u64 guid first,
//! `008db550`). A 35/2 AddNpc carries the actor id at 4..12 (`00921a10`).

#[path = "support/archer_entry.rs"]
mod archer_entry;

use archer_entry::{
    BOLLYLOG, Fixture, HRUGAGU, LUGABOW, PATH, accept, activate_brawler, character_at_for,
    content_position, learn_archer, offer, quest_reply, recruit_spawn, saved, seed_active_profile,
    send,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    quest_engine::{CharacterEligibility, QuestEngine},
};
use std::collections::BTreeSet;

const MERIGOLD: u64 = 9_000_000_402;
const RAINA: u64 = 9_000_000_177;
const HIGHROAD: u32 = 3_000_020;
const AVAILABLE: i32 = 2;
const TURN_IN: i32 = 6;
const OTHER: u64 = archer_entry::CHARACTER + 1;
/// Rules of the Road: one Talk to its own giver (9000000295), then License to Drive is given
/// by a different NPC (9000000289). Both stand within 25 m of each other.
const ROAD_GIVER: u64 = 9_000_000_295;
const ROAD_NEXT_GIVER: u64 = 9_000_000_289;
const RULES_OF_THE_ROAD: u32 = 3_060_004;
const DERBY_PROFILE: i32 = 49;

fn at(npc: u64, file: &str) -> CharacterState {
    character_at_for(archer_entry::CHARACTER, 0, content_position(file, npc))
}

fn at_merigold() -> CharacterState {
    at(MERIGOLD, "highroad_junction.json")
}

fn word(bytes: &[u8], at: usize) -> usize {
    i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize
}

fn guid(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

/// Every (guid, image id) the 35/10 packets in `packets` carry.
fn shown(packets: &[Vec<u8>]) -> Vec<(u64, i32)> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 10, 0]))
        .flat_map(|packet| {
            (0..word(packet, 4)).map(move |row| {
                let at = 8 + row * 39;
                (guid(packet, at), word(packet, at + 17) as i32)
            })
        })
        .collect()
}

/// Every guid the 35/11 packets in `packets` clear.
fn cleared(packets: &[Vec<u8>]) -> Vec<u64> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 11, 0]))
        .flat_map(|packet| (0..word(packet, 4)).map(move |row| guid(packet, 8 + row * 16)))
        .collect()
}

fn marker_count(packets: &[Vec<u8>]) -> usize {
    shown(packets).len() + cleared(packets).len()
}

fn spawn_index(packets: &[Vec<u8>], actor: u64) -> Option<usize> {
    packets
        .iter()
        .position(|packet| packet.starts_with(&[35, 0, 2, 0]) && guid(packet, 4) == actor)
}

fn marker_index(packets: &[Vec<u8>], actor: u64, image: i32) -> Option<usize> {
    packets
        .iter()
        .position(|packet| shown(std::slice::from_ref(packet)).contains(&(actor, image)))
}

fn offer_and_accept(game: &mut Gameplay, character: &CharacterState, giver: u64, quest: u32) {
    offer(game, character, giver, quest, 2_000);
    accept(game, character, quest, 2_100);
}

#[test]
fn giver_shows_available_after_add_npc() {
    let fixture = Fixture::new();
    // Per player: the 35/10 follows Merigold's 35/2 in the same batch.
    let mut game = fixture.game(None);
    let batch = game.update_npc_visibility(&at_merigold()).unwrap();
    let spawn = spawn_index(&batch, MERIGOLD).expect("Merigold is added");
    let marker = marker_index(&batch, MERIGOLD, AVAILABLE).expect("Merigold gets '!'");
    assert!(spawn < marker, "a marker never precedes its AddNpc");

    // Shared field: the same order out of `shared_npc_view`, once, and nothing when known.
    let game = fixture.game(None);
    let (visible, batch) = game
        .shared_npc_view(&at_merigold(), &[], &BTreeSet::new())
        .unwrap();
    let spawn = spawn_index(&batch, MERIGOLD).expect("Merigold is added");
    let marker = marker_index(&batch, MERIGOLD, AVAILABLE).expect("Merigold gets '!'");
    assert!(spawn < marker, "a marker never precedes its AddNpc");
    let (_, again) = game.shared_npc_view(&at_merigold(), &[], &visible).unwrap();
    assert_eq!(again, Vec::<Vec<u8>>::new(), "a known view sends nothing");
}

#[test]
fn accept_moves_the_marker() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let character = at_merigold();
    game.update_npc_visibility(&character).unwrap();
    let before = game.quest_marker_revision();
    offer(&mut game, &character, MERIGOLD, HIGHROAD, 2_000);
    assert_ne!(game.quest_marker_revision(), before, "an offer moves it");
    let accepted = accept(&mut game, &character, HIGHROAD, 2_100);
    assert!(cleared(&accepted.replies).contains(&MERIGOLD));
    assert!(shown(&accepted.replies).contains(&(RAINA, TURN_IN)));
    assert!(
        !shown(&accepted.replies)
            .iter()
            .any(|(id, _)| *id == MERIGOLD)
    );
}

#[test]
fn turn_in_clears_and_unlocks() {
    let fixture = Fixture::new();
    let mut state = fixture.game(None).state();
    seed_active_profile(&mut state, DERBY_PROFILE, &[]);
    let mut game = fixture.game(Some(state));
    let character = at(ROAD_GIVER, "wilds.json");
    let first = game.update_npc_visibility(&character).unwrap();
    assert!(shown(&first).contains(&(ROAD_GIVER, AVAILABLE)));
    assert!(!shown(&first).iter().any(|(id, _)| *id == ROAD_NEXT_GIVER));

    offer_and_accept(&mut game, &character, ROAD_GIVER, RULES_OF_THE_ROAD);
    let talked = archer_entry::talk(&mut game, &character, ROAD_GIVER, 2_200);
    assert!(talked.changed, "the self-talk completes the only goal");
    assert!(
        !shown(&talked.replies)
            .iter()
            .any(|(id, _)| *id == ROAD_NEXT_GIVER),
        "the next giver waits for the turn-in"
    );
    send(
        &mut game,
        &character,
        &archer_entry::dialogue_choice(archer_entry::QUEST_CHOICE),
        2_300,
    );
    let done = send(
        &mut game,
        &character,
        &archer_entry::quest_end_reply(RULES_OF_THE_ROAD),
        2_400,
    );
    assert!(
        cleared(&done.replies).contains(&ROAD_GIVER),
        "the hand-in marker goes"
    );
    assert!(shown(&done.replies).contains(&(ROAD_NEXT_GIVER, AVAILABLE)));
    assert_eq!(
        done.replies.last(),
        Some(&vec![26, 0, 4, 0]),
        "markers go ahead of the dialogue close"
    );
}

#[test]
fn refused_accept_changes_no_marker() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let character = at_merigold();
    game.update_npc_visibility(&character).unwrap();
    let (revision, before) = (game.quest_marker_revision(), saved(&game));
    // Merigold never offered it: the forged 49/2 is refused.
    let refused = game.handle_in_world(&quest_reply(HIGHROAD, true), &character, 2_000);
    let replies = refused.map(|out| out.replies).unwrap_or_default();
    assert_eq!(marker_count(&replies), 0, "no marker packet for a refusal");
    assert_eq!(game.quest_marker_revision(), revision);
    assert_eq!(saved(&game), before);
}

#[test]
fn players_see_their_own_markers() {
    let fixture = Fixture::new();
    let mut first = fixture.game_as(archer_entry::CHARACTER, None);
    let mut second = fixture.game_as(OTHER, None);
    let (one, two) = (
        at_merigold(),
        character_at_for(OTHER, 0, at_merigold().position),
    );
    first.update_npc_visibility(&one).unwrap();
    second.update_npc_visibility(&two).unwrap();
    let (revision, before) = (second.quest_marker_revision(), saved(&second));
    offer(&mut first, &one, MERIGOLD, HIGHROAD, 2_000);
    let accepted = accept(&mut first, &one, HIGHROAD, 2_100);
    assert!(accepted.broadcast.is_none() && accepted.nearby.is_empty());
    // The other player's state, revision and view are untouched, and still show '!'.
    assert_eq!(second.quest_marker_revision(), revision);
    assert_eq!(saved(&second), before);
    let (visible, batch) = second.shared_npc_view(&two, &[], &BTreeSet::new()).unwrap();
    assert!(shown(&batch).contains(&(MERIGOLD, AVAILABLE)));
    assert!(!shown(&second.quest_marker_packets(&visible)).contains(&(RAINA, TURN_IN)));
    assert!(shown(&first.quest_marker_packets(&visible)).contains(&(RAINA, TURN_IN)));
}

#[test]
fn reconnect_recomputes_markers() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let character = at_merigold();
    game.update_npc_visibility(&character).unwrap();
    offer_and_accept(&mut game, &character, MERIGOLD, HIGHROAD);
    let mut back = fixture.reload_serialized(&game);
    let batch = back.update_npc_visibility(&character).unwrap();
    assert!(
        shown(&batch).contains(&(RAINA, TURN_IN)),
        "'?' over the talk target"
    );
    assert!(!shown(&batch).iter().any(|(id, _)| *id == MERIGOLD));
    assert!(spawn_index(&batch, RAINA) < marker_index(&batch, RAINA, TURN_IN));
    assert!(
        cleared(&batch).is_empty(),
        "a new actor has nothing to clear"
    );
}

#[test]
fn idle_pass_sends_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let character = at_merigold();
    assert!(!game.update_npc_visibility(&character).unwrap().is_empty());
    let revision = game.quest_marker_revision();
    for _ in 0..2 {
        assert_eq!(
            game.update_npc_visibility(&character).unwrap(),
            Vec::<Vec<u8>>::new()
        );
    }
    // A request that changes nothing sends no marker either.
    let ignored = game.handle_in_world(&quest_reply(HIGHROAD, false), &character, 2_000);
    assert_eq!(
        marker_count(&ignored.map(|out| out.replies).unwrap_or_default()),
        0
    );
    assert_eq!(game.quest_marker_revision(), revision);
}

/// The job is part of quest eligibility, so a job switch refreshes it inside its own packet: the
/// marker pass at the end of that packet sees it, and a later refused request moves nothing.
#[test]
fn job_switch_settles_markers_in_its_own_packet() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let character = at_merigold();
    game.update_npc_visibility(&character).unwrap();
    let before = game.quest_marker_revision();
    activate_brawler(&mut game, &character, &mut 2_000);
    let switched = game.quest_marker_revision();
    assert_ne!(switched, before, "the job switch moved eligibility");
    let refused = game.handle_in_world(&quest_reply(HIGHROAD, true), &character, 3_000);
    assert_eq!(
        marker_count(&refused.map(|out| out.replies).unwrap_or_default()),
        0
    );
    assert_eq!(game.quest_marker_revision(), switched);
}

#[test]
fn extra_spawn_marker_uses_the_runtime_guid() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let lugabow = at(LUGABOW, "greenwood_forest.json");
    let mut now = 2_000;
    learn_archer(&mut game, &lugabow, &mut now);
    game.update_npc_visibility(&lugabow).unwrap();
    offer(&mut game, &lugabow, LUGABOW, PATH, now + 10);
    let accepted = accept(&mut game, &lugabow, PATH, now + 20);
    let (spawn_guid, _) = recruit_spawn(BOLLYLOG);
    assert_ne!(
        spawn_guid, BOLLYLOG,
        "the extra row spawns under its own guid"
    );
    // The path is ordered: the first recruit is the open talk target, the second is not yet.
    assert!(shown(&accepted.replies).contains(&(spawn_guid, TURN_IN)));
    assert!(
        !shown(&accepted.replies)
            .iter()
            .any(|(id, _)| *id == BOLLYLOG)
    );
    let (second, _) = recruit_spawn(HRUGAGU);
    assert!(!shown(&accepted.replies).iter().any(|(id, _)| *id == second));
}

#[test]
fn revision_moves_only_when_quest_state_does() {
    let eligibility = CharacterEligibility {
        player_level: 10,
        job_levels: Default::default(),
        member: true,
        nonmember_quest_limit: 5,
        member_quest_limit: 5,
    };
    let mut engine = QuestEngine::from_split_content(
        &archer_entry::content_dir(),
        archer_entry::CHARACTER,
        eligibility.clone(),
        None,
    )
    .unwrap();
    let start = engine.revision();
    // Refusals and no-ops change nothing.
    assert!(!engine.offer(HIGHROAD, RAINA).changed, "wrong giver");
    assert!(!engine.accept(HIGHROAD).changed, "nothing offered");
    engine.set_eligibility(eligibility.clone());
    assert!(engine.set_open_events(&BTreeSet::new()).is_empty());
    assert_eq!(engine.revision(), start);
    // Each real change moves it.
    assert!(engine.offer(HIGHROAD, MERIGOLD).changed);
    let offered = engine.revision();
    assert_ne!(offered, start);
    assert!(engine.decline(HIGHROAD).changed);
    assert_ne!(engine.revision(), offered);
    let declined = engine.revision();
    engine.set_eligibility(CharacterEligibility {
        player_level: 11,
        ..eligibility
    });
    assert_ne!(engine.revision(), declined);
    assert!(engine.is_quest_npc(MERIGOLD) && engine.is_quest_npc(RAINA));
    assert!(!engine.is_quest_npc(1));
}

#[test]
fn old_startup_snapshots_still_load_the_renamed_add_npc_field() {
    // `lab/content/startup-snapshot/startup.json` spells the field `unknown_field_67`.
    let mut row = serde_json::to_value(osfr_tools::startup_world::NpcSpawn::default()).unwrap();
    let value = row
        .as_object_mut()
        .unwrap()
        .remove("notification_image_set_id")
        .expect("the field serializes under its new name");
    row["unknown_field_67"] = value;
    let spawn: osfr_tools::startup_world::NpcSpawn = serde_json::from_value(row).unwrap();
    assert_eq!(spawn.notification_image_set_id, 0);
}
