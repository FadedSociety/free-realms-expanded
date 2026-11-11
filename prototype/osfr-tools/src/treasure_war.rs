//! Treasure War, MiniGameData 727 (the Games window row 1078): the rules, the cards and the
//! computer opponent. The table that runs them on the minigame lifecycle is
//! `crate::treasure_war_table`.
//!
//! Retail ran this game in a card client compiled into the exe, on card data its own server sent
//! at login; none of that data survives. This is a server-run stand-in: our rules, played through
//! a movie of our own on the shared Flash minigame channel (39/14 text both ways).
//!
//! The rules, from the game's own rules panel, the arena keeper's lines (client text 432006 and
//! 432008) and the Fandom page:
//! - Two players, each with a hand and face-down treasures. Both pick a card; the cards flip.
//!   Red beats green, green beats blue, blue beats red; the same colour goes to the higher power,
//!   and an equal power is a tie: both pick again.
//! - The duel's winner picks one of the loser's face-down treasures. It flips, and the winning
//!   card must beat it the same way. If it does the treasure is taken; if not it goes back face
//!   down, and the player has to remember it.
//! - Whoever takes every treasure of the other side wins.
//!
//! Reconstructed (R), all in `treasure_war.json`: the hand of four, the deck size, colour and
//! power of every card no footage shows, that a played card is discarded and replaced, that the
//! discards are reshuffled when the deck runs out, that a tie against a treasure leaves it, the
//! round cap (more treasures taken wins, level is nobody's win), and the opponents' skill.
//! Client text 432006 names the colour wheel the other way round; the rules panel and a filmed
//! duel (blue 6 beating red 4) agree with each other, so they win.
use crate::power_ups::Rng;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

pub const CONTENT_FILE: &str = "treasure_war.json";
const MAX_CONTENT: u64 = 256 * 1024;
const MAX_CARDS: usize = 512;
const MAX_OPPONENTS: usize = 256;
const MAX_POWER: u8 = 12;
const MAX_SIDE: usize = 8;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Red,
    Green,
    Blue,
}

impl Color {
    fn beats(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Red, Self::Green) | (Self::Green, Self::Blue) | (Self::Blue, Self::Red)
        )
    }

    /// The movie's colour number.
    fn code(self) -> u8 {
        self as u8
    }
}

/// Where a card's colour and power come from; never gameplay input.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum Evidence {
    #[serde(rename = "frame")]
    Frame,
    R,
}

/// One card: its art id (`<id>.dds`), its title's text id, and what decides a duel.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Card {
    pub id: u32,
    pub name_id: u32,
    pub color: Color,
    pub power: u8,
    pub evidence: Evidence,
}

impl Card {
    /// `Greater` when this card beats `other`.
    pub fn duel(&self, other: &Self) -> Ordering {
        if self.color.beats(other.color) {
            Ordering::Greater
        } else if other.color.beats(self.color) {
            Ordering::Less
        } else {
            self.power.cmp(&other.power)
        }
    }

    fn token(&self) -> String {
        format!(
            "{},{},{},{}",
            self.id,
            self.name_id,
            self.color.code(),
            self.power
        )
    }
}

/// A computer opponent: its name's text id and its `campaignImage_<image>.dds` picture.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Opponent {
    pub name_id: u32,
    pub image: u32,
    /// (R) 0 plays at random, 1 plays its strongest card and remembers treasures, 2 plays the
    /// card that beats the most of the set.
    pub skill: u8,
}

/// The arena keeper who offers the game in the world (`npc_spawns_extra.json`).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Arena {
    pub npc: u64,
    pub name: String,
    pub position: [f32; 3],
}

/// The client's own lines for each state of the game.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Prompts {
    pub choose: u32,
    pub opponent_choosing: u32,
    pub attack: u32,
    pub opponent_attacking: u32,
    pub waiting: u32,
    pub won: u32,
    pub lost: u32,
}

/// `treasure_war.json`.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Content {
    pub schema_version: u32,
    /// Provenance note; never gameplay input.
    #[serde(default)]
    pub source: Option<String>,
    /// Whether players can start the game. The movie is our own, so the row stays closed until
    /// the file reaches players.
    pub open: bool,
    pub game_id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub icon_id: i32,
    pub movie: String,
    pub preload_file: String,
    pub hand_size: usize,
    pub treasures: usize,
    pub deck_size: usize,
    pub max_rounds: u32,
    pub win_coins: u32,
    pub goal_name_id: u32,
    pub goal_description_id: u32,
    pub arena: Arena,
    pub prompts: Prompts,
    pub cards: Vec<Card>,
    pub opponents: Vec<Opponent>,
}

impl Content {
    pub fn load(directory: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(directory.join(CONTENT_FILE))?
            .take(MAX_CONTENT + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_CONTENT {
            return Err(invalid("treasure war content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(invalid)?;
        content.validate()?;
        Ok(content)
    }

    pub fn validate(&self) -> io::Result<()> {
        let sides = [self.hand_size, self.treasures];
        if self.schema_version != 1
            || self.game_id <= 0
            || self.name_id <= 0
            || self.description_id <= 0
            || self.icon_id <= 0
            || self.goal_name_id == 0
            || sides.iter().any(|size| !(1..=MAX_SIDE).contains(size))
            // One card to spare, so a hand can always be refilled.
            || self.deck_size <= self.hand_size + self.treasures
            || self.deck_size > self.cards.len()
            || self.cards.len() > MAX_CARDS
            || !(1..=1000).contains(&self.max_rounds)
            || self.win_coins > 1000
            || self.opponents.is_empty()
            || self.opponents.len() > MAX_OPPONENTS
            || self.arena.npc == 0
            || self.arena.name.is_empty()
        {
            return Err(invalid("invalid treasure war content"));
        }
        let mut seen = BTreeSet::new();
        for card in &self.cards {
            if card.id == 0 || card.name_id == 0 || card.power > MAX_POWER || !seen.insert(card.id)
            {
                return Err(invalid("invalid treasure war card"));
            }
        }
        if self
            .opponents
            .iter()
            .any(|row| row.name_id == 0 || row.skill > 2)
        {
            return Err(invalid("invalid treasure war opponent"));
        }
        Ok(())
    }
}

/// What a seat may be asked to do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// Each seat's pick, face down until both are in.
    Choose([Option<Card>; 2]),
    /// The duel's winner picks a treasure; both played cards stay on the table.
    Attack { attacker: usize, cards: [Card; 2] },
    /// `None` when the round cap ended it level.
    Over(Option<usize>),
}

#[derive(Clone, Copy, Debug)]
struct Treasure {
    card: Card,
    taken: bool,
    /// The other seat saw it defend and win.
    seen: bool,
}

#[derive(Clone, Debug)]
struct Side {
    deck: Vec<Card>,
    hand: Vec<Card>,
    discard: Vec<Card>,
    treasures: Vec<Treasure>,
}

/// One seat's message, for whoever sits there.
pub type Said = (usize, String);

/// One game between seats 0 and 1. Each seat is told only what it may see: its own hand, both
/// cards of a finished duel, and a treasure when it flips.
#[derive(Clone, Debug)]
pub struct Match {
    content: Arc<Content>,
    sides: [Side; 2],
    phase: Phase,
    round: u32,
    rng: Rng,
}

impl Match {
    /// Deals both seats a shuffled deck from the whole set (R: everyone owns every card).
    pub fn new(content: Arc<Content>, mut rng: Rng) -> Self {
        let mut deal = || {
            let mut deck = content.cards.clone();
            shuffle(&mut deck, &mut rng);
            deck.truncate(content.deck_size);
            let mut take = |count: usize| deck.split_off(deck.len().saturating_sub(count));
            let treasures = take(content.treasures);
            let hand = take(content.hand_size);
            Side {
                deck,
                hand,
                discard: Vec::new(),
                treasures: (treasures.into_iter())
                    .map(|card| Treasure {
                        card,
                        taken: false,
                        seen: false,
                    })
                    .collect(),
            }
        };
        let sides = [deal(), deal()];
        Self {
            content,
            sides,
            phase: Phase::Choose([None; 2]),
            round: 0,
            rng,
        }
    }

    /// The winner's seat once the game is over; `Some(None)` is a level game, nobody's win.
    pub fn result(&self) -> Option<Option<usize>> {
        match self.phase {
            Phase::Over(winner) => Some(winner),
            _ => None,
        }
    }

    /// Which seats the game waits on. Both, while neither has picked: then neither is the one
    /// holding the game up.
    pub fn waiting(&self) -> [bool; 2] {
        match self.phase {
            Phase::Choose(picks) => picks.map(|pick| pick.is_none()),
            Phase::Attack { attacker, .. } => [attacker == 0, attacker == 1],
            Phase::Over(_) => [false; 2],
        }
    }

    /// A seat the game waits on, lowest first.
    pub fn waiting_on(&self) -> Option<usize> {
        self.waiting().iter().position(|waited| *waited)
    }

    /// Treasures `seat` has taken.
    pub fn taken_by(&self, seat: usize) -> usize {
        let theirs = &self.sides[1 - seat % 2].treasures;
        theirs.iter().filter(|treasure| treasure.taken).count()
    }

    /// Everything `seat` may see, enough for its movie to draw the table from nothing.
    pub fn snapshot(&self, seat: usize) -> Vec<Said> {
        let mut out = Vec::new();
        self.say_hand(&mut out, seat);
        self.say_board(&mut out, seat);
        self.say_duel(&mut out, seat);
        self.say_phase(&mut out, seat);
        out
    }

    /// `seat` plays the card in hand slot `slot`. `None` when that is not theirs to do now.
    pub fn play(&mut self, seat: usize, slot: usize) -> Option<Vec<Said>> {
        let Phase::Choose(mut picks) = self.phase else {
            return None;
        };
        if seat > 1 || picks[seat].is_some() || slot >= self.sides[seat].hand.len() {
            return None;
        }
        picks[seat] = Some(self.sides[seat].hand.remove(slot));
        self.phase = Phase::Choose(picks);
        let mut out = Vec::new();
        self.say_hand(&mut out, seat);
        let [Some(first), Some(second)] = picks else {
            for to in [0, 1] {
                self.say_duel(&mut out, to);
                self.say_phase(&mut out, to);
            }
            return Some(out);
        };
        let cards = [first, second];
        match first.duel(&second) {
            Ordering::Equal => {
                for to in [0, 1] {
                    out.push((to, duel_text(cards[to], cards[1 - to], 3)));
                }
                self.end_round(cards, &mut out);
            }
            order => {
                let attacker = usize::from(order == Ordering::Less);
                self.phase = Phase::Attack { attacker, cards };
                for to in [0, 1] {
                    self.say_duel(&mut out, to);
                    self.say_phase(&mut out, to);
                }
            }
        }
        Some(out)
    }

    /// The duel's winner attacks treasure `slot` of the other seat. `None` when that is not
    /// theirs to do now.
    pub fn take(&mut self, seat: usize, slot: usize) -> Option<Vec<Said>> {
        let Phase::Attack { attacker, cards } = self.phase else {
            return None;
        };
        let defender = 1 - attacker;
        let treasure = self.sides[defender].treasures.get_mut(slot)?;
        if seat != attacker || treasure.taken {
            return None;
        }
        let taken = cards[attacker].duel(&treasure.card) == Ordering::Greater;
        treasure.taken = taken;
        treasure.seen |= !taken;
        let flip = |mine: u8| {
            format!(
                "TW_Flip\t{mine}\t{slot}\t{}\t{}",
                treasure.card.token(),
                u8::from(taken)
            )
        };
        let mut out = vec![(attacker, flip(1)), (defender, flip(0))];
        if self.sides[defender]
            .treasures
            .iter()
            .all(|treasure| treasure.taken)
        {
            self.phase = Phase::Over(Some(attacker));
            for to in [0, 1] {
                self.say_board(&mut out, to);
                self.say_phase(&mut out, to);
            }
        } else {
            self.end_round(cards, &mut out);
        }
        Some(out)
    }

    /// Both played cards are discarded and replaced (R), and the next round opens, unless the
    /// cap ends the game on treasures taken.
    fn end_round(&mut self, cards: [Card; 2], out: &mut Vec<Said>) {
        let hand_size = self.content.hand_size;
        for (side, card) in self.sides.iter_mut().zip(cards) {
            side.discard.push(card);
            while side.hand.len() < hand_size {
                if side.deck.is_empty() {
                    shuffle(&mut side.discard, &mut self.rng);
                    side.deck.append(&mut side.discard);
                }
                let Some(card) = side.deck.pop() else { break };
                side.hand.push(card);
            }
        }
        self.round += 1;
        self.phase = if self.round >= self.content.max_rounds {
            Phase::Over(match self.taken_by(0).cmp(&self.taken_by(1)) {
                Ordering::Greater => Some(0),
                Ordering::Less => Some(1),
                Ordering::Equal => None,
            })
        } else {
            Phase::Choose([None; 2])
        };
        for to in [0, 1] {
            self.say_hand(out, to);
            self.say_board(out, to);
            self.say_duel(out, to);
            self.say_phase(out, to);
        }
    }

    fn say_hand(&self, out: &mut Vec<Said>, seat: usize) {
        let cards = self.sides[seat]
            .hand
            .iter()
            .map(|card| format!("\t{}", card.token()));
        out.push((seat, format!("TW_Hand{}", cards.collect::<String>())));
    }

    /// Each side's treasures, the seat's own first: `d` face down, `t` taken.
    fn say_board(&self, out: &mut Vec<Said>, seat: usize) {
        let row = |side: &Side| -> String {
            let slots = side.treasures.iter();
            slots
                .map(|treasure| if treasure.taken { 't' } else { 'd' })
                .collect()
        };
        let (mine, theirs) = (row(&self.sides[seat]), row(&self.sides[1 - seat]));
        out.push((seat, format!("TW_Board\t{mine}\t{theirs}")));
    }

    /// The two duel slots: the seat's own card, then the other's, which is `?` until both are in.
    fn say_duel(&self, out: &mut Vec<Said>, seat: usize) {
        let text = match self.phase {
            Phase::Choose(picks) => {
                let mine = picks[seat].map_or("-".into(), |card| card.token());
                let theirs = if picks[1 - seat].is_some() { "?" } else { "-" };
                format!("TW_Duel\t{mine}\t{theirs}\t0")
            }
            Phase::Attack { attacker, cards } => duel_text(
                cards[seat],
                cards[1 - seat],
                if attacker == seat { 1 } else { 2 },
            ),
            Phase::Over(_) => "TW_Duel\t-\t-\t0".into(),
        };
        out.push((seat, text));
    }

    fn say_phase(&self, out: &mut Vec<Said>, seat: usize) {
        let prompts = &self.content.prompts;
        let (name, text_id) = match self.phase {
            Phase::Choose(picks) if picks[seat].is_none() => ("choose", prompts.choose),
            Phase::Choose(_) => ("wait", prompts.opponent_choosing),
            Phase::Attack { attacker, .. } if attacker == seat => ("attack", prompts.attack),
            Phase::Attack { .. } => ("defend", prompts.opponent_attacking),
            Phase::Over(Some(winner)) if winner == seat => ("won", prompts.won),
            Phase::Over(_) => ("lost", prompts.lost),
        };
        let score = format!("{}\t{}", self.taken_by(seat), self.taken_by(1 - seat));
        out.push((seat, format!("TW_Phase\t{name}\t{text_id}\t{score}")));
    }

    /// The computer's moves for `seat`, as long as the game waits on it. It reads nothing a
    /// player in that seat could not see.
    pub fn play_computer(&mut self, seat: usize, skill: u8) -> Vec<Said> {
        let mut out = Vec::new();
        loop {
            let said = match self.phase {
                Phase::Choose(picks) if picks[seat].is_none() => {
                    let slot = self.computer_card(seat, skill);
                    self.play(seat, slot)
                }
                Phase::Attack { attacker, cards } if attacker == seat => {
                    let slot = self.computer_treasure(seat, skill, cards[seat]);
                    self.take(seat, slot)
                }
                _ => None,
            };
            match said {
                Some(said) => out.extend(said),
                None => return out,
            }
        }
    }

    fn pick(&mut self, count: usize) -> usize {
        // `count` is a hand or a treasure row, never 0 where this is called.
        (self.rng.next_u64() % count.max(1) as u64) as usize
    }

    fn computer_card(&mut self, seat: usize, skill: u8) -> usize {
        let hand = &self.sides[seat].hand;
        let set = &self.content.cards;
        let beaten = |card: &Card| {
            set.iter()
                .filter(|other| card.duel(other) == Ordering::Greater)
                .count()
        };
        let best = match skill {
            0 => None,
            1 => (0..hand.len()).max_by_key(|slot| hand[*slot].power),
            _ => (0..hand.len()).max_by_key(|slot| beaten(&hand[*slot])),
        };
        let count = hand.len();
        best.unwrap_or_else(|| self.pick(count))
    }

    fn computer_treasure(&mut self, seat: usize, skill: u8, card: Card) -> usize {
        let theirs = &self.sides[1 - seat].treasures;
        let open = |keep: &dyn Fn(&Treasure) -> bool| -> Vec<usize> {
            let slots = theirs.iter().enumerate();
            slots
                .filter(|(_, treasure)| !treasure.taken && keep(treasure))
                .map(|(slot, _)| slot)
                .collect()
        };
        let beats = |treasure: &Treasure| card.duel(&treasure.card) == Ordering::Greater;
        // A remembered treasure it can beat, else one it has not seen, else any.
        let mut choices = Vec::new();
        if skill > 0 {
            choices = open(&|treasure| treasure.seen && beats(treasure));
            if choices.is_empty() {
                choices = open(&|treasure| !treasure.seen);
            }
        }
        if choices.is_empty() {
            choices = open(&|_| true);
        }
        let at = self.pick(choices.len());
        choices.get(at).copied().unwrap_or(0)
    }
}

fn duel_text(mine: Card, theirs: Card, outcome: u8) -> String {
    format!("TW_Duel\t{}\t{}\t{outcome}", mine.token(), theirs.token())
}

fn shuffle(cards: &mut [Card], rng: &mut Rng) {
    for last in (1..cards.len()).rev() {
        cards.swap(last, (rng.next_u64() % (last as u64 + 1)) as usize);
    }
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use std::path::PathBuf;

    pub(crate) fn content() -> Arc<Content> {
        let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        Arc::new(Content::load(&folder).unwrap())
    }

    fn card(color: Color, power: u8) -> Card {
        Card {
            id: 1,
            name_id: 1,
            color,
            power,
            evidence: Evidence::R,
        }
    }

    fn texts(said: &[Said], seat: usize) -> Vec<&str> {
        said.iter()
            .filter(|(to, _)| *to == seat)
            .map(|(_, text)| text.as_str())
            .collect()
    }

    /// A game where seat 0 holds only `mine` and seat 1 only `theirs`, treasures included.
    fn rigged(mine: Card, theirs: Card) -> Match {
        let mut game = Match::new(content(), Rng::seeded(7));
        for (side, card) in game.sides.iter_mut().zip([mine, theirs]) {
            side.hand.fill(card);
            side.deck.fill(card);
            side.treasures
                .iter_mut()
                .for_each(|treasure| treasure.card = card);
        }
        game
    }

    #[test]
    fn the_shipped_content_loads_closed_and_deals_a_whole_table() {
        let content = content();
        assert!(
            !content.open,
            "the movie has not shipped: the row stays closed"
        );
        assert_eq!((content.hand_size, content.treasures), (4, 4));
        for color in [Color::Red, Color::Green, Color::Blue] {
            assert!(content.cards.iter().any(|card| card.color == color));
        }
        let game = Match::new(content.clone(), Rng::seeded(1));
        for side in &game.sides {
            assert_eq!((side.hand.len(), side.treasures.len()), (4, 4));
            assert_eq!(side.deck.len(), content.deck_size - 8);
        }
    }

    #[test]
    fn the_colour_wheel_then_power_then_a_tie() {
        use Color::*;
        // The filmed duel: Robgoblin Wizard, blue 6, beat Scaled Dragon, red 4.
        assert_eq!(card(Blue, 6).duel(&card(Red, 4)), Ordering::Greater);
        assert_eq!(card(Red, 0).duel(&card(Green, 12)), Ordering::Greater);
        assert_eq!(card(Green, 0).duel(&card(Blue, 12)), Ordering::Greater);
        assert_eq!(card(Green, 12).duel(&card(Red, 0)), Ordering::Less);
        assert_eq!(card(Red, 7).duel(&card(Red, 4)), Ordering::Greater);
        assert_eq!(card(Red, 4).duel(&card(Red, 4)), Ordering::Equal);
    }

    #[test]
    fn a_pick_stays_hidden_until_both_are_in() {
        let mut game = rigged(card(Color::Blue, 6), card(Color::Red, 4));
        let said = game.play(0, 0).unwrap();
        assert!(texts(&said, 1).contains(&"TW_Duel\t-\t?\t0"), "{said:?}");
        assert!(
            texts(&said, 1).iter().all(|text| !text.contains(",2,6")),
            "the card leaked"
        );
        assert!(game.play(0, 0).is_none(), "one card a round");
        assert_eq!(game.waiting_on(), Some(1));
        let said = game.play(1, 0).unwrap();
        assert!(texts(&said, 0).contains(&"TW_Duel\t1,1,2,6\t1,1,0,4\t1"));
        assert!(texts(&said, 1).contains(&"TW_Duel\t1,1,0,4\t1,1,2,6\t2"));
        assert!(
            texts(&said, 0)
                .iter()
                .any(|text| text.starts_with("TW_Phase\tattack\t433303"))
        );
        assert!(
            texts(&said, 1)
                .iter()
                .any(|text| text.starts_with("TW_Phase\tdefend\t433299"))
        );
    }

    #[test]
    fn four_treasures_taken_wins_and_only_the_winner_attacks() {
        let mut game = rigged(card(Color::Blue, 6), card(Color::Red, 4));
        for slot in 0..4 {
            game.play(0, 0).unwrap();
            game.play(1, 0).unwrap();
            assert!(game.take(1, slot).is_none(), "the loser does not attack");
            assert!(game.take(0, 9).is_none(), "no such treasure");
            assert_eq!(game.result(), None);
            let said = game.take(0, slot).unwrap();
            assert!(texts(&said, 0)[0].starts_with(&format!("TW_Flip\t1\t{slot}\t1,1,0,4\t1")));
            assert!(texts(&said, 1)[0].starts_with(&format!("TW_Flip\t0\t{slot}\t")));
            assert!(game.take(0, slot).is_none(), "a taken treasure is gone");
        }
        assert_eq!(game.result(), Some(Some(0)));
        assert_eq!((game.taken_by(0), game.taken_by(1)), (4, 0));
        assert!(game.play(0, 0).is_none() && game.waiting_on().is_none());
        let seen = game.snapshot(1);
        assert!(texts(&seen, 1).contains(&"TW_Board\ttttt\tdddd"));
        assert!(texts(&seen, 1).contains(&"TW_Phase\tlost\t433308\t0\t4"));
    }

    #[test]
    fn a_treasure_that_defends_goes_back_and_the_round_ends() {
        let mut game = rigged(card(Color::Blue, 6), card(Color::Red, 4));
        // Seat 1's treasures are green: blue loses to green.
        game.sides[1]
            .treasures
            .iter_mut()
            .for_each(|t| t.card = card(Color::Green, 1));
        game.play(0, 0).unwrap();
        game.play(1, 0).unwrap();
        let said = game.take(0, 2).unwrap();
        assert!(texts(&said, 0)[0].ends_with("\t0"), "not taken: {said:?}");
        assert!(
            texts(&said, 0).contains(&"TW_Board\tdddd\tdddd"),
            "it went back face down"
        );
        assert!(
            texts(&said, 0)
                .iter()
                .any(|text| text.starts_with("TW_Phase\tchoose"))
        );
        assert_eq!(game.sides[0].hand.len(), 4, "the played card was replaced");
        assert!(game.sides[1].treasures[2].seen);
    }

    #[test]
    fn a_tie_discards_both_cards_and_the_round_cap_ends_a_level_game() {
        let mut game = rigged(card(Color::Red, 4), card(Color::Red, 4));
        let rounds = game.content.max_rounds;
        for round in 0..rounds {
            assert_eq!(game.result(), None, "round {round}");
            game.play(1, 0).unwrap();
            let said = game.play(0, 0).unwrap();
            assert!(
                texts(&said, 0).iter().any(|text| text.ends_with("\t3")),
                "a tie: {said:?}"
            );
            assert_eq!(
                game.sides[0].hand.len(),
                4,
                "the deck recycles its discards"
            );
        }
        assert_eq!(
            game.result(),
            Some(None),
            "level on treasures: nobody's win"
        );
        assert!(texts(&game.snapshot(0), 0).contains(&"TW_Phase\tlost\t433308\t0\t0"));
    }

    #[test]
    fn computers_of_every_skill_finish_a_game_by_the_rules() {
        for skill in 0..=2 {
            for seed in 0..40 {
                let mut game = Match::new(content(), Rng::seeded(seed));
                let mut steps = 0;
                while game.result().is_none() {
                    let seat = game.waiting_on().unwrap();
                    assert!(!game.play_computer(seat, skill).is_empty());
                    steps += 1;
                    assert!(steps < 10_000, "skill {skill} seed {seed} never ends");
                }
                let (first, second) = (game.taken_by(0), game.taken_by(1));
                match game.result().unwrap() {
                    Some(winner) => {
                        let loser = game.taken_by(1 - winner);
                        assert!(game.taken_by(winner) > loser, "{first} to {second}");
                    }
                    None => assert_eq!(first, second),
                }
            }
        }
    }

    #[test]
    fn a_computer_that_remembers_goes_back_for_a_treasure_it_can_beat() {
        let mut game = rigged(card(Color::Blue, 6), card(Color::Red, 4));
        game.sides[1].treasures[3].seen = true;
        assert_eq!(game.computer_treasure(0, 1, card(Color::Blue, 6)), 3);
        // One it saw and cannot beat is left for last.
        game.sides[1].treasures[3].card = card(Color::Green, 1);
        assert_ne!(game.computer_treasure(0, 1, card(Color::Blue, 6)), 3);
    }

    #[test]
    fn bad_content_is_refused() {
        let good = (*content()).clone();
        let broken: [fn(&mut Content); 5] = [
            |c| c.deck_size = c.hand_size + c.treasures,
            |c| c.cards[1].id = c.cards[0].id,
            |c| c.cards[0].power = 13,
            |c| c.opponents.clear(),
            |c| c.max_rounds = 0,
        ];
        for breakage in broken {
            let mut content = good.clone();
            breakage(&mut content);
            assert!(content.validate().is_err());
        }
    }
}
