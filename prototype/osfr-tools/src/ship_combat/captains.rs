//! The AI captains: the ships the server sails beside the player. Each takes the nearest free chest
//! and brings it home to its own pad, and the client shows them as it shows any other ship: the
//! standings row, the flag, the health bar and the chests over the mast. (R) The live AI (Kynapse,
//! `EnableAI` and `NpcCount` in the zone loader) never shipped, so this behaviour follows the
//! videos: "they collect, bank, ram and sink like players".
use super::battle::{self, Aim, Battery, Hull, Mark, Shot};
use super::{
    Arena, Content, Decal, Deserialize, Ship, Treasure, UPDATE_SHIP_PHYSICS, VecDeque,
    WHIRLPOOL_HOLD_MS, destroy_pickup, distance, packet, player_gold, player_state, refresh_ui,
    reset_ship, ship_state,
};

/// How often a captain's ship reports: the client flies it on between reports.
const REPORT_MS: u64 = 250;
/// The longest step a captain's clock takes, so a stalled tick cannot jump a ship ashore.
const MAX_STEP_MS: u64 = 500;
/// Far from any player guid: their low nibble is 1, and 0 here sends the standings to the name id
/// (`sub_A62FD0`).
const GUID_BASE: u64 = 0x7000_0000_0000_0000;

/// One seat in `Content::captains`: a name from the client's locale, the ship and the flag.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Captain {
    pub name_id: u32,
    pub ship: String,
    pub flag: String,
}

/// A captain at sea.
#[derive(Clone, Debug, PartialEq)]
pub struct Bot {
    pub guid: u64,
    /// Its dock slot and standings colour.
    pub slot: u32,
    pub name_id: u32,
    pub ship: Ship,
    flag: Option<Decal>,
    pub(super) at: [f32; 3],
    /// Radians from +z towards +x, as `atan2(dx, dz)`.
    heading: f32,
    /// The chest it is after; none sends it home.
    goal: Option<usize>,
    hold: u32,
    gold: u32,
    stepped_ms: Option<u64>,
    reported_ms: u64,
    unloaded_ms: u64,
    /// The speed it last made, for the balls it fires.
    speed: f32,
    battery: Battery,
    hull: Hull,
    down: Option<Down>,
}

/// A captain out of the fight: going down, then back on its pad and held, each until a time.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Down {
    Sinking(u64),
    Refloating(u64),
}

impl Content {
    /// The captains for `place`, each on its own pad after the player's, as many as there are
    /// seats and docks.
    pub fn crew(&self, place: &Arena) -> Vec<Bot> {
        let seats = self.captains.iter().zip(place.docks.iter().skip(1)).zip(1..);
        seats
            .filter_map(|((captain, dock), slot)| {
                let mut ship = self.fleet_ship(&captain.ship)?;
                let flag = self.decals.iter().find(|d| d.alias == captain.flag).cloned();
                ship.flag.clone_from(&captain.flag);
                Some(Bot {
                    guid: GUID_BASE + u64::from(slot) * 16,
                    slot,
                    name_id: captain.name_id,
                    ship,
                    flag,
                    at: dock.bank,
                    heading: 0.0,
                    goal: None,
                    hold: 0,
                    gold: 0,
                    stepped_ms: None,
                    reported_ms: 0,
                    unloaded_ms: 0,
                    speed: 0.0,
                    battery: Battery::default(),
                    hull: Hull::default(),
                    down: None,
                })
            })
            .collect()
    }
}

impl Bot {
    /// Its ship and standings row, for the start packets.
    pub fn introduce(&self, content: &Content) -> [Vec<u8>; 2] {
        let colour = content.colours.get(self.slot as usize).map_or("", String::as_str);
        [
            super::create_ship(self.guid, self.slot, self.at, &self.ship, colour),
            super::captain_card(self.guid, self.name_id, self.slot, &self.ship, self.flag.as_ref()),
        ]
    }

    /// What lets the ship go, as the first UpdatePlayerState does the player's.
    pub fn launch(&self) -> Vec<u8> {
        player_state(self.guid, self.hull.health(&self.ship), self.hold)
    }

    /// One tick at `now_ms`: sail for the goal, take what lies in reach out of `chests` (`None`
    /// where one lies free), and bank on the pad.
    pub fn sail(
        &mut self,
        now_ms: u64,
        place: &Arena,
        treasure: &Treasure,
        chests: &mut [Option<u64>],
        grace_ms: u64,
    ) -> Vec<Vec<u8>> {
        let Some(home) = place.docks.get(self.slot as usize).map(|dock| dock.bank) else {
            return Vec::new();
        };
        if self.down.is_some() {
            return self.recover(now_ms, home, grace_ms);
        }
        let step_ms = now_ms.saturating_sub(self.stepped_ms.unwrap_or(now_ms)).min(MAX_STEP_MS);
        self.stepped_ms = Some(now_ms);
        let mut out = Vec::new();
        self.choose(place, chests);
        let target = self.goal.and_then(|i| place.pickups.get(i)).copied().unwrap_or(home);
        let (speed, yaw) = self.steer(target, step_ms, home);
        self.speed = speed;
        let capacity = u32::try_from(self.ship.hold_capacity).unwrap_or(0);
        let held = self.hold;
        for ((point, until), id) in place.pickups.iter().zip(chests.iter_mut()).zip(1..) {
            if until.is_none()
                && self.hold < capacity
                && distance(*point, self.at) <= treasure.pickup_radius
            {
                *until = Some(now_ms.saturating_add(u64::from(treasure.respawn_seconds) * 1000));
                self.hold += 1;
                out.push(destroy_pickup(id, true));
            }
        }
        let unload = 1000 / u64::try_from(self.ship.hold_unload_per_second.max(1)).unwrap_or(1);
        let due = now_ms.saturating_sub(self.unloaded_ms) >= unload;
        if self.hold > 0 && due && distance(home, self.at) <= treasure.bank_radius {
            self.hold -= 1;
            self.unloaded_ms = now_ms;
            self.gold = self.gold.saturating_add(treasure.value);
            out.extend([player_gold(self.guid, self.gold), refresh_ui()]);
        }
        // The home pad repairs, as the player's does.
        let healed = self.hull.damage > 0 && distance(home, self.at) <= treasure.bank_radius;
        if healed {
            self.hull.damage = 0;
        }
        if self.hold != held || healed {
            out.push(self.launch());
        }
        if now_ms.saturating_sub(self.reported_ms) >= REPORT_MS {
            self.reported_ms = now_ms;
            out.push(ship_physics(self.guid, now_ms, (self.heading, yaw), self.at, speed));
        }
        out
    }

    /// Sunk, it waits out the sinking, comes back whole on its pad, and is let go a beat later.
    fn recover(&mut self, now_ms: u64, home: [f32; 3], grace_ms: u64) -> Vec<Vec<u8>> {
        match self.down {
            Some(Down::Sinking(until)) if until <= now_ms => {
                self.at = home;
                self.hull.repair(now_ms.saturating_add(WHIRLPOOL_HOLD_MS + grace_ms));
                self.goal = None;
                self.stepped_ms = None;
                self.down = Some(Down::Refloating(now_ms.saturating_add(WHIRLPOOL_HOLD_MS)));
                vec![reset_ship(self.guid, home, 0.0)]
            }
            Some(Down::Refloating(until)) if until <= now_ms => {
                self.down = None;
                vec![self.launch()]
            }
            _ => Vec::new(),
        }
    }

    /// Whether a ball can strike it now.
    pub(super) fn exposed(&self, now_ms: u64) -> bool {
        self.down.is_none() && now_ms >= self.hull.safe_until
    }

    /// A ball struck it near `ball`: the health shows, and at none it goes down with its hold.
    pub(super) fn strike(
        &mut self,
        damage: i32,
        ball: [f32; 3],
        now_ms: u64,
        sink_ms: u64,
    ) -> Vec<Vec<u8>> {
        let (x, z) = self.heading.sin_cos();
        let side = battle::side_of(self.at, [x, 0.0, z], ball);
        let hit = battle::ship_hit(self.guid, damage, side);
        if self.hull.strike(&self.ship, damage) {
            self.hold = 0;
            self.down = Some(Down::Sinking(now_ms.saturating_add(sink_ms)));
            // The flag only shows the sinking: the last report's throttle would sail it on, and
            // this tick's report may already have used `now_ms`.
            let stopped = ship_physics(self.guid, now_ms + 1, (self.heading, 0.0), self.at, 0.0);
            return vec![hit, stopped, ship_state(self.guid, battle::SINKING, 0, 0)];
        }
        vec![hit, self.launch()]
    }

    /// Where a gun of the player's aims: the ship and its way.
    pub(super) fn mark(&self) -> Mark {
        let (x, z) = self.heading.sin_cos();
        Mark { at: self.at, velocity: [x * self.speed, 0.0, z * self.speed] }
    }

    /// Fires the side that has `target` in its guns' arc and reach. (R) Captains do not turn to
    /// fight, and they aim where the player is, not where it is going, so a moving ship can dodge.
    pub(super) fn engage(
        &mut self,
        target: [f32; 3],
        now_ms: u64,
        content: &Content,
        air: (&mut VecDeque<Shot>, &mut u32),
    ) -> Vec<Vec<u8>> {
        let (x, z) = self.heading.sin_cos();
        let (dx, dz) = (target[0] - self.at[0], target[2] - self.at[2]);
        let range = dx.hypot(dz);
        let reach = content.gunnery.speed * content.gunnery.flight_seconds;
        if self.down.is_some() || range < 1.0 || range > reach || (dx * x + dz * z).abs() > 0.8 * range {
            return Vec::new();
        }
        let side = battle::side_of(self.at, [x, 0.0, z], target);
        let aim = Aim {
            owner: self.guid,
            slot: self.slot,
            at: self.at,
            forward: [x, 0.0, z],
            velocity: [x * self.speed, 0.0, z * self.speed],
        };
        let mark = [Mark { at: target, velocity: [0.0; 3] }];
        let ship = self.ship.clone();
        self.battery.fire(side, now_ms, &ship, content, (&aim, &mark), air)
    }

    /// With a chest aboard it goes home, as a hold of one is worth banking (R); otherwise it wants
    /// the nearest free chest.
    fn choose(&mut self, place: &Arena, chests: &[Option<u64>]) {
        if self.hold > 0 {
            self.goal = None;
            return;
        }
        if self.goal.and_then(|i| chests.get(i)).is_some_and(Option::is_none) {
            return;
        }
        let free = (place.pickups.iter().zip(chests).enumerate())
            .filter(|(_, (_, until))| until.is_none())
            .map(|(index, (point, _))| (index, distance(*point, self.at)));
        self.goal = free.min_by(|a, b| a.1.total_cmp(&b.1)).map(|(index, _)| index);
    }

    /// Turns towards `target` at the ship's own turn rate and moves on at its own top speed, the
    /// speed the client's own physics reach (a slower report would fight them), slowing while the
    /// bow points away so a chest inside its turning circle is still reached. Moored at `home`
    /// with nothing to fetch it stays still. The speed it made and its turn in radians a second.
    fn steer(&mut self, target: [f32; 3], step_ms: u64, home: [f32; 3]) -> (f32, f32) {
        if self.goal.is_none() && distance(home, self.at) < 1.0 {
            return (0.0, 0.0);
        }
        let seconds = step_ms as f32 / 1000.0;
        let turn = self.ship.gears.first().and_then(|gear| gear.get("MaxAngularSpeed"));
        let turn = (turn.copied().unwrap_or(60.0) as f32).to_radians() * seconds;
        let wanted = (target[0] - self.at[0]).atan2(target[2] - self.at[2]);
        let off = (wanted - self.heading + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let turned = off.clamp(-turn, turn);
        self.heading += turned;
        let top = self.ship.physics.get("MaxSpeed").copied().unwrap_or(0.0) as f32;
        let speed = top * off.cos().max(0.1);
        // Not past the target: the pad is a small place.
        let travel = (speed * seconds).min(distance(target, self.at));
        self.at[0] += self.heading.sin() * travel;
        self.at[2] += self.heading.cos() * travel;
        (speed, if seconds > 0.0 { turned / seconds } else { 0.0 })
    }

    /// The gold it has banked.
    pub fn gold(&self) -> u32 {
        self.gold
    }
}

/// UpdateShipPhysics (16) for a ship the server sails (reader `sub_CA5F90`, handler 00ca6c90): the
/// guid, a flag, a stamp that must rise, the transform as `create_ship` writes it, the velocity,
/// the rate of turn about +y (`sub_FEDC80`), then the vehicle's throttle and steering and a word
/// of bits. The client flies the ship on its own physics between these (full throttle, no
/// steering) and eases it to each one.
fn ship_physics(
    guid: u64,
    stamp: u64,
    (heading, yaw): (f32, f32),
    at: [f32; 3],
    speed: f32,
) -> Vec<u8> {
    let (x, z) = heading.sin_cos();
    let mut body = guid.to_le_bytes().to_vec();
    body.push(0);
    body.extend(stamp.to_le_bytes());
    let throttle = if speed > 0.0 { 1.0 } else { 0.0 };
    let values = [
        z, 0.0, -x, 0.0, 0.0, 1.0, 0.0, 0.0, x, 0.0, z, 0.0, at[0], at[1], at[2], 1.0, x * speed,
        0.0, z * speed, 0.0, yaw, throttle, 0.0,
    ];
    for value in values {
        body.extend(f32::to_le_bytes(value));
    }
    body.extend(0_u32.to_le_bytes());
    packet(UPDATE_SHIP_PHYSICS, &body)
}

#[cfg(test)]
impl Bot {
    /// Puts it somewhere and stops it, for a test.
    pub(super) fn moor(&mut self, at: [f32; 3]) {
        self.at = at;
    }

    /// Whether it is out of the fight.
    pub(super) fn down(&self) -> bool {
        self.down.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ship_position;
    use super::*;
    use std::path::Path;

    fn content() -> Content {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        Content::load(&dir).unwrap().unwrap()
    }

    /// Every seat takes a dock of its own after the player's, with the client's own names.
    #[test]
    fn the_crew_takes_the_docks_after_the_players() {
        let content = content();
        let place = &content.arenas[0];
        let crew = content.crew(place);
        assert_eq!(crew.len(), 5);
        assert_eq!(crew[0].slot, 1);
        assert_eq!(crew[0].at, place.docks[1].bank);
        assert_eq!(crew[0].name_id, 427_711, "Commodore Kraken");
        assert!(crew.iter().all(|bot| bot.guid & 0xf == 0));
        let few = Arena { docks: place.docks[..2].to_vec(), ..place.clone() };
        assert_eq!(content.crew(&few).len(), 1, "no dock, no seat");
        let [ship, card] = crew[0].introduce(&content);
        assert_eq!(ship[..4], [94, 0, 9, 0]);
        assert_eq!(ship[4..12], crew[0].guid.to_le_bytes());
        assert_eq!(card[..4], [94, 0, 8, 0]);
        // The tail: the name id, the slot and the colour.
        let tail = card.len() - 12;
        assert_eq!(card[tail..tail + 4], 427_711_u32.to_le_bytes());
        assert_eq!(card[tail + 4..tail + 8], 1_i32.to_le_bytes());
        assert_eq!(card[tail + 8..], 1_u32.to_le_bytes());
    }

    /// The report has the size and shape of the client's own: its reader takes the same packet.
    #[test]
    fn a_captains_report_matches_the_clients_own() {
        let out = ship_physics(7, 9, (0.0, 0.5), [1.0, 2.0, 3.0], 20.0);
        assert_eq!(out.len(), 117);
        assert_eq!(out[4 + 97..4 + 101], 0.5_f32.to_le_bytes(), "the turn, after the velocity");
        assert_eq!(ship_position(&out), Some((7, [1.0, 2.0, 3.0])));
        let motion = super::super::ship_motion(&out).unwrap();
        assert_eq!(motion, ([0.0, 0.0, 1.0], [0.0, 0.0, 20.0]));
    }

    /// A captain takes the nearest chest, turns for home and banks it; a chest someone else took
    /// sends it for another.
    #[test]
    fn a_captain_fetches_a_chest_and_banks_it() {
        let content = content();
        let mut place = content.arenas[0].clone();
        let bank = place.docks[1].bank;
        let near = [bank[0] + 100.0, bank[1], bank[2]];
        let far = [bank[0], bank[1], bank[2] + 400.0];
        place.pickups = vec![far, near];
        let mut bot = content.crew(&place).remove(0);
        let mut chests = vec![None, None];
        chests[1] = Some(1_000_000);
        let treasure = &content.treasure;
        let mut now = 1;
        bot.sail(now, &place, treasure, &mut chests, 0);
        assert_eq!(bot.goal, Some(0), "the near one is gone");
        chests[1] = None;
        bot.goal = None;
        bot.sail(now, &place, treasure, &mut chests, 0);
        assert_eq!(bot.goal, Some(1), "the nearest");
        let mut packets = Vec::new();
        while bot.gold == 0 {
            now += 100;
            assert!(now < 120_000, "banked by now");
            packets.extend(bot.sail(now, &place, treasure, &mut chests, 0));
        }
        let has = |head: [u8; 4]| packets.iter().any(|p| p.starts_with(&head));
        assert!(has([94, 0, 29, 0]), "the chest goes");
        assert!(has([94, 0, 16, 0]), "the ship reports");
        assert!(has([94, 0, 17, 0]), "the hold shows");
        assert_eq!(bot.gold, treasure.value);
        assert_eq!(bot.hold, 0);
    }

    /// It fires only with the player in its guns' arc and reach, from the side that faces them.
    #[test]
    fn a_captain_fires_on_the_beam() {
        let content = content();
        let place = content.arenas[0].clone();
        let mut bot = content.crew(&place).remove(0);
        bot.at = [0.0, 53.6, 0.0];
        let (mut shots, mut next) = (VecDeque::new(), 1);
        let mut fire = |bot: &mut Bot, target: [f32; 3], now: u64| {
            bot.engage(target, now, &content, (&mut shots, &mut next)).len()
        };
        assert_eq!(fire(&mut bot, [0.0, 53.6, 40.0], 0), 0, "dead ahead");
        assert_eq!(fire(&mut bot, [500.0, 53.6, 0.0], 0), 0, "out of reach");
        let side = bot.ship.cannons.iter().filter(|gun| gun.left).count();
        assert_eq!(fire(&mut bot, [40.0, 53.6, 0.0], 0), side, "the left side, +x");
        assert_eq!(fire(&mut bot, [40.0, 53.6, 0.0], 10), 0, "reloading");
        assert_eq!(fire(&mut bot, [-40.0, 53.6, 0.0], 10), side, "the other side was loaded");
    }

    /// A chest just inside the turning circle is still reached: the ship slows to come round.
    #[test]
    fn a_captain_never_orbits_a_chest() {
        let content = content();
        let mut place = content.arenas[0].clone();
        let mut bot = content.crew(&place).remove(0);
        let abeam = [bot.at[0] + 20.0, bot.at[1], bot.at[2]];
        place.pickups = vec![abeam];
        place.docks[bot.slot as usize].bank = [abeam[0] + 400.0, abeam[1], abeam[2] + 400.0];
        let mut chests = vec![None];
        let mut now = 1;
        while bot.hold == 0 {
            bot.sail(now, &place, &content.treasure, &mut chests, 0);
            now += 50;
            assert!(now < 30_000, "still circling at {:?}", bot.at);
        }
    }

    /// With nothing to fetch it stays on its pad, and a stalled tick moves it no more than a step.
    #[test]
    fn a_captain_waits_at_home_and_never_jumps() {
        let content = content();
        let place = content.arenas[0].clone();
        let mut bot = content.crew(&place).remove(0);
        let mut taken = vec![Some(u64::MAX); place.pickups.len()];
        let home = bot.at;
        bot.sail(1, &place, &content.treasure, &mut taken, 0);
        bot.sail(5_000, &place, &content.treasure, &mut taken, 0);
        assert_eq!(bot.at, home);
        let mut free = vec![None; place.pickups.len()];
        bot.sail(5_001, &place, &content.treasure, &mut free, 0);
        bot.sail(60_000, &place, &content.treasure, &mut free, 0);
        let reach = 0.5 * bot.ship.physics["MaxSpeed"] as f32 + 1.0;
        assert!(distance(bot.at, home) <= reach, "{:?}", bot.at);
    }
}
