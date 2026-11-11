//! L042: the host half of moderation. The gateway stages a role-gated `realm mute|unmute|kick|
//! ban|unban`; once the moderator's transaction commits, this runs it against the account
//! store, answers the moderator, writes one audit row (ids only) and, for a kick or ban, ends
//! the target's bound session. A mute is read back by the target's own session
//! (`Gateway::chat_muted`), so nothing here touches another player's gateway.
//!
//! GM panel (G1): chat and the panel share one path. Both run the store change through
//! `rules::run_with_change`, the session half through [`finish`], and write the same
//! `admin::audit` row (`via` chat or panel) beside the account store.
use super::*;
use osfr_tools::admin::{self, audit};
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
        let reply = match &options.account_store {
            Some(path) if session.selected_character => {
                run_chat(session, sessions, world, path, &command)
            }
            _ => "Cannot do that: a verified account binding is required.",
        };
        session
            .sender
            .queue_message(&wrap_application(&chat::system(reply)))?;
    }
    Ok(())
}

/// One chat command: the store change, its session half, then the audit row. Returns the
/// moderator's reply line.
fn run_chat(
    session: &Session,
    sessions: &mut SessionMap,
    world: &SharedWorld,
    path: &Path,
    command: &Command,
) -> &'static str {
    let now = rules::unix_now();
    let actor = session.account_guid;
    let result = rules::run_with_change(path, actor, command, now).and_then(|change| {
        finish(sessions, world, actor, change.0)?;
        Ok(change)
    });
    let change = match result {
        Ok(change) => change,
        Err(refusal) => return refusal,
    };
    let staff = admin_http::chat_staff(session);
    let row = admin::moderation::sanction_row(&staff, command, change, None, None, "chat", now);
    if let Err(error) = audit::record(&audit::dir_for(path), &row) {
        eprintln!("admin audit: {error}");
    }
    change.0.action.done()
}

/// The session half of an applied sanction, for chat and the GM panel alike: a kick or ban
/// ends the target's bound sessions (a kick of nobody online is refused, unrecorded) and the
/// security log gets its row.
pub(super) fn finish(
    sessions: &mut SessionMap,
    world: &SharedWorld,
    actor: u64,
    applied: rules::Applied,
) -> Result<(), &'static str> {
    let ended = matches!(applied.action, Action::Kick | Action::Ban)
        && end_bound_sessions(sessions, applied.target_account_id);
    if applied.action == Action::Kick && !ended {
        return Err("Cannot do that: that player is not online.");
    }
    audit(world, actor, applied);
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
pub(super) mod tests;
