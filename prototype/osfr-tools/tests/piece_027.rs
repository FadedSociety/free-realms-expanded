//! Work item 027 acceptance: housing fixture snap, rotation, build bounds and instance limits.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/working/OSFR.Client/OSFR Client/UI/ScriptsBase.bin`, decompiled to
//!   `lab/decompiled/client-ui/ScriptsBase.lua` — the client Lua. `Housing.GRID_SIZE_ON = 25`,
//!   `Housing.GRID_SIZE_OFF = 5`, `Housing:ToggleGrid()`, `Housing:SetGrid()` reading the saved user
//!   option `Housing.gridSpacing` (default `GRID_SIZE_ON`) and calling
//!   `House.SetGridSpacingInCentimeters(gridSpacing)`; `Housing:MakeHousingRadialMenu()` offers
//!   `House.RotateFixtureClockwise(guid)` and `House.RotateFixtureCounterClockwise(guid)`;
//!   `Housing:SetData()` forwards `HousingInstance_DS.CurFixtureCount/MaxFixtureCount/
//!   CurLandmarkCount/MaxLandmarkCount` to the edit panel `SetCounts`.
//! * `lab/reports/piece-027-20261009T025107Z/housing-snap-export.txt` — the client Lua binding table
//!   maps those names to `RotateFixtureClockwise` = `FUN_00c0c0f0`, `RotateFixtureCounterClockwise` =
//!   `FUN_00c0c0a0`, `SetGridSpacingInCentimeters` = `FUN_00c0ba80`, `GetUseAutoOrientation` =
//!   `FUN_00c0c310`, `SetUseAutoOrientation` = `FUN_00c0b510`.
//! * `lab/reports/piece-027-20261009T025107Z/housing-action-decompile.txt` — `FUN_00ab1e30` rotates by
//!   `DAT_017f1914` (0.2617994 rad = 15 degrees) and negates it for the clockwise action id 7;
//!   `DAT_017ed818` (1.5707964 rad = 90 degrees) is the other angle in the same function.
//! * `lab/reports/piece-027-20261009T025107Z/housing-snap-constant.txt` — `_DAT_0175b420` = 1.0,
//!   `DAT_017ec67c` = 100.0 (`housing-constants.txt`). `FUN_00c0ba80` stores `100 / centimeters` and
//!   `FUN_00aab720` snaps with `(int)(value * step) * (1.0 / step)`.
//! * `lab/references/Sanctuary/src/Resources/Houses.json` (sha256
//!   `854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`) — 31 community property
//!   rows with `BuildAreas` min/max triples; this is community content, not client data.
//! * `lab/references/Sanctuary/src/Sanctuary.Packet.Common/Housing/PlayerHousingInstanceData.cs` —
//!   the max/current fixture and landmark counts and `BuildAreas` arrive from the server.
//!
//! Nothing here claims collision, physics or native acceptance. Collision semantics stay unrecovered.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_027;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_027.rs"]
#[allow(dead_code)]
mod piece_027;

use piece_027::{
    BuildBounds, CENTIMETERS_PER_WORLD_UNIT, FixtureLimitsError, GRID_SPACING_OFF_CM,
    GRID_SPACING_ON_CM, GridSnap, HousingFixtureLimits, PlacementDenial, QUARTER_TURN_RADIANS,
    ROTATION_STEP_RADIANS, RotationDirection,
};

const TOLERANCE: f32 = 1e-5;

#[test]
fn grid_pair_is_the_recovered_25_and_5_centimeters() {
    assert_eq!(GRID_SPACING_ON_CM, 25);
    assert_eq!(GRID_SPACING_OFF_CM, 5);
    assert_eq!(CENTIMETERS_PER_WORLD_UNIT, 100.0);
    assert_eq!(GridSnap::on().spacing_cm(), 25);
    assert_eq!(GridSnap::off().spacing_cm(), 5);
}

#[test]
fn grid_divisions_match_the_recovered_100_over_centimeters_formula() {
    assert!((GridSnap::on().divisions_per_world_unit() - 4.0).abs() < TOLERANCE);
    assert!((GridSnap::off().divisions_per_world_unit() - 20.0).abs() < TOLERANCE);
}

#[test]
fn toggle_switches_between_the_recovered_pair() {
    assert_eq!(GridSnap::on().toggled(), GridSnap::off());
    assert_eq!(GridSnap::off().toggled(), GridSnap::on());
}

#[test]
fn snap_quantises_x_and_z_and_leaves_y_alone() {
    let snapped = GridSnap::on().snap_position([1.13, 1.13, -0.37]);
    assert!((snapped[0] - 1.0).abs() < TOLERANCE, "{snapped:?}");
    assert_eq!(snapped[1], 1.13);
    assert!((snapped[2] - -0.25).abs() < TOLERANCE, "{snapped:?}");
}

#[test]
fn snap_truncates_toward_zero_and_keeps_exact_multiples() {
    let grid = GridSnap::on();
    // Truncation toward zero, as in the recovered `(int)` cast: -4.52 -> -4, not -5.
    assert!((grid.snap_axis(-1.13) - -1.0).abs() < TOLERANCE);
    assert!((grid.snap_axis(1.25) - 1.25).abs() < TOLERANCE);
    // The fine grid is 20 divisions per world unit.
    assert!((GridSnap::off().snap_axis(0.37) - 0.35).abs() < TOLERANCE);
}

#[test]
fn snap_rejects_a_non_positive_spacing() {
    assert_eq!(
        GridSnap::new(0),
        Err(FixtureLimitsError::NonPositiveGridSpacing)
    );
}

#[test]
fn rotation_step_is_fifteen_degrees_and_signed_by_direction() {
    assert!((ROTATION_STEP_RADIANS - 0.261_799_4).abs() < TOLERANCE);
    assert!(
        (RotationDirection::Clockwise.signed_step() - -ROTATION_STEP_RADIANS).abs() < TOLERANCE
    );
    assert!(
        (RotationDirection::CounterClockwise.signed_step() - ROTATION_STEP_RADIANS).abs()
            < TOLERANCE
    );
}

#[test]
fn quarter_turn_constant_matches_the_same_function() {
    assert!((QUARTER_TURN_RADIANS - 1.570_796_4).abs() < TOLERANCE);
}

#[test]
fn bounds_validate_rejects_inverted_or_non_finite_boxes() {
    let inverted = BuildBounds {
        min: [0.0, 0.0, 0.0],
        max: [-1.0, 1.0, 1.0],
    };
    assert_eq!(
        inverted.validate(),
        Err(FixtureLimitsError::InvalidBuildBounds)
    );
    let non_finite = BuildBounds {
        min: [f32::NAN, 0.0, 0.0],
        max: [1.0, 1.0, 1.0],
    };
    assert_eq!(
        non_finite.validate(),
        Err(FixtureLimitsError::NonFiniteValue)
    );
}

#[test]
fn bounds_contain_their_closed_edges() {
    let bounds = BuildBounds {
        min: [322.0, -100.0, 392.1],
        max: [443.3, 60.0, 477.8],
    };
    assert!(bounds.validate().is_ok());
    assert!(bounds.contains([322.0, -100.0, 392.1]));
    assert!(bounds.contains([443.3, 60.0, 477.8]));
    assert!(bounds.contains([400.0, 0.0, 400.0]));
    assert!(!bounds.contains([443.4, 0.0, 400.0]));
}

#[test]
fn limits_follow_the_server_supplied_maximum_counts() {
    let limits = HousingFixtureLimits {
        max_fixtures: 3,
        max_landmarks: 1,
    };
    assert!(limits.allows_fixture(2));
    assert!(!limits.allows_fixture(3));
    assert_eq!(
        limits.denial(3, 0),
        Some(PlacementDenial::FixtureLimitReached)
    );
    assert_eq!(
        limits.denial(0, 1),
        Some(PlacementDenial::LandmarkLimitReached)
    );
    assert_eq!(limits.denial(2, 0), None);
}

#[test]
fn a_zero_maximum_allows_nothing_without_inventing_a_constant() {
    let limits = HousingFixtureLimits {
        max_fixtures: 0,
        max_landmarks: 0,
    };
    assert_eq!(
        limits.denial(0, 0),
        Some(PlacementDenial::FixtureLimitReached)
    );
}
