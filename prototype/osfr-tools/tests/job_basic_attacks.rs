//! T010: each combat job starts with its client-derived weapon and can defeat
//! a cave enemy through the ordinary action-bar packet.
//!
//! The test content is the preserved private opening package. Its profile
//! starter lists predate the Ninja, Warrior and Wizard weapons, so this also
//! covers the reconstructed (R) fallback to the canonical basic-attack row.
use osfr_tools::{
    abilities::equipped_weapon_matches,
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    encounter::EncounterPhase,
    gameplay::{Gameplay, GameplayState},
    inventory::Inventory,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

const COMBAT_JOBS: [(i32, i32, i32); 6] = [
    (2, 75091, 6),
    (11, 4622, 27),
    (12, 4914, 10),
    (32, 29963, 17),
    (35, 4266, 18),
    (43, 75030, 71),
];

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/opening-anvil-placement-v1/content");
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t010-combat-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let source = entry.unwrap().path();
            if source.is_file() {
                fs::copy(&source, path.join(source.file_name().unwrap())).unwrap();
            }
        }
        // The private campaign carries the first three jobs only. Use the
        // shipped canonical roster to exercise all six without changing the
        // pinned package on disk.
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/opening-abilities.json"),
            path.join("opening-abilities.json"),
        )
        .unwrap();
        patch_weapon_definitions(&path);
        patch_encounter(&path);
        Self(path)
    }

    fn content(&self) -> &Path {
        &self.0
    }
}

/// One harmless 300 HP target makes each test about the job's weapon and
/// action-bar path, not four separate dungeon objectives.
fn patch_encounter(path: &Path) {
    let encounter_path = path.join("encounters.json");
    let mut encounters: Value =
        serde_json::from_slice(&fs::read(&encounter_path).unwrap()).unwrap();
    let cave = encounters["dungeons"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|dungeon| dungeon["id"] == 411)
        .expect("opening cave 411 exists");
    cave["enemies"].as_array_mut().unwrap().truncate(1);
    cave["enemies"][0]["max_health"] = serde_json::json!(300);
    cave["enemies"][0]["damage"] = serde_json::json!(0);
    fs::write(encounter_path, serde_json::to_vec(&encounters).unwrap()).unwrap();
}

/// The pinned early opening package predates the Wizard and Warrior weapons.
/// Add only the two client-derived ids the fallback needs, with the same
/// minimal serialized row shape already used by the other four weapons.
fn patch_weapon_definitions(path: &Path) {
    let inventory_path = path.join("inventory.json");
    let mut inventory: Value = serde_json::from_slice(&fs::read(&inventory_path).unwrap()).unwrap();
    let definitions = inventory["definitions"].as_array_mut().unwrap();
    for row in [
        serde_json::json!({
            "id": 4914,
            "slot": 7,
            "class_id": 10,
            "default_tint": 0,
            "max_stack": -1,
            "model": "wand_ar_ag_weapon_crude.adr",
            "texture": "",
            "tint_alias": "",
            "effect": 0,
            "wield_type": 10,
            "serialized_hex": "32130000000000000000000000000000000000000000000000000000000000000a0000000000000007000000000000000000000000000000",
            "sha256": "28b02e59bcf7c0fe6de8cfa2c429bcf5ebcbf3e1e20d1d9a6917599017ef5f14"
        }),
        serde_json::json!({
            "id": 29963,
            "slot": 7,
            "class_id": 17,
            "default_tint": 0,
            "max_stack": -1,
            "model": "club_ar_ag_weapon_cudgel.adr",
            "texture": "",
            "tint_alias": "",
            "effect": 0,
            "wield_type": 12,
            "serialized_hex": "0b75000000000000000000000000000000000000000000000000000000000000110000000000000007000000000000000000000000000000",
            "sha256": "dd8b3607d16fd953e370c2af82240ee38bb5a5f4f2aef8d42b1ddc0a51799e96"
        }),
    ] {
        if !definitions
            .iter()
            .any(|definition| definition["id"] == row["id"])
        {
            definitions.push(row);
        }
    }
    fs::write(inventory_path, serde_json::to_vec(&inventory).unwrap()).unwrap();
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn command(family: u16, sub: u16, fields: &[i32]) -> Vec<u8> {
    [
        family.to_le_bytes().to_vec(),
        sub.to_le_bytes().to_vec(),
        fields
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect(),
    ]
    .concat()
}

fn basic(target: u64) -> Vec<u8> {
    [command(36, 10, &[1, 0, 0]), target.to_le_bytes().to_vec()].concat()
}

fn hit(out: &osfr_tools::gameplay::Outcome) -> bool {
    out.replies.iter().any(|packet| {
        packet.starts_with(&[32, 0, 7, 0])
            && packet.get(4..12) == Some(1_u64.to_le_bytes().as_slice())
    })
}

fn fresh_game(fixture: &Fixture, job: i32) -> (Gameplay, CharacterState, GameplayState) {
    let base = Gameplay::load(fixture.content(), None).unwrap();
    let mut seed = base.state();
    let profiles = seed.profiles.as_mut().unwrap();
    profiles.unlocked.extend([2, 11, 12, 32, 35, 43]);
    profiles.active = 1;
    for equipment in &mut seed.inventory.equipment {
        if equipment.profile_id == job && equipment.slot == 7 {
            equipment.item_id = 0;
        }
    }
    let location = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0; 3],
    };
    let mut game = Gameplay::load(fixture.content(), Some(seed.clone())).unwrap();
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    let out = game
        .handle_in_world(&command(26, 13, &[job]), &location, 100)
        .unwrap();
    assert!(
        out.replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 21, 0])),
        "job {job} activation"
    );
    (game, location, seed)
}

fn equipped_weapon(fixture: &Fixture, game: &Gameplay, job: i32, class: i32) -> i32 {
    let state = game.state().inventory;
    let equipped = state
        .equipment
        .iter()
        .find(|entry| entry.profile_id == job && entry.slot == 7)
        .expect("the job has a weapon slot");
    assert_ne!(equipped.item_id, 0, "job {job} has a starter weapon");
    let item = state
        .items
        .iter()
        .find(|item| item.id == equipped.item_id)
        .expect("equipped weapon instance exists");
    let definition = item.definition_id;
    let inventory = Inventory::load(fixture.content(), Some(state)).unwrap();
    assert!(
        equipped_weapon_matches(&inventory, job, Some(class)),
        "job {job} starter weapon passes the live cast gate"
    );
    definition
}

fn start_cave(game: &mut Gameplay, location: &CharacterState) -> ([f32; 3], u64, u32) {
    let out = game
        .handle_in_world(&chat::local_request("realm battle 411"), location, 200)
        .unwrap();
    assert!(!out.replies.is_empty(), "opening cave 411 exists");
    let active = game
        .state()
        .encounter
        .and_then(|encounter| encounter.active)
        .expect("the cave encounter starts");
    let enemy = active.enemies.first().expect("the cave has an enemy");
    (enemy.position, enemy.guid, enemy.health)
}

fn kill_with_basic(
    game: &mut Gameplay,
    location: &CharacterState,
    target: u64,
    starting_health: u32,
) {
    for attempt in 0..16 {
        let out = game
            .handle_in_world(&basic(target), location, 1_000 + attempt * 700)
            .unwrap();
        if attempt == 0 {
            assert!(hit(&out), "the basic attack produces a player hit");
        }
        let encounter = game.state().encounter.unwrap();
        let enemy = encounter
            .active
            .as_ref()
            .unwrap()
            .enemies
            .iter()
            .find(|enemy| enemy.guid == target)
            .unwrap();
        assert!(
            enemy.health < starting_health,
            "job basic attack dealt damage"
        );
        if encounter.active.as_ref().unwrap().phase == EncounterPhase::Won {
            assert_eq!(enemy.health, 0);
            return;
        }
    }
    panic!("job basic attack did not defeat the cave enemy");
}

fn job_kills_cave_enemy(job: i32, weapon: i32, class: i32) {
    let fixture = Fixture::new();
    let (mut game, mut location, _) = fresh_game(&fixture, job);
    assert_eq!(
        equipped_weapon(&fixture, &game, job, class),
        weapon,
        "job {job} starter weapon"
    );
    let (position, target, health) = start_cave(&mut game, &location);
    location.zone_id = 411;
    location.position = position;
    game.initialize_world(&location, 300).unwrap();
    kill_with_basic(&mut game, &location, target, health);
}

macro_rules! starter_test {
    ($name:ident, $job:expr, $weapon:expr, $class:expr) => {
        #[test]
        fn $name() {
            job_kills_cave_enemy($job, $weapon, $class);
        }
    };
}

starter_test!(ninja_starter_weapon_kills_cave_enemy, 2, 75091, 6);
starter_test!(medic_starter_weapon_kills_cave_enemy, 11, 4622, 27);
starter_test!(wizard_starter_weapon_kills_cave_enemy, 12, 4914, 10);
starter_test!(warrior_starter_weapon_kills_cave_enemy, 32, 29963, 17);
starter_test!(archer_starter_weapon_kills_cave_enemy, 35, 4266, 18);
starter_test!(brawler_starter_weapon_kills_cave_enemy, 43, 75030, 71);

#[test]
fn shipped_rows_name_each_jobs_starter_weapon() {
    let rows: Value =
        serde_json::from_str(include_str!("../data/content/opening-abilities.json")).unwrap();
    for (job, weapon, class) in COMBAT_JOBS {
        let row = rows["basics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["profile_id"] == job)
            .unwrap_or_else(|| panic!("profile {job} basic row"));
        assert_eq!(row["weapon_definition_id"], weapon, "profile {job}");
        assert_eq!(row["weapon_class"], class, "profile {job}");
    }
}

#[test]
fn clothed_warrior_without_a_weapon_gets_the_starter_cudgel() {
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, 32);
    game.handle_in_world(&command(42, 2, &[7, 32]), &location, 200)
        .unwrap();
    let given = game
        .handle_in_world(&chat::local_request("realm give 46644"), &location, 300)
        .unwrap();
    assert!(!given.replies.is_empty());
    let helmet = game
        .state()
        .inventory
        .items
        .iter()
        .find(|item| item.definition_id == 46644)
        .expect("the Warrior chest was granted")
        .id;
    game.handle_in_world(&command(42, 3, &[helmet, 32, 1]), &location, 400)
        .unwrap();
    assert!(
        game.state()
            .inventory
            .equipment
            .iter()
            .any(|entry| entry.profile_id == 32 && entry.slot == 1 && entry.item_id != 0),
        "the fixture save carries a Warrior clothing slot"
    );
    game.handle_in_world(&command(26, 13, &[32]), &location, 500)
        .unwrap();
    assert_eq!(equipped_weapon(&fixture, &game, 32, 17), 29963);
}

#[test]
fn unequipped_basic_is_refused_without_combat_change() {
    let fixture = Fixture::new();
    let (mut game, mut location, _) = fresh_game(&fixture, 2);
    let (position, target, _) = start_cave(&mut game, &location);
    location.zone_id = 411;
    location.position = position;
    game.initialize_world(&location, 300).unwrap();
    game.handle_in_world(&command(42, 2, &[7, 2]), &location, 400)
        .unwrap();
    let before = game.state();
    let out = game
        .handle_in_world(&basic(target), &location, 1_000)
        .unwrap();
    assert!(!hit(&out), "unarmed attack produces no player hit");
    assert_eq!(
        game.state(),
        before,
        "a refused attack mutates no combat state"
    );
}
