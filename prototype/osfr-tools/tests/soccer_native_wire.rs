//! Native soccer wire repair acceptance.
//!
//! Every byte here is constructed from the revision-3 layouts
//! (`notes/Research/Soccer Kick Pass Wire Mapping.md`): sub 14 is 62 bytes
//! (`u64` player, `u8` state, `u8` aux, `u32` sequence, three `f32[4]`),
//! sub 16 is 20 bytes (`u64`, two `f32`, `u32` control bits), sub 13 is
//! 36 bytes (`u32`, `f32[4]`, `f32`, `f32`, `u32`, `f32`). No captured raw
//! bytes are embedded; the byte-exact captured round trip lives in the
//! private report.
use osfr_tools::soccer::{Message, SoccerPacket, decode, encode};
use osfr_tools::soccer_match::{
    ClientPacket, Intent, StateAction, classify_state, client_intent, client_packet,
};

fn envelope(sub: u16, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(5u16.to_le_bytes());
    out.push(1);
    out.extend(((payload.len() + 4) as u32).to_le_bytes());
    out.extend(76u16.to_le_bytes());
    out.extend(sub.to_le_bytes());
    out.extend(payload);
    out
}

fn f32s(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn sub14_payload(state: u8, aux: u8) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend(0x1122_3344_5566_7788u64.to_le_bytes());
    payload.push(state);
    payload.push(aux);
    payload.extend(0x0BAD_F00Du32.to_le_bytes());
    payload.extend(f32s(&[1.0, -2.5, 3.25, -4.0]));
    payload.extend(f32s(&[0.5, 0.25, -0.125, 8.0]));
    payload.extend(f32s(&[-1.0, 2.0, -3.0, 4.0]));
    payload
}

fn sub16_payload(bits: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend(0x0102_0304_0506_0708u64.to_le_bytes());
    payload.extend(f32s(&[0.75, -1.25]));
    payload.extend(bits.to_le_bytes());
    payload
}

fn sub13_payload() -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend(0u32.to_le_bytes());
    payload.extend(f32s(&[1.0, 2.0, 3.0, 4.0]));
    payload.extend(5.5f32.to_le_bytes());
    payload.extend((-6.25f32).to_le_bytes());
    payload.extend(0xDEAD_BEEFu32.to_le_bytes());
    payload.extend(7.75f32.to_le_bytes());
    payload
}

#[test]
fn sub14_exact_vector_round_trips_with_u8_state_aux_and_three_vectors() {
    let bytes = envelope(14, &sub14_payload(17, 0x2A));
    let message: Message = decode(&bytes).expect("sub 14 decodes");
    assert_eq!(
        message.packet,
        SoccerPacket::UpdatePlayerState {
            player: 0x1122_3344_5566_7788,
            state: 17,
            aux: 0x2A,
            state_counter: 0x0BAD_F00D,
            vectors: [
                [1.0, -2.5, 3.25, -4.0],
                [0.5, 0.25, -0.125, 8.0],
                [-1.0, 2.0, -3.0, 4.0],
            ],
        }
    );
    assert_eq!(encode(&message), bytes, "byte-exact sub-14 round trip");
}

#[test]
fn sub14_kick_state_refuses_context_free_intent_conversion() {
    let bytes = envelope(14, &sub14_payload(17, 0xFF));
    match client_packet(&bytes, 99).expect("typed request") {
        ClientPacket::State(request) => {
            assert_eq!(request.player, 0x1122_3344_5566_7788);
            assert_eq!(request.sequence, 99);
            assert_eq!(request.state, 17);
            assert_eq!(request.aux, 0xFF);
            assert_eq!(
                request.state_counter, 0x0BAD_F00D,
                "the wire state counter is not a transport sequence"
            );
            assert_eq!(request.action, StateAction::Kick);
        }
        other => panic!("expected a typed state request, got {other:?}"),
    }
    let error = client_intent(&bytes, 99).unwrap_err().to_string();
    // W37 applies aux through configured R charge policy; pass requests also need a verified
    // target. Client evidence proves aux/255, not power scale or target rules. See
    // notes/Research/Soccer Service.md (Client evidence; Reconstructed service rules).
    assert!(
        error.contains("configured charge policy") && error.contains("server-verified pass target"),
        "{error}"
    );
    assert!(
        !error.contains("Move"),
        "a kick state must never be translated as Move: {error}"
    );

    // Non kick/pass states stay read-only and preserve aux verbatim.
    let run = envelope(14, &sub14_payload(11, 0x5A));
    match client_packet(&run, 100).expect("read-only state") {
        ClientPacket::ReadOnly(SoccerPacket::UpdatePlayerState { state, aux, .. }) => {
            assert_eq!((state, aux), (11, 0x5A));
        }
        other => panic!("expected read-only state, got {other:?}"),
    }
    assert!(
        client_intent(&run, 100)
            .unwrap_err()
            .to_string()
            .contains("read-only")
    );
    // An unknown state byte is read-only, never a kick/pass.
    let unknown = envelope(14, &sub14_payload(0xFF, 0));
    assert!(matches!(
        client_packet(&unknown, 101).unwrap(),
        ClientPacket::ReadOnly(_)
    ));
}

#[test]
fn sub16_exact_vector_keeps_control_bits_as_a_u32() {
    let bytes = envelope(16, &sub16_payload(0x41));
    let message: Message = decode(&bytes).expect("sub 16 decodes");
    assert_eq!(
        message.packet,
        SoccerPacket::UpdatePlayerControls {
            player: 0x0102_0304_0506_0708,
            direction: [0.75, -1.25],
            controls: 0x41,
        }
    );
    assert_eq!(encode(&message), bytes, "byte-exact sub-16 round trip");
    match client_intent(&bytes, 5).expect("controls intent") {
        Intent::Controls {
            direction,
            controls,
            ..
        } => {
            assert_eq!(direction, [0.75, -1.25]);
            assert_eq!(controls, 0x41, "control bits stay integer input state");
        }
        other => panic!("expected controls intent, got {other:?}"),
    }
}

#[test]
fn control_bits_and_unknown_states_never_classify_as_kick_or_pass() {
    for code in 0..=u8::MAX {
        let expected = match code {
            16..=26 | 31..=34 => Some(StateAction::Kick),
            27..=30 => Some(StateAction::Pass),
            _ => None,
        };
        assert_eq!(classify_state(code), expected, "state code {code}");
    }

    // Raw server-relay bit patterns survive decode and re-encode untouched and
    // are never interpreted as kick/pass anywhere in the adapter.
    for bits in [0u32, 1, 2, 0x11, 0x41, 0x100, 0xFFFF_FFFF] {
        let bytes = envelope(16, &sub16_payload(bits));
        let message = decode(&bytes).unwrap();
        match message.packet {
            SoccerPacket::UpdatePlayerControls { controls, .. } => assert_eq!(controls, bits),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(encode(&message), bytes);
        match client_intent(&bytes, 7).unwrap() {
            Intent::Controls { controls, .. } => assert_eq!(controls, bits),
            other => panic!("unexpected {other:?}"),
        }
    }
}

#[test]
fn sub13_impulse_round_trips_but_is_refused_as_a_client_request() {
    let bytes = envelope(13, &sub13_payload());
    let message = decode(&bytes).expect("sub 13 decodes");
    assert_eq!(
        message.packet,
        SoccerPacket::ImpulseBall {
            word: 0,
            vector: [1.0, 2.0, 3.0, 4.0],
            value_a: 5.5,
            value_b: -6.25,
            action: 0xDEAD_BEEF,
            value_c: 7.75,
        }
    );
    assert_eq!(encode(&message), bytes, "byte-exact sub-13 round trip");
    let error = client_packet(&bytes, 1).unwrap_err().to_string();
    assert!(error.contains("server-authored"), "{error}");
}

#[test]
fn exact_offsets_match_the_revision_3_readers() {
    let bytes = envelope(14, &sub14_payload(17, 0x2A));
    let payload = &bytes[11..];
    assert_eq!(&payload[0..8], &0x1122_3344_5566_7788u64.to_le_bytes());
    assert_eq!(payload[8], 17);
    assert_eq!(payload[9], 0x2A);
    assert_eq!(&payload[10..14], &0x0BAD_F00Du32.to_le_bytes());
    assert_eq!(&payload[14..30], &f32s(&[1.0, -2.5, 3.25, -4.0])[..]);
    assert_eq!(&payload[30..46], &f32s(&[0.5, 0.25, -0.125, 8.0])[..]);
    assert_eq!(&payload[46..62], &f32s(&[-1.0, 2.0, -3.0, 4.0])[..]);

    let controls = envelope(16, &sub16_payload(0x41));
    let payload = &controls[11..];
    assert_eq!(&payload[0..8], &0x0102_0304_0506_0708u64.to_le_bytes());
    assert_eq!(&payload[8..12], &0.75f32.to_le_bytes());
    assert_eq!(&payload[12..16], &(-1.25f32).to_le_bytes());
    assert_eq!(&payload[16..20], &0x41u32.to_le_bytes());

    let impulse = envelope(13, &sub13_payload());
    let payload = &impulse[11..];
    assert_eq!(&payload[0..4], &0u32.to_le_bytes());
    assert_eq!(&payload[4..20], &f32s(&[1.0, 2.0, 3.0, 4.0])[..]);
    assert_eq!(&payload[20..24], &5.5f32.to_le_bytes());
    assert_eq!(&payload[24..28], &(-6.25f32).to_le_bytes());
    assert_eq!(&payload[28..32], &0xDEAD_BEEFu32.to_le_bytes());
    assert_eq!(&payload[32..36], &7.75f32.to_le_bytes());
}

#[test]
fn malformed_native_vectors_are_refused() {
    let bytes = envelope(14, &sub14_payload(17, 0));
    assert!(decode(&bytes[..bytes.len() - 1]).is_err(), "short sub 14");
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(decode(&trailing).is_err(), "trailing byte");
    let mut wrong_length = bytes;
    wrong_length[3] += 1;
    assert!(decode(&wrong_length).is_err(), "length mismatch");

    let controls = envelope(16, &sub16_payload(0x41));
    assert!(
        decode(&controls[..controls.len() - 1]).is_err(),
        "short sub 16"
    );
    let impulse = envelope(13, &sub13_payload());
    assert!(
        decode(&impulse[..impulse.len() - 1]).is_err(),
        "short sub 13"
    );
}

#[test]
fn nan_direction_bits_are_preserved_not_silently_normalized() {
    let nan_payload = {
        let mut payload = sub16_payload(0x41);
        payload[8..12].copy_from_slice(&f32::NAN.to_le_bytes());
        payload
    };
    let bytes = envelope(16, &nan_payload);
    let message = decode(&bytes).expect("decode preserves the wire bits");
    match message.packet {
        SoccerPacket::UpdatePlayerControls { direction, .. } => {
            assert!(direction[0].is_nan());
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(encode(&message), bytes, "NaN bits survive the round trip");
}
