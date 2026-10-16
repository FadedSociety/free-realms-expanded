//! Player radial menu and Inspect (W143, Unrouted Client Requests brief #13).
//!
//! Wire flow, per session:
//! 1. Client click on another player: `26/19` SelectPlayer (C>S), `u64` + `u64`.
//!    Native `00934870` (and `0093bff0`) build `CommandPacketSelectPlayer` (ctor
//!    `008b6af0`, sub 0x13) and send it through `00925ec0`, which serializes the
//!    two `u64` fields at object +0x10 and +0x18: the first comes from the local
//!    player object (+0x54 vslot 0x10), the second is the clicked actor's key
//!    (`param_1+8/+0xc`). The peer schema calls them `Guid` and `SelectGuid`.
//!    The first is never trusted here; identity comes from the session (R).
//! 2. Server answers `26/9` with one Inspect entry. Entry rows are seven i32
//!    (`00a9dca0`): id, icon, label string id, then four unused. The radial popup
//!    (`00a38570`) calls `HandlerRadialMenu:setButton(index, icon, text, eventId)`
//!    with entry+4, label(entry+8), entry+0 respectively.
//! 3. Click: Lua `Ui.RadialMenuClicked(eventId)` (registered at `0182dd90`, body
//!    `00c15050`) calls `SendInteractionSelectThenReset` (`00a39090`), which sends
//!    `26/10` = the menu guid stored by the 26/9 handler (`u64`) + the entry id
//!    (`i32`). The entry id is server content; the client has no Inspect constant.
//! 4. Server answers `149/1` StartInspect: `u16 149, u8 1, u64 guid, bool pedestal,
//!    u32 length + InspectProxy payload` (header reader `008d6310`, body reader
//!    `008d6370`, case 0x95 of `009471a0`, payload reader `00ad83b0`). The client
//!    looks the guid up among its actors first (`00984960`) and does nothing when
//!    the actor is absent, so the target must already have been AddPc'd to it.
//! 5. Closing the panel: `149/2` StopInspect (C>S), header only: `u16 149, u8 2`
//!    (`009347d0`, ctor `008a3bd0`). Nothing is sent back.
//!
//! Recovered facts: everything above plus the label string 3902 = "Inspect" in the
//! shipped en_us locale. Icon 133 and the seven-field row come from the peer
//! emulator schema (behaviour reference only, no code copied). Reconstructed
//! policy (R): the entry id, the reach limit, one-shot menus, what a stranger may
//! see (jobs, ranks and worn items only), refusals sending nothing at all.

use std::fmt;

use crate::players::{PlayerRegistry, SessionId};

pub const SELECT_PLAYER_LEN: usize = 20;
/// (R) Server-chosen radial entry id. 101-104 are used by creature menus.
pub const INSPECT_ENTRY_ID: i32 = 1001;
const INSPECT_ICON: i32 = 133; // peer schema; not checked against client assets.
const INSPECT_LABEL: i32 = 3902; // en_us locale "Inspect".
/// (R) Default selection reach in metres, tunable per instance.
pub const DEFAULT_REACH_M: f32 = 40.0;
const MAX_NAME_BYTES: usize = 64; // (R)
const MAX_PROFILES: usize = 64; // (R)
const MAX_SLOT: i32 = 17; // `00ad83b0` accepts slots 1..=17 only.
const MAX_RECORD_BYTES: usize = 64 * 1024; // (R)
const MAX_PAYLOAD_BYTES: usize = 512 * 1024; // (R)

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RadialError {
    Malformed,
    NotInWorld,
    SelfTarget,
    /// Unknown, never shown to this viewer, other zone or instance, or gone.
    NotVisible,
    TooFar,
    InvalidChoice,
    NoLook,
    BadLook(&'static str),
}

impl fmt::Display for RadialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => f.write_str("malformed player radial request"),
            Self::NotInWorld => f.write_str("viewer is not in the world registry"),
            Self::SelfTarget => f.write_str("player selected themselves"),
            Self::NotVisible => f.write_str("target is not visible to this viewer"),
            Self::TooFar => f.write_str("target is beyond selection reach"),
            Self::InvalidChoice => f.write_str("choice is not on the offered menu"),
            Self::NoLook => f.write_str("no public look for target"),
            Self::BadLook(why) => write!(f, "invalid public look: {why}"),
        }
    }
}

impl std::error::Error for RadialError {}

/// A worn item of the target's active profile. `definition_record` is the
/// serialized `ClientItemDefinition` row, the same bytes as in a 35/37 batch.
#[derive(Clone, Debug, PartialEq)]
pub struct PublicItem {
    pub slot: i32,
    pub definition_id: i32,
    pub tint: i32,
    pub definition_record: Vec<u8>,
}

/// Everything a stranger may see. There is deliberately no field for coins,
/// quests, collections, stats or titles: those go out as zero (R).
#[derive(Clone, Debug, PartialEq)]
pub struct PublicLook {
    /// (job id, rank)
    pub profiles: Vec<(i32, i32)>,
    pub active_profile: i32,
    pub items: Vec<PublicItem>,
}

/// Per-session radial state: at most one open menu and one open inspect.
#[derive(Clone, Debug)]
pub struct PlayerRadial {
    reach_m: f32,
    menu_target: Option<u64>,
    inspecting: Option<u64>,
}

impl Default for PlayerRadial {
    fn default() -> Self {
        Self {
            reach_m: DEFAULT_REACH_M,
            menu_target: None,
            inspecting: None,
        }
    }
}

impl PlayerRadial {
    /// `None` when the reach is not a finite, positive number.
    pub fn with_reach(reach_m: f32) -> Option<Self> {
        (reach_m.is_finite() && reach_m > 0.0).then(|| Self {
            reach_m,
            ..Self::default()
        })
    }

    pub fn menu_target(&self) -> Option<u64> {
        self.menu_target
    }

    pub fn inspecting(&self) -> Option<u64> {
        self.inspecting
    }

    /// Call on zoning and when the target despawns.
    pub fn reset(&mut self) {
        self.menu_target = None;
        self.inspecting = None;
    }

    /// 26/19 -> 26/9. Refusals return an error and send nothing, so a probe
    /// cannot tell an unknown id from a hidden one.
    pub fn select_player(
        &mut self,
        registry: &PlayerRegistry,
        viewer: SessionId,
        inner: &[u8],
        has_look: impl FnOnce(u64) -> bool,
    ) -> Result<Vec<Vec<u8>>, RadialError> {
        self.menu_target = None;
        if inner.len() != SELECT_PLAYER_LEN || inner[..4] != [26, 0, 19, 0] {
            return Err(RadialError::Malformed);
        }
        // inner[4..12] is the client's own guid: ignored on purpose.
        let target = u64::from_le_bytes(inner[12..20].try_into().unwrap());
        let name = self.visible_name(registry, viewer, target)?;
        // Only offer an action that can work: no public look, no Inspect entry.
        if !has_look(target) {
            return Err(RadialError::NoLook);
        }
        self.menu_target = Some(target);
        Ok(vec![menu_packet(target, &name)])
    }

    /// 26/10. `Ok(None)` means "not a player menu choice": hand it to the NPC
    /// path unchanged. A matching guid always consumes the menu (one-shot).
    pub fn choose(
        &mut self,
        registry: &PlayerRegistry,
        viewer: SessionId,
        inner: &[u8],
        look_of: impl FnOnce(u64) -> Option<PublicLook>,
    ) -> Result<Option<Vec<Vec<u8>>>, RadialError> {
        if inner.len() != 16 || inner[..4] != [26, 0, 10, 0] {
            return Ok(None);
        }
        let guid = u64::from_le_bytes(inner[4..12].try_into().unwrap());
        if self.menu_target != Some(guid) {
            return Ok(None);
        }
        self.menu_target = None;
        let id = i32::from_le_bytes(inner[12..16].try_into().unwrap());
        if id != INSPECT_ENTRY_ID {
            return Err(RadialError::InvalidChoice);
        }
        self.visible_name(registry, viewer, guid)?; // stale or moved away since the menu
        let look = look_of(guid).ok_or(RadialError::NoLook)?;
        let packet = start_inspect_packet(guid, &look)?;
        self.inspecting = Some(guid);
        Ok(Some(vec![packet]))
    }

    /// 149/2. Idempotent; no reply.
    pub fn stop_inspect(&mut self, inner: &[u8]) -> Result<(), RadialError> {
        if inner != [149, 0, 2] {
            return Err(RadialError::Malformed);
        }
        self.inspecting = None;
        Ok(())
    }

    /// Display name of a target this viewer may select right now.
    fn visible_name(
        &self,
        registry: &PlayerRegistry,
        viewer: SessionId,
        target: u64,
    ) -> Result<String, RadialError> {
        let me = registry.get(viewer).ok_or(RadialError::NotInWorld)?;
        if me.character_guid == target {
            return Err(RadialError::SelfTarget);
        }
        // Only a player this viewer was sent an AddPc for (same zone instance,
        // inside the interest radius) can be named; ids are never guessable.
        if !registry.knows_character(viewer, target) {
            return Err(RadialError::NotVisible);
        }
        let other = registry
            .nearby_sessions(me.zone, me.position)
            .into_iter()
            .filter_map(|id| registry.get(id))
            .find(|p| p.character_guid == target)
            .ok_or(RadialError::NotVisible)?;
        let reach = f64::from(self.reach_m);
        let distance_squared: f64 = (0..3)
            .map(|i| (f64::from(me.position[i]) - f64::from(other.position[i])).powi(2))
            .sum();
        if distance_squared > reach * reach {
            return Err(RadialError::TooFar);
        }
        let p = &other.presentation;
        let full = [p.first_name.as_str(), p.last_name.as_str()]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
        Ok(truncate(&full, MAX_NAME_BYTES).to_string())
    }
}

fn truncate(text: &str, max: usize) -> &str {
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// 26/9 body: guid, auto-select bool, entry count, seven-i32 rows, name,
/// two trailing bools (`00aa2220`; same shape as the proven NPC menus).
fn menu_packet(guid: u64, name: &str) -> Vec<u8> {
    let mut out = vec![26, 0, 9, 0];
    out.extend_from_slice(&guid.to_le_bytes());
    out.push(0);
    put_i32(&mut out, 1);
    for value in [INSPECT_ENTRY_ID, INSPECT_ICON, INSPECT_LABEL, 0, 0, 0, 0] {
        put_i32(&mut out, value);
    }
    put_i32(&mut out, name.len() as i32);
    out.extend_from_slice(name.as_bytes());
    out.extend_from_slice(&[1, 0]);
    out
}

/// 149/1 with the InspectProxy payload in `00ad83b0` read order.
fn start_inspect_packet(guid: u64, look: &PublicLook) -> Result<Vec<u8>, RadialError> {
    if look.profiles.len() > MAX_PROFILES {
        return Err(RadialError::BadLook("too many profiles"));
    }
    if look.items.len() > MAX_SLOT as usize {
        return Err(RadialError::BadLook("too many items"));
    }
    let mut payload = Vec::new();
    put_i32(&mut payload, look.profiles.len() as i32);
    let mut levels: i32 = 0;
    for &(id, rank) in &look.profiles {
        put_i32(&mut payload, id);
        put_i32(&mut payload, rank);
        levels = levels.saturating_add(rank.saturating_sub(1).max(0));
    }
    put_i32(&mut payload, look.active_profile);
    put_i32(&mut payload, look.items.len() as i32);
    let mut seen = 0u32;
    for item in &look.items {
        if !(1..=MAX_SLOT).contains(&item.slot) || seen & (1 << item.slot) != 0 {
            return Err(RadialError::BadLook("bad or repeated slot"));
        }
        seen |= 1 << item.slot;
        let record = &item.definition_record;
        if record.len() < 4
            || record.len() > MAX_RECORD_BYTES
            || record[..4] != item.definition_id.to_le_bytes()
        {
            return Err(RadialError::BadLook("definition record does not match id"));
        }
        put_i32(&mut payload, item.slot);
        put_i32(&mut payload, item.definition_id);
        put_i32(&mut payload, item.tint);
        payload.extend_from_slice(record);
    }
    put_i32(&mut payload, 0); // achievements (R: private)
    put_i32(&mut payload, 0); // achievement points (R: private)
    put_i32(&mut payload, 0); // VIP rank, f32 0.0 (R)
    put_i32(&mut payload, 0); // VIP icon
    put_i32(&mut payload, 0); // VIP title
    put_i32(&mut payload, 0); // active title
    put_i32(&mut payload, 0); // coins (R: private)
    put_i32(&mut payload, levels);
    put_i32(&mut payload, 0); // collections (R: private)
    put_i32(&mut payload, 0); // quests (R: private)
    put_i32(&mut payload, 0); // stats: none (R)
    put_i32(&mut payload, 0); // trailing string, empty
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err(RadialError::BadLook("payload too large"));
    }
    let mut out = vec![149, 0, 1];
    out.extend_from_slice(&guid.to_le_bytes());
    out.push(1); // show pedestal
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}
