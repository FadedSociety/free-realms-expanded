//! L103: Mail Sorting, the Postman's minigame, on the shared Type8 engine.
//!
//! Recovered: the client's `game_mailsorting.gfx` is the mining movie with other art and sounds.
//! Its server network, level, spawn, progression and bin classes are byte-identical to
//! `game_mining.gfx`'s (only `ConfigServer.GAME_ID` 12 and the client sound table differ), so it
//! speaks the Type8 protocol the mining and harvesting engines serve. Its eight items are the
//! client `CodeStringMappings.txt` rows `mailValue401`..`mailValue408` (Sealed Envelope, Blue
//! Envelope, Box with Twine, Gift Box, Stamped Envelope, Opened Envelope, Scroll, Box), and its
//! tutorial (Global.Text 19323) asks four items per level, filled by chains of three or more:
//! the farming board's selectable goals. The movie ships selectable and power-up tiles but no
//! collectible tile. The six town games are client MiniGameData 376-381 (type 8), the live Games
//! window rows 414, 415, 416, 567, 1037 and 1038.
//!
//! Reconstructed (R): which four items each town asks for, the board tuning (the farming
//! board's) and the payout, which is no item and Postman XP per sorted run like the Miner's and
//! Chef's. The Games window and a town postbox's Mail Sorting row (`gameplay/npc/mailbox.rs`, the
//! same gates) start it; there is no gathering node. L106: its rows ask the live Postman rank
//! floors (`rank_floor`).

/// L106: the Postman rank a town's sorting asks. The live Games window rows carried
/// RequirementSets 7/8/9 (Seaside 415 and Merry Vale 1038: 7; Wugachug 416: 8; Snowhill 567 and
/// Shrouded Glade 1037: 9), client Requirements 9/10/11 = `CheckRankLastProfileByType` profile
/// type 3 at rank 5/9/13; Sanctuary 414 carried none. (R) Type 3 is the Postman.
pub fn rank_floor(game: i32) -> u32 {
    match game {
        377 | 381 => 5,
        378 => 9,
        379 | 380 => 13,
        _ => 0,
    }
}

/// The content file; absent means no Mail Sorting.
pub const CONTENT_FILE: &str = "mail_sorting.json";
/// Sanctuary Mail Sorting (client MiniGameData 376), the file's own game.
pub const GAME_ID: i32 = 376;
/// The Postman job (`jobs.json` profile 4).
pub const POSTMAN_PROFILE_ID: i32 = 4;
/// (R) Run ids of its own, apart from every other minigame's.
pub const FIRST_INSTANCE_ID: i32 = 376_000_001;
/// (R) Postman XP for a sorted run, the Miner's and Chef's amount.
pub const RECONSTRUCTED_POSTMAN_XP_PER_WIN: u32 = 25;

use crate::mining::{Type8Engine, Type8State};
use std::{io, path::Path};

/// The engine for `directory`'s Mail Sorting file with the player's saved runs, bound to the
/// Postman job.
pub fn load(directory: &Path, saved: Option<Type8State>) -> io::Result<Type8Engine> {
    let engine = Type8Engine::load(directory, CONTENT_FILE, GAME_ID, FIRST_INSTANCE_ID, saved)?;
    if engine.profile_id() != POSTMAN_PROFILE_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "mail sorting content does not bind the Postman job",
        ));
    }
    // A sorted run pays XP only; a board with item outputs would fail at its win instead.
    if engine.games().any(|(_, spec)| !spec.outputs.is_empty()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "mail sorting boards pay no items",
        ));
    }
    Ok(engine)
}
