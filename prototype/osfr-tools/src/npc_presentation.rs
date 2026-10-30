//! T899: what an NPC's name id says about how it looks and answers a hover (`npc_presentation.json`):
//! the hover cursor (35/12), the shop or combat marker rows (35/10), the line under the name and
//! the AddNpc disposition. One row per name string id, applied when `world.rs` loads an NPC row
//! or an extra spawn; a value the NPC row sets itself wins.
//!
//! Adapted from Sanctuary (AGPL-3.0-or-later), `src/Resources/Npcs.json` at main 3b3aec3
//! (`CursorId`, `RelevanceUnknown2`, `SubTextNameId`, `Disposition`, `Notifications`) and the
//! entry layout of its `PlayerUpdatePacketNpcRelevance`. The rows are data; this file is our
//! loader and encoder.
//!
//! Client reader for 35/12 (dispatch case 12, `0092f460`): an i32 count, then per entry `008db5e0`
//! reads u64 guid, u8 has-cursor, a signed byte cursor id only when that flag is set, and a u8
//! flag. `00962f30` stores the id (or -1 without the flag) at actor `+0x708` and the flag at bit
//! `0x10` of `+0x145`. Cursor ids are `Resources/Cursors.txt` rows (5 talk, 11 fight, 17 use).
//! What the trailing flag means is not recovered (R); rows carry the captured value.

use crate::{
    npc_notifications::{self, Notification},
    startup_world::NpcSpawn,
};
use serde::Deserialize;
use std::collections::HashMap;

/// The content file beside `npc_spawns_extra.json`.
pub const CONTENT_FILE: &str = "npc_presentation.json";
/// The client reads the cursor id as a signed byte.
const MAX_CURSOR_ID: u8 = 127;
/// (R) AddNpc dispositions in use are 0 (hostile), 1 (friendly) and 2 (companion); a typo guard.
const MAX_DISPOSITION: i32 = 2;

/// One content row. Every field but the key is optional: an absent field leaves the NPC alone.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// The NPC's `name_string_id` (Sanctuary `NameId`).
    pub name_string_id: u32,
    /// For readers of the file only.
    #[serde(default)]
    pub name: Option<String>,
    /// The hover cursor (35/12); absent sends no entry.
    #[serde(default)]
    pub cursor_id: Option<u8>,
    /// The trailing byte of the 35/12 entry (Sanctuary `RelevanceUnknown2`).
    #[serde(default)]
    pub relevance_flag: bool,
    /// AddNpc `sub_text_name_string_id`: the locale id of the line under the name.
    #[serde(default)]
    pub sub_text_name_string_id: Option<i32>,
    /// AddNpc `disposition`; 0 draws the name red.
    #[serde(default)]
    pub disposition: Option<i32>,
    /// The 35/10 rows this NPC shows (shop, combat, minigame markers).
    #[serde(default)]
    pub notifications: Vec<Notification>,
}

/// A 35/12 entry's cursor: the `Cursors.txt` id and the trailing flag byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub id: u8,
    pub flag: bool,
}

/// The validated rows, by name string id.
#[derive(Clone, Debug, Default)]
pub struct Presentation {
    by_name: HashMap<u32, Record>,
}

impl Record {
    fn validate(&self) -> Option<&'static str> {
        if self.cursor_id.is_some_and(|id| id > MAX_CURSOR_ID) {
            Some("cursor id is above the client's signed byte")
        } else if self.relevance_flag && self.cursor_id.is_none() {
            Some("relevance flag without a cursor id")
        } else if self.sub_text_name_string_id.is_some_and(|id| id < 1) {
            Some("sub text id is not positive")
        } else if self
            .disposition
            .is_some_and(|value| !(0..=MAX_DISPOSITION).contains(&value))
        {
            Some("disposition out of bounds")
        } else if self.notifications.len() > npc_notifications::MAX_ENTRIES {
            Some("too many notifications")
        } else {
            self.notifications.iter().find_map(Notification::validate)
        }
    }
}

impl Presentation {
    /// Validates `records`; a bad or repeated row refuses the whole table.
    pub fn new(records: Vec<Record>) -> Result<Self, String> {
        let mut by_name = HashMap::with_capacity(records.len());
        for record in records {
            if let Some(reason) = record.validate() {
                return Err(format!("name id {}: {reason}", record.name_string_id));
            }
            let id = record.name_string_id;
            if by_name.insert(id, record).is_some() {
                return Err(format!("name id {id}: repeated row"));
            }
        }
        Ok(Self { by_name })
    }

    /// Fills `spawn` and `notifications` from the row for the spawn's name id and returns the
    /// hover cursor. A line under the name or notifications the NPC row already has are kept.
    pub fn apply(
        &self,
        spawn: &mut NpcSpawn,
        notifications: &mut Vec<Notification>,
    ) -> Option<Cursor> {
        let row = self.by_name.get(&(spawn.name_string_id as u32))?;
        if let Some(sub_text) = row
            .sub_text_name_string_id
            .filter(|_| spawn.sub_text_name_string_id == -1)
        {
            spawn.sub_text_name_string_id = sub_text;
        }
        if let Some(disposition) = row.disposition {
            spawn.disposition = disposition;
        }
        if notifications.is_empty() {
            notifications.clone_from(&row.notifications);
        }
        row.cursor_id.map(|id| Cursor {
            id,
            flag: row.relevance_flag,
        })
    }
}

/// 35/12 for `(runtime actor guid, cursor)` entries, each with the has-cursor flag set.
pub fn encode_cursors(entries: &[(u64, Cursor)]) -> Vec<u8> {
    let mut bytes = vec![35, 0, 12, 0];
    bytes.extend((entries.len() as i32).to_le_bytes());
    for (guid, cursor) in entries {
        bytes.extend(guid.to_le_bytes());
        bytes.extend([1, cursor.id, u8::from(cursor.flag)]);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::content_spawn_template;

    fn row(json: &str) -> Record {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn the_wire_is_the_client_reader_layout() {
        // 008db5e0: u64 guid, u8 has-cursor, i8 cursor id, u8 flag; count first (dispatch 0092f460).
        let bytes = encode_cursors(&[
            (
                0x0102_0304_0506_0708,
                Cursor {
                    id: 17,
                    flag: false,
                },
            ),
            (9, Cursor { id: 11, flag: true }),
        ]);
        let mut expected = vec![35, 0, 12, 0, 2, 0, 0, 0];
        expected.extend([8, 7, 6, 5, 4, 3, 2, 1, 1, 17, 0]);
        expected.extend([9, 0, 0, 0, 0, 0, 0, 0, 1, 11, 1]);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn a_row_fills_what_the_npc_leaves_empty_and_never_overwrites() {
        let table = Presentation::new(vec![row(
            r#"{"name_string_id": 7, "cursor_id": 17, "sub_text_name_string_id": 36, "disposition": 0,
                "notifications": [{"kind": 1, "image_id": 12, "state": 1, "name_id": 7, "subtext_id": 36}]}"#,
        )])
        .unwrap();
        let mut spawn = NpcSpawn {
            name_string_id: 7,
            ..content_spawn_template()
        };
        let mut shown = Vec::new();
        let cursor = table.apply(&mut spawn, &mut shown);
        assert_eq!(
            cursor,
            Some(Cursor {
                id: 17,
                flag: false
            })
        );
        assert_eq!(spawn.sub_text_name_string_id, 36);
        assert_eq!(spawn.disposition, 0);
        assert_eq!(shown.len(), 1);

        // The NPC's own line and notifications win; another name id is untouched.
        let own = Notification {
            kind: 6,
            image_id: 3,
            state: 3,
            name_id: 7,
            subtext_id: 0,
            hide_overhead: false,
            enabled: true,
        };
        let mut spawn = NpcSpawn {
            name_string_id: 7,
            sub_text_name_string_id: 99,
            ..content_spawn_template()
        };
        let mut shown = vec![own];
        table.apply(&mut spawn, &mut shown);
        assert_eq!((spawn.sub_text_name_string_id, shown), (99, vec![own]));
        let mut other = NpcSpawn {
            name_string_id: 8,
            ..content_spawn_template()
        };
        assert_eq!(table.apply(&mut other, &mut Vec::new()), None);
        assert_eq!(other.disposition, content_spawn_template().disposition);
    }

    #[test]
    fn a_bad_or_repeated_row_refuses_the_table() {
        for bad in [
            r#"{"name_string_id": 1, "cursor_id": 200}"#,
            r#"{"name_string_id": 1, "relevance_flag": true}"#,
            r#"{"name_string_id": 1, "sub_text_name_string_id": 0}"#,
            r#"{"name_string_id": 1, "disposition": 5}"#,
            r#"{"name_string_id": 1, "notifications": [{"image_id": 0, "name_id": 1}]}"#,
        ] {
            assert!(Presentation::new(vec![row(bad)]).is_err(), "{bad}");
        }
        let twice = r#"{"name_string_id": 1, "cursor_id": 5}"#;
        assert!(Presentation::new(vec![row(twice), row(twice)]).is_err());
        assert!(Presentation::new(vec![row(twice)]).is_ok());
    }
}
