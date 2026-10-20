//! (R) Outbound application-reply budget for one client message (F179).
//!
//! `Gateway::handle_at` charges a whole `[0, 25]` bundle's replies as it builds them. The session
//! cannot see those bytes when it splits a combat bundle into ordered children, so `BundleBudget`
//! gives that split path the same ceiling. This is server memory policy below the sender's 32 MiB
//! queue, not a recovered client limit; the lead accepted the value on 2026-10-19. A refusal ends
//! only that player's session.
//!
//! The charge is taken when replies are staged, before that child's transaction commits. A
//! precommit transfer refusal discards its staged replies but leaves their bytes charged for the
//! rest of the bundle (F179 finding 4; F286 documents this as the deliberate accounting the
//! review accepted in place of a refund): the ceiling bounds the bytes one message stages, not
//! the bytes the sender queues. Refunding them would change the lead-signed budget policy and
//! needs lead sign-off, so builders must not add a release here.

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
    /// The charge stays even if the staged child's transaction then refuses (module note).
    pub fn stage(&mut self, replies: &[Vec<u8>]) -> io::Result<()> {
        let Some(used) = &mut self.0 else {
            return Ok(());
        };
        *used = used.saturating_add(replies.iter().map(Vec::len).sum::<usize>());
        if *used > MAX_BUNDLE_REPLY_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "bundled application response exceeds the outbound budget",
            ));
        }
        Ok(())
    }

    /// Reply bytes charged so far; None outside a bundle. Observation only: the budget tests are
    /// its only callers; the server never reads it.
    pub fn used(&self) -> Option<usize> {
        self.0
    }
}
