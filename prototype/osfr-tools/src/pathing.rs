//! Answer to the client's path request (98/1): the "Take Me There" breadcrumb trail.
//!
//! Evidence (FreeRealms.exe, `project/scripts/native_map/nmq.py`):
//! - Request: `RequestKynapsePath` `009cd010` builds `ClientPathRequestPacket` (ctor
//!   `008b7df0`, sub 1) and sends it with `BuildTunneledClientPacket` `0092a710`, whose
//!   writer `0091b8d0` fixes the layout: `u16 98`, `u8 1`, `u64` own actor handle
//!   (`0090b0f0`, `008fb810`), `u32 kind`, `u32 aux`, vec4 from, vec4 to (`0091a640`, four
//!   `f32` each), `u32` tail. 55 bytes. Kind 1 is the breadcrumb request (`009cd010`); kind
//!   2 is the point-of-interest teleport request (`009ce560`); this module answers kind 1
//!   only. The client asks only when the squared 4D distance is at most `01810054` =
//!   90000.0 (300 m), else it logs "too long. Not attempting" and sends nothing.
//! - Reply: dispatcher `009471a0` case 0x62 reads `i16 98`, `u8 sub` (`008f6a70`); sub 2 is
//!   the reply, read by `008faf30`: `u32 kind` (1 goes to `HandleKynapsePathReply`
//!   `009cd2f0`, 2 to `009cead0`), `u32 aux`, `i32 count`, `count` x vec4 (`008e7710`).
//! - No reply: the request leaves state 1 (`+0xd0c`); after 10 s `UpdateBreadcrumbPathState`
//!   `009cd410` clears the breadcrumb target. An empty (count 0) reply is the client's
//!   failure path: state 3, one immediate retry with the fallback flag, then it holds and
//!   asks again every 5 s once the player has moved 10 m. A non-empty reply replaces the
//!   route with the nodes (start to goal) and the client re-requests as the player moves.
//!
//! (R) Not recovered, chosen here: the reply echoes the request's `kind` and `aux`; the
//! path is a straight line sampled every [`STEP`] metres with ground heights from
//! [`Terrain`] (no obstacle avoidance); the request handle and tail are ignored; the
//! rate cap; "own zone" means both points are covered by the zone's height grid and
//! the start is within 300 m of the server's position for the player.
use crate::terrain::Terrain;

const FAMILY: u16 = 98;
const REQUEST: u8 = 1;
const REPLY: u8 = 2;
/// The breadcrumb request (`009cd010` writes 1); the POI request writes 2.
const KIND_BREADCRUMB: u32 = 1;
/// Bytes after the 3-byte header: handle 8, kind 4, aux 4, two vec4 32, tail 4.
const BODY: usize = 52;
/// The client's own limit: `01810054` = 90000.0 = 300 squared.
const MAX_RANGE_SQ: f32 = 90_000.0;
/// (R) Metres between samples. The client walks an effect along the polyline
/// (`009cbcf0`), so the shape matters more than the spacing.
pub const STEP: f32 = 5.0;
/// Hard cap on points per reply; 300 m / `STEP` + 1 = 61 never reaches it.
pub const MAX_POINTS: usize = 80;
/// (R) Answered requests per second per session. The client sends one every 5 s as the
/// player moves, plus one immediate retry after a failure.
pub const MAX_PER_SECOND: u32 = 4;
const WINDOW_MS: u64 = 1_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Malformed,
    NotFinite,
    TooFar,
    OutOfZone,
    TooFast,
}

/// Per-session request budget. Lives in `Gameplay`.
#[derive(Clone, Debug, Default)]
pub struct Pathing {
    window_start_ms: u64,
    used: u32,
}

struct Request {
    kind: u32,
    aux: u32,
    from: [f32; 4],
    to: [f32; 4],
}

fn parse(inner: &[u8]) -> Option<Request> {
    let body = inner.strip_prefix(&[FAMILY as u8, 0, REQUEST])?;
    let word = |at: usize| u32::from_le_bytes(body[at..at + 4].try_into().unwrap());
    let vec4 = |at: usize| std::array::from_fn(|i| f32::from_bits(word(at + 4 * i)));
    (body.len() == BODY).then(|| Request {
        kind: word(8),
        aux: word(12),
        from: vec4(16),
        to: vec4(32),
    })
}

fn reply(kind: u32, aux: u32, points: &[[f32; 4]]) -> Vec<u8> {
    let mut out = vec![FAMILY as u8, 0, REPLY];
    out.extend(kind.to_le_bytes());
    out.extend(aux.to_le_bytes());
    out.extend((points.len() as i32).to_le_bytes());
    out.extend(points.iter().flatten().flat_map(|v| v.to_le_bytes()));
    out
}

impl Pathing {
    /// `None`: not a 98/1 breadcrumb request, the caller carries on. `Some(Err)`: refused,
    /// send nothing. `Some(Ok)`: the 98/2 reply, a path or the client's empty failure reply.
    pub fn answer(
        &mut self,
        inner: &[u8],
        zone: u32,
        player: [f32; 3],
        terrain: Option<&Terrain>,
        now_ms: u64,
    ) -> Option<Result<Vec<u8>, Refusal>> {
        if !inner.starts_with(&[FAMILY as u8, 0, REQUEST]) {
            return None;
        }
        let Some(request) = parse(inner) else {
            return Some(Err(Refusal::Malformed));
        };
        if request.kind != KIND_BREADCRUMB {
            return None;
        }
        Some(self.breadcrumb(&request, zone, player, terrain, now_ms))
    }

    fn breadcrumb(
        &mut self,
        request: &Request,
        zone: u32,
        player: [f32; 3],
        terrain: Option<&Terrain>,
        now_ms: u64,
    ) -> Result<Vec<u8>, Refusal> {
        let (from, to) = (request.from, request.to);
        if !from.iter().chain(&to).all(|v| v.is_finite()) {
            return Err(Refusal::NotFinite);
        }
        let sq = |a: f32, b: f32| (a - b) * (a - b);
        // Same test as the client (4D, inclusive); `!(<=)` also refuses overflow to infinity.
        let reach: f32 = (0..4).map(|i| sq(from[i], to[i])).sum();
        if !(reach <= MAX_RANGE_SQ) {
            return Err(Refusal::TooFar);
        }
        if !(sq(from[0], player[0]) + sq(from[2], player[2]) <= MAX_RANGE_SQ) {
            return Err(Refusal::OutOfZone);
        }
        // Only an answered request spends budget: the gateway drops state for a silent
        // refusal, and silence costs nothing to send.
        if now_ms.saturating_sub(self.window_start_ms) >= WINDOW_MS {
            *self = Self {
                window_start_ms: now_ms,
                used: 0,
            };
        }
        if self.used >= MAX_PER_SECOND {
            return Err(Refusal::TooFast);
        }
        let Some(grid) = terrain.and_then(|t| t.grid(zone)) else {
            self.used += 1;
            return Ok(reply(request.kind, request.aux, &[]));
        };
        if grid.height_at(from[0], from[2]).is_none() || grid.height_at(to[0], to[2]).is_none() {
            return Err(Refusal::OutOfZone);
        }
        self.used += 1;
        let run = (sq(from[0], to[0]) + sq(from[2], to[2])).sqrt();
        let segments = ((run / STEP).ceil() as usize).clamp(1, MAX_POINTS - 1);
        let mut y = from[1];
        let points: Vec<[f32; 4]> = (0..=segments)
            .map(|i| {
                let t = i as f32 / segments as f32;
                let lerp = |k: usize| from[k] + (to[k] - from[k]) * t;
                // The ends are the client's own points; between them follow the ground,
                // holding the last height across a hole in the grid.
                y = match i {
                    0 => from[1],
                    _ if i == segments => to[1],
                    _ => grid.height_at(lerp(0), lerp(2)).unwrap_or(y),
                };
                [lerp(0), y, lerp(2), lerp(3)]
            })
            .collect();
        Ok(reply(request.kind, request.aux, &points))
    }
}
