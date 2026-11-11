//! Treasure War's tables: one player's run against the computer, and the game two players share
//! through a lobby. Both ride the shared minigame lifecycle (offer 39/16, start 39/5, movie text
//! 39/14, result 39/18, close 26/42), as the checkers table does. The rules are
//! `crate::treasure_war`.
//!
//! The movie is our own, so the text is too (tab separated, the name first):
//! - Client to server: `TW_Ready` (the movie is up, or wants the whole table again), `TW_Play
//!   <hand slot>`, `TW_Take <treasure slot>`, and `FRServer_GameOver` once it has shown the
//!   verdict, which settles the run.
//! - Server to client: `TW_Names <name id> <picture>` (0 0 for another player), then whatever
//!   `Match::snapshot` says: `TW_Hand`, `TW_Board`, `TW_Duel`, `TW_Flip`, `TW_Phase`.
//!
//! A message that is not the seat's to send changes nothing and is answered with the whole
//! table, so a movie that fell out of step recovers.
//!
//! Reconstructed (R): the declared client type 7 (checkers', a board game against the server;
//! the table's own type 23 would start the exe's card client), the run ids, that a win pays
//! `win_coins` and a loss or a quit nothing, and that a lobby game pays like a solo one.
use crate::{
    chess::ChessMessage,
    minigame::{self, Goal, MinigameRuns, RunMode, RunSpec, RunState},
    music_game::{Finish, MusicOutcome as TableOutcome, field, payload_packet},
    power_ups::Rng,
    rewards::RewardState,
    tower_defense::win_grant,
    treasure_war::{Content, Match, Opponent, Prompts, Said},
};
use std::{io, sync::Arc};

/// (R) See the module note.
pub const GAME_TYPE: i32 = 7;
/// (R) Run ids of its own, apart from every other minigame's.
pub const FIRST_INSTANCE_ID: i32 = 727_000_001;
const GOAL: u32 = 1;

/// What the movie may ask of a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Request {
    Ready,
    Play(usize),
    Take(usize),
    GameOver,
}

fn parse(text: &str) -> Option<Request> {
    let fields: Vec<&str> = text.split('\t').collect();
    match fields.as_slice() {
        ["TW_Ready"] => Some(Request::Ready),
        ["TW_Play", slot] => slot.parse().ok().map(Request::Play),
        ["TW_Take", slot] => slot.parse().ok().map(Request::Take),
        ["FRServer_GameOver"] => Some(Request::GameOver),
        _ => None,
    }
}

fn run_spec(content: &Content) -> RunSpec {
    let movie = content.movie.clone();
    let mut spec = RunSpec::empty(content.game_id, GAME_TYPE, RunMode::Flash { movie });
    spec.name_id = content.name_id;
    spec.description_id = content.description_id;
    spec.icon_id = content.icon_id;
    spec.preload_file.clone_from(&content.preload_file);
    spec.objectives = vec![Goal {
        id: GOAL,
        name_id: content.goal_name_id,
        description_id: content.goal_description_id,
        category: 1,
        bonus: false,
        stars: 0,
        coins: 0,
        total: 0,
        failure_condition: false,
        members_only: false,
    }];
    spec
}

/// The game two lobby players share. The World owns it and a seat's `TreasureWarTable` borrows
/// it for one message, as for checkers (`checkers_lobby::HostedGame`), whose shape this keeps.
#[derive(Clone, Debug)]
pub struct HostedMatch {
    id: u32,
    game: Match,
    game_id: i32,
    prompts: Prompts,
    /// Each seat's player and run instance, seat 0 first.
    seats: [(u64, i32); 2],
    ready: [bool; 2],
    /// The winner when a seat gave up, or `Some(None)` when both let the clock run out; the
    /// game's own result otherwise.
    ended: Option<Option<usize>>,
    done: [bool; 2],
}

impl HostedMatch {
    pub fn new(
        id: u32,
        first: (u64, i32),
        second: (u64, i32),
        content: Arc<Content>,
        rng: Rng,
    ) -> Self {
        Self {
            id,
            game_id: content.game_id,
            prompts: content.prompts,
            game: Match::new(content, rng),
            seats: [first, second],
            ready: [false; 2],
            ended: None,
            done: [false; 2],
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    fn seat(&self, player: u64) -> Option<usize> {
        self.seats.iter().position(|seat| seat.0 == player)
    }

    pub fn seated(&self, player: u64) -> bool {
        self.seat(player).is_some()
    }

    fn addressed(&self, said: Vec<Said>) -> io::Result<Vec<ChessMessage>> {
        let message = |(seat, text): (usize, String)| {
            let (player_id, instance) = self.seats[seat % 2];
            let packet = payload_packet(instance, self.game_id, &text)?;
            Ok(ChessMessage {
                player_id,
                text,
                packet,
            })
        };
        said.into_iter().map(message).collect()
    }

    /// The winner's seat, once there is a result; `Some(None)` is nobody's win.
    fn result(&self) -> Option<Option<usize>> {
        self.ended.or(self.game.result())
    }

    fn verdict(&self, seat: usize) -> Said {
        let won = self.result() == Some(Some(seat));
        let (name, text_id) = if won {
            ("won", self.prompts.won)
        } else {
            ("lost", self.prompts.lost)
        };
        (seat, format!("TW_Phase\t{name}\t{text_id}\t0\t0"))
    }

    /// `TW_Ready`: the seat's whole table once both have sat down, and again whenever it asks.
    pub fn ready(&mut self, player: u64) -> io::Result<Vec<ChessMessage>> {
        let Some(seat) = self.seat(player) else {
            return Ok(Vec::new());
        };
        let first = !std::mem::replace(&mut self.ready[seat], true);
        let mut said = vec![(seat, "TW_Names\t0\t0".to_owned())];
        if self.ended.is_some() {
            // A seat that left is done; one the idle limit gave up on is still here, loading.
            if !self.done[seat] {
                said.push(self.verdict(seat));
            }
        } else if self.ready == [true; 2] {
            said.extend(self.game.snapshot(seat));
            if first {
                said.extend(self.game.snapshot(1 - seat));
            }
        } else {
            said.push((
                seat,
                format!("TW_Phase\twaiting\t{}\t0\t0", self.prompts.waiting),
            ));
        }
        self.addressed(said)
    }

    fn act(
        &mut self,
        player: u64,
        act: impl FnOnce(&mut Match, usize) -> Option<Vec<Said>>,
    ) -> io::Result<Vec<ChessMessage>> {
        let Some(seat) = self.seat(player) else {
            return Ok(Vec::new());
        };
        if self.ended.is_some() || self.ready != [true; 2] {
            return Ok(Vec::new());
        }
        let said = act(&mut self.game, seat).unwrap_or_else(|| self.game.snapshot(seat));
        self.addressed(said)
    }

    pub fn play(&mut self, player: u64, slot: usize) -> io::Result<Vec<ChessMessage>> {
        self.act(player, |game, seat| game.play(seat, slot))
    }

    pub fn take(&mut self, player: u64, slot: usize) -> io::Result<Vec<ChessMessage>> {
        self.act(player, |game, seat| game.take(seat, slot))
    }

    /// A seat gave up, in play or still sitting down: the other wins and hears so.
    pub fn forfeit(&mut self, player: u64) -> io::Result<Vec<ChessMessage>> {
        let Some(seat) = self.seat(player).filter(|_| self.result().is_none()) else {
            return Ok(Vec::new());
        };
        self.ended = Some(Some(1 - seat));
        // A client still loading would lose the message, so `ready` repeats it.
        let other = 1 - seat;
        let said = if self.ready[other] {
            vec![self.verdict(other)]
        } else {
            Vec::new()
        };
        self.addressed(said)
    }

    /// Which seats the game waits on: those still sitting down, else those yet to act.
    fn waiting(&self) -> [bool; 2] {
        if self.result().is_some() {
            [false; 2]
        } else if self.ready != [true; 2] {
            self.ready.map(|ready| !ready)
        } else {
            self.game.waiting()
        }
    }

    /// A seat the game is waiting on, for the lobby's idle clock. It may not be the only one:
    /// when the clock runs out call [`Self::time_out`], which knows the difference, rather than
    /// forfeiting this seat.
    pub fn waiting_on(&self) -> Option<u64> {
        let seat = self.waiting().iter().position(|waited| *waited)?;
        Some(self.seats[seat].0)
    }

    /// The idle clock ran out. One seat holding the game up loses it. When the game waits on
    /// both (neither has picked a card, or neither sat down) nobody wins: otherwise a seat could
    /// sit still and be handed the game.
    pub fn time_out(&mut self) -> io::Result<Vec<ChessMessage>> {
        match self.waiting() {
            [false, false] => Ok(Vec::new()),
            [true, true] => {
                self.ended = Some(None);
                let seats = [0, 1].into_iter().filter(|seat| self.ready[*seat]);
                let said = seats.map(|seat| self.verdict(seat)).collect();
                self.addressed(said)
            }
            [first, _] => {
                let player = self.seats[usize::from(!first)].0;
                let mut messages = self.forfeit(player)?;
                messages.extend(self.timed_out(player)?);
                Ok(messages)
            }
        }
    }

    /// What the seat that gave up hears, when the game gave up on it.
    pub fn timed_out(&self, player: u64) -> io::Result<Vec<ChessMessage>> {
        let said = self.seat(player).map(|seat| self.verdict(seat));
        self.addressed(said.into_iter().collect())
    }

    /// Whether this player won, once the game has a result.
    pub fn won(&self, player: u64) -> Option<bool> {
        let winner = self.result()?;
        self.seat(player).map(|seat| winner == Some(seat))
    }

    pub fn is_settled(&self, player: u64) -> bool {
        self.seat(player).is_some_and(|seat| self.done[seat])
    }

    /// A seat that has settled, or has gone.
    pub fn release(&mut self, player: u64) {
        if let Some(seat) = self.seat(player) {
            self.done[seat] = true;
        }
    }

    pub fn all_settled(&self) -> bool {
        self.done == [true; 2]
    }
}

/// One player's Treasure War: a game against the computer, or a seat at a lobby game.
/// Session-local like a checkers table: a reconnect starts fresh.
#[derive(Clone)]
pub struct TreasureWarTable {
    content: Arc<Content>,
    runs: MinigameRuns,
    /// The game against the computer, from the start (39/5) on.
    game: Option<Match>,
    opponent: Opponent,
    rng: Rng,
    started_ms: u64,
    /// A lobby game's table id; the game is then the World's.
    hosted: Option<u64>,
    /// That game, on loan for one message, and what the other seat is owed.
    lent: Option<HostedMatch>,
    relay: Vec<ChessMessage>,
}

impl TreasureWarTable {
    pub fn new(content: Content, rng: Rng) -> io::Result<Self> {
        content.validate()?;
        // `offer` validates the movie and preload names; fail at load, not at the first Play.
        MinigameRuns::new(FIRST_INSTANCE_ID)?.offer(run_spec(&content))?;
        Ok(Self {
            opponent: content.opponents[0],
            content: Arc::new(content),
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            game: None,
            rng,
            started_ms: 0,
            hosted: None,
            lent: None,
            relay: Vec::new(),
        })
    }

    pub fn content(&self) -> &Arc<Content> {
        &self.content
    }

    /// Whether players can start the game (`treasure_war.json` `open`).
    pub fn is_open(&self) -> bool {
        self.content.open
    }

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    /// Offer a game against the computer (39/16); `None` while a run is already open.
    pub fn begin(&mut self) -> io::Result<Option<TableOutcome>> {
        if self.is_active() {
            return Ok(None);
        }
        let mut staged = self.clone();
        let offered = staged.runs.offer(run_spec(&self.content))?;
        if !offered.accepted {
            return Ok(None);
        }
        let rows = &self.content.opponents;
        staged.opponent = rows[(staged.rng.next_u64() % rows.len() as u64) as usize];
        *self = staged;
        Ok(Some(TableOutcome::from_run(offered)))
    }

    /// Start a lobby game's run, as `CheckersTable::begin_hosted`: the countdown was the Go, so
    /// the offer hides its window and the start follows at once. `None` while a run is open.
    pub fn begin_hosted(
        &mut self,
        table_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<(TableOutcome, i32)>> {
        if self.is_active() {
            return Ok(None);
        }
        let mut spec = run_spec(&self.content);
        spec.hidden_offer = true;
        let mut staged = self.clone();
        let offered = staged.runs.offer(spec)?;
        let Some(instance) = staged
            .runs
            .current_instance_id()
            .filter(|_| offered.accepted)
        else {
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

    /// Lend the World's game for one message.
    pub fn lend(&mut self, game: HostedMatch) {
        self.lent = Some(game);
    }

    /// The lent game back, with what the other seat is owed.
    pub fn reclaim(&mut self) -> Option<(HostedMatch, Vec<ChessMessage>)> {
        let game = self.lent.take()?;
        Some((game, std::mem::take(&mut self.relay)))
    }

    /// One client packet while a run is open; `None` for a packet that is not the table's. A
    /// refused or stale packet changes nothing. A win pays `payer` (player guid and rewards).
    pub fn route_with(
        &mut self,
        packet: &[u8],
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<Option<TableOutcome>> {
        let Some(instance) = self.runs.current_instance_id() else {
            return Ok(None);
        };
        let mut staged = self.clone();
        let out = if packet.starts_with(&[39, 0, 14]) {
            if field(packet, 3) != Some(instance) {
                return Ok(None);
            }
            staged.movie(instance, packet, now_ms, payer)?
        } else if matches!(packet.get(..3), Some([39, 0, 5..=7])) || packet == [26, 0, 42, 0] {
            let mut out = TableOutcome::from_run(staged.runs.handle_client(packet)?);
            if packet[2] == 5 && packet[0] == 39 && out.accepted {
                let seed = staged.rng.next_u64();
                staged.game = Some(Match::new(staged.content.clone(), Rng::seeded(seed)));
                staged.started_ms = now_ms;
                let playing = staged.runs.started(instance);
                out.changed |= playing.changed;
                out.replies.extend(playing.replies);
            }
            if staged.runs.current_instance_id().is_none() {
                staged.game = None;
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
        let text = minigame::movie_text(packet)?;
        let game_id = self.content.game_id;
        // The client's SendPayload names game -1; ours names the game.
        if field(packet, 7) != Some(-1)
            || !matches!(field(packet, 11), Some(id) if id == -1 || id == game_id)
        {
            return Err(invalid("treasure war text for another game"));
        }
        if self.runs.state(instance) != Some(RunState::Playing) {
            return Ok(TableOutcome::refused("treasure war is not being played"));
        }
        let Some(request) = parse(text) else {
            return Ok(TableOutcome::refused("not a Treasure War message"));
        };
        if self.hosted.is_some() {
            return self.hosted_movie(instance, request, now_ms, payer);
        }
        let Some(game) = self.game.as_mut() else {
            return Ok(TableOutcome::refused("no treasure war game is open"));
        };
        let skill = self.opponent.skill;
        let mut said = Vec::new();
        match request {
            Request::GameOver => {
                let Some(winner) = game.result() else {
                    return Ok(TableOutcome::refused("the game is not over"));
                };
                self.game = None;
                return self.settle(instance, winner == Some(0), now_ms, payer);
            }
            Request::Ready => {
                said.push((
                    0,
                    format!(
                        "TW_Names\t{}\t{}",
                        self.opponent.name_id, self.opponent.image
                    ),
                ));
                said.extend(game.play_computer(1, skill));
                said.extend(game.snapshot(0));
            }
            Request::Play(slot) => said = game.play(0, slot).unwrap_or_else(|| game.snapshot(0)),
            Request::Take(slot) => said = game.take(0, slot).unwrap_or_else(|| game.snapshot(0)),
        }
        said.extend(game.play_computer(1, skill));
        let mine = said.into_iter().filter(|(seat, _)| *seat == 0);
        let replies = mine.map(|(_, text)| payload_packet(instance, game_id, &text));
        Ok(TableOutcome {
            accepted: true,
            replies: replies.collect::<io::Result<_>>()?,
            ..TableOutcome::default()
        })
    }

    /// One movie message of a lobby game. A game that is not here has closed: the movie's text
    /// has nothing to reach.
    fn hosted_movie(
        &mut self,
        instance: i32,
        request: Request,
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<TableOutcome> {
        let (Some((player, _)), Some(game)) = (payer, self.lent.as_mut()) else {
            return Ok(TableOutcome::default());
        };
        let messages = match request {
            Request::GameOver => {
                let Some(won) = game.won(player).filter(|_| !game.is_settled(player)) else {
                    return Ok(TableOutcome::default());
                };
                game.release(player);
                return self.settle(instance, won, now_ms, payer);
            }
            Request::Ready => game.ready(player)?,
            Request::Play(slot) => game.play(player, slot)?,
            Request::Take(slot) => game.take(player, slot)?,
        };
        let (mine, theirs) =
            (messages.into_iter()).partition::<Vec<_>, _>(|message| message.player_id == player);
        self.relay.extend(theirs);
        Ok(TableOutcome {
            accepted: true,
            replies: mine.into_iter().map(|message| message.packet).collect(),
            ..TableOutcome::default()
        })
    }

    /// The run's end: the goal, then the lifecycle's result. Only a win pays.
    fn settle(
        &mut self,
        instance: i32,
        won: bool,
        now_ms: u64,
        payer: Option<(u64, &RewardState)>,
    ) -> io::Result<TableOutcome> {
        let elapsed = now_ms.saturating_sub(self.started_ms) / 1000;
        let finish = Finish {
            won,
            score: 0,
            elapsed_seconds: u32::try_from(elapsed).unwrap_or(u32::MAX),
        };
        let content = &self.content;
        // `finish` changes nothing on an error.
        let run = match (won, payer) {
            (true, Some((player, rewards))) => {
                let grant = win_grant(player, content.win_coins, content.icon_id, content.name_id);
                self.runs
                    .finish(instance, &finish, Some((rewards, &grant)))?
            }
            (true, None) => self.runs.finish_win_without_reward(instance, &finish)?,
            (false, _) => self.runs.finish(instance, &finish, None)?,
        };
        let mut out = TableOutcome::from_run(run);
        let goal = if won {
            minigame::goal_done_packets(GOAL)
        } else {
            minigame::goal_failed_packets(GOAL)
        };
        out.replies.splice(0..0, goal);
        Ok(out)
    }
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::treasure_war::tests::content;

    const ALPHA: u64 = 11;
    const BETA: u64 = 22;

    fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
        let mut packet = vec![39, 0, sub];
        for value in [instance, -1, -1] {
            packet.extend(value.to_le_bytes());
        }
        packet
    }

    fn texts(out: &TableOutcome) -> Vec<String> {
        (out.replies.iter())
            .filter(|reply| reply.starts_with(&[39, 0, 14]))
            .map(|reply| minigame::movie_text(reply).unwrap().to_owned())
            .collect()
    }

    fn table(seed: u64) -> TreasureWarTable {
        TreasureWarTable::new((*content()).clone(), Rng::seeded(seed)).unwrap()
    }

    fn say(table: &mut TreasureWarTable, text: &str, now_ms: u64) -> TableOutcome {
        let instance = table.runs.current_instance_id().unwrap();
        let packet = payload_packet(instance, -1, text).unwrap();
        table.route_with(&packet, now_ms, None).unwrap().unwrap()
    }

    /// The last `TW_Phase` name in `texts`.
    fn phase(texts: &[String]) -> Option<String> {
        let phases = texts
            .iter()
            .filter_map(|text| text.strip_prefix("TW_Phase\t"));
        phases
            .last()
            .and_then(|rest| rest.split('\t').next())
            .map(str::to_owned)
    }

    /// Plays the seat's side with the first legal move each time, until the movie's verdict.
    fn play_out(table: &mut TreasureWarTable, mut heard: Vec<String>) -> bool {
        for _ in 0..2_000 {
            let next = match phase(&heard).as_deref() {
                Some("choose") => "TW_Play\t0".to_owned(),
                Some("attack") => {
                    let board = heard
                        .iter()
                        .rev()
                        .find(|text| text.starts_with("TW_Board"))
                        .unwrap();
                    let theirs = board.split('\t').nth(2).unwrap();
                    format!("TW_Take\t{}", theirs.find('d').unwrap())
                }
                Some("won") => return true,
                Some("lost") => return false,
                other => panic!("the game waits on nobody: {other:?} in {heard:?}"),
            };
            let out = say(table, &next, 5_000);
            assert!(out.accepted, "{next} refused");
            heard.extend(texts(&out));
        }
        panic!("the game never ended");
    }

    #[test]
    fn a_solo_game_runs_from_the_offer_to_one_settlement() {
        let (mut wins, mut losses) = (0, 0);
        for seed in 0..12 {
            let mut table = table(seed);
            let offer = table.begin().unwrap().unwrap();
            assert!(
                offer
                    .replies
                    .iter()
                    .any(|reply| reply.starts_with(&[39, 0, 16]))
            );
            assert!(table.begin().unwrap().is_none(), "one run at a time");
            let instance = table.runs.current_instance_id().unwrap();
            assert!(
                say(&mut table, "TW_Ready", 0).refused.is_some(),
                "not started yet"
            );
            table
                .route_with(&lifecycle(5, instance), 1_000, None)
                .unwrap()
                .unwrap();
            let ready = texts(&say(&mut table, "TW_Ready", 1_500));
            assert!(ready[0].starts_with("TW_Names\t"), "{ready:?}");
            assert!(ready.iter().any(|text| text.starts_with("TW_Hand\t")));
            assert!(
                say(&mut table, "FRServer_GameOver", 2_000)
                    .refused
                    .is_some(),
                "not over yet"
            );
            let won = play_out(&mut table, ready);
            let settled = say(&mut table, "FRServer_GameOver", 60_000);
            assert!(
                settled
                    .replies
                    .iter()
                    .any(|reply| reply.starts_with(&[39, 0, 18]))
            );
            assert!(table.game.is_none());
            // The run is finished: a second acknowledgement finds nothing to settle.
            let again = payload_packet(instance, -1, "FRServer_GameOver").unwrap();
            let twice = table.route_with(&again, 61_000, None).unwrap();
            assert!(
                twice.is_none_or(|out| !out.replies.iter().any(|r| r.starts_with(&[39, 0, 18])))
            );
            if won { wins += 1 } else { losses += 1 }
        }
        assert!(
            wins > 0 && losses > 0,
            "{wins} wins, {losses} losses: the computer is no contest"
        );
    }

    #[test]
    fn a_move_out_of_turn_or_out_of_range_answers_with_the_table_and_changes_nothing() {
        let mut table = table(3);
        table.begin().unwrap().unwrap();
        let instance = table.runs.current_instance_id().unwrap();
        table
            .route_with(&lifecycle(5, instance), 0, None)
            .unwrap()
            .unwrap();
        let ready = texts(&say(&mut table, "TW_Ready", 10));
        for bad in ["TW_Take\t0", "TW_Play\t99"] {
            let heard = texts(&say(&mut table, bad, 20));
            let hand = |texts: &[String]| {
                texts
                    .iter()
                    .find(|text| text.starts_with("TW_Hand"))
                    .cloned()
            };
            assert_eq!(hand(&heard), hand(&ready), "{bad}");
            assert_eq!(phase(&heard).as_deref(), Some("choose"));
        }
        for junk in ["TW_Play", "TW_Play\t-1", "TW_Play\tx", "OnConnectMsg", ""] {
            let packet = payload_packet(instance, -1, junk);
            let Ok(packet) = packet else { continue };
            // An empty text is malformed; anything else is refused without a change.
            if let Ok(Some(out)) = table.route_with(&packet, 30, None) {
                assert!(out.refused.is_some() && out.replies.is_empty(), "{junk:?}");
            }
        }
        let mut cut = payload_packet(instance, -1, "TW_Ready").unwrap();
        cut.truncate(17);
        assert!(
            table.route_with(&cut, 40, None).is_err(),
            "a cut payload is an error, not a panic"
        );
    }

    #[test]
    fn a_quit_closes_the_run_and_the_next_game_is_fresh() {
        let mut table = table(5);
        table.begin().unwrap().unwrap();
        let instance = table.runs.current_instance_id().unwrap();
        table
            .route_with(&lifecycle(5, instance), 0, None)
            .unwrap()
            .unwrap();
        say(&mut table, "TW_Ready", 10);
        let quit = table
            .route_with(&lifecycle(6, instance), 500, None)
            .unwrap()
            .unwrap();
        assert!(quit.accepted || quit.changed);
        for _ in 0..4 {
            if !table.is_active() {
                break;
            }
            table.route_with(&[26, 0, 42, 0], 600, None).unwrap();
            let Some(instance) = table.runs.current_instance_id() else {
                break;
            };
            table
                .route_with(&lifecycle(7, instance), 700, None)
                .unwrap();
        }
        assert!(
            !table.is_active() && table.game.is_none(),
            "the quit left a run open"
        );
        assert!(table.begin().unwrap().is_some());
    }

    /// A wallet for `player`, as the checkers table's tests build one.
    fn rewards(player: u64) -> RewardState {
        use sha2::{Digest, Sha256};
        static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let call = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "osfr-treasure-war-pay-{}-{call}",
            std::process::id()
        ));
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
            experience: std::collections::BTreeMap::new(),
        };
        RewardState::new(inventory, profiles, 8)
    }

    fn hosted() -> HostedMatch {
        HostedMatch::new(9, (ALPHA, 1), (BETA, 2), content(), Rng::seeded(4))
    }

    fn heard(messages: &[ChessMessage], player: u64) -> Vec<String> {
        let mine = messages
            .iter()
            .filter(|message| message.player_id == player);
        mine.map(|message| message.text.clone()).collect()
    }

    #[test]
    fn a_lobby_game_waits_for_both_seats_and_tells_each_only_its_own_hand() {
        let mut game = hosted();
        assert_eq!(game.waiting_on(), Some(ALPHA), "nobody has sat down");
        let first = game.ready(ALPHA).unwrap();
        assert_eq!(phase(&heard(&first, ALPHA)).as_deref(), Some("waiting"));
        assert!(
            game.play(ALPHA, 0).unwrap().is_empty(),
            "nobody plays alone"
        );
        assert_eq!(game.waiting_on(), Some(BETA));
        let second = game.ready(BETA).unwrap();
        let (alpha, beta) = (heard(&second, ALPHA), heard(&second, BETA));
        assert_eq!(phase(&alpha).as_deref(), Some("choose"));
        assert_eq!(phase(&beta).as_deref(), Some("choose"));
        let hand = |texts: &[String]| {
            texts
                .iter()
                .find(|text| text.starts_with("TW_Hand"))
                .cloned()
        };
        assert_ne!(hand(&alpha), hand(&beta), "each seat holds its own cards");
        // Asking again repeats only the asker's table.
        let again = game.ready(BETA).unwrap();
        assert!(heard(&again, ALPHA).is_empty() && hand(&heard(&again, BETA)) == hand(&beta));
        let played = game.play(ALPHA, 0).unwrap();
        assert!(heard(&played, BETA).contains(&"TW_Duel\t-\t?\t0".to_owned()));
        assert_eq!(game.waiting_on(), Some(BETA));
    }

    #[test]
    fn a_forfeit_is_a_win_for_the_other_seat_and_a_loading_seat_hears_it_when_ready() {
        let mut game = hosted();
        game.ready(ALPHA).unwrap();
        game.ready(BETA).unwrap();
        let out = game.forfeit(ALPHA).unwrap();
        assert_eq!(phase(&heard(&out, BETA)).as_deref(), Some("won"));
        assert_eq!((game.won(BETA), game.won(ALPHA)), (Some(true), Some(false)));
        assert!(
            game.forfeit(BETA).unwrap().is_empty(),
            "the game is decided"
        );
        assert_eq!(game.waiting_on(), None);
        assert_eq!(
            phase(&heard(&game.timed_out(ALPHA).unwrap(), ALPHA)).as_deref(),
            Some("lost")
        );

        let mut game = hosted();
        game.ready(ALPHA).unwrap();
        assert!(game.forfeit(ALPHA).unwrap().is_empty());
        assert_eq!(
            phase(&heard(&game.ready(BETA).unwrap(), BETA)).as_deref(),
            Some("won")
        );
        game.release(ALPHA);
        assert!(!game.all_settled() && game.is_settled(ALPHA));
        game.release(BETA);
        assert!(game.all_settled());
    }

    #[test]
    fn a_clock_that_runs_out_on_both_seats_is_nobodys_win_and_on_one_seat_is_its_loss() {
        // Neither has picked: seat 1 cannot sit still and be handed the game.
        let mut game = hosted();
        game.ready(ALPHA).unwrap();
        game.ready(BETA).unwrap();
        let out = game.time_out().unwrap();
        assert_eq!(phase(&heard(&out, ALPHA)).as_deref(), Some("lost"));
        assert_eq!(phase(&heard(&out, BETA)).as_deref(), Some("lost"));
        assert_eq!(
            (game.won(ALPHA), game.won(BETA)),
            (Some(false), Some(false))
        );
        assert!(game.time_out().unwrap().is_empty() && game.waiting_on().is_none());

        // Neither sat down: the same.
        let mut game = hosted();
        assert!(
            game.time_out().unwrap().is_empty(),
            "nobody is there to hear it"
        );
        assert_eq!(
            (game.won(ALPHA), game.won(BETA)),
            (Some(false), Some(false))
        );

        // Alpha picked, Beta did not: Beta alone held the game up.
        let mut game = hosted();
        game.ready(ALPHA).unwrap();
        game.ready(BETA).unwrap();
        game.play(ALPHA, 0).unwrap();
        let out = game.time_out().unwrap();
        assert_eq!(phase(&heard(&out, ALPHA)).as_deref(), Some("won"));
        assert_eq!(phase(&heard(&out, BETA)).as_deref(), Some("lost"));
        assert_eq!((game.won(ALPHA), game.won(BETA)), (Some(true), Some(false)));

        // Only Alpha sat down: Beta loses.
        let mut game = hosted();
        game.ready(ALPHA).unwrap();
        game.time_out().unwrap();
        assert_eq!((game.won(ALPHA), game.won(BETA)), (Some(true), Some(false)));
    }

    /// One seat's message, with the World's game lent around it as `room_routing` does.
    fn seat_says(
        table: &mut TreasureWarTable,
        game: &mut HostedMatch,
        player: u64,
        text: &str,
    ) -> (TableOutcome, Vec<ChessMessage>) {
        let instance = table.runs.current_instance_id().unwrap();
        let packet = payload_packet(instance, -1, text).unwrap();
        let rewards = rewards(player);
        table.lend(game.clone());
        let out = table
            .route_with(&packet, 9_000, Some((player, &rewards)))
            .unwrap()
            .unwrap();
        let (back, relay) = table.reclaim().unwrap();
        *game = back;
        (out, relay)
    }

    #[test]
    fn two_tables_share_one_game_through_to_a_settlement_each() {
        let (mut alpha, mut beta) = (table(1), table(2));
        let (_, first) = alpha.begin_hosted(9, 1_000).unwrap().unwrap();
        let (_, second) = beta.begin_hosted(9, 1_000).unwrap().unwrap();
        assert!(
            alpha.begin_hosted(9, 1_000).unwrap().is_none(),
            "one run at a time"
        );
        assert_eq!(alpha.hosted_table(), Some(9));
        let mut game =
            HostedMatch::new(9, (ALPHA, first), (BETA, second), content(), Rng::seeded(8));
        let mut seen = [Vec::new(), Vec::new()];
        let players = [ALPHA, BETA];
        let tell = |seen: &mut [Vec<String>; 2],
                    seat: usize,
                    out: &TableOutcome,
                    relay: &[ChessMessage]| {
            seen[seat].extend(texts(out));
            seen[1 - seat].extend(heard(relay, players[1 - seat]));
        };
        for seat in [0, 1] {
            let table = if seat == 0 { &mut alpha } else { &mut beta };
            let (out, relay) = seat_says(table, &mut game, players[seat], "TW_Ready");
            tell(&mut seen, seat, &out, &relay);
        }
        let mut verdicts = [None, None];
        for _ in 0..4_000 {
            let Some(waiting) = game.waiting_on() else {
                break;
            };
            let seat = usize::from(waiting == BETA);
            let next = match phase(&seen[seat]).as_deref() {
                Some("choose") => "TW_Play\t0".to_owned(),
                Some("attack") => {
                    let board = seen[seat]
                        .iter()
                        .rev()
                        .find(|t| t.starts_with("TW_Board"))
                        .unwrap();
                    format!(
                        "TW_Take\t{}",
                        board.split('\t').nth(2).unwrap().find('d').unwrap()
                    )
                }
                other => panic!("seat {seat} is waited on in {other:?}"),
            };
            let table = if seat == 0 { &mut alpha } else { &mut beta };
            let (out, relay) = seat_says(table, &mut game, players[seat], &next);
            assert!(out.accepted);
            tell(&mut seen, seat, &out, &relay);
        }
        for seat in [0, 1] {
            verdicts[seat] = phase(&seen[seat]);
            let table = if seat == 0 { &mut alpha } else { &mut beta };
            let (out, _) = seat_says(table, &mut game, players[seat], "FRServer_GameOver");
            assert!(
                out.replies
                    .iter()
                    .any(|reply| reply.starts_with(&[39, 0, 18])),
                "seat {seat}"
            );
            assert!(game.is_settled(players[seat]));
            let won = game.won(players[seat]) == Some(true);
            assert_eq!(out.reward_state.is_some(), won, "only a win pays");
        }
        assert!(game.all_settled());
        let won = |seat: usize| verdicts[seat].as_deref() == Some("won");
        assert_eq!(game.won(ALPHA), Some(won(0)));
        assert!(!(won(0) && won(1)), "one winner at most: {verdicts:?}");
    }
}
