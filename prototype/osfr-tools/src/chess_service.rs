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
    chess::{self, ChessParticipant, ChessTables, Mode, Side},
    minigame::{MinigameRuns, RunMode, RunOutcome, RunSpec, RunState},
    rewards::{RewardDisplay, RewardGrant, RewardState},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
};

/// Optional content next to `field_battles.json`. Absent file = no chess rows.
pub const CONTENT_FILE: &str = "chess.json";
/// (R) Instance ids are per session; the shared lifecycle starts at 1.
pub const FIRST_INSTANCE_ID: i32 = 1;
const MAX_FILE: u64 = 64 * 1024;
const MAX_ROWS: usize = 12;
/// (R) Upper bound on a reconstructed coin award, so content cannot mint unlimited coins.
const MAX_COINS_PER_WIN: u32 = 10_000;

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

/// One character's chess state. Cloneable so `Gameplay` can stage a packet and
/// discard it on refusal, exactly like the other activity adapters.
#[derive(Clone)]
pub struct ChessService {
    tables: ChessTables,
    runs: MinigameRuns,
    content: ChessContent,
    current: Option<ChessRow>,
    started_ms: Option<u64>,
}

impl ChessService {
    pub fn new(content: ChessContent) -> io::Result<Self> {
        content.validate()?;
        Ok(Self {
            tables: ChessTables::default(),
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            content,
            current: None,
            started_ms: None,
        })
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
        self.tables.open_single_with_skill(
            instance as u64,
            row.row_id,
            ChessParticipant {
                player_id,
                instance_id: instance,
            },
            self.content.side(),
            row.skill as u8,
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
            .table(instance as u64)
            .and_then(|table| table.result())
            .and_then(|result| result.winner)
            == Some(player_id);
        let elapsed = self
            .started_ms
            .map_or(0, |start| now_ms.saturating_sub(start) / 1000);
        let elapsed = u32::try_from(elapsed).unwrap_or(u32::MAX);
        // (R) Chess score and stars were never recovered; the results page
        // shows zero while the coin award comes from content.
        let grant = won.then(|| row.win_grant(player_id));
        let settlement = grant.as_ref().map(|grant| (rewards, grant));
        let settled = self.tables.settle_player(
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
        }
        Ok(())
    }

    fn close_table(&mut self, instance: i32) {
        self.tables.close(instance as u64);
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
