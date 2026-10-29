//! T879: field enemies and battle spawners show a kind-3 "Combat" marker (35/10), sent right
//! after the actor's 35/2. The rows are content (`combat_marker` in `field_enemies.json` and
//! `field_battles.json`, per-actor override or `"off"`), so every expectation is read from the
//! shipped files. Every path that spawns a field enemy is driven through the production engine:
//! the zone-ready batch, a quest unlock, a respawn, and the shared-field `initialization`; the
//! battle spawners go through the visibility view and the shared view.
//!
//! Client rules: the 35/10 handler skips a guid the client has not spawned (`00984960`), a full
//! record needs an image key that resolves to a minimap image (`01025670`), and the marker dies
//! with the actor (so no 35/11). The decoder is `support/quest_marker_packets.rs`.
use osfr_tools::world::NpcVisibility;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs};

#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "support/json.rs"]
mod json_support;
#[path = "support/quest_marker_packets.rs"]
mod quest_marker_packets;

use field_enemy_fixture::{Fixture, content_dir, spawned};
use quest_marker_packets::{Marker, markers, spawn_index};

/// The shipped battle whose spawner the battle tests use.
const BATTLE: u32 = 96;

fn field_file() -> Value {
    json_support::read(&content_dir().join("field_enemies.json"))
}

/// The first shipped enemy that is always present (not gated on a quest).
fn always_on_row() -> Value {
    field_file()["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["quest_ids"] == json!([]))
        .unwrap()
        .clone()
}

fn battle_file() -> Value {
    json_support::read(&content_dir().join("field_battles.json"))
}

/// What a row of the shared `combat_marker` object should put on the wire for an actor whose
/// own name id is `own_name`.
fn expected(shared: &Value, guid: u64, own_name: i64) -> Marker {
    let name = shared["name_id"].as_i64().filter(|id| *id != 0);
    Marker {
        index: 0,
        guid,
        kind: shared["kind"].as_i64().unwrap() as i32,
        state: shared["state"].as_i64().unwrap_or(0) as i32,
        image: shared["image_id"].as_i64().unwrap() as i32,
        name: name.unwrap_or(own_name) as i32,
        subtext: shared["subtext_id"].as_i64().unwrap_or(0) as i32,
    }
}

/// `packets` has `guid`'s 35/2 and, immediately after it, exactly one 35/10 row for it.
fn marker_after_spawn(packets: &[Vec<u8>], guid: u64) -> Option<Marker> {
    let spawn = spawn_index(packets, guid)?;
    let mine: Vec<_> = markers(packets)
        .0
        .into_iter()
        .filter(|marker| marker.guid == guid)
        .collect();
    assert!(mine.len() <= 1, "one marker per spawn");
    mine.first().map(|marker| {
        assert_eq!(marker.index, spawn + 1, "the marker follows the spawn");
        Marker {
            index: 0,
            ..*marker
        }
    })
}

fn rewrite(fixture: &Fixture, edit: impl FnOnce(&mut Value)) {
    let mut file: Value = json_support::read(fixture.field_path());
    edit(&mut file);
    fs::write(fixture.field_path(), file.to_string()).unwrap();
}

#[test]
fn shipped_content_marks_every_always_on_enemy_as_combat() {
    let file = field_file();
    let shared = &file["combat_marker"];
    assert_eq!(shared["kind"], 3, "kind 3 is the client's Combat category");
    assert!(shared["image_id"].as_i64().unwrap() > 0);
    let mut engine = Fixture::shipped().load();
    let packets = engine.on_zone_ready(11, 0).unwrap().replies;
    let always_on: Vec<_> = file["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["quest_ids"] == json!([]))
        .collect();
    assert!(!always_on.is_empty());
    for row in &always_on {
        let guid = row["guid"].as_u64().unwrap();
        assert_eq!(
            marker_after_spawn(&packets, guid),
            Some(expected(shared, guid, row["name_id"].as_i64().unwrap())),
            "enemy {guid}"
        );
    }
    assert_eq!(markers(&packets).0.len(), always_on.len());
    assert!(
        markers(&packets).1.is_empty(),
        "no 35/11: the marker dies with the actor"
    );
}

#[test]
fn a_quest_unlocked_enemy_is_marked_when_it_appears() {
    let file = field_file();
    let row = file["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["quest_ids"] != json!([]))
        .expect("the shipped content has a quest-gated enemy");
    let guid = row["guid"].as_u64().unwrap();
    let quest = row["quest_ids"][0].as_u64().unwrap() as u32;
    let mut engine = Fixture::shipped().load();
    engine.on_zone_ready(11, 0).unwrap();
    let packets = engine.set_active_quests(&BTreeSet::from([quest])).replies;
    assert_eq!(
        marker_after_spawn(&packets, guid),
        Some(expected(
            &file["combat_marker"],
            guid,
            row["name_id"].as_i64().unwrap()
        ))
    );
}

#[test]
fn a_respawned_enemy_gets_its_marker_again() {
    const WOLF: u64 = 131_000_102_001;
    let file = field_file();
    let name = field_enemy_fixture::shipped_row(WOLF).unwrap()["name_id"]
        .as_i64()
        .unwrap();
    let mut engine = Fixture::shipped().load();
    engine.on_zone_ready(11, 0).unwrap();
    let home = engine
        .field_teleport_targets()
        .into_iter()
        .find(|target| target.guid == WOLF)
        .unwrap()
        .position;
    let near = [home[0] + 1.0, home[1], home[2]];
    let mut now = 20_000;
    while engine.is_field_enemy(WOLF) && now < 40_000 {
        engine.attack(WOLF, near, 11, 0, now).unwrap();
        now += 700;
    }
    assert!(
        !engine
            .field_teleport_targets()
            .iter()
            .any(|t| t.guid == WOLF),
        "killed"
    );
    // The marker is not re-sent while the wolf is dead; the respawn spawns it and marks it.
    let mut back = None;
    for second in 1..=120 {
        let out = engine.tick(near, 11, now + second * 1000).unwrap();
        if spawned(&out).contains(&WOLF) {
            back = Some(out.replies);
            break;
        }
    }
    assert_eq!(
        marker_after_spawn(&back.expect("the wolf respawns"), WOLF),
        Some(expected(&file["combat_marker"], WOLF, name))
    );
}

#[test]
fn the_shared_field_view_sends_the_marker_after_the_spawn() {
    let file = field_file();
    let row = &always_on_row();
    let guid = row["guid"].as_u64().unwrap();
    let mut engine = Fixture::shipped().load();
    let field = engine.take_shared_fields().unwrap();
    let packets = field.initialization(guid).unwrap();
    assert_eq!(
        marker_after_spawn(&packets, guid),
        Some(expected(
            &file["combat_marker"],
            guid,
            row["name_id"].as_i64().unwrap()
        ))
    );
}

#[test]
fn content_edits_change_the_packet_with_no_code_change() {
    let fixture = Fixture::copied();
    let file = field_file();
    let rows = file["enemies"].as_array().unwrap();
    let always_on: Vec<u64> = rows
        .iter()
        .filter(|row| row["quest_ids"] == json!([]))
        .map(|row| row["guid"].as_u64().unwrap())
        .collect();
    let [off, own, plain] = [always_on[0], always_on[1], always_on[2]];
    let own_row =
        json!({"kind": 3, "image_id": 25, "name_id": 77, "subtext_id": 55, "hide_overhead": true});
    rewrite(&fixture, |file| {
        file["combat_marker"]["image_id"] = json!(26);
        for enemy in file["enemies"].as_array_mut().unwrap() {
            match enemy["guid"].as_u64().unwrap() {
                guid if guid == off => enemy["combat_marker"] = json!("off"),
                guid if guid == own => enemy["combat_marker"] = own_row.clone(),
                _ => {}
            }
        }
    });
    let mut engine = fixture.load();
    let packets = engine.on_zone_ready(11, 0).unwrap().replies;
    // Switched off: still spawned, no 35/10.
    assert!(spawn_index(&packets, off).is_some());
    assert_eq!(marker_after_spawn(&packets, off), None);
    // Its own row wins over the shared one.
    let mut mine = expected(&own_row, own, 0);
    mine.state = 0;
    assert_eq!(marker_after_spawn(&packets, own), Some(mine));
    // Everyone else follows the edited shared row.
    let name = rows.iter().find(|row| row["guid"] == plain).unwrap()["name_id"]
        .as_i64()
        .unwrap();
    let mut shared = expected(&file["combat_marker"], plain, name);
    shared.image = 26;
    assert_eq!(marker_after_spawn(&packets, plain), Some(shared));
}

#[test]
fn a_file_with_no_shared_row_marks_nothing() {
    let fixture = Fixture::copied();
    rewrite(&fixture, |file| {
        file.as_object_mut().unwrap().remove("combat_marker");
    });
    let mut engine = fixture.load();
    let packets = engine.on_zone_ready(11, 0).unwrap().replies;
    assert!(spawn_index(&packets, always_on_row()["guid"].as_u64().unwrap()).is_some());
    assert!(markers(&packets).0.is_empty());
}

#[test]
fn a_bad_marker_row_is_refused_and_a_refused_reload_changes_nothing() {
    let fixture = Fixture::copied();
    let mut engine = fixture.load();
    let before = engine.on_zone_ready(11, 0).unwrap().replies;
    assert!(!markers(&before).0.is_empty());
    // An image key of 0 is a row the client would drop whole; the loader refuses it.
    rewrite(&fixture, |file| {
        file["combat_marker"]["image_id"] = json!(0)
    });
    let error = engine
        .reload_field_enemies(fixture.field_path())
        .unwrap_err();
    assert!(error.to_string().contains("image id"), "{error}");
    let after = engine.on_zone_ready(11, 0).unwrap().replies;
    assert_eq!(after, before, "the refused reload kept the old rows");
}

#[test]
fn battle_spawners_are_marked_in_both_views() {
    let file = battle_file();
    let shared = &file["combat_marker"];
    let battle = file["battles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == BATTLE)
        .expect("the shipped content has the battle");
    let guid = battle["spawner"]["guid"].as_u64().unwrap();
    let position = battle["spawner"]["position"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect::<Vec<_>>();
    let position = [position[0], position[1], position[2]];
    let mut engine = Fixture::shipped().load();
    engine
        .load_field_battles(&content_dir().join("field_battles.json"))
        .unwrap();
    let want = Some(expected(
        shared,
        guid,
        battle["spawner"]["name_id"].as_i64().unwrap(),
    ));

    // The per-player view: the spawner is a static actor with a notification.
    let mut visibility = NpcVisibility::default();
    visibility.add_battle_spawners(&engine);
    let packets = visibility.update(0, position).unwrap();
    assert_eq!(marker_after_spawn(&packets, guid), want);

    // The shared view: the field owns the spawner and sends it from `initialization`.
    let field = engine.take_shared_fields().unwrap();
    assert_eq!(
        marker_after_spawn(&field.initialization(guid).unwrap(), guid),
        want
    );
}

#[test]
fn a_battle_with_its_marker_off_sends_none() {
    let mut file = battle_file();
    for battle in file["battles"].as_array_mut().unwrap() {
        if battle["id"] == BATTLE {
            battle["combat_marker"] = json!("off");
        }
    }
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("t879-battles-{}.json", std::process::id()));
    fs::write(&path, file.to_string()).unwrap();
    let mut engine = Fixture::shipped().load();
    let loaded = engine.load_field_battles(&path);
    fs::remove_file(&path).unwrap();
    loaded.unwrap();
    let kept = battle_file();
    let guid_of = |id: u32| {
        kept["battles"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["id"] == id)
            .unwrap()["spawner"]["guid"]
            .as_u64()
            .unwrap()
    };
    let marked: BTreeSet<u64> = engine
        .battle_markers()
        .into_iter()
        .map(|(guid, _)| guid)
        .collect();
    assert!(
        !marked.contains(&guid_of(BATTLE)),
        "the battle with its marker off"
    );
    assert!(
        !marked.is_empty(),
        "the other battles follow the shared row"
    );
}
