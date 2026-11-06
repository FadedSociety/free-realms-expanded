//! L123: "Feeling the Beat!", client minigame type 9 (MiniGameData 201-204, the Games window rows
//! 511-513 and 568): the music conducting movie `game_music.gfx`, refereed by the server.
//!
//! Recovered from the movie's own scripts (`MusicConducting.*` on the shared "Mimic" framework;
//! JPEXS export in the lab, protocol note L123):
//! - The movie's bundled `GameServer` runs inside the client (`SoeClient.InitializeNetworking`
//!   forces its local network), so its level, score and next-level messages never reach us. The
//!   wire carries `GameExternalNetwork`'s tab-separated text, the framework cooking speaks.
//! - Client to server: `OnConnectMsg`, `FRServer_ReadyForGameData`, `FRServer_Pause <bool>`, the
//!   running total `FRServer_ScoreSet <n>` every scoring frame, then `FRServer_GameWon <n> 0` when
//!   the song ends (`MusicGameClient.showFinalFeedback`) or `FRServer_GameLost undefined` when the
//!   life bar or the last track gives out (`onLose`; `SoeInterface.GameLost` drops the score),
//!   and `FRServer_GameOver` after the verdict.
//! - Server to client: `OnGameDataMsg <game>`, the level row `OnPhaseDataMsg <row>`,
//!   `OnDataComplete`, then the verdict `OnGameWonMsg` / `OnGameLostMsg`. A row's STAGE_TYPE - 300
//!   picks Song1-5 (`MusicGameServer.OnLevelData`); the client plays the first level only.
//! - Scoring (`MusicRoundPattern.setItemsWithTime`): a frame adds at most the top combo 4 times
//!   the full closeness 150, plus two combo bonuses of at most 32; the total never goes down.
//!
//! Reconstructed (R): difficulty N plays SongN (the row and song names match), the phase ids, the
//! row's DURATION (the song's seconds; the movie ignores it) and the goal 0 (the movie sets none);
//! the score bound counts twice the movie's 30 frames a second over the song; a win needs the
//! song's length to have passed since the row went out, a total within the bound and not below
//! the last report, and the level's goal, else the server settles a loss; a refused report
//! changes nothing; a win pays [`WIN_COINS`] (L961) with the result (stars 0).
use crate::{
    minigame::{
        self, Activity, ActivityFinish, MinigameRuns, RunMode, RunOutcome, RunSpec, RunState,
    },
    rewards::{RewardDisplay, RewardGrant, RewardState},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
};

pub const GAME_TYPE: i32 = 9;
pub const MOVIE: &str = "game_music.gfx";
pub const CONTENT_FILE: &str = "music_game.json";
/// L961: the client's preload list for the movie (manifest `MusicConductingAssets.txt.z`; the movie's
/// own name is MusicConducting). Type 9 is not one the client readies by itself (`009beb70`: 1, 2,
/// 17, 22), so without it the start screen stays on "Loading..." (native 2026-11-04), as checkers
/// did before L918. `MoreMusicConductingAssets.txt` is the other list the manifest carries (R).
pub const PRELOAD_FILE: &str = "MusicConductingAssets.txt";
/// (R) Run ids of its own, apart from every other minigame's.
pub const FIRST_INSTANCE_ID: i32 = 201_000_001;
/// L961 (R): a won song pays checkers' award (L920), so no Games window game pays nothing.
// ponytail: one constant for every song; move it to `music_game.json` rows with other tuning.
pub const WIN_COINS: u32 = 10;
const MAX_CONTENT: u64 = 16 * 1024;
const MAX_GAMES: usize = 16;
const MAX_TEXT: usize = 256;
/// `LastTime` (ms) of `MusicConducting.Songs.Song1` to `Song5`.
const SONG_MS: [u32; 5] = [37_000, 40_000, 49_500, 55_000, 58_000];
/// `MusicGameServer.mStageTypeOffset`: stage type 301 plays Song1.
const STAGE_TYPE_OFFSET: i32 = 300;
/// The most one scoring frame adds: top combo 4 x closeness 150, plus two combo bonuses of 32.
const MAX_FRAME_POINTS: u64 = 4 * 150 + 2 * 32;
/// (R) Twice the movie's 30 frames a second, for frame timing slack.
// ponytail: loose bound (about 2x perfect play); simulate each song's ComboTargets to tighten it.
const BOUND_FPS: u64 = 60;

/// `music_game.json`: one Feeling the Beat game per MiniGameData row.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MusicContent {
    pub schema_version: u32,
    /// Provenance note; never gameplay input.
    #[serde(default)]
    pub source: Option<String>,
    pub description_id: i32,
    pub icon_id: i32,
    pub games: Vec<MusicGameRow>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MusicGameRow {
    pub game_id: i32,
    pub name_id: i32,
    pub difficulty: i32,
    /// (R) The phase row's ID column.
    pub phase_id: i32,
    /// 300 + the song (1-5) the movie plays.
    pub stage_type: i32,
    /// (R) The least final total that wins; the movie itself sets none (0).
    pub goal: u32,
}

impl MusicContent {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(directory.join(CONTENT_FILE))?
            .take(MAX_CONTENT + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_CONTENT {
            return Err(invalid("music game content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(invalid)?;
        content.validate()?;
        Ok(content)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1
            || self.description_id <= 0
            || self.icon_id <= 0
            || self.games.is_empty()
            || self.games.len() > MAX_GAMES
        {
            return Err(invalid("invalid music game content"));
        }
        let mut seen = BTreeSet::new();
        for row in &self.games {
            let level = row
                .level()
                .ok_or_else(|| invalid("music game stage type must be 301 to 305"))?;
            if row.game_id <= 0
                || row.name_id <= 0
                || row.phase_id <= 0
                || !(1..=5).contains(&row.difficulty)
                || !seen.insert(row.game_id)
                || f64::from(row.goal) > level.max_score
            {
                return Err(invalid("invalid music game row"));
            }
        }
        Ok(())
    }

    fn game(&self, game_id: i32) -> Option<&MusicGameRow> {
        self.games.iter().find(|row| row.game_id == game_id)
    }
}

/// One game's level: its row, its song's length and the highest total that song allows.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Level {
    row: MusicGameRow,
    song_ms: u32,
    max_score: f64,
}

impl MusicGameRow {
    fn level(&self) -> Option<Level> {
        let song = usize::try_from(self.stage_type.checked_sub(STAGE_TYPE_OFFSET + 1)?).ok()?;
        let song_ms = *SONG_MS.get(song)?;
        let frames = (u64::from(song_ms) * BOUND_FPS).div_ceil(1000);
        Some(Level {
            row: *self,
            song_ms,
            max_score: (frames * MAX_FRAME_POINTS) as f64,
        })
    }
}

impl Level {
    /// The `OnPhaseDataMsg` row: ID^GAME^SEQUENCE^DURATION^OBJECTIVE_ID^STARTID^ENDID^STAGE_TYPE
    /// ^DATA1..4 (`MusicGameServer.LD_*`), with cooking's trailing `^`.
    fn phase_row(&self) -> String {
        let row = self.row;
        format!(
            "{}^{}^1^{}^0^0^0^{}^0^0^0^0^",
            row.phase_id,
            row.game_id,
            self.song_ms.div_ceil(1000),
            row.stage_type
        )
    }
}

/// What one packet did, for gameplay to apply. `changed` marks a lifecycle step worth saving;
/// score reports never set it (the run is session-local).
#[derive(Default)]
pub struct MusicOutcome {
    pub accepted: bool,
    pub changed: bool,
    pub replies: Vec<Vec<u8>>,
    /// Why a client report was refused (nothing changed) or a reported win was overruled.
    pub refused: Option<&'static str>,
    /// L961: a won song's payout, to adopt before sending `reward_packets`, as for checkers.
    pub reward_packets: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
}

impl MusicOutcome {
    pub(crate) fn from_run(run: RunOutcome) -> Self {
        Self {
            accepted: run.accepted,
            changed: run.changed,
            replies: run.replies,
            refused: None,
            reward_packets: run.reward_packets,
            reward_state: run.reward_state,
        }
    }

    pub(crate) fn ack() -> Self {
        Self {
            accepted: true,
            ..Self::default()
        }
    }

    pub(crate) fn refused(reason: &'static str) -> Self {
        Self {
            refused: Some(reason),
            ..Self::default()
        }
    }
}

/// The open run's level and what the client has reported so far.
#[derive(Clone, Copy, Debug)]
struct Play {
    level: Level,
    started_ms: u64,
    /// When the level row went out (`FRServer_ReadyForGameData` answered).
    data_sent_ms: Option<u64>,
    /// The last accepted running total.
    score: f64,
}

pub(crate) struct Finish {
    pub(crate) won: bool,
    pub(crate) score: u32,
    pub(crate) elapsed_seconds: u32,
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

/// One player's Feeling the Beat run against the server, on the shared minigame lifecycle
/// (offer 39/16, start 39/5, movie text 39/14, result 39/18, close 26/42). Session-local like a
/// checkers table: a reconnect starts fresh.
#[derive(Clone)]
pub struct MusicGame {
    content: MusicContent,
    runs: MinigameRuns,
    play: Option<Play>,
}

impl MusicGame {
    pub fn new(content: MusicContent) -> io::Result<Self> {
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

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    /// Offer game `game_id` (39/16); `None` while a run is already open. An id the content does
    /// not carry is an error.
    pub fn begin(&mut self, game_id: i32) -> io::Result<Option<MusicOutcome>> {
        let row = *self
            .content
            .game(game_id)
            .ok_or_else(|| invalid("not a Feeling the Beat game"))?;
        let level = row
            .level()
            .ok_or_else(|| invalid("invalid music game row"))?;
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
        // `offer` changes nothing on an error or a refusal.
        let offered = self.runs.offer(spec)?;
        if !offered.accepted {
            return Ok(None);
        }
        self.play = Some(Play {
            level,
            started_ms: 0,
            data_sent_ms: None,
            score: 0.0,
        });
        Ok(Some(MusicOutcome::from_run(offered)))
    }

    /// One client packet while a run is open; `None` for a packet that is not this run's. A
    /// refused or stale packet changes nothing; a malformed one is an error.
    pub fn route(&mut self, packet: &[u8], now_ms: u64) -> io::Result<Option<MusicOutcome>> {
        self.route_with(packet, now_ms, None)
    }

    /// `route`, where a won song pays [`WIN_COINS`] to `payer` (player guid and reward state).
    pub fn route_with(
        &mut self,
        packet: &[u8],
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<Option<MusicOutcome>> {
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
        let mut out = MusicOutcome::from_run(runs.handle_client(packet)?);
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
    ) -> io::Result<MusicOutcome> {
        let text = minigame::movie_text(packet)?;
        let Some(mut play) = self.play else {
            return Ok(MusicOutcome::refused("no music game is open"));
        };
        let game_id = play.level.row.game_id;
        // The client's SendPayload names game -1; ours names the game.
        if field(packet, 7) != Some(-1)
            || !matches!(field(packet, 11), Some(id) if id == -1 || id == game_id)
        {
            return Err(invalid("music game text for another game"));
        }
        let fields: Vec<&str> = text.split('\t').collect();
        // Connect, the verdict's acknowledgements and pause reports are harmless at any time.
        if matches!(
            fields.as_slice(),
            ["OnConnectMsg" | "FRServer_GameOver" | "FRServer_GameClose"]
                | ["FRServer_Pause", "true" | "false"]
        ) {
            return Ok(MusicOutcome::ack());
        }
        if self.runs.state(instance) != Some(RunState::Playing) {
            return Ok(MusicOutcome::refused("the music game is not being played"));
        }
        let out = match (fields.as_slice(), play.data_sent_ms) {
            (["FRServer_ReadyForGameData"], None) => {
                // Sent once: a second row set would reload the movie's levels mid-song.
                let texts = [
                    format!("OnGameDataMsg\t{game_id}"),
                    format!("OnPhaseDataMsg\t{}", play.level.phase_row()),
                    "OnDataComplete".to_owned(),
                ];
                let replies = (texts.iter())
                    .map(|text| payload_packet(instance, game_id, text))
                    .collect::<io::Result<_>>()?;
                play.data_sent_ms = Some(now_ms);
                MusicOutcome {
                    accepted: true,
                    replies,
                    ..MusicOutcome::default()
                }
            }
            (["FRServer_ReadyForGameData"], Some(_)) => {
                MusicOutcome::refused("the song's data was already sent")
            }
            (["FRServer_ScoreSet", total], Some(_)) => match checked_total(total, &play) {
                Ok(total) => {
                    play.score = total;
                    MusicOutcome::ack()
                }
                Err(reason) => MusicOutcome::refused(reason),
            },
            (["FRServer_GameWon", total, "0"], Some(sent_ms)) => {
                let verdict = checked_total(total, &play).and_then(|total| {
                    if now_ms.saturating_sub(sent_ms) < u64::from(play.level.song_ms) {
                        Err("the song cannot have ended yet")
                    } else {
                        Ok(total)
                    }
                });
                match verdict {
                    Ok(total) => {
                        let won = total >= f64::from(play.level.row.goal);
                        self.settle(instance, &play, won, total, now_ms, payer)?
                    }
                    Err(reason) => MusicOutcome {
                        refused: Some(reason),
                        ..self.settle(instance, &play, false, play.score, now_ms, None)?
                    },
                }
            }
            (["FRServer_ScoreSet", _] | ["FRServer_GameWon", _, "0"], None) => {
                MusicOutcome::refused("a score before the song's data")
            }
            (["FRServer_GameLost", score], _)
                if *score == "undefined" || parse_score(score).is_some() =>
            {
                self.settle(instance, &play, false, play.score, now_ms, None)?
            }
            _ => MusicOutcome::refused("not a Feeling the Beat message"),
        };
        if out.accepted {
            self.play = Some(play);
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
    ) -> io::Result<MusicOutcome> {
        let elapsed = now_ms.saturating_sub(play.started_ms) / 1000;
        let finish = Finish {
            won,
            // Clamped to the song's bound, which fits an i32; `as` floors a finite total.
            score: score.min(play.level.max_score) as u32,
            elapsed_seconds: u32::try_from(elapsed).unwrap_or(u32::MAX),
        };
        // `finish` changes nothing on an error.
        let run = match (won, payer) {
            (true, Some((player, rewards))) => {
                let grant = win_grant(player, self.content.icon_id, play.level.row.name_id);
                self.runs
                    .finish(instance, &finish, Some((rewards, &grant)))?
            }
            (true, None) => self.runs.finish_win_without_reward(instance, &finish)?,
            (false, _) => self.runs.finish(instance, &finish, None)?,
        };
        Ok(MusicOutcome::from_run(run))
    }
}

fn win_grant(player_id: u64, icon_id: i32, name_id: i32) -> RewardGrant {
    RewardGrant {
        coins: WIN_COINS,
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

/// A reported total: a finite, non-negative number within the song's bound, not below the last.
fn checked_total(text: &str, play: &Play) -> Result<f64, &'static str> {
    let total = parse_score(text).ok_or("the total is not a number")?;
    if total > play.level.max_score {
        return Err("the total is beyond what the song allows");
    }
    if total < play.score {
        return Err("the total went down");
    }
    Ok(total)
}

/// AS2 `Number.toString` of a total: a finite, non-negative decimal.
pub(crate) fn parse_score(text: &str) -> Option<f64> {
    text.parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value >= 0.0)
}

/// 39/14: instance, the -1 group, the game id, then the NUL-terminated text.
pub(crate) fn payload_packet(instance: i32, game_id: i32, text: &str) -> io::Result<Vec<u8>> {
    if text.len() > MAX_TEXT || text.contains('\0') {
        return Err(invalid("music game payload text"));
    }
    let length = i32::try_from(text.len() + 1).map_err(invalid)?;
    let mut packet = vec![39, 0, 14];
    for value in [instance, -1, game_id, length] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(text.as_bytes());
    packet.push(0);
    Ok(packet)
}

/// The little-endian i32 at `at`, if the packet is long enough.
pub(crate) fn field(packet: &[u8], at: usize) -> Option<i32> {
    let bytes: [u8; 4] = packet.get(at..at + 4)?.try_into().ok()?;
    Some(i32::from_le_bytes(bytes))
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn content() -> MusicContent {
        MusicContent::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap()
    }

    fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
        let mut packet = vec![39, 0, sub];
        for value in [instance, -1, -1] {
            packet.extend(value.to_le_bytes());
        }
        packet
    }

    fn texts(out: &MusicOutcome) -> Vec<String> {
        (out.replies.iter())
            .filter(|p| p.starts_with(&[39, 0, 14]))
            .map(|p| String::from_utf8_lossy(&p[19..p.len() - 1]).into_owned())
            .collect()
    }

    fn result(out: &MusicOutcome) -> &[u8] {
        (out.replies.iter())
            .find(|p| p.starts_with(&[39, 0, 18]))
            .expect("39/18")
    }

    /// Offer `game` with `music`, start it at t=1s; the run's instance.
    fn start(music: &mut MusicGame, game: i32) -> i32 {
        let offer = music.begin(game).unwrap().expect("offered");
        assert!(offer.replies[0].starts_with(&[39, 0, 16]));
        let instance = i32::from_le_bytes(offer.replies[0][3..7].try_into().unwrap());
        assert!(music.begin(game).unwrap().is_none(), "one run at a time");
        let started = music
            .route(&lifecycle(5, instance), 1_000)
            .unwrap()
            .unwrap();
        assert!(started.accepted && started.changed);
        assert!(
            (started.replies.iter()).any(|p| p.starts_with(&[26, 0, 12, 0])
                && p.windows(MOVIE.len()).any(|w| w == MOVIE.as_bytes())),
            "the music movie launches"
        );
        instance
    }

    fn say(music: &mut MusicGame, instance: i32, text: &str, now_ms: u64) -> MusicOutcome {
        let packet = payload_packet(instance, -1, text).unwrap();
        music
            .route(&packet, now_ms)
            .unwrap()
            .expect("the run's text")
    }

    #[test]
    fn shipped_content_maps_the_four_rows_to_their_songs() {
        let music = MusicGame::new(content()).unwrap();
        assert_eq!(music.game_ids().collect::<Vec<_>>(), [201, 202, 203, 204]);
        let level = |game| music.content.game(game).unwrap().level().unwrap();
        assert_eq!(level(201).phase_row(), "20101^201^1^37^0^0^0^301^0^0^0^0^");
        assert_eq!(level(203).phase_row(), "20301^203^1^50^0^0^0^303^0^0^0^0^");
        assert_eq!(level(204).song_ms, 55_000);
        // 37 s at 60 frames a second, 664 points a frame.
        assert_eq!(level(201).max_score, 1_474_080.0);
    }

    #[test]
    fn content_out_of_bounds_is_refused() {
        let good = content();
        let mut bad = good.clone();
        bad.games[0].stage_type = 306;
        assert!(bad.validate().is_err(), "no Song6");
        let mut bad = good.clone();
        bad.games[1].game_id = 201;
        assert!(bad.validate().is_err(), "duplicate game");
        let mut bad = good.clone();
        bad.games[2].goal = 2_000_000;
        assert!(bad.validate().is_err(), "a goal no song can reach");
        let mut bad = good;
        bad.games[3].difficulty = 6;
        assert!(bad.validate().is_err());
        assert!(
            serde_json::from_str::<MusicContent>(
                r#"{"schema_version":1,"description_id":1,"icon_id":1,"games":[],"extra":1}"#
            )
            .is_err(),
            "unknown fields are refused"
        );
        let mut music = MusicGame::new(content()).unwrap();
        assert!(music.begin(205).is_err(), "not in this content");
        assert!(music.begin(-1).is_err());
    }

    #[test]
    fn a_full_won_game() {
        let mut music = MusicGame::new(content()).unwrap();
        let instance = start(&mut music, 201);
        let connect = say(&mut music, instance, "OnConnectMsg", 1_500);
        assert!(connect.accepted && connect.replies.is_empty());
        let data = say(&mut music, instance, "FRServer_ReadyForGameData", 2_000);
        assert_eq!(
            texts(&data),
            [
                "OnGameDataMsg\t201",
                "OnPhaseDataMsg\t20101^201^1^37^0^0^0^301^0^0^0^0^",
                "OnDataComplete"
            ]
        );
        assert!(
            data.replies
                .iter()
                .all(|p| p[11..15] == 201_i32.to_le_bytes())
        );
        for (text, at) in [
            ("FRServer_ScoreSet\t0", 10_000),
            ("FRServer_Pause\ttrue", 11_000),
            ("FRServer_Pause\tfalse", 12_000),
            ("FRServer_ScoreSet\t1234.56789", 20_000),
        ] {
            let out = say(&mut music, instance, text, at);
            assert!(
                out.accepted && !out.changed && out.replies.is_empty(),
                "{text}"
            );
        }
        let won = say(
            &mut music,
            instance,
            "FRServer_GameWon\t1234.56789\t0",
            40_000,
        );
        assert_eq!(won.refused, None);
        assert!(won.changed);
        assert_eq!(texts(&won), ["OnGameWonMsg"]);
        let screen = result(&won);
        assert_eq!(screen[15], 1, "won");
        assert_eq!(
            &screen[20..24],
            &39_i32.to_le_bytes(),
            "seconds since the start"
        );
        assert_eq!(
            &screen[24..28],
            &1234_i32.to_le_bytes(),
            "the floored total"
        );
        assert!(won.replies.iter().any(|p| p.starts_with(&[39, 0, 19])));
        assert!(say(&mut music, instance, "FRServer_GameOver", 41_000).accepted);
        let late = say(&mut music, instance, "FRServer_ScoreSet\t2000", 41_100);
        assert!(late.refused.is_some(), "nothing counts after the verdict");
        let again = say(
            &mut music,
            instance,
            "FRServer_GameWon\t1234.56789\t0",
            41_200,
        );
        assert!(again.replies.is_empty(), "the verdict is given once");
        let close = music.route(&[26, 0, 42, 0], 45_000).unwrap().unwrap();
        assert!(close.accepted && close.changed);
        assert!(!music.is_active());
        assert!(music.route(&[26, 0, 42, 0], 45_100).unwrap().is_none());
        assert!(
            music.begin(202).unwrap().is_some(),
            "a new run after the close"
        );
    }

    #[test]
    fn a_lost_game_and_a_win_below_the_goal() {
        let mut music = MusicGame::new(content()).unwrap();
        let instance = start(&mut music, 203);
        say(&mut music, instance, "FRServer_ReadyForGameData", 2_000);
        assert!(say(&mut music, instance, "FRServer_ScoreSet\t500.5", 9_000).accepted);
        let lost = say(&mut music, instance, "FRServer_GameLost\tundefined", 12_000);
        assert_eq!(texts(&lost), ["OnGameLostMsg"]);
        let screen = result(&lost);
        assert_eq!(screen[15], 0, "lost");
        assert_eq!(&screen[24..28], &500_i32.to_le_bytes());

        // A tuned goal: a finished song below it is the server's loss.
        let mut tuned = content();
        tuned.games[1].goal = 1_000;
        let mut music = MusicGame::new(tuned).unwrap();
        let instance = start(&mut music, 202);
        say(&mut music, instance, "FRServer_ReadyForGameData", 2_000);
        let below = say(&mut music, instance, "FRServer_GameWon\t999\t0", 60_000);
        assert_eq!(below.refused, None, "a fair loss, not a refusal");
        assert_eq!(texts(&below), ["OnGameLostMsg"]);
        assert_eq!(result(&below)[15], 0);
    }

    #[test]
    fn an_out_of_bounds_score_is_refused() {
        let mut music = MusicGame::new(content()).unwrap();
        let instance = start(&mut music, 201);
        say(&mut music, instance, "FRServer_ReadyForGameData", 2_000);
        assert!(say(&mut music, instance, "FRServer_ScoreSet\t100", 5_000).accepted);
        for total in ["1474080.5", "-1", "NaN", "inf", "1e400", "99", "abc", ""] {
            let out = say(
                &mut music,
                instance,
                &format!("FRServer_ScoreSet\t{total}"),
                6_000,
            );
            assert!(!out.accepted && out.refused.is_some(), "{total}");
            assert_eq!(music.play.unwrap().score, 100.0, "{total} changed nothing");
        }
        assert!(say(&mut music, instance, "FRServer_ScoreSet\t1474080", 7_000).accepted);

        // A win reported too early is overruled into a loss.
        let early = say(&mut music, instance, "FRServer_GameWon\t1474080\t0", 30_000);
        assert_eq!(early.refused, Some("the song cannot have ended yet"));
        assert_eq!(result(&early)[15], 0);

        // So is an impossible final total, with the last fair total on the result.
        let mut music = MusicGame::new(content()).unwrap();
        let instance = start(&mut music, 201);
        say(&mut music, instance, "FRServer_ReadyForGameData", 2_000);
        say(&mut music, instance, "FRServer_ScoreSet\t300", 20_000);
        let cheat = say(&mut music, instance, "FRServer_GameWon\t9999999\t0", 50_000);
        assert_eq!(
            cheat.refused,
            Some("the total is beyond what the song allows")
        );
        assert_eq!(texts(&cheat), ["OnGameLostMsg"]);
        assert_eq!(&result(&cheat)[24..28], &300_i32.to_le_bytes());
    }

    #[test]
    fn a_message_in_the_wrong_phase_is_refused() {
        let mut music = MusicGame::new(content()).unwrap();
        let offer = music.begin(204).unwrap().unwrap();
        let instance = i32::from_le_bytes(offer.replies[0][3..7].try_into().unwrap());
        let early = say(&mut music, instance, "FRServer_ReadyForGameData", 500);
        assert!(
            early.refused.is_some() && early.replies.is_empty(),
            "before the start"
        );
        music.route(&lifecycle(5, instance), 1_000).unwrap();
        let score = say(&mut music, instance, "FRServer_ScoreSet\t10", 1_500);
        assert_eq!(score.refused, Some("a score before the song's data"));
        let win = say(&mut music, instance, "FRServer_GameWon\t0\t0", 90_000);
        assert!(
            win.refused.is_some() && win.replies.is_empty(),
            "no verdict before the data"
        );
        let data = say(&mut music, instance, "FRServer_ReadyForGameData", 2_000);
        assert_eq!(texts(&data).len(), 3);
        let twice = say(&mut music, instance, "FRServer_ReadyForGameData", 2_100);
        assert!(twice.refused.is_some() && twice.replies.is_empty());
        // The movie never sends these; they change nothing.
        for text in [
            "FRServer_CompleteLevel",
            "FRServer_StopLevel\t1\t1",
            "OnStartGameMsg",
        ] {
            assert!(
                say(&mut music, instance, text, 3_000).refused.is_some(),
                "{text}"
            );
        }
        // Quitting mid-song (39/6) ends the run.
        let quit = music
            .route(&lifecycle(6, instance), 4_000)
            .unwrap()
            .unwrap();
        assert!(quit.accepted && quit.replies[0].starts_with(&[39, 0, 19]));
        assert!(!music.is_active());
    }

    #[test]
    fn malformed_text_is_an_error_and_garbled_text_is_refused() {
        let mut music = MusicGame::new(content()).unwrap();
        let instance = start(&mut music, 202);
        say(&mut music, instance, "FRServer_ReadyForGameData", 2_000);
        let good = payload_packet(instance, -1, "FRServer_ScoreSet\t5").unwrap();
        let mut long = good.clone();
        long[15] += 1;
        let mut no_nul = good.clone();
        *no_nul.last_mut().unwrap() = b'x';
        let mut not_utf8 = payload_packet(instance, -1, "ab").unwrap();
        not_utf8[19] = 0xff;
        let mut group = good.clone();
        group[7..11].copy_from_slice(&5_i32.to_le_bytes());
        let mut game = good.clone();
        game[11..15].copy_from_slice(&999_i32.to_le_bytes());
        for packet in [
            long,
            no_nul,
            not_utf8,
            group,
            game,
            good[..17].to_vec(),
            lifecycle(5, instance)[..10].to_vec(),
        ] {
            assert!(music.route(&packet, 3_000).is_err(), "{packet:?}");
        }
        let other = payload_packet(instance + 1, -1, "FRServer_ScoreSet\t5").unwrap();
        assert!(
            music.route(&other, 3_000).unwrap().is_none(),
            "another run's text"
        );
        for text in [
            "FRServer_GameWon\t5",
            "FRServer_GameWon\t5\t1",
            "FRServer_GameLost\tlots",
            "FRServer_Pause\tmaybe",
            "FRServer_ScoreSet\t5\t6",
            "Hello",
        ] {
            let out = say(&mut music, instance, text, 3_000);
            assert!(!out.accepted && out.replies.is_empty(), "{text}");
        }
        assert!(
            music.is_active() && music.play.unwrap().score == 0.0,
            "nothing changed"
        );
    }
}
