//! Typed local inventory state and evidenced inventory packet shapes.
//! Private content is prepared from supplied JSON and byte-checked captures.
//!
//! W88 integration: place W84's catalogue.json at CONTENT/item-catalogue.json.
//! Inventory::load merges it automatically; inventory.json rows always win.
//! The existing 35/34 Inventory::handle route uses definition_replies(inner).
//! A lead adding a separate route should forward
//! `out.replies.extend(self.inventory.definition_replies(inner)?);` and leave
//! inventory/save state unchanged. Apply the session's packet/rate budget first.
//! No bulk-ID wire format is inferred: W87 proves one ID per 12-byte request.
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[path = "inventory_v2_fixes.rs"]
mod v2_fixes;

/// Occupied stacks one player's bag may hold. `validate_state` rejects a save
/// above it and `grant` refuses a new stack at it.
pub const MAX_ITEMS: usize = 4096;
const MAX_CONTENT: usize = 4 * 1024 * 1024;
/// (R) Encounter content bounds a reward bundle by rows: dungeon `reward_items`
/// at 16 (`src/encounter.rs:1011`) and field-battle `prizes` at 4
/// (`src/encounter/field.rs:308`). `grant_bundle` commits no row cap of its own,
/// so the preflight keeps the wider loader bound as the one deliberate limit.
const MAX_REWARD_BUNDLE: usize = 16;
// Reconstructed resource limits (R), separate from the player-owned item cap.
const MAX_CATALOGUE_FILE: u64 = 128 * 1024 * 1024;
const MAX_CATALOGUE_ROWS: usize = 65_536;
const MAX_CATALOGUE_BYTES: usize = 32 * 1024 * 1024;

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

/// Equip restrictions read from a definition's wire row. Recovered layout:
/// gender usage `+0x68`, members-only `+0x74`, minimum profile rank `+0x84`
/// (Items and Inventory guide, definition table; `00baec20`, `00a142c0`).
/// Checked against the 58,835-row generated catalogue (W138).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Requirement {
    /// 0 any, 1 boys, 2 girls.
    gender: i32,
    members_only: bool,
    min_rank: i32,
}

impl Requirement {
    /// Skips the fixed 46-byte prefix and the two length-prefixed strings.
    /// A row too short to carry the block (test stubs) has no restriction.
    fn from_wire(bytes: &[u8]) -> Self {
        let mut at = 46;
        for _ in 0..2 {
            let Some(len) = read(bytes, at).ok().and_then(|n| usize::try_from(n).ok()) else {
                return Self::default();
            };
            at = at.saturating_add(4).saturating_add(len);
        }
        match (read(bytes, at), bytes.get(at + 12), read(bytes, at + 26)) {
            (Ok(gender), Some(members), Ok(min_rank)) => Self {
                gender,
                members_only: *members != 0,
                min_rank,
            },
            _ => Self::default(),
        }
    }
}

/// Minimum profile rank of an equipment slot: Star rune 10, Moon rune 20
/// (`EquipmentSlotDefinitions.txt` MIN_PROFILE_RANK; no slot is members-only).
fn slot_min_rank(slot: i32) -> i32 {
    match slot {
        14 => 10,
        15 => 20,
        _ => 0,
    }
}

/// Slots the client's `ProcessProfileSwap` (`00bfb620`) requires before it
/// applies a job outfit. The shipped slot table marks Chest and Legs required;
/// see `notes/Build Guides/Gameplay/Character and Customization.md`.
pub const REQUIRED_PROFILE_SLOTS: [i32; 2] = [3, 4];

/// The player facts equip eligibility needs, owned by the gameplay layer.
/// (R) Server-side enforcement: the client only greys out and ignores some
/// requests, and the original server rule was not recovered.
#[derive(Clone, Debug, Default)]
pub struct EquipContext {
    /// Character gender, 1 boy or 2 girl; 0 unknown (no identity yet), not checked.
    pub gender: i32,
    /// Membership active now.
    pub member: bool,
    /// Rank of every unlocked profile, keyed by profile id. A profile that is
    /// absent is not owned: equip and unequip on it are refused.
    pub profile_ranks: BTreeMap<i32, i32>,
    /// "Now" in the unit of `RentalState::expiration` (R: the unit is not
    /// proven). `None` skips the rental expiry check.
    pub rental_now: Option<i64>,
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
    #[serde(skip)]
    requirement: Requirement,
}

impl Definition {
    /// Whether `count` of this definition fits one inventory stack. The wire
    /// count is a signed i32; a non-positive `max_stack` means uncapped.
    fn fits_one_stack(&self, count: u32) -> bool {
        count <= i32::MAX as u32 && (self.max_stack <= 0 || count <= self.max_stack as u32)
    }

    fn decode(mut self) -> io::Result<Self> {
        self.bytes = decode_hex(&self.serialized_hex)?;
        if self.bytes.len() < 4
            || self.bytes.len() > 65_536
            || self.bytes[..4] != self.id.to_le_bytes()
            || hash(&self.bytes) != self.sha256
            || self.id <= 0
            || [self.model.len(), self.texture.len(), self.tint_alias.len()]
                .iter()
                .any(|size| *size > 4096)
        {
            return Err(invalid("invalid inventory definition content"));
        }
        self.requirement = Requirement::from_wire(&self.bytes);
        // Keep one wire copy, not its hex and hash as well, for all 58k rows.
        self.serialized_hex = String::new();
        self.sha256 = String::new();
        Ok(self)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalogue {
    schema_version: u32,
    #[serde(deserialize_with = "catalogue_definitions")]
    definitions: BTreeMap<i32, Definition>,
}

fn catalogue_definitions<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<i32, Definition>, D::Error> {
    // Stream rows through serde's parser: never hold a JSON DOM or a second
    // vector of all decoded definitions. Source/default tags stay in
    // W84's private file; generated server values remain reconstructed (R).
    struct Rows;
    impl<'de> serde::de::Visitor<'de> for Rows {
        type Value = BTreeMap<i32, Definition>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a bounded item definition array")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut rows: A,
        ) -> Result<Self::Value, A::Error> {
            use serde::de::Error;
            let mut definitions = BTreeMap::new();
            let mut total = 0usize;
            while let Some(definition) = rows.next_element::<Definition>()? {
                if definitions.len() >= MAX_CATALOGUE_ROWS {
                    return Err(A::Error::custom("item catalogue row limit exceeded"));
                }
                let definition = definition.decode().map_err(A::Error::custom)?;
                total += definition.bytes.len(); // bounded row and row count
                if total > MAX_CATALOGUE_BYTES {
                    return Err(A::Error::custom("item catalogue byte limit exceeded"));
                }
                if definitions.insert(definition.id, definition).is_some() {
                    return Err(A::Error::custom("duplicate item catalogue definition"));
                }
            }
            Ok(definitions)
        }
    }
    deserializer.deserialize_seq(Rows)
}

fn load_catalogue(file: File) -> io::Result<BTreeMap<i32, Definition>> {
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_CATALOGUE_FILE {
        return Err(invalid(
            "item catalogue must be a regular file of at most 128 MiB",
        ));
    }
    // One bounded temporary source buffer avoids serde's per-byte Read overhead.
    // Pre-size from metadata so the normal 68 MiB input does not double capacity.
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_CATALOGUE_FILE + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CATALOGUE_FILE {
        return Err(invalid("item catalogue exceeds 128 MiB"));
    }
    let catalogue: Catalogue = serde_json::from_slice(&bytes).map_err(invalid)?;
    if catalogue.schema_version != 1 {
        return Err(invalid("unsupported item catalogue"));
    }
    Ok(catalogue.definitions)
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
    captured_definition_ids: BTreeSet<i32>,
    initial_state: Option<InventoryState>,
}

#[derive(Clone)]
pub struct Inventory {
    content: Arc<Content>,
    state: InventoryState,
}

/// Parsed content is shared by every session and Coin Store transaction that
/// loads the same files: parsing the generated catalogue (~58k rows) per login
/// or per purchase cost seconds and ~50 MiB each. The key includes each file's
/// size and modification time, so edited content is parsed again.
fn shared_content(directory: &Path) -> io::Result<Arc<Content>> {
    type Key = (PathBuf, Option<[u8; 32]>, Option<(u64, SystemTime)>);
    static CACHE: OnceLock<Mutex<HashMap<Key, Arc<Content>>>> = OnceLock::new();
    // inventory.json (~2 MB) is keyed by its content: a same-size rewrite inside
    // one timestamp tick must not return the old definitions. The ~50 MB
    // catalogue keeps the cheap size + time stamp (R: it is generated, not edited).
    const MAX_HASHED: u64 = 64 * 1024 * 1024;
    let inventory = directory.join("inventory.json");
    let digest = fs::metadata(&inventory)
        .ok()
        .filter(|m| m.len() <= MAX_HASHED)
        .and_then(|_| fs::read(&inventory).ok())
        .map(|bytes| <[u8; 32]>::from(Sha256::digest(&bytes)));
    let stamp = fs::metadata(directory.join("item-catalogue.json"))
        .ok()
        .and_then(|m| Some((m.len(), m.modified().ok()?)));
    let key = (directory.to_path_buf(), digest, stamp);
    let cache = CACHE.get_or_init(Default::default);
    if let Some(content) = cache.lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return Ok(content.clone());
    }
    let content = Inventory::parse_content(directory)?;
    let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
    if cache.len() >= 8 {
        cache.clear(); // a handful of content folders per process; never unbounded
    }
    cache.insert(key, content.clone());
    Ok(content)
}

impl Inventory {
    pub fn load(directory: &Path, state: Option<InventoryState>) -> io::Result<Self> {
        let content = shared_content(directory)?;
        let state = state
            .or_else(|| content.initial_state.clone())
            .unwrap_or_else(|| content.default_state.clone());
        let result = Self { content, state };
        result.validate_content_state()?;
        result.warn_undefined_items();
        Ok(result)
    }

    fn parse_content(directory: &Path) -> io::Result<Arc<Content>> {
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
        for definition in manifest.definitions {
            let definition = definition.decode()?;
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
        let captured_definition_ids = definitions.keys().copied().collect();
        // Fixed operator-selected content path; remote item IDs never become paths.
        match File::open(directory.join("item-catalogue.json")) {
            Ok(file) => {
                for (id, definition) in load_catalogue(file)? {
                    definitions.entry(id).or_insert(definition);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
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
            captured_definition_ids,
            initial_state,
        });
        let mut end = 13;
        let mut kinds = BTreeSet::new();
        for span in &content.spans {
            let expected = span_bytes(&content.default_state, &span.kind, span.profile_id)?;
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
            || content
                .profiles
                .keys()
                .any(|id| !kinds.contains(&("equipment".into(), *id)))
        {
            return Err(invalid("missing proven inventory span"));
        }
        Ok(content)
    }

    fn warn_undefined_items(&self) {
        let result = self;
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
    }

    pub fn state(&self) -> &InventoryState {
        &self.state
    }

    /// Whether the loaded catalogue defines this item id. Read-only helper for
    /// content bindings that must not reference unknown reward definitions.
    pub fn has_definition(&self, definition_id: i32) -> bool {
        self.content.definitions.contains_key(&definition_id)
    }

    /// Every item definition id the loaded catalogue defines, ascending. A
    /// content check can pick a real row without re-parsing the catalogue.
    pub fn definition_ids(&self) -> impl Iterator<Item = i32> + '_ {
        self.content.definitions.keys().copied()
    }

    /// Whether this catalogue can grant `quantity` of `definition_id` as one
    /// stack, using the same bound as `grant_with_tint`. A content preflight
    /// uses this to refuse an unpayable reward before entry or a payout. It
    /// checks one row at a time; the whole-bundle preflight goes through
    /// `accepts_reward_bundle`.
    pub fn accepts_reward(&self, definition_id: i32, quantity: u32) -> bool {
        quantity > 0
            && self
                .content
                .definitions
                .get(&definition_id)
                .is_some_and(|definition| definition.fits_one_stack(quantity))
    }

    /// Whether one clone of this bag could grant every `(definition_id,
    /// quantity)` row in order, through `grant_bundle`, the same path a real
    /// payout uses. This is the **bundle** preflight: `accepts_reward` checks
    /// one row's catalogue and stack bounds, but two rows that each pass can
    /// still need a second stack when the bag is full, or overflow `MAX_ITEMS`
    /// together.
    ///
    /// State-only and read-only for the live bag: the clone and its reply
    /// packets are dropped. An empty list fits. A row `grant_bundle` refuses,
    /// or a list longer than `MAX_REWARD_BUNDLE`, is `false`, never a panic.
    pub fn accepts_reward_bundle(&self, items: &[(i32, u32)]) -> bool {
        items.len() <= MAX_REWARD_BUNDLE && self.clone().grant_bundle(0, items).is_ok()
    }

    /// Whether the login 35/37 catalogue already carried this row to the client.
    pub fn definition_was_captured(&self, definition_id: i32) -> bool {
        self.content
            .captured_definition_ids
            .contains(&definition_id)
    }

    /// The serialized `ClientItemDefinition` row, the same bytes a 35/37 batch carries.
    pub fn definition_record(&self, definition_id: i32) -> Option<&[u8]> {
        self.content
            .definitions
            .get(&definition_id)
            .map(|definition| definition.bytes.as_slice())
    }

    /// What a trade needs from a definition (W163), as `(no_trade, max_stack)`. NO_TRADE is the
    /// byte after the eleven leading words of the serialized row (`piece_096`
    /// `ItemDefinition.no_trade`, inside the 46-byte prefix `Requirement::from_wire` skips) and
    /// a max stack of zero or less means uncapped (R). A row too short to carry the flag has no
    /// entry, so it cannot be offered.
    pub fn trade_flags(&self, definition_id: i32) -> Option<(bool, Option<u32>)> {
        let definition = self.content.definitions.get(&definition_id)?;
        let max_stack = u32::try_from(definition.max_stack)
            .ok()
            .filter(|max| *max > 0);
        Some((*definition.bytes.get(44)? != 0, max_stack))
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
        let visible_owned_ids: Vec<_> = self
            .state
            .items
            .iter()
            .filter(|item| self.content.definitions.contains_key(&item.definition_id))
            .map(|item| item.definition_id)
            .collect();
        // (R) Send captured rows plus owned additions, not the full 58k map.
        let mut ids = self.content.captured_definition_ids.clone();
        ids.extend(visible_owned_ids.iter().copied());
        let definitions: Vec<_> = ids
            .iter()
            .filter_map(|id| self.content.definitions.get(id))
            .map(|definition| definition.bytes.clone())
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

    /// Whether this profile has every slot `ProcessProfileSwap` requires.
    /// Uses the same visible equipment rows that `profile_attachments` sends.
    pub fn has_required_profile_slots(&self, profile_id: i32) -> bool {
        let visible = self.client_visible_state();
        REQUIRED_PROFILE_SLOTS.into_iter().all(|slot| {
            visible.equipment.iter().any(|entry| {
                entry.profile_id == profile_id && entry.slot == slot && entry.item_id != 0
            })
        })
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
        self.equip(item.id, profile_id, slot, None)
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

    /// Replace the whole state with `after`, a settled trade's validated state (W163, lead),
    /// and return the client updates that turn the current view into it: removed stacks
    /// (38/4), new stacks with their definitions (38/2), changed counts (item update) and the
    /// coin balance (38/19). Nothing changes if `after` does not validate.
    pub fn adopt_state(&mut self, after: InventoryState) -> io::Result<Vec<Vec<u8>>> {
        let mut staged = self.clone();
        staged.state = after;
        staged.validate_content_state()?;
        let mut replies = Vec::new();
        for old in &self.state.items {
            if !staged.state.items.iter().any(|item| item.id == old.id) {
                let mut reply = header(38, 4);
                put(&mut reply, old.id);
                replies.push(reply);
            }
        }
        for item in &staged.state.items {
            match self.state.items.iter().find(|old| old.id == item.id) {
                Some(old) if old == item => {}
                Some(_) => replies.push(item_update(item)),
                None => {
                    let definition = staged
                        .content
                        .definitions
                        .get(&item.definition_id)
                        .ok_or_else(|| invalid("unknown traded definition"))?;
                    if !staged
                        .content
                        .captured_definition_ids
                        .contains(&item.definition_id)
                    {
                        replies.push(
                            v2_fixes::encode_definition_batch_v2(
                                &[definition.bytes.clone()],
                                &[item.definition_id],
                            )
                            .map_err(invalid)?,
                        );
                    }
                    let mut payload = Vec::new();
                    write_item(&mut payload, item)?;
                    payload.extend_from_slice(&definition.bytes);
                    let mut reply = header(38, 2);
                    put(&mut reply, payload.len() as i32);
                    reply.extend(payload);
                    replies.push(reply);
                }
            }
        }
        if staged.state.coins != self.state.coins {
            let mut reply = header(38, 19);
            put(&mut reply, staged.state.coins as i32);
            replies.push(reply);
        }
        *self = staged;
        Ok(replies)
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
        if !definition.fits_one_stack(quantity) {
            return Err(invalid("reward exceeds maximum stack"));
        }
        if !staged
            .content
            .captured_definition_ids
            .contains(&definition_id)
        {
            // (R) Re-send this small batch even for a stack update. No per-client
            // cache bookkeeping: the definition always precedes its inventory use.
            outcome.replies.push(
                v2_fixes::encode_definition_batch_v2(&[definition.bytes.clone()], &[definition_id])
                    .map_err(invalid)?,
            );
        }
        if let Some(item) = staged.state.items.iter_mut().find(|item| {
            item.definition_id == definition_id
                && item.tint == tint
                && item.rental.is_none()
                && item
                    .count
                    .checked_add(quantity)
                    .is_some_and(|count| definition.fits_one_stack(count))
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

    /// Pure lookup/encoding: no I/O or mutation. W87's 00a3da40 writer and
    /// 008d7980 reader carry family, subtype, ID, blob length and blob.
    /// (R) Cap at one ID, require an empty request blob and ignore unknown IDs.
    /// Replies are bounded by the validated 65,536-byte row limit plus header.
    pub fn definition_replies(&self, inner: &[u8]) -> io::Result<Vec<Vec<u8>>> {
        if inner.len() != 12 || !inner.starts_with(&[35, 0, 34, 0]) || read(inner, 8)? != 0 {
            return Err(invalid("invalid item definition request"));
        }
        let id = read(inner, 4)?;
        if id <= 0 {
            return Err(invalid("invalid item definition ID"));
        }
        let Some(definition) = self.content.definitions.get(&id) else {
            return Ok(Vec::new());
        };
        let mut reply = header(35, 34);
        put(&mut reply, id);
        put(&mut reply, definition.bytes.len() as i32);
        reply.extend_from_slice(&definition.bytes);
        Ok(vec![reply])
    }

    /// Equip routing without player facts: ownership, slot and class only.
    /// Callers that know the player should use `handle_with`.
    pub fn handle(&mut self, inner: &[u8]) -> io::Result<InventoryOutcome> {
        self.route(inner, None)
    }

    /// As `handle`, and also refuses equips and unequips the player may not
    /// make (profile not unlocked, members only, gender, rank, expired rental).
    /// A refusal leaves state unchanged and replies with a chat line (R: no
    /// native refusal packet is recovered; the client greys such items out).
    pub fn handle_with(
        &mut self,
        inner: &[u8],
        player: &EquipContext,
    ) -> io::Result<InventoryOutcome> {
        self.route(inner, Some(player))
    }

    fn route(
        &mut self,
        inner: &[u8],
        player: Option<&EquipContext>,
    ) -> io::Result<InventoryOutcome> {
        if inner.len() < 4 || inner.len() > 4096 {
            return Err(invalid("invalid inventory message length"));
        }
        let family = u16::from_le_bytes(inner[..2].try_into().unwrap());
        let opcode = u16::from_le_bytes(inner[2..4].try_into().unwrap());
        if family == 35 && opcode == 34 {
            return Ok(InventoryOutcome {
                replies: self.definition_replies(inner)?,
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
                if player.is_some_and(|p| !p.profile_ranks.contains_key(&profile)) {
                    return Ok(refused("That class is not unlocked."));
                }
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
                self.equip(read(inner, 4)?, read(inner, 8)?, read(inner, 12)?, player)
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
                self.equip(id, read(inner, 12)?, read(inner, 16)?, player)
            }
            // Style-card/customization, toolbar and other actions remain outside
            // this layer. They must not silently consume or delete an item.
            _ => Ok(InventoryOutcome::default()),
        }
    }

    fn equip(
        &mut self,
        id: i32,
        profile_id: i32,
        slot: i32,
        player: Option<&EquipContext>,
    ) -> io::Result<InventoryOutcome> {
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
        if let Some(reason) = player.and_then(|p| ineligible(p, item, definition, profile_id, slot))
        {
            return Ok(refused(&reason));
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

/// A system chat line (family 15/3, the form `chat::system` sends) carrying a
/// fixed, escape-free reason. Built here because `restoration/piece_045.rs`
/// includes this file without the crate's `chat` module; a test pins equality.
fn refused(reason: &str) -> InventoryOutcome {
    let mut line = header(15, 3);
    line.extend((reason.len() as u32).to_le_bytes());
    line.extend(reason.as_bytes());
    line.push(1);
    InventoryOutcome {
        replies: vec![line],
        changed: false,
    }
}
/// Why this player cannot put this item in this slot of this profile, if so.
fn ineligible(
    player: &EquipContext,
    item: &InventoryItem,
    definition: &Definition,
    profile_id: i32,
    slot: i32,
) -> Option<String> {
    let need = &definition.requirement;
    let Some(&rank) = player.profile_ranks.get(&profile_id) else {
        return Some("That class is not unlocked.".into());
    };
    if need.members_only && !player.member {
        return Some("This item is for members only.".into());
    }
    if need.gender != 0 && player.gender != 0 && need.gender != player.gender {
        let who = if need.gender == 1 { "boys" } else { "girls" };
        return Some(format!("This item is only for {who}."));
    }
    let min_rank = need.min_rank.max(slot_min_rank(slot));
    if rank < min_rank {
        return Some(format!("This item needs class rank {min_rank}."));
    }
    let expired = item
        .rental
        .as_ref()
        .zip(player.rental_now)
        .is_some_and(|(rental, now)| rental.expiration <= now);
    expired.then(|| "This rental has expired.".into())
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
