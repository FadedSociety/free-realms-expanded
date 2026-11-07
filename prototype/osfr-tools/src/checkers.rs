//! L104: checkers rules and movie text for client minigame type 7 (MiniGameData 21, the live
//! Games window row 514), played against the server.
//!
//! Recovered from the client's own `game_checkers.gfx` (its bundled `GameServer`, `BoardProxy`
//! and `CheckersNetworkEvent` classes; JPEXS export in the lab):
//! - 32 dark squares, row = square / 4; a diagonal neighbour is column + dx (+0 or +1), shifted
//!   by -1 on odd rows (`GetSpotFromOffset`).
//! - Player 0 starts on squares 20-31 and moves up, player 1 on 0-11 and moves down; player 0
//!   moves first (`ProxyReset`, the server's `OnReady`: INIT 0/1 then TURN_START 0). The client
//!   is player 0 and is not red, so its square numbers are the network's.
//! - A jump is forced whenever one exists, and a jumping piece keeps jumping while it can; its
//!   directions are the ones it started the turn with (`StartMove`, `CanMove`, `MyMove`).
//! - A piece landing on row 0 or 7 is crowned (`GameServer.IsKing`); the jumped square is
//!   `(s+d+1)/2` from an even row, `(s+d-1)/2` from an odd one (`IsJump`).
//! - Client to server: `OnReadyMsg`, `OnMoveMsg <start> <d1,d2,..>`. Server to client:
//!   `OnCheckersEventMsg <type> [a] [b]`, the event codes below; tab separated, stopping at the
//!   first absent argument (`SoeNetworkTypeFreeRealms.AddMessage`).
//! - The movie's own computer player (`PickRandomMove`): a random piece among those that can
//!   jump and a random jump, otherwise a random piece and a random step.
//!
//! Reconstructed (R): the server plays the computer side itself (as for chess) and never sends
//! `VS_AI`, so the client waits for the server's moves instead of choosing them; a side with no
//! legal move at its turn loses.
use crate::{
    checkers_lobby::{HOSTED_ROW, HostedGame},
    chess::ChessMessage,
    minigame::{
        Activity, ActivityFinish, Goal, MinigameRuns, RunMode, RunOutcome, RunSpec, RunState,
    },
    power_ups::Rng,
    rewards::{RewardDisplay, RewardGrant, RewardState},
};
use std::{collections::BTreeMap, io};

pub const TURN_START: u8 = 1;
pub const TURN_END: u8 = 2;
pub const MOVE: u8 = 10;
pub const MOVE_JUMPED: u8 = 11;
pub const MOVE_KING: u8 = 12;
pub const MOVE_INVALID: u8 = 20;
pub const INIT: u8 = 0;
pub const BOARD: u8 = 30;
/// The client's side.
pub const HUMAN: u8 = 0;
/// The server's side.
pub const COMPUTER: u8 = 1;
const SQUARES: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub owner: u8,
    pub king: bool,
}

/// One event of a played turn, in the order the movie's server sends them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Move(u8, u8),
    Jumped(u8),
    King(u8),
}

impl Event {
    /// The `OnCheckersEventMsg` payload text.
    pub fn text(self) -> String {
        match self {
            Event::Move(from, to) => message(MOVE, &[from, to]),
            Event::Jumped(spot) => message(MOVE_JUMPED, &[spot]),
            Event::King(spot) => message(MOVE_KING, &[spot]),
        }
    }
}

/// `OnCheckersEventMsg` with an event code and its arguments.
pub fn message(kind: u8, args: &[u8]) -> String {
    args.iter()
        .fold(format!("OnCheckersEventMsg\t{kind}"), |text, arg| {
            format!("{text}\t{arg}")
        })
}

/// A client message the table understands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Ready,
    Move { start: u8, path: Vec<u8> },
}

/// Parse one client payload; `None` for any other text.
pub fn parse(text: &str) -> Option<Request> {
    let mut fields = text.split('\t');
    match fields.next()? {
        "OnReadyMsg" => Some(Request::Ready),
        "OnMoveMsg" => {
            let start = fields.next()?.parse().ok()?;
            let path = (fields.next()?.split(','))
                .map(|spot| spot.parse().ok())
                .collect::<Option<Vec<u8>>>()?;
            (fields.next().is_none() && !path.is_empty() && path.len() <= 12)
                .then_some(Request::Move { start, path })
        }
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    squares: [Option<Piece>; SQUARES],
}

impl Default for Board {
    fn default() -> Self {
        let mut squares = [None; SQUARES];
        for spot in 0..12 {
            squares[spot] = Some(Piece {
                owner: COMPUTER,
                king: false,
            });
            squares[spot + 20] = Some(Piece {
                owner: HUMAN,
                king: false,
            });
        }
        Self { squares }
    }
}

/// `GetSpotFromOffset`: the neighbour `dx` (0 or 1) and `dy` (-1 or 1) away, if on the board.
fn offset(spot: usize, dx: usize, dy: i32) -> Option<usize> {
    let row = (spot / 4) as i32 + dy;
    let column = (spot % 4) as i32 + dx as i32 - ((spot / 4) % 2) as i32;
    ((0..8).contains(&row) && (0..4).contains(&column)).then(|| (row * 4 + column) as usize)
}

impl Board {
    pub fn piece(&self, spot: u8) -> Option<Piece> {
        *self.squares.get(spot as usize)?
    }

    /// The rows a piece may head for: forward only, both for a king.
    fn directions(piece: Piece) -> &'static [i32] {
        match (piece.king, piece.owner == HUMAN) {
            (true, _) => &[-1, 1],
            (false, true) => &[-1],
            (false, false) => &[1],
        }
    }

    fn steps(&self, spot: usize, piece: Piece) -> Vec<usize> {
        (Self::directions(piece).iter())
            .flat_map(|&dy| [0, 1].map(|dx| offset(spot, dx, dy)))
            .flatten()
            .filter(|&to| self.squares[to].is_none())
            .collect()
    }

    fn jumps(&self, spot: usize, piece: Piece) -> Vec<usize> {
        (Self::directions(piece).iter())
            .flat_map(|&dy| [0, 1].map(|dx| (dx, dy)))
            .filter_map(|(dx, dy)| {
                let over = offset(spot, dx, dy)?;
                let to = offset(over, dx, dy)?;
                let foe = self.squares[over].is_some_and(|p| p.owner != piece.owner);
                (foe && self.squares[to].is_none()).then_some(to)
            })
            .collect()
    }

    fn own(&self, player: u8) -> impl Iterator<Item = (usize, Piece)> + '_ {
        (self.squares.iter().enumerate())
            .filter_map(move |(spot, p)| p.filter(|p| p.owner == player).map(|p| (spot, p)))
    }

    fn must_jump(&self, player: u8) -> bool {
        self.own(player)
            .any(|(spot, piece)| !self.jumps(spot, piece).is_empty())
    }

    /// Whether `player` has any legal move; without one it has lost.
    pub fn can_move(&self, player: u8) -> bool {
        (self.own(player))
            .any(|(spot, p)| !self.jumps(spot, p).is_empty() || !self.steps(spot, p).is_empty())
    }

    /// Check and play one turn of `player`: `start` then each landing square. Nothing changes
    /// on an `Err`.
    pub fn play(&mut self, player: u8, start: u8, path: &[u8]) -> Result<Vec<Event>, &'static str> {
        let piece = self
            .piece(start)
            .filter(|p| p.owner == player)
            .ok_or("not your piece")?;
        let jumping = self.must_jump(player);
        if !jumping && path.len() != 1 {
            return Err("illegal move");
        }
        let (mut next, mut events, mut moving) = (self.clone(), Vec::new(), piece);
        let mut at = start as usize;
        next.squares[at] = None;
        for &to in path {
            let to = to as usize;
            // The turn keeps the directions the piece started it with.
            let legal = if jumping {
                next.jumps(at, piece)
            } else {
                next.steps(at, piece)
            };
            if !legal.contains(&to) {
                return Err(if jumping {
                    "a jump is required"
                } else {
                    "illegal move"
                });
            }
            events.push(Event::Move(at as u8, to as u8));
            if jumping {
                let over = jumped(at, to);
                next.squares[over] = None;
                events.push(Event::Jumped(over as u8));
            }
            if to / 4 == 0 || to / 4 == 7 {
                moving.king = true;
                events.push(Event::King(to as u8));
            }
            at = to;
        }
        if jumping && !next.jumps(at, piece).is_empty() {
            return Err("the jumping piece must keep jumping");
        }
        next.squares[at] = Some(moving);
        *self = next;
        Ok(events)
    }

    /// The movie's computer player: a random jump if any (then keep jumping at random), else a
    /// random step. `None` when the side cannot move.
    pub fn computer_turn(&self, player: u8, rng: &mut Rng) -> Option<(u8, Vec<u8>)> {
        let mut pick = |n: usize| (rng.next_u64() % n as u64) as usize;
        let pieces: Vec<_> = self.own(player).collect();
        let jumpers: Vec<_> = (pieces.iter())
            .filter(|(s, p)| !self.jumps(*s, *p).is_empty())
            .collect();
        if !jumpers.is_empty() {
            let (start, piece) = *jumpers[pick(jumpers.len())];
            let mut board = self.clone();
            board.squares[start] = None;
            let (mut at, mut path) = (start, Vec::new());
            loop {
                let options = board.jumps(at, piece);
                if options.is_empty() {
                    return Some((start as u8, path));
                }
                let to = options[pick(options.len())];
                board.squares[jumped(at, to)] = None;
                path.push(to as u8);
                at = to;
            }
        }
        let movers: Vec<_> = (pieces.iter())
            .filter(|(s, p)| !self.steps(*s, *p).is_empty())
            .collect();
        if movers.is_empty() {
            return None;
        }
        let (start, piece) = *movers[pick(movers.len())];
        let steps = self.steps(start, piece);
        Some((start as u8, vec![steps[pick(steps.len())] as u8]))
    }
}

/// `IsJump`: the square a jump from `at` to `to` passes over.
fn jumped(at: usize, to: usize) -> usize {
    if (at / 4) % 2 == 0 {
        (at + to + 1) / 2
    } else {
        (at + to - 1) / 2
    }
}

/// Client MiniGameData 21 (type 7, name 900, description 901, icon 917): the checkers row.
pub const GAME_ID: i32 = 21;
pub const GAME_TYPE: i32 = 7;
pub const NAME_ID: i32 = 900;
pub const DESCRIPTION_ID: i32 = 901;
pub const ICON_ID: i32 = 917;
pub const MOVIE: &str = "game_checkers.gfx";
/// L918: the client's own preload list for the movie (manifest `CheckersAssets.txt.z`). Type 7 is not
/// one the client readies by itself (`009beb70`: 1, 2, 17, 22), so without it the start screen stays
/// on "Loading..." (owner playtest 2026-11-02).
pub const PRELOAD_FILE: &str = "CheckersAssets.txt";
const MAX_TEXT: usize = 256;
/// (R) Run ids of its own, apart from chess's and every other minigame's.
pub const FIRST_INSTANCE_ID: i32 = 21_000_001;
/// L920 (R, owner playtest 2026-11-02: "no rewards"): a win pays chess's easiest-row award.
// ponytail: a constant; move it to content with any other checkers tuning.
pub const WIN_COINS: u32 = 10;

/// The Goals panel, from the client's text ids (wiki, `hBXYh5wDFIs_0.00`). Nothing ticks them yet.
fn goals() -> Vec<Goal> {
    let goal = |id, name_id, description_id, category, total| Goal {
        id,
        name_id,
        description_id,
        category,
        bonus: category == 4,
        stars: 0,
        coins: 0,
        total,
        failure_condition: false,
        members_only: false,
    };
    vec![
        goal(1, 47799, 47800, 1, 0),
        goal(2, 47795, 47796, 2, 5),
        goal(3, 93570, 93571, 4, 4),
        goal(4, 102282, 47796, 4, 2),
        Goal {
            failure_condition: true,
            ..goal(5, 93594, 93595, 5, 0)
        },
    ]
}

/// What one packet did at the table, for gameplay to apply.
#[derive(Default)]
pub struct TableOutcome {
    pub accepted: bool,
    pub changed: bool,
    pub replies: Vec<Vec<u8>>,
    /// L920: a won game's payout, to adopt before sending `reward_packets`, as for chess.
    pub reward_packets: Vec<Vec<u8>>,
    pub reward_state: Option<RewardState>,
}

impl TableOutcome {
    fn from_run(run: RunOutcome) -> Self {
        Self {
            accepted: run.accepted,
            changed: run.changed,
            replies: run.replies,
            reward_packets: run.reward_packets,
            reward_state: run.reward_state,
        }
    }
}

fn win_grant(player_id: u64) -> RewardGrant {
    RewardGrant {
        coins: WIN_COINS,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        sc: 0,
        display: RewardDisplay {
            source_guid: player_id,
            player_guid: player_id,
            reward_kind: 0,
            icon_id: ICON_ID,
            name_id: NAME_ID,
            rows: Vec::new(),
        },
    }
}

/// L104: this session's checkers table against the server, on the shared minigame lifecycle
/// (offer 39/16, start 39/5, movie text 39/14, result 39/18, close 26/42). Session-local like a
/// chess table: a reconnect starts fresh. (R) A win pays nothing but the minigame-win event.
#[derive(Clone)]
pub struct CheckersTable {
    runs: MinigameRuns,
    board: Option<Board>,
    won: Option<bool>,
    rng: Rng,
    started_ms: u64,
    /// A lobby game's table id; the board is then the World's.
    hosted: Option<u64>,
    /// That board, on loan for one message, and what the other seat is owed.
    lent: Option<HostedGame>,
    relay: Vec<ChessMessage>,
}

struct Finish {
    won: bool,
    elapsed_seconds: u32,
}

impl Activity for Finish {
    fn score(&self) -> u32 {
        0
    }
    fn finish(&self) -> Option<ActivityFinish> {
        Some(ActivityFinish {
            won: self.won,
            stars: 0,
            elapsed_seconds: self.elapsed_seconds,
        })
    }
}

impl CheckersTable {
    pub fn new(rng: Rng) -> io::Result<Self> {
        Ok(Self {
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            board: None,
            won: None,
            rng,
            started_ms: 0,
            hosted: None,
            lent: None,
            relay: Vec::new(),
        })
    }

    /// A table with its own random stream for the computer's choices.
    pub fn fresh() -> io::Result<Self> {
        Self::new(Rng::from_entropy()?)
    }

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    fn run_spec(game_id: i32) -> RunSpec {
        let mut spec = RunSpec::empty(
            game_id,
            GAME_TYPE,
            RunMode::Flash {
                movie: MOVIE.into(),
            },
        );
        spec.name_id = NAME_ID;
        spec.description_id = DESCRIPTION_ID;
        spec.icon_id = ICON_ID;
        spec.preload_file = PRELOAD_FILE.into();
        spec.objectives = goals();
        spec
    }

    /// Start a lobby game's run, as `ChessService::begin_hosted`: the countdown was the Go, so the
    /// offer hides its window and the start follows at once. `None` while a run is open.
    pub fn begin_hosted(
        &mut self,
        table_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<(TableOutcome, i32)>> {
        if self.is_active() {
            return Ok(None);
        }
        let mut spec = Self::run_spec(HOSTED_ROW);
        spec.hidden_offer = true;
        let mut staged = self.clone();
        let offered = staged.runs.offer(spec)?;
        let Some(instance) = staged.runs.current_instance_id().filter(|_| offered.accepted) else {
            return Ok(None);
        };
        let started = staged.runs.start(instance)?;
        staged.runs.started(instance);
        staged.started_ms = now_ms;
        staged.hosted = Some(table_id);
        *self = staged;
        let mut out = TableOutcome::from_run(offered);
        out.replies.extend(started.replies);
        Ok(Some((out, instance)))
    }

    /// The lobby game this run belongs to.
    pub fn hosted_table(&self) -> Option<u64> {
        self.hosted
    }

    /// Lend the World's board for one message.
    pub fn lend(&mut self, game: HostedGame) {
        self.lent = Some(game);
    }

    /// The lent board back, with what the other seat is owed.
    pub fn reclaim(&mut self) -> Option<(HostedGame, Vec<ChessMessage>)> {
        let game = self.lent.take()?;
        Some((game, std::mem::take(&mut self.relay)))
    }

    /// Offer the table (39/16); `None` while a run is already open.
    pub fn begin(&mut self) -> io::Result<Option<TableOutcome>> {
        if self.is_active() {
            return Ok(None);
        }
        let spec = Self::run_spec(GAME_ID);
        let mut staged = self.clone();
        let offered = staged.runs.offer(spec)?;
        if !offered.accepted {
            return Ok(None);
        }
        *self = staged;
        Ok(Some(TableOutcome::from_run(offered)))
    }

    /// One client packet while a run is open; `None` for a packet that is not the table's. A
    /// refused or stale packet changes nothing.
    pub fn route(&mut self, packet: &[u8], now_ms: u64) -> io::Result<Option<TableOutcome>> {
        self.route_with(packet, now_ms, None)
    }

    /// `route`, where a won game pays [`WIN_COINS`] to `payer` (player guid and reward state).
    pub fn route_with(
        &mut self,
        packet: &[u8],
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<Option<TableOutcome>> {
        let Some(instance) = self.runs.current_instance_id() else {
            return Ok(None);
        };
        let lifecycle = packet.len() >= 3 && packet[..2] == [39, 0] && matches!(packet[2], 5..=7);
        let mut staged = self.clone();
        let out = if packet.len() >= 3 && packet[..3] == [39, 0, 14] {
            staged.movie(instance, packet, now_ms, payer)?
        } else if lifecycle || packet == [26, 0, 42, 0] {
            let mut out = TableOutcome::from_run(staged.runs.handle_client(packet)?);
            if lifecycle && packet[2] == 5 && out.accepted {
                staged.board = Some(Board::default());
                staged.won = None;
                staged.started_ms = now_ms;
                let playing = staged.runs.started(instance);
                out.changed |= playing.changed;
                out.replies.extend(playing.replies);
            }
            if staged.runs.current_instance_id().is_none() {
                staged.board = None;
                staged.hosted = None;
            }
            out
        } else {
            return Ok(None);
        };
        if out.accepted || out.changed {
            *self = staged;
        }
        Ok(Some(out))
    }

    fn movie(
        &mut self,
        instance: i32,
        packet: &[u8],
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<TableOutcome> {
        let mut out = TableOutcome::default();
        if self.runs.state(instance) != Some(RunState::Playing) {
            return Ok(out);
        }
        let game = if self.hosted.is_some() { HOSTED_ROW } else { GAME_ID };
        let text = read_payload(instance, game, packet)?;
        let say = |out: &mut TableOutcome, text: &str| -> io::Result<()> {
            out.replies.push(payload_packet(instance, game, text)?);
            Ok(())
        };
        if text == "OnConnectMsg" {
            say(&mut out, "OnServerReadyMsg")?;
        } else if text.starts_with("OnStartGameMsg") {
            say(&mut out, "OnStartGameMsg")?;
            say(&mut out, &message(BOARD, &[1]))?;
        } else if self.hosted.is_some() {
            return self.hosted_movie(instance, &text, now_ms, payer);
        } else if text == "FRServer_GameOver" {
            // The client's acknowledgement settles the decided game, as for chess.
            let Some(won) = self.won.take() else {
                return Ok(out);
            };
            let elapsed = now_ms.saturating_sub(self.started_ms) / 1000;
            let finish = Finish {
                won,
                elapsed_seconds: u32::try_from(elapsed).unwrap_or(u32::MAX),
            };
            let run = match (won, payer) {
                (true, Some((player, rewards))) => {
                    self.runs
                        .finish(instance, &finish, Some((rewards, &win_grant(player))))?
                }
                (true, None) => self.runs.finish_win_without_reward(instance, &finish)?,
                (false, _) => self.runs.finish(instance, &finish, None)?,
            };
            self.board = None;
            return Ok(TableOutcome::from_run(run));
        } else {
            match (parse(&text), self.board.as_mut()) {
                // INIT rebuilds the client's board at the opening (`ProxyReset`), so a Ready
                // restarts the server's too and the two never differ.
                (Some(Request::Ready), Some(board)) if self.won.is_none() => {
                    *board = Board::default();
                    say(&mut out, &message(INIT, &[HUMAN, COMPUTER]))?;
                    say(&mut out, &message(TURN_START, &[HUMAN]))?;
                }
                (Some(Request::Move { start, path }), Some(board)) if self.won.is_none() => {
                    // (R) The movie has no MOVE_INVALID handler, so a refused move answers with a
                    // new turn, which re-opens its moves. An honest client never sends one.
                    let Ok(events) = board.play(HUMAN, start, &path) else {
                        say(&mut out, &message(MOVE_INVALID, &[HUMAN]))?;
                        say(&mut out, &message(TURN_START, &[HUMAN]))?;
                        return Ok(out);
                    };
                    for event in events {
                        say(&mut out, &event.text())?;
                    }
                    say(&mut out, &message(TURN_END, &[HUMAN]))?;
                    if board.can_move(COMPUTER) {
                        say(&mut out, &message(TURN_START, &[COMPUTER]))?;
                        let (start, path) = (board.computer_turn(COMPUTER, &mut self.rng))
                            .ok_or_else(|| invalid("the computer has no move"))?;
                        for event in board.play(COMPUTER, start, &path).map_err(invalid)? {
                            say(&mut out, &event.text())?;
                        }
                        say(&mut out, &message(TURN_END, &[COMPUTER]))?;
                        if board.can_move(HUMAN) {
                            say(&mut out, &message(TURN_START, &[HUMAN]))?;
                        } else {
                            self.won = Some(false);
                        }
                    } else {
                        self.won = Some(true);
                    }
                    if let Some(won) = self.won {
                        say(&mut out, if won { "OnGameWonMsg" } else { "OnGameLostMsg" })?;
                    }
                    out.changed = true;
                }
                _ => {}
            }
        }
        out.accepted = true;
        Ok(out)
    }
}

impl CheckersTable {
    /// One movie message of a lobby game. A board that is not here has closed: the movie's text has
    /// nothing to reach.
    fn hosted_movie(
        &mut self,
        instance: i32,
        text: &str,
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<TableOutcome> {
        let mut out = TableOutcome::default();
        let (Some((player, rewards)), Some(game)) = (payer, self.lent.as_mut()) else {
            return Ok(out);
        };
        if text == "FRServer_GameOver" {
            let Some(won) = game.won(player).filter(|_| !game.is_settled(player)) else {
                return Ok(out);
            };
            game.release(player);
            let elapsed = now_ms.saturating_sub(self.started_ms) / 1000;
            let finish = Finish {
                won,
                elapsed_seconds: u32::try_from(elapsed).unwrap_or(u32::MAX),
            };
            let grant = win_grant(player);
            let settlement = won.then_some((rewards, &grant));
            return Ok(TableOutcome::from_run(self.runs.finish(instance, &finish, settlement)?));
        }
        let messages = match parse(text) {
            Some(Request::Ready) => game.ready(player)?,
            Some(Request::Move { start, path }) => game.play(player, start, &path)?,
            _ => return Ok(out),
        };
        let (mine, theirs) = (messages.into_iter())
            .partition::<Vec<_>, _>(|message| message.player_id == player);
        self.relay.extend(theirs);
        out.replies = mine.into_iter().map(|message| message.packet).collect();
        out.accepted = true;
        Ok(out)
    }
}

/// 39/14: instance, the -1 group, the checkers game id, then the NUL-terminated text.
pub(crate) fn payload_packet(instance: i32, game: i32, text: &str) -> io::Result<Vec<u8>> {
    if text.len() > MAX_TEXT || text.contains('\0') {
        return Err(invalid("checkers payload text"));
    }
    let mut packet = vec![39, 0, 14];
    for value in [instance, -1, game] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(((text.len() + 1) as i32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.push(0);
    Ok(packet)
}

fn read_payload(instance: i32, game: i32, packet: &[u8]) -> io::Result<String> {
    let at = |offset: usize| crate::le::i32(packet, offset);
    let length = at(15).and_then(|n| usize::try_from(n).ok()).unwrap_or(0);
    if at(3) != Some(instance)
        || at(11).is_none_or(|id| id != -1 && id != game)
        || length == 0
        || length > MAX_TEXT + 1
        || packet.len() != 19 + length
        || packet.last() != Some(&0)
    {
        return Err(invalid("invalid checkers payload"));
    }
    String::from_utf8(packet[19..packet.len() - 1].to_vec()).map_err(invalid)
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_follow_the_movie_and_the_opening_moves() {
        // Row 5 (odd) square 21 reaches 16/17 upward; row 4 (even) square 16 reaches 12/13.
        assert_eq!((offset(21, 0, -1), offset(21, 1, -1)), (Some(16), Some(17)));
        assert_eq!((offset(16, 0, -1), offset(16, 1, -1)), (Some(12), Some(13)));
        assert_eq!(offset(3, 1, 1), None, "off the right edge");
        let board = Board::default();
        assert!(board.can_move(HUMAN) && board.can_move(COMPUTER));
        let mut b = board.clone();
        assert_eq!(b.play(HUMAN, 21, &[17]), Ok(vec![Event::Move(21, 17)]));
        assert_eq!(board.clone().play(HUMAN, 21, &[13]), Err("illegal move"));
        assert_eq!(board.clone().play(HUMAN, 9, &[13]), Err("not your piece"));
    }

    #[test]
    fn a_jump_is_forced_and_continues_and_crowns() {
        let mut b = Board {
            squares: [None; SQUARES],
        };
        b.squares[21] = Some(Piece {
            owner: HUMAN,
            king: false,
        });
        b.squares[17] = Some(Piece {
            owner: COMPUTER,
            king: false,
        });
        b.squares[9] = Some(Piece {
            owner: COMPUTER,
            king: false,
        });
        b.squares[30] = Some(Piece {
            owner: HUMAN,
            king: false,
        });
        assert_eq!(b.clone().play(HUMAN, 30, &[26]), Err("a jump is required"));
        assert_eq!(
            b.clone().play(HUMAN, 21, &[14]),
            Err("the jumping piece must keep jumping")
        );
        let events = b.play(HUMAN, 21, &[14, 5]).unwrap();
        assert_eq!(
            events,
            [
                Event::Move(21, 14),
                Event::Jumped(17),
                Event::Move(14, 5),
                Event::Jumped(9)
            ]
        );
        assert!(!b.can_move(COMPUTER), "no computer piece left");
        let mut crown = Board {
            squares: [None; SQUARES],
        };
        crown.squares[5] = Some(Piece {
            owner: HUMAN,
            king: false,
        });
        assert_eq!(crown.play(HUMAN, 5, &[1]).unwrap()[1], Event::King(1));
        assert_eq!(
            crown.piece(1),
            Some(Piece {
                owner: HUMAN,
                king: true
            })
        );
    }

    /// The client crowns only on the server's MOVE_KING echo, so a piece crowned mid-jump keeps
    /// the directions it started the turn with: continuing as a king would hang the client.
    #[test]
    fn a_piece_crowned_mid_jump_keeps_its_starting_directions() {
        let mut b = Board {
            squares: [None; SQUARES],
        };
        b.squares[10] = Some(Piece {
            owner: HUMAN,
            king: false,
        });
        b.squares[6] = Some(Piece {
            owner: COMPUTER,
            king: false,
        });
        b.squares[5] = Some(Piece {
            owner: COMPUTER,
            king: false,
        });
        assert!(
            b.clone().play(HUMAN, 10, &[1, 8]).is_err(),
            "no king jump in the same turn"
        );
        let events = b.play(HUMAN, 10, &[1]).unwrap();
        assert_eq!(events.last(), Some(&Event::King(1)));
    }

    #[test]
    fn the_computer_plays_only_legal_turns() {
        let mut rng = Rng::seeded(7);
        let mut board = Board::default();
        for turn in 0..40 {
            let player = (turn % 2) as u8;
            let Some((start, path)) = board.computer_turn(player, &mut rng) else {
                assert!(!board.can_move(player));
                return;
            };
            board
                .play(player, start, &path)
                .expect("the computer's turn is legal");
        }
    }

    fn texts(out: &TableOutcome) -> Vec<String> {
        (out.replies.iter())
            .filter(|p| p.starts_with(&[39, 0, 14]))
            .map(|p| String::from_utf8_lossy(&p[19..p.len() - 1]).into_owned())
            .collect()
    }

    /// Offer, start, handshake: the table's own instance and the movie's opening events.
    fn opened(seed: u64) -> (CheckersTable, i32) {
        let mut table = CheckersTable::new(Rng::seeded(seed)).unwrap();
        let offer = table.begin().unwrap().expect("offered");
        assert!(offer.replies[0].starts_with(&[39, 0, 16]));
        let instance = i32::from_le_bytes(offer.replies[0][3..7].try_into().unwrap());
        assert!(table.begin().unwrap().is_none(), "one open table at a time");
        let mut start = vec![39, 0, 5];
        for value in [instance, -1, -1] {
            start.extend(value.to_le_bytes());
        }
        assert!(table.route(&start, 1_000).unwrap().unwrap().accepted);
        let say = |table: &mut CheckersTable, text: &str| {
            texts(
                &table
                    .route(&payload_packet(instance, GAME_ID, text).unwrap(), 1_100)
                    .unwrap()
                    .unwrap(),
            )
        };
        assert_eq!(say(&mut table, "OnConnectMsg"), ["OnServerReadyMsg"]);
        assert_eq!(
            say(&mut table, "OnStartGameMsg\t0"),
            ["OnStartGameMsg", "OnCheckersEventMsg\t30\t1"]
        );
        assert_eq!(
            say(&mut table, "OnReadyMsg"),
            ["OnCheckersEventMsg\t0\t0\t1", "OnCheckersEventMsg\t1\t0"]
        );
        (table, instance)
    }

    #[test]
    fn a_move_is_answered_by_the_computer_and_a_refused_one_changes_nothing() {
        let (mut table, instance) = opened(5);
        let before = table.board.clone();
        let refused = table.route(
            &payload_packet(instance, GAME_ID, "OnMoveMsg\t21\t13").unwrap(),
            2_000,
        );
        let refused = refused.unwrap().unwrap();
        assert!(!refused.changed);
        assert_eq!(table.board, before, "a refused move changes nothing");
        let answer = ["OnCheckersEventMsg	20	0", "OnCheckersEventMsg	1	0"];
        assert_eq!(texts(&refused), answer, "the turn comes back");
        let out = table.route(
            &payload_packet(instance, GAME_ID, "OnMoveMsg\t21\t17").unwrap(),
            2_100,
        );
        let said = texts(&out.unwrap().unwrap());
        assert_eq!(
            said[..3],
            [
                "OnCheckersEventMsg\t10\t21\t17",
                "OnCheckersEventMsg\t2\t0",
                "OnCheckersEventMsg\t1\t1"
            ]
        );
        assert!(
            said.iter()
                .any(|t| t.starts_with("OnCheckersEventMsg\t10\t")),
            "the computer moved"
        );
        assert_eq!(
            said.last().unwrap(),
            "OnCheckersEventMsg\t1\t0",
            "back to the player"
        );
    }

    #[test]
    fn taking_the_last_piece_wins_and_the_clients_game_over_settles_once() {
        let (mut table, instance) = opened(9);
        let mut board = Board {
            squares: [None; SQUARES],
        };
        board.squares[21] = Some(Piece {
            owner: HUMAN,
            king: false,
        });
        board.squares[17] = Some(Piece {
            owner: COMPUTER,
            king: false,
        });
        table.board = Some(board);
        let out = table.route(
            &payload_packet(instance, GAME_ID, "OnMoveMsg\t21\t14").unwrap(),
            3_000,
        );
        assert_eq!(
            texts(&out.unwrap().unwrap()).last().unwrap(),
            "OnGameWonMsg"
        );
        let late = table.route(
            &payload_packet(instance, GAME_ID, "OnMoveMsg\t14\t10").unwrap(),
            3_100,
        );
        assert!(
            !late.unwrap().unwrap().changed,
            "no move after the game is decided"
        );
        let over = payload_packet(instance, GAME_ID, "FRServer_GameOver").unwrap();
        let settled = table.route(&over, 9_000).unwrap().unwrap();
        let result = (settled.replies.iter())
            .find(|p| p.starts_with(&[39, 0, 18]))
            .expect("39/18");
        assert_eq!(result[15], 1, "won");
        let again = table.route(&over, 9_100).unwrap();
        assert!(again.is_none_or(|out| !out.replies.iter().any(|p| p.starts_with(&[39, 0, 18]))));
    }

    #[test]
    fn a_cut_payload_is_refused_not_a_panic() {
        let packet = payload_packet(7, GAME_ID, "OnReadyMsg").unwrap();
        assert_eq!(read_payload(7, GAME_ID, &packet).unwrap(), "OnReadyMsg");
        assert!(read_payload(7, HOSTED_ROW, &packet).is_err(), "another game's message");
        for end in 0..packet.len() {
            assert!(read_payload(7, GAME_ID, &packet[..end]).is_err(), "cut at {end}");
        }
    }

    #[test]
    fn client_text_parses_and_events_frame() {
        assert_eq!(parse("OnReadyMsg"), Some(Request::Ready));
        assert_eq!(
            parse("OnMoveMsg\t21\t12,5"),
            Some(Request::Move {
                start: 21,
                path: vec![12, 5]
            })
        );
        assert_eq!(parse("OnMoveMsg\t21\t"), None);
        assert_eq!(parse("OnMoveMsg\t21\tx"), None);
        assert_eq!(Event::Move(21, 17).text(), "OnCheckersEventMsg\t10\t21\t17");
        assert_eq!(message(TURN_START, &[0]), "OnCheckersEventMsg\t1\t0");
    }

    /// An empty bag with 0 coins, as `Inventory::load` reads it (the chess tests' fixture).
    fn rewards(player: u64) -> RewardState {
        use sha2::{Digest, Sha256};
        // Tests run side by side, so each call has its own folder.
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let call = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir()
            .join(format!("osfr-checkers-pay-{}-{call}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let zero = 0_i32.to_le_bytes();
        let span = |kind: &str, offset: usize, profile_id: i32| {
            serde_json::json!({"kind": kind, "offset": offset, "length": 4, "profile_id": profile_id,
                "sha256": format!("{:x}", Sha256::digest(zero))})
        };
        let manifest = serde_json::json!({
            "schema_version": 1,
            "player_guid": player,
            "default_state": {"coins": 0, "items": [], "equipment": [], "next_instance_id": null},
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [span("coins", 13, 0), span("items", 17, 0), span("equipment", 21, 1)],
            "definitions": []
        });
        std::fs::write(directory.join("inventory.json"), manifest.to_string()).unwrap();
        let inventory = crate::inventory::Inventory::load(&directory, None).unwrap();
        let _ = std::fs::remove_dir_all(&directory);
        let profiles = crate::profiles::ProfileState {
            version: 1,
            active: 1,
            unlocked: [1].into(),
            experience: BTreeMap::new(),
        };
        RewardState::new(inventory, profiles, 8)
    }

    /// L920: a won game settled with a payer pays `WIN_COINS`; a lost one pays nothing.
    #[test]
    fn a_win_pays_its_coins_and_a_loss_pays_nothing() {
        let player = 7;
        let state = rewards(player);
        for (won, coins) in [(true, WIN_COINS), (false, 0)] {
            let (mut table, instance) = opened(9);
            table.won = Some(won);
            let over = payload_packet(instance, GAME_ID, "FRServer_GameOver").unwrap();
            let out = table
                .route_with(&over, 9_000, Some((player, &state)))
                .unwrap()
                .unwrap();
            let paid = out
                .reward_state
                .map_or(0, |next| next.inventory.state().coins);
            assert_eq!(paid, coins, "won {won}");
            assert_eq!(out.reward_packets.is_empty(), !won);
        }
    }

    const ALPHA: u64 = 7;
    const BETA: u64 = 8;

    /// One seat of a lobby game: its table and run.
    fn seat(seed: u64) -> (CheckersTable, i32) {
        let mut table = CheckersTable::new(Rng::seeded(seed)).unwrap();
        let (started, instance) = table.begin_hosted(5, 1_000).unwrap().unwrap();
        for sub in [16, 17] {
            assert!(started.replies.iter().any(|reply| reply.starts_with(&[39, 0, sub])), "39/{sub}");
        }
        assert!(table.begin_hosted(5, 1_000).unwrap().is_none(), "one run at a time");
        (table, instance)
    }

    /// One message on a seat's table, the board lent and returned as the World does it.
    fn say(
        (table, instance): &mut (CheckersTable, i32),
        game: &mut HostedGame,
        player: u64,
        text: &str,
        rewards: &RewardState,
    ) -> (TableOutcome, Vec<ChessMessage>) {
        table.lend(game.clone());
        let packet = payload_packet(*instance, HOSTED_ROW, text).unwrap();
        let out = table.route_with(&packet, 9_000, Some((player, rewards))).unwrap().unwrap();
        let (back, relay) = table.reclaim().unwrap();
        *game = back;
        (out, relay)
    }

    /// The last capture ends it for both: each seat's own `FRServer_GameOver` settles it once, the
    /// winner paid and the loser not, and only then is the table done with.
    #[test]
    fn taking_the_last_piece_settles_both_seats_once() {
        let state = rewards(ALPHA);
        let (mut alpha, mut beta) = (seat(1), seat(2));
        let mut game = HostedGame::new(5, (ALPHA, alpha.1), (BETA, beta.1));
        say(&mut alpha, &mut game, ALPHA, "OnReadyMsg", &state);
        say(&mut beta, &mut game, BETA, "OnReadyMsg", &state);
        let mut board = Board { squares: [None; SQUARES] };
        board.squares[21] = Some(Piece { owner: 0, king: false });
        board.squares[17] = Some(Piece { owner: 1, king: false });
        game.set_board(board);

        let (moved, relay) = say(&mut alpha, &mut game, ALPHA, "OnMoveMsg\t21\t14", &state);
        assert_eq!(texts(&moved).last().unwrap(), "OnGameWonMsg");
        assert_eq!(relay.last().unwrap().text, "OnGameLostMsg");

        let (won, _) = say(&mut alpha, &mut game, ALPHA, "FRServer_GameOver", &state);
        assert_eq!(won.reward_state.map(|next| next.inventory.state().coins), Some(WIN_COINS));
        assert!(!game.all_settled(), "the loser is still to finish");
        let (lost, _) = say(&mut beta, &mut game, BETA, "FRServer_GameOver", &state);
        assert!(lost.reward_state.is_none() && lost.changed, "a loss settles and pays nothing");
        assert!(game.all_settled());
    }

    /// Two tables, one board: each hears its own side, a move reaches the other seat, a seat that
    /// quits forfeits, and the winner's own `FRServer_GameOver` pays once.
    #[test]
    fn two_tables_share_one_board_and_a_quit_pays_the_other_seat_once() {
        let state = rewards(ALPHA);
        let (mut alpha, mut beta) = (seat(1), seat(2));
        let mut game = HostedGame::new(5, (ALPHA, alpha.1), (BETA, beta.1));

        let connected = say(&mut alpha, &mut game, ALPHA, "OnConnectMsg", &state).0;
        assert_eq!(texts(&connected), ["OnServerReadyMsg"]);
        let (ready, _) = say(&mut alpha, &mut game, ALPHA, "OnReadyMsg", &state);
        assert_eq!(texts(&ready), ["OnCheckersEventMsg\t0\t0\t1", "OnCheckersEventMsg\t41"]);
        let (ready, relay) = say(&mut beta, &mut game, BETA, "OnReadyMsg", &state);
        assert_eq!(texts(&ready)[0], "OnCheckersEventMsg\t0\t1\t1", "the second seat is red");
        assert_eq!(relay.len(), 1, "Alpha hears the first turn");

        let (moved, relay) = say(&mut alpha, &mut game, ALPHA, "OnMoveMsg\t21\t17", &state);
        assert!(moved.accepted);
        assert!(!relay.is_empty() && relay.iter().all(|message| message.player_id == BETA));

        // Beta quits: the run ends here, the World sees no hosted table left and forfeits the seat.
        let mut quit = vec![39, 0, 6];
        for value in [beta.1, -1, -1] {
            quit.extend(value.to_le_bytes());
        }
        beta.0.lend(game.clone());
        beta.0.route_with(&quit, 9_000, Some((BETA, &state))).unwrap().unwrap();
        assert_eq!(beta.0.hosted_table(), None, "a quit run leaves its table");
        game = beta.0.reclaim().unwrap().0;
        let told = game.forfeit(BETA).unwrap();
        game.release(BETA);
        assert_eq!(told.iter().map(|m| m.player_id).collect::<Vec<_>>(), [ALPHA]);

        let (over, _) = say(&mut alpha, &mut game, ALPHA, "FRServer_GameOver", &state);
        let paid = over.reward_state.map(|next| next.inventory.state().coins);
        assert_eq!(paid, Some(WIN_COINS));
        assert!(game.all_settled());
        let (again, _) = say(&mut alpha, &mut game, ALPHA, "FRServer_GameOver", &state);
        assert!(again.reward_state.is_none(), "a second GameOver pays nothing");
    }
}
