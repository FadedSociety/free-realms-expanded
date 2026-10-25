//! T493: Bruce's concert. `data/content/world_events.json` carries two scheduled events,
//! `bruce_snowhill` and `bruce_wugachug`, that alternate on 240-minute slots (offsets 0 and 120).
//! The first test checks every client id the rows use against the client's own tables; the rest
//! drive the real `WorldEvents` runner on a fixed clock with the shipped file and areas.
use osfr_tools::client_areas::ClientAreas;
use osfr_tools::players::{InterestPolicy, PlayerRegistry, SessionId, ZoneInstance};
use osfr_tools::world_events::{self, WorldEvents, piece_052::animation_packet};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const WORLD: ZoneInstance = ZoneInstance {
    zone_id: 0,
    instance_id: 0,
};
/// 2026-10-24T00:00:00Z in unix ms. Local midnight is this minus the shipped UTC offset.
const MIDNIGHT_UTC_MS: u64 = 1_792_800_000_000;
const IDLE: [i32; 6] = [3333, 3354, 3363, 3319, 3347, 3346];
const AIR_GUITAR: i32 = 3350;
const POOF: i32 = 21;
const GATHER_S: u64 = 300;
const LEAVES_S: u64 = 481; // gathering 300 s plus the 181 s song
const SNOWHILL: [f32; 3] = [317.66, 29.83, 484.14];
const WUGACHUG: [f32; 3] = [74.6, 26.78, -413.2];

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn content() -> PathBuf {
    manifest().join("data/content")
}

fn lab() -> PathBuf {
    manifest().join("../../../lab")
}

fn client_resources() -> PathBuf {
    lab().join("working/OSFR.Client/OSFR Client/Resources")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn shipped_file() -> Value {
    serde_json::from_str(&read(&content().join(world_events::FILE))).unwrap()
}

/// The value of attribute `name` in an XML start tag.
fn attribute<'a>(tag: &'a str, name: &str) -> &'a str {
    let tail = tag
        .split(&format!(" {name}=\""))
        .nth(1)
        .unwrap_or_else(|| panic!("no {name} in {tag}"));
    &tail[..tail.find('"').unwrap()]
}

/// The start tag of the `<AreaDefinition>` with `id` in the client's FabledRealmsAreas.xml.
fn area_tag(text: &str, id: i32) -> &str {
    text.split("<AreaDefinition ")
        .skip(1)
        .map(|tail| &tail[..tail.find('>').unwrap()])
        .find(|tag| attribute(tag, "id") == id.to_string())
        .unwrap_or_else(|| panic!("area {id} is not in FabledRealmsAreas.xml"))
}

/// Every id the rows use, gathered from the raw file so the check cannot share a mistake with the loader.
#[derive(Default)]
struct Ids {
    models: BTreeSet<i64>,
    names: BTreeSet<i64>,
    locales: Vec<(i64, String)>,
    areas: BTreeSet<i64>,
    animations: BTreeSet<i64>,
    effects: BTreeSet<i64>,
}

fn collect(file: &Value) -> Ids {
    let mut ids = Ids::default();
    let bruce = |event: &&Value| event["key"].as_str().unwrap().starts_with("bruce_");
    for event in file["events"].as_array().unwrap().iter().filter(bruce) {
        for actor in event["actors"].as_array().unwrap() {
            ids.models.insert(actor["model_id"].as_i64().unwrap());
            ids.names.insert(actor["name_string_id"].as_i64().unwrap());
        }
        for step in event["timeline"].as_array().unwrap() {
            let (op, body) = step["op"].as_object().unwrap().iter().next().unwrap();
            match op.as_str() {
                "spawn" | "remove" => {
                    ids.effects.extend(body["effect_id"].as_i64());
                }
                "broadcast" => {
                    let text = body["text"].as_str().unwrap().to_owned();
                    ids.locales
                        .extend(body["locale_id"].as_i64().map(|id| (id, text)));
                }
                "area" => {
                    ids.areas.insert(body["area_id"].as_i64().unwrap());
                }
                "cycle_anim" => ids.animations.extend(
                    body["ids"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_i64().unwrap()),
                ),
                other => panic!("the concert uses no {other} op"),
            }
        }
    }
    ids
}

#[test]
fn every_id_resolves_in_the_client_tables() {
    let ids = collect(&shipped_file());
    // Models.txt: the model row only. Its other numbers (3333, ...) are unrelated models.
    let models = read(&client_resources().join("Models.txt"));
    assert_eq!(ids.models, BTreeSet::from([20]));
    assert!(
        models
            .lines()
            .any(|l| l.starts_with("20^human_rockstar_m.adr^"))
    );
    // Locale: the nameplate is "Bruce", and a broadcast with a locale id copies that client line.
    let locale: Value =
        serde_json::from_str(&read(&lab().join("reports/locale-id-to-text.json"))).unwrap();
    let text = |id: i64| {
        let row = locale
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id);
        row.unwrap_or_else(|| panic!("locale {id} is missing"))["text"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    assert_eq!(ids.names, BTreeSet::from([201]));
    assert_eq!(text(201), "Bruce");
    assert_eq!(
        ids.locales,
        [(
            5142,
            "A concert is starting at the Wugachug Stage!".to_owned()
        )]
    );
    assert_eq!(text(5142), ids.locales[0].1);
    // Animations: the emote slot and group, in both client tables.
    let types = read(&client_resources().join("AnimationTypes.xml"));
    let groups = read(&client_resources().join("AnimationGroups.xml"));
    let wanted: BTreeSet<i64> = IDLE
        .iter()
        .chain([&AIR_GUITAR])
        .map(|&id| i64::from(id))
        .collect();
    assert_eq!(ids.animations, wanted);
    for id in wanted {
        let in_types = types.lines().any(|l| {
            l.contains("<AnimationSlot name=\"emo_") && l.contains(&format!(" id=\"{id}\" "))
        });
        let in_groups = groups.lines().any(|l| {
            l.contains("<AnimationGroup name=\"emo_") && l.contains(&format!(" id=\"{id}\""))
        });
        assert!(
            in_types && in_groups,
            "animation {id}: types {in_types}, groups {in_groups}"
        );
    }
    // Areas: both stage areas ship disabled with a music property, and each stage is inside its own.
    let areas_xml =
        read(&lab().join("extracted/named/Assets_000/Assets_000/FabledRealmsAreas.xml"));
    assert_eq!(ids.areas, BTreeSet::from([17, 4285]));
    for (id, stage, music) in [(17, SNOWHILL, "5393"), (4285, WUGACHUG, "17715")] {
        let tag = area_tag(&areas_xml, id);
        assert_eq!(attribute(tag, "AreaEnabled"), "0");
        let after = &areas_xml[areas_xml.find(tag).unwrap()..];
        let body = &after[..after.find("</AreaDefinition>").unwrap()];
        assert!(
            body.contains(&format!("SoundDefinitionId=\"{music}\"")),
            "area {id} music"
        );
        let centre = ["x1", "y1", "z1"].map(|axis| attribute(tag, axis).parse::<f64>().unwrap());
        let radius: f64 = attribute(tag, "radius").parse().unwrap();
        let distance = (0..3)
            .map(|i| (f64::from(stage[i]) - centre[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(
            distance <= radius,
            "stage {stage:?} is {distance} m from area {id}, radius {radius}"
        );
    }
    // The poof is Sanctuary's StageChangeCompositeEffectId (BruceConcert.lua:8), PFX_smoke_black_explosion.
    assert_eq!(ids.effects, BTreeSet::from([i64::from(POOF)]));
    let effects = read(&client_resources().join("ActorCompositeEffectDefinitions.xml"));
    assert!(effects.contains("<EffectDefinition name=\"PFX_smoke_black_explosion\" id=\"21\" "));
}

/// A packet's meaning: what the test counts and orders.
#[derive(Clone, Debug, PartialEq)]
enum Seen {
    AddNpc,
    /// 35/16 PlayCompositeEffect at the actor, the spawn poof.
    Poof(i32),
    Remove(i32),
    Area(i32, bool),
    Line(String),
    Idle(i32),
    Guitar,
}

struct Rig {
    events: WorldEvents,
    areas: ClientAreas,
    players: PlayerRegistry,
    /// Local midnight in unix ms.
    midnight: u64,
    /// Every packet the runner sent, with the local second it was sent. Session 1 stands at
    /// Snowhill's stage and session 2 at Wugachug's.
    log: Vec<(u64, SessionId, Seen)>,
    guid: Option<u64>,
}

impl Rig {
    fn new() -> Self {
        let areas = ClientAreas::load(&content()).unwrap();
        let midnight = MIDNIGHT_UTC_MS
            - u64::try_from(i64::from(areas.utc_offset_minutes()) * 60_000).unwrap();
        let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
        for (session, at) in [(1, SNOWHILL), (2, WUGACHUG)] {
            players
                .join(SessionId(session), session, session, WORLD, at, 0.0)
                .unwrap();
        }
        Self {
            events: WorldEvents::default(),
            areas,
            players,
            midnight,
            log: Vec::new(),
            guid: None,
        }
    }

    /// Arm the shipped schedule at `second` of the local day.
    fn schedule(&mut self, second: u64) {
        // Only the two stages: the dance-off (T464) shares the file and runs every 30 minutes in season.
        let events: Vec<_> = world_events::load_scheduled(&content())
            .unwrap()
            .into_iter()
            .filter(|event| event.key.starts_with("bruce_"))
            .collect();
        assert_eq!(events.len(), 2);
        self.events
            .schedule(events, WORLD, self.midnight + second * 1000, &self.areas);
    }

    /// Tick every `step_s` from `from_s` to `to_s` (local seconds); the monotonic clock is the local second.
    fn run(&mut self, from_s: u64, to_s: u64, step_s: u64) {
        for second in (from_s..=to_s).step_by(step_s as usize) {
            self.tick(second);
        }
    }

    fn tick(&mut self, second: u64) {
        let out = self.events.tick(
            second * 1000,
            self.midnight + second * 1000,
            &self.players,
            &mut self.areas,
        );
        for (session, packet) in out {
            let seen = self.decode(&packet);
            self.log.push((second, session, seen));
        }
    }

    fn decode(&mut self, packet: &[u8]) -> Seen {
        let i32_at = |at: usize| i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
        match packet[..2] {
            [89, 0] => Seen::Area(i32_at(2), packet[6] != 0),
            [15, 0] => Seen::Line(String::from_utf8(packet[8..packet.len() - 1].to_vec()).unwrap()),
            [35, 0] if packet[2] == 2 => {
                self.guid = Some(u64::from_le_bytes(packet[4..12].try_into().unwrap()));
                Seen::AddNpc
            }
            [35, 0] if packet[2] == 16 => Seen::Poof(i32_at(20)),
            [35, 0] if packet[2] == 3 => Seen::Remove(i32_at(23)),
            [35, 0] if packet[2] == 8 => {
                let guid = self.guid.expect("an animation before any AddNpc");
                let idle = IDLE
                    .iter()
                    .find(|&&id| packet == animation_packet(guid, id, false));
                match idle {
                    Some(&id) => Seen::Idle(id),
                    None => {
                        assert_eq!(packet, animation_packet(guid, AIR_GUITAR, false));
                        Seen::Guitar
                    }
                }
            }
            _ => panic!("unexpected packet {packet:?}"),
        }
    }

    /// What `session` saw, as (local second, thing).
    fn seen_by(&self, session: u64) -> Vec<(u64, Seen)> {
        let own = self.log.iter().filter(|(_, s, _)| *s == SessionId(session));
        own.map(|(second, _, seen)| (*second, seen.clone()))
            .collect()
    }
}

/// One whole run as one session saw it, relative to the run's start. The stage's own viewer sees
/// Bruce; both see the zone-wide line and the music area.
fn expected_run(start: u64, area: i32, line: &str, viewer: bool) -> Vec<(u64, Seen)> {
    let mut run = vec![(start, Seen::Line(line.to_owned()))];
    if viewer {
        run.splice(0..0, [(start, Seen::AddNpc), (start, Seen::Poof(POOF))]);
    }
    run.push((start + GATHER_S, Seen::Area(area, true)));
    run.push((start + LEAVES_S, Seen::Area(area, false)));
    if viewer {
        run.push((start + LEAVES_S, Seen::Remove(POOF)));
    }
    run
}

/// Everything but the animation plays, in order, with the plays checked on their own.
fn without_plays(seen: Vec<(u64, Seen)>) -> Vec<(u64, Seen)> {
    seen.into_iter()
        .filter(|(_, s)| !matches!(s, Seen::Idle(_) | Seen::Guitar))
        .collect()
}

const HOUR: u64 = 3600;
/// Local midnight of the day after the rig's day 0, so the clock can start a minute before it.
const DAY: u64 = 24 * HOUR;
const SNOW: &str = "A concert is starting at the Snowhill Stage!";
const WUGA: &str = "A concert is starting at the Wugachug Stage!";

/// Nothing the concert held on is still on, and no actor is left for a viewer who arrives later.
fn assert_nothing_left(rig: &mut Rig, second: u64) {
    let held: Vec<i32> = rig
        .areas
        .world_ready_packets(rig.midnight + second * 1000)
        .iter()
        .map(|packet| i32::from_le_bytes(packet[2..6].try_into().unwrap()))
        .collect();
    assert!(!held.contains(&17) && !held.contains(&4285), "{held:?}");
    let before = rig.log.len();
    rig.players
        .join(SessionId(3), 3, 3, WORLD, SNOWHILL, 0.0)
        .unwrap();
    rig.run(second, second + 60, 1);
    assert_eq!(rig.log.len(), before, "a tick after the run sent something");
}

#[test]
fn the_two_events_alternate_between_the_stages_every_two_hours() {
    let mut rig = Rig::new();
    rig.schedule(DAY - 60);
    rig.run(DAY - 60, DAY + 4 * HOUR + 600, 1);
    let (snow_0, wuga, snow_4) = (DAY, DAY + 2 * HOUR, DAY + 4 * HOUR);
    let both = |first: Vec<(u64, Seen)>, second: Vec<(u64, Seen)>, third: Vec<(u64, Seen)>| {
        [first, second, third].concat()
    };
    // Snowhill's viewer sees Bruce at 00:00 and 04:00 and only the line and music at 02:00.
    assert_eq!(
        without_plays(rig.seen_by(1)),
        both(
            expected_run(snow_0, 17, SNOW, true),
            expected_run(wuga, 4285, WUGA, false),
            expected_run(snow_4, 17, SNOW, true)
        )
    );
    assert_eq!(
        without_plays(rig.seen_by(2)),
        both(
            expected_run(snow_0, 17, SNOW, false),
            expected_run(wuga, 4285, WUGA, true),
            expected_run(snow_4, 17, SNOW, false)
        )
    );
    // Bruce's animations: idle plays 15-25 s apart while gathering, then the air guitar every 8 s.
    for (viewer, starts) in [(1, [snow_0, snow_4].as_slice()), (2, [wuga].as_slice())] {
        let plays = rig.seen_by(viewer);
        for &start in starts {
            let idle: Vec<u64> = plays
                .iter()
                .filter(|(_, s)| matches!(s, Seen::Idle(_)))
                .map(|(t, _)| *t)
                .filter(|t| (start..start + GATHER_S).contains(t))
                .collect();
            assert_eq!(idle[0], start, "the first idle play is at the start");
            assert!(
                idle.windows(2).all(|w| (15..=26).contains(&(w[1] - w[0]))),
                "{idle:?}"
            );
            let guitar: Vec<u64> = plays
                .iter()
                .filter(|(_, s)| *s == Seen::Guitar)
                .map(|(t, _)| *t)
                .filter(|t| (start..start + LEAVES_S).contains(t))
                .collect();
            assert_eq!(
                guitar,
                (0..23)
                    .map(|k| start + GATHER_S + 8 * k)
                    .collect::<Vec<_>>()
            );
        }
    }
    // Nothing is sent outside the three runs: every packet falls in a run's 481 s.
    for (second, _, seen) in &rig.log {
        let inside = [snow_0, wuga, snow_4]
            .iter()
            .any(|s| (*s..=s + LEAVES_S).contains(second));
        assert!(inside, "{seen:?} at {second}");
    }
    assert_nothing_left(&mut rig, snow_4 + 600);
}

#[test]
fn nothing_is_sent_between_the_concerts() {
    let mut rig = Rig::new();
    rig.schedule(DAY + LEAVES_S + 1);
    rig.run(DAY + LEAVES_S + 1, DAY + 2 * HOUR - 1, 7);
    assert!(rig.log.is_empty(), "{:?}", rig.log.first());
    // The second the slot opens, the concert starts.
    rig.run(DAY + 2 * HOUR, DAY + 2 * HOUR, 1);
    assert_eq!(rig.seen_by(2)[0].1, Seen::AddNpc);
}

#[test]
fn stopping_a_concert_early_turns_the_music_off_and_removes_bruce() {
    let mut rig = Rig::new();
    let events = world_events::load(&content()).unwrap();
    let event = events
        .into_iter()
        .find(|e| e.key == "bruce_wugachug")
        .unwrap();
    let start = 5 * HOUR;
    assert!(rig.events.start(event, WORLD, start * 1000));
    rig.run(start, start + GATHER_S + 10, 1);
    assert!(
        rig.seen_by(2)
            .contains(&(start + GATHER_S, Seen::Area(4285, true))),
        "the song is on"
    );
    let quiet = rig.log.len();
    assert!(rig.events.stop("bruce_wugachug"));
    assert!(
        !rig.events.stop("bruce_wugachug"),
        "a second stop finds nothing"
    );
    let stopped = start + GATHER_S + 11;
    rig.run(stopped, stopped, 1);
    let after: Vec<_> = rig.log[quiet..]
        .iter()
        .map(|(_, session, seen)| (session.0, seen.clone()))
        .collect();
    assert_eq!(
        after,
        [
            (2, Seen::Remove(POOF)),
            (1, Seen::Area(4285, false)),
            (2, Seen::Area(4285, false))
        ]
    );
    rig.run(stopped + 1, stopped + 2 * HOUR, 5);
    assert_eq!(rig.log.len(), quiet + 3, "nothing more after the stop");
    assert_nothing_left(&mut rig, stopped + 2 * HOUR + 1);
}
