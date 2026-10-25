//! Session-scoped mount ownership and actor lifecycle.
//!
//! Mount packets are synthetic inner messages based on the client readers in
//! `notes/Build Guides/Gameplay/Mounts.md`; datagram byte order is unverified.

use crate::{
    inventory::{Inventory, InventoryItem, InventoryState, ItemLook},
    startup_catalogues::{self, CharacterStat, StatUpdate, StatValue},
    startup_world, world,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
    sync::Arc,
};

const FAMILY: u16 = 168;
const LIST: u8 = 5;
// (R) The ride is its own world actor. Its GUID is the owner's XOR a fixed tag: unique per owner,
// never equal to the owner, and outside the other actor GUID spaces (derby 0xd0.., housing 0x0f..).
const RIDE_GUID_TAG: u64 = 0xc000_0000_0000_0000;
const MAX_CONTENT_BYTES: usize = 4 * 1024 * 1024;
// (R) Server-side duplicate guard mirrors the client's 00a2e540 request window.
const SPAWN_DEBOUNCE_MS: u64 = 1_000;
// (R) The seated rider's "queue position" for the Mount call. The client tests 168/2 field 4 for 1
// (success, `00b06100` case 2) and passes field 5 on as `Mount`'s fourth argument (unused once the
// rider is seated), and the AddPc tail feeds the same slot (`0095ad80`); FreeRealms-Legacy, which
// this client accepts, sends 1 in all three places. Used for both so the two routes agree.
const SEATED_QUEUE: i32 = 1;
// (R) Sanity ceiling for ride, glide and on-foot speeds (units per second).
const MAX_SPEED: f32 = 64.0;
// (R) Sanity ceiling for the composite effect id of a boarding or dismount flash.
const MAX_EFFECT_ID: i32 = 100_000;
// (R) Sanity bound for the 168/14 upgrade-item allowlist.
const MAX_UPGRADE_ITEMS: usize = 16;
// Character stat ids index the client's stat-name table at `0x01af2008` (4 bytes per id):
// 2 MaxMovementSpeed (`0x01af2010`); 59-63 GlideDefaultForwardSpeed, GlideMinForwardSpeed,
// GlideMaxForwardSpeed, GlideFallTime, GlideFallSpeed (`0x01af20f4`-`0x01af2104`); 64 GlideEnabled
// (`0x01af2108`); 67 GlideAccel (`0x01af2114`); 68 JumpHeight (`0x01af2118`).
const MAX_MOVEMENT_SPEED: i32 = 2;
const GLIDE_ENABLED: i32 = 64;
const JUMP_HEIGHT: i32 = 68;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "evidence", rename_all = "snake_case")]
pub enum MountSource {
    Client(String),
    Legacy(String),
    Wiki(String),
    Reconstructed(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MountType {
    pub id: u32,
    pub anim_slot_prefix: String,
    pub description: String,
    pub source: Vec<MountSource>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MountDefinition {
    pub id: i32,
    pub item_definition_id_candidates: Vec<i32>,
    pub name: String,
    pub name_string_id: i32,
    pub image_set_id: i32,
    pub model_id: Option<i32>,
    pub mount_type_id: Option<u32>,
    pub max_movement_speed: f32,
    pub members_only: bool,
    pub is_upgradable: bool,
    pub is_upgraded: bool,
    pub tint_id: i32,
    pub tint_alias: String,
    pub seat_count: u8,
    /// Glide and jump stats the rider gets on boarding (`MountCatalog::boarded_stats`).
    #[serde(default)]
    pub stats: RideStats,
    pub source: Vec<MountSource>,
    #[serde(default)]
    pub field_sources: BTreeMap<String, Vec<MountSource>>,
}

/// A ride's movement stats for its rider, sent as character stats (38/7). A missing field is 0
/// (off), which is what a ride without the stat sends.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct RideStats {
    pub glide_default_forward_speed: f32,
    pub glide_min_forward_speed: f32,
    pub glide_max_forward_speed: f32,
    pub glide_fall_time: f32,
    pub glide_fall_speed: f32,
    pub glide_accel: f32,
    pub glide_enabled: bool,
    pub jump_height: f32,
}

impl RideStats {
    /// (R) Sanity bounds: speeds and accel 0..=64, fall time 0..=10, jump 0..=32 (each range also
    /// refuses NaN and infinities, which the client's stat reader rejects), and an enabled glide
    /// keeps min <= default <= max.
    fn valid(&self) -> bool {
        [
            self.glide_default_forward_speed,
            self.glide_min_forward_speed,
            self.glide_max_forward_speed,
            self.glide_fall_speed,
            self.glide_accel,
        ]
        .iter()
        .all(|speed| (0.0..=MAX_SPEED).contains(speed))
            && (0.0..=10.0).contains(&self.glide_fall_time)
            && (0.0..=32.0).contains(&self.jump_height)
            && (!self.glide_enabled
                || (self.glide_min_forward_speed <= self.glide_default_forward_speed
                    && self.glide_default_forward_speed <= self.glide_max_forward_speed))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OnFoot {
    max_movement_speed: f32,
    source: Vec<MountSource>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrainedGlide {
    stats: RideStats,
    source: Vec<MountSource>,
}

/// The composite effect a live boarding or dismount plays on the rider (0 turns it off).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FlashEffect {
    id: i32,
    source: Vec<MountSource>,
}

/// T527: one item definition that trains an owned ride to glide. This list is the only set of
/// item ids 168/14 accepts, so a hostile client cannot name any other item it happens to hold.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpgradeItem {
    item_definition_id: i32,
    source: Vec<MountSource>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MountsDocument {
    schema_version: u32,
    on_foot: OnFoot,
    trained_glide: TrainedGlide,
    mount_effect: FlashEffect,
    upgrade_items: Vec<UpgradeItem>,
    mount_types: Vec<MountType>,
    mounts: Vec<MountDefinition>,
}

#[derive(Clone, Debug)]
pub struct MountCatalog {
    types: BTreeMap<u32, MountType>,
    mounts: BTreeMap<i32, Arc<MountDefinition>>,
    item_to_mount: BTreeMap<i32, i32>,
    on_foot_speed: f32,
    trained_glide: RideStats,
    mount_effect_id: i32,
    /// T527: item definitions that may train a ride (`mounts.json` `upgrade_items`).
    upgrade_items: BTreeSet<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MountError {
    InvalidCatalog,
    InvalidGuid,
    UnknownMount,
    UnknownMountItem,
    NotOwned,
    NotUpgradable,
    AlreadyUpgraded,
    NotUpgradeItem,
    UpgradeItemNotOwned,
    Restricted,
    WrongMountActor,
    InvalidSeat,
    NoSpawnedRide,
}

impl MountCatalog {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;
        if !metadata.is_file() || metadata.len() > MAX_CONTENT_BYTES as u64 {
            return Err(invalid_data(
                "mount content must be a regular file under 4 MiB",
            ));
        }
        let bytes = fs::read(path)?;
        let document: MountsDocument = serde_json::from_slice(&bytes).map_err(invalid_data)?;
        if document.schema_version != 1
            || !valid_sources(&document.on_foot.source)
            || !valid_sources(&document.trained_glide.source)
            || !valid_sources(&document.mount_effect.source)
        {
            return Err(invalid_data("unsupported mount content schema"));
        }
        let mut catalog = Self::new(
            document.mount_types,
            document.mounts,
            document.on_foot.max_movement_speed,
            document.trained_glide.stats,
            document.mount_effect.id,
        )
        .map_err(|_| invalid_data("invalid mount content records"))?;
        catalog.upgrade_items = upgrade_item_ids(document.upgrade_items)
            .map_err(|_| invalid_data("invalid mount upgrade items"))?;
        Ok(catalog)
    }

    /// `on_foot_speed` is what a ride's end restores; `trained_glide` is the glide block of an
    /// upgradable ride its owner trained; `mount_effect_id` is the composite effect a live
    /// boarding or dismount plays (T483; 0 is off).
    pub fn new(
        mount_types: Vec<MountType>,
        mounts: Vec<MountDefinition>,
        on_foot_speed: f32,
        trained_glide: RideStats,
        mount_effect_id: i32,
    ) -> Result<Self, MountError> {
        if mount_types.is_empty()
            || mount_types.len() > 128
            || mounts.is_empty()
            || mounts.len() > 4096
            || !(on_foot_speed > 0.0 && on_foot_speed <= MAX_SPEED)
            || !trained_glide.valid()
            || !(0..=MAX_EFFECT_ID).contains(&mount_effect_id)
        {
            return Err(MountError::InvalidCatalog);
        }
        let mut types = BTreeMap::new();
        for mount_type in mount_types {
            let empty_sentinel = mount_type.id == 0
                && mount_type.anim_slot_prefix.is_empty()
                && mount_type.description.is_empty();
            if (!empty_sentinel
                && (mount_type.id == 0
                    || mount_type.anim_slot_prefix.is_empty()
                    || mount_type.description.is_empty()))
                || !valid_sources(&mount_type.source)
                || types.insert(mount_type.id, mount_type).is_some()
            {
                return Err(MountError::InvalidCatalog);
            }
        }

        let mut records = BTreeMap::new();
        let mut item_to_mount = BTreeMap::new();
        for mount in mounts {
            if mount.id <= 0
                || mount.item_definition_id_candidates.is_empty()
                || mount
                    .item_definition_id_candidates
                    .iter()
                    .any(|id| *id <= 0)
                || mount.name.trim().is_empty()
                || mount.name_string_id <= 0
                || mount.image_set_id <= 0
                || !(0.0..=MAX_SPEED).contains(&mount.max_movement_speed)
                || !mount.stats.valid()
                || !(1..=99).contains(&mount.seat_count)
                || mount.tint_alias.len() > 4096
                || !valid_sources(&mount.source)
                || mount
                    .field_sources
                    .values()
                    .any(|sources| !valid_sources(sources))
                || mount
                    .mount_type_id
                    .is_some_and(|type_id| type_id == 0 || !types.contains_key(&type_id))
                || records.contains_key(&mount.id)
            {
                return Err(MountError::InvalidCatalog);
            }
            let mount_id = mount.id;
            for item_id in &mount.item_definition_id_candidates {
                if item_to_mount.insert(*item_id, mount_id).is_some() {
                    return Err(MountError::InvalidCatalog);
                }
            }
            records.insert(mount_id, Arc::new(mount));
        }
        Ok(Self {
            types,
            mounts: records,
            item_to_mount,
            on_foot_speed,
            trained_glide,
            mount_effect_id,
            upgrade_items: BTreeSet::new(),
        })
    }

    /// The full 38/7 block a boarded ride sends its rider, always all nine stats so a replacement
    /// ride never keeps an earlier ride's glide. The glide comes from the ride's own stats when
    /// they enable it, else from `trained_glide` for a trained upgradable ride, else it is off.
    fn boarded_stats(
        &self,
        mount: &MountDefinition,
        instance_id: i32,
        upgraded: &BTreeSet<i32>,
    ) -> Vec<CharacterStat> {
        let glide = if mount.stats.glide_enabled {
            mount.stats
        } else if mount.is_upgradable && trained(mount, instance_id, upgraded) {
            self.trained_glide
        } else {
            RideStats::default()
        };
        vec![
            float_stat(MAX_MOVEMENT_SPEED, mount.max_movement_speed),
            float_stat(59, glide.glide_default_forward_speed),
            float_stat(60, glide.glide_min_forward_speed),
            float_stat(61, glide.glide_max_forward_speed),
            float_stat(62, glide.glide_fall_time),
            float_stat(63, glide.glide_fall_speed),
            int_stat(GLIDE_ENABLED, glide.glide_enabled.into()),
            float_stat(67, glide.glide_accel),
            float_stat(JUMP_HEIGHT, mount.stats.jump_height),
        ]
    }

    /// (R) What a ride's end restores: on-foot speed, glide off and no jump bonus. Both community
    /// servers send exactly these three; the captured snapshot holds only the speed.
    fn on_foot_stats(&self) -> Vec<CharacterStat> {
        vec![
            float_stat(MAX_MOVEMENT_SPEED, self.on_foot_speed),
            int_stat(GLIDE_ENABLED, 0),
            float_stat(JUMP_HEIGHT, 0.0),
        ]
    }

    pub fn mount_types(&self) -> impl Iterator<Item = &MountType> {
        self.types.values()
    }

    pub fn mounts(&self) -> impl Iterator<Item = &MountDefinition> {
        self.mounts.values().map(|mount| &**mount)
    }

    pub fn mount(&self, mount_id: i32) -> Option<&MountDefinition> {
        self.mounts.get(&mount_id).map(|mount| &**mount)
    }

    fn mount_arc(&self, mount_id: i32) -> Option<Arc<MountDefinition>> {
        self.mounts.get(&mount_id).cloned()
    }

    pub fn mount_for_item(&self, item_definition_id: i32) -> Option<&MountDefinition> {
        self.item_to_mount
            .get(&item_definition_id)
            .and_then(|mount_id| self.mount(*mount_id))
    }

    /// T527: whether this item definition may train a ride (the content allowlist).
    pub fn is_upgrade_item(&self, item_definition_id: i32) -> bool {
        self.upgrade_items.contains(&item_definition_id)
    }

    /// Ownership is derived from positive-count inventory instances. Item-to-
    /// mount candidates in the data are reconstructed crosswalks (R).
    pub fn owned_mounts<'a>(&'a self, inventory: &InventoryState) -> Vec<&'a MountDefinition> {
        let owned_items: BTreeSet<i32> = inventory
            .items
            .iter()
            .filter(|item| item.count > 0)
            .map(|item| item.definition_id)
            .collect();
        self.mounts()
            .filter(|mount| {
                mount
                    .item_definition_id_candidates
                    .iter()
                    .any(|item_id| owned_items.contains(item_id))
            })
            .collect()
    }

    /// T482: one roster row per owned ride item, ascending by instance id: an inventory instance
    /// with `count > 0` and a positive id (the id is what the client sends back) whose item maps
    /// to a ride.
    fn owned_rides<'a>(
        &'a self,
        inventory: &'a InventoryState,
    ) -> Vec<(&'a InventoryItem, &'a MountDefinition)> {
        let mut rows: Vec<_> = inventory
            .items
            .iter()
            .filter(|item| item.count > 0 && item.id > 0)
            .filter_map(|item| Some((item, self.mount_for_item(item.definition_id)?)))
            .collect();
        rows.sort_by_key(|(item, _)| item.id);
        rows
    }

    /// Whether this inventory row is a roster row: owned, positively numbered, and a ride item.
    fn is_ride_row(&self, item: &InventoryItem) -> bool {
        item.count > 0 && item.id > 0 && self.item_to_mount.contains_key(&item.definition_id)
    }

    /// The owned ride instance with this row id; `None` for an unknown, unowned, zero-count,
    /// non-positive or non-ride id.
    fn owned_instance<'a>(
        &self,
        inventory: &'a InventoryState,
        instance_id: i32,
    ) -> Option<&'a InventoryItem> {
        let is_row = |item: &&InventoryItem| item.id == instance_id && self.is_ride_row(item);
        inventory.items.iter().find(is_row)
    }

    /// The owned ride instance with the lowest id whose item definition passes `item_matches`.
    fn lowest_instance<'a>(
        &self,
        inventory: &'a InventoryState,
        item_matches: impl Fn(i32) -> bool,
    ) -> Option<&'a InventoryItem> {
        let wanted =
            |item: &&InventoryItem| self.is_ride_row(item) && item_matches(item.definition_id);
        inventory
            .items
            .iter()
            .filter(wanted)
            .min_by_key(|item| item.id)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideRestrictions {
    /// Zone and activity restrictions are reconstructed server policy (R).
    pub zone_allows_riding: bool,
    pub in_minigame: bool,
    pub ability_active: bool,
    /// A knocked-out player cannot summon or board a ride (R).
    pub knocked_out: bool,
}

impl RideRestrictions {
    fn allows(self) -> bool {
        self.zone_allows_riding && !self.in_minigame && !self.ability_active && !self.knocked_out
    }
}

/// How a ride actor is drawn (T483): the owned item's catalogue texture and tint, so rides that
/// share a model look different, else the ride row's own values.
#[derive(Clone, Debug, PartialEq)]
pub struct RideLook {
    pub texture_alias: String,
    pub tint_alias: String,
    pub tint_id: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveRide {
    pub mount_id: i32,
    /// T482: the inventory instance (roster row id) this ride was summoned from.
    pub instance_id: i32,
    pub actor_guid: u64,
    /// T483: fixed when the ride is summoned, so observers and late joiners draw the same ride.
    pub look: RideLook,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MountState {
    Empty,
    Spawned(ActiveRide),
    Mounted {
        ride: ActiveRide,
        rider_guid: u64,
        seat_index: u32,
    },
}

impl MountState {
    pub fn active_ride(&self) -> Option<&ActiveRide> {
        match self {
            Self::Empty => None,
            Self::Spawned(ride) | Self::Mounted { ride, .. } => Some(ride),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MountEffectKind {
    /// The lead routes this through the world actor and Players interest paths.
    SpawnRide {
        owner_guid: u64,
        mount_id: i32,
        actor_guid: u64,
        model_id: Option<i32>,
        max_movement_speed: f32,
        look: RideLook,
    },
    DespawnRide {
        actor_guid: u64,
    },
    /// Synthetic inner application packet, before gateway tunnel wrapping.
    Packet(Vec<u8>),
    /// The rider's character stats (38/7): the ride's block on boarding, the on-foot values when
    /// a boarded ride ends. Owner only.
    RiderStats {
        rider_guid: u64,
        stats: Vec<CharacterStat>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct MountEffect {
    /// Character GUID recipients; nearby observers come from Players interest.
    pub recipients: Vec<u64>,
    pub kind: MountEffectKind,
}

impl MountEffect {
    /// Everything but the owner's roster list and stats is public: nearby players need the ride
    /// actor, the rider's attachment and its removal to draw the rider on the ride (client
    /// 00b06100 handles 168/2 and 168/4 for any rider it knows, not only the local player).
    pub fn nearby(&self) -> bool {
        match &self.kind {
            MountEffectKind::Packet(bytes) => bytes.get(2) != Some(&LIST),
            MountEffectKind::RiderStats { .. } => false,
            MountEffectKind::SpawnRide { .. } | MountEffectKind::DespawnRide { .. } => true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MountOutcome {
    pub effects: Vec<MountEffect>,
    pub changed: bool,
    pub debounced: bool,
}

#[derive(Clone, Debug)]
pub struct MountService {
    catalog: Arc<MountCatalog>,
    state: MountState,
    last_spawn_ms: Option<u64>,
    /// Session-only: what the last roster (168/5) sent. `list_packet` records it, `roster_sync`
    /// compares with it, `reset` clears it; `None` until world entry sends the first roster.
    last_sent: Option<SentRoster>,
    /// T482: the catalogue look of every ride item, for each row's tint alias.
    looks: BTreeMap<i32, ItemLook>,
    /// T527: inventory instance ids this character trained to glide. Persisted as
    /// `GameplayState::upgraded_rides` and set only after a saddle was consumed.
    upgraded: BTreeSet<i32>,
}

/// T482: every field of the last roster that comes from live state. A row's name and image set
/// come from the ride and its alias from the item's catalogue row, both fixed by the item
/// definition, so a row changes only through these.
#[derive(Clone, Debug)]
struct SentRoster {
    /// (instance id, item definition, tint) per row, ascending: the order of `owned_rides`.
    rows: Vec<(i32, i32, i32)>,
    active_guid: Option<u64>,
}

impl MountService {
    pub fn new(catalog: MountCatalog) -> Self {
        Self {
            catalog: Arc::new(catalog),
            state: MountState::Empty,
            last_spawn_ms: None,
            last_sent: None,
            looks: BTreeMap::new(),
            upgraded: BTreeSet::new(),
        }
    }

    /// The service for the ride catalogue at `path`, with the look of every ride item read from
    /// the loaded item catalogue (`Inventory::look`).
    pub fn load(path: &Path, inventory: &Inventory) -> io::Result<Self> {
        let mut service = Self::new(MountCatalog::load(path)?);
        service.looks = (service.catalog.item_to_mount.keys())
            .filter_map(|id| Some((*id, inventory.look(*id)?)))
            .collect();
        Ok(service)
    }

    pub fn state(&self) -> &MountState {
        &self.state
    }

    pub fn catalog(&self) -> &MountCatalog {
        &self.catalog
    }

    /// T527: restore the flags a save carries (`GameplayState::upgraded_rides`).
    pub fn with_upgraded(mut self, upgraded: BTreeSet<i32>) -> Self {
        self.upgraded = upgraded;
        self
    }

    /// T527: the saved form, keeping only instance ids this character still owns as ride rows.
    /// A traded or sold ride loses its training (R).
    pub fn saved_upgrades(&self, inventory: &InventoryState) -> BTreeSet<i32> {
        self.upgraded
            .iter()
            .copied()
            .filter(|instance_id| {
                (self.catalog)
                    .owned_instance(inventory, *instance_id)
                    .is_some()
            })
            .collect()
    }

    /// True while the rider sits on the ride. This is the `mounted` input of the emote rules
    /// (`social_chat::emote_context`).
    pub fn is_mounted(&self) -> bool {
        matches!(self.state, MountState::Mounted { .. })
    }

    /// The boarded ride's speed; `None` while on foot or beside a summoned ride.
    pub fn mounted_speed(&self) -> Option<f32> {
        let MountState::Mounted { ride, .. } = &self.state else {
            return None;
        };
        Some(self.catalog.mount(ride.mount_id)?.max_movement_speed)
    }

    /// A new network session starts with no ride out (the actor does not survive a disconnect).
    pub fn reset(&mut self) {
        self.state = MountState::Empty;
        self.last_spawn_ms = None;
        self.last_sent = None;
    }

    /// The owner's roster (168/5). Every roster sent goes through here, so it also records what
    /// the client now holds (`last_sent`).
    pub fn list_packet(&mut self, inventory: &InventoryState) -> Vec<u8> {
        let rows = self.catalog.owned_rides(inventory);
        let active = self.state.active_ride();
        self.last_sent = Some(SentRoster {
            rows: rows
                .iter()
                .map(|(item, _)| (item.id, item.definition_id, item.tint))
                .collect(),
            active_guid: active.map(|ride| ride.actor_guid),
        });
        encode_mount_list(&rows, active, &self.looks, &self.upgraded)
    }

    /// The roster again when an owned ride item, its tint or the active ride changed since the
    /// last one sent; `None` when nothing changed or before world entry sent the first. It runs
    /// after every message and tick, so it allocates nothing unless it sends: every live ride row
    /// must be a remembered one (the remembered rows are sorted and instance ids are unique) and
    /// the counts must match, so no remembered row is gone.
    pub fn roster_sync(&mut self, inventory: &InventoryState) -> Option<Vec<u8>> {
        let sent = self.last_sent.as_ref()?;
        let mut live = 0;
        let mut all_sent = true;
        for item in inventory
            .items
            .iter()
            .filter(|i| self.catalog.is_ride_row(i))
        {
            live += 1;
            let row = (item.id, item.definition_id, item.tint);
            all_sent &= sent.rows.binary_search(&row).is_ok();
        }
        let unchanged = all_sent
            && live == sent.rows.len()
            && sent.active_guid == self.state.active_ride().map(|ride| ride.actor_guid);
        (!unchanged).then(|| self.list_packet(inventory))
    }

    /// Runs after every message and tick (`Gameplay::settle_mounts`). The ride is put away when
    /// `end` holds (zoning, a fight or minigame, a knockout) or its item is gone; otherwise the
    /// roster is resent when ownership changed. A put-away carries the roster itself, so one
    /// change sends one 168/5.
    pub fn settle(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        end: bool,
    ) -> MountOutcome {
        let mut outcome = if end || self.ride_lost(inventory) {
            self.dismount_request(owner_guid, inventory, &[])
        } else {
            MountOutcome::default()
        };
        let roster = self.roster_sync(inventory);
        outcome
            .effects
            .extend(roster.map(|list| packet(vec![owner_guid], list)));
        outcome
    }

    /// (R) Losing the ride item (trade, sale, drop) ends the ride: the instance it was summoned
    /// from (T482: that row, not any item of the same ride) is no longer owned.
    fn ride_lost(&self, inventory: &InventoryState) -> bool {
        self.state
            .active_ride()
            .is_some_and(|ride| !self.owns(ride, inventory))
    }

    /// Whether the ride's own instance is still owned and still maps to the ride.
    fn owns(&self, ride: &ActiveRide, inventory: &InventoryState) -> bool {
        self.catalog
            .owned_instance(inventory, ride.instance_id)
            .and_then(|item| self.catalog.item_to_mount.get(&item.definition_id))
            == Some(&ride.mount_id)
    }

    /// (R) A ride actor must have its own nonzero world GUID, and riding must be allowed.
    fn spawn_guard(
        owner_guid: u64,
        actor_guid: u64,
        restrictions: RideRestrictions,
    ) -> Result<(), MountError> {
        if owner_guid == 0 || actor_guid == 0 || actor_guid == owner_guid {
            return Err(MountError::InvalidGuid);
        }
        if !restrictions.allows() {
            return Err(MountError::Restricted);
        }
        Ok(())
    }

    /// Summon a ride by definition: its owned instance with the lowest id (T482).
    pub fn spawn(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        mount_id: i32,
        actor_guid: u64,
        restrictions: RideRestrictions,
        now_ms: u64,
        observers: &[u64],
    ) -> Result<MountOutcome, MountError> {
        Self::spawn_guard(owner_guid, actor_guid, restrictions)?;
        let mount = self
            .catalog
            .mount(mount_id)
            .ok_or(MountError::UnknownMount)?;
        let candidates = &mount.item_definition_id_candidates;
        let instance = (self.catalog)
            .lowest_instance(inventory, |item| candidates.contains(&item))
            .ok_or(MountError::NotOwned)?
            .id;
        self.spawn_instance(
            owner_guid, inventory, instance, actor_guid, now_ms, observers,
        )
    }

    /// Summon the ride a roster row (168/6) names: that exact owned instance. An id that is
    /// unknown, not owned, zero-count, not positive or not a ride is refused (T482).
    pub fn spawn_row(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        instance_id: i32,
        actor_guid: u64,
        restrictions: RideRestrictions,
        now_ms: u64,
        observers: &[u64],
    ) -> Result<MountOutcome, MountError> {
        Self::spawn_guard(owner_guid, actor_guid, restrictions)?;
        self.spawn_instance(
            owner_guid,
            inventory,
            instance_id,
            actor_guid,
            now_ms,
            observers,
        )
    }

    /// 168/8: the owned instance of that item with the lowest id (T482).
    pub fn spawn_by_item_definition(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        item_definition_id: i32,
        actor_guid: u64,
        restrictions: RideRestrictions,
        now_ms: u64,
        observers: &[u64],
    ) -> Result<MountOutcome, MountError> {
        (self.catalog)
            .mount_for_item(item_definition_id)
            .ok_or(MountError::UnknownMountItem)?;
        Self::spawn_guard(owner_guid, actor_guid, restrictions)?;
        let instance = (self.catalog)
            .lowest_instance(inventory, |item| item == item_definition_id)
            .ok_or(MountError::NotOwned)?
            .id;
        self.spawn_instance(
            owner_guid, inventory, instance, actor_guid, now_ms, observers,
        )
    }

    /// T527: validate a 168/14 request. The row must be an owned ride instance, its ride
    /// upgradable and not already trained, the item a configured upgrade item and the player must
    /// hold at least one (the lowest instance id is the one consumed). Pure: nothing changes until
    /// the caller consumed that item and called `commit_upgrade`.
    pub fn upgrade_plan(
        &self,
        inventory: &InventoryState,
        restrictions: RideRestrictions,
        row_id: i32,
        item_definition_id: i32,
    ) -> Result<RideUpgrade, MountError> {
        if !restrictions.allows() {
            return Err(MountError::Restricted);
        }
        let item = (self.catalog)
            .owned_instance(inventory, row_id)
            .ok_or(MountError::NotOwned)?;
        let mount = (self.catalog)
            .mount_for_item(item.definition_id)
            .ok_or(MountError::NotOwned)?;
        if !mount.is_upgradable {
            return Err(MountError::NotUpgradable);
        }
        if trained(mount, row_id, &self.upgraded) {
            return Err(MountError::AlreadyUpgraded);
        }
        if !self.catalog.is_upgrade_item(item_definition_id) {
            return Err(MountError::NotUpgradeItem);
        }
        let saddle = inventory
            .items
            .iter()
            .filter(|held| held.definition_id == item_definition_id && held.count > 0)
            .min_by_key(|held| held.id)
            .ok_or(MountError::UpgradeItemNotOwned)?;
        Ok(RideUpgrade {
            row_id,
            item_instance_id: saddle.id,
        })
    }

    /// T527: record the training after the saddle was consumed.
    pub fn commit_upgrade(&mut self, row_id: i32) {
        self.upgraded.insert(row_id);
    }

    /// T527: 168/15 (reader `00b05590` -> `00b05220` reads one i32 into the reply at +0xc). On it
    /// the client shows "UpgradeSuccessful", dismounts and re-summons this row with 168/6
    /// (`00b06100` case 15, "Automount after upgrade: id %d").
    pub fn upgrade_reply(row_id: i32) -> Vec<u8> {
        let mut out = header(15);
        put_i32(&mut out, row_id);
        out
    }

    /// T483: The ride item's catalogue texture and tint, tinted the way `write_attachment` dresses
    /// an item (the instance's dye, else the item's default). An item the catalogue does not
    /// define falls back to the ride row.
    fn ride_look(&self, item: &InventoryItem, mount: &MountDefinition) -> RideLook {
        match self.looks.get(&item.definition_id) {
            Some(look) => RideLook {
                texture_alias: look.texture.clone(),
                tint_alias: look.tint_alias.clone(),
                tint_id: if item.tint != 0 {
                    item.tint
                } else {
                    look.default_tint
                },
            },
            None => RideLook {
                texture_alias: String::new(),
                tint_alias: mount.tint_alias.clone(),
                tint_id: mount.tint_id,
            },
        }
    }

    /// The summon itself, after the guards passed and the instance was chosen.
    fn spawn_instance(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        instance_id: i32,
        actor_guid: u64,
        now_ms: u64,
        observers: &[u64],
    ) -> Result<MountOutcome, MountError> {
        let item = (self.catalog)
            .owned_instance(inventory, instance_id)
            .ok_or(MountError::NotOwned)?;
        let mount = (self.catalog)
            .mount_for_item(item.definition_id)
            .and_then(|mount| self.catalog.mount_arc(mount.id))
            .ok_or(MountError::UnknownMount)?;
        let mount_id = mount.id;
        let look = self.ride_look(item, &mount);
        if self
            .last_spawn_ms
            .is_some_and(|last| now_ms.saturating_sub(last) < SPAWN_DEBOUNCE_MS)
        {
            return Ok(MountOutcome {
                debounced: true,
                ..MountOutcome::default()
            });
        }

        let recipients = recipients(owner_guid, observers);
        let mut outcome = MountOutcome {
            changed: true,
            ..MountOutcome::default()
        };
        let was_mounted = self.is_mounted();
        // (R) Replace any prior summoned actor after the client spawn handshake.
        if let Some(old_ride) = self.state.active_ride().cloned() {
            if let MountState::Mounted { rider_guid, .. } = &self.state {
                outcome.effects.push(packet(
                    recipients.clone(),
                    encode_dismount_response(*rider_guid, self.catalog.mount_effect_id),
                ));
            }
            outcome.effects.push(MountEffect {
                recipients: recipients.clone(),
                kind: MountEffectKind::DespawnRide {
                    actor_guid: old_ride.actor_guid,
                },
            });
        }
        self.state = MountState::Spawned(ActiveRide {
            mount_id,
            instance_id,
            actor_guid,
            look: look.clone(),
        });
        self.last_spawn_ms = Some(now_ms);
        // The client needs the automount id/GUID list entry before the actor update.
        outcome
            .effects
            .push(packet(vec![owner_guid], self.list_packet(inventory)));
        outcome.effects.push(MountEffect {
            recipients,
            kind: MountEffectKind::SpawnRide {
                owner_guid,
                mount_id,
                actor_guid,
                model_id: mount.model_id,
                max_movement_speed: mount.max_movement_speed,
                look,
            },
        });
        if was_mounted {
            outcome
                .effects
                .push(rider_stats(owner_guid, self.catalog.on_foot_stats()));
        }
        Ok(outcome)
    }

    pub fn mount(
        &mut self,
        owner_guid: u64,
        rider_guid: u64,
        inventory: &InventoryState,
        target_actor_guid: u64,
        seat_index: u32,
        restrictions: RideRestrictions,
        observers: &[u64],
    ) -> Result<MountOutcome, MountError> {
        // (R) This service only mounts the authenticated character's own ride.
        if owner_guid == 0 || rider_guid == 0 || owner_guid != rider_guid {
            return Err(MountError::InvalidGuid);
        }
        if !restrictions.allows() {
            return Err(MountError::Restricted);
        }
        let ride = match &self.state {
            MountState::Spawned(ride) => ride.clone(),
            MountState::Empty | MountState::Mounted { .. } => {
                return Err(MountError::NoSpawnedRide);
            }
        };
        if ride.actor_guid != target_actor_guid {
            return Err(MountError::WrongMountActor);
        }
        let mount = self
            .catalog
            .mount(ride.mount_id)
            .ok_or(MountError::UnknownMount)?;
        if !self.owns(&ride, inventory) {
            return Err(MountError::NotOwned);
        }
        if seat_index >= u32::from(mount.seat_count) {
            return Err(MountError::InvalidSeat);
        }
        self.state = MountState::Mounted {
            ride: ride.clone(),
            rider_guid,
            seat_index,
        };
        Ok(MountOutcome {
            changed: true,
            effects: vec![
                packet(
                    recipients(owner_guid, observers),
                    encode_mount_response(
                        rider_guid,
                        ride.actor_guid,
                        seat_index,
                        self.catalog.mount_effect_id,
                    ),
                ),
                rider_stats(
                    owner_guid,
                    self.catalog
                        .boarded_stats(mount, ride.instance_id, &self.upgraded),
                ),
            ],
            debounced: false,
        })
    }

    /// 168/3 is shared by Lua `Dismount` and `Despawn` and the client cannot say which one ran, so
    /// the server treats every request as "put the ride away": a rider is dismounted and the ride
    /// actor removed in one step (R; W25 kept the actor standing until a second request). The
    /// client always sends 168/3 before a new 168/6 spawn (00bff6f0), so a re-summon starts clean.
    /// Server-side triggers (zoning, combat, knockout, minigame) call this too.
    pub fn dismount_request(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        observers: &[u64],
    ) -> MountOutcome {
        let all_recipients = recipients(owner_guid, observers);
        let mut effects = Vec::new();
        let was_mounted = self.is_mounted();
        let ride = match std::mem::replace(&mut self.state, MountState::Empty) {
            // (R) Empty dismount requests are idempotent no-ops.
            MountState::Empty => return MountOutcome::default(),
            MountState::Spawned(ride) => ride,
            MountState::Mounted {
                ride, rider_guid, ..
            } => {
                effects.push(packet(
                    all_recipients.clone(),
                    encode_dismount_response(rider_guid, self.catalog.mount_effect_id),
                ));
                ride
            }
        };
        effects.push(MountEffect {
            recipients: all_recipients,
            kind: MountEffectKind::DespawnRide {
                actor_guid: ride.actor_guid,
            },
        });
        effects.push(packet(vec![owner_guid], self.list_packet(inventory)));
        if was_mounted {
            effects.push(rider_stats(owner_guid, self.catalog.on_foot_stats()));
        }
        MountOutcome {
            changed: true,
            effects,
            debounced: false,
        }
    }

    pub fn on_minigame_start(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        observers: &[u64],
    ) -> MountOutcome {
        self.dismount_request(owner_guid, inventory, observers)
    }

    pub fn on_ability_use(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        observers: &[u64],
    ) -> MountOutcome {
        self.dismount_request(owner_guid, inventory, observers)
    }
}

/// Client-to-server mount requests (family 168). Bodies follow the client constructors:
/// 168/1 `PacketMountRequest` (`008b3730`: target GUID plus further fields), 168/3
/// `PacketDismountRequest` (`009aa410`, no fields), 168/6 spawn (`00b038c0`, one i32 ride id,
/// writer `00b05a80`) and 168/8 spawn by item (`00b03970`, i32 item definition and a flag byte
/// that is always 1, writer `00b05ae0`). T482: the 168/6 id is a roster row id, an inventory
/// instance id (the client echoes the id of the row it clicked, `00bff6f0` "Lua Spawn and
/// automount: id %d"), so `Spawn::mount_id` names that instance, not a ride.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountRequest {
    Mount {
        target: u64,
    },
    Dismount,
    Spawn {
        mount_id: i32,
    },
    SpawnByItem {
        item_definition_id: i32,
    },
    /// T527: train the owned ride row `row_id` with the item definition
    /// `item_definition_id` (both are client-supplied and validated).
    Upgrade {
        row_id: i32,
        item_definition_id: i32,
    },
}

/// `Ok(None)` when the message is not a mount request this server answers; `Err` when it is one
/// with a malformed body. The 168/1 and 168/3 serializers were not recovered, so those two accept
/// a bounded tail (R) and read only the leading target GUID. T527: 168/14's writer `00bff500`
/// writes `this+0xc` then `this+0x10` as two i32s (`Mount.Upgrade` `00bffbc0` stores the Lua
/// mount id in the first and the item definition id in the second).
pub fn decode_request(inner: &[u8]) -> io::Result<Option<MountRequest>> {
    if inner.len() < 3 || inner[..2] != FAMILY.to_le_bytes() {
        return Ok(None);
    }
    let body = &inner[3..];
    let word = |at: usize| i32::from_le_bytes(body[at..at + 4].try_into().unwrap());
    Ok(Some(match (inner[2], body.len()) {
        (1, 8..=24) => MountRequest::Mount {
            target: u64::from_le_bytes(body[..8].try_into().unwrap()),
        },
        (3, 0..=16) => MountRequest::Dismount,
        (6, 4) => MountRequest::Spawn { mount_id: word(0) },
        (8, 5) => MountRequest::SpawnByItem {
            item_definition_id: word(0),
        },
        (14, 8) => MountRequest::Upgrade {
            row_id: word(0),
            item_definition_id: word(4),
        },
        (1 | 3 | 6 | 8 | 14, _) => return Err(invalid_data("malformed mount request")),
        _ => return Ok(None),
    }))
}

/// Where the ride actor stands when it appears: the owner's pose. Heading is radians, the
/// convention of `players.rs` (`atan2(x, z)` of the character rotation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RidePose {
    pub position: [f32; 3],
    pub heading: f32,
}

/// What the gateway layer knows when a request arrives.
pub struct RideContext<'a> {
    pub guid: u64,
    pub inventory: &'a InventoryState,
    pub restrictions: RideRestrictions,
    pub pose: RidePose,
    pub now_ms: u64,
}

/// T527: what a valid 168/14 request will change. The plan is pure; the caller consumes
/// `item_instance_id` and only then calls `MountService::commit_upgrade`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RideUpgrade {
    /// The owned ride row the client picked (the request's first i32).
    pub row_id: i32,
    /// The owned saddle instance to consume (one count).
    pub item_instance_id: i32,
}

/// Inner application packets for one handled event. `private` goes to the owner only; `shared`
/// goes to the owner and every player in interest range of the owner (the gateway fans it out
/// through `PlayerRegistry::nearby_sessions`). `notice` is a refusal for a system chat line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RideWire {
    pub private: Vec<Vec<u8>>,
    pub shared: Vec<Vec<u8>>,
    pub notice: Option<&'static str>,
}

/// The mount block at the end of a player's AddPc (35/1): the ride's GUID, seat, queue position and
/// a lift (the client also reads it as `Mount`'s last argument, so keep it 0). `NONE` is what an
/// unmounted player always carried: no GUID and -1 for seat and queue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MountTail {
    pub ride_guid: u64,
    pub seat: i32,
    pub queue: i32,
    pub lift: f32,
}

impl MountTail {
    pub const NONE: Self = Self {
        ride_guid: 0,
        seat: -1,
        queue: -1,
        lift: 0.0,
    };
}

/// A spawned ride as other players must see it. A player who enters interest range later gets
/// `packets` (ride actor, then the rider's attachment) so the late joiner draws the same scene.
#[derive(Clone, Debug, PartialEq)]
pub struct RideView {
    pub rider_guid: u64,
    pub actor_guid: u64,
    pub seat: Option<u32>,
    mount: Arc<MountDefinition>,
    look: RideLook,
}

impl RideView {
    /// Remove-actor packet for a viewer that loses sight of the rider or the ride.
    pub fn removal_packet(&self) -> Vec<u8> {
        remove_actor(self.actor_guid)
    }

    /// The mount block of this rider's AddPc (35/1), for a viewer who meets the rider later.
    /// Natively (`0092f460` case 1 -> `SetPendingMountInfo` `0095ad80`) the client keeps these
    /// values and calls `Mount` (`00972d30`) from `ProxiedCharacter::Process` as soon as the ride
    /// actor exists, so the order of AddPc and the ride's AddNpc does not matter. A ride that is
    /// summoned but not boarded has no seat to carry.
    pub fn tail(&self) -> MountTail {
        self.seat.map_or(MountTail::NONE, |seat| MountTail {
            ride_guid: self.actor_guid,
            seat: seat as i32,
            queue: SEATED_QUEUE,
            lift: 0.0,
        })
    }

    /// The ride's copy of the rider's 125 position update (`rider_update` is the inner packet).
    ///
    /// (R) Hedge. Natively the client carries a ridden actor with its rider: the world's frame loop
    /// (`00984010`) skips `ProxiedCharacter::Process` for an actor whose rider GUID (+0x17a0, set
    /// by `Mount`) is set, and the 125 handler (`0090de90`) ignores such an actor, so these copies
    /// are most likely dropped by the client. They cost one small packet per rider move and keep
    /// the ride moving if that reading is wrong (FreeRealms-Legacy sends them too). Remove this
    /// relay once a native capture shows the ride follows without it.
    pub fn follow(&self, rider_update: &[u8]) -> Option<Vec<u8>> {
        self.seat?;
        if rider_update.get(..2)? != [125, 0]
            || rider_update.get(2..10)? != self.rider_guid.to_le_bytes()
        {
            return None;
        }
        let mut copy = rider_update.to_vec();
        copy[2..10].copy_from_slice(&self.actor_guid.to_le_bytes());
        Some(copy)
    }

    pub fn packets(&self, pose: RidePose) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        out.extend(add_ride_npc(&self.mount, &self.look, self.actor_guid, pose));
        if let Some(seat) = self.seat {
            // Effect 0: the viewer walked up, so the boarding flash must not replay (T483).
            out.push(encode_mount_response(
                self.rider_guid,
                self.actor_guid,
                seat,
                0,
            ));
        }
        out
    }
}

impl MountService {
    /// (R) The ride's world GUID for an owner.
    pub fn ride_guid(owner_guid: u64) -> u64 {
        owner_guid ^ RIDE_GUID_TAG
    }

    pub fn view(&self, owner_guid: u64) -> Option<RideView> {
        let ride = self.state.active_ride()?;
        Some(RideView {
            rider_guid: owner_guid,
            actor_guid: ride.actor_guid,
            seat: match self.state {
                MountState::Mounted { seat_index, .. } => Some(seat_index),
                _ => None,
            },
            mount: self.catalog.mount_arc(ride.mount_id)?,
            look: ride.look.clone(),
        })
    }

    /// A refusal changes nothing and sends the owner a roster refresh, which makes the client
    /// rebuild its mount UI, plus a short notice (R: no refusal packet was recovered).
    pub fn refusal(&mut self, inventory: &InventoryState, error: MountError) -> RideWire {
        RideWire {
            private: vec![self.list_packet(inventory)],
            notice: Some(refusal_text(error)),
            ..RideWire::default()
        }
    }

    /// Answer one client request. A refusal changes nothing and sends the owner a roster refresh,
    /// which makes the client rebuild its mount UI, plus a short notice (R: no refusal packet was
    /// recovered).
    pub fn handle(&mut self, inner: &[u8], ctx: &RideContext) -> io::Result<Option<RideWire>> {
        let Some(request) = decode_request(inner)? else {
            return Ok(None);
        };
        let owner = ctx.guid;
        let actor = Self::ride_guid(owner);
        let result = match request {
            // T527: gameplay routes 168/14 before this service because consuming the saddle
            // belongs to the inventory it holds; the session-scoped service never commits one.
            MountRequest::Upgrade { .. } => return Ok(None),
            MountRequest::Spawn { mount_id } => self.spawn_row(
                owner,
                ctx.inventory,
                mount_id,
                actor,
                ctx.restrictions,
                ctx.now_ms,
                &[],
            ),
            MountRequest::SpawnByItem { item_definition_id } => self.spawn_by_item_definition(
                owner,
                ctx.inventory,
                item_definition_id,
                actor,
                ctx.restrictions,
                ctx.now_ms,
                &[],
            ),
            // (R) The request names no seat; every shipped-slice ride has one seat.
            MountRequest::Mount { target } => self.mount(
                owner,
                owner,
                ctx.inventory,
                target,
                0,
                ctx.restrictions,
                &[],
            ),
            MountRequest::Dismount => Ok(self.dismount_request(owner, ctx.inventory, &[])),
        };
        Ok(Some(match result {
            Ok(outcome) => self.wire(&outcome, ctx.pose),
            Err(error) => self.refusal(ctx.inventory, error),
        }))
    }

    /// Turn an outcome into inner packets. Ride actors are AddNpc / remove-actor packets.
    pub fn wire(&self, outcome: &MountOutcome, pose: RidePose) -> RideWire {
        let mut wire = RideWire::default();
        for effect in &outcome.effects {
            let target = if effect.nearby() {
                &mut wire.shared
            } else {
                &mut wire.private
            };
            match &effect.kind {
                MountEffectKind::Packet(bytes) => target.push(bytes.clone()),
                MountEffectKind::SpawnRide {
                    mount_id,
                    actor_guid,
                    look,
                    ..
                } => {
                    if let Some(mount) = self.catalog.mount_arc(*mount_id) {
                        target.extend(add_ride_npc(&mount, look, *actor_guid, pose));
                    }
                }
                MountEffectKind::DespawnRide { actor_guid } => {
                    target.push(remove_actor(*actor_guid))
                }
                MountEffectKind::RiderStats { rider_guid, stats } => target.extend(
                    world::unframe_tunnel(&startup_catalogues::build_stat_update(StatUpdate {
                        target_guid: *rider_guid,
                        stats,
                    }))
                    .ok(),
                ),
            }
        }
        wire
    }
}

fn refusal_text(error: MountError) -> &'static str {
    match error {
        MountError::NotOwned => "You do not own that ride.",
        MountError::NotUpgradable => "That ride cannot learn Gliding.",
        MountError::AlreadyUpgraded => "That ride already knows Gliding.",
        MountError::NotUpgradeItem => "That item cannot train a ride.",
        MountError::UpgradeItemNotOwned => "You do not have that training saddle.",
        MountError::Restricted => "You cannot use a ride here right now.",
        _ => "That ride is not available.",
    }
}

/// AddNpc for a ride. Field values come from the captured content template (`world.rs`), made
/// non-interactable; the model and name come from the ride record, the texture and tint from the
/// owned item (`RideLook`). The animation and seat definitions live in the client's actor
/// definition, not in this packet (R for everything else). The nameplate is hidden as Sanctuary
/// does for a ride (R).
fn add_ride_npc(
    mount: &MountDefinition,
    look: &RideLook,
    actor_guid: u64,
    pose: RidePose,
) -> Option<Vec<u8>> {
    let npc = startup_world::NpcSpawn {
        actor_id: actor_guid,
        name_string_id: mount.name_string_id,
        model_id: mount.model_id?,
        texture_alias: look.texture_alias.clone(),
        tint_alias: look.tint_alias.clone(),
        tint_id: look.tint_id,
        hide_nameplate: 1,
        speed: mount.max_movement_speed,
        is_interactable: 0,
        interact_range: 0,
        position: [pose.position[0], pose.position[1], pose.position[2], 1.0],
        rotation: [pose.heading.sin(), 0.0, pose.heading.cos(), 0.0],
        ..world::content_spawn_template()
    };
    world::unframe_tunnel(&startup_world::build_add_npc(&npc)).ok()
}

/// Remove-actor packet, the 35/3 shape the opening encounter and field enemies use (R).
fn remove_actor(guid: u64) -> Vec<u8> {
    let mut out = vec![35, 0, 3, 0, 1, 0];
    out.extend_from_slice(&guid.to_le_bytes());
    out.extend_from_slice(&[0; 17]);
    out
}

/// Client reader: 00b06100 case 5 -> 00b05e00 -> 00b05830 (i32 count) -> 008e74c0 per record;
/// 00b05600 then replaces the BaseClient mount store and refreshes the UI.
///
/// T482: one record per owned ride item. The id is the inventory instance id, which the client
/// sends back in 168/6 (`00b038c0`); the tint is the instance's own (0 when undyed, as the
/// captured rows show); the alias is the item's catalogue alias, else the ride's.
fn encode_mount_list(
    rows: &[(&InventoryItem, &MountDefinition)],
    active: Option<&ActiveRide>,
    looks: &BTreeMap<i32, ItemLook>,
    upgraded: &BTreeSet<i32>,
) -> Vec<u8> {
    let mut out = header(LIST);
    put_i32(&mut out, rows.len() as i32);
    for (item, mount) in rows {
        put_i32(&mut out, item.id);
        put_i32(&mut out, mount.name_string_id);
        put_i32(&mut out, mount.image_set_id);
        put_u64(
            &mut out,
            active
                .filter(|ride| ride.instance_id == item.id)
                .map_or(0, |ride| ride.actor_guid),
        );
        out.push(u8::from(mount.members_only));
        put_i32(&mut out, item.tint);
        let alias = looks.get(&item.definition_id);
        put_string(
            &mut out,
            alias.map_or(&mount.tint_alias, |look| &look.tint_alias),
        );
        out.push(u8::from(mount.is_upgradable));
        // T527: training is per owned instance, so two copies of one ride differ here.
        out.push(u8::from(trained(mount, item.id, upgraded)));
    }
    out
}

/// Client reader `00b04c80` (read by `00b06100` case 2): rider, mount, seat, queue, Mount's third
/// argument, then the i32 stored on the rider (+0x17bc) that `Mount` (`00972d30`) looks up
/// (`007c4710`) and plays as a composite effect, then a float.
fn encode_mount_response(
    rider_guid: u64,
    mount_guid: u64,
    seat_index: u32,
    effect_id: i32,
) -> Vec<u8> {
    let mut out = header(2);
    put_u64(&mut out, rider_guid);
    put_u64(&mut out, mount_guid);
    put_i32(&mut out, seat_index as i32);
    put_i32(&mut out, SEATED_QUEUE); // queue position; the client mounts when it is 1 (00b06100)
    put_i32(&mut out, SEATED_QUEUE); // (R) Mount's fourth argument, as in the AddPc tail
    put_i32(&mut out, effect_id); // stored on the rider (+0x17bc): the boarding flash, 0 = none
    put_f32(&mut out, 0.0); // (R) Mount's last argument; keep 0
    out
}

/// Client reader `00b04dc0` (read by `00b06100` case 4): rider, then the i32 stored on the rider
/// (+0x17c0) that `Dismount` (`00973050`) looks up (`007c4710`) and plays like the boarding one.
fn encode_dismount_response(rider_guid: u64, effect_id: i32) -> Vec<u8> {
    let mut out = header(4);
    put_u64(&mut out, rider_guid);
    put_i32(&mut out, effect_id); // the dismount flash, 0 = none
    out
}

fn header(sub_opcode: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(3);
    out.extend_from_slice(&FAMILY.to_le_bytes());
    out.push(sub_opcode);
    out
}

fn packet(recipients: Vec<u64>, bytes: Vec<u8>) -> MountEffect {
    MountEffect {
        recipients,
        kind: MountEffectKind::Packet(bytes),
    }
}

fn rider_stats(rider_guid: u64, stats: Vec<CharacterStat>) -> MountEffect {
    MountEffect {
        recipients: vec![rider_guid],
        kind: MountEffectKind::RiderStats { rider_guid, stats },
    }
}

/// Whether this owned instance knows Gliding: the ride definition's own flag, or the saved
/// per-instance training (T527). The flag lives with the inventory instance, so two copies of
/// one ride train separately.
fn trained(mount: &MountDefinition, instance_id: i32, upgraded: &BTreeSet<i32>) -> bool {
    mount.is_upgraded || upgraded.contains(&instance_id)
}

fn float_stat(id: i32, value: f32) -> CharacterStat {
    CharacterStat {
        id,
        value: StatValue::Float(value),
    }
}

fn int_stat(id: i32, value: i32) -> CharacterStat {
    CharacterStat {
        id,
        value: StatValue::Integer(value),
    }
}

fn recipients(owner_guid: u64, observers: &[u64]) -> Vec<u64> {
    let mut recipients = vec![owner_guid];
    for observer in observers.iter().copied() {
        if observer != 0 && observer != owner_guid && !recipients.contains(&observer) {
            recipients.push(observer);
        }
    }
    recipients
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    put_i32(out, value.len() as i32);
    out.extend_from_slice(value.as_bytes());
}

fn valid_sources(sources: &[MountSource]) -> bool {
    !sources.is_empty()
        && sources.iter().all(|source| match source {
            MountSource::Client(value)
            | MountSource::Legacy(value)
            | MountSource::Wiki(value)
            | MountSource::Reconstructed(value) => !value.trim().is_empty(),
        })
}

/// T527: the item definitions 168/14 accepts, validated and deduplicated.
fn upgrade_item_ids(items: Vec<UpgradeItem>) -> Result<BTreeSet<i32>, MountError> {
    if items.len() > MAX_UPGRADE_ITEMS {
        return Err(MountError::InvalidCatalog);
    }
    let mut ids = BTreeSet::new();
    for item in items {
        if item.item_definition_id <= 0
            || !valid_sources(&item.source)
            || !ids.insert(item.item_definition_id)
        {
            return Err(MountError::InvalidCatalog);
        }
    }
    Ok(ids)
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
