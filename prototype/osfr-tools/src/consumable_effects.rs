//! T473: the per-player timed effect stack behind treats, cakes and transformations. One
//! transform (temporary model), one food aura and one size change can be live at a time; each
//! has a buff-bar row. Pure state and no I/O: the caller owns the clock and the recipient lists,
//! and sends what `Emit` returns (`gameplay/item_use.rs` does, T474).
//!
//! Wire, every reader checked in the client (dispatcher `0092f460`):
//! * 35/13 scale: ctor `008b42b0`, reader `008d71b0` (u64 guid, f32 scale); applied by `00962200`.
//! * 35/14 temporary appearance: ctor `008b4370`, reader `008d6890` (i32 model, then u64 guid);
//!   applier `ProxiedCharacter::SwitchToTemporaryAppearance` `0096bd90`.
//! * 35/15 revert: ctor `008b4310`, reader `008d7230` (u64 guid, i32 0).
//! * 35/16 poof: `encode_play_composite_effect`, reader `008e5f50`.
//! * 35/33 strip request from the buff bar's cancel: ctor `008a0fb0` (class
//!   `PlayerUpdatePacketRequestStripEffect`), sender `00934740`; its serializer is `00913b80`,
//!   which writes exactly one i32 (the dword at +0xC); the dword at +0x18 is never written.
//! * 35/41 and 35/42 composite effect on an actor: readers `008d7ad0` and `008d7bc0` (the
//!   piece_056 encoders); 38/16 and 38/17 buff rows: `build_effect_tag`, reader `008d8870`.
//!
//! Aura resync (R): OSFR saw the client drop tags across a temporary-appearance change and ignore
//! a repeated tag id, so every 35/14 or 35/15 is followed by 35/42 (old tag) and 35/41 (fresh
//! tag) for each started aura. The buff id never changes.

use crate::abilities::{
    build_effect_tag,
    effect_packets::{
        encode_add_composite_effect, encode_play_composite_effect, encode_remove_composite_effect,
        encode_remove_effect_tag,
    },
};

// (R) Sanity bounds, not tuning values: a hostile or buggy caller gets a Refusal, never a panic.
pub const MAX_DURATION_MS: u64 = 86_400_000;
pub const MAX_DELAY_MS: u64 = 10_000;
pub const MIN_SCALE: f32 = 0.25;
pub const MAX_SCALE: f32 = 5.0;

// (R) Buff ids and world tags live in their own id ranges, far above the captured startup tags
// (2 and 4), the job tags (composite ids such as 15255) and the power-up tags (definition ids).
const BUFF_ID_BASE: i32 = 1_000_000_000;
const WORLD_TAG_BASE: i32 = 1_500_000_000;
const ID_RANGE: i32 = 500_000_000;

/// Reads the client's 35/33 (`u16 35, u16 33, i32 id`, 8 bytes, serializer `00913b80`). `Ok(None)`
/// means some other packet; a short or long 35/33 is an error for that session only.
pub fn parse_strip(inner: &[u8]) -> std::io::Result<Option<i32>> {
    if !inner.starts_with(&[35, 0, 33, 0]) {
        return Ok(None);
    }
    let body: [u8; 4] = inner[4..].try_into().map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "35/33 strip request is not one i32",
        )
    })?;
    Ok(Some(i32::from_le_bytes(body)))
}

/// 35/13: the actor's model scale.
pub fn encode_update_scale(guid: u64, scale: f32) -> Vec<u8> {
    let mut bytes = vec![35, 0, 13, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.extend(scale.to_le_bytes());
    bytes
}

/// 35/14: swap to a temporary model.
pub fn encode_temporary_appearance(model_id: i32, guid: u64) -> Vec<u8> {
    let mut bytes = vec![35, 0, 14, 0];
    bytes.extend(model_id.to_le_bytes());
    bytes.extend(guid.to_le_bytes());
    bytes
}

/// 35/15: drop the temporary model.
pub fn encode_remove_temporary_appearance(guid: u64) -> Vec<u8> {
    let mut bytes = vec![35, 0, 15, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// `poof_effect_id` 0 plays no poof.
    Transform {
        model_id: i32,
        poof_effect_id: i32,
    },
    /// The looping food aura; it starts `delay_ms` after it is applied (0 means now).
    Aura {
        composite_effect_id: i32,
        delay_ms: u64,
    },
    Scale {
        scale: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spec {
    pub kind: Kind,
    pub duration_ms: u64,
    pub icon_id: i32,
    pub name_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Duration,
    Delay,
    Scale,
    /// A transform model or aura composite effect id that is not positive.
    Id,
}

/// Packets for one `apply`, `tick` or `strip`. 38/16 and 38/17 go to the owner only because the
/// buff bar is theirs; the 35 family goes to both lists.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Emit {
    pub to_self: Vec<Vec<u8>>,
    pub to_nearby: Vec<Vec<u8>>,
}

impl Emit {
    pub fn is_empty(&self) -> bool {
        self.to_self.is_empty() && self.to_nearby.is_empty()
    }

    fn both(&mut self, packet: Vec<u8>) {
        self.to_nearby.push(packet.clone());
        self.to_self.push(packet);
    }
}

/// What the actor should look like now, for the AddPc snapshot (`players.rs`, T512).
#[derive(Clone, Debug, PartialEq)]
pub struct Visual {
    pub temporary_model: i32,
    pub scale: f32,
    /// (world tag, composite effect id) per started aura.
    pub auras: Vec<(i32, i32)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Live {
    buff_id: i32,
    spec: Spec,
    ends_at: u64,
    /// Aura only: when the 35/41 starts, and the world tag once it has.
    start_at: u64,
    world_tag: Option<i32>,
}

const TRANSFORM: usize = 0;
const AURA: usize = 1;
const SCALE: usize = 2;

/// One player's live effects. Session-only: nothing here is saved.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectStack {
    slots: [Option<Live>; 3],
    next_buff_id: i32,
    next_world_tag: i32,
}

impl Default for EffectStack {
    fn default() -> Self {
        Self {
            slots: [None; 3],
            next_buff_id: BUFF_ID_BASE,
            next_world_tag: WORLD_TAG_BASE,
        }
    }
}

/// The next id of a range, skipping any that is still live and wrapping at the top.
fn next_id(cursor: &mut i32, base: i32, live: &[i32]) -> i32 {
    loop {
        let id = *cursor;
        *cursor = id
            .checked_add(1)
            .filter(|next| *next < base + ID_RANGE)
            .unwrap_or(base);
        if !live.contains(&id) {
            return id;
        }
    }
}

impl EffectStack {
    /// A pure check: callers stage with it before they spend an item.
    pub fn admits(&self, spec: &Spec) -> Result<(), Refusal> {
        if !(1..=MAX_DURATION_MS).contains(&spec.duration_ms) {
            return Err(Refusal::Duration);
        }
        match spec.kind {
            Kind::Transform { model_id, .. } if model_id <= 0 => Err(Refusal::Id),
            Kind::Aura {
                composite_effect_id,
                ..
            } if composite_effect_id <= 0 => Err(Refusal::Id),
            Kind::Aura { delay_ms, .. } if delay_ms > MAX_DELAY_MS => Err(Refusal::Delay),
            Kind::Scale { scale } if !(MIN_SCALE..=MAX_SCALE).contains(&scale) => {
                Err(Refusal::Scale)
            }
            _ => Ok(()),
        }
    }

    /// Adds `spec`, replacing the live effect of its kind, and returns the new buff id.
    /// A refusal changes nothing.
    pub fn apply(
        &mut self,
        guid: u64,
        position: [f32; 3],
        spec: Spec,
        now_ms: u64,
    ) -> Result<(i32, Emit), Refusal> {
        self.admits(&spec)?;
        let mut emit = Emit::default();
        let slot = match spec.kind {
            Kind::Transform { .. } => TRANSFORM,
            Kind::Aura { .. } => AURA,
            Kind::Scale { .. } => SCALE,
        };
        if let Some(old) = self.slots[slot].take() {
            retire(guid, old, false, &mut emit);
        }
        let live_ids: Vec<i32> = self.slots.iter().flatten().map(|l| l.buff_id).collect();
        let buff_id = next_id(&mut self.next_buff_id, BUFF_ID_BASE, &live_ids);
        let live = Live {
            buff_id,
            spec,
            ends_at: now_ms.saturating_add(spec.duration_ms),
            start_at: now_ms.saturating_add(match spec.kind {
                Kind::Aura { delay_ms, .. } => delay_ms,
                _ => 0,
            }),
            world_tag: None,
        };
        let row = build_effect_tag(buff_id, spec.duration_ms, spec.icon_id, spec.name_id);
        match spec.kind {
            Kind::Transform {
                model_id,
                poof_effect_id,
            } => {
                if poof_effect_id != 0 {
                    emit.both(encode_play_composite_effect(guid, poof_effect_id, position));
                }
                emit.both(encode_temporary_appearance(model_id, guid));
                self.slots[TRANSFORM] = Some(live);
                self.resync(guid, &mut emit);
                emit.to_self.push(row);
            }
            Kind::Aura { delay_ms, .. } => {
                emit.to_self.push(row);
                self.slots[AURA] = Some(live);
                if delay_ms == 0 {
                    self.start_aura(guid, &mut emit);
                }
            }
            Kind::Scale { scale } => {
                emit.both(encode_update_scale(guid, scale));
                emit.to_self.push(row);
                self.slots[SCALE] = Some(live);
            }
        }
        Ok((buff_id, emit))
    }

    /// Starts delayed auras and expires each effect exactly once. Allocates nothing when
    /// nothing is due. A clock that went backwards fires nothing: every deadline is absolute.
    pub fn tick(&mut self, guid: u64, now_ms: u64) -> Emit {
        let mut emit = Emit::default();
        let due = |live: &Option<Live>| live.is_some_and(|l| now_ms >= l.ends_at);
        let transform_ended = due(&self.slots[TRANSFORM]);
        if transform_ended {
            self.end(guid, TRANSFORM, &mut emit);
        }
        if due(&self.slots[AURA]) {
            self.end(guid, AURA, &mut emit);
        }
        if transform_ended {
            self.resync(guid, &mut emit);
        }
        if self.slots[AURA].is_some_and(|l| l.world_tag.is_none() && now_ms >= l.start_at) {
            self.start_aura(guid, &mut emit);
        }
        if due(&self.slots[SCALE]) {
            self.end(guid, SCALE, &mut emit);
        }
        emit
    }

    /// Ends an effect this stack owns, with the packets of an expiry; an unknown id is `None`.
    pub fn strip(&mut self, guid: u64, buff_id: i32) -> Option<Emit> {
        let slot = self
            .slots
            .iter()
            .position(|l| l.is_some_and(|l| l.buff_id == buff_id))?;
        let mut emit = Emit::default();
        self.end(guid, slot, &mut emit);
        if slot == TRANSFORM {
            self.resync(guid, &mut emit);
        }
        Some(emit)
    }

    pub fn visual(&self) -> Visual {
        let transform = self.slots[TRANSFORM].map(|l| l.spec.kind);
        let scale = self.slots[SCALE].map(|l| l.spec.kind);
        Visual {
            temporary_model: match transform {
                Some(Kind::Transform { model_id, .. }) => model_id,
                _ => 0,
            },
            scale: match scale {
                Some(Kind::Scale { scale }) => scale,
                _ => 1.0,
            },
            auras: self.started_aura().into_iter().collect(),
        }
    }

    /// Everything the owner's client needs to show the live effects again: the model, the size,
    /// each started aura and every buff row with its remaining time. `now_ms` is a deviation
    /// from the brief's signature: the remaining time needs the clock.
    pub fn resend_to_self(&self, guid: u64, now_ms: u64) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        let visual = self.visual();
        if visual.temporary_model != 0 {
            packets.push(encode_temporary_appearance(visual.temporary_model, guid));
        }
        if self.slots[SCALE].is_some() {
            packets.push(encode_update_scale(guid, visual.scale));
        }
        for (tag, composite) in visual.auras {
            packets.push(encode_add_composite_effect(guid, tag, composite, guid));
        }
        for live in self.slots.iter().flatten() {
            let remaining = live.ends_at.saturating_sub(now_ms);
            packets.push(build_effect_tag(
                live.buff_id,
                remaining,
                live.spec.icon_id,
                live.spec.name_id,
            ));
        }
        packets
    }

    fn started_aura(&self) -> Option<(i32, i32)> {
        let live = self.slots[AURA]?;
        match (live.world_tag, live.spec.kind) {
            (
                Some(tag),
                Kind::Aura {
                    composite_effect_id,
                    ..
                },
            ) => Some((tag, composite_effect_id)),
            _ => None,
        }
    }

    /// Ends the effect in `slot` as an expiry (the transform reverts, the size resets).
    fn end(&mut self, guid: u64, slot: usize, emit: &mut Emit) {
        if let Some(live) = self.slots[slot].take() {
            retire(guid, live, true, emit);
        }
    }

    /// 35/41 for the aura in its slot, with a fresh world tag.
    fn start_aura(&mut self, guid: u64, emit: &mut Emit) {
        let tag = next_id(&mut self.next_world_tag, WORLD_TAG_BASE, &[]);
        if let Some(live) = self.slots[AURA].as_mut()
            && let Kind::Aura {
                composite_effect_id,
                ..
            } = live.spec.kind
        {
            live.world_tag = Some(tag);
            emit.both(encode_add_composite_effect(
                guid,
                tag,
                composite_effect_id,
                guid,
            ));
        }
    }

    /// The aura resync after a 35/14 or 35/15: 35/42 with the old tag, 35/41 with a fresh one.
    fn resync(&mut self, guid: u64, emit: &mut Emit) {
        let Some(old) = self.slots[AURA].and_then(|l| l.world_tag) else {
            return;
        };
        emit.both(encode_remove_composite_effect(guid, old));
        let tag = next_id(&mut self.next_world_tag, WORLD_TAG_BASE, &[old]);
        if let Some(live) = self.slots[AURA].as_mut()
            && let Kind::Aura {
                composite_effect_id,
                ..
            } = live.spec.kind
        {
            live.world_tag = Some(tag);
            emit.both(encode_add_composite_effect(
                guid,
                tag,
                composite_effect_id,
                guid,
            ));
        }
    }
}

/// The packets that take `live` off the actor and the buff bar. `revert` is false when a new
/// effect of the same kind replaces it and sets the model or size itself.
fn retire(guid: u64, live: Live, revert: bool, emit: &mut Emit) {
    match live.spec.kind {
        Kind::Transform { .. } if revert => emit.both(encode_remove_temporary_appearance(guid)),
        Kind::Aura { .. } => {
            if let Some(tag) = live.world_tag {
                emit.both(encode_remove_composite_effect(guid, tag));
            }
        }
        Kind::Scale { .. } if revert => emit.both(encode_update_scale(guid, 1.0)),
        _ => {}
    }
    emit.to_self.push(encode_remove_effect_tag(live.buff_id));
}
