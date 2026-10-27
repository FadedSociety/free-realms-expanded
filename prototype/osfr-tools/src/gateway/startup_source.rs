//! Where the gateway's startup base comes from: the validated capture folder (`--reference`, lab
//! use) or the repo's own `<content>/startup/` files, which need no capture.
//!
//! `startup/manifest.json` holds the client version and the initial character, `self_data.json`
//! the typed defaults of the self-data message (opcode 12, `startup_selfdata::SelfData`) and
//! `snapshot.json` the typed snapshot behind the other 45 messages. Nothing is replayed bytes.

use super::{
    Gateway, MAX_REFERENCE, invalid, load_profile_rank_offsets, load_reference, read_bounded,
};
use crate::character_store::CharacterState;
use crate::gameplay::{Gameplay, GameplayState};
use crate::startup_selfdata::{self, SelfData};
use crate::startup_snapshot::configured_path;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

/// What `Gateway::load_internal` takes from either source.
pub(super) struct Base {
    pub version: String,
    pub initial_state: CharacterState,
    pub self_data: SelfData,
}

#[derive(Clone, Copy)]
pub(super) enum Source<'a> {
    /// A capture folder, validated in full by `load_reference`.
    Reference(&'a Path),
    /// A content directory holding `startup/`.
    Content(&'a Path),
}

/// A plain reference folder, as every `Gateway::load*` takes it.
impl<'a> From<&'a Path> for Source<'a> {
    fn from(directory: &'a Path) -> Self {
        Source::Reference(directory)
    }
}

/// Unknown keys (the file's `provenance` notes) are ignored.
#[derive(Deserialize)]
struct ContentManifest {
    schema_version: u32,
    client_version: String,
    initial_state: CharacterState,
}

/// The client version and the initial character from `<content>/startup/manifest.json`.
pub(super) fn content_manifest(content: &Path) -> io::Result<(String, CharacterState)> {
    let bytes = read_bounded(&content.join("startup/manifest.json"), 65_536)?;
    let manifest: ContentManifest =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid startup manifest JSON"))?;
    manifest.initial_state.validate()?;
    if manifest.schema_version != 1
        || manifest.client_version.is_empty()
        || manifest.client_version.len() > 64
    {
        return Err(invalid("unsupported startup manifest"));
    }
    Ok((manifest.client_version, manifest.initial_state))
}

impl Source<'_> {
    pub(super) fn base(self) -> io::Result<Base> {
        match self {
            Source::Reference(directory) => {
                let (manifest, self_bytes) = load_reference(directory)?;
                let self_data =
                    std::panic::catch_unwind(|| startup_selfdata::parse_self(&self_bytes))
                        .map_err(|_| invalid("invalid typed self-data reference template"))?;
                Ok(Base {
                    version: manifest.client_version,
                    initial_state: manifest.initial_state,
                    self_data,
                })
            }
            Source::Content(content) => {
                let (version, initial_state) = content_manifest(content)?;
                let bytes = read_bounded(&content.join("startup/self_data.json"), MAX_REFERENCE)?;
                let self_data: SelfData = serde_json::from_slice(&bytes)
                    .map_err(|_| invalid("invalid startup self-data JSON"))?;
                // A hand-edited file must fail here, not as a panic at the first login.
                std::panic::catch_unwind(|| startup_selfdata::build_self(&self_data))
                    .map_err(|_| invalid("startup self-data cannot be serialized"))?;
                Ok(Base {
                    version,
                    initial_state,
                    self_data,
                })
            }
        }
    }

    /// The typed startup snapshot file. Content mode never reads `OSFR_GATEWAY_STARTUP_SNAPSHOT`.
    pub(super) fn snapshot_path(self) -> io::Result<PathBuf> {
        match self {
            Source::Reference(directory) => configured_path(directory),
            Source::Content(content) => Ok(content.join("startup/snapshot.json")),
        }
    }
}

/// The constructors for a run without a `--reference` folder.
impl Gateway {
    /// The initial character from the repo's startup content.
    pub fn content_state(content: &Path) -> io::Result<CharacterState> {
        Ok(content_manifest(content)?.1)
    }

    /// `load`, with every startup message built from `<content>/startup/`.
    pub fn load_from_content(content: &Path, state: CharacterState) -> io::Result<Self> {
        Self::load_internal(Source::Content(content), state, None, None, BTreeMap::new())
    }

    /// `load_gameplay`, with every startup message built from `<content>/startup/`.
    pub fn load_gameplay_from_content(
        state: CharacterState,
        content: &Path,
        saved: Option<GameplayState>,
    ) -> io::Result<Self> {
        let mut gameplay = Gameplay::load(content, saved)?;
        gameplay.bind_npc_quest_catalogue(content, state.guid)?;
        let offsets = load_profile_rank_offsets(content)?;
        Self::load_internal(
            Source::Content(content),
            state,
            Some(gameplay),
            None,
            offsets,
        )
    }
}
