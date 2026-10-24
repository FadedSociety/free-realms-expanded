//! T469: the game clock in the opcode 52 time-sync reply, so the client's sky runs.
//!
//! Client facts (IDA, 2026-10-22):
//! - Reader `0x00bd1700`: u16 opcode, i64 Time, i32 ServerRate, bool UseClientTime (15 bytes).
//! - Apply `0x00bd1830` calls SetFromUnixTime `0x010bd270` with Time, then stores ServerRate as
//!   the sky scale (+0x60) and UseClientTime as the freeze byte (+0x64).
//! - While +0x64 is set, `0x010bd270` and AdvanceClock `0x010bd380` do nothing, so a reply with
//!   UseClientTime 1 froze the sky at login. Unfrozen, AdvanceClock moves the time of day by
//!   scale x elapsed between syncs, so Time advances at the same rate and a resync never jumps.
//! - `0x010bd270` takes the time of day from localtime64 (`0x007026b0`, local flag), so each
//!   player's sky is offset by their own UTC offset. Retail did the same; we do not compensate.
use serde::Deserialize;
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

const CONTENT_FILE: &str = "game_clock.json";
const MAX_FILE: u64 = 4 * 1024;
/// `0x007026b0` turns any Time above this into 1970-01-01 00:00, a sky jump.
const LOCALTIME64_MAX: i64 = 0x7_9340_6FFF;

/// Sky clock tuning from `game_clock.json`. Content, not character state: never saved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameClock {
    /// Sky seconds per real second (ServerRate). (R) Bounds 1..=16: 0 freezes the sky
    /// (`0x010bd380` never advances).
    pub rate: i32,
    /// The Unix second where game Time is 0.
    pub epoch_unix: i64,
}

impl Default for GameClock {
    /// The client's default scale 8 (ctor `0x010bd190`) and the (R) retail epoch; the
    /// provenance is in `data/content/game_clock.json`.
    fn default() -> Self {
        Self {
            rate: 8,
            epoch_unix: 1_221_361_670,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentFile {
    schema_version: u32,
    /// Provenance for readers; never gameplay input.
    #[serde(default, rename = "source")]
    _source: Option<String>,
    rate: i32,
    epoch_unix: i64,
}

impl GameClock {
    /// `game_clock.json` from `directory`, or the default when the file is absent. A malformed
    /// or out-of-bounds file refuses the content load.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let mut text = String::new();
        match fs::File::open(directory.join(CONTENT_FILE)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            file => file?.take(MAX_FILE + 1).read_to_string(&mut text)?,
        };
        if text.len() as u64 > MAX_FILE {
            return Err(invalid("game clock content exceeds 4 KiB"));
        }
        let file: ContentFile = serde_json::from_str(&text).map_err(|e| invalid(&e.to_string()))?;
        if file.schema_version != 1 {
            return Err(invalid("unsupported game clock content version"));
        }
        Self {
            rate: file.rate,
            epoch_unix: file.epoch_unix,
        }
        .checked()
    }

    pub fn checked(self) -> io::Result<Self> {
        if !(1..=16).contains(&self.rate) || self.epoch_unix < 0 {
            return Err(invalid("game clock needs rate 1..=16 and epoch_unix >= 0"));
        }
        Ok(self)
    }

    /// The unwrapped opcode 52 reply (reader `0x00bd1700`). A pure function of `now_unix`, so
    /// every session sees the same sky.
    pub fn sync_reply(&self, now_unix: i64) -> Vec<u8> {
        // Clock skew before the epoch sends 0, never a negative Time. (R) Past the client's
        // localtime64 cap (year 2137 at rate 8) Time holds at the cap instead of jumping to 1970.
        let time = now_unix
            .saturating_sub(self.epoch_unix)
            .saturating_mul(i64::from(self.rate))
            .clamp(0, LOCALTIME64_MAX);
        let mut reply = 52u16.to_le_bytes().to_vec();
        reply.extend(time.to_le_bytes());
        reply.extend(self.rate.to_le_bytes());
        reply.push(0); // UseClientTime false: the client's clock runs.
        reply
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-21 00:00:00 UTC.
    const NOW: i64 = 1_792_540_800;
    /// 2100-01-01 00:00:00 UTC.
    const YEAR_2100: i64 = 4_102_444_800;

    fn time(clock: GameClock, now: i64) -> i64 {
        i64::from_le_bytes(clock.sync_reply(now)[2..10].try_into().unwrap())
    }

    #[test]
    fn reply_matches_reader_0x00bd1700_byte_for_byte() {
        // Time = 8 x (1792540800 - 1221361670) = 4569433040 = 0x1_105C_03D0.
        assert_eq!(
            GameClock::default().sync_reply(NOW),
            [52, 0, 0xD0, 0x03, 0x5C, 0x10, 1, 0, 0, 0, 8, 0, 0, 0, 0]
        );
    }

    #[test]
    fn time_runs_at_the_rate_and_never_decreases() {
        let clock = GameClock::default();
        assert_eq!(time(clock, NOW + 11) - time(clock, NOW), 88);
        for now in NOW..NOW + 100 {
            assert!(time(clock, now + 1) >= time(clock, now));
        }
        // Concurrent players: the same second gives the same bytes.
        assert_eq!(clock.sync_reply(NOW), clock.sync_reply(NOW));
    }

    #[test]
    fn bounds_refuse_bad_tuning_and_time_stays_inside_localtime64() {
        let default = GameClock::default();
        for rate in [0, 17, -8] {
            assert!(GameClock { rate, ..default }.checked().is_err(), "{rate}");
        }
        let early = GameClock {
            epoch_unix: -1,
            ..default
        };
        assert!(early.checked().is_err());
        assert_eq!(default.checked().unwrap(), default);
        let fast = GameClock {
            rate: 16,
            ..default
        }
        .checked()
        .unwrap();
        // Time never decreases in now, so the largest now bounds every earlier one.
        for now in [NOW, 3_254_814_469, 3_254_814_470, YEAR_2100, i64::MAX] {
            assert!(time(fast, now) <= LOCALTIME64_MAX, "{now}");
        }
        assert_eq!(time(fast, i64::MAX), LOCALTIME64_MAX);
        // The default rate is still exact, unclamped, in 2100.
        assert_eq!(
            time(default, YEAR_2100),
            8 * (YEAR_2100 - default.epoch_unix)
        );
    }

    #[test]
    fn clock_skew_before_the_epoch_sends_zero() {
        let clock = GameClock::default();
        for now in [clock.epoch_unix - 1, 0, i64::MIN] {
            assert_eq!(time(clock, now), 0, "{now}");
        }
    }

    #[test]
    fn content_file_loads_and_bad_files_refuse() {
        let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        assert_eq!(GameClock::load(&shipped).unwrap(), GameClock::default());
        let dir = std::env::temp_dir().join(format!("osfr-t469-clock-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert_eq!(GameClock::load(&dir).unwrap(), GameClock::default());
        let file = |text: &str| {
            fs::write(dir.join(CONTENT_FILE), text).unwrap();
            GameClock::load(&dir)
        };
        let tuned = r#"{"schema_version":1,"source":"test","rate":4,"epoch_unix":7}"#;
        let tuned = file(tuned).unwrap();
        assert_eq!((tuned.rate, tuned.epoch_unix), (4, 7));
        for bad in [
            r#"{"schema_version":1,"rate":0,"epoch_unix":7}"#,
            r#"{"schema_version":1,"rate":17,"epoch_unix":7}"#,
            r#"{"schema_version":1,"rate":8,"epoch_unix":-1}"#,
            r#"{"schema_version":2,"rate":8,"epoch_unix":7}"#,
            r#"{"schema_version":1,"rate":8,"epoch_unix":7,"extra":1}"#,
            r#"{"schema_version":1,"rate":8}"#,
            "not json",
            &format!(
                r#"{{"schema_version":1,"source":"{}","rate":8,"epoch_unix":7}}"#,
                "x".repeat(5000)
            ),
        ] {
            assert!(file(bad).is_err(), "{bad}");
        }
        fs::remove_dir_all(&dir).unwrap();
    }
}
