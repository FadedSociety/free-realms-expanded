//! L962: Tower Defense, client minigame type 14 (MiniGameData 150-161, 250-261 and 350-361; Games window
//! rows 195-206, 520-540 and 549-551): the movie `towerdefense.gfx`, refereed by the server.
//!
//! Recovered from the movie's own scripts (JPEXS export in the lab; protocol note L962):
//! - The movie's GameServer runs inside the client (`Game.networking` is `SoeNetwork.TYPE_LOCAL`), so the wire
//!   carries `FRApi`'s tab-separated text, the framework cooking and music speak.
//! - Server to client: `OnGameDataMsg <game>`, the map row `OnMapDataMsg`, a row per tower rank
//!   `OnTowerDataMsg` (the map's towers in order, each one's ranks in turn: the movie builds its buttons in
//!   arrival order), the enemies `OnEnemyDataMsg` (its waves cycle through them in order), then
//!   `OnGameDataCompleteMsg`, which starts the game. Every row keeps the Free Realms trailing `^`
//!   (`Game.OnMapDataMsg` chops the last character before it splits).
//! - Client to server: `FRServer_CompletedStage <n>` from 0 as the first wave starts to STAGES just before
//!   `FRServer_GameWon <score>` (`TD_Game.nextLevel`), `FRServer_LivesCount`, `FRServer_ScoreSet`, the tower,
//!   money, enemy and power-up reports, `FRServer_GameLost <score>` at no lives, then `FRServer_GameOver`.
//!
//! Reconstructed (R): the level tables were the live server's and never shipped; `tower_defense.json` resolves
//! Sanctuary's authored tables (BubbaJoeX fork, AGPL-3.0, credited) per game. A win needs every stage reported
//! in order and lives left, else the server settles a loss; the score is the client's (quest credit stays
//! id-only); a win also needs `min_stage_ms` per stage since the data went out (the next-wave button re-arms
//! only once a wave is cleared, `TD_Game.as` 1002-1015, so a real game cannot be faster); a win pays the
//! content's `win_coins`.
use crate::{
    minigame::{
        self, Activity, ActivityFinish, Goal, MinigameRuns, RunMode, RunOutcome, RunSpec, RunState,
    },
    npc_interaction::{TableMenu, TableMenus},
    rewards::{RewardDisplay, RewardGrant, RewardState},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

pub const GAME_TYPE: i32 = 14;
pub const MOVIE: &str = "towerdefense.gfx";
pub const CONTENT_FILE: &str = "tower_defense.json";
/// The client's preload list for the movie (manifest `TowerDefenseAssets.txt.z`). Type 14 is not one the client
/// readies by itself (`009beb70`: 1, 2, 17, 22), so without it the start screen stays on "Loading..." (L918).
pub const PRELOAD_FILE: &str = "TowerDefenseAssets.txt";
/// (R) Run ids of their own, apart from every other minigame's.
pub const FIRST_INSTANCE_ID: i32 = 150_000_001;
const MAX_CONTENT: u64 = 1024 * 1024;
const MAX_GAMES: usize = 64;
const MAX_TEXT: usize = 512;
/// `Game.MAP_*`: the movie's map row has 25 columns (ID to ENEMY_HP_MULTI).
const MAP_COLUMNS: usize = 25;
const MAP_GAME_ID: usize = 1;
const MAP_LIVES: usize = 7;
const MAP_STAGES: usize = 8;
/// `Game.TOWER_*` and `Game.ENEMY_*` row widths.
const TOWER_COLUMNS: usize = 22;
const ENEMY_COLUMNS: usize = 11;
const MAX_STAGES: u32 = 999;
/// A map holds far fewer towers; this only bounds what a scripted client can make us remember.
const MAX_TOWERS: usize = 1024;

/// `tower_defense.json`: one game per MiniGameData row, plus the tower and enemy rows the games share.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TdContent {
    pub schema_version: u32,
    /// Provenance note; never gameplay input.
    #[serde(default)]
    pub source: Option<String>,
    /// (R) What a win pays.
    pub win_coins: u32,
    /// (R) The least time a stage can take, so a scripted client cannot win in a burst of reports.
    pub min_stage_ms: u32,
    pub games: Vec<TdGameRow>,
    /// The world's tables and what a click on each lists.
    #[serde(default)]
    pub tables: Vec<TdTable>,
    /// TowerData rows (`Game.TOWER_*`): ID, TOWER_ID (the type), RANK, ...
    pub towers: Vec<String>,
    /// EnemyData rows (`Game.ENEMY_*`): ID, SWF_NAME, ...
    pub enemies: Vec<String>,
}

/// What moves a goal's count.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalCount {
    Win,
    Build,
    Upgrade,
    /// Built towers of a type, a limit not to pass.
    Avoid,
    Score,
}

/// How one of a game's `goals` counts.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GoalRule {
    pub goal: u32,
    pub counts: GoalCount,
    /// The tower type watched (`TOWER_TOWER_ID`), for `Build`, `Upgrade` and `Avoid`.
    #[serde(default)]
    pub tower: i32,
    /// What meets the goal; for `Avoid`, the most that may be built.
    pub target: u32,
}

/// One world table: its NPC content key and the games its menu lists, in order.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TdTable {
    pub npc: u64,
    pub name: String,
    /// Where its spawn stands, for the click's range check.
    pub position: [f32; 3],
    pub games: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TdGameRow {
    pub game_id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub icon_id: i32,
    pub difficulty: i32,
    /// The movie's map row (`Game.MAP_*`).
    pub map: String,
    /// The map's tower types (TOWER_ID), in button order.
    pub towers: Vec<i32>,
    /// The map's enemy ids, in wave order.
    pub enemies: Vec<i32>,
    /// What the offer lists (39/16).
    #[serde(default)]
    pub goals: Vec<Goal>,
    /// How those goals count. Empty, nothing ticks.
    #[serde(default)]
    pub progress: Vec<GoalRule>,
}

impl TdContent {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(directory.join(CONTENT_FILE))?
            .take(MAX_CONTENT + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_CONTENT {
            return Err(invalid("tower defense content too large"));
        }
        serde_json::from_str(&text).map_err(invalid)
    }

    /// Every game resolved into its level; any bad row is an error.
    fn levels(&self) -> io::Result<BTreeMap<i32, Level>> {
        if self.schema_version != 1 || self.games.is_empty() || self.games.len() > MAX_GAMES {
            return Err(invalid("invalid tower defense content"));
        }
        let towers = (self.towers.iter())
            .map(|row| Ok((columns(row, TOWER_COLUMNS)?, row.as_str())))
            .collect::<io::Result<Vec<_>>>()?;
        let enemies = (self.enemies.iter())
            .map(|row| Ok((columns(row, ENEMY_COLUMNS)?, row.as_str())))
            .collect::<io::Result<Vec<_>>>()?;
        let mut levels = BTreeMap::new();
        for game in &self.games {
            let map = columns(&game.map, MAP_COLUMNS)?;
            let number = |column: usize| map[column].parse::<u32>().ok();
            let (Some(lives @ 1..), Some(stages @ 1..=MAX_STAGES)) =
                (number(MAP_LIVES), number(MAP_STAGES))
            else {
                return Err(invalid("tower defense map lives or stages"));
            };
            let watched = |rule: &GoalRule| {
                game.goals.iter().any(|goal| goal.id == rule.goal)
                    && (matches!(rule.counts, GoalCount::Win | GoalCount::Score)
                        || game.towers.contains(&rule.tower))
            };
            if game.game_id <= 0
                || !game.progress.iter().all(watched)
                || map[MAP_GAME_ID].parse() != Ok(game.game_id)
                || !(1..=5).contains(&game.difficulty)
                || game.towers.is_empty()
                || game.enemies.is_empty()
                || game.goals.len() > minigame::MAX_GOALS
                || game
                    .goals
                    .iter()
                    .any(|goal| !(1..=5).contains(&goal.category))
            {
                return Err(invalid("invalid tower defense game"));
            }
            let mut texts = vec![
                format!("OnGameDataMsg\t{}", game.game_id),
                format!("OnMapDataMsg\t{}", game.map),
            ];
            for kind in &game.towers {
                let mut ranks = (towers.iter())
                    .filter(|(fields, _)| fields[1].parse() == Ok(*kind))
                    .map(|(fields, row)| Ok((fields[2].parse::<u32>().map_err(invalid)?, *row)))
                    .collect::<io::Result<Vec<_>>>()?;
                ranks.sort_by_key(|(rank, _)| *rank);
                // The movie indexes a tower's ranks from 0 (`mTowerDataArray[upgradeLevel]`).
                let contiguous =
                    (ranks.iter().enumerate()).all(|(i, (rank, _))| *rank as usize == i);
                if ranks.is_empty() || !contiguous {
                    return Err(invalid(
                        "tower defense map names an unknown or gapped tower",
                    ));
                }
                texts.extend(
                    ranks
                        .iter()
                        .map(|(_, row)| format!("OnTowerDataMsg\t{row}")),
                );
            }
            for id in &game.enemies {
                let (_, row) = (enemies.iter())
                    .find(|(fields, _)| fields[0].parse() == Ok(*id))
                    .ok_or_else(|| invalid("tower defense map names an unknown enemy"))?;
                texts.push(format!("OnEnemyDataMsg\t{row}"));
            }
            texts.push("OnGameDataCompleteMsg".to_owned());
            let level = Level {
                row: game.clone(),
                lives,
                stages,
                texts,
            };
            if levels.insert(game.game_id, level).is_some() {
                return Err(invalid("duplicate tower defense game"));
            }
        }
        Ok(levels)
    }

    /// Each table's menu, a game id and its name per row; a table with no known game is an error.
    fn table_menus(&self, levels: &BTreeMap<i32, Level>) -> io::Result<TableMenus> {
        let mut menus = BTreeMap::new();
        for table in &self.tables {
            let games = (table.games.iter())
                .map(|&game| {
                    let level = i32::try_from(game).ok().and_then(|id| levels.get(&id))?;
                    Some((game, u32::try_from(level.row.name_id).ok()?))
                })
                .collect::<Option<Vec<_>>>()
                .filter(|rows| !rows.is_empty())
                .ok_or_else(|| invalid("a tower defense table lists an unknown game"))?;
            let menu = TableMenu {
                name: table.name.clone(),
                position: table.position,
                games,
            };
            if menus.insert(table.npc, menu).is_some() {
                return Err(invalid("duplicate tower defense table"));
            }
        }
        Ok(menus)
    }
}

/// A row's fields without its trailing `^`: exactly `count` of them, with no tab or NUL to break the text.
fn columns(row: &str, count: usize) -> io::Result<Vec<&str>> {
    let fields: Vec<&str> = (row.strip_suffix('^'))
        .filter(|body| body.len() < MAX_TEXT && !body.contains(['\t', '\0']))
        .map(|body| body.split('^').collect())
        .unwrap_or_default();
    if fields.len() != count {
        return Err(invalid("malformed tower defense row"));
    }
    Ok(fields)
}

/// One game's resolved level: its row, the map's lives and stages, and the data messages in order.
#[derive(Clone, Debug)]
struct Level {
    row: TdGameRow,
    lives: u32,
    stages: u32,
    texts: Vec<String>,
}

/// The open run's game and what the client has reported so far.
#[derive(Clone, Copy, Debug)]
struct Play {
    game_id: i32,
    started_ms: u64,
    /// When the level rows went out (`FRServer_ReadyForGameData` answered).
    data_sent_ms: Option<u64>,
    /// The last stage the movie moved past (`FRServer_CompletedStage`); `None` before the first wave.
    stage: Option<u32>,
    lives: u32,
    /// The last accepted running total.
    score: f64,
}

/// What the open game's goals have counted, and which are settled.
#[derive(Clone, Debug, Default)]
struct GoalTally {
    /// Towers built, by tower type.
    built: BTreeMap<i32, u32>,
    /// Upgraded towers: tower type, the movie's tower number.
    upgraded: BTreeSet<(i32, u32)>,
    /// The last count sent, by goal.
    sent: BTreeMap<u32, u32>,
    settled: BTreeSet<u32>,
}

/// What the reports so far mean for the goals: a new count, then a goal met or broken. The panel
/// shows a count only for building and upgrading (`F60aMWGo3a0_66.00`). (R) Their order and the
/// banner per goal are unseen until a native run; a game with no `progress` sends none of this.
fn goal_ticks(rules: &[GoalRule], tally: &mut GoalTally, score: u32, won: bool) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();
    for rule in rules {
        if tally.settled.contains(&rule.goal) {
            continue;
        }
        let built = tally.built.get(&rule.tower).copied().unwrap_or(0);
        let upgraded = (tally.upgraded.iter()).filter(|(kind, _)| *kind == rule.tower).count();
        let count = match rule.counts {
            GoalCount::Win => u32::from(won),
            GoalCount::Build | GoalCount::Avoid => built,
            GoalCount::Upgrade => u32::try_from(upgraded).unwrap_or(u32::MAX),
            GoalCount::Score => score,
        };
        if matches!(rule.counts, GoalCount::Build | GoalCount::Upgrade)
            && tally.sent.get(&rule.goal).copied().unwrap_or(0) != count
        {
            tally.sent.insert(rule.goal, count);
            packets.extend(minigame::goal_progress_packets(rule.goal, count));
        }
        let (met, broken) = match rule.counts {
            GoalCount::Avoid => (won && count <= rule.target, count > rule.target),
            _ => (count >= rule.target, false),
        };
        if met {
            tally.settled.insert(rule.goal);
            packets.extend(minigame::goal_done_packets(rule.goal));
        } else if broken {
            tally.settled.insert(rule.goal);
            packets.extend(minigame::goal_failed_packets(rule.goal));
        }
    }
    packets
}

struct Finish {
    won: bool,
    score: u32,
    elapsed_seconds: u32,
}

impl Activity for Finish {
    fn score(&self) -> u32 {
        self.score
    }
    fn finish(&self) -> Option<ActivityFinish> {
        Some(ActivityFinish {
            won: self.won,
            stars: 0,
            elapsed_seconds: self.elapsed_seconds,
        })
    }
}

/// What one packet did, for gameplay to apply. `changed` marks a lifecycle step worth saving; reports never
/// set it (the run is session-local).
#[derive(Default)]
pub struct TdOutcome {
    pub accepted: bool,
    pub changed: bool,
    pub replies: Vec<Vec<u8>>,
    /// Why a client report was refused (nothing changed) or a reported win was overruled.
    pub refused: Option<&'static str>,
    /// A won game's payout, to adopt before sending `reward_packets`, as for checkers.
    pub reward_packets: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
}

impl TdOutcome {
    fn from_run(run: RunOutcome) -> Self {
        Self {
            accepted: run.accepted,
            changed: run.changed,
            replies: run.replies,
            refused: None,
            reward_packets: run.reward_packets,
            reward_state: run.reward_state,
        }
    }

    fn ack() -> Self {
        Self {
            accepted: true,
            ..Self::default()
        }
    }

    fn refused(reason: &'static str) -> Self {
        Self {
            refused: Some(reason),
            ..Self::default()
        }
    }
}

/// One player's Tower Defense run against the server, on the shared minigame lifecycle (offer 39/16, start
/// 39/5, movie text 39/14, result 39/18, close 26/42). Session-local like checkers: a reconnect starts fresh.
#[derive(Clone)]
pub struct TowerDefense {
    levels: Arc<BTreeMap<i32, Level>>,
    tables: Arc<TableMenus>,
    /// Tower row id (`TOWER_ID`) to its type, for the movie's tower reports.
    tower_kinds: Arc<BTreeMap<u32, i32>>,
    tally: GoalTally,
    win_coins: u32,
    min_stage_ms: u32,
    runs: MinigameRuns,
    play: Option<Play>,
}

impl TowerDefense {
    /// What a click on each world table lists.
    pub fn tables(&self) -> &TableMenus {
        &self.tables
    }

    pub fn new(content: &TdContent) -> io::Result<Self> {
        let levels = content.levels()?;
        let tower_kinds = (content.towers.iter())
            .filter_map(|row| {
                let mut fields = row.split('^');
                Some((fields.next()?.parse().ok()?, fields.next()?.parse().ok()?))
            })
            .collect();
        Ok(Self {
            tower_kinds: Arc::new(tower_kinds),
            tally: GoalTally::default(),
            tables: Arc::new(content.table_menus(&levels)?),
            levels: Arc::new(levels),
            win_coins: content.win_coins,
            min_stage_ms: content.min_stage_ms,
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            play: None,
        })
    }

    /// The MiniGameData ids this content offers (the Games window rows' ids).
    pub fn game_ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.levels.keys().copied()
    }

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    /// Offer game `game_id` (39/16); `None` while a run is already open. An id the content does not carry is
    /// an error.
    pub fn begin(&mut self, game_id: i32) -> io::Result<Option<TdOutcome>> {
        let level = self
            .levels
            .get(&game_id)
            .ok_or_else(|| invalid("not a Tower Defense game"))?;
        if self.is_active() {
            return Ok(None);
        }
        let row = &level.row;
        let mut spec = RunSpec::empty(
            game_id,
            GAME_TYPE,
            RunMode::Flash {
                movie: MOVIE.into(),
            },
        );
        spec.name_id = row.name_id;
        spec.description_id = row.description_id;
        spec.icon_id = row.icon_id;
        spec.difficulty = row.difficulty;
        spec.preload_file = PRELOAD_FILE.into();
        spec.objectives.clone_from(&row.goals);
        let lives = level.lives;
        // `offer` changes nothing on an error or a refusal.
        let offered = self.runs.offer(spec)?;
        if !offered.accepted {
            return Ok(None);
        }
        self.tally = GoalTally::default();
        self.play = Some(Play {
            game_id,
            started_ms: 0,
            data_sent_ms: None,
            stage: None,
            lives,
            score: 0.0,
        });
        Ok(Some(TdOutcome::from_run(offered)))
    }

    /// One client packet while a run is open, where a won game pays the content's coins to `payer` (player
    /// guid and reward state); `None` for a packet that is not this run's. A refused or stale packet changes
    /// nothing; a malformed one is an error.
    pub fn route_with(
        &mut self,
        packet: &[u8],
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<Option<TdOutcome>> {
        let Some(instance) = self.runs.current_instance_id() else {
            return Ok(None);
        };
        if packet.starts_with(&[39, 0, 14]) {
            if crate::le::i32(packet, 3) != Some(instance) {
                return Ok(None);
            }
            return self.movie(instance, packet, now_ms, payer).map(Some);
        }
        let start = matches!(packet.get(..3), Some([39, 0, 5]));
        if !matches!(packet.get(..3), Some([39, 0, 5..=7])) && packet != [26, 0, 42, 0] {
            return Ok(None);
        }
        let mut runs = self.runs.clone();
        let mut out = TdOutcome::from_run(runs.handle_client(packet)?);
        if start && out.accepted {
            out.changed |= runs.started(instance).changed;
            if let Some(play) = self.play.as_mut() {
                play.started_ms = now_ms;
            }
        }
        if out.accepted || out.changed {
            self.runs = runs;
            if !self.is_active() {
                self.play = None;
            }
        }
        Ok(Some(out))
    }

    fn movie(
        &mut self,
        instance: i32,
        packet: &[u8],
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<TdOutcome> {
        let text = minigame::movie_text(packet)?;
        let Some(mut play) = self.play else {
            return Ok(TdOutcome::refused("no tower defense game is open"));
        };
        // The client's SendPayload names game -1; ours names the game.
        if crate::le::i32(packet, 7) != Some(-1)
            || !matches!(crate::le::i32(packet, 11), Some(id) if id == -1 || id == play.game_id)
        {
            return Err(invalid("tower defense text for another game"));
        }
        let levels = Arc::clone(&self.levels);
        let level = &levels[&play.game_id];
        let (lives, stages) = (level.lives, level.stages);
        let fields: Vec<&str> = text.split('\t').collect();
        // Connect, pause and close reports are harmless at any time.
        if matches!(
            fields.as_slice(),
            ["OnConnectMsg" | "FRServer_GameClose"] | ["OnPauseMsg", _]
        ) {
            return Ok(TdOutcome::ack());
        }
        let playing = self.runs.state(instance) == Some(RunState::Playing);
        if fields.as_slice() == ["FRServer_GameOver"] {
            // The movie has shown its ending; Penguin Wars (`TD_SnowGame`) ends a lost game here without
            // `FRServer_GameLost`, so a game still open settles as a loss.
            return match playing {
                true => self.settle(instance, &play, false, play.score, now_ms, None),
                false => Ok(TdOutcome::ack()),
            };
        }
        if !playing {
            return Ok(TdOutcome::refused(
                "the tower defense game is not being played",
            ));
        }
        let Some(sent_ms) = play.data_sent_ms else {
            if fields.as_slice() != ["FRServer_ReadyForGameData"] {
                return Ok(TdOutcome::refused("a report before the game's data"));
            }
            // Sent once: the movie adds every row it gets, so a second copy would double its towers.
            let replies = (level.texts.iter())
                .map(|text| minigame::payload_packet(instance, play.game_id, text))
                .collect::<io::Result<_>>()?;
            play.data_sent_ms = Some(now_ms);
            self.play = Some(play);
            return Ok(TdOutcome {
                accepted: true,
                replies,
                ..TdOutcome::default()
            });
        };
        let floor_ms = u64::from(stages) * u64::from(self.min_stage_ms);
        let number = |text: &str| text.parse::<u32>().ok();
        let mut out = match fields.as_slice() {
            ["FRServer_ReadyForGameData"] => TdOutcome::refused("the game's data was already sent"),
            ["FRServer_CompletedStage", stage] => match (number(stage), play.stage) {
                (Some(0), None) => {
                    play.stage = Some(0);
                    TdOutcome::ack()
                }
                (Some(next), Some(last))
                    if (next == last || next == last + 1) && next <= stages =>
                {
                    play.stage = Some(next);
                    TdOutcome::ack()
                }
                _ => TdOutcome::refused("a stage out of order"),
            },
            ["FRServer_LivesCount", left] => match number(left) {
                Some(left) if left <= lives => {
                    play.lives = left;
                    TdOutcome::ack()
                }
                _ => TdOutcome::refused("more lives than the map gives"),
            },
            ["FRServer_ScoreSet", total] => match checked_total(total, &play) {
                Ok(total) => {
                    play.score = total;
                    TdOutcome::ack()
                }
                Err(reason) => TdOutcome::refused(reason),
            },
            ["FRServer_TowerBuilt", _, row] => {
                if let Some(kind) = number(row).and_then(|row| self.tower_kinds.get(&row)) {
                    *self.tally.built.entry(*kind).or_default() += 1;
                }
                TdOutcome::ack()
            }
            ["FRServer_TowerUpgraded", tower, row, _] => {
                let kind = number(row).and_then(|row| self.tower_kinds.get(&row));
                if let (Some(kind), Some(tower)) = (kind, number(tower))
                    && self.tally.upgraded.len() < MAX_TOWERS
                {
                    self.tally.upgraded.insert((*kind, tower));
                }
                TdOutcome::ack()
            }
            // The movie keeps its money, towers and kills; these only tell the server what it did.
            ["FRServer_ScoreIncrement", _]
            | ["FRServer_EnemyDied", _]
            | ["FRServer_TowerSold", _]
            | ["FRServer_MoneySpent", _]
            | ["FRServer_MoneyEarned", _]
            | ["FRServer_PowerUpCollected" | "FRServer_PowerUpUsed"] => TdOutcome::ack(),
            ["FRServer_GameWon", total] => {
                let verdict = checked_total(total, &play).and_then(|total| {
                    let cleared = play.stage == Some(stages) && play.lives > 0;
                    match (cleared, now_ms.saturating_sub(sent_ms) >= floor_ms) {
                        (true, true) => Ok(total),
                        (false, _) => Err("a win before every stage, or with no lives"),
                        (true, false) => Err("a win faster than its stages allow"),
                    }
                });
                match verdict {
                    Ok(total) => {
                        // Bounded by `checked_total`; `as` floors a finite total.
                        let ticks = goal_ticks(&level.row.progress, &mut self.tally, total as u32, true);
                        let mut won = self.settle(instance, &play, true, total, now_ms, payer)?;
                        won.replies.splice(0..0, ticks);
                        won
                    }
                    Err(reason) => TdOutcome {
                        refused: Some(reason),
                        ..self.settle(instance, &play, false, play.score, now_ms, None)?
                    },
                }
            }
            ["FRServer_GameLost", total] => {
                let total = checked_total(total, &play).unwrap_or(play.score);
                self.settle(instance, &play, false, total, now_ms, None)?
            }
            _ => TdOutcome::refused("not a Tower Defense message"),
        };
        if out.accepted {
            self.play = Some(play);
            let ticks = goal_ticks(&level.row.progress, &mut self.tally, play.score as u32, false);
            out.replies.extend(ticks);
        }
        Ok(out)
    }

    /// The server's verdict: `OnGameWonMsg` / `OnGameLostMsg`, then the result screen.
    fn settle(
        &mut self,
        instance: i32,
        play: &Play,
        won: bool,
        score: f64,
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<TdOutcome> {
        let elapsed = now_ms.saturating_sub(play.started_ms) / 1000;
        let finish = Finish {
            won,
            // Bounded by `checked_total`; `as` floors a finite total.
            score: score as u32,
            elapsed_seconds: u32::try_from(elapsed).unwrap_or(u32::MAX),
        };
        // `finish` changes nothing on an error.
        let run = match (won, payer) {
            (true, Some((player, rewards))) => {
                let row = &self.levels[&play.game_id].row;
                let grant = win_grant(player, self.win_coins, row.icon_id, row.name_id);
                self.runs
                    .finish(instance, &finish, Some((rewards, &grant)))?
            }
            (true, None) => self.runs.finish_win_without_reward(instance, &finish)?,
            (false, _) => self.runs.finish(instance, &finish, None)?,
        };
        Ok(TdOutcome::from_run(run))
    }
}

pub(crate) fn win_grant(player_id: u64, coins: u32, icon_id: i32, name_id: i32) -> RewardGrant {
    RewardGrant {
        coins,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        sc: 0,
        display: RewardDisplay {
            source_guid: player_id,
            player_guid: player_id,
            reward_kind: 0,
            icon_id,
            name_id,
            rows: Vec::new(),
        },
    }
}

/// A reported total: a finite, non-negative number within the shared bound, not below the last.
fn checked_total(text: &str, play: &Play) -> Result<f64, &'static str> {
    let total = (text.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
        .ok_or("the total is not a number")?;
    if total > f64::from(crate::content::MAX_MINIGAME_SCORE) {
        return Err("the total is beyond any game");
    }
    // The periodic report is `Math.round(mScore)` while the win carries the raw total (`TD_Game.as`).
    if total + 0.5 < play.score {
        return Err("the total went down");
    }
    Ok(total)
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// The goal packets a report sent, as (family, sub, goal, count) with 0 where none is carried.
    fn ticks(out: &TdOutcome) -> Vec<(u16, u8, u32, u32)> {
        (out.replies.iter())
            .filter(|p| p.starts_with(&[47, 0]) || p.starts_with(&[45, 0]))
            .map(|p| {
                let word = |at: usize| (p.get(at..at + 4)).map_or(0, |w| u32::from_le_bytes(w.try_into().unwrap()));
                (u16::from(p[0]), p[2], word(3), if p[2] == 2 { word(7) } else { 0 })
            })
            .collect()
    }

    /// Crop Dusting - Hard: upgrade 4 Garden, upgrade 4 Dwarf, no more than 2 Water Guns, 930,000.
    #[test]
    fn tower_reports_tick_the_games_goals() {
        let mut td = TowerDefense::new(&content()).unwrap();
        let instance = start(&mut td, 156);
        let upgrade = |td: &mut TowerDefense, tower: u32| {
            let text = format!("FRServer_TowerUpgraded\t{tower}\t121\t1");
            say(td, instance, &text, 3_000)
        };
        // A Garden Tower built moves nothing: this goal counts upgrades.
        assert!(ticks(&say(&mut td, instance, "FRServer_TowerBuilt\t1\t30", 3_000)).is_empty());
        assert_eq!(ticks(&upgrade(&mut td, 9)), [(47, 2, 3, 1), (45, 2, 3, 1)]);
        assert!(ticks(&upgrade(&mut td, 9)).is_empty(), "the same tower again");
        for tower in [10, 11] {
            upgrade(&mut td, tower);
        }
        let met = ticks(&upgrade(&mut td, 12));
        assert_eq!(met, [(47, 2, 3, 4), (45, 2, 3, 4), (47, 3, 3, 0), (45, 3, 3, 0)]);
        assert!(ticks(&upgrade(&mut td, 13)).is_empty(), "a met goal is closed");
        // Two Water Guns are allowed; the third breaks the goal, once.
        for tower in [20, 21] {
            let text = format!("FRServer_TowerBuilt\t{tower}\t70");
            assert!(ticks(&say(&mut td, instance, &text, 3_000)).is_empty());
        }
        let broken = say(&mut td, instance, "FRServer_TowerBuilt\t22\t70", 3_000);
        assert_eq!(ticks(&broken), [(47, 4, 4, 0), (45, 4, 4, 0)]);
        let score = say(&mut td, instance, "FRServer_ScoreSet\t930000", 3_000);
        assert_eq!(ticks(&score), [(47, 3, 5, 0), (45, 3, 5, 0)]);
        // A row no game has counts for nothing.
        assert!(ticks(&say(&mut td, instance, "FRServer_TowerBuilt\t23\t9999", 3_000)).is_empty());
    }

    /// The basic rows stand on their tables with three goals: no limit and no Elite.
    #[test]
    fn the_basic_rows_are_games_of_their_own() {
        let content = content();
        for id in 450..=457 {
            let game = content.games.iter().find(|g| g.game_id == id).unwrap();
            assert_eq!(game.goals.len(), 3, "{id}");
            assert_eq!(game.progress.len(), 3, "{id}");
        }
        let td = TowerDefense::new(&content).unwrap();
        let basic = (content.tables.iter()).find(|t| t.name == "Garden Defense").unwrap();
        assert_eq!(basic.games, [453, 455]);
        assert_eq!(td.levels[&454].stages, 10);
        assert_eq!(td.levels[&453].stages, 10);
    }

    /// A win meets the Primary and an unbroken "don't build", ahead of the result.
    #[test]
    fn a_win_settles_the_goals_before_the_result() {
        let mut td = TowerDefense::new(&content()).unwrap();
        let instance = start(&mut td, 256);
        clear_stages(&mut td, instance, 25);
        let won = say(&mut td, instance, "FRServer_GameWon\t4321", 60_000);
        assert_eq!(ticks(&won), [(47, 3, 1, 0), (45, 3, 1, 0), (47, 3, 4, 0), (45, 3, 4, 0)]);
        assert!(won.replies[0].starts_with(&[47, 0, 3]));
        assert!(result(&won).is_some());
    }

    fn content() -> TdContent {
        TdContent::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap()
    }

    fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
        let mut packet = vec![39, 0, sub];
        for value in [instance, -1, -1] {
            packet.extend(value.to_le_bytes());
        }
        packet
    }

    fn texts(out: &TdOutcome) -> Vec<String> {
        (out.replies.iter())
            .filter(|p| p.starts_with(&[39, 0, 14]))
            .map(|p| String::from_utf8_lossy(&p[19..p.len() - 1]).into_owned())
            .collect()
    }

    fn result(out: &TdOutcome) -> Option<&[u8]> {
        (out.replies.iter())
            .find(|p| p.starts_with(&[39, 0, 18]))
            .map(Vec::as_slice)
    }

    /// Offer `game`, start it at t=1s and send its data at t=2s; the run's instance.
    fn start(td: &mut TowerDefense, game: i32) -> i32 {
        let offer = td.begin(game).unwrap().expect("offered");
        assert!(offer.replies[0].starts_with(&[39, 0, 16]));
        let preload = PRELOAD_FILE.as_bytes();
        assert!(
            offer.replies[0]
                .windows(preload.len())
                .any(|w| w == preload)
        );
        let instance = i32::from_le_bytes(offer.replies[0][3..7].try_into().unwrap());
        assert!(td.begin(game).unwrap().is_none(), "one run at a time");
        let started = td
            .route_with(&lifecycle(5, instance), 1_000, None)
            .unwrap()
            .unwrap();
        assert!(started.accepted && started.changed);
        assert!(say(td, instance, "OnConnectMsg", 1_500).accepted);
        let data = say(td, instance, "FRServer_ReadyForGameData", 2_000);
        assert!(data.accepted && !data.changed);
        instance
    }

    fn say(td: &mut TowerDefense, instance: i32, text: &str, now_ms: u64) -> TdOutcome {
        let packet = minigame::payload_packet(instance, -1, text).unwrap();
        td.route_with(&packet, now_ms, None)
            .unwrap()
            .expect("the run's text")
    }

    /// Reports every stage of a map, 0 to its last, as `TD_Game.nextLevel` does.
    fn clear_stages(td: &mut TowerDefense, instance: i32, stages: u32) {
        for stage in 0..=stages {
            let out = say(
                td,
                instance,
                &format!("FRServer_CompletedStage\t{stage}"),
                3_000,
            );
            assert!(out.accepted, "stage {stage}");
        }
    }

    #[test]
    fn shipped_content_resolves_every_game_into_its_rows() {
        let td = TowerDefense::new(&content()).unwrap();
        let ids: Vec<_> = td.game_ids().collect();
        let expected: Vec<i32> = (150..=161)
            .chain(250..=261)
            .chain(350..=361)
            .chain(450..=457)
            .collect();
        assert_eq!(ids, expected);
        let level = &td.levels[&150];
        assert_eq!((level.lives, level.stages), (15, 50));
        assert_eq!(level.texts[0], "OnGameDataMsg\t150");
        assert!(level.texts[1].starts_with("OnMapDataMsg\t150^150^1^1^1^"));
        assert!(
            level.texts[1].ends_with("^0.8^"),
            "the trailing ^ survives ChopLastChar"
        );
        assert_eq!(level.texts.last().unwrap(), "OnGameDataCompleteMsg");
        // Towers in the map's order (8, 0, 7, 17), each tower's ranks in turn.
        let towers: Vec<_> = (level.texts.iter())
            .filter_map(|t| t.strip_prefix("OnTowerDataMsg\t"))
            .map(|row| {
                let f: Vec<&str> = row.split('^').collect();
                (f[1].parse::<i32>().unwrap(), f[2].parse::<i32>().unwrap())
            })
            .collect();
        let kinds: Vec<i32> = towers.iter().map(|t| t.0).collect::<Vec<_>>();
        let mut order = kinds.clone();
        order.dedup();
        assert_eq!(order, [8, 0, 7, 17]);
        assert!(
            towers
                .windows(2)
                .all(|w| w[0].0 != w[1].0 || w[0].1 < w[1].1),
            "ranks ascend"
        );
        let enemies: Vec<_> = (level.texts.iter())
            .filter_map(|t| t.strip_prefix("OnEnemyDataMsg\t"))
            .map(|row| row.split('^').next().unwrap().parse::<i32>().unwrap())
            .collect();
        assert_eq!(enemies, [5, 3, 2]);
    }

    #[test]
    fn a_cleared_map_wins_once_and_the_data_goes_out_once() {
        let mut td = TowerDefense::new(&content()).unwrap();
        let instance = start(&mut td, 350);
        let again = say(&mut td, instance, "FRServer_ReadyForGameData", 2_100);
        assert!(
            again.refused.is_some() && again.replies.is_empty(),
            "data once"
        );
        let stages = td.levels[&350].stages;
        for report in [
            "OnPauseMsg\ttrue",
            "FRServer_TowerBuilt\t0\t10",
            "FRServer_MoneySpent\t15",
            "FRServer_EnemyDied\t1",
            "FRServer_MoneyEarned\t2",
            "FRServer_LivesCount\t9",
            "FRServer_ScoreSet\t120.5",
        ] {
            let out = say(&mut td, instance, report, 2_500);
            assert!(
                out.accepted && !out.changed && out.replies.is_empty(),
                "{report}"
            );
        }
        clear_stages(&mut td, instance, stages);
        let won = say(&mut td, instance, "FRServer_GameWon\t9000", 60_000);
        assert_eq!(won.refused, None);
        assert_eq!(texts(&won), ["OnGameWonMsg"]);
        let screen = result(&won).expect("the result screen");
        assert_eq!(screen[15], 1, "won");
        assert_eq!(&screen[24..28], &9000_i32.to_le_bytes(), "the total");
        let late = say(&mut td, instance, "FRServer_GameWon\t9000", 60_100);
        assert!(result(&late).is_none(), "the verdict is given once");
        assert!(say(&mut td, instance, "FRServer_GameOver", 61_000).accepted);
        let close = td
            .route_with(&[26, 0, 42, 0], 62_000, None)
            .unwrap()
            .unwrap();
        assert!(close.accepted && close.changed);
        assert!(!td.is_active());
        assert!(
            td.begin(150).unwrap().is_some(),
            "a new run after the close"
        );
    }

    #[test]
    fn reports_out_of_bounds_are_refused_and_a_short_win_is_a_loss() {
        let mut td = TowerDefense::new(&content()).unwrap();
        let instance = start(&mut td, 250);
        for report in [
            "FRServer_CompletedStage\t1",
            "FRServer_LivesCount\t99",
            "FRServer_ScoreSet\t-4",
            "FRServer_ScoreSet\tNaN",
            "FRServer_Teleport\t1",
        ] {
            assert!(
                say(&mut td, instance, report, 3_000).refused.is_some(),
                "{report}"
            );
        }
        assert!(say(&mut td, instance, "FRServer_ScoreSet\t50", 3_000).accepted);
        assert!(
            say(&mut td, instance, "FRServer_ScoreSet\t40", 3_000)
                .refused
                .is_some(),
            "down"
        );
        assert!(say(&mut td, instance, "FRServer_CompletedStage\t0", 3_000).accepted);
        assert!(
            say(&mut td, instance, "FRServer_CompletedStage\t2", 3_000)
                .refused
                .is_some(),
            "a skip"
        );
        let short = say(&mut td, instance, "FRServer_GameWon\t50", 4_000);
        assert!(short.refused.is_some(), "overruled");
        assert_eq!(texts(&short), ["OnGameLostMsg"]);
        assert_eq!(result(&short).expect("the result screen")[15], 0, "lost");
    }

    #[test]
    fn a_win_with_no_lives_and_a_silent_game_over_both_lose() {
        let mut td = TowerDefense::new(&content()).unwrap();
        let instance = start(&mut td, 150);
        let stages = td.levels[&150].stages;
        clear_stages(&mut td, instance, stages);
        assert!(say(&mut td, instance, "FRServer_LivesCount\t0", 5_000).accepted);
        let none_left = say(&mut td, instance, "FRServer_GameWon\t10", 6_000);
        assert_eq!(texts(&none_left), ["OnGameLostMsg"]);

        let mut td = TowerDefense::new(&content()).unwrap();
        let instance = start(&mut td, 151);
        let over = say(&mut td, instance, "FRServer_GameOver", 7_000);
        assert_eq!(texts(&over), ["OnGameLostMsg"], "Penguin Wars' silent end");
        assert_eq!(result(&over).expect("the result screen")[15], 0);
    }

    #[test]
    fn a_report_before_the_data_an_early_win_and_a_reported_loss() {
        let mut td = TowerDefense::new(&content()).unwrap();
        let offer = td.begin(352).unwrap().unwrap();
        let instance = i32::from_le_bytes(offer.replies[0][3..7].try_into().unwrap());
        td.route_with(&lifecycle(5, instance), 1_000, None)
            .unwrap()
            .unwrap();
        let early = say(&mut td, instance, "FRServer_ScoreSet\t5", 1_100);
        assert!(early.refused.is_some(), "a report before the data");
        assert!(say(&mut td, instance, "FRServer_ReadyForGameData", 2_000).accepted);
        let stages = td.levels[&352].stages;
        clear_stages(&mut td, instance, stages);
        let burst = say(&mut td, instance, "FRServer_GameWon\t100", 2_500);
        assert_eq!(burst.refused, Some("a win faster than its stages allow"));
        assert_eq!(texts(&burst), ["OnGameLostMsg"]);

        let mut td = TowerDefense::new(&content()).unwrap();
        let instance = start(&mut td, 353);
        let stages = td.levels[&353].stages;
        clear_stages(&mut td, instance, stages);
        let past = format!("FRServer_CompletedStage\t{}", stages + 1);
        assert!(
            say(&mut td, instance, &past, 4_000).refused.is_some(),
            "past the last stage"
        );
        assert!(say(&mut td, instance, "FRServer_ScoreSet\t800", 4_000).accepted);
        assert!(
            say(&mut td, instance, "FRServer_ScoreSet\t799.6", 4_100).accepted,
            "rounding slack"
        );
        let lost = say(&mut td, instance, "FRServer_GameLost\t812", 5_000);
        assert_eq!(texts(&lost), ["OnGameLostMsg"]);
        let screen = result(&lost).expect("the result screen");
        assert_eq!(
            (screen[15], &screen[24..28]),
            (0, &812_i32.to_le_bytes()[..])
        );
    }

    /// B7, D19: every game's offer lists its goals, the Primary first.
    #[test]
    fn every_shipped_game_offers_its_goals() {
        let content = content();
        for game in &content.games {
            assert_eq!(
                game.goals.first().map(|g| g.category),
                Some(1),
                "{}",
                game.game_id
            );
            // The client adds the category prefix itself; the 94xxx names carry none.
            assert!(
                game.goals.iter().all(|g| g.name_id >= 94_000),
                "{}",
                game.game_id
            );
            let mut td = TowerDefense::new(&content).unwrap();
            let offered = td.begin(game.game_id).unwrap().unwrap();
            let bundles = 3 + 12 + 24 + 1 + 3 * 69;
            let count = &offered.replies[0][bundles..bundles + 4];
            assert_eq!(
                count,
                (game.goals.len() as i32).to_le_bytes(),
                "{}",
                game.game_id
            );
        }
    }

    #[test]
    fn bad_content_is_refused() {
        let good = content();
        let mut cases = Vec::new();
        let mut c = good.clone();
        c.games[0].goals[0].category = 6;
        cases.push(c);
        let mut c = good.clone();
        c.tables[0].games.push(999);
        cases.push(c);
        let mut c = good.clone();
        c.tables[0].games.clear();
        cases.push(c);
        let mut c = good.clone();
        c.tables.push(c.tables[0].clone());
        cases.push(c);
        let mut c = good.clone();
        c.games[0].map = c.games[0].map.trim_end_matches('^').to_owned();
        cases.push(c);
        let mut c = good.clone();
        c.games[0].towers.push(999);
        cases.push(c);
        let mut c = good.clone();
        c.games[1].game_id = c.games[0].game_id;
        cases.push(c);
        let mut c = good.clone();
        c.enemies[0] = c.enemies[0].replace("Frost Giant", "Frost\tGiant");
        cases.push(c);
        let mut c = good.clone();
        c.towers[1] = c.towers[1].replacen("^1^", "^0^", 1);
        cases.push(c);
        for (n, case) in cases.iter().enumerate() {
            assert!(TowerDefense::new(case).is_err(), "case {n}");
        }
    }
}
