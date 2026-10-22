//! L042: the host half of moderation. The gateway stages a role-gated `realm mute|unmute|kick|
//! ban|unban`; once the moderator's transaction commits, this runs it against the account
//! store, answers the moderator, writes one audit row (ids only) and, for a kick or ban, ends
//! the target's bound session. A mute is read back by the target's own session
//! (`Gateway::chat_muted`), so nothing here touches another player's gateway.
use super::*;
use osfr_tools::chat;
use osfr_tools::moderation::{self as rules, Action, Command};

/// Run each staged command for this session's account. The actor is the verified binding
/// from `select_character_from_login`, never a packet field.
pub(super) fn run_staged(
    session: &mut Session,
    sessions: &mut SessionMap,
    world: &SharedWorld,
    options: &Options,
    staged: Vec<Command>,
) -> Result<(), Box<dyn std::error::Error>> {
    for command in staged {
        let result = match &options.account_store {
            Some(path) if session.selected_character => {
                rules::run(path, session.account_guid, &command, rules::unix_now())
            }
            _ => Err("Cannot do that: a verified account binding is required."),
        };
        let reply = match result {
            Ok(applied) => {
                let ended = matches!(applied.action, Action::Kick | Action::Ban)
                    && end_bound_sessions(sessions, applied.target_account_id);
                if applied.action == Action::Kick && !ended {
                    "Cannot do that: that player is not online."
                } else {
                    audit(world, session.account_guid, applied);
                    applied.action.done()
                }
            }
            Err(refusal) => refusal,
        };
        session
            .sender
            .queue_message(&wrap_application(&chat::system(reply)))?;
    }
    Ok(())
}

/// Character binding refuses a banned account until the ban lapses.
pub(super) fn refuse_banned(store: &AccountStore, account: u64) -> Result<(), &'static str> {
    if store.sanctions(account).banned(rules::unix_now()) {
        return Err("account is banned");
    }
    Ok(())
}

/// Mark the target's bound sessions; `Server::step` ends them through `end_session`, which
/// saves the character. Only a selected session carries a verified account id, so a
/// handshake session's placeholder id never matches.
fn end_bound_sessions(sessions: &mut SessionMap, account: u64) -> bool {
    let mut ended = false;
    for session in sessions.values_mut() {
        if session.selected_character && session.account_guid == account {
            session.kicked = true;
            ended = true;
        }
    }
    ended
}

/// One row per applied action. Like every L9 row it is best effort: a log failure is
/// reported and never undoes the action or drops the moderator.
fn audit(world: &SharedWorld, actor: u64, applied: rules::Applied) {
    let mut security = world
        .security
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some(log) = &mut security.log
        && let Err(error) = log.record_moderation(actor, applied)
    {
        eprintln!("security log: {error}");
    }
}

/// Typed (15/1) and quick (67/2, 67/3) chat, every channel and tells included, and the guild
/// MOTD (129/11) from a muted account: one refusal line to the sender; nothing is relayed,
/// recorded or stored.
pub(super) fn muted_chat(session: &Session, inner: &[u8]) -> Option<Vec<Delivery>> {
    let is_chat = [[15, 0, 1, 0], [67, 0, 2, 0], [67, 0, 3, 0], [129, 0, 11, 0]]
        .iter()
        .any(|prefix| inner.starts_with(prefix));
    (is_chat && session.gateway.chat_muted()).then(|| {
        vec![Delivery {
            recipient: session.id,
            bytes: wrap_application(&chat::system(
                "Cannot do that: your account cannot chat right now.",
            )),
        }]
    })
}

#[cfg(test)]
#[path = "l042_moderation.rs"]
mod tests;
