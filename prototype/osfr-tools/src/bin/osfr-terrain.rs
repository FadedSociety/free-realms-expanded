//! Build a zone's ground-height grid from the client's terrain chunks.
//!
//! osfr-terrain --catalog lab/manifests/rust-asset-catalog.tsv
//!   --assets lab/extracted/client-assets --chunks FabledRealms --zone 0
//!   --out lab/terrain [--cell 2] [--check data/content/npc_spawns_extra.json]
//!
//! The output is derived from client data: keep it under `lab/`, out of Git.
#[path = "../restoration/piece_098.rs"]
#[allow(dead_code)]
mod piece_098;

use osfr_tools::terrain::HeightGrid;
use std::{fs, path::PathBuf};

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
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--catalog" => catalog = Some(PathBuf::from(value()?)),
            "--assets" => assets = Some(PathBuf::from(value()?)),
            "--chunks" => chunks = Some(value()?),
            "--zone" => zone = Some(value()?.parse::<u32>().map_err(|e| e.to_string())?),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--cell" => cell = value()?.parse().map_err(|e| format!("--cell: {e}"))?,
            "--check" => check = Some(PathBuf::from(value()?)),
            _ => return Err(format!("unknown option {flag} (see the file header for usage)")),
        }
    }
    let (catalog, assets, chunks, zone, out) = (
        catalog.ok_or("--catalog is required")?,
        assets.ok_or("--assets is required")?,
        chunks.ok_or("--chunks (file name prefix, e.g. FabledRealms) is required")?,
        zone.ok_or("--zone is required")?,
        out.ok_or("--out is required")?,
    );
    let listing = fs::read_to_string(&catalog).map_err(|e| format!("{}: {e}", catalog.display()))?;
    let prefix = format!("{chunks}_");
    let mut triangles = Vec::new();
    let (mut loaded, mut skipped) = (0, 0);
    for line in listing.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 7 || !fields[2].starts_with(&prefix) || !fields[2].ends_with(".gcnk") {
            continue;
        }
        let bytes = fs::read(assets.join(fields[6])).map_err(|e| format!("{}: {e}", fields[6]))?;
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
                let vertex = |i: u16| tile.vertices[i as usize + batch.vertex_offset as usize].position;
                triangles.push([vertex(tri[0]), vertex(tri[1]), vertex(tri[2])]);
            }
        }
    }
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
        return Err(format!(
            "{chunks}: the terrain is a flat placeholder (every height {lo:.1}..{hi:.1}); this zone's floor is model geometry, so no grid was written"
        ));
    }
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let path = out.join(format!("zone_{zone}.hgt"));
    fs::write(&path, grid.to_bytes()).map_err(|e| format!("{}: {e}", path.display()))?;
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
            let [x, y, z] = [0, 1, 2].map(|i| p.get(i).and_then(|v| v.as_f64()).unwrap_or(f64::NAN) as f32);
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
