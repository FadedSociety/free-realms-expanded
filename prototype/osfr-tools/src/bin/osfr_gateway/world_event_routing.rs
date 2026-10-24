//! T458: the gateway host's side of world events (`osfr_tools::world_events`). Gameplay stages an
//! admin's `realm event start|stop <key>` (the role gate is `command_refusal`: `event` is
//! admin-only by `account_store::can_run_realm_command`'s default); `apply` runs it once the
//! admin's own transaction commits and answers with one chat line. `tick` runs in the 20 Hz world
//! tick beside the shared fields.
use super::*;
use osfr_tools::world_events::{self, WorldEventCommand};

/// (R) Runs use the public open world: zone 0 keeps the startup instance id 0 (`World::stage_public`).
const OPEN_WORLD: ZoneInstance = ZoneInstance {
    zone_id: 0,
    instance_id: 0,
};

pub(super) fn apply(
    admin: &mut Session,
    world: &mut SharedWorld,
    now_ms: u64,
    command: WorldEventCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let reply = match command {
        WorldEventCommand::Start(key) => start(admin, world, now_ms, &key),
        WorldEventCommand::Stop(key) if world.world_events.stop(&key) => {
            format!("World event {key} stopped.")
        }
        WorldEventCommand::Stop(key) => format!("World event {key} is not running."),
    };
    admin
        .sender
        .queue_message(&wrap_application(&osfr_tools::chat::system(&reply)))?;
    Ok(())
}

/// Read the admin's content directory (where `mounts.json` lives) at each start, so an edited
/// event applies to its next run. Every refusal changes nothing.
fn start(admin: &Session, world: &mut SharedWorld, now_ms: u64, key: &str) -> String {
    let file = world_events::FILE;
    let Some(dir) = admin.gateway.gameplay().map(|game| game.content_dir()) else {
        return "World events need the content catalogue. Nothing changed.".into();
    };
    let event = match world_events::load(dir) {
        Ok(events) => events.into_iter().find(|event| event.key == key),
        Err(error) => return format!("Could not read {file}: {error}. Nothing changed."),
    };
    let Some(event) = event else {
        return format!("{key} is not a world event in {file}. Nothing changed.");
    };
    if world.world_events.start(event, OPEN_WORLD, now_ms) {
        format!("World event {key} started.")
    } else {
        format!("World event {key} is already running. Nothing changed.")
    }
}

/// T462: read the events that carry a `schedule` from the content directory, once at startup (R:
/// an edit to a schedule applies at the next restart; `event start` still reads the file each
/// time). Needs `world.client_areas` loaded first: it owns the clock's offset and the seasons. A
/// file that cannot be read means no schedule and one log line, never a failed start.
pub(super) fn schedule(world: &mut SharedWorld, options: &Options) {
    let Some(dir) = options.content.as_deref() else {
        return;
    };
    match world_events::load_scheduled(dir) {
        Ok(events) => world.world_events.schedule(
            events,
            OPEN_WORLD,
            osfr_tools::client_areas::unix_now_ms(),
            &world.client_areas,
        ),
        Err(error) => {
            event(json!({"event":"world_event_schedule_unavailable","reason":error.to_string()}))
        }
    }
}

/// One world-event tick. Once a session's queue refuses a packet it gets none of its later
/// packets this tick (a small op behind a refused AddNpc would name a guid the client never
/// saw) and one log line; the runner owes it the removals it missed and shows it the live
/// actors again later (`WorldEvents::refused`). Only that session is affected (worker rule 17).
pub(super) fn tick(world: &mut SharedWorld, sessions: &mut SessionMap, now_ms: u64, unix_ms: u64) {
    let mut refused: BTreeMap<SessionId, Vec<Vec<u8>>> = BTreeMap::new();
    let areas = &mut world.client_areas;
    for (recipient, packet) in world
        .world_events
        .tick(now_ms, unix_ms, &world.players, areas)
    {
        if let Some(undelivered) = refused.get_mut(&recipient) {
            undelivered.push(packet);
            continue;
        }
        let key = world.session_keys.get(&recipient);
        let Some(session) = key.and_then(|key| sessions.get_mut(key)) else {
            continue;
        };
        if let Err(error) = session.queue_shared_message(&wrap_application(&packet)) {
            event(json!({"event":"world_event_delivery_refused",
                "character":session.character_guid,"reason":error.to_string()}));
            refused.insert(recipient, vec![packet]);
        }
    }
    for (session, undelivered) in refused {
        world.world_events.refused(session, undelivered);
    }
}
