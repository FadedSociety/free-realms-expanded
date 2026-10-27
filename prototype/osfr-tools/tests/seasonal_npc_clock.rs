//! T775: seasonal event NPCs follow their `client_areas.json` season instead of spawning all
//! year. The shipped `npc_spawns_extra.json` Tags the Legacy event block: the Super Spooktacular
//! Pumpkin Prince and its ghosts (Halloween), the Snow Days presents and Snowman Invaders, the
//! Festival of Hearts critters and the anniversary wagon (Birthday Bash). Every view judges the
//! tag at the wall clock with the same `ClientAreas::is_season_active` predicate the world-event
//! scheduler uses for the Halloween dance-off (`world_events/schedule.rs`), so a season boundary
//! adds/removes the actors through the ordinary 35/2 AddNpc and 35/3 removal diff.
//!
//! Wire: AddNpc body [35,0,2,0], reader `0x00921a10`; removal [35,0,3,0], submode 1, guid at
//! offset 6, readers `0x008d69f0` / `0x008d6a60` (`restoration/piece_056.rs`). The client has no
//! season field, so the gate is server content only.
//!
//! No test reads today's date: every clock below is fixed.
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;

use npc_visibility::{content_dir, seen_from};
use osfr_tools::{client_areas::ClientAreas, startup_world::parse_add_npc, world::NpcVisibility};
use serde_json::json;
use std::collections::BTreeSet;

/// 2026-10-31 12:00 UTC: inside Halloween (9/20 to 11/11) and no other shipped season.
const HALLOWEEN_MS: u64 = 1_793_448_000_000;
/// 2026-11-12 00:00 UTC: the first minute after Halloween's last day.
const HALLOWEEN_OVER_MS: u64 = 1_794_441_600_000;
/// 2026-12-20 12:00 UTC: inside Snow Days (12/1 to 1/14).
const SNOW_DAYS_MS: u64 = 1_797_768_000_000;
/// 2026-02-14 12:00 UTC: inside the Festival of Hearts (2/1 to 3/31).
const FESTIVAL_MS: u64 = 1_771_070_400_000;
/// 2026-05-01 12:00 UTC: inside the Birthday Bash (4/28 to 5/24).
const BIRTHDAY_MS: u64 = 1_777_636_800_000;
/// 2026-01-15 12:00 UTC: after Snow Days ends and before the Festival of Hearts.
const NO_SEASON_MS: u64 = 1_768_478_400_000;
/// 2026-02-28 12:00 and 2026-03-01 12:00 UTC: the synthetic area-less Quiet season (3/1-3/2).
const QUIET_OFF_MS: u64 = 1_772_280_000_000;
const QUIET_ON_MS: u64 = 1_772_366_400_000;
const QUIET_GUID: u64 = 900_000_000_001;
const QUIET_KEY: u64 = 775_001;

/// The Pumpkin Prince (Blackspore, Super Spooktacular) and its two ghosts.
const PUMPKINS: [u64; 3] = [100_000_031_334, 100_000_031_480, 100_000_031_482];
const PUMPKINS_AT: [f32; 3] = [-2393.0, -33.08, 1155.0];
/// The winter-holiday tree presents and the two Snowman Invaders in Snowhill.
const SNOW: [u64; 5] = [
    100_000_031_425,
    100_000_031_427,
    100_000_031_428,
    100_000_031_443,
    100_000_031_445,
];
const SNOW_AT: [f32; 3] = [190.0, 23.0, 385.0];
/// The five Festival of Hearts critters in Merry Vale.
const HEARTS: [u64; 5] = [
    100_000_031_455,
    100_000_031_456,
    100_000_031_457,
    100_000_031_459,
    100_000_031_460,
];
const HEARTS_AT: [f32; 3] = [-1122.0, -39.93, -407.5];
/// The first-anniversary fireworks wagon.
const WAGON: [u64; 1] = [100_000_004_059];
const WAGON_AT: [f32; 3] = [-490.266, 9.608_817, 775.985];

/// Every shipped seasonal row and its season; the content gate below pins this table.
const SEASONAL: [(u64, &str); 14] = [
    (100_000_004_059, "Birthday Bash"),
    (100_000_031_334, "Halloween"),
    (100_000_031_425, "Snow Days"),
    (100_000_031_427, "Snow Days"),
    (100_000_031_428, "Snow Days"),
    (100_000_031_443, "Snow Days"),
    (100_000_031_445, "Snow Days"),
    (100_000_031_455, "Festival of Hearts"),
    (100_000_031_456, "Festival of Hearts"),
    (100_000_031_457, "Festival of Hearts"),
    (100_000_031_459, "Festival of Hearts"),
    (100_000_031_460, "Festival of Hearts"),
    (100_000_031_480, "Halloween"),
    (100_000_031_482, "Halloween"),
];

/// The 35/2 AddNpc bodies in `packets`, as actor ids (reader 0x00921a10).
fn adds(packets: &[Vec<u8>]) -> BTreeSet<u64> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .filter_map(|packet| parse_add_npc(packet).ok())
        .map(|npc| npc.actor_id)
        .collect()
}

/// The 35/3 removal bodies in `packets`, as actor ids (guid at offset 6).
fn removals(packets: &[Vec<u8>]) -> BTreeSet<u64> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 3, 0]))
        .filter_map(|packet| Some(u64::from_le_bytes(packet.get(6..14)?.try_into().ok()?)))
        .collect()
}

/// The actors a fresh per-session view sends from `position`.
fn seen(visibility: &NpcVisibility, position: [f32; 3]) -> BTreeSet<u64> {
    seen_from(visibility, position)
        .into_iter()
        .map(|npc| npc.actor_id)
        .collect()
}

fn load(unix_ms: u64) -> NpcVisibility {
    NpcVisibility::load_in_season_at(&content_dir(), unix_ms).unwrap()
}

#[test]
fn the_shipped_rows_tag_every_event_npc_with_a_defined_season() {
    let doc = json::read(&content_dir().join("npc_spawns_extra.json"));
    let shipped: std::collections::BTreeMap<u64, String> = doc["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| {
            Some((
                row["spawn_guid"].as_u64()?,
                row["season"].as_str()?.to_owned(),
            ))
        })
        .collect();
    let expected: std::collections::BTreeMap<u64, String> = SEASONAL
        .iter()
        .map(|(guid, season)| (*guid, (*season).to_owned()))
        .collect();
    assert_eq!(
        shipped, expected,
        "the seasonal rows are the Legacy event block; an untagged row spawns all year"
    );
    let areas = ClientAreas::load(&content_dir()).unwrap();
    for season in shipped.values() {
        assert!(
            areas.season_state(season, 0).is_some(),
            "{season} is not a client_areas.json season"
        );
    }
}

#[test]
fn outside_every_season_its_npcs_are_absent() {
    let visibility = load(NO_SEASON_MS);
    let seasonal: BTreeSet<u64> = SEASONAL.iter().map(|(guid, _)| *guid).collect();
    for at in [PUMPKINS_AT, SNOW_AT, HEARTS_AT, WAGON_AT] {
        assert!(
            seen(&visibility, at).is_disjoint(&seasonal),
            "a seasonal NPC is drawn at {at:?} outside every season"
        );
    }
}

/// Each group is drawn only at its own season's clock, never at another's.
#[test]
fn inside_its_season_its_npcs_appear_and_other_seasons_stay_off() {
    let groups: [(&str, &[u64], [f32; 3]); 4] = [
        ("Halloween", &PUMPKINS, PUMPKINS_AT),
        ("Snow Days", &SNOW, SNOW_AT),
        ("Festival of Hearts", &HEARTS, HEARTS_AT),
        ("Birthday Bash", &WAGON, WAGON_AT),
    ];
    for (clock, season) in [
        (HALLOWEEN_MS, "Halloween"),
        (SNOW_DAYS_MS, "Snow Days"),
        (FESTIVAL_MS, "Festival of Hearts"),
        (BIRTHDAY_MS, "Birthday Bash"),
    ] {
        let visibility = load(clock);
        for (name, guids, at) in groups {
            let got = seen(&visibility, at);
            let drawn = guids.iter().all(|guid| got.contains(guid));
            assert_eq!(
                drawn,
                name == season,
                "{name} at the {season} clock {clock}: brought {got:?}"
            );
        }
    }
}

/// The live transition: the same table re-diffs when the season ends and starts again, so a
/// nearby shared viewer gets the 35/3 removals (and later the 35/2 additions) with no reload.
#[test]
fn a_season_ending_removes_them_for_a_nearby_shared_viewer() {
    let pumpkins: BTreeSet<u64> = PUMPKINS.into_iter().collect();
    let mut visibility = load(HALLOWEEN_MS);
    let (wanted, packets) = visibility
        .shared_view(0, PUMPKINS_AT, &[], &BTreeSet::new())
        .unwrap();
    assert!(pumpkins.is_subset(&wanted), "the pumpkin rows are nearby");
    assert!(
        pumpkins.is_subset(&adds(&packets)),
        "the first view adds them"
    );
    assert_eq!(visibility.model_of(PUMPKINS[0]), Some(1702));
    assert!(visibility.actor_position(PUMPKINS[0]).is_some());

    visibility.set_season_clock(Some(HALLOWEEN_OVER_MS));
    let (after, packets) = visibility
        .shared_view(0, PUMPKINS_AT, &[], &wanted)
        .unwrap();
    let expected: BTreeSet<u64> = wanted.difference(&pumpkins).copied().collect();
    assert_eq!(
        after, expected,
        "only the seasonal rows leave; every untagged neighbour stays"
    );
    assert_eq!(
        removals(&packets),
        pumpkins,
        "the nearby viewer is told to remove exactly the seasonal actors"
    );
    assert!(adds(&packets).is_empty(), "{packets:?}");
    assert_eq!(visibility.model_of(PUMPKINS[0]), None);
    assert_eq!(visibility.actor_position(PUMPKINS[0]), None);

    // A client that connects after the season sees the same absence without a reload.
    let reconnected = load(HALLOWEEN_OVER_MS);
    assert!(seen(&reconnected, PUMPKINS_AT).is_disjoint(&pumpkins));

    // The next Halloween brings them back through the same table and the same diff.
    visibility.set_season_clock(Some(HALLOWEEN_MS));
    let (again, packets) = visibility.shared_view(0, PUMPKINS_AT, &[], &after).unwrap();
    assert!(pumpkins.is_subset(&again));
    assert_eq!(
        adds(&packets),
        pumpkins,
        "the returning season re-adds exactly them"
    );
}

/// A refused interaction: an out-of-season row keeps no reach and no content key, so the click
/// path cannot open its menu. The fixture has one season-tagged, content-keyed extra row.
#[test]
fn an_out_of_season_actor_has_no_position_model_or_content_key() {
    let dir = quiet_package("gate");
    let inside = NpcVisibility::load_in_season_at(&dir, QUIET_ON_MS).unwrap();
    assert_eq!(inside.model_of(QUIET_GUID), Some(7));
    assert_eq!(inside.content_key(QUIET_GUID), Some(QUIET_KEY));
    assert_eq!(inside.actor_position(QUIET_GUID), Some([1.0, 0.0, 1.0]));
    let wanted = BTreeSet::from([QUIET_GUID]);
    assert_eq!(
        inside.marker_targets(&wanted).collect::<Vec<_>>(),
        vec![(QUIET_GUID, QUIET_KEY, 0)],
        "a marker follows the same gate"
    );
    assert_eq!(inside.teleport_targets(0).len(), 1);
    assert_eq!(inside.model_actors().count(), 1);

    let outside = NpcVisibility::load_in_season_at(&dir, QUIET_OFF_MS).unwrap();
    assert_eq!(outside.model_of(QUIET_GUID), None);
    assert_eq!(
        outside.content_key(QUIET_GUID),
        None,
        "an out-of-season actor keeps no interaction"
    );
    assert_eq!(outside.actor_position(QUIET_GUID), None);
    assert!(outside.marker_targets(&wanted).next().is_none());
    assert!(outside.teleport_targets(0).is_empty());
    assert_eq!(outside.model_actors().count(), 0);
    let _ = std::fs::remove_dir_all(&dir);
}

fn quiet_package(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("osfr-t775-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("npcs")).unwrap();
    let write = |name: &str, value: serde_json::Value| {
        std::fs::write(dir.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    write(
        "zones.json",
        json!({"records": [{"id": 1, "name": "open"}]}),
    );
    write(
        "npcs/anchor.json",
        json!({"records": [{"npc_id": 1, "position": [0.0, 0.0, 0.0]}]}),
    );
    write(
        "npc_spawns_extra.json",
        json!({"schema_version": 1, "records": [{
            "spawn_guid": QUIET_GUID, "model_id": 7, "position": [1.0, 0.0, 1.0],
            "season": "Quiet", "content_key": QUIET_KEY,
        }]}),
    );
    write(
        "client_areas.json",
        json!({
            "schema_version": 1,
            "switchable": {"area_ids": [], "source": "test"},
            "seasons": [{"name": "Quiet", "start_month": 3, "start_day": 1,
                "end_month": 3, "end_day": 2, "area_ids": [], "source": "test"}],
        }),
    );
    dir
}
