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

    /// `realm role <account-id | First Last> <role>`: admin only (`set_account_role` checks the
    /// actor). L081: a character name works too, for the panel's `!admin promote|demote`.
    fn change_account_role(&mut self, command: &str) -> &'static str {
        use crate::account_store::{AccountRole, AccountStore, AccountStoreLock, STAFF_LOCK_WAIT};
        let usage = "Usage: realm role <account-id or First Last> <player|builder|moderator|admin>";
        let words: Vec<_> = command.split_whitespace().collect();
        let ["role", target @ .., role] = words.as_slice() else {
            return usage;
        };
        // Account 0 is never real: an id of 0 stays a usage error, as before names were taken.
        let zero = matches!(target, [id] if id.parse::<u64>() == Ok(0));
        let Some(role) = AccountRole::parse(role).filter(|_| !target.is_empty() && !zero) else {
            return usage;
        };
        let Some((path, actor)) = &self.role_account else {
            return "Cannot do that: a verified account binding is required.";
        };
        // (R) Cooperate with login/CLI writers; never overwrite a newer key or role.
        let Ok(_lock) = AccountStoreLock::acquire_within(path, STAFF_LOCK_WAIT) else {
            return "Cannot do that: account store is busy or unavailable.";
        };
        let result = AccountStore::load(path).and_then(|mut store| {
            let id = match target {
                [id] if id.parse::<u64>().is_ok() => id.parse().ok(),
                name => store.account_by_display_name(
                    &name.iter().map(|w| w.to_string()).collect::<Vec<_>>(),
                ),
            };
            let id = id
                .filter(|id| *id > 0)
                .ok_or_else(|| std::io::Error::other("no such account"))?;
            store.set_account_role(*actor, id, role)?;
            store.save(path)
        });
        self.refresh_role();
        if result.is_ok() {
            "Account role saved."
        } else {
            "Cannot do that: account role was not saved."
        }
    }

    /// L042: whether this account's chat is refused now. `refresh_role` re-reads the stored
    /// mute with the role on every message; it expires by the wall clock.
    pub fn chat_muted(&self) -> bool {
        self.sanctions.muted(unix_now())
    }
}
