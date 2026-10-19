//! Progress-driven achievements over authoritative gameplay state.
//!
//! Job rank and completed-quest counters are read from the same saved state the
//! rest of gameplay mutates. Only a counter that actually moved is observed, so
//! an ordinary request or tick emits no progress packet and clones no reward
//! state when nothing was earned. Every transaction that can move a counter
//! settles its achievements here: job XP (`apply_encounter`, mining/harvesting
//! and the derby), quest completion, and the minigame payouts applied by
//! `tick`. The request-only hook left tick and shared-field rewards unnoticed
//! until the next reconnect. A claim/title capacity failure defers the award to
//! a later transaction instead of failing the reward transaction, because the
//! same settlement runs inside the shared-field tick.
use super::*;

impl Gameplay {
    /// Data-bound counter values: one rank counter per unlocked job and the
    /// completed regional story quest total.
    pub(super) fn achievement_counters(&self) -> BTreeMap<String, u32> {
        let mut counters = BTreeMap::<String, u32>::new();
        if let Some(profiles) = &self.profiles {
            for profile in profiles
                .state()
                .unlocked
                .iter()
                .copied()
                .chain(profiles.state().experience.keys().copied())
            {
                if let Some(slug) = self.achievement_bindings.job_slug(profile) {
                    counters.insert(
                        format!("job_rank:{slug}"),
                        profiles.rank(profile).saturating_sub(1),
                    );
                }
            }
        }
        if let Some(quests) = &self.npc_quests {
            counters.insert(
                "quest_completed_count".into(),
                u32::try_from(
                    quests
                        .state()
                        .quests
                        .values()
                        .filter(|quest| quest.status == crate::quest_engine::QuestStatus::Completed)
                        .count(),
                )
                .unwrap_or(1_000_000),
            );
        }
        counters
    }

    /// Observe authoritative persisted progress. Job promotions and completed
    /// regional quests feed the data-bound achievement rules; only a newly
    /// completed rule emits packets, grants its title or marks the character
    /// changed (R). Progress-only observation stays session-local.
    ///
    /// A capacity refusal (claim inbox or title book full) defers the award to
    /// a later transaction instead of failing the reward transaction that moved
    /// the counter. `apply_encounter` also runs inside the shared-field tick,
    /// where an `Err` would freeze every player in the instance; the moved
    /// counter is re-observed by the next transaction once capacity exists.
    pub(super) fn sync_achievement_progress(
        &mut self,
        out: &mut Outcome,
        only: Option<&BTreeSet<String>>,
    ) -> io::Result<()> {
        if self.achievement_bindings.rules().is_empty() {
            return Ok(());
        }
        match self.try_sync_achievement_progress(out, only) {
            Ok(()) => Ok(()),
            Err(error) => {
                eprintln!("achievement: settlement deferred: {error}");
                Ok(())
            }
        }
    }

    /// Stage the whole completion before committing it. A refusal after one
    /// claim was issued would otherwise leave a partial award that can never be
    /// retried; a deferred award leaves no title, claim or counter behind.
    fn try_sync_achievement_progress(
        &mut self,
        out: &mut Outcome,
        only: Option<&BTreeSet<String>>,
    ) -> io::Result<()> {
        let counters = self.achievement_counters();
        let rewards = self.reward_snapshot();
        let before = self.progression_extras.achievement_state();
        let mut achievements = crate::progression_extras::AchievementProgression::new(
            self.achievement_bindings.rules().to_vec(),
            before,
        )?;
        let mut packets = Vec::new();
        let mut completed = Vec::new();
        for (counter, value) in counters {
            if only.is_some_and(|keys| !keys.contains(&counter)) {
                continue;
            }
            let transition = achievements.observe(&counter, value, &rewards)?;
            if !transition.newly_completed.is_empty() {
                packets.extend(transition.packets);
                packets.extend(transition.reward_packets);
                completed.extend(transition.newly_completed);
            }
        }
        if completed.is_empty() {
            return Ok(());
        }
        let mut staged = self.progression_extras.clone();
        for binding in self.achievement_bindings.claim_bindings() {
            if completed.contains(&binding.achievement_key) {
                staged.issue_server_grant(binding.claim_id, binding.grant.clone())?;
            }
        }
        packets.extend(staged.award_completed_achievement_titles(
            &completed,
            self.achievement_bindings.title_bindings(),
        )?);
        staged.set_achievement_state(achievements.state().clone());
        self.progression_extras = staged;
        out.replies.extend(packets);
        out.changed = true;
        Ok(())
    }

    /// Compare the counters with a snapshot taken before one authoritative
    /// transaction and observe only the ones that moved. A transaction that
    /// changed nothing relevant does no achievement work.
    pub(super) fn settle_achievement_counters(
        &mut self,
        before: &BTreeMap<String, u32>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let moved: BTreeSet<String> = self
            .achievement_counters()
            .into_iter()
            .filter(|(key, value)| before.get(key) != Some(value))
            .map(|(key, _)| key)
            .collect();
        if !moved.is_empty() {
            self.sync_achievement_progress(out, Some(&moved))?;
        }
        Ok(())
    }
}
