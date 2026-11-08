//! T504: the card-game activity rows (Trading Card Game, category 11, and Treasure War, category
//! 26) the activity portal publishes, loaded from `data/content/card_game.json`.
//!
//! # Recovered original facts (addresses in `FreeRealms.exe`)
//!
//! * `TradingCardGame.StartTutorial` (`00c04970`) and `StartTreasureWar` (`00c049f0`) scan the
//!   settings list for the keys `0x6D54A188` / `0xBFD77CB0` (Lua globals `Tcg.ActivityId` and
//!   `Scg.ActivityId`, registered by PlayClient init `0092e3bc`) and fall back to 1049.0 and
//!   1078.0, then call `ActivityJoinRequest_SendByType` (`00be7d80`). Those defaults are the
//!   activity ids of the tutorial row and of Treasure War here.
//! * Every other card button (welcome window, utility window, store and inventory item use,
//!   marketplace) ends in the same 167/1/2 join, so one refusal covers them all.
//!
//! # Reconstructed policy (R)
//!
//! * Without a `launch` block the portal lists these rows and answers Play with a chat refusal
//!   (`activity_portal::Refusal`). With one (T765), Play on a Trading Card Game row starts a
//!   type-16 run (`Launches`) whose 39/37 gets a one-time launch ticket in 39/0x28; Treasure
//!   War (type 23) stays refused.
//! * T795, the start order the client's own code follows (`ClientTradingCardGameManager`):
//!   39/16 (naming the card asset list, which is what starts the client's loader) -> the client
//!   downloads its card assets and sends 39/37 (`00ad4f50` state 2, `009bf590`), answered while
//!   the run is still offered with 39/0x28 (slot 25 `00ad32e0`: stores the ticket, enables the
//!   Start button) -> the Start click 39/5 -> 39/17 (`009b9550` runs `Startup` `00ad47f0` on the
//!   stored ticket) -> the card window's close, 39/6 -> 39/19, and the client's 39/7. The card
//!   window's own asset download needs the client's `[AssetDelivery] TcgServerAddress` to answer
//!   `manifest.crc` (`0109bd00`); `scripts/asset_delivery.py` serves it for the local harness.
//!   T849: the card manager's state 1 loads `TradingCardAssets.txt` (`00ad4f50`), the client's
//!   `Resources` list of the TCG `.lst` and `.dds` art; that is the 39/16 preload name too.
//! * Rows publish entry type 1 (the world tunnel); `server_type` (Legacy's 2, the client tunnel
//!   `00be78c0`) stays in data for the launch slice.
//! * The row's MiniGameData link (`minigame_id`) is not in the published row; it names the run and
//!   picks the client's scenario (`SCENARIO_BLOCKS`).
//! * T545: `type16_minigame_ids` lists every client `MiniGameData.txt` TYPE 16 row. The admin
//!   `realm duel` credit refuses every other id, so a kart, derby or Treasure War row cannot
//!   stand in for a card duel.
//! * L935: the card window plays Tutorials, Scenarios and Skirmishes itself (its own rules engine
//!   and AI) and reports them (`AccountCommand_RecordSinglePlayerGame`, class 487); `Launches::record`
//!   turns a bounded win into a [`Win`] for the row the run was launched from (spec:
//!   lab/autopilot/opus/minigame-notes/L935-tcg-protocol.md). Server-hosted matches and NPC decks
//!   would need a rules engine this repo does not have.

use crate::activity_portal::{Activity, Kind};
use crate::card_service::{LaunchTickets, SinglePlayerRecord};
use crate::minigame::{MinigameRuns, RunMode, RunOutcome, RunSpec, RunState, card_start_packet};
use crate::startup_messages;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

/// Optional content next to `field_battles.json`. An absent file publishes no rows.
pub const CONTENT_FILE: &str = "card_game.json";
/// Client `ActivityCategories.txt` row 11, "Trading Card Game" (CONTENT_ID 36).
pub const TRADING_CARD_CATEGORY: u32 = 11;
/// Client `ActivityCategories.txt` row 26, "Treasure War" (CONTENT_ID 36).
pub const TREASURE_WAR_CATEGORY: u32 = 26;
const MAX_FILE: u64 = 1 << 20;
const MAX_ROWS: usize = 32;
const MAX_HIDDEN_ROWS: usize = 200;
const MAX_IMAGE_NAME: usize = 64;
/// T545: bound on the client's TYPE 16 id list (the client table has 124 such rows).
const MAX_TYPE16_IDS: usize = 1024;

/// One published row: the values of one 167/1/1 row (client reader `00be63e0`) plus provenance.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub id: u32,
    /// The client `MiniGameData.txt` row this activity opens (see the module docs).
    pub minigame_id: u32,
    pub app_system_id: u32,
    pub category: u32,
    /// Row `+0x14`; read as an `Images.txt` id (see the row's `source`).
    pub image_set_id: u32,
    pub position: u32,
    /// Legacy's entry type; the server publishes entry type 1 instead (R).
    pub server_type: u32,
    pub display_name_id: u32,
    pub display_description_id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub can_player_join: bool,
    #[serde(default)]
    pub tutorial_activity_id: Option<u32>,
    pub detail_image: String,
    pub thumbnail_image: String,
    /// The lobby is -1; the wire carries the i32 bit pattern (R).
    pub difficulty: i32,
    pub source: Vec<Source>,
}

/// `(first row, last row, first scenario)`: the client script `MMOScenario_N` of each
/// `MiniGameData.txt` card row. The client's index has 1..=37 and 39..=121 and no row list; 38 is
/// its built-in tutorial (`104b9160`). Each owner's rows are one run of numbers, and the DLL's
/// intro switch `1047aac0` names the owners (N 6 "says Poe", 55 Tyrone, 59 Sir Wellington, 77
/// Jonathon, 89 Darkcheat, 113 Jort, 121 Ironteeth), which pins the blocks. (R) 39, 64 and 65
/// (rows 661, 710, 711): no owner names them; they follow the row order.
const SCENARIO_BLOCKS: [(i32, i32, i32); 7] = [
    (500, 537, 1),
    (661, 661, 39),
    (666, 669, 40),
    (684, 703, 44),
    (710, 711, 64),
    (387, 426, 66),
    (1083, 1098, 106),
];

/// The client scenario a card row launches; `None` for the rows that open the lobby.
fn scenario_of(minigame_id: i32) -> Option<i32> {
    SCENARIO_BLOCKS.iter().find_map(|&(first, last, scenario)| {
        (first..=last)
            .contains(&minigame_id)
            .then(|| scenario + minigame_id - first)
    })
}

/// (R) The decks one NPC duels with: Sam Potts opened her deck's offer from her own Talk (the
/// sheet's `iud_NyVa_2o_3884.00`), and each row names its NPC.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct NpcDecks {
    pub npc_id: u64,
    pub activities: Vec<u32>,
}

/// One evidence entry (`legacy-data`, `client` or `reconstructed`), the shape `content_check.py` counts.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub kind: String,
    pub evidence: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Content {
    schema_version: u32,
    #[serde(rename = "source")]
    _source: String,
    rows: Vec<Row>,
    /// (R) Decks with no Games-window row (108 of the 124 card rows): only their NPC offers them.
    #[serde(default)]
    hidden_rows: Vec<Row>,
    /// (R) Which NPC offers which deck.
    #[serde(default)]
    npc_decks: Vec<NpcDecks>,
    /// T765: present once a card service runs; until then every card row stays refused.
    #[serde(default)]
    launch: Option<Launch>,
    /// T545: every client `MiniGameData.txt` row whose TYPE column is 16 (the card duels),
    /// sorted. The developer `realm duel` credit refuses any other id; `type16_source`
    /// below names the table, its hash and the column. A pre-T545 content tree has no list
    /// (default empty), so [`Content::is_duel_row`] refuses every id there: fail closed.
    #[serde(default)]
    type16_minigame_ids: Vec<u32>,
    /// T545: the one `client` provenance entry for `type16_minigame_ids`; `None` only in a
    /// pre-T545 tree, which has no list either.
    #[serde(default)]
    type16_source: Option<Source>,
}

/// T765: how a Trading Card Game row launches (R values, bounded in `validate`).
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Launch {
    /// How long a 39/0x28 launch ticket stays redeemable by the card login.
    pub ticket_ttl_ms: u64,
    /// T770: the card service's TCP port beside the gateway (the client's `host.svr` `gport`);
    /// 0 keeps it off, and the card window then reports the service unreachable.
    #[serde(default)]
    pub port: u16,
    /// T795: the asset list the offer names (default [`PRELOAD_FILE`], the client's
    /// `TradingCardAssets.txt` card art list); never empty.
    #[serde(default = "default_preload_file")]
    pub preload_file: String,
    /// T813: the lobby a card login enters; without it the card window stays on "Entering main
    /// lobby..." after its login.
    #[serde(default)]
    pub lobby: Option<Lobby>,
    /// L935: what a single-player win reported by the card window pays; without it a launch
    /// block opens the window but no game result counts.
    #[serde(default)]
    pub duel: Option<Duel>,
    /// (R) What each deck pays the first time a character beats it: "you will win 3 cards from
    /// that player", once per deck (wiki). Which cards is not recorded anywhere; see
    /// `prizes_source`.
    #[serde(default)]
    pub prizes: Vec<DeckPrize>,
    #[serde(default)]
    pub prizes_source: Vec<Source>,
    /// (R) What a bought pack opens into.
    #[serde(default)]
    pub packs: Option<CardPacks>,
    pub source: Vec<Source>,
}

/// (R) Booster packs and starter decks, bought from an NPC's menu and opened on purchase. The
/// client's merchant window keeps its Buy button off for these items.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CardPacks {
    pub boosters: Vec<BoosterPack>,
    pub pools: BTreeMap<String, RarityPool>,
    pub starter_decks: Vec<DeckPack>,
    pub sales: Vec<PackSale>,
    pub source: Vec<Source>,
}

/// One menu row: `npc_id` sells `item_id` for `coins`, labelled with the item's own name.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackSale {
    pub npc_id: u64,
    pub item_id: i32,
    pub name_id: u32,
    pub coins: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BoosterPack {
    pub item_id: i32,
    pub series: u32,
}

/// One card series' archetypes by rarity.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RarityPool {
    pub common: Vec<i32>,
    pub uncommon: Vec<i32>,
    pub rare: Vec<i32>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeckPack {
    pub item_id: i32,
    /// One row per copy.
    pub cards: Vec<i32>,
}

impl CardPacks {
    /// The cards `item_id` opens into, or `None` when it isn't a pack. A booster is 1 rare, 3
    /// uncommon and 6 common cards (wiki), picked from `seed`; a starter deck is fixed.
    pub fn open(&self, item_id: i32, mut seed: u64) -> Option<Vec<i32>> {
        if let Some(deck) = self.starter_decks.iter().find(|deck| deck.item_id == item_id) {
            return Some(deck.cards.clone());
        }
        let booster = self.boosters.iter().find(|pack| pack.item_id == item_id)?;
        let pool = self.pools.get(&booster.series.to_string())?;
        let mut pick = |tier: &[i32], count: usize| -> Vec<i32> {
            (0..count)
                .map(|_| {
                    // ponytail: xorshift, not a dice roll anyone can bet on; nothing here is scarce.
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    tier[(seed % tier.len() as u64) as usize]
                })
                .collect()
        };
        let mut cards = pick(&pool.rare, 1);
        cards.extend(pick(&pool.uncommon, 3));
        cards.extend(pick(&pool.common, 6));
        Some(cards)
    }

    fn valid(&self) -> bool {
        let pool_ok = |p: &RarityPool| {
            [&p.common, &p.uncommon, &p.rare].iter().all(|t| !t.is_empty() && t.iter().all(|c| *c > 0))
        };
        !self.source.is_empty()
            && self.pools.values().all(pool_ok)
            && self.boosters.iter().all(|b| self.pools.contains_key(&b.series.to_string()))
            && self.starter_decks.iter().all(|d| d.cards.len() >= 40 && d.cards.iter().all(|c| *c > 0))
            && self.sales.iter().all(|s| s.coins > 0 && self.open(s.item_id, 1).is_some())
    }
}

/// L935: how a Skirmish or Scenario win reported by the card window pays (R values, bounded in
/// `validate`). The DLL plays those games itself and reports the result, so the server bounds
/// what it accepts instead of replaying it (spec: L935-tcg-protocol.md).
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Duel {
    /// `AccountCommand_RecordSinglePlayerGame` game types that pay: 3 Skirmish, 2 Scenario (the
    /// scripted Tutorial, type 1, stays out).
    pub game_types: Vec<i32>,
    /// A win reported less than this many server seconds after its start record pays nothing.
    pub min_seconds: u32,
    /// Coins one qualifying win pays; the Card Duelist's `activity_xp` rides on top.
    pub win_coins: u32,
}

/// One deck's first-win prize: card archetypes, one copy each.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeckPrize {
    pub minigame_id: u32,
    pub cards: Vec<i32>,
}

/// The most copies of one card a sync carries for a character (the client's own `default.cln`
/// holds up to 100).
const MAX_EARNED_COPIES: i32 = 100;
/// ponytail: a card login writes every copy in one frame (about 10 bytes each), so purchases stop
/// here; split that frame if a player ever needs more.
const MAX_TOTAL_COPIES: usize = 4_000;

/// The cards a character has earned beyond the starter set, and the decks that already paid.
/// Saved; the card window learns them from the collection sync at its next login.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EarnedCards {
    #[serde(default)]
    pub copies: BTreeMap<i32, i32>,
    #[serde(default)]
    pub beaten: BTreeSet<u32>,
}

impl EarnedCards {
    pub fn is_empty(&self) -> bool {
        self.copies.is_empty() && self.beaten.is_empty()
    }

    /// Adds one copy of each card.
    pub fn add(&mut self, cards: &[i32]) {
        for card in cards {
            *self.copies.entry(*card).or_default() += 1;
        }
    }

    /// Whether every card in `cards` can still be kept, so a purchase never takes coins for nothing.
    pub fn has_room_for(&self, cards: &[i32]) -> bool {
        let total: usize = self.copies.values().map(|&held| held.max(0) as usize).sum();
        total + cards.len() <= MAX_TOTAL_COPIES
            && cards.iter().all(|card| {
                let bought = cards.iter().filter(|other| *other == card).count() as i32;
                self.copies.get(card).copied().unwrap_or(0) + bought <= MAX_EARNED_COPIES
            })
    }

    /// Pays `prize` the first time `deck` is beaten; true when it paid.
    pub fn pay_prize(&mut self, deck: u32, prize: &[i32]) -> bool {
        let first = self.beaten.insert(deck);
        if first {
            self.add(prize);
        }
        first
    }

    /// One row per card, each count bounded: the save is a file, and a sync sends every copy.
    pub fn rows(&self) -> Vec<StarterCard> {
        self.copies
            .iter()
            .map(|(&archetype_id, &copies)| StarterCard {
                archetype_id,
                copies: copies.clamp(1, MAX_EARNED_COPIES),
            })
            .collect()
    }
}

/// `starter` with `earned` folded in, one row per archetype in `starter`'s order.
pub fn collection_with(starter: &[StarterCard], earned: &[StarterCard]) -> Vec<StarterCard> {
    let mut all = starter.to_vec();
    for card in earned {
        match all.iter_mut().find(|row| row.archetype_id == card.archetype_id) {
            Some(row) => row.copies += card.copies,
            None => all.push(card.clone()),
        }
    }
    all
}

/// The Card Duelist job profile (`job_xp_curve.json`, 466 `activity_xp` per duel).
pub const CARD_DUELIST_PROFILE: i32 = 120;

/// L935: a win the card window reported that passed every server bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Win {
    /// The MiniGameData row the run was launched from.
    pub minigame_id: u32,
    /// The client's game start time: unique per game, it keys the payout.
    pub started: i32,
    /// Server seconds from the start record to the end record.
    pub elapsed_seconds: u32,
    pub coins: u32,
}

/// T813: the one lobby group the card service defines for a logged-in card window (R: the
/// client data names no lobby; bounded in `validate`). Group 1 is the client's root lobby: on
/// learning it the client asks to join it itself (`10371960`).
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Lobby {
    /// The group id (`Lobby` GroupID), the key every lobby lookup uses (`101717d0`). Only the
    /// client's root group 1 is supported: the client's root Join is fixed at 1 (`10371960`), so
    /// any other group would be defined but never entered (`validate`).
    pub group_id: i32,
    /// `eLobbyTypeID` (R).
    pub lobby_type: i32,
    /// The player's role in it: 1 general, 2 player, 3 observer, 4 moderator, 5 staff (the
    /// `Lobby` writer's role lists, `101558e0`); 0 is the client's invalid role.
    pub role: i32,
    /// T825: the starter collection the card service syncs after the lobby. R:
    /// the client data holds no starter deck list (the skirmish's "Digital Starter" decks,
    /// `104ab920`, are not in `tcg/data/decks`), so the ids are chosen from the client's own card
    /// data: archetypes of `tcg/data/archetypes/storage.dat` that its `default.cln` collection
    /// also lists.
    #[serde(default)]
    pub collection: Vec<StarterCard>,
}

/// T825: one card archetype of the starter collection and how many copies of it the player owns.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StarterCard {
    pub archetype_id: i32,
    pub copies: i32,
}

fn default_preload_file() -> String {
    PRELOAD_FILE.to_owned()
}

/// The client's mini-game type of the DLL card game (`MiniGameTypeData` 16; `00ad47f0`).
pub const CARD_GAME_TYPE: i32 = 16;
/// The card art list the 39/16 offer names. T849 recovered the client's own name: its card
/// manager loads `TradingCardAssets.txt` in state 1 (`00ad4f50`, string at `00ad4f9e`; the
/// loader is `00ae7950`). The client's `Resources/TradingCardAssets.txt` (3,864 bytes) names 45
/// TCG `.lst` UI lists and 151 `.dds` card images. The previous value,
/// `TradingCardGameAssets.txt`, is a real main-pack list (`Assets_009.pack` entry 3,843,
/// 1,523 bytes) but it holds 77 `TCGNarrator*.mp3` narration files, so the 39/16 loader never
/// loaded card art. (R) that the retail server named this list; the client's card manager
/// hardcodes it and `MiniGameData`'s `PRELOAD_FILE` column is empty on every row.
pub const PRELOAD_FILE: &str = "TradingCardAssets.txt";
/// (R) Card runs take their own instance range, apart from chess (1), soccer (47,000,001)
/// and harvesting (58,000,001).
pub const FIRST_INSTANCE_ID: i32 = 61_000_001;
/// The client's results Close (`CommandPacketClosedMinigameEndScreen`, constructor `008a2eb0`):
/// the same 26/42 a shared run retires on after a win or loss. (R) that a terminal card run's
/// results window sends this one; the card window's results are the DLL's own (`Launches::record`
/// settles them), so no run reaches Won or Lost today.
const RESULTS_CLOSE: [u8; 4] = [26, 0, 42, 0];

/// T765: one session's card launches: the type-16 run lifecycle and the one-time tickets its
/// 39/37 mints. Session-only; never saved.
#[derive(Clone, Debug)]
pub struct Launches {
    runs: MinigameRuns,
    tickets: LaunchTickets,
    preload_file: String,
    duel: Option<Duel>,
    /// L935: the live run's (instance, MiniGameData id), set when its row is played.
    entered: Option<(i32, i32)>,
    /// L935: the single-player game in progress: the client's start time and the server clock
    /// (ms) when its start record arrived.
    playing: Option<(i32, u64)>,
    /// L935: the newest game start accepted; a start record must be newer, so a replayed record
    /// of an earlier game pays nothing.
    last_started: Option<i32>,
}

impl Launches {
    pub fn new(launch: &Launch) -> io::Result<Self> {
        Ok(Self {
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            tickets: LaunchTickets::new(launch.ticket_ttl_ms),
            preload_file: launch.preload_file.clone(),
            duel: launch.duel.clone(),
            entered: None,
            playing: None,
            last_started: None,
        })
    }

    /// L935: take the card window's single-player record for the live run. A start record
    /// remembers the game; an end record that names it settles it exactly once, and only a win
    /// of a paying game type, `min_seconds` of server time after the start, comes back as a
    /// [`Win`]. A record with no live run, no duel settings, an older start or no matching start
    /// changes nothing and pays nothing.
    pub fn record(&mut self, record: &SinglePlayerRecord, now_ms: u64) -> Option<Win> {
        let (instance, minigame_id) = self.entered?;
        if self.runs.current_instance_id() != Some(instance) {
            self.entered = None;
            self.playing = None;
            return None;
        }
        let duel = self.duel.as_ref()?;
        match record.phase {
            SinglePlayerRecord::STARTED => {
                if self.last_started.is_none_or(|last| record.started > last) {
                    self.last_started = Some(record.started);
                    self.playing = Some((record.started, now_ms));
                }
                None
            }
            SinglePlayerRecord::ENDED => {
                let (started, at_ms) = self.playing.take_if(|(s, _)| *s == record.started)?;
                let elapsed_seconds =
                    u32::try_from(now_ms.saturating_sub(at_ms) / 1_000).unwrap_or(u32::MAX);
                (record.result == SinglePlayerRecord::WON
                    && duel.game_types.contains(&record.game_type)
                    && elapsed_seconds >= duel.min_seconds)
                    .then_some(Win {
                        minigame_id: u32::try_from(minigame_id).ok()?,
                        started,
                        elapsed_seconds,
                        coins: duel.win_coins,
                    })
            }
            _ => None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.runs.current_instance_id().is_some()
    }

    /// Whether `packet` belongs to this session's current or just-ended run. Nothing else is the
    /// run's: the generic 26/42 Close the client sends first when its start screen is closed would
    /// retire an offered run before the 39/6 cancel that follows (T782). The 26/42 *is* the run's
    /// once the run is terminal (Won/Lost): it is the client's results Close, and a run that no
    /// owner retires would stay current and keep the player busy for the rest of the session.
    pub fn owns(&self, packet: &[u8]) -> bool {
        if packet == RESULTS_CLOSE {
            return self.runs.current_instance_id().is_some_and(|instance| {
                matches!(
                    self.runs.state(instance),
                    Some(RunState::Won | RunState::Lost)
                )
            });
        }
        self.runs.names_run(packet)
    }

    /// Offer the row's run (39/16). The card asset list in it starts the client's loader, which
    /// ends in the client's 39/37 ticket request and then its 39/5 Start click. The flags stay
    /// the client default: `+616` = 1 makes the server's 39/19 on leaving take the standard
    /// end-of-game path (`009bf190` -> `009b7170`). Clearing it instead runs Lua
    /// `:QuitGameWithNoEndGameScreen`, which re-enters the minigame leave while the 39/19
    /// handler is on the stack and overflows the client's stack (T856, native 0xC00000FD).
    pub fn begin(&mut self, row: &Row) -> io::Result<RunOutcome> {
        let game_id = i32::try_from(row.minigame_id)
            .map_err(|_| invalid("card mini-game id is out of range"))?;
        let mut spec = RunSpec::empty(game_id, CARD_GAME_TYPE, RunMode::CardClient);
        spec.preload_file.clone_from(&self.preload_file);
        (spec.name_id, spec.description_id) = (row.name_id as i32, row.description_id as i32);
        spec.difficulty = row.difficulty.max(1);
        let offered = self.runs.offer(spec)?;
        if offered.accepted {
            self.entered = self.runs.current_instance_id().map(|run| (run, game_id));
            self.playing = None;
        }
        Ok(offered)
    }

    /// The client scenario of the row in play, for the card login's challenge.
    pub fn scenario(&self) -> Option<i32> {
        let (run, row) = self.entered?;
        scenario_of(row).filter(|_| self.runs.current_instance_id() == Some(run))
    }

    /// The run's lifecycle packets (39/5 start, 39/6 leave, 39/7 ack). A 39/37, while the run is
    /// offered or later, mints the character's launch ticket and answers 39/0x28; the Start
    /// click is refused until one was sent. T866: an accepted Start renews that ticket's expiry
    /// (same value), so a run panel read past `ticket_ttl_ms` no longer refuses the card login.
    pub fn route(&mut self, packet: &[u8], character: u64, now_ms: u64) -> io::Result<RunOutcome> {
        let mut outcome = self.runs.handle_client(packet)?;
        if let Some(request) = outcome.card_ticket {
            let ticket = self.tickets.mint(character, now_ms)?;
            outcome.replies.push(card_start_packet(request, ticket));
        }
        // Only an accepted Start changes the run to Starting (`MinigameRuns::start`), so both
        // halves of the guard are needed. A repeated 39/37 *re-mints* - even while Starting,
        // replacing the ticket the client already passed to the card DLL - and the 39/7 ack
        // reports Starting; both leave `changed` false and must not renew. The other changed
        // outcomes (39/6 leave, 26/42 close) are not Starting.
        if outcome.changed && outcome.state == Some(RunState::Starting) {
            self.tickets.renew(character, now_ms);
        }
        Ok(outcome)
    }

    /// The card login's ticket and the character id it carries, if any (T766's listener): `Some`
    /// holds the character on success or the refusal word ([`LaunchTickets::redeem`]), `None`
    /// when this session's store does not hold the ticket. A refusal changes nothing; the retail
    /// 411's character id is empty (T804), so the ticket alone finds the session.
    pub fn redeem(
        &mut self,
        ticket: &str,
        character: &str,
        now_ms: u64,
    ) -> Option<Result<u64, &'static str>> {
        self.tickets.redeem(ticket, character, now_ms)
    }
}

/// T798: the 82 a card-enabled session sends once per login (`PacketUpdateClientSessionData`).
/// The card window starts its session login only for a non-empty session ticket (DLL
/// `1037b5f0`), and the exe's card manager (`00ad47f0`) takes that ticket and the station name
/// from this packet only. The ticket is a fresh random value that is not a credential: the card
/// login is authorised by the one-time launch ticket (`LaunchTickets`; a character id the 411
/// carries, if any, must name the owner, and the retail client sends none, T804), and the card
/// service only echoes the session id back. The client logs it and the DLL sends it in plain
/// TCP, so it is never the launcher session, the gateway ticket or a launch ticket. The station
/// name and realms stay empty (R): the card service needs none of them.
pub fn session_data_packet() -> io::Result<Vec<u8>> {
    let mut bytes = [0; 16];
    getrandom::getrandom(&mut bytes)
        .map_err(|_| invalid("no OS randomness for a card session ticket"))?;
    let ticket = format!("{:032x}", u128::from_le_bytes(bytes));
    Ok(startup_messages::build_update_client_session_data(
        &ticket, "", "", "",
    ))
}

impl Content {
    /// `None` when the content directory has no card file.
    pub fn load_optional(directory: &Path) -> io::Result<Option<Arc<Self>>> {
        let path = directory.join(CONTENT_FILE);
        path.is_file()
            .then(|| Self::load(&path).map(Arc::new))
            .transpose()
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(path)?
            .take(MAX_FILE + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_FILE {
            return Err(invalid("card game content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(|e| invalid(&e.to_string()))?;
        content.validate()?;
        Ok(content)
    }

    fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1 {
            return Err(invalid("unsupported card game content version"));
        }
        if self.rows.is_empty() || self.rows.len() > MAX_ROWS {
            return Err(invalid("card game content needs 1 to 32 rows"));
        }
        if self.hidden_rows.len() > MAX_HIDDEN_ROWS
            || self.npc_decks.iter().flat_map(|npc| &npc.activities).any(|id| self.row(*id).is_none())
        {
            return Err(invalid("card game NPC decks name a row the content lacks"));
        }
        if self.type16_minigame_ids.len() > MAX_TYPE16_IDS
            || self
                .type16_minigame_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || self.type16_minigame_ids.contains(&0)
        {
            return Err(invalid(
                "card TYPE 16 ids must be at most 1024 sorted unique nonzero ids",
            ));
        }
        // A pre-T545 tree has neither field; otherwise the list and its one `client` source
        // travel together, so a content edit cannot drop the provenance silently.
        match (&self.type16_source, self.type16_minigame_ids.is_empty()) {
            (None, true) => {}
            (Some(source), false)
                if source.kind == "client" && !source.evidence.trim().is_empty() => {}
            _ => {
                return Err(invalid(
                    "card TYPE 16 ids need one client source and a non-empty list",
                ));
            }
        }
        let mut ids = BTreeSet::new();
        // Only the Trading Card Game launches from an NPC's menu, and only a joinable row.
        if self
            .hidden_rows
            .iter()
            .any(|row| row.category != TRADING_CARD_CATEGORY || !row.can_player_join)
        {
            return Err(invalid("card game hidden rows are joinable category 11 rows"));
        }
        for row in self.rows.iter().chain(&self.hidden_rows) {
            if row.id == 0 || !ids.insert(row.id) {
                return Err(invalid("card game ids must be unique and nonzero"));
            }
            if ![TRADING_CARD_CATEGORY, TREASURE_WAR_CATEGORY].contains(&row.category) {
                return Err(invalid("card game rows are category 11 or 26"));
            }
            if ![&row.detail_image, &row.thumbnail_image]
                .into_iter()
                .all(|name| is_name(name))
            {
                return Err(invalid("card game image names are 1 to 64 ASCII bytes"));
            }
            if row.source.is_empty() {
                return Err(invalid("card game rows need a source"));
            }
        }
        if self
            .rows
            .iter()
            .filter_map(|row| row.tutorial_activity_id)
            .any(|tutorial| !ids.contains(&tutorial))
        {
            return Err(invalid("card game tutorial id is not a row of the file"));
        }
        if self.launch.as_ref().is_some_and(|launch| {
            !(1_000..=600_000).contains(&launch.ticket_ttl_ms) || launch.source.is_empty()
        }) {
            return Err(invalid(
                "card launch ticket lifetime is 1 s to 10 min, with a source",
            ));
        }
        // An empty list never starts the client's loader: the start screen stays on "Loading...".
        if self
            .launch
            .as_ref()
            .is_some_and(|launch| !is_name(&launch.preload_file))
        {
            return Err(invalid("card launch asset list is 1 to 64 ASCII bytes"));
        }
        // L935 (R) bounds: the DLL has three single-player game types (1 to 3), each named once; a
        // win must take a moment (an hour at most) and pay a small purse.
        if self
            .launch
            .as_ref()
            .and_then(|launch| launch.duel.as_ref())
            .is_some_and(|duel| {
                duel.game_types.is_empty()
                    || duel.game_types.len() > 3
                    || duel.game_types.iter().any(|kind| !(1..=3).contains(kind))
                    || duel.game_types.iter().collect::<BTreeSet<_>>().len()
                        != duel.game_types.len()
                    || !(1..=3_600).contains(&duel.min_seconds)
                    || duel.win_coins > 10_000
            })
        {
            return Err(invalid(
                "card duel needs distinct game types 1 to 3, 1 to 3600 seconds and at most 10000 coins",
            ));
        }
        if self.launch.as_ref().and_then(|l| l.packs.as_ref()).is_some_and(|packs| !packs.valid()) {
            return Err(invalid("card packs need a source, full pools and 40-card starter decks"));
        }
        let (prizes, sourced) = self.launch.as_ref().map_or((&[][..], true), |launch| {
            (&launch.prizes[..], launch.prizes.is_empty() || !launch.prizes_source.is_empty())
        });
        let is_row = |id| self.rows.iter().chain(&self.hidden_rows).any(|row| row.minigame_id == id);
        if !sourced
            || prizes.iter().any(|p| {
                !(1..=3).contains(&p.cards.len())
                    || p.cards.iter().any(|c| *c <= 0)
                    || !is_row(p.minigame_id)
            })
            || prizes.iter().map(|p| p.minigame_id).collect::<BTreeSet<_>>().len() != prizes.len()
        {
            return Err(invalid(
                "card prizes need a source, 1 to 3 real cards, a real row and one prize per deck",
            ));
        }
        // (R) bounds: only the client's root group 1 until a second group is decoded (`10371960`
        // joins group 1 itself), a known role, a small type.
        if self
            .launch
            .as_ref()
            .and_then(|l| l.lobby.as_ref())
            .is_some_and(|lobby| {
                lobby.group_id != 1
                    || !(1..=5).contains(&lobby.role)
                    || !(0..=255).contains(&lobby.lobby_type)
            })
        {
            return Err(invalid(
                "card lobby needs the root group 1, a role 1 to 5 and a type 0 to 255",
            ));
        }
        // (R) bounds: distinct positive archetypes (at most 100, so one SendInstances stays
        // small) and 1..=16 copies each. The 16 is a server choice for a small starter, not a
        // client limit: the client's default collection lists 16 instances of a card (two have
        // 100).
        let collection = self
            .launch
            .iter()
            .flat_map(|l| &l.lobby)
            .flat_map(|l| &l.collection);
        let mut seen = std::collections::BTreeSet::new();
        for card in collection {
            if card.archetype_id < 1
                || !(1..=16).contains(&card.copies)
                || !seen.insert(card.archetype_id)
            {
                return Err(invalid(
                    "card starter collection needs distinct positive archetypes, 1 to 16 copies each",
                ));
            }
        }
        if seen.len() > 100 {
            return Err(invalid(
                "card starter collection has at most 100 archetypes",
            ));
        }
        Ok(())
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// T765: the launch settings, when Trading Card Game rows may start.
    pub fn launch(&self) -> Option<&Launch> {
        self.launch.as_ref()
    }

    /// The packs each NPC sells, as (item, name line, coins) rows.
    pub fn pack_sales(&self) -> Vec<(u64, Vec<(i32, u32, u32)>)> {
        let mut by_npc = BTreeMap::<u64, Vec<(i32, u32, u32)>>::new();
        for sale in self.launch.iter().filter_map(|l| l.packs.as_ref()).flat_map(|p| &p.sales) {
            by_npc.entry(sale.npc_id).or_default().push((sale.item_id, sale.name_id, sale.coins));
        }
        by_npc.into_iter().collect()
    }

    /// The cards deck `minigame_id` pays the first time it is beaten.
    pub fn deck_prize(&self, minigame_id: u32) -> Option<&[i32]> {
        let launch = self.launch.as_ref()?;
        Some(&launch.prizes.iter().find(|prize| prize.minigame_id == minigame_id)?.cards)
    }

    /// The row behind activity `id`.
    pub fn row(&self, id: u32) -> Option<&Row> {
        self.rows.iter().chain(&self.hidden_rows).find(|row| row.id == id)
    }

    /// T545: whether `minigame_id` is a client TYPE 16 row, the only kind a card duel
    /// result may name. A pre-T545 tree without the list refuses every id.
    pub fn is_duel_row(&self, minigame_id: u32) -> bool {
        self.type16_minigame_ids.binary_search(&minigame_id).is_ok()
    }

    /// The catalogue rows (ids, names, image, difficulty); the rest of each wire row is read
    /// back from `rows()` by `Catalogue::list_packet`.
    /// The hidden decks an NPC offers: they credit their MiniGameData id like a Games-window row.
    pub fn offered_hidden_rows(&self) -> impl Iterator<Item = &Row> {
        let offered: std::collections::BTreeSet<u32> =
            self.npc_decks.iter().flat_map(|npc| npc.activities.iter().copied()).collect();
        self.hidden_rows.iter().filter(move |row| offered.contains(&row.id))
    }

    /// (R) NPC -> its decks as (activity id, deck name line, MiniGameData id), for the NPC menu rows.
    pub fn npc_decks(&self) -> Vec<(u64, Vec<(u32, u32, u32)>)> {
        self.npc_decks
            .iter()
            .map(|npc| {
                let rows = npc.activities.iter().filter_map(|id| self.row(*id));
                (npc.npc_id, rows.map(|row| (row.id, row.name_id, row.minigame_id)).collect())
            })
            .collect()
    }

    pub fn portal_rows(&self) -> Vec<Activity> {
        self.rows
            .iter()
            .map(|row| Activity {
                id: row.id,
                name_id: row.name_id,
                description_id: row.description_id,
                icon_id: row.image_set_id,
                difficulty: row.difficulty as u32,
                required_level: 0,
                kind: Kind::CardGame {
                    minigame_id: row.minigame_id,
                },
            })
            .collect()
    }
}

fn is_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= MAX_IMAGE_NAME && name.is_ascii()
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minigame::{Activity, ActivityFinish};

    struct Loss;

    impl Activity for Loss {
        fn score(&self) -> u32 {
            0
        }

        fn finish(&self) -> Option<ActivityFinish> {
            Some(ActivityFinish {
                won: false,
                stars: 0,
                elapsed_seconds: 1,
            })
        }
    }

    fn client(sub: u8, instance: i32) -> Vec<u8> {
        let mut packet = vec![39, 0, sub];
        for value in [instance, -1, -1] {
            packet.extend(value.to_le_bytes());
        }
        packet
    }

    /// A collection refuses a pack that would pass either bound.
    #[test]
    fn a_full_collection_has_no_room_for_a_pack() {
        let mut earned = EarnedCards::default();
        earned.add(&vec![5; MAX_EARNED_COPIES as usize - 1]);
        assert!(earned.has_room_for(&[5]));
        assert!(!earned.has_room_for(&[5, 5]), "one card's bound");
        assert!(earned.has_room_for(&[6, 6]));
        for card in 100..140 {
            earned.add(&vec![card; MAX_EARNED_COPIES as usize]);
        }
        assert!(!earned.has_room_for(&[6]), "the whole collection's bound");
    }

    /// Every shipped pack opens to the right number of cards, and a seed varies a booster.
    #[test]
    fn a_pack_opens_into_its_cards() {
        let cards = Content::load(&crate::content::bundled_content().join(CONTENT_FILE)).unwrap();
        let packs = cards.launch().unwrap().packs.as_ref().unwrap();
        for pack in &packs.boosters {
            assert_eq!(packs.open(pack.item_id, 88_172_645_463_325_252).unwrap().len(), 10);
        }
        for deck in &packs.starter_decks {
            assert_eq!(packs.open(deck.item_id, 1).unwrap().len(), 40);
        }
        let item = packs.boosters[0].item_id;
        assert_ne!(packs.open(item, 7), packs.open(item, 991));
        assert_eq!(packs.open(1, 7), None, "not a pack");
    }

    /// A deck pays once, and earned copies stack on the starter's.
    #[test]
    fn a_deck_pays_its_prize_once_and_its_cards_join_the_collection() {
        let mut earned = EarnedCards::default();
        assert!(earned.pay_prize(504, &[7, 8, 8]));
        assert!(!earned.pay_prize(504, &[7, 8, 8]), "once per deck");
        let starter = [StarterCard { archetype_id: 8, copies: 3 }];
        assert_eq!(
            collection_with(&starter, &earned.rows()),
            [
                StarterCard { archetype_id: 8, copies: 5 },
                StarterCard { archetype_id: 7, copies: 1 }
            ]
        );
    }

    /// Every deck row with a client scenario has a prize (the tutorial has no opponent).
    #[test]
    fn every_scenario_deck_has_a_prize() {
        let cards = Content::load(&crate::content::bundled_content().join(CONTENT_FILE)).unwrap();
        let rows = cards.rows.iter().chain(&cards.hidden_rows);
        for row in rows.filter(|row| scenario_of(row.minigame_id as i32).is_some() && row.minigame_id != 537) {
            let prize = cards.deck_prize(row.minigame_id).expect("a prize");
            assert_eq!(prize.len(), 3, "row {}", row.minigame_id);
        }
    }

    /// The client has 1..=121, each one row; the owners' intros pin the blocks.
    #[test]
    fn every_client_scenario_is_exactly_one_row() {
        let mut numbers: Vec<i32> = SCENARIO_BLOCKS
            .iter()
            .flat_map(|&(first, last, scenario)| {
                (0..=last - first).map(move |offset| scenario + offset)
            })
            .collect();
        numbers.sort_unstable();
        assert_eq!(numbers, (1..=121).collect::<Vec<_>>());
        let named = [(505, 6), (695, 55), (699, 59), (398, 77), (410, 89), (1090, 113), (1098, 121)];
        for (row, scenario) in named {
            assert_eq!(scenario_of(row), Some(scenario), "row {row}");
        }
        assert_eq!(scenario_of(537), Some(38), "the client's own tutorial");
        for lobby in [499, 727, 1036, 9997] {
            assert_eq!(scenario_of(lobby), None, "row {lobby}");
        }
    }

    fn launches() -> Launches {
        let launch: Launch = serde_json::from_value(serde_json::json!({
            "ticket_ttl_ms": 120_000,
            "source": [{"kind": "reconstructed", "evidence": "F800 test"}]
        }))
        .unwrap();
        Launches::new(&launch).unwrap()
    }

    fn dueling() -> Launches {
        let launch: Launch = serde_json::from_value(serde_json::json!({
            "ticket_ttl_ms": 120_000,
            "duel": {"game_types": [2, 3], "min_seconds": 60, "win_coins": 25},
            "source": [{"kind": "reconstructed", "evidence": "L935 test"}]
        }))
        .unwrap();
        Launches::new(&launch).unwrap()
    }

    fn record(phase: i32, game_type: i32, result: i32, started: i32) -> SinglePlayerRecord {
        SinglePlayerRecord {
            account: 1,
            phase,
            game_type,
            result,
            started,
        }
    }

    /// The instance the row's 39/16 offer names.
    fn enter(launches: &mut Launches, row: u32) -> i32 {
        let cards = Content::load(&crate::content::bundled_content().join(CONTENT_FILE)).unwrap();
        let offer = launches.begin(cards.row(row).unwrap()).unwrap();
        i32::from_le_bytes(offer.replies[0][3..7].try_into().unwrap())
    }

    /// The start screen reads its title and description from the offer; the row in play picks
    /// the client scenario, and the lobby row has none.
    #[test]
    fn a_card_offer_names_its_row_and_scenario() {
        let cards = Content::load(&crate::content::bundled_content().join(CONTENT_FILE)).unwrap();
        let mut launches = launches();
        let row = cards.row(491).unwrap();
        let offer = launches.begin(row).unwrap();
        // The 39/16 body after its 15-byte header: name, icon, description, difficulty.
        let field = |at: usize| i32::from_le_bytes(offer.replies[0][at..at + 4].try_into().unwrap());
        assert_eq!(field(11), row.minigame_id as i32, "the header's last field");
        assert_eq!(field(15), row.name_id as i32);
        assert_eq!(field(23), row.description_id as i32);
        assert_eq!(field(27), row.difficulty);
        assert_eq!(launches.scenario(), Some(5), "Poe's Trick Deck, row 504");
        let mut lobby = self::launches();
        lobby.begin(cards.row(7).unwrap()).unwrap();
        assert_eq!(lobby.scenario(), None);
    }

    const START: i32 = SinglePlayerRecord::STARTED;
    const END: i32 = SinglePlayerRecord::ENDED;
    const WIN: i32 = SinglePlayerRecord::WON;

    /// L935: a Skirmish or Scenario win pays the launched row once, and only after the pacing floor
    /// measured on the server's clock between the start record and the end record.
    #[test]
    fn a_win_after_the_floor_credits_the_launched_row_once() {
        let mut launches = dueling();
        enter(&mut launches, 490);
        assert_eq!(launches.record(&record(START, 3, 0, 1_000), 5_000), None);
        let win = launches
            .record(&record(END, 3, WIN, 1_000), 65_000)
            .unwrap();
        assert_eq!(
            win,
            Win {
                minigame_id: 500,
                started: 1_000,
                elapsed_seconds: 60,
                coins: 25
            }
        );
        // The same end record again, and a replay of the same start, pay nothing.
        assert_eq!(launches.record(&record(END, 3, WIN, 1_000), 130_000), None);
        assert_eq!(launches.record(&record(START, 3, 0, 1_000), 131_000), None);
        assert_eq!(launches.record(&record(END, 3, WIN, 1_000), 200_000), None);
        // An older start is ignored; a newer game (the scenario, type 2) pays again.
        assert_eq!(launches.record(&record(START, 2, 0, 999), 201_000), None);
        assert_eq!(launches.record(&record(END, 2, WIN, 999), 300_000), None);
        assert_eq!(launches.record(&record(START, 2, 0, 2_000), 301_000), None);
        assert!(
            launches
                .record(&record(END, 2, WIN, 2_000), 362_000)
                .is_some()
        );
    }

    #[test]
    fn a_result_that_misses_a_bound_pays_nothing() {
        let mut launches = dueling();
        // No run yet: nothing is tracked.
        assert_eq!(launches.record(&record(START, 3, 0, 1_000), 0), None);
        enter(&mut launches, 490);
        assert_eq!(
            launches.record(&record(END, 3, WIN, 1_000), 90_000),
            None,
            "no start record"
        );

        let mut play = |kind: i32, result: i32, started: i32, seconds: u64| {
            let at = 1_000_000 * started.unsigned_abs() as u64;
            launches.record(&record(START, kind, 0, started), at);
            launches.record(&record(END, kind, result, started), at + seconds * 1_000)
        };
        assert_eq!(play(3, WIN, 1, 59), None, "under the floor");
        assert_eq!(play(3, 2, 2, 600), None, "a loss");
        assert_eq!(play(3, 0, 3, 600), None, "an unfinished result");
        assert_eq!(play(1, WIN, 4, 600), None, "the scripted Tutorial");
        assert_eq!(play(9, WIN, 5, 600), None, "a type the DLL does not have");
        assert!(
            play(3, WIN, 6, 60).is_some(),
            "the control: a Skirmish win at the floor"
        );

        // An end that names another game neither pays nor consumes the pending game.
        launches.record(&record(START, 3, 0, 7), 20_000_000);
        assert_eq!(launches.record(&record(END, 3, WIN, 8), 90_000_000), None);
        assert!(
            launches
                .record(&record(END, 3, WIN, 7), 90_000_001)
                .is_some()
        );
    }

    #[test]
    fn a_closed_run_or_missing_duel_settings_pay_nothing() {
        let mut launches = dueling();
        let instance = enter(&mut launches, 491);
        launches.record(&record(START, 3, 0, 1_000), 0);
        launches.route(&client(6, instance), 1, 1_000).unwrap();
        assert_eq!(
            launches.record(&record(END, 3, WIN, 1_000), 100_000),
            None,
            "the card window was left"
        );

        // A launch block with no `duel` settings opens the window but pays no game.
        let mut plain = self::launches();
        enter(&mut plain, 491);
        plain.record(&record(START, 3, 0, 1_000), 0);
        assert_eq!(plain.record(&record(END, 3, WIN, 1_000), 100_000), None);
    }

    /// T782 review: a card run that reached Won/Lost owns the client's results Close (26/42) and
    /// retires on it; while it is offered or playing that same Close is still not the run's.
    #[test]
    fn a_terminal_card_run_owns_the_results_close_once() {
        let shipped = crate::content::bundled_content().join(CONTENT_FILE);
        let cards = Content::load(&shipped).unwrap();
        let row = cards.row(490).expect("the Trading Card Game row");
        let mut launches = launches();
        let offer = launches.begin(row).unwrap();
        let instance = i32::from_le_bytes(offer.replies[0][3..7].try_into().unwrap());
        assert!(
            !launches.owns(&RESULTS_CLOSE),
            "offered: the start-screen close is not the run's (T782)"
        );
        launches.route(&client(37, instance), 1, 1_000).unwrap();
        launches.route(&client(5, instance), 1, 1_100).unwrap();
        assert!(
            !launches.owns(&RESULTS_CLOSE),
            "playing: there is no result to close yet"
        );

        launches.runs.started(instance);
        launches.runs.finish(instance, &Loss, None).unwrap();
        assert!(launches.owns(&RESULTS_CLOSE));
        let close = launches.route(&RESULTS_CLOSE, 1, 1_200).unwrap();
        assert!(close.accepted && close.changed && close.replies.is_empty());
        assert!(!launches.is_active(), "the next Play is free again");
        assert!(
            !launches.owns(&RESULTS_CLOSE),
            "a repeated results close is not the run's"
        );
        let repeat = launches.route(&RESULTS_CLOSE, 1, 1_300).unwrap();
        assert!(!repeat.accepted && !repeat.changed);
    }
}
