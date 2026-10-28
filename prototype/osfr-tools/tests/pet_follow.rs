//! T455: the pure pet-follow step (`src/pet_follow.rs`) on the shipped `pets.json` follow block,
//! and the 125 frames `PetService::follow_owner` sends. The real gameplay path (owner moves,
//! viewers get the frames, a late viewer sees the pet where it stands) is in `pet_wired.rs`.
use osfr_tools::pet_follow::{FollowConfig, Gait, Move, MoveKind, OwnerPose, PetPose, spawn, step};
use osfr_tools::pets::{PetBusy, PetContent, PetService};
use osfr_tools::world_events::piece_052::ANIM_ID_WALK;
use serde_json::json;

#[path = "support/json.rs"]
mod json;

/// The shipped follow block and the pet's walking speed (the `movement_speed` its actor spawns
/// with), read from `pets.json` once per test.
struct Shipped {
    cfg: FollowConfig,
    pace: f32,
}

fn shipped() -> Shipped {
    let content = json::pets_json();
    Shipped {
        cfg: serde_json::from_value(content["follow"].clone()).expect("shipped follow block"),
        pace: content["movement_speed"]["speed"].as_f64().unwrap() as f32,
    }
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
    sh: &Shipped,
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
            &sh.cfg,
            sh.pace,
            OwnerPose {
                position: owner,
                at_ms: t0 + t,
            },
            pet,
            ground,
            false,
            false,
        );
        pet = next;
        frames.extend(frame);
    }
    (pet, owner, frames)
}

/// Hold the owner still, stepping every `tick_ms` for `ms`.
fn hold(
    sh: &Shipped,
    mut pet: PetPose,
    owner: [f32; 3],
    t0: u64,
    ms: u64,
    tick_ms: usize,
) -> (PetPose, Vec<Move>) {
    let mut frames = Vec::new();
    for t in (tick_ms..=ms as usize).step_by(tick_ms) {
        let t = t as u64;
        let (next, frame) = step(
            &sh.cfg,
            sh.pace,
            OwnerPose {
                position: owner,
                at_ms: t0 + t,
            },
            pet,
            flat,
            false,
            false,
        );
        pet = next;
        frames.extend(frame);
    }
    (pet, frames)
}

/// The world tick and the client's report interval (`pet_follow::OWNER_UPDATE_MS`, client
/// `00934a20`): the pet is stepped every tick, the owner's position changes every update.
const TICK_MS: usize = 50;
const UPDATE_MS: u64 = 500;

/// A walking owner as the server really sees one: `speed` m/s along +z from the origin for `ms`,
/// the position changing only every `UPDATE_MS`, the pet stepped every `TICK_MS`. Returns the
/// pet, the owner's last position, every frame, and the worst owner-to-pet gap.
fn walk_at_client_cadence(
    sh: &Shipped,
    speed: f32,
    ms: u64,
) -> (PetPose, [f32; 3], Vec<Move>, f32) {
    let mut pet = spawn(&sh.cfg, [0.0, 0.0, 0.0]);
    let (mut owner, mut frames, mut worst) = ([0.0; 3], Vec::new(), 0.0_f32);
    for t in (0..=ms as usize).step_by(TICK_MS) {
        let t = t as u64;
        owner[2] = speed * (t / UPDATE_MS * UPDATE_MS) as f32 / 1000.0;
        let (next, frame) = step(
            &sh.cfg,
            sh.pace,
            OwnerPose {
                position: owner,
                at_ms: t,
            },
            pet,
            flat,
            false,
            false,
        );
        pet = next;
        worst = worst.max(dist(pet.position, owner));
        frames.extend(frame);
    }
    (pet, owner, frames, worst)
}

#[test]
fn the_shipped_follow_block_is_valid() {
    assert!(shipped().cfg.valid());
}

/// T827 removed `min_speed`/`max_speed` from the follow block. A content package cut before
/// T827 still loads whole: the two old keys are accepted and ignored.
#[test]
fn a_pre_t827_content_package_still_loads() {
    let mut document = json::pets_json();
    document["follow"]["min_speed"] = json!(2.0);
    document["follow"]["max_speed"] = json!(12.0);
    let bytes = serde_json::to_vec(&document).unwrap();
    PetContent::from_json(&bytes).expect("a package cut before T827 still loads");
}

#[test]
fn a_walking_owner_is_followed_on_flat_and_sloped_ground() {
    let sh = shipped();
    let cfg = &sh.cfg;
    for ground in [flat as fn(f32, f32) -> Option<f32>, slope] {
        let start = [0.0, 0.0, 0.0];
        let (pet, owner, frames) = walk(&sh, spawn(cfg, start), start, 5.0, 4_000, 0, ground);
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
    let sh = shipped();
    let cfg = &sh.cfg;
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&sh, spawn(cfg, start), start, 20.0, 3_000, 0, flat);
    assert!(
        dist(pet.position, owner) <= 20.0,
        "{} m behind",
        dist(pet.position, owner)
    );
}

#[test]
fn a_still_owner_gets_one_stop_frame_and_a_settled_pet_beside_them() {
    let sh = shipped();
    let cfg = &sh.cfg;
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&sh, spawn(cfg, start), start, 5.0, 2_000, 0, flat);
    let (pet, frames) = hold(&sh, pet, owner, 2_000, 4_000, 100);
    let stops: Vec<_> = frames.iter().filter(|f| f.kind == MoveKind::Stop).collect();
    // T827: the settle is the one Stop; reaching the trailing point between two owner reports
    // keeps the gait and sends no Stop.
    assert_eq!(stops.len(), 1, "{frames:?}");
    assert_eq!(frames.last().map(|f| f.kind), Some(MoveKind::Stop));
    assert!(pet.settled());
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
    let facing = stops.last().unwrap().heading;
    assert!(
        facing[0] * to_owner[0] + facing[2] * to_owner[1] > 0.0,
        "faces the owner"
    );
}

#[test]
fn a_teleport_warps_the_pet_beside_the_owner_once() {
    let sh = shipped();
    let cfg = &sh.cfg;
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&sh, spawn(cfg, start), start, 5.0, 1_000, 0, flat);
    let jumped = [owner[0] + 50.0, 0.0, owner[2]];
    let (pet, frame) = step(
        cfg,
        sh.pace,
        OwnerPose {
            position: jumped,
            at_ms: 1_100,
        },
        pet,
        flat,
        false,
        false,
    );
    assert_eq!(frame.map(|f| f.kind), Some(MoveKind::Warp));
    assert!(dist(pet.position, jumped) <= cfg.side_m + 0.1);
    let (_, next) = step(
        cfg,
        sh.pace,
        OwnerPose {
            position: jumped,
            at_ms: 1_200,
        },
        pet,
        flat,
        false,
        false,
    );
    assert_ne!(next.map(|f| f.kind), Some(MoveKind::Warp), "one warp only");
}

#[test]
fn walking_sends_at_most_two_frames_a_second() {
    // T455/T827: one frame per `send_ms` at the shipped 500 ms, driven at the real client
    // cadence (the owner reports every 500 ms, the world tick steps the pet every 50 ms). The
    // sign-off names 500 ms, so the ceiling is pinned instead of derived from the data it
    // protects; a faster cadence is a lead rate decision.
    let sh = shipped();
    assert_eq!(sh.cfg.send_ms, 500, "the T455 accepted cadence");
    let (_, _, frames, _) = walk_at_client_cadence(&sh, 5.0, 10_000);
    let due = 10_000 / 500 + 1; // plus the t = 0 frame
    assert!(
        frames.len() <= due,
        "{} frames in 10 s, due at most {due}",
        frames.len()
    );
    assert!(
        frames.len() >= 10_000 / (500 * 2),
        "a walking owner's pet moves on every send interval, not only after a stop: {} frames",
        frames.len()
    );
    assert!(
        frames.iter().all(|frame| frame.kind == MoveKind::Walk),
        "a steady walk has no stop frame"
    );
}

/// T827: a running owner at the client's real cadence. The pet is never left more than the
/// catch-up distance behind, never warps, and a frame is never a jump bigger than a send
/// interval's travel.
#[test]
fn a_running_owner_is_trailed_continuously_and_never_teleported() {
    let sh = shipped();
    let cfg = &sh.cfg;
    let (pet, _, frames, worst) = walk_at_client_cadence(&sh, 10.0, 20_000);
    let hop = sh.pace * cfg.max_speed_multiplier * (cfg.send_ms + TICK_MS as u64) as f32 / 1000.0;
    let mut last = spawn(cfg, [0.0, 0.0, 0.0]).position;
    for frame in &frames {
        assert_ne!(frame.kind, MoveKind::Warp, "no teleport: {frame:?}");
        assert!(
            dist(frame.position, last) <= hop,
            "{frame:?} jumped from {last:?}"
        );
        last = frame.position;
    }
    assert!(
        worst <= cfg.catch_up_start_m + 5.0,
        "{worst} m behind at worst"
    );
    assert!(pet.moving(), "still trailing at the end");
}

/// T827: the client reports a walking owner every 500 ms while the world steps the pet every
/// 50 ms, so the pet reaches its trailing point between two reports. It must not stand up and
/// walk again each time: no Stop frame and one gait while the owner walks, then a Stop and
/// Stand once the owner halts.
#[test]
fn a_walking_pet_does_not_stand_between_owner_updates() {
    let sh = shipped();
    let (pet, owner, frames, _) = walk_at_client_cadence(&sh, 4.0, 20_000);
    let odd: Vec<_> = frames.iter().filter(|f| f.kind != MoveKind::Walk).collect();
    assert!(
        odd.is_empty(),
        "{} stop or warp frames while the owner walks, first {:?}",
        odd.len(),
        odd.first()
    );
    let gaits: Vec<_> = frames.iter().filter_map(|f| f.gait).collect();
    assert_eq!(gaits, [Gait::Walk], "one gait change in 20 s");
    assert!(pet.moving());
    let (pet, rest) = hold(&sh, pet, owner, 20_000, 4_000, TICK_MS);
    let stops = rest.iter().filter(|f| f.kind == MoveKind::Stop).count();
    assert_eq!(stops, 1, "one stop once the owner halts: {rest:?}");
    assert_eq!(
        rest.last().map(|f| (f.kind, f.gait)),
        Some((MoveKind::Stop, Some(Gait::Stand)))
    );
    assert!(pet.settled() && !pet.moving());
}

/// Only an owner jump past `warp_owner_jump_m` (a warp or a zone change) teleports the pet.
#[test]
fn only_a_jump_beyond_the_leash_teleports() {
    let sh = shipped();
    let cfg = &sh.cfg;
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&sh, spawn(cfg, start), start, 5.0, 1_000, 0, flat);
    for (jump, warps) in [
        (cfg.warp_owner_jump_m - 5.0, false),
        (cfg.warp_owner_jump_m + 5.0, true),
    ] {
        let to = [owner[0], 0.0, owner[2] + jump];
        let (_, frame) = step(
            cfg,
            sh.pace,
            OwnerPose {
                position: to,
                at_ms: 1_100,
            },
            pet,
            flat,
            false,
            false,
        );
        let frame = frame.expect("the pet reacts to the jump");
        assert_eq!(frame.kind == MoveKind::Warp, warps, "{jump} m: {frame:?}");
    }
}

/// The far leash: a pet left beyond `warp_far_m` for `warp_far_ms` teleports then, and not before.
/// Every owner step stays below `warp_owner_jump_m`, so the jump rule cannot answer for the leash.
#[test]
fn the_far_leash_teleports_only_after_its_delay() {
    let sh = shipped();
    let cfg = &sh.cfg;
    let mut pet = spawn(cfg, [0.0, 0.0, 0.0]);
    let step_m = cfg.warp_owner_jump_m - 1.0;
    let mut owner = [0.0; 3];
    let (mut far_at, mut warps) = (None, Vec::new());
    for tick in 1..=16u64 {
        let at = tick * 500;
        if tick <= 12 {
            owner[2] += step_m;
        }
        // The step's own leash test sees this gap, before the pet moves.
        if far_at.is_none() && dist(pet.position, owner) > cfg.warp_far_m {
            far_at = Some(at);
        }
        let (next, frame) = step(
            cfg,
            sh.pace,
            OwnerPose {
                position: owner,
                at_ms: at,
            },
            pet,
            flat,
            false,
            false,
        );
        pet = next;
        if frame.is_some_and(|frame| frame.kind == MoveKind::Warp) {
            assert!(
                dist(pet.position, owner) <= cfg.side_m + 0.1,
                "a leash warp lands beside the owner"
            );
            warps.push(at);
        }
    }
    let due = far_at.expect("the pet leaves the leash") + cfg.warp_far_ms;
    assert_eq!(warps, [due], "the leash teleports once, when it is due");
}

/// T827: every change of gait is one base animation: walk on the first step, run while catching
/// up with a far owner, stand when the pet reaches its spot.
#[test]
fn the_pet_walks_runs_and_stands_with_its_movement() {
    let sh = shipped();
    let cfg = &sh.cfg;
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, mut frames) = walk(&sh, spawn(cfg, start), start, 20.0, 3_000, 0, flat);
    let (_, rest) = hold(&sh, pet, owner, 3_000, 8_000, 100);
    frames.extend(rest);
    let gaits: Vec<_> = frames.iter().filter_map(|f| f.gait).collect();
    assert_eq!(
        gaits,
        [Gait::Walk, Gait::Run, Gait::Walk, Gait::Stand],
        "one base animation per real gait change: walk, run while catching up, walk back to \
         the settle spot, stand"
    );
}

#[test]
fn an_open_pet_window_stops_the_pet_until_it_closes() {
    let sh = shipped();
    let cfg = &sh.cfg;
    let start = [0.0, 0.0, 0.0];
    let (pet, owner, _) = walk(&sh, spawn(cfg, start), start, 5.0, 1_000, 0, flat);
    let moved = [owner[0], 0.0, owner[2] + 0.5];
    let (paused, frame) = step(
        cfg,
        sh.pace,
        OwnerPose {
            position: moved,
            at_ms: 1_100,
        },
        pet,
        flat,
        false,
        true,
    );
    assert_eq!(frame.map(|f| f.kind), Some(MoveKind::Stop));
    let (_, again) = step(
        cfg,
        sh.pace,
        OwnerPose {
            position: moved,
            at_ms: 1_200,
        },
        paused,
        flat,
        false,
        true,
    );
    assert_eq!(again, None, "one stop frame while paused");
    let (resumed, _, frames) = walk(&sh, paused, moved, 5.0, 2_000, 1_300, flat);
    assert!(frames.iter().any(|f| f.kind == MoveKind::Walk));
    assert!(dist(resumed.position, [moved[0], 0.0, moved[2] + 10.0]) <= 3.0);
}

#[test]
fn a_bad_ground_or_owner_value_skips_the_step() {
    let sh = shipped();
    let cfg = &sh.cfg;
    let pet = spawn(cfg, [0.0, 0.0, 0.0]);
    let nan_ground = |_: f32, _: f32| Some(f32::NAN);
    let (kept, frame) = step(
        cfg,
        sh.pace,
        OwnerPose {
            position: [0.0, 0.0, 9.0],
            at_ms: 100,
        },
        pet,
        nan_ground,
        false,
        false,
    );
    assert_eq!((kept, frame), (pet, None));
    let (kept, frame) = step(
        cfg,
        sh.pace,
        OwnerPose {
            position: [f32::NAN, 0.0, 0.0],
            at_ms: 100,
        },
        pet,
        flat,
        false,
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
        let wire = service.follow_owner(33, [0.0, 0.0, t as f32 * 0.5], flat, false, t * 100);
        assert!(!wire.changed, "following never marks a save");
        assert_eq!(
            wire.private, wire.shared,
            "owner and viewers get the same frame"
        );
        frames.extend(wire.shared);
    }
    let frame = frames.first().expect("a walking owner moves the pet");
    // T827: the first move comes with the base walk animation (35/8, base flag) for owner and
    // viewers. Layout from the client reader `008e5dd0` (via `00908a50`): opcode 35, subtype 8,
    // u64 actor, i32 animation id, i32 0, u8 flags; the flags bit-0 branch at `00931649` assigns
    // the actor's base animation.
    let mut walk = vec![35_u8, 0, 8, 0];
    walk.extend(44_u64.to_le_bytes());
    walk.extend(ANIM_ID_WALK.to_le_bytes());
    walk.extend(0_i32.to_le_bytes());
    walk.push(1);
    assert_eq!(frames[1], walk);
    // u16 125, u64 actor, 3 x f32 position, 3 x f32 heading, u8 state, u8 unknown.
    assert_eq!(frame.len(), 36);
    assert_eq!(frame[..2], 125u16.to_le_bytes());
    assert_eq!(frame[2..10], 44u64.to_le_bytes(), "the summoned actor");
    assert!(
        service
            .follow_owner(99, [0.0, 0.0, 30.0], flat, false, 5_000)
            .shared
            .is_empty(),
        "only its own owner"
    );
}

/// F833/T490: a follow click is bounded around the owner (the client's own sender `00b5fcf0`
/// measures a 4D distance in [8, 40] m), so a forged coordinate cannot park the pet. T490/F850
/// also enforces the sender's gate A, the 10 m minimum displacement between clicks.
#[test]
fn a_follow_click_is_bounded_around_the_owner() {
    let sh = shipped();
    let cfg = &sh.cfg;
    assert_eq!(cfg.click_max_m, 40.0, "the client's own maximum");
    let owner = [10.0, 0.0, 10.0];
    let pet = spawn(cfg, owner);
    assert_eq!(
        pet.owner_position(),
        pet.position,
        "before the first owner update the spawn stands in for the owner"
    );
    assert!(
        cfg.admits_click(owner, [owner[0] + cfg.click_max_m, owner[1], owner[2], 0.0]),
        "the boundary itself is admitted"
    );
    assert!(!cfg.admits_click(
        owner,
        [owner[0] + cfg.click_max_m + 0.5, owner[1], owner[2], 0.0]
    ));
    assert!(!cfg.admits_click(
        owner,
        [owner[0], owner[1] + cfg.click_max_m + 0.5, owner[2], 0.0]
    ));
    assert!(
        !cfg.admits_click(owner, [owner[0] + 30.0, owner[1] + 30.0, owner[2], 0.0]),
        "the client's 4D distance is 42 m even though each axis is inside 40"
    );
    assert!(!cfg.admits_click(owner, [1e38, 1e38, 1e38, 0.0]));
    assert!(
        cfg.advances_click([10.0, 0.0, 0.0, 0.0], [20.0, 0.0, 0.0, 0.0]),
        "10 m is the client's own minimum displacement"
    );
    assert!(
        !cfg.advances_click([10.0, 0.0, 0.0, 0.0], [15.0, 0.0, 0.0, 5.0]),
        "a 5 m step and a 5 m fourth component is 7 m, under the 10 m minimum"
    );
    let (stepped, _) = step(
        cfg,
        sh.pace,
        OwnerPose {
            position: owner,
            at_ms: 100,
        },
        pet,
        flat,
        false,
        false,
    );
    assert_eq!(stepped.owner_position(), owner, "after an owner update");
}
