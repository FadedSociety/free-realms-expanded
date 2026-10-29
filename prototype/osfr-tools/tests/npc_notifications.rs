//! T555: 35/10 notifications (minimap indicator and overhead icon) for the Sanctuary, Lakeshore
//! and Snowhill warpstones and the Snowhill copper veins.
//!
//! The byte test pins the client reader's order (`008db310`; count `0090aa90`), with a distinct
//! value in every slot so a swapped or dropped field fails. The view tests drive the production
//! paths: the shared fan-out (`NpcVisibility::shared_view`), the per-player batch
//! (`Gameplay::update_npc_visibility`) and the node draw (`apply_node_changes`). The decoder is
//! `support/quest_marker_packets.rs`; the Miner fixture is `support/mining_quest_chain.rs`.

#[path = "support/mining_quest_chain.rs"]
mod mining_quest_chain;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_marker_packets.rs"]
mod quest_marker_packets;

use mining_quest_chain::{Fixture, LEGACY_COPPER_MODEL, nodes_of_model};
use npc_quest_chain::{ADVENTURER, CHARACTER, character_at_for, content_dir};
use osfr_tools::{
    harvest_nodes::{NodeChange, apply_node_changes},
    mining::MINER_PROFILE_ID,
    npc_notifications::{MAX_ENTRIES, Notification, encode_add},
    world::NpcVisibility,
};
use quest_marker_packets::{Marker, markers, spawn_index};
use std::collections::BTreeSet;

/// `Resources/Models.txt` 4019, the Snowhill vein.
const VEIN_MODEL: u32 = 4_019;
/// (spawn guid, position, name string id) of the three Sanctuary-main warpstones
/// (`npc_spawns_extra.json`; guid = 100000000000 + `Npcs.json` Id).
const STONES: [(u64, [f32; 3], i32); 3] = [
    (100_000_033_167, [-1418.3218, -27.002932, 347.82056], 20_831),
    (100_000_033_558, [-721.72, 9.56, 948.72], 41_298),
    (100_000_033_602, [50.93, 32.43, 370.30], 20_830),
];

fn row(index: usize, guid: u64, name: i32) -> Marker {
    Marker {
        index,
        guid,
        kind: 6,
        state: 3,
        image: 3,
        name,
        subtext: 0,
    }
}

fn shared(
    visibility: &NpcVisibility,
    position: [f32; 3],
    known: &BTreeSet<u64>,
) -> (BTreeSet<u64>, Vec<Vec<u8>>) {
    visibility
        .shared_view(0, position, &[], known)
        .expect("the shared view builds")
}

#[test]
fn add_notifications_bytes_match_the_reader() {
    let notification = Notification {
        kind: 6,
        image_id: 22,
        state: 11,
        name_id: 33,
        subtext_id: 44,
        hide_overhead: true,
        enabled: true,
    };
    let second = Notification {
        hide_overhead: false,
        enabled: false,
        ..notification
    };
    let bytes = encode_add(&[(0x0102_0304_0506_0708, notification), (9, second)]).unwrap();
    let mut expected = vec![35, 0, 10, 0, 2, 0, 0, 0];
    for (guid, hide, enabled) in [(0x0102_0304_0506_0708_u64, 1, 1), (9, 0, 0)] {
        expected.extend(guid.to_le_bytes());
        expected.push(0); // compact
        expected.extend(6i32.to_le_bytes()); // kind (+28)
        expected.extend(11i32.to_le_bytes()); // state (+8)
        expected.extend(22i32.to_le_bytes()); // image id (+16)
        expected.extend(0i32.to_le_bytes()); // (+12)
        expected.extend(33i32.to_le_bytes()); // name id (+20)
        expected.extend(44i32.to_le_bytes()); // sub-text id (+24)
        expected.push(hide); // hide overhead (+32)
        expected.extend(0i32.to_le_bytes()); // (+36)
        expected.push(enabled); // trailing (+41), read for both layouts
    }
    assert_eq!(bytes, expected);
    assert_eq!(bytes.len(), 8 + 39 * 2);
    assert_eq!(encode_add(&[]).unwrap(), [35, 0, 10, 0, 0, 0, 0, 0]);
}

#[test]
fn bad_rows_are_refused_and_one_packet_is_bounded() {
    let good = Notification {
        kind: 6,
        image_id: 3,
        state: 3,
        name_id: 1,
        subtext_id: -1,
        hide_overhead: false,
        enabled: true,
    };
    assert_eq!(good.validate(), None);
    let bad = [
        Notification {
            image_id: 0,
            ..good
        },
        Notification {
            image_id: 10_001,
            ..good
        },
        Notification { kind: 256, ..good },
        Notification { kind: -1, ..good },
        Notification { state: -1, ..good },
        Notification {
            name_id: -1,
            ..good
        },
        Notification {
            subtext_id: -2,
            ..good
        },
    ];
    for (index, notification) in bad.iter().enumerate() {
        assert!(notification.validate().is_some(), "case {index}");
    }
    // One packet never carries a count it could not hold.
    let rows = vec![(1, good); MAX_ENTRIES + 1];
    assert!(encode_add(&rows).is_none());
    assert!(encode_add(&rows[..MAX_ENTRIES]).is_some());
}

#[test]
fn a_viewer_learns_each_warpstone_then_its_notification() {
    let visibility = NpcVisibility::load(&content_dir()).expect("the shipped NPC content loads");
    for (guid, position, name) in STONES {
        let (_, packets) = shared(&visibility, position, &BTreeSet::new());
        let spawn = spawn_index(&packets, guid).expect("the viewer is sent the stone");
        let (shown, cleared) = markers(&packets);
        assert!(cleared.is_empty());
        let mine: Vec<_> = shown.iter().filter(|marker| marker.guid == guid).collect();
        assert_eq!(mine, [&row(spawn + 1, guid, name)], "stone {guid}");
    }
}

#[test]
fn the_per_player_batch_sends_the_notification_after_the_spawn() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER_PROFILE_ID, &[ADVENTURER, MINER_PROFILE_ID]);
    let (guid, position, name) = STONES[2];
    let character = character_at_for(CHARACTER, 0, position);
    let packets = game.update_npc_visibility(&character).unwrap();
    let spawn = spawn_index(&packets, guid).expect("the batch spawns the stone");
    assert!(markers(&packets).0.contains(&row(spawn + 1, guid, name)));
    // Already sent: the client keeps the actor and its notification, so nothing repeats.
    let again = game.update_npc_visibility(&character).unwrap();
    assert!(markers(&again).0.iter().all(|marker| marker.guid != guid));
}

#[test]
fn an_actor_the_viewer_has_not_been_sent_or_has_forgotten_gets_no_notification() {
    let visibility = NpcVisibility::load(&content_dir()).expect("the shipped NPC content loads");
    let (guid, position, name) = STONES[0];
    let (near, packets) = shared(&visibility, position, &BTreeSet::new());
    assert!(near.contains(&guid));
    assert!(markers(&packets).0.iter().any(|marker| marker.guid == guid));

    // Far away the stone is never spawned for this viewer, so it is never notified.
    let far = [position[0] + 5_000.0, position[1], position[2]];
    let (seen, packets) = shared(&visibility, far, &BTreeSet::new());
    assert!(!seen.contains(&guid));
    assert!(spawn_index(&packets, guid).is_none());
    assert!(markers(&packets).0.iter().all(|marker| marker.guid != guid));

    // Walking away removes it (35/3) with no notification; the client forgot it.
    let (seen, packets) = shared(&visibility, far, &near);
    assert!(!seen.contains(&guid));
    assert!(markers(&packets).0.iter().all(|marker| marker.guid != guid));

    // A viewer that already knows the stone is not sent its spawn or notification again.
    let (_, packets) = shared(&visibility, position, &near);
    assert!(spawn_index(&packets, guid).is_none());
    assert!(markers(&packets).0.iter().all(|marker| marker.guid != guid));

    // Coming back after forgetting it sends both again, in order.
    let (_, packets) = shared(&visibility, position, &seen);
    let spawn = spawn_index(&packets, guid).expect("re-spawned");
    assert!(markers(&packets).0.contains(&row(spawn + 1, guid, name)));
}

#[test]
fn the_copper_veins_show_on_the_minimap_and_the_old_pool_does_not() {
    let veins = nodes_of_model(VEIN_MODEL);
    assert_eq!(veins.len(), 8);
    let expected = Marker {
        index: 0,
        guid: 0,
        kind: 6,
        state: 340,
        image: 340,
        name: 7_228,
        subtext: -1,
    };
    for vein in &veins {
        let mut visibility = NpcVisibility::default();
        apply_node_changes(&mut visibility, vec![NodeChange::Show(vein.clone())]);
        let packets = visibility.update(0, vein.position).unwrap();
        let spawn = spawn_index(&packets, vein.guid).expect("the vein is drawn");
        assert_eq!(
            markers(&packets).0,
            [Marker {
                index: spawn + 1,
                guid: vein.guid,
                ..expected
            }]
        );

        // Depleted and respawned: the hide sends no notification, the next show sends one.
        apply_node_changes(
            &mut visibility,
            vec![NodeChange::Hide {
                guid: vein.guid,
                companion: None,
            }],
        );
        let gone = visibility.update(0, vein.position).unwrap();
        assert!(markers(&gone).0.is_empty());
        apply_node_changes(&mut visibility, vec![NodeChange::Show(vein.clone())]);
        let back = visibility.update(0, vein.position).unwrap();
        assert_eq!(markers(&back).0.len(), 1);
    }
    for old in nodes_of_model(LEGACY_COPPER_MODEL) {
        let mut visibility = NpcVisibility::default();
        apply_node_changes(&mut visibility, vec![NodeChange::Show(old.clone())]);
        let packets = visibility.update(0, old.position).unwrap();
        assert!(spawn_index(&packets, old.guid).is_some());
        assert!(markers(&packets).0.is_empty());
    }
}
