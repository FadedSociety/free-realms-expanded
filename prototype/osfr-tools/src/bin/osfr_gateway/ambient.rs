//! T507: the per-tick ambient animation pass for passive content NPCs.
//!
//! The table and its stateless schedule live in `osfr_tools::npc_ambient`, so
//! two viewers that know the same actor play the same 35/8 in the same window.
//! One viewer's full reliable queue is refused for that viewer only; the shared
//! tick and every other session keep running.
use super::*;

/// Queue this viewer's due ambient 35/8 packets for `(from_ms, to_ms]`.
pub(super) fn queue_due(session: &mut Session, from_ms: u64, to_ms: u64) {
    let Some(game) = session.gateway.gameplay() else {
        return;
    };
    let Some(ambient) = game.npc_ambient() else {
        return;
    };
    let due = ambient.due_for_known(
        &session.known_npcs,
        |guid| game.npc_model(guid),
        from_ms,
        to_ms,
    );
    for (guid, id) in due {
        // 35/8 SetAnimation: header reader 0x008d6830, payload reader 0x008e5dd0.
        // Flags 0 clears the base bit, so the client plays the row's emote now.
        let packet = wrap_application(&osfr_tools::encounter::animation(guid, id, false));
        if let Err(error) = session.queue_shared_message(&packet) {
            Server::refuse_field_delivery(session, error);
            return;
        }
    }
}
