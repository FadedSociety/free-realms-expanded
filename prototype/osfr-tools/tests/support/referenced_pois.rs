//! POI rows for the scratch packages the flat-schema content gates assemble.
//!
//! `ContentDb::load` checks every `reach_poi` goal against `pois.json`; the shipped split quest
//! files name points of the captured client startup table, which these scratch packages do not
//! include. Test-only: include with `#[path = "support/referenced_pois.rs"]`.

use serde_json::{Value, json};

/// A POI row for every point a quest's `reach_poi` goal names.
pub fn referenced_pois(quests: &[Value]) -> Value {
    let ids: std::collections::BTreeSet<_> = quests
        .iter()
        .flat_map(|quest| quest["objectives"].as_array().into_iter().flatten())
        .filter_map(|goal| goal["poi_id"].as_i64())
        .collect();
    let source = json!([{"kind": "reconstructed", "evidence": "(R) test fixture: a quest-referenced point of interest"}]);
    json!({"schema_version": 1, "records": ids.iter().map(|id| json!({
        "id": id, "zone_id": 7, "name_string_id": 0,
        "transform": {"position": [0.0, 0.0, 0.0], "position_w": null,
            "rotation": {"kind": "heading", "value": 0.0}, "scale": null},
        "source": source,
    })).collect::<Vec<_>>()})
}
