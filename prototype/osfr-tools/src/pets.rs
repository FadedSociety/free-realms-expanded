//! Saved pet roster and bounded client-shaped pet state.
//!
//! Client facts and source tags live in `data/content/pets.json`. Pet spawning,
//! care effects, need scale/rates, and server replies remain reconstructed (R)
//! because no original pet datagram was recovered.

use crate::{
    game_store,
    inventory::{Inventory, InventoryState},
    startup_world, world,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::Path,
    sync::Arc,
};

const PET_FAMILY: u16 = 53;
/// (R) Hard ceiling on a saved roster; the client can display a bounded list.
const MAX_PETS: usize = 50;
const PET_CONTENT: &str = include_str!("../data/content/pets.json");
/// (R) One active pet per owner, so an owner-derived actor id is unique while summoned.
const PET_GUID_TAG: u64 = 0x4000_0000_0000_0000;
/// Client `Models.txt` pet ADR -> numeric model id. The pet content only ships
/// the ADR, so this recovered binding is what lets `AddNpc` name the model.
/// Where a male and female id share an ADR, the lower id is used (R).
const MODEL_IDS: &[(&str, i32)] = &[
    ("bat_m_pinkeye.adr", 4566),
    ("beetle_clockwork.adr", 3885),
    ("dog_large_dalmation.adr", 514),
    ("dog_large_dukey.adr", 3793),
    ("dog_large_floppy.adr", 169),
    ("dog_large_husky.adr", 1454),
    ("dog_large_pointyghost.adr", 1018),
    ("dog_large_stbernard.adr", 4109),
    ("dog_small_fluffy.adr", 364),
    ("dragon_m_basic.adr", 1756),
    ("dragon_m_phoenix.adr", 3358),
    ("duckbill_m.adr", 1921),
    ("groundhog_m.adr", 2036),
    ("groundhog_m_sharkfin.adr", 3030),
    ("hydra_basic.adr", 3405),
    ("monkey_clockwork_m.adr", 939),
    ("pony_pegasus_m_fire.adr", 3432),
    ("reindeer_m.adr", 1934),
    ("robgoblin_minimech.adr", 4068),
    ("rofle_m.adr", 3287),
    ("spider_m.adr", 1924),
    ("tiger_m_lava.adr", 3479),
    ("treeble.adr", 376),
    ("tyrannosaurus_m.adr", 1759),
    ("vulture_pet.adr", 3985),
    ("wolf_coyote.adr", 1521),
];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetContent {
    schema_version: u32,
    #[serde(alias = "definitions")]
    species: Vec<PetSpeciesRecord>,
    #[serde(default)]
    named_pets: Vec<NamedPetRecord>,
    definition_gap: DefinitionGap,
    names: PetNameParts,
    tricks: Vec<PetTrickDefinition>,
    care_items: Vec<PetCareItem>,
    need_policy: NeedPolicy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PetSpeciesRecord {
    #[serde(default)]
    species_id: Option<u32>,
    #[serde(default)]
    name: String,
    #[serde(default)]
    wiki_title: String,
    #[serde(default)]
    client_name_string_id: Option<u32>,
    #[serde(default)]
    client_name_string_id_candidates: Vec<u32>,
    #[serde(default)]
    species_kind: Option<String>,
    #[serde(default)]
    breed: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    model_files: Option<Vec<String>>,
    #[serde(default)]
    texture_alias: Option<String>,
    #[serde(default)]
    icon: Option<PetIcon>,
    #[serde(default)]
    adoption_item_ids: Vec<i32>,
    #[serde(default)]
    member_only: Option<bool>,
    #[serde(default)]
    adoption_source: Vec<PetAdoptionSource>,
    source: Vec<SourceTag>,
    #[serde(default)]
    field_sources: BTreeMap<String, Vec<SourceTag>>,
    #[serde(default)]
    unknown_reasons: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PetIcon {
    image_set_id: i32,
    tint_id: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PetAdoptionSource {
    kind: String,
    #[serde(default)]
    store_item_id: Option<u32>,
    #[serde(default)]
    store_price_coins: Option<u32>,
    #[serde(default)]
    price: Option<u32>,
    #[serde(default)]
    member_price: Option<u32>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    reward: Option<String>,
    #[serde(default)]
    promotion: Option<String>,
    source: Vec<SourceTag>,
    #[serde(default)]
    field_sources: BTreeMap<String, Vec<SourceTag>>,
    #[serde(default)]
    unknown_reasons: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NamedPetRecord {
    name: String,
    species_kind: Option<String>,
    npc_owner: Option<String>,
    source: Vec<SourceTag>,
    field_sources: BTreeMap<String, Vec<SourceTag>>,
    unknown_reasons: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DefinitionGap {
    source: SourceTag,
    status: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PetNameParts {
    first: Vec<NamePart>,
    second: Vec<NamePart>,
    source: SourceTag,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NamePart {
    key: String,
    value: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PetTrickDefinition {
    id: u32,
    waypoint_count: usize,
    source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PetCareItem {
    definition_id: i32,
    label: String,
    action: String,
    need: String,
    restore: f32,
    consumes: bool,
    source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NeedPolicy {
    maximum: f32,
    idle_decay_per_second: f32,
    summoned_decay_per_second: f32,
    source: SourceTag,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", content = "evidence", rename_all = "snake_case")]
enum SourceTag {
    Client(String),
    Reference(String),
    Reconstructed(String),
    Wiki(String),
}

impl PetContent {
    pub fn bundled() -> io::Result<Self> {
        Self::from_json(PET_CONTENT.as_bytes())
    }

    pub fn from_json(bytes: &[u8]) -> io::Result<Self> {
        let content: Self = serde_json::from_slice(bytes).map_err(parse_error)?;
        content.validate()?;
        Ok(content)
    }

    pub fn definition(&self, species_id: u32) -> Option<PetDefinition> {
        // R: Wiki catalogue rows without a resolved runtime id and model remain research-only.
        self.species
            .iter()
            .find(|definition| definition.species_id == Some(species_id))
            .and_then(|definition| {
                Some(PetDefinition {
                    species_id,
                    model: definition.model.clone()?,
                    adoption_item_definition_id: definition.adoption_item_ids.first().copied(),
                    initial_tricks: Vec::new(),
                })
            })
    }

    /// Resolve the pet created by an owned adoption item definition.
    pub fn definition_for_item(&self, item_definition_id: i32) -> Option<PetDefinition> {
        let species_id = self
            .species
            .iter()
            .find(|species| species.adoption_item_ids.contains(&item_definition_id))?
            .species_id?;
        self.definition(species_id)
    }

    fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1
            || self.definition_gap.status.is_empty()
            || self.names.first.len() != 60
            || self.names.second.len() != 250
            || self.names.first[0].value.trim().len() != 0
            || self.tricks.is_empty()
            || self.need_policy.maximum != 100.0
            || !self.need_policy.idle_decay_per_second.is_finite()
            || !self.need_policy.summoned_decay_per_second.is_finite()
            || self.need_policy.idle_decay_per_second <= 0.0
            || self.need_policy.summoned_decay_per_second < self.need_policy.idle_decay_per_second
        {
            return Err(invalid("unsupported pet content"));
        }
        let mut species_ids = BTreeSet::new();
        if self.species.iter().any(|species| {
            species
                .species_id
                .is_some_and(|id| id == 0 || !species_ids.insert(id))
                || species
                    .model
                    .as_ref()
                    .is_some_and(|model| !valid_model(model))
                || species.model.as_ref().is_some_and(|_| {
                    species
                        .model_files
                        .as_ref()
                        .is_none_or(|files| files.is_empty())
                })
                || species
                    .model_files
                    .as_ref()
                    .is_some_and(|files| files.iter().any(|file| file.is_empty()))
                || species
                    .texture_alias
                    .as_ref()
                    .is_some_and(|alias| alias.len() > 4096)
                || species
                    .icon
                    .as_ref()
                    .is_some_and(|icon| icon.image_set_id <= 0 || icon.tint_id < 0)
                || species.adoption_item_ids.iter().any(|id| *id <= 0)
                || species.source.is_empty()
                || (species.species_id.is_none()
                    && (species.name.is_empty() || species.wiki_title.is_empty()))
                || species.field_sources.values().any(Vec::is_empty)
                || species.adoption_source.iter().any(|adoption| {
                    adoption.kind.is_empty()
                        || adoption.source.is_empty()
                        || adoption.field_sources.values().any(Vec::is_empty)
                })
        }) || self.named_pets.iter().any(|pet| {
            pet.name.is_empty()
                || pet.source.is_empty()
                || pet.field_sources.values().any(Vec::is_empty)
        }) {
            return Err(invalid("invalid pet definition content"));
        }
        let mut keys = BTreeSet::new();
        for part in self.names.first.iter().chain(&self.names.second) {
            if part.key.is_empty()
                || part.value.chars().count() > 32
                || part.value.chars().any(char::is_control)
                || !keys.insert(part.key.as_str())
            {
                return Err(invalid("invalid pet name part"));
            }
        }
        let mut trick_ids = BTreeSet::new();
        for trick in &self.tricks {
            if trick.id == 0
                || !(3..=15).contains(&trick.waypoint_count)
                || !trick_ids.insert(trick.id)
            {
                return Err(invalid("invalid pet trick content"));
            }
        }
        let mut item_ids = BTreeSet::new();
        for item in &self.care_items {
            if item.definition_id <= 0
                || !item.consumes
                || !item.restore.is_finite()
                || !(0.0..=self.need_policy.maximum).contains(&item.restore)
                || !matches!(
                    (item.action.as_str(), item.need.as_str()),
                    ("feed", "food") | ("groom", "groom") | ("play", "play")
                )
                || !item_ids.insert(item.definition_id)
                || item.source.is_empty()
            {
                return Err(invalid("invalid pet care item"));
            }
        }
        Ok(())
    }

    fn has_trick(&self, trick_id: u32) -> bool {
        self.tricks.iter().any(|trick| trick.id == trick_id)
    }
}

/// Resolve the recovered numeric model id for a pet ADR. Unknown ADRs remain
/// zero; every model bound by the bundled content has a checked mapping.
pub fn model_id(model: &str) -> i32 {
    MODEL_IDS
        .iter()
        .find(|(file, _)| *file == model)
        .map_or(0, |(_, id)| *id)
}

/// Content-backed definition; W46c species ids are explicitly reconstructed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetDefinition {
    pub species_id: u32,
    pub model: String,
    pub adoption_item_definition_id: Option<i32>,
    pub initial_tricks: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PetBusy {
    /// (R) Summoning is refused while a minigame run is active.
    pub minigame_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PetNeeds {
    // (R) The local storage order is food/groom/play/mood; exact native offsets are unresolved.
    pub food: f32,
    pub groom: f32,
    pub play: f32,
    pub mood: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PetEquipment {
    pub head: Option<i32>,
    pub collar: Option<i32>,
    pub chest: Option<i32>,
    pub feet: Option<i32>,
    pub toy: Option<i32>,
    pub utilities: [Option<i32>; 8],
}

impl Default for PetEquipment {
    fn default() -> Self {
        Self {
            head: None,
            collar: None,
            chest: None,
            feet: None,
            toy: None,
            utilities: [None; 8],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pet {
    pub id: u32,
    pub species_id: u32,
    pub model: String,
    pub name: String,
    pub needs: PetNeeds,
    pub learned_tricks: BTreeSet<u32>,
    pub favorite_tricks: [Option<u32>; 4],
    pub equipment: PetEquipment,
    /// The inventory definition consumed to adopt this pet, when one was required.
    #[serde(default)]
    pub adoption_item_definition_id: Option<i32>,
    updated_at: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PetRoster {
    version: u8,
    next_pet_id: u32,
    pets: Vec<Pet>,
    vendor_grants: BTreeMap<u64, u32>,
}

impl Default for PetRoster {
    fn default() -> Self {
        Self {
            version: 1,
            next_pet_id: 1,
            pets: Vec::new(),
            vendor_grants: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PetPresence {
    pub owner_guid: u64,
    pub pet_id: u32,
    pub actor_guid: u64,
    pub species_id: u32,
    pub model: String,
    /// Numeric model id from the recovered `Models.txt` binding; zero leaves the
    /// actor to the client's model-customization string.
    pub model_id: i32,
    pub texture_alias: String,
    pub name: String,
    pub position: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub enum PetPresenceEvent {
    Spawn(PetPresence),
    Despawn { owner_guid: u64, actor_guid: u64 },
    Follow { actor_guid: u64, target: [f32; 3] },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PetTransition {
    /// Inner pet-family packet, before the gateway tunnel wrapper.
    pub owner_message: Vec<u8>,
    /// (R) Adapter events for other players; no actor-spawn datagram is recovered.
    pub broadcasts: Vec<PetPresenceEvent>,
    /// The roster or active-pet state differs from the state before this transition.
    pub changed: bool,
    /// The saved pet roster differs from the state before this transition.
    pub pets_changed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PetCareOutcome {
    pub inventory_replies: Vec<Vec<u8>>,
    pub pet_status_packet: Vec<u8>,
}

#[derive(Clone, Copy)]
enum Need {
    Food,
    Groom,
    Play,
}

/// A decoded pet request. Only layouts pinned by a client sender are accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PetRequest {
    Summon { pet_id: u32 },
    Recall { pet_id: u32 },
    ForcedDismiss,
    SummonByItem { pet_id: u32, item_instance_id: u32 },
    Rename { name: String },
    UiMode(u32),
}

impl PetRequest {
    /// Decode one tunnel-unwrapped application body, including its `53` family header.
    pub fn decode(inner: &[u8]) -> io::Result<Option<Self>> {
        let Some(family) = inner.get(..2) else {
            return Ok(None);
        };
        if u16::from_le_bytes([family[0], family[1]]) != PET_FAMILY {
            return Ok(None);
        }
        let subtype = *inner
            .get(2)
            .ok_or_else(|| invalid("pet request has no subtype"))?;
        let body = inner.get(3..).unwrap_or_default();
        let word = |offset: usize| {
            body.get(offset..offset + 4)
                .and_then(|bytes| bytes.try_into().ok())
                .map(u32::from_le_bytes)
        };
        match subtype {
            4 => match body {
                [pet_id @ .., action] if body.len() == 5 => {
                    let pet_id = word(0).ok_or_else(|| invalid("short pet summon request"))?;
                    let request = match (*action, pet_id) {
                        (1, pet_id) => PetRequest::Summon { pet_id },
                        (0, pet_id) => PetRequest::Recall { pet_id },
                        (_, _) => return Err(invalid("unknown pet summon action")),
                    };
                    if pet_id == 0 {
                        return Err(invalid("pet summon request has a zero pet id"));
                    }
                    Ok(Some(request))
                }
                [0] => Ok(Some(PetRequest::ForcedDismiss)),
                [1] => Err(invalid("forced pet dismissal cannot summon")),
                _ => Err(invalid("pet summon request has the wrong length")),
            },
            6 => {
                if body.len() < 4 {
                    return Err(invalid("pet name request is truncated"));
                }
                let length = word(0).ok_or_else(|| invalid("short pet name request"))? as usize;
                if length == 0 || length > 256 || body.len() != length + 4 {
                    return Err(invalid("pet name request has the wrong length"));
                }
                let name = std::str::from_utf8(&body[4..])
                    .map_err(|_| invalid("pet name is not UTF-8"))?;
                validate_name(name)?;
                Ok(Some(PetRequest::Rename {
                    name: name.to_owned(),
                }))
            }
            24 => {
                let mode = word(0).ok_or_else(|| invalid("short pet UI mode request"))?;
                if body.len() != 4 {
                    return Err(invalid("pet UI mode request has the wrong length"));
                }
                if mode > 2 {
                    return Err(invalid("pet UI mode is outside the recovered range"));
                }
                Ok(Some(PetRequest::UiMode(mode)))
            }
            32 => {
                if body.len() != 8 {
                    return Err(invalid("pet item summon request has the wrong length"));
                }
                let pet_id = word(0).ok_or_else(|| invalid("short pet item request"))?;
                let item_instance_id = word(4).ok_or_else(|| invalid("short pet item instance"))?;
                if pet_id == 0 || item_instance_id == 0 {
                    return Err(invalid("pet item summon request has a zero field"));
                }
                Ok(Some(PetRequest::SummonByItem {
                    pet_id,
                    item_instance_id,
                }))
            }
            // Other client-bound pet subtypes are not enabled without a recovered layout.
            _ => Ok(None),
        }
    }
}

/// Inner packets produced by one pet request. `shared` packets are also in `private`
/// for the owner and must be relayed to interested neighbours by the gateway.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PetWire {
    pub private: Vec<Vec<u8>>,
    pub shared: Vec<Vec<u8>>,
    pub notice: Option<&'static str>,
    pub changed: bool,
    pub pets_changed: bool,
}

#[derive(Clone, Copy)]
struct ActivePet {
    pet_id: u32,
    owner_guid: u64,
    actor_guid: u64,
    position: [f32; 3],
    follow_target: Option<[f32; 3]>,
    in_water: bool,
}

#[derive(Clone)]
pub struct PetService {
    content: Arc<PetContent>,
    roster: PetRoster,
    active: Option<ActivePet>,
    ui_mode: u32,
}

impl PetService {
    pub fn bundled_content() -> io::Result<PetContent> {
        PetContent::bundled()
    }

    pub fn new(content: PetContent) -> io::Result<Self> {
        content.validate()?;
        Ok(Self {
            content: Arc::new(content),
            roster: PetRoster::default(),
            active: None,
            ui_mode: 0,
        })
    }

    pub fn load_from(content: PetContent, path: &Path) -> io::Result<Self> {
        content.validate()?;
        let roster = game_store::load_pets_state(path)?.unwrap_or_default();
        validate_roster(&content, &roster)?;
        Ok(Self {
            content: Arc::new(content),
            roster,
            active: None,
            ui_mode: 0,
        })
    }

    pub fn from_saved(content: PetContent, roster: Option<PetRoster>) -> io::Result<Self> {
        content.validate()?;
        let roster = roster.unwrap_or_default();
        validate_roster(&content, &roster)?;
        Ok(Self {
            content: Arc::new(content),
            roster,
            active: None,
            ui_mode: 0,
        })
    }

    /// The persisted roster without the session-only active actor.
    pub fn roster_snapshot(&self) -> Option<PetRoster> {
        (!self.roster.pets.is_empty()).then(|| self.roster.clone())
    }

    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        validate_roster(&self.content, &self.roster)?;
        game_store::save_pets_state(path, &self.roster)
    }

    pub fn pet(&self, pet_id: u32) -> Option<&Pet> {
        self.roster.pets.iter().find(|pet| pet.id == pet_id)
    }

    pub fn active_pet_id(&self) -> Option<u32> {
        self.active.map(|active| active.pet_id)
    }

    pub fn ui_mode(&self) -> u32 {
        self.ui_mode
    }

    fn texture_alias(&self, species_id: u32) -> String {
        self.content
            .species
            .iter()
            .find(|species| species.species_id == Some(species_id))
            .and_then(|species| species.texture_alias.clone())
            .unwrap_or_default()
    }

    fn display_name(&self, pet: &Pet) -> String {
        if !pet.name.is_empty() {
            return pet.name.clone();
        }
        self.content
            .species
            .iter()
            .find(|species| species.species_id == Some(pet.species_id))
            .map(|species| species.name.clone())
            .unwrap_or_default()
    }

    /// (R) One active pet per owner; the actor id never comes from the client.
    pub fn pet_guid(owner_guid: u64) -> u64 {
        owner_guid ^ PET_GUID_TAG
    }

    pub fn view(&self, owner_guid: u64) -> Option<PetPresence> {
        let active = self.active?;
        if active.owner_guid != owner_guid {
            return None;
        }
        let pet = self.pet(active.pet_id)?;
        Some(PetPresence {
            owner_guid,
            pet_id: pet.id,
            actor_guid: active.actor_guid,
            species_id: pet.species_id,
            model: pet.model.clone(),
            model_id: model_id(&pet.model),
            texture_alias: self.texture_alias(pet.species_id),
            name: self.display_name(pet),
            position: active.position,
        })
    }

    /// Reset only the connection-scoped actor state; the saved roster stays.
    pub fn reset_session(&mut self) {
        self.active = None;
        self.ui_mode = 0;
    }

    /// A save may carry a later monotonic clock than a restarted server. Needs
    /// do not decay across a restart, so pull the saved clocks back (R).
    pub(crate) fn rebase_clock(&mut self, now: u64) {
        for pet in &mut self.roster.pets {
            pet.updated_at = pet.updated_at.min(now);
        }
    }

    pub fn set_ui_mode(&mut self, mode: u32) -> io::Result<()> {
        if mode > 2 {
            return Err(invalid("pet UI mode is outside the recovered range"));
        }
        self.ui_mode = mode;
        Ok(())
    }

    pub fn active_follow_state(&self) -> Option<(u64, Option<[f32; 3]>, bool)> {
        self.active
            .map(|active| (active.actor_guid, active.follow_target, active.in_water))
    }

    pub fn adopt_from_owned_item(
        &mut self,
        inventory: &mut Inventory,
        item_instance_id: i32,
        definition: &PetDefinition,
        now: u64,
    ) -> io::Result<u32> {
        self.adopt_with_replies(inventory, item_instance_id, definition, now)
            .map(|(id, _)| id)
    }

    /// Adopt an owned item and keep the inventory replies needed by the client.
    pub fn adopt_with_replies(
        &mut self,
        inventory: &mut Inventory,
        item_instance_id: i32,
        definition: &PetDefinition,
        now: u64,
    ) -> io::Result<(u32, Vec<Vec<u8>>)> {
        let item_definition_id = definition
            .adoption_item_definition_id
            .ok_or_else(|| invalid("pet definition has no adoption item"))?;
        let item = inventory
            .state()
            .items
            .iter()
            .find(|item| item.id == item_instance_id)
            .ok_or_else(|| invalid("pet adoption item is not owned"))?;
        if item.definition_id != item_definition_id {
            return Err(invalid("owned item is not this pet's adoption item"));
        }
        let mut next = self.roster.clone();
        let id = next.next_pet_id;
        insert_pet(&self.content, &mut next, definition, now)?;
        validate_roster(&self.content, &next)?;
        let mut next_inventory = inventory.clone();
        let consumed = next_inventory.consume(item_instance_id, 1)?;
        if !consumed.changed {
            return Err(invalid("pet adoption item was not consumed"));
        }
        self.roster = next;
        *inventory = next_inventory;
        Ok((id, consumed.replies))
    }

    /// (R) A vendor/reward grant creates a pet directly; grant ids deduplicate retries.
    pub fn adopt_vendor_grant(
        &mut self,
        grant_id: u64,
        definition: &PetDefinition,
        now: u64,
    ) -> io::Result<u32> {
        if grant_id == 0 {
            return Err(invalid("vendor pet grant id must be non-zero"));
        }
        if let Some(&pet_id) = self.roster.vendor_grants.get(&grant_id) {
            return Ok(pet_id);
        }
        let mut next = self.roster.clone();
        let pet_id = insert_pet(&self.content, &mut next, definition, now)?;
        next.vendor_grants.insert(grant_id, pet_id);
        validate_roster(&self.content, &next)?;
        self.roster = next;
        Ok(pet_id)
    }

    /// Client sends selected zero-based rows from `PetNames.txt`; custom vanity
    /// names and name-change purchases are outside this recovered slice.
    pub fn rename(&mut self, pet_id: u32, first: usize, second: usize) -> io::Result<String> {
        let first = self
            .content
            .names
            .first
            .get(first)
            .ok_or_else(|| invalid("invalid pet first-name selection"))?;
        let second = self
            .content
            .names
            .second
            .get(second)
            .ok_or_else(|| invalid("invalid pet second-name selection"))?;
        let name = format!("{} {}", first.value.trim(), second.value.trim())
            .trim()
            .to_owned();
        self.rename_selected(pet_id, &name)?;
        Ok(name)
    }

    /// Apply a subtype-6 name to the active pet. The request carries no pet id;
    /// `00b5f500` finds the active pet record before it sends the string.
    fn rename_selected(&mut self, pet_id: u32, name: &str) -> io::Result<()> {
        validate_name(name)?;
        let selected = self.content.names.first.iter().any(|first| {
            self.content.names.second.iter().any(|second| {
                format!("{} {}", first.value.trim(), second.value.trim()).trim() == name
            })
        });
        if !selected {
            return Err(invalid("pet name is not in the recovered name lists"));
        }
        pet_mut(&mut self.roster, pet_id)?.name = name.to_owned();
        Ok(())
    }

    pub fn summon(
        &mut self,
        pet_id: u32,
        actor_guid: u64,
        owner_guid: u64,
        owner_position: [f32; 3],
        busy: PetBusy,
        now: u64,
    ) -> io::Result<PetTransition> {
        if busy.minigame_active {
            return Err(invalid("cannot summon a pet during a minigame"));
        }
        if actor_guid == 0
            || owner_guid == 0
            || !owner_position.iter().all(|value| value.is_finite())
        {
            return Err(invalid("invalid pet summon identity or position"));
        }
        // (R) The adapter supplies the actor id; this prototype starts it at the owner's position.
        let (species_id, model, name) = {
            let pet = self
                .pet(pet_id)
                .ok_or_else(|| invalid("pet is not owned"))?;
            (pet.species_id, pet.model.clone(), self.display_name(pet))
        };
        self.advance_time(now)?;
        let mut broadcasts = Vec::new();
        if let Some(active) = self.active {
            broadcasts.push(PetPresenceEvent::Despawn {
                owner_guid: active.owner_guid,
                actor_guid: active.actor_guid,
            });
        }
        self.active = Some(ActivePet {
            pet_id,
            owner_guid,
            actor_guid,
            position: owner_position,
            follow_target: None,
            in_water: false,
        });
        let model_id = model_id(&model);
        broadcasts.push(PetPresenceEvent::Spawn(PetPresence {
            owner_guid,
            pet_id,
            actor_guid,
            species_id,
            model,
            model_id,
            texture_alias: self.texture_alias(species_id),
            name,
            position: owner_position,
        }));
        Ok(PetTransition {
            owner_message: pet_active_packet(pet_id, actor_guid),
            broadcasts,
            changed: true,
            pets_changed: true,
        })
    }

    pub fn dismiss(&mut self, now: u64) -> io::Result<PetTransition> {
        if self.active.is_none() {
            return Ok(PetTransition {
                owner_message: pet_active_packet(0, 0),
                broadcasts: Vec::new(),
                changed: false,
                pets_changed: false,
            });
        }
        self.advance_time(now)?;
        let broadcasts = self.active.take().map_or_else(Vec::new, |active| {
            vec![PetPresenceEvent::Despawn {
                owner_guid: active.owner_guid,
                actor_guid: active.actor_guid,
            }]
        });
        // (R) The client sentinel pair is unresolved; zero id/GUID is the local policy.
        Ok(PetTransition {
            owner_message: pet_active_packet(0, 0),
            broadcasts,
            changed: true,
            pets_changed: true,
        })
    }

    pub fn recall(&mut self, pet_id: u32, now: u64) -> io::Result<PetTransition> {
        if self.active.map(|active| active.pet_id) != Some(pet_id) {
            return Err(invalid("that pet is not summoned"));
        }
        self.dismiss(now)
    }

    /// Summon by an inventory instance only when it is the same adoption item that created
    /// this saved pet. The roster remains the ownership authority.
    pub fn summon_by_item(
        &mut self,
        pet_id: u32,
        item_instance_id: u32,
        actor_guid: u64,
        owner_guid: u64,
        owner_position: [f32; 3],
        inventory: &InventoryState,
        busy: PetBusy,
        now: u64,
    ) -> io::Result<PetTransition> {
        let expected = self
            .pet(pet_id)
            .ok_or_else(|| invalid("pet is not owned"))?
            .adoption_item_definition_id
            .ok_or_else(|| invalid("pet has no adoption item"))?;
        let owned = inventory.items.iter().any(|item| {
            i32::try_from(item_instance_id).ok() == Some(item.id)
                && item.count > 0
                && item.definition_id == expected
        });
        if !owned {
            return Err(invalid("pet adoption item is not owned"));
        }
        self.summon(pet_id, actor_guid, owner_guid, owner_position, busy, now)
    }

    /// Route subtype 32. The two recovered words have no settled field meaning,
    /// so an owned item may be named by either instance or definition id; the
    /// roster and item definition still decide whether a pet is created.
    fn summon_by_item_owned(
        &mut self,
        pet_id: u32,
        item_id: u32,
        actor_guid: u64,
        owner_guid: u64,
        owner_position: [f32; 3],
        inventory: &mut Inventory,
        busy: PetBusy,
        now: u64,
    ) -> io::Result<(PetTransition, Vec<Vec<u8>>)> {
        let item = inventory
            .state()
            .items
            .iter()
            .filter(|item| item.count > 0)
            .find(|item| {
                i32::try_from(item_id).ok() == Some(item.id)
                    || i32::try_from(item_id).ok() == Some(item.definition_id)
            })
            .or_else(|| {
                inventory.state().items.iter().find(|item| {
                    item.count > 0
                        && (i32::try_from(pet_id).ok() == Some(item.id)
                            || i32::try_from(pet_id).ok() == Some(item.definition_id))
                })
            })
            .cloned()
            .ok_or_else(|| invalid("pet adoption item is not owned"))?;

        if let Some(pet) = self.pet(pet_id) {
            if pet.adoption_item_definition_id != Some(item.definition_id) {
                return Err(invalid("pet adoption item is not owned"));
            }
            return self
                .summon(pet_id, actor_guid, owner_guid, owner_position, busy, now)
                .map(|transition| (transition, Vec::new()));
        }

        let definition = self
            .content
            .definition_for_item(item.definition_id)
            .ok_or_else(|| invalid("item is not a configured pet adoption item"))?;
        let (new_pet_id, replies) =
            self.adopt_with_replies(inventory, item.id, &definition, now)?;
        let transition = self.summon(
            new_pet_id,
            actor_guid,
            owner_guid,
            owner_position,
            busy,
            now,
        )?;
        Ok((transition, replies))
    }

    /// Decode and apply one pet-family request. Refusals are non-mutating and return a
    /// private roster refresh plus a visible notice.
    pub fn handle(
        &mut self,
        inner: &[u8],
        owner_guid: u64,
        owner_position: [f32; 3],
        inventory: &InventoryState,
        busy: PetBusy,
        now: u64,
    ) -> io::Result<Option<PetWire>> {
        let Some(request) = PetRequest::decode(inner)? else {
            return Ok(None);
        };
        let actor_guid = Self::pet_guid(owner_guid);
        let mut next = self.clone();
        let applied = match request {
            PetRequest::Summon { pet_id } => {
                next.rebase_clock(now);
                next.summon(pet_id, actor_guid, owner_guid, owner_position, busy, now)
            }
            PetRequest::Recall { pet_id } => {
                next.rebase_clock(now);
                next.recall(pet_id, now)
            }
            PetRequest::ForcedDismiss => {
                if next.active.is_some() {
                    next.rebase_clock(now);
                }
                next.dismiss(now)
            }
            PetRequest::SummonByItem {
                pet_id,
                item_instance_id,
            } => {
                next.rebase_clock(now);
                next.summon_by_item(
                    pet_id,
                    item_instance_id,
                    actor_guid,
                    owner_guid,
                    owner_position,
                    inventory,
                    busy,
                    now,
                )
            }
            PetRequest::Rename { name } => {
                let Some(active) = next.active else {
                    return Ok(Some(PetWire {
                        private: vec![pet_set_name_packet(0, "", false), next.list_packet()],
                        ..PetWire::default()
                    }));
                };
                if next.rename_selected(active.pet_id, &name).is_err() {
                    return Ok(Some(PetWire {
                        private: vec![
                            pet_set_name_packet(active.actor_guid, "", false),
                            next.list_packet(),
                        ],
                        ..PetWire::default()
                    }));
                }
                let wire = PetWire {
                    private: vec![
                        pet_set_name_packet(active.actor_guid, &name, true),
                        next.list_packet(),
                    ],
                    changed: true,
                    pets_changed: true,
                    ..PetWire::default()
                };
                *self = next;
                return Ok(Some(wire));
            }
            PetRequest::UiMode(mode) => {
                next.set_ui_mode(mode)?;
                *self = next;
                return Ok(Some(PetWire::default()));
            }
        };
        match applied {
            Ok(transition) => {
                *self = next;
                Ok(Some(self.wire(transition)))
            }
            Err(error) => Ok(Some(PetWire {
                private: vec![self.list_packet()],
                notice: Some(refusal_text(&error)),
                ..PetWire::default()
            })),
        }
    }

    /// Handle pet requests with the character's live inventory. This is the
    /// gateway route: subtype 32 may create a pet from an owned adoption item,
    /// while all other requests use the same staged path as `handle`.
    pub fn handle_with_inventory(
        &mut self,
        inner: &[u8],
        owner_guid: u64,
        owner_position: [f32; 3],
        inventory: &mut Inventory,
        busy: PetBusy,
        now: u64,
    ) -> io::Result<Option<PetWire>> {
        let Some(request) = PetRequest::decode(inner)? else {
            return Ok(None);
        };
        let PetRequest::SummonByItem {
            pet_id,
            item_instance_id,
        } = request
        else {
            return self.handle(
                inner,
                owner_guid,
                owner_position,
                inventory.state(),
                busy,
                now,
            );
        };

        let actor_guid = Self::pet_guid(owner_guid);
        let mut next = self.clone();
        next.rebase_clock(now);
        let mut next_inventory = inventory.clone();
        let applied = next.summon_by_item_owned(
            pet_id,
            item_instance_id,
            actor_guid,
            owner_guid,
            owner_position,
            &mut next_inventory,
            busy,
            now,
        );
        match applied {
            Ok((transition, inventory_replies)) => {
                *self = next;
                *inventory = next_inventory;
                let mut wire = self.wire(transition);
                let mut private = inventory_replies;
                private.push(self.list_packet());
                private.extend(wire.private);
                wire.private = private;
                Ok(Some(wire))
            }
            Err(error) => Ok(Some(PetWire {
                private: vec![self.list_packet()],
                notice: Some(refusal_text(&error)),
                ..PetWire::default()
            })),
        }
    }

    pub fn wire(&self, transition: PetTransition) -> PetWire {
        let mut wire = PetWire {
            private: vec![transition.owner_message],
            changed: transition.changed,
            pets_changed: transition.pets_changed,
            ..PetWire::default()
        };
        for event in transition.broadcasts {
            match event {
                PetPresenceEvent::Spawn(presence) => {
                    wire.shared.extend(presence.spawn_packet());
                }
                PetPresenceEvent::Despawn { actor_guid, .. } => {
                    wire.shared.push(pet_removal_packet(actor_guid));
                }
                // Pet locomotion has no recovered actor-move packet; the owner still receives
                // the client-shaped location acknowledgement from `follow_location`.
                PetPresenceEvent::Follow { .. } => {}
            }
        }
        wire.private.extend(wire.shared.iter().cloned());
        wire
    }

    pub fn list_packet(&self) -> Vec<u8> {
        let mut bytes = pet_header(5);
        bytes.extend((self.roster.pets.len() as u32).to_le_bytes());
        for pet in &self.roster.pets {
            bytes.extend(pet.id.to_le_bytes()); // list key, before the record
            bytes.extend(pet.id.to_le_bytes());
            append_pet_record(&mut bytes, pet);
        }
        let (pet_id, actor_guid) = self
            .active
            .map_or((0, 0), |active| (active.pet_id, active.actor_guid));
        bytes.extend(pet_id.to_le_bytes());
        bytes.extend(actor_guid.to_le_bytes());
        bytes
    }

    /// Store a finite follow destination and return the client acknowledgement
    /// plus a world-adapter event. Reachability thresholds remain unrecovered.
    pub fn follow_location(&mut self, target: [f32; 3]) -> io::Result<PetTransition> {
        if !target.iter().all(|value| value.is_finite()) {
            return Err(invalid("invalid pet follow location"));
        }
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| invalid("no active pet"))?;
        active.follow_target = Some(target);
        // (R) No distance constants were recovered; a finite target receives an allow reply here.
        Ok(PetTransition {
            owner_message: pet_can_go_packet(active.actor_guid, true),
            broadcasts: vec![PetPresenceEvent::Follow {
                actor_guid: active.actor_guid,
                target,
            }],
            changed: false,
            pets_changed: false,
        })
    }

    pub fn set_in_water(&mut self, in_water: bool) -> io::Result<()> {
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| invalid("no active pet"))?;
        active.in_water = in_water;
        Ok(())
    }

    /// (R) A toy is retained equipment; playing raises the play bar without consuming it.
    pub fn play(&mut self, pet_id: u32, now: u64) -> io::Result<Vec<u8>> {
        if self.active.map(|active| active.pet_id) != Some(pet_id) {
            return Err(invalid("play requires the target pet to be active"));
        }
        let mut next = self.roster.clone();
        advance_roster(
            &self.content.need_policy,
            &mut next,
            self.active.map(|active| active.pet_id),
            now,
        )?;
        let pet = pet_mut(&mut next, pet_id)?;
        if pet.equipment.toy.is_none() {
            return Err(invalid("play requires an equipped pet toy"));
        }
        add_need(
            &mut pet.needs,
            Need::Play,
            20.0,
            self.content.need_policy.maximum,
        );
        pet.needs.mood = (pet.needs.food + pet.needs.groom + pet.needs.play) / 3.0;
        validate_roster(&self.content, &next)?;
        let packet = pet_status_packet(pet_id, &pet_mut(&mut next, pet_id)?.needs);
        self.roster = next;
        Ok(packet)
    }

    pub fn equip_item(
        &mut self,
        pet_id: u32,
        item_instance_id: i32,
        slot: PetEquipmentSlot,
        inventory: &Inventory,
    ) -> io::Result<()> {
        if !inventory
            .state()
            .items
            .iter()
            .any(|item| item.id == item_instance_id && item.count > 0)
        {
            return Err(invalid("pet equipment item is not owned"));
        }
        // (R) The local save stores owned inventory instance ids in client-shaped pet slots.
        let pet = pet_mut(&mut self.roster, pet_id)?;
        match slot {
            PetEquipmentSlot::Head => pet.equipment.head = Some(item_instance_id),
            PetEquipmentSlot::Collar => pet.equipment.collar = Some(item_instance_id),
            PetEquipmentSlot::Chest => pet.equipment.chest = Some(item_instance_id),
            PetEquipmentSlot::Feet => pet.equipment.feet = Some(item_instance_id),
            PetEquipmentSlot::Toy => pet.equipment.toy = Some(item_instance_id),
            PetEquipmentSlot::Utility(index) if index < 8 => {
                pet.equipment.utilities[index as usize] = Some(item_instance_id)
            }
            PetEquipmentSlot::Utility(_) => return Err(invalid("invalid pet utility slot")),
        }
        Ok(())
    }

    pub fn unequip_item(&mut self, pet_id: u32, slot: PetEquipmentSlot) -> io::Result<()> {
        let pet = pet_mut(&mut self.roster, pet_id)?;
        match slot {
            PetEquipmentSlot::Head => pet.equipment.head = None,
            PetEquipmentSlot::Collar => pet.equipment.collar = None,
            PetEquipmentSlot::Chest => pet.equipment.chest = None,
            PetEquipmentSlot::Feet => pet.equipment.feet = None,
            PetEquipmentSlot::Toy => pet.equipment.toy = None,
            PetEquipmentSlot::Utility(index) if index < 8 => {
                pet.equipment.utilities[index as usize] = None
            }
            PetEquipmentSlot::Utility(_) => return Err(invalid("invalid pet utility slot")),
        }
        Ok(())
    }

    /// Consume exactly one configured care item through Inventory's
    /// staged transaction, then return the client-shaped status update.
    pub fn care(
        &mut self,
        pet_id: u32,
        item_instance_id: i32,
        inventory: &mut Inventory,
        busy: PetBusy,
        now: u64,
    ) -> io::Result<PetCareOutcome> {
        if busy.minigame_active {
            return Err(invalid("cannot care for a pet during a minigame"));
        }
        let active = self
            .active
            .filter(|active| active.pet_id == pet_id)
            .ok_or_else(|| invalid("care requires the target pet to be active"))?;
        if active.in_water {
            return Err(invalid("cannot care for a pet while its owner is in water"));
        }
        let item = inventory
            .state()
            .items
            .iter()
            .find(|item| item.id == item_instance_id)
            .ok_or_else(|| invalid("pet care item is not owned"))?;
        if !self
            .pet(pet_id)
            .is_some_and(|pet| pet.equipment.utilities.contains(&Some(item_instance_id)))
        {
            return Err(invalid("pet care item is not equipped in a utility slot"));
        }
        let rule = self
            .content
            .care_items
            .iter()
            .find(|rule| rule.definition_id == item.definition_id && rule.consumes)
            .ok_or_else(|| invalid("item is not a configured pet care consumable"))?;
        let need = parse_need(&rule.need)?;
        let mut next_roster = self.roster.clone();
        advance_roster(
            &self.content.need_policy,
            &mut next_roster,
            self.active.map(|active| active.pet_id),
            now,
        )?;
        let pet = pet_mut(&mut next_roster, pet_id)?;
        add_need(
            &mut pet.needs,
            need,
            rule.restore,
            self.content.need_policy.maximum,
        );
        pet.needs.mood = (pet.needs.food + pet.needs.groom + pet.needs.play) / 3.0;
        validate_roster(&self.content, &next_roster)?;

        let mut next_inventory = inventory.clone();
        let item_result = next_inventory.consume(item_instance_id, 1)?;
        if !next_inventory
            .state()
            .items
            .iter()
            .any(|item| item.id == item_instance_id)
        {
            for pet in &mut next_roster.pets {
                clear_equipped_instance(&mut pet.equipment, item_instance_id);
            }
        }
        let status = pet_status_packet(pet_id, &pet_mut(&mut next_roster, pet_id)?.needs);
        self.roster = next_roster;
        *inventory = next_inventory;
        Ok(PetCareOutcome {
            inventory_replies: item_result.replies,
            pet_status_packet: status,
        })
    }

    /// (R) Decay rates use client food/groom/play bars; the shipped client has
    /// no interval or decay table. Active pets decay faster, following the wiki.
    pub fn advance_time(&mut self, now: u64) -> io::Result<()> {
        let mut next = self.roster.clone();
        advance_roster(
            &self.content.need_policy,
            &mut next,
            self.active.map(|active| active.pet_id),
            now,
        )?;
        validate_roster(&self.content, &next)?;
        self.roster = next;
        Ok(())
    }

    /// Caller applies the reconstructed Pet Trainer job/level gate before unlocking a trick.
    pub fn learn_trick(&mut self, pet_id: u32, trick_id: u32) -> io::Result<()> {
        if !self.content.has_trick(trick_id) {
            return Err(invalid("unknown pet trick id"));
        }
        pet_mut(&mut self.roster, pet_id)?
            .learned_tricks
            .insert(trick_id);
        Ok(())
    }

    /// `client_slot` is one-based. Client sender 00b5f670 converts it to the
    /// zero-based slot stored by the server-side pet record.
    pub fn set_favorite(
        &mut self,
        pet_id: u32,
        client_slot: u8,
        trick_id: u32,
    ) -> io::Result<Vec<u8>> {
        if !(1..=4).contains(&client_slot) || !self.content.has_trick(trick_id) {
            return Err(invalid("invalid pet favorite slot or trick"));
        }
        let pet = pet_mut(&mut self.roster, pet_id)?;
        if !pet.learned_tricks.contains(&trick_id) {
            return Err(invalid("pet does not know favorite trick"));
        }
        let slot = usize::from(client_slot - 1);
        pet.favorite_tricks[slot] = Some(trick_id);
        Ok(pet_favorite_packet(pet_id, slot as u32, trick_id))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetEquipmentSlot {
    Head,
    Collar,
    Chest,
    Feet,
    Toy,
    Utility(u8),
}

fn insert_pet(
    content: &PetContent,
    roster: &mut PetRoster,
    definition: &PetDefinition,
    now: u64,
) -> io::Result<u32> {
    if roster.pets.len() >= MAX_PETS {
        return Err(invalid("pet roster is full"));
    }
    if definition.species_id == 0
        || !valid_model(&definition.model)
        || definition
            .adoption_item_definition_id
            .is_some_and(|id| id <= 0)
        || definition
            .initial_tricks
            .iter()
            .any(|id| !content.has_trick(*id))
    {
        return Err(invalid("invalid pet definition"));
    }
    let id = roster.next_pet_id;
    if id == 0 {
        return Err(invalid("pet id allocation exhausted"));
    }
    roster.next_pet_id = id
        .checked_add(1)
        .ok_or_else(|| invalid("pet id allocation exhausted"))?;
    roster.pets.push(Pet {
        id,
        species_id: definition.species_id,
        model: definition.model.clone(),
        name: String::new(),
        // (R) Need range and initial fullness are not present in client tables.
        needs: PetNeeds {
            food: 100.0,
            groom: 100.0,
            play: 100.0,
            mood: 100.0,
        },
        learned_tricks: definition.initial_tricks.iter().copied().collect(),
        favorite_tricks: [None; 4],
        equipment: PetEquipment::default(),
        adoption_item_definition_id: definition.adoption_item_definition_id,
        updated_at: now,
    });
    Ok(id)
}

fn validate_roster(content: &PetContent, roster: &PetRoster) -> io::Result<()> {
    if roster.version != 1 || roster.next_pet_id == 0 || roster.pets.len() > MAX_PETS {
        return Err(invalid("unsupported pet roster snapshot"));
    }
    let mut ids = BTreeSet::new();
    let mut maximum_id = 0;
    for pet in &roster.pets {
        if pet.id == 0
            || !ids.insert(pet.id)
            || pet.species_id == 0
            || !valid_model(&pet.model)
            || !valid_name_or_empty(&pet.name)
            || [
                pet.needs.food,
                pet.needs.groom,
                pet.needs.play,
                pet.needs.mood,
            ]
            .iter()
            .any(|need| !need.is_finite() || !(0.0..=content.need_policy.maximum).contains(need))
            || pet.learned_tricks.iter().any(|id| !content.has_trick(*id))
            || pet
                .favorite_tricks
                .iter()
                .flatten()
                .any(|id| !pet.learned_tricks.contains(id))
            || pet.adoption_item_definition_id.is_some_and(|id| id <= 0)
        {
            return Err(invalid("invalid pet roster record"));
        }
        maximum_id = maximum_id.max(pet.id);
    }
    if roster.next_pet_id <= maximum_id
        || roster
            .vendor_grants
            .iter()
            .any(|(grant, pet_id)| *grant == 0 || !ids.contains(pet_id))
    {
        return Err(invalid("invalid pet roster counters or grants"));
    }
    Ok(())
}

fn advance_roster(
    policy: &NeedPolicy,
    roster: &mut PetRoster,
    active_pet_id: Option<u32>,
    now: u64,
) -> io::Result<()> {
    for pet in &mut roster.pets {
        let elapsed = now
            .checked_sub(pet.updated_at)
            .ok_or_else(|| invalid("pet clock moved backwards"))?;
        if elapsed == 0 {
            continue;
        }
        let per_second = if active_pet_id == Some(pet.id) {
            policy.summoned_decay_per_second
        } else {
            policy.idle_decay_per_second
        };
        let decay = per_second * elapsed as f32;
        pet.needs.food = (pet.needs.food - decay).max(0.0);
        pet.needs.groom = (pet.needs.groom - decay).max(0.0);
        pet.needs.play = (pet.needs.play - decay).max(0.0);
        // (R) Mood follows the average of client need bars until its formula is recovered.
        pet.needs.mood = (pet.needs.food + pet.needs.groom + pet.needs.play) / 3.0;
        pet.updated_at = now;
    }
    Ok(())
}

fn pet_mut(roster: &mut PetRoster, pet_id: u32) -> io::Result<&mut Pet> {
    roster
        .pets
        .iter_mut()
        .find(|pet| pet.id == pet_id)
        .ok_or_else(|| invalid("pet is not owned"))
}

fn validate_name(name: &str) -> io::Result<()> {
    if name.is_empty() || name.chars().count() > 64 || name.chars().any(char::is_control) {
        return Err(invalid("invalid pet name"));
    }
    Ok(())
}

fn valid_model(model: &str) -> bool {
    !model.is_empty()
        && model.len() <= 128
        && model
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        && !model.contains("..")
}

fn valid_name_or_empty(name: &str) -> bool {
    name.is_empty() || validate_name(name).is_ok()
}

fn parse_need(value: &str) -> io::Result<Need> {
    match value {
        "food" => Ok(Need::Food),
        "groom" => Ok(Need::Groom),
        "play" => Ok(Need::Play),
        _ => Err(invalid("unknown pet need")),
    }
}

fn add_need(needs: &mut PetNeeds, which: Need, amount: f32, maximum: f32) {
    let value = match which {
        Need::Food => &mut needs.food,
        Need::Groom => &mut needs.groom,
        Need::Play => &mut needs.play,
    };
    *value = (*value + amount).min(maximum);
}

fn clear_equipped_instance(equipment: &mut PetEquipment, item_instance_id: i32) {
    for slot in [
        &mut equipment.head,
        &mut equipment.collar,
        &mut equipment.chest,
        &mut equipment.feet,
        &mut equipment.toy,
    ] {
        if *slot == Some(item_instance_id) {
            *slot = None;
        }
    }
    for slot in &mut equipment.utilities {
        if *slot == Some(item_instance_id) {
            *slot = None;
        }
    }
}

fn pet_active_packet(pet_id: u32, actor_guid: u64) -> Vec<u8> {
    // Client reader 00b556c0 consumes a u32 pet id and an 8-byte actor guid.
    // Server subtype 9 and little-endian bytes are reconstructed; no datagram exists.
    let mut bytes = pet_header(9);
    bytes.extend(pet_id.to_le_bytes());
    bytes.extend(actor_guid.to_le_bytes());
    bytes
}

fn pet_set_name_packet(actor_guid: u64, name: &str, accepted: bool) -> Vec<u8> {
    // Client reader 00b55730: the pet actor guid, a length-prefixed string, then
    // the accept flag. The server-side response is reconstructed (R).
    let mut bytes = pet_header(10);
    bytes.extend(actor_guid.to_le_bytes());
    bytes.extend((name.len() as u32).to_le_bytes());
    bytes.extend(name.as_bytes());
    bytes.push(u8::from(accepted));
    bytes
}

impl PetPresence {
    /// (R) AddNpc construction reuses the captured content template. The numeric model id
    /// comes from the recovered `Models.txt` binding and the species texture alias is
    /// carried separately from the actor's display name.
    pub fn spawn_packet(&self) -> Option<Vec<u8>> {
        let npc = startup_world::NpcSpawn {
            actor_id: self.actor_guid,
            model_id: self.model_id,
            texture_alias: self.texture_alias.clone(),
            npc_name: self.name.clone(),
            // (R) Pet movement speed is not present in the recovered content tables.
            speed: 5.0,
            position: [self.position[0], self.position[1], self.position[2], 1.0],
            rotation: [0.0, 0.0, 1.0, 0.0],
            is_interactable: 0,
            interact_range: 0,
            ..world::content_spawn_template()
        };
        world::unframe_tunnel(&startup_world::build_add_npc(&npc)).ok()
    }
}

pub fn pet_removal_packet(actor_guid: u64) -> Vec<u8> {
    // The 35/3 remove-actor shape already used by rides and field enemies (R).
    let mut bytes = vec![35, 0, 3, 0, 1, 0];
    bytes.extend(actor_guid.to_le_bytes());
    bytes.extend([0; 17]);
    bytes
}

fn append_pet_record(out: &mut Vec<u8>, pet: &Pet) {
    put_u32(out, pet.id);
    out.push(0); // Unknown client flag at record +0x04.
    put_u32(out, pet.species_id);
    for value in [
        pet.needs.food,
        pet.needs.groom,
        pet.needs.play,
        pet.needs.mood,
    ] {
        put_f32(out, value);
    }
    out.push(0); // Unknown client flag at record +0x30.
    put_count(out, pet.learned_tricks.len());
    for trick_id in &pet.learned_tricks {
        put_u32(out, *trick_id);
        for _ in 0..8 {
            put_u32(out, 0);
        }
        out.push(1);
    }
    let item_ids: Vec<i32> = [
        pet.equipment.head,
        pet.equipment.collar,
        pet.equipment.chest,
        pet.equipment.feet,
        pet.equipment.toy,
    ]
    .into_iter()
    .flatten()
    .collect();
    put_count(out, item_ids.len());
    for item_id in item_ids {
        put_u32(out, item_id as u32);
    }
    put_count(out, 0); // Profile item rows are not part of this bounded slice.
    put_string(out, &pet.name);
    put_u32(out, 0);
    put_u32(out, 0);
    put_string(out, "");
    put_u32(out, 0);
    out.push(0);
    put_u32(out, 0);
    out.push(0);
    for trick_id in pet.favorite_tricks {
        put_u32(out, trick_id.unwrap_or(0));
    }
    for item_id in pet.equipment.utilities {
        put_u32(out, item_id.unwrap_or(0) as u32);
    }
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_le_bytes());
}

fn put_f32(out: &mut Vec<u8>, value: f32) {
    out.extend(value.to_bits().to_le_bytes());
}

fn put_count(out: &mut Vec<u8>, count: usize) {
    put_u32(
        out,
        u32::try_from(count).expect("bounded pet list count fits u32"),
    );
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    put_u32(
        out,
        u32::try_from(value.len()).expect("bounded pet string fits u32"),
    );
    out.extend(value.as_bytes());
}

fn pet_status_packet(pet_id: u32, needs: &PetNeeds) -> Vec<u8> {
    // Reader 00b555a0 consumes four floats; field mapping/order remains (R).
    let mut bytes = pet_header(3);
    bytes.extend(pet_id.to_le_bytes());
    for value in [needs.food, needs.groom, needs.play, needs.mood] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn pet_favorite_packet(pet_id: u32, slot: u32, trick_id: u32) -> Vec<u8> {
    // Reader 00b55a20 reads three dwords after the pet-family header; server echo is (R).
    let mut bytes = pet_header(14);
    for value in [pet_id, slot, trick_id] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn pet_can_go_packet(actor_guid: u64, can_go: bool) -> Vec<u8> {
    // Reader 00b578d0 consumes a guid pair and flag; server acknowledgement is (R).
    let mut bytes = pet_header(26);
    bytes.extend(actor_guid.to_le_bytes());
    bytes.push(u8::from(can_go));
    bytes
}

fn pet_header(subtype: u8) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(16);
    bytes.extend(PET_FAMILY.to_le_bytes());
    bytes.push(subtype);
    bytes
}

fn refusal_text(error: &io::Error) -> &'static str {
    match error.to_string().as_str() {
        "pet is not owned" => "You do not own that pet.",
        "pet adoption item is not owned" | "pet has no adoption item" => {
            "You do not own that pet item."
        }
        "that pet is not summoned" => "That pet is not summoned.",
        "cannot summon a pet during a minigame" => "You cannot summon a pet right now.",
        _ => "That pet request is not available.",
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn parse_error(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
