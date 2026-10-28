//! T490: pet navigation. `walkable` samples the heightfield along a path (a rise over 1 m, a
//! drop over 3 m or unknown ground refuses it), the follow step refuses a cliff and warps after
//! `warp_far_ms` without progress, and the routed 53/26 tile click answers the client's can-go
//! flag and walks the pet to the spot. `Gameplay::handle_in_world` and `Gameplay::pet_tick` are
//! driven for the full path in the last test; the rest use the same production service entry
//! points (`PetService::handle_at_ms` and `PetService::follow_owner`).
#[path = "support/pet_care.rs"]
mod care;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "support/pet_wire.rs"]
mod pet_wire;
#[path = "support/terrain.rs"]
mod terrain;

use care::{OWNER, dog, pets_json, send};
use npc_quest_chain::{CHARACTER, character_at_for};
use osfr_tools::{
    gameplay::Gameplay,
    pet_follow::{FollowConfig, MoveKind, OwnerPose, go_to, spawn, step, walkable},
    pets::{PetBusy, PetContent, PetRequest, PetService},
    terrain::Terrain,
};
use pet_inventory::InventoryFixture;
use pet_wire::{can_go, can_go_reply, read_can_go_reply, summon};
use std::{fs, sync::Arc};

/// The shipped `follow` block.
fn follow() -> FollowConfig {
    serde_json::from_value(pets_json()["follow"].clone()).expect("the shipped follow block")
}

/// The speed the pet's actor spawns with (`pets.json` `movement_speed`).
fn pace() -> f32 {
    pets_json()["movement_speed"]["speed"]
        .as_f64()
        .expect("a pet movement speed") as f32
}

fn grid_ground(
    grid: &osfr_tools::terrain::HeightGrid,
) -> impl Fn(f32, f32) -> Option<f32> + Copy + '_ {
    |x, z| grid.height_at(x, z)
}

/// A flat 40 x 40 m field with a 2 m ledge at x = 0: the walk from -5 to 5 rises 2 m between
/// two 0.5 m samples.
fn two_metre_step() -> osfr_tools::terrain::HeightGrid {
    terrain::surface([-20.0, -20.0], [20.0, 20.0], 0.5, |x, _| {
        if x < 0.0 { 0.0 } else { 2.0 }
    })
}

/// A flat 40 x 40 m field with a 10 m cliff at x = 0.
fn ten_metre_cliff() -> osfr_tools::terrain::HeightGrid {
    terrain::surface([-20.0, -20.0], [20.0, 20.0], 0.5, |x, _| {
        if x < 0.0 { 0.0 } else { 10.0 }
    })
}

/// A 4 m drop at x = 0: under the follow climb limit (6 m), over the step-down limit (3 m), so
/// only the walk check can refuse it.
fn four_metre_cliff() -> osfr_tools::terrain::HeightGrid {
    terrain::surface([-20.0, -20.0], [20.0, 20.0], 0.5, |x, _| {
        if x < 0.0 { 0.0 } else { 4.0 }
    })
}

#[test]
fn a_two_metre_rise_is_refused() {
    let grid = two_metre_step();
    let ground = grid_ground(&grid);
    assert!(
        !walkable(ground, [-5.0, 0.0, 0.0], [5.0, 0.0, 0.0], 1.0, 3.0, 0.5),
        "a 2 m rise is over the 1 m step-up limit"
    );
    assert!(
        walkable(ground, [-5.0, 0.0, 0.0], [5.0, 0.0, 0.0], 2.5, 3.0, 0.5),
        "the same ledge is fine with a 2.5 m step-up limit"
    );
}

#[test]
fn a_ten_metre_cliff_is_refused_going_down() {
    let grid = ten_metre_cliff();
    let ground = grid_ground(&grid);
    assert!(
        !walkable(ground, [5.0, 10.0, 0.0], [-5.0, 0.0, 0.0], 1.0, 3.0, 0.5),
        "a 10 m drop is over the 3 m step-down limit"
    );
    assert!(
        walkable(ground, [-5.0, 0.0, 0.0], [5.0, 10.0, 0.0], 10.0, 20.0, 0.5),
        "the same cliff passes with wider limits, so the height is what refused it"
    );
}

#[test]
fn a_gentle_slope_is_allowed() {
    let grid = terrain::surface([-20.0, -20.0], [20.0, 20.0], 0.5, |x, _| 0.2 * x);
    let ground = grid_ground(&grid);
    assert!(walkable(
        ground,
        [-5.0, 0.0, 0.0],
        [5.0, 2.0, 0.0],
        1.0,
        3.0,
        0.5
    ));
}

/// T490/F850: a step shorter than `sample_m` was folded into equal parts, so its only inner
/// sample sat at half the step and missed the cliff; at a 20 Hz tick a 12 m/s pet steps 0.6 m.
#[test]
fn a_short_step_still_samples_the_whole_path() {
    let grid = four_metre_cliff();
    let ground = grid_ground(&grid);
    assert!(
        !walkable(ground, [0.0, 4.0, 0.0], [-0.6, 0.0, 0.0], 1.0, 3.0, 0.5),
        "the 0.5 m sample lands on the low side of the 4 m cliff"
    );
    assert!(
        walkable(ground, [0.0, 4.0, 0.0], [-0.2, 3.2, 0.0], 1.0, 3.0, 0.5),
        "a 0.2 m step down the interpolated cell is under the step-down limit"
    );
}

#[test]
fn unknown_ground_is_not_walkable() {
    let hole = |x: f32, _: f32| (x < 2.0).then_some(0.0);
    assert!(!walkable(
        hole,
        [-5.0, 0.0, 0.0],
        [5.0, 0.0, 0.0],
        1.0,
        3.0,
        0.5
    ));
    assert!(!walkable(
        |_, _| None,
        [-5.0, 0.0, 0.0],
        [5.0, 0.0, 0.0],
        1.0,
        3.0,
        0.5
    ));
}

/// F858 (review F850): the point the next step lands on is checked, not only the lookahead, so
/// a hole narrower than `sample_m` at that point is refused even when the lookahead is on ground.
#[test]
fn a_step_endpoint_over_a_hole_is_refused() {
    let cfg = follow();
    let owner = [0.0, 0.0, 0.0];
    let pet = go_to(spawn(&cfg, owner), [12.0, 0.0, 0.0], owner);
    // Ground everywhere except the endpoint of the pet's first 50 ms step at `pace` (5 m/s).
    let hole = |x: f32, z: f32| {
        if (x - 2.25).abs() < 1e-3 && z.abs() < 1e-3 {
            None
        } else {
            Some(0.0)
        }
    };
    let (still, _) = step(
        &cfg,
        pace(),
        OwnerPose {
            position: owner,
            at_ms: 100,
        },
        pet,
        hole,
        true,
        false,
    );
    assert_eq!(still.position, pet.position, "the first tick has no time");
    let (next, frame) = step(
        &cfg,
        pace(),
        OwnerPose {
            position: owner,
            at_ms: 150,
        },
        still,
        hole,
        true,
        false,
    );
    assert_eq!(
        next.position, pet.position,
        "a hole at the step endpoint is not walkable"
    );
    assert_eq!(frame, None, "a refused step sends nothing");
}

#[test]
fn a_following_pet_refuses_a_cliff_and_warps_after_the_timeout() {
    let cfg = follow();
    let grid = four_metre_cliff();
    let ground = grid_ground(&grid);
    let mut owner = [2.0, 4.0, 0.0];
    let mut pet = spawn(&cfg, owner);
    let mut frames: Vec<(u64, MoveKind, [f32; 3], [f32; 3])> = Vec::new();
    let mut now = 0;
    for index in 0..40 {
        now += 500;
        // The owner leaves the high side for 10 m past the cliff, then holds there.
        if index < 24 {
            owner[0] -= 0.5;
            owner[1] = ground(owner[0], owner[2]).expect("the owner stays over the grid");
        }
        let (next, frame) = step(
            &cfg,
            pace(),
            OwnerPose {
                position: owner,
                at_ms: now,
            },
            pet,
            ground,
            true,
            false,
        );
        pet = next;
        if let Some(frame) = frame {
            frames.push((now, frame.kind, frame.position, owner));
        }
    }
    let warp = frames
        .iter()
        .position(|(_, kind, _, _)| *kind == MoveKind::Warp)
        .expect("a blocked pet warps after warp_far_ms");
    let last_walk = frames[..warp]
        .iter()
        .rposition(|(_, kind, _, _)| *kind == MoveKind::Walk)
        .expect("the pet walked down to the cliff edge first");
    assert!(
        frames[warp].0.saturating_sub(frames[last_walk].0) >= cfg.warp_far_ms,
        "the warp waits for warp_far_ms of no progress"
    );
    for (at, kind, position, _) in &frames[..warp] {
        assert_eq!(
            *kind,
            MoveKind::Walk,
            "{at} ms: only walk frames before the warp"
        );
        assert!(
            position[1] > 3.9,
            "{at} ms: the pet stayed on the high side, {position:?}"
        );
        assert!(
            position[0] >= -0.001,
            "{at} ms: {position:?} crossed the cliff"
        );
    }
    let (_, _, landed, owner_at) = frames[warp];
    assert!(
        landed[1] < 0.1,
        "the warp lands on the low side: {landed:?}"
    );
    assert!(
        (landed[0] - owner_at[0]).abs() <= cfg.side_m + 0.2,
        "the warp lands beside the owner at {owner_at:?}: {landed:?}"
    );
}

#[test]
fn can_go_requests_decode_with_the_exact_layout() {
    let body = can_go([10.0, 1.0, -2.0, 0.0]);
    assert_eq!(body.len(), 20, "3 header bytes, four floats and a flag");
    assert_eq!(
        PetRequest::decode(&body).unwrap(),
        Some(PetRequest::CanGo {
            position: [10.0, 1.0, -2.0, 0.0]
        })
    );
    let mut cases = vec![
        body[..body.len() - 1].to_vec(),
        [body.clone(), vec![0]].concat(),
        vec![53, 0, 26],
    ];
    for (offset, value) in [
        (3, f32::NAN),
        (7, f32::INFINITY),
        (11, f32::NEG_INFINITY),
        (15, 1.0e7),
    ] {
        let mut bad = body.clone();
        bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        cases.push(bad);
    }
    let mut unknown_flag = body.clone();
    unknown_flag[19] = 1;
    cases.push(unknown_flag);
    for bad in cases {
        assert!(PetRequest::decode(&bad).is_err(), "{bad:02x?}");
    }
    assert_eq!(
        PetRequest::decode(&[53, 0, 27]).unwrap(),
        None,
        "27 is still not decoded"
    );
}

/// A summoned dog at the care suite's owner position.
fn summoned() -> (PetService, InventoryFixture) {
    let (mut service, pet) = dog();
    let mut fixture = InventoryFixture::new();
    send(&mut service, &mut fixture, &summon(pet), 1_000);
    (service, fixture)
}

#[test]
fn a_reachable_click_walks_the_pet_there_and_it_waits() {
    let cfg = follow();
    let flat = |_: f32, _: f32| Some(0.0);
    let (mut service, mut fixture) = summoned();
    let saved = service.document_value();
    let accepted = service
        .handle_at_ms(
            // The click's y is the client's, and the server never uses it: the pet's height
            // comes from `flat` (0.0).
            &can_go([10.0, 5.0, 0.0, 0.0]),
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            flat,
            2_000,
        )
        .unwrap()
        .unwrap();
    let actor = PetService::pet_guid(OWNER);
    assert_eq!(
        accepted.private,
        vec![can_go_reply([10.0, 5.0, 0.0, 0.0], true)],
        "the reply echoes the click vector the client reader consumes"
    );
    assert_eq!(
        read_can_go_reply(&accepted.private[0]),
        Some(([10.0, 5.0, 0.0, 0.0], true)),
        "the client reader 00b578d0 reads four dwords and then the flag"
    );
    assert_eq!(
        accepted.private[0].len(),
        20,
        "3 header bytes, the four-dword payload and the flag"
    );
    let short = [vec![53, 0, 26], actor.to_le_bytes().to_vec(), vec![1]].concat();
    assert_eq!(
        read_can_go_reply(&short),
        None,
        "a body short of the reader's 16 payload bytes cannot set the client's can-go flag"
    );
    assert_eq!(
        service.document_value(),
        saved,
        "a click is session-only, never a saved change"
    );
    assert!(accepted.shared.is_empty(), "the reply is owner-only");

    let mut arrived_at = None;
    let mut resumed = false;
    let mut walks = 0;
    let mut t = 2_500;
    while t <= 30_000 {
        let wire = service.follow_owner(OWNER, [0.0; 3], flat, false, t);
        if wire
            .private
            .iter()
            .any(|packet| packet.starts_with(&[125, 0]))
        {
            walks += 1;
            assert_eq!(
                wire.private, wire.shared,
                "the owner and the viewers get the same frames"
            );
        }
        let position = service.view(OWNER).expect("the pet is out").position;
        assert!(
            position[1].abs() <= 0.01,
            "{t} ms: the client's click y never becomes the pet's height: {position:?}"
        );
        if arrived_at.is_none() && (position[0] - 10.0).abs() <= 0.2 {
            arrived_at = Some(t);
        } else if let Some(at) = arrived_at {
            if t < at + cfg.go_timeout_ms {
                assert!(
                    (position[0] - 10.0).abs() <= 0.2,
                    "{t} ms: the pet waits at the clicked spot, {position:?}"
                );
            } else if t >= at + cfg.go_timeout_ms + 500 {
                assert!(
                    position[0] < 9.8,
                    "{t} ms: following resumes after go_timeout_ms, {position:?}"
                );
                resumed = true;
                break;
            }
        }
        t += 500;
    }
    assert!(walks > 4, "the pet walks with 125 frames: {walks}");
    assert!(arrived_at.is_some(), "the pet reaches the clicked spot");
    assert!(resumed, "following resumes after the go timeout");
}

#[test]
fn an_unreachable_click_is_refused_and_following_continues() {
    // Flat until x = 5, then an 8 m rise: the pet at x = 2 cannot reach a click at x = 10.
    let grid = terrain::surface([-20.0, -20.0], [20.0, 20.0], 0.5, |x, _| {
        if x > 5.0 { 8.0 } else { 0.0 }
    });
    let ground = grid_ground(&grid);
    let (mut service, mut fixture) = summoned();
    let before = service.active_follow_state();
    let refused = service
        .handle_at_ms(
            // 10 m from the owner, but 8 m of it is over the step-up limit.
            &can_go([10.0, 2.0, 0.0, 0.0]),
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            ground,
            2_000,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        refused.private,
        vec![can_go_reply([10.0, 2.0, 0.0, 0.0], false)]
    );
    assert_eq!(
        service.active_follow_state(),
        before,
        "a refusal stores no destination"
    );

    service.follow_owner(OWNER, [0.0; 3], ground, true, 3_000);
    service.follow_owner(OWNER, [0.0, 0.0, 5.0], ground, true, 3_500);
    let position = service.view(OWNER).expect("the pet is out").position;
    assert!(
        position[2] > 2.0,
        "the pet keeps following after the refusal: {position:?}"
    );
}

#[test]
fn a_click_outside_the_client_limits_is_refused() {
    let flat = |_: f32, _: f32| Some(0.0);
    for (target, why) in [
        ([5.0, 0.0, 0.0, 0.0], "inside the client's 8 m minimum"),
        ([50.0, 0.0, 0.0, 0.0], "past the client's 40 m maximum"),
        ([-40.0, 0.0, 0.0, 0.0], "one click path past the pet"),
        (
            [30.0, 0.0, 0.0, 30.0],
            "the client's 4D distance is over 40 m even though x is inside it",
        ),
    ] {
        let (mut service, mut fixture) = summoned();
        let before = (service.active_follow_state(), service.view(OWNER));
        let refused = service
            .handle_at_ms(
                &can_go(target),
                OWNER,
                [0.0; 3],
                &mut fixture.inventory,
                PetBusy::default(),
                flat,
                2_000,
            )
            .unwrap()
            .unwrap();
        assert_eq!(refused.private, vec![can_go_reply(target, false)], "{why}");
        assert_eq!(refused.shared, Vec::<Vec<u8>>::new(), "{why}: no broadcast");
        assert_eq!(
            (service.active_follow_state(), service.view(OWNER)),
            before,
            "{why}: a refused click changes nothing"
        );
    }
}

/// A body that cannot be this client's 53/26 is an `Err` like the other arms, and the session is
/// untouched: the decode runs before any clock, pose or destination is read or written.
#[test]
fn a_malformed_click_returns_err_and_changes_nothing() {
    let flat = |_: f32, _: f32| Some(0.0);
    let (mut service, mut fixture) = summoned();
    let before = (service.active_follow_state(), service.view(OWNER));
    let mut bad_flag = can_go([10.0, 0.0, 0.0, 0.0]);
    *bad_flag.last_mut().unwrap() = 1;
    let mut not_finite = can_go([10.0, 0.0, 0.0, 0.0]);
    not_finite[7..11].copy_from_slice(&f32::NAN.to_le_bytes());
    for body in [
        vec![53, 0, 26],
        [can_go([10.0, 0.0, 0.0, 0.0]), vec![0]].concat(),
        bad_flag,
        not_finite,
    ] {
        assert!(
            service
                .handle_at_ms(
                    &body,
                    OWNER,
                    [0.0; 3],
                    &mut fixture.inventory,
                    PetBusy::default(),
                    flat,
                    2_000,
                )
                .is_err(),
            "{body:02x?}"
        );
        assert_eq!(
            (service.active_follow_state(), service.view(OWNER)),
            before,
            "{body:02x?}"
        );
    }
}

#[test]
fn a_second_click_inside_the_interval_is_ignored() {
    let flat = |_: f32, _: f32| Some(0.0);
    let (mut service, mut fixture) = summoned();
    let click = |service: &mut PetService, fixture: &mut InventoryFixture, target, now| {
        service
            .handle_at_ms(
                &can_go(target),
                OWNER,
                [0.0; 3],
                &mut fixture.inventory,
                PetBusy::default(),
                flat,
                now,
            )
            .unwrap()
            .unwrap()
    };
    let first = click(&mut service, &mut fixture, [10.0, 0.0, 0.0, 0.0], 2_000);
    assert_eq!(
        first.private,
        vec![can_go_reply([10.0, 0.0, 0.0, 0.0], true)]
    );
    let stored = service.active_follow_state();
    let early = click(&mut service, &mut fixture, [20.0, 0.0, 0.0, 0.0], 2_100);
    assert_eq!(
        early.private,
        vec![can_go_reply([20.0, 0.0, 0.0, 0.0], false)],
        "100 ms is inside the 200 ms interval"
    );
    assert_eq!(
        service.active_follow_state(),
        stored,
        "the ignored click changes nothing"
    );
    let later = click(&mut service, &mut fixture, [20.0, 0.0, 0.0, 0.0], 2_200);
    assert_eq!(
        later.private,
        vec![can_go_reply([20.0, 0.0, 0.0, 0.0], true)]
    );
}

/// T490/F850: the client's gate A requires 10 m between two sent clicks (`00b5fcf0`), so a
/// fine-grained click sequence the client would never send is refused; a refusal leaves the
/// stored click alone, and a click 10 m on is accepted.
#[test]
fn a_click_inside_the_client_step_band_is_refused() {
    let flat = |_: f32, _: f32| Some(0.0);
    let (mut service, mut fixture) = summoned();
    let click = |service: &mut PetService, fixture: &mut InventoryFixture, target, now| {
        service
            .handle_at_ms(
                &can_go(target),
                OWNER,
                [0.0; 3],
                &mut fixture.inventory,
                PetBusy::default(),
                flat,
                now,
            )
            .unwrap()
            .unwrap()
    };
    let first = click(&mut service, &mut fixture, [10.0, 0.0, 0.0, 0.0], 2_000);
    assert_eq!(
        first.private,
        vec![can_go_reply([10.0, 0.0, 0.0, 0.0], true)]
    );
    let stored = (service.active_follow_state(), service.view(OWNER));
    // 15 m from the owner (inside the 8-40 m band) but only 5 m from the accepted click.
    let near = click(&mut service, &mut fixture, [15.0, 0.0, 0.0, 0.0], 2_300);
    assert_eq!(
        near.private,
        vec![can_go_reply([15.0, 0.0, 0.0, 0.0], false)],
        "the client would not have sent a click 5 m from the last one"
    );
    assert_eq!(
        (service.active_follow_state(), service.view(OWNER)),
        stored,
        "the refused step changes no destination"
    );
    // 10 m from the accepted click: the client's own minimum displacement.
    let far = click(&mut service, &mut fixture, [20.0, 0.0, 0.0, 0.0], 2_400);
    assert_eq!(far.private, vec![can_go_reply([20.0, 0.0, 0.0, 0.0], true)]);
}

/// A zone without a height grid has nothing to walk on: the click answers 0 (R) and changes
/// nothing, and the pet keeps the T455 owner-height follow rule.
#[test]
fn a_click_without_a_terrain_grid_answers_zero() {
    let (mut service, mut fixture) = summoned();
    let before = service.active_follow_state();
    let refused = service
        .handle_at_ms(
            &can_go([10.0, 0.0, 0.0, 0.0]),
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            |_, _| None,
            2_000,
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        refused.private,
        vec![can_go_reply([10.0, 0.0, 0.0, 0.0], false)]
    );
    assert_eq!(
        service.active_follow_state(),
        before,
        "a refused click stores no destination"
    );
}

#[test]
fn a_click_without_a_pet_sends_nothing() {
    let mut service = PetService::new(PetService::bundled_content().unwrap()).unwrap();
    let mut fixture = InventoryFixture::new();
    let wire = service
        .handle_at_ms(
            &can_go([10.0, 0.0, 0.0, 0.0]),
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            |_, _| Some(0.0),
            1_000,
        )
        .unwrap()
        .unwrap();
    assert!(wire.private.is_empty() && wire.shared.is_empty());
}

/// A zone-7 world whose terrain is `grid`, with the care suite's dog summoned beside an owner at
/// `at`. The fixture is returned so its content folder outlives the world that loaded from it.
fn world_with(
    grid: osfr_tools::terrain::HeightGrid,
    at: [f32; 3],
) -> (
    npc_quest_chain::Fixture,
    Gameplay,
    osfr_tools::character_store::CharacterState,
) {
    let fixture = npc_quest_chain::Fixture::new();
    let terrain_dir = fixture.path().join("terrain");
    fs::create_dir_all(&terrain_dir).unwrap();
    fs::write(terrain_dir.join("zone_7.hgt"), grid.to_bytes()).unwrap();
    let mut game = Gameplay::load(fixture.path(), None).unwrap();
    game.set_terrain(Some(Arc::new(Terrain::load(&terrain_dir).unwrap())));
    let content = PetContent::bundled().unwrap();
    let mut pets = PetService::new(content.clone()).unwrap();
    let definition = content.definition(1_000_015).unwrap();
    let pet = pets.adopt_vendor_grant(1, &definition, 0).unwrap();
    game.adopt_pets(pets.document_value()).unwrap();
    let who = character_at_for(CHARACTER, 7, at);
    game.initialize_world(&who, 0).unwrap();
    let summoned = game.handle_in_world(&summon(pet), &who, 1_000).unwrap();
    assert!(
        summoned
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[53, 0, 9])),
        "the summon activates the pet"
    );
    (fixture, game, who)
}

#[test]
fn a_clicked_spot_is_walked_to_through_the_world_tick() {
    let grid = terrain::surface([-50.0, -50.0], [50.0, 50.0], 1.0, |_, _| 0.0);
    let (_fixture, mut game, who) = world_with(grid, [0.0, 0.0, 0.0]);

    let reply = game
        .handle_in_world(&can_go([10.0, 0.0, 0.0, 0.0]), &who, 2_000)
        .unwrap();
    assert!(
        reply
            .replies
            .contains(&can_go_reply([10.0, 0.0, 0.0, 0.0], true)),
        "the world route answers can-go: {:?}",
        reply.replies
    );

    let mut position = [0.0; 3];
    let mut arrived = false;
    for t in (2_500..=14_000).step_by(500) {
        game.pet_tick(&who, t);
        position = game.pet_view(who.guid).expect("the pet is out").position;
        if (position[0] - 10.0).abs() <= 0.2 {
            arrived = true;
            break;
        }
    }
    assert!(arrived, "the pet reaches the clicked spot: {position:?}");

    // The owner walks 6 m away (over resume_m): the go ends and following resumes.
    let moved = character_at_for(CHARACTER, 7, [0.0, 0.0, 6.0]);
    let resumed = game.pet_tick(&moved, 15_000).expect("the tick answers");
    assert!(
        resumed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[125, 0])),
        "a 125 follows the moved owner: {:?}",
        resumed.replies
    );
}

/// T490/F850: the production tick is 20 Hz, so a follow step is a fraction of `sample_m` (0.25 m
/// at the shipped 5 m/s pace) and the walk check must look a `sample_m` ahead. Before the fix the
/// heightfield's one-cell interpolation let the pet creep down a 4 m cliff 0.8 m per tick.
#[test]
fn a_fifty_millisecond_follow_step_does_not_walk_down_a_cliff() {
    let grid = terrain::surface([-20.0, -20.0], [20.0, 20.0], 0.5, |x, _| {
        if x < 0.0 { 0.0 } else { 4.0 }
    });
    let (_fixture, mut game, mut who) = world_with(grid, [2.0, 4.0, 0.0]);
    let mut previous = game.pet_view(who.guid).expect("the pet is out").position;
    let mut last_move = 1_000;
    let mut landing = None;
    for step in 1..=240u64 {
        let t = 1_000 + step * 50;
        if step < 200 {
            // The owner walks left at 2 m/s and drops to the low side at x = 0.
            let x = 2.0 - 0.1 * step as f32;
            who = character_at_for(CHARACTER, 7, [x, if x < 0.0 { 0.0 } else { 4.0 }, 0.0]);
        }
        game.pet_tick(&who, t);
        let position = game.pet_view(who.guid).expect("the pet is out").position;
        if position[1] <= 0.1 {
            landing = Some((t, position));
            break;
        }
        assert!(
            position[1] >= 3.9,
            "{t} ms: the pet descended the cliff part way: {position:?}"
        );
        assert!(
            position[0] >= -0.001,
            "{t} ms: the pet crossed the cliff edge: {position:?}"
        );
        if (position[0] - previous[0]).abs() >= 1e-4 || (position[2] - previous[2]).abs() >= 1e-4 {
            last_move = t;
        }
        previous = position;
    }
    let (at, landed) = landing.expect("a blocked pet warps after warp_far_ms");
    assert!(
        at.saturating_sub(last_move) >= follow().warp_far_ms,
        "the warp waits warp_far_ms after the last step: {last_move} -> {at}"
    );
    assert!(
        landed[0] < 0.0,
        "the warp lands on the low side: {landed:?}"
    );
}

/// F858 (review F850): a dip narrower than `sample_m` at the point the next 20 Hz step lands on.
/// With a 0.25 m grid the pet's first 0.25 m step lands on the dip while its 0.5 m lookahead is
/// back on flat ground, so checking only the lookahead walked the pet 4 m down.
#[test]
fn a_fifty_millisecond_step_does_not_walk_into_a_narrow_dip() {
    let grid = terrain::surface([-20.0, -20.0], [20.0, 20.0], 0.25, |x, z| {
        if (x - 2.25).abs() < 0.01 && z.abs() < 0.01 {
            -4.0
        } else {
            0.0
        }
    });
    let (_fixture, mut game, who) = world_with(grid, [0.0, 0.0, 0.0]);
    let spawn = game.pet_view(who.guid).expect("the pet is out").position;
    assert_eq!(spawn, [2.0, 0.0, 0.0], "the pet spawns beside its owner");
    let reply = game
        .handle_in_world(&can_go([12.0, 0.0, 0.0, 0.0]), &who, 2_000)
        .unwrap();
    assert!(
        reply
            .replies
            .contains(&can_go_reply([12.0, 0.0, 0.0, 0.0], true)),
        "the click's 0.5 m samples pass the dip: {:?}",
        reply.replies
    );
    for t in (2_050..7_000).step_by(50) {
        game.pet_tick(&who, t);
        let position = game.pet_view(who.guid).expect("the pet is out").position;
        assert!(
            position[1] > -0.1,
            "{t} ms: the 4 m dip at the step endpoint was walked into: {position:?}"
        );
        assert_eq!(
            position, spawn,
            "{t} ms: a refused step moves the pet nowhere"
        );
    }
    // After warp_far_ms of refused steps the pet goes to its owner instead.
    let warped = (7_000..8_000)
        .step_by(50)
        .find(|&t| {
            game.pet_tick(&who, t).is_some_and(|out| {
                out.replies
                    .iter()
                    .any(|packet| packet.starts_with(&[125, 0]))
            })
        })
        .expect("a blocked pet warps after warp_far_ms");
    assert!(
        warped >= 2_100 + follow().warp_far_ms,
        "the warp waits warp_far_ms after the first refused step: {warped} ms"
    );
    assert_eq!(
        game.pet_view(who.guid).expect("the pet is out").position,
        spawn,
        "{warped} ms: the warp lands beside the owner"
    );
}

/// T490/F850: a zone with a height grid has a hole where the grid has no ground, and that is not
/// walkable. Before the fix, a step whose two ends were both unknown kept T455's owner-height
/// rule, so a pet standing beside an owner outside the grid walked after them at their height.
#[test]
fn a_pet_over_a_hole_in_a_gridded_zone_does_not_walk() {
    // A grid that covers only the middle of the zone: the owner and pet stand outside it.
    let grid = terrain::surface([-2.0, -2.0], [2.0, 2.0], 1.0, |_, _| 0.0);
    let (_fixture, mut game, who) = world_with(grid, [20.0, 0.0, 20.0]);
    let spawn = game.pet_view(who.guid).expect("the pet is out").position;
    assert_eq!(spawn, [22.0, 0.0, 20.0], "the pet spawns beside its owner");
    for t in (1_050..5_000).step_by(50) {
        game.pet_tick(&who, t);
        assert_eq!(
            game.pet_view(who.guid).expect("the pet is out").position,
            spawn,
            "{t} ms: unknown ground under both ends of the step is not walkable"
        );
    }
    // After warp_far_ms of refused steps the pet goes to its owner instead.
    let warped = (5_000..8_000)
        .step_by(50)
        .find(|&t| {
            game.pet_tick(&who, t).is_some_and(|out| {
                out.replies
                    .iter()
                    .any(|packet| packet.starts_with(&[125, 0]))
            })
        })
        .expect("a blocked pet warps after warp_far_ms");
    assert_eq!(
        game.pet_view(who.guid).expect("the pet is out").position,
        spawn,
        "{warped} ms: the warp lands beside the owner"
    );
}
