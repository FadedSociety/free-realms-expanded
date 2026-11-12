//! Why a quest giver cannot offer a quest now: the one gate list `offer_denial` and `accept`
//! read, kept finer than [`QuestDenial`] so an NPC click that offers nothing logs each quest's
//! reason (`npc_select ... no_eligible_quest`, rc-0136: Caitlyn, Petal and Smoggo said nothing
//! and the log could not say why). A child of `quest_engine`.
use super::*;

/// One quest's block. `denial` is what the client-facing paths have always answered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum OfferBlock {
    Unknown,
    NoSteps,
    Disabled,
    /// T418: a live event no admin has opened.
    LiveEventOff,
    WrongGiver,
    /// Another quest's offer is still open.
    OtherOffer,
    Offered,
    Active,
    Done,
    /// A repeatable quest finished less than `repeat_hours` ago: offered again from `until`
    /// (unix seconds).
    Resting {
        until: u64,
    },
    /// Another quest must be finished `need` times first (Halloween Fanatic).
    Completions {
        quest: u32,
        need: u32,
        done: u32,
    },
    /// T465: an exclusive branch already taken.
    Excluded {
        by: u32,
    },
    /// The quests still to finish first.
    Prerequisite {
        missing: Vec<u32>,
    },
    Level {
        need: u8,
        have: u8,
    },
    /// The quest's job: `learned` false when the character never learned it.
    Job {
        job: String,
        learned: bool,
        need: u8,
        have: u8,
    },
    Membership,
    Limit,
}

impl OfferBlock {
    pub fn denial(&self) -> QuestDenial {
        match self {
            Self::WrongGiver => QuestDenial::WrongGiver,
            Self::Prerequisite { .. } | Self::Completions { .. } => QuestDenial::Prerequisite,
            Self::Level { .. } => QuestDenial::Level,
            Self::Job { .. } => QuestDenial::Job,
            Self::Membership => QuestDenial::Membership,
            Self::Limit => QuestDenial::Limit,
            _ => QuestDenial::Unavailable,
        }
    }
}

impl QuestEngine {
    /// Why `giver_npc_id` cannot offer `quest_id` now; `None` when it can.
    pub fn offer_block(&self, quest_id: u32, giver_npc_id: u64) -> Option<OfferBlock> {
        let Some(definition) = self.definitions.get(&quest_id) else {
            return Some(OfferBlock::Unknown);
        };
        // (R) Incomplete content stays available to editors and saved state, but cannot be
        // offered, even when `enabled` is accidentally true.
        if definition.objectives.is_empty() {
            return Some(OfferBlock::NoSteps);
        }
        if self.disabled.contains(&quest_id) {
            return Some(OfferBlock::Disabled);
        }
        if self.event_closed(quest_id) {
            return Some(OfferBlock::LiveEventOff);
        }
        if definition.giver_npc_id != giver_npc_id {
            return Some(OfferBlock::WrongGiver);
        }
        if (self.state.pending_offer).is_some_and(|pending| pending != quest_id) {
            return Some(OfferBlock::OtherOffer);
        }
        match self
            .state
            .quests
            .get(&quest_id)
            .map(|progress| &progress.status)
        {
            Some(QuestStatus::Offered) => return Some(OfferBlock::Offered),
            Some(QuestStatus::Active) => return Some(OfferBlock::Active),
            // A repeatable quest comes round again `repeat_hours` after its last run.
            Some(QuestStatus::Completed) => match self.repeat_ready_at(quest_id) {
                Some(until) if until > self.now_secs() => {
                    return Some(OfferBlock::Resting { until });
                }
                Some(_) => {}
                None => return Some(OfferBlock::Done),
            },
            _ => {}
        }
        if let Some(block) = self.requirement_block(definition) {
            return Some(block);
        }
        let limit = if self.eligibility.member {
            self.eligibility.member_quest_limit
        } else {
            self.eligibility.nonmember_quest_limit
        };
        let occupied = (self.state.quests.values())
            .filter(|progress| {
                matches!(progress.status, QuestStatus::Offered | QuestStatus::Active)
            })
            .count();
        (occupied >= limit).then_some(OfferBlock::Limit)
    }

    /// The gates `offer_block` and `accept` share, after the quest's own status.
    pub(super) fn requirement_block(&self, definition: &QuestDefinition) -> Option<OfferBlock> {
        let status = |id: &u32| self.state.quests.get(id).map(|progress| &progress.status);
        // Ever finished: a repeatable prerequisite on another run still counts.
        let done = |id: &u32| self.state.is_done(*id);
        // T465: an excluded quest that is offered, active or completed closes this one, checked
        // before the prerequisite (Lxzxrus b84b467: two exclusive branches behind one
        // prerequisite were both offered). Abandoned reopens (R).
        let taken = (self.exclusions.get(&definition.id).into_iter().flatten()).find(|id| {
            matches!(
                status(id),
                Some(QuestStatus::Offered | QuestStatus::Active | QuestStatus::Completed)
            )
        });
        if let Some(&by) = taken {
            return Some(OfferBlock::Excluded { by });
        }
        let mut missing: Vec<u32> = (definition.prerequisite_quest_ids.iter())
            .filter(|id| !done(id))
            .copied()
            .collect();
        if let Some(any) = self.any_prerequisites.get(&definition.id)
            && !any.iter().any(done)
        {
            missing.extend(any.iter().copied());
        }
        if !missing.is_empty() {
            return Some(OfferBlock::Prerequisite { missing });
        }
        if let Some((quest, need, done)) = self.completions_short(definition.id) {
            return Some(OfferBlock::Completions { quest, need, done });
        }
        let have = self.eligibility.player_level;
        if let Some(need) = definition
            .requirements
            .player_level
            .filter(|need| have < *need)
        {
            return Some(OfferBlock::Level { need, have });
        }
        if let Some((job, need)) = &definition.requirements.job {
            let have = self.eligibility.job_levels.get(job).copied();
            if have.unwrap_or(0) < *need {
                return Some(OfferBlock::Job {
                    job: job.clone(),
                    learned: have.is_some(),
                    need: *need,
                    have: have.unwrap_or(0),
                });
            }
        }
        if definition.requirements.members_only && !self.eligibility.member {
            return Some(OfferBlock::Membership);
        }
        None
    }

    /// The job (profile id) a job quest belongs to, which accepting it switches the player into.
    pub fn required_profile(&self, quest_id: u32) -> Option<i32> {
        let (job, _) = self.definitions.get(&quest_id)?.requirements.job.as_ref()?;
        self.job_profile_ids.get(job).copied()
    }
}
