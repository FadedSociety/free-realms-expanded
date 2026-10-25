//! T519: Fotomat 156/2 -> 156/3 and the quiet 115 portrait CRC.
//!
//! The decoder mirrors the client readers `00bd4180` (packet), `00bd37e0` (record),
//! `009131c0` (attachments) and `009538e0` (image), and requires full consumption.
//! The login-time request is driven through the real `Gateway::handle_at` pre-load path;
//! the in-world route is covered by the binary's `fotomat_routing` module.

#[path = "common/fotomat_wire.rs"]
mod fotomat_wire;
#[path = "common/style_card_fixture.rs"]
#[allow(dead_code)]
mod style_card_fixture;

use fotomat_wire::{decode, request, tunnel_inner};
use osfr_tools::{
    fotomat,
    gateway::Gateway,
    players::{InterestPolicy, PlayerPresentation, PlayerRegistry, SessionId, ZoneInstance},
};
use std::path::PathBuf;

const VIEWER: SessionId = SessionId(1);
const VIEWER_GUID: u64 = 1001;
const VISIBLE: SessionId = SessionId(2);
const VISIBLE_GUID: u64 = 1002;
const FAR: SessionId = SessionId(3);
const FAR_GUID: u64 = 1003;
const ELSEWHERE: SessionId = SessionId(4);
const ELSEWHERE_GUID: u64 = 1004;

fn zone(zone_id: u32) -> ZoneInstance {
    ZoneInstance {
        zone_id,
        instance_id: 1,
    }
}

fn presentation(guid: u64) -> PlayerPresentation {
    PlayerPresentation {
        character_guid: guid,
        first_name: format!("P{guid}"),
        model: 40 + guid as i32,
        head: format!("head-{guid}.adr"),
        hair: format!("hair-{guid}.adr"),
        hair_color: 7,
        eye_color: 8,
        skin_tone: format!("skin-{guid}"),
        face_paint: format!("paint-{guid}"),
        model_customization: format!("custom-{guid}.adr"),
        customization_ids: [11, 12, 13, 14],
        attachments_wire: fotomat_wire::one_attachment(),
        ..PlayerPresentation::default()
    }
}

fn registry() -> PlayerRegistry {
    let mut registry = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
    for (session, guid, zone_id, position) in [
        (VIEWER, VIEWER_GUID, 1, [0.0, 0.0, 0.0]),
        (VISIBLE, VISIBLE_GUID, 1, [5.0, 0.0, 0.0]),
        (FAR, FAR_GUID, 1, [1_000.0, 0.0, 0.0]),
        (ELSEWHERE, ELSEWHERE_GUID, 2, [1.0, 0.0, 0.0]),
    ] {
        registry
            .join_with_presentation(
                session,
                guid + 10_000,
                guid,
                zone(zone_id),
                position,
                0.0,
                presentation(guid),
            )
            .unwrap();
    }
    registry
}

fn reply(body: &[u8], viewer: SessionId, registry: &PlayerRegistry) -> Vec<u8> {
    fotomat::request(body, viewer, registry).expect("expected one 156/3")
}

#[test]
fn fotomat_answers_self_and_visible_players() {
    let registry = registry();
    // The directed view never contains the viewer itself; `presentation_of` adds the self
    // case on top of `knows_character` (the 156/2 rule).
    assert!(!registry.knows_character(VIEWER, VIEWER_GUID));
    assert!(registry.knows_character(VIEWER, VISIBLE_GUID));
    assert!(!registry.knows_character(VIEWER, FAR_GUID));
    assert!(!registry.knows_character(VIEWER, ELSEWHERE_GUID));

    for (guid, provider) in [(VIEWER_GUID, "Headshot"), (VISIBLE_GUID, "Portrait")] {
        let packet = reply(&request(guid, provider), VIEWER, &registry);
        let image = decode(&packet);
        assert_eq!(image.guid, guid);
        assert_eq!(image.provider, provider);
        assert_eq!(
            image.record_provider, provider,
            "record echoes the provider"
        );
        assert!(!image.compressed);
        assert_eq!((image.unknown, image.unknown2), (0, 1));
        assert_eq!(image.record_guid, guid);
        assert!(
            image.image.is_empty(),
            "empty image until the upload ingress"
        );

        let expected = presentation(guid);
        assert_eq!(image.model, expected.model);
        assert_eq!(image.head, expected.head);
        assert_eq!(image.hair, expected.hair);
        assert_eq!(image.hair_color, expected.hair_color);
        assert_eq!(image.eye_color, expected.eye_color);
        assert_eq!(image.skin_tone, expected.skin_tone);
        assert_eq!(image.face_paint, expected.face_paint);
        assert_eq!(image.model_customization, expected.model_customization);
        assert_eq!(image.customization_ids, expected.customization_ids);
        assert_eq!(image.attachments, expected.attachments_wire);
    }

    for guid in [FAR_GUID, ELSEWHERE_GUID, 9999] {
        assert!(
            fotomat::request(&request(guid, "Headshot"), VIEWER, &registry).is_none(),
            "no reply for a character outside the viewer's directed AddPc: {guid}"
        );
    }
}

#[test]
fn fotomat_refuses_without_reply() {
    let registry = registry();

    let mut trailing = request(VISIBLE_GUID, "Headshot");
    trailing.push(0);
    let mut wrong_family = request(VISIBLE_GUID, "Headshot");
    wrong_family[..2].copy_from_slice(&155u16.to_le_bytes());
    let mut wrong_subtype = request(VISIBLE_GUID, "Headshot");
    wrong_subtype[2..4].copy_from_slice(&3u16.to_le_bytes());

    for body in [
        request(VISIBLE_GUID, "Bogus"),
        request(VISIBLE_GUID, "Data"),
        request(VISIBLE_GUID, "HeadshotHeadshot!"),
        trailing,
        wrong_family,
        wrong_subtype,
        vec![156, 0, 2, 0],
        vec![156, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0],
    ] {
        assert!(
            fotomat::request(&body, VIEWER, &registry).is_none(),
            "refused: {body:02x?}"
        );
    }

    // 115 is the top-level portrait CRC: exactly u16 115 + u32 CRC, and it never replies.
    assert!(fotomat::is_upload(&[115, 0, 1, 2, 3, 4]));
    assert!(fotomat::handles(&[115, 0, 1, 2, 3, 4]));
    assert!(!fotomat::is_request(&[115, 0, 1, 2, 3, 4]));
    for body in [
        vec![],
        vec![115, 0],
        vec![115, 0, 1],
        vec![115, 0, 1, 2, 3],
        vec![115, 0, 1, 2, 3, 4, 5],
        vec![114, 0, 1, 2, 3, 4],
    ] {
        assert!(!fotomat::is_upload(&body), "{body:02x?}");
        assert!(
            fotomat::handle(&body, |_| panic!("115 must not look up a presentation")).is_none(),
            "{body:02x?}"
        );
    }
    assert!(
        fotomat::handle(&[115, 0, 0, 0, 0, 0], |_| {
            panic!("115 must not look up a presentation")
        })
        .is_none()
    );
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut out = vec![5, 0, 1];
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-gateway-reference")
}

#[test]
fn login_time_request_is_handled_or_dropped_cleanly() {
    let reference = reference();
    if !reference.join("manifest.json").is_file() {
        eprintln!("skip t519 login-time: the private lab reference is absent");
        return;
    }

    let state = Gateway::reference_state(&reference).unwrap();
    let guid = state.guid;
    let mut identity = style_card_fixture::boy();
    identity.guid = guid;
    let mut gateway = Gateway::load(&reference, state).unwrap();
    gateway.set_identity(identity).unwrap();

    let outcome = gateway
        .handle_at(&wrap(&request(guid, "Headshot")), 700)
        .unwrap();
    assert_eq!(outcome.replies.len(), 1, "one 156/3 before Loaded");
    let image = decode(tunnel_inner(&outcome.replies[0]));
    assert_eq!(image.guid, guid);
    assert_eq!(image.provider, "Headshot");
    assert_eq!(image.record_guid, guid);
    assert!(image.image.is_empty());
    assert_eq!(image.head, "<race>_<gender>_head_african.adr");
    assert_eq!(image.customization_ids, [2, 1, 3, 0]);
    assert_eq!(
        fotomat::requested_guid(&request(guid, "Headshot")),
        Some(guid)
    );

    // A six-byte 115 is owned and shape-checked before Loaded: no reply, no state, and the
    // trace stays "handled" instead of being forwarded or dropped as unloaded.
    gateway.set_trace_enabled(true);
    let before = gateway.snapshot();
    let outcome = gateway
        .handle_at(&wrap(&[115, 0, 1, 2, 3, 4]), 700)
        .unwrap();
    assert!(outcome.replies.is_empty());
    assert_eq!(outcome.trace[0].handler, "fotomat");
    assert_eq!(outcome.trace[0].outcome, "handled");
    assert_eq!(gateway.snapshot(), before);

    // Any other 115 length is not the notice: it falls to the normal pre-Loaded path.
    let outcome = gateway.handle_at(&wrap(&[115, 0, 1, 2, 3]), 700).unwrap();
    assert!(outcome.replies.is_empty());
    assert_ne!(
        outcome.trace[0].outcome, "handled",
        "a malformed 115 is not the six-byte notice"
    );
    assert_eq!(gateway.snapshot(), before);

    // Another guid is not this session's presentation: no reply and no state.
    let outcome = gateway
        .handle_at(&wrap(&request(guid + 1, "Headshot")), 700)
        .unwrap();
    assert!(outcome.replies.is_empty());
    assert_eq!(gateway.snapshot(), before);

    // Without an identity or a saved look there is nothing to answer: a clean drop.
    let state = Gateway::reference_state(&reference).unwrap();
    let mut fresh = Gateway::load(&reference, state).unwrap();
    let own = fresh.state().guid;
    let before = fresh.snapshot();
    let outcome = fresh
        .handle_at(&wrap(&request(own, "Portrait")), 700)
        .unwrap();
    assert!(outcome.replies.is_empty());
    assert_eq!(fresh.snapshot(), before);
}
