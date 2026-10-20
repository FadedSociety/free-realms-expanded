//! W158 slice 2b: the world remembers which kills already gave their group heal.
use osfr_tools::world::{InstanceKind, World, WorldPose};

/// Put `guid` in zone 0's public instance, or in `target` (a zone-0 child).
fn join(world: &mut World, guid: u64, target: Option<i32>) {
    let change = world
        .prepare_membership(guid, 0, target, WorldPose::ORIGIN, true, 0)
        .expect("zone 0 is registered");
    world.commit_membership(change);
}

#[test]
fn a_kill_gives_its_group_heal_once() {
    let mut world = World::runtime();
    join(&mut world, 101, None);
    assert_eq!(world.claim_group_heal(101, (7, 9)), Some(0));
    assert_eq!(
        world.claim_group_heal(101, (7, 9)),
        None,
        "second heart, same kill"
    );
    assert_eq!(
        world.claim_group_heal(101, (7, 10)),
        Some(0),
        "same enemy, next kill"
    );
    assert_eq!(
        world.claim_group_heal(101, (8, 9)),
        Some(0),
        "other enemy, same time"
    );
}

#[test]
fn the_same_kill_id_in_two_instances_heals_each_group_once() {
    let mut world = World::runtime();
    let child = world
        .create_child_instance(0, 0, InstanceKind::Encounter { encounter_id: 0 }, 0)
        .unwrap();
    join(&mut world, 101, None);
    assert_eq!(world.claim_group_heal(101, (7, 9)), Some(0));
    let moved = world
        .prepare_membership(101, 0, Some(child), WorldPose::ORIGIN, true, 1)
        .unwrap();
    world.commit_membership(moved);
    assert_eq!(
        world.claim_group_heal(101, (7, 9)),
        Some(child),
        "the same authored enemy guid and millisecond is a separate kill in another instance"
    );
    assert_eq!(
        world.claim_group_heal(101, (7, 9)),
        None,
        "duplicate in the child instance"
    );
}

#[test]
fn a_picker_without_a_committed_place_claims_nothing() {
    let mut world = World::runtime();
    assert_eq!(world.claim_group_heal(101, (7, 9)), None);
    join(&mut world, 101, None);
    assert_eq!(
        world.claim_group_heal(101, (7, 9)),
        Some(0),
        "the refused claim consumed nothing"
    );
}

#[test]
fn the_memory_is_bounded() {
    let mut world = World::runtime();
    join(&mut world, 101, None);
    assert_eq!(world.claim_group_heal(101, (7, 9)), Some(0));
    for i in 0..300 {
        world.claim_group_heal(101, (1, i));
    }
    assert_eq!(
        world.claim_group_heal(101, (7, 9)),
        Some(0),
        "an old kill is forgotten, not kept forever"
    );
}
