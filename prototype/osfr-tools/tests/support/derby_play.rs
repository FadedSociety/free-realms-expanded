//! Shared derby wire builders for the suites that drive a real derby match
//! (`derby_chapter`, `derby_route`, `derby_rows`, `derby_shipped_package`). Test-only.
//!
//! Each builds one client-to-server packet: the family-39 native Go and the 34/x derby
//! inner messages (register, phase, countdown, vehicle hit), exactly as the client sends them.
#![allow(dead_code)]

use osfr_tools::derby::{DerbyMessage, ParticipantKey, VehicleHitReport, encode_inner};

pub fn start_request(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 5];
    bytes.extend(instance.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

pub fn register_for(key: ParticipantKey) -> Vec<u8> {
    encode_inner(&DerbyMessage::RegisterPlayer {
        key,
        unknown_key: 0,
    })
    .unwrap()
}

pub fn phase_for(key: ParticipantKey, phase: i32) -> Vec<u8> {
    encode_inner(&DerbyMessage::PlayerPhase { key, phase }).unwrap()
}

pub fn countdown_for(key: ParticipantKey, count: u32) -> Vec<u8> {
    encode_inner(&DerbyMessage::CountdownStatus { key, count }).unwrap()
}

pub fn hit_for(reporter: ParticipantKey, other: ParticipantKey, impact: f32) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter,
        other,
        impact_bits: impact.to_bits(),
    }))
    .unwrap()
}
