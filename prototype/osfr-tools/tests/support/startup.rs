//! The shared login/ready/world-entry driver for the startup content suites.
//!
//! `tests/startup_content.rs` pins the 46 golden startup messages; the T853
//! `tests/release_content.rs` suite also needs the world-entry (10/0) replies, so
//! the two drivers live here beside the manifest version read from the same
//! content directory. Keeping one sequence keeps both suites from drifting apart.

use crate::{
    gateway_packets::{login_with_version, wrap},
    json,
};
use osfr_tools::{character_store::CharacterState, gateway::Gateway};
use std::path::Path;

/// The login and (13/0) ready replies, before the client's world-entry signal.
fn login_and_ready(gateway: &mut Gateway, state: &CharacterState, version: &str) -> Vec<Vec<u8>> {
    let mut messages = gateway
        .handle(&login_with_version(state.guid, version))
        .unwrap()
        .replies;
    messages.extend(gateway.handle(&wrap(&[13, 0])).unwrap().replies);
    messages
}

/// A session up to the client's ready and loaded signals: the 46 startup messages,
/// in order. The world-entry replies are produced and checked but not returned.
#[allow(dead_code)] // The release suite needs the world-entry replies instead.
pub fn startup_of(gateway: &mut Gateway, state: &CharacterState, version: &str) -> Vec<Vec<u8>> {
    let messages = login_and_ready(gateway, state, version);
    assert!(gateway.handle(&wrap(&[10, 0])).unwrap().entered_world);
    assert!(gateway.is_world_loaded());
    messages
}

/// The same session plus the (10/0) world-entry replies (actor spawns, the 44/2
/// category packet and the rest of the loaded world).
#[allow(dead_code)] // The T807 golden suite does not compare the world-entry replies.
pub fn startup_and_world(
    gateway: &mut Gateway,
    state: &CharacterState,
    version: &str,
) -> Vec<Vec<u8>> {
    let mut messages = login_and_ready(gateway, state, version);
    let world = gateway.handle(&wrap(&[10, 0])).unwrap();
    assert!(world.entered_world);
    assert!(gateway.is_world_loaded());
    messages.extend(world.replies);
    messages
}

/// The client version the content directory's startup manifest claims.
pub fn content_version(content: &Path) -> String {
    json::read(&content.join("startup/manifest.json"))["client_version"]
        .as_str()
        .unwrap()
        .to_owned()
}
