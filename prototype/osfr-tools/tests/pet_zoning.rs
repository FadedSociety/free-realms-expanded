//! T487: a summoned pet survives a zone change.
//!
//! The server remembers the pet at the zone change (`settle_pets`), and the destination's
//! `Loaded` signal brings it back through the normal summon path (AddNpc beside the owner, 53/9,
//! 53/5) once nothing closes the pets gate. A pets-blocking activity (a kart race, a battle
//! instance) keeps the memory until the destination loads. The memory is session only: a dismiss,
//! a summon and a reconnect each drop it.
//!
//! Real path: `Gateway` message and tick dispatch, the client's Ready13/Loaded10 pair and the
//! production commit tail (`Session::commit_outcome` runs `settle_pets` after every message and
//! tick). No pet id, actor id or position comes from the client.
//!
//! The kart race is the real pets-blocking activity here: the developer `realm race` command
//! (`begin_kart_race`, the same route the NPC menu choice uses) zones into the track, and 26/42
//! is the client's mid-race exit.

#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/gateway_packets.rs"]
mod gateway_packets;

use gateway_packets::{chat, login, move_to, wrap};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    gameplay::Gameplay,
    gateway::{Gateway, GatewayOutcome},
    pets::{PetBusy, PetService, pet_removal_packet},
    startup_world,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

/// The bundled Char: `pets.json` carries its model and adoption item.
const SPECIES: u32 = 1_000_015;
/// The first pet the roster grants.
const PET: u32 = 1;
const GUID: u64 = 1;
const OWNER: [f32; 3] = [-440.0, 0.8, 1292.0];
static NEXT: AtomicU64 = AtomicU64::new(0);

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn lab(relative: &str) -> PathBuf {
    manifest_dir().join("../../../lab").join(relative)
}

/// The reference content plus the shipped kart entry, so the developer `realm race` command
/// starts a real race (a kart race closes the pets gate).
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-pet-zoning-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        content_copy::copy_tree(
            &lab("references/rust-opening-cooking-original-assets"),
            &directory,
        );
        fs::copy(
            manifest_dir().join("data/content/kart_entry.json"),
            directory.join("kart_entry.json"),
        )
        .unwrap();
        Self(directory)
    }

    fn reference(&self) -> PathBuf {
        lab("references/rust-gateway-reference")
    }

    fn game(&self) -> Gameplay {
        Gameplay::load(&self.0, None).unwrap()
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn who() -> CharacterState {
    CharacterState {
        guid: GUID,
        zone_id: 0,
        position: OWNER,
        rotation: [1.0, 0.0, 0.0],
    }
}

/// One owned pet, as the vendor adoption path leaves the saved roster.
fn roster() -> Option<serde_json::Value> {
    let content = PetService::bundled_content().unwrap();
    let definition = content.definition(SPECIES).unwrap();
    let mut pets = PetService::new(content).unwrap();
    pets.adopt_vendor_grant(GUID, &definition, 0).unwrap();
    pets.document_value()
}

/// A session logged in and loaded into the world (login, Ready13, Loaded10).
fn session(content: &Content) -> Gateway {
    let mut gateway =
        Gateway::load_gameplay(&content.reference(), who(), &content.0, None).unwrap();
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);
    gateway.adopt_pets(roster()).unwrap();
    gateway.handle_at(&login(GUID), 0).unwrap();
    send(&mut gateway, &[13, 0], 100);
    send(&mut gateway, &[10, 0], 200);
    gateway
}

/// One client message, then the production commit tail (`Session::commit_outcome`): a zone
/// change ends the active pet and remembers it for the destination.
fn send(gateway: &mut Gateway, inner: &[u8], now_ms: u64) -> GatewayOutcome {
    let zone = gateway.state().zone_id;
    let outcome = gateway.handle_at(&wrap(inner), now_ms).unwrap();
    let zoned = outcome.began_zoning || gateway.state().zone_id != zone;
    commit(gateway, outcome, zoned, now_ms)
}

/// One world tick through the same commit tail.
fn tick(gateway: &mut Gateway, now_ms: u64) -> GatewayOutcome {
    let zone = gateway.state().zone_id;
    let outcome = gateway.tick_at(now_ms).unwrap();
    let zoned = outcome.began_zoning || gateway.state().zone_id != zone;
    commit(gateway, outcome, zoned, now_ms)
}

fn commit(
    gateway: &mut Gateway,
    mut outcome: GatewayOutcome,
    zoned: bool,
    now_ms: u64,
) -> GatewayOutcome {
    let ended = gateway.settle_pets(zoned, now_ms).unwrap();
    outcome.state_changed |= ended.state_changed;
    outcome.gameplay_changed |= ended.gameplay_changed;
    outcome.replies.extend(ended.replies);
    outcome.nearby.extend(ended.nearby);
    outcome
}

/// Client `Pet.Summon` (`53/4` action 1) and `Pet.Dismiss` (`53/4` action 0).
fn summon(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![1]].concat()
}

fn dismiss() -> Vec<u8> {
    [vec![53, 0, 4], 0u32.to_le_bytes().to_vec(), vec![0]].concat()
}

fn actor() -> u64 {
    PetService::pet_guid(GUID, PET)
}

/// One kart race and its exit through the world's loaded signal.
struct Race {
    /// The race-entry commit (the zone change into the track).
    started: GatewayOutcome,
    /// The Loaded10 of the race track: a pets-blocking destination.
    track: GatewayOutcome,
    /// The Loaded10 after the exit: the world the pet comes back to.
    loaded: GatewayOutcome,
}

/// Enter a kart race (`realm race`), load the track, exit (26/42) and load the world again.
fn race(gateway: &mut Gateway, now: &mut u64) -> Race {
    let started = send(gateway, &chat("realm race"), *now);
    *now += 1_000;
    assert!(
        started.began_zoning,
        "realm race must zone into the track: {:?}",
        chat_text(&started)
    );
    send(gateway, &[13, 0], *now);
    *now += 100;
    let track = send(gateway, &[10, 0], *now);
    *now += 1_000;
    let exited = send(gateway, &[26, 0, 42, 0], *now);
    *now += 1_000;
    assert!(exited.began_zoning, "the race exit zones back");
    send(gateway, &[13, 0], *now);
    *now += 100;
    let loaded = send(gateway, &[10, 0], *now);
    *now += 100;
    Race {
        started,
        track,
        loaded,
    }
}

/// Every AddNpc (35/2, client reader `00921a10`) for `actor`, parsed by the production reader.
/// Replies carry the client's `005` tunnel; the nearby fanout list is bare application bodies.
fn adds(packets: &[Vec<u8>], actor: u64) -> Vec<startup_world::NpcSpawn> {
    packets
        .iter()
        .map(|packet| gateway_packets::body(packet))
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
        .filter(|spawn| spawn.actor_id == actor)
        .collect()
}

/// Every remove-actor (35/3) for `actor`.
fn removals(packets: &[Vec<u8>], actor: u64) -> usize {
    packets
        .iter()
        .map(|packet| gateway_packets::body(packet))
        .filter(|packet| *packet == pet_removal_packet(actor).as_slice())
        .count()
}

fn has(packets: &[Vec<u8>], family: u16, subtype: u8) -> bool {
    packets
        .iter()
        .map(|packet| gateway_packets::body(packet))
        .any(|packet| {
            packet.get(..2) == Some(&family.to_le_bytes()) && packet.get(2) == Some(&subtype)
        })
}

fn chat_text(outcome: &GatewayOutcome) -> String {
    outcome
        .replies
        .iter()
        .map(|reply| gateway_packets::body(reply))
        .filter(|body| body.starts_with(&[15, 0, 3, 0]) && body.len() >= 8)
        .map(|body| {
            let length = u32::from_le_bytes(body[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&body[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_warp_remembers_the_pet_and_the_loaded_signal_brings_it_back_once() {
    let content = Content::new();
    let mut gateway = session(&content);
    let actor = actor();

    // The pet is out, beside its owner, before any warp.
    let summoned = send(&mut gateway, &summon(PET), 1_000);
    assert!(has(&summoned.replies, 53, 9), "active pair");
    assert_eq!(adds(&summoned.replies, actor).len(), 1, "owner actor");
    assert_eq!(adds(&summoned.nearby, actor).len(), 1, "viewer actor");
    assert_eq!(gateway.pet_view().map(|view| view.pet_id), Some(PET));

    // The warp: a kart race closes the pets gate at the destination.
    let mut now = 2_000;
    let raced = send(&mut gateway, &chat("realm race"), now);
    now += 1_000;
    assert!(raced.began_zoning, "{:?}", chat_text(&raced));
    assert_eq!(removals(&raced.nearby, actor), 1, "the warp ends the pet");
    assert!(gateway.pet_view().is_none());

    // Nothing comes back while the race runs.
    send(&mut gateway, &[13, 0], now);
    now += 100;
    let track = send(&mut gateway, &[10, 0], now);
    now += 100;
    assert!(adds(&track.replies, actor).is_empty(), "no pet in a race");
    for _ in 0..5 {
        let out = tick(&mut gateway, now);
        now += 100;
        assert!(adds(&out.replies, actor).is_empty());
        assert!(adds(&out.nearby, actor).is_empty());
    }
    assert!(gateway.pet_view().is_none(), "no pet during the race");

    // The exit is another warp; the pet comes back after the destination loads, once.
    let exited = send(&mut gateway, &[26, 0, 42, 0], now);
    now += 1_000;
    assert!(exited.began_zoning);
    assert!(
        adds(&exited.replies, actor).is_empty(),
        "the pet waits for the loaded signal"
    );
    let ready = send(&mut gateway, &[13, 0], now);
    now += 100;
    assert!(adds(&ready.replies, actor).is_empty(), "not at Ready13");
    let loaded = send(&mut gateway, &[10, 0], now);
    assert_eq!(adds(&loaded.replies, actor).len(), 1, "one resummon");
    assert!(has(&loaded.replies, 53, 9), "the active pair follows");
    assert!(has(&loaded.replies, 53, 5), "and the roster");
    let view = gateway.pet_view().expect("the pet is out again");
    assert_eq!(view.pet_id, PET);
    assert_eq!(view.actor_guid, actor);
    // (R) Beside the owner, not inside it: `pets.json` follow.spawn is 2 m behind and 2 m aside.
    let spawn = &adds(&loaded.replies, actor)[0];
    let owner = gateway.state().position;
    let gap =
        ((spawn.position[0] - owner[0]).powi(2) + (spawn.position[2] - owner[2]).powi(2)).sqrt();
    assert!((2.0..=4.0).contains(&gap), "pet {gap} m from its owner");

    // Never two actors: later ticks keep one pet and add no second one.
    for _ in 0..5 {
        let out = tick(&mut gateway, now);
        now += 100;
        assert!(adds(&out.replies, actor).is_empty());
        assert!(adds(&out.nearby, actor).is_empty());
        assert_eq!(gateway.pet_view().map(|view| view.pet_id), Some(PET));
    }
}

#[test]
fn two_warps_in_a_row_leave_one_pet_and_no_duplicate_actor() {
    let content = Content::new();
    let mut gateway = session(&content);
    let actor = actor();
    send(&mut gateway, &summon(PET), 1_000);
    let mut now = 2_000;

    let first = race(&mut gateway, &mut now);
    assert_eq!(
        removals(&first.started.nearby, actor),
        1,
        "the warp ends the pet"
    );
    assert!(adds(&first.track.replies, actor).is_empty());
    assert_eq!(adds(&first.loaded.replies, actor).len(), 1);
    assert_eq!(gateway.pet_view().map(|view| view.pet_id), Some(PET));

    let second = race(&mut gateway, &mut now);
    assert_eq!(
        removals(&second.started.nearby, actor),
        1,
        "the second warp ends the pet it summoned back"
    );
    assert_eq!(
        adds(&second.track.replies, actor).len(),
        0,
        "the second track holds no pet either"
    );
    assert_eq!(adds(&second.loaded.replies, actor).len(), 1, "one pet");
    let view = gateway.pet_view().expect("one pet after two warps");
    assert_eq!((view.pet_id, view.actor_guid), (PET, actor));
    for _ in 0..3 {
        let out = tick(&mut gateway, now);
        now += 100;
        assert!(adds(&out.nearby, actor).is_empty(), "no second actor");
    }
}

#[test]
fn a_dismissed_pet_never_comes_back_across_a_warp() {
    let content = Content::new();

    // Part one: summon, dismiss, then warp. Nothing was remembered, so nothing returns.
    let mut gateway = session(&content);
    let actor = actor();
    send(&mut gateway, &summon(PET), 1_000);
    let dismissed = send(&mut gateway, &dismiss(), 1_100);
    assert!(has(&dismissed.replies, 53, 9), "the zero pair");
    assert_eq!(removals(&dismissed.nearby, actor), 1);
    assert!(gateway.pet_view().is_none());
    let mut now = 2_000;
    let trip = race(&mut gateway, &mut now);
    assert_eq!(removals(&trip.started.nearby, actor), 0);
    assert!(
        adds(&trip.loaded.replies, actor).is_empty(),
        "a dismissed pet stays away"
    );
    assert!(gateway.pet_view().is_none());

    // Part two: the warp remembers the pet; the player then dismisses it at the destination
    // (a loaded client, no pet out). The dismiss must forget it too (canary: dropping the
    // memory clear in `PetService::dismiss` fails this test).
    send(&mut gateway, &summon(PET), now);
    now += 1_000;
    let started = send(&mut gateway, &chat("realm race"), now);
    now += 1_000;
    assert_eq!(removals(&started.nearby, actor), 1, "the warp ends the pet");
    send(&mut gateway, &[13, 0], now);
    now += 100;
    send(&mut gateway, &[10, 0], now);
    now += 1_000;
    let dismissed = send(&mut gateway, &dismiss(), now);
    assert!(
        has(&dismissed.replies, 53, 9),
        "the client's dismiss reaches the pet service"
    );
    assert!(
        !dismissed.state_changed,
        "a dismiss with no pet changes nothing"
    );
    assert!(adds(&dismissed.replies, actor).is_empty());
    let exited = send(&mut gateway, &[26, 0, 42, 0], now);
    now += 1_000;
    assert!(exited.began_zoning);
    send(&mut gateway, &[13, 0], now);
    now += 100;
    let loaded = send(&mut gateway, &[10, 0], now);
    assert!(
        adds(&loaded.replies, actor).is_empty(),
        "the dismiss cleared the remembered pet"
    );
    assert!(gateway.pet_view().is_none());
}

#[test]
fn a_viewer_loses_the_pet_at_the_warp_and_sees_it_walk_again() {
    let content = Content::new();
    let mut gateway = session(&content);
    let actor = actor();
    send(&mut gateway, &summon(PET), 1_000);
    let mut now = 2_000;
    let trip = race(&mut gateway, &mut now);

    // Viewers: the zone change removes the actor through `nearby` (the old instance's fanout).
    // The loaded commit cannot fan out the new actor: the player is not back in the interest grid
    // until the join in that same commit, so the loaded AddNpc reaches viewers through the W177
    // `with_pets` replay on that join (`pet_view` is what it replays; F762's test proves it).
    assert_eq!(
        removals(&trip.started.nearby, actor),
        1,
        "viewers lose the pet"
    );
    assert_eq!(
        adds(&trip.loaded.replies, actor).len(),
        1,
        "the owner's actor"
    );
    assert!(gateway.pet_view().is_some(), "the presence viewers replay");

    // Walking after the warp sends the pet's own 125 frames to viewers again, under the new
    // actor. Frames are rate-bounded by `pets.json` follow.send_ms (500 ms).
    let mut moved = gateway.state().position;
    let mut frames = 0;
    for _ in 0..6 {
        moved[2] += 0.5;
        let out = send(&mut gateway, &move_to(GUID, moved), now);
        now += 200;
        for packet in out
            .nearby
            .iter()
            .map(|packet| gateway_packets::body(packet))
        {
            if packet.starts_with(&[125, 0]) && packet[2..10] == actor.to_le_bytes() {
                frames += 1;
            }
        }
    }
    assert!(frames > 0, "the pet's frames reach viewers after the warp");
}

#[test]
fn a_reconnect_forgets_the_pet_and_the_save_never_holds_it() {
    let content = Content::new();
    let mut gateway = session(&content);
    let actor = actor();
    send(&mut gateway, &summon(PET), 1_000);
    let saved = gateway.snapshot();
    let mut now = 2_000;
    // The warp remembers the pet and a kart race holds it back; the disconnect then happens with
    // that memory still pending (canary: keeping it across `reset_session` fails this test).
    let started = send(&mut gateway, &chat("realm race"), now);
    now += 1_000;
    assert!(started.began_zoning);
    assert_eq!(removals(&started.nearby, actor), 1);
    send(&mut gateway, &[13, 0], now);
    now += 100;
    send(&mut gateway, &[10, 0], now);
    now += 100;
    assert!(gateway.pet_view().is_none());

    // The warp is session state: the stored document still holds exactly one pet.
    let after = gateway.snapshot().pets.expect("the saved roster");
    assert_eq!(
        after["next_pet_id"],
        saved.pets.as_ref().unwrap()["next_pet_id"]
    );
    assert_eq!(
        after["pets"].as_array().map(Vec::len),
        Some(1),
        "no second pet"
    );

    // The reconnect teardown drops the session, and the next login never resummons.
    gateway.reset_session();
    let again = gateway.handle_at(&login(GUID), now).unwrap();
    assert!(!again.replies.is_empty(), "the login replies");
    send(&mut gateway, &[13, 0], now + 100);
    let loaded = send(&mut gateway, &[10, 0], now + 200);
    assert!(
        adds(&loaded.replies, actor).is_empty(),
        "a reconnect does not resummon"
    );
    assert!(gateway.pet_view().is_none());

    // A fresh session from the save is the same: the memory was never in the document.
    let mut fresh = Gateway::load_gameplay(
        &content.reference(),
        gateway.state().clone(),
        &content.0,
        gateway.snapshot().gameplay,
    )
    .unwrap();
    fresh.adopt_pets(gateway.snapshot().pets).unwrap();
    fresh.handle_at(&login(GUID), now + 300).unwrap();
    send(&mut fresh, &[13, 0], now + 400);
    let loaded = send(&mut fresh, &[10, 0], now + 500);
    assert!(adds(&loaded.replies, actor).is_empty());
    assert!(fresh.pet_view().is_none());
}

#[test]
fn the_first_tick_after_a_destination_that_skips_the_loaded_stream_brings_the_pet_back() {
    // A house lot's loaded stream is its own branch (it sends no world entry), so the pet a warp
    // remembers arrives on the next world tick. The production hook is `Gameplay::pet_tick`.
    let content = Content::new();
    let mut game = content.game();
    game.adopt_pets(roster()).unwrap();
    game.initialize_world(&who(), 0).unwrap();
    let actor = actor();
    let summoned = game.handle_in_world(&summon(PET), &who(), 1_000).unwrap();
    assert_eq!(adds(&summoned.replies, actor).len(), 1);
    let ended = game.settle_pets(&who(), true, 1_100).unwrap();
    assert!(ended.changed);
    assert_eq!(removals(&ended.nearby, actor), 1);
    assert!(game.pet_view(GUID).is_none());

    let out = game
        .pet_tick(&who(), 1_200)
        .expect("the tick brings the pet back");
    assert_eq!(adds(&out.replies, actor).len(), 1, "owner actor");
    assert_eq!(adds(&out.nearby, actor).len(), 1, "viewer actor");
    assert_eq!(game.pet_view(GUID).map(|view| view.pet_id), Some(PET));
    // One pet only: the next tick adds nothing.
    let quiet = game.pet_tick(&who(), 1_300).expect("a pet is out");
    assert!(adds(&quiet.replies, actor).is_empty());
    assert!(adds(&quiet.nearby, actor).is_empty());
}

#[test]
fn a_pets_gate_holds_the_memory_and_a_summon_replaces_it() {
    let content = PetService::bundled_content().unwrap();
    let definition = content.definition(SPECIES).unwrap();
    let mut pets = PetService::new(content.clone()).unwrap();
    pets.adopt_vendor_grant(GUID, &definition, 0).unwrap();
    let actor = actor();
    let busy = PetBusy {
        minigame_active: true,
    };
    let open = PetBusy {
        minigame_active: false,
    };
    pets.summon(PET, actor, GUID, OWNER, open, 1, Vec::new())
        .unwrap();
    assert_eq!(pets.remembered_pet_id(), None, "a summon remembers nothing");
    pets.zone_away(2).unwrap();
    assert!(pets.active_pet_id().is_none());
    assert_eq!(pets.remembered_pet_id(), Some(PET));

    // A closed pets gate keeps the memory for a later attempt and changes nothing else.
    let before = pets.document_value();
    assert!(
        pets.resummon(actor, GUID, OWNER, busy, 3, Vec::new())
            .unwrap()
            .is_none(),
        "a minigame refuses the resummon"
    );
    assert_eq!(pets.remembered_pet_id(), Some(PET), "kept for later");
    assert_eq!(pets.document_value(), before, "a refusal changes nothing");
    let wire = pets
        .resummon(actor, GUID, OWNER, open, 4, Vec::new())
        .unwrap()
        .expect("the pet comes back once the gate opens");
    assert_eq!(pets.active_pet_id(), Some(PET));
    assert_eq!(pets.remembered_pet_id(), None, "the memory is spent");
    // The wire order is the summon's: the AddNpc first (the client binds only an actor that
    // already exists, `00b54080`), then the 53/9 pair and the 53/5 roster; and no save is marked.
    assert_eq!(wire.shared.len(), 1, "one AddNpc to viewers");
    assert!(wire.private[0].starts_with(&[35, 0, 2, 0]));
    assert!(wire.private.iter().any(|packet| packet.get(2) == Some(&9)));
    assert!(wire.private.iter().any(|packet| packet.get(2) == Some(&5)));
    assert!(!wire.changed, "a resummon never marks a save");
    assert!(
        pets.resummon(actor, GUID, OWNER, open, 5, Vec::new())
            .unwrap()
            .is_none(),
        "the memory is spent"
    );

    // A summon replaces the memory; a later dismiss drops what is left, so nothing returns.
    pets.zone_away(6).unwrap();
    pets.summon(PET, actor, GUID, OWNER, open, 7, Vec::new())
        .unwrap();
    assert_eq!(pets.remembered_pet_id(), None, "a summon replaces it");
    pets.dismiss(8).unwrap();
    assert!(
        pets.resummon(actor, GUID, OWNER, open, 9, Vec::new())
            .unwrap()
            .is_none(),
        "a dismiss leaves nothing to bring back"
    );

    // A saved-and-reloaded roster carries no memory: the service starts without one.
    let reloaded =
        PetService::from_document(content, pets.document_value()).expect("a saved roster");
    assert_eq!(reloaded.remembered_pet_id(), None);

    // A pose the server cannot place is an error for the caller, which logs it; nothing returns,
    // the pending memory is dropped so it is not retried every tick, and the roster is untouched.
    pets.summon(PET, actor, GUID, OWNER, open, 10, Vec::new())
        .unwrap();
    pets.zone_away(11).unwrap();
    let before = pets.document_value();
    assert!(
        pets.resummon(actor, GUID, [f32::NAN; 3], open, 12, Vec::new())
            .is_err(),
        "an unusable owner pose refuses the summon"
    );
    assert!(
        pets.remembered_pet_id().is_none(),
        "a refused resummon is not retried on every tick"
    );
    assert!(pets.active_pet_id().is_none());
    assert_eq!(pets.document_value(), before, "the roster is untouched");
}
