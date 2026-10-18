//! One local character snapshot. This is not an account or world database.
//!
//! Saves replace the destination with a synced temporary file in the same
//! directory. A failed validation, write, or rename leaves the previous save
//! intact; the destination is never deleted before replacement.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};

// (R) Local JSON format version; no original client save schema was recovered.
const SCHEMA_VERSION: u32 = 2;
pub const MAX_FILE_BYTES: usize = 16 * 1024;
/// A local defensive bound, not a recovered Free Realms world or angle limit.
pub const MAX_COMPONENT_MAGNITUDE: f32 = 1_000_000.0;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterState {
    pub guid: u64,
    pub zone_id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

impl CharacterState {
    pub fn validate(&self) -> io::Result<()> {
        if self.guid == 0 {
            return Err(invalid_data("character GUID must be nonzero"));
        }
        for component in self.position.iter().chain(&self.rotation) {
            if !component.is_finite() || component.abs() > MAX_COMPONENT_MAGNITUDE {
                return Err(invalid_data(
                    "character coordinates exceed the finite local bounds",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schema_version: Option<u32>,
    #[serde(default, skip_serializing)]
    version: Option<u32>,
    character: CharacterState,
}

/// Load an existing save, or persist and return `default` if the file is absent.
/// Corrupt, unsupported, oversized, and unreadable existing saves are errors;
/// they are never silently replaced with the default character.
pub fn load_or_create(path: &Path, default: CharacterState) -> io::Result<CharacterState> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            save(path, &default)?;
            return Ok(default);
        }
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take((MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(invalid_data("character save exceeds 16 KiB"));
    }
    let document: Document = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    let supported = match (document.schema_version, document.version) {
        (Some(SCHEMA_VERSION), None) | (None, Some(1)) => true,
        _ => false,
    };
    if !supported {
        return Err(invalid_data("unsupported character save version"));
    }
    document.character.validate()?;
    Ok(document.character)
}

/// Atomically replace a save on the same filesystem, including on Windows.
/// There must be only one writer for this character file. This does not promise
/// directory-entry durability across power loss or cross-process write locking.
pub fn save(path: &Path, state: &CharacterState) -> io::Result<()> {
    state.validate()?;
    let document = Document {
        schema_version: Some(SCHEMA_VERSION),
        version: None,
        character: state.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&document).map_err(invalid_data)?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(invalid_data("character save exceeds 16 KiB"));
    }
    replace_bytes(path, &bytes)
}

/// Shared same-directory replacement used by the expanded gameplay snapshot.
pub fn replace_bytes(path: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_bytes_with(path, bytes, |from, to| fs::rename(from, to))
}

fn replace_bytes_with(
    path: &Path,
    bytes: &[u8],
    mut replace: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let mut temporary = create_temporary(path)?;
    let file = temporary.file.as_mut().expect("temporary file is open");
    file.write_all(bytes)?;
    file.sync_all()?;
    // Windows requires closing our temporary-file handle before replacement.
    drop(temporary.file.take());
    let mut retries = 0;
    loop {
        match replace(&temporary.path, path) {
            Ok(()) => break,
            // A concurrent Windows reader without FILE_SHARE_DELETE can deny
            // replacement briefly. Retry only that atomic rename, never delete
            // or truncate the previous save. Persistent denial still fails.
            Err(error)
                if cfg!(windows)
                    && matches!(error.raw_os_error(), Some(5 | 32))
                    && retries < 20 =>
            {
                retries += 1;
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(error) => return Err(error),
        }
    }
    temporary.committed = true;
    Ok(())
}

struct TemporaryFile {
    path: PathBuf,
    file: Option<File>,
    committed: bool,
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn create_temporary(destination: &Path) -> io::Result<TemporaryFile> {
    let filename = destination
        .file_name()
        .ok_or_else(|| invalid_data("character save path must name a file"))?;
    let directory = destination.parent().unwrap_or_else(|| Path::new("."));
    for _ in 0..32 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut name = filename.to_os_string();
        name.push(format!(".{}.{}.tmp", std::process::id(), sequence));
        let path = directory.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                return Ok(TemporaryFile {
                    path,
                    file: Some(file),
                    committed: false,
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a unique character save temporary file",
    ))
}

fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::replace_bytes_with;
    use std::{fs, io, sync::atomic::Ordering};

    #[test]
    fn injected_replacement_failure_preserves_the_previous_file() {
        let sequence = super::TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "osfr-save-replacement-failure-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("save.json");
        let previous = br#"{"state":"old"}"#;
        fs::write(&path, previous).unwrap();

        let error = replace_bytes_with(&path, br#"{"state":"new"}"#, |_, _| {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected rename failure",
            ))
        })
        .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(fs::read(&path).unwrap(), previous);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
