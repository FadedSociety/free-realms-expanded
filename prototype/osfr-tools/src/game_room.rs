//! The game lobby's rooms: who opened the list, who sits in which game, and when it starts. One
//! table for the whole server (`World`), driven after a message commits, so a refused message
//! never touches a room. What goes out is addressed to characters; the gateway finds their
//! sessions. The rows' wire layout is `crate::lobby_wire`.
use crate::lobby_wire::{self, HostName, LobbyGame};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

/// The client's own countdown after 101/3 (`lobbyBottom` 421-436): five beeps, then Go.
pub const COUNTDOWN_MS: u64 = 5_000;
/// (R) Open games a server keeps at once, so one player cannot fill the list.
pub const MAX_ROOMS: usize = 256;

/// One packet for one character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Addressed {
    pub to: u64,
    pub bytes: Vec<u8>,
}

/// A player at a game: the character and the name its row shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seat {
    pub guid: u64,
    pub first: String,
    pub last: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Open,
    /// The countdown runs until this time.
    Starting(u64),
    Running,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    pub id: u32,
    pub lobby_type: u32,
    pub capacity: usize,
    pub phase: Phase,
    /// The host first.
    pub seats: Vec<Seat>,
    pub options: Vec<(u32, u32)>,
}

impl Room {
    pub fn row(&self) -> LobbyGame {
        let (host, first, last) = self.seats.first().map_or((0, "", ""), |seat| {
            (seat.guid, seat.first.as_str(), seat.last.as_str())
        });
        LobbyGame {
            id: self.id,
            options: self.options.clone(),
            host_name: HostName {
                first: first.to_owned(),
                last: last.to_owned(),
                ..HostName::default()
            },
            host,
            players: self.seats.len() as u32,
            ..LobbyGame::default()
        }
    }

    pub fn members(&self) -> impl Iterator<Item = u64> + '_ {
        self.seats.iter().map(|seat| seat.guid)
    }
}

/// Every open and running game, and who has each lobby's list open.
#[derive(Clone, Debug, Default)]
pub struct Rooms {
    next_id: u32,
    rooms: BTreeMap<u32, Room>,
    watchers: BTreeMap<u32, BTreeSet<u64>>,
}

impl Rooms {
    pub fn room(&self, id: u32) -> Option<&Room> {
        self.rooms.get(&id)
    }

    /// The game `guid` sits in.
    pub fn room_of(&self, guid: u64) -> Option<&Room> {
        self.rooms.values().find(|room| room.members().any(|seat| seat == guid))
    }

    /// 101/6 to one player, who then hears every change to that lobby.
    pub fn open_list(&mut self, lobby_type: u32, guid: u64) -> io::Result<Vec<Addressed>> {
        let bytes = self.list(lobby_type)?;
        for watchers in self.watchers.values_mut() {
            watchers.remove(&guid);
        }
        self.watchers.entry(lobby_type).or_default().insert(guid);
        Ok(vec![Addressed { to: guid, bytes }])
    }

    /// 101/6: a lobby's open games.
    fn list(&self, lobby_type: u32) -> io::Result<Vec<u8>> {
        let rows: Vec<_> = (self.rooms.values())
            .filter(|room| room.lobby_type == lobby_type && room.phase == Phase::Open)
            .map(Room::row)
            .collect();
        let kind = i32::try_from(lobby_type).map_err(|_| invalid("lobby type"))?;
        lobby_wire::show(&rows, kind)
    }

    /// 101/7 to a player leaving the list; a seat they hold goes with it, unless its game has
    /// started.
    pub fn close_list(&mut self, guid: u64) -> io::Result<Vec<Addressed>> {
        let mut out = if self.committed(guid) { Vec::new() } else { self.leave(guid)? };
        for watchers in self.watchers.values_mut() {
            watchers.remove(&guid);
        }
        out.push(Addressed { to: guid, bytes: lobby_wire::close() });
        Ok(out)
    }

    /// A seat past its START: only quitting the game, or going, gives it up.
    pub fn committed(&self, guid: u64) -> bool {
        self.room_of(guid).is_some_and(|room| room.phase != Phase::Open)
    }

    /// The lobby whose list this player has open.
    pub fn watching(&self, guid: u64) -> Option<u32> {
        (self.watchers.iter())
            .find(|(_, watchers)| watchers.contains(&guid))
            .map(|(lobby_type, _)| *lobby_type)
    }

    /// A player left the world: off every list.
    pub fn forget(&mut self, guid: u64) {
        for watchers in self.watchers.values_mut() {
            watchers.remove(&guid);
        }
    }

    /// A new game with `host` in its first seat. A player already seated, or a full server, gets
    /// the client's Unable to Create line.
    pub fn create(
        &mut self,
        lobby_type: u32,
        capacity: usize,
        host: Seat,
        options: Vec<(u32, u32)>,
    ) -> io::Result<Vec<Addressed>> {
        let guid = host.guid;
        if self.room_of(guid).is_some() || self.rooms.len() >= MAX_ROOMS || capacity < 2 {
            return Ok(refused(guid, lobby_wire::ERROR_CREATE));
        }
        self.next_id = self.next_id.checked_add(1).ok_or_else(|| invalid("room ids spent"))?;
        let id = self.next_id;
        let phase = Phase::Open;
        let room = Room { id, lobby_type, capacity, phase, seats: vec![host], options };
        let mut out = vec![Addressed { to: guid, bytes: lobby_wire::joined(id, lobby_type) }];
        out.extend(self.row_update(&room)?);
        self.rooms.insert(id, room);
        Ok(out)
    }

    /// Takes a seat in an open game that has one.
    pub fn join(&mut self, id: u32, seat: Seat) -> io::Result<Vec<Addressed>> {
        let guid = seat.guid;
        let seated = self.room_of(guid).is_some();
        let Some(room) = self.rooms.get_mut(&id).filter(|room| {
            !seated && room.phase == Phase::Open && room.seats.len() < room.capacity
        }) else {
            return Ok(refused(guid, lobby_wire::ERROR_JOIN));
        };
        room.seats.push(seat);
        let bytes = lobby_wire::joined(id, room.lobby_type);
        let room = room.clone();
        let mut out = vec![Addressed { to: guid, bytes }];
        out.extend(self.row_update(&room)?);
        Ok(out)
    }

    /// Gives up `guid`'s seat, if any. A game with no host left is over for everyone in it.
    pub fn leave(&mut self, guid: u64) -> io::Result<Vec<Addressed>> {
        let Some(id) = self.room_of(guid).map(|room| room.id) else {
            return Ok(Vec::new());
        };
        let Some(room) = self.rooms.get_mut(&id) else {
            return Ok(Vec::new());
        };
        // A running game belongs to its table now: the seat just goes.
        if room.phase == Phase::Running {
            room.seats.retain(|seat| seat.guid != guid);
            if room.seats.is_empty() {
                self.rooms.remove(&id);
            }
            return Ok(Vec::new());
        }
        let lobby_type = room.lobby_type;
        let counting = matches!(room.phase, Phase::Starting(_));
        let staying: Vec<u64> = room.members().filter(|&member| member != guid).collect();
        let was_host = room.seats.first().is_some_and(|seat| seat.guid == guid);
        let mut out: Vec<_> = if was_host {
            let everyone: Vec<u64> = room.members().collect();
            let left = lobby_wire::left(id, lobby_type);
            let tell = |to| Addressed { to, bytes: left.clone() };
            everyone.into_iter().map(tell).collect()
        } else {
            let bytes = lobby_wire::left(id, lobby_type);
            vec![Addressed { to: guid, bytes }]
        };
        if was_host {
            self.rooms.remove(&id);
            out.extend(self.tell_watchers(lobby_type, &lobby_wire::removed(id)));
        } else {
            room.seats.retain(|seat| seat.guid != guid);
            // A countdown needs its seats, and puts the row back on the list.
            if matches!(room.phase, Phase::Starting(_)) {
                room.phase = Phase::Open;
            }
            if room.phase == Phase::Open {
                let room = room.clone();
                out.extend(self.row_update(&room)?);
            }
        }
        // The client locks its lobby for a countdown; only 101/6 resets it (`Lobby:Show`).
        if counting {
            let list = self.list(lobby_type)?;
            out.extend(staying.into_iter().map(|to| Addressed { to, bytes: list.clone() }));
        }
        Ok(out)
    }

    /// The host starts a full game: 101/3 to every seat, and the list loses its row.
    pub fn start(&mut self, guid: u64, now_ms: u64) -> io::Result<Vec<Addressed>> {
        let Some(room) = (self.rooms.values_mut())
            .find(|room| room.seats.first().is_some_and(|seat| seat.guid == guid))
            .filter(|room| room.phase == Phase::Open && room.seats.len() == room.capacity)
        else {
            return Ok(refused(guid, lobby_wire::ERROR_START));
        };
        room.phase = Phase::Starting(now_ms.saturating_add(COUNTDOWN_MS));
        let (id, lobby_type) = (room.id, room.lobby_type);
        let starting = lobby_wire::starting(id, lobby_type);
        let tell = |to| Addressed { to, bytes: starting.clone() };
        let mut out: Vec<_> = room.members().map(tell).collect();
        let seated: BTreeSet<u64> = room.members().collect();
        // Sub 8 on a player's own game clears their seat (`00baff30`), so the seats don't hear it.
        let removed = self.tell_watchers(lobby_type, &lobby_wire::removed(id));
        out.extend(removed.into_iter().filter(|packet| !seated.contains(&packet.to)));
        Ok(out)
    }

    /// Who sits in a room whose countdown has ended.
    pub fn seats_due(&self, now_ms: u64) -> Vec<u64> {
        let due = |room: &&Room| matches!(room.phase, Phase::Starting(at) if at <= now_ms);
        self.rooms.values().filter(due).flat_map(Room::members).collect()
    }

    /// Rooms whose countdown ended, now running. The caller opens each one's game.
    pub fn due(&mut self, now_ms: u64) -> Vec<u32> {
        let due = |room: &&mut Room| matches!(room.phase, Phase::Starting(at) if at <= now_ms);
        let rooms = self.rooms.values_mut().filter(due);
        rooms
            .map(|room| {
                room.phase = Phase::Running;
                room.id
            })
            .collect()
    }

    /// A finished game's seats go free.
    pub fn close(&mut self, id: u32) {
        self.rooms.remove(&id);
    }

    /// 101/4 to everyone watching this game's lobby.
    fn row_update(&self, room: &Room) -> io::Result<Vec<Addressed>> {
        let bytes = lobby_wire::update(&room.row())?;
        Ok(self.tell_watchers(room.lobby_type, &bytes))
    }

    fn tell_watchers(&self, lobby_type: u32, bytes: &[u8]) -> Vec<Addressed> {
        let watchers = self.watchers.get(&lobby_type).into_iter().flatten();
        watchers.map(|&to| Addressed { to, bytes: bytes.to_vec() }).collect()
    }
}

fn refused(to: u64, code: u32) -> Vec<Addressed> {
    vec![Addressed { to, bytes: lobby_wire::error(code) }]
}

fn invalid(error: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHESS: u32 = 4;
    const HOST: u64 = 1;
    const GUEST: u64 = 2;
    const WATCHER: u64 = 3;

    fn seat(guid: u64) -> Seat {
        Seat { guid, first: format!("P{guid}"), last: String::new() }
    }

    fn to(out: &[Addressed], guid: u64) -> Vec<Vec<u8>> {
        (out.iter().filter(|p| p.to == guid)).map(|p| p.bytes.clone()).collect()
    }

    fn table() -> (Rooms, u32) {
        let mut rooms = Rooms::default();
        rooms.open_list(CHESS, WATCHER).unwrap();
        rooms.create(CHESS, 2, seat(HOST), Vec::new()).unwrap();
        let id = rooms.room_of(HOST).unwrap().id;
        (rooms, id)
    }

    #[test]
    fn a_new_game_seats_its_host_and_reaches_the_list() {
        let mut rooms = Rooms::default();
        rooms.open_list(CHESS, WATCHER).unwrap();
        let out = rooms.create(CHESS, 2, seat(HOST), vec![(5, 6)]).unwrap();
        let id = rooms.room_of(HOST).unwrap().id;
        assert_eq!(to(&out, HOST), [lobby_wire::joined(id, CHESS)]);
        let row = rooms.room(id).unwrap().row();
        assert_eq!((row.host, row.options.as_slice()), (HOST, &[(5, 6)][..]));
        assert_eq!(to(&out, WATCHER), [lobby_wire::update(&row).unwrap()]);
        let list = rooms.open_list(CHESS, GUEST).unwrap();
        assert_eq!(list[0].bytes, lobby_wire::show(&[row], 4).unwrap());
    }

    #[test]
    fn a_game_takes_two_and_refuses_a_third() {
        let (mut rooms, id) = table();
        let out = rooms.join(id, seat(GUEST)).unwrap();
        assert_eq!(to(&out, GUEST), [lobby_wire::joined(id, CHESS)]);
        let refused = rooms.join(id, seat(WATCHER)).unwrap();
        assert_eq!(to(&refused, WATCHER), [lobby_wire::error(lobby_wire::ERROR_JOIN)]);
        assert_eq!(rooms.room(id).unwrap().seats.len(), 2);
    }

    #[test]
    fn a_seated_player_can_not_sit_twice() {
        let (mut rooms, id) = table();
        let again = rooms.create(CHESS, 2, seat(HOST), Vec::new()).unwrap();
        assert_eq!(to(&again, HOST), [lobby_wire::error(lobby_wire::ERROR_CREATE)]);
        let again = rooms.join(id, seat(HOST)).unwrap();
        assert_eq!(to(&again, HOST), [lobby_wire::error(lobby_wire::ERROR_JOIN)]);
    }

    #[test]
    fn only_the_host_of_a_full_game_starts_it_and_the_countdown_ends_once() {
        let (mut rooms, id) = table();
        let early = rooms.start(HOST, 1_000).unwrap();
        assert_eq!(to(&early, HOST), [lobby_wire::error(lobby_wire::ERROR_START)]);
        rooms.join(id, seat(GUEST)).unwrap();
        let guest = rooms.start(GUEST, 1_000).unwrap();
        assert_eq!(to(&guest, GUEST), [lobby_wire::error(lobby_wire::ERROR_START)]);
        let out = rooms.start(HOST, 1_000).unwrap();
        let starting = lobby_wire::starting(id, CHESS);
        assert_eq!(to(&out, HOST), [starting.clone()]);
        assert_eq!(to(&out, GUEST), [starting]);
        assert_eq!(to(&out, WATCHER), [lobby_wire::removed(id)]);
        assert!(rooms.due(5_999).is_empty());
        assert_eq!(rooms.due(6_000), [id]);
        assert!(rooms.due(7_000).is_empty(), "once");
    }

    #[test]
    fn a_seat_watching_the_list_does_not_hear_its_own_row_go() {
        let (mut rooms, id) = table();
        rooms.open_list(CHESS, HOST).unwrap();
        rooms.join(id, seat(GUEST)).unwrap();
        let out = rooms.start(HOST, 1_000).unwrap();
        assert_eq!(to(&out, HOST), [lobby_wire::starting(id, CHESS)]);
    }

    #[test]
    fn a_guest_leaving_a_running_game_adds_no_row() {
        let (mut rooms, id) = table();
        rooms.join(id, seat(GUEST)).unwrap();
        rooms.start(HOST, 1_000).unwrap();
        assert_eq!(rooms.due(6_000), [id]);
        let out = rooms.leave(GUEST).unwrap();
        assert!(to(&out, WATCHER).is_empty());
    }

    #[test]
    fn a_running_game_keeps_its_room_until_the_last_seat_goes() {
        let (mut rooms, id) = table();
        rooms.join(id, seat(GUEST)).unwrap();
        rooms.start(HOST, 1_000).unwrap();
        assert_eq!(rooms.seats_due(6_000), [HOST, GUEST]);
        rooms.due(6_000);
        assert!(rooms.leave(HOST).unwrap().is_empty(), "the guest plays on");
        assert!(rooms.room(id).is_some());
        assert!(rooms.leave(GUEST).unwrap().is_empty());
        assert!(rooms.room(id).is_none());
    }

    /// 101/6 is the only thing that frees a client locked by the countdown.
    #[test]
    fn a_broken_countdown_resets_the_seat_that_stays() {
        let (mut rooms, id) = table();
        rooms.join(id, seat(GUEST)).unwrap();
        rooms.start(HOST, 1_000).unwrap();
        assert!(rooms.committed(GUEST));
        let out = rooms.leave(GUEST).unwrap();
        assert!(to(&out, HOST).iter().any(|bytes| bytes.starts_with(&[101, 0, 6])));
        assert!(!rooms.committed(HOST));

        rooms.join(id, seat(GUEST)).unwrap();
        rooms.start(HOST, 2_000).unwrap();
        let out = rooms.leave(HOST).unwrap();
        let heard = to(&out, GUEST);
        assert_eq!(heard[0], lobby_wire::left(id, CHESS));
        assert!(heard[1].starts_with(&[101, 0, 6]));
    }

    #[test]
    fn closing_the_list_keeps_a_started_seat() {
        let (mut rooms, id) = table();
        rooms.join(id, seat(GUEST)).unwrap();
        rooms.start(HOST, 1_000).unwrap();
        let out = rooms.close_list(GUEST).unwrap();
        assert_eq!(to(&out, GUEST), [lobby_wire::close()]);
        assert_eq!(rooms.room_of(GUEST).unwrap().id, id);
    }

    #[test]
    fn opening_another_lobby_stops_the_old_ones_news() {
        let (mut rooms, id) = table();
        rooms.open_list(5, WATCHER).unwrap();
        let out = rooms.join(id, seat(GUEST)).unwrap();
        assert!(to(&out, WATCHER).is_empty());
    }

    #[test]
    fn a_host_leaving_ends_the_game_for_everyone() {
        let (mut rooms, id) = table();
        rooms.join(id, seat(GUEST)).unwrap();
        let out = rooms.leave(HOST).unwrap();
        let left = lobby_wire::left(id, CHESS);
        assert_eq!(to(&out, HOST), [left.clone()]);
        assert_eq!(to(&out, GUEST), [left]);
        assert_eq!(to(&out, WATCHER), [lobby_wire::removed(id)]);
        assert!(rooms.room(id).is_none());
    }

    #[test]
    fn a_guest_leaving_frees_the_seat_and_stops_a_countdown() {
        let (mut rooms, id) = table();
        rooms.join(id, seat(GUEST)).unwrap();
        rooms.start(HOST, 1_000).unwrap();
        let out = rooms.leave(GUEST).unwrap();
        assert_eq!(to(&out, GUEST), [lobby_wire::left(id, CHESS)]);
        assert_eq!(rooms.room(id).unwrap().phase, Phase::Open);
        assert!(rooms.due(9_000).is_empty());
        assert_eq!(rooms.room(id).unwrap().seats.len(), 1);
        assert!(rooms.leave(GUEST).unwrap().is_empty(), "nothing left to leave");
    }

    #[test]
    fn closing_the_list_gives_up_the_seat_and_the_news() {
        let (mut rooms, id) = table();
        rooms.join(id, seat(GUEST)).unwrap();
        let out = rooms.close_list(GUEST).unwrap();
        assert_eq!(to(&out, GUEST), [lobby_wire::left(id, CHESS), lobby_wire::close()]);
        rooms.open_list(CHESS, GUEST).unwrap();
        rooms.close_list(GUEST).unwrap();
        let news = rooms.leave(HOST).unwrap();
        assert!(to(&news, GUEST).is_empty());
    }
}
