//! Staff `realm` words on a session gateway: `realm role` and the L042 moderation commands.
//!
//! The hub keeps only dispatch. A moderation command is staged in `GatewayOutcome::moderation`
//! and run by the host after this transaction commits (`osfr_gateway/moderation.rs`), so a
//! refused or failed transaction changes nothing.

use super::{Gateway, GatewayOutcome, wrapped};
use crate::gameplay::Gameplay;
use crate::moderation::{Action, Command, unix_now};

impl Gateway {
    /// Answer or stage `realm role` and the moderation words; false for any other text, which
    /// goes on to gameplay. The role gate (`Gameplay::may_run`, the account-store role) runs
    /// before the name is parsed, so an unauthorized sender learns nothing about other players.
    pub(super) fn staff_command(
        &mut self,
        game: &mut Gameplay,
        text: &str,
        outcome: &mut GatewayOutcome,
    ) -> bool {
        let lower = text.trim().to_ascii_lowercase();
        let command = lower.strip_prefix("realm ").unwrap_or_default();
        let first = command.split_whitespace().next().unwrap_or_default();
        if first != "role" && Action::parse(first).is_none() {
            return false;
        }
        let reply = if !game.may_run(first) {
            outcome.dev_command_denials += 1;
            "Cannot do that: developer command requires --dev-tools and an authorized account role."
        } else if first == "role" {
            self.change_account_role(command)
        } else {
            match Command::parse(command) {
                Ok(staged) => {
                    outcome.moderation.push(staged);
                    return true;
                }
                Err(usage) => usage,
            }
        };
        game.set_role(self.role);
        outcome.replies.push(wrapped(&crate::chat::system(reply)));
        true
    }

    /// L042: whether this account's chat is refused now. `refresh_role` re-reads the stored
    /// mute with the role on every message; it expires by the wall clock.
    pub fn chat_muted(&self) -> bool {
        self.sanctions.muted(unix_now())
    }
}
