//! The game lobby in the shared world: `osfr_tools::game_room` holds the rooms, this finds their
//! players. A table's lobby option opens a list after its commit; family 101 then runs here, and a
//! player who leaves the world gives up their seat. The rooms address characters, so a player who
//! has already gone is skipped.
//!
//! A countdown that ends opens the game: each seat gets its own offer and run, and the World owns
//! the one table between them. As for a shared arena, a seat's message borrows the table for its
//! own transaction and it comes back after the commit. Rewards settle per player on their own
//! gateway, since each client ends its own run. (R) A seat that quits or goes forfeits. A table closes once every seat has settled or gone, whatever
//! the room's seats say: a late `FRServer_GameOver` still needs it.
use super::*;
use osfr_tools::{
    checkers_lobby::HostedGame,
    chess::{ChessMessage, ChessParticipant},
    game_room::{Addressed, Limits, Seat},
    hosted_table::HostedTable,
    lobby_wire::{self, Request},
};

/// Chess and checkers seat two (the captured 102/2 rows fix Max Players at 2); a derby room's
/// seats come from its host's options.
const SEATS: usize = 2;
/// (R) A seat this long without a move loses: neither movie has a clock, and the other player
/// would otherwise wait for ever, then pay the idle seat by quitting.
const IDLE_LIMIT_MS: u64 = 5 * 60 * 1000;

fn deliveries(world: &SharedWorld, packets: Vec<Addressed>) -> Vec<Delivery> {
    let deliver = |packet: Addressed| {
        Some(Delivery {
            recipient: *world.active_sessions.get(&packet.to)?,
            bytes: wrap_application(&packet.bytes),
        })
    };
    packets.into_iter().filter_map(deliver).collect()
}

fn seat(world: &SharedWorld, guid: u64) -> Option<Seat> {
    let name = &world.identities.get(&guid)?.name;
    Some(Seat {
        guid,
        first: name.first.clone(),
        last: name.last.clone(),
    })
}

/// A 101 message; `None` for any other family. A malformed one is dropped, a refused one answers
/// its sender through the room table.
pub(super) fn application(
    session: &Session,
    inner: &[u8],
    world: &mut SharedWorld,
    now_ms: u64,
) -> Option<Vec<Delivery>> {
    let request = match lobby_wire::decode_request(inner) {
        Ok(request) => request?,
        Err(_) => return Some(Vec::new()),
    };
    let guid = session.character_guid;
    let seated = seat(world, guid);
    let result = match request {
        Request::Join { game, .. } => seated.map(|seat| {
            let mut out = world.rooms.join(game, seat)?;
            out.extend(world.rooms.auto_start(guid, now_ms)?);
            Ok(out)
        }),
        Request::Leave { .. } if world.rooms.committed(guid) => None,
        Request::Leave { .. } => Some(world.rooms.leave(guid)),
        Request::Start { .. } => Some(world.rooms.start(guid, now_ms)),
        Request::LeaveLobby { .. } => Some(world.rooms.close_list(guid)),
        Request::Create(body) => {
            let options = lobby_wire::create_options(&body);
            match (seated, world.rooms.watching(guid), options) {
                (Some(host), Some(lobby_type), Ok(options)) => {
                    event(json!({"event": "lobby_created", "options": options}));
                    match session.gateway.lobby_limits(lobby_type, &options) {
                        Ok(limits) => {
                            let limits = limits.unwrap_or(Limits::full(SEATS));
                            Some(world.rooms.create_with(lobby_type, limits, host, options))
                        }
                        // Options the catalogue does not list: the client's Unable to Create.
                        Err(()) => {
                            let bytes = lobby_wire::error(lobby_wire::ERROR_CREATE);
                            Some(Ok(vec![Addressed { to: guid, bytes }]))
                        }
                    }
                }
                _ => None,
            }
        }
    };
    let packets = match result {
        Some(Ok(packets)) => packets,
        Some(Err(error)) => {
            event(json!({"event": "lobby_refused", "reason": error.to_string()}));
            Vec::new()
        }
        None => Vec::new(),
    };
    Some(deliveries(world, packets))
}

/// A committed table choice: this lobby's list for the player who chose it.
pub(super) fn open(session: &mut Session, world: &mut SharedWorld, lobby_type: u32) {
    match world.rooms.open_list(lobby_type, session.character_guid) {
        Ok(packets) => {
            for packet in packets {
                if let Err(error) = session.queue_shared_message(&wrap_application(&packet.bytes)) {
                    event(json!({"event": "lobby_send_refused", "reason": error.to_string()}));
                }
            }
        }
        Err(error) => event(json!({"event": "lobby_refused", "reason": error.to_string()})),
    }
}

/// Before a seat's message: its game's table, on loan to the staged gateway.
pub(super) fn lend(gateway: &mut Gateway, world: &SharedWorld) {
    let Some(id) = gateway.hosted_table() else {
        return;
    };
    match world.tables.get(&(id as u32)).cloned() {
        Some(table) => gateway.lend_table(table),
        None => gateway.orphan_table(),
    }
}

/// After the commit: the table goes back, the other seat hears what it is owed, and a game every
/// seat has finished with closes. A seat whose run ended early has forfeited.
pub(super) fn publish(
    me: &Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    returned: Option<(HostedTable, Vec<ChessMessage>)>,
    now_ms: u64,
) {
    let Some((mut table, mut relay)) = returned else {
        return;
    };
    let guid = me.character_guid;
    if me.gateway.hosted_table().is_none() {
        match table.forfeit(guid) {
            Ok(messages) => relay.extend(messages.into_iter().filter(|m| m.player_id != guid)),
            Err(error) => event(json!({"event": "lobby_refused", "reason": error.to_string()})),
        }
        table.release(guid);
    }
    if !relay.is_empty() {
        world.table_moved.insert(table.id(), now_ms);
    }
    for message in relay {
        if let Some(seat) = encounter_invites::session_of(world, sessions, message.player_id) {
            if let Err(error) = seat.queue_shared_message(&wrap_application(&message.packet)) {
                event(json!({"event": "lobby_send_refused", "reason": error.to_string()}));
            }
        }
    }
    settle_draw(&mut table, sessions, world, now_ms);
    let id = table.id();
    if table.all_settled() {
        world.tables.remove(&id);
        world.table_moved.remove(&id);
        world.rooms.close(id);
        return;
    }
    if table.is_settled(guid) {
        if let Err(error) = world.rooms.leave(guid) {
            event(json!({"event": "lobby_refused", "reason": error.to_string()}));
        }
    }
    world.tables.insert(id, table);
}

/// A draw ends without the movie's `FRServer_GameOver`, so the seat that did not end it settles
/// from here, on its own gateway. (R) It pays nothing, so there is no reward to commit.
fn settle_draw(
    table: &mut HostedTable,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    now_ms: u64,
) {
    let HostedTable::Chess(chess) = &*table else {
        return;
    };
    if chess.result().is_none_or(|result| result.winner.is_some()) {
        return;
    }
    for guid in chess.unsettled() {
        let Some(seat) = encounter_invites::session_of(world, sessions, guid) else {
            continue;
        };
        seat.gateway.lend_table(table.clone());
        let settled = seat.gateway.settle_hosted_chess(now_ms);
        if let Some((back, _)) = seat.gateway.reclaim_table() {
            *table = back;
        }
        let replies = settled.unwrap_or_else(|error| {
            event(json!({"event": "lobby_refused", "reason": error.to_string()}));
            Vec::new()
        });
        for reply in replies {
            if let Err(error) = seat.queue_shared_message(&wrap_application(&reply)) {
                event(json!({"event": "lobby_send_refused", "reason": error.to_string()}));
            }
        }
    }
}

/// Countdowns that ended: each room's game opens. A seat that cannot play gives it up first, which
/// sends the others back to the list.
pub(super) fn tick(server: &mut Server, now_ms: u64) {
    for guid in server.world.rooms.seats_due(now_ms) {
        if !ready(server, guid) {
            match server.world.rooms.leave(guid) {
                Ok(packets) => tell(server, packets),
                Err(error) => event(json!({"event": "lobby_refused", "reason": error.to_string()})),
            }
        }
    }
    time_out_idle(server, now_ms);
    watch(server, now_ms);
    for id in server.world.rooms.due(now_ms) {
        if let Err(error) = open_game(server, id, now_ms) {
            event(json!({"event": "lobby_game_failed", "room": id, "reason": error.to_string()}));
            server.world.rooms.close(id);
        }
    }
}

/// Games nobody has moved in for too long: the seat they wait on loses.
fn time_out_idle(server: &mut Server, now_ms: u64) {
    let world = &mut server.world;
    let idle = |(_, moved): (&u32, &u64)| now_ms.saturating_sub(*moved) >= IDLE_LIMIT_MS;
    let due: Vec<u32> = world.table_moved.iter().filter(|row| idle(*row)).map(|(id, _)| *id).collect();
    let mut told = Vec::new();
    for id in due {
        world.table_moved.remove(&id);
        let Some(table) = world.tables.get_mut(&id) else {
            continue;
        };
        match table.time_out() {
            Ok(messages) => told.extend(messages),
            Err(error) => event(json!({"event": "lobby_refused", "reason": error.to_string()})),
        }
    }
    let packets = told.into_iter().map(|message| Addressed { to: message.player_id, bytes: message.packet });
    tell(server, packets.collect());
}

/// Games with their own clock: a derby's start goes on without a driver who never loaded.
fn watch(server: &mut Server, now_ms: u64) {
    let (mut told, mut over) = (Vec::new(), Vec::new());
    for (id, table) in server.world.tables.iter_mut() {
        match table.watch(now_ms) {
            Ok(messages) => told.extend(messages),
            Err(error) => event(json!({"event": "lobby_refused", "reason": error.to_string()})),
        }
        // A start that cut its last driver has nobody left to publish its end.
        if table.all_settled() {
            over.push(*id);
        }
    }
    for id in over {
        server.world.tables.remove(&id);
        server.world.table_moved.remove(&id);
        server.world.rooms.close(id);
    }
    let packets = told.into_iter().map(|message| Addressed { to: message.player_id, bytes: message.packet });
    tell(server, packets.collect());
}

fn ready(server: &Server, guid: u64) -> bool {
    let session = group_arena::key_of(server, guid).and_then(|key| server.sessions.get(&key));
    session.is_some_and(|session| session.gateway.can_host())
}

fn tell(server: &mut Server, packets: Vec<Addressed>) {
    for packet in packets {
        let key = group_arena::key_of(server, packet.to);
        let Some(session) = key.and_then(|key| server.sessions.get_mut(&key)) else {
            continue;
        };
        if let Err(error) = session.queue_shared_message(&wrap_application(&packet.bytes)) {
            event(json!({"event": "lobby_send_refused", "reason": error.to_string()}));
        }
    }
}

fn open_game(server: &mut Server, id: u32, now_ms: u64) -> io::Result<()> {
    let room = server.world.rooms.room(id);
    let seats: Vec<u64> = room.map(|room| room.members().collect()).unwrap_or_default();
    let lobby_type = room.map_or(0, |room| room.lobby_type);
    let session = |guid: &u64| group_arena::key_of(server, *guid).and_then(|key| server.sessions.get(&key));
    if seats.iter().filter_map(session).any(|seat| seat.gateway.is_derby_lobby(lobby_type)) {
        return open_derby(server, id);
    }
    let mut taken = Vec::new();
    for guid in seats {
        let Some(key) = group_arena::key_of(server, guid) else {
            continue;
        };
        let Some(session) = server.sessions.get_mut(&key) else {
            continue;
        };
        let (replies, instance) = session.gateway.begin_hosted(lobby_type, u64::from(id), now_ms)?;
        for reply in replies {
            if let Err(error) = session.queue_shared_message(&wrap_application(&reply)) {
                event(json!({"event": "lobby_send_refused", "reason": error.to_string()}));
            }
        }
        if let Some(instance_id) = instance {
            taken.push((key, ChessParticipant { player_id: guid, instance_id }));
        }
    }
    let [(host, white), (_, black)] = taken.as_slice() else {
        return Err(io::Error::other("a lobby game needs both seats"));
    };
    let session = server.sessions.get_mut(host).ok_or_else(|| io::Error::other("host left"))?;
    let table = if lobby_type == osfr_tools::hosted_table::CHECKERS_LOBBY {
        let seat = |seat: &ChessParticipant| (seat.player_id, seat.instance_id);
        HostedTable::Checkers(HostedGame::new(id, seat(white), seat(black)))
    } else {
        HostedTable::Chess(session.gateway.open_hosted_chess(u64::from(id), *white, *black)?)
    };
    server.world.tables.insert(id, table);
    server.world.table_moved.insert(id, now_ms);
    Ok(())
}

/// A derby room's match: built once with every seat in it, then each driver's own gateway takes
/// its seat and starts on its next tick (the job, the arena, the offer). The first seat's content
/// builds it; every session loads the same content.
fn open_derby(server: &mut Server, id: u32) -> io::Result<()> {
    let room = server.world.rooms.room(id).ok_or_else(|| io::Error::other("no such room"))?;
    let (options, names) = (room.options.clone(), room.seats.clone());
    let (mut seats, mut keys) = (Vec::new(), Vec::new());
    for seat in names {
        let key = group_arena::key_of(server, seat.guid);
        let Some((key, session)) = key.and_then(|key| Some((key, server.sessions.get(&key)?))) else {
            continue;
        };
        let name = format!("{} {}", seat.first, seat.last).trim().to_owned();
        if let Some(driver) = session.gateway.derby_seat(name) {
            seats.push(driver);
            keys.push(key);
        }
    }
    let host = keys.first().and_then(|key| server.sessions.get(key));
    let host = host.ok_or_else(|| io::Error::other("a derby room needs a driver"))?;
    let table = host.gateway.host_derby(id, &options, &seats)?;
    for key in keys {
        if let Some(session) = server.sessions.get_mut(&key) {
            session.gateway.sit_derby(&table);
        }
    }
    server.world.tables.insert(id, HostedTable::Derby(Box::new(table)));
    Ok(())
}

impl SharedWorld {
    /// A player's session ended: their seat and their place on any list go. A game they were
    /// playing is theirs to lose, and the other seat hears it. A zone change is not this: a derby
    /// zones its drivers into the arena and back, and the client keeps its lobby across zones
    /// (`LoadZone` `009386f0` shows it again), so only `end_session` calls it.
    pub(super) fn leave_rooms(&mut self, guid: u64) -> Vec<Delivery> {
        let mut told = Vec::new();
        let mut finished = Vec::new();
        for (id, table) in self.tables.iter_mut().filter(|(_, table)| table.seated(guid)) {
            match table.forfeit(guid) {
                Ok(messages) => told.extend((messages.into_iter())
                    .filter(|message| message.player_id != guid)
                    .map(|message| Addressed { to: message.player_id, bytes: message.packet })),
                Err(error) => event(json!({"event": "lobby_refused", "reason": error.to_string()})),
            }
            table.release(guid);
            if table.all_settled() {
                finished.push(*id);
            }
        }
        for id in finished {
            self.tables.remove(&id);
            self.table_moved.remove(&id);
            self.rooms.close(id);
        }
        match self.rooms.leave(guid) {
            Ok(mut packets) => {
                self.rooms.forget(guid);
                packets.extend(told);
                deliveries(self, packets)
            }
            Err(error) => {
                event(json!({"event": "lobby_refused", "reason": error.to_string()}));
                Vec::new()
            }
        }
    }
}
