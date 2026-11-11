//! G3 (gm-panel): the gateway host of `osfr_tools::character_admin`, the one facade G1's admin
//! routes and the `realm give|take|coins <First Last> ...` chat words call. It finds the
//! character and runs the edit where the character lives:
//!
//! - **online**: on a clone of the character's own session, committed through that session's
//!   `commit_outcome` (its client packets, the players near it, its save), as the L081 gift is;
//! - **offline**: the save is loaded through the login path (`load_gateway_from`), edited by the
//!   same gameplay code, and written whole with the atomic `game_store::save` after a
//!   timestamped copy of the old file (`characters/admin-backups/<guid>-<unix ms>.json`).
//!
//! One writer per save: a [`Host`] holds the server's session map, so it exists only on the
//! main loop, where logins are handled too. A login loads the save inside
//! `select_character_from_login` and refuses a second owner (`owning_session`), so between this
//! module's ownership check and its write no session can load the old save; a save that still
//! changes under the write (a pose recovered at load) is refused, not overwritten.
use super::*;
use osfr_tools::account_store::AccountRole;
use osfr_tools::character_admin::{
    self as admin, Actor, AdminError, Change, CharacterView, CoinOp, Edit, EditRecord,
    InventoryView, ItemPage, JobSet, QuestOp, Receipt, Removal,
};
use osfr_tools::gameplay::Outcome;

/// What every call needs from the server loop; build one per request on the main loop.
pub(crate) struct Host<'a> {
    pub options: &'a Options,
    pub sessions: &'a mut SessionMap,
    pub world: &'a mut SharedWorld,
    pub membership: &'a mut World,
    pub now_ms: u64,
    /// The session whose message is being handled (the chat words). It is out of `sessions`
    /// meanwhile, so an edit of the sender's own character goes through it.
    pub me: Option<&'a mut Session>,
}

fn owns(target: u64) -> impl Fn(&&mut Session) -> bool {
    move |session| session.selected_character && session.character_guid == target
}

fn online_key(sessions: &SessionMap, target: u64) -> Option<SessionKey> {
    sessions
        .iter()
        .find(|(_, s)| s.selected_character && s.character_guid == target)
        .map(|(key, _)| *key)
}

fn failed(text: impl Into<String>) -> AdminError {
    AdminError::Failed(text.into())
}

/// The bag and the worn items (moderators and admins).
pub(crate) fn inventory(
    host: &Host,
    actor: &Actor,
    target: u64,
) -> Result<InventoryView, AdminError> {
    read(host, actor, target, Gameplay::admin_inventory)
}

/// Coins, jobs and levels, quests, mounts, pets and collections (moderators and admins).
pub(crate) fn profile(
    host: &Host,
    actor: &Actor,
    target: u64,
) -> Result<CharacterView, AdminError> {
    read(host, actor, target, Gameplay::admin_profile)
}

/// Item definitions by name substring or id, a page at a time (moderators and admins).
#[allow(dead_code)] // the panel's `/items` searches on its HTTP thread (`admin_wire::items`)
pub(crate) fn search_items(
    host: &Host,
    actor: &Actor,
    query: &str,
    page: usize,
) -> Result<ItemPage, AdminError> {
    if !actor.may_read() {
        return Err(AdminError::Forbidden);
    }
    let content = host
        .options
        .content
        .as_deref()
        .ok_or_else(|| failed("this server has no content"))?;
    admin::search_items(content, query, page)
        .map_err(|error| AdminError::Refused(error.to_string()))
}

pub(crate) fn add_items(
    host: &mut Host,
    actor: &Actor,
    target: u64,
    item_id: i32,
    count: u32,
    equip: bool,
) -> Result<Receipt, AdminError> {
    edit(
        host,
        actor,
        target,
        Edit::AddItems {
            item_id,
            count,
            equip,
        },
    )
}

pub(crate) fn remove_items(
    host: &mut Host,
    actor: &Actor,
    target: u64,
    items: Vec<Removal>,
) -> Result<Receipt, AdminError> {
    edit(host, actor, target, Edit::RemoveItems(items))
}

pub(crate) fn coins(
    host: &mut Host,
    actor: &Actor,
    target: u64,
    op: CoinOp,
    amount: u32,
) -> Result<Receipt, AdminError> {
    edit(host, actor, target, Edit::Coins(op, amount))
}

pub(crate) fn quests(
    host: &mut Host,
    actor: &Actor,
    target: u64,
    op: QuestOp,
    quest_id: u32,
) -> Result<Receipt, AdminError> {
    edit(host, actor, target, Edit::Quest(op, quest_id))
}

pub(crate) fn jobs(
    host: &mut Host,
    actor: &Actor,
    target: u64,
    job_id: i32,
    set: JobSet,
) -> Result<Receipt, AdminError> {
    edit(host, actor, target, Edit::Job(job_id, set))
}

/// Reverse an earlier write, refused (changing nothing) when the character moved on since.
pub(crate) fn undo(
    host: &mut Host,
    actor: &Actor,
    record: &EditRecord,
) -> Result<Receipt, AdminError> {
    edit(
        host,
        actor,
        record.target,
        Edit::Apply(record.change.inverse()),
    )
}

fn read<T>(
    host: &Host,
    actor: &Actor,
    target: u64,
    view: fn(&Gameplay) -> T,
) -> Result<T, AdminError> {
    if !actor.may_read() {
        return Err(AdminError::Forbidden);
    }
    let live = host
        .me
        .as_deref()
        .filter(|s| s.selected_character && s.character_guid == target)
        .or_else(|| {
            host.sessions
                .values()
                .find(|s| s.selected_character && s.character_guid == target)
        });
    if let Some(session) = live {
        return session
            .gateway
            .gameplay()
            .map(view)
            .ok_or_else(|| AdminError::Conflict("that player is still loading".into()));
    }
    let gateway = load_offline(host.options, target)?;
    gateway
        .gameplay()
        .map(view)
        .ok_or_else(|| failed("that save has no gameplay"))
}

/// Every write: the role and bounds first, then the character's own gameplay, online or off.
/// An edit that changes nothing is refused before anything is committed or written.
pub(crate) fn edit(
    host: &mut Host,
    actor: &Actor,
    target: u64,
    edit: Edit,
) -> Result<Receipt, AdminError> {
    if !actor.may_write() {
        return Err(AdminError::Forbidden);
    }
    edit.check()?;
    let now_ms = host.now_ms;
    let mut change = Change::default();
    let mut staging = |game: &mut Gameplay| -> Result<Outcome, AdminError> {
        let before = game.state();
        let out = game.admin_edit(&edit, target, now_ms)?;
        change = Change::between(&before, &game.state());
        if change.is_empty() {
            return Err(AdminError::Refused("that changes nothing".into()));
        }
        Ok(out)
    };
    let (online, backup) = if let Some(me) = host.me.as_deref_mut().filter(owns(target)) {
        let parts = (&mut *host.sessions, &mut *host.world, &mut *host.membership);
        commit_online(me, &mut staging, parts, now_ms)?;
        (true, None)
    } else if let Some(key) = online_key(host.sessions, target) {
        let mut other = host
            .sessions
            .remove(&key)
            .ok_or_else(|| failed("that player's session is gone"))?;
        let parts = (&mut *host.sessions, &mut *host.world, &mut *host.membership);
        let result = commit_online(&mut other, &mut staging, parts, now_ms);
        host.sessions.insert(key, other);
        result?;
        (true, None)
    } else {
        (
            false,
            Some(commit_offline(host.options, target, &mut staging)?),
        )
    };
    let record = EditRecord {
        target,
        actor_account: actor.account_id,
        at_unix: osfr_tools::moderation::unix_now(),
        summary: edit.summary(),
        change,
    };
    // Ids and numbers only, like every staff row; G1's audit log keeps the record itself.
    event(serde_json::json!({
        "event": "character_admin",
        "actor_account": actor.account_id,
        "target_guid": target,
        "online": online,
        "summary": record.summary,
    }));
    Ok(Receipt {
        record,
        online,
        backup,
    })
}

type Staging<'s> = dyn FnMut(&mut Gameplay) -> Result<Outcome, AdminError> + 's;

/// Stage on a clone of the session's gateway and commit through that session.
fn commit_online(
    session: &mut Session,
    staging: &mut Staging<'_>,
    (sessions, world, membership): (&mut SessionMap, &mut SharedWorld, &mut World),
    now_ms: u64,
) -> Result<(), AdminError> {
    if !session.gateway.is_world_loaded() {
        return Err(AdminError::Conflict(
            "that player is still loading; try again".into(),
        ));
    }
    let previous = session.gateway.state().clone();
    let mut staged = session.gateway.clone();
    let outcome = staged.stage_admin_edit(|game| staging(game))?;
    match session.commit_outcome(
        staged, previous, outcome, world, membership, sessions, now_ms,
    ) {
        Ok(true) => Ok(()),
        Ok(false) => Err(AdminError::Refused(
            "the player's session refused it".into(),
        )),
        Err(_) => Err(failed("the player's session failed; check the character")),
    }
}

/// Edit a save no session owns: load it as a login would, stage the edit, then (only if the
/// file is still the one loaded) keep a timestamped copy and write the result atomically.
fn commit_offline(
    options: &Options,
    target: u64,
    staging: &mut Staging<'_>,
) -> Result<PathBuf, AdminError> {
    let mut gateway = load_offline(options, target)?;
    let path = character_state_path(&options.state, target);
    let loaded = fs::read(&path).map_err(|_| failed("the save could not be read"))?;
    gateway.stage_admin_edit(|game| staging(game))?;
    if fs::read(&path).ok().as_deref() != Some(loaded.as_slice()) {
        return Err(AdminError::Conflict(
            "the save changed during the edit".into(),
        ));
    }
    let backups = path
        .parent()
        .map(|dir| dir.join("admin-backups"))
        .ok_or_else(|| failed("the save has no folder"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let backup = backups.join(format!("{target}-{stamp}.json"));
    fs::create_dir_all(&backups)
        .and_then(|()| osfr_tools::character_store::replace_bytes(&backup, &loaded))
        .map_err(|_| failed("the backup could not be written; the save is unchanged"))?;
    save(&path, &gateway.snapshot()).map_err(|_| failed("the save was not written"))?;
    Ok(backup)
}

/// A saved character's gateway, built the way `select_character_from_login` builds it (the
/// account store's first pose, the login load, the account's rules and the identity).
fn load_offline(options: &Options, target: u64) -> Result<Gateway, AdminError> {
    let store_path = options
        .account_store
        .as_ref()
        .ok_or_else(|| failed("offline edits need the account store"))?;
    let store = AccountStore::load(store_path).map_err(|_| failed("the account store is busy"))?;
    let record = store
        .character(target)
        .ok_or_else(|| AdminError::NotFound(format!("no character {target}")))?;
    let account = store
        .account(record.owner_account_id)
        .ok_or_else(|| AdminError::NotFound(format!("character {target} has no account")))?;
    let path = character_state_path(&options.state, target);
    if !path.is_file() {
        return Err(AdminError::NotFound(format!(
            "character {target} has never entered the world"
        )));
    }
    let mut initial = match record.world_state.clone() {
        Some(state) => state,
        None => startup_source::initial_state(options).map_err(|e| failed(e.to_string()))?,
    };
    initial.guid = target;
    let mut gateway = load_gateway_from(options, &path, initial)
        .map_err(|error| failed(format!("the save did not load: {error}")))?;
    gateway.set_game_rules(GameRuleSet::new(
        MembershipState::new(account.flags.membership, None),
        RulePolicy::default(),
    ));
    let mut identity = record.identity.clone();
    identity.guid = target;
    gateway.set_identity(identity)?;
    Ok(gateway)
}

/// The staff audit log's action name for an edit.
pub(crate) fn action(edit: &Edit) -> &'static str {
    match edit {
        Edit::AddItems { .. } => "inventory_add",
        Edit::RemoveItems(_) => "inventory_remove",
        Edit::Coins(..) => "coins",
        Edit::Quest(..) => "quests",
        Edit::Job(..) => "jobs",
        Edit::Apply(_) => "undo",
    }
}

/// (R) An edit row larger than this keeps its summary but not its change (no undo): the audit
/// log refuses rows past 16 KiB.
const ROW_BUDGET: usize = 14 * 1024;

/// The staff audit row of one applied edit. `before` and `after` say in short what changed
/// (coins, item stacks by item and count, worn rows, quest states, job XP); `after` also carries
/// the whole change and the key `POST /admin/api/undo` takes to reverse it, when they fit.
pub(crate) fn audit_row(
    staff: osfr_tools::admin::Staff,
    via: &str,
    target: osfr_tools::admin::audit::Target,
    edit: &Edit,
    receipt: &Receipt,
) -> osfr_tools::admin::audit::Row {
    let change = &receipt.record.change;
    let (mut before, mut after) = (serde_json::Map::new(), serde_json::Map::new());
    let mut side = |key: &str, old: serde_json::Value, new: serde_json::Value| {
        before.insert(key.into(), old);
        after.insert(key.into(), new);
    };
    if let Some([old, new]) = change.coins {
        side("coins", old.into(), new.into());
    }
    let stacks =
        |pick: fn(&admin::StackChange) -> &Option<osfr_tools::inventory::InventoryItem>| {
            (change.stacks.iter())
                .filter_map(|stack| pick(stack).as_ref())
                .map(|item| serde_json::json!({"item_id": item.definition_id, "count": item.count}))
                .collect::<Vec<_>>()
        };
    if !change.stacks.is_empty() {
        side(
            "items",
            stacks(|s| &s.before).into(),
            stacks(|s| &s.after).into(),
        );
    }
    if !change.equipment.is_empty() {
        let worn = |at: usize| {
            (change.equipment.iter())
                .map(
                    |rows| serde_json::json!({"slot": rows[at].slot, "instance": rows[at].item_id}),
                )
                .collect::<Vec<_>>()
        };
        side("worn", worn(0).into(), worn(1).into());
    }
    if !change.quests.is_empty() {
        let quests =
            |pick: fn(&admin::QuestChange) -> &Option<osfr_tools::quest_engine::QuestProgress>| {
                (change.quests.iter())
                    .map(|quest| {
                        let status = pick(quest).as_ref().map(|progress| progress.status.clone());
                        (quest.quest_id.to_string(), serde_json::json!(status))
                    })
                    .collect::<serde_json::Map<String, serde_json::Value>>()
            };
        side(
            "quests",
            quests(|q| &q.before).into(),
            quests(|q| &q.after).into(),
        );
    }
    if !change.jobs.is_empty() {
        let jobs = |old: bool| {
            (change.jobs.iter())
                .map(|job| {
                    let xp = if old { job.before } else { job.after };
                    (job.job_id.to_string(), serde_json::Value::from(xp))
                })
                .collect::<serde_json::Map<String, serde_json::Value>>()
        };
        side("job_xp", jobs(true).into(), jobs(false).into());
    }
    if let Some([old, new]) = &change.learned {
        side(
            "jobs_learned",
            serde_json::json!(old),
            serde_json::json!(new),
        );
    }
    if let Some([old, new]) = change.active_job {
        side("active_job", old.into(), new.into());
    }
    after.insert("summary".into(), receipt.record.summary.clone().into());
    after.insert("online".into(), receipt.online.into());
    let mut row = osfr_tools::admin::audit::Row {
        time: receipt.record.at_unix,
        staff,
        action: action(edit).to_owned(),
        target,
        before: before.into(),
        after: after.clone().into(),
        reason: None,
        via: via.to_owned(),
    };
    let mut key = [0u8; 16];
    if getrandom::getrandom(&mut key).is_ok() {
        let mut undoable = after;
        undoable.insert("change".into(), serde_json::json!(change));
        undoable.insert("undo".into(), admin_http::hex(&key).into());
        let full = osfr_tools::admin::audit::Row {
            after: undoable.into(),
            ..row.clone()
        };
        if serde_json::to_vec(&full).is_ok_and(|bytes| bytes.len() <= ROW_BUDGET) {
            row = full;
        }
    }
    row
}

/// After an admin's message commits: run its `realm give|take|coins <First Last> ...` words.
/// `Gateway::staff_command` already refused every other role and answered usage errors.
pub(super) fn run_chat(
    session: &mut Session,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    message: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    if session.gateway.role() != AccountRole::Admin {
        return Ok(());
    }
    let Ok(children) = session.gateway.application_requests(message) else {
        return Ok(());
    };
    for child in children {
        let Some(text) = osfr_tools::chat::request(&child, session.character_guid)
            .ok()
            .flatten()
        else {
            continue;
        };
        let lower = text.trim().to_ascii_lowercase();
        let Some(command) = lower.strip_prefix("realm ") else {
            continue;
        };
        let first = command.split_whitespace().next().unwrap_or_default();
        let allowed = session.gateway.gameplay().is_some_and(|g| g.may_run(first));
        let Some(Ok(parsed)) = admin::parse_chat(command).filter(|_| allowed) else {
            continue;
        };
        let actor = Actor {
            account_id: session.account_guid,
            role: AccountRole::Admin,
        };
        let target = options
            .account_store
            .as_ref()
            .and_then(|path| AccountStore::load(path).ok())
            .and_then(|store| store.character_by_display_name(&parsed.name));
        let reply = match target {
            None => "Cannot do that: no single character has that name. Nothing changed.".into(),
            Some(guid) => {
                let mut host = Host {
                    options,
                    sessions: &mut *sessions,
                    world: &mut *world,
                    membership: &mut *membership,
                    now_ms,
                    me: Some(&mut *session),
                };
                let done = edit(&mut host, &actor, guid, parsed.edit.clone());
                if let (Ok(receipt), Some(store)) = (&done, options.account_store.as_deref()) {
                    let target = admin_http::audit_target(store, Some(guid));
                    let staff = admin_http::chat_staff(session);
                    let row = audit_row(staff, "chat", target, &parsed.edit, receipt);
                    osfr_tools::admin::audit::record_beside(store, &row);
                }
                match done {
                    Ok(receipt) => format!(
                        "Done for character {guid}: {}{}.",
                        receipt.record.summary,
                        if receipt.online {
                            " (online)"
                        } else {
                            " (saved; the old save is kept as a backup)"
                        }
                    ),
                    Err(error) => format!("Cannot do that: {error}. Nothing changed."),
                }
            }
        };
        session
            .sender
            .queue_message(&wrap_application(&osfr_tools::chat::system(&reply)))?;
    }
    Ok(())
}
#[cfg(test)]
#[path = "g3_character_admin.rs"]
mod tests;
