//! Application-delivery framing for the gateway: one place that wraps an inner packet in
//! the 5/1 tunnel the client decodes, builds a chat refusal, or fans a server notice out.
use super::*;

/// Wrap one inner application packet in the 5/1 tunnel.
pub(super) fn wrap_application(inner: &[u8]) -> Vec<u8> {
    let mut out = vec![5, 0, 1];
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

/// T008/F290: one opcode-164 jump body to the sessions whose directed view
/// contains the sender (the `ride_fanout` visibility rule). Called after the
/// commit succeeds, like the T199 broadcast; the sender never receives its own
/// jump. Each character has a relay budget: over budget the 164 is dropped
/// silently, while the accepted pose still fans out through the 125 path. The
/// budget uses the server's monotonic wall clock (`Instant::now()`), the
/// lead-accepted F290 behaviour; F309 briefly anchored it to the dispatch clock
/// and F323 reverted that unsigned clock change.
pub(super) fn jump_fanout(
    world: &SharedWorld,
    sender: SessionId,
    jump: Option<Vec<u8>>,
) -> Vec<Delivery> {
    let Some(jump) = jump else {
        return Vec::new();
    };
    let Some((zone, position, character)) = world
        .players
        .get(sender)
        .filter(|_| world.joined.contains(&sender))
        .map(|player| (player.zone, player.position, player.character_guid))
    else {
        return Vec::new();
    };
    let viewers = world
        .players
        .nearby_sessions(zone, position)
        .into_iter()
        .filter(|recipient| {
            *recipient != sender && world.players.knows_character(*recipient, character)
        })
        .collect::<Vec<_>>();
    if viewers.is_empty()
        || !world
            .security
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .jump_relays
            .allow(character, Instant::now())
    {
        return Vec::new();
    }
    let bytes = wrap_application(&jump);
    viewers
        .into_iter()
        .map(|recipient| Delivery {
            recipient,
            bytes: bytes.clone(),
        })
        .collect()
}

/// Queue the notices staged with a successful transaction: the T199 broadcast,
/// then the T008/F290 jump relay. The caller passes only a committed outcome, so
/// a refused or failed transaction queues neither.
pub(super) fn post_commit(
    session: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    broadcast: Option<String>,
    jump: Option<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(text) = broadcast {
        for delivery in broadcast_deliveries(&text, world) {
            queue_delivery(session, sessions, world, delivery)?;
        }
    }
    for delivery in jump_fanout(world, session.id, jump) {
        queue_delivery(session, sessions, world, delivery)?;
    }
    Ok(())
}

/// Drop a character's relay budget when its session ends.
pub(super) fn forget_jump_relay(world: &SharedWorld, character: u64) {
    world
        .security
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .jump_relays
        .remove(&character);
}

/// One client-refusal reply, or nothing when the error has no client form.
pub(super) fn chat_error_delivery(
    error: osfr_tools::social_chat::ChatError,
    recipient: SessionId,
) -> Vec<Delivery> {
    error
        .client_reply()
        .map(|bytes| {
            vec![Delivery {
                recipient,
                bytes: wrap_application(&bytes),
            }]
        })
        .unwrap_or_default()
}

/// T199: one wrapped 15/3 system line for every session that has entered the world
/// (`chat_sessions`), including the sender. Encoded once; each recipient gets its own
/// copy. The caller queues these only after `commit_outcome` succeeds; a recipient that
/// leaves before queueing is skipped by `queue_delivery`.
pub(super) fn broadcast_deliveries(text: &str, world: &SharedWorld) -> Vec<Delivery> {
    let bytes = wrap_application(&osfr_tools::world_notice::encode(text));
    world
        .chat_sessions
        .keys()
        .map(|recipient| Delivery {
            recipient: *recipient,
            bytes: bytes.clone(),
        })
        .collect()
}
