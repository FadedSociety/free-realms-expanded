//! T455: the pure pet-follow step (`src/pet_follow.rs`) on the shipped `pets.json` follow block,
//! and the 125 frames `PetService::follow_owner` sends. The real gameplay path (owner moves,
//! viewers get the frames, a late viewer sees the pet where it stands) is in `pet_wired.rs`.
use osfr_tools::pet_follow::{FollowConfig, Move, MoveKind, OwnerPose, PetPose, spawn, step};
use osfr_tools::pets::{PetBusy, PetService};

fn config() -> FollowConfig {
    let content: serde_json::Value =
        serde_json::from_str(include_str!("../data/content/pets.json")).unwrap();
    serde_json::from_value(content["follow"].clone()).expect("shipped follow block")
}

fn flat(_: f32, _: f32) -> Option<f32> {
    Some(0.0)
}

fn slope(x: f32, z: f32) -> Option<f32> {
    Some(0.1 * x + 0.05 * z)
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Walk the owner along +z at `speed` m/s for `ms` in 100 ms steps; every frame is returned.
fn walk(
    cfg: &FollowConfig,
    mut pet: PetPose,
    start: [f32; 3],
    speed: f32,
    ms: u64,
    t0: u64,
    ground: fn(f32, f32) -> Option<f32>,
) -> (PetPose, [f32; 3], Vec<Move>) {
    let mut frames = Vec::new();
    let mut owner = start;
    for t in (0..=ms).step_by(100) {
        owner[2] = start[2] + speed * t as f32 / 1000.0;
        owner[1] = ground(owner[0], owner[2]).unwrap();
        let (next, frame) = step(
            cfg,
            OwnerPose {
                position: owner,
                at_ms: t0 + t,
            },
            pet,
            ground,
            false,
        );
        pet = next;
        frames.extend(frame);
    }
    (pet, owner, frames)
}

/// Hold the owner still, stepping every 100 ms for `ms`.
fn hold(
    cfg: &FollowConfig,
    mut pet: PetPose,
    owner: [f32; 3],
    t0: u64,
    ms: u64,
) -> (PetPose, Vec<Move>) {
    let mut frames = Vec::new();
    for t in (100..=ms).step_by(100) {
        let (next, frame) = step(
            cfg,
            OwnerPose {
                position: owner,
                at_ms: t0 + t,
            },
            pet,
            flat,
            false,
        );
        pet = next;
        frames.extend(frame);
    }
    (pet, frames)
}

#[test]
fn the_shipped_follow_block_is_valid() {
    assert!(config().valid());
}

#[test]
fn a_walking_owner_is_followed_on_flat_and_sloped_ground() {
    let cfg = config();
    for ground in [flat as fn(f32, f32) -> Option<f32>, slope] {
        let start = [0.0, 0.0, 0.0];
        let (pet, owner, frames) = walk(&cfg, spawn(&cfg, start), start, 5.0, 4_000, 0, ground);
        assert!(
            dist(pet.position, owner) <= 3.0,
            "pet {pet:?} owner {owner:?}"
        );
        assert!(pet.position[2] < owner[2], "the pet trails behind");
        for frame in &frames {
            let y = ground(frame.position[0], frame.position[2]).unwrap();
            assert!(
                (frame.position[1] - y).abs() <= 0.1,
                "{frame:?} vs ground {y}"
            );
        }
    }
}

#[test]
fn a_fast_rider_is_caught_up_with() {
    // The canary for catch-up: without it a 20 m/s rider leaves the 12 m/s pet ~26 m behind.
    let cfg = config();
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&cfg, spawn(&cfg, start), start, 20.0, 3_000, 0, flat);
    assert!(
        dist(pet.position, owner) <= 20.0,
        "{} m behind",
        dist(pet.position, owner)
    );
}

#[test]
fn a_still_owner_gets_one_stop_frame_and_a_settled_pet_beside_them() {
    let cfg = config();
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&cfg, spawn(&cfg, start), start, 5.0, 2_000, 0, flat);
    let (pet, frames) = hold(&cfg, pet, owner, 2_000, 4_000);
    let stops: Vec<_> = frames.iter().filter(|f| f.kind == MoveKind::Stop).collect();
    assert_eq!(stops.len(), 1, "{frames:?}");
    assert!((dist(pet.position, owner) - cfg.settle_radius_m).abs() <= 0.15);
    // Owner faced +z; the pet sits 30-70 degrees off that facing and faces the owner.
    let bearing = [pet.position[0] - owner[0], pet.position[2] - owner[2]];
    let angle = (bearing[1] / bearing[0].hypot(bearing[1]))
        .acos()
        .to_degrees();
    assert!(
        (cfg.settle_min_deg - 0.5..=cfg.settle_max_deg + 0.5).contains(&angle),
        "{angle}"
    );
    let to_owner = [-bearing[0], -bearing[1]];
    let facing = stops[0].heading;
    assert!(
        facing[0] * to_owner[0] + facing[2] * to_owner[1] > 0.0,
        "faces the owner"
    );
}

#[test]
fn a_teleport_warps_the_pet_beside_the_owner_once() {
    let cfg = config();
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&cfg, spawn(&cfg, start), start, 5.0, 1_000, 0, flat);
    let jumped = [owner[0] + 50.0, 0.0, owner[2]];
    let (pet, frame) = step(
        &cfg,
        OwnerPose {
            position: jumped,
            at_ms: 1_100,
        },
        pet,
        flat,
        false,
    );
    assert_eq!(frame.map(|f| f.kind), Some(MoveKind::Warp));
    assert!(dist(pet.position, jumped) <= cfg.side_m + 0.1);
    let (_, next) = step(
        &cfg,
        OwnerPose {
            position: jumped,
            at_ms: 1_200,
        },
        pet,
        flat,
        false,
    );
    assert_ne!(next.map(|f| f.kind), Some(MoveKind::Warp), "one warp only");
}

#[test]
fn walking_sends_at_most_two_frames_a_second() {
    let cfg = config();
    let start = [0.0, 0.0, 0.0];
    let (_, _, frames) = walk(&cfg, spawn(&cfg, start), start, 5.0, 10_000, 0, flat);
    assert!(frames.len() <= 21, "{} frames in 10 s", frames.len());
}

#[test]
fn an_open_pet_window_stops_the_pet_until_it_closes() {
    let cfg = config();
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&cfg, spawn(&cfg, start), start, 5.0, 1_000, 0, flat);
    let moved = [owner[0], 0.0, owner[2] + 0.5];
    let (paused, frame) = step(
        &cfg,
        OwnerPose {
            position: moved,
            at_ms: 1_100,
        },
        pet,
        flat,
        true,
    );
    assert_eq!(frame.map(|f| f.kind), Some(MoveKind::Stop));
    let (_, again) = step(
        &cfg,
        OwnerPose {
            position: moved,
            at_ms: 1_200,
        },
        paused,
        flat,
        true,
    );
    assert_eq!(again, None, "one stop frame while paused");
    let (resumed, _, frames) = walk(&cfg, paused, moved, 5.0, 2_000, 1_300, flat);
    assert!(frames.iter().any(|f| f.kind == MoveKind::Walk));
    assert!(dist(resumed.position, [moved[0], 0.0, moved[2] + 10.0]) <= 3.0);
}

#[test]
fn a_bad_ground_or_owner_value_skips_the_step() {
    let cfg = config();
    let pet = spawn(&cfg, [0.0, 0.0, 0.0]);
    let nan_ground = |_: f32, _: f32| Some(f32::NAN);
    let (kept, frame) = step(
        &cfg,
        OwnerPose {
            position: [0.0, 0.0, 9.0],
            at_ms: 100,
        },
        pet,
        nan_ground,
        false,
    );
    assert_eq!((kept, frame), (pet, None));
    let (kept, frame) = step(
        &cfg,
        OwnerPose {
            position: [f32::NAN, 0.0, 0.0],
            at_ms: 100,
        },
        pet,
        flat,
        false,
    );
    assert_eq!((kept, frame), (pet, None));
}

#[test]
fn follow_frames_use_the_enemy_movement_layout() {
    let content = PetService::bundled_content().unwrap();
    let definition = content.definition(1_000_015).unwrap();
    let mut service = PetService::new(content).unwrap();
    let id = service.adopt_vendor_grant(1, &definition, 0).unwrap();
    service
        .summon(id, 44, 33, [0.0, 0.0, 0.0], PetBusy::default(), 0)
        .unwrap();
    let mut frames = Vec::new();
    for t in 1..=20u64 {
        let wire = service.follow_owner(33, [0.0, 0.0, t as f32 * 0.5], flat, t * 100);
        assert!(!wire.changed, "following never marks a save");
        assert_eq!(
            wire.private, wire.shared,
            "owner and viewers get the same frame"
        );
        frames.extend(wire.shared);
    }
    let frame = frames.first().expect("a walking owner moves the pet");
    // u16 125, u64 actor, 3 x f32 position, 3 x f32 heading, u8 state, u8 unknown.
    assert_eq!(frame.len(), 36);
    assert_eq!(frame[..2], 125u16.to_le_bytes());
    assert_eq!(frame[2..10], 44u64.to_le_bytes(), "the summoned actor");
    assert!(
        service
            .follow_owner(99, [0.0, 0.0, 30.0], flat, 5_000)
            .shared
            .is_empty(),
        "only its own owner"
    );
}

/// F833: a follow click is bounded around the owner (the client's own sender `00b5fcf0` reports
/// only targets 8-40 m away), so a forged coordinate cannot park the pet.
#[test]
fn a_follow_click_is_bounded_around_the_owner() {
    let cfg = config();
    assert_eq!(cfg.click_max_m, 40.0, "the client's own maximum");
    let owner = [10.0, 0.0, 10.0];
    let pet = spawn(&cfg, owner);
    assert_eq!(
        pet.owner_position(),
        pet.position,
        "before the first owner update the spawn stands in for the owner"
    );
    assert!(
        cfg.admits_click(owner, [owner[0] + cfg.click_max_m, owner[1], owner[2]]),
        "the boundary itself is admitted"
    );
    assert!(!cfg.admits_click(
        owner,
        [owner[0] + cfg.click_max_m + 0.5, owner[1], owner[2]]
    ));
    assert!(!cfg.admits_click(
        owner,
        [owner[0], owner[1] + cfg.click_max_m + 0.5, owner[2]]
    ));
    assert!(!cfg.admits_click(owner, [1e38, 1e38, 1e38]));
    let (stepped, _) = step(
        &cfg,
        OwnerPose {
            position: owner,
            at_ms: 100,
        },
        pet,
        flat,
        false,
    );
    assert_eq!(stepped.owner_position(), owner, "after an owner update");
}
