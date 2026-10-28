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
//! * The row's MiniGameData link (`minigame_id`) stays in data; no wire field is known to hold it.

use crate::activity_portal::{Activity, Kind};
use crate::card_service::LaunchTickets;
use crate::minigame::{MinigameRuns, RunMode, RunOutcome, RunSpec, RunState, card_start_packet};
use crate::startup_messages;
use serde::Deserialize;
use std::{
    collections::BTreeSet,
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
const MAX_IMAGE_NAME: usize = 64;

/// One published row: the values of one 167/1/1 row (client reader `00be63e0`) plus provenance.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub id: u32,
    /// The client `MiniGameData.txt` row this activity opens (data only, see the module docs).
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
    /// T765: present once a card service runs; until then every card row stays refused.
    #[serde(default)]
    launch: Option<Launch>,
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
    pub source: Vec<Source>,
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
    /// T825: the starter collection the card service syncs after the lobby (empty: no sync). R:
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
/// results window sends this one; the card results path does not exist yet.
const RESULTS_CLOSE: [u8; 4] = [26, 0, 42, 0];

/// T765: one session's card launches: the type-16 run lifecycle and the one-time tickets its
/// 39/37 mints. Session-only; never saved.
#[derive(Clone, Debug)]
pub struct Launches {
    runs: MinigameRuns,
    tickets: LaunchTickets,
    preload_file: String,
}

impl Launches {
    pub fn new(launch: &Launch) -> io::Result<Self> {
        Ok(Self {
            runs: MinigameRuns::new(FIRST_INSTANCE_ID)?,
            tickets: LaunchTickets::new(launch.ticket_ttl_ms),
            preload_file: launch.preload_file.clone(),
        })
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
        self.runs.offer(spec)
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
        let mut ids = BTreeSet::new();
        for row in &self.rows {
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

    /// The row behind activity `id`.
    pub fn row(&self, id: u32) -> Option<&Row> {
        self.rows.iter().find(|row| row.id == id)
    }

    /// The catalogue rows (ids, names, image, difficulty); the rest of each wire row is read
    /// back from `rows()` by `Catalogue::list_packet`.
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

    fn launches() -> Launches {
        let launch: Launch = serde_json::from_value(serde_json::json!({
            "ticket_ttl_ms": 120_000,
            "source": [{"kind": "reconstructed", "evidence": "F800 test"}]
        }))
        .unwrap();
        Launches::new(&launch).unwrap()
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
