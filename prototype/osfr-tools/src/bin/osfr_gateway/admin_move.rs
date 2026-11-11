//! L043: carry out a committed admin `realm move`. Gameplay already refused every other role
//! and resolved the open-world arrival; this finds the character and moves it. Online, through
//! its own session (the 122 rescue's relocation, then that session's commit and save); offline,
//! by rewriting only the saved pose while no session owns the character. A refusal changes
//! nothing and is told to the admin alone. Every move writes one security-log row of numbers.
use super::*;
use osfr_tools::gameplay::{AdminMove, MoveTarget};
use osfr_tools::security_log::AdminMoveRow;

const OPEN_WORLD_ZONE: u32 = 0;

pub(super) fn run(
    admin: &mut Session,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    request: AdminMove,
) -> Result<(), Box<dyn std::error::Error>> {
    let by = Mover::of(admin);
    let target = request.target.clone();
    let moved = move_character(by, options, sessions, world, membership, now_ms, request);
    if let (Ok(reply), Some(store)) = (&moved, options.account_store.as_deref()) {
        let guid = match target {
            MoveTarget::Guid(guid) => Some(guid),
            MoveTarget::Name(name) => AccountStore::load(store)
                .ok()
                .and_then(|store| store.character_by_display_name(&name)),
        };
        admin_http::audit_chat(options, admin, "move", guid, json!({ "result": reply }));
    }
    let reply = moved.unwrap_or_else(|refusal| format!("Cannot move: {refusal}. Nothing changed."));
    admin
        .sender
        .queue_message(&wrap_application(&osfr_tools::chat::system(&reply)))?;
    Ok(())
}

/// Who asks for a move: the staff account, its session and its own character (both 0 for the
/// web panel's staff member, who may have no character in the world).
#[derive(Clone, Copy)]
pub(super) struct Mover {
    pub(super) account: u64,
    pub(super) session: u64,
    pub(super) character: u64,
}

impl Mover {
    pub(super) fn of(session: &Session) -> Self {
        Self {
            account: session.account_guid,
            session: session.id.0,
            character: session.character_guid,
        }
    }
}

pub(super) fn move_character(
    admin: Mover,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    request: AdminMove,
) -> Result<String, String> {
    let store = options
        .account_store
        .as_ref()
        .ok_or("this server has no account store")?;
    // (R) Fail closed: a move that cannot be audited is not made.
    if world
        .security
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .log
        .is_none()
    {
        return Err("this server has no security log (--security-log)".into());
    }
    let store = AccountStore::load(store).map_err(|_| "the account store is unreadable")?;
    let guid = match &request.target {
        MoveTarget::Guid(guid) => store.character(*guid).map(|_| *guid),
        MoveTarget::Name(name) => store.character_by_display_name(name),
    }
    .ok_or("no single character has that name or GUID")?;
    if guid == admin.character {
        return Err("use realm goto to move yourself".into());
    }
    let owner = store
        .character(guid)
        .map(|record| record.owner_account_id)
        .ok_or("no single character has that name or GUID")?;
    let online = owning_session(sessions, owner, guid);
    let (from, to) = match online {
        Some(key) => move_online(key, guid, &request, sessions, world, membership, now_ms)?,
        None => osfr_tools::game_store::replace_pose(
            &character_state_path(&options.state, guid),
            OPEN_WORLD_ZONE,
            request.position,
            request.rotation,
        )
        .map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => "that character has never entered the world",
            io::ErrorKind::InvalidInput => "that character was saved outside the open world",
            _ => "that character's save could not be rewritten",
        })?,
    };
    let row = AdminMoveRow {
        account_id: admin.account,
        session_id: admin.session,
        target_guid: guid,
        online: online.is_some(),
        from: (from.zone_id, from.position),
        to: (to.zone_id, to.position),
    };
    let audited = world
        .security
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .log
        .as_mut()
        .map(|log| log.record_admin_move(row));
    let [x, y, z] = to.position;
    let mut reply = format!("Moved character {guid} to x {x:.1}, y {y:.2}, z {z:.1}.");
    if !matches!(audited, Some(Ok(()))) {
        eprintln!("security log: admin move of {guid} was not recorded");
        reply.push_str(" The security log row was NOT written.");
    }
    Ok(reply)
}

/// Stage the move on a clone of the owning session's gateway and commit it through that
/// session. The session goes back into the map whatever happens.
fn move_online(
    key: SessionKey,
    guid: u64,
    request: &AdminMove,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
) -> Result<(CharacterState, CharacterState), String> {
    let mut target = sessions
        .remove(&key)
        .ok_or("that character's session is gone")?;
    let result = if target.character_guid != guid {
        Err("that account is online with another character".to_owned())
    } else {
        let (position, rotation) = (request.position, request.rotation);
        relocate(
            &mut target,
            position,
            rotation,
            sessions,
            world,
            membership,
            now_ms,
        )
    };
    sessions.insert(key, target);
    result
}

/// Stage a relocation on a clone of `session`'s gateway and commit it through that session.
/// L081: the admin's own GM wheel go-to uses it too (the dispatching session is not in `sessions`).
pub(super) fn relocate(
    session: &mut Session,
    position: [f32; 3],
    rotation: Option<[f32; 3]>,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
) -> Result<(CharacterState, CharacterState), String> {
    let before = session.gateway.state().clone();
    let mut staged = session.gateway.clone();
    let outcome = staged
        .stage_admin_relocation(position, rotation, now_ms)
        .map_err(|error| error.to_string())?;
    let arrived = staged.state().clone();
    let committed = session.commit_outcome(
        staged,
        before.clone(),
        outcome,
        world,
        membership,
        sessions,
        now_ms,
    );
    match committed {
        Ok(true) => Ok((before, arrived)),
        // A queue error after the install cannot undo the move or its save.
        Err(_) if session.gateway.state() == &arrived => Ok((before, arrived)),
        _ => Err("the move was not saved".to_owned()),
    }
}
