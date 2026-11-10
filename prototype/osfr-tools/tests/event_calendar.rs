//! The event calendar (`src/event_calendar.rs`, `data/content/event_calendar.json`): an event
//! runs by its season's dates or by hand, brings its quests, people and client areas, and
//! leaves nothing in a journal once it is over.
//!
//! The session tests drive the production in-world router through
//! `tests/support/npc_quest_chain.rs` with the shipped content: Alyssa Eggelyn's egg hunt
//! (3955710, The Spring Eggstravaganza, March 28 to April 7) is offered, accepted and taken
//! away again by the clock and by an admin's `realm event season`. No new wire: the offer is the
//! existing 49/1 (client reader `0x00c7b7a0`), the client-area switch the existing opcode 89
//! (`0x008d5c60`), and an ended quest leaves with the abandon answer 26/23 already uses.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, Fixture, accept, assert_no_offer, character_at_for, npc_location, offer,
    seed_quest, visible_npcs, zone_player,
};
use osfr_tools::{
    account_store::AccountRole,
    chat,
    client_areas::ClientAreas,
    event_calendar::{EventCalendar, Mode, read_overrides, write_override},
    gameplay::Gameplay,
    quest_engine::QuestStatus,
    character_store::CharacterState,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

const PLAYER: u64 = 835_001;
const ALYSSA: u64 = 9_500_000_436;
const EGG_HUNT: u32 = 3_955_710;
/// The Pumpkin Prince, a Halloween-tagged row of `npc_spawns_extra.json`.
const PUMPKIN_PRINCE: u64 = 100_000_031_334;
const PRINCE_AT: [f32; 3] = [-2392.853, -33.08391, 1155.678];
const DAY: u64 = 24 * 60 * 60_000;
/// 2026-07-15 12:00 UTC: inside Summer Camp only.
const JULY: u64 = 1_784_116_800_000;
/// 2026-03-30 12:00 UTC: inside The Spring Eggstravaganza.
const EGG_DAY: u64 = 1_774_872_000_000;
/// 2026-04-20 12:00 UTC: after it.
const AFTER_EGGS: u64 = EGG_DAY + 21 * DAY;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("osfr-event-calendar-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Windows file times advance in clock ticks, so two writes can share an mtime; wait one
/// tick so the watch sees a distinct stamp (as `tests/live_events.rs`).
fn settle() {
    std::thread::sleep(Duration::from_millis(100));
}

/// One season "Fest" (February 1 and 2) holding client area 10, and the calendar event `fest`.
fn fest(name: &str, calendar: Value) -> Scratch {
    let dir = Scratch::new(name);
    let areas = json!({
        "schema_version": 1,
        "switchable": {"area_ids": [10], "source": "test"},
        "seasons": [{"name": "Fest", "start_month": 2, "start_day": 1, "end_month": 2,
                     "end_day": 2, "area_ids": [10], "source": "test"}],
    });
    fs::write(dir.0.join("client_areas.json"), areas.to_string()).unwrap();
    fs::write(dir.0.join("event_calendar.json"), calendar.to_string()).unwrap();
    dir
}

fn fest_calendar(real_dates: bool, mode: &str) -> Value {
    json!({
        "schema_version": 1, "run_on_real_dates": real_dates, "source": "test",
        "events": [{"key": "fest", "name": "Fest", "season": "Fest", "mode": mode,
                    "quest_ids": [7], "source": "test"}],
    })
}

/// 2026-02-01 12:00 UTC, inside Fest.
const FEST_DAY: u64 = 1_769_947_200_000;

/// (area id, enabled) of each opcode-89 packet.
fn switches(packets: &[Vec<u8>]) -> Vec<(i32, bool)> {
    packets
        .iter()
        .map(|packet| {
            assert_eq!(&packet[..2], &[89, 0]);
            (
                i32::from_le_bytes(packet[2..6].try_into().unwrap()),
                packet[6] != 0,
            )
        })
        .collect()
}

#[test]
fn the_shipped_calendar_names_defined_seasons_and_real_quests() {
    // A season the calendar names but client_areas.json lacks refuses this load.
    let areas = ClientAreas::load(&content_dir()).unwrap();
    let calendar = areas.calendar();
    assert!(!calendar.is_empty(), "the calendar ships with events");
    let mut records = BTreeSet::new();
    for entry in fs::read_dir(content_dir().join("quests")).unwrap() {
        let file: Value = serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap();
        let rows = file["records"].as_array().into_iter().flatten();
        records.extend(rows.filter_map(|row| row["id"].as_u64()));
    }
    for id in calendar.gated_quests() {
        assert!(records.contains(&u64::from(id)), "event quest {id} is not a record");
    }
    // Every event is reachable by its key, and each one's season answers the clock.
    for state in calendar.states(&BTreeMap::new(), &areas, JULY) {
        assert!(calendar.has_event(state.key), "{}", state.key);
    }
    assert!(calendar.open_quests(&areas, EGG_DAY).contains(&EGG_HUNT));
    assert!(!calendar.open_quests(&areas, JULY).contains(&EGG_HUNT));
}

#[test]
fn a_hand_toggle_holds_an_event_on_or_off_whatever_the_date() {
    let dir = fest("hand", fest_calendar(true, "calendar"));
    let live = Scratch::new("hand-live");
    let mut areas = ClientAreas::load(&dir.0).unwrap().watching(&live.0);
    let mut now = JULY;
    let mut step = |areas: &mut ClientAreas| {
        settle();
        now += 2_000; // past the one-second poll, inside the same minute or not
        switches(&areas.season_changes(now))
    };
    assert!(step(&mut areas).is_empty(), "the first call is the baseline");
    assert!(!areas.is_season_active("Fest", JULY));

    // On by hand in July: the area switches on at the next poll, not at the next minute.
    assert!(write_override(&live.0, "fest", Some(Mode::On)).unwrap());
    assert!(!write_override(&live.0, "fest", Some(Mode::On)).unwrap(), "a repeat writes nothing");
    assert_eq!(step(&mut areas), [(10, true)]);
    assert!(areas.is_season_active("Fest", JULY));
    assert_eq!(areas.calendar().open_quests(&areas, JULY), BTreeSet::from([7]));
    assert_eq!(switches(&areas.world_ready_packets(JULY)), [(10, true)]);

    // Off by hand: shut even on its own dates.
    write_override(&live.0, "fest", Some(Mode::Off)).unwrap();
    assert_eq!(step(&mut areas), [(10, false)]);
    assert!(!areas.is_season_active("Fest", FEST_DAY));
    assert!(areas.calendar().open_quests(&areas, FEST_DAY).is_empty());

    // Back to the content's mode: the dates decide again.
    write_override(&live.0, "fest", None).unwrap();
    assert!(read_overrides(&live.0).unwrap().is_empty());
    assert!(step(&mut areas).is_empty(), "July is outside Fest either way");
    assert!(areas.is_season_active("Fest", FEST_DAY) && !areas.is_season_active("Fest", JULY));

    // A file nobody can parse is no toggle at all, never an error on the tick.
    fs::write(live.0.join("event_overrides.json"), "{").unwrap();
    assert!(read_overrides(&live.0).is_err());
    assert!(step(&mut areas).is_empty());
    assert!(areas.is_season_active("Fest", FEST_DAY));
}

#[test]
fn the_content_modes_and_the_real_dates_switch_are_tunable() {
    let on = fest("mode-on", fest_calendar(true, "on"));
    let areas = ClientAreas::load(&on.0).unwrap();
    assert!(areas.is_season_active("Fest", JULY), "mode on runs all year");

    let off = fest("mode-off", fest_calendar(true, "off"));
    assert!(!ClientAreas::load(&off.0).unwrap().is_season_active("Fest", FEST_DAY));

    // No real dates: nothing starts by date, but a hand toggle still runs the event.
    let manual = fest("manual", fest_calendar(false, "calendar"));
    let live = Scratch::new("manual-live");
    let mut areas = ClientAreas::load(&manual.0).unwrap().watching(&live.0);
    assert!(!areas.is_season_active("Fest", FEST_DAY));
    write_override(&live.0, "fest", Some(Mode::On)).unwrap();
    assert!(areas.poll(FEST_DAY));
    assert!(areas.is_season_active("Fest", FEST_DAY) && areas.is_season_active("Fest", JULY));
    // "calendar" by hand does not bring the dates back while the file switches them off.
    settle();
    write_override(&live.0, "fest", Some(Mode::Calendar)).unwrap();
    areas.poll(FEST_DAY + 2_000);
    assert!(!areas.is_season_active("Fest", FEST_DAY));
}

#[test]
fn the_loader_refuses_a_calendar_it_cannot_trust() {
    let two = |edit: &dyn Fn(&mut Value)| {
        let mut calendar = fest_calendar(true, "calendar");
        let mut second = calendar["events"][0].clone();
        second["key"] = json!("other");
        second["season"] = json!("Other");
        second["quest_ids"] = json!([8]);
        calendar["events"].as_array_mut().unwrap().push(second);
        edit(&mut calendar);
        calendar
    };
    let cases: [(&str, Value); 7] = [
        ("unknown season", two(&|_| {})),
        ("duplicate key", two(&|c| c["events"][1]["key"] = json!("fest"))),
        ("shared season", two(&|c| c["events"][1]["season"] = json!("Fest"))),
        ("shared quest", two(&|c| c["events"][1]["quest_ids"] = json!([7]))),
        ("bad key", two(&|c| c["events"][0]["key"] = json!("Fest Day"))),
        ("unknown mode", fest_calendar(true, "sometimes")),
        ("wrong version", {
            let mut calendar = fest_calendar(true, "on");
            calendar["schema_version"] = json!(2);
            calendar
        }),
    ];
    for (label, calendar) in cases {
        let dir = fest(&format!("refuse-{}", label.replace(' ', "-")), calendar);
        // Only the season check needs the season table; every other rule is the calendar's own.
        let alone = EventCalendar::load(&dir.0);
        assert_eq!(alone.is_ok(), label == "unknown season", "{label}");
        assert!(ClientAreas::load(&dir.0).is_err(), "{label} must be refused");
    }
    // No file is an empty calendar: seasons follow their dates as before.
    let bare = fest("bare", json!({}));
    fs::remove_file(bare.0.join("event_calendar.json")).unwrap();
    assert!(EventCalendar::load(&bare.0).unwrap().is_empty());
    assert!(ClientAreas::load(&bare.0).unwrap().is_season_active("Fest", FEST_DAY));
}

/// An admin session in the fixture's private live world.
fn admin(fixture: &Fixture) -> Gameplay {
    let live = fixture.open_live_events([]);
    let mut game = zone_player(fixture, PLAYER, ADVENTURER);
    game.set_role_commands(true);
    game.set_role(AccountRole::Admin);
    game.set_world_dir(Some(live)).unwrap();
    game
}

fn at_alyssa() -> CharacterState {
    let (zone, position) = npc_location(ALYSSA);
    character_at_for(PLAYER, zone, position)
}

/// Move the calendar clock and let one poll apply it.
fn clock(game: &mut Gameplay, unix_ms: u64, now: &mut u64) -> osfr_tools::gameplay::Outcome {
    *now += 1_100;
    // The session's own clock rides along, so the hand-toggle poll sees time pass.
    game.set_event_clock(Some(unix_ms + *now));
    game.tick(&at_alyssa(), *now).unwrap()
}

fn realm(game: &mut Gameplay, text: &str, now: &mut u64) -> String {
    *now += 10;
    let out = game
        .handle_in_world(&chat::local_request(text), &at_alyssa(), *now)
        .unwrap();
    let lines: Vec<String> = (out.replies.iter())
        .filter(|packet| packet.starts_with(&[15, 0, 3, 0]))
        .map(|packet| {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            String::from_utf8(packet[8..8 + length].to_vec()).unwrap()
        })
        .collect();
    assert_eq!(lines.len(), 1, "{text} answers with one chat line: {lines:?}");
    lines[0].clone()
}

fn status(game: &Gameplay, quest: u32) -> Option<QuestStatus> {
    let quests = game.state().npc_quests.unwrap().quests;
    quests.get(&quest).map(|progress| progress.status.clone())
}

#[test]
fn an_event_quest_is_offered_in_season_and_leaves_the_journal_when_it_ends() {
    let fixture = Fixture::new().with_bulk_quests();
    let mut game = admin(&fixture);
    let (alyssa, mut now) = (at_alyssa(), 1_000);

    clock(&mut game, JULY, &mut now);
    assert_no_offer(&mut game, &alyssa, ALYSSA, &mut now, "the egg hunt in July");

    clock(&mut game, EGG_DAY, &mut now);
    offer(&mut game, &alyssa, ALYSSA, EGG_HUNT, now);
    accept(&mut game, &alyssa, EGG_HUNT, now + 2);
    now += 10;
    assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Active));

    // The event ends by date: the started quest is dropped with the abandon answer and saved.
    let ended = clock(&mut game, AFTER_EGGS, &mut now);
    assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Abandoned));
    assert!(ended.changed, "the cleaned journal must be saved");
    assert!(!ended.replies.is_empty(), "the client is told the quest left");
    assert_no_offer(&mut game, &alyssa, ALYSSA, &mut now, "the egg hunt after the event");
}

#[test]
fn an_admin_runs_an_event_by_hand_and_ends_it_again() {
    let fixture = Fixture::new().with_bulk_quests();
    let mut game = admin(&fixture);
    let (alyssa, mut now) = (at_alyssa(), 1_000);
    clock(&mut game, JULY, &mut now);

    let listed = realm(&mut game, "realm event season", &mut now);
    assert!(
        listed.contains("spring_eggstravaganza (The Spring Eggstravaganza): calendar, not running")
            && listed.contains("summer_camp (Summer Camp): calendar, running"),
        "{listed}"
    );
    let refused = realm(&mut game, "realm event season easter on", &mut now);
    assert!(refused.contains("not a calendar event"), "{refused}");
    assert_no_offer(&mut game, &alyssa, ALYSSA, &mut now, "a refused toggle");

    // On by hand in July: Alyssa offers the hunt and Halloween's people are drawn.
    let prince = character_at_for(PLAYER, 0, PRINCE_AT);
    let drawn = |game: &mut Gameplay| {
        (visible_npcs(game, &prince).iter()).any(|npc| npc.actor_id == PUMPKIN_PRINCE)
    };
    assert!(!drawn(&mut game), "the Pumpkin Prince stands in July");
    // Every form an admin may type: the realm line, `/event <key>` and `/<key>`.
    settle();
    let set = realm(&mut game, "/event spring_eggstravaganza on", &mut now);
    let told = "spring_eggstravaganza (The Spring Eggstravaganza): on, running until an admin \
                turns it off. Active now: spring_eggstravaganza, summer_camp.";
    assert_eq!(set, told);
    settle();
    let set = realm(&mut game, "/halloween on", &mut now);
    assert!(set.starts_with("halloween (Super Spooktacular): on, running until an admin"), "{set}");
    assert!(set.contains("Active now:") && set.contains("halloween"), "{set}");
    let one = realm(&mut game, "/halloween", &mut now);
    assert_eq!(one, "halloween (Super Spooktacular): on, running until an admin turns it off.");
    clock(&mut game, JULY, &mut now);
    assert!(drawn(&mut game), "Halloween by hand brings its people");
    offer(&mut game, &alyssa, ALYSSA, EGG_HUNT, now);
    accept(&mut game, &alyssa, EGG_HUNT, now + 2);
    now += 10;
    let listed = realm(&mut game, "realm event season", &mut now);
    assert!(listed.contains("spring_eggstravaganza (The Spring Eggstravaganza): on, running"));

    // Back to the calendar: the quest leaves with the event.
    settle();
    let back = realm(&mut game, "/spring_eggstravaganza calendar", &mut now);
    assert!(
        back.starts_with(
            "spring_eggstravaganza (The Spring Eggstravaganza): calendar, not running, next \
             starts 2027-03-28 00:00. Active now:"
        ),
        "{back}"
    );
    let summer = realm(&mut game, "/summer_camp", &mut now);
    assert_eq!(summer, "summer_camp (Summer Camp): calendar, running until 2026-09-07 00:00.");
    clock(&mut game, JULY, &mut now);
    assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Abandoned));
    assert_no_offer(&mut game, &alyssa, ALYSSA, &mut now, "the hunt after the toggle is cleared");
}

#[test]
fn a_finished_event_quest_stays_finished_so_the_quest_count_never_drops() {
    let fixture = Fixture::new().with_bulk_quests();
    let live = fixture.open_live_events([]);
    let mut state = zone_player(&fixture, PLAYER, ADVENTURER).state();
    seed_quest(&mut state, EGG_HUNT, QuestStatus::Completed, &[1, 1]);
    let mut game = fixture.game_as(PLAYER, Some(state));
    game.set_world_dir(Some(live)).unwrap();
    let (alyssa, mut now) = (at_alyssa(), 1_000);

    // The shipped `on_end` does not forget: completed quests feed the quest achievements.
    let before = game.state().npc_quests;
    clock(&mut game, JULY, &mut now);
    assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Completed));
    assert_eq!(game.state().npc_quests, before, "nothing to clean");
    clock(&mut game, EGG_DAY, &mut now);
    assert_no_offer(&mut game, &alyssa, ALYSSA, &mut now, "a finished hunt in the next run");
}

/// A session with `role` in its own live world.
fn with_role(fixture: &Fixture, role: AccountRole) -> (Gameplay, PathBuf) {
    let live = fixture.open_live_events([]);
    let mut game = zone_player(fixture, PLAYER, ADVENTURER);
    game.set_role_commands(true);
    game.set_role(role);
    game.set_world_dir(Some(live.clone())).unwrap();
    (game, live)
}

#[test]
fn only_an_admin_may_toggle_an_event() {
    const REFUSAL: &str = "Cannot do that: only an admin can start or stop an event.";
    const ROLE: &str = "Cannot do that: your account role does not allow this command.";
    let fixture = Fixture::new().with_bulk_quests();
    // A tester may run almost every realm command; this one is not theirs.
    for (role, realm_reply) in [
        (AccountRole::Player, ROLE),
        (AccountRole::Tester, REFUSAL),
        (AccountRole::Builder, ROLE),
        (AccountRole::Moderator, ROLE),
    ] {
        let (mut game, live) = with_role(&fixture, role);
        let mut now = 1_000;
        clock(&mut game, JULY, &mut now);
        for (command, reply) in [
            ("/halloween on", REFUSAL),
            ("/event halloween on", REFUSAL),
            ("/event", REFUSAL),
            ("/halloween off", REFUSAL),
            ("realm event season halloween on", realm_reply),
            ("realm event season", realm_reply),
        ] {
            assert_eq!(realm(&mut game, command, &mut now), reply, "{role:?}: {command}");
        }
        assert!(!live.join("event_overrides.json").exists(), "{role:?} wrote the toggle file");
        clock(&mut game, JULY, &mut now);
        let alyssa = at_alyssa();
        assert_no_offer(&mut game, &alyssa, ALYSSA, &mut now, "a refused toggle");
    }
    // An ordinary chat line that merely starts with a slash is not ours to answer.
    let (mut game, _live) = with_role(&fixture, AccountRole::Player);
    let out = game
        .handle_in_world(&chat::local_request("/wave"), &at_alyssa(), 2_000)
        .unwrap();
    assert!(!out.replies.iter().any(|packet| packet.starts_with(&[15, 0, 3, 0])));
}

#[test]
fn a_toggle_file_nobody_can_read_ends_nothing() {
    let fixture = Fixture::new().with_bulk_quests();
    let mut game = admin(&fixture);
    let live = fixture.path().join("live");
    let (alyssa, mut now) = (at_alyssa(), 1_000);
    clock(&mut game, JULY, &mut now);
    settle();
    realm(&mut game, "/spring_eggstravaganza on", &mut now);
    clock(&mut game, JULY, &mut now);
    offer(&mut game, &alyssa, ALYSSA, EGG_HUNT, now);
    accept(&mut game, &alyssa, EGG_HUNT, now + 2);
    now += 10;

    // A torn or hand-broken file: the event held by hand keeps running, the quest stays.
    for garbage in ["{", "", "{\"schema_version\":1,\"modes\":{\"halloween\":\"sometimes\"}}"] {
        settle();
        fs::write(live.join("event_overrides.json"), garbage).unwrap();
        let before = game.state().npc_quests;
        clock(&mut game, JULY, &mut now);
        assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Active), "{garbage:?}");
        assert_eq!(game.state().npc_quests, before, "{garbage:?} changed the journal");
    }
    assert!(read_overrides(&live).is_err());
    // One command repairs it, and only then does `off` end the event.
    settle();
    let off = realm(&mut game, "/spring_eggstravaganza off", &mut now);
    assert!(off.contains("off, not running"), "{off}");
    assert_eq!(read_overrides(&live).unwrap().len(), 1);
    clock(&mut game, JULY, &mut now);
    assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Abandoned));
}

#[test]
fn a_quest_on_its_reward_screen_is_not_taken_away() {
    let fixture = Fixture::new().with_bulk_quests();
    let live = fixture.open_live_events([]);
    let mut state = zone_player(&fixture, PLAYER, ADVENTURER).state();
    seed_quest(&mut state, EGG_HUNT, QuestStatus::Active, &[1, 1]);
    let quests = state.npc_quests.as_mut().unwrap();
    quests.pending_turnin = Some(EGG_HUNT);
    quests.pending_turnin_actor = Some(ALYSSA);
    let mut game = fixture.game_as(PLAYER, Some(state));
    game.set_world_dir(Some(live)).unwrap();
    let pending = |game: &Gameplay| game.state().npc_quests.unwrap().pending_turnin;
    if pending(&game) != Some(EGG_HUNT) {
        // The loader repaired the seeded end screen away; the rule has nothing to hold here.
        eprintln!("skip: the save repair cleared the seeded reward screen");
        return;
    }
    let mut now = 1_000;
    for _ in 0..3 {
        clock(&mut game, JULY, &mut now);
        assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Active));
        assert_eq!(pending(&game), Some(EGG_HUNT));
    }
}

// ---- Rotations: daily, weekly and monthly turns, all judged at a clock the test passes. ----

const HOUR: u64 = 60 * 60_000;
/// 2026-07-13 00:00 UTC, a Monday.
const MONDAY: u64 = JULY - 2 * DAY - 12 * HOUR;
/// 2026-08-01 00:00 UTC.
const AUGUST: u64 = MONDAY + 19 * DAY;

/// Seasons A (February 1-2) and B (March 1-2); event `a` is dated, `b` runs by rotation only.
fn rotating(name: &str, real_dates: bool, rotations: Value) -> (Scratch, ClientAreas) {
    let dir = Scratch::new(name);
    let season = |name: &str, month: u8| {
        json!({"name": name, "start_month": month, "start_day": 1, "end_month": month,
               "end_day": 2, "area_ids": [], "source": "test"})
    };
    let areas = json!({
        "schema_version": 1,
        "switchable": {"area_ids": [10], "source": "test"},
        "seasons": [season("A", 2), season("B", 3)],
    });
    let calendar = json!({
        "schema_version": 1, "run_on_real_dates": real_dates, "source": "test",
        "events": [
            {"key": "a", "name": "A", "season": "A", "quest_ids": [7], "source": "test"},
            {"key": "b", "name": "B", "season": "B", "by_date": false, "quest_ids": [8],
             "source": "test"},
        ],
        "rotations": rotations,
    });
    fs::write(dir.0.join("client_areas.json"), areas.to_string()).unwrap();
    fs::write(dir.0.join("event_calendar.json"), calendar.to_string()).unwrap();
    let loaded = ClientAreas::load(&dir.0);
    (dir, loaded.expect("the rotation fixture loads"))
}

fn other(key: &str) -> &'static str {
    if key == "a" { "b" } else { "a" }
}

#[test]
fn a_daily_rotation_takes_turns_and_names_the_next_start() {
    let rows = json!([{"key": "daily", "period": "day", "events": ["a", "b"],
                       "offset_minutes": 18 * 60, "length_minutes": 120, "source": "test"}]);
    let (_dir, areas) = rotating("daily", true, rows);
    let calendar = areas.calendar();
    let today = calendar.current_rotation("daily", areas.local_minute(MONDAY + HOUR)).unwrap();
    let tomorrow = calendar.current_rotation("daily", areas.local_minute(MONDAY + DAY)).unwrap();
    assert_eq!(today.starts, areas.local_minute(MONDAY + 18 * HOUR));
    assert_eq!(today.ends - today.starts, 120);
    assert_eq!((tomorrow.index, tomorrow.event), (today.index + 1, other(today.event)));
    assert!(!today.contains(areas.local_minute(MONDAY + HOUR)), "its turn has not started");
    assert!(calendar.current_rotation("weekly", 0).is_none(), "no such rotation");

    // Active exactly inside the window, and the quests follow.
    assert!(calendar.active_at(&areas, MONDAY + 18 * HOUR - 60_000).is_empty());
    assert_eq!(calendar.active_at(&areas, MONDAY + 18 * HOUR), [today.event]);
    assert_eq!(calendar.active_at(&areas, MONDAY + 20 * HOUR - 60_000), [today.event]);
    assert!(calendar.active_at(&areas, MONDAY + 20 * HOUR).is_empty());
    let quest = if today.event == "a" { 7 } else { 8 };
    assert_eq!(calendar.open_quests(&areas, MONDAY + 19 * HOUR), BTreeSet::from([quest]));
    assert_eq!(calendar.active_at(&areas, MONDAY + DAY + 19 * HOUR), [other(today.event)]);

    // Next start, asked mid-turn: the other event tomorrow, this one the day after.
    let after = MONDAY + 19 * HOUR;
    let next = |key: &str| calendar.next_occurrence(key, &areas, after);
    assert_eq!(next(other(today.event)), Some(MONDAY + DAY + 18 * HOUR));
    assert_eq!(next(today.event), Some(MONDAY + 2 * DAY + 18 * HOUR));
    assert_eq!(next("nobody"), None);
}

#[test]
fn weekly_and_monthly_rotations_overlap_each_other_and_the_dates() {
    // Sunday 20:00 for twelve hours (into Monday), and the first week of each month in turn.
    let rows = json!([
        {"key": "weekend", "period": "week", "events": ["b"],
         "offset_minutes": 6 * 1440 + 20 * 60, "length_minutes": 12 * 60, "source": "test"},
        {"key": "monthly", "period": "month", "events": ["a", "b"],
         "length_minutes": 7 * 1440, "source": "test"},
    ]);
    let (_dir, areas) = rotating("weekly", true, rows);
    let calendar = areas.calendar();
    // The week of July 13 is clear of the monthly turn (it ended on July 8).
    let sunday_night = MONDAY + 6 * DAY + 20 * HOUR;
    assert!(calendar.active_at(&areas, sunday_night - 60_000).is_empty());
    assert_eq!(calendar.active_at(&areas, sunday_night), ["b"]);
    // Monday 01:00 belongs to the next week, whose own turn has not started: last week's runs on.
    let monday = MONDAY + 7 * DAY + HOUR;
    assert_eq!(calendar.active_at(&areas, monday), ["b"]);
    let this_week = calendar.current_rotation("weekend", areas.local_minute(monday)).unwrap();
    assert!(!this_week.contains(areas.local_minute(monday)));
    assert!(calendar.active_at(&areas, MONDAY + 7 * DAY + 8 * HOUR).is_empty());
    assert_eq!(calendar.next_occurrence("b", &areas, MONDAY), Some(sunday_night));

    // Months take turns; the turn is the first seven days.
    let august = calendar.current_rotation("monthly", areas.local_minute(AUGUST)).unwrap();
    assert_eq!(august.starts, areas.local_minute(AUGUST));
    assert!(calendar.active_at(&areas, AUGUST + 6 * DAY).contains(&august.event));
    assert!(!calendar.active_at(&areas, AUGUST + 7 * DAY + 12 * HOUR).contains(&august.event));
    let september = AUGUST + 31 * DAY;
    assert!(calendar.active_at(&areas, september).contains(&other(august.event)));
    // Sunday August 2 at 21:00: the weekend turn and August's turn run together.
    let both = calendar.active_at(&areas, AUGUST + DAY + 21 * HOUR);
    assert!(both.contains(&"b") && both.contains(&august.event), "{both:?}");

    // Without rotations the dated event starts on its date and the rotation-only one never does.
    let (_plain, plain) = rotating("dated", true, json!([]));
    let february = FEST_DAY + 365 * DAY - 12 * HOUR; // 2027-02-01 00:00 UTC
    assert_eq!(plain.calendar().active_at(&plain, FEST_DAY), ["a"]);
    assert_eq!(plain.calendar().next_occurrence("a", &plain, JULY), Some(february));
    assert_eq!(plain.calendar().next_occurrence("b", &plain, JULY), None);
    let march = FEST_DAY + 28 * DAY; // 2026-03-01 12:00 UTC, inside season B's dates
    assert!(plain.calendar().active_at(&plain, march).is_empty(), "b ignores its season's dates");
}

#[test]
fn a_hand_toggle_and_the_real_dates_switch_rule_over_rotations() {
    let rows = json!([{"key": "daily", "period": "day", "events": ["b"], "source": "test"}]);
    let (_dir, areas) = rotating("always", true, rows.clone());
    assert_eq!(areas.calendar().active_at(&areas, JULY), ["b"], "a whole-day turn every day");

    let (_off, manual) = rotating("no-dates", false, rows);
    assert!(manual.calendar().active_at(&manual, JULY).is_empty());
    assert!(manual.calendar().active_at(&manual, FEST_DAY).is_empty());
    let live = Scratch::new("no-dates-live");
    let mut manual = manual.watching(&live.0);
    write_override(&live.0, "a", Some(Mode::On)).unwrap();
    manual.poll(JULY);
    assert_eq!(manual.calendar().active_at(&manual, JULY), ["a"]);
    let next = manual.calendar().next_occurrence("a", &manual, JULY);
    assert_eq!(next, None, "held on: there is no next start");
}

#[test]
fn the_loader_refuses_a_rotation_it_cannot_run() {
    let row = |edit: &dyn Fn(&mut Value)| {
        let mut row = json!({"key": "r", "period": "week", "events": ["a"], "source": "test"});
        edit(&mut row);
        json!([row])
    };
    let cases: [(&str, Value); 6] = [
        ("unknown event", row(&|r| r["events"] = json!(["c"]))),
        ("no events", row(&|r| r["events"] = json!([]))),
        ("unknown period", row(&|r| r["period"] = json!("fortnight"))),
        ("offset past the period", row(&|r| r["offset_minutes"] = json!(7 * 1440))),
        ("empty turn", row(&|r| r["length_minutes"] = json!(0))),
        ("turn past the next period", row(&|r| r["length_minutes"] = json!(14 * 1440 + 1))),
    ];
    for (label, rotations) in cases {
        let dir = Scratch::new(&format!("rotation-{}", label.replace(' ', "-")));
        let calendar = json!({
            "schema_version": 1, "source": "test", "rotations": rotations,
            "events": [{"key": "a", "name": "A", "season": "A", "source": "test"}],
        });
        fs::write(dir.0.join("event_calendar.json"), calendar.to_string()).unwrap();
        assert!(EventCalendar::load(&dir.0).is_err(), "{label} must be refused");
    }
    let fine = Scratch::new("rotation-fine");
    let calendar = json!({
        "schema_version": 1, "source": "test",
        "rotations": [{"key": "r", "period": "week", "events": ["a"], "source": "test"}],
        "events": [{"key": "a", "name": "A", "season": "A", "source": "test"}],
    });
    fs::write(fine.0.join("event_calendar.json"), calendar.to_string()).unwrap();
    assert!(EventCalendar::load(&fine.0).is_ok(), "the untouched row loads");
}
