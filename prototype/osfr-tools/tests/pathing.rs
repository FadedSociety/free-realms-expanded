//! W164: the 98/1 path request and its 98/2 reply (layouts and evidence: `src/pathing.rs`).
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    pathing::{MAX_PER_SECOND, MAX_POINTS, Pathing, Refusal, STEP},
    road_graph::{CONTENT_FILE, RoadGraph},
    terrain::{HeightGrid, Terrain},
};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

#[path = "support/json.rs"]
mod json;
#[path = "support/node_content.rs"]
mod node_content;
#[path = "support/terrain.rs"]
mod terrain_support;

const ZONE: u32 = 7;
const PLAYER: [f32; 3] = [10.0, 1.0, 10.0];
static NEXT: AtomicU32 = AtomicU32::new(0);

/// Ground y = x / 10 over x in [-100, 500], z in [-100, 100], zone `ZONE`.
fn terrain() -> Terrain {
    let mut grid = HeightGrid::new([-100.0, -100.0], [500.0, 100.0], 2.0).unwrap();
    let p = |x: f32, z: f32| [x, x / 10.0, z];
    grid.add_triangle([p(-100.0, -100.0), p(500.0, -100.0), p(500.0, 100.0)]);
    grid.add_triangle([p(-100.0, -100.0), p(500.0, 100.0), p(-100.0, 100.0)]);
    load_terrain(ZONE, grid)
}

/// Flat ground at y = 3 over `min..max`, in `zone`.
fn flat_terrain(zone: u32, min: [f32; 2], max: [f32; 2]) -> Terrain {
    load_terrain(zone, terrain_support::surface(min, max, 20.0, |_, _| 3.0))
}

/// One zone's grid, loaded through the production loader from a temporary directory.
fn load_terrain(zone: u32, grid: HeightGrid) -> Terrain {
    let dir = temp_dir("terrain");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("zone_{zone}.hgt")), grid.to_bytes()).unwrap();
    let terrain = Terrain::load(&dir).unwrap();
    fs::remove_dir_all(&dir).unwrap();
    terrain
}

fn temp_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "osfr-w164-{tag}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

/// The shipped `data/content/road_graph.json`, through the production loader.
fn shipped_graph() -> RoadGraph {
    RoadGraph::load(&node_content::content_dir())
        .expect("the shipped road graph loads")
        .expect("the shipped road graph is present")
}

/// A synthetic road graph, loaded through the production loader from a temporary directory.
fn road_graph(document: serde_json::Value) -> RoadGraph {
    graph_load(document)
        .expect("the graph loads")
        .expect("the graph is present")
}

fn graph_load(document: serde_json::Value) -> std::io::Result<Option<RoadGraph>> {
    let dir = temp_dir("graph");
    json::write_pretty(&dir.join(CONTENT_FILE), &document);
    let graph = RoadGraph::load(&dir);
    fs::remove_dir_all(&dir).unwrap();
    graph
}

/// A five-node graph in `ZONE`: 0-1-2-4 is 30 m and 0-1-3-4 is about 39 m, so A* must
/// prefer the shorter branch instead of the first edge it sees.
fn five_node_graph() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "source": {"asset": "synthetic", "sha256": "", "decoder": "test"},
        "zone": ZONE,
        "snap_radius_m": 40,
        "max_route_m": 600,
        "nodes": [
            [0, 0.0, 0.0, 0.0, [1]],
            [1, 10.0, 0.0, 0.0, [0, 2, 3]],
            [2, 20.0, 0.0, 0.0, [1, 4]],
            [3, 15.0, 0.0, 10.0, [1, 4]],
            [4, 30.0, 0.0, 0.0, [2, 3]]
        ]
    })
}

/// Three nodes where the road from 0 to 2 swings through (15, 30): about 67 m of road
/// against 30 m straight.
fn detour_graph(max_route_m: f32) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "source": {"asset": "synthetic", "sha256": "", "decoder": "test"},
        "zone": ZONE,
        "snap_radius_m": 40,
        "max_route_m": max_route_m,
        "nodes": [
            [0, 0.0, 0.0, 0.0, [1]],
            [1, 15.0, 0.0, 30.0, [0, 2]],
            [2, 30.0, 0.0, 0.0, [1]]
        ]
    })
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
    path.answer(packet, ZONE, PLAYER, Some(terrain), None, now)
}

/// The request answered with `graph` on a fresh budget.
fn ask_road(graph: &RoadGraph, terrain: &Terrain, packet: &[u8]) -> Vec<u8> {
    let road = Some(graph);
    Pathing::default()
        .answer(packet, ZONE, PLAYER, Some(terrain), road, 0)
        .unwrap()
        .unwrap()
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
    let reply = path.answer(&walk(60.0), 99, PLAYER, Some(&terrain), None, 0);
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
        .answer(&walk(60.0), ZONE, PLAYER, None, None, 0)
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

/// Today's no-graph reply, the straight-line fallback the road cases must reproduce.
fn straight_reply(terrain: &Terrain, packet: &[u8]) -> Vec<u8> {
    ask(&mut Pathing::default(), terrain, packet, 0)
        .unwrap()
        .unwrap()
}

/// T554: A* over the synthetic graph prefers the shorter road and repeats exactly.
#[test]
fn astar_follows_the_shortest_road_on_a_five_node_graph() {
    let graph = road_graph(five_node_graph());
    let expected = vec![
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [20.0, 0.0, 0.0],
        [30.0, 0.0, 0.0],
    ];
    assert_eq!(
        graph.route([0.0, 0.0, 0.0], [30.0, 0.0, 0.0]),
        Some(expected)
    );
    assert_eq!(
        graph.route([0.0, 0.0, 0.0], [30.0, 0.0, 0.0]),
        graph.route([0.0, 0.0, 0.0], [30.0, 0.0, 0.0]),
        "the same request gives the same route"
    );
}

/// Both ends must be within the graph's snap radius; a far end has no road route.
#[test]
fn an_end_outside_the_snap_radius_has_no_road_route() {
    let graph = road_graph(five_node_graph());
    assert!(graph.route([0.0, 0.0, 0.0], [30.0, 0.0, 0.0]).is_some());
    assert!(graph.route([0.0, 0.0, 0.0], [30.0, 0.0, 100.0]).is_none());
    assert!(graph.route([100.0, 0.0, 0.0], [30.0, 0.0, 0.0]).is_none());
}

#[test]
fn broken_and_out_of_bounds_graphs_are_refused_at_load() {
    let mut dangling = five_node_graph();
    dangling["nodes"][0][4] = serde_json::json!([99]);
    assert!(graph_load(dangling).is_err(), "dangling neighbour");

    let mut one_way = five_node_graph();
    one_way["nodes"][1][4] = serde_json::json!([0, 2]);
    assert!(graph_load(one_way).is_err(), "one-way edge");

    for (field, value) in [
        ("snap_radius_m", 4.0),
        ("snap_radius_m", 201.0),
        ("max_route_m", 49.0),
        ("max_route_m", 5001.0),
    ] {
        let mut document = five_node_graph();
        document[field] = serde_json::json!(value);
        assert!(graph_load(document).is_err(), "{field} {value}");
    }
    // A content set with no graph is not an error: it answers straight lines.
    assert!(RoadGraph::load(&temp_dir("missing")).unwrap().is_none());
}

/// The graph belongs to its own zone (FabledRealms is zone 0); the same request on the
/// zone-7 terrain is exactly today's straight line.
#[test]
fn a_graph_for_another_zone_keeps_the_straight_line() {
    let terrain = terrain();
    let straight = straight_reply(&terrain, &walk(110.0));
    let reply = ask_road(&shipped_graph(), &terrain, &walk(110.0));
    assert_eq!(reply, straight);
    assert_eq!(decode(&reply).points.len(), 21);
}

#[test]
fn an_end_outside_the_snap_radius_keeps_the_straight_line() {
    let terrain = flat_terrain(ZONE, [-10.0, -10.0], [40.0, 110.0]);
    let packet = request(1, [0.0, 3.0, 0.0, 1.0], [30.0, 3.0, 100.0, 1.0]);
    let straight = straight_reply(&terrain, &packet);
    let reply = ask_road(&road_graph(five_node_graph()), &terrain, &packet);
    assert_eq!(reply, straight);
}

#[test]
fn a_disconnected_road_keeps_the_straight_line() {
    let terrain = flat_terrain(ZONE, [-10.0, -10.0], [40.0, 40.0]);
    let packet = request(1, [0.0, 3.0, 0.0, 1.0], [30.0, 3.0, 0.0, 1.0]);
    let mut document = detour_graph(600.0);
    document["nodes"][1][4] = serde_json::json!([0]);
    document["nodes"][2] = serde_json::json!([2, 30.0, 0.0, 0.0, []]);
    let straight = straight_reply(&terrain, &packet);
    let reply = ask_road(&road_graph(document), &terrain, &packet);
    assert_eq!(reply, straight);
}

#[test]
fn a_route_longer_than_the_limit_keeps_the_straight_line() {
    let terrain = flat_terrain(ZONE, [-10.0, -10.0], [40.0, 40.0]);
    let packet = request(1, [0.0, 3.0, 0.0, 1.0], [30.0, 3.0, 0.0, 1.0]);
    let straight = straight_reply(&terrain, &packet);
    // About 67 m of road against a 50 m limit.
    let limited = ask_road(&road_graph(detour_graph(50.0)), &terrain, &packet);
    assert_eq!(limited, straight);
    // The same road fits a 600 m limit, so the limit is what fell back.
    let allowed = ask_road(&road_graph(detour_graph(600.0)), &terrain, &packet);
    assert_ne!(allowed, straight);
    assert_eq!(decode(&allowed).points.len(), 15);
}

/// Planar distance from a point to a segment in the x/z plane.
fn point_segment(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
    let length_sq = dx * dx + dz * dz;
    let t = if length_sq == 0.0 {
        0.0
    } else {
        (((point[0] - a[0]) * dx + (point[1] - a[1]) * dz) / length_sq).clamp(0.0, 1.0)
    };
    ((point[0] - a[0] - t * dx).powi(2) + (point[1] - a[1] - t * dz).powi(2)).sqrt()
}

/// Planar distance from (x, z) to the nearest road edge in the shipped graph document.
fn distance_to_edges(document: &serde_json::Value, x: f32, z: f32) -> f32 {
    let rows = document["nodes"].as_array().unwrap();
    let positions: Vec<[f32; 2]> = rows
        .iter()
        .map(|row| {
            [
                row[1].as_f64().unwrap() as f32,
                row[3].as_f64().unwrap() as f32,
            ]
        })
        .collect();
    let mut nearest = f32::INFINITY;
    for row in rows {
        let a = positions[row[0].as_u64().unwrap() as usize];
        for neighbour in row[4].as_array().unwrap() {
            let b = positions[neighbour.as_u64().unwrap() as usize];
            nearest = nearest.min(point_segment([x, z], a, b));
        }
    }
    nearest
}

/// The 213 m request between FabledRealms road nodes 316 and 851, and the flat zone-0 ground
/// it is answered on.
const ROAD_FROM: [f32; 4] = [-600.49866, -1.0554962, -127.82866, 1.0];
const ROAD_TO: [f32; 4] = [-640.18634, -1.4318695, -336.721, 1.0];

fn road_terrain() -> Terrain {
    flat_terrain(0, [-650.0, -355.0], [-438.0, -118.0])
}

/// The decoded reply must follow the shipped graph's edges: every sample within 5 m of an
/// edge, within the point cap, with the client's own two ends, and bent away from the
/// direct line (the furthest sample is 172 m off it), so the graph answered and not the
/// straight-line fallback.
fn assert_follows_the_roads(reply: &[u8]) {
    let document = json::read(&node_content::content_dir().join(CONTENT_FILE));
    let decoded = decode(reply);
    assert!(decoded.points.len() <= MAX_POINTS);
    assert_eq!(decoded.points[0], ROAD_FROM);
    assert_eq!(*decoded.points.last().unwrap(), ROAD_TO);
    for point in &decoded.points {
        assert!(
            distance_to_edges(&document, point[0], point[2]) <= 5.0,
            "sample {point:?} is not on a road edge"
        );
    }
    let (from, to) = ([ROAD_FROM[0], ROAD_FROM[2]], [ROAD_TO[0], ROAD_TO[2]]);
    let off_line = decoded
        .points
        .iter()
        .map(|point| point_segment([point[0], point[2]], from, to))
        .fold(0.0, f32::max);
    assert!(off_line > 20.0, "the reply is the straight line");
}

/// T554 gate: the shipped graph answers the request along the roads.
#[test]
fn a_shipped_road_request_follows_the_graphs_edges() {
    let player = [ROAD_FROM[0], ROAD_FROM[1], ROAD_FROM[2]];
    let reply = Pathing::default()
        .answer(
            &request(1, ROAD_FROM, ROAD_TO),
            0,
            player,
            Some(&road_terrain()),
            Some(&shipped_graph()),
            0,
        )
        .unwrap()
        .unwrap();
    assert_follows_the_roads(&reply);
}

/// The production binding: `Gameplay::load` reads the content set's `road_graph.json` and
/// hands it to the path request, for both kinds, and a zone without a graph gets the
/// straight line.
#[test]
fn a_loaded_game_answers_road_routes_from_its_content_set() {
    let mut game = Gameplay::load(&node_content::content_dir(), None).unwrap();
    game.set_terrain(Some(Arc::new(road_terrain())));
    let player = |zone| CharacterState {
        guid: 1,
        zone_id: zone,
        position: [ROAD_FROM[0], ROAD_FROM[1], ROAD_FROM[2]],
        rotation: [1.0, 0.0, 0.0],
    };
    let mut reply = |kind: u32, zone: u32, at: u64| {
        let out = game
            .handle_in_world(&request(kind, ROAD_FROM, ROAD_TO), &player(zone), at)
            .unwrap();
        assert_eq!(out.replies.len(), 1);
        out.replies.into_iter().next().unwrap()
    };
    assert_follows_the_roads(&reply(1, 0, 1_000));
    let take_me_there = reply(2, 0, 2_000);
    assert_eq!(decode(&take_me_there).kind, 2);
    assert_follows_the_roads(&take_me_there);
    // Zone 99 has no height grid: the client's empty failure reply, not a road route.
    assert!(decode(&reply(1, 99, 3_000)).points.is_empty());
}
