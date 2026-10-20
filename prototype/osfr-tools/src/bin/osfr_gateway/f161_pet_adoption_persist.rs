//! F161: a pet adoption spends an inventory certificate and adds a pet in one session commit.
//! Both halves must reach one atomic document (never two writes), and a refused replacement must
//! leave the document and the live session untouched. The requests below run through
//! `Session::dispatch_message`, the same staging/commit/save path a socket datagram takes.
use super::test_support::{self, Scratch};
use super::*;
use osfr_tools::inventory::InventoryItem;

#[path = "pet_wire.rs"]
mod pet_wire;

/// Bundled `pets.json`: species 1000015 (Char) is adopted from item 70352.
const ADOPTION_ITEM: i32 = 70_352;
const PET_SPECIES: u32 = 1_000_015;

/// A selected, world-loaded session, as `Session::new` + the handshake leave it.
fn enter(options: &Options, id: u32) -> Session {
    let mut session = Session::new(
        SessionId(u64::from(id)),
        format!("127.0.0.1:{}", 23_000 + id).parse().unwrap(),
        SessionRequest {
            id,
            crc_length: 3,
            buffer_size: 512,
        },
        options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["f161-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle(&login).unwrap();
    session.gateway.handle(&wrap(&[13, 0])).unwrap();
    session.gateway.handle(&wrap(&[10, 0])).unwrap();
    session
}

/// One session and the shared containers a `Server` owns. Rebuilding it from `Options` is the
/// gateway's restart path: the session loads the save and the pet roster again.
struct Scene {
    session: Session,
    world: SharedWorld,
    membership: World,
    sessions: SessionMap,
}

impl Scene {
    fn new(options: &Options, id: u32) -> Self {
        Self {
            session: enter(options, id),
            world: SharedWorld::new(),
            membership: World::runtime(),
            sessions: SessionMap::new(),
        }
    }

    fn send(&mut self, options: &Options, inner: &[u8], now: u64) -> Result<(), String> {
        self.session
            .dispatch_message(
                &wrap(inner),
                options,
                now,
                &mut self.world,
                &mut self.membership,
                &mut self.sessions,
                None,
            )
            .map_err(|error| error.to_string())
    }
}

/// Add one owned adoption certificate to the live inventory, as a purchase or reward would.
fn grant_certificate(session: &mut Session) {
    let mut state = session
        .gateway
        .inventory_state()
        .expect("a content session owns an inventory")
        .clone();
    let id = state.items.iter().map(|item| item.id).max().unwrap_or(0) + 1_000;
    state.items.push(InventoryItem {
        definition_id: ADOPTION_ITEM,
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

fn owns_certificate(session: &Session) -> bool {
    session.gateway.inventory_state().is_some_and(|state| {
        state
            .items
            .iter()
            .any(|item| item.definition_id == ADOPTION_ITEM)
    })
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

/// The gateway source is embedded by five integration tests, which compile this module too.
/// `CARGO_BIN_NAME` is set only for the binary's own test target, so the scenario runs once
/// there. An embedded copy says so on stderr (libtest does not capture a direct fd write), so a
/// green integration-target line is not mistaken for a real run.
fn bin_test_target() -> bool {
    if option_env!("CARGO_BIN_NAME").is_some() {
        return true;
    }
    use std::io::Write as _;
    let _ = std::io::stderr().write_all(
        b"f161: embedded gateway copy skipped; run `cargo test --bin osfr-gateway f161_`\n",
    );
    false
}

#[test]
fn a_spent_certificate_and_its_pet_survive_a_restart_together() {
    if !bin_test_target() {
        return;
    }
    let scratch = Scratch::new("f161-adopt");
    let state = scratch.state();
    let options = test_support::options(state.clone());
    let mut scene = Scene::new(&options, 611);
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
    let mut scene = Scene::new(&options, 612);
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

/// A concurrent Windows reader without `FILE_SHARE_DELETE` makes the atomic rename fail after the
/// document was read and built. On Unix a read-only directory, not a file handle, decides the
/// rename, so the injection has no equivalent.
#[cfg(windows)]
#[test]
fn a_refused_adoption_save_leaves_the_document_and_the_live_session_unchanged() {
    use std::os::windows::fs::OpenOptionsExt;

    if !bin_test_target() {
        return;
    }
    let scratch = Scratch::new("f161-adopt-refused");
    let state = scratch.state();
    let options = test_support::options(state.clone());
    let mut scene = Scene::new(&options, 621);
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
    // The live roster is still empty, read back through the production combined save.
    let probe = scratch.path().join("live-roster.json");
    scene.session.gateway.save_with_pet_roster(&probe).unwrap();
    assert_eq!(
        document(&probe)["pets"]["pets"].as_array().unwrap().len(),
        0,
        "a refused commit leaves no live roster entry"
    );

    // Production ends the sender's session when `commit_outcome` returns Err (`Server::step` ->
    // `contain` -> `end_session`), so the client's retry is a reconnect, not the same session.
    // Drop the failed scene and load the same save in a fresh one, as the restart test does.
    drop(scene);
    let mut scene = Scene::new(&options, 622);
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
