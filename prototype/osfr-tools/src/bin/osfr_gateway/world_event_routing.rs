//! T458: the gateway host's side of world events (`osfr_tools::world_events`). Gameplay stages an
//! admin's `realm event start|stop <key>` (the role gate is `command_refusal`: `event` is
//! admin-only by `account_store::can_run_realm_command`'s default); `apply` runs it once the
//! admin's own transaction commits and answers with one chat line. `tick` runs in the 20 Hz world
//! tick beside the shared fields.
use super::*;
use osfr_tools::npc_interaction;
use osfr_tools::world_events::{self, WorldEventCommand};

/// (R) Runs use the public open world: zone 0 keeps the startup instance id 0 (`World::stage_public`).
const OPEN_WORLD: ZoneInstance = ZoneInstance {
    zone_id: 0,
    instance_id: 0,
};

/// T538: one event-guid answer, wrapped for `session`. Empty replies still count as handled:
/// an event guid never belongs to the content NPC catalogue, so it must not fall through.
fn deliveries(session: &Session, replies: Vec<Vec<u8>>) -> Vec<Delivery> {
    replies
        .into_iter()
        .map(|reply| Delivery {
            recipient: session.id,
            bytes: wrap_application(&reply),
        })
        .collect()
}

/// T538: the bin's route for family 26 on an event actor's guid (the `EVNT` range), the way
/// `route_player_radial` claims player guids. `None` leaves the packet with its existing path,
/// so a normal NPC click, a player menu and an NPC dialogue answer are untouched. The voting
/// box opens the 26/3 prompt; a 26/6 that answers one of the prompt's own rows counts the vote,
/// and the winning cake answers its 15/4 line. A refusal changes nothing and answers nothing,
/// so the client keeps its prompt open. A client 26/4 is deliberately not routed: it may be the
/// client's echo of a close the server sent, and clearing the prompt on it refused a retry.
pub(super) fn application(
    session: &Session,
    inner: &[u8],
    world: &mut SharedWorld,
) -> Option<Vec<Delivery>> {
    // The registry's character guid, not the session's: a viewer session that never logged in
    // still has one, and a reconnect reuses it, which is what the run's participation keys on.
    let place = world
        .players
        .get(session.id)
        .map(|player| (player.zone, player.position, player.character_guid));
    match inner {
        [26, 0, 8, ..] if inner.len() == 13 => {
            let Some((_, _, character)) = place else {
                return None;
            };
            let request = npc_interaction::decode_npc_interact_26_8(inner).ok()?;
            if !osfr_tools::world_events::WorldEvents::is_event_guid(request.target) {
                // The player moved on to a content NPC: an abandoned vote prompt must not
                // answer a later 26/6 for that NPC's dialogue.
                world.world_events.clear_vote_prompt(character);
                return None;
            }
            let (instance, position, _) = place?;
            let replies =
                world
                    .world_events
                    .interact(request.target, character, instance, position);
            Some(deliveries(session, replies))
        }
        // No event interaction uses 26/10 today; an event guid still never reaches gameplay.
        [26, 0, 10, ..] if inner.len() == 16 => {
            let target = u64::from_le_bytes(inner[4..12].try_into().ok()?);
            osfr_tools::world_events::WorldEvents::is_event_guid(target).then(Vec::new)
        }
        [26, 0, 6, ..] if inner.len() == 8 => {
            // The canonical 26/6 reader, shared with the NPC dialogue path.
            let response = npc_interaction::decode_dialogue_response_26_6(inner).ok()?;
            let (instance, position, character) = place?;
            let replies = world
                .world_events
                .vote(character, response, instance, position)?;
            Some(deliveries(session, replies))
        }
        _ => None,
    }
}

/// T538: one world tick's costume join. It runs from `tick`, before the event steps, against the
/// committed registry instance and pose (the same values the vote and cake routes validate), so a
/// scene transfer cannot mix the old instance with a new position. The join is recorded in the
/// run (in-memory) and the quest hook is queued on the player's live gameplay; the session's own
/// tick emits and saves it through `commit_outcome`. Nothing is saved or installed here, so this
/// is not a second character-save writer, and one player's failure is logged, never propagated
/// out of the world tick (worker rule 17).
fn participate(world: &mut SharedWorld, sessions: &mut SessionMap, now_ms: u64) {
    if !world.world_events.dance_off_scan_due(now_ms) {
        return;
    }
    let candidates: Vec<(u32, SessionId, u64)> = {
        let events = &world.world_events;
        let players = &world.players;
        events
            .open_dance_offs()
            .into_iter()
            .flat_map(|(instance, serial)| {
                players.sessions_in(instance).filter_map(move |player| {
                    events.dance_off_area(serial, player.position).then_some((
                        serial,
                        player.session_id,
                        player.character_guid,
                    ))
                })
            })
            .collect()
    };
    for (serial, id, character) in candidates {
        let Some(key) = world.session_keys.get(&id).copied() else {
            continue;
        };
        let Some(session) = sessions.get_mut(&key) else {
            continue;
        };
        let Some(game) = session.gateway.gameplay() else {
            continue;
        };
        let team = match world.world_events.dance_off_participant(serial, character) {
            Some(team) => team,
            None => {
                let names = game.equipped_costume_name_ids();
                let Some(team) = world.world_events.dance_off_team(serial, character, &names)
                else {
                    continue;
                };
                if !world.world_events.join_dance_off(serial, character, team) {
                    event(json!({"event": "world_event_join_expired", "character": character}));
                    continue;
                }
                team
            }
        };
        // Queue the hook on the live gameplay; the session's own tick emits it in its staged
        // clone. A participant whose hook was lost (a disconnect) is queued again next scan.
        let credited = session
            .gateway
            .gameplay()
            .is_some_and(|game| game.dance_off_join_credited(serial, team));
        if !credited && let Some(game) = session.gateway.gameplay_mut() {
            game.queue_dance_off_join(serial, team);
        }
    }
}

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
    participate(world, sessions, now_ms);
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
