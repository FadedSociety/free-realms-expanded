//! T222: the held combat power-up's use request and its own key slot.
//!
//! The client sends 36/10 bar 1 slot 2 (`power_ups::SLOT`) for `ActionBarSlot3`
//! (default key `3`); see `notes/Research/Power-up Use Key.md`. The encounter
//! engine owns the held kind and its effect; this module is the one seam that
//! turns the request into a result, sends the bar with or without the kind, and
//! keeps the job-ability fall-through when nothing is held.

use super::*;

impl Gameplay {
    /// The held combat power-up (session state: never saved, never traded).
    pub fn held_power_up(&self) -> Option<&str> {
        self.encounter.as_ref()?.held_power_up()
    }

    /// The toolbar (36/13 definitions, then 36/5) with the held power-up on its
    /// own key slot. Every path that sends a job toolbar goes through here, so
    /// a held kind cannot be lost by a job switch, level-up, derby return or
    /// reload and cannot linger after it is used.
    pub(crate) fn ability_activation_for(&self, profile: i32) -> io::Result<Vec<Vec<u8>>> {
        let abilities = self
            .abilities
            .as_ref()
            .ok_or_else(|| invalid("ability content is not loaded"))?;
        let power_up = self.encounter.as_ref().and_then(|e| e.held_power_up_slot());
        abilities.activation_with_power_up(profile, self.job_level(profile), power_up.as_ref())
    }

    /// After a world tick, resend the toolbar only when a pickup or a
    /// stale-table clear changed the held kind (the client shows the kind
    /// only as a bar slot).
    pub(crate) fn refresh_power_up_slot(
        &self,
        held_before: Option<&str>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if self.held_power_up() == held_before {
            return Ok(());
        }
        let profile = self.profiles.as_ref().unwrap().active();
        out.replies.extend(self.ability_activation_for(profile)?);
        Ok(())
    }

    /// Handle a job-bar slot request for the held power-up. `None` means no
    /// power-up is held (or this is another slot), so the caller keeps its
    /// normal job-ability handling. A noncombat job cannot use a combat kind.
    /// `field` swaps the shared field in for the caller that owns it.
    pub(crate) fn use_power_up_request(
        &mut self,
        slot: i32,
        character: &CharacterState,
        now_ms: u64,
        field: Option<&mut crate::encounter::SharedFieldState>,
    ) -> io::Result<Option<Outcome>> {
        if slot != crate::power_ups::SLOT {
            return Ok(None);
        }
        let Some(profile) = self
            .profiles
            .as_ref()
            .map(crate::profiles::Profiles::active)
        else {
            return Ok(None);
        };
        if self.held_power_up().is_none() {
            return Ok(None);
        }
        if !self
            .abilities
            .as_ref()
            .is_some_and(|a| a.supports_basic(profile))
        {
            return Ok(Some(Outcome {
                replies: vec![crate::abilities::unsupported_cast_failure()],
                ..Default::default()
            }));
        }
        // The shared-field caller also serves this action only for the field
        // zone and a non-instance fight, exactly like the job ability route.
        if field.is_some() {
            let Some(engine) = self.encounter.as_ref() else {
                return Ok(None);
            };
            if engine.active() || character.zone_id != engine.world().zone_id {
                return Ok(None);
            }
        }
        let held_before = self.held_power_up().map(str::to_owned);
        let Some(engine) = self.encounter.as_mut() else {
            return Ok(None);
        };
        let result = match field {
            Some(field) => engine.with_shared_fields(field, character.guid, |engine| {
                engine.use_power_up(character.position, profile, now_ms)
            })?,
            None => engine.use_power_up(character.position, profile, now_ms)?,
        };
        let Some(result) = result else {
            return Ok(None);
        };
        let mut out = Outcome::default();
        self.apply_encounter(result, &mut out)?;
        // Only a changed hold needs the bar: a refusal keeps the same slot.
        if self.held_power_up().map(str::to_owned) != held_before {
            out.replies.extend(self.ability_activation_for(profile)?);
        }
        Ok(Some(out))
    }
}
