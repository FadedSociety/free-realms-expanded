//! L104: the checkers table's glue for `Gameplay`: Play on the Games window row offers the
//! table, and its lifecycle and movie packets route to it. The rules and the table live in
//! `crate::checkers`.
use super::*;
use crate::npc_interaction::{TableMenu, TableMenus};

const BOARD_TABLES_FILE: &str = "board_tables.json";
const MAX_BOARD_TABLES: u64 = 64 * 1024;
/// MiniGameData's checkers row; a table's other game is a chess row.
const CHECKERS_GAME: u32 = 21;

/// `board_tables.json`: the world's game tables and what each offers.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BoardTables {
    schema_version: u32,
    source: String,
    name: String,
    /// Game id and the menu row's label.
    games: Vec<(u32, u32)>,
    tables: Vec<BoardTable>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BoardTable {
    npc: u64,
    position: [f32; 3],
}

/// Each game table's menu; no file, no tables.
pub(super) fn board_table_menus(content: &Path) -> io::Result<TableMenus> {
    let path = content.join(BOARD_TABLES_FILE);
    if !path.is_file() {
        return Ok(TableMenus::new());
    }
    if std::fs::metadata(&path)?.len() > MAX_BOARD_TABLES {
        return Err(invalid("board tables too large"));
    }
    let file: BoardTables = serde_json::from_slice(&std::fs::read(&path)?).map_err(invalid)?;
    if file.schema_version != 1 || file.source.is_empty() {
        return Err(invalid("unsupported board tables"));
    }
    let menu = |table: &BoardTable| TableMenu {
        name: file.name.clone(),
        position: table.position,
        games: file.games.clone(),
    };
    Ok(file.tables.iter().map(|table| (table.npc, menu(table))).collect())
}

impl Gameplay {
    /// A table's choice: the same offer as that game's Games window row.
    pub(super) fn begin_board_game(&mut self, game: u32, player_id: u64) -> io::Result<Outcome> {
        // The Games window's own gate: a knocked-out player can't sit down either.
        if self.warp_blocker().is_some() {
            return Ok(Outcome {
                replies: vec![chat::system(&crate::activity_portal::Refusal::Busy.message())],
                ..Outcome::default()
            });
        }
        if game == CHECKERS_GAME {
            return self.begin_checkers();
        }
        if (self.sword.as_ref()).is_some_and(|sword| sword.game_id() as u32 == game) {
            return self.begin_sword();
        }
        let mut out = Outcome::default();
        self.begin_chess(game, player_id, &mut out)?;
        Ok(out)
    }

    /// Play on the checkers row: the 39/16 offer, or one refusal line while a game is open.
    pub(super) fn begin_checkers(&mut self) -> io::Result<Outcome> {
        Ok(match self.checkers.begin()? {
            Some(table) => Outcome {
                changed: table.changed,
                replies: table.replies,
                ..Outcome::default()
            },
            None => Outcome {
                replies: vec![chat::system("Finish your checkers game first.")],
                ..Outcome::default()
            },
        })
    }

    /// One packet while the table is open (`None` otherwise). As for chess, only the lifecycle
    /// result (39/18) credits the minigame, once.
    /// L920: a won game pays its coins on the results page, adopted before the reward packets.
    pub(super) fn handle_checkers(
        &mut self,
        inner: &[u8],
        player_id: u64,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !self.checkers.is_active() {
            return Ok(None);
        }
        let rewards = self.reward_snapshot();
        let Some(table) = self
            .checkers
            .route_with(inner, now_ms, Some((player_id, &rewards)))?
        else {
            return Ok(None);
        };
        let mut out = Outcome {
            changed: table.changed,
            replies: table.replies.clone(),
            ..Outcome::default()
        };
        if let Some(next) = table.reward_state {
            self.adopt_reward_state(next, &mut out)?;
        }
        out.replies.extend(table.reward_packets);
        if table.replies.iter().any(|p| p.starts_with(&[39, 0, 18])) {
            self.npc_minigame_win(&table.replies, &mut out)?;
        }
        Ok(Some(out))
    }
}
