use super::*;
use serde_json::json;

const GUID: u64 = 100_000_001_631;
const MODEL: u32 = 1;

fn row(
    model_ids: serde_json::Value,
    ids: serde_json::Value,
    min: u64,
    max: u64,
) -> serde_json::Value {
    json!({
        "model_ids": model_ids,
        "model_file_name": "test.adr",
        "animation_ids": ids,
        "min_interval_ms": min,
        "max_interval_ms": max,
        "source": "test (R)",
    })
}

fn with_extra(mut row: serde_json::Value, extra: serde_json::Value) -> serde_json::Value {
    let object = row.as_object_mut().unwrap();
    for (key, value) in extra.as_object().unwrap() {
        object.insert(key.clone(), value.clone());
    }
    row
}

fn content(rows: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schema_version": 1,
        "source": "test content",
        "max_packets_per_viewer_per_tick": 4,
        "rows": rows,
    }))
    .unwrap()
}

fn table(rows: serde_json::Value) -> NpcAmbient {
    NpcAmbient::from_json(&content(rows)).unwrap()
}

fn one_row(ids: serde_json::Value, min: u64, max: u64) -> NpcAmbient {
    table(json!([row(json!([1]), ids, min, max)]))
}

fn guids(updates: &[AmbientUpdate]) -> Vec<u64> {
    updates
        .iter()
        .map(|update| match update {
            AmbientUpdate::Animation { guid, .. }
            | AmbientUpdate::Idle { guid, .. }
            | AmbientUpdate::Conversation { guid, .. }
            | AmbientUpdate::LookAt { guid, .. } => *guid,
        })
        .collect()
}

#[test]
fn the_same_guid_and_window_are_deterministic() {
    let ambient = one_row(json!([3302, 3306]), 500, 5000);
    for from in [0, 1, 999, 4_000] {
        let first = ambient.due(GUID, 1, from, from + 6_000);
        let second = ambient.due(GUID, 1, from, from + 6_000);
        assert_eq!(first, second);
        assert!(first.is_some(), "a six-period window holds a firing");
    }
}

#[test]
fn ten_periods_fire_ten_times_and_never_twice_in_the_next_window() {
    let ambient = one_row(json!([3302]), 1_000, 1_000);
    let mut firings = 0;
    for t in 0..10_000 {
        if let Some(id) = ambient.due(GUID, 1, t, t + 1) {
            assert_eq!(id, 3302);
            firings += 1;
            assert!(
                ambient.due(GUID, 1, t + 1, t + 2).is_none(),
                "a firing must not repeat in the next window"
            );
        }
    }
    assert_eq!(firings, 10, "one firing per period");
}

#[test]
fn the_per_viewer_cap_keeps_the_lowest_guids() {
    let capped = NpcAmbient::from_json(
        &serde_json::to_vec(&json!({
            "schema_version": 1,
            "source": "test content",
            "max_packets_per_viewer_per_tick": 2,
            "rows": [row(json!([1]), json!([3302]), 1_000, 1_000)],
        }))
        .unwrap(),
    )
    .unwrap();
    let known: BTreeSet<u64> = [3, 1, 2].into_iter().collect();
    let due = capped.due_for_known(&known, |_| Some(1), 0, 3_000);
    assert_eq!(guids(&due), [1, 2]);
}

#[test]
fn unknown_models_are_not_scheduled() {
    let ambient = one_row(json!([3302]), 1_000, 2_000);
    assert_eq!(ambient.due(GUID, 99, 0, 10_000), None);
    let known: BTreeSet<u64> = [GUID].into_iter().collect();
    assert!(
        ambient
            .due_for_known(&known, |_| None, 0, 10_000)
            .is_empty()
    );
}

/// A pair within 6 m XZ alternates speakers; a third talker 15 m past the
/// second stays a singleton, and a stalled window never replays a backlog.
#[test]
fn conversation_groups_take_turns_within_six_metres() {
    let mut ambient = table(json!([with_extra(
        row(json!([MODEL]), json!([1_000]), 1_000, 1_000),
        json!({
            "conversation_animation_ids": [3302, 3306],
            "conversation_min_gap_ms": 100,
            "conversation_max_gap_ms": 100,
        }),
    )]));
    ambient.place_actors(&[
        (10, MODEL, [0.0, 0.0, 0.0]),
        (20, MODEL, [5.0, 0.0, 0.0]),
        (30, MODEL, [20.0, 0.0, 0.0]),
    ]);
    let known: BTreeSet<u64> = [10, 20, 30].into_iter().collect();
    let mut speakers = Vec::new();
    for t in 1..=10_000 {
        for update in ambient.due_for_known(&known, |_| Some(MODEL), t - 1, t) {
            if let AmbientUpdate::Conversation { guid, .. } = update {
                speakers.push(guid);
            }
        }
    }
    assert!(speakers.len() >= 6);
    assert!(
        speakers.windows(2).all(|pair| pair[0] != pair[1]),
        "a group of two must take turns: {speakers:?}"
    );
    assert!(
        !speakers.contains(&30),
        "a chained radius keeps the third talker a singleton"
    );
    // Only the latest firing in a long stall is reported.
    let updates = ambient.due_for_known(&known, |_| Some(MODEL), 0, 10_000);
    assert_eq!(
        updates
            .iter()
            .filter(|update| matches!(update, AmbientUpdate::Conversation { .. }))
            .count(),
        1
    );
}

/// An idle row cycles its ids at the fixed interval and queues 35/26 plus
/// the base 35/8 (flags bit 0).
#[test]
fn idle_rows_cycle_and_send_the_base_flag() {
    let mut ambient = table(json!([with_extra(
        row(json!([MODEL]), json!([]), 0, 0),
        json!({
            "idle_animation_ids": [2100, 2101],
            "idle_interval_ms": 1000,
        }),
    )]));
    ambient.place_actors(&[(GUID, MODEL, [0.0, 0.0, 0.0])]);
    let known: BTreeSet<u64> = [GUID].into_iter().collect();
    let mut ids = Vec::new();
    for t in 1..=4_000 {
        for update in ambient.due_for_known(&known, |_| Some(MODEL), t - 1, t) {
            if let AmbientUpdate::Idle { animation_id, .. } = update {
                ids.push(animation_id);
            }
        }
    }
    assert_eq!(ids, [2100, 2101, 2100, 2101]);
    let packets = AmbientUpdate::Idle {
        guid: GUID,
        animation_id: 2100,
    }
    .packets(0);
    assert_eq!(packets.len(), 2);
    let mut idle = vec![35, 0, 26, 0];
    idle.extend(GUID.to_le_bytes());
    idle.extend(2100_i32.to_le_bytes());
    assert_eq!(packets[0], idle);
    let mut base = vec![35, 0, 8, 0];
    base.extend(GUID.to_le_bytes());
    base.extend(2100_i32.to_le_bytes());
    base.extend([0; 4]);
    base.push(1);
    assert_eq!(packets[1], base);
}

#[test]
fn invalid_rows_are_a_load_error() {
    let cases = [
        row(json!([1]), json!([3302]), 2_000, 1_000),
        row(json!([1]), json!([]), 1_000, 1_000),
        row(json!([1]), json!([0]), 1_000, 1_000),
        row(json!([1]), json!([3302, 3302]), 1_000, 1_000),
        row(json!([1]), json!([3302]), 499, 1_000),
        row(json!([1]), json!([3302]), 1_000, 600_001),
        {
            let mut too_many = Vec::new();
            for id in 1..=17 {
                too_many.push(id);
            }
            json!({
                "model_ids": [1],
                "model_file_name": "test.adr",
                "animation_ids": too_many,
                "min_interval_ms": 1_000,
                "max_interval_ms": 1_000,
                "source": "test (R)",
            })
        },
        json!({
            "model_ids": [],
            "model_file_name": "test.adr",
            "animation_ids": [3302],
            "min_interval_ms": 1_000,
            "max_interval_ms": 1_000,
            "source": "test (R)",
        }),
        with_extra(
            row(json!([1]), json!([3302]), 1_000, 1_000),
            json!({
                "conversation_animation_ids": [3302],
                "conversation_min_gap_ms": 100,
                "conversation_max_gap_ms": 60_001,
            }),
        ),
        with_extra(
            row(json!([1]), json!([3302]), 1_000, 1_000),
            json!({
                "conversation_animation_ids": [3302],
                "conversation_min_gap_ms": 200,
                "conversation_max_gap_ms": 100,
            }),
        ),
        with_extra(
            row(json!([1]), json!([3302]), 1_000, 1_000),
            json!({
                "conversation_animation_ids": [3302, 3302],
                "conversation_min_gap_ms": 100,
                "conversation_max_gap_ms": 100,
            }),
        ),
        with_extra(
            row(json!([1]), json!([3302]), 1_000, 1_000),
            json!({
                "idle_animation_ids": [2100],
                "idle_interval_ms": 499,
            }),
        ),
        with_extra(
            row(json!([1]), json!([3302]), 1_000, 1_000),
            json!({
                "look_at_radius": 0.5,
                "look_at_interval_ms": 1_000,
            }),
        ),
        with_extra(
            row(json!([1]), json!([3302]), 1_000, 1_000),
            json!({
                "look_at_radius": 30.5,
                "look_at_interval_ms": 1_000,
            }),
        ),
        with_extra(
            row(json!([1]), json!([3302]), 1_000, 1_000),
            json!({
                "look_at_radius": 10.0,
                "look_at_interval_ms": 499,
            }),
        ),
        row(json!([1]), json!([]), 0, 0),
    ];
    for case in cases {
        let bytes = content(json!([case]));
        assert!(NpcAmbient::from_json(&bytes).is_err(), "{case}");
    }
    // Two rows binding the same client model are ambiguous.
    let duplicate = content(json!([
        row(json!([1]), json!([3302]), 1_000, 1_000),
        row(json!([1]), json!([3306]), 1_000, 1_000),
    ]));
    assert!(NpcAmbient::from_json(&duplicate).is_err());
    // The per-viewer cap is bounded too.
    for cap in [0, 17] {
        let bytes = serde_json::to_vec(&json!({
            "schema_version": 1,
            "source": "test content",
            "max_packets_per_viewer_per_tick": cap,
            "rows": [row(json!([1]), json!([3302]), 1_000, 1_000)],
        }))
        .unwrap();
        assert!(NpcAmbient::from_json(&bytes).is_err());
    }
    for radius in [0.5_f64, 30.5] {
        let bytes = serde_json::to_vec(&json!({
            "schema_version": 1,
            "source": "test content",
            "max_packets_per_viewer_per_tick": 4,
            "conversation_radius_m": radius,
            "rows": [row(json!([1]), json!([3302]), 1_000, 1_000)],
        }))
        .unwrap();
        assert!(NpcAmbient::from_json(&bytes).is_err());
    }
}

/// The client reads 35/8 as a 4-byte header (`0x008d6830`: u16 opcode, u16
/// subtype), u64 GUID, i32 animation, i32 zero, u8 flags (`0x008e5dd0`);
/// 35/26 is the header, u64 GUID, i32 id (`0x008d77a0`); 35/17 is the header
/// and two u64 GUIDs (`0x008d75d0`).
#[test]
fn ambient_packets_match_the_client_reader_layout() {
    let packet = crate::encounter::animation(GUID, 3302, false);
    let mut expected = vec![35, 0, 8, 0];
    expected.extend(GUID.to_le_bytes());
    expected.extend(3302_i32.to_le_bytes());
    expected.extend([0; 4]);
    expected.push(0);
    assert_eq!(packet, expected);

    let idle = idle_animation(GUID, 2100);
    let mut expected = vec![35, 0, 26, 0];
    expected.extend(GUID.to_le_bytes());
    expected.extend(2100_i32.to_le_bytes());
    assert_eq!(idle, expected);

    let target = 100_000_001_632_u64;
    let looked = look_at(GUID, target);
    let mut expected = vec![35, 0, 17, 0];
    expected.extend(GUID.to_le_bytes());
    expected.extend(target.to_le_bytes());
    assert_eq!(looked, expected);
    assert_eq!(look_at(GUID, 0).len(), 20, "target 0 is the clear sentinel");
}

/// One model with an emote row and a conversation row (100 ms gaps).
fn talking_table() -> NpcAmbient {
    table(json!([with_extra(
        row(json!([MODEL]), json!([3302]), 1_000, 1_000),
        json!({
            "conversation_animation_ids": [3306],
            "conversation_min_gap_ms": 100,
            "conversation_max_gap_ms": 100,
        }),
    )]))
}

/// A talker joins a group through any chain of neighbours within 6 m XZ: 10, 20
/// and 30 stand 5 m apart (the ends are 10 m apart, and height is ignored), 40
/// and 50 are a separate pair, and 60 is alone and in no group.
#[test]
fn conversation_groups_are_the_chains_of_six_metre_neighbours() {
    let mut ambient = talking_table();
    ambient.place_actors(&[
        (50, MODEL, [105.0, 0.0, 2.0]),
        (30, MODEL, [10.0, 0.0, 0.0]),
        (60, MODEL, [300.0, 0.0, 0.0]),
        (10, MODEL, [0.0, 0.0, 0.0]),
        (40, MODEL, [100.0, 0.0, 0.0]),
        (20, MODEL, [5.0, 100.0, 0.0]),
    ]);
    let mut groups: Vec<Vec<u64>> = ambient
        .conversation_groups
        .iter()
        .map(|group| group.members.iter().map(|member| member.guid).collect())
        .collect();
    groups.sort();
    assert_eq!(groups, [vec![10, 20, 30], vec![40, 50]]);
}

/// A conversation member's turns are its animation: it does not also play the
/// random emote of its row, while a lone actor of the same model still does.
#[test]
fn conversation_members_do_not_also_play_the_random_emote() {
    let mut ambient = talking_table();
    ambient.place_actors(&[
        (10, MODEL, [0.0, 0.0, 0.0]),
        (20, MODEL, [5.0, 0.0, 0.0]),
        (30, MODEL, [100.0, 0.0, 0.0]),
    ]);
    let known: BTreeSet<u64> = [10, 20, 30].into_iter().collect();
    let mut emoting = BTreeSet::new();
    let mut talking = BTreeSet::new();
    for t in 1..=10_000 {
        for update in ambient.due_for_known(&known, |_| Some(MODEL), t - 1, t) {
            match update {
                AmbientUpdate::Animation { guid, .. } => emoting.insert(guid),
                AmbientUpdate::Conversation { guid, .. } => talking.insert(guid),
                other => unreachable!("{other:?}"),
            };
        }
    }
    assert_eq!(emoting, BTreeSet::from([30]));
    assert_eq!(talking, BTreeSet::from([10, 20]));
}
