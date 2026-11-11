//! G3: the quest engine's staff setter, compiled as a child of `quest_engine` (which includes
//! this file) so an edit keeps the engine's own bookkeeping (pending offer and turn-in, the
//! selection, the revision the quest markers watch), sends the client messages the player's own
//! accept, turn-in and abandon send, and is checked by the same `validate_state` a login runs.
use super::*;
use crate::character_admin::QuestOp;

impl QuestEngine {
    /// The record a staff quest op leaves, or the reason it does not apply.
    pub fn admin_target(
        &self,
        quest_id: u32,
        op: QuestOp,
    ) -> Result<Option<QuestProgress>, String> {
        let definition = self
            .definitions
            .get(&quest_id)
            .ok_or_else(|| format!("quest {quest_id} is not in the quest data"))?;
        let current = self.state.quests.get(&quest_id);
        let status = current.map(|progress| &progress.status);
        Ok(match op {
            QuestOp::Grant => {
                if !self.is_enabled(quest_id) {
                    return Err(format!("quest {quest_id} is disabled or has no steps"));
                }
                if matches!(status, Some(QuestStatus::Active | QuestStatus::Completed)) {
                    return Err(format!("quest {quest_id} is already active or done"));
                }
                Some(QuestProgress {
                    status: QuestStatus::Active,
                    counts: vec![0; definition.objectives.len()],
                })
            }
            QuestOp::Complete => {
                if status == Some(&QuestStatus::Completed) {
                    return Err(format!("quest {quest_id} is already done"));
                }
                // A disabled quest is kept only as history, with no counters (`validate_state`).
                let counts = if self.disabled.contains(&quest_id) {
                    Vec::new()
                } else {
                    definition.objectives.iter().map(|o| o.required).collect()
                };
                Some(QuestProgress {
                    status: QuestStatus::Completed,
                    counts,
                })
            }
            QuestOp::Reset => {
                if current.is_none() {
                    return Err(format!("the character has no record of quest {quest_id}"));
                }
                None
            }
            QuestOp::Abandon => match current {
                Some(progress) if progress.status == QuestStatus::Active => Some(QuestProgress {
                    status: QuestStatus::Abandoned,
                    counts: progress.counts.clone(),
                }),
                _ => return Err(format!("quest {quest_id} is not active")),
            },
        })
    }

    /// Move `quest_id`'s record from `from` to `to` (`None` = no record). Refused, changing
    /// nothing, when the record is no longer `from` (a stale undo), when `to` is an open offer,
    /// when a new active quest would pass the journal limit, or when the result would not load.
    /// No reward is paid: a completed quest's turn-in is skipped.
    pub fn admin_set_progress(
        &mut self,
        quest_id: u32,
        from: Option<&QuestProgress>,
        to: Option<QuestProgress>,
    ) -> Result<QuestOutcome, String> {
        let definition = self
            .definitions
            .get(&quest_id)
            .ok_or_else(|| format!("quest {quest_id} is not in the quest data"))?;
        if self.state.quests.get(&quest_id) != from {
            return Err(format!("quest {quest_id} changed since; nothing changed"));
        }
        let status = |progress: Option<&QuestProgress>| progress.map(|p| p.status.clone());
        let (was, now) = (status(from), status(to.as_ref()));
        let active = |status: &Option<QuestStatus>| *status == Some(QuestStatus::Active);
        let done = |status: &Option<QuestStatus>| *status == Some(QuestStatus::Completed);
        if now == Some(QuestStatus::Offered) {
            return Err("an open quest offer cannot be set".into());
        }
        let mut next = self.state.clone();
        if active(&now) && !active(&was) {
            let limit = if self.eligibility.member {
                self.eligibility.member_quest_limit
            } else {
                self.eligibility.nonmember_quest_limit
            };
            let open = next
                .quests
                .values()
                .filter(|p| p.status == QuestStatus::Active)
                .count();
            if open >= limit {
                return Err(format!("the quest journal is full ({limit} active)"));
            }
        }
        let mut outbound = Vec::new();
        if active(&was) && !active(&now) {
            outbound.push(if done(&now) {
                QuestMessage::QuestComplete { quest_id }
            } else {
                QuestMessage::QuestAbandoned { quest_id }
            });
        }
        if active(&now) && !active(&was) {
            // As `accept` opens a quest: 49/3, then the open steps (only the first when ordered).
            outbound.push(QuestMessage::QuestAdd { quest_id });
            outbound.extend(
                definition
                    .objectives
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !definition.ordered || *index == 0)
                    .map(|(_, objective)| QuestMessage::ObjectiveActivated {
                        quest_id,
                        objective_id: objective.id,
                        required: objective.required,
                    }),
            );
        }
        match to {
            Some(progress) => next.quests.insert(quest_id, progress),
            None => next.quests.remove(&quest_id),
        };
        if next.pending_offer == Some(quest_id) {
            next.pending_offer = None;
        }
        if !active(&now) {
            if next.pending_turnin == Some(quest_id) {
                next.clear_pending_turnin();
            }
            if next.selected_quest_id == Some(quest_id) {
                next.selected_quest_id = None;
            }
        }
        validate_state(&next, self.character_id, &self.definitions, &self.disabled)
            .map_err(|error| format!("quest {quest_id} would not load: {error}"))?;
        if done(&was) != done(&now) {
            let count = next
                .quests
                .values()
                .filter(|p| p.status == QuestStatus::Completed)
                .count();
            outbound.push(QuestMessage::CompletedQuestCount {
                count: u32::try_from(count).unwrap_or(u32::MAX),
            });
        }
        if self.dev_offer == Some(quest_id) {
            self.dev_offer = None;
        }
        self.state = next;
        self.bump();
        Ok(QuestOutcome {
            changed: true,
            outbound: outbound.into_iter().map(OutboundMessage::Quest).collect(),
            ..QuestOutcome::default()
        })
    }

    /// The quests a staff Complete of `quest_id` finishes first: every quest its prerequisites
    /// name, theirs in turn, earliest first, that is not done yet. Only the all-of lists: an
    /// any-of gate names alternative branches, and finishing one would close the others (R).
    pub fn admin_prerequisite_chain(&self, quest_id: u32) -> Vec<u32> {
        fn visit(engine: &QuestEngine, id: u32, seen: &mut BTreeSet<u32>, out: &mut Vec<u32>) {
            if !seen.insert(id) {
                return;
            }
            let earlier = engine
                .definitions
                .get(&id)
                .map(|d| d.prerequisite_quest_ids.as_slice());
            for &before in earlier.unwrap_or_default() {
                visit(engine, before, seen, out);
            }
            out.push(id);
        }
        let mut chain = Vec::new();
        visit(self, quest_id, &mut BTreeSet::new(), &mut chain);
        let done = |id: &u32| {
            (self.state.quests.get(id)).is_some_and(|p| p.status == QuestStatus::Completed)
        };
        chain.retain(|id| *id != quest_id && self.definitions.contains_key(id) && !done(id));
        chain
    }

    /// The quest's authored title, for the staff views.
    pub fn admin_title(&self, quest_id: u32) -> Option<&str> {
        Some(self.definitions.get(&quest_id)?.title.as_str())
    }
}
