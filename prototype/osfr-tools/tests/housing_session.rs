//! Lead housing slice 1 through the real `Gameplay`: 127/23 grants the starter lot and lists it
//! (127/39), 127/19 enters it (scene change now, house packets after the scene loads), saves and
//! reconnects keep the world pose, 127/10 returns to it, and refusals change nothing. The starter
//! is the Wilds Condo (property 25), and a character who already holds the old starter (property 1)
//! gains it once. Raw inner packets as the client sends them; no player data.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    gateway::Gateway,
    housing_native::{InstanceList, PlayerHousingInstanceInfo, ZoneData},
    housing_service::HousingContent,
    zone_transition,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const OWNER: u64 = 1;
static NEXT: AtomicU64 = AtomicU64::new(0);
const GRANT: [u8; 4] = [127, 0, 23, 0];
const LEAVE: [u8; 4] = [127, 0, 10, 0];
/// Locale name id of the Wilds Condo, the starter since 2012-01-26 (houses.json property 25).
const WILDS_CONDO: i32 = 296;

/// The opening reference content every gameplay route test loads (`OSFR_ACTIVITY_CONTENT`
/// overrides it), optionally with the repository's `houses.json` copied in.
fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_ACTIVITY_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

struct Content(PathBuf);

impl Content {
    fn with_houses() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-housing-session-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&reference_content(), &dir);
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/houses.json"),
            dir.join("houses.json"),
        )
        .unwrap();
        Self(dir)
    }

    /// The bundled lots with another starter: a save made before the Wilds Condo became it.
    fn with_starter(property_id: i32) -> Self {
        let content = Self::with_houses();
        let mut houses = HousingContent::bundled().unwrap();
        houses.starter_property_id = property_id;
        fs::write(
            content.0.join("houses.json"),
            serde_json::to_vec(&houses).unwrap(),
        )
        .unwrap();
        content
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

fn character() -> CharacterState {
    CharacterState {
        guid: OWNER,
        zone_id: 0,
        position: [-1904.0, -42.0, 420.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

fn enter(house_guid: u64) -> Vec<u8> {
    let mut inner = vec![127, 0, 19, 0];
    inner.extend(house_guid.to_le_bytes());
    inner.extend(0i32.to_le_bytes());
    inner.push(0);
    inner
}

fn list_request(owner: u64) -> Vec<u8> {
    let mut inner = vec![127, 0, 14, 0];
    inner.extend(owner.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
}

fn subtype(reply: &[u8]) -> Option<u16> {
    (reply.len() >= 4 && reply[..2] == [127, 0]).then(|| u16::from_le_bytes([reply[2], reply[3]]))
}

fn only_list(replies: &[Vec<u8>]) -> InstanceList {
    assert_eq!(replies.len(), 1, "one 127/39 reply");
    InstanceList::decode(&replies[0][2..]).expect("a well-formed 127/39")
}

fn granted(game: &mut Gameplay) -> PlayerHousingInstanceInfo {
    let out = game.handle_in_world(&GRANT, &character(), 1_000).unwrap();
    let list = only_list(&out.replies);
    assert_eq!(list.owner_guid, OWNER);
    assert_eq!(list.records.len(), 1);
    list.records[0].clone()
}

#[test]
fn the_first_grant_gives_the_starter_lot_once_and_lists_it() {
    let content = Content::with_houses();
    let mut game = content.game();
    assert!(game.state().housing.is_none(), "no lot before the first ask");
    let first = game.handle_in_world(&GRANT, &character(), 1_000).unwrap();
    assert!(first.changed, "the grant is saved");
    let lot = only_list(&first.replies).records[0].clone();
    assert_eq!(lot.owner_guid, OWNER);
    let starter = HousingContent::bundled().unwrap();
    let row = starter
        .lots
        .iter()
        .find(|row| row.property_id == starter.starter_property_id)
        .unwrap();
    assert_eq!(lot.icon_id, row.icon_id);
    assert_eq!(game.state().housing.unwrap().lots.len(), 1);

    let again = game.handle_in_world(&GRANT, &character(), 2_000).unwrap();
    assert!(!again.changed, "a repeated ask grants nothing");
    assert_eq!(only_list(&again.replies).records, vec![lot.clone()]);
    let listed = game.handle_in_world(&list_request(OWNER), &character(), 3_000).unwrap();
    assert_eq!(only_list(&listed.replies).records, vec![lot]);
}

#[test]
fn a_new_character_is_granted_the_wilds_condo() {
    let content = Content::with_houses();
    let lot = granted(&mut content.game());
    assert_eq!((lot.name_id, lot.icon_id), (WILDS_CONDO, 38460));
}

#[test]
fn an_owner_of_lot_one_gains_the_condo_once_and_keeps_lot_one() {
    let mut before = Content::with_starter(1).game();
    let lot_one = granted(&mut before);
    assert_ne!(lot_one.name_id, WILDS_CONDO);
    let content = Content::with_houses();
    let mut game = Gameplay::load(&content.0, Some(before.state())).unwrap();
    let first = game.handle_in_world(&GRANT, &character(), 2_000).unwrap();
    assert!(first.changed, "the condo is saved");
    let lots = only_list(&first.replies).records;
    assert_eq!(lots.len(), 2);
    assert!(lots.contains(&lot_one), "lot 1 is kept unchanged");
    assert!(lots.iter().any(|lot| lot.name_id == WILDS_CONDO));

    let saved = game.state();
    let again = game.handle_in_world(&GRANT, &character(), 3_000).unwrap();
    assert!(!again.changed, "a second ask grants nothing");
    assert_eq!(only_list(&again.replies).records, lots);
    assert_eq!(game.state(), saved);
}

#[test]
fn entering_changes_scene_and_the_house_packets_wait_for_the_load() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    let out = game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000).unwrap();
    let scene = out.scene_zoning.expect("entering moves the client to the lot scene");
    let starter = HousingContent::bundled().unwrap();
    let row = starter
        .lots
        .iter()
        .find(|row| row.property_id == starter.starter_property_id)
        .unwrap();
    assert_eq!(scene.scene, row.scene.as_ref().unwrap().name);
    assert_eq!(scene.position, row.spawn_position);
    assert!(out.replies.is_empty(), "nothing is sent into the old scene");

    let loaded = game.initialize_world(&character(), 3_000).unwrap();
    let order: Vec<_> = loaded.replies.iter().map(|r| subtype(r)).collect();
    assert_eq!(
        order,
        [Some(45), Some(28), Some(44), Some(43)],
        "zone data, then the house record before any 127/4x"
    );
    let zone = ZoneData::decode(&loaded.replies[0][2..]).unwrap();
    assert_eq!(zone.instance, lot);
    let again = game.initialize_world(&character(), 4_000).unwrap();
    assert!(again.replies.is_empty(), "a second load inside the house resends nothing");
}

#[test]
fn saves_and_reconnects_inside_a_house_keep_the_world_pose() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    assert_eq!(game.house_return(), None, "outside, saves use the live pose");
    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000).unwrap();
    assert_eq!(game.house_return(), Some(character()));
    assert_eq!(game.abandon_house_for_reconnect(), Some(character()));
    assert_eq!(game.house_return(), None, "the reconnect left the house");
    let saved = game.state();
    let mut reloaded = Gameplay::load(&content.0, Some(saved)).unwrap();
    let listed = reloaded.handle_in_world(&list_request(OWNER), &character(), 3_000).unwrap();
    assert_eq!(only_list(&listed.replies).records, vec![lot]);
}

/// L045: a character save from before `placed_by` (housing schema 1, fixture with no placer)
/// loads through the real `Gameplay`, the fixture is attributed to the lot owner, the next save
/// is schema 2, and the placer survives another relog.
#[test]
fn an_old_schema_save_migrates_placed_by_and_keeps_it_across_relogs() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    let mut save = serde_json::to_value(game.state()).unwrap();
    let housing = &mut save["housing"];
    assert_eq!(housing["schema_version"], 2);
    housing["schema_version"] = serde_json::json!(1);
    let old_fixture = serde_json::json!({
        "fixture_guid": 0x0f00_0000_0000_0042_u64,
        "house_guid": lot.instance_guid,
        "item_instance_id": 999_999,
        "fixture_definition_id": 34,
        "transform": {
            "position": [0.0, 0.0, 0.0],
            "rotation": [1.0, 0.0, 0.0, 0.0],
            "tilt": [1.0, 0.0, 0.0, 0.0],
            "scale": 1.0,
            "tint_id": 0
        }
    });
    housing["lots"][0]["fixtures"] = serde_json::json!([old_fixture]);
    housing["lots"][0]["count"] = serde_json::json!(1);
    let old = serde_json::from_value(save).unwrap();

    let migrated = Gameplay::load(&content.0, Some(old)).unwrap().state();
    let housing = migrated.housing.clone().expect("the lot survives");
    // The next save is the current schema, and the old fixture belongs to the owner.
    assert_eq!(housing.schema_version, 2);
    assert_eq!(housing.lots[0].fixtures[0].placed_by, OWNER);

    // placed_by survives a relog.
    let again = Some(migrated.clone());
    let relogged = Gameplay::load(&content.0, again).unwrap().state();
    assert_eq!(relogged.housing, migrated.housing);
}

#[test]
fn leaving_returns_to_the_world_pose() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000).unwrap();
    game.initialize_world(&character(), 3_000).unwrap();
    // Inside the lot the client reports lot coordinates; the return still uses the entry pose.
    let mut inside = character();
    inside.zone_id = 1;
    inside.position = [440.0, 5.0, 430.0];
    let out = game.handle_in_world(&LEAVE, &inside, 4_000).unwrap();
    let scene = out.scene_zoning.expect("leaving moves the client back");
    assert_eq!(&scene.position[..3], &character().position);
    assert_eq!(scene.id, 0);
    assert_eq!(game.house_return(), None);
    let world = game.initialize_world(&character(), 5_000).unwrap();
    assert!(
        world.replies.iter().all(|r| subtype(r).is_none()),
        "back outside, the ordinary world entry runs, with no house packets"
    );
}

#[test]
fn refused_requests_change_nothing() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    let before = game.state();
    for (what, inner) in [
        ("an unknown house", enter(lot.instance_guid ^ 0xff)),
        ("leaving while outside", LEAVE.to_vec()),
        ("a truncated enter", enter(lot.instance_guid)[..10].to_vec()),
    ] {
        let out = game.handle_in_world(&inner, &character(), 5_000);
        if let Ok(out) = out {
            assert!(!out.changed, "{what}");
            assert!(out.scene_zoning.is_none(), "{what}");
            assert!(out.replies.iter().all(|r| subtype(r).is_none()), "{what}: a chat line only");
        }
        assert_eq!(game.state(), before, "{what}");
        assert_eq!(game.house_return(), None, "{what}");
    }
}

#[test]
fn the_instance_list_codec_round_trips() {
    let content = Content::with_houses();
    let mut game = content.game();
    let out = game.handle_in_world(&GRANT, &character(), 1_000).unwrap();
    let list = only_list(&out.replies);
    let mut body = vec![127, 0];
    body.extend(list.encode().unwrap());
    assert_eq!(body, out.replies[0], "decode then encode is byte-identical");
}

#[test]
fn content_without_houses_leaves_family_127_alone() {
    let mut game = Gameplay::load(&reference_content(), None).unwrap();
    let before = game.state();
    let out = game.handle_in_world(&GRANT, &character(), 1_000).unwrap();
    assert!(out.replies.is_empty() && !out.changed && out.scene_zoning.is_none());
    assert_eq!(game.state(), before);
    assert!(game.state().housing.is_none());
}

// ---- Gateway route: login, tunnel, gameplay and the scene change, as the client sees it. ----

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["housing-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

/// A gateway logged in and standing in the world, with the content's housing loaded.
fn world_gateway(content: &Content) -> Gateway {
    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    let character = Gateway::reference_state(&reference).unwrap();
    let state = Gameplay::load(&content.0, None).unwrap().state();
    let mut gateway =
        Gateway::load_gameplay(&reference, character, &content.0, Some(state)).unwrap();
    gateway.set_trace_enabled(true);
    gateway.handle_at(&login(), 10_000).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), 10_000).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), 10_000).unwrap();
    gateway
}

/// 127/23 through the gateway, answered by the owner's 127/39 list.
fn gateway_grant(gateway: &mut Gateway, now_ms: u64) -> InstanceList {
    let listed = gateway.handle_at(&wrap(&GRANT), now_ms).unwrap();
    listed
        .replies
        .iter()
        .filter_map(|r| r.get(7..))
        .find(|r| subtype(r) == Some(39))
        .map(|r| InstanceList::decode(&r[2..]).unwrap())
        .expect("127/39 through the gateway")
}

/// The opcode-31 scene change among a gateway's replies.
fn op31(replies: &[Vec<u8>]) -> Option<zone_transition::ZoneTransition> {
    replies.iter().find_map(|r| zone_transition::decode(r).ok())
}

#[test]
fn the_wilds_condo_is_entered_by_op31_and_leave_returns_to_the_world_pose() {
    let content = Content::with_houses();
    let mut gateway = world_gateway(&content);
    let world = gateway.state().clone();
    let condo = gateway_grant(&mut gateway, 11_000).records[0].clone();
    assert_eq!(condo.name_id, WILDS_CONDO);
    let entered = gateway
        .handle_at(&wrap(&enter(condo.instance_guid)), 12_000)
        .unwrap();
    let scene = op31(&entered.replies).expect("entering sends op31");
    assert_eq!(
        (scene.scene.as_str(), scene.id),
        ("hsg_hum_condo_w_lot", 25)
    );
    gateway.handle_at(&wrap(&[13, 0]), 13_000).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), 13_000).unwrap();
    let left = gateway.handle_at(&wrap(&LEAVE), 14_000).unwrap();
    let back = op31(&left.replies).expect("leaving sends op31");
    assert_eq!(&back.position[..3], &world.position);
    assert_eq!(gateway.state(), &world, "back at the entry pose");
}

#[test]
fn the_gateway_sends_the_scene_change_then_the_house_after_the_load() {
    let content = Content::with_houses();
    let mut gateway = world_gateway(&content);
    let list = gateway_grant(&mut gateway, 11_000);
    let entered = gateway
        .handle_at(&wrap(&enter(list.records[0].instance_guid)), 12_000)
        .unwrap();
    assert!(entered.began_zoning, "{:?}", entered.trace);
    assert!(
        entered.replies.iter().any(|r| r.get(7..9) == Some(&[31, 0][..])),
        "an opcode-31 begin-zoning frame is sent"
    );
    // The server binary then registers the player's scene and places them in it (the step
    // that refused the first native run with UnknownZone(1)): the lot must be that scene.
    let zone = gateway.runtime_zone().unwrap();
    let state = gateway.state().clone();
    assert_eq!(zone.id, state.zone_id, "the registered scene is the lot");
    let mut membership = osfr_tools::world::World::runtime();
    membership.register_runtime_zone(zone).unwrap();
    let pose = osfr_tools::world::WorldPose {
        position: [state.position[0], state.position[1], state.position[2], 1.0],
        rotation: [state.rotation[0], state.rotation[1], state.rotation[2], 1.0],
    };
    membership
        .prepare_membership(1, state.zone_id, None, pose, false, 12_000)
        .expect("the lot gets a membership instance");
    gateway.handle_at(&wrap(&[13, 0]), 13_000).unwrap();
    let loaded = gateway.handle_at(&wrap(&[10, 0]), 13_000).unwrap();
    let house: Vec<_> = loaded
        .replies
        .iter()
        .filter_map(|r| r.get(7..).and_then(subtype))
        .collect();
    assert_eq!(house, [45, 28, 44, 43], "the house follows the lot load");
}
