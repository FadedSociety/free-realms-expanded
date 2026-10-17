//! Content-backed zone instances, player membership, and transfer staging.
//!
//! World owns membership and field runtime under the gateway single writer.
//! Encounter calculates mutations; the gateway stages persistence and delivery.

use crate::{
    content::{ContentDb, Dungeon, NpcSpawn as ContentNpcSpawn, Rotation, Zone},
    startup_messages::{self, ZoneDetails},
    startup_world::{self, NpcSpawn},
    zone_transition::{self, ZoneTransition},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

const NPC_VISIBILITY_RADIUS: f64 = 100.0; // (R) Reuses the local player-interest radius.
const NPC_VISIBILITY_LIMIT: usize = 64; // (R) Reuses the local player-interest cap.
const MAX_NPC_DATA_BYTES: u64 = 16 * 1024 * 1024;
/// (R) Regional `zone_id`s are content package labels (NPC Placement note), not
/// native zones: every regional and Legacy spawn shares the one continent
/// coordinate space, whose spawn point matches the local world zone 0.
pub(crate) const OPEN_WORLD_ZONE: u32 = 0;

/// An NPC placed with the Studio placer (`npc_placements.json`, beside the
/// Legacy spawns so their provenance stays separate).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Placement {
    pub guid: u64,
    pub model_id: u32,
    /// Client locale text id for the nameplate; 0 shows no name.
    #[serde(default)]
    pub name_string_id: u32,
    /// Label for tools; the client shows `name_string_id`.
    #[serde(default)]
    pub name: Option<String>,
    pub position: [f32; 3],
    #[serde(default)]
    pub heading: f32,
    #[serde(default = "unit_scale")]
    pub scale: f32,
    /// Other fields (`source`, notes) are kept when tools rewrite the file.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn unit_scale() -> f32 {
    1.0
}

/// Placed NPC GUIDs start here, clear of Legacy (1e11 + id) and battle ranges.
pub const PLACEMENT_GUID_BASE: u64 = 130_000_000_000;

#[derive(Deserialize, Serialize)]
pub struct PlacementFile {
    pub schema_version: u32,
    pub records: Vec<Placement>,
}

#[derive(Clone)]
struct VisibleNpc {
    spawn: NpcSpawn,
    name: Option<String>,
    position: [f32; 3],
}

#[derive(Clone, Default)]
pub struct NpcVisibility {
    // (R) Parsed content is immutable during play; staged Gameplay clones share
    // it. Host content edits use copy-on-write, keeping rollback independent.
    by_zone: Arc<BTreeMap<u32, Vec<VisibleNpc>>>,
    content_keys: HashMap<u64, u64>,
    excluded: HashSet<u64>,
    visible: BTreeSet<u64>,
    zone_id: Option<u32>,
    // Where the last update looked from, so content reloads can resend.
    last_position: Option<[f32; 3]>,
    placed: BTreeSet<u64>,
    shared: bool,
    revision: u64,
}

/// (R) One candidate in the common passive/hostile NPC budget. Engaged actors
/// retain priority inside the radius; remaining ties use distance then GUID.
#[derive(Clone, Copy, PartialEq)]
pub struct NpcInterest {
    pub guid: u64,
    pub position: [f32; 3],
    pub engaged: bool,
}

#[derive(Deserialize)]
struct Rows<T> {
    records: Vec<T>,
}

#[derive(Deserialize)]
struct ZoneName {
    id: u32,
    name: String,
}

#[derive(Deserialize)]
struct RegionalNpc {
    npc_id: u64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    zone_id: Option<u32>,
    #[serde(default)]
    zone: Option<String>,
    #[serde(default)]
    model_id: Option<u32>,
    #[serde(default)]
    name_string_id: Option<u32>,
    #[serde(default)]
    position: Option<[f32; 3]>,
    #[serde(default)]
    position_w: Option<f32>,
    #[serde(default)]
    rotation: Option<[f32; 4]>,
    #[serde(default)]
    heading: Option<f32>,
    #[serde(default)]
    scale: Option<f32>,
    #[serde(default)]
    placement: Option<String>,
}

#[derive(Deserialize)]
struct ExtraNpc {
    spawn_guid: u64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    model_id: u32,
    #[serde(default)]
    model_file_name: Option<String>,
    #[serde(default)]
    name_string_id: Option<u32>,
    position: [f32; 3],
    #[serde(default)]
    position_w: Option<f32>,
    #[serde(default)]
    heading: f32,
    #[serde(default)]
    capture_match: Option<serde_json::Value>,
}

impl NpcVisibility {
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn enable_shared_interest(&mut self) {
        self.shared = true;
    }

    /// (R) Pure directed diff for the single writer. The caller owns the full
    /// known-NPC set; passive startup/movement/reload cannot bypass this budget.
    pub fn shared_view(
        &self,
        zone: u32,
        position: [f32; 3],
        shared: &[NpcInterest],
        known: &BTreeSet<u64>,
    ) -> io::Result<(BTreeSet<u64>, Vec<Vec<u8>>)> {
        if !position.iter().all(|v| v.is_finite()) {
            return Err(invalid("NPC visibility position is not finite"));
        }
        let passive = self
            .by_zone
            .get(&zone)
            .into_iter()
            .flatten()
            .filter(|npc| !self.excluded.contains(&npc.spawn.actor_id));
        let mut candidates: Vec<_> = passive
            .clone()
            .map(|npc| NpcInterest {
                guid: npc.spawn.actor_id,
                position: npc.position,
                engaged: false,
            })
            .chain(shared.iter().copied())
            .filter(|npc| npc.position.iter().all(|v| v.is_finite()))
            .map(|mut npc| {
                // (R) Retain an observed engagement; do not evict a viewer's
                // entire passive set for an unrelated fight they never saw.
                npc.engaged = npc.engaged && known.contains(&npc.guid);
                (npc, distance_squared(position, npc.position))
            })
            .filter(|(_, d)| *d <= NPC_VISIBILITY_RADIUS.powi(2))
            .collect();
        candidates.sort_by(|(a, da), (b, db)| {
            b.engaged
                .cmp(&a.engaged)
                .then(da.total_cmp(db))
                .then(a.guid.cmp(&b.guid))
        });
        let mut next = BTreeSet::new();
        for (npc, _) in candidates {
            next.insert(npc.guid);
            if next.len() == NPC_VISIBILITY_LIMIT {
                break;
            }
        }
        let mut packets = Vec::new();
        for guid in known.difference(&next) {
            packets.push(crate::encounter::SharedFieldState::removal(*guid));
        }
        for npc in passive {
            if next.contains(&npc.spawn.actor_id) && !known.contains(&npc.spawn.actor_id) {
                packets.push(unframe_tunnel(&startup_world::build_add_npc(&npc.spawn))?);
            }
        }
        Ok((next, packets))
    }

    /// Loads regional NPC rows and the zone-less Legacy placement table.
    /// Extra spawn zones use nearest positioned regional NPC as local policy (R).
    pub fn load(content_dir: &Path) -> io::Result<Self> {
        let zones: Rows<ZoneName> = read_rows(&content_dir.join("zones.json"))?;
        let zone_ids: HashMap<_, _> = zones
            .records
            .into_iter()
            .map(|zone| (zone.name.to_lowercase(), zone.id))
            .collect();
        let mut regional = Vec::new();
        let npc_dir = content_dir.join("npcs");
        for entry in fs::read_dir(&npc_dir)? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let rows: Rows<RegionalNpc> = read_rows(&path)?;
                regional.extend(rows.records);
            }
        }
        regional.sort_by_key(|npc| npc.npc_id);

        let mut result = Self::default();
        let mut anchors = Vec::new();
        for npc in regional {
            let Some(position) = npc.position.filter(|p| p.iter().all(|v| v.is_finite())) else {
                continue;
            };
            let zone_id = npc
                .zone_id
                .or_else(|| {
                    npc.zone
                        .as_ref()
                        .and_then(|name| zone_ids.get(&name.to_lowercase()).copied())
                })
                // (R) Positioned legacy_data rows without a regional label are
                // part of the shared open-world coordinate space.
                .unwrap_or(OPEN_WORLD_ZONE);
            anchors.push((zone_id, position));
            // Only data-backed placements spawn; reconstructed POI/zone-centre
            // positions are build scaffolding, not NPC transforms.
            if npc.placement.as_deref() != Some("legacy_data") {
                continue;
            }
            // Content `rotation` is a Y-axis quaternion; AddNpc wants a facing
            // direction, so prefer the heading and fall back to the quaternion yaw.
            let heading = npc
                .heading
                .unwrap_or_else(|| npc.rotation.map(|q| 2.0 * q[1].atan2(q[3])).unwrap_or(0.0));
            let rotation = facing(heading);
            let mut spawn = NpcSpawn {
                actor_id: npc.npc_id,
                name_string_id: npc.name_string_id.unwrap_or(0) as i32,
                model_id: npc.model_id.unwrap_or(0) as i32,
                ..content_spawn_template()
            };
            // (R) Regional rows without a model or string ID retain zero-valued fields.
            spawn.position = [
                position[0],
                position[1],
                position[2],
                npc.position_w.unwrap_or(1.0),
            ];
            spawn.rotation = rotation;
            spawn.scale = npc.scale.unwrap_or(1.0);
            result.content_keys.insert(npc.npc_id, npc.npc_id);
            Arc::make_mut(&mut result.by_zone)
                .entry(OPEN_WORLD_ZONE)
                .or_default()
                .push(VisibleNpc {
                    spawn,
                    name: npc.name,
                    position,
                });
        }
        if anchors.is_empty() {
            return Err(invalid("NPC content has no positioned regional anchors"));
        }

        let extras: Rows<ExtraNpc> = read_rows(&content_dir.join("npc_spawns_extra.json"))?;
        for npc in extras.records {
            // The client-assets catalog contains knocker_m_boss.adr but no
            // knocker_m_boss.dme base model; native NPC creation fails for it.
            // Filter at load time so authored content remains provenance intact.
            if npc.capture_match.is_some()
                || npc
                    .model_file_name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case("knocker_m_boss.adr"))
                || !npc.position.iter().all(|v| v.is_finite())
            {
                continue;
            }
            let zone_id = anchors
                .iter()
                .min_by(|left, right| {
                    distance_squared(npc.position, left.1)
                        .total_cmp(&distance_squared(npc.position, right.1))
                        .then_with(|| left.0.cmp(&right.0))
                })
                .map(|anchor| anchor.0)
                .expect("anchors checked above");
            let mut spawn = NpcSpawn {
                actor_id: npc.spawn_guid,
                name_string_id: npc.name_string_id.unwrap_or(0) as i32,
                model_id: npc.model_id as i32,
                ..content_spawn_template()
            };
            spawn.position = [
                npc.position[0],
                npc.position[1],
                npc.position[2],
                npc.position_w.unwrap_or(1.0),
            ];
            spawn.rotation = facing(npc.heading);
            let _ = zone_id; // nearest-anchor label kept for diagnostics only
            Arc::make_mut(&mut result.by_zone)
                .entry(OPEN_WORLD_ZONE)
                .or_default()
                .push(VisibleNpc {
                    spawn,
                    name: npc.name,
                    position: npc.position,
                });
        }
        let placements = content_dir.join("npc_placements.json");
        if placements.is_file() {
            let rows: Rows<Placement> = read_rows(&placements)?;
            for placement in &rows.records {
                result.place(placement);
            }
        }
        for npcs in Arc::make_mut(&mut result.by_zone).values_mut() {
            npcs.sort_by_key(|npc| npc.spawn.actor_id);
        }
        Ok(result)
    }

    /// Add or replace a placed NPC in the open world. Call [`Self::remove`]
    /// first when it may already be visible, so the client drops the old one.
    pub fn place(&mut self, placement: &Placement) {
        if !placement.position.iter().all(|v| v.is_finite()) {
            return;
        }
        self.revision = self.revision.wrapping_add(1);
        let mut spawn = NpcSpawn {
            actor_id: placement.guid,
            name_string_id: placement.name_string_id as i32,
            model_id: placement.model_id as i32,
            ..content_spawn_template()
        };
        let p = placement.position;
        spawn.position = [p[0], p[1], p[2], 1.0];
        spawn.rotation = facing(placement.heading);
        spawn.scale = placement.scale;
        // AddNpc's literal name; whether the nameplate shows it without a
        // locale id is the World Builder native text check.
        spawn.npc_name = placement.name.clone().unwrap_or_default();
        let npcs = Arc::make_mut(&mut self.by_zone)
            .entry(OPEN_WORLD_ZONE)
            .or_default();
        npcs.retain(|npc| npc.spawn.actor_id != placement.guid);
        npcs.push(VisibleNpc {
            spawn,
            name: placement.name.clone(),
            position: p,
        });
        npcs.sort_by_key(|npc| npc.spawn.actor_id);
        self.placed.insert(placement.guid);
    }

    /// Swap every placed NPC for `placements` (a reload of
    /// `npc_placements.json`), returning the 35/3 removals and 35/2 additions.
    pub fn replace_placements(&mut self, placements: &[Placement]) -> io::Result<Vec<Vec<u8>>> {
        for guid in std::mem::take(&mut self.placed) {
            self.remove(guid);
        }
        // Drop the old copies from the client before sending the new ones.
        let mut packets = self.refresh()?;
        for placement in placements {
            self.place(placement);
        }
        packets.extend(self.refresh()?);
        Ok(packets)
    }

    /// Re-run the last visibility update (after content changed).
    pub fn refresh(&mut self) -> io::Result<Vec<Vec<u8>>> {
        match (self.zone_id, self.last_position) {
            (Some(zone), Some(position)) => self.update(zone, position),
            _ => Ok(Vec::new()),
        }
    }

    /// Remove an actor; the next [`Self::update`] sends its 35/3 removal.
    pub fn remove(&mut self, actor_id: u64) -> bool {
        self.revision = self.revision.wrapping_add(1);
        self.placed.remove(&actor_id);
        let npcs = Arc::make_mut(&mut self.by_zone)
            .entry(OPEN_WORLD_ZONE)
            .or_default();
        let before = npcs.len();
        npcs.retain(|npc| npc.spawn.actor_id != actor_id);
        npcs.len() != before
    }

    /// Add a static content actor (e.g. a battle spawner) to the open world.
    /// It is sent by proximity like every other NPC: the client ignores actors
    /// spawned far away at login (spawner unseen in native runs 015208/015919).
    pub fn add_static_actor(
        &mut self,
        actor_id: u64,
        name: &str,
        name_string_id: i32,
        model_id: i32,
        position: [f32; 3],
        nameplate_image_id: i32,
    ) {
        self.revision = self.revision.wrapping_add(1);
        let mut spawn = NpcSpawn {
            actor_id,
            name_string_id,
            model_id,
            nameplate_image_id,
            ..content_spawn_template()
        };
        if nameplate_image_id > 0 {
            // The overhead icon falls back to NameplateImageId only when the
            // ActiveProfile does not select a configured icon first
            // (009622f0); a plain actor uses profile 0.
            spawn.active_profile = 0;
        }
        spawn.position = [position[0], position[1], position[2], 1.0];
        spawn.rotation = facing(0.0);
        let npcs = Arc::make_mut(&mut self.by_zone)
            .entry(OPEN_WORLD_ZONE)
            .or_default();
        npcs.retain(|npc| npc.spawn.actor_id != actor_id);
        npcs.push(VisibleNpc {
            spawn,
            name: Some(name.to_string()),
            position,
        });
        npcs.sort_by_key(|npc| npc.spawn.actor_id);
    }

    pub fn exclude_actor_guids(&mut self, actor_guids: impl IntoIterator<Item = u64>) {
        self.revision = self.revision.wrapping_add(1);
        self.excluded.extend(actor_guids);
        self.visible.retain(|guid| !self.excluded.contains(guid));
    }

    pub fn content_key(&self, runtime_guid: u64) -> Option<u64> {
        (!self.excluded.contains(&runtime_guid))
            .then(|| self.content_keys.get(&runtime_guid).copied())
            .flatten()
    }

    /// Placed actor model ids and positions, for content-defined scene fixtures
    /// such as crafting stations. Zones use the open-world label recorded by
    /// the loader.
    pub fn model_actors(&self) -> impl Iterator<Item = (u32, u32, [f32; 3])> + '_ {
        self.by_zone.iter().flat_map(|(&zone_id, actors)| {
            actors.iter().map(move |actor| {
                (
                    zone_id,
                    u32::try_from(actor.spawn.model_id).unwrap_or(0),
                    actor.position,
                )
            })
        })
    }

    /// Content NPC targets for the developer teleport index. The same rows
    /// back 35/2 visibility: regional `legacy_data` placements and renderable
    /// extra spawns only. Runtime-excluded startup actors stay excluded.
    pub fn teleport_targets(&self, zone_id: u32) -> Vec<crate::dev_teleport::Target> {
        self.by_zone
            .get(&zone_id)
            .into_iter()
            .flatten()
            .filter(|npc| !self.excluded.contains(&npc.spawn.actor_id))
            .map(|npc| {
                crate::dev_teleport::Target::content_npc(
                    npc.spawn.actor_id,
                    npc.name.as_deref(),
                    zone_id,
                    npc.position,
                )
            })
            .collect()
    }

    /// Returns unframed 35/2 additions and 35/3 removals for the current view.
    pub fn update(&mut self, zone_id: u32, position: [f32; 3]) -> io::Result<Vec<Vec<u8>>> {
        if !position.iter().all(|value| value.is_finite()) {
            return Err(invalid("NPC visibility position is not finite"));
        }
        if self.shared {
            return Ok(Vec::new()); // (R) Fan-out only in Server's 20 Hz tick.
        }
        if self.zone_id != Some(zone_id) {
            self.visible.clear();
            self.zone_id = Some(zone_id);
        }
        self.last_position = Some(position);
        let mut candidates: Vec<_> = self
            .by_zone
            .get(&zone_id)
            .into_iter()
            .flatten()
            .filter(|npc| !self.excluded.contains(&npc.spawn.actor_id))
            .map(|npc| (distance_squared(position, npc.position), npc))
            .filter(|(distance, _)| *distance <= NPC_VISIBILITY_RADIUS * NPC_VISIBILITY_RADIUS)
            .collect();
        candidates.sort_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.spawn.actor_id.cmp(&right.1.spawn.actor_id))
        });
        let next: BTreeSet<_> = candidates
            .iter()
            .take(NPC_VISIBILITY_LIMIT)
            .map(|(_, npc)| npc.spawn.actor_id)
            .collect();
        let mut packets = Vec::new();
        for guid in self.visible.difference(&next) {
            let mut bytes = vec![35, 0, 3, 0];
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(guid.to_le_bytes());
            bytes.extend([0; 17]);
            packets.push(bytes);
        }
        for (_, npc) in candidates.into_iter().take(NPC_VISIBILITY_LIMIT) {
            if !self.visible.contains(&npc.spawn.actor_id) {
                let framed = startup_world::build_add_npc(&npc.spawn);
                packets.push(unframe_tunnel(&framed)?);
            }
        }
        self.visible = next;
        Ok(packets)
    }
}

/// Base AddNpc fields for content spawns: the values shared by all 23 captured
/// startup AddNpc rows (`startup.json` `world.npc_spawns`, reader `00921a10`).
/// The derived all-zero default left content NPCs non-interactable natively
/// (run 034759: `is_interactable` 0, `interact_range` 0). Captured-constant
/// fields whose meaning is unresolved are copied as-is.
pub(crate) fn content_spawn_template() -> NpcSpawn {
    NpcSpawn {
        chat_bubble_foreground_color: 408_679,
        chat_bubble_background_color: 13_951_728,
        chat_bubble_size: 1,
        scale: 1.0,
        animation_id: 1,
        disposition: 1,
        unknown_flag_16: 1,
        interact_range: 100,
        walk_animation_id: -1,
        run_animation_id: -1,
        stand_animation_id: -1,
        sub_text_name_string_id: -1,
        area_definition_id: -1,
        is_interactable: 1,
        movement_type: 2,
        active_profile: 8,
        unknown_field_68: 3442,
        ..NpcSpawn::default()
    }
}

/// AddNpc `rotation` is a facing direction `(sin h, 0, cos h, 0)`, not a
/// quaternion: every captured startup AddNpc has this shape (y = w = 0), and
/// the native run 034234 showed spawns tipped over with a Z-axis quaternion.
/// The heading convention matches Legacy's spawn data (R until a capture
/// pairs a heading with a non-staged rotation).
fn facing(heading: f32) -> [f32; 4] {
    [heading.sin(), 0.0, heading.cos(), 0.0]
}

fn distance_squared(left: [f32; 3], right: [f32; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| f64::from(*a - b).powi(2))
        .sum()
}

fn read_rows<T: DeserializeOwned>(path: &Path) -> io::Result<Rows<T>> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > MAX_NPC_DATA_BYTES {
        return Err(invalid("NPC content file exceeds 16 MiB"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_NPC_DATA_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_NPC_DATA_BYTES {
        return Err(invalid("NPC content file exceeds 16 MiB"));
    }
    serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("invalid NPC content JSON: {error}")))
}

pub(crate) fn unframe_tunnel(bytes: &[u8]) -> io::Result<Vec<u8>> {
    if bytes.len() < 7 || bytes[..3] != [5, 0, 1] {
        return Err(invalid("NPC builder returned an invalid tunnel"));
    }
    let length = u32::from_le_bytes(bytes[3..7].try_into().unwrap()) as usize;
    if length != bytes.len() - 7 {
        return Err(invalid("NPC builder tunnel length differs"));
    }
    Ok(bytes[7..].to_vec())
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

pub type PlayerId = u64;
pub type InstanceId = i32;

/// Position and quaternion carried into the scene transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldPose {
    pub position: [f32; 4],
    pub rotation: [f32; 4],
}

impl WorldPose {
    /// (R) Neutral identity pose for synthetic callers; not a recovered spawn.
    pub const ORIGIN: Self = Self {
        position: [0.0, 0.0, 0.0, 1.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
    };

    fn is_finite(self) -> bool {
        self.position
            .iter()
            .chain(self.rotation.iter())
            .all(|value| value.is_finite())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstanceKind {
    Public,
    Dungeon { dungeon_id: String },
    Encounter { encounter_id: i32 },
    House { owner_id: PlayerId },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerLocation {
    pub instance_id: InstanceId,
    pub pose: WorldPose,
    pub ready: bool,
}

/// Ordered, already framed application packets for one scene entry.
#[derive(Clone, Debug, PartialEq)]
pub struct EnterSequence {
    pub zone_id: u32,
    pub instance_id: InstanceId,
    pub packets: Vec<Vec<u8>>,
    /// The caller must wait for the client's normal ready/load event before
    /// activating zone-specific gameplay. Encounter packet ordering remains
    /// owned by `EncounterEngine`.
    pub awaits_client_ready: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldError {
    UnknownZone(u32),
    UnknownInstance(InstanceId),
    UnknownDungeon(String),
    DungeonHasNoZone(String),
    UnknownParent(InstanceId),
    PublicMustBeShared,
    PlayerNotInInstance(PlayerId),
    InvalidPose,
    NumericRange(&'static str),
    WireEncoding(String),
    InstanceIdsExhausted,
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownZone(id) => write!(out, "unknown zone {id}"),
            Self::UnknownInstance(id) => write!(out, "unknown instance {id}"),
            Self::UnknownDungeon(id) => write!(out, "unknown dungeon {id}"),
            Self::DungeonHasNoZone(id) => write!(out, "dungeon {id} has no zone mapping"),
            Self::UnknownParent(id) => write!(out, "unknown parent instance {id}"),
            Self::PublicMustBeShared => out.write_str("public instances are keyed by zone"),
            Self::PlayerNotInInstance(id) => write!(out, "player {id} is not in that instance"),
            Self::InvalidPose => out.write_str("pose contains a non-finite component"),
            Self::NumericRange(field) => write!(out, "{field} is outside the wire range"),
            Self::WireEncoding(message) => out.write_str(message),
            Self::InstanceIdsExhausted => out.write_str("instance id space is exhausted"),
        }
    }
}

impl std::error::Error for WorldError {}

#[derive(Clone, Debug)]
pub struct ZoneInstance {
    pub id: InstanceId,
    pub zone_id: u32,
    pub kind: InstanceKind,
    pub parent: Option<InstanceId>,
    pub members: BTreeSet<PlayerId>,
    /// NPC definitions are copied from the zone's content rows at creation.
    pub npc_spawns: Vec<ContentNpcSpawn>,
    pub empty_since_ms: Option<u64>,
    ready_members: BTreeSet<PlayerId>,
}

pub struct World {
    // (R) Single-writer combat state; never part of SharedWorld::clone.
    fields: BTreeMap<InstanceId, crate::encounter::SharedFieldState>,
    // (R) Committed action deltas wait for the 20 Hz writer, bounded by the
    // admitted-session combat budgets and the 64-row field content limit.
    field_packets: BTreeMap<InstanceId, Vec<Vec<u8>>>,
    zones: BTreeMap<u32, Zone>,
    npcs_by_zone: BTreeMap<u32, Vec<ContentNpcSpawn>>,
    dungeons: BTreeMap<String, Dungeon>,
    instances: BTreeMap<InstanceId, ZoneInstance>,
    public_instances: BTreeMap<u32, InstanceId>,
    players: BTreeMap<PlayerId, PlayerLocation>,
    // (R) This process-local counter is not a durable retail allocator.
    next_instance_id: InstanceId,
    // (R) Empty private instances use a local grace interval.
    empty_instance_grace_ms: u64,
    // (R) Kills whose big-heart group heal was already given (W158); bounded.
    group_heals: std::collections::VecDeque<(u64, u64)>,
}

/// (R) Only touched membership data is staged. No packets or world-wide clone.
/// Constructed by World and committed by the single writer after persistence.
pub struct MembershipChange {
    player: PlayerId,
    location: PlayerLocation,
    instance: Option<ZoneInstance>,
    next_instance_id: Option<InstanceId>,
    now_ms: u64,
}

impl MembershipChange {
    pub fn instance_id(&self) -> InstanceId {
        self.location.instance_id
    }
}

impl World {
    pub fn queue_field_packets(&mut self, instance: InstanceId, packets: Vec<Vec<u8>>) {
        self.field_packets
            .entry(instance)
            .or_default()
            .extend(packets);
    }
    /// True the first time a kill's big-heart group heal is claimed (R: one
    /// group heal per kill, however many participants picked up a heart). The
    /// last 256 kills are remembered.
    pub fn claim_group_heal(&mut self, kill: (u64, u64)) -> bool {
        if self.group_heals.contains(&kill) {
            return false;
        }
        if self.group_heals.len() >= 256 {
            self.group_heals.pop_front();
        }
        self.group_heals.push_back(kill);
        true
    }
    pub fn take_field_packets(&mut self, instance: InstanceId) -> Vec<Vec<u8>> {
        self.field_packets.remove(&instance).unwrap_or_default()
    }
    /// (R) Shared actions never accept an instance ID from a client.
    pub fn field_for_player(
        &self,
        player: PlayerId,
        zone: u32,
    ) -> Option<(InstanceId, &crate::encounter::SharedFieldState)> {
        let location = self.player_location(player)?;
        if !location.ready || self.instance(location.instance_id)?.zone_id != zone {
            return None;
        }
        Some((location.instance_id, self.field(location.instance_id)?))
    }
    pub fn field(&self, instance: InstanceId) -> Option<&crate::encounter::SharedFieldState> {
        self.fields.get(&instance)
    }
    pub fn install_field(
        &mut self,
        instance: InstanceId,
        field: crate::encounter::SharedFieldState,
    ) {
        self.fields.insert(instance, field);
    }

    /// (R) Live scene descriptors come from Gateway's validated scene route,
    /// not the regional labels in zones.json. NPC startup remains Gateway-owned.
    pub fn runtime() -> Self {
        let mut world = Self::new(
            ContentDb {
                zones: Vec::new(),
                npcs: Vec::new(),
                pois: Vec::new(),
                items: Vec::new(),
                quests: Vec::new(),
                jobs: Vec::new(),
                dungeons: Vec::new(),
                minigames: Vec::new(),
            },
            60_000,
        );
        world
            .register_runtime_zone(Zone {
                id: 0,
                name: String::new(),
                geometry_id: Some(0),
                zone_type: Some(2),
                tutorial: Some(false),
                sky: Some(String::new()),
                source: Vec::new(),
            })
            .expect("constant public scene is valid");
        let public = world
            .make_instance(0, 0, InstanceKind::Public, None, 0)
            .unwrap();
        world.instances.insert(0, public);
        world.public_instances.insert(0, 0);
        world
    }

    pub fn register_runtime_zone(&mut self, zone: Zone) -> Result<(), WorldError> {
        i32::try_from(zone.id).map_err(|_| WorldError::NumericRange("zone id"))?;
        if !self.zones.contains_key(&zone.id) && self.zones.len() >= 16 {
            return Err(WorldError::NumericRange("runtime scene count")); // (R) bounded host metadata.
        }
        self.zones.insert(zone.id, zone);
        Ok(())
    }

    /// (R) Existing targets are supplied only by server-authorized entry paths.
    /// A scene change to nonzero allocates a solo child until group consent is wired.
    pub fn prepare_membership(
        &self,
        player: PlayerId,
        zone_id: u32,
        target: Option<InstanceId>,
        pose: WorldPose,
        ready: bool,
        now_ms: u64,
    ) -> Result<MembershipChange, WorldError> {
        if !pose.is_finite() {
            return Err(WorldError::InvalidPose);
        }
        if !self.zones.contains_key(&zone_id) {
            return Err(WorldError::UnknownZone(zone_id));
        }
        let mut instance = None;
        let mut next_instance_id = None;
        let id = if let Some(id) = target {
            let row = self
                .instances
                .get(&id)
                .ok_or(WorldError::UnknownInstance(id))?;
            if row.zone_id != zone_id {
                return Err(WorldError::UnknownZone(zone_id));
            }
            id
        } else if zone_id == 0 {
            let id = self.public_instance(zone_id).unwrap_or(0);
            if !self.instances.contains_key(&id) {
                instance =
                    Some(self.make_instance(id, zone_id, InstanceKind::Public, None, now_ms)?);
            }
            id
        } else {
            if !self.instances.contains_key(&0) {
                return Err(WorldError::UnknownParent(0));
            }
            let (id, next) = self.available_instance_id()?;
            instance = Some(self.make_instance(
                id,
                zone_id,
                InstanceKind::Encounter { encounter_id: 0 },
                Some(0),
                now_ms,
            )?);
            next_instance_id = Some(next);
            id
        };
        Ok(MembershipChange {
            player,
            location: PlayerLocation {
                instance_id: id,
                pose,
                ready,
            },
            instance,
            next_instance_id,
            now_ms,
        })
    }

    /// Infallible install after save; no other writer may run between prepare/install.
    pub fn commit_membership(&mut self, change: MembershipChange) {
        if let Some(instance) = change.instance {
            if instance.kind == InstanceKind::Public {
                self.public_instances.insert(instance.zone_id, instance.id);
            }
            self.instances.insert(instance.id, instance);
        }
        if let Some(next) = change.next_instance_id {
            self.next_instance_id = next;
        }
        self.leave(change.player, change.now_ms);
        let row = self
            .instances
            .get_mut(&change.location.instance_id)
            .expect("prepared instance remains owned by the single writer");
        row.members.insert(change.player);
        row.empty_since_ms = None;
        let ready = change.location.ready;
        let target = change.location.instance_id;
        self.players.insert(
            change.player,
            PlayerLocation {
                ready: false,
                ..change.location
            },
        );
        if ready {
            self.mark_zone_ready(change.player, target)
                .expect("prepared membership matches its ready instance");
        }
    }

    /// Build indexes from a `ContentDb` already accepted by `ContentDb::load`.
    pub fn new(content: ContentDb, empty_instance_grace_ms: u64) -> Self {
        let zones: BTreeMap<_, _> = content.zones.into_iter().map(|row| (row.id, row)).collect();
        let mut npcs_by_zone: BTreeMap<u32, Vec<ContentNpcSpawn>> = BTreeMap::new();
        for npc in content.npcs {
            npcs_by_zone.entry(npc.zone_id).or_default().push(npc);
        }
        let dungeons = content
            .dungeons
            .into_iter()
            .map(|row| (row.id.clone(), row))
            .collect();
        let next_instance_id = zones
            .keys()
            .filter_map(|id| i32::try_from(*id).ok())
            .max()
            .and_then(|max| max.checked_add(1))
            .unwrap_or(1)
            .max(1);

        Self {
            fields: BTreeMap::new(),
            field_packets: BTreeMap::new(),
            zones,
            npcs_by_zone,
            dungeons,
            instances: BTreeMap::new(),
            public_instances: BTreeMap::new(),
            players: BTreeMap::new(),
            next_instance_id,
            empty_instance_grace_ms,
            group_heals: std::collections::VecDeque::new(),
        }
    }

    pub fn player_location(&self, player: PlayerId) -> Option<&PlayerLocation> {
        self.players.get(&player)
    }

    pub fn instance(&self, id: InstanceId) -> Option<&ZoneInstance> {
        self.instances.get(&id)
    }

    pub fn public_instance(&self, zone_id: u32) -> Option<InstanceId> {
        self.public_instances.get(&zone_id).copied()
    }

    /// Enter the zone's shared public instance, creating it on first use.
    pub fn enter_public(
        &mut self,
        player: PlayerId,
        zone_id: u32,
        pose: WorldPose,
        now_ms: u64,
    ) -> Result<EnterSequence, WorldError> {
        let created = !self.public_instances.contains_key(&zone_id);
        if created {
            let zone = self
                .zones
                .get(&zone_id)
                .ok_or(WorldError::UnknownZone(zone_id))?;
            let id = i32::try_from(zone.id).map_err(|_| WorldError::NumericRange("zone id"))?;
            // (R) Public instance identity follows the zone id so zone 0 keeps
            // the existing startup identifier; private children use new ids.
            let instance = self.make_instance(id, zone_id, InstanceKind::Public, None, now_ms)?;
            self.instances.insert(id, instance);
            self.public_instances.insert(zone_id, id);
        }
        let id = self.public_instances[&zone_id];
        match self.enter_instance(player, id, pose, now_ms) {
            Ok(sequence) => Ok(sequence),
            Err(error) => {
                if created {
                    self.instances.remove(&id);
                    self.public_instances.remove(&zone_id);
                }
                Err(error)
            }
        }
    }

    /// Create one private child. Its content set comes from the requested zone.
    pub fn create_child_instance(
        &mut self,
        parent: InstanceId,
        zone_id: u32,
        kind: InstanceKind,
        now_ms: u64,
    ) -> Result<InstanceId, WorldError> {
        if !self.instances.contains_key(&parent) {
            return Err(WorldError::UnknownParent(parent));
        }
        if kind == InstanceKind::Public {
            return Err(WorldError::PublicMustBeShared);
        }
        if !self.zones.contains_key(&zone_id) {
            return Err(WorldError::UnknownZone(zone_id));
        }
        let id = self.allocate_instance_id()?;
        let instance = self.make_instance(id, zone_id, kind, Some(parent), now_ms)?;
        self.instances.insert(id, instance);
        Ok(id)
    }

    /// Create and enter a dungeon child using its content zone link.
    pub fn enter_dungeon(
        &mut self,
        player: PlayerId,
        parent: InstanceId,
        dungeon_id: &str,
        pose: WorldPose,
        now_ms: u64,
    ) -> Result<EnterSequence, WorldError> {
        let dungeon = self
            .dungeons
            .get(dungeon_id)
            .ok_or_else(|| WorldError::UnknownDungeon(dungeon_id.to_owned()))?;
        let zone_id = dungeon
            .zone_id
            .ok_or_else(|| WorldError::DungeonHasNoZone(dungeon_id.to_owned()))?;
        let id = self.create_child_instance(
            parent,
            zone_id,
            InstanceKind::Dungeon {
                dungeon_id: dungeon_id.to_owned(),
            },
            now_ms,
        )?;
        match self.enter_instance(player, id, pose, now_ms) {
            Ok(sequence) => Ok(sequence),
            Err(error) => {
                self.instances.remove(&id);
                Err(error)
            }
        }
    }

    /// Prepare every packet before changing membership. A build error leaves
    /// the player's previous instance and pose intact.
    pub fn enter_instance(
        &mut self,
        player: PlayerId,
        target: InstanceId,
        pose: WorldPose,
        now_ms: u64,
    ) -> Result<EnterSequence, WorldError> {
        let instance = self
            .instances
            .get(&target)
            .ok_or(WorldError::UnknownInstance(target))?;
        let zone = self
            .zones
            .get(&instance.zone_id)
            .ok_or(WorldError::UnknownZone(instance.zone_id))?;
        // (R) A first zone-0 entry follows the existing login path (43 details
        // without op31); every other entry is a scene transition.
        let include_begin_zoning = self.players.contains_key(&player) || instance.zone_id != 0;
        let sequence = build_sequence(zone, instance, pose, include_begin_zoning)?;

        if let Some(previous) = self.players.get(&player).cloned() {
            if let Some(old) = self.instances.get_mut(&previous.instance_id) {
                old.members.remove(&player);
                old.ready_members.remove(&player);
                if old.members.is_empty() {
                    old.empty_since_ms = Some(now_ms);
                }
            }
        }
        let instance = self.instances.get_mut(&target).unwrap();
        instance.members.insert(player);
        instance.ready_members.remove(&player);
        instance.empty_since_ms = None;
        self.players.insert(
            player,
            PlayerLocation {
                instance_id: target,
                pose,
                ready: false,
            },
        );
        Ok(sequence)
    }

    /// Mark the client's zone load complete. Returns true when all current
    /// instance members are ready; the encounter engine owns its wire burst.
    pub fn mark_zone_ready(
        &mut self,
        player: PlayerId,
        instance_id: InstanceId,
    ) -> Result<bool, WorldError> {
        let location = self
            .players
            .get(&player)
            .ok_or(WorldError::PlayerNotInInstance(player))?;
        if location.instance_id != instance_id {
            return Err(WorldError::PlayerNotInInstance(player));
        }
        let instance = self
            .instances
            .get_mut(&instance_id)
            .ok_or(WorldError::UnknownInstance(instance_id))?;
        instance.ready_members.insert(player);
        self.players.get_mut(&player).unwrap().ready = true;
        // (R) Require every current member; original multi-session policy is unknown.
        Ok(!instance.members.is_empty() && instance.ready_members == instance.members)
    }

    /// Remove one player. Empty instances start their reconstructed grace timer.
    pub fn leave(&mut self, player: PlayerId, now_ms: u64) -> bool {
        let Some(location) = self.players.remove(&player) else {
            return false;
        };
        if let Some(instance) = self.instances.get_mut(&location.instance_id) {
            instance.members.remove(&player);
            instance.ready_members.remove(&player);
            if instance.members.is_empty() {
                instance.empty_since_ms = Some(now_ms);
            }
        }
        true
    }

    /// Destroy empty private instances after the configured `(R)` grace.
    /// (R) Parents remain until their child instances have also been removed.
    pub fn collect_empty_instances(&mut self, now_ms: u64) -> Vec<InstanceId> {
        let mut removed = Vec::new();
        loop {
            let candidates: Vec<_> = self
                .instances
                .values()
                .filter(|instance| {
                    instance.kind != InstanceKind::Public
                        && instance.members.is_empty()
                        && instance.empty_since_ms.is_some_and(|since| {
                            now_ms.saturating_sub(since) >= self.empty_instance_grace_ms
                        })
                        && !self
                            .instances
                            .values()
                            .any(|child| child.parent == Some(instance.id))
                })
                .map(|instance| instance.id)
                .collect();
            if candidates.is_empty() {
                break;
            }
            for id in candidates {
                self.instances.remove(&id);
                self.fields.remove(&id);
                self.field_packets.remove(&id);
                removed.push(id);
            }
        }
        removed.sort_unstable();
        removed
    }

    fn make_instance(
        &self,
        id: InstanceId,
        zone_id: u32,
        kind: InstanceKind,
        parent: Option<InstanceId>,
        now_ms: u64,
    ) -> Result<ZoneInstance, WorldError> {
        if !self.zones.contains_key(&zone_id) {
            return Err(WorldError::UnknownZone(zone_id));
        }
        Ok(ZoneInstance {
            id,
            zone_id,
            kind,
            parent,
            members: BTreeSet::new(),
            npc_spawns: self.npcs_by_zone.get(&zone_id).cloned().unwrap_or_default(),
            empty_since_ms: Some(now_ms),
            ready_members: BTreeSet::new(),
        })
    }

    fn allocate_instance_id(&mut self) -> Result<InstanceId, WorldError> {
        let (id, next) = self.available_instance_id()?;
        self.next_instance_id = next;
        Ok(id)
    }

    fn available_instance_id(&self) -> Result<(InstanceId, InstanceId), WorldError> {
        // (R) Retained empty children also count toward the M4 instance bound.
        if self
            .instances
            .values()
            .filter(|row| row.kind != InstanceKind::Public)
            .count()
            >= 16
        {
            return Err(WorldError::InstanceIdsExhausted);
        }
        let mut next = self.next_instance_id;
        loop {
            let id = next;
            next = id.checked_add(1).ok_or(WorldError::InstanceIdsExhausted)?;
            if id > 0
                && !self.instances.contains_key(&id)
                && !self.public_instances.values().any(|value| *value == id)
                && !u32::try_from(id)
                    .ok()
                    .is_some_and(|zone_id| self.zones.contains_key(&zone_id))
            {
                return Ok((id, next));
            }
        }
    }
}

fn build_sequence(
    zone: &Zone,
    instance: &ZoneInstance,
    pose: WorldPose,
    include_begin_zoning: bool,
) -> Result<EnterSequence, WorldError> {
    if !pose.is_finite() {
        return Err(WorldError::InvalidPose);
    }
    // (R) Missing content metadata falls back to the current gateway's zone
    // type and neutral geometry/tutorial/sky values until content is complete.
    let geometry_id = zone.geometry_id.unwrap_or(0);
    let name = zone.name.as_str();
    let sky = zone.sky.as_deref().unwrap_or("");
    let zone_type = zone.zone_type.unwrap_or(2);
    let tutorial = zone.tutorial.unwrap_or(false);
    let mut packets = Vec::new();

    if include_begin_zoning {
        let zone_wire_id =
            i32::try_from(instance.zone_id).map_err(|_| WorldError::NumericRange("zone id"))?;
        let geometry_wire =
            i32::try_from(geometry_id).map_err(|_| WorldError::NumericRange("geometry id"))?;
        let transition = ZoneTransition {
            envelope_flag: 1,
            scene: name.to_owned(),
            type_id: zone_type,
            position: pose.position,
            rotation: pose.rotation,
            sky: sky.to_owned(),
            tutorial,
            // (R) The content schema has no value for this preserved byte.
            unknown: 0,
            // The existing gateway stores op31's Id as CharacterState.zone_id;
            // private instance identity remains server-side.
            id: zone_wire_id,
            // (R) The content model has no update radius; zero preserves the
            // existing typed transition's neutral value.
            update_radius: 0.0,
            geometry_id: geometry_wire,
            // (R) Content has no radius-override field.
            override_update_radius: false,
            // (R) Ask the client to hold the scene transition for its load gate.
            wait_for_zone_ready: true,
        };
        packets.push(
            zone_transition::encode(&transition)
                .map_err(|error| WorldError::WireEncoding(error.to_string()))?,
        );
    }

    packets.push(startup_messages::build_zone_details(&ZoneDetails {
        name,
        zone_type,
        tutorial,
        sky,
        zone_id: instance.zone_id,
        geometry_id,
    }));
    for npc in &instance.npc_spawns {
        packets.push(startup_world::build_add_npc(&to_wire_npc(npc)?));
    }

    Ok(EnterSequence {
        zone_id: instance.zone_id,
        instance_id: instance.id,
        packets,
        awaits_client_ready: true,
    })
}

fn to_wire_npc(npc: &ContentNpcSpawn) -> Result<NpcSpawn, WorldError> {
    let mut row = NpcSpawn {
        actor_id: npc.actor_id,
        name_string_id: i32::try_from(npc.name_string_id)
            .map_err(|_| WorldError::NumericRange("NPC name string id"))?,
        model_id: i32::try_from(npc.model_id)
            .map_err(|_| WorldError::NumericRange("NPC model id"))?,
        ..content_spawn_template()
    };
    // (R) Missing captured transform components use neutral defaults.
    let position_w = npc.transform.position_w.unwrap_or(1.0);
    row.position = [
        npc.transform.position[0],
        npc.transform.position[1],
        npc.transform.position[2],
        position_w,
    ];
    row.rotation = match &npc.transform.rotation {
        Rotation::Quaternion(value) => *value,
        Rotation::Heading(value) => {
            // (R) Conventional yaw-to-quaternion conversion; content reader
            // evidence does not define this crosswalk.
            [0.0, 0.0, (value * 0.5).sin(), (value * 0.5).cos()]
        }
    };
    // (R) A content row without scale uses the neutral unit scale.
    row.scale = npc.transform.scale.unwrap_or(1.0);
    if !row
        .position
        .iter()
        .chain(row.rotation.iter())
        .all(|value| value.is_finite())
        || !row.scale.is_finite()
    {
        return Err(WorldError::InvalidPose);
    }
    // (R) The content schema lacks the rest of 35/2's display, animation,
    // interaction, and unknown fields. `Default` keeps those fields zero/empty.
    Ok(row)
}
