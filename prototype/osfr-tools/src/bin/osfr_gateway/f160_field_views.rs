//! F160: shared-field view reconciliation around readiness/world-load loss, and
//! the per-recipient delivery isolation that keeps one full queue out of the tick.
use super::*;

impl Server {
    /// Set only the server-owned membership readiness flag.
    pub fn test_set_ready(&mut self, guid: u64, ready: bool) {
        let location = self.membership.player_location(guid).unwrap().clone();
        let zone_id = self
            .membership
            .instance(location.instance_id)
            .unwrap()
            .zone_id;
        let change = self
            .membership
            .prepare_membership(
                guid,
                zone_id,
                Some(location.instance_id),
                location.pose,
                ready,
                self.start.elapsed().as_millis() as u64,
            )
            .unwrap();
        self.membership.commit_membership(change);
    }

    /// World-load loss alone: the loader reset the session while the character
    /// stays assigned to and ready in its instance.
    pub fn test_lose_world(&mut self, guid: u64) {
        let id = self.world.active_sessions[&guid];
        let key = self.world.session_keys[&id];
        self.sessions.get_mut(&key).unwrap().gateway.reset_session();
        self.test_set_ready(guid, true);
    }
}

fn sender(max_queued_bytes: usize) -> ReliableSender {
    ReliableSender::new(
        Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap(),
        SenderConfig {
            datagram_limit: 512,
            window: 32,
            max_message: MAX_MESSAGE,
            max_queued_bytes,
            retry_ms: 250,
            max_retries: 40,
        },
    )
    .unwrap()
}

/// A full reliable queue must refuse only that viewer: the shared field keeps
/// ticking, the other viewer's removal is delivered, and a later tick retries.
#[test]
fn f160_a_full_reliable_queue_refuses_only_its_own_view() {
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("f160-view");
    let known = [131000108001_u64, 131000108002];
    // Both players are in field instance 0 and not ready: the F132 removal
    // window. Only the first still knows the field actors.
    for (index, guid) in [301_u64, 302].into_iter().enumerate() {
        let change = server
            .membership
            .prepare_membership(guid, 0, Some(0), WorldPose::ORIGIN, false, 0)
            .unwrap();
        server.membership.commit_membership(change);
        server
            .sessions
            .get_mut(&keys[index])
            .unwrap()
            .known_npcs
            .extend(known);
    }
    // The first viewer cannot accept any reliable frame.
    server.sessions.get_mut(&keys[0]).unwrap().sender = sender(1);
    let before = server.sessions[&keys[0]].known_npcs.clone();
    server.tick_shared_fields(1_000);
    assert!(
        server.faulted_fields.is_empty(),
        "one full queue must not fault the shared field"
    );
    assert_eq!(
        server.sessions[&keys[0]].known_npcs, before,
        "a refused view delivery changes nothing"
    );
    assert!(
        server.sessions[&keys[1]].known_npcs.is_empty(),
        "the other viewer is still reconciled"
    );
    assert!(
        !server.sessions[&keys[1]].sender.is_idle(),
        "the other viewer's removal was queued"
    );
    // The next tick retries the refused viewer once its queue can accept.
    server.sessions.get_mut(&keys[0]).unwrap().sender = sender(32 * 1024 * 1024);
    server.tick_shared_fields(2_000);
    assert!(server.sessions[&keys[0]].known_npcs.is_empty());
    // The whole event loop still runs with both sessions alive.
    server.step().unwrap();
    assert_eq!(server.sessions.len(), 3);
    let _ = fs::remove_dir_all(scratch);
}
