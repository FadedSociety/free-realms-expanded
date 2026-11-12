//! Repeatable quests and completion counts (owner 2026-11-11: a "Repeatable" checkbox and
//! "every N hours", 24 being daily; fanbyte: Going Batty "is repeatable once a day"). A child of
//! `quest_engine`.
//!
//! A repeatable quest stays `Completed` until `repeat_hours` after its last completion, then
//! `offer_block` lets its giver offer it again; the offer, accept and turn-in are the ordinary
//! ones and the reward pays again. The save keeps each repeatable quest's last completion and how
//! often it was done (`QuestState::repeats`), so the wait outlives a relog and a later quest can
//! need a count (Halloween Fanatic: Trick-or-Treat three times). The client has no repeat flag:
//! the offer screen is `QuestStartHandler:Show(NPCDialog, Name, Description, MembersOnly,
//! undeclinable)` with the fixed card icon 454 (ScriptsBase.lua), 49/4 removes the finished
//! quest's record (`DispatchQuestPacket` 00c7cd40 case 4, `RemoveQuestRecord`) and 49/3 builds a
//! fresh one (`ApplyQuestAddPacket` 00c7bd10), so a repeat is the same packets again. Nothing
//! moves the marker revision on a timer, so the session's once-a-second event tick asks
//! [`QuestEngine::tick_repeats`] and a quest coming due draws its giver's (!) again.
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

const HOUR: u64 = 3_600;
/// Sanity bound on a completion-count gate, not a game rule.
const MAX_NEEDED: u32 = 1_000;

/// One repeatable quest's completions: the last one (unix seconds) and how many.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuestRepeat {
    pub last: u64,
    pub times: u32,
}

/// Quest id to its [`QuestRepeat`]; boxed so the session state stays one pointer wider.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Repeats(Box<BTreeMap<u32, QuestRepeat>>);

impl Repeats {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, quest_id: u32) -> Option<QuestRepeat> {
        self.0.get(&quest_id).copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert(&mut self, quest_id: u32, repeat: QuestRepeat) {
        self.0.insert(quest_id, repeat);
    }
}

/// The catalogue's repeat and completion-count rules, and the clock they are judged at.
#[derive(Clone, Debug, Default)]
pub(super) struct Rules {
    hours: BTreeMap<u32, u32>,
    /// Quest id to its `event_calendar.json` key (`content::LiveEvent::Calendar`).
    event_keys: BTreeMap<u32, String>,
    /// Quest id to (prerequisite quest, completions it needs).
    needs: BTreeMap<u32, Vec<(u32, u32)>>,
    /// Tests pin the clock (unix seconds); `None` is the wall clock.
    clock: Option<u64>,
    /// When the next finished repeatable quest comes due (unix seconds), if one waits.
    wake: Option<u64>,
}

impl QuestState {
    /// Whether the quest was ever completed: `Completed`, or a repeatable quest on another run.
    pub fn is_done(&self, quest_id: u32) -> bool {
        self.repeats.get(quest_id).is_some()
            || (self.quests.get(&quest_id)).is_some_and(|p| p.status == QuestStatus::Completed)
    }

    /// How many times the quest was completed. A non-repeatable quest counts once; a save
    /// written before completions were counted counts a finished quest once (R).
    pub fn completions(&self, quest_id: u32) -> u32 {
        match self.repeats.get(quest_id) {
            Some(repeat) => repeat.times,
            None => u32::from(self.is_done(quest_id)),
        }
    }

    /// The quests ever completed, each once (49/12 and the quest-count achievements): a
    /// repeatable quest taken again does not lower it.
    pub fn completed_count(&self) -> usize {
        let finished = self.quests.iter().filter(|(_, p)| p.status == QuestStatus::Completed);
        let running = self.repeats.0.keys().filter(|id| {
            (self.quests.get(id)).is_none_or(|p| p.status != QuestStatus::Completed)
        });
        finished.count() + running.count()
    }
}

impl QuestEngine {
    /// Read each record's repeat, event key and completion-count rules.
    pub(super) fn set_rules(&mut self, quests: &[crate::content::Quest]) -> io::Result<()> {
        let mut rules = Rules {
            clock: self.rules.clock,
            ..Rules::default()
        };
        for quest in quests {
            if let Some(hours) = quest.repeat_rule().map_err(invalid)? {
                rules.hours.insert(quest.id, hours);
            }
            if let Some(key) = quest.live_event.key() {
                let sane = (1..=32).contains(&key.len())
                    && key.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
                if !sane {
                    return Err(invalid("quest live_event key must be a calendar event key"));
                }
                rules.event_keys.insert(quest.id, key.to_owned());
            }
            for (&other, &times) in &quest.prerequisite_completions {
                if other == quest.id || !self.definitions.contains_key(&other) || !(1..=MAX_NEEDED).contains(&times) {
                    return Err(invalid("prerequisite_completions needs other known quests, 1 to 1000 times"));
                }
                rules.needs.entry(quest.id).or_default().push((other, times));
            }
        }
        self.rules = Box::new(rules);
        self.rewake();
        Ok(())
    }

    pub(super) fn now_secs(&self) -> u64 {
        self.rules.clock.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_secs())
        })
    }

    /// Pin the clock repeats are judged at (unix seconds; `None`: the wall clock).
    pub fn set_clock(&mut self, unix_secs: Option<u64>) {
        self.rules.clock = unix_secs;
        self.rewake();
    }

    pub fn repeat_hours(&self, quest_id: u32) -> Option<u32> {
        self.rules.hours.get(&quest_id).copied()
    }

    /// Every calendar-gated quest and its event key.
    pub fn event_quests(&self) -> impl Iterator<Item = (u32, &str)> {
        (self.rules.event_keys.iter()).map(|(id, key)| (*id, key.as_str()))
    }

    /// When a finished repeatable quest may be offered again (unix seconds); `None` for any
    /// other quest. A finished quest with no count yet (an older save) is due at once.
    pub(super) fn repeat_ready_at(&self, quest_id: u32) -> Option<u64> {
        let hours = self.repeat_hours(quest_id)?;
        let finished = self.state.quests.get(&quest_id)?.status == QuestStatus::Completed;
        let last = self.state.repeats.get(quest_id).map_or(0, |repeat| repeat.last);
        finished.then(|| last.saturating_add(u64::from(hours) * HOUR))
    }

    /// Each (quest, times) `quest_id` needs finished first (`prerequisite_completions`).
    pub fn completions_needed(&self, quest_id: u32) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.rules.needs.get(&quest_id).into_iter().flatten().copied()
    }

    /// The first completion count `quest_id` still lacks: (quest, needed, done).
    pub(super) fn completions_short(&self, quest_id: u32) -> Option<(u32, u32, u32)> {
        let needs = self.rules.needs.get(&quest_id)?;
        needs.iter().find_map(|&(other, times)| {
            let done = self.state.completions(other);
            (done < times).then_some((other, times, done))
        })
    }

    /// Count a turn-in. A repeatable quest's completion is recorded with its time; the return is
    /// its ordinal, which keeps each run's reward grant key apart (`None`: not repeatable).
    pub(super) fn record_completion(&mut self, quest_id: u32) -> Option<u32> {
        self.repeat_hours(quest_id)?;
        // The quest is `Completed` already: count from the record alone (R: a run finished
        // before completions were counted is not in it).
        let times = (self.state.repeats.get(quest_id)).map_or(0, |repeat| repeat.times).saturating_add(1);
        let last = self.now_secs();
        self.state.repeats.insert(quest_id, QuestRepeat { last, times });
        self.rewake();
        Some(times)
    }

    /// Once a second: when a finished repeatable quest has come due, move the marker revision
    /// so its giver draws its (!) again. Returns whether it moved.
    pub fn tick_repeats(&mut self) -> bool {
        match self.rules.wake {
            Some(at) if self.now_secs() >= at => {
                self.bump();
                self.rewake();
                true
            }
            _ => false,
        }
    }

    fn rewake(&mut self) {
        let now = self.now_secs();
        let pending = (self.rules.hours.keys()).filter_map(|id| self.repeat_ready_at(*id));
        self.rules.wake = pending.filter(|at| *at > now).min();
    }
}
