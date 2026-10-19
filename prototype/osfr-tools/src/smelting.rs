//! Original smelting GFx protocol with explicitly reconstructed phase tuning.
//!
//! The caller must stage this clonable service together with inventory, apply
//! `transaction` atomically, and persist before committing either state. The
//! movie reports its own play; these checks are not an authoritative simulation.
use crate::recipes::{CraftingJob, RecipeCatalog, RecipeFile};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

const GAME: i32 = 77;
const PROFILE: i32 = 14;
const MAX_CONTENT: u64 = 32 * 1024;
const MAX_PAYLOAD: usize = 4096;
const FIRST_INSTANCE: i32 = 77_000_001;

#[derive(Clone, Debug, Deserialize)]
struct Content {
    schema_version: u32,
    game_id: i32,
    profile_id: i32,
    name_id: i32,
    description_id: i32,
    icon_id: i32,
    reconstructed_phase_values: bool,
    phase_notice: String,
    phases: Vec<Phase>,
    recipe: RecipeSelector,
}

/// The 19 caret-separated columns read by SmeltingGameServer.OnLevelData.
#[derive(Clone, Debug, Deserialize)]
struct Phase {
    id: i32,
    sequence: i32,
    duration: i32,
    objective_id: i32,
    start_id: i32,
    end_id: i32,
    stage_type: i32,
    data: [i32; 11],
}
impl Phase {
    fn row(&self, game_id: i32) -> String {
        let mut values = vec![
            self.id,
            game_id,
            self.sequence,
            self.duration,
            self.objective_id,
            self.start_id,
            self.end_id,
            self.stage_type,
        ];
        values.extend(self.data);
        values
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join("^")
            + "^"
    }
}

#[derive(Clone, Debug, Deserialize)]
struct RecipeSelector {
    #[serde(default = "default_recipe_key")]
    key: String,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SmeltingState {
    pub version: u32,
    pub next_instance_id: i32,
    pub completed: u32,
    #[serde(default)]
    pub active_recipe_key: Option<String>,
    pub active: Option<SmeltingRun>,
}
impl Default for SmeltingState {
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
pub enum SmeltingPhase {
    Offered,
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SmeltingRun {
    pub instance_id: i32,
    pub phase: SmeltingPhase,
    pub data_sent: bool,
    pub round_stats: [[Option<f64>; 4]; 4],
    pub game_stats: [Option<f64>; 5],
    pub score: u32,
}

/// Per-job alias of the shared completion transaction (`recipe_book::CraftTransaction`).
pub type SmeltingTransaction = crate::recipe_book::CraftTransaction;

/// Per-job alias of the shared completion record (`recipe_book::CraftOutcome`).
pub type SmeltingOutcome = crate::recipe_book::CraftOutcome;

#[derive(Clone, Debug)]
pub struct Smelting {
    content: Arc<Content>,
    recipes: HashMap<String, Recipe>,
    state: SmeltingState,
    results_presented: bool,
}

impl Smelting {
    pub fn load(directory: &Path, state: Option<SmeltingState>) -> io::Result<Self> {
        let path = directory.join("smelting.json");
        if fs::metadata(&path)?.len() > MAX_CONTENT {
            return Err(invalid("smelting content too large"));
        }
        let mut bytes = Vec::new();
        fs::File::open(&path)?
            .take(MAX_CONTENT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_CONTENT {
            return Err(invalid("smelting content too large"));
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
            return Err(invalid("saved smelting recipe is unavailable"));
        }
        Ok(Self {
            content: Arc::new(content),
            recipes,
            state,
            results_presented: false,
        })
    }
    pub fn state(&self) -> &SmeltingState {
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

    pub fn validate_state(state: &SmeltingState) -> io::Result<()> {
        if state.version != 1
            || state.next_instance_id < FIRST_INSTANCE
            || state.completed > (state.next_instance_id - FIRST_INSTANCE) as u32
            || state
                .active_recipe_key
                .as_ref()
                .is_some_and(String::is_empty)
        {
            return Err(invalid("invalid smelting state"));
        }
        if let Some(a) = &state.active {
            if a.instance_id < FIRST_INSTANCE
                || a.instance_id >= state.next_instance_id
                || a.score > 1_000_000_000
                || a.round_stats
                    .iter()
                    .flatten()
                    .chain(a.game_stats.iter())
                    .flatten()
                    .any(|n| !n.is_finite() || *n < 0.0 || *n > 1_000_000_000.0)
                || (!a.data_sent && a.round_stats.iter().flatten().any(Option::is_some))
                || (a.phase == SmeltingPhase::Offered && a.data_sent)
            {
                return Err(invalid("invalid saved smelting run"));
            }
            let mut unfinished = false;
            for round in &a.round_stats {
                for (kind, value) in round.iter().enumerate() {
                    if let Some(value) = value {
                        if (kind == 3 && *value > 3600.0) || (kind != 3 && value.fract() != 0.0) {
                            return Err(invalid("invalid saved smelting round statistic"));
                        }
                    }
                }
                if unfinished && round.iter().any(Option::is_some) {
                    return Err(invalid("unordered saved smelting rounds"));
                }
                unfinished |= round.iter().any(Option::is_none);
            }
            for (kind, value) in a.game_stats.iter().enumerate() {
                if let Some(value) = value {
                    if unfinished
                        || (kind != 0 && value.fract() != 0.0)
                        || (kind == 4 && *value > 10.0)
                    {
                        return Err(invalid("invalid saved final smelting statistic"));
                    }
                }
            }
            if a.phase == SmeltingPhase::Won && (!a.data_sent || unfinished || state.completed == 0)
            {
                return Err(invalid("invalid saved smelting victory"));
            }
        }
        Ok(())
    }

    /// (R) Compatibility entry for the default recipe; the gateway must use
    /// `begin_recipe` with learned state, rank, and actual inventory.
    pub fn begin(&mut self, profile: i32) -> io::Result<SmeltingOutcome> {
        let key = self
            .state
            .active_recipe_key
            .as_deref()
            .unwrap_or(&self.content.recipe.key)
            .to_owned();
        let available = self
            .recipes
            .get(&key)
            .ok_or_else(|| invalid("unknown smelting recipe"))?
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
    ) -> io::Result<SmeltingOutcome> {
        if profile != PROFILE || self.active() {
            return Err(invalid("smelting requires an idle Miner"));
        }
        let recipe = self
            .recipes
            .get(recipe_key)
            .ok_or_else(|| invalid("unknown smelting recipe"))?;
        if !learned
            || recipe.job_rank.is_some_and(|required| job_rank < required)
            || !has_ingredients(&recipe.consumes, available)
        {
            return Err(invalid(
                "smelting recipe is locked or ingredients are missing",
            ));
        }
        let id = self.state.next_instance_id;
        let next = id
            .checked_add(1)
            .ok_or_else(|| invalid("smelting instance IDs exhausted"))?;
        self.state.next_instance_id = next;
        self.state.active_recipe_key = Some(recipe_key.to_owned());
        self.state.active = Some(SmeltingRun {
            instance_id: id,
            phase: SmeltingPhase::Offered,
            data_sent: false,
            round_stats: [[None; 4]; 4],
            game_stats: [None; 5],
            score: 0,
        });
        self.results_presented = false;
        Ok(SmeltingOutcome {
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

    pub fn cancel(&mut self) -> SmeltingOutcome {
        let Some(a) = self.state.active.take() else {
            return SmeltingOutcome::default();
        };
        let replies = if self.results_presented {
            vec![]
        } else {
            vec![header_with_game(19, a.instance_id, self.selected_game_id())]
        };
        self.results_presented = false;
        SmeltingOutcome {
            replies,
            changed: true,
            ..Default::default()
        }
    }

    pub fn handle(&mut self, inner: &[u8]) -> io::Result<SmeltingOutcome> {
        if inner.len() < 2 || inner[..2] != 39_u16.to_le_bytes() {
            return Ok(SmeltingOutcome::default());
        }
        if inner.len() < 15 {
            return Err(invalid("truncated minigame header"));
        }
        let sub = inner[2];
        if !matches!(sub, 5 | 6 | 7 | 14) {
            return Ok(SmeltingOutcome::default());
        }
        let id = read_i32(inner, 3);
        let group = read_i32(inner, 7);
        let game = read_i32(inner, 11);
        // Parse first even when inactive; malformed recognized packets are rejected.
        let message = if sub == 14 {
            Some(decode_payload(inner)?)
        } else {
            if inner.len() != 15 {
                return Err(invalid("malformed minigame start request"));
            }
            None
        };
        let Some(a) = &self.state.active else {
            return Ok(SmeltingOutcome::default());
        };
        if sub == 7 {
            // Removal ACK is not the actual native results Close26/42.
            return Ok(SmeltingOutcome::default());
        }
        if sub == 6 {
            // Shared original Ui.QuitMiniGame00c17470 ->00939100 ->009be8e0 ->
            //009bdc60(6,-1,-1) serializes the current instance with both
            // sentinels. Ordinary Quit cancels only a pre-win run; terminal
            // Won/Lost state and economy stay until the actual results
            // Close26/42, and no transaction is created.
            if !matches!(a.phase, SmeltingPhase::Offered | SmeltingPhase::Playing)
                || id != a.instance_id
                || group != -1
                || game != -1
            {
                return Ok(SmeltingOutcome::default());
            }
            return Ok(self.cancel());
        }
        let game_id = self.selected_game_id();
        // Shared original RequestStartGame 009be7d0 -> 009bdc60 uses
        // GameId=-1 for the current offered instance, independent of game type.
        let native_offer_start = sub == 5 && a.phase == SmeltingPhase::Offered && game == -1;
        // Native shared SendPayload constructor009b09d0 keeps both sentinels.
        let native_payload = sub == 14 && a.phase != SmeltingPhase::Offered && game == -1;
        if id != a.instance_id
            || group != -1
            || (game != game_id && !native_offer_start && !native_payload)
        {
            return Ok(SmeltingOutcome::default());
        }
        if sub == 5 {
            let launch = a.phase == SmeltingPhase::Offered;
            let mut replies = vec![header_with_game(17, id, game_id)];
            if launch {
                // Native start acknowledgement only changes the state/HUD.
                // Command26/12 is what invokes MiniGameFlash:Show with the GFx.
                replies.push(launch_packet());
                self.state.active.as_mut().unwrap().phase = SmeltingPhase::Playing;
            }
            return Ok(SmeltingOutcome {
                replies,
                changed: launch,
                ..Default::default()
            });
        }
        // Clone and commit only after the complete message has been validated.
        let mut next = self.state.clone();
        let a = next.active.as_mut().unwrap();
        let mut out = SmeltingOutcome::default();
        let fields: Vec<_> = message.unwrap().split('\t').collect();
        match fields.as_slice() {
            ["OnConnectMsg"] | ["FRServer_Connect"] => {}
            ["FRServer_ReadyForGameData"] => {
                if matches!(a.phase, SmeltingPhase::Won | SmeltingPhase::Lost) {
                    return Ok(out);
                }
                // Reliable replies already retransmit. Re-appending phase
                // data to a running movie would reset its local game instance.
                if a.data_sent {
                    return Ok(out);
                }
                let mut messages = vec![format!("OnGameDataMsg\t{game_id}")];
                messages.extend(
                    self.content
                        .phases
                        .iter()
                        .map(|p| format!("OnPhaseDataMsg\t{}", p.row(game_id))),
                );
                messages.push("OnDataComplete".into());
                out.replies = messages
                    .iter()
                    .map(|s| payload_packet_with_game(id, s, game_id))
                    .collect::<io::Result<_>>()?;
                a.data_sent = true;
                a.phase = SmeltingPhase::Playing;
            }
            ["FRServer_StatUpdate", stage, kind, value] => {
                if a.phase != SmeltingPhase::Playing || !a.data_sent {
                    return Err(invalid("smelting statistics before data"));
                }
                let stage: i32 = stage.parse().map_err(invalid)?;
                let kind: usize = kind.parse().map_err(invalid)?;
                // Same original malformed FScommand as Cooking: the native
                // bridge retains one ')' on the final numeric field only.
                let value = value.strip_suffix(')').unwrap_or(value);
                let value: f64 = value.parse().map_err(invalid)?;
                if !value.is_finite() || !(0.0..=1_000_000_000.0).contains(&value) {
                    return Err(invalid("invalid smelting statistic"));
                }
                if (101..=104).contains(&stage) && kind < 4 {
                    let round = (stage - 101) as usize;
                    if a.round_stats[..round]
                        .iter()
                        .any(|r| r.iter().any(Option::is_none))
                        || (kind != 3 && value.fract() != 0.0)
                        // The supplied TorchTimer is a stopwatch. This is a
                        // local defensive bound, not the nominal row duration.
                        || (kind == 3 && value > 3600.0)
                    {
                        return Err(invalid("unordered or invalid smelting round"));
                    }
                    a.round_stats[round][kind] = Some(value);
                } else if stage == -1 && (4..=8).contains(&kind) {
                    if a.round_stats.iter().any(|r| r.iter().any(Option::is_none))
                        || (kind != 4 && value.fract() != 0.0)
                        || (kind == 8 && value > 10.0)
                    {
                        return Err(invalid("invalid final smelting statistics"));
                    }
                    a.game_stats[kind - 4] = Some(value);
                } else {
                    return Err(invalid("unknown smelting statistic"));
                }
            }
            ["FRServer_ScoreSet", score] => {
                let score = parse_score(score)?;
                if matches!(a.phase, SmeltingPhase::Won | SmeltingPhase::Lost) {
                    return Ok(out);
                }
                a.score = score;
            }
            ["FRServer_GameWon", score, quantity] => {
                let score = parse_score(score)?;
                let quantity: u32 = quantity.parse().map_err(invalid)?;
                if quantity > 10 {
                    return Err(invalid("invalid reported smelting quantity"));
                }
                if a.phase == SmeltingPhase::Won {
                    return Ok(out);
                }
                if a.phase != SmeltingPhase::Playing
                    || a.round_stats.iter().any(|r| r.iter().any(Option::is_none))
                {
                    return Err(invalid("smelting victory before four rounds"));
                }
                a.phase = SmeltingPhase::Won;
                a.score = score;
                next.completed = next
                    .completed
                    .checked_add(1)
                    .ok_or_else(|| invalid("smelting count overflow"))?;
                let recipe = self.selected_recipe();
                out.transaction = Some(SmeltingTransaction {
                    consumes: recipe.consumes.clone(),
                    grants: vec![(recipe.product_definition_id, recipe.product_quantity)],
                    experience_points: recipe.experience_points,
                });
                out.completed = true;
                out.replies = vec![payload_packet_with_game(id, "OnGameWonMsg", game_id)?];
                out.replies.extend(results(a, true, game_id));
                self.results_presented = true;
            }
            ["FRServer_GameLost", score] | ["FRServer_GameQuit", score] => {
                // SoeInterface.GameLost() drops score before forwarding in
                // this movie, resulting in AS2's literal "undefined".
                if *score != "undefined" {
                    parse_score(score)?;
                }
                if matches!(a.phase, SmeltingPhase::Won | SmeltingPhase::Lost) {
                    return Ok(out);
                }
                a.phase = SmeltingPhase::Lost;
                out.replies = vec![payload_packet_with_game(id, "OnGameLostMsg", game_id)?];
                out.replies.extend(results(a, false, game_id));
                self.results_presented = true;
            }
            ["FRServer_GameClose"] => {
                return Ok(self.cancel());
            }
            ["FRServer_GameOver"] | ["OnStartGameMsg"] | ["FRServer_CompleteLevel"] => {}
            ["FRServer_Pause", value] | ["FRServer_RoundTransitionPause", value] => {
                if !matches!(*value, "true" | "false") {
                    return Err(invalid("invalid smelting pause"));
                }
            }
            ["FRServer_StopLevel", level, won] => {
                let won = won.strip_suffix(')').unwrap_or(won);
                if *level != "4" || !matches!(won, "0" | "1") {
                    return Err(invalid("invalid smelting stop"));
                }
            }
            _ => {
                return Err(invalid("unsupported smelting payload"));
            }
        }
        out.changed = self.state != next;
        self.state = next;
        Ok(out)
    }

    fn info_packet(&self, id: i32) -> Vec<u8> {
        // Shared MiniGameInfo format: native FUN_009bdd70, corroborated PR76.
        let c = &self.content;
        let game_id = self.selected_game_id();
        let mut b = header_with_game(16, id, game_id);
        for n in [c.name_id, c.icon_id, c.description_id, 1, PROFILE, 9] {
            b.extend(n.to_le_bytes());
        }
        b.push(0); // MembersOnly.
        b.extend([0; 207]); // Three empty 69-byte RewardBundleBase records.
        b.extend(0_i32.to_le_bytes()); // No inline objective rows.
        b.extend([1; 5]);
        // Native FUN_009beb70 passes this field to the asset-list preloader.
        let preload = b"SmeltingAssets.txt";
        b.extend((preload.len() as i32).to_le_bytes());
        b.extend(preload);
        b.extend(1_i32.to_le_bytes());
        b.push(1);
        b.extend(game_id.to_le_bytes()); // PreselectedGameId.
        b.extend([0; 4]);
        b.extend(0_i32.to_le_bytes()); // ActivityId not recovered; no invented ID.
        b.extend(0_i32.to_le_bytes()); // MiniGameInfoPacket trailing field.
        b
    }
}

fn validate_content(c: &Content, recipes: &HashMap<String, Recipe>) -> io::Result<()> {
    if c.schema_version != 1
        || c.game_id != GAME
        || c.profile_id != PROFILE
        || c.name_id <= 0
        || c.description_id <= 0
        || c.icon_id <= 0
        || !c.reconstructed_phase_values
        || c.phase_notice.is_empty()
        || c.phase_notice.len() > 1024
        || c.phases.len() != 4
        || !recipes.contains_key(&c.recipe.key)
    {
        return Err(invalid("invalid or unlabelled smelting content"));
    }
    for (i, p) in c.phases.iter().enumerate() {
        if p.id <= 0
            || p.sequence != i as i32 + 1
            || p.stage_type != i as i32 + 101
            || !(10..=600).contains(&p.duration)
            || p.objective_id < 0
            || p.start_id < 0
            || p.end_id < 0
            || p.data.iter().any(|v| !(0..=50_000).contains(v))
        {
            return Err(invalid("invalid smelting phase row"));
        }
    }
    let d = c.phases[0].data;
    if d[0] != 1
        || !(1..=5).contains(&d[1])
        || !(1..=10).contains(&d[2])
        || !(1..=10).contains(&d[3])
        || d[4] > 300
        || !(10..=100).contains(&d[5])
        || !(1..=5).contains(&d[6])
        || !(1..=5).contains(&d[7])
        || !(50..=10_000).contains(&d[8])
        || d[9] < 2 * d[8]
        || d[9] > 10_000
        || d[10] < 2 * d[9]
        || d[10] > 10_000
    {
        return Err(invalid("unsafe ore-breaking phase"));
    }
    let d = c.phases[1].data;
    if !(1..=100).contains(&d[0])
        || !(1..=25).contains(&d[1])
        || d[2] > d[1]
        || !(100..=50_000).contains(&d[3])
        || d[4] < d[3]
    {
        return Err(invalid("unsafe measuring phase"));
    }
    let d = c.phases[2].data;
    if !(1..=1000).contains(&d[0]) || d[1] > 100 || !(500..=50_000).contains(&d[2]) {
        return Err(invalid("unsafe furnace phase"));
    }
    let d = c.phases[3].data;
    if !(1..=100).contains(&d[0]) || !(500..=50_000).contains(&d[1]) {
        return Err(invalid("unsafe molding phase"));
    }
    Ok(())
}

/// Native SendMessage passes string length + 1 (x86 00c08519), including NUL.
pub fn payload_packet(instance_id: i32, message: &str) -> io::Result<Vec<u8>> {
    payload_packet_with_game(instance_id, message, GAME)
}

fn payload_packet_with_game(instance_id: i32, message: &str, game_id: i32) -> io::Result<Vec<u8>> {
    validate_text(message)?;
    let mut b = header_with_game(14, instance_id, game_id);
    b.extend(((message.len() + 1) as i32).to_le_bytes());
    b.extend(message.as_bytes());
    b.push(0);
    Ok(b)
}
fn decode_payload(inner: &[u8]) -> io::Result<&str> {
    if inner.len() < 20 {
        return Err(invalid("truncated smelting payload"));
    }
    let length = read_i32(inner, 15);
    if length < 1
        || length as usize > MAX_PAYLOAD
        || inner.len() != 19 + length as usize
        || *inner.last().unwrap() != 0
    {
        return Err(invalid("invalid smelting payload length"));
    }
    let text = std::str::from_utf8(&inner[19..inner.len() - 1]).map_err(invalid)?;
    validate_text(text)?;
    Ok(text)
}
fn validate_text(s: &str) -> io::Result<()> {
    if s.is_empty()
        || s.len() + 1 > MAX_PAYLOAD
        || !s.bytes().all(|b| b == b'\t' || (32..=126).contains(&b))
    {
        return Err(invalid("invalid smelting message text"));
    }
    Ok(())
}
#[cfg(test)]
fn header(sub: u8, id: i32) -> Vec<u8> {
    header_with_game(sub, id, GAME)
}
fn game_over(id: i32, won: bool, elapsed: u32, score: u32, game_id: i32) -> Vec<u8> {
    let mut b = header_with_game(18, id, game_id);
    b.push(u8::from(won));
    for n in [0_i32, elapsed as i32, score as i32] {
        b.extend(n.to_le_bytes());
    }
    b
}
fn results(a: &SmeltingRun, won: bool, game_id: i32) -> Vec<Vec<u8>> {
    let elapsed = a.game_stats[0]
        .unwrap_or_else(|| a.round_stats.iter().filter_map(|r| r[3]).sum())
        .floor() as u32;
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

fn default_recipe_key() -> String {
    "miner.copper_bar.sample".into()
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
    for row in catalog.by_job(CraftingJob::Miner) {
        if row.minigame_type_id != 9 {
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
        return Err(invalid("recipe table has no Miner smelting recipes"));
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

fn header_with_game(sub: u8, id: i32, game_id: i32) -> Vec<u8> {
    let mut b = 39_u16.to_le_bytes().to_vec();
    b.push(sub);
    for n in [id, -1, game_id] {
        b.extend(n.to_le_bytes());
    }
    b
}
fn launch_packet() -> Vec<u8> {
    // Native FUN_00a9d180: two LP strings and a bool after BaseCommand.
    // FUN_009bd650 invokes <handler>:Show(movie, title, icon), using MiniGameInfo.
    let mut b = [26_u16.to_le_bytes(), 12_u16.to_le_bytes()].concat();
    for s in ["MiniGameFlash", "game_smelting.gfx"] {
        b.extend((s.len() as i32).to_le_bytes());
        b.extend(s.as_bytes());
    }
    b.push(0);
    b
}
fn read_i32(b: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(b[offset..offset + 4].try_into().unwrap())
}
fn parse_score(s: &str) -> io::Result<u32> {
    let score = s.parse().map_err(invalid)?;
    if score > 1_000_000_000 {
        return Err(invalid("smelting score too large"));
    }
    Ok(score)
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

    struct Fixture {
        path: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "osfr-smelting-quit-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            let data = [
                [1, 5, 1, 1, 0, 20, 2, 3, 50, 100, 200],
                [65, 8, 3, 100, 100, 0, 0, 0, 0, 0, 0],
                [100, 1, 500, 0, 0, 0, 0, 0, 0, 0, 0],
                [2, 500, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ];
            let content = serde_json::json!({"schema_version":1,"game_id":77,"profile_id":14,
                "name_id":501,"description_id":502,"icon_id":503,
                "reconstructed_phase_values":true,"phase_notice":"synthetic explicitly reconstructed test configuration",
                "phases":data.iter().enumerate().map(|(i,d)| serde_json::json!({"id":i+1,"sequence":i+1,"duration":90,
                    "objective_id":0,"start_id":0,"end_id":0,"stage_type":101+i,"data":d})).collect::<Vec<_>>(),
                "recipe":{"key":"miner.copper_bar.sample"}});
            fs::write(
                path.join("smelting.json"),
                serde_json::to_vec(&content).unwrap(),
            )
            .unwrap();
            fs::write(
                path.join("recipes.json"),
                include_str!("../data/content/recipes.json"),
            )
            .unwrap();
            Self { path }
        }
        fn load(&self, state: Option<SmeltingState>) -> Smelting {
            Smelting::load(&self.path, state).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.path.join("smelting.json"));
            let _ = fs::remove_file(self.path.join("recipes.json"));
            let _ = fs::remove_dir(&self.path);
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
    fn won_state() -> SmeltingState {
        SmeltingState {
            version: 1,
            next_instance_id: FIRST_INSTANCE + 1,
            completed: 1,
            active_recipe_key: Some(default_recipe_key()),
            active: Some(SmeltingRun {
                instance_id: FIRST_INSTANCE,
                phase: SmeltingPhase::Won,
                data_sent: true,
                round_stats: [[Some(0.0), Some(0.0), Some(0.0), Some(1.0)]; 4],
                game_stats: [Some(4.0), Some(0.0), Some(0.0), Some(0.0), Some(0.0)],
                score: 900,
            }),
        }
    }
    fn lost_state() -> SmeltingState {
        SmeltingState {
            version: 1,
            next_instance_id: FIRST_INSTANCE + 1,
            completed: 0,
            active_recipe_key: Some(default_recipe_key()),
            active: Some(SmeltingRun {
                instance_id: FIRST_INSTANCE,
                phase: SmeltingPhase::Lost,
                data_sent: true,
                round_stats: [
                    [Some(0.0), None, None, Some(2.0)],
                    [None; 4],
                    [None; 4],
                    [None; 4],
                ],
                game_stats: [None; 5],
                score: 0,
            }),
        }
    }
    fn active_instance(game: &Smelting) -> i32 {
        game.state().active.as_ref().unwrap().instance_id
    }

    #[test]
    fn native_ordinary_quit_cancels_pre_win_runs_with_the_compiled_vector() {
        let fixture = Fixture::new();
        let mut game = fixture.load(None);
        game.begin(PROFILE).unwrap();
        let instance = active_instance(&game);
        assert_eq!(instance, 77_000_001);
        let packet = quit(instance);
        // Pinned unchanged community BaseMiniGamePacket writer for the native
        // Ui.QuitMiniGame constructor: header, sub6, current instance, -1, -1.
        assert_eq!(packet, hex("27000641ed9604ffffffffffffffff"));
        let offered = game.handle(&packet).unwrap();
        assert_eq!(offered.replies, vec![header(19, instance)]);
        assert!(offered.changed);
        assert!(offered.transaction.is_none());
        assert!(!offered.completed);
        assert!(game.state().active.is_none());

        game.begin(PROFILE).unwrap();
        let playing_instance = active_instance(&game);
        assert_eq!(playing_instance, 77_000_002);
        assert_eq!(
            game.handle(&start(playing_instance)).unwrap().replies.len(),
            2
        );
        assert_eq!(
            game.state().active.as_ref().unwrap().phase,
            SmeltingPhase::Playing
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
        let playing = live.state().clone();
        assert!(
            live.handle(&request(7, FIRST_INSTANCE))
                .unwrap()
                .replies
                .is_empty()
        );
        assert_eq!(live.state(), &playing);

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
        let prior = SmeltingState {
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
