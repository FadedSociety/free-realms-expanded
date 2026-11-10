//! The engine side of the brain: applying acts, and the points where the engine tells the
//! brain that something happened. Piece P3 (`combat-brain`) owns the bodies.
//!
//! Frozen by the combat seam: the names and signatures below. `encounter.rs` already calls
//! each hook once; the bodies here hold what the engine did at that point before the seam
//! (nothing, except the summon ladder on a hit).
use super::*;

/// The engine field `brain_state`: loaded programs and whatever of the brain must outlive
/// `reset_session` (per-run memory still belongs in `ArenaSim`). P3 fills it; no other piece
/// looks inside.
#[derive(Clone, Debug, Default)]
pub struct BrainState {}

/// What `enemy_died` decides for an enemy that reached zero health.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // the engine handles all three; the brain answers the last two once it lands
pub enum Death {
    /// Dies as before: removal, kill credit, goal progress, drops.
    Die,
    /// `yield`: credited as a kill with everything that follows, but no removal is sent; the
    /// actor stays as an NPC.
    Yield,
    /// `become`, `revive`: not a death. Nothing of the death path runs; the brain has already
    /// put the enemy's health back and sent its own packets.
    Survive,
}

impl EncounterEngine {
    /// Carry out one act of enemy `guid` (arena and field enemies alike).
    #[allow(dead_code)] // called by both enemy ticks once the brain decides anything
    pub(super) fn apply_act(
        &mut self,
        _guid: u64,
        _act: &brain::Act,
        _now_ms: u64,
        _out: &mut EncounterOutcome,
    ) {
    }

    /// Engine load: read this piece's own content from `directory` (absent files leave the
    /// feature dormant).
    pub(super) fn load_brains(&mut self, _directory: &Path) -> io::Result<()> {
        Ok(())
    }

    /// A run's actors were sent to the client (`guids` holds every actor shown, the guide too).
    pub(super) fn enemies_spawned(
        &mut self,
        _guids: &[u64],
        _now_ms: u64,
        _out: &mut EncounterOutcome,
    ) {
    }

    /// A player hit landed on enemy `guid` for `dealt`, leaving it `hp` of `max_health`.
    pub(super) fn enemy_hit_taken(
        &mut self,
        guid: u64,
        _dealt: u32,
        hp: u32,
        max_health: u32,
        _now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        if hp > 0 && self.active() {
            self.summon(guid, hp, max_health, out);
        }
    }

    /// Enemy `guid` reached zero health; called before its removal is sent. The answer decides
    /// whether it dies (`Death`).
    pub(super) fn enemy_died(
        &mut self,
        _guid: u64,
        _now_ms: u64,
        _out: &mut EncounterOutcome,
    ) -> Death {
        Death::Die
    }
}
