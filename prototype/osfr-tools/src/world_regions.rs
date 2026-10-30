//! T753: the shared world zone/region/place table (`data/content/regions.json`) and
//! the position -> region/place lookup later zone slices call.
//!
//! Client facts: the ten region ids and banner movies are the first-party Lua table
//! `NotificationHandler:LoadingZone` (`ScriptsBase.lua:11642-11664`). Region 1 is the
//! selector's fallback (`0x00a44b70` sets region id 1 when no journal geometry id
//! matches) and the only client `RegionOverride` is the Newbiezone area
//! (`0x00a44b10`). The world scene is `FabledRealms.gzne` with the captured geometry
//! id 2 (piece_078 capture).
//!
//! Loaded by [`crate::world::NpcVisibility::load`], the live content path that
//! `Gameplay::load` calls. The table is read-only after load: a missing file means no
//! table (a content set without one), a malformed one refuses the session load. No
//! packet is sent and nothing is persisted by this module.
//!
//! Shape membership is the T749 section 3.4 rule projected to x/z (R): the client
//! area tests (`0x00b0b0b0` sphere, `0x00b0b1c0` box) compare four components, so a
//! player far above or below an area is still inside it here. The 2D projection is
//! the accepted coarse rule for the banner regions; a later slice that needs exact
//! volumes must add y to the shapes.

use crate::content::{SourceTag, check_sources};
use serde::Deserialize;
use std::{collections::BTreeSet, fmt, fs, io, path::Path};

/// The content file this table reads, beside the other runtime tables.
pub const CONTENT_FILE: &str = "regions.json";

/// The client's default region: the only `RegionOverride` is 1 and the selector
/// `0x00a44b70` falls back to 1. It has no banner movie.
const DEFAULT_REGION_ID: u32 = 1;

/// (R) Caps keep one malformed file from making the lookup unbounded. The shipped
/// table is 1 zone, 10 regions, 20 places and 816 shapes.
const MAX_ZONES: usize = 16;
const MAX_PLACES: usize = 256;
const MAX_SHAPES: usize = 4096;
const MAX_NAME_IDS: usize = 256;
const MAX_COORD: f32 = 20_000.0;
const MAX_BANNER_BYTES: usize = 64;
/// T900: sanity bound for `banner_settle_ms`, and its value when a content set omits it.
const MAX_SETTLE_MS: u32 = 60_000;
const DEFAULT_SETTLE_MS: u32 = 1_500;

/// One row of the world zone/region/place table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Content {
    schema_version: u32,
    /// Client locale ids (`Global.Text.<id>`) this recovered table may name. Every
    /// region and place `name_string_id` must be in this set; `scripts/content_check.py`
    /// resolves each one in the shipped client locale.
    client_name_ids: Vec<u32>,
    /// T900 (R): how long a player must stay in a new named area before the area banner
    /// shows. Community value (Sanctuary `Player.cs` `AreaSettleMilliseconds`); the client
    /// has no such delay. Tunable, 0..=60000.
    #[serde(default = "default_settle_ms")]
    banner_settle_ms: u32,
    zones: Vec<Zone>,
    regions: Vec<Region>,
    places: Vec<Place>,
}

/// (R) A scene row. Only zone 0's regions are looked up in this slice; instance zones
/// are accepted so the data file can grow with the dungeon slice.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Zone {
    id: u32,
    /// `.gzne` stem. The world row must be the captured `FabledRealms`; unique,
    /// because the client zones by scene name.
    scene: String,
    kind: ZoneKind,
    /// `PacketClientBeginZoning` geometry id; the captured world value is 2.
    geometry_id: u32,
    source: Vec<SourceTag>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ZoneKind {
    World,
    Instance,
}

/// One client region (ids 1..=10), with its coarse bounds and banner movie.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    /// Client region id. 1 is the default/fallback region; 2..=10 are the banner
    /// regions. Unique and exactly 1..=10 in the table.
    pub id: u32,
    /// Recovered client locale text, for diagnostics only; the client shows
    /// [`Self::name_string_id`].
    pub name: String,
    /// Client locale id (`Global.Text.<id>`) of the region name (15/2 later).
    pub name_string_id: u32,
    /// 47/9 banner movie key for regions 2..=10; `None` for region 1, which has no
    /// banner and must never be sent to `NotificationHandler:LoadingZone`.
    pub banner_swf: Option<String>,
    /// Zone the region belongs to; this slice's regions are world zone 0.
    pub parent_zone: u32,
    /// Banner bounds: for 2..=10 the client music/ambience areas of the region's places and
    /// their child areas, for region 1 the client Newbiezone sphere (section 3.4 steps 2
    /// and 3; `scripts/build_region_areas.py`, T876). x/z only (R).
    shapes: Vec<Shape>,
    /// Sum of the shape areas, computed at load; the tie-break after region id.
    #[serde(skip)]
    area: f64,
    pub source: Vec<SourceTag>,
}

/// One first-party client area group (a "place"), with its client locale name.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Place {
    /// Stable content key, unique across places. Section 3.4's stable id tie-break.
    pub id: String,
    /// Recovered client locale text, for diagnostics only.
    pub name: String,
    /// Client locale id (`Global.Text.<id>`) the 15/2 chat line names later.
    pub name_string_id: u32,
    /// Parent client region id; its row supplies the banner for this place.
    pub region_id: u32,
    /// Section 3.4 priority: a higher number wins an overlap (starting area 100,
    /// banner region groups 50, named places 30, Wilds 10, Sunstone 5); ties then use
    /// the smaller group area, then the smaller [`Self::id`].
    pub priority: u16,
    /// Client area group bounds (top-level, always-on, positive-radius spheres, plus the
    /// place's music/ambience areas with their child areas), projected to x/z (R).
    shapes: Vec<Shape>,
    /// Sum of the shape areas, computed at load; the section 3.4 tie-break.
    #[serde(skip)]
    area: f64,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case", deny_unknown_fields)]
enum Shape {
    Sphere { x: f32, z: f32, r: f32 },
    Rect { x1: f32, z1: f32, x2: f32, z2: f32 },
}

impl Shape {
    /// 2D x/z membership (R; see the module note).
    fn contains(self, x: f32, z: f32) -> bool {
        match self {
            Self::Sphere { x: sx, z: sz, r } => (x - sx).powi(2) + (z - sz).powi(2) <= r * r,
            Self::Rect { x1, z1, x2, z2 } => x1 <= x && x <= x2 && z1 <= z && z <= z2,
        }
    }

    fn area(self) -> f64 {
        match self {
            Self::Sphere { r, .. } => std::f64::consts::PI * f64::from(r).powi(2),
            Self::Rect { x1, z1, x2, z2 } => f64::from(x2 - x1) * f64::from(z2 - z1),
        }
    }

    fn validate(self) -> Result<(), String> {
        let coord = |value: f32| value.is_finite() && value.abs() <= MAX_COORD;
        match self {
            Self::Sphere { x, z, r } => {
                if !coord(x) || !coord(z) || !coord(r) {
                    return Err(format!(
                        "sphere coordinates/radius must be finite and within +/-{MAX_COORD}"
                    ));
                }
                if r <= 0.0 {
                    return Err("sphere radius must be positive".to_owned());
                }
                Ok(())
            }
            Self::Rect { x1, z1, x2, z2 } => {
                if !coord(x1) || !coord(z1) || !coord(x2) || !coord(z2) {
                    return Err(format!(
                        "rect coordinates must be finite and within +/-{MAX_COORD}"
                    ));
                }
                if x1 >= x2 || z1 >= z2 {
                    return Err("rect corners must be ordered (x1 < x2, z1 < z2)".to_owned());
                }
                Ok(())
            }
        }
    }
}

fn default_settle_ms() -> u32 {
    DEFAULT_SETTLE_MS
}

fn named(path: &Path, error: impl fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {error}", path.display()),
    )
}

impl Content {
    /// Refuses every malformed row and returns the world zone id for lookups.
    fn finish(&mut self) -> Result<u32, String> {
        if self.schema_version != 1 {
            return Err("schema_version must be 1".to_owned());
        }
        if self.banner_settle_ms > MAX_SETTLE_MS {
            return Err(format!("banner_settle_ms must be 0..={MAX_SETTLE_MS}"));
        }
        if self.zones.is_empty() || self.zones.len() > MAX_ZONES {
            return Err(format!("zones must contain 1..={MAX_ZONES} rows"));
        }
        if self.client_name_ids.len() > MAX_NAME_IDS {
            return Err(format!(
                "client_name_ids must contain at most {MAX_NAME_IDS} ids"
            ));
        }
        let mut name_ids = BTreeSet::new();
        for id in &self.client_name_ids {
            if *id == 0 || !name_ids.insert(*id) {
                return Err("client_name_ids must be unique positive locale ids".to_owned());
            }
        }
        let mut zone_ids = BTreeSet::new();
        let mut scenes = BTreeSet::new();
        for zone in &self.zones {
            if !zone_ids.insert(zone.id) {
                return Err(format!("duplicate zone id {}", zone.id));
            }
            let scene = zone.scene.trim();
            if scene.is_empty() || zone.scene.chars().any(char::is_control) {
                return Err(format!("zone {} needs a .gzne scene name", zone.id));
            }
            if !scenes.insert(scene) {
                return Err(format!("duplicate zone scene {scene:?}"));
            }
            check_sources(&format!("zone {}", zone.id), &zone.source)
                .map_err(|error| error.to_string())?;
            match zone.kind {
                ZoneKind::World => {
                    if zone.id != crate::world::OPEN_WORLD_ZONE {
                        return Err(format!(
                            "world zone id must be {}",
                            crate::world::OPEN_WORLD_ZONE
                        ));
                    }
                    if scene != "FabledRealms" {
                        return Err(
                            "world scene must be the captured FabledRealms.gzne stem".to_owned()
                        );
                    }
                    if zone.geometry_id != 2 {
                        return Err(
                            "world geometry_id must be the captured 2 (the 214 placeholder is refused)"
                                .to_owned(),
                        );
                    }
                }
                ZoneKind::Instance => {
                    if zone.geometry_id == 0 || (1..=10).contains(&zone.id) {
                        return Err(format!(
                            "instance zone {} needs a geometry id and an id that is not a client region id",
                            zone.id
                        ));
                    }
                }
            }
        }
        let world = self
            .zones
            .iter()
            .filter(|zone| zone.kind == ZoneKind::World)
            .collect::<Vec<_>>();
        if world.len() != 1 {
            return Err("exactly one world zone row is required".to_owned());
        }
        let world_zone = world[0].id;

        if self.regions.len() != 10 {
            return Err("exactly ten client regions (ids 1..=10) are required".to_owned());
        }
        let mut region_ids = BTreeSet::new();
        for region in &mut self.regions {
            if !(1..=10).contains(&region.id) {
                return Err(format!(
                    "region {} is outside the client region ids 1..=10",
                    region.id
                ));
            }
            if !region_ids.insert(region.id) {
                return Err(format!("duplicate region id {}", region.id));
            }
            if region.name.trim().is_empty() {
                return Err(format!("region {} needs a name", region.id));
            }
            if !name_ids.contains(&region.name_string_id) {
                return Err(format!(
                    "region {} name id {} is not in client_name_ids",
                    region.id, region.name_string_id
                ));
            }
            if region.parent_zone != world_zone {
                return Err(format!(
                    "region {} parent_zone {} is not the world zone {world_zone}",
                    region.id, region.parent_zone
                ));
            }
            if region.id == DEFAULT_REGION_ID {
                if region.banner_swf.is_some() {
                    return Err(
                        "the default region 1 has no banner movie and must not name one".to_owned(),
                    );
                }
            } else {
                let Some(banner) = region.banner_swf.as_deref() else {
                    return Err(format!("region {} needs a .swf banner movie", region.id));
                };
                if !banner.ends_with(".swf")
                    || banner.len() > MAX_BANNER_BYTES
                    || banner.chars().any(char::is_control)
                {
                    return Err(format!("region {} banner movie is invalid", region.id));
                }
            }
            check_sources(&format!("region {}", region.id), &region.source)
                .map_err(|error| error.to_string())?;
            region.area = shapes_area(&format!("region {}", region.id), &region.shapes)?;
        }

        if self.places.len() > MAX_PLACES {
            return Err(format!("places must contain at most {MAX_PLACES} rows"));
        }
        let mut place_ids = BTreeSet::new();
        let mut shape_count = self
            .regions
            .iter()
            .map(|region| region.shapes.len())
            .sum::<usize>();
        for place in &mut self.places {
            if place.id.trim().is_empty() || place.id.chars().any(char::is_control) {
                return Err("place ids must be non-empty and free of control characters".to_owned());
            }
            if !place_ids.insert(place.id.clone()) {
                return Err(format!("duplicate place id {:?}", place.id));
            }
            if place.name.trim().is_empty() {
                return Err(format!("place {:?} needs a name", place.id));
            }
            if !name_ids.contains(&place.name_string_id) {
                return Err(format!(
                    "place {:?} name id {} is not in client_name_ids",
                    place.id, place.name_string_id
                ));
            }
            if !region_ids.contains(&place.region_id) {
                return Err(format!(
                    "place {:?} references unknown region {}",
                    place.id, place.region_id
                ));
            }
            if place.priority > 1000 {
                return Err(format!("place {:?} priority is outside 0..=1000", place.id));
            }
            check_sources(&format!("place {:?}", place.id), &place.source)
                .map_err(|error| error.to_string())?;
            place.area = shapes_area(&format!("place {:?}", place.id), &place.shapes)?;
            shape_count += place.shapes.len();
        }
        if shape_count > MAX_SHAPES {
            return Err(format!("at most {MAX_SHAPES} shapes may be loaded"));
        }

        self.regions.sort_by(|left, right| {
            left.area
                .total_cmp(&right.area)
                .then(left.id.cmp(&right.id))
        });
        // Section 3.4: highest priority, then the smaller group, then the stable id.
        self.places.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then(left.area.total_cmp(&right.area))
                .then(left.id.cmp(&right.id))
        });
        Ok(world_zone)
    }
}

fn shapes_area(owner: &str, shapes: &[Shape]) -> Result<f64, String> {
    if shapes.is_empty() {
        return Err(format!("{owner} needs at least one shape"));
    }
    let mut area = 0.0;
    for shape in shapes {
        shape.validate()?;
        area += shape.area();
    }
    Ok(area)
}

/// The live world region table and its position -> region/place lookup.
#[derive(Debug)]
pub struct RegionTable {
    content: Content,
    /// The one world zone id; lookups for any other-zone id return `None`.
    world_zone: u32,
}

impl RegionTable {
    /// Reads `regions.json` from `directory`. `Ok(None)` when the content set has no
    /// file; a present but malformed file is refused and names the file.
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
        let path = directory.join(CONTENT_FILE);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(named(&path, error)),
        };
        let mut content: Content =
            serde_json::from_slice(&bytes).map_err(|error| named(&path, error))?;
        let world_zone = content.finish().map_err(|error| named(&path, error))?;
        Ok(Some(Self {
            content,
            world_zone,
        }))
    }

    /// The client region for a world position, or `None` outside every region (the
    /// caller must not invent one). `zone` must be the table's world zone.
    pub fn region_at(&self, zone: u32, x: f32, z: f32) -> Option<u32> {
        self.classify(zone, x, z).map(|(region, _)| region)
    }

    /// The first-party place group for a world position, or `None` when only a
    /// region (or nothing) covers it.
    pub fn place_at(&self, zone: u32, x: f32, z: f32) -> Option<&Place> {
        self.classify(zone, x, z).and_then(|(_, place)| place)
    }

    /// T900: the named area a world position is in, as `(region id, name string id)`: the
    /// first-party place's locale id when a place group covers it, else the region's. The
    /// region id is the 104 POI id (2..=10 shows the banner, 1 does not); the name id is the
    /// 104 name and the 15/2 area. `None` outside every region.
    pub fn named_area_at(&self, zone: u32, x: f32, z: f32) -> Option<(u32, u32)> {
        let (region, place) = self.classify(zone, x, z)?;
        let name = match place {
            Some(place) => place.name_string_id,
            None => self.region(region)?.name_string_id,
        };
        Some((region, name))
    }

    /// T900: the area-banner settle time from `regions.json` (`banner_settle_ms`).
    pub fn banner_settle_ms(&self) -> u32 {
        self.content.banner_settle_ms
    }

    fn classify(&self, zone: u32, x: f32, z: f32) -> Option<(u32, Option<&Place>)> {
        if zone != self.world_zone || !x.is_finite() || !z.is_finite() {
            return None;
        }
        // Section 3.4 step 1: the highest-priority first-party place group wins.
        if let Some(place) = self
            .content
            .places
            .iter()
            .find(|place| place.contains(x, z))
        {
            return Some((place.region_id, Some(place)));
        }
        // Step 2: the banner-region shapes (ids 2..=10), smallest first.
        for region in &self.content.regions {
            if region.id != DEFAULT_REGION_ID && region.contains(x, z) {
                return Some((region.id, None));
            }
        }
        // Step 3: inside the client Newbiezone sphere, region 1 wins (no banner).
        let fallback = self
            .content
            .regions
            .iter()
            .find(|region| region.id == DEFAULT_REGION_ID)?;
        fallback.contains(x, z).then_some((DEFAULT_REGION_ID, None))
    }

    /// One region row by client id.
    pub fn region(&self, id: u32) -> Option<&Region> {
        self.content.regions.iter().find(|region| region.id == id)
    }

    /// Region rows in lookup order (area, then id).
    pub fn regions(&self) -> &[Region] {
        &self.content.regions
    }

    /// Place rows in section 3.4 lookup order (priority, area, id).
    pub fn places(&self) -> &[Place] {
        &self.content.places
    }
}

impl Region {
    /// Whether `(x, z)` is inside this region's coarse shapes (2D, R). The table's
    /// classifier uses this; tests count the NPC rows each region covers with it.
    pub fn contains(&self, x: f32, z: f32) -> bool {
        self.shapes.iter().any(|shape| shape.contains(x, z))
    }
}

impl Place {
    /// Whether `(x, z)` is inside this place group's client area shapes (2D, R).
    pub fn contains(&self, x: f32, z: f32) -> bool {
        self.shapes.iter().any(|shape| shape.contains(x, z))
    }
}
