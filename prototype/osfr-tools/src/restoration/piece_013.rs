//! Recovered Smelting77 furnace definitions, materials and payload surface.
//!
//! Game 77 (Miner profile 14). The constants below were recovered from the
//! original `game_smelting.gfx` actionscript export and the supplied client
//! resources; nothing here is invented balance:
//!
//! - `Smelting/SmeltingData.as` holds the five metal ores, their break clicks,
//!   powder/liquid thickness tables, 50ml-per-ore mix, color transforms and the
//!   measure targets (Copper 0.65 .. Gold 0.85), plus the furnace scalars
//!   (door close 0.3, required metal 0.7, mix 150, hammers 100/130, measure
//!   precision 8/3, spill 2.5, five-ore maximum).
//! - `Smelting/SmeltingGameServer.as` carries the 19-column level-data order
//!   and the per-stage meaning of data1..data11.
//! - `SpecialRounds/HeatFurnace/Data.as` carries the heat limits and angles.
//! - `Smelting/SmeltingGameClient.as:303` sends `SendGameWon(this.nTotalScore,10)`;
//!   the quantity field is a literal, while the produced ingot count is the
//!   separate `nQuantity = int(mold performance / 20)` reported as
//!   `STAT_GAME_QUANTITY` (kind 8). Server-side reward values stay reconstructed.
//!
//! `classify_phases` is the enforcement side: the lead can register this module
//! and reject prepared content whose rows contradict the recovered movie
//! semantics (unknown ore, unreachable measure target, wider perfect-measure
//! window than the good one, non-positive heat or spill).

use serde::Deserialize;

pub const GAME_ID: i32 = 77;
pub const PROFILE_ID: i32 = 14;

/// `SmeltingGameClient.as` `onWin`: `SendGameWon(this.nTotalScore,10)`.
pub const GAME_WON_QUANTITY_LITERAL: u32 = 10;
/// `GameData.as` `STAT_GAME_QUANTITY`, reported from the mold round.
pub const STAT_GAME_QUANTITY: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ore {
    pub id: i32,
    pub name: &'static str,
    /// `aBREAK_CLICKS` A/B/C hammer depth counts.
    pub break_clicks: [i32; 3],
    /// `aORE_MEASURES` target fill height as a percentage.
    pub measure_percent: i32,
    /// `aML_PER_ORE`, identical for every recovered metal.
    pub ml_per_ore: i32,
}

/// `aBREAK_CLICKS` and `aORE_MEASURES` from `Smelting/SmeltingData.as`.
pub const ORES: [Ore; 5] = [
    Ore {
        id: 1,
        name: "Copper",
        break_clicks: [1, 1, 1],
        measure_percent: 65,
        ml_per_ore: 50,
    },
    Ore {
        id: 2,
        name: "Tin",
        break_clicks: [1, 1, 1],
        measure_percent: 70,
        ml_per_ore: 50,
    },
    Ore {
        id: 3,
        name: "Iron",
        break_clicks: [2, 1, 1],
        measure_percent: 75,
        ml_per_ore: 50,
    },
    Ore {
        id: 4,
        name: "Silver",
        break_clicks: [2, 2, 1],
        measure_percent: 80,
        ml_per_ore: 50,
    },
    Ore {
        id: 5,
        name: "Gold",
        break_clicks: [3, 2, 1],
        measure_percent: 85,
        ml_per_ore: 50,
    },
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Furnace {
    pub door_close_time: f32,
    pub required_metal: f32,
    pub mix_quantity: i32,
    pub hammer_normal_size: i32,
    pub hammer_big_size: i32,
    pub measure_precision: i32,
    pub measure_perfect_precision: i32,
    pub spill_data: f32,
    pub max_num_ore: i32,
}

pub const FURNACE: Furnace = Furnace {
    door_close_time: 0.3,
    required_metal: 0.7,
    mix_quantity: 150,
    hammer_normal_size: 100,
    hammer_big_size: 130,
    measure_precision: 8,
    measure_perfect_precision: 3,
    spill_data: 2.5,
    max_num_ore: 5,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatFurnace {
    pub gain_max: f32,
    pub loss_rate: f32,
    pub player_start_angle: i32,
    pub player_end_angle: i32,
    pub player_start2_angle: i32,
    pub blower_start_angle: i32,
    pub blower_end_angle: i32,
}

pub const HEAT_FURNACE: HeatFurnace = HeatFurnace {
    gain_max: 30.0,
    loss_rate: 0.05,
    player_start_angle: 65,
    player_end_angle: 90,
    player_start2_angle: 115,
    blower_start_angle: 115,
    blower_end_angle: 90,
};

/// Messages the original movie sends to the server (GameExternalNetwork.as and
/// the SoeInterfaceFreeRealms.as fscommand bridge).
pub const CLIENT_TO_SERVER_MESSAGES: [&str; 15] = [
    "FRServer_Connect",
    "FRServer_ReadyForGameData",
    "FRServer_StatUpdate",
    "FRServer_Pause",
    "FRServer_RoundTransitionPause",
    "OnStartGameMsg",
    "FRServer_CompleteLevel",
    "FRServer_StopLevel",
    "FRServer_GameWon",
    "FRServer_ScoreSet",
    "FRServer_ScoreIncrement",
    "FRServer_GameLost",
    "FRServer_GameQuit",
    "FRServer_GameOver",
    "FRServer_GameClose",
];

/// Reply handlers the original movie registers for server messages.
pub const SERVER_TO_CLIENT_HANDLERS: [&str; 8] = [
    "OnGameDataMsg",
    "OnPhaseDataMsg",
    "OnCustomDataMsg",
    "OnDataComplete",
    "OnGameWonMsg",
    "OnGameLostMsg",
    "OnServerAddScoreMsg",
    "OnSkipStageOrLevelMsg",
];

/// Where a prepared phase cell comes from, relative to the recovered movie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Equal to a recovered movie constant.
    Original,
    /// Row override of a recovered default; the movie honours the row.
    MovieDefault,
    /// No original server phase table was recovered for this cell.
    Reconstructed,
    /// No recovered client reader consumes this cell.
    Unused,
    /// The movie would clamp this value before use.
    Clamped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageReport {
    pub stage_type: i32,
    pub stage: &'static str,
    pub ore_index: Option<usize>,
    pub cells: [Origin; 11],
}

impl StageReport {
    pub fn ore(&self) -> Option<&'static Ore> {
        self.ore_index.map(|index| &ORES[index])
    }
}

/// One `OnPhaseDataMsg` row as prepared content stores it: stage type plus the
/// eleven integer data columns parsed by `SmeltingGameServer.OnLevelData`.
#[derive(Clone, Debug, Deserialize)]
pub struct PhaseRow {
    pub stage_type: i32,
    pub data: [i32; 11],
}

#[derive(Deserialize)]
struct PreparedContent {
    phases: Vec<PhaseRow>,
}

const STAGE_TYPES: [(i32, &str); 4] = [
    (101, "breakDownOre"),
    (102, "fillFurnace"),
    (103, "heatFurnace"),
    (104, "moldMetal"),
];

fn ore_index(ore_id: i32) -> Option<usize> {
    ORES.iter().position(|ore| ore.id == ore_id)
}

fn clip(value: i32, minimum: i32, maximum: i32) -> i32 {
    value.max(minimum).min(maximum)
}

/// Classify a complete prepared phase table against the recovered movie.
///
/// Returns one report per stage, ordered `breakDownOre`, `fillFurnace`,
/// `heatFurnace`, `moldMetal`. Contradictions with the recovered movie are
/// errors; values the movie would clamp are reported as `Origin::Clamped`.
pub fn classify_phases(phases: &[PhaseRow]) -> Result<Vec<StageReport>, String> {
    if phases.len() != STAGE_TYPES.len() {
        return Err(format!(
            "expected {} phase rows, found {}",
            STAGE_TYPES.len(),
            phases.len()
        ));
    }
    let mut reports: Vec<StageReport> = Vec::with_capacity(phases.len());
    for (position, row) in phases.iter().enumerate() {
        let expected = STAGE_TYPES[position].0;
        if row.stage_type != expected {
            return Err(format!(
                "phase {} has stage_type {}, expected {}",
                position + 1,
                row.stage_type,
                expected
            ));
        }
        let data = row.data;
        let mut report = StageReport {
            stage_type: row.stage_type,
            stage: STAGE_TYPES[position].1,
            ore_index: None,
            cells: [Origin::Unused; 11],
        };
        match row.stage_type {
            101 => {
                let index = ore_index(data[0]).ok_or_else(|| {
                    format!(
                        "stage 101 ore id {} is not in the recovered movie ore table 1-5",
                        data[0]
                    )
                })?;
                report.ore_index = Some(index);
                let ore = &ORES[index];
                report.cells[0] = Origin::Original;
                report.cells[1] = Origin::Reconstructed; // performance = min(100, v*20)
                for (cell, key) in [(2, 0), (3, 1)] {
                    report.cells[cell] = if data[cell] == ore.break_clicks[key] {
                        Origin::Original
                    } else if (1..=5).contains(&data[cell]) {
                        Origin::MovieDefault
                    } else {
                        Origin::Clamped
                    };
                }
                report.cells[4] = if data[4] == 0 {
                    Origin::Original
                } else if (0..=300).contains(&data[4]) {
                    Origin::Reconstructed
                } else {
                    Origin::Clamped
                };
                report.cells[5] = if (10..=100).contains(&data[5].abs()) {
                    Origin::Reconstructed
                } else {
                    Origin::Clamped
                };
                for cell in 6..=7 {
                    report.cells[cell] = if (1..=5).contains(&data[cell]) {
                        Origin::Reconstructed
                    } else {
                        Origin::Clamped
                    };
                }
                let mut previous = None;
                for cell in 8..=10 {
                    let minimum = previous.map_or(50, |score: i32| 2 * score);
                    let effective = clip(data[cell], minimum, 10_000);
                    report.cells[cell] = if effective == data[cell] {
                        Origin::Reconstructed
                    } else {
                        Origin::Clamped
                    };
                    previous = Some(effective);
                }
            }
            102 => {
                let ore = reports
                    .iter()
                    .rev()
                    .find(|report| report.ore_index.is_some())
                    .and_then(StageReport::ore);
                if !(1..=100).contains(&data[0]) {
                    return Err(format!(
                        "stage 102 measure {} percent is outside the movie's accepted 1-100 range",
                        data[0]
                    ));
                }
                report.cells[0] = if ore.is_some_and(|ore| ore.measure_percent == data[0]) {
                    Origin::Original
                } else {
                    Origin::Reconstructed
                };
                report.cells[1] = if data[1] == FURNACE.measure_precision {
                    Origin::Original
                } else {
                    Origin::Reconstructed
                };
                report.cells[2] = if data[2] == FURNACE.measure_perfect_precision {
                    Origin::Original
                } else {
                    Origin::Reconstructed
                };
                if data[1] <= 0 || data[2] > data[1] {
                    return Err(format!(
                        "stage 102 measure precision {} is below perfect precision {}",
                        data[1], data[2]
                    ));
                }
                report.cells[3] = if (100..=50_000).contains(&data[3]) {
                    Origin::Reconstructed
                } else {
                    Origin::Clamped
                };
                report.cells[4] = if data[3] <= data[4] && data[4] <= 50_000 {
                    Origin::Reconstructed
                } else {
                    Origin::Clamped
                };
            }
            103 => {
                if data[0] <= 0 {
                    return Err(format!("stage 103 heat goal {} is not positive", data[0]));
                }
                if data[1] < 0 {
                    return Err(format!(
                        "stage 103 heat loss multiplier {} is negative",
                        data[1]
                    ));
                }
                report.cells[0] = Origin::Reconstructed;
                report.cells[1] = Origin::Reconstructed;
            }
            104 => {
                if data[0] <= 0 {
                    return Err(format!("stage 104 spill {} is not positive", data[0]));
                }
                report.cells[0] = Origin::Reconstructed;
                report.cells[1] = if (500..=50_000).contains(&data[1]) {
                    Origin::Reconstructed
                } else {
                    Origin::Clamped
                };
            }
            _ => unreachable!("stage list is fixed"),
        }
        reports.push(report);
    }
    Ok(reports)
}

/// Parse prepared content JSON (`phases[].stage_type/data`) and classify it.
pub fn check_content(json: &str) -> Result<Vec<StageReport>, String> {
    let content: PreparedContent =
        serde_json::from_str(json).map_err(|error| format!("invalid smelting content: {error}"))?;
    classify_phases(&content.phases)
}

/// Every message the Rust layer acts on must be one the original movie sends.
pub fn check_payload_surface(handled: &[&str]) -> Result<(), String> {
    for message in handled {
        if !CLIENT_TO_SERVER_MESSAGES.contains(message) {
            return Err(format!(
                "{message} is not part of the recovered Smelting77 movie payload surface"
            ));
        }
    }
    Ok(())
}
