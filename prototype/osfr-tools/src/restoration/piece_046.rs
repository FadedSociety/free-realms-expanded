//! Work item 046: consumable use requests, quantity/effect/icon presentation and a reconstructed
//! once-only consumption / cooldown / expiry policy.
//!
//! # Recovered original facts
//!
//! * Packet ids. The decompiled original server enums name the wire families used here:
//!   `InventoryPacketItemActionBarAssign = 6` and `InventoryPacketCheckImmediateActivationItem = 16`,
//!   `InventoryPacketUseImmediateActivationItem = 15` on base `0x002a` = 42;
//!   `AbilityPacketClientRequestStartAbility = 10` and `AbilityPacketFailed = 1` on base `0x0024` = 36;
//!   `ClientUpdatePacketUpdateActionBarSlot = 25` on base `0x0026` = 38. The client dock calls
//!   `Ui.ClickedAbility(slot, 2)` / `Ui.AssignToItemActionBar(slot, ownedItemGUID)` for that item bar.
//! * Request schemas. The native constructors/writers pinned in
//!   `lab/reports/original-inventory-use-v1/proof.json` establish the exact framing: 42/6 is
//!   `u16 main, u16 sub, i32 slot, i32 owned item instance id` (12 bytes); 36/10 is
//!   `u16, u16, i32 bar, i32 slot, i32 target mode` plus a `u64` GUID for mode 0 (24 bytes) or a
//!   `Vector4` for mode 1 (32 bytes); 42/16 and 42/15 are `u16, u16, i32 owned item instance id`
//!   (8 bytes) and are a distinct immediate-activation confirmation flow, not ordinary use.
//! * Slot reply. 38/25 is `4-byte header + 8-byte ActionBarData(bar, slot) + 51-byte slot`. The original
//!   server serializer `ActionBarSlot.Serialize` in `lab/decompiled/server/Gateway/Gateway.Login/ClientPcData.cs`
//!   and the native client reader agree on the 15 fields and their widths; the captured 63-byte vectors
//!   in `lab/reports/original-item-toolbar-v1/` parse with these exact offsets. The native reader treats
//!   the selector at slot offset 21 as an `f32`, converts the last `i32` from an age into a clock and
//!   uses `i32::MAX` as the never-cast sentinel; `total_refresh_time` 0 means no recast delay.
//! * Icons and effects. The item rows recovered from the pinned AGPL-3.0
//!   `FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json` carry the `Icon.Id`, `NameId`,
//!   `DescriptionId`, class 9, category 9 (`Potions`) / 10 (`Food & Drink`) and the
//!   `ActivatableAbilityId` that the server resolves from the assigned owned item. Category name ids
//!   961/962 resolve through `lab/reports/locale-id-to-text.json`.
//! * Failure display. 36/1 carries the localized refusal string id 3079,
//!   "You can't use that ability right now."
//!
//! # Reconstructed policy (not original)
//!
//! `ConsumableLedger` is an offline decision model over state that is already owned. Every duration is
//! caller-supplied: no historical cooldown, healing amount, duration, size multiplier or stacking rule is
//! baked in, because none of those values is recovered. The reconstructed rules are: only the item bar is
//! accepted; a slot must hold a recovered catalogue record; a depleted record, a set bit of the recovered
//! client input mask, an unexpired recast window or a non-monotonic request tick refuses the use; an
//! accepted use decrements exactly one item, starts one recast window and replaces that instance's effect
//! window. Nothing here grants items, currency or experience, and no packet is sent.
//!
//! # Explicit gaps
//!
//! The numeric effects (healing percent/second, instant heal, Stew health/size tuning) are unresolved:
//! the original description says five seconds while the cached 2013/2021 wiki revisions say three
//! seconds plus a 2% instant heal. The 38/25 target metadata semantics, the icon id to texture mapping,
//! the consumable-to-effect-tag binding and any original server-side consumption implementation remain
//! unrecovered. See `notes/Research/Pieces/046 Consumables and buffs.md`.

/// Base opcode of the inventory packet family (recovered `0x002a`).
pub const INVENTORY_MAIN: u16 = 42;
/// Base opcode of the ability packet family (recovered `0x0024`).
pub const ABILITY_MAIN: u16 = 36;
/// Base opcode of the client-update packet family (recovered `0x0026`).
pub const CLIENT_UPDATE_MAIN: u16 = 38;
/// Recovered `InventoryPacketItemActionBarAssign`.
pub const ITEM_ACTION_BAR_ASSIGN_SUB: u16 = 6;
/// Recovered `InventoryPacketCheckImmediateActivationItem`.
pub const CHECK_IMMEDIATE_ACTIVATION_SUB: u16 = 16;
/// Recovered `InventoryPacketUseImmediateActivationItem`.
pub const USE_IMMEDIATE_ACTIVATION_SUB: u16 = 15;
/// Recovered `AbilityPacketClientRequestStartAbility`.
pub const CLIENT_REQUEST_START_ABILITY_SUB: u16 = 10;
/// Recovered `AbilityPacketFailed`.
pub const ABILITY_FAILED_SUB: u16 = 1;
/// Recovered `ClientUpdatePacketUpdateActionBarSlot`.
pub const UPDATE_ACTION_BAR_SLOT_SUB: u16 = 25;
/// Recovered `ClientUpdatePacketAddEffectTag`; the buff/effect-tag binding is unrecovered.
pub const ADD_EFFECT_TAG_SUB: u16 = 16;
/// Recovered `ClientUpdatePacketRemoveEffectTag`; the buff/effect-tag binding is unrecovered.
pub const REMOVE_EFFECT_TAG_SUB: u16 = 17;
/// Recovered `ClientUpdatePacketImmediateActivationCheck` (native `0089ea30`, handler `00a1a7d0`
/// raises `InventoryHandler:OnIsItemUsable`).
pub const IMMEDIATE_ACTIVATION_CHECK_SUB: u16 = 30;

/// Recovered item action bar id used by the dock's consumable calls.
pub const ITEM_ACTION_BAR_ID: i32 = 2;
/// Recovered `ActionBarSlot` wire width (15 fields).
pub const ACTION_BAR_SLOT_WIRE_LEN: usize = 51;
/// Recovered 38/25 length: 4-byte header + 8-byte bar/slot + the slot payload.
pub const UPDATE_ACTION_BAR_SLOT_LEN: usize = 63;
/// Recovered native client cast-block status mask; a clear mask is required for input.
pub const CAST_BLOCK_STATUS_MASK: u32 = 0x0015_42C6;
/// Recovered never-cast sentinel of the slot's last cast-age field.
pub const NEVER_CAST_SENTINEL: i32 = i32::MAX;
/// Recovered localized refusal string id carried by 36/1.
pub const USE_FAILURE_STRING_ID: i32 = 3079;

/// Recovered consumable item class (`ItemClasses` 9).
pub const CONSUMABLE_CLASS: i32 = 9;
/// Recovered `Potions` item category id.
pub const POTION_CATEGORY: i32 = 9;
/// Recovered `Food & Drink` item category id.
pub const FOOD_CATEGORY: i32 = 10;
/// Recovered `Potions` category name id.
pub const POTION_CATEGORY_NAME_ID: i32 = 961;
/// Recovered `Food & Drink` category name id.
pub const FOOD_CATEGORY_NAME_ID: i32 = 962;

/// One recovered consumable item record. The strings are the recovered locale text for the ids.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConsumableItem {
    pub id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub icon_id: i32,
    pub class_id: i32,
    pub category_id: i32,
    pub ability_id: i32,
    pub power_rating: i32,
    pub single_use: bool,
    pub cost: i32,
    pub name: &'static str,
    pub description: &'static str,
}

/// The seven recovered consumable rows in id order.
///
/// The recovered catalogue contains two distinct `Small Health Potion` records (2173 with ability 267
/// and 11002 with ability 4696); the displayed name alone cannot select the ability, so both rows are
/// kept separate rather than collapsed.
pub const CONSUMABLE_ITEMS: &[ConsumableItem] = &[
    ConsumableItem {
        id: 35,
        name_id: 1232,
        description_id: 3172,
        icon_id: 1016,
        class_id: CONSUMABLE_CLASS,
        category_id: FOOD_CATEGORY,
        ability_id: 3264,
        power_rating: 1,
        single_use: true,
        cost: 180,
        name: "Strong Arm Stew",
        description: "Eating this food increases the health and size of the player.",
    },
    ConsumableItem {
        id: 1195,
        name_id: 6278,
        description_id: 6282,
        icon_id: 1241,
        class_id: CONSUMABLE_CLASS,
        category_id: FOOD_CATEGORY,
        ability_id: 232,
        power_rating: 1,
        single_use: true,
        cost: 10,
        name: "Wheeler on the Cob",
        description: "Now made with real corn!",
    },
    ConsumableItem {
        id: 2173,
        name_id: 422909,
        description_id: 422916,
        icon_id: 730,
        class_id: CONSUMABLE_CLASS,
        category_id: POTION_CATEGORY,
        ability_id: 267,
        power_rating: 42,
        single_use: true,
        cost: 15,
        name: "Small Health Potion",
        description: "A potion that restores some of your health over 5 seconds.",
    },
    ConsumableItem {
        id: 3021,
        name_id: 422910,
        description_id: 422916,
        icon_id: 543,
        class_id: CONSUMABLE_CLASS,
        category_id: POTION_CATEGORY,
        ability_id: 4697,
        power_rating: 3,
        single_use: true,
        cost: 25,
        name: "Medium Health Potion",
        description: "A potion that restores some of your health over 5 seconds.",
    },
    ConsumableItem {
        id: 11002,
        name_id: 422909,
        description_id: 422916,
        icon_id: 730,
        class_id: CONSUMABLE_CLASS,
        category_id: POTION_CATEGORY,
        ability_id: 4696,
        power_rating: 1,
        single_use: true,
        cost: 15,
        name: "Small Health Potion",
        description: "A potion that restores some of your health over 5 seconds.",
    },
    ConsumableItem {
        id: 65820,
        name_id: 4531,
        description_id: 4532,
        icon_id: 749,
        class_id: CONSUMABLE_CLASS,
        category_id: POTION_CATEGORY,
        ability_id: 4696,
        power_rating: 1,
        single_use: true,
        cost: 10,
        name: "Wheeler Health Potion",
        description: "Potion that heals the player on consumption.",
    },
    ConsumableItem {
        id: 68989,
        name_id: 3966,
        description_id: 3966,
        icon_id: 543,
        class_id: CONSUMABLE_CLASS,
        category_id: POTION_CATEGORY,
        ability_id: 4340,
        power_rating: 1,
        single_use: true,
        // The recovered record has no Cost field; the absent field serializes as 0.
        cost: 0,
        name: "Health Potion",
        description: "Health Potion",
    },
];

/// The seven recovered consumable rows.
pub fn consumable_definitions() -> &'static [ConsumableItem] {
    CONSUMABLE_ITEMS
}

/// Look up a recovered consumable row; unknown ids fail closed.
pub fn consumable(item_id: i32) -> Option<&'static ConsumableItem> {
    CONSUMABLE_ITEMS.iter().find(|item| item.id == item_id)
}

/// Decoded 42/6 assign request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssignRequest {
    /// Zero-based slot index inside the item bar.
    pub slot: i32,
    /// Owned item instance id, not the item definition id.
    pub instance_id: i32,
}

/// Recovered target selection of the 36/10 request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    /// Target mode 0: an explicit actor GUID follows the header.
    Guid(u64),
    /// Target mode 1: a Vector4 position follows the header.
    Position([f32; 4]),
}

/// Decoded 36/10 activation request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivateRequest {
    pub bar: i32,
    pub slot: i32,
    pub target: Target,
}

/// The two recovered immediate-activation forms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImmediateActivation {
    /// 42/16: ask the client whether the item can be activated immediately.
    Check,
    /// 42/15: confirm and use it after the Utility window's Okay.
    Use,
}

/// Encode the recovered 12-byte 42/6 request.
pub fn encode_item_action_bar_assign(slot: i32, instance_id: i32) -> [u8; 12] {
    let mut bytes = [0u8; 12];
    bytes[0..2].copy_from_slice(&INVENTORY_MAIN.to_le_bytes());
    bytes[2..4].copy_from_slice(&ITEM_ACTION_BAR_ASSIGN_SUB.to_le_bytes());
    bytes[4..8].copy_from_slice(&slot.to_le_bytes());
    bytes[8..12].copy_from_slice(&instance_id.to_le_bytes());
    bytes
}

/// Decode the recovered 12-byte 42/6 request; any other length is refused.
pub fn decode_item_action_bar_assign(bytes: &[u8]) -> Option<AssignRequest> {
    if bytes.len() != 12 {
        return None;
    }
    if u16::from_le_bytes([bytes[0], bytes[1]]) != INVENTORY_MAIN
        || u16::from_le_bytes([bytes[2], bytes[3]]) != ITEM_ACTION_BAR_ASSIGN_SUB
    {
        return None;
    }
    Some(AssignRequest {
        slot: i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        instance_id: i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
    })
}

/// Encode the recovered 36/10 request: 24 bytes for a GUID target, 32 for a position target.
pub fn encode_client_request_start_ability(bar: i32, slot: i32, target: Target) -> Vec<u8> {
    let target_mode: i32 = match target {
        Target::Guid(_) => 0,
        Target::Position(_) => 1,
    };
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(&ABILITY_MAIN.to_le_bytes());
    bytes.extend_from_slice(&CLIENT_REQUEST_START_ABILITY_SUB.to_le_bytes());
    bytes.extend_from_slice(&bar.to_le_bytes());
    bytes.extend_from_slice(&slot.to_le_bytes());
    bytes.extend_from_slice(&target_mode.to_le_bytes());
    match target {
        Target::Guid(guid) => bytes.extend_from_slice(&guid.to_le_bytes()),
        Target::Position(position) => {
            for axis in position {
                bytes.extend_from_slice(&axis.to_le_bytes());
            }
        }
    }
    bytes
}

/// Decode a 36/10 request; only the two recovered lengths are accepted.
pub fn decode_client_request_start_ability(bytes: &[u8]) -> Option<ActivateRequest> {
    if bytes.len() != 24 && bytes.len() != 32 {
        return None;
    }
    if u16::from_le_bytes([bytes[0], bytes[1]]) != ABILITY_MAIN
        || u16::from_le_bytes([bytes[2], bytes[3]]) != CLIENT_REQUEST_START_ABILITY_SUB
    {
        return None;
    }
    let bar = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let slot = i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let target_mode = i32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    let target = match (target_mode, bytes.len()) {
        (0, 24) => Target::Guid(u64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ])),
        (1, 32) => {
            let axis = |offset: usize| {
                f32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ])
            };
            Target::Position([axis(16), axis(20), axis(24), axis(28)])
        }
        _ => return None,
    };
    Some(ActivateRequest { bar, slot, target })
}

/// Encode the recovered 8-byte 42/16 or 42/15 immediate-activation request.
pub fn encode_immediate_activation(kind: ImmediateActivation, instance_id: i32) -> [u8; 8] {
    let sub = match kind {
        ImmediateActivation::Check => CHECK_IMMEDIATE_ACTIVATION_SUB,
        ImmediateActivation::Use => USE_IMMEDIATE_ACTIVATION_SUB,
    };
    let mut bytes = [0u8; 8];
    bytes[0..2].copy_from_slice(&INVENTORY_MAIN.to_le_bytes());
    bytes[2..4].copy_from_slice(&sub.to_le_bytes());
    bytes[4..8].copy_from_slice(&instance_id.to_le_bytes());
    bytes
}

/// Decode an 8-byte immediate-activation request; other subs fail closed.
pub fn decode_immediate_activation(bytes: &[u8]) -> Option<(ImmediateActivation, i32)> {
    if bytes.len() != 8 || u16::from_le_bytes([bytes[0], bytes[1]]) != INVENTORY_MAIN {
        return None;
    }
    let kind = match u16::from_le_bytes([bytes[2], bytes[3]]) {
        CHECK_IMMEDIATE_ACTIVATION_SUB => ImmediateActivation::Check,
        USE_IMMEDIATE_ACTIVATION_SUB => ImmediateActivation::Use,
        _ => return None,
    };
    Some((
        kind,
        i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
    ))
}

/// Encode the recovered 25-byte 38/30 `ClientUpdatePacketImmediateActivationCheck`: the instance
/// id, a marketplace category and bundle (read only when `success` is clear), the dialog's title
/// and body string ids, then the success flag. Success makes Okay send 42/15 for this instance;
/// failure routes Okay to the bundle instead.
pub fn encode_immediate_activation_check(
    instance_id: i32,
    category_id: i32,
    bundle_id: i32,
    title_string_id: i32,
    body_string_id: i32,
    success: bool,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(25);
    bytes.extend_from_slice(&CLIENT_UPDATE_MAIN.to_le_bytes());
    bytes.extend_from_slice(&IMMEDIATE_ACTIVATION_CHECK_SUB.to_le_bytes());
    for value in [
        instance_id,
        category_id,
        bundle_id,
        title_string_id,
        body_string_id,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.push(u8::from(success));
    bytes
}

/// Decode the recovered 8-byte 36/1 failure and return its localized string id.
pub fn decode_ability_failed(bytes: &[u8]) -> Option<i32> {
    if bytes.len() != 8
        || u16::from_le_bytes([bytes[0], bytes[1]]) != ABILITY_MAIN
        || u16::from_le_bytes([bytes[2], bytes[3]]) != ABILITY_FAILED_SUB
    {
        return None;
    }
    Some(i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
}

/// Decoded 38/25 item-bar slot reply. Field meanings come from the native client reader and the
/// original server serializer; the `unknown*` names are kept where no semantics are recovered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActionBarSlotUpdate {
    pub bar: i32,
    pub slot: i32,
    pub is_empty: bool,
    pub icon: i32,
    pub icon_tint: i32,
    pub name: i32,
    /// Recovered selector A (`Unknown5`); the self-target template uses 1.
    pub selector_a: i32,
    /// Recovered target selector (`Unknown6`); the self-target template uses 2.
    pub target_selector: i32,
    /// Recovered numeric targeting parameter; the native reader treats these bits as `f32`.
    pub numeric_target_parameter: f32,
    pub mana_cost: i32,
    /// Recovered server-side enable bit, independent of any item-definition field.
    pub server_enabled: bool,
    pub unknown10: i32,
    /// Recovered recast window length in milliseconds; 0 means no recast delay.
    pub total_refresh_time: i32,
    pub unknown12: i32,
    pub quantity: i32,
    /// Recovered dismount flag used by the borrowed toolbar template.
    pub force_dismount: bool,
    /// Recovered cast age in milliseconds; `NEVER_CAST_SENTINEL` means never cast.
    pub cast_age_ms: i32,
}

impl ActionBarSlotUpdate {
    /// Recovered client recast gate: remaining delay is `total_refresh_time - age`, floored at zero.
    /// `extra_age_ms` is the additional wall-clock time since the snapshot was parsed.
    pub fn recast_remaining_ms(&self, extra_age_ms: i64) -> i64 {
        if self.total_refresh_time <= 0 || self.cast_age_ms == NEVER_CAST_SENTINEL {
            return 0;
        }
        let age = i64::from(self.cast_age_ms) + extra_age_ms.max(0);
        (i64::from(self.total_refresh_time) - age).max(0)
    }

    /// Recovered client input gate: non-empty, server-enabled, no cast-blocking status bit and
    /// enough mana. The recovered recast gate is separate; call [`Self::recast_remaining_ms`].
    pub fn usable_now(&self, status_mask: u32, mana: i32) -> bool {
        !self.is_empty
            && self.server_enabled
            && status_mask & CAST_BLOCK_STATUS_MASK == 0
            && mana >= self.mana_cost
    }
}

/// Parse the recovered 63-byte 38/25 reply; wrong header or length fails closed.
pub fn decode_slot_update(bytes: &[u8]) -> Option<ActionBarSlotUpdate> {
    if bytes.len() != UPDATE_ACTION_BAR_SLOT_LEN
        || u16::from_le_bytes([bytes[0], bytes[1]]) != CLIENT_UPDATE_MAIN
        || u16::from_le_bytes([bytes[2], bytes[3]]) != UPDATE_ACTION_BAR_SLOT_SUB
    {
        return None;
    }
    let bar = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let slot_index = i32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let payload = &bytes[12..];
    let read_i32 = |offset: usize| {
        i32::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
        ])
    };
    Some(ActionBarSlotUpdate {
        bar,
        slot: slot_index,
        is_empty: payload[0] != 0,
        icon: read_i32(1),
        icon_tint: read_i32(5),
        name: read_i32(9),
        selector_a: read_i32(13),
        target_selector: read_i32(17),
        numeric_target_parameter: f32::from_le_bytes([
            payload[21],
            payload[22],
            payload[23],
            payload[24],
        ]),
        mana_cost: read_i32(25),
        server_enabled: payload[29] != 0,
        unknown10: read_i32(30),
        total_refresh_time: read_i32(34),
        unknown12: read_i32(38),
        quantity: read_i32(42),
        force_dismount: payload[46] != 0,
        cast_age_ms: read_i32(47),
    })
}

/// One owned consumable instance. Seeded from inventory state; never granted here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedInstance {
    pub instance_id: i32,
    pub item_id: i32,
    pub count: u32,
}

/// Recovered 42/6 semantics: a slot points at an owned item instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotAssignment {
    pub bar: i32,
    pub slot: i32,
    pub instance_id: i32,
}

/// One recast window for a bar slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotCooldown {
    pub bar: i32,
    pub slot: i32,
    pub ready_at_ms: i64,
}

/// One active consumable effect window (reconstructed model of the sourced effect duration).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveConsumableEffect {
    pub instance_id: i32,
    pub item_id: i32,
    pub ability_id: i32,
    pub started_at_ms: i64,
    pub expires_at_ms: i64,
}

impl ActiveConsumableEffect {
    /// The window is over on the exact boundary tick.
    pub fn expired(&self, now_ms: i64) -> bool {
        now_ms >= self.expires_at_ms
    }
}

/// Why an assignment was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignError {
    WrongBar { bar: i32 },
    NegativeSlot { slot: i32 },
    UnknownInstance { instance_id: i32 },
}

/// A use request over the recovered request fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UseRequest {
    pub bar: i32,
    pub slot: i32,
    /// Recovered client status bits; any overlapping cast-block bit refuses the use.
    pub status_mask: u32,
    pub now_ms: i64,
}

/// Caller-supplied durations; no historical value is assumed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsePolicy {
    pub cooldown_ms: i64,
    pub effect_duration_ms: i64,
}

/// Why a use was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseRejection {
    WrongBar {
        bar: i32,
    },
    SlotNotAssigned {
        bar: i32,
        slot: i32,
    },
    /// The assigned instance references an item id with no recovered catalogue row.
    UncataloguedItem {
        item_id: i32,
    },
    Depleted {
        instance_id: i32,
    },
    BlockedStatus {
        blocked_bits: u32,
    },
    OnCooldown {
        ready_at_ms: i64,
        remaining_ms: i64,
    },
    ReplayedOrStaleRequest {
        last_accept_ms: i64,
    },
}

/// Accepted use or a refusal. The consumed variant reports exactly what changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseOutcome {
    Consumed {
        item_id: i32,
        instance_id: i32,
        ability_id: i32,
        remaining: u32,
        cooldown_ready_at_ms: i64,
        effect_expires_at_ms: Option<i64>,
    },
    Refused(UseRejection),
}

/// Reconstructed once-only consumption, cooldown and expiry model over owned instances.
#[derive(Debug, Default)]
pub struct ConsumableLedger {
    instances: Vec<OwnedInstance>,
    assignments: Vec<SlotAssignment>,
    cooldowns: Vec<SlotCooldown>,
    effects: Vec<ActiveConsumableEffect>,
    last_accept_ms: Option<i64>,
}

impl ConsumableLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an already-owned instance. Upserts by instance id; this is not a grant path.
    pub fn seed_owned(&mut self, instance_id: i32, item_id: i32, count: u32) {
        match self
            .instances
            .iter_mut()
            .find(|row| row.instance_id == instance_id)
        {
            Some(row) => {
                row.item_id = item_id;
                row.count = count;
            }
            None => self.instances.push(OwnedInstance {
                instance_id,
                item_id,
                count,
            }),
        }
    }

    /// Apply the recovered 42/6 assignment for the item bar only.
    pub fn assign(&mut self, bar: i32, slot: i32, instance_id: i32) -> Result<(), AssignError> {
        if bar != ITEM_ACTION_BAR_ID {
            return Err(AssignError::WrongBar { bar });
        }
        if slot < 0 {
            return Err(AssignError::NegativeSlot { slot });
        }
        if !self
            .instances
            .iter()
            .any(|row| row.instance_id == instance_id)
        {
            return Err(AssignError::UnknownInstance { instance_id });
        }
        match self
            .assignments
            .iter_mut()
            .find(|row| row.bar == bar && row.slot == slot)
        {
            Some(row) => row.instance_id = instance_id,
            None => self.assignments.push(SlotAssignment {
                bar,
                slot,
                instance_id,
            }),
        }
        Ok(())
    }

    pub fn owned_count(&self, instance_id: i32) -> Option<u32> {
        self.instances
            .iter()
            .find(|row| row.instance_id == instance_id)
            .map(|row| row.count)
    }

    pub fn cooldown_ready_at(&self, bar: i32, slot: i32) -> Option<i64> {
        self.cooldowns
            .iter()
            .find(|row| row.bar == bar && row.slot == slot)
            .map(|row| row.ready_at_ms)
    }

    pub fn effects(&self) -> &[ActiveConsumableEffect] {
        &self.effects
    }

    /// Effects that have not reached their expiry boundary yet.
    pub fn live_effects(&self, now_ms: i64) -> impl Iterator<Item = &ActiveConsumableEffect> + '_ {
        self.effects
            .iter()
            .filter(move |effect| !effect.expired(now_ms))
    }

    /// Reconstructed policy. Refusals never consume; an accepted use consumes exactly one item,
    /// starts one recast window and replaces that instance's effect window.
    pub fn use_item(&mut self, request: UseRequest, policy: UsePolicy) -> UseOutcome {
        if request.bar != ITEM_ACTION_BAR_ID {
            return UseOutcome::Refused(UseRejection::WrongBar { bar: request.bar });
        }
        let Some(assignment) = self
            .assignments
            .iter()
            .find(|row| row.bar == request.bar && row.slot == request.slot)
            .copied()
        else {
            return UseOutcome::Refused(UseRejection::SlotNotAssigned {
                bar: request.bar,
                slot: request.slot,
            });
        };
        let Some(instance_index) = self
            .instances
            .iter()
            .position(|row| row.instance_id == assignment.instance_id)
        else {
            return UseOutcome::Refused(UseRejection::SlotNotAssigned {
                bar: request.bar,
                slot: request.slot,
            });
        };
        let instance = self.instances[instance_index];
        let Some(item) = consumable(instance.item_id) else {
            return UseOutcome::Refused(UseRejection::UncataloguedItem {
                item_id: instance.item_id,
            });
        };
        if instance.count == 0 {
            return UseOutcome::Refused(UseRejection::Depleted {
                instance_id: instance.instance_id,
            });
        }
        let blocked_bits = request.status_mask & CAST_BLOCK_STATUS_MASK;
        if blocked_bits != 0 {
            return UseOutcome::Refused(UseRejection::BlockedStatus { blocked_bits });
        }
        if let Some(cooldown) = self
            .cooldowns
            .iter()
            .find(|row| row.bar == request.bar && row.slot == request.slot)
        {
            if request.now_ms < cooldown.ready_at_ms {
                return UseOutcome::Refused(UseRejection::OnCooldown {
                    ready_at_ms: cooldown.ready_at_ms,
                    remaining_ms: cooldown.ready_at_ms - request.now_ms,
                });
            }
        }
        if let Some(last_accept_ms) = self.last_accept_ms {
            if request.now_ms <= last_accept_ms {
                return UseOutcome::Refused(UseRejection::ReplayedOrStaleRequest {
                    last_accept_ms,
                });
            }
        }

        self.instances[instance_index].count = instance.count - 1;
        let remaining = self.instances[instance_index].count;
        let ready_at_ms = request.now_ms + policy.cooldown_ms.max(0);
        match self
            .cooldowns
            .iter_mut()
            .find(|row| row.bar == request.bar && row.slot == request.slot)
        {
            Some(row) => row.ready_at_ms = ready_at_ms,
            None => self.cooldowns.push(SlotCooldown {
                bar: request.bar,
                slot: request.slot,
                ready_at_ms,
            }),
        }
        let effect_expires_at_ms = if policy.effect_duration_ms > 0 {
            let expires_at_ms = request.now_ms + policy.effect_duration_ms;
            self.effects
                .retain(|effect| effect.instance_id != instance.instance_id);
            self.effects.push(ActiveConsumableEffect {
                instance_id: instance.instance_id,
                item_id: instance.item_id,
                ability_id: item.ability_id,
                started_at_ms: request.now_ms,
                expires_at_ms,
            });
            Some(expires_at_ms)
        } else {
            None
        };
        self.last_accept_ms = Some(request.now_ms);
        UseOutcome::Consumed {
            item_id: instance.item_id,
            instance_id: instance.instance_id,
            ability_id: item.ability_id,
            remaining,
            cooldown_ready_at_ms: ready_at_ms,
            effect_expires_at_ms,
        }
    }
}
