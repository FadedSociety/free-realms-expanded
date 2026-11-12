//! T879: the combat marker an overworld enemy or battle spawner shows on the minimap, a 35/10
//! notification of kind 3 ("Combat") sent right after the actor's 35/2. A battle spawner sends
//! the full record (named on-screen label); a fighting enemy the compact one, as live did
//! (`enemy`). The row is content: `field_enemies.json` and `field_battles.json` each carry a
//! shared `combat_marker`, and an actor may override it with its own row or switch it off.
//!
//! Client rules (35/10 handler `0093213a`-`009321d1`, minimap image rule `01025670`): the guid
//! must already be spawned, the marker dies with the actor on the client (so a despawn needs no
//! 35/11), and a record whose `image_id` does not resolve to a minimap image is dropped whole.
use super::*;
use crate::npc_notifications::{self, Notification};

/// An actor's `combat_marker` content value: absent follows the file's shared row, `"off"` sends
/// none, an object is this actor's own row.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(untagged)]
pub(super) enum MarkerSetting {
    #[default]
    Shared,
    Off(Off),
    Row(Notification),
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Off {
    Off,
}

impl MarkerSetting {
    /// The row this actor sends, bounded by [`Notification::validate`]. A row whose `name_id` is
    /// 0 takes the actor's own name id; an actor with no name id has nothing to label, so it
    /// sends none (the compact record, which draws no name, is not wired).
    pub(super) fn resolve(
        self,
        shared: Option<Notification>,
        own_name_id: i32,
    ) -> io::Result<Option<Notification>> {
        let row = match self {
            Self::Shared => shared,
            Self::Off(_) => None,
            Self::Row(row) => Some(row),
        };
        let Some(row) = row else { return Ok(None) };
        if let Some(why) = row.validate() {
            return Err(invalid(why));
        }
        let name_id = if row.name_id == 0 {
            own_name_id
        } else {
            row.name_id
        };
        Ok((name_id > 0).then_some(Notification { name_id, ..row }))
    }
}

/// The 35/10 that follows a battle starter's 35/2 when it has a marker: the full record, named
/// label and hover panel included.
pub(super) fn packet(guid: u64, marker: Option<Notification>) -> Option<Vec<u8>> {
    npc_notifications::encode_add(&[(guid, marker?)])
}

/// The 35/10 that follows an open-world enemy's 35/2: the compact record (u64 guid, u8 1, i32
/// kind, u8), which the client draws as the nameless `ind_enemy` minimap dot with no hover
/// panel. Live sent exactly that for every fighting actor (Bixie Soldiers and Battlemages,
/// Robgoblin Kaboomers, Skullz, the drakes; capture p12 and packets_3) and kept the full
/// named record for the battle starters; the full record on a field enemy showed the starter's
/// label and panel (owner, rc-0136: the Kaboomers "show the interact ui for instance enemies").
pub(super) fn enemy(guid: u64, marker: Option<Notification>) -> Option<Vec<u8>> {
    let kind = marker?.kind;
    let mut b = vec![35, 0, 10, 0];
    b.extend(1_i32.to_le_bytes());
    b.extend(guid.to_le_bytes());
    b.push(1); // compact
    b.extend(kind.to_le_bytes());
    b.push(1); // +41, never read; Sanctuary sends 1
    Some(b)
}
