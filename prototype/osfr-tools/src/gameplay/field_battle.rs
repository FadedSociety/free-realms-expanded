//! Field-battle start screen, prize preflight and arena entry.
//!
//! Every production starter (client Go, activity portal, developer command and
//! the balance simulator) asks this module before the encounter, inventory or
//! open start screen can change.

use super::*;

/// (R) Local copy: no recovered client locale row exists for this refusal.
pub(super) const UNAVAILABLE_BATTLE_PRIZE: &str =
    "This battle is unavailable because one of its rewards is missing.";

/// Result of a production field-battle start attempt.
pub(super) enum BattleStart {
    Started,
    /// A previewed prize cannot resolve against this save's item catalogue.
    Unavailable,
    /// The encounter engine refused the start (busy, level or content state).
    Refused(io::Error),
}

impl Gameplay {
    /// A field battle is playable only when every previewed prize resolves
    /// against the loaded item catalogue.
    pub(super) fn battle_prizes_available(&self, id: u32) -> bool {
        self.encounter.as_ref().is_some_and(|encounter| {
            encounter.battle_prizes_available(id, |item| self.inventory.has_definition(item))
        })
    }

    /// Start a battle from a production path other than the open start screen.
    /// The prize gate runs before the encounter or inventory can change.
    pub(super) fn begin_field_battle(
        &mut self,
        battle: u32,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<BattleStart> {
        if !self.battle_prizes_available(battle) {
            return Ok(BattleStart::Unavailable);
        }
        let profile = self.profiles.as_ref().map_or(0, Profiles::active);
        let Some(encounter) = self.encounter.as_mut() else {
            return Ok(BattleStart::Refused(invalid("no encounter engine")));
        };
        match encounter.begin(battle, character.clone(), profile) {
            Ok(result) => {
                self.apply_encounter(result, out)?;
                Ok(BattleStart::Started)
            }
            Err(error) => Ok(BattleStart::Refused(error)),
        }
    }

    /// Go (41/108) on the open battle start screen; client constructor
    /// 0x008b6e70 and sender 0x00938f00. Returns false when no battle is
    /// pending so the caller can use the ordinary encounter handler.
    /// A refusal reopens the same screen instead of consuming it.
    pub(super) fn handle_battle_go(
        &mut self,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<bool> {
        let profile = self.profiles.as_ref().unwrap().active();
        if let Some(required) = self
            .encounter
            .as_ref()
            .and_then(|encounter| encounter.pending_battle_level_refusal(self.job_level(profile)))
        {
            out.replies.push(chat::system(
                &crate::activity_portal::Refusal::LevelTooLow(required).message(),
            ));
            return Ok(true);
        }
        let Some(battle) = self
            .encounter
            .as_mut()
            .and_then(|encounter| encounter.take_pending_battle())
        else {
            return Ok(false);
        };
        if !self.battle_prizes_available(battle) {
            self.refuse_battle(battle, profile, UNAVAILABLE_BATTLE_PRIZE, out);
            return Ok(true);
        }
        if !self.abilities.as_ref().unwrap().supports_basic(profile) {
            // The start screen hides the HUD; keep it up so Switch Jobs is
            // reachable, then Go again or close.
            self.refuse_battle(
                battle,
                profile,
                "Switch to a combat job before starting a battle.",
                out,
            );
            return Ok(true);
        }
        let result = self
            .encounter
            .as_mut()
            .unwrap()
            .begin(battle, character.clone(), profile)?;
        self.apply_encounter(result, out)?;
        Ok(true)
    }

    fn refuse_battle(&mut self, battle: u32, profile: i32, message: &str, out: &mut Outcome) {
        out.replies.push(chat::system(message));
        out.replies.extend(
            self.encounter
                .as_mut()
                .unwrap()
                .reopen_battle_screen(battle, profile),
        );
    }
}
