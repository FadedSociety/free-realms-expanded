//! T900: the area-enter banner. A player who stays in a new named area for the settle time
//! gets 104 `PacketPOIChangeMessage` (the banner) and then 15/2 `ChatPacketEnterArea`
//! ("Now entering ...").
//!
//! Adapted from Sanctuary (AGPL-3.0, GroaxyVRC/Sanctuary, main 3b3aec3), `Player.cs`
//! `QueuePointOfInterest`, `UpdatePendingPointOfInterest` and `UpdatePointOfInterest`: queue
//! the area under the player, restart the wait when it changes, drop it when the player is
//! back in the announced area, announce after the settle time. Here the area comes from
//! the shared [`RegionTable`] instead of Sanctuary's tile grid, and the state is kept per
//! session in [`AreaBanners`] (never saved: a reconnect announces the area once).
//!
//! Client wire, 104: dispatcher `0x009471a0` case 104 (`0x009489a7`) builds the packet
//! (ctor `0x0089b9f0`), reads it with `0x008d5dd0` (`i16` opcode, then `u32` name id, `u32`
//! POI id, `u32` area id), resolves the name id in the locale and calls `0x00a12a80` ->
//! `0x00c8e440`, which stores the three `CurrentPOI` columns and notifies Lua only when the
//! POI id or area id differs from the stored pair. `BaseClient_CurrentPOI_OnDataUpdate`
//! (`ScriptsBase.lua:5049`) plays `NotificationHandler:LoadingZone` for `1 < poiId < 11`, the
//! nine region movies. So the POI id is the region id (1, the default region, shows no banner
//! and resets the client's stored pair), and the area id stays 0 (R: its meaning is
//! unrecovered, and a per-place value would replay the banner inside one region).

use crate::world_regions::RegionTable;
use std::collections::BTreeMap;

/// 104 `PacketPOIChangeMessage`, client ctor `0x0089b9f0`.
const POI_CHANGE: u16 = 104;

/// 104 wire: `u16` opcode, `u32` name id, `u32` POI id, `u32` area id (14 bytes).
pub fn poi_change(name_string_id: u32, region_id: u32, area_id: u32) -> Vec<u8> {
    let mut packet = POI_CHANGE.to_le_bytes().to_vec();
    packet.extend(name_string_id.to_le_bytes());
    packet.extend(region_id.to_le_bytes());
    packet.extend(area_id.to_le_bytes());
    packet
}

/// A named area: `(region id, name string id)` from [`RegionTable::named_area_at`].
type Area = (u32, u32);

/// One session's banner state.
#[derive(Clone, Debug, Default)]
pub struct AreaBanner {
    /// The area the client was last told about.
    announced: Option<Area>,
    /// The area the player stands in and the time they entered it, until it settles.
    pending: Option<(Area, u64)>,
    /// Zone and position of the last lookup: a still player costs no lookup.
    seen: Option<(u32, [f32; 3])>,
}

impl AreaBanner {
    /// Looks at where the player stands and returns the packets due now (empty on almost
    /// every call). `now_ms` is any monotonic clock.
    fn poll(
        &mut self,
        table: &RegionTable,
        zone: u32,
        position: [f32; 3],
        now_ms: u64,
    ) -> Vec<Vec<u8>> {
        if self.seen != Some((zone, position)) {
            self.seen = Some((zone, position));
            let here = table.named_area_at(zone, position[0], position[2]);
            // Outside every region keeps what was announced and waits for the next area.
            match here {
                Some(area) if Some(area) != self.announced => {
                    if self.pending.map(|(pending, _)| pending) != Some(area) {
                        self.pending = Some((area, now_ms));
                    }
                }
                _ => self.pending = None,
            }
        }
        let Some((area, since)) = self.pending else {
            return Vec::new();
        };
        if now_ms.saturating_sub(since) < u64::from(table.banner_settle_ms()) {
            return Vec::new();
        }
        self.pending = None;
        self.announced = Some(area);
        vec![
            poi_change(area.1, area.0, 0),
            crate::chat::enter_area(area.1),
        ]
    }
}

/// Every session's banner state, keyed by session id.
#[derive(Clone, Debug, Default)]
pub struct AreaBanners(BTreeMap<u64, AreaBanner>);

impl AreaBanners {
    /// The packets `session` is due. `view` is the region table and the session's zone and
    /// position while it is in the open world; `None` (loading, instance, gone) forgets the
    /// session, so coming back announces the area again.
    pub fn poll(
        &mut self,
        session: u64,
        view: Option<(&RegionTable, u32, [f32; 3])>,
        now_ms: u64,
    ) -> Vec<Vec<u8>> {
        let Some((table, zone, position)) = view else {
            self.0.remove(&session);
            return Vec::new();
        };
        self.0
            .entry(session)
            .or_default()
            .poll(table, zone, position, now_ms)
    }

    /// Drops the state of sessions that no longer exist (they were never told to leave).
    /// Runs the scan only when more sessions hold state than `sessions` exist.
    pub fn prune(&mut self, sessions: usize, live: impl Fn(u64) -> bool) {
        if self.0.len() > sessions {
            self.0.retain(|session, _| live(*session));
        }
    }
}
