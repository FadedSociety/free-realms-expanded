//! Application requests handled around the gateway's pre-world gate: W174's accepted and
//! discarded quiet list, and the login-time Fotomat 156/2 that the client sends before Loaded.
use super::*;
use crate::players::PlayerPresentation;

/// Requests accepted and discarded before gameplay (W174). 109 text can hold local paths, so
/// nothing is stored and nothing is relayed.
pub(super) fn quiet_application(body: &[u8]) -> bool {
    matches!(
        application_handler(body),
        "client_metrics"
            | "client_log"
            | "ui_telemetry"
            | "announcements_request"
            | "client_exit_notice"
            | "free_interaction"
            | "profile_notice"
    )
}

/// The pre-Loaded Fotomat answer. `presentation` is called only for the session's own,
/// well-formed 156/2, so a foreign guid or a 115 refuses before the presentation clone. The
/// caller has already checked [`crate::fotomat::handles`] and `!loaded`.
pub(super) fn loading_reply(
    inner: &[u8],
    own_guid: u64,
    presentation: impl FnOnce() -> Option<PlayerPresentation>,
) -> Option<Vec<u8>> {
    if crate::fotomat::is_upload(inner) {
        // `is_upload` is the exact six-byte shape. Nothing is stored until the lead's
        // pending-CRC ingress, and the quiet notice never replies.
        return None;
    }
    if crate::fotomat::requested_guid(inner)? != own_guid {
        return None;
    }
    let presentation = presentation()?;
    crate::fotomat::handle(inner, |_| Some(&presentation))
}

impl Gateway {
    /// A 156/2 for this session's own character before the world is loaded. `None` means no
    /// reply: another guid, a bad provider, or no appearance yet. After Loaded the in-world
    /// route answers, where it can also see other players' presentations.
    pub(super) fn fotomat_loading_reply(&self, inner: &[u8], own_guid: u64) -> Option<Vec<u8>> {
        loading_reply(inner, own_guid, || {
            self.current_look(self.gameplay.as_ref())?;
            self.player_presentation(false).ok()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(guid: u64) -> Vec<u8> {
        let mut out = vec![156, 0, 2, 0];
        out.extend_from_slice(&guid.to_le_bytes());
        out.extend_from_slice(&8u32.to_le_bytes());
        out.extend_from_slice(b"Headshot");
        out
    }

    fn presentation(guid: u64) -> PlayerPresentation {
        PlayerPresentation {
            character_guid: guid,
            ..PlayerPresentation::default()
        }
    }

    #[test]
    fn a_foreign_guid_never_builds_the_presentation() {
        let reply = loading_reply(&request(2), 1, || {
            panic!("a foreign guid must refuse before the presentation clone")
        });
        assert!(reply.is_none());
    }

    #[test]
    fn an_upload_never_builds_the_presentation() {
        let reply = loading_reply(&[115, 0, 1, 2, 3, 4], 1, || {
            panic!("a 115 must refuse before the presentation clone")
        });
        assert!(reply.is_none());
        // A malformed 115 is not `is_upload`; it also refuses at the request parse.
        let reply = loading_reply(&[115, 0, 1, 2, 3], 1, || {
            panic!("a malformed 115 must not build the presentation")
        });
        assert!(reply.is_none());
    }

    #[test]
    fn an_own_request_builds_the_presentation_and_replies() {
        let reply = loading_reply(&request(1), 1, || Some(presentation(1)));
        assert!(
            reply.is_some_and(|bytes| bytes.starts_with(&[156, 0, 3, 0])),
            "the own 156/2 gets one 156/3"
        );
    }
}
