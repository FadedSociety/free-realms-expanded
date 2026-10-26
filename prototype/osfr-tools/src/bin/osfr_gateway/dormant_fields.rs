//! Dormant by default: a shared field with nobody in it ticks once more, then sleeps.
use super::*;

#[test]
fn an_empty_field_settles_once_then_sleeps_until_someone_enters() {
    if !super::tests::test_support::bin_test_target("dormant") {
        return;
    }
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("dormant");
    let instance = server.membership.public_instance(0).unwrap();
    assert!(server.membership.field_instances().contains(&instance));
    // The field's own tick clock: `due` stamps it on every real tick.
    let last_tick = |server: &Server| {
        u64::MAX
            - server
                .membership
                .field(instance)
                .unwrap()
                .clone()
                .due(u64::MAX)
                .unwrap()
    };

    // Nobody is in the instance: the first tick still runs, dropping departed casters' state.
    server.tick_shared_fields(1_000);
    assert_eq!(
        last_tick(&server),
        1_000,
        "the first empty tick settles the field"
    );
    server.tick_shared_fields(2_000);
    assert_eq!(last_tick(&server), 1_000, "a settled empty field sleeps");

    // Someone enters (any readiness): it ticks again.
    let guid = server.sessions[&keys[0]].character_guid;
    server
        .membership
        .enter_public(guid, 0, osfr_tools::world::WorldPose::ORIGIN, 3_000)
        .unwrap();
    server.tick_shared_fields(3_000);
    assert_eq!(last_tick(&server), 3_000, "an occupied field wakes");
    let _ = fs::remove_dir_all(scratch);
}
