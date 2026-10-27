//! 045: cosmetic equipment validation and native-equip replay.
//!
//! ORIGINAL facts carried by this module (all preserved outside Git, see
//! `notes/Research/Pieces/045 Cosmetic equipment.md`):
//! * The supplied client logs an applied equip as
//!   `"EquipItem: Slot=%d, ModelName=%s, TextureAlias=%s, TintAlias=%s, TintIdOverride=%d,
//!   Effect=%d."` (`lab/decompiled/client/FreeRealms/pseudocode.c` line 1099052). That field order
//!   is what `src/inventory.rs` serializes as the attachment.
//! * `ClientItemDefinitions.json` (community starter import, AGPL-3.0, revision
//!   `9e082025391177d5a33ee18c714ddd06b457a695`) carries a `Slot` and `CategoryId` on each
//!   wearable row. `restoration::piece_045::COSMETIC_SLOTS` transcribes the seven recovered
//!   appearance pairs and the row counts the independent checker re-derives.
//! * `lab/references/opening-anvil-placement-v1/content/inventory.json` is the running content:
//!   the captured `PlayerCode.SendPlayerUpdateItemDefinitions` payload with community starter rows
//!   merged in. Its `definitions[].model`, `.texture`, `.tint_alias` and `.default_tint` are the
//!   appearance values production sends.
//! * The community handler `Sanctuary.Gateway/Handlers/BaseInventoryPacket/
//!   InventoryPacketEquipByGuidHandler.cs` resolves the wire tint as
//!   `clientItem.Tint == 0 ? clientItemDefinition.Icon.TintId : clientItem.Tint` and fills the
//!   attachment from the definition only. Production's `write_attachment` matches that rule.
//!
//! RECONSTRUCTED here: the appearance predicate set, the refusal of slot-mismatched equips, the
//! idempotent replay classification and the restart comparison. They are validation policy over
//! the recovered shapes, not recovered server code. Nothing in this module grants an item, changes
//! economy state, or proves native equip; the lead still owns native acceptance.
//!
//! INTEGRATION: when the lead registers `pub mod piece_045;`, replace the `#[path]` include below
//! with `use crate::inventory::{Inventory, InventoryState};` and drop the re-export block. Until
//! then the owned test target includes the module by path, exactly as `tests/inventory.rs` uses
//! the crate module.

use std::io;

#[path = "../inventory.rs"]
pub mod inventory;

#[allow(unused_imports)]
pub use inventory::{Inventory, InventoryState};

/// Recovered appearance slots: `Slot` -> `CategoryId` pairs from the pinned starter catalogue,
/// with the number of recovered rows in that pair. Slots 7+ (tools, weapons, shards, jewelry) are
/// deliberately absent: they are gear mechanics, owned by other work items. The declared pair is
/// one category per slot; slot 17 also carries 541 `Shirts & Dresses` and 55 `Hats` costume
/// overrides, which the private recovery checker `piece_045.py` reports instead of hiding behind the dominant
/// count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CosmeticSlot {
    pub slot: i32,
    pub category_id: i32,
    pub category: &'static str,
    pub name_id: i32,
    pub recovered_rows: u32,
}

pub const COSMETIC_SLOTS: [CosmeticSlot; 7] = [
    CosmeticSlot {
        slot: 1,
        category_id: 101,
        category: "Hats",
        name_id: 23319,
        recovered_rows: 7522,
    },
    CosmeticSlot {
        slot: 2,
        category_id: 102,
        category: "Gloves",
        name_id: 23320,
        recovered_rows: 3189,
    },
    CosmeticSlot {
        slot: 3,
        category_id: 103,
        category: "Shirts & Dresses",
        name_id: 23321,
        recovered_rows: 8254,
    },
    CosmeticSlot {
        slot: 4,
        category_id: 104,
        category: "Pants",
        name_id: 23322,
        recovered_rows: 5873,
    },
    CosmeticSlot {
        slot: 5,
        category_id: 105,
        category: "Shoes",
        name_id: 2482,
        recovered_rows: 5867,
    },
    CosmeticSlot {
        slot: 6,
        category_id: 106,
        category: "Miscellaneous",
        name_id: 23324,
        recovered_rows: 2301,
    },
    CosmeticSlot {
        slot: 17,
        category_id: 201,
        category: "Costumes",
        name_id: 5273,
        recovered_rows: 93,
    },
];

/// The only model placeholders the recovered cosmetic rows use; 25,483 rows carry both, 6,366
/// carry `<race>` only and 6 carry none.
pub const MODEL_PLACEHOLDERS: [&str; 2] = ["<race>", "<gender>"];

/// Recovered dominant tint alias on the cosmetic rows (`dyetint`); the checker records the two
/// recovered outliers instead of silently widening the rule.
pub const TINT_ALIAS: &str = "dyetint";

/// Bound shared with the content loader: one attachment text field.
pub const MAX_APPEARANCE_TEXT: usize = 4096;
/// Reconstructed bound on an attachment list in one profile.
pub const MAX_APPEARANCE_ROWS: usize = 1024;

pub fn cosmetic_slot(slot: i32) -> Option<&'static CosmeticSlot> {
    COSMETIC_SLOTS.iter().find(|row| row.slot == slot)
}

/// One applied appearance, in the client's own `EquipItem` log order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Appearance {
    pub model: String,
    pub texture: String,
    pub tint_alias: String,
    pub tint: i32,
    pub effect: i32,
    pub slot: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CosmeticError {
    NotCosmeticSlot {
        slot: i32,
    },
    TextTooLong {
        slot: i32,
    },
    EmptyModel {
        slot: i32,
    },
    ModelNotMesh {
        slot: i32,
    },
    UnknownModelPlaceholder {
        slot: i32,
        token: String,
    },
    EmptyTexture {
        slot: i32,
    },
    TexturePlaceholder {
        slot: i32,
    },
    EmptyTintAlias {
        slot: i32,
    },
    NegativeTint {
        slot: i32,
        tint: i32,
    },
    UnownedItem {
        item_id: i32,
    },
    Malformed(&'static str),
    Inventory {
        kind: io::ErrorKind,
        message: String,
    },
    NotEquipRequest {
        family: u16,
        opcode: u16,
    },
    RestartDiffers {
        profile_id: i32,
    },
    RestartProfileCount {
        before: usize,
        after: usize,
    },
}

impl CosmeticError {
    pub fn from_io(error: io::Error) -> Self {
        Self::Inventory {
            kind: error.kind(),
            message: error.to_string(),
        }
    }
}

impl std::fmt::Display for CosmeticError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotCosmeticSlot { slot } => write!(formatter, "slot {slot} is not cosmetic"),
            Self::TextTooLong { slot } => {
                write!(formatter, "slot {slot} appearance text is too long")
            }
            Self::EmptyModel { slot } => write!(formatter, "slot {slot} has no model"),
            Self::ModelNotMesh { slot } => write!(formatter, "slot {slot} model is not a mesh"),
            Self::UnknownModelPlaceholder { slot, token } => {
                write!(
                    formatter,
                    "slot {slot} model uses unknown placeholder {token}"
                )
            }
            Self::EmptyTexture { slot } => write!(formatter, "slot {slot} has no texture alias"),
            Self::TexturePlaceholder { slot } => {
                write!(formatter, "slot {slot} texture alias carries a placeholder")
            }
            Self::EmptyTintAlias { slot } => write!(formatter, "slot {slot} has no tint alias"),
            Self::NegativeTint { slot, tint } => {
                write!(formatter, "slot {slot} tint {tint} is negative")
            }
            Self::UnownedItem { item_id } => {
                write!(formatter, "equipment names unowned item {item_id}")
            }
            Self::Malformed(reason) => write!(formatter, "{reason}"),
            Self::Inventory { kind, message } => write!(formatter, "inventory {kind:?}: {message}"),
            Self::NotEquipRequest { family, opcode } => {
                write!(formatter, "{family}/{opcode} is not an equip request")
            }
            Self::RestartDiffers { profile_id } => {
                write!(formatter, "profile {profile_id} changed across the restart")
            }
            Self::RestartProfileCount { before, after } => {
                write!(
                    formatter,
                    "restart profile count changed from {before} to {after}"
                )
            }
        }
    }
}

impl std::error::Error for CosmeticError {}

/// Enforce the recovered shape of an applied appearance. The rules are:
/// cosmetic slot; model is a non-empty `.adr` mesh whose placeholders are only `<race>`/`<gender>`;
/// texture alias is a non-empty concrete name; tint alias is present; tint is not negative.
pub fn validate_appearance(appearance: &Appearance) -> Result<(), CosmeticError> {
    let slot = appearance.slot;
    if cosmetic_slot(slot).is_none() {
        return Err(CosmeticError::NotCosmeticSlot { slot });
    }
    if [
        &appearance.model,
        &appearance.texture,
        &appearance.tint_alias,
    ]
    .iter()
    .any(|text| text.len() > MAX_APPEARANCE_TEXT)
    {
        return Err(CosmeticError::TextTooLong { slot });
    }
    if appearance.model.is_empty() {
        return Err(CosmeticError::EmptyModel { slot });
    }
    if !appearance.model.ends_with(".adr") {
        return Err(CosmeticError::ModelNotMesh { slot });
    }
    for token in placeholders(&appearance.model) {
        if !MODEL_PLACEHOLDERS.contains(&token.as_str()) {
            return Err(CosmeticError::UnknownModelPlaceholder { slot, token });
        }
    }
    if appearance.texture.is_empty() {
        return Err(CosmeticError::EmptyTexture { slot });
    }
    if appearance.texture.contains(['<', '>']) {
        return Err(CosmeticError::TexturePlaceholder { slot });
    }
    if appearance.tint_alias.is_empty() {
        return Err(CosmeticError::EmptyTintAlias { slot });
    }
    if appearance.tint < 0 {
        return Err(CosmeticError::NegativeTint {
            slot,
            tint: appearance.tint,
        });
    }
    Ok(())
}

fn placeholders(model: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = model;
    while let Some(start) = rest.find('<') {
        let after = &rest[start..];
        match after.find('>') {
            Some(end) => {
                tokens.push(after[..=end].to_owned());
                rest = &after[end + 1..];
            }
            None => {
                tokens.push(after.to_owned());
                break;
            }
        }
    }
    tokens
}

/// Parse production's count-prefixed `profile_attachments` list.
pub fn parse_profile_attachments(bytes: &[u8]) -> Result<Vec<Appearance>, CosmeticError> {
    let mut cursor = Cursor { bytes, offset: 0 };
    let count = cursor.read_i32()?;
    if count < 0 || count as usize > MAX_APPEARANCE_ROWS {
        return Err(CosmeticError::Malformed(
            "attachment count is out of bounds",
        ));
    }
    let mut rows = Vec::new();
    for _ in 0..count {
        rows.push(cursor.read_appearance()?);
    }
    if cursor.offset != bytes.len() {
        return Err(CosmeticError::Malformed(
            "attachment list has trailing bytes",
        ));
    }
    Ok(rows)
}

/// A parsed `ClientUpdatePacketEquipItem` (family 38, opcode 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquipReply {
    pub item_id: i32,
    pub appearance: Appearance,
    pub profile_id: i32,
    pub class_id: i32,
    pub equip: bool,
}

pub fn parse_equip_reply(packet: &[u8]) -> Result<EquipReply, CosmeticError> {
    if packet.len() < 4 || packet[..2] != 38u16.to_le_bytes() || packet[2..4] != 5u16.to_le_bytes()
    {
        return Err(CosmeticError::Malformed(
            "not a ClientUpdatePacketEquipItem",
        ));
    }
    let mut cursor = Cursor {
        bytes: packet,
        offset: 4,
    };
    let item_id = cursor.read_i32()?;
    let appearance = cursor.read_appearance()?;
    let profile_id = cursor.read_i32()?;
    let class_id = cursor.read_i32()?;
    let Some(equip) = packet.get(cursor.offset) else {
        return Err(CosmeticError::Malformed("truncated equip reply"));
    };
    cursor.offset += 1;
    if cursor.offset != packet.len() {
        return Err(CosmeticError::Malformed("equip reply has trailing bytes"));
    }
    Ok(EquipReply {
        item_id,
        appearance,
        profile_id,
        class_id,
        equip: *equip != 0,
    })
}

/// A parsed `PlayerUpdatePacketEquipItemChange` (family 35, opcode 6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibleReply {
    pub guid: u64,
    pub item_id: i32,
    pub appearance: Appearance,
    pub profile_id: i32,
    pub active: i32,
    pub wield_type: i32,
}

pub fn parse_visible_reply(packet: &[u8]) -> Result<VisibleReply, CosmeticError> {
    if packet.len() < 4 || packet[..2] != 35u16.to_le_bytes() || packet[2..4] != 6u16.to_le_bytes()
    {
        return Err(CosmeticError::Malformed(
            "not a PlayerUpdatePacketEquipItemChange",
        ));
    }
    let mut cursor = Cursor {
        bytes: packet,
        offset: 4,
    };
    let guid = u64::from_le_bytes(
        packet
            .get(cursor.offset..cursor.offset + 8)
            .and_then(|slice| slice.try_into().ok())
            .ok_or(CosmeticError::Malformed("truncated equip reply"))?,
    );
    cursor.offset += 8;
    let item_id = cursor.read_i32()?;
    let appearance = cursor.read_appearance()?;
    let profile_id = cursor.read_i32()?;
    let active = cursor.read_i32()?;
    let wield_type = cursor.read_i32()?;
    if cursor.offset != packet.len() {
        return Err(CosmeticError::Malformed("equip reply has trailing bytes"));
    }
    Ok(VisibleReply {
        guid,
        item_id,
        appearance,
        profile_id,
        active,
        wield_type,
    })
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Cursor<'_> {
    fn read_i32(&mut self) -> Result<i32, CosmeticError> {
        let value = self
            .bytes
            .get(self.offset..self.offset + 4)
            .and_then(|slice| slice.try_into().ok())
            .map(i32::from_le_bytes)
            .ok_or(CosmeticError::Malformed("truncated attachment field"))?;
        self.offset += 4;
        Ok(value)
    }

    fn read_text(&mut self) -> Result<String, CosmeticError> {
        let length = self.read_i32()?;
        if length < 0 || length as usize > MAX_APPEARANCE_TEXT {
            return Err(CosmeticError::Malformed(
                "attachment text length is out of bounds",
            ));
        }
        let text = self
            .bytes
            .get(self.offset..self.offset + length as usize)
            .ok_or(CosmeticError::Malformed("truncated attachment field"))?;
        self.offset += length as usize;
        String::from_utf8(text.to_vec())
            .map_err(|_| CosmeticError::Malformed("attachment text is not UTF-8"))
    }

    fn read_appearance(&mut self) -> Result<Appearance, CosmeticError> {
        Ok(Appearance {
            model: self.read_text()?,
            texture: self.read_text()?,
            tint_alias: self.read_text()?,
            tint: self.read_i32()?,
            effect: self.read_i32()?,
            slot: self.read_i32()?,
        })
    }
}

/// One equipped cosmetic row resolved from the saved state and the wire attachment list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EquippedCosmetic {
    pub profile_id: i32,
    pub slot: i32,
    pub item_id: i32,
    pub definition_id: i32,
    pub tint: i32,
    pub appearance: Appearance,
}

/// Validate every equipped cosmetic in an inventory. Ownership comes from the saved item rows,
/// the applied appearance comes from `profile_attachments`, and non-cosmetic slots (weapons,
/// tools, shards) are left to their own work items.
pub fn inspect(inventory: &Inventory) -> Result<Vec<EquippedCosmetic>, CosmeticError> {
    let state = inventory.state();
    let mut profiles: Vec<i32> = state
        .equipment
        .iter()
        .map(|entry| entry.profile_id)
        .collect();
    profiles.sort_unstable();
    profiles.dedup();
    let mut rows = Vec::new();
    for profile_id in profiles {
        let equipped: Vec<_> = state
            .equipment
            .iter()
            .filter(|entry| entry.profile_id == profile_id && entry.item_id != 0)
            .collect();
        for entry in &equipped {
            if !state.items.iter().any(|item| item.id == entry.item_id) {
                return Err(CosmeticError::UnownedItem {
                    item_id: entry.item_id,
                });
            }
        }
        let attachments = parse_profile_attachments(
            &inventory
                .profile_attachments(profile_id)
                .map_err(CosmeticError::from_io)?,
        )?;
        if attachments.len() != equipped.len() {
            return Err(CosmeticError::Malformed(
                "attachment list does not match the equipment rows",
            ));
        }
        for (entry, appearance) in equipped.iter().zip(attachments) {
            if cosmetic_slot(entry.slot).is_none() {
                continue;
            }
            validate_appearance(&appearance)?;
            if appearance.slot != entry.slot {
                return Err(CosmeticError::Malformed(
                    "attachment slot differs from the equipment row",
                ));
            }
            let item = state
                .items
                .iter()
                .find(|item| item.id == entry.item_id)
                .expect("ownership was checked above");
            rows.push(EquippedCosmetic {
                profile_id,
                slot: entry.slot,
                item_id: entry.item_id,
                definition_id: item.definition_id,
                tint: item.tint,
                appearance,
            });
        }
    }
    Ok(rows)
}

/// Offline replay of one recovered equip request against production's inventory handler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Replay {
    Accepted {
        reply: EquipReply,
        visible: Option<VisibleReply>,
        equipment: Vec<u8>,
    },
    Refused {
        error: CosmeticError,
        equipment: Vec<u8>,
    },
}

/// Feed a raw `42/3` (equip by instance) or `42/8` (equip by item record) request through the real
/// handler, then validate the appearance it produced. Any other request is reported as
/// `NotEquipRequest` instead of being treated as an equip. `profile_id` must be the profile the
/// request names; the equipment bytes are always taken for that profile.
pub fn replay_equip(inventory: &mut Inventory, profile_id: i32, request: &[u8]) -> Replay {
    let refusal = |error| Replay::Refused {
        error,
        equipment: inventory.profile_equipment(profile_id),
    };
    if request.len() < 4 {
        return refusal(CosmeticError::Malformed("equip request is truncated"));
    }
    let family = u16::from_le_bytes([request[0], request[1]]);
    let opcode = u16::from_le_bytes([request[2], request[3]]);
    if family != 42 || !matches!(opcode, 3 | 8) {
        return refusal(CosmeticError::NotEquipRequest { family, opcode });
    }
    let before = inventory.profile_equipment(profile_id);
    match inventory.handle(request) {
        Err(error) => Replay::Refused {
            error: CosmeticError::from_io(error),
            equipment: before,
        },
        Ok(outcome) => {
            let equipment = inventory.profile_equipment(profile_id);
            let Some(packet) = outcome
                .replies
                .iter()
                .find(|packet| packet.starts_with(&[38, 0, 5, 0]))
            else {
                return Replay::Refused {
                    error: CosmeticError::Malformed(
                        "equip produced no ClientUpdatePacketEquipItem",
                    ),
                    equipment,
                };
            };
            let reply = match parse_equip_reply(packet)
                .and_then(|reply| validate_appearance(&reply.appearance).map(|()| reply))
            {
                Ok(reply) => reply,
                Err(error) => return Replay::Refused { error, equipment },
            };
            let visible = match outcome
                .replies
                .iter()
                .find(|packet| packet.starts_with(&[35, 0, 6, 0]))
            {
                Some(packet) => match parse_visible_reply(packet) {
                    Ok(visible) => Some(visible),
                    Err(error) => return Replay::Refused { error, equipment },
                },
                None => None,
            };
            Replay::Accepted {
                reply,
                visible,
                equipment,
            }
        }
    }
}

/// The bytes a restart must preserve for one profile: the saved equipment rows and the applied
/// attachment list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileBytes {
    pub profile_id: i32,
    pub equipment: Vec<u8>,
    pub attachments: Vec<u8>,
}

pub fn capture_profiles(inventory: &Inventory) -> Result<Vec<ProfileBytes>, CosmeticError> {
    let mut profiles: Vec<i32> = inventory
        .state()
        .equipment
        .iter()
        .map(|entry| entry.profile_id)
        .collect();
    profiles.sort_unstable();
    profiles.dedup();
    profiles
        .into_iter()
        .map(|profile_id| {
            Ok(ProfileBytes {
                profile_id,
                equipment: inventory.profile_equipment(profile_id),
                attachments: inventory
                    .profile_attachments(profile_id)
                    .map_err(CosmeticError::from_io)?,
            })
        })
        .collect()
}

pub fn assert_profiles_identical(
    before: &[ProfileBytes],
    after: &[ProfileBytes],
) -> Result<(), CosmeticError> {
    if before.len() != after.len() {
        return Err(CosmeticError::RestartProfileCount {
            before: before.len(),
            after: after.len(),
        });
    }
    for (before, after) in before.iter().zip(after) {
        if before.profile_id != after.profile_id
            || before.equipment != after.equipment
            || before.attachments != after.attachments
        {
            return Err(CosmeticError::RestartDiffers {
                profile_id: before.profile_id,
            });
        }
    }
    Ok(())
}
