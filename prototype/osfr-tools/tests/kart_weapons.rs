//! Kart weapon codecs + bounded local lifecycle - owned vectors.
//!
//! The module under test lives at `src/kart_weapons.rs` and is compiled here through `#[path]`
//! because the lead owns the `lib.rs` registration; the file is self-contained (std + serde only),
//! so the copy under test is byte-identical to the one the lead integrates.
//!
//! Layout evidence: `lab/reports/kart-weapon-layouts-20261009/REPORT.md` plus the corrected reader
//! facts: row-major 4x4 with the translation in slots 12/13/14, owner-local create matrices, the
//! missile `26` update as a direct world matrix (`FUN_00fe2850` reads the full matrix) and the mine
//! `31` update as the inverse-owner local matrix. No explosion `27`/`28` is emitted by the
//! lifecycle. Hits are out of scope.

use osfr_tools::kart_weapons::*;

const OWNER: u64 = 0x4b52_cc6a_9573_ca91;
const RACE_START: u64 = 1_000;
const RACE_END: u64 = 600_000;

fn subtype(frame: &[u8]) -> u16 {
    u16::from_le_bytes([frame[2], frame[3]])
}

fn payload(frame: &[u8]) -> &[u8] {
    &frame[4..]
}

fn identity_pose(position: [f32; 3]) -> WeaponPose {
    WeaponPose {
        basis: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        position,
    }
}

fn observation(position: [f32; 3], at_ms: u64) -> WeaponObservation {
    WeaponObservation {
        pose: identity_pose(position),
        at_ms,
    }
}

#[test]
fn codec_payload_lengths_match_the_reader_report() {
    assert_eq!(MISSILE_CREATE_PAYLOAD_LEN, 4 + 4 + 16 * 4 + 4 + 8 + 4);
    assert_eq!(MISSILE_UPDATE_PAYLOAD_LEN, 4 + 8 + 16 * 4 + 16 + 16 + 8);
    assert_eq!(EXPLOSION_CREATE_PAYLOAD_LEN, 4 + 4 + 8 + 16 + 4 * 4);
    assert_eq!(MINE_CREATE_PAYLOAD_LEN, 4 + 4 + 16 * 4 + 8);
    assert_eq!(MINE_UPDATE_PAYLOAD_LEN, 4 + 16 * 4);
    assert_eq!(
        (
            MISSILE_CREATE_PAYLOAD_LEN,
            MISSILE_DESTROY_PAYLOAD_LEN,
            MISSILE_UPDATE_PAYLOAD_LEN,
            EXPLOSION_CREATE_PAYLOAD_LEN,
            EXPLOSION_DESTROY_PAYLOAD_LEN,
            MINE_CREATE_PAYLOAD_LEN,
            MINE_DESTROY_PAYLOAD_LEN,
            MINE_UPDATE_PAYLOAD_LEN,
        ),
        (88, 4, 116, 48, 4, 80, 4, 68)
    );
}

#[test]
fn codecs_round_trip_every_subtype_and_keep_the_offsets() {
    let matrix: [f32; 16] = std::array::from_fn(|index| index as f32);
    let missile_create = MissileCreate {
        id: 7,
        model: MISSILE_MODEL_ID,
        matrix,
        scalar_0x60: 1.0,
        field_0x68: OWNER,
        field_0x70: 0,
    };
    let encoded = encode_missile_create(&missile_create);
    assert_eq!(encoded.len(), MISSILE_CREATE_PAYLOAD_LEN);
    assert_eq!(&encoded[0..4], &7u32.to_le_bytes());
    assert_eq!(&encoded[4..8], &MISSILE_MODEL_ID.to_le_bytes());
    assert_eq!(&encoded[72..76], &1.0f32.to_le_bytes());
    assert_eq!(&encoded[76..84], &OWNER.to_le_bytes());
    assert_eq!(decode_missile_create(&encoded).unwrap(), missile_create);

    let missile_update = MissileUpdate {
        id: 8,
        field_0x10: 9,
        matrix,
        tail_0x60: [1.0, 2.0, 3.0, 4.0],
        tail_0x70: [5.0, 6.0, 7.0, 8.0],
        field_0x80: 11,
    };
    let encoded = encode_missile_update(&missile_update);
    assert_eq!(encoded.len(), MISSILE_UPDATE_PAYLOAD_LEN);
    assert_eq!(&encoded[0..4], &8u32.to_le_bytes());
    assert_eq!(&encoded[4..12], &9u64.to_le_bytes());
    assert_eq!(&encoded[12..16], &0.0f32.to_le_bytes()[..]);
    assert_eq!(&encoded[72..76], &matrix[15].to_le_bytes()[..]);
    assert_eq!(
        &encoded[76..92],
        &[1.0f32, 2.0, 3.0, 4.0].map(f32::to_le_bytes).concat()[..]
    );
    assert_eq!(&encoded[108..116], &11u64.to_le_bytes()[..]);
    assert_eq!(decode_missile_update(&encoded).unwrap(), missile_update);

    let explosion = ExplosionCreate {
        id: 9,
        field_0x10: 0,
        field_0x18: OWNER,
        block_0x20: [1.0, 2.0, 3.0, 1.0],
        scalar_0x30: 0.5,
        scalar_0x34: 0.25,
        scalar_0x38: 0.0,
        scalar_0x3c: 0.0,
    };
    let encoded = encode_explosion_create(&explosion);
    assert_eq!(encoded.len(), EXPLOSION_CREATE_PAYLOAD_LEN);
    assert_eq!(&encoded[4..8], &0u32.to_le_bytes());
    assert_eq!(&encoded[8..16], &OWNER.to_le_bytes());
    assert_eq!(decode_explosion_create(&encoded).unwrap(), explosion);

    let mine_create = MineCreate {
        id: 10,
        model: MINE_MODEL_ID,
        matrix,
        field_0x60: OWNER,
    };
    let encoded = encode_mine_create(&mine_create);
    assert_eq!(encoded.len(), MINE_CREATE_PAYLOAD_LEN);
    assert_eq!(&encoded[72..80], &OWNER.to_le_bytes());
    assert_eq!(decode_mine_create(&encoded).unwrap(), mine_create);

    let mine_update = MineUpdate { id: 11, matrix };
    let encoded = encode_mine_update(&mine_update);
    assert_eq!(encoded.len(), MINE_UPDATE_PAYLOAD_LEN);
    assert_eq!(&encoded[0..4], &11u32.to_le_bytes());
    assert_eq!(decode_mine_update(&encoded).unwrap(), mine_update);

    for subtype in [
        MISSILE_DESTROY_SUBTYPE,
        EXPLOSION_DESTROY_SUBTYPE,
        MINE_DESTROY_SUBTYPE,
    ] {
        let value = WeaponDestroy { id: 12 };
        let encoded = encode_destroy(&value);
        assert_eq!(encoded.len(), 4);
        assert_eq!(decode_destroy(subtype, &encoded).unwrap(), value);
    }
}

#[test]
fn inner_frames_wrap_opcode_and_subtype() {
    let payload = encode_mine_update(&MineUpdate {
        id: 3,
        matrix: [0.0; 16],
    });
    let frame = inner_frame(MINE_UPDATE_SUBTYPE, &payload);
    assert_eq!(&frame[0..2], &INNER_OPCODE.to_le_bytes());
    assert_eq!(&frame[2..4], &MINE_UPDATE_SUBTYPE.to_le_bytes());
    assert_eq!(frame.len(), 4 + MINE_UPDATE_PAYLOAD_LEN);
}

#[test]
fn decoders_refuse_wrong_lengths_and_non_finite_values() {
    let short = vec![0u8; 10];
    assert_eq!(
        decode_missile_create(&short).unwrap_err(),
        WeaponError::PayloadLength {
            subtype: MISSILE_CREATE_SUBTYPE,
            expected: MISSILE_CREATE_PAYLOAD_LEN,
            actual: 10,
        }
    );
    assert!(matches!(
        decode_missile_update(&short).unwrap_err(),
        WeaponError::PayloadLength {
            subtype: MISSILE_UPDATE_SUBTYPE,
            ..
        }
    ));
    assert!(matches!(
        decode_explosion_create(&short).unwrap_err(),
        WeaponError::PayloadLength {
            subtype: EXPLOSION_CREATE_SUBTYPE,
            ..
        }
    ));
    assert!(matches!(
        decode_mine_create(&short).unwrap_err(),
        WeaponError::PayloadLength {
            subtype: MINE_CREATE_SUBTYPE,
            ..
        }
    ));
    assert!(matches!(
        decode_mine_update(&short).unwrap_err(),
        WeaponError::PayloadLength {
            subtype: MINE_UPDATE_SUBTYPE,
            ..
        }
    ));
    assert!(matches!(
        decode_destroy(MISSILE_DESTROY_SUBTYPE, &short).unwrap_err(),
        WeaponError::PayloadLength { .. }
    ));

    let mut nan_payload = vec![0u8; MINE_UPDATE_PAYLOAD_LEN];
    nan_payload[4..8].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(matches!(
        decode_mine_update(&nan_payload).unwrap_err(),
        WeaponError::NonFinite { .. }
    ));
    let mut explosion = vec![0u8; EXPLOSION_CREATE_PAYLOAD_LEN];
    explosion[32..36].copy_from_slice(&f32::INFINITY.to_le_bytes());
    assert!(matches!(
        decode_explosion_create(&explosion).unwrap_err(),
        WeaponError::NonFinite { .. }
    ));
}

#[test]
fn pose_validation_requires_a_finite_orthonormal_right_handed_basis() {
    identity_pose([1.0, 2.0, 3.0]).validate().unwrap();
    assert_eq!(identity_pose([0.0; 3]).heading(), [0.0, 0.0, 1.0]);

    let mut non_finite = identity_pose([0.0; 3]);
    non_finite.basis[0][0] = f32::NAN;
    assert!(matches!(
        non_finite.validate().unwrap_err(),
        WeaponError::PoseNotFinite { .. }
    ));

    let mut scaled = identity_pose([0.0; 3]);
    scaled.basis[2][2] = 1.5;
    assert_eq!(
        scaled.validate().unwrap_err(),
        WeaponError::PoseNotOrthonormal
    );

    let left_handed = WeaponPose {
        basis: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]],
        position: [0.0; 3],
    };
    assert_eq!(
        left_handed.validate().unwrap_err(),
        WeaponError::PoseNotRightHanded
    );

    // The native sender is quiet at rest; a cached pose stays valid, but future stamps refuse.
    observation([0.0; 3], 10_000).validate_for(10_500).unwrap();
    observation([0.0; 3], 10_000).validate_for(55_000).unwrap();
    assert!(matches!(
        observation([0.0; 3], 10_001)
            .validate_for(10_000)
            .unwrap_err(),
        WeaponError::FuturePose { .. }
    ));
}

#[test]
fn use_item_writes_owner_words_and_owner_local_create_matrices() {
    let pose = identity_pose([100.0, 0.0, 0.0]);
    let observation = WeaponObservation {
        pose,
        at_ms: RACE_START,
    };
    let mut state = WeaponState::new();

    let frames = state
        .use_item(
            WeaponKind::Missile,
            OWNER,
            &observation,
            RACE_START,
            RACE_END,
        )
        .unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(subtype(&frames[0]), MISSILE_CREATE_SUBTYPE);
    let missile = decode_missile_create(payload(&frames[0])).unwrap();
    assert_eq!(missile.id, 1);
    assert_eq!(missile.model, MISSILE_MODEL_ID);
    // Owner-local: identity basis with the forward offset in the translation slots.
    assert_eq!(
        &missile.matrix[..12],
        &[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]
    );
    assert_eq!(
        &missile.matrix[12..],
        &[0.0, 0.0, MISSILE_CREATE_FORWARD_M, 1.0]
    );
    assert_eq!(missile.scalar_0x60, 1.0);
    assert_eq!(
        missile.field_0x68, OWNER,
        "the owner pair must be the session guid"
    );
    assert_eq!(missile.field_0x70, 0);

    let frames = state
        .use_item(WeaponKind::Mine, OWNER, &observation, RACE_START, RACE_END)
        .unwrap();
    assert_eq!(subtype(&frames[0]), MINE_CREATE_SUBTYPE);
    let mine = decode_mine_create(payload(&frames[0])).unwrap();
    assert_eq!(mine.id, 2);
    assert_eq!(mine.model, MINE_MODEL_ID);
    assert_eq!(&mine.matrix[12..], &[0.0, 0.0, MINE_CREATE_FORWARD_M, 1.0]);
    assert_eq!(mine.field_0x60, OWNER);
}

#[test]
fn missile_update_is_a_world_matrix_and_expires_with_a_terminal_point() {
    let owner = observation([100.0, 0.0, 0.0], RACE_START);
    let mut state = WeaponState::new();
    state
        .use_item(WeaponKind::Missile, OWNER, &owner, RACE_START, RACE_END)
        .unwrap();
    assert_eq!(state.live(), 1);
    assert_eq!(state.weapons[0].expires_ms, RACE_START + MISSILE_TTL_MS);

    // Nothing before the first 10 Hz slot.
    let fresh = observation([100.0, 0.0, 0.0], RACE_START + 99);
    assert!(state.tick(RACE_START + 99, Some(&fresh)).is_empty());
    // The first update carries the world matrix: identity basis and the position in slots 12..14.
    let fresh = observation([100.0, 0.0, 0.0], RACE_START + 100);
    let frames = state.tick(RACE_START + 100, Some(&fresh));
    assert_eq!(frames.len(), 1);
    assert_eq!(subtype(&frames[0]), MISSILE_UPDATE_SUBTYPE);
    let update = decode_missile_update(payload(&frames[0])).unwrap();
    assert_eq!(
        &update.matrix[..12],
        &[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]
    );
    let expected = MISSILE_SPEED_MPS * (MISSILE_UPDATE_INTERVAL_MS as f32 / 1_000.0);
    assert_eq!(
        &update.matrix[12..],
        &[100.0, 0.0, MISSILE_CREATE_FORWARD_M + expected, 0.0],
        "slot 15 stays 0 for updates (a stray 1.0 would read as 1 m/s linear velocity)"
    );
    // The two velocity triples stay zero for a pure position stream.
    assert_eq!(update.tail_0x60, [0.0; 4]);
    assert_eq!(update.tail_0x70, [0.0; 4]);

    // The owner moving does not change the missile's world matrix.
    let moved = WeaponObservation {
        pose: identity_pose([500.0, 0.0, 0.0]),
        at_ms: RACE_START + 200,
    };
    let frames = state.tick(RACE_START + 200, Some(&moved));
    let update = decode_missile_update(payload(&frames[0])).unwrap();
    assert_eq!(
        update.matrix[12], 100.0,
        "the missile stays where the owner fired it"
    );
    assert_eq!(update.matrix[13], 0.0);
    assert!((update.matrix[14] - (MISSILE_CREATE_FORWARD_M + 2.0 * expected)).abs() < 1e-3);

    // A late tick integrates only to the expiry, publishes that terminal point, then destroys once.
    let expiry = RACE_START + MISSILE_TTL_MS;
    let late = observation([100.0, 0.0, 0.0], expiry + 5_000);
    let frames = state.tick(expiry + 5_000, Some(&late));
    assert_eq!(
        frames
            .iter()
            .map(|frame| subtype(frame))
            .collect::<Vec<_>>(),
        vec![MISSILE_UPDATE_SUBTYPE, MISSILE_DESTROY_SUBTYPE],
        "terminal update then destroy, and no 27 explosion"
    );
    let terminal = decode_missile_update(payload(&frames[0])).unwrap();
    let full_flight = MISSILE_SPEED_MPS * (MISSILE_TTL_MS as f32 / 1_000.0);
    assert!((terminal.matrix[14] - (MISSILE_CREATE_FORWARD_M + full_flight)).abs() < 1e-2);
    assert_eq!(terminal.matrix[15], 0.0);
    assert!(state.is_empty());
    assert!(state.tick(expiry + 6_000, Some(&late)).is_empty());
}

#[test]
fn mine_update_is_inverse_owner_local_and_expires_once() {
    let owner = observation([100.0, 0.0, 0.0], RACE_START);
    let mut state = WeaponState::new();
    state
        .use_item(WeaponKind::Mine, OWNER, &owner, RACE_START, RACE_END)
        .unwrap();

    // The owner drives five metres forward: the mine keeps its world spot, so its local position
    // becomes -5 on the axis that left the owner - and it keeps the -3 local drop offset.
    let moved = observation([105.0, 0.0, 0.0], RACE_START + 100);
    let frames = state.tick(RACE_START + 100, Some(&moved));
    assert_eq!(frames.len(), 1);
    assert_eq!(subtype(&frames[0]), MINE_UPDATE_SUBTYPE);
    let update = decode_mine_update(payload(&frames[0])).unwrap();
    assert_eq!(
        &update.matrix[12..],
        &[-5.0, 0.0, MINE_CREATE_FORWARD_M, 1.0]
    );

    // Without a fresh observation a static mine emits nothing at all.
    let cached = observation([105.0, 0.0, 0.0], RACE_START);
    assert_eq!(state.tick(RACE_START + 1_200, Some(&cached)).len(), 1);
    let future = observation([105.0, 0.0, 0.0], RACE_START + 1_301);
    assert!(state.tick(RACE_START + 1_300, Some(&future)).is_empty());
    assert_eq!(state.live(), 1);

    // Expiry destroys once, with no update (mines do not integrate a terminal leg) and no 27.
    let expiry = RACE_START + MINE_TTL_MS;
    let late = observation([105.0, 0.0, 0.0], expiry);
    let frames = state.tick(expiry, Some(&late));
    assert_eq!(
        frames
            .iter()
            .map(|frame| subtype(frame))
            .collect::<Vec<_>>(),
        vec![MINE_DESTROY_SUBTYPE]
    );
    assert!(state.tick(expiry + 1_000, Some(&late)).is_empty());
}

#[test]
fn status_refuses_without_mutating_and_covers_every_guard() {
    let mut state = WeaponState::new();
    let owner = observation([0.0; 3], RACE_START);

    // At or past the race end.
    assert!(matches!(
        state
            .use_item(WeaponKind::Missile, OWNER, &owner, RACE_END, RACE_END)
            .unwrap_err(),
        WeaponError::RaceEnded { .. }
    ));
    // A zero owner guid.
    assert_eq!(
        state
            .use_item(WeaponKind::Missile, 0, &owner, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::ZeroOwner
    );
    // A regressed clock relative to the last lifecycle tick.
    state.tick(50_000, None);
    assert!(matches!(
        state
            .use_item(WeaponKind::Missile, OWNER, &owner, 40_000, RACE_END)
            .unwrap_err(),
        WeaponError::ClockRegression { .. }
    ));
    assert!(state.is_empty(), "every refusal left the state untouched");
    assert_eq!(state.next_id, 0);

    // Capacity: sixteen live missiles, the seventeenth refuses without allocating.
    let mut full = WeaponState::new();
    for index in 0..MAX_LIVE_WEAPONS {
        let at = RACE_START + index as u64;
        let observation = observation([0.0; 3], at);
        full.use_item(WeaponKind::Missile, OWNER, &observation, at, RACE_END)
            .unwrap();
    }
    let last = observation([0.0; 3], RACE_START + MAX_LIVE_WEAPONS as u64);
    assert!(matches!(
        full.use_item(
            WeaponKind::Missile,
            OWNER,
            &last,
            RACE_START + MAX_LIVE_WEAPONS as u64,
            RACE_END
        )
        .unwrap_err(),
        WeaponError::TooManyLive { .. }
    ));
    assert_eq!(full.next_id, MAX_LIVE_WEAPONS as u32);

    // The id budget itself.
    let mut exhausted = WeaponState::new();
    exhausted.next_id = MAX_TOTAL_ALLOCATIONS;
    assert!(matches!(
        exhausted
            .use_item(WeaponKind::Missile, OWNER, &owner, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::IdBudgetExhausted { .. }
    ));
}

#[test]
fn clear_keeps_the_id_counter_and_begin_resets_it() {
    let owner = observation([0.0; 3], RACE_START);
    let mut state = WeaponState::new();
    state
        .use_item(WeaponKind::Missile, OWNER, &owner, RACE_START, RACE_END)
        .unwrap();
    state.effects.push(EffectRegistration {
        id: 2,
        owner: OWNER,
        expires_ms: RACE_START + 1_000,
        position: [0.0, 0.0, 0.0, 1.0],
    });
    state.next_id = 2; // Synthetic effect id 2 must have a matching allocation history.
    let frames = state.clear();
    assert_eq!(
        frames
            .iter()
            .map(|frame| subtype(frame))
            .collect::<Vec<_>>(),
        vec![MISSILE_DESTROY_SUBTYPE, EXPLOSION_DESTROY_SUBTYPE]
    );
    assert!(state.is_empty());
    assert_eq!(state.next_id, 2, "clear keeps the allocation history");

    let next = observation([0.0; 3], RACE_START + 1_000);
    state
        .use_item(
            WeaponKind::Missile,
            OWNER,
            &next,
            RACE_START + 1_000,
            RACE_END,
        )
        .unwrap();
    assert_eq!(state.weapons[0].id, 3);

    state.begin();
    assert_eq!(state.next_id, 0, "only a new race resets the ids");
    let next = observation([0.0; 3], RACE_START + 2_000);
    state
        .use_item(
            WeaponKind::Missile,
            OWNER,
            &next,
            RACE_START + 2_000,
            RACE_END,
        )
        .unwrap();
    assert_eq!(state.weapons[0].id, 1);
}

#[test]
fn serde_defaults_round_trip_and_validate_for_refuses_forged_saves() {
    let default: WeaponState = serde_json::from_str("{}").unwrap();
    assert!(default.is_empty());
    assert_eq!(default.next_id, 0);
    default.validate_for(OWNER, RACE_START, RACE_END).unwrap();

    let owner = observation([0.0; 3], RACE_START);
    let mut state = WeaponState::new();
    state
        .use_item(WeaponKind::Missile, OWNER, &owner, RACE_START, RACE_END)
        .unwrap();
    state.tick(RACE_START + 100, Some(&owner));
    let json = serde_json::to_string(&state).unwrap();
    let restored: WeaponState = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, state);
    restored.validate_for(OWNER, RACE_START, RACE_END).unwrap();

    // Forged owner, id, window, span, basis and clock are all refused.
    let mut foreign = state.clone();
    foreign.weapons[0].owner = OWNER + 1;
    assert_eq!(
        foreign
            .validate_for(OWNER, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::ForeignOwner {
            owner: OWNER + 1,
            expected: OWNER
        }
    );
    let mut zero_id = state.clone();
    zero_id.weapons[0].id = 0;
    assert_eq!(
        zero_id
            .validate_for(OWNER, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::IdOutOfRange { id: 0 }
    );
    let mut over_ttl = state.clone();
    over_ttl.weapons[0].expires_ms = over_ttl.weapons[0].spawned_ms + MAX_WEAPON_TTL_MS + 1;
    assert!(matches!(
        over_ttl
            .validate_for(OWNER, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::BadTtlSpan { .. }
    ));
    let mut before_start = state.clone();
    before_start.weapons[0].spawned_ms = RACE_START - 1;
    assert!(matches!(
        before_start
            .validate_for(OWNER, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::BadTtlSpan { .. }
    ));
    let mut next_update = state.clone();
    next_update.weapons[0].next_update_ms = next_update.weapons[0].expires_ms + 1;
    assert!(matches!(
        next_update
            .validate_for(OWNER, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::OutsideWindow { .. }
    ));
    let mut forged_basis = state.clone();
    forged_basis.weapons[0].basis[2][2] = 9.0;
    assert_eq!(
        forged_basis
            .validate_for(OWNER, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::PoseNotOrthonormal
    );
    let mut forged_clock = state.clone();
    forged_clock.last_tick_ms = RACE_END + 1;
    assert_eq!(
        forged_clock
            .validate_for(OWNER, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::BadClock {
            last_tick_ms: RACE_END + 1
        }
    );
    let mut duplicate = state.clone();
    duplicate.effects.push(EffectRegistration {
        id: duplicate.weapons[0].id,
        owner: OWNER,
        expires_ms: RACE_START + 1_000,
        position: [0.0; 4],
    });
    assert!(matches!(
        duplicate
            .validate_for(OWNER, RACE_START, RACE_END)
            .unwrap_err(),
        WeaponError::DuplicateId { .. }
    ));
}

#[test]
fn lifecycle_never_emits_an_explosion_or_a_hit_frame() {
    let owner = observation([0.0; 3], RACE_START);
    let mut state = WeaponState::new();
    let mut frames = state
        .use_item(WeaponKind::Missile, OWNER, &owner, RACE_START, RACE_END)
        .unwrap();
    let expiry = RACE_START + MISSILE_TTL_MS;
    let late = observation([0.0; 3], expiry);
    frames.extend(state.tick(expiry, Some(&late)));
    let mut mine = WeaponState::new();
    mine.use_item(WeaponKind::Mine, OWNER, &owner, RACE_START, RACE_END)
        .unwrap();
    frames.extend(mine.tick(
        RACE_START + MINE_TTL_MS,
        Some(&observation([0.0; 3], RACE_START + MINE_TTL_MS)),
    ));
    frames.extend(mine.clear());
    let subtypes: Vec<u16> = frames.iter().map(|frame| subtype(frame)).collect();
    for subtype in &subtypes {
        assert!(
            matches!(
                *subtype,
                MISSILE_CREATE_SUBTYPE
                    | MISSILE_DESTROY_SUBTYPE
                    | MISSILE_UPDATE_SUBTYPE
                    | EXPLOSION_CREATE_SUBTYPE
                    | EXPLOSION_DESTROY_SUBTYPE
                    | MINE_CREATE_SUBTYPE
                    | MINE_DESTROY_SUBTYPE
                    | MINE_UPDATE_SUBTYPE
            ),
            "unexpected subtype {subtype}"
        );
        assert_ne!(*subtype, EXPLOSION_CREATE_SUBTYPE, "27 stays deferred");
        assert_ne!(*subtype, EXPLOSION_DESTROY_SUBTYPE, "28 stays deferred");
    }
    assert!(subtypes.contains(&MISSILE_DESTROY_SUBTYPE));
    assert!(subtypes.contains(&MINE_DESTROY_SUBTYPE));
}
