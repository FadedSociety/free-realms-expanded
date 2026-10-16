//! Checkers (work item 009) rule and session engine.
//!
//! Recovered original facts (provenance outside Git, see
//! `notes/Research/Pieces/009 Checkers.md`):
//! - `game_checkers.gfx` (`AssetsW_010.pack` 853) is the TYPE 7 Checkers movie.
//!   Its own `.lst` header names
//!   `C:\dev\FreeRealms\Live\Runtime\Client\Minigames\Checkers\game_checkers.swf`.
//! - `ConfigServer.GAME_ID = 21`; the six `MiniGameData` TYPE 7 rows are ids
//!   21, 68, 69, 70, 541, 542 with `MIN_PLAYERS = MAX_PLAYERS = 0`.
//! - `BoardProxy` fixes 32 playable spots, 12 pieces per side, red on spots
//!   0..=11, black on 20..=31, and `GetSpotFromOffset` geometry: odd rows are
//!   shifted left one column slot. A pawn walks one row towards its opponent,
//!   a king keeps all four diagonals (`GameServer.IsKing` promotes on row 0 or
//!   row 7).
//! - Captures are compulsory: `BoardProxy.StartMove` first offers every piece
//!   `CanMove(piece, true)` (jumps only) and falls back to plain moves only
//!   when no jump exists. After a jump the turn is locked to the jumping piece
//!   (`mJumpPiece`) until no further jump is available, so a submitted move is
//!   one capture chain.
//! - `GameServer.OnMove(start, moves)` emits, per hop, `MOVE(10)`, then
//!   `MOVE_JUMPED(11)` for the captured spot and `MOVE_KING(12)` on promotion,
//!   and finally `TURN_END(2)`/`TURN_START(1)`.
//! - A side with no legal move loses (`StartMove` returns false; the movie
//!   reports Win/Loss for the local-computer path). No draw rule was found in
//!   the movie.
//! - `GameClientNetwork.SendMove(start, end)` relays `OnMoveMsg\t<start>\t<moves>`
//!   through the Free Realms `FR_event` bridge; the movie's server splits the
//!   destinations on `,`.
//!
//! Reconstructed policy (explicitly not original): this session's replay
//! protection and error taxonomy, the `wire_index` player-number convention
//! beyond the recovered `INIT(0, 1)`/`TURN_START(0)` bootstrap, and the
//! instance namespace. The binary packet that carries `OnCheckersEventMsg`
//! is not recovered; only the movie-level message text is.
//!
//! The engine is self-contained (no new dependencies). The caller owns
//! persistence, rewards and the wire encoding: `apply` returns the movie's
//! event stream for one accepted move, and the whole session is `Clone` so a
//! gateway can persist it before committing replies.

use std::fmt;

/// `ConfigServer.GAME_ID` in `game_checkers.gfx`.
pub const CHECKERS_GAME_ID: i32 = 21;
/// `BoardProxy.SPOTS`: the 32 dark squares of the 8x8 board.
pub const SPOTS: usize = 32;
/// `BoardProxy.PIECES_PER_SIDE`.
pub const PIECES_PER_SIDE: usize = 12;
/// `BoardProxy.PIECES`.
pub const PIECES: usize = 24;
/// Upper bound for one capture chain; one side can never capture more than
/// its opponent's 12 pieces.
pub const MAX_CHAIN: usize = PIECES_PER_SIDE;
/// Bound for one movie-level `OnMoveMsg` payload.
pub const MAX_MOVE_PAYLOAD: usize = 256;

/// `CheckersNetworkEvent.INIT`.
pub const EV_INIT: i32 = 0;
/// `CheckersNetworkEvent.TURN_START`.
pub const EV_TURN_START: i32 = 1;
/// `CheckersNetworkEvent.TURN_END`.
pub const EV_TURN_END: i32 = 2;
/// `CheckersNetworkEvent.MOVE`.
pub const EV_MOVE: i32 = 10;
/// `CheckersNetworkEvent.MOVE_JUMPED`.
pub const EV_MOVE_JUMPED: i32 = 11;
/// `CheckersNetworkEvent.MOVE_KING`.
pub const EV_MOVE_KING: i32 = 12;
/// `CheckersNetworkEvent.MOVE_INVALID`.
pub const EV_MOVE_INVALID: i32 = 20;
/// `CheckersNetworkEvent.BOARD`.
pub const EV_BOARD: i32 = 30;
/// `CheckersNetworkEvent.VS_AI`.
pub const EV_VS_AI: i32 = 32;
/// `CheckersNetworkEvent.PLAYERLEFT`.
pub const EV_PLAYERLEFT: i32 = 40;
/// `CheckersNetworkEvent.WAITING`.
pub const EV_WAITING: i32 = 41;

/// The movie sends two-argument events; this marks the omitted argument.
pub const UNDEFINED_ARG: i32 = -1;

/// The movie-level move message name (`GameClientNetwork.SendMove`).
pub const MOVE_MESSAGE: &str = "OnMoveMsg";

/// Board colour. Red occupies spots 0..=11 and walks towards row 7; black
/// occupies 20..=31 and walks towards row 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Red,
    Black,
}

impl Side {
    pub const fn opponent(self) -> Side {
        match self {
            Side::Red => Side::Black,
            Side::Black => Side::Red,
        }
    }

    /// Player number used by `TURN_START`/`TURN_END`. Recovered bootstrap:
    /// `GameServer.OnReady` sends `INIT(0, 1)` and then `TURN_START(0)`.
    pub const fn wire_index(self) -> i32 {
        match self {
            Side::Black => 0,
            Side::Red => 1,
        }
    }

    /// Black moves first (`TURN_START(0)` with the red player at id 1).
    pub const fn plays_first() -> Side {
        Side::Black
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Side::Red => "red",
            Side::Black => "black",
        })
    }
}

/// One checker. `king` mirrors `PieceProxy.mKing`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub side: Side,
    pub king: bool,
}

/// One legal capture hop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Jump {
    /// Enemy spot that would be removed.
    pub over: usize,
    /// Empty landing spot.
    pub land: usize,
}

/// One complete legal move: a start square and the destinations in order.
/// A capture chain is a single action because the movie submits the whole
/// `mMoveArray` with one `SendMove`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub from: usize,
    pub path: Vec<usize>,
}

/// One movie-level `OnCheckersEventMsg(type, argA, argB)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckersEvent {
    pub kind: i32,
    pub arg_a: Option<i32>,
    pub arg_b: Option<i32>,
}

impl CheckersEvent {
    /// Event with two arguments; `UNDEFINED_ARG` becomes "omitted".
    pub fn new(kind: i32, arg_a: i32, arg_b: i32) -> Self {
        CheckersEvent {
            kind,
            arg_a: pick(arg_a),
            arg_b: pick(arg_b),
        }
    }

    /// Event with one argument, as `SendCheckersEvent(type, argA)` sends it.
    pub fn single(kind: i32, arg_a: i32) -> Self {
        CheckersEvent {
            kind,
            arg_a: pick(arg_a),
            arg_b: None,
        }
    }
}

fn pick(arg: i32) -> Option<i32> {
    if arg == UNDEFINED_ARG {
        None
    } else {
        Some(arg)
    }
}

/// Why a submitted move was refused. The taxonomy is reconstructed policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveError {
    /// A spot in the action is outside `0..SPOTS`.
    OutOfRange,
    /// The action has no destination.
    EmptyChain,
    /// The session already ended; later submissions are refused.
    GameFinished,
    /// This exact action was already applied.
    Replayed,
    /// Another player owns the current turn.
    NotYourTurn,
    /// The start square is empty.
    NoPiece,
    /// The start square holds the opponent's piece.
    WrongSide,
    /// A jump exists for this side, so quiet moves are not allowed.
    CompulsoryCapture,
    /// The submitted chain stops while the jumping piece can still capture.
    CaptureContinuationRequired,
    /// The destination chain is not one of the legal actions.
    IllegalDestination,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            MoveError::OutOfRange => "spot outside the 32-square board",
            MoveError::EmptyChain => "move has no destination",
            MoveError::GameFinished => "checkers session already finished",
            MoveError::Replayed => "move was already applied",
            MoveError::NotYourTurn => "not this side's turn",
            MoveError::NoPiece => "start square is empty",
            MoveError::WrongSide => "start square holds the other side's piece",
            MoveError::CompulsoryCapture => "a capture is available and must be played",
            MoveError::CaptureContinuationRequired => "the jumping piece can still capture",
            MoveError::IllegalDestination => "destination is not a legal checkers move",
        };
        f.write_str(text)
    }
}

impl std::error::Error for MoveError {}

/// Why a movie-level `OnMoveMsg` text was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadError {
    /// First field is not `OnMoveMsg`.
    WrongMessage,
    /// Too many fields or payload over `MAX_MOVE_PAYLOAD`.
    MalformedPayload,
    /// The destination list is empty.
    EmptyMoves,
    /// A field is not an integer.
    MalformedNumber,
    /// A spot is outside `0..SPOTS`.
    SpotOutOfRange,
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            PayloadError::WrongMessage => "payload is not OnMoveMsg",
            PayloadError::MalformedPayload => "payload has the wrong shape",
            PayloadError::EmptyMoves => "payload lists no destination",
            PayloadError::MalformedNumber => "payload field is not an integer",
            PayloadError::SpotOutOfRange => "payload spot is outside the board",
        };
        f.write_str(text)
    }
}

impl std::error::Error for PayloadError {}

/// The 32 square checkers board. `spots[spot]` is `None` on an empty square.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    spots: [Option<Piece>; SPOTS],
}

impl Board {
    pub fn empty() -> Board {
        Board {
            spots: [None; SPOTS],
        }
    }

    /// `BoardProxy.ProxyReset`: red pawns on 0..=11 walking down, black pawns
    /// on 20..=31 walking up.
    pub fn initial() -> Board {
        let mut board = Board::empty();
        for spot in 0..PIECES_PER_SIDE {
            board.set(
                spot,
                Some(Piece {
                    side: Side::Red,
                    king: false,
                }),
            );
            board.set(
                spot + 20,
                Some(Piece {
                    side: Side::Black,
                    king: false,
                }),
            );
        }
        board
    }

    pub fn piece(&self, spot: usize) -> Option<Piece> {
        self.spots.get(spot).copied().flatten()
    }

    /// Fixture/restore hook. Returns `false` when `spot` is off the board.
    pub fn set(&mut self, spot: usize, piece: Option<Piece>) -> bool {
        match self.spots.get_mut(spot) {
            Some(square) => {
                *square = piece;
                true
            }
            None => false,
        }
    }

    /// Exact `BoardProxy.GetSpotFromOffset`: odd rows are shifted left by one
    /// column slot, so `dx` 0 and 1 are the two real diagonals.
    pub fn spot_from_offset(spot: usize, dx: i32, dy: i32) -> Option<usize> {
        if spot >= SPOTS {
            return None;
        }
        let mut row = (spot / 4) as i32;
        let mut col = (spot % 4) as i32;
        if row % 2 != 0 {
            col -= 1;
        }
        row += dy;
        col += dx;
        if !(0..4).contains(&col) || !(0..8).contains(&row) {
            return None;
        }
        Some((row * 4 + col) as usize)
    }

    /// Diagonal offsets in the movie's `GetMoves`/`GetJumps` order: the
    /// forward block first, then the backwards block for kings.
    fn diagonals(piece: Piece) -> &'static [(i32, i32)] {
        const RED_FORWARD: [(i32, i32); 2] = [(0, 1), (1, 1)];
        const BLACK_FORWARD: [(i32, i32); 2] = [(0, -1), (1, -1)];
        const FORWARD_DOWN: [(i32, i32); 4] = [(0, 1), (1, 1), (0, -1), (1, -1)];
        const FORWARD_UP: [(i32, i32); 4] = [(0, -1), (1, -1), (0, 1), (1, 1)];
        if piece.king {
            return match piece.side {
                Side::Red => &FORWARD_DOWN,
                Side::Black => &FORWARD_UP,
            };
        }
        match piece.side {
            Side::Red => &RED_FORWARD,
            Side::Black => &BLACK_FORWARD,
        }
    }

    /// Quiet destinations for the piece on `spot` (`BoardProxy.GetMoves`).
    pub fn steps(&self, spot: usize) -> Vec<usize> {
        let Some(piece) = self.piece(spot) else {
            return Vec::new();
        };
        Self::diagonals(piece)
            .iter()
            .filter_map(|(dx, dy)| Self::spot_from_offset(spot, *dx, *dy))
            .filter(|target| self.piece(*target).is_none())
            .collect()
    }

    /// Capture hops for the piece on `spot` (`BoardProxy.GetJumps`).
    pub fn jumps(&self, spot: usize) -> Vec<Jump> {
        let Some(piece) = self.piece(spot) else {
            return Vec::new();
        };
        Self::diagonals(piece)
            .iter()
            .filter_map(|(dx, dy)| {
                let over = Self::spot_from_offset(spot, *dx, *dy)?;
                let victim = self.piece(over)?;
                if victim.side == piece.side {
                    return None;
                }
                let land = Self::spot_from_offset(over, *dx, *dy)?;
                if self.piece(land).is_some() {
                    return None;
                }
                Some(Jump { over, land })
            })
            .collect()
    }

    /// Every legal action for `side`. When any capture exists the result holds
    /// only complete capture chains (compulsory capture); otherwise it holds
    /// every quiet step.
    pub fn legal_actions(&self, side: Side) -> Vec<Action> {
        let mut actions = Vec::new();
        for spot in 0..SPOTS {
            let Some(piece) = self.piece(spot) else {
                continue;
            };
            if piece.side != side {
                continue;
            }
            let mut work = self.clone();
            let mut path = Vec::new();
            collect_chains(&mut work, spot, piece, spot, &mut path, &mut actions);
        }
        if actions.is_empty() {
            for spot in 0..SPOTS {
                let Some(piece) = self.piece(spot) else {
                    continue;
                };
                if piece.side != side {
                    continue;
                }
                for target in self.steps(spot) {
                    actions.push(Action {
                        from: spot,
                        path: vec![target],
                    });
                }
            }
        }
        actions
    }

    pub fn has_action(&self, side: Side) -> bool {
        !self.legal_actions(side).is_empty()
    }

    /// `GameServer.IsJump`: `None` when the two spots are one row apart.
    pub fn jumped_spot(from: usize, to: usize) -> Option<usize> {
        if from >= SPOTS || to >= SPOTS {
            return None;
        }
        let row_from = from / 4;
        let row_to = to / 4;
        if row_from.abs_diff(row_to) != 2 {
            return None;
        }
        let mid = if row_from % 2 == 0 {
            (from + to + 1) / 2
        } else {
            (from + to - 1) / 2
        };
        (mid < SPOTS).then_some(mid)
    }

    /// `GameServer.IsKing`: a piece promotes on row 0 or row 7.
    pub const fn is_promotion_row(spot: usize) -> bool {
        spot / 4 == 0 || spot / 4 == 7
    }

    /// Move the piece from `from` to `to`, placing `placed` there (promotion
    /// may change the piece). Returns the captured spot, if any.
    fn move_piece(&mut self, from: usize, to: usize, placed: Piece) -> Option<usize> {
        self.spots[from] = None;
        let jumped = Board::jumped_spot(from, to);
        if let Some(over) = jumped {
            self.spots[over] = None;
        }
        self.spots[to] = Some(placed);
        jumped
    }
}

fn collect_chains(
    work: &mut Board,
    from: usize,
    piece: Piece,
    origin: usize,
    path: &mut Vec<usize>,
    out: &mut Vec<Action>,
) {
    if path.len() >= MAX_CHAIN {
        return;
    }
    let jumps = work.jumps(from);
    if jumps.is_empty() {
        return;
    }
    for jump in jumps {
        let captured = work.piece(jump.over);
        let moved = Piece {
            side: piece.side,
            king: piece.king || Board::is_promotion_row(jump.land),
        };
        work.set(from, None);
        work.set(jump.over, None);
        work.set(jump.land, Some(moved));
        path.push(jump.land);

        let before = out.len();
        collect_chains(work, jump.land, moved, origin, path, out);
        if out.len() == before {
            out.push(Action {
                from: origin,
                path: path.clone(),
            });
        }

        path.pop();
        work.set(jump.land, None);
        work.set(jump.over, captured);
        work.set(from, Some(piece));
    }
}

/// One accepted move, kept for replay protection and audit logs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedMove {
    pub side: Side,
    pub from: usize,
    pub path: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Finished,
}

/// Server-authoritative checkers session. The caller persists the whole value
/// (it is `Clone` + `PartialEq`) before committing replies or rewards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckersSession {
    instance_id: i32,
    board: Board,
    current: Side,
    phase: Phase,
    winner: Option<Side>,
    applied: Vec<AppliedMove>,
    events: Vec<CheckersEvent>,
}

impl CheckersSession {
    /// Fresh board with black to move (`TURN_START(0)`).
    pub fn new(instance_id: i32) -> CheckersSession {
        CheckersSession::from_board(instance_id, Board::initial(), Side::plays_first())
    }

    /// Restore or fixture entry point. A side that cannot move immediately
    /// loses, mirroring `BoardProxy.StartMove` returning false.
    pub fn from_board(instance_id: i32, board: Board, first: Side) -> CheckersSession {
        let mut session = CheckersSession {
            instance_id,
            board,
            current: first,
            phase: Phase::Playing,
            winner: None,
            applied: Vec::new(),
            events: Vec::new(),
        };
        session.settle();
        session
    }

    pub fn instance_id(&self) -> i32 {
        self.instance_id
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn current(&self) -> Side {
        self.current
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn is_finished(&self) -> bool {
        self.phase == Phase::Finished
    }

    pub fn winner(&self) -> Option<Side> {
        self.winner
    }

    pub fn events(&self) -> &[CheckersEvent] {
        &self.events
    }

    pub fn applied(&self) -> &[AppliedMove] {
        &self.applied
    }

    /// Check a submission without changing the session.
    pub fn validate(&self, side: Side, from: usize, path: &[usize]) -> Result<Action, MoveError> {
        if from >= SPOTS || path.iter().any(|spot| *spot >= SPOTS) {
            return Err(MoveError::OutOfRange);
        }
        if path.is_empty() {
            return Err(MoveError::EmptyChain);
        }
        if self.phase == Phase::Finished {
            return Err(MoveError::GameFinished);
        }
        if self
            .applied
            .iter()
            .any(|move_| move_.side == side && move_.from == from && move_.path == path)
        {
            return Err(MoveError::Replayed);
        }
        if side != self.current {
            return Err(MoveError::NotYourTurn);
        }
        match self.board.piece(from) {
            None => return Err(MoveError::NoPiece),
            Some(piece) if piece.side != side => return Err(MoveError::WrongSide),
            Some(_) => {}
        }

        let legal = self.board.legal_actions(side);
        if let Some(action) = legal
            .iter()
            .find(|action| action.from == from && action.path == path)
        {
            return Ok(action.clone());
        }
        let captures = legal.iter().any(|action| {
            action
                .path
                .first()
                .and_then(|to| Board::jumped_spot(action.from, *to))
                .is_some()
        });
        let stopped_early = legal.iter().any(|action| {
            action.from == from && action.path.len() > path.len() && action.path.starts_with(path)
        });
        if stopped_early {
            return Err(MoveError::CaptureContinuationRequired);
        }
        if captures {
            return Err(MoveError::CompulsoryCapture);
        }
        Err(MoveError::IllegalDestination)
    }

    /// Apply one move. On success returns the movie events for this move and
    /// leaves them appended to [`CheckersSession::events`]. On any error the
    /// session is untouched.
    pub fn apply(
        &mut self,
        side: Side,
        from: usize,
        path: &[usize],
    ) -> Result<Vec<CheckersEvent>, MoveError> {
        let action = self.validate(side, from, path)?;
        let mover = self
            .board
            .piece(action.from)
            .expect("validated start square");
        let mover_index = mover.side.wire_index();

        let mut emitted = Vec::new();
        let mut current = action.from;
        let mut piece = mover;
        for target in &action.path {
            let target = *target;
            let jumped = Board::jumped_spot(current, target);
            let moved = Piece {
                side: piece.side,
                king: piece.king || Board::is_promotion_row(target),
            };
            self.board.move_piece(current, target, moved);
            emitted.push(CheckersEvent::new(EV_MOVE, current as i32, target as i32));
            if let Some(over) = jumped {
                emitted.push(CheckersEvent::single(EV_MOVE_JUMPED, over as i32));
            }
            // `GameServer.IsKing(dest)` is destination-based, so a king that
            // steps back onto row 0/7 re-emits MOVE_KING exactly like a fresh
            // promotion; the client treats it as an idempotent `Kinged`.
            if Board::is_promotion_row(target) {
                emitted.push(CheckersEvent::single(EV_MOVE_KING, target as i32));
            }
            piece = moved;
            current = target;
        }

        self.current = mover.side.opponent();
        emitted.push(CheckersEvent::single(EV_TURN_END, mover_index));
        emitted.push(CheckersEvent::single(
            EV_TURN_START,
            self.current.wire_index(),
        ));

        self.applied.push(AppliedMove {
            side,
            from,
            path: path.to_vec(),
        });
        self.events.extend_from_slice(&emitted);
        self.settle();
        Ok(emitted)
    }

    /// Close the session when the side to move has no legal action.
    fn settle(&mut self) {
        if self.phase == Phase::Finished {
            return;
        }
        if !self.board.has_action(self.current) {
            self.phase = Phase::Finished;
            self.winner = Some(self.current.opponent());
        }
    }
}

/// `GameClientNetwork.SendMove(start, end)` text: `OnMoveMsg\t<start>\t<d1,d2>`.
pub fn encode_move_payload(from: usize, path: &[usize]) -> Result<String, MoveError> {
    if from >= SPOTS || path.iter().any(|spot| *spot >= SPOTS) {
        return Err(MoveError::OutOfRange);
    }
    if path.is_empty() {
        return Err(MoveError::EmptyChain);
    }
    let payload = format!(
        "{MOVE_MESSAGE}\t{from}\t{}",
        path.iter()
            .map(|spot| spot.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    if payload.len() > MAX_MOVE_PAYLOAD {
        return Err(MoveError::OutOfRange);
    }
    Ok(payload)
}

/// Parse the movie-level move text. `GameServer.OnMove` splits the third field
/// on `,` and the Free Realms bridge splits the whole message on tabs.
pub fn decode_move_payload(text: &str) -> Result<(usize, Vec<usize>), PayloadError> {
    if text.len() > MAX_MOVE_PAYLOAD {
        return Err(PayloadError::MalformedPayload);
    }
    let fields: Vec<&str> = text.split('\t').collect();
    if fields.first() != Some(&MOVE_MESSAGE) {
        return Err(PayloadError::WrongMessage);
    }
    if fields.len() != 3 {
        return Err(PayloadError::MalformedPayload);
    }
    let from = parse_spot(fields[1])?;
    if fields[2].is_empty() {
        return Err(PayloadError::EmptyMoves);
    }
    let mut path = Vec::new();
    for field in fields[2].split(',') {
        path.push(parse_spot(field)?);
    }
    Ok((from, path))
}

fn parse_spot(field: &str) -> Result<usize, PayloadError> {
    let value: i64 = field.parse().map_err(|_| PayloadError::MalformedNumber)?;
    if !(0..SPOTS as i64).contains(&value) {
        return Err(PayloadError::SpotOutOfRange);
    }
    Ok(value as usize)
}
