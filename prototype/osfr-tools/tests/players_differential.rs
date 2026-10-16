//! W96: exact ordered messages and session state against the pre-grid algorithm.
#[path = "players_reference.rs"]
mod old;

use osfr_tools::players::{
    InterestPolicy, PlayerPresentation, PlayerRegistry, RegistryError, SessionId, ZoneInstance,
};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn position(&mut self) -> [f32; 3] {
        // Quantization deliberately creates equal distances and cell/radius boundaries.
        std::array::from_fn(|_| ((self.next() % 33) as i32 - 16) as f32 * 25.0)
    }
    fn zone(&mut self) -> ZoneInstance {
        ZoneInstance {
            zone_id: (self.next() % 2) as u32,
            instance_id: self.next() % 2,
        }
    }
}

#[test]
fn randomized_ordered_messages_match_pre_grid_algorithm() {
    for radius in [0.0, f32::from_bits(1), 25.0, 100.0, f32::MAX] {
        for cap in [0, 1, 4, 64] {
            for seed in [1, 0x96_1234_5678] {
                let mut rng = Rng(seed);
                let mut grid = PlayerRegistry::new(InterestPolicy::new(radius, cap).unwrap());
                let mut old =
                    old::PlayerRegistry::new(old::InterestPolicy::new(radius, cap).unwrap());
                for step in 0..600 {
                    let id = SessionId(rng.next() % 40);
                    let zone = rng.zone();
                    let position = rng.position();
                    let heading = (rng.next() % 8) as f32 * 0.5;
                    match rng.next() % 5 {
                        0 | 1 => {
                            let presentation = PlayerPresentation {
                                character_guid: 2000 + id.0,
                                model: id.0 as i32,
                                head: "synthetic.adr".into(),
                                ..PlayerPresentation::default()
                            };
                            assert_eq!(
                                grid.join_with_presentation(
                                    id,
                                    1000 + id.0,
                                    2000 + id.0,
                                    zone,
                                    position,
                                    heading,
                                    presentation.clone()
                                ),
                                old.join_with_presentation(
                                    id,
                                    1000 + id.0,
                                    2000 + id.0,
                                    zone,
                                    position,
                                    heading,
                                    presentation
                                ),
                                "join seed={seed} step={step} radius={radius} cap={cap}"
                            );
                        }
                        2 => assert_eq!(grid.leave(id), old.leave(id), "leave step={step}"),
                        3 => assert_eq!(
                            grid.update_pose(id, zone, position, heading),
                            old.update_pose(id, zone, position, heading),
                            "pose seed={seed} step={step} radius={radius} cap={cap}"
                        ),
                        _ => {
                            // Stationary flags-only updates, no-ops and non-finite rejections.
                            let (zone, position, heading) = grid
                                .get(id)
                                .map(|p| (p.zone, p.position, p.heading))
                                .unwrap_or((zone, position, heading));
                            let state = rng.next() as u8;
                            let heading = if step % 17 == 0 { f32::NAN } else { heading };
                            assert_eq!(
                                grid.update_pose_with_state(
                                    id, zone, position, heading, state, 255
                                ),
                                old.update_pose_with_state(id, zone, position, heading, state, 255),
                                "state step={step}"
                            );
                        }
                    }
                    for n in 0..40 {
                        assert_eq!(
                            grid.get(SessionId(n)),
                            old.get(SessionId(n)),
                            "session step={step}"
                        );
                    }
                    // A cloned registry must keep a correct, independent index too.
                    if step % 100 == 0 {
                        assert_eq!(grid.clone().leave(id), old.clone().leave(id));
                    }
                }
            }
        }
    }
}

#[test]
fn neighbourhood_canary_and_extreme_finite_coordinates_match() {
    let zone = ZoneInstance {
        zone_id: 7,
        instance_id: 9,
    };
    for radius in [0.0, f32::from_bits(1), 100.0, f32::MAX] {
        let mut grid = PlayerRegistry::new(InterestPolicy::new(radius, 2).unwrap());
        let mut old = old::PlayerRegistry::new(old::InterestPolicy::new(radius, 2).unwrap());
        // Negative-zero-adjacent values exercise f64 distance rounding at the inclusive radius.
        for (i, position) in [
            [-f32::from_bits(1), 0.0, 0.0],
            [radius, 0.0, 0.0],
            [0.0, radius, 0.0],
            [0.0, 0.0, radius],
            [-radius, 0.0, 0.0],
            [f32::MAX; 3],
            [-f32::MAX; 3],
            [f32::MAX; 3],
        ]
        .into_iter()
        .enumerate()
        {
            let id = SessionId(i as u64);
            assert_eq!(
                grid.join(id, 100 + id.0, 200 + id.0, zone, position, 0.0),
                old.join(id, 100 + id.0, 200 + id.0, zone, position, 0.0),
                "join radius={radius} subject={i}"
            );
        }
        for id in 0..8 {
            assert_eq!(
                grid.update_pose(SessionId(id), zone, [-radius, 0.0, radius], 1.0),
                old.update_pose(SessionId(id), zone, [-radius, 0.0, radius], 1.0)
            );
            assert_eq!(grid.leave(SessionId(id)), old.leave(SessionId(id)));
        }
    }
    // Across ordinary adjacent cells: deliberately restricting the grid to its own cell fails here.
    let mut grid = PlayerRegistry::new(InterestPolicy::new(100.0, 1).unwrap());
    grid.join(SessionId(1), 101, 201, zone, [99.0; 3], 0.0)
        .unwrap();
    assert_eq!(
        grid.join(SessionId(2), 102, 202, zone, [101.0; 3], 0.0)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn rejected_poses_and_identities_do_not_poison_the_grid() {
    let zone = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };
    let mut grid = PlayerRegistry::new(InterestPolicy::new(100.0, 2).unwrap());
    let mut old = old::PlayerRegistry::new(old::InterestPolicy::new(100.0, 2).unwrap());
    assert_eq!(
        grid.join(SessionId(1), 101, 201, zone, [0.0; 3], 0.0),
        old.join(SessionId(1), 101, 201, zone, [0.0; 3], 0.0)
    );
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for axis in 0..4 {
            let mut position = [0.0; 3];
            let heading = if axis == 3 {
                value
            } else {
                position[axis] = value;
                0.0
            };
            assert_eq!(
                grid.join(SessionId(2), 102, 202, zone, position, heading),
                Err(RegistryError::InvalidPose)
            );
            assert_eq!(
                grid.update_pose(SessionId(1), zone, position, heading),
                Err(RegistryError::InvalidPose)
            );
            assert_eq!(grid.get(SessionId(1)), old.get(SessionId(1)));
            assert!(grid.get(SessionId(2)).is_none());
        }
    }
    for (account, character) in [(0, 202), (102, 0), (101, 202), (102, 201)] {
        assert_eq!(
            grid.join(SessionId(2), account, character, zone, [0.0; 3], 0.0),
            old.join(SessionId(2), account, character, zone, [0.0; 3], 0.0)
        );
    }
    assert_eq!(
        grid.join(SessionId(2), 102, 202, zone, [1.0; 3], 0.0),
        old.join(SessionId(2), 102, 202, zone, [1.0; 3], 0.0)
    );
    assert_eq!(grid.leave(SessionId(1)), old.leave(SessionId(1)));
}

#[test]
fn dense_randomized_ordered_messages_match_reference() {
    let zone = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };
    for cap in [0, 1, 64, 128] {
        // Zero seed keeps every random coordinate/id at zero: a fully
        // coincident crowd, with the lowest-id subject moving in and out.
        for seed in [0, 106, 0x106_1234_5678] {
            let mut rng = Rng(seed);
            let mut grid = PlayerRegistry::new(InterestPolicy::new(100.0, cap).unwrap());
            let mut old = old::PlayerRegistry::new(old::InterestPolicy::new(100.0, cap).unwrap());
            // All 96 viewers share one cell and are in range, including ties.
            for id in 0..96 {
                let p = std::array::from_fn(|_| (rng.next() % 12) as f32);
                assert_eq!(
                    grid.join(SessionId(id), 1000 + id, 2000 + id, zone, p, 0.0),
                    old.join(SessionId(id), 1000 + id, 2000 + id, zone, p, 0.0)
                );
            }
            for step in 0..240 {
                let id = SessionId(rng.next() % 96);
                let mut p = std::array::from_fn(|_| (rng.next() % 12) as f32);
                let mut target_zone = zone;
                // Mix tiny moves, radius crossings, zone changes, flags, and rejoin.
                match step % 8 {
                    0 => {
                        assert_eq!(grid.leave(id), old.leave(id));
                        assert_eq!(
                            grid.join(id, 1000 + id.0, 2000 + id.0, zone, p, 0.0),
                            old.join(id, 1000 + id.0, 2000 + id.0, zone, p, 0.0)
                        );
                    }
                    1 => p = grid.get(id).unwrap().position.map(|x| x + 0.25),
                    2 => p = [100.0, 0.0, 0.0],
                    3 => p = [100.000_01, 0.0, 0.0],
                    4 => target_zone.instance_id = 1,
                    5 => p = grid.get(id).unwrap().position,
                    6 => p = [0.0; 3],
                    _ => {}
                }
                assert_eq!(
                    grid.update_pose_with_state(id, target_zone, p, 0.5, step as u8, 255),
                    old.update_pose_with_state(id, target_zone, p, 0.5, step as u8, 255),
                    "dense seed={seed} step={step} cap={cap}"
                );
                assert_eq!(grid.get(id), old.get(id));
                if step % 40 == 0 {
                    assert_eq!(grid.clone().leave(id), old.clone().leave(id));
                }
            }
        }
    }
}

#[test]
fn cutoff_crossing_and_tie_canary_matches_reference() {
    let zone = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };
    let mut grid = PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap());
    let mut old = old::PlayerRegistry::new(old::InterestPolicy::new(100.0, 64).unwrap());
    for id in 0..80 {
        let p = [id as f32, 0.0, 0.0];
        assert_eq!(
            grid.join(SessionId(id), 1000 + id, 2000 + id, zone, p, 0.0),
            old.join(SessionId(id), 1000 + id, 2000 + id, zone, p, 0.0)
        );
    }
    // Viewer 0 starts with ids 1..=64. Exercise invisible entry, cutoff
    // retreat/advance, equal-distance session-id ties, and radius exit/entry.
    for (id, x) in [
        (79, 0.5),
        (79, 90.0),
        (64, 0.25),
        (65, 63.5),
        (64, 64.0),
        (65, 64.0),
        (63, 64.0),
        (1, 100.0),
        (1, 100.000_01),
        (1, 1.0),
    ] {
        assert_eq!(
            grid.update_pose(SessionId(id), zone, [x, 0.0, 0.0], 0.0),
            old.update_pose(SessionId(id), zone, [x, 0.0, 0.0], 0.0),
            "cutoff id={id} x={x}"
        );
    }
}
