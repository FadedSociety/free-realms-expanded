//! W164: the 98/1 path request and its 98/2 reply (layouts and evidence: `src/pathing.rs`).
use osfr_tools::{
    pathing::{MAX_PER_SECOND, MAX_POINTS, Pathing, Refusal, STEP},
    terrain::{HeightGrid, Terrain},
};
use std::{
    fs,
    sync::atomic::{AtomicU32, Ordering},
};

const ZONE: u32 = 7;
const PLAYER: [f32; 3] = [10.0, 1.0, 10.0];
static NEXT: AtomicU32 = AtomicU32::new(0);

/// Ground y = x / 10 over x in [-100, 500], z in [-100, 100], zone `ZONE`.
fn terrain() -> Terrain {
    let mut grid = HeightGrid::new([-100.0, -100.0], [500.0, 100.0], 2.0).unwrap();
    let p = |x: f32, z: f32| [x, x / 10.0, z];
    grid.add_triangle([p(-100.0, -100.0), p(500.0, -100.0), p(500.0, 100.0)]);
    grid.add_triangle([p(-100.0, -100.0), p(500.0, 100.0), p(-100.0, 100.0)]);
    let dir = std::env::temp_dir().join(format!(
        "osfr-w164-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("zone_{ZONE}.hgt")), grid.to_bytes()).unwrap();
    let terrain = Terrain::load(&dir).unwrap();
    fs::remove_dir_all(&dir).unwrap();
    terrain
}

/// A 98/1 request: handle, kind, aux, two vec4, tail.
fn request(kind: u32, from: [f32; 4], to: [f32; 4]) -> Vec<u8> {
    let mut p = vec![98, 0, 1];
    p.extend(0x1234_5678_9abc_def0u64.to_le_bytes());
    p.extend(kind.to_le_bytes());
    p.extend(7u32.to_le_bytes());
    for v in from.iter().chain(&to) {
        p.extend(v.to_le_bytes());
    }
    p.extend(0u32.to_le_bytes());
    p
}

/// From the player's position along +x to `to_x`, at the player's height.
fn walk(to_x: f32) -> Vec<u8> {
    request(1, [10.0, 1.0, 10.0, 1.0], [to_x, 1.0, 10.0, 1.0])
}

struct Decoded {
    kind: u32,
    aux: u32,
    points: Vec<[f32; 4]>,
}

fn decode(reply: &[u8]) -> Decoded {
    assert_eq!(&reply[..3], &[98, 0, 2]);
    let word = |at: usize| u32::from_le_bytes(reply[at..at + 4].try_into().unwrap());
    let count = word(11) as usize;
    assert_eq!(reply.len(), 15 + count * 16, "count matches the body");
    let points = (0..count)
        .map(|i| std::array::from_fn(|k| f32::from_bits(word(15 + i * 16 + k * 4))))
        .collect();
    Decoded {
        kind: word(3),
        aux: word(7),
        points,
    }
}

type Answer = Option<Result<Vec<u8>, Refusal>>;

fn ask(path: &mut Pathing, terrain: &Terrain, packet: &[u8], now: u64) -> Answer {
    path.answer(packet, ZONE, PLAYER, Some(terrain), now)
}

#[test]
fn a_valid_request_gets_a_ground_following_path() {
    let (terrain, mut path) = (terrain(), Pathing::default());
    let reply = ask(&mut path, &terrain, &walk(110.0), 0).unwrap().unwrap();
    let d = decode(&reply);
    assert_eq!((d.kind, d.aux), (1, 7), "kind and aux are echoed");
    // 100 m at 5 m: 20 segments, 21 points, the client's own ends, ground between.
    assert_eq!(d.points.len(), 21);
    assert_eq!(d.points[0], [10.0, 1.0, 10.0, 1.0]);
    assert_eq!(d.points[20], [110.0, 1.0, 10.0, 1.0]);
    for pair in d.points.windows(2) {
        assert!((pair[1][0] - pair[0][0] - STEP).abs() < 1e-3);
    }
    for p in &d.points[1..20] {
        assert!(
            (p[1] - p[0] / 10.0).abs() < 0.1,
            "y follows the ground: {p:?}"
        );
    }
}

#[test]
fn the_longest_legal_request_stays_under_the_point_cap() {
    let (terrain, mut path) = (terrain(), Pathing::default());
    // Exactly the client's 300 m limit is answered; a 5 m rise on top of it is not.
    let d = decode(&ask(&mut path, &terrain, &walk(310.0), 0).unwrap().unwrap());
    assert!(d.points.len() <= MAX_POINTS);
    assert_eq!(d.points.len(), 61);
    let over = request(1, [10.0, 0.0, 10.0, 1.0], [310.0, 5.0, 10.0, 1.0]);
    assert_eq!(
        ask(&mut path, &terrain, &over, 0),
        Some(Err(Refusal::TooFar))
    );
}

#[test]
fn nearby_and_identical_points_give_a_short_path() {
    let (terrain, mut path) = (terrain(), Pathing::default());
    let same = request(1, [10.0, 1.0, 10.0, 1.0], [10.0, 1.0, 10.0, 1.0]);
    let reply = ask(&mut path, &terrain, &same, 0).unwrap().unwrap();
    assert_eq!(decode(&reply).points.len(), 2);
    let near = ask(&mut path, &terrain, &walk(12.0), 0).unwrap().unwrap();
    assert_eq!(decode(&near).points.len(), 2);
}

#[test]
fn refusals_each_have_a_canary() {
    let terrain = terrain();
    // The unmodified request is answered; every case below changes one thing in it.
    let mut control = Pathing::default();
    assert!(ask(&mut control, &terrain, &walk(60.0), 0).unwrap().is_ok());
    let refused = |packet: Vec<u8>, why: Refusal| {
        let mut fresh = Pathing::default();
        assert_eq!(
            ask(&mut fresh, &terrain, &packet, 0),
            Some(Err(why)),
            "{why:?}"
        );
    };
    let from = [10.0, 1.0, 10.0, 1.0];
    // Too far: 301 m.
    refused(walk(311.0), Refusal::TooFar);
    // Not finite, in every position class.
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        refused(request(1, from, [bad, 0.0, 10.0, 1.0]), Refusal::NotFinite);
        refused(
            request(1, [10.0, bad, 10.0, 1.0], [60.0, 0.0, 10.0, 1.0]),
            Refusal::NotFinite,
        );
        refused(
            request(1, [10.0, 1.0, 10.0, bad], [60.0, 0.0, 10.0, 1.0]),
            Refusal::NotFinite,
        );
    }
    // Finite, but the distance sum overflows.
    refused(
        request(1, [1e30, 0.0, 0.0, 1.0], [-1e30, 0.0, 0.0, 1.0]),
        Refusal::TooFar,
    );
    // Out of the zone: the goal is off the height grid...
    refused(
        request(1, from, [10.0, 1.0, 150.0, 1.0]),
        Refusal::OutOfZone,
    );
    refused(
        request(1, from, [-150.0, 1.0, 10.0, 1.0]),
        Refusal::OutOfZone,
    );
    // ...or the start is nowhere near the player's own position.
    refused(
        request(1, [400.0, 1.0, 10.0, 1.0], [410.0, 1.0, 10.0, 1.0]),
        Refusal::OutOfZone,
    );
    // Malformed: truncated, a trailing byte, header only.
    let good = walk(60.0);
    refused(good[..good.len() - 1].to_vec(), Refusal::Malformed);
    refused([good.clone(), vec![0]].concat(), Refusal::Malformed);
    refused(vec![98, 0, 1], Refusal::Malformed);
}

#[test]
fn a_zone_without_a_grid_gets_the_failure_reply_not_a_path() {
    let terrain = terrain();
    let mut path = Pathing::default();
    let reply = path.answer(&walk(60.0), 99, PLAYER, Some(&terrain), 0);
    assert!(decode(&reply.unwrap().unwrap()).points.is_empty());
}

#[test]
fn refused_requests_leave_the_rate_budget_alone() {
    let (terrain, mut path) = (terrain(), Pathing::default());
    for _ in 0..50 {
        assert!(ask(&mut path, &terrain, &walk(400.0), 0).unwrap().is_err());
    }
    for _ in 0..MAX_PER_SECOND {
        assert!(ask(&mut path, &terrain, &walk(60.0), 0).unwrap().is_ok());
    }
}

#[test]
fn the_rate_cap_holds_and_resets_each_second() {
    let (terrain, mut path) = (terrain(), Pathing::default());
    for i in 0..MAX_PER_SECOND {
        let at = 5_000 + u64::from(i) * 100;
        assert!(ask(&mut path, &terrain, &walk(60.0), at).unwrap().is_ok());
    }
    for at in [5_300, 5_900, 5_999] {
        assert_eq!(
            ask(&mut path, &terrain, &walk(60.0), at),
            Some(Err(Refusal::TooFast))
        );
    }
    assert!(
        ask(&mut path, &terrain, &walk(60.0), 6_000)
            .unwrap()
            .is_ok(),
        "next window"
    );
    for _ in 1..MAX_PER_SECOND {
        assert!(
            ask(&mut path, &terrain, &walk(60.0), 6_000)
                .unwrap()
                .is_ok()
        );
    }
    // A clock that steps backwards cannot reopen a spent window.
    assert_eq!(
        ask(&mut path, &terrain, &walk(60.0), 100),
        Some(Err(Refusal::TooFast))
    );
}

#[test]
fn missing_terrain_gets_the_clients_failure_reply() {
    let mut path = Pathing::default();
    let reply = path
        .answer(&walk(60.0), ZONE, PLAYER, None, 0)
        .unwrap()
        .unwrap();
    // 98/2, kind 1, aux 7, zero nodes: HandleKynapsePathReply's "no path generated" branch.
    let expected = [
        &[98, 0, 2][..],
        &1u32.to_le_bytes(),
        &7u32.to_le_bytes(),
        &0i32.to_le_bytes(),
    ];
    assert_eq!(reply, expected.concat());
}

/// T404: Take Me There (`009ce560` writes kind 2). Its reply reader `009cead0` takes the
/// same count and nodes as kind 1's; the kind routes the reply there.
#[test]
fn take_me_there_gets_the_breadcrumb_path_under_its_own_kind() {
    let terrain = terrain();
    let (from, to) = ([10.0, 1.0, 10.0, 1.0], [110.0, 1.0, 10.0, 1.0]);
    let answer = |kind: u32| {
        let packet = request(kind, from, to);
        decode(
            &ask(&mut Pathing::default(), &terrain, &packet, 0)
                .unwrap()
                .unwrap(),
        )
    };
    let (breadcrumb, take_me_there) = (answer(1), answer(2));
    assert_eq!((take_me_there.kind, take_me_there.aux), (2, 7));
    assert_eq!(take_me_there.points.len(), 21);
    assert_eq!(take_me_there.points, breadcrumb.points);
    // Five metres from the objective (the native run's second click): the client's own
    // two points, byte for byte.
    let near = [15.0, 1.5, 10.0, 1.0];
    let reply = ask(
        &mut Pathing::default(),
        &terrain,
        &request(2, from, near),
        0,
    );
    let mut expected = vec![98, 0, 2];
    expected.extend([2u32, 7].iter().flat_map(|w| w.to_le_bytes()));
    expected.extend(2i32.to_le_bytes());
    expected.extend(from.iter().chain(&near).flat_map(|v| v.to_le_bytes()));
    assert_eq!(reply, Some(Ok(expected)));
}

#[test]
fn take_me_there_is_refused_and_rate_capped_like_the_breadcrumb() {
    let terrain = terrain();
    let from = [10.0, 1.0, 10.0, 1.0];
    let refused = [
        (request(2, from, [311.0, 1.0, 10.0, 1.0]), Refusal::TooFar),
        (
            request(2, from, [f32::NAN, 1.0, 10.0, 1.0]),
            Refusal::NotFinite,
        ),
        (
            request(2, from, [10.0, 1.0, 150.0, 1.0]),
            Refusal::OutOfZone,
        ),
        (
            request(2, [400.0, 1.0, 10.0, 1.0], [410.0, 1.0, 10.0, 1.0]),
            Refusal::OutOfZone,
        ),
    ];
    // Refusals send nothing and spend none of the budget the two kinds share.
    let mut path = Pathing::default();
    for (packet, why) in &refused {
        for _ in 0..10 {
            assert_eq!(
                ask(&mut path, &terrain, packet, 0),
                Some(Err(*why)),
                "{why:?}"
            );
        }
    }
    let take_me_there = request(2, from, [60.0, 1.0, 10.0, 1.0]);
    for i in 0..MAX_PER_SECOND {
        let packet = if i % 2 == 0 {
            walk(60.0)
        } else {
            take_me_there.clone()
        };
        assert!(ask(&mut path, &terrain, &packet, 0).unwrap().is_ok(), "{i}");
    }
    assert_eq!(
        ask(&mut path, &terrain, &take_me_there, 999),
        Some(Err(Refusal::TooFast))
    );
}

#[test]
fn other_packets_and_other_kinds_are_not_ours() {
    let (terrain, mut path) = (terrain(), Pathing::default());
    for kind in [0, 3, 4, u32::MAX] {
        let packet = request(kind, [10.0, 1.0, 10.0, 1.0], [60.0, 1.0, 10.0, 1.0]);
        assert_eq!(ask(&mut path, &terrain, &packet, 0), None, "kind {kind}");
    }
    for packet in [
        &[][..],
        &[98][..],
        &[98, 0][..],
        &[98, 0, 2, 0][..],
        &[47, 0, 1][..],
    ] {
        assert_eq!(ask(&mut path, &terrain, packet, 0), None);
    }
}
