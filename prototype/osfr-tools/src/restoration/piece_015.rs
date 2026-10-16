//! Piece 015: recovered minigame launch catalogue.
//!
//! ORIGINAL FACTS (recovered, preserved in the lab): the ORIGINAL client ships four
//! caret-delimited tables under `Resources/` and every row of `MiniGameData.txt`
//! names a `TYPE` row whose `IS_FLASH` flag declares the client mode:
//!
//! * `MiniGameData.txt`       - one row per launchable activity (783 rows)
//! * `MiniGameTypeData.txt`   - 22 declared types with the `IS_FLASH` flag
//! * `MiniGameGroupData.txt`  - 20 declared groups
//! * `MiniGameGroupLinks.txt` - 610 group -> game links
//!
//! Hash-pinned copies and the full 783-row mapping with implementation coverage
//! live in `lab/reports/piece-015-20261009T030000Z/` and
//! `notes/Research/Pieces/015 Minigame launch catalogue.md`.
//!
//! RECONSTRUCTED: this module parses the shipped tables and exposes a launch-mode
//! lookup for the server. It is not registered in `lib.rs`; the lead owns that
//! integration, and no gameplay behaviour is claimed from the table alone.

use std::collections::{BTreeMap, BTreeSet};

pub const GAME_TABLE: &str = "MiniGameData.txt";
pub const TYPE_TABLE: &str = "MiniGameTypeData.txt";
pub const GROUP_TABLE: &str = "MiniGameGroupData.txt";
pub const LINK_TABLE: &str = "MiniGameGroupLinks.txt";

/// Client mode declared by `MiniGameTypeData.IS_FLASH` for a game row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    Flash,
    Native,
    /// The row references a type id the shipped type table does not declare.
    Undeclared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRow {
    pub id: u32,
    pub kind: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
    pub min_players: u32,
    pub max_players: u32,
    pub leaderboard_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRow {
    pub id: u32,
    pub is_flash: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupLink {
    pub id: u32,
    pub group: u32,
    pub link_game: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CatalogueCounts {
    pub games: usize,
    pub types: usize,
    pub groups: usize,
    pub links: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogueError {
    MissingHeader {
        source: &'static str,
    },
    Row {
        source: &'static str,
        line: usize,
        fields: usize,
        expected: usize,
    },
    Number {
        source: &'static str,
        line: usize,
        column: &'static str,
    },
    DuplicateId {
        source: &'static str,
        id: u32,
    },
}

struct Table {
    header: Vec<String>,
    rows: Vec<(usize, Vec<String>)>,
}

impl Table {
    fn parse(source: &'static str, text: &str) -> Result<Self, CatalogueError> {
        let mut header: Option<Vec<String>> = None;
        let mut rows = Vec::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let trimmed = raw.trim_end_matches('\r');
            if trimmed.trim().is_empty() {
                continue;
            }
            let Some(header_fields) = header.as_ref() else {
                let body = trimmed.strip_prefix('#').unwrap_or(trimmed);
                header = Some(body.split('^').map(str::to_string).collect());
                continue;
            };
            let fields: Vec<String> = trimmed.split('^').map(str::to_string).collect();
            if fields.len() != header_fields.len() {
                return Err(CatalogueError::Row {
                    source,
                    line,
                    fields: fields.len(),
                    expected: header_fields.len(),
                });
            }
            rows.push((line, fields));
        }
        let header = header.ok_or(CatalogueError::MissingHeader { source })?;
        Ok(Self { header, rows })
    }

    fn column(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|field| field == name)
    }

    fn value<'a>(&self, row: &'a [String], name: &str) -> &'a str {
        self.column(name)
            .and_then(|index| row.get(index))
            .map(String::as_str)
            .unwrap_or("")
    }

    fn required_u32(
        &self,
        source: &'static str,
        line: usize,
        row: &[String],
        name: &'static str,
    ) -> Result<u32, CatalogueError> {
        self.value(row, name)
            .parse::<u32>()
            .map_err(|_| CatalogueError::Number {
                source,
                line,
                column: name,
            })
    }

    /// Shipped rows leave optional numeric cells empty; those read as 0.
    fn optional_u32(&self, row: &[String], name: &str) -> u32 {
        self.value(row, name).parse::<u32>().unwrap_or(0)
    }
}

/// Recovered launch catalogue: game rows, declared types, groups and links.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchCatalogue {
    games: BTreeMap<u32, GameRow>,
    types: BTreeMap<u32, TypeRow>,
    groups: BTreeSet<u32>,
    links: Vec<GroupLink>,
}

impl LaunchCatalogue {
    /// Parse the four preserved tables. Row order is irrelevant; ids are unique.
    pub fn parse(
        game_text: &str,
        type_text: &str,
        group_text: &str,
        link_text: &str,
    ) -> Result<Self, CatalogueError> {
        let games_table = Table::parse(GAME_TABLE, game_text)?;
        let types_table = Table::parse(TYPE_TABLE, type_text)?;
        let groups_table = Table::parse(GROUP_TABLE, group_text)?;
        let links_table = Table::parse(LINK_TABLE, link_text)?;

        let mut games = BTreeMap::new();
        for (line, row) in &games_table.rows {
            let id = games_table.required_u32(GAME_TABLE, *line, row, "ID")?;
            let game = GameRow {
                id,
                kind: games_table.required_u32(GAME_TABLE, *line, row, "TYPE")?,
                name_id: games_table.optional_u32(row, "NAME_ID"),
                description_id: games_table.optional_u32(row, "DESCRIPTION_ID"),
                icon_id: games_table.optional_u32(row, "ICON_ID"),
                difficulty: games_table.optional_u32(row, "DIFFICULTY"),
                min_players: games_table.optional_u32(row, "MIN_PLAYERS"),
                max_players: games_table.optional_u32(row, "MAX_PLAYERS"),
                leaderboard_id: games_table.optional_u32(row, "LEADERBOARD_ID"),
            };
            if games.insert(id, game).is_some() {
                return Err(CatalogueError::DuplicateId {
                    source: GAME_TABLE,
                    id,
                });
            }
        }

        let mut types = BTreeMap::new();
        for (line, row) in &types_table.rows {
            let id = types_table.required_u32(TYPE_TABLE, *line, row, "TYPE")?;
            let entry = TypeRow {
                id,
                is_flash: types_table.value(row, "IS_FLASH") == "1",
            };
            if types.insert(id, entry).is_some() {
                return Err(CatalogueError::DuplicateId {
                    source: TYPE_TABLE,
                    id,
                });
            }
        }

        let mut groups = BTreeSet::new();
        for (line, row) in &groups_table.rows {
            let id = groups_table.required_u32(GROUP_TABLE, *line, row, "ID")?;
            if !groups.insert(id) {
                return Err(CatalogueError::DuplicateId {
                    source: GROUP_TABLE,
                    id,
                });
            }
        }

        let mut links = Vec::new();
        for (line, row) in &links_table.rows {
            links.push(GroupLink {
                id: links_table.required_u32(LINK_TABLE, *line, row, "ID")?,
                group: links_table.required_u32(LINK_TABLE, *line, row, "GROUP_ID")?,
                link_game: links_table.required_u32(LINK_TABLE, *line, row, "LINK_GAME_ID")?,
            });
        }

        Ok(Self {
            games,
            types,
            groups,
            links,
        })
    }

    pub fn game(&self, id: u32) -> Option<&GameRow> {
        self.games.get(&id)
    }

    /// Declared client mode for a game row, or `None` for an unknown row.
    pub fn launch_mode(&self, id: u32) -> Option<LaunchMode> {
        let game = self.games.get(&id)?;
        Some(match self.types.get(&game.kind) {
            Some(entry) if entry.is_flash => LaunchMode::Flash,
            Some(_) => LaunchMode::Native,
            None => LaunchMode::Undeclared,
        })
    }

    /// Game ids linked to a group, sorted; unknown ids and groups read empty.
    pub fn games_in_group(&self, group: u32) -> Vec<u32> {
        let mut ids: Vec<u32> = self
            .links
            .iter()
            .filter(|link| link.group == group && self.games.contains_key(&link.link_game))
            .map(|link| link.link_game)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    pub fn counts(&self) -> CatalogueCounts {
        CatalogueCounts {
            games: self.games.len(),
            types: self.types.len(),
            groups: self.groups.len(),
            links: self.links.len(),
        }
    }
}
