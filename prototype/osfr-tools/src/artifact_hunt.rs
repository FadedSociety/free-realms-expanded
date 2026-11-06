//! Artifact Hunt, on the shared Type8 engine.
//!
//! Recovered: the client's `game_artifact.gfx` is the mining movie with other art and sounds. Its
//! `GameServer` differs from `game_mining.gfx`'s by three trace lines and `ConfigServer` by
//! `GAME_ID` (11, mining's is 1), so it speaks the Type8 protocol. Its text (Global.Text 35605,
//! tutorial 19301) asks four kinds of artifact per level, in chains of three or more: Mail
//! Sorting's board. Its eight items are `artifactValue401`..`408` in `CodeStringMappings.txt`
//! (Figurine, Chalice, Bone, Jug, Helmet, Arrowhead, Bowl, Goblet). The game is MiniGameData 57
//! (type 8), the live Games window row 413.
//!
//! Reconstructed (R): which four items a level asks, the board tuning (Mail Sorting's) and the
//! payout. No job is named anywhere for it: it is an arcade row, so like chess it runs in whatever
//! job the player is in, and pays that job. The engine wants a profile, so the file names the
//! Adventurer, whom everyone has. Only the Games window starts it.
use crate::mining::{Type8Engine, Type8State};
use std::{io, path::Path};

/// The content file; absent means no Artifact Hunt.
pub const CONTENT_FILE: &str = "artifact_hunt.json";
/// Client MiniGameData 57.
pub const GAME_ID: i32 = 57;
/// (R) The Adventurer job (`jobs.json` profile 1).
pub const ADVENTURER_PROFILE_ID: i32 = 1;
/// (R) Run ids of its own, apart from every other minigame's.
pub const FIRST_INSTANCE_ID: i32 = 57_000_001;
/// (R) XP for a finished hunt, flat, to whichever job the player is in.
pub const RECONSTRUCTED_XP_PER_WIN: u32 = 25;

/// The engine for `directory`'s file with the player's saved runs, bound to the Adventurer.
pub fn load(directory: &Path, saved: Option<Type8State>) -> io::Result<Type8Engine> {
    let engine = Type8Engine::load(directory, CONTENT_FILE, GAME_ID, FIRST_INSTANCE_ID, saved)?;
    if engine.profile_id() != ADVENTURER_PROFILE_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "artifact hunt content does not bind the Adventurer job",
        ));
    }
    // A hunt pays XP only; a board with item outputs would fail at its win instead.
    if engine.games().any(|(_, spec)| !spec.outputs.is_empty()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "artifact hunt boards pay no items",
        ));
    }
    Ok(engine)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: usize = 15;

    /// The shipped file loads, offers the one row, and starts the artifact movie.
    #[test]
    fn the_shipped_hunt_starts_its_own_movie() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut engine = load(&directory, None).unwrap();
        assert_eq!(engine.launch_movie(), "game_artifact.gfx");
        assert_eq!(engine.preload_file(), "ArtifactAssets.txt");
        // Retail level 1's four bins, each counting up to 15 (`f3ZJ9i5YnaI_148.00`).
        let goals = engine.spec().goals.iter();
        assert_eq!(
            goals
                .map(|g| (g.value, g.quota, g.per_reward))
                .collect::<Vec<_>>(),
            [(401, 15, 1), (402, 15, 1), (405, 15, 1), (407, 15, 1)]
        );
        let games: Vec<_> = engine.games().map(|(game, _)| game).collect();
        assert_eq!(games, [GAME_ID]);
        let offer = engine.begin_game(None, ADVENTURER_PROFILE_ID).unwrap();
        assert!(engine.active());
        // Retail's offer says Difficulty 2: the fourth id after the family, subtype and three ids.
        let fourth = HEADER + 12;
        assert_eq!(offer.replies[0][fourth..fourth + 4], 2_i32.to_le_bytes());
    }

    /// Only the Adventurer's start is accepted.
    #[test]
    fn the_hunt_is_refused_under_another_job() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut engine = load(&directory, None).unwrap();
        assert!(engine.begin_game(None, 4).is_err() || !engine.active());
    }
}
