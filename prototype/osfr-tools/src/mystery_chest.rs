//! Mystery Chest protocol codec.
//!
//! Scope: the three packets the reconstructed server needs for the inventory Mystery Chest flow.
//! Layouts below are little-endian. Source evidence is the pinned community server
//! (`lab/references/Sanctuary/src`) cross-checked against the original Windows client
//! `1.910.1.530630` pseudocode (`lab/decompiled/client/FreeRealms/pseudocode.c`) and Lua
//! (`lab/decompiled/client-ui/ScriptsBase.lua`); exact addresses are in the integration test and
//! the private evidence report.
//!
//! ## Opcode framing
//!
//! - Root 42 (`BaseInventoryPacket`) is `u16 opcode + u16 subtype`. The native constructor
//!   `FUN_00a10400` writes `0x2a` and then the subtype.
//! - Root 38 (`BaseClientUpdatePacket`) is `u16 opcode + u16 subtype` (`FUN_0089d180` writes
//!   `0x26`).
//! - Root 50 (`RewardBasePacket`) is `u16 opcode + u8 subtype` (`FUN_00b88ad0` sets base
//!   `0x32`; the base reader `FUN_00b89190` consumes two bytes then one byte). The subtype-1
//!   bundle therefore starts `32 00 01`, not `32 00 01 00`.
//!
//! ## Client requests
//!
//! | Subtype | Packet | Body |
//! | --- | --- | --- |
//! | 15 (`0x0f`) | `InventoryPacketUseImmediateActivationItem` | `i32` item instance id |
//! | 16 (`0x10`) | `InventoryPacketCheckImmediateActivationItem` | `i32` item instance id |
//!
//! The native client constructors pass `0x0f` (`FUN_00c10940`) and `0x10` (`FUN_00c109f0`);
//! pinned Sanctuary's subtype table agrees. The integration's initial subtype-14 test was
//! incorrect and was replaced with subtype 16 before native acceptance.
//!
//! ## Check reply (38/30)
//!
//! `i32 itemId, i32 categoryId(243), i32 bundleId(5021), i32 titleStringId(436474),
//! i32 bodyStringId(436472), u8 success`. Lua `InventoryHandler.OnIsItemUsable` receives
//! `(success, guid, catId, bundleId, titleStrId, bodyStrId)`; `success == 1` routes the dialog's
//! Okay button to `EquipScreen.UseImmediateActivationItem(guid)`, any other value routes to the
//! marketplace bundle instead.
//!
//! ## Reward bundle (50/1)
//!
//! Body offsets, relative to the byte after the 3-byte header:
//!
//! | Offset | Width | Meaning |
//! | --- | --- | --- |
//! | 0 | 1 | success flag |
//! | 1 | 4 | coin award amount |
//! | 5 | 4 | XP/stars award amount |
//! | 9 | 4 | not resolved (written zero) |
//! | 13 | 4 | reward kind (`16` = item notification / Mystery Chest) |
//! | 17 | 4 | not resolved (zero) |
//! | 21 | 4 | not resolved (zero) |
//! | 25 | 4 | multiplier, float `1.0` |
//! | 29 | 4 | not resolved (zero) |
//! | 33 | 4 | not resolved (zero) |
//! | 37 | 8 | source guid |
//! | 45 | 8 | player guid |
//! | 53 | 4 | panel icon id |
//! | 57 | 4 | panel name/string id |
//! | 61 | 4 | entry count |
//! | 65 | .. | entries |
//! | after entries | 4 | appended final word (zero) |
//!
//! An empty-entry body is 69 bytes, matching `FUN_008e7930` and the reward-toast note. Each item
//! entry is: `i32 type(1), u8 hidden, i32 iconId, i32 iconTintId, i32 nameId, i32 quantity,
//! i32 definitionId, i32 tint, i32-prefixed UTF-8 description (empty -> single zero i32),
//! i32 itemTextColor, u8 membersOnly, i32 item instance guid`. The native item entry allocates
//! `0x48` bytes and stores its instance guid at `+0x44`, i.e. one i32 past the 68-byte common
//! fields, which matches the community serializer used by the supplied originals.

use std::io;

/// Root opcode for inventory (client request) packets: `u16` opcode + `u16` subtype.
pub const OPCODE_INVENTORY: u16 = 42;
/// Root opcode for client-update (server reply) packets: `u16` opcode + `u16` subtype.
pub const OPCODE_CLIENT_UPDATE: u16 = 38;
/// Root opcode for reward packets: `u16` opcode + `u8` subtype.
pub const OPCODE_REWARD: u16 = 50;

/// `InventoryPacketUseImmediateActivationItem` subtype.
pub const SUBTYPE_USE_IMMEDIATE_ACTIVATION: u16 = 15;
/// `InventoryPacketCheckImmediateActivationItem` subtype.
pub const SUBTYPE_CHECK_IMMEDIATE_ACTIVATION: u16 = 16;
/// `ClientUpdatePacketImmediateActivationCheck` subtype.
pub const SUBTYPE_IMMEDIATE_ACTIVATION_REPLY: u16 = 30;
/// `RewardBundlePacket` subtype.
pub const SUBTYPE_REWARD_BUNDLE: u8 = 1;

/// Reward kind that selects the item-notification helper and the Mystery Chest panel.
pub const REWARD_KIND_MYSTERY_CHEST: i32 = 16;
/// Marketplace category carried in the check reply (supplied originals use the generic chest set).
pub const CHECK_REPLY_CATEGORY_ID: i32 = 243;
/// Marketplace bundle carried in the check reply.
pub const CHECK_REPLY_BUNDLE_ID: i32 = 5021;
/// Dialog title string id carried in the check reply.
pub const CHECK_REPLY_TITLE_STRING_ID: i32 = 436474;
/// Dialog body string id carried in the check reply.
pub const CHECK_REPLY_BODY_STRING_ID: i32 = 436472;
/// Reward bundle entry type for a plain item.
pub const ENTRY_TYPE_ITEM: i32 = 1;

/// Decoded Mystery Chest client request. The `i32` is the item instance id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// `42/16`: the client asks whether the item can be opened.
    Check(i32),
    /// `42/15`: the client confirmed the dialog and wants the item consumed.
    Use(i32),
}

/// Strictly parses one unmarshalled (tunnel payload) Mystery Chest request.
///
/// Returns `Ok(None)` for packets this codec does not own, including other root opcodes and
/// other inventory subtypes. Returns `Err(UnexpectedEof)` for a truncated owned packet and
/// `Err(InvalidData)` for trailing bytes inside an owned packet or a non-positive instance id.
pub fn parse_request(bytes: &[u8]) -> io::Result<Option<Request>> {
    let opcode = read_u16(bytes, 0)?;
    if opcode != OPCODE_INVENTORY {
        return Ok(None);
    }
    let subtype = read_u16(bytes, 2)?;
    if subtype != SUBTYPE_USE_IMMEDIATE_ACTIVATION && subtype != SUBTYPE_CHECK_IMMEDIATE_ACTIVATION
    {
        return Ok(None);
    }
    if bytes.len() < 8 {
        return Err(eof("immediate activation request body"));
    }
    if bytes.len() > 8 {
        return Err(invalid("immediate activation request has trailing bytes"));
    }
    let item_id = i32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if item_id <= 0 {
        return Err(invalid("item instance id must be positive"));
    }
    Ok(Some(if subtype == SUBTYPE_USE_IMMEDIATE_ACTIVATION {
        Request::Use(item_id)
    } else {
        Request::Check(item_id)
    }))
}

/// Builds the `38/30` check reply that lets the client show its open-item dialog.
///
/// `success == true` (byte 1) makes the dialog's Okay button send the use request for this
/// instance; `false` routes Okay to the marketplace bundle instead.
pub fn check_reply(item_id: i32, success: bool) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(25);
    bytes.extend_from_slice(&OPCODE_CLIENT_UPDATE.to_le_bytes());
    bytes.extend_from_slice(&SUBTYPE_IMMEDIATE_ACTIVATION_REPLY.to_le_bytes());
    bytes.extend_from_slice(&item_id.to_le_bytes());
    bytes.extend_from_slice(&CHECK_REPLY_CATEGORY_ID.to_le_bytes());
    bytes.extend_from_slice(&CHECK_REPLY_BUNDLE_ID.to_le_bytes());
    bytes.extend_from_slice(&CHECK_REPLY_TITLE_STRING_ID.to_le_bytes());
    bytes.extend_from_slice(&CHECK_REPLY_BODY_STRING_ID.to_le_bytes());
    bytes.push(u8::from(success));
    bytes
}

/// Builds the `50/1` reward bundle that opens the Mystery Chest panel for one item.
///
/// Coins and XP stay zero (item-only grant), the multiplier is exactly `1.0`, the reward kind is
/// [`REWARD_KIND_MYSTERY_CHEST`] and the bundle carries a single quantity-1 item entry.
/// `tint` and `icon_tint` are passed through untouched so `-1` keeps the originaldata sentinel.
pub fn item_reward(
    source_guid: u64,
    player_guid: u64,
    item_id: i32,
    definition_id: i32,
    tint: i32,
    icon_tint: i32,
    icon_id: i32,
    name_id: i32,
) -> io::Result<Vec<u8>> {
    if item_id <= 0 {
        return Err(invalid("reward item instance id must be positive"));
    }
    if definition_id <= 0 {
        return Err(invalid("reward item definition id must be positive"));
    }

    let mut bytes = Vec::with_capacity(114);
    bytes.extend_from_slice(&OPCODE_REWARD.to_le_bytes());
    bytes.push(SUBTYPE_REWARD_BUNDLE);

    bytes.push(1); // success
    bytes.extend_from_slice(&0i32.to_le_bytes()); // coin award
    bytes.extend_from_slice(&0i32.to_le_bytes()); // XP award
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&REWARD_KIND_MYSTERY_CHEST.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&1.0f32.to_le_bytes()); // multiplier
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&0i32.to_le_bytes()); // unresolved, zero
    bytes.extend_from_slice(&source_guid.to_le_bytes());
    bytes.extend_from_slice(&player_guid.to_le_bytes());
    bytes.extend_from_slice(&icon_id.to_le_bytes());
    bytes.extend_from_slice(&name_id.to_le_bytes());
    bytes.extend_from_slice(&1i32.to_le_bytes()); // one entry

    bytes.extend_from_slice(&ENTRY_TYPE_ITEM.to_le_bytes());
    bytes.push(0); // not hidden
    bytes.extend_from_slice(&icon_id.to_le_bytes());
    bytes.extend_from_slice(&icon_tint.to_le_bytes());
    bytes.extend_from_slice(&name_id.to_le_bytes());
    bytes.extend_from_slice(&1i32.to_le_bytes()); // quantity
    bytes.extend_from_slice(&definition_id.to_le_bytes());
    bytes.extend_from_slice(&tint.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes()); // empty description
    bytes.extend_from_slice(&0i32.to_le_bytes()); // item text color
    bytes.push(0); // not members only
    bytes.extend_from_slice(&item_id.to_le_bytes());

    bytes.extend_from_slice(&0i32.to_le_bytes()); // appended final word
    Ok(bytes)
}

fn read_u16(bytes: &[u8], offset: usize) -> io::Result<u16> {
    let pair = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| eof("packet header"))?;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}

fn eof(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, format!("truncated {what}"))
}

fn invalid(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, what.to_string())
}
