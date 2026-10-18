use osfr_tools::{
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterPhase},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const BATTLE: u32 = 12303;
const SPAWNER: u64 = 100_001_230_300;
const STONE_HEART: u64 = 100_001_230_301;
const HEARTBREAKERS: [u64; 3] = [100_001_230_302, 100_001_230_303, 100_001_230_304];
const MATCH_MAKER_MALLET: i32 = 76_471;
const BRAWLER: i32 = 43;

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Fixture {
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "stone-heart-hideaway-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();

        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();

        let actor = |guid: u64, definition_id: i32| {
            json!({
                "guid": guid,
                "definition_id": definition_id,
                "model_id": 3,
                "name_id": 100,
                "name": "Synthetic actor",
                "texture_alias": "",
                "position": [2.0, 0.0, 0.0],
                "max_health": 100,
                "damage": 10,
                "boss": false
            })
        };
        let zone = |id: u32, name: &str| {
            json!({
                "zone_id": id,
                "name": name,
                "sky": "",
                "geometry_id": 214,
                "position": [0.0, 0.0, 0.0],
                "rotation": [0.0, 0.0, 0.0],
                "tutorial": id != 0
            })
        };
        let content = json!({
            "schema_version": 1,
            "template_file": "npc.bin",
            "template_sha256": format!("{:x}", Sha256::digest(&template)),
            "world": zone(0, "SyntheticWorld"),
            "world_enemies": [],
            "dungeons": [{
                "id": 411,
                "title_id": 12,
                "description_id": 13,
                "objective_id": 14,
                "zone": zone(411, "SyntheticCave"),
                "guide": null,
                "enemies": [actor(900, 400)],
                "reward_coins": 0,
                "reward_items": []
            }]
        });
        fs::write(
            dir.join("encounters.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self(dir)
    }

    fn load(&self) -> EncounterEngine {
        let mut engine = EncounterEngine::load(&self.0, None).unwrap();
        let battles = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/field_battles.json");
        engine.load_field_battles(&battles).unwrap();
        engine
    }
}

fn world_location() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-940.0, -34.0, -645.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

fn interact(guid: u64) -> Vec<u8> {
    [vec![26, 0, 8, 0], guid.to_le_bytes().to_vec(), vec![0]].concat()
}

fn i32_at(packet: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

#[test]
fn stone_heart_hideaway_enters_clears_rewards_once_and_exits() {
    let fixture = Fixture::new();
    let mut engine = fixture.load();
    engine.set_player_job(BRAWLER, 13);

    let spawner = engine
        .battle_spawners()
        .into_iter()
        .find(|entry| entry.0 == SPAWNER)
        .expect("the legacy Hideaway entrance is registered");
    assert_eq!(spawner.1, "Stone Heart's Hideaway");
    assert_eq!(spawner.2, 426_971);
    assert_eq!(spawner.3, 2056);
    assert_eq!(spawner.4, [-950.582, -34.42554, -651.9259]);

    let screen = engine
        .handle(&interact(SPAWNER), 1, spawner.4, BRAWLER, 0)
        .unwrap();
    let details = screen
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
        .expect("clicking the entrance opens 41/114");
    assert_eq!(i32_at(details, 47), 426_971);
    assert_eq!(i32_at(details, 51), -1);
    assert_eq!(i32_at(details, 55), 426_974);
    assert_eq!(i32_at(details, 59), 5);
    assert_eq!(i32_at(details, 63), 2, "combat profile type");
    assert_eq!(i32_at(details, 67), 4, "combat minigame type");
    assert_eq!(engine.take_pending_battle(), Some(BATTLE));

    let entered = engine.begin(BATTLE, world_location(), BRAWLER).unwrap();
    let arena = entered.zoning.expect("Go zones into the arena");
    assert_eq!(arena.zone_id, 9600);
    assert_eq!(arena.name, "sg_random_encounter_clearing");
    assert_eq!(arena.position, [136.0, -0.5, 155.0]);

    let ready = engine.on_zone_ready(BRAWLER, 1000).unwrap();
    for guid in std::iter::once(STONE_HEART).chain(HEARTBREAKERS) {
        assert!(
            ready
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[35, 0, 2, 0])
                    && packet[4..12] == guid.to_le_bytes()),
            "enemy {guid} spawned"
        );
    }
    assert!(
        ready.replies.iter().any(|packet| packet
            .windows(4)
            .any(|field| field == 426_977_i32.to_le_bytes())),
        "the recovered Defeat Stone Heart objective reaches the client"
    );

    let mut now = 2000;
    let mut win = None;
    for guid in HEARTBREAKERS.into_iter().chain([STONE_HEART]) {
        loop {
            let (health, position) = {
                let active = engine.state().active.as_ref().unwrap();
                let enemy = active
                    .enemies
                    .iter()
                    .find(|enemy| enemy.guid == guid)
                    .unwrap();
                (enemy.health, enemy.position)
            };
            if health == 0 {
                break;
            }
            now += 700;
            let outcome = engine.attack(guid, position, BRAWLER, 0, now).unwrap();
            if guid != STONE_HEART {
                assert!(outcome.rewards.is_empty(), "reward only on the final goal");
            }
            if guid == STONE_HEART && outcome.defeated.contains(&930_022) {
                win = Some(outcome);
            }
        }
    }

    let win = win.expect("the Stone Heart killing blow completes the battle");
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        EncounterPhase::Won
    );
    assert_eq!(win.rewards.len(), 1, "one reward bundle");
    assert_eq!(win.rewards[0].coins, 0);
    assert_eq!(win.rewards[0].job_xp, 50);
    assert_eq!(win.rewards[0].items, vec![(MATCH_MAKER_MALLET, 1)]);
    assert!(
        win.replies
            .iter()
            .any(|packet| packet.starts_with(&[39, 0, 45])),
        "the loot wheel lands on the granted prize"
    );

    let repeated = engine
        .attack(STONE_HEART, [0.0; 3], BRAWLER, 0, now + 700)
        .unwrap();
    assert!(repeated.rewards.is_empty(), "a won fight cannot pay twice");

    let exit = engine
        .handle(&[26, 0, 42, 0], 1, [0.0; 3], BRAWLER, now + 1400)
        .unwrap();
    assert_eq!(exit.completed, Some(BATTLE));
    assert!(exit.rewards.is_empty(), "closing results grants nothing");
    let returned = exit.zoning.expect("results Close returns to the world");
    assert_eq!(returned.zone_id, 0);
    assert_eq!(returned.position, world_location().position);
    assert!(!engine.active(), "the active encounter is cleared");

    let duplicate = engine
        .handle(&[26, 0, 42, 0], 1, [0.0; 3], BRAWLER, now + 2100)
        .unwrap();
    assert!(duplicate.rewards.is_empty());
    assert!(duplicate.zoning.is_none());
    assert!(!duplicate.changed);
}
