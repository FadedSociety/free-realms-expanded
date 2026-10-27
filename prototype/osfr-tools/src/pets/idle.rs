//! T531: pet idle life and thought bubbles.
//!
//! A summoned pet that stands beside a still owner (the T455 settle) plays idle animations,
//! sometimes a bathroom animation, and after a while lies down in a rest posture (enter, loop,
//! fidgets) until the owner moves. A dog or cat shakes itself dry once it stands still after the
//! owner leaves water. Every animation is a 35/8 (`piece_052::animation_packet`, base flag for the
//! looping rest pose). Separately, every `bubble_min_ms`..`bubble_max_ms` a non-vanity pet shows a
//! thought bubble for its lowest need under the threshold (else for a low mood): 35/27, sent to the
//! owner and to the viewers.
//!
//! Client facts: 35/27 `PlayerUpdatePacketThoughtBubble` (ctor `008b4870`, reader `008d7880`, base
//! words `008d6830`) is a u64 actor guid, an i32 icon, an i32 seconds and a u8 clear flag. The
//! dispatcher (`0092f460`, jump table `0092f540` case 27, `00933124`) finds the actor by guid
//! (`00984960`) and calls `0096c120`: it raises the Lua event `PetsHandler:ThoughtBubble` with the
//! three values (the shipped Lua handler is empty), stops the actor's previous bubble
//! (`00960300`), and, when the flag is 0, plays the icon id as an actor composite effect
//! (`00960170` -> `00bf6770`, which looks the id up in the effect table `01025390`) while it keeps
//! the icon, the seconds and the start time on the actor (+0x5f8, +0x5fc, +0x600). So the icon
//! must be an id of `ActorCompositeEffectDefinitions.xml`. The shipped ids are OSFR's and (R), and
//! unverified: in that table 39, 40 and 42 are unrelated particle effects and 41 is undefined
//! (see `care.idle.source` in `pets.json`), so they need a native check and a tune.
//!
//! Everything here is a session-only visual: nothing is saved, no coins, items or XP move. All
//! timings, chances, thresholds and icon ids are reconstructed (R) and live in `care.idle` and in
//! each model's `idle` row of `pets.json`, which `scripts/pets_import.py` fills from the OSFR
//! reference data after checking each animation id against the client.

use super::{PetService, PetWire, SourceTag, decayed};
use crate::world_events::piece_052::{self, ANIM_ID_STAND};
use serde::{Deserialize, Serialize};

/// Longest run time one animation row may claim (ms).
const MAX_ANIMATION_MS: u32 = 60_000;
/// The base-player-update opcode and the thought-bubble subtype (`008b4870` passes 0x1b).
const PLAYER_UPDATE: u16 = 35;
const THOUGHT_BUBBLE: u16 = 27;

/// One client animation slot (`AnimationTypes.xml`) and how long it runs.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetAnimation {
    pub animation_id: i32,
    pub duration_ms: u32,
}

impl PetAnimation {
    fn valid(&self) -> bool {
        self.animation_id > 0 && (1..=MAX_ANIMATION_MS).contains(&self.duration_ms)
    }
}

/// A rest posture: the transition into it, the looping base animations (one is picked), the
/// fidgets played now and then while it lasts, and the transition out of it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetRestPosture {
    pub enter: PetAnimation,
    pub exit: PetAnimation,
    pub loops: Vec<i32>,
    pub fidgets: Vec<PetAnimation>,
}

/// A model's idle life. `shake_off` is set for the models whose ADR declares `pet_dry_off` (the
/// dogs and cats); the importer checks every id against the client.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetIdleSet {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<PetAnimation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bathroom: Vec<PetAnimation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rest_postures: Vec<PetRestPosture>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shake_off: Option<PetAnimation>,
    src: String,
}

impl PetIdleSet {
    /// Positive ids, bounded run times and list sizes, every posture loopable: checked at content
    /// load, so a bad file fails there and never mid-session.
    pub(super) fn valid(&self) -> bool {
        !self.src.is_empty()
            && self.animations.len() <= 64
            && self.bathroom.len() <= 16
            && self.rest_postures.len() <= 8
            && self
                .animations
                .iter()
                .chain(&self.bathroom)
                .chain(&self.shake_off)
                .all(PetAnimation::valid)
            && self.rest_postures.iter().all(|posture| {
                posture.enter.valid()
                    && posture.exit.valid()
                    && (1..=16).contains(&posture.loops.len())
                    && posture.loops.iter().all(|id| *id > 0)
                    && posture.fidgets.len() <= 32
                    && posture.fidgets.iter().all(PetAnimation::valid)
            })
    }
}

/// The `care.idle` rules; `valid` keeps them sane at content load.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct IdleConfig {
    /// How long a pet must have been settled before its first idle animation.
    idle_after_ms: u64,
    /// The gap between two idle (or fidget) animations is a random value in this range.
    idle_min_ms: u64,
    idle_max_ms: u64,
    /// The chance that an idle animation is a bathroom one, when the model has any.
    bathroom_chance: f32,
    /// How long a pet must have been settled before it lies down.
    rest_after_ms: u64,
    /// The gap between two thought bubbles is a random value in this range.
    bubble_min_ms: u64,
    bubble_max_ms: u64,
    /// How long the client keeps a bubble (the 35/27 seconds word).
    bubble_seconds: u32,
    /// A need under this shows its icon; a mood under `mood_bubble_threshold` shows `sad_icon`
    /// when no need does.
    need_bubble_threshold: f32,
    mood_bubble_threshold: f32,
    hunger_icon: i32,
    hygiene_icon: i32,
    play_icon: i32,
    sad_icon: i32,
    source: SourceTag,
}

impl IdleConfig {
    pub(super) fn valid(&self) -> bool {
        let percent = |value: f32| value.is_finite() && (0.0..=100.0).contains(&value);
        (1_000..=3_600_000).contains(&self.idle_min_ms)
            && self.idle_min_ms <= self.idle_max_ms
            && self.idle_max_ms <= 3_600_000
            && self.idle_after_ms <= 3_600_000
            && self.rest_after_ms <= 3_600_000
            && self.bathroom_chance.is_finite()
            && (0.0..=1.0).contains(&self.bathroom_chance)
            && (1_000..=3_600_000).contains(&self.bubble_min_ms)
            && self.bubble_min_ms <= self.bubble_max_ms
            && self.bubble_max_ms <= 3_600_000
            && (1..=60).contains(&self.bubble_seconds)
            && percent(self.need_bubble_threshold)
            && percent(self.mood_bubble_threshold)
            && [
                self.hunger_icon,
                self.hygiene_icon,
                self.play_icon,
                self.sad_icon,
            ]
            .iter()
            .all(|icon| *icon > 0)
    }
}

/// A small seeded generator (SplitMix64): the idle choices are visual, not security. The seed is
/// injected (`PetService::set_idle_seed`), so a test replays any session.
#[derive(Clone, Copy)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A value in `low..=high` (callers pass `low <= high`).
    fn range(&mut self, low: u64, high: u64) -> u64 {
        low + self.next() % (high.saturating_sub(low) + 1)
    }

    fn chance(&mut self, probability: f32) -> bool {
        (self.next() >> 40) as f32 / 16_777_216.0 < probability
    }

    fn index(&mut self, len: usize) -> Option<usize> {
        (len > 0).then(|| (self.next() % len as u64) as usize)
    }
}

/// What the idle life asks the wire to play on the pet actor.
struct Play {
    animation_id: i32,
    base: bool,
}

impl Play {
    fn once(animation_id: i32) -> Self {
        Self {
            animation_id,
            base: false,
        }
    }

    fn base(animation_id: i32) -> Self {
        Self {
            animation_id,
            base: true,
        }
    }
}

/// The session-only idle state of the summoned pet (it lives in the `ActivePet`, which every
/// summon replaces, so no timer outlives its pet).
#[derive(Clone, Copy)]
pub(super) struct IdleState {
    rng: Rng,
    /// The pet stands settled (false while it walks, swims, has the pet window open or is being
    /// fed, groomed or taught a trick); the timers below run only then.
    settled: bool,
    /// The next idle animation while it stands, or the next fidget while it rests.
    animation_at_ms: u64,
    /// When it lies down.
    rest_at_ms: u64,
    /// The animation now playing ends here; nothing else starts before.
    busy_until_ms: u64,
    /// The index of the rest posture it lies in.
    rest: Option<usize>,
    /// The owner left water: a dog or cat shakes once it stands still.
    dry_off: bool,
    /// The next thought bubble; scheduled by the first tick.
    bubble_at_ms: Option<u64>,
}

impl IdleState {
    pub(super) fn new(seed: u64) -> Self {
        Self {
            rng: Rng(seed),
            settled: false,
            animation_at_ms: u64::MAX,
            rest_at_ms: u64::MAX,
            busy_until_ms: 0,
            rest: None,
            dry_off: false,
            bubble_at_ms: None,
        }
    }

    /// The owner's water state changed: leaving it arms a shake-off, entering it (or a repeated
    /// "out" with no water before) does not.
    pub(super) fn water(&mut self, was_in_water: bool, in_water: bool) {
        self.dry_off = (was_in_water || self.dry_off) && !in_water;
    }

    /// Something is due, so the model's rows are worth a look.
    fn due(&self, now_ms: u64) -> bool {
        now_ms >= self.busy_until_ms
            && (self.dry_off || now_ms >= self.animation_at_ms || now_ms >= self.rest_at_ms)
    }

    fn gap(&mut self, config: &IdleConfig) -> u64 {
        self.rng.range(config.idle_min_ms, config.idle_max_ms)
    }

    /// Schedule the next thought bubble.
    fn schedule_bubble(&mut self, config: &IdleConfig, now_ms: u64) {
        let gap = self.rng.range(config.bubble_min_ms, config.bubble_max_ms);
        self.bubble_at_ms = Some(now_ms.saturating_add(gap));
    }

    /// One step of a settled pet: the shake-off, a fidget while it rests, lying down, or an idle
    /// or bathroom animation. At most one animation per call; a model with no rows idles never.
    fn advance(&mut self, config: &IdleConfig, set: Option<&PetIdleSet>, now_ms: u64) -> Vec<Play> {
        let Some(set) = set else {
            self.animation_at_ms = u64::MAX;
            self.rest_at_ms = u64::MAX;
            self.dry_off = false;
            return Vec::new();
        };
        if std::mem::take(&mut self.dry_off) {
            if let Some(shake) = set.shake_off {
                self.busy_until_ms = now_ms.saturating_add(u64::from(shake.duration_ms));
                return vec![Play::once(shake.animation_id)];
            }
        }
        if let Some(index) = self.rest {
            if now_ms < self.animation_at_ms {
                return Vec::new();
            }
            self.animation_at_ms = now_ms.saturating_add(self.gap(config));
            let fidgets = set.rest_postures.get(index).map_or(&[][..], |p| &p.fidgets);
            return self.play_any(fidgets, now_ms);
        }
        if now_ms >= self.rest_at_ms {
            self.rest_at_ms = u64::MAX;
            if let Some(index) = self.rng.index(set.rest_postures.len()) {
                let posture = &set.rest_postures[index];
                let looped = self
                    .rng
                    .index(posture.loops.len())
                    .map(|i| posture.loops[i]);
                self.rest = Some(index);
                self.busy_until_ms = now_ms.saturating_add(u64::from(posture.enter.duration_ms));
                self.animation_at_ms = self.busy_until_ms.saturating_add(self.gap(config));
                // The loop is the base animation; the enter transition plays over it.
                return looped
                    .map(Play::base)
                    .into_iter()
                    .chain([Play::once(posture.enter.animation_id)])
                    .collect();
            }
        }
        if now_ms < self.animation_at_ms {
            return Vec::new();
        }
        self.animation_at_ms = now_ms.saturating_add(self.gap(config));
        let bathroom = !set.bathroom.is_empty() && self.rng.chance(config.bathroom_chance);
        self.play_any(
            if bathroom {
                &set.bathroom
            } else {
                &set.animations
            },
            now_ms,
        )
    }

    /// Play one random row now; the next animation waits for it to end.
    fn play_any(&mut self, rows: &[PetAnimation], now_ms: u64) -> Vec<Play> {
        let Some(index) = self.rng.index(rows.len()) else {
            return Vec::new();
        };
        let animation = rows[index];
        self.busy_until_ms = now_ms.saturating_add(u64::from(animation.duration_ms));
        self.animation_at_ms = self.animation_at_ms.max(self.busy_until_ms);
        vec![Play::once(animation.animation_id)]
    }

    /// The pet stopped being settled: a pet that lay down gets back on its feet (the base
    /// animation returns to standing, then the exit transition plays); a standing one has nothing
    /// to undo. No timer runs until it settles again.
    fn stand_up(&mut self, set: Option<&PetIdleSet>, now_ms: u64) -> Vec<Play> {
        self.animation_at_ms = u64::MAX;
        self.rest_at_ms = u64::MAX;
        let Some(index) = self.rest.take() else {
            return Vec::new();
        };
        let mut plays = vec![Play::base(ANIM_ID_STAND)];
        if let Some(posture) = set.and_then(|set| set.rest_postures.get(index)) {
            self.busy_until_ms = now_ms.saturating_add(u64::from(posture.exit.duration_ms));
            plays.push(Play::once(posture.exit.animation_id));
        }
        plays
    }
}

/// 35/27 (reader `008d7880`): the actor, the icon, the seconds the client keeps it, and the clear
/// flag, which stays 0: a bubble ends by itself and the next one replaces it (`0096c120`).
fn thought_bubble_packet(actor_guid: u64, icon: i32, seconds: u32) -> Vec<u8> {
    let mut bytes = PLAYER_UPDATE.to_le_bytes().to_vec();
    bytes.extend(THOUGHT_BUBBLE.to_le_bytes());
    bytes.extend(actor_guid.to_le_bytes());
    bytes.extend(icon.to_le_bytes());
    bytes.extend(seconds.to_le_bytes());
    bytes.push(0);
    bytes
}

impl PetService {
    /// The seed of the idle choices of the next summon. A summon mixes it with its owner and
    /// time, so two sessions differ even on the default; tests set it to replay one.
    pub fn set_idle_seed(&mut self, seed: u64) {
        self.idle_seed = seed;
    }

    /// The model's idle rows, when it has any.
    fn idle_set(&self, pet_id: u32) -> Option<&PetIdleSet> {
        let pet = self.pet(pet_id)?;
        self.content
            .model(self.content.model_id(&pet.model))?
            .idle
            .as_ref()
    }

    /// The world tick of the summoned pet's idle life and thought bubbles; both go to the owner
    /// (`private`) and the viewers (`shared`). Costs a few comparisons per tick: the model's rows
    /// are read only when something is due. Nothing here is saved.
    pub(super) fn idle_tick(&mut self, config: &IdleConfig, wire: &mut PetWire, now_ms: u64) {
        let Some(active) = self.active else { return };
        let mut state = active.idle;
        let settled = active.pose.settled() && !active.in_water && !active.interacting();
        let mut plays = Vec::new();
        if settled {
            if !state.settled {
                state.settled = true;
                state.animation_at_ms = now_ms.saturating_add(config.idle_after_ms);
                state.rest_at_ms = now_ms.saturating_add(config.rest_after_ms);
            }
            if state.due(now_ms) {
                plays = state.advance(config, self.idle_set(active.pet_id), now_ms);
            }
        } else if std::mem::take(&mut state.settled) {
            plays = state.stand_up(self.idle_set(active.pet_id), now_ms);
        }
        let mut packets: Vec<Vec<u8>> = plays
            .iter()
            .map(|play| {
                piece_052::animation_packet(active.actor_guid, play.animation_id, play.base)
            })
            .collect();
        match state.bubble_at_ms {
            None => state.schedule_bubble(config, now_ms),
            Some(at) if now_ms >= at => {
                state.schedule_bubble(config, now_ms);
                packets.extend(self.bubble(config, now_ms));
            }
            Some(_) => {}
        }
        if let Some(active) = self.active.as_mut() {
            active.idle = state;
        }
        for packet in packets {
            wire.private.push(packet.clone());
            wire.shared.push(packet);
        }
    }

    /// The thought bubble the summoned pet shows now: its lowest need's icon when that need is
    /// under the threshold (food, then groom, then play on a tie), else the sad icon for a low
    /// mood, else none. A vanity pet has none, and a clock that ran backwards shows none.
    fn bubble(&self, config: &IdleConfig, now_ms: u64) -> Option<Vec<u8>> {
        let active = self.active?;
        let pet = self.pet(active.pet_id)?;
        if self.content.is_vanity(self.content.model_id(&pet.model)) {
            return None;
        }
        let needs = decayed(&self.content.need_policy, pet, true, now_ms / 1_000)?;
        let (lowest, icon) = [
            (needs.food, config.hunger_icon),
            (needs.groom, config.hygiene_icon),
            (needs.play, config.play_icon),
        ]
        .into_iter()
        .reduce(|best, next| if next.0 < best.0 { next } else { best })?;
        let icon = if lowest < config.need_bubble_threshold {
            icon
        } else if needs.mood < config.mood_bubble_threshold {
            config.sad_icon
        } else {
            return None;
        };
        Some(thought_bubble_packet(
            active.actor_guid,
            icon,
            config.bubble_seconds,
        ))
    }
}
