use osfr_tools::inventory::Inventory;
use osfr_tools::recipes::{
    CraftingJob, RecipeCatalog, RecipeWireComponent, RecipeWireRow, encode_component_update,
    encode_recipe_add, encode_recipe_remove, encode_recipe_replace_all,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn hex(bytes: &str) -> Vec<u8> {
    bytes
        .split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).unwrap())
        .collect()
}

fn reference_content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn loaded() -> (RecipeCatalog, Inventory) {
    let content = reference_content();
    let inventory = Inventory::load(&content, None).unwrap();
    let catalog = RecipeCatalog::load_runtime(&content).unwrap();
    (catalog, inventory)
}

fn emitted_row(catalog: &RecipeCatalog, inventory: &Inventory, id: u32) -> RecipeWireRow {
    let rows = catalog.wire_rows(&HashSet::from([id]), inventory).unwrap();
    assert_eq!(rows.len(), 1, "recipe {id} should emit exactly one row");
    rows.into_iter().next().unwrap().1
}

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// A scratch copy of the catalogue fixture with the captured recipe-51 product
/// definition merged in. Item 21825 is in the deployed item catalogue but not
/// in the checked-in inventory fixture.
struct CapturedProductContent(PathBuf);

impl CapturedProductContent {
    fn new() -> Self {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "recipes-wire-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        fs::copy(
            reference_content().join("inventory.json"),
            path.join("inventory.json"),
        )
        .unwrap();
        let definition = definition_with_presentation(21_825, 38_663, 1_006, 0);
        fs::write(
            path.join("item-catalogue.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "definitions": [definition]})).unwrap(),
        )
        .unwrap();
        Self(path)
    }
}

impl Drop for CapturedProductContent {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Minimal `ClientItemDefinition` row whose name/icon/tint are readable by
/// `Inventory::presentation`: two zero-length strings follow the 46-byte
/// prefix, then the item type.
fn definition_with_presentation(id: i32, name: i32, icon: i32, tint: i32) -> serde_json::Value {
    let mut bytes = [0u8; 96];
    bytes[0..4].copy_from_slice(&id.to_le_bytes());
    bytes[4..8].copy_from_slice(&name.to_le_bytes());
    bytes[12..16].copy_from_slice(&icon.to_le_bytes());
    bytes[16..20].copy_from_slice(&tint.to_le_bytes());
    let digest = format!("{:x}", Sha256::digest(bytes));
    json!({
        "id": id,
        "slot": 0,
        "class_id": 0,
        "default_tint": 0,
        "max_stack": -1,
        "model": "",
        "texture": "",
        "tint_alias": "dyetint",
        "effect": 0,
        "wield_type": 0,
        "serialized_hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "sha256": digest,
    })
}

fn sample_row() -> RecipeWireRow {
    RecipeWireRow {
        id: 0x0102_0304,
        name_id: 0x1112_1314,
        icon_id: 0x2122_2324,
        tint_key: 0x3132_3334,
        description_id: 0x4142_4344,
        unknown_1c: 0x5152_5354,
        members_only: true,
        filter_type: 0x6162_6364,
        components: vec![RecipeWireComponent {
            name_id: 0x7172_7374,
            icon_id: 0x8182_8384,
            tint_key: 0x9192_9394,
            description_id: 0xa1a2_a3a4,
            required_count: 0xb1b2_b3b4,
            inventory_key: 0xc1c2_c3c4,
            unknown_20: 0xd1d2_d3d4,
            item_definition_id: 0xe1e2_e3e4,
        }],
        item_definition_id: 0xf1f2_f3f4,
        passes_requirement: true,
        unknown_2c: 0x0101_0101,
    }
}

#[test]
fn add_frame_uses_native_recipe_row_field_order() {
    let frame = encode_recipe_add(&sample_row()).unwrap();
    assert_eq!(
        frame,
        hex(r#"3f 00 01 04 03 02 01 14 13 12 11 24 23 22 21 34 33 32 31
               44 43 42 41 54 53 52 51 01 64 63 62 61 01 00 00 00
               74 73 72 71 84 83 82 81 94 93 92 91 a4 a3 a2 a1
               b4 b3 b2 b1 c4 c3 c2 c1 d4 d3 d2 d1 e4 e3 e2 e1
               f4 f3 f2 f1 01 01 01 01 01"#)
    );
}

#[test]
fn component_update_frame_uses_recipe_component_value_fields() {
    assert_eq!(
        encode_component_update(0x1122_3344, 0x5566_7788, 0x99aa_bbcc),
        hex("3f 00 02 44 33 22 11 88 77 66 55 cc bb aa 99")
    );
}

#[test]
fn remove_frame_carries_recipe_key() {
    assert_eq!(
        encode_recipe_remove(0x1122_3344),
        hex("3f 00 03 44 33 22 11")
    );
}

#[test]
fn replacement_frame_has_count_then_keyed_rows() {
    let row = RecipeWireRow {
        id: 1,
        name_id: 2,
        icon_id: 3,
        tint_key: 4,
        description_id: 5,
        unknown_1c: 6,
        members_only: false,
        filter_type: 7,
        components: Vec::new(),
        item_definition_id: 8,
        passes_requirement: false,
        unknown_2c: 9,
    };
    assert_eq!(
        encode_recipe_replace_all(&[(1, row)]).unwrap(),
        hex(r#"3f 00 05 01 00 00 00 01 00 00 00 01 00 00 00
               02 00 00 00 03 00 00 00 04 00 00 00 05 00 00 00
               06 00 00 00 00 07 00 00 00 00 00 00 00
               08 00 00 00 00 09 00 00 00"#)
    );
}

#[test]
fn learned_catalog_recipes_encode_add_and_replace_all() {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets");
    let inventory = Inventory::load(&content, None).unwrap();
    let catalog = RecipeCatalog::load_runtime(&content).unwrap();
    let learned = HashSet::from([11, 70]);
    let rows = catalog.wire_rows(&learned, &inventory).unwrap();
    assert_eq!(rows.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [11, 70]);
    let replace = encode_recipe_replace_all(&rows).unwrap();
    assert_eq!(&replace[..7], &[0x3f, 0, 5, 2, 0, 0, 0]);
    assert_eq!(&encode_recipe_add(&rows[1].1).unwrap()[..3], &[0x3f, 0, 1]);
}

#[test]
fn a_proven_recipe_row_matches_the_captured_product_fields() {
    // Captured `RecipeData` row 4: `PacketSendSelfToClient.json` Recipes[0] and
    // the raw bytes in lab/references/rust-gateway-reference/0007-login.bin
    // (body offset 189344; this row starts at file offset 189361). Its product
    // words are name 104, icon 462, tint 0 and item 21785. The pattern row 381
    // presents 333/2148/0, so the assertions pin product over pattern.
    let (catalog, inventory) = loaded();
    let row = emitted_row(&catalog, &inventory, 4);
    assert_eq!(
        (row.id, row.name_id, row.icon_id, row.tint_key),
        (4, 104, 462, 0)
    );
    assert_eq!(row.item_definition_id, 21_785);
    assert_eq!(row.filter_type, 9);
    assert_eq!(
        inventory.presentation(381).unwrap().name_id,
        333,
        "the pattern presentation is the old, wrong display"
    );
    assert_eq!(
        row.components
            .iter()
            .map(|component| (component.item_definition_id, component.required_count))
            .collect::<Vec<_>>(),
        [(2739, 5), (879, 3)],
        "ingredient rows keep the shipped data order"
    );
    let frame = encode_recipe_add(&row).unwrap();
    assert_eq!(
        frame,
        hex(r#"3f 00 01 04 00 00 00 68 00 00 00 ce 01 00 00 00
               00 00 00 00 00 00 00 00 00 00 00 00 09 00 00 00
               02 00 00 00 9d 55 00 00 ba 02 00 00 64 00 00 00
               00 00 00 00 05 00 00 00 b3 0a 00 00 00 00 00 00
               b3 0a 00 00 6e 03 00 00 03 07 00 00 00 00 00 00
               00 00 00 00 03 00 00 00 6f 03 00 00 00 00 00 00
               6f 03 00 00 19 55 00 00 01 00 00 00 00"#)
    );
    // Product words at the native row reader's offsets (row 0090a720).
    assert_eq!(&frame[3 + 4..3 + 8], &104_u32.to_le_bytes());
    assert_eq!(&frame[3 + 8..3 + 12], &462_u32.to_le_bytes());
    assert_eq!(&frame[3 + 12..3 + 16], &0_u32.to_le_bytes());
    assert_eq!(&frame[3 + 97..3 + 101], &21_785_u32.to_le_bytes());

    // Captured row 51 (body offset 190423) presents name 38663, icon 1006 and
    // item 21825. Its product definition is in the deployed item catalogue but
    // not the checked-in fixture, so merge the captured presentation instead of
    // dropping the second proven row.
    let fixture = CapturedProductContent::new();
    let captured_product = Inventory::load(&fixture.0, None).unwrap();
    let row = emitted_row(&catalog, &captured_product, 51);
    assert_eq!(
        (row.id, row.name_id, row.icon_id, row.tint_key),
        (51, 38_663, 1_006, 0)
    );
    assert_eq!(row.item_definition_id, 21_825);
    assert_eq!(row.filter_type, 10);
    assert_eq!(
        captured_product.presentation(861).unwrap().name_id,
        1_528,
        "the pattern presentation is not the captured product"
    );
}

#[test]
fn a_missing_product_presentation_falls_back_to_the_pattern() {
    let (catalog, inventory) = loaded();
    assert!(!inventory.has_definition(21_825));
    let row = emitted_row(&catalog, &inventory, 51);
    assert_eq!(
        (row.id, row.name_id, row.icon_id, row.tint_key),
        (51, 1_528, 2_148, 0),
        "row 51 shows pattern item 861 when product 21825 is unavailable"
    );
    assert_eq!(row.item_definition_id, 861);
    assert_eq!(row.components.len(), 4);
}

#[test]
fn a_missing_pattern_presentation_still_uses_the_product() {
    let (catalog, inventory) = loaded();
    assert!(!inventory.has_definition(843));
    let row = emitted_row(&catalog, &inventory, 33);
    // Captured row 33 (body offset 189573) carries 38645/1005/0 and item 21807.
    assert_eq!(
        (row.id, row.name_id, row.icon_id, row.tint_key),
        (33, 38_645, 1_005, 0)
    );
    assert_eq!(row.item_definition_id, 21_807);
}

#[test]
fn both_missing_presents_zero_fields_but_keeps_the_recipe_and_product_ids() {
    let (catalog, inventory) = loaded();
    assert!(!inventory.has_definition(848));
    assert!(!inventory.has_definition(21_812));
    let row = emitted_row(&catalog, &inventory, 38);
    assert_eq!(
        (row.id, row.name_id, row.icon_id, row.tint_key),
        (38, 0, 0, 0)
    );
    assert_eq!(row.item_definition_id, 848, "pattern id is the fallback");
    assert!(!row.components.is_empty());
    assert!(
        catalog.records.iter().any(|recipe| {
            recipe.recipe_id == Some(38) && recipe.output.definition_id == 21_812
        })
    );
    let frame = encode_recipe_add(&row).unwrap();
    assert_eq!(frame.len(), 3 + 42 + 32 * row.components.len());
}

#[test]
fn the_filter_column_is_unchanged_or_equals_the_proven_capture() {
    // Captured rows group by job: Miner rows 4/5 carry filter 9, Chef rows
    // 33-59 carry 10 and Blacksmith rows 155/165 carry 12.
    assert_eq!(
        (
            CraftingJob::Chef.recipe_filter(),
            CraftingJob::Blacksmith.recipe_filter(),
            CraftingJob::Miner.recipe_filter(),
        ),
        (10, 12, 9)
    );
    let (catalog, inventory) = loaded();
    for (id, filter) in [(4, 9), (5, 9), (33, 10), (51, 10), (11, 12)] {
        assert_eq!(
            emitted_row(&catalog, &inventory, id).filter_type,
            filter,
            "recipe {id}"
        );
    }
}

#[test]
fn unlearned_and_unknown_recipes_are_not_emitted() {
    let (catalog, inventory) = loaded();
    assert!(
        catalog
            .wire_rows(&HashSet::new(), &inventory)
            .unwrap()
            .is_empty()
    );
    assert!(
        catalog
            .wire_rows(&HashSet::from([999_999]), &inventory)
            .unwrap()
            .is_empty()
    );
    let rows = catalog
        .wire_rows(&HashSet::from([4, 999_999]), &inventory)
        .unwrap();
    assert_eq!(rows.iter().map(|(id, _)| *id).collect::<Vec<_>>(), [4]);
}

#[test]
fn the_existing_frame_layout_and_length_are_unchanged() {
    // Native row 0090a720: 42 fixed bytes plus 32 per component. Operation 1
    // adds the 3-byte 3f/00/op header; operation 5 adds a u32 count before each
    // keyed row.
    let (catalog, inventory) = loaded();
    let mut rows = Vec::new();
    for id in [4, 11, 33, 51] {
        let row = emitted_row(&catalog, &inventory, id);
        let add = encode_recipe_add(&row).unwrap();
        assert_eq!(add.len(), 3 + 42 + 32 * row.components.len(), "row {id}");
        assert_eq!(&add[..3], &[0x3f, 0, 1]);
        let passes_at = 3 + 37 + 32 * row.components.len();
        assert_eq!(add[passes_at], u8::from(row.passes_requirement));
        assert_eq!(&add[passes_at + 1..], &row.unknown_2c.to_le_bytes());
        rows.push((id, row));
    }
    let replace = encode_recipe_replace_all(&rows).unwrap();
    let expected = 3
        + 4
        + rows
            .iter()
            .map(|(_, row)| 4 + 42 + 32 * row.components.len())
            .sum::<usize>();
    assert_eq!(replace.len(), expected);
    assert_eq!(&replace[..7], &[0x3f, 0, 5, 4, 0, 0, 0]);
}
