//! L955: the soccer ball in flight and on the turf: gravity, air drag, bounce and grass friction,
//! stepped in 10 ms substeps. Before, a ground ball rolled on at its kick speed until something
//! stopped it (owner playtest 2026-11-03: passes ran in straight lines across the pitch).
//!
//! Adapted from League-Soccer (github.com/awest813/League-Soccer, src/onthepitch/ballphysics.cpp
//! `ApplyBallMotionForces` and the 10 ms step of ball.cpp), Apache-2.0, of the GameplayFootball /
//! Google Research Football lineage. Their z (up) is our y; their ground is our pitch floor, and
//! our ball position is the ball's lowest point (it rests on the floor), so the centre height they
//! work with is `y - floor + radius`.
//!
//! The coefficients are not theirs: their drag and grass friction grow with speed squared and stop
//! a 30 m/s ground shot within 25 m (the live pitch is 54 m). The live server's ball (76/4 every
//! 0.155 s in the soccer capture) loses a fixed share of its speed instead: 0.966 per update in
//! the air, about 0.93 rolling, with gravity 9.81 and bounces keeping about 0.75. So drag and
//! grass friction here are linear in speed, fitted to those updates (R: a dozen flights).
use serde::Deserialize;
use std::io;

/// The ball's tuning (fitted to the live ball; all tunable in `soccer.json` `match_config.ball`).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct BallTuning {
    pub radius: f32,
    /// Vertical speed kept on a bounce, then `linear_bounce` m/s less.
    pub bounce: f32,
    pub linear_bounce: f32,
    /// Air drag: the ball loses `drag` of its speed per second.
    pub drag: f32,
    /// Grass friction on a low ball: ground speed loses `friction * speed + linear_friction` per
    /// second, scaled by how deep in the grass the ball sits. `linear_friction` stops it.
    pub friction: f32,
    pub linear_friction: f32,
    pub grass_height: f32,
    /// Seconds per physics substep.
    pub substep: f32,
}

impl Default for BallTuning {
    fn default() -> Self {
        Self {
            radius: 0.11,
            // Live peaks after each bounce fell to 0.5-0.7 of the last (R).
            bounce: 0.75,
            // League-Soccer's (R for our ball).
            linear_bounce: 0.05,
            // Live: 0.966 of the speed kept per 0.155 s update in the air.
            drag: 0.225,
            // Live: about 0.93 per update rolling (R: two updates); the constant part is ours (R).
            friction: 0.24,
            linear_friction: 0.3,
            grass_height: 0.025,
            substep: 0.01,
        }
    }
}

impl BallTuning {
    pub fn validate(&self) -> io::Result<()> {
        let values = [
            self.radius,
            self.bounce,
            self.linear_bounce,
            self.drag,
            self.friction,
            self.linear_friction,
            self.grass_height,
            self.substep,
        ];
        if values.iter().any(|v| !v.is_finite() || *v < 0.0)
            || self.radius <= 0.0
            || self.grass_height <= 0.0
            || self.bounce > 1.0
            || !(0.001..=0.05).contains(&self.substep)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "soccer ball tuning is out of range",
            ));
        }
        Ok(())
    }
}

/// One substep of the forces on (`centre` height `h` above the floor, `velocity`), as
/// `ApplyBallMotionForces`: gravity, drag, the bounce, then grass friction on a low ball.
fn forces(h: &mut f32, velocity: &mut [f32; 3], gravity: f32, t: &BallTuning, dt: f32) {
    velocity[1] -= gravity * dt;
    let kept = (1.0 - t.drag * dt).max(0.0);
    for v in velocity.iter_mut() {
        *v *= kept;
    }
    let bottom = *h - t.radius;
    let grass = (1.0 - bottom / t.grass_height).clamp(0.0, 1.0).powf(0.7);
    if *h < t.radius {
        if velocity[1] < 0.0 {
            velocity[1] = (-velocity[1] * t.bounce - t.linear_bounce).max(0.0);
        }
        *h = t.radius;
    }
    if *h < t.radius + t.grass_height {
        let ground = (velocity[0] * velocity[0] + velocity[2] * velocity[2]).sqrt();
        if ground > 0.0 {
            let slowed = (ground - (t.friction * ground + t.linear_friction) * grass * dt).max(0.0);
            velocity[0] *= slowed / ground;
            velocity[2] *= slowed / ground;
        }
    }
}

/// Move the ball `dt` seconds: forces, then position, every `substep`. `position` is the ball's
/// lowest point, `floor` the pitch height and `gravity` positive (m/s^2, down).
pub fn advance(
    position: &mut [f32; 3],
    velocity: &mut [f32; 3],
    floor: f32,
    gravity: f32,
    tuning: &BallTuning,
    dt: f32,
) {
    let mut h = position[1] - floor + tuning.radius;
    let mut left = dt;
    while left > 0.0 {
        let step = left.min(tuning.substep);
        forces(&mut h, velocity, gravity, tuning, step);
        position[0] += velocity[0] * step;
        h += velocity[1] * step;
        position[2] += velocity[2] * step;
        left -= step;
    }
    h = h.max(tuning.radius);
    // A ball on the turf micro-bounces under gravity every substep (as theirs does), and the floor
    // snap keeps feeding it; once its bounce would rise less than REST_HOP, it rests (the client
    // gets no vertical jitter).
    let hop = h - tuning.radius + velocity[1] * velocity[1] / (2.0 * gravity.max(f32::EPSILON));
    if hop < REST_HOP {
        velocity[1] = 0.0;
        h = tuning.radius;
    }
    position[1] = floor + (h - tuning.radius);
}

/// The bounce height (m) under which a ball rests on the turf (R).
const REST_HOP: f32 = 0.02;

#[cfg(test)]
mod tests {
    use super::*;

    fn roll(speed: f32, seconds: f32) -> ([f32; 3], [f32; 3]) {
        let (mut p, mut v) = ([0.0; 3], [speed, 0.0, 0.0]);
        let t = BallTuning::default();
        for _ in 0..(seconds / 0.25) as usize {
            advance(&mut p, &mut v, 0.0, 9.81, &t, 0.25);
        }
        (p, v)
    }

    #[test]
    fn a_ground_pass_slows_and_stops_on_the_grass() {
        // 14 m/s, the bots' pass: about 26 m, then still (closed form with a = 0.465, b = 0.3:
        // v0 / a - b / a^2 ln(1 + a v0 / b)).
        let (p, v) = roll(14.0, 10.0);
        assert!((22.0..30.0).contains(&p[0]), "rolled {}", p[0]);
        assert_eq!(v, [0.0; 3], "stopped");
        assert!(p[1].abs() < 1e-4, "stays on the floor");
    }

    #[test]
    fn a_full_power_shot_from_the_centre_reaches_the_goal_with_pace() {
        // The live pitch is 54 m long; a 30 m/s ground shot from the centre spot crosses 27 m still
        // fast enough to beat a keeper (live shots crossed the line at about 20 m/s).
        let (mut p, mut v) = ([0.0; 3], [30.0, 0.0, 0.0]);
        let t = BallTuning::default();
        while p[0] < 27.0 && v[0] > 0.0 {
            advance(&mut p, &mut v, 0.0, 9.81, &t, 0.05);
        }
        assert!(v[0] > 12.0, "arrives at {}", v[0]);
    }

    #[test]
    fn a_ball_in_the_air_keeps_what_the_live_ball_kept() {
        // One live update apart (0.155 s): 0.966 of the horizontal speed, about 1.52 m/s fallen.
        let (mut p, mut v) = ([0.0, 3.0, 0.0], [-11.12, 2.66, 1.51]);
        advance(&mut p, &mut v, 0.0, 9.81, &BallTuning::default(), 0.155);
        assert!((v[0] / -11.12 - 0.966).abs() < 0.005, "{v:?}");
        assert!((v[1] - 1.07).abs() < 0.1, "live 1.07: {v:?}");
    }

    #[test]
    fn a_lofted_ball_bounces_lower_each_time_and_settles() {
        let (mut p, mut v) = ([0.0, 0.0, 0.0], [4.0, 8.0, 0.0]);
        let t = BallTuning::default();
        let mut peaks = Vec::new();
        let mut rising = true;
        let mut last = 0.0f32;
        for _ in 0..700 {
            advance(&mut p, &mut v, 0.0, 9.81, &t, 0.02);
            if rising && p[1] < last {
                peaks.push(last);
            }
            rising = p[1] >= last;
            last = p[1];
        }
        assert!(peaks.len() >= 4, "it bounces: {peaks:?}");
        // About bounce^2 (0.56) of the last height while the bounces are real; the sub-centimetre
        // tail is sampling noise.
        assert!(
            peaks[..4].windows(2).all(|w| w[1] < w[0] * 0.65),
            "each bounce lower: {peaks:?}"
        );
        assert!(p[1] < 0.01 && v[0] == 0.0, "settled: {p:?} {v:?}");
    }

    #[test]
    fn tuning_is_bounded() {
        assert!(BallTuning::default().validate().is_ok());
        let bad = BallTuning {
            bounce: 1.5,
            ..BallTuning::default()
        };
        assert!(bad.validate().is_err());
        let slow = BallTuning {
            substep: 0.5,
            ..BallTuning::default()
        };
        assert!(slow.validate().is_err());
    }
}
