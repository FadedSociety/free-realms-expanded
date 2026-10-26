//! T536: Sanctuary's two world radios are permanent XVD-S dance spots. The shipped
//! `npc_spawns_extra.json` carries them with a `boombox_item_id`; `boombox_props` reads them
//! back, `Boomboxes::permanent` binds them to the catalogue (an item it lacks fails the load), and
//! the prop itself is drawn as a bouncing, nameless, non-interactable box. The dance behaviour is
//! gated in `boombox.rs` (`tests::permanent`) and, through the gateway tick, in `boombox_routing`.
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;

use osfr_tools::{
    boombox::{Boomboxes, Catalogue},
    world::{self, BoomboxProp, NpcVisibility},
};
use serde_json::{Value, json};
use std::fs;

const XVD: i32 = 48244;
/// Sanctuary-main 10f129e `FabledRealms.lua:3838` and `:4000` (Npcs.json 33617, 33838).
const RADIOS: [(u64, [f32; 3]); 2] = [
    (100_000_033_617, [88.90495, 23.542553, 410.57498]),
    (100_000_033_838, [-63.054962, 5.882222, 327.70673]),
];

fn extras() -> Vec<Value> {
    let file = json::read(&npc_visibility::content_dir().join("npc_spawns_extra.json"));
    file["records"].as_array().unwrap().clone()
}

#[test]
fn the_two_sanctuary_radios_are_the_only_permanent_boxes_and_bind_to_the_catalogue() {
    let props = world::boombox_props(&npc_visibility::content_dir()).unwrap();
    let expected: Vec<_> = RADIOS
        .iter()
        .map(|(spawn_guid, position)| BoomboxProp {
            spawn_guid: *spawn_guid,
            item_id: XVD,
            position: *position,
        })
        .collect();
    assert_eq!(props, expected);
    let catalogue = Catalogue::load(&npc_visibility::content_dir()).unwrap();
    assert!(Boomboxes::permanent(&catalogue, &props).is_ok());
    // The row is the Boombox XVD-S radio model.
    assert_eq!(catalogue.get(XVD).unwrap().model_id, 185);
}

#[test]
fn no_legacy_radio_lies_within_a_metre_of_a_sanctuary_one_so_both_are_new_rows() {
    let rows = extras();
    for (guid, position) in RADIOS {
        let row = rows.iter().find(|r| r["spawn_guid"] == guid).unwrap();
        let evidence: Vec<_> = row["source"].as_array().unwrap().iter().collect();
        let kinds: Vec<_> = evidence
            .iter()
            .map(|tag| tag["kind"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, ["reference", "client", "reconstructed"], "{guid}");
        assert!(
            evidence[2]["evidence"].as_str().unwrap().starts_with("(R)"),
            "{guid}: the dance-spot role is reconstructed"
        );
        for other in rows
            .iter()
            .filter(|r| r["model_id"] == 185 && r.get("boombox_item_id").is_none())
        {
            let at: Vec<f64> = other["position"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let distance = at
                .iter()
                .zip(position)
                .map(|(a, b)| (a - f64::from(b)).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(
                distance > 1.0,
                "radio {} is within 1 m of {guid}: tag it instead of adding a row",
                other["spawn_guid"]
            );
        }
    }
}

#[test]
fn the_radio_prop_is_drawn_bouncing_nameless_and_not_interactable() {
    let visibility = NpcVisibility::load(&npc_visibility::content_dir()).unwrap();
    let mut drawn = std::collections::BTreeMap::new();
    for (_, position) in RADIOS {
        for npc in npc_visibility::seen_from(&visibility, position) {
            drawn.insert(npc.actor_id, npc);
        }
    }
    for (guid, position) in RADIOS {
        let npc = drawn
            .get(&guid)
            .unwrap_or_else(|| panic!("{guid} not drawn"));
        assert_eq!(npc.model_id, 185);
        assert_eq!(npc.name_string_id, 18044, "locale 18044 = Boom Box");
        assert_eq!(npc.position[..3], position);
        assert_eq!(
            (
                npc.animation_id,
                npc.hide_nameplate,
                npc.is_interactable,
                npc.interact_range
            ),
            (2100, 1, 0, 0)
        );
    }
}

/// A content directory holding only an `npc_spawns_extra.json` of `rows`.
fn content_with(tag: &str, rows: Vec<Value>) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("osfr-t536-{tag}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let file = json!({"schema_version": 1, "records": rows});
    fs::write(dir.join("npc_spawns_extra.json"), file.to_string()).unwrap();
    dir
}

fn radio_row(guid: u64, boombox: Option<i32>, capture: Option<Value>) -> Value {
    let mut row = json!({
        "spawn_guid": guid, "model_id": 185, "position": [1.0, 2.0, 3.0],
        "capture_match": capture,
    });
    if let Some(item) = boombox {
        row["boombox_item_id"] = json!(item);
    }
    row
}

#[test]
fn a_row_naming_a_boombox_the_catalogue_lacks_fails_the_load_naming_it() {
    let dir = content_with(
        "missing",
        vec![
            radio_row(7, Some(XVD), None),
            radio_row(8, Some(999_999), None),
        ],
    );
    let props = world::boombox_props(&dir).unwrap();
    let catalogue = Catalogue::load(&npc_visibility::content_dir()).unwrap();
    let error = Boomboxes::permanent(&catalogue, &props)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("NPC spawn 8") && error.contains("999999"),
        "{error}"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn only_drawn_rows_with_an_item_are_props_and_a_set_without_the_file_has_none() {
    let dir = content_with(
        "drawn",
        vec![
            radio_row(1, None, None),
            radio_row(2, Some(XVD), Some(json!({"actor_id": 5}))),
            radio_row(3, Some(XVD), None),
        ],
    );
    let guids: Vec<_> = world::boombox_props(&dir)
        .unwrap()
        .iter()
        .map(|p| p.spawn_guid)
        .collect();
    assert_eq!(
        guids,
        [3],
        "a plain radio and an undrawn captured row are not dance spots"
    );
    fs::remove_dir_all(&dir).unwrap();
    assert!(world::boombox_props(&dir).unwrap().is_empty());
}
