//! Saved pet roster and bounded client-shaped pet state.
//!
//! Client facts and source tags live in `data/content/pets.json`. Pet spawning,
//! care effects, need scale/rates, and server replies remain reconstructed (R)
//! because no original pet datagram was recovered.

use crate::{
    inventory::{Inventory, InventoryState},
    startup_world, world,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    sync::Arc,
};

const PET_FAMILY: u16 = 53;
/// (R) Hard ceiling on a saved roster; the client can display a bounded list.
const MAX_PETS: usize = 50;
const PET_CONTENT: &str = include_str!("../data/content/pets.json");
/// (R) One active pet per owner, so an owner-derived actor id is unique while summoned.
const PET_GUID_TAG: u64 = 0x4000_0000_0000_0000;

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
    movement_speed: MovementSpeed,
    /// T454: the sources `scripts/pets_import.py` read (path, commit, sha256).
    #[serde(default)]
    imports: Vec<PetImport>,
    /// T454: every pet model the client ships (`Models.txt` ids), with its tricks and item classes.
    #[serde(default)]
    models: Vec<PetModel>,
    /// T454: every pet item (`ClientItemDefinitions` type 2) whose model resolves to a `models` row.
    #[serde(default)]
    items: Vec<PetItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PetImport {
    name: String,
    path: String,
    commit: Option<String>,
    sha256: String,
}

/// One client pet model: `Models.txt` id and ADR, icon, default tint, vanity flag, its
/// tricks in the client's order and its item classes.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetModel {
    pub model_id: i32,
    pub adr: String,
    #[serde(default)]
    pub gender: Option<i32>,
    #[serde(default)]
    pub icon_id: Option<i32>,
    #[serde(default)]
    pub default_tint_id: Option<i32>,
    #[serde(default)]
    pub default_tint_alias: Option<String>,
    #[serde(default)]
    pub vanity: bool,
    #[serde(default)]
    pub tricks: Vec<PetModelTrick>,
    #[serde(default)]
    pub item_classes: Vec<PetItemClass>,
    src: String,
    #[serde(default)]
    unknown_reasons: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetModelTrick {
    pub id: u32,
    pub known: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetItemClass {
    pub key: i32,
    pub id: i32,
    pub unknown: i32,
}

/// One pet item: the adoption item a player uses (53/32) to get a pet of `model_id`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetItem {
    pub item_id: i32,
    pub model_id: i32,
    pub name_id: i32,
    #[serde(default)]
    pub image_set_id: Option<i32>,
    #[serde(default)]
    pub icon_tint_id: Option<i32>,
    #[serde(default)]
    pub tint_alias: Option<String>,
    #[serde(default)]
    pub texture_alias: Option<String>,
    #[serde(default)]
    pub members_only: bool,
    src: String,
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
    /// PetTrickGesturePoints.txt waypoints; absent = no gesture record (client 00cc9330 scores nothing).
    #[serde(default)]
    waypoint_count: Option<usize>,
    #[serde(default)]
    icon_id: Option<i32>,
    #[serde(default)]
    name_id: Option<i32>,
    #[serde(default)]
    tooltip_id: Option<i32>,
    #[serde(default)]
    skill_level: Option<i32>,
    #[serde(default)]
    unknown5: Option<i32>,
    #[serde(default)]
    unknown6: Option<i32>,
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

/// (R) Pet actor movement speed. No recovered pet content supplies a speed, so
/// the package owns the value instead of every actor taking a code constant.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct MovementSpeed {
    speed: f32,
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
        let Some(species) = self
            .species
            .iter()
            .find(|definition| definition.species_id == Some(species_id))
        else {
            // A pet adopted from an item with no wiki species row keeps the item id as its species (R).
            return i32::try_from(species_id)
                .ok()
                .and_then(|item_id| self.item_definition(item_id, species_id));
        };
        let model = species.model.clone()?;
        Some(PetDefinition {
            species_id,
            initial_tricks: self.known_tricks(self.model_id(&model)),
            model,
            adoption_item_definition_id: species.adoption_item_ids.first().copied(),
        })
    }

    /// Resolve the pet created by an owned adoption item definition: the imported item row
    /// first, then (for items without one) the wiki species that lists the item.
    pub fn definition_for_item(&self, item_definition_id: i32) -> Option<PetDefinition> {
        let wiki_species = self
            .species
            .iter()
            .find(|species| species.adoption_item_ids.contains(&item_definition_id))
            .and_then(|species| species.species_id);
        if self.item(item_definition_id).is_some() {
            // (R) An item no wiki species lists becomes its own species id.
            let species_id = wiki_species.or_else(|| u32::try_from(item_definition_id).ok())?;
            return self.item_definition(item_definition_id, species_id);
        }
        self.definition(wiki_species?)
    }

    fn item_definition(&self, item_id: i32, species_id: u32) -> Option<PetDefinition> {
        let item = self.item(item_id)?;
        Some(PetDefinition {
            species_id,
            model: self.model(item.model_id)?.adr.clone(),
            adoption_item_definition_id: Some(item_id),
            initial_tricks: self.known_tricks(item.model_id),
        })
    }

    fn known_tricks(&self, model_id: i32) -> Vec<u32> {
        self.model(model_id).map_or_else(Vec::new, |model| {
            model
                .tricks
                .iter()
                .filter(|trick| trick.known)
                .map(|trick| trick.id)
                .collect()
        })
    }

    pub fn model(&self, model_id: i32) -> Option<&PetModel> {
        self.models.iter().find(|model| model.model_id == model_id)
    }

    pub fn item(&self, item_id: i32) -> Option<&PetItem> {
        self.items.iter().find(|item| item.item_id == item_id)
    }

    pub fn is_vanity(&self, model_id: i32) -> bool {
        self.model(model_id).is_some_and(|model| model.vanity)
    }

    /// The client model id for a pet ADR; where two ids (male and female) share an ADR, the
    /// lower one (R). Unknown ADRs stay zero.
    pub fn model_id(&self, adr: &str) -> i32 {
        self.models
            .iter()
            .filter(|model| model.adr == adr)
            .map(|model| model.model_id)
            .min()
            .unwrap_or(0)
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
            || !self.movement_speed.speed.is_finite()
            || !(0.1..=20.0).contains(&self.movement_speed.speed)
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
                || trick
                    .waypoint_count
                    .is_some_and(|count| !(3..=15).contains(&count))
                || !trick_ids.insert(trick.id)
            {
                return Err(invalid("invalid pet trick content"));
            }
        }
        let mut model_ids = BTreeSet::new();
        for model in &self.models {
            if model.model_id <= 0
                || !valid_model(&model.adr)
                || model.src.is_empty()
                || !model_ids.insert(model.model_id)
                || model.tricks.len() > 16
                || model.item_classes.len() > 16
                || model
                    .tricks
                    .iter()
                    .any(|trick| !trick_ids.contains(&trick.id))
            {
                return Err(invalid("invalid pet model content"));
            }
        }
        let species_ids: BTreeSet<i64> = self
            .species
            .iter()
            .filter_map(|species| species.species_id.map(i64::from))
            .collect();
        let mut pet_item_ids = BTreeSet::new();
        for item in &self.items {
            if item.item_id <= 0
                || item.src.is_empty()
                || !pet_item_ids.insert(item.item_id)
                || !model_ids.contains(&item.model_id)
                || species_ids.contains(&i64::from(item.item_id))
            {
                return Err(invalid("invalid pet item content"));
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
    /// Package-configured actor speed copied from the pet content.
    pub speed: f32,
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
    Summon {
        pet_id: u32,
    },
    /// Client `Pet.Dismiss`/forced dismiss: action 0 with the id word left
    /// uninitialized by the client, so the id never selects a pet.
    Dismiss,
    /// One client `ItemRecord`: definition id then tint. The pair names an
    /// owned inventory instance; neither word is a pet or instance id.
    SummonByItem {
        definition_id: u32,
        tint: u32,
    },
    Rename {
        name: String,
    },
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
            4 => {
                // Client writer 00b5a650 (summon 00b60080, recall 00b601a0, dismiss
                // 00c07420/00b60610) always serializes a u32 id followed by the action
                // byte. `Pet.Dismiss` assigns only the action (00c07454) and never the
                // id word, so an action-0 request must dismiss the active pet whatever
                // four bytes precede the action.
                if body.len() != 5 {
                    return Err(invalid("pet summon request has the wrong length"));
                }
                let pet_id = word(0).ok_or_else(|| invalid("short pet summon request"))?;
                match (body[4], pet_id) {
                    (1, 0) => Err(invalid("pet summon request has a zero pet id")),
                    (1, pet_id) => Ok(Some(PetRequest::Summon { pet_id })),
                    (0, _) => Ok(Some(PetRequest::Dismiss)),
                    (_, _) => Err(invalid("unknown pet summon action")),
                }
            }
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
                let definition_id = word(0).ok_or_else(|| invalid("short pet item request"))?;
                let tint = word(4).ok_or_else(|| invalid("short pet item tint"))?;
                if definition_id == 0 {
                    return Err(invalid(
                        "pet item summon request has a zero item definition",
                    ));
                }
                Ok(Some(PetRequest::SummonByItem {
                    definition_id,
                    tint,
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

    /// S17: rebuild from the character document's `pets` value (`game_store::Snapshot::pets`).
    /// The roster is validated against the content before any session uses it.
    pub fn from_document(content: PetContent, pets: Option<serde_json::Value>) -> io::Result<Self> {
        let roster = pets
            .map(serde_json::from_value)
            .transpose()
            .map_err(|_| invalid("unreadable pet roster snapshot"))?;
        Self::from_saved(content, roster)
    }

    /// The persisted roster without the session-only active actor.
    pub fn roster_snapshot(&self) -> Option<PetRoster> {
        (!self.roster.pets.is_empty()).then(|| self.roster.clone())
    }

    /// S17: the roster as `game_store::Snapshot::pets` stores it. `None` while the roster is
    /// empty, which keeps the on-disk value; nothing removes a pet yet (see that field).
    pub fn document_value(&self) -> Option<serde_json::Value> {
        self.roster_snapshot()
            .map(|roster| serde_json::to_value(roster).expect("a pet roster always serializes"))
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
            model_id: self.content.model_id(&pet.model),
            texture_alias: self.texture_alias(pet.species_id),
            name: self.display_name(pet),
            speed: self.content.movement_speed.speed,
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
        let model_id = self.content.model_id(&model);
        broadcasts.push(PetPresenceEvent::Spawn(PetPresence {
            owner_guid,
            pet_id,
            actor_guid,
            species_id,
            model,
            model_id,
            texture_alias: self.texture_alias(species_id),
            name,
            speed: self.content.movement_speed.speed,
            position: owner_position,
        }));
        Ok(PetTransition {
            owner_message: pet_active_packet(pet_id, actor_guid),
            broadcasts,
            changed: true,
        })
    }

    pub fn dismiss(&mut self, now: u64) -> io::Result<PetTransition> {
        if self.active.is_none() {
            return Ok(PetTransition {
                owner_message: pet_active_packet(0, 0),
                broadcasts: Vec::new(),
                changed: false,
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
        })
    }

    pub fn recall(&mut self, pet_id: u32, now: u64) -> io::Result<PetTransition> {
        if self.active.map(|active| active.pet_id) != Some(pet_id) {
            return Err(invalid("that pet is not summoned"));
        }
        self.dismiss(now)
    }

    /// Route subtype 32 through the live inventory. The client sends the
    /// definition and tint of the item record it used (client `ItemRecord` +0x04
    /// and +0x08; `00a1ec00` passes them to `00b60110`); the server resolves the
    /// owned instance, adopts one new pet from it and summons that pet. It never
    /// treats either word as a pet or instance id.
    fn summon_by_item_owned(
        &mut self,
        definition_id: u32,
        tint: u32,
        actor_guid: u64,
        owner_guid: u64,
        owner_position: [f32; 3],
        inventory: &mut Inventory,
        busy: PetBusy,
        now: u64,
    ) -> io::Result<(PetTransition, Vec<Vec<u8>>)> {
        let (instance_id, item_definition) =
            owned_item_for_record(inventory.state(), definition_id, tint)?;
        // Lead (2026-10-18): every owned adoption item adopts a new pet and is consumed.
        // Pets may be owned in duplicate (Pets.md:20); an owned pet is summoned by id
        // (subtype 3). Matching an owned species here stranded later certificates.
        let definition = self
            .content
            .definition_for_item(item_definition)
            .ok_or_else(|| invalid("item is not a configured pet adoption item"))?;
        let (new_pet_id, replies) =
            self.adopt_with_replies(inventory, instance_id, &definition, now)?;
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
            PetRequest::Dismiss => {
                if next.active.is_some() {
                    next.rebase_clock(now);
                }
                next.dismiss(now)
            }
            // Redeeming an adoption item consumes it, which needs the live inventory
            // (`handle_with_inventory`); this read-only view cannot adopt.
            PetRequest::SummonByItem { .. } => {
                Err(invalid("pet adoption requires the live inventory"))
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
            definition_id,
            tint,
        } = request
        else {
            return self.handle(inner, owner_guid, owner_position, busy, now);
        };

        let actor_guid = Self::pet_guid(owner_guid);
        let mut next = self.clone();
        next.rebase_clock(now);
        let mut next_inventory = inventory.clone();
        let applied = next.summon_by_item_owned(
            definition_id,
            tint,
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
                // Actor creation precedes both the roster (whose trailing active
                // pair activates the pet) and the explicit activation.
                let mut wire = self.wire(transition.clone());
                let mut private = inventory_replies;
                private.extend(wire.shared.iter().cloned());
                private.push(self.list_packet());
                private.push(transition.owner_message);
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
            changed: transition.changed,
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
        // Actor removal and creation must precede activation: the client's
        // PetActive handler (00b58bd0 -> 00b54080) resolves the actor at once and
        // skips binding when the proxied actor does not exist yet.
        wire.private = wire.shared.clone();
        wire.private.push(transition.owner_message);
        wire
    }

    pub fn list_packet(&self) -> Vec<u8> {
        let mut bytes = pet_header(5);
        bytes.extend((self.roster.pets.len() as u32).to_le_bytes());
        for pet in &self.roster.pets {
            bytes.extend(pet.id.to_le_bytes()); // list key, before the record
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

/// Resolve one owned inventory instance from the client `ItemRecord` pair.
/// Returns `(instance id, definition id)`; a tint mismatch is not a fallback.
fn owned_item_for_record(
    inventory: &InventoryState,
    definition_id: u32,
    tint: u32,
) -> io::Result<(i32, i32)> {
    let definition_id = i32::try_from(definition_id)
        .map_err(|_| invalid("pet adoption item definition is out of range"))?;
    let tint =
        i32::try_from(tint).map_err(|_| invalid("pet adoption item tint is out of range"))?;
    inventory
        .items
        .iter()
        .find(|item| item.count > 0 && item.definition_id == definition_id && item.tint == tint)
        .map(|item| (item.id, item.definition_id))
        .ok_or_else(|| invalid("pet adoption item is not owned"))
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
            speed: self.speed,
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
