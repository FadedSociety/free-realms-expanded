//! L930: kart row selection at entry (the derby's `derby_rows.rs`, for the kart race).
//!
//! The row a race runs as is the first open `finish_minigame` objective that names a row the
//! race can run, else the Games window row played, else the gates' default row. The choice reads
//! the saved quest state and the server's row table only, never a client field.
use super::*;

impl Gameplay {
    /// Switches to Kart Driver and enters the track, as the row the player's open objective or
    /// the Games window row `entered` names (`None` at a gate NPC).
    pub(super) fn begin_kart_row(
        &mut self,
        character: &CharacterState,
        entered: Option<u32>,
    ) -> io::Result<Outcome> {
        let wanted = (self.npc_quests.as_ref())
            .map(crate::quest_engine::QuestEngine::open_finish_minigame_ids)
            .unwrap_or_default();
        if let Some(kart) = self.kart_entry.as_mut() {
            kart.choose_row(&wanted, entered);
        }
        self.begin_kart_race(character)
    }
}
