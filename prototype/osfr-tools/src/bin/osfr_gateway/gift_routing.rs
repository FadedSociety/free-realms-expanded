//! gifts-codes: the host half of the Claim window's gifts, ticked from `mail_routing::tick` (the
//! realm's post, one parent, so the hub keeps no line for it). A player who has just entered the
//! world, an account a gift was sent to, an account whose list a character claim moved, and a
//! session whose running events changed get the whole list again (108/2, which also posts the
//! client's own `ClaimNotification` per row) after the due standing offers are added, plus a
//! chat line when gifts wait. The tick reads files only for those sessions.
use super::*;
use osfr_tools::chat;
use osfr_tools::gifts::{self, claim};
use std::collections::{BTreeSet, HashMap};
use std::sync::{LazyLock, Mutex, PoisonError};

/// (R) How often a session's running events are compared (wall-clock ms); tests pin event
/// clocks and compare on every tick.
const EVENT_CHECK_MS: u64 = if cfg!(test) { 0 } else { 5_000 };

/// What this host has told each session, by session id: never saved.
#[derive(Default)]
struct Seen {
    /// Event key to its run, as `claim::running_events` read them last.
    events: gifts::Events,
    checked_ms: u64,
}

// ponytail: one process map keyed by account store (test gateways each have their own), as
// `gifts`' hand-offs; a field of `Server` would need a hub line.
static SEEN: LazyLock<Mutex<HashMap<(PathBuf, u64), Seen>>> = LazyLock::new(Default::default);

pub(super) fn tick(sessions: &mut SessionMap, store: &Path, now_ms: u64) {
    let delivered = gifts::take_delivered(store);
    let resend = gifts::take_resend(store);
    let mut seen = SEEN.lock().unwrap_or_else(PoisonError::into_inner);
    let mut live = BTreeSet::new();
    for session in sessions.values_mut() {
        if !session.selected_character || !session.gateway.is_world_loaded() {
            continue;
        }
        let id = (store.to_path_buf(), u64::from(session.id.0));
        live.insert(id.clone());
        let Some(game) = session.gateway.gameplay() else {
            continue;
        };
        let first = !seen.contains_key(&id);
        let state = seen.entry(id).or_default();
        let mut moved = false;
        if first || now_ms.saturating_sub(state.checked_ms) >= EVENT_CHECK_MS {
            let events = claim::running_events(game);
            // An event that starts brings its rows; one that ends takes them out of the list.
            moved = !first && events != state.events;
            state.events = events;
            state.checked_ms = now_ms;
        }
        let account = session.account_guid;
        let arrived = delivered.contains(&account);
        if !(first || moved || arrived || resend.contains(&account)) {
            continue;
        }
        let listing = claim::refresh(store, account, game, session.character_guid);
        let mut packets = vec![listing.packet];
        if (first || arrived || listing.added > 0) && listing.gifts > 0 {
            let plural = if listing.gifts == 1 { "gift is" } else { "gifts are" };
            packets.push(chat::system(&format!(
                "{} {plural} waiting in your Claim window (Marketplace).",
                listing.gifts
            )));
        }
        for packet in packets {
            if let Err(error) = session.sender.queue_message(&wrap_application(&packet)) {
                eprintln!("gifts: session {}: {error}", session.id.0);
            }
        }
    }
    seen.retain(|id, _| id.0 != store || live.contains(id));
}

#[cfg(test)]
#[path = "gift_routing_tests.rs"]
mod tests;
