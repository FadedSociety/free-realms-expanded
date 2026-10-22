//! Shared test helper: the audited field camp centres (T415).
//!
//! Include with `#[path = "common/field_camp_centres.rs"] mod field_camp_centres;`.
//! The 12 W102 starter centres passed `lab/reports/field-enemies-w102/check_placements.py`.
//! T415 moved the 18 level 5-20 camps off them; each new centre passed that
//! checker's `validate()` plus the separation, battle-marker and Flame Wave
//! clearances (checker output per centre in `notes/Autopilot/T415.md`). A home
//! that is not listed here has not been audited.
#![allow(dead_code)] // each suite uses a subset

use serde_json::Value;

/// Spawn-area id and centre, in `data/content/field_enemies.json` order: the 12
/// W102 starter camps first.
pub const VERIFIED_CENTRES: [(&str, [f32; 3]); 30] = [
    ("W102-F1", [-1899.204, -50.59, 307.902]),
    ("W102-F2", [-1887.867, -51.14, 291.159]),
    ("W102-F3", [-1962.89, -46.92, 284.723]),
    ("W102-H1", [-1779.455, -28.88, 562.833]),
    ("W102-H2", [-1792.921, -29.33, 579.55]),
    ("W102-H3", [-1789.626, -30.24, 616.51]),
    ("W102-Q1", [-664.762, 2.4, 299.608]),
    ("W102-Q2", [-692.141, 2.4, 220.373]),
    ("W102-Q3", [-651.667, 2.4, 231.947]),
    ("W102-T1", [-347.084, 0.61, 1068.348]),
    ("W102-T2", [-351.103, 2.33, 1048.674]),
    ("W102-T3", [-447.084, 3.03, 1128.348]),
    ("MID-F1", [-2077.0, -48.7, 425.0]),
    ("MID-F2", [-2065.0, -47.17, 425.0]),
    ("MID-H1", [-2013.0, -37.6, 609.0]),
    ("MID-H2", [-2013.0, -35.75, 597.0]),
    ("MID-T1", [-480.0, 4.6, 1110.0]),
    ("MID-T2", [-489.0, 6.9, 1118.0]),
    ("MID-Q1", [-608.0, -2.2, 172.0]),
    ("MID-Q2", [-608.0, -2.1, 160.0]),
    ("HIGH-F3", [-2136.0, -40.2, 516.0]),
    ("HIGH-F2", [-2148.0, -37.8, 516.0]),
    ("HIGH-T3", [-512.0, 6.1, 1214.0]),
    ("HIGH-T2", [-512.0, 5.5, 1226.0]),
    ("ELITE-T1", [-408.0, 0.1, 1362.0]),
    ("ELITE-T2", [-420.0, 0.0, 1362.0]),
    ("ELITE-Q1", [-572.0, 16.3, 320.0]),
    ("ELITE-Q2", [-560.0, 16.6, 320.0]),
    ("ELITE-H1", [-1817.0, -44.45, 176.0]),
    ("ELITE-H2", [-1817.0, -43.45, 188.0]),
];

/// The audited centre of `area`; usable in a `const`, where a typo is a compile error.
pub const fn verified_centre(area: &str) -> [f32; 3] {
    let mut i = 0;
    while i < VERIFIED_CENTRES.len() {
        let name = VERIFIED_CENTRES[i].0.as_bytes();
        let wanted = area.as_bytes();
        let mut same = name.len() == wanted.len();
        let mut j = 0;
        while same && j < name.len() {
            same = name[j] == wanted[j];
            j += 1;
        }
        if same {
            return VERIFIED_CENTRES[i].1;
        }
        i += 1;
    }
    panic!("not an audited field camp centre");
}

/// A pinned row with its placement masked. T415 moved every camp whose area is
/// not a W102 starter area, so its position, centre, reference spawn and
/// provenance text are left out; a stat, level, area or group change still
/// changes the digest. Any other row is returned unchanged.
pub fn placement_masked(row: &Value) -> Value {
    let mut row = row.clone();
    if row["spawn_area"]["id"]
        .as_str()
        .is_some_and(|area| !area.starts_with("W102-"))
    {
        let fields = row.as_object_mut().unwrap();
        for key in ["position", "reference_spawn_guid", "source"] {
            fields.remove(key);
        }
        row["spawn_area"].as_object_mut().unwrap().remove("centre");
    }
    row
}
