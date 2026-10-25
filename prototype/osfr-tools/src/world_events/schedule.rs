//! T462: world events, part 3. An event row may carry a `schedule` so the world tick starts it
//! on wall-clock slots instead of by an admin's command.
//!
//! Local time is the unix clock plus `utc_offset_minutes` of `client_areas.json`: one fixed
//! offset for the whole server, no daylight saving (R; Sanctuary's `EventSchedule.lua` reads the
//! host's local clock). A slot starts at every minute of the local day where
//! `(minute - offset_minutes)` is divisible by `slot_minutes` (the count restarts at midnight).
//!
//! A slot is skipped, never queued, when it is out of its `season` (judged at the slot's start,
//! so one that opens just after the season ends neither starts nor announces) or when its start
//! is also a slot start of an event in `skip_if_slot_matches` (R: plain slot arithmetic, as
//! Sanctuary's dance-off against Bruce's 2-hour boundary; the other event's season and run state
//! do not count). Nothing is saved: a restart computes the next slot from the clock, and a slot
//! that fell while the server was down is not run late (R).
use super::{MAX_NAME_BYTES, MAX_TEXT_BYTES, WorldEvent, valid_key};
use crate::client_areas::ClientAreas;
use crate::players::ZoneInstance;
use serde::Deserialize;
use std::sync::Arc;

const MIN_SLOT_MINUTES: u32 = 5;
const MAX_SLOT_MINUTES: u32 = 1440;
const MAX_SKIPS: usize = 8;
const MS_PER_MINUTE: i64 = 60_000;
const DAY_MINUTES: i64 = 1440;
/// Upper bound on slots examined per search: a day is one step while out of season, so this
/// covers ~11 years of closed season and every pattern of overlap skips; a search that runs
/// out ends the event's schedule (logged), never the tick.
const MAX_SEARCH: usize = 4096;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ScheduleRow {
    slot_minutes: u32,
    #[serde(default)]
    offset_minutes: u32,
    season: Option<String>,
    announce_lead_ms: Option<u64>,
    announce: Option<AnnounceRow>,
    #[serde(default)]
    skip_if_slot_matches: Vec<String>,
}

/// The client locale line, copied as text; `locale_id` is its provenance and is never sent.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnnounceRow {
    text: String,
    locale_id: i32,
}

/// Where an event's slots fall in the local day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Slot {
    minutes: i64,
    offset: i64,
}

/// A validated schedule.
#[derive(Debug, PartialEq)]
pub struct Schedule {
    slot: Slot,
    season: Option<String>,
    /// Milliseconds before a slot's start, and the broadcast line.
    announce: Option<(u64, String)>,
    skip_keys: Vec<String>,
}

impl Schedule {
    pub(super) fn compile(row: ScheduleRow, key: &str) -> Result<Self, String> {
        let slot_ms = u64::from(row.slot_minutes) * MS_PER_MINUTE as u64;
        let paired = row.announce_lead_ms.is_some() == row.announce.is_some();
        let announce = match (row.announce_lead_ms, row.announce) {
            (Some(lead), Some(line)) => {
                let text = !line.text.trim().is_empty()
                    && line.text.len() <= MAX_TEXT_BYTES
                    && !line.text.chars().any(char::is_control);
                // Lead < slot keeps the line after the previous slot's start.
                (text && line.locale_id > 0 && (1..slot_ms).contains(&lead))
                    .then_some((lead, line.text))
            }
            _ => None,
        };
        let sane = (MIN_SLOT_MINUTES..=MAX_SLOT_MINUTES).contains(&row.slot_minutes)
            && row.offset_minutes < row.slot_minutes
            // A lead without a line, a line without a lead or an unusable pair is refused.
            && paired
            && announce.is_some() == row.announce_lead_ms.is_some()
            && row
                .season
                .as_deref()
                .is_none_or(|name| (1..=MAX_NAME_BYTES).contains(&name.len()))
            && row.skip_if_slot_matches.len() <= MAX_SKIPS
            && row
                .skip_if_slot_matches
                .iter()
                .all(|other| valid_key(other) && other != key);
        if !sane {
            return Err("schedule needs slot_minutes of 5-1440, offset_minutes below it, a season name of at most 64 bytes, announce_lead_ms (below the slot) and announce {text, locale_id} together, and at most 8 other events to skip".into());
        }
        Ok(Self {
            slot: Slot {
                minutes: i64::from(row.slot_minutes),
                offset: i64::from(row.offset_minutes),
            },
            season: row.season,
            announce,
            skip_keys: row.skip_if_slot_matches,
        })
    }

    /// Every event that carries a schedule, with the slots of the events it skips for. An event
    /// whose skip list names an event without a schedule is left out and logged.
    pub(super) fn entries(events: Vec<WorldEvent>, instance: ZoneInstance) -> Vec<Entry> {
        let slots: Vec<(String, Slot)> = events
            .iter()
            .filter_map(|e| Some((e.key.clone(), e.schedule.as_ref()?.slot)))
            .collect();
        let find = |key: &String| slots.iter().find(|(k, _)| k == key).map(|(_, slot)| *slot);
        let entry = |event: WorldEvent| {
            let schedule = event.schedule.as_ref()?;
            let skips = schedule.skip_keys.iter().map(find).collect::<Option<_>>();
            let Some(skips) = skips else {
                println!(
                    "{}",
                    serde_json::json!({"event": "world_event_schedule_refused", "key": event.key,
                        "reason": "skip_if_slot_matches names an event without a schedule"})
                );
                return None;
            };
            Some(Entry {
                event: Arc::new(event),
                instance,
                skips,
                next: None,
                announced: false,
            })
        };
        events.into_iter().filter_map(entry).collect()
    }
}

impl Slot {
    /// The local minute (counted from the unix epoch) that `unix_ms` falls in.
    fn local_minute(unix_ms: u64, utc_offset: i32) -> i64 {
        (unix_ms / MS_PER_MINUTE as u64) as i64 + i64::from(utc_offset)
    }

    fn unix_ms(local_minute: i64, utc_offset: i32) -> Option<u64> {
        let minute = local_minute.checked_sub(i64::from(utc_offset))?;
        u64::try_from(minute.checked_mul(MS_PER_MINUTE)?).ok()
    }

    /// Whether the local minute is a slot start.
    fn starts_at(self, local_minute: i64) -> bool {
        (local_minute.rem_euclid(DAY_MINUTES) - self.offset).rem_euclid(self.minutes) == 0
    }

    /// The first slot start strictly after `after_ms`, in unix ms.
    fn next_start(self, after_ms: u64, utc_offset: i32) -> Option<u64> {
        let from = Self::local_minute(after_ms, utc_offset) + 1;
        let (day, minute) = (from.div_euclid(DAY_MINUTES), from.rem_euclid(DAY_MINUTES));
        let first = match minute <= self.offset {
            true => self.offset,
            false => {
                self.offset
                    + (minute - self.offset + self.minutes - 1) / self.minutes * self.minutes
            }
        };
        let local = match first < DAY_MINUTES {
            true => day * DAY_MINUTES + first,
            false => (day + 1) * DAY_MINUTES + self.offset,
        };
        Self::unix_ms(local, utc_offset)
    }

    /// The latest slot start at or before `at_ms`, in unix ms.
    fn latest_start(self, at_ms: u64, utc_offset: i32) -> Option<u64> {
        let from = Self::local_minute(at_ms, utc_offset);
        let (day, minute) = (from.div_euclid(DAY_MINUTES), from.rem_euclid(DAY_MINUTES));
        let local = match minute < self.offset {
            true => {
                let last =
                    self.offset + (DAY_MINUTES - 1 - self.offset) / self.minutes * self.minutes;
                (day - 1) * DAY_MINUTES + last
            }
            false => {
                day * DAY_MINUTES
                    + self.offset
                    + (minute - self.offset) / self.minutes * self.minutes
            }
        };
        Self::unix_ms(local, utc_offset)
    }
}

/// One scheduled event and where its next slot stands. Built by `Schedule::entries`, advanced
/// by `WorldEvents::tick`.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Entry {
    pub(super) event: Arc<WorldEvent>,
    /// Where the event runs and its line is heard.
    pub(super) instance: ZoneInstance,
    /// Slots of the events this one skips for.
    skips: Vec<Slot>,
    /// The next slot that will start (unix ms); `None` when no valid slot is in reach.
    next: Option<u64>,
    /// Whether this slot's line was sent, or its moment had passed when the slot was chosen.
    announced: bool,
}

/// What `Entry::step` wants done now.
#[derive(Debug, PartialEq)]
pub(super) enum Due {
    Idle,
    Announce(String),
    Start,
}

impl Entry {
    fn schedule(&self) -> &Schedule {
        self.event
            .schedule
            .as_ref()
            .expect("entries carry a schedule")
    }

    /// Whether the slot starting at `start_ms` runs: in season, and not on a skipped event's start.
    fn runs_at(&self, start_ms: u64, areas: &ClientAreas) -> bool {
        let offset = areas.utc_offset_minutes();
        let season = self.schedule().season.as_deref();
        season.is_none_or(|name| areas.is_season_active(name, start_ms))
            && !self
                .skips
                .iter()
                .any(|slot| slot.starts_at(Slot::local_minute(start_ms, offset)))
    }

    /// The first slot after `after_ms` that runs.
    fn next_running(&self, mut after_ms: u64, areas: &ClientAreas) -> Option<u64> {
        let offset = areas.utc_offset_minutes();
        let slot = self.schedule().slot;
        for _ in 0..MAX_SEARCH {
            let start = slot.next_start(after_ms, offset)?;
            if self.runs_at(start, areas) {
                return Some(start);
            }
            // Out of season, the rest of the local day is too (seasons are whole dates).
            let out = self
                .schedule()
                .season
                .as_deref()
                .is_some_and(|name| !areas.is_season_active(name, start));
            after_ms = match out {
                true => {
                    let next_day = (Slot::local_minute(start, offset).div_euclid(DAY_MINUTES) + 1)
                        * DAY_MINUTES;
                    Slot::unix_ms(next_day, offset)?.saturating_sub(1)
                }
                false => start,
            };
        }
        None
    }

    /// Choose the next slot after `after_ms`; its line counts as sent when `now_ms` is already
    /// past the moment it was due (a restart inside the lead does not announce late).
    fn choose(&mut self, after_ms: u64, now_ms: u64, areas: &ClientAreas) {
        self.next = self.next_running(after_ms, areas);
        let lead = self.schedule().announce.as_ref().map(|(lead, _)| *lead);
        self.announced = match (self.next, lead) {
            (Some(next), Some(lead)) => now_ms.saturating_add(lead) >= next,
            _ => true,
        };
    }

    /// Fresh from the clock: nothing that fell before `now_ms` runs.
    pub(super) fn begin(&mut self, now_ms: u64, areas: &ClientAreas) {
        self.choose(now_ms, now_ms, areas);
        if self.next.is_none() {
            println!(
                "{}",
                serde_json::json!({"event": "world_event_never_scheduled", "key": self.event.key})
            );
        }
    }

    /// The unix ms this entry next needs the tick: its line's moment, else its slot's start.
    pub(super) fn wake_ms(&self) -> u64 {
        let lead = self.schedule().announce.as_ref().map(|(lead, _)| *lead);
        match (self.next, lead) {
            (Some(next), Some(lead)) if !self.announced => next.saturating_sub(lead),
            (Some(next), _) => next,
            (None, _) => u64::MAX,
        }
    }

    /// Advance to `now_ms`. Once a slot is due only the latest slot at or before `now_ms` can
    /// start (a clock that jumped forward past several runs the last, never each); the next
    /// slot is then chosen after `now_ms`, so a clock that jumps back repeats nothing.
    pub(super) fn step(&mut self, now_ms: u64, areas: &ClientAreas) -> Due {
        let Some(next) = self.next else {
            return Due::Idle;
        };
        if now_ms >= next {
            let latest = self
                .schedule()
                .slot
                .latest_start(now_ms, areas.utc_offset_minutes());
            let start = latest.is_some_and(|slot| self.runs_at(slot, areas));
            self.choose(now_ms, now_ms, areas);
            return if start { Due::Start } else { Due::Idle };
        }
        match &self.schedule().announce {
            Some((lead, line)) if !self.announced && now_ms.saturating_add(*lead) >= next => {
                let line = line.clone();
                self.announced = true;
                Due::Announce(line)
            }
            _ => Due::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{FILE, WorldEvents, load, load_scheduled};
    use super::*;
    use crate::players::{InterestPolicy, PlayerRegistry, SessionId};
    use serde_json::{Value, json};
    use std::path::{Path, PathBuf};

    const WORLD: ZoneInstance = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };
    const MINUTE: u64 = 60_000;
    const HOUR: u64 = 60 * MINUTE;
    const DANCE_LINE: &str = "The dance-off starts in two minutes!";

    /// Unix ms of a UTC date and time, by Howard Hinnant's public-domain `days_from_civil`.
    fn utc(year: i64, month: i64, day: i64, hour: u64, minute: u64) -> u64 {
        let year = year - i64::from(month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let shifted = (month + 9) % 12;
        let day_of_year = (153 * shifted + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        let days = era * 146_097 + day_of_era - 719_468;
        days as u64 * 24 * HOUR + hour * HOUR + minute * MINUTE
    }

    fn scratch(name: &str) -> PathBuf {
        let thread = format!("{:?}", std::thread::current().id()).replace(['(', ')'], "");
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("osfr-t462-{pid}-{thread}-{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The shipped Halloween season (September 20 to November 11) at a fixed UTC offset.
    fn areas(utc_offset_minutes: i32) -> ClientAreas {
        let dir = scratch("areas");
        let file = json!({"schema_version": 1, "utc_offset_minutes": utc_offset_minutes,
            "switchable": {"area_ids": [3992], "source": "T462 test"},
            "seasons": [{"name": "Halloween", "start_month": 9, "start_day": 20,
                "end_month": 11, "end_day": 11, "area_ids": [3992], "source": "T462 test"}]});
        std::fs::write(dir.join("client_areas.json"), file.to_string()).unwrap();
        let areas = ClientAreas::load(&dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        areas
    }

    fn row(key: &str, schedule: Value) -> Value {
        json!({"key": key, "schedule": schedule,
            "actors": [{"key": "a", "model_id": 1, "name_string_id": 0, "npc_name": "Event NPC",
                "position": [0.0, 0.0, 0.0], "heading": 0.0, "stand_animation_id": 1,
                "hide_nameplate": false, "source": "T462 test fixture"}],
            "timeline": [{"at_ms": 0, "op": {"spawn": {"actor": "a"}}}]})
    }

    fn write(name: &str, rows: Vec<Value>) -> PathBuf {
        let dir = scratch(name);
        std::fs::write(dir.join(FILE), json!({"events": rows}).to_string()).unwrap();
        dir
    }

    fn scheduled(name: &str, rows: Vec<Value>) -> Vec<WorldEvent> {
        let dir = write(name, rows);
        let events = load_scheduled(&dir);
        let _ = std::fs::remove_dir_all(&dir);
        events.unwrap()
    }

    /// Bruce's concert shape: every 2 hours from local midnight.
    fn bruce() -> Value {
        row("bruce", json!({"slot_minutes": 120}))
    }

    /// The dance-off shape: every 30 minutes in season, a line 2 minutes before, never on
    /// Bruce's boundary.
    fn dance() -> Value {
        row(
            "dance",
            json!({"slot_minutes": 30, "season": "Halloween", "announce_lead_ms": 2 * MINUTE,
                "announce": {"text": DANCE_LINE, "locale_id": 432_430},
                "skip_if_slot_matches": ["bruce"]}),
        )
    }

    /// `dance` with nothing to skip, so it can be scheduled without Bruce.
    fn dance_alone() -> Value {
        let mut dance = dance();
        dance["schedule"]["skip_if_slot_matches"] = json!([]);
        dance
    }

    struct Rig {
        events: WorldEvents,
        players: PlayerRegistry,
        areas: ClientAreas,
        /// The server's own monotonic clock, which only moves forward.
        mono: u64,
    }

    /// What a drive saw: (unix ms of the tick, event key) for each start, and the unix ms of
    /// each kickoff line.
    #[derive(Default, Debug, PartialEq)]
    struct Seen {
        starts: Vec<(u64, String)>,
        lines: Vec<u64>,
    }

    fn players() -> PlayerRegistry {
        let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
        let near = [1.0, 0.0, 0.0];
        players.join(SessionId(1), 1, 1, WORLD, near, 0.0).unwrap();
        players
    }

    impl Rig {
        fn new(rows: Vec<Value>, now: u64, utc_offset_minutes: i32) -> Self {
            let areas = areas(utc_offset_minutes);
            let mut events = WorldEvents::default();
            events.schedule(scheduled("rig", rows), WORLD, now, &areas);
            Self {
                events,
                players: players(),
                areas,
                mono: 1_000,
            }
        }

        /// One tick at wall-clock `unix_ms`.
        fn tick(&mut self, unix_ms: u64, seen: &mut Seen) {
            self.mono += 1;
            let before: Vec<String> = self.events.runs.keys().cloned().collect();
            let areas = &mut self.areas;
            let out = self.events.tick(self.mono, unix_ms, &self.players, areas);
            let line = crate::chat::system(DANCE_LINE);
            let lines = out.iter().filter(|(_, packet)| *packet == line);
            seen.lines.extend(lines.map(|_| unix_ms));
            let started: Vec<String> = self
                .events
                .runs
                .keys()
                .filter(|key| !before.contains(key))
                .cloned()
                .collect();
            for key in started {
                // Stopped at once, so the same key can start again in a later slot.
                self.events.stop(&key);
                seen.starts.push((unix_ms, key));
            }
        }

        /// Ticks every `step` ms from `from` up to and including `to`.
        fn drive(&mut self, from: u64, to: u64, step: u64) -> Seen {
            let mut seen = Seen::default();
            for at in (from..=to).step_by(step as usize) {
                self.tick(at, &mut seen);
            }
            seen
        }
    }

    fn starts(seen: &Seen, key: &str) -> Vec<u64> {
        let hits = seen.starts.iter().filter(|(_, k)| k == key);
        hits.map(|(at, _)| *at).collect()
    }

    /// Whole minutes after `day`.
    fn minutes(day: u64, at: &[u64]) -> Vec<u64> {
        at.iter().map(|t| (t - day) / MINUTE).collect()
    }

    /// 1. Slot math against a minute-by-minute scan, across midnight, with offsets, with the
    /// utc offset, and without a panic at the ends of the clock.
    #[test]
    fn slots_follow_the_local_day_across_midnight_offsets_and_utc() {
        let day = utc(2026, 10, 21, 0, 0);
        let shapes = [
            (120, 0, 0),
            (30, 0, 0),
            (120, 30, -90),
            (7, 3, 330),
            (1440, 0, -720),
            (45, 44, 840),
        ];
        for (minutes, offset, utc_offset) in shapes {
            let slot = Slot { minutes, offset };
            let scan = |from: u64, forward: bool| {
                let minute = (from / MINUTE) as i64;
                let at = |d: i64| if forward { minute + 1 + d } else { minute - d };
                let hit = |m: &i64| slot.starts_at(m + i64::from(utc_offset));
                (0..3 * 1440).map(at).find(hit).map(|m| m as u64 * MINUTE)
            };
            for probe in (0..2 * 1440).step_by(13) {
                let at = day + probe * MINUTE + 17_000;
                let label = format!("{minutes}/{offset}/{utc_offset} at {probe}");
                assert_eq!(slot.next_start(at, utc_offset), scan(at, true), "{label}");
                assert_eq!(
                    slot.latest_start(at, utc_offset),
                    scan(at, false),
                    "{label}"
                );
            }
        }
        // 23:59:30 local is followed by 00:00 the next day, and the offset moves the slots.
        let slot = Slot {
            minutes: 120,
            offset: 0,
        };
        let late = day + 23 * HOUR + 59 * MINUTE + 30_000;
        assert_eq!(slot.next_start(late, 0), Some(day + 24 * HOUR));
        let shifted = Slot {
            minutes: 120,
            offset: 30,
        };
        assert_eq!(shifted.next_start(day, -90), Some(day + 2 * HOUR));
        // Negative offsets near the epoch and the far end of the clock never panic.
        let _ = (slot.next_start(0, -720), slot.latest_start(0, -720));
        assert_eq!(slot.next_start(u64::MAX, 840), None);
        assert_eq!(slot.latest_start(u64::MAX, -720), None);
    }

    /// 2. The season gate, judged at the slot's start: on in the season's last local minute, off
    /// the day after, and the utc offset moves the date.
    #[test]
    fn a_seasonal_event_starts_in_season_and_not_after() {
        let seasonal = || {
            vec![row(
                "dance",
                json!({"slot_minutes": 30, "season": "Halloween"}),
            )]
        };
        let day = utc(2026, 10, 21, 0, 0);
        let mut rig = Rig::new(seasonal(), day, 0);
        let seen = rig.drive(day, day + 2 * HOUR, 10_000);
        let expected = [30, 60, 90, 120].map(|m| day + m * MINUTE);
        assert_eq!(starts(&seen, "dance"), expected);
        let after = utc(2026, 11, 12, 0, 0);
        let mut rig = Rig::new(seasonal(), after, 0);
        assert!(
            rig.drive(after, after + 24 * HOUR, MINUTE)
                .starts
                .is_empty()
        );
        // 23:30 UTC on November 11 is the last in-season slot; midnight is already the 12th.
        let edge = utc(2026, 11, 11, 23, 0);
        let mut rig = Rig::new(seasonal(), edge, 0);
        let seen = rig.drive(edge, edge + 2 * HOUR, 30_000);
        assert_eq!(starts(&seen, "dance"), [edge + 30 * MINUTE]);
        // At UTC+2 that instant is 01:00 on the 12th: nothing. Two hours earlier is 23:00 on the
        // 11th, so the slots at 23:30 local and 00:00 local (the 12th) split on the season.
        let mut rig = Rig::new(seasonal(), edge, 120);
        assert!(rig.drive(edge, edge + HOUR, 30_000).starts.is_empty());
        let mut rig = Rig::new(seasonal(), edge - 2 * HOUR, 120);
        let seen = rig.drive(edge - 2 * HOUR, edge - HOUR, 30_000);
        assert_eq!(starts(&seen, "dance"), [edge - 90 * MINUTE]);
    }

    /// 3 and 4. A dance-off slot on Bruce's boundary is skipped and the next runs; the line goes
    /// out once, two minutes before each slot that runs, and never for the skipped one.
    #[test]
    fn an_overlapping_slot_is_skipped_with_no_line_and_the_next_runs() {
        let day = utc(2026, 10, 21, 0, 0);
        let mut rig = Rig::new(vec![bruce(), dance()], day, 0);
        let seen = rig.drive(day, day + 4 * HOUR, 10_000);
        assert_eq!(minutes(day, &starts(&seen, "bruce")), [120, 240]);
        let dance_starts = minutes(day, &starts(&seen, "dance"));
        assert_eq!(dance_starts, [30, 60, 90, 150, 180, 210]);
        // One line per slot that runs, at its start minus the lead, to the zone's session.
        assert_eq!(minutes(day, &seen.lines), [28, 58, 88, 148, 178, 208]);
        assert!(seen.lines.iter().all(|t| (t - day) % MINUTE == 0));
        // Out of season the line stays quiet too, and Bruce has no season.
        let off = utc(2026, 11, 12, 0, 0);
        let mut rig = Rig::new(vec![bruce(), dance()], off, 0);
        let seen = rig.drive(off, off + 4 * HOUR, 10_000);
        assert!(seen.lines.is_empty() && starts(&seen, "dance").is_empty());
        assert_eq!(starts(&seen, "bruce").len(), 2);
    }

    /// 5. A clock that jumps back never starts or announces anything twice; one that jumps
    /// forward past several slots starts only the latest due slot, and not at all when that one
    /// is skipped.
    #[test]
    fn clock_jumps_repeat_nothing_and_start_only_the_latest_due_slot() {
        let day = utc(2026, 10, 21, 0, 0);
        let mut rig = Rig::new(vec![bruce(), dance()], day, 0);
        let mut seen = Seen::default();
        rig.tick(day + 28 * MINUTE, &mut seen);
        rig.tick(day + 30 * MINUTE + 1_000, &mut seen);
        assert_eq!((seen.lines.len(), seen.starts.len()), (1, 1));
        // Back to before the line and the start, then forward again: nothing repeats.
        let back = [5 * MINUTE, 29 * MINUTE, 30 * MINUTE + 500, 15 * MINUTE];
        for at in back {
            rig.tick(day + at, &mut seen);
        }
        rig.tick(day + 31 * MINUTE, &mut seen);
        assert_eq!((seen.lines.len(), seen.starts.len()), (1, 1));
        // Forward past 01:00, 01:30 and 02:00 at once. The latest due dance slot is 02:00,
        // Bruce's boundary, which dance skips, so the earlier ones do not run late; Bruce's
        // own slot starts once.
        rig.tick(day + 125 * MINUTE, &mut seen);
        assert_eq!(starts(&seen, "bruce"), [day + 125 * MINUTE]);
        assert_eq!((starts(&seen, "dance").len(), seen.lines.len()), (1, 1));
        // Forward past 02:30, 03:00 and 03:30: only the latest (03:30) starts, with no late
        // lines for the passed slots, and the passed slots are not queued behind it.
        rig.tick(day + 215 * MINUTE, &mut seen);
        let dance_starts = starts(&seen, "dance");
        assert_eq!(
            dance_starts,
            [day + 30 * MINUTE + 1_000, day + 215 * MINUTE]
        );
        rig.tick(day + 216 * MINUTE, &mut seen);
        assert_eq!((seen.starts.len(), seen.lines.len()), (3, 1));
    }

    /// 6. With nothing scheduled the tick does nothing and changes nothing; an entry waits for
    /// its moment without being touched.
    #[test]
    fn nothing_scheduled_is_dormant() {
        let day = utc(2026, 10, 21, 0, 0);
        let mut events = WorldEvents::default();
        let out = events.tick(1, day, &players(), &mut areas(0));
        assert!(out.is_empty());
        assert_eq!(events, WorldEvents::default());
        let mut rig = Rig::new(vec![dance_alone()], day, 0);
        let (before, wake) = (rig.events.schedule.clone(), rig.events.wake_ms);
        assert_eq!(wake, day + 28 * MINUTE, "the line's moment is the earliest");
        let mut seen = Seen::default();
        rig.tick(day + 10 * MINUTE, &mut seen);
        assert_eq!((rig.events.schedule, rig.events.wake_ms), (before, wake));
        assert_eq!(seen, Seen::default());
    }

    /// Nothing is saved: a restart computes the next slot from the clock, and a slot that fell
    /// while down (or a line whose moment passed) is not run late.
    #[test]
    fn a_restart_computes_the_next_slot_from_the_clock() {
        let day = utc(2026, 10, 21, 0, 0);
        let mut rig = Rig::new(vec![dance_alone()], day + 10 * MINUTE, 0);
        let seen = rig.drive(day + 10 * MINUTE, day + 31 * MINUTE, 10_000);
        let at_half = vec![day + 30 * MINUTE];
        assert_eq!(starts(&seen, "dance"), at_half, "00:00 fell while down");
        // Restarted at 00:29, inside the lead: the line's moment passed, the slot still runs.
        let mut rig = Rig::new(vec![dance_alone()], day + 29 * MINUTE, 0);
        let seen = rig.drive(day + 29 * MINUTE, day + 31 * MINUTE, 10_000);
        assert_eq!((seen.lines.len(), starts(&seen, "dance")), (0, at_half));
        // Restarted exactly at a slot start: that slot is not run.
        let mut rig = Rig::new(vec![bruce()], day + 2 * HOUR, 0);
        let seen = rig.drive(day + 2 * HOUR, day + 3 * HOUR, 10_000);
        assert!(seen.starts.is_empty());
    }

    /// An event that is already running (an admin's start) is skipped, not queued, and a run in
    /// one event never stops another's schedule.
    #[test]
    fn a_running_event_is_skipped_and_never_blocks_another_schedule() {
        let day = utc(2026, 10, 21, 0, 0);
        let show = row("show", json!({"slot_minutes": 120}));
        let mut rig = Rig::new(vec![bruce(), show], day + MINUTE, 0);
        let admin = scheduled("admin", vec![bruce()]).pop().unwrap();
        assert!(rig.events.start(admin, WORLD, 1));
        let serial = rig.events.serial;
        let mut seen = Seen::default();
        rig.tick(day + 2 * HOUR, &mut seen);
        assert_eq!(seen.starts, [(day + 2 * HOUR, "show".to_string())]);
        assert_eq!(rig.events.serial, serial + 1, "only show took a serial");
        // Stopping the admin's run does not start the slot that was skipped.
        rig.events.stop("bruce");
        rig.tick(day + 2 * HOUR + MINUTE, &mut seen);
        assert!(!rig.events.runs.contains_key("bruce"));
    }

    /// One broken row never stops another event's schedule; the strict loader still refuses the
    /// file, and a skip that names an unscheduled event leaves only that event unscheduled.
    #[test]
    fn one_broken_event_leaves_the_other_schedules_running() {
        let mut broken = row("broken", json!({"slot_minutes": 30}));
        broken["timeline"] = json!([{"at_ms": 0, "op": {"spawn": {"actor": "nobody"}}}]);
        let mut plain = row("plain", json!(null));
        plain.as_object_mut().unwrap().remove("schedule");
        let skipping = json!({"slot_minutes": 30, "skip_if_slot_matches": ["plain"]});
        let rows = vec![broken, plain, row("nowhere", skipping), bruce()];
        let dir = write("broken", rows.clone());
        assert!(load(&dir).is_err(), "the strict loader refuses the file");
        let loaded = scheduled("broken2", rows);
        let keys: Vec<_> = loaded.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(
            keys,
            ["nowhere", "bruce"],
            "only well-formed scheduled rows"
        );
        let day = utc(2026, 10, 21, 0, 0);
        let mut events = WorldEvents::default();
        events.schedule(loaded, WORLD, day + MINUTE, &areas(0));
        let kept: Vec<_> = events.schedule.iter().map(|e| &*e.event.key).collect();
        assert_eq!(kept, ["bruce"], "nowhere names an event without a schedule");
        let mut rig = Rig {
            events,
            players: players(),
            areas: areas(0),
            mono: 0,
        };
        let seen = rig.drive(day + MINUTE, day + 2 * HOUR, MINUTE);
        assert_eq!(starts(&seen, "bruce"), [day + 2 * HOUR]);
    }

    /// Every schedule field is bounded at load.
    #[test]
    fn the_loader_refuses_schedules_outside_the_bounds() {
        let ok = |schedule: Value| load(&write("bounds", vec![row("show", schedule)])).is_ok();
        let line = json!({"text": "Soon", "locale_id": 1});
        let announced =
            |lead: u64| json!({"slot_minutes": 30, "announce_lead_ms": lead, "announce": line});
        assert!(ok(json!({"slot_minutes": 5})));
        assert!(ok(json!({"slot_minutes": 1440, "offset_minutes": 1439})));
        assert!(ok(announced(1_799_999)));
        let blank = json!({"text": " ", "locale_id": 1});
        let no_id = json!({"text": "Soon", "locale_id": 0});
        let nine = ["a", "b", "c", "d", "e", "f", "g", "h", "i"];
        let cases = [
            ("slot 4", json!({"slot_minutes": 4})),
            ("slot 1441", json!({"slot_minutes": 1441})),
            (
                "offset = slot",
                json!({"slot_minutes": 30, "offset_minutes": 30}),
            ),
            ("lead = slot", announced(1_800_000)),
            ("zero lead", announced(0)),
            (
                "lead alone",
                json!({"slot_minutes": 30, "announce_lead_ms": 1000}),
            ),
            ("line alone", json!({"slot_minutes": 30, "announce": line})),
            (
                "blank line",
                json!({"slot_minutes": 30, "announce_lead_ms": 1000, "announce": blank}),
            ),
            (
                "no locale id",
                json!({"slot_minutes": 30, "announce_lead_ms": 1000, "announce": no_id}),
            ),
            (
                "skips itself",
                json!({"slot_minutes": 30, "skip_if_slot_matches": ["show"]}),
            ),
            (
                "bad skip key",
                json!({"slot_minutes": 30, "skip_if_slot_matches": ["Bruce"]}),
            ),
            (
                "nine skips",
                json!({"slot_minutes": 30, "skip_if_slot_matches": nine}),
            ),
            ("empty season", json!({"slot_minutes": 30, "season": ""})),
            ("no slot", json!({"offset_minutes": 3})),
            ("unknown field", json!({"slot_minutes": 30, "every": 3})),
        ];
        for (label, bad) in cases {
            assert!(!ok(bad), "{label} was accepted");
        }
        // The shipped file is still valid; its scheduled event (T464) is not dropped.
        let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let keys: Vec<_> = load_scheduled(&shipped)
            .unwrap()
            .into_iter()
            .map(|e| e.key)
            .collect();
        assert_eq!(keys, ["halloween_dance_off"]);
    }
}
