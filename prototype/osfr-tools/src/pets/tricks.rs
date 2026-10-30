//! T489: pet tricks.
//!
//! The client sends 53/7 (`Pet.PerformTrick` -> `DispatchPetActionRecord` `00b5fb30`, then
//! `UnlockPetTricksAfterGesture` `00cc7420`, or `BeginPetGestureRound` `00cc9330` for a drawing
//! round) with the trick id, and 53/8 right after it (`00cc7420`) or when the drawing ends
//! (`FinishPetGestureState` `00cc9a90`); 53/8 sets the pet handler's state word to 4 (`00b5ffe0`).
//! The grade is computed on the client and never sent, so nothing here depends on it.
//!
//! An accepted 53/7 answers 53/15 kind 1 (the trick-name line, `00b5ce60` case 15) and 53/20 kind 5
//! (marks the pet actor, `00b5ecef`), and plays the owner animation at once. The world tick plays
//! the pet animation after the row's delay and, when its run time is over, sends 53/15 kind 2
//! (the result), kind 3 (`PetsHandler:UnlockTricks`, `00b59070`) and 53/20 kind 6 (clears the
//! state word and the actor mark, `00b56a00`). A refusal sends only the two unlocks.
//!
//! The server owns the session, its animations and its end; a trick pays nothing (no XP, items or
//! coins), so a hostile 53/8 can at most end a trick early. All timing values are reconstructed
//! (R) and live in the model rows and the `care.trick` block of `pets.json`.

use super::{
    NOTIFY_TRICK, NOTIFY_UNLOCK, Pet, PetBusy, PetService, PetWire, SourceTag,
    pet_trick_notify_packet, pet_utility_notify_packet,
};
use crate::world_events::piece_052;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 53/15 PetTrickNotify kinds (`00b5ce60` case 15): the trick-name line, the result, the unlock.
const TRICK_NAME: u32 = 1;
const TRICK_RESULT: u32 = 2;
const TRICK_UNLOCK: u32 = 3;

/// The `care.trick` rules; `valid` keeps them sane at content load, so a bad file fails there.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrickConfig {
    /// A 53/8 for the trick that just ended is answered with the unlocks again for this long
    /// (ms): the client sets its state word to 4 on every 53/8, and a drawing round can end after
    /// the server's own end. The round force-ends 20 s after it starts (R: its settings file).
    gesture_grace_ms: u64,
    /// The text id and value of the kind 2 result. The client's grade never reaches the server,
    /// so every trick that ran reports this one text (R): the locale's success string, since the
    /// event is the client's `TrickSuccessResult`. No Lua handler consumes the event.
    result_text_id: u32,
    result_value: u32,
    source: SourceTag,
}

impl TrickConfig {
    pub(super) fn valid(&self) -> bool {
        (1_000..=60_000).contains(&self.gesture_grace_ms)
            && self.result_text_id > 0
            && self.result_value <= 1_000
    }
}

/// A trick between its accepted 53/7 and its end. The pet animation plays at `animation_at_ms`,
/// the unlocks go out at `ends_ms`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Trick {
    id: u32,
    animation: i32,
    animation_at_ms: u64,
    animated: bool,
    ends_ms: u64,
}

/// Session only: a dismiss, zoning or a disconnect drop it (it lives in the `ActivePet`, which
/// every summon replaces, so no timer outlives its pet).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum TrickState {
    #[default]
    Idle,
    Running(Trick),
    /// The trick ended at `at_ms`; a 53/8 for it is still answered until the grace window passes.
    Ended {
        id: u32,
        at_ms: u64,
    },
}

impl TrickState {
    pub(super) fn running(&self) -> bool {
        matches!(self, Self::Running(_))
    }
}

/// 53/15 kind 3 and 53/20 kind 6: the trick panel and the pet's state word are free again.
fn unlocks(pet_id: u32, trick_id: u32) -> Vec<Vec<u8>> {
    vec![
        pet_trick_notify_packet(pet_id, TRICK_UNLOCK, trick_id, 0, 0),
        pet_utility_notify_packet(pet_id, NOTIFY_UNLOCK, 0),
    ]
}

impl PetService {
    /// The pet knows the trick: learned, or listed as known by its model row (the rule the 53/5
    /// record shows).
    pub(super) fn trick_known(&self, pet: &Pet, trick_id: u32) -> bool {
        pet.learned_tricks.contains(&trick_id)
            || self
                .content
                .model(self.content.model_id(&pet.model))
                .is_some_and(|model| {
                    model
                        .tricks
                        .iter()
                        .any(|row| row.id == trick_id && row.known)
                })
    }

    /// 53/7: start the trick on the summoned pet. It needs the rules in `care.trick`, an animation
    /// row for the trick on the pet's model, a trick the pet knows, a pet that is not vanity (R),
    /// an owner out of water and not busy, and no feed, grooming or trick running. Every refusal
    /// answers the unlocks and changes nothing.
    pub(super) fn perform_trick(&mut self, trick_id: u32, busy: PetBusy, now_ms: u64) -> PetWire {
        self.try_trick(trick_id, busy, now_ms)
            .unwrap_or_else(|| PetWire {
                private: unlocks(self.active.map_or(0, |active| active.pet_id), trick_id),
                ..PetWire::default()
            })
    }

    fn try_trick(&mut self, trick_id: u32, busy: PetBusy, now_ms: u64) -> Option<PetWire> {
        let content = Arc::clone(&self.content);
        content.care.as_ref()?.trick.as_ref()?;
        let active = self.active?;
        if busy.minigame_active || active.in_water || active.interacting() {
            return None;
        }
        let pet = self.pet(active.pet_id)?;
        let model = content.model(content.model_id(&pet.model))?;
        let row = model
            .trick_animations
            .iter()
            .find(|row| row.trick_id == trick_id)?;
        if model.vanity || !self.trick_known(pet, trick_id) {
            return None;
        }
        let name_id = content
            .tricks
            .iter()
            .find(|trick| trick.id == trick_id)?
            .name_id
            .and_then(|id| u32::try_from(id).ok())
            .unwrap_or(0);
        let animation_at_ms = now_ms.saturating_add(u64::from(row.start_delay_ms));
        // Everything that can refuse is above; from here the session starts.
        self.active.as_mut()?.trick = TrickState::Running(Trick {
            id: trick_id,
            animation: row.animation_id,
            animation_at_ms,
            animated: false,
            ends_ms: animation_at_ms.saturating_add(u64::from(row.duration_ms)),
        });
        let owner = piece_052::animation_packet(active.owner_guid, row.owner_animation_id, false);
        // T888: the owner's own client posts the trick-name line itself (53/15 kind 1, `00b5ce60`
        // case 15), so only viewers get the bubble, as a say line from the owner (15/4, reader
        // `0x00b0f680`) naming the same locale string.
        let mut shared = vec![owner.clone()];
        if let Some(id) = i32::try_from(name_id).ok().filter(|id| *id > 0) {
            shared.push(crate::chat::string_say(active.owner_guid, id));
        }
        Some(PetWire {
            private: vec![
                pet_trick_notify_packet(active.pet_id, TRICK_NAME, trick_id, name_id, 0),
                pet_utility_notify_packet(active.pet_id, NOTIFY_TRICK, 0),
                owner,
            ],
            shared,
            accepted_trick: Some(trick_id),
            ..PetWire::default()
        })
    }

    /// 53/8: the client's gesture report. It carries no grade and pays nothing. For the running
    /// trick, once its pet animation has begun, it ends the trick now (R: before that it is the
    /// report `00cc7420` sends right behind 53/7, which must not cut the show short). For the
    /// trick that just ended, within the grace window, it answers the unlocks again, so the state
    /// word 53/8 sets is cleared. Anything else is dropped without an answer.
    pub(super) fn trick_result(&mut self, trick_id: u32, now_ms: u64) -> PetWire {
        let content = Arc::clone(&self.content);
        let (Some(config), Some(active)) = (
            content.care.as_ref().and_then(|care| care.trick.as_ref()),
            self.active,
        ) else {
            return PetWire::default();
        };
        match active.trick {
            TrickState::Running(trick)
                if trick.id == trick_id && now_ms >= trick.animation_at_ms =>
            {
                self.trick_step(config, now_ms, true)
            }
            TrickState::Ended { id, at_ms }
                if id == trick_id && now_ms.saturating_sub(at_ms) <= config.gesture_grace_ms =>
            {
                PetWire {
                    private: unlocks(active.pet_id, id),
                    ..PetWire::default()
                }
            }
            _ => PetWire::default(),
        }
    }

    /// The world tick of a running trick: the pet animation at its delay, then the end.
    pub(super) fn trick_tick(&mut self, config: &TrickConfig, wire: &mut PetWire, now_ms: u64) {
        let stepped = self.trick_step(config, now_ms, false);
        wire.private.extend(stepped.private);
        wire.shared.extend(stepped.shared);
    }

    /// Advance the running trick: play the pet animation once it is due (for the owner and the
    /// viewers), and end the trick when its time is over or `finish_now`: the result, then the
    /// unlocks. Nothing is saved.
    fn trick_step(&mut self, config: &TrickConfig, now_ms: u64, finish_now: bool) -> PetWire {
        let mut wire = PetWire::default();
        let Some(active) = self.active else {
            return wire;
        };
        let TrickState::Running(mut trick) = active.trick else {
            return wire;
        };
        if !trick.animated && now_ms >= trick.animation_at_ms {
            trick.animated = true;
            let animation = piece_052::animation_packet(active.actor_guid, trick.animation, false);
            wire.private.push(animation.clone());
            wire.shared.push(animation);
        }
        let state = if finish_now || now_ms >= trick.ends_ms {
            wire.private.push(pet_trick_notify_packet(
                active.pet_id,
                TRICK_RESULT,
                trick.id,
                config.result_text_id,
                config.result_value,
            ));
            wire.private.extend(unlocks(active.pet_id, trick.id));
            TrickState::Ended {
                id: trick.id,
                at_ms: now_ms,
            }
        } else {
            TrickState::Running(trick)
        };
        if let Some(active) = self.active.as_mut() {
            active.trick = state;
        }
        wire
    }

    /// The unlocks of a trick still running (a dismiss, a re-summon or zoning ends it), else none.
    pub(super) fn trick_end_packets(&self) -> Vec<Vec<u8>> {
        match self.active {
            Some(active) => match active.trick {
                TrickState::Running(trick) => unlocks(active.pet_id, trick.id),
                _ => Vec::new(),
            },
            None => Vec::new(),
        }
    }

    /// A trick ended from outside (the owner entered water): the unlocks and no result.
    pub(super) fn stop_trick(&mut self) -> Vec<Vec<u8>> {
        let ended = self.trick_end_packets();
        if let Some(active) = self.active.as_mut() {
            active.trick = TrickState::Idle;
        }
        ended
    }
}
