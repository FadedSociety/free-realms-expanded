//! Encounter payout application: one reward at a time, with the live wallet
//! and active job deciding whether the pair can be represented.
//!
//! A field kill or battle win is already a server-authoritative fact when its
//! payout is applied. A pair the wallet or the active job cannot take (the wire
//! coin field is `i32`, experience is `u32`) is therefore refused whole, logged
//! and answered with one system line instead of faulting the shared-field tick
//! for every other player. The refusal says the reward and any prize item it
//! carried were not granted, so a capped player knows where the item went.
use super::*;

/// (R) Local copy: no recovered client locale row exists for this refusal.
const REWARD_UNREPRESENTABLE: &str = "This reward and any prize item it carried were not granted: your coin balance or active job is at its limit.";

/// (R) The same refusal for a finished derby's flat completion payout (`derby_session.rs`). No
/// recovered client locale row names it either, so it uses one bounded local line.
pub(super) const DERBY_PAYOUT_REFUSED: &str = "Your demolition derby completion reward was not granted: your coin balance or Demo Derby Driver progress is at its limit.";

impl Gameplay {
    /// Apply one encounter reward. A reward that cannot be represented is
    /// refused whole for this player and leaves the rest of the outcome alone;
    /// inventory/item refusals keep the existing atomic `grant_bundle` rule.
    pub(super) fn apply_encounter_reward(
        &mut self,
        reward: crate::encounter::EncounterReward,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let coins_fit = self.inventory.can_grant_coins(reward.coins);
        let xp_fit = reward.job_xp == 0
            || self
                .profiles
                .as_ref()
                .is_some_and(|profiles| profiles.can_add_experience(reward.job_xp));
        if !coins_fit || !xp_fit {
            eprintln!(
                "encounter reward refused: coins={} job_xp={}",
                reward.coins, reward.job_xp
            );
            out.replies.push(chat::system(REWARD_UNREPRESENTABLE));
            return Ok(());
        }
        self.grant_reward(reward.coins, &reward.items, out)?;
        if reward.job_xp > 0 {
            self.award_job_xp(reward.job_xp, out)?;
        }
        Ok(())
    }
}
