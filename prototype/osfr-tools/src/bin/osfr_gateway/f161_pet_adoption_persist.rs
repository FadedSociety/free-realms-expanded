//! F161: a pet adoption spends an inventory certificate and adds a pet in one session commit.
//! Both halves must reach one atomic document (never two writes), and a refused replacement must
//! leave the document and the live session untouched. The requests below run through
//! `Session::dispatch_message`, the same staging/commit/save path a socket datagram takes.
use super::test_support::{self, Scene, Scratch, bin_test_target};
use super::*;
use osfr_tools::inventory::InventoryItem;

#[path = "../../../tests/support/pet_wire.rs"]
pub(crate) mod pet_wire;

/// Bundled `pets.json`: species 1000015 (Char) is adopted from item 70352.
pub(crate) const ADOPTION_ITEM: i32 = 70_352;
const PET_SPECIES: u32 = 1_000_015;
/// The Wolf item (15493) with its model 3284: a Type-2 item no wiki species row lists, so its species
/// id is the item id and its name and texture come from the item row. It is defined in this pack's
/// `inventory.json` (Gloria's Crab 7885 is not), so the real gateway can grant and adopt it.
const WOLF_ITEM: i32 = 15_493;
const WOLF_MODEL: i32 = 3_284;

/// A selected, world-loaded session, as `Session::new` + the handshake leave it.
fn world_loaded(options: &Options, id: u32) -> Scene {
    let mut scene = Scene::new(options, id);
    scene.session.gateway.handle(&wrap(&[13, 0])).unwrap();
    scene.session.gateway.handle(&wrap(&[10, 0])).unwrap();
    scene
}

/// Add one owned adoption certificate to the live inventory, as a purchase or reward would.
pub(crate) fn grant_certificate(session: &mut Session) {
    grant_certificate_for(session, ADOPTION_ITEM);
}

/// Add one owned adoption certificate for `definition` to the live inventory, as a purchase or
/// reward would (F833 reuses this for the lab in the gateway late-viewer test).
pub(crate) fn grant_certificate_for(session: &mut Session, definition: i32) {
    let mut state = session
        .gateway
        .inventory_state()
        .expect("a content session owns an inventory")
        .clone();
    let id = state.items.iter().map(|item| item.id).max().unwrap_or(0) + 1_000;
    state.items.push(InventoryItem {
        definition_id: definition,
        tint: 0,
        id,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i32::MAX as i64,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    state.next_instance_id = Some(i64::from(id) + 1);
    session.gateway.adopt_trade_inventory(state).unwrap();
}

fn owns_definition(session: &Session, definition: i32) -> bool {
    session.gateway.inventory_state().is_some_and(|state| {
        state
            .items
            .iter()
            .any(|item| item.definition_id == definition)
    })
}

fn owns_certificate(session: &Session) -> bool {
    owns_definition(session, ADOPTION_ITEM)
}

/// A pose that the movement reader accepts, so the certificate reaches disk before adoption.
fn pose(session: &Session) -> Vec<u8> {
    let state = session.gateway.state();
    let mut bytes = vec![125, 0];
    bytes.extend(state.guid.to_le_bytes());
    for value in [
        state.position[0] + 0.5,
        state.position[1],
        state.position[2],
    ]
    .into_iter()
    .chain(state.rotation)
    {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([2, 255]);
    bytes
}

fn document(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn carries_certificate(document: &serde_json::Value, defined: i32) -> bool {
    document["gameplay"]["inventory"]["items"]
        .as_array()
        .is_some_and(|items| {
            items
                .iter()
                .any(|item| item["definition_id"].as_i64() == Some(i64::from(defined)))
        })
}

#[test]
fn a_spent_certificate_and_its_pet_survive_a_restart_together() {
    if !bin_test_target("f161") {
        return;
    }
    let scratch = Scratch::new("f161-adopt");
    let state = scratch.state();
    let options = test_support::options(state.clone());
    let mut scene = world_loaded(&options, 611);
    grant_certificate(&mut scene.session);
    let movement = pose(&scene.session);
    scene
        .send(&options, &movement, 10_000)
        .expect("the pose commits");
    let before = document(&state);
    assert!(
        carries_certificate(&before, ADOPTION_ITEM),
        "the certificate must be on disk before adoption"
    );

    let adopt = pet_wire::summon_by_item(ADOPTION_ITEM as u32, 0);
    scene
        .send(&options, &adopt, 11_000)
        .expect("adoption commits");
    assert!(
        !owns_certificate(&scene.session),
        "the certificate is spent"
    );
    let adopted = scene
        .session
        .gateway
        .pet_view()
        .expect("the adopted pet is out");
    assert_eq!(adopted.species_id, PET_SPECIES);

    // One document carries both halves: the consumed inventory and the new roster.
    let saved = document(&state);
    let roster = saved["pets"]["pets"]
        .as_array()
        .expect("the roster is in the same document");
    assert_eq!(roster.len(), 1);
    assert_eq!(roster[0]["species_id"], PET_SPECIES);
    assert!(
        !carries_certificate(&saved, ADOPTION_ITEM),
        "the consumed certificate lands in the same atomic write"
    );
    let pet_id = roster[0]["id"].as_u64().unwrap() as u32;

    // Restart exactly as the gateway does: a new session with fresh shared containers loads the
    // same save and roster, and only then may the character summon the pet by id.
    drop(scene);
    let mut scene = world_loaded(&options, 612);
    assert!(
        !owns_certificate(&scene.session),
        "the spent certificate must stay spent across the restart"
    );
    assert!(
        scene.session.gateway.pet_view().is_none(),
        "the active actor is session state, not save state"
    );
    scene
        .send(&options, &pet_wire::summon(pet_id), 20_000)
        .expect("the restored roster accepts a summon");
    assert_eq!(
        scene.session.gateway.pet_view().map(|pet| pet.species_id),
        Some(PET_SPECIES)
    );
}

/// T454/F759: an item-only pet (the Wolf item) has no wiki species row, so its species id is the
/// item id and its display name and texture come from the item row. The real gateway path must
/// adopt it, persist the species and its model tricks, and still name it after a reconnect.
#[test]
fn an_item_only_pet_pal_survives_a_restart_with_its_name() {
    if !bin_test_target("f161") {
        return;
    }
    let scratch = Scratch::new("f161-wolf");
    let state = scratch.state();
    let options = test_support::options(state.clone());
    let mut scene = world_loaded(&options, 641);
    grant_certificate_for(&mut scene.session, WOLF_ITEM);
    let movement = pose(&scene.session);
    scene
        .send(&options, &movement, 10_000)
        .expect("the pose commits");
    assert!(carries_certificate(&document(&state), WOLF_ITEM));

    scene
        .send(
            &options,
            &pet_wire::summon_by_item(WOLF_ITEM as u32, 0),
            11_000,
        )
        .expect("adoption commits");
    assert!(
        !owns_definition(&scene.session, WOLF_ITEM),
        "the Wolf item is spent"
    );
    let adopted = scene
        .session
        .gateway
        .pet_view()
        .expect("the adopted pet is out");
    assert_eq!(adopted.species_id, WOLF_ITEM as u32);
    assert_eq!(adopted.model_id, WOLF_MODEL);
    assert_eq!(adopted.name, "Wolf");
    assert_eq!(adopted.texture_alias, "base");

    let saved = document(&state);
    let roster = saved["pets"]["pets"]
        .as_array()
        .expect("the roster is in the same document");
    assert_eq!(roster.len(), 1);
    assert_eq!(roster[0]["species_id"], serde_json::json!(WOLF_ITEM));
    assert_eq!(
        roster[0]["adoption_item_definition_id"],
        serde_json::json!(WOLF_ITEM)
    );
    assert_eq!(
        roster[0]["learned_tricks"],
        serde_json::json!([2057, 2058, 2059]),
        "the model's known tricks persist with the pet"
    );
    let pet_id = roster[0]["id"].as_u64().unwrap() as u32;

    drop(scene);
    let mut scene = world_loaded(&options, 642);
    scene
        .send(&options, &pet_wire::summon(pet_id), 20_000)
        .expect("the restored roster accepts a summon");
    let restored = scene
        .session
        .gateway
        .pet_view()
        .expect("the restored pet is out");
    assert_eq!(restored.species_id, WOLF_ITEM as u32);
    assert_eq!(
        restored.name, "Wolf",
        "the saved adoption item names the pet after the reconnect"
    );
    assert_eq!(restored.texture_alias, "base");
}

/// S17 (17c): the roster rides in the one character document. The adoption's file inventory and
/// roster equal the live session's, a refused pet request writes nothing (not even a deferred
/// pose), and the later pose flush keeps the roster.
#[test]
fn the_roster_rides_in_every_document_save_and_a_refusal_writes_nothing() {
    if !bin_test_target("f161") {
        return;
    }
    let scratch = Scratch::new("f161-document");
    let state = scratch.state();
    let options = test_support::options(state.clone());
    let mut scene = world_loaded(&options, 631);
    grant_certificate(&mut scene.session);
    let movement = pose(&scene.session);
    scene
        .send(&options, &movement, 10_000)
        .expect("the pose commits");

    let adopt = pet_wire::summon_by_item(ADOPTION_ITEM as u32, 0);
    scene
        .send(&options, &adopt, 11_000)
        .expect("adoption commits");
    let adopted = document(&state);
    let live = serde_json::to_value(scene.snapshot()).unwrap();
    assert!(!carries_certificate(&adopted, ADOPTION_ITEM));
    assert_eq!(
        adopted["gameplay"]["inventory"],
        live["gameplay"]["inventory"]
    );
    assert_eq!(adopted["pets"], live["pets"]);
    assert_eq!(adopted["pets"]["pets"].as_array().map(Vec::len), Some(1));

    // A pose inside POSITION_SAVE_MS is deferred; a refused pet request must not write it.
    let movement = pose(&scene.session);
    scene
        .send(&options, &movement, 12_000)
        .expect("the pose is accepted");
    let bytes = fs::read(&state).unwrap();
    assert_eq!(
        document(&state)["character"],
        adopted["character"],
        "the pose is deferred"
    );
    scene
        .send(&options, &adopt, 12_500)
        .expect("a spent certificate is refused with a reply, not an error");
    assert_eq!(
        fs::read(&state).unwrap(),
        bytes,
        "a refused pet request wrote the document"
    );
    assert_eq!(
        serde_json::to_value(scene.snapshot()).unwrap()["pets"],
        adopted["pets"]
    );

    scene.tick(&options, 17_000);
    let flushed = document(&state);
    assert_ne!(
        flushed["character"]["position"], adopted["character"]["position"],
        "the tick flushed the deferred pose"
    );
    assert_eq!(
        flushed["pets"], adopted["pets"],
        "the pose flush kept the roster"
    );
}

/// A concurrent Windows reader without `FILE_SHARE_DELETE` makes the atomic rename fail after the
/// document was read and built. On Unix a read-only directory, not a file handle, decides the
/// rename, so the injection has no equivalent.
#[cfg(windows)]
#[test]
fn a_refused_adoption_save_leaves_the_document_and_the_live_session_unchanged() {
    use std::os::windows::fs::OpenOptionsExt;

    if !bin_test_target("f161") {
        return;
    }
    let scratch = Scratch::new("f161-adopt-refused");
    let state = scratch.state();
    let options = test_support::options(state.clone());
    let mut scene = world_loaded(&options, 621);
    grant_certificate(&mut scene.session);
    let movement = pose(&scene.session);
    scene
        .send(&options, &movement, 10_000)
        .expect("the pose commits");
    let before = fs::read(&state).unwrap();
    let live_before = serde_json::to_value(scene.session.gateway.snapshot()).unwrap();

    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(1) // FILE_SHARE_READ only: the save's own read still works, the rename does not.
        .open(&state)
        .unwrap();
    // The save reads the document before it replaces it; that read must still succeed here, so
    // the refusal below is the replacement (rename), not a failure to read.
    assert_eq!(fs::read(&state).unwrap(), before);
    let adopt = pet_wire::summon_by_item(ADOPTION_ITEM as u32, 0);
    let refused = scene.send(&options, &adopt, 11_000);
    drop(held);
    assert!(
        refused.is_err(),
        "a blocked replacement must refuse the adoption"
    );
    assert_eq!(
        fs::read(&state).unwrap(),
        before,
        "the refused save leaves the document byte-identical"
    );
    assert!(
        owns_certificate(&scene.session),
        "the certificate is not spent by a refused commit"
    );
    assert!(
        scene.session.gateway.pet_view().is_none(),
        "a refused commit leaves no live pet actor"
    );
    assert_eq!(
        serde_json::to_value(scene.session.gateway.snapshot()).unwrap(),
        live_before,
        "live gameplay is unchanged by a refused commit"
    );
    // The live roster is still empty: the document value every save would write is `None`.
    assert_eq!(
        scene.snapshot().pets,
        None,
        "a refused commit leaves no live roster entry"
    );

    // Production ends the sender's session when `commit_outcome` returns Err (`Server::step` ->
    // `contain` -> `end_session`), so the client's retry is a reconnect, not the same session.
    // Drop the failed scene and load the same save in a fresh one, as the restart test does.
    drop(scene);
    let mut scene = world_loaded(&options, 622);
    assert!(
        owns_certificate(&scene.session),
        "the refused commit left the certificate on disk for the reconnect"
    );
    scene
        .send(&options, &adopt, 12_000)
        .expect("the same request commits after the reconnect");
    assert!(!owns_certificate(&scene.session));
    assert_eq!(
        scene.session.gateway.pet_view().map(|pet| pet.species_id),
        Some(PET_SPECIES)
    );
}
