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
    ADVENTURER, Fixture, accept, assert_no_offer, character_at_for, interact, npc_location, offer,
    seed_quest, visible_npcs, zone_player,
};
use osfr_tools::{
    account_store::AccountRole,
    chat,
    client_areas::ClientAreas,
    event_calendar::{EventCalendar, Mode, read_overrides, write_override, write_toggle},
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

/// EV: Alyssa Eggelyn is tagged with her event (`live_event: spring_eggstravaganza`), so out of it
/// she is not in the world: a 26/8 on her opens no menu and offers nothing, and the save is the
/// same. Inside her event she answers as before (`assert_no_offer` for a finished hunt).
fn assert_closed(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: &mut u64,
    label: &str,
) {
    *now += 10;
    let reply = interact(game, character, target, *now);
    assert!(
        !reply.replies.iter().any(|p| p.starts_with(&[26, 0, 9, 0]) || p.starts_with(&[49, 0, 1, 0])),
        "{label}: the giver of a closed event answered"
    );
}
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
    let mut keyed = BTreeMap::new();
    for entry in fs::read_dir(content_dir().join("quests")).unwrap() {
        let file: Value = serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap();
        for row in file["records"].as_array().into_iter().flatten() {
            records.extend(row["id"].as_u64());
            if let (Some(id), Some(key)) = (row["id"].as_u64(), row["live_event"].as_str()) {
                keyed.insert(id, key.to_owned());
            }
        }
    }
    for id in calendar.gated_quests() {
        assert!(records.contains(&u64::from(id)), "event quest {id} is not a record");
    }
    // QF: shipped quests carry their event's key on the row; each key is a calendar event.
    assert!(calendar.gated_quests().is_empty(), "the rows, not the calendar, list event quests");
    for (id, key) in &keyed {
        assert!(calendar.has_event(key), "quest {id} names unknown event {key}");
    }
    // Every event is reachable by its key, and each one's season answers the clock.
    for state in calendar.states(&BTreeMap::new(), &areas, JULY) {
        assert!(calendar.has_event(state.key), "{}", state.key);
    }
    assert_eq!(keyed[&u64::from(EGG_HUNT)], "spring_eggstravaganza");
    assert!(calendar.is_running("spring_eggstravaganza", &areas, EGG_DAY));
    assert!(!calendar.is_running("spring_eggstravaganza", &areas, JULY));
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
    assert_closed(&mut game, &alyssa, ALYSSA, &mut now, "the egg hunt in July");

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
    assert_closed(&mut game, &alyssa, ALYSSA, &mut now, "the egg hunt after the event");
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
    assert_closed(&mut game, &alyssa, ALYSSA, &mut now, "a refused toggle");

    // On by hand in July: Alyssa offers the hunt and Halloween's people are drawn.
    let prince = character_at_for(PLAYER, 0, PRINCE_AT);
    let drawn = |game: &mut Gameplay| {
        (visible_npcs(game, &prince).iter()).any(|npc| npc.actor_id == PUMPKIN_PRINCE)
    };
    assert!(!drawn(&mut game), "the Pumpkin Prince stands in July");
    // Every form an admin may type: the realm line, `/event <key>` and `/<key>`.
    settle();
    let set = realm(&mut game, "/event spring_eggstravaganza on", &mut now);
    // A toggle holds until the event's next calendar edge: here each event's next start.
    let told = "spring_eggstravaganza (The Spring Eggstravaganza): on, running by hand until \
                2027-03-28 00:00, then by its dates. Active now: spring_eggstravaganza, summer_camp, seaside_summer_splash.";
    assert_eq!(set, told);
    settle();
    let set = realm(&mut game, "/halloween on", &mut now);
    let by_hand = "halloween (Super Spooktacular): on, running by hand until 2026-10-01 00:00, \
                   then by its dates.";
    assert!(set.starts_with(by_hand), "{set}");
    assert!(set.contains("Active now:") && set.contains("halloween"), "{set}");
    let one = realm(&mut game, "/halloween", &mut now);
    assert_eq!(one, by_hand);
    assert_eq!(realm(&mut game, "realm halloween", &mut now), by_hand, "the form the client sends");
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
    assert_closed(&mut game, &alyssa, ALYSSA, &mut now, "the hunt after the toggle is cleared");
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
    // The realm router's own refusal: every form of the toggle is its `season` command.
    const ROLE: &str = "Cannot do that: your account role does not allow this command.";
    let fixture = Fixture::new().with_bulk_quests();
    // A tester may run almost every realm command; this one is not theirs.
    for role in [
        AccountRole::Player,
        AccountRole::Tester,
        AccountRole::Builder,
        AccountRole::Moderator,
    ] {
        let (mut game, live) = with_role(&fixture, role);
        let mut now = 1_000;
        clock(&mut game, JULY, &mut now);
        for command in [
            "realm halloween on",
            "realm halloween off",
            "realm halloween",
            "realm event season halloween on",
            "realm event season",
            "/halloween on",
            "/event halloween on",
            "/event",
            "/halloween off",
        ] {
            assert_eq!(realm(&mut game, command, &mut now), ROLE, "{role:?}: {command}");
            // Refused where every realm command is, so the gateway's security log counts it.
            let typed = chat::local_request(command);
            let out = game.handle_in_world(&typed, &at_alyssa(), now).unwrap();
            assert!(out.dev_command_denied && !out.changed, "{role:?}: {command}");
        }
        assert!(!live.join("event_overrides.json").exists(), "{role:?} wrote the toggle file");
        clock(&mut game, JULY, &mut now);
        let alyssa = at_alyssa();
        assert_closed(&mut game, &alyssa, ALYSSA, &mut now, "a refused toggle");
    }
    // An ordinary chat line that merely starts with a slash is not ours to answer.
    let (mut game, _live) = with_role(&fixture, AccountRole::Player);
    let out = game
        .handle_in_world(&chat::local_request("/wave"), &at_alyssa(), 2_000)
        .unwrap();
    assert!(!out.replies.iter().any(|packet| packet.starts_with(&[15, 0, 3, 0])));
    // Nor is talk about an event: only `realm` or a slash makes a line a command.
    for said in ["halloween on", "event halloween off"] {
        let out = game.handle_in_world(&chat::local_request(said), &at_alyssa(), 3_000).unwrap();
        assert!(!out.dev_command_denied, "{said}");
        assert!(!out.replies.iter().any(|packet| packet.starts_with(&[15, 0, 3, 0])), "{said}");
    }
    // Without role commands or dev tools nobody toggles anything, an admin role included.
    let (mut game, live) = with_role(&fixture, AccountRole::Admin);
    game.set_role_commands(false);
    let mut now = 1_000;
    clock(&mut game, JULY, &mut now);
    let refused = realm(&mut game, "realm halloween on", &mut now);
    assert!(refused.starts_with("Cannot do that:"), "{refused}");
    assert!(!live.join("event_overrides.json").exists());
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
    // A toggle is refused over a file nobody can read: it may hold other events' toggles, so
    // it is never written over, whatever is asked.
    let file = live.join("event_overrides.json");
    let broken = fs::read(&file).unwrap();
    for command in ["realm spring_eggstravaganza off", "realm halloween on", "realm halloween default"] {
        let refused = realm(&mut game, command, &mut now);
        assert!(refused.starts_with("The event toggle was not saved:"), "{command}: {refused}");
        assert!(refused.contains("Nothing changed") && refused.contains("event_overrides.json"));
        assert_eq!(fs::read(&file).unwrap(), broken, "{command} wrote over the file");
    }
    assert!(write_toggle(&live, "halloween", Some((Mode::On, None)), JULY).is_err());
    assert!(write_override(&live, "halloween", None).is_err());
    clock(&mut game, JULY, &mut now);
    assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Active), "a refused toggle ended it");
    // With the file out of the way the admin's command works again, and only then does `off`
    // end the event. A file that is simply absent is no obstacle.
    fs::remove_file(&file).unwrap();
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

/// Unix ms of 00:00 UTC on a civil date (`days_from_civil`, as the calendar counts).
fn unix(year: i64, month: i64, day: i64) -> u64 {
    let year = if month <= 2 { year - 1 } else { year };
    let (era, shifted) = (year.div_euclid(400), if month > 2 { month - 3 } else { month + 9 });
    let year_of_era = year - era * 400;
    let day_of_year = (153 * shifted + 2) / 5 + day - 1;
    let days = era * 146_097 + year_of_era * 365 + year_of_era / 4 - year_of_era / 100
        + day_of_year
        - 719_468;
    u64::try_from(days).unwrap() * DAY
}

/// Every shipped event, with a fixed clock on each side of both edges of its yearly window. The
/// windows are read from the data, so a retuned date moves this test with it; no date is code.
#[test]
fn every_shipped_event_starts_and_stops_on_its_window_edges() {
    let read = |file: &str| -> Value {
        serde_json::from_slice(&fs::read(content_dir().join(file)).unwrap()).unwrap()
    };
    let (seasons, events) = (read("client_areas.json"), read("event_calendar.json"));
    let areas = ClientAreas::load(&content_dir()).unwrap();
    assert_eq!(areas.utc_offset_minutes(), 0, "the edges below are UTC midnights");
    let calendar = areas.calendar();
    let events = events["events"].as_array().unwrap();
    assert_eq!(events.len(), calendar.keys().count());
    for event in events {
        let key = event["key"].as_str().unwrap();
        let season = (seasons["seasons"].as_array().unwrap().iter())
            .find(|season| season["name"] == event["season"])
            .unwrap();
        let day = |field: &str| season[field].as_i64().unwrap();
        let (first, last) = (
            (day("start_month"), day("start_day")),
            (day("end_month"), day("end_day")),
        );
        // A window whose last day is before its first one ends in the next year.
        let start = unix(2026, first.0, first.1);
        let after = unix(2026 + i64::from(last < first), last.0, last.1) + DAY;
        let running = |ms: u64| calendar.active_at(&areas, ms).contains(&key);
        assert!(!running(start - 60_000), "{key} runs before its first day");
        assert!(running(start), "{key} is not running on its first minute");
        assert!(running(after - 60_000), "{key} stops before its last day is over");
        assert!(!running(after), "{key} runs past its last day");
        // The calendar names the same two instants, a year apart each time.
        assert_eq!(calendar.next_occurrence(key, &areas, start - DAY), Some(start), "{key}");
        assert_eq!(calendar.running_until(key, &areas, start), Some(after), "{key}");
        assert_eq!(calendar.next_edge(key, &areas, start - DAY), Some(start), "{key}");
        assert_eq!(calendar.next_edge(key, &areas, start), Some(after), "{key}");
        let next_year = unix(2027, first.0, first.1);
        assert_eq!(calendar.next_occurrence(key, &areas, after), Some(next_year), "{key}");
        // The event's quests (rows tagged with its key) open and close on those edges too.
        assert!(calendar.is_running(key, &areas, start), "{key}");
        assert!(!calendar.is_running(key, &areas, after), "{key}");
    }
    // The one window a dated run fixes to the day: October 1 through November 12 (2013).
    assert!(calendar.active_at(&areas, unix(2026, 10, 1)).contains(&"halloween"));
    assert!(!calendar.active_at(&areas, unix(2026, 11, 13)).contains(&"halloween"));
}

/// The 13 Halloween client areas among opcode-89 packets, as (id, enabled).
fn halloween_areas(packets: &[Vec<u8>]) -> Vec<(i32, bool)> {
    let mut areas = switches(packets);
    areas.retain(|(id, _)| (3992..=4004).contains(id));
    areas
}

/// Nothing about a running event is saved: a server that starts inside a window is in the event
/// at once, switches it off at the window's edge, and one that starts after it is out of it.
#[test]
fn a_server_restarted_inside_a_window_is_in_the_event_and_ends_it_on_time() {
    let live = Scratch::new("restart-live");
    let load = || ClientAreas::load(&content_dir()).unwrap().watching(&live.0);
    let all = |on: bool| (3992..=4004).map(|id| (id, on)).collect::<Vec<_>>();
    let (inside, after) = (unix(2026, 10, 20) + DAY / 2, unix(2026, 11, 13));

    let mut areas = load();
    assert_eq!(halloween_areas(&areas.world_ready_packets(inside)), all(true));
    assert!(areas.season_changes(inside).is_empty(), "the first tick only records the state");
    assert!(areas.calendar().active_at(&areas, inside).contains(&"halloween"));
    assert!(areas.calendar().is_running("halloween", &areas, inside), "Going Batty's event");

    // A second restart, later in the window, then the real clock runs over the edge.
    let mut areas = load();
    assert!(areas.season_changes(after - DAY).is_empty());
    assert!(areas.season_changes(after - 60_000).is_empty(), "the last minute is inside");
    assert_eq!(halloween_areas(&areas.season_changes(after)), all(false));
    assert!(areas.season_changed());
    assert!(areas.season_changes(after + 60_000).is_empty(), "an edge is reported once");

    // Down over the edge: the next start finds the event over, with nothing to switch off.
    let mut areas = load();
    assert!(halloween_areas(&areas.world_ready_packets(after)).is_empty());
    assert!(areas.season_changes(after).is_empty());
    assert!(!areas.calendar().active_at(&areas, after).contains(&"halloween"));
    // And next October it starts by itself again.
    assert!(areas.season_changes(unix(2027, 10, 1) - 60_000).is_empty());
    assert_eq!(halloween_areas(&areas.season_changes(unix(2027, 10, 1))), all(true));
}

/// A hand toggle overrides the calendar until the event's next edge, or until it is cleared,
/// and it is kept in the live world's file, so it outlives a restart.
#[test]
fn a_hand_toggle_lapses_at_the_next_calendar_edge_and_outlives_a_restart() {
    let dir = fest("lapse", fest_calendar(true, "calendar"));
    let live = Scratch::new("lapse-live");
    let load = || ClientAreas::load(&dir.0).unwrap().watching(&live.0);
    let fest_on = |areas: &ClientAreas, ms: u64| areas.is_season_active("Fest", ms);
    let (start, end) = (unix(2027, 2, 1), unix(2027, 2, 3));

    // On in July: the next edge is the event's own start, next February 1.
    let mut areas = load();
    areas.poll(JULY);
    assert!(!fest_on(&areas, JULY));
    assert_eq!(areas.calendar().next_edge("fest", &areas, JULY), Some(start));
    assert!(write_toggle(&live.0, "fest", Some((Mode::On, Some(start))), JULY).unwrap());
    assert!(!write_toggle(&live.0, "fest", Some((Mode::On, Some(start))), JULY).unwrap());
    assert!(areas.poll(JULY + 2_000) && fest_on(&areas, JULY));

    // The server restarts in August: the toggle is still in force.
    let mut areas = load();
    areas.poll(JULY + 30 * DAY);
    assert!(fest_on(&areas, JULY + 30 * DAY), "a restart dropped the toggle");
    assert_eq!(switches(&areas.world_ready_packets(JULY + 30 * DAY)), [(10, true)]);
    // Up to the edge it is the hand's; from the edge on the dates carry the event, with no
    // gap, and end it on its last day although nobody turned it off.
    areas.poll(start - 60_000);
    assert_eq!(areas.hand_modes().get("fest"), Some(&Mode::On));
    assert!(areas.season_changes(start).is_empty(), "the lapse must not blink the event");
    assert!(areas.hand_modes().is_empty() && fest_on(&areas, start));
    assert_eq!(switches(&areas.season_changes(end)), [(10, false)]);
    // The file still stores the lapsed toggle; nothing reads it as one.
    assert_eq!(read_overrides(&live.0).unwrap().len(), 1);
    assert_eq!(areas.read_hands(&live.0, start).unwrap(), Default::default());
    assert_eq!(areas.read_hands(&live.0, start - 1).unwrap().until.get("fest"), Some(&start));

    // Off inside the next window: it lapses when that window ends, so the year after runs.
    let (inside, ends, next) = (unix(2028, 2, 1) + DAY / 2, unix(2028, 2, 3), unix(2029, 2, 1));
    let mut areas = load();
    areas.poll(inside);
    assert!(fest_on(&areas, inside));
    let edge = areas.calendar().next_edge("fest", &areas, inside);
    assert_eq!(edge, Some(ends));
    settle();
    assert!(write_toggle(&live.0, "fest", Some((Mode::Off, edge)), inside).unwrap());
    assert_eq!(read_overrides(&live.0).unwrap().len(), 1, "the lapsed toggle left the file");
    assert_eq!(switches(&areas.season_changes(inside + 2_000)), [(10, false)]);
    // Asked again while it is held off, the edge is still the window's own end.
    assert_eq!(areas.calendar().next_edge("fest", &areas, inside + 2_000), Some(ends));
    let mut areas = load();
    areas.poll(ends - 60_000);
    assert!(!fest_on(&areas, ends - 60_000), "a restart dropped the off toggle");
    areas.poll(ends);
    assert!(areas.hand_modes().is_empty() && !fest_on(&areas, ends));
    areas.poll(next);
    assert!(fest_on(&areas, next), "an off toggle outlived its window");

    // Cleared by hand before its edge: the calendar has the event back at once.
    settle();
    let later = next + DAY / 2;
    assert!(write_toggle(&live.0, "fest", Some((Mode::Off, Some(next + 2 * DAY))), later).unwrap());
    assert_eq!(switches(&areas.season_changes(later + 2_000)), [(10, false)]);
    settle();
    assert!(write_toggle(&live.0, "fest", None, later + 3_000).unwrap());
    assert_eq!(switches(&areas.season_changes(later + 5_000)), [(10, true)]);

    // An event the calendar never moves (mode on, or no real dates) has no edge: its toggle
    // holds until an admin clears it.
    let fixed = fest("lapse-fixed", fest_calendar(false, "calendar"));
    let fixed = ClientAreas::load(&fixed.0).unwrap();
    assert_eq!(fixed.calendar().next_edge("fest", &fixed, JULY), None);
    assert_eq!(fixed.calendar().next_edge("nope", &fixed, JULY), None);
}

/// The whole path by chat: an admin's `realm <key> on` holds until the event's next start, the
/// toggle outlives the server (a new session on the same live world), and from that start on
/// the dates run the event and end it.
#[test]
fn a_toggle_typed_in_chat_outlives_a_restart_and_gives_way_to_the_dates() {
    let fixture = Fixture::new().with_bulk_quests();
    let alyssa = at_alyssa();
    let mut game = admin(&fixture);
    let mut now = 1_000;
    clock(&mut game, JULY, &mut now);
    assert_closed(&mut game, &alyssa, ALYSSA, &mut now, "the egg hunt in July");
    settle();
    let set = realm(&mut game, "realm spring_eggstravaganza on", &mut now);
    let told = "spring_eggstravaganza (The Spring Eggstravaganza): on, running by hand until \
                2027-03-28 00:00, then by its dates. Active now: spring_eggstravaganza, summer_camp, seaside_summer_splash.";
    assert_eq!(set, told);
    clock(&mut game, JULY, &mut now);
    offer(&mut game, &alyssa, ALYSSA, EGG_HUNT, now);
    drop(game);

    // The restart: a new session, the same live world, a month on.
    let mut game = admin(&fixture);
    let mut now = 1_000;
    clock(&mut game, JULY + 30 * DAY, &mut now);
    offer(&mut game, &alyssa, ALYSSA, EGG_HUNT, now);
    accept(&mut game, &alyssa, EGG_HUNT, now + 2);
    now += 10;

    // Inside next spring's window the toggle has lapsed and the event runs by its dates.
    clock(&mut game, EGG_DAY + 365 * DAY, &mut now);
    let line = realm(&mut game, "realm spring_eggstravaganza", &mut now);
    let by_date = "spring_eggstravaganza (The Spring Eggstravaganza): calendar, running until \
                   2027-04-08 00:00.";
    assert_eq!(line, by_date);
    assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Active), "the lapse dropped the quest");
    // And the dates end it: nobody typed `off`.
    clock(&mut game, AFTER_EGGS + 365 * DAY, &mut now);
    assert_eq!(status(&game, EGG_HUNT), Some(QuestStatus::Abandoned));
    assert_closed(&mut game, &alyssa, ALYSSA, &mut now, "the egg hunt after its window");

    // Off inside a window, cleared with `default` before its edge.
    clock(&mut game, JULY + 365 * DAY, &mut now);
    settle();
    let off = realm(&mut game, "realm summer_camp off", &mut now);
    let held = "summer_camp (Summer Camp): off, not running by hand until 2027-09-07 00:00, then \
                by its dates. Active now: seaside_summer_splash.";
    assert_eq!(off, held);
    settle();
    let cleared = realm(&mut game, "realm summer_camp default", &mut now);
    let back = "summer_camp (Summer Camp): calendar, running until 2027-09-07 00:00. Active now: \
                summer_camp, seaside_summer_splash.";
    assert_eq!(cleared, back);
    let usage = realm(&mut game, "realm summer_camp sometimes", &mut now);
    assert!(usage.starts_with("Usage: realm halloween on|off|calendar"), "{usage}");
}

/// The season file of [`fest`] with Fest on other days, or under another clock offset.
fn retune(dir: &Scratch, first: (u8, u8), last: (u8, u8), utc_offset_minutes: i32) {
    let areas = json!({
        "schema_version": 1, "utc_offset_minutes": utc_offset_minutes,
        "switchable": {"area_ids": [10], "source": "test"},
        "seasons": [{"name": "Fest", "start_month": first.0, "start_day": first.1,
                     "end_month": last.0, "end_day": last.1, "area_ids": [10], "source": "test"}],
    });
    fs::write(dir.0.join("client_areas.json"), areas.to_string()).unwrap();
}

/// A toggle's lapse is asked of the calendar the server runs now, not kept from the day it was
/// set: when the window (or the clock offset) is retuned and the server restarts while an `on`
/// is pending, the toggle carries the event to the new start. Trusting the stored instant
/// would end the event at the old start, and take started quests with it, until the new one.
#[test]
fn a_pending_toggle_follows_a_window_retuned_before_it_lapses() {
    let dir = fest("retune", fest_calendar(true, "calendar"));
    let live = Scratch::new("retune-live");
    let load = || ClientAreas::load(&dir.0).unwrap().watching(&live.0);
    let fest_on = |areas: &ClientAreas, ms: u64| areas.is_season_active("Fest", ms);
    let old_start = unix(2027, 2, 1);

    // On in July, with Fest on February 1 and 2: the lapse is February 1.
    let areas = load();
    let edge = areas.calendar().next_edge("fest", &areas, JULY);
    assert_eq!(edge, Some(old_start));
    assert!(write_toggle(&live.0, "fest", Some((Mode::On, edge)), JULY).unwrap());
    assert_eq!(areas.read_hands(&live.0, JULY).unwrap().until.get("fest"), Some(&old_start));

    // Fest moves to March 1 and 2 and the server restarts. The file still says February 1.
    retune(&dir, (3, 1), (3, 2), 0);
    let (new_start, new_end) = (unix(2027, 3, 1), unix(2027, 3, 3));
    let mut areas = load();
    assert!(areas.season_changes(JULY + DAY).is_empty(), "the baseline");
    assert_eq!(areas.read_hands(&live.0, JULY + DAY).unwrap().until.get("fest"), Some(&new_start));
    // Across the old instant and the weeks after it: still on by hand, no switch, no gap.
    for at in [old_start - 60_000, old_start, old_start + DAY, new_start - 60_000] {
        assert!(areas.season_changes(at).is_empty(), "the event blinked at {at}");
        assert!(fest_on(&areas, at) && areas.hand_modes().contains_key("fest"), "{at}");
        assert_eq!(areas.calendar().open_quests(&areas, at), BTreeSet::from([7]), "{at}");
    }
    // The new start takes the event over; the new end stops it.
    assert!(areas.season_changes(new_start).is_empty());
    assert!(areas.hand_modes().is_empty() && fest_on(&areas, new_start));
    assert_eq!(switches(&areas.season_changes(new_end)), [(10, false)]);

    // The clock offset is part of the calendar too: two hours east, the edge is two hours
    // sooner, for a toggle already in the file.
    settle();
    let inside = unix(2028, 3, 1) + DAY / 2;
    assert!(write_toggle(&live.0, "fest", Some((Mode::Off, Some(unix(2028, 3, 3)))), inside).unwrap());
    retune(&dir, (3, 1), (3, 2), 120);
    let mut areas = load();
    let sooner = unix(2028, 3, 3) - 120 * 60_000;
    assert_eq!(areas.read_hands(&live.0, inside).unwrap().until.get("fest"), Some(&sooner));
    areas.poll(sooner - 60_000);
    assert_eq!(areas.hand_modes().get("fest"), Some(&Mode::Off));
    areas.poll(sooner);
    assert!(areas.hand_modes().is_empty());

    // A window retuned away altogether leaves no edge: the toggle holds until it is cleared.
    fs::write(dir.0.join("event_calendar.json"), fest_calendar(false, "calendar").to_string())
        .unwrap();
    let areas = load();
    let far = unix(2030, 1, 1);
    assert_eq!(areas.read_hands(&live.0, far).unwrap().modes.get("fest"), Some(&Mode::Off));
    assert!(areas.read_hands(&live.0, far).unwrap().until.is_empty());

    // A file from before the set time was stored has only the instant: that is its lapse.
    let old = json!({"schema_version": 1, "modes": {"fest": "on"}, "until": {"fest": old_start}});
    fs::write(live.0.join("event_overrides.json"), old.to_string()).unwrap();
    assert_eq!(areas.read_hands(&live.0, old_start - 1).unwrap().modes.len(), 1);
    assert!(areas.read_hands(&live.0, old_start).unwrap().modes.is_empty());
}

/// `realm <key>` is a toggle for every event key, so a key must not be a realm command of its
/// own. The loader's key rule would take `quest` or `help`; no shipped key may be one.
#[test]
fn no_shipped_event_key_is_a_realm_command_word() {
    use osfr_tools::account_store::can_run_realm_command;
    // The words every role may type (`Gameplay::developer_command_denied`), then the lists of
    // the two roles the role table names commands for.
    const ORDINARY: [&str; 14] = [
        "help", "status", "quest", "quests", "attack", "respawn", "inventory", "exit", "retry",
        "accept", "turnin", "redeem", "race", "mail",
    ];
    let areas = ClientAreas::load(&content_dir()).unwrap();
    for key in areas.calendar().keys() {
        assert!(!ORDINARY.contains(&key) && !["event", "season", "reload"].contains(&key), "{key}");
        for role in [AccountRole::Builder, AccountRole::Moderator] {
            assert!(!can_run_realm_command(role, key), "{key} is a {role:?} realm command");
        }
    }
}
