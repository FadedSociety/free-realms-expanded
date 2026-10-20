//! T222: the held combat power-up on its own action-bar key.
//!
//! The client's `ActionBarSlot3` (default `3`) dispatches job-bar slot 2 and
//! sends 36/10 bar 1 slot 2 (see `notes/Research/Power-up Use Key.md`). These
//! tests drive that real request through `Gameplay::handle_in_world`, with the
//! shipped field Hooligans and a guaranteed drop loaded by the production
//! `Gameplay::load` content root. Nothing here calls the power-up code directly.
use osfr_tools::{
    character_store::CharacterState,
    encounter::SharedFieldAction,
    gameplay::{Gameplay, Outcome},
    power_ups::{PowerUps, Slot},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// Shipped `data/content/field_enemies.json` Hooligan Wolf whose `standard`
/// table this fixture replaces with a guaranteed Flame Wave. It is always
/// present (no quest gate).
const TARGET: u64 = 131_000_102_001;
const TARGET_AT: [f32; 3] = [-1899.204, -50.59, 307.902];
/// A clear point 40 m east of the F1 camp. T245 appends a level-5 wolf on the
/// same F1 centre, so the Flame Wave is cast here where nothing is in range.
const CLEAR_AT: [f32; 3] = [-1859.204, -50.59, 307.902];
/// T245's level-5 MID-F1 wolf: the same centre as `TARGET`, so a cast at the
/// authored F1 pickup point has a live target.
const MID_F1: u64 = 131_000_102_013;
/// A second always-present Hooligan, where the player walks after the pickup;
/// its own position is inside the Flame Wave radius.
const SECOND: u64 = 131_000_102_004;
const SECOND_AT: [f32; 3] = [-1779.455, -28.88, 562.833];
/// Brawler XP for level 15, where its slot-2 job ability is unlocked.
const LEVEL_15_XP: u32 = 1400;

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Fixture {
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t222-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        for entry in fs::read_dir(reference()).unwrap() {
            let source = entry.unwrap().path();
            if source.is_file() {
                fs::copy(&source, dir.join(source.file_name().unwrap())).unwrap();
            }
        }
        // The canonical combat roster (starter weapon classes), as the other
        // job tests do; the pinned opening package predates some weapons.
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/opening-abilities.json"),
            dir.join("opening-abilities.json"),
        )
        .unwrap();
        // Own NPC content root: this makes `Gameplay::load` read the fixture's
        // field enemies, job bar and power-up tables (the production loader),
        // instead of the bundled `data/content` fallback.
        let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        fs::create_dir_all(dir.join("npcs")).unwrap();
        fs::write(dir.join("npc_spawns_extra.json"), r#"{"records":[]}"#).unwrap();
        fs::write(
            dir.join("npcs/anchor.json"),
            r#"{"records":[{"npc_id":1,"name":"T222 anchor","zone_id":0,"model_id":1,"name_string_id":1,"position":[0.0,0.0,0.0],"placement":"legacy_data"}]}"#,
        )
        .unwrap();
        for name in [
            "zones.json",
            "field_enemies.json",
            "job_abilities.json",
            "behaviours.json",
        ] {
            fs::copy(bundled.join(name), dir.join(name)).unwrap();
        }
        // Every kind and table stays byte-identical; only the shipped standard
        // table becomes a guaranteed Flame Wave.
        let mut power_ups: Value =
            serde_json::from_str(include_str!("../data/content/power_ups.json")).unwrap();
        power_ups["tables"]["standard"] =
            json!({"guaranteed":["flame_wave"],"source":"T222 test table"});
        fs::write(dir.join("power_ups.json"), power_ups.to_string()).unwrap();
        Self(dir)
    }

    fn content(&self) -> &Path {
        &self.0
    }

    fn slot(&self, kind: &str) -> Slot {
        PowerUps::load(self.0.join("power_ups.json"))
            .unwrap()
            .slot(kind)
            .unwrap()
    }
}

fn who() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: TARGET_AT,
        rotation: [0.0; 3],
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

/// Bar 1, slot 0 (the basic attack) with a target guid.
fn basic(target: u64) -> Vec<u8> {
    [command(36, 10, &[1, 0, 0]), target.to_le_bytes().to_vec()].concat()
}

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState { position, ..who() }
}

/// Bar 1, slot `slot`, target mode 0 (self cast uses the player's own guid).
fn cast(slot: i32, target: u64) -> Vec<u8> {
    [
        command(36, 10, &[1, slot, 0]),
        target.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn failed(out: &Outcome) -> bool {
    out.replies.len() == 1 && out.replies[0].starts_with(&[36, 0, 1, 0])
}

fn toolbar(out: &Outcome) -> Option<&Vec<u8>> {
    out.replies
        .iter()
        .rev()
        .find(|p| p.starts_with(&[36, 0, 5, 0]))
}

/// Decode the eight toolbar slots; index 2 is the power-up's key (`3`).
fn slot_ids(bar: &[u8]) -> [Option<i32>; 8] {
    assert!(bar.starts_with(&[36, 0, 5, 0]));
    assert_eq!(i32::from_le_bytes(bar[8..12].try_into().unwrap()), 8);
    let mut at = 12;
    let mut ids = [None; 8];
    for id in &mut ids {
        let kind = i32::from_le_bytes(bar[at..at + 4].try_into().unwrap());
        match kind {
            0 => at += 4,
            3 => {
                *id = Some(i32::from_le_bytes(bar[at + 4..at + 8].try_into().unwrap()));
                at += 41;
            }
            other => panic!("unexpected toolbar entry type {other}"),
        }
    }
    assert_eq!(at, bar.len());
    ids
}

fn definition_ids(out: &Outcome) -> Vec<i32> {
    out.replies
        .iter()
        .filter(|p| p.len() == 160 && p.starts_with(&[36, 0, 13, 0]))
        .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
        .collect()
}

fn definition(out: &Outcome, id: i32) -> Option<&Vec<u8>> {
    out.replies
        .iter()
        .find(|p| p.starts_with(&[36, 0, 13, 0]) && p[4..8] == id.to_le_bytes())
}

/// Model ids of the pickups (`AddNpc`) in one outcome.
fn pickup_models(out: &Outcome) -> Vec<i32> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .filter(|p| {
            u64::from_le_bytes(p[4..12].try_into().unwrap()) & 0xffff_ffff_0000_0000
                == 0x5057_5550_0000_0000
        })
        .map(|p| i32::from_le_bytes(p[16..20].try_into().unwrap()))
        .collect()
}

/// Targets of the combat damage results in one outcome.
fn hit_targets(out: &Outcome) -> Vec<u64> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[32, 0, 7, 0]))
        .map(|p| u64::from_le_bytes(p[20..28].try_into().unwrap()))
        .collect()
}

fn enemy_health(out: &Outcome, guid: u64) -> Option<i32> {
    out.replies
        .iter()
        .rev()
        .find(|p| p.starts_with(&[35, 0, 5, 0]) && p[4..12] == guid.to_le_bytes())
        .map(|p| i32::from_le_bytes(p[12..16].try_into().unwrap()))
}

/// Load the reference package, force a Brawler level and switch to it through
/// the real 26/13 request. Returns the bar right after the switch.
fn fresh_game(fixture: &Fixture, xp: u32) -> (Gameplay, CharacterState, Vec<u8>) {
    let base = Gameplay::load(fixture.content(), None).unwrap();
    let mut seed = base.state();
    seed.profiles.as_mut().unwrap().experience.insert(43, xp);
    let mut game = Gameplay::load(fixture.content(), Some(seed)).unwrap();
    let location = who();
    let switched = game
        .handle_in_world(&command(26, 13, &[43]), &location, 100)
        .unwrap();
    let bar = toolbar(&switched)
        .expect("a job switch sends the job bar")
        .clone();
    game.initialize_world(&location, 200).unwrap();
    (game, location, bar)
}

/// Hit the always-present Hooligan Wolf until the guaranteed Flame Wave drop
/// appears.
fn kill_target(game: &mut Gameplay, location: &CharacterState, start: u64) -> u64 {
    for attempt in 0..12 {
        let now = start + attempt * 700;
        let out = game.handle_in_world(&basic(TARGET), location, now).unwrap();
        if pickup_models(&out).contains(&1949) {
            return now;
        }
    }
    panic!("the Hooligan Wolf never dropped the guaranteed power-up");
}

#[test]
fn a_held_power_up_uses_its_own_key_once_and_restores_the_job_bar() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, LEVEL_15_XP);
    let before = slot_ids(&before_bar);
    assert!(
        before[2].is_some(),
        "the level-15 Brawler has a slot-2 job ability"
    );
    let slot = fixture.slot("flame_wave");

    // The pickup puts the kind on its own key slot and leaves every other
    // job slot exactly where it was.
    let kill = kill_target(&mut game, &location, 1_000);
    let picked = game.tick(&location, kill + 200).unwrap();
    assert_eq!(game.held_power_up(), Some("flame_wave"));
    let held_bar = toolbar(&picked).expect("the pickup resends the bar");
    let held = slot_ids(held_bar);
    assert_eq!(held[2], Some(slot.definition_id));
    assert_eq!(held[..2], before[..2]);
    assert_eq!(held[3..], before[3..]);
    let power_up_definition = definition(&picked, slot.definition_id)
        .expect("the power-up definition precedes its toolbar");
    assert_eq!(
        i32::from_le_bytes(power_up_definition[10..14].try_into().unwrap()),
        slot.name_id
    );
    assert_eq!(
        i32::from_le_bytes(power_up_definition[18..22].try_into().unwrap()),
        slot.icon_id
    );

    // The client's key request (bar 1, slot 2) consumes the kind and restores
    // the authored job row. The T245 mid-level camp shares the F1 centre, so
    // this cast happens at a clear point with no target in the Flame Wave radius.
    let use_at = kill + 1_000;
    let used = game
        .handle_in_world(&cast(2, 1), &at(CLEAR_AT), use_at)
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    let restored = toolbar(&used).expect("the use resends the restored bar");
    assert_eq!(slot_ids(restored), before);
    assert!(!definition_ids(&used).contains(&slot.definition_id));
    assert!(hit_targets(&used).is_empty(), "no target in range");
}

#[test]
fn the_f1_pickup_point_hits_the_co_located_mid_camp_wolf() {
    // The authored scenario: the pickup is taken at F1, where T245's level-5
    // MID-F1 wolf now stands beside the dead starter wolf. A level-1 Flame Wave
    // hits it, leaves it alive and fabricates no drop.
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = kill_target(&mut game, &location, 1_000);
    game.tick(&location, kill + 200).unwrap();
    assert_eq!(game.held_power_up(), Some("flame_wave"));

    let used = game
        .handle_in_world(&cast(2, 1), &at(TARGET_AT), kill + 1_000)
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    let hits = hit_targets(&used);
    assert!(
        hits.contains(&MID_F1),
        "the F1 cast hits the co-located MID-F1 wolf: {hits:?}"
    );
    assert!(
        !hits.contains(&TARGET),
        "the dead starter wolf is not hit again"
    );
    let health = enemy_health(&used, MID_F1).expect("the hit reports the wolf's health");
    assert!(
        health > 0,
        "a level-1 Flame Wave leaves the level-5 wolf alive"
    );
    assert!(
        pickup_models(&used).is_empty(),
        "a surviving wolf drops nothing"
    );
}

#[test]
fn the_flame_wave_burn_ticks_and_a_repeat_changes_nothing() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, 0);
    let kill = kill_target(&mut game, &location, 1_000);
    game.tick(&location, kill + 200).unwrap();
    assert_eq!(game.held_power_up(), Some("flame_wave"));
    // At level 1 the tier-1 Flame Wave (damage_by_tier[0] = 1.0) leaves the
    // 220-health Hooligan alive, so the authored burn can tick on it.
    let moved = at(SECOND_AT);
    let use_at = kill + 1_000;
    let used = game.handle_in_world(&cast(2, 1), &moved, use_at).unwrap();
    assert_eq!(game.held_power_up(), None);
    assert_eq!(
        slot_ids(toolbar(&used).expect("the bar")),
        slot_ids(&before_bar)
    );
    let hits = hit_targets(&used);
    let mut unique = hits.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), hits.len(), "one initial hit per target");
    assert!(unique.contains(&SECOND), "the Flame Wave hits {hits:?}");
    let after_hit = enemy_health(&used, SECOND).expect("the target's health");
    assert!(after_hit > 0, "the target survives the initial hit");
    let ticked = game.tick(&moved, use_at + 1_100).unwrap();
    let after_burn = enemy_health(&ticked, SECOND).expect("the burn tick reports health");
    assert!(
        after_burn < after_hit,
        "the burn from power_ups.json ticks: {after_hit} -> {after_burn}"
    );

    // The kind is gone; the same key is now the (empty) job slot and a repeat
    // request is refused without consuming or damaging anything.
    let before = game.state();
    let replay = game
        .handle_in_world(&cast(2, 1), &moved, use_at + 2_000)
        .unwrap();
    assert!(failed(&replay), "nothing is held");
    assert_eq!(game.state(), before);
}

#[test]
fn a_use_request_with_nothing_held_changes_nothing() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, 0);
    assert_eq!(
        slot_ids(&before_bar)[2],
        None,
        "level-1 Brawler has no slot-2 job ability"
    );
    let before = game.state();
    let out = game.handle_in_world(&cast(2, 1), &location, 1_000).unwrap();
    assert!(failed(&out), "an empty key slot is refused");
    assert_eq!(game.state(), before, "a refused use changes nothing");
    assert_eq!(game.held_power_up(), None);
}

#[test]
fn a_knocked_out_player_keeps_the_power_up_and_changes_nothing() {
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = kill_target(&mut game, &location, 1_000);
    game.tick(&location, kill + 200).unwrap();
    assert_eq!(game.held_power_up(), Some("flame_wave"));
    game.receive_field_damage(u32::MAX / 2, kill + 300, true)
        .unwrap();
    assert_eq!(game.state().encounter.unwrap().player_health, 0);
    let before = game.state();
    let refused = game
        .handle_in_world(&cast(2, 1), &location, kill + 400)
        .unwrap();
    assert!(failed(&refused), "a knocked-out player gets 36/1");
    assert_eq!(game.state(), before, "the refusal changes nothing");
    assert_eq!(
        game.held_power_up(),
        Some("flame_wave"),
        "the kind stays held"
    );
}

#[test]
fn a_held_power_up_is_session_state_and_a_reconnect_clears_it() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, 0);
    let kill = kill_target(&mut game, &location, 1_000);
    game.tick(&location, kill + 200).unwrap();
    assert_eq!(game.held_power_up(), Some("flame_wave"));

    // The held kind is never written to the character save (W158 design): a
    // reconnect starts with the job bar and no kind.
    let saved = game.state();
    let mut reconnected = Gameplay::load(fixture.content(), Some(saved)).unwrap();
    let init = reconnected.initialize_world(&location, 5_000).unwrap();
    assert_eq!(reconnected.held_power_up(), None);
    assert_eq!(
        slot_ids(toolbar(&init).expect("world entry sends the bar"))[2],
        slot_ids(&before_bar)[2],
        "the reconnect shows the authored job bar"
    );
    let before = reconnected.state();
    let out = reconnected
        .handle_in_world(&cast(2, 1), &location, 6_000)
        .unwrap();
    assert!(failed(&out) && !out.changed);
    assert_eq!(reconnected.state(), before);
}

#[test]
fn the_shared_field_route_uses_the_same_key_and_consumes_once() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, 0);
    let kill = kill_target(&mut game, &location, 1_000);
    game.tick(&location, kill + 200).unwrap();
    assert_eq!(game.held_power_up(), Some("flame_wave"));
    // Move the field simulation to its runtime, exactly as the gateway does.
    let mut field = game
        .take_shared_fields()
        .expect("the field enemies move to the shared runtime");
    let moved = at(SECOND_AT);
    let action = SharedFieldAction::Ability { slot: 2, target: 1 };
    let used = game
        .handle_shared_field(
            &action,
            &mut field,
            &BTreeSet::from([TARGET, SECOND]),
            &moved,
            kill + 1_000,
        )
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    assert_eq!(
        slot_ids(toolbar(&used).expect("the use resends the restored bar")),
        slot_ids(&before_bar)
    );
    let hit = hit_targets(&used);
    assert!(
        hit.contains(&SECOND),
        "the nearby shared enemy takes the hit: {hit:?}"
    );
    assert!(
        hit.iter().all(|guid| *guid != TARGET),
        "the dead enemy is not hit again"
    );
}
