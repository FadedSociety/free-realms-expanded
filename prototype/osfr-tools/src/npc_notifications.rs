//! Actor notifications: the 35/10 `PlayerUpdatePacketAddNotifications` packet and the content row
//! behind it (a minimap indicator and an overhead icon for an actor). Quest markers
//! (`quest_markers.rs`) and static actors (warpstones, veins) share this one encoder.
//!
//! Client readers (35 dispatch switch `0092F540`, case 10 at `009320ac`): ctor `008d06d0`, body
//! `009213e0` -> `0090aa90` (an i32 count, refused when negative or larger than the bytes left),
//! then one `008db310` element per notification, in this order (offset in the client record):
//! u64 guid (+0), u8 compact (+40), i32 kind (+28), then only when compact is 0: i32 (+8), i32
//! (+16), i32 (+12), i32 (+20), i32 (+24), u8 (+32), i32 (+36); then u8 (+41), read for both.
//! The handler loop at `00932100` looks the guid up in the actor table (`00984960`) and skips a
//! guid the client has not spawned, so a caller sends the actor's 35/2 first.
//!
//! What the client does with each field (the handler, `0093213a`-`009321d1`):
//! - `image_id` (+16): the `NotificationImages.txt` `#ID` key. At 0 or less the whole
//!   notification is skipped; otherwise `01025670` resolves it to the minimap image and
//!   `00a91800` places the indicator with `kind`, `name_id` (+20) and `subtext_id` (+24), both
//!   locale ids; the overhead icon (`00960170`) takes `image_id`, `state` (+8) and the type
//!   string for `kind`, unless `hide_overhead` (+32) is set.
//! - `+12` and `+36` are only stored on the actor (`+0x718`) and handed to `0096d2e0`; they are
//!   0 on every row we send, and `+41` is never read by the handler. (R) naming for those.
//!
//! Sanctuary's rows name the first value after `kind` `IconId` and the second `IconState`; the
//! client keys the icon on the second, so the content rows put the icon in both slots (R).

use serde::Deserialize;

/// (R) One packet never carries more entries; the encoder refuses instead of truncating the count.
pub const MAX_ENTRIES: usize = 256;
/// `NotificationImages.txt` keys are small; this bounds a typo, not the table.
const MAX_IMAGE_ID: i32 = 10_000;
/// (R) Sanctuary's rows use kinds 0 to 6; the bound is a typo guard.
const MAX_KIND: i32 = 255;

/// One notification an actor shows. Values live in content (`notifications` on a spawn row or a
/// node type); [`Self::validate`] bounds them at load.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    /// The notification type (+28): picks the type string and is passed to the minimap.
    #[serde(default)]
    pub kind: i32,
    /// `NotificationImages.txt` key (+16); 1 or more, or the client skips the row.
    pub image_id: i32,
    /// Overhead icon argument (+8), 0 or more.
    #[serde(default)]
    pub state: i32,
    /// Locale id of the indicator's name (+20).
    pub name_id: i32,
    /// Locale id of the indicator's sub-text (+24); 0 or -1 for none.
    #[serde(default)]
    pub subtext_id: i32,
    /// Skip the overhead icon and show the minimap indicator only (+32).
    #[serde(default)]
    pub hide_overhead: bool,
    /// Trailing byte (+41); the handler never reads it, Sanctuary sends 1 on every row.
    #[serde(default = "enabled")]
    pub enabled: bool,
}

fn enabled() -> bool {
    true
}

impl Notification {
    /// Why a content row is refused, if it is.
    pub fn validate(&self) -> Option<&'static str> {
        if !(1..=MAX_IMAGE_ID).contains(&self.image_id) {
            Some("notification image id must be 1 to the local bound")
        } else if !(0..=MAX_KIND).contains(&self.kind) || self.state < 0 || self.name_id < 0 {
            Some("notification kind, state or name id out of bounds")
        } else if self.subtext_id < -1 {
            Some("notification sub-text id is below -1")
        } else {
            None
        }
    }

    fn write(&self, guid: u64, bytes: &mut Vec<u8>) {
        let word = |bytes: &mut Vec<u8>, value: i32| bytes.extend(value.to_le_bytes());
        bytes.extend(guid.to_le_bytes());
        bytes.push(0); // compact: the full record
        word(bytes, self.kind);
        word(bytes, self.state);
        word(bytes, self.image_id);
        word(bytes, 0); // +12
        word(bytes, self.name_id);
        word(bytes, self.subtext_id);
        bytes.push(u8::from(self.hide_overhead));
        word(bytes, 0); // +36
        bytes.push(u8::from(self.enabled));
    }
}

/// 35/10 for `(runtime actor guid, notification)` rows; `None` for more than [`MAX_ENTRIES`].
pub fn encode_add(entries: &[(u64, Notification)]) -> Option<Vec<u8>> {
    if entries.len() > MAX_ENTRIES {
        return None;
    }
    let mut bytes = vec![35, 0, 10, 0];
    bytes.extend((entries.len() as i32).to_le_bytes());
    for (guid, notification) in entries {
        notification.write(*guid, &mut bytes);
    }
    Some(bytes)
}
