//! T240/F276: field-enemy kills pay coins and active-job XP once per credited player.
//!
//! Every test drives the production shared-field seams: `handle_shared_field`
//! (attack), `tick_shared_effects` (claim), `Gameplay::load` (content/reconnect),
//! `Gameplay::reload_content` (the dev reload path) and, for a battle win,
//! `handle_in_world` (realm battle/win). The reward table is
//! `data/content/field_kill_rewards.json`; the field actors stay in
//! `field_enemies.json` with their existing schema. F276 adds the disconnect
//! prune (`retain_casters`, the real shared-tick call), the mid-fight reload
//! proof, the battle-reward gate and the shipped level-band assertions. The
//! real-session save guard lives in the gateway binary
//! (`src/bin/osfr_gateway/f276_field_session_save.rs`), because only the
//! binary writes the character save.
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    encounter::{EncounterPhase, SharedFieldAction, SharedFieldState},
    game_store::{self, Snapshot},
    gameplay::{Gameplay, Outcome},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "support/packets.rs"]
mod packets;
#[allow(dead_code)] // Only the shared reply-text extractor is used here.
#[path = "common/s6.rs"]
mod s6;

use packets::{carrier, count, packet};

const ENEMY: u64 = 131000109001;
const ENEMY_TWO: u64 = 131000109002;
const LEVEL_ONE: i32 = 930108;
const WOLF: i32 = 3156;
const NONCOMBAT: i32 = 45;
/// T245's level-5 MID-F1 camp wolf. The shipped row keeps definition 3156, so
/// the kind's override (not the level 5-8 band) owns its payout.
const MID_F1: u64 = 131_000_102_013;
/// `encounter::field_rewards::CREDIT_TTL_MS` (private to the engine).
const CREDIT_TTL_MS: u64 = 10_000;
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(actors: Vec<Value>) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t240-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        copy_missing(&opening_content(), &dir);
        copy_missing(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"),
            &dir,
        );
        // The opening inventory does not carry every optional farming reward;
        // this suite exercises field combat only.
        let _ = fs::remove_file(dir.join("harvesting.json"));
        let fixture = Self { dir };
        fixture.write_field(&json!({"schema_version":1,"enemies":actors}));
        fixture
    }

    fn write_field(&self, field: &Value) {
        fs::write(
            self.dir.join("field_enemies.json"),
            serde_json::to_vec(field).unwrap(),
        )
        .unwrap();
    }

    fn write_rewards(&self, rewards: &Value) {
        fs::write(
            self.dir.join("field_kill_rewards.json"),
            serde_json::to_vec(rewards).unwrap(),
        )
        .unwrap();
    }

    fn save_path(&self, character: &CharacterState) -> PathBuf {
        self.dir.join(format!("character-{}.json", character.guid))
    }
}

impl Drop for Fixture {
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

fn copy_missing(from: &Path, to: &Path) {
    for row in fs::read_dir(from).unwrap() {
        let path = row.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_missing(&path, &target);
        } else if !target.exists() {
            fs::copy(path, target).unwrap();
        }
    }
}

fn source() -> Value {
    json!([{"kind":"reconstructed","evidence":"(R) local test balance"}])
}

/// A field actor with no power-up table: only the kill reward path is used.
fn actor(guid: u64, definition_id: i32, level: u32, position: [f32; 3], max_health: u32) -> Value {
    json!({
        "guid": guid,
        "definition_id": definition_id,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic field reward",
        "texture_alias": "",
        "position": position,
        "max_health": max_health,
        "damage": 1,
        "boss": false,
        "level": level,
        "respawn_seconds": 60,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode":"stationary","reach":2.5,"attack_ms":1000},
        "quest_ids": []
    })
}

/// The shipped shape: one level band and no definition override.
fn levels(coins: u32, job_xp: u32) -> Value {
    json!({
        "schema_version": 1,
        "level_rewards": [
            {"min_level": 1, "max_level": 2, "coins": coins, "job_xp": job_xp, "source": source()}
        ],
        "definition_overrides": []
    })
}

fn character_at(guid: u64, position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn player(
    fixture: &Fixture,
    guid: u64,
    profile: i32,
    position: [f32; 3],
    coins: u32,
    xp: Option<(i32, u32)>,
) -> Gameplay {
    let base = Gameplay::load(&fixture.dir, None).unwrap();
    let mut state = base.state();
    state.inventory.coins = coins;
    {
        let profiles = state.profiles.as_mut().unwrap();
        profiles.unlocked.insert(profile);
        profiles.active = profile;
        profiles.unlocked.extend([35, 43, NONCOMBAT]);
        if let Some((xp_profile, xp)) = xp {
            profiles.experience.insert(xp_profile, xp);
        }
    }
    let mut game = Gameplay::load(&fixture.dir, Some(state)).unwrap();
    game.initialize_world(&character_at(guid, position), 0)
        .unwrap();
    game
}

/// The shipped field row with its shipped spawn group's policy applied inline,
/// because this fixture carries no `spawn_groups`.
fn shipped_field_actor(guid: u64) -> (Value, [f32; 3]) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/field_enemies.json");
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

/// The production reconnect reads the character snapshot back through
/// `game_store::load_or_create` and then loads the gameplay state.
fn reconnect(fixture: &Fixture, character: &CharacterState) -> Gameplay {
    let snapshot =
        game_store::load_or_create(&fixture.save_path(character), character.clone()).unwrap();
    let mut game = Gameplay::load(&fixture.dir, snapshot.gameplay).unwrap();
    game.initialize_world(character, 0).unwrap();
    game.take_shared_fields();
    game
}

fn save(game: &Gameplay, fixture: &Fixture, character: &CharacterState) -> PathBuf {
    let path = fixture.save_path(character);
    game_store::save(
        &path,
        &Snapshot {
            character: character.clone(),
            gameplay: Some(game.state()),
        },
    )
    .unwrap();
    path
}

fn attack(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    known: &BTreeSet<u64>,
    target: u64,
    now_ms: u64,
) -> std::io::Result<Outcome> {
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

fn kill(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    known: &BTreeSet<u64>,
    target: u64,
    mut now_ms: u64,
) -> (Outcome, u64) {
    for _ in 0..10 {
        let out = attack(game, field, character, known, target, now_ms).unwrap();
        if field.health(target) == Some(0) {
            return (out, now_ms);
        }
        now_ms += 700;
    }
    panic!("field enemy did not die");
}

fn switch_job(
    game: &mut Gameplay,
    character: &CharacterState,
    profile: i32,
    now_ms: u64,
) -> Outcome {
    let mut select = vec![26, 0, 13, 0];
    select.extend(profile.to_le_bytes());
    game.handle_in_world(&select, character, now_ms).unwrap()
}

fn bank(game: &Gameplay) -> (u32, BTreeMap<i32, u32>) {
    let state = game.state();
    (state.inventory.coins, state.profiles.unwrap().experience)
}

/// The observable runtime of one field actor; a refusal must not move any of it.
fn field_view(field: &SharedFieldState, guid: u64) -> (Option<u32>, [f32; 3], Option<u64>) {
    (
        field.health(guid),
        field.position(guid).unwrap(),
        field.respawn_at(guid),
    )
}

#[test]
fn a_solo_field_kill_pays_once_and_only_the_active_job() {
    let fixture = Fixture::new(vec![actor(ENEMY, LEVEL_ONE, 1, [0.0; 3], 100)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at(101, at);
    let mut game = player(&fixture, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();
    let known = BTreeSet::from([ENEMY]);

    let (lethal, killed_at) = kill(&mut game, &mut field, &character, &known, ENEMY, 0);
    assert_eq!(field.health(ENEMY), Some(0));
    // Byte-level carriers: 38/19 is one i32 total (client ctor 0x0089e670);
    // 38/14 is profile, rank, progress and three zero fields (reader 0x008d86e0).
    assert_eq!(count(&lethal.replies, 38, 19), 1);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 2, 0, 0, 0].as_slice()
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 5, 0, 0, 0]).as_slice()
    );
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 2);
    assert_eq!(xp.get(&43), Some(&5));
    assert_eq!(xp.get(&35), None, "only the active job is paid");
    assert_eq!(xp.get(&NONCOMBAT), None);

    // Replaying the lethal hit or hitting the corpse must change nothing. The
    // gateway only saves when the action changed state, and the real-session
    // save guard for that rule is `f276_field_session_save` in the binary.
    let state_before = game.state();
    let field_before = field_view(&field, ENEMY);
    let replay = attack(
        &mut game,
        &mut field,
        &character,
        &known,
        ENEMY,
        killed_at + 700,
    )
    .unwrap();
    assert!(!replay.changed && replay.replies.is_empty());
    let corpse = attack(
        &mut game,
        &mut field,
        &character,
        &known,
        ENEMY,
        killed_at + 1_400,
    )
    .unwrap();
    assert!(!corpse.changed && corpse.replies.is_empty());
    assert_eq!(game.state(), state_before);
    assert_eq!(field_view(&field, ENEMY), field_before);
}

#[test]
fn the_mid_level_farnum_camp_pays_the_kind_override_rate() {
    // The level-5 MID-F camp reuses definition 3156. The override exists so the
    // kind's rate cannot be retuned by a level edit; it wins over the level 5-8
    // band for this camp, so the payout stays 2 coins / 5 job XP.
    let (row, home) = shipped_field_actor(MID_F1);
    assert_eq!(row["level"], 5);
    let fixture = Fixture::new(vec![row]);
    let at = [home[0] + 1.0, home[1], home[2]];
    let character = character_at(101, at);
    let mut game = player(&fixture, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();
    let known = BTreeSet::from([MID_F1]);

    let (lethal, _) = kill(&mut game, &mut field, &character, &known, MID_F1, 0);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 2, 0, 0, 0].as_slice(),
        "definition 3156 keeps its 2 coins at level 5, not the band's 7"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 5, 0, 0, 0]).as_slice(),
        "and its 5 job XP, not the level 5-8 band's 15"
    );
    assert_eq!(game.state().inventory.coins, 2);
    assert_eq!(
        game.state().profiles.as_ref().unwrap().experience.get(&43),
        Some(&5)
    );
}

#[test]
fn a_shared_kill_pays_each_credited_player_once() {
    // Definition 3156's override pays 2/5 at level 3, where the band pays 4/10.
    let fixture = Fixture::new(vec![actor(ENEMY, WOLF, 3, [0.0; 3], 500)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let c_at = [2.0, 0.0, 0.0];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let c_character = character_at(303, c_at);
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, 0, None);
    let mut c = player(&fixture, 303, 43, c_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    c.take_shared_fields();
    let known = BTreeSet::from([ENEMY]);

    attack(&mut a, &mut field, &a_character, &known, ENEMY, 0).unwrap();
    attack(&mut b, &mut field, &b_character, &known, ENEMY, 0).unwrap();
    // B's job changes after its hit and before A's kill: the XP follows the
    // job active at the claim, not a profile recorded at damage time.
    let switched = switch_job(&mut b, &b_character, 35, 100);
    assert!(
        switched.changed,
        "{}",
        switched
            .replies
            .iter()
            .map(|row| String::from_utf8_lossy(row))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let (lethal, killed_at) = kill(&mut a, &mut field, &a_character, &known, ENEMY, 200);
    assert_eq!(count(&lethal.replies, 38, 19), 1);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 2, 0, 0, 0].as_slice(),
        "the definition override wins over the level-3 band"
    );
    assert_eq!(a.state().inventory.coins, 2);
    assert_eq!(a.state().profiles.unwrap().experience.get(&43), Some(&5));

    let c_before = bank(&c);
    c.tick_shared_effects(&mut field, &c_character, killed_at + 50)
        .unwrap();
    assert_eq!(
        bank(&c),
        c_before,
        "a bystander with no credit gets nothing"
    );

    let claimed = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 50)
        .unwrap();
    assert_eq!(count(&claimed.replies, 38, 19), 1);
    assert_eq!(
        packet(&claimed.replies, 38, 14),
        carrier(38, 14, &[35, 1, 5, 0, 0, 0]).as_slice(),
        "B's new active job receives the XP"
    );
    let paid = bank(&b);
    assert_eq!(paid.0, 2);
    assert_eq!(paid.1.get(&35), Some(&5));
    assert_eq!(paid.1.get(&43), None);

    // The killer is not duplicated through the credit path.
    a.tick_shared_effects(&mut field, &a_character, killed_at + 60)
        .unwrap();
    assert_eq!(a.state().inventory.coins, 2);
    // A second claim pays nothing.
    let again = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 100)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
    assert_eq!(bank(&b), paid);
}

#[test]
fn a_credit_expires_and_a_bad_row_changes_nothing() {
    let fixture = Fixture::new(vec![actor(ENEMY, LEVEL_ONE, 1, [0.0; 3], 500)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([ENEMY]);

    attack(&mut a, &mut field, &a_character, &known, ENEMY, 0).unwrap();
    attack(&mut b, &mut field, &b_character, &known, ENEMY, 0).unwrap();
    let (_, killed_at) = kill(&mut a, &mut field, &a_character, &known, ENEMY, 700);
    let before = bank(&b);
    let expired = b
        .tick_shared_effects(&mut field, &b_character, killed_at + CREDIT_TTL_MS + 1)
        .unwrap();
    assert!(!expired.changed && expired.replies.is_empty());
    assert_eq!(bank(&b), before, "an expired credit pays nothing");
    b.tick_shared_effects(&mut field, &b_character, killed_at + CREDIT_TTL_MS + 100)
        .unwrap();
    assert_eq!(bank(&b), before, "the expired credit was consumed once");

    // Overlapping bands are refused by the loader, not at kill time.
    fixture.write_rewards(&json!({
        "schema_version": 1,
        "level_rewards": [
            {"min_level": 1, "max_level": 2, "coins": 2, "job_xp": 5, "source": source()},
            {"min_level": 2, "max_level": 3, "coins": 4, "job_xp": 10, "source": source()}
        ],
        "definition_overrides": []
    }));
    assert!(
        Gameplay::load(&fixture.dir, None).is_err(),
        "overlapping bands must fail at load"
    );
    // Duplicate definition overrides are refused by the loader too.
    fixture.write_rewards(&json!({
        "schema_version": 1,
        "level_rewards": [],
        "definition_overrides": [
            {"definition_id": 3156, "coins": 2, "job_xp": 5, "source": source()},
            {"definition_id": 3156, "coins": 4, "job_xp": 10, "source": source()}
        ]
    }));
    assert!(
        Gameplay::load(&fixture.dir, None).is_err(),
        "duplicate overrides must fail at load"
    );
    // The shipped table still loads once the malformed file is gone.
    let shipped = serde_json::from_slice::<Value>(
        &fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/field_kill_rewards.json"),
        )
        .unwrap(),
    )
    .unwrap();
    fixture.write_rewards(&shipped);
    assert!(Gameplay::load(&fixture.dir, None).is_ok());
}

#[test]
fn an_overflowing_reward_refuses_one_player_without_stopping_the_field() {
    // The loader's per-row arithmetic limit: 1000 coins / 1000 job XP.
    let fixture = Fixture::new(vec![
        actor(ENEMY, LEVEL_ONE, 1, [0.0; 3], 500),
        actor(ENEMY_TWO, LEVEL_ONE, 1, [10.0, 0.0, 0.0], 500),
    ]);
    fixture.write_rewards(&levels(1000, 1));
    let a_at = [1.0, 0.0, 0.0];
    let a_second = [11.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let b_second = [9.0, 0.0, 0.0];
    let c_at = [-2.0, 0.0, 0.0];
    let c_second = [8.0, 0.0, 0.0];
    let a_character = character_at(101, a_at);
    let a_character_two = character_at(101, a_second);
    let b_character = character_at(202, b_at);
    let b_character_two = character_at(202, b_second);
    let c_character = character_at(303, c_at);
    let c_character_two = character_at(303, c_second);
    // Two credits of 1000 fit the wallet once; the second cannot be represented.
    let b_coins = i32::MAX as u32 - 1_500;
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, b_coins, None);
    let mut c = player(&fixture, 303, 43, c_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    c.take_shared_fields();
    let known = BTreeSet::from([ENEMY, ENEMY_TWO]);

    for (game, character) in [(&mut b, &b_character), (&mut c, &c_character)] {
        attack(game, &mut field, character, &known, ENEMY, 0).unwrap();
    }
    let (_, first_at) = kill(&mut a, &mut field, &a_character, &known, ENEMY, 200);
    for (game, character) in [(&mut b, &b_character_two), (&mut c, &c_character_two)] {
        attack(
            game,
            &mut field,
            character,
            &known,
            ENEMY_TWO,
            first_at + 700,
        )
        .unwrap();
    }
    let (_, second_at) = kill(
        &mut a,
        &mut field,
        &a_character_two,
        &known,
        ENEMY_TWO,
        first_at + 1_400,
    );
    assert_eq!(a.state().inventory.coins, 2_000);
    assert_eq!(a.state().profiles.unwrap().experience.get(&43), Some(&2));

    // B's one tick holds two rewards; the first is paid, the second refused
    // whole, including any prize item it carried.
    let b_out = b
        .tick_shared_effects(&mut field, &b_character, second_at + 50)
        .unwrap();
    assert!(b_out.changed, "the first reward is still paid");
    assert_eq!(count(&b_out.replies, 38, 19), 1);
    assert_eq!(count(&b_out.replies, 38, 14), 1);
    assert!(
        s6::chat_texts(&b_out.replies)
            .iter()
            .any(|text| text.contains("prize item was not granted")),
        "the whole-reward refusal names the lost prize"
    );
    let b_after = bank(&b);
    assert_eq!(b_after.0, b_coins + 1000);
    assert_eq!(b_after.1.get(&43), Some(&1));
    assert_eq!(b_after.1.len(), 1);

    // Another credited player is paid its whole pair in the same field.
    let c_out = c
        .tick_shared_effects(&mut field, &c_character, second_at + 50)
        .unwrap();
    assert_eq!(count(&c_out.replies, 38, 19), 2);
    assert_eq!(c.state().inventory.coins, 2_000);
    assert_eq!(c.state().profiles.unwrap().experience.get(&43), Some(&2));

    // The field tick still completes for everyone and pays nothing more.
    let quiet = b
        .tick_shared_effects(&mut field, &b_character, second_at + 100)
        .unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
    assert_eq!(bank(&b), b_after);
    assert_eq!(field.health(ENEMY), Some(0));
    assert_eq!(field.health(ENEMY_TWO), Some(0));
}

#[test]
fn an_overflowing_job_xp_refuses_the_whole_payout() {
    let fixture = Fixture::new(vec![actor(ENEMY, LEVEL_ONE, 1, [0.0; 3], 500)]);
    fixture.write_rewards(&levels(1, 1000));
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let b_xp = u32::MAX - 500;
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, 0, Some((35, b_xp)));
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([ENEMY]);

    attack(&mut a, &mut field, &a_character, &known, ENEMY, 0).unwrap();
    attack(&mut b, &mut field, &b_character, &known, ENEMY, 0).unwrap();
    // B's near-max XP belongs to the job it switches to before the claim; a
    // level-1 damage profile keeps the shared enemy alive for A's kill.
    assert!(switch_job(&mut b, &b_character, 35, 100).changed);
    let (_, killed_at) = kill(&mut a, &mut field, &a_character, &known, ENEMY, 200);
    assert_eq!(a.state().inventory.coins, 1, "the killer is paid");
    assert_eq!(a.state().profiles.unwrap().experience.get(&43), Some(&1000));

    let before = bank(&b);
    let out = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 50)
        .unwrap();
    assert!(!out.changed);
    assert_eq!(
        count(&out.replies, 38, 19),
        0,
        "the coins are not half-paid"
    );
    assert_eq!(count(&out.replies, 38, 14), 0, "the XP is not half-paid");
    let text = s6::chat_texts(&out.replies).join("\n");
    assert!(
        text.contains("at its limit") && text.contains("prize item was not granted"),
        "the whole-reward refusal names the lost prize: {text}"
    );
    let after = bank(&b);
    assert_eq!(after, before, "the refused payout changes nothing");
    assert_eq!(after.1.get(&35), Some(&b_xp), "the XP is not half-paid");
    assert_eq!(after.1.get(&43), None);
    assert_eq!(field.health(ENEMY), Some(0));
    // The field and the other players keep running.
    let quiet = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 100)
        .unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
}

#[test]
fn a_field_kill_credit_survives_an_unrelated_content_reload() {
    let fixture = Fixture::new(vec![actor(ENEMY, LEVEL_ONE, 1, [0.0; 3], 500)]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([ENEMY]);

    attack(&mut a, &mut field, &a_character, &known, ENEMY, 0).unwrap();
    attack(&mut b, &mut field, &b_character, &known, ENEMY, 0).unwrap();
    let (_, killed_at) = kill(&mut a, &mut field, &a_character, &known, ENEMY, 700);
    let field_before = field_view(&field, ENEMY);

    // The developer reload runs on a session that is not in the shared field.
    let mut admin = player(&fixture, 999, 43, [0.0; 3], 0, None);
    fixture.write_field(&json!({
        "schema_version": 1,
        "enemies": [actor(ENEMY, LEVEL_ONE, 1, [0.0; 3], 100)]
    }));
    // The reward table changes too; the pending pair is already carried.
    fixture.write_rewards(&levels(9, 9));
    let mut out = Outcome::default();
    let done = admin.reload_content("enemies", &mut out).unwrap();
    assert_eq!(done, "field enemies");
    assert!(
        out.replies
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 2, 0])),
        "the reload respawns the field actor"
    );
    assert_eq!(field_view(&field, ENEMY), field_before);

    // `realm reload` touches only the reloading session: a session loaded now
    // reads the new 9/9 table (the table is loaded with the engine).
    let mut fresh = player(&fixture, 888, 43, [0.0; 3], 0, None);
    let mut fresh_field = fresh.take_shared_fields().unwrap();
    let fresh_character = character_at(888, [0.0; 3]);
    let (fresh_kill, _) = kill(
        &mut fresh,
        &mut fresh_field,
        &fresh_character,
        &known,
        ENEMY,
        0,
    );
    assert_eq!(
        packet(&fresh_kill.replies, 38, 19),
        [38u8, 0, 19, 0, 9, 0, 0, 0].as_slice(),
        "a new engine reads the rewritten table"
    );
    assert_eq!(fresh.state().inventory.coins, 9);
    assert_eq!(
        fresh.state().profiles.unwrap().experience.get(&43),
        Some(&9)
    );

    // The pending credit still pays its carried pair, not the new table.
    let paid = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 50)
        .unwrap();
    assert_eq!(
        packet(&paid.replies, 38, 19),
        [38u8, 0, 19, 0, 2, 0, 0, 0].as_slice(),
        "the credit pays its carried pair, not the reloaded table"
    );
    assert_eq!(
        packet(&paid.replies, 38, 14),
        carrier(38, 14, &[43, 1, 5, 0, 0, 0]).as_slice()
    );
    assert_eq!(b.state().inventory.coins, 2);
    assert_eq!(b.state().profiles.unwrap().experience.get(&43), Some(&5));
    let again = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 100)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
}

#[test]
fn a_credit_survives_a_reconnect_and_expires_once() {
    let fixture = Fixture::new(vec![
        actor(ENEMY, LEVEL_ONE, 1, [0.0; 3], 500),
        actor(ENEMY_TWO, LEVEL_ONE, 1, [10.0, 0.0, 0.0], 500),
    ]);
    let a_at = [1.0, 0.0, 0.0];
    let b_at = [-1.0, 0.0, 0.0];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let known = BTreeSet::from([ENEMY, ENEMY_TWO]);

    attack(&mut a, &mut field, &a_character, &known, ENEMY, 0).unwrap();
    attack(&mut b, &mut field, &b_character, &known, ENEMY, 0).unwrap();
    let (_, killed_at) = kill(&mut a, &mut field, &a_character, &known, ENEMY, 700);
    save(&b, &fixture, &b_character);
    let before = bank(&b);
    drop(b);
    // The real disconnect: the next shared-field tick sees B leave the ready
    // set (`Server::tick_shared_instance` calls `retain_casters(&active)`).
    assert_eq!(
        field.pending_kill_credits(b_character.guid),
        1,
        "the kill credited the non-killer"
    );
    field.retain_casters(&BTreeSet::from([a_character.guid]), killed_at + 25);
    assert_eq!(
        field.pending_kill_credits(b_character.guid),
        1,
        "a disconnect inside the TTL keeps the earned pair"
    );
    let mut b = reconnect(&fixture, &b_character);
    assert_eq!(bank(&b), before, "the save alone carries no credit");
    let claimed = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 50)
        .unwrap();
    assert_eq!(count(&claimed.replies, 38, 19), 1);
    assert_eq!(
        packet(&claimed.replies, 38, 14),
        carrier(38, 14, &[43, 1, 5, 0, 0, 0]).as_slice(),
        "the reconnected player receives the carried job XP"
    );
    assert_eq!(b.state().inventory.coins, 2);
    assert_eq!(
        b.state().profiles.as_ref().unwrap().experience.get(&43),
        Some(&5)
    );
    assert_eq!(field.pending_kill_credits(b_character.guid), 0);
    let paid = bank(&b);
    let again = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 100)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
    assert_eq!(bank(&b), paid);

    // A new credit after the reconnect expires and pays nothing.
    let a_second = character_at(101, [11.0, 0.0, 0.0]);
    let b_second = character_at(202, [9.0, 0.0, 0.0]);
    attack(
        &mut a,
        &mut field,
        &a_second,
        &known,
        ENEMY_TWO,
        killed_at + 20_000,
    )
    .unwrap();
    attack(
        &mut b,
        &mut field,
        &b_second,
        &known,
        ENEMY_TWO,
        killed_at + 20_000,
    )
    .unwrap();
    let (_, second_at) = kill(
        &mut a,
        &mut field,
        &a_second,
        &known,
        ENEMY_TWO,
        killed_at + 20_700,
    );
    let before = bank(&b);
    // The TTL bounds what a departed player keeps: the same prune removes an
    // expired credit instead of retaining it forever.
    field.retain_casters(
        &BTreeSet::from([a_character.guid]),
        second_at + CREDIT_TTL_MS + 1,
    );
    assert_eq!(
        field.pending_kill_credits(b_character.guid),
        0,
        "an expired credit is not retained for a departed player"
    );
    let expired = b
        .tick_shared_effects(&mut field, &b_character, second_at + CREDIT_TTL_MS + 1)
        .unwrap();
    assert!(!expired.changed && expired.replies.is_empty());
    assert_eq!(bank(&b), before);
}

/// Start `battle` through the production dev route and load its arena the way
/// the client does (a zone transfer, then Ready/Loaded -> `initialize_world`),
/// which is what moves the encounter from Loading to Fighting.
fn start_battle(
    game: &mut Gameplay,
    character: &CharacterState,
    battle: u32,
    now_ms: u64,
) -> CharacterState {
    let started = game
        .handle_in_world(
            &chat::local_request(&format!("realm battle {battle}")),
            character,
            now_ms,
        )
        .unwrap();
    let zoning = started.zoning.expect("the battle starts its arena");
    let arena = CharacterState {
        zone_id: zoning.zone_id,
        position: zoning.position,
        rotation: zoning.rotation,
        ..character.clone()
    };
    game.initialize_world(&arena, now_ms + 100).unwrap();
    arena
}

/// F276: the representability gate also covers the one non-field encounter
/// reward (a battle win). A player at the coin cap gets no half-payout and no
/// prize item, one explicit system line that names the lost prize, and the
/// battle still completes; another player is paid.
#[test]
fn a_battle_reward_at_the_coin_cap_is_refused_with_feedback() {
    // Overworld battle 12310: 25 coins, one prize row and no combat-level
    // requirement, started through the production dev route.
    let fixture = Fixture::new(vec![actor(ENEMY, LEVEL_ONE, 1, [0.0; 3], 500)]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at(101, at);
    let mut capped = player(&fixture, 101, 43, at, i32::MAX as u32, None);
    capped.set_dev_tools(true);
    capped.set_role(AccountRole::Admin);
    let arena = start_battle(&mut capped, &character, 12310, 100);
    let before = bank(&capped);
    let items_before = capped.state().inventory.items.len();
    let won = capped
        .handle_in_world(&chat::local_request("realm win"), &arena, 400)
        .unwrap();
    let text = s6::chat_texts(&won.replies).join("\n");
    assert!(won.changed, "the battle still completes: {text}");
    assert!(
        text.contains("at its limit") && text.contains("prize item was not granted"),
        "the player is told the reward and its prize were not granted: {text}"
    );
    let state = capped.state();
    assert_eq!(
        state.inventory.coins,
        i32::MAX as u32,
        "the coins are not half-paid"
    );
    assert_eq!(
        state.inventory.items.len(),
        items_before,
        "the prize item is not half-paid"
    );
    let after = bank(&capped);
    assert_eq!(after, before, "the refused battle payout changes nothing");
    assert_eq!(
        state
            .encounter
            .and_then(|encounter| encounter.active)
            .map(|active| active.phase),
        Some(EncounterPhase::Won)
    );

    // The same battle pays a player below the cap: the refusal above is the
    // cap, not a battle that carries no reward.
    let paid_character = character_at(202, at);
    let mut paid = player(&fixture, 202, 43, at, 0, None);
    paid.set_dev_tools(true);
    paid.set_role(AccountRole::Admin);
    let paid_arena = start_battle(&mut paid, &paid_character, 12310, 100);
    let paid_items_before = paid.state().inventory.items.len();
    paid.handle_in_world(&chat::local_request("realm win"), &paid_arena, 400)
        .unwrap();
    let state = paid.state();
    assert_eq!(state.inventory.coins, 25, "battle 12310 pays its 25 coins");
    assert_eq!(
        state.inventory.items.len(),
        paid_items_before + 1,
        "one prize row is granted"
    );
}
