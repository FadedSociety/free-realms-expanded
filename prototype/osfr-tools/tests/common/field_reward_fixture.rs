//! Shared test helper: a production-shaped field-reward load.
//!
//! Include with `#[path = "common/field_reward_fixture.rs"] mod field_reward_fixture;`.
//! Test-only; the production path never builds this root. The opening reference
//! supplies encounters, profiles and the opening; the bundled content fills the
//! rest, including the shipped `field_kill_rewards.json`. Each test writes the
//! field file it exercises.
#![allow(dead_code)] // each suite drives a subset of the helpers

use osfr_tools::{
    character_store::CharacterState,
    encounter::{SharedFieldAction, SharedFieldState},
    gameplay::{Gameplay, GameplayState, Outcome},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "content_copy.rs"]
mod content_copy;

/// The engine's shared-kill credit window, linked so a value change cannot desync.
pub const CREDIT_TTL_MS: u64 = osfr_tools::encounter::field_rewards::CREDIT_TTL_MS;
/// A profile that is not a combat job.
pub const NONCOMBAT: i32 = 45;

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// The production load root for one reward test.
pub struct RewardFixture {
    dir: PathBuf,
}

impl RewardFixture {
    fn root() -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "field-reward-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        content_copy::copy_missing(&opening_content(), &dir);
        dir
    }

    pub fn new(actors: Vec<Value>) -> Self {
        let dir = Self::root();
        content_copy::copy_missing(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content"),
            &dir,
        );
        // The opening inventory does not carry every optional farming reward;
        // these suites exercise field combat only.
        let _ = fs::remove_file(dir.join("harvesting.json"));
        let fixture = Self { dir };
        fixture.write_field(&json!({"schema_version": 1, "enemies": actors}));
        fixture
    }

    /// The opening fixture plus only the named bundled files. A loop that logs
    /// in repeatedly cannot pay the full overlay's load cost per login, and it
    /// needs the opening quest story out of its exact reward deltas.
    pub fn minimal(actors: Vec<Value>, bundled: &[&str]) -> Self {
        let dir = Self::root();
        let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        for name in bundled {
            fs::copy(content.join(name), dir.join(name)).unwrap();
        }
        let fixture = Self { dir };
        fixture.write_field(&json!({"schema_version": 1, "enemies": actors}));
        fixture
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn write_field(&self, field: &Value) {
        fs::write(
            self.dir.join("field_enemies.json"),
            serde_json::to_vec(field).unwrap(),
        )
        .unwrap();
    }

    pub fn write_rewards(&self, rewards: &Value) {
        fs::write(
            self.dir.join("field_kill_rewards.json"),
            serde_json::to_vec(rewards).unwrap(),
        )
        .unwrap();
    }

    /// The real-session save path for `game_store` round trips.
    pub fn save_path(&self, character: &CharacterState) -> PathBuf {
        self.dir.join(format!("character-{}.json", character.guid))
    }

    pub fn base_state(&self) -> GameplayState {
        Gameplay::load(&self.dir, None).unwrap().state()
    }
}

impl Drop for RewardFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn opening_content() -> PathBuf {
    std::env::var_os("OSFR_OPENING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-anvil-placement-v1/content")
        })
}

pub fn source() -> Value {
    json!([{"kind":"reconstructed","evidence":"(R) local test balance"}])
}

pub fn character_at(guid: u64, position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

/// A character save at `profile`, with profiles 35/43/45 unlocked so a test may
/// switch to a noncombat job.
pub fn player_state(
    base: &GameplayState,
    profile: i32,
    coins: u32,
    xp: Option<(i32, u32)>,
) -> GameplayState {
    let mut state = base.clone();
    state.inventory.coins = coins;
    let profiles = state.profiles.as_mut().unwrap();
    profiles.unlocked.insert(profile);
    profiles.active = profile;
    profiles.unlocked.extend([35, 43, NONCOMBAT]);
    if let Some((xp_profile, xp)) = xp {
        profiles.experience.insert(xp_profile, xp);
    }
    state
}

pub fn player(
    fixture: &RewardFixture,
    guid: u64,
    profile: i32,
    position: [f32; 3],
    coins: u32,
    xp: Option<(i32, u32)>,
) -> Gameplay {
    let state = player_state(&fixture.base_state(), profile, coins, xp);
    let mut game = Gameplay::load(fixture.dir(), Some(state)).unwrap();
    game.initialize_world(&character_at(guid, position), 0)
        .unwrap();
    game
}

/// The shipped field row with its shipped spawn group's policy applied inline,
/// because this fixture carries no `spawn_groups`.
pub fn shipped_field_actor(guid: u64) -> (Value, [f32; 3]) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/field_enemies.json");
    let field: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let mut row = field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["guid"].as_u64() == Some(guid))
        .unwrap_or_else(|| panic!("shipped field actor {guid}"))
        .clone();
    let group = field["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["members"].as_array().unwrap().contains(&json!(guid)))
        .unwrap_or_else(|| panic!("shipped spawn group for {guid}"));
    for key in [
        "respawn_seconds",
        "aggro_range",
        "leash_radius",
        "behaviour",
    ] {
        row[key] = group[key].clone();
    }
    let position = serde_json::from_value(row["position"].clone()).unwrap();
    (row, position)
}

pub fn attack(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    known: &BTreeSet<u64>,
    target: u64,
    now_ms: u64,
) -> io::Result<Outcome> {
    game.handle_shared_field(
        &SharedFieldAction::Select {
            target,
            choice: 103,
        },
        field,
        known,
        character,
        now_ms,
    )
}

/// A solo attack: the client knows only the actor it selected.
pub fn attack_one(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    target: u64,
    now_ms: u64,
) -> Outcome {
    attack(
        game,
        field,
        character,
        &BTreeSet::from([target]),
        target,
        now_ms,
    )
    .unwrap()
}

/// One shared-field kill through the production attack path. The loop is
/// bounded; the strongest shipped field actor (a level-20 elite, 2569 HP) dies
/// in twenty-six 100-damage hits, inside the 30 s participation window.
pub fn kill(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    known: &BTreeSet<u64>,
    target: u64,
    mut now_ms: u64,
) -> (Outcome, u64) {
    for _ in 0..40 {
        let out = attack(game, field, character, known, target, now_ms).unwrap();
        if field.health(target) == Some(0) {
            return (out, now_ms);
        }
        now_ms += 700;
    }
    panic!("field actor {target} did not die");
}

/// A solo kill: the client knows only the actor it selected.
pub fn kill_one(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    target: u64,
    now_ms: u64,
) -> (Outcome, u64) {
    kill(
        game,
        field,
        character,
        &BTreeSet::from([target]),
        target,
        now_ms,
    )
}

pub fn bank(game: &Gameplay) -> (u32, BTreeMap<i32, u32>) {
    let state = game.state();
    (state.inventory.coins, state.profiles.unwrap().experience)
}
