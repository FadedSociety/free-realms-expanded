//! (R) Outbound application-reply budget for one client message (F179).
//!
//! `Gateway::handle_at` charges a whole `[0, 25]` bundle's replies as it builds them. The session
//! cannot see those bytes when it splits a combat bundle into ordered children, so `BundleBudget`
//! gives that split path the same ceiling. This is server memory policy below the sender's 32 MiB
//! queue, not a recovered client limit; the lead accepted the value on 2026-10-19. A refusal ends
//! only that player's session.

use std::io;

/// (R) One bundled request may stage at most this many application-reply bytes.
const MAX_BUNDLE_REPLY_BYTES: usize = 8 * 1024 * 1024;

/// Staged application-reply bytes for one client message; None outside a `[0, 25]` bundle.
pub struct BundleBudget(Option<usize>);

impl BundleBudget {
    /// Outside a bundle the sender queue already bounds the replies.
    pub fn outside() -> Self {
        Self(None)
    }

    /// A message whose bundle flag is already known.
    pub fn bundle(combined: bool) -> Self {
        Self(combined.then_some(0))
    }

    /// A message as it arrived; only a `[0, 25]` bundle accumulates.
    pub fn message(message: &[u8]) -> Self {
        Self::bundle(message.starts_with(&[0, 25]))
    }

    /// Charge replies about to be staged; a bundle over the budget is refused before commit.
    /// Returns the bytes charged so a caller that discards them can [`Self::release`] them.
    pub fn stage(&mut self, replies: &[Vec<u8>]) -> io::Result<usize> {
        let Some(used) = &mut self.0 else {
            return Ok(0);
        };
        let charged = replies.iter().map(Vec::len).sum::<usize>();
        *used = used.saturating_add(charged);
        if *used > MAX_BUNDLE_REPLY_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "bundled application response exceeds the outbound budget",
            ));
        }
        Ok(charged)
    }

    /// Drop a charge whose replies were discarded before queueing (a refused precommit
    /// transfer). A refusal must not charge later children for replies nobody receives.
    pub fn release(&mut self, bytes: usize) {
        if let Some(used) = &mut self.0 {
            *used = used.saturating_sub(bytes);
        }
    }

    /// Reply bytes charged so far; None outside a bundle.
    pub fn used(&self) -> Option<usize> {
        self.0
    }
}
