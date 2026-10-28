//! T464: the Halloween Vampires vs Werewolves dance-off, the shipped `halloween_dance_off` row of
//! `world_events.json`. Every id in it is gated against the client's own tables; the run is
//! driven through the real `WorldEvents::tick` for each ending; the schedule is swept over a day
//! in and a day out of season.
#[path = "support/world_event_content.rs"]
mod world_event_content;

use osfr_tools::{
    chat,
    client_areas::ClientAreas,
    players::{InterestPolicy, PlayerRegistry, SessionId, ZoneInstance},
    world_events::{self, WorldEvent, WorldEvents},
};
use serde_json::Value;
use std::collections::BTreeSet;
use world_event_content::{assert_every_id_resolves, content, locale, ops, row, shipped};

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

#[test]
fn every_id_resolves_against_the_client() {
    let file = shipped();
    let event = row(&file, KEY);
    let ids = assert_every_id_resolves(event);
    assert_eq!(
        ids.areas,
        BTreeSet::from([4299, 4300, 4302, 4303, 4304, 4305])
    );
    assert_eq!(
        ids.effects,
        BTreeSet::from([POOF, BOOMBOX_SPARKS, FIREWORKS])
    );
    assert_eq!(
        event["schedule"]["announce"]["text"],
        locale()[&5197],
        "the kickoff line is the client's"
    );
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
    let event = row(&file, KEY);
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
    let event = row(&file, KEY);
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
        (11, 11),
        "every shown actor is removed: the ten stage actors and T538's Voting Box"
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

/// Sweeps one UTC day from midnight in 30 s ticks: `(kickoff lines, start lines, start minutes)`.
fn sweep(day_ms: u64) -> (usize, usize, Vec<u64>) {
    let file = shipped();
    let event = row(&file, KEY);
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
    // Bruce's two slot rows are in the shipped file; the dance-off's skip list needs them (T493).
    let scheduled = world_events::load_scheduled(&content()).unwrap();
    events.schedule(scheduled, WORLD, day_ms, &areas);
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
