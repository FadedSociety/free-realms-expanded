//! T879: the combat marker an overworld enemy or battle spawner shows on the minimap and as an
//! on-screen label: a full 35/10 notification of kind 3 ("Combat"), sent right after the
//! actor's 35/2. The row is content: `field_enemies.json` and `field_battles.json` each carry a
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

/// The 35/10 that follows `guid`'s 35/2 when it has a marker.
pub(super) fn packet(guid: u64, marker: Option<Notification>) -> Option<Vec<u8>> {
    npc_notifications::encode_add(&[(guid, marker?)])
}
