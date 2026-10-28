//! Build a zone's ground-height grid and, with `--obstacles`, its placed-object
//! collision mesh from the client's terrain chunks.
//!
//! osfr-terrain --catalog lab/manifests/rust-asset-catalog.tsv
//!   --assets lab/extracted/client-assets --chunks FabledRealms --zone 0
//!   --out lab/terrain [--cell 2] [--obstacles]
//!   [--check data/content/npc_spawns_extra.json]
//!
//! `--obstacles` reads every placed `.adr` runtime object, follows its section-13
//! `.cdt` companion, and transforms the `CDTA` prefix triangles into world space
//! (T534). The output is derived from client data: keep it under `lab/`, out of Git.
#[path = "../restoration/piece_064.rs"]
#[allow(dead_code)]
mod piece_064;
#[path = "../restoration/piece_098.rs"]
#[allow(dead_code)]
mod piece_098;

use osfr_tools::{
    character_store::replace_bytes,
    obstacles::{CELL_M, ZoneObstacles, placed},
    terrain::HeightGrid,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

/// T534: how far outside the rasterised terrain grid a placed mesh may reach before the
/// placement is treated as corrupt/foreign and skipped (m). Terrain objects sit on the tile,
/// so this only rejects a bad asset, never a real bridge or building.
const ZONE_OBJECT_MARGIN_M: f32 = 512.0;

fn main() {
    if let Err(error) = run() {
        eprintln!("osfr-terrain: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut catalog, mut assets, mut chunks, mut zone, mut out, mut cell, mut check) =
        (None, None, None, None, None, 2.0f32, None);
    let mut obstacles = false;
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--catalog" => catalog = Some(PathBuf::from(value()?)),
            "--assets" => assets = Some(PathBuf::from(value()?)),
            "--chunks" => chunks = Some(value()?),
            "--zone" => zone = Some(value()?.parse::<u32>().map_err(|e| e.to_string())?),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--cell" => cell = value()?.parse().map_err(|e| format!("--cell: {e}"))?,
            "--obstacles" => obstacles = true,
            "--check" => check = Some(PathBuf::from(value()?)),
            _ => {
                return Err(format!(
                    "unknown option {flag} (see the file header for usage)"
                ));
            }
        }
    }
    let (catalog, assets, chunks, zone, out) = (
        catalog.ok_or("--catalog is required")?,
        assets.ok_or("--assets is required")?,
        chunks.ok_or("--chunks (file name prefix, e.g. FabledRealms) is required")?,
        zone.ok_or("--zone is required")?,
        out.ok_or("--out is required")?,
    );
    let listing =
        fs::read_to_string(&catalog).map_err(|e| format!("{}: {e}", catalog.display()))?;
    let mut assets_by_name: HashMap<String, String> = HashMap::new();
    for line in listing.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() >= 7 {
            assets_by_name
                .entry(fields[2].to_ascii_lowercase())
                .or_insert_with(|| fields[6].to_string());
        }
    }
    let mut inputs: BTreeMap<String, (u64, String)> = BTreeMap::new();
    if obstacles {
        inputs.insert(
            catalog.file_name().map_or_else(
                || "rust-asset-catalog.tsv".into(),
                |name| name.to_string_lossy().into_owned(),
            ),
            (listing.len() as u64, sha256_hex(listing.as_bytes())),
        );
    }
    let prefix = format!("{chunks}_");
    let mut triangles = Vec::new();
    let mut objects = Vec::new();
    let (mut loaded, mut skipped) = (0, 0);
    for line in listing.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 7 || !fields[2].starts_with(&prefix) || !fields[2].ends_with(".gcnk") {
            continue;
        }
        let bytes = fs::read(assets.join(fields[6])).map_err(|e| format!("{}: {e}", fields[6]))?;
        if obstacles {
            inputs.insert(
                fields[6].to_string(),
                (bytes.len() as u64, sha256_hex(&bytes)),
            );
        }
        // Placeholder tiles (`hello` stubs) and readers' known gaps are skipped, not fatal.
        let Ok(tile) = piece_098::load_terrain(&bytes) else {
            skipped += 1;
            continue;
        };
        loaded += 1;
        for batch in &tile.render_batches {
            let start = batch.index_offset as usize;
            let indices = &tile.indices[start..start + batch.index_count as usize];
            for tri in indices.chunks_exact(3) {
                let vertex =
                    |i: u16| tile.vertices[i as usize + batch.vertex_offset as usize].position;
                triangles.push([vertex(tri[0]), vertex(tri[1]), vertex(tri[2])]);
            }
        }
        if obstacles {
            for record in &tile.tiles {
                objects.extend(record.objects.iter().cloned());
            }
        }
    }
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    if triangles.is_empty() {
        return Err(format!("no terrain triangles for {chunks}"));
    }
    let (mut min, mut max) = ([f32::MAX; 2], [f32::MIN; 2]);
    for v in triangles.iter().flatten() {
        min = [min[0].min(v[0]), min[1].min(v[2])];
        max = [max[0].max(v[0]), max[1].max(v[2])];
    }
    let mut grid = HeightGrid::new(min, max, cell)?;
    for triangle in &triangles {
        grid.add_triangle(*triangle);
    }
    // Interiors such as the tutorial cave ship a flat y = 0 plane; their floor
    // is room-model geometry, so a grid would pull enemies underground.
    if let Some((lo, hi)) = grid.height_range().filter(|(lo, hi)| hi - lo < 0.05) {
        // A refused bake writes no obstacle file either: a stale pair from an older run
        // would leave a `.obs` with no `.hgt` beside it.
        let _ = fs::remove_file(out.join(format!("zone_{zone}.obs")));
        let _ = fs::remove_file(out.join(format!("zone_{zone}.obs.json")));
        return Err(format!(
            "{chunks}: the terrain is a flat placeholder (every height {lo:.1}..{hi:.1}); this zone's floor is model geometry, so no grid was written"
        ));
    }
    let path = out.join(format!("zone_{zone}.hgt"));
    fs::write(&path, grid.to_bytes()).map_err(|e| format!("{}: {e}", path.display()))?;
    // T534: the obstacle file is written only after the height grid was accepted, so a
    // zone that the CLI refuses cannot end up with obstacles but no ground grid.
    if obstacles {
        // Runtime objects belong to this terrain tile; a placement far outside the
        // rasterised extent is corrupt or from another zone and must not stretch the
        // obstacle grid past its bounds.
        let bounds = [
            grid.min_x - ZONE_OBJECT_MARGIN_M,
            grid.min_z - ZONE_OBJECT_MARGIN_M,
            grid.min_x + grid.cols.saturating_sub(1) as f32 * grid.cell + ZONE_OBJECT_MARGIN_M,
            grid.min_z + grid.rows.saturating_sub(1) as f32 * grid.cell + ZONE_OBJECT_MARGIN_M,
        ];
        bake_obstacles(
            &assets,
            &assets_by_name,
            &chunks,
            zone,
            &out,
            &objects,
            &mut inputs,
            bounds,
        )?;
    }
    println!(
        "{chunks} -> {}: {loaded} chunks ({skipped} skipped), {} triangles, {}x{} nodes at {cell} m, x {:.0}..{:.0}, z {:.0}..{:.0}, {:.1}% covered",
        path.display(),
        triangles.len(),
        grid.cols,
        grid.rows,
        min[0],
        max[0],
        min[1],
        max[1],
        grid.coverage() * 100.0
    );
    if let Some(check) = check {
        report_check(&grid, &check)?;
    }
    Ok(())
}

/// T534: bake the placed `CDTA` meshes of one zone and a sidecar manifest of every
/// input's sha256. The bytes depend only on the (ordered) client assets, so baking
/// twice writes the same file.
fn bake_obstacles(
    assets: &Path,
    assets_by_name: &HashMap<String, String>,
    chunks: &str,
    zone: u32,
    out: &Path,
    objects: &[piece_098::TerrainObject],
    inputs: &mut BTreeMap<String, (u64, String)>,
    bounds: [f32; 4],
) -> Result<(), String> {
    // A failed or empty re-bake must not leave an older mesh beside the new grid: remove the
    // previous pair before any work, and write both files again only after the mesh is built.
    let path = out.join(format!("zone_{zone}.obs"));
    let manifest_path = out.join(format!("zone_{zone}.obs.json"));
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&manifest_path);
    let mut seen = HashSet::new();
    let (mut unique, mut duplicates, mut adr_objects) = (0, 0, 0);
    let (mut collision_objects, mut missing, mut unparsed, mut non_finite) = (0, 0, 0, 0);
    let mut outliers = 0;
    let mut collision = HashMap::<String, Option<Vec<[[f32; 3]; 3]>>>::new();
    let mut actor_cache = HashMap::<String, Option<String>>::new();
    let mut world = Vec::new();
    for object in objects {
        if !seen.insert(object.object_id) {
            duplicates += 1;
            continue;
        }
        unique += 1;
        let name = object.file_name.to_ascii_lowercase();
        if !name.ends_with(".adr") {
            continue;
        }
        adr_objects += 1;
        let Some((position, rotation)) = finite_placement(object) else {
            non_finite += 1;
            continue;
        };
        let Some(adr_path) = assets_by_name.get(&name) else {
            missing += 1;
            continue;
        };
        let collision_name = actor_cache
            .entry(name.clone())
            .or_insert_with(|| {
                let bytes = read_input(assets, adr_path, inputs)?;
                piece_098::load_actor(&bytes)
                    .ok()
                    .and_then(|actor| actor.collision_file)
            })
            .clone();
        let Some(collision_name) = collision_name else {
            continue;
        };
        let cached = collision.entry(collision_name.clone()).or_insert_with(|| {
            let path = assets_by_name.get(&collision_name.to_ascii_lowercase())?;
            let bytes = read_input(assets, path, inputs)?;
            let mesh = piece_064::CdtaMesh::parse(&bytes).ok()?;
            Some(mesh.triangles().collect())
        });
        let Some(mesh) = cached.as_ref() else {
            if assets_by_name.contains_key(&collision_name.to_ascii_lowercase()) {
                unparsed += 1;
            } else {
                missing += 1;
            }
            continue;
        };
        collision_objects += 1;
        let mut placed_triangles = Vec::with_capacity(mesh.len());
        let mut outside = false;
        for triangle in mesh {
            let world_triangle =
                triangle.map(|vertex| placed(position, rotation, object.scale, vertex));
            if !world_triangle
                .iter()
                .flatten()
                .all(|value| value.is_finite())
            {
                continue;
            }
            outside |= !world_triangle.iter().all(|vertex| {
                (bounds[0]..=bounds[2]).contains(&vertex[0])
                    && (bounds[1]..=bounds[3]).contains(&vertex[2])
            });
            placed_triangles.push(world_triangle);
        }
        if outside {
            outliers += 1;
            continue;
        }
        world.extend(placed_triangles);
    }
    if world.is_empty() {
        println!(
            "{chunks}: zone {zone} has no resolved CDTA meshes; no zone_{zone}.obs written ({outliers} object(s) outside the zone bounds skipped)"
        );
        return Ok(());
    }
    let mesh = ZoneObstacles::from_world_triangles(world, CELL_M)
        .map_err(|e| format!("{chunks}: zone {zone}: {e}"))?;
    replace_bytes(&path, &mesh.to_bytes()).map_err(|e| format!("{}: {e}", path.display()))?;
    let manifest = serde_json::json!({
        "version": 1,
        "zone": zone,
        "chunks": chunks,
        "cell_m": CELL_M,
        "objects": objects.len(),
        "unique_objects": unique,
        "duplicate_objects": duplicates,
        "adr_objects": adr_objects,
        "collision_objects": collision_objects,
        "missing_collision": missing,
        "unparsed_collision": unparsed,
        "non_finite_objects": non_finite,
        "outlier_objects": outliers,
        "triangles": mesh.triangle_count(),
        "inputs": inputs
            .iter()
            .map(|(path, (bytes, sha256))| serde_json::json!({
                "path": path,
                "bytes": bytes,
                "sha256": sha256,
            }))
            .collect::<Vec<_>>(),
    });
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    replace_bytes(&manifest_path, &bytes)
        .map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    println!(
        "{chunks} -> {}: {unique} objects ({duplicates} duplicate ids skipped), {adr_objects} ADR, {collision_objects} collision meshes, {missing} missing, {unparsed} unparsed, {outliers} outside the zone bounds, {} triangles, {:.1} MB; manifest {}",
        path.display(),
        mesh.triangle_count(),
        path.metadata().map_or(0.0, |meta| meta.len() as f64 / 1e6),
        manifest_path.display()
    );
    Ok(())
}

fn finite_placement(object: &piece_098::TerrainObject) -> Option<([f32; 3], [f32; 3])> {
    let position = [object.position[0], object.position[1], object.position[2]];
    let rotation = [object.rotation[0], object.rotation[1], object.rotation[2]];
    if position
        .iter()
        .chain(&rotation)
        .all(|value| value.is_finite())
        && object.scale.is_finite()
    {
        Some((position, rotation))
    } else {
        None
    }
}

fn read_input(
    assets: &Path,
    relative: &str,
    inputs: &mut BTreeMap<String, (u64, String)>,
) -> Option<Vec<u8>> {
    let bytes = fs::read(assets.join(relative)).ok()?;
    inputs
        .entry(relative.to_string())
        .or_insert_with(|| (bytes.len() as u64, sha256_hex(&bytes)));
    Some(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Compare spawn heights (mostly NPCs standing on the ground) with the grid.
fn report_check(grid: &HeightGrid, path: &PathBuf) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut errors: Vec<f32> = json["records"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|record| {
            let p = record["position"].as_array()?;
            let [x, y, z] =
                [0, 1, 2].map(|i| p.get(i).and_then(|v| v.as_f64()).unwrap_or(f64::NAN) as f32);
            Some((grid.height_at(x, z)? - y).abs())
        })
        .collect();
    errors.sort_by(f32::total_cmp);
    if errors.is_empty() {
        println!("check: no spawn lies on this grid");
        return Ok(());
    }
    let at = |q: f32| errors[((errors.len() as f32 * q) as usize).min(errors.len() - 1)];
    let within = |m: f32| errors.iter().filter(|e| **e <= m).count() * 100 / errors.len();
    println!(
        "check: {} spawns on the grid, median error {:.2} m, p90 {:.2} m, {}% within 0.5 m, {}% within 2 m",
        errors.len(),
        at(0.5),
        at(0.9),
        within(0.5),
        within(2.0)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_compressed_length(out: &mut Vec<u8>, value: usize) {
        if value < 0x80 {
            out.push(value as u8);
        } else if value < 0x10000 {
            out.push(0x80 | ((value >> 8) as u8));
            out.push((value & 0xff) as u8);
        } else {
            out.push(0xff);
            out.extend((value as u32).to_le_bytes());
        }
    }

    fn push_record(out: &mut Vec<u8>, record_type: u8, payload: &[u8]) {
        out.push(record_type);
        push_compressed_length(out, payload.len());
        out.extend_from_slice(payload);
    }

    /// The smallest `ADR` the reader accepts: one section-13 record naming the collision
    /// companion, which is the only field the bake reads.
    fn synthetic_adr() -> Vec<u8> {
        let mut collision = Vec::new();
        push_record(&mut collision, 1, b"treeble.cdt\0");
        let mut file = Vec::new();
        file.push(13);
        push_compressed_length(&mut file, collision.len());
        file.extend(collision);
        file
    }

    /// One `CDTA` prefix triangle: the run's local (0,0,0)-(1,0,0)-(0,2,0).
    fn synthetic_cdta() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(b"CDTA");
        bytes.extend(1u32.to_le_bytes());
        for value in [1u32, 1, 0] {
            bytes.extend(value.to_le_bytes());
        }
        let vertices = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 2.0, 0.0]];
        bytes.extend((vertices.len() as u32).to_le_bytes());
        for vertex in vertices {
            for value in vertex {
                bytes.extend(value.to_le_bytes());
            }
        }
        bytes.extend(3u32.to_le_bytes());
        for index in [0u16, 1, 2] {
            bytes.extend(index.to_le_bytes());
        }
        bytes
    }

    /// The bake writes both the `.obs` and its input manifest; two runs over the same
    /// assets must produce byte-identical files, and the manifest must name every input
    /// with its sha256.
    #[test]
    fn the_bake_is_deterministic_and_lists_its_inputs() {
        let root = std::env::temp_dir().join(format!("osfr-terrain-bake-{}", std::process::id()));
        let assets = root.join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("treeble.adr"), synthetic_adr()).unwrap();
        fs::write(assets.join("treeble.cdt"), synthetic_cdta()).unwrap();
        let names: HashMap<String, String> = [
            ("treeble.adr".to_string(), "treeble.adr".to_string()),
            ("treeble.cdt".to_string(), "treeble.cdt".to_string()),
        ]
        .into_iter()
        .collect();
        let objects = [piece_098::TerrainObject {
            file_name: "treeble.adr".to_string(),
            alias: String::new(),
            position: [0.0, 0.0, 0.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: 1.0,
            material_name: None,
            tint_alias: None,
            object_id: 7,
            unknown12_entries: 0,
        }];
        let mut bakes = Vec::new();
        for run in 0..2 {
            let out = root.join(format!("out{run}"));
            fs::create_dir_all(&out).unwrap();
            let mut inputs = BTreeMap::new();
            bake_obstacles(
                &assets,
                &names,
                "Synthetic",
                7,
                &out,
                &objects,
                &mut inputs,
                [-10.0, -10.0, 10.0, 10.0],
            )
            .unwrap();
            bakes.push((
                fs::read(out.join("zone_7.obs")).unwrap(),
                fs::read(out.join("zone_7.obs.json")).unwrap(),
            ));
        }
        assert_eq!(bakes[0].0, bakes[1].0, "the obstacle file is deterministic");
        assert_eq!(bakes[0].1, bakes[1].1, "the manifest is deterministic");
        assert_eq!(
            sha256_hex(&bakes[1].0),
            sha256_hex(&bakes[0].0),
            "two bakes give one sha256"
        );
        let manifest: serde_json::Value = serde_json::from_slice(&bakes[0].1).unwrap();
        assert_eq!(manifest["zone"], 7);
        assert_eq!(manifest["unique_objects"], 1);
        assert_eq!(manifest["collision_objects"], 1);
        assert_eq!(manifest["outlier_objects"], 0);
        assert_eq!(manifest["triangles"], 1);
        let inputs = manifest["inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), 2, "the ADR and the CDT are both hashed");
        for input in inputs {
            assert_eq!(
                input["sha256"].as_str().map(str::len),
                Some(64),
                "every input has a sha256: {input:?}"
            );
        }
        fs::remove_dir_all(&root).unwrap();
    }

    fn object(file_name: &str, position: [f32; 3], object_id: i32) -> piece_098::TerrainObject {
        piece_098::TerrainObject {
            file_name: file_name.to_string(),
            alias: String::new(),
            position: [position[0], position[1], position[2], 1.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: 1.0,
            material_name: None,
            tint_alias: None,
            object_id,
            unknown12_entries: 0,
        }
    }

    fn synthetic_assets(label: &str) -> (std::path::PathBuf, HashMap<String, String>) {
        let root =
            std::env::temp_dir().join(format!("osfr-terrain-stale-{}-{label}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let assets = root.join("assets");
        fs::create_dir_all(&assets).unwrap();
        fs::write(assets.join("treeble.adr"), synthetic_adr()).unwrap();
        fs::write(assets.join("treeble.cdt"), synthetic_cdta()).unwrap();
        let names: HashMap<String, String> = [
            ("treeble.adr".to_string(), "treeble.adr".to_string()),
            ("treeble.cdt".to_string(), "treeble.cdt".to_string()),
        ]
        .into_iter()
        .collect();
        (root, names)
    }

    /// T534 review: a failed or empty re-bake must remove the previous `.obs` pair; otherwise
    /// the new height grid would be loaded beside an older mesh.
    #[test]
    fn an_empty_or_refused_rebake_removes_stale_obstacle_files() {
        let (root, names) = synthetic_assets("empty");
        let out = root.join("out");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("zone_7.obs"), b"stale").unwrap();
        fs::write(out.join("zone_7.obs.json"), b"stale").unwrap();
        let mut inputs = BTreeMap::new();
        bake_obstacles(
            &root.join("assets"),
            &names,
            "Synthetic",
            7,
            &out,
            &[],
            &mut inputs,
            [-10.0, -10.0, 10.0, 10.0],
        )
        .unwrap();
        assert!(!out.join("zone_7.obs").exists(), "the stale mesh is gone");
        assert!(
            !out.join("zone_7.obs.json").exists(),
            "the stale manifest is gone"
        );

        fs::write(out.join("zone_7.obs"), b"stale").unwrap();
        fs::write(out.join("zone_7.obs.json"), b"stale").unwrap();
        let refused = [object("treeble.adr", [f32::NAN, 0.0, 0.0], 7)];
        bake_obstacles(
            &root.join("assets"),
            &names,
            "Synthetic",
            7,
            &out,
            &refused,
            &mut inputs,
            [-10.0, -10.0, 10.0, 10.0],
        )
        .unwrap();
        assert!(!out.join("zone_7.obs").exists());
        assert!(!out.join("zone_7.obs.json").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    /// T534 review: one finite but far-away placement is skipped and reported, instead of
    /// stretching the grid past its bounds and failing the whole zone.
    #[test]
    fn a_placement_outside_the_zone_bounds_is_skipped_and_logged() {
        let (root, names) = synthetic_assets("outlier");
        let out = root.join("out");
        fs::create_dir_all(&out).unwrap();
        let objects = [
            object("treeble.adr", [0.0, 0.0, 0.0], 7),
            object("treeble.adr", [1_000_000.0, 0.0, 0.0], 8),
        ];
        let mut inputs = BTreeMap::new();
        bake_obstacles(
            &root.join("assets"),
            &names,
            "Synthetic",
            7,
            &out,
            &objects,
            &mut inputs,
            [-10.0, -10.0, 10.0, 10.0],
        )
        .unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("zone_7.obs.json")).unwrap()).unwrap();
        assert_eq!(manifest["outlier_objects"], 1);
        assert_eq!(manifest["triangles"], 1, "the good placement is kept");
        assert!(out.join("zone_7.obs").exists());
        fs::remove_dir_all(&root).unwrap();
    }
}
