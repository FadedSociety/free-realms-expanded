//! (R) T787: one settled server minigame reports its result facts to the quest engine.
//!
//! The id-only `FinishMinigame` event keeps crediting every goal authored without a
//! `require`, exactly as before. `MinigameResult` is the only event a `require` goal
//! matches, so a settle can never count one objective twice.
use super::*;

impl Gameplay {
    /// Credit one settle at most once per objective. `result` carries only facts the
    /// server observed; a producer that has none sends the id-only event alone.
    pub(super) fn credit_minigame(
        &mut self,
        minigame_id: u32,
        result: crate::quest_engine::MinigameResult,
        out: &mut Outcome,
    ) -> io::Result<()> {
        self.npc_quest_event(
            crate::quest_engine::QuestEvent::FinishMinigame { minigame_id },
            out,
        )?;
        self.npc_quest_event(
            crate::quest_engine::QuestEvent::MinigameResult {
                minigame_id,
                result,
            },
            out,
        )
    }
}
