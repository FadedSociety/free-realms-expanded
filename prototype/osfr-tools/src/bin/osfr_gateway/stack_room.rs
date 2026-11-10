//! Stack room for the session host.
//!
//! `Session`, `Server` and `Gateway` are moved by value: out of and into the session map,
//! through the staging tuples, out of the loaders and the test rigs. In a debug build every such
//! move, and every staged clone, is its own stack slot in its frame. While `Gameplay` (the whole
//! per-character game state, about 16 KB and growing with every system) sat inline in `Gateway`,
//! and `Gateway` inline in `Session` and `Server`, the per-packet path held several copies in
//! frames that stay live under the whole dispatch: the session's staged gateway, and in
//! `Gateway::handle_at` the staged `Option<Gameplay>` plus one more clone per application
//! message. `quest_reward_save_failure::a_failed_save_ends_only_that_session` then needed about
//! 2.0 MB of its 2 MB debug test-thread stack and overflowed (exit 0xc00000fd) on any build that
//! added a few hundred bytes of game state.
//!
//! So the big value is boxed at each owner: `Gateway::gameplay`, `Session::gateway` and
//! `Server::template`. A staged copy is now a pointer, and game-state growth no longer reaches
//! these types or those frames. The ceilings below make the next inline growth fail here, with
//! the size, instead of as a stack crash somewhere else. If one fails: box the new large field.
//! Do not lift a ceiling past a few KB, and do not raise any stack size.

use super::super::*;
use std::mem::size_of;

/// Measured 1,032 (19,112 with the gateway inline).
const SESSION_CEILING: usize = 2 * 1024;
/// Measured 2,952 (21,032 with the template gateway inline).
const SERVER_CEILING: usize = 4 * 1024;
/// Measured 2,224 (18,088 with `Gameplay` inline).
const GATEWAY_CEILING: usize = 4 * 1024;

#[test]
fn stack_room_sizes() {
    let sizes = [
        ("Session", size_of::<Session>(), SESSION_CEILING),
        ("Server", size_of::<Server>(), SERVER_CEILING),
        ("Gateway", size_of::<Gateway>(), GATEWAY_CEILING),
    ];
    eprintln!(
        "SIZES {sizes:?} gameplay={} outcome={} world={}",
        size_of::<osfr_tools::gameplay::Gameplay>(),
        size_of::<GatewayOutcome>(),
        size_of::<SharedWorld>(),
    );
    for (name, size, ceiling) in sizes {
        assert!(
            size <= ceiling,
            "{name} is {size} bytes (ceiling {ceiling}): it is moved by value on the packet path, box the new large field"
        );
    }
}
