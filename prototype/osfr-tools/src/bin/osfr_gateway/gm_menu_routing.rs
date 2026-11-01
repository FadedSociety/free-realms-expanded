//! L081: carry out an admin's GM wheel choice (`osfr_tools::player_gm_menu`) once the admin's
//! message commits. The wheel only queues; each action re-checks through the path its chat
//! command uses: the moderation rules for mute, kick and ban (role, hierarchy, audit, kick),
//! `admin_move` for summon, the account store's admin check for a role, and the other player's
//! own session for a gift or the admin's own for a go-to. A refusal changes nothing and is told
//! to the admin alone.
use super::*;
use osfr_tools::account_store::{AccountRole, AccountStoreLock, STAFF_LOCK_WAIT};
use osfr_tools::chat;
use osfr_tools::gameplay::{AdminMove, MoveTarget};
use osfr_tools::moderation::Command;
use osfr_tools::player_gm_menu::GmAction;

const OPEN_WORLD_ZONE: u32 = 0;

pub(super) fn run(
    session: &mut Session,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for (target, action, at) in session.radial.take_gm() {
        // Only this message's choices, and only while the chooser is still an admin.
        if at != now_ms || session.gateway.role() != AccountRole::Admin {
            continue;
        }
        let reply = match action {
            GmAction::Mute { .. } | GmAction::Kick | GmAction::Ban { .. } => {
                match moderation_command(options, target, action) {
                    Ok(command) => {
                        // It answers the admin itself.
                        moderation::run_staged(session, sessions, world, options, vec![command])?;
                        continue;
                    }
                    Err(refusal) => refusal.to_owned(),
                }
            }
            GmAction::Summon => {
                let here = session.gateway.state();
                if here.zone_id != OPEN_WORLD_ZONE {
                    "Cannot summon: you are not in the open world. Nothing changed.".to_owned()
                } else {
                    let request = AdminMove {
                        target: MoveTarget::Guid(target),
                        position: here.position,
                        rotation: None,
                    };
                    // It answers the admin itself.
                    admin_move::run(
                        session, options, sessions, world, membership, now_ms, request,
                    )?;
                    continue;
                }
            }
            GmAction::GoTo => go_to(session, target, sessions, world, membership, now_ms)
                .unwrap_or_else(|refusal| format!("Cannot go there: {refusal}. Nothing changed.")),
            GmAction::Coins(n) | GmAction::StationCash(n) => {
                let (coins, sc) = match action {
                    GmAction::Coins(_) => (n, 0),
                    _ => (0, n),
                };
                match gift(target, coins, sc, sessions, world, membership, now_ms) {
                    Ok(()) => "Gift sent.".to_owned(),
                    // Gameplay's refusals are already sentences.
                    Err(r) => format!("Cannot give that: {}.", r.trim_end_matches('.')),
                }
            }
            GmAction::Role(role) => match set_role(options, session.account_guid, target, role) {
                Ok(owner) => audit_role(world, session.account_guid, owner, role),
                Err(r) => format!("Cannot do that: {r}."),
            },
        };
        session
            .sender
            .queue_message(&wrap_application(&chat::system(&reply)))?;
    }
    Ok(())
}

/// The chat command for a wheel sanction, by the target's stored display name (names hold no
/// digits, so a trailing number stays the minutes).
fn moderation_command(
    options: &Options,
    target: u64,
    action: GmAction,
) -> Result<Command, &'static str> {
    let path = options
        .account_store
        .as_ref()
        .ok_or("Cannot do that: a verified account binding is required.")?;
    let store =
        AccountStore::load(path).map_err(|_| "Cannot do that: the account store is busy.")?;
    let name = store
        .character(target)
        .map(|record| record.identity.display_name())
        .ok_or("Cannot do that: no single character has that name.")?;
    Command::parse(&match action {
        GmAction::Mute { minutes } => format!("mute {name} {minutes}"),
        GmAction::Ban {
            minutes: Some(minutes),
        } => format!("ban {name} {minutes}"),
        GmAction::Ban { minutes: None } => format!("ban {name}"),
        _ => format!("kick {name}"),
    })
}

/// The selected session playing `guid`, if any.
fn online(sessions: &SessionMap, guid: u64) -> Option<SessionKey> {
    sessions
        .iter()
        .find(|(_, other)| other.selected_character && other.character_guid == guid)
        .map(|(key, _)| *key)
}

/// Move the admin to the other player, both in the open world.
fn go_to(
    session: &mut Session,
    target: u64,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
) -> Result<String, String> {
    let key = online(sessions, target).ok_or("that player is not online")?;
    let there = sessions[&key].gateway.state();
    if there.zone_id != OPEN_WORLD_ZONE || session.gateway.state().zone_id != OPEN_WORLD_ZONE {
        return Err("go to works only in the open world".into());
    }
    let position = there.position;
    admin_move::relocate(session, position, None, sessions, world, membership, now_ms)?;
    let [x, y, z] = position;
    Ok(format!("Moved you to x {x:.1}, y {y:.2}, z {z:.1}."))
}

/// Stage the gift on a clone of the other player's gateway and commit it through that session,
/// which goes back into the map whatever happens.
fn gift(
    target: u64,
    coins: u32,
    sc: u32,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
) -> Result<(), String> {
    let key = online(sessions, target).ok_or("that player is not online")?;
    let mut other = sessions.remove(&key).ok_or("that player is not online")?;
    let result = (|| {
        let before = other.gateway.state().clone();
        let mut staged = other.gateway.clone();
        let outcome = staged
            .stage_admin_grant(coins, sc)
            .map_err(|error| error.to_string())?;
        match other.commit_outcome(staged, before, outcome, world, membership, sessions, now_ms) {
            Ok(true) => Ok(()),
            Ok(false) => Err("it was refused".to_owned()),
            Err(_) => Err("that player's session failed; check their balance".to_owned()),
        }
    })();
    sessions.insert(key, other);
    result
}

/// L090: the security-log row for a saved role change. Best effort like every L9 row: a log
/// failure is told to the admin and never undoes the change.
fn audit_role(world: &SharedWorld, actor: u64, owner: u64, role: AccountRole) -> String {
    let mut security = world
        .security
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    match security
        .log
        .as_mut()
        .map(|log| log.record_role_change(actor, owner, role))
    {
        Some(Ok(())) | None => "Role saved.".into(),
        Some(Err(error)) => {
            eprintln!("security log: {error}");
            "Role saved. The security log row was NOT written.".into()
        }
    }
}

/// `realm role`'s store change for the account that owns `target`, returning that account; an
/// online target reads it back on its next message (`Gateway::refresh_role`).
fn set_role(
    options: &Options,
    actor: u64,
    target: u64,
    role: AccountRole,
) -> Result<u64, &'static str> {
    let path = options
        .account_store
        .as_ref()
        .ok_or("this server has no account store")?;
    // (R) Cooperate with login/CLI writers; never overwrite a newer key or role.
    let _lock = AccountStoreLock::acquire_within(path, STAFF_LOCK_WAIT)
        .map_err(|_| "the account store is busy")?;
    let mut store = AccountStore::load(path).map_err(|_| "the account store is busy")?;
    let owner = store
        .character(target)
        .map(|record| record.owner_account_id)
        .ok_or("no such character")?;
    if owner == actor {
        return Err("you cannot change your own role");
    }
    store
        .set_account_role(actor, owner, role)
        .map_err(|_| "admin role required")?;
    store.save(path).map_err(|_| "the role was not saved")?;
    Ok(owner)
}
