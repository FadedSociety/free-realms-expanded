//! Work item 045: cosmetic equipment validation and native-equip replay.
//!
//! ORIGINAL evidence used by these vectors (all outside Git):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` line 1099052 logs an applied equip as
//!   `"EquipItem: Slot=%d, ModelName=%s, TextureAlias=%s, TintAlias=%s, TintIdOverride=%d,
//!   Effect=%d."` — the exact field order the attachment writer uses.
//! * `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json` (AGPL-3.0,
//!   revision `9e082025391177d5a33ee18c714ddd06b457a695`) — every `Slot`/`CategoryId` pair and
//!   the `Icon.TintId` fallback (`src/Sanctuary.Gateway/Handlers/BaseInventoryPacket/
//!   InventoryPacketEquipByGuidHandler.cs` uses `clientItem.Tint == 0 ? Icon.TintId : Tint`).
//! * `lab/references/opening-anvil-placement-v1/content/inventory.json` — the running server
//!   content, built from the captured `PlayerCode.SendPlayerUpdateItemDefinitions` payload with
//!   community starter rows merged in.
//! * `lab/state/*.json` — saved native gameplay snapshots that carry the equipped item rows.
//!
//! The slot table, the appearance predicates and the replay/restart checks in the module are
//! RECONSTRUCTED policy over those facts. Nothing here runs the native client, grants an item or
//! proves native equip; the private gate below replays a real saved snapshot off-line.
//!
//! This file is a `#[path]` include so it exercises the real module before the lead registers
//! `pub mod piece_045;` and the matching declarations in `src/lib.rs`.

#[path = "../src/restoration/piece_045.rs"]
#[allow(dead_code)]
mod piece_045;

use osfr_inventory::{EquippedItem, Inventory, InventoryItem, InventoryState};
use piece_045::{
    Appearance, COSMETIC_SLOTS, CosmeticError, assert_profiles_identical, capture_profiles,
    cosmetic_slot, inspect, parse_equip_reply, parse_profile_attachments, replay_equip,
    validate_appearance,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Re-export shim so the vectors below can name the included module's `Inventory` type.
mod osfr_inventory {
    pub use super::piece_045::inventory::{EquippedItem, Inventory, InventoryItem, InventoryState};
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn item(id: i32, definition_id: i32, tint: i32) -> InventoryItem {
    InventoryItem {
        id,
        definition_id,
        tint,
        count: 1,
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
        i32::try_from(item.last_cast_time).unwrap(),
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(u8::from(item.rental.is_some()));
    if let Some(rental) = &item.rental {
        bytes.extend(rental.expiration.to_le_bytes());
        bytes.extend(rental.upsells.to_le_bytes());
        bytes.extend(rental.bundle_id.to_le_bytes());
    }
    bytes.extend(item.ability_count.to_le_bytes());
    bytes.push(u8::from(item.activate_enabled));
    bytes
}

/// Production's private `span_bytes` layout, re-derived in the test the same way
/// `tests/inventory.rs` does.
fn span_bytes(state: &InventoryState, kind: &str, profile_id: i32) -> Vec<u8> {
    let mut bytes = Vec::new();
    match kind {
        "coins" => bytes.extend((state.coins as i32).to_le_bytes()),
        "items" => {
            bytes.extend((state.items.len() as i32).to_le_bytes());
            for item in &state.items {
                bytes.extend(serialize_item(item));
            }
        }
        "equipment" => {
            let entries: Vec<_> = state
                .equipment
                .iter()
                .filter(|entry| entry.profile_id == profile_id)
                .collect();
            bytes.extend((entries.len() as i32).to_le_bytes());
            for entry in entries {
                for value in [entry.slot, entry.item_id, entry.category] {
                    bytes.extend(value.to_le_bytes());
                }
            }
        }
        other => panic!("unexpected span kind {other}"),
    }
    bytes
}

fn command(family: u16, opcode: u16, fields: &[i32]) -> Vec<u8> {
    let mut bytes = [family.to_le_bytes(), opcode.to_le_bytes()].concat();
    for field in fields {
        bytes.extend(field.to_le_bytes());
    }
    bytes
}

/// Synthetic content that mirrors the shipped shapes: a hat (slot 1), pants (slot 4) and a
/// weapon (slot 7). Only two profiles are modelled so the fixture stays small.
struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-piece-045-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        let state = InventoryState {
            next_instance_id: None,
            coins: 7,
            items: vec![item(1, 100, 0), item(2, 101, 12), item(3, 102, 0)],
            equipment: vec![
                EquippedItem {
                    profile_id: 1,
                    slot: 1,
                    item_id: 1,
                    category: 1,
                },
                EquippedItem {
                    profile_id: 1,
                    slot: 3,
                    item_id: 0,
                    category: 3,
                },
                EquippedItem {
                    profile_id: 1,
                    slot: 4,
                    item_id: 2,
                    category: 4,
                },
                EquippedItem {
                    profile_id: 1,
                    slot: 7,
                    item_id: 3,
                    category: 7,
                },
                EquippedItem {
                    profile_id: 2,
                    slot: 1,
                    item_id: 0,
                    category: 1,
                },
            ],
        };
        let mut packet = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        let mut spans = Vec::new();
        for (kind, profile_id) in [
            ("coins", 0),
            ("items", 0),
            ("equipment", 1),
            ("equipment", 2),
        ] {
            let bytes = span_bytes(&state, kind, profile_id);
            let offset = packet.len();
            packet.extend(&bytes);
            spans.push(json!({
                "kind": kind,
                "offset": offset,
                "length": bytes.len(),
                "profile_id": profile_id,
                "sha256": digest(&bytes),
            }));
        }
        let length = packet.len();
        packet[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        packet[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());
        let definition = |id: i32,
                          slot: i32,
                          class_id: i32,
                          tint: i32,
                          model: &str,
                          texture: &str,
                          effect: i32,
                          wield: i32| {
            let bytes = [id.to_le_bytes(), 8i32.to_le_bytes()].concat();
            json!({
                "id": id,
                "slot": slot,
                "class_id": class_id,
                "default_tint": tint,
                "max_stack": -1,
                "model": model,
                "texture": texture,
                "tint_alias": "dyetint",
                "effect": effect,
                "wield_type": wield,
                "serialized_hex": bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
                "sha256": digest(&bytes),
            })
        };
        let content = json!({
            "schema_version": 1,
            "player_guid": 1,
            "default_state": state,
            "profiles": [
                {"id": 1, "allowed_classes": [1], "slots": [1, 3, 4, 7]},
                {"id": 2, "allowed_classes": [1], "slots": [1]},
            ],
            "spans": spans,
            "definitions": [
                definition(100, 1, 1, 214, "hat_<race>_<gender>_head_cap.adr", "hat-plain-L1", 0, 0),
                definition(101, 4, 1, 231, "pants_<race>_<gender>_legs_bootcut.adr", "pants-plain-L1", 0, 0),
                definition(102, 7, 1, 0, "hammer_ar_ag_weapon_logjammer.adr", "brawler-clothhead-L1", 5485, 12),
            ],
        });
        fs::write(
            directory.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { directory }
    }

    fn content(&self) -> Value {
        serde_json::from_slice(&fs::read(self.directory.join("inventory.json")).unwrap()).unwrap()
    }

    fn inventory(&self) -> Inventory {
        Inventory::load(&self.directory, None).expect("the synthetic content loads")
    }

    fn inventory_with(&self, state: InventoryState) -> std::io::Result<Inventory> {
        Inventory::load(&self.directory, Some(state))
    }

    fn state(&self) -> InventoryState {
        serde_json::from_value(self.content()["default_state"].clone()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("inventory.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}

fn appearance(slot: i32, model: &str) -> Appearance {
    Appearance {
        model: model.to_owned(),
        texture: "hat-plain-L1".to_owned(),
        tint_alias: "dyetint".to_owned(),
        tint: 214,
        effect: 0,
        slot,
    }
}

fn encode_attachment(bytes: &mut Vec<u8>, appearance: &Appearance) {
    for text in [
        &appearance.model,
        &appearance.texture,
        &appearance.tint_alias,
    ] {
        bytes.extend((text.len() as i32).to_le_bytes());
        bytes.extend(text.as_bytes());
    }
    for value in [appearance.tint, appearance.effect, appearance.slot] {
        bytes.extend(value.to_le_bytes());
    }
}

#[test]
fn cosmetic_slots_are_the_recovered_appearance_pairs() {
    assert_eq!(COSMETIC_SLOTS.len(), 7);
    let mut slots: Vec<i32> = COSMETIC_SLOTS.iter().map(|row| row.slot).collect();
    slots.sort_unstable();
    slots.dedup();
    assert_eq!(slots.len(), COSMETIC_SLOTS.len(), "slots are unique");
    for row in COSMETIC_SLOTS {
        assert!(row.category_id > 0 && row.name_id > 0 && row.recovered_rows > 0);
        assert!(!row.category.is_empty());
        assert_eq!(
            cosmetic_slot(row.slot).map(|found| found.category_id),
            Some(row.category_id)
        );
    }
    assert_eq!(cosmetic_slot(1).unwrap().category, "Hats");
    assert_eq!(cosmetic_slot(3).unwrap().category, "Shirts & Dresses");
    assert_eq!(cosmetic_slot(4).unwrap().category, "Pants");
    assert_eq!(cosmetic_slot(5).unwrap().category, "Shoes");
    assert_eq!(cosmetic_slot(6).unwrap().category, "Miscellaneous");
    assert_eq!(cosmetic_slot(17).unwrap().category, "Costumes");
    // A weapon/tool slot is not a cosmetic appearance slot; that belongs to gear attributes.
    assert!(cosmetic_slot(7).is_none());
    assert!(cosmetic_slot(0).is_none());
}

#[test]
fn appearance_validation_uses_the_recovered_shape_rules() {
    validate_appearance(&appearance(1, "hat_<race>_<gender>_head_cap.adr")).unwrap();
    // ORIGINAL: 6366 cosmetic rows carry only the race placeholder, 6 carry none at all.
    validate_appearance(&appearance(5, "worker_human_<race>_feet_boots.adr")).unwrap();
    validate_appearance(&appearance(3, "freestyle_human_m_chest_shirt.adr")).unwrap();

    assert_eq!(
        validate_appearance(&appearance(7, "hammer_ar_ag_weapon.adr")).unwrap_err(),
        CosmeticError::NotCosmeticSlot { slot: 7 }
    );
    assert_eq!(
        validate_appearance(&appearance(1, "")).unwrap_err(),
        CosmeticError::EmptyModel { slot: 1 }
    );
    assert_eq!(
        validate_appearance(&appearance(1, "hat_<race>_head_cap.png")).unwrap_err(),
        CosmeticError::ModelNotMesh { slot: 1 }
    );
    assert_eq!(
        validate_appearance(&appearance(1, "hat_<haircolor>_head_cap.adr")).unwrap_err(),
        CosmeticError::UnknownModelPlaceholder {
            slot: 1,
            token: "<haircolor>".to_owned(),
        }
    );

    let mut missing_texture = appearance(1, "hat_<race>_head_cap.adr");
    missing_texture.texture.clear();
    assert_eq!(
        validate_appearance(&missing_texture).unwrap_err(),
        CosmeticError::EmptyTexture { slot: 1 }
    );

    let mut placeholder_texture = appearance(1, "hat_<race>_head_cap.adr");
    placeholder_texture.texture = "hat-<racetint>-L1".to_owned();
    assert_eq!(
        validate_appearance(&placeholder_texture).unwrap_err(),
        CosmeticError::TexturePlaceholder { slot: 1 }
    );

    let mut missing_alias = appearance(1, "hat_<race>_head_cap.adr");
    missing_alias.tint_alias.clear();
    assert_eq!(
        validate_appearance(&missing_alias).unwrap_err(),
        CosmeticError::EmptyTintAlias { slot: 1 }
    );

    let mut negative = appearance(1, "hat_<race>_head_cap.adr");
    negative.tint = -1;
    assert_eq!(
        validate_appearance(&negative).unwrap_err(),
        CosmeticError::NegativeTint { slot: 1, tint: -1 }
    );

    let mut oversized = appearance(1, "hat_<race>_head_cap.adr");
    oversized.model = "a".repeat(piece_045::MAX_APPEARANCE_TEXT + 1);
    assert_eq!(
        validate_appearance(&oversized).unwrap_err(),
        CosmeticError::TextTooLong { slot: 1 }
    );
}

#[test]
fn attachment_parser_reads_the_writer_layout_and_refuses_malformed_input() {
    let rows = vec![
        appearance(1, "hat_<race>_<gender>_head_cap.adr"),
        appearance(4, "pants_<race>_<gender>_legs_bootcut.adr"),
    ];
    let mut bytes = (rows.len() as i32).to_le_bytes().to_vec();
    for row in &rows {
        encode_attachment(&mut bytes, row);
    }
    assert_eq!(parse_profile_attachments(&bytes).unwrap(), rows);

    let empty = 0i32.to_le_bytes().to_vec();
    assert!(parse_profile_attachments(&empty).unwrap().is_empty());

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        parse_profile_attachments(&trailing).unwrap_err(),
        CosmeticError::Malformed("attachment list has trailing bytes")
    );

    let mut negative_count = bytes.clone();
    negative_count[..4].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(
        parse_profile_attachments(&negative_count).unwrap_err(),
        CosmeticError::Malformed("attachment count is out of bounds")
    );

    assert_eq!(
        parse_profile_attachments(&[7, 0]).unwrap_err(),
        CosmeticError::Malformed("truncated attachment field")
    );

    let mut bad_text = Vec::new();
    bad_text.extend(1i32.to_le_bytes());
    bad_text.extend((4i32).to_le_bytes());
    bad_text.extend(b"abcd");
    bad_text.extend((-1i32).to_le_bytes());
    assert_eq!(
        parse_profile_attachments(&bad_text).unwrap_err(),
        CosmeticError::Malformed("attachment text length is out of bounds")
    );
}

#[test]
fn equip_reply_parser_reads_the_client_update_packet() {
    let row = appearance(1, "hat_<race>_<gender>_head_cap.adr");
    // Production `Inventory::equip` writes 38/5 as: header, item id, attachment, profile,
    // class id, equip flag.
    let mut packet = vec![38, 0, 5, 0];
    packet.extend(1i32.to_le_bytes());
    encode_attachment(&mut packet, &row);
    packet.extend(1i32.to_le_bytes());
    packet.extend(1i32.to_le_bytes());
    packet.push(1);
    let parsed = parse_equip_reply(&packet).unwrap();
    assert_eq!(parsed.item_id, 1);
    assert_eq!(parsed.profile_id, 1);
    assert_eq!(parsed.class_id, 1);
    assert!(parsed.equip);
    assert_eq!(parsed.appearance, row);

    assert_eq!(
        parse_equip_reply(&[35, 0, 6, 0]).unwrap_err(),
        CosmeticError::Malformed("not a ClientUpdatePacketEquipItem")
    );
    let mut truncated = packet.clone();
    truncated.truncate(packet.len() - 1);
    assert_eq!(
        parse_equip_reply(&truncated).unwrap_err(),
        CosmeticError::Malformed("truncated equip reply")
    );
}

#[test]
fn inspect_reports_owned_appearance_rows_and_skips_weapons() {
    let fixture = Fixture::new();
    let rows = inspect(&fixture.inventory()).unwrap();
    let summary: Vec<(i32, i32, i32, i32, String)> = rows
        .iter()
        .map(|row| {
            (
                row.profile_id,
                row.slot,
                row.item_id,
                row.appearance.tint,
                row.appearance.model.clone(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        vec![
            (1, 1, 1, 214, "hat_<race>_<gender>_head_cap.adr".to_owned()),
            (
                1,
                4,
                2,
                12,
                "pants_<race>_<gender>_legs_bootcut.adr".to_owned()
            ),
        ],
        "the slot 7 weapon is not a cosmetic row and is skipped"
    );
    assert_eq!(rows[0].definition_id, 100);
    assert_eq!(rows[1].definition_id, 101);
    assert_eq!(rows[1].tint, 12, "the instance tint overrides the default");
}

#[test]
fn replay_accepts_owned_cosmetics_and_refuses_unowned_or_mismatched_slots() {
    let fixture = Fixture::new();
    let mut inventory = fixture.inventory();
    let before = inventory.profile_equipment(1);

    // Equip owned instance 2 (pants) into profile 1 / slot 4; it is already there, so the
    // request must be accepted and byte-stable.
    let replay = replay_equip(&mut inventory, 1, &command(42, 3, &[2, 1, 4]));
    let piece_045::Replay::Accepted {
        reply,
        visible,
        equipment,
    } = replay
    else {
        panic!("an owned cosmetic must be accepted, got {replay:?}");
    };
    assert_eq!(reply.item_id, 2);
    assert_eq!(reply.profile_id, 1);
    assert_eq!(reply.appearance.slot, 4);
    assert_eq!(reply.appearance.tint, 12);
    assert!(reply.appearance.model.ends_with(".adr"));
    assert_eq!(equipment, before);
    let visible = visible.expect("production also sends the visible-change packet");
    assert_eq!(visible.item_id, reply.item_id);
    assert_eq!(visible.appearance, reply.appearance);

    // An unowned instance id is refused and nothing moves.
    let refused = replay_equip(&mut inventory, 1, &command(42, 3, &[99, 1, 4]));
    let piece_045::Replay::Refused { error, equipment } = refused else {
        panic!("an unowned instance must be refused");
    };
    assert_eq!(
        error,
        CosmeticError::Inventory {
            kind: std::io::ErrorKind::InvalidData,
            message: "unowned inventory item".to_owned(),
        }
    );
    assert_eq!(equipment, before);

    // The definition belongs to slot 4, so a slot 1 request is refused.
    let refused = replay_equip(&mut inventory, 1, &command(42, 3, &[2, 1, 1]));
    let piece_045::Replay::Refused { error, equipment } = refused else {
        panic!("a mismatched slot must be refused");
    };
    assert_eq!(
        error,
        CosmeticError::Inventory {
            kind: std::io::ErrorKind::InvalidData,
            message: "equipment does not match profile or slot".to_owned(),
        }
    );
    assert_eq!(equipment, before);

    // The item-record shape (42/8) resolves an owned definition and tint instead of an instance.
    let refused = replay_equip(&mut inventory, 1, &command(42, 8, &[100, 12, 1, 1]));
    let piece_045::Replay::Refused { error, equipment } = refused else {
        panic!("a mismatched item-record tint must be refused");
    };
    assert_eq!(
        error,
        CosmeticError::Inventory {
            kind: std::io::ErrorKind::InvalidData,
            message: "unowned item record".to_owned(),
        }
    );
    assert_eq!(equipment, before);

    let replay = replay_equip(&mut inventory, 1, &command(42, 8, &[101, 12, 1, 4]));
    let piece_045::Replay::Accepted {
        reply, equipment, ..
    } = replay
    else {
        panic!("an owned item record must be accepted");
    };
    assert_eq!(reply.appearance.slot, 4);
    assert_eq!(equipment, before);

    // Requests outside the two recovered equip shapes are not silently treated as equips.
    let refused = replay_equip(&mut inventory, 1, &command(42, 2, &[4, 1]));
    let piece_045::Replay::Refused { error, .. } = refused else {
        panic!("an unequip request is not an equip replay");
    };
    assert_eq!(
        error,
        CosmeticError::NotEquipRequest {
            family: 42,
            opcode: 2
        }
    );
    assert_eq!(inventory.profile_equipment(1), before);
}

#[test]
fn restart_round_trip_preserves_profile_bytes() {
    let fixture = Fixture::new();
    let inventory = fixture.inventory();
    let before = capture_profiles(&inventory).unwrap();
    assert_eq!(before.len(), 2, "both supported profiles are captured");
    assert_eq!(before[0].profile_id, 1);
    assert!(!before[0].attachments.is_empty());
    assert_eq!(before[1].profile_id, 2);
    assert_eq!(before[1].equipment.len(), 16, "one saved equipment row");
    assert_eq!(before[1].attachments, 0i32.to_le_bytes());

    // Save -> load is exactly what a server restart does with the version-two snapshot.
    let state = fixture.state();
    let encoded = serde_json::to_vec(&state).unwrap();
    let decoded: InventoryState = serde_json::from_slice(&encoded).unwrap();
    let reloaded = fixture.inventory_with(decoded).unwrap();
    assert_profiles_identical(&before, &capture_profiles(&reloaded).unwrap()).unwrap();

    // The comparison is meaningful: a changed equip layout is reported.
    let mut changed = fixture.state();
    changed.items.push(item(4, 100, 0));
    changed.equipment[0].item_id = 4;
    let altered = fixture.inventory_with(changed).unwrap();
    assert_eq!(
        assert_profiles_identical(&before, &capture_profiles(&altered).unwrap()).unwrap_err(),
        CosmeticError::RestartDiffers { profile_id: 1 }
    );
}

#[test]
fn a_saved_equip_that_names_an_unowned_item_is_refused_at_load() {
    let fixture = Fixture::new();
    let mut state = fixture.state();
    state.equipment[0].item_id = 42;
    let error = match fixture.inventory_with(state) {
        Ok(_) => panic!("an unowned saved equipment row must not load"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        error.to_string().contains("invalid inventory equipment")
            || error.to_string().contains("equipment item is missing"),
        "unexpected load error: {error}"
    );
}

/// Private gate over the real saved snapshot. The evidence directory is private lab state, so the
/// test is ignored unless the environment points at it explicitly:
///
/// ```text
/// $env:OSFR_PIECE_045_CONTENT="%USERPROFILE%\Documents\Projects\CartoonMMO\lab\references\opening-anvil-placement-v1\content"
/// $env:OSFR_PIECE_045_SNAPSHOT="%USERPROFILE%\Documents\Projects\CartoonMMO\lab\state\opening-after-native-restart-203139.json"
/// $env:OSFR_PIECE_045_EVIDENCE_DIR="%USERPROFILE%\Documents\Projects\CartoonMMO\lab\reports\piece-045-<stamp>"
/// scripts\cargo.ps1 test --test piece_045 -- --ignored
/// ```
#[test]
#[ignore = "requires the preserved content directory and a saved native snapshot"]
fn private_snapshot_cosmetics_match_pinned_content() {
    #[derive(serde::Deserialize)]
    struct SnapshotFile {
        gameplay: Option<GameplayFile>,
    }
    #[derive(serde::Deserialize)]
    struct GameplayFile {
        inventory: InventoryState,
    }

    let content = std::env::var_os("OSFR_PIECE_045_CONTENT")
        .expect("OSFR_PIECE_045_CONTENT must point at the prepared content directory");
    let snapshot = std::env::var_os("OSFR_PIECE_045_SNAPSHOT")
        .expect("OSFR_PIECE_045_SNAPSHOT must point at a saved gameplay snapshot");
    let snapshot_bytes = fs::read(&snapshot).expect("readable snapshot");
    let file: SnapshotFile = serde_json::from_slice(&snapshot_bytes).expect("snapshot parses");
    let state = file
        .gameplay
        .expect("snapshot carries gameplay state")
        .inventory;
    let inventory = Inventory::load(Path::new(&content), Some(state.clone()))
        .expect("the prepared content accepts the saved inventory");

    let rows = inspect(&inventory).expect("every equipped cosmetic passes validation");
    assert!(
        !rows.is_empty(),
        "the saved snapshot has at least one equipped cosmetic"
    );
    for row in &rows {
        validate_appearance(&row.appearance).unwrap();
        assert_eq!(row.appearance.slot, row.slot);
        assert!(row.appearance.tint >= 0);
    }

    let before = capture_profiles(&inventory).unwrap();
    let encoded = serde_json::to_vec(&state).unwrap();
    let decoded: InventoryState = serde_json::from_slice(&encoded).unwrap();
    let reloaded = Inventory::load(Path::new(&content), Some(decoded))
        .expect("the saved inventory reloads after a restart");
    assert_profiles_identical(&before, &capture_profiles(&reloaded).unwrap()).unwrap();

    // Replay the recovered equip shapes against the real content.
    let mut replayed = Inventory::load(Path::new(&content), Some(state.clone())).unwrap();
    let mut accepted = 0usize;
    let mut refused = 0usize;
    let mut samples = Vec::new();
    for row in &rows {
        let request = command(42, 3, &[row.item_id, row.profile_id, row.slot]);
        let equipment_before = replayed.profile_equipment(row.profile_id);
        match replay_equip(&mut replayed, row.profile_id, &request) {
            piece_045::Replay::Accepted {
                reply,
                visible,
                equipment,
            } => {
                assert_eq!(reply.appearance.slot, row.slot);
                assert_eq!(
                    equipment, equipment_before,
                    "an accepted re-equip is byte-stable"
                );
                assert_eq!(
                    visible.map(|packet| packet.appearance),
                    Some(reply.appearance.clone()),
                    "both equip replies carry the same attachment"
                );
                if samples.len() < 8 {
                    samples.push(json!({
                        "profile_id": row.profile_id,
                        "slot": row.slot,
                        "item_id": row.item_id,
                        "definition_id": row.definition_id,
                        "model": reply.appearance.model,
                        "texture": reply.appearance.texture,
                        "tint_alias": reply.appearance.tint_alias,
                        "tint": reply.appearance.tint,
                        "effect": reply.appearance.effect,
                    }));
                }
                accepted += 1;
            }
            piece_045::Replay::Refused { error, equipment } => {
                assert_eq!(equipment, equipment_before, "a refusal is byte-stable");
                panic!("equipped cosmetic replay was refused: {error:?}");
            }
        }

        // A wrong-slot request for the same instance must be refused byte-stably.
        let foreign = match row.slot {
            1 => 4,
            _ => 1,
        };
        let bad = command(42, 3, &[row.item_id, row.profile_id, foreign]);
        let equipment_before = replayed.profile_equipment(row.profile_id);
        match replay_equip(&mut replayed, row.profile_id, &bad) {
            piece_045::Replay::Refused { equipment, .. } => {
                assert_eq!(equipment, equipment_before);
                refused += 1;
            }
            accepted => panic!("a mismatched slot was not refused: {accepted:?}"),
        }
    }
    assert!(accepted >= rows.len() && refused == rows.len());

    if let Some(directory) = std::env::var_os("OSFR_PIECE_045_EVIDENCE_DIR").map(PathBuf::from) {
        fs::create_dir_all(&directory).unwrap();
        let profiles: Vec<_> = before
            .iter()
            .map(|profile| {
                json!({
                    "profile_id": profile.profile_id,
                    "equipment_bytes": profile.equipment.len(),
                    "equipment_sha256": digest(&profile.equipment),
                    "attachment_bytes": profile.attachments.len(),
                    "attachment_sha256": digest(&profile.attachments),
                })
            })
            .collect();
        let report = json!({
            "consensus": "reconstructed offline replay over the preserved content and a saved native snapshot",
            "cosmetic_rows": rows.len(),
            "profiles": profiles,
            "replay": {"accepted": accepted, "refused": refused, "samples": samples},
            "restart": "profile equipment and attachment bytes are identical after save/load",
            "native_gate": "not exercised; the lead still owns native equip and restart acceptance",
        });
        fs::write(
            directory.join("rust-replay.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
    }
}
