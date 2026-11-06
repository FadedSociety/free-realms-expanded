//! Spot the Difference, client minigame type 1009 (MiniGameData 1040, 1042, 1044, the Games window
//! rows 1087 to 1089): the movie `ForcePerception.gfx`, refereed by the server.
//!
//! The movie owns nearly everything: pictures, difference positions, rounds, times and points all
//! live in the client (`pieces/spot-the-difference/sheet.md` section 7). The server owns the tier
//! and the verdict:
//! - Client to server: `FRServer_RequestStageId` once, `FRServer_StatUpdate 22 1` for each
//!   difference found (21, 23 and 24 are trophy counters), then `FRServer_GameWon <score>`, one
//!   field, or `FRServer_GameLost <n>`; `FRServer_GameClose` after a verdict, sometimes twice.
//! - Server to client: `VOnServerSetStageIdMsg <tier>`, without which the game never starts, then
//!   `OnGameWonMsg` or `OnGameLostMsg`.
//!
//! Reconstructed (R): the tiers 1, 3 and 5 (Fanbyte's numbers, so Easy, Medium and Hard run 120, 90
//! and 60 s; the row's own difficulty 3 plays as Normal), the coins, the goal's text ids, and that
//! a loss pays nothing. A win needs its stage id, a score above 0, a finished round, a score the
//! finds allow, and each finished round's 2 s tween. There is deliberately no upper time bound and
//! no "all five rounds" rule: the end-of-round screen waits on the player, and the movie itself
//! wins after one round when the player quits. Anything else settles a loss.
use crate::{
    minigame::{self, Goal, MinigameRuns, RunMode, RunSpec, RunState},
    music_game::{Finish, MusicOutcome as SpotOutcome, field, parse_score, payload_packet},
    rewards::RewardState,
    tower_defense::win_grant,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::Path,
};

pub const GAME_TYPE: i32 = 1009;
pub const MOVIE: &str = "ForcePerception.gfx";
pub const CONTENT_FILE: &str = "spot_game.json";
/// The client's preload list. Type 1009 is not one the client readies by itself (`009beb70`).
pub const PRELOAD_FILE: &str = "ForcePerception.txt";
/// (R) Run ids of its own, apart from every other minigame's.
pub const FIRST_INSTANCE_ID: i32 = 1_040_000_001;
const FINDS_PER_ROUND: u32 = 5;
const MAX_FINDS: u32 = 25;
/// `Game.as`: a finished round's closing tween.
const ROUND_END_MS: u64 = 2_000;
const MAX_TIER: u32 = 5;
const GOAL: u32 = 1;
const MAX_CONTENT: u64 = 16 * 1024;
const MAX_GAMES: usize = 8;

/// `spot_game.json`: one game per MiniGameData row.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SpotContent {
    pub schema_version: u32,
    /// Provenance note; never gameplay input.
    #[serde(default)]
    pub source: Option<String>,
    pub description_id: i32,
    pub icon_id: i32,
    /// (R) A win pays `score / 10,000 x this`, at least `win_coins_min`.
    pub win_coins_per_10000: u32,
    pub win_coins_min: u32,
    pub goal_name_id: u32,
    pub goal_description_id: u32,
    pub games: Vec<SpotRow>,
    /// The world's way in, one menu of every game on each.
    pub telescopes: Vec<SpotTelescope>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpotRow {
    pub game_id: i32,
    pub name_id: i32,
    pub difficulty: i32,
    /// `VOnServerSetStageIdMsg`'s tier: the one value the server owns.
    pub stage_id: u32,
    /// The most one find can score at this tier. This mirrors the movie; it tunes nothing.
    pub max_find_points: u32,
}

/// A telescope's spawn (`npc_spawns_extra.json`), as a game table's.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SpotTelescope {
    pub npc: u64,
    pub position: [f32; 3],
}

impl SpotContent {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(directory.join(CONTENT_FILE))?
            .take(MAX_CONTENT + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_CONTENT {
            return Err(invalid("spot game content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(invalid)?;
        content.validate()?;
        Ok(content)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1
            || self.description_id <= 0
            || self.icon_id <= 0
            || self.goal_name_id == 0
            || self.games.is_empty()
            || self.games.len() > MAX_GAMES
        {
            return Err(invalid("invalid spot game content"));
        }
        let mut seen = BTreeSet::new();
        for row in &self.games {
            // The movie never starts on tier 0.
            if row.game_id <= 0
                || row.name_id <= 0
                || !(1..=5).contains(&row.difficulty)
                || !(1..=MAX_TIER).contains(&row.stage_id)
                || row.max_find_points == 0
                || !seen.insert(row.game_id)
            {
                return Err(invalid("invalid spot game row"));
            }
        }
        Ok(())
    }

    fn game(&self, game_id: i32) -> Option<&SpotRow> {
        self.games.iter().find(|row| row.game_id == game_id)
    }

    /// The Goals panel: one goal, ids chosen by wording (R).
    fn goals(&self) -> Vec<Goal> {
        vec![Goal {
            id: GOAL,
            name_id: self.goal_name_id,
            description_id: self.goal_description_id,
            category: 1,
            bonus: false,
            stars: 0,
            coins: 0,
            total: 0,
            failure_condition: false,
            members_only: false,
        }]
    }

    fn coins(&self, score: u32) -> u32 {
        (score / 10_000)
            .saturating_mul(self.win_coins_per_10000)
            .max(self.win_coins_min)
    }
}

/// The open run and what the client has reported so far.
#[derive(Clone, Copy, Debug)]
struct Play {
    row: SpotRow,
    started_ms: u64,
    /// When the tier went out (`FRServer_RequestStageId` answered).
    stage_sent_ms: Option<u64>,
    /// Differences found, by `FRServer_StatUpdate 22 1`.
    finds: u32,
}

/// One player's game against the server, on the shared minigame lifecycle (offer 39/16, start
/// 39/5, movie text 39/14, result 39/18, close 26/42). Session-local: a reconnect starts fresh.
#[derive(Clone)]
pub struct SpotGame {
    content: SpotContent,
    runs: MinigameRuns,
    play: Option<Play>,
}

impl SpotGame {
    pub fn new(content: SpotContent) -> io::Result<Self> {
        content.validate()?;
        Ok(Self {
            content,
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            play: None,
        })
    }

    /// The MiniGameData ids this content offers (the Games window rows' ids).
    pub fn game_ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.content.games.iter().map(|row| row.game_id)
    }

    /// Each game with its menu label, and every telescope that offers them.
    pub fn telescopes(&self) -> (Vec<(u32, u32)>, &[SpotTelescope]) {
        // `validate` holds both ids above 0.
        let games = self.content.games.iter();
        let menu = games.map(|row| (row.game_id as u32, row.name_id as u32));
        (menu.collect(), &self.content.telescopes)
    }

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    /// Offer game `game_id` (39/16); `None` while a run is already open. An id the content does
    /// not carry is an error.
    pub fn begin(&mut self, game_id: i32) -> io::Result<Option<SpotOutcome>> {
        let row = *self
            .content
            .game(game_id)
            .ok_or_else(|| invalid("not a Spot the Difference game"))?;
        if self.is_active() {
            return Ok(None);
        }
        let mut spec = RunSpec::empty(
            game_id,
            GAME_TYPE,
            RunMode::Flash {
                movie: MOVIE.into(),
            },
        );
        spec.name_id = row.name_id;
        spec.description_id = self.content.description_id;
        spec.icon_id = self.content.icon_id;
        spec.difficulty = row.difficulty;
        spec.preload_file = PRELOAD_FILE.into();
        spec.objectives = self.content.goals();
        // `offer` changes nothing on an error or a refusal.
        let offered = self.runs.offer(spec)?;
        if !offered.accepted {
            return Ok(None);
        }
        self.play = Some(Play {
            row,
            started_ms: 0,
            stage_sent_ms: None,
            finds: 0,
        });
        Ok(Some(SpotOutcome::from_run(offered)))
    }

    /// One client packet while a run is open; `None` for a packet that is not this run's, which
    /// includes the movie's late verdict after a quit has closed the run. A refused or stale
    /// packet changes nothing; a malformed one is an error. A win pays `payer`.
    pub fn route_with(
        &mut self,
        packet: &[u8],
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<Option<SpotOutcome>> {
        let Some(instance) = self.runs.current_instance_id() else {
            return Ok(None);
        };
        if packet.starts_with(&[39, 0, 14]) {
            if field(packet, 3) != Some(instance) {
                return Ok(None);
            }
            return self.movie(instance, packet, now_ms, payer).map(Some);
        }
        let start = matches!(packet.get(..3), Some([39, 0, 5]));
        if !matches!(packet.get(..3), Some([39, 0, 5..=7])) && packet != [26, 0, 42, 0] {
            return Ok(None);
        }
        let mut runs = self.runs.clone();
        let mut out = SpotOutcome::from_run(runs.handle_client(packet)?);
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
    ) -> io::Result<SpotOutcome> {
        let text = minigame::movie_text(packet)?;
        let Some(mut play) = self.play else {
            return Ok(SpotOutcome::refused("no spot game is open"));
        };
        let game_id = play.row.game_id;
        // The client's SendPayload names game -1; ours names the game.
        if field(packet, 7) != Some(-1)
            || !matches!(field(packet, 11), Some(id) if id == -1 || id == game_id)
        {
            return Err(invalid("spot game text for another game"));
        }
        let fields: Vec<&str> = text.split('\t').collect();
        // The verdict's acknowledgements, pause reports and trophy counters are harmless at any time.
        if matches!(
            fields.as_slice(),
            ["FRServer_GameClose"]
                | ["FRServer_Pause", "true" | "false"]
                | ["FRServer_StatUpdate", "21" | "23" | "24", "1"]
        ) {
            return Ok(SpotOutcome::ack());
        }
        if self.runs.state(instance) != Some(RunState::Playing) {
            return Ok(SpotOutcome::refused("the spot game is not being played"));
        }
        let out = match (fields.as_slice(), play.stage_sent_ms) {
            (["FRServer_RequestStageId"], None) => {
                // Answered once: a second tier would reload the round.
                play.stage_sent_ms = Some(now_ms);
                let tier = format!("VOnServerSetStageIdMsg\t{}", play.row.stage_id);
                SpotOutcome {
                    accepted: true,
                    replies: vec![payload_packet(instance, game_id, &tier)?],
                    ..SpotOutcome::default()
                }
            }
            (["FRServer_RequestStageId"], Some(_)) => {
                SpotOutcome::refused("the stage was already sent")
            }
            (["FRServer_StatUpdate", "22", "1"], Some(_)) if play.finds < MAX_FINDS => {
                play.finds += 1;
                SpotOutcome::ack()
            }
            (["FRServer_GameWon", score], Some(sent_ms)) => {
                match checked_win(score, &play, now_ms.saturating_sub(sent_ms)) {
                    Ok(score) => self.settle(instance, &play, true, score, now_ms, payer)?,
                    Err(reason) => SpotOutcome {
                        refused: Some(reason),
                        ..self.settle(instance, &play, false, 0.0, now_ms, None)?
                    },
                }
            }
            (["FRServer_GameWon", _], None) => SpotOutcome::refused("a win before the stage"),
            (["FRServer_GameLost", score], _)
                if *score == "undefined" || parse_score(score).is_some() =>
            {
                self.settle(instance, &play, false, 0.0, now_ms, None)?
            }
            _ => SpotOutcome::refused("not a Spot the Difference message"),
        };
        if out.accepted {
            self.play = Some(play);
        }
        Ok(out)
    }

    /// The server's verdict: the goal, then `OnGameWonMsg` / `OnGameLostMsg` and the result.
    fn settle(
        &mut self,
        instance: i32,
        play: &Play,
        won: bool,
        score: f64,
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<SpotOutcome> {
        let elapsed = now_ms.saturating_sub(play.started_ms) / 1000;
        let finish = Finish {
            won,
            // Bounded by `checked_win`; `as` floors a finite score.
            score: score as u32,
            elapsed_seconds: u32::try_from(elapsed).unwrap_or(u32::MAX),
        };
        let content = &self.content;
        // `finish` changes nothing on an error.
        let run = match (won, payer) {
            (true, Some((player, rewards))) => {
                let coins = content.coins(finish.score);
                let grant = win_grant(player, coins, content.icon_id, play.row.name_id);
                self.runs
                    .finish(instance, &finish, Some((rewards, &grant)))?
            }
            (true, None) => self.runs.finish_win_without_reward(instance, &finish)?,
            (false, _) => self.runs.finish(instance, &finish, None)?,
        };
        let mut out = SpotOutcome::from_run(run);
        let mut goal = if won {
            minigame::goal_done_packets(GOAL).to_vec()
        } else {
            minigame::goal_failed_packets(GOAL).to_vec()
        };
        goal.append(&mut out.replies);
        out.replies = goal;
        Ok(out)
    }
}

/// A reported win: the score, if everything the server can hold agrees with it.
fn checked_win(text: &str, play: &Play, since_stage_ms: u64) -> Result<f64, &'static str> {
    let score = parse_score(text).ok_or("the score is not a number")?;
    if score <= 0.0 {
        return Err("a win scores something");
    }
    if play.finds < FINDS_PER_ROUND {
        return Err("no round was finished");
    }
    if score > f64::from(play.finds) * f64::from(play.row.max_find_points) {
        return Err("the score is beyond what the finds allow");
    }
    if since_stage_ms < u64::from(play.finds / FINDS_PER_ROUND) * ROUND_END_MS {
        return Err("the rounds cannot have been played yet");
    }
    Ok(score)
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn content() -> SpotContent {
        SpotContent::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap()
    }

    fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
        let mut packet = vec![39, 0, sub];
        for value in [instance, -1, -1] {
            packet.extend(value.to_le_bytes());
        }
        packet
    }

    fn texts(out: &SpotOutcome) -> Vec<String> {
        (out.replies.iter())
            .filter(|reply| reply.starts_with(&[39, 0, 14]))
            .map(|reply| minigame::movie_text(reply).unwrap().to_owned())
            .collect()
    }

    fn say(game: &mut SpotGame, instance: i32, text: &str, now_ms: u64) -> SpotOutcome {
        let packet = payload_packet(instance, -1, text).unwrap();
        game.route_with(&packet, now_ms, None).unwrap().unwrap()
    }

    /// Game `game_id`, started, its movie told the stage at 1,000 ms.
    fn playing(game_id: i32) -> (SpotGame, i32, SpotOutcome) {
        let mut game = SpotGame::new(content()).unwrap();
        game.begin(game_id).unwrap().unwrap();
        let instance = game.runs.current_instance_id().unwrap();
        game.route_with(&lifecycle(5, instance), 0, None).unwrap();
        let stage = say(&mut game, instance, "FRServer_RequestStageId", 1_000);
        (game, instance, stage)
    }

    fn find(game: &mut SpotGame, instance: i32, count: u32) {
        for _ in 0..count {
            assert!(say(game, instance, "FRServer_StatUpdate\t22\t1", 5_000).accepted);
        }
    }

    fn won(out: &SpotOutcome) -> bool {
        texts(out).iter().any(|text| text == "OnGameWonMsg")
    }

    #[test]
    fn each_row_gets_its_tier_once() {
        for (game_id, tier) in [(1040, 1), (1042, 3), (1044, 5)] {
            let (mut game, instance, stage) = playing(game_id);
            assert_eq!(texts(&stage), [format!("VOnServerSetStageIdMsg\t{tier}")]);
            let twice = say(&mut game, instance, "FRServer_RequestStageId", 1_100);
            assert!(twice.refused.is_some() && twice.replies.is_empty());
        }
    }

    #[test]
    fn nothing_counts_before_the_stage() {
        let mut game = SpotGame::new(content()).unwrap();
        game.begin(1040).unwrap().unwrap();
        let instance = game.runs.current_instance_id().unwrap();
        game.route_with(&lifecycle(5, instance), 0, None).unwrap();
        let find = say(&mut game, instance, "FRServer_StatUpdate\t22\t1", 500);
        assert!(find.refused.is_some());
        let win = say(&mut game, instance, "FRServer_GameWon\t100", 500);
        assert!(win.refused.is_some() && win.replies.is_empty());
    }

    #[test]
    fn a_full_game_wins_with_the_goal() {
        let (mut game, instance, _) = playing(1040);
        find(&mut game, instance, 25);
        let out = say(&mut game, instance, "FRServer_GameWon\t90000", 60_000);
        assert!(won(&out) && out.refused.is_none());
        assert!(
            out.replies
                .iter()
                .any(|reply| reply.starts_with(&[39, 0, 18]))
        );
        let [panel, _] = minigame::goal_done_packets(GOAL);
        assert!(out.replies.contains(&panel));
    }

    /// The movie wins after one finished round when the player quits, so one round stands.
    #[test]
    fn one_finished_round_is_a_win() {
        let (mut game, instance, _) = playing(1044);
        find(&mut game, instance, 5);
        assert!(won(&say(
            &mut game,
            instance,
            "FRServer_GameWon\t20000",
            30_000
        )));
    }

    #[test]
    fn four_finds_is_a_loss() {
        let (mut game, instance, _) = playing(1040);
        find(&mut game, instance, 4);
        let out = say(&mut game, instance, "FRServer_GameWon\t5000", 30_000);
        assert!(!won(&out) && out.refused.is_some());
    }

    #[test]
    fn a_score_past_the_finds_is_a_loss() {
        let (mut game, instance, _) = playing(1040);
        find(&mut game, instance, 5);
        let over = format!("FRServer_GameWon\t{}", 5 * 3_900 + 1);
        let out = say(&mut game, instance, &over, 30_000);
        assert!(!won(&out) && out.refused.is_some());
    }

    #[test]
    fn a_win_too_fast_is_a_loss() {
        let (mut game, instance, _) = playing(1040);
        find(&mut game, instance, 25);
        // Five rounds need 10 s of tweens after the stage at 1 s.
        let out = say(&mut game, instance, "FRServer_GameWon\t90000", 10_999);
        assert!(!won(&out) && out.refused.is_some());
    }

    #[test]
    fn a_zero_score_is_a_loss() {
        let (mut game, instance, _) = playing(1040);
        find(&mut game, instance, 5);
        assert!(!won(&say(
            &mut game,
            instance,
            "FRServer_GameWon\t0",
            30_000
        )));
    }

    #[test]
    fn a_26th_find_is_refused() {
        let (mut game, instance, _) = playing(1040);
        find(&mut game, instance, 25);
        let extra = say(&mut game, instance, "FRServer_StatUpdate\t22\t1", 5_000);
        assert!(extra.refused.is_some());
        assert_eq!(game.play.unwrap().finds, 25);
    }

    #[test]
    fn counters_pause_and_close_are_harmless() {
        let (mut game, instance, _) = playing(1040);
        for text in [
            "FRServer_StatUpdate\t21\t1",
            "FRServer_StatUpdate\t23\t1",
            "FRServer_StatUpdate\t24\t1",
            "FRServer_Pause\ttrue",
            "FRServer_Pause\tfalse",
            "FRServer_GameClose",
            "FRServer_GameClose",
        ] {
            assert!(say(&mut game, instance, text, 2_000).accepted, "{text}");
        }
        assert_eq!(game.play.unwrap().finds, 0);
    }

    #[test]
    fn a_loss_fails_the_goal_and_pays_nothing() {
        let (mut game, instance, _) = playing(1040);
        let out = say(&mut game, instance, "FRServer_GameLost\t0", 20_000);
        let [failed, _] = minigame::goal_failed_packets(GOAL);
        assert!(out.replies.contains(&failed));
        assert!(texts(&out).iter().any(|text| text == "OnGameLostMsg"));
        assert!(out.reward_state.is_none());
    }

    #[test]
    fn coins_follow_the_score_with_a_floor() {
        let content = content();
        assert_eq!(content.coins(9_999), content.win_coins_min);
        assert_eq!(content.coins(90_000), 9 * content.win_coins_per_10000);
    }

    #[test]
    fn invalid_content_is_refused() {
        let broken: [fn(&mut SpotContent); 3] = [
            |c| c.games[0].stage_id = 0,
            |c| c.games[0].stage_id = 6,
            |c| c.games[0].max_find_points = 0,
        ];
        for edit in broken {
            let mut content = content();
            edit(&mut content);
            assert!(content.validate().is_err());
        }
    }
}
