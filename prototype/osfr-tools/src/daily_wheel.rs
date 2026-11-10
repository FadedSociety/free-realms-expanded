//! The daily wheels ("Spin For The Win!"): offer the spins, open the movie, answer the movie's
//! messages, draw and pay the prize, and keep each character's spins.
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
//!   `00ada830` takes the node whose second string matches exactly (`00ada790`), else sums the
//!   nodes whose first string matches; `NextTime` (`00ada8b0`) reads the +0x50 time of a node
//!   with a count above zero. One node named "wheel" in both strings carries the spins of every
//!   wheel added up (R: the original may have sent one node per wheel; the client adds them up
//!   either way).
//! * 143/3 RepeatingActivityConsecutiveMsg (`00adb070` reads six i32: id, day, spin, string id,
//!   day2, spin2; `00adba80` fills the string's `spin`/`day` tokens, and `spin2`/`day2` when day2
//!   is above zero, and shows it through `NotificationHandler:WheelMessage`). Client strings
//!   415176 "Nice! You get [spin] for playing [day] in a row! If you spin for [day2] in a row
//!   you get [spin2]." and 415177 "Awesome! You get [spin] for playing [day] in a row."
//! * Start: `MiniGameFlashC::StartWheel` (`009bdbd0`) sends 26/11
//!   (`CommandPacketInteractionStartWheel`, constructor `009aad80`: header only) only while the
//!   count of "wheel" is above zero. ScriptsBase.lua also disables the Games window's Play (with
//!   the text `NoSpinMessage`, string 432721) and the calendar's spin when the count is below
//!   one, so the count is the client's whole gate.
//! * Launch: the server answers with a type-22 39/16 (`009bfc50`) whose start-screen byte
//!   `+0x267` (`flags[3]`, passed on as `a4` by `009bfdb0`) is off, so `009beb70` skips the start
//!   screen and asks to start itself (`009be7d0`, 39/5; the type list at 22 marks the state
//!   loaded without a preload). 39/5 is then answered with 39/17 and the Flash launch 26/12
//!   ("MiniGameFlash", movie), exactly as Mining is.
//! * The movie (`game_wheel.gfx`; ActionScript classes `Wheel.WheelGameClient`, `Wheel.WheelMC`,
//!   `Wheel.WheelSpinner`, `Shared.RewardWindow`, `GameClientNetwork`) talks in 39/14 text
//!   messages, fields separated by tabs. It sends `OnConnectMsg`; the server answers, per wheel,
//!   `OnWheelDataMsg` (wheel id, type, the wedge picture frames separated by spaces, name string
//!   id, message string id) and `OnWheelUpdateMsg` (id, spins, consecutive days, seconds to the
//!   next spin), then `OnServerReadyMsg`. The movie shows the first wheel with a spin, with a
//!   page button to each neighbour (`OnChangeWheelRequestMsg` -> `OnWheelChangedMsg`).
//!   Releasing the spin button sends `OnWheelSpinRequestMsg` (id) and starts the turn: `power`
//!   (1..5, from the held meter) full turns plus a fraction of a wedge. `OnSpinInfoMsg` (index)
//!   adds exactly that many wedges, so the needle stops inside the wedge with that index; index
//!   0 is the wedge left of the needle at rest and the indexes run counter-clockwise
//!   (`mcCategory1..10` placements in `mcSpinner`). When the wheel stops the movie sends
//!   `OnWheelStopMsg` (id) and the server answers `OnRewardInfoMsg` (icon id, item definition id
//!   for the tooltip, quantity, name string id, body string id, tint id), which opens the reward
//!   window; icon 22663 is the coin picture and adds the window's "Shop Now!" button. Closing
//!   the window with no spins left removes that wheel, and after the last one the movie sends
//!   `FRServer_GameClose`.
//! * Wheel types with art: 1 the Daily Wheel (ten wedges), 5 the Coin Wheel (ten), 6 the
//!   Festival of Hearts wheel (eight). The wedge sprite has 25 frames (1 ring, 2 clothes, 3 bee
//!   and battle items, 4 the wheel itself, 5 cards, 6 grab bag, 7 sports gear, 8 potion and
//!   scrolls, 9 ticket, 10 sword and wrench, 13 house, 15-21 a coin marked 2 5 10 20 30 40 50,
//!   22 a goblin figure, 23 a heart).
//! * The movie's own test server (`Wheel.WheelGameServer`) picks the wedge with
//!   `Random(0, slotCount - 1)`: every wedge is equally likely. The shipped weights are all 1.
//!
//! # Reconstructed policy (R)
//!
//! * One free spin per wheel per UTC day; a missed day is lost (the fan wiki: "spins do not
//!   accumulate"). Extra spins (the Extra Spin wedge, the streak bonus) are kept until used.
//!   A spin is free when today is after the wheel's stamped day, so a clock that goes backwards
//!   never grants one.
//! * Streak: a free spin the day after the last one adds a day, any gap starts again at one.
//!   Reaching a `streak_bonuses` row banks its spins on that wheel (the wiki: three days one
//!   extra spin, seven days two); after the last row the count starts over.
//! * The wedge and the prize are drawn and paid when the spin is requested, in the same outcome
//!   that uses up the spin (one save), so closing the movie or losing the connection mid-turn
//!   neither loses the prize nor frees another spin, and a client can never choose or reroll.
//!   The reward packets wait for the stop (or the close) so nothing shows before the wheel has
//!   turned. Before the draw the bag must have room for every prize the wheel could pay;
//!   otherwise the wheel closes with the spin kept, whatever would have been drawn.
//! * A player in the world across UTC midnight is sent the new day's count on the next tick.
//! * Every prize, weight and amount in `daily_wheel.json` is tunable data; its `source` names
//!   the evidence per wedge.
//!
//! The battle and dungeon prize wheel (`LootWheelOfAwesomeness.gfx`, 39/45) is another movie
//! with another protocol (`minigame::choose_wheel_prize`, `encounter/arena.rs`); the two share
//! only `power_ups::pick_weighted`.

use crate::{
    minigame::{
        MinigameRuns, RunMode, RunOutcome, RunSpec, movie_text, payload_packet, state_remove_packet,
    },
    power_ups::{Rng, pick_weighted},
    rewards::{ItemGrant, RewardDisplay, RewardGrant},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

/// Optional content beside the other activity files; absent content leaves the wheel off.
pub const CONTENT_FILE: &str = "daily_wheel.json";
const MAX_FILE: u64 = 64 * 1024;
const DAY_SECONDS: u64 = 86_400;
/// (R) The wheel's own instance range, apart from every other minigame ledger.
const FIRST_INSTANCE_ID: i32 = 62_000_001;
/// The client's name for the wheel in its repeating-activity list (Lua `GetRepeatingActivityCount`).
const LIST_NAME: &str = "wheel";
/// `CommandPacketInteractionStartWheel` (26/11), constructor `009aad80`: no body.
pub const START_WHEEL: [u8; 4] = [26, 0, 11, 0];
/// The client's `NoSpinMessage` (string 432721), shown under the Games window's greyed Play.
pub const NO_SPIN: &str = "You are out of wheel spins for today.";
/// Typo guards for the data, not gameplay values.
const MAX_WHEELS: usize = 4;
const MAX_WEIGHT: u32 = 1_000;
const MAX_COINS: u32 = 10_000;
const MAX_SPINS: u32 = 10;
const MAX_PRIZES: usize = 32;
const MAX_PRIZE_ITEMS: usize = 6;
/// The most extra spins one wheel keeps (a runaway guard).
const MAX_BANKED: u32 = 99;
/// The movie's `mcCategory` sprite has 25 picture frames.
const CATEGORY_FRAMES: i32 = 25;

/// One thing a wedge can pay, with its share of that wedge's draw: coins, extra spins on the
/// same wheel, items, or several of them at once (the grab bag).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prize {
    weight: u32,
    #[serde(default)]
    pub coins: u32,
    #[serde(default)]
    pub spins: u32,
    #[serde(default)]
    pub items: Vec<ItemGrant>,
}

impl Prize {
    fn valid(&self) -> bool {
        (1..=MAX_WEIGHT).contains(&self.weight)
            && self.coins <= MAX_COINS
            && self.spins <= MAX_SPINS
            && self.items.len() <= MAX_PRIZE_ITEMS
            && (self.coins > 0 || self.spins > 0 || !self.items.is_empty())
            && (self.items.iter()).all(|item| item.definition_id > 0 && (1..=999).contains(&item.count))
    }
}

/// One wedge: the picture frame it shows, its share of the spin and what it can pay.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Slot {
    category: i32,
    weight: u32,
    prizes: Vec<Prize>,
}

/// Spinning `days` days in a row banks `spins` extra spins.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StreakBonus {
    days: u32,
    spins: u32,
}

/// One wheel: what the movie shows and what every wedge pays.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wheel {
    /// The save key; never shown.
    key: String,
    wheel_id: i32,
    /// 1 Daily, 5 Coin, 6 Festival of Hearts (`mcBase` frames with art).
    wheel_type: i32,
    name_id: i32,
    message_id: i32,
    reward_message_id: i32,
    #[serde(default)]
    streak_bonuses: Vec<StreakBonus>,
    slots: Vec<Slot>,
}

impl Wheel {
    fn valid(&self) -> bool {
        let wedges = match self.wheel_type {
            1 | 5 => 10,
            6 => 8,
            _ => return false,
        };
        let ids = [self.wheel_id, self.name_id, self.message_id, self.reward_message_id];
        !self.key.is_empty()
            && self.key.len() <= 24
            && self.key.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'_')
            && ids.iter().all(|id| *id > 0)
            && self.slots.len() == wedges
            && self.slots.iter().all(|slot| {
                (1..=CATEGORY_FRAMES).contains(&slot.category)
                    && (1..=MAX_WEIGHT).contains(&slot.weight)
                    && (1..=MAX_PRIZES).contains(&slot.prizes.len())
                    && slot.prizes.iter().all(Prize::valid)
            })
            && self.streak_bonuses.len() <= 8
            && (self.streak_bonuses.iter()).all(|row| row.days >= 2 && (1..=MAX_SPINS).contains(&row.spins))
            && self.streak_bonuses.windows(2).all(|pair| pair[0].days < pair[1].days)
    }
}

/// `daily_wheel.json`: the client rows, the movie and the wheels. Tunable data, not code.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Content {
    schema_version: u32,
    /// Provenance for readers; never gameplay input.
    #[serde(rename = "source")]
    _source: serde_json::Value,
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
    /// The reward window's picture and name for coins (22663 is the client's coin case).
    coin_icon_id: i32,
    coin_name_id: i32,
    /// The reward window's picture, name and body line for extra spins.
    spin_icon_id: i32,
    spin_name_id: i32,
    spin_message_id: i32,
    /// The streak notice with a next step, and the one for the last step (143/3).
    streak_message_id: i32,
    streak_last_message_id: i32,
    wheels: Vec<Wheel>,
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
        let ids = [
            content.coin_icon_id,
            content.coin_name_id,
            content.spin_icon_id,
            content.spin_name_id,
            content.spin_message_id,
            content.streak_message_id,
            content.streak_last_message_id,
        ];
        let distinct = |key: fn(&Wheel) -> String| {
            let mut keys: Vec<String> = content.wheels.iter().map(key).collect();
            keys.sort();
            keys.windows(2).all(|pair| pair[0] != pair[1])
        };
        // The run spec's own validation covers the game ids and the movie name.
        if content.schema_version != 2
            || !(1..=i32::MAX as u32).contains(&content.activity_id)
            || ids.iter().any(|id| *id <= 0)
            || !(1..=MAX_WHEELS).contains(&content.wheels.len())
            || !content.wheels.iter().all(Wheel::valid)
            || !distinct(|wheel| wheel.key.clone())
            || !distinct(|wheel| wheel.wheel_id.to_string())
        {
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

    /// Every (wheel id, wedge index, prize index, prize) the content can pay.
    pub fn prizes(&self) -> impl Iterator<Item = (i32, usize, usize, &Prize)> {
        self.wheels.iter().flat_map(|wheel| {
            wheel.slots.iter().enumerate().flat_map(move |(slot, wedge)| {
                (wedge.prizes.iter().enumerate())
                    .map(move |(at, prize)| (wheel.wheel_id, slot, at, prize))
            })
        })
    }
}

/// One wheel's saved spins for one character.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spins {
    /// UTC day index of the last free spin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    day: Option<u64>,
    /// Extra spins not yet used.
    #[serde(default, skip_serializing_if = "is_zero")]
    extra: u32,
    /// Days in a row with a free spin, up to `day`.
    #[serde(default, skip_serializing_if = "is_zero")]
    streak: u32,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

/// Every wheel's saved spins, by the wheel's `key`. Absent from a save until a wheel is spun.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Ledger(BTreeMap<String, Spins>);

impl Ledger {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// A ledger with every named wheel's free spin used on `day` (fixtures).
    pub fn spun_on(day: u64, keys: &[&str]) -> Self {
        Self(
            keys.iter()
                .map(|key| ((*key).to_owned(), Spins { day: Some(day), ..Spins::default() }))
                .collect(),
        )
    }
}

/// Why a start request is refused; none of them changes anything.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The wheel is already open.
    Open,
    /// No wheel has a spin left today.
    NoSpin,
}

impl Refusal {
    pub fn message(self) -> &'static str {
        match self {
            Self::Open => "The wheel is already open.",
            Self::NoSpin => NO_SPIN,
        }
    }
}

/// One message of the wheel movie, already checked to name the live run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Say {
    Connect,
    Spin(i32),
    Stop(i32),
    Change(i32),
    Close,
}

/// A spin the server has drawn and not yet committed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draw {
    wheel: usize,
    /// The wedge index the needle will stop on.
    pub slot: usize,
    pub prize: Prize,
}

/// What a paid spin shows once the wheel has stopped.
#[derive(Clone, Debug)]
struct Held {
    wheel: usize,
    window: String,
    packets: Vec<Vec<u8>>,
}

/// One session's wheels: the content, the run ledger and the saved spins.
#[derive(Clone, Debug)]
pub struct DailyWheel {
    content: Option<Arc<Content>>,
    runs: MinigameRuns,
    ledger: Ledger,
    /// (R) Test/offline server clock in Unix seconds; the gateway never sets it.
    clock: Option<u64>,
    rng: Rng,
    /// (R) Test seam: the next draw's wedge and prize indexes.
    forced: Option<(usize, usize)>,
    /// The movie has connected and been sent the wheels.
    connected: bool,
    held: Option<Held>,
    /// The UTC day the count was last published on (world entry), for the midnight resend.
    published_day: Option<u64>,
}

impl DailyWheel {
    pub fn load(directory: &Path, ledger: Ledger) -> io::Result<Self> {
        Ok(Self {
            content: Content::load_optional(directory)?,
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            ledger,
            clock: None,
            rng: Rng::from_entropy()?,
            forced: None,
            connected: false,
            held: None,
            published_day: None,
        })
    }

    pub fn enabled(&self) -> bool {
        self.content.is_some()
    }

    /// The wheel's Games window row while the wheel is on (L095 lists that live row open).
    pub fn activity_id(&self) -> Option<u32> {
        self.content.as_ref().map(|content| content.activity_id)
    }

    pub fn minigame_id(&self) -> Option<u32> {
        u32::try_from(self.content.as_ref()?.minigame_id).ok()
    }

    /// What `GameplayState` saves. Kept even while the content is absent.
    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    /// The first wheel's day stamp, mirrored into the older save field (written, never read).
    pub fn last_spin_day(&self) -> Option<u64> {
        let first = self.content.as_ref()?.wheels.first()?;
        self.ledger.0.get(&first.key)?.day
    }

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    /// (R) Pin the server clock for tests and offline fixtures.
    pub fn set_clock(&mut self, unix_seconds: u64) {
        self.clock = Some(unix_seconds);
    }

    /// (R) Deterministic draws for tests.
    pub fn reseed(&mut self, seed: u64) {
        self.rng = Rng::seeded(seed);
    }

    /// (R) Test seam: the next draw lands on this wedge and prize.
    pub fn force(&mut self, slot: usize, prize: usize) {
        self.forced = Some((slot, prize));
    }

    fn now(&self) -> u64 {
        self.clock.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_secs())
        })
    }

    fn today(&self) -> u64 {
        self.now() / DAY_SECONDS
    }

    fn seconds_to_reset(&self) -> i32 {
        (DAY_SECONDS - self.now() % DAY_SECONDS) as i32
    }

    fn saved(&self, wheel: &Wheel) -> Spins {
        self.ledger.0.get(&wheel.key).cloned().unwrap_or_default()
    }

    fn free_spin(&self, wheel: &Wheel) -> bool {
        self.saved(wheel).day.is_none_or(|day| self.today() > day)
    }

    fn wheel_spins(&self, wheel: &Wheel) -> u32 {
        u32::from(self.free_spin(wheel)).saturating_add(self.saved(wheel).extra)
    }

    /// Spins left on every wheel together: the number the client's list shows.
    pub fn spins_left(&self) -> u32 {
        let Some(content) = &self.content else {
            return 0;
        };
        content.wheels.iter().map(|wheel| self.wheel_spins(wheel)).sum()
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

    /// 143/1: the spins left, the first wheel's streak and the seconds to the next UTC midnight.
    /// The last i32 is 0 (the client's meaning of it is not recovered).
    fn spins_packet(&self) -> Option<Vec<u8>> {
        let content = self.content.as_ref()?;
        let streak = content.wheels.first().map_or(0, |wheel| self.saved(wheel).streak);
        let to_reset = self.seconds_to_reset();
        let mut bytes = vec![143, 0, 1];
        for value in [content.activity_id as i32, self.spins_left() as i32, streak as i32] {
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

    /// World entry: a run the client's scene load left behind is gone with its movie, so end it
    /// (a prize it had not shown yet is shown now); then publish the count (143/1).
    pub fn world_entry(&mut self) -> Vec<Vec<u8>> {
        self.runs.disconnect();
        self.connected = false;
        self.published_day = Some(self.today());
        let mut out = self.held.take().map(|held| held.packets).unwrap_or_default();
        out.extend(self.spins_packet());
        out
    }

    /// A player in the world across UTC midnight: publish the new day's count once, so the
    /// Games window's Play is live again without a relog.
    pub fn day_rollover(&mut self) -> Option<Vec<u8>> {
        let today = self.today();
        if self.published_day.is_none_or(|day| day == today) {
            return None;
        }
        self.published_day = Some(today);
        self.spins_packet()
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
        self.connected = false;
        Ok(Ok(self.runs.offer(content.spec())?.replies))
    }

    /// A lifecycle packet naming the wheel's current or just-ended run (39/5 start, 39/6 leave,
    /// 39/7 acknowledgement); `None` leaves every other packet to later owners. A leave mid-turn
    /// still delivers the prize the spin already paid.
    pub fn route(&mut self, packet: &[u8]) -> io::Result<Option<RunOutcome>> {
        if !self.runs.names_run(packet) {
            return Ok(None);
        }
        let mut outcome = self.runs.handle_client(packet)?;
        if outcome.accepted && !self.is_active() {
            let unseen = self.take_unseen();
            outcome.replies.splice(0..0, unseen);
        }
        Ok(Some(outcome))
    }

    /// The movie message in `packet`, when it names the live run. Anything else (another
    /// instance, an unknown name, a missing or non-numeric wheel id) is `None` and ignored.
    pub fn say(&self, packet: &[u8]) -> Option<Say> {
        let live = self.runs.current_instance_id()?;
        let content = self.content.as_ref()?;
        let game = int_at(packet, 11)?;
        if packet.get(..3) != Some(&[39, 0, 14][..])
            || int_at(packet, 3)? != live
            || int_at(packet, 7)? != -1
            || (game != -1 && game != content.minigame_id)
        {
            return None;
        }
        let mut fields = movie_text(packet).ok()?.split('\t');
        let name = fields.next()?;
        let mut wheel = || fields.next()?.parse::<i32>().ok();
        match name {
            "OnConnectMsg" => Some(Say::Connect),
            "OnWheelSpinRequestMsg" => wheel().map(Say::Spin),
            "OnWheelStopMsg" => wheel().map(Say::Stop),
            "OnChangeWheelRequestMsg" => wheel().map(Say::Change),
            "FRServer_GameClose" => Some(Say::Close),
            _ => None,
        }
    }

    fn message(&self, text: &str) -> io::Result<Vec<u8>> {
        let (Some(instance), Some(content)) =
            (self.runs.current_instance_id(), self.content.as_ref())
        else {
            return Err(invalid("no wheel run"));
        };
        payload_packet(instance, content.minigame_id, text)
    }

    fn update_text(&self, wheel: &Wheel) -> String {
        let spins = self.wheel_spins(wheel);
        let next = if spins == 0 { self.seconds_to_reset() } else { 0 };
        format!(
            "OnWheelUpdateMsg\t{}\t{spins}\t{}\t{next}",
            wheel.wheel_id,
            self.saved(wheel).streak
        )
    }

    /// The movie connected: send every wheel that has a spin, with its spins, then the ready
    /// message. Once per run. A wheel without a spin is not sent: the movie shows the first
    /// wheel with spins but keeps its current-wheel index at 0 (`OnServerReadyMsg`), so a spent
    /// wheel in front would take the spin button's clicks and the movie would never close.
    pub fn connect(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let (Some(content), Some(instance)) =
            (self.content.clone(), self.runs.current_instance_id())
        else {
            return Ok(Vec::new());
        };
        if self.connected {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for wheel in (content.wheels.iter()).filter(|wheel| self.wheel_spins(wheel) > 0) {
            let frames: Vec<String> = (wheel.slots.iter())
                .map(|slot| slot.category.to_string())
                .collect();
            out.push(self.message(&format!(
                "OnWheelDataMsg\t{}\t{}\t{}\t{}\t{}",
                wheel.wheel_id,
                wheel.wheel_type,
                frames.join(" "),
                wheel.name_id,
                wheel.message_id
            ))?);
            out.push(self.message(&self.update_text(wheel))?);
        }
        out.push(self.message("OnServerReadyMsg")?);
        self.connected = true;
        self.runs.started(instance);
        Ok(out)
    }

    /// The page buttons: confirm a wheel this content has.
    pub fn change(&self, wheel_id: i32) -> io::Result<Option<Vec<u8>>> {
        let known = (self.content.iter())
            .flat_map(|content| &content.wheels)
            .any(|wheel| wheel.wheel_id == wheel_id);
        (known && self.connected)
            .then(|| self.message(&format!("OnWheelChangedMsg\t{wheel_id}")))
            .transpose()
    }

    /// The wheel a spin request may turn right now, or `None`: no live connected run, an unknown
    /// wheel, a turn still in flight, no spin left.
    fn spinnable(&self, wheel_id: i32) -> Option<(usize, &Wheel)> {
        let wheels = &self.content.as_ref()?.wheels;
        let at = wheels.iter().position(|wheel| wheel.wheel_id == wheel_id)?;
        let wheel = wheels.get(at)?;
        (self.connected && self.held.is_none() && self.is_active() && self.wheel_spins(wheel) > 0)
            .then_some((at, wheel))
    }

    /// Every grant a spin of this wheel could pay, when a spin is allowed now (`None` when it
    /// is not). The caller refuses the spin unless all of them fit, so a refusal never depends
    /// on what would have been drawn.
    pub fn possible_grants(&self, wheel_id: i32, player_guid: u64) -> Option<Vec<RewardGrant>> {
        let (_, wheel) = self.spinnable(wheel_id)?;
        let prizes = wheel.slots.iter().flat_map(|slot| &slot.prizes);
        Some(prizes.filter_map(|prize| prize_grant(wheel, prize, player_guid)).collect())
    }

    /// Draw the wedge and its prize for an allowed spin request. Nothing is used up until
    /// [`Self::commit`].
    pub fn draw(&mut self, wheel_id: i32) -> Option<Draw> {
        let content = self.content.clone()?;
        let at = self.spinnable(wheel_id)?.0;
        let wheel = content.wheels.get(at)?;
        let (slot, prize) = match self.forced.take() {
            Some((slot, prize)) => (slot, wheel.slots.get(slot)?.prizes.get(prize)?),
            None => {
                let order: Vec<usize> = (0..wheel.slots.len()).collect();
                let weight = |at: &usize| wheel.slots.get(*at).map_or(0, |slot| slot.weight);
                let slot = *pick_weighted(order.iter().map(|at| (at, weight(at))), &mut self.rng)?;
                let prizes = &wheel.slots.get(slot)?.prizes;
                (slot, pick_weighted(prizes.iter().map(|p| (p, p.weight)), &mut self.rng)?)
            }
        };
        Some(Draw { wheel: at, slot, prize: prize.clone() })
    }

    /// The coins and items of a draw as one grant (`None` for a prize of spins alone).
    pub fn grant(&self, draw: &Draw, player_guid: u64) -> Option<RewardGrant> {
        let wheel = self.content.as_ref()?.wheels.get(draw.wheel)?;
        prize_grant(wheel, &draw.prize, player_guid)
    }

    /// A drawn spin was paid: use up the spin (the free one first), count the streak, bank the
    /// extra spins, hold what the stop will show, and answer with the wedge to stop on.
    /// `paid_coins` is what the coins really paid; `item` the first item's (icon, name id).
    pub fn commit(
        &mut self,
        draw: &Draw,
        paid_coins: u32,
        item: Option<(i32, i32)>,
        mut packets: Vec<Vec<u8>>,
    ) -> io::Result<Vec<u8>> {
        let content = self.content.clone().ok_or_else(|| invalid("no wheel content"))?;
        let wheel = content.wheels.get(draw.wheel).ok_or_else(|| invalid("no such wheel"))?;
        let today = self.today();
        let free = self.free_spin(wheel);
        let spins = self.ledger.0.entry(wheel.key.clone()).or_default();
        if free {
            let in_a_row = spins.day.is_some_and(|day| day.checked_add(1) == Some(today));
            spins.streak = if in_a_row { spins.streak.saturating_add(1) } else { 1 };
            spins.day = Some(today);
            let reached = spins.streak;
            if let Some(at) = wheel.streak_bonuses.iter().position(|row| row.days == reached) {
                let bonus = wheel.streak_bonuses[at];
                spins.extra = spins.extra.saturating_add(bonus.spins).min(MAX_BANKED);
                let next = wheel.streak_bonuses.get(at + 1);
                if next.is_none() {
                    spins.streak = 0;
                }
                packets.push(streak_packet(&content, bonus, next.copied()));
            }
        } else {
            spins.extra = spins.extra.saturating_sub(1);
        }
        spins.extra = spins.extra.saturating_add(draw.prize.spins).min(MAX_BANKED);
        let (icon, tooltip, quantity, name) = match (draw.prize.items.first(), paid_coins) {
            (Some(first), _) => {
                let (icon, name) = item.unwrap_or_default();
                (icon, first.definition_id, first.count, name)
            }
            (None, 1..) => (content.coin_icon_id, 0, paid_coins, content.coin_name_id),
            (None, 0) => (content.spin_icon_id, 0, draw.prize.spins, content.spin_name_id),
        };
        let body = if draw.prize.items.is_empty() && paid_coins == 0 {
            content.spin_message_id
        } else {
            wheel.reward_message_id
        };
        self.held = Some(Held {
            wheel: draw.wheel,
            window: format!("OnRewardInfoMsg\t{icon}\t{tooltip}\t{quantity}\t{name}\t{body}\t0"),
            packets,
        });
        self.message(&format!("OnSpinInfoMsg\t{}", draw.slot))
    }

    /// The wheel stopped: the reward window, the held reward packets, the wheel's spins (before
    /// the player can close the window, which reads them) and the list's count.
    pub fn stop(&mut self, wheel_id: i32) -> io::Result<Vec<Vec<u8>>> {
        let Some(content) = self.content.clone() else {
            return Ok(Vec::new());
        };
        let pending = (self.held.as_ref()).and_then(|held| content.wheels.get(held.wheel));
        let Some(wheel) = pending.filter(|wheel| wheel.wheel_id == wheel_id) else {
            return Ok(Vec::new());
        };
        let Some(held) = self.held.take() else {
            return Ok(Vec::new());
        };
        let mut out = vec![self.message(&held.window)?, self.message(&self.update_text(wheel))?];
        out.extend(held.packets);
        out.extend(self.spins_packet());
        Ok(out)
    }

    /// The reward packets of a paid spin whose stop never came, with the new count.
    fn take_unseen(&mut self) -> Vec<Vec<u8>> {
        let Some(held) = self.held.take() else {
            return Vec::new();
        };
        let mut out = held.packets;
        out.extend(self.spins_packet());
        out
    }

    /// End the run from the server's side (the movie's close, or a refused payout): anything not
    /// shown yet, then 39/19.
    pub fn close(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let (Some(instance), Some(content)) =
            (self.runs.current_instance_id(), self.content.as_ref())
        else {
            return Ok(Vec::new());
        };
        let removal = state_remove_packet(instance, content.minigame_id);
        let mut out = self.take_unseen();
        out.push(removal);
        self.runs.disconnect();
        self.connected = false;
        Ok(out)
    }
}

/// The coins and items of one prize as a grant (`None` for a prize of spins alone).
fn prize_grant(wheel: &Wheel, prize: &Prize, player_guid: u64) -> Option<RewardGrant> {
    (prize.coins > 0 || !prize.items.is_empty()).then(|| RewardGrant {
        coins: prize.coins,
        items: prize.items.clone(),
        job_xp: Default::default(),
        sc: 0,
        display: RewardDisplay {
            source_guid: 0,
            player_guid,
            reward_kind: 0,
            icon_id: 0,
            name_id: wheel.name_id,
            rows: Vec::new(),
        },
    })
}

/// 143/3 (`00adb070`): id, day, spin, string id, day2, spin2.
fn streak_packet(content: &Content, reached: StreakBonus, next: Option<StreakBonus>) -> Vec<u8> {
    let (message, day2, spin2) = match next {
        Some(next) => (content.streak_message_id, next.days, next.spins),
        None => (content.streak_last_message_id, 0, 0),
    };
    let mut bytes = vec![143, 0, 3];
    for value in [
        content.activity_id as i32,
        reached.days as i32,
        reached.spins as i32,
        message,
        day2 as i32,
        spin2 as i32,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn int_at(bytes: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
