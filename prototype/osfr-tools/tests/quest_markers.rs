//! T499: quest markers on the visibility and quest paths. Every step is a client request through
//! `Gameplay::handle_in_world`, the real gateway wrapper (`Gateway::handle_at`), or the NPC-view
//! calls the gateway makes: `update_npc_visibility` (per player) and `shared_npc_view` plus
//! `quest_marker_packets` (shared field).
//!
//! The wire decoder is `support/quest_marker_packets.rs` (client readers `008db310`, `008db550`).

#[path = "support/archer_entry.rs"]
mod archer_entry;
#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "support/quest_marker_packets.rs"]
mod quest_marker_packets;
// Only `reference()` is used; the rest of the shared round70 fixture is other suites' base.
#[path = "common/round70_content.rs"]
#[allow(dead_code)]
mod round70_content;

use archer_entry::{
    BOLLYLOG, BRAWLER, Fixture, HRUGAGU, LUGABOW, PATH, accept, activate_brawler, character_at_for,
    content_position, learn_archer, offer, quest_reply, recruit_spawn, saved, seed_active_profile,
    send,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    gateway::Gateway,
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
/// A derby arena driver; `derby_gates.json` lists it and it stands in `wilds.json` beside the
/// Rules of the Road giver, whose quest is gated on the Demo Derby Driver job.
const CRASH: u64 = ROAD_NEXT_GIVER;
const DERBY_LABEL: u32 = 3053;
/// A kart race gate (`kart_gates.json`) in `wilds.json`, with its menu label.
const KART_GATE: u64 = 9_000_000_273;
const KART_LABEL: u32 = 3052;

fn at(npc: u64, file: &str) -> CharacterState {
    character_at_for(archer_entry::CHARACTER, 0, content_position(file, npc))
}

fn at_merigold() -> CharacterState {
    at(MERIGOLD, "highroad_junction.json")
}

/// Every (guid, image id) the 35/10 packets in `packets` carry.
fn shown(packets: &[Vec<u8>]) -> Vec<(u64, i32)> {
    let (rows, _) = quest_marker_packets::markers(packets);
    rows.iter().map(|row| (row.guid, row.image)).collect()
}

/// Every guid the 35/11 packets in `packets` clear.
fn cleared(packets: &[Vec<u8>]) -> Vec<u64> {
    quest_marker_packets::markers(packets).1
}

fn marker_count(packets: &[Vec<u8>]) -> usize {
    shown(packets).len() + cleared(packets).len()
}

fn spawn_index(packets: &[Vec<u8>], actor: u64) -> Option<usize> {
    quest_marker_packets::spawn_index(packets, actor)
}

fn marker_index(packets: &[Vec<u8>], actor: u64, image: i32) -> Option<usize> {
    quest_marker_packets::markers(packets)
        .0
        .iter()
        .find(|row| (row.guid, row.image) == (actor, image))
        .map(|row| row.index)
}

/// The application bodies of a `Gateway::handle_at` reply batch (the shared tunnel reader).
fn bodies(replies: &[Vec<u8>]) -> Vec<Vec<u8>> {
    replies
        .iter()
        .map(|reply| gateway_packets::body(reply).to_vec())
        .collect()
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
    // The row carries the NPC's own name string id, the one its AddNpc names.
    let name = osfr_tools::startup_world::parse_add_npc(&batch[spawn])
        .unwrap()
        .name_string_id;
    let row = quest_marker_packets::markers(&batch).0;
    assert_ne!(name, 0);
    assert_eq!(row.iter().find(|r| r.guid == MERIGOLD).unwrap().name, name);

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

/// A live rebind (an admin content reload) rebuilds the engine on the same `Gameplay`; it must
/// continue the revision past the engine it replaces, or the marker pass would see an unchanged
/// number and never refresh what the client still shows.
#[test]
fn live_rebind_continues_the_revision_and_keeps_markers() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let character = at_merigold();
    game.update_npc_visibility(&character).unwrap();
    offer(&mut game, &character, MERIGOLD, HIGHROAD, 2_000);
    let before = game.quest_marker_revision();
    game.bind_npc_quest_catalogue(&fixture.quest_catalogue(), archer_entry::CHARACTER)
        .expect("the live rebind loads");
    let rebound = game.quest_marker_revision();
    assert!(
        rebound > before,
        "a rebind continues past the engine it replaces"
    );
    // The rebind restored the offer from the live state, so the next pass still owns a marker
    // change to send for what the client already shows.
    let refreshed = game.update_npc_visibility(&character).unwrap();
    assert!(marker_count(&refreshed) > 0, "the rebind refreshes markers");
    assert_eq!(
        game.update_npc_visibility(&character).unwrap(),
        Vec::<Vec<u8>>::new(),
        "one refresh per rebind"
    );
}

/// The real gateway wrapper (`gateway.rs` `Event::Position`): a move that brings Merigold inside
/// the 100-unit view sends her AddNpc and then her '!' from the same pass, so a client never
/// receives a notification for an actor it has not spawned.
#[test]
fn gateway_position_sends_the_marker_after_the_add_npc() {
    let fixture = Fixture::new();
    // The narrowed quest catalogue is the production bind's input; copy its split files into the
    // fixture so `Gameplay::load` (NPC rows, profiles) and the bind read one content directory.
    archer_entry::copy_missing(&fixture.quest_catalogue(), fixture.path());
    let npc = content_position("highroad_junction.json", MERIGOLD);
    let far = [npc[0] + 150.0, npc[1], npc[2]];
    let character = character_at_for(archer_entry::CHARACTER, 0, far);
    let mut gateway = Gateway::load_gameplay(
        &round70_content::reference(),
        character,
        fixture.path(),
        None,
    )
    .expect("the gateway content loads");
    let guid = archer_entry::CHARACTER;
    gateway
        .handle_at(&gateway_packets::login(guid), 1_000)
        .expect("login handles");
    gateway
        .handle_at(&gateway_packets::wrap(&[13, 0]), 1_010)
        .expect("ready handles");
    let loaded = gateway
        .handle_at(&gateway_packets::wrap(&[10, 0]), 1_020)
        .expect("loaded handles");
    assert!(
        spawn_index(&bodies(&loaded.replies), MERIGOLD).is_none(),
        "150 units is outside the 100-unit view"
    );
    let near = [npc[0] + 40.0, npc[1], npc[2]];
    let moved = gateway
        .handle_at(
            &gateway_packets::wrap(&gateway_packets::move_to(guid, near)),
            2_000,
        )
        .expect("the accepted move handles");
    let batch = bodies(&moved.replies);
    let spawn = spawn_index(&batch, MERIGOLD).expect("the move adds Merigold");
    let marker = marker_index(&batch, MERIGOLD, AVAILABLE).expect("Merigold gets '!'");
    assert!(spawn < marker, "the marker follows its AddNpc");
}

/// A session in `wilds.json` with the kart and derby packages bound, the shape the shipped
/// content has: the derby gate is the NPC the Rules of the Road quest already stands beside.
fn race_fixture(active: i32, stands_at: u64) -> (Fixture, Gameplay, CharacterState) {
    let fixture = Fixture::new();
    for file in [
        "kart_entry.json",
        "kart_gates.json",
        "kart_lobby.json",
        "derby_entry.json",
        "derby_gates.json",
    ] {
        std::fs::copy(
            archer_entry::content_dir().join(file),
            fixture.path().join(file),
        )
        .unwrap();
    }
    let mut state = fixture.game(None).state();
    seed_active_profile(&mut state, active, &[BRAWLER]);
    let game = fixture.game(Some(state));
    let character = at(stands_at, "wilds.json");
    (fixture, game, character)
}

/// Entering the derby through the driver's menu changes the job inside the same packet that the
/// marker pass reads: the job-gated giver shows its '!' in that reply, not one request later.
#[test]
fn derby_entry_settles_markers_in_its_own_packet() {
    let (_fixture, mut game, character) = race_fixture(BRAWLER, CRASH);
    let first = game.update_npc_visibility(&character).unwrap();
    assert!(
        !shown(&first).iter().any(|(id, _)| *id == ROAD_GIVER),
        "the Brawler cannot take the derby quest"
    );
    let menu = archer_entry::interact(&mut game, &character, CRASH, 2_000);
    let option = archer_entry::menu_option(&menu, DERBY_LABEL);
    let entered = send(
        &mut game,
        &character,
        &archer_entry::npc_select(CRASH, option),
        2_100,
    );
    assert!(
        shown(&entered.replies).contains(&(ROAD_GIVER, AVAILABLE)),
        "the new job's quest shows in the entry reply"
    );
}

/// Every kart switch settles eligibility where the job changes: the race clears the derby job's
/// '!' in its own reply, the exit puts it back in its own reply, and a refused request after the
/// exit sends no marker and moves nothing.
#[test]
fn kart_race_switches_settle_markers_in_their_own_packets() {
    let (_fixture, mut game, character) = race_fixture(DERBY_PROFILE, KART_GATE);
    let first = game.update_npc_visibility(&character).unwrap();
    assert!(shown(&first).contains(&(ROAD_GIVER, AVAILABLE)));
    let menu = archer_entry::interact(&mut game, &character, KART_GATE, 1_900);
    let option = archer_entry::menu_option(&menu, KART_LABEL);
    let racing = send(
        &mut game,
        &character,
        &archer_entry::npc_select(KART_GATE, option),
        2_000,
    );
    assert!(
        cleared(&racing.replies).contains(&ROAD_GIVER),
        "the Kart Driver cannot take the derby quest"
    );
    let back = send(
        &mut game,
        &character,
        &gateway_packets::chat("realm exit"),
        3_000,
    );
    assert!(
        shown(&back.replies).contains(&(ROAD_GIVER, AVAILABLE)),
        "the restored job's quest returns in the exit reply"
    );
    let settled = game.quest_marker_revision();
    let before = saved(&game);
    let refused = game.handle_in_world(&quest_reply(RULES_OF_THE_ROAD, true), &character, 4_000);
    assert_eq!(
        marker_count(&refused.map(|out| out.replies).unwrap_or_default()),
        0
    );
    assert_eq!(game.quest_marker_revision(), settled);
    assert_eq!(saved(&game), before);
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
