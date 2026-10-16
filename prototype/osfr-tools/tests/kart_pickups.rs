//! Kart pickup set - owned vectors against the original capture.
//!
//! Golden frames are the real `lab/reports/original-minigame-packets` captures (application envelope
//! plus inner frame); the sha256 pins each complete 43/15-byte frame. The pickup set works on the
//! inner bytes the gateway hands it, so the tests drop the 7-byte envelope.
//!
//! Exercised contract: exact 32-byte `33/15`, 4-byte `33/16`, 4-byte `33/18` codecs; bounded
//! definitions; one owned claim consumes exactly one box after the caller stages, replying grant
//! then destroy in the original order; snapshot restore is known+unique and legacy saves are empty.

use osfr_tools::kart_pickups::{
    BOOST_ITEM_TYPE, CLAIM_SUBTYPE, CREATE_PAYLOAD_LEN, CREATE_SUBTYPE, DEFAULT_RADIUS_M,
    DESTROY_SUBTYPE, EXPIRE_SUBTYPE, GRANT_SUBTYPE, MAX_PICKUPS, MAX_RADIUS_M,
    MAX_RESPAWN_DELAY_MS, MAX_RESPAWNS_PER_PICKUP, MIN_RESPAWN_DELAY_MS, MINE_ITEM_TYPE,
    MISSILE_ITEM_TYPE, PICKUP_MODEL, PendingRespawn, PickupError, PickupRefusal, PickupRespawn,
    PickupSet, PickupSnapshot, PickupSpec, SHIELD_EXPIRE_VALUE, SHIELD_ITEM_TYPE, encode_create,
    encode_create_payload, encode_destroy, encode_expire, encode_grant, is_recovered_item_type,
};

/// The first captured `33/15` (handle 1, model 497) with its complete frame and pinned hash.
const CAPTURED_CREATE: &str =
    "0500012400000021000f0001000000f1010000bc8c204400008d42d17205440000803f0000803f2c050000";
const CAPTURED_CREATE_SHA256: &str =
    "27a773b09822e8eaa89d7cc07d39bd1ac8da7aaacaf87a04de90903baefda9fb";
/// The captured `33/16` that removes handle 1 in the same frame as the type-5 grant.
const CAPTURED_DESTROY: &str = "050001080000002100100001000000";
const CAPTURED_DESTROY_SHA256: &str =
    "42e3c12b1479d9d32273fafeba6c54414df586283827e4b82920ac4c34de008a";
/// The captured `33/18` that grants the boost (type 5) for that box.
const CAPTURED_GRANT: &str = "050001080000002100120005000000";
const CAPTURED_GRANT_SHA256: &str =
    "3d9c2751ef7177229e25b8be50fff13fee6d65e6577efbeae0465bbbdfac7ff4";

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len() / 2)
        .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap())
        .collect()
}

/// The gateway strips the 7-byte application envelope before the service sees a frame.
fn inner(frame: &[u8]) -> &[u8] {
    &frame[7..]
}

fn sha256(frame: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(frame)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn captured_box() -> PickupSpec {
    PickupSpec {
        handle: 1,
        model: PICKUP_MODEL,
        position: [642.198_974_609_375, 70.5, 533.794_006_347_656_2],
        radius: DEFAULT_RADIUS_M,
        item_type: BOOST_ITEM_TYPE,
    }
}

fn subtypes(frames: &[Vec<u8>]) -> Vec<u16> {
    frames
        .iter()
        .map(|frame| u16::from_le_bytes([frame[2], frame[3]]))
        .collect()
}

fn payload(frame: &[u8]) -> &[u8] {
    &frame[4..]
}

#[test]
fn captured_create_destroy_and_grant_frames_round_trip_exactly() {
    let create = bytes(CAPTURED_CREATE);
    let destroy = bytes(CAPTURED_DESTROY);
    let grant = bytes(CAPTURED_GRANT);
    assert_eq!(sha256(&create), CAPTURED_CREATE_SHA256);
    assert_eq!(sha256(&destroy), CAPTURED_DESTROY_SHA256);
    assert_eq!(sha256(&grant), CAPTURED_GRANT_SHA256);
    assert_eq!(create.len(), 43);
    assert_eq!(destroy.len(), 15);
    assert_eq!(grant.len(), 15);

    let spec = captured_box();
    // `inner()` still carries the 4-byte op/sub header; the payload starts after it.
    let captured_payload = &inner(&create)[4..];
    assert_eq!(encode_create_payload(&spec), captured_payload);
    assert_eq!(encode_create_payload(&spec).len(), CREATE_PAYLOAD_LEN);
    assert_eq!(encode_create(&spec), inner(&create));
    assert_eq!(encode_destroy(spec.handle), inner(&destroy));
    assert_eq!(encode_grant(BOOST_ITEM_TYPE), inner(&grant));
    // The two unlabelled capture words are written back verbatim, never reinterpreted.
    assert_eq!(
        u32::from_le_bytes(captured_payload[28..32].try_into().unwrap()),
        1324
    );
    assert_eq!(
        f32::from_le_bytes(captured_payload[24..28].try_into().unwrap()),
        1.0
    );
    assert_eq!(CLAIM_SUBTYPE, 17);
    assert_eq!(
        (CREATE_SUBTYPE, DESTROY_SUBTYPE, GRANT_SUBTYPE),
        (15, 16, 18)
    );
}

/// The expiry frame: `33/21` `VehicleRacePacketItemExpired` carries one u32, and the lead's
/// raw PE comparison turns the type-4 shield off with value 4; value 1 is ignored.
#[test]
fn item_expired_frame_matches_the_recovered_shape() {
    let frame = encode_expire(SHIELD_EXPIRE_VALUE);
    assert_eq!(subtypes(std::slice::from_ref(&frame)), vec![EXPIRE_SUBTYPE]);
    assert_eq!(payload(&frame), SHIELD_EXPIRE_VALUE.to_le_bytes());
    // Raw native PE at 0xba88c1/0xba88c6 subtracts 2 twice, then calls the
    // shield-off toggler at 0xba88cc. Selector 1 is silently ignored.
    assert_eq!((EXPIRE_SUBTYPE, SHIELD_EXPIRE_VALUE), (21, 4));
}

#[test]
fn definitions_are_bounded_and_typed() {
    let native = |handle: u32| PickupSpec::native(handle, [handle as f32, 0.0, 0.0]);

    let too_many: Vec<PickupSpec> = (1..=(MAX_PICKUPS as u32 + 1)).map(native).collect();
    assert_eq!(
        PickupSet::new(too_many).unwrap_err(),
        PickupError::TooMany {
            count: MAX_PICKUPS + 1,
            max: MAX_PICKUPS
        }
    );
    assert_eq!(
        PickupSet::new(vec![native(1), native(1)]).unwrap_err(),
        PickupError::DuplicateHandle { handle: 1 }
    );
    assert_eq!(
        PickupSet::new(vec![PickupSpec::native(0, [0.0, 0.0, 0.0])]).unwrap_err(),
        PickupError::ZeroHandle
    );
    let mut nan = native(2);
    nan.position = [f32::NAN, 0.0, 0.0];
    assert_eq!(
        PickupSet::new(vec![nan]).unwrap_err(),
        PickupError::NonFinitePosition { handle: 2 }
    );
    for radius in [0.0, -1.0, f32::INFINITY, MAX_RADIUS_M + 0.25] {
        let mut spec = native(3);
        spec.radius = radius;
        assert_eq!(
            PickupSet::new(vec![spec]).unwrap_err(),
            PickupError::BadRadius { handle: 3, radius }
        );
    }
    let mut combat = native(4);
    // 1/3/4/5 are the recovered types now; 2 is not one of them.
    combat.item_type = 2;
    assert_eq!(
        PickupSet::new(vec![combat]).unwrap_err(),
        PickupError::UnsupportedItemType {
            handle: 4,
            item_type: 2
        }
    );
}

#[test]
fn claim_consumes_once_and_replies_grant_then_destroy() {
    let spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
    let mut set = PickupSet::new(vec![spec]).unwrap();
    assert_eq!(set.create_frames(), vec![encode_create(&spec)]);
    assert_eq!(set.live_create_frames(), set.create_frames());
    assert_eq!(set.live_count(), 1);

    let claimed = set
        .claim(7, 7, [1.0, 0.0, 0.0], 1_000, 600_000)
        .expect("owned claim inside the radius");
    assert_eq!(claimed.handle, 1);
    assert_eq!(claimed.item_type, BOOST_ITEM_TYPE);
    // The original lifecycle is grant (33/18) then destroy (33/16), nothing else.
    assert_eq!(
        subtypes(&claimed.replies),
        vec![GRANT_SUBTYPE, DESTROY_SUBTYPE]
    );
    assert_eq!(payload(&claimed.replies[0]), BOOST_ITEM_TYPE.to_le_bytes());
    assert_eq!(payload(&claimed.replies[1]), 1u32.to_le_bytes());
    assert_eq!(
        claimed.replies.len(),
        2,
        "a claim produces no economy frames"
    );
    assert!(set.is_consumed(1));
    assert_eq!(set.live_count(), 0);
    assert!(
        set.create_frames().is_empty(),
        "a consumed box is not re-announced"
    );
    assert!(set.live_create_frames().is_empty());

    // The same position can never pay twice: the box is gone.
    assert_eq!(
        set.claim(7, 7, [1.0, 0.0, 0.0], 2_000, 600_000)
            .unwrap_err(),
        PickupRefusal::NoPickupInRange
    );
    // Committing the same staged claim twice is refused, not silently paid.
    let staged = set
        .stage_claim(7, 7, [0.5, 0.0, 0.0])
        .err()
        .expect("the only box is consumed");
    assert_eq!(staged, PickupRefusal::NoPickupInRange);
    let mut fresh = PickupSet::new(vec![spec]).unwrap();
    let staged = fresh.stage_claim(7, 7, [0.0, 0.0, 0.0]).unwrap();
    assert_eq!(staged.handle(), 1);
    assert_eq!(staged.item_type(), BOOST_ITEM_TYPE);
    assert_eq!(fresh.commit(staged, 1_000, 600_000).unwrap().len(), 2);
    assert_eq!(
        fresh.commit(staged, 1_001, 600_000).unwrap_err(),
        PickupRefusal::AlreadyConsumed { handle: 1 }
    );
}

#[test]
fn nearest_and_equidistant_boxes_are_stable() {
    let near = PickupSpec::native(20, [2.0, 0.0, 0.0]);
    let tie = PickupSpec::native(10, [0.0, 0.0, 0.0]);
    let mut set = PickupSet::new(vec![near, tie]).unwrap();
    // Both are 1 m from x = 1: the lowest handle wins, deterministically.
    let first = set.claim(1, 1, [1.0, 0.0, 0.0], 1_000, 600_000).unwrap();
    assert_eq!(first.handle, 10);
    let second = set.claim(1, 1, [1.0, 0.0, 0.0], 2_000, 600_000).unwrap();
    assert_eq!(second.handle, 20);
    assert_eq!(set.live_count(), 0);
    // The same geometry restored from a snapshot resolves identically.
    let restored =
        PickupSet::restore(vec![near, tie], None, PickupSnapshot::default(), 0, 600_000).unwrap();
    assert_eq!(
        restored
            .stage_claim(1, 1, [1.0, 0.0, 0.0])
            .unwrap()
            .handle(),
        10
    );
}

#[test]
fn snapshot_round_trips_and_legacy_restores_empty() {
    let specs = vec![
        PickupSpec::native(1, [0.0, 0.0, 0.0]),
        PickupSpec::native(2, [10.0, 0.0, 0.0]),
    ];
    let mut set = PickupSet::new(specs.clone()).unwrap();
    set.claim(3, 3, [0.0, 0.0, 0.0], 1_000, 600_000).unwrap();
    let snapshot = set.snapshot();
    assert_eq!(snapshot.consumed, vec![1]);
    let restored = PickupSet::restore(specs.clone(), None, snapshot.clone(), 0, 600_000).unwrap();
    assert_eq!(restored, set);
    assert_eq!(restored.live_count(), 1);
    assert!(restored.is_consumed(1));
    // A restored set re-announces only the boxes that are still live.
    assert_eq!(
        restored.create_frames(),
        vec![encode_create(&PickupSpec::native(2, [10.0, 0.0, 0.0]))]
    );
    assert_eq!(restored.create_frames(), restored.live_create_frames());

    let legacy: PickupSnapshot = serde_json::from_str("{}").unwrap();
    assert!(legacy.consumed.is_empty());
    let empty = PickupSet::restore(specs.clone(), None, legacy, 0, 600_000).unwrap();
    assert_eq!(empty.live_count(), 2);
    assert_eq!(
        empty.create_frames().len(),
        2,
        "a fresh set announces every box"
    );
    assert_eq!(empty.snapshot().consumed, Vec::<u32>::new());

    for consumed in [vec![1u32, 1u32], vec![9u32]] {
        let error = PickupSet::restore(
            specs.clone(),
            None,
            PickupSnapshot {
                consumed: consumed.clone(),
                ..Default::default()
            },
            0,
            600_000,
        )
        .unwrap_err();
        match consumed.as_slice() {
            [1, 1] => assert_eq!(error, PickupError::DuplicateConsumed { handle: 1 }),
            _ => assert_eq!(error, PickupError::UnknownHandle { handle: 9 }),
        }
    }
}

#[test]
fn foreign_nonfinite_and_far_claims_move_nothing() {
    let spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
    let mut set = PickupSet::new(vec![spec]).unwrap();
    assert_eq!(
        set.claim(9, 7, [0.0, 0.0, 0.0], 1_000, 600_000)
            .unwrap_err(),
        PickupRefusal::ForeignGuid {
            offered: 9,
            owned: 7
        }
    );
    assert_eq!(
        set.claim(7, 7, [f32::NAN, 0.0, 0.0], 1_100, 600_000)
            .unwrap_err(),
        PickupRefusal::NonFinitePosition
    );
    assert_eq!(
        set.claim(7, 7, [spec.radius + 0.01, 0.0, 0.0], 1_200, 600_000)
            .unwrap_err(),
        PickupRefusal::NoPickupInRange
    );
    assert_eq!(set.live_count(), 1);
    assert_eq!(set.snapshot().consumed, Vec::<u32>::new());
    // Resetting is the new-race operation: consumed marks clear, definitions stay.
    set.claim(7, 7, [0.0, 0.0, 0.0], 2_000, 600_000).unwrap();
    assert_eq!(set.live_count(), 0);
    set.reset();
    assert_eq!(set.live_count(), 1);
    assert!(!set.is_consumed(1));
    assert_eq!(set.specs(), &[spec]);
}

#[test]
fn respawn_returns_the_same_handle_after_the_delay_and_only_to_its_budget() {
    let spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
    let policy = PickupRespawn {
        delay_ms: 5_000,
        max_respawns: 1,
    };
    let mut set = PickupSet::with_respawn(vec![spec], Some(policy)).unwrap();
    assert_eq!(set.respawn(), Some(policy));
    let claimed = set
        .claim(7, 7, [0.0, 0.0, 0.0], 10_000, 20_000)
        .expect("owned claim inside the radius");
    assert_eq!(
        subtypes(&claimed.replies),
        vec![GRANT_SUBTYPE, DESTROY_SUBTYPE]
    );
    let snapshot = set.snapshot();
    assert_eq!(snapshot.consumed, vec![1]);
    assert_eq!(
        snapshot.pending,
        vec![PendingRespawn {
            handle: 1,
            due_ms: 15_000
        }]
    );
    assert!(snapshot.respawned.is_empty());
    assert_eq!(set.live_count(), 0);
    // Before the due time the box stays away; at the due time it returns with the same handle.
    assert!(set.due_respawns(14_999).is_empty());
    assert_eq!(set.snapshot().consumed, vec![1]);
    assert_eq!(set.due_respawns(15_000), vec![encode_create(&spec)]);
    assert_eq!(set.live_count(), 1);
    assert!(set.snapshot().consumed.is_empty());
    assert_eq!(set.snapshot().respawned, vec![1]);
    assert!(set.snapshot().pending.is_empty());
    // The second collection spends the only respawn; the box then stays gone.
    assert_eq!(
        subtypes(
            &set.claim(7, 7, [0.0, 0.0, 0.0], 16_000, 20_000)
                .unwrap()
                .replies
        ),
        vec![GRANT_SUBTYPE, DESTROY_SUBTYPE]
    );
    assert_eq!(set.snapshot().consumed, vec![1]);
    assert!(set.snapshot().pending.is_empty(), "the budget is spent");
    assert_eq!(
        set.claim(7, 7, [0.0, 0.0, 0.0], 17_000, 20_000)
            .unwrap_err(),
        PickupRefusal::NoPickupInRange
    );
    assert!(set.due_respawns(600_000).is_empty());
}

#[test]
fn respawn_policy_bounds_are_enforced() {
    let spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
    for (delay_ms, max_respawns, field) in [
        (MIN_RESPAWN_DELAY_MS - 1, 1, "delay_ms"),
        (MAX_RESPAWN_DELAY_MS + 1, 1, "delay_ms"),
        (MIN_RESPAWN_DELAY_MS, 0, "max_respawns"),
        (
            MIN_RESPAWN_DELAY_MS,
            MAX_RESPAWNS_PER_PICKUP + 1,
            "max_respawns",
        ),
    ] {
        assert_eq!(
            PickupSet::with_respawn(
                vec![spec],
                Some(PickupRespawn {
                    delay_ms,
                    max_respawns
                })
            )
            .unwrap_err(),
            PickupError::BadRespawnPolicy { field }
        );
    }
    // The boundary values are usable, and absent stays the legacy permanent behavior.
    assert!(
        PickupSet::with_respawn(
            vec![spec],
            Some(PickupRespawn {
                delay_ms: MIN_RESPAWN_DELAY_MS,
                max_respawns: MAX_RESPAWNS_PER_PICKUP
            })
        )
        .is_ok()
    );
    assert!(PickupSet::new(vec![spec]).unwrap().respawn().is_none());
}

#[test]
fn a_collection_too_late_in_the_race_stays_permanent() {
    let spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
    let policy = PickupRespawn {
        delay_ms: 5_000,
        max_respawns: 2,
    };
    // A due time exactly on the window end is still scheduled.
    let mut set = PickupSet::with_respawn(vec![spec], Some(policy)).unwrap();
    set.claim(7, 7, [0.0, 0.0, 0.0], 10_000, 15_000).unwrap();
    assert_eq!(set.snapshot().pending.len(), 1);
    // One millisecond later the return could never fire before the window closes.
    let mut late = PickupSet::with_respawn(vec![spec], Some(policy)).unwrap();
    late.claim(7, 7, [0.0, 0.0, 0.0], 10_001, 15_000).unwrap();
    assert!(late.snapshot().pending.is_empty());
    assert_eq!(late.snapshot().consumed, vec![1]);
    assert!(late.due_respawns(u64::MAX).is_empty());
}

#[test]
fn typed_items_accept_only_the_four_recovered_types() {
    for item_type in [
        MISSILE_ITEM_TYPE,
        MINE_ITEM_TYPE,
        SHIELD_ITEM_TYPE,
        BOOST_ITEM_TYPE,
    ] {
        assert!(is_recovered_item_type(item_type));
        let mut spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
        spec.item_type = item_type;
        assert!(
            PickupSet::new(vec![spec]).is_ok(),
            "type {item_type} must be accepted"
        );
    }
    for item_type in [0u32, 2, 6, u32::MAX] {
        assert!(!is_recovered_item_type(item_type));
        let mut spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
        spec.item_type = item_type;
        assert_eq!(
            PickupSet::new(vec![spec]).unwrap_err(),
            PickupError::UnsupportedItemType {
                handle: 1,
                item_type
            }
        );
    }
    // The granted type is the definition's, and the grant frame carries it verbatim.
    let mut shield = PickupSpec::native(9, [0.0, 0.0, 0.0]);
    shield.item_type = SHIELD_ITEM_TYPE;
    let mut set = PickupSet::new(vec![shield]).unwrap();
    let claimed = set.claim(1, 1, [0.0, 0.0, 0.0], 1_000, 600_000).unwrap();
    assert_eq!(claimed.item_type, SHIELD_ITEM_TYPE);
    assert_eq!(payload(&claimed.replies[0]), SHIELD_ITEM_TYPE.to_le_bytes());
}

#[test]
fn respawn_snapshot_round_trips_and_tampered_entries_are_refused() {
    let spec = PickupSpec::native(1, [0.0, 0.0, 0.0]);
    let policy = PickupRespawn {
        delay_ms: 8_000,
        max_respawns: 2,
    };
    let mut set = PickupSet::with_respawn(vec![spec], Some(policy)).unwrap();
    set.claim(7, 7, [0.0, 0.0, 0.0], 10_000, 20_000).unwrap();
    let snapshot = set.snapshot();
    let restored =
        PickupSet::restore(vec![spec], Some(policy), snapshot.clone(), 0, 600_000).unwrap();
    assert_eq!(restored, set);
    // Legacy snapshots restore empty with or without a policy.
    let legacy: PickupSnapshot = serde_json::from_str("{}").unwrap();
    let legacy_set = PickupSet::restore(vec![spec], Some(policy), legacy, 0, 600_000).unwrap();
    assert!(legacy_set.snapshot().consumed.is_empty());
    assert!(legacy_set.snapshot().pending.is_empty());
    assert!(PickupSet::restore(vec![spec], None, PickupSnapshot::default(), 0, 600_000).is_ok());
    // Tampering is refused: unknown handle, a return without a collection, a return outside the
    // race window, a duplicate pending return, and a fired count beyond the policy.
    for (tampered, expected) in [
        (
            PickupSnapshot {
                consumed: vec![1],
                pending: vec![PendingRespawn {
                    handle: 9,
                    due_ms: 12_000,
                }],
                ..Default::default()
            },
            PickupError::UnknownHandle { handle: 9 },
        ),
        (
            PickupSnapshot {
                pending: vec![PendingRespawn {
                    handle: 1,
                    due_ms: 12_000,
                }],
                ..Default::default()
            },
            PickupError::IncoherentRespawn { handle: 1 },
        ),
        (
            PickupSnapshot {
                consumed: vec![1],
                pending: vec![PendingRespawn {
                    handle: 1,
                    due_ms: 601_000,
                }],
                ..Default::default()
            },
            PickupError::IncoherentRespawn { handle: 1 },
        ),
        (
            PickupSnapshot {
                consumed: vec![1],
                pending: vec![
                    PendingRespawn {
                        handle: 1,
                        due_ms: 12_000,
                    },
                    PendingRespawn {
                        handle: 1,
                        due_ms: 13_000,
                    },
                ],
                ..Default::default()
            },
            PickupError::IncoherentRespawn { handle: 1 },
        ),
        (
            PickupSnapshot {
                respawned: vec![1, 1, 1],
                ..Default::default()
            },
            PickupError::IncoherentRespawn { handle: 1 },
        ),
    ] {
        assert_eq!(
            PickupSet::restore(vec![spec], Some(policy), tampered, 0, 600_000).unwrap_err(),
            expected
        );
    }
    // A return before the race even started is a tampered clock.
    assert_eq!(
        PickupSet::restore(
            vec![spec],
            Some(policy),
            PickupSnapshot {
                consumed: vec![1],
                pending: vec![PendingRespawn {
                    handle: 1,
                    due_ms: 9_999,
                }],
                ..Default::default()
            },
            10_000,
            600_000
        )
        .unwrap_err(),
        PickupError::IncoherentRespawn { handle: 1 }
    );
    // Without a policy any stored respawn state is a tamper.
    assert_eq!(
        PickupSet::restore(
            vec![spec],
            None,
            PickupSnapshot {
                respawned: vec![1],
                ..Default::default()
            },
            0,
            600_000
        )
        .unwrap_err(),
        PickupError::IncoherentRespawn { handle: 1 }
    );
}
