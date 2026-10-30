//! T889: the Legacy Hooligan (4667) and Hooligan Archer (4675) spawns the owner clicked are
//! friendly extra spawns; `field_enemies.json` gives each a hostile body with the same GUID.
//!
//! Client evidence: the 35/2 AddNpc reader `0x00921A10` takes the nameplate colour from the
//! disposition word (wire offset 77: 0 hostile, red; 1 friendly, blue), the same word every
//! other field enemy is sent with (`encounter.rs` `spawn_actor`).

#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "support/json.rs"]
mod json;

use field_enemy_fixture::{Fixture, content_path};

const TENT: [(u64, &str, u32, [f32; 2]); 2] = [
    (100000004667, "Hooligan", 7, [-2033.0, 634.0]),
    (100000004675, "Hooligan Archer", 8, [-2090.0, 694.0]),
];

#[test]
fn the_tent_hooligans_are_sent_hostile_with_their_authored_level() {
    let mut engine = Fixture::shipped().load();
    let out = engine.on_zone_ready(11, 0).unwrap();
    for (guid, _, level, _) in TENT {
        let packet = out
            .replies
            .iter()
            .find(|p| p.starts_with(&[35, 0, 2, 0]) && p[4..12] == guid.to_le_bytes())
            .unwrap_or_else(|| panic!("{guid} is spawned through the 35/2 AddNpc path"));
        assert_eq!(
            i32::from_le_bytes(packet[77..81].try_into().unwrap()),
            0,
            "{guid} is sent hostile (red nameplate), not friendly (blue)"
        );
        assert_eq!(engine.field_level(guid), Some(level));
    }
}

#[test]
fn the_tent_hooligans_sit_in_one_always_on_group_at_their_audited_spots() {
    let field = json::read(&content_path());
    let group = field["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["id"] == "mid_highroad_tent_hooligans")
        .expect("the tent group exists");
    for (guid, name, level, at) in TENT {
        assert!(group["members"].as_array().unwrap().contains(&guid.into()));
        let row = field["enemies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["guid"] == guid)
            .unwrap();
        assert_eq!(row["name"], name);
        assert_eq!(row["level"], level);
        assert_eq!(row["quest_ids"], serde_json::json!([]), "always on");
        assert_eq!(row["position"][0].as_f64().unwrap() as f32, at[0]);
        assert_eq!(row["position"][2].as_f64().unwrap() as f32, at[1]);
    }
}
