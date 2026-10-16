use osfr_tools::inventory::Inventory;
use osfr_tools::recipes::{
    RecipeCatalog, RecipeWireComponent, RecipeWireRow, encode_component_update, encode_recipe_add,
    encode_recipe_remove, encode_recipe_replace_all,
};
use std::collections::HashSet;
use std::path::PathBuf;

fn hex(bytes: &str) -> Vec<u8> {
    bytes
        .split_whitespace()
        .map(|byte| u8::from_str_radix(byte, 16).unwrap())
        .collect()
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
