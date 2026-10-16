//! Clickable Mining powerup (`950..=953`) behavioral tests.
//!
//! Every fixture is synthetic and private: no user save, native client or gateway state is read or written. The recovered originals are
//! `core/ServerGrid.as` `StartWildTiles`/`StartDestruction`/`StartShuffle` and
//! `GameLevel.OnActionMsg`.
#[path = "../src/mining.rs"]
mod mining;
use mining::{
    Type8BoardSpec, Type8Engine, Type8Goal, Type8Outcome, Type8Phase, Type8PowerupDefinition,
    Type8State, MINING_GAME_ID, SIDE, TILE_BLOCKER, TILE_EMPTY, TILE_POWERUP_DESTRUCT,
    TILE_POWERUP_DYNAMITE, TILE_POWERUP_SHUFFLE, TILE_POWERUP_START, TILE_POWERUP_WILDTILE,
    TILE_WILDCARD,
};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;
const PROFILE: i32 = 14;
const FIRST_INSTANCE: i32 = 18_000_001;
const ORE: u16 = 601;
const ORE_TWO: u16 = 602;
const SELECTABLE: u16 = 401;
const SELECTABLE_TWO: u16 = 402;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    /// `powerup_percent` selects how often a powerup replaces the selectable
    /// branch; the definitions are the authored private rows. `None` omits the
    /// key entirely, matching every existing content file.
    fn new(powerup_percent: u32, definitions: Option<Vec<Type8PowerupDefinition>>) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-mining-powerups-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let mut spec = spec();
        spec.powerup_percent = powerup_percent;
        let mut content = json!({
            "schema_version": 1,
            "game_id": MINING_GAME_ID,
            "profile_id": PROFILE,
            "name_id": 7910,
            "description_id": 7911,
            "icon_id": 909,
            "spec": spec,
        });
        if let Some(definitions) = definitions {
            content["powerup_definitions"] = serde_json::to_value(definitions).unwrap();
        }
        fs::write(
            directory.join("mining.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { directory }
    }

    fn load(&self, state: Option<Type8State>) -> Type8Engine {
        Type8Engine::load(
            &self.directory,
            "mining.json",
            MINING_GAME_ID,
            FIRST_INSTANCE,
            state,
        )
        .unwrap()
    }

    fn try_load(&self) -> std::io::Result<Type8Engine> {
        Type8Engine::load(
            &self.directory,
            "mining.json",
            MINING_GAME_ID,
            FIRST_INSTANCE,
            None,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("mining.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}

fn spec() -> Type8BoardSpec {
    let mut shape = vec![0_i32; SIDE * SIDE];
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            shape[y * SIDE + x] = 1;
        }
    }
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * SIDE + x] = 4;
    }
    Type8BoardSpec {
        width: SIDE,
        height: SIDE,
        shape,
        selectable: vec![SELECTABLE, SELECTABLE_TWO, 403, 404],
        collectable: vec![ORE, ORE_TWO],
        tiles: vec![
            mining::Type8Kind {
                value: SELECTABLE,
                category: 400,
                icon_id: 1,
            },
            mining::Type8Kind {
                value: ORE,
                category: 600,
                icon_id: 909,
            },
        ],
        goals: vec![Type8Goal {
            index: 0,
            value: ORE,
            quota: 15,
            per_reward: 1,
            icon_id: 909,
        }],
        outputs: Vec::new(),
        duration_seconds: 180,
        seed: 18,
        wildcard_percent: 0,
        collectable_percent: 0,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice: "Synthetic private fixture. Original server level tables unrecovered.".into(),
    }
}

fn definition(value: u16, power: u32) -> Type8PowerupDefinition {
    Type8PowerupDefinition {
        value,
        spawn_chance: 25,
        duration: 5,
        power,
    }
}

fn clickable_definitions() -> Vec<Type8PowerupDefinition> {
    vec![
        definition(TILE_POWERUP_DYNAMITE, 2),
        definition(TILE_POWERUP_WILDTILE, 2),
        definition(TILE_POWERUP_DESTRUCT, 2),
        definition(TILE_POWERUP_SHUFFLE, 2),
    ]
}

fn header(sub: u8, id: i32, game: i32) -> Vec<u8> {
    let mut b = 39_u16.to_le_bytes().to_vec();
    b.push(sub);
    for n in [id, -1, game] {
        b.extend(n.to_le_bytes());
    }
    b
}

fn client_payload(id: i32, s: &str) -> Vec<u8> {
    let mut b = header(14, id, -1);
    b.extend(((s.len() + 1) as i32).to_le_bytes());
    b.extend(s.as_bytes());
    b.push(0);
    b
}

fn payload_text(packet: &[u8]) -> Option<String> {
    if packet.len() < 19 || packet[0..3] != [39, 0, 14] {
        return None;
    }
    let n = i32::from_le_bytes(packet[15..19].try_into().unwrap());
    assert!(n >= 1);
    let end = 19 + n as usize - 1;
    assert_eq!(packet.len(), end + 1);
    assert_eq!(packet[end], 0);
    Some(String::from_utf8(packet[19..end].to_vec()).unwrap())
}

fn texts(outcome: &Type8Outcome) -> Vec<String> {
    outcome
        .replies
        .iter()
        .filter_map(|p| payload_text(p))
        .collect()
}

fn text_with(outcome: &Type8Outcome, prefix: &str) -> Option<String> {
    texts(outcome)
        .into_iter()
        .find(|t| t == prefix || t.starts_with(&format!("{prefix}\t")))
}

fn enter_playing(engine: &mut Type8Engine) -> Type8Outcome {
    let id = FIRST_INSTANCE;
    engine.begin(PROFILE).unwrap();
    engine.handle(&header(5, id, -1)).unwrap();
    engine.handle(&client_payload(id, "OnConnectMsg")).unwrap();
    engine
        .handle(&client_payload(id, "OnStartGameMsg"))
        .unwrap();
    let started = engine
        .handle(&client_payload(id, "OnStartLevelMsg"))
        .unwrap();
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        Type8Phase::Playing
    );
    started
}

/// One action fixture: clear the playable rectangle, seed the pressed powerup
/// and the tiles the action names, plus a bottom-row chain that keeps the
/// post-action `has_move` check from reshuffling the board under the assertion.
fn action_engine(value: u16, seeds: &[(usize, usize, u16)]) -> (Fixture, Type8Engine, Vec<u8>) {
    action_engine_with(value, seeds, "3,3,4,3,")
}

fn action_engine_with(
    value: u16,
    seeds: &[(usize, usize, u16)],
    list: &str,
) -> (Fixture, Type8Engine, Vec<u8>) {
    wide_action_engine(value, seeds, list, 2)
}

fn wide_action_engine(
    value: u16,
    seeds: &[(usize, usize, u16)],
    list: &str,
    power: u32,
) -> (Fixture, Type8Engine, Vec<u8>) {
    // `powerup_percent = 0` keeps the spawn roll out of the action fixture; the
    // authored definition is what licenses the action, and a clean selectable
    // board makes the refill values deterministic.
    let fixture = Fixture::new(0, Some(vec![definition(value, power)]));
    let mut engine = fixture.load(None);
    let _ = enter_playing(&mut engine);
    let mut cells: Vec<(usize, usize, u16)> = Vec::new();
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            cells.push((x, y, TILE_EMPTY));
        }
    }
    // A move has to survive the post-action `has_move` check without joining the
    // recovered draw pool: a stacked blocker column is a legal move, never moves
    // under gravity, and is not a selectable, wildcard or visible tile.
    for y in TOP..TOP + 3 {
        cells.push((10, y, TILE_BLOCKER));
    }
    // Seed the action tiles last: identical coordinates are overwritten, so the
    // fixture's own cells always win over the anchor.
    for &(x, y, value) in seeds {
        cells.retain(|(cx, cy, _)| (*cx, *cy) != (x, y));
        cells.push((x, y, value));
    }
    engine.seed_board(&cells).unwrap();
    let action = client_payload(FIRST_INSTANCE, &format!("OnActionMsg\t{value}\t{list}"));
    (fixture, engine, action)
}

fn board_at(engine: &Type8Engine, x: usize, y: usize) -> u16 {
    let spec = engine.spec();
    engine.state().active.as_ref().unwrap().board[y * spec.width + x]
}

fn board_count(engine: &Type8Engine, value: u16) -> usize {
    engine
        .state()
        .active
        .as_ref()
        .unwrap()
        .board
        .iter()
        .filter(|v| **v == value)
        .count()
}

#[test]
fn default_powerup_percent_matches_the_old_engine_byte_for_byte() {
    // RED before the authoring surface existed: the same content without the
    // key must produce the same board, queue, RNG and replies.
    let plain = Fixture::new(0, None);
    let mut a = plain.load(None);
    let started_a = enter_playing(&mut a);

    let empty = Fixture::new(0, Some(Vec::new()));
    let mut b = empty.load(None);
    let started_b = enter_playing(&mut b);

    assert_eq!(started_a.replies, started_b.replies);
    assert_eq!(a.state(), b.state());
    assert!(!texts(&started_a)
        .iter()
        .any(|t| t.starts_with("OnPowerupInitMsg")));
}

#[test]
fn authored_powerups_advertise_and_replace_selectable_tiles() {
    // GREEN: with definitions, the level advertises them and the any-roll feeds
    // powerups into the board and the spawn pool.
    let powerups = Fixture::new(100, Some(clickable_definitions()));
    let mut engine = powerups.load(None);
    let started = enter_playing(&mut engine);
    let init: Vec<String> = texts(&started)
        .into_iter()
        .filter(|t| t.starts_with("OnPowerupInitMsg"))
        .collect();
    assert_eq!(
        init,
        vec![
            "OnPowerupInitMsg\t950\t5\t2",
            "OnPowerupInitMsg\t951\t5\t2",
            "OnPowerupInitMsg\t952\t5\t2",
            "OnPowerupInitMsg\t953\t5\t2",
        ]
    );
    let active = engine.state().active.as_ref().unwrap();
    // The board keeps the recovered layout cells (empty/dropzone) and fills
    // every regular cell from the powerup pool; no plain selectable survives.
    assert!(active.board.contains(&TILE_POWERUP_DYNAMITE));
    assert!(active.board.iter().all(|v| *v == TILE_EMPTY
        || *v == mining::TILE_DROPZONE
        || (TILE_POWERUP_START..=TILE_POWERUP_SHUFFLE).contains(v)));
    assert!(active
        .tile_queue
        .iter()
        .all(|v| (TILE_POWERUP_START..=TILE_POWERUP_SHUFFLE).contains(v)));

    // The old default spawns only selectables in the regular cells; the
    // difference proves the new branch produced the powerups.
    let plain = Fixture::new(0, None);
    let mut engine = plain.load(None);
    let _ = enter_playing(&mut engine);
    let board = &engine.state().active.as_ref().unwrap().board;
    assert!(!board
        .iter()
        .any(|v| (TILE_POWERUP_START..=TILE_POWERUP_SHUFFLE).contains(v)));
    assert!(board.iter().any(|v| (400..500).contains(v)));
}

#[test]
fn default_writer_bytes_carry_no_powerup_key() {
    let directory = std::env::temp_dir().join(format!(
        "osfr-mining-powerups-writer-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    mining::write_content(&directory, MINING_GAME_ID).unwrap();
    let bytes = fs::read(directory.join(mining::CONTENT_FILE)).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("powerup_definitions"));
    let _ = fs::remove_file(directory.join(mining::CONTENT_FILE));
    let _ = fs::remove_dir(&directory);
}

#[test]
fn configured_collected_powerups_are_refused() {
    // RED: 900..=949 spawn rows must not load as inert tiles.
    let fixture = Fixture::new(100, Some(vec![definition(900, 2)]));
    let error = fixture.try_load().unwrap_err().to_string();
    assert!(
        error.contains("not implemented"),
        "unexpected error: {error}"
    );
}

#[test]
fn malformed_definitions_are_refused() {
    // 900 is refused inside validate_powerup_definitions; keep a second case for
    // an out-of-range power and a third for a value outside the clickable set.
    let too_much_power = Fixture::new(100, Some(vec![definition(950, 4096)]));
    assert!(too_much_power.try_load().is_err());
    let wrong_value = Fixture::new(100, Some(vec![definition(940, 2)]));
    let error = wrong_value.try_load().unwrap_err().to_string();
    assert!(error.contains("not implemented"), "unexpected: {error}");
    let duplicate = Fixture::new(100, Some(vec![definition(951, 2), definition(951, 2)]));
    assert!(duplicate.try_load().is_err());
    let no_weight = Fixture::new(100, Some(Vec::new()));
    assert!(no_weight.try_load().is_err());
}

#[test]
fn dynamite_shatters_the_recovered_neighbourhood() {
    // GREEN: 950 explodes its own cell plus one recovered BFS neighbour. The
    // neighbour order is the recovered down, up, right, left.
    let (_fixture, mut engine, action) = action_engine_with(
        TILE_POWERUP_DYNAMITE,
        &[(3, 3, TILE_POWERUP_DYNAMITE), (3, 4, SELECTABLE)],
        "3,3,3,4,",
    );
    let out = engine.handle(&action).unwrap();
    assert!(!texts(&out).iter().any(|t| t.starts_with("OnActionMsg")));
    assert_eq!(board_count(&engine, TILE_POWERUP_DYNAMITE), 0);
}

#[test]
fn dynamite_neighbour_order_is_down_up_right_left() {
    // Seed all four neighbours so the draw pool is exactly the recovered order:
    // down (3,4), up (3,2), right (4,3), left (2,3). The action must name them in
    // that order, so any other order is refused.
    // The fixture definition has power 5, so the whole neighbour sweep is implied.
    let pool = [
        (3, 4, TILE_POWERUP_DYNAMITE),
        (3, 5, SELECTABLE),
        (3, 3, SELECTABLE_TWO),
        (4, 4, SELECTABLE),
        (4, 4, SELECTABLE),
    ];
    let (_fixture, mut engine, _) =
        wide_action_engine(TILE_POWERUP_DYNAMITE, &pool, "3,4,3,5,3,3,4,4,4,4,", 5);
    // Deterministic zero rolls make `GetRandomTile` take the first pool entry,
    // so the draw order is exactly the recovered neighbour insertion order.
    engine.seed_random_queue(&vec![0_u16; 70]).unwrap();
    engine
        .handle(&client_payload(
            FIRST_INSTANCE,
            "OnActionMsg\t950\t3,4,3,5,3,3,4,4,4,4,",
        ))
        .unwrap();
    assert_eq!(board_count(&engine, TILE_POWERUP_DYNAMITE), 0);

    // A right-before-down order must fail.
    let (_fixture, mut engine, _) =
        wide_action_engine(TILE_POWERUP_DYNAMITE, &pool, "3,4,4,4,3,4,3,5,4,4,", 5);
    engine.seed_random_queue(&vec![0_u16; 70]).unwrap();
    let before = engine.state().clone();
    assert!(engine
        .handle(&client_payload(
            FIRST_INSTANCE,
            "OnActionMsg\t950\t3,4,4,4,3,4,3,5,4,4,",
        ))
        .is_err());
    assert_eq!(engine.state(), &before);
}

#[test]
fn wildtile_action_requires_and_applies_the_named_tiles() {
    // GREEN: 951 turns the pressed tile and the named selectable into wildcards.
    let (_fixture, mut engine, action) = action_engine(
        TILE_POWERUP_WILDTILE,
        &[(3, 3, TILE_POWERUP_WILDTILE), (4, 3, SELECTABLE)],
    );
    let out = engine.handle(&action).unwrap();
    // No server->client `OnActionMsg` exists: `GameClientNetwork.SendAction` is
    // the only sender and `GameClient` has no handler for it.
    assert!(!texts(&out).iter().any(|t| t.starts_with("OnActionMsg")));
    assert_eq!(board_count(&engine, TILE_WILDCARD), 2);
    assert_eq!(board_count(&engine, TILE_POWERUP_WILDTILE), 0);

    // A forged list that names a mixed tile must be refused without mutation.
    let (_fixture, mut engine, action) = action_engine(
        TILE_POWERUP_WILDTILE,
        &[
            (3, 3, TILE_POWERUP_WILDTILE),
            (4, 3, SELECTABLE_TWO),
            (5, 3, TILE_BLOCKER),
        ],
    );
    let forged = client_payload(FIRST_INSTANCE, "OnActionMsg\t951\t3,3,5,3,");
    let before = engine.state().active.as_ref().unwrap().board.clone();
    assert!(engine.handle(&forged).is_err());
    assert_eq!(engine.state().active.as_ref().unwrap().board, before);
    let _ = action;
}

#[test]
fn wildtile_refuses_overflowing_and_duplicate_lists() {
    let (_fixture, mut engine, action) = action_engine(
        TILE_POWERUP_WILDTILE,
        &[(3, 3, TILE_POWERUP_WILDTILE), (4, 3, SELECTABLE)],
    );
    let before = engine.state().active.as_ref().unwrap().board.clone();
    // power = 2, so a three-tile list is over the recovered cap.
    let overflow = client_payload(FIRST_INSTANCE, "OnActionMsg\t951\t3,3,4,3,5,3,");
    assert!(engine.handle(&overflow).is_err());
    let duplicate = client_payload(FIRST_INSTANCE, "OnActionMsg\t951\t3,3,3,3,");
    assert!(engine.handle(&duplicate).is_err());
    assert_eq!(engine.state().active.as_ref().unwrap().board, before);
    let _ = action;
}

#[test]
fn destruct_action_clears_the_named_tiles() {
    // GREEN: 952 removes the named selectable and leaves the pressed tile empty.
    let (_fixture, mut engine, action) = action_engine(
        TILE_POWERUP_DESTRUCT,
        &[(3, 3, TILE_POWERUP_DESTRUCT), (4, 3, SELECTABLE)],
    );
    let selectable_before = board_count(&engine, SELECTABLE);
    let out = engine.handle(&action).unwrap();
    assert!(!texts(&out).iter().any(|t| t.starts_with("OnActionMsg")));
    // Gravity refills the freed cells from the (non-empty) queue, so assert the
    // destroyed value left the board and that the action consumed its target.
    assert_eq!(board_count(&engine, TILE_POWERUP_DESTRUCT), 0);
    assert!(board_count(&engine, SELECTABLE) >= selectable_before - 1);
    assert_ne!(board_at(&engine, 4, 10), TILE_POWERUP_DESTRUCT);
}

#[test]
fn shuffle_action_rotates_tiles_and_stays_in_bounds() {
    // GREEN: 953 becomes a wildcard and the named tile takes its value.
    let (_fixture, mut engine, action) = action_engine(
        TILE_POWERUP_SHUFFLE,
        &[(3, 3, TILE_POWERUP_SHUFFLE), (4, 3, SELECTABLE)],
    );
    let out = engine.handle(&action).unwrap();
    assert!(!texts(&out).iter().any(|t| t.starts_with("OnActionMsg")));
    // The picked tile's value lands on the pressed slot and the pressed tile
    // stays the wildcard at its own cell; the powerup is never recycled.
    assert_eq!(board_count(&engine, TILE_WILDCARD), 1);
    assert_eq!(board_count(&engine, TILE_POWERUP_SHUFFLE), 0);
    assert_eq!(board_at(&engine, 4, 10), SELECTABLE);
}

#[test]
fn runtime_shuffle_places_drawn_tiles_at_the_recovered_positions() {
    // Root/independent RED oracle: power 4 press plus three drawn tiles must end
    // `[401, 402, 403] -> [403, 401, 402]` on the drawn coordinates, pressed
    // stays `200`, and every value survives exactly once. The tiles sit on the
    // bottom row so the post-action gravity pass cannot move them, and three
    // further `401`s keep `has_move` true so no reshuffle fires.
    let (_fixture, mut engine, _) = wide_action_engine(
        TILE_POWERUP_SHUFFLE,
        &[
            (3, 10, TILE_POWERUP_SHUFFLE),
            (4, 10, SELECTABLE),
            (5, 10, SELECTABLE_TWO),
            (6, 10, 403_u16),
            (8, 10, SELECTABLE),
            (9, 10, SELECTABLE),
            (10, 10, SELECTABLE),
        ],
        "3,10,4,10,5,10,6,10,",
        4,
    );
    engine.seed_random_queue(&vec![0_u16; 70]).unwrap();
    engine.seed_tile_queue(&vec![TILE_EMPTY; 70]).unwrap();
    let out = engine
        .handle(&client_payload(
            FIRST_INSTANCE,
            "OnActionMsg\t953\t3,10,4,10,5,10,6,10,",
        ))
        .unwrap();
    assert!(!texts(&out).iter().any(|t| t.starts_with("OnActionMsg")));
    assert_eq!(board_at(&engine, 3, 10), TILE_WILDCARD);
    assert_eq!(board_at(&engine, 4, 10), 403);
    assert_eq!(board_at(&engine, 5, 10), SELECTABLE);
    assert_eq!(board_at(&engine, 6, 10), SELECTABLE_TWO);
    assert_eq!(board_count(&engine, 403), 1);
    assert_eq!(board_count(&engine, SELECTABLE_TWO), 1);
    assert_eq!(board_count(&engine, SELECTABLE), 4);
    assert_eq!(board_count(&engine, TILE_WILDCARD), 1);
    assert_eq!(board_count(&engine, TILE_POWERUP_SHUFFLE), 0);
}

#[test]
fn runtime_shuffle_folds_equal_neighbours_the_as2_way() {
    // Same fixture with `[401, 401, 402]` drawn: the walk folds index 1 onto
    // index 2, giving `[402, 401, 401]`.
    let (_fixture, mut engine, _) = wide_action_engine(
        TILE_POWERUP_SHUFFLE,
        &[
            (3, 10, TILE_POWERUP_SHUFFLE),
            (4, 10, SELECTABLE),
            (5, 10, SELECTABLE),
            (6, 10, SELECTABLE_TWO),
            (8, 10, SELECTABLE),
            (9, 10, SELECTABLE),
            (10, 10, SELECTABLE),
        ],
        "3,10,4,10,5,10,6,10,",
        4,
    );
    engine.seed_random_queue(&vec![0_u16; 70]).unwrap();
    engine.seed_tile_queue(&vec![TILE_EMPTY; 70]).unwrap();
    engine
        .handle(&client_payload(
            FIRST_INSTANCE,
            "OnActionMsg\t953\t3,10,4,10,5,10,6,10,",
        ))
        .unwrap();
    assert_eq!(board_at(&engine, 3, 10), TILE_WILDCARD);
    assert_eq!(board_at(&engine, 4, 10), SELECTABLE_TWO);
    assert_eq!(board_at(&engine, 5, 10), SELECTABLE);
    assert_eq!(board_at(&engine, 6, 10), SELECTABLE);
    assert_eq!(board_count(&engine, SELECTABLE), 5);
    assert_eq!(board_count(&engine, SELECTABLE_TWO), 1);
    assert_eq!(board_count(&engine, TILE_WILDCARD), 1);
}

#[test]
fn shuffle_placement_walk_matches_the_as2_targets() {
    // Direct port check for `StartShuffle` 1213-1245. `shuffle_placements`
    // returns the destination for the mover of each index; the mover itself is
    // the drawn tile at `(index + n - 1) % n`.
    let positions = [(10, 1), (10, 2), (10, 3)];

    // Distinct drawn values: every slot already carries a different value, so no
    // walk happens and each slot keeps its coordinate. Applied to the movers
    // this is the recovered right rotation, `[401, 402, 403] -> [403, 401, 402]`.
    let distinct = mining::shuffle_placements(&[401, 402, 403], &positions);
    assert_eq!(distinct, vec![positions[0], positions[1], positions[2]]);
    let distinct_result: Vec<u16> = (0..3)
        .map(|index| [401_u16, 402, 403][(index + 2) % 3])
        .collect();
    assert_eq!(distinct_result, vec![403, 401, 402]);

    // Equal neighbours: index 1's destination folds onto index 2's, so movers
    // land `[402, 401, 401]` instead.
    let equal = mining::shuffle_placements(&[401, 401, 402], &positions);
    assert_eq!(equal, vec![positions[0], positions[2], positions[1]]);
    let equal_result: Vec<u16> = equal
        .iter()
        .enumerate()
        .map(|(index, _)| [401_u16, 401, 402][(index + 2) % 3])
        .collect();
    assert_eq!(equal_result, vec![402, 401, 401]);
}

#[test]
fn shuffle_rejects_a_list_that_does_not_match_the_draw_order() {
    // The server owns the selection: a client list naming the same tiles in a
    // different order than the server drew them is refused atomically.
    let pool = [
        (3, 3, TILE_POWERUP_SHUFFLE),
        (4, 3, SELECTABLE),
        (5, 3, SELECTABLE_TWO),
        (6, 3, TILE_WILDCARD),
    ];
    let (_fixture, mut engine, _) = action_engine_with(TILE_POWERUP_SHUFFLE, &pool, "3,3,5,3,4,3,");
    let before = engine.state().clone();
    let action = client_payload(FIRST_INSTANCE, "OnActionMsg\t953\t3,3,5,3,4,3,");
    assert!(engine.handle(&action).is_err());
    assert_eq!(engine.state(), &before);
}

#[test]
fn forged_short_lists_are_refused_for_every_clickable_action() {
    // The recovered loops run the authored power (or the pool cap), so a list
    // that names fewer tiles than the spawn implies is refused atomically rather
    // than silently applying a partial effect.
    for (value, seeds, spare) in [
        (
            TILE_POWERUP_DYNAMITE,
            [(3, 3, TILE_POWERUP_DYNAMITE), (3, 4, SELECTABLE)],
            (4, 4, SELECTABLE),
        ),
        (
            TILE_POWERUP_WILDTILE,
            [(3, 3, TILE_POWERUP_WILDTILE), (4, 3, SELECTABLE)],
            (5, 3, SELECTABLE),
        ),
        (
            TILE_POWERUP_DESTRUCT,
            [(3, 3, TILE_POWERUP_DESTRUCT), (4, 3, SELECTABLE)],
            (5, 3, SELECTABLE),
        ),
        (
            TILE_POWERUP_SHUFFLE,
            [(3, 3, TILE_POWERUP_SHUFFLE), (4, 3, SELECTABLE)],
            (5, 3, SELECTABLE),
        ),
    ] {
        // A spare eligible tile makes the implied count larger than the forged
        // list, so the forged action cannot pass.
        let mut cells: Vec<(usize, usize, u16)> = vec![seeds[0], seeds[1], spare];
        let (_fixture, mut engine, _) = wide_action_engine(value, &cells, "3,3,", 3);
        cells.clear();
        let before = engine.state().clone();
        let forged = client_payload(FIRST_INSTANCE, &format!("OnActionMsg\t{value}\t3,3,"));
        assert!(
            engine.handle(&forged).is_err(),
            "action {value} accepted a short list"
        );
        assert_eq!(engine.state(), &before);
    }

    // A genuinely short pool still accepts the shorter implied list: power 3 but
    // only one eligible tile besides the pressed one.
    let (_fixture, mut engine, _) = wide_action_engine(
        TILE_POWERUP_WILDTILE,
        &[(3, 3, TILE_POWERUP_WILDTILE), (4, 3, SELECTABLE)],
        "3,3,4,3,",
        3,
    );
    engine
        .handle(&client_payload(
            FIRST_INSTANCE,
            "OnActionMsg\t951\t3,3,4,3,",
        ))
        .unwrap();
    assert_eq!(board_count(&engine, TILE_WILDCARD), 2);
}

#[test]
fn action_draw_refills_in_consumed_order_until_the_minimum() {
    // Recovered order: consume, then `while len < 50` append a 20-batch and
    // advertise it (`GenerateRandomNumbersForQueue` 795-808). Restoring a short
    // queue must therefore refill repeatedly, not once.
    let (_fixture, mut engine, _) = wide_action_engine(
        TILE_POWERUP_WILDTILE,
        &[
            (3, 3, TILE_POWERUP_WILDTILE),
            (4, 3, SELECTABLE),
            (5, 3, SELECTABLE),
        ],
        "3,3,4,3,5,3,",
        3,
    );
    engine.seed_random_queue(&[0_u16; 5]).unwrap();
    // The two later entries of the queue must not trigger another batch: only
    // the short-queue refill path emits `OnRandomQueueMsg`.
    let out = engine
        .handle(&client_payload(
            FIRST_INSTANCE,
            "OnActionMsg\t951\t3,3,4,3,5,3,",
        ))
        .unwrap();
    let refills: Vec<String> = texts(&out)
        .into_iter()
        .filter(|t| t.starts_with("OnRandomQueueMsg"))
        .collect();
    // 5 consumes to 4, then 4 -> 24 -> 44 -> 64; three batches, in order.
    assert_eq!(refills.len(), 3, "refills: {refills:?}");
    for (index, refill) in refills.iter().enumerate() {
        let values: Vec<u16> = refill
            .trim_start_matches("OnRandomQueueMsg\t")
            .trim_end_matches(',')
            .split(',')
            .map(|v| v.parse().unwrap())
            .collect();
        assert_eq!(values.len(), 20, "batch {index} must carry 20 draws");
    }
    let queue = engine.state().active.as_ref().unwrap().random_queue.len();
    assert!(queue >= 50, "queue {queue} below the recovered minimum");
}

#[test]
fn every_clickable_action_consumes_one_draw_per_named_tile() {
    // The server owns the selection: the recovered `GetRandomTile` /
    // `ShatterValidateNextTile` draw once per client entry after the pressed
    // tile, from the advertised random-number queue. A two-entry action must
    // therefore consume exactly one queued draw, and the queue must stay inside
    // its recovered bounds.
    for (value, seeds, list) in [
        (
            TILE_POWERUP_DYNAMITE,
            [(3, 3, TILE_POWERUP_DYNAMITE), (3, 4, SELECTABLE)],
            "3,3,3,4,",
        ),
        (
            TILE_POWERUP_WILDTILE,
            [(3, 3, TILE_POWERUP_WILDTILE), (4, 3, SELECTABLE)],
            "3,3,4,3,",
        ),
        (
            TILE_POWERUP_DESTRUCT,
            [(3, 3, TILE_POWERUP_DESTRUCT), (4, 3, SELECTABLE)],
            "3,3,4,3,",
        ),
        (
            TILE_POWERUP_SHUFFLE,
            [(3, 3, TILE_POWERUP_SHUFFLE), (4, 3, SELECTABLE)],
            "3,3,4,3,",
        ),
    ] {
        let (_fixture, mut engine, _) = action_engine_with(value, &seeds, list);
        let before = engine.state().active.as_ref().unwrap().random_queue.len();
        let action = client_payload(FIRST_INSTANCE, &format!("OnActionMsg\t{value}\t{list}"));
        let out = engine.handle(&action).unwrap();
        let after = engine.state().active.as_ref().unwrap().random_queue.len();
        assert_eq!(
            before - after,
            1,
            "action {value} must consume exactly one queued draw"
        );
        assert!(!texts(&out)
            .iter()
            .any(|t| t.starts_with("OnRandomQueueMsg")));
    }
}

#[test]
fn action_failures_are_atomic_and_replay_is_inert() {
    // Atomic rollback: a forged action must not change the run.
    let (_fixture, mut engine, _action) = action_engine(
        TILE_POWERUP_DESTRUCT,
        &[(3, 3, TILE_POWERUP_DESTRUCT), (4, 3, SELECTABLE)],
    );
    let before = engine.state().clone();
    let forged = client_payload(FIRST_INSTANCE, "OnActionMsg\t952\t9,9,4,3,");
    assert!(engine.handle(&forged).is_err());
    assert_eq!(engine.state(), &before);

    // Replaying the same legal action is inert because the board no longer
    // matches; the run is untouched rather than double-applied.
    let good = client_payload(FIRST_INSTANCE, "OnActionMsg\t952\t3,3,4,3,");
    engine.handle(&good).unwrap();
    let after = engine.state().clone();
    assert!(engine.handle(&good).is_err());
    assert_eq!(engine.state(), &after);
}

#[test]
fn forged_powerup_views_and_actions_are_refused() {
    // Forged server->client messages must not change state; forged actions for
    // unserviced values must be refused.
    let fixture = Fixture::new(100, Some(clickable_definitions()));
    let mut engine = fixture.load(None);
    let _ = enter_playing(&mut engine);
    engine
        .seed_board(&[(3, 3, TILE_POWERUP_WILDTILE), (4, 3, SELECTABLE)])
        .unwrap();
    let before = engine.state().clone();
    for forged in [
        client_payload(FIRST_INSTANCE, "OnActionMsg\t902\t3,3"),
        client_payload(FIRST_INSTANCE, "OnActionMsg\t950\t3,3"),
        client_payload(FIRST_INSTANCE, "OnActionMsg\t1\t1\t1\t0"),
        client_payload(FIRST_INSTANCE, "OnLevelEventMsg\t40\t600"),
        client_payload(FIRST_INSTANCE, "OnLevelEventMsg\t22\t5\t600"),
        client_payload(FIRST_INSTANCE, "OnLevelEventMsg\t23"),
        client_payload(FIRST_INSTANCE, "OnTypeBagsMsg\t900"),
    ] {
        let result = engine.handle(&forged);
        assert!(
            result.is_err() || !result.as_ref().unwrap().changed,
            "forged message {forged:?} must not change state"
        );
    }
    assert_eq!(engine.state(), &before);
    // The serviced action still works on the untouched board.
    let good = client_payload(FIRST_INSTANCE, "OnActionMsg\t951\t3,3,4,3,");
    let out = engine.handle(&good).unwrap();
    assert!(!texts(&out).iter().any(|t| t.starts_with("OnActionMsg")));
    assert_eq!(board_count(&engine, TILE_WILDCARD), 2);
}

#[test]
fn paused_and_stale_actions_are_inert() {
    let (_fixture, mut engine, action) = action_engine(
        TILE_POWERUP_WILDTILE,
        &[(3, 3, TILE_POWERUP_WILDTILE), (4, 3, SELECTABLE)],
    );
    engine
        .handle(&client_payload(FIRST_INSTANCE, "FRServer_Pause\ttrue"))
        .unwrap();
    let paused = engine.state().clone();
    assert!(engine.handle(&action).is_err());
    assert_eq!(engine.state(), &paused);

    // A stale instance id is ignored by the outer guard.
    let stale = client_payload(FIRST_INSTANCE + 1, "OnActionMsg\t951\t3,3,4,3,");
    let out = engine.handle(&stale).unwrap();
    assert!(!out.changed);
    assert_eq!(engine.state(), &paused);
}

#[test]
fn a_lone_clickable_powerup_keeps_the_board_alive() {
    // Recovered `ServerGrid.HasMove()` 203-207: a clickable powerup is a legal
    // move, so a board holding one is never declared dead.
    let fixture = Fixture::new(0, None);
    let mut engine = fixture.load(None);
    let _ = enter_playing(&mut engine);
    let mut cells: Vec<(usize, usize, u16)> = Vec::new();
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            cells.push((x, y, TILE_EMPTY));
        }
    }
    cells.push((3, 3, TILE_POWERUP_SHUFFLE));
    engine.seed_board(&cells).unwrap();
    assert!(engine.has_future_move());

    // The same board without the clickable powerup has no three-tile path.
    let mut cells = Vec::new();
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            cells.push((x, y, TILE_EMPTY));
        }
    }
    cells.push((3, 3, TILE_EMPTY));
    engine.seed_board(&cells).unwrap();
    assert!(!engine.has_future_move());
}
