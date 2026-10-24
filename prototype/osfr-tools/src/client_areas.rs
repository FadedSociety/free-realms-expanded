//! T457: opcode 89 `PacketSetClientArea`, the server's switch for the client areas that
//! FabledRealmsAreas.xml ships with `AreaEnabled="0"`: always-on areas, month/day seasons and
//! runtime event toggles. Content: `client_areas.json`; a missing file means no areas.
//!
//! Wire (client): case 89 of dispatcher 0x009471a0 (0x00948821) reads through 0x008f6c70 with
//! the trailing-bytes flag set, so reader 0x008d5c60 takes u16 opcode, i32 area id, u8 bool and
//! i32 elapsed. An area the client does not know is dropped (0x008b80a0). 0x00bead80 applies
//! the state only when it differs from the current one, and on enable sets
//! start = timeGetTime() - elapsed (milliseconds, 0x007027f0).

use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const OPCODE: u16 = 89;
const MAX_AREA_IDS: usize = 256;
/// One bit per season in `ClientAreas::active`.
const MAX_SEASONS: usize = 64;
const DAYS_IN_MONTH: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Content {
    schema_version: u32,
    /// (R) One offset for the whole server; the season dates are local dates.
    #[serde(default)]
    utc_offset_minutes: i32,
    /// Every `AreaEnabled="0"` area in FabledRealmsAreas.xml: the only ids the server switches.
    switchable: Switchable,
    #[serde(default)]
    always_on: Vec<AlwaysOn>,
    #[serde(default)]
    seasons: Vec<Season>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Switchable {
    area_ids: Vec<i32>,
    source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AlwaysOn {
    area_id: i32,
    source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Season {
    name: String,
    start_month: u8,
    start_day: u8,
    end_month: u8,
    end_day: u8,
    area_ids: Vec<i32>,
    source: String,
}

impl Season {
    /// Inclusive on both days; a start after the end wraps the year end.
    fn contains(&self, date: (u8, u8)) -> bool {
        let start = (self.start_month, self.start_day);
        let end = (self.end_month, self.end_day);
        if start <= end {
            start <= date && date <= end
        } else {
            date >= start || date <= end
        }
    }
}

fn valid_date(month: u8, day: u8) -> bool {
    (1..=12).contains(&month) && (1..=DAYS_IN_MONTH[usize::from(month) - 1]).contains(&day)
}

/// Positive, unique, at most `MAX_AREA_IDS`, and (when `within` is given) switchable.
fn valid_ids(ids: &[i32], within: Option<&[i32]>) -> bool {
    let mut seen = BTreeSet::new();
    ids.len() <= MAX_AREA_IDS
        && ids.iter().all(|id| {
            *id > 0 && seen.insert(*id) && within.is_none_or(|within| within.contains(id))
        })
}

impl Content {
    fn validate(&self) -> Result<(), &'static str> {
        let switchable = self.switchable.area_ids.as_slice();
        let mut names = BTreeSet::new();
        let always_on = self
            .always_on
            .iter()
            .map(|row| row.area_id)
            .collect::<Vec<_>>();
        if self.schema_version != 1 {
            Err("schema_version must be 1")
        } else if !(-720..=840).contains(&self.utc_offset_minutes) {
            Err("utc_offset_minutes must be in -720..=840")
        } else if !valid_ids(switchable, None) || self.switchable.source.trim().is_empty() {
            Err("switchable needs at most 256 unique positive area ids and a source")
        } else if !valid_ids(&always_on, Some(switchable))
            || self
                .always_on
                .iter()
                .any(|row| row.source.trim().is_empty())
        {
            Err("always_on rows need a unique switchable area id and a source")
        } else if self.seasons.len() > MAX_SEASONS {
            Err("at most 64 seasons")
        } else if !self.seasons.iter().all(|season| {
            !season.name.trim().is_empty()
                && names.insert(season.name.as_str())
                && valid_date(season.start_month, season.start_day)
                && valid_date(season.end_month, season.end_day)
                && !season.area_ids.is_empty()
                && valid_ids(&season.area_ids, Some(switchable))
                && !season.source.trim().is_empty()
        }) {
            Err("seasons need a unique name, valid dates, unique switchable area ids and a source")
        } else {
            Ok(())
        }
    }
}

/// The wall clock the season dates and event toggles use, in unix milliseconds.
pub fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

/// Client wire for one area: 11 bytes, reader 0x008d5c60. Elapsed past i32 saturates.
fn packet(area_id: i32, enabled: bool, elapsed_ms: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(11);
    out.extend(OPCODE.to_le_bytes());
    out.extend(area_id.to_le_bytes());
    out.push(u8::from(enabled));
    out.extend(i32::try_from(elapsed_ms).unwrap_or(i32::MAX).to_le_bytes());
    out
}

/// Nothing here is saved: a restart or reconnect recomputes from the clock, and event toggles
/// belong to the event that set them.
#[derive(Clone, Debug, Default)]
pub struct ClientAreas {
    content: Arc<Content>,
    /// Event toggles: area id to the unix ms it was switched on.
    toggles: BTreeMap<i32, u64>,
    /// Seasons active at the last `season_changes` evaluation, one bit per season index.
    active: u64,
    /// Unix minute of that evaluation; `None` until the first call records the baseline.
    checked_minute: Option<u64>,
}

impl ClientAreas {
    /// Reads `client_areas.json` from `directory`. A missing file means no areas; a malformed
    /// one is an error naming the file.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join("client_areas.json");
        let named = |error: &dyn std::fmt::Display| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {error}", path.display()),
            )
        };
        let bytes = match std::fs::read(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(named(&error)),
            Ok(bytes) => bytes,
        };
        let content: Content = serde_json::from_slice(&bytes).map_err(|error| named(&error))?;
        content.validate().map_err(|error| named(&error))?;
        Ok(Self {
            content: Arc::new(content),
            ..Self::default()
        })
    }

    /// (month, day) of `unix_ms` after the UTC offset, by Howard Hinnant's public-domain
    /// `civil_from_days`.
    fn month_day(&self, unix_ms: u64) -> (u8, u8) {
        // u64::MAX / 60_000 fits an i64, so neither the cast nor the offset can overflow.
        let minutes = (unix_ms / 60_000) as i64 + i64::from(self.content.utc_offset_minutes);
        let day_of_era = (minutes.div_euclid(1440) + 719_468).rem_euclid(146_097);
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
        let month = if shifted_month < 10 {
            shifted_month + 3
        } else {
            shifted_month - 9
        };
        (month as u8, day as u8)
    }

    fn active_mask(&self, unix_ms: u64) -> u64 {
        let date = self.month_day(unix_ms);
        self.content
            .seasons
            .iter()
            .enumerate()
            .filter(|(_, season)| season.contains(date))
            .fold(0, |mask, (index, _)| mask | 1 << index)
    }

    /// The fixed offset local time uses (no daylight saving).
    pub fn utc_offset_minutes(&self) -> i32 {
        self.content.utc_offset_minutes
    }

    /// Unknown season names are never active.
    pub fn is_season_active(&self, name: &str, unix_ms: u64) -> bool {
        let date = self.month_day(unix_ms);
        self.content
            .seasons
            .iter()
            .any(|season| season.name == name && season.contains(date))
    }

    /// Areas held on by always-on rows, the seasons in `seasons`, or an event toggle.
    fn held(&self, seasons: u64) -> BTreeSet<i32> {
        let mut held = self
            .content
            .always_on
            .iter()
            .map(|row| row.area_id)
            .collect::<BTreeSet<_>>();
        for (index, season) in self.content.seasons.iter().enumerate() {
            if seasons & 1 << index != 0 {
                held.extend(&season.area_ids);
            }
        }
        held.extend(self.toggles.keys());
        held
    }

    /// One enable per held area for a client that just loaded the world. Always-on and season
    /// areas start at elapsed 0 (R); a toggled area reports the time since its event began.
    pub fn world_ready_packets(&self, unix_ms: u64) -> Vec<Vec<u8>> {
        self.held(self.active_mask(unix_ms))
            .into_iter()
            .map(|id| {
                let since = self
                    .toggles
                    .get(&id)
                    .map_or(0, |at| unix_ms.saturating_sub(*at));
                packet(id, true, since)
            })
            .collect()
    }

    /// An event switches `area_id`. Returns the packet every loaded client needs, or `None` when
    /// clients see no change: an id that is not switchable (state untouched), a repeat, or an
    /// area an always-on row or an active season still holds (the toggle is still recorded).
    pub fn set_area(&mut self, area_id: i32, on: bool, now_ms: u64) -> Option<Vec<u8>> {
        let switchable = self.content.switchable.area_ids.contains(&area_id);
        if !switchable || self.toggles.contains_key(&area_id) == on {
            return None;
        }
        let seasons = self.active_mask(now_ms);
        let shown = self.held(seasons).contains(&area_id);
        if on {
            self.toggles.insert(area_id, now_ms);
        } else {
            self.toggles.remove(&area_id);
        }
        (self.held(seasons).contains(&area_id) != shown).then(|| packet(area_id, on, 0))
    }

    /// Enable/disable packets for areas whose season state changed since the last call.
    /// Evaluates at most once per wall-clock minute; the first call only records the baseline
    /// (clients that loaded before it got the clock's areas from `world_ready_packets`). A clock
    /// that moves backwards is just another date, so an area is never enabled twice in a row.
    /// No seasons: no work and no allocation.
    pub fn season_changes(&mut self, unix_ms: u64) -> Vec<Vec<u8>> {
        let minute = unix_ms / 60_000;
        if self.content.seasons.is_empty() || self.checked_minute == Some(minute) {
            return Vec::new();
        }
        let first = self.checked_minute.replace(minute).is_none();
        let active = self.active_mask(unix_ms);
        if first || active == self.active {
            self.active = active;
            return Vec::new();
        }
        let (before, after) = (self.held(self.active), self.held(active));
        self.active = active;
        before
            .difference(&after)
            .map(|id| packet(*id, false, 0))
            .chain(after.difference(&before).map(|id| packet(*id, true, 0)))
            .collect()
    }
}
