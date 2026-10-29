//! Harvesting (original MatchThree "Farming" movie, MiniGameData type 8).
//!
//! Recovered original material: the `game_farming.gfx` script package
//! (`GameData.as`, `GameServer.as`, `core/Grid.as`), the caret-delimited
//! `MatchThree*Data.txt` schema those scripts read, the shared type-8 message
//! names, per-region produce sets from the original locale descriptions
//! (94149..94154), and bundle item definitions from the pinned
//! `ClientItemDefinitions.json`. Board layout, quotas, bundle cadence, timer
//! and spawn percentages on the live servers were never shipped with the
//! client and remain labelled reconstructions; audit
//! `notes/Research/Harvesting Restoration.md` before treating them as
//! historical. This module owns harvesting-specific configuration, goal and
//! drop proposals and the original schema oracle only: the generic type-8
//! board engine belongs to the shared engine module supplied by the mining
//! restoration task.
//!
//! Executable geometry: the shared engine only runs full rectangles with a
//! dropzone row, so the emitted spec carries a 10x9 playable rectangle plus one
//! dropzone row inside an 11x11 grid whose row 0 and column 0 stay empty (the
//! original fall loops never visit index 0; see
//! `notes/Research/Harvesting Border Compatibility.md`). The staggered board
//! drawn by the pinned screenshot stays recorded by [`diamond_shape`] and is
//! labelled as a fidelity gap in every `rule_notice`; it is not silently
//! claimed as the executable layout. Launch assets are selectable content
//! fields with the recovered farming names ([`FARMING_PRELOAD_FILE`],
//! [`FARMING_LAUNCH_MOVIE`]).
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

#[path = "harvesting_lifecycle.rs"]
pub mod lifecycle;

/// Outer message family shared by every type-8 minigame (39/x).
pub const TYPE8_GAME_FAMILY: u16 = 39;
/// File name handed to the shared type-8 engine loader.
pub const CONTENT_FILE: &str = "harvesting.json";
pub const SCHEMA_VERSION: u32 = 1;
/// Recovered farming preload asset list (`Assets_000.pack` entry 50; 97 names
/// ending in `game_farming.lst`), sent in `39/16` in place of the mining list.
pub const FARMING_PRELOAD_FILE: &str = "FarmingAssets.txt";
/// Recovered farming movie (`AssetsW_009.pack` entry 1448; `.lst` srcfilename
/// `.../Minigames/MatchThree/Farming/game_farming.swf`), requested through
/// `26/12 MiniGameFlash`.
pub const FARMING_LAUNCH_MOVIE: &str = "game_farming.gfx";
/// (R) Local instance id namespace above the Mining and Smelting ranges.
pub const FIRST_INSTANCE_ID: i32 = 58_000_001;

/// Generic "Harvesting" definition (58): shared description only, no produce.
pub const GENERIC_HARVESTING_GAME_ID: i32 = 58;
/// Regional definitions whose produce sets are named in the original locale.
pub const RECOVERED_REGION_GAME_IDS: [i32; 6] = [109, 111, 113, 115, 117, 119];
/// Additional "Harvesting" rows that reuse the generic name/description and
/// carry no produce evidence. Documented gap: do not invent a produce set.
pub const UNNAMED_VARIANT_GAME_IDS: [i32; 7] = [110, 112, 114, 116, 118, 120, 217];

/// Original harvesting icon shared by 58 and every regional row.
pub const HARVESTING_ICON_ID: i32 = 908;

/// (R) Local Chef binding, supported by the Chef page's harvesting XP note;
/// the original server's profile gate has not been captured.
pub const CHEF_PROFILE_ID: i32 = 45;

// ---------------------------------------------------------------------------
// Original scoring and timing constants (GameData.as, verbatim values).
// ---------------------------------------------------------------------------
pub const MIN_CHAIN_TILES: u32 = 3;
pub const CHAIN_SCORE: i64 = 400;
pub const CHAIN_BONUS_TILE_SCORE: i64 = 200;
pub const COLLECTABLE_SCORE: i64 = 500;
pub const BONUS_SCORE: i64 = 2000;
pub const POWERUP_SCORE: i64 = 250;
pub const REMAINING_TIME_SCORE: i64 = 25;
pub const MIN_CHAIN_TIME_BONUS_SECONDS: u32 = 1;
pub const TILE_SIZE_PIXELS: u32 = 40;
/// ORIGINAL `GameData.TILE_SELECTABLE`: `core/TileGenerator.as::SetTileInfo` derives the
/// tile-kind branch as `Math.floor(value / 100) * 100`, so a selectable value's category
/// is this 400 block.
pub const TILE_SELECTABLE_CATEGORY: u16 = 400;
/// ORIGINAL `GameData.TILE_COLLECTABLE`, the 600 block of the same derived
/// category. Harvesting never uses it: the shipped client
/// `Resources/CodeStringMappings.txt` names only `farmingValue401..416`, so a
/// 600-block basket keeps the unresolved `##farmingValue60…` placeholder
/// (native run `client-baseline-20261009T202011Z`, screenshot
/// `harvesting-placeholder-goals.jpg`).
pub const TILE_COLLECTABLE_CATEGORY: u16 = 600;
/// First recovered produce value. The original server invariant is
/// `value = TILE_SELECTABLE + type` with the 1-based produce type; the type is
/// both the `farmingValue<value>` code-string number and the
/// `SelectableTile<type>` frame number in `game_farming.gfx`.
pub const FIRST_PRODUCE_VALUE: u16 = TILE_SELECTABLE_CATEGORY + 1;

/// Recovered value (`400 + type`) of a [`PRODUCE`] row, by 0-based index.
///
/// `farmingValue401..416` in the shipped client's `CodeStringMappings.txt`
/// resolves to the same sixteen names in the same order as [`PRODUCE`] and the
/// bundle locale ids `23285..23300`.
pub fn produce_value(index: usize) -> Option<u16> {
    (index < PRODUCE.len()).then_some(FIRST_PRODUCE_VALUE + index as u16)
}

/// 0-based [`PRODUCE`] index of a recovered produce value.
pub fn produce_index(value: u16) -> Option<usize> {
    let index = value.checked_sub(FIRST_PRODUCE_VALUE)? as usize;
    (index < PRODUCE.len()).then_some(index)
}

/// Recovered produce value for a catalogue slug.
pub fn produce_value_for(slug: &str) -> Option<u16> {
    PRODUCE
        .iter()
        .position(|p| p.slug == slug)
        .and_then(produce_value)
}

/// `GameLevel.CalculateChainScore`: base 400 plus 200 per tile past the
/// minimum chain of three.
pub fn chain_score(chain_tiles: u32) -> i64 {
    let mut score = CHAIN_SCORE;
    if chain_tiles > MIN_CHAIN_TILES {
        score += i64::from(chain_tiles - MIN_CHAIN_TILES) * CHAIN_BONUS_TILE_SCORE;
    }
    score
}

/// `GameLevel.CalculateChainTimeBonus` without the (unrecovered) progression
/// term: two seconds per chain tile, zero for an empty selection, never below
/// one second once a chain exists.
pub fn chain_time_bonus_seconds(chain_tiles: u32) -> u32 {
    if chain_tiles == 0 {
        return 0;
    }
    (chain_tiles * 2).max(MIN_CHAIN_TIME_BONUS_SECONDS)
}

/// `GameLevel.HandleCollectableTile`: 500 points per collected tile, doubled
/// once the goal's own quota has been reached.
pub fn collectable_score(collected_tiles_after: u32, quota: u32) -> i64 {
    if quota > 0 && collected_tiles_after >= quota {
        COLLECTABLE_SCORE * 2
    } else {
        COLLECTABLE_SCORE
    }
}

// ---------------------------------------------------------------------------
// Recovered produce catalogue.
// ---------------------------------------------------------------------------

/// One harvesting produce item.
///
/// `name_locale_id` is the "Bundle of ..." locale entry (23285..23300);
/// `item_definition_id` is that locale entry's `NameId` in the pinned
/// FreeRealms-Legacy `ClientItemDefinitions.json` (all `Type: 1`,
/// `MaxStackSize: -1`). `duplicate_definition_id` records the later reissued
/// definition that repeats the same `NameId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Produce {
    pub slug: &'static str,
    pub item_definition_id: i32,
    pub name_locale_id: i32,
    pub icon_id: i32,
    pub duplicate_definition_id: Option<i32>,
}

pub const PRODUCE: [Produce; 16] = [
    Produce {
        slug: "gummyfruit",
        item_definition_id: 2875,
        name_locale_id: 23285,
        icon_id: 706,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "snowberry",
        item_definition_id: 2876,
        name_locale_id: 23286,
        icon_id: 711,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "spiralmint",
        item_definition_id: 2877,
        name_locale_id: 23287,
        icon_id: 712,
        duplicate_definition_id: Some(30574),
    },
    Produce {
        slug: "moonbean",
        item_definition_id: 2878,
        name_locale_id: 23288,
        icon_id: 708,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "choychoy",
        item_definition_id: 2879,
        name_locale_id: 23289,
        icon_id: 704,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "sunleaf",
        item_definition_id: 2880,
        name_locale_id: 23290,
        icon_id: 716,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "honeytwig",
        item_definition_id: 2881,
        name_locale_id: 23291,
        icon_id: 707,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "punkin",
        item_definition_id: 2882,
        name_locale_id: 23292,
        icon_id: 709,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "briarmelon",
        item_definition_id: 2883,
        name_locale_id: 23293,
        icon_id: 703,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "rainleaf",
        item_definition_id: 2884,
        name_locale_id: 23294,
        icon_id: 710,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "stickfruit",
        item_definition_id: 2885,
        name_locale_id: 23295,
        icon_id: 715,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "starnut",
        item_definition_id: 2886,
        name_locale_id: 23296,
        icon_id: 714,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "stainberry",
        item_definition_id: 2887,
        name_locale_id: 23297,
        icon_id: 713,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "chugaroot",
        item_definition_id: 2888,
        name_locale_id: 23298,
        icon_id: 705,
        duplicate_definition_id: None,
    },
    Produce {
        slug: "bumbleberry",
        item_definition_id: 2889,
        name_locale_id: 23299,
        icon_id: 667,
        duplicate_definition_id: Some(30575),
    },
    Produce {
        slug: "tumtumnip",
        item_definition_id: 2890,
        name_locale_id: 23300,
        icon_id: 717,
        duplicate_definition_id: None,
    },
];

pub fn produce(slug: &str) -> Option<&'static Produce> {
    PRODUCE.iter().find(|p| p.slug == slug)
}

/// One regional harvesting definition as it appears in MiniGameData.
///
/// `produce` keeps the order of the original locale description
/// (94149..94154). Every slot's produce is a *selectable* tile: the board value
/// is `400 + produce type` ([`produce_value_for`]), the basket label is the
/// client's `farmingValue<value>` string and the basket icon is the movie's
/// `SelectableTile<type>` frame (`GameLevel.AddSelectableSpawnData` +
/// `ServerTileGenerator.SendTileInfo(400 + GetType(), GetType())`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub game_id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub icon_id: i32,
    pub difficulty: u8,
    pub produce: [&'static str; 4],
}

pub const REGIONS: [Region; 6] = [
    Region {
        game_id: 109,
        name_id: 100833,
        description_id: 94149,
        icon_id: HARVESTING_ICON_ID,
        difficulty: 2,
        produce: ["gummyfruit", "rainleaf", "bumbleberry", "honeytwig"],
    },
    Region {
        game_id: 111,
        name_id: 100834,
        description_id: 94150,
        icon_id: HARVESTING_ICON_ID,
        difficulty: 2,
        produce: ["stickfruit", "stainberry", "tumtumnip", "spiralmint"],
    },
    Region {
        game_id: 113,
        name_id: 100838,
        description_id: 94151,
        icon_id: HARVESTING_ICON_ID,
        difficulty: 2,
        produce: ["rainleaf", "chugaroot", "choychoy", "gummyfruit"],
    },
    Region {
        game_id: 115,
        name_id: 100836,
        description_id: 94152,
        icon_id: HARVESTING_ICON_ID,
        difficulty: 2,
        produce: ["snowberry", "stainberry", "starnut", "spiralmint"],
    },
    Region {
        game_id: 117,
        name_id: 100837,
        description_id: 94153,
        icon_id: HARVESTING_ICON_ID,
        difficulty: 2,
        produce: ["punkin", "sunleaf", "rainleaf", "gummyfruit"],
    },
    Region {
        game_id: 119,
        name_id: 100869,
        description_id: 94154,
        icon_id: HARVESTING_ICON_ID,
        difficulty: 2,
        produce: ["spiralmint", "briarmelon", "stainberry", "moonbean"],
    },
];

pub fn region(game_id: i32) -> Option<&'static Region> {
    REGIONS.iter().find(|r| r.game_id == game_id)
}

// ---------------------------------------------------------------------------
// Original content schema oracle (MatchThree*Data.txt, GameServer.as).
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum HarvestError {
    Schema(String),
    Content(String),
    Io(std::io::Error),
}

impl fmt::Display for HarvestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HarvestError::Schema(m) => write!(f, "schema: {m}"),
            HarvestError::Content(m) => write!(f, "content: {m}"),
            HarvestError::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl std::error::Error for HarvestError {}

impl From<std::io::Error> for HarvestError {
    fn from(e: std::io::Error) -> Self {
        HarvestError::Io(e)
    }
}

/// Split one original caret-delimited row, mirroring `ChopLastChar` +
/// `split("^")`, and require an exact column count.
pub fn parse_caret_exact(line: &str, columns: usize) -> Result<Vec<i32>, HarvestError> {
    let trimmed = line.trim_end_matches(['\r', '\n', '\0']);
    let fields: Vec<&str> = trimmed.split('^').collect();
    if fields.len() != columns {
        return Err(HarvestError::Schema(format!(
            "expected {columns} caret columns, found {}",
            fields.len()
        )));
    }
    fields
        .iter()
        .map(|f| {
            f.trim()
                .parse::<i32>()
                .map_err(|_| HarvestError::Schema(format!("invalid integer {f:?}")))
        })
        .collect()
}

/// Convenience helper for tests and ad-hoc parsing of caret rows of any
/// length.
pub fn parse_caret(line: &str) -> Result<Vec<i32>, HarvestError> {
    let trimmed = line.trim_end_matches(['\r', '\n', '\0']);
    let fields: Vec<&str> = trimmed.split('^').collect();
    fields
        .iter()
        .map(|f| {
            f.trim()
                .parse::<i32>()
                .map_err(|_| HarvestError::Schema(format!("invalid integer {f:?}")))
        })
        .collect()
}

fn to_rows(values: &[i32]) -> String {
    values
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join("^")
}

/// `MatchThreeSpawnData.txt` row (GameServer.as SD_* order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnRow {
    pub id: i32,
    pub category: i32,
    pub spawn_kind: i32,
    pub spawn_chance: i32,
    pub create_maximum: i32,
    pub collect_maximum: i32,
    pub quota: i32,
    pub reward_set: i32,
    pub icon_id: i32,
    pub duration: i32,
    pub power: i32,
    pub per_reward: i32,
    pub collection_type: i32,
    pub timer_id: i32,
}

impl SpawnRow {
    pub const COLUMNS: usize = 14;

    pub fn parse(line: &str) -> Result<Self, HarvestError> {
        let v = parse_caret_exact(line, Self::COLUMNS)?;
        Ok(Self {
            id: v[0],
            category: v[1],
            spawn_kind: v[2],
            spawn_chance: v[3],
            create_maximum: v[4],
            collect_maximum: v[5],
            quota: v[6],
            reward_set: v[7],
            icon_id: v[8],
            duration: v[9],
            power: v[10],
            per_reward: v[11],
            collection_type: v[12],
            timer_id: v[13],
        })
    }

    pub fn to_row(&self) -> String {
        to_rows(&[
            self.id,
            self.category,
            self.spawn_kind,
            self.spawn_chance,
            self.create_maximum,
            self.collect_maximum,
            self.quota,
            self.reward_set,
            self.icon_id,
            self.duration,
            self.power,
            self.per_reward,
            self.collection_type,
            self.timer_id,
        ])
    }
}

/// `MatchThreeLevelData.txt` row (GameServer.as LD_* order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelRow {
    pub id: i32,
    pub game: i32,
    pub level_index: i32,
    pub duration: i32,
    pub objective_id: i32,
    pub start_id: i32,
    pub end_id: i32,
    pub grid_id: i32,
    pub collectable_spawn: i32,
    pub bonus_spawn: i32,
    pub selectable_spawn: i32,
    pub powerup_spawn: i32,
    pub hazard_spawn: i32,
    pub blocker_spawn: i32,
    pub wildcard_spawn: i32,
    pub reward: i32,
    pub kind: i32,
    pub type_data: i32,
}

impl LevelRow {
    pub const COLUMNS: usize = 18;

    pub fn parse(line: &str) -> Result<Self, HarvestError> {
        let v = parse_caret_exact(line, Self::COLUMNS)?;
        Ok(Self {
            id: v[0],
            game: v[1],
            level_index: v[2],
            duration: v[3],
            objective_id: v[4],
            start_id: v[5],
            end_id: v[6],
            grid_id: v[7],
            collectable_spawn: v[8],
            bonus_spawn: v[9],
            selectable_spawn: v[10],
            powerup_spawn: v[11],
            hazard_spawn: v[12],
            blocker_spawn: v[13],
            wildcard_spawn: v[14],
            reward: v[15],
            kind: v[16],
            type_data: v[17],
        })
    }

    pub fn to_row(&self) -> String {
        to_rows(&[
            self.id,
            self.game,
            self.level_index,
            self.duration,
            self.objective_id,
            self.start_id,
            self.end_id,
            self.grid_id,
            self.collectable_spawn,
            self.bonus_spawn,
            self.selectable_spawn,
            self.powerup_spawn,
            self.hazard_spawn,
            self.blocker_spawn,
            self.wildcard_spawn,
            self.reward,
            self.kind,
            self.type_data,
        ])
    }
}

/// `MatchThreeProgression.txt` row (GameServer.as PD_* order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgressionRow {
    pub id: i32,
    pub level: i32,
    pub property: i32,
    pub init: i32,
    pub delta: i32,
    pub max: i32,
}

impl ProgressionRow {
    pub const COLUMNS: usize = 6;

    pub fn parse(line: &str) -> Result<Self, HarvestError> {
        let v = parse_caret_exact(line, Self::COLUMNS)?;
        Ok(Self {
            id: v[0],
            level: v[1],
            property: v[2],
            init: v[3],
            delta: v[4],
            max: v[5],
        })
    }

    pub fn to_row(&self) -> String {
        to_rows(&[
            self.id,
            self.level,
            self.property,
            self.init,
            self.delta,
            self.max,
        ])
    }
}

/// `MatchThreeTimerData.txt` row. `TimerData.SetChain` divides the raw columns
/// by ten, so the chain values are stored here as seconds with the raw column
/// reconstructed on write.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimerRow {
    pub id: i32,
    pub timer_kind: i32,
    pub chain_m: f64,
    pub chain_f: f64,
    pub chain_b: f64,
    pub per_tile: i32,
}

impl TimerRow {
    pub const COLUMNS: usize = 6;

    pub fn parse(line: &str) -> Result<Self, HarvestError> {
        let v = parse_caret_exact(line, Self::COLUMNS)?;
        Ok(Self {
            id: v[0],
            timer_kind: v[1],
            chain_m: f64::from(v[2]) / 10.0,
            chain_f: f64::from(v[3]) / 10.0,
            chain_b: f64::from(v[4]) / 10.0,
            per_tile: v[5],
        })
    }

    pub fn to_row(&self) -> String {
        let raw = |v: f64| (v * 10.0).round() as i32;
        to_rows(&[
            self.id,
            self.timer_kind,
            raw(self.chain_m),
            raw(self.chain_f),
            raw(self.chain_b),
            self.per_tile,
        ])
    }
}

/// One `grid^N` block from `MatchThreeGridData.txt`: rows of `aMyGrid` codes
/// (0 empty, 1 regular, 2 any, 3 blocker, 4 dropzone, negative specific
/// values, positive category values).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridTemplate {
    pub id: i32,
    pub rows: Vec<Vec<i32>>,
}

pub fn parse_grid_templates(text: &str) -> Result<Vec<GridTemplate>, HarvestError> {
    let mut templates: Vec<GridTemplate> = Vec::new();
    let mut current: Option<usize> = None;
    for raw_line in text.lines() {
        let line = raw_line.trim_end_matches(['\r', '\n', '\0']);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('^').collect();
        if fields[0].eq_ignore_ascii_case("grid") {
            let id: i32 = fields
                .get(1)
                .and_then(|f| f.trim().parse().ok())
                .ok_or_else(|| HarvestError::Schema(format!("invalid grid header {line:?}")))?;
            if id as usize == templates.len() {
                templates.push(GridTemplate {
                    id,
                    rows: Vec::new(),
                });
                current = Some(templates.len() - 1);
            } else {
                current = None;
            }
            continue;
        }
        if let Some(index) = current {
            if line
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit() || c == '-')
            {
                templates[index].rows.push(parse_caret(line)?);
            } else {
                current = None;
            }
        }
    }
    Ok(templates)
}

// ---------------------------------------------------------------------------
// Result proposal (original quota/score semantics).
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoalProgress {
    pub index: i32,
    pub value: u16,
    pub collected_tiles: u32,
    pub quota: u32,
    pub per_reward: u32,
}

impl GoalProgress {
    /// `floor(collected / per_reward)`, the bundle count the original bins
    /// display.
    pub fn clusters(&self) -> u32 {
        if self.per_reward == 0 {
            0
        } else {
            self.collected_tiles / self.per_reward
        }
    }

    /// `GameLevel.CheckCollectableQuota`: a goal with `quota == 0` is not a
    /// winning condition.
    pub fn complete(&self) -> bool {
        self.quota == 0 || self.collected_tiles >= self.quota
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HarvestResult {
    pub success: bool,
    pub goals: Vec<GoalProgress>,
    pub total_bundles: u32,
}

/// Evaluate a finished board against its goals. Input tuples are
/// `(collected_tiles, quota, per_reward)` in board order; goal values follow
/// the recovered produce order ([`produce_value`], `400 + type`).
pub fn evaluate(goals: &[(u32, u32, u32)]) -> HarvestResult {
    let progress: Vec<GoalProgress> = goals
        .iter()
        .enumerate()
        .map(|(slot, (collected, quota, per_reward))| GoalProgress {
            index: slot as i32,
            value: produce_value(slot).unwrap_or(0),
            collected_tiles: *collected,
            quota: *quota,
            per_reward: *per_reward,
        })
        .collect();
    let total_bundles = progress.iter().map(GoalProgress::clusters).sum();
    HarvestResult {
        success: progress.iter().all(GoalProgress::complete),
        goals: progress,
        total_bundles,
    }
}

// ---------------------------------------------------------------------------
// Reconstruction + shared type-8 engine contract (mining_restore v1).
// ---------------------------------------------------------------------------

/// Reconstructed padded grid width. The original fall loops never visit index 0, so the
/// emitted board keeps row 0 and column 0 empty (see `notes/Research/Harvesting Border
/// Compatibility.md`).
pub const RECONSTRUCTED_BOARD_WIDTH: i32 = 11;
/// Reconstructed padded grid height, matching [`RECONSTRUCTED_BOARD_WIDTH`].
pub const RECONSTRUCTED_BOARD_HEIGHT: i32 = 11;
/// Playable rows in the engine-compatible rectangle; the tenth row of the
/// padded grid (index 10) is the dropzone row the shared engine requires.
pub const RECONSTRUCTED_PLAYABLE_ROWS: i32 = 9;
/// Empty left/top border rows and columns. `ServerGrid.as`/`Grid.as::checkFalls` scan with
/// `while (... > 0)` and `addColumnFall` marks with `while (... > 0)`, so a playable cell at
/// index 0 could never fall; the border keeps index 0 inert.
pub const RECONSTRUCTED_BORDER: i32 = 1;
/// Original `aMyGrid` dropzone code (`core/ServerGrid.buildMap` case 4).
pub const SHAPE_DROPZONE: i32 = 4;
/// (R) Two-minute visible candidate timer; live server timer tables were not shipped.
pub const RECONSTRUCTED_DURATION_SECONDS: u32 = 120;
/// (R) Five visible bundles at five tiles per bundle. The live quota is unknown.
pub const RECONSTRUCTED_QUOTA_TILES: u32 = 25;
/// (R) Local tiles-per-bundle cadence; the server table is unavailable.
pub const RECONSTRUCTED_PER_REWARD_TILES: u32 = 5;
/// (R) Fixed seed for the deterministic 22-chain region-111 acceptance replay.
pub const RECONSTRUCTED_SEED: u32 = 2;
/// (R) Local wildcard rate; the server spawn table is unavailable.
pub const RECONSTRUCTED_WILDCARD_PERCENT: u32 = 5;
/// (R) Reconstructed 600-block spawn rate. The recovered farming goals are
/// selectables, and the client ships no `farmingValue` names in the 600 block,
/// so the reconstructed level keeps the block empty instead of spawning tiles
/// whose basket label would stay unresolved.
pub const RECONSTRUCTED_COLLECTABLE_PERCENT: u32 = 0;
/// (R) Blockers are disabled in this local round candidate.
pub const RECONSTRUCTED_BLOCKER_PERCENT: u32 = 0;
/// (R) Powerups are disabled in this local round candidate.
pub const RECONSTRUCTED_POWERUP_PERCENT: u32 = 0;
/// (R) Item quantity granted per completed cluster. The original reward-set
/// mapping (`SpawnData.mRewardSetId`) lived in unavailable server tables.
pub const RECONSTRUCTED_BUNDLES_PER_CLUSTER: u32 = 1;

pub const RULE_NOTICE: &str = "Recovered: type-8 family 39 envelope and payload names, caret-delimited MatchThree*Data.txt schema, \
chain/minimum-3 rule, tile and score constants from GameData.as, per-region produce sets from locale 94149..94154, the \
farmingValue401..416 code strings and SelectableTile1..16 frames that name and draw every basket, bundle item \
definitions 2875..2890 from the pinned ClientItemDefinitions.json, the farming preload list FarmingAssets.txt and the \
game_farming.gfx movie from the client packs, and the staggered/stepped board drawn by the pinned wiki screenshot \
(diamond_shape). Reconstructed (server tables never shipped): the emitted 10x9 playable rectangle plus one dropzone row inside \
an 11x11 padded grid (row 0 and column 0 stay empty because the original fall loops never visit index 0), duration, per-goal \
quotas, tiles-per-bundle cadence, seed, spawn percentages, and item quantity per bundle cluster. Fidelity \
gap: the shared engine executes only full rectangles with a dropzone row, so the screenshot-observed staggered shape is \
recorded but not launched.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Type8Kind {
    pub value: u16,
    pub category: u16,
    pub icon_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Type8Goal {
    pub index: i32,
    pub value: u16,
    pub quota: u32,
    pub per_reward: u32,
    pub icon_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Type8Output {
    pub value: u16,
    pub definition_id: i32,
    pub cluster_quantity: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Type8BoardSpec {
    pub width: i32,
    pub height: i32,
    pub shape: Vec<i32>,
    pub selectable: Vec<u16>,
    pub collectable: Vec<u16>,
    pub tiles: Vec<Type8Kind>,
    pub goals: Vec<Type8Goal>,
    pub outputs: Vec<Type8Output>,
    pub duration_seconds: u32,
    pub seed: u32,
    pub wildcard_percent: u32,
    pub collectable_percent: u32,
    pub blocker_percent: u32,
    pub powerup_percent: u32,
    pub meter_id: i32,
    pub reconstructed_rules: bool,
    pub rule_notice: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HarvestContent {
    pub schema_version: u32,
    pub game_id: i32,
    pub profile_id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub icon_id: i32,
    /// Preload asset-list name advertised in `39/16`; the recovered farming
    /// list replaces the shared engine's mining default.
    pub preload_file: String,
    /// Movie requested through `26/12 MiniGameFlash`; the recovered farming
    /// movie replaces the shared engine's mining default.
    pub launch_movie: String,
    pub spec: Type8BoardSpec,
}

/// The staggered/stepped playfield drawn by the pinned wiki screenshot: the
/// outer corners of each row stay `0` (empty) and the widths step
/// 4/6/8/10/10/10/10/8/6/4. Kept as the observed-shape artifact only; the
/// shared engine executes full rectangles with a dropzone row, so the emitted
/// spec uses [`engine_rectangle_shape`] instead. This difference is a labelled
/// fidelity gap, not a recovered grid.
pub fn diamond_shape(width: i32, height: i32) -> Vec<i32> {
    let mut shape = Vec::with_capacity((width * height) as usize);
    for row in 0..height {
        let distance = row.min(height - 1 - row);
        let indent = ((height / 2 - 2 - distance).max(0)) * 2;
        let fill = (width - indent).max(0);
        let left = (width - fill) / 2;
        for column in 0..width {
            shape.push(if column >= left && column < left + fill {
                1
            } else {
                0
            });
        }
    }
    shape
}

/// Engine-compatible reconstruction: `playable_rows` full rows of regular
/// tiles followed by one full row of original dropzone codes (`4`). This is
/// the smallest shape the shared type-8 engine accepts for the harvested
/// 10-wide reconstruction.
pub fn engine_rectangle_shape(width: i32, playable_rows: i32) -> Vec<i32> {
    let mut shape = vec![1; (width * playable_rows) as usize];
    shape.extend(vec![SHAPE_DROPZONE; width as usize]);
    shape
}

/// Engine-compatible padded shape: `border` empty rows and columns, the `playable_rows` full
/// rows of regular tiles at x = border.., then one dropzone row over the same columns. This is
/// the smallest border the original `x > 0` / `y > 0` fall loops accept with the reconstructed
/// 10x9 playable area.
pub fn padded_engine_shape(width: i32, height: i32, border: i32, playable_rows: i32) -> Vec<i32> {
    let mut shape = vec![0; (width * height) as usize];
    for row in border..border + playable_rows {
        for column in border..width {
            shape[(row * width + column) as usize] = 1;
        }
    }
    for column in border..width {
        shape[((border + playable_rows) * width + column) as usize] = SHAPE_DROPZONE;
    }
    shape
}

/// Build the harvesting-specific board spec for a MiniGameData game id.
///
/// Regional ids produce four *selectable* produce goals, one per recovered
/// spawn type, mapped to the recovered bundle item definitions. The generic
/// `58` and the unnamed variants carry no recovered produce set, so they
/// refuse to propose a board at all instead of emitting a spec the shared
/// engine must reject.
pub fn board_spec(game_id: i32) -> Result<Type8BoardSpec, HarvestError> {
    let (selectable, goals, outputs, tiles, notice) = match region(game_id) {
        Some(r) => {
            let mut selectable = Vec::with_capacity(4);
            let mut goals = Vec::with_capacity(4);
            let mut outputs = Vec::with_capacity(4);
            let mut tiles = Vec::with_capacity(4);
            for (slot, slug) in r.produce.iter().enumerate() {
                let p = produce(slug).ok_or_else(|| {
                    HarvestError::Content(format!("region {game_id} names unknown produce {slug}"))
                })?;
                // Recovered original: a farming goal is a *selectable* produce tile, so the
                // basket name (`farmingValue<value>`) and the icon (`SelectableTile<type>`)
                // come from the same `400 + type` value. The client's
                // `CodeStringMappings.txt` carries no farmingValue entry in the 600 block,
                // which is why the 601..604 reconstruction rendered `##farmingValue60…`.
                let value = produce_value_for(slug).ok_or_else(|| {
                    HarvestError::Content(format!(
                        "region {game_id} produce {slug} has no recovered farmingValue"
                    ))
                })?;
                selectable.push(value);
                goals.push(Type8Goal {
                    index: slot as i32,
                    value,
                    quota: RECONSTRUCTED_QUOTA_TILES,
                    per_reward: RECONSTRUCTED_PER_REWARD_TILES,
                    // (R) Original SD_ICONID table is absent. This item icon is
                    // metadata only; the movie draws the basket tile from value.
                    icon_id: p.icon_id,
                });
                outputs.push(Type8Output {
                    value,
                    definition_id: p.item_definition_id,
                    cluster_quantity: RECONSTRUCTED_BUNDLES_PER_CLUSTER,
                });
                tiles.push(Type8Kind {
                    value,
                    category: TILE_SELECTABLE_CATEGORY,
                    // The engine emits `OnTileInfoMsg(value, value - category)`, which the
                    // client resolves to the movie's `SelectableTile<type>` frame.
                    icon_id: i32::from(value - TILE_SELECTABLE_CATEGORY),
                });
            }
            (selectable, goals, outputs, tiles, RULE_NOTICE.to_string())
        }
        None => {
            if game_id != GENERIC_HARVESTING_GAME_ID && !UNNAMED_VARIANT_GAME_IDS.contains(&game_id)
            {
                return Err(HarvestError::Content(format!(
                    "unknown harvesting game id {game_id}"
                )));
            }
            return Err(HarvestError::Content(format!(
                "harvesting game {game_id} carries only the generic name/description; its \
                 produce set is unrecovered, so launching it is unsupported"
            )));
        }
    };
    Ok(Type8BoardSpec {
        width: RECONSTRUCTED_BOARD_WIDTH,
        height: RECONSTRUCTED_BOARD_HEIGHT,
        shape: padded_engine_shape(
            RECONSTRUCTED_BOARD_WIDTH,
            RECONSTRUCTED_BOARD_HEIGHT,
            RECONSTRUCTED_BORDER,
            RECONSTRUCTED_PLAYABLE_ROWS,
        ),
        selectable,
        collectable: Vec::new(),
        tiles,
        goals,
        outputs,
        duration_seconds: RECONSTRUCTED_DURATION_SECONDS,
        // (R) Selected by the deterministic region-111 whole-round acceptance replay.
        seed: RECONSTRUCTED_SEED,
        wildcard_percent: RECONSTRUCTED_WILDCARD_PERCENT,
        collectable_percent: RECONSTRUCTED_COLLECTABLE_PERCENT,
        blocker_percent: RECONSTRUCTED_BLOCKER_PERCENT,
        powerup_percent: RECONSTRUCTED_POWERUP_PERCENT,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice: notice,
    })
}

pub fn content(game_id: i32) -> Result<HarvestContent, HarvestError> {
    let r = region(game_id).ok_or_else(|| {
        HarvestError::Content(format!(
            "harvesting game {game_id} carries only the generic name/description; its produce \
             set is unrecovered, so launching it is unsupported"
        ))
    })?;
    let c = HarvestContent {
        schema_version: SCHEMA_VERSION,
        game_id,
        profile_id: CHEF_PROFILE_ID,
        name_id: r.name_id,
        description_id: r.description_id,
        icon_id: r.icon_id,
        preload_file: FARMING_PRELOAD_FILE.to_string(),
        launch_movie: FARMING_LAUNCH_MOVIE.to_string(),
        spec: board_spec(game_id)?,
    };
    validate_content(&c)?;
    Ok(c)
}

pub fn content_json(game_id: i32) -> Result<String, HarvestError> {
    serde_json::to_string_pretty(&content(game_id)?)
        .map_err(|e| HarvestError::Content(e.to_string()))
}

/// Write `<directory>/harvesting.json` for the shared type-8 engine.
pub fn write_content(directory: &Path, game_id: i32) -> Result<PathBuf, HarvestError> {
    let path = directory.join(CONTENT_FILE);
    fs::write(&path, content_json(game_id)?)?;
    Ok(path)
}

/// Narrow guard for the emitted content. Rejects unknown ids, malformed
/// boards, produce sets that disagree with the recovered locale order, and
/// bundle definitions outside the pinned item table.
pub fn validate_content(content: &HarvestContent) -> Result<(), HarvestError> {
    let spec = &content.spec;
    if region(content.game_id).is_none() {
        return if content.game_id == GENERIC_HARVESTING_GAME_ID
            || UNNAMED_VARIANT_GAME_IDS.contains(&content.game_id)
        {
            Err(HarvestError::Content(format!(
                "harvesting game {} has no recovered produce set; launching it is unsupported",
                content.game_id
            )))
        } else {
            Err(HarvestError::Content(format!(
                "unknown harvesting game id {}",
                content.game_id
            )))
        };
    }
    if content.profile_id != CHEF_PROFILE_ID {
        return Err(HarvestError::Content(format!(
            "harvesting must bind the recovered Chef profile {CHEF_PROFILE_ID}, not {}",
            content.profile_id
        )));
    }
    if content.preload_file != FARMING_PRELOAD_FILE || content.launch_movie != FARMING_LAUNCH_MOVIE
    {
        return Err(HarvestError::Content(format!(
            "harvesting must launch the recovered farming assets ({FARMING_PRELOAD_FILE}, \
             {FARMING_LAUNCH_MOVIE})"
        )));
    }
    if spec.width <= 0 || spec.height <= 0 {
        return Err(HarvestError::Content(
            "board dimensions must be positive".into(),
        ));
    }
    if spec.shape.len() != (spec.width * spec.height) as usize {
        return Err(HarvestError::Content(format!(
            "shape length {} does not match {}x{}",
            spec.shape.len(),
            spec.width,
            spec.height
        )));
    }
    if spec.shape.iter().all(|c| *c == 0) {
        return Err(HarvestError::Content("shape has no playable tiles".into()));
    }
    if spec.shape.iter().any(|c| !(-4..=1000).contains(c)) {
        return Err(HarvestError::Content(
            "shape code outside the aMyGrid range".into(),
        ));
    }
    validate_border(&spec.shape, spec.width as usize, spec.height as usize)?;
    let width = spec.width as usize;
    let height = spec.height as usize;
    // Mirror the shared engine's `board_bounds`: the playable rectangle is derived from every
    // non-empty, non-dropzone cell, it must be full, and the drop row sits directly below it.
    // A leading empty border (index 0) is allowed and is exactly what the border fix emits.
    let mut min_x = usize::MAX;
    let mut min_y = usize::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut dropzones = Vec::new();
    for (index, code) in spec.shape.iter().enumerate() {
        let (x, y) = (index % width, index / width);
        match *code {
            0 => {}
            4 => dropzones.push((x, y)),
            _ => {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if min_x == usize::MAX || max_x - min_x + 1 < 6 || max_y - min_y + 1 < 6 {
        return Err(HarvestError::Content(
            "playable area must be an engine-sized rectangle".into(),
        ));
    }
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if spec.shape[y * width + x] == 0 {
                return Err(HarvestError::Content(
                    "playable area must be a full rectangle for the shared engine".into(),
                ));
            }
        }
    }
    let dropzone_row = max_y + 1;
    if dropzone_row >= height
        || (min_x..=max_x).any(|x| spec.shape[dropzone_row * width + x] != SHAPE_DROPZONE)
    {
        return Err(HarvestError::Content(
            "board needs a full dropzone row for the shared engine".into(),
        ));
    }
    if dropzones
        .iter()
        .any(|(x, y)| *y != dropzone_row || *x < min_x || *x > max_x)
    {
        return Err(HarvestError::Content(
            "dropzone tiles must sit in the drop row inside the playable columns".into(),
        ));
    }
    if !spec.reconstructed_rules || spec.rule_notice.trim().is_empty() {
        return Err(HarvestError::Content(
            "reconstructed rules must stay labelled with a notice".into(),
        ));
    }
    validate_tile_kinds(spec)?;
    match region(content.game_id) {
        Some(r) => {
            if spec.goals.len() != r.produce.len() || spec.outputs.len() != r.produce.len() {
                return Err(HarvestError::Content(
                    "regional content needs one goal and one output per produce".into(),
                ));
            }
            for (slot, slug) in r.produce.iter().enumerate() {
                let p = produce(slug)
                    .ok_or_else(|| HarvestError::Content(format!("unknown produce {slug}")))?;
                let value = produce_value_for(slug).ok_or_else(|| {
                    HarvestError::Content(format!(
                        "produce {slug} has no recovered farmingValue in the produce table"
                    ))
                })?;
                let goal = &spec.goals[slot];
                if goal.value != value || goal.quota == 0 || goal.per_reward == 0 {
                    return Err(HarvestError::Content(format!(
                        "goal {slot} must use the recovered produce value {value} \
                         (farmingValue{value}); the 600 block has no recovered name"
                    )));
                }
                if !spec
                    .tiles
                    .iter()
                    .any(|kind| kind.value == value && kind.category == TILE_SELECTABLE_CATEGORY)
                {
                    return Err(HarvestError::Content(format!(
                        "goal {slot} produce value {value} is not registered as a selectable tile; \
                         the client cannot resolve SelectableTile{}",
                        value - TILE_SELECTABLE_CATEGORY
                    )));
                }
                if goal.icon_id != p.icon_id
                    || spec.outputs[slot].definition_id != p.item_definition_id
                {
                    return Err(HarvestError::Content(format!(
                        "goal {slot} icon or bundle definition does not match the pinned item table"
                    )));
                }
                if spec.outputs[slot].value != value || spec.outputs[slot].cluster_quantity == 0 {
                    return Err(HarvestError::Content(format!("output {slot} is malformed")));
                }
            }
            if !spec.collectable.is_empty() {
                return Err(HarvestError::Content(
                    "harvesting must not use the 600-block collectables: the shipped client \
                     names only farmingValue401..416"
                        .into(),
                ));
            }
            if spec.selectable.len() != r.produce.len() {
                return Err(HarvestError::Content(
                    "selectable set size mismatch: one spawn type per recovered produce".into(),
                ));
            }
        }
        // Gap profiles are refused before reaching this point; keep the arm
        // explicit so a future caller cannot validate one into a launch.
        None => {
            return Err(HarvestError::Content(format!(
                "harvesting game {} has no recovered produce set; launching it is unsupported",
                content.game_id
            )));
        }
    }
    if spec.duration_seconds == 0 {
        return Err(HarvestError::Content("duration must be positive".into()));
    }
    Ok(())
}

/// Every value the shared engine can spawn must be registered exactly once in the tile
/// list, with the recovered `floor(value / 100) * 100` category. The original client only
/// learns a value in `core/TileGenerator.as::SetTileInfo(value, index)` and
/// `generate_selectable`/`generate_collectable` dereference `aNames[value]`; the shared
/// engine emits `OnTileInfoMsg<value>\t<value - category>` per list entry, so a missing or
/// duplicated entry either leaves regular tiles undrawable or double-registers one index.
fn validate_tile_kinds(spec: &Type8BoardSpec) -> Result<(), HarvestError> {
    let tiles: Vec<(u16, u16)> = spec
        .tiles
        .iter()
        .map(|kind| (kind.value, kind.category))
        .collect();
    validate_tile_kinds_parts(&spec.selectable, &spec.collectable, &tiles)
}

/// The Harvest gameplay load gate: the engine binds Chef and the farming assets, and every game it
/// offers (T553: the file's own and its `games`) is a recovered region whose values are
/// registered, whose shape clears the border and whose reward items the loaded inventory
/// defines. The shared engine stays permissive so the Mining contract is unchanged; see
/// `notes/Research/Harvesting Runtime Tile Guard.md`.
pub(crate) fn validate_engine(
    engine: &crate::mining::Type8Engine,
    inventory: &crate::inventory::Inventory,
) -> Result<(), String> {
    if engine.profile_id() != CHEF_PROFILE_ID {
        return Err("harvesting content does not bind the Chef job".into());
    }
    if engine.preload_file() != FARMING_PRELOAD_FILE
        || engine.launch_movie() != FARMING_LAUNCH_MOVIE
    {
        return Err("harvesting content does not bind the farming assets".into());
    }
    for (game, spec) in engine.games() {
        if region(game).is_none() {
            return Err(format!("harvesting content offers unknown game {game}"));
        }
        let tiles: Vec<(u16, u16)> = spec
            .tiles
            .iter()
            .map(|kind| (kind.value, kind.category))
            .collect();
        validate_tile_kinds_parts(&spec.selectable, &spec.collectable, &tiles)
            .map_err(|error| format!("harvesting tile registration: {error}"))?;
        // A playable cell on shape row 0 or column 0 could never fall under the original
        // loops, so an unpadded persisted document is refused before any offer.
        validate_border(&spec.shape, spec.width, spec.height)
            .map_err(|error| format!("harvesting border gate: {error}"))?;
        for output in &spec.outputs {
            if !inventory.has_definition(output.definition_id) {
                return Err(format!(
                    "harvesting reward definition {} is not in the loaded inventory",
                    output.definition_id
                ));
            }
        }
    }
    Ok(())
}

/// Slice form of the registration check, so the optional Harvest gameplay load gate can
/// validate the shared engine's parsed spec without a second parser.
pub(crate) fn validate_tile_kinds_parts(
    selectable: &[u16],
    collectable: &[u16],
    tiles: &[(u16, u16)],
) -> Result<(), HarvestError> {
    let mut registered: Vec<u16> = Vec::with_capacity(tiles.len());
    for (value, category) in tiles {
        let implied = (value / 100) * 100;
        if *category != implied {
            return Err(HarvestError::Content(format!(
                "tile kind {} carries category {} but its value implies {implied}",
                value, category
            )));
        }
        if registered.contains(value) {
            return Err(HarvestError::Content(format!(
                "duplicate tile kind for value {}",
                value
            )));
        }
        registered.push(*value);
    }
    for value in selectable.iter().chain(collectable.iter()) {
        if !registered.contains(value) {
            return Err(HarvestError::Content(format!(
                "value {value} has no registered tile kind; the client dereferences \
                 aNames[value] and cannot draw it"
            )));
        }
    }
    for (value, _) in tiles {
        if !selectable.contains(value) && !collectable.contains(value) {
            return Err(HarvestError::Content(format!(
                "tile kind {} is neither selectable nor collectable",
                value
            )));
        }
    }
    Ok(())
}

/// Refuse a Harvest shape that exposes a playable cell on shape row 0 or column 0. The original
/// `ServerGrid.as`/`Grid.as::checkFalls` loops scan `while (... > 0)` and `addColumnFall` marks
/// rows `while (... > 0)`, so a tile on index 0 can never fall; only empty (`0`) borders and the
/// dropzone code (`4`) are accepted on those axes. The bounds themselves stay private in the
/// shared engine, so this inspects the spec's own edge codes (the pre-border 10x10 export put
/// `1`s there and is refused).
pub(crate) fn validate_border(
    shape: &[i32],
    width: usize,
    height: usize,
) -> Result<(), HarvestError> {
    if width == 0 || height == 0 || shape.len() != width * height {
        return Err(HarvestError::Content(
            "shape is not a width*height grid".into(),
        ));
    }
    for (index, code) in shape.iter().enumerate() {
        let (x, y) = (index % width, index / width);
        if (x == 0 || y == 0) && *code != 0 && *code != SHAPE_DROPZONE {
            return Err(HarvestError::Content(format!(
                "playable shape code {code} sits on the index-0 border (x {x}, y {y}); the \
                 original fall loops never move row 0 or column 0"
            )));
        }
    }
    Ok(())
}
