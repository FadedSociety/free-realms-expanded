//! W150: the activity portal list (167/1/1), its refresh (167/1/4) and the Play choice (167/1/2).
//!
//! # Recovered original facts (addresses in `FreeRealms.exe`)
//!
//! * Family 167 header, 4 bytes: `u16 167`, `u8 service`, `u8 sub-op` (`BaseActivityServicePacket`
//!   `008a20c0` stores 167 and the service, `BaseActivityPacket` `008a21f0` stores the sub-op;
//!   the writer `00913f70`/`0090b490` emits `u16`, `u8`, `u8`; the reader `00be25e0`/`00be2e80`
//!   reads them back). The activity packets are service 1; service 2 is the scheduled list.
//! * 167/1/4 refresh request (`008a2330`): the 4 header bytes and nothing else. The client sends it
//!   through both tunnels (`0092a2d0`, `0092a3e0`) when a job level changes (`009392c0`, update
//!   subtype 14), when membership activates (`009471a0`, message 100) and when it leaves a
//!   minigame (`009bd240`). It is NOT sent at login, so the list also has to be pushed.
//! * 167/1/2 Play (`ActivityJoinRequest_SendByType` `00be7d80`, ctor `00bdefa0`): the sender looks
//!   the id up in the list it was given first (an unlisted id never leaves the client), then
//!   writes header + `u32 activity id` + `u32` taken from `[client+0x74]` (`00be7010`); for entry
//!   type 1 it goes through the world tunnel (`00be77b0`), for type 2 through the client tunnel
//!   (`00be78c0`). 12 bytes. The last dword is the client's server-clock offset (`008bb4f0` adds it
//!   to the wall clock): never trusted, never read here.
//! * 167/1/1 list (`00be79d0` -> `00be75c0` -> `00be6fa0` -> `00be63e0`): header, `u32 entry type`
//!   (the client first removes every listed activity of that entry type, then adds the new rows),
//!   `i32 count`, then per row, little endian: `u32` Id, AppSystemId, Category, ImageSetId,
//!   ActivityPositionId, DisplayNameId, DisplayDescriptionId, NameId, DescriptionId, EntryType;
//!   `u8` CanPlayerJoin; `u32` preferred-state; `u32` featured-window count (+ 36 bytes each, always
//!   0 here); `u32` TutorialActivityId; `u8` MembersOnly; `i32 len` + bytes DetailImageFilename;
//!   `i32 len` + bytes ThumbnailImageFilename; `u32` Difficulty, unknown `+0xc`, unknown `+0x68`,
//!   MysteryChestIcon. The reader fails (and the client drops the list) if the packet has trailing
//!   or missing bytes. Field-to-column mapping is from `ClientActivityDatasource::GetCell`
//!   (`00cfc180`, jump table `00cfc434`) and the Lua `Activities_col`.
//! * `Ui.RequestToJoinActivity(id, 0)` (portal Play button) is the only Lua entry to `00be7d80`.
//!
//! # Reconstructed policy (R)
//!
//! * Only activities backed by a running battle or a configured native soccer match are published:
//!   one row per `field_battles.json` battle (category 18) and one per `soccer.json` game (category
//!   22, art `Soccer_Briarwood.dds`; both values are the client `ActivityCategories.txt` row 22),
//!   entry type 1. Every other activity (races, mining, card game ...) is simply not listed, so the
//!   client cannot ask for it.
//! * `CanPlayerJoin` is per player: battle rows require a combat job at the configured level;
//!   chess is open, and soccer uses its own configured level floor without a job gate. A
//!   refused Play says why in chat (the 147/2 join-error body is not recovered). The owning
//!   activity, never the portal request, awards.
//! * The preferred/featured/tutorial/AppSystem/chest fields and the two unknown dwords are 0.
//!   Image names come from each row's client category (`hacknslash_detail.dds` for combat and
//!   `Soccer_Briarwood.dds` for soccer).

use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::Path,
};

const FAMILY: u16 = 167;
const SERVICE_ACTIVITY: u8 = 1;
const SUB_LIST: u8 = 1;
const SUB_PLAY: u8 = 2;
const SUB_REFRESH: u8 = 4;
const ENTRY_WORLD_TUNNEL: u32 = 1;
const COMBAT_CATEGORY: u32 = 18;
/// Recovered soccer category row 22 in the client `Resources/ActivityCategories.txt`.
pub const SOCCER_CATEGORY: u32 = 22;
/// Recovered category-22 art name in the same client table.
pub const SOCCER_ART: &str = "Soccer_Briarwood.dds";
const ART: &str = "hacknslash_detail.dds";
/// (R) The type-6 row in `MiniGameTypeData.txt` carries CATEGORY_ID 0; the
/// portal categories table has no board-game row, so chess rows use it as-is.
const CHESS_CATEGORY: u32 = 0;
const CHESS_DETAIL: &str = "chess_detail.dds";
const CHESS_THUMBNAIL: &str = "chess_thumb.dds";
const MAX_ACTIVITIES: usize = 64;
const MAX_FILE: u64 = 1 << 20;
const PLAY_LEN: usize = 12;

/// What a published row starts. Battles use the encounter engine; chess rows
/// name the client minigame row the chess table service understands; soccer
/// rows begin the session-local native match service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Battle,
    Chess { row_id: u32 },
    Soccer,
}

/// One published activity: a battle of the frozen package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Activity {
    /// Wire id; equals the battle id.
    pub id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
    pub required_level: u32,
    pub kind: Kind,
}

/// One published chess row. Ids, names, icon and difficulty come from the
/// client `MiniGameData` type-6 rows; only the portal id is server policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChessActivity {
    pub row_id: u32,
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
}

#[derive(Deserialize)]
struct BattleFile {
    battles: Vec<Battle>,
}

#[derive(Deserialize)]
struct Battle {
    id: u32,
    title_id: u32,
    description_id: u32,
    difficulty: u32,
    #[serde(default)]
    required_combat_level: u32,
    #[serde(default)]
    icon_id: i64,
}

/// What the server knows about the asking player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewer {
    /// The active job is a combat job.
    pub combat_job: bool,
    /// Level of the active job.
    pub level: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Not a published activity (forged or stale id).
    Unknown,
    /// Another activity, fight, knock-out or movie owns the player.
    Busy,
    NotCombatJob,
    LevelTooLow(u32),
}

impl Refusal {
    pub fn message(self) -> String {
        match self {
            Self::Unknown => "That activity is not available.".into(),
            Self::Busy => "Finish what you are doing before starting an activity.".into(),
            Self::NotCombatJob => "Switch to a combat job before starting a battle.".into(),
            Self::LevelTooLow(level) => format!("This activity needs level {level}."),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    Refresh,
    Play { activity: u32 },
}

/// The published activities. Empty by default, which publishes an empty list.
///
/// Soccer rows live in the same list so the client can ask for them with the
/// portal Play request. The wire row shape is identical; `Kind` selects the
/// admission and category rules.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalogue {
    rows: Vec<Activity>,
}

impl Catalogue {
    /// Build from rows; ids must be unique and nonzero (they are the client's hash keys).
    pub fn new(rows: Vec<Activity>) -> io::Result<Self> {
        validate_rows(&rows)?;
        Ok(Self { rows })
    }

    /// Append soccer rows loaded from the soccer content file. Their ids share
    /// the portal namespace with battles, so a collision is refused.
    pub fn add_soccer_rows(&mut self, rows: Vec<Activity>) -> io::Result<()> {
        validate_rows(&rows)?;
        if self.rows.len() + rows.len() > MAX_ACTIVITIES {
            return Err(invalid("too many activities"));
        }
        if rows
            .iter()
            .any(|row| self.rows.iter().any(|other| other.id == row.id))
        {
            return Err(invalid("activity ids must be unique and nonzero"));
        }
        if rows.iter().any(|row| row.kind != Kind::Soccer) {
            return Err(invalid("soccer catalogue rows have the wrong kind"));
        }
        self.rows.extend(rows);
        Ok(())
    }

    /// True when `id` is a published soccer row (never a battle).
    pub fn is_soccer(&self, id: u32) -> bool {
        self.kind(id) == Some(Kind::Soccer)
    }

    /// Read `field_battles.json` (the same file the battle engine loads, so every row here is a
    /// battle `EncounterEngine::begin` accepts). Unknown fields are ignored.
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(path)?
            .take(MAX_FILE + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_FILE {
            return Err(invalid("field battles file too large"));
        }
        let file: BattleFile = serde_json::from_str(&text).map_err(|e| invalid(&e.to_string()))?;
        Self::new(
            file.battles
                .into_iter()
                .map(|b| Activity {
                    id: b.id,
                    name_id: b.title_id,
                    description_id: b.description_id,
                    // -1 is "no icon" in the battle data; the portal shows image set 0 then.
                    icon_id: u32::try_from(b.icon_id).unwrap_or(0),
                    difficulty: b.difficulty,
                    required_level: b.required_combat_level,
                    kind: Kind::Battle,
                })
                .collect(),
        )
    }

    pub fn activities(&self) -> &[Activity] {
        &self.rows
    }

    /// The kind behind an id, without admitting the player. Unknown ids are `None`.
    pub fn kind(&self, id: u32) -> Option<Kind> {
        self.rows.iter().find(|a| a.id == id).map(|a| a.kind)
    }

    /// Publish chess rows after the battle rows. A chess row is one
    /// single-player minigame row; the encounter engine never sees it.
    pub fn with_chess_rows(mut self, rows: Vec<ChessActivity>) -> io::Result<Self> {
        let mut ids: BTreeSet<u32> = self.rows.iter().map(|a| a.id).collect();
        for row in rows {
            if row.row_id == 0 || !ids.insert(row.row_id) {
                return Err(invalid("chess activity ids must be unique and nonzero"));
            }
            self.rows.push(Activity {
                id: row.row_id,
                name_id: row.name_id,
                description_id: row.description_id,
                icon_id: row.icon_id,
                difficulty: row.difficulty,
                required_level: 0,
                kind: Kind::Chess { row_id: row.row_id },
            });
        }
        if self.rows.len() > MAX_ACTIVITIES {
            return Err(invalid("too many activities"));
        }
        Ok(self)
    }

    /// Admission for a Play. Order matters: an unknown id reveals nothing about the player.
    pub fn admit(&self, id: u32, viewer: Viewer, busy: bool) -> Result<&Activity, Refusal> {
        let activity = self
            .rows
            .iter()
            .find(|a| a.id == id)
            .ok_or(Refusal::Unknown)?;
        if busy {
            return Err(Refusal::Busy);
        }
        match activity.kind {
            Kind::Battle => {
                if !viewer.combat_job {
                    return Err(Refusal::NotCombatJob);
                }
                if viewer.level < activity.required_level {
                    return Err(Refusal::LevelTooLow(activity.required_level));
                }
            }
            // (R) Chess needs no job or level; the table is open to every player.
            Kind::Chess { .. } => {}
            // (R) Soccer is a world activity, not a combat-job gate. Its row
            // still honors the configured level floor; the original gate is unknown.
            Kind::Soccer => {
                if viewer.level < activity.required_level {
                    return Err(Refusal::LevelTooLow(activity.required_level));
                }
            }
        }
        Ok(activity)
    }

    /// 167/1/1: the whole list for this player. Sent at world entry and on every 167/1/4.
    pub fn list_packet(&self, viewer: Viewer) -> Vec<u8> {
        let mut out = header(SUB_LIST);
        out.extend(ENTRY_WORLD_TUNNEL.to_le_bytes());
        out.extend((self.rows.len() as i32).to_le_bytes());
        for (position, a) in self.rows.iter().enumerate() {
            let (category, detail, thumbnail, can_join) = match a.kind {
                Kind::Battle => (
                    COMBAT_CATEGORY,
                    ART,
                    ART,
                    viewer.combat_job && viewer.level >= a.required_level,
                ),
                Kind::Chess { .. } => (CHESS_CATEGORY, CHESS_DETAIL, CHESS_THUMBNAIL, true),
                Kind::Soccer => (
                    SOCCER_CATEGORY,
                    SOCCER_ART,
                    SOCCER_ART,
                    viewer.level >= a.required_level,
                ),
            };
            for dword in [
                a.id,
                0, // AppSystemId
                category,
                a.icon_id,
                position as u32 + 1, // ActivityPositionId
                a.name_id,
                a.description_id,
                a.name_id,
                a.description_id,
                ENTRY_WORLD_TUNNEL,
            ] {
                out.extend(dword.to_le_bytes());
            }
            out.push(u8::from(can_join));
            out.extend(0u32.to_le_bytes()); // preferred state
            out.extend(0u32.to_le_bytes()); // featured windows
            out.extend(0u32.to_le_bytes()); // TutorialActivityId
            out.push(0); // MembersOnly
            for name in [detail, thumbnail] {
                out.extend((name.len() as i32).to_le_bytes());
                out.extend(name.as_bytes());
            }
            for dword in [a.difficulty, 0, 0, 0] {
                out.extend(dword.to_le_bytes()); // Difficulty, +0xc, +0x68, MysteryChestIcon
            }
        }
        out
    }
}

/// Decode a client message of family 167. `Ok(None)` for other families and for sub-ops the
/// client never sends here; `Err` for a Play or Refresh with the wrong length (forged).
pub fn parse(inner: &[u8]) -> io::Result<Option<Request>> {
    if inner.len() < 4 || inner[..2] != FAMILY.to_le_bytes() || inner[2] != SERVICE_ACTIVITY {
        return Ok(None);
    }
    match inner[3] {
        SUB_REFRESH if inner.len() == 4 => Ok(Some(Request::Refresh)),
        SUB_PLAY if inner.len() == PLAY_LEN => Ok(Some(Request::Play {
            // The trailing dword is the client clock offset; ignored on purpose.
            activity: u32::from_le_bytes(inner[4..8].try_into().unwrap()),
        })),
        SUB_REFRESH | SUB_PLAY => Err(invalid("malformed activity request")),
        _ => Ok(None),
    }
}

fn header(sub_op: u8) -> Vec<u8> {
    let mut out = FAMILY.to_le_bytes().to_vec();
    out.extend([SERVICE_ACTIVITY, sub_op]);
    out
}

fn validate_rows(rows: &[Activity]) -> io::Result<()> {
    if rows.len() > MAX_ACTIVITIES {
        return Err(invalid("too many activities"));
    }
    for (i, row) in rows.iter().enumerate() {
        if row.id == 0 || rows[..i].iter().any(|other| other.id == row.id) {
            return Err(invalid("activity ids must be unique and nonzero"));
        }
    }
    Ok(())
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}
