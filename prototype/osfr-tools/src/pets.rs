//! Saved pet roster and bounded client-shaped pet state.
//!
//! Client facts and source tags live in `data/content/pets.json`. Pet spawning,
//! care effects, need scale/rates, and server replies remain reconstructed (R)
//! because no original pet datagram was recovered.

use crate::{
    abilities::{build_effect_tag, effect_packets::encode_remove_effect_tag},
    inventory::{Inventory, InventoryState},
    startup_world, world,
    world_events::piece_052,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    sync::Arc,
};

mod grooming;
mod idle;
mod treasure;
mod tricks;

pub use idle::{PetAnimation, PetIdleSet, PetRestPosture};
pub use treasure::TreasureRefusal;

const PET_FAMILY: u16 = 53;
/// (R) Hard ceiling on a saved roster; the client can display a bounded list.
const MAX_PETS: usize = 50;
const PET_CONTENT: &str = include_str!("../data/content/pets.json");
/// (R) One active pet per owner, so an owner-derived actor id is unique while summoned.
const PET_GUID_TAG: u64 = 0x4000_0000_0000_0000;
/// T485: the pet subtypes `PetRequest::decode` accepts; the gateway's pet label reads this list.
/// T488: 18 (groom) and 22 (tactile stroke) join them.
/// T489: 7 (perform trick), 8 (gesture result) and 14 (favourite trick) join them.
pub const DECODED_SUBTYPES: [u8; 12] = [4, 6, 7, 8, 14, 16, 18, 19, 22, 24, 28, 32];
/// T485: the client's utility slots are the kit item's definition slot minus 199 (`00c077a0`):
/// shampoo 200 -> 1 .. vitamins 206 -> 7. [`KitRole`] owns both directions.
const UTILITY_SLOT_OFFSET: i32 = 199;
/// T485: the item type of every pet kit row (Legacy ClientItemDefinitions slots 200..206).
const PET_KIT_TYPE: i32 = 4;
/// T485: 53/20 PetUtilityNotify kinds (`00b5ce60` switch): begin, end + unlock, unlock.
const NOTIFY_BEGIN: u32 = 1;
const NOTIFY_END: u32 = 2;
const NOTIFY_UNLOCK: u32 = 6;
/// T488: kind 3 starts a grooming session (state word 2, `PetTactile::RefreshSpots`, a timer in
/// seconds: `00b5ec14`); kind 4 ends it and releases the spots (`00b56920`).
const NOTIFY_GROOM: u32 = 3;
const NOTIFY_GROOM_END: u32 = 4;
/// T489: kind 5 marks the pet's actor for a trick (`00b53710(1)` at `00b5ecef`; kind 6 clears it).
const NOTIFY_TRICK: u32 = 5;
/// T489 (R): sanity bound on a trick's start delay plus its run time, in ms.
const MAX_TRICK_MS: u32 = 30_000;
/// T531: the idle seed of a service that never called `set_idle_seed`; each summon mixes its
/// owner and time into it.
const IDLE_SEED: u64 = 0x5EED_1D1E;
/// Sanity cap on a care item's restore: Legacy food Param2 runs 50..400 on an unrecovered scale,
/// and the bar itself clamps at `need_policy.maximum`.
const MAX_CARE_RESTORE: f32 = 1_000.0;

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
    /// T797: every client item definition with `Type` 4 (the pet kit) in its care-panel slot, with the
    /// client locale name and `Param2`; the care path accepts only items recorded here.
    care_items: Vec<PetCareItem>,
    /// T797: the pinned client item definitions file the `care_items` rows were read from (path,
    /// commit, sha256), so a client-data drop knows which revision the kit table copies.
    #[serde(default)]
    care_items_source: Option<PetImport>,
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
    /// T455: how a summoned pet follows its owner; all (R) defaults.
    #[serde(default)]
    follow: Option<crate::pet_follow::FollowConfig>,
    /// T485: care timings and the live-needs refresh; without it feeding is refused.
    #[serde(default)]
    care: Option<CareTiming>,
    /// T533: the pet treasure hunt, its chests and its reward tables; without it a pet never digs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    treasure: Option<treasure::TreasureConfig>,
    /// T532: the happy-pet speed buff; without it a summoned pet never speeds its owner up.
    #[serde(default)]
    happy_buff: Option<HappyBuff>,
}

/// T485: when a feed applies and ends after its 53/20 begin, and how often live 53/3 needs go
/// out. All server policy (R); `valid` keeps them sane.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct CareTiming {
    feed_effect_ms: u64,
    feed_end_ms: u64,
    /// T797: the animation the pet actor plays when the food is applied. The client's 53/20
    /// begin only fires its `PetsHandler:BeginInteraction` event (`00b5ce60` case 20 kind 1);
    /// the server orders the eat animation itself, as grooming orders its dry-off. 5507 is the
    /// client `Resources/AnimationTypes.xml` slot `pet_feed`.
    feed_animation: i32,
    status_interval_ms: u64,
    status_epsilon: f32,
    /// T488: the grooming session and petting rules; without them 53/18 is refused and 53/22
    /// does nothing.
    #[serde(default)]
    groom: Option<grooming::GroomConfig>,
    /// T489: the trick session rules; without them 53/7 is refused with the unlocks.
    #[serde(default)]
    trick: Option<tricks::TrickConfig>,
    /// T531: idle life and thought bubbles; without it a pet only follows.
    #[serde(default)]
    idle: Option<idle::IdleConfig>,
    source: SourceTag,
}

impl CareTiming {
    fn valid(&self) -> bool {
        self.feed_effect_ms <= self.feed_end_ms
            && (1..=60_000).contains(&self.feed_end_ms)
            && (1..=4_000_000).contains(&self.feed_animation)
            && (100..=60_000).contains(&self.status_interval_ms)
            && self.status_epsilon.is_finite()
            && (0.01..=50.0).contains(&self.status_epsilon)
            && self.groom.as_ref().is_none_or(grooming::GroomConfig::valid)
            && self.trick.as_ref().is_none_or(tricks::TrickConfig::valid)
            && self.idle.as_ref().is_none_or(idle::IdleConfig::valid)
    }
}

/// T532: the happy-pet speed buff. Every value is server policy (R) except `name_id`, the
/// client locale row (116881 "Pet Happiness Speed Buff", hash of `Global.Text.116881`);
/// `tag_id` is server-assigned and `icon_id` is a client `Resources/Images/Images.txt` image.
/// The client draws no buff row for icon 0, and one row per `tag_id`, so a refresh replaces it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct HappyBuff {
    /// The decayed mood at which the buff holds (0..=100).
    happy_mood: f32,
    /// The owner's speed bonus in world units per second.
    speed_bonus: f32,
    /// The buff row's duration; the client counts it down.
    duration_ms: u64,
    /// How often the row is re-sent while the rule holds; under `duration_ms`.
    refresh_ms: u64,
    /// The row's instance id; above the consumable effect ids so it can never alias one.
    tag_id: i32,
    /// The client image the buff bar draws; zero draws no row at all.
    icon_id: i32,
    /// The client locale row the buff bar shows as the row's name.
    name_id: i32,
    source: SourceTag,
}

impl HappyBuff {
    fn valid(&self) -> bool {
        self.happy_mood.is_finite()
            && (0.0..=100.0).contains(&self.happy_mood)
            && self.speed_bonus.is_finite()
            && (0.1..=10.0).contains(&self.speed_bonus)
            && (1_000..=600_000).contains(&self.duration_ms)
            && (100..self.duration_ms).contains(&self.refresh_ms)
            // `consumable_effects` owns 1.0e9..1.5e9 and job/power-up rows use definition ids.
            && self.tag_id > 1_500_000_000
            && self.icon_id != 0
            && self.name_id != 0
    }
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
    /// T489: how each of the model's tricks plays (owner and pet animation groups and timing).
    #[serde(default)]
    pub trick_animations: Vec<PetTrickAnimation>,
    /// T531: the model's idle, bathroom and rest animations and its shake-off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle: Option<PetIdleSet>,
    /// T533: the model's ADR declares `pet_sniff` and `pet_digging`, so its pet plays the hunt
    /// animations (the importer checks it); any other pet hunts without them.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub digs: bool,
    src: String,
    #[serde(default)]
    unknown_reasons: BTreeMap<String, String>,
}

impl PetModel {
    /// Every animation row names one of the model's tricks, once, with positive ids and a delay
    /// plus run time within `MAX_TRICK_MS`.
    fn animations_valid(&self) -> bool {
        let mut seen = BTreeSet::new();
        self.trick_animations.iter().all(|row| {
            self.tricks.iter().any(|trick| trick.id == row.trick_id)
                && seen.insert(row.trick_id)
                && row.animation_id > 0
                && row.owner_animation_id > 0
                && !row.src.is_empty()
                && (1..=MAX_TRICK_MS).contains(&row.duration_ms)
                && row.start_delay_ms <= MAX_TRICK_MS - row.duration_ms
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetModelTrick {
    pub id: u32,
    pub known: bool,
}

/// One trick of a model: the animation group the pet plays after `start_delay_ms`, the group the
/// owner plays at once, and how long the pet's animation runs. Ids are client animation slots
/// (`AnimationTypes.xml`); `scripts/pets_import.py` checks them against the model's ADR.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PetTrickAnimation {
    pub trick_id: u32,
    pub animation_id: i32,
    pub owner_animation_id: i32,
    pub duration_ms: u32,
    pub start_delay_ms: u32,
    src: String,
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
    /// The client locale text for `name_id`. An item-only pet has no wiki species row, so this is
    /// its display name and AddNpc name (the 53/5 record still carries `name_id`).
    pub name: String,
    #[serde(default)]
    pub image_set_id: Option<i32>,
    #[serde(default)]
    pub icon_tint_id: Option<i32>,
    #[serde(default)]
    pub tint_alias: Option<String>,
    /// The client item's texture alias for this pet model; used when no wiki species row has one.
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

/// T797: one client pet kit item (`data/content/pets.json` `care_items`): a type-4 inventory
/// definition in one of the client care-panel slots. A row copies the client definition's `Slot`
/// and `Param2` and the client locale text at `name_id`; the slot decides the panel tab, the
/// action and whether the care path spends the item, so a new slot needs a decision in
/// [`KitRole`] rather than another flag on this row.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PetCareItem {
    definition_id: i32,
    /// The client locale id the client displays for the item.
    name_id: i32,
    /// The client locale text at `name_id`, verbatim (52 kit names carry the client's
    /// `{n}Name{pl="Plural"}` markup).
    label: String,
    /// The client item definition `Slot` field (200..206); the utility slot is this minus 199
    /// (`00c077a0`).
    slot: i32,
    /// The client item definition `Param2` (only foods use it: the need points one food
    /// restores). Kept as the client value; a food's must be positive.
    restore: f32,
    /// Where the row came from (client item definition row and locale id).
    src: String,
}

/// T797: the client care-panel slots, named by the panel tab each one fills
/// (`BaseClient.PetInventory` data sources `ShampooData`, `ConditionerData`, `BrushData`,
/// `FoodBowlData`, `FoodData` and `VitaminData`; locale 17765/17766/17767 name the food-bowl,
/// food and vitamin tabs). The slot is the client item definition's `Slot` field. The bowl is
/// the feed's prerequisite and the brush the grooming session's; neither is spent. A food raises
/// the food need, the grooming kit (and vitamins, per the client text at locale 35839) the groom
/// need; vitamins' own action is not routed yet (R).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KitRole {
    Shampoo,
    Conditioner,
    Brush,
    Bowl,
    Food,
    Vitamin,
}

impl KitRole {
    fn from_slot(slot: i32) -> Option<Self> {
        match slot {
            200 => Some(Self::Shampoo),
            201 => Some(Self::Conditioner),
            203 => Some(Self::Brush),
            204 => Some(Self::Bowl),
            205 => Some(Self::Food),
            206 => Some(Self::Vitamin),
            _ => None,
        }
    }

    /// The fitted utility slot: the definition slot minus [`UTILITY_SLOT_OFFSET`]
    /// (`00c077a0`), found through [`Self::from_slot`] so the fit and the action cannot
    /// disagree; the bowl is 5 and the food 6.
    fn utility_slot(self) -> usize {
        (UTILITY_SLOT_OFFSET + 1..=UTILITY_SLOT_OFFSET + 7)
            .find(|slot| Self::from_slot(*slot) == Some(self))
            .map_or(0, |slot| (slot - UTILITY_SLOT_OFFSET) as usize)
    }
}

impl PetCareItem {
    fn role(&self) -> Option<KitRole> {
        KitRole::from_slot(self.slot)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NeedPolicy {
    maximum: f32,
    /// Where a newly adopted pet's food, groom and play bars start (R), so care has work to do.
    initial: f32,
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

    /// T788: the trick ids a `perform_pet_trick` objective may name. `validate` already proved
    /// them unique and non-zero, and the quest engine reads this table instead of re-parsing the
    /// JSON, so a schema change cannot leave the reachability gate on a stale copy.
    pub fn trick_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.tricks.iter().map(|trick| trick.id)
    }

    pub fn item(&self, item_id: i32) -> Option<&PetItem> {
        self.items.iter().find(|item| item.item_id == item_id)
    }

    pub fn is_vanity(&self, model_id: i32) -> bool {
        self.model(model_id).is_some_and(|model| model.vanity)
    }

    /// The client model id for a pet ADR. Seven ADRs have two or three client ids (male/female or
    /// colour variants) and the item and species rows name only the ADR, so the lower id is the
    /// deterministic choice (R); no client field distinguishes the variants. Unknown ADRs stay zero.
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
            || self.need_policy.summoned_decay_per_second > 1.0
            || !(1.0..=self.need_policy.maximum).contains(&self.need_policy.initial)
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
        if self.follow.as_ref().is_some_and(|follow| !follow.valid()) {
            return Err(invalid("invalid pet follow content"));
        }
        if self.care.as_ref().is_some_and(|care| !care.valid()) {
            return Err(invalid("invalid pet care content"));
        }
        if self.happy_buff.as_ref().is_some_and(|buff| !buff.valid()) {
            return Err(invalid("invalid pet happy-buff content"));
        }
        if self
            .treasure
            .as_ref()
            .is_some_and(|treasure| !treasure.valid())
        {
            return Err(invalid("invalid pet treasure content"));
        }
        let mut model_ids = BTreeSet::new();
        for model in &self.models {
            if model.model_id <= 0
                || !valid_model(&model.adr)
                || model.src.is_empty()
                || !model_ids.insert(model.model_id)
                || model.tricks.len() > 16
                || model.item_classes.len() > 16
                || !model.animations_valid()
                || !model.idle.as_ref().is_none_or(idle::PetIdleSet::valid)
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
                || item.name.is_empty()
                || item.name.chars().count() > 64
                || item.name.chars().any(char::is_control)
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
            let role = item.role();
            if item.definition_id <= 0
                || item.name_id <= 0
                || item.label.is_empty()
                || item.label.chars().count() > 200
                || item.label.chars().any(char::is_control)
                || role.is_none()
                || !item.restore.is_finite()
                || !(0.0..=MAX_CARE_RESTORE).contains(&item.restore)
                // Only a food's Param2 is applied; a food with no restore would be a no-op.
                || (role == Some(KitRole::Food) && item.restore <= 0.0)
                || item.src.is_empty()
                || !item_ids.insert(item.definition_id)
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
    // The record order (T485): PetListDataSource's getter `00ccc680` reads FoodNeed at +0x20,
    // GroomNeed +0x24, PlayNeed +0x28 and Mood +0x2c.
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
    /// A set, not the client's order: the model's `tricks` rows keep the Pets.json `Unknown8` order
    /// for the 53/5 record, and this holds learned ids that model row does not list (old saves).
    pub learned_tricks: BTreeSet<u32>,
    pub favorite_tricks: [Option<u32>; 4],
    pub equipment: PetEquipment,
    /// The inventory definition consumed to adopt this pet, when one was required.
    #[serde(default)]
    pub adoption_item_definition_id: Option<i32>,
    /// T456: the adoption item instance's tint; `None` uses the model's default tint. Left out of
    /// the save when unset, so an older roster round-trips byte for byte (l043 admin move).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<i32>,
    /// T533: when this pet last unearthed a treasure chest (Unix seconds, the wall clock, not the
    /// session clock, so the cooldown survives a restart). Left out of the save until it digs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treasure_found_at: Option<u64>,
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
    /// T531: the rest loop the pet is lying in, as a base animation; a late viewer is sent it
    /// with the spawn so it does not see a standing pet during the rest. `None` while standing.
    pub base_animation: Option<i32>,
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
    /// T485 `Pet.EquipUtility`/`UnequipUtility` (writer `00c06ea0`): u8 action (0 fit,
    /// 1 remove), the active pet id, the utility slot and the Lua item id, the same key the
    /// client passes to `Ui.ClearItemNewFlag` (ScriptsBase.lua `equipItem`), i.e. the owned
    /// inventory instance (R).
    Utility {
        remove: bool,
        pet_id: u32,
        slot: u32,
        item: u32,
    },
    /// T485 `Pet.StartFeed` (`00b5f720`): the active pet id and a zero word.
    Feed {
        pet_id: u32,
    },
    /// T485 53/28 (`00b60350`/`00b60420`): the owner entered (true) or left water.
    Water(bool),
    /// T488 `Pet.StartGroom` (`00b5f8c0`, packet ctor `00b51610`): the active pet id and a zero word.
    Groom {
        pet_id: u32,
    },
    /// T488 53/22 PetTactileNotify (sender `00ccebc0`, packet ctor `00ccdb70`): the client's
    /// stroke report. Four u32: the pet handler's state word (`mode`: 2 while grooming, else 0), a
    /// constant 3, the stroke time in ms and the movement sum, raw (the handler clamps them).
    Tactile {
        mode: u32,
        duration_ms: u32,
        movement: u32,
    },
    /// T489 `Pet.PerformTrick` (`00c06890` -> `00b5fb30` -> `UnlockPetTricksAfterGesture`
    /// `00cc7420`, or `BeginPetGestureRound` `00cc9330`; writer `00b5ff50`, ctor `00b50a10`): the
    /// trick id. No pet id: the active pet performs it.
    PerformTrick {
        trick_id: u32,
    },
    /// T489 53/8 `PetTrickGestureResult` (writer `00b5ffe0`, ctor `00b50c20`; sent right after 53/7
    /// by `00cc7420` and at the end of the drawing round by `FinishPetGestureState` `00cc9a90`):
    /// the trick id and no grade.
    TrickResult {
        trick_id: u32,
    },
    /// T489 `Pet.SetFavoriteTrick` (`00b5f670`, ctor `00b51290`): the active pet id, the slot as
    /// the sender computed it (the one-based UI slot minus one, so 0 to 3) and the trick id.
    Favorite {
        pet_id: u32,
        slot: u32,
        trick_id: u32,
    },
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
            16 => {
                if body.len() != 13 {
                    return Err(invalid("pet utility request has the wrong length"));
                }
                let remove = match body[0] {
                    0 => false,
                    1 => true,
                    _ => return Err(invalid("unknown pet utility action")),
                };
                let word = |offset: usize| word(offset).ok_or_else(|| invalid("short pet utility"));
                Ok(Some(PetRequest::Utility {
                    remove,
                    pet_id: word(1)?,
                    slot: word(5)?,
                    item: word(9)?,
                }))
            }
            19 => {
                if body.len() != 8 {
                    return Err(invalid("pet feed request has the wrong length"));
                }
                let pet_id = word(0).ok_or_else(|| invalid("short pet feed request"))?;
                Ok(Some(PetRequest::Feed { pet_id }))
            }
            28 => match body {
                [0] => Ok(Some(PetRequest::Water(false))),
                [1] => Ok(Some(PetRequest::Water(true))),
                _ => Err(invalid("pet water request has the wrong length or value")),
            },
            18 => {
                if body.len() != 8 {
                    return Err(invalid("pet groom request has the wrong length"));
                }
                let pet_id = word(0).ok_or_else(|| invalid("short pet groom request"))?;
                Ok(Some(PetRequest::Groom { pet_id }))
            }
            22 => {
                if body.len() != 16 {
                    return Err(invalid("pet tactile request has the wrong length"));
                }
                let word = |offset: usize| word(offset).ok_or_else(|| invalid("short pet tactile"));
                Ok(Some(PetRequest::Tactile {
                    mode: word(0)?,
                    duration_ms: word(8)?,
                    movement: word(12)?,
                }))
            }
            7 | 8 => {
                if body.len() != 4 {
                    return Err(invalid("pet trick request has the wrong length"));
                }
                let trick_id = word(0).ok_or_else(|| invalid("short pet trick request"))?;
                Ok(Some(if subtype == 7 {
                    PetRequest::PerformTrick { trick_id }
                } else {
                    PetRequest::TrickResult { trick_id }
                }))
            }
            14 => {
                if body.len() != 12 {
                    return Err(invalid("pet favorite request has the wrong length"));
                }
                let word =
                    |offset: usize| word(offset).ok_or_else(|| invalid("short pet favorite"));
                Ok(Some(PetRequest::Favorite {
                    pet_id: word(0)?,
                    slot: word(4)?,
                    trick_id: word(8)?,
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
    /// T489: the trick id of a 53/7 the server accepted, set when it is accepted, not when the
    /// animation ends. A refusal, a late 53/8 and every tick leave it `None`, so a caller (the
    /// T788 quest objective) can credit exactly the accepted performs.
    pub accepted_trick: Option<u32>,
}

#[derive(Clone, Copy)]
struct ActivePet {
    pet_id: u32,
    owner_guid: u64,
    actor_guid: u64,
    position: [f32; 3],
    follow_target: Option<[f32; 3]>,
    in_water: bool,
    /// T455: session-only follow pose; never saved, never part of the roster.
    pose: crate::pet_follow::PetPose,
    /// T485: the running feed (session only; dismiss, zoning or disconnect drop it).
    feeding: Option<Feeding>,
    /// T485: the needs last sent as a live 53/3, and when (ms).
    shown: Option<(PetNeeds, u64)>,
    /// T488: the running grooming session (session only, like `feeding`).
    grooming: Option<grooming::Grooming>,
    /// T488: when petting last raised play (ms); the rate limit reads it. Session only.
    petted_ms: Option<u64>,
    /// T489: the running trick, or the one that just ended (session only, like `feeding`).
    trick: tricks::TrickState,
    /// T531: idle animations, rest posture, shake-off and thought-bubble timers (session only).
    idle: idle::IdleState,
    /// T533: where the pet is in its treasure hunt (session only).
    treasure: treasure::Hunt,
}

impl ActivePet {
    /// A feed, a grooming session or a trick is running.
    fn caring(&self) -> bool {
        self.feeding.is_some() || self.grooming.is_some() || self.trick.running()
    }

    /// Care, or (T533) a sniff or a dig: no other care action may start.
    fn interacting(&self) -> bool {
        self.caring() || self.treasure.working()
    }
}

/// T532: the live happy-speed buff (session-only; never saved, dropped on reconnect).
#[derive(Clone, Copy, Debug, PartialEq)]
struct HappyBuffLive {
    /// When the next row refresh is due (ms). The row itself already carries its duration.
    refresh_at_ms: u64,
}

/// T485: a feed between its 53/20 begin and end.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Feeding {
    started_ms: u64,
    /// The fitted food instance, consumed once at the apply step; `None` after it.
    food: Option<i32>,
}

#[derive(Clone)]
pub struct PetService {
    content: Arc<PetContent>,
    roster: PetRoster,
    active: Option<ActivePet>,
    ui_mode: u32,
    /// T487: the pet a zone change left behind, for the destination's loaded signal to bring back.
    /// Session only: never saved and never sent. `Some` means no pet is active, because a summon
    /// and a dismiss both clear it; `reset_session` drops it with the connection.
    remembered: Option<u32>,
    /// T531: seeds the idle choices of the next summon (`set_idle_seed`).
    idle_seed: u64,
    /// T533: the treasure generator and the live chest (session only).
    treasure: treasure::State,
    /// T532: the live happy-speed buff; session-only, so `reset_session` drops it.
    happy: Option<HappyBuffLive>,
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
            remembered: None,
            idle_seed: IDLE_SEED,
            treasure: treasure::State::new(),
            happy: None,
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
            remembered: None,
            idle_seed: IDLE_SEED,
            treasure: treasure::State::new(),
            happy: None,
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

    /// T488: whether a grooming session (a timed occupation) is running on the summoned pet.
    pub fn is_grooming(&self) -> bool {
        self.active.is_some_and(|active| active.grooming.is_some())
    }

    pub fn ui_mode(&self) -> u32 {
        self.ui_mode
    }

    /// The wiki species texture alias, else the adoption item's (an item-only pet has no species row).
    fn texture_alias(&self, pet: &Pet) -> String {
        self.content
            .species
            .iter()
            .find(|species| species.species_id == Some(pet.species_id))
            .and_then(|species| species.texture_alias.clone())
            .or_else(|| {
                self.adoption_item(pet)
                    .and_then(|item| item.texture_alias.clone())
            })
            .unwrap_or_default()
    }

    /// The chosen name, else the wiki species name, else the adoption item's client name. An
    /// item-only species id is the item id, so the item row is the only name source it has.
    fn display_name(&self, pet: &Pet) -> String {
        if !pet.name.is_empty() {
            return pet.name.clone();
        }
        if let Some(species) = self
            .content
            .species
            .iter()
            .find(|species| species.species_id == Some(pet.species_id))
        {
            return species.name.clone();
        }
        self.adoption_item(pet)
            .map(|item| item.name.clone())
            .unwrap_or_default()
    }

    /// The item definition this pet was adopted from, when the save records one.
    fn adoption_item(&self, pet: &Pet) -> Option<&PetItem> {
        pet.adoption_item_definition_id
            .and_then(|id| self.content.item(id))
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
            texture_alias: self.texture_alias(pet),
            name: self.display_name(pet),
            speed: self.content.movement_speed.speed,
            position: active.position,
            base_animation: active.idle.base_animation(),
        })
    }

    /// Reset only the connection-scoped actor state; the saved roster stays. T487: the pet a zone
    /// change left behind is session state too, so a disconnect or reconnect forgets it. T532: the
    /// happy pet's buff is connection-scoped too, so a reconnect starts without it.
    pub fn reset_session(&mut self) {
        self.active = None;
        self.ui_mode = 0;
        self.remembered = None;
        self.happy = None;
        // T533: a chest belongs to one connection; the pet's cooldown stays in the roster.
        self.clear_treasure();
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
        let item_tint = item.tint;
        let mut next = self.roster.clone();
        let id = next.next_pet_id;
        insert_pet(&self.content, &mut next, definition, now)?;
        // F767: the owned unit is the tint's source, so every adoption path keeps it.
        if let Some(pet) = next.pets.last_mut() {
            pet.tint = (item_tint != 0).then_some(item_tint);
        }
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

    /// T772: how many more pets the roster holds.
    pub fn room(&self) -> usize {
        MAX_PETS.saturating_sub(self.roster.pets.len())
    }

    /// T772: whether `item_definition_id` creates a pet when granted. One binding answers both
    /// the coin store's pet rows and this adoption, so a data change moves both together.
    pub fn is_adoption_item(&self, item_definition_id: i32) -> bool {
        self.content
            .definition_for_item(item_definition_id)
            .is_some()
    }

    /// T772: the inventory stacks a buy or grant added (`granted` as `(instance id, units)`)
    /// become pets at once. The original is thought to create the pet at purchase (R); here the
    /// bag hides category-200 rows (`00a6d9b0`) and the Pets window reads only the 53/5 roster
    /// (`00b596f0`). Each unit is consumed into its own pet; a unit past the roster's room stays
    /// owned. Returns the inventory replies and the 53/5 roster, or nothing when no pet was
    /// adopted.
    pub fn adopt_granted(
        &mut self,
        granted: &[(i32, u32)],
        inventory: &mut Inventory,
        now: u64,
    ) -> io::Result<Vec<Vec<u8>>> {
        let mut replies = Vec::new();
        let mut room = self.room();
        for &(instance_id, units) in granted {
            if room == 0 {
                break;
            }
            let Some(item) = inventory
                .state()
                .items
                .iter()
                .find(|item| item.id == instance_id)
            else {
                continue;
            };
            let item_definition = item.definition_id;
            let Some(definition) = self.content.definition_for_item(item_definition) else {
                continue;
            };
            for _ in 0..units.min(room as u32) {
                let (_, adopted) =
                    self.adopt_with_replies(inventory, instance_id, &definition, now)?;
                replies.extend(adopted);
                room -= 1;
            }
        }
        if !replies.is_empty() {
            replies.push(self.list_packet());
        }
        Ok(replies)
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
        let (species_id, model, name, texture_alias) = {
            let pet = self
                .pet(pet_id)
                .ok_or_else(|| invalid("pet is not owned"))?;
            (
                pet.species_id,
                pet.model.clone(),
                self.display_name(pet),
                self.texture_alias(pet),
            )
        };
        self.advance_time(now)?;
        let mut broadcasts = Vec::new();
        if let Some(active) = self.active {
            broadcasts.push(PetPresenceEvent::Despawn {
                owner_guid: active.owner_guid,
                actor_guid: active.actor_guid,
            });
        }
        // T455: the pet appears beside its owner, not inside it (the first follow step snaps it
        // to the ground). Content without a follow block keeps the owner's position.
        let pose = self.content.follow.as_ref().map_or_else(
            || crate::pet_follow::spawn_at(owner_position),
            |follow| crate::pet_follow::spawn(follow, owner_position),
        );
        // T487: a summoned pet is out, so nothing is left behind for a zone change to bring back.
        self.remembered = None;
        self.active = Some(ActivePet {
            pet_id,
            owner_guid,
            actor_guid,
            position: pose.position,
            follow_target: None,
            in_water: false,
            pose,
            feeding: None,
            shown: None,
            grooming: None,
            petted_ms: None,
            trick: tricks::TrickState::Idle,
            idle: idle::IdleState::new(self.idle_seed ^ owner_guid.rotate_left(32) ^ now),
            treasure: treasure::Hunt::default(),
        });
        let model_id = self.content.model_id(&model);
        broadcasts.push(PetPresenceEvent::Spawn(PetPresence {
            owner_guid,
            pet_id,
            actor_guid,
            species_id,
            model,
            model_id,
            texture_alias,
            name,
            speed: self.content.movement_speed.speed,
            // F762: the AddNpc places the pet where its pose is (beside the owner), not on them.
            position: pose.position,
            // A new summon stands, so it has no rest loop to replay.
            base_animation: None,
        }));
        Ok(PetTransition {
            owner_message: pet_active_packet(pet_id, actor_guid),
            broadcasts,
            changed: true,
        })
    }

    pub fn dismiss(&mut self, now: u64) -> io::Result<PetTransition> {
        // T487: a dismiss is explicit or forced (a recalled pet, a disconnect), so it also drops
        // the pet a zone change would bring back.
        self.remembered = None;
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

    /// T487: the pet a zone change left behind and the destination has not summoned back. Session
    /// state only (never saved); the tests read it to pin the forget rules.
    pub fn remembered_pet_id(&self) -> Option<u32> {
        self.remembered
    }

    pub fn recall(&mut self, pet_id: u32, now: u64) -> io::Result<PetTransition> {
        if self.active.map(|active| active.pet_id) != Some(pet_id) {
            return Err(invalid("that pet is not summoned"));
        }
        self.dismiss(now)
    }

    /// T487: a zone change dismisses the active pet as before but remembers it, so the
    /// destination's loaded signal can bring it back (`resummon`). `settle_pets` is the only
    /// caller: an explicit or forced dismiss (`dismiss`, `recall`) still forgets it.
    pub fn zone_away(&mut self, now: u64) -> io::Result<PetTransition> {
        let remembered = self.active.map(|active| active.pet_id);
        let transition = self.dismiss(now)?;
        self.remembered = remembered;
        Ok(transition)
    }

    /// T487: bring back the pet a zone change left behind through the normal summon path, beside
    /// its owner. `None` while a pet is out, while an activity closes the pets gate (the memory is
    /// then kept for the first update or tick after it ends), for a pet the roster no longer holds
    /// (forgotten silently) and with nothing remembered. The wire is the summon's own plus the
    /// roster a summon sends, and it never marks a save: only the session remembers the pet.
    pub fn resummon(
        &mut self,
        actor_guid: u64,
        owner_guid: u64,
        owner_position: [f32; 3],
        busy: PetBusy,
        now: u64,
    ) -> io::Result<Option<PetWire>> {
        let Some(pet_id) = self.remembered else {
            return Ok(None);
        };
        if busy.minigame_active || self.active.is_some() {
            return Ok(None);
        }
        // Taken before the attempt: a pet the roster lost, or a summon the server cannot place
        // (an unusable owner pose), must not be retried on every tick.
        self.remembered = None;
        if self.pet(pet_id).is_none() {
            return Ok(None);
        }
        let transition = self.summon(pet_id, actor_guid, owner_guid, owner_position, busy, now)?;
        let mut wire = self.wire(transition);
        wire.private.push(self.list_packet());
        wire.changed = false;
        Ok(Some(wire))
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
        let summoning = matches!(request, PetRequest::Summon { .. });
        // T485: replacing or dismissing the active pet ends its feed (nothing more is consumed).
        // T488: a grooming session. T489: a trick (the unlocks).
        let care_end = (summoning || matches!(request, PetRequest::Dismiss))
            .then(|| self.care_end_packets())
            .unwrap_or_default();
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
            // (`handle_at_ms`); this read-only view cannot adopt.
            PetRequest::SummonByItem { .. } => {
                Err(invalid("pet adoption requires the live inventory"))
            }
            // T485: fitting and feeding read and spend the live inventory (`handle_at_ms`).
            // T488: so do grooming and tactile strokes (`handle_at_ms`). T489: tricks and
            // favourites run on the same route, with the millisecond clock.
            PetRequest::Utility { .. }
            | PetRequest::Feed { .. }
            | PetRequest::Groom { .. }
            | PetRequest::Tactile { .. }
            | PetRequest::PerformTrick { .. }
            | PetRequest::TrickResult { .. }
            | PetRequest::Favorite { .. } => Err(invalid("pet care requires the live inventory")),
            // The client reports water whether or not a pet is out; only a summoned pet keeps it.
            // T488: entering water ends a grooming session. T489: and a trick.
            PetRequest::Water(in_water) => {
                let mut wire = PetWire::default();
                if self.active.is_some() {
                    self.set_in_water(in_water)?;
                    if in_water {
                        wire = self.stop_groom();
                        wire.private.extend(self.stop_trick());
                    }
                }
                return Ok(Some(wire));
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
                    shared: vec![pet_set_name_packet(active.actor_guid, &name, true)],
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
                let mut wire = self.wire(transition);
                if summoning {
                    wire.private.push(self.list_packet());
                }
                wire.private.extend(care_end);
                Ok(Some(wire))
            }
            Err(error) => Ok(Some(PetWire {
                private: vec![self.list_packet()],
                notice: Some(refusal_text(&error)),
                ..PetWire::default()
            })),
        }
    }

    /// Handle pet requests with the character's live inventory on a clock in whole seconds.
    /// The gateway route is `handle_at_ms`; the tests that predate the millisecond care timers
    /// use this wrapper. Every other request uses the same staged path as `handle`.
    pub fn handle_at_seconds(
        &mut self,
        inner: &[u8],
        owner_guid: u64,
        owner_position: [f32; 3],
        inventory: &mut Inventory,
        busy: PetBusy,
        now: u64,
    ) -> io::Result<Option<PetWire>> {
        self.handle_at_ms(
            inner,
            owner_guid,
            owner_position,
            inventory,
            busy,
            now.saturating_mul(1_000),
        )
    }

    /// The gateway route with the session's millisecond clock: care timers (T485) run in ms,
    /// need clocks in whole seconds.
    pub fn handle_at_ms(
        &mut self,
        inner: &[u8],
        owner_guid: u64,
        owner_position: [f32; 3],
        inventory: &mut Inventory,
        busy: PetBusy,
        now_ms: u64,
    ) -> io::Result<Option<PetWire>> {
        let Some(request) = PetRequest::decode(inner)? else {
            return Ok(None);
        };
        let now = now_ms / 1_000;
        let (definition_id, tint) = match request {
            PetRequest::SummonByItem {
                definition_id,
                tint,
            } => (definition_id, tint),
            PetRequest::Utility {
                remove,
                pet_id,
                slot,
                item,
            } => return Ok(Some(self.utility(remove, pet_id, slot, item, inventory))),
            PetRequest::Feed { pet_id } => {
                return Ok(Some(self.start_feed(pet_id, inventory, busy, now_ms)));
            }
            PetRequest::Groom { pet_id } => {
                return Ok(Some(self.start_groom(pet_id, inventory, busy, now_ms)));
            }
            PetRequest::Tactile {
                mode,
                duration_ms,
                movement,
            } => {
                return Ok(Some(self.tactile(
                    mode,
                    duration_ms,
                    movement,
                    busy,
                    now_ms,
                )));
            }
            PetRequest::PerformTrick { trick_id } => {
                return Ok(Some(self.perform_trick(trick_id, busy, now_ms)));
            }
            PetRequest::TrickResult { trick_id } => {
                return Ok(Some(self.trick_result(trick_id, now_ms)));
            }
            // A refused favourite sends nothing: the client sets the slot only from the echo.
            PetRequest::Favorite {
                pet_id,
                slot,
                trick_id,
            } => {
                return Ok(Some(self.set_favorite(pet_id, slot, trick_id).map_or_else(
                    |_| PetWire::default(),
                    |echo| PetWire {
                        private: vec![echo],
                        changed: true,
                        ..PetWire::default()
                    },
                )));
            }
            _ => return self.handle(inner, owner_guid, owner_position, busy, now),
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
                // F778: the adopted pet replaces the active one, which drops a running feed,
                // grooming session or trick. End them like a normal summon does; the client's
                // care panel only unlocks on 53/20 kinds 2, 4 and 6.
                let care_end = self.care_end_packets();
                *self = next;
                *inventory = next_inventory;
                // Actor creation precedes both the roster (whose trailing active
                // pair activates the pet) and the explicit activation.
                let mut wire = self.wire(transition.clone());
                let mut private = inventory_replies;
                private.extend(wire.shared.iter().cloned());
                private.push(self.list_packet());
                private.push(transition.owner_message);
                private.extend(care_end);
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
                // T455: a follow click moves the pet to the spot with the enemy 125 writer (R).
                PetPresenceEvent::Follow { actor_guid, target } => {
                    let heading = self.active.map_or([0.0, 0.0, 1.0], |active| {
                        let d = [
                            target[0] - active.position[0],
                            target[2] - active.position[2],
                        ];
                        let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
                        if n > 1e-4 {
                            [d[0] / n, 0.0, d[1] / n]
                        } else {
                            [0.0, 0.0, 1.0]
                        }
                    });
                    wire.shared
                        .push(crate::enemies::movement_packet(actor_guid, target, heading));
                }
            }
        }
        // Actor removal and creation must precede activation: the client's
        // PetActive handler (00b58bd0 -> 00b54080) resolves the actor at once and
        // skips binding when the proxied actor does not exist yet.
        wire.private = wire.shared.clone();
        wire.private.push(transition.owner_message);
        wire
    }

    /// T455: move the summoned pet after its owner (an accepted owner update or a world tick).
    /// The 125 frames go to the owner (`private`) and to viewers (`shared`). Never marks a save.
    pub fn follow_owner(
        &mut self,
        owner_guid: u64,
        owner_position: [f32; 3],
        ground: impl Fn(f32, f32) -> Option<f32>,
        now_ms: u64,
    ) -> PetWire {
        let mut wire = PetWire::default();
        let (Some(follow), Some(active)) = (self.content.follow.as_ref(), self.active.as_mut())
        else {
            return wire;
        };
        if active.owner_guid != owner_guid {
            return wire;
        }
        let owner = crate::pet_follow::OwnerPose {
            position: owner_position,
            at_ms: now_ms,
        };
        let (pose, step) =
            crate::pet_follow::step(follow, owner, active.pose, ground, self.ui_mode != 0);
        active.pose = pose;
        active.position = pose.position;
        if let Some(step) = step {
            let packet =
                crate::enemies::movement_packet(active.actor_guid, step.position, step.heading);
            wire.private.push(packet.clone());
            wire.shared.push(packet);
        }
        wire
    }

    /// T485: fit (53/16 action 0) or remove (action 1) a kit item in utility slot 1..=7. A fit
    /// needs an owned pet that is not vanity (R) and an owned type-4 item whose definition slot
    /// is `slot + 199` and whose class the pet's model lists. A refusal of a slot in 1..=7 echoes
    /// 53/16 with the slot as it was (nothing changes). F778: a slot outside 1..=7 echoes nothing
    /// at all, because the client's 53/16 handler (`00b5ce60` case 16, write at `00b5ea69`)
    /// stores the echoed slot into its pet record without a bounds check.
    fn utility(
        &mut self,
        remove: bool,
        pet_id: u32,
        slot: u32,
        item: u32,
        inventory: &Inventory,
    ) -> PetWire {
        let Some(index) = usize::try_from(slot)
            .ok()
            .filter(|index| (1..=7).contains(index))
        else {
            return PetWire::default();
        };
        let echo = |fitted: Option<i32>| PetWire {
            private: vec![pet_utility_packet(
                fitted.is_none(),
                pet_id,
                slot,
                fitted.map_or(0, |id| id as u32),
            )],
            ..PetWire::default()
        };
        let before = self
            .pet(pet_id)
            .and_then(|pet| pet.equipment.utilities[index]);
        let accepted = match self.pet(pet_id) {
            Some(_) if remove => Some(None),
            Some(pet) => self.kit_item(pet, index, item, inventory).map(Some),
            None => None,
        };
        let Some(next) = accepted else {
            return echo(before);
        };
        if let Ok(pet) = pet_mut(&mut self.roster, pet_id) {
            pet.equipment.utilities[index] = next;
        }
        PetWire {
            changed: next != before,
            ..echo(next)
        }
    }

    /// T485: the owned instance `item` when it may be fitted to `pet`'s utility slot `index`.
    fn kit_item(&self, pet: &Pet, index: usize, item: u32, inventory: &Inventory) -> Option<i32> {
        let model = self.content.model(self.content.model_id(&pet.model))?;
        let item = i32::try_from(item).ok()?;
        let owned = inventory
            .state()
            .items
            .iter()
            .find(|owned| owned.id == item && owned.count > 0)?;
        let (item_type, slot, class) = inventory.kit_facts(owned.definition_id)?;
        (!model.vanity
            && item_type == PET_KIT_TYPE
            && slot == index as i32 + UTILITY_SLOT_OFFSET
            && model.item_classes.iter().any(|known| known.id == class))
        .then_some(item)
    }

    /// T485: 53/19 starts a feed and answers 53/20 begin; `care_tick` consumes the food later.
    /// Every refusal answers 53/20 unlock instead, so the client's care panel never sticks.
    fn start_feed(
        &mut self,
        pet_id: u32,
        inventory: &Inventory,
        busy: PetBusy,
        now_ms: u64,
    ) -> PetWire {
        let food = self.feedable(pet_id, inventory, busy);
        if let (Some(food), Some(active)) = (food, self.active.as_mut()) {
            active.feeding = Some(Feeding {
                started_ms: now_ms,
                food: Some(food),
            });
        }
        let kind = if food.is_some() {
            NOTIFY_BEGIN
        } else {
            NOTIFY_UNLOCK
        };
        PetWire {
            private: vec![pet_utility_notify_packet(pet_id, kind, 0)],
            ..PetWire::default()
        }
    }

    /// The fitted food a feed would eat: the pet summoned and not vanity (R), the owner not in
    /// water or busy, no feed running, a recorded bowl in slot 5 and a recorded food in slot 6.
    /// T797: both must be client kit rows in their slot, so a stale save that left some other
    /// item in a utility slot cannot feed it.
    fn feedable(&self, pet_id: u32, inventory: &Inventory, busy: PetBusy) -> Option<i32> {
        let active = self.active.filter(|active| active.pet_id == pet_id)?;
        if self.content.care.is_none()
            || busy.minigame_active
            || active.in_water
            || active.interacting()
        {
            return None;
        }
        let pet = self.pet(pet_id)?;
        if self
            .content
            .model(self.content.model_id(&pet.model))
            .is_some_and(|model| model.vanity)
        {
            return None;
        }
        let bowl = fitted_item(pet, inventory, KitRole::Bowl.utility_slot())?;
        self.is_kit(bowl.definition_id, KitRole::Bowl)
            .then_some(())?;
        let food = fitted_item(pet, inventory, KitRole::Food.utility_slot())?;
        self.is_kit(food.definition_id, KitRole::Food)
            .then_some(food.id)
    }

    /// Whether `definition_id` is a recorded kit item of `role`.
    fn is_kit(&self, definition_id: i32, role: KitRole) -> bool {
        self.content
            .care_items
            .iter()
            .any(|rule| rule.definition_id == definition_id && rule.role() == Some(role))
    }

    /// T485: the summoned pet's care on the world tick. A feed applies at `feed_effect_ms` (one
    /// food consumed through `care`, the client's `pet_feed` animation ordered, `changed` set) and
    /// ends with 53/20 kind 2 at `feed_end_ms`. T488: a grooming session that ran out its time
    /// ends here with its final needs, the dry-off animation and 53/20 kind 4 (`changed`, the
    /// gains are saved). Live needs:
    /// a decayed view (the roster is not written) goes out as 53/3
    /// once it moved by `status_epsilon` and `status_interval_ms` has passed. T531: a settled pet's
    /// idle animations and its thought bubbles ride the same tick (`idle_tick`). F778: an apply that
    /// emptied the fitted food's stack also echoes that slot as a 53/16 clear, so the care panel
    /// does not keep a ghost food. A failure is a refusal for this player, never an error out of the
    /// tick.
    pub fn care_tick(&mut self, inventory: &mut Inventory, now_ms: u64) -> PetWire {
        let mut wire = PetWire::default();
        // The content is shared (`Arc`): holding it costs no copy on this per-tick path.
        let content = Arc::clone(&self.content);
        let (Some(care), Some(active)) = (content.care.as_ref(), self.active) else {
            return wire;
        };
        if let Some(mut feeding) = active.feeding {
            let elapsed = now_ms.saturating_sub(feeding.started_ms);
            if let Some(food) = feeding.food.filter(|_| elapsed >= care.feed_effect_ms) {
                feeding.food = None;
                let fitted = self.pet(active.pet_id).map(|pet| pet.equipment.utilities);
                // A food that left the bag consumes nothing; the feed still ends on time.
                if let Ok(outcome) = self.care(
                    active.pet_id,
                    food,
                    inventory,
                    PetBusy::default(),
                    now_ms / 1_000,
                ) {
                    wire.private.extend(outcome.inventory_replies);
                    wire.private.push(outcome.pet_status_packet);
                    // T797: the 53/20 begin only fires the client's own
                    // `PetsHandler:BeginInteraction` (`00b5ce60` case 20 kind 1), so the server
                    // orders the eat animation with the apply, like grooming's dry-off. It goes
                    // to the owner and to the viewers of the pet actor.
                    let eat =
                        piece_052::animation_packet(active.actor_guid, care.feed_animation, false);
                    wire.private.push(eat.clone());
                    wire.shared.push(eat);
                    wire.changed = true;
                    // F778: the last unit clears its fitted slot in `care` (`spend_one`); the
                    // care panel still shows the food the 53/16 echo told it about (`00b5ce60`
                    // case 16), so clear those slots on the client too.
                    if let (Some(fitted), Some(pet)) = (fitted, self.pet(active.pet_id)) {
                        for (index, was) in fitted.iter().enumerate() {
                            if *was == Some(food) && pet.equipment.utilities[index].is_none() {
                                wire.private.push(pet_utility_packet(
                                    true,
                                    active.pet_id,
                                    index as u32,
                                    0,
                                ));
                            }
                        }
                    }
                    let needs = self.pet(active.pet_id).map(|pet| pet.needs);
                    if let (Some(active), Some(needs)) = (self.active.as_mut(), needs) {
                        active.shown = Some((needs, now_ms));
                    }
                }
            }
            let running = elapsed < care.feed_end_ms;
            if !running {
                wire.private
                    .push(pet_utility_notify_packet(active.pet_id, NOTIFY_END, 0));
            }
            if let Some(active) = self.active.as_mut() {
                active.feeding = running.then_some(feeding);
            }
        }
        if let Some(groom) = care.groom.as_ref() {
            self.groom_tick(groom, &mut wire, now_ms);
        }
        if let Some(trick) = care.trick.as_ref() {
            self.trick_tick(trick, &mut wire, now_ms);
        }
        if let Some(idle) = care.idle.as_ref() {
            self.idle_tick(idle, &mut wire, now_ms);
        }
        if let Some(status) = self.live_needs(care, now_ms) {
            wire.private.push(status);
        }
        wire
    }

    /// The live 53/3 for the summoned pet when its decayed needs are due to be shown.
    fn live_needs(&mut self, care: &CareTiming, now_ms: u64) -> Option<Vec<u8>> {
        let active = self.active?;
        let pet = self.pet(active.pet_id)?;
        let needs = decayed(&self.content.need_policy, pet, true, now_ms / 1_000)?;
        let moved = |shown: &PetNeeds| {
            [
                shown.food - needs.food,
                shown.groom - needs.groom,
                shown.play - needs.play,
                shown.mood - needs.mood,
            ]
            .iter()
            .fold(0.0_f32, |most, delta| most.max(delta.abs()))
        };
        let due = active.shown.map_or(true, |(shown, at)| {
            now_ms.saturating_sub(at) >= care.status_interval_ms
                && moved(&shown) >= care.status_epsilon
        });
        if !due {
            return None;
        }
        self.active.as_mut()?.shown = Some((needs, now_ms));
        Some(pet_status_packet(active.pet_id, &needs))
    }

    /// T485: the 53/20 end for a feed (kind 2) or, T488, a grooming session (kind 4) still running,
    /// and, T489, the unlocks of a running trick (a dismiss, re-summon or zone change ends it).
    pub fn care_end_packets(&self) -> Vec<Vec<u8>> {
        let Some(active) = self.active else {
            return Vec::new();
        };
        let kind = match (active.grooming, active.feeding) {
            (Some(_), _) => Some(NOTIFY_GROOM_END),
            (None, Some(_)) => Some(NOTIFY_END),
            (None, None) => None,
        };
        kind.map(|kind| pet_utility_notify_packet(active.pet_id, kind, 0))
            .into_iter()
            .chain(self.trick_end_packets())
            .collect()
    }

    /// T532: the live happy-pet speed bonus in world units per second, `None` without the buff.
    /// The movement validator's buff factor and the character speed stat read this one value.
    pub fn happy_speed_bonus(&self) -> Option<f32> {
        self.happy.and(
            self.content
                .happy_buff
                .as_ref()
                .map(|buff| buff.speed_bonus),
        )
    }

    /// T532: one happy-speed-buff step. The rule (R): the summoned pet is not vanity and its
    /// decayed mood is at least `happy_mood`; while it holds, one buff-bar row (38/16, reader
    /// `008d8870`) is refreshed every `refresh_ms`, and when it stops the row is removed (38/17).
    /// Returns the rows it sent, so the caller re-sends its speed stat then and reads the new value
    /// from [`Self::happy_speed_bonus`]. Session-only: nothing here marks a save. `None` when
    /// nothing is due, so an idle tick allocates nothing.
    pub fn happy_buff_tick(&mut self, now_ms: u64) -> Option<Vec<Vec<u8>>> {
        // Without a pet the rule cannot hold, and without a live buff there is nothing to end, so
        // the ordinary tick is these two checks: no content borrow, no allocation.
        if self.active.is_none() && self.happy.is_none() {
            return None;
        }
        // The content is shared (`Arc`): holding it costs no copy on this per-tick path.
        let content = Arc::clone(&self.content);
        let config = content.happy_buff.as_ref()?;
        if !self.happy_rule_holds(config, now_ms) {
            return self
                .happy
                .take()
                .map(|_| vec![encode_remove_effect_tag(config.tag_id)]);
        }
        if self.happy.is_some_and(|live| now_ms < live.refresh_at_ms) {
            return None;
        }
        // One row per tag id, so a refresh replaces the client's row and never stacks.
        self.happy = Some(HappyBuffLive {
            refresh_at_ms: now_ms.saturating_add(config.refresh_ms),
        });
        Some(vec![build_effect_tag(
            config.tag_id,
            config.duration_ms,
            config.icon_id,
            config.name_id,
        )])
    }

    /// T532 (R): the summoned pet is not vanity and its decayed mood is at least `happy_mood`.
    /// The decayed view is the one the client's live 53/3 carries (`live_needs`), not the saved
    /// copy, so a pet that starves below the threshold loses the buff on the same clock.
    fn happy_rule_holds(&self, config: &HappyBuff, now_ms: u64) -> bool {
        let Some(active) = self.active else {
            return false;
        };
        let Some(pet) = self.pet(active.pet_id) else {
            return false;
        };
        if self.content.is_vanity(self.content.model_id(&pet.model)) {
            return false;
        }
        decayed(&self.content.need_policy, pet, true, now_ms / 1_000)
            .is_some_and(|needs| needs.mood >= config.happy_mood)
    }

    pub fn list_packet(&self) -> Vec<u8> {
        let mut bytes = pet_header(5);
        bytes.extend((self.roster.pets.len() as u32).to_le_bytes());
        for pet in &self.roster.pets {
            bytes.extend(pet.id.to_le_bytes()); // list key, before the record
            append_pet_record(&mut bytes, pet, &self.content);
        }
        let (pet_id, actor_guid) = self
            .active
            .map_or((0, 0), |active| (active.pet_id, active.actor_guid));
        bytes.extend(pet_id.to_le_bytes());
        bytes.extend(actor_guid.to_le_bytes());
        bytes
    }

    /// Store a follow destination and return the client acknowledgement plus a world-adapter
    /// event. F833 (R): a target farther from the owner than `follow.click_max_m` (ground plane or
    /// height) is refused before anything moves, so a forged coordinate cannot park the pet there;
    /// reachability itself remains unrecovered.
    pub fn follow_location(&mut self, target: [f32; 3]) -> io::Result<PetTransition> {
        if !target.iter().all(|value| value.is_finite()) {
            return Err(invalid("invalid pet follow location"));
        }
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| invalid("no active pet"))?;
        // Without follow rules there is no anchor to bound the click against, so it fails closed.
        let follow = self
            .content
            .follow
            .as_ref()
            .ok_or_else(|| invalid("no pet follow rules"))?;
        if !follow.admits_click(active.pose.owner_position(), target) {
            return Err(invalid("pet follow location out of range"));
        }
        active.follow_target = Some(target);
        // T531: the click's 125 already moves the client pet to `target`; move the server pose too
        // and stop it counting as settled, so the idle life pauses until the follow step parks it.
        // `active.position` stays behind until that step copies the pose, so `wire` still derives
        // the 125 heading from the pet's position before the click.
        active.pose.goto(target);
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
        active.idle.water(active.in_water, in_water);
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

    /// Consume exactly one recorded food through Inventory's staged transaction, then return the
    /// client-shaped status update. T797: the row is a kit item in the client food slot, and the
    /// client `Param2` restore applies to the food need.
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
            .find(|rule| rule.definition_id == item.definition_id)
            .filter(|rule| rule.role() == Some(KitRole::Food))
            .ok_or_else(|| invalid("item is not a recorded pet food"))?;
        let need = Need::Food;
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
        let replies = spend_one(&mut next_roster, &mut next_inventory, item_instance_id)?;
        let status = pet_status_packet(pet_id, &pet_mut(&mut next_roster, pet_id)?.needs);
        self.roster = next_roster;
        *inventory = next_inventory;
        Ok(PetCareOutcome {
            inventory_replies: replies,
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

    /// `slot` is the wire value, 0 to 3: the client sender `00b5f670` sends the one-based UI slot
    /// minus one (it refuses anything outside 1 to 4 itself). The inbound handler (`00b5ce60`
    /// case 14) also stores slot 4, which addresses the first utility slot (+0x98), so 4 and more
    /// are refused here and never echoed. The trick must be one the pet knows.
    pub fn set_favorite(&mut self, pet_id: u32, slot: u32, trick_id: u32) -> io::Result<Vec<u8>> {
        let slot = usize::try_from(slot)
            .ok()
            .filter(|slot| *slot < 4)
            .ok_or_else(|| invalid("invalid pet favorite slot"))?;
        let known = self.content.has_trick(trick_id)
            && self
                .pet(pet_id)
                .is_some_and(|pet| self.trick_known(pet, trick_id));
        if !known {
            return Err(invalid("pet does not know favorite trick"));
        }
        let pet = pet_mut(&mut self.roster, pet_id)?;
        // A trick the model lists as known counts as learned from now on, so the roster check
        // (a favourite is a learned trick) holds for a pet saved before its model row existed.
        pet.learned_tricks.insert(trick_id);
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
            food: content.need_policy.initial,
            groom: content.need_policy.initial,
            play: content.need_policy.initial,
            mood: content.need_policy.initial,
        },
        learned_tricks: definition.initial_tricks.iter().copied().collect(),
        favorite_tricks: [None; 4],
        equipment: PetEquipment::default(),
        adoption_item_definition_id: definition.adoption_item_definition_id,
        tint: None,
        treasure_found_at: None,
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
        pet.needs = decayed(policy, pet, active_pet_id == Some(pet.id), now)
            .ok_or_else(|| invalid("pet clock moved backwards"))?;
        pet.updated_at = now;
    }
    Ok(())
}

/// A pet's needs decayed from its saved clock to `now` without writing the pet (T485's live
/// view reads it every tick); `None` when the clock went backwards.
fn decayed(policy: &NeedPolicy, pet: &Pet, summoned: bool, now: u64) -> Option<PetNeeds> {
    let elapsed = now.checked_sub(pet.updated_at)?;
    let mut needs = pet.needs;
    if elapsed == 0 {
        return Some(needs);
    }
    let per_second = if summoned {
        policy.summoned_decay_per_second
    } else {
        policy.idle_decay_per_second
    };
    let decay = per_second * elapsed as f32;
    needs.food = (needs.food - decay).max(0.0);
    needs.groom = (needs.groom - decay).max(0.0);
    needs.play = (needs.play - decay).max(0.0);
    // (R) Mood follows the average of client need bars until its formula is recovered.
    needs.mood = (needs.food + needs.groom + needs.play) / 3.0;
    Some(needs)
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

fn add_need(needs: &mut PetNeeds, which: Need, amount: f32, maximum: f32) {
    let value = match which {
        Need::Food => &mut needs.food,
        Need::Groom => &mut needs.groom,
        Need::Play => &mut needs.play,
    };
    *value = (*value + amount).min(maximum);
}

/// Consume one unit of an owned care item on staged copies; when the stack empties, clear the item
/// from every pet's slots. Returns the inventory replies.
fn spend_one(
    roster: &mut PetRoster,
    inventory: &mut Inventory,
    instance: i32,
) -> io::Result<Vec<Vec<u8>>> {
    let outcome = inventory.consume(instance, 1)?;
    if !inventory
        .state()
        .items
        .iter()
        .any(|item| item.id == instance)
    {
        for pet in &mut roster.pets {
            clear_equipped_instance(&mut pet.equipment, instance);
        }
    }
    Ok(outcome.replies)
}

/// The owned item fitted in utility slot `index` of `pet`, when the bag still holds it.
fn fitted_item<'a>(
    pet: &Pet,
    inventory: &'a Inventory,
    index: usize,
) -> Option<&'a crate::inventory::InventoryItem> {
    let id = pet.equipment.utilities[index]?;
    inventory
        .state()
        .items
        .iter()
        .find(|item| item.id == id && item.count > 0)
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

    /// T531: the AddNpc and, while the pet lies in a rest posture, the base loop it is playing, so
    /// a viewer who arrives mid-rest does not see it standing (`with_pets`).
    pub fn spawn_packets(&self) -> Option<Vec<Vec<u8>>> {
        let spawn = self.spawn_packet()?;
        let base = self.base_animation.map(|animation_id| {
            crate::world_events::piece_052::animation_packet(self.actor_guid, animation_id, true)
        });
        Some(std::iter::once(spawn).chain(base).collect())
    }
}

pub fn pet_removal_packet(actor_guid: u64) -> Vec<u8> {
    // The 35/3 remove-actor shape already used by rides and field enemies (R).
    let mut bytes = vec![35, 0, 3, 0, 1, 0];
    bytes.extend(actor_guid.to_le_bytes());
    bytes.extend([0; 17]);
    bytes
}

/// T456: one 53/5 pet record in the order the client's reader `00912cf0` consumes it (offsets
/// are the record object's): u32 id (+0x00), u8 sex (+0x04), u32 icon (+0x08), four f32 needs
/// (+0x20..+0x2c), u8 members-only (+0x30), tricks (+0x34: key, 8 u32, u8 known; element reader
/// `008ce1a0`), equipped ids (+0xbc), item classes (+0xe4: key, id, unknown), name (+0x0c),
/// u32 name text id (+0x1c), u32 tint (+0x100), tint alias (+0x108), u32 image set (+0xfc),
/// u8 vanity (+0x104), u32 attachment group (+0x118), u8 tutorial (+0x11c), 4 favourite and
/// 8 utility u32.
fn append_pet_record(out: &mut Vec<u8>, pet: &Pet, content: &PetContent) {
    let model = content.model(content.model_id(&pet.model));
    let item = pet
        .adoption_item_definition_id
        .and_then(|id| content.item(id));
    // Old wiki species without catalogue rows fall back to the species icon (R).
    let species_icon = content
        .species
        .iter()
        .find(|species| species.species_id == Some(pet.species_id))
        .and_then(|species| species.icon.as_ref());
    put_u32(out, pet.id);
    // The sex column is a bool (`00912cf0` stores `value != 0`, shown as "0"/"1"). (R) Models.txt
    // GENDER 2 (female) is true, and 1 or none false, inferred from the one recorded sample
    // (York, model 364, item 23930).
    out.push(u8::from(model.and_then(|m| m.gender) == Some(2)));
    put_u32(out, model.and_then(|m| m.icon_id).unwrap_or(0) as u32);
    for value in [
        pet.needs.food,
        pet.needs.groom,
        pet.needs.play,
        pet.needs.mood,
    ] {
        put_f32(out, value);
    }
    out.push(u8::from(item.is_some_and(|item| item.members_only)));
    // The model's trick list in the client's order, then any learned trick it lacks (old saves).
    let mut rows: Vec<(u32, bool)> = model.map_or_else(Vec::new, |model| {
        model
            .tricks
            .iter()
            .map(|t| (t.id, t.known || pet.learned_tricks.contains(&t.id)))
            .collect()
    });
    for id in &pet.learned_tricks {
        if !rows.iter().any(|(row, _)| row == id) {
            rows.push((*id, true));
        }
    }
    put_count(out, rows.len());
    for (order, (trick_id, known)) in rows.into_iter().enumerate() {
        let trick = content.tricks.iter().find(|t| t.id == trick_id);
        let field =
            |f: fn(&PetTrickDefinition) -> Option<i32>| trick.and_then(f).unwrap_or(0) as u32;
        put_u32(out, trick_id);
        put_u32(out, trick_id);
        put_u32(out, field(|t| t.icon_id));
        put_u32(out, field(|t| t.name_id));
        put_u32(out, field(|t| t.skill_level));
        put_u32(out, field(|t| t.unknown5));
        put_u32(out, field(|t| t.unknown6));
        put_u32(out, field(|t| t.tooltip_id));
        // (R) The per-model order: the model rows are stored in Pets.json Unknown8 order.
        put_u32(out, order as u32 + 1);
        out.push(u8::from(known));
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
    let classes = model.map_or(&[][..], |m| m.item_classes.as_slice());
    put_count(out, classes.len());
    for class in classes {
        put_u32(out, class.key as u32);
        put_u32(out, class.id as u32);
        put_u32(out, class.unknown as u32);
    }
    put_string(out, &pet.name);
    put_u32(out, item.map_or(0, |item| item.name_id) as u32);
    let tint = pet
        .tint
        .or_else(|| model.and_then(|m| m.default_tint_id))
        .or_else(|| species_icon.map(|icon| icon.tint_id))
        .unwrap_or(0);
    put_u32(out, tint as u32);
    // (R) +0x108 resolves through sub_7961D0; the model's default tint alias fills it.
    put_string(
        out,
        model
            .and_then(|m| m.default_tint_alias.as_deref())
            .unwrap_or(""),
    );
    let image_set = item
        .and_then(|item| item.image_set_id)
        .or_else(|| species_icon.map(|icon| icon.image_set_id))
        .unwrap_or(0);
    put_u32(out, image_set as u32);
    out.push(u8::from(model.is_some_and(|m| m.vanity)));
    put_u32(out, 0); // (R) Attachment item group: no source.
    out.push(0); // Tutorial pet flag.
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

/// T485: the 53/16 echo in the client writer's shape (`00c06ea0`; inbound reader `00b55b80`):
/// u8 removed flag, pet id, utility slot, item id (0 for an empty slot).
fn pet_utility_packet(removed: bool, pet_id: u32, slot: u32, item: u32) -> Vec<u8> {
    let mut bytes = pet_header(16);
    bytes.push(u8::from(removed));
    for value in [pet_id, slot, item] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// T485: 53/20 PetUtilityNotify (reader `00b55cc0`): six u32, the pet id, the kind and a timer in
/// seconds (T488: only kind 3 reads it, `00b5ec68` -> `FlashTimerSetTimer` `00a8eff0`), then
/// three words this server leaves 0 (R).
fn pet_utility_notify_packet(pet_id: u32, kind: u32, timer: u32) -> Vec<u8> {
    let mut bytes = pet_header(20);
    for value in [pet_id, kind, timer, 0, 0, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// T489: 53/15 PetTrickNotify (reader `00b55ab0`, handler `00b5ce60` case 15): five u32, the pet id,
/// the kind (1 trick-name line, 2 result, 3 unlock), the trick id, a text id (kind 1: the trick's
/// name for `PetOrderDoTrick`; kind 2: the result text) and a value (kind 2). The client's handler
/// reads only the kind, the text id and the value (it takes the pet from its own state); the pet
/// and trick words are filled for any reader that wants them (R).
fn pet_trick_notify_packet(
    pet_id: u32,
    kind: u32,
    trick_id: u32,
    text_id: u32,
    value: u32,
) -> Vec<u8> {
    let mut bytes = pet_header(15);
    for word in [pet_id, kind, trick_id, text_id, value] {
        bytes.extend(word.to_le_bytes());
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
