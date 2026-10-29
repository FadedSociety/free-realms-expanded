//! (R) Deterministic seams for tests and offline fixtures. The gateway loads tables from OS
//! entropy and follows the wall clock, and never calls any of these.
use super::*;

impl Gameplay {
    /// Deterministic chess replies.
    pub fn seed_chess(&mut self, seed: u64) {
        if let Some(chess) = &mut self.chess {
            chess.reseed(seed);
        }
    }

    /// A fixed UTC day for the chess reward ledger.
    pub fn seed_chess_day(&mut self, day: u64) {
        if let Some(chess) = &mut self.chess {
            chess.set_day(day);
        }
    }

    /// T560: a fixed server clock (Unix seconds) for the daily wheel's day boundary.
    pub fn seed_wheel_clock(&mut self, unix_seconds: u64) {
        self.daily_wheel.set_clock(unix_seconds);
    }
}
