//! W158 slice 2b: the world remembers which kills already gave their group heal.
use osfr_tools::world::World;

#[test]
fn a_kill_gives_its_group_heal_once() {
    let mut world = World::runtime();
    assert!(world.claim_group_heal((7, 9)));
    assert!(!world.claim_group_heal((7, 9)), "second heart, same kill");
    assert!(world.claim_group_heal((7, 10)), "same enemy, next kill");
    assert!(world.claim_group_heal((8, 9)), "other enemy, same time");
}

#[test]
fn the_memory_is_bounded() {
    let mut world = World::runtime();
    assert!(world.claim_group_heal((7, 9)));
    for i in 0..300 {
        world.claim_group_heal((1, i));
    }
    assert!(
        world.claim_group_heal((7, 9)),
        "an old kill is forgotten, not kept forever"
    );
}
