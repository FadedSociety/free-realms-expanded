//! T493: Bruce's concert. `data/content/world_events.json` carries two scheduled events,
//! `bruce_snowhill` and `bruce_wugachug`, that alternate on 240-minute slots (offsets 0 and 120).
//! The first test checks every client id the rows use against the client's own tables; the rest
//! drive the real `WorldEvents` runner on a fixed clock with the shipped file and areas.
#[path = "support/world_event_content.rs"]
mod world_event_content;

use osfr_tools::{
    client_areas::ClientAreas,
    players::{InterestPolicy, PlayerRegistry, SessionId, ZoneInstance},
    startup_world::{NpcSpawn, parse_add_npc},
    world_events::{self, WorldEvents},
};
use std::collections::BTreeSet;
use world_event_content::{
    assert_every_id_resolves, content, int, lab, locale, read, read_models, resources, row, shipped,
};

const WORLD: ZoneInstance = ZoneInstance {
    zone_id: 0,
    instance_id: 0,
};
/// 2026-10-24T00:00:00Z in unix ms. Local midnight is this minus the shipped UTC offset.
const MIDNIGHT_UTC_MS: u64 = 1_792_800_000_000;
const IDLE: [i32; 6] = [3333, 3354, 3363, 3319, 3347, 3346];
const AIR_GUITAR: i32 = 3350;
/// The stand animation every actor row uses, in both client animation tables as `loc_stand`.
const STAND: i32 = 1;
const POOF: i32 = 21;
const GATHER_S: u64 = 300;
const LEAVES_S: u64 = 481; // gathering 300 s plus the 181 s song
const SNOWHILL: [f32; 3] = [317.66, 29.83, 484.14];
const WUGACHUG: [f32; 3] = [74.6, 26.78, -413.2];
/// The script's headings (BruceConcert.lua:28,34), which the shipped rows carry.
const SNOW_HEADING: f32 = -2.4;
const WUGA_HEADING: f32 = -1.715594;

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

/// Local midnight in unix ms: UTC midnight minus the content's UTC offset. A negative offset
/// (west of UTC) moves local midnight later, so a checked signed subtraction is required.
fn local_midnight(unix_midnight_ms: u64, offset_minutes: i32) -> u64 {
    u64::try_from(i64::try_from(unix_midnight_ms).unwrap() - i64::from(offset_minutes) * 60_000)
        .unwrap()
}

/// The stand slot plus the two `cycle_anim` sets, so a change to either list moves both checks.
fn wanted_animations() -> BTreeSet<i32> {
    IDLE.into_iter().chain([STAND, AIR_GUITAR]).collect()
}

#[test]
fn every_id_resolves_in_the_client_tables() {
    let file = shipped();
    // Models.txt: the model row only. Its other numbers (3333, ...) are unrelated models.
    let models = read_models();
    assert!(
        models[&20].contains("human_rockstar_m.adr"),
        "{:?}",
        models[&20]
    );
    let locale = locale();
    assert_eq!(locale[&201], "Bruce");
    let types = read(&resources("AnimationTypes.xml"));
    let groups = read(&resources("AnimationGroups.xml"));
    let effects = read(&resources("ActorCompositeEffectDefinitions.xml"));
    let areas_xml = read(&lab(
        "extracted/named/Assets_000/Assets_000/FabledRealmsAreas.xml",
    ));
    for (key, area, stage, heading, music, line) in [
        ("bruce_snowhill", 17, SNOWHILL, SNOW_HEADING, "5393", None),
        (
            "bruce_wugachug",
            4285,
            WUGACHUG,
            WUGA_HEADING,
            "17715",
            Some(5142),
        ),
    ] {
        let event = row(&file, key);
        let ids = assert_every_id_resolves(event);
        assert_eq!(ids.models, BTreeSet::from([20]));
        let mut strings = BTreeSet::from([201]);
        strings.extend(line);
        assert_eq!(ids.strings, strings);
        if let Some(id) = line {
            // The client line exists and copies verbatim (shared gate: it ends the text).
            let (locale_id, text) = &ids.broadcasts[0];
            assert_eq!(
                (*locale_id, text.as_str()),
                (id, locale[&i64::from(id)].as_str())
            );
        } else {
            assert!(ids.broadcasts.is_empty(), "Snowhill has no client line");
        }
        assert_eq!(ids.areas, BTreeSet::from([area]));
        assert_eq!(ids.effects, BTreeSet::from([POOF]));
        // Animations: the stand slot, and the emote slot and group, in both client tables.
        let wanted = wanted_animations();
        assert_eq!(ids.animations, wanted);
        for id in wanted {
            let id = i64::from(id);
            let in_types = if id == i64::from(STAND) {
                types
                    .lines()
                    .any(|l| l.contains("<AnimationSlot name=\"loc_stand\" id=\"1\" "))
            } else {
                types.lines().any(|l| {
                    l.contains("<AnimationSlot name=\"emo_")
                        && l.contains(&format!(" id=\"{id}\" "))
                })
            };
            let in_groups = if id == i64::from(STAND) {
                groups
                    .lines()
                    .any(|l| l.contains("<AnimationGroup name=\"loc_stand\" id=\"1\">"))
            } else {
                groups.lines().any(|l| {
                    l.contains("<AnimationGroup name=\"emo_")
                        && l.contains(&format!(" id=\"{id}\""))
                })
            };
            assert!(
                in_types && in_groups,
                "animation {id}: types {in_types}, groups {in_groups}"
            );
        }
        // Areas: each stage ships disabled with its music, and the stage is inside it.
        let tag = area_tag(&areas_xml, area);
        assert_eq!(attribute(tag, "AreaEnabled"), "0");
        let after = &areas_xml[areas_xml.find(tag).unwrap()..];
        let body = &after[..after.find("</AreaDefinition>").unwrap()];
        assert!(
            body.contains(&format!("SoundDefinitionId=\"{music}\"")),
            "area {area} music"
        );
        let centre = ["x1", "y1", "z1"].map(|axis| attribute(tag, axis).parse::<f64>().unwrap());
        let radius: f64 = attribute(tag, "radius").parse().unwrap();
        let distance = (0..3)
            .map(|i| (f64::from(stage[i]) - centre[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(
            distance <= radius,
            "stage {stage:?} is {distance} m from area {area}, radius {radius}"
        );
        // The shipped pose is the script's, so a typo in the row fails even if the wire copies it.
        let actor = &event["actors"][0];
        for (axis, expected) in stage.iter().enumerate() {
            let got = actor["position"][axis].as_f64().unwrap() as f32;
            assert!((got - expected).abs() < 1e-3, "{key} x{axis}: {got}");
        }
        let shipped_heading = actor["heading"].as_f64().unwrap() as f32;
        assert!(
            (shipped_heading - heading).abs() < 1e-6,
            "{key} heading {shipped_heading}"
        );
        assert_eq!(int(&actor["stand_animation_id"]), STAND);
    }
    // The poof is Sanctuary's StageChangeCompositeEffectId (BruceConcert.lua:8),
    // PFX_smoke_black_explosion.
    assert!(effects.contains("<EffectDefinition name=\"PFX_smoke_black_explosion\" id=\"21\" "));
}

#[test]
fn a_negative_utc_offset_moves_local_midnight_later() {
    assert_eq!(local_midnight(MIDNIGHT_UTC_MS, 0), MIDNIGHT_UTC_MS);
    assert_eq!(
        local_midnight(MIDNIGHT_UTC_MS, -360),
        MIDNIGHT_UTC_MS + 6 * HOUR * 1000
    );
    assert_eq!(
        local_midnight(MIDNIGHT_UTC_MS, 330),
        MIDNIGHT_UTC_MS - 5 * HOUR * 1000 - 30 * 60_000
    );
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
    /// Every AddNpc body the runner sent, decoded by the production reader.
    spawns: Vec<(SessionId, NpcSpawn)>,
    guid: Option<u64>,
}

impl Rig {
    fn new() -> Self {
        let areas = ClientAreas::load(&content()).unwrap();
        let midnight = local_midnight(MIDNIGHT_UTC_MS, areas.utc_offset_minutes());
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
            spawns: Vec::new(),
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
            let seen = self.decode(session, &packet);
            self.log.push((second, session, seen));
        }
    }

    fn decode(&mut self, session: SessionId, packet: &[u8]) -> Seen {
        let i32_at = |at: usize| i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
        match packet[..4] {
            // 89/0 area, client reader 0x008d5c60: i32 area id at 2, on flag at 6.
            [89, 0, ..] => Seen::Area(i32_at(2), packet[6] != 0),
            // 15/3 system line from `chat::system`: the text follows the 8-byte header.
            [15, 0, ..] => {
                Seen::Line(String::from_utf8(packet[8..packet.len() - 1].to_vec()).unwrap())
            }
            // 35/2 AddNpc, client body reader 0x00921a10: actor id at 4, name/model and the
            // transform follow; the production parser checks the whole sequence.
            [35, 0, 2, 0] => {
                let spawn = parse_add_npc(packet).expect("the AddNpc body parses");
                self.guid = Some(spawn.actor_id);
                self.spawns.push((session, spawn));
                Seen::AddNpc
            }
            // 35/16 PlayCompositeEffect, client reader 0x008e5f50: effect id at 20.
            [35, 0, 16, 0] => Seen::Poof(i32_at(20)),
            // 35/3 RemovePlayerGracefully, client readers 0x008d69f0 then 0x008d6a60:
            // composite effect id at 23.
            [35, 0, 3, 0] => Seen::Remove(i32_at(23)),
            // 35/8 SetAnimation, client reader 0x008e5dd0: u64 guid at 4, i32 animation id
            // at 12, i32 zero at 16, u8 base flag at 20 (21 bytes).
            [35, 0, 8, 0] => {
                assert_eq!(packet.len(), 21, "{packet:?}");
                let guid = u64::from_le_bytes(packet[4..12].try_into().unwrap());
                assert_eq!(Some(guid), self.guid, "an animation for another actor");
                assert_eq!(i32_at(16), 0, "the client reader's second i32");
                assert_eq!(packet[20], 0, "a one-shot animation, not a base change");
                let id = i32_at(12);
                if IDLE.contains(&id) {
                    Seen::Idle(id)
                } else {
                    assert_eq!(id, AIR_GUITAR);
                    Seen::Guitar
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
fn the_spawn_carries_the_shipped_actor_and_pose() {
    let mut rig = Rig::new();
    // A slot start is chosen strictly after the arming time, so arm a minute early.
    rig.schedule(DAY - 60);
    rig.run(DAY - 60, DAY, 1);
    rig.run(DAY + 2 * HOUR, DAY + 2 * HOUR, 1);
    let file = shipped();
    for (session, key) in [(1, "bruce_snowhill"), (2, "bruce_wugachug")] {
        let actor = &row(&file, key)["actors"][0];
        // 35/2 AddNpc (client body reader 0x00921a10) carries the row's model, name and pose.
        let spawned: Vec<&NpcSpawn> = rig
            .spawns
            .iter()
            .filter(|(other, _)| other.0 == session)
            .map(|(_, spawn)| spawn)
            .collect();
        assert_eq!(spawned.len(), 1, "{key}");
        let spawn = spawned[0];
        assert_eq!(spawn.model_id, int(&actor["model_id"]));
        assert_eq!(spawn.name_string_id, int(&actor["name_string_id"]));
        assert_eq!(spawn.npc_name, actor["npc_name"].as_str().unwrap());
        assert_eq!(spawn.stand_animation_id, int(&actor["stand_animation_id"]));
        for axis in 0..3 {
            let expected = actor["position"][axis].as_f64().unwrap() as f32;
            assert!(
                (spawn.position[axis] - expected).abs() < 1e-3,
                "{key} AddNpc x{axis}: {}",
                spawn.position[axis]
            );
        }
        assert_eq!(spawn.position[3], 1.0);
        assert_eq!(spawn.rotation[1], 0.0);
        assert_eq!(spawn.rotation[3], 0.0);
        let heading = actor["heading"].as_f64().unwrap() as f32;
        let spawned_heading = spawn.rotation[0].atan2(spawn.rotation[2]);
        assert!(
            (spawned_heading - heading).abs() < 1e-4,
            "{key} AddNpc heading {spawned_heading}"
        );
    }
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
