//! A derby with several human drivers, as a game lobby's room runs it: the seats are set before
//! the start, the lobby's countdown is the Go, and a driver who leaves takes only their own car.
//! The match rules stay in `derby.rs`; nothing here is used by a solo match.
use super::*;

impl DerbyService {
    /// Whether `key` still drives in this match.
    pub fn has_human(&self, key: ParticipantKey) -> bool {
        self.is_connected_human(key)
    }

    /// Every human still driving.
    pub fn human_keys(&self) -> Vec<ParticipantKey> {
        self.connected_humans()
    }

    /// (R) The room's own field: `cars` drivers in all, AI filling the seats no human took (none
    /// with `cars` at 0), and the host's match length. Call before the match is set up.
    pub fn set_field(&mut self, cars: usize, time_limit_ms: Option<u64>) {
        self.rules.max_drivers = cars.min(MAX_DERBY_DRIVERS);
        if let Some(limit) = time_limit_ms.filter(|limit| *limit > 0) {
            self.rules.time_limit_ms = Some(limit);
        }
        self.lobby_deadline_ms = None;
    }

    /// (R) Bound a human's pose travel: a car that moves faster than `max_speed_mps` between two
    /// poses may not hit another human for `penalty_ms`. Every hit is the attacker's own report
    /// from the attacker's own pose, so without this a client could place itself beside any car
    /// and knock it out every cooldown. A respawn moves a car honestly, which is why the pose is
    /// kept and only the hit waits.
    pub fn set_pose_guard(&mut self, max_speed_mps: f32, penalty_ms: u64) {
        let sound = max_speed_mps.is_finite() && max_speed_mps > 0.0;
        self.pose_guard = sound.then_some((max_speed_mps, penalty_ms));
    }

    /// Note a human pose that moved further than the guard allows since the last one.
    pub(super) fn guard_pose(&mut self, sender: ParticipantKey, pose: &VehiclePose, now_ms: u64) {
        let Some((max_speed, penalty_ms)) = self.pose_guard else {
            return;
        };
        let Some(driver) = self.drivers.get(&sender) else {
            return;
        };
        let last = driver.pose.as_ref().and_then(pose_position).zip(driver.pose_at_ms);
        let (Some((from, at)), Some(to)) = (last, pose_position(pose)) else {
            return;
        };
        let seconds = now_ms.saturating_sub(at).min(self.rules.max_pose_age_ms) as f32 / 1_000.0;
        let reach = max_speed * seconds + self.rules.contact_radius_m;
        if distance_squared(from, to) > reach * reach {
            self.pose_jumps.insert(sender, now_ms.saturating_add(penalty_ms));
        }
    }

    /// Whether `sender`'s hit on `other` is refused because `sender` just jumped: humans only.
    pub(super) fn pose_jumped(&self, sender: ParticipantKey, other: ParticipantKey, now_ms: u64) -> bool {
        let human = self.drivers.get(&other).is_some_and(|driver| driver.state.human);
        human && self.pose_jumps.get(&sender).is_some_and(|until| *until > now_ms)
    }

    /// The 39/16 offer of a lobby game, its window kept shut: the lobby's countdown was the Go.
    pub fn hosted_offer(&self) -> Vec<u8> {
        self.offer_bytes(true)
    }

    /// The server's own Go for every seat: the match leaves its lobby. `false` when it already has.
    pub fn go(&mut self) -> bool {
        if self.phase != DerbyPhase::Lobby {
            return false;
        }
        self.phase = DerbyPhase::Starting;
        self.lobby_deadline_ms = None;
        true
    }

    /// The 39/17 a seat is sent in place of its own Go request.
    pub fn go_packet(&self) -> Vec<u8> {
        let mut ack = vec![39, 0, 17];
        put_i32(&mut ack, self.instance_id);
        put_i32(&mut ack, -1);
        put_i32(&mut ack, self.game_id);
        ack
    }

    /// One human leaves and the match goes on for the rest: their car goes, and a step that was
    /// waiting only on them moves. The last human out aborts the match.
    pub fn drop_human(
        &mut self,
        key: ParticipantKey,
        now_ms: u64,
    ) -> Result<Vec<DerbyAction>, DerbyError> {
        // Before the cars are made there is nothing on the other clients to take away.
        let on_floor = !matches!(self.phase, DerbyPhase::Lobby | DerbyPhase::Starting);
        self.closed.remove(&key);
        let mut actions = self.disconnect(key)?;
        if !on_floor {
            actions.clear();
        }
        if self.human_count() == 0 {
            return Ok(actions);
        }
        match self.phase {
            DerbyPhase::Starting if self.registered.len() == self.human_count() => {
                actions.extend(self.setup_match(now_ms)?);
            }
            DerbyPhase::WaitingForState(_) => actions.extend(self.advance_phase()?),
            DerbyPhase::Countdown => actions.extend(self.advance_countdown(now_ms)?),
            DerbyPhase::CloseEnabled if self.closed.len() == self.human_count() => {
                self.phase = DerbyPhase::Closed;
            }
            _ => {}
        }
        Ok(actions)
    }

    /// The step the start is on, while it waits on its humans; `None` once the race runs.
    pub fn start_step(&self) -> Option<u32> {
        match self.phase {
            DerbyPhase::Starting => Some(1),
            DerbyPhase::WaitingForState(state) => Some(state.unsigned_abs()),
            DerbyPhase::Countdown => Some(20 + self.countdown_value),
            _ => None,
        }
    }

    /// The humans the current start step still waits on.
    pub fn laggards(&self) -> Vec<ParticipantKey> {
        let done = match self.phase {
            DerbyPhase::Starting => &self.registered,
            DerbyPhase::WaitingForState(_) => &self.phase_acks,
            DerbyPhase::Countdown => &self.countdown_acks,
            _ => return Vec::new(),
        };
        let waiting = |key: &ParticipantKey| !done.contains(key);
        self.connected_humans().into_iter().filter(waiting).collect()
    }

    /// This match as `key` alone sees it, for a driver who quits a race the others go on with:
    /// their own copy ends through the results screen as a solo forfeit does.
    pub fn solo_copy(&self, key: ParticipantKey) -> Result<Self, DerbyError> {
        let mut copy = self.clone();
        for other in self.connected_humans().into_iter().filter(|other| *other != key) {
            copy.disconnect(other)?;
        }
        Ok(copy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: ParticipantKey = ParticipantKey(11);
    const GUEST: ParticipantKey = ParticipantKey(22);

    fn vehicle() -> DerbyVehicleTemplate {
        DerbyVehicleTemplate {
            name: Vec::new(),
            definition: vec![1],
            start_transform_bits: [0; 16],
            unknown_flag: false,
            unknown_value_bits: 0,
        }
    }

    fn room(cars: usize) -> DerbyService {
        let mut service =
            DerbyService::new(7, 39, HOST, vehicle(), DerbyRules::default(), Vec::new()).unwrap();
        assert!(service.add_human(GUEST, vehicle()).unwrap());
        service.set_field(cars, Some(120_000));
        assert!(service.go());
        service
    }

    fn register(service: &mut DerbyService, key: ParticipantKey) -> Vec<DerbyAction> {
        let packet = encode_inner(&DerbyMessage::RegisterPlayer { key, unknown_key: 0 }).unwrap();
        service.handle_client(key, &packet, 0).unwrap()
    }

    fn has(actions: &[DerbyAction], subtype: u8) -> bool {
        actions.iter().any(|action| action.packet.starts_with(&[34, 0, subtype]))
    }

    #[test]
    fn a_room_goes_once_and_hides_its_offer() {
        let mut service = room(5);
        assert!(!service.go(), "once");
        let offer = service.hosted_offer();
        assert_eq!(offer[..3], [39, 0, 16]);
        assert_eq!(offer[offer.len() - 9], 1, "the hidden byte");
        assert_eq!(service.go_packet()[..3], [39, 0, 17]);
    }

    #[test]
    fn ai_fills_the_seats_the_host_asked_for_and_none_when_it_is_off() {
        let mut filled = room(5);
        register(&mut filled, HOST);
        assert!(has(&register(&mut filled, GUEST), 10));
        assert_eq!((filled.human_count(), filled.npc_count()), (2, 3));

        let mut bare = room(0);
        register(&mut bare, HOST);
        register(&mut bare, GUEST);
        assert_eq!((bare.human_count(), bare.npc_count()), (2, 0));
    }

    #[test]
    fn a_start_waiting_on_a_driver_who_left_moves_on() {
        let mut service = room(0);
        assert!(register(&mut service, HOST).is_empty(), "waits for the guest");
        assert_eq!(service.laggards(), [GUEST]);
        let actions = service.drop_human(GUEST, 5).unwrap();
        assert!(has(&actions, 10), "the host's car is made");
        assert!(!has(&actions, 8), "no car to take away yet");
        assert!(actions.iter().all(|action| action.recipients == [HOST]));
        assert_eq!(service.phase(), DerbyPhase::WaitingForState(3));
        assert!(!service.has_human(GUEST));
    }

    #[test]
    fn the_last_driver_out_aborts_the_match() {
        let mut service = room(5);
        service.drop_human(HOST, 1).unwrap();
        assert_eq!(service.phase(), DerbyPhase::Starting);
        service.drop_human(GUEST, 2).unwrap();
        assert_eq!(service.phase(), DerbyPhase::Aborted);
    }

    fn pose_at(key: ParticipantKey, x: f32) -> Vec<u8> {
        let mut transform = [0.0_f32; 16];
        (transform[0], transform[5], transform[10], transform[15]) = (1.0, 1.0, 1.0, 1.0);
        transform[12] = x;
        encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
            key,
            transform_bits: transform.map(f32::to_bits),
            vector_a_bits: [0; 4],
            vector_b_bits: [0; 4],
            controls_bits: [0; 3],
            flags: [0; 4],
            sequence: 1,
        }))
        .unwrap()
    }

    /// A client that places itself beside another human's car scores nothing until the penalty
    /// is over; a car that drove there does.
    #[test]
    fn a_car_that_jumps_beside_a_human_can_not_hit_them() {
        let mut service = room(0);
        service.set_pose_guard(60.0, 2_000);
        register(&mut service, HOST);
        register(&mut service, GUEST);
        for state in 3..=8 {
            for key in [HOST, GUEST] {
                let ack = encode_inner(&DerbyMessage::PlayerPhase { key, phase: state }).unwrap();
                service.handle_client(key, &ack, 0).unwrap();
            }
        }
        for count in (0..=4).rev() {
            for key in [HOST, GUEST] {
                let ack = encode_inner(&DerbyMessage::CountdownStatus { key, count }).unwrap();
                service.handle_client(key, &ack, 1_000).unwrap();
            }
        }
        assert_eq!(service.phase(), DerbyPhase::Running);
        let hit = |service: &mut DerbyService, now: u64| {
            let report = VehicleHitReport { reporter: HOST, other: GUEST, impact_bits: 1.0_f32.to_bits() };
            let packet = encode_inner(&DerbyMessage::VehicleHit(report)).unwrap();
            service.handle_client(HOST, &packet, now).unwrap();
            service.driver(GUEST).unwrap().damage_received
        };
        service.handle_client(GUEST, &pose_at(GUEST, 500.0), 2_000).unwrap();
        service.handle_client(HOST, &pose_at(HOST, 0.0), 2_000).unwrap();
        // 499 m in a tenth of a second.
        service.handle_client(HOST, &pose_at(HOST, 499.0), 2_100).unwrap();
        assert_eq!(hit(&mut service, 2_200), 0, "the jump scores nothing");
        assert_eq!(hit(&mut service, 3_900), 0, "nor inside the penalty");
        service.handle_client(GUEST, &pose_at(GUEST, 500.0), 4_150).unwrap();
        service.handle_client(HOST, &pose_at(HOST, 499.5), 4_150).unwrap();
        assert!(hit(&mut service, 4_200) > 0, "a car that stayed beside them hits");
    }

    #[test]
    fn a_quitter_keeps_a_copy_with_only_their_own_seat() {
        let service = room(5);
        let copy = service.solo_copy(GUEST).unwrap();
        assert_eq!(copy.human_keys(), [GUEST]);
        assert_eq!(service.human_keys(), [HOST, GUEST], "the room is untouched");
    }
}
