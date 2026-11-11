//! G5: each staff member's saved spots (`realm waypoint save|go|list|del`), keyed by account.
//! One small JSON file beside the character saves, read on each use and written whole through
//! the atomic `character_store::replace_bytes`, so a crash leaves the old file or the new one.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;

/// (R) Bounds: spots per staff account, label length, and the file a load accepts.
pub const MAX_PER_STAFF: usize = 64;
pub const MAX_LABEL: usize = 32;
const MAX_FILE_BYTES: u64 = 1 << 20;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Waypoint {
    pub label: String,
    pub zone_id: u32,
    pub position: [f32; 3],
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Waypoints {
    /// By staff account id.
    staff: BTreeMap<u64, Vec<Waypoint>>,
}

/// A label is 1 to `MAX_LABEL` characters with no control characters.
pub fn check_label(label: &str) -> Result<(), &'static str> {
    let count = label.chars().count();
    if count == 0 || count > MAX_LABEL || label.chars().any(char::is_control) {
        return Err("A waypoint label is 1 to 32 characters.");
    }
    Ok(())
}

impl Waypoints {
    /// The saved file, or none yet. A file that is too big or does not validate is refused.
    pub fn load(path: &Path) -> io::Result<Self> {
        match std::fs::metadata(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
            Ok(meta) if meta.len() > MAX_FILE_BYTES => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "waypoint file too large",
                ));
            }
            Ok(_) => {}
        }
        let saved: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        let valid = saved.staff.values().all(|spots| {
            spots.len() <= MAX_PER_STAFF
                && spots.iter().all(|spot| {
                    check_label(&spot.label).is_ok()
                        && spot.position.iter().all(|v| {
                            v.is_finite()
                                && v.abs() <= crate::character_store::MAX_COMPONENT_MAGNITUDE
                        })
                })
        });
        if !valid {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "waypoint file holds an invalid spot",
            ));
        }
        Ok(saved)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        crate::character_store::replace_bytes(path, &serde_json::to_vec_pretty(self)?)
    }

    pub fn list(&self, account: u64) -> &[Waypoint] {
        self.staff.get(&account).map_or(&[], Vec::as_slice)
    }

    /// The spot named `label`, ignoring case.
    pub fn find(&self, account: u64, label: &str) -> Option<&Waypoint> {
        self.list(account)
            .iter()
            .find(|spot| spot.label.eq_ignore_ascii_case(label))
    }

    /// Save or replace `label`.
    pub fn put(&mut self, account: u64, spot: Waypoint) -> Result<(), &'static str> {
        check_label(&spot.label)?;
        let spots = self.staff.entry(account).or_default();
        spots.retain(|old| !old.label.eq_ignore_ascii_case(&spot.label));
        if spots.len() >= MAX_PER_STAFF {
            return Err("You already have 64 waypoints; delete one first.");
        }
        spots.push(spot);
        spots.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
        Ok(())
    }

    /// Whether `label` was there.
    pub fn remove(&mut self, account: u64, label: &str) -> bool {
        let Some(spots) = self.staff.get_mut(&account) else {
            return false;
        };
        let before = spots.len();
        spots.retain(|spot| !spot.label.eq_ignore_ascii_case(label));
        let removed = spots.len() != before;
        if spots.is_empty() {
            self.staff.remove(&account);
        }
        removed
    }
}
