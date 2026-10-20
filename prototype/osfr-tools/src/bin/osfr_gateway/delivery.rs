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
