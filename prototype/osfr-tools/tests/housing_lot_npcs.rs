//! P-housing: an NPC on a player's lot, through the real `Gameplay` entry path. The Wilds Farm
//! (houses.json property 34, scene `farming_wilds_farmstead_01`) is granted free beside the
//! starter on the first 127/23 (wiki Farms: "Wilds Farm free"; its tutorial quest, client text
//! 435268, says "Click the Wilds Farm option in your Houses Browser to meet Farmer Chug"). Farmer
//! Chug (npcs/wilds.json, `house_zone_id` 34) is drawn only to a player inside that lot, answers a
//! click there, and is neither drawn nor reachable from the continent at the same coordinates.
//! Raw inner packets as the client sends them; no player data.
#[path = "support/pet_inventory.rs"]
mod pet_inventory;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    housing_native::{InstanceList, PlayerHousingInstanceInfo},
    housing_service::HousingContent,
    pets::PetService,
    startup_world,
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
/// Locale 5409 "Wilds Farm", the name id of property 34.
const WILDS_FARM: i32 = 5409;
const FARM_PROPERTY: i32 = 34;
/// Locale 434254 "Farmer Chug".
const FARMER_CHUG_NAME: i32 = 434254;
/// Models.txt 488 chugawug_farmer_m.adr.
const FARMER_CHUG_MODEL: i32 = 488;

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

/// The opening reference content with the repository's `houses.json`; the NPC tables come from
/// the bundled `data/content`, as every gameplay route test's do.
struct Content(PathBuf);

impl Content {
    fn with_houses() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-housing-lot-npcs-{}-{}",
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

    fn game(&self) -> Gameplay {
        Gameplay::load(&self.0, None).unwrap()
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The player in the continent, at the opening pose.
fn in_world() -> CharacterState {
    CharacterState {
        guid: OWNER,
        zone_id: 0,
        position: [-1904.0, -42.0, 420.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

/// The farm lot row and Farmer Chug's row, read from the content the server ships.
fn farm_row() -> osfr_tools::housing_service::HousingLotDefinition {
    let houses = HousingContent::bundled().unwrap();
    (houses.lots.into_iter())
        .find(|lot| lot.property_id == FARM_PROPERTY)
        .expect("the Wilds Farm row")
}

fn chug() -> (u64, [f32; 3]) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/npcs/wilds.json");
    let document: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let row = (document["records"].as_array().unwrap().iter())
        .find(|row| row["name"] == "Farmer Chug")
        .expect("Farmer Chug in npcs/wilds.json");
    assert_eq!(row["house_zone_id"].as_i64(), Some(i64::from(FARM_PROPERTY)));
    let position: Vec<f32> = (row["position"].as_array().unwrap().iter())
        .map(|value| value.as_f64().unwrap() as f32)
        .collect();
    (
        row["npc_id"].as_u64().unwrap(),
        [position[0], position[1], position[2]],
    )
}

/// The player standing on the farm, two metres from Farmer Chug, in the lot's own zone.
fn on_farm_near_chug() -> CharacterState {
    let (_, position) = chug();
    CharacterState {
        guid: OWNER,
        zone_id: FARM_PROPERTY as u32,
        position: [position[0] + 2.0, position[1], position[2]],
        rotation: [-1.0, 0.0, 0.0],
    }
}

fn enter(house_guid: u64) -> Vec<u8> {
    let mut inner = vec![127, 0, 19, 0];
    inner.extend(house_guid.to_le_bytes());
    inner.extend(0i32.to_le_bytes());
    inner.push(0);
    inner
}

fn interact(target: u64) -> Vec<u8> {
    let mut inner = vec![26, 0, 8, 0];
    inner.extend(target.to_le_bytes());
    inner.push(0);
    inner
}

fn subtype(reply: &[u8]) -> Option<u16> {
    (reply.len() >= 4 && reply[..2] == [127, 0]).then(|| u16::from_le_bytes([reply[2], reply[3]]))
}

fn list(replies: &[Vec<u8>]) -> InstanceList {
    let reply = (replies.iter())
        .find(|reply| subtype(reply) == Some(39))
        .expect("a 127/39 list");
    InstanceList::decode(&reply[2..]).expect("a well-formed 127/39")
}

/// The actors a reply set draws (35/2), parsed.
fn drawn(replies: &[Vec<u8>]) -> Vec<startup_world::NpcSpawn> {
    (replies.iter())
        .filter(|reply| reply.starts_with(&[35, 0, 2, 0]))
        .map(|reply| startup_world::parse_add_npc(reply).expect("a well-formed 35/2"))
        .collect()
}

fn grant(game: &mut Gameplay) -> Vec<PlayerHousingInstanceInfo> {
    let out = game.handle_in_world(&GRANT, &in_world(), 1_000).unwrap();
    assert!(out.changed, "the grant is saved");
    list(&out.replies).records
}

fn farm(records: &[PlayerHousingInstanceInfo]) -> PlayerHousingInstanceInfo {
    (records.iter())
        .find(|record| record.name_id == WILDS_FARM)
        .cloned()
        .expect("the Wilds Farm is listed")
}

#[test]
fn the_wilds_farm_ships_enterable_and_granted_with_the_starter() {
    let row = farm_row();
    assert!(row.enabled && row.granted_with_starter, "{row:?}");
    let scene = row.scene.as_ref().expect("a scene");
    assert_eq!(scene.name, "farming_wilds_farmstead_01");
    assert_eq!(scene.zone_id, FARM_PROPERTY as u32);
    assert_eq!(row.name_id, WILDS_FARM);
    let (_, position) = chug();
    assert!(
        row.build_areas.iter().any(|b| {
            (0..3).all(|axis| b[axis] <= position[axis] && position[axis] <= b[axis + 3])
        }),
        "Farmer Chug stands in the farm's build box"
    );
}

#[test]
fn the_first_grant_lists_the_wilds_farm_beside_the_condo_once() {
    let content = Content::with_houses();
    let mut game = content.game();
    let records = grant(&mut game);
    assert_eq!(records.len(), 2, "the condo and the farm: {records:?}");
    let farm = farm(&records);
    assert_eq!(farm.owner_guid, OWNER);
    let again = game.handle_in_world(&GRANT, &in_world(), 2_000).unwrap();
    assert!(!again.changed, "a second ask grants nothing");
    assert_eq!(list(&again.replies).records, records);
    let lots = game.state().housing.unwrap().lots;
    assert_eq!(
        lots.iter().filter(|lot| lot.property_id == FARM_PROPERTY).count(),
        1,
        "one farm per character"
    );
}

/// A character saved before the farm was free holds the condo only; the next 127/23 adds the
/// farm without standing the condo's kit a second time.
#[test]
fn an_old_condo_owner_gains_the_farm_once_without_a_second_kit() {
    let content = Content::with_houses();
    let mut game = content.game();
    grant(&mut game);
    let mut state = game.state();
    let housing = state.housing.as_mut().unwrap();
    housing.lots.retain(|lot| lot.property_id != FARM_PROPERTY);
    let condo_fixtures = housing.lots[0].fixtures.len();
    assert!(condo_fixtures > 0, "the condo stands its kit");

    let mut game = Gameplay::load(&content.0, Some(state)).unwrap();
    let out = game.handle_in_world(&GRANT, &in_world(), 3_000).unwrap();
    assert!(out.changed, "the farm is saved");
    let records = list(&out.replies).records;
    assert_eq!(records.len(), 2);
    farm(&records);
    let lots = game.state().housing.unwrap().lots;
    let condo = lots.iter().find(|lot| lot.property_id != FARM_PROPERTY).unwrap();
    assert_eq!(condo.fixtures.len(), condo_fixtures, "no second kit");
    let farm = lots.iter().find(|lot| lot.property_id == FARM_PROPERTY).unwrap();
    assert!(farm.fixtures.is_empty(), "the farm comes empty");
}

/// The real path: 127/19 moves the player to the farm scene; after the load the house records go
/// out, and the NPC visibility pass draws Farmer Chug (35/2) for the player on the lot. In the
/// continent, at the very same coordinates, nothing of him is drawn and a click on him is refused.
#[test]
fn farmer_chug_is_drawn_on_the_farm_and_answers_a_click_there_only() {
    let content = Content::with_houses();
    let mut game = content.game();
    let farm = farm(&grant(&mut game));
    let (chug_guid, _) = chug();

    let entered = game
        .handle_in_world(&enter(farm.instance_guid), &in_world(), 2_000)
        .unwrap();
    let scene = entered.scene_zoning.expect("entering moves the client to the lot");
    assert_eq!(
        (scene.scene.as_str(), scene.id),
        ("farming_wilds_farmstead_01", FARM_PROPERTY)
    );
    let row = farm_row();
    assert_eq!(scene.position, row.spawn_position);

    let on_farm = on_farm_near_chug();
    let loaded = game.initialize_world(&on_farm, 3_000).unwrap();
    let order: Vec<_> = loaded.replies.iter().map(|r| subtype(r)).collect();
    assert_eq!(
        order,
        vec![Some(45), Some(28), Some(44), Some(43)],
        "zone data, the house record, house info, the item list; no fixture stands yet"
    );
    let visible = game.update_npc_visibility(&on_farm).unwrap();
    let actors = drawn(&visible);
    let chug_actor = (actors.iter())
        .find(|actor| actor.actor_id == chug_guid)
        .unwrap_or_else(|| panic!("Farmer Chug is drawn on the farm: {actors:?}"));
    assert_eq!(chug_actor.name_string_id, FARMER_CHUG_NAME);
    assert_eq!(chug_actor.model_id, FARMER_CHUG_MODEL);
    assert_eq!(actors.len(), 1, "he is the only actor the farm draws");

    let clicked = game
        .handle_in_world(&interact(chug_guid), &on_farm, 4_000)
        .expect("a click on the farm's own NPC is answered");
    assert!(
        clicked.replies.iter().any(|reply| reply.starts_with(&[26, 0])),
        "a dialogue answers the click: {:?}",
        clicked.replies.iter().map(|r| &r[..4.min(r.len())]).collect::<Vec<_>>()
    );

    let left = game.handle_in_world(&LEAVE, &on_farm, 5_000).unwrap();
    let back = left.scene_zoning.expect("leaving returns to the world");
    assert_eq!(&back.position[..3], &in_world().position);

    // The continent at the same spot: the farm's NPC is neither drawn nor reachable.
    let mut world = content.game();
    let mut at_farm_coordinates = on_farm.clone();
    at_farm_coordinates.zone_id = 0;
    let visible = world.update_npc_visibility(&at_farm_coordinates).unwrap();
    assert!(
        !drawn(&visible).iter().any(|actor| actor.actor_id == chug_guid),
        "Farmer Chug is not drawn in the continent"
    );
    let result = world.handle_in_world(&interact(chug_guid), &at_farm_coordinates, 6_000);
    let heads: Vec<Vec<u8>> = (result.as_ref().ok().into_iter())
        .flat_map(|out| out.replies.iter().map(|reply| reply[..4.min(reply.len())].to_vec()))
        .collect();
    assert!(
        !heads.iter().any(|head| head.starts_with(&[26, 0, 3, 0])),
        "no dialogue from the continent: {heads:?}"
    );
}

/// The bundled Char (pets.json) with the opening package's shampoo (1836) and brush (56778)
/// fitted, as `tests/pet_wired.rs` builds it.
const PET_SPECIES: u32 = 1_000_015;
const SHAMPOO_DEF: i32 = 1_836;
const BRUSH_DEF: i32 = 56_778;
const SHAMPOO_INSTANCE: i32 = 9_003;
const BRUSH_INSTANCE: i32 = 9_004;

fn fitted_roster() -> serde_json::Value {
    let content = PetService::bundled_content().unwrap();
    let mut pets = PetService::new(content.clone()).unwrap();
    let definition = content.definition(PET_SPECIES).unwrap();
    assert_eq!(pets.adopt_vendor_grant(OWNER, &definition, 0).unwrap(), 1);
    let mut document = pets.document_value().unwrap();
    document["pets"][0]["equipment"]["utilities"][1] = serde_json::json!(SHAMPOO_INSTANCE);
    document["pets"][0]["equipment"]["utilities"][4] = serde_json::json!(BRUSH_INSTANCE);
    document
}

fn summon(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![1]].concat()
}

fn groom(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 18], pet_id.to_le_bytes().to_vec(), vec![0; 4]].concat()
}

fn has(packets: &[Vec<u8>], family: u16, subtype: u8) -> bool {
    packets.iter().any(|packet| {
        packet.get(..2) == Some(&family.to_le_bytes()) && packet.get(2) == Some(&subtype)
    })
}

fn says(packets: &[Vec<u8>], text: &str) -> bool {
    packets
        .iter()
        .any(|packet| packet.windows(text.len()).any(|window| window == text.as_bytes()))
}

/// Only the Housing occupation is exempt from the busy rule inside a lot: a groom started on the
/// farm still refuses an NPC click, as it does in the continent, and refuses entering another lot.
#[test]
fn a_groom_on_the_farm_still_blocks_the_click_and_a_second_lot() {
    let content = Content::with_houses();
    let mut state = content.game().state();
    state.inventory.items.push(pet_inventory::item(SHAMPOO_INSTANCE, SHAMPOO_DEF, 2));
    state.inventory.items.push(pet_inventory::item(BRUSH_INSTANCE, BRUSH_DEF, 1));
    let mut game = Gameplay::load(&content.0, Some(state)).unwrap();
    game.adopt_pets(Some(fitted_roster())).unwrap();
    let records = grant(&mut game);
    let farm = farm(&records);
    let condo = (records.iter())
        .find(|record| record.name_id != WILDS_FARM)
        .cloned()
        .unwrap();
    let (chug_guid, _) = chug();

    game.handle_in_world(&enter(farm.instance_guid), &in_world(), 2_000)
        .unwrap();
    let on_farm = on_farm_near_chug();
    game.initialize_world(&on_farm, 3_000).unwrap();
    let summoned = game.handle_in_world(&summon(1), &on_farm, 4_000).unwrap();
    assert!(has(&summoned.replies, 53, 9), "the pet is out on the lot");
    let groomed = game.handle_in_world(&groom(1), &on_farm, 5_000).unwrap();
    assert!(has(&groomed.replies, 53, 20), "the groom session starts: {:?}", groomed.replies.len());

    let clicked = game
        .handle_in_world(&interact(chug_guid), &on_farm, 6_000)
        .unwrap();
    assert!(
        !has(&clicked.replies, 26, 3),
        "no dialogue while grooming"
    );
    assert!(
        says(&clicked.replies, "Finish the active activity before interacting."),
        "the busy line as in the continent"
    );
    let hop = game
        .handle_in_world(&enter(condo.instance_guid), &on_farm, 7_000)
        .unwrap();
    assert!(hop.scene_zoning.is_none(), "no lot hop while grooming");
    assert!(says(&hop.replies, "Finish the active activity before entering a house."));
}

/// House zone ids (1-34) and the regional labels (1-17) share one number space. Inside lot 17
/// (its scene zone is 17) a continent NPC labelled 17 (Farmer Murglot, Wugachug) must not
/// answer, even with the player at his own coordinates; back in the continent he does.
#[test]
fn a_continent_npc_with_the_lots_number_is_out_of_reach_inside_the_lot() {
    const MURGLOT: u64 = 9500000975;
    const LOT: i32 = 17;
    let content = Content::with_houses();
    let mut houses = HousingContent::bundled().unwrap();
    let lot = (houses.lots.iter()).find(|lot| lot.property_id == LOT).unwrap();
    assert!(lot.enabled && lot.zone_id == 17, "lot 17 is enterable with zone 17");
    houses.starter_property_id = LOT;
    houses.starter_kit = None;
    fs::write(content.0.join("houses.json"), serde_json::to_vec(&houses).unwrap()).unwrap();

    let mut game = content.game();
    let records = grant(&mut game);
    let lot = (records.iter()).find(|record| record.name_id != WILDS_FARM).unwrap();
    let entered = game
        .handle_in_world(&enter(lot.instance_guid), &in_world(), 2_000)
        .unwrap();
    assert_eq!(entered.scene_zoning.map(|scene| scene.id), Some(17));
    let at_murglot = CharacterState {
        guid: OWNER,
        zone_id: 17,
        position: [67.18324 + 1.0, 31.33448, -346.3687],
        rotation: [1.0, 0.0, 0.0],
    };
    game.initialize_world(&at_murglot, 3_000).unwrap();
    let result = game.handle_in_world(&interact(MURGLOT), &at_murglot, 4_000);
    let answered = (result.as_ref().ok())
        .is_some_and(|out| out.replies.iter().any(|reply| reply.starts_with(&[26, 0])));
    assert!(!answered, "a continent NPC answered from inside a lot");

    // Back in the continent, at the same spot, he answers.
    game.handle_in_world(&LEAVE, &at_murglot, 5_000).unwrap();
    let in_continent = CharacterState {
        zone_id: 0,
        ..at_murglot
    };
    let clicked = game
        .handle_in_world(&interact(MURGLOT), &in_continent, 6_000)
        .unwrap();
    assert!(
        clicked.replies.iter().any(|reply| reply.starts_with(&[26, 0])),
        "Farmer Murglot answers in the continent"
    );
}

/// The farm's standing stays across a relog: the lot list and the scene are the same after the
/// save is loaded again.
#[test]
fn the_farm_survives_a_reload() {
    let content = Content::with_houses();
    let mut game = content.game();
    let records = grant(&mut game);
    let saved = game.state();
    let mut again = Gameplay::load(&content.0, Some(saved)).unwrap();
    let listed = again
        .handle_in_world(&[127, 0, 14, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], &in_world(), 7_000)
        .unwrap();
    assert_eq!(list(&listed.replies).records, records);
    let entered = again
        .handle_in_world(&enter(farm(&records).instance_guid), &in_world(), 8_000)
        .unwrap();
    assert_eq!(
        entered.scene_zoning.map(|scene| scene.scene),
        Some("farming_wilds_farmstead_01".to_owned())
    );
}
