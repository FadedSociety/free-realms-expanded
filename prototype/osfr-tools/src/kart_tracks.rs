//! P-kart-racing: one race package per track scene beyond the frozen Wildwood one
//! (`data/content/kart_tracks/<scene>.json`, built from the client's own gate curtains and track
//! collision by `lab/tools/kart_track_build.py`; see the kart-racing sheet, section 7 item 6).
//!
//! A package names the scene the race zones into and the race the service judges there. The entry
//! frames, the kart, the schedule and the price stay the Wildwood package's: the gateway rebinds
//! the scene position and the start transform when a row on one of these tracks runs, and binds
//! the registered price (a package carries no `reward`). Every rule in a package is reconstructed
//! (R) and tunable; its `source` says what the client proved.
use crate::{
    content::{SourceTag, check_sources},
    kart_service::{KartService, KartServiceContent},
    zone_transition::{self, ZoneTransition},
};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    io::{self, Read},
    path::Path,
};

const DIRECTORY: &str = "kart_tracks";
const MAX_FILE: usize = 512 * 1024;
const MAX_TRACKS: usize = 16;

/// One track's package: where the race zones into and what the service judges there.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackPackage {
    schema_version: u32,
    /// The op31 scene frame: name, spawn position, sky. `id` is rebound to the instance at entry.
    pub scene: ZoneTransition,
    /// (R) The kart's facing on the grid, degrees clockwise from +z; the start basis is built
    /// from it.
    #[serde(default)]
    pub heading_deg: f32,
    pub race: KartServiceContent,
    pub source: Vec<SourceTag>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

/// Reads every `kart_tracks/<scene>.json`; an absent directory means the Wildwood package alone.
/// The gateway binds each package to the base package's price before it serves a race.
pub fn load(directory: &Path) -> io::Result<BTreeMap<String, TrackPackage>> {
    let dir = directory.join(DIRECTORY);
    let mut tracks = BTreeMap::new();
    let entries = match std::fs::read_dir(&dir) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(tracks),
        Err(e) => return Err(e),
        Ok(entries) => entries,
    };
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.len() > MAX_TRACKS {
        return Err(invalid("too many kart track packages"));
    }
    for path in paths {
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| invalid("kart track file name"))?
            .to_owned();
        if !std::fs::symlink_metadata(&path)?.is_file() {
            return Err(invalid(format!("kart track {stem} must be a regular file")));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(&path)?
            .take((MAX_FILE + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_FILE {
            return Err(invalid(format!("kart track {stem} exceeds 512 KiB")));
        }
        let package: TrackPackage = serde_json::from_slice(&bytes)
            .map_err(|e| invalid(format!("kart track {stem}: {e}")))?;
        if package.schema_version != 1
            || package.scene.scene != stem
            || !package.heading_deg.is_finite()
            || package.scene.id != 0
        {
            return Err(invalid(format!("invalid kart track package {stem}")));
        }
        // Every track pays the base package's registered price; a price of its own would be dead.
        if package.race.reward.is_some() {
            return Err(invalid(format!(
                "kart track {stem} names a reward; the base package's price applies"
            )));
        }
        zone_transition::encode(&package.scene)?;
        KartService::new(package.race.clone())
            .map_err(|e| invalid(format!("kart track {stem}: {e}")))?;
        check_sources(&format!("kart track {stem}"), &package.source).map_err(invalid)?;
        tracks.insert(stem, package);
    }
    Ok(tracks)
}

impl TrackPackage {
    /// The 3 x 3 row-major basis of a kart facing `heading_deg` on level ground: row 2 is the
    /// forward axis, as the captured 33/5 and 33/6 bodies carry it (identity at Wildwood).
    pub fn start_basis(&self) -> [f32; 9] {
        let (sin, cos) = self.heading_deg.to_radians().sin_cos();
        [cos, 0.0, -sin, 0.0, 1.0, 0.0, sin, 0.0, cos]
    }
}
