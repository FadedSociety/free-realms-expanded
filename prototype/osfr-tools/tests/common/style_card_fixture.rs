//! T478: what the style card use tests share: the client's own mapping tables, the cards from
//! our item catalogue, and one bearded human boy and one human girl as creation identities.
//! Test-only, with no `crate::` items so `#[path]`-inclusion works in the library and in the
//! gateway binary. Card rows and the identity's look are the ones `tests/style_cards.rs` pins.

use osfr_tools::{
    identity::{CharacterIdentity, NativeCreation},
    inventory::InventoryItem,
    style_cards::ClientTables,
};
use std::{path::PathBuf, sync::Arc};

pub const HUMAN_BOY: i32 = 1;
pub const HUMAN_GIRL: i32 = 60;

/// Definition ids in `lab/reports/round70-native-20261016T061000Z/content/item-catalogue.json`.
pub const CLEAN_SHAVEN: i32 = 36022; // PARAM1 5, PARAM2 0, boys
pub const BEARD: i32 = 36023; // PARAM1 5, PARAM2 1, boys
pub const SKULL_FACE_PAINT: i32 = 36004; // PARAM1 6, PARAM2 1
pub const FACE_TYPE: i32 = 77472; // PARAM1 9, PARAM2 1
pub const BUTTERFLY_WINGS: i32 = 36030; // PARAM1 5, PARAM2 7, pixie girls
pub const FAIRY_WINGS: i32 = 77536; // PARAM1 7, PARAM2 13, pixie boys
pub const CHINA_HEAD: i32 = 78086; // PARAM1 0
pub const MODEL_CARD: i32 = 77641; // PARAM1 8
/// A real catalogue row that is not a style card.
pub const NOT_A_CARD: i32 = 75_060;

/// The client's `Resources` tree with `Models.txt` and `CharacterCreate/*Mappings.txt`.
pub fn tables() -> Arc<ClientTables> {
    let resources = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/working/OSFR.Client/OSFR Client/Resources");
    Arc::new(
        ClientTables::load(&resources)
            .unwrap_or_else(|error| panic!("{} loads: {error}", resources.display())),
    )
}

fn identity(model: i32, gender: i32, hair: (&str, i32), beard: (&str, i32)) -> CharacterIdentity {
    let mut choices = [0; 21];
    // Model, head, hair, model customization, face paint, skin tone; eye and hair tints.
    choices[..6].copy_from_slice(&[model, 2, hair.1, beard.1, 0, 3]);
    choices[18..20].copy_from_slice(&[6, 5]);
    CharacterIdentity {
        guid: 1,
        first_name: "A".into(),
        last_name: String::new(),
        model,
        head: "<race>_<gender>_head_african.adr".into(),
        hair: hair.0.into(),
        model_customization: beard.0.into(),
        face_paint: "none".into(),
        skin_tone: "skintone3".into(),
        eye_color: 6,
        hair_color: 5,
        gender,
        creation: NativeCreation {
            first_name: "A".into(),
            last_name: String::new(),
            temporary_first_name: String::new(),
            temporary_last_name: String::new(),
            choices,
        },
        attachments: Vec::new(),
    }
}

/// A human boy with the first beard.
pub fn boy() -> CharacterIdentity {
    identity(
        HUMAN_BOY,
        1,
        ("<race>_m_hair_afro_<hairtype>.adr", 1),
        ("human_m_facialhair_beard.adr", 1),
    )
}

pub fn girl() -> CharacterIdentity {
    identity(
        HUMAN_GIRL,
        2,
        ("<race>_f_hair_flame_<hairtype>.adr", 11),
        ("", 0),
    )
}

/// One inventory row; the row's `id` is the instance id a 42/10 names.
pub fn card_item(id: i32, definition_id: i32, count: u32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

/// The 8-byte 42/10 or 42/12 body (`00c11600`, `00a1b9a0`).
pub fn request(opcode: u8, value: i32) -> Vec<u8> {
    let mut packet = vec![42, 0, opcode, 0];
    packet.extend(value.to_le_bytes());
    packet
}
