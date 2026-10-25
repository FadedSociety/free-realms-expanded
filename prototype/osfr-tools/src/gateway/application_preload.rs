//! Application requests handled around the gateway's pre-world gate: W174's accepted and
//! discarded quiet list, and the login-time Fotomat 156/2 that the client sends before Loaded.
use super::*;

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

impl Gateway {
    /// A 156/2 for this session's own character before the world is loaded. `None` means no
    /// reply: another guid, a bad provider, or no appearance yet. After Loaded the in-world
    /// route answers, where it can also see other players' presentations.
    pub(super) fn fotomat_loading_reply(
        &self,
        inner: &[u8],
        own_guid: u64,
        loaded: bool,
    ) -> Option<Vec<u8>> {
        if loaded || !crate::fotomat::is_request(inner) {
            return None;
        }
        self.current_look(self.gameplay.as_ref())?;
        let presentation = self.player_presentation(false).ok()?;
        crate::fotomat::handle(inner, |guid| (guid == own_guid).then_some(&presentation))
    }
}
