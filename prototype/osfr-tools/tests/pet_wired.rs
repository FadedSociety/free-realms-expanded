//! W177 patch proof: the saved pet roster and active actor through `Gameplay` and `Gateway`.
//! Drop this in as `tests/pet_wired.rs` after `apply_shared_patch.py`; it does not compile before
//! (`Gameplay::adopt_pets`, `handle_pets`, `settle_pets`, `Gateway::pet_view`).
//! F778: the gateway tick's save flags and a whole-document save/reload of a feed's apply.
#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "support/json.rs"]
mod json;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;

use gateway_packets::{body, wrap};
use osfr_tools::{
    character_store::CharacterState,
    game_store,
    gameplay::Gameplay,
    gateway::Gateway,
    pets::{PetService, pet_removal_packet},
    world_events::piece_052::animation_packet,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const PET_SPECIES: u32 = 1_000_015; // Char; bundled `pets.json` has model + adoption item 70352.
const NOW: u64 = 10_000;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

fn reference() -> PathBuf {
    lab().join("references/rust-opening-cooking-original-assets")
}

fn who() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}

struct Save {
    directory: PathBuf,
    path: PathBuf,
}

impl Save {
    fn new() -> Self {
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("osfr-pet-wired-{}-{sequence}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("save.json");
        let content = PetService::bundled_content().unwrap();
        let mut pets = PetService::new(content.clone()).unwrap();
        let definition = content.definition(PET_SPECIES).unwrap();
        pets.adopt_vendor_grant(1, &definition, 0).unwrap();
        save_roster(&path, pets.document_value());
        Self { directory, path }
    }
}

impl Drop for Save {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_dir(&self.directory);
    }
}

/// The production document write: `Snapshot::pets` rides in the one `game_store::save`.
fn save_roster(path: &Path, pets: Option<serde_json::Value>) {
    let snapshot = game_store::Snapshot {
        character: who(),
        gameplay: None,
        pets,
    };
    game_store::save(path, &snapshot).unwrap();
}

fn saved_roster(path: &Path) -> Option<serde_json::Value> {
    game_store::load_or_create(path, who()).unwrap().pets
}

fn game(path: &Path) -> Gameplay {
    let mut game = Gameplay::load(&reference(), None).unwrap();
    game.adopt_pets(saved_roster(path)).unwrap();
    game.initialize_world(&who(), 0).unwrap();
    game
}

fn summon(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![1]].concat()
}

fn name_request(name: &str) -> Vec<u8> {
    [
        vec![53, 0, 6],
        (name.len() as u32).to_le_bytes().to_vec(),
        name.as_bytes().to_vec(),
    ]
    .concat()
}

fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["w177-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if index == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

fn has(packets: &[Vec<u8>], family: u16, subtype: u8) -> bool {
    packets.iter().any(|packet| {
        packet.get(..2) == Some(&family.to_le_bytes()) && packet.get(2) == Some(&subtype)
    })
}

/// The kind word of the first 53/20 PetUtilityNotify in `packets` (tunnel-unwrapped).
fn notify_kind(packets: &[Vec<u8>]) -> Option<u32> {
    packets
        .iter()
        .find(|packet| body(packet).starts_with(&[53, 0, 20]))
        .map(|packet| u32::from_le_bytes(body(packet)[7..11].try_into().unwrap()))
}

fn actor_packet(packets: &[Vec<u8>], actor: u64) -> bool {
    packets
        .iter()
        .any(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == actor.to_le_bytes())
}

fn removal_packet(packets: &[Vec<u8>], actor: u64) -> bool {
    packets
        .iter()
        .any(|packet| packet == &pet_removal_packet(actor))
}

#[test]
fn an_owned_saved_pet_is_listed_summoned_seen_nearby_and_dismissed() {
    let save = Save::new();
    let mut game = game(&save.path);
    let actor = PetService::pet_guid(who().guid);

    let loaded = game.initialize_world(&who(), NOW).unwrap();
    assert!(has(&loaded.replies, 53, 5), "pet roster at world entry");

    let summoned = game.handle_in_world(&summon(1), &who(), NOW + 1).unwrap();
    assert!(has(&summoned.replies, 53, 9), "active pair");
    assert!(actor_packet(&summoned.replies, actor), "owner actor");
    assert!(actor_packet(&summoned.nearby, actor), "neighbour actor");
    assert_eq!(game.pet_view(1).unwrap().pet_id, 1);

    let dismissed = game
        .handle_in_world(&[53, 0, 4, 0, 0, 0, 0, 0], &who(), NOW + 2)
        .unwrap();
    assert!(has(&dismissed.replies, 53, 9));
    assert!(removal_packet(&dismissed.nearby, actor));
    assert!(game.pet_view(1).is_none());
}

#[test]
fn foreign_pet_and_unowned_item_requests_leave_gameplay_unchanged() {
    let save = Save::new();
    let mut game = game(&save.path);
    let before = game.state();

    let foreign = game.handle_in_world(&summon(999), &who(), NOW).unwrap();
    assert!(!foreign.changed);
    assert!(has(&foreign.replies, 53, 5), "roster refresh");
    assert!(
        foreign
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[15, 0, 3, 0]))
    );
    assert!(foreign.nearby.is_empty());
    assert_eq!(game.state(), before);
    assert!(game.pet_view(1).is_none());

    let foreign_item = [
        vec![53, 0, 32],
        1u32.to_le_bytes().to_vec(),
        10u32.to_le_bytes().to_vec(),
    ]
    .concat();
    let refused = game
        .handle_in_world(&foreign_item, &who(), NOW + 1)
        .unwrap();
    assert!(!refused.changed);
    assert!(
        refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[15, 0, 3, 0]))
    );
    assert!(refused.nearby.is_empty());
    assert_eq!(game.state(), before);
    assert!(game.pet_view(1).is_none());
}

#[test]
fn zoning_dismisses_without_persisting_the_actor_and_reload_keeps_roster() {
    let save = Save::new();
    let mut game = game(&save.path);
    let actor = PetService::pet_guid(who().guid);
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    assert!(game.pet_view(1).is_some());
    let ended = game.settle_pets(&who(), true, NOW + 1).unwrap();
    assert!(ended.changed);
    assert!(has(&ended.replies, 53, 9));
    assert!(removal_packet(&ended.nearby, actor));
    assert!(game.pet_view(1).is_none());

    game.reset_session();
    let mut reloaded = Gameplay::load(&reference(), None).unwrap();
    reloaded.adopt_pets(saved_roster(&save.path)).unwrap();
    let list = reloaded.initialize_world(&who(), NOW + 2).unwrap();
    assert!(has(&list.replies, 53, 5));
    assert!(reloaded.pet_view(1).is_none());
}

#[test]
fn an_accepted_name_is_written_to_the_top_level_pet_payload() {
    let save = Save::new();
    let mut game = game(&save.path);
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    let renamed = game
        .handle_in_world(&name_request("Admiral Ace"), &who(), NOW + 1)
        .unwrap();
    assert!(renamed.changed);
    save_roster(&save.path, game.pets_document());
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(&save.path).unwrap()).unwrap();
    assert_eq!(document["pets"]["pets"][0]["name"], "Admiral Ace");

    let mut reloaded = Gameplay::load(&reference(), None).unwrap();
    reloaded.adopt_pets(saved_roster(&save.path)).unwrap();
    assert!(has(
        &reloaded.initialize_world(&who(), NOW + 2).unwrap().replies,
        53,
        5
    ));
}

#[test]
fn the_gateway_exposes_the_saved_pet_roster_and_clears_the_actor_on_zoning() {
    let save = Save::new();
    let content = lab().join("references/rust-opening-cooking-original-assets");
    let reference = lab().join("references/rust-gateway-reference");
    let state = Gameplay::load(&content, None).unwrap().state();
    let mut gateway = Gateway::load_gameplay(&reference, who(), &content, Some(state)).unwrap();
    gateway.adopt_pets(saved_roster(&save.path)).unwrap();
    gateway.handle_at(&login(), NOW).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), NOW + 1).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), NOW + 2).unwrap();
    let summoned = gateway.handle_at(&wrap(&summon(1)), NOW + 3).unwrap();
    assert!(summoned.nearby.iter().any(|packet| actor_packet(
        std::slice::from_ref(packet),
        PetService::pet_guid(who().guid)
    )));
    assert!(gateway.pet_view().is_some());
    let ended = gateway.settle_pets(true, NOW + 4).unwrap();
    assert!(ended.state_changed && ended.gameplay_changed);
    assert!(
        ended
            .nearby
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 3, 0, 1, 0]))
    );
    assert!(gateway.pet_view().is_none());
}

/// T455: frames for the pet's own actor (opcode 125).
fn pet_moves(packets: &[Vec<u8>], actor: u64) -> usize {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[125, 0]) && packet[2..10] == actor.to_le_bytes())
        .count()
}

#[test]
fn a_walking_owner_moves_the_summoned_pet_for_viewers_without_a_save() {
    let save = Save::new();
    let mut game = game(&save.path);
    let actor = PetService::pet_guid(who().guid);
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    let mut owner = who();
    let (mut to_viewers, mut to_owner) = (0, 0);
    for step in 1..=20u64 {
        owner.position[2] += 0.5;
        let out = game.on_location(&owner, NOW + step * 100).unwrap();
        assert!(!out.changed, "following never marks a save");
        to_viewers += pet_moves(&out.nearby, actor);
        to_owner += pet_moves(&out.replies, actor);
    }
    assert!(
        to_viewers > 0 && to_viewers == to_owner,
        "{to_viewers} / {to_owner}"
    );
    // The world tick settles the pet beside a still owner: one stop frame, no save.
    let mut settled = 0;
    for step in 21..=60u64 {
        let out = game
            .pet_tick(&owner, NOW + step * 100)
            .expect("a pet is out");
        assert!(!out.changed);
        settled += pet_moves(&out.nearby, actor);
    }
    // Walk and stop frames share the wire layout; the pure test pins exactly one stop frame.
    assert!(
        (1..=4).contains(&settled),
        "{settled} frames while settling"
    );
    for step in 61..=80u64 {
        let out = game.pet_tick(&owner, NOW + step * 100).unwrap();
        assert_eq!(
            pet_moves(&out.nearby, actor),
            0,
            "a settled pet stays quiet"
        );
    }
    // A late viewer is shown the pet where it stands, not on its owner.
    let view = game.pet_view(who().guid).unwrap();
    let gap = ((view.position[0] - owner.position[0]).powi(2)
        + (view.position[2] - owner.position[2]).powi(2))
    .sqrt();
    assert!(
        (1.0..=3.0).contains(&gap),
        "pet {:?} owner {:?}",
        view.position,
        owner.position
    );
}

#[test]
fn the_world_tick_shows_a_summoned_pets_live_needs_without_a_save() {
    let save = Save::new();
    let mut game = game(&save.path);
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    let out = game.pet_tick(&who(), NOW + 100).expect("a pet is out");
    assert!(has(&out.replies, 53, 3), "T485 live 53/3 needs");
    assert!(!out.changed, "live needs never mark a save");
}

#[test]
fn no_pet_means_no_tick_and_a_dismissed_pet_sends_no_moves() {
    let save = Save::new();
    let mut game = game(&save.path);
    assert!(game.pet_tick(&who(), NOW).is_none());
    let actor = PetService::pet_guid(who().guid);
    game.handle_in_world(&summon(1), &who(), NOW).unwrap();
    game.handle_in_world(&[53, 0, 4, 0, 0, 0, 0, 0], &who(), NOW + 1)
        .unwrap();
    let mut owner = who();
    owner.position[2] += 5.0;
    let out = game.on_location(&owner, NOW + 2_000).unwrap();
    assert_eq!(pet_moves(&out.nearby, actor), 0);
    assert!(game.pet_tick(&owner, NOW + 3_000).is_none());
}

/// The opening package's catalogue: bowl 8 (slot 204) and Simple Pet Food 1835 (slot 205), both
/// type 4 and class 73, so the bundled Char (model class 73) takes them.
const BOWL_DEF: i32 = 8;
const FOOD_DEF: i32 = 1_835;
const BOWL_INSTANCE: i32 = 9_001;
const FOOD_INSTANCE: i32 = 9_002;

/// A saved roster whose only pet has `(utility slot, item instance)` rows fitted.
fn fitted_roster(utilities: &[(usize, i32)]) -> serde_json::Value {
    let content = PetService::bundled_content().unwrap();
    let mut pets = PetService::new(content.clone()).unwrap();
    let definition = content.definition(PET_SPECIES).unwrap();
    assert_eq!(pets.adopt_vendor_grant(1, &definition, 0).unwrap(), 1);
    let mut document = pets.document_value().unwrap();
    for &(slot, instance) in utilities {
        document["pets"][0]["equipment"]["utilities"][slot] = serde_json::json!(instance);
    }
    document
}

/// The opening package's kit rows: shampoo 1836 (slot 200) and brush 56778 (slot 203), both
/// type 4 class 73, so the bundled Char takes them.
const SHAMPOO_DEF: i32 = 1_836;
const BRUSH_DEF: i32 = 56_778;
const SHAMPOO_INSTANCE: i32 = 9_003;
const BRUSH_INSTANCE: i32 = 9_004;

/// A logged-in gateway at the opening scene whose bag holds the grooming kit and whose summoned
/// Char has it fitted.
fn groom_gateway() -> Gateway {
    let content = lab().join("references/rust-opening-cooking-original-assets");
    let reference = lab().join("references/rust-gateway-reference");
    let mut state = Gameplay::load(&content, None).unwrap().state();
    state
        .inventory
        .items
        .push(pet_inventory::item(SHAMPOO_INSTANCE, SHAMPOO_DEF, 2));
    state
        .inventory
        .items
        .push(pet_inventory::item(BRUSH_INSTANCE, BRUSH_DEF, 1));
    let mut gateway = Gateway::load_gameplay(&reference, who(), &content, Some(state)).unwrap();
    gateway
        .adopt_pets(Some(fitted_roster(&[
            (1, SHAMPOO_INSTANCE),
            (4, BRUSH_INSTANCE),
        ])))
        .unwrap();
    gateway.handle_at(&login(), NOW).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), NOW + 1).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), NOW + 2).unwrap();
    let summoned = gateway.handle_at(&wrap(&summon(1)), NOW + 3).unwrap();
    assert!(
        summoned
            .replies
            .iter()
            .any(|packet| body(packet).starts_with(&[53, 0, 9])),
        "the pet is out"
    );
    gateway
}

/// One number from the shipped `care.groom` rules.
fn groom_rule(key: &str) -> i64 {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/pets.json");
    json::read(&path)["care"]["groom"][key].as_i64().unwrap()
}

/// The configured dry-off (35/8) animation for `actor`.
fn dry_off(actor: u64) -> Vec<u8> {
    animation_packet(actor, groom_rule("dry_off_animation") as i32, false)
}

/// A feed that started in the opening scene applies through the gateway tick, marks both save
/// flags, and the eaten food and raised need survive a whole-document save and reload (F778).
#[test]
fn a_feed_apply_sets_both_save_flags_and_survives_a_whole_document_reload() {
    let save = Save::new();
    let content = lab().join("references/rust-opening-cooking-original-assets");
    let reference = lab().join("references/rust-gateway-reference");
    let mut state = Gameplay::load(&content, None).unwrap().state();
    state
        .inventory
        .items
        .push(pet_inventory::item(BOWL_INSTANCE, BOWL_DEF, 1));
    state
        .inventory
        .items
        .push(pet_inventory::item(FOOD_INSTANCE, FOOD_DEF, 2));
    let mut gateway = Gateway::load_gameplay(&reference, who(), &content, Some(state)).unwrap();
    gateway
        .adopt_pets(Some(fitted_roster(&[
            (5, BOWL_INSTANCE),
            (6, FOOD_INSTANCE),
        ])))
        .unwrap();
    gateway.handle_at(&login(), NOW).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), NOW + 1).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), NOW + 2).unwrap();
    let summoned = gateway.handle_at(&wrap(&summon(1)), NOW + 3).unwrap();
    assert!(
        summoned
            .replies
            .iter()
            .any(|packet| body(packet).starts_with(&[53, 0, 9])),
        "the pet is out"
    );
    let started = gateway
        .handle_at(&wrap(&pet_wire::feed(1)), NOW + 4)
        .unwrap();
    assert_eq!(notify_kind(&started.replies), Some(1), "the feed begins");
    // F778: this is the opening scene's tick branch, which used to run `gameplay.tick` alone, so
    // the feed never applied there.
    let applied = gateway.tick_at(NOW + 1_000).unwrap();
    assert!(
        applied
            .replies
            .iter()
            .any(|packet| body(packet).starts_with(&[53, 0, 3])),
        "the apply sends the raised needs"
    );
    assert!(
        applied.state_changed && applied.gameplay_changed,
        "the apply marks both save flags"
    );
    let live = gateway.gameplay().unwrap().inventory_state();
    assert_eq!(
        live.items
            .iter()
            .find(|item| item.id == FOOD_INSTANCE)
            .map(|item| item.count),
        Some(1),
        "one food is eaten"
    );

    game_store::save(&save.path, &gateway.snapshot()).unwrap();
    let document = game_store::load_or_create(&save.path, who()).unwrap();
    {
        let gameplay = document.gameplay.as_ref().expect("the gameplay document");
        assert_eq!(
            gameplay
                .inventory
                .items
                .iter()
                .find(|item| item.id == FOOD_INSTANCE)
                .map(|item| item.count),
            Some(1),
            "the eaten food is in the saved document"
        );
        let pets = document.pets.as_ref().expect("the roster is saved");
        assert_eq!(
            pets["pets"][0]["needs"]["food"].as_f64(),
            Some(100.0),
            "the raised food need is saved"
        );
        assert_eq!(
            pets["pets"][0]["equipment"]["utilities"][6],
            serde_json::json!(FOOD_INSTANCE),
            "the fitted food stays in the saved roster"
        );
    }
    let mut reloaded = Gameplay::load(&content, document.gameplay).unwrap();
    reloaded.adopt_pets(document.pets).unwrap();
    assert_eq!(
        reloaded
            .inventory_state()
            .items
            .iter()
            .find(|item| item.id == FOOD_INSTANCE)
            .map(|item| item.count),
        Some(1),
        "the reloaded session keeps the food consumed"
    );
}

/// T488/F796: the wrappers around a grooming session that the service suite drives directly. A
/// stroke that fills the bar ends through `handle_pets`/`pet_outcome` (kind 4 to the owner and the
/// dry-off to viewers); a timeout ends through the world tick's `pet_tick` (`nearby` dry-off); and
/// a zone change ends it through `settle_pets` (kind 4).
#[test]
fn a_groom_ends_through_the_gameplay_fanout_tick_and_zoning_with_kind_4() {
    let actor = PetService::pet_guid(who().guid);
    let dry_off = dry_off(actor);

    // Full bar: brisk strokes fill the bar through the request path (200 is well over the shipped
    // 100-point bar at the shipped cap; the loop stops at the first end).
    let mut gateway = groom_gateway();
    let started = gateway
        .handle_at(&wrap(&pet_wire::groom(1)), NOW + 4)
        .unwrap();
    assert_eq!(notify_kind(&started.replies), Some(3), "the session starts");
    let mut ended = None;
    for step in 1..=200u64 {
        let wire = gateway
            .handle_at(
                &wrap(&pet_wire::tactile(2, 300, 15_000)),
                NOW + 4 + step * 300,
            )
            .unwrap();
        if notify_kind(&wire.replies) == Some(4) {
            ended = Some(wire);
            break;
        }
    }
    let ended = ended.expect("the bar fills before the timeout");
    assert_eq!(notify_kind(&ended.replies), Some(4), "the session ends");
    assert!(
        ended
            .replies
            .iter()
            .any(|p| body(p).starts_with(&[15, 0, 3, 0])),
        "T812: a full bar also tells the owner in chat"
    );
    assert!(
        ended.replies.iter().any(|packet| body(packet) == dry_off),
        "the owner sees the dry-off"
    );
    assert!(
        ended.nearby.iter().any(|packet| *packet == dry_off),
        "viewers see the dry-off"
    );
    assert!(
        ended.state_changed && ended.gameplay_changed,
        "the end is saved"
    );
    assert!(gateway.pet_view().is_some(), "the pet stays out");

    // Timeout: the world tick ends the session and fans the dry-off to viewers.
    let mut gateway = groom_gateway();
    gateway
        .handle_at(&wrap(&pet_wire::groom(1)), NOW + 4)
        .unwrap();
    let after = NOW + 4 + groom_rule("seconds") as u64 * 1_000 + 1_000;
    let ticked = gateway.tick_at(after).unwrap();
    assert_eq!(notify_kind(&ticked.replies), Some(4), "the tick ends it");
    assert!(
        ticked.nearby.iter().any(|packet| *packet == dry_off),
        "the tick fans the dry-off to viewers"
    );
    assert!(ticked.state_changed && ticked.gameplay_changed);

    // Zoning: the session ends with kind 4 before the pet is dismissed.
    let mut gateway = groom_gateway();
    gateway
        .handle_at(&wrap(&pet_wire::groom(1)), NOW + 4)
        .unwrap();
    let zoned = gateway.settle_pets(true, NOW + 5).unwrap();
    assert_eq!(notify_kind(&zoned.replies), Some(4), "zoning ends it");
    assert!(zoned.state_changed && zoned.gameplay_changed);
    assert!(gateway.pet_view().is_none(), "the pet was left behind");
}
