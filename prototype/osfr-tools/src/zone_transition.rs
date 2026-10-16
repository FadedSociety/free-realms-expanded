//! Original world scene transition packet (opcode 31, `ClientBeginZoning`).
//!
//! Field order and the trailing layout come from the first-party native reader:
//! the opcode `0x1f` case of the client's packet switch constructs
//! `PacketClientBeginZoning` and calls the body reader `FUN_008e4ae0`
//! (`lab/decompiled/client/FreeRealms/pseudocode.c`, reader at `:1029104`,
//! opcode case at `:1108664`). That reader consumes, in order: `u16` opcode,
//! length-prefixed name, `i32` type, position `4 x f32`, rotation `4 x f32`,
//! length-prefixed sky, `u8` tutorial, `u8` unknown, `i32` id, `f32` update
//! radius (the native reader also rejects NaN here), `i32` geometry id and two
//! `u8` booleans. The pinned community writer
//! (`Sanctuary.Packet.PacketClientBeginZoning` + `PacketWriter`) emits the same
//! order, and its strings are UTF-8 byte length prefixes.
//!
//! This codec models the complete frame, including the three-field envelope
//! (`u16` family `5`, one flag byte, `u32` inner length). The flag is preserved
//! verbatim instead of invented; every preserved opcode-31 capture carries `1`.
//! The legacy zone-details packet is opcode 43 and is a different family; this
//! module never decodes it.

use serde::{Deserialize, Serialize};
use std::io;

/// Client begin-zoning opcode.
pub const OPCODE: u16 = 31;
/// Outer envelope family observed on every preserved opcode-31 frame.
pub const OUTER_FAMILY: u16 = 5;
/// Upper bound for the two length-prefixed UTF-8 strings.
pub const MAX_STRING_BYTES: usize = 4096;
/// Outer envelope size before the opcode: `u16` family + flag + `u32` length.
const HEADER_BYTES: usize = 7;
/// Fixed tail after the sky string: tutorial, unknown, id, radius, geometry, 2 booleans.
const TAIL_BYTES: usize = 1 + 1 + 4 + 4 + 4 + 1 + 1;

/// One complete begin-zoning packet.
///
/// Defaults are zeros with empty strings: the community writer's `Type = 2`
/// default and the captured envelope flag are caller-supplied, not invented.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZoneTransition {
    /// Outer envelope flag byte, preserved verbatim on decode.
    pub envelope_flag: u8,
    /// Scene/zone name (`Name`).
    pub scene: String,
    /// Packet type (`Type`).
    pub type_id: i32,
    /// Spawn position, `w` included.
    pub position: [f32; 4],
    /// Spawn rotation quaternion `(x, y, z, w)`.
    pub rotation: [f32; 4],
    /// Sky definition name (`Sky`).
    pub sky: String,
    /// Tutorial mode flag.
    pub tutorial: bool,
    /// Unknown byte with no recovered meaning; stored verbatim.
    pub unknown: u8,
    /// Zone instance id.
    pub id: i32,
    /// Position update radius.
    pub update_radius: f32,
    /// Adventurers' Journal geometry id.
    pub geometry_id: i32,
    /// Override the caller's update radius.
    pub override_update_radius: bool,
    /// Wait for the zone-ready packet before spawning.
    pub wait_for_zone_ready: bool,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn checked_text<'a>(what: &str, value: &'a str) -> io::Result<&'a [u8]> {
    let bytes = value.as_bytes();
    if bytes.len() > MAX_STRING_BYTES {
        return Err(invalid(format!(
            "zone transition {what} is {} bytes, over the {MAX_STRING_BYTES}-byte bound",
            bytes.len()
        )));
    }
    Ok(bytes)
}

fn checked_floats(what: &str, values: &[f32]) -> io::Result<()> {
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(invalid(format!(
                "zone transition {what} {index} is not finite"
            )));
        }
    }
    Ok(())
}

/// Serialize one complete opcode-31 frame.
pub fn encode(packet: &ZoneTransition) -> io::Result<Vec<u8>> {
    let scene = checked_text("scene", &packet.scene)?;
    let sky = checked_text("sky", &packet.sky)?;
    checked_floats("position", &packet.position)?;
    checked_floats("rotation", &packet.rotation)?;
    checked_floats("update radius", &[packet.update_radius])?;

    let mut inner = Vec::with_capacity(2 + 4 + scene.len() + 4 + 32 + 4 + sky.len() + TAIL_BYTES);
    inner.extend_from_slice(&OPCODE.to_le_bytes());
    inner.extend_from_slice(&(scene.len() as i32).to_le_bytes());
    inner.extend_from_slice(scene);
    inner.extend_from_slice(&packet.type_id.to_le_bytes());
    for value in packet.position.iter().chain(packet.rotation.iter()) {
        inner.extend_from_slice(&value.to_le_bytes());
    }
    inner.extend_from_slice(&(sky.len() as i32).to_le_bytes());
    inner.extend_from_slice(sky);
    inner.push(u8::from(packet.tutorial));
    inner.push(packet.unknown);
    inner.extend_from_slice(&packet.id.to_le_bytes());
    inner.extend_from_slice(&packet.update_radius.to_le_bytes());
    inner.extend_from_slice(&packet.geometry_id.to_le_bytes());
    inner.push(u8::from(packet.override_update_radius));
    inner.push(u8::from(packet.wait_for_zone_ready));

    let length = u32::try_from(inner.len())
        .map_err(|_| invalid("zone transition frame exceeds the length field"))?;
    let mut bytes = Vec::with_capacity(HEADER_BYTES + inner.len());
    bytes.extend_from_slice(&OUTER_FAMILY.to_le_bytes());
    bytes.push(packet.envelope_flag);
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(&inner);
    Ok(bytes)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, count: usize, what: &str) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| {
                invalid(format!(
                    "zone transition {what} runs past the frame (offset {} + {count} > {})",
                    self.offset,
                    self.bytes.len()
                ))
            })?;
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn u8(&mut self, what: &str) -> io::Result<u8> {
        Ok(self.take(1, what)?[0])
    }

    fn u16(&mut self, what: &str) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2, what)?.try_into().unwrap()))
    }

    fn i32(&mut self, what: &str) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4, what)?.try_into().unwrap()))
    }

    fn f32(&mut self, what: &str) -> io::Result<f32> {
        let value = f32::from_le_bytes(self.take(4, what)?.try_into().unwrap());
        if !value.is_finite() {
            return Err(invalid(format!("zone transition {what} is not finite")));
        }
        Ok(value)
    }

    fn text(&mut self, what: &str) -> io::Result<String> {
        let length = self.i32(what)?;
        if length < 0 || length as usize > MAX_STRING_BYTES {
            return Err(invalid(format!(
                "zone transition {what} length {length} is outside 0..={MAX_STRING_BYTES}"
            )));
        }
        let bytes = self.take(length as usize, what)?;
        std::str::from_utf8(bytes)
            .map(str::to_string)
            .map_err(|_| invalid(format!("zone transition {what} is not valid UTF-8")))
    }
}

/// Parse one complete opcode-31 frame, refusing malformed lengths, non-finite
/// floats, oversized or truncated strings and trailing bytes.
pub fn decode(bytes: &[u8]) -> io::Result<ZoneTransition> {
    if bytes.len() < HEADER_BYTES + 2 {
        return Err(invalid(format!(
            "zone transition frame is {} bytes, under the 9-byte minimum",
            bytes.len()
        )));
    }
    let family = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
    if family != OUTER_FAMILY {
        return Err(invalid(format!(
            "zone transition outer family {family} is not {OUTER_FAMILY}"
        )));
    }
    let declared = u32::from_le_bytes(bytes[3..7].try_into().unwrap()) as usize;
    if declared != bytes.len() - HEADER_BYTES {
        return Err(invalid(format!(
            "zone transition declares {declared} inner bytes but the frame holds {}",
            bytes.len() - HEADER_BYTES
        )));
    }
    let envelope_flag = bytes[2];
    let mut cursor = Cursor {
        bytes,
        offset: HEADER_BYTES,
    };
    let opcode = cursor.u16("opcode")?;
    if opcode != OPCODE {
        return Err(invalid(format!(
            "zone transition opcode {opcode} is not {OPCODE}"
        )));
    }
    let scene = cursor.text("scene")?;
    let type_id = cursor.i32("type")?;
    let position = [
        cursor.f32("position 0")?,
        cursor.f32("position 1")?,
        cursor.f32("position 2")?,
        cursor.f32("position 3")?,
    ];
    let rotation = [
        cursor.f32("rotation 0")?,
        cursor.f32("rotation 1")?,
        cursor.f32("rotation 2")?,
        cursor.f32("rotation 3")?,
    ];
    let sky = cursor.text("sky")?;
    let tutorial = cursor.u8("tutorial")? != 0;
    let unknown = cursor.u8("unknown")?;
    let id = cursor.i32("id")?;
    let update_radius = cursor.f32("update radius")?;
    let geometry_id = cursor.i32("geometry")?;
    let override_update_radius = cursor.u8("override update radius")? != 0;
    let wait_for_zone_ready = cursor.u8("wait for zone ready")? != 0;
    if cursor.offset != bytes.len() {
        return Err(invalid(format!(
            "zone transition carries {} trailing bytes",
            bytes.len() - cursor.offset
        )));
    }
    Ok(ZoneTransition {
        envelope_flag,
        scene,
        type_id,
        position,
        rotation,
        sky,
        tutorial,
        unknown,
        id,
        update_radius,
        geometry_id,
        override_update_radius,
        wait_for_zone_ready,
    })
}
