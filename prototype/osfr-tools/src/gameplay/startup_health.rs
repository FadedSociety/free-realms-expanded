//! The active job's level sets the encounter's max health, and a world reload's startup stream
//! (index 33, 38/1) reports that health instead of the captured 400/400 (T437).
use super::*;

impl Gameplay {
    /// Max health and weapon damage follow the active combat job's level
    /// (final-era scaling in the encounter engine); other jobs stay at the
    /// level-1 baseline.
    pub(super) fn sync_player_level(&mut self, out: &mut Outcome) {
        let Some(profile) = self.profiles.as_ref().map(Profiles::active) else {
            return;
        };
        let combat = self
            .abilities
            .as_ref()
            .is_some_and(|a| a.supports_basic(profile));
        let level = if combat { self.job_level(profile) } else { 1 };
        if let Some(encounter) = self.encounter.as_mut() {
            out.replies.extend(encounter.set_player_job(profile, level));
            self.health_synced = true;
        }
    }

    /// The encounter's (current, max) health once `sync_player_level` has run. Before the first
    /// world init the engine still holds its level-1 default, so this is `None` and the gateway
    /// keeps the captured pair.
    pub fn startup_health(&self) -> Option<(u32, u32)> {
        let encounter = self.encounter.as_ref().filter(|_| self.health_synced)?;
        Some(encounter.player_health())
    }
}
