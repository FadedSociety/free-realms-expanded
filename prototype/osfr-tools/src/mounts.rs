//! Session-scoped mount ownership and actor lifecycle.
//!
//! Mount packets are synthetic inner messages based on the client readers in
//! `notes/Build Guides/Gameplay/Mounts.md`; datagram byte order is unverified.

use crate::inventory::InventoryState;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

const FAMILY: u16 = 168;
const MAX_CONTENT_BYTES: usize = 4 * 1024 * 1024;
// (R) Server-side duplicate guard mirrors the client's 00a2e540 request window.
const SPAWN_DEBOUNCE_MS: u64 = 1_000;

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
    pub source: Vec<MountSource>,
    #[serde(default)]
    pub field_sources: BTreeMap<String, Vec<MountSource>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MountsDocument {
    schema_version: u32,
    mount_types: Vec<MountType>,
    mounts: Vec<MountDefinition>,
}

#[derive(Clone, Debug)]
pub struct MountCatalog {
    types: BTreeMap<u32, MountType>,
    mounts: BTreeMap<i32, MountDefinition>,
    item_to_mount: BTreeMap<i32, i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MountError {
    InvalidCatalog,
    InvalidGuid,
    UnknownMount,
    UnknownMountItem,
    NotOwned,
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
        if document.schema_version != 1 {
            return Err(invalid_data("unsupported mount content schema"));
        }
        Self::new(document.mount_types, document.mounts)
            .map_err(|_| invalid_data("invalid mount content records"))
    }

    pub fn new(
        mount_types: Vec<MountType>,
        mounts: Vec<MountDefinition>,
    ) -> Result<Self, MountError> {
        if mount_types.is_empty()
            || mount_types.len() > 128
            || mounts.is_empty()
            || mounts.len() > 4096
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
                || !mount.max_movement_speed.is_finite()
                || mount.max_movement_speed < 0.0
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
            records.insert(mount_id, mount);
        }
        Ok(Self {
            types,
            mounts: records,
            item_to_mount,
        })
    }

    pub fn mount_types(&self) -> impl Iterator<Item = &MountType> {
        self.types.values()
    }

    pub fn mounts(&self) -> impl Iterator<Item = &MountDefinition> {
        self.mounts.values()
    }

    pub fn mount(&self, mount_id: i32) -> Option<&MountDefinition> {
        self.mounts.get(&mount_id)
    }

    pub fn mount_for_item(&self, item_definition_id: i32) -> Option<&MountDefinition> {
        self.item_to_mount
            .get(&item_definition_id)
            .and_then(|mount_id| self.mounts.get(mount_id))
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
        self.mounts
            .values()
            .filter(|mount| {
                mount
                    .item_definition_id_candidates
                    .iter()
                    .any(|item_id| owned_items.contains(item_id))
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RideRestrictions {
    /// Zone and activity restrictions are reconstructed server policy (R).
    pub zone_allows_riding: bool,
    pub in_minigame: bool,
    pub ability_active: bool,
}

impl RideRestrictions {
    fn allows(self) -> bool {
        self.zone_allows_riding && !self.in_minigame && !self.ability_active
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveRide {
    pub mount_id: i32,
    pub actor_guid: u64,
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
    },
    DespawnRide {
        actor_guid: u64,
    },
    /// Synthetic inner application packet, before gateway tunnel wrapping.
    Packet(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct MountEffect {
    /// Character GUID recipients; nearby observers come from Players interest.
    pub recipients: Vec<u64>,
    pub kind: MountEffectKind,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MountOutcome {
    pub effects: Vec<MountEffect>,
    pub changed: bool,
    pub debounced: bool,
}

#[derive(Clone, Debug)]
pub struct MountService {
    catalog: MountCatalog,
    state: MountState,
    last_spawn_ms: Option<u64>,
}

impl MountService {
    pub fn new(catalog: MountCatalog) -> Self {
        Self {
            catalog,
            state: MountState::Empty,
            last_spawn_ms: None,
        }
    }

    pub fn state(&self) -> &MountState {
        &self.state
    }

    pub fn catalog(&self) -> &MountCatalog {
        &self.catalog
    }

    pub fn list_packet(&self, inventory: &InventoryState) -> Vec<u8> {
        encode_mount_list(&self.catalog, inventory, self.state.active_ride())
    }

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
        // (R) A ride actor must have its own nonzero world GUID.
        if owner_guid == 0 || actor_guid == 0 || actor_guid == owner_guid {
            return Err(MountError::InvalidGuid);
        }
        if !restrictions.allows() {
            return Err(MountError::Restricted);
        }
        let mount = self
            .catalog
            .mount(mount_id)
            .ok_or(MountError::UnknownMount)?;
        if !is_owned(mount, inventory) {
            return Err(MountError::NotOwned);
        }
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
        // (R) Replace any prior summoned actor after the client spawn handshake.
        if let Some(old_ride) = self.state.active_ride().cloned() {
            if let MountState::Mounted { rider_guid, .. } = &self.state {
                outcome.effects.push(packet(
                    recipients.clone(),
                    encode_dismount_response(*rider_guid),
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
            actor_guid,
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
            },
        });
        Ok(outcome)
    }

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
        let mount_id = self
            .catalog
            .mount_for_item(item_definition_id)
            .map(|mount| mount.id)
            .ok_or(MountError::UnknownMountItem)?;
        self.spawn(
            owner_guid,
            inventory,
            mount_id,
            actor_guid,
            restrictions,
            now_ms,
            observers,
        )
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
        if !is_owned(mount, inventory) {
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
            effects: vec![packet(
                recipients(owner_guid, observers),
                encode_mount_response(rider_guid, ride.actor_guid, seat_index),
            )],
            debounced: false,
        })
    }

    /// 168/3 is shared by Lua `Dismount` and `Despawn`; state is the only
    /// available discriminator. First detach a rider, then a later request
    /// removes the now-unridden summoned actor (R).
    pub fn dismount_request(
        &mut self,
        owner_guid: u64,
        inventory: &InventoryState,
        observers: &[u64],
    ) -> MountOutcome {
        let all_recipients = recipients(owner_guid, observers);
        match self.state.clone() {
            // (R) Empty dismount requests are idempotent no-ops.
            MountState::Empty => MountOutcome::default(),
            MountState::Mounted {
                ride, rider_guid, ..
            } => {
                self.state = MountState::Spawned(ride);
                MountOutcome {
                    changed: true,
                    effects: vec![packet(all_recipients, encode_dismount_response(rider_guid))],
                    debounced: false,
                }
            }
            MountState::Spawned(ride) => {
                self.state = MountState::Empty;
                MountOutcome {
                    changed: true,
                    effects: vec![
                        MountEffect {
                            recipients: all_recipients,
                            kind: MountEffectKind::DespawnRide {
                                actor_guid: ride.actor_guid,
                            },
                        },
                        packet(vec![owner_guid], self.list_packet(inventory)),
                    ],
                    debounced: false,
                }
            }
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

fn is_owned(mount: &MountDefinition, inventory: &InventoryState) -> bool {
    inventory.items.iter().any(|item| {
        item.count > 0
            && mount
                .item_definition_id_candidates
                .contains(&item.definition_id)
    })
}

fn encode_mount_list(
    catalog: &MountCatalog,
    inventory: &InventoryState,
    active: Option<&ActiveRide>,
) -> Vec<u8> {
    let mounts = catalog.owned_mounts(inventory);
    let mut out = header(5);
    put_i32(&mut out, mounts.len() as i32);
    for mount in mounts {
        put_i32(&mut out, mount.id);
        put_i32(&mut out, mount.name_string_id);
        put_i32(&mut out, mount.image_set_id);
        put_u64(
            &mut out,
            active
                .filter(|ride| ride.mount_id == mount.id)
                .map_or(0, |ride| ride.actor_guid),
        );
        out.push(u8::from(mount.members_only));
        put_i32(&mut out, mount.tint_id);
        put_string(&mut out, &mount.tint_alias);
        out.push(u8::from(mount.is_upgradable));
        out.push(u8::from(mount.is_upgraded));
    }
    out
}

fn encode_mount_response(rider_guid: u64, mount_guid: u64, seat_index: u32) -> Vec<u8> {
    let mut out = header(2);
    put_u64(&mut out, rider_guid);
    put_u64(&mut out, mount_guid);
    put_i32(&mut out, seat_index as i32);
    put_i32(&mut out, 1); // (R) accepted result
    put_i32(&mut out, 0); // (R) queue position
    put_i32(&mut out, 0); // (R) unresolved response field
    put_f32(&mut out, 0.0); // (R) unresolved response field
    out
}

fn encode_dismount_response(rider_guid: u64) -> Vec<u8> {
    let mut out = header(4);
    put_u64(&mut out, rider_guid);
    put_i32(&mut out, 0); // (R) unresolved dismount result
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

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
