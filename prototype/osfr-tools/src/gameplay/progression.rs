//! Progress-driven achievements over authoritative gameplay state.
//!
//! Job rank and completed-quest counters are read from the same saved state the
//! rest of gameplay mutates. Only a counter that actually moved is observed, so
//! an ordinary request or tick emits no progress packet and clones no reward
//! state when nothing was earned. Every transaction that can move a counter
//! settles its achievements here: job XP (`apply_encounter`, mining/harvesting
//! and the derby), quest completion, and the minigame payouts applied by
//! `tick`. The request-only hook left tick and shared-field rewards unnoticed
//! until the next reconnect.
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
    pub(super) fn sync_achievement_progress(
        &mut self,
        out: &mut Outcome,
        only: Option<&BTreeSet<String>>,
    ) -> io::Result<()> {
        if self.achievement_bindings.rules().is_empty() {
            return Ok(());
        }
        let counters = self.achievement_counters();
        let rewards = self.reward_snapshot();
        let before = self.progression_extras.achievement_state();
        let mut achievements = crate::progression_extras::AchievementProgression::new(
            self.achievement_bindings.rules().to_vec(),
            before.clone(),
        )?;
        let mut completed = Vec::new();
        for (counter, value) in counters {
            if only.is_some_and(|keys| !keys.contains(&counter)) {
                continue;
            }
            let transition = achievements.observe(&counter, value, &rewards)?;
            if !transition.newly_completed.is_empty() {
                out.replies.extend(transition.packets);
                out.replies.extend(transition.reward_packets);
                completed.extend(transition.newly_completed);
            }
        }
        let changed = !completed.is_empty();
        let claim_bindings = self.achievement_bindings.claim_bindings().to_vec();
        for binding in &claim_bindings {
            if completed.contains(&binding.achievement_key) {
                self.issue_claim(binding.claim_id, binding.grant.clone())?;
            }
        }
        out.replies
            .extend(self.progression_extras.award_completed_achievement_titles(
                &completed,
                self.achievement_bindings.title_bindings(),
            )?);
        let after = achievements.state().clone();
        if changed {
            self.progression_extras.set_achievement_state(after);
            out.changed = true;
        }
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
