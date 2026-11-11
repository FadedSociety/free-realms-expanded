//! G5: carry out the staff powers (`osfr_tools::gm_powers`) on live sessions. Chat reaches
//! `run` through `chat` (the `realm` words, gated by `Gameplay::may_run`, which is the role
//! table `can_run_realm_command`); an admin HTTP route calls `run` with a power that one of the
//! `gm_powers` facade functions authorized. `tick` keeps each watcher on its target every
//! server tick and puts back a freeze, flight or hidden presence a zone change replaced; `end`
//! undoes a session's powers before its logout save. Every use writes one row to the security
//! log when one is open (numbers only) and keeps it in `GmPowers::audit`; an applied use also
//! writes one row to the staff audit log the panel reads (`admin::audit`). A refusal changes
//! nothing and is told to the staff member alone. The web panel calls `run` through the staff
//! member's own session, or `run_without_character` for the powers that act on another player
//! alone; `jail` is the panel's moderation hold.
use super::*;
use admin_move::Mover;
use osfr_tools::account_store::AccountRole;
use osfr_tools::admin::{Staff, audit};
use osfr_tools::chat;
use osfr_tools::encounter::ZoneTransition;
use osfr_tools::gameplay::{AdminMove, MoveTarget};
use osfr_tools::gm_powers::waypoints::{Waypoint, Waypoints};
use osfr_tools::gm_powers::{self, AuditRow, Power, Return, Watch, WaypointOp, Who};
use osfr_tools::movement::StaffMotion;

/// (R) A watcher already this close to its lock pose is not moved again.
const LOCK_EPSILON_M: f32 = 0.05;
const OPEN_WORLD_ZONE: u32 = 0;
const WAYPOINT_FILE: &str = "gm-waypoints.json";
/// The refusal for a panel power that moves, hides or answers the staff member's own character.
pub(super) const NEEDS_CHARACTER: &str =
    "that power works on your own character: sign in to the game with this account first";

/// Who uses a power, for its audit rows: the staff account, its character in the world (0 for a
/// panel call without one), the name the staff log shows and the front end (`chat`, `panel`).
pub(super) struct Actor {
    pub(super) account: u64,
    pub(super) character: u64,
    pub(super) staff: Staff,
    pub(super) via: &'static str,
}

impl Actor {
    fn chat(session: &Session) -> Self {
        Self {
            account: session.account_guid,
            character: session.character_guid,
            staff: admin_http::chat_staff(session),
            via: "chat",
        }
    }
}

/// The `realm` pre-route, before gameplay stages the message: count and log the developer
/// commands this role may not run (moved here from `dispatch_message_inner`), then run a GM
/// power typed on its own. `true`: handled, nothing else runs. `false`: gameplay takes the
/// message, a refused word included, so its refusal line stays the existing one.
pub(super) fn chat(
    session: &mut Session,
    message: &[u8],
    options: &Options,
    now_ms: u64,
    world: &mut SharedWorld,
    membership: &mut World,
    sessions: &mut SessionMap,
) -> Result<bool, Box<dyn std::error::Error>> {
    for _ in 0..session.gateway.denied_dev_commands(message)? {
        world
            .security
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .record(
                EventKind::DevCommandDenial,
                options
                    .account_store
                    .as_ref()
                    .and(session.selected_character.then_some(session.account_guid)),
                Some(session.id.0),
            );
    }
    let requests = session.gateway.application_requests(message)?;
    let [inner] = requests.as_slice() else {
        return Ok(false);
    };
    let Some(text) = chat::request(inner, session.character_guid).ok().flatten() else {
        return Ok(false);
    };
    let Some((key, parsed)) = gm_powers::parse(&text) else {
        return Ok(false);
    };
    let allowed = session
        .gateway
        .gameplay()
        .is_some_and(|game| game.may_run(key));
    // A word bundled with other messages goes on to gameplay with them.
    let alone = session.gateway.ordered_messages(message)?.len() == 1;
    if !allowed || !alone || !session.gateway.is_world_loaded() {
        return Ok(false);
    }
    let power = match parsed {
        Ok(power) => power,
        Err(usage) => {
            session
                .sender
                .queue_message(&wrap_application(&chat::system(usage)))?;
            return Ok(true);
        }
    };
    // `realm goto <name>` that names no online player is still the developer teleport.
    if let Power::Goto(who) = &power
        && online(world, session.character_guid, who).is_err()
    {
        return Ok(false);
    }
    let actor = Actor::chat(session);
    let _told = run(
        session, options, sessions, world, membership, now_ms, power, &actor,
    )?;
    Ok(true)
}

/// Carry out `power` for `staff`, whose role the caller checked. Answers with one system line,
/// writes the audit rows, and gives back the reply or the refusal.
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    staff: &mut Session,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    power: Power,
    actor: &Actor,
) -> Result<Result<String, String>, Box<dyn std::error::Error>> {
    let action = power.key();
    let target = target_of(world, staff.character_guid, &power);
    let result = apply(staff, options, sessions, world, membership, now_ms, power);
    let reply = match &result {
        Ok(reply) => reply.clone(),
        Err(refusal) => format!("Cannot do that: {refusal}. Nothing changed."),
    };
    audit(world, options, actor, action, target, &result);
    staff
        .sender
        .queue_message(&wrap_application(&chat::system(&reply)))?;
    Ok(result)
}

/// A panel power for a staff member with no character in the world: those that act on another
/// player alone (freeze, unfreeze, send). Every other power refuses with [`NEEDS_CHARACTER`].
#[allow(clippy::too_many_arguments)]
pub(super) fn run_without_character(
    actor: &Actor,
    role: AccountRole,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    power: Power,
) -> Result<String, String> {
    let action = power.key();
    let target = target_of(world, 0, &power);
    let result = match power {
        Power::Freeze(who, on) => freeze(role, 0, sessions, world, &who, on),
        Power::Send(who, position) => {
            let by = Mover {
                account: actor.account,
                session: 0,
                character: 0,
            };
            let request = AdminMove {
                target: move_target(who),
                position,
                rotation: None,
            };
            admin_move::move_character(by, options, sessions, world, membership, now_ms, request)
        }
        _ => return Err(NEEDS_CHARACTER.into()),
    };
    audit(world, options, actor, action, target, &result);
    result
}

/// The character a power acts on, for its audit rows: the one named (a GUID as given, a name
/// when one online player has it), else the one the staff member watches.
fn target_of(world: &SharedWorld, me: u64, power: &Power) -> Option<u64> {
    match power {
        Power::Spectate(who)
        | Power::Freeze(who, _)
        | Power::Follow(who, _)
        | Power::Bring(who)
        | Power::Goto(who)
        | Power::Send(who, _) => match who {
            Who::Guid(guid) => Some(*guid),
            Who::Name(_) => online(world, me, who).ok(),
        },
        _ => world.gm.watching.get(&me).map(Watch::target),
    }
}

/// The panel's moderation hold: `on` sends `guid` to `spot` (online, or its save as `send`
/// does) and holds it there with the staff freeze, across its relogs until a release or a
/// restart; release lifts the hold where it stands. The caller checked the role and hierarchy
/// and writes the audit row.
#[allow(clippy::too_many_arguments)]
pub(super) fn jail(
    account: u64,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    guid: u64,
    on: bool,
    spot: [f32; 3],
) -> Result<String, String> {
    if world.gm.jailed.contains(&guid) == on {
        return Err(if on {
            "that player is already jailed".into()
        } else {
            "that player is not jailed".into()
        });
    }
    let moved = if on {
        let by = Mover {
            account,
            session: 0,
            character: 0,
        };
        let request = AdminMove {
            target: MoveTarget::Guid(guid),
            position: spot,
            rotation: None,
        };
        admin_move::move_character(by, options, sessions, world, membership, now_ms, request)?
    } else {
        String::new()
    };
    if on {
        world.gm.jailed.insert(guid);
    } else {
        world.gm.jailed.remove(&guid);
    }
    // Online: held (or let go) at once and told; a pending relocation is left to the tick,
    // which holds every jailed character. Offline: held from its next login.
    let held = on || world.gm.frozen.contains(&guid);
    if let Some(other) = session_key(world, guid).and_then(|key| sessions.get_mut(&key)) {
        other
            .gateway
            .set_staff_motion(held.then_some(StaffMotion::Held));
        let notice = if on {
            "A GM has jailed you; you stay here until a GM releases you."
        } else {
            "A GM has released you from jail; you can move again."
        };
        let told = [client_freeze(held), chat::system(notice)]
            .iter()
            .try_for_each(|packet| other.sender.queue_message(&wrap_application(packet)));
        if let Err(error) = told {
            eprintln!("gm powers: jail notice to {guid}: {error}");
        }
    }
    Ok(if on {
        format!("Jailed {}. {moved}", name(world, guid))
    } else {
        format!("Released {}.", name(world, guid))
    })
}

/// What the panel shows of the powers: those of the staff member's own character `me` (none
/// when it is not in the world) and every frozen or jailed player.
pub(super) fn state(world: &SharedWorld, me: Option<u64>) -> serde_json::Value {
    let watch = me.and_then(|guid| world.gm.watching.get(&guid));
    json!({
        "character_id": me,
        "invisible": me.is_some_and(|guid| world.players.is_hidden(guid)),
        "spectating": match watch { Some(Watch::Spectate { target, .. }) => Some(*target), _ => None },
        "following": match watch { Some(Watch::Follow { target, .. }) => Some(*target), _ => None },
        "flying": me.is_some_and(|guid| world.gm.flying.contains(&guid)),
        "frozen": world.gm.frozen,
        "jailed": world.gm.jailed,
    })
}

fn apply(
    staff: &mut Session,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    power: Power,
) -> Result<String, String> {
    let guid = staff.character_guid;
    if !staff.gateway.is_world_loaded() {
        return Err("you are still loading".into());
    }
    let moves_self = matches!(
        power,
        Power::Bring(_) | Power::Goto(_) | Power::Fly(_) | Power::Waypoint(WaypointOp::Go(_))
    );
    if moves_self && world.gm.watching.contains_key(&guid) {
        return Err("stop spectating or following first".into());
    }
    match power {
        Power::Spectate(who) => {
            let target = online(world, staff.character_guid, &who)?;
            watch(staff, sessions, world, membership, now_ms, target, None)?;
            Ok(format!(
                "Spectating {}. Nobody can see you; realm unspectate brings you back.",
                name(world, target)
            ))
        }
        Power::Follow(who, metres) => {
            let target = online(world, staff.character_guid, &who)?;
            watch(
                staff,
                sessions,
                world,
                membership,
                now_ms,
                target,
                Some(metres),
            )?;
            Ok(format!(
                "Following {} at {metres:.0} m; realm unfollow stops.",
                name(world, target)
            ))
        }
        Power::Unspectate | Power::Unfollow => {
            let watching = world.gm.watching.get(&guid);
            let spectating = matches!(watching, Some(Watch::Spectate { .. }));
            let following = matches!(watching, Some(Watch::Follow { .. }));
            if power == Power::Unspectate && !spectating {
                return Err("you are not spectating".into());
            }
            if power == Power::Unfollow && !following {
                return Err("you are not following anyone".into());
            }
            stop(staff, sessions, world, membership, now_ms)?;
            Ok(if spectating {
                "Back where you started.".into()
            } else {
                "You stopped following.".into()
            })
        }
        Power::Invisible(on) => {
            if !on && matches!(world.gm.watching.get(&guid), Some(Watch::Spectate { .. })) {
                return Err("you are spectating; realm unspectate shows you again".into());
            }
            hide(staff, sessions, world, on)?;
            Ok(if on {
                "You are invisible to players; realm visible shows you again.".into()
            } else {
                "You are visible again.".into()
            })
        }
        Power::Freeze(who, on) => freeze(staff.gateway.role(), guid, sessions, world, &who, on),
        Power::Bring(who) => {
            let here = staff.gateway.state();
            if here.zone_id != OPEN_WORLD_ZONE {
                return Err("you are not in the open world".into());
            }
            let request = AdminMove {
                target: move_target(who),
                position: here.position,
                rotation: None,
            };
            admin_move::move_character(
                Mover::of(staff),
                options,
                sessions,
                world,
                membership,
                now_ms,
                request,
            )
        }
        Power::Send(who, position) => {
            let request = AdminMove {
                target: move_target(who),
                position,
                rotation: None,
            };
            admin_move::move_character(
                Mover::of(staff),
                options,
                sessions,
                world,
                membership,
                now_ms,
                request,
            )
        }
        Power::Goto(who) => {
            let target = online(world, guid, &who)?;
            gm_menu_routing::go_to(staff, target, sessions, world, membership, now_ms)
        }
        Power::Fly(on) => {
            let motion = on.then_some(StaffMotion::Flight);
            if !staff.gateway.set_staff_motion(motion) {
                return Err("you are between places; try again".into());
            }
            if on {
                world.gm.flying.insert(guid);
            } else {
                world.gm.flying.remove(&guid);
            }
            Ok(if on {
                "Flight on: the movement checks no longer correct you. The original client cannot fly; this is for the new client.".into()
            } else {
                "Flight off.".into()
            })
        }
        Power::Waypoint(op) => waypoint(staff, options, sessions, world, membership, now_ms, op),
    }
}

/// Each server tick, for one session: keep its watch on its target, and put back a freeze,
/// flight or hidden presence that a zone change or a relocation replaced.
pub(super) fn tick(
    session: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let guid = session.character_guid;
    let loaded = session.gateway.is_world_loaded();
    if loaded
        && world.gm.watching.contains_key(&guid)
        && let Err(reason) = lock(session, sessions, world, membership, now_ms)
    {
        let back = stop(session, sessions, world, membership, now_ms).err();
        let mut line = format!("Your watch ended: {reason}.");
        if let Some(error) = back {
            line.push_str(&format!(" You could not be taken back: {error}."));
        }
        session
            .sender
            .queue_message(&wrap_application(&chat::system(&line)))?;
    }
    let motion = if world.gm.frozen.contains(&guid) || world.gm.jailed.contains(&guid) {
        Some(StaffMotion::Held)
    } else if world.gm.flying.contains(&guid) {
        Some(StaffMotion::Flight)
    } else {
        None
    };
    if loaded && motion.is_some() && session.gateway.staff_motion() != motion {
        // A pending relocation token is left alone; the next tick tries again.
        session.gateway.set_staff_motion(motion);
    }
    let listed =
        |world: &SharedWorld| world.social.is_online(guid) || world.guilds.store().is_online(guid);
    if world.players.is_hidden(guid) && listed(world) {
        for delivery in presence(world, session, false) {
            queue_delivery(session, sessions, world, delivery)?;
        }
    }
    Ok(())
}

/// Before an ending session's save: a spectator is saved where it began, and its watch,
/// freeze, flight and invisibility end with it. Only the session that owns its character.
pub(super) fn end(world: &mut SharedWorld, session: &mut Session) {
    let guid = session.character_guid;
    if world.active_sessions.get(&guid) != Some(&session.id) {
        return;
    }
    if let Some(Watch::Spectate { back, .. }) = world.gm.watching.remove(&guid) {
        let scene = back.scene;
        let restored =
            session
                .gateway
                .restore_saved_pose(scene.zone_id, scene.position, scene.rotation);
        if let Err(error) = restored {
            eprintln!("gm powers: spectator {guid} kept its last pose: {error}");
        }
    }
    world.gm.watching.remove(&guid);
    world.gm.frozen.remove(&guid);
    world.gm.flying.remove(&guid);
    world.players.forget_hidden(guid);
}

/// Start, or move to another target, a watch: spectate (`follow` `None`: hidden, the way
/// back saved once) or follow at `follow` metres. Then the first lock, at once.
fn watch(
    staff: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    target: u64,
    follow: Option<f32>,
) -> Result<(), String> {
    let guid = staff.character_guid;
    if world.gm.watching.contains_key(&target) {
        return Err("that player is spectating or following someone".into());
    }
    let current = world.gm.watching.get(&guid).cloned();
    if current.is_none()
        && let Some(block) = staff
            .gateway
            .gameplay()
            .and_then(|game| game.staff_move_block(staff.gateway.state()))
    {
        return Err(block.into());
    }
    let watch = match (follow, current) {
        (Some(_), Some(Watch::Spectate { .. })) => {
            return Err("you are spectating; realm unspectate first".into());
        }
        (Some(metres), _) => Watch::Follow { target, metres },
        (None, Some(Watch::Spectate { back, .. })) => Watch::Spectate { target, back },
        (None, _) => Watch::Spectate {
            target,
            back: way_back(staff, world, membership)?,
        },
    };
    let spectating = matches!(watch, Watch::Spectate { .. });
    world.gm.flying.remove(&guid);
    world.gm.watching.insert(guid, watch);
    if spectating {
        hide(staff, sessions, world, true)?;
    }
    if let Err(reason) = lock(staff, sessions, world, membership, now_ms) {
        // Nothing moved: undo the watch (and the hiding) before refusing.
        let _ = stop(staff, sessions, world, membership, now_ms);
        return Err(reason);
    }
    Ok(())
}

/// Keep a watcher on its target: follow it into another zone instance, then stand the watch's
/// distance behind it. Nothing moves while the target loads or sits in a minigame scene, or
/// when the watcher is already there. An error ends the watch.
fn lock(
    staff: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
) -> Result<(), String> {
    let guid = staff.character_guid;
    let Some(watch) = world.gm.watching.get(&guid).cloned() else {
        return Ok(());
    };
    let target = watch.target();
    let other = session_key(world, target)
        .and_then(|key| sessions.get(&key))
        .ok_or("your target left the game")?;
    if !other.gateway.is_world_loaded() {
        return Ok(());
    }
    let there = other.gateway.state().clone();
    let followable = other
        .gateway
        .gameplay()
        .is_some_and(|game| game.staff_followable(&there));
    let zone = other.gateway.runtime_zone();
    let pose = gm_powers::behind(there.position, there.rotation, watch.metres());
    let instance = |guid| membership.player_location(guid).map(|row| row.instance_id);
    let (theirs, mine) = (instance(target), instance(guid));
    if there.zone_id != staff.gateway.state().zone_id || theirs != mine {
        let Some(theirs) = theirs.filter(|_| followable) else {
            return Ok(());
        };
        let scene = scene_at(zone.map_err(|e| e.to_string())?, pose, there.rotation)?;
        let to = ZoneInstance::from_world(there.zone_id, theirs).map_err(|e| e.to_string())?;
        return commit(staff, sessions, world, membership, now_ms, |gateway| {
            gateway.stage_instance_transfer(to, scene, now_ms)
        });
    }
    let here = staff.gateway.state();
    let settled = distance(here.position, pose) < LOCK_EPSILON_M
        && here.rotation == there.rotation
        && staff.gateway.staff_motion() == Some(StaffMotion::Driven);
    if settled {
        return Ok(());
    }
    commit(staff, sessions, world, membership, now_ms, |gateway| {
        gateway.stage_staff_pose(pose, there.rotation, Some(StaffMotion::Driven), now_ms)
    })
}

/// End this staff character's watch: a spectator goes back where it began (its zone too)
/// and is shown again unless it was invisible before; a follower stays where it is.
fn stop(
    staff: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
) -> Result<(), String> {
    let Some(watch) = world.gm.watching.remove(&staff.character_guid) else {
        return Ok(());
    };
    let Watch::Spectate { back, .. } = watch else {
        staff.gateway.set_staff_motion(None);
        return Ok(());
    };
    let result = go_back(staff, sessions, world, membership, now_ms, &back);
    if !back.was_hidden {
        hide(staff, sessions, world, false)?;
    }
    result
}

fn go_back(
    staff: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    back: &Return,
) -> Result<(), String> {
    let scene = back.scene.clone();
    let mine = membership
        .player_location(staff.character_guid)
        .map(|row| row.instance_id);
    if staff.gateway.state().zone_id == scene.zone_id && mine == Some(back.instance_id) {
        return commit(staff, sessions, world, membership, now_ms, |gateway| {
            gateway.stage_staff_pose(scene.position, scene.rotation, None, now_ms)
        });
    }
    // The instance it left may have closed meanwhile; the zone's public one stands in.
    let instance = membership
        .instance(back.instance_id)
        .filter(|row| row.zone_id == scene.zone_id)
        .map(|_| back.instance_id)
        .or_else(|| membership.public_instance(scene.zone_id))
        .ok_or("the place you left has closed")?;
    let to = ZoneInstance::from_world(scene.zone_id, instance).map_err(|e| e.to_string())?;
    commit(staff, sessions, world, membership, now_ms, |gateway| {
        gateway.stage_instance_transfer(to, scene, now_ms)
    })
}

/// Where a spectate begins, to come back to.
fn way_back(staff: &Session, world: &SharedWorld, membership: &World) -> Result<Return, String> {
    let here = staff.gateway.state();
    let zone = staff.gateway.runtime_zone().map_err(|e| e.to_string())?;
    Ok(Return {
        instance_id: membership
            .player_location(staff.character_guid)
            .map(|row| row.instance_id)
            .ok_or("you are in no world instance")?,
        scene: scene_at(zone, here.position, here.rotation)?,
        was_hidden: world.players.is_hidden(staff.character_guid),
    })
}

/// A zone's scene with an arrival pose, for an instance transfer (opcode-31 begin zoning). The
/// party venues (`party_routing`) send their guests in and back with it too.
pub(super) fn scene_at(
    zone: osfr_tools::content::Zone,
    position: [f32; 3],
    rotation: [f32; 3],
) -> Result<ZoneTransition, String> {
    Ok(ZoneTransition {
        zone_id: zone.id,
        name: zone.name,
        sky: zone.sky.unwrap_or_default(),
        geometry_id: zone
            .geometry_id
            .and_then(|id| i32::try_from(id).ok())
            .ok_or("that zone has no scene geometry")?,
        position,
        rotation,
        tutorial: zone.tutorial.unwrap_or(false),
    })
}

/// Stage a change on a clone of `session`'s gateway and commit it through that session (pose
/// fanout, membership, save), as `admin_move::relocate` does.
pub(super) fn commit(
    session: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    stage: impl FnOnce(&mut Gateway) -> io::Result<GatewayOutcome>,
) -> Result<(), String> {
    let previous = session.gateway.state().clone();
    let mut staged = session.gateway.clone();
    let outcome = stage(&mut *staged).map_err(|error| error.to_string())?;
    match session.commit_outcome(
        staged, previous, outcome, world, membership, sessions, now_ms,
    ) {
        Ok(true) => Ok(()),
        Ok(false) => Err("the move was refused".into()),
        Err(_) => Err("the move was not saved".into()),
    }
}

/// Hide or show the staff character to every other player: its body in their worlds (the
/// registry's directed views, with its ride and pet) and its presence in friends' and guild
/// lists, which show it offline while it is hidden.
fn hide(
    staff: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    on: bool,
) -> Result<(), String> {
    let seen = world
        .players
        .set_hidden(staff.character_guid, on)
        .into_iter()
        .map(|message| Delivery {
            recipient: message.recipient,
            bytes: message.bytes,
        })
        .collect();
    let mut deliveries = world.with_pets(world.with_rides(seen));
    deliveries.extend(presence(world, staff, !on));
    for delivery in deliveries {
        queue_delivery(staff, sessions, world, delivery).map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// The friends' and guild presence a login or logout sends, without ending the session.
fn presence(world: &mut SharedWorld, session: &Session, online: bool) -> Vec<Delivery> {
    let guid = session.character_guid;
    let messages = if online {
        world.social.login(guid)
    } else {
        world.social.logout(guid)
    };
    let mut deliveries: Vec<Delivery> = messages
        .unwrap_or_default()
        .into_iter()
        .filter_map(|message| world.social_delivery(message))
        .collect();
    deliveries.extend(if online {
        world.guild_login(session)
    } else {
        world.guild_logout(guid)
    });
    deliveries
}

/// Hold `who` where it stands (35/32 to its client too), or release it, for a staff member of
/// `role` whose own character is `me` (0: none). A moderator holds only a lower role; a jailed
/// player is held by its jail and left to `jail`.
fn freeze(
    role: AccountRole,
    me: u64,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    who: &Who,
    on: bool,
) -> Result<String, String> {
    let target = online(world, me, who)?;
    if world.gm.frozen.contains(&target) == on {
        return Err(if on {
            "that player is already frozen".into()
        } else {
            "that player is not frozen".into()
        });
    }
    if world.gm.watching.contains_key(&target) {
        return Err("that player is spectating or following".into());
    }
    if world.gm.jailed.contains(&target) {
        return Err("that player is jailed; release them from jail instead".into());
    }
    let other = session_key(world, target)
        .and_then(|key| sessions.get_mut(&key))
        .ok_or("that player is not online")?;
    if !role.outranks(other.gateway.role()) {
        return Err("you may freeze only a lower role".into());
    }
    if on && !other.gateway.is_world_loaded() {
        return Err("that player is still loading".into());
    }
    if !other
        .gateway
        .set_staff_motion(on.then_some(StaffMotion::Held))
    {
        return Err("that player is between places; try again".into());
    }
    let notice = if on {
        "A GM has frozen you in place."
    } else {
        "A GM has released you; you can move again."
    };
    let sent = [client_freeze(on), chat::system(notice)]
        .iter()
        .try_for_each(|packet| other.sender.queue_message(&wrap_application(packet)));
    if on {
        world.gm.frozen.insert(target);
    } else {
        world.gm.frozen.remove(&target);
    }
    sent.map_err(|error| error.to_string())?;
    Ok(format!(
        "{} {}.",
        if on { "Froze" } else { "Released" },
        name(world, target)
    ))
}

/// The client's own 35/32 `PlayerUpdatePacketFreeze`: header then one bool (case 0x20 of
/// `0092f460`, reader `008f8110`), which sets or clears the motion gate bit 0x10 at actor
/// +0x146 (`008dcaa0`, read by `UpdateMotionAndActorState` `0096e490`).
fn client_freeze(on: bool) -> Vec<u8> {
    vec![35, 0, 32, 0, u8::from(on)]
}

fn waypoint(
    staff: &mut Session,
    options: &Options,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    op: WaypointOp,
) -> Result<String, String> {
    let path = options
        .state
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(WAYPOINT_FILE);
    let mut book = Waypoints::load(&path).map_err(|_| "the waypoint file is unreadable")?;
    let account = staff.account_guid;
    let here = staff.gateway.state().clone();
    let saved = |book: &Waypoints| book.save(&path).map_err(|_| "the waypoints were not saved");
    Ok(match op {
        WaypointOp::Save(label) => {
            let [x, y, z] = here.position;
            let spot = Waypoint {
                label: label.clone(),
                zone_id: here.zone_id,
                position: here.position,
            };
            book.put(account, spot)?;
            saved(&book)?;
            format!(
                "Waypoint {label} saved: zone {}, x {x:.1}, y {y:.2}, z {z:.1}.",
                here.zone_id
            )
        }
        WaypointOp::Delete(label) => {
            if !book.remove(account, &label) {
                return Err(format!("you have no waypoint named {label}"));
            }
            saved(&book)?;
            format!("Waypoint {label} deleted.")
        }
        WaypointOp::List => {
            let spots: Vec<String> = book
                .list(account)
                .iter()
                .map(|spot| {
                    let [x, _, z] = spot.position;
                    format!("{} (zone {}, {x:.0}, {z:.0})", spot.label, spot.zone_id)
                })
                .collect();
            if spots.is_empty() {
                "No waypoints yet; realm waypoint save <label> saves where you stand.".into()
            } else {
                format!("Waypoints: {}.", spots.join("; "))
            }
        }
        WaypointOp::Go(label) => {
            let spot = book
                .find(account, &label)
                .cloned()
                .ok_or_else(|| format!("you have no waypoint named {label}"))?;
            if spot.zone_id != here.zone_id {
                return Err(format!(
                    "waypoint {label} is in zone {}; you are in zone {}",
                    spot.zone_id, here.zone_id
                ));
            }
            if let Some(block) = staff
                .gateway
                .gameplay()
                .and_then(|game| game.staff_move_block(&here))
            {
                return Err(block.into());
            }
            commit(staff, sessions, world, membership, now_ms, |gateway| {
                gateway.stage_staff_pose(spot.position, here.rotation, None, now_ms)
            })?;
            format!("Moved you to waypoint {label}.")
        }
    })
}

/// The audit rows of one power use: the security log (numbers only) and memory every time, and
/// the staff audit log (names and the reply, which the panel's Audit view shows) when applied.
fn audit(
    world: &mut SharedWorld,
    options: &Options,
    actor: &Actor,
    action: &'static str,
    target: Option<u64>,
    result: &Result<String, String>,
) {
    if let (Ok(reply), Some(store)) = (result, options.account_store.as_deref()) {
        let row = audit::Row {
            time: osfr_tools::moderation::unix_now(),
            staff: actor.staff.clone(),
            action: action.to_owned(),
            target: admin_http::audit_target(store, target),
            before: serde_json::Value::Null,
            after: json!({ "result": reply }),
            reason: None,
            via: actor.via.to_owned(),
        };
        audit::record_beside(store, &row);
    }
    let row = AuditRow {
        timestamp_ms: gm_powers::unix_ms(),
        account_id: actor.account,
        character_guid: actor.character,
        action,
        target_guid: target,
        applied: result.is_ok(),
    };
    let written = world
        .security
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .log
        .as_mut()
        .map(|log| log.record_gm_power(&row));
    if let Some(Err(error)) = written {
        eprintln!("security log: gm power row not written: {error}");
    }
    world.gm.record(row);
}

/// The online character `who` names (a GUID, or a displayed name ignoring case), never the
/// caller's own character `me`.
fn online(world: &SharedWorld, me: u64, who: &Who) -> Result<u64, String> {
    let guid = match who {
        Who::Guid(guid) => Some(*guid).filter(|guid| world.active_sessions.contains_key(guid)),
        Who::Name(typed) => {
            let typed = typed.trim().to_lowercase();
            let mut hits = world
                .active_sessions
                .keys()
                .filter(|guid| name_of(world, **guid).is_some_and(|n| n.to_lowercase() == typed));
            match (hits.next(), hits.next()) {
                (Some(guid), None) => Some(*guid),
                _ => None,
            }
        }
    }
    .ok_or("no single online player has that name")?;
    if guid == me {
        return Err("that is you".into());
    }
    Ok(guid)
}

fn name_of(world: &SharedWorld, guid: u64) -> Option<String> {
    let name = &world.identities.get(&guid)?.name;
    Some(format!("{} {}", name.first, name.last).trim().to_owned())
}

fn name(world: &SharedWorld, guid: u64) -> String {
    name_of(world, guid).unwrap_or_else(|| format!("character {guid}"))
}

fn session_key(world: &SharedWorld, guid: u64) -> Option<SessionKey> {
    world
        .active_sessions
        .get(&guid)
        .and_then(|id| world.session_keys.get(id))
        .copied()
}

fn move_target(who: Who) -> MoveTarget {
    match who {
        Who::Guid(guid) => MoveTarget::Guid(guid),
        Who::Name(name) => MoveTarget::Name(name),
    }
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}
