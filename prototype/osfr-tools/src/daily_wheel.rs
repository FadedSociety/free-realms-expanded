//! T560: the daily rewards wheel ("Spin For The Win!"), part 1: offer the spin and open the movie.
//!
//! # Recovered client facts (addresses in `FreeRealms.exe`)
//!
//! * Spin count: the client keeps a list of repeating activities and finds the wheel by the name
//!   `"wheel"` (Lua `Ui.GetRepeatingActivityCount` / `GetRepeatingActivityNextTime`, registered
//!   at table slots `0182ded8` / `0182dee0`, bodies `00c17f80` / `00c18000`). 143/1
//!   RepeatingActivityAdd (RTTI `.?AVRepeatingActivityAddPacket@@`) is read by `00adafa0`:
//!   header u16 opcode + **u8** sub (`00adae70`), then five i32 (`00adaed0`: id, count,
//!   consecutive, two second counts), two length-prefixed strings and one i32. The handler
//!   `00adb990` finds a node by the id and updates it in place (`00ada6f0`: count, consecutive,
//!   `now + secs` into +0x50 and `now + secs` into +0x48); a new id is added. The count lookup
//!   `00ada830` matches the second string exactly, else sums nodes whose first string matches;
//!   `NextTime` (`00ada8b0`) reads the +0x50 time, so "wheel" goes in both strings.
//! * Start: `MiniGameFlashC::StartWheel` (`009bdbd0`) sends 26/11
//!   (`CommandPacketInteractionStartWheel`, constructor `009aad80`: header only) only while the
//!   count of "wheel" is above zero. ScriptsBase.lua also disables the Games window's Play and the
//!   calendar's spin when the count is below one, so the count is the client's whole gate.
//! * Launch: the server answers with a type-22 39/16 (`009bfc50`) whose start-screen byte
//!   `+0x267` (`flags[3]`, passed on as `a4` by `009bfdb0`) is off, so `009beb70` skips the start
//!   screen and asks to start itself (`009be7d0`, 39/5; the type list at 22 marks the state
//!   loaded without a preload). 39/5 is then answered with 39/17 and the Flash launch 26/12
//!   ("MiniGameFlash", movie), exactly as Mining is.
//!
//! # Reconstructed policy (R)
//!
//! * One spin per UTC day. `last_spin_day` (a day index of the server clock) is the only state,
//!   saved in `GameplayState`; a spin is available when today is after it, so a clock that goes
//!   backwards never grants a second spin. This part never writes it: the spin is used up only
//!   when a prize is paid (part 2).
//! * 143/1 id is the wheel's activity id; consecutive is 0; both second counts carry the seconds
//!   to the next UTC midnight; the last i32 is 0 (the client's meaning of it is not recovered).
//! * The run is session-only: a disconnect, a zone entry or a leave (39/6) ends it and costs no
//!   spin.

use crate::minigame::{MinigameRuns, RunMode, RunOutcome, RunSpec};
use serde::Deserialize;
use std::{
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

/// Optional content beside the other activity files; absent content leaves the wheel off.
pub const CONTENT_FILE: &str = "daily_wheel.json";
const MAX_FILE: u64 = 4 * 1024;
const DAY_SECONDS: u64 = 86_400;
/// (R) The wheel's own instance range, apart from every other minigame ledger.
const FIRST_INSTANCE_ID: i32 = 62_000_001;
/// The client's name for the wheel in its repeating-activity list (Lua `GetRepeatingActivityCount`).
const LIST_NAME: &str = "wheel";
/// `CommandPacketInteractionStartWheel` (26/11), constructor `009aad80`: no body.
pub const START_WHEEL: [u8; 4] = [26, 0, 11, 0];

/// `daily_wheel.json`: which client rows and movie the wheel uses. Tunable data, not code.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Content {
    schema_version: u32,
    /// Provenance for readers; never gameplay input.
    #[serde(rename = "source")]
    _source: String,
    /// The Games window's Play request for this activity (167/1/2) starts the same spin.
    pub activity_id: u32,
    /// `MiniGameData.txt` row; the run's game id.
    pub minigame_id: i32,
    /// `MiniGameData.txt` TYPE column (22, the wheel).
    pub type_id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub icon_id: i32,
    pub movie: String,
}

impl Content {
    /// `None` when the content directory has no wheel file.
    pub fn load_optional(directory: &Path) -> io::Result<Option<Arc<Self>>> {
        let path = directory.join(CONTENT_FILE);
        if !path.is_file() {
            return Ok(None);
        }
        let mut text = String::new();
        fs::File::open(path)?
            .take(MAX_FILE + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_FILE {
            return Err(invalid("daily wheel content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(invalid)?;
        // The run spec's own validation covers the ids and the movie name.
        if content.schema_version != 1 || !(1..=i32::MAX as u32).contains(&content.activity_id) {
            return Err(invalid("unsupported daily wheel content"));
        }
        MinigameRuns::new(FIRST_INSTANCE_ID)?.offer(content.spec())?;
        Ok(Some(Arc::new(content)))
    }

    fn spec(&self) -> RunSpec {
        let mut spec = RunSpec::empty(
            self.minigame_id,
            self.type_id,
            RunMode::Flash {
                movie: self.movie.clone(),
            },
        );
        spec.name_id = self.name_id;
        spec.description_id = self.description_id;
        spec.icon_id = self.icon_id;
        // `009bfdb0` passes flags[3] (+0x267) as "show the start screen": off, the client starts
        // the run by itself.
        spec.flags[3] = 0;
        spec
    }
}

/// Why a start request is refused; none of them changes anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The wheel is already open.
    Open,
    /// Today's spin is used up.
    NoSpin,
}

impl Refusal {
    pub fn message(self) -> &'static str {
        match self {
            Self::Open => "The wheel is already open.",
            Self::NoSpin => "You have already spun the wheel today. Come back tomorrow!",
        }
    }
}

/// One session's wheel: the content, the run ledger and the saved day stamp.
#[derive(Clone, Debug)]
pub struct DailyWheel {
    content: Option<Arc<Content>>,
    runs: MinigameRuns,
    last_spin_day: Option<u64>,
    /// (R) Test/offline server clock in Unix seconds; the gateway never sets it.
    clock: Option<u64>,
}

impl DailyWheel {
    pub fn load(directory: &Path, last_spin_day: Option<u64>) -> io::Result<Self> {
        Ok(Self {
            content: Content::load_optional(directory)?,
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            last_spin_day,
            clock: None,
        })
    }

    pub fn enabled(&self) -> bool {
        self.content.is_some()
    }

    /// The day stamp `GameplayState` saves. Kept even while the content is absent.
    pub fn last_spin_day(&self) -> Option<u64> {
        self.last_spin_day
    }

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    /// (R) Pin the server clock for tests and offline fixtures.
    pub fn set_clock(&mut self, unix_seconds: u64) {
        self.clock = Some(unix_seconds);
    }

    fn now(&self) -> u64 {
        self.clock.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_secs())
        })
    }

    /// Spins left today: 1 until one is paid (part 2), then 0 until the next UTC day.
    pub fn spins_left(&self) -> u32 {
        let today = self.now() / DAY_SECONDS;
        u32::from(self.enabled() && self.last_spin_day.is_none_or(|day| today > day))
    }

    /// Whether `inner` starts the wheel: the dock's 26/11, or the Games window's Play for the
    /// wheel's activity row.
    pub fn is_start_request(&self, inner: &[u8]) -> bool {
        use crate::activity_portal::{Request, parse};
        let Some(content) = &self.content else {
            return false;
        };
        inner == START_WHEEL
            || matches!(
                parse(inner),
                Ok(Some(Request::Play { activity })) if activity == content.activity_id
            )
    }

    /// World entry: a run the client's scene load left behind is gone with its movie, so end it
    /// without a spin used; then publish the count (143/1).
    pub fn world_entry(&mut self) -> Option<Vec<u8>> {
        self.runs.disconnect();
        let content = self.content.as_ref()?;
        let to_reset = (DAY_SECONDS - self.now() % DAY_SECONDS) as i32;
        let mut bytes = vec![143, 0, 1];
        for value in [content.activity_id as i32, self.spins_left() as i32, 0] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(to_reset.to_le_bytes());
        bytes.extend(to_reset.to_le_bytes());
        for name in [LIST_NAME, LIST_NAME] {
            bytes.extend((name.len() as i32).to_le_bytes());
            bytes.extend(name.as_bytes());
        }
        bytes.extend(0_i32.to_le_bytes());
        Some(bytes)
    }

    /// Offer the run (39/16). The caller has already refused a busy player.
    pub fn begin(&mut self) -> io::Result<Result<Vec<Vec<u8>>, Refusal>> {
        let Some(content) = self.content.clone() else {
            return Ok(Err(Refusal::NoSpin));
        };
        if self.is_active() {
            return Ok(Err(Refusal::Open));
        }
        if self.spins_left() == 0 {
            return Ok(Err(Refusal::NoSpin));
        }
        Ok(Ok(self.runs.offer(content.spec())?.replies))
    }

    /// A lifecycle packet naming the wheel's current or just-ended run (39/5 start, 39/6 leave,
    /// 39/7 acknowledgement); `None` leaves every other packet to later owners.
    pub fn route(&mut self, packet: &[u8]) -> io::Result<Option<RunOutcome>> {
        if !self.runs.names_run(packet) {
            return Ok(None);
        }
        self.runs.handle_client(packet).map(Some)
    }
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
