//! Saved pet roster and bounded client-shaped pet state.
//!
//! Client facts and source tags live in `data/content/pets.json`. Pet spawning,
//! care effects, need scale/rates, and server replies remain reconstructed (R)
//! because no original pet datagram was recovered.

use crate::{game_store, inventory::Inventory};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    path::Path,
};

const PET_FAMILY: u16 = 53;
const PET_CONTENT: &str = include_str!("../data/content/pets.json");

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
    updated_at: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PetRoster {
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

#[derive(Clone, Copy)]
struct ActivePet {
    pet_id: u32,
    owner_guid: u64,
    actor_guid: u64,
    follow_target: Option<[f32; 3]>,
    in_water: bool,
}

pub struct PetService {
    content: PetContent,
    roster: PetRoster,
    active: Option<ActivePet>,
}

impl PetService {
    pub fn bundled_content() -> io::Result<PetContent> {
        PetContent::bundled()
    }

    pub fn new(content: PetContent) -> io::Result<Self> {
        content.validate()?;
        Ok(Self {
            content,
            roster: PetRoster::default(),
            active: None,
        })
    }

    pub fn load_from(content: PetContent, path: &Path) -> io::Result<Self> {
        content.validate()?;
        let roster = game_store::load_pets_state(path)?.unwrap_or_default();
        validate_roster(&content, &roster)?;
        Ok(Self {
            content,
            roster,
            active: None,
        })
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
        self.roster = next;
        *inventory = next_inventory;
        Ok(id_from_consume_result(id, consumed.changed))
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
        validate_name(&name)?;
        let pet = pet_mut(&mut self.roster, pet_id)?;
        pet.name = name.clone();
        Ok(name)
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
            (pet.species_id, pet.model.clone(), pet.name.clone())
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
            follow_target: None,
            in_water: false,
        });
        broadcasts.push(PetPresenceEvent::Spawn(PetPresence {
            owner_guid,
            pet_id,
            actor_guid,
            species_id,
            model,
            name,
            position: owner_position,
        }));
        Ok(PetTransition {
            owner_message: pet_active_packet(pet_id, actor_guid),
            broadcasts,
        })
    }

    pub fn dismiss(&mut self, now: u64) -> io::Result<PetTransition> {
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
        })
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
        updated_at: now,
    });
    Ok(id)
}

fn validate_roster(content: &PetContent, roster: &PetRoster) -> io::Result<()> {
    if roster.version != 1 || roster.next_pet_id == 0 {
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

fn id_from_consume_result(id: u32, changed: bool) -> u32 {
    debug_assert!(changed, "one owned pet adoption item must be consumed");
    id
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn parse_error(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
