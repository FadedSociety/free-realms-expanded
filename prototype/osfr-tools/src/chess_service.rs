//! Chess routing through the shared minigame lifecycle.
//!
//! `chess` owns the table rules and movie text protocol; `minigame` owns the
//! launch, start, result and close sequence. This adapter joins them for the
//! single-player rows the activity portal publishes:
//!
//! entry 167/1/2 Play -> 39/16 info -> client 39/5 -> 39/17 + 26/12 movie ->
//! 39/14 `OnConnectMsg` / `OnStartGameMsg` -> moves -> `OnGameWonMsg` ->
//! client `FRServer_GameOver` -> 39/18, 39/47, 39/19 and the coin payout ->
//! client 26/42 close.
//!
//! Original facts: the portal Play request, the family-39 lifecycle bodies and
//! the chess text protocol are in `notes/Build Guides/Gameplay/Chess.md` and
//! `notes/Build Guides/Gameplay/Minigame Framework.md`. The pairing of portal
//! row, chess row and payout, the reward amounts, the human colour and the
//! single-player-only scope are reconstructed policy (R). Two-player rows are
//! not published: pairing them needs cross-session routing that this slice
//! does not own.

use crate::{
    activity_portal::ChessActivity,
    chess::{self, ChessParticipant, ChessScoring, ChessTables, Mode, Side},
    minigame::{MinigameRuns, RunMode, RunOutcome, RunSpec, RunState},
    rewards::{RewardDisplay, RewardGrant, RewardState},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Optional content next to `field_battles.json`. Absent file = no chess rows.
pub const CONTENT_FILE: &str = "chess.json";
/// (R) Instance ids are per session; the shared lifecycle starts at 1.
pub const FIRST_INSTANCE_ID: i32 = 1;
const MAX_FILE: u64 = 64 * 1024;
const MAX_ROWS: usize = 12;
/// (R) Upper bound on a reconstructed coin award, so content cannot mint unlimited coins.
const MAX_COINS_PER_WIN: u32 = 10_000;
/// (R) Upper bound on the daily chess coin cap, so content cannot mint unlimited coins.
pub const MAX_DAILY_REWARD_COINS: u32 = 100_000;
/// (R) UTC day length used by the persisted daily ledger.
const DAY_MS: u64 = 86_400_000;

/// (R) The wall-clock UTC day index. The ledger must not use the gateway's
/// monotonic uptime (`now_ms`): restarting the process would reset the cap.
fn utc_day() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64 / DAY_MS)
}

fn default_daily_reward_coins() -> u32 {
    1_000
}

fn default_near_equal_margin() -> u32 {
    chess::DEFAULT_NEAR_EQUAL_MARGIN as u32
}

/// Why an ordinary entry was refused. Nothing changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryDenied {
    /// The id is not a published chess row.
    Unknown,
    /// A chess run is already offered, playing or awaiting its close.
    Busy,
    /// A two-player row (pairing is not routed yet).
    Unsupported,
    /// The offer could not be built from the loaded content.
    Unavailable,
}

impl EntryDenied {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unknown => "That chess table is not available.",
            Self::Busy => "Finish the current game before starting another.",
            Self::Unsupported => "Two-player chess needs another player at the table.",
            Self::Unavailable => "Chess is not open yet.",
        }
    }
}

/// One published single-player row. `row_id` is the client `MiniGameData` id;
/// `success_coins` is reconstructed (R) and tunable.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChessRow {
    pub row_id: u32,
    /// `MiniGameData` DIFFICULTY (1 to 5), sent in the 39/16 info record.
    pub difficulty: u32,
    /// Server-reply skill used by the local computer move chooser (0 to 100).
    pub skill: u32,
    pub success_coins: u32,
}

/// `chess.json`: a bounded, strict content file.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChessContent {
    pub schema_version: u32,
    /// Provenance note for the content checker; never gameplay input.
    #[serde(default)]
    pub source: Option<String>,
    /// "white" or "black". (R) One-player rows say "random order" in the
    /// locale; a fixed colour keeps the run deterministic and tunable.
    pub human_side: String,
    /// (R) Coins one character may earn from chess per UTC day. Generous by
    /// default; 0 disables chess rewards.
    #[serde(default = "default_daily_reward_coins")]
    pub daily_reward_coins: u32,
    /// (R) Score window, in `score_move` units, for the computer's near-equal
    /// move choice. Larger values make the computer less predictable; zero is
    /// refused because it collapses every reply to the single best move.
    #[serde(default = "default_near_equal_margin")]
    pub near_equal_margin: u32,
    /// (R) Required weights for the server's one-ply move chooser. The shipped
    /// `chess.json` carries every value; an omitted block is refused, never
    /// replaced with compiled tuning.
    pub scoring: ChessScoring,
    pub rows: Vec<ChessRow>,
}

impl ChessContent {
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(path)?
            .take(MAX_FILE + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_FILE {
            return Err(invalid("chess content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(|e| invalid(e.to_string()))?;
        content.validate()?;
        Ok(content)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1 {
            return Err(invalid("unsupported chess content version"));
        }
        if self.rows.is_empty() || self.rows.len() > MAX_ROWS {
            return Err(invalid("chess content needs 1 to 12 rows"));
        }
        if !matches!(self.human_side.as_str(), "white" | "black") {
            return Err(invalid("chess human_side must be white or black"));
        }
        if self.daily_reward_coins > MAX_DAILY_REWARD_COINS {
            return Err(invalid("chess daily reward cap exceeds the local bound"));
        }
        if !(1..=chess::MAX_NEAR_EQUAL_MARGIN).contains(&i64::from(self.near_equal_margin)) {
            return Err(invalid(
                "chess near-equal window must be 1 to the local bound",
            ));
        }
        self.scoring.validate()?;
        let mut seen = BTreeSet::new();
        for row in &self.rows {
            if row.row_id == 0 || row.row_id > i32::MAX as u32 || !seen.insert(row.row_id) {
                return Err(invalid("chess row ids must be unique positive i32 values"));
            }
            if !(1..=5).contains(&row.difficulty) {
                return Err(invalid("chess difficulty must be 1 to 5"));
            }
            if row.skill > 100 {
                return Err(invalid("chess skill must be 0 to 100"));
            }
            if row.success_coins > MAX_COINS_PER_WIN {
                return Err(invalid("chess coin award exceeds the local bound"));
            }
            if !matches!(Mode::for_row(row.row_id), Some(Mode::Single { .. })) {
                return Err(invalid("chess row is not a single-player type-6 row"));
            }
        }
        Ok(())
    }

    pub fn side(&self) -> Side {
        if self.human_side == "white" {
            Side::White
        } else {
            Side::Black
        }
    }

    /// Portal rows: the client row ids double as activity ids (they do not
    /// collide with the battle ids 96, 12301 and 12302).
    pub fn activities(&self) -> Vec<ChessActivity> {
        self.rows
            .iter()
            .map(|row| ChessActivity {
                row_id: row.row_id,
                name_id: chess::NAME_ID as u32,
                description_id: chess::DESCRIPTION_ID as u32,
                icon_id: chess::ICON_ID as u32,
                difficulty: row.difficulty,
            })
            .collect()
    }
}

/// Staged result of one accepted chess packet. The caller persists
/// `reward_state` with its inventory before sending packets.
#[derive(Default)]
pub struct ChessActivityOutcome {
    pub accepted: bool,
    pub changed: bool,
    pub replies: Vec<Vec<u8>>,
    pub reward_packets: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
}

/// (R) One character's daily chess coin ledger, persisted with the character
/// save as `GameplayState::chess` so a reconnect cannot reset the cap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChessRewardLedger {
    /// UTC day index from the wall clock (`unix_ms / 86_400_000`), independent
    /// of the process uptime the gateway reports as `now_ms`.
    pub day: u64,
    /// Coins already paid on `day`; a new day starts from zero.
    pub paid_coins: u32,
}

impl ChessRewardLedger {
    fn paid_on(&self, day: u64) -> u32 {
        if self.day == day { self.paid_coins } else { 0 }
    }

    fn record(&mut self, day: u64, coins: u32) {
        if self.day != day {
            *self = Self { day, paid_coins: 0 };
        }
        self.paid_coins = self.paid_coins.saturating_add(coins);
    }
}

/// One character's chess state. Cloneable so `Gameplay` can stage a packet and
/// discard it on refusal, exactly like the other activity adapters.
#[derive(Clone)]
pub struct ChessService {
    /// `None` when the OS seed was unavailable at load. The rows stay
    /// published, `begin` answers `Unavailable`, and the character/world load
    /// still succeeds.
    tables: Option<ChessTables>,
    runs: MinigameRuns,
    content: ChessContent,
    current: Option<ChessRow>,
    started_ms: Option<u64>,
    ledger: ChessRewardLedger,
    /// (R) Test/offline UTC day. Production leaves it `None` and follows the
    /// wall clock; the gateway never sets it.
    day: Option<u64>,
}

impl ChessService {
    /// Rebuild a character's service and restore the saved daily ledger. An
    /// unavailable OS seed leaves chess closed instead of failing the load.
    pub fn with_state(content: ChessContent, saved: Option<ChessRewardLedger>) -> io::Result<Self> {
        Self::with_entropy(content, ChessTables::entropy(), saved)
    }

    /// (R) Deterministic table seeds for tests and offline fixtures. The
    /// gateway's [`Self::with_state`] uses OS entropy; no client value reaches either.
    pub fn with_seed(content: ChessContent, seed: u64) -> io::Result<Self> {
        Self::with_entropy(content, Ok(ChessTables::with_seed(seed)), None)
    }

    /// (R) A fixed UTC day for tests and offline fixtures. The gateway never
    /// calls this; production reads the wall clock.
    pub fn set_day(&mut self, day: u64) {
        self.day = Some(day);
    }

    fn today(&self) -> u64 {
        self.day.unwrap_or_else(utc_day)
    }

    /// (R) A deterministic table stream for tests and offline fixtures. Only
    /// tables opened after this call use it; the gateway's constructors seed
    /// from OS entropy and never call this.
    pub(crate) fn reseed(&mut self, seed: u64) {
        let mut tables = self
            .tables
            .take()
            .unwrap_or_else(|| ChessTables::with_seed(seed));
        tables.reseed(seed);
        self.tables = Some(tables);
    }

    /// Build from the OS seed result. A failed source leaves chess closed
    /// rather than failing the character or world load; content errors still
    /// propagate.
    fn with_entropy(
        content: ChessContent,
        tables: io::Result<ChessTables>,
        saved: Option<ChessRewardLedger>,
    ) -> io::Result<Self> {
        content.validate()?;
        let tables = match tables {
            Ok(mut tables) => {
                tables.set_scoring(content.scoring)?;
                Some(tables)
            }
            Err(_) => None,
        };
        let mut service = Self {
            tables,
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            content,
            current: None,
            started_ms: None,
            ledger: ChessRewardLedger::default(),
            day: None,
        };
        service.restore_ledger(saved)?;
        Ok(service)
    }

    /// Restore the save's `chess` field. A malformed ledger is refused, never clamped.
    pub fn restore_ledger(&mut self, saved: Option<ChessRewardLedger>) -> io::Result<()> {
        match saved {
            None => self.ledger = ChessRewardLedger::default(),
            Some(ledger) if ledger.paid_coins > MAX_DAILY_REWARD_COINS => {
                return Err(invalid("saved chess ledger exceeds the local bound"));
            }
            Some(ledger) => self.ledger = ledger,
        }
        Ok(())
    }

    /// The save's `chess` field: absent until the first paid win.
    pub fn ledger(&self) -> Option<ChessRewardLedger> {
        (self.ledger.paid_coins > 0).then_some(self.ledger)
    }

    /// A run exists from the offer until the results Close (26/42) or a quit.
    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    pub fn activities(&self) -> Vec<ChessActivity> {
        self.content.activities()
    }

    /// 167/1/2 Play for a published chess row: publish the 39/16 offer.
    pub fn begin(
        &mut self,
        row_id: u32,
        player_id: u64,
    ) -> Result<ChessActivityOutcome, EntryDenied> {
        if player_id == 0 {
            return Err(EntryDenied::Unavailable);
        }
        if self.is_active() {
            return Err(EntryDenied::Busy);
        }
        if self.tables.is_none() {
            return Err(EntryDenied::Unavailable);
        }
        let Some(row) = self
            .content
            .rows
            .iter()
            .find(|row| row.row_id == row_id)
            .cloned()
        else {
            return Err(EntryDenied::Unknown);
        };
        if !matches!(Mode::for_row(row.row_id), Some(Mode::Single { .. })) {
            return Err(EntryDenied::Unsupported);
        }
        let mut staged = self.clone();
        let offered = staged
            .runs
            .offer(run_spec(&row))
            .map_err(|_| EntryDenied::Unavailable)?;
        if !offered.accepted {
            return Err(EntryDenied::Busy);
        }
        staged.current = Some(row);
        staged.started_ms = None;
        *self = staged;
        Ok(from_run(offered))
    }

    /// Route one client packet while a chess run exists. `None` means the
    /// packet belongs to another handler.
    pub fn route(
        &mut self,
        player_id: u64,
        packet: &[u8],
        now_ms: u64,
        rewards: &RewardState,
    ) -> io::Result<Option<ChessActivityOutcome>> {
        let mut staged = self.clone();
        let out = staged.route_inner(player_id, packet, now_ms, rewards)?;
        if out.is_some() {
            *self = staged;
        }
        Ok(out)
    }

    fn route_inner(
        &mut self,
        player_id: u64,
        packet: &[u8],
        now_ms: u64,
        rewards: &RewardState,
    ) -> io::Result<Option<ChessActivityOutcome>> {
        if !self.is_active() {
            return Ok(None);
        }
        if packet.len() >= 3 && packet[..2] == chess::FAMILY_MINIGAME.to_le_bytes() {
            let sub = packet[2];
            match sub {
                5 | 6 | 7 => {
                    let instance = self
                        .runs
                        .current_instance_id()
                        .ok_or_else(|| invalid("chess run vanished"))?;
                    let run = self.runs.handle_client(packet)?;
                    let mut out = from_run(run);
                    if sub == 5 && out.accepted {
                        self.start_table(instance, player_id, now_ms)?;
                        let playing = self.runs.started(instance);
                        out.accepted |= playing.accepted;
                        out.changed |= playing.changed;
                    } else if sub == 6 && out.accepted {
                        self.close_table(instance);
                    }
                    return Ok(Some(out));
                }
                14 => return self.handle_movie(player_id, packet, now_ms, rewards),
                _ => return Ok(Some(ChessActivityOutcome::default())),
            }
        }
        if packet == [26, 0, 42, 0] {
            let instance = self
                .runs
                .current_instance_id()
                .ok_or_else(|| invalid("chess run vanished"))?;
            let run = self.runs.handle_client(packet)?;
            // 26/42 retires only an offered or finished run; during play it is inert.
            if run.changed {
                self.close_table(instance);
            }
            return Ok(Some(from_run(run)));
        }
        Ok(None)
    }

    fn handle_movie(
        &mut self,
        player_id: u64,
        packet: &[u8],
        now_ms: u64,
        rewards: &RewardState,
    ) -> io::Result<Option<ChessActivityOutcome>> {
        let Some(instance) = self.runs.current_instance_id() else {
            return Ok(Some(ChessActivityOutcome::default()));
        };
        // Movie traffic before START and after settlement is inert.
        if self.runs.state(instance) != Some(RunState::Playing) {
            return Ok(Some(ChessActivityOutcome::default()));
        }
        let table = self
            .tables
            .as_mut()
            .ok_or_else(|| invalid("chess tables are not seeded"))?
            .handle_payload(instance as u64, player_id, packet)?;
        let terminal = table.terminal.is_some();
        let mut out = ChessActivityOutcome {
            accepted: table.accepted,
            changed: table.changed,
            replies: table
                .messages
                .into_iter()
                .map(|message| message.packet)
                .collect(),
            ..Default::default()
        };
        if terminal {
            // A decisive result waits for the client's FRServer_GameOver; the
            // table's settle refuses until that acknowledgement. Draws settle
            // here because the movie sends nothing after OnGameStoppedMsg.
            self.settle(instance, player_id, now_ms, rewards, &mut out)?;
        }
        Ok(Some(out))
    }

    fn start_table(&mut self, instance: i32, player_id: u64, now_ms: u64) -> io::Result<()> {
        let row = self
            .current
            .as_ref()
            .ok_or_else(|| invalid("chess offer has no row"))?;
        self.tables
            .as_mut()
            .ok_or_else(|| invalid("chess tables are not seeded"))?
            .open_single_with_skill(
                instance as u64,
                row.row_id,
                ChessParticipant {
                    player_id,
                    instance_id: instance,
                },
                self.content.side(),
                row.skill as u8,
                i64::from(self.content.near_equal_margin),
            )?;
        self.started_ms = Some(now_ms);
        Ok(())
    }

    fn settle(
        &mut self,
        instance: i32,
        player_id: u64,
        now_ms: u64,
        rewards: &RewardState,
        out: &mut ChessActivityOutcome,
    ) -> io::Result<()> {
        let Some(row) = self.current.as_ref() else {
            return Ok(());
        };
        let won = self
            .tables
            .as_ref()
            .and_then(|tables| tables.table(instance as u64))
            .and_then(|table| table.result())
            .and_then(|result| result.winner)
            == Some(player_id);
        let elapsed = self
            .started_ms
            .map_or(0, |start| now_ms.saturating_sub(start) / 1000);
        let elapsed = u32::try_from(elapsed).unwrap_or(u32::MAX);
        // (R) Chess score and stars were never recovered; the results page
        // shows zero while the coin award comes from content.
        // The daily cap is checked before the grant is staged: a capped win
        // still settles the run but pays nothing.
        let day = self.today();
        let remaining = self
            .content
            .daily_reward_coins
            .saturating_sub(self.ledger.paid_on(day));
        let grant = won
            .then(|| row.win_grant(player_id))
            .filter(|grant| grant.coins <= remaining);
        // The ledger counts what the grant actually pays, not the row's
        // nominal award, so a future grant adjustment cannot skew the cap.
        let paid_coins = grant.as_ref().map(|grant| grant.coins);
        let settlement = grant.as_ref().map(|grant| (rewards, grant));
        let settled = self
            .tables
            .as_mut()
            .ok_or_else(|| invalid("chess tables are not seeded"))?
            .settle_player(
                instance as u64,
                player_id,
                &mut self.runs,
                0,
                0,
                elapsed,
                settlement,
            )?;
        out.accepted |= settled.accepted;
        out.changed |= settled.changed;
        out.replies.extend(settled.replies);
        out.reward_packets.extend(settled.reward_packets);
        if settled.reward_state.is_some() {
            out.reward_state = settled.reward_state;
            if let Some(coins) = paid_coins {
                self.ledger.record(day, coins);
            }
        }
        Ok(())
    }

    fn close_table(&mut self, instance: i32) {
        if let Some(tables) = &mut self.tables {
            tables.close(instance as u64);
        }
        self.current = None;
        self.started_ms = None;
    }
}

impl ChessRow {
    fn win_grant(&self, player_id: u64) -> RewardGrant {
        RewardGrant {
            coins: self.success_coins,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            display: RewardDisplay {
                source_guid: player_id,
                player_guid: player_id,
                reward_kind: 0,
                icon_id: chess::ICON_ID,
                name_id: chess::NAME_ID,
                rows: Vec::new(),
            },
        }
    }
}

fn run_spec(row: &ChessRow) -> RunSpec {
    let mut spec = RunSpec::empty(
        row.row_id as i32,
        chess::GAME_TYPE,
        RunMode::Flash {
            movie: chess::MOVIE.to_owned(),
        },
    );
    spec.name_id = chess::NAME_ID;
    spec.icon_id = chess::ICON_ID;
    spec.description_id = chess::DESCRIPTION_ID;
    spec.difficulty = row.difficulty as i32;
    spec
}

fn from_run(run: RunOutcome) -> ChessActivityOutcome {
    ChessActivityOutcome {
        accepted: run.accepted,
        changed: run.changed,
        replies: run.replies,
        reward_packets: run.reward_packets,
        reward_state: run.reward_state,
    }
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn shipped_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/chess.json")
    }

    #[test]
    fn chess_scoring_is_required_content() {
        let path = shipped_path();
        ChessContent::load(&path).unwrap();
        let shipped: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();

        // (R) A pack that omits the block must not silently use compiled tuning.
        let mut missing = shipped.clone();
        missing.as_object_mut().unwrap().remove("scoring");
        assert!(
            serde_json::from_value::<ChessContent>(missing).is_err(),
            "an omitted scoring block is refused"
        );

        // A partial block must not mix shipped and compiled values either.
        let mut partial = shipped;
        partial["scoring"].as_object_mut().unwrap().remove("queen");
        assert!(
            serde_json::from_value::<ChessContent>(partial).is_err(),
            "a partial scoring block is refused"
        );
    }

    #[test]
    fn an_unavailable_os_seed_closes_chess_without_failing_the_load() {
        // `with_state` delegates to this constructor with the `getrandom`
        // result; a failure must disable chess, not the character/world load.
        let content = ChessContent::load(&shipped_path()).unwrap();
        let failed = Err(io::Error::other("no OS randomness"));
        let mut service = ChessService::with_entropy(content, failed, None).unwrap();
        assert!(!service.is_active());
        let rows = service.activities();
        assert!(!rows.is_empty(), "the portal rows stay published");
        assert!(matches!(
            service.begin(20, 71),
            Err(EntryDenied::Unavailable)
        ));
        assert!(!service.is_active(), "the refusal opened nothing");
        assert_eq!(service.activities(), rows, "the refusal changed the rows");
        assert_eq!(service.ledger(), None);
    }

    #[test]
    fn the_ledger_day_follows_the_clock_not_the_gateway_uptime() {
        let content = ChessContent::load(&shipped_path()).unwrap();
        // A real character starts on the wall-clock day, not on day zero.
        let unseeded = ChessService::with_seed(content.clone(), 22).unwrap();
        assert!(
            unseeded.today() > 0,
            "the wall clock supplies a real UTC day"
        );
        // Tests and offline fixtures can pin the day without the clock.
        let mut pinned = ChessService::with_seed(content, 22).unwrap();
        pinned.set_day(20_000);
        assert_eq!(pinned.today(), 20_000);
    }
}
