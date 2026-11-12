//! The event calendar: which events exist, what each brings, and when each runs. Content:
//! `event_calendar.json`, beside `client_areas.json`; a missing file means no calendar and
//! every season follows its dates, as before. A new event or rotation is one JSON entry.
//!
//! An event names one `client_areas.json` season. That season already carries the event's
//! decorations (opcode-89 client areas), its people and props (`season` on a spawn row,
//! `world.rs`) and its scheduled shows (`world_events/schedule.rs`); the calendar adds the
//! event's quests and decides when the season is on:
//! * by date: the season's own month/day window, once a year (`by_date`, the default);
//! * by rotation: a `rotations` row takes turns through its `events`, one per `day`, `week`
//!   (Monday first) or `month` of local time, each turn running from `offset_minutes` into the
//!   period for `length_minutes` (the whole period when absent). One event in a row is a plain
//!   daily, weekly or monthly event. Rows are independent, so events overlap whenever their
//!   windows do, and an event may be dated and in rotations at once;
//! * by hand: an event's `mode` is `calendar` (the two rules above), `on` or `off`, and a hand
//!   toggle (`realm event season <key> on|off|calendar|default`, kept in the live world's
//!   `event_overrides.json`) wins over the content's mode until the event's next calendar edge
//!   (the first start or end its dates and rotations give it after the toggle was set, by
//!   the calendar as it is when the lapse is checked) or until an admin clears it,
//!   whichever comes first; an event the calendar never moves keeps its toggle until cleared.
//!   The file outlives a restart, so a toggle does too. `run_on_real_dates: false` turns every
//!   `calendar` event off, so nothing starts unless someone switches it on.
//!
//! Everything here is a pure function of a clock the caller passes, so "what is active at this
//! time", "whose turn is it" and "when does it next start" are all answerable for any instant.
//! The switch works by answering for the season inside [`ClientAreas`], so everything that asks
//! "is this season active" follows the calendar without knowing about it. When an event ends,
//! its quests close with it (`gameplay/live_events.rs`): nothing of it is left drawn, offered or
//! in a journal.
use crate::client_areas::ClientAreas;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

pub const FILE: &str = "event_calendar.json";
pub const OVERRIDES_FILE: &str = "event_overrides.json";
// (R) Sanity bounds for hand-written content, not tuning values.
const MAX_BYTES: u64 = 256 * 1024;
const MAX_OVERRIDE_BYTES: u64 = 4 * 1024;
const MAX_EVENTS: usize = 64;
const MAX_ROTATIONS: usize = 32;
const MAX_ROTATION_EVENTS: usize = 32;
const MAX_EVENT_QUESTS: usize = 256;
const MAX_KEY_BYTES: usize = 32;
const DAY: i64 = 1440;
/// How far [`EventCalendar::next_occurrence`] looks: every yearly date and rotation comes round.
const HORIZON_DAYS: i64 = 400;
/// (R) A hand toggle is human-paced: one stat a second, as the placement watch.
const POLL_MS: u64 = 1_000;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// The event's dates and rotations.
    #[default]
    Calendar,
    On,
    Off,
}

/// The length of one rotation turn, in local time.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Period {
    Day,
    /// Monday 00:00 to the next Monday.
    Week,
    /// The first of the month to the next first.
    Month,
}

/// Days since 1970-01-01 of a civil date (Howard Hinnant's public-domain `days_from_civil`).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * shifted + 2) / 5 + day - 1;
    era * 146_097 + year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year - 719_468
}

/// "2026-10-31 18:00" for a local minute since 1970, as the admin's reply shows it.
pub fn local_date(minute: i64) -> String {
    let (year, month, day) = civil(minute.div_euclid(DAY));
    let of_day = minute.rem_euclid(DAY);
    format!("{year:04}-{month:02}-{day:02} {:02}:{:02}", of_day / 60, of_day % 60)
}

/// (year, month 1-12) of a day count.
fn year_month(days: i64) -> (i64, i64) {
    let (year, month, _) = civil(days);
    (year, month)
}

/// (year, month 1-12, day 1-31) of a day count (`civil_from_days`).
fn civil(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

impl Period {
    /// The shortest this period can be, in minutes.
    fn least_minutes(self) -> i64 {
        match self {
            Self::Day => DAY,
            Self::Week => 7 * DAY,
            Self::Month => 28 * DAY,
        }
    }

    /// Which period holds local minute `minute`. Periods count from 1970 and may be negative.
    fn index(self, minute: i64) -> i64 {
        let day = minute.div_euclid(DAY);
        match self {
            Self::Day => day,
            // 1970-01-01 was a Thursday; the Monday before it is day -3.
            Self::Week => (day + 3).div_euclid(7),
            Self::Month => {
                let (year, month) = year_month(day);
                year * 12 + month - 1
            }
        }
    }

    /// The first local minute of period `index`.
    fn start(self, index: i64) -> i64 {
        DAY * match self {
            Self::Day => index,
            Self::Week => index * 7 - 3,
            Self::Month => days_from_civil(index.div_euclid(12), index.rem_euclid(12) + 1, 1),
        }
    }
}

/// What happens to an event's quests in a character's journal once the event is over.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OnEnd {
    /// (R) Drop a started quest: its people and props are gone, so it could not be finished.
    pub abandon_active: bool,
    /// (R) Forget a finished quest, so the next run offers it again. Off by default: a
    /// forgotten quest no longer counts toward `quest_completed_count` (the quest achievements
    /// at 10 to 400), and the save has no field yet to remember it by.
    pub forget_completed: bool,
}

impl Default for OnEnd {
    fn default() -> Self {
        Self {
            abandon_active: true,
            forget_completed: false,
        }
    }
}

fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Content {
    schema_version: u32,
    /// False: no event starts by date or rotation; only a mode or hand toggle of `on` runs one.
    #[serde(default = "yes")]
    run_on_real_dates: bool,
    #[serde(default)]
    on_end: OnEnd,
    events: Vec<Event>,
    #[serde(default)]
    rotations: Vec<Rotation>,
    source: String,
}

impl Default for Content {
    fn default() -> Self {
        Self {
            schema_version: 1,
            run_on_real_dates: true,
            on_end: OnEnd::default(),
            events: Vec::new(),
            rotations: Vec::new(),
            source: String::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Event {
    key: String,
    name: String,
    /// A `client_areas.json` season; at most one event per season.
    season: String,
    #[serde(default)]
    mode: Mode,
    /// False: the season's yearly dates do not start it; only a rotation or a hand does.
    #[serde(default = "yes")]
    by_date: bool,
    /// Quests offered only while the event runs. Shipped content tags the quest row instead
    /// (`live_event`: this event's key, asked through [`EventCalendar::is_running`]); this list
    /// is the older way to say the same and still gates.
    #[serde(default)]
    quest_ids: Vec<u32>,
    source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rotation {
    key: String,
    period: Period,
    /// Event keys in turn order; period `n` (counted from 1970) is `events[n mod len]`'s.
    events: Vec<String>,
    /// Where in the period a turn starts.
    #[serde(default)]
    offset_minutes: u32,
    /// How long a turn runs; absent means to the end of the period.
    length_minutes: Option<u32>,
    source: String,
}

/// One turn of a rotation: whose it is and when it runs, in local minutes since 1970
/// ([`ClientAreas::local_minute`]). `ends` is exclusive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn<'a> {
    pub rotation: &'a str,
    pub event: &'a str,
    /// The period number, counted from 1970.
    pub index: i64,
    pub starts: i64,
    pub ends: i64,
}

impl Turn<'_> {
    pub fn contains(&self, minute: i64) -> bool {
        (self.starts..self.ends).contains(&minute)
    }
}

impl Rotation {
    fn turn(&self, index: i64) -> Turn<'_> {
        let start = self.period.start(index);
        let starts = start + i64::from(self.offset_minutes);
        let ends = match self.length_minutes {
            Some(length) => starts + i64::from(length),
            None => self.period.start(index + 1),
        };
        // `validate` keeps `events` non-empty and at most 32 long.
        let slot = index.rem_euclid(self.events.len().max(1) as i64) as usize;
        Turn {
            rotation: &self.key,
            event: self.events.get(slot).map_or("", String::as_str),
            index,
            starts,
            ends,
        }
    }

    /// The turn running at `minute`. A turn may run past its period's end into the next one
    /// (never further: `validate`), so the period before is looked at too; the newer one wins.
    fn running(&self, minute: i64) -> Option<Turn<'_>> {
        let index = self.period.index(minute);
        [index, index - 1]
            .into_iter()
            .map(|index| self.turn(index))
            .find(|turn| turn.contains(minute))
    }
}

fn valid_key(key: &str) -> bool {
    (1..=MAX_KEY_BYTES).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

impl Content {
    fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.events.len() > MAX_EVENTS
            || self.rotations.len() > MAX_ROTATIONS
        {
            return Err("needs schema_version 1, at most 64 events and at most 32 rotations".into());
        }
        if !self.events.is_empty() && self.source.trim().is_empty() {
            return Err("needs a source".into());
        }
        let (mut keys, mut seasons, mut quests) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
        for event in &self.events {
            let sane = valid_key(&event.key)
                && keys.insert(event.key.as_str())
                && !event.name.trim().is_empty()
                && !event.season.trim().is_empty()
                && seasons.insert(event.season.as_str())
                && !event.source.trim().is_empty()
                && event.quest_ids.len() <= MAX_EVENT_QUESTS
                && event.quest_ids.iter().all(|id| *id != 0 && quests.insert(*id));
            if !sane {
                return Err(format!(
                    "event {:?} needs a unique key of a-z, 0-9 and _ (at most 32 bytes), a name, a \
                     season no other event uses, a source and at most 256 quest ids no other event lists",
                    event.key
                ));
            }
        }
        let mut rotation_keys = BTreeSet::new();
        for rotation in &self.rotations {
            let least = rotation.period.least_minutes();
            let offset = i64::from(rotation.offset_minutes);
            let length = rotation.length_minutes.map(i64::from);
            let sane = valid_key(&rotation.key)
                && rotation_keys.insert(rotation.key.as_str())
                && (1..=MAX_ROTATION_EVENTS).contains(&rotation.events.len())
                && rotation.events.iter().all(|key| keys.contains(key.as_str()))
                && offset < least
                // A turn ends inside the next period at the latest, so two periods cover a search.
                && length.is_none_or(|length| length >= 1 && offset + length <= 2 * least)
                && !rotation.source.trim().is_empty();
            if !sane {
                return Err(format!(
                    "rotation {:?} needs a unique key, 1 to 32 events that the calendar defines, an \
                     offset inside its period, a length that ends within the next period and a source",
                    rotation.key
                ));
            }
        }
        Ok(())
    }
}

fn invalid(path: &Path, error: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {error}", path.display()),
    )
}

/// At most `max` bytes of a regular file; `NotFound` passes through.
fn read_bounded(path: &Path, max: u64) -> io::Result<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(max + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max {
        return Err(invalid(path, format!("larger than {max} bytes")));
    }
    Ok(bytes)
}

/// One event as the admin's `realm event season` line shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventState<'a> {
    pub key: &'a str,
    pub name: &'a str,
    pub mode: Mode,
    pub running: bool,
}

#[derive(Clone, Debug, Default)]
pub struct EventCalendar {
    content: Arc<Content>,
}

impl EventCalendar {
    /// Reads `event_calendar.json` from `directory`. A missing file is an empty calendar; a
    /// malformed one is an error naming the file.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join(FILE);
        let bytes = match read_bounded(&path, MAX_BYTES) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            other => other?,
        };
        let content: Content =
            serde_json::from_slice(&bytes).map_err(|error| invalid(&path, error))?;
        content.validate().map_err(|error| invalid(&path, error))?;
        Ok(Self {
            content: Arc::new(content),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.content.events.is_empty()
    }

    pub fn on_end(&self) -> OnEnd {
        self.content.on_end
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.content.events.iter().map(|event| event.key.as_str())
    }

    pub fn has_event(&self, key: &str) -> bool {
        self.content.events.iter().any(|event| event.key == key)
    }

    /// An event's display name and its mode under the hand toggles.
    pub fn describe(&self, key: &str, hand: &BTreeMap<String, Mode>) -> Option<(&str, Mode)> {
        let event = self.content.events.iter().find(|event| event.key == key)?;
        Some((&event.name, self.mode(event, hand)))
    }

    /// Every season an event names.
    pub fn seasons(&self) -> impl Iterator<Item = &str> {
        self.content.events.iter().map(|event| event.season.as_str())
    }

    fn mode(&self, event: &Event, hand: &BTreeMap<String, Mode>) -> Mode {
        hand.get(&event.key).copied().unwrap_or(event.mode)
    }

    /// The rotation turns running at local minute `minute`, one per rotation at most.
    pub fn turns_at(&self, minute: i64) -> impl Iterator<Item = Turn<'_>> {
        (self.content.rotations.iter()).filter_map(move |rotation| rotation.running(minute))
    }

    /// Whose turn rotation `key`'s period is at local minute `minute`, with the turn's window.
    /// The turn may not have started yet, or be over already: ask [`Turn::contains`].
    pub fn current_rotation(&self, key: &str, minute: i64) -> Option<Turn<'_>> {
        let rotation = self.content.rotations.iter().find(|row| row.key == key)?;
        Some(rotation.turn(rotation.period.index(minute)))
    }

    /// The calendar's word on `season` at local minute `minute`, given the hand toggles (event
    /// key to mode): held on, held off, or `None` when the season's own dates decide (no event
    /// names it, or its event is dated and no rotation runs it now).
    // ponytail: linear in events and rotations per question (64 and 32 at most); index by
    // season if a profile ever shows it.
    pub fn held(&self, season: &str, hand: &BTreeMap<String, Mode>, minute: i64) -> Option<bool> {
        let content = &self.content;
        let event = content.events.iter().find(|event| event.season == season)?;
        match self.mode(event, hand) {
            Mode::On => Some(true),
            Mode::Off => Some(false),
            Mode::Calendar if !content.run_on_real_dates => Some(false),
            Mode::Calendar if self.turns_at(minute).any(|turn| turn.event == event.key) => {
                Some(true)
            }
            Mode::Calendar => (!event.by_date).then_some(false),
        }
    }

    /// Every quest an event gates.
    pub fn gated_quests(&self) -> BTreeSet<u32> {
        (self.content.events.iter())
            .flat_map(|event| event.quest_ids.iter().copied())
            .collect()
    }

    fn running<'a>(&'a self, areas: &'a ClientAreas, unix_ms: u64) -> impl Iterator<Item = &'a Event> {
        (self.content.events.iter())
            .filter(move |event| areas.is_season_active(&event.season, unix_ms))
    }

    /// The keys of the events running at `unix_ms`. `areas` must be the season table loaded
    /// beside this calendar; it carries the hand toggles and the clock's offset.
    pub fn active_at<'a>(&'a self, areas: &'a ClientAreas, unix_ms: u64) -> Vec<&'a str> {
        (self.running(areas, unix_ms).map(|event| event.key.as_str())).collect()
    }

    /// The one "is this event running now" gate (event-repeat contract): event `key`'s dates
    /// and rotations under the hand toggles `areas` carries. An unknown key never runs. Gifts
    /// (`crate::gifts`) and quest rows (`live_event`, `gameplay/live_events.rs`) ask it; any
    /// row kind with a `live_event` key can.
    pub fn is_running(&self, key: &str, areas: &ClientAreas, unix_ms: u64) -> bool {
        self.running(areas, unix_ms).any(|event| event.key == key)
    }

    /// The quests of the events running at `unix_ms`.
    pub fn open_quests(&self, areas: &ClientAreas, unix_ms: u64) -> BTreeSet<u32> {
        (self.running(areas, unix_ms))
            .flat_map(|event| event.quest_ids.iter().copied())
            .collect()
    }

    /// The next instant after `after_ms` at which event `key` starts (unix ms), by its dates or
    /// any rotation, within the next 400 days. `None` for an unknown event, one that never
    /// starts by itself, or one held on or off by hand (it has no next start).
    pub fn next_occurrence(&self, key: &str, areas: &ClientAreas, after_ms: u64) -> Option<u64> {
        self.next_change(key, areas, after_ms, true)
    }

    /// The instant after `after_ms` at which a running event `key` stops by itself (unix ms),
    /// within the next 400 days; `None` when it is held by hand or does not stop in that time.
    pub fn running_until(&self, key: &str, areas: &ClientAreas, after_ms: u64) -> Option<u64> {
        self.next_change(key, areas, after_ms, false)
    }

    /// The next instant after `after_ms` at which event `key` starts or stops by its own
    /// calendar, its hand toggle aside: where a toggle set now lapses. `None` when the calendar
    /// never moves it (a content mode of on or off, no dates), so the toggle holds until cleared.
    pub fn next_edge(&self, key: &str, areas: &ClientAreas, after_ms: u64) -> Option<u64> {
        let mut plain = areas.clone();
        let mut hand = plain.hand_modes().clone();
        hand.remove(key);
        plain.set_hand_modes(hand);
        let running = self.active_at(&plain, after_ms).contains(&key);
        self.next_change(key, &plain, after_ms, !running)
    }

    /// The first instant after `after_ms` at which the event turns `to` (running or not).
    fn next_change(&self, key: &str, areas: &ClientAreas, after_ms: u64, to: bool) -> Option<u64> {
        let event = self.content.events.iter().find(|event| event.key == key)?;
        let after = areas.local_minute(after_ms);
        // A change is a local midnight (a dated season) or the start or end of one of its turns.
        let mut starts: BTreeSet<i64> = (1..=HORIZON_DAYS)
            .map(|day| (after.div_euclid(DAY) + day) * DAY)
            .collect();
        let horizon = after + HORIZON_DAYS * DAY;
        for rotation in &self.content.rotations {
            let mut index = rotation.period.index(after) - 1;
            loop {
                let turn = rotation.turn(index);
                if turn.starts > horizon {
                    break;
                }
                if turn.event == event.key {
                    starts.extend([turn.starts, turn.ends]);
                }
                index += 1;
            }
        }
        let unix = |minute: i64| {
            let utc = minute.checked_sub(i64::from(areas.utc_offset_minutes()))?;
            u64::try_from(utc.checked_mul(60_000)?).ok()
        };
        let active = |minute: i64| {
            unix(minute).is_some_and(|ms| areas.is_season_active(&event.season, ms))
        };
        let first = (starts.into_iter())
            .find(|start| *start > after && active(*start) == to && active(start - 1) != to);
        first.and_then(unix)
    }

    /// Each event with its effective mode and whether it runs at `unix_ms`.
    pub fn states<'a>(
        &'a self,
        hand: &BTreeMap<String, Mode>,
        areas: &ClientAreas,
        unix_ms: u64,
    ) -> Vec<EventState<'a>> {
        (self.content.events.iter())
            .map(|event| EventState {
                key: &event.key,
                name: &event.name,
                mode: self.mode(event, hand),
                running: areas.is_season_active(&event.season, unix_ms),
            })
            .collect()
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct OverridesFile {
    schema_version: u32,
    modes: BTreeMap<String, Mode>,
    /// Event key to the unix ms a lapsing toggle was set at. Its lapse is the event's first
    /// calendar edge after that instant, worked out from the calendar as it is when asked, so
    /// a retuned window or clock offset moves the lapse with it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    since: BTreeMap<String, u64>,
    /// Event key to the lapse as it stood when the toggle was set: for a reader of the file,
    /// and the lapse itself for a file written before `since` existed. A key in neither map
    /// holds until an admin clears it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    until: BTreeMap<String, u64>,
}

impl OverridesFile {
    /// The toggles in force at `unix_ms`, each with its lapse. `edge(key, since)` is the
    /// event's first calendar edge after `since` ([`EventCalendar::next_edge`]).
    fn in_force(self, unix_ms: u64, edge: &dyn Fn(&str, u64) -> Option<u64>) -> Hands {
        let mut hands = Hands::default();
        for (key, mode) in self.modes {
            let lapse = match self.since.get(&key) {
                Some(since) => edge(&key, *since),
                None => self.until.get(&key).copied(),
            };
            if lapse.is_some_and(|at| unix_ms >= at) {
                continue;
            }
            if let Some(at) = lapse {
                hands.until.insert(key.clone(), at);
            }
            hands.modes.insert(key, mode);
        }
        hands
    }
}

/// The hand toggles in force: event key to mode, and when each lapses (unix ms; a key absent
/// from `until` holds until cleared).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hands {
    pub modes: BTreeMap<String, Mode>,
    pub until: BTreeMap<String, u64>,
}

fn read_file(live_dir: &Path) -> io::Result<OverridesFile> {
    let path = live_dir.join(OVERRIDES_FILE);
    let bytes = match read_bounded(&path, MAX_OVERRIDE_BYTES) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(OverridesFile::default());
        }
        other => other?,
    };
    let file: OverridesFile =
        serde_json::from_slice(&bytes).map_err(|error| invalid(&path, error))?;
    let sized = |map: &BTreeMap<String, u64>| map.len() <= MAX_EVENTS;
    if file.schema_version != 1
        || file.modes.len() > MAX_EVENTS
        || !sized(&file.until)
        || !sized(&file.since)
    {
        return Err(invalid(&path, "needs schema_version 1 and at most 64 modes"));
    }
    Ok(file)
}

/// The hand toggles as `live_dir` stores them, lapsed or not: event key to mode. A missing
/// file is none; a malformed, oversized or unreadable one is an error.
pub fn read_overrides(live_dir: &Path) -> io::Result<BTreeMap<String, Mode>> {
    Ok(read_file(live_dir)?.modes)
}

/// The hand toggles of `live_dir` in force at `unix_ms`: a toggle whose lapse has passed is no
/// toggle. `edge(key, since)` is the event's first calendar edge after `since`; callers with a
/// season table use [`ClientAreas::read_hands`]. Errors as [`read_overrides`]; a poll then
/// keeps the toggles it had.
pub fn read_hands(
    live_dir: &Path,
    unix_ms: u64,
    edge: &dyn Fn(&str, u64) -> Option<u64>,
) -> io::Result<Hands> {
    Ok(read_file(live_dir)?.in_force(unix_ms, edge))
}

/// Set (`Some`) or clear (`None`) one event's hand toggle, held until it is cleared. Returns
/// whether the file changed.
pub fn write_override(live_dir: &Path, key: &str, mode: Option<Mode>) -> io::Result<bool> {
    write_toggle(live_dir, key, mode.map(|mode| (mode, None)), 0)
}

/// Set (`Some`: the mode, and the unix ms it lapses at by today's calendar, if ever) or clear
/// (`None`) one event's hand toggle at `unix_ms`. A toggle with a lapse is stored with the
/// instant it was set, and lapses at the event's first calendar edge after that, whatever the
/// calendar says by then; one without holds until cleared. Returns whether the file changed.
///
/// # Errors
/// A file that exists but cannot be read or parsed is left alone and the toggle refused: it
/// may hold other events' toggles, and writing over it would end them.
pub fn write_toggle(
    live_dir: &Path,
    key: &str,
    toggle: Option<(Mode, Option<u64>)>,
    unix_ms: u64,
) -> io::Result<bool> {
    let mut file = read_file(live_dir)?;
    let before = (file.modes.get(key).copied()).map(|mode| (mode, file.until.get(key).copied()));
    // The same toggle again is no write, unless the stored one may have lapsed since.
    if before == toggle && toggle.is_none_or(|(_, until)| until.is_none_or(|at| unix_ms < at)) {
        return Ok(false);
    }
    file.modes.remove(key);
    file.until.remove(key);
    file.since.remove(key);
    if let Some((mode, until)) = toggle {
        file.modes.insert(key.to_owned(), mode);
        if let Some(until) = until {
            file.until.insert(key.to_owned(), until);
            file.since.insert(key.to_owned(), unix_ms);
        }
    }
    file.schema_version = 1;
    let path = live_dir.join(OVERRIDES_FILE);
    let text = serde_json::to_string_pretty(&file).map_err(|error| invalid(&path, error))? + "\n";
    crate::character_store::replace_bytes(&path, text.as_bytes())?;
    Ok(true)
}

/// Watches `event_overrides.json` by its modification time, at most one stat a second.
#[derive(Clone, Debug)]
pub struct HandWatch {
    live_dir: PathBuf,
    /// `None` until the first poll; then the stamp last applied (`None` inside: no file).
    seen: Option<Option<SystemTime>>,
    /// The stamp of a file that could not be read, so its refusal is logged once.
    refused: Option<Option<SystemTime>>,
    /// When the first toggle in force lapses (unix ms): the file is read again then.
    lapse: Option<u64>,
    next_ms: u64,
}

impl HandWatch {
    pub fn new(live_dir: &Path) -> Self {
        Self {
            live_dir: live_dir.to_owned(),
            seen: None,
            refused: None,
            lapse: None,
            next_ms: 0,
        }
    }

    /// The toggles in force when the file changed since the last poll (always on the first
    /// one) or one of them lapsed: the clock passed the event's first calendar edge after the
    /// toggle was set (`edge`, as [`read_hands`]), so the calendar has the event again from
    /// that edge on. The edge is asked of the calendar this process loaded, never trusted from
    /// the file, so a restart over a retuned window lapses the toggle at the new edge. Only a
    /// missing file means "no toggles": one that cannot be read or parsed changes nothing (an
    /// event held by hand must not end, and take journals with it, over a torn write), is not
    /// remembered as seen, and is logged once per version.
    pub fn poll(
        &mut self,
        unix_ms: u64,
        edge: &dyn Fn(&str, u64) -> Option<u64>,
    ) -> Option<BTreeMap<String, Mode>> {
        // A clock that went backwards by more than the poll interval polls at once.
        if unix_ms < self.next_ms && self.next_ms - unix_ms <= POLL_MS {
            return None;
        }
        self.next_ms = unix_ms.saturating_add(POLL_MS);
        let path = self.live_dir.join(OVERRIDES_FILE);
        let stamp = match std::fs::metadata(&path) {
            Ok(meta) => Some(meta.modified().ok()?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return self.refuse(None, &error),
        };
        if self.seen == Some(stamp) && self.lapse.is_none_or(|at| unix_ms < at) {
            return None;
        }
        match read_hands(&self.live_dir, unix_ms, edge) {
            Ok(hands) => {
                self.seen = Some(stamp);
                self.lapse = hands.until.values().copied().min();
                Some(hands.modes)
            }
            Err(error) => self.refuse(stamp, &error),
        }
    }

    fn refuse(&mut self, stamp: Option<SystemTime>, error: &io::Error) -> Option<BTreeMap<String, Mode>> {
        if self.refused != Some(stamp) {
            self.refused = Some(stamp);
            let line = serde_json::json!({
                "event": "event_overrides_refused", "error": error.to_string(),
            });
            println!("{line}");
        }
        None
    }
}
