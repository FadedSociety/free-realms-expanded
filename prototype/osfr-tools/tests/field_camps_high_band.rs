//! T296: the level 11-14 high field camps and the 14-20 kill-reward band.
//!
//! Content slice. The two new camps repeat four W102 terrain-checked centres,
//! the existing `field_roamer` preset and the shipped wolf/shaman definitions.
//! The reward band is one appended row in `field_kill_rewards.json`. No
//! production behaviour, packet or schema changed.
//!
//! Client evidence for the existing paths this suite asserts:
//! - 35/2 AddNpc reader `0x00921A10`: the u64 actor id is the first field and
//!   there is no per-actor level field.
//! - 38/19 coin balance: one i32, client constructor `0x0089E670`.
//! - 38/14 job update: six i32, client reader `0x008D86E0`.

use osfr_tools::{
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterOutcome, SharedFieldAction, SharedFieldState},
    gameplay::{Gameplay, GameplayState, Outcome},
    inventory::{EquippedItem, InventoryItem},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;

use field_enemy_fixture::{Fixture as EngineFixture, content_dir, content_path, spawned};

/// The four new actors in file order: GUID, authored level, spawn area and the
/// byte-identical W102 centre they repeat.
const HIGH: [(u64, u32, &str, [f32; 3]); 4] = [
    (131000102021, 11, "HIGH-F3", [-1962.89, -46.92, 284.723]),
    (131000102022, 12, "HIGH-F2", [-1887.867, -51.14, 291.159]),
    (131000102023, 13, "HIGH-T3", [-447.084, 3.03, 1128.348]),
    (131000102024, 14, "HIGH-T2", [-351.103, 2.33, 1048.674]),
];
const W102_FOR: [&str; 4] = ["W102-F3", "W102-F2", "W102-T3", "W102-T2"];
const REFERENCE_ROW: [u64; 4] = [131000102003, 131000102002, 131000102012, 131000102011];
/// The source W102 rows' community spawn anchors, repeated by the new rows.
const REFERENCE_GUID: [u64; 4] = [100000004166, 100000004165, 100000003149, 100000003150];

/// The 24 pre-T296 field actors in file order: 12 starter, 4 quest-gated and 8
/// MID-level. The four HIGH actors follow them.
const EXISTING_ACTORS: [u64; 24] = [
    100000004154,
    100000004155,
    5064655448674664449,
    5064655448674664450,
    131000102001,
    131000102002,
    131000102003,
    131000102004,
    131000102005,
    131000102006,
    131000102007,
    131000102008,
    131000102009,
    131000102010,
    131000102011,
    131000102012,
    131000102013,
    131000102014,
    131000102015,
    131000102016,
    131000102017,
    131000102018,
    131000102019,
    131000102020,
];

/// The eight pre-T296 spawn groups in file order.
const EXISTING_GROUPS: [&str; 8] = [
    "starting_farnum_wolves",
    "starting_highroad_hooligans",
    "starting_queensfields_bixies",
    "starting_wilds_shamans",
    "mid_farnum_wolves",
    "mid_highroad_hooligans",
    "mid_wilds_shamans",
    "mid_queensfields_bixies",
];

/// The four pre-T296 reward bands, unchanged.
const EXISTING_BANDS: [(u32, u32, u32, u32); 4] =
    [(1, 2, 2, 5), (3, 4, 4, 10), (5, 8, 7, 15), (9, 13, 12, 25)];
/// The appended 14-20 band.
const HIGH_BAND: (u32, u32, u32, u32) = (14, 20, 18, 35);

const HIGH_SHAMAN_13: u64 = 131000102023;
const HIGH_SHAMAN_14: u64 = 131000102024;
const HIGH_SHAMAN_14_HOME: [f32; 3] = [-351.103, 2.33, 1048.674];
const HIGH_SHAMAN_13_HOME: [f32; 3] = [-447.084, 3.03, 1128.348];
const BIXIE_10: u64 = 131000102019;
const BIXIE_10_HOME: [f32; 3] = [-664.762, 2.4, 299.608];
const LEVEL_TWENTY_GUID: u64 = 131000110020;
/// `encounter::CREDIT_TTL_MS`; the constant is private to the engine.
const CREDIT_TTL_MS: u64 = 10_000;
static SERIAL: AtomicU64 = AtomicU64::new(0);

fn shipped_field() -> Value {
    serde_json::from_slice(&fs::read(content_path()).unwrap()).unwrap()
}

fn shipped_rewards() -> Value {
    serde_json::from_slice(&fs::read(content_dir().join("field_kill_rewards.json")).unwrap())
        .unwrap()
}

fn actor<'a>(field: &'a Value, guid: u64) -> &'a Value {
    field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["guid"].as_u64() == Some(guid))
        .unwrap_or_else(|| panic!("field actor {guid} missing"))
}

fn group<'a>(field: &'a Value, id: &str) -> &'a Value {
    field["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("field spawn group {id} missing"))
}

/// Compare authored decimal coordinates at the runtime's `f32` precision.
fn position(value: &Value) -> [f32; 3] {
    let points = value.as_array().unwrap();
    [
        points[0].as_f64().unwrap() as f32,
        points[1].as_f64().unwrap() as f32,
        points[2].as_f64().unwrap() as f32,
    ]
}

fn max_health(out: &EncounterOutcome, guid: u64) -> Option<u32> {
    out.replies
        .iter()
        .find(|packet| {
            packet.len() >= 20
                && packet.starts_with(&[35, 0, 5, 0])
                && packet[4..12] == guid.to_le_bytes()
        })
        .map(|packet| u32::from_le_bytes(packet[16..20].try_into().unwrap()))
}

/// The shipped row plus its spawn group's policy, because the reward fixture
/// carries no `spawn_groups`.
fn shipped_actor(guid: u64) -> Value {
    let field = shipped_field();
    let mut row = actor(&field, guid).clone();
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
    row
}

fn level_twenty_actor() -> Value {
    json!({
        "guid": LEVEL_TWENTY_GUID,
        "definition_id": 930108,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic field reward",
        "texture_alias": "",
        "position": [0.0, 0.0, 0.0],
        // Authored level-one HP; the loader scales it by level 20 (x14.27).
        "max_health": 35,
        "damage": 0,
        "boss": false,
        "level": 20,
        "respawn_seconds": 60,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode": "stationary", "reach": 2.5, "attack_ms": 1000},
        "quest_ids": []
    })
}

fn source() -> Value {
    json!([{"kind":"reconstructed","evidence":"(R) local test balance"}])
}

fn band_row(min_level: u32, max_level: u32, coins: u32, job_xp: u32) -> Value {
    json!({
        "min_level": min_level,
        "max_level": max_level,
        "coins": coins,
        "job_xp": job_xp,
        "source": source()
    })
}

fn overlap_table() -> Value {
    json!({
        "schema_version": 1,
        "level_rewards": [band_row(1, 2, 2, 5), band_row(2, 3, 4, 10)],
        "definition_overrides": []
    })
}

fn overflow_table() -> Value {
    json!({
        "schema_version": 1,
        "level_rewards": [band_row(1, 2, 1001, 5)],
        "definition_overrides": []
    })
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

/// The production load root: the opening reference supplies encounters,
/// profiles and the opening; the bundled content fills the rest, including the
/// shipped reward table. The field file is the one under test.
struct RewardFixture {
    dir: PathBuf,
}

impl RewardFixture {
    fn new(actors: Vec<Value>) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t296-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        copy_missing(&opening_content(), &dir);
        copy_missing(&content_dir(), &dir);
        // The opening inventory does not carry every optional farming reward;
        // this suite exercises field combat only.
        let _ = fs::remove_file(dir.join("harvesting.json"));
        let fixture = Self { dir };
        fixture.write_field(&json!({"schema_version": 1, "enemies": actors}));
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

    fn base_state(&self) -> GameplayState {
        Gameplay::load(&self.dir, None).unwrap().state()
    }
}

impl Drop for RewardFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn character_at(guid: u64, position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn player_state(
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
    profiles.unlocked.extend([35, 43, 45]);
    if let Some((xp_profile, xp)) = xp {
        profiles.experience.insert(xp_profile, xp);
    }
    state
}

fn player(
    fixture: &RewardFixture,
    base: &GameplayState,
    guid: u64,
    profile: i32,
    position: [f32; 3],
    coins: u32,
    xp: Option<(i32, u32)>,
) -> Gameplay {
    let state = player_state(base, profile, coins, xp);
    let mut game = Gameplay::load(&fixture.dir, Some(state)).unwrap();
    game.initialize_world(&character_at(guid, position), 0)
        .unwrap();
    game
}

/// The Archer's starter bow (definition 4266, class 18, native weapon slot 7)
/// in the opening item catalogue. Profile 35's initial save carries no weapon.
fn equip_archer_weapon(state: &mut GameplayState) {
    let instance = 760_000_296;
    state.inventory.items.push(InventoryItem {
        definition_id: 4266,
        tint: 0,
        id: instance,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    state
        .inventory
        .equipment
        .retain(|row| !(row.profile_id == 35 && row.slot == 7));
    state.inventory.equipment.push(EquippedItem {
        profile_id: 35,
        slot: 7,
        item_id: instance,
        category: 7,
    });
}

fn attack(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    target: u64,
    now_ms: u64,
) -> Outcome {
    let known = BTreeSet::from([target]);
    game.handle_shared_field(
        &SharedFieldAction::Select {
            target,
            choice: 103,
        },
        field,
        &known,
        character,
        now_ms,
    )
    .unwrap()
}

/// One shared-field kill through the production attack path. The loop is
/// bounded; a level-14 actor dies in seven 100-damage basic attacks.
fn kill(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    target: u64,
    mut now_ms: u64,
) -> (Outcome, u64) {
    for _ in 0..12 {
        let out = attack(game, field, character, target, now_ms);
        if field.health(target) == Some(0) {
            return (out, now_ms);
        }
        now_ms += 700;
    }
    panic!("field actor {target} did not die");
}

fn count(replies: &[Vec<u8>], family: u16, subtype: u16) -> usize {
    replies
        .iter()
        .filter(|packet| {
            packet.starts_with(&family.to_le_bytes()) && packet[2..4] == subtype.to_le_bytes()
        })
        .count()
}

/// The one reply of a family/subtype, asserted byte-for-byte.
fn packet<'a>(replies: &'a [Vec<u8>], family: u16, subtype: u16) -> &'a [u8] {
    replies
        .iter()
        .find(|packet| {
            packet.starts_with(&family.to_le_bytes()) && packet[2..4] == subtype.to_le_bytes()
        })
        .unwrap_or_else(|| panic!("missing {family}/{subtype}"))
}

fn carrier(family: u16, subtype: u16, values: &[i32]) -> Vec<u8> {
    let mut bytes = vec![
        family as u8,
        (family >> 8) as u8,
        subtype as u8,
        (subtype >> 8) as u8,
    ];
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn bank(game: &Gameplay) -> (u32, BTreeMap<i32, u32>) {
    let state = game.state();
    (state.inventory.coins, state.profiles.unwrap().experience)
}

#[test]
fn the_loader_accepts_the_high_camps_and_exposes_them() {
    let mut engine = EngineFixture::shipped().load();
    let out = engine.on_zone_ready(11, 0).unwrap();
    let spawned = spawned(&out);
    let mut scaled = Vec::new();
    for (guid, level, area, _) in HIGH {
        assert!(
            spawned.contains(&guid),
            "{area} ({guid}) must spawn through the existing 35/2 AddNpc path"
        );
        assert_eq!(
            engine.field_level(guid),
            Some(level),
            "the loader must consume {area}'s authored level"
        );
        assert_eq!(engine.behaviour(guid).unwrap().leash, 12.0);
        let hp = max_health(&out, guid).expect("35/5 HP packet");
        assert!(
            hp > 100,
            "{area} authored 100 HP at level 1; the loader must scale level {level} once, got {hp}"
        );
        scaled.push(hp);
    }
    assert!(
        scaled.windows(2).all(|pair| pair[0] < pair[1]),
        "each higher authored level must raise the scaled HP: {scaled:?}"
    );
}

#[test]
fn the_high_camps_use_verified_w102_centres_and_unique_guids() {
    let field = shipped_field();
    let enemies = field["enemies"].as_array().unwrap();
    let groups = field["spawn_groups"].as_array().unwrap();
    assert_eq!(enemies.len(), EXISTING_ACTORS.len() + HIGH.len());
    assert_eq!(groups.len(), EXISTING_GROUPS.len() + 2);

    let mut guids = BTreeSet::new();
    let mut areas = BTreeSet::new();
    for row in enemies {
        assert!(
            guids.insert(row["guid"].as_u64().unwrap()),
            "duplicate actor GUID {}",
            row["guid"]
        );
        if let Some(area) = row.get("spawn_area") {
            assert!(
                areas.insert(area["id"].as_str().unwrap().to_owned()),
                "duplicate spawn area id {}",
                area["id"]
            );
        }
    }
    assert_eq!(areas.len(), 24, "24 actors carry a spawn area");
    assert_eq!(guids.len(), 28);

    let starting_wolf = actor(&field, 131000102001);
    let starting_shaman = actor(&field, 131000102010);
    for (index, (guid, level, area, centre)) in HIGH.into_iter().enumerate() {
        let row = actor(&field, guid);
        let w102 = W102_FOR[index];
        assert_eq!(row["spawn_area"]["id"], area);
        assert_eq!(
            position(&row["position"]).map(f32::to_bits),
            centre.map(f32::to_bits),
            "{area} must stay byte-identical to {w102}"
        );
        assert_eq!(
            position(&row["spawn_area"]["centre"]).map(f32::to_bits),
            centre.map(f32::to_bits)
        );
        assert_eq!(row["spawn_area"]["radius"], 6.0);
        assert_eq!(row["spawn_area"]["count"], 1);
        assert_eq!(row["level"], level);
        assert_eq!(row["quest_ids"], json!([]), "{guid} must be always-on");
        assert_eq!(row["boss"], false);
        assert_eq!(row["power_up_table"], "standard");
        assert_eq!(row["reference_spawn_guid"], REFERENCE_GUID[index]);

        let (counterpart, group_id) = if index < 2 {
            (starting_wolf, "high_farnum_wolves")
        } else {
            (starting_shaman, "high_wilds_shamans")
        };
        for key in [
            "definition_id",
            "model_id",
            "name_id",
            "name",
            "max_health",
            "damage",
            "texture_alias",
        ] {
            assert_eq!(
                row[key], counterpart[key],
                "{guid} {key} must match its starting-band counterpart"
            );
        }

        let group = group(&field, group_id);
        assert_eq!(
            group["members"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|member| member.as_u64() == Some(guid))
                .count(),
            1,
            "{guid} must be in {group_id} exactly once"
        );
        let band = group["level_band"].as_array().unwrap();
        assert!(
            band[0].as_u64().unwrap() <= u64::from(level)
                && u64::from(level) <= band[1].as_u64().unwrap(),
            "{guid} level {level} outside {group_id}'s band"
        );
        assert_eq!(group["zone_id"], 0);
        assert_eq!(
            group["region"],
            if index < 2 {
                "Farnum outskirts"
            } else {
                "Lakeshore / northeast Wilds"
            }
        );
        assert_eq!(group["respawn_seconds"], if index < 2 { 120 } else { 150 });
        assert_eq!(group["aggro_range"], 8.0);
        assert_eq!(group["leash_radius"], 12.0);
        assert_eq!(group["behaviour"], "field_roamer");

        let kinds: BTreeSet<&str> = row["source"]
            .as_array()
            .unwrap()
            .iter()
            .map(|source| source["kind"].as_str().unwrap())
            .collect();
        for kind in ["client", "reference", "legacy-data", "reconstructed"] {
            assert!(kinds.contains(kind), "{guid} lost its {kind} provenance");
        }
        let reconstructed = row["source"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap();
        let evidence = reconstructed["evidence"].as_str().unwrap();
        assert!(evidence.starts_with("(R)"));
        assert!(
            evidence.contains(&format!("row {}", REFERENCE_ROW[index])),
            "{area} must name its source W102 row: {evidence}"
        );
        assert!(
            evidence.contains(w102),
            "{area} must name the reused {w102} centre: {evidence}"
        );
        assert!(
            evidence.contains(&format!("level {level}")),
            "{area} must record its level policy: {evidence}"
        );
        assert!(
            evidence.contains(&format!("{} s respawn", group["respawn_seconds"])),
            "{area} must record its respawn policy: {evidence}"
        );
        assert!(group["source"].as_array().unwrap().iter().any(|source| {
            source["kind"] == "reconstructed"
                && source["evidence"]
                    .as_str()
                    .is_some_and(|text| text.starts_with("(R)"))
        }));
    }
}

#[test]
fn the_existing_camps_and_rewards_are_unchanged() {
    let field = shipped_field();
    let enemies = field["enemies"].as_array().unwrap();
    let groups = field["spawn_groups"].as_array().unwrap();
    assert_eq!(enemies.len(), EXISTING_ACTORS.len() + HIGH.len());
    assert_eq!(groups.len(), EXISTING_GROUPS.len() + 2);
    assert_eq!(
        enemies
            .iter()
            .take(EXISTING_ACTORS.len())
            .map(|row| row["guid"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        EXISTING_ACTORS.as_slice(),
        "the 24 pre-T296 actors keep their order and identity"
    );
    assert_eq!(
        enemies
            .iter()
            .skip(EXISTING_ACTORS.len())
            .map(|row| row["guid"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        HIGH.iter().map(|(guid, ..)| *guid).collect::<Vec<_>>(),
        "the four HIGH actors are appended"
    );
    assert_eq!(
        groups
            .iter()
            .take(EXISTING_GROUPS.len())
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        EXISTING_GROUPS.as_slice(),
        "the eight pre-T296 groups keep their order and identity"
    );
    assert_eq!(
        groups
            .iter()
            .skip(EXISTING_GROUPS.len())
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["high_farnum_wolves", "high_wilds_shamans"],
        "the two HIGH groups are appended"
    );

    let rewards = shipped_rewards();
    let bands = rewards["level_rewards"].as_array().unwrap();
    let values = bands
        .iter()
        .map(|band| {
            (
                band["min_level"].as_u64().unwrap() as u32,
                band["max_level"].as_u64().unwrap() as u32,
                band["coins"].as_u64().unwrap() as u32,
                band["job_xp"].as_u64().unwrap() as u32,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 5);
    assert_eq!(&values[..4], EXISTING_BANDS.as_slice());
    assert_eq!(values[4], HIGH_BAND);
    assert_eq!(
        rewards["definition_overrides"].as_array().unwrap().len(),
        1,
        "the wolf definition override is unchanged"
    );
}

#[test]
fn a_level_fourteen_kill_pays_the_new_band_once() {
    let fixture = RewardFixture::new(vec![shipped_actor(HIGH_SHAMAN_14)]);
    let base = fixture.base_state();
    let at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let character = character_at(101, at);
    let mut game = player(&fixture, &base, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();

    let (lethal, killed_at) = kill(&mut game, &mut field, &character, HIGH_SHAMAN_14, 0);
    assert_eq!(field.health(HIGH_SHAMAN_14), Some(0));
    assert_eq!(count(&lethal.replies, 38, 19), 1);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "the level-14 shaman pays the 14-20 band's 18 coins"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice(),
        "and 35 active-job XP"
    );
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 18);
    assert_eq!(xp.get(&43), Some(&35));

    // A replay or a corpse hit changes neither the wallet nor the field.
    let before = bank(&game);
    for now in [killed_at + 700, killed_at + 1_400] {
        let replay = attack(&mut game, &mut field, &character, HIGH_SHAMAN_14, now);
        assert!(!replay.changed && replay.replies.is_empty());
    }
    assert_eq!(bank(&game), before);
    let quiet = game
        .tick_shared_effects(&mut field, &character, killed_at + 50)
        .unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
    assert_eq!(bank(&game), before, "the killer is not paid twice");
}

#[test]
fn the_active_job_not_the_actor_kind_receives_the_xp() {
    let fixture = RewardFixture::new(vec![shipped_actor(HIGH_SHAMAN_14)]);
    let base = fixture.base_state();
    let at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let character = character_at(202, at);
    // Profile 35's initial save has no weapon; give it the Archer starter bow
    // so the production weapon gate admits its basic attack.
    let mut state = player_state(&base, 35, 0, Some((43, 7)));
    equip_archer_weapon(&mut state);
    let mut game = Gameplay::load(&fixture.dir, Some(state)).unwrap();
    game.initialize_world(&character, 0).unwrap();
    let mut field = game.take_shared_fields().unwrap();

    let (_, _) = kill(&mut game, &mut field, &character, HIGH_SHAMAN_14, 0);
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 18);
    assert_eq!(xp.get(&35), Some(&35), "the active job receives the XP");
    assert_eq!(
        xp.get(&43),
        Some(&7),
        "the inactive combat profile is untouched"
    );
    assert_eq!(
        xp.get(&930025),
        None,
        "the defeated actor's kind is not a job profile"
    );
}

#[test]
fn the_level_thirteen_and_ten_controls_keep_the_nine_thirteen_band() {
    let fixture = RewardFixture::new(vec![shipped_actor(HIGH_SHAMAN_13), shipped_actor(BIXIE_10)]);
    let base = fixture.base_state();
    let shaman_at = [
        HIGH_SHAMAN_13_HOME[0] + 1.0,
        HIGH_SHAMAN_13_HOME[1],
        HIGH_SHAMAN_13_HOME[2],
    ];
    let bixie_at = [BIXIE_10_HOME[0] + 1.0, BIXIE_10_HOME[1], BIXIE_10_HOME[2]];
    let a_character = character_at(101, shaman_at);
    let b_character = character_at(202, bixie_at);
    let mut a = player(&fixture, &base, 101, 43, shaman_at, 0, None);
    let mut b = player(&fixture, &base, 202, 43, bixie_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();

    let (lethal, _) = kill(&mut a, &mut field, &a_character, HIGH_SHAMAN_13, 0);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 12, 0, 0, 0].as_slice(),
        "a level-13 shaman stays on the 9-13 band"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 25, 0, 0, 0]).as_slice()
    );
    assert_eq!(a.state().inventory.coins, 12);
    assert_eq!(a.state().profiles.unwrap().experience.get(&43), Some(&25));

    let (lethal, _) = kill(&mut b, &mut field, &b_character, BIXIE_10, 7_000);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 12, 0, 0, 0].as_slice(),
        "the level-10 bixie still pays the 9-13 band"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 25, 0, 0, 0]).as_slice()
    );
    assert_eq!(b.state().inventory.coins, 12);
    assert_eq!(b.state().profiles.unwrap().experience.get(&43), Some(&25));
}

#[test]
fn a_level_twenty_actor_pays_the_new_band() {
    let fixture = RewardFixture::new(vec![level_twenty_actor()]);
    let base = fixture.base_state();
    let at = [1.0, 0.0, 0.0];
    let character = character_at(101, at);
    let mut game = player(&fixture, &base, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();

    let (lethal, _) = kill(&mut game, &mut field, &character, LEVEL_TWENTY_GUID, 0);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "level 20 is the top of the 14-20 band"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice()
    );
    assert_eq!(game.state().inventory.coins, 18);
}

#[test]
fn a_level_outside_the_bands_pays_nothing_without_an_error() {
    let fixture = RewardFixture::new(vec![level_twenty_actor()]);
    // A valid table with no band covering 20: the lookup, not the loader,
    // decides the payout.
    fixture.write_rewards(&json!({
        "schema_version": 1,
        "level_rewards": [band_row(1, 13, 12, 25)],
        "definition_overrides": []
    }));
    let base = fixture.base_state();
    let at = [1.0, 0.0, 0.0];
    let character = character_at(101, at);
    let mut game = player(&fixture, &base, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();

    let (lethal, _) = kill(&mut game, &mut field, &character, LEVEL_TWENTY_GUID, 0);
    assert!(
        lethal.changed,
        "the server-authoritative kill still happens"
    );
    assert_eq!(field.health(LEVEL_TWENTY_GUID), Some(0));
    assert_eq!(count(&lethal.replies, 38, 19), 0);
    assert_eq!(count(&lethal.replies, 38, 14), 0);
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 0);
    assert_eq!(xp.get(&43), None, "an uncovered level pays no job XP");
}

#[test]
fn an_unclaimed_high_camp_credit_expires_or_is_lost_on_leaving_and_pays_nothing() {
    let fixture = RewardFixture::new(vec![shipped_actor(HIGH_SHAMAN_14)]);
    let base = fixture.base_state();
    let a_at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let b_at = [
        HIGH_SHAMAN_14_HOME[0] - 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let mut a = player(&fixture, &base, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, &base, 202, 43, b_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();

    attack(&mut a, &mut field, &a_character, HIGH_SHAMAN_14, 0);
    attack(&mut b, &mut field, &b_character, HIGH_SHAMAN_14, 0);
    let (_, killed_at) = kill(&mut a, &mut field, &a_character, HIGH_SHAMAN_14, 700);
    assert_eq!(a.state().inventory.coins, 18, "the killer is paid at once");

    let before = bank(&b);
    let expired = b
        .tick_shared_effects(&mut field, &b_character, killed_at + CREDIT_TTL_MS + 1)
        .unwrap();
    assert!(!expired.changed && expired.replies.is_empty());
    assert_eq!(bank(&b), before, "an expired credit pays nothing");
    let again = b
        .tick_shared_effects(&mut field, &b_character, killed_at + CREDIT_TTL_MS + 100)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
    assert_eq!(bank(&b), before, "the expired credit was consumed once");
}

#[test]
fn two_players_share_a_high_camp_kill_and_each_is_paid_once() {
    let fixture = RewardFixture::new(vec![shipped_actor(HIGH_SHAMAN_14)]);
    let base = fixture.base_state();
    let a_at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let b_at = [
        HIGH_SHAMAN_14_HOME[0] - 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let mut a = player(&fixture, &base, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, &base, 202, 43, b_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let full_health = field.health(HIGH_SHAMAN_14).unwrap();

    attack(&mut a, &mut field, &a_character, HIGH_SHAMAN_14, 0);
    attack(&mut b, &mut field, &b_character, HIGH_SHAMAN_14, 0);
    let (lethal, killed_at) = kill(&mut a, &mut field, &a_character, HIGH_SHAMAN_14, 700);
    assert_eq!(count(&lethal.replies, 38, 19), 1);
    assert_eq!(a.state().inventory.coins, 18);
    assert_eq!(a.state().profiles.unwrap().experience.get(&43), Some(&35));

    let claimed = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 50)
        .unwrap();
    assert_eq!(
        packet(&claimed.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "the credited second player is paid the same band"
    );
    assert_eq!(
        packet(&claimed.replies, 38, 14),
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice()
    );
    assert_eq!(b.state().inventory.coins, 18);
    assert_eq!(b.state().profiles.unwrap().experience.get(&43), Some(&35));
    let paid = bank(&b);
    let again = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 100)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
    assert_eq!(bank(&b), paid, "each player is paid once");
    assert_eq!(field.health(HIGH_SHAMAN_14), Some(0), "one shared kill");

    // The group's 150 s timer respawns the actor, not one millisecond early.
    let players = vec![
        a.field_participant(&a_character).unwrap(),
        b.field_participant(&b_character).unwrap(),
    ];
    a.tick_shared_motion(&mut field, &players, killed_at + 149_900, 100)
        .unwrap();
    assert_eq!(field.health(HIGH_SHAMAN_14), Some(0));
    a.tick_shared_motion(&mut field, &players, killed_at + 150_100, 100)
        .unwrap();
    assert_eq!(field.health(HIGH_SHAMAN_14), Some(full_health));
    assert_eq!(field.respawn_at(HIGH_SHAMAN_14), None);
}

#[test]
fn the_band_table_has_no_gap_or_overlap_and_bounded_values() {
    let rewards = shipped_rewards();
    let bands = rewards["level_rewards"].as_array().unwrap();
    assert_eq!(bands.len(), 5);
    let mut next = 1;
    for band in bands {
        let min = band["min_level"].as_u64().unwrap() as u32;
        let max = band["max_level"].as_u64().unwrap() as u32;
        let coins = band["coins"].as_u64().unwrap() as u32;
        let job_xp = band["job_xp"].as_u64().unwrap() as u32;
        assert_eq!(min, next, "bands must be sorted, contiguous and gapless");
        assert!(min >= 1 && min <= max && max <= 20);
        assert!(coins <= 1000 && job_xp <= 1000);
        assert!(!band["source"].as_array().unwrap().is_empty());
        next = max + 1;
    }
    assert_eq!(next, 21, "level 21 is outside every band");
    let high = &bands[4];
    assert_eq!(high["min_level"], HIGH_BAND.0);
    assert_eq!(high["max_level"], HIGH_BAND.1);
    assert_eq!(high["coins"], HIGH_BAND.2);
    assert_eq!(high["job_xp"], HIGH_BAND.3);

    // A synthetic overlapping or overflowing band is refused at load, before
    // any actor exists or any payout can be computed.
    let fixture = RewardFixture::new(vec![shipped_actor(HIGH_SHAMAN_14)]);
    for bad in [overlap_table(), overflow_table()] {
        fixture.write_rewards(&bad);
        assert!(
            EncounterEngine::load(&fixture.dir, None).is_err(),
            "a malformed band must fail the engine load"
        );
    }
    fixture.write_rewards(&shipped_rewards());
    assert!(EncounterEngine::load(&fixture.dir, None).is_ok());
}

#[test]
fn a_malformed_high_camp_or_band_is_refused_atomically() {
    let fixture = EngineFixture::copied();
    let mut engine = fixture.load();
    let before = engine.field_teleport_targets();
    let behaviour = engine.behaviour(HIGH_SHAMAN_14).cloned().unwrap();
    let original = shipped_field();
    let mutations: [(&str, fn(&mut Value)); 3] = [
        ("duplicate member", |field| {
            let group = field["spawn_groups"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|group| group["id"] == "high_wilds_shamans")
                .unwrap();
            group["members"] = json!([HIGH_SHAMAN_13, HIGH_SHAMAN_13]);
        }),
        ("level outside band", |field| {
            let row = field["enemies"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["guid"] == HIGH_SHAMAN_14)
                .unwrap();
            row["level"] = json!(15);
        }),
        ("position off its centre", |field| {
            let row = field["enemies"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["guid"] == HIGH_SHAMAN_14)
                .unwrap();
            row["position"] = json!([0.0, 0.0, 0.0]);
        }),
    ];
    for (label, mutate) in mutations {
        let mut field = original.clone();
        mutate(&mut field);
        fs::write(fixture.field_path(), serde_json::to_vec(&field).unwrap()).unwrap();
        assert!(
            engine.reload_field_enemies(fixture.field_path()).is_err(),
            "{label} must be refused"
        );
        assert_eq!(
            engine.field_teleport_targets(),
            before,
            "{label} changed the live field"
        );
        assert_eq!(
            engine.behaviour(HIGH_SHAMAN_14),
            Some(&behaviour),
            "{label} changed the live behaviour"
        );
    }

    // The malformed band is likewise a load refusal, not a payout-time panic.
    let rewards = RewardFixture::new(vec![shipped_actor(HIGH_SHAMAN_14)]);
    rewards.write_rewards(&overlap_table());
    assert!(EncounterEngine::load(&rewards.dir, None).is_err());
    rewards.write_rewards(&shipped_rewards());
    assert!(EncounterEngine::load(&rewards.dir, None).is_ok());
}
