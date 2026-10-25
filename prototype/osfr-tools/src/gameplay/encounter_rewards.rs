//! Encounter payout application: one reward at a time, with the live wallet
//! and active job deciding whether the pair can be represented.
//!
//! The developer `realm win`, `realm exit` and `realm retry` routes also need
//! the encounter engine; a content set without one gets one bounded line back
//! instead of an `unwrap` that would disconnect the session.
//!
//! A field kill or battle win is already a server-authoritative fact when its
//! payout is applied. A pair the wallet or the active job cannot take (the wire
//! coin field is `i32`, experience is `u32`) is therefore refused whole, logged
//! and answered with one system line instead of faulting the shared-field tick
//! for every other player. The refusal says the reward, any prize item and any
//! chance drop it carried were not granted, so a capped player knows where the
//! item went.
use super::*;

/// (R) Local copy: no recovered client locale row exists for this refusal.
const REWARD_UNREPRESENTABLE: &str = "This reward, any prize item and any chance drop it carried were not granted: your coin balance or active job is at its limit.";

/// (R) The same for a developer route that needs an encounter engine and has none.
const NO_ACTIVE_ENCOUNTER: &str = "No encounter is active.";

/// (R) The same refusal for a finished derby's flat completion payout (`derby_session.rs`). No
/// recovered client locale row names it either, so it uses one bounded local line.
pub(super) const DERBY_PAYOUT_REFUSED: &str = "Your demolition derby completion reward was not granted: your coin balance or Demo Derby Driver progress is at its limit.";

impl Gameplay {
    /// The active encounter engine, or one bounded refusal. Every developer
    /// route that needs the engine goes through here instead of an `unwrap`.
    fn active_encounter(&mut self, out: &mut Outcome) -> Option<&mut EncounterEngine> {
        if self.encounter.is_none() {
            out.replies.push(chat::system(NO_ACTIVE_ENCOUNTER));
        }
        self.encounter.as_mut()
    }

    /// Apply the developer `realm win` route to the active encounter. A content
    /// set without an encounter engine refuses with feedback, never a panic.
    pub(super) fn developer_win(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let Some(encounter) = self.active_encounter(out) else {
            return Ok(());
        };
        let result = encounter.debug_win(now_ms)?;
        self.apply_encounter(result, out)
    }

    /// Apply the developer `realm exit` route to the active encounter. A content
    /// set without an encounter engine refuses with feedback, never a panic.
    pub(super) fn developer_exit(&mut self, out: &mut Outcome) -> io::Result<()> {
        let Some(encounter) = self.active_encounter(out) else {
            return Ok(());
        };
        let result = encounter.exit()?;
        self.apply_encounter(result, out)
    }

    /// Apply the developer `realm retry` route to a lost encounter. The engine
    /// ignores the job id today; pass the real active job if one is loaded.
    pub(super) fn developer_retry(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let active = self
            .profiles
            .as_ref()
            .map_or(0, |profiles| profiles.active());
        let Some(encounter) = self.active_encounter(out) else {
            return Ok(());
        };
        let result = encounter.retry(active, now_ms)?;
        self.apply_encounter(result, out)
    }

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
        // A chance drop lands only if the bag takes it whole; a full bag loses
        // the drop, never the win's coins, prize or XP (T658).
        for &drop in &reward.bonus_items {
            if self.inventory.accepts_reward_bundle(&[drop]) {
                self.grant_reward(0, &[drop], out)?;
            }
        }
        Ok(())
    }
}
