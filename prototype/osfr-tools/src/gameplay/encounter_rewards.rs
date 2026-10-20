//! Encounter payout application: one reward at a time, with the live wallet
//! and active job deciding whether the pair can be represented.
//!
//! A field kill is already a server-authoritative fact when its payout is
//! applied. A pair the wallet or the active job cannot take (the wire coin
//! field is `i32`, experience is `u32`) is therefore refused alone and logged
//! instead of faulting the shared-field tick for every other player.
use super::*;

impl Gameplay {
    /// Apply one encounter reward. A reward that cannot be represented is
    /// refused for this player and leaves the rest of the outcome alone;
    /// inventory/item refusals keep the existing atomic `grant_bundle` rule.
    pub(super) fn apply_encounter_reward(
        &mut self,
        reward: crate::encounter::EncounterReward,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let coins_fit = self
            .inventory
            .state()
            .coins
            .checked_add(reward.coins)
            .is_some_and(|coins| coins <= i32::MAX as u32);
        let xp_fit = reward.job_xp == 0
            || self.profiles.as_ref().is_some_and(|profiles| {
                profiles
                    .state()
                    .experience
                    .get(&profiles.active())
                    .copied()
                    .unwrap_or(0)
                    .checked_add(reward.job_xp)
                    .is_some()
            });
        if !coins_fit || !xp_fit {
            eprintln!(
                "encounter reward refused: coins={} job_xp={}",
                reward.coins, reward.job_xp
            );
            return Ok(());
        }
        self.grant_reward(reward.coins, &reward.items, out)?;
        if reward.job_xp > 0 {
            self.award_job_xp(reward.job_xp, out)?;
        }
        Ok(())
    }
}
