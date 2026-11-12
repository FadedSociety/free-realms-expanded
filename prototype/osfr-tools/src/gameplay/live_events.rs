//! T418: live events an admin opens and closes while the world runs.
//!
//! A quest marked `live_event` in content (the scavenger hunt, 3000200) is
//! offered and accepted only while `<live_dir>/events.json`
//! (`{"schema_version":1,"open":[ids]}`, at most 4 KiB and 64 ids) lists it.
//! `realm event open <id>`, `realm event close <id>` and `realm event list` keep
//! that file; they are admin-only through `account_store::can_run_realm_command`'s
//! default (`event` is neither a builder nor an ordinary command). A session reads
//! the file when its quest catalogue binds or its live folder changes, then polls
//! it once a second with the placement watch, so another session's open needs no
//! relog. A missing, malformed or oversized file closes every event. A closed
//! event is never disabled: an active quest still advances and turns in (the
//! gates are `QuestEngine::offer_denial` and `accept`). No packet is added: the
//! offer is the existing 49/1 (client reader 0x00c7bb60) and each command answers
//! with one `chat::system` line.
//!
//! The event calendar (`crate::event_calendar`) shares the gate: a quest an event lists is
//! open while that event's season is active (by date or by hand), beside whatever the admin
//! has opened. Once a second the session re-reads the hand toggles, and when the running
//! events moved it holds the seasons in its NPC view and closes the ended events' quests:
//! a started one is abandoned (26/23's own `QuestAbandoned` answer) and a finished one is
//! forgotten, as the calendar's `on_end` says. `realm event season` shows and sets the toggles;
//! a toggle holds until the event's next calendar edge or until it is cleared.
use super::*;
use crate::client_areas::{ClientAreas, unix_now_ms};
use crate::event_calendar::{self, Mode};
use crate::world_events::WorldEventCommand;
use std::fs;

const EVENTS_FILE: &str = "events.json";
/// (R) Sanity bounds for an admin's hand-sized list, not tuning values.
const MAX_EVENTS_BYTES: u64 = 4 * 1024;
const MAX_OPEN_EVENTS: usize = 64;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EventsFile {
    schema_version: u32,
    open: BTreeSet<u32>,
}

/// The open ids. A non-regular, oversized or malformed file is an error, which
/// every caller treats as "everything closed"; `NotFound` passes through so the
/// watch and the commands can tell a missing file apart.
fn read_events(path: &Path) -> io::Result<BTreeSet<u32>> {
    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_EVENTS_BYTES {
        return Err(invalid(
            "events.json must be a regular file of at most 4 KiB",
        ));
    }
    let mut bytes = Vec::new();
    file.take(MAX_EVENTS_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_EVENTS_BYTES {
        return Err(invalid(
            "events.json must be a regular file of at most 4 KiB",
        ));
    }
    let events: EventsFile = serde_json::from_slice(&bytes).map_err(invalid)?;
    if events.schema_version != 1 || events.open.len() > MAX_OPEN_EVENTS {
        return Err(invalid(
            "events.json needs schema_version 1 and at most 64 ids",
        ));
    }
    Ok(events.open)
}

/// What this session has applied of `events.json` and the event calendar. Never saved.
#[derive(Clone, Default)]
pub(super) struct EventWatch {
    file: placement_watch::PlacementWatch,
    /// The admin's open list, kept so a calendar change does not need the file again.
    admin_open: BTreeSet<u32>,
    /// This session's season table with the calendar's hand toggles; `None` without content.
    seasons: Option<ClientAreas>,
    /// Tests pin the clock the calendar is judged at; `None` is the wall clock.
    clock: Option<u64>,
    /// The journal has not been cleaned since the last load: the first tick does it, once the
    /// live world (and so the hand toggles) is known and the client can be told.
    unsettled: bool,
}

impl Gameplay {
    /// Pin the clock seasonal content and event quests are judged at (`None`: the wall clock).
    pub fn set_event_clock(&mut self, unix_ms: Option<u64>) {
        self.event_watch.clock = unix_ms;
        if let Some(visibility) = self.npc_visibility.as_mut() {
            visibility.set_season_clock(unix_ms);
        }
    }

    /// Open the admin's events and the running calendar events, hold the seasons in the NPC
    /// view, and (on a tick, when `out` can carry the answer) clear what ended events left in
    /// the journal. Idempotent: it reads state, not transitions, so a character that was
    /// offline when an event ended is cleaned on its first tick.
    fn apply_events(&mut self, unix_ms: u64, out: Option<&mut Outcome>) {
        let watch = &self.event_watch;
        let mut open = watch.admin_open.clone();
        let mut ended = BTreeSet::new();
        let mut on_end = event_calendar::OnEnd::default();
        if let Some(areas) = &watch.seasons {
            let running = areas.calendar().open_quests(areas, unix_ms);
            ended = &areas.calendar().gated_quests() - &running;
            ended.retain(|id| !open.contains(id));
            on_end = areas.calendar().on_end();
            open.extend(running);
            if let Some(visibility) = self.npc_visibility.as_mut() {
                visibility.set_event_modes(areas.hand_modes().clone());
            }
        }
        let Some(engine) = self.npc_quests.as_mut() else {
            return;
        };
        let ignored: Vec<u32> = (engine.set_open_events(&open).into_iter())
            .filter(|id| watch.admin_open.contains(id))
            .collect();
        if !ignored.is_empty() {
            npc_event(serde_json::json!({"event": "live_events_ignored", "ids": ignored}));
        }
        let Some(out) = out else {
            return;
        };
        let (mut dropped, mut forgot, mut waiting) = (Vec::new(), false, false);
        for id in ended {
            match engine.state().quests.get(&id).map(|progress| progress.status.clone()) {
                // A player on this quest's reward screen keeps the reward: try again later.
                Some(crate::quest_engine::QuestStatus::Active)
                    if engine.state().pending_turnin == Some(id) =>
                {
                    waiting = true;
                }
                Some(crate::quest_engine::QuestStatus::Active) if on_end.abandon_active => {
                    dropped.push((id, engine.abandon(id)));
                }
                Some(crate::quest_engine::QuestStatus::Completed) if on_end.forget_completed => {
                    forgot |= engine.forget_completed(id);
                }
                _ => {}
            }
        }
        self.event_watch.unsettled = waiting;
        out.changed |= forgot;
        for (id, result) in dropped {
            if (self.quest_conversation.as_ref()).is_some_and(|open| open.quest_id == id) {
                self.quest_conversation = None;
            }
            if let Err(error) = self.append_npc_quest_outcome(result, out) {
                npc_event(serde_json::json!({
                    "event": "event_quest_close_failed", "quest": id, "error": error.to_string(),
                }));
            }
        }
    }

    /// gifts-codes: whether calendar event `key` runs now for this session, through the shared
    /// gate (`EventCalendar::is_running`) and the hand toggles this session follows.
    pub(crate) fn gift_event_running(&self, key: &str) -> bool {
        let unix_ms = self.event_watch.clock.unwrap_or_else(unix_now_ms);
        (self.event_watch.seasons.as_ref())
            .is_some_and(|areas| areas.calendar().is_running(key, areas, unix_ms))
    }

    /// Every calendar event running now, by the same gate, with its run: the local date the run
    /// ends, or `held` while a hand toggle holds it (what a limited gift's stock counts per).
    pub(crate) fn gift_running_events(&self) -> crate::gifts::Events {
        let Some(areas) = self.event_watch.seasons.as_ref() else {
            return crate::gifts::Events::new();
        };
        let unix_ms = self.event_watch.clock.unwrap_or_else(unix_now_ms);
        let calendar = areas.calendar();
        (calendar.keys())
            .filter(|key| self.gift_event_running(key))
            .map(|key| {
                let run = calendar.running_until(key, areas, unix_ms).map_or_else(
                    || "held".to_owned(),
                    |end| event_calendar::local_date(areas.local_minute(end)),
                );
                (key.to_owned(), run)
            })
            .collect()
    }

    fn events_path(&self) -> PathBuf {
        self.live_dir().join(EVENTS_FILE)
    }

    /// Read the open list now (catalogue bind, live folder change, own write),
    /// so an NPC Talk never waits for the first tick to see the file.
    pub(super) fn load_live_events(&mut self) {
        let clock = self.event_watch.clock;
        // The season table the NPC view reads (`Gameplay::load`), so both follow one calendar.
        let content = crate::content::npc_content(&self.content_dir);
        let seasons = match ClientAreas::load(&content) {
            Ok(areas) => Some(areas.watching(&self.live_dir())),
            Err(error) => {
                npc_event(serde_json::json!({
                    "event": "event_calendar_refused", "error": error.to_string(),
                }));
                None
            }
        };
        if let (Some(engine), Some(areas)) = (self.npc_quests.as_mut(), &seasons) {
            engine.gate_events(areas.calendar().gated_quests());
        }
        self.event_watch = EventWatch {
            seasons,
            clock,
            unsettled: true,
            ..EventWatch::default()
        };
        self.tick_live_events(0, None);
    }

    /// Apply a changed `events.json` to the quest engine, at most one stat per
    /// second. A refused file closes every event; it and any id that is not a
    /// live event are logged once per file version. World state only: no
    /// packet and no save for the admin's list; an idle tick is one comparison. The same
    /// second's poll follows the event calendar (see the module note).
    pub(super) fn tick_live_events(&mut self, now_ms: u64, out: Option<&mut Outcome>) {
        if !self.event_watch.file.due(now_ms) {
            return;
        }
        let path = self.events_path();
        let mut moved = match self.event_watch.file.poll_with(&path, now_ms, read_events) {
            Ok(Some(open)) => {
                self.event_watch.admin_open = open;
                true
            }
            Ok(None) => false,
            Err(error) => {
                npc_event(serde_json::json!({
                    "event": "live_events_refused",
                    "error": error.to_string(),
                }));
                self.event_watch.admin_open.clear();
                true
            }
        };
        let unix_ms = self.event_watch.clock.unwrap_or_else(unix_now_ms);
        if let Some(areas) = self.event_watch.seasons.as_mut() {
            moved |= areas.poll(unix_ms);
        }
        if moved || out.is_some() && self.event_watch.unsettled {
            self.apply_events(unix_ms, out);
        }
    }

    /// The `realm event` arm. T458's `start <key>` and `stop <key>` only stage the world event
    /// for the host, which answers once the admin's transaction commits (`crate::world_events`);
    /// every other verb answers with one line here.
    pub(super) fn realm_event(&mut self, args: &[&str], out: &mut Outcome) -> io::Result<()> {
        match args {
            ["start", key] => out.world_event = Some(WorldEventCommand::Start(key.to_string())),
            ["stop", key] => out.world_event = Some(WorldEventCommand::Stop(key.to_string())),
            _ => out
                .replies
                .push(chat::system(&self.live_event_command(args)?)),
        }
        Ok(())
    }

    /// `realm event open <id>`, `realm event close <id>` and `realm event list`.
    /// One chat line each; a refusal writes nothing and changes nothing.
    fn live_event_command(&mut self, args: &[&str]) -> io::Result<String> {
        if let ["season", rest @ ..] = args {
            return Ok(self.event_season_command(rest));
        }
        let Some(mut live) = self.npc_quests.as_ref().map(|q| q.live_events().clone()) else {
            return Ok("Live events need the quest catalogue.".into());
        };
        // A calendar event's quests follow their season; `realm event season` switches those.
        if let Some(areas) = &self.event_watch.seasons {
            let seasonal = areas.calendar().gated_quests();
            live.retain(|id, _| !seasonal.contains(id));
        }
        let (opening, id) = match args {
            ["list"] => {
                let ids = |open: bool| {
                    let ids: Vec<String> = live
                        .iter()
                        .filter(|(_, is_open)| **is_open == open)
                        .map(|(id, _)| id.to_string())
                        .collect();
                    if ids.is_empty() {
                        "none".to_owned()
                    } else {
                        ids.join(", ")
                    }
                };
                return Ok(format!(
                    "Open events: {}. Closed events: {}.",
                    ids(true),
                    ids(false)
                ));
            }
            [verb @ ("open" | "close"), id] => (*verb == "open", *id),
            _ => {
                return Ok(
                    "Usage: realm event open|close <quest id>, realm event list or realm event start|stop <world event key>."
                        .into(),
                );
            }
        };
        let Some(id) = id.parse::<u32>().ok().filter(|id| live.contains_key(id)) else {
            return Ok(format!(
                "{id} is not a live event quest; realm event list names them."
            ));
        };
        let path = self.events_path();
        let mut open = match read_events(&path) {
            Ok(open) => open,
            Err(error) if error.kind() == io::ErrorKind::NotFound => BTreeSet::new(),
            Err(error) => {
                return Ok(format!(
                    "Could not read {EVENTS_FILE}: {error}. Nothing changed."
                ));
            }
        };
        let state = if opening { "open" } else { "closed" };
        let changed = if opening {
            open.insert(id)
        } else {
            open.remove(&id)
        };
        if !changed {
            return Ok(format!("Event {id} is already {state}."));
        }
        if open.len() > MAX_OPEN_EVENTS {
            return Ok(format!(
                "At most {MAX_OPEN_EVENTS} events can be open. Nothing changed."
            ));
        }
        let file = EventsFile {
            schema_version: 1,
            open,
        };
        let text = serde_json::to_string_pretty(&file).map_err(invalid)? + "\n";
        if let Err(error) = crate::character_store::replace_bytes(&path, text.as_bytes()) {
            return Ok(format!(
                "Could not save {EVENTS_FILE}: {error}. Nothing changed."
            ));
        }
        self.load_live_events(); // the acting session applies its own write at once
        Ok(format!("Event {id} is {state}."))
    }

    /// The words of an event toggle, whichever way it was typed (`lower`: the trimmed,
    /// lower-cased chat line): `realm event season <words>`, the short `realm <key> <words>`
    /// for every key of `event_calendar.json` (`realm halloween on`), and the slash forms
    /// `/event <words>` and `/<key> <words>` (`/halloween on`). The stock client runs a line
    /// that starts with `/` in its own console and answers an unknown one itself ("Command not
    /// supported", client 0x00a9e480 and 0x00a9e2d0), so only the `realm` forms leave it; the
    /// slash forms are for a client that sends them. `None`: not an event toggle.
    pub(super) fn season_args<'a>(&self, lower: &'a str) -> Option<Vec<&'a str>> {
        let calendar = self.event_watch.seasons.as_ref()?.calendar();
        let (slash, rest) = match lower.strip_prefix('/') {
            Some(rest) => (true, rest),
            None => (false, lower.strip_prefix("realm ")?),
        };
        let mut words = rest.split_whitespace();
        match words.next()? {
            "event" if slash => Some(words.collect()),
            "event" => (words.next() == Some("season")).then(|| words.collect()),
            key if calendar.has_event(key) => Some(std::iter::once(key).chain(words).collect()),
            _ => None,
        }
    }

    /// Chat's first stop: the realm router's role refusal for any command this account may not
    /// run, then the event toggles. Every form of a toggle is the realm command `season` to the
    /// role table (`account_store::can_run_realm_command` through `developer_command_denied`),
    /// so a refused toggle gets the same line, flag and security-log entry as any other.
    pub(super) fn event_command(&mut self, inner: &[u8], guid: u64) -> io::Result<Option<Outcome>> {
        if let Some(refused) = self.command_refusal(inner, guid)? {
            return Ok(Some(refused));
        }
        let Some(text) = chat::request(inner, guid)? else {
            return Ok(None);
        };
        let lower = text.trim().to_ascii_lowercase();
        let Some(args) = self.season_args(&lower) else {
            return Ok(None);
        };
        let reply = chat::system(&self.event_season_command(&args));
        Ok(Some(Outcome {
            replies: vec![reply],
            ..Outcome::default()
        }))
    }

    /// One event as an admin reads it: its mode, whether it runs, and until or from when.
    /// `until` is when its hand toggle lapses (the event's next calendar edge), if it does.
    fn event_line(areas: &ClientAreas, key: &str, until: Option<u64>, unix_ms: u64) -> String {
        let calendar = areas.calendar();
        let Some((name, mode)) = calendar.describe(key, areas.hand_modes()) else {
            return format!("{key} is not a calendar event");
        };
        let date = |ms: u64| event_calendar::local_date(areas.local_minute(ms));
        let running = calendar.active_at(areas, unix_ms).contains(&key);
        let state = match (mode, running, until) {
            (Mode::On, _, Some(until)) => {
                format!("on, running by hand until {}, then by its dates", date(until))
            }
            (Mode::Off, _, Some(until)) => {
                format!("off, not running by hand until {}, then by its dates", date(until))
            }
            (Mode::On, ..) => "on, running until an admin turns it off".to_owned(),
            (Mode::Off, ..) => "off, not running until an admin turns it on".to_owned(),
            (Mode::Calendar, true, _) => match calendar.running_until(key, areas, unix_ms) {
                Some(end) => format!("calendar, running until {}", date(end)),
                None => "calendar, running".to_owned(),
            },
            (Mode::Calendar, false, _) => match calendar.next_occurrence(key, areas, unix_ms) {
                Some(start) => format!("calendar, not running, next starts {}", date(start)),
                None => "calendar, not running".to_owned(),
            },
        };
        format!("{key} ({name}): {state}")
    }

    /// The event toggles, for admins only (owner 2026-11-09): list the calendar's events, show
    /// one, or set its hand toggle. `on` starts the event and its quests now, `off` ends it now
    /// by the calendar's end rules; either holds until the event's next calendar edge (the
    /// start or end its dates give it next), where the calendar takes over again, or until
    /// `default` clears it (`calendar` also returns it to its dates, over a content mode of on
    /// or off too). The write goes to the live world's `event_overrides.json`; every session
    /// and the gateway's shared world pick it up at their next one-second poll, this one
    /// included. Only the role table's `season` command may use it (admins: a tester's wide
    /// realm rights stop short of a switch that changes the world for every player), and a
    /// refused caller never reaches the file.
    fn event_season_command(&mut self, args: &[&str]) -> String {
        if !self.may_run("season") {
            return "Cannot do that: your account role does not allow this command.".into();
        }
        let Some(areas) = self.event_watch.seasons.as_ref() else {
            return "The event calendar is not loaded.".into();
        };
        let (live, unix_ms) = (self.live_dir(), self.event_watch.clock.unwrap_or_else(unix_now_ms));
        // The answer describes the toggles as the file now holds them, not the last poll.
        let view = |areas: &ClientAreas| {
            let mut view = areas.clone();
            let hands = areas.read_hands(&live, unix_ms).unwrap_or_else(|_| {
                event_calendar::Hands {
                    modes: view.hand_modes().clone(),
                    ..Default::default()
                }
            });
            view.set_hand_modes(hands.modes);
            (view, hands.until)
        };
        let line = |(view, until): &(ClientAreas, BTreeMap<String, u64>), key: &str| {
            Self::event_line(view, key, until.get(key).copied(), unix_ms)
        };
        let (key, mode) = match args {
            [] => {
                let view = view(areas);
                let lines: Vec<String> =
                    (view.0.calendar().keys()).map(|key| line(&view, key)).collect();
                if lines.is_empty() {
                    return "The event calendar is empty.".into();
                }
                return lines.join("; ") + ".";
            }
            [key] if areas.calendar().has_event(key) => return line(&view(areas), key) + ".",
            [key, "on"] => (*key, Some(Mode::On)),
            [key, "off"] => (*key, Some(Mode::Off)),
            [key, "calendar"] => (*key, Some(Mode::Calendar)),
            [key, "default"] => (*key, None),
            _ => {
                return "Usage: realm halloween on|off|calendar (any event key), or realm event season."
                    .into();
            }
        };
        if !areas.calendar().has_event(key) {
            return format!("{key} is not a calendar event; realm event season lists them.");
        }
        // On and off lapse at the event's next calendar edge; `calendar` has no edge to wait for.
        let toggle = mode.map(|mode| {
            let edge = || areas.calendar().next_edge(key, areas, unix_ms);
            (mode, (mode != Mode::Calendar).then(edge).flatten())
        });
        if let Err(error) = event_calendar::write_toggle(&live, key, toggle, unix_ms) {
            // An unreadable file may hold other events' toggles: it is never written over.
            return format!(
                "The event toggle was not saved: {error}. Nothing changed; repair or delete \
                 {} in the live world, then try again.",
                event_calendar::OVERRIDES_FILE
            );
        }
        let view = view(areas);
        let active = view.0.calendar().active_at(&view.0, unix_ms).join(", ");
        let active = if active.is_empty() { "nothing".to_owned() } else { active };
        format!("{}. Active now: {active}.", line(&view, key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("osfr-live-events-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn the_reader_refuses_everything_outside_the_bounds() {
        let dir = directory("bounds");
        let path = dir.join(EVENTS_FILE);
        let ids = |count: u32| (1..=count).map(|id| id.to_string()).collect::<Vec<_>>();
        let cases = [
            (r#"{"schema_version":1,"open":[3000200]}"#.to_owned(), true),
            (
                format!(r#"{{"schema_version":1,"open":[{}]}}"#, ids(64).join(",")),
                true,
            ),
            (
                format!(r#"{{"schema_version":1,"open":[{}]}}"#, ids(65).join(",")),
                false,
            ),
            (r#"{"schema_version":2,"open":[3000200]}"#.to_owned(), false),
            (
                r#"{"schema_version":1,"open":[3000200],"extra":1}"#.to_owned(),
                false,
            ),
            (r#"{"schema_version":1,"open":[-1]}"#.to_owned(), false),
            (r#"{"schema_version":1,"open":["#.to_owned(), false),
            (
                format!(
                    r#"{{"schema_version":1,"open":[3000200]}}{}"#,
                    " ".repeat(4096)
                ),
                false,
            ),
        ];
        for (text, accepted) in cases {
            fs::write(&path, &text).unwrap();
            assert_eq!(
                read_events(&path).is_ok(),
                accepted,
                "{}",
                &text[..text.len().min(60)]
            );
        }
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(read_events(&path).is_err(), "a directory is refused");
        let _ = fs::remove_dir_all(&dir);
    }

    /// Only a start or stop touches `Outcome::world_event`; the other verbs leave a staged
    /// command alone.
    #[test]
    fn only_a_start_or_stop_stages_the_world_event() {
        let content = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-opening-cooking-original-assets");
        let mut game = Gameplay::load(&content, None).unwrap();
        let staged = Some(WorldEventCommand::Stop("kept".into()));
        let mut out = Outcome {
            world_event: staged.clone(),
            ..Outcome::default()
        };
        game.realm_event(&["list"], &mut out).unwrap();
        assert_eq!(out.world_event, staged, "event list leaves it staged");
        assert_eq!(out.replies.len(), 1, "and still answers");
        game.realm_event(&["start", "show"], &mut out).unwrap();
        assert_eq!(
            out.world_event,
            Some(WorldEventCommand::Start("show".into()))
        );
    }

    /// The tick logs a refused file when the poll errors; the watch's
    /// `Rejected` stamp makes that once per file version, not once per second.
    #[test]
    fn a_refused_file_is_reported_once_per_version() {
        let dir = directory("once");
        let path = dir.join(EVENTS_FILE);
        fs::write(&path, "{").unwrap();
        let mut watch = placement_watch::PlacementWatch::default();
        assert!(watch.poll_with(&path, 1_000, read_events).is_err());
        assert_eq!(watch.poll_with(&path, 2_000, read_events).unwrap(), None);
        assert_eq!(watch.poll_with(&path, 3_000, read_events).unwrap(), None);
        fs::remove_file(&path).unwrap();
        assert_eq!(
            watch.poll_with(&path, 4_000, read_events).unwrap(),
            Some(BTreeSet::new()),
            "a deleted file is a change to everything closed"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
