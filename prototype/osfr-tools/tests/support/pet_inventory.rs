//! Shared pet test inventory (T454, moved from tests/pets.rs): a temporary inventory package
//! holding the given items, each with a minimal client-shaped definition, loaded through the
//! real `Inventory::load`.
#![allow(dead_code)]

use osfr_tools::inventory::{EquippedItem, Inventory, InventoryItem, InventoryState};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub struct InventoryFixture {
    directory: PathBuf,
    pub inventory: Inventory,
}

impl InventoryFixture {
    /// The four-item inventory tests/pets.rs has always used.
    pub fn new() -> Self {
        Self::with_items(&[(10, 9001, 1), (11, 1835, 2), (12, 1836, 1), (13, 9002, 1)])
    }

    /// `(instance id, definition id, count)` rows; each distinct definition gets a minimal row.
    pub fn with_items(rows: &[(i32, i32, u32)]) -> Self {
        let tinted: Vec<_> = rows
            .iter()
            .map(|&(id, def, count)| (id, def, count, 0))
            .collect();
        Self::with_tinted_items(&tinted)
    }

    /// `(instance id, definition id, count, tint)` rows (T456: a tinted adoption item).
    pub fn with_tinted_items(rows: &[(i32, i32, u32, i32)]) -> Self {
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "osfr-pets-inventory-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let state = InventoryState {
            coins: 0,
            items: rows
                .iter()
                .map(|&(id, definition_id, count, tint)| InventoryItem {
                    tint,
                    ..item(id, definition_id, count)
                })
                .collect(),
            equipment: Vec::<EquippedItem>::new(),
            next_instance_id: None,
        };
        let mut packet = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        packet.extend([0xA5; 8]);
        let coins_offset = packet.len();
        packet.extend(0i32.to_le_bytes());
        let items_offset = packet.len();
        packet.extend((state.items.len() as i32).to_le_bytes());
        for row in &state.items {
            packet.extend(serialize_item(row));
        }
        packet.extend(b"fixture-tail");
        let length = packet.len();
        packet[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        packet[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());
        let span = |kind: &str, offset: usize, length: usize| {
            json!({
                "kind": kind,
                "offset": offset,
                "length": length,
                "profile_id": 0,
                "sha256": digest(&packet[offset..offset + length]),
            })
        };
        let mut ids: Vec<i32> = rows.iter().map(|row| row.1).collect();
        ids.sort_unstable();
        ids.dedup();
        let definitions: Vec<_> = ids
            .into_iter()
            .map(|id| {
                let bytes = [id.to_le_bytes(), 8i32.to_le_bytes()].concat();
                json!({
                    "id": id,
                    "slot": 0,
                    "class_id": 1,
                    "default_tint": 0,
                    "max_stack": 20,
                    "model": "",
                    "texture": "",
                    "tint_alias": "",
                    "effect": 0,
                    "wield_type": 0,
                    "serialized_hex": bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
                    "sha256": digest(&bytes),
                })
            })
            .collect();
        let manifest = json!({
            "schema_version": 1,
            "player_guid": 1,
            "default_state": state,
            "profiles": [],
            "spans": [
                span("coins", coins_offset, 4),
                span("items", items_offset, packet.len() - items_offset - b"fixture-tail".len()),
            ],
            "definitions": definitions,
        });
        fs::write(
            directory.join("inventory.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let inventory = Inventory::load(&directory, None).unwrap();
        Self {
            directory,
            inventory,
        }
    }
}

impl Drop for InventoryFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("inventory.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}

pub fn item(id: i32, definition_id: i32, count: u32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

fn serialize_item(item: &InventoryItem) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [
        item.definition_id,
        item.tint,
        item.id,
        item.count as i32,
        item.consumed_count as i32,
        item.last_cast_time as i32,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(u8::from(item.rental.is_some()));
    bytes.extend(item.ability_count.to_le_bytes());
    bytes.push(u8::from(item.activate_enabled));
    bytes
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
