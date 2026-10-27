//! T441: a group's shared battle run, as one session's gateway sees it: lend the World's arena to
//! the engine around one staged action, and the member-side steps of the arena tick.

use super::Gateway;
use crate::encounter::{EncounterPhase, FieldParticipant, SharedArena};
use crate::gameplay::Outcome;
use std::io;

impl Gateway {
    /// Copy this player's live battle run into a new shared arena.
    pub fn open_arena(&self) -> Option<SharedArena> {
        self.gameplay.as_ref()?.open_arena(self.state.guid)
    }

    /// Lend `arena` to this player's engine for the staged action. `false`: not in that run.
    pub fn enter_arena(&mut self, arena: &SharedArena) -> bool {
        let guid = self.state.guid;
        self.gameplay
            .as_mut()
            .is_some_and(|game| game.enter_arena(arena, guid))
    }

    pub fn leave_arena(&mut self, arena: &mut SharedArena) {
        if let Some(game) = &mut self.gameplay {
            game.leave_arena(arena);
        }
    }

    /// This player as a target of the arena's enemies, once its scene is loaded.
    pub fn arena_participant(&self) -> Option<FieldParticipant> {
        if !self.loaded {
            return None;
        }
        self.gameplay.as_ref()?.arena_participant(&self.state)
    }

    pub fn run_phase(&self) -> Option<EncounterPhase> {
        self.gameplay.as_ref()?.run_phase()
    }

    pub fn tick_arena_enemies(
        &mut self,
        players: &[FieldParticipant],
        now_ms: u64,
        elapsed: u64,
    ) -> Vec<(u64, u32, Vec<Vec<u8>>)> {
        self.gameplay.as_mut().map_or_else(Vec::new, |game| {
            game.tick_arena_enemies(players, now_ms, elapsed)
        })
    }

    pub fn receive_arena_damage(&mut self, damage: u32, now_ms: u64) -> io::Result<Outcome> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| super::invalid("arena damage requires gameplay"))?
            .receive_arena_damage(damage, now_ms)
    }

    pub fn finish_arena_win(&mut self, now_ms: u64) -> io::Result<Outcome> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| super::invalid("arena win requires gameplay"))?
            .finish_arena_win(now_ms)
    }
}
