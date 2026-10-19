//! The isolation boundary: one session's panic ends that session alone.
use super::*;
use std::cell::Cell;

thread_local! {
    /// Test-only: the session whose next tick panics, standing in for any feature bug.
    pub(super) static PANIC_IN_TICK: Cell<Option<SessionId>> = const { Cell::new(None) };
}

#[test]
fn a_panicking_session_ends_alone_and_the_server_keeps_running() {
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("isolation");
    let victim = server.sessions.get_mut(&keys[0]).unwrap();
    let (victim_id, victim_guid) = (victim.id, victim.character_guid);
    // Its save cannot be written either (the parent is a file): cleanup must still run.
    fs::write(scratch.join("blocker"), b"").unwrap();
    victim.selected_character = true;
    victim.state_path = scratch.join("blocker/character.json");
    PANIC_IN_TICK.set(Some(victim_id));
    server.next_tick_ms = 0;

    let stepped = server.step();
    PANIC_IN_TICK.set(None);
    stepped.unwrap();

    assert!(!server.sessions.contains_key(&keys[0]));
    assert!(
        !server.world.active_sessions.contains_key(&victim_guid),
        "no ghost left behind"
    );
    assert!(server.sessions.contains_key(&keys[1]) && server.sessions.contains_key(&keys[2]));
    server.next_tick_ms = 0;
    server.step().unwrap();
    assert_eq!(server.sessions.len(), 2);
    let _ = fs::remove_dir_all(scratch);
}

#[test]
fn a_failing_tick_save_ends_alone_and_the_server_keeps_running() {
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("tick-save-error");
    let victim = server.sessions.get_mut(&keys[0]).unwrap();
    let old_guid = victim.character_guid;
    let victim_guid = victim.gateway.state().guid;
    // A bare `Session` keeps the template guid until the first commit re-stamps
    // it from the gateway; mirror that in the world so disconnect cleanup runs.
    victim.character_guid = victim_guid;
    server.world.active_sessions.remove(&old_guid);
    server.world.active_sessions.insert(victim_guid, victim.id);
    // The per-session tick writes a deferred pose; its parent is a file, so the
    // production save fails and the tick returns Err. No test-only hook.
    fs::write(scratch.join("blocker"), b"").unwrap();
    victim.state_path = scratch.join("blocker/character.json");
    victim.position_dirty = true;
    victim.position_saved_ms = None;
    server.next_tick_ms = 0;

    server.step().unwrap();

    assert!(!server.sessions.contains_key(&keys[0]));
    assert!(
        !server.world.active_sessions.contains_key(&victim_guid),
        "no ghost left behind"
    );
    assert!(server.sessions.contains_key(&keys[1]) && server.sessions.contains_key(&keys[2]));
    server.next_tick_ms = 0;
    server.step().unwrap();
    assert_eq!(server.sessions.len(), 2);
    let _ = fs::remove_dir_all(scratch);
}
