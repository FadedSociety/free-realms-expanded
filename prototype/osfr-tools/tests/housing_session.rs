//! Lead housing slice 1 through the real `Gameplay`: 127/23 grants the starter lot and lists it
//! (127/39), 127/19 enters it (scene change now, house packets after the scene loads), saves and
//! reconnects keep the world pose, 127/10 returns to it, and refusals change nothing. The starter
//! is the Wilds Condo (property 25), and a character who already holds the old starter (property 1)
//! gains it once. Slice 2 decorates it: edit mode, 127/1 -> 127/42, place, pick up, pick up all,
//! a relog, and every refusal answered by 127/53. A new Condo comes with its starter kit (L040):
//! a Chicken in the bag and Basic Square Blocks already standing. Raw inner packets as the client
//! sends them; no player data.
#[path = "common/item_catalogue.rs"]
mod item_catalogue;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    gateway::Gateway,
    housing::{FixtureTransform, SavedFixture},
    housing_native::{
        FixtureAsset, FixtureItemList, FixtureRemove, FixtureUpdate, HouseInfo, InstanceData,
        InstanceList, PlacementDenied, PlayerHousingInstanceInfo, ZoneData,
    },
    housing_service::HousingContent,
    inventory::{InventoryItem, MAX_ITEMS},
    zone_transition,
};
use std::{
    collections::BTreeSet,
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
        // The captured catalogue has no Basic Square Block and no Standard Outdoor Chair, so a
        // picked-up kit block and the generated table's chair need these synthetic definition rows
        // to enter the bag.
        item_catalogue::write(
            &dir,
            &[
                item_catalogue::sized_row(KIT_BLOCK as u32, 64),
                item_catalogue::sized_row(TABLE_CHAIR as u32, 64),
            ],
        );
        Self(dir)
    }

    /// The bundled lots with another starter and no starter kit: a save made before the Wilds
    /// Condo became the starter.
    fn with_starter(property_id: i32) -> Self {
        let content = Self::with_houses();
        let mut houses = HousingContent::bundled().unwrap();
        houses.starter_property_id = property_id;
        houses.starter_kit = None;
        fs::write(
            content.0.join("houses.json"),
            serde_json::to_vec(&houses).unwrap(),
        )
        .unwrap();
        content
    }

    /// The bundled lots plus the generated fixture table (T518).
    fn with_fixtures() -> Self {
        let content = Self::with_houses();
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/housing_fixtures.json"),
            content.0.join("housing_fixtures.json"),
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
    assert_eq!(housing(replies), [39], "one 127/39 reply");
    InstanceList::decode(reply(replies, 39)).expect("a well-formed 127/39")
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
    assert!(
        game.state().housing.is_none(),
        "no lot before the first ask"
    );
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
    let listed = game
        .handle_in_world(&list_request(OWNER), &character(), 3_000)
        .unwrap();
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
    assert!(
        matches!(owned(&game, CHICKEN)[..], [(_, 1)]),
        "the condo comes with one Chicken"
    );
    let held = game.state().housing.unwrap().lots;
    let furnished: Vec<_> = held
        .iter()
        .map(|l| (l.property_id, l.fixtures.len()))
        .collect();
    assert!(furnished.contains(&(25, KIT_BLOCKS)) && furnished.contains(&(1, 0)));

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
    let out = game
        .handle_in_world(&enter(lot.instance_guid), &character(), 2_000)
        .unwrap();
    let scene = out
        .scene_zoning
        .expect("entering moves the client to the lot scene");
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
    let mut expected = vec![Some(45), Some(28), Some(44), Some(43)];
    expected.extend([Some(40); KIT_BLOCKS]);
    assert_eq!(
        order, expected,
        "zone data, then the house record before any 127/4x, then the standing kit"
    );
    let zone = ZoneData::decode(&loaded.replies[0][2..]).unwrap();
    assert_eq!(zone.instance, lot);
    assert_eq!(
        zone.head_size_tenths, 10,
        "a zero head size hides the player's head"
    );
    let again = game.initialize_world(&character(), 4_000).unwrap();
    assert!(
        again.replies.is_empty(),
        "a second load inside the house resends nothing"
    );
}

/// T799: the client refuses every placement unless the preview box sits inside a box of the
/// 127/28 list (`sub_105C290` over the list `00ab7250` reads). A whole 127/40 record per fixture
/// shifted every field after the fixtures, so the Condo, which stands four kit blocks, sent a box
/// the client never saw. The record must hold the lot's box, w = 1 as retail sends it, and the
/// kit as bare instances that the 127/40s then complete by guid.
#[test]
fn the_condos_house_record_carries_its_build_box_and_the_kit_as_bare_instances() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000)
        .unwrap();
    let loaded = game.initialize_world(&character(), 3_000).unwrap();
    let record = InstanceData::decode(reply(&loaded.replies, 28)).unwrap();

    let starter = HousingContent::bundled().unwrap();
    let row = starter
        .lots
        .iter()
        .find(|row| row.property_id == starter.starter_property_id)
        .unwrap();
    let [a, b, c, d, e, f] = row.build_areas[0];
    let expected = [[a, b, c, 1.0], [d, e, f, 1.0]];
    assert_eq!(record.build_areas, vec![expected]);
    let [min, max] = expected;
    assert!((0..3).all(|lane| min[lane] < max[lane]), "min below max");
    assert!(
        (0..3).all(|lane| min[lane] <= ON_THE_LOT[lane] && ON_THE_LOT[lane] <= max[lane]),
        "the test click spot is on the lot"
    );

    let kit: BTreeSet<u64> = updates(&loaded.replies)
        .iter()
        .map(|u| u.instance.guid)
        .collect();
    assert_eq!(kit.len(), KIT_BLOCKS);
    let listed: BTreeSet<u64> = record.fixtures.iter().map(|(_, f)| f.guid).collect();
    assert_eq!(listed, kit, "the 127/28 lists the kit the 127/40s complete");
}

#[test]
fn saves_and_reconnects_inside_a_house_keep_the_world_pose() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    assert_eq!(
        game.house_return(),
        None,
        "outside, saves use the live pose"
    );
    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000)
        .unwrap();
    assert_eq!(game.house_return(), Some(character()));
    assert_eq!(game.abandon_house_for_reconnect(), Some(character()));
    assert_eq!(game.house_return(), None, "the reconnect left the house");
    let saved = game.state();
    let mut reloaded = Gameplay::load(&content.0, Some(saved)).unwrap();
    let listed = reloaded
        .handle_in_world(&list_request(OWNER), &character(), 3_000)
        .unwrap();
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
    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000)
        .unwrap();
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
            assert!(
                out.replies.iter().all(|r| subtype(r).is_none()),
                "{what}: a chat line only"
            );
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

// ---- Decorate (slice 2): 127/6 edit mode, 127/1 -> 127/42, 127/2 place, 127/3 and 127/4. ----

/// houses.json `starter_kit` (L040): a new Condo puts one Chicken in the bag and stands four
/// Basic Square Blocks in the lot.
const CHICKEN: i32 = 70_429;
const KIT_BLOCK: i32 = 32_647;
const KIT_BLOCKS: usize = 4;
/// Inside the Wilds Condo's build box (houses.json property 25).
const ON_THE_LOT: [f32; 4] = [100.0, 0.0, -30.0, 1.0];
const PICKUP_ALL: [u8; 4] = [127, 0, 4, 0];

fn edit(on: bool) -> Vec<u8> {
    vec![127, 0, 6, 0, u8::from(on)]
}

/// 127/1 as the client sends it: binding `00c0b710` calls `00ac4200(id, -1, -1)`, flag 0.
fn ask(item: u32) -> Vec<u8> {
    let mut inner = vec![127, 0, 1, 0];
    inner.extend(item.to_le_bytes());
    inner.extend((-1i32).to_le_bytes());
    inner.extend((-1i32).to_le_bytes());
    inner.push(0);
    inner
}

/// 127/2 (writer `00abc9a0`): the selection, the position and rotation groups, then the scale.
fn place(item: u32) -> Vec<u8> {
    let mut inner = vec![127, 0, 2, 0];
    inner.extend(item.to_le_bytes());
    for value in ON_THE_LOT.into_iter().chain([0.0; 4]).chain([1.0]) {
        inner.extend(value.to_le_bytes());
    }
    inner
}

fn pickup(fixture_guid: u64) -> Vec<u8> {
    let mut inner = vec![127, 0, 3, 0];
    inner.extend(fixture_guid.to_le_bytes());
    inner
}

/// The housing subtypes of a reply set, in order (chat and inventory replies are not family 127).
fn housing(replies: &[Vec<u8>]) -> Vec<u16> {
    replies.iter().filter_map(|r| subtype(r)).collect()
}

/// The body of the 127 reply with this subtype, after the family prefix.
fn reply(replies: &[Vec<u8>], wanted: u16) -> &[u8] {
    let found = replies.iter().find(|r| subtype(r) == Some(wanted));
    &found.unwrap_or_else(|| panic!("127/{wanted} in {replies:?}"))[2..]
}

/// The (item id, count) of every row of `definition` in the saved backpack.
fn owned(game: &Gameplay, definition: i32) -> Vec<(i32, u32)> {
    let state = game.state().inventory;
    let items = state
        .items
        .iter()
        .filter(|item| item.definition_id == definition);
    items.map(|item| (item.id, item.count)).collect()
}

/// The saved fixtures of the one lot this character holds.
fn standing(game: &Gameplay) -> Vec<SavedFixture> {
    let lots = game.state().housing.unwrap().lots;
    assert_eq!(lots.len(), 1, "one lot");
    lots[0].fixtures.clone()
}

/// The 127/40 records among `replies`.
fn updates(replies: &[Vec<u8>]) -> Vec<FixtureUpdate> {
    let records = replies.iter().filter(|r| subtype(r) == Some(40));
    records
        .map(|r| FixtureUpdate::decode(&r[2..]).unwrap())
        .collect()
}

/// Item ids the client is told about: 38/2 adds (payload: definition, tint, id) and 38/4 removals.
fn backpack(replies: &[Vec<u8>], op: u8) -> Vec<i32> {
    let at = if op == 2 { 16 } else { 4 };
    let updates = replies.iter().filter(|r| r.starts_with(&[38, 0, op, 0]));
    updates
        .map(|r| i32::from_le_bytes(r[at..at + 4].try_into().unwrap()))
        .collect()
}

/// Granted, entered, loaded and in edit mode: the lot and the item guid from the 127/43 list.
fn decorating(game: &mut Gameplay) -> (u64, u32) {
    let lot = granted(game);
    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000)
        .unwrap();
    let loaded = game.initialize_world(&character(), 3_000).unwrap();
    let list = FixtureItemList::decode(reply(&loaded.replies, 43)).unwrap();
    assert_eq!(list.items.len(), 1, "the starter Chicken is listed");
    assert_eq!(list.items[0].item_definition_id, CHICKEN);
    let on = game
        .handle_in_world(&edit(true), &character(), 4_000)
        .unwrap();
    assert_eq!(housing(&on.replies), [44]);
    assert!(
        HouseInfo::decode(reply(&on.replies, 44))
            .unwrap()
            .in_edit_mode
    );
    let item = u32::try_from(list.items[0].fixture_guid).unwrap();
    (lot.instance_guid, item)
}

/// 127/1 then 127/2 for one owned item: the placed fixture's guid.
fn place_item(game: &mut Gameplay, item: u32) -> u64 {
    let asked = game
        .handle_in_world(&ask(item), &character(), 5_000)
        .unwrap();
    assert_eq!(housing(&asked.replies), [42]);
    let asset = FixtureAsset::decode(reply(&asked.replies, 42)).unwrap();
    assert_eq!(asset.selection_id, item, "127/42 echoes the 127/43 id");
    let placed = game
        .handle_in_world(&place(item), &character(), 6_000)
        .unwrap();
    assert!(placed.changed, "a placement is saved");
    assert_eq!(housing(&placed.replies), [40, 44]);
    assert_eq!(
        backpack(&placed.replies, 4),
        [item as i32],
        "the client drops the item"
    );
    FixtureUpdate::decode(reply(&placed.replies, 40))
        .unwrap()
        .instance
        .guid
}

/// A decorate refusal: a chat line plus the client's 127/53 (its reason is returned), and nothing
/// changes.
fn refused(game: &mut Gameplay, what: &str, inner: &[u8]) -> String {
    let before = serde_json::to_vec(&game.state()).unwrap();
    let out = game.handle_in_world(inner, &character(), 9_000).unwrap();
    assert!(!out.changed, "{what}");
    assert_eq!(housing(&out.replies), [53], "{what}");
    let denied = PlacementDenied::decode(reply(&out.replies, 53)).unwrap();
    assert!(!denied.message.is_empty(), "{what}");
    let after = serde_json::to_vec(&game.state()).unwrap();
    assert_eq!(after, before, "{what}: state and inventory stay byte-equal");
    denied.message
}

#[test]
fn a_new_condo_comes_with_a_chicken_and_standing_blocks_once() {
    let content = Content::with_houses();
    let mut game = content.game();
    let first = game.handle_in_world(&GRANT, &character(), 1_000).unwrap();
    let [(chicken, 1)] = owned(&game, CHICKEN)[..] else {
        panic!("one single Chicken: {:?}", owned(&game, CHICKEN))
    };
    assert_eq!(
        backpack(&first.replies, 2),
        [chicken],
        "the client is sent the Chicken"
    );
    let kit = HousingContent::bundled().unwrap().starter_kit.unwrap();
    let blocks = standing(&game);
    assert_eq!(blocks.len(), KIT_BLOCKS);
    for at in &kit.placed_positions {
        assert!(blocks.iter().any(|b| b.transform.position == *at), "{at:?}");
    }
    assert!(blocks.iter().all(|b| b.fixture_definition_id == KIT_BLOCK));
    // Each block has its own item id: never in the bag, and the bag's counter moved past it.
    let ids: BTreeSet<i64> = blocks
        .iter()
        .map(|b| i64::from(b.item_instance_id))
        .collect();
    let bag = game.state().inventory;
    assert_eq!(ids.len(), KIT_BLOCKS);
    assert!(
        bag.items
            .iter()
            .all(|item| !ids.contains(&i64::from(item.id)))
    );
    assert!(bag.next_instance_id > ids.last().copied());

    let saved = game.state();
    let again = game.handle_in_world(&GRANT, &character(), 2_000).unwrap();
    assert!(!again.changed && backpack(&again.replies, 2).is_empty());
    assert_eq!(game.state(), saved, "a repeat ask grants no second kit");
}

#[test]
fn a_standing_block_picks_up_as_one_block_and_the_save_reloads() {
    let content = Content::with_houses();
    let mut game = content.game();
    decorating(&mut game);
    let block = standing(&game)[0].fixture_guid;
    let picked = game
        .handle_in_world(&pickup(block), &character(), 7_000)
        .unwrap();
    assert!(picked.changed);
    assert_eq!(housing(&picked.replies), [41, 44]);
    let [(back, 1)] = owned(&game, KIT_BLOCK)[..] else {
        panic!("one block back: {:?}", owned(&game, KIT_BLOCK))
    };
    assert_eq!(backpack(&picked.replies, 2), [back]);
    assert_eq!(standing(&game).len(), KIT_BLOCKS - 1);
    // The reload re-runs conservation: no id is both in the bag and on the lot.
    let relogged = Gameplay::load(&content.0, Some(game.state())).unwrap();
    assert_eq!(owned(&relogged, KIT_BLOCK), [(back, 1)]);
    assert_eq!(standing(&relogged), standing(&game));
}

/// One single unit of `definition_id` under item id `id`, as a save holds it.
fn single(id: i32, definition_id: i32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

/// The generated table's Standard Outdoor Chair (item 15602, client model 1865). The capture
/// defaults have no row for it, so it distinguishes the two tables.
const TABLE_CHAIR: i32 = 15_602;
const TABLE_CHAIR_MODEL: i32 = 1_865;

/// A character save holding one single unit of each item, loaded through the real save path.
fn owning(content: &Content, definitions: &[i32]) -> Gameplay {
    let mut state = content.game().state();
    let mut id = state
        .inventory
        .items
        .iter()
        .map(|i| i.id)
        .max()
        .unwrap_or(0);
    for &definition in definitions {
        id += 1;
        state.inventory.items.push(single(id, definition));
    }
    Gameplay::load(&content.0, Some(state)).unwrap()
}

#[test]
fn the_generated_fixture_table_lists_an_owned_block_and_chair_on_entry() {
    let content = Content::with_fixtures();
    let mut game = owning(&content, &[KIT_BLOCK, TABLE_CHAIR]);
    let lot = granted(&mut game);
    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000)
        .unwrap();
    let loaded = game.initialize_world(&character(), 3_000).unwrap();
    let list = FixtureItemList::decode(reply(&loaded.replies, 43)).unwrap();
    let mut listed: Vec<i32> = list
        .items
        .iter()
        .map(|item| item.item_definition_id)
        .collect();
    listed.sort();
    assert_eq!(
        listed,
        [TABLE_CHAIR, KIT_BLOCK, CHICKEN],
        "the bag's three placeable items are listed"
    );
    let block = list
        .definitions
        .iter()
        .find(|definition| definition.item_definition_id == KIT_BLOCK)
        .expect("the block's generated definition");
    assert_eq!((block.id, block.model_id), (KIT_BLOCK, 1_926));
    let chair = list
        .definitions
        .iter()
        .find(|definition| definition.item_definition_id == TABLE_CHAIR)
        .expect("the chair's generated definition");
    assert_eq!((chair.id, chair.model_id), (TABLE_CHAIR, TABLE_CHAIR_MODEL));
    // The 127/40 records carry the same generated row: model and aliases.
    let updates = updates(&loaded.replies);
    assert_eq!(updates.len(), KIT_BLOCKS);
    let kit = &updates[0];
    assert_eq!(kit.definition.model_id, 1_926);
    assert_eq!(kit.instance.customization.texture_alias, "fun-block-L1");
    assert_eq!(kit.instance.customization.tint_alias, "dyetint");
}

#[test]
fn without_the_generated_table_the_capture_defaults_still_apply() {
    let content = Content::with_houses();
    let mut game = owning(&content, &[TABLE_CHAIR]);
    let lot = granted(&mut game);
    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000)
        .unwrap();
    let loaded = game.initialize_world(&character(), 3_000).unwrap();
    let list = FixtureItemList::decode(reply(&loaded.replies, 43)).unwrap();
    assert_eq!(list.items.len(), 1, "the Chicken only");
    assert_eq!(list.items[0].item_definition_id, CHICKEN);
    assert!(
        list.items
            .iter()
            .all(|item| item.item_definition_id != TABLE_CHAIR),
        "the capture defaults have no chair row"
    );
}

#[test]
fn the_kit_chicken_arrives_unstacked_beside_one_already_owned() {
    let content = Content::with_houses();
    let mut state = content.game().state();
    let id = state
        .inventory
        .items
        .iter()
        .map(|i| i.id)
        .max()
        .unwrap_or(0)
        + 1;
    state.inventory.items.push(single(id, CHICKEN));
    let mut game = Gameplay::load(&content.0, Some(state)).unwrap();
    granted(&mut game);
    let chickens = owned(&game, CHICKEN);
    assert_eq!(chickens.len(), 2, "{chickens:?}");
    assert!(
        chickens.iter().all(|&(_, count)| count == 1),
        "each can be placed: {chickens:?}"
    );
}

#[test]
fn a_full_bag_refuses_the_whole_grant_and_a_retry_gets_the_kit() {
    let content = Content::with_houses();
    let mut state = content.game().state();
    let mut id = state.inventory.items.iter().map(|i| i.id).max().unwrap();
    while state.inventory.items.len() < MAX_ITEMS {
        id += 1;
        // Any catalogued item fills a row: 7466 is the captured Blue Square Block.
        state.inventory.items.push(single(id, 7_466));
    }
    let mut game = Gameplay::load(&content.0, Some(state)).unwrap();
    let before = serde_json::to_vec(&game.state()).unwrap();
    let out = game.handle_in_world(&GRANT, &character(), 1_000).unwrap();
    assert!(
        !out.changed && housing(&out.replies).is_empty(),
        "no house list"
    );
    let text = String::from_utf8_lossy(&out.replies.concat()).into_owned();
    assert!(text.contains("The inventory could not apply that change."));
    let after = serde_json::to_vec(&game.state()).unwrap();
    assert_eq!(after, before, "nothing changes");

    let mut state = game.state();
    state.inventory.items.pop();
    let mut game = Gameplay::load(&content.0, Some(state)).unwrap();
    granted(&mut game);
    assert!(matches!(owned(&game, CHICKEN)[..], [(_, 1)]));
    assert_eq!(standing(&game).len(), KIT_BLOCKS);
}

#[test]
fn kit_ids_skip_an_id_a_lot_already_holds() {
    // A pre-condo save with no id counter whose lot-1 fixture holds the id just above the bag's
    // highest row (that item left the bag when it was placed).
    let mut before = Content::with_starter(1).game();
    granted(&mut before);
    let mut state = before.state();
    assert_eq!(state.inventory.next_instance_id, None);
    let held = state.inventory.items.iter().map(|i| i.id).max().unwrap() as u32 + 1;
    let lot = &mut state.housing.as_mut().unwrap().lots[0];
    let area = lot.build_areas[0];
    lot.fixtures.push(SavedFixture {
        fixture_guid: 0x00a0_0000_0000_0001,
        house_guid: lot.lot_guid,
        item_instance_id: held,
        fixture_definition_id: 34,
        transform: FixtureTransform {
            position: [area[0], area[1], area[2]],
            ..FixtureTransform::default()
        },
        placed_by: lot.owner_guid, // L045's field (rebase onto 770b791): the owner placed it.
    });
    lot.count = 1;
    let content = Content::with_houses();
    let mut game = Gameplay::load(&content.0, Some(state)).unwrap();
    let out = game.handle_in_world(&GRANT, &character(), 2_000).unwrap();
    assert!(out.changed);
    let [(chicken, 1)] = owned(&game, CHICKEN)[..] else {
        panic!("one Chicken: {:?}", owned(&game, CHICKEN))
    };
    assert_ne!(chicken as u32, held);
    // The save reloads: the conservation check finds no id both in the bag and on a lot.
    Gameplay::load(&content.0, Some(game.state())).expect("conservation holds");
}

#[test]
fn edit_mode_on_then_off_flips_the_house_flag() {
    let content = Content::with_houses();
    let mut game = content.game();
    decorating(&mut game);
    let off = game
        .handle_in_world(&edit(false), &character(), 5_000)
        .unwrap();
    assert_eq!(housing(&off.replies), [44]);
    assert!(
        !HouseInfo::decode(reply(&off.replies, 44))
            .unwrap()
            .in_edit_mode
    );
    assert!(!off.changed);
}

#[test]
fn the_listed_chicken_is_asked_for_placed_and_leaves_the_backpack() {
    let content = Content::with_houses();
    let mut game = content.game();
    let (_, item) = decorating(&mut game);
    let fixture = place_item(&mut game, item);
    assert!(
        owned(&game, CHICKEN).is_empty(),
        "the item left the inventory"
    );
    let placed = standing(&game);
    assert_eq!(placed.len(), KIT_BLOCKS + 1);
    let chicken = placed.iter().find(|f| f.fixture_guid == fixture).unwrap();
    assert_eq!(chicken.item_instance_id, item);
}

#[test]
fn pickup_returns_the_chicken_and_it_places_again() {
    let content = Content::with_houses();
    let mut game = content.game();
    let (_, item) = decorating(&mut game);
    let fixture = place_item(&mut game, item);
    let picked = game
        .handle_in_world(&pickup(fixture), &character(), 7_000)
        .unwrap();
    assert!(picked.changed);
    assert_eq!(housing(&picked.replies), [41, 44]);
    let removed = FixtureRemove::decode(reply(&picked.replies, 41)).unwrap();
    assert_eq!(removed.fixture_guid, fixture);
    // The unit comes back as one single item (a fresh id), not a stack of two.
    let [(back, 1)] = owned(&game, CHICKEN)[..] else {
        panic!("one single Chicken back: {:?}", owned(&game, CHICKEN))
    };
    assert_eq!(
        backpack(&picked.replies, 2),
        [back],
        "the client is sent the item"
    );
    assert_eq!(standing(&game).len(), KIT_BLOCKS, "only the kit stands");
    place_item(&mut game, u32::try_from(back).unwrap());
}

#[test]
fn a_new_127_1_retires_the_previous_selection() {
    let content = Content::with_houses();
    let mut game = content.game();
    let (_, first) = decorating(&mut game);
    let fixture = place_item(&mut game, first);
    game.handle_in_world(&pickup(fixture), &character(), 7_000)
        .unwrap();
    let [(back, 1)] = owned(&game, CHICKEN)[..] else {
        panic!("one single Chicken back: {:?}", owned(&game, CHICKEN))
    };
    let back = u32::try_from(back).unwrap();
    assert_ne!(back, first, "pickup returns the unit under a fresh id");
    place_item(&mut game, back);
    // Only the latest 127/1 stays bound, so a place/pickup loop cannot grow the bindings.
    let reason = refused(&mut game, "127/2 with the retired id", &place(first));
    assert_eq!(reason, "That fixture selection is no longer available.");
}

#[test]
fn pickup_all_returns_every_standing_fixture() {
    let content = Content::with_houses();
    let mut game = content.game();
    let (_, item) = decorating(&mut game);
    let fixture = place_item(&mut game, item);
    let all = game
        .handle_in_world(&PICKUP_ALL, &character(), 7_000)
        .unwrap();
    assert!(all.changed);
    let mut expected = vec![41; KIT_BLOCKS + 1];
    expected.push(44);
    assert_eq!(housing(&all.replies), expected);
    let removed = all.replies.iter().filter(|r| subtype(r) == Some(41));
    let mut removed = removed.map(|r| FixtureRemove::decode(&r[2..]).unwrap().fixture_guid);
    assert!(removed.any(|guid| guid == fixture));
    let [(back, 1)] = owned(&game, CHICKEN)[..] else {
        panic!("one single Chicken back: {:?}", owned(&game, CHICKEN))
    };
    assert!(backpack(&all.replies, 2).contains(&back));
    // The blocks come back too (as one stack: the L031 return rule merges equal units).
    let blocks: u32 = owned(&game, KIT_BLOCK).iter().map(|&(_, n)| n).sum();
    assert_eq!(blocks, KIT_BLOCKS as u32);
    assert!(standing(&game).is_empty());
}

#[test]
fn a_placed_chicken_and_the_kit_survive_save_reload_and_reentry() {
    let content = Content::with_houses();
    let mut game = content.game();
    let (lot, item) = decorating(&mut game);
    let fixture = place_item(&mut game, item);
    let mut relogged = Gameplay::load(&content.0, Some(game.state())).unwrap();
    assert_eq!(standing(&relogged), standing(&game));
    relogged
        .handle_in_world(&enter(lot), &character(), 8_000)
        .unwrap();
    let loaded = relogged.initialize_world(&character(), 9_000).unwrap();
    let mut expected = vec![45, 28, 44, 43];
    expected.extend([40; KIT_BLOCKS + 1]);
    assert_eq!(housing(&loaded.replies), expected);
    let updates = updates(&loaded.replies);
    let chicken = updates.iter().find(|u| u.instance.guid == fixture).unwrap();
    assert_eq!(
        (chicken.instance.house_guid, chicken.info.item_definition_id),
        (lot, CHICKEN)
    );
    let blocks = updates
        .iter()
        .filter(|u| u.info.item_definition_id == KIT_BLOCK);
    assert_eq!(blocks.count(), KIT_BLOCKS);
    let list = FixtureItemList::decode(reply(&loaded.replies, 43)).unwrap();
    assert!(
        list.items.is_empty(),
        "the placed Chicken is not offered again"
    );
    assert!(owned(&relogged, CHICKEN).is_empty());
}

#[test]
fn decorate_refusals_answer_127_53_and_change_nothing() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    let [(item, 1)] = owned(&game, CHICKEN)[..] else {
        panic!("one single Chicken")
    };
    let item = u32::try_from(item).unwrap();
    for (what, inner) in [
        ("127/1 not in a house", ask(item)),
        ("127/2 not in a house", place(item)),
        ("127/3 not in a house", pickup(1)),
        ("127/4 not in a house", PICKUP_ALL.to_vec()),
        ("127/6 not in a house", edit(true)),
    ] {
        refused(&mut game, what, &inner);
    }

    game.handle_in_world(&enter(lot.instance_guid), &character(), 2_000)
        .unwrap();
    game.initialize_world(&character(), 3_000).unwrap();
    for (what, inner) in [
        ("127/1 not in edit mode", ask(item)),
        ("127/2 not in edit mode", place(item)),
        ("127/3 not in edit mode", pickup(1)),
        ("127/4 not in edit mode", PICKUP_ALL.to_vec()),
    ] {
        refused(&mut game, what, &inner);
    }

    game.handle_in_world(&edit(true), &character(), 4_000)
        .unwrap();
    for (what, inner) in [
        ("an unknown selection", ask(item + 1_000)),
        ("a zero selection", ask(0)),
        ("127/2 before its 127/1", place(item)),
        ("an unknown fixture", pickup(1)),
    ] {
        refused(&mut game, what, &inner);
    }

    // A save whose Chicken is a stack of two: the stack cannot back one fixture.
    let mut stacked = game.state();
    let chicken = stacked
        .inventory
        .items
        .iter_mut()
        .find(|i| i.definition_id == CHICKEN);
    chicken.unwrap().count = 2;
    let mut game = Gameplay::load(&content.0, Some(stacked)).unwrap();
    game.handle_in_world(&enter(lot.instance_guid), &character(), 5_000)
        .unwrap();
    game.handle_in_world(&edit(true), &character(), 6_000)
        .unwrap();
    refused(&mut game, "a stacked item", &ask(item));
    refused(&mut game, "127/2 after a refused 127/1", &place(item));
}

#[test]
fn a_reconnect_ends_the_edit_session() {
    let content = Content::with_houses();
    let mut game = content.game();
    let (_, item) = decorating(&mut game);
    assert_eq!(game.abandon_house_for_reconnect(), Some(character()));
    refused(&mut game, "127/1 after a reconnect", &ask(item));
    refused(&mut game, "127/6 after a reconnect", &edit(true));
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
        entered
            .replies
            .iter()
            .any(|r| r.get(7..9) == Some(&[31, 0][..])),
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
    let mut expected = vec![45, 28, 44, 43];
    expected.extend([40; KIT_BLOCKS]);
    assert_eq!(house, expected, "the house follows the lot load");
}

// ---- T517: the rating directory probe (174/16 -> an empty 174/17), before and inside a lot. ----

/// 174/16 as the client writer `0x00abfe70` lays it out: header, system, empty candidate id, owner
/// and correlation. 34 bytes for the housing system.
fn candidate_info(system: &str, owner: u64, correlation: u64) -> Vec<u8> {
    let mut inner = vec![174, 0, 16];
    for text in [system, ""] {
        inner.extend((text.len() as u32).to_le_bytes());
        inner.extend(text.as_bytes());
    }
    inner.extend(owner.to_le_bytes());
    inner.extend(correlation.to_le_bytes());
    inner
}

/// 174/17 with no candidates: header, echoed correlation, u32 count.
fn empty_candidate_info_reply(correlation: u64) -> Vec<u8> {
    let mut bytes = vec![174, 0, 17];
    bytes.extend(correlation.to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes
}

#[test]
fn the_rating_probe_is_answered_with_an_empty_directory_inside_and_outside_a_house() {
    let content = Content::with_houses();
    let mut game = content.game();
    let lot = granted(&mut game);
    let correlation = 0x1122_3344_5566_7788;
    let request = candidate_info("Housing", OWNER, correlation);
    assert_eq!(request.len(), 34, "the native layout");
    let expected = empty_candidate_info_reply(correlation);

    // The 127/39 list path sends the probe with no house visited.
    let outside = serde_json::to_vec(&game.state()).unwrap();
    let out = game.handle_in_world(&request, &character(), 2_000).unwrap();
    assert!(!out.changed, "the probe is read-only");
    assert_eq!(out.replies, vec![expected.clone()], "one empty 174/17");
    assert_eq!(
        serde_json::to_vec(&game.state()).unwrap(),
        outside,
        "no housing, inventory or save change"
    );

    // The 127/45 zone-data path sends it again inside the lot, and every repeat is answered.
    game.handle_in_world(&enter(lot.instance_guid), &character(), 3_000)
        .unwrap();
    game.initialize_world(&character(), 4_000).unwrap();
    let inside = serde_json::to_vec(&game.state()).unwrap();
    for now in [5_000, 6_000] {
        let out = game.handle_in_world(&request, &character(), now).unwrap();
        assert!(!out.changed);
        assert_eq!(
            out.replies,
            vec![expected.clone()],
            "each repeated probe gets one reply"
        );
        assert_eq!(
            serde_json::to_vec(&game.state()).unwrap(),
            inside,
            "no housing, inventory or save change"
        );
    }
}

#[test]
fn malformed_rating_probes_send_nothing_and_change_nothing() {
    let content = Content::with_houses();
    let mut game = content.game();
    granted(&mut game);
    let valid = candidate_info("Housing", OWNER, 7);
    let mut trailing = valid.clone();
    trailing.push(0);
    let mut wrong_subtype = valid.clone();
    wrong_subtype[2] = 15;
    let mut oversize = vec![174, 0, 16];
    oversize.extend(257u32.to_le_bytes());
    oversize.extend(vec![b'A'; 257]);
    let mut bad_utf8 = vec![174, 0, 16];
    bad_utf8.extend(7u32.to_le_bytes());
    bad_utf8.extend([0xff; 7]);

    for (what, inner) in [
        ("truncated", valid[..valid.len() - 1].to_vec()),
        ("trailing byte", trailing),
        ("oversize string", oversize),
        ("wrong system", candidate_info("Chat", OWNER, 7)),
        ("invalid utf-8", bad_utf8),
        ("wrong subtype", wrong_subtype),
    ] {
        let before = serde_json::to_vec(&game.state()).unwrap();
        let out = game.handle_in_world(&inner, &character(), 8_000).unwrap();
        assert!(!out.changed, "{what}");
        assert!(
            out.replies.is_empty(),
            "{what}: no reply, got {:02x?}",
            out.replies
        );
        assert_eq!(
            serde_json::to_vec(&game.state()).unwrap(),
            before,
            "{what}: state and inventory stay byte-equal"
        );
    }
}

#[test]
fn content_without_houses_leaves_the_rating_probe_alone() {
    let mut game = Gameplay::load(&reference_content(), None).unwrap();
    let before = serde_json::to_vec(&game.state()).unwrap();
    let out = game
        .handle_in_world(&candidate_info("Housing", OWNER, 1), &character(), 1_000)
        .unwrap();
    assert!(!out.changed);
    assert!(
        out.replies.is_empty(),
        "dormant without houses.json: {:02x?}",
        out.replies
    );
    assert_eq!(serde_json::to_vec(&game.state()).unwrap(), before);
    assert!(game.state().housing.is_none());
}

#[test]
fn the_gateway_answers_the_rating_probe_instead_of_logging_unknown_application_message() {
    let content = Content::with_houses();
    let mut gateway = world_gateway(&content);
    let before = gateway.snapshot();
    let correlation = 0x0f0e_0d0c_0b0a_0908;
    let request = candidate_info("Housing", OWNER, correlation);
    let out = gateway.handle_at(&wrap(&request), 12_000).unwrap();
    let expected = empty_candidate_info_reply(correlation);
    let inner: Vec<_> = out
        .replies
        .iter()
        .filter_map(|reply| reply.get(7..))
        .collect();
    assert_eq!(inner, vec![&expected[..]], "exactly one 174/17 reply");
    assert!(!out.state_changed);
    assert!(
        out.trace
            .iter()
            .all(|event| event.reason.as_deref() != Some("unknown application message")),
        "{:?}",
        out.trace
    );
    assert_eq!(gateway.snapshot(), before, "the probe is read-only");
}
