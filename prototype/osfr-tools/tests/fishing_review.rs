//! Independent review harness for MMO003 fishing (`src/fishing.rs`).
//!
//! Owned by the independent reviewer; the module under review is not edited.
//! Every field order and body size asserted here was re-derived from the
//! preserved client pseudocode (`lab/decompiled/client/FreeRealms/pseudocode.c`)
//! and the preserved server enum
//! (`lab/decompiled/server/LibSOE/SOE/BaseFishingPackets.cs`):
//!
//! - envelope: two little-endian u16 (opcode 138, sub-opcode) `FUN_00b66830` /
//!   `FUN_00b69550`;
//! - actor handle: two u32 (`FUN_008fb810`);
//! - vector: four f32 (`FUN_0091a640` write, `FUN_008e2410` read);
//! - requests: `FUN_00b6a400` (2), `FUN_00b6a380` (5), `FUN_00b6a460` (6),
//!   `FUN_00b6a4d0` (7), `FUN_00b6b510` (15);
//! - server readers: `FUN_00b66890` (1), `FUN_00b668f0` (5 echo),
//!   `FUN_00b66990` (7 echo), `FUN_00b66a00` (8), `FUN_00b66a80` (10),
//!   `FUN_00b68810` (12), `FUN_00b66b80` (13), `FUN_00b66be0` (14).
//!
//! Bodies are built byte by byte with `to_le_bytes`, not through the module's
//! own encoder, so an encoder/decoder pair that shares a wrong field order
//! still fails.

use osfr_tools::fishing::{
    ClientRequest, Error, Handle, MAX_BLOB, MAX_BODY, OPCODE, Outcome, Phase, ResultBody,
    ServerPacket, Session, Sub, Vec4,
};

const ACTOR: Handle = Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};
const OTHER: Handle = Handle {
    id: 0xdead_beef,
    seq: 0x0bad_f00d,
};

/// `FUN_008fb810`: eight bytes, id then seq, little-endian.
fn handle_bytes(h: Handle) -> Vec<u8> {
    let mut out = Vec::with_capacity(8);
    out.extend(h.id.to_le_bytes());
    out.extend(h.seq.to_le_bytes());
    out
}

/// `FUN_0091a640`: four f32 in order.
fn vec_bytes(vals: [f32; 4]) -> Vec<u8> {
    let mut out = Vec::with_capacity(16);
    for v in vals {
        out.extend(v.to_le_bytes());
    }
    out
}

/// `FUN_00b69550`: opcode then sub-opcode, both u16 little-endian.
fn frame(sub: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(4);
    out.extend(OPCODE.to_le_bytes());
    out.extend(sub.to_le_bytes());
    out
}

fn vec4() -> Vec4 {
    Vec4 {
        x: 1.5,
        y: -2.25,
        z: 0.0,
        w: 1.0,
    }
}

/// Minimum `FishingResult` body: every variable-length run empty.
/// 8 + 4 + 1 + 4 + 4 + 4 + 4 + 4 + 4 + 16 + 4 + 4 + 4 + 4 + 1 + 4 = 74 bytes.
fn minimal_result_body() -> Vec<u8> {
    let mut b = Vec::with_capacity(74);
    b.extend(handle_bytes(ACTOR));
    b.extend(0u32.to_le_bytes()); // at_18
    b.push(0); // at_1c
    b.extend(0u32.to_le_bytes()); // at_20
    b.extend(0u32.to_le_bytes()); // at_24
    b.extend(0u32.to_le_bytes()); // blob_2c, zero length
    b.extend(1.0f32.to_le_bytes()); // at_38
    b.extend(0u32.to_le_bytes()); // at_3c
    b.extend(1.0f32.to_le_bytes()); // at_40
    for _ in 0..4 {
        b.extend(0u32.to_le_bytes()); // at_44..at_50
    }
    b.extend(0u32.to_le_bytes()); // blob_58, zero length
    b.extend(0u32.to_le_bytes()); // blob_64, zero length
    b.extend(0u32.to_le_bytes()); // at_74
    b.extend(1.0f32.to_le_bytes()); // at_7c
    b.push(0); // at_80
    b.extend(0u32.to_le_bytes()); // at_78
    b
}

/// `FUN_00b66be0` read order with one distinct value per field so a
/// transposition cannot cancel out.
fn distinctive_result_body() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend(handle_bytes(ACTOR));
    b.extend(0xA1A2_A3A4u32.to_le_bytes()); // at_18
    b.push(1); // at_1c
    b.extend(0xB1B2_B3B4u32.to_le_bytes()); // at_20
    b.extend(0xC1C2_C3C4u32.to_le_bytes()); // at_24
    b.extend(3u32.to_le_bytes());
    b.extend([0x2c, 0x2d, 0x2e]); // blob_2c
    b.extend(1.25f32.to_le_bytes()); // at_38
    b.extend(0xD1D2_D3D4u32.to_le_bytes()); // at_3c
    b.extend(2.5f32.to_le_bytes()); // at_40
    b.extend(0xE1E2_E3E4u32.to_le_bytes()); // at_44
    b.extend(0xF1F2_F3F4u32.to_le_bytes()); // at_48
    b.extend(0x0102_0304u32.to_le_bytes()); // at_4c
    b.extend(0x0506_0708u32.to_le_bytes()); // at_50
    b.extend(2u32.to_le_bytes());
    b.extend([0x58, 0x59]); // blob_58
    b.extend(4u32.to_le_bytes());
    b.extend([0x64, 0x65, 0x66, 0x67]); // blob_64
    b.extend(0x0A0B_0C0Du32.to_le_bytes()); // at_74
    b.extend(3.75f32.to_le_bytes()); // at_7c
    b.push(1); // at_80
    b.extend(0x0E0F_1011u32.to_le_bytes()); // at_78
    b
}

fn patch_f32(body: &mut [u8], offset: usize, value: f32) {
    body[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn xorshift(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

#[test]
fn native_envelope_and_register_field_order() {
    // FUN_00b66830/FUN_00b69550: opcode 138 then sub-opcode; FUN_00b6a400:
    // handle then one flag byte.
    let mut bytes = frame(2);
    bytes.extend(handle_bytes(ACTOR));
    bytes.push(7);
    assert_eq!(bytes[..4], [0x8a, 0x00, 0x02, 0x00]);
    assert_eq!(bytes.len(), 13);
    assert_eq!(
        ClientRequest::decode(&bytes).unwrap(),
        ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 7
        }
    );
    assert_eq!(
        ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 7
        }
        .encode(),
        bytes
    );
}

#[test]
fn native_cast_anim_and_cast_field_order() {
    // FUN_00b6a380: handle, float distance at body+8, vector at body+12.
    let mut anim = frame(5);
    anim.extend(handle_bytes(ACTOR));
    anim.extend(12.5f32.to_le_bytes());
    anim.extend(vec_bytes([1.0, 2.0, 3.0, 4.0]));
    assert_eq!(anim.len(), 32);
    match ClientRequest::decode(&anim).unwrap() {
        ClientRequest::CastAnim {
            actor,
            distance,
            position,
        } => {
            assert_eq!(actor, ACTOR);
            assert_eq!(distance, 12.5);
            assert_eq!(
                position,
                Vec4 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                    w: 4.0
                }
            );
        }
        other => panic!("expected cast animation, got {other:?}"),
    }
    // Field-order sensitivity: touching the distance cannot move the vector.
    let mut flipped = anim.clone();
    flipped[12] ^= 0x80;
    match ClientRequest::decode(&flipped).unwrap() {
        ClientRequest::CastAnim {
            distance, position, ..
        } => {
            assert_ne!(distance, 12.5);
            assert_eq!(position.x, 1.0);
        }
        other => panic!("expected cast animation, got {other:?}"),
    }
    assert_eq!(
        ClientRequest::CastAnim {
            actor: ACTOR,
            distance: 12.5,
            position: Vec4 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                w: 4.0
            }
        }
        .encode(),
        anim
    );

    // Bit-level losslessness for a refused value: a signalling NaN payload
    // must survive decode/encode unchanged, not be canonicalized.
    const NAN_BITS: u32 = 0x7fc0_1234;
    let mut nan_anim = frame(5);
    nan_anim.extend(handle_bytes(ACTOR));
    nan_anim.extend(NAN_BITS.to_le_bytes());
    nan_anim.extend(vec_bytes([1.0, 2.0, 3.0, 4.0]));
    let decoded = ClientRequest::decode(&nan_anim).unwrap();
    match decoded {
        ClientRequest::CastAnim { distance, .. } => {
            assert_eq!(distance.to_bits(), NAN_BITS);
        }
        other => panic!("expected cast animation, got {other:?}"),
    }
    assert_eq!(decoded.encode(), nan_anim);

    // FUN_00b6a460: handle, vector at body+8, flag at body+24.
    let mut cast = frame(6);
    cast.extend(handle_bytes(ACTOR));
    cast.extend(vec_bytes([9.0, 8.0, 7.0, 6.0]));
    cast.push(3);
    assert_eq!(cast.len(), 29);
    assert_eq!(
        ClientRequest::decode(&cast).unwrap(),
        ClientRequest::Cast {
            actor: ACTOR,
            target: Vec4 {
                x: 9.0,
                y: 8.0,
                z: 7.0,
                w: 6.0
            },
            flag: 3
        }
    );

    // FUN_00b6a4d0: handle then flag (13 bytes); FUN_00b6b510: handle only.
    let mut reel = frame(7);
    reel.extend(handle_bytes(ACTOR));
    reel.push(1);
    assert_eq!(reel.len(), 13);
    assert_eq!(
        ClientRequest::decode(&reel).unwrap(),
        ClientRequest::ReelIn {
            actor: ACTOR,
            flag: 1
        }
    );
    let mut special = frame(15);
    special.extend(handle_bytes(ACTOR));
    assert_eq!(special.len(), 12);
    assert_eq!(
        ClientRequest::decode(&special).unwrap(),
        ClientRequest::Special { actor: ACTOR }
    );
}

#[test]
fn native_result_body_read_order_is_preserved() {
    let body = distinctive_result_body();
    let packet = ServerPacket::decode_known(Sub::Result, &body).unwrap();
    assert_eq!(packet.sub(), Sub::Result);
    match packet {
        ServerPacket::Result { body } => {
            assert_eq!(body.actor, ACTOR);
            assert_eq!(body.at_18, 0xA1A2_A3A4);
            assert!(body.at_1c);
            assert_eq!(body.at_20, 0xB1B2_B3B4);
            assert_eq!(body.at_24, 0xC1C2_C3C4);
            assert_eq!(body.blob_2c, [0x2c, 0x2d, 0x2e]);
            assert_eq!(body.at_38, 1.25);
            assert_eq!(body.at_3c, 0xD1D2_D3D4);
            assert_eq!(body.at_40, 2.5);
            assert_eq!(body.at_44, 0xE1E2_E3E4);
            assert_eq!(body.at_48, 0xF1F2_F3F4);
            assert_eq!(body.at_4c, 0x0102_0304);
            assert_eq!(body.at_50, 0x0506_0708);
            assert_eq!(body.blob_58, [0x58, 0x59]);
            assert_eq!(body.blob_64, [0x64, 0x65, 0x66, 0x67]);
            assert_eq!(body.at_74, 0x0A0B_0C0D);
            assert_eq!(body.at_7c, 3.75);
            assert!(body.at_80);
            // Read last, from its own trailing dword after the bool.
            assert_eq!(body.at_78, 0x0E0F_1011);
            assert!(body.valid());
        }
        other => panic!("expected result, got {other:?}"),
    }
    // Exact consumption: one trailing byte is refused.
    let mut trailing = body;
    trailing.push(0);
    assert!(matches!(
        ServerPacket::decode_known(Sub::Result, &trailing),
        Err(Error::TrailingBytes)
    ));
}

#[test]
fn result_prefixes_are_rejected_without_panicking() {
    let full = minimal_result_body();
    assert_eq!(full.len(), 74);
    assert!(ServerPacket::decode_known(Sub::Result, &full).is_ok());
    for len in 0..full.len() {
        assert!(
            ServerPacket::decode_known(Sub::Result, &full[..len]).is_err(),
            "prefix of {len} bytes decoded as a full result"
        );
    }
}

#[test]
fn result_blob_lengths_are_bounded() {
    // blob_2c length field at body offset 21.
    let mut oversized = minimal_result_body();
    oversized[21..25].copy_from_slice(&((MAX_BLOB + 1) as u32).to_le_bytes());
    assert!(matches!(
        ServerPacket::decode_known(Sub::Result, &oversized),
        Err(Error::Oversize)
    ));
    // u32::MAX must be refused before any length arithmetic or allocation.
    let mut absurd = minimal_result_body();
    absurd[21..25].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        ServerPacket::decode_known(Sub::Result, &absurd),
        Err(Error::Oversize)
    ));
    // A within-bound length that overruns the body is truncation, not a panic.
    let mut truncated = minimal_result_body();
    truncated[21..25].copy_from_slice(&10u32.to_le_bytes());
    truncated.truncate(26);
    assert!(matches!(
        ServerPacket::decode_known(Sub::Result, &truncated),
        Err(Error::Truncated)
    ));
}

#[test]
fn decode_known_enforces_the_decoder_body_bound() {
    let oversized = vec![0u8; MAX_BODY + 1];
    assert!(matches!(
        ServerPacket::decode_known(Sub::SpawnFishRun, &oversized),
        Err(Error::Oversize)
    ));
    assert!(matches!(
        ServerPacket::decode_known(Sub::Result, &oversized),
        Err(Error::Oversize)
    ));
    // Unrecovered bodies stay byte-lossless within the bound.
    let exact = vec![0x5au8; MAX_BODY];
    match ServerPacket::decode_known(Sub::FishInfoUpdate, &exact).unwrap() {
        ServerPacket::Opaque { sub, body } => {
            assert_eq!(sub, Sub::FishInfoUpdate);
            assert_eq!(body, exact);
        }
        other => panic!("expected opaque body, got {other:?}"),
    }
    for sub in [
        Sub::RegisterPlayerResponse,
        Sub::SpawnSchool,
        Sub::SpecialResponse,
        Sub::SpawnFishRun,
    ] {
        match ServerPacket::decode_known(sub, &[1, 2, 3]).unwrap() {
            ServerPacket::Opaque { sub: got, body } => {
                assert_eq!(got, sub);
                assert_eq!(body, [1, 2, 3]);
            }
            other => panic!("expected opaque body, got {other:?}"),
        }
    }
}

#[test]
fn deterministic_byte_scan_never_panics_and_round_trips() {
    let mut state = 0x2026_1009_0bad_f00du64;
    let subs: Vec<Sub> = (0u16..=18).filter_map(Sub::from_u16).collect();
    assert_eq!(subs.len(), 16, "all 16 recovered sub-opcodes decode");
    for len in 0..=(MAX_BODY + 64) {
        let mut bytes = vec![0u8; len];
        for byte in &mut bytes {
            *byte = (xorshift(&mut state) & 0xff) as u8;
        }
        if let Ok(request) = ClientRequest::decode(&bytes) {
            assert_eq!(ClientRequest::decode(&request.encode()).unwrap(), request);
        }
        for sub in &subs {
            if let Ok(packet) = ServerPacket::decode_known(*sub, &bytes) {
                assert_eq!(packet.sub(), *sub);
                if matches!(packet, ServerPacket::Opaque { .. }) {
                    assert!(bytes.len() <= MAX_BODY);
                }
            }
        }
    }
}

#[test]
fn request_bodies_are_exact_length() {
    let bodies: [(u16, Vec<u8>); 5] = [
        (2, {
            let mut b = handle_bytes(ACTOR);
            b.push(0);
            b
        }),
        (5, {
            let mut b = handle_bytes(ACTOR);
            b.extend(1.0f32.to_le_bytes());
            b.extend(vec_bytes([1.0, 2.0, 3.0, 4.0]));
            b
        }),
        (6, {
            let mut b = handle_bytes(ACTOR);
            b.extend(vec_bytes([1.0, 2.0, 3.0, 4.0]));
            b.push(0);
            b
        }),
        (7, {
            let mut b = handle_bytes(ACTOR);
            b.push(0);
            b
        }),
        (15, handle_bytes(ACTOR)),
    ];
    for (sub, body) in bodies {
        let mut good = frame(sub);
        good.extend(&body);
        assert!(ClientRequest::decode(&good).is_ok(), "sub {sub} body");
        // Empty body, short actor, and one trailing byte are all refused.
        assert!(matches!(
            ClientRequest::decode(&frame(sub)),
            Err(Error::Truncated)
        ));
        let mut short = frame(sub);
        short.extend(&body[..body.len() - 1]);
        assert!(matches!(
            ClientRequest::decode(&short),
            Err(Error::Truncated)
        ));
        let mut trailing = good;
        trailing.push(0);
        assert!(matches!(
            ClientRequest::decode(&trailing),
            Err(Error::TrailingBytes)
        ));
    }
}

#[test]
fn request_decode_rejects_non_request_sub_opcodes() {
    // 3/4/11/16/18 are recovered server-side packets, not client requests.
    let body = vec![0u8; 16];
    for code in [3u16, 4, 11, 16, 18, 9, 17, 0] {
        let mut bytes = frame(code);
        bytes.extend(&body);
        assert!(
            matches!(ClientRequest::decode(&bytes), Err(Error::UnknownSub(got)) if got == code),
            "sub {code} should not decode as a client request"
        );
    }
}

#[test]
fn actor_accessor_matches_every_request_variant() {
    let requests = [
        ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 0,
        },
        ClientRequest::CastAnim {
            actor: ACTOR,
            distance: 1.0,
            position: vec4(),
        },
        ClientRequest::Cast {
            actor: ACTOR,
            target: vec4(),
            flag: 0,
        },
        ClientRequest::ReelIn {
            actor: ACTOR,
            flag: 0,
        },
        ClientRequest::Special { actor: ACTOR },
    ];
    for request in requests {
        assert_eq!(request.actor(), ACTOR);
    }
}

#[test]
fn session_binding_lifecycle() {
    let mut session = Session::new();
    assert_eq!(session.actor(), None);
    assert!(!session.registered());
    assert_eq!(session.phase(), Phase::Idle);
    assert!(matches!(
        session.request(&ClientRequest::Cast {
            actor: ACTOR,
            target: vec4(),
            flag: 0
        }),
        Err(Error::OutOfOrder)
    ));
    assert_eq!(
        session
            .request(&ClientRequest::RegisterPlayer {
                actor: ACTOR,
                flag: 0
            })
            .unwrap(),
        Outcome::Applied
    );
    assert_eq!(session.actor(), Some(ACTOR));
    assert!(session.registered());
    // Reconstructed duplicate policy: same actor re-registering is a no-op.
    assert_eq!(
        session
            .request(&ClientRequest::RegisterPlayer {
                actor: ACTOR,
                flag: 1
            })
            .unwrap(),
        Outcome::Duplicate
    );
    assert_eq!(session.actor(), Some(ACTOR));
    // A second actor cannot displace the binding or act.
    assert!(matches!(
        session.request(&ClientRequest::RegisterPlayer {
            actor: OTHER,
            flag: 0
        }),
        Err(Error::ForeignActor)
    ));
    assert!(matches!(
        session.request(&ClientRequest::Special { actor: OTHER }),
        Err(Error::ForeignActor)
    ));
    assert_eq!(session.actor(), Some(ACTOR));
    // reset unbinds so a new player may register.
    session.reset();
    assert_eq!(session.actor(), None);
    assert!(!session.registered());
    assert_eq!(session.phase(), Phase::Idle);
    assert_eq!(
        session
            .request(&ClientRequest::RegisterPlayer {
                actor: OTHER,
                flag: 0
            })
            .unwrap(),
        Outcome::Applied
    );
    assert_eq!(session.actor(), Some(OTHER));
}

#[test]
fn refused_requests_leave_the_session_untouched() {
    let mut session = Session::new();
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 0,
        })
        .unwrap();
    session
        .request(&ClientRequest::Cast {
            actor: ACTOR,
            target: vec4(),
            flag: 0,
        })
        .unwrap();
    assert_eq!(session.phase(), Phase::Casting);

    let refusals = [
        // Non-finite distance and non-finite vector components.
        ClientRequest::CastAnim {
            actor: ACTOR,
            distance: f32::NAN,
            position: vec4(),
        },
        ClientRequest::CastAnim {
            actor: ACTOR,
            distance: f32::INFINITY,
            position: vec4(),
        },
        ClientRequest::CastAnim {
            actor: ACTOR,
            distance: -0.5,
            position: vec4(),
        },
        ClientRequest::CastAnim {
            actor: ACTOR,
            distance: 1.0,
            position: Vec4 {
                z: f32::NEG_INFINITY,
                ..vec4()
            },
        },
        ClientRequest::Cast {
            actor: ACTOR,
            target: Vec4 {
                w: f32::NAN,
                ..vec4()
            },
            flag: 0,
        },
        // Foreign actor on an otherwise valid request.
        ClientRequest::ReelIn {
            actor: OTHER,
            flag: 0,
        },
    ];
    for request in refusals {
        assert!(matches!(
            session.request(&request),
            Err(Error::InvalidValue | Error::ForeignActor)
        ));
        assert_eq!(session.phase(), Phase::Casting);
        assert_eq!(session.actor(), Some(ACTOR));
        assert!(session.registered());
    }

    // Value validation runs before the duplicate short-circuit: an invalid
    // duplicate cast is an invalid value, not a duplicate.
    assert!(matches!(
        session.request(&ClientRequest::Cast {
            actor: ACTOR,
            target: Vec4 {
                x: f32::NAN,
                ..vec4()
            },
            flag: 0
        }),
        Err(Error::InvalidValue)
    ));
    assert_eq!(session.phase(), Phase::Casting);
}

#[test]
fn invalid_results_do_not_mutate_the_session() {
    let mut session = Session::new();
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 0,
        })
        .unwrap();
    session.hooked();
    assert_eq!(session.phase(), Phase::Reeling);
    for patch in [25usize, 33, 65] {
        let mut body = minimal_result_body();
        patch_f32(&mut body, patch, f32::NAN);
        let packet = ServerPacket::decode_known(Sub::Result, &body).unwrap();
        match &packet {
            ServerPacket::Result { body } => assert!(!body.valid()),
            other => panic!("expected result, got {other:?}"),
        }
        assert!(matches!(session.server(&packet), Err(Error::InvalidValue)));
        assert_eq!(session.phase(), Phase::Reeling);
        assert_eq!(session.actor(), Some(ACTOR));
        assert!(session.registered());
    }
}

// REVIEW FINDING R1, CLOSED 2026-10-09 (was
// `replayed_result_is_reapplied_without_a_nonce`): a repeated result used to
// re-apply because results carry no sequence or nonce. `Session::server` now
// accepts a result only from `Phase::Reeling`, the phase an owned reel-in
// produces, and consumes it one-shot by landing. The original falsifying probe
// asserted `Err(OutOfOrder)` and failed against the old code
// (lab/reports/fishing-review-20261009T024626Z/red-probe-cargo-test-fishing-review.log);
// it now passes.
#[test]
fn replayed_result_is_refused_once_landed() {
    let mut session = Session::new();
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 0,
        })
        .unwrap();
    session
        .request(&ClientRequest::Cast {
            actor: ACTOR,
            target: vec4(),
            flag: 0,
        })
        .unwrap();
    session.hooked();
    session
        .request(&ClientRequest::ReelIn {
            actor: ACTOR,
            flag: 0,
        })
        .unwrap();
    let packet = ServerPacket::decode_known(Sub::Result, &minimal_result_body()).unwrap();
    assert_eq!(session.server(&packet).unwrap(), Some(Phase::Landing));
    assert_eq!(session.phase(), Phase::Landing);
    // Landing keeps the registration and the actor binding so a cancel can
    // re-arm the session for the next cast.
    assert!(session.registered());
    assert_eq!(session.actor(), Some(ACTOR));
    // One-shot: with no nonce on the wire, the consumed landing is the
    // correlation the server has, so the same body is refused.
    assert!(matches!(session.server(&packet), Err(Error::OutOfOrder)));
    assert_eq!(session.phase(), Phase::Landing);
}

// REVIEW FINDING R2, CLOSED 2026-10-09 (was
// `result_lands_while_idle_after_registration`): the tracker used to accept a
// valid result while the session was still Idle after registration, before any
// cast, so landing was not gated on hook or reel-in state. The same gate
// refuses it now; the original falsifying probe asserted `Err(OutOfOrder)` and
// failed with `Ok(Some(Landing))` against the old code.
#[test]
fn result_is_refused_while_idle_after_registration() {
    let mut session = Session::new();
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 0,
        })
        .unwrap();
    let packet = ServerPacket::decode_known(Sub::Result, &minimal_result_body()).unwrap();
    assert_eq!(session.phase(), Phase::Idle);
    assert!(matches!(session.server(&packet), Err(Error::OutOfOrder)));
    assert_eq!(session.phase(), Phase::Idle);
    assert!(session.registered());
}

// Regression addition for the R1/R2 fix: only the full owned sequence reaches
// Landing, and not even `cancel` (which re-arms the next cast) lets the stale
// body apply again.
#[test]
fn only_an_owned_cast_hook_reel_sequence_lands_a_result() {
    let mut session = Session::new();
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 0,
        })
        .unwrap();
    let packet = ServerPacket::decode_known(Sub::Result, &minimal_result_body()).unwrap();
    // Registration alone is not evidence; neither is a cast before the bite.
    assert!(matches!(session.server(&packet), Err(Error::OutOfOrder)));
    session
        .request(&ClientRequest::Cast {
            actor: ACTOR,
            target: vec4(),
            flag: 0,
        })
        .unwrap();
    assert!(matches!(session.server(&packet), Err(Error::OutOfOrder)));
    // A reel-in without a hook is refused, so no result follows it either.
    assert!(matches!(
        session.request(&ClientRequest::ReelIn {
            actor: ACTOR,
            flag: 0
        }),
        Err(Error::OutOfOrder)
    ));
    session.hooked();
    assert_eq!(
        session
            .request(&ClientRequest::ReelIn {
                actor: ACTOR,
                flag: 0
            })
            .unwrap(),
        Outcome::Applied
    );
    assert_eq!(session.server(&packet).unwrap(), Some(Phase::Landing));
    // Cancel re-arms the next cast but does not bless the already-consumed
    // body, and a new cast still needs its own hook and reel.
    session.cancel();
    assert_eq!(session.phase(), Phase::Idle);
    assert!(matches!(session.server(&packet), Err(Error::OutOfOrder)));
    session
        .request(&ClientRequest::Cast {
            actor: ACTOR,
            target: vec4(),
            flag: 0,
        })
        .unwrap();
    assert!(matches!(session.server(&packet), Err(Error::OutOfOrder)));
    assert_eq!(session.phase(), Phase::Casting);
}

#[test]
fn foreign_and_unbound_results_are_refused() {
    let mut session = Session::new();
    let mut foreign = minimal_result_body();
    foreign[..8].copy_from_slice(&handle_bytes(OTHER));
    let packet = ServerPacket::decode_known(Sub::Result, &foreign).unwrap();
    // No registration yet: nobody is bound, so every result is foreign.
    assert!(matches!(session.server(&packet), Err(Error::ForeignActor)));
    assert_eq!(session.phase(), Phase::Idle);
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: ACTOR,
            flag: 0,
        })
        .unwrap();
    assert!(matches!(session.server(&packet), Err(Error::ForeignActor)));
    assert_eq!(session.phase(), Phase::Idle);
    assert_eq!(session.actor(), Some(ACTOR));
    assert!(session.registered());
}

#[test]
fn echo_and_result_value_strictness() {
    // FUN_00b668f0 flags a NaN distance in the cast-animation echo; the module
    // decodes it losslessly and leaves echo validation to the caller.
    // `decode_known` takes the body after the four-byte envelope: handle,
    // float, vector.
    let mut echo = handle_bytes(ACTOR);
    echo.extend(f32::NAN.to_le_bytes());
    echo.extend(vec_bytes([1.0, 2.0, 3.0, 4.0]));
    assert_eq!(echo.len(), 28);
    match ServerPacket::decode_known(Sub::CastAnimRequest, &echo).unwrap() {
        ServerPacket::CastAnimEcho { distance, .. } => assert!(distance.is_nan()),
        other => panic!("expected cast animation echo, got {other:?}"),
    }
    // FUN_00b66be0 only flags NaN; `valid()` additionally refuses infinities,
    // which is stricter than the recovered reader.
    let mut infinite = minimal_result_body();
    patch_f32(&mut infinite, 25, f32::INFINITY);
    match ServerPacket::decode_known(Sub::Result, &infinite).unwrap() {
        ServerPacket::Result { body } => {
            assert_eq!(body.at_38, f32::INFINITY);
            assert!(!body.valid(), "infinite result values are refused");
        }
        other => panic!("expected result, got {other:?}"),
    }
    // A finite body is valid.
    match ServerPacket::decode_known(Sub::Result, &minimal_result_body()).unwrap() {
        ServerPacket::Result { body } => assert!(body.valid()),
        other => panic!("expected result, got {other:?}"),
    }
}

#[test]
fn catch_authority_surface_is_bounded_at_compile_time() {
    // Adding a reward/catch variant to either enum breaks this match, which is
    // the review's check that no catch authority has appeared in the module.
    fn outcome_surface(outcome: Outcome) -> &'static str {
        match outcome {
            Outcome::Applied => "applied",
            Outcome::Duplicate => "duplicate",
        }
    }
    fn phase_surface(phase: Phase) -> &'static str {
        match phase {
            Phase::Idle => "idle",
            Phase::Hook => "hook",
            Phase::Reeling => "reeling",
            Phase::Casting => "casting",
            Phase::Landing => "landing",
        }
    }
    assert_eq!(outcome_surface(Outcome::Applied), "applied");
    assert_eq!(phase_surface(Phase::Landing), "landing");
    assert_eq!(ResultBody::default().valid(), true);
}
