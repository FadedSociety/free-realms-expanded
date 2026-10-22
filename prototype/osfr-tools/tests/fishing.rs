use osfr_tools::fishing::{
    self, ClientRequest, Error, Handle, OPCODE, Outcome, Phase, ServerPacket, Sub, Vec4,
};

fn handle() -> Handle {
    Handle {
        id: 0x1122_3344,
        seq: 0x5566_7788,
    }
}

fn vec4() -> Vec4 {
    Vec4 {
        x: 1.5,
        y: -2.25,
        z: 0.0,
        w: 1.0,
    }
}

#[test]
fn header_is_two_u16_fields() {
    // Native BaseFishingPacket::Read/Write moves two 16-bit fields:
    // opcode 138 then the sub-opcode (pseudocode 00b66830 / 00b69550).
    let bytes = ClientRequest::Special { actor: handle() }.encode();
    assert_eq!(OPCODE, 138);
    assert_eq!(&bytes[..2], &[0x8a, 0x00]);
    assert_eq!(&bytes[2..4], &[15, 0]);
    assert_eq!(bytes.len(), 4 + 8);
}

#[test]
fn every_recovered_request_round_trips() {
    let cases = [
        ClientRequest::RegisterPlayer {
            actor: handle(),
            flag: 1,
        },
        ClientRequest::CastAnim {
            actor: handle(),
            distance: 12.5,
            position: vec4(),
        },
        ClientRequest::Cast {
            actor: handle(),
            target: vec4(),
            flag: 0,
        },
        ClientRequest::ReelIn {
            actor: handle(),
            flag: 1,
        },
        ClientRequest::Special { actor: handle() },
    ];
    for case in cases {
        let bytes = case.encode();
        assert_eq!(ClientRequest::decode(&bytes).unwrap(), case);
    }
}

#[test]
fn request_lengths_match_native_writers() {
    assert_eq!(
        ClientRequest::RegisterPlayer {
            actor: handle(),
            flag: 0
        }
        .encode()
        .len(),
        13
    );
    assert_eq!(
        ClientRequest::CastAnim {
            actor: handle(),
            distance: 1.0,
            position: vec4()
        }
        .encode()
        .len(),
        32
    );
    assert_eq!(
        ClientRequest::Cast {
            actor: handle(),
            target: vec4(),
            flag: 0
        }
        .encode()
        .len(),
        29
    );
    assert_eq!(
        ClientRequest::ReelIn {
            actor: handle(),
            flag: 0
        }
        .encode()
        .len(),
        13
    );
    assert_eq!(
        ClientRequest::Special { actor: handle() }.encode().len(),
        12
    );
}

#[test]
fn decode_rejects_bad_envelopes() {
    let good = ClientRequest::Special { actor: handle() }.encode();
    assert!(matches!(
        ClientRequest::decode(&good[..6]),
        Err(Error::Truncated)
    ));
    let mut wrong_op = good.clone();
    wrong_op[0] = 0x8b;
    assert!(matches!(
        ClientRequest::decode(&wrong_op),
        Err(Error::WrongOpcode)
    ));
    let mut unknown_sub = good.clone();
    unknown_sub[2] = 9;
    assert!(matches!(
        ClientRequest::decode(&unknown_sub),
        Err(Error::UnknownSub(9))
    ));
    let mut trailing = good;
    trailing.push(0);
    assert!(matches!(
        ClientRequest::decode(&trailing),
        Err(Error::TrailingBytes)
    ));
}

#[test]
fn server_layouts_decode_from_native_field_order() {
    // UpdateData (1): handle + vec4 (FUN_00b66890).
    let mut update = Vec::new();
    update.extend(handle().encode());
    update.extend(Vec4 { x: 4.0, ..vec4() }.encode());
    let packet = ServerPacket::decode_known(Sub::UpdateData, &update).unwrap();
    assert!(matches!(packet, ServerPacket::UpdateData { position, .. } if position.x == 4.0));

    // DespawnProxiedFishingSchool (12): one u32 (FUN_00b68810).
    let despawn = 0x2a_u32.to_le_bytes();
    assert_eq!(
        ServerPacket::decode_known(Sub::DespawnSchool, &despawn).unwrap(),
        ServerPacket::DespawnSchool { school: 0x2a }
    );

    // UpdateProxiedFishingSchool (13): u32 + vec4 (FUN_00b66b80).
    let mut school = Vec::new();
    school.extend(7_u32.to_le_bytes());
    school.extend(vec4().encode());
    assert_eq!(
        ServerPacket::decode_known(Sub::UpdateSchool, &school).unwrap(),
        ServerPacket::UpdateSchool {
            school: 7,
            position: vec4()
        }
    );

    // SpawnProxiedFishingBobber (8): handle + u32 + vec4 + vec4 (FUN_00b66a00).
    let mut bobber = Vec::new();
    bobber.extend(handle().encode());
    bobber.extend(3_u32.to_le_bytes());
    bobber.extend(vec4().encode());
    bobber.extend(vec4().encode());
    assert_eq!(bobber.len(), 44);
    assert!(ServerPacket::decode_known(Sub::SpawnBobber, &bobber).is_ok());
    assert!(matches!(
        ServerPacket::decode_known(Sub::SpawnBobber, &bobber[..43]),
        Err(Error::Truncated)
    ));

    // Undocumented bodies stay opaque instead of being guessed.
    assert!(matches!(
        ServerPacket::decode_known(Sub::SpawnSchool, &[1, 2, 3]).unwrap(),
        ServerPacket::Opaque { .. }
    ));
}

#[test]
fn spawn_fish_run_matches_the_client_reader_byte_for_byte() {
    // Reader chain: 0x00b6a2b0 header, 0x00b694e0 count, 0x00b67550 per fish, 0x008d53b0 tail.
    let motion = [1.5f32, 2.0, 2.0, 1.0, 1.0, 3.0, 1.0, 0.5, 3.0, 1.0];
    let run = fishing::FishRun {
        flag: true,
        model: 1673,
        texture: Vec::new(),
        fish: vec![fishing::RunFish {
            id: 0x0102_0304,
            model: 1671,
            size_grade: 2,
            on_hook: true,
            motion,
            motion_tail: 3.0,
            ..Default::default()
        }],
    };
    let mut expected = vec![0x8a, 0, 18, 0];
    expected.push(1); // u8 flag (+0x0c)
    expected.extend(1673i32.to_le_bytes()); // header model (+0x10)
    expected.extend(0i32.to_le_bytes()); // empty texture string (+0x14)
    expected.extend(1i32.to_le_bytes()); // record count
    assert_eq!(expected.len(), 17);
    expected.extend(0x0102_0304u32.to_le_bytes()); // fish id (+0x00)
    expected.extend(1671i32.to_le_bytes()); // model (+0x04)
    expected.extend(0i32.to_le_bytes()); // empty string (+0x08)
    expected.extend(0i32.to_le_bytes()); // empty string (+0x18)
    expected.extend(2i32.to_le_bytes()); // size grade (+0x28)
    expected.extend(0i32.to_le_bytes()); // +0x2c
    expected.push(1); // on_hook (+0x30)
    for value in motion.iter().chain([&3.0f32]) {
        expected.extend(value.to_le_bytes()); // +0x34..=+0x5c
    }
    assert_eq!(expected.len(), 17 + 69);

    assert_eq!(fishing::encode_spawn_fish_run(&run), expected);
    let body = &expected[4..];
    assert_eq!(
        ServerPacket::decode_known(Sub::SpawnFishRun, body).unwrap(),
        ServerPacket::SpawnFishRun { run }
    );
    // 0x00b6aed0 refuses a body the reader does not consume exactly.
    let mut trailing = body.to_vec();
    trailing.push(0);
    assert!(matches!(
        ServerPacket::decode_known(Sub::SpawnFishRun, &trailing),
        Err(Error::TrailingBytes)
    ));
    assert!(matches!(
        ServerPacket::decode_known(Sub::SpawnFishRun, &body[..body.len() - 1]),
        Err(Error::Truncated)
    ));
    // 0x00b694e0 loops `j < count` on an i32: a negative count is an empty list, not a refusal.
    let mut negative = body[..9].to_vec();
    negative.extend((-1i32).to_le_bytes());
    match ServerPacket::decode_known(Sub::SpawnFishRun, &negative).unwrap() {
        ServerPacket::SpawnFishRun { run } => assert!(run.fish.is_empty()),
        packet => panic!("expected a fish run, got {packet:?}"),
    }
}

#[test]
fn fishing_result_rejects_nan_fields() {
    // The native reader flags NaN floats (param_2 + 0x11) in FUN_00b66be0.
    let mut body = Vec::new();
    body.extend(handle().encode());
    body.extend(1_u32.to_le_bytes());
    body.push(1);
    body.extend(2_u32.to_le_bytes());
    body.extend(3_u32.to_le_bytes());
    body.extend(0_i32.to_le_bytes());
    body.extend(f32::to_le_bytes(f32::NAN));
    body.extend(0_u32.to_le_bytes());
    body.extend(f32::to_le_bytes(1.0));
    for _ in 0..4 {
        body.extend(0_u32.to_le_bytes());
    }
    body.extend(0_i32.to_le_bytes());
    body.extend(0_i32.to_le_bytes());
    body.extend(0_u32.to_le_bytes());
    body.extend(f32::to_le_bytes(2.0));
    body.push(0);
    body.extend(9_u32.to_le_bytes());
    let result = ServerPacket::decode_known(Sub::Result, &body).unwrap();
    match result {
        ServerPacket::Result { body } => assert!(!body.valid()),
        other => panic!("expected result, got {other:?}"),
    }
}

#[test]
fn session_enforces_recovered_ordering() {
    let actor = handle();
    let mut session = fishing::Session::new();
    // Cast before registering is refused.
    assert!(matches!(
        session.request(&ClientRequest::Cast {
            actor,
            target: vec4(),
            flag: 0
        }),
        Err(Error::OutOfOrder)
    ));
    assert_eq!(
        session
            .request(&ClientRequest::RegisterPlayer { actor, flag: 0 })
            .unwrap(),
        Outcome::Applied
    );
    assert_eq!(
        session
            .request(&ClientRequest::Cast {
                actor,
                target: vec4(),
                flag: 0
            })
            .unwrap(),
        Outcome::Applied
    );
    assert_eq!(session.phase(), Phase::Casting);
    // Native ControllerFishing drops a duplicate cast while already in the cast state.
    assert_eq!(
        session
            .request(&ClientRequest::Cast {
                actor,
                target: vec4(),
                flag: 0
            })
            .unwrap(),
        Outcome::Duplicate
    );
    // Reel-in only makes sense once a fish is hooked.
    assert!(matches!(
        session.request(&ClientRequest::ReelIn { actor, flag: 0 }),
        Err(Error::OutOfOrder)
    ));
    session.hooked();
    assert_eq!(
        session
            .request(&ClientRequest::ReelIn { actor, flag: 0 })
            .unwrap(),
        Outcome::Applied
    );
    assert_eq!(session.phase(), Phase::Reeling);
    // A valid fishing result lands the catch; reset returns the session to idle.
    let landed = ServerPacket::Result {
        body: fishing::ResultBody {
            actor: handle(),
            at_7c: 2.0,
            ..Default::default()
        },
    };
    session.server(&landed).unwrap();
    assert_eq!(session.phase(), Phase::Landing);
    session.reset();
    assert_eq!(session.phase(), Phase::Idle);
}

fn other() -> Handle {
    Handle {
        id: 0xdead_beef,
        seq: 0x0bad_f00d,
    }
}

#[test]
fn session_rejects_requests_from_a_foreign_actor() {
    let mut session = fishing::Session::new();
    assert!(matches!(
        session.request(&ClientRequest::CastAnim {
            actor: handle(),
            distance: 1.0,
            position: vec4(),
        }),
        Err(Error::OutOfOrder)
    ));
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: handle(),
            flag: 0,
        })
        .unwrap();
    for foreign in [
        ClientRequest::Cast {
            actor: other(),
            target: vec4(),
            flag: 0,
        },
        ClientRequest::ReelIn {
            actor: other(),
            flag: 0,
        },
        ClientRequest::Special { actor: other() },
    ] {
        assert!(matches!(
            session.request(&foreign),
            Err(Error::ForeignActor)
        ));
    }
    assert_eq!(session.phase(), Phase::Idle);
    assert_eq!(
        session
            .request(&ClientRequest::Cast {
                actor: handle(),
                target: vec4(),
                flag: 0,
            })
            .unwrap(),
        Outcome::Applied
    );
}

#[test]
fn re_registration_and_reset_policy() {
    let mut session = fishing::Session::new();
    let first = ClientRequest::RegisterPlayer {
        actor: handle(),
        flag: 0,
    };
    assert_eq!(session.request(&first).unwrap(), Outcome::Applied);
    // Reconstructed policy: the same actor re-registering is a no-op, another is refused.
    assert_eq!(session.request(&first).unwrap(), Outcome::Duplicate);
    assert!(matches!(
        session.request(&ClientRequest::RegisterPlayer {
            actor: other(),
            flag: 0,
        }),
        Err(Error::ForeignActor)
    ));
    // Reconstructed policy: reset unbinds the actor so a new player can register.
    session.reset();
    assert_eq!(session.actor(), None);
    assert_eq!(
        session
            .request(&ClientRequest::RegisterPlayer {
                actor: other(),
                flag: 0,
            })
            .unwrap(),
        Outcome::Applied
    );
    assert_eq!(session.actor(), Some(other()));
}

#[test]
fn session_rejects_nonfinite_and_negative_values_without_mutating() {
    let mut session = fishing::Session::new();
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: handle(),
            flag: 0,
        })
        .unwrap();
    let bad = [
        ClientRequest::CastAnim {
            actor: handle(),
            distance: f32::NAN,
            position: vec4(),
        },
        ClientRequest::CastAnim {
            actor: handle(),
            distance: f32::INFINITY,
            position: vec4(),
        },
        ClientRequest::CastAnim {
            actor: handle(),
            distance: -1.0,
            position: vec4(),
        },
        ClientRequest::CastAnim {
            actor: handle(),
            distance: 1.0,
            position: Vec4 {
                x: f32::NAN,
                ..vec4()
            },
        },
        ClientRequest::Cast {
            actor: handle(),
            target: Vec4 {
                w: f32::NEG_INFINITY,
                ..vec4()
            },
            flag: 0,
        },
    ];
    for request in bad {
        assert!(matches!(
            session.request(&request),
            Err(Error::InvalidValue)
        ));
        assert_eq!(session.phase(), Phase::Idle);
    }
}

#[test]
fn wire_codec_stays_lossless_for_values_the_session_refuses() {
    let invalid = ClientRequest::CastAnim {
        actor: handle(),
        distance: f32::NAN,
        position: vec4(),
    };
    let decoded = ClientRequest::decode(&invalid.encode()).unwrap();
    match decoded {
        ClientRequest::CastAnim {
            distance, position, ..
        } => {
            assert!(distance.is_nan());
            assert_eq!(position, vec4());
        }
        other => panic!("expected cast animation, got {other:?}"),
    }
}

#[test]
fn session_rejects_a_result_for_a_foreign_actor() {
    let mut session = fishing::Session::new();
    session
        .request(&ClientRequest::RegisterPlayer {
            actor: handle(),
            flag: 0,
        })
        .unwrap();
    let foreign = ServerPacket::Result {
        body: fishing::ResultBody {
            actor: other(),
            at_7c: 2.0,
            ..Default::default()
        },
    };
    assert!(matches!(session.server(&foreign), Err(Error::ForeignActor)));
    assert_eq!(session.phase(), Phase::Idle);
}
