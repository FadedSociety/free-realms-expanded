//! T788: the accepted pet trick feeds the quest engine.
//!
//! The client's `Pet.PerformTrick` (`00c06890`) reaches `DispatchPetActionRecord` (`00b5fb30`),
//! which finds the trick row on the summoned pet and passes its id to
//! `UnlockPetTricksAfterGesture` (`00cc7420`); that function sends 53/7
//! (`sub_B5FF50`) and 53/8 (`sub_B5FFE0`). The server accepts a 53/7 in
//! `pets::tricks::PetService::try_trick`, which sets `PetWire::accepted_trick`; every refusal leaves
//! it `None`. The grade is computed on the client and never sent, so the objective is "the server
//! accepted this trick", never "performed well".
use super::*;

impl Gameplay {
    /// Credit `QuestEvent::PetTrick` for one accepted 53/7, if any. `accepted_trick` is set by the
    /// pet service on acceptance only: an unknown trick, a pet the owner has not summoned, a blocked
    /// pet (a minigame) and a late 53/8 all leave it `None`, so they reach no engine.
    pub(super) fn pet_trick_quest_event(
        &mut self,
        accepted_trick: Option<u32>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(trick_id) = accepted_trick {
            self.npc_quest_event(crate::quest_engine::QuestEvent::PetTrick { trick_id }, out)?;
        }
        Ok(())
    }
}
