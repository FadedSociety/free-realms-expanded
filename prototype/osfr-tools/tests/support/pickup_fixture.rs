//! Shared fixture and refusal snapshot for the personal-pickup suites (T450 exploration
//! coins, T524 collection nodes).
//!
//! Include with `#[path = "support/pickup_fixture.rs"] mod pickup_fixture;`. The fixture is
//! the private opening package plus the shipped `mining.json` (the busy cases start a real
//! Mining18 run); NPC and collection content are the repository's `data/content` unless
//! [`PickupFixture::npc_package`] makes the folder the session's NPC content root. Test-only:
//! production never copies content.
#![allow(dead_code)]

#[path = "../common/content_copy.rs"]
mod content_copy;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    mining,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub fn bundled() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn opening() -> PathBuf {
    std::env::var_os("OSFR_TYPE8_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-journal-text-v2/content")
        })
}

/// The clicking character: guid 1, the open world, standing at `position`.
pub fn at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

pub struct PickupFixture {
    dir: PathBuf,
}

impl PickupFixture {
    /// The opening package plus the shipped Mining18 board.
    pub fn new(prefix: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-{prefix}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        content_copy::copy_tree(&opening(), &dir);
        fs::copy(
            bundled().join(mining::CONTENT_FILE),
            dir.join(mining::CONTENT_FILE),
        )
        .unwrap();
        Self { dir }
    }

    /// The same save content, marked as the session's NPC content root (`npcs/` plus
    /// `npc_spawns_extra.json`) with one regional file for the loader's anchors and no extra
    /// spawns. The caller supplies the NPC and collection files this test needs.
    pub fn npc_package(prefix: &str) -> Self {
        let fixture = Self::new(prefix);
        let content = bundled();
        fs::copy(content.join("zones.json"), fixture.dir.join("zones.json")).unwrap();
        fs::write(
            fixture.dir.join("npc_spawns_extra.json"),
            br#"{"records":[]}"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.dir.join("npcs")).unwrap();
        fs::copy(
            content.join("npcs/durango_zoo.json"),
            fixture.dir.join("npcs/durango_zoo.json"),
        )
        .unwrap();
        fixture
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn load(&self, state: Option<GameplayState>) -> Gameplay {
        Gameplay::load(&self.dir, state).expect("gameplay content loads")
    }

    /// The opening save: neither gather job is unlocked, so a collect click that ran the
    /// gather admission would answer NoJob.
    pub fn jobless(&self) -> GameplayState {
        self.load(None).state()
    }
}

impl Drop for PickupFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// What a refusal must leave alone: the save and the visibility revision.
pub fn snapshot(game: &Gameplay) -> (GameplayState, u64) {
    (game.state(), game.npc_visibility_revision())
}

/// 35/2 AddNpc: the actor guid follows the four-byte header (reader `0x00921a10`).
pub fn added(packets: &[Vec<u8>], guid: u64) -> bool {
    packets.iter().any(|packet| {
        packet.starts_with(&[35, 0, 2, 0]) && packet.get(4..12) == Some(&guid.to_le_bytes()[..])
    })
}

/// 35/3 removal: a u16 flag at 4..6, then the actor guid at 6..14 (reader `0x008d69f0`).
pub fn removed(packets: &[Vec<u8>], guid: u64) -> bool {
    packets.iter().any(|packet| {
        packet.starts_with(&[35, 0, 3, 0]) && packet.get(6..14) == Some(&guid.to_le_bytes()[..])
    })
}
