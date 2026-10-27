//! T488: pet grooming and petting.
//!
//! 53/18 (`Pet.StartGroom`, sender `00b5f8c0`) starts a timed brushing session: the server answers
//! 53/20 kind 3, whose third word is the on-screen timer in seconds (`00b5ec14` calls
//! `FlashTimerSetTimer` `00a8eff0`, which multiplies it by 1000 ms), and spends the shampoo (and a
//! fitted conditioner) up front. The client then reports its strokes as 53/22 (sender `00ccebc0`,
//! packet ctor `00ccdb70`): `mode` is the pet handler's state word (2 while grooming, set by kind 3
//! at `00b5ec17`, cleared by kinds 2, 4 and 6), then a constant 3, the stroke time in ms and the
//! movement sum. Both numbers are the controller's integer accumulators, sent only once the stroke
//! time passes 250 ms (`00ccecf0`, `00ccf260`); the client's own consumer divides movement by time.
//! A session ends with 53/20 kind 4, which releases the spots and unlocks the panel (`00b56920`).
//!
//! The server owns every number a stroke is worth: the client's words are clamped here and the
//! gains are bounded per message and per second (a hostile client gains at most the cap). All
//! values are reconstructed (R) and live in the `pets.json` `care.groom` block.

use super::{
    KitRole, NOTIFY_GROOM, NOTIFY_GROOM_END, NOTIFY_UNLOCK, Need, PetBusy, PetNeeds, PetService,
    PetWire, SourceTag, add_need, decayed, fitted_item, pet_status_packet,
    pet_utility_notify_packet, spend_one,
};
use crate::{inventory::Inventory, world_events::piece_052};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// The pet handler's state word while grooming (`00b5ec17`) and while idle (kinds 2, 4, 6).
const TACTILE_GROOM: u32 = 2;
const TACTILE_PET: u32 = 0;
/// Utility slots of the grooming kit: the definition slot minus 199 (`00c077a0`), so shampoo 200
/// is 1, conditioner 201 is 2 and the brush 203 is 4 (R: the 15 catalogue rows in slot 203).
const SHAMPOO_SLOT: usize = 1;
const CONDITIONER_SLOT: usize = 2;
const BRUSH_SLOT: usize = 4;
/// The chat line when a full bar ends the session: no client string says it (R, T812; the
/// locale has only quest-specific lines), so the owner is told in chat.
const GROOM_DONE: &str = "Your pet is squeaky clean!";
/// The per-second gain cap counts in fixed windows this long. A span that straddles a boundary
/// can see up to two caps (worst case 2x over one second), accepted and pinned by
/// `a_stroke_span_across_a_window_boundary_sees_at_most_two_caps`: a session is 60 s and only
/// need points move, so the cap bounds a fast groom, not an economy.
const WINDOW_MS: u64 = 1_000;

/// The `care.groom` rules; `valid` keeps them sane at content load, so a bad file fails there.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GroomConfig {
    /// The session length, and the timer word of kind 3 (seconds). The locale says the pet "will
    /// only hold still for a minute" (R: 60; OSFR uses 25).
    seconds: u32,
    /// Groom points per unit of stroke speed (movement over time, an integer).
    stroke_gain: f32,
    /// The stroke speed a message may claim.
    max_stroke_speed: u32,
    max_gain_per_message: f32,
    max_gain_per_second: f32,
    /// Play points one petting message adds; at most 2 (R, OSFR).
    petting_gain: f32,
    /// The least time between two petting messages that count.
    petting_min_interval_ms: u64,
    /// The animation group played on the pet when a session ends by itself (R, OSFR).
    dry_off_animation: i32,
    source: SourceTag,
}

impl GroomConfig {
    pub(super) fn valid(&self) -> bool {
        let points = |value: f32, most: f32| value.is_finite() && value > 0.0 && value <= most;
        (1..=300).contains(&self.seconds)
            && points(self.stroke_gain, 10.0)
            && (1..=100_000).contains(&self.max_stroke_speed)
            && points(self.max_gain_per_message, 100.0)
            && points(self.max_gain_per_second, 100.0)
            && self.max_gain_per_message <= self.max_gain_per_second
            && points(self.petting_gain, 2.0)
            && (100..=60_000).contains(&self.petting_min_interval_ms)
            && self.dry_off_animation > 0
    }

    /// What one stroke message is worth. The client's words are unsigned on the wire but signed
    /// integers in the client, so a "negative" word counts as 0; no time means no stroke.
    fn stroke_points(&self, duration_ms: u32, movement: u32) -> f32 {
        let count = |raw: u32| if raw > i32::MAX as u32 { 0 } else { raw };
        let (time, movement) = (count(duration_ms), count(movement));
        if time == 0 {
            return 0.0;
        }
        let speed = (movement / time).min(self.max_stroke_speed);
        (self.stroke_gain * speed as f32).min(self.max_gain_per_message)
    }
}

/// A session between its kind 3 and kind 4. Session only: a dismiss, zoning or a disconnect drop
/// it, and the shampoo it spent stays spent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Grooming {
    started_ms: u64,
    /// The current one-second window of the per-second cap, and what it has added so far.
    window_ms: u64,
    window_gain: f32,
}

impl PetService {
    /// 53/18: start a grooming session and answer 53/20 kind 3. The pet must be summoned, not
    /// vanity (R), its owner not in water or busy, no feed or session running, its groom bar below
    /// the maximum (R: no shampoo is wasted on a clean pet), and a shampoo and a brush fitted.
    /// Every refusal answers kind 6 and changes nothing.
    pub(super) fn start_groom(
        &mut self,
        pet_id: u32,
        inventory: &mut Inventory,
        busy: PetBusy,
        now_ms: u64,
    ) -> PetWire {
        self.try_groom(pet_id, inventory, busy, now_ms)
            .unwrap_or_else(|| Self::unlock(pet_id))
    }

    /// The 53/20 kind 6 unlock: the client clears its state word and unlocks the utilities
    /// (`00b56a00`, which does not read the pet id word).
    fn unlock(pet_id: u32) -> PetWire {
        PetWire {
            private: vec![pet_utility_notify_packet(pet_id, NOTIFY_UNLOCK, 0)],
            ..PetWire::default()
        }
    }

    fn try_groom(
        &mut self,
        pet_id: u32,
        inventory: &mut Inventory,
        busy: PetBusy,
        now_ms: u64,
    ) -> Option<PetWire> {
        let content = Arc::clone(&self.content);
        let groom = content.care.as_ref()?.groom.as_ref()?;
        let (shampoo, conditioner) = self.groom_kit(pet_id, inventory, busy, now_ms)?;
        // The shampoo and conditioner are spent on copies and committed together with the session.
        let mut roster = self.roster.clone();
        let mut staged = inventory.clone();
        let mut private = Vec::new();
        for instance in [Some(shampoo), conditioner].into_iter().flatten() {
            private.extend(spend_one(&mut roster, &mut staged, instance).ok()?);
        }
        self.roster = roster;
        *inventory = staged;
        if let Some(active) = self.active.as_mut() {
            active.grooming = Some(Grooming {
                started_ms: now_ms,
                window_ms: now_ms,
                window_gain: 0.0,
            });
        }
        private.push(pet_utility_notify_packet(
            pet_id,
            NOTIFY_GROOM,
            groom.seconds,
        ));
        Some(PetWire {
            private,
            changed: true,
            ..PetWire::default()
        })
    }

    /// The owned shampoo and the optional conditioner a session would spend, when it may start.
    fn groom_kit(
        &self,
        pet_id: u32,
        inventory: &Inventory,
        busy: PetBusy,
        now_ms: u64,
    ) -> Option<(i32, Option<i32>)> {
        let active = self.active.filter(|active| active.pet_id == pet_id)?;
        if busy.minigame_active || active.in_water || active.interacting() {
            return None;
        }
        let pet = self.pet(pet_id)?;
        if self.content.is_vanity(self.content.model_id(&pet.model)) {
            return None;
        }
        let policy = &self.content.need_policy;
        if decayed(policy, pet, true, now_ms / 1_000)?.groom >= policy.maximum {
            return None;
        }
        fitted_item(pet, inventory, BRUSH_SLOT)
            .filter(|brush| self.is_kit(brush.definition_id, KitRole::Brush))?;
        // T797: the shampoo and conditioner must be recorded kit items in their slots; the brush
        // above is kept, these two are spent.
        let consumable = |slot, role| {
            let item = fitted_item(pet, inventory, slot)?;
            self.is_kit(item.definition_id, role).then_some(item.id)
        };
        Some((
            consumable(SHAMPOO_SLOT, KitRole::Shampoo)?,
            consumable(CONDITIONER_SLOT, KitRole::Conditioner),
        ))
    }

    /// 53/22: a stroke report. While grooming, mode 2 raises the groom bar; with no session,
    /// mode 0 is petting and raises play. A grooming-mode report with no server session is a
    /// stale client lock (it missed kind 4), so it is answered with kind 6 and changes nothing.
    /// Anything else (petting in water, on a vanity pet or while busy, an unknown mode) is
    /// dropped silently: kind 6 would clear the client's state under a session that is still
    /// running.
    pub(super) fn tactile(
        &mut self,
        mode: u32,
        duration_ms: u32,
        movement: u32,
        busy: PetBusy,
        now_ms: u64,
    ) -> PetWire {
        if mode == TACTILE_GROOM && self.active.is_none_or(|active| active.grooming.is_none()) {
            return Self::unlock(self.active.map_or(0, |active| active.pet_id));
        }
        let content = Arc::clone(&self.content);
        let (Some(groom), Some(active)) = (
            content.care.as_ref().and_then(|care| care.groom.as_ref()),
            self.active,
        ) else {
            return PetWire::default();
        };
        if busy.minigame_active || active.in_water {
            return PetWire::default();
        }
        match (active.grooming, mode) {
            (Some(session), TACTILE_GROOM) => {
                self.groom_stroke(groom, session, duration_ms, movement, now_ms)
            }
            (None, TACTILE_PET) => {
                self.pet_stroke(groom, now_ms);
                PetWire::default()
            }
            _ => PetWire::default(),
        }
    }

    /// One groom stroke: the points the message is worth, bounded by what is left of this
    /// second's cap. A full bar ends the session at once.
    fn groom_stroke(
        &mut self,
        groom: &GroomConfig,
        mut session: Grooming,
        duration_ms: u32,
        movement: u32,
        now_ms: u64,
    ) -> PetWire {
        if now_ms.saturating_sub(session.window_ms) >= WINDOW_MS {
            session.window_ms = now_ms;
            session.window_gain = 0.0;
        }
        let points = groom
            .stroke_points(duration_ms, movement)
            .min(groom.max_gain_per_second - session.window_gain)
            .max(0.0);
        session.window_gain += points;
        let Some(active) = self.active.as_mut() else {
            return PetWire::default();
        };
        active.grooming = Some(session);
        let pet_id = active.pet_id;
        let full = points > 0.0
            && self
                .raise_need(pet_id, Need::Groom, points, now_ms)
                .is_some_and(|needs| needs.groom >= self.content.need_policy.maximum);
        if full {
            PetWire {
                notice: Some(GROOM_DONE),
                ..self.finish_groom(groom, now_ms)
            }
        } else {
            PetWire::default()
        }
    }

    /// One petting message: play rises by `petting_gain`, at most once per
    /// `petting_min_interval_ms`. The next live 53/3 shows it; nothing is saved per message.
    fn pet_stroke(&mut self, groom: &GroomConfig, now_ms: u64) {
        let Some(active) = self.active else { return };
        // The rate limit comes first: a flood is dropped before it costs a pet lookup.
        if active
            .petted_ms
            .is_some_and(|at| now_ms.saturating_sub(at) < groom.petting_min_interval_ms)
        {
            return;
        }
        let vanity = self
            .pet(active.pet_id)
            .is_none_or(|pet| self.content.is_vanity(self.content.model_id(&pet.model)));
        if vanity {
            return;
        }
        if self
            .raise_need(active.pet_id, Need::Play, groom.petting_gain, now_ms)
            .is_some()
        {
            if let Some(active) = self.active.as_mut() {
                active.petted_ms = Some(now_ms);
            }
        }
    }

    /// Raise one need of the summoned pet by `points` on its decayed value (mood follows as the
    /// average) and return the new needs. `None`, with nothing changed, when the pet is gone or
    /// its clock went backwards. Patches the one pet in place: this runs per stroke message.
    fn raise_need(
        &mut self,
        pet_id: u32,
        need: Need,
        points: f32,
        now_ms: u64,
    ) -> Option<PetNeeds> {
        let now = now_ms / 1_000;
        let pet = self.roster.pets.iter_mut().find(|pet| pet.id == pet_id)?;
        let policy = &self.content.need_policy;
        let mut needs = decayed(policy, pet, true, now)?;
        add_need(&mut needs, need, points, policy.maximum);
        needs.mood = (needs.food + needs.groom + needs.play) / 3.0;
        pet.needs = needs;
        pet.updated_at = now;
        Some(needs)
    }

    /// A session that ran out its time ends on the world tick (`care_tick`).
    pub(super) fn groom_tick(&mut self, groom: &GroomConfig, wire: &mut PetWire, now_ms: u64) {
        let Some(session) = self.active.and_then(|active| active.grooming) else {
            return;
        };
        if now_ms.saturating_sub(session.started_ms) >= u64::from(groom.seconds) * 1_000 {
            let ended = self.finish_groom(groom, now_ms);
            wire.private.extend(ended.private);
            wire.shared.extend(ended.shared);
            wire.changed |= ended.changed;
        }
    }

    /// A session that ends by itself (a full bar or its time): the final 53/3, the dry-off
    /// animation for the owner and the viewers, then 53/20 kind 4. The groom gains already sit in
    /// the roster, so the save is marked.
    fn finish_groom(&mut self, groom: &GroomConfig, now_ms: u64) -> PetWire {
        let mut wire = PetWire::default();
        let Some(active) = self.active.filter(|active| active.grooming.is_some()) else {
            return wire;
        };
        let needs = self
            .pet(active.pet_id)
            .and_then(|pet| decayed(&self.content.need_policy, pet, true, now_ms / 1_000));
        if let Some(needs) = needs {
            wire.private.push(pet_status_packet(active.pet_id, &needs));
        }
        let dry_off =
            piece_052::animation_packet(active.actor_guid, groom.dry_off_animation, false);
        wire.private.push(dry_off.clone());
        wire.shared.push(dry_off);
        wire.private.push(pet_utility_notify_packet(
            active.pet_id,
            NOTIFY_GROOM_END,
            0,
        ));
        wire.changed = true;
        if let Some(active) = self.active.as_mut() {
            active.grooming = None;
            active.shown = needs.map(|needs| (needs, now_ms)).or(active.shown);
        }
        wire
    }

    /// A session ended from outside (the owner entered water): kind 4 and no further gain. The
    /// gains so far are in the roster, so the save is marked.
    pub(super) fn stop_groom(&mut self) -> PetWire {
        let Some(active) = self
            .active
            .as_mut()
            .filter(|active| active.grooming.is_some())
        else {
            return PetWire::default();
        };
        active.grooming = None;
        PetWire {
            private: vec![pet_utility_notify_packet(
                active.pet_id,
                NOTIFY_GROOM_END,
                0,
            )],
            changed: true,
            ..PetWire::default()
        }
    }
}
