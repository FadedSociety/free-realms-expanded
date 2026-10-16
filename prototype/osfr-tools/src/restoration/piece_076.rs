//! Work item 076: regional NPC and spawn content.
//!
//! Recovered original/community facts (sources listed in
//! `notes/Research/Pieces/076 Regional NPC and spawn content.md`):
//!
//! * `REGION_AREAS` is the recovered `AreaDefinitions` table of world zone 1
//!   (`FabledRealms`): nine named runtime areas, four of them rectangles and five
//!   circles, with the original coordinates. Membership below is an inference from
//!   those shapes, not a separately recovered per-placement zone field.
//! * `NPC_GUID_BASE` is the recovered placement guid convention: 3421 of the 3426
//!   placements use `100000000000 + npc id`, four carry the npc row's recovered
//!   `CaptureGuid`, and one (npc 33081) carries a guid that matches neither.
//! * Four npc rows are interaction actors bound to an existing terrain object
//!   (`ModelId` 0 plus a `TerrainObjectId`); they need no model asset.
//! * Of the 793 distinct model files referenced by the zone script, 737 resolve in
//!   the extracted client catalogue and 56 do not; those stay explicit gaps.
//!
//! Reconstructed policy: identifier and asset validation with two severities. A
//! contradiction (duplicate guid, unknown npc row, guid aliasing a different npc
//! id, or a row with neither a model nor a terrain binding) makes a catalogue
//! unusable. A finding (a guid outside the convention that aliases no other npc
//! row) is reported but does not invalidate the placement. Nothing here spawns an
//! actor, writes content or claims native acceptance.

use std::collections::BTreeMap;

/// Recovered placement guid convention: `100000000000 + npc id`.
pub const NPC_GUID_BASE: u64 = 100_000_000_000;

/// Recovered world zone id.
pub const ZONE_ID: u32 = 1;

/// Recovered world zone name.
pub const ZONE_NAME: &str = "FabledRealms";

/// Recovered world zone tile size.
pub const ZONE_TILE_SIZE: i32 = 64;

/// Recovered number of named runtime areas inside the world zone.
pub const AREA_COUNT: usize = 9;

/// Recovered `spawnNpcWithGuid` call count in the pinned zone script.
pub const RECOVERED_PLACEMENTS: usize = 3426;

/// Recovered number of distinct npc ids across those placements.
pub const RECOVERED_DISTINCT_NPC_IDS: usize = 3426;

/// Recovered placements using `NPC_GUID_BASE + npc id`.
pub const RECOVERED_CONVENTIONAL_GUIDS: usize = 3421;

/// Recovered placements using the npc row's `CaptureGuid`.
pub const RECOVERED_CAPTURE_GUIDS: usize = 4;

/// Recovered npc ids whose guid matches neither convention.
pub const RECOVERED_UNEXPECTED_GUIDS: &[i32] = &[33081];

/// Recovered interaction npcs bound to an existing terrain object.
pub const RECOVERED_TERRAIN_BOUND_INTERACTION_NPCS: &[i32] = &[36143, 36307, 36308, 36309];

/// Recovered distinct model files referenced by the placements.
pub const RECOVERED_MODEL_ASSETS: usize = 793;

/// Recovered model files that do not resolve in the extracted client catalogue.
pub const RECOVERED_UNRESOLVED_MODEL_ASSETS: usize = 56;

/// Recovered placements in the second pinned revision of the same zone script.
pub const LEGACY_PLACEMENTS: usize = 4253;

/// Recovered npc ids present in both pinned revisions.
pub const CROSS_REVISION_SHARED_IDS: usize = 3419;

/// Recovered model conflicts between the two pinned revisions.
pub const CROSS_REVISION_MODEL_CONFLICTS: usize = 0;

/// Recovered model file names used by more than one recovered row.
pub struct Assets {
    pub dwarf_therin: &'static str,
    pub painting_chaos_laughter: &'static str,
    pub warpstone: &'static str,
}

pub const ASSET: Assets = Assets {
    dwarf_therin: "dwarf_m_therin.adr",
    painting_chaos_laughter: "hsg_painting_tcg_chaos_creepylaughter_01.adr",
    warpstone: "sg_warpstone_01.adr",
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AreaShape {
    Circle { x: f32, z: f32, radius: f32 },
    Rectangle { x1: f32, z1: f32, x2: f32, z2: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegionArea {
    pub id: i32,
    pub name: &'static str,
    pub shape: AreaShape,
}

/// Recovered `AreaDefinitions` of world zone 1.
pub const REGION_AREAS: &[RegionArea] = &[
    RegionArea {
        id: 2,
        name: "Blackspore",
        shape: AreaShape::Rectangle {
            x1: -1680.0,
            z1: 1700.0,
            x2: -2820.0,
            z2: 820.0,
        },
    },
    RegionArea {
        id: 3,
        name: "Briarwood",
        shape: AreaShape::Circle {
            x: -540.0,
            z: 2240.0,
            radius: 700.0,
        },
    },
    RegionArea {
        id: 4,
        name: "Merryvale",
        shape: AreaShape::Circle {
            x: -1040.0,
            z: -404.0,
            radius: 275.0,
        },
    },
    RegionArea {
        id: 5,
        name: "Sanctuary",
        shape: AreaShape::Rectangle {
            x1: -860.0,
            z1: 490.0,
            x2: -1540.0,
            z2: 130.0,
        },
    },
    RegionArea {
        id: 6,
        name: "Seaside",
        shape: AreaShape::Rectangle {
            x1: -1162.0,
            z1: -1343.0,
            x2: 37.0,
            z2: -993.0,
        },
    },
    RegionArea {
        id: 7,
        name: "Shrouded Glades",
        shape: AreaShape::Circle {
            x: 96.0,
            z: 1190.0,
            radius: 250.0,
        },
    },
    RegionArea {
        id: 8,
        name: "Snowhill",
        shape: AreaShape::Circle {
            x: 380.0,
            z: 400.0,
            radius: 350.0,
        },
    },
    RegionArea {
        id: 9,
        name: "Wugachug",
        shape: AreaShape::Circle {
            x: 66.0,
            z: -396.0,
            radius: 165.0,
        },
    },
    RegionArea {
        id: 10,
        name: "Sunstone Valley",
        shape: AreaShape::Circle {
            x: 711.0,
            z: 2143.0,
            radius: 765.0,
        },
    },
];

/// One recovered `zone:spawnNpcWithGuid(npcId, guid, x, y, z, heading)` placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub npc_id: i32,
    pub guid: u64,
    pub position: [f32; 3],
    pub heading: f32,
}

/// One recovered `Npcs.json` row, reduced to the fields this piece validates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NpcRow {
    pub id: i32,
    pub model_id: i32,
    pub model_file: Option<&'static str>,
    pub terrain_object_id: Option<i32>,
    pub capture_guid: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuidStatus {
    Conventional,
    CaptureGuid,
    Unexpected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetBinding {
    Model(&'static str),
    TerrainObject(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Breaks the catalogue.
    Contradiction,
    /// Recovered deviation worth a review; the placement stays usable.
    Finding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementIssue {
    DuplicateGuid(u64),
    GuidCollision { npc_id: i32, aliases: i32 },
    UnexpectedGuid(i32),
    UnknownNpcRow(i32),
    MissingModelBinding(i32),
}

impl PlacementIssue {
    pub fn severity(&self) -> Severity {
        match self {
            PlacementIssue::UnexpectedGuid(_) => Severity::Finding,
            _ => Severity::Contradiction,
        }
    }
}

pub fn guid_for_npc(npc_id: i32) -> u64 {
    NPC_GUID_BASE + npc_id as u64
}

/// Recovers the guid kind of one placement against its recovered npc row.
pub fn classify_guid(npc_id: i32, guid: u64, capture_guid: Option<u64>) -> GuidStatus {
    match capture_guid {
        Some(capture) if capture == guid => GuidStatus::CaptureGuid,
        Some(_) => GuidStatus::Unexpected,
        None if guid == guid_for_npc(npc_id) => GuidStatus::Conventional,
        None => GuidStatus::Unexpected,
    }
}

pub fn area_contains(area: &RegionArea, x: f32, z: f32) -> bool {
    match area.shape {
        AreaShape::Circle {
            x: cx,
            z: cz,
            radius,
        } => {
            let (dx, dz) = (x - cx, z - cz);
            dx * dx + dz * dz <= radius * radius
        }
        AreaShape::Rectangle { x1, z1, x2, z2 } => {
            let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
            let (min_z, max_z) = if z1 <= z2 { (z1, z2) } else { (z2, z1) };
            x >= min_x && x <= max_x && z >= min_z && z <= max_z
        }
    }
}

/// Inferred membership: which recovered runtime areas contain this position.
pub fn regions_at(position: [f32; 3]) -> Vec<&'static RegionArea> {
    REGION_AREAS
        .iter()
        .filter(|area| area_contains(area, position[0], position[2]))
        .collect()
}

/// Asset binding of one recovered npc row: a model file, a terrain object, or nothing.
pub fn asset_binding(row: &NpcRow) -> Option<AssetBinding> {
    row.model_file
        .map(AssetBinding::Model)
        .or_else(|| row.terrain_object_id.map(AssetBinding::TerrainObject))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Audit {
    pub conventional: usize,
    pub capture: usize,
    pub unexpected: Vec<i32>,
    pub terrain_bound: Vec<i32>,
    pub unknown_ids: Vec<i32>,
    pub issues: Vec<PlacementIssue>,
    pub unresolved_models: Vec<&'static str>,
    pub region_counts: BTreeMap<&'static str, usize>,
    pub outside_declared_areas: usize,
}

impl Audit {
    pub fn contradictions(&self) -> Vec<&PlacementIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity() == Severity::Contradiction)
            .collect()
    }

    pub fn findings(&self) -> Vec<&PlacementIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity() == Severity::Finding)
            .collect()
    }
}

/// Validates recovered placements against the recovered npc table and the set of
/// model file names that actually resolve in the preserved client catalogue.
pub fn audit(places: &[Place], rows: &[NpcRow], resolved_assets: &[&str]) -> Audit {
    let mut result = Audit::default();
    let mut guid_counts: BTreeMap<u64, usize> = BTreeMap::new();

    for place in places {
        *guid_counts.entry(place.guid).or_insert(0) += 1;
        let Some(row) = rows.iter().find(|row| row.id == place.npc_id) else {
            result.unknown_ids.push(place.npc_id);
            result
                .issues
                .push(PlacementIssue::UnknownNpcRow(place.npc_id));
            continue;
        };

        match classify_guid(place.npc_id, place.guid, row.capture_guid) {
            GuidStatus::Conventional => result.conventional += 1,
            GuidStatus::CaptureGuid => result.capture += 1,
            GuidStatus::Unexpected => {
                result.unexpected.push(place.npc_id);
                let aliased = place.guid.checked_sub(NPC_GUID_BASE).filter(|id| *id > 0);
                match aliased {
                    Some(alias)
                        if i32::try_from(alias).ok() != Some(place.npc_id)
                            && rows.iter().any(|row| row.id as u64 == alias) =>
                    {
                        result.issues.push(PlacementIssue::GuidCollision {
                            npc_id: place.npc_id,
                            aliases: alias as i32,
                        });
                    }
                    _ => result
                        .issues
                        .push(PlacementIssue::UnexpectedGuid(place.npc_id)),
                }
            }
        }

        match asset_binding(row) {
            Some(AssetBinding::Model(name)) => {
                if !resolved_assets.iter().any(|resolved| *resolved == name) {
                    result.unresolved_models.push(name);
                }
            }
            Some(AssetBinding::TerrainObject(_)) => result.terrain_bound.push(place.npc_id),
            None => result
                .issues
                .push(PlacementIssue::MissingModelBinding(place.npc_id)),
        }

        match regions_at(place.position).as_slice() {
            [] => result.outside_declared_areas += 1,
            areas => {
                for area in areas {
                    *result.region_counts.entry(area.name).or_insert(0) += 1;
                }
            }
        }
    }

    for (guid, count) in guid_counts {
        if count > 1 {
            result.issues.push(PlacementIssue::DuplicateGuid(guid));
        }
    }
    result.unexpected.sort_unstable();
    result.unexpected.dedup();
    result.terrain_bound.sort_unstable();
    result.terrain_bound.dedup();
    result.unknown_ids.sort_unstable();
    result.unknown_ids.dedup();
    result.unresolved_models.sort_unstable();
    result.unresolved_models.dedup();
    result
}

/// Cross-zone/revision identifier check: one model per shared npc id.
pub fn model_conflicts<'a>(
    a: &'a [NpcRow],
    b: &'a [NpcRow],
) -> Vec<(i32, &'static str, &'static str)> {
    let mut conflicts = Vec::new();
    for left in a {
        let Some(right) = b.iter().find(|row| row.id == left.id) else {
            continue;
        };
        if left.model_id != right.model_id || left.model_file != right.model_file {
            conflicts.push((
                left.id,
                left.model_file.unwrap_or(""),
                right.model_file.unwrap_or(""),
            ));
        }
    }
    conflicts
}

/// Recovered sample placements used by the acceptance test: one conventional
/// placement, the four capture-guid interaction actors, the non-conventional
/// painting placement and the terrain-bound warpstones.
pub const RECOVERED_SAMPLE_PLACEMENTS: &[Place] = &[
    Place {
        npc_id: 1201,
        guid: 100000001201,
        position: [-955.8561, -44.62718, -1170.239],
        heading: 2.3732,
    },
    Place {
        npc_id: 33167,
        guid: 14293651161170,
        position: [-1418.3218, -27.002932, 347.82056],
        heading: -0.0622950028,
    },
    Place {
        npc_id: 33558,
        guid: 23983097380882,
        position: [-721.7169, 9.560479, 948.71606],
        heading: -1.81613004,
    },
    Place {
        npc_id: 33602,
        guid: 7215545057314,
        position: [50.92852, 32.425617, 370.30267],
        heading: 1.21874107,
    },
    Place {
        npc_id: 36143,
        guid: 13125420056642,
        position: [902.139038, 2.06609654, 2235.62817],
        heading: -0.820305467,
    },
    Place {
        npc_id: 36307,
        guid: 100000036307,
        position: [-1270.01453, -49.5988922, 785.111572],
        heading: -0.383972555,
    },
    Place {
        npc_id: 36308,
        guid: 100000036308,
        position: [-2128.67554, -32.8866158, 1044.89343],
        heading: -1.83259571,
    },
    Place {
        npc_id: 36309,
        guid: 100000036309,
        position: [501.101013, 69.9525299, 1779.1543],
        heading: -0.820305467,
    },
    Place {
        npc_id: 33081,
        guid: 100000033190,
        position: [-1933.191, -43.63443, -133.8725],
        heading: 1.370251,
    },
];

/// Recovered `Npcs.json` rows for the sample placements.
pub const RECOVERED_SAMPLE_ROWS: &[NpcRow] = &[
    NpcRow {
        id: 1201,
        model_id: 46,
        model_file: Some("dwarf_m_therin.adr"),
        terrain_object_id: None,
        capture_guid: None,
    },
    NpcRow {
        id: 33081,
        model_id: 942,
        model_file: Some("hsg_painting_tcg_chaos_creepylaughter_01.adr"),
        terrain_object_id: None,
        capture_guid: None,
    },
    NpcRow {
        id: 33167,
        model_id: 280,
        model_file: Some("sg_warpstone_01.adr"),
        terrain_object_id: None,
        capture_guid: Some(14293651161170),
    },
    NpcRow {
        id: 33558,
        model_id: 280,
        model_file: Some("sg_warpstone_01.adr"),
        terrain_object_id: None,
        capture_guid: Some(23983097380882),
    },
    NpcRow {
        id: 33602,
        model_id: 280,
        model_file: Some("sg_warpstone_01.adr"),
        terrain_object_id: None,
        capture_guid: Some(7215545057314),
    },
    NpcRow {
        id: 36143,
        model_id: 0,
        model_file: None,
        terrain_object_id: Some(1346725607),
        capture_guid: Some(13125420056642),
    },
    NpcRow {
        id: 36307,
        model_id: 0,
        model_file: None,
        terrain_object_id: Some(1254775835),
        capture_guid: None,
    },
    NpcRow {
        id: 36308,
        model_id: 0,
        model_file: None,
        terrain_object_id: Some(1254779212),
        capture_guid: None,
    },
    NpcRow {
        id: 36309,
        model_id: 0,
        model_file: None,
        terrain_object_id: Some(1345844843),
        capture_guid: None,
    },
];
