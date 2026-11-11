//! gm-panel G2: the host half of mail. Once a world tick it starts the staff sends sessions
//! staged with a committed message (`Gateway::take_mail_sends`), tells each player who has just
//! entered the world how many letters wait, and tells an online recipient as a job's letter
//! lands. The tick reads at most one small mailbox file per login and never writes one: the
//! jobs write on their own threads (`osfr_tools::mail::admin`), and no character save is
//! touched by mail.
use super::*;
use osfr_tools::account_store::{AccountRole, can_run_realm_command};
use osfr_tools::chat;
use osfr_tools::mail::{
    self,
    admin::{self, OnlineCharacter, Staff, To},
    chat::{StagedSend, Target},
};

pub(super) fn tick(sessions: &mut SessionMap, options: &Options) {
    let Some(store) = options.account_store.as_deref() else {
        return;
    };
    start_staged(sessions, store);
    notify(sessions, store);
}

/// Every selected session, as `mail::admin::send` filters `online` and `zone` sends.
fn roster(sessions: &SessionMap) -> Vec<OnlineCharacter> {
    sessions
        .values()
        .filter(|session| session.selected_character)
        .map(|session| OnlineCharacter {
            guid: session.character_guid,
            zone: session.gateway.state().zone_id,
            role: session.gateway.role(),
        })
        .collect()
}

fn tell(session: &mut Session, inner: &[u8]) {
    if let Err(error) = session.sender.queue_message(&wrap_application(inner)) {
        eprintln!("mail: session {}: {error}", session.id.0);
    }
}

fn start_staged(sessions: &mut SessionMap, store: &Path) {
    let staged: Vec<_> = sessions
        .iter_mut()
        .filter_map(|(key, session)| {
            let sends = session.gateway.take_mail_sends();
            (!sends.is_empty()).then(|| {
                let sender = (session.account_guid, session.character_guid);
                (*key, sender, session.gateway.role(), sends)
            })
        })
        .collect();
    if staged.is_empty() {
        return;
    }
    let online = roster(sessions);
    for (key, sender, role, sends) in staged {
        for send in sends {
            let reply = start(store, sender, role, send, &online);
            if let Some(session) = sessions.get_mut(&key) {
                tell(session, &chat::system(&reply));
            }
        }
    }
}

/// One staged send: the role re-checked from the committed session, names resolved in the
/// account store (one character per name, as `realm move` reads them), then the facade.
fn start(
    store: &Path,
    (account, guid): (u64, u64),
    role: AccountRole,
    send: StagedSend,
    online: &[OnlineCharacter],
) -> String {
    if !can_run_realm_command(role, "mail") {
        return "Cannot do that: your account role does not allow this command.".into();
    }
    // Attachments make items and coins, which only admins may write (the panel's rule too).
    if !send.letter.attachments.is_empty() && role != AccountRole::Admin {
        return "Cannot send mail: only admins may attach items or coins.".into();
    }
    let Ok(accounts) = AccountStore::load(store) else {
        return "Cannot send mail: the account store is unavailable.".into();
    };
    let to = match send.to {
        Target::Names(names) => {
            let mut guids = Vec::with_capacity(names.len());
            for name in &names {
                let Some(guid) = accounts.character_by_display_name(name) else {
                    return format!("Cannot send mail: no single character is named {name}.");
                };
                guids.push(guid);
            }
            To::Characters(guids)
        }
        Target::Everyone => To::Everyone,
        Target::Online => To::Online,
        Target::Role(role) => To::Role(role),
        Target::Zone(zone) => To::Zone(zone),
    };
    let name = (accounts.character(guid))
        .map_or_else(|| "Free Realms".to_owned(), |r| r.identity.display_name());
    let staff = Staff {
        account_id: account,
        name: name.clone(),
    };
    let letter = mail::Letter {
        from: name,
        ..send.letter
    };
    // The sender's own session checked each attached item against its tables when it staged.
    match admin::send(store, &staff, "chat", &to, letter, online, &|_, _| true) {
        Ok(receipt) => format!(
            "Mail queued for {} character{} (job {}).",
            receipt.recipients,
            if receipt.recipients == 1 { "" } else { "s" },
            receipt.job_id
        ),
        Err(error) => format!("Cannot send mail: {error}."),
    }
}

/// The notice: once per login when the world has loaded, and for each letter a job puts in an
/// online mailbox (the job's own unread count, so no file is read for it).
fn notify(sessions: &mut SessionMap, store: &Path) {
    // The last count per character wins: it is the newest.
    let arrived: BTreeMap<u64, usize> = admin::take_delivered(store).into_iter().collect();
    for session in sessions.values_mut() {
        if !session.selected_character || !session.gateway.is_world_loaded() {
            continue;
        }
        let guid = session.character_guid;
        let unread = if session.gateway.take_mail_notice_due() {
            Some(mail::unread(&mail::mailbox_path(store, guid)))
        } else {
            arrived.get(&guid).copied()
        };
        if let Some(unread) = unread.filter(|unread| *unread > 0) {
            tell(session, &mail::notice(unread));
        }
    }
}

#[cfg(test)]
#[path = "mail_routing_tests.rs"]
mod tests;
