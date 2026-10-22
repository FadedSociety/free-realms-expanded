//! Work item 056: combat status effects - the recovered effect-tag wire, the client buff-row
//! contract, and a bounded server-side status ledger.
//!
//! ## ORIGINAL facts used here (paths are under the private lab, outside this repository)
//!
//! * `decompiled/server/Gateway/Gateway.Login/ClientPcData.cs` declares `EffectTag` (19 members) and
//!   `ClientEffectTag : EffectTag` (4 more) and serializes them with .NET `BinaryWriter` widths, so
//!   a `ClientEffectTag` is 72 + 13 = 85 bytes. The same file holds
//!   `List<(int, ClientEffectTag)> EffectTags` and writes it into the self-to-client payload as
//!   `count` followed by `(int TagId, ClientEffectTag)` per row. A captured retail character
//!   (`lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json`) carries exactly
//!   one such row.
//! * Packet ids: `BaseClientUpdatePackets.cs` `AddEffectTag = 16`, `RemoveEffectTag = 17`;
//!   `BasePlayerUpdatePackets.cs` `UpdateCharacterState = 20`, `AddEffectTagCompositeEffect = 41`,
//!   `RemoveEffectTagCompositeEffect = 42`, `EffectTagCompositeEffectsEnable = 43`;
//!   `BasePackets.cs` `BasePlayerUpdatePacket = 35`, `BaseClientUpdatePacket = 38`.
//! * The original client sets those numbers as *sub* opcodes. Its constructors call
//!   `FUN_0089d180(0x10)` / `(0x11)` before the `ClientUpdatePacketAddEffectTag` /
//!   `ClientUpdatePacketRemoveEffectTag` vftable stores, and `FUN_0089fbb0(0x29)` / `0x2a` /
//!   `0x2b` / `0x14` before the `PlayerUpdatePacketAddEffectTagCompositeEffect` /
//!   `RemoveEffectTagCompositeEffect` / `EffectTagCompositeEffectsEnable` / `UpdateCharacterState`
//!   stores. `FUN_0089fbb0` receives `0x23` for `PlayerUpdatePacketHitPointModification`, whose
//!   enum value is 35, so the composite-effect packets are 35/41 and 35/42 over the wire.
//! * `decompiled/client-ui/ScriptsBase.lua`: `GroupHandler.BuffsDsName =
//!   "BaseClient.PlayerEffectTags"`; each row is read as instance, icon, duration, timeLeft, name;
//!   `AddAbilityEffect` returns early when the icon is 0; `BuffsUpdated` removes any local buff row
//!   the datasource no longer carries.
//! * The captured row itself: TagId 2, `unknown_2` 495, duration 3600, `unknown_10`/`unknown_11`
//!   142556/3978, `unknown_20` 7334, `unknown_22` 4270. Icon 7334 is the image of original stat 37
//!   "Coin Boost" (`Resources/CharacterStatDefinitions.txt`, NameId 140800) and 4270 resolves to
//!   "Power Hour Coin Boost" in the recovered locale export - a one hour coin boost, matching the
//!   3600 second duration. `composite_effect_id` (member 13) is 0 for that row.
//! * Immunity vocabulary from original locale strings: 70204 "immune to stun", 70209 "immune to
//!   slow", 70214 "immune to poison", 436620 "Forced Effect Immunity", 436622 "Illusion Immunity",
//!   436675 "Fear Immunity", 438016 "Knockback Immunity", 438557 "Death Prevent + Knockback/Root
//!   Immunity", 438559 "Status Immunity".
//!
//! ## RECONSTRUCTED policy (labelled; not historical behaviour)
//!
//! * Tag-id allocation (monotonic from 1) and the "same definition from the same source refreshes
//!   one row, a different source stacks an independent row" stacking rule. The client's store is
//!   keyed by tag id, so either outcome is representable; the recovered corpus has no server-side
//!   apply code to settle which one the original used.
//! * Immunity categories are caller-owned state. The module only refuses to create a row for a kind
//!   the caller marked immune, and never computes damage, healing or drain from an effect.
//! * Offline expiry: a restored row keeps its id and loses `downtime_seconds` from its duration.
//!   Rows whose duration elapses during downtime are dropped.
//! * `MAX_EFFECT_SECONDS` is a local sanity bound; the recovered data proves 1800 s transformations
//!   and this 3600 s buff, not an original maximum.
//!
//! ## GAPS (explicit)
//!
//! * The meanings of `unknown_9`, `unknown_10`/`unknown_11`, `unknown_15`/`unknown_16` and the
//!   remaining members are unrecovered; only instance id (`unknown_1`), duration, icon, name and
//!   composite effect are pinned.
//! * `EffectTagCompositeEffectsEnable` (35/43) has a known id and no recovered body.
//! * No server-side effect application, stacking, immunity or cleanse logic was recovered: the
//!   supplied server only ships one hard-coded demonstration tag on login.
//! * The 97-byte hex literal in the supplied server's `UpdateStats.cs` is a whole 38/16 packet:
//!   header, tag id 4, blob length 85, the 85-byte row. It is the same row as the verified startup
//!   reference `0037-ready.bin`, and `tests/piece_056.rs` decodes both captured startup rows.

use std::collections::BTreeMap;

pub const CLIENT_UPDATE_OPCODE: u16 = 38;
pub const CLIENT_UPDATE_SUB_ADD_EFFECT_TAG: u16 = 16;
pub const CLIENT_UPDATE_SUB_REMOVE_EFFECT_TAG: u16 = 17;

pub const PLAYER_UPDATE_OPCODE: u16 = 35;
pub const PLAYER_UPDATE_SUB_UPDATE_CHARACTER_STATE: u16 = 20;
pub const PLAYER_UPDATE_SUB_ADD_COMPOSITE_EFFECT: u16 = 41;
pub const PLAYER_UPDATE_SUB_REMOVE_COMPOSITE_EFFECT: u16 = 42;
pub const PLAYER_UPDATE_SUB_EFFECT_TAG_COMPOSITE_EFFECTS_ENABLE: u16 = 43;

/// The 19 `EffectTag` members at `BinaryWriter` widths.
pub const EFFECT_TAG_LEN: usize = 72;
/// `ClientEffectTag` adds IconId, a bool, NameId and one more int.
pub const CLIENT_EFFECT_TAG_LEN: usize = EFFECT_TAG_LEN + 13;
/// Header, tag id, i32 blob length, then the blob (client reader `0x008d8870`).
pub const ADD_EFFECT_TAG_LEN: usize = 4 + 4 + 4 + CLIENT_EFFECT_TAG_LEN;
pub const REMOVE_EFFECT_TAG_LEN: usize = 4 + 4;
pub const ADD_COMPOSITE_EFFECT_LEN: usize = 4 + 8 + 4 + 4 + 8 + 8;
pub const REMOVE_COMPOSITE_EFFECT_LEN: usize = 4 + 8 + 4;
pub const CHARACTER_STATE_LEN: usize = 4 + 8 + 4;

/// Local sanity bound for a reconstructed row; not a recovered original maximum.
pub const MAX_EFFECT_SECONDS: i32 = 86_400;
/// First tag id handed out by [`StatusLedger`]. The original allocation rule is unrecovered.
pub const FIRST_TAG_ID: i32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusError {
    BadLength {
        expected: usize,
        got: usize,
    },
    BadOpcode {
        expected: (u16, u16),
        got: (u16, u16),
    },
    InvalidDuration(i32),
    InvalidDefinition(i32),
    UnnamedBuff,
    UnknownTag(i32),
    Immune(StatusKind),
}

impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusError::BadLength { expected, got } => {
                write!(f, "expected {expected} bytes, got {got}")
            }
            StatusError::BadOpcode { expected, got } => {
                write!(f, "expected opcode {expected:?}, got {got:?}")
            }
            StatusError::InvalidDuration(value) => write!(f, "invalid effect duration {value}"),
            StatusError::InvalidDefinition(value) => write!(f, "invalid effect definition {value}"),
            StatusError::UnnamedBuff => write!(f, "a visible buff row needs a name id"),
            StatusError::UnknownTag(tag_id) => write!(f, "no active effect with tag id {tag_id}"),
            StatusError::Immune(kind) => write!(f, "the target is immune to {kind:?}"),
        }
    }
}

impl std::error::Error for StatusError {}

/// The recovered detrimental-effect kinds plus the generic status entry.
///
/// `immune_locale_id` returns the original locale string that names an immunity for the kind. The
/// recovered corpus only carries dedicated strings for stun, slow, poison, root, knockback, fear,
/// illusion and forced effects, so sleep, silence and confuse fall back to the generic
/// "Status Immunity" string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusKind {
    Sleep,
    Silence,
    Root,
    Stun,
    Knockback,
    Confuse,
    Fear,
    Poison,
    Slow,
    Illusion,
    ForcedEffect,
    Status,
}

impl StatusKind {
    pub const ALL: [StatusKind; 12] = [
        StatusKind::Sleep,
        StatusKind::Silence,
        StatusKind::Root,
        StatusKind::Stun,
        StatusKind::Knockback,
        StatusKind::Confuse,
        StatusKind::Fear,
        StatusKind::Poison,
        StatusKind::Slow,
        StatusKind::Illusion,
        StatusKind::ForcedEffect,
        StatusKind::Status,
    ];

    pub const fn index(self) -> u32 {
        match self {
            StatusKind::Sleep => 0,
            StatusKind::Silence => 1,
            StatusKind::Root => 2,
            StatusKind::Stun => 3,
            StatusKind::Knockback => 4,
            StatusKind::Confuse => 5,
            StatusKind::Fear => 6,
            StatusKind::Poison => 7,
            StatusKind::Slow => 8,
            StatusKind::Illusion => 9,
            StatusKind::ForcedEffect => 10,
            StatusKind::Status => 11,
        }
    }

    /// Original locale string that names an immunity for this kind.
    pub const fn immune_locale_id(self) -> u32 {
        match self {
            StatusKind::Stun => 70204,
            StatusKind::Slow => 70209,
            StatusKind::Poison => 70214,
            StatusKind::Knockback => 438016,
            StatusKind::Root => 438557,
            StatusKind::Status => 438559,
            StatusKind::Illusion => 436622,
            StatusKind::Fear => 436675,
            StatusKind::ForcedEffect => 436620,
            StatusKind::Sleep | StatusKind::Silence | StatusKind::Confuse => 438559,
        }
    }

    /// COMMUNITY: the character-state flags recovered in the pinned Sanctuary PR research.
    /// `None` means the pinned flags carry no bit for the kind.
    pub const fn character_state_bit(self) -> Option<i32> {
        match self {
            StatusKind::Fear => Some(0x2),
            StatusKind::Sleep => Some(0x4),
            StatusKind::Silence => Some(0x8),
            StatusKind::Root => Some(0x20),
            StatusKind::Stun => Some(0x40),
            StatusKind::Knockback => Some(0x200),
            StatusKind::Confuse => Some(0x2000),
            StatusKind::Poison
            | StatusKind::Slow
            | StatusKind::Illusion
            | StatusKind::ForcedEffect
            | StatusKind::Status => None,
        }
    }
}

/// Caller-owned immunity state, one bit per [`StatusKind`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImmunitySet(u32);

impl ImmunitySet {
    pub const fn none() -> Self {
        Self(0)
    }

    pub fn with(self, kind: StatusKind) -> Self {
        Self(self.0 | 1 << kind.index())
    }

    pub fn without(self, kind: StatusKind) -> Self {
        Self(self.0 & !(1 << kind.index()))
    }

    pub const fn contains(self, kind: StatusKind) -> bool {
        self.0 & (1 << kind.index()) != 0
    }
}

/// The original `ClientEffectTag` members in their recovered serialization order.
///
/// Members whose meaning is not recovered keep their original `Unknown*` ordinal; the four pinned
/// roles are `duration_seconds`, `icon_id`, `name_id` and `composite_effect_id`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectTagPayload {
    /// The client's instance id: reader `0x008e93e0` stores it at record +8, and the
    /// `PlayerEffectTags` getter `0x00c8e720` returns +8 as column 0, which `BuffsUpdated` keys
    /// one buff icon by and `CancelBuff` sends back. Writers set it to the row's tag id: both
    /// captured startup rows have it (tag 2 -> 2, tag 4 -> 4; tag 4's `unknown_3` is 8, so it is
    /// not that field).
    pub unknown_1: i32,
    pub unknown_2: i32,
    pub unknown_3: i32,
    pub unknown_4: i32,
    pub unknown_5: i32,
    /// Float bits in the captured row (100.0f).
    pub unknown_6: i32,
    pub unknown_7: i32,
    pub unknown_8: bool,
    pub unknown_9: i64,
    pub unknown_10: i32,
    pub unknown_11: i32,
    pub unknown_12: i32,
    pub composite_effect_id: i32,
    pub unknown_14: i64,
    pub unknown_15: i32,
    pub unknown_16: i32,
    pub unknown_17: bool,
    pub unknown_18: bool,
    pub unknown_19: bool,
    pub unknown_20: i32,
    pub unknown_21: bool,
    pub unknown_22: i32,
    pub unknown_23: i32,
}

impl EffectTagPayload {
    /// The single effect tag captured on the supplied retail character.
    pub const fn captured_coin_boost() -> Self {
        Self {
            unknown_1: 2,
            unknown_2: 495,
            unknown_3: 2,
            unknown_4: 0,
            unknown_5: 0,
            unknown_6: 0x42c8_0000,
            unknown_7: 3600,
            unknown_8: false,
            unknown_9: 5_428_099_824_451_223_953,
            unknown_10: 142_556,
            unknown_11: 3_978,
            unknown_12: 0,
            composite_effect_id: 0,
            unknown_14: 0,
            unknown_15: 0,
            unknown_16: 6,
            unknown_17: false,
            unknown_18: true,
            unknown_19: true,
            unknown_20: 7334,
            unknown_21: false,
            unknown_22: 4270,
            unknown_23: 268,
        }
    }

    pub const fn duration_seconds(&self) -> i32 {
        self.unknown_7
    }

    pub const fn icon_id(&self) -> i32 {
        self.unknown_20
    }

    pub const fn name_id(&self) -> i32 {
        self.unknown_22
    }

    pub fn with_duration_seconds(mut self, seconds: i32) -> Self {
        self.unknown_7 = seconds;
        self
    }
}

/// One buff/world effect row requested by the caller.
///
/// The caller keeps one ledger per affected actor. `source_guid` is the only guid the request
/// carries: the pinned community writer repeats it as both the composite-effect `Guid` and
/// `SourceGuid` for self-applied effects, and no captured non-self example exists to separate them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectRequest {
    pub kind: StatusKind,
    pub definition_id: i32,
    pub source_guid: u64,
    pub duration_seconds: i32,
    pub icon_id: i32,
    pub name_id: i32,
    pub composite_effect_id: i32,
}

impl EffectRequest {
    /// A neutral `ClientEffectTag`: only the four recovered roles and the instance id carry caller
    /// data, every other member is zero because its meaning is unrecovered. `unknown_2` is used to
    /// carry the definition id so a persisted row can be matched again after a restart.
    fn to_payload(&self, tag_id: i32) -> EffectTagPayload {
        EffectTagPayload {
            unknown_1: tag_id,
            unknown_2: self.definition_id,
            unknown_3: 0,
            unknown_4: 0,
            unknown_5: 0,
            unknown_6: 0,
            unknown_7: self.duration_seconds,
            unknown_8: false,
            unknown_9: 0,
            unknown_10: 0,
            unknown_11: 0,
            unknown_12: 0,
            composite_effect_id: self.composite_effect_id,
            unknown_14: 0,
            unknown_15: 0,
            unknown_16: 0,
            unknown_17: false,
            unknown_18: false,
            unknown_19: false,
            unknown_20: self.icon_id,
            unknown_21: false,
            unknown_22: self.name_id,
            unknown_23: 0,
        }
    }
}

/// An active row held by [`StatusLedger`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveEffect {
    pub tag_id: i32,
    pub kind: StatusKind,
    pub definition_id: i32,
    pub source_guid: u64,
    pub expires_at_ms: u64,
    pub tag: EffectTagPayload,
}

impl ActiveEffect {
    /// Remaining whole seconds, or 0 once the row has expired.
    pub fn remaining_seconds(&self, now_ms: u64) -> i32 {
        if now_ms >= self.expires_at_ms {
            return 0;
        }
        let remaining_ms = self.expires_at_ms - now_ms;
        i32::try_from(remaining_ms / 1000)
            .unwrap_or(i32::MAX)
            .max(1)
    }

    /// The client only draws a buff row for a non-zero icon.
    pub fn has_buff_row(&self) -> bool {
        self.tag.icon_id() != 0
    }

    pub fn has_composite_effect(&self) -> bool {
        self.tag.composite_effect_id != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedEffect {
    pub tag_id: i32,
    pub refreshed: bool,
    pub packets: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpiredEffect {
    pub tag_id: i32,
    pub kind: StatusKind,
    pub packets: Vec<Vec<u8>>,
}

/// Per-actor status ledger. It emits only the recovered effect-tag packets; it grants nothing and
/// computes no gameplay numbers.
#[derive(Clone, Debug)]
pub struct StatusLedger {
    next_tag_id: i32,
    active: BTreeMap<i32, ActiveEffect>,
}

impl Default for StatusLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusLedger {
    pub fn new() -> Self {
        Self {
            next_tag_id: FIRST_TAG_ID,
            active: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active.is_empty()
    }

    pub fn get(&self, tag_id: i32) -> Option<&ActiveEffect> {
        self.active.get(&tag_id)
    }

    pub fn active(&self) -> impl Iterator<Item = &ActiveEffect> {
        self.active.values()
    }

    /// Apply one requested row. A duplicate of the same definition from the same source refreshes
    /// the existing row and keeps its tag id; any other request allocates a fresh one.
    pub fn apply(
        &mut self,
        request: EffectRequest,
        now_ms: u64,
        immunity: &ImmunitySet,
    ) -> Result<AppliedEffect, StatusError> {
        if immunity.contains(request.kind) {
            return Err(StatusError::Immune(request.kind));
        }
        if request.duration_seconds <= 0 || request.duration_seconds > MAX_EFFECT_SECONDS {
            return Err(StatusError::InvalidDuration(request.duration_seconds));
        }
        if request.definition_id <= 0 {
            return Err(StatusError::InvalidDefinition(request.definition_id));
        }
        if request.icon_id != 0 && request.name_id <= 0 {
            return Err(StatusError::UnnamedBuff);
        }

        let refreshed_on = self
            .active
            .values()
            .find(|effect| {
                effect.definition_id == request.definition_id
                    && effect.source_guid == request.source_guid
                    && effect.kind == request.kind
            })
            .map(|effect| effect.tag_id);
        let refreshed = refreshed_on.is_some();
        let tag_id = refreshed_on.unwrap_or_else(|| self.allocate());

        let tag = request.to_payload(tag_id);
        let expires_at_ms =
            now_ms.saturating_add(u64::try_from(request.duration_seconds).unwrap_or(0) * 1000);
        self.active.insert(
            tag_id,
            ActiveEffect {
                tag_id,
                kind: request.kind,
                definition_id: request.definition_id,
                source_guid: request.source_guid,
                expires_at_ms,
                tag,
            },
        );

        Ok(AppliedEffect {
            tag_id,
            refreshed,
            packets: effect_packets(tag_id, request.source_guid, &tag),
        })
    }

    /// Remove a row by tag id and return its removal packets.
    pub fn remove(&mut self, tag_id: i32) -> Result<Vec<Vec<u8>>, StatusError> {
        let effect = self
            .active
            .remove(&tag_id)
            .ok_or(StatusError::UnknownTag(tag_id))?;
        Ok(remove_packets(&effect))
    }

    /// Expire every row whose recorded duration has run out, once.
    pub fn tick(&mut self, now_ms: u64) -> Vec<ExpiredEffect> {
        let expired: Vec<i32> = self
            .active
            .values()
            .filter(|effect| effect.expires_at_ms <= now_ms)
            .map(|effect| effect.tag_id)
            .collect();
        expired
            .into_iter()
            .filter_map(|tag_id| {
                let effect = self.active.remove(&tag_id)?;
                Some(ExpiredEffect {
                    tag_id,
                    kind: effect.kind,
                    packets: remove_packets(&effect),
                })
            })
            .collect()
    }

    /// The rows the caller persists into the recovered self-to-client payload. Rows that have
    /// already run out are omitted and the stored duration becomes the remaining time.
    pub fn profile_rows(&self, now_ms: u64) -> Vec<(i32, EffectTagPayload)> {
        self.active
            .values()
            .filter_map(|effect| {
                let remaining = effect.remaining_seconds(now_ms);
                if remaining <= 0 {
                    return None;
                }
                Some((effect.tag_id, effect.tag.with_duration_seconds(remaining)))
            })
            .collect()
    }

    /// Rebuild a ledger from persisted rows. `now_ms` anchors the new clock and `downtime_seconds`
    /// is subtracted from every stored duration. The effect kind is not part of the recovered row,
    /// so restored rows carry [`StatusKind::Status`] until the caller reclassifies them.
    pub fn restore(
        rows: &[(i32, EffectTagPayload)],
        now_ms: u64,
        downtime_seconds: i32,
    ) -> Result<(Self, Vec<Vec<u8>>), StatusError> {
        let mut ledger = Self::new();
        let mut packets = Vec::new();
        for (tag_id, tag) in rows {
            let stored = tag.duration_seconds();
            let remaining = stored - downtime_seconds.max(0);
            if remaining <= 0 {
                continue;
            }
            let tag = EffectTagPayload {
                unknown_1: *tag_id,
                ..tag.with_duration_seconds(remaining)
            };
            let effect = ActiveEffect {
                tag_id: *tag_id,
                kind: StatusKind::Status,
                definition_id: tag.unknown_2,
                source_guid: 0,
                expires_at_ms: now_ms.saturating_add(u64::try_from(remaining).unwrap_or(0) * 1000),
                tag,
            };
            ledger.next_tag_id = ledger.next_tag_id.max(tag_id.saturating_add(1));
            packets.extend(effect_packets(*tag_id, effect.source_guid, &tag));
            ledger.active.insert(*tag_id, effect);
        }
        Ok((ledger, packets))
    }

    /// Drop every row for a session that does not persist them.
    pub fn disconnect_clear(&mut self) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        for effect in self.active.values() {
            packets.extend(remove_packets(effect));
        }
        self.active.clear();
        packets
    }

    fn allocate(&mut self) -> i32 {
        if self.next_tag_id < FIRST_TAG_ID {
            self.next_tag_id = FIRST_TAG_ID;
        }
        while self.active.contains_key(&self.next_tag_id) {
            self.next_tag_id = self.next_tag_id.saturating_add(1);
        }
        let tag_id = self.next_tag_id;
        self.next_tag_id = self.next_tag_id.saturating_add(1);
        tag_id
    }
}

fn effect_packets(tag_id: i32, source_guid: u64, tag: &EffectTagPayload) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();
    if tag.icon_id() != 0 {
        packets.push(encode_add_effect_tag(tag_id, tag));
    }
    if tag.composite_effect_id != 0 {
        packets.push(encode_add_composite_effect(
            source_guid,
            tag_id,
            tag.composite_effect_id,
            source_guid,
        ));
    }
    packets
}

fn remove_packets(effect: &ActiveEffect) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();
    if effect.has_buff_row() {
        packets.push(encode_remove_effect_tag(effect.tag_id));
    }
    if effect.has_composite_effect() {
        packets.push(encode_remove_composite_effect(
            effect.source_guid,
            effect.tag_id,
        ));
    }
    packets
}

/// Header for the 38 family.
fn client_header(sub: u16) -> [u8; 4] {
    let mut bytes = [0_u8; 4];
    bytes[..2].copy_from_slice(&CLIENT_UPDATE_OPCODE.to_le_bytes());
    bytes[2..].copy_from_slice(&sub.to_le_bytes());
    bytes
}

/// Header for the 35 family.
fn player_header(sub: u16) -> [u8; 4] {
    let mut bytes = [0_u8; 4];
    bytes[..2].copy_from_slice(&PLAYER_UPDATE_OPCODE.to_le_bytes());
    bytes[2..].copy_from_slice(&sub.to_le_bytes());
    bytes
}

pub fn encode_add_effect_tag(tag_id: i32, tag: &EffectTagPayload) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(ADD_EFFECT_TAG_LEN);
    bytes.extend(client_header(CLIENT_UPDATE_SUB_ADD_EFFECT_TAG));
    push_i32(&mut bytes, tag_id);
    // Reader 0x008d8870 -> 0x008bfc00 reads an i32 length, then that many blob bytes. The parse
    // is strict (case 16 at 0x0093a5fd passes 0 to 0x008f8ef0): any trailing byte drops the row.
    push_i32(&mut bytes, CLIENT_EFFECT_TAG_LEN as i32);
    push_payload(&mut bytes, tag);
    bytes
}

pub fn decode_add_effect_tag(bytes: &[u8]) -> Result<(i32, EffectTagPayload), StatusError> {
    check(
        bytes,
        (CLIENT_UPDATE_OPCODE, CLIENT_UPDATE_SUB_ADD_EFFECT_TAG),
        ADD_EFFECT_TAG_LEN,
    )?;
    let tag_id = read_i32(bytes, 4)?;
    let blob_len = read_i32(bytes, 8)?;
    if blob_len != CLIENT_EFFECT_TAG_LEN as i32 {
        return Err(StatusError::BadLength {
            expected: CLIENT_EFFECT_TAG_LEN,
            got: usize::try_from(blob_len).unwrap_or(0),
        });
    }
    Ok((tag_id, read_payload(bytes, 12)?))
}

pub fn encode_remove_effect_tag(tag_id: i32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(REMOVE_EFFECT_TAG_LEN);
    bytes.extend(client_header(CLIENT_UPDATE_SUB_REMOVE_EFFECT_TAG));
    push_i32(&mut bytes, tag_id);
    bytes
}

pub fn decode_remove_effect_tag(bytes: &[u8]) -> Result<i32, StatusError> {
    check(
        bytes,
        (CLIENT_UPDATE_OPCODE, CLIENT_UPDATE_SUB_REMOVE_EFFECT_TAG),
        REMOVE_EFFECT_TAG_LEN,
    )?;
    read_i32(bytes, 4)
}

/// The `PlayerUpdatePacketAddEffectTagCompositeEffect` body. The trailing `unused` long the pinned
/// writer emits is zero-filled because its meaning is unrecovered.
pub fn encode_add_composite_effect(
    guid: u64,
    tag_id: i32,
    composite_effect_id: i32,
    source_guid: u64,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(ADD_COMPOSITE_EFFECT_LEN);
    bytes.extend(player_header(PLAYER_UPDATE_SUB_ADD_COMPOSITE_EFFECT));
    bytes.extend(guid.to_le_bytes());
    push_i32(&mut bytes, tag_id);
    push_i32(&mut bytes, composite_effect_id);
    bytes.extend(source_guid.to_le_bytes());
    bytes.extend(0_i64.to_le_bytes());
    bytes
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompositeEffect {
    pub guid: u64,
    pub tag_id: i32,
    pub composite_effect_id: i32,
    pub source_guid: u64,
}

pub fn decode_add_composite_effect(bytes: &[u8]) -> Result<CompositeEffect, StatusError> {
    check(
        bytes,
        (PLAYER_UPDATE_OPCODE, PLAYER_UPDATE_SUB_ADD_COMPOSITE_EFFECT),
        ADD_COMPOSITE_EFFECT_LEN,
    )?;
    Ok(CompositeEffect {
        guid: read_u64(bytes, 4)?,
        tag_id: read_i32(bytes, 12)?,
        composite_effect_id: read_i32(bytes, 16)?,
        source_guid: read_u64(bytes, 20)?,
    })
}

pub fn encode_remove_composite_effect(guid: u64, tag_id: i32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(REMOVE_COMPOSITE_EFFECT_LEN);
    bytes.extend(player_header(PLAYER_UPDATE_SUB_REMOVE_COMPOSITE_EFFECT));
    bytes.extend(guid.to_le_bytes());
    push_i32(&mut bytes, tag_id);
    bytes
}

pub fn decode_remove_composite_effect(bytes: &[u8]) -> Result<(u64, i32), StatusError> {
    check(
        bytes,
        (
            PLAYER_UPDATE_OPCODE,
            PLAYER_UPDATE_SUB_REMOVE_COMPOSITE_EFFECT,
        ),
        REMOVE_COMPOSITE_EFFECT_LEN,
    )?;
    Ok((read_u64(bytes, 4)?, read_i32(bytes, 12)?))
}

/// `PlayerUpdatePacketUpdateCharacterState` (35/20): the flagged status bitmask. The bit values come
/// from the pinned community research, not from an original table.
pub fn encode_character_state(guid: u64, flags: i32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(CHARACTER_STATE_LEN);
    bytes.extend(player_header(PLAYER_UPDATE_SUB_UPDATE_CHARACTER_STATE));
    bytes.extend(guid.to_le_bytes());
    push_i32(&mut bytes, flags);
    bytes
}

pub fn decode_character_state(bytes: &[u8]) -> Result<(u64, i32), StatusError> {
    check(
        bytes,
        (
            PLAYER_UPDATE_OPCODE,
            PLAYER_UPDATE_SUB_UPDATE_CHARACTER_STATE,
        ),
        CHARACTER_STATE_LEN,
    )?;
    Ok((read_u64(bytes, 4)?, read_i32(bytes, 12)?))
}

/// The recovered `ClientPcData.EffectTags` member: a count followed by `(tag id, ClientEffectTag)`.
pub fn encode_profile_rows(rows: &[(i32, EffectTagPayload)]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(4 + rows.len() * (4 + CLIENT_EFFECT_TAG_LEN));
    push_i32(&mut bytes, i32::try_from(rows.len()).unwrap_or(i32::MAX));
    for (tag_id, tag) in rows {
        push_i32(&mut bytes, *tag_id);
        push_payload(&mut bytes, tag);
    }
    bytes
}

pub fn decode_profile_rows(bytes: &[u8]) -> Result<Vec<(i32, EffectTagPayload)>, StatusError> {
    let count_length = 4;
    if bytes.len() < count_length {
        return Err(StatusError::BadLength {
            expected: count_length,
            got: bytes.len(),
        });
    }
    let count = read_i32(bytes, 0)?;
    if count < 0 {
        return Err(StatusError::BadLength {
            expected: count_length + 4 + CLIENT_EFFECT_TAG_LEN,
            got: bytes.len(),
        });
    }
    let expected = count_length + (count as usize) * (4 + CLIENT_EFFECT_TAG_LEN);
    if bytes.len() != expected {
        return Err(StatusError::BadLength {
            expected,
            got: bytes.len(),
        });
    }
    let mut rows = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let offset = count_length + index * (4 + CLIENT_EFFECT_TAG_LEN);
        rows.push((read_i32(bytes, offset)?, read_payload(bytes, offset + 4)?));
    }
    Ok(rows)
}

fn push_i32(out: &mut Vec<u8>, value: i32) {
    out.extend(value.to_le_bytes());
}

fn push_payload(out: &mut Vec<u8>, tag: &EffectTagPayload) {
    for value in [
        tag.unknown_1,
        tag.unknown_2,
        tag.unknown_3,
        tag.unknown_4,
        tag.unknown_5,
        tag.unknown_6,
        tag.unknown_7,
    ] {
        push_i32(out, value);
    }
    out.push(u8::from(tag.unknown_8));
    out.extend(tag.unknown_9.to_le_bytes());
    for value in [
        tag.unknown_10,
        tag.unknown_11,
        tag.unknown_12,
        tag.composite_effect_id,
    ] {
        push_i32(out, value);
    }
    out.extend(tag.unknown_14.to_le_bytes());
    push_i32(out, tag.unknown_15);
    push_i32(out, tag.unknown_16);
    out.push(u8::from(tag.unknown_17));
    out.push(u8::from(tag.unknown_18));
    out.push(u8::from(tag.unknown_19));
    push_i32(out, tag.unknown_20);
    out.push(u8::from(tag.unknown_21));
    push_i32(out, tag.unknown_22);
    push_i32(out, tag.unknown_23);
}

fn read_payload(bytes: &[u8], start: usize) -> Result<EffectTagPayload, StatusError> {
    let mut offset = start;
    let mut ints = [0_i32; 7];
    for value in &mut ints {
        *value = read_i32(bytes, offset)?;
        offset += 4;
    }
    let unknown_8 = read_bool(bytes, offset)?;
    offset += 1;
    let unknown_9 = read_i64(bytes, offset)?;
    offset += 8;
    let unknown_10 = read_i32(bytes, offset)?;
    offset += 4;
    let unknown_11 = read_i32(bytes, offset)?;
    offset += 4;
    let unknown_12 = read_i32(bytes, offset)?;
    offset += 4;
    let composite_effect_id = read_i32(bytes, offset)?;
    offset += 4;
    let unknown_14 = read_i64(bytes, offset)?;
    offset += 8;
    let unknown_15 = read_i32(bytes, offset)?;
    offset += 4;
    let unknown_16 = read_i32(bytes, offset)?;
    offset += 4;
    let unknown_17 = read_bool(bytes, offset)?;
    offset += 1;
    let unknown_18 = read_bool(bytes, offset)?;
    offset += 1;
    let unknown_19 = read_bool(bytes, offset)?;
    offset += 1;
    let unknown_20 = read_i32(bytes, offset)?;
    offset += 4;
    let unknown_21 = read_bool(bytes, offset)?;
    offset += 1;
    let unknown_22 = read_i32(bytes, offset)?;
    offset += 4;
    let unknown_23 = read_i32(bytes, offset)?;

    Ok(EffectTagPayload {
        unknown_1: ints[0],
        unknown_2: ints[1],
        unknown_3: ints[2],
        unknown_4: ints[3],
        unknown_5: ints[4],
        unknown_6: ints[5],
        unknown_7: ints[6],
        unknown_8,
        unknown_9,
        unknown_10,
        unknown_11,
        unknown_12,
        composite_effect_id,
        unknown_14,
        unknown_15,
        unknown_16,
        unknown_17,
        unknown_18,
        unknown_19,
        unknown_20,
        unknown_21,
        unknown_22,
        unknown_23,
    })
}

fn check(bytes: &[u8], expected: (u16, u16), length: usize) -> Result<(), StatusError> {
    if bytes.len() != length {
        return Err(StatusError::BadLength {
            expected: length,
            got: bytes.len(),
        });
    }
    let got = (
        u16::from_le_bytes(bytes[..2].try_into().unwrap()),
        u16::from_le_bytes(bytes[2..4].try_into().unwrap()),
    );
    if got != expected {
        return Err(StatusError::BadOpcode { expected, got });
    }
    Ok(())
}

fn read_bool(bytes: &[u8], offset: usize) -> Result<bool, StatusError> {
    bytes
        .get(offset)
        .map(|value| *value != 0)
        .ok_or(StatusError::BadLength {
            expected: offset + 1,
            got: bytes.len(),
        })
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, StatusError> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or(StatusError::BadLength {
            expected: offset + 4,
            got: bytes.len(),
        })?;
    Ok(i32::from_le_bytes(slice.try_into().unwrap()))
}

fn read_i64(bytes: &[u8], offset: usize) -> Result<i64, StatusError> {
    let slice = bytes
        .get(offset..offset + 8)
        .ok_or(StatusError::BadLength {
            expected: offset + 8,
            got: bytes.len(),
        })?;
    Ok(i64::from_le_bytes(slice.try_into().unwrap()))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, StatusError> {
    let slice = bytes
        .get(offset..offset + 8)
        .ok_or(StatusError::BadLength {
            expected: offset + 8,
            got: bytes.len(),
        })?;
    Ok(u64::from_le_bytes(slice.try_into().unwrap()))
}
