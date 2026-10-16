//! Session/control framing verified against the supplied SOE implementation and
//! pinned Sanctuary Multi/ClockSync handlers. No application messages live here.

use crate::soe::{Codec, Packet};

pub const CRC_SEED: u32 = 1_884_358_976;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRequest {
    pub id: u32,
    pub crc_length: u8,
    pub buffer_size: usize,
}

impl SessionRequest {
    pub fn parse(wire: &[u8]) -> Result<Self, &'static str> {
        if wire.len() != 24 {
            return Err("expected a CGAPI_527 session request");
        }
        if wire[0] != 0 || wire[1] != 1 || &wire[14..] != b"CGAPI_527\0" {
            return Err("expected a CGAPI_527 session request");
        }
        let crc = u32::from_be_bytes([wire[2], wire[3], wire[4], wire[5]]);
        let buffer = u32::from_be_bytes([wire[10], wire[11], wire[12], wire[13]]);
        if crc > 4 || !(64..=65_507).contains(&buffer) {
            return Err("unsupported CRC length or UDP buffer size");
        }
        Ok(Self {
            id: u32::from_be_bytes([wire[6], wire[7], wire[8], wire[9]]),
            crc_length: crc as u8,
            buffer_size: buffer as usize,
        })
    }

    pub fn response(&self) -> Vec<u8> {
        self.response_with_seed(CRC_SEED)
    }

    pub fn response_with_seed(&self, seed: u32) -> Vec<u8> {
        let mut wire = vec![0, 2];
        wire.extend(self.id.to_be_bytes());
        wire.extend(seed.to_be_bytes());
        wire.extend([self.crc_length, 1, 0]);
        wire.extend((self.buffer_size as u32).to_be_bytes());
        wire.extend(3u32.to_be_bytes());
        wire
    }
}

/// Validate the whole transport bundle before handing any child to session
/// state. Children are already cooked: no per-child CRC or compression flag.
pub fn decode_controls(codec: &Codec, wire: &[u8]) -> Result<Vec<Packet>, String> {
    let packet = if wire == [0, 6] {
        // The legacy peer's KeepAlive response is deliberately bare.
        Packet {
            opcode: 6,
            payload: Vec::new(),
        }
    } else {
        codec.decode(wire).map_err(|e| e.to_string())?
    };
    let mut output = Vec::new();
    flatten(packet, 0, &mut output)?;
    Ok(output)
}

fn flatten(packet: Packet, depth: usize, output: &mut Vec<Packet>) -> Result<(), String> {
    if output.len() >= 256 || depth > 3 {
        return Err("SOE control bundle exceeds its bounds".into());
    }
    if packet.opcode != 3 {
        output.push(packet);
        return Ok(());
    }
    let mut remaining = packet.payload.as_slice();
    if remaining.is_empty() {
        return Err("empty SOE control bundle".into());
    }
    while !remaining.is_empty() {
        let length = usize::from(remaining[0]);
        remaining = &remaining[1..];
        if length < 2 || length > remaining.len() {
            return Err("invalid SOE control child length".into());
        }
        let (child, rest) = remaining.split_at(length);
        flatten(
            Packet {
                opcode: u16::from_be_bytes([child[0], child[1]]),
                payload: child[2..].to_vec(),
            },
            depth + 1,
            output,
        )?;
        remaining = rest;
    }
    Ok(())
}

/// Reflect a valid legacy clock sync. Counts refer to this session only.
pub fn clock_reflect(packet: &Packet, now_ms: u64, sent: u64, received: u64) -> Option<Packet> {
    if packet.opcode != 7 || packet.payload.len() != 38 {
        return None;
    }
    let mut payload = packet.payload[..2].to_vec();
    payload.extend((now_ms as u32).to_be_bytes());
    payload.extend(&packet.payload[22..38]);
    payload.extend(sent.to_be_bytes());
    payload.extend(received.to_be_bytes());
    Some(Packet { opcode: 8, payload })
}
