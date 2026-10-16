//! Typed local inventory state and evidenced inventory packet shapes.
//! Private content is prepared from supplied JSON and byte-checked captures.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[path = "inventory_v2_fixes.rs"]
mod v2_fixes;

const MAX_ITEMS: usize = 4096;
const MAX_CONTENT: usize = 4 * 1024 * 1024;

#[path = "restoration/piece_081.rs"]
pub mod instance_ids;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RentalState {
    pub expiration: i64,
    pub upsells: i32,
    pub bundle_id: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryItem {
    pub definition_id: i32,
    pub tint: i32,
    pub id: i32,
    pub count: u32,
    pub consumed_count: u32,
    /// Reconstructed startup default is zero for old save snapshots (R).
    #[serde(default)]
    pub stored_count: u32,
    pub last_cast_time: i64,
    pub rental: Option<RentalState>,
    pub ability_count: i32,
    pub activate_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquippedItem {
    pub profile_id: i32,
    pub slot: i32,
    pub item_id: i32,
    pub category: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryState {
    pub coins: u32,
    pub items: Vec<InventoryItem>,
    pub equipment: Vec<EquippedItem>,
    /// Reconstructed allocation metadata; absent in legacy snapshots and never on wire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_instance_id: Option<i64>,
}

#[derive(Clone, Debug, Default)]
pub struct InventoryOutcome {
    /// Inner application messages, before the gateway's tunnel wrapper.
    pub replies: Vec<Vec<u8>>,
    pub changed: bool,
}

/// Fields from the community-verified ClientItemDefinition serializer.
#[derive(Clone, Copy, Debug)]
pub struct ItemPresentation {
    pub name_id: i32,
    pub icon_id: i32,
    pub icon_tint: i32,
    pub item_type: i32,
}

#[derive(Deserialize, PartialEq)]
struct Definition {
    id: i32,
    slot: i32,
    class_id: i32,
    default_tint: i32,
    max_stack: i32,
    model: String,
    texture: String,
    tint_alias: String,
    effect: i32,
    wield_type: i32,
    serialized_hex: String,
    sha256: String,
    #[serde(skip)]
    bytes: Vec<u8>,
}

#[derive(Deserialize)]
struct Profile {
    id: i32,
    allowed_classes: Vec<i32>,
    slots: Vec<i32>,
}

#[derive(Deserialize)]
struct Span {
    kind: String,
    offset: usize,
    length: usize,
    sha256: String,
    profile_id: i32,
}

#[derive(Deserialize)]
struct Manifest {
    schema_version: u32,
    player_guid: u64,
    default_state: InventoryState,
    #[serde(default)]
    initial_state: Option<InventoryState>,
    profiles: Vec<Profile>,
    spans: Vec<Span>,
    definitions: Vec<Definition>,
}

struct Content {
    player_guid: u64,
    default_state: InventoryState,
    supported_equipment: Vec<EquippedItem>,
    profiles: BTreeMap<i32, Profile>,
    spans: Vec<Span>,
    definitions: BTreeMap<i32, Definition>,
}

#[derive(Clone)]
pub struct Inventory {
    content: Arc<Content>,
    state: InventoryState,
}

impl Inventory {
    pub fn load(directory: &Path, state: Option<InventoryState>) -> io::Result<Self> {
        let file = File::open(directory.join("inventory.json"))?;
        if !file.metadata()?.is_file() {
            return Err(invalid("inventory content must be a regular file"));
        }
        let mut bytes = Vec::new();
        file.take((MAX_CONTENT + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_CONTENT {
            return Err(invalid("inventory content exceeds 4 MiB"));
        }
        let manifest: Manifest = serde_json::from_slice(&bytes).map_err(invalid)?;
        if manifest.schema_version != 1
            || manifest.player_guid == 0
            || manifest.definitions.len() > MAX_ITEMS
            || manifest.profiles.len() > 32
            || manifest.spans.len() != manifest.profiles.len() + 2
        {
            return Err(invalid("unsupported inventory content"));
        }
        Self::validate_state(&manifest.default_state)?;
        let mut definitions: BTreeMap<i32, Definition> = BTreeMap::new();
        for mut definition in manifest.definitions {
            definition.bytes = decode_hex(&definition.serialized_hex)?;
            if definition.bytes.len() < 4
                || definition.bytes.len() > 65_536
                || definition.bytes[..4] != definition.id.to_le_bytes()
                || hash(&definition.bytes) != definition.sha256
                || definition.id <= 0
                || [
                    definition.model.len(),
                    definition.texture.len(),
                    definition.tint_alias.len(),
                ]
                .iter()
                .any(|size| *size > 4096)
            {
                return Err(invalid("invalid inventory definition content"));
            }
            if let Some(previous) = definitions.get(&definition.id) {
                // The captured catalog repeats several byte-identical entries.
                // Conflicting definitions still fail instead of changing meaning.
                if previous != &definition {
                    return Err(invalid("conflicting inventory definition"));
                }
            } else {
                definitions.insert(definition.id, definition);
            }
        }
        let mut profiles = BTreeMap::new();
        for profile in manifest.profiles {
            if profile.id <= 0
                || profile.slots.len() > 64
                || profile.allowed_classes.len() > 1024
                || profiles.insert(profile.id, profile).is_some()
            {
                return Err(invalid("invalid inventory profiles"));
            }
        }
        let initial_state = manifest.initial_state;
        let supported_equipment = initial_state
            .as_ref()
            .unwrap_or(&manifest.default_state)
            .equipment
            .clone();
        let content = Arc::new(Content {
            player_guid: manifest.player_guid,
            default_state: manifest.default_state,
            supported_equipment,
            profiles,
            spans: manifest.spans,
            definitions,
        });
        let state = state
            .or(initial_state)
            .unwrap_or_else(|| content.default_state.clone());
        let result = Self { content, state };
        result.validate_content_state()?;
        let missing: BTreeSet<_> = result
            .state
            .items
            .iter()
            .filter(|item| !result.content.definitions.contains_key(&item.definition_id))
            .map(|item| item.definition_id)
            .collect();
        if !missing.is_empty() {
            // (R) Preserve these instances in saved state, but withhold them
            // from every startup inventory/equipment packet until definitions exist.
            eprintln!("inventory: withholding undefined owned item definitions {missing:?}");
        }
        let mut end = 13;
        let mut kinds = BTreeSet::new();
        for span in &result.content.spans {
            let expected = span_bytes(&result.content.default_state, &span.kind, span.profile_id)?;
            if span.offset < end
                || span.offset > 4 * 1024 * 1024
                || span.length != expected.len()
                || hash(&expected) != span.sha256
                || !kinds.insert((span.kind.clone(), span.profile_id))
            {
                return Err(invalid("invalid proven inventory span"));
            }
            end = span
                .offset
                .checked_add(span.length)
                .ok_or_else(|| invalid("inventory span overflow"))?;
        }
        if !kinds.contains(&("coins".into(), 0))
            || !kinds.contains(&("items".into(), 0))
            || result
                .content
                .profiles
                .keys()
                .any(|id| !kinds.contains(&("equipment".into(), *id)))
        {
            return Err(invalid("missing proven inventory span"));
        }
        Ok(result)
    }

    pub fn state(&self) -> &InventoryState {
        &self.state
    }

    /// Whether the loaded catalogue defines this item id. Read-only helper for
    /// content bindings that must not reference unknown reward definitions.
    pub fn has_definition(&self, definition_id: i32) -> bool {
        self.content.definitions.contains_key(&definition_id)
    }

    pub fn presentation(&self, definition_id: i32) -> io::Result<ItemPresentation> {
        let definition = self
            .content
            .definitions
            .get(&definition_id)
            .ok_or_else(|| invalid("unknown item definition"))?;
        let bytes = &definition.bytes;
        // Eleven words, two flags, model string, texture string, GenderUsage,
        // then Type. String lengths count UTF-8 bytes, not characters.
        let mut offset = 46usize;
        for _ in 0..2 {
            let length = usize::try_from(read(bytes, offset)?)
                .map_err(|_| invalid("negative item string length"))?;
            offset = offset
                .checked_add(4)
                .and_then(|n| n.checked_add(length))
                .filter(|n| *n <= bytes.len())
                .ok_or_else(|| invalid("truncated item definition string"))?;
        }
        Ok(ItemPresentation {
            name_id: read(bytes, 4)?,
            icon_id: read(bytes, 12)?,
            icon_tint: read(bytes, 16)?,
            item_type: read(bytes, offset + 4)?,
        })
    }

    pub fn profile_equipment(&self, profile_id: i32) -> Vec<u8> {
        let visible = self.client_visible_state();
        let entries: Vec<_> = visible
            .equipment
            .iter()
            .filter(|e| e.profile_id == profile_id)
            .collect();
        let mut bytes = (entries.len() as i32).to_le_bytes().to_vec();
        for entry in entries {
            bytes.extend(entry.slot.to_le_bytes());
            bytes.extend(entry.item_id.to_le_bytes());
            bytes.extend(entry.category.to_le_bytes());
        }
        bytes
    }

    pub fn definitions_packet(&self) -> Vec<u8> {
        let definitions: Vec<_> = self
            .content
            .definitions
            .values()
            .map(|definition| definition.bytes.clone())
            .collect();
        let visible_owned_ids: Vec<_> = self
            .state
            .items
            .iter()
            .filter(|item| self.content.definitions.contains_key(&item.definition_id))
            .map(|item| item.definition_id)
            .collect();
        // `client_visible_state` applies the fail-closed (R) policy to the
        // corresponding self/equipment packets; this verifies their coverage.
        v2_fixes::encode_definition_batch_v2(&definitions, &visible_owned_ids)
            .expect("visible owned definitions are present in the startup catalogue")
    }

    /// Native CharacterAttachmentData list for a complete class outfit.
    pub fn profile_attachments(&self, profile_id: i32) -> io::Result<Vec<u8>> {
        let visible = self.client_visible_state();
        let mut attachments = Vec::new();
        let mut count = 0i32;
        for entry in visible
            .equipment
            .iter()
            .filter(|e| e.profile_id == profile_id && e.item_id != 0)
        {
            let item = self
                .state
                .items
                .iter()
                .find(|i| i.id == entry.item_id)
                .ok_or_else(|| invalid("equipment item is missing"))?;
            let definition = self
                .content
                .definitions
                .get(&item.definition_id)
                .ok_or_else(|| invalid("equipment definition is missing"))?;
            write_attachment(&mut attachments, definition, item.tint, entry.slot);
            count += 1;
        }
        let mut bytes = count.to_le_bytes().to_vec();
        bytes.extend(attachments);
        Ok(bytes)
    }

    pub fn equip_owned_definition(
        &mut self,
        definition_id: i32,
        profile_id: i32,
    ) -> io::Result<InventoryOutcome> {
        let definition = self
            .content
            .definitions
            .get(&definition_id)
            .ok_or_else(|| invalid("unknown starter definition"))?;
        let slot = definition.slot;
        let item = self
            .state
            .items
            .iter()
            .find(|i| i.definition_id == definition_id)
            .ok_or_else(|| invalid("starter item is not owned"))?;
        self.equip(item.id, profile_id, slot)
    }

    pub fn validate_state(state: &InventoryState) -> io::Result<()> {
        if state.coins > i32::MAX as u32
            || state.items.len() > MAX_ITEMS
            || state.equipment.len() > 1024
        {
            return Err(invalid("inventory state exceeds bounds"));
        }
        let mut ids = BTreeSet::new();
        for item in &state.items {
            if item.id <= 0
                || item.definition_id <= 0
                || item.tint < 0
                || item.count == 0
                || item.count > i32::MAX as u32
                || item.consumed_count > item.count
                || item.stored_count > i32::MAX as u32
                || item.ability_count < 0
                || !ids.insert(item.id)
                || item
                    .rental
                    .as_ref()
                    .is_some_and(|r| r.expiration < 0 || r.upsells < 0 || r.bundle_id < 0)
            {
                return Err(invalid("invalid inventory item"));
            }
        }
        let mut slots = BTreeSet::new();
        if state.next_instance_id.is_some_and(|next| {
            next <= i64::from(ids.last().copied().unwrap_or(0))
                || next > i64::from(i32::MAX) + 1
                || next < 1
        }) {
            return Err(invalid("invalid inventory instance counter"));
        }
        for equipment in &state.equipment {
            if equipment.profile_id <= 0
                || equipment.slot < 0
                || equipment.category < 0
                || equipment.item_id < 0
                || (equipment.item_id != 0 && !ids.contains(&equipment.item_id))
                || !slots.insert((equipment.profile_id, equipment.slot))
            {
                return Err(invalid("invalid inventory equipment"));
            }
        }
        Ok(())
    }

    fn validate_content_state(&self) -> io::Result<()> {
        Self::validate_state(&self.state)?;
        let expected: BTreeSet<_> = self
            .content
            .supported_equipment
            .iter()
            .map(|e| (e.profile_id, e.slot))
            .collect();
        let actual: BTreeSet<_> = self
            .state
            .equipment
            .iter()
            .map(|e| (e.profile_id, e.slot))
            .collect();
        if expected != actual {
            return Err(invalid(
                "saved equipment layout differs from supported profiles",
            ));
        }
        for item in &self.state.items {
            if !self.content.definitions.contains_key(&item.definition_id)
                && !self.content.default_state.items.iter().any(|initial| {
                    initial.id == item.id
                        && initial.definition_id == item.definition_id
                        && initial.tint == item.tint
                })
            {
                return Err(invalid("saved inventory has an unknown definition"));
            }
        }
        for equipment in &self.state.equipment {
            let initial = self
                .content
                .supported_equipment
                .iter()
                .find(|entry| {
                    entry.profile_id == equipment.profile_id && entry.slot == equipment.slot
                })
                .ok_or_else(|| invalid("unknown saved equipment slot"))?;
            let profile = self
                .content
                .profiles
                .get(&equipment.profile_id)
                .ok_or_else(|| invalid("unknown saved equipment profile"))?;
            if !profile.slots.contains(&equipment.slot)
                || (equipment.category != equipment.slot && equipment.category != initial.category)
            {
                return Err(invalid("invalid saved equipment layout"));
            }
            if equipment.item_id != 0 {
                let item = self
                    .state
                    .items
                    .iter()
                    .find(|item| item.id == equipment.item_id)
                    .ok_or_else(|| invalid("unowned saved equipment"))?;
                let Some(definition) = self.content.definitions.get(&item.definition_id) else {
                    let is_default_undefined_item =
                        self.content.default_state.items.iter().any(|initial| {
                            initial.id == item.id
                                && initial.definition_id == item.definition_id
                                && initial.tint == item.tint
                        });
                    if is_default_undefined_item {
                        // (R) Keep the saved link; startup packet views clear it.
                        continue;
                    }
                    return Err(invalid("unknown saved equipment definition"));
                };
                if definition.slot != equipment.slot
                    || !profile.allowed_classes.contains(&definition.class_id)
                {
                    return Err(invalid("saved equipment does not match profile or slot"));
                }
            }
        }
        Ok(())
    }

    /// Startup packet view. Reconstructed policy (R): retain unavailable rows
    /// in persisted state while omitting their instances and equipment links.
    fn client_visible_state(&self) -> InventoryState {
        let mut visible = self.state.clone();
        visible
            .items
            .retain(|item| self.content.definitions.contains_key(&item.definition_id));
        let visible_ids: BTreeSet<_> = visible.items.iter().map(|item| item.id).collect();
        for equipment in &mut visible.equipment {
            if equipment.item_id != 0 && !visible_ids.contains(&equipment.item_id) {
                equipment.item_id = 0;
            }
        }
        visible
    }

    pub fn grant_coins(&mut self, amount: u32) -> io::Result<InventoryOutcome> {
        let coins = self
            .state
            .coins
            .checked_add(amount)
            .filter(|coins| *coins <= i32::MAX as u32)
            .ok_or_else(|| invalid("coin balance overflow"))?;
        if amount == 0 {
            return Ok(InventoryOutcome::default());
        }
        self.state.coins = coins;
        let mut reply = header(38, 19);
        put(&mut reply, coins as i32);
        Ok(InventoryOutcome {
            replies: vec![reply],
            changed: true,
        })
    }

    pub fn grant(
        &mut self,
        coins: u32,
        definition_id: i32,
        quantity: u32,
    ) -> io::Result<InventoryOutcome> {
        self.grant_with_tint(coins, definition_id, quantity, 0)
    }

    pub fn grant_with_tint(
        &mut self,
        coins: u32,
        definition_id: i32,
        quantity: u32,
        tint: i32,
    ) -> io::Result<InventoryOutcome> {
        if tint < 0 {
            return Err(invalid("invalid item tint"));
        }
        if quantity == 0 || quantity > i32::MAX as u32 {
            return Err(invalid("invalid reward quantity"));
        }
        let mut staged = self.clone();
        let mut outcome = staged.grant_coins(coins)?;
        let definition = staged
            .content
            .definitions
            .get(&definition_id)
            .ok_or_else(|| invalid("unknown reward definition"))?;
        let fits = |count: u32| {
            count <= i32::MAX as u32
                && (definition.max_stack <= 0 || count <= definition.max_stack as u32)
        };
        if !fits(quantity) {
            return Err(invalid("reward exceeds maximum stack"));
        }
        if let Some(item) = staged.state.items.iter_mut().find(|item| {
            item.definition_id == definition_id
                && item.tint == tint
                && item.rental.is_none()
                && item.count.checked_add(quantity).is_some_and(fits)
        }) {
            item.count += quantity;
            outcome.replies.push(item_update(item));
        } else {
            if staged.state.items.len() >= MAX_ITEMS {
                return Err(invalid("inventory is full"));
            }
            let live_ids: Vec<_> = staged.state.items.iter().map(|item| item.id).collect();
            let (id, next) =
                instance_ids::allocate_from_snapshot(staged.state.next_instance_id, &live_ids)
                    .map_err(|error| invalid(format!("item ID allocation: {error:?}")))?;
            staged.state.next_instance_id = Some(next);
            let item = InventoryItem {
                definition_id,
                tint,
                id,
                count: quantity,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: i64::from(i32::MAX),
                rental: None,
                ability_count: 0,
                activate_enabled: false,
            };
            let mut payload = Vec::new();
            write_item(&mut payload, &item)?;
            payload.extend_from_slice(&definition.bytes);
            let mut reply = header(38, 2);
            put(&mut reply, payload.len() as i32);
            reply.extend(payload);
            outcome.replies.push(reply);
            staged.state.items.push(item);
        }
        outcome.changed = true;
        staged.validate_content_state()?;
        *self = staged;
        Ok(outcome)
    }

    /// Commit an entire reward bundle, including its allocation metadata, or nothing.
    pub fn grant_bundle(
        &mut self,
        coins: u32,
        items: &[(i32, u32)],
    ) -> io::Result<InventoryOutcome> {
        let mut staged = self.clone();
        let mut outcome = staged.grant_coins(coins)?;
        for &(definition, quantity) in items {
            let item = staged.grant(0, definition, quantity)?;
            outcome.changed |= item.changed;
            outcome.replies.extend(item.replies);
        }
        *self = staged;
        Ok(outcome)
    }

    /// Server-authorized consumption; no unverified client delete opcode is invented.
    pub fn consume(&mut self, item_id: i32, quantity: u32) -> io::Result<InventoryOutcome> {
        let mut staged = self.clone();
        let index = staged
            .state
            .items
            .iter()
            .position(|item| item.id == item_id)
            .ok_or_else(|| invalid("unknown item"))?;
        if quantity == 0 || quantity > staged.state.items[index].count {
            return Err(invalid("invalid consumed quantity"));
        }
        let mut outcome = InventoryOutcome {
            replies: Vec::new(),
            changed: true,
        };
        if staged.state.items[index].count == quantity {
            for equipped in staged
                .state
                .equipment
                .iter_mut()
                .filter(|entry| entry.item_id == item_id)
            {
                equipped.item_id = 0;
                let mut reply = header(38, 6);
                put(&mut reply, equipped.slot);
                put(&mut reply, equipped.profile_id);
                outcome.replies.push(reply);
            }
            staged.state.items.remove(index);
            let mut reply = header(38, 4);
            put(&mut reply, item_id);
            outcome.replies.push(reply);
        } else {
            let item = &mut staged.state.items[index];
            item.count -= quantity;
            item.consumed_count = item.consumed_count.min(item.count);
            outcome.replies.push(item_update(item));
        }
        staged.validate_content_state()?;
        *self = staged;
        Ok(outcome)
    }

    pub fn handle(&mut self, inner: &[u8]) -> io::Result<InventoryOutcome> {
        if inner.len() < 4 || inner.len() > 4096 {
            return Err(invalid("invalid inventory message length"));
        }
        let family = u16::from_le_bytes(inner[..2].try_into().unwrap());
        let opcode = u16::from_le_bytes(inner[2..4].try_into().unwrap());
        if family == 35 && opcode == 34 {
            if inner.len() != 12 || read(inner, 8)? != 0 {
                return Err(invalid("invalid item definition request"));
            }
            let id = read(inner, 4)?;
            let Some(definition) = self.content.definitions.get(&id) else {
                return Ok(InventoryOutcome::default());
            };
            let mut reply = header(35, 34);
            put(&mut reply, id);
            put(&mut reply, definition.bytes.len() as i32);
            reply.extend_from_slice(&definition.bytes);
            return Ok(InventoryOutcome {
                replies: vec![reply],
                changed: false,
            });
        }
        if family != 42 {
            return Err(invalid("not an inventory request"));
        }
        match opcode {
            2 => {
                if inner.len() != 12 {
                    return Err(invalid("invalid unequip request"));
                }
                let (slot, profile) = (read(inner, 4)?, read(inner, 8)?);
                let entry = self
                    .state
                    .equipment
                    .iter_mut()
                    .find(|entry| entry.profile_id == profile && entry.slot == slot)
                    .ok_or_else(|| invalid("unknown equipment slot"))?;
                let changed = entry.item_id != 0;
                entry.item_id = 0;
                let mut reply = header(38, 6);
                put(&mut reply, slot);
                put(&mut reply, profile);
                Ok(InventoryOutcome {
                    replies: vec![reply],
                    changed,
                })
            }
            3 => {
                if inner.len() != 16 {
                    return Err(invalid("invalid equip request"));
                }
                self.equip(read(inner, 4)?, read(inner, 8)?, read(inner, 12)?)
            }
            8 => {
                if inner.len() != 20 {
                    return Err(invalid("invalid item-record equip request"));
                }
                let (definition, tint) = (read(inner, 4)?, read(inner, 8)?);
                let id = self
                    .state
                    .items
                    .iter()
                    .find(|item| item.definition_id == definition && item.tint == tint)
                    .ok_or_else(|| invalid("unowned item record"))?
                    .id;
                self.equip(id, read(inner, 12)?, read(inner, 16)?)
            }
            // Style-card/customization, toolbar and other actions remain outside
            // this layer. They must not silently consume or delete an item.
            _ => Ok(InventoryOutcome::default()),
        }
    }

    fn equip(&mut self, id: i32, profile_id: i32, slot: i32) -> io::Result<InventoryOutcome> {
        let item = self
            .state
            .items
            .iter()
            .find(|item| item.id == id)
            .ok_or_else(|| invalid("unowned inventory item"))?;
        let definition = self
            .content
            .definitions
            .get(&item.definition_id)
            .ok_or_else(|| invalid("unknown equipment definition"))?;
        let profile = self
            .content
            .profiles
            .get(&profile_id)
            .ok_or_else(|| invalid("unknown profile"))?;
        if definition.slot != slot
            || !profile.slots.contains(&slot)
            || !profile.allowed_classes.contains(&definition.class_id)
        {
            return Err(invalid("equipment does not match profile or slot"));
        }
        let entry = self
            .state
            .equipment
            .iter_mut()
            .find(|entry| entry.profile_id == profile_id && entry.slot == slot)
            .ok_or_else(|| invalid("unknown equipment slot"))?;
        let changed = entry.item_id != id;
        let mut reply = header(38, 5);
        put(&mut reply, id);
        write_attachment(&mut reply, definition, item.tint, slot);
        put(&mut reply, profile_id);
        put(&mut reply, definition.class_id);
        reply.push(1);
        let mut visible = header(35, 6);
        visible.extend(self.content.player_guid.to_le_bytes());
        put(&mut visible, id);
        write_attachment(&mut visible, definition, item.tint, slot);
        put(&mut visible, profile_id);
        put(&mut visible, 0);
        put(&mut visible, definition.wield_type);
        entry.item_id = id;
        entry.category = slot;
        Ok(InventoryOutcome {
            replies: vec![reply, visible],
            changed,
        })
    }

    /// GenderFilter immediately follows the verified item list in ClientPcData.
    /// Apply before any variable-length inventory or appearance splice.
    pub fn patch_gender(&self, template: &[u8], gender: i32) -> io::Result<Vec<u8>> {
        if !matches!(gender, 1 | 2) {
            return Err(invalid("invalid character gender"));
        }
        let span = self
            .content
            .spans
            .iter()
            .find(|s| s.kind == "items")
            .ok_or_else(|| invalid("missing items proof"))?;
        let end = span
            .offset
            .checked_add(span.length)
            .ok_or_else(|| invalid("items range overflow"))?;
        if template.get(span.offset..end).map(hash).as_deref() != Some(span.sha256.as_str())
            || !matches!(read(template, end)?, 1 | 2)
        {
            return Err(invalid(
                "self gender field differs from verified item boundary",
            ));
        }
        let mut result = template.to_vec();
        result[end..end + 4].copy_from_slice(&gender.to_le_bytes());
        Ok(result)
    }

    /// Patch only proven original spans; unrelated position/quest patches survive.
    pub fn initialization_patch(&self, self_template: &[u8]) -> io::Result<Vec<u8>> {
        if self_template.len() < 13
            || self_template.len() > MAX_CONTENT
            || self_template[..3] != [5, 0, 1]
            || self_template[7..9] != [12, 0]
            || u32::from_le_bytes(self_template[3..7].try_into().unwrap()) as usize
                != self_template.len() - 7
            || u32::from_le_bytes(self_template[9..13].try_into().unwrap()) as usize
                != self_template.len() - 13
        {
            return Err(invalid("invalid self template envelope"));
        }
        let mut result = self_template.to_vec();
        let visible = self.client_visible_state();
        for span in self.content.spans.iter().rev() {
            let end = span
                .offset
                .checked_add(span.length)
                .ok_or_else(|| invalid("inventory span overflow"))?;
            let bytes = result
                .get(span.offset..end)
                .ok_or_else(|| invalid("truncated inventory span"))?;
            if hash(bytes) != span.sha256 {
                return Err(invalid("self template inventory span differs from proof"));
            }
            let replacement = span_bytes(&visible, &span.kind, span.profile_id)?;
            result.splice(span.offset..end, replacement);
            if result.len() > MAX_CONTENT {
                return Err(invalid("patched self exceeds bounds"));
            }
        }
        let tunnel_length = (result.len() - 7) as u32;
        let payload_length = (result.len() - 13) as u32;
        result[3..7].copy_from_slice(&tunnel_length.to_le_bytes());
        result[9..13].copy_from_slice(&payload_length.to_le_bytes());
        Ok(result)
    }
}

fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn put(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}
fn header(family: u16, opcode: u16) -> Vec<u8> {
    [family.to_le_bytes(), opcode.to_le_bytes()].concat()
}
fn read(bytes: &[u8], offset: usize) -> io::Result<i32> {
    Ok(i32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(|| invalid("truncated inventory field"))?
            .try_into()
            .unwrap(),
    ))
}
fn write_text(bytes: &mut Vec<u8>, value: &str) {
    put(bytes, value.len() as i32);
    bytes.extend(value.as_bytes());
}
fn decode_hex(value: &str) -> io::Result<Vec<u8>> {
    if value.len() % 2 != 0 || value.len() > 131_072 {
        return Err(invalid("invalid definition hex size"));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |c: u8| match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                _ => None,
            };
            Ok(
                digit(pair[0]).ok_or_else(|| invalid("invalid definition hex"))? * 16
                    + digit(pair[1]).ok_or_else(|| invalid("invalid definition hex"))?,
            )
        })
        .collect()
}
fn write_item(bytes: &mut Vec<u8>, item: &InventoryItem) -> io::Result<()> {
    // Existing self-body serialization carries the relative i32 input consumed
    // by the native reader; 38/3 uses the separate 64-bit state value below.
    let self_last_cast = i32::try_from(item.last_cast_time)
        .map_err(|_| invalid("self-body last-cast value exceeds its i32 field"))?;
    for value in [
        item.definition_id,
        item.tint,
        item.id,
        item.count as i32,
        item.consumed_count as i32,
        self_last_cast,
    ] {
        put(bytes, value);
    }
    bytes.push(u8::from(item.rental.is_some()));
    if let Some(rental) = &item.rental {
        bytes.extend(rental.expiration.to_le_bytes());
        put(bytes, rental.upsells);
        put(bytes, rental.bundle_id);
    }
    put(bytes, item.ability_count);
    bytes.push(u8::from(item.activate_enabled));
    Ok(())
}
fn span_bytes(state: &InventoryState, kind: &str, profile_id: i32) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    match kind {
        "coins" if profile_id == 0 => put(&mut bytes, state.coins as i32),
        "items" if profile_id == 0 => {
            put(&mut bytes, state.items.len() as i32);
            for item in &state.items {
                write_item(&mut bytes, item)?;
            }
        }
        "equipment" if profile_id > 0 => {
            let entries: Vec<_> = state
                .equipment
                .iter()
                .filter(|entry| entry.profile_id == profile_id)
                .collect();
            put(&mut bytes, entries.len() as i32);
            for entry in entries {
                put(&mut bytes, entry.slot);
                put(&mut bytes, entry.item_id);
                put(&mut bytes, entry.category);
            }
        }
        _ => return Err(invalid("unknown inventory span kind")),
    }
    Ok(bytes)
}
fn item_update(item: &InventoryItem) -> Vec<u8> {
    v2_fixes::encode_item_update_v2(v2_fixes::ItemUpdateV2 {
        item_id: item.id,
        count: item.count as i32,
        item_bar_slot: -1,
        consumed_count: item.consumed_count as i32,
        stored_count: item.stored_count as i32,
        last_cast_time: item.last_cast_time,
    })
}
fn write_attachment(bytes: &mut Vec<u8>, definition: &Definition, tint: i32, slot: i32) {
    for value in [
        &definition.model,
        &definition.texture,
        &definition.tint_alias,
    ] {
        write_text(bytes, value);
    }
    for value in [
        if tint == 0 {
            definition.default_tint
        } else {
            tint
        },
        definition.effect,
        slot,
    ] {
        put(bytes, value);
    }
}
