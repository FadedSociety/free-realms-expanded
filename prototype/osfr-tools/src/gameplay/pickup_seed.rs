//! (R) Test/diagnostic surface for this session's ground pickups: a read-only count and
//! one seed through the real drop roll (`EncounterEngine::drop_power_ups`). The seed
//! grants nothing and is unreachable from a packet; only the F261 regression calls it.
use super::Gameplay;

impl Gameplay {
    /// How many ground pickups this session still holds (tests and diagnostics).
    pub fn pickup_count(&self) -> usize {
        self.encounter
            .as_ref()
            .map_or(0, crate::encounter::EncounterEngine::pickup_count)
    }

    /// The encounter engine, so a test can seed through the real drop path.
    pub(crate) fn encounter_mut(&mut self) -> Option<&mut crate::encounter::EncounterEngine> {
        self.encounter.as_mut()
    }
}
