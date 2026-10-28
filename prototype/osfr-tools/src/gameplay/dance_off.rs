//! T538: the dance-off's gameplay side. `world_events` owns the run's participation, votes and
//! cakes; this module answers the two questions gameplay can only answer: which client item
//! names the character wears, and the `join_dance_off` quest hook for a costume join.
use super::{Gameplay, Outcome, npc_event};

/// Client equipment slot 17, the whole-body costume (`00ad83b0` accepts slots 1..=17).
const COSTUME_SLOT: i32 = 17;

/// T538: the world tick's join-hook hand-off. The tick detects the costume join and records the
/// participation in `world_events`; it queues `pending` on the live gameplay, and the session's
/// own tick emits the quest event from its staged clone, so the existing `commit_outcome` saves
/// it. `credited` remembers the run this session has emitted, so the join pass queues once; a
/// reconnect starts with none, so the next scan emits it once more (the objective completes once).
/// Neither field is persisted: the run is in-memory by design.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TickState {
    pending: Option<(u32, crate::quest_engine::DanceTeam)>,
    credited: Option<(u32, crate::quest_engine::DanceTeam)>,
}

impl Gameplay {
    /// The client `NameId` of every whole-body costume (slot 17) the character visibly wears,
    /// from the item definition's presentation row (`presentation`, reader `008e3760`). The
    /// visible outfit is `active_profile` (`Profiles::shown`): a worn staff job overrides the
    /// active job, so a costume parked on the hidden job never joins. A character with no
    /// definitions loaded, no items or no costume yields an empty list; the dance-off's content
    /// maps these names to teams and nothing here decides a side.
    pub fn equipped_costume_name_ids(&self) -> Vec<i32> {
        let profile = self.active_profile();
        let state = self.inventory.state();
        state
            .equipment
            .iter()
            .filter(|entry| entry.slot == COSTUME_SLOT && entry.item_id != 0)
            .filter(|entry| profile.is_none_or(|profile| entry.profile_id == profile))
            .filter_map(|entry| state.items.iter().find(|item| item.id == entry.item_id))
            .filter(|item| item.count > 0)
            .filter_map(|item| self.inventory.presentation(item.definition_id).ok())
            .filter(|presentation| presentation.name_id > 0)
            .map(|presentation| presentation.name_id)
            .collect()
    }

    /// T538: queue the quest hook for a join the world tick recorded, unless this session
    /// already emitted this run's side. Transient: the session's own tick consumes it.
    pub fn queue_dance_off_join(&mut self, serial: u32, team: crate::quest_engine::DanceTeam) {
        if self.dance_off.credited != Some((serial, team)) {
            self.dance_off.pending = Some((serial, team));
        }
    }

    /// T538: whether this session has emitted run `serial`'s join hook.
    pub fn dance_off_join_credited(
        &self,
        serial: u32,
        team: crate::quest_engine::DanceTeam,
    ) -> bool {
        self.dance_off.credited == Some((serial, team))
    }

    /// T538: emit a queued join hook. The session's own tick runs this on its staged clone, so
    /// the quest change is saved by `commit_outcome` like any other tick change. A quest error
    /// is logged and the hook is dropped for this run: one player's hook must not fail the
    /// session or the server tick (worker rule 17).
    pub(crate) fn credit_dance_off_join(&mut self, out: &mut Outcome) {
        let Some((serial, team)) = self.dance_off.pending.take() else {
            return;
        };
        if self.dance_off.credited == Some((serial, team)) {
            return;
        }
        self.dance_off.credited = Some((serial, team));
        if let Err(error) =
            self.npc_quest_event(crate::quest_engine::QuestEvent::JoinDanceOff { team }, out)
        {
            npc_event(serde_json::json!({
                "event": "world_event_join_hook_refused",
                "run": serial,
                "reason": error.to_string()
            }));
        }
    }
}
