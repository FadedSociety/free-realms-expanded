use std::cell::Cell;

use osfr_tools::player_radial::{
    DEFAULT_REACH_M, INSPECT_ENTRY_ID, PlayerRadial, PublicItem, PublicLook, RadialError,
};
use osfr_tools::players::{
    InterestPolicy, PlayerPresentation, PlayerRegistry, SessionId, ZoneInstance,
};

const VIEWER: u64 = 11;
const OTHER: u64 = 22;
const FAR: u64 = 33;
const ELSEWHERE: u64 = 44;
const NEVER: u64 = 99;
// Private values that must never reach the wire; the look type has no field for them.
const CANARY: i32 = 0x5EC2_E7ED;

fn zone(instance: u64) -> ZoneInstance {
    ZoneInstance {
        zone_id: 1,
        instance_id: instance,
    }
}

fn join(
    registry: &mut PlayerRegistry,
    session: u64,
    guid: u64,
    zone: ZoneInstance,
    position: [f32; 3],
    name: &str,
) {
    let presentation = PlayerPresentation {
        character_guid: guid,
        first_name: name.into(),
        ..PlayerPresentation::default()
    };
    registry
        .join_with_presentation(
            SessionId(session),
            session,
            guid,
            zone,
            position,
            0.0,
            presentation,
        )
        .unwrap();
}

/// Viewer and OTHER 5 m apart; FAR 90 m away (inside the 100 m interest radius);
/// ELSEWHERE next to the viewer but in another instance.
fn world() -> PlayerRegistry {
    let mut r = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
    join(&mut r, 1, VIEWER, zone(1), [0.0, 0.0, 0.0], "Viewer");
    join(&mut r, 2, OTHER, zone(1), [5.0, 0.0, 0.0], "Other");
    join(&mut r, 3, FAR, zone(1), [90.0, 0.0, 0.0], "Far");
    join(&mut r, 4, ELSEWHERE, zone(2), [1.0, 0.0, 0.0], "Else");
    r
}

fn select(own: u64, target: u64) -> Vec<u8> {
    let mut b = vec![26, 0, 19, 0];
    b.extend(own.to_le_bytes());
    b.extend(target.to_le_bytes());
    b
}

fn choice(guid: u64, id: i32) -> Vec<u8> {
    let mut b = vec![26, 0, 10, 0];
    b.extend(guid.to_le_bytes());
    b.extend(id.to_le_bytes());
    b
}

fn record(id: i32, extra: usize) -> Vec<u8> {
    let mut b = id.to_le_bytes().to_vec();
    b.extend(vec![7u8; extra]);
    b
}

fn look() -> PublicLook {
    PublicLook {
        profiles: vec![(1, 3), (6, 1)],
        active_profile: 6,
        items: vec![
            PublicItem {
                slot: 2,
                definition_id: 1234,
                tint: 77,
                definition_record: record(1234, 10),
            },
            PublicItem {
                slot: 9,
                definition_id: 55,
                tint: 0,
                definition_record: record(55, 0),
            },
        ],
    }
}

fn viewer() -> SessionId {
    SessionId(1)
}

/// Reader that mirrors native `00ad83b0` order; returns every scalar it saw.
struct Payload {
    profiles: Vec<(i32, i32)>,
    active: i32,
    items: Vec<(i32, i32, i32, Vec<u8>)>,
    tail: Vec<i32>,
}

fn parse_inspect(packet: &[u8]) -> (u64, bool, Payload) {
    assert_eq!(&packet[..3], &[149, 0, 1]);
    let guid = u64::from_le_bytes(packet[3..11].try_into().unwrap());
    let pedestal = packet[11] == 1;
    let len = u32::from_le_bytes(packet[12..16].try_into().unwrap()) as usize;
    assert_eq!(
        packet.len(),
        16 + len,
        "length prefix covers the whole payload"
    );
    let body = &packet[16..];
    let mut at = 0;
    let i32_at = |at: &mut usize| {
        let v = i32::from_le_bytes(body[*at..*at + 4].try_into().unwrap());
        *at += 4;
        v
    };
    let n = i32_at(&mut at);
    let profiles = (0..n).map(|_| (i32_at(&mut at), i32_at(&mut at))).collect();
    let active = i32_at(&mut at);
    let n = i32_at(&mut at);
    let mut items = Vec::new();
    for _ in 0..n {
        let (slot, def, tint) = (i32_at(&mut at), i32_at(&mut at), i32_at(&mut at));
        // This test's records are: id + filler; the filler is 7s up to the next slot,
        // so read until the fixed 12-field tail starts.
        let record_len = 4 + body[at + 4..].iter().take_while(|&&b| b == 7).count();
        items.push((slot, def, tint, body[at..at + record_len].to_vec()));
        at += record_len;
    }
    let tail = (0..12).map(|_| i32_at(&mut at)).collect();
    assert_eq!(at, body.len(), "payload fully consumed");
    (
        guid,
        pedestal,
        Payload {
            profiles,
            active,
            items,
            tail,
        },
    )
}

fn open_menu(radial: &mut PlayerRadial, registry: &PlayerRegistry) {
    radial
        .select_player(registry, viewer(), &select(VIEWER, OTHER), |_| true)
        .unwrap();
}

#[test]
fn select_builds_a_one_entry_menu_for_the_target() {
    let registry = world();
    let mut radial = PlayerRadial::default();
    let out = radial
        .select_player(&registry, viewer(), &select(VIEWER, OTHER), |_| true)
        .unwrap();
    assert_eq!(out.len(), 1);
    let m = &out[0];
    assert_eq!(&m[..4], &[26, 0, 9, 0]);
    assert_eq!(u64::from_le_bytes(m[4..12].try_into().unwrap()), OTHER);
    assert_eq!(m[12], 0, "no auto-select");
    assert_eq!(i32::from_le_bytes(m[13..17].try_into().unwrap()), 1);
    let row: Vec<i32> = (0..7)
        .map(|i| i32::from_le_bytes(m[17 + 4 * i..21 + 4 * i].try_into().unwrap()))
        .collect();
    assert_eq!(row, [INSPECT_ENTRY_ID, 133, 3902, 0, 0, 0, 0]);
    let name_len = i32::from_le_bytes(m[45..49].try_into().unwrap()) as usize;
    assert_eq!(&m[49..49 + name_len], b"Other");
    assert_eq!(&m[49 + name_len..], &[1, 0]);
    assert_eq!(radial.menu_target(), Some(OTHER));
}

#[test]
fn forged_own_guid_field_changes_nothing() {
    let registry = world();
    let mut a = PlayerRadial::default();
    let mut b = PlayerRadial::default();
    let honest = a
        .select_player(&registry, viewer(), &select(VIEWER, OTHER), |_| true)
        .unwrap();
    // Claims to be somebody else, even the target: the reply is still the viewer's.
    for forged in [OTHER, FAR, 0, u64::MAX] {
        let got = b
            .select_player(&registry, viewer(), &select(forged, OTHER), |_| true)
            .unwrap();
        assert_eq!(got, honest);
    }
}

#[test]
fn every_refusal_sends_nothing_and_leaves_no_menu() {
    let registry = world();
    let cases: [(Vec<u8>, RadialError); 6] = [
        (select(VIEWER, VIEWER), RadialError::SelfTarget),
        (select(VIEWER, NEVER), RadialError::NotVisible),
        (select(VIEWER, 0), RadialError::NotVisible),
        (select(VIEWER, ELSEWHERE), RadialError::NotVisible),
        (select(VIEWER, u64::MAX), RadialError::NotVisible),
        (select(VIEWER, OTHER)[..19].to_vec(), RadialError::Malformed),
    ];
    for (packet, expected) in cases {
        let mut radial = PlayerRadial::default();
        assert_eq!(
            radial.select_player(&registry, viewer(), &packet, |_| true),
            Err(expected)
        );
        assert_eq!(radial.menu_target(), None);
    }
    // A viewer that never joined gets nothing either.
    let mut radial = PlayerRadial::default();
    assert_eq!(
        radial.select_player(&registry, SessionId(77), &select(VIEWER, OTHER), |_| true),
        Err(RadialError::NotInWorld)
    );
}

#[test]
fn malformed_requests_are_rejected_without_panics() {
    let registry = world();
    let good = select(VIEWER, OTHER);
    for len in 0..good.len() {
        let mut radial = PlayerRadial::default();
        assert!(
            radial
                .select_player(&registry, viewer(), &good[..len], |_| true)
                .is_err()
        );
    }
    let mut long = good.clone();
    long.push(0);
    let mut wrong_sub = good.clone();
    wrong_sub[2] = 20;
    for bad in [long, wrong_sub] {
        assert_eq!(
            PlayerRadial::default().select_player(&registry, viewer(), &bad, |_| true),
            Err(RadialError::Malformed)
        );
    }
}

#[test]
fn reach_limit_is_tunable_and_bounded() {
    let registry = world();
    // FAR is 90 m away: inside interest, beyond the 40 m default reach.
    assert!(DEFAULT_REACH_M < 90.0);
    assert_eq!(
        PlayerRadial::default().select_player(&registry, viewer(), &select(VIEWER, FAR), |_| true),
        Err(RadialError::TooFar)
    );
    let mut wide = PlayerRadial::with_reach(95.0).unwrap();
    assert!(
        wide.select_player(&registry, viewer(), &select(VIEWER, FAR), |_| true)
            .is_ok()
    );
    let mut tight = PlayerRadial::with_reach(2.0).unwrap();
    assert_eq!(
        tight.select_player(&registry, viewer(), &select(VIEWER, OTHER), |_| true),
        Err(RadialError::TooFar)
    );
    for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(PlayerRadial::with_reach(bad).is_none());
    }
}

#[test]
fn name_is_bounded_and_utf8_safe() {
    let mut registry = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
    join(&mut registry, 1, VIEWER, zone(1), [0.0; 3], "V");
    let long = "é".repeat(200);
    join(&mut registry, 2, OTHER, zone(1), [1.0, 0.0, 0.0], &long);
    let out = PlayerRadial::default()
        .select_player(&registry, viewer(), &select(VIEWER, OTHER), |_| true)
        .unwrap();
    let m = &out[0];
    let n = i32::from_le_bytes(m[45..49].try_into().unwrap()) as usize;
    assert!(n <= 64);
    assert!(std::str::from_utf8(&m[49..49 + n]).is_ok());
}

#[test]
fn inspect_choice_sends_the_public_look_only() {
    let registry = world();
    let mut radial = PlayerRadial::default();
    open_menu(&mut radial, &registry);
    let calls = Cell::new(0);
    let out = radial
        .choose(&registry, viewer(), &choice(OTHER, INSPECT_ENTRY_ID), |g| {
            calls.set(calls.get() + 1);
            assert_eq!(g, OTHER);
            Some(look())
        })
        .unwrap()
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(out.len(), 1);
    let (guid, pedestal, p) = parse_inspect(&out[0]);
    assert_eq!(guid, OTHER);
    assert!(pedestal);
    assert_eq!(p.profiles, [(1, 3), (6, 1)]);
    assert_eq!(p.active, 6);
    assert_eq!(p.items.len(), 2);
    assert_eq!((p.items[0].0, p.items[0].1, p.items[0].2), (2, 1234, 77));
    assert_eq!(p.items[0].3, record(1234, 10));
    assert_eq!(p.items[1].3, record(55, 0));
    // vip rank f32, icon, vip title, active title, coins, levels, collections,
    // quests, stats, string, plus the two leading achievement words.
    assert_eq!(p.tail, [0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0]);
    // Canary: nothing private exists to leak; a scan for the marker finds nothing.
    assert!(!out[0].windows(4).any(|w| w == CANARY.to_le_bytes()));
    assert_eq!(radial.inspecting(), Some(OTHER));
    assert_eq!(radial.menu_target(), None, "menu is one-shot");
}

#[test]
fn replay_and_foreign_choices_do_not_inspect() {
    let registry = world();
    let mut radial = PlayerRadial::default();
    let never = |_| -> Option<PublicLook> { panic!("look must not be read") };
    // No menu open: passes through to the NPC path.
    assert_eq!(
        radial.choose(&registry, viewer(), &choice(OTHER, INSPECT_ENTRY_ID), never),
        Ok(None)
    );
    open_menu(&mut radial, &registry);
    // Different guid (an NPC or another player): not ours, menu stays open.
    assert_eq!(
        radial.choose(&registry, viewer(), &choice(FAR, INSPECT_ENTRY_ID), never),
        Ok(None)
    );
    assert_eq!(radial.menu_target(), Some(OTHER));
    // Wrong width is not a 26/10 for us either.
    assert_eq!(
        radial.choose(&registry, viewer(), &choice(OTHER, 1)[..15], never),
        Ok(None)
    );
    // Right guid, id never offered (e.g. an unoffered Trade id): refused, menu gone.
    for id in [0, 1, 103, 1002, i32::MIN, i32::MAX] {
        open_menu(&mut radial, &registry);
        assert_eq!(
            radial.choose(&registry, viewer(), &choice(OTHER, id), never),
            Err(RadialError::InvalidChoice)
        );
        assert_eq!(radial.menu_target(), None);
    }
    // Replay after a successful inspect does nothing.
    open_menu(&mut radial, &registry);
    radial
        .choose(
            &registry,
            viewer(),
            &choice(OTHER, INSPECT_ENTRY_ID),
            |_| Some(look()),
        )
        .unwrap();
    assert_eq!(
        radial.choose(&registry, viewer(), &choice(OTHER, INSPECT_ENTRY_ID), never),
        Ok(None)
    );
}

#[test]
fn stale_target_at_choice_time_reveals_nothing() {
    let mut registry = world();
    let mut radial = PlayerRadial::default();
    open_menu(&mut radial, &registry);
    registry.leave(SessionId(2));
    let never = |_| -> Option<PublicLook> { panic!("look must not be read") };
    assert_eq!(
        radial.choose(&registry, viewer(), &choice(OTHER, INSPECT_ENTRY_ID), never),
        Err(RadialError::NotVisible)
    );
    assert_eq!(radial.inspecting(), None);
    // Same when the target walks out of reach after the menu opened.
    let mut registry = world();
    let mut radial = PlayerRadial::default();
    open_menu(&mut radial, &registry);
    registry
        .update_pose(SessionId(2), zone(1), [60.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert_eq!(
        radial.choose(&registry, viewer(), &choice(OTHER, INSPECT_ENTRY_ID), never),
        Err(RadialError::TooFar)
    );
    // And when the target changes instance.
    let mut registry = world();
    let mut radial = PlayerRadial::default();
    open_menu(&mut radial, &registry);
    registry
        .update_pose(SessionId(2), zone(2), [5.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert_eq!(
        radial.choose(&registry, viewer(), &choice(OTHER, INSPECT_ENTRY_ID), never),
        Err(RadialError::NotVisible)
    );
}

#[test]
fn missing_or_invalid_look_is_refused() {
    let registry = world();
    let run = |look: Option<PublicLook>| {
        let mut radial = PlayerRadial::default();
        open_menu(&mut radial, &registry);
        let r = radial.choose(
            &registry,
            viewer(),
            &choice(OTHER, INSPECT_ENTRY_ID),
            |_| look,
        );
        assert_eq!(radial.inspecting(), None);
        r
    };
    assert_eq!(run(None), Err(RadialError::NoLook));
    let mut bad = look();
    bad.items[0].slot = 0;
    assert!(matches!(run(Some(bad)), Err(RadialError::BadLook(_))));
    let mut bad = look();
    bad.items[0].slot = 18;
    assert!(matches!(run(Some(bad)), Err(RadialError::BadLook(_))));
    let mut bad = look();
    bad.items[1].slot = bad.items[0].slot;
    assert!(matches!(run(Some(bad)), Err(RadialError::BadLook(_))));
    let mut bad = look();
    bad.items[0].definition_id = 1; // record starts with 1234
    assert!(matches!(run(Some(bad)), Err(RadialError::BadLook(_))));
    let mut bad = look();
    bad.items[0].definition_record = vec![0xD2, 0x04, 0]; // too short for an id
    assert!(matches!(run(Some(bad)), Err(RadialError::BadLook(_))));
    let mut bad = look();
    bad.items[0].definition_record = record(1234, 70_000);
    assert!(matches!(run(Some(bad)), Err(RadialError::BadLook(_))));
    let mut bad = look();
    bad.profiles = vec![(1, 1); 65];
    assert!(matches!(run(Some(bad)), Err(RadialError::BadLook(_))));
    let mut bad = look();
    bad.items = (1..=18)
        .map(|s| PublicItem {
            slot: s,
            definition_id: 5,
            tint: 0,
            definition_record: record(5, 0),
        })
        .collect();
    assert!(matches!(run(Some(bad)), Err(RadialError::BadLook(_))));
}

#[test]
fn stop_inspect_closes_idempotently_and_is_strict() {
    let registry = world();
    let mut radial = PlayerRadial::default();
    open_menu(&mut radial, &registry);
    radial
        .choose(
            &registry,
            viewer(),
            &choice(OTHER, INSPECT_ENTRY_ID),
            |_| Some(look()),
        )
        .unwrap();
    assert_eq!(radial.inspecting(), Some(OTHER));
    for bad in [
        &[149u8, 0][..],
        &[149, 0, 2, 0],
        &[149, 0, 1],
        &[150, 0, 2],
        &[],
    ] {
        assert_eq!(radial.stop_inspect(bad), Err(RadialError::Malformed));
        assert_eq!(
            radial.inspecting(),
            Some(OTHER),
            "bad close leaves state alone"
        );
    }
    assert_eq!(radial.stop_inspect(&[149, 0, 2]), Ok(()));
    assert_eq!(radial.inspecting(), None);
    assert_eq!(radial.stop_inspect(&[149, 0, 2]), Ok(()));
}

#[test]
fn reset_clears_menu_and_inspect() {
    let registry = world();
    let mut radial = PlayerRadial::default();
    open_menu(&mut radial, &registry);
    radial.reset();
    assert_eq!((radial.menu_target(), radial.inspecting()), (None, None));
}

#[test]
fn a_new_selection_replaces_the_open_menu() {
    let registry = world();
    let mut radial = PlayerRadial::default();
    open_menu(&mut radial, &registry);
    // A refused follow-up click also closes the old menu (fail closed).
    assert!(
        radial
            .select_player(&registry, viewer(), &select(VIEWER, NEVER), |_| true)
            .is_err()
    );
    assert_eq!(radial.menu_target(), None);
    open_menu(&mut radial, &registry);
    radial
        .select_player(&registry, viewer(), &select(VIEWER, FAR), |_| true)
        .unwrap_err();
    assert_eq!(radial.menu_target(), None);
}

#[test]
fn no_public_look_means_no_menu_and_hidden_targets_are_never_asked() {
    let registry = world();
    let mut radial = PlayerRadial::default();
    assert_eq!(
        radial.select_player(&registry, viewer(), &select(VIEWER, OTHER), |g| {
            assert_eq!(g, OTHER);
            false
        }),
        Err(RadialError::NoLook)
    );
    assert_eq!(radial.menu_target(), None);
    // Canary: the look lookup is never consulted for a refused target.
    for target in [VIEWER, NEVER, ELSEWHERE, FAR] {
        let r = radial.select_player(&registry, viewer(), &select(VIEWER, target), |_| {
            panic!("look source consulted for a refused target")
        });
        assert!(r.is_err());
    }
}
