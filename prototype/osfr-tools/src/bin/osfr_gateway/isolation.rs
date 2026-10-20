//! The isolation boundary: one session's panic ends that session alone.
use super::*;
use fixtures::{bundle, wrap};
use std::cell::Cell;

#[path = "fishing_fixtures.rs"]
#[allow(dead_code)]
mod fixtures;

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

/// A gateway whose Harvesting run is live, started through the real dev command
/// so the movie gate (not a test hook) produces the malformed-close error.
/// `None` when the private W104 fixture is absent (a clean checkout or a cleaned
/// lab build): the test skips instead of failing for an environment reason.
fn movie_gateway(reference: &std::path::Path) -> Option<Gateway> {
    let content =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/build/w104/scratch/content");
    if !content.join("harvesting.json").is_file() {
        eprintln!(
            "skipping: the private W104 harvesting fixture is missing at {}",
            content.display()
        );
        return None;
    }
    let mut state = Gameplay::load(&content, None).unwrap().state();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = 45;
    profiles.unlocked.insert(45);
    let character = Gateway::reference_state(reference).unwrap();
    let mut gateway = Gateway::load_gameplay(reference, character, &content, Some(state)).unwrap();
    gateway.set_dev_tools(true);
    gateway.set_role(osfr_tools::account_store::AccountRole::Admin);
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["movie-error", "1.910.1.530630", ""].iter().enumerate() {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(1_u64.to_le_bytes());
        }
    }
    gateway.handle_at(&login, 0).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), 100).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), 200).unwrap();
    let mut chat = vec![15, 0, 1, 0, 0, 0];
    chat.extend(1_u64.to_le_bytes());
    chat.extend(0_u64.to_le_bytes());
    for _ in 0..2 {
        chat.extend([0; 12]);
        chat.extend(0_u32.to_le_bytes());
        chat.extend(0_u32.to_le_bytes());
    }
    let command = b"realm harvest";
    chat.extend((command.len() as u32).to_le_bytes());
    chat.extend(command);
    chat.extend([0; 28]);
    gateway.handle_at(&wrap(&chat), 300).unwrap();
    assert!(
        gateway
            .snapshot()
            .gameplay
            .and_then(|gameplay| gameplay.harvesting)
            .is_some_and(|harvesting| harvesting.active.is_some()),
        "the fixture must leave a live harvesting run"
    );
    Some(gateway)
}

#[test]
fn a_movie_error_ends_only_that_session_and_not_the_server() {
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("movie-error");
    let reference = server.options.reference.clone();
    let Some(gateway) = movie_gateway(&reference) else {
        let _ = fs::remove_dir_all(scratch);
        return;
    };
    // A real client socket at the session's peer address; the malformed close
    // arrives over the production reliable transport and `Server::step` path.
    let client = UdpSocket::bind("127.0.0.1:0").expect("client socket binds");
    let victim = server.sessions.get_mut(&keys[0]).unwrap();
    victim.gateway = gateway;
    victim.selected_character = true;
    victim.state_path = scratch.join("victim.json");
    let victim_guid = victim.character_guid;
    let victim_id = victim.id;
    let mut victim = server.sessions.remove(&keys[0]).unwrap();
    victim.peer = client.local_addr().expect("client address");
    let key = SessionKey {
        peer: victim.peer,
        connection_id: victim.request.id,
    };
    server.world.session_keys.insert(victim_id, key);
    server.sessions.insert(key, victim);

    // A 26/42 with a trailing byte is a handler error; a bundle makes the child
    // error surface instead of a silent single-request refusal, and the server
    // turns it into an end for this session alone (see `Server::step`).
    let wire = {
        let session = &server.sessions[&key];
        let mut payload = 0u16.to_be_bytes().to_vec();
        payload.extend(bundle(&[vec![26, 0, 42, 0, 0]]));
        session
            .codec
            .encode(RELIABLE_DATA, &payload)
            .expect("malformed close datagram encodes")
    };
    client
        .send_to(&wire, server.local_addr().expect("server address"))
        .expect("malformed close datagram sends");
    server
        .step()
        .expect("one handler error must not stop the server");

    assert!(!server.sessions.contains_key(&key));
    assert!(
        !server.world.active_sessions.contains_key(&victim_guid),
        "no ghost left behind"
    );
    assert!(server.sessions.contains_key(&keys[1]) && server.sessions.contains_key(&keys[2]));
    // Loading the movie content twice above can outlast the 60 s idle budget on
    // a loaded host. The two survivors send nothing in this test, so refresh
    // their liveness; the step below must keep them for the movie error alone.
    for key in &keys[1..] {
        if let Some(session) = server.sessions.get_mut(key) {
            session.last_activity = std::time::Instant::now();
        }
    }
    server.next_tick_ms = 0;
    server.step().unwrap();
    assert_eq!(server.sessions.len(), 2);
    let _ = fs::remove_dir_all(scratch);
}
