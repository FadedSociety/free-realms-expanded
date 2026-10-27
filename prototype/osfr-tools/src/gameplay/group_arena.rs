//! T441: a group's shared battle run, as one member's gameplay sees it. The arena itself is the
//! World's (`encounter::SharedArena`); these only lend it to the engine for one action and carry
//! the member's own results through `apply_encounter` like every other encounter outcome.

use super::*;
use crate::encounter::{EncounterPhase, FieldParticipant, SharedArena};

impl Gameplay {
    pub fn open_arena(&self, guid: u64) -> Option<SharedArena> {
        self.encounter.as_ref()?.open_arena(guid)
    }

    pub fn enter_arena(&mut self, arena: &SharedArena, guid: u64) -> bool {
        self.encounter
            .as_mut()
            .is_some_and(|engine| engine.enter_arena(arena, guid))
    }

    pub fn leave_arena(&mut self, arena: &mut SharedArena) {
        if let Some(engine) = &mut self.encounter {
            engine.leave_arena(arena);
        }
    }

    pub fn arena_participant(&self, character: &CharacterState) -> Option<FieldParticipant> {
        self.encounter
            .as_ref()?
            .arena_participant(character.guid, character.position)
    }

    pub fn run_phase(&self) -> Option<EncounterPhase> {
        self.encounter.as_ref()?.run_phase()
    }

    pub fn tick_arena_enemies(
        &mut self,
        players: &[FieldParticipant],
        now_ms: u64,
        elapsed: u64,
    ) -> Vec<(u64, u32, Vec<Vec<u8>>)> {
        self.encounter
            .as_mut()
            .map_or_else(Vec::new, |e| e.tick_arena_enemies(players, now_ms, elapsed))
    }

    /// The shared enemies hit this member: its own health, knockout and results screen.
    pub fn receive_arena_damage(&mut self, damage: u32, now_ms: u64) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if let Some(engine) = &mut self.encounter {
            let result = engine.receive_arena_damage(damage, now_ms);
            self.apply_encounter(result, &mut out)?;
        }
        Ok(out)
    }

    /// Another member won the shared run: this member's own results, prize and quest credit.
    pub fn finish_arena_win(&mut self, now_ms: u64) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        if let Some(engine) = &mut self.encounter {
            let result = engine.finish_shared_win(now_ms);
            self.apply_encounter(result, &mut out)?;
        }
        Ok(out)
    }
}
