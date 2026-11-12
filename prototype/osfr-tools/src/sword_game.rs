//! Hack 'n' Slash, client minigame type 9 (MiniGameData 79, the Games window row 510): the sword
//! testing movie `game_sword.gfx`, refereed by the server.
//!
//! Same "Mimic" framework and wire as `crate::music_game`, different rules (scripts in the movie;
//! `pieces/hack-n-slash/sheet.md` sections 5 and 7):
//! - Server to client: `OnGameDataMsg <game>`, one `OnPhaseDataMsg <row>` a round, the sword row
//!   `OnCustomDataMsg <row>`, `OnDataComplete`, then the verdict. The movie itself picks each
//!   round's technique from the row's DATA2-5, so the server never learns which one ran.
//! - Client to server: `FRServer_ScoreSet <total>` then `FRServer_CompleteLevel` for a completed
//!   round. A round that times out sends nothing, costs a life and moves on. After the last round
//!   `FRServer_StopLevel <rounds> 1` then `FRServer_GameWon <total> 0`, or at no lives
//!   `FRServer_GameLost undefined` (`SoeInterface.GameLost` drops the score).
//! - Score (`SwordGameClient.applyEndRoundBonus`): time left only, at most 5 s x 100 x the movie's
//!   multiplier of 10 a round. The damage test `isWinner` is always true, so a win is the last
//!   round ended with a life left.
//!
//! Reconstructed (R): every row of `sword_game.json` but its layout (retail's table is lost), the
//! coins, `min_round_ms`, the goal description, and that a loss pays nothing. A win needs its
//! `StopLevel`, a total the completed rounds allow, fewer time-outs than lives, and a loose least
//! time; there is deliberately no upper time bound and no "every round completed" rule, because a
//! pause and a win with time-outs are both legal. Anything else settles a loss.
use crate::{
    minigame::{self, Goal, MinigameRuns, RunMode, RunSpec, RunState},
    music_game::{Finish, MusicOutcome as SwordOutcome, field, parse_score, payload_packet},
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

pub const GAME_TYPE: i32 = 9;
pub const MOVIE: &str = "game_sword.gfx";
pub const CONTENT_FILE: &str = "sword_game.json";
/// The client's preload list. Type 9 is not one the client readies by itself (`009beb70`).
pub const PRELOAD_FILE: &str = "SwordTestingAssets.txt";
/// (R) Run ids of its own, apart from every other minigame's.
pub const FIRST_INSTANCE_ID: i32 = 79_000_001;
/// `SwordTechniques.as`: ids past 22 are unreachable. 0 is an empty candidate.
const MAX_TECHNIQUE: u32 = 22;
/// `game_sword.gfx` sprite 109 has `mcSword1` to `mcSword10`.
const MAX_LIVES: u32 = 10;
/// The most a round can add: 5 s left, in centiseconds, times `SCORE_MULTIPLIER` 10.
const MAX_ROUND_POINTS: u32 = 5_000;
/// `SwordData.as`'s defaults for the sword row's six resistances. No script reads them.
const UNREAD_RESISTANCES: &str = "3^5^5^7^3^4";
/// (R) Every value the movie ignores in a phase row: DURATION 5, OBJECTIVE, START, END, STAGE_TYPE.
const UNREAD_PHASE_COLUMNS: &str = "5^0^0^0^201";
const PRIMARY_GOAL: u32 = 1;
const BONUS_GOAL: u32 = 2;
const MAX_CONTENT: u64 = 16 * 1024;
const MAX_ROUNDS: usize = 32;

/// `sword_game.json`: the one game.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SwordContent {
    pub schema_version: u32,
    /// Provenance note; never gameplay input.
    #[serde(default)]
    pub source: Option<String>,
    pub game_id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub icon_id: i32,
    pub difficulty: i32,
    /// (R) The sword row's ID column.
    pub sword_id: i32,
    /// LIVES of the sword row: a time-out costs one, and at none the game is lost.
    pub lives: u32,
    pub win_coins: u32,
    /// (R) The least a round can take, for the win's time check.
    pub min_round_ms: u32,
    pub primary_goal_name_id: u32,
    pub bonus_goal_name_id: u32,
    pub bonus_goal_description_id: u32,
    pub bonus_score: u32,
    pub rounds: Vec<SwordRound>,
    /// The world's way in: "can be started at any Training Dummy" (Fanbyte).
    pub dummies: Vec<SwordDummy>,
}

/// A Practice Dummy's spawn (`npc_spawns_extra.json`), as a game table's.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SwordDummy {
    pub npc: u64,
    pub position: [f32; 3],
}

/// One round: an `OnPhaseDataMsg` row. Its SEQUENCE is its place in the list, from 1.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SwordRound {
    /// (R) The row's ID column.
    pub phase_id: i32,
    /// DATA1: the Jab count. `RainPoint` ends at exactly 0 life, so 0 could never finish a Jab.
    pub sword_rain: u32,
    /// DATA2-5: the movie picks one non-zero id at random, or any of 1-22 if all are 0.
    pub techniques: [u32; 4],
}

impl SwordContent {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(directory.join(CONTENT_FILE))?
            .take(MAX_CONTENT + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_CONTENT {
            return Err(invalid("sword game content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(invalid)?;
        content.validate()?;
        Ok(content)
    }

    pub fn validate(&self) -> io::Result<()> {
        let ids = [
            self.game_id,
            self.name_id,
            self.description_id,
            self.icon_id,
            self.sword_id,
        ];
        if self.schema_version != 1
            || ids.iter().any(|id| *id <= 0)
            || !(1..=5).contains(&self.difficulty)
            || !(1..=MAX_LIVES).contains(&self.lives)
            || self.primary_goal_name_id == 0
            || self.bonus_goal_name_id == 0
            || self.rounds.is_empty()
            || self.rounds.len() > MAX_ROUNDS
        {
            return Err(invalid("invalid sword game content"));
        }
        let mut phases = BTreeSet::new();
        for round in &self.rounds {
            if round.phase_id <= 0
                || round.sword_rain == 0
                || round.techniques.iter().any(|id| *id > MAX_TECHNIQUE)
                || !phases.insert(round.phase_id)
            {
                return Err(invalid("invalid sword game round"));
            }
        }
        Ok(())
    }

    fn round_count(&self) -> u32 {
        // `validate` holds it to MAX_ROUNDS.
        self.rounds.len() as u32
    }

    /// The `OnPhaseDataMsg` rows, `ID^GAME^SEQUENCE^DURATION^OBJECTIVE_ID^STARTID^ENDID^STAGE_TYPE
    /// ^DATA1..5` (`SwordGameServer.LD_*`), with cooking's trailing `^`.
    fn phase_rows(&self) -> impl Iterator<Item = String> + '_ {
        (self.rounds.iter().zip(1..)).map(|(round, sequence)| {
            let [a, b, c, d] = round.techniques;
            format!(
                "{}^{}^{sequence}^{UNREAD_PHASE_COLUMNS}^{}^{a}^{b}^{c}^{d}^",
                round.phase_id, self.game_id, round.sword_rain
            )
        })
    }

    /// The `OnCustomDataMsg` row, `ID^GAME^LIVES^HELM^SHIELD^BODY_ARMOR^LEGS^HEAD^BODY`.
    fn sword_row(&self) -> String {
        format!(
            "{}^{}^{}^{UNREAD_RESISTANCES}",
            self.sword_id, self.game_id, self.lives
        )
    }

    /// The Goals panel (`2OACNzhHYME_0.00`). No source gives the primary goal a description.
    fn goals(&self) -> Vec<Goal> {
        let goal = |id, name_id, description_id, category| Goal {
            id,
            name_id,
            description_id,
            category,
            bonus: category == 4,
            stars: 0,
            coins: 0,
            total: 0,
            failure_condition: false,
            members_only: false,
        };
        vec![
            goal(PRIMARY_GOAL, self.primary_goal_name_id, 0, 1),
            goal(
                BONUS_GOAL,
                self.bonus_goal_name_id,
                self.bonus_goal_description_id,
                4,
            ),
        ]
    }
}

/// The open run: what the client has reported so far.
#[derive(Clone, Copy, Debug)]
struct Play {
    started_ms: u64,
    /// When the rows went out (`FRServer_ReadyForGameData` answered).
    data_sent_ms: Option<u64>,
    /// The last accepted running total.
    score: f64,
    /// Rounds the movie reported finished, by `FRServer_CompleteLevel`.
    completed: u32,
    /// `FRServer_StopLevel <rounds> 1` arrived.
    stopped: bool,
}

/// One player's game against the server, on the shared minigame lifecycle (offer 39/16, start
/// 39/5, movie text 39/14, result 39/18, close 26/42). Session-local: a reconnect starts fresh.
#[derive(Clone)]
pub struct SwordGame {
    content: SwordContent,
    runs: MinigameRuns,
    play: Option<Play>,
    /// `minigame_rewards.json`'s wheels (C4); empty until the session attaches the table.
    rewards: std::sync::Arc<minigame::MinigameRewards>,
}

impl SwordGame {
    pub fn new(content: SwordContent) -> io::Result<Self> {
        content.validate()?;
        Ok(Self {
            content,
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            play: None,
            rewards: Default::default(),
        })
    }

    /// Attach `minigame_rewards.json`: the game's prize wheel (C4).
    pub fn with_rewards(mut self, rewards: minigame::MinigameRewards) -> Self {
        self.rewards = std::sync::Arc::new(rewards);
        self
    }

    /// The game's wheel, when it has one.
    fn wheel(&self) -> Option<&minigame::Wheel> {
        Some(&self.rewards.game(self.content.game_id)?.wheel).filter(|wheel| wheel.shows())
    }

    /// The MiniGameData id this content offers (the Games window row's id).
    pub fn game_id(&self) -> i32 {
        self.content.game_id
    }

    /// The name on the dummy menu's row, and every dummy that offers it.
    pub fn dummies(&self) -> (u32, &[SwordDummy]) {
        // `validate` holds the id above 0.
        (self.content.name_id as u32, &self.content.dummies)
    }

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    /// Offer the game (39/16); `None` while a run is already open.
    pub fn begin(&mut self) -> io::Result<Option<SwordOutcome>> {
        self.begin_for(&|_| true)
    }

    /// [`Self::begin`] for one player: the wheel's preview lists only the items `fits` accepts (their
    /// inventory defines them and their gender wears them, `Wheel::for_player`), the same rows the
    /// win then lands on.
    pub fn begin_for(&mut self, fits: &dyn Fn(i32) -> bool) -> io::Result<Option<SwordOutcome>> {
        if self.is_active() {
            return Ok(None);
        }
        let content = &self.content;
        let mut spec = RunSpec::empty(
            content.game_id,
            GAME_TYPE,
            RunMode::Flash {
                movie: MOVIE.into(),
            },
        );
        spec.name_id = content.name_id;
        spec.description_id = content.description_id;
        spec.icon_id = content.icon_id;
        spec.difficulty = content.difficulty;
        spec.preload_file = PRELOAD_FILE.into();
        spec.objectives = content.goals();
        if let Some(wheel) = self.wheel().map(|wheel| wheel.keep(fits)) {
            spec.reward_bundles = wheel.offer_bundles();
        }
        // `offer` changes nothing on an error or a refusal.
        let offered = self.runs.offer(spec)?;
        if !offered.accepted {
            return Ok(None);
        }
        self.play = Some(Play {
            started_ms: 0,
            data_sent_ms: None,
            score: 0.0,
            completed: 0,
            stopped: false,
        });
        Ok(Some(SwordOutcome::from_run(offered)))
    }

    /// One client packet while a run is open; `None` for a packet that is not this run's. A
    /// refused or stale packet changes nothing; a malformed one is an error. A win pays `payer`
    /// (player guid and reward state).
    pub fn route_with(
        &mut self,
        packet: &[u8],
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<Option<SwordOutcome>> {
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
        let mut out = SwordOutcome::from_run(runs.handle_client(packet)?);
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
    ) -> io::Result<SwordOutcome> {
        let text = minigame::movie_text(packet)?;
        let Some(mut play) = self.play else {
            return Ok(SwordOutcome::refused("no sword game is open"));
        };
        let game_id = self.content.game_id;
        // The client's SendPayload names game -1; ours names the game.
        if field(packet, 7) != Some(-1)
            || !matches!(field(packet, 11), Some(id) if id == -1 || id == game_id)
        {
            return Err(invalid("sword game text for another game"));
        }
        let fields: Vec<&str> = text.split('\t').collect();
        // Connect, the verdict's acknowledgements and pause reports are harmless at any time.
        if matches!(
            fields.as_slice(),
            ["OnConnectMsg" | "FRServer_Connect" | "FRServer_GameOver" | "FRServer_GameClose"]
                | ["FRServer_Pause", "true" | "false"]
        ) {
            return Ok(SwordOutcome::ack());
        }
        if self.runs.state(instance) != Some(RunState::Playing) {
            return Ok(SwordOutcome::refused("the sword game is not being played"));
        }
        let rounds = self.content.round_count();
        let out = match (fields.as_slice(), play.data_sent_ms) {
            (["FRServer_ReadyForGameData"], None) => {
                // Sent once: a second set would reload the movie's rounds mid-game.
                play.data_sent_ms = Some(now_ms);
                SwordOutcome {
                    accepted: true,
                    replies: self.data_packets(instance)?,
                    ..SwordOutcome::default()
                }
            }
            (["FRServer_ReadyForGameData"], Some(_)) => {
                SwordOutcome::refused("the game's data was already sent")
            }
            (["FRServer_ScoreSet", total], Some(_)) => {
                // The round being scored has not been reported complete yet.
                match checked_total(total, &play, play.completed + 1) {
                    Ok(total) => {
                        play.score = total;
                        SwordOutcome::ack()
                    }
                    Err(reason) => SwordOutcome::refused(reason),
                }
            }
            (["FRServer_CompleteLevel"], Some(_)) if play.completed < rounds => {
                play.completed += 1;
                SwordOutcome::ack()
            }
            (["FRServer_StopLevel", level, "1"], Some(_)) if *level == rounds.to_string() => {
                play.stopped = true;
                SwordOutcome::ack()
            }
            // `isWinner` is always true, so this is unreachable; a loss follows it if it ever is.
            (["FRServer_StopLevel", _, "0"], Some(_)) => SwordOutcome::ack(),
            (["FRServer_GameWon", total, "0"], Some(sent_ms)) => {
                match self.checked_win(total, &play, now_ms.saturating_sub(sent_ms)) {
                    Ok(total) => self.settle(instance, &play, true, total, now_ms, payer)?,
                    Err(reason) => SwordOutcome {
                        refused: Some(reason),
                        ..self.settle(instance, &play, false, play.score, now_ms, None)?
                    },
                }
            }
            (["FRServer_ScoreSet", _] | ["FRServer_GameWon", _, "0"], None) => {
                SwordOutcome::refused("a score before the game's data")
            }
            (["FRServer_GameLost", score], _)
                if *score == "undefined" || parse_score(score).is_some() =>
            {
                self.settle(instance, &play, false, play.score, now_ms, None)?
            }
            _ => SwordOutcome::refused("not a Hack 'n' Slash message"),
        };
        if out.accepted {
            self.play = Some(play);
        }
        Ok(out)
    }

    fn data_packets(&self, instance: i32) -> io::Result<Vec<Vec<u8>>> {
        let content = &self.content;
        let rows = content
            .phase_rows()
            .map(|row| format!("OnPhaseDataMsg\t{row}"));
        let texts = [format!("OnGameDataMsg\t{}", content.game_id)]
            .into_iter()
            .chain(rows)
            .chain([
                format!("OnCustomDataMsg\t{}", content.sword_row()),
                "OnDataComplete".to_owned(),
            ]);
        texts
            .map(|text| payload_packet(instance, content.game_id, &text))
            .collect()
    }

    /// A reported win: the total, if everything the server can hold agrees with it.
    fn checked_win(
        &self,
        total: &str,
        play: &Play,
        since_data_ms: u64,
    ) -> Result<f64, &'static str> {
        let content = &self.content;
        let rounds = content.round_count();
        let total = checked_total(total, play, play.completed)?;
        if !play.stopped {
            return Err("the last round has not ended");
        }
        if rounds - play.completed >= content.lives {
            return Err("more rounds were missed than there are lives");
        }
        if since_data_ms < u64::from(rounds) * u64::from(content.min_round_ms) {
            return Err("the rounds cannot have been played yet");
        }
        Ok(total)
    }

    /// The server's verdict: the goals, then `OnGameWonMsg` / `OnGameLostMsg` and the result.
    fn settle(
        &mut self,
        instance: i32,
        play: &Play,
        won: bool,
        score: f64,
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<SwordOutcome> {
        let elapsed = now_ms.saturating_sub(play.started_ms) / 1000;
        let finish = Finish {
            won,
            // Bounded by `checked_total`; `as` floors a finite total.
            score: score as u32,
            elapsed_seconds: u32::try_from(elapsed).unwrap_or(u32::MAX),
        };
        let wheel = self.wheel().cloned();
        let content = &self.content;
        // `finish` changes nothing on an error.
        let run = match (won, payer, wheel) {
            (true, Some((player, rewards)), wheel) => {
                let grant = win_grant(player, content.win_coins, content.icon_id, content.name_id);
                match wheel {
                    Some(wheel) => (self.runs)
                        .finish_with_wheel(instance, &finish, Some((rewards, &grant)), &wheel)?,
                    None => self.runs.finish(instance, &finish, Some((rewards, &grant)))?,
                }
            }
            (true, None, Some(wheel)) => self.runs.finish_with_wheel(instance, &finish, None, &wheel)?,
            (true, None, None) => self.runs.finish_win_without_reward(instance, &finish)?,
            (false, _, _) => self.runs.finish(instance, &finish, None)?,
        };
        let mut out = SwordOutcome::from_run(run);
        let mut goals = if won {
            minigame::goal_done_packets(PRIMARY_GOAL).to_vec()
        } else {
            minigame::goal_failed_packets(PRIMARY_GOAL).to_vec()
        };
        if won && finish.score >= self.content.bonus_score {
            goals.extend(minigame::goal_done_packets(BONUS_GOAL));
        }
        goals.append(&mut out.replies);
        out.replies = goals;
        Ok(out)
    }
}

/// A reported total: a finite, non-negative number within `rounds` rounds' worth of points, not
/// below the last. An equal one is fine: a round can score nothing.
fn checked_total(text: &str, play: &Play, rounds: u32) -> Result<f64, &'static str> {
    let total = parse_score(text).ok_or("the total is not a number")?;
    if total > f64::from(rounds) * f64::from(MAX_ROUND_POINTS) {
        return Err("the total is beyond what the rounds allow");
    }
    if total < play.score {
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

    fn content() -> SwordContent {
        SwordContent::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap()
    }

    fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
        let mut packet = vec![39, 0, sub];
        for value in [instance, -1, -1] {
            packet.extend(value.to_le_bytes());
        }
        packet
    }

    fn texts(out: &SwordOutcome) -> Vec<String> {
        (out.replies.iter())
            .filter(|reply| reply.starts_with(&[39, 0, 14]))
            .map(|reply| minigame::movie_text(reply).unwrap().to_owned())
            .collect()
    }

    /// A started game with its data sent at 1,000 ms.
    fn playing() -> (SwordGame, i32) {
        let mut game = SwordGame::new(content()).unwrap();
        game.begin().unwrap().unwrap();
        let instance = game.runs.current_instance_id().unwrap();
        game.route_with(&lifecycle(5, instance), 0, None).unwrap();
        say(&mut game, instance, "FRServer_ReadyForGameData", 1_000);
        (game, instance)
    }

    fn say(game: &mut SwordGame, instance: i32, text: &str, now_ms: u64) -> SwordOutcome {
        let packet = payload_packet(instance, -1, text).unwrap();
        game.route_with(&packet, now_ms, None).unwrap().unwrap()
    }

    /// `completed` rounds at `points` each, then the movie's ending, at 60 s.
    fn play_to_end(
        game: &mut SwordGame,
        instance: i32,
        completed: u32,
        points: u32,
    ) -> SwordOutcome {
        for round in 1..=completed {
            let total = round * points;
            for _ in 0..2 {
                let set = say(
                    game,
                    instance,
                    &format!("FRServer_ScoreSet\t{total}"),
                    5_000,
                );
                assert!(set.refused.is_none());
            }
            assert!(say(game, instance, "FRServer_CompleteLevel", 5_000).accepted);
        }
        let rounds = game.content.round_count();
        let stop = format!("FRServer_StopLevel\t{rounds}\t1");
        assert!(say(game, instance, &stop, 59_000).accepted);
        say(
            game,
            instance,
            &format!("FRServer_GameWon\t{}\t0", completed * points),
            60_000,
        )
    }

    fn won(out: &SwordOutcome) -> bool {
        texts(out).iter().any(|text| text == "OnGameWonMsg")
    }

    #[test]
    fn the_rows_go_out_in_the_movies_order() {
        let mut game = SwordGame::new(content()).unwrap();
        game.begin().unwrap().unwrap();
        let instance = game.runs.current_instance_id().unwrap();
        game.route_with(&lifecycle(5, instance), 0, None).unwrap();
        let data = say(&mut game, instance, "FRServer_ReadyForGameData", 1_000);
        assert_eq!(
            texts(&data),
            [
                "OnGameDataMsg\t79",
                "OnPhaseDataMsg\t7901^79^1^5^0^0^0^201^8^1^2^4^0^",
                "OnPhaseDataMsg\t7902^79^2^5^0^0^0^201^8^3^5^8^10^",
                "OnPhaseDataMsg\t7903^79^3^5^0^0^0^201^8^6^9^16^17^",
                "OnPhaseDataMsg\t7904^79^4^5^0^0^0^201^8^11^12^13^14^",
                "OnPhaseDataMsg\t7905^79^5^5^0^0^0^201^8^7^18^19^0^",
                "OnPhaseDataMsg\t7906^79^6^5^0^0^0^201^8^15^20^21^22^",
                "OnCustomDataMsg\t7901^79^3^3^5^5^7^3^4",
                "OnDataComplete",
            ]
        );
        let twice = say(&mut game, instance, "FRServer_ReadyForGameData", 1_100);
        assert!(twice.refused.is_some() && twice.replies.is_empty());
    }

    #[test]
    fn a_full_game_wins_with_both_goals() {
        let (mut game, instance) = playing();
        let out = play_to_end(&mut game, instance, 6, 3_000);
        assert!(won(&out) && out.refused.is_none());
        assert!(
            out.replies
                .iter()
                .any(|reply| reply.starts_with(&[39, 0, 18]))
        );
        for goal in [PRIMARY_GOAL, BONUS_GOAL] {
            let [panel, _] = minigame::goal_done_packets(goal);
            assert!(out.replies.contains(&panel), "goal {goal}");
        }
    }

    #[test]
    fn a_low_score_leaves_the_bonus_goal_open() {
        let (mut game, instance) = playing();
        let out = play_to_end(&mut game, instance, 6, 2_000);
        assert!(won(&out));
        let [bonus, _] = minigame::goal_done_packets(BONUS_GOAL);
        assert!(!out.replies.contains(&bonus));
    }

    /// Two time-outs leave a life, and a time-out sends nothing, so this is a legal win.
    #[test]
    fn a_win_with_two_time_outs_stands() {
        let (mut game, instance) = playing();
        assert!(won(&play_to_end(&mut game, instance, 4, 3_000)));
    }

    #[test]
    fn a_forged_win_with_three_time_outs_is_a_loss() {
        let (mut game, instance) = playing();
        let out = play_to_end(&mut game, instance, 3, 3_000);
        assert!(!won(&out) && out.refused.is_some());
        assert!(texts(&out).iter().any(|text| text == "OnGameLostMsg"));
    }

    #[test]
    fn a_win_with_no_stop_level_is_a_loss() {
        let (mut game, instance) = playing();
        let out = say(&mut game, instance, "FRServer_GameWon\t0\t0", 60_000);
        assert!(!won(&out) && out.refused.is_some());
    }

    #[test]
    fn a_win_too_soon_is_a_loss() {
        let (mut game, instance) = playing();
        for _ in 0..6 {
            say(&mut game, instance, "FRServer_CompleteLevel", 2_000);
        }
        say(&mut game, instance, "FRServer_StopLevel\t6\t1", 2_000);
        let out = say(&mut game, instance, "FRServer_GameWon\t0\t0", 2_000);
        assert!(!won(&out) && out.refused.is_some());
    }

    #[test]
    fn a_total_past_the_rounds_or_below_the_last_is_refused() {
        let (mut game, instance) = playing();
        let jump = say(&mut game, instance, "FRServer_ScoreSet\t5001", 2_000);
        assert!(jump.refused.is_some());
        assert!(
            say(&mut game, instance, "FRServer_ScoreSet\t4000", 2_000)
                .refused
                .is_none()
        );
        let down = say(&mut game, instance, "FRServer_ScoreSet\t3000", 2_000);
        assert!(down.refused.is_some());
        // The refusal left the run as it was.
        assert_eq!(game.play.unwrap().score, 4_000.0);
    }

    #[test]
    fn a_win_above_what_the_completed_rounds_allow_is_a_loss() {
        let (mut game, instance) = playing();
        for _ in 0..4 {
            say(&mut game, instance, "FRServer_CompleteLevel", 5_000);
        }
        say(&mut game, instance, "FRServer_StopLevel	6	1", 59_000);
        let out = say(&mut game, instance, "FRServer_GameWon	20001	0", 60_000);
        assert!(!won(&out) && out.refused.is_some());
    }

    #[test]
    fn a_level_too_many_or_the_wrong_stop_changes_nothing() {
        let (mut game, instance) = playing();
        for _ in 0..6 {
            assert!(say(&mut game, instance, "FRServer_CompleteLevel", 2_000).accepted);
        }
        let extra = say(&mut game, instance, "FRServer_CompleteLevel", 2_000);
        assert!(extra.refused.is_some());
        let stop = say(&mut game, instance, "FRServer_StopLevel\t5\t1", 2_000);
        assert!(stop.refused.is_some() && !game.play.unwrap().stopped);
    }

    #[test]
    fn a_pause_mid_round_and_a_long_game_are_legal() {
        let (mut game, instance) = playing();
        assert!(say(&mut game, instance, "FRServer_Pause\ttrue", 2_000).accepted);
        assert!(say(&mut game, instance, "FRServer_Pause\tfalse", 2_000).accepted);
        for _ in 0..6 {
            say(&mut game, instance, "FRServer_CompleteLevel", 900_000);
        }
        say(&mut game, instance, "FRServer_StopLevel\t6\t1", 900_000);
        assert!(won(&say(
            &mut game,
            instance,
            "FRServer_GameWon\t0\t0",
            900_000
        )));
    }

    #[test]
    fn a_loss_fails_the_primary_goal_and_pays_nothing() {
        let (mut game, instance) = playing();
        let out = say(&mut game, instance, "FRServer_GameLost\tundefined", 20_000);
        let [failed, _] = minigame::goal_failed_packets(PRIMARY_GOAL);
        assert!(out.replies.contains(&failed));
        assert!(texts(&out).iter().any(|text| text == "OnGameLostMsg"));
        assert!(out.reward_state.is_none());
    }

    #[test]
    fn invalid_content_is_refused() {
        let broken: [fn(&mut SwordContent); 4] = [
            |c| c.lives = 0,
            |c| c.lives = 11,
            |c| c.rounds[0].sword_rain = 0,
            |c| c.rounds[0].techniques[0] = 23,
        ];
        for edit in broken {
            let mut content = content();
            edit(&mut content);
            assert!(content.validate().is_err());
        }
    }
}
