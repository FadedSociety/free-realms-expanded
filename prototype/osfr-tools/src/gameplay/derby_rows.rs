//! Derby row selection at ordinary NPC entry (T791).
//!
//! A gate NPC hosts the default package (row 39) first and any `derby_rows.json` rows whose
//! `npcs` name it after that. The row a player enters is the row named by the first open
//! `finish_minigame` objective; with no such objective it is the NPC's first row, which keeps
//! every pre-T791 entry on row 39. The choice reads the saved quest state and the server's row
//! table only, never a client field, so a reconnect while the objective is still open selects the
//! same row. Which arena a gate NPC runs is not recovered (R); the row's own `npcs` list is the
//! authored assignment.

use super::*;

impl Gameplay {
    /// The `game_id` the derby option at `npc_id` enters now. `None` only when this NPC hosts no
    /// row, which the caller's `DerbyGateway::admit` already refuses.
    pub(super) fn derby_row_for(&self, npc_id: u64) -> Option<i32> {
        let derby = self.derby.as_ref()?;
        let wanted = self
            .npc_quests
            .as_ref()
            .map(crate::quest_engine::QuestEngine::open_finish_minigame_ids)
            .unwrap_or_default();
        let rows: Vec<_> = derby.rows_for(npc_id).collect();
        for id in wanted {
            if let Some(row) = rows.iter().find(|row| row.game_id() as u32 == id) {
                return Some(row.game_id());
            }
        }
        rows.first().map(|row| row.game_id())
    }
}
