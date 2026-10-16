//! Builders for startup packets whose client-facing layouts are currently
//! understood well enough to serialize without consulting captured packet
//! bytes. This is a deliberately partial foundation; see the research note
//! before wiring it into the gateway.

const TUNNEL_PREFIX: [u8; 3] = [5, 0, 1];

// (R) unknown zone field, value from capture.
const ZONE_UNKNOWN_FLAG_AFTER_TUTORIAL: u8 = 0;
// (R) unknown zone field, value from capture.
const ZONE_UNKNOWN_FLAG_AFTER_SKY: u8 = 0;
// (R) unknown zone field, value from capture.
const ZONE_UNKNOWN_TRAILING_FLAG_A: u8 = 0;
// (R) unknown zone field, value from capture.
const ZONE_UNKNOWN_TRAILING_FLAG_B: u8 = 0;
// (R) unknown preload-marker field, value from capture.
const PRELOAD_MARKER_UNKNOWN_BYTE: u8 = 0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZoneDetails<'a> {
    pub name: &'a str,
    pub zone_type: i32,
    pub tutorial: bool,
    pub sky: &'a str,
    pub zone_id: u32,
    pub geometry_id: u32,
}

fn tunnel(inner: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(TUNNEL_PREFIX.len() + 4 + inner.len());
    out.extend_from_slice(&TUNNEL_PREFIX);
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn counted_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

/// ClientGateway opcode 2. `result` is the one-byte login result field.
pub fn build_login_reply(result: u8) -> Vec<u8> {
    tunnel(&[2, 0, result])
}

/// Opcode 166 carries a NUL-terminated environment string.
pub fn build_initialization_parameters(environment: &str) -> Vec<u8> {
    assert!(
        !environment.as_bytes().contains(&0),
        "environment contains NUL"
    );
    let mut inner = 166u16.to_le_bytes().to_vec();
    inner.extend_from_slice(environment.as_bytes());
    inner.push(0);
    tunnel(&inner)
}

/// PacketSendZoneDetails (opcode 43), using the field order walked by the
/// existing gateway parser and serializer. Four uninterpreted booleans remain
/// fixed to their captured values until a client reader assigns meaning.
pub fn build_zone_details(zone: &ZoneDetails<'_>) -> Vec<u8> {
    let mut inner = 43u16.to_le_bytes().to_vec();
    counted_string(&mut inner, zone.name);
    inner.extend_from_slice(&zone.zone_type.to_le_bytes());
    inner.push(u8::from(zone.tutorial));
    inner.push(ZONE_UNKNOWN_FLAG_AFTER_TUTORIAL);
    counted_string(&mut inner, zone.sky);
    inner.push(ZONE_UNKNOWN_FLAG_AFTER_SKY);
    inner.extend_from_slice(&zone.zone_id.to_le_bytes());
    inner.extend_from_slice(&zone.geometry_id.to_le_bytes());
    inner.push(ZONE_UNKNOWN_TRAILING_FLAG_A);
    inner.push(ZONE_UNKNOWN_TRAILING_FLAG_B);
    tunnel(&inner)
}

/// Family 38, sub-opcode 1: current and maximum hit points.
pub fn build_hit_points(current: i32, maximum: i32) -> Vec<u8> {
    let mut inner = vec![38, 0, 1, 0];
    inner.extend_from_slice(&current.to_le_bytes());
    inner.extend_from_slice(&maximum.to_le_bytes());
    tunnel(&inner)
}

/// Family 38, sub-opcode 13: current value, secondary value, then flag byte.
pub fn build_mana(current: i32, secondary: i32, flag: u8) -> Vec<u8> {
    let mut inner = vec![38, 0, 13, 0];
    inner.extend_from_slice(&current.to_le_bytes());
    inner.extend_from_slice(&secondary.to_le_bytes());
    inner.push(flag);
    tunnel(&inner)
}

/// Family 209, u32 subtype 2: the character's journal key/value set.
pub fn build_journal_set(entries: &[(u32, u32)]) -> Vec<u8> {
    let mut inner = Vec::with_capacity(10 + entries.len() * 8);
    inner.extend_from_slice(&209u16.to_le_bytes());
    inner.extend_from_slice(&2u32.to_le_bytes());
    inner.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (key, value) in entries {
        inner.extend_from_slice(&key.to_le_bytes());
        inner.extend_from_slice(&value.to_le_bytes());
    }
    tunnel(&inner)
}

/// Family 38, sub-opcode 26: end of preload character data.
pub fn build_done_sending_preload_characters() -> Vec<u8> {
    tunnel(&[38, 0, 26, 0, PRELOAD_MARKER_UNKNOWN_BYTE])
}
