//! T457: opcode 89 client areas (`osfr_tools::client_areas`) against the client reader
//! 0x008d5c60 (u16 opcode, i32 area id, u8 bool, i32 elapsed ms), the shipped
//! `client_areas.json`, and the client's FabledRealmsAreas.xml in the lab.
use osfr_tools::client_areas::ClientAreas;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const HALLOWEEN: std::ops::RangeInclusive<i32> = 3992..=4004;

/// Unix ms of a UTC date and time (Hinnant's `days_from_civil`, the inverse of the code under
/// test, so the two cannot share a mistake).
fn at(year: i64, month: i64, day: i64, hour: i64, minute: i64) -> u64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    ((days * 1440 + hour * 60 + minute) * 60_000) as u64
}

fn shipped() -> ClientAreas {
    ClientAreas::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap()
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn scratch(tag: &str) -> Scratch {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("osfr-t457-{tag}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    Scratch(path)
}

/// A content file with the given offset and seasons over a small switchable set.
fn custom(tag: &str, offset: i32, seasons: serde_json::Value) -> (Scratch, ClientAreas) {
    let dir = scratch(tag);
    let content = serde_json::json!({
        "schema_version": 1,
        "utc_offset_minutes": offset,
        "switchable": {"area_ids": [17, 4285, 4173, 3992], "source": "test"},
        "always_on": [],
        "seasons": seasons,
    });
    fs::write(dir.0.join("client_areas.json"), content.to_string()).unwrap();
    let areas = ClientAreas::load(&dir.0).unwrap();
    (dir, areas)
}

fn season(name: &str, start: (u8, u8), end: (u8, u8)) -> serde_json::Value {
    serde_json::json!({"name": name, "start_month": start.0, "start_day": start.1,
        "end_month": end.0, "end_day": end.1, "area_ids": [17], "source": "test"})
}

/// (area id, enabled, elapsed ms) of one 11-byte packet, read field by field like 0x008d5c60.
fn read(packet: &[u8]) -> (i32, bool, i32) {
    assert_eq!(packet.len(), 11, "{packet:?}");
    assert_eq!(&packet[..2], &89u16.to_le_bytes());
    (
        i32::from_le_bytes(packet[2..6].try_into().unwrap()),
        packet[6] != 0,
        i32::from_le_bytes(packet[7..11].try_into().unwrap()),
    )
}

fn read_all(packets: &[Vec<u8>]) -> Vec<(i32, bool, i32)> {
    packets.iter().map(|packet| read(packet)).collect()
}

fn enables(ids: impl IntoIterator<Item = i32>) -> Vec<(i32, bool, i32)> {
    ids.into_iter().map(|id| (id, true, 0)).collect()
}

#[test]
fn enable_and_disable_packets_are_the_readers_eleven_bytes() {
    let mut areas = shipped();
    let now = at(2026, 6, 1, 12, 0);
    // 4285 = 0x10BD (bruce_concert_wugachug_large); 89 = 0x59.
    assert_eq!(
        areas.set_area(4285, true, now).unwrap(),
        [0x59, 0, 0xBD, 0x10, 0, 0, 1, 0, 0, 0, 0]
    );
    assert_eq!(
        areas.set_area(4285, false, now).unwrap(),
        [0x59, 0, 0xBD, 0x10, 0, 0, 0, 0, 0, 0, 0]
    );
    // Out of season only the Sanctuary pad 4173 = 0x104D is on.
    assert_eq!(
        areas.world_ready_packets(now),
        [vec![0x59, 0, 0x4D, 0x10, 0, 0, 1, 0, 0, 0, 0]]
    );
}

#[test]
fn season_windows_are_inclusive_wrap_the_year_and_follow_the_offset() {
    let areas = shipped();
    let halloween = |time| areas.is_season_active("Halloween", time);
    assert!(halloween(at(2026, 10, 21, 12, 0)));
    assert!(!halloween(at(2026, 9, 19, 23, 59)));
    assert!(halloween(at(2026, 9, 20, 0, 0)), "the first day is inside");
    assert!(
        halloween(at(2026, 11, 11, 23, 59)),
        "the last day is inside"
    );
    assert!(!halloween(at(2026, 11, 12, 0, 0)));
    assert!(!halloween(at(2026, 6, 1, 0, 0)));
    assert!(!areas.is_season_active("Winterfest", at(2026, 10, 21, 0, 0)));

    let (_dir, wrap) = custom(
        "wrap",
        0,
        serde_json::json!([season("Yule", (12, 20), (1, 5))]),
    );
    let yule = |time| wrap.is_season_active("Yule", time);
    assert!(yule(at(2026, 12, 31, 12, 0)));
    assert!(yule(at(2027, 1, 1, 0, 0)));
    assert!(yule(at(2026, 12, 20, 0, 0)) && yule(at(2027, 1, 5, 23, 59)));
    assert!(!yule(at(2026, 12, 19, 23, 59)) && !yule(at(2027, 1, 6, 0, 0)));
    assert!(!yule(at(2027, 7, 1, 0, 0)));

    let (_dir, leap) = custom(
        "leap",
        0,
        serde_json::json!([season("Leap", (2, 29), (2, 29))]),
    );
    let leap_day = |time| leap.is_season_active("Leap", time);
    assert!(leap_day(at(2028, 2, 29, 12, 0)));
    assert!(!leap_day(at(2028, 2, 28, 23, 59)) && !leap_day(at(2028, 3, 1, 0, 0)));
    assert!(!leap_day(at(2027, 2, 28, 12, 0)) && !leap_day(at(2027, 3, 1, 12, 0)));

    // 03:00 UTC on Nov 12 is still Nov 11 at UTC-5; 23:30 UTC on Sep 19 is Sep 20 at UTC+1.
    let (_dir, west) = custom(
        "west",
        -300,
        serde_json::json!([season("H", (9, 20), (11, 11))]),
    );
    let (_dir2, east) = custom(
        "east",
        60,
        serde_json::json!([season("H", (9, 20), (11, 11))]),
    );
    let (_dir3, utc) = custom(
        "utc",
        0,
        serde_json::json!([season("H", (9, 20), (11, 11))]),
    );
    assert!(west.is_season_active("H", at(2026, 11, 12, 3, 0)));
    assert!(!utc.is_season_active("H", at(2026, 11, 12, 3, 0)));
    assert!(east.is_season_active("H", at(2026, 9, 19, 23, 30)));
    assert!(!utc.is_season_active("H", at(2026, 9, 19, 23, 30)));
}

#[test]
fn world_ready_sends_the_pad_and_the_season_areas_only_in_season() {
    let areas = shipped();
    let october = read_all(&areas.world_ready_packets(at(2026, 10, 21, 18, 0)));
    assert_eq!(october.len(), 14);
    assert_eq!(october, enables(HALLOWEEN.chain([4173])));
    assert_eq!(
        read_all(&areas.world_ready_packets(at(2026, 11, 12, 0, 0))),
        enables([4173])
    );
    // Nothing is saved: a fresh load (a restart or reconnect) recomputes the same answer.
    assert_eq!(
        read_all(&shipped().world_ready_packets(at(2026, 10, 21, 18, 0))),
        october
    );
}

/// `(name, (start month, day), (end month, day), area ids)` of every season row in the shipped file.
fn shipped_seasons() -> Vec<(String, (i64, i64), (i64, i64), Vec<i64>)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/client_areas.json");
    let content: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let number = |row: &serde_json::Value, key: &str| row[key].as_i64().unwrap();
    let rows = content["seasons"].as_array().unwrap().iter();
    rows.map(|row| {
        let ids = row["area_ids"].as_array().unwrap().iter();
        (
            row["name"].as_str().unwrap().to_string(),
            (number(row, "start_month"), number(row, "start_day")),
            (number(row, "end_month"), number(row, "end_day")),
            ids.map(|id| id.as_i64().unwrap()).collect(),
        )
    })
    .collect()
}

/// First and last minute of a season's window in `year`; a window that wraps the year end
/// (start after end) finishes in the next year.
fn window(year: i64, start: (i64, i64), end: (i64, i64)) -> (u64, u64) {
    let wraps = i64::from(start > end);
    (
        at(year, start.0, start.1, 0, 0),
        at(year + wraps, end.0, end.1, 23, 59),
    )
}

#[test]
fn every_shipped_season_is_active_inside_its_window_and_not_outside() {
    const DAY: u64 = 24 * 60 * 60_000;
    let areas = shipped();
    let rows = shipped_seasons();
    // The seasons the schedule evidence gives (quest-lines/wiring/seasons.json, merged by name).
    let names: BTreeSet<&str> = rows.iter().map(|row| row.0.as_str()).collect();
    for name in [
        "Halloween",
        "Snow Days",
        "12 Days of Presents",
        "Festival of Hearts",
        "Birthday Bash",
        "Child's Play Charity Drive",
        "Summer Camp",
        "Turkey Dance",
    ] {
        assert!(
            names.contains(name),
            "{name} is missing from client_areas.json"
        );
    }
    for (name, start, end, _) in rows {
        let active = |time| areas.is_season_active(&name, time);
        // 2028 is a leap year, 2026 is not.
        for year in [2026, 2028] {
            let (first, last) = window(year, start, end);
            assert!(
                last - first < 182 * DAY,
                "{name} is longer than half a year"
            );
            assert!(active(first) && active(last), "{name} {year} edges");
            assert!(active(first + (last - first) / 2), "{name} {year} middle");
            assert!(!active(first - 60_000), "{name} {year} the day before");
            assert!(!active(last + 60_000), "{name} {year} the day after");
            let opposite = first + (last - first) / 2 + 183 * DAY;
            assert!(!active(opposite), "{name} {year} half a year later");
        }
    }
}

#[test]
fn a_season_without_client_areas_gates_by_date_and_switches_nothing() {
    let rows = shipped_seasons();
    let quiet: Vec<_> = rows.iter().filter(|row| row.3.is_empty()).collect();
    assert!(
        !quiet.is_empty(),
        "the evidence seasons have no client areas"
    );
    for (name, start, end, _) in quiet {
        let (first, last) = window(2026, *start, *end);
        let mut areas = shipped();
        assert!(
            !areas.is_season_active("Halloween", first),
            "{name} overlaps Halloween"
        );
        // Only the pad is held, in and out of the window.
        let pad = enables([4173]);
        assert_eq!(read_all(&areas.world_ready_packets(first)), pad, "{name}");
        assert_eq!(
            read_all(&areas.world_ready_packets(last + 60_000)),
            pad,
            "{name}"
        );
        // Crossing either edge changes no area, so no packet.
        assert!(areas.season_changes(first - 60_000).is_empty());
        assert!(areas.season_changes(first).is_empty(), "{name} opens");
        assert!(
            areas.season_changes(last + 60_000).is_empty(),
            "{name} closes"
        );
    }

    // The loader takes an empty list too, whatever the shipped rows are.
    let (_dir, areas) = custom(
        "quiet",
        0,
        serde_json::json!([{"name": "Quiet", "start_month": 3, "start_day": 1, "end_month": 3,
            "end_day": 2, "area_ids": [], "source": "test"}]),
    );
    assert!(areas.is_season_active("Quiet", at(2026, 3, 1, 12, 0)));
    assert!(areas.world_ready_packets(at(2026, 3, 1, 12, 0)).is_empty());
}

#[test]
fn an_event_toggle_reports_elapsed_time_and_unknown_ids_are_refused() {
    let mut areas = shipped();
    let t0 = at(2026, 6, 1, 20, 0);
    assert!(areas.set_area(4285, true, t0).is_some());
    assert!(areas.set_area(4285, true, t0 + 5_000).is_none(), "a repeat");
    assert_eq!(
        read_all(&areas.world_ready_packets(t0 + 30_000)),
        [(4173, true, 0), (4285, true, 30_000)]
    );
    // A clock behind the toggle never underflows.
    assert_eq!(read(&areas.world_ready_packets(t0 - 1)[1]), (4285, true, 0));
    let before = areas.world_ready_packets(t0 + 30_000);
    // 4172 is defined but enabled by default; 999_999 and 0 are not client areas.
    for id in [4172, 999_999, 0, -4285] {
        assert!(areas.set_area(id, true, t0).is_none(), "{id}");
        assert!(areas.set_area(id, false, t0).is_none(), "{id}");
    }
    assert_eq!(
        areas.world_ready_packets(t0 + 30_000),
        before,
        "a refusal changed state"
    );
    assert_eq!(
        read(&areas.set_area(4285, false, t0).unwrap()),
        (4285, false, 0)
    );
    assert!(areas.set_area(4285, false, t0).is_none());
    assert_eq!(read_all(&areas.world_ready_packets(t0)), enables([4173]));
}

#[test]
fn season_changes_report_each_boundary_once_even_with_a_clock_going_back() {
    let mut areas = shipped();
    let end = at(2026, 11, 12, 0, 0);
    assert!(
        areas.season_changes(end - 60_000).is_empty(),
        "the first call is the baseline"
    );
    assert!(areas.season_changes(end - 1).is_empty());
    let off = read_all(&areas.season_changes(end));
    assert_eq!(off, HALLOWEEN.map(|id| (id, false, 0)).collect::<Vec<_>>());
    assert!(areas.season_changes(end + 30_000).is_empty(), "same minute");
    assert!(areas.season_changes(end + 60_000).is_empty(), "no change");
    // The clock goes back into the season: one enable each, never repeated.
    assert_eq!(
        read_all(&areas.season_changes(end - 120_000)),
        enables(HALLOWEEN)
    );
    assert!(areas.season_changes(end - 180_000).is_empty());
    assert!(areas.season_changes(at(2026, 10, 1, 0, 0)).is_empty());

    let mut start = shipped();
    let first = at(2026, 9, 20, 0, 0);
    assert!(start.season_changes(first - 60_000).is_empty());
    assert_eq!(read_all(&start.season_changes(first)), enables(HALLOWEEN));
    assert!(start.season_changes(first + 60_000).is_empty());
}

#[test]
fn an_event_and_a_season_sharing_an_area_keep_it_on_while_either_holds_it() {
    let mut areas = shipped();
    let october = at(2026, 10, 31, 20, 0);
    // The season already shows 3992: the toggle is recorded but sends nothing either way.
    assert!(areas.set_area(3992, true, october).is_none());
    assert!(areas.set_area(3992, false, october).is_none());
    assert_eq!(
        read_all(&areas.world_ready_packets(october)),
        enables(HALLOWEEN.chain([4173]))
    );
    // Held by the toggle when the season ends: only the other twelve go off.
    assert!(areas.set_area(3992, true, october).is_none());
    let end = at(2026, 11, 12, 0, 0);
    assert!(areas.season_changes(end - 60_000).is_empty());
    assert_eq!(
        read_all(&areas.season_changes(end)),
        (3993..=4004).map(|id| (id, false, 0)).collect::<Vec<_>>()
    );
    let after = read_all(&areas.world_ready_packets(end));
    assert_eq!(
        after,
        [(3992, true, (end - october) as i32), (4173, true, 0)]
    );
    assert_eq!(
        read(&areas.set_area(3992, false, end).unwrap()),
        (3992, false, 0)
    );
    // The always-on pad cannot be switched off by an event.
    assert!(areas.set_area(4173, true, end).is_none());
    assert!(areas.set_area(4173, false, end).is_none());
    assert_eq!(read_all(&areas.world_ready_packets(end)), enables([4173]));
}

#[test]
fn missing_content_means_no_areas_and_malformed_content_names_the_file() {
    let empty = scratch("empty");
    let mut areas = ClientAreas::load(&empty.0).unwrap();
    assert!(areas.world_ready_packets(at(2026, 10, 21, 0, 0)).is_empty());
    assert!(areas.season_changes(at(2026, 10, 21, 0, 0)).is_empty());
    assert!(areas.set_area(4173, true, 0).is_none());
    assert!(!areas.is_season_active("Halloween", at(2026, 10, 21, 0, 0)));

    let valid = || {
        serde_json::json!({
            "schema_version": 1,
            "switchable": {"area_ids": [17, 4285], "source": "test"},
            "always_on": [{"area_id": 17, "source": "test"}],
            "seasons": [season("A", (1, 1), (1, 2))],
        })
    };
    let dir = scratch("bad");
    let path = dir.0.join("client_areas.json");
    fs::write(&path, valid().to_string()).unwrap();
    ClientAreas::load(&dir.0).expect("the base case loads");
    let many: Vec<i32> = (1..=257).collect();
    let cases: Vec<(&str, Box<dyn Fn(&mut serde_json::Value)>)> = vec![
        ("schema", Box::new(|c| c["schema_version"] = 2.into())),
        ("unknown field", Box::new(|c| c["extra"] = 1.into())),
        (
            "offset high",
            Box::new(|c| c["utc_offset_minutes"] = 841.into()),
        ),
        (
            "offset low",
            Box::new(|c| c["utc_offset_minutes"] = (-721).into()),
        ),
        (
            "257 ids",
            Box::new(move |c| c["switchable"]["area_ids"] = many.clone().into()),
        ),
        (
            "duplicate id",
            Box::new(|c| c["switchable"]["area_ids"] = serde_json::json!([17, 17])),
        ),
        (
            "zero id",
            Box::new(|c| c["switchable"]["area_ids"] = serde_json::json!([0, 17])),
        ),
        (
            "no source",
            Box::new(|c| c["switchable"]["source"] = " ".into()),
        ),
        (
            "pad not switchable",
            Box::new(|c| c["always_on"][0]["area_id"] = 4173.into()),
        ),
        (
            "Feb 30",
            Box::new(|c| c["seasons"][0] = season("A", (2, 1), (2, 30))),
        ),
        (
            "month 13",
            Box::new(|c| c["seasons"][0]["start_month"] = 13.into()),
        ),
        (
            "day 0",
            Box::new(|c| c["seasons"][0]["start_day"] = 0.into()),
        ),
        (
            "season id",
            Box::new(|c| c["seasons"][0]["area_ids"] = serde_json::json!([4173])),
        ),
        (
            "duplicate name",
            Box::new(|c| {
                c["seasons"] =
                    serde_json::json!([season("A", (1, 1), (1, 2)), season("A", (3, 1), (3, 2))])
            }),
        ),
        ("malformed", Box::new(|c| *c = "not an object".into())),
    ];
    for (label, change) in cases {
        let mut content = valid();
        change(&mut content);
        fs::write(&path, content.to_string()).unwrap();
        let error = ClientAreas::load(&dir.0).expect_err(label);
        assert!(
            error.to_string().contains("client_areas.json"),
            "{label}: {error}"
        );
    }
}

/// `(id, AreaEnabled="0")` for every `<AreaDefinition>` in the client's FabledRealmsAreas.xml.
fn client_area_definitions() -> Vec<(i32, bool)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/extracted/named/Assets_000/Assets_000/FabledRealmsAreas.xml");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    text.split("<AreaDefinition ")
        .skip(1)
        .map(|tail| {
            let tag = &tail[..tail.find('>').unwrap()];
            let id = tag
                .split(" id=\"")
                .nth(1)
                .unwrap_or_else(|| panic!("no id: {tag}"));
            let id = id[..id.find('"').unwrap()].parse().unwrap();
            (id, tag.contains("AreaEnabled=\"0\""))
        })
        .collect()
}

#[test]
fn every_content_area_is_a_client_area_the_client_ships_disabled() {
    let definitions = client_area_definitions();
    assert_eq!(definitions.len(), 1279);
    let disabled: BTreeSet<i32> = definitions
        .iter()
        .filter(|(_, disabled)| *disabled)
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(disabled.len(), 138);
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/client_areas.json");
    let content: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let ids = |value: &serde_json::Value| -> Vec<i32> {
        let ids = value.as_array().unwrap().iter();
        ids.map(|id| id.as_i64().unwrap() as i32).collect()
    };
    let switchable = ids(&content["switchable"]["area_ids"]);
    assert_eq!(
        switchable.iter().copied().collect::<BTreeSet<_>>(),
        disabled
    );
    let rows = content["always_on"].as_array().unwrap();
    let mut used = ids(&rows.iter().map(|row| row["area_id"].clone()).collect());
    for season in content["seasons"].as_array().unwrap() {
        used.extend(ids(&season["area_ids"]));
    }
    assert!(!used.is_empty());
    for id in used {
        assert!(
            disabled.contains(&id),
            "{id} is not an AreaEnabled=\"0\" client area"
        );
    }
    // The event areas later briefs switch are client areas too (concert 17/4285, dance-off).
    for id in [17, 4285, 4299, 4300, 4302, 4303, 4304, 4305] {
        assert!(disabled.contains(&id), "{id}");
    }
}
