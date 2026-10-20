//! Matchmaking list/stat replies, the empty in-progress list, and the lobby
//! close request. The wire codecs and the reconstructed listing policy live in
//! `crate::matchmaking`; this module only connects them to the activity
//! catalogue the portal already publishes.
//!
//! A join is refused without touching queue, character or session state. The
//! handler borrows `&self`, so a refusal cannot commit anything.

use super::*;

impl Gameplay {
    pub(super) fn handle_matchmaking(&self, inner: &[u8]) -> io::Result<Option<Outcome>> {
        // Cheap family gate: every other in-world packet returns before the
        // catalogue is turned into queue rows.
        if !crate::matchmaking::handles(inner) {
            return Ok(None);
        }
        let listings = crate::matchmaking::Listings::from_activities(self.activities.activities())
            .map_err(|error| invalid(error.to_string()))?;
        let Some(result) = crate::matchmaking::route(inner, &listings)
            .map_err(|error| invalid(error.to_string()))?
        else {
            return Ok(None);
        };
        let mut out = Outcome {
            replies: result.replies,
            ..Outcome::default()
        };
        if let Some(refusal) = result.refusal {
            out.replies.push(chat::system(refusal.message()));
        }
        Ok(Some(out))
    }
}
