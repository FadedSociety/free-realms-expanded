//! Shared helpers for the job/field playtest suites (`job_basic_attacks`,
//! `job_trainers`, `job_ability_bars`, `field_enemies_runtime`).
//!
//! Include with `#[path = "common/job_playtest.rs"] mod job_playtest;`.
//! Test-only; production never builds this module. The two captured T010 client
//! rows and the 36/5 toolbar and 32/7 combat decoders live here once so the
//! suites that use them cannot drift apart.
#![allow(dead_code)] // each include site uses a subset of the helpers

use serde_json::Value;
use std::{fs, path::Path};

/// The pinned reference packages predate the Wizard and Warrior starter weapons;
/// the production `item-catalogue.json` carries both. These are the captured
/// client rows `job_basic_attacks` (T010) installs. Only definitions are added,
/// so the starter grant still decides what is owned.
pub fn install_starter_weapon_definitions(dir: &Path) {
    let path = dir.join("inventory.json");
    let mut inventory: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let definitions = inventory["definitions"].as_array_mut().unwrap();
    for row in [
        serde_json::json!({
            "id": 4914,
            "slot": 7,
            "class_id": 10,
            "default_tint": 0,
            "max_stack": -1,
            "model": "wand_ar_ag_weapon_crude.adr",
            "texture": "",
            "tint_alias": "",
            "effect": 0,
            "wield_type": 10,
            "serialized_hex": "32130000000000000000000000000000000000000000000000000000000000000a0000000000000007000000000000000000000000000000",
            "sha256": "28b02e59bcf7c0fe6de8cfa2c429bcf5ebcbf3e1e20d1d9a6917599017ef5f14"
        }),
        serde_json::json!({
            "id": 29963,
            "slot": 7,
            "class_id": 17,
            "default_tint": 0,
            "max_stack": -1,
            "model": "club_ar_ag_weapon_cudgel.adr",
            "texture": "",
            "tint_alias": "",
            "effect": 0,
            "wield_type": 12,
            "serialized_hex": "0b75000000000000000000000000000000000000000000000000000000000000110000000000000007000000000000000000000000000000",
            "sha256": "dd8b3607d16fd953e370c2af82240ee38bb5a5f4f2aef8d42b1ddc0a51799e96"
        }),
    ] {
        if !definitions
            .iter()
            .any(|definition| definition["id"] == row["id"])
        {
            definitions.push(row);
        }
    }
    fs::write(path, serde_json::to_vec(&inventory).unwrap()).unwrap();
}

/// The production play package ships Ninja/Wizard/Warrior starter outfits; the
/// pinned reference packages predate them (T201). Add the captured Chest/Legs
/// pieces T010 uses when the reference has no list, so the real
/// trainer/activation path can dress all six.
pub fn install_starter_outfits(dir: &Path) {
    let path = dir.join("profiles.json");
    let mut profiles: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let starter = profiles["starter_items"].as_object_mut().unwrap();
    for (job, items) in [
        (2, [19197, 19787]),
        (12, [12060, 12709]),
        (32, [46137, 46732]),
    ] {
        starter
            .entry(job.to_string())
            .or_insert_with(|| serde_json::json!(items));
    }
    fs::write(path, serde_json::to_vec(&profiles).unwrap()).unwrap();
}

/// The 32/7 `attack_processed` hits for `attacker`: the damage dword at packet
/// offset 28. The client reader `0x00a2a910` reads three 8-byte pairs first, so
/// `[28..32]` is the server's first post-GUID dword, not a guessed field.
pub fn hits_from(replies: &[Vec<u8>], attacker: u64) -> Vec<u32> {
    replies
        .iter()
        .filter(|packet| {
            packet.starts_with(&[32, 0, 7, 0])
                && packet.get(4..12) == Some(attacker.to_le_bytes().as_slice())
        })
        .map(|packet| u32::from_le_bytes(packet[28..32].try_into().unwrap()))
        .collect()
}

/// Decode the native variable-length 36/5 toolbar into eight definition ids,
/// with `None` for an empty slot. Index 0 is the basic attack; job slot N is
/// index N. Client handler `0x00a35cc0` case 5 -> serializer `0x008fa5c0` (slot
/// count) -> slot reader `0x008e6760`: kind at +0, id at +4, kind 0 is 4 bytes
/// and kinds 1/3 are 41 bytes.
pub fn bar_ids(packets: &[Vec<u8>]) -> Vec<Option<i32>> {
    let toolbar = packets.last().expect("toolbar");
    assert!(toolbar.starts_with(&[36, 0, 5, 0]));
    assert_eq!(i32::from_le_bytes(toolbar[8..12].try_into().unwrap()), 8);
    let mut at = 12;
    let mut ids = Vec::new();
    for _ in 0..8 {
        let kind = i32::from_le_bytes(toolbar[at..at + 4].try_into().unwrap());
        match kind {
            0 => {
                ids.push(None);
                at += 4;
            }
            3 => {
                ids.push(Some(i32::from_le_bytes(
                    toolbar[at + 4..at + 8].try_into().unwrap(),
                )));
                at += 41;
            }
            other => panic!("unexpected toolbar entry type {other}"),
        }
    }
    assert_eq!(at, toolbar.len());
    ids
}
