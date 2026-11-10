//! A derby played from the game lobby: one match, several human drivers. The World owns the match
//! (`HostedDerby`); each driver's own `DerbyGateway` keeps what is theirs alone (the ledger, the
//! job, the arena visit) and borrows the match around one message or tick, as a lobby board game
//! borrows its table (`bin/osfr_gateway/room_routing.rs`).
//!
//! The room's options are the captured 102/2 catalogue's (`kart_lobby.json`, lobby type 2). What
//! each one does to the match is reconstructed (R) and lives in `derby_lobby.json`.
use super::*;
use crate::game_room::Limits;

pub const LOBBY_FILE: &str = "derby_lobby.json";
const MAX_LOBBY_BYTES: usize = 16 * 1024;

/// `derby_lobby.json`: the lobby type, its gate label, and what each option value means. Option
/// values are the catalogue's display keys.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyLobby {
    schema_version: u32,
    source: String,
    pub lobby_type: u32,
    /// The gate option that opens the list ("Demo Derby Lobby").
    pub menu_string_id: u32,
    /// (R) The solo match's label on a gate that also lists the lobby ("Demo Derby").
    pub solo_menu_string_id: u32,
    max_players_setting: u32,
    max_players: BTreeMap<u32, usize>,
    track_setting: u32,
    /// Track value to the free-play row of that arena.
    tracks: BTreeMap<u32, i32>,
    difficulty_setting: u32,
    /// (R) Difficulty value to the AI cars' speed, as a share of the row's.
    difficulty_speed: BTreeMap<u32, f32>,
    ai_setting: u32,
    total_time_setting: u32,
    total_time_s: BTreeMap<u32, u64>,
    auto_start_setting: u32,
    /// The value both switches use for Yes.
    yes: u32,
    /// The catalogue's defaults, for an option the client left out.
    default_max_players: usize,
    default_track: i32,
    default_ai: bool,
    /// (R) How long a start waits on a driver who has not loaded before going on without them.
    start_timeout_ms: u64,
    /// (R) The fastest a human's car may move between two poses (m/s) before its hits on other
    /// humans are refused for `pose_jump_penalty_ms`.
    max_pose_speed_mps: f32,
    pose_jump_penalty_ms: u64,
    /// (R) Whether the client keeps its lobby through the match (it then reopens the list at the
    /// gate afterwards); off closes it at the start, and the match ends as a solo one does.
    keep_lobby: bool,
}

/// One room's options, read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomPlan {
    pub seats: usize,
    pub ai: bool,
    pub time_limit_ms: Option<u64>,
    pub game_id: i32,
    pub auto_start: bool,
    pub ai_speed: f32,
}

impl DerbyLobby {
    fn check(&self) -> io::Result<()> {
        let seats = |seats: &usize| (2..=crate::derby::MAX_DERBY_DRIVERS).contains(seats);
        let speeds = |speed: &f32| speed.is_finite() && (0.1..=4.0).contains(speed);
        let sound = self.schema_version == 1
            && !self.source.is_empty()
            && self.lobby_type != 0
            && self.menu_string_id != 0
            && self.solo_menu_string_id != 0
            && !self.max_players.is_empty()
            && self.max_players.values().all(seats)
            && seats(&self.default_max_players)
            && self.tracks.values().all(|row| *row > 0)
            && self.default_track > 0
            && self.difficulty_speed.values().all(speeds)
            && self.total_time_s.values().all(|s| (30..=3_600).contains(s))
            && (1_000..=600_000).contains(&self.start_timeout_ms)
            && self.max_pose_speed_mps.is_finite()
            && (1.0..=1_000.0).contains(&self.max_pose_speed_mps)
            && self.pose_jump_penalty_ms <= 60_000;
        if sound { Ok(()) } else { Err(invalid("invalid derby lobby content")) }
    }

    /// The host's options as a match. An option left out keeps the catalogue's default; a value
    /// the catalogue does not list refuses the room.
    pub fn plan(&self, options: &[(u32, u32)]) -> Option<RoomPlan> {
        let value = |setting: u32| options.iter().find(|(key, _)| *key == setting).map(|pair| pair.1);
        let seats = match value(self.max_players_setting) {
            Some(key) => *self.max_players.get(&key)?,
            None => self.default_max_players,
        };
        let game_id = match value(self.track_setting) {
            Some(key) => *self.tracks.get(&key)?,
            None => self.default_track,
        };
        let time_limit_ms = match value(self.total_time_setting) {
            Some(key) => Some(self.total_time_s.get(&key)?.saturating_mul(1_000)),
            None => None,
        };
        let speed = value(self.difficulty_setting).and_then(|key| self.difficulty_speed.get(&key));
        Some(RoomPlan {
            seats,
            ai: value(self.ai_setting).map_or(self.default_ai, |key| key == self.yes),
            time_limit_ms,
            game_id,
            auto_start: value(self.auto_start_setting) == Some(self.yes),
            ai_speed: speed.copied().unwrap_or(1.0),
        })
    }
}

pub(super) fn load_lobby(directory: &Path) -> io::Result<Option<Arc<DerbyLobby>>> {
    let path = directory.join(LOBBY_FILE);
    let Some(bytes) = read_regular_limited(&path, MAX_LOBBY_BYTES, "derby lobby")? else {
        return Ok(None);
    };
    let lobby: DerbyLobby = serde_json::from_slice(&bytes).map_err(invalid)?;
    lobby.check()?;
    Ok(Some(Arc::new(lobby)))
}

/// A driver taking a seat: who, the name on their car, their garage car and the next match
/// instance their own ledger would give.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerbySeat {
    pub guid: u64,
    pub name: String,
    pub look: Option<Vec<u8>>,
    pub next_instance: u32,
}

/// The match as the World holds it between messages.
#[derive(Clone)]
pub struct HostedDerby {
    room: u32,
    lobby_type: u32,
    service: DerbyService,
    /// Drivers whose own run is over (closed, quit or gone).
    gone: BTreeSet<u64>,
    /// When the start began to wait on its drivers; one deadline for the whole start.
    start_since: Option<u64>,
    now_ms: u64,
    start_timeout_ms: u64,
}

impl std::fmt::Debug for HostedDerby {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = f.debug_struct("HostedDerby");
        out.field("room", &self.room).field("instance", &self.service.instance_id());
        out.field("phase", &self.service.phase()).finish()
    }
}

impl HostedDerby {
    pub fn id(&self) -> u32 {
        self.room
    }

    pub fn lobby_type(&self) -> u32 {
        self.lobby_type
    }

    pub fn instance_id(&self) -> i32 {
        self.service.instance_id()
    }

    pub fn service(&self) -> &DerbyService {
        &self.service
    }

    pub fn seated(&self, guid: u64) -> bool {
        !self.gone.contains(&guid) && self.service.has_human(ParticipantKey(guid))
    }

    /// `guid` gives up their car: what the other drivers are owed. Once the results are up their
    /// board is left alone.
    pub fn forfeit(&mut self, guid: u64) -> io::Result<Vec<(u64, Vec<u8>)>> {
        let key = ParticipantKey(guid);
        if !self.service.has_human(key) {
            return Ok(Vec::new());
        }
        let quiet = matches!(
            self.service.phase(),
            DerbyPhase::Results | DerbyPhase::CloseEnabled | DerbyPhase::Closed | DerbyPhase::Aborted
        );
        let actions = self.service.drop_human(key, self.now_ms).map_err(invalid)?;
        Ok(if quiet { Vec::new() } else { addressed(&actions) })
    }

    pub fn release(&mut self, guid: u64) {
        self.gone.insert(guid);
    }

    pub fn is_settled(&self, guid: u64) -> bool {
        !self.seated(guid)
    }

    pub fn all_settled(&self) -> bool {
        matches!(self.service.phase(), DerbyPhase::Closed | DerbyPhase::Aborted)
            || self.service.human_keys().iter().all(|key| self.gone.contains(&key.raw()))
    }

    /// Once a tick: a start that has taken too long goes on without the drivers it waits on.
    /// The clock runs from the first tick of the start to the race, whatever step it is on, so a
    /// driver who answers each step late cannot hold the others; after a cut the rest get one
    /// more window.
    pub fn watch(&mut self, now_ms: u64) -> io::Result<Vec<(u64, Vec<u8>)>> {
        self.now_ms = now_ms;
        if self.service.start_step().is_none() {
            self.start_since = None;
            return Ok(Vec::new());
        }
        let since = *self.start_since.get_or_insert(now_ms);
        if now_ms.saturating_sub(since) < self.start_timeout_ms {
            return Ok(Vec::new());
        }
        self.start_since = Some(now_ms);
        let mut told = Vec::new();
        for key in self.service.laggards() {
            let actions = self.service.drop_human(key, now_ms).map_err(invalid)?;
            told.extend(addressed(&actions));
        }
        Ok(told)
    }
}

fn addressed(actions: &[DerbyAction]) -> Vec<(u64, Vec<u8>)> {
    let each = |action: &DerbyAction| {
        let packet = action.packet.clone();
        action.recipients.iter().map(move |key| (key.raw(), packet.clone())).collect::<Vec<_>>()
    };
    actions.iter().flat_map(each).collect()
}

/// A gateway's side of a hosted match.
#[derive(Clone, Default)]
pub(super) struct Hosting {
    /// The room this driver sits in.
    room: Option<u32>,
    /// The start records are still to send (the driver's first tick after the countdown).
    begin: bool,
    /// The World's match, lent for this transaction.
    lent: Option<HostedDerby>,
    /// What this transaction owes the other drivers.
    relay: Vec<(u64, Vec<u8>)>,
    /// The World no longer has the match.
    orphaned: bool,
    /// 101/2 for the client's lobby when this driver's run ends.
    lobby_exit: Option<Vec<u8>>,
    /// The start records were handed to this driver's client (or wait on its arena's load).
    offered: bool,
}

impl DerbyGateway {
    pub fn lobby(&self) -> Option<&DerbyLobby> {
        self.lobby.as_deref()
    }

    /// The limits of a room in the derby's lobby. `Ok(None)`: not this lobby. `Err`: the options
    /// name something the catalogue or the hosted rows do not have.
    pub fn lobby_limits(&self, lobby_type: u32, options: &[(u32, u32)]) -> Result<Option<Limits>, ()> {
        let Some(lobby) = self.lobby().filter(|lobby| lobby.lobby_type == lobby_type) else {
            return Ok(None);
        };
        let plan = lobby.plan(options).ok_or(())?;
        if !self.rows.iter().any(|row| row.game_id() == plan.game_id) {
            return Err(());
        }
        // (R) AI off needs a second driver to hit; with AI the host may start alone.
        let min_seats = if plan.ai { 1 } else { 2 };
        Ok(Some(Limits { capacity: plan.seats, min_seats, auto_start: plan.auto_start }))
    }

    /// The next match instance this driver's ledger would give: a shared match takes the highest
    /// of its seats', so no driver is ever paid for the same id twice.
    pub fn next_instance(&self) -> u32 {
        self.next_instance
    }

    /// Build the room's match with every seat in it and the Go already given; the host first.
    pub fn host(
        &self,
        room: u32,
        options: &[(u32, u32)],
        seats: &[DerbySeat],
    ) -> io::Result<HostedDerby> {
        let lobby = self.lobby().ok_or_else(|| invalid("no derby lobby content"))?;
        let plan = lobby.plan(options).ok_or_else(|| invalid("unknown derby lobby option"))?;
        let row = (self.rows.iter().find(|row| row.game_id() == plan.game_id))
            .ok_or_else(|| invalid("derby lobby track is not hosted"))?;
        let package = row.package();
        let (host, guests) = seats.split_first().ok_or_else(|| invalid("a derby needs a driver"))?;
        let instance = seats.iter().map(|seat| seat.next_instance).max().unwrap_or(0);
        let instance = i32::try_from(instance).map_err(|_| invalid("derby instance overflow"))?;
        // A guest starts where the last AI cars would: the AI fills its slots from the first.
        let car = |index: usize, seat: &DerbySeat| {
            let mut vehicle = package.vehicle.clone();
            if let Some(look) = &seat.look {
                vehicle.definition = look.clone();
            }
            vehicle.name = seat.name.clone().into_bytes();
            let slot = package.npc_vehicles.len().checked_sub(index);
            if let Some(slot) = slot.filter(|_| index > 0).and_then(|at| package.npc_vehicles.get(at)) {
                vehicle.start_transform_bits = slot.start_transform_bits;
            }
            vehicle
        };
        let mut service = DerbyService::new(
            instance,
            package.game_id,
            ParticipantKey(host.guid),
            car(0, host),
            package.rules.clone(),
            package.pickups.clone(),
        )
        .map_err(invalid)?;
        let driving = row.npc_driving().map(|mut driving| {
            driving.speed_mps *= plan.ai_speed;
            driving
        });
        // A speed the rules refuse keeps the row's own.
        if service.set_npc_driving(driving).is_err() {
            service.set_npc_driving(row.npc_driving()).map_err(invalid)?;
        }
        service.set_npc_vehicles(package.npc_vehicles.clone()).map_err(invalid)?;
        service.set_npc_drivers(package.npc_drivers.clone()).map_err(invalid)?;
        service.set_goal(package.goal.clone().unwrap_or_default()).map_err(invalid)?;
        service.set_name_id(self.row_names.get(&package.game_id).copied());
        service.set_pickup_schedule(package.pickup_schedule.clone());
        for (index, guest) in guests.iter().enumerate() {
            let key = ParticipantKey(guest.guid);
            if !service.add_human(key, car(index + 1, guest)).map_err(invalid)? {
                return Err(invalid("derby room has no seat left"));
            }
        }
        service.set_field(if plan.ai { plan.seats.max(seats.len()) } else { 0 }, plan.time_limit_ms);
        service.set_pose_guard(lobby.max_pose_speed_mps, lobby.pose_jump_penalty_ms);
        service.go();
        Ok(HostedDerby {
            room,
            lobby_type: lobby.lobby_type,
            service,
            gone: BTreeSet::new(),
            start_since: None,
            now_ms: 0,
            start_timeout_ms: lobby.start_timeout_ms,
        })
    }

    /// Take this driver's seat in `table`: their start records go out on their next tick.
    pub fn sit(&mut self, table: &HostedDerby, guid: u64) {
        let instance = table.instance_id().unsigned_abs();
        self.service = Some(table.service.clone());
        self.owner = Some(ParticipantKey(guid));
        self.next_instance = self.next_instance.max(instance.saturating_add(1));
        self.previous_job = None;
        self.arena = None;
        let keep = self.lobby().is_some_and(|lobby| lobby.keep_lobby);
        self.hosting = Hosting {
            room: Some(table.room),
            begin: true,
            lobby_exit: keep.then(|| crate::lobby_wire::left(table.room, table.lobby_type)),
            ..Hosting::default()
        };
    }

    /// The room this driver sits in, while their run is the shared one.
    pub fn hosted_room(&self) -> Option<u32> {
        self.hosting.room
    }

    pub fn lend(&mut self, table: HostedDerby) {
        self.service = Some(table.service.clone());
        self.hosting.lent = Some(table);
    }

    /// The World's match is gone while this driver still sat in it.
    pub fn orphan(&mut self) {
        self.hosting.orphaned = self.hosting.room.is_some();
    }

    /// After the message: the match as this driver left it, and what the others are owed.
    pub fn reclaim(&mut self) -> Option<(HostedDerby, Vec<(u64, Vec<u8>)>)> {
        let mut table = self.hosting.lent.take()?;
        if let (Some(_), Some(service)) = (self.hosting.room, &self.service) {
            table.service = service.clone();
        }
        Some((table, std::mem::take(&mut self.hosting.relay)))
    }

    /// The start records of a lobby match, once: the offer with its window shut, its ready, and
    /// the Go the lobby's countdown stood for.
    pub fn take_hosted_begin(&mut self) -> Option<Vec<Vec<u8>>> {
        if !std::mem::take(&mut self.hosting.begin) {
            return None;
        }
        let (service, owner) = (self.service.as_ref()?, self.owner?);
        Some(vec![
            service.hosted_offer(),
            service.ready_packet(),
            service.go_packet(),
            crate::kart_gateway::player_status(owner.raw(), ACTIVE_STATUS_MASK),
        ])
    }

    /// 101/7 before the start when the client's lobby is not kept through the match.
    pub fn lobby_close(&self) -> Option<Vec<u8>> {
        let keep = self.lobby().is_some_and(|lobby| lobby.keep_lobby);
        (!keep).then(crate::lobby_wire::close)
    }

    /// 101/2 for the client's lobby as this driver's run ends, once.
    pub fn take_lobby_exit(&mut self) -> Option<Vec<u8>> {
        self.hosting.lobby_exit.take()
    }

    /// This driver's seat is gone from under them: dropped at the start, the match closed or
    /// aborted, or the World no longer holds it.
    pub fn hosted_lost(&self) -> bool {
        let seated = || {
            let (key, service) = (self.owner?, self.service.as_ref()?);
            let over = matches!(service.phase(), DerbyPhase::Closed | DerbyPhase::Aborted);
            Some(!over && service.has_human(key))
        };
        self.hosting.room.is_some() && (self.hosting.orphaned || seated() != Some(true))
    }

    /// The start records went out: a lost seat now has a game on the client to take away.
    pub fn mark_offered(&mut self) {
        self.hosting.offered = true;
    }

    /// 39/19 for the match this driver lost their seat in; none when they were never offered it.
    pub fn lost_packet(&self) -> Option<Vec<u8>> {
        let service = self.service.as_ref().filter(|_| self.hosting.offered)?;
        Some(crate::minigame::state_remove_packet(service.instance_id(), service.game_id()))
    }

    /// Give up the seat; the World's copy of the match stays as it was lent.
    pub(super) fn leave_room(&mut self) {
        self.hosting.room = None;
        self.hosting.begin = false;
        self.hosting.orphaned = false;
    }

    /// One driver's replies out of the match's actions; in a room the other drivers' go to the
    /// relay. A solo match with a second recipient is still an integration error.
    pub(super) fn deliver(
        &mut self,
        actions: &[DerbyAction],
        owner: ParticipantKey,
    ) -> io::Result<Vec<Vec<u8>>> {
        if self.hosting.room.is_none() {
            return packets_for_owner(actions, owner);
        }
        let mut mine = Vec::new();
        for action in actions {
            for recipient in &action.recipients {
                if *recipient == owner {
                    mine.push(action.packet.clone());
                } else {
                    self.hosting.relay.push((recipient.raw(), action.packet.clone()));
                }
            }
        }
        Ok(mine)
    }

    /// A quit in a room's race: the others drive on, and this driver's own copy of the match ends
    /// through the results screen, unpaid, as a solo forfeit does. `None`: not a quit.
    pub(super) fn hosted_forfeit(
        &mut self,
        inner: &[u8],
        now_ms: u64,
    ) -> io::Result<Option<Vec<Vec<u8>>>> {
        let (Some(service), Some(owner)) = (&self.service, self.owner) else {
            return Ok(None);
        };
        let mut own = service.solo_copy(owner).map_err(invalid)?;
        let Some(actions) = own.forfeit(owner, inner, now_ms).map_err(invalid)? else {
            return Ok(None);
        };
        self.leave_room();
        self.service = Some(own);
        packets_for_owner(&actions, owner).map(Some)
    }

    /// A cancel before a room's race: only this driver's own copy is retired.
    pub(super) fn hosted_cancel(&mut self, packet: &[u8]) -> io::Result<Vec<Vec<u8>>> {
        let (Some(service), Some(owner)) = (&self.service, self.owner) else {
            return Ok(Vec::new());
        };
        let mut own = service.clone();
        Ok(own.cancel(owner, packet).map_err(invalid)?.into_iter().collect())
    }
}
