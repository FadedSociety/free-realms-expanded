//! (R) Test/diagnostic seed for ground pickups. Production never calls it; the F261
//! regression seeds through the real drop roll, then drives the real revive commit.
use super::Gateway;
use crate::gameplay::Gameplay;

impl Gateway {
    /// (R) Test/diagnostic: seed one ground pickup through the real drop roll
    /// (see `EncounterEngine::seed_power_up_drop`). The gateway never calls this.
    pub fn seed_power_up_drop(&mut self, table: &str, at: [f32; 3], now_ms: u64) {
        if let Some(engine) = self.gameplay.as_mut().and_then(Gameplay::encounter_mut) {
            engine.seed_power_up_drop(table, at, now_ms);
        }
    }
}
