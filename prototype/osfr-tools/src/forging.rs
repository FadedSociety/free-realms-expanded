//! Forging139 drives the original match-three movie with a server-owned board.
//!
//! (R) Board/timer tuning and the sample selection are reconstructed. Recipe
//! rewards come from the local catalog; persist this clone with inventory and
//! opening before committing state or sending replies.
//! On reconnect call `cancel`: a fresh movie cannot resume an old tile queue.
use crate::recipes::{CraftingJob, RecipeCatalog, RecipeFile};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

const GAME: i32 = 139;
const PROFILE: i32 = 16;
const FIRST_INSTANCE: i32 = 139_000_001;
const SIDE: usize = 14; // Native movie's optional verification walks14x14.
const CELLS: usize = SIDE * SIDE;
const MAX_CONTENT: u64 = 32 * 1024;
const MAX_PAYLOAD: usize = 4096;
const MAX_WALL_MS: u32 = 1_800_000;
const COLLECTABLE: u16 = 606; // Mapping index6 is the original Hammer art.

#[derive(Clone, Debug, Deserialize)]
struct Content {
    schema_version: u32,
    game_id: i32,
    profile_id: i32,
    name_id: i32,
    description_id: i32,
    icon_id: i32,
    reconstructed_rules: bool,
    rule_notice: String,
    board: BoardConfig,
    recipe: RecipeSelector,
}

#[derive(Clone, Debug, Deserialize)]
struct BoardConfig {
    // One rectangular playable area with a drop-zone below every column.
    left: usize,
    top: usize,
    width: usize,
    height: usize,
    duration_seconds: u32,
    quota: u32,
    selectable_types: Vec<u16>,
    collectable_percent: u32,
    seed: u32,
    mold_id: u32,
}

#[derive(Clone, Debug, Deserialize)]
struct RecipeSelector {
    #[serde(default = "default_recipe_key")]
    key: String,
    #[serde(default)]
    reconstructed_consumption: bool,
    #[serde(default)]
    reconstructed_tint_selection: bool,
}

#[derive(Clone, Debug)]
struct Recipe {
    job_rank: Option<u8>,
    game_id: Option<i32>,
    consumes: Vec<(i32, u32)>,
    product_definition_id: i32,
    product_quantity: u32,
    experience_points: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForgingState {
    pub version: u32,
    pub next_instance_id: i32,
    pub completed: u32,
    #[serde(default)]
    pub active_recipe_key: Option<String>,
    pub active: Option<ForgingRun>,
}
impl Default for ForgingState {
    fn default() -> Self {
        Self {
            version: 1,
            next_instance_id: FIRST_INSTANCE,
            completed: 0,
            active_recipe_key: None,
            active: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForgingPhase {
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
pub struct ForgingRun {
    pub instance_id: i32,
    pub phase: ForgingPhase,
    pub board: Vec<u16>,
    pub idle: Vec<bool>,
    pub tile_queue: Vec<u16>,
    pub rng: u32,
    pub collected: u32,
    pub score: u32,
    pub remaining_ms: u32,
    pub wall_ms: u32,
    pub paused: bool,
    pub chains: u32,
    pub reshuffles: u32,
    pub end_sent: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForgingTransaction {
    pub consumes: Vec<(i32, u32)>,
    pub grants: Vec<(i32, u32)>,
    pub experience_points: u32,
}

#[derive(Debug, Default)]
pub struct ForgingOutcome {
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
    pub completed: bool,
    pub transaction: Option<ForgingTransaction>,
}

#[derive(Clone, Debug)]
pub struct Forging {
    content: Arc<Content>,
    recipes: HashMap<String, Recipe>,
    state: ForgingState,
    results_presented: bool,
}

impl Forging {
    pub fn load(directory: &Path, state: Option<ForgingState>) -> io::Result<Self> {
        let path = directory.join("forging.json");
        if fs::metadata(&path)?.len() > MAX_CONTENT {
            return Err(invalid("forging content too large"));
        }
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(MAX_CONTENT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_CONTENT {
            return Err(invalid("forging content too large"));
        }
        let content: Content = serde_json::from_slice(&bytes).map_err(invalid)?;
        let recipes = load_recipes(directory)?;
        validate_content(&content, &recipes)?;
        let state = state.unwrap_or_default();
        Self::validate_state(&state)?;
        if state
            .active_recipe_key
            .as_ref()
            .is_some_and(|key| !recipes.contains_key(key))
        {
            return Err(invalid("saved forging recipe is unavailable"));
        }
        if let Some(a) = &state.active {
            validate_run(a, &content.board)?;
        }
        Ok(Self {
            content: Arc::new(content),
            recipes,
            state,
            results_presented: false,
        })
    }

    pub fn state(&self) -> &ForgingState {
        &self.state
    }
    pub fn active(&self) -> bool {
        self.state.active.is_some()
    }
    pub fn requirements(&self) -> &[(i32, u32)] {
        &self.selected_recipe().consumes
    }
    pub fn recipe_key(&self) -> &str {
        self.state
            .active_recipe_key
            .as_deref()
            .unwrap_or(&self.content.recipe.key)
    }

    pub fn validate_state(s: &ForgingState) -> io::Result<()> {
        if s.version != 1
            || s.next_instance_id < FIRST_INSTANCE
            || s.completed > (s.next_instance_id - FIRST_INSTANCE) as u32
            || s.active_recipe_key.as_ref().is_some_and(String::is_empty)
        {
            return Err(invalid("invalid forging state"));
        }
        if let Some(a) = &s.active {
            if a.instance_id < FIRST_INSTANCE
                || a.instance_id >= s.next_instance_id
                || a.board.len() > CELLS
                || a.idle.len() != a.board.len()
                || a.tile_queue.len() > 70
                || a.rng == 0
                || a.collected > 100
                || a.score > 1_000_000_000
                || a.remaining_ms > 600_000
                || a.wall_ms > MAX_WALL_MS
                || a.chains > 100_000
                || a.reshuffles > 1024
                || a.board
                    .iter()
                    .chain(&a.tile_queue)
                    .any(|v| !valid_value(*v))
                || (a.phase == ForgingPhase::Won && s.completed == 0)
                || (a.end_sent && !matches!(a.phase, ForgingPhase::Won | ForgingPhase::Lost))
            {
                return Err(invalid("invalid forging run"));
            }
        }
        Ok(())
    }

    /// (R) Compatibility entry for the default recipe; the gateway must use
    /// `begin_recipe` with learned state, rank, and actual inventory.
    pub fn begin(&mut self, profile: i32) -> io::Result<ForgingOutcome> {
        let key = self
            .state
            .active_recipe_key
            .as_deref()
            .unwrap_or(&self.content.recipe.key)
            .to_owned();
        let available = self
            .recipes
            .get(&key)
            .ok_or_else(|| invalid("unknown forging recipe"))?
            .consumes
            .clone();
        self.begin_recipe(profile, &key, u8::MAX, true, &available)
    }

    /// Starts a learned, rank-unlocked catalog recipe after checking exact
    /// ingredients. All rejection checks precede mutation.
    pub fn begin_recipe(
        &mut self,
        profile: i32,
        recipe_key: &str,
        job_rank: u8,
        learned: bool,
        available: &[(i32, u32)],
    ) -> io::Result<ForgingOutcome> {
        if profile != PROFILE || self.active() {
            return Err(invalid("forging requires an idle Blacksmith"));
        }
        let recipe = self
            .recipes
            .get(recipe_key)
            .ok_or_else(|| invalid("unknown forging recipe"))?;
        if !learned
            || recipe.job_rank.is_some_and(|required| job_rank < required)
            || !has_ingredients(&recipe.consumes, available)
        {
            return Err(invalid(
                "forging recipe is locked or ingredients are missing",
            ));
        }
        let id = self.state.next_instance_id;
        let next = id
            .checked_add(1)
            .ok_or_else(|| invalid("forging IDs exhausted"))?;
        let rng = self.content.board.seed ^ (id as u32).wrapping_mul(0x9e37_79b9);
        let a = ForgingRun {
            instance_id: id,
            phase: ForgingPhase::Offered,
            board: Vec::new(),
            idle: Vec::new(),
            tile_queue: Vec::new(),
            rng: rng.max(1),
            collected: 0,
            score: 0,
            remaining_ms: self.content.board.duration_seconds * 1000,
            wall_ms: 0,
            paused: true,
            chains: 0,
            reshuffles: 0,
            end_sent: false,
        };
        self.state.next_instance_id = next;
        self.state.active_recipe_key = Some(recipe_key.to_owned());
        self.state.active = Some(a);
        self.results_presented = false;
        Ok(ForgingOutcome {
            replies: vec![self.info_packet(id)],
            changed: true,
            ..Default::default()
        })
    }

    fn selected_recipe(&self) -> &Recipe {
        let key = self
            .state
            .active_recipe_key
            .as_deref()
            .unwrap_or(&self.content.recipe.key);
        &self.recipes[key]
    }

    fn selected_game_id(&self) -> i32 {
        // (R) A missing table binding reuses the configured sample movie.
        self.selected_recipe()
            .game_id
            .unwrap_or(self.content.game_id)
    }

    pub fn cancel(&mut self) -> ForgingOutcome {
        let Some(a) = self.state.active.take() else {
            return ForgingOutcome::default();
        };
        let replies = if self.results_presented {
            vec![]
        } else {
            vec![header_with_game(19, a.instance_id, self.selected_game_id())]
        };
        self.results_presented = false;
        ForgingOutcome {
            replies,
            changed: true,
            ..Default::default()
        }
    }

    /// Elapsed server time between calls, in milliseconds. Never client-reported.
    pub fn tick(&mut self, elapsed_ms: u32) -> io::Result<ForgingOutcome> {
        let Some(current) = &self.state.active else {
            return Ok(ForgingOutcome::default());
        };
        if elapsed_ms == 0 || matches!(current.phase, ForgingPhase::Won | ForgingPhase::Lost) {
            return Ok(ForgingOutcome::default());
        }
        let game_id = self.selected_game_id();
        let mut next = self.state.clone();
        let a = next.active.as_mut().unwrap();
        a.wall_ms = a.wall_ms.saturating_add(elapsed_ms).min(MAX_WALL_MS);
        let mut out = ForgingOutcome {
            changed: true,
            ..Default::default()
        };
        if a.phase == ForgingPhase::Playing && !a.paused {
            a.remaining_ms = a.remaining_ms.saturating_sub(elapsed_ms);
        }
        if a.wall_ms == MAX_WALL_MS || (a.phase == ForgingPhase::Playing && a.remaining_ms == 0) {
            a.phase = ForgingPhase::Lost;
            // Whole-movie Pause suppresses the native end-stage/GameOver
            // timers. StopLevel blocks only board input; keep GFx ticking.
            if a.paused {
                message_with_game(&mut out, a.instance_id, "OnPauseMsg\ttrue", game_id)?;
                a.paused = false;
            }
            message_with_game(&mut out, a.instance_id, "OnStopLevel\t1\tfalse", game_id)?;
        }
        self.state = next;
        Ok(out)
    }

    pub fn handle(&mut self, inner: &[u8]) -> io::Result<ForgingOutcome> {
        if inner.len() < 2 || inner[..2] != 39_u16.to_le_bytes() {
            return Ok(ForgingOutcome::default());
        }
        if inner.len() < 15 {
            return Err(invalid("truncated minigame header"));
        }
        let sub = inner[2];
        if !matches!(sub, 5 | 6 | 7 | 14) {
            return Ok(ForgingOutcome::default());
        }
        let msg = if sub == 14 {
            Some(decode_payload(inner)?)
        } else {
            if inner.len() != 15 {
                return Err(invalid("invalid forging start request"));
            }
            None
        };
        let Some(a) = &self.state.active else {
            return Ok(ForgingOutcome::default());
        };
        if sub == 7 {
            // Removal ACK is not the actual native results Close26/42.
            return Ok(ForgingOutcome::default());
        }
        if sub == 6 {
            // Shared original Ui.QuitMiniGame00c17470 ->00939100 ->009be8e0 ->
            //009bdc60(6,-1,-1) serializes the current instance with both
            // sentinels. Ordinary Quit cancels only a pre-win run; terminal
            // Won/Lost state and economy stay until the actual results
            // Close26/42, and no transaction is created.
            if matches!(a.phase, ForgingPhase::Won | ForgingPhase::Lost)
                || read_i32(inner, 3) != a.instance_id
                || read_i32(inner, 7) != -1
                || read_i32(inner, 11) != -1
            {
                return Ok(ForgingOutcome::default());
            }
            return Ok(self.cancel());
        }
        // Shared original RequestStartGame 009be7d0 -> 009bdc60 uses the
        // current instance and GameId=-1 only at the offered start boundary.
        let game = read_i32(inner, 11);
        let game_id = self.selected_game_id();
        let native_offer_start = sub == 5 && a.phase == ForgingPhase::Offered && game == -1;
        // Native shared SendPayload constructor009b09d0 keeps both sentinels.
        let native_payload = sub == 14 && a.phase != ForgingPhase::Offered && game == -1;
        if read_i32(inner, 3) != a.instance_id
            || read_i32(inner, 7) != -1
            || (game != game_id && !native_offer_start && !native_payload)
        {
            return Ok(ForgingOutcome::default());
        }
        let mut next = self.state.clone();
        let a = next.active.as_mut().unwrap();
        let id = a.instance_id;
        let mut out = ForgingOutcome::default();
        let mut presented = self.results_presented;
        if sub == 5 {
            out.replies.push(header_with_game(17, id, game_id));
            if a.phase == ForgingPhase::Offered {
                out.replies.push(launch_packet());
                a.phase = ForgingPhase::Launched;
            }
        } else {
            let fields: Vec<_> = msg.unwrap().split('\t').collect();
            match fields.as_slice() {
                ["OnConnectMsg"] => {
                    if a.phase == ForgingPhase::Launched {
                        a.phase = ForgingPhase::Connected;
                        message_with_game(&mut out, id, "OnServerReadyMsg", game_id)?;
                    }
                }
                ["FRServer_Connect"] | ["FRServer_ReadyForGameData"] => {
                    // Wait for the actual match-three network callback to be
                    // installed and OnConnectMsg sent before ServerReady.
                }
                ["OnStartGameMsg"] | ["OnStartGameMsg", "0"] => {
                    if a.phase == ForgingPhase::Connected {
                        a.phase = ForgingPhase::LevelOffered;
                        // Client BeginLevel constructs the grid before StartLevel.
                        message_with_game(&mut out, id, "OnNextLevelMsg\t1\t0\t0", game_id)?;
                    }
                }
                ["OnStartLevelMsg"] | ["OnStartLevelMsg", "0"] => {
                    if a.phase == ForgingPhase::LevelOffered {
                        a.phase = ForgingPhase::Playing;
                        a.paused = false;
                        initialize_board(a, &self.content.board, &mut out, false, game_id)?;
                        message_with_game(
                            &mut out,
                            id,
                            &format!(
                                "OnLevelInitMsg\t1\t{}\t7",
                                self.content.board.duration_seconds
                            ),
                            game_id,
                        )?;
                        message_with_game(
                            &mut out,
                            id,
                            &format!(
                                "OnQuotaInitMsg\t0\t{COLLECTABLE}\t{}\t1\t{}",
                                self.content.board.quota, self.content.icon_id
                            ),
                            game_id,
                        )?;
                        message_with_game(
                            &mut out,
                            id,
                            &format!(
                                "OnLevelEventMsg\t32\t{}\t{}",
                                self.content.board.mold_id, self.content.board.quota
                            ),
                            game_id,
                        )?;
                        message_with_game(&mut out, id, "OnActiveMsg\ttrue", game_id)?;
                    }
                }
                ["OnChainMsg", list] => {
                    if a.phase == ForgingPhase::Playing {
                        if a.paused {
                            return Err(invalid("forging is paused"));
                        }
                        let chain = coordinates(list)?;
                        validate_chain(a, &chain, &self.content.board)?;
                        for &(x, y) in &chain {
                            a.board[y * SIDE + x] = 0;
                        }
                        a.chains = a
                            .chains
                            .checked_add(1)
                            .ok_or_else(|| invalid("chain count overflow"))?;
                        if a.chains > 100_000 {
                            return Err(invalid("too many forging chains"));
                        }
                        let score = 400 + (chain.len() as u32 - 3) * 200;
                        a.score = a
                            .score
                            .checked_add(score)
                            .filter(|s| *s <= 1_000_000_000)
                            .ok_or_else(|| invalid("forging score overflow"))?;
                        message_with_game(
                            &mut out,
                            id,
                            &format!("OnLevelEventMsg\t21\t{score}\t0"),
                            game_id,
                        )?;
                        let bonus = 2 * chain.len() as u32;
                        let applied = (600_000 - a.remaining_ms).min(bonus * 1000) / 1000;
                        a.remaining_ms += applied * 1000;
                        message_with_game(
                            &mut out,
                            id,
                            &format!("OnLevelEventMsg\t25\t{applied}\t1"),
                            game_id,
                        )?;
                        gravity(a, &self.content.board, &mut out, game_id)?;
                        if !has_move(a, &self.content.board)
                            && !has_bottom_piece(a, &self.content.board)
                        {
                            initialize_board(a, &self.content.board, &mut out, true, game_id)?;
                        }
                    } else {
                        let _ = coordinates(list)?;
                    }
                }
                ["OnIdleMsg", list] => {
                    let points = coordinates(list)?;
                    if a.phase == ForgingPhase::Playing {
                        for (x, y) in points {
                            if !in_board(x, y, &self.content.board) {
                                return Err(invalid("idle outside forging area"));
                            }
                            a.idle[y * SIDE + x] = true;
                        }
                    }
                }
                ["OnVerifyMsg", list] => {
                    let values = number_list(list, 3)?;
                    if a.phase == ForgingPhase::Playing {
                        for v in values.chunks_exact(3) {
                            let (x, y) = (v[0] as usize, v[1] as usize);
                            if x >= SIDE || y >= SIDE || a.board[y * SIDE + x] != v[2] {
                                return Err(invalid("forging board verification mismatch"));
                            }
                        }
                    }
                }
                ["OnLevelEventMsg", "50", x, y] => {
                    let (x, y) = (number(x)? as usize, number(y)? as usize);
                    if a.phase == ForgingPhase::Playing {
                        let b = &self.content.board;
                        if !in_board(x, y, b)
                            || y != b.top + b.height - 1
                            || a.board[y * SIDE + x] != COLLECTABLE
                        {
                            return Err(invalid("invalid hammer-piece drop"));
                        }
                        a.board[y * SIDE + x] = 0;
                        a.collected += 1;
                        a.score = a
                            .score
                            .checked_add(500)
                            .filter(|s| *s <= 1_000_000_000)
                            .ok_or_else(|| invalid("forging score overflow"))?;
                        message_with_game(
                            &mut out,
                            id,
                            &format!("OnQuotaUpdateMsg\t0\t{}\t{}\t1", a.collected, b.quota),
                            game_id,
                        )?;
                        message_with_game(&mut out, id, "OnLevelEventMsg\t21\t500\t3", game_id)?;
                        // Client removes this piece and immediately consumes queue
                        // entries; match it even for the final successful piece.
                        gravity(a, b, &mut out, game_id)?;
                        if a.collected >= b.quota {
                            a.phase = ForgingPhase::LevelComplete;
                            if a.paused {
                                message_with_game(&mut out, id, "OnPauseMsg\ttrue", game_id)?;
                                a.paused = false;
                            }
                            message_with_game(&mut out, id, "OnStopLevel\t1\ttrue", game_id)?;
                        } else if !has_move(a, b) && !has_bottom_piece(a, b) {
                            initialize_board(a, b, &mut out, true, game_id)?;
                        }
                    }
                }
                ["OnLevelEventMsg", "60"] | ["OnLevelEventMsg", "61"] => {
                    // This bounded board only appends queue messages (reset=-1).
                    // No queue reset is pending, so no untrusted ACK mutates it.
                }
                ["FRServer_Pause", v] | ["OnPauseMsg", v] => {
                    let paused = boolean(v)?;
                    if a.phase == ForgingPhase::Playing && paused != a.paused {
                        a.paused = paused;
                        message_with_game(
                            &mut out,
                            id,
                            if paused {
                                "OnPauseMsg\tfalse"
                            } else {
                                "OnPauseMsg\ttrue"
                            },
                            game_id,
                        )?;
                    }
                }
                ["OnNextLevelMsg"] => {
                    if a.phase == ForgingPhase::LevelComplete {
                        a.phase = ForgingPhase::Won;
                        next.completed = next
                            .completed
                            .checked_add(1)
                            .ok_or_else(|| invalid("completion overflow"))?;
                        message_with_game(&mut out, id, "OnGameWonMsg", game_id)?;
                        out.completed = true;
                        let recipe = self.selected_recipe();
                        out.transaction = Some(ForgingTransaction {
                            consumes: recipe.consumes.clone(),
                            grants: vec![(recipe.product_definition_id, recipe.product_quantity)],
                            experience_points: recipe.experience_points,
                        });
                    }
                }
                ["FRServer_GameOver"] => {
                    if matches!(a.phase, ForgingPhase::Won | ForgingPhase::Lost) && !presented {
                        out.replies.extend(results(
                            a,
                            a.phase == ForgingPhase::Won,
                            self.content.board.duration_seconds,
                            game_id,
                        ));
                        a.end_sent = true;
                        presented = true;
                    }
                }
                ["FRServer_GameClose"] => {
                    next.active = None;
                    if !presented {
                        out.replies.push(header_with_game(19, id, game_id));
                    }
                    presented = false;
                }
                ["FRServer_GameWon", score]
                | ["FRServer_GameWon", score, _]
                | ["FRServer_GameLost", score]
                | ["FRServer_GameQuit", score]
                | ["FRServer_ScoreSet", score] => {
                    // Informational client reports never affect authoritative
                    // score, board, completion, recipe, output or quantities.
                    let _ = number(score)?;
                    if fields.len() == 3 {
                        let _ = number(fields[2])?;
                    }
                }
                _ => return Err(invalid("unsupported or malformed forging message")),
            }
        }
        out.changed = next != self.state;
        Self::validate_state(&next)?;
        if let Some(a) = &next.active {
            validate_run(a, &self.content.board)?;
        }
        self.state = next;
        self.results_presented = presented;
        Ok(out)
    }

    fn info_packet(&self, id: i32) -> Vec<u8> {
        let c = &self.content;
        let game_id = self.selected_game_id();
        let mut b = header_with_game(16, id, game_id);
        for n in [c.name_id, c.icon_id, c.description_id, 1, PROFILE, 8] {
            b.extend(n.to_le_bytes());
        }
        b.push(0);
        b.extend([0; 207]); // Three empty69-byte RewardBundleBase records.
        b.extend(0_i32.to_le_bytes());
        b.extend([1; 5]);
        let s = b"ForgingAssets.txt";
        b.extend((s.len() as i32).to_le_bytes());
        b.extend(s);
        b.extend(1_i32.to_le_bytes());
        b.push(1);
        b.extend(game_id.to_le_bytes());
        b.extend([0; 4]);
        b.extend([0; 8]);
        b
    }
}

fn initialize_board(
    a: &mut ForgingRun,
    c: &BoardConfig,
    out: &mut ForgingOutcome,
    reshuffle: bool,
    game_id: i32,
) -> io::Result<()> {
    if reshuffle {
        a.reshuffles += 1;
        if a.reshuffles > 1024 {
            return Err(invalid("too many forging reshuffles"));
        }
        message_with_game(out, a.instance_id, "OnLevelEventMsg\t11\t1", game_id)?;
    }
    a.board = vec![0; CELLS];
    a.idle = vec![true; CELLS];
    a.tile_queue.clear();
    let mut shape = vec![0_u16; CELLS];
    for y in c.top..c.top + c.height {
        for x in c.left..c.left + c.width {
            // Initial bottom two rows contain selectable tiles so entry does not
            // award pieces automatically. The original139 distribution is unknown.
            let v = random_tile(a, c, y + 2 < c.top + c.height);
            a.board[y * SIDE + x] = v;
            shape[y * SIDE + x] = 1;
        }
    }
    for x in c.left..c.left + c.width {
        let i = (c.top + c.height) * SIDE + x;
        a.board[i] = 100;
        shape[i] = 4;
    }
    // Guarantee an initial legal move; reshuffling is reconstructed numeric
    // tuning, while the movie's actual interaction/rules remain unchanged.
    for x in c.left..c.left + 3 {
        a.board[(c.top + c.height - 1) * SIDE + x] = 400 + c.selectable_types[0];
    }
    if !reshuffle {
        for &t in &c.selectable_types {
            message_with_game(
                out,
                a.instance_id,
                &format!("OnTileInfoMsg\t{}\t{t}", 400 + t),
                game_id,
            )?;
        }
        message_with_game(
            out,
            a.instance_id,
            &format!("OnTileInfoMsg\t{COLLECTABLE}\t6"),
            game_id,
        )?;
    }
    message_with_game(
        out,
        a.instance_id,
        &format!("OnTileQueueMsg\t-1\t{}", csv(&a.board)),
        game_id,
    )?;
    message_with_game(
        out,
        a.instance_id,
        &format!("OnBuildMapMsg\t{SIDE}\t{SIDE}\t{}", csv(&shape)),
        game_id,
    )?;
    let mut values = Vec::new();
    for _ in 0..70 {
        values.push(random_tile(a, c, true));
    }
    a.tile_queue = values;
    message_with_game(
        out,
        a.instance_id,
        &format!("OnTileQueueMsg\t-1\t{}", csv(&a.tile_queue)),
        game_id,
    )?;
    if !reshuffle {
        // No supported powerup consumes random numbers in this board.
        let mut random = Vec::new();
        for _ in 0..70 {
            random.push((rng(a) % 1000) as u16);
        }
        message_with_game(
            out,
            a.instance_id,
            &format!("OnRandomQueueMsg\t{}", csv(&random)),
            game_id,
        )?;
    }
    Ok(())
}

fn gravity(
    a: &mut ForgingRun,
    c: &BoardConfig,
    out: &mut ForgingOutcome,
    game_id: i32,
) -> io::Result<()> {
    // AS2 scans columns right-to-left and holes bottom-to-top. The earliest
    // queued replacement ends lowest among the new top rows of that column.
    for x in (c.left..c.left + c.width).rev() {
        let mut target = c.top + c.height;
        for y in (c.top..c.top + c.height).rev() {
            let v = a.board[y * SIDE + x];
            if v != 0 {
                target -= 1;
                if target != y {
                    a.board[target * SIDE + x] = v;
                    a.idle[target * SIDE + x] = false;
                }
            }
        }
        while target > c.top {
            target -= 1;
            let v = *a
                .tile_queue
                .first()
                .ok_or_else(|| invalid("forging tile queue empty"))?;
            a.tile_queue.remove(0);
            a.board[target * SIDE + x] = v;
            a.idle[target * SIDE + x] = false;
            if a.tile_queue.len() < 50 {
                let mut add = Vec::new();
                for _ in 0..20 {
                    add.push(random_tile(a, c, true));
                }
                a.tile_queue.extend(&add);
                message_with_game(
                    out,
                    a.instance_id,
                    &format!("OnTileQueueMsg\t-1\t{}", csv(&add)),
                    game_id,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_chain(a: &ForgingRun, points: &[(usize, usize)], c: &BoardConfig) -> io::Result<()> {
    if points.len() < 3 {
        return Err(invalid("forging chain needs at least3 tiles"));
    }
    let mut seen = HashSet::new();
    let mut value = None;
    let mut previous: Option<(usize, usize)> = None;
    for &(x, y) in points {
        if !in_board(x, y, c) || !seen.insert((x, y)) {
            return Err(invalid("forging chain outside board or repeated tile"));
        }
        if let Some((px, py)) = previous {
            if x.abs_diff(px) + y.abs_diff(py) != 1 {
                return Err(invalid("forging chain is not orthogonal"));
            }
        }
        let i = y * SIDE + x;
        let v = a.board[i];
        if !a.idle[i] || !(401..=405).contains(&v) {
            return Err(invalid("forging chain tile unavailable"));
        }
        if value.is_some_and(|old| old != v) {
            return Err(invalid("forging chain colors differ"));
        }
        value = Some(v);
        previous = Some((x, y));
    }
    Ok(())
}

fn has_move(a: &ForgingRun, c: &BoardConfig) -> bool {
    let mut seen = [false; CELLS];
    for y in c.top..c.top + c.height {
        for x in c.left..c.left + c.width {
            let start = y * SIDE + x;
            let v = a.board[start];
            if seen[start] || !(401..=405).contains(&v) {
                continue;
            }
            let mut stack = vec![(x, y)];
            seen[start] = true;
            let mut count = 0;
            while let Some((x, y)) = stack.pop() {
                count += 1;
                if count >= 3 {
                    return true;
                }
                for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    let i = ny * SIDE + nx;
                    if in_board(nx, ny, c) && !seen[i] && a.board[i] == v {
                        seen[i] = true;
                        stack.push((nx, ny));
                    }
                }
            }
        }
    }
    false
}
fn has_bottom_piece(a: &ForgingRun, c: &BoardConfig) -> bool {
    (c.left..c.left + c.width).any(|x| a.board[(c.top + c.height - 1) * SIDE + x] == COLLECTABLE)
}
fn in_board(x: usize, y: usize, c: &BoardConfig) -> bool {
    x >= c.left && x < c.left + c.width && y >= c.top && y < c.top + c.height
}
fn rng(a: &mut ForgingRun) -> u32 {
    let mut x = a.rng;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    a.rng = x;
    x
}
fn random_tile(a: &mut ForgingRun, c: &BoardConfig, pieces: bool) -> u16 {
    let n = rng(a);
    if pieces && n % 100 < c.collectable_percent {
        COLLECTABLE
    } else {
        400 + c.selectable_types[(n / 100) as usize % c.selectable_types.len()]
    }
}
fn valid_value(v: u16) -> bool {
    matches!(v, 0 | 100 | 401..=405 | COLLECTABLE)
}

fn default_recipe_key() -> String {
    "blacksmith.spunky_scrapper_hammer.sample".into()
}

fn load_recipes(directory: &Path) -> io::Result<HashMap<String, Recipe>> {
    let package_path = directory.join("recipes.json");
    let path = if package_path.is_file() {
        package_path
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes.json")
    };
    let file: RecipeFile = serde_json::from_slice(&fs::read(&path)?).map_err(invalid)?;
    let item_definitions: HashSet<_> = file
        .records
        .iter()
        .flat_map(|recipe| {
            recipe
                .ingredients
                .iter()
                .map(|item| item.definition_id)
                .chain([recipe.output.definition_id])
                .chain(recipe.recipe_item_definition_id)
        })
        .collect();
    let catalog = RecipeCatalog::load(path, &item_definitions)?;
    let mut recipes = HashMap::new();
    for row in catalog.by_job(CraftingJob::Blacksmith) {
        if row.minigame_type_id != 8 {
            continue;
        }
        let game_id = row
            .minigame_id
            .map(|id| i32::try_from(id).map_err(invalid))
            .transpose()?;
        recipes.insert(
            row.key.clone(),
            Recipe {
                job_rank: row.required_job_rank,
                game_id,
                consumes: row
                    .ingredients
                    .iter()
                    .map(|item| (item.definition_id, item.count))
                    .collect(),
                product_definition_id: row.output.definition_id,
                product_quantity: row.output.count,
                experience_points: row.experience_points,
            },
        );
    }
    if recipes.is_empty() {
        return Err(invalid("recipe table has no Blacksmith forging recipes"));
    }
    Ok(recipes)
}

fn has_ingredients(required: &[(i32, u32)], available: &[(i32, u32)]) -> bool {
    required.iter().all(|(definition, needed)| {
        available
            .iter()
            .filter(|(id, _)| id == definition)
            .map(|(_, count)| u64::from(*count))
            .sum::<u64>()
            >= u64::from(*needed)
    })
}

fn validate_content(c: &Content, recipes: &HashMap<String, Recipe>) -> io::Result<()> {
    let b = &c.board;
    let r = &c.recipe;
    if c.schema_version != 1
        || c.game_id != GAME
        || c.profile_id != PROFILE
        || c.name_id <= 0
        || c.description_id <= 0
        || c.icon_id <= 0
        || !c.reconstructed_rules
        || c.rule_notice.is_empty()
        || c.rule_notice.len() > 2048
        || b.left < 1
        || b.top < 1
        || !(3..=10).contains(&b.width)
        || !(3..=10).contains(&b.height)
        || b.left > 10
        || b.top > 10
        || b.left + b.width >= SIDE
        || b.top + b.height >= SIDE
        || !(30..=300).contains(&b.duration_seconds)
        || !(1..=100).contains(&b.quota)
        || 100 % b.quota != 0
        || b.selectable_types.is_empty()
        || b.selectable_types.len() > 5
        || b.selectable_types.iter().any(|t| !(1..=5).contains(t))
        || b.selectable_types
            .iter()
            .copied()
            .collect::<HashSet<_>>()
            .len()
            != b.selectable_types.len()
        || !(1..=20).contains(&b.collectable_percent)
        || b.seed == 0
        || b.mold_id != 6
        || !recipes.contains_key(&r.key)
        || !r.reconstructed_consumption
        || !r.reconstructed_tint_selection
    {
        return Err(invalid("invalid or unlabelled forging content"));
    }
    Ok(())
}
fn validate_run(a: &ForgingRun, c: &BoardConfig) -> io::Result<()> {
    let started = matches!(
        a.phase,
        ForgingPhase::Playing | ForgingPhase::LevelComplete | ForgingPhase::Won
    ) || (a.phase == ForgingPhase::Lost && !a.board.is_empty());
    if started {
        if a.board.len() != CELLS
            || !(50..=70).contains(&a.tile_queue.len())
            || a.collected > c.quota
            || a.tile_queue
                .iter()
                .any(|v| *v != COLLECTABLE && !c.selectable_types.contains(&v.saturating_sub(400)))
        {
            return Err(invalid("invalid forging board snapshot"));
        }
        for y in 0..SIDE {
            for x in 0..SIDE {
                let v = a.board[y * SIDE + x];
                let valid = if in_board(x, y, c) {
                    v == COLLECTABLE || c.selectable_types.contains(&v.saturating_sub(400))
                } else if y == c.top + c.height && x >= c.left && x < c.left + c.width {
                    v == 100
                } else {
                    v == 0
                };
                if !valid {
                    return Err(invalid("forging snapshot shape mismatch"));
                }
            }
        }
    } else if !a.board.is_empty()
        || !a.idle.is_empty()
        || !a.tile_queue.is_empty()
        || a.collected != 0
        || a.score != 0
        || a.chains != 0
        || a.reshuffles != 0
    {
        return Err(invalid("forging snapshot started prematurely"));
    }
    if matches!(a.phase, ForgingPhase::LevelComplete | ForgingPhase::Won) && a.collected != c.quota
    {
        return Err(invalid("forging snapshot victory lacks quota"));
    }
    Ok(())
}

fn coordinates(list: &str) -> io::Result<Vec<(usize, usize)>> {
    Ok(number_list(list, 2)?
        .chunks_exact(2)
        .map(|p| (p[0] as usize, p[1] as usize))
        .collect())
}
fn number_list(list: &str, stride: usize) -> io::Result<Vec<u16>> {
    let list = list.strip_suffix(',').unwrap_or(list);
    if list.is_empty() {
        return Err(invalid("empty forging coordinate list"));
    }
    let fields: Vec<_> = list.split(',').collect();
    if fields.len() > CELLS * stride || fields.len() % stride != 0 {
        return Err(invalid("invalid forging coordinate count"));
    }
    fields
        .iter()
        .map(|s| s.parse::<u16>().map_err(invalid))
        .collect()
}
fn number(s: &str) -> io::Result<u32> {
    let n = s.parse().map_err(invalid)?;
    if n > 1_000_000_000 {
        return Err(invalid("forging number too large"));
    }
    Ok(n)
}
fn boolean(s: &str) -> io::Result<bool> {
    match s {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(invalid("invalid forging boolean")),
    }
}
fn csv(v: &[u16]) -> String {
    v.iter().map(u16::to_string).collect::<Vec<_>>().join(",")
}
fn message_with_game(out: &mut ForgingOutcome, id: i32, s: &str, game_id: i32) -> io::Result<()> {
    out.replies.push(payload_packet_with_game(id, s, game_id)?);
    Ok(())
}
pub fn payload_packet(id: i32, s: &str) -> io::Result<Vec<u8>> {
    payload_packet_with_game(id, s, GAME)
}
fn payload_packet_with_game(id: i32, s: &str, game_id: i32) -> io::Result<Vec<u8>> {
    validate_text(s)?;
    let mut b = header_with_game(14, id, game_id);
    b.extend(((s.len() + 1) as i32).to_le_bytes());
    b.extend(s.as_bytes());
    b.push(0);
    Ok(b)
}
fn decode_payload(b: &[u8]) -> io::Result<&str> {
    if b.len() < 20 {
        return Err(invalid("truncated forging payload"));
    }
    let n = read_i32(b, 15);
    if n < 1 || n as usize > MAX_PAYLOAD || b.len() != 19 + n as usize || b.last() != Some(&0) {
        return Err(invalid("invalid forging payload length"));
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
        return Err(invalid("invalid forging text"));
    }
    Ok(())
}
#[cfg(test)]
fn header(sub: u8, id: i32) -> Vec<u8> {
    header_with_game(sub, id, GAME)
}
fn header_with_game(sub: u8, id: i32, game_id: i32) -> Vec<u8> {
    let mut b = GAME_FAMILY.to_le_bytes().to_vec();
    b.push(sub);
    for n in [id, -1, game_id] {
        b.extend(n.to_le_bytes());
    }
    b
}
const GAME_FAMILY: u16 = 39;
fn game_over(id: i32, won: bool, elapsed: u32, score: u32, game_id: i32) -> Vec<u8> {
    let mut b = header_with_game(18, id, game_id);
    b.push(u8::from(won));
    for n in [0_i32, elapsed as i32, score as i32] {
        b.extend(n.to_le_bytes());
    }
    b
}
fn results(a: &ForgingRun, won: bool, duration: u32, game_id: i32) -> Vec<Vec<u8>> {
    let elapsed = duration.saturating_mul(1000).saturating_sub(a.remaining_ms) / 1000;
    let mut scores = header_with_game(47, a.instance_id, game_id);
    scores.extend(1_i32.to_le_bytes());
    let name = b"scoreTotalScore";
    scores.extend((name.len() as i32).to_le_bytes());
    scores.extend(name);
    for n in [-1_i32, 4, -1, -1, a.score as i32] {
        scores.extend(n.to_le_bytes());
    }
    vec![
        game_over(a.instance_id, won, elapsed, a.score, game_id),
        scores,
        header_with_game(19, a.instance_id, game_id),
    ]
}
fn launch_packet() -> Vec<u8> {
    let mut b = [26_u16.to_le_bytes(), 12_u16.to_le_bytes()].concat();
    for s in ["MiniGameFlash", "game_forging.gfx"] {
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

#[cfg(test)]
mod ordinary_quit {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    // Geometry mirrors the synthetic fixture below (left/top 3, 8x8 field).
    const LEFT: usize = 3;
    const TOP: usize = 3;
    const WIDTH: usize = 8;
    const HEIGHT: usize = 8;
    const QUOTA: u32 = 1;

    struct Fixture {
        directory: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let directory = std::env::temp_dir().join(format!(
                "osfr-forging-quit-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&directory).unwrap();
            let content = serde_json::json!({"schema_version":1,"game_id":139,"profile_id":16,"name_id":501,"description_id":502,"icon_id":503,
                "reconstructed_rules":true,"rule_notice":"Explicit synthetic reconstructed test board, consumption and tint.",
                "board":{"left":LEFT,"top":TOP,"width":WIDTH,"height":HEIGHT,"duration_seconds":180,"quota":QUOTA,"selectable_types":[1,2,3,4,5],"collectable_percent":10,"seed":139,"mold_id":6},
                "recipe":{"key":"blacksmith.spunky_scrapper_hammer.sample","reconstructed_consumption":true,"reconstructed_tint_selection":true}});
            fs::write(
                directory.join("forging.json"),
                serde_json::to_vec(&content).unwrap(),
            )
            .unwrap();
            fs::write(
                directory.join("recipes.json"),
                include_str!("../data/content/recipes.json"),
            )
            .unwrap();
            Self { directory }
        }
        fn load(&self, state: Option<ForgingState>) -> Forging {
            Forging::load(&self.directory, state).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.directory.join("forging.json"));
            let _ = fs::remove_file(self.directory.join("recipes.json"));
            let _ = fs::remove_dir(&self.directory);
        }
    }

    fn request(sub: u8, instance: i32) -> Vec<u8> {
        let mut b = vec![39, 0, sub];
        for n in [instance, -1, -1] {
            b.extend(n.to_le_bytes());
        }
        b
    }
    fn quit(instance: i32) -> Vec<u8> {
        request(6, instance)
    }
    fn start(instance: i32) -> Vec<u8> {
        request(5, instance)
    }
    fn hex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .chunks_exact(2)
            .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
            .collect()
    }
    fn finished_board() -> Vec<u16> {
        let mut board = vec![0_u16; CELLS];
        for y in 0..SIDE {
            for x in 0..SIDE {
                board[y * SIDE + x] =
                    if x >= LEFT && x < LEFT + WIDTH && y >= TOP && y < TOP + HEIGHT {
                        401
                    } else if y == TOP + HEIGHT && x >= LEFT && x < LEFT + WIDTH {
                        100
                    } else {
                        0
                    };
            }
        }
        board
    }
    fn won_state() -> ForgingState {
        let board = finished_board();
        ForgingState {
            version: 1,
            next_instance_id: FIRST_INSTANCE + 1,
            completed: 1,
            active_recipe_key: Some(default_recipe_key()),
            active: Some(ForgingRun {
                instance_id: FIRST_INSTANCE,
                phase: ForgingPhase::Won,
                idle: vec![true; board.len()],
                board,
                tile_queue: vec![401; 50],
                rng: 1,
                collected: QUOTA,
                score: 40,
                remaining_ms: 5_000,
                wall_ms: 20_000,
                paused: true,
                chains: 2,
                reshuffles: 0,
                end_sent: true,
            }),
        }
    }
    fn lost_state() -> ForgingState {
        ForgingState {
            version: 1,
            next_instance_id: FIRST_INSTANCE + 1,
            completed: 0,
            active_recipe_key: Some(default_recipe_key()),
            active: Some(ForgingRun {
                instance_id: FIRST_INSTANCE,
                phase: ForgingPhase::Lost,
                board: Vec::new(),
                idle: Vec::new(),
                tile_queue: Vec::new(),
                rng: 1,
                collected: 0,
                score: 0,
                remaining_ms: 0,
                wall_ms: 180_000,
                paused: false,
                chains: 0,
                reshuffles: 0,
                end_sent: true,
            }),
        }
    }
    fn active_instance(game: &Forging) -> i32 {
        game.state().active.as_ref().unwrap().instance_id
    }

    #[test]
    fn native_ordinary_quit_cancels_pre_win_runs_with_the_compiled_vector() {
        let fixture = Fixture::new();
        let mut game = fixture.load(None);
        game.begin(PROFILE).unwrap();
        let instance = active_instance(&game);
        assert_eq!(instance, 139_000_001);
        let packet = quit(instance);
        // Pinned unchanged community BaseMiniGamePacket writer for the native
        // Ui.QuitMiniGame constructor: header, sub6, current instance, -1, -1.
        assert_eq!(packet, hex("270006c1f84808ffffffffffffffff"));
        let offered = game.handle(&packet).unwrap();
        assert_eq!(offered.replies, vec![header(19, instance)]);
        assert!(offered.changed);
        assert!(offered.transaction.is_none());
        assert!(!offered.completed);
        assert!(game.state().active.is_none());

        game.begin(PROFILE).unwrap();
        let playing_instance = active_instance(&game);
        assert_eq!(playing_instance, 139_000_002);
        assert_eq!(
            game.handle(&start(playing_instance)).unwrap().replies.len(),
            2
        );
        assert_eq!(
            game.state().active.as_ref().unwrap().phase,
            ForgingPhase::Launched
        );
        let played = game.handle(&quit(playing_instance)).unwrap();
        assert_eq!(played.replies, vec![header(19, playing_instance)]);
        assert!(played.transaction.is_none());
        assert!(game.state().active.is_none());
    }

    #[test]
    fn native_ordinary_quit_rejects_malformed_stale_and_foreign_packets() {
        let fixture = Fixture::new();
        let mut game = fixture.load(None);
        let inactive = game.state().clone();
        let valid = quit(FIRST_INSTANCE);
        for malformed in [valid[..14].to_vec(), [valid.clone(), vec![0]].concat()] {
            assert!(game.handle(&malformed).is_err());
            assert_eq!(game.state(), &inactive);
        }
        assert!(game.handle(&valid).unwrap().replies.is_empty());
        assert_eq!(game.state(), &inactive);

        game.begin(PROFILE).unwrap();
        let offered = game.state().clone();
        assert!(game.handle(&valid[..14]).is_err());
        assert!(game.handle(&[valid.clone(), vec![0]].concat()).is_err());
        assert_eq!(game.state(), &offered);
        for (offset, value) in [
            (3, FIRST_INSTANCE + 1),
            (7, 0),
            (7, 46),
            (11, 0),
            (11, GAME),
            (11, -2),
        ] {
            let mut foreign = valid.clone();
            foreign[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            let out = game.handle(&foreign).unwrap();
            assert!(out.replies.is_empty(), "offset {offset} value {value}");
            assert_eq!(game.state(), &offered);
        }
        let cancelled = game.handle(&valid).unwrap();
        assert_eq!(cancelled.replies, vec![header(19, FIRST_INSTANCE)]);
        assert!(game.state().active.is_none());
    }

    #[test]
    fn native_ordinary_quit_is_idempotent_and_distinct_from_ack_and_results_close() {
        let fixture = Fixture::new();
        let mut game = fixture.load(None);
        game.begin(PROFILE).unwrap();
        let packet = quit(FIRST_INSTANCE);
        assert_eq!(
            game.handle(&packet).unwrap().replies,
            vec![header(19, FIRST_INSTANCE)]
        );
        let cancelled = game.state().clone();
        let replay = game.handle(&packet).unwrap();
        assert!(replay.replies.is_empty());
        assert!(!replay.changed);
        assert_eq!(game.state(), &cancelled);
        let ack_out = game.handle(&request(7, FIRST_INSTANCE)).unwrap();
        assert!(ack_out.replies.is_empty());
        assert_eq!(game.state(), &cancelled);
        assert!(
            game.handle(&payload_packet(FIRST_INSTANCE, "FRServer_GameClose").unwrap())
                .unwrap()
                .replies
                .is_empty()
        );
        let mut live = fixture.load(None);
        live.begin(PROFILE).unwrap();
        let offered = live.state().clone();
        assert!(
            live.handle(&request(7, FIRST_INSTANCE))
                .unwrap()
                .replies
                .is_empty()
        );
        assert_eq!(live.state(), &offered);

        for state in [won_state(), lost_state()] {
            let mut terminal = fixture.load(Some(state));
            let before = terminal.state().clone();
            let refused = terminal.handle(&packet).unwrap();
            assert!(refused.replies.is_empty());
            assert!(!refused.changed);
            assert_eq!(terminal.state(), &before);
            let closed = terminal
                .handle(&payload_packet(FIRST_INSTANCE, "FRServer_GameClose").unwrap())
                .unwrap();
            assert_eq!(closed.replies, vec![header(19, FIRST_INSTANCE)]);
            assert!(terminal.state().active.is_none());
        }
    }

    #[test]
    fn native_ordinary_quit_preserves_cancellation_economics_and_once_only_state() {
        let fixture = Fixture::new();
        let prior = ForgingState {
            version: 1,
            next_instance_id: FIRST_INSTANCE + 1,
            completed: 1,
            active_recipe_key: None,
            active: None,
        };
        let mut game = fixture.load(Some(prior));
        let requirements = game.requirements().to_vec();
        game.begin(PROFILE).unwrap();
        let instance = active_instance(&game);
        assert_eq!(instance, FIRST_INSTANCE + 1);
        let out = game.handle(&quit(instance)).unwrap();
        assert!(out.transaction.is_none());
        assert!(!out.completed);
        assert!(out.changed);
        assert_eq!(game.state().completed, 1);
        assert_eq!(game.requirements(), requirements.as_slice());
        assert!(game.state().active.is_none());
        game.begin(PROFILE).unwrap();
        assert_eq!(active_instance(&game), instance + 1);
    }
}
