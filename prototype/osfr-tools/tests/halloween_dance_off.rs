//! T464: the Halloween Vampires vs Werewolves dance-off, the shipped `halloween_dance_off` row of
//! `world_events.json`. Every id in it is gated against the client's own tables; the run is
//! driven through the real `WorldEvents::tick` for each ending; the schedule is swept over a day
//! in and a day out of season.
#[path = "support/models.rs"]
mod models;

use osfr_tools::{
    chat,
    client_areas::ClientAreas,
    players::{InterestPolicy, PlayerRegistry, SessionId, ZoneInstance},
    world_events::{self, WorldEvent, WorldEvents},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
};

const KEY: &str = "halloween_dance_off";
const WORLD: ZoneInstance = ZoneInstance {
    zone_id: 0,
    instance_id: 0,
};
/// The song-area centre in the client (FabledRealmsAreas.xml:3488); the viewer stands on it.
const DANCE_SPOT: [f32; 3] = [-2384.4, -26.77, 1214.94];
const OCT_21_2026_MS: u64 = 1_792_540_800_000; // 00:00 UTC, in the season
const NOV_12_2026_MS: u64 = 1_794_441_600_000; // the day after the season's last day
const DAY_MS: u64 = 86_400_000;
const POOF: i32 = 21; // PFX_smoke_black_explosion
const BOOMBOX_SPARKS: i32 = 5120;
const FIREWORKS: i32 = 5349;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn content() -> PathBuf {
    crate_root().join("data/content")
}
fn lab(path: &str) -> PathBuf {
    crate_root().join("../../../lab").join(path)
}
fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}
fn shipped() -> Value {
    serde_json::from_str(&read(&content().join(world_events::FILE))).unwrap()
}
fn row(file: &Value) -> &Value {
    file["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["key"] == KEY)
        .unwrap_or_else(|| panic!("{KEY} is not shipped"))
}
fn int(value: &Value) -> i32 {
    i32::try_from(value.as_i64().unwrap()).unwrap()
}
/// Every `id="N"` of a `<tag ...>` element in a client file.
fn xml_ids(path: &Path, tag: &str) -> HashSet<i32> {
    let open = format!("<{tag} ");
    read(path)
        .split(&open)
        .skip(1)
        .filter_map(|element| {
            let element = &element[..element.find('>')?];
            let at = element.find(" id=\"")? + 5;
            element[at..].split('"').next()?.parse().ok()
        })
        .collect()
}
fn resources(file: &str) -> PathBuf {
    lab("working/OSFR.Client/OSFR Client/Resources").join(file)
}
fn locale() -> BTreeMap<i64, String> {
    let rows: Value = serde_json::from_str(&read(&lab("reports/locale-id-to-text.json"))).unwrap();
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["id"].as_i64().unwrap(),
                row["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// Every step's op of the timeline, the `one_of` branches' included: `(op name, its fields)`.
fn ops(timeline: &Value) -> Vec<(String, Value)> {
    let mut found = Vec::new();
    for step in timeline.as_array().unwrap() {
        let (name, fields) = step["op"].as_object().unwrap().iter().next().unwrap();
        match name.as_str() {
            "one_of" => {
                for branch in fields["branches"].as_array().unwrap() {
                    found.extend(ops(branch));
                }
            }
            _ => found.push((name.clone(), fields.clone())),
        }
    }
    found
}

#[test]
fn every_id_resolves_against_the_client() {
    let file = shipped();
    let event = row(&file);
    let ops = ops(&event["timeline"]);
    let locale = locale();
    let (mut strings, mut areas, mut animations, mut effects) = (
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeSet::new(),
    );
    let announce = &event["schedule"]["announce"];
    strings.insert(int(&announce["locale_id"]));
    assert_eq!(
        announce["text"], locale[&5197],
        "the kickoff line is the client's"
    );

    let models = models::read_models();
    for actor in event["actors"].as_array().unwrap() {
        let model = actor["model_id"].as_i64().unwrap();
        assert!(
            models.contains_key(&model),
            "model {model} is not in Models.txt"
        );
        animations.insert(int(&actor["stand_animation_id"]));
        let name = int(&actor["name_string_id"]);
        if name != 0 {
            strings.insert(name);
            assert_eq!(
                actor["npc_name"],
                locale[&i64::from(name)],
                "actor {}",
                actor["key"]
            );
        }
        assert!(!actor["source"].as_str().unwrap().is_empty());
    }
    for (name, op) in &ops {
        let id = |field: &str| int(&op[field]);
        match name.as_str() {
            "say" => drop(strings.insert(id("string_id"))),
            "broadcast" => {
                strings.insert(id("locale_id"));
                let line = locale[&i64::from(id("locale_id"))].as_str();
                assert!(
                    op["text"].as_str().unwrap().ends_with(line),
                    "{}",
                    op["text"]
                );
            }
            "area" => drop(areas.insert(id("area_id"))),
            "base_anim" => drop(animations.insert(id("id"))),
            "cycle_anim" => animations.extend(op["ids"].as_array().unwrap().iter().map(int)),
            "tag_on" | "tag_off" => drop(effects.insert(id("effect_id"))),
            "spawn" | "remove" if !op["effect_id"].is_null() => {
                effects.insert(id("effect_id"));
            }
            "spawn" | "remove" | "move" | "face" => {}
            other => panic!("unexpected op {other}"),
        }
    }
    for id in &strings {
        assert!(
            locale.contains_key(&i64::from(*id)),
            "string {id} is not in the locale"
        );
    }
    let area_file = lab("extracted/named/Assets_000/Assets_000/FabledRealmsAreas.xml");
    let known = xml_ids(&area_file, "AreaDefinition");
    assert_eq!(areas, BTreeSet::from([4299, 4300, 4302, 4303, 4304, 4305]));
    assert!(areas.iter().all(|id| known.contains(id)), "{areas:?}");
    let mut known = xml_ids(&resources("AnimationTypes.xml"), "AnimationSlot");
    known.extend(xml_ids(&resources("AnimationGroups.xml"), "AnimationGroup"));
    assert!(
        animations.iter().all(|id| known.contains(id)),
        "{animations:?}"
    );
    let known = xml_ids(
        &resources("ActorCompositeEffectDefinitions.xml"),
        "EffectDefinition",
    );
    assert_eq!(effects, BTreeSet::from([POOF, BOOMBOX_SPARKS, FIREWORKS]));
    assert!(effects.iter().all(|id| known.contains(id)), "{effects:?}");
}

/// One viewer standing on the dance spot, in the open world.
fn viewer() -> PlayerRegistry {
    let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
    players
        .join(SessionId(1), 1, 1, WORLD, DANCE_SPOT, 0.0)
        .unwrap();
    players
}
fn event() -> WorldEvent {
    let mut events = world_events::load(&content()).unwrap();
    let at = events.iter().position(|event| event.key == KEY).unwrap();
    events.swap_remove(at)
}

/// What the viewer was sent, in order.
type Heard = Vec<Vec<u8>>;

/// Runs the event from `start_ms` for ten minutes (it lasts about ten and a half) in 250 ms
/// ticks; the returned run is the viewer's packets plus whether the run has ended.
fn play(start_ms: u64, stop_at: Option<u64>) -> (Heard, WorldEvents, ClientAreas) {
    let (players, mut areas) = (viewer(), ClientAreas::load(&content()).unwrap());
    let mut events = WorldEvents::default();
    assert!(events.start(event(), WORLD, start_ms));
    let mut heard = Heard::new();
    for now in (start_ms..start_ms + 650_000).step_by(250) {
        if stop_at.is_some_and(|stop| now == start_ms + stop) {
            assert!(events.stop(KEY));
        }
        let out = events.tick(now, OCT_21_2026_MS + now, &players, &mut areas);
        heard.extend(out.into_iter().map(|(_, packet)| packet));
    }
    (heard, events, areas)
}

fn guid(packet: &[u8]) -> u64 {
    u64::from_le_bytes(packet[4..12].try_into().unwrap())
}
fn i32_at(packet: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

/// The actors' indices in the event row, so a guid's low byte names who it is.
fn index_of(event: &Value, key: &str) -> u64 {
    event["actors"]
        .as_array()
        .unwrap()
        .iter()
        .position(|actor| actor["key"] == key)
        .unwrap_or_else(|| panic!("no actor {key}")) as u64
}

/// The two endings: a run whose prince announces the vampires, and one that announces the
/// werewolves. The branch comes from the run's seed (`start_ms` and the serial), so the first
/// start times that draw each are found by running.
fn both_endings() -> [Heard; 2] {
    let mut found: [Option<Heard>; 2] = [None, None];
    for start_ms in 0..64 {
        let heard = play(start_ms, None).0;
        let says = |id: i32| {
            heard
                .iter()
                .any(|p| p[..4] == [15, 0, 4, 0] && i32_at(p, 12) == id)
        };
        let side = match (says(432_432), says(432_407)) {
            (true, false) => 0,
            (false, true) => 1,
            other => panic!("one ending per run, saw {other:?}"),
        };
        found[side].get_or_insert(heard);
        if found.iter().all(Option::is_some) {
            break;
        }
    }
    found.map(|heard| heard.expect("both endings are reachable within 64 seeds"))
}

#[test]
fn the_six_rounds_alternate_sides_and_every_song_area_goes_off_once() {
    let file = shipped();
    let event = row(&file);
    let idx =
        |keys: &[&str]| -> BTreeSet<u64> { keys.iter().map(|k| index_of(event, k)).collect() };
    let vampires = idx(&["ed", "vampire1", "vampire2", "vampire3"]);
    let werewolves = idx(&["jake", "werewolf1", "werewolf2", "werewolf3"]);
    let boombox = index_of(event, "boombox");
    let song_areas = [4299, 4300, 4302, 4303, 4304, 4305];
    for heard in both_endings() {
        // Area packets: opcode 89, area i32 at 2, on-flag at 6. Each goes on and then off, in order.
        let area_packets: Vec<(usize, i32, bool)> = heard
            .iter()
            .enumerate()
            .filter(|(_, p)| p[..2] == [89, 0])
            .map(|(at, p)| (at, i32_at(p, 2), p[6] == 1))
            .collect();
        let sequence: Vec<_> = area_packets
            .iter()
            .map(|(_, area, on)| (*area, *on))
            .collect();
        let expected: Vec<_> = song_areas
            .iter()
            .flat_map(|a| [(*a, true), (*a, false)])
            .collect();
        assert_eq!(
            sequence, expected,
            "each area on then off, one at a time, in song order"
        );

        for (round, pair) in area_packets.chunks(2).enumerate() {
            let window = &heard[pair[0].0..pair[1].0];
            // The dancers are the ones given a one-shot move (35/8, base flag clear) in the song.
            let dancers: BTreeSet<u64> = window
                .iter()
                .filter(|p| p[..4] == [35, 0, 8, 0] && p[20] == 0)
                .map(|p| guid(p) & 0xff)
                .collect();
            let (side, rival) = match round % 2 {
                0 => (&vampires, &werewolves),
                _ => (&werewolves, &vampires),
            };
            assert_eq!(&dancers, side, "round {round} is danced by one whole side");
            let speakers: BTreeSet<u64> = window
                .iter()
                .filter(|p| p[..4] == [15, 0, 4, 0])
                .map(|p| guid(p) & 0xff)
                .collect();
            assert!(
                !speakers.is_empty() && speakers.iter().all(|s| rival.contains(s)),
                "round {round}: the rival side taunts, {speakers:?}"
            );
            // The boombox sparks are on for the song and off after it.
            let tags = |sub: u8| -> Vec<&Vec<u8>> {
                window
                    .iter()
                    .filter(|p| p[..4] == [35, 0, sub, 0] && guid(p) & 0xff == boombox)
                    .collect()
            };
            assert_eq!(tags(41).len(), 1, "round {round}: the sparks come on once");
            assert_eq!(i32_at(tags(41)[0], 16), BOOMBOX_SPARKS);
            assert!(
                tags(42).is_empty(),
                "round {round}: not off until the song ends"
            );
        }
        let sparks_on = heard.iter().filter(|p| p[..4] == [35, 0, 41, 0]).count();
        let sparks_off = heard.iter().filter(|p| p[..4] == [35, 0, 42, 0]).count();
        assert_eq!((sparks_on, sparks_off), (6, 6));
    }
}

#[test]
fn each_ending_names_its_winner_leaves_one_cake_and_no_actor_behind() {
    let file = shipped();
    let event = row(&file);
    let key = |index: u64| event["actors"][index as usize]["key"].as_str().unwrap();
    let says = |heard: &Heard| -> Vec<(String, i32)> {
        heard
            .iter()
            .filter(|p| p[..4] == [15, 0, 4, 0])
            .map(|p| (key(guid(p) & 0xff).to_owned(), i32_at(p, 12)))
            .collect()
    };
    let [vampire_win, werewolf_win] = both_endings();
    for (heard, winner_line, reward, cake, other_lines) in [
        (
            &vampire_win,
            432_432,
            432_433,
            "cake_vampire",
            [432_407, 432_435],
        ),
        (
            &werewolf_win,
            432_407,
            432_435,
            "cake_werewolf",
            [432_432, 432_433],
        ),
    ] {
        let said = says(heard);
        let ids: Vec<i32> = said.iter().map(|(_, id)| *id).collect();
        assert!(ids.contains(&winner_line) && ids.contains(&reward));
        assert!(
            other_lines.iter().all(|id| !ids.contains(id)),
            "the other ending's lines stay out"
        );
        assert!(
            said.contains(&("prince".into(), reward)),
            "the prince gives the cake"
        );

        // Every spawn is matched by a removal with the poof; exactly one cake, with fireworks.
        let spawned: Vec<u64> = heard
            .iter()
            .filter(|p| p[..4] == [35, 0, 2, 0])
            .map(|p| guid(p))
            .collect();
        let removed: Vec<(u64, i32)> = heard
            .iter()
            .filter(|p| p[..4] == [35, 0, 3, 0])
            .map(|p| {
                (
                    u64::from_le_bytes(p[6..14].try_into().unwrap()),
                    i32_at(p, 23),
                )
            })
            .collect();
        let live: BTreeSet<u64> = spawned.iter().copied().collect();
        assert_eq!(live.len(), spawned.len(), "no actor is shown twice");
        assert_eq!(
            live,
            removed
                .iter()
                .map(|(guid, _)| *guid)
                .collect::<BTreeSet<_>>()
        );
        assert!(
            removed.iter().all(|(_, effect)| *effect == POOF),
            "{removed:?}"
        );
        let cakes: Vec<&str> = live
            .iter()
            .map(|g| key(g & 0xff))
            .filter(|k| k.starts_with("cake"))
            .collect();
        assert_eq!(cakes, [cake]);
        let fireworks: Vec<u64> = heard
            .iter()
            .filter(|p| p[..4] == [35, 0, 16, 0] && i32_at(p, 20) == FIREWORKS)
            .map(|p| guid(p) & 0xff)
            .collect();
        assert_eq!(
            fireworks,
            [index_of(event, cake)],
            "one firework, on the cake"
        );
    }

    // The run is over: nothing is live, so the same key can start again at once.
    let (_, mut events, _) = play(0, None);
    assert!(
        events.start(self::event(), WORLD, 1_000_000),
        "the run ended with no actor live"
    );
}

#[test]
fn stopping_early_switches_the_song_off_and_removes_every_actor() {
    // Round 2's song (area 4300) is running 70 s in.
    let (heard, mut events, _) = play(0, Some(70_000));
    let areas: Vec<(i32, bool)> = heard
        .iter()
        .filter(|p| p[..2] == [89, 0])
        .map(|p| (i32_at(p, 2), p[6] == 1))
        .collect();
    let last = *areas.last().unwrap();
    assert!(
        areas.iter().any(|a| *a == (4300, true)),
        "stopped inside round 2's song: {areas:?}"
    );
    assert_eq!(
        last,
        (4300, false),
        "the song the stop caught is switched off: {areas:?}"
    );
    assert!(
        areas.len() % 2 == 0 && !areas.contains(&(4302, true)),
        "round 3 never starts"
    );

    let spawned = heard.iter().filter(|p| p[..4] == [35, 0, 2, 0]).count();
    let removed: Vec<i32> = heard
        .iter()
        .filter(|p| p[..4] == [35, 0, 3, 0])
        .map(|p| i32_at(p, 23))
        .collect();
    assert_eq!(
        (spawned, removed.len()),
        (10, 10),
        "every shown actor is removed"
    );
    assert!(
        removed.iter().all(|effect| *effect == POOF),
        "with the poof"
    );
    assert!(
        events.start(event(), WORLD, 1_000_000),
        "stopped means startable"
    );
}

/// The shipped file plus the two Bruce stages when T493 has not shipped them: the dance-off's
/// skip list needs their slot rows, so the sweep works before and after that content lands.
fn scheduled_events() -> Vec<WorldEvent> {
    let mut file = shipped();
    let rows = file["events"].as_array_mut().unwrap();
    for (key, offset) in [("bruce_snowhill", 0), ("bruce_wugachug", 120)] {
        if !rows.iter().any(|event| event["key"] == key) {
            rows.push(json!({"key": key, "actors": [], "timeline": [],
                "schedule": {"slot_minutes": 240, "offset_minutes": offset}}));
        }
    }
    let dir = std::env::temp_dir().join(format!("osfr-t464-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(world_events::FILE), file.to_string()).unwrap();
    let events = world_events::load_scheduled(&dir);
    let _ = fs::remove_dir_all(&dir);
    events.unwrap()
}

/// Sweeps one UTC day from midnight in 30 s ticks: `(kickoff lines, start lines, start minutes)`.
fn sweep(day_ms: u64) -> (usize, usize, Vec<u64>) {
    let file = shipped();
    let event = row(&file);
    let kickoff = chat::system(event["schedule"]["announce"]["text"].as_str().unwrap());
    let start_line = ops(&event["timeline"])
        .into_iter()
        .find(|(name, op)| name == "broadcast" && op["locale_id"] == 432_431)
        .unwrap()
        .1["text"]
        .clone();
    let start = chat::system(start_line.as_str().unwrap());
    let (players, mut areas) = (viewer(), ClientAreas::load(&content()).unwrap());
    let mut events = WorldEvents::default();
    events.schedule(scheduled_events(), WORLD, day_ms, &areas);
    let (mut kickoffs, mut starts, mut minutes) = (0, 0, Vec::new());
    for at in (0..DAY_MS).step_by(30_000) {
        for (_, packet) in events.tick(at, day_ms + at, &players, &mut areas) {
            kickoffs += usize::from(packet == kickoff);
            if packet == start {
                starts += 1;
                minutes.push((day_ms + at) / 60_000);
            }
        }
    }
    (kickoffs, starts, minutes)
}

#[test]
fn the_dance_off_runs_every_half_hour_in_season_and_not_after_it() {
    // 48 half-hour slots, minus the 12 on Bruce's two-hour starts, minus midnight (the sweep
    // starts at it): 36 kickoff lines and 36 starts, none on a Bruce start.
    let (kickoffs, starts, minutes) = sweep(OCT_21_2026_MS);
    assert_eq!((kickoffs, starts), (36, 36));
    assert!(
        minutes.iter().all(|m| m % 30 == 0 && m % 120 != 0),
        "{minutes:?}"
    );
    // The season ends on 11/11: a day later nothing is announced or started.
    assert_eq!(sweep(NOV_12_2026_MS), (0, 0, Vec::new()));
}
