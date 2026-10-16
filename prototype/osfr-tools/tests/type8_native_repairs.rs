//! Type8 native repair round: Harvesting produce goals, names and icons.
//!
//! Native run `client-baseline-20261009T202011Z` loaded the original
//! `game_farming.gfx` board with four baskets reading unresolved
//! `##farmingValue60…` text and blank produce icons. The recovered original
//! contract is:
//!
//! 1. `GameClient.as::OnQuotaInitMsg` names a bin with
//!    `LookupString(GameData.sGAME + "Value" + value)`, and the shipped client
//!    `Resources/CodeStringMappings.txt` only maps `farmingValue401..416`
//!    (there is no `farmingValue` in the 600 block), so a farming basket value
//!    must be `400 + produce type`.
//! 2. `core/TileGenerator.as::SetTileInfo(value, index)` builds a basket icon
//!    from the same value through the tile generator, so the icon frame is
//!    `SelectableTile<type>`; `game_farming.gfx` carries
//!    `SelectableTile1..16`.
//! 3. `GameLevel.QuotasInit` sends quotas for every spawn entry whose quota is
//!    non-zero — selectables first — and `HandleSelectableTile` counts every
//!    collected chain tile through `OnQuotaUpdateMsg`, exactly like
//!    `HandleCollectableTile` does for the mining 600 block.
//!
//! The fixture exercises the registered crate modules; all runtime inputs are
//! synthetic and private.

use harvesting::{produce, PRODUCE, REGIONS};
use mining::{Type8Engine, Type8Outcome, Type8Phase};
use osfr_tools::{harvesting, mining};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const GAME_ID: i32 = 111;
const PROFILE: i32 = harvesting::CHEF_PROFILE_ID;
const FIRST_INSTANCE: i32 = harvesting::FIRST_INSTANCE_ID;
const SELECTABLE: u16 = harvesting::TILE_SELECTABLE_CATEGORY;

/// Recovered original produce contract, in order: `(value, client locale id,
/// name, movie frame)`. Sources: `Resources/CodeStringMappings.txt`
/// (`farmingValue401..416` -> locale id) resolved against the stock
/// `locale/en_us_data.dat` through the pinned community reader, the bundle
/// names at locale 23285..23300, and the `SelectableTile1..16` frame labels
/// recovered from `game_farming.gfx` (JPEXS 26.3.0 export).
const RECOVERED_PRODUCE: [(u16, i32, &str); 16] = [
    (401, 117115, "Gummyfruit"),
    (402, 69328, "Snowberry"),
    (403, 5398, "Spiralmint"),
    (404, 117118, "Moonbean"),
    (405, 117119, "Choychoy"),
    (406, 69333, "Sunleaf"),
    (407, 117121, "Honeytwig"),
    (408, 117122, "Punkin"),
    (409, 69334, "Briarmelon"),
    (410, 69332, "Rainleaf"),
    (411, 117125, "Stickfruit"),
    (412, 117126, "Starnut"),
    (413, 69329, "Stainberry"),
    (414, 69330, "Chugaroot"),
    (415, 69327, "Bumbleberry"),
    (416, 117130, "Tumtumnip"),
];

const SIDE: usize = 14;
const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new(name: &str, content: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-type8-repair-{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("harvesting.json"), content).unwrap();
        Self { directory }
    }

    fn load(&self) -> std::io::Result<Type8Engine> {
        Type8Engine::load(
            &self.directory,
            "harvesting.json",
            GAME_ID,
            FIRST_INSTANCE,
            None,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("harvesting.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}

fn header(sub: u8, id: i32, game: i32) -> Vec<u8> {
    let mut b = 39_u16.to_le_bytes().to_vec();
    b.push(sub);
    for n in [id, -1, game] {
        b.extend(n.to_le_bytes());
    }
    b
}

/// Client `39/14` payload: current instance, two `-1` sentinels, length + text + NUL.
fn client_payload(id: i32, s: &str) -> Vec<u8> {
    let mut b = header(14, id, -1);
    b.extend(((s.len() + 1) as i32).to_le_bytes());
    b.extend(s.as_bytes());
    b.push(0);
    b
}

fn texts(outcome: &Type8Outcome) -> Vec<String> {
    outcome
        .replies
        .iter()
        .filter_map(|packet| {
            if packet.len() < 19 || packet[0..3] != [39, 0, 14] {
                return None;
            }
            let n = i32::from_le_bytes(packet[15..19].try_into().unwrap());
            assert!(n >= 1);
            let end = 19 + n as usize - 1;
            assert_eq!(packet.len(), end + 1);
            assert_eq!(packet[end], 0);
            String::from_utf8(packet[19..end].to_vec()).ok()
        })
        .collect()
}

fn has_text(outcome: &Type8Outcome, text: &str) -> bool {
    texts(outcome).iter().any(|t| t == text)
}

fn enter_playing(engine: &mut Type8Engine) -> Vec<String> {
    let id = FIRST_INSTANCE;
    engine.begin(PROFILE).unwrap();
    engine.handle(&header(5, id, -1)).unwrap();
    engine.handle(&client_payload(id, "OnConnectMsg")).unwrap();
    engine
        .handle(&client_payload(id, "OnStartGameMsg"))
        .unwrap();
    let out = engine
        .handle(&client_payload(id, "OnStartLevelMsg"))
        .unwrap();
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        Type8Phase::Playing
    );
    texts(&out)
}

/// The recovered value for one produce slug: `400 + type`, with the produce
/// table order equal to the client's `farmingValue401..416` order.
fn recovered_value(slug: &str) -> u16 {
    let index = PRODUCE
        .iter()
        .position(|p| p.slug == slug)
        .unwrap_or_else(|| panic!("unknown produce {slug}"));
    401 + index as u16
}

fn farming_spec(goal_value: u16, quota: u32, rows: [[i32; PLAYABLE]; PLAYABLE]) -> String {
    let mut shape = vec![0_i32; SIDE * SIDE];
    for (dy, row) in rows.iter().enumerate() {
        for (dx, value) in row.iter().enumerate() {
            shape[(TOP + dy) * SIDE + LEFT + dx] = *value;
        }
    }
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * SIDE + x] = 4;
    }
    let selectable: Vec<u16> = vec![411, 413, 416, 403];
    let tiles: Vec<serde_json::Value> = selectable
        .iter()
        .map(|value| json!({"value": value, "category": 400, "icon_id": value - 400}))
        .collect();
    json!({
        "schema_version": 1,
        "game_id": GAME_ID,
        "profile_id": PROFILE,
        "name_id": 100834,
        "description_id": 94150,
        "icon_id": harvesting::HARVESTING_ICON_ID,
        "preload_file": harvesting::FARMING_PRELOAD_FILE,
        "launch_movie": harvesting::FARMING_LAUNCH_MOVIE,
        "spec": {
            "width": SIDE,
            "height": SIDE,
            "shape": shape,
            "selectable": selectable,
            "collectable": Vec::<u16>::new(),
            "tiles": tiles,
            "goals": [{
                "index": 0,
                "value": goal_value,
                "quota": quota,
                "per_reward": 1,
                "icon_id": 706,
            }],
            "outputs": [],
            "duration_seconds": 180,
            "seed": 111,
            "wildcard_percent": 0,
            "collectable_percent": 0,
            "blocker_percent": 0,
            "powerup_percent": 0,
            "meter_id": 0,
            "reconstructed_rules": true,
            "rule_notice": "Synthetic private Type8 repair fixture. Original server level tables unrecovered.",
        }
    })
    .to_string()
}

fn default_rows() -> [[i32; PLAYABLE]; PLAYABLE] {
    // A full regular rectangle; individual tests overwrite the cells they own.
    [[1; PLAYABLE]; PLAYABLE]
}

#[test]
fn recovered_produce_table_matches_the_client_code_strings_and_movie_frames() {
    assert_eq!(PRODUCE.len(), RECOVERED_PRODUCE.len());
    for (index, (value, locale_id, name)) in RECOVERED_PRODUCE.iter().enumerate() {
        assert_eq!(
            *value,
            401 + index as u16,
            "farmingValue values are 400 + 1-based produce type"
        );
        assert_eq!(
            name.to_ascii_lowercase(),
            PRODUCE[index].slug,
            "the client `farmingValue{}` code string ({locale_id}) must name this produce",
            value
        );
        let p = &PRODUCE[index];
        assert_eq!(
            p.name_locale_id,
            23285 + index as i32,
            "bundle locale order must equal the recovered farmingValue order"
        );
        assert_eq!(
            p.item_definition_id,
            2875 + index as i32,
            "bundle item definitions follow the recovered produce order"
        );
        assert_eq!(
            *value - SELECTABLE,
            index as u16 + 1,
            "the movie frame is SelectableTile<type>, type = value - 400"
        );
    }
}

#[test]
fn harvesting_goals_use_the_recovered_produce_values_and_frames() {
    for r in REGIONS {
        let spec = harvesting::board_spec(r.game_id)
            .unwrap_or_else(|error| panic!("game {}: {error}", r.game_id));
        let expected: Vec<u16> = r.produce.iter().map(|slug| recovered_value(slug)).collect();
        assert_eq!(
            spec.selectable, expected,
            "game {} must spawn exactly its recovered produce values",
            r.game_id
        );
        assert!(
            spec.collectable.is_empty(),
            "game {} must not use the unnamed 600 block",
            r.game_id
        );
        assert_eq!(spec.collectable_percent, 0);
        let goal_values: Vec<u16> = spec.goals.iter().map(|goal| goal.value).collect();
        assert_eq!(
            goal_values, expected,
            "game {} baskets must name the recovered produce values",
            r.game_id
        );
        assert_eq!(
            spec.outputs.iter().map(|o| o.value).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(spec.tiles.len(), expected.len());
        for (slot, kind) in spec.tiles.iter().enumerate() {
            assert_eq!(kind.category, SELECTABLE);
            assert_eq!(
                kind.value - kind.category,
                (kind.value - 400) as u16,
                "the tile index is the movie frame number"
            );
            let frame = kind.value - kind.category;
            assert!(
                (1..=16).contains(&frame),
                "game {} goal {slot} frame {frame} must exist in SelectableTile1..16",
                r.game_id
            );
        }
        for (slot, slug) in r.produce.iter().enumerate() {
            let p = produce(slug).unwrap();
            assert_eq!(spec.goals[slot].icon_id, p.icon_id);
            assert_eq!(
                spec.outputs[slot].definition_id, p.item_definition_id,
                "game {} output {slot} must keep the recovered bundle definition",
                r.game_id
            );
        }
    }
}

#[test]
fn harvesting_content_round_trips_through_the_repaired_writer() {
    for r in REGIONS {
        let content = harvesting::content(r.game_id).unwrap();
        harvesting::validate_content(&content)
            .unwrap_or_else(|error| panic!("game {} content must validate: {error}", r.game_id));
        let json = harvesting::content_json(r.game_id).unwrap();
        let mirror: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(mirror["spec"]["collectable"], json!([]));
        assert_eq!(mirror["spec"]["collectable_percent"], json!(0));
    }
}

#[test]
fn harvesting_content_refuses_a_goal_outside_the_produce_namespace() {
    let mut content = harvesting::content(GAME_ID).unwrap();
    // The 601..604 baskets that produced the native `##farmingValue60…`
    // placeholders must never be emitted again.
    content.spec.goals[0].value = 601;
    let error = harvesting::validate_content(&content).unwrap_err();
    assert!(
        error.to_string().contains("produce"),
        "unexpected error: {error}"
    );
    // The unnamed 600 block must stay out of a farming package entirely.
    let mut content = harvesting::content(GAME_ID).unwrap();
    content.spec.collectable.push(601);
    content.spec.tiles.push(harvesting::Type8Kind {
        value: 601,
        category: harvesting::TILE_COLLECTABLE_CATEGORY,
        icon_id: 909,
    });
    let error = harvesting::validate_content(&content).unwrap_err();
    assert!(
        error.to_string().contains("600-block"),
        "unexpected error: {error}"
    );
}

#[test]
fn start_level_registers_produce_tiles_with_the_movie_frame_index() {
    let content = harvesting::content_json(GAME_ID).unwrap();
    let fixture = Fixture::new("frames", &content);
    let mut engine = fixture.load().unwrap();
    let replies = enter_playing(&mut engine);
    let tile_info: Vec<String> = replies
        .into_iter()
        .filter(|text| text.starts_with("OnTileInfoMsg"))
        .collect();
    assert_eq!(
        tile_info,
        vec![
            "OnTileInfoMsg\t411\t11",
            "OnTileInfoMsg\t413\t13",
            "OnTileInfoMsg\t416\t16",
            "OnTileInfoMsg\t403\t3",
        ],
        "the client resolves the second field as the SelectableTile frame number"
    );
}

#[test]
fn farming_chain_feeds_the_produce_goal_and_stops_the_level() {
    let mut rows = default_rows();
    rows[6][0] = -411;
    rows[6][1] = -411;
    rows[6][2] = -411;
    let fixture = Fixture::new("chain", &farming_spec(411, 3, rows));
    let mut engine = fixture.load().unwrap();
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let out = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnChainMsg\t{},{},{},{},{},{}",
                LEFT,
                TOP + 6,
                LEFT + 1,
                TOP + 6,
                LEFT + 2,
                TOP + 6
            ),
        ))
        .unwrap();
    let replies = texts(&out);
    assert_eq!(
        replies
            .iter()
            .filter(|text| text.starts_with("OnQuotaUpdateMsg"))
            .cloned()
            .collect::<Vec<_>>(),
        vec![
            "OnQuotaUpdateMsg\t0\t1\t3\t1",
            "OnQuotaUpdateMsg\t0\t2\t3\t1",
            "OnQuotaUpdateMsg\t0\t3\t3\t1",
        ],
        "every chained produce tile must count toward its basket"
    );
    assert!(
        replies.iter().any(|text| text == "OnStopLevel\t1\ttrue"),
        "the completed quota must stop the level: {replies:?}"
    );
    let run = engine.state().active.as_ref().unwrap();
    assert_eq!(run.collected, vec![3]);
    assert_eq!(run.phase, Type8Phase::LevelComplete);
    let out = engine
        .handle(&client_payload(id, "OnNextLevelMsg"))
        .unwrap();
    assert!(has_text(&out, "OnGameWonMsg"));
    assert!(engine.state().completed >= 1);
}

#[test]
fn mining_chain_still_does_not_feed_a_collectable_goal() {
    let mut rows = default_rows();
    rows[6][0] = -401;
    rows[6][1] = -401;
    rows[6][2] = -401;
    let mut spec: serde_json::Value = serde_json::from_str(&farming_spec(601, 1, rows)).unwrap();
    spec["spec"]["selectable"] = json!([401, 402]);
    spec["spec"]["collectable"] = json!([601]);
    spec["spec"]["tiles"] = json!([
        {"value": 401, "category": 400, "icon_id": 1},
        {"value": 601, "category": 600, "icon_id": 909},
    ]);
    spec["spec"]["goals"] = json!([{
        "index": 0,
        "value": 601,
        "quota": 1,
        "per_reward": 1,
        "icon_id": 909,
    }]);
    let fixture = Fixture::new("mining-regression", &spec.to_string());
    let mut engine = fixture.load().unwrap();
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let out = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnChainMsg\t{},{},{},{},{},{}",
                LEFT,
                TOP + 6,
                LEFT + 1,
                TOP + 6,
                LEFT + 2,
                TOP + 6
            ),
        ))
        .unwrap();
    assert!(
        !texts(&out)
            .iter()
            .any(|text| text.starts_with("OnQuotaUpdateMsg")),
        "mining collectable goals advance on falls, not on chains"
    );
    let run = engine.state().active.as_ref().unwrap();
    assert_eq!(run.collected, vec![0]);
    assert_eq!(run.phase, Type8Phase::Playing);
}

#[test]
fn engine_accepts_a_registered_selectable_goal_value() {
    // The value must be registered as a tile kind; a goal on an unregistered
    // selectable value stays refused instead of counting silently.
    let mut rows = default_rows();
    rows[6][0] = -411;
    rows[6][1] = -411;
    rows[6][2] = -411;
    let mut spec: serde_json::Value = serde_json::from_str(&farming_spec(411, 3, rows)).unwrap();
    spec["spec"]["goals"][0]["value"] = json!(412);
    let fixture = Fixture::new("unregistered-goal", &spec.to_string());
    let error = fixture.load().err().expect("unregistered goal must refuse");
    assert!(
        error.to_string().contains("goal"),
        "unexpected error: {error}"
    );
}
