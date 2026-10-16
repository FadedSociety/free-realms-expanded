//! Housing fixture snap, rotation, build bounds and instance limits (work item 027).
//!
//! ORIGINAL facts implemented here, with the recovered source of each:
//! * the housing snap grid is a two-value toggle: 25 cm on, 5 cm off
//!   (`lab/decompiled/client-ui/ScriptsBase.lua`, `Housing.GRID_SIZE_ON/GRID_SIZE_OFF`, default user
//!   option `Housing.gridSpacing` = 25);
//! * the client converts centimetres to world units with `100.0 / centimetres`
//!   (`FUN_00c0ba80`, constant `DAT_017ec67c` = 100.0) and snaps only the horizontal axes with
//!   `(int)(value * step) * (1.0 / step)` (`FUN_00aab720`, constant `_DAT_0175b420` = 1.0);
//! * a rotate click is 15 degrees (`DAT_017f1914` = 0.2617994 rad) and the clockwise action negates
//!   it (`FUN_00ab1e30`, action id 7); the counter-clockwise action id is 6
//!   (`FUN_00c0c0a0`). The same function carries a 90 degree constant (`DAT_017ed818` = 1.5707964);
//! * the radial menu exposes `House.RotateFixtureClockwise/CounterClockwise`, and the options panel
//!   exposes an auto-orientation toggle backed by `House.Get/SetUseAutoOrientation`
//!   (`ScriptsBase.lua` and the decompiled `HousingOptions` SWF);
//! * a lot's fixture and landmark maxima arrive from the server (`PlayerHousingInstanceData`
//!   `MaxFixtureCount` / `MaxLandmarkCount`) and the edit panel displays them through `SetCounts`
//!   (client SWF `housingEditPanel`). `HousingMaxFixtures` is the only recovered denial string.
//!
//! RECONSTRUCTED here: nothing numeric. The only local rules are the arithmetic of the recovered
//! formulas and an inclusive closed-box containment test. No ceiling, radius or damage-style value is
//! invented.
//!
//! Explicit gaps, do not imply these:
//! * collision, floor/surface detection and vertical bounds remain unrecovered. `snap_position`
//!   deliberately leaves y untouched because the recovered snap only quantised x and z;
//! * the fixture flag at `+0x128` that selects the 90 degree constant and skips the 15 degree path is
//!   not identified, so `QUARTER_TURN_RADIANS` is exported as evidence and never applied
//!   automatically;
//! * no numeric fixture or landmark maximum is recovered. Limits are carried, not chosen;
//! * the landmark denial text is unknown; only the fixture denial string is recorded.

/// ORIGINAL: `Housing.GRID_SIZE_ON` in the client Lua.
pub const GRID_SPACING_ON_CM: u32 = 25;
/// ORIGINAL: `Housing.GRID_SIZE_OFF` in the client Lua.
pub const GRID_SPACING_OFF_CM: u32 = 5;
/// ORIGINAL: `DAT_017ec67c` = 100.0, the denominator `FUN_00c0ba80` divides by the centimetres.
pub const CENTIMETERS_PER_WORLD_UNIT: f32 = 100.0;
/// ORIGINAL: `DAT_017f1914` = 0.2617994 rad, the rotate step applied by `FUN_00ab1e30`.
pub const ROTATION_STEP_RADIANS: f32 = 0.261_799_4;
/// ORIGINAL: `DAT_017ed818` = 1.5707964 rad, the other angle in `FUN_00ab1e30`. The condition that
/// selects it is unrecovered, so nothing applies it automatically.
pub const QUARTER_TURN_RADIANS: f32 = 1.570_796_4;
/// ORIGINAL: the recovered fixture-limit denial string. The landmark denial text is unknown.
pub const FIXTURE_LIMIT_CLIENT_STRING: &str = "HousingMaxFixtures";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FixtureLimitsError {
    NonPositiveGridSpacing,
    NonFiniteValue,
    InvalidBuildBounds,
}

impl std::fmt::Display for FixtureLimitsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::NonPositiveGridSpacing => "grid spacing must be a positive number of centimetres",
            Self::NonFiniteValue => "value is not finite",
            Self::InvalidBuildBounds => "build bounds are inverted",
        };
        formatter.write_str(text)
    }
}

impl std::error::Error for FixtureLimitsError {}

/// ORIGINAL: the housing snap toggle. The client stores the spacing as a user option and pushes it to
/// the engine in centimetres through `House.SetGridSpacingInCentimeters`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridSnap {
    spacing_cm: u32,
}

impl GridSnap {
    /// Rejects a non-positive spacing the way `FUN_00c0ba80` ignores `0 < centimetres` comparisons.
    pub fn new(spacing_cm: u32) -> Result<Self, FixtureLimitsError> {
        if spacing_cm == 0 {
            return Err(FixtureLimitsError::NonPositiveGridSpacing);
        }
        Ok(Self { spacing_cm })
    }

    /// The snap-on state, `GRID_SIZE_ON`.
    pub fn on() -> Self {
        Self {
            spacing_cm: GRID_SPACING_ON_CM,
        }
    }

    /// The snap-off state, `GRID_SIZE_OFF`.
    pub fn off() -> Self {
        Self {
            spacing_cm: GRID_SPACING_OFF_CM,
        }
    }

    pub fn spacing_cm(&self) -> u32 {
        self.spacing_cm
    }

    /// ORIGINAL arithmetic: `100.0 / centimetres`, the value stored at housing manager `+0x4188`.
    pub fn divisions_per_world_unit(&self) -> f32 {
        CENTIMETERS_PER_WORLD_UNIT / self.spacing_cm as f32
    }

    /// ORIGINAL arithmetic from `FUN_00aab720`: `(int)(value * step) * (1.0 / step)`. The cast
    /// truncates toward zero, so a negative coordinate rounds up toward zero rather than down.
    pub fn snap_axis(&self, value: f32) -> f32 {
        let step = self.divisions_per_world_unit();
        (value * step).trunc() / step
    }

    /// ORIGINAL: only x and z are quantised; the recovered function leaves y untouched. Vertical
    /// placement semantics stay unrecovered.
    pub fn snap_position(&self, position: [f32; 3]) -> [f32; 3] {
        [
            self.snap_axis(position[0]),
            position[1],
            self.snap_axis(position[2]),
        ]
    }

    /// ORIGINAL: `Housing:ToggleGrid()` swaps exactly the two recovered values.
    pub fn toggled(&self) -> Self {
        if self.spacing_cm == GRID_SPACING_OFF_CM {
            Self::on()
        } else {
            Self::off()
        }
    }
}

/// ORIGINAL: the two radial-menu rotate actions. Clockwise negates the recovered 15 degree step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

impl RotationDirection {
    pub fn signed_step(&self) -> f32 {
        match self {
            Self::Clockwise => -ROTATION_STEP_RADIANS,
            Self::CounterClockwise => ROTATION_STEP_RADIANS,
        }
    }

    /// Degrees, for operator-facing reports only. The engine works in radians.
    pub fn step_degrees(&self) -> f32 {
        self.signed_step().to_degrees()
    }
}

/// ORIGINAL packet shape: `PlayerHousingInstanceData.BuildAreas` is a list of min/max boxes. One box
/// here; containment is the closed-box test the module performs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuildBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BuildBounds {
    pub fn validate(&self) -> Result<(), FixtureLimitsError> {
        for value in self.min.iter().chain(self.max.iter()) {
            if !value.is_finite() {
                return Err(FixtureLimitsError::NonFiniteValue);
            }
        }
        for axis in 0..3 {
            if self.min[axis] > self.max[axis] {
                return Err(FixtureLimitsError::InvalidBuildBounds);
            }
        }
        Ok(())
    }

    /// Closed box, so a fixture exactly on an edge is inside. This is a proxy for native placement,
    /// not collision.
    pub fn contains(&self, point: [f32; 3]) -> bool {
        (0..3).all(|axis| point[axis] >= self.min[axis] && point[axis] <= self.max[axis])
    }
}

/// ORIGINAL fields: the server supplies both maxima per instance. No numeric maximum is recovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HousingFixtureLimits {
    pub max_fixtures: u32,
    pub max_landmarks: u32,
}

impl HousingFixtureLimits {
    pub fn allows_fixture(&self, current_fixtures: u32) -> bool {
        current_fixtures < self.max_fixtures
    }

    pub fn allows_landmark(&self, current_landmarks: u32) -> bool {
        current_landmarks < self.max_landmarks
    }

    /// The fixture check runs first because `HousingMaxFixtures` is the only recovered denial string.
    pub fn denial(&self, current_fixtures: u32, current_landmarks: u32) -> Option<PlacementDenial> {
        if !self.allows_fixture(current_fixtures) {
            return Some(PlacementDenial::FixtureLimitReached);
        }
        if !self.allows_landmark(current_landmarks) {
            return Some(PlacementDenial::LandmarkLimitReached);
        }
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementDenial {
    FixtureLimitReached,
    LandmarkLimitReached,
}

impl PlacementDenial {
    /// ORIGINAL for the fixture limit; the landmark denial text was not recovered.
    pub fn client_string(&self) -> Option<&'static str> {
        match self {
            Self::FixtureLimitReached => Some(FIXTURE_LIMIT_CLIENT_STRING),
            Self::LandmarkLimitReached => None,
        }
    }
}
