//! D5: the welcome window's Popular Items read `undefined` because the Marketplace catalogue
//! (66/9 bundle groups, 66/5 store) reached the client after the welcome splash (93).
//!
//! `WelcomeHandler:Show` selects store group 6 as the window opens; the selection fills the
//! contents data source from the group record at that moment (`0099efd0` -> `009a3a50`,
//! `StoreBundleGroupContents__Refresh`), and a group that has not arrived leaves it empty. The
//! 2026-11-12 baseline run sent 93 at 77467 ms, got the client's 66/10 at 77574 ms and only then
//! sent 66/9 and the 320 KB 66/5. The login stream now carries the catalogue before the splash.

#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/gateway_packets.rs"]
#[allow(dead_code)]
mod gateway_packets;
#[path = "support/json.rs"]
#[allow(dead_code)]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;

use gateway_packets::{login_with_version, wrap};
use npc_visibility::content_dir;
use osfr_tools::gateway::Gateway;
use std::{fs, path::Path};

/// The index of the first gateway reply whose inner packet starts with `head`. A reply is the
/// application frame `[5, 0, 1, u32 length, inner...]`.
fn first_reply(replies: &[Vec<u8>], head: &[u8]) -> Option<usize> {
    replies.iter().position(|reply| reply.get(7..).is_some_and(|inner| inner.starts_with(head)))
}

#[test]
fn the_marketplace_catalogue_reaches_the_client_before_the_welcome_splash() {
    let root = std::env::temp_dir().join(format!("osfr-popular-order-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    content_copy::copy_tree(&content_dir(), &root);
    content_copy::copy_tree(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release"), &root);
    let state = Gateway::content_state(&root).unwrap();
    let version = json::read(&root.join("startup/manifest.json"))["client_version"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut gateway = Gateway::load_gameplay_from_content(state.clone(), &root, None).unwrap();
    let mut replies = gateway.handle(&login_with_version(state.guid, &version)).unwrap().replies;
    replies.extend(gateway.handle(&wrap(&[13, 0])).unwrap().replies);
    let _ = fs::remove_dir_all(&root);

    let welcome = first_reply(&replies, &[93, 0]).expect("the login stream carries the splash");
    let groups = first_reply(&replies, &[66, 0, 9, 0]).expect("66/9 is in the login stream");
    let store = first_reply(&replies, &[66, 0, 5, 0]).expect("66/5 is in the login stream");
    assert!(groups < welcome, "66/9 (at {groups}) must precede the welcome 93 (at {welcome})");
    assert!(store < welcome, "66/5 (at {store}) must precede the welcome 93 (at {welcome})");
}
