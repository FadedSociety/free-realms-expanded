//! Work item 013: recovered Smelting77 furnace definitions, materials and payload surface.
//!
//! Independent module inclusion allows focused proof before the lead registers
//! the module in lib.rs. The private prepared-content check is opt-in because
//! the lab corpus is not part of this repository.
#[path = "../src/restoration/piece_013.rs"]
mod piece_013;

use piece_013::{
    CLIENT_TO_SERVER_MESSAGES, FURNACE, GAME_ID, GAME_WON_QUANTITY_LITERAL, HEAT_FURNACE, ORES,
    Origin, PROFILE_ID, SERVER_TO_CLIENT_HANDLERS, STAT_GAME_QUANTITY, check_content,
    check_payload_surface,
};
use std::{fs, path::PathBuf};

fn prepared(phases: serde_json::Value) -> String {
    serde_json::json!({
        "schema_version": 1,
        "game_id": 77,
        "profile_id": 14,
        "phases": phases,
    })
    .to_string()
}

fn recovered_phases() -> serde_json::Value {
    serde_json::json!([
        {"id": 1, "sequence": 1, "duration": 90, "stage_type": 101,
         "data": [1, 5, 1, 1, 0, 20, 2, 3, 50, 100, 200]},
        {"id": 2, "sequence": 2, "duration": 90, "stage_type": 102,
         "data": [65, 8, 3, 100, 100, 0, 0, 0, 0, 0, 0]},
        {"id": 3, "sequence": 3, "duration": 90, "stage_type": 103,
         "data": [100, 1, 500, 0, 0, 0, 0, 0, 0, 0, 0]},
        {"id": 4, "sequence": 4, "duration": 90, "stage_type": 104,
         "data": [2, 500, 0, 0, 0, 0, 0, 0, 0, 0, 0]}
    ])
}

#[test]
fn recovered_movie_tables_are_present() {
    assert_eq!(GAME_ID, 77);
    assert_eq!(PROFILE_ID, 14);
    assert_eq!(CLIENT_TO_SERVER_MESSAGES.len(), 15);
    assert_eq!(SERVER_TO_CLIENT_HANDLERS.len(), 8);
    let copper = ORES.iter().find(|ore| ore.id == 1).unwrap();
    assert_eq!(copper.name, "Copper");
    assert_eq!(copper.measure_percent, 65);
    assert_eq!(copper.break_clicks, [1, 1, 1]);
    assert_eq!(ORES.len(), 5);
    assert_eq!(ORES[4].name, "Gold");
    assert_eq!(ORES[4].measure_percent, 85);
    assert_eq!(FURNACE.measure_precision, 8);
    assert_eq!(FURNACE.measure_perfect_precision, 3);
    assert_eq!(FURNACE.max_num_ore, 5);
    assert_eq!(FURNACE.spill_data, 2.5);
    assert_eq!(HEAT_FURNACE.gain_max, 30.0);
    assert_eq!(HEAT_FURNACE.loss_rate, 0.05);
    assert_eq!(GAME_WON_QUANTITY_LITERAL, 10);
    assert_eq!(STAT_GAME_QUANTITY, 8);
}

#[test]
fn prepared_content_classifies_recovered_and_reconstructed_cells() {
    let stages = check_content(&prepared(recovered_phases())).unwrap();
    assert_eq!(stages.len(), 4);
    assert_eq!(stages[0].stage, "breakDownOre");
    assert_eq!(stages[0].ore().unwrap().name, "Copper");
    assert_eq!(stages[0].cells[0], Origin::Original);
    assert_eq!(stages[0].cells[1], Origin::Reconstructed);
    assert_eq!(stages[0].cells[2], Origin::Original);
    assert_eq!(stages[0].cells[4], Origin::Original);
    assert_eq!(stages[1].cells[0], Origin::Original);
    assert_eq!(stages[1].cells[1], Origin::Original);
    assert_eq!(stages[1].cells[2], Origin::Original);
    assert_eq!(stages[1].cells[3], Origin::Reconstructed);
    assert_eq!(stages[1].cells[5], Origin::Unused);
    assert_eq!(stages[2].cells[0], Origin::Reconstructed);
    assert_eq!(stages[2].cells[2], Origin::Unused);
    assert_eq!(stages[3].cells[1], Origin::Reconstructed);
}

#[test]
fn contradictory_rows_are_rejected() {
    let cases: [(&str, usize, usize, i32); 6] = [
        ("unknown movie ore", 0, 0, 7),
        ("measure outside the movie range", 1, 0, 0),
        ("perfect precision wider than good", 1, 1, 1),
        ("non-positive heat goal", 2, 0, 0),
        ("non-positive spill", 3, 0, 0),
        (
            "hits outside the clip range is a warning not an error",
            0,
            6,
            9,
        ),
    ];
    for (label, stage, cell, value) in cases {
        let mut phases = recovered_phases();
        phases[stage]["data"][cell] = serde_json::json!(value);
        let result = check_content(&prepared(phases.clone()));
        if label.starts_with("hits") {
            let stages = result.unwrap();
            assert_eq!(stages[0].cells[6], Origin::Clamped, "{label}");
        } else {
            let error = result.unwrap_err();
            assert!(!error.is_empty(), "{label}");
        }
    }
    let mut phases = recovered_phases();
    phases[1]["data"][1] = serde_json::json!(1);
    assert!(
        check_content(&prepared(phases))
            .unwrap_err()
            .contains("precision")
    );
}

#[test]
fn wrong_phase_count_or_order_is_rejected() {
    let mut phases = recovered_phases();
    phases.as_array_mut().unwrap().pop();
    assert!(check_content(&prepared(phases)).is_err());
    let mut phases = recovered_phases();
    phases[2]["stage_type"] = serde_json::json!(102);
    assert!(
        check_content(&prepared(phases))
            .unwrap_err()
            .contains("stage_type")
    );
    let mut phases = recovered_phases();
    phases[0]["data"] = serde_json::json!([1, 5, 1]);
    assert!(check_content(&prepared(phases)).unwrap_err().contains("11"));
}

#[test]
fn payload_surface_rejects_messages_the_movie_never_sends() {
    let handled = [
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
        "FRServer_GameLost",
        "FRServer_GameQuit",
        "FRServer_GameOver",
        "FRServer_GameClose",
    ];
    check_payload_surface(&handled).unwrap();
    let error = check_payload_surface(&["FRServer_InventedRecipe"]).unwrap_err();
    assert!(error.contains("FRServer_InventedRecipe"));
}

#[test]
#[ignore = "requires the opt-in private prepared Smelting77 content, no client launch"]
fn private_prepared_content_is_conformant() {
    let path = std::env::var_os("OSFR_PIECE_013_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening/smelting.json")
        });
    let stages = check_content(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(stages[0].ore().unwrap().name, "Copper");
    assert_eq!(stages[1].cells[0], Origin::Original);
    assert_eq!(stages[2].cells[0], Origin::Reconstructed);
}
