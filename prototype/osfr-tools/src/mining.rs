//! Mining18 (TYPE8) shared match-three engine for the original
//! `game_mining.gfx` movie.
//!
//! Recovered from the frozen client assets (provenance outside Git):
//! - outer `39/<sub>` envelope, `39/14` tab-delimited payloads, `26/12`
//!   `MiniGameFlash` launcher, `39/16` info, `39/17` start, `39/18` game
//!   over, `39/19` removal and `39/47` score rows;
//! - server messages `OnServerReadyMsg`, `OnNextLevelMsg`, `OnBuildMapMsg`,
//!   `OnTileQueueMsg`, `OnRandomQueueMsg`, `OnLevelInitMsg`,
//!   `OnQuotaInitMsg`, `OnQuotaUpdateMsg`, `OnLevelEventMsg`,
//!   `OnPowerupInitMsg`, `OnStopLevel`, `OnGameWonMsg`, `OnGameLostMsg`;
//! - `aMyGrid` shape codes `0` empty, `1` regular random, `2` any random,
//!   `3` blocker, `4` dropzone, negative = specific value, positive =
//!   category with random value;
//! - chain rules: at least three orthogonally adjacent, distinct idle
//!   selectable tiles of one value, `200` wildcards match and adopt the
//!   chain value;
//! - queue discipline 50 minimum / 20 batch / 70 target, random queue
//!   `0..=999`;
//! - quota math: `OnQuotaInitMsg(index,value,quotaTiles,perReward,iconId)`,
//!   collected tiles drop into the bin, clusters are
//!   `floor(collected / perReward)`, a goal completes at
//!   `collected >= quotaTiles`.
//!
//! Reconstructed (explicitly not original): every concrete board layout,
//! quota, spawn percentage, seed, RNG-to-tile mapping, output mapping, the
//! instance-id namespace and the operational caps (board cells, level timer).
//! The original MatchThree server level tables were not recovered; see
//! `notes/Research/Mining Restoration.md`.
//!
//! The engine is data driven: load a `mining.json` content file and drive it
//! with the real client. It owns board, queue, RNG mapping, score, timer,
//! goal progress and the win/loss lifecycle; the caller persists the whole
//! [`Type8State`] before committing replies or any reward transaction.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

#[path = "mining_lifecycle.rs"]
#[allow(dead_code)] // Mining engine tests include this file without exercising the adapter.
pub mod lifecycle;

pub const TYPE8_GAME_FAMILY: u16 = 39;
pub const MINING_GAME_ID: i32 = 18;
pub const COPPER_MINING_GAME_ID: i32 = 91;
/// Gateway content file name for the reconstructed Mining18 board.
pub const CONTENT_FILE: &str = "mining.json";
/// ORIGINAL: the opening content binds the accepted Miner job to profile 14.
pub const MINER_PROFILE_ID: i32 = 14;
/// ORIGINAL: client locale ids recovered for the Mining activity.
pub const MINING_NAME_ID: i32 = 7910;
pub const MINING_DESCRIPTION_ID: i32 = 7911;
/// Client icon index carried in the recovered info packet and ore goals.
pub const MINING_ICON_ID: i32 = 909;
pub const SIDE: usize = 14;
pub const TILE_EMPTY: u16 = 0;
pub const TILE_DROPZONE: u16 = 100;
pub const TILE_WILDCARD: u16 = 200;
pub const TILE_SELECTABLE_START: u16 = 400;
pub const TILE_COLLECTABLE_START: u16 = 600;
pub const TILE_BLOCKER: u16 = 800;
/// ORIGINAL: `GameData.TILE_POWERUP`; `900..=949` is the collected range, at
/// `GameData.TILE_POWERUP_PRESSME` (950) the four clickable originals begin
/// (`core/TileGenerator.generate_powerup`).
pub const TILE_POWERUP_START: u16 = 900;
pub const TILE_POWERUP_PRESSME: u16 = 950;
/// ORIGINAL: `TILE_POWERUP_DYNAMITE`, `_WILDTILE`, `_DESTRUCT`, `_SHUFFLE`.
pub const TILE_POWERUP_DYNAMITE: u16 = 950;
pub const TILE_POWERUP_WILDTILE: u16 = 951;
pub const TILE_POWERUP_DESTRUCT: u16 = 952;
pub const TILE_POWERUP_SHUFFLE: u16 = 953;

/// Recovered mining preload asset list (`Assets_005.pack` entry 3842; 129
/// names ending in `game_mining.gfx`), sent in `39/16`.
pub const MINING_PRELOAD_FILE: &str = "MiningAssets.txt";
/// Recovered mining movie (`AssetsW_008.pack` entry 1399; `.lst` srcfilename
/// `.../Minigames/MatchThree/Mining/game_mining.swf`), launched through `26/12`.
pub const MINING_LAUNCH_MOVIE: &str = "game_mining.gfx";

fn default_preload_file() -> String {
    MINING_PRELOAD_FILE.to_string()
}

fn default_launch_movie() -> String {
    MINING_LAUNCH_MOVIE.to_string()
}

/// RECONSTRUCTED: first instance id of the mining namespace (18_000_001).
pub const FIRST_MINING_INSTANCE: i32 = 18_000_001;
const MIN_INSTANCE_ID: i32 = 1_000_000;
const MAX_CONTENT: u64 = 64 * 1024;
const MAX_PAYLOAD: usize = 4096;
const MAX_WALL_MS: u32 = 1_800_000;
const MAX_GOALS: usize = 8;
/// Reconstructed operational bound shared by the content validator and the
/// run validator: one run never holds more than this many board cells.
const MAX_BOARD_CELLS: usize = SIDE * 32;
/// Reconstructed operational bound shared by the content validator and the
/// run validator: the level timer, including chain bonuses, stays under ten
/// minutes.
const MAX_REMAINING_MS: u32 = 600_000;
const QUEUE_MIN: usize = 50;
const QUEUE_BATCH: usize = 20;
const QUEUE_TARGET: usize = QUEUE_MIN + QUEUE_BATCH;
/// Reconstructed bound on one clickable powerup's reach; the original level
/// table supplied the real `power` values and they were not recovered.
const MAX_POWERUP_POWER: u32 = 64;
const MAX_POWERUP_DEFINITIONS: usize = 4;
const RANDOM_MAX_VALUE: u32 = 999;
const CHAIN_SCORE: u32 = 400;
const CHAIN_TILE_SCORE: u32 = 200;
const COLLECTABLE_SCORE: u32 = 500;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Type8Kind {
    pub value: u16,
    pub category: u16,
    pub icon_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Type8Goal {
    pub index: i32,
    pub value: u16,
    /// Quota in tiles, as sent by `OnQuotaInitMsg`.
    pub quota: u32,
    pub per_reward: u32,
    pub icon_id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Type8Output {
    pub value: u16,
    pub definition_id: i32,
    /// Reconstructed grant per completed cluster of `value`.
    pub cluster_quantity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Type8BoardSpec {
    pub width: usize,
    pub height: usize,
    /// Flattened original `aMyGrid`, `width * height` row-major entries.
    pub shape: Vec<i32>,
    pub selectable: Vec<u16>,
    pub collectable: Vec<u16>,
    pub tiles: Vec<Type8Kind>,
    pub goals: Vec<Type8Goal>,
    pub outputs: Vec<Type8Output>,
    pub duration_seconds: u32,
    pub seed: u32,
    pub wildcard_percent: u32,
    pub collectable_percent: u32,
    pub blocker_percent: u32,
    pub powerup_percent: u32,
    /// `OnLevelEventMsg 32` meter type; `0` means no mold meter.
    pub meter_id: u32,
    pub reconstructed_rules: bool,
    pub rule_notice: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type8Phase {
    Offered,
    Launched,
    Connected,
    LevelOffered,
    Playing,
    LevelComplete,
    Won,
    Lost,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Type8Run {
    pub instance_id: i32,
    pub phase: Type8Phase,
    pub board: Vec<u16>,
    pub idle: Vec<bool>,
    pub tile_queue: Vec<u16>,
    pub random_queue: Vec<u16>,
    pub rng: u32,
    /// Collected tiles per goal, same order as `Type8BoardSpec::goals`.
    pub collected: Vec<u32>,
    pub score: u32,
    pub remaining_ms: u32,
    pub wall_ms: u32,
    pub paused: bool,
    pub chains: u32,
    pub reshuffles: u32,
    pub end_sent: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Type8State {
    pub version: u32,
    pub next_instance_id: i32,
    pub completed: u32,
    #[serde(default)]
    pub active: Option<Type8Run>,
}

impl Default for Type8State {
    fn default() -> Self {
        Self {
            version: 1,
            next_instance_id: FIRST_MINING_INSTANCE,
            completed: 0,
            active: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type8Transaction {
    pub consumes: Vec<(i32, u32)>,
    pub grants: Vec<(i32, u32)>,
}

#[derive(Debug, Default)]
pub struct Type8Outcome {
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
    pub completed: bool,
    pub transaction: Option<Type8Transaction>,
}

#[derive(Clone, Debug, Deserialize)]
struct Content {
    schema_version: u32,
    game_id: i32,
    profile_id: i32,
    name_id: i32,
    description_id: i32,
    icon_id: i32,
    /// Preload asset-list name advertised in `39/16`. Defaults to the
    /// recovered mining list so existing `mining.json` files keep their bytes.
    #[serde(default = "default_preload_file")]
    preload_file: String,
    /// Movie requested through `26/12 MiniGameFlash`. Defaults to the
    /// recovered mining movie.
    #[serde(default = "default_launch_movie")]
    launch_movie: String,
    /// RECONSTRUCTED private authoring surface for the four clickable powerups
    /// (`950..=953`). Kept off `Type8BoardSpec` so the public board API, the
    /// writer bytes and every existing content file stay unchanged; a content
    /// file without this key behaves exactly like the pre-powerup engine.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    powerup_definitions: Vec<Type8PowerupDefinition>,
    spec: Type8BoardSpec,
}

/// RECONSTRUCTED per-powerup authoring row. The original table lived in the
/// server `SpawnData` rows (`category 3`) and was never shipped in the movie, so
/// every value here is caller-authored tuning, not recovered balance.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Type8PowerupDefinition {
    /// Tile value; only `950..=953` are currently honoured.
    pub value: u16,
    /// Spawn chance in percent (`SpawnData.GetSpawnChance`).
    #[serde(default)]
    pub spawn_chance: u32,
    /// `SpawnData.GetDuration`; carried to the client through
    /// `OnPowerupInitMsg` and advertised only.
    #[serde(default)]
    pub duration: u32,
    /// `SpawnData.GetPower`; the original cap on tiles an action may touch.
    #[serde(default)]
    pub power: u32,
}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    left: usize,
    top: usize,
    width: usize,
    height: usize,
}

#[derive(Clone, Debug)]
pub struct Type8Engine {
    content: Arc<Content>,
    game_id: i32,
    first_instance_id: i32,
    bounds: Bounds,
    state: Type8State,
    results_presented: bool,
}

impl Type8Engine {
    pub fn load(
        directory: &Path,
        content_file: &str,
        game_id: i32,
        first_instance_id: i32,
        state: Option<Type8State>,
    ) -> io::Result<Self> {
        if content_file.is_empty() || content_file.contains(['/', '\\']) {
            return Err(invalid("invalid content file name"));
        }
        let path = directory.join(content_file);
        if fs::metadata(&path)?.len() > MAX_CONTENT {
            return Err(invalid("mining content too large"));
        }
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_CONTENT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_CONTENT {
            return Err(invalid("mining content too large"));
        }
        let content: Content = serde_json::from_slice(&bytes).map_err(invalid)?;
        if content.schema_version != 1 || content.game_id != game_id {
            return Err(invalid("mining content game binding mismatch"));
        }
        validate_asset_name(&content.preload_file, "preload file")?;
        validate_asset_name(&content.launch_movie, "launch movie")?;
        let bounds = validate_spec(&content.spec)?;
        validate_powerup_definitions(&content)?;
        let state = state.unwrap_or(Type8State {
            version: 1,
            next_instance_id: first_instance_id,
            completed: 0,
            active: None,
        });
        Self::validate_state(&state)?;
        if state.next_instance_id < first_instance_id
            || state.completed > (state.next_instance_id - first_instance_id) as u32
        {
            return Err(invalid("mining state outside its instance namespace"));
        }
        if let Some(run) = &state.active {
            validate_run(run, &content.spec, bounds)?;
        }
        Ok(Self {
            content: Arc::new(content),
            game_id,
            first_instance_id,
            bounds,
            state,
            results_presented: false,
        })
    }

    pub fn state(&self) -> &Type8State {
        &self.state
    }

    pub fn spec(&self) -> &Type8BoardSpec {
        &self.content.spec
    }

    /// Preload asset-list name advertised to the client in `39/16`.
    pub fn preload_file(&self) -> &str {
        &self.content.preload_file
    }

    /// Movie requested through `26/12 MiniGameFlash` for this content.
    pub fn launch_movie(&self) -> &str {
        &self.content.launch_movie
    }

    /// The job profile this content binds; `begin` refuses every other id.
    pub fn profile_id(&self) -> i32 {
        self.content.profile_id
    }

    pub fn active(&self) -> bool {
        self.state.active.is_some()
    }

    /// Test-only board seeding so a powerup fixture does not depend on the RNG
    /// rolling the exact tile it wants to exercise. Refuses anything the run
    /// validator would reject.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn seed_board(&mut self, cells: &[(usize, usize, u16)]) -> io::Result<()> {
        let Some(run) = self.state.active.as_mut() else {
            return Err(invalid("mining has no active run"));
        };
        if run.phase != Type8Phase::Playing {
            return Err(invalid("mining is not playing"));
        }
        for &(x, y, value) in cells {
            if !in_board(x, y, self.bounds) || !valid_value(value) {
                return Err(invalid("invalid mining seed"));
            }
            let i = board_index(&self.content.spec, (x, y));
            run.board[i] = value;
            run.idle[i] = true;
        }
        Ok(())
    }

    /// Test-only random-number-queue seeding so forge/order fixtures can make
    /// the recovered server draw deterministic without a live client.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn seed_random_queue(&mut self, values: &[u16]) -> io::Result<()> {
        let Some(run) = self.state.active.as_mut() else {
            return Err(invalid("mining has no active run"));
        };
        if run.phase != Type8Phase::Playing {
            return Err(invalid("mining is not playing"));
        }
        if values.is_empty() || values.len() > QUEUE_TARGET * 2 {
            return Err(invalid("mining random queue outside its bounds"));
        }
        run.random_queue = values.to_vec();
        Ok(())
    }

    /// Test-only replacement-tile-queue seeding so a fixture can keep its exact
    /// board after the post-action `gravity` pass.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn seed_tile_queue(&mut self, values: &[u16]) -> io::Result<()> {
        let Some(run) = self.state.active.as_mut() else {
            return Err(invalid("mining has no active run"));
        };
        if run.phase != Type8Phase::Playing {
            return Err(invalid("mining is not playing"));
        }
        if values.len() > QUEUE_TARGET * 2 || values.iter().any(|value| !valid_value(*value)) {
            return Err(invalid("invalid mining tile queue seed"));
        }
        run.tile_queue = values.to_vec();
        Ok(())
    }

    /// Test-only view of the recovered future-move rule.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn has_future_move(&self) -> bool {
        match &self.state.active {
            Some(run) => has_move(run, &self.content.spec, self.bounds),
            None => false,
        }
    }

    pub fn validate_state(s: &Type8State) -> io::Result<()> {
        if s.version != 1 || s.next_instance_id < MIN_INSTANCE_ID || s.completed > 1_000_000 {
            return Err(invalid("invalid mining state"));
        }
        if let Some(a) = &s.active {
            if a.instance_id < 1
                || a.instance_id >= s.next_instance_id
                || a.board.len() > MAX_BOARD_CELLS
                || a.idle.len() != a.board.len()
                || a.tile_queue.len() > QUEUE_TARGET * 2
                || a.random_queue.len() > QUEUE_TARGET * 2
                || a.rng == 0
                || a.collected.len() > MAX_GOALS
                || a.score > 1_000_000_000
                || a.remaining_ms > MAX_REMAINING_MS
                || a.wall_ms > MAX_WALL_MS
                || a.chains > 100_000
                || a.reshuffles > 1024
                || a.board.iter().any(|v| !valid_value(*v))
                || (a.phase == Type8Phase::Won && s.completed == 0)
                || (a.end_sent && !matches!(a.phase, Type8Phase::Won | Type8Phase::Lost))
            {
                return Err(invalid("invalid mining run"));
            }
        }
        Ok(())
    }

    /// The caller checks the accepted Miner job, proximity and any content
    /// prerequisite before offering a run.
    pub fn begin(&mut self, profile: i32) -> io::Result<Type8Outcome> {
        if profile != self.content.profile_id || self.active() {
            return Err(invalid("mining requires an idle Miner"));
        }
        let id = self.state.next_instance_id;
        let next = id
            .checked_add(1)
            .ok_or_else(|| invalid("mining IDs exhausted"))?;
        if id < self.first_instance_id {
            return Err(invalid("mining instance below its namespace"));
        }
        let rng = self.content.spec.seed ^ (id as u32).wrapping_mul(0x9e37_79b9);
        let run = Type8Run {
            instance_id: id,
            phase: Type8Phase::Offered,
            board: Vec::new(),
            idle: Vec::new(),
            tile_queue: Vec::new(),
            random_queue: Vec::new(),
            rng: rng.max(1),
            collected: vec![0; self.content.spec.goals.len()],
            score: 0,
            remaining_ms: self.content.spec.duration_seconds * 1000,
            wall_ms: 0,
            paused: true,
            chains: 0,
            reshuffles: 0,
            end_sent: false,
        };
        self.state.next_instance_id = next;
        self.state.active = Some(run);
        self.results_presented = false;
        Ok(Type8Outcome {
            replies: vec![self.info_packet(id)],
            changed: true,
            ..Default::default()
        })
    }

    /// Reconnect cannot resume an old client tile queue: drop the run. The
    /// actual native results close is `26/42`, not the `39/7` removal ACK.
    pub fn cancel(&mut self) -> Type8Outcome {
        let Some(run) = self.state.active.take() else {
            return Type8Outcome::default();
        };
        let replies = if self.results_presented {
            Vec::new()
        } else {
            vec![header(19, run.instance_id, self.game_id)]
        };
        self.results_presented = false;
        Type8Outcome {
            replies,
            changed: true,
            ..Default::default()
        }
    }

    /// Elapsed server time in milliseconds. Never client-reported.
    pub fn tick(&mut self, elapsed_ms: u32) -> io::Result<Type8Outcome> {
        let Some(current) = &self.state.active else {
            return Ok(Type8Outcome::default());
        };
        if elapsed_ms == 0 || matches!(current.phase, Type8Phase::Won | Type8Phase::Lost) {
            return Ok(Type8Outcome::default());
        }
        let mut next = self.state.clone();
        let run = next.active.as_mut().unwrap();
        run.wall_ms = run.wall_ms.saturating_add(elapsed_ms).min(MAX_WALL_MS);
        let mut out = Type8Outcome {
            changed: true,
            ..Default::default()
        };
        if run.phase == Type8Phase::Playing && !run.paused {
            run.remaining_ms = run.remaining_ms.saturating_sub(elapsed_ms);
        }
        if run.wall_ms == MAX_WALL_MS || (run.phase == Type8Phase::Playing && run.remaining_ms == 0)
        {
            run.phase = Type8Phase::Lost;
            // Whole-movie pause suppresses the native end-stage timers, so a
            // paused loss resumes the movie before StopLevel.
            if run.paused {
                message(&mut out, run.instance_id, self.game_id, "OnPauseMsg\ttrue")?;
                run.paused = false;
            }
            message(
                &mut out,
                run.instance_id,
                self.game_id,
                "OnStopLevel\t1\tfalse",
            )?;
        }
        self.state = next;
        Ok(out)
    }

    pub fn handle(&mut self, inner: &[u8]) -> io::Result<Type8Outcome> {
        if inner.len() < 2 || inner[..2] != TYPE8_GAME_FAMILY.to_le_bytes() {
            return Ok(Type8Outcome::default());
        }
        if inner.len() < 15 {
            return Err(invalid("truncated minigame header"));
        }
        let sub = inner[2];
        if !matches!(sub, 5 | 6 | 7 | 14) {
            return Ok(Type8Outcome::default());
        }
        let msg = if sub == 14 {
            Some(decode_payload(inner)?)
        } else {
            if inner.len() != 15 {
                return Err(invalid("invalid mining request length"));
            }
            None
        };
        let Some(run) = &self.state.active else {
            return Ok(Type8Outcome::default());
        };
        if sub == 7 {
            // Removal ACK; the native results close is 26/42.
            return Ok(Type8Outcome::default());
        }
        if sub == 6 {
            // Shared original Ui.QuitMiniGame serializes the current instance
            // with both sentinels. Ordinary Quit cancels only a pre-win run.
            if matches!(run.phase, Type8Phase::Won | Type8Phase::Lost)
                || read_i32(inner, 3) != run.instance_id
                || read_i32(inner, 7) != -1
                || read_i32(inner, 11) != -1
            {
                return Ok(Type8Outcome::default());
            }
            return Ok(self.cancel());
        }
        let game = read_i32(inner, 11);
        let native_offer_start = sub == 5 && run.phase == Type8Phase::Offered && game == -1;
        let native_payload = sub == 14 && run.phase != Type8Phase::Offered && game == -1;
        if read_i32(inner, 3) != run.instance_id
            || read_i32(inner, 7) != -1
            || (game != self.game_id && !native_offer_start && !native_payload)
        {
            return Ok(Type8Outcome::default());
        }
        let mut next = self.state.clone();
        let run = next.active.as_mut().unwrap();
        let id = run.instance_id;
        let mut out = Type8Outcome::default();
        let mut presented = self.results_presented;
        if sub == 5 {
            if run.phase == Type8Phase::Offered {
                out.replies.push(header(17, id, self.game_id));
                out.replies.push(launch_packet(&self.content.launch_movie));
                run.phase = Type8Phase::Launched;
            }
        } else {
            let fields: Vec<_> = msg.unwrap().split('\t').collect();
            match fields.as_slice() {
                ["OnConnectMsg"] => {
                    if run.phase == Type8Phase::Launched {
                        run.phase = Type8Phase::Connected;
                        message(&mut out, id, self.game_id, "OnServerReadyMsg")?;
                    }
                }
                ["FRServer_Connect"] | ["FRServer_ReadyForGameData"] => {
                    // Wait for the match-three network callback to install and
                    // send OnConnectMsg before ServerReady.
                }
                ["OnStartGameMsg"] | ["OnStartGameMsg", "0"] => {
                    if run.phase == Type8Phase::Connected {
                        run.phase = Type8Phase::LevelOffered;
                        message(&mut out, id, self.game_id, "OnNextLevelMsg\t1\t0\t0")?;
                    }
                }
                ["OnStartLevelMsg"] | ["OnStartLevelMsg", "0"] => {
                    if run.phase == Type8Phase::LevelOffered {
                        run.phase = Type8Phase::Playing;
                        run.paused = false;
                        initialize_board(
                            run,
                            &self.content.spec,
                            &mut out,
                            self.game_id,
                            false,
                            &self.content.powerup_definitions,
                        )?;
                        message(
                            &mut out,
                            id,
                            self.game_id,
                            &format!(
                                "OnLevelInitMsg\t1\t{}\t3",
                                self.content.spec.duration_seconds
                            ),
                        )?;
                        for (index, goal) in self.content.spec.goals.iter().enumerate() {
                            message(
                                &mut out,
                                id,
                                self.game_id,
                                &format!(
                                    "OnQuotaInitMsg\t{index}\t{}\t{}\t{}\t{}",
                                    goal.value, goal.quota, goal.per_reward, goal.icon_id
                                ),
                            )?;
                        }
                        let total: u32 = self.content.spec.goals.iter().map(|g| g.quota).sum();
                        message(
                            &mut out,
                            id,
                            self.game_id,
                            &format!(
                                "OnLevelEventMsg\t32\t{}\t{}",
                                self.content.spec.meter_id, total
                            ),
                        )?;
                        message(&mut out, id, self.game_id, "OnActiveMsg\ttrue")?;
                    }
                }
                ["OnChainMsg", list] => {
                    let chain = coordinates(list)?;
                    if run.phase == Type8Phase::Playing {
                        if run.paused {
                            return Err(invalid("mining is paused"));
                        }
                        let chain_value =
                            validate_chain(run, &chain, &self.content.spec, self.bounds)?;
                        let width = self.content.spec.width;
                        // `ServerGrid.doMatch3` adopts the chain color into every wildcard
                        // before the collected tiles are handled, so a wildcard counts as
                        // the chain value too.
                        let collected_values: Vec<u16> = chain
                            .iter()
                            .map(|&(x, y)| {
                                let value = run.board[y * width + x];
                                if value == TILE_WILDCARD {
                                    chain_value.unwrap_or(value)
                                } else {
                                    value
                                }
                            })
                            .collect();
                        for &(x, y) in &chain {
                            run.board[y * width + x] = TILE_EMPTY;
                        }
                        run.chains = run
                            .chains
                            .checked_add(1)
                            .ok_or_else(|| invalid("chain count overflow"))?;
                        if run.chains > 100_000 {
                            return Err(invalid("too many mining chains"));
                        }
                        let score = CHAIN_SCORE + (chain.len() as u32 - 3) * CHAIN_TILE_SCORE;
                        run.score = run
                            .score
                            .checked_add(score)
                            .filter(|s| *s <= 1_000_000_000)
                            .ok_or_else(|| invalid("mining score overflow"))?;
                        message(
                            &mut out,
                            id,
                            self.game_id,
                            &format!("OnLevelEventMsg\t21\t{score}\t0"),
                        )?;
                        let bonus = 2 * chain.len() as u32;
                        let applied = MAX_REMAINING_MS
                            .saturating_sub(run.remaining_ms)
                            .min(bonus * 1000)
                            / 1000;
                        run.remaining_ms += applied * 1000;
                        message(
                            &mut out,
                            id,
                            self.game_id,
                            &format!("OnLevelEventMsg\t25\t{applied}\t1"),
                        )?;
                        collect_chain_goals(
                            run,
                            &self.content.spec,
                            &mut out,
                            self.game_id,
                            &collected_values,
                        )?;
                        gravity(
                            run,
                            &self.content.spec,
                            self.bounds,
                            &mut out,
                            self.game_id,
                            &self.content.powerup_definitions,
                        )?;
                        settle_after_collect(
                            run,
                            &self.content.spec,
                            self.bounds,
                            &mut out,
                            self.game_id,
                            &self.content.powerup_definitions,
                        )?;
                    }
                }
                ["OnActionMsg", index, list] => {
                    // Recovered client->server powerup action (`Grid.as`
                    // 1473/1554/1607/1690, `GameLevel.OnActionMsg` 507-542).
                    // Only the four clickable originals are serviced; the
                    // collected `900..=949` powerups are deferred, so a forged
                    // action for one is unsupported exactly like loot-drop.
                    let index = number(index)?;
                    if run.phase == Type8Phase::Playing {
                        if run.paused {
                            return Err(invalid("mining is paused"));
                        }
                        let index = u16::try_from(index)
                            .map_err(|_| invalid("unsupported mining action"))?;
                        apply_powerup_action(
                            run,
                            &self.content.spec,
                            self.bounds,
                            &self.content.powerup_definitions,
                            index,
                            list,
                            &mut out,
                            self.game_id,
                        )?;
                    }
                }
                ["OnIdleMsg", list] => {
                    let points = idle_points(list)?;
                    if run.phase == Type8Phase::Playing {
                        for (x, y) in points {
                            if !in_board(x, y, self.bounds) {
                                return Err(invalid("idle outside mining area"));
                            }
                            let width = self.content.spec.width;
                            run.idle[y * width + x] = true;
                        }
                    }
                }
                ["OnVerifyMsg", list] => {
                    let values = number_list(list, 3)?;
                    if run.phase == Type8Phase::Playing {
                        let width = self.content.spec.width;
                        for triple in values.chunks_exact(3) {
                            let (x, y) = (triple[0] as usize, triple[1] as usize);
                            if x >= width || y >= self.content.spec.height {
                                return Err(invalid("mining verification outside map"));
                            }
                            // Zero means "do not check" in the original
                            // ServerGrid.CheckGrid.
                            if triple[2] != 0 && run.board[y * width + x] != triple[2] {
                                return Err(invalid("mining board verification mismatch"));
                            }
                        }
                    }
                }
                ["OnLevelEventMsg", "50", x, y] => {
                    let (x, y) = (number(x)? as usize, number(y)? as usize);
                    if run.phase == Type8Phase::Playing {
                        collect_fall(
                            run,
                            &self.content.spec,
                            self.bounds,
                            &mut out,
                            self.game_id,
                            x,
                            y,
                            &self.content.powerup_definitions,
                        )?;
                    }
                }
                ["OnLevelEventMsg", "51", x, y] => {
                    // Convert events are valid on transform-type boards; this
                    // reconstructed board has none.
                    let _ = (number(x)?, number(y)?);
                }
                ["OnLevelEventMsg", "60"] | ["OnLevelEventMsg", "61"] => {
                    // This bounded board only appends queue messages
                    // (reset = -1), so no untrusted ACK mutates the queue.
                }
                ["FRServer_Pause", v] | ["OnPauseMsg", v] => {
                    let paused = boolean(v)?;
                    if run.phase == Type8Phase::Playing && paused != run.paused {
                        run.paused = paused;
                        message(
                            &mut out,
                            id,
                            self.game_id,
                            if paused {
                                "OnPauseMsg\tfalse"
                            } else {
                                "OnPauseMsg\ttrue"
                            },
                        )?;
                    }
                }
                ["OnNextLevelMsg"] => {
                    if run.phase == Type8Phase::LevelComplete {
                        run.phase = Type8Phase::Won;
                        next.completed = next
                            .completed
                            .checked_add(1)
                            .ok_or_else(|| invalid("completion overflow"))?;
                        message(&mut out, id, self.game_id, "OnGameWonMsg")?;
                        out.completed = true;
                        out.transaction = reward_transaction(run, &self.content.spec)?;
                    }
                }
                ["FRServer_GameOver"] => {
                    if matches!(run.phase, Type8Phase::Won | Type8Phase::Lost) && !presented {
                        out.replies.extend(results(
                            run,
                            run.phase == Type8Phase::Won,
                            self.content.spec.duration_seconds,
                            self.game_id,
                        ));
                        run.end_sent = true;
                        presented = true;
                    }
                }
                ["FRServer_GameClose"] => {
                    next.active = None;
                    if !presented {
                        out.replies.push(header(19, id, self.game_id));
                    }
                    presented = false;
                }
                ["FRServer_GameWon", score]
                | ["FRServer_GameWon", score, _]
                | ["FRServer_GameLost", score]
                | ["FRServer_GameQuit", score]
                | ["FRServer_ScoreSet", score] => {
                    // Client reports never affect authoritative score, board,
                    // quota progress, output or quantities.
                    let _ = number(score)?;
                    if fields.len() == 3 {
                        let _ = number(fields[2])?;
                    }
                }
                _ => return Err(invalid("unsupported or malformed mining message")),
            }
        }
        out.changed = next != self.state;
        Self::validate_state(&next)?;
        if let Some(run) = &next.active {
            validate_run(run, &self.content.spec, self.bounds)?;
        }
        self.state = next;
        self.results_presented = presented;
        Ok(out)
    }

    fn info_packet(&self, id: i32) -> Vec<u8> {
        let c = &self.content;
        let mut b = header(16, id, self.game_id);
        for n in [c.name_id, c.icon_id, c.description_id, 1, c.profile_id, 8] {
            b.extend(n.to_le_bytes());
        }
        b.push(0);
        b.extend([0; 207]); // Three empty 69-byte RewardBundleBase records.
        b.extend(0_i32.to_le_bytes());
        b.extend([1; 5]);
        let s = self.content.preload_file.as_bytes();
        b.extend((s.len() as i32).to_le_bytes());
        b.extend(s);
        b.extend(1_i32.to_le_bytes());
        b.push(1);
        b.extend(self.game_id.to_le_bytes());
        b.extend([0; 4]);
        b.extend([0; 8]);
        b
    }
}

/// Reconstructed default Mining18 content: 8x8 playable rectangle inside the
/// movie's 14x14 `aMyGrid` with a dropzone row underneath. Original server
/// level tables, quotas and outputs were not recovered.
pub fn mining_spec() -> Type8BoardSpec {
    let mut shape = vec![0_i32; SIDE * SIDE];
    for y in 3..3 + 8 {
        for x in 3..3 + 8 {
            shape[y * SIDE + x] = 1;
        }
    }
    for x in 3..3 + 8 {
        shape[11 * SIDE + x] = 4;
    }
    Type8BoardSpec {
        width: SIDE,
        height: SIDE,
        shape,
        selectable: vec![401, 402, 403, 404, 405],
        collectable: vec![601, 602],
        tiles: vec![
            Type8Kind {
                value: 401,
                category: 400,
                icon_id: 1,
            },
            Type8Kind {
                value: 402,
                category: 400,
                icon_id: 2,
            },
            Type8Kind {
                value: 403,
                category: 400,
                icon_id: 3,
            },
            Type8Kind {
                value: 404,
                category: 400,
                icon_id: 4,
            },
            Type8Kind {
                value: 405,
                category: 400,
                icon_id: 5,
            },
            Type8Kind {
                value: 601,
                category: 600,
                icon_id: 909,
            },
            Type8Kind {
                value: 602,
                category: 600,
                icon_id: 909,
            },
        ],
        goals: vec![
            Type8Goal {
                index: 0,
                value: 601,
                quota: 6,
                per_reward: 1,
                icon_id: 909,
            },
            Type8Goal {
                index: 1,
                value: 602,
                quota: 6,
                per_reward: 2,
                icon_id: 909,
            },
        ],
        // (R) The original Mining drop table is unknown. Reuse the catalogued
        // ore definition until Mining's intended output mapping is recovered.
        outputs: vec![
            Type8Output {
                value: 601,
                definition_id: 2739,
                cluster_quantity: 1,
            },
            Type8Output {
                value: 602,
                definition_id: 2739,
                cluster_quantity: 2,
            },
        ],
        duration_seconds: 180,
        seed: MINING_GAME_ID as u32,
        wildcard_percent: 5,
        collectable_percent: 40,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice: "Reconstructed Mining18 rules: board layout, 40% collectable rate, 6+6 quotas, seed, item mapping and Miner XP are local tuning. The envelope, message names, shape codes, chain rules, queue discipline and cluster math are recovered from game_mining.gfx.".into(),
    }
}

/// Write the reconstructed Mining18 content file (`mining.json`).
///
/// Only game 18 is bound: every other TYPE 8 variant still lacks recovered level
/// tables, so this refuses them instead of shipping the mining board under
/// another game id. The written file carries the recovered asset names and the
/// explicit `reconstructed_rules` notice.
pub fn write_content(directory: &Path, game_id: i32) -> io::Result<PathBuf> {
    if game_id != MINING_GAME_ID {
        return Err(invalid("mining content does not support this game id"));
    }
    let content = serde_json::json!({
        "schema_version": 1,
        "game_id": MINING_GAME_ID,
        "profile_id": MINER_PROFILE_ID,
        "name_id": MINING_NAME_ID,
        "description_id": MINING_DESCRIPTION_ID,
        "icon_id": MINING_ICON_ID,
        "preload_file": MINING_PRELOAD_FILE,
        "launch_movie": MINING_LAUNCH_MOVIE,
        "spec": mining_spec(),
    });
    let path = directory.join(CONTENT_FILE);
    fs::write(&path, serde_json::to_vec(&content).map_err(invalid)?)?;
    Ok(path)
}

/// Reconstructed clickable-powerup authoring guard. Refuses every configuration
/// the engine cannot service yet so no content can spawn an inert powerup:
/// `900..=949` collected powerups are explicitly deferred, and the four
/// clickable values need an authored `power` inside the operational bound.
fn validate_powerup_definitions(content: &Content) -> io::Result<()> {
    let spec = &content.spec;
    if spec.powerup_percent > 100 {
        return Err(invalid("invalid mining spawn percentage"));
    }
    if content.powerup_definitions.is_empty() {
        return if spec.powerup_percent == 0 {
            Ok(())
        } else {
            Err(invalid("mining powerup spawn chance needs definitions"))
        };
    }
    if content.powerup_definitions.len() > MAX_POWERUP_DEFINITIONS {
        return Err(invalid("too many mining powerup definitions"));
    }
    for definition in &content.powerup_definitions {
        if (TILE_POWERUP_START..TILE_POWERUP_PRESSME).contains(&definition.value) {
            return Err(invalid(
                "collected mining powerups 900..=949 are not implemented",
            ));
        }
        if !(TILE_POWERUP_PRESSME..=TILE_POWERUP_SHUFFLE).contains(&definition.value) {
            return Err(invalid("invalid mining powerup value"));
        }
        if definition.power == 0 || definition.power > MAX_POWERUP_POWER {
            return Err(invalid("invalid mining powerup power"));
        }
        if definition.spawn_chance > 100 || definition.duration > MAX_REMAINING_MS / 1000 {
            return Err(invalid("invalid mining powerup tuning"));
        }
    }
    let mut values: Vec<u16> = content
        .powerup_definitions
        .iter()
        .map(|definition| definition.value)
        .collect();
    values.sort_unstable();
    values.dedup();
    if values.len() != content.powerup_definitions.len() {
        return Err(invalid("duplicate mining powerup value"));
    }
    if spec.powerup_percent > 0
        && !content
            .powerup_definitions
            .iter()
            .any(|definition| definition.spawn_chance > 0)
    {
        return Err(invalid(
            "mining powerup spawn chance needs a weighted value",
        ));
    }
    Ok(())
}

fn validate_spec(spec: &Type8BoardSpec) -> io::Result<Bounds> {
    if spec.width < 6
        || spec.height < 6
        || spec.width > 32
        || spec.height > 32
        || spec.width * spec.height > MAX_BOARD_CELLS
        || spec.shape.len() != spec.width * spec.height
    {
        return Err(invalid("invalid mining board dimensions"));
    }
    if spec.selectable.is_empty()
        || spec.selectable.len() > 16
        || spec
            .selectable
            .iter()
            .any(|v| !(TILE_SELECTABLE_START..TILE_SELECTABLE_START + 100).contains(v))
        || spec.collectable.len() > 16
        || spec
            .collectable
            .iter()
            .any(|v| !(TILE_COLLECTABLE_START..TILE_COLLECTABLE_START + 100).contains(v))
    {
        return Err(invalid("invalid mining tile kinds"));
    }
    // A goal may sit on either spawn family: Mining18's baskets are 600-block
    // collectables (`HandleCollectableTile`), while the farming boards' produce
    // baskets are 400-block selectables (`HandleSelectableTile`). The recovered
    // client names them with `LookupString(sGAME + "Value" + value)`, so the
    // emitting content owns choosing a value that resolves.
    if spec.goals.len() > MAX_GOALS
        || spec.goals.iter().any(|g| {
            !(spec.selectable.contains(&g.value) || spec.collectable.contains(&g.value))
                || g.per_reward == 0
                || g.quota % g.per_reward != 0
        })
    {
        return Err(invalid("invalid mining goals"));
    }
    if spec.outputs.len() > 16
        || spec.outputs.iter().any(|o| {
            // Every proposed output must be tied to a goal of the same tile
            // value; an orphaned output would silently lose its grant.
            !(spec.selectable.contains(&o.value) || spec.collectable.contains(&o.value))
                || !spec.goals.iter().any(|g| g.value == o.value)
                || o.cluster_quantity == 0
                || o.definition_id == 0
        })
    {
        return Err(invalid("invalid mining outputs"));
    }
    if spec.duration_seconds == 0
        || spec.duration_seconds > MAX_REMAINING_MS / 1000
        || spec.seed == 0
    {
        return Err(invalid("invalid mining level timing"));
    }
    for percent in [
        spec.wildcard_percent,
        spec.collectable_percent,
        spec.blocker_percent,
        spec.powerup_percent,
    ] {
        if percent > 100 {
            return Err(invalid("invalid mining spawn percentage"));
        }
    }
    for code in &spec.shape {
        let code = *code;
        // Recovered `ServerGrid.buildMap` grammar, restricted to the shapes
        // this engine can actually execute: `0..=4` markers, a positive
        // category code with an available pool (`floor(v/100)*100` is 400 or
        // 600), or a negated specific value the board validator accepts.
        let valid = match code {
            0..=4 => true,
            100..=699 => matches!(code / 100 * 100, 400 | 600),
            -1002..=-100 => valid_value((-code) as u16),
            _ => false,
        };
        if !valid {
            return Err(invalid("invalid mining shape code"));
        }
    }
    let mut min_x = usize::MAX;
    let mut min_y = usize::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut dropzones = Vec::new();
    for y in 0..spec.height {
        for x in 0..spec.width {
            let code = spec.shape[y * spec.width + x];
            match code {
                0 => {}
                1..=3 => {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
                4 => dropzones.push((x, y)),
                code if code < 0 => {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
                _code => {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }
    }
    if min_x == usize::MAX {
        return Err(invalid("mining board has no playable tiles"));
    }
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    if width < 6 || height < 6 {
        return Err(invalid("mining playable area too small"));
    }
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if spec.shape[y * spec.width + x] == 0 {
                return Err(invalid("mining playable area is not rectangular"));
            }
        }
    }
    let dropzone = max_y + 1;
    if dropzone >= spec.height {
        return Err(invalid("mining board has no dropzone row"));
    }
    for x in min_x..=max_x {
        if spec.shape[dropzone * spec.width + x] != 4 {
            return Err(invalid("mining dropzone row is incomplete"));
        }
    }
    if dropzones
        .iter()
        .any(|(x, y)| *y != dropzone || *x < min_x || *x > max_x)
    {
        return Err(invalid("mining dropzone outside the playable area"));
    }
    Ok(Bounds {
        left: min_x,
        top: min_y,
        width,
        height,
    })
}

fn validate_run(run: &Type8Run, spec: &Type8BoardSpec, bounds: Bounds) -> io::Result<()> {
    let beyond_start = matches!(
        run.phase,
        Type8Phase::Playing | Type8Phase::LevelComplete | Type8Phase::Won | Type8Phase::Lost
    );
    if beyond_start && run.board.is_empty() {
        return Err(invalid("mining run without a board"));
    }
    if !run.board.is_empty() {
        if run.board.len() != spec.width * spec.height || run.idle.len() != run.board.len() {
            return Err(invalid("mining board size mismatch"));
        }
        let _ = bounds;
    }
    for v in &run.board {
        if !valid_value(*v) {
            return Err(invalid("mining board value out of range"));
        }
    }
    if run.collected.len() != spec.goals.len() {
        return Err(invalid("mining collected vector mismatch"));
    }
    if run.collected.iter().any(|c| *c > 1_000_000) {
        return Err(invalid("mining collected overflow"));
    }
    Ok(())
}

fn valid_value(v: u16) -> bool {
    v == TILE_EMPTY
        || v == TILE_DROPZONE
        || v == TILE_WILDCARD
        || (TILE_SELECTABLE_START..TILE_SELECTABLE_START + 100).contains(&v)
        || (TILE_COLLECTABLE_START..TILE_COLLECTABLE_START + 100).contains(&v)
        || v == TILE_BLOCKER
        || (900..=953).contains(&v)
}

fn initialize_board(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    out: &mut Type8Outcome,
    game_id: i32,
    reshuffle: bool,
    powerup_definitions: &[Type8PowerupDefinition],
) -> io::Result<()> {
    if reshuffle {
        run.reshuffles += 1;
        if run.reshuffles > 1024 {
            return Err(invalid("too many mining reshuffles"));
        }
        message(out, run.instance_id, game_id, "OnLevelEventMsg\t11\t1")?;
    }
    run.board = vec![TILE_EMPTY; spec.width * spec.height];
    run.idle = vec![true; spec.width * spec.height];
    run.tile_queue.clear();
    for y in 0..spec.height {
        for x in 0..spec.width {
            let code = spec.shape[y * spec.width + x];
            let value = match code {
                0 => TILE_EMPTY,
                // `1` is TILE_BUILDREGULAR: the original regular randomizer
                // excludes collectables (ServerTileGenerator.as 107-110, 126-137).
                1 => random_tile(run, spec, false, powerup_definitions)?,
                2 => random_tile(run, spec, true, powerup_definitions)?,
                3 => TILE_BLOCKER,
                4 => TILE_DROPZONE,
                code if code < 0 => (-code) as u16,
                code => category_tile(run, spec, (code / 100 * 100) as u16)?,
            };
            run.board[y * spec.width + x] = value;
        }
    }
    while run.tile_queue.len() < QUEUE_TARGET {
        let value = random_tile(run, spec, true, powerup_definitions)?;
        run.tile_queue.push(value);
    }
    if !reshuffle {
        for kind in &spec.tiles {
            // The recovered server sends `category + type` as the value and the
            // per-kind `type` as the second field (`core/ServerTileGenerator.as`
            // `SendTileInfo(400 + GetType(), GetType())`). The client resolves it
            // as `"SelectableTile" + type` / `aCollectableNames[type * 2]`, so
            // sending the category instead left every regular tile on the
            // combined clip's default frame (native run 121834, frames
            // 003562-003568). Refuse content whose value sits below its category
            // rather than wrapping.
            let index = kind
                .value
                .checked_sub(kind.category)
                .ok_or_else(|| invalid("mining tile value below its category"))?;
            message(
                out,
                run.instance_id,
                game_id,
                &format!("OnTileInfoMsg\t{}\t{}", kind.value, index),
            )?;
        }
        // Recovered original: one `OnPowerupInitMsg value, duration, power` per
        // authored spawn row, emitted at level start (GameLevel.as 120-128,
        // ServerNetwork 60-63). Ascending value keeps the order deterministic.
        let mut definitions = powerup_definitions.to_vec();
        definitions.sort_by_key(|definition| definition.value);
        for definition in &definitions {
            message(
                out,
                run.instance_id,
                game_id,
                &format!(
                    "OnPowerupInitMsg\t{}\t{}\t{}",
                    definition.value, definition.duration, definition.power
                ),
            )?;
        }
        run.random_queue.clear();
        while run.random_queue.len() < QUEUE_TARGET {
            let value = (rng(run) % (RANDOM_MAX_VALUE + 1)) as u16;
            run.random_queue.push(value);
        }
    }
    message(
        out,
        run.instance_id,
        game_id,
        &format!("OnTileQueueMsg\t-1\t{}", csv(&run.board)),
    )?;
    message(
        out,
        run.instance_id,
        game_id,
        &format!(
            "OnBuildMapMsg\t{}\t{}\t{}",
            spec.width,
            spec.height,
            csv_i32(&spec.shape)
        ),
    )?;
    message(
        out,
        run.instance_id,
        game_id,
        &format!("OnTileQueueMsg\t-1\t{}", csv(&run.tile_queue)),
    )?;
    if !reshuffle {
        message(
            out,
            run.instance_id,
            game_id,
            &format!("OnRandomQueueMsg\t{}", csv(&run.random_queue)),
        )?;
    }
    Ok(())
}

fn category_tile(run: &mut Type8Run, spec: &Type8BoardSpec, category: u16) -> io::Result<u16> {
    let pool: Vec<u16> = match category {
        400 => spec.selectable.clone(),
        600 => spec.collectable.clone(),
        _ => return Err(invalid("unsupported mining category")),
    };
    if pool.is_empty() {
        return Err(invalid("empty mining tile pool"));
    }
    let pick = rng(run) as usize % pool.len();
    Ok(pool[pick])
}

/// Weighted clickable-powerup pick over the authored private definitions.
/// Returns `None` when no definition carries weight, which keeps a
/// `powerup_percent = 0` content byte- and draw-identical to the old engine.
fn weighted_powerup(
    run: &mut Type8Run,
    definitions: &[Type8PowerupDefinition],
) -> Option<Type8PowerupDefinition> {
    let total: u32 = definitions
        .iter()
        .map(|definition| definition.spawn_chance)
        .sum();
    if total == 0 {
        return None;
    }
    let mut roll = rng(run) % total;
    for definition in definitions {
        if roll < definition.spawn_chance {
            return Some(definition.clone());
        }
        roll -= definition.spawn_chance;
    }
    // Unreachable: `roll < total` always lands in a weighted definition.
    definitions.last().cloned()
}

fn random_tile(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    allow_collectable: bool,
    powerup_definitions: &[Type8PowerupDefinition],
) -> io::Result<u16> {
    // `powerup_percent = 0` (the shipped default and every existing content
    // file) takes the original branch below and draws exactly as before.
    if spec.powerup_percent > 0 {
        if let Some(definition) = weighted_powerup(run, powerup_definitions) {
            if rng(run) % 100 < spec.powerup_percent {
                return Ok(definition.value);
            }
        }
    }
    let roll = rng(run) % 100;
    if spec.wildcard_percent > 0 && roll < spec.wildcard_percent {
        return Ok(TILE_WILDCARD);
    }
    if allow_collectable
        && spec.collectable_percent > 0
        && roll < spec.wildcard_percent + spec.collectable_percent
    {
        return category_tile(run, spec, 600);
    }
    category_tile(run, spec, 400)
}

fn gravity(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    out: &mut Type8Outcome,
    game_id: i32,
    powerup_definitions: &[Type8PowerupDefinition],
) -> io::Result<()> {
    // AS2 scans columns right-to-left and holes bottom-to-top; the earliest
    // queued replacement ends lowest among the new top rows of that column.
    for x in (bounds.left..bounds.left + bounds.width).rev() {
        let mut target = bounds.top + bounds.height;
        for y in (bounds.top..bounds.top + bounds.height).rev() {
            let i = y * spec.width + x;
            let value = run.board[i];
            if value == TILE_EMPTY || value == TILE_BLOCKER {
                continue;
            }
            target -= 1;
            if target != y {
                run.board[target * spec.width + x] = value;
                run.board[i] = TILE_EMPTY;
                run.idle[target * spec.width + x] = false;
            }
        }
        while target > bounds.top {
            target -= 1;
            let value = *run
                .tile_queue
                .first()
                .ok_or_else(|| invalid("mining tile queue empty"))?;
            run.tile_queue.remove(0);
            run.board[target * spec.width + x] = value;
            run.idle[target * spec.width + x] = false;
            if run.tile_queue.len() < QUEUE_MIN {
                let mut add = Vec::new();
                for _ in 0..QUEUE_BATCH {
                    add.push(random_tile(run, spec, true, powerup_definitions)?);
                }
                run.tile_queue.extend(&add);
                message(
                    out,
                    run.instance_id,
                    game_id,
                    &format!("OnTileQueueMsg\t-1\t{}", csv(&add)),
                )?;
            }
        }
    }
    Ok(())
}

fn collect_fall(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    out: &mut Type8Outcome,
    game_id: i32,
    x: usize,
    y: usize,
    powerup_definitions: &[Type8PowerupDefinition],
) -> io::Result<()> {
    let bottom = bounds.top + bounds.height - 1;
    if !in_board(x, y, bounds) || y != bottom {
        return Err(invalid("mining fall outside the bottom row"));
    }
    let value = run.board[y * spec.width + x];
    if (TILE_POWERUP_START..TILE_POWERUP_PRESSME).contains(&value) {
        // Recovered original: the three collected powerups are fall types
        // (`ServerGrid.isInFallType` defaults to 600..949), reach
        // `FallOffBottom`/`HandlePowerupTile` and are removed. Their timers and
        // score multipliers stay deferred, so no effect is applied yet.
        return Ok(());
    }
    if (TILE_POWERUP_PRESSME..).contains(&value) && value <= TILE_POWERUP_SHUFFLE {
        // Clickable powerups sit above the fall default; a native client should
        // never report one, so treat it as untrusted rather than inventing an
        // effect for it.
        return Err(invalid("mining clickable powerup cannot fall"));
    }
    let goal_index = spec
        .goals
        .iter()
        .position(|g| g.value == value)
        .ok_or_else(|| invalid("mining fall is not a goal collectable"))?;
    run.board[y * spec.width + x] = TILE_EMPTY;
    run.collected[goal_index] = run.collected[goal_index]
        .checked_add(1)
        .ok_or_else(|| invalid("mining collected overflow"))?;
    let goal = spec.goals[goal_index].clone();
    run.score = run
        .score
        .checked_add(COLLECTABLE_SCORE)
        .filter(|s| *s <= 1_000_000_000)
        .ok_or_else(|| invalid("mining score overflow"))?;
    message(
        out,
        run.instance_id,
        game_id,
        &format!(
            "OnQuotaUpdateMsg\t{goal_index}\t{}\t{}\t{}",
            run.collected[goal_index], goal.quota, goal.per_reward
        ),
    )?;
    message(
        out,
        run.instance_id,
        game_id,
        &format!("OnLevelEventMsg\t21\t{COLLECTABLE_SCORE}\t3"),
    )?;
    gravity(run, spec, bounds, out, game_id, powerup_definitions)?;
    settle_after_collect(run, spec, bounds, out, game_id, powerup_definitions)
}

/// ORIGINAL `GameLevel.HandleCollected` through `HandleSelectableTile`: every
/// tile of an accepted chain feeds the bin whose quota value matches, so a
/// selectable-valued goal (the farming produce baskets) advances on the chain
/// itself. Mining18's 600-block goals never match a 400-block chain value, so
/// those stay fall-only. The original selectable handler adds no per-tile
/// score — only the chain score above.
fn collect_chain_goals(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    out: &mut Type8Outcome,
    game_id: i32,
    collected_values: &[u16],
) -> io::Result<()> {
    for value in collected_values {
        let Some((goal_index, goal)) = spec
            .goals
            .iter()
            .enumerate()
            .find(|(_, goal)| goal.value == *value)
        else {
            continue;
        };
        run.collected[goal_index] = run.collected[goal_index]
            .checked_add(1)
            .ok_or_else(|| invalid("mining collected overflow"))?;
        message(
            out,
            run.instance_id,
            game_id,
            &format!(
                "OnQuotaUpdateMsg\t{goal_index}\t{}\t{}\t{}",
                run.collected[goal_index], goal.quota, goal.per_reward
            ),
        )?;
    }
    Ok(())
}

/// Shared tail of the fall and chain paths: gravity has run, so either stop the
/// level once every quota goal is complete or reshuffle a board without moves.
fn settle_after_collect(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    out: &mut Type8Outcome,
    game_id: i32,
    powerup_definitions: &[Type8PowerupDefinition],
) -> io::Result<()> {
    let complete = spec
        .goals
        .iter()
        .zip(&run.collected)
        .all(|(goal, collected)| goal.quota == 0 || *collected >= goal.quota);
    let any_goal = spec.goals.iter().any(|g| g.quota > 0);
    if any_goal && complete {
        run.phase = Type8Phase::LevelComplete;
        if run.paused {
            message(out, run.instance_id, game_id, "OnPauseMsg\ttrue")?;
            run.paused = false;
        }
        message(out, run.instance_id, game_id, "OnStopLevel\t1\ttrue")?;
    } else if !has_move(run, spec, bounds) {
        initialize_board(run, spec, out, game_id, true, powerup_definitions)?;
    }
    Ok(())
}

/// Fail-closed reward proposal. An output without a matching goal is an error
/// (never a silent drop), and a zero bundle count for one goal no longer
/// discards the grants of the other goals.
fn reward_transaction(
    run: &Type8Run,
    spec: &Type8BoardSpec,
) -> io::Result<Option<Type8Transaction>> {
    if spec.outputs.is_empty() {
        return Ok(None);
    }
    let mut grants = Vec::new();
    for output in &spec.outputs {
        let Some((goal_index, goal)) = spec
            .goals
            .iter()
            .enumerate()
            .find(|(_, goal)| goal.value == output.value)
        else {
            // `validate_spec` refuses this at load; keep the runtime path
            // closed rather than dropping every grant.
            return Err(invalid("mining output has no matching goal"));
        };
        let clusters = run.collected[goal_index] / goal.per_reward;
        let quantity = clusters
            .checked_mul(output.cluster_quantity)
            .ok_or_else(|| invalid("mining grant overflow"))?;
        if quantity > 0 {
            grants.push((output.definition_id, quantity));
        }
    }
    if grants.is_empty() {
        return Ok(None);
    }
    Ok(Some(Type8Transaction {
        consumes: Vec::new(),
        grants,
    }))
}

fn validate_chain(
    run: &Type8Run,
    points: &[(usize, usize)],
    spec: &Type8BoardSpec,
    bounds: Bounds,
) -> io::Result<Option<u16>> {
    if points.len() < 3 {
        return Err(invalid("mining chain needs at least 3 tiles"));
    }
    let mut seen = HashSet::new();
    let mut value: Option<u16> = None;
    let mut previous: Option<(usize, usize)> = None;
    for &(x, y) in points {
        if !in_board(x, y, bounds) || !seen.insert((x, y)) {
            return Err(invalid("mining chain outside board or repeated tile"));
        }
        if let Some((px, py)) = previous {
            if x.abs_diff(px) + y.abs_diff(py) != 1 {
                return Err(invalid("mining chain is not orthogonal"));
            }
        }
        let i = y * spec.width + x;
        if !run.idle[i] {
            return Err(invalid("mining chain tile is not idle"));
        }
        let v = run.board[i];
        if v == TILE_WILDCARD {
            previous = Some((x, y));
            continue;
        }
        if !(TILE_SELECTABLE_START..TILE_SELECTABLE_START + 100).contains(&v) {
            return Err(invalid("mining chain tile is not selectable"));
        }
        if value.is_some_and(|old| old != v) {
            return Err(invalid("mining chain colors differ"));
        }
        value = Some(v);
        previous = Some((x, y));
    }
    // The chain's matching value (wildcards adopt it in `doMatch3`); `None` for a
    // chain made only of wildcards.
    Ok(value)
}

/// Applies one clickable-powerup action. The four effects mirror the originals
/// Applies one clickable-powerup action with the recovered selection ownership:
/// the server draws targets from `aRandomNumberQueue` and requires the client's
/// list to name exactly those tiles in order. `StartWildTiles` (ServerGrid
/// 988-1046), `StartDestruction` (1102-1165), `StartShuffle` (1166-1249) and
/// `StartShatter` (889-955) are the references; the press arm in
/// `Grid.PowerupPressed` (390-419) sends the pressed tile first.
fn apply_powerup_action(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    definitions: &[Type8PowerupDefinition],
    index: u16,
    list: &str,
    out: &mut Type8Outcome,
    game_id: i32,
) -> io::Result<()> {
    let Some(definition) = definitions
        .iter()
        .find(|definition| definition.value == index)
    else {
        return Err(invalid("unsupported mining action"));
    };
    let points = powerup_points(list)?;
    let power = definition.power as usize;
    if points.len() > power {
        return Err(invalid("mining action exceeds its power"));
    }
    let pressed = points[0];
    if !in_board(pressed.0, pressed.1, bounds)
        || run.board[board_index(spec, pressed)] != index
        || !run.idle[board_index(spec, pressed)]
    {
        return Err(invalid("mining action does not match the pressed tile"));
    }
    match index {
        TILE_POWERUP_DYNAMITE => shatter(run, spec, bounds, &points, power, out, game_id)?,
        TILE_POWERUP_WILDTILE => wild_tiles(run, spec, bounds, &points, power, out, game_id)?,
        TILE_POWERUP_DESTRUCT => destruction(run, spec, bounds, &points, power, out, game_id)?,
        TILE_POWERUP_SHUFFLE => shuffle_tiles(run, spec, bounds, &points, power, out, game_id)?,
        _ => return Err(invalid("unsupported mining action")),
    }
    gravity(run, spec, bounds, out, game_id, definitions)?;
    if !has_move(run, spec, bounds) {
        initialize_board(run, spec, out, game_id, true, definitions)?;
    }
    Ok(())
}

/// ORIGINAL `ServerGrid.StartWildTiles` (988-1046): `nPower` draws from the idle
/// selectable pool, each verified against the client's list, then turned into a
/// `200` wildcard in place.
fn wild_tiles(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    points: &[(usize, usize)],
    power: usize,
    out: &mut Type8Outcome,
    game_id: i32,
) -> io::Result<()> {
    let mut tiles = init_random_tiles(run, spec, bounds, is_selectable);
    // The pressed tile turns into a wildcard first (`doWildTiles`, 1035-1046)
    // and is popped out of the array, so it is never redrawn.
    run.board[board_index(spec, points[0])] = TILE_WILDCARD;
    run.idle[board_index(spec, points[0])] = false;
    tiles.retain(|tile| *tile != points[0]);
    // The original loops the full `_nPower`, stopping early only when the array is
    // exhausted (1011-1031). The client list must name every implied tile, and a
    // pool shorter than `power` shortens the implied count.
    let expected = power.min(tiles.len() + 1);
    if points.len() != expected {
        return Err(invalid("mining action tile count does not match the spawn"));
    }
    for &(x, y) in points.iter().skip(1) {
        let Some(picked) = get_random_tile(run, out, game_id, &mut tiles)? else {
            break;
        };
        if picked != (x, y) || !run.idle[board_index(spec, (x, y))] {
            return Err(invalid("mining action tile mismatch"));
        }
        run.board[board_index(spec, (x, y))] = TILE_WILDCARD;
        run.idle[board_index(spec, (x, y))] = false;
    }
    Ok(())
}

/// ORIGINAL `ServerGrid.StartDestruction` (1102-1165): the same draw-and-verify
/// contract over idle selectables plus blockers, each removed in place.
fn destruction(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    points: &[(usize, usize)],
    power: usize,
    out: &mut Type8Outcome,
    game_id: i32,
) -> io::Result<()> {
    let mut tiles = init_random_tiles(run, spec, bounds, |value| {
        is_selectable(value) || value == TILE_BLOCKER
    });
    // `StartDestruction` 1102-1165 draws the pressed tile first and then loops
    // `_nPower` over the remaining idle selectables/blockers, stopping early only
    // when the pool is exhausted; the implied count must therefore be supplied.
    if power == 0 {
        return Err(invalid("mining action exceeds its power"));
    }
    run.board[board_index(spec, points[0])] = TILE_EMPTY;
    tiles.retain(|tile| *tile != points[0]);
    let expected = power.min(tiles.len() + 1);
    if points.len() != expected {
        return Err(invalid("mining action tile count does not match the spawn"));
    }
    for &(x, y) in points.iter().skip(1) {
        let Some(picked) = get_random_tile(run, out, game_id, &mut tiles)? else {
            break;
        };
        if picked != (x, y) || !run.idle[board_index(spec, (x, y))] {
            return Err(invalid("mining action tile mismatch"));
        }
        run.board[board_index(spec, (x, y))] = TILE_EMPTY;
    }
    Ok(())
}

/// ORIGINAL `ServerGrid.StartShuffle` (1166-1249): `doWildTiles` turns the
/// pressed tile into a wildcard and pops it out of the client list, `power - 1`
/// draws rotate the remaining tiles' values, and the pressed wildcard stays put.
fn shuffle_tiles(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    points: &[(usize, usize)],
    power: usize,
    out: &mut Type8Outcome,
    game_id: i32,
) -> io::Result<()> {
    let mut candidates = init_random_tiles(run, spec, bounds, |value| {
        value >= TILE_WILDCARD && value != TILE_BLOCKER && value != TILE_DROPZONE
    });
    // Recovered `StartShuffle` 1166-1249: `doWildTiles(PRESSED)` first, then the
    // pressed coordinate is shifted out of the client list and `_nPower` drops by
    // one, so the rotation never includes it and a powerup can never be recycled.
    // `InitRandomTileArray(IDLE)` also excludes the freshly appeared wildcard.
    candidates.retain(|tile| *tile != points[0]);
    run.board[board_index(spec, points[0])] = TILE_WILDCARD;
    run.idle[board_index(spec, points[0])] = false;
    if power == 0 {
        return Err(invalid("mining action exceeds its power"));
    }
    let cap = (power - 1).min(candidates.len());
    // The original shifts the pressed coordinate out of the list and loops the
    // reduced power over the drawn pool (1166-1217), so a forged short list
    // cannot pass and a short pool cannot demand missing coordinates.
    if points.len() != cap + 1 {
        return Err(invalid("mining action tile count does not match the spawn"));
    }
    let mut placements: Vec<(usize, usize)> = Vec::new();
    for &(x, y) in points.iter().skip(1) {
        let Some(picked) = get_random_tile(run, out, game_id, &mut candidates)? else {
            break;
        };
        if picked != (x, y) || !run.idle[board_index(spec, (x, y))] {
            return Err(invalid("mining action tile mismatch"));
        }
        placements.push((x, y));
    }
    let n = placements.len();
    // Snapshot the drawn values before any write; the pressed tile stays a
    // wildcard and is never part of the rotation.
    let drawn_values: Vec<u16> = placements
        .iter()
        .map(|&position| run.board[board_index(spec, position)])
        .collect();
    let destinations = shuffle_placements(&drawn_values, &placements);
    for index in 0..n {
        // The mover of index `i` is the tile drawn at `(i + n - 1) % n`, which
        // the recovered list parks for slot `i`.
        let mover = drawn_values[(index + n - 1) % n];
        run.board[board_index(spec, destinations[index])] = mover;
    }
    Ok(())
}

/// ORIGINAL `ServerGrid.StartShatter` (889-955) plus `ShatterAddNeighbors`
/// (956-987). The pressed tile is entry 0 and is never drawn; every later entry
/// is a draw from the two-level BFS pool (`aShatterArray1`, then a refill from
/// `aShatterArray2`), where each confirmed tile contributes its four idle
/// selectable/wildcard/blocker neighbours in the original down/up/right/left
/// order (`ShatterAddNeighbors` 956-966: `_loc5_`, `_loc4_`, `_loc3_`, `_loc6_`).
fn shatter(
    run: &mut Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    points: &[(usize, usize)],
    power: usize,
    out: &mut Type8Outcome,
    game_id: i32,
) -> io::Result<()> {
    let mut array1: Vec<(usize, usize)> = Vec::new();
    let mut array2: Vec<(usize, usize)> = Vec::new();
    let mut current = points[0];
    // The original loops `_loc4_` (the authored power) and stops early only when
    // both shatter arrays are exhausted (889-948), so a forged short list cannot
    // skip the implied tiles.
    let expected = power.max(1);
    let mut exploded = 0;
    while exploded < expected {
        if exploded != 0 {
            if array1.is_empty() {
                array1 = array2.clone();
                array2.clear();
            }
            let Some(picked) = get_random_tile(run, out, game_id, &mut array1)? else {
                break;
            };
            let Some(&expected_point) = points.get(exploded) else {
                return Err(invalid("mining action list is shorter than the spawn"));
            };
            if picked != expected_point {
                return Err(invalid("mining action tile mismatch"));
            }
            current = picked;
        }
        if !run.idle[board_index(spec, current)] {
            return Err(invalid("mining action tile not idle"));
        }
        run.board[board_index(spec, current)] = TILE_EMPTY;
        // Recovered `ShatterAddNeighbors` order: down, up, right, left.
        for offset in [(0, 1isize), (0, -1), (1, 0), (-1, 0)] {
            let Some(next) = neighbour(current.0, current.1, offset, bounds) else {
                continue;
            };
            let value = run.board[board_index(spec, next)];
            let eligible = is_selectable(value) || value == TILE_WILDCARD || value == TILE_BLOCKER;
            if run.idle[board_index(spec, next)] && eligible && !array2.contains(&next) {
                array2.push(next);
            }
        }
        exploded += 1;
    }
    Ok(())
}

fn is_selectable(value: u16) -> bool {
    (TILE_SELECTABLE_START..TILE_SELECTABLE_START + 100).contains(&value)
}

fn board_index(spec: &Type8BoardSpec, position: (usize, usize)) -> usize {
    position.1 * spec.width + position.0
}

/// Direct port of the `StartShuffle` placement walk (`ServerGrid.as` 1213-1245).
///
/// `drawn_values[i]` is the value carried by the tile drawn at index `i`, and
/// `positions[i]` is that tile's coordinate (`_loc6_[i]`, `_loc4_.getPosition()`
/// at 1213). The tile drawn at index `i` is parked for slot `(i + 1) % n`
/// (`_loc13_[(_loc2_ + 1) % _loc7_] = _loc4_`, 1214), so the second loop moves
/// the tile drawn at `(i + n - 1) % n` onto the slot it settles at.
///
/// The inner walk (1228-1242) reads positions from the *mutable* destination
/// list while reading values from the *static* grid, and when the destination
/// already carries the mover's value it advances and folds the skipped
/// destination back onto the current one (`_loc6_[_loc2_ + _loc5_] = _loc6_[_loc2_]`,
/// 1240). Running past the rotated count reverts to the current slot (1234-1237).
///
/// The returned vector is the destination for the mover of index `i`.
#[allow(dead_code)]
pub fn shuffle_placements(
    drawn_values: &[u16],
    positions: &[(usize, usize)],
) -> Vec<(usize, usize)> {
    let n = drawn_values.len();
    let mut destinations = vec![positions[0]; n];
    if n == 0 {
        return destinations;
    }
    // `aCurrentGrid` is only refreshed after the loop (`updateCurrentGridArray`,
    // 1247), so slot values always come from the drawn snapshot.
    let grid_value = |at: (usize, usize)| -> Option<u16> {
        positions
            .iter()
            .position(|position| *position == at)
            .map(|index| drawn_values[index])
    };
    let mut slots = positions.to_vec();
    for index in 0..n {
        let value = drawn_values[(index + n - 1) % n];
        let mut destination = slots[index];
        let mut look = 0usize;
        let mut destination_value = grid_value(destination).unwrap_or(value);
        while destination_value == value && index + look < n {
            look += 1;
            if index + look >= n {
                break;
            }
            destination = slots[index + look];
            destination_value = grid_value(destination).unwrap_or(value);
        }
        if index + look >= n {
            destination = slots[index];
        } else if look != 0 {
            slots[index + look] = slots[index];
        }
        destinations[index] = destination;
    }
    destinations
}

/// The recovered client and server share one random queue, so the client's
/// action list must name exactly the tiles the server accepts, in order and
/// A future move exists when the board holds a three-tile path `validate_chain` would accept:
/// three distinct, orthogonally connected cells that are each selectable (`400..=499`) or a
/// wildcard (`200`), with every non-wildcard value equal and wildcards bridging. Any accepted
/// chain of three or more tiles contains such a path, so enumerating each cell as the centre with
/// two distinct orthogonal neighbours (six pairs) is exhaustive and needs no flood fill.
///
/// Idle flags are deliberately ignored: this is the post-chain "can the player still move" check
/// that drives the reshuffle, not a statement about tiles currently animating. The previous flood
/// treated every neighbour of a wildcard as a match (including goal collectables and mixed
/// colours), which reported a move on native run 143211 where the client had none.
fn has_move(run: &Type8Run, spec: &Type8BoardSpec, bounds: Bounds) -> bool {
    const NEIGHBOURS: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    const PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    for y in bounds.top..bounds.top + bounds.height {
        for x in bounds.left..bounds.left + bounds.width {
            // Recovered `ServerGrid.HasMove()` 203-207: a clickable powerup is a
            // legal move on its own, so a board holding one is never declared
            // dead. The rule holds only while such a tile is on the board.
            let value = run.board[y * spec.width + x];
            if (TILE_POWERUP_PRESSME..=TILE_POWERUP_SHUFFLE).contains(&value) {
                return true;
            }
            for (first, second) in PAIRS {
                let Some(one) = neighbour(x, y, NEIGHBOURS[first], bounds) else {
                    continue;
                };
                let Some(other) = neighbour(x, y, NEIGHBOURS[second], bounds) else {
                    continue;
                };
                let values = [
                    run.board[y * spec.width + x],
                    run.board[one.1 * spec.width + one.0],
                    run.board[other.1 * spec.width + other.0],
                ];
                if chain_values_match(values) {
                    return true;
                }
            }
        }
    }
    false
}

/// Step one orthogonal offset from `(x, y)`, or `None` outside the playable bounds.
fn neighbour(x: usize, y: usize, offset: (isize, isize), bounds: Bounds) -> Option<(usize, usize)> {
    let next_x = x as isize + offset.0;
    let next_y = y as isize + offset.1;
    if next_x < 0 || next_y < 0 {
        return None;
    }
    let (next_x, next_y) = (next_x as usize, next_y as usize);
    in_board(next_x, next_y, bounds).then_some((next_x, next_y))
}

/// `validate_chain`'s value rule for one three-cell path: every cell is a selectable tile or a
/// wildcard, and all non-wildcard values are equal. A pure-wildcard path is legal.
fn chain_values_match(values: [u16; 3]) -> bool {
    let mut colour = None;
    for value in values {
        if value == TILE_WILDCARD {
            continue;
        }
        if !(TILE_SELECTABLE_START..TILE_SELECTABLE_START + 100).contains(&value) {
            return false;
        }
        match colour {
            None => colour = Some(value),
            Some(old) if old != value => return false,
            Some(_) => {}
        }
    }
    true
}

fn in_board(x: usize, y: usize, bounds: Bounds) -> bool {
    x >= bounds.left
        && x < bounds.left + bounds.width
        && y >= bounds.top
        && y < bounds.top + bounds.height
}

fn rng(run: &mut Type8Run) -> u32 {
    let mut x = run.rng;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    run.rng = x;
    x
}

/// One action draw from the recovered `aRandomNumberQueue`
/// (`ServerGrid.GetRandomNumberFromQueue`, 783-793). The level start already
/// seeded the queue; a low queue refills by `nRANDOM_QUEUE_BATCHSIZE` and is
/// re-advertised with `SendRandomNumberQueue`, mapping to
/// `GenerateRandomNumbersForQueue` (795-808).
fn action_draw(run: &mut Type8Run, out: &mut Type8Outcome, game_id: i32) -> io::Result<u32> {
    if run.random_queue.is_empty() {
        return Err(invalid("mining random queue empty"));
    }
    let value = run.random_queue.remove(0);
    while run.random_queue.len() < QUEUE_MIN {
        let mut add = Vec::new();
        for _ in 0..QUEUE_BATCH {
            add.push((rng(run) % (RANDOM_MAX_VALUE + 1)) as u16);
        }
        run.random_queue.extend(&add);
        message(
            out,
            run.instance_id,
            game_id,
            &format!("OnRandomQueueMsg\t{}", csv(&add)),
        )?;
    }
    Ok(value as u32)
}

/// Recovered `InitRandomTileArray` (`ServerGrid` 685-712): row-major board order
/// over the playable rectangle, holding every idle eligible tile.
fn init_random_tiles(
    run: &Type8Run,
    spec: &Type8BoardSpec,
    bounds: Bounds,
    eligible: impl Fn(u16) -> bool,
) -> Vec<(usize, usize)> {
    let mut tiles = Vec::new();
    for y in bounds.top..bounds.top + bounds.height {
        for x in bounds.left..bounds.left + bounds.width {
            let i = board_index(spec, (x, y));
            if run.idle[i] && eligible(run.board[i]) {
                tiles.push((x, y));
            }
        }
    }
    tiles
}

/// Recovered `GetRandomTile` (`ServerGrid` 714-724): one queue draw selects the
/// index into the pool and the pick is removed.
fn get_random_tile(
    run: &mut Type8Run,
    out: &mut Type8Outcome,
    game_id: i32,
    tiles: &mut Vec<(usize, usize)>,
) -> io::Result<Option<(usize, usize)>> {
    if tiles.is_empty() {
        return Ok(None);
    }
    let pick = (action_draw(run, out, game_id)? as usize) % tiles.len();
    Ok(Some(tiles.remove(pick)))
}

/// Shared recovered `x,y` list parser. The movie appends `"x,y,"` per tile
/// (`core/Grid.doMatch3`, `core/Grid.UpdateIdle`) and each recovered server
/// reader splits on `,` and pops exactly that one empty tail
/// (`GameLevel.OnChainMsg` 499-500, `OnIdleMsg` 549-550). One trailing empty
/// field is original data; any other empty field stays invalid. `minimum_values`
/// is the reader's own field floor, which differs per message.
fn coordinate_list(list: &str, minimum_values: usize) -> io::Result<Vec<(usize, usize)>> {
    let list = list.strip_suffix(',').unwrap_or(list);
    let values: Vec<usize> = list
        .split(',')
        .map(|v| {
            let n: usize = v
                .trim()
                .parse()
                .map_err(|_| invalid("invalid mining coordinate"))?;
            if n > 4096 {
                return Err(invalid("mining coordinate too large"));
            }
            Ok(n)
        })
        .collect::<io::Result<_>>()?;
    if values.len() < minimum_values || values.len() % 2 != 0 {
        return Err(invalid("malformed mining coordinate list"));
    }
    Ok(values.chunks_exact(2).map(|c| (c[0], c[1])).collect())
}

/// Chain coordinates: the recovered chain reader needs at least
/// `GameData.nMINIMUM_CHAIN_LENGTH` (3) tiles (`ServerGrid.ValidateAndRemoveChain`
/// 103-133 rejects anything shorter).
fn coordinates(list: &str) -> io::Result<Vec<(usize, usize)>> {
    coordinate_list(list, 6)
}

/// Idle acknowledgements: `Grid.UpdateIdle` (1356-1363) sends the pending
/// `sNowIdle` string as soon as it is non-empty, `GameLevel.OnIdleMsg`
/// (543-552) forwards every pair and `ServerGrid.NowIdle` (78-102) loops
/// `length / 2` entries with no minimum. A single fired tile is legal, so the
/// floor here is one pair and the chain's three-tile minimum does not apply.
fn idle_points(list: &str) -> io::Result<Vec<(usize, usize)>> {
    coordinate_list(list, 2)
}

/// Powerup action lists: the pressed tile is always named, so one pair is the
/// floor, matching [`idle_points`] on the shared trailing-comma contract.
fn powerup_points(list: &str) -> io::Result<Vec<(usize, usize)>> {
    coordinate_list(list, 2)
}

fn number_list(list: &str, width: usize) -> io::Result<Vec<u16>> {
    // Same recovered trailing-comma contract as `coordinates`.
    let list = list.strip_suffix(',').unwrap_or(list);
    let values: Vec<u16> = list
        .split(',')
        .map(|v| {
            v.trim()
                .parse()
                .map_err(|_| invalid("invalid mining number list"))
        })
        .collect::<io::Result<_>>()?;
    if values.is_empty() || values.len() % width != 0 {
        return Err(invalid("malformed mining number list"));
    }
    Ok(values)
}

fn number(s: &str) -> io::Result<i64> {
    let n: i64 = s
        .trim()
        .parse()
        .map_err(|_| invalid("invalid mining number"))?;
    if !(-1_000_000..=1_000_000_000).contains(&n) {
        return Err(invalid("mining number too large"));
    }
    Ok(n)
}

fn boolean(s: &str) -> io::Result<bool> {
    match s {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(invalid("invalid mining boolean")),
    }
}

fn csv(v: &[u16]) -> String {
    v.iter().map(u16::to_string).collect::<Vec<_>>().join(",")
}

fn csv_i32(v: &[i32]) -> String {
    v.iter().map(i32::to_string).collect::<Vec<_>>().join(",")
}

fn message(out: &mut Type8Outcome, id: i32, game_id: i32, s: &str) -> io::Result<()> {
    out.replies.push(payload_packet_for(id, game_id, s)?);
    Ok(())
}

/// `39/14` payload packet with the shared `-1` sentinel and this game id.
pub fn payload_packet(id: i32, s: &str) -> io::Result<Vec<u8>> {
    payload_packet_for(id, MINING_GAME_ID, s)
}

fn payload_packet_for(id: i32, game_id: i32, s: &str) -> io::Result<Vec<u8>> {
    validate_text(s)?;
    let mut b = header(14, id, game_id);
    b.extend(((s.len() + 1) as i32).to_le_bytes());
    b.extend(s.as_bytes());
    b.push(0);
    Ok(b)
}

fn decode_payload(b: &[u8]) -> io::Result<&str> {
    if b.len() < 20 {
        return Err(invalid("truncated mining payload"));
    }
    let n = read_i32(b, 15);
    if n < 1 || n as usize > MAX_PAYLOAD || b.len() != 19 + n as usize || b.last() != Some(&0) {
        return Err(invalid("invalid mining payload length"));
    }
    let s = std::str::from_utf8(&b[19..b.len() - 1]).map_err(invalid)?;
    validate_text(s)?;
    Ok(s)
}

fn validate_text(s: &str) -> io::Result<()> {
    if s.is_empty()
        || s.len() + 1 > MAX_PAYLOAD
        || !s.bytes().all(|b| b == b'\t' || (32..=126).contains(&b))
    {
        return Err(invalid("invalid mining text"));
    }
    Ok(())
}

fn header(sub: u8, id: i32, game_id: i32) -> Vec<u8> {
    let mut b = TYPE8_GAME_FAMILY.to_le_bytes().to_vec();
    b.push(sub);
    for n in [id, -1, game_id] {
        b.extend(n.to_le_bytes());
    }
    b
}

fn game_over(id: i32, game_id: i32, won: bool, elapsed: u32, score: u32) -> Vec<u8> {
    let mut b = header(18, id, game_id);
    b.push(u8::from(won));
    for n in [0_i32, elapsed as i32, score as i32] {
        b.extend(n.to_le_bytes());
    }
    b
}

fn results(run: &Type8Run, won: bool, duration: u32, game_id: i32) -> Vec<Vec<u8>> {
    let elapsed = duration
        .saturating_mul(1000)
        .saturating_sub(run.remaining_ms)
        / 1000;
    let mut scores = header(47, run.instance_id, game_id);
    scores.extend(1_i32.to_le_bytes());
    let name = b"scoreTotalScore";
    scores.extend((name.len() as i32).to_le_bytes());
    scores.extend(name);
    for n in [-1_i32, 4, -1, -1, run.score as i32] {
        scores.extend(n.to_le_bytes());
    }
    vec![
        game_over(run.instance_id, game_id, won, elapsed, run.score),
        scores,
        header(19, run.instance_id, game_id),
    ]
}

fn launch_packet(movie: &str) -> Vec<u8> {
    let mut b = [26_u16.to_le_bytes(), 12_u16.to_le_bytes()].concat();
    for s in ["MiniGameFlash", movie] {
        b.extend((s.len() as i32).to_le_bytes());
        b.extend(s.as_bytes());
    }
    b.push(0);
    b
}

fn read_i32(b: &[u8], p: usize) -> i32 {
    i32::from_le_bytes(b[p..p + 4].try_into().unwrap())
}

fn invalid(e: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
}

/// Launch/preload names travel in recovered packets, so accept only plain
/// asset file names; anything path-like or empty fails the load closed.
fn validate_asset_name(name: &str, what: &str) -> io::Result<()> {
    let plain = !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && !name.ends_with('.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.');
    if plain {
        Ok(())
    } else {
        Err(invalid(format!("invalid mining {what} name")))
    }
}
