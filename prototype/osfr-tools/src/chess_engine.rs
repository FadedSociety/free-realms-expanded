//! L105: an optional UCI chess engine (the official, unmodified Stockfish binary) for the
//! computer side of a chess table.
//!
//! Owner 2026-11-01: a stronger, level-limited opponent so players can practise. Stockfish is
//! GPL-3.0 and runs as its own process; the server only speaks UCI to it (`uci`, `setoption`,
//! `isready`, `position fen`, `go movetime`, then reads `bestmove`). Nothing a client sends reaches
//! the engine: the position is the server's own, and the table plays the reply only if it is one
//! of the position's legal moves.
//!
//! (R) The host opts in by pointing `OSFR_CHESS_ENGINE` at the binary. Without it, or when the
//! engine fails to start, answers late or answers nonsense, the table's built-in chooser plays and
//! the failed process is dropped (respawned no sooner than a minute later). One engine process
//! serves every table of this server; its use is serialized, and each reply has a hard deadline.
//! The gateway's one event loop waits for each reply, so the engine may spend at most
//! `BUDGET_PER_SECOND` of every second across the server; past it the built-in chooser plays.
//! A worker thread polled by the loop would remove the wait (follow-up).
use serde::Deserialize;
use std::{
    collections::VecDeque,
    ffi::OsStr,
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Mutex, OnceLock, mpsc},
    time::{Duration, Instant},
};

/// The host's switch: the engine binary's path.
pub const ENGINE_VAR: &str = "OSFR_CHESS_ENGINE";
/// (R) A start beyond this is a failed engine.
const START_DEADLINE: Duration = Duration::from_secs(3);
/// (R) The per-move `readyok` deadline.
const READY_DEADLINE: Duration = Duration::from_secs(1);
/// (R) Slack over the move time before a reply counts as late.
const REPLY_SLACK: Duration = Duration::from_millis(500);
/// (R) Engine time the whole server may spend per second; the event loop waits for it.
const BUDGET_PER_SECOND: Duration = Duration::from_millis(250);
/// (R) After a failure the engine is not respawned for this long.
const RESPAWN_BACKOFF: Duration = Duration::from_secs(60);

/// One table's strength, from `chess.json`: Stockfish's `Skill Level` (0-20) or, with
/// `UCI_LimitStrength`, a target `UCI_Elo` (1320-3190).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Strength {
    #[serde(default)]
    pub skill_level: Option<u8>,
    #[serde(default)]
    pub elo: Option<u32>,
}

impl Strength {
    pub fn valid(&self) -> bool {
        self.skill_level.is_none_or(|level| level <= 20)
            && self.elo.is_none_or(|elo| (1320..=3190).contains(&elo))
            && (self.skill_level.is_some() || self.elo.is_some())
    }

    fn options(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "setoption name UCI_LimitStrength value {}",
            self.elo.is_some()
        )];
        lines.extend(
            self.elo
                .map(|elo| format!("setoption name UCI_Elo value {elo}")),
        );
        lines.extend(
            (self.skill_level).map(|level| format!("setoption name Skill Level value {level}")),
        );
        lines
    }
}

/// Talks UCI over any writer and a channel of the engine's output lines.
struct Session<W: Write> {
    input: W,
    lines: mpsc::Receiver<String>,
}

impl<W: Write> Session<W> {
    fn send(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.input, "{line}")?;
        self.input.flush()
    }

    /// The first line starting with `prefix` before `deadline`.
    fn wait(&self, prefix: &str, deadline: Instant) -> Option<String> {
        loop {
            let left = deadline.checked_duration_since(Instant::now())?;
            let line = self.lines.recv_timeout(left).ok()?;
            if line.starts_with(prefix) {
                return Some(line);
            }
        }
    }

    fn handshake(&mut self) -> Option<()> {
        self.send("uci").ok()?;
        self.wait("uciok", Instant::now() + START_DEADLINE)
            .map(drop)
    }

    /// The engine's move for `fen` at `strength`, as UCI text.
    fn best_move(&mut self, fen: &str, strength: Strength, movetime_ms: u32) -> Option<String> {
        if fen.contains(['\n', '\r']) || !strength.valid() {
            return None;
        }
        for line in strength.options() {
            self.send(&line).ok()?;
        }
        self.send("isready").ok()?;
        self.wait("readyok", Instant::now() + READY_DEADLINE)?;
        self.send(&format!("position fen {fen}")).ok()?;
        self.send(&format!("go movetime {movetime_ms}")).ok()?;
        let deadline = Instant::now() + Duration::from_millis(movetime_ms.into()) + REPLY_SLACK;
        let reply = self.wait("bestmove ", deadline)?;
        reply.split_whitespace().nth(1).map(str::to_owned)
    }
}

/// The engine process; dropping it ends the process.
struct Engine {
    child: Child,
    session: Session<ChildStdin>,
}

impl Engine {
    fn spawn(path: &OsStr) -> Option<Self> {
        let mut command = Command::new(path);
        // Stockfish needs no environment, and on Windows no console window of its own.
        command
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        std::os::windows::process::CommandExt::creation_flags(&mut command, 0x0800_0000);
        let mut child = command.spawn().ok()?;
        let (stdin, stdout) = (child.stdin.take()?, child.stdout.take()?);
        let (send, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if line.map(|line| send.send(line)).is_err() {
                    break;
                }
            }
        });
        let mut engine = Self {
            child,
            session: Session {
                input: stdin,
                lines,
            },
        };
        engine.session.handshake()?;
        Some(engine)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The engine, when its last failure was, and the engine time spent in the last second.
#[derive(Default)]
struct Slot {
    engine: Option<Engine>,
    failed_at: Option<Instant>,
    spent: VecDeque<(Instant, Duration)>,
}

impl Slot {
    /// Whether an ask of up to `movetime` fits the per-second budget at `now`.
    fn within_budget(&mut self, now: Instant, movetime: Duration) -> bool {
        self.spent
            .retain(|(at, _)| now.duration_since(*at) < Duration::from_secs(1));
        self.spent.iter().map(|(_, used)| *used).sum::<Duration>() + movetime <= BUDGET_PER_SECOND
    }
}

static ENGINE: OnceLock<Mutex<Slot>> = OnceLock::new();

/// The configured engine's move for `fen`, or `None` (no engine configured, the budget is spent,
/// it failed lately, or it fails now: the caller's own chooser plays).
pub fn best_move(fen: &str, strength: Strength, movetime_ms: u32) -> Option<String> {
    let path = std::env::var_os(ENGINE_VAR)?;
    let slot = ENGINE.get_or_init(|| Mutex::new(Slot::default()));
    let mut slot = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = Instant::now();
    let movetime = Duration::from_millis(movetime_ms.into());
    if slot
        .failed_at
        .is_some_and(|at| now.duration_since(at) < RESPAWN_BACKOFF)
        || !slot.within_budget(now, movetime)
    {
        return None;
    }
    if slot.engine.is_none() {
        slot.engine = Engine::spawn(&path);
    }
    let reply =
        (slot.engine.as_mut()).and_then(|e| e.session.best_move(fen, strength, movetime_ms));
    slot.spent.push_back((now, now.elapsed()));
    if reply.is_none() {
        slot.engine = None;
        slot.failed_at = Some(Instant::now());
    }
    reply
}

/// UCI move text (`e2e4`, `e7e8q`) as squares in the chess table's numbering (a8 = 0, FEN order)
/// and the promotion letter.
pub fn parse_uci(text: &str) -> Option<(u8, u8, Option<char>)> {
    let bytes = text.as_bytes();
    let square = |file: u8, rank: u8| {
        ((b'a'..=b'h').contains(&file) && (b'1'..=b'8').contains(&rank))
            .then(|| (b'8' - rank) * 8 + (file - b'a'))
    };
    let promote = match bytes.len() {
        4 => None,
        5 if b"qrbn".contains(&bytes[4]) => Some(bytes[4] as char),
        _ => return None,
    };
    Some((
        square(bytes[0], bytes[1])?,
        square(bytes[2], bytes[3])?,
        promote,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(replies: &[&str]) -> Session<Vec<u8>> {
        let (send, lines) = mpsc::channel();
        for reply in replies {
            send.send(reply.to_string()).unwrap();
        }
        Session {
            input: Vec::new(),
            lines,
        }
    }

    #[test]
    fn uci_moves_map_to_the_table_squares() {
        assert_eq!(parse_uci("e2e4"), Some((52, 36, None)));
        assert_eq!(parse_uci("a8h1"), Some((0, 63, None)));
        assert_eq!(parse_uci("e7e8q"), Some((12, 4, Some('q'))));
        for bad in ["", "e2", "e2e9", "i2e4", "e7e8k", "(none)"] {
            assert_eq!(parse_uci(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_turn_sets_the_strength_and_reads_the_best_move() {
        let mut s = session(&[
            "info string hi",
            "readyok",
            "info depth 1",
            "bestmove e7e5 ponder g1f3",
        ]);
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        let strength = Strength {
            skill_level: Some(0),
            elo: None,
        };
        assert_eq!(s.best_move(fen, strength, 50).as_deref(), Some("e7e5"));
        let sent = String::from_utf8(s.input).unwrap();
        assert_eq!(
            sent,
            format!(
                "setoption name UCI_LimitStrength value false\nsetoption name Skill Level value 0\n\
                 isready\nposition fen {fen}\ngo movetime 50\n"
            )
        );
    }

    #[test]
    fn a_silent_engine_or_a_bad_request_gives_no_move() {
        let elo = Strength {
            skill_level: None,
            elo: Some(1600),
        };
        let mut silent = session(&["readyok"]);
        assert_eq!(silent.best_move("8/8/8/8/8/8/8/8 w - - 0 1", elo, 1), None);
        let mut s = session(&["readyok", "bestmove a2a3"]);
        assert_eq!(
            s.best_move("bad\nfen", elo, 1),
            None,
            "no line breaks into the engine"
        );
        let weak = Strength {
            skill_level: None,
            elo: Some(800),
        };
        assert!(!weak.valid(), "below Stockfish's UCI_Elo floor");
        assert_eq!(s.best_move("8/8/8/8/8/8/8/8 w - - 0 1", weak, 1), None);
    }

    #[test]
    fn without_the_host_switch_there_is_no_engine() {
        assert!(
            std::env::var_os(ENGINE_VAR).is_none(),
            "unset {ENGINE_VAR} to run the chess suites: they expect the built-in chooser"
        );
        let any = Strength {
            skill_level: Some(5),
            elo: None,
        };
        assert_eq!(best_move("8/8/8/8/8/8/8/8 w - - 0 1", any, 10), None);
    }

    #[test]
    fn the_engine_may_spend_only_its_budget_each_second() {
        let mut slot = Slot::default();
        let now = Instant::now();
        assert!(slot.within_budget(now, Duration::from_millis(250)));
        slot.spent.push_back((now, Duration::from_millis(200)));
        assert!(
            !slot.within_budget(now, Duration::from_millis(100)),
            "over the second's budget"
        );
        assert!(slot.within_budget(now, Duration::from_millis(50)));
        let later = now + Duration::from_millis(1_001);
        assert!(
            slot.within_budget(later, Duration::from_millis(250)),
            "a new second"
        );
        assert!(
            slot.spent.is_empty(),
            "spent time older than a second is dropped"
        );
    }
    /// Run by hand against the host's binary: `OSFR_CHESS_ENGINE=<stockfish> cargo test --lib
    /// chess_engine::tests::a_real_engine -- --ignored`. Checks the spawn, handshake and reply path.
    #[test]
    #[ignore]
    fn a_real_engine_answers_each_strength() {
        assert!(
            std::env::var_os(ENGINE_VAR).is_some(),
            "set {ENGINE_VAR} to the engine binary"
        );
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        for strength in [
            Strength {
                skill_level: Some(0),
                elo: None,
            },
            Strength {
                skill_level: None,
                elo: Some(1600),
            },
            Strength {
                skill_level: None,
                elo: Some(2200),
            },
        ] {
            std::thread::sleep(Duration::from_millis(1_100)); // the per-second budget
            let reply = best_move(fen, strength, 200).expect("the engine answers");
            assert!(parse_uci(&reply).is_some(), "{reply}");
        }
    }
}
