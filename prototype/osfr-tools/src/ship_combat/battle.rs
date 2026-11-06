//! Cannon fire between ships: the guns a ship carries, the balls in the air and the hulls they
//! strike. The server owns all of it; the client only says which side to fire.
use super::{Content, Ship, VecDeque, near, packet, spawn_projectile};

/// ShipHit (23; reader `sub_CA59A0`, handler 00ca64e0).
const SHIP_HIT: u16 = 23;
/// The ship flag that sinks it (`sub_C9E680`: animation 7120 and the sink effect; clearing it
/// plays the idle again).
pub(super) const SINKING: u32 = 0x40;
/// The owner of the player's own balls: no captain's guid is 0.
pub(super) const PLAYER: u64 = 0;

/// A ball in the air, from `from` at `velocity` since `fired_ms`.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Shot {
    pub(super) id: u32,
    pub(super) owner: u64,
    from: [f32; 3],
    velocity: [f32; 3],
    fired_ms: u64,
    pub(super) lands_ms: u64,
    /// Where it meets the water.
    pub(super) at: [f32; 3],
    /// How far along its flight hits have been looked for.
    checked_ms: u64,
}

impl Shot {
    /// Where it is over the water at `now_ms`, at the height of the water it will meet.
    fn over(&self, now_ms: u64) -> [f32; 3] {
        let seconds = now_ms.min(self.lands_ms).saturating_sub(self.fired_ms) as f32 / 1000.0;
        let [x, _, z] = self.from;
        [x + self.velocity[0] * seconds, self.at[1], z + self.velocity[2] * seconds]
    }

    /// The stretch it has flown since last asked, as two points.
    pub(super) fn flown(&mut self, now_ms: u64) -> ([f32; 3], [f32; 3]) {
        let stretch = (self.over(self.checked_ms), self.over(now_ms));
        self.checked_ms = now_ms;
        stretch
    }
}

/// Whether the stretch from `a` to `b` passes within `radius` of `point`, along the water: a ball
/// flies metres between ticks, so its ends alone could step over a hull.
pub(super) fn passes(a: [f32; 3], b: [f32; 3], point: [f32; 3], radius: f32) -> bool {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    let length = dx * dx + dz * dz;
    let along = if length > 0.0 {
        (((point[0] - a[0]) * dx + (point[2] - a[2]) * dz) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    near(point, [a[0] + dx * along, a[1], a[2] + dz * along], radius)
}

/// A ship a gun can swing to: where it is, and how fast it moves if the gunner leads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Mark {
    pub(super) at: [f32; 3],
    pub(super) velocity: [f32; 3],
}

/// (R) How far a gun swings off the beam, as the cosine of the angle: about 53 degrees. The live
/// cannon config carried this as AutoFireMinAngle and AutoFireMaxAngle, with AutoFireRadius (here
/// the ball's reach) and AutoFireLeadTime, so its guns laid themselves.
const SWING: f32 = 0.6;
/// (R) The shortest flight a laid gun takes.
const MIN_FLIGHT: f32 = 0.4;

/// Where a gun at `from` firing along `out_of_side` puts its ball: the nearest mark in its arc and
/// reach, led by the flight, as the landing point and the flight. `None` fires straight out.
fn lay(
    from: [f32; 3],
    out_of_side: [f32; 2],
    (speed, flight): (f32, f32),
    marks: &[Mark],
) -> Option<([f32; 2], f32)> {
    let reach = speed * flight;
    let bearing = |mark: &&Mark| {
        let (dx, dz) = (mark.at[0] - from[0], mark.at[2] - from[2]);
        let range = dx.hypot(dz);
        let on_side = dx * out_of_side[0] + dz * out_of_side[1];
        (range > 1.0 && range <= reach && on_side >= SWING * range).then_some(range)
    };
    let nearest = (marks.iter()).filter(|mark| bearing(mark).is_some()).min_by(|a, b| {
        bearing(a).unwrap_or(reach).total_cmp(&bearing(b).unwrap_or(reach))
    })?;
    let time = (bearing(&nearest)? / speed).clamp(MIN_FLIGHT, flight);
    let [vx, _, vz] = nearest.velocity;
    Some(([nearest.at[0] + vx * time, nearest.at[2] + vz * time], time))
}

/// Who fires, from where, and how it moves: the bow is flat and of length 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Aim {
    pub(super) owner: u64,
    pub(super) slot: u32,
    pub(super) at: [f32; 3],
    pub(super) forward: [f32; 3],
    pub(super) velocity: [f32; 3],
}

/// One ship's guns: when each is loaded again, in the ship's order (one past the end is loaded),
/// and when each side's crew (right, left) has finished the guns it has been given.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Battery {
    pub(super) reloads: Vec<u64>,
    crews: [u64; 2],
}

impl Battery {
    /// `side` fires (1 left, 2 right, 5 both): every loaded gun there sends a ball at the nearest
    /// of `marks` in its arc, or straight out to land on the water `flight_seconds` on, and its
    /// side's crew takes it next, so a side reloads in `reload_seconds` however long the button
    /// stays down. The client's pool (`ProjectileMax`) bounds the balls in the air; a gun past it
    /// keeps its ball.
    pub(super) fn fire(
        &mut self,
        side: u32,
        now_ms: u64,
        ship: &Ship,
        content: &Content,
        (aim, marks): (&Aim, &[Mark]),
        (shots, next_shot): (&mut VecDeque<Shot>, &mut u32),
    ) -> Vec<Vec<u8>> {
        let zone = |key: &str| content.zone_config.get(key).copied().unwrap_or(0.0);
        let gravity = zone("Gravity") as f32;
        let water = zone("WaterLevel") as f32;
        let model = (zone("ProjectileModelId") as u32, content.gunnery.ball_scale);
        let flight = content.gunnery.flight_seconds;
        let most = zone("ProjectileMax") as usize;
        let per_side = (ship.cannons.len() / 2).max(1) as f32;
        let [fx, _, fz] = aim.forward;
        self.reloads.resize(ship.cannons.len(), 0);
        let mut out = Vec::new();
        for (gun, ready) in ship.cannons.iter().zip(&mut self.reloads) {
            if *ready > now_ms || (gun.left && side == 2) || (!gun.left && side == 1) {
                continue;
            }
            if shots.len() >= most {
                break;
            }
            let crew = &mut self.crews[usize::from(gun.left)];
            let turn = (gun.reload_seconds * 1000.0 / per_side) as u64;
            *crew = (*crew).max(now_ms).saturating_add(turn);
            *ready = *crew;
            // The ship's frame: the left is the bow turned a quarter (the rows `create_ship` writes).
            let [ox, oy, oz] = gun.offset;
            let at = aim.at;
            let from = [at[0] + fz * ox + fx * oz, at[1] + oy, at[2] - fx * ox + fz * oz];
            let sign = if gun.left { 1.0 } else { -1.0 };
            let laid = lay(from, [fz * sign, -fx * sign], (gun.speed, flight), marks);
            let (vx, vz, time) = match laid {
                Some(([x, z], time)) => ((x - from[0]) / time, (z - from[2]) / time, time),
                None => (
                    aim.velocity[0] + fz * sign * gun.speed,
                    aim.velocity[2] - fx * sign * gun.speed,
                    flight,
                ),
            };
            // Up just enough to meet the water as the flight ends.
            let vy = (water - from[1]) / time + gravity * time / 2.0;
            let id = *next_shot;
            *next_shot = next_shot.wrapping_add(1).max(1);
            let velocity = [vx, vy, vz];
            let ball = (from, velocity, gravity);
            out.push(spawn_projectile(id, aim.slot, &gun.name, model, ball));
            shots.push_back(Shot {
                id,
                owner: aim.owner,
                from,
                velocity,
                fired_ms: now_ms,
                lands_ms: now_ms.saturating_add((time * 1000.0) as u64),
                at: [from[0] + vx * time, water, from[2] + vz * time],
                checked_ms: now_ms,
            });
        }
        out
    }

    /// The loaded guns as left and right masks.
    pub(super) fn loaded(&self, ship: &Ship, now_ms: u64) -> (u32, u32) {
        let (mut left, mut right) = (0, 0);
        for (index, gun) in ship.cannons.iter().enumerate() {
            if self.reloads.get(index).is_none_or(|ready| *ready <= now_ms) {
                *(if gun.left { &mut left } else { &mut right }) |= gun.bit;
            }
        }
        (left, right)
    }
}

/// What a ship has taken: the damage counts up from a full hull, so a new ship needs no setup.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Hull {
    pub(super) damage: i32,
    pub(super) sunk: bool,
    /// No ball counts until then: the grace after a respawn.
    pub(super) safe_until: u64,
}

impl Hull {
    /// Health left in `ship`.
    pub(super) fn health(&self, ship: &Ship) -> i32 {
        (ship.health - self.damage).max(0)
    }

    /// A ball strikes: true if that sank it.
    pub(super) fn strike(&mut self, ship: &Ship, damage: i32) -> bool {
        self.damage = self.damage.saturating_add(damage);
        self.sunk = self.health(ship) == 0;
        self.sunk
    }

    /// Whole again, with the respawn's grace.
    pub(super) fn repair(&mut self, safe_until: u64) {
        *self = Self { damage: 0, sunk: false, safe_until };
    }
}

/// ShipHit: the ship, the damage (0 is ignored), the side struck (1 left, 2 right: where the sparks
/// show, `sub_C9BEE0`) and 1 for a ball.
pub(super) fn ship_hit(guid: u64, damage: i32, side: u32) -> Vec<u8> {
    let mut body = guid.to_le_bytes().to_vec();
    body.extend(u32::try_from(damage).unwrap_or(0).to_le_bytes());
    body.extend(side.to_le_bytes());
    body.extend(1_u32.to_le_bytes());
    packet(SHIP_HIT, &body)
}

/// Which side of a ship at `at` with bow `forward` lies towards `ball`: 1 left, 2 right.
pub(super) fn side_of(at: [f32; 3], forward: [f32; 3], ball: [f32; 3]) -> u32 {
    let [fx, _, fz] = forward;
    if (ball[0] - at[0]) * fz - (ball[2] - at[2]) * fx >= 0.0 { 1 } else { 2 }
}

/// The seconds a sinking takes and the grace after a respawn, in milliseconds.
pub(super) fn sinking_ms(content: &Content) -> (u64, u64) {
    let zone = |key: &str| content.zone_config.get(key).copied().unwrap_or(0.0);
    let sink = zone("MaxSinkingTime") + zone("RespawnTime");
    ((sink * 1000.0) as u64, (zone("InvincibleAfterRespawnTime") * 1000.0) as u64)
}
