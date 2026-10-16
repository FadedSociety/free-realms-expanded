//! Trading card game (work item 010) deck catalogue and match rule slice.
//!
//! Recovered original facts (provenance outside Git, see
//! `notes/Research/Pieces/010 Trading card game.md`):
//! - `MiniGameData.txt` declares four TYPE 3 rows (TCG lobby/counter slots)
//!   and 124 TYPE 16 rows: 44 campaign rounds, 67 named decks, 3 `Battle`
//!   rows, the lobby, the tutorial, Kai's single-player challenge and a
//!   handful of one-off and unnamed rows.
//! - 116 of those 124 rows carry the same shipped victory sentence: "Be the
//!   first to score 12 points to win the match". 108 of them also name the
//!   card the winner takes from the opponent's deck.
//! - The four deck archetypes in the shipped names are `resource`, `trick`,
//!   `creature` and `tournament` (12 rows each), plus descriptive one-off
//!   deck names.
//! - The community packet catalogue records `BaseMiniGamePacket = 39` with
//!   `RequestTCGChallengePacket = 38`, `TradingCardStartGamePacket = 40`,
//!   `MiniGameGameOverPacket = 18`, `MiniGameGameEndScorePacket = 47` and
//!   `MiniGameStarsEarnedPacket = 21`. The native client's constructors agree
//!   (`FUN_009a9ec0` builds the challenge packet with sub-opcode 0x26 and
//!   `FUN_009a9de0` builds the start packet with sub-opcode 0x28 and two
//!   payload fields), and `TradingCardGameHandler` loads
//!   `TradingCardAssets.txt` and `.\UI\TradingCard\TCCW_Main.swf`.
//!
//! Reconstructed policy (explicitly not original): strict monotonic event ids
//! for replay protection, the error taxonomy, the snapshot shape, and the
//! "loser receives nothing" rule. How points are actually earned in a match
//! is *not* recovered: the client's card engine carries card text only, while
//! card numbers arrive as card-data messages. This module therefore rewards a
//! validated point stream and never invents scoring actions.
//!
//! No new dependencies: the caller owns persistence, wire encoding and the
//! economy. [`TcgMatch`] is `Clone` + `PartialEq` so a gateway can persist it
//! before committing replies or rewards.

use std::fmt;

/// `BaseMiniGamePacket`: the minigame wire family the TCG sub-opcodes ride on.
pub const MINIGAME_BASE_PACKET: i32 = 39;
/// `BaseMiniGamePackets.RequestTCGChallengePacket`.
pub const REQUEST_TCG_CHALLENGE_PACKET: i32 = 38;
/// `BaseMiniGamePackets.TradingCardStartGamePacket`.
pub const TRADING_CARD_START_GAME_PACKET: i32 = 40;
/// `BaseMiniGamePackets.MiniGameGameOverPacket`.
pub const MINIGAME_GAME_OVER_PACKET: i32 = 18;
/// `BaseMiniGamePackets.MiniGameGameEndScorePacket`.
pub const MINIGAME_GAME_END_SCORE_PACKET: i32 = 47;
/// `BaseMiniGamePackets.MiniGameStarsEarnedPacket`.
pub const MINIGAME_STARS_EARNED_PACKET: i32 = 21;

/// The victory target shipped in the TYPE 16 deck descriptions.
pub const WIN_SCORE: i32 = 12;
/// The highest declared `DIFFICULTY` in the shipped TYPE 16 rows.
pub const MAX_DIFFICULTY: i32 = 5;
/// `MiniGameData` uses `-1` for rows whose difficulty is not declared.
pub const UNDECLARED_DIFFICULTY: i32 = -1;

/// Side of one match. The challenger is the player who sent
/// `RequestTCGChallengePacket`; the opponent is the challenged side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Challenger,
    Opponent,
}

impl Side {
    pub const fn index(self) -> usize {
        match self {
            Side::Challenger => 0,
            Side::Opponent => 1,
        }
    }

    pub const fn opponent(self) -> Side {
        match self {
            Side::Challenger => Side::Opponent,
            Side::Opponent => Side::Challenger,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Side::Challenger => "challenger",
            Side::Opponent => "opponent",
        })
    }
}

/// The four archetypes the shipped TYPE 16 deck names use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Archetype {
    Resource,
    Trick,
    Creature,
    Tournament,
}

impl Archetype {
    /// Case-insensitive match against the deck-name suffix.
    pub fn from_deck_name(name: &str) -> Option<Archetype> {
        match name.to_ascii_lowercase().as_str() {
            "resource" => Some(Archetype::Resource),
            "trick" => Some(Archetype::Trick),
            "creature" => Some(Archetype::Creature),
            "tournament" => Some(Archetype::Tournament),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Archetype::Resource => "resource",
            Archetype::Trick => "trick",
            Archetype::Creature => "creature",
            Archetype::Tournament => "tournament",
        }
    }

    pub const fn all() -> [Archetype; 4] {
        [
            Archetype::Resource,
            Archetype::Trick,
            Archetype::Creature,
            Archetype::Tournament,
        ]
    }
}

impl fmt::Display for Archetype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Row families observed in the shipped TYPE 16 names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    Round,
    Deck,
    Battle,
    Challenge,
    Lobby,
    Tutorial,
    Unnamed,
    Other,
}

impl RowKind {
    /// Classify one shipped `localized_name`. Case-insensitive; `""` is the
    /// unnamed rows the four tables contain.
    pub fn classify(name: &str) -> RowKind {
        if name.trim().is_empty() {
            return RowKind::Unnamed;
        }
        let lowered = name.to_ascii_lowercase();
        if lowered.contains("trading card game lobby") {
            return RowKind::Lobby;
        }
        if lowered.contains("trading card game tutorial") {
            return RowKind::Tutorial;
        }
        if let Some((_, rest)) = split_possessive(name) {
            let rest_lower = rest.to_ascii_lowercase();
            if matches!(
                rest_lower.as_str(),
                "first round"
                    | "second round"
                    | "third round"
                    | "fourth round"
                    | "fifth round"
                    | "final round"
            ) {
                return RowKind::Round;
            }
            if rest_lower.ends_with(" deck") {
                return RowKind::Deck;
            }
        }
        if lowered.starts_with("battle ") {
            return RowKind::Battle;
        }
        if lowered.ends_with("challenge") {
            return RowKind::Challenge;
        }
        RowKind::Other
    }
}

/// Split `"Tyrone's trick deck"` and `"Slowhands' First Round"` into owner and
/// remainder. Both possessive spellings appear in the shipped rows.
fn split_possessive(name: &str) -> Option<(&str, &str)> {
    if let Some(index) = name.find("'s ") {
        let (owner, rest) = name.split_at(index);
        if owner.is_empty() {
            return None;
        }
        return Some((owner, rest.trim_start_matches("'s ")));
    }
    if let Some(index) = name.find("s' ") {
        let owner = &name[..index + 1];
        let rest = &name[index..];
        return Some((owner, rest.trim_start_matches("s' ")));
    }
    None
}

/// One recovered `MiniGameData` TYPE 16 row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeckRow {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub kind: RowKind,
    pub opponent: Option<String>,
    pub archetype: Option<Archetype>,
    pub difficulty: Option<i32>,
    pub score_target: Option<i32>,
    pub reward_card: Option<String>,
    pub gaps: Vec<String>,
}

impl DeckRow {
    /// Build a row from its shipped fields. `difficulty` is the raw table
    /// value as text so `-1` and empty stay distinguishable.
    pub fn parse(
        id: i32,
        name: Option<&str>,
        description: Option<&str>,
        difficulty: &str,
    ) -> DeckRow {
        let kind = RowKind::classify(name.unwrap_or(""));
        let description_text = description.unwrap_or("");
        let opponent = match kind {
            RowKind::Round | RowKind::Deck | RowKind::Battle => {
                if kind == RowKind::Battle {
                    name.unwrap_or("")
                        .strip_prefix("Battle ")
                        .map(str::to_string)
                } else {
                    split_possessive(name.unwrap_or("")).map(|(owner, _)| owner.to_string())
                }
            }
            RowKind::Challenge => name.unwrap_or("").split('\'').next().map(str::to_string),
            _ => None,
        };
        let archetype = match kind {
            RowKind::Deck => split_possessive(name.unwrap_or(""))
                .map(|(_, rest)| rest.trim_end_matches(" deck").trim_end_matches(" Deck"))
                .and_then(Archetype::from_deck_name),
            _ => None,
        };
        let difficulty_value = difficulty.trim().parse::<i32>().ok().filter(|value| {
            *value != UNDECLARED_DIFFICULTY && (1..=MAX_DIFFICULTY).contains(value)
        });
        let score_target = parse_score_target(description_text);
        let reward_card = parse_reward_card(description_text);

        let mut gaps = Vec::new();
        if name.is_none() {
            gaps.push("name_locale_id".to_string());
        }
        if description.is_none() {
            gaps.push("description_locale_id".to_string());
        }
        if name.unwrap_or("").is_empty() {
            gaps.push("opponent".to_string());
        } else if opponent.is_none() {
            gaps.push("opponent".to_string());
        }
        if kind == RowKind::Deck && archetype.is_none() {
            gaps.push("archetype".to_string());
        }
        if score_target.is_none() {
            gaps.push("score_target".to_string());
        }
        if reward_card.is_none() {
            gaps.push("reward_card".to_string());
        }

        DeckRow {
            id,
            name: name.map(str::to_string),
            description: description.map(str::to_string),
            kind,
            opponent,
            archetype,
            difficulty: difficulty_value,
            score_target,
            reward_card,
            gaps,
        }
    }

    /// A row can pay out only when it both names a target and a card.
    pub fn reward_eligible(&self) -> bool {
        self.score_target.is_some() && self.reward_card.is_some()
    }
}

/// Recover `"Be the first to score 12 points"` without assuming the number.
fn parse_score_target(description: &str) -> Option<i32> {
    let anchor = description.find("score ")?;
    let rest = &description[anchor + "score ".len()..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let tail = &rest[digits.len()..];
    if !tail.starts_with(" points") {
        return None;
    }
    digits.parse().ok()
}

/// Recover the named reward: `"win the match and a <card> from <owner>'s deck"`
/// or the `"one of <opponent>'s Hero cards"` variant.
fn parse_reward_card(description: &str) -> Option<String> {
    let anchor = description.find("win the match and ")?;
    let rest = &description[anchor + "win the match and ".len()..];
    let rest = rest
        .strip_prefix("a ")
        .or_else(|| rest.strip_prefix("an "))
        .unwrap_or(rest);
    if let Some(hero) = rest.strip_prefix("one of ")
        && let Some(end) = hero.find(" Hero card")
    {
        let reward = &hero[..end + " Hero card".len()];
        return Some(reward.to_string());
    }
    let start = rest.find(" from ")?;
    let candidate = rest[..start].trim();
    let owner = &rest[start + " from ".len()..];
    let deck_index = owner.find(" deck")?;
    if deck_index > 40 || candidate.is_empty() {
        return None;
    }
    Some(candidate.to_string())
}

/// Why a point event or snapshot was refused. The taxonomy is reconstructed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchError {
    /// The target is outside `1..=WIN_SCORE`.
    InvalidTarget,
    /// The start packet has not been accepted yet.
    NotStarted,
    /// One point event must carry a positive number of points.
    InvalidPoints,
    /// A point event claimed more than the remaining target.
    PointsExceedTarget,
    /// Event ids must increase; this id was already consumed.
    ReplayedEvent,
    /// Event ids must increase; this id predates the last one.
    EventOutOfOrder,
    /// The match already has a winner.
    MatchFinished,
    /// The snapshot describes a state the rules cannot reach.
    InvalidSnapshot,
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MatchError::InvalidTarget => "score target is outside the recovered range",
            MatchError::NotStarted => "the match has not started",
            MatchError::InvalidPoints => "a point event must be positive",
            MatchError::PointsExceedTarget => "a point event exceeds the match target",
            MatchError::ReplayedEvent => "this point event was already applied",
            MatchError::EventOutOfOrder => "point event ids must strictly increase",
            MatchError::MatchFinished => "the match already has a winner",
            MatchError::InvalidSnapshot => "snapshot is not a reachable match state",
        })
    }
}

impl std::error::Error for MatchError {}

/// One accepted point event, kept for audit logs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppliedPoints {
    pub side: Side,
    pub score: i32,
    pub winner: Option<Side>,
}

/// One accepted point event with its wire id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppliedEvent {
    pub event_id: i32,
    pub side: Side,
    pub points: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    WaitingToStart,
    Playing,
    Finished,
}

/// Portable match state for reconnect. Same value as [`TcgMatch`] minus the
/// audit log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchSnapshot {
    pub scores: [i32; 2],
    pub finished: bool,
    pub winner: Option<Side>,
    pub last_event_id: i32,
}

/// Server-authoritative TCG match. The caller persists the whole value before
/// committing replies or rewards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TcgMatch {
    instance_id: i32,
    target: i32,
    scores: [i32; 2],
    phase: Phase,
    winner: Option<Side>,
    last_event_id: i32,
    applied: Vec<AppliedEvent>,
}

impl TcgMatch {
    /// A fresh challenge. `target` must be `1..=WIN_SCORE`; the only value the
    /// shipped rows use is [`WIN_SCORE`].
    pub fn new(instance_id: i32, target: i32) -> Result<TcgMatch, MatchError> {
        if !(1..=WIN_SCORE).contains(&target) {
            return Err(MatchError::InvalidTarget);
        }
        Ok(TcgMatch {
            instance_id,
            target,
            scores: [0, 0],
            phase: Phase::WaitingToStart,
            winner: None,
            last_event_id: 0,
            applied: Vec::new(),
        })
    }

    /// Restore a persisted session. `last_event_id` keeps replay protection
    /// across a reconnect.
    pub fn from_snapshot(
        instance_id: i32,
        target: i32,
        snapshot: MatchSnapshot,
    ) -> Result<TcgMatch, MatchError> {
        let mut game = TcgMatch::new(instance_id, target)?;
        if snapshot
            .scores
            .iter()
            .any(|score| !(0..=target).contains(score))
        {
            return Err(MatchError::InvalidSnapshot);
        }
        if snapshot.last_event_id < 0 {
            return Err(MatchError::InvalidSnapshot);
        }
        match (snapshot.finished, snapshot.winner) {
            (false, None) => {
                if snapshot.scores.iter().any(|score| *score >= target) {
                    return Err(MatchError::InvalidSnapshot);
                }
                game.scores = snapshot.scores;
                game.phase = Phase::Playing;
            }
            (true, Some(winner)) => {
                if snapshot.scores[winner.index()] < target {
                    return Err(MatchError::InvalidSnapshot);
                }
                if snapshot.scores[winner.opponent().index()] >= target {
                    return Err(MatchError::InvalidSnapshot);
                }
                game.scores = snapshot.scores;
                game.phase = Phase::Finished;
                game.winner = Some(winner);
            }
            // A finished match without a winner, or an unfinished match with
            // one, is not a state this rule set can reach.
            _ => return Err(MatchError::InvalidSnapshot),
        }
        game.last_event_id = snapshot.last_event_id;
        Ok(game)
    }

    /// `TradingCardStartGamePacket` accepted: points may now be applied.
    pub fn start(&mut self) {
        if self.phase == Phase::WaitingToStart {
            self.phase = Phase::Playing;
        }
    }

    pub fn instance_id(&self) -> i32 {
        self.instance_id
    }

    pub fn target(&self) -> i32 {
        self.target
    }

    pub fn scores(&self) -> [i32; 2] {
        self.scores
    }

    pub fn score(&self, side: Side) -> i32 {
        self.scores[side.index()]
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn winner(&self) -> Option<Side> {
        self.winner
    }

    pub fn last_event_id(&self) -> i32 {
        self.last_event_id
    }

    pub fn applied(&self) -> &[AppliedEvent] {
        &self.applied
    }

    /// Apply one validated point event. On error the match is untouched.
    pub fn apply_points(
        &mut self,
        side: Side,
        event_id: i32,
        points: i32,
    ) -> Result<AppliedPoints, MatchError> {
        match self.phase {
            Phase::WaitingToStart => return Err(MatchError::NotStarted),
            Phase::Finished => return Err(MatchError::MatchFinished),
            Phase::Playing => {}
        }
        if points <= 0 {
            return Err(MatchError::InvalidPoints);
        }
        if points > self.target {
            return Err(MatchError::PointsExceedTarget);
        }
        if event_id == self.last_event_id
            || self.applied.iter().any(|event| event.event_id == event_id)
        {
            return Err(MatchError::ReplayedEvent);
        }
        if event_id < self.last_event_id {
            return Err(MatchError::EventOutOfOrder);
        }

        let score = self.scores[side.index()] + points;
        self.scores[side.index()] = score;
        self.last_event_id = event_id;
        self.applied.push(AppliedEvent {
            event_id,
            side,
            points,
        });
        let winner = if score >= self.target {
            self.phase = Phase::Finished;
            self.winner = Some(side);
            Some(side)
        } else {
            None
        };
        Ok(AppliedPoints {
            side,
            score,
            winner,
        })
    }

    /// `decks[0]` belongs to [`Side::Challenger`], `decks[1]` to
    /// [`Side::Opponent`]. Grants the winner's named card, and only once the
    /// match is finished.
    pub fn reward_card<'a>(&self, decks: [&'a DeckRow; 2]) -> Option<&'a str> {
        let winner = self.winner?;
        let row = decks[winner.index()];
        if !row.reward_eligible() {
            return None;
        }
        row.reward_card.as_deref()
    }

    pub fn snapshot(&self) -> MatchSnapshot {
        MatchSnapshot {
            scores: self.scores,
            finished: self.phase == Phase::Finished,
            winner: self.winner,
            last_event_id: self.last_event_id,
        }
    }
}
