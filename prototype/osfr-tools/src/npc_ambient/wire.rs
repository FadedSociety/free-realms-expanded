//! T507/T552: the three ambient wire bodies and the update that owns them.

/// One due server->client update. `packets` owns the wire bodies so the byte
/// layout lives beside the schedule that produced it.
#[derive(Clone, Debug, PartialEq)]
pub enum AmbientUpdate {
    /// 35/8 SetAnimation with flags 0 (the slice-1 emote row).
    Animation { guid: u64, animation_id: i32 },
    /// 35/26 UpdateIdleAnim plus a 35/8 base animation (flags bit 0).
    Idle { guid: u64, animation_id: i32 },
    /// 35/8 SetAnimation for a conversation group speaker (flags 0).
    Conversation { guid: u64, animation_id: i32 },
    /// A due look-at; the gateway resolves the target from the player grid and
    /// sends 35/17 only to viewers that already know the target.
    LookAt {
        guid: u64,
        position: [f32; 3],
        radius: f32,
    },
}

impl AmbientUpdate {
    /// The application bodies to queue. `target` is used only by [`Self::LookAt`];
    /// `0` is the client's null GUID and clears the look-at (0x0095a460).
    pub fn packets(&self, target: u64) -> Vec<Vec<u8>> {
        match self {
            Self::Animation { guid, animation_id } | Self::Conversation { guid, animation_id } => {
                vec![crate::encounter::animation(*guid, *animation_id, false)]
            }
            Self::Idle { guid, animation_id } => vec![
                idle_animation(*guid, *animation_id),
                crate::encounter::animation(*guid, *animation_id, true),
            ],
            Self::LookAt { guid, .. } => vec![look_at(*guid, target)],
        }
    }
}

/// 35/26 UpdateIdleAnim. Client reader `0x008d77a0`: the `0x008d6830` header
/// (two i16), a u64 GUID and an i32 animation id, 16 bytes total.
pub fn idle_animation(guid: u64, animation_id: i32) -> Vec<u8> {
    let mut body = header(35, 26);
    body.extend(guid.to_le_bytes());
    body.extend(animation_id.to_le_bytes());
    body
}

/// 35/17 SetLookAt. Client reader `0x008d75d0`: the `0x008d6830` header, a u64
/// actor GUID and a u64 target GUID, 20 bytes total. The target pair is compared
/// with the null GUID at `0x0095a460`; target 0 takes the clear branch.
pub fn look_at(guid: u64, target: u64) -> Vec<u8> {
    let mut body = header(35, 17);
    body.extend(guid.to_le_bytes());
    body.extend(target.to_le_bytes());
    body
}

fn header(op: u16, sub: u16) -> Vec<u8> {
    [op.to_le_bytes(), sub.to_le_bytes()].concat()
}
