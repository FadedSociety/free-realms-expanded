//! Who may do what on the GM panel. Each Admin Core action names its minimum role (PLAN 4.8):
//! moderators get moderation, mail and the read-only views; admins get everything. Only staff
//! (moderator or admin) may sign in to the panel at all. The moderation verbs are checked again
//! by `account_store::can_run_realm_command` inside `crate::moderation`, the chat commands' gate.
use crate::account_store::AccountRole;

/// Every action the panel offers, with the least role that may run it (R). An action not in
/// this list is admin only, like an unknown `realm` command.
pub const ACTIONS: &[(&str, AccountRole)] = &[
    // Read-only views: status, the online list, search, profiles, the audit log, icons, items.
    ("view", AccountRole::Moderator),
    ("mute", AccountRole::Moderator),
    ("unmute", AccountRole::Moderator),
    ("kick", AccountRole::Moderator),
    ("ban", AccountRole::Moderator),
    ("unban", AccountRole::Moderator),
    ("warn", AccountRole::Moderator),
    ("jail", AccountRole::Moderator),
    ("release", AccountRole::Moderator),
    ("note", AccountRole::Moderator),
    ("mail", AccountRole::Moderator),
    ("announce", AccountRole::Admin),
    ("inventory", AccountRole::Admin),
    ("coins", AccountRole::Admin),
    ("quests", AccountRole::Admin),
    ("move", AccountRole::Admin),
    ("powers", AccountRole::Admin),
];

/// Staff roles: the only ones the panel signs in.
pub fn is_staff(role: AccountRole) -> bool {
    matches!(role, AccountRole::Admin | AccountRole::Moderator)
}

/// Whether `role` may run `action`.
pub fn allowed(role: AccountRole, action: &str) -> bool {
    let least = ACTIONS
        .iter()
        .find(|(name, _)| *name == action)
        .map_or(AccountRole::Admin, |(_, least)| *least);
    match role {
        AccountRole::Admin => true,
        AccountRole::Moderator => least == AccountRole::Moderator,
        _ => false,
    }
}

/// The actions `role` may run, for `GET /admin/api/me`.
pub fn permissions(role: AccountRole) -> Vec<&'static str> {
    ACTIONS
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| allowed(role, name))
        .collect()
}

/// The role's stored name (`accounts.json`, `realm role`).
pub fn name(role: AccountRole) -> &'static str {
    match role {
        AccountRole::Player => "player",
        AccountRole::Builder => "builder",
        AccountRole::Moderator => "moderator",
        AccountRole::Admin => "admin",
        AccountRole::Tester => "tester",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moderators_get_moderation_mail_and_views_admins_everything() {
        let moderator = permissions(AccountRole::Moderator);
        for action in ["view", "mute", "kick", "ban", "warn", "note", "mail"] {
            assert!(moderator.contains(&action), "{action}");
        }
        for action in ["announce", "inventory", "coins", "quests", "powers", "unknown"] {
            assert!(!allowed(AccountRole::Moderator, action), "{action}");
            assert!(allowed(AccountRole::Admin, action), "{action}");
        }
        assert_eq!(permissions(AccountRole::Admin).len(), ACTIONS.len());
        for role in [AccountRole::Player, AccountRole::Builder, AccountRole::Tester] {
            assert!(!is_staff(role) && permissions(role).is_empty());
        }
        // The moderation verbs agree with the chat commands' gate.
        for verb in ["mute", "unmute", "kick", "ban", "unban"] {
            assert!(crate::account_store::can_run_realm_command(
                AccountRole::Moderator,
                verb
            ));
        }
    }
}
