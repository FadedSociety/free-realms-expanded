//! Work item 048: original action-bar/hotbar binding packets.
//!
//! Recovered original facts, all from sources listed in
//! `notes/Research/Pieces/048 Ability bar.md`:
//!
//! * `ASSIGN_MAIN`/`ASSIGN_SUB` and the 12-byte request. The native base
//!   constructor `00a10400` writes `param_1[1] = 0x2a` (42) and copies the sub
//!   opcode; `00c14720` constructs `InventoryPacketItemActionBarAssign` with sub 6
//!   plus a zero-based slot and an owned item instance id; the header writer
//!   `00a1ada0` emits both 16-bit headers. The pinned community deserializer reads
//!   exactly `Slot` then `Guid` and rejects any remaining byte.
//! * `ASSIGN_BY_RECORD_SUB` exists. `00a122e0` constructs
//!   `InventoryPacketItemActionBarAssignByItemRecord` with sub 9, a slot and three
//!   copied item-record fields, but the serialized record field order is not
//!   recovered, so this module refuses it instead of guessing.
//! * `SLOT_UPDATE_MAIN`/`SLOT_UPDATE_SUB` and the 63-byte update packet. `008dadd0`
//!   consumes the bar id and slot index; `008dae30` consumes the 51-byte slot as
//!   three 1-byte bools, eleven 4-byte values (one float-compared for NaN) and a
//!   trailing 4-byte cast age whose `0x7fffffff` value selects the never-cast
//!   sentinel. `SLOT_FIELDS` is that exact order.
//! * `ITEM_BAR_ID` is 2. The original client UI assigns consumables with
//!   `AssignToItemActionBar(button - 5, itemId)` and activates them with
//!   `ClickedAbility(button - 5, 2)`; the pinned community handler also hardcodes
//!   bar 2 for the item action bar.
//! * `ITEM_BAR_SLOTS` is 4. The preserved captured self packet carries exactly the
//!   slots `0..3` for bar 2, and the community default creates the same four.
//! * The recovered self-targeting template is selector 1 with target selector 2 or
//!   4. `00a35600` forces a self target for exactly that pair; the captured slot
//!   uses 1/2.
//! * The captured reference slots in `CAPTURED_*` are reconstructions of the
//!   preserved captured self packet's field values through this recovered layout.
//!   Their `serverEnabled` flags are false and slot 1 carries the never-cast
//!   sentinel, so an assignment must choose an enabled state instead of copying the
//!   capture.
//!
//! Reconstructed policy (not a recovered original rule): the item bar is bounded to
//! four slots, a freshly bound slot is rendered `serverEnabled = true` with the
//! recovered self-targeting template, persisted bindings are pruned against the live
//! inventory on every change and on reconnect, and an empty stack is not an
//! available action. The module never grants, consumes or mutates an item; it only
//! reads the caller's inventory projection and returns reply bytes.

use serde::{Deserialize, Serialize};
use std::io;

/// Recovered bar category used by the combat/ability toolbar (`AbilityPacketSetDefinition`).
pub const ABILITY_BAR_ID: i32 = 1;
/// Recovered bar category of the item action bar; the 42/6 request has no bar field.
pub const ITEM_BAR_ID: i32 = 2;
/// Slots present in the preserved captured self packet for `ITEM_BAR_ID`.
pub const ITEM_BAR_SLOTS: i32 = 4;
/// Inventory packet family of the native action-bar request.
pub const ASSIGN_MAIN: u16 = 42;
/// Sub opcode of `InventoryPacketItemActionBarAssign`.
pub const ASSIGN_SUB: u16 = 6;
/// Sub opcode of `InventoryPacketItemActionBarAssignByItemRecord`; wire unrecovered.
pub const ASSIGN_BY_RECORD_SUB: u16 = 9;
/// Client update family of the server slot refresh.
pub const SLOT_UPDATE_MAIN: u16 = 38;
/// Sub opcode of `ClientUpdatePacketUpdateActionBarSlot`.
pub const SLOT_UPDATE_SUB: u16 = 25;
/// Exact length of the recovered assign request: two 16-bit headers and two i32.
pub const ASSIGN_REQUEST_BYTES: usize = 12;
/// Exact serialized length of one action-bar slot.
pub const SLOT_BYTES: usize = 51;
/// Exact length of a full slot refresh: 4 header + 8 bar/slot + one slot.
pub const SLOT_UPDATE_BYTES: usize = 63;
/// Recovered cast-age value that selects the native never-cast sentinel branch.
pub const NEVER_CAST_AGE: i32 = i32::MAX;
/// Version of the persisted action-bar record.
pub const STATE_VERSION: u32 = 1;

/// Byte offsets inside the 51-byte serialized slot.
pub mod slot_field {
    /// 1-byte empty flag.
    pub const IS_EMPTY: usize = 0;
    /// Icon id.
    pub const ICON_ID: usize = 1;
    /// Icon tint id.
    pub const ICON_TINT_ID: usize = 5;
    /// Localized name id.
    pub const NAME_ID: usize = 9;
    /// First targeting selector; the captured self packet uses 1.
    pub const SELECTOR_A: usize = 13;
    /// Second targeting selector; the captured self packet uses 2 for self.
    pub const TARGET_SELECTOR: usize = 17;
    /// Targeting parameter, serialized as a float and NaN-checked by the native reader.
    pub const TARGETING_PARAMETER: usize = 21;
    /// Mana cost.
    pub const MANA_COST: usize = 25;
    /// Server enable flag, independent of the item's own activation field.
    pub const ENABLED: usize = 29;
    /// Field the community still calls `Unknown10` (elapsed/refresh variant).
    pub const UNKNOWN_10: usize = 30;
    /// Total refresh time.
    pub const TOTAL_REFRESH_TIME: usize = 34;
    /// Field the community still calls `Unknown12` (elapsed/refresh variant).
    pub const UNKNOWN_12: usize = 38;
    /// Displayed quantity (the item stack count).
    pub const QUANTITY: usize = 42;
    /// Force-dismount flag copied into the native cast state.
    pub const FORCE_DISMOUNT: usize = 46;
    /// Cast age; `NEVER_CAST_AGE` selects the native never-cast sentinel.
    pub const CAST_AGE: usize = 47;
}

/// Wire width class of one serialized slot field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotKind {
    /// One byte; any nonzero value decodes as true.
    Bool,
    /// Four-byte little-endian signed integer.
    I32,
    /// Four-byte little-endian float.
    F32,
}

/// One field of the recovered 51-byte slot layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotField {
    /// Recovered or community field name.
    pub name: &'static str,
    /// Byte offset inside the slot.
    pub offset: usize,
    /// Serialized width in bytes.
    pub width: usize,
    /// Serialized kind.
    pub kind: SlotKind,
}

/// The recovered serialized order of the native slot reader `008dae30`.
pub const SLOT_FIELDS: [SlotField; 15] = [
    SlotField {
        name: "is_empty",
        offset: slot_field::IS_EMPTY,
        width: 1,
        kind: SlotKind::Bool,
    },
    SlotField {
        name: "icon_id",
        offset: slot_field::ICON_ID,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "icon_tint_id",
        offset: slot_field::ICON_TINT_ID,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "name_id",
        offset: slot_field::NAME_ID,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "selector_a",
        offset: slot_field::SELECTOR_A,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "target_selector",
        offset: slot_field::TARGET_SELECTOR,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "targeting_parameter",
        offset: slot_field::TARGETING_PARAMETER,
        width: 4,
        kind: SlotKind::F32,
    },
    SlotField {
        name: "mana_cost",
        offset: slot_field::MANA_COST,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "enabled",
        offset: slot_field::ENABLED,
        width: 1,
        kind: SlotKind::Bool,
    },
    SlotField {
        name: "unknown_10",
        offset: slot_field::UNKNOWN_10,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "total_refresh_time",
        offset: slot_field::TOTAL_REFRESH_TIME,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "unknown_12",
        offset: slot_field::UNKNOWN_12,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "quantity",
        offset: slot_field::QUANTITY,
        width: 4,
        kind: SlotKind::I32,
    },
    SlotField {
        name: "force_dismount",
        offset: slot_field::FORCE_DISMOUNT,
        width: 1,
        kind: SlotKind::Bool,
    },
    SlotField {
        name: "cast_age",
        offset: slot_field::CAST_AGE,
        width: 4,
        kind: SlotKind::I32,
    },
];

/// Serialized length of the recovered slot layout.
pub fn slot_layout_bytes() -> usize {
    SLOT_FIELDS.iter().map(|field| field.width).sum()
}

/// Captured reference empty slot (bar 2, slot 3) rendered through this layout.
/// Field values are the preserved capture's; the encoding is this module's.
pub const CAPTURED_EMPTY_SLOT: [u8; SLOT_BYTES] = [
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
];

/// Captured reference slot 1: disabled, quantity 59, cast age 1789747881.
/// Field values are the preserved capture's; the encoding is this module's.
pub const CAPTURED_SLOT_0: [u8; SLOT_BYTES] = [
    0x00, 0x1f, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfe, 0x73, 0x06, 0x00, 0x01, 0x00, 0x00,
    0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3b, 0x00, 0x00, 0x00, 0x01, 0xa9,
    0x62, 0xad, 0x6a,
];

/// Captured reference slot 2: quantity 28 and the never-cast cast-age sentinel.
/// Field values are the preserved capture's; the encoding is this module's.
pub const CAPTURED_SLOT_1: [u8; SLOT_BYTES] = [
    0x00, 0x09, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x74, 0x06, 0x00, 0x01, 0x00, 0x00,
    0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1c, 0x00, 0x00, 0x00, 0x01, 0xff,
    0xff, 0xff, 0x7f,
];

/// Captured reference slot 3: quantity 33.
/// Field values are the preserved capture's; the encoding is this module's.
pub const CAPTURED_SLOT_2: [u8; SLOT_BYTES] = [
    0x00, 0x20, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfb, 0x73, 0x06, 0x00, 0x01, 0x00, 0x00,
    0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x00, 0x00, 0x00, 0x01, 0x01,
    0x84, 0xac, 0x6a,
];

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn put_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("checked length"),
    )
}

/// One parsed native action-bar assign request (42/6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssignRequest {
    /// Zero-based item-bar slot.
    pub slot: i32,
    /// Owned item instance id; zero unassigns the slot.
    pub item_instance_id: i32,
}

/// Result of classifying one inventory-family message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignParse {
    /// The recovered 12-byte action-bar assign.
    Request(AssignRequest),
    /// The sub-9 by-item-record variant whose serialized record layout is unrecovered.
    ByRecordUnrecovered {
        /// Observed message length, kept for the next recovery attempt.
        observed_len: usize,
    },
    /// Another inventory sub opcode owned by a different piece.
    OtherSubOpcode,
}

/// Classify an inventory-family message without touching any state.
pub fn parse_assign(inner: &[u8]) -> io::Result<AssignParse> {
    if inner.len() < 4 {
        return Err(invalid("inventory request is shorter than its header"));
    }
    if u16::from_le_bytes(inner[..2].try_into().expect("checked length")) != ASSIGN_MAIN {
        return Err(invalid("not an inventory request"));
    }
    let sub = u16::from_le_bytes(inner[2..4].try_into().expect("checked length"));
    match sub {
        ASSIGN_SUB => {
            if inner.len() != ASSIGN_REQUEST_BYTES {
                return Err(invalid("action bar assign must be exactly 12 bytes"));
            }
            Ok(AssignParse::Request(AssignRequest {
                slot: read_i32(inner, 4),
                item_instance_id: read_i32(inner, 8),
            }))
        }
        ASSIGN_BY_RECORD_SUB => Ok(AssignParse::ByRecordUnrecovered {
            observed_len: inner.len(),
        }),
        _ => Ok(AssignParse::OtherSubOpcode),
    }
}

/// One serialized action-bar slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActionBarSlot {
    /// Empty flag; an empty slot rejects activation natively.
    pub is_empty: bool,
    /// Icon id.
    pub icon_id: i32,
    /// Icon tint id.
    pub icon_tint_id: i32,
    /// Localized name id.
    pub name_id: i32,
    /// First targeting selector.
    pub selector_a: i32,
    /// Second targeting selector; 1 + 2 (or 4) forces self natively.
    pub target_selector: i32,
    /// Targeting parameter, NaN-checked by the native reader.
    pub targeting_parameter: f32,
    /// Mana cost.
    pub mana_cost: i32,
    /// Server enable flag.
    pub enabled: bool,
    /// Community `Unknown10`.
    pub unknown_10: i32,
    /// Total refresh time.
    pub total_refresh_time: i32,
    /// Community `Unknown12`.
    pub unknown_12: i32,
    /// Displayed quantity.
    pub quantity: i32,
    /// Force-dismount flag.
    pub force_dismount: bool,
    /// Cast age; `NEVER_CAST_AGE` means never cast.
    pub cast_age: i32,
}

impl ActionBarSlot {
    /// The captured reference empty slot.
    pub fn empty() -> Self {
        Self {
            is_empty: true,
            icon_id: 0,
            icon_tint_id: 0,
            name_id: 0,
            selector_a: 0,
            target_selector: 0,
            targeting_parameter: 0.0,
            mana_cost: 0,
            enabled: false,
            unknown_10: 0,
            total_refresh_time: 0,
            unknown_12: 0,
            quantity: 0,
            force_dismount: false,
            cast_age: 0,
        }
    }

    /// Render one owned action into a freshly bound slot.
    ///
    /// Recovered: the captured self-targeting selector pair 1/2, the zero
    /// targeting parameter, quantity from the live stack and the never-cast sentinel.
    /// Reconstructed: `serverEnabled = true` so the fresh binding is not disabled the
    /// way the captured reference potion slot is.
    pub fn for_owned_action(action: &OwnedAction) -> Self {
        Self {
            is_empty: false,
            icon_id: action.icon_id,
            icon_tint_id: action.icon_tint_id,
            name_id: action.name_id,
            selector_a: 1,
            target_selector: 2,
            targeting_parameter: 0.0,
            mana_cost: 0,
            enabled: true,
            unknown_10: 0,
            total_refresh_time: 0,
            unknown_12: 0,
            quantity: action.quantity(),
            force_dismount: true,
            cast_age: NEVER_CAST_AGE,
        }
    }

    /// Serialize the 51-byte slot exactly as the native reader consumes it.
    pub fn to_bytes(&self) -> [u8; SLOT_BYTES] {
        let mut out = [0u8; SLOT_BYTES];
        out[slot_field::IS_EMPTY] = u8::from(self.is_empty);
        out[slot_field::ICON_ID..slot_field::ICON_ID + 4]
            .copy_from_slice(&self.icon_id.to_le_bytes());
        out[slot_field::ICON_TINT_ID..slot_field::ICON_TINT_ID + 4]
            .copy_from_slice(&self.icon_tint_id.to_le_bytes());
        out[slot_field::NAME_ID..slot_field::NAME_ID + 4]
            .copy_from_slice(&self.name_id.to_le_bytes());
        out[slot_field::SELECTOR_A..slot_field::SELECTOR_A + 4]
            .copy_from_slice(&self.selector_a.to_le_bytes());
        out[slot_field::TARGET_SELECTOR..slot_field::TARGET_SELECTOR + 4]
            .copy_from_slice(&self.target_selector.to_le_bytes());
        out[slot_field::TARGETING_PARAMETER..slot_field::TARGETING_PARAMETER + 4]
            .copy_from_slice(&self.targeting_parameter.to_le_bytes());
        out[slot_field::MANA_COST..slot_field::MANA_COST + 4]
            .copy_from_slice(&self.mana_cost.to_le_bytes());
        out[slot_field::ENABLED] = u8::from(self.enabled);
        out[slot_field::UNKNOWN_10..slot_field::UNKNOWN_10 + 4]
            .copy_from_slice(&self.unknown_10.to_le_bytes());
        out[slot_field::TOTAL_REFRESH_TIME..slot_field::TOTAL_REFRESH_TIME + 4]
            .copy_from_slice(&self.total_refresh_time.to_le_bytes());
        out[slot_field::UNKNOWN_12..slot_field::UNKNOWN_12 + 4]
            .copy_from_slice(&self.unknown_12.to_le_bytes());
        out[slot_field::QUANTITY..slot_field::QUANTITY + 4]
            .copy_from_slice(&self.quantity.to_le_bytes());
        out[slot_field::FORCE_DISMOUNT] = u8::from(self.force_dismount);
        out[slot_field::CAST_AGE..slot_field::CAST_AGE + 4]
            .copy_from_slice(&self.cast_age.to_le_bytes());
        out
    }

    /// Decode one 51-byte slot; a NaN targeting parameter is rejected like the
    /// native reader's NaN branch.
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != SLOT_BYTES {
            return Err(invalid("action bar slot must be exactly 51 bytes"));
        }
        let targeting_parameter = f32::from_le_bytes(
            bytes[slot_field::TARGETING_PARAMETER..slot_field::TARGETING_PARAMETER + 4]
                .try_into()
                .expect("checked length"),
        );
        if targeting_parameter.is_nan() {
            return Err(invalid("action bar targeting parameter is not a number"));
        }
        Ok(Self {
            is_empty: bytes[slot_field::IS_EMPTY] != 0,
            icon_id: read_i32(bytes, slot_field::ICON_ID),
            icon_tint_id: read_i32(bytes, slot_field::ICON_TINT_ID),
            name_id: read_i32(bytes, slot_field::NAME_ID),
            selector_a: read_i32(bytes, slot_field::SELECTOR_A),
            target_selector: read_i32(bytes, slot_field::TARGET_SELECTOR),
            targeting_parameter,
            mana_cost: read_i32(bytes, slot_field::MANA_COST),
            enabled: bytes[slot_field::ENABLED] != 0,
            unknown_10: read_i32(bytes, slot_field::UNKNOWN_10),
            total_refresh_time: read_i32(bytes, slot_field::TOTAL_REFRESH_TIME),
            unknown_12: read_i32(bytes, slot_field::UNKNOWN_12),
            quantity: read_i32(bytes, slot_field::QUANTITY),
            force_dismount: bytes[slot_field::FORCE_DISMOUNT] != 0,
            cast_age: read_i32(bytes, slot_field::CAST_AGE),
        })
    }
}

/// Render one 38/25 slot refresh for the item bar.
pub fn item_slot_update(slot: i32, data: &ActionBarSlot) -> io::Result<Vec<u8>> {
    if !(0..ITEM_BAR_SLOTS).contains(&slot) {
        return Err(invalid(
            "item action bar slot is outside the recovered range",
        ));
    }
    let mut out = Vec::with_capacity(SLOT_UPDATE_BYTES);
    out.extend_from_slice(&SLOT_UPDATE_MAIN.to_le_bytes());
    out.extend_from_slice(&SLOT_UPDATE_SUB.to_le_bytes());
    put_i32(&mut out, ITEM_BAR_ID);
    put_i32(&mut out, slot);
    out.extend_from_slice(&data.to_bytes());
    Ok(out)
}

/// Render the empty-slot refresh used when a binding is removed.
pub fn empty_item_slot_update(slot: i32) -> io::Result<Vec<u8>> {
    item_slot_update(slot, &ActionBarSlot::empty())
}

/// Community tint rule: a zero instance tint falls back to the definition tint.
pub fn resolve_tint(instance_tint: i32, definition_tint: i32) -> i32 {
    if instance_tint == 0 {
        definition_tint
    } else {
        instance_tint
    }
}

/// Read-only projection of one owned item instance that can back an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedAction {
    /// Owned item instance id (the assign request's `Guid`).
    pub item_instance_id: i32,
    /// Item definition id.
    pub definition_id: i32,
    /// Live stack count.
    pub count: u32,
    /// Definition icon id.
    pub icon_id: i32,
    /// Already resolved icon tint id (`resolve_tint`).
    pub icon_tint_id: i32,
    /// Localized name id.
    pub name_id: i32,
}

impl OwnedAction {
    /// Reconstructed availability rule: an empty stack cannot back a slot.
    pub fn is_available(&self) -> bool {
        self.count > 0
    }

    /// Slot quantity, clamped to the i32 wire field.
    pub fn quantity(&self) -> i32 {
        self.count.min(i32::MAX as u32) as i32
    }
}

/// Read-only inventory view. The production implementation borrows the live
/// inventory; this module never mutates it.
pub trait ActionBarItems {
    /// Return the owned instance when it exists, or `None` when it is not owned.
    fn owned_action(&self, item_instance_id: i32) -> Option<OwnedAction>;
}

/// One persisted binding: which owned instance fills a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionBarBinding {
    /// Zero-based item-bar slot.
    pub slot: i32,
    /// Owned item instance id.
    pub item_instance_id: i32,
    /// Item definition id at bind time; drift drops the binding.
    pub definition_id: i32,
}

/// Persisted item-bar bindings. Only owned, available instances are kept.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionBarState {
    version: u32,
    #[serde(default)]
    bindings: Vec<ActionBarBinding>,
}

impl Default for ActionBarState {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionBarState {
    /// An empty, current-version record.
    pub fn new() -> Self {
        Self {
            version: STATE_VERSION,
            bindings: Vec::new(),
        }
    }

    /// Record version, for snapshot validation.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Persisted bindings in slot order.
    pub fn bindings(&self) -> &[ActionBarBinding] {
        &self.bindings
    }

    /// The binding of one slot, if any.
    pub fn bound(&self, slot: i32) -> Option<ActionBarBinding> {
        self.bindings.iter().copied().find(|b| b.slot == slot)
    }

    /// Reject a record that no assignment could have produced.
    pub fn validate(&self) -> io::Result<()> {
        if self.version != STATE_VERSION {
            return Err(invalid("unsupported action bar state version"));
        }
        if self.bindings.len() > ITEM_BAR_SLOTS as usize {
            return Err(invalid("action bar state exceeds the item bar slots"));
        }
        let mut previous: Option<i32> = None;
        for binding in &self.bindings {
            if !(0..ITEM_BAR_SLOTS).contains(&binding.slot) {
                return Err(invalid(
                    "action bar binding slot is outside the recovered range",
                ));
            }
            if binding.item_instance_id <= 0 {
                return Err(invalid("action bar binding needs a positive item instance"));
            }
            if binding.definition_id <= 0 {
                return Err(invalid("action bar binding needs a positive definition"));
            }
            if previous.is_some_and(|slot| slot >= binding.slot) {
                return Err(invalid("action bar bindings must use unique ordered slots"));
            }
            previous = Some(binding.slot);
        }
        Ok(())
    }

    /// Drop every binding whose instance is no longer owned, whose definition
    /// changed, or whose stack is empty. Returns how many bindings were dropped.
    pub fn reconcile(&mut self, items: &impl ActionBarItems) -> usize {
        let before = self.bindings.len();
        self.bindings.retain(|binding| {
            items
                .owned_action(binding.item_instance_id)
                .is_some_and(|action| {
                    action.is_available() && action.definition_id == binding.definition_id
                })
        });
        before - self.bindings.len()
    }

    /// Apply one classified request. Stale bindings are pruned first, so the
    /// persisted set always only holds owned, available actions.
    pub fn apply_parse(
        &mut self,
        parse: AssignParse,
        items: &impl ActionBarItems,
    ) -> io::Result<AssignOutcome> {
        match parse {
            AssignParse::Request(request) => self.assign(request, items),
            AssignParse::ByRecordUnrecovered { .. } => Ok(AssignOutcome::rejected(
                AssignRejection::ByRecordUnrecovered,
            )),
            AssignParse::OtherSubOpcode => Ok(AssignOutcome::ignored()),
        }
    }

    /// Apply one recovered 42/6 assignment or unassignment.
    pub fn assign(
        &mut self,
        request: AssignRequest,
        items: &impl ActionBarItems,
    ) -> io::Result<AssignOutcome> {
        self.validate()?;
        self.reconcile(items);
        if !(0..ITEM_BAR_SLOTS).contains(&request.slot) {
            return Ok(AssignOutcome::rejected(AssignRejection::SlotOutOfRange));
        }
        if request.item_instance_id == 0 {
            let changed = self.remove(request.slot);
            return Ok(AssignOutcome {
                changed,
                reply: Some(empty_item_slot_update(request.slot)?),
                rejection: None,
            });
        }
        let Some(action) = items.owned_action(request.item_instance_id) else {
            return Ok(AssignOutcome::rejected(AssignRejection::UnknownItem));
        };
        if !action.is_available() {
            return Ok(AssignOutcome::rejected(AssignRejection::Unavailable));
        }
        let binding = ActionBarBinding {
            slot: request.slot,
            item_instance_id: action.item_instance_id,
            definition_id: action.definition_id,
        };
        let changed = self.bound(request.slot) != Some(binding);
        self.set(binding);
        let reply = item_slot_update(request.slot, &ActionBarSlot::for_owned_action(&action))?;
        self.validate()?;
        Ok(AssignOutcome {
            changed,
            reply: Some(reply),
            rejection: None,
        })
    }

    /// Refresh packets for every live binding, in slot order. Stale bindings are
    /// skipped rather than re-rendered.
    pub fn slot_updates(&self, items: &impl ActionBarItems) -> io::Result<Vec<Vec<u8>>> {
        self.validate()?;
        let mut out = Vec::new();
        for binding in &self.bindings {
            let Some(action) = items.owned_action(binding.item_instance_id) else {
                continue;
            };
            if !action.is_available() || action.definition_id != binding.definition_id {
                continue;
            }
            out.push(item_slot_update(
                binding.slot,
                &ActionBarSlot::for_owned_action(&action),
            )?);
        }
        Ok(out)
    }

    fn set(&mut self, binding: ActionBarBinding) {
        self.bindings
            .retain(|existing| existing.slot != binding.slot);
        self.bindings.push(binding);
        self.bindings.sort_by_key(|existing| existing.slot);
    }

    fn remove(&mut self, slot: i32) -> bool {
        let before = self.bindings.len();
        self.bindings.retain(|existing| existing.slot != slot);
        before != self.bindings.len()
    }
}

/// Why a recovered request did not change the bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignRejection {
    /// The item instance is not owned by this character.
    UnknownItem,
    /// The owned instance has an empty stack.
    Unavailable,
    /// The slot is outside the recovered item-bar range.
    SlotOutOfRange,
    /// The sub-9 by-item-record variant whose wire layout is unrecovered.
    ByRecordUnrecovered,
}

/// Result of one request: whether state changed, the reply to send, and any refusal.
#[derive(Clone, Debug, PartialEq)]
pub struct AssignOutcome {
    /// True when the persisted binding set changed.
    pub changed: bool,
    /// Exact 38/25 reply bytes, when the original flow answers.
    pub reply: Option<Vec<u8>>,
    /// Refusal reason, when the request was not applied.
    pub rejection: Option<AssignRejection>,
}

impl AssignOutcome {
    /// A refusal that answers nothing, matching the community handler's unknown-item
    /// behavior.
    pub fn rejected(rejection: AssignRejection) -> Self {
        Self {
            changed: false,
            reply: None,
            rejection: Some(rejection),
        }
    }

    /// A message this module does not own.
    pub fn ignored() -> Self {
        Self {
            changed: false,
            reply: None,
            rejection: None,
        }
    }
}
