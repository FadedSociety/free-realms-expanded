//! T519: the Fotomat 156/2 and 115 through `Session::dispatch_message`, the same
//! staging/commit/route path a socket datagram takes. The 156/3 is rebuilt from the target's
//! live `PlayerPresentation`; a refusal returns no deliveries and leaves the world unchanged.
use super::tests::test_support::{self, Scene, Scratch, bin_test_target};
use super::*;
use osfr_tools::players::PlayerPresentation;

#[path = "../../../tests/common/fotomat_wire.rs"]
mod fotomat_wire;
#[path = "../../../tests/common/style_card_fixture.rs"]
#[allow(dead_code)]
mod style_card_fixture;

const TARGET_GUID: u64 = 7001;
const TARGET_SESSION: SessionId = SessionId(91);
const FAR_GUID: u64 = 7002;
const FAR_SESSION: SessionId = SessionId(92);
const ELSEWHERE_GUID: u64 = 7003;
const ELSEWHERE_SESSION: SessionId = SessionId(93);

fn options(scratch: &Scratch) -> Options {
    let mut options = test_support::options(scratch.state());
    let identity = scratch.path().join("identity.json");
    osfr_tools::identity::save(&identity, &style_card_fixture::boy()).unwrap();
    options.identity = Some(identity);
    options
}

fn presentation(guid: u64) -> PlayerPresentation {
    PlayerPresentation {
        character_guid: guid,
        first_name: format!("P{guid}"),
        model: 70,
        head: format!("head-{guid}.adr"),
        hair: format!("hair-{guid}.adr"),
        hair_color: 9,
        eye_color: 10,
        skin_tone: format!("skin-{guid}"),
        face_paint: String::new(),
        model_customization: String::new(),
        customization_ids: [4, 5, 6, 7],
        attachments_wire: fotomat_wire::one_attachment(),
        ..PlayerPresentation::default()
    }
}

fn enter_world(scene: &mut Scene, options: &Options) {
    scene.send(options, &[13, 0], 100).unwrap();
    scene.send(options, &[10, 0], 200).unwrap();
    let _ = scene.drain(200);
}

fn join_targets(scene: &mut Scene) {
    let me = scene.session.id;
    let row = scene.world.players.get(me).unwrap().clone();
    let near = [row.position[0] + 1.0, row.position[1], row.position[2]];
    let far = [row.position[0] + 10_000.0, row.position[1], row.position[2]];
    for (session, guid, zone, position) in [
        (TARGET_SESSION, TARGET_GUID, row.zone, near),
        (FAR_SESSION, FAR_GUID, row.zone, far),
        (
            ELSEWHERE_SESSION,
            ELSEWHERE_GUID,
            ZoneInstance {
                zone_id: row.zone.zone_id + 1,
                instance_id: row.zone.instance_id,
            },
            near,
        ),
    ] {
        scene
            .world
            .players
            .join_with_presentation(
                session,
                guid + 10_000,
                guid,
                zone,
                position,
                0.0,
                presentation(guid),
            )
            .unwrap();
    }
}

fn image_reply(replies: &[Vec<u8>]) -> Option<&[u8]> {
    replies.iter().find_map(|packet| {
        (packet.starts_with(&[5, 0, 1]) && packet.get(7..11) == Some(&[156, 0, 3, 0]))
            .then(|| fotomat_wire::tunnel_inner(packet))
    })
}

fn send(scene: &mut Scene, options: &Options, body: &[u8], now: u64) -> Vec<Vec<u8>> {
    scene.send(options, body, now).unwrap_or_else(|error| {
        panic!(
            "packet {:02x?} refused: {error}",
            &body[..body.len().min(4)]
        )
    });
    scene.drain(now)
}

fn assert_refused(scene: &mut Scene, options: &Options, body: &[u8], now: u64) {
    let snapshot = scene.session.gateway.snapshot();
    let target = scene.world.players.get(TARGET_SESSION).cloned();
    let replies = send(scene, options, body, now);
    assert!(
        replies.is_empty(),
        "a refusal must send nothing: {body:02x?} -> {replies:02x?}"
    );
    assert_eq!(scene.session.gateway.snapshot(), snapshot, "{body:02x?}");
    assert_eq!(
        scene.world.players.get(TARGET_SESSION).cloned(),
        target,
        "{body:02x?}"
    );
}

#[test]
fn fotomat_route_answers_self_and_visible_players() {
    if !bin_test_target("t519") {
        return;
    }
    let scratch = Scratch::new("t519-route");
    let options = options(&scratch);
    if !test_support::fixtures_present(&options) {
        eprintln!("skip t519-route: the private lab fixtures are absent");
        return;
    }
    let mut scene = Scene::new(&options, 1);
    enter_world(&mut scene, &options);
    assert!(scene.world.joined.contains(&scene.session.id));
    join_targets(&mut scene);
    assert!(
        scene
            .world
            .players
            .knows_character(scene.session.id, TARGET_GUID)
    );
    assert!(
        !scene
            .world
            .players
            .knows_character(scene.session.id, FAR_GUID)
    );
    assert!(
        !scene
            .world
            .players
            .knows_character(scene.session.id, ELSEWHERE_GUID)
    );

    let before = scene.session.gateway.snapshot();
    let replies = send(
        &mut scene,
        &options,
        &fotomat_wire::request(TARGET_GUID, "Headshot"),
        300,
    );
    let image = decode_one(&replies);
    assert_eq!(image.guid, TARGET_GUID);
    assert_eq!(image.provider, "Headshot");
    assert_eq!(image.record_provider, "Headshot");
    assert!(image.image.is_empty());
    let target = scene.world.players.get(TARGET_SESSION).unwrap();
    assert_eq!(image.model, target.presentation.model);
    assert_eq!(image.head, target.presentation.head);
    assert_eq!(
        image.customization_ids,
        target.presentation.customization_ids
    );
    assert_eq!(image.attachments, target.presentation.attachments_wire);
    assert_eq!(
        scene.session.gateway.snapshot(),
        before,
        "a reply changes no state"
    );

    let own = scene.session.gateway.player_presentation(false).unwrap();
    let replies = send(
        &mut scene,
        &options,
        &fotomat_wire::request(own.character_guid, "Portrait"),
        400,
    );
    let image = decode_one(&replies);
    assert_eq!(image.guid, own.character_guid);
    assert_eq!(image.provider, "Portrait");
    assert_eq!(image.head, own.head);
    assert_eq!(image.hair, own.hair);
    assert_eq!(image.customization_ids, own.customization_ids);
    assert_eq!(image.attachments, own.attachments_wire);
    assert!(image.image.is_empty());
}

fn decode_one(replies: &[Vec<u8>]) -> fotomat_wire::ImageData<'_> {
    let count = replies
        .iter()
        .filter(|packet| {
            packet.starts_with(&[5, 0, 1]) && packet.get(7..11) == Some(&[156, 0, 3, 0])
        })
        .count();
    assert_eq!(count, 1, "one 156/3 per request");
    let image = image_reply(replies).expect("one 156/3");
    fotomat_wire::decode(image)
}

#[test]
fn fotomat_route_refuses_without_reply() {
    if !bin_test_target("t519") {
        return;
    }
    let scratch = Scratch::new("t519-refuse");
    let options = options(&scratch);
    if !test_support::fixtures_present(&options) {
        eprintln!("skip t519-refuse: the private lab fixtures are absent");
        return;
    }
    let mut scene = Scene::new(&options, 1);
    enter_world(&mut scene, &options);
    join_targets(&mut scene);

    let mut trailing = fotomat_wire::request(TARGET_GUID, "Headshot");
    trailing.push(0);
    let mut wrong_family = fotomat_wire::request(TARGET_GUID, "Headshot");
    wrong_family[..2].copy_from_slice(&155u16.to_le_bytes());
    let mut wrong_subtype = fotomat_wire::request(TARGET_GUID, "Headshot");
    wrong_subtype[2..4].copy_from_slice(&3u16.to_le_bytes());

    let mut now = 300;
    for body in [
        fotomat_wire::request(FAR_GUID, "Headshot"),
        fotomat_wire::request(ELSEWHERE_GUID, "Headshot"),
        fotomat_wire::request(9999, "Headshot"),
        fotomat_wire::request(TARGET_GUID, "Data"),
        fotomat_wire::request(TARGET_GUID, "HeadshotHeadshot!"),
        trailing,
        wrong_family,
        wrong_subtype,
        vec![156, 0, 2, 0],
        vec![115, 0, 1, 2, 3],
        vec![115, 0, 1, 2, 3, 4],
        vec![115, 0, 0, 0, 0, 0, 0],
    ] {
        assert_refused(&mut scene, &options, &body, now);
        now += 1;
    }
}

#[test]
fn login_time_request_is_answered_before_world_entry() {
    if !bin_test_target("t519") {
        return;
    }
    let scratch = Scratch::new("t519-login");
    let options = options(&scratch);
    if !test_support::fixtures_present(&options) {
        eprintln!("skip t519-login: the private lab fixtures are absent");
        return;
    }
    let mut scene = Scene::new(&options, 1);
    let guid = scene.session.gateway.state().guid;
    assert!(!scene.world.joined.contains(&scene.session.id));

    let replies = send(
        &mut scene,
        &options,
        &fotomat_wire::request(guid, "Headshot"),
        50,
    );
    let image = decode_one(&replies);
    assert_eq!(image.guid, guid);
    assert_eq!(image.provider, "Headshot");
    assert!(image.image.is_empty());
    assert!(!scene.world.joined.contains(&scene.session.id));

    // The client can still finish entering the world after the pre-load answer.
    scene.send(&options, &[13, 0], 100).unwrap();
    scene.send(&options, &[10, 0], 200).unwrap();
    let _ = scene.drain(200);
    assert!(scene.world.joined.contains(&scene.session.id));
    assert!(scene.session.gateway.is_world_loaded());
}
