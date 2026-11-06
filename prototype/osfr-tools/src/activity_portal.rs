//! W150: the activity portal list (167/1/1), its refresh (167/1/4) and the Play choice (167/1/2).
//!
//! # Recovered original facts (addresses in `FreeRealms.exe`)
//!
//! * Family 167 header, 4 bytes: `u16 167`, `u8 service`, `u8 sub-op` (`BaseActivityServicePacket`
//!   `008a20c0` stores 167 and the service, `BaseActivityPacket` `008a21f0` stores the sub-op;
//!   the writer `00913f70`/`0090b490` emits `u16`, `u8`, `u8`; the reader `00be25e0`/`00be2e80`
//!   reads them back). The activity packets are service 1; service 2 is the scheduled list.
//! * 167/1/4 refresh request (`008a2330`): the 4 header bytes and nothing else. The client sends it
//!   through both tunnels (`0092a2d0`, `0092a3e0`) when a job level changes (`009392c0`, update
//!   subtype 14), when membership activates (`009471a0`, message 100) and when it leaves a
//!   minigame (`009bd240`). It is NOT sent at login, so the list also has to be pushed.
//! * 167/1/2 Play (`ActivityJoinRequest_SendByType` `00be7d80`, ctor `00bdefa0`): the sender looks
//!   the id up in the list it was given first (an unlisted id never leaves the client), then
//!   writes header + `u32 activity id` + `u32` taken from `[client+0x74]` (`00be7010`); for entry
//!   type 1 it goes through the world tunnel (`00be77b0`), for type 2 through the client tunnel
//!   (`00be78c0`). 12 bytes. The last dword is the client's server-clock offset (`008bb4f0` adds it
//!   to the wall clock): never trusted, never read here.
//! * 167/1/1 list (`00be79d0` -> `00be75c0` -> `00be6fa0` -> `00be63e0`): header, `u32 entry type`
//!   (the client first removes every listed activity of that entry type, then adds the new rows),
//!   `i32 count`, then per row, little endian: `u32` Id, AppSystemId, Category, ImageSetId,
//!   ActivityPositionId, DisplayNameId, DisplayDescriptionId, NameId, DescriptionId, EntryType;
//!   `u8` CanPlayerJoin; `u32` RecommendedSetId; `u32` featured-window count (+ 33 bytes each,
//!   always 0 here); `u32` TutorialActivityId; `u8` MembersOnly; `i32 len` + bytes
//!   DetailImageFilename; `i32 len` + bytes ThumbnailImageFilename; `u32` Difficulty, ContentId
//!   (`+0xc`), unknown `+0x68`, MysteryChestIcon. The reader fails (and the client drops the list)
//!   if the packet has trailing or missing bytes. Field-to-column mapping is from
//!   `ClientActivityDatasource::GetCell` (`00cfc180`, jump table `00cfc434`) and the Lua
//!   `Activities_col`.
//! * L095, the live server's own lists (the community captures `packets_2.pcapng`, `p2`, `p3`, `p12`
//!   and the racing and demo derby captures, 2014-03-24..30, decoded with our transport framing):
//!   entry type 1 carried 238 world activities (battles, the kart garage, Pirate's Plunder, events,
//!   meeting places), entry type 2 the 179 minigames. Every row parses to the last byte with 33-byte
//!   featured windows. ContentId is the client `MiniGameData.txt` row of every entry-type-2 row (all
//!   179 names agree) and the battle's own id on battle rows. RecommendedSetId is a client
//!   `RequirementSets.txt` row (mining 15-18 = Miner rank 5/9/13/17, cooking 11-14 Chef, forging
//!   19-22 Blacksmith, battles 2-6 combat-rank bands); CanPlayerJoin is set apart from it (every
//!   battle was joinable whatever its band). Rows outside every Games window tile carry category 99
//!   and `placeholder_*.dds` art, the field battles among them. All 417 rows name art the client
//!   ships.
//! * `Ui.RequestToJoinActivity(id, 0)` (portal Play button) is the only Lua entry to `00be7d80`.
//! * Category decides where a row shows (T399). After every list the client files each row under
//!   the datasource `"Activities.Category%d"` of its Category dword (`00cea720`, format `00cfc0b0`
//!   at `00cea896`). The Games window lists the `ActivityCategories.txt` rows (loader `00be58f0`)
//!   and opens the finder on `Activities.Category<row ID>`, adding every row of it (Lua
//!   `BrowserV2:PopulateGames`, `MinigameDetail:Populate`). Row 10 is "Battles" (name 31018), row
//!   18 is "Micro-Games" (31026), row 22 is "Soccer" (31030).
//! * Card-game rows (T504, `card_game.rs`) take every wire value from their data row: category 11
//!   "Trading Card Game" and 26 "Treasure War" (`ActivityCategories.txt`), app system id,
//!   position, display and tutorial ids, art and difficulty.
//!
//! # Reconstructed policy (R)
//!
//! * L095: with `activity_catalogue.json` (the live Games window rows: all 16 tiles, every wire value
//!   but CanPlayerJoin) the client sees the live tree with its own art, in the live entry-type lists.
//!   A live row whose game this server runs is joined to that service row: a battle or soccer row by
//!   tile and name, chess by its MiniGameData row, a card row by id, the wheel by its activity id.
//!   Play on the live id starts the service row. Every other live row is listed with CanPlayerJoin 0
//!   and its Play is refused ("not open yet"). A field battle the live tiles lack is listed the way
//!   the live server listed its own: category 99 and placeholder art, so no tile shows it. Other
//!   unjoined service rows are not listed. Without the file the bullets below hold unchanged.
//! * L100: every game a Type8 engine offers (`mining.json`, `harvesting.json` `games`) is a
//!   gather row joined by its MiniGameData row. A recipe row is joinable at the recipe's rank in
//!   its job, active or not. The live lists back the rule's shape, not its numbers: their mining
//!   rows asked Miner rank 5/9/13/17 (`RequirementSets.txt` 15-18) and the capturing player could
//!   join exactly the rows under its ranks, while iron, silver and gold smelting carried no set
//!   and were closed too, so the live server kept its own recipe ranks.
//! * L102: each `fishing_spots.json` hole is a fishing row joined by its MiniGameData row (the six
//!   live rows 560-565 name 596-601); Play starts the round its guide offers, without the guide.
//! * Only activities backed by a running battle or a configured native soccer match are published:
//!   one row per `field_battles.json` battle (category 10, Battles) and one per `soccer.json` game
//!   (category 22, art `Soccer_Briarwood.dds`; both values are the client `ActivityCategories.txt`
//!   row 22), entry type 1, plus chess and the card-game rows. Every other activity (races,
//!   mining ...) is simply not listed, so the client cannot ask for it.
//! * `CanPlayerJoin` is per player and mirrors Play's own gates, so the client greys out what
//!   Play would refuse (B4; the live server's rows were greyed, not refused): battle rows require
//!   a combat job at the configured level; chess is open, and soccer uses its own configured
//!   level floor without a job gate. A refused Play says why in chat, or for a gather or recipe
//!   row's missing job with the client's own 147/2 alert. The owning activity, never the portal
//!   request, awards.
//! * For battles, chess and soccer the preferred/featured/tutorial/AppSystem/chest fields and the two
//!   unknown dwords are 0.
//!   Image names come from each row's client category (`bandit_hideout_detail.dds` for battles and
//!   `Soccer_Briarwood.dds` for soccer).
//! * (R) A card-game row is listed greyed while the card game is closed; a forged Play answers
//!   with the client's own "currently unavailable" line (Global.Text 140987). Rows publish entry
//!   type 1 like every other row.

use crate::{
    card_game, chess_service::ChessService, harvest_nodes::Gather, recipes::CraftingJob,
    soccer_entry,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

const FAMILY: u16 = 167;
/// The 147/2 ActivityJoinErrorPacket (`00be7360`): `u16 147`, `u8 2`, `u32` Global.Text id, shown
/// by `ActivityPortal:OnJoinError` as a yellow alert.
const JOIN_ERROR_FAMILY: u16 = 147;
const SUB_JOIN_ERROR: u8 = 2;
/// Client `CodeStringMappings.txt` JoinFailedYouAreMissingProfile: "You can't play this minigame
/// because you don't have the required job."
pub const MISSING_JOB_TEXT: u32 = 116_214;
const SERVICE_ACTIVITY: u8 = 1;
const SUB_LIST: u8 = 1;
const SUB_PLAY: u8 = 2;
const SUB_REFRESH: u8 = 4;
const ENTRY_WORLD_TUNNEL: u32 = 1;
/// L095: the live minigame rows' entry type; the client sends their Play through the client tunnel.
const ENTRY_CLIENT_TUNNEL: u32 = 2;
/// L095: the category the live server gave rows listed outside every Games window tile.
const HIDDEN_CATEGORY: u32 = 99;
const HIDDEN_DETAIL: &str = "placeholder_detail.dds";
const HIDDEN_THUMBNAIL: &str = "placeholder_thumb.dds";
/// L095: the live Games window rows, beside the other content files.
pub const CATALOGUE_FILE: &str = "activity_catalogue.json";
/// The live lists held 417 rows.
const MAX_LIVE_ROWS: usize = 512;
const MAX_IMAGE_NAME: usize = 64;
/// "Battles", row 10 of the client `Resources/ActivityCategories.txt` (T399; was 18, Micro-Games).
const COMBAT_CATEGORY: u32 = 10;
/// Recovered soccer category row 22 in the client `Resources/ActivityCategories.txt`.
pub const SOCCER_CATEGORY: u32 = 22;
/// Recovered category-22 art name in the same client table.
pub const SOCCER_ART: &str = "Soccer_Briarwood.dds";
/// Category-10 art in the same client table.
const ART: &str = "bandit_hideout_detail.dds";
/// (R) The type-6 row in `MiniGameTypeData.txt` carries CATEGORY_ID 0; the
/// portal categories table has no board-game row, so chess rows use it as-is.
const CHESS_CATEGORY: u32 = 0;
const CHESS_DETAIL: &str = "chess_detail.dds";
const CHESS_THUMBNAIL: &str = "chess_thumb.dds";
/// L099: the service rows include one per runnable recipe (31 today). L962: raised from 128 when
/// Tower Defense's 36 games pushed a full content set past it; the live catalogue has 211 rows.
const MAX_ACTIVITIES: usize = 256;
const MAX_FILE: u64 = 1 << 20;
const PLAY_LEN: usize = 12;

/// What a published row starts. Battles use the encounter engine; chess rows
/// name the client minigame row the chess table service understands; soccer
/// rows begin the session-local native match service; card-game rows start
/// nothing yet (no card service): Play is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Battle,
    Chess {
        row_id: u32,
    },
    Soccer,
    CardGame {
        minigame_id: u32,
    },
    /// L097: the shipped kart race and the default demolition derby; the row's id is their
    /// MiniGameData row and only a live Games window row lists them.
    Kart,
    Derby,
    /// The Kart & Demo Derby Garage: row 1079 joins by its own id, as ContentId 121 is a dead row.
    Garage,
    /// L098: a recipe's minigame (cooking, forging, smelting); the row's id is its MiniGameData
    /// row and Play starts `recipe_id` through the recipe book, anywhere (R). L100: `job` is the
    /// recipe's job, whose rank the row's `required_level` asks for.
    Recipe {
        recipe_id: u32,
        job: CraftingJob,
    },
    /// L100: a mining or harvesting game a Type8 engine offers; the row's id is its MiniGameData
    /// row and Play starts that game's board, anywhere (R).
    Gather {
        gather: Gather,
    },
    /// L102: a fishing hole; the row's id is its MiniGameData row and Play starts the round its
    /// `guide` offers, anywhere (R).
    Fishing {
        guide: u64,
    },
    /// L104: the checkers table against the server; the row's id is MiniGameData 21.
    Checkers,
    /// L961 (L123's engine): a Feeling the Beat! game against the server; the row's id is its
    /// MiniGameData row (201-204).
    Music,
    /// L962: a Tower Defense game against the server; the row's id is its MiniGameData row.
    TowerDefense,
    /// L963: Pirate's Plunder; the row's id is MiniGameData 54, the content of the world-tunnel
    /// Games row 1059.
    Pirates,
}

/// One published activity: a battle of the frozen package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Activity {
    /// Wire id; equals the battle id.
    pub id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
    pub required_level: u32,
    pub kind: Kind,
}

/// One published chess row. Ids, names, icon and difficulty come from the
/// client `MiniGameData` type-6 rows; only the portal id is server policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessActivity {
    pub row_id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
}

impl From<ChessActivity> for Activity {
    fn from(row: ChessActivity) -> Self {
        Self {
            id: row.row_id,
            name_id: row.name_id,
            description_id: row.description_id,
            icon_id: row.icon_id,
            difficulty: row.difficulty,
            required_level: 0,
            kind: Kind::Chess { row_id: row.row_id },
        }
    }
}

#[derive(Deserialize)]
struct BattleFile {
    battles: Vec<Battle>,
}

#[derive(Deserialize)]
struct Battle {
    id: u32,
    title_id: u32,
    description_id: u32,
    difficulty: u32,
    #[serde(default)]
    required_combat_level: u32,
    #[serde(default)]
    icon_id: i64,
}

/// L095: one row of the live server's Games window lists: every wire value but CanPlayerJoin.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LiveRow {
    pub entry_type: u32,
    pub id: u32,
    pub app_system_id: u32,
    pub category: u32,
    pub image_set_id: u32,
    pub position: u32,
    pub display_name_id: u32,
    pub display_description_id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub recommended_set_id: u32,
    pub tutorial_activity_id: u32,
    pub difficulty: u32,
    pub content_id: u32,
    pub detail_image: String,
    pub thumbnail_image: String,
    pub members_only: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveFile {
    schema_version: u32,
    #[allow(dead_code)]
    source: String,
    rows: Vec<LiveRow>,
}

/// What starts a live row: a service row (an index into `Catalogue::rows`), the daily wheel (its
/// own handler takes the Play first), or nothing here yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Runner {
    Service(usize),
    Wheel,
    Closed,
}

/// L095: the live rows in `directory`, or none when the file is absent.
/// What a game's own Games window row says on its offer card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfferText {
    pub name_id: i32,
    pub description_id: i32,
    pub difficulty: i32,
}

/// Each game row's offer text by game id, from `directory`'s catalogue or the bundled one.
pub fn offer_texts(directory: &Path) -> io::Result<BTreeMap<i32, OfferText>> {
    let mut rows = load_live(directory)?;
    if rows.is_empty() {
        rows = load_live(&crate::content::bundled_content())?;
    }
    // Wire values: some rows hold -1 (0xFFFFFFFF) as their difficulty.
    let texts = (rows.iter().filter(|row| row.entry_type == 2)).map(|row| {
        let text = OfferText {
            name_id: row.name_id.cast_signed(),
            description_id: row.description_id.cast_signed(),
            difficulty: row.difficulty.cast_signed(),
        };
        (row.content_id.cast_signed(), text)
    });
    Ok(texts.collect())
}

pub fn load_live(directory: &Path) -> io::Result<Vec<LiveRow>> {
    let path = directory.join(CATALOGUE_FILE);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let file: LiveFile =
        serde_json::from_str(&read_bounded(&path)?).map_err(|e| invalid(&e.to_string()))?;
    if file.schema_version != 1 {
        return Err(invalid("unsupported activity catalogue"));
    }
    validate_live(&file.rows)?;
    Ok(file.rows)
}

/// What the server knows about the asking player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewer<'a> {
    /// The active job is a combat job.
    pub combat_job: bool,
    /// Level of the active job.
    pub level: u32,
    /// L100: rank of each crafting job, active or not, in `CraftingJob` order (Chef, Blacksmith,
    /// Miner).
    pub crafts: [u32; 3],
    /// L106: the Postman rank, for the Mail Sorting rows' floors.
    pub postman: u32,
    /// The jobs the player has (unlocked profile ids): a gather or recipe row needs its job.
    pub jobs: &'a BTreeSet<i32>,
    /// The recipes the player has learned: a recipe row needs its recipe.
    pub learned: &'a BTreeSet<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Not a published activity (forged or stale id).
    Unknown,
    /// Another activity, fight, knock-out or movie owns the player.
    Busy,
    NotCombatJob,
    LevelTooLow(u32),
    /// The card content has no launch block (T504; T765 opens it).
    CardGameClosed,
    TreasureWarClosed,
    /// L095: a live row this server does not run yet (its Play is greyed out; this one was forged).
    NotOpen,
}

impl Refusal {
    pub fn message(self) -> String {
        match self {
            Self::Unknown => "That activity is not available.".into(),
            Self::Busy => "Finish what you are doing before starting an activity.".into(),
            Self::NotCombatJob => "Switch to a combat job before starting a battle.".into(),
            Self::LevelTooLow(level) => format!("This activity needs level {level}."),
            // Client Global.Text 140987.
            Self::CardGameClosed => "The Free Realms Trading Card Game is currently unavailable. Please try again in a few minutes.".into(),
            // (R) The client has no Treasure War closed line.
            Self::TreasureWarClosed => "Treasure War is not open yet.".into(),
            Self::NotOpen => "That activity is not open yet.".into(),
        }
    }
}

/// 147/2 with a client Global.Text id (FINDINGS `join-error-147-20261102`; no capture has one).
pub fn join_error(text_id: u32) -> Vec<u8> {
    let mut out = JOIN_ERROR_FAMILY.to_le_bytes().to_vec();
    out.push(SUB_JOIN_ERROR);
    out.extend(text_id.to_le_bytes());
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    Refresh,
    Play { activity: u32 },
}

/// The published activities. Empty by default, which publishes an empty list.
///
/// Soccer, chess and card-game rows live in the same list so the client can ask for them with the
/// portal Play request. The wire row shape is identical; `Kind` selects the
/// admission and category rules.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalogue {
    rows: Vec<Activity>,
    /// The card-game rows' own wire values (`Kind::CardGame` rows read them by id).
    cards: Option<Arc<card_game::Content>>,
    /// L095: the live Games window rows and what starts each one.
    live: Vec<(LiveRow, Runner)>,
}

impl Catalogue {
    /// Build from rows; ids must be unique and nonzero (they are the client's hash keys).
    pub fn new(rows: Vec<Activity>) -> io::Result<Self> {
        validate_rows(&rows)?;
        Ok(Self {
            rows,
            ..Self::default()
        })
    }

    /// L095: list the live Games window rows, each joined to the service row that runs it (see
    /// `runs`); `wheel` is the daily wheel's activity id while the wheel is on. L097: `games` are
    /// the kart/derby rows, added only with live rows (nothing else lists them). Call after the
    /// last `append`. A service row joins at most one live row.
    pub fn with_live(
        mut self,
        live: Vec<LiveRow>,
        wheel: Option<u32>,
        games: Vec<Activity>,
    ) -> io::Result<Self> {
        validate_live(&live)?;
        if !live.is_empty() {
            // A game whose MiniGameData row is already a service id stays unlisted (the tests
            // prove every live row that should run is joined).
            let games = (games.into_iter())
                .filter(|game| !self.rows.iter().any(|row| row.id == game.id))
                .collect::<Vec<_>>();
            self = self.append(games)?;
        }
        let mut taken = vec![false; self.rows.len()];
        self.live = live
            .into_iter()
            .map(|row| {
                let joined = (wheel != Some(row.id))
                    .then(|| (0..self.rows.len()).find(|&i| !taken[i] && runs(&row, &self.rows[i])))
                    .flatten();
                let runner = match joined {
                    Some(i) => {
                        taken[i] = true;
                        Runner::Service(i)
                    }
                    None if wheel == Some(row.id) => Runner::Wheel,
                    None => Runner::Closed,
                };
                (row, runner)
            })
            .collect();
        if self
            .own_rows()
            .any(|(_, a)| self.live.iter().any(|(row, _)| row.id == a.id))
        {
            return Err(invalid("an activity id repeats a live activity id"));
        }
        Ok(self)
    }

    /// The live rows, so a rebuild can publish them again.
    pub fn live(&self) -> Vec<LiveRow> {
        self.live.iter().map(|(row, _)| row.clone()).collect()
    }

    /// Service rows listed by their own id, with their index: all of them without live rows; with
    /// live rows only the field battles no live row runs (listed hidden, as the live server listed
    /// its field battles).
    fn own_rows(&self) -> impl Iterator<Item = (usize, &Activity)> {
        self.rows.iter().enumerate().filter(|(i, a)| {
            self.live.is_empty()
                || (a.kind == Kind::Battle
                    && !self.live.iter().any(|(_, r)| *r == Runner::Service(*i)))
        })
    }

    /// The service row a client id starts; `Ok(None)` for a listed live row nothing here runs.
    fn lookup(&self, id: u32) -> Result<Option<&Activity>, Refusal> {
        if let Some((_, runner)) = self.live.iter().find(|(row, _)| row.id == id) {
            return Ok(match *runner {
                Runner::Service(i) => Some(&self.rows[i]),
                Runner::Wheel | Runner::Closed => None,
            });
        }
        self.own_rows()
            .find(|(_, a)| a.id == id)
            .map(|(_, a)| Some(a))
            .ok_or(Refusal::Unknown)
    }

    /// The one place the catalogue is assembled, at startup and on `realm reload battles`:
    /// battles, then chess, soccer and card-game rows. Every append is validated together
    /// (ids unique and nonzero, at most `MAX_ACTIVITIES`), so both callers publish the same list.
    pub fn assemble(
        battles: Option<&Path>,
        chess: Option<&ChessService>,
        soccer: Option<&Arc<soccer_entry::Content>>,
        cards: Option<&Arc<card_game::Content>>,
    ) -> io::Result<Self> {
        let base = battles.map_or_else(|| Ok(Self::default()), Self::load)?;
        let chess = chess.map(ChessService::activities).unwrap_or_default();
        let soccer = soccer.map(|content| content.portal_rows());
        let mut catalogue = base
            .append(chess.into_iter().map(Activity::from))?
            .append(soccer.unwrap_or_default())?;
        if let Some(cards) = cards {
            catalogue = catalogue.append(cards.portal_rows())?;
            catalogue.cards = Some(cards.clone());
        }
        Ok(catalogue)
    }

    /// The card-game content behind the card rows, so a rebuild can publish them again.
    pub fn cards(&self) -> Option<&Arc<card_game::Content>> {
        self.cards.as_ref()
    }

    /// Rows after the current ones. Ids share one namespace, so a collision is refused.
    pub fn append(mut self, rows: impl IntoIterator<Item = Activity>) -> io::Result<Self> {
        self.rows.extend(rows);
        validate_rows(&self.rows)?;
        Ok(self)
    }

    /// True when `id` is a published soccer row (never a battle).
    pub fn is_soccer(&self, id: u32) -> bool {
        self.kind(id) == Some(Kind::Soccer)
    }

    /// Read `field_battles.json` (the same file the battle engine loads, so every row here is a
    /// battle `EncounterEngine::begin` accepts). Unknown fields are ignored.
    pub fn load(path: &Path) -> io::Result<Self> {
        let file: BattleFile =
            serde_json::from_str(&read_bounded(path)?).map_err(|e| invalid(&e.to_string()))?;
        Self::new(
            file.battles
                .into_iter()
                .map(|b| Activity {
                    id: b.id,
                    name_id: b.title_id,
                    description_id: b.description_id,
                    // -1 is "no icon" in the battle data; the portal shows image set 0 then.
                    icon_id: u32::try_from(b.icon_id).unwrap_or(0),
                    difficulty: b.difficulty,
                    required_level: b.required_combat_level,
                    kind: Kind::Battle,
                })
                .collect(),
        )
    }

    pub fn activities(&self) -> &[Activity] {
        &self.rows
    }

    /// The kind behind a client id, without admitting the player. Unknown ids and live rows
    /// nothing here runs are `None`.
    pub fn kind(&self, id: u32) -> Option<Kind> {
        self.lookup(id).ok().flatten().map(|a| a.kind)
    }

    /// Admission for a Play on client id `id`; the result is the service row it starts (its own
    /// id, which a live row's differs from). Order matters: an unknown or closed id reveals
    /// nothing about the player.
    pub fn admit(&self, id: u32, viewer: Viewer, busy: bool) -> Result<&Activity, Refusal> {
        let activity = self.lookup(id)?.ok_or(Refusal::NotOpen)?;
        if busy {
            return Err(Refusal::Busy);
        }
        match activity.kind {
            Kind::Battle => {
                if !viewer.combat_job {
                    return Err(Refusal::NotCombatJob);
                }
                if let Some(required) =
                    crate::encounter::level_refusal(viewer.level, activity.required_level)
                {
                    return Err(Refusal::LevelTooLow(required));
                }
            }
            // (R) Chess needs no job or level; the table is open to every player.
            Kind::Chess { .. } => {}
            // L097: the race and derby services decide admission at Play.
            Kind::Kart
            | Kind::Derby
            | Kind::Recipe { .. }
            | Kind::Gather { .. }
            | Kind::Fishing { .. }
            | Kind::Checkers
            | Kind::Music
            | Kind::TowerDefense
            | Kind::Garage
            | Kind::Pirates => {}
            // (R) Soccer is a world activity, not a combat-job gate. Its row
            // still honors the configured level floor; the original gate is unknown.
            Kind::Soccer => {
                if viewer.level < activity.required_level {
                    return Err(Refusal::LevelTooLow(activity.required_level));
                }
            }
            // A refusal comes before any state changes.
            Kind::CardGame { .. } => {
                if let Some(refusal) = self.card_refusal(activity.id) {
                    return Err(refusal);
                }
            }
        }
        Ok(activity)
    }

    /// The card-game data row behind a `Kind::CardGame` activity id.
    fn card(&self, id: u32) -> Option<&card_game::Row> {
        self.cards.as_ref()?.row(id)
    }

    /// (R) Treasure War has no service; a Trading Card Game row starts only when the card content
    /// has a launch block (T765) and its data row is joinable.
    fn card_refusal(&self, id: u32) -> Option<Refusal> {
        let launching = (self.cards.as_ref()).is_some_and(|cards| cards.launch().is_some());
        match self.card(id) {
            Some(card) if card.category == card_game::TREASURE_WAR_CATEGORY => {
                Some(Refusal::TreasureWarClosed)
            }
            Some(card) if launching && card.can_player_join => None,
            Some(_) => Some(Refusal::CardGameClosed),
            None => Some(Refusal::Unknown),
        }
    }

    /// CanPlayerJoin for a service row, the same gates as Play so the client greys out (client
    /// `MinigameDetail.onHorzItemSelect`) what Play would refuse: battles want a combat job at the
    /// level, soccer its level floor, a card row an open game and its data's flag; chess is open.
    /// A gather or recipe row wants its job (client 116214); L100 (R) a recipe row also wants the
    /// recipe learned and its rank in its job, active or not (the client's own row requirement is
    /// `CheckRankLastProfileByType`); only the mail towns have a gather rank floor.
    fn can_join(&self, a: &Activity, viewer: Viewer) -> bool {
        match a.kind {
            Kind::Battle => viewer.combat_job && viewer.level >= a.required_level,
            Kind::Chess { .. } => true,
            // Each driving job comes with its first car.
            Kind::Garage => viewer.jobs.contains(&48) || viewer.jobs.contains(&49),
            Kind::Soccer => viewer.level >= a.required_level,
            Kind::CardGame { .. } => self.card_refusal(a.id).is_none(),
            Kind::Recipe { recipe_id, job } => {
                viewer.jobs.contains(&job.profile_id())
                    && viewer.learned.contains(&recipe_id)
                    && viewer.crafts[job as usize] >= a.required_level
            }
            // L106: a mail town opens at its Postman rank floor.
            Kind::Gather { gather } => {
                viewer.jobs.contains(&gather.job())
                    && match gather {
                        Gather::Mail => viewer.postman >= a.required_level,
                        // L934: an ore row opens at its Miner rank.
                        Gather::Mining => {
                            viewer.crafts[CraftingJob::Miner as usize] >= a.required_level
                        }
                        Gather::Harvesting => true,
                    }
            }
            Kind::Kart
            | Kind::Derby
            | Kind::Fishing { .. }
            | Kind::Checkers
            | Kind::Music
            | Kind::TowerDefense
            | Kind::Pirates => true,
        }
    }

    /// A live row as the client gets it: every value from the row, CanPlayerJoin from its runner.
    fn live_wire<'a>(&'a self, row: &'a LiveRow, runner: Runner, viewer: Viewer) -> Wire<'a> {
        Wire {
            id: row.id,
            app_system_id: row.app_system_id,
            category: row.category,
            image_set: row.image_set_id,
            position: row.position,
            display_name: row.display_name_id,
            display_description: row.display_description_id,
            name: row.name_id,
            description: row.description_id,
            entry_type: row.entry_type,
            can_join: match runner {
                Runner::Service(i) => self.can_join(&self.rows[i], viewer),
                Runner::Wheel => true,
                Runner::Closed => false,
            },
            recommended_set: row.recommended_set_id,
            tutorial: row.tutorial_activity_id,
            members_only: row.members_only,
            detail: &row.detail_image,
            thumbnail: &row.thumbnail_image,
            difficulty: row.difficulty,
            content: row.content_id,
        }
    }

    /// A service row listed by its own id (`index` in `rows`); `None` when its data row is
    /// missing. With live rows only hidden field battles come here.
    fn own_wire(&self, index: usize, a: &Activity, viewer: Viewer) -> Option<Wire<'_>> {
        let plain = Wire {
            id: a.id,
            app_system_id: 0,
            category: 0,
            image_set: a.icon_id,
            position: index as u32 + 1,
            display_name: a.name_id,
            display_description: a.description_id,
            name: a.name_id,
            description: a.description_id,
            entry_type: ENTRY_WORLD_TUNNEL, // (R) card rows too; see `card_game.rs`
            can_join: self.can_join(a, viewer),
            recommended_set: 0,
            tutorial: 0,
            members_only: false,
            detail: ART,
            thumbnail: ART,
            difficulty: a.difficulty,
            content: 0,
        };
        Some(match a.kind {
            Kind::Battle if !self.live.is_empty() => Wire {
                category: HIDDEN_CATEGORY,
                detail: HIDDEN_DETAIL,
                thumbnail: HIDDEN_THUMBNAIL,
                ..plain
            },
            Kind::Battle => Wire {
                category: COMBAT_CATEGORY,
                ..plain
            },
            Kind::Chess { .. } => Wire {
                category: CHESS_CATEGORY,
                detail: CHESS_DETAIL,
                thumbnail: CHESS_THUMBNAIL,
                ..plain
            },
            Kind::Soccer => Wire {
                category: SOCCER_CATEGORY,
                detail: SOCCER_ART,
                thumbnail: SOCCER_ART,
                ..plain
            },
            Kind::CardGame { .. } => {
                let card = self.card(a.id)?;
                Wire {
                    app_system_id: card.app_system_id,
                    category: card.category,
                    position: card.position,
                    display_name: card.display_name_id,
                    display_description: card.display_description_id,
                    tutorial: card.tutorial_activity_id.unwrap_or(0),
                    detail: &card.detail_image,
                    thumbnail: &card.thumbnail_image,
                    ..plain
                }
            }
            // Only live rows list them (`with_live`).
            Kind::Kart
            | Kind::Derby
            | Kind::Recipe { .. }
            | Kind::Gather { .. }
            | Kind::Fishing { .. }
            | Kind::Checkers
            | Kind::Music
            | Kind::TowerDefense
            | Kind::Garage
            | Kind::Pirates => return None,
        })
    }

    /// 167/1/1 for entry type 1, the world activities: the whole list for this player. Sent at
    /// world entry and on every 167/1/4.
    pub fn list_packet(&self, viewer: Viewer) -> Vec<u8> {
        self.entry_list(ENTRY_WORLD_TUNNEL, viewer)
    }

    /// L095: 167/1/1 for entry type 2 (the live minigame rows), sent beside `list_packet`; `None`
    /// without live minigame rows.
    pub fn minigame_list_packet(&self, viewer: Viewer) -> Option<Vec<u8>> {
        self.live
            .iter()
            .any(|(row, _)| row.entry_type == ENTRY_CLIENT_TUNNEL)
            .then(|| self.entry_list(ENTRY_CLIENT_TUNNEL, viewer))
    }

    /// One row writer for every row of one entry type; the layout is the client reader `00be63e0`.
    fn entry_list(&self, entry_type: u32, viewer: Viewer) -> Vec<u8> {
        let live = (self.live.iter()).map(|(row, runner)| self.live_wire(row, *runner, viewer));
        let own = self
            .own_rows()
            .filter_map(|(index, a)| self.own_wire(index, a, viewer));
        let rows: Vec<Wire> = live
            .chain(own)
            .filter(|w| w.entry_type == entry_type)
            .collect();
        let mut out = header(SUB_LIST);
        out.extend(entry_type.to_le_bytes());
        out.extend((rows.len() as i32).to_le_bytes());
        for w in rows {
            for dword in [
                w.id,
                w.app_system_id,
                w.category,
                w.image_set,
                w.position,
                w.display_name,
                w.display_description,
                w.name,
                w.description,
                w.entry_type,
            ] {
                out.extend(dword.to_le_bytes());
            }
            out.push(u8::from(w.can_join));
            out.extend(w.recommended_set.to_le_bytes()); // RecommendedSetId
            out.extend(0u32.to_le_bytes()); // featured windows
            out.extend(w.tutorial.to_le_bytes()); // TutorialActivityId
            out.push(u8::from(w.members_only));
            for name in [w.detail, w.thumbnail] {
                out.extend((name.len() as i32).to_le_bytes());
                out.extend(name.as_bytes());
            }
            for dword in [w.difficulty, w.content, 0, 0] {
                out.extend(dword.to_le_bytes()); // Difficulty, ContentId, +0x68, MysteryChestIcon
            }
        }
        out
    }
}

/// One 167/1/1 row as written.
struct Wire<'a> {
    id: u32,
    app_system_id: u32,
    category: u32,
    image_set: u32,
    position: u32,
    display_name: u32,
    display_description: u32,
    name: u32,
    description: u32,
    entry_type: u32,
    can_join: bool,
    recommended_set: u32,
    tutorial: u32,
    members_only: bool,
    detail: &'a str,
    thumbnail: &'a str,
    difficulty: u32,
    content: u32,
}

/// L095: whether service row `a` runs live row `row`: a battle or soccer row by its tile and
/// name, chess by its MiniGameData row (an entry-type-2 ContentId), a card row by id (the card
/// data already carries the live ids); L097: the kart race and the derby by their MiniGameData
/// row (the service row's id).
fn runs(row: &LiveRow, a: &Activity) -> bool {
    match a.kind {
        Kind::Battle => row.category == COMBAT_CATEGORY && row.name_id == a.name_id,
        Kind::Soccer => row.category == SOCCER_CATEGORY && row.name_id == a.name_id,
        Kind::Chess { row_id } => row.entry_type == ENTRY_CLIENT_TUNNEL && row.content_id == row_id,
        Kind::CardGame { .. } | Kind::Garage => row.id == a.id,
        Kind::Kart
        | Kind::Derby
        | Kind::Recipe { .. }
        | Kind::Gather { .. }
        | Kind::Fishing { .. }
        | Kind::Checkers
        | Kind::Music
        | Kind::TowerDefense => row.entry_type == ENTRY_CLIENT_TUNNEL && row.content_id == a.id,
        Kind::Pirates => row.entry_type == ENTRY_WORLD_TUNNEL && row.content_id == a.id,
    }
}

/// L097: the service row for a race or derby the server runs, as a Games window join key.
pub fn game_row(minigame_id: u32, kind: Kind) -> Activity {
    Activity {
        id: minigame_id,
        name_id: 0,
        description_id: 0,
        icon_id: 0,
        difficulty: 1,
        required_level: 0,
        kind,
    }
}

fn validate_live(rows: &[LiveRow]) -> io::Result<()> {
    if rows.len() > MAX_LIVE_ROWS {
        return Err(invalid("too many live activities"));
    }
    for (i, row) in rows.iter().enumerate() {
        if row.id == 0 || rows[..i].iter().any(|other| other.id == row.id) {
            return Err(invalid("live activity ids must be unique and nonzero"));
        }
        if !matches!(row.entry_type, ENTRY_WORLD_TUNNEL | ENTRY_CLIENT_TUNNEL) {
            return Err(invalid("unknown activity entry type"));
        }
        if [&row.detail_image, &row.thumbnail_image]
            .iter()
            .any(|name| name.len() > MAX_IMAGE_NAME || !name.is_ascii())
        {
            return Err(invalid("bad activity image name"));
        }
    }
    Ok(())
}

/// A content file, refused past `MAX_FILE`.
fn read_bounded(path: &Path) -> io::Result<String> {
    let mut text = String::new();
    fs::File::open(path)?
        .take(MAX_FILE + 1)
        .read_to_string(&mut text)?;
    if text.len() as u64 > MAX_FILE {
        return Err(invalid("activity file too large"));
    }
    Ok(text)
}

/// Decode a client message of family 167. `Ok(None)` for other families and for sub-ops the
/// client never sends here; `Err` for a Play or Refresh with the wrong length (forged).
pub fn parse(inner: &[u8]) -> io::Result<Option<Request>> {
    if inner.len() < 4 || inner[..2] != FAMILY.to_le_bytes() || inner[2] != SERVICE_ACTIVITY {
        return Ok(None);
    }
    match inner[3] {
        SUB_REFRESH if inner.len() == 4 => Ok(Some(Request::Refresh)),
        SUB_PLAY if inner.len() == PLAY_LEN => Ok(Some(Request::Play {
            // The trailing dword is the client clock offset; ignored on purpose.
            activity: crate::le::u32(inner, 4)
                .ok_or_else(|| invalid("malformed activity request"))?,
        })),
        SUB_REFRESH | SUB_PLAY => Err(invalid("malformed activity request")),
        _ => Ok(None),
    }
}

fn header(sub_op: u8) -> Vec<u8> {
    let mut out = FAMILY.to_le_bytes().to_vec();
    out.extend([SERVICE_ACTIVITY, sub_op]);
    out
}

fn validate_rows(rows: &[Activity]) -> io::Result<()> {
    if rows.len() > MAX_ACTIVITIES {
        return Err(invalid("too many activities"));
    }
    for (i, row) in rows.iter().enumerate() {
        if row.id == 0 || rows[..i].iter().any(|other| other.id == row.id) {
            return Err(invalid("activity ids must be unique and nonzero"));
        }
    }
    Ok(())
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}
