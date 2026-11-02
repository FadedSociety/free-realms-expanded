//! Shared request builders and the fish-definition fixture for the fishing tests
//! (`fishing_entry`, `fishing_service`, `fisherman_entry_chain`,
//! `fishing_guide_drawable` and `quest_any_of_objectives`). Test-only; production
//! never includes this file.
//!
//! 138 request shapes come from the client writers `00b6a380|400|460|4d0`; the
//! family-39 envelope from the base constructor `008a4420` (family, subtype, then
//! three i32: instance, `-1`, game id). 39/5 start and 39/6 quit are sent by
//! `009be7d0` / `009be8e0`, both through `009bdc60`; 41/109 is the existing
//! encounter close.
#![allow(dead_code)]

use osfr_tools::{
    fishing::Handle, inventory::Inventory, profiles::ProfileState, rewards::RewardState,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

/// Every fish item id the shipped `fishing_spots.json` binds.
pub fn fish_item_ids() -> BTreeSet<i32> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/fishing_spots.json");
    let value: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    value["items"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|sizes| sizes.as_array().unwrap().iter())
        .map(|id| id.as_i64().unwrap() as i32)
        .filter(|id| *id > 0)
        .collect()
}

/// (T790) One item definition row for a fishing fixture `inventory.json`: the given id
/// in the shape the `fishing_service` fixture already used (slot 3, class 1).
pub fn definition_row(id: i32) -> Value {
    let bytes = [id.to_le_bytes(), 8i32.to_le_bytes()].concat();
    json!({
        "id": id,
        "slot": 3,
        "class_id": 1,
        "default_tint": 5,
        "max_stack": 10,
        "model": "fixture.adr",
        "texture": "fixture",
        "tint_alias": "fixture",
        "effect": 0,
        "wield_type": 0,
        "serialized_hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "sha256": format!("{:x}", Sha256::digest(&bytes)),
    })
}

/// (T790) Write the minimal `inventory.json` the fishing fixtures share: no state rows,
/// one unlocked profile that allows item slot 3, and `definitions`. Creates `directory`,
/// so a caller can point it at its own temp dir. Shared by `fishing_service` and the
/// any_of suite.
pub fn write_inventory(directory: &Path, player_guid: u64, definitions: &[Value]) {
    let coins = 0u32.to_le_bytes();
    let empty = 0i32.to_le_bytes();
    let span = |kind: &str, offset: usize, profile_id: usize, bytes: &[u8]| {
        json!({
            "kind": kind,
            "offset": offset,
            "length": bytes.len(),
            "profile_id": profile_id,
            "sha256": format!("{:x}", Sha256::digest(bytes)),
        })
    };
    let content = json!({
        "schema_version": 1,
        "player_guid": player_guid,
        "default_state": {
            "coins": 0,
            "items": [],
            "equipment": [],
            "next_instance_id": null
        },
        "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
        "spans": [
            span("coins", 13, 0, &coins),
            span("items", 17, 0, &empty),
            span("equipment", 21, 1, &empty)
        ],
        "definitions": definitions
    });
    fs::create_dir_all(directory).unwrap();
    fs::write(
        directory.join("inventory.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .unwrap();
}

/// (T790) Load that inventory and the one unlocked profile into a reward ledger. The
/// bag capacity is a test value because `route` recomputes the fishing cap from the
/// live inventory.
pub fn reward_state(directory: &Path, capacity: usize) -> RewardState {
    let inventory = Inventory::load(directory, None).unwrap();
    let profiles = ProfileState {
        version: 1,
        active: 1,
        unlocked: BTreeSet::from([1]),
        experience: BTreeMap::new(),
    };
    RewardState::new(inventory, profiles, capacity)
}

/// (T790) A scratch directory's inventory holding every fish the shipped spot file can
/// grant, and the reward ledger a fishing round uses.
pub fn fish_inventory_rewards(directory: &Path, capacity: usize) -> RewardState {
    write_inventory(directory, 1, &[]);
    add_fish_definitions(directory);
    reward_state(directory, capacity)
}

/// Widen a fixture `inventory.json` with a stub definition for every fish item the real
/// spot file binds, so a catch can be stored. The spot file itself is shipped content.
pub fn add_fish_definitions(directory: &Path) {
    let path = directory.join("inventory.json");
    let mut inventory: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let known: BTreeSet<i32> = inventory["definitions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["id"].as_i64().map(|id| id as i32))
        .collect();
    let definitions = inventory["definitions"].as_array_mut().unwrap();
    for id in fish_item_ids() {
        if known.contains(&id) {
            continue;
        }
        definitions.push(definition_row(id));
    }
    fs::write(&path, serde_json::to_vec(&inventory).unwrap()).unwrap();
}

pub fn handle_bytes(handle: Handle) -> Vec<u8> {
    [handle.id.to_le_bytes(), handle.seq.to_le_bytes()].concat()
}

/// One opcode-138 request: `u16` opcode, `u16` sub-opcode, body.
pub fn request(sub: u16, body: &[u8]) -> Vec<u8> {
    let mut packet = vec![138, 0];
    packet.extend(sub.to_le_bytes());
    packet.extend(body);
    packet
}

pub fn register(handle: Handle) -> Vec<u8> {
    let mut body = handle_bytes(handle);
    body.push(0);
    request(2, &body)
}

/// 138/6 cast: handle, target vector (w = 1), flag byte.
pub fn cast_at(handle: Handle, origin: [f32; 3], distance: f32) -> Vec<u8> {
    let mut body = handle_bytes(handle);
    for coordinate in [origin[0] + distance, origin[1], origin[2], 1.0f32] {
        body.extend(coordinate.to_le_bytes());
    }
    body.push(0);
    request(6, &body)
}

/// 138/7 reel: handle, flag byte.
pub fn reel(handle: Handle) -> Vec<u8> {
    let mut body = handle_bytes(handle);
    body.push(1);
    request(7, &body)
}

/// Family-39 lifecycle envelope: instance, `-1`, game id.
pub fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet
}

pub fn encounter_close(instance: i32) -> Vec<u8> {
    let mut packet = vec![41, 0, 109, 0];
    packet.extend(0i32.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet
}

/// One Fish Finder row, as the client reads it (`0x00b673b0`): three u32, a skip byte, the
/// FishSpecial byte, the lure-hint id, an i32-length string, then the school id (L933: the school
/// the finder points at, 0 for none), the catchable byte and the finder effect id (the composite
/// effect played on that school). The first u32 is the row's type; the client drops a row whose
/// type it has already listed (`0x00b65ed0`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FishRow {
    pub type_id: u32,
    pub name_id: u32,
    pub icon_id: u32,
    pub special: bool,
    pub lure_hint_id: u32,
    /// `fisherman_level >= minimum_level`; the client's FishCatchable column.
    pub catchable: bool,
    pub school_id: u32,
    pub finder_effect: u32,
}

/// The fish rows of a 138/3: the list follows the 16 player words, the underwater-sky string,
/// the 5 zone words and the u32 model-id array (`0x00b65880`, `0x00b67240`, `0x008dab40`,
/// `0x00b691d0`).
pub fn register_fish_rows(packet: &[u8]) -> Vec<FishRow> {
    let (_, at) = register_models_and_rows_at(packet);
    fish_rows(&packet[4 + at..])
}

/// The model ids a 138/3 preloads (`0x008dab40`): the fish shapes and, since L933, the school
/// models.
pub fn register_models(packet: &[u8]) -> Vec<u32> {
    register_models_and_rows_at(packet).0
}

/// The preload model ids and the body offset where the fish rows start.
fn register_models_and_rows_at(packet: &[u8]) -> (Vec<u32>, usize) {
    let body = &packet[4..];
    let word = |at: usize| u32::from_le_bytes(body[at..at + 4].try_into().unwrap());
    let mut at = 16 * 4;
    at += 4 + word(at) as usize;
    at += 5 * 4;
    let models = (0..word(at) as usize)
        .map(|i| word(at + 4 + i * 4))
        .collect::<Vec<_>>();
    at += 4 + models.len() * 4;
    (models, at)
}

/// The fish rows of a 138/4 FishInfoUpdate: a count then the same rows.
pub fn fish_info_rows(packet: &[u8]) -> Vec<FishRow> {
    fish_rows(&packet[4..])
}

fn fish_rows(body: &[u8]) -> Vec<FishRow> {
    let word = |at: usize| u32::from_le_bytes(body[at..at + 4].try_into().unwrap());
    let count = word(0) as usize;
    let mut at = 4;
    let mut rows = Vec::with_capacity(count);
    for _ in 0..count {
        let name_id = word(at + 4);
        let icon_id = word(at + 8);
        let special = body[at + 13] != 0;
        let lure_hint_id = word(at + 14);
        let string = word(at + 18) as usize;
        let next = at + 22 + string;
        let catchable = body[next + 4] != 0;
        rows.push(FishRow {
            school_id: word(next),
            finder_effect: word(next + 5),
            type_id: word(at),
            name_id,
            icon_id,
            special,
            lure_hint_id,
            catchable,
        });
        at = next + 9;
    }
    assert_eq!(at, body.len(), "the fish rows consume the body");
    rows
}

/// One 138/11 school as the client reads it (`0x00b69130`): the school id, its position and
/// facing vectors, the model records (id, heading, animation speed), the composite effect ids
/// and 8 trailing bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct School {
    pub id: u32,
    pub position: [f32; 3],
    pub models: Vec<(i32, f32, f32)>,
    pub effects: Vec<i32>,
}

pub fn school(packet: &[u8]) -> School {
    assert_eq!(&packet[..4], &[138, 0, 11, 0], "a 138/11 school");
    let body = &packet[4..];
    let word = |at: usize| u32::from_le_bytes(body[at..at + 4].try_into().unwrap());
    let float = |at: usize| f32::from_bits(word(at));
    let position = [float(4), float(8), float(12)];
    let mut at = 4 + 16 + 16;
    let models = (0..word(at) as usize)
        .map(|i| {
            let row = at + 4 + i * 12;
            (word(row) as i32, float(row + 4), float(row + 8))
        })
        .collect::<Vec<_>>();
    at += 4 + models.len() * 12;
    let effects = (0..word(at) as usize)
        .map(|i| word(at + 4 + i * 4) as i32)
        .collect::<Vec<_>>();
    at += 4 + effects.len() * 4 + 8;
    assert_eq!(at, body.len(), "the school consumes the body");
    School {
        id: word(0),
        position,
        models,
        effects,
    }
}
