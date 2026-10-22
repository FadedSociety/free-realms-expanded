//! Work item 049 acceptance: character appearance coverage and invalid-asset exclusions.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/working/OSFR.Client/OSFR Client/Resources/CharacterCreateCharacterTypes.txt` - the four
//!   creator character types (Human/Pixie x Male/Female) with locale name ids 430557-430560.
//! * `Resources/Models.txt` - the four rows with IS_VALID_FOR_PC = 1: 1 `human_m.adr`,
//!   2 `fairy_m.adr`, 60 `human_f.adr`, 61 `fairy_f.adr`; RACE_ID 1 is named `Human` and
//!   RACE_ID 2 is named `Pixie` (locale 2230/2231) although the model files say `fairy`.
//! * `Resources/CharacterCreateCategories.txt` and `CharacterCreateCustomizationCategories.txt` -
//!   the nine creator categories, including the Human/Male only `FaceHair` and the Pixie only
//!   `WingsPixie` scopes.
//! * `Resources/CharacterCreateCustomization*.txt` - 272 creator option rows (68 per character
//!   type). Every row references a `CharacterCreate/CharacterCreateItemMappings.txt` row, which
//!   in turn resolves to a type-12 `ItemDefinitions.txt` appearance definition, except clothing
//!   sets which resolve through `CharacterCreateItemClothingSets.txt` to three item definitions.
//! * Twelve of those 272 rows resolve into the class-1 `CATEGORY_ID 141` definition family
//!   (78086-78091): the two extra skin tones 7/8 and the four China head variants. The community
//!   public creator tables remove exactly those twelve rows, and
//!   `lab/reports/creator-appearance-availability.json` records that the Human male Caucasian
//!   head textures 7/8 are absent from the supplied client and the scanned public packs while
//!   1-6 match the original manifest CRC. Run `client-baseline-20261008T065152Z` failed on
//!   `human_m_head_caucasian_08.dds` (404).
//! * `ItemDefinitions.txt` holds 42 class-1 `CATEGORY_ID 141` definitions; 30 of them
//!   (78037-78066, the `FRC` hair family) are referenced by no creator row at all, and 49
//!   mapping rows overall are unreachable from the creator tables.
//!
//! BOUNDARIES: this module is recovery metadata plus pure lookups. It does not read the client,
//! does not send or accept a create request, does not reject a choice in production and does not
//! change any supplied resource. Server-side login validation derives the same sets from the
//! pinned creator tables (identity.rs, T444; tests/creator_allow_list.rs), and the China head
//! exclusion is community-table evidence, not a proven missing asset.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_049;` under `src/restoration/` and `src/lib.rs`.

#[path = "../src/restoration/piece_049.rs"]
#[allow(dead_code)]
mod piece_049;

use piece_049::{
    CHARACTER_TYPES, CREATOR_CATEGORIES, EXCLUDED_CREATOR_ROWS, EXCLUDED_HEAD_IDS,
    EXCLUDED_SKIN_TONE_IDS, ExclusionReason, OFFERED_MAPPING_ROWS, UNOFFERED_MAPPING_ROWS,
    excluded_rows_for, head_exclusion, skin_tone_exclusion, validate,
};

#[test]
fn four_creator_character_types_bind_to_the_pc_models() {
    assert_eq!(CHARACTER_TYPES.len(), 4);
    let rows: Vec<(i32, i32, i32, &str, i32)> = CHARACTER_TYPES
        .iter()
        .map(|row| {
            (
                row.id,
                row.name_id,
                row.model_id,
                row.model_file,
                row.race_id,
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (1, 430560, 61, "fairy_f.adr", 2),
            (2, 430558, 60, "human_f.adr", 1),
            (3, 430557, 1, "human_m.adr", 1),
            (4, 430559, 2, "fairy_m.adr", 2),
        ]
    );
    for row in CHARACTER_TYPES {
        assert!(matches!(row.gender_id, 1 | 2));
        assert!(matches!(row.race_id, 1 | 2));
    }
}

#[test]
fn categories_carry_one_mapping_type_and_the_recovered_row_counts() {
    assert_eq!(CREATOR_CATEGORIES.len(), 9);
    let mut types: Vec<i32> = CREATOR_CATEGORIES
        .iter()
        .map(|row| row.mapping_type)
        .collect();
    types.sort_unstable();
    assert_eq!(types, (0..=8).collect::<Vec<i32>>());

    let total: u32 = CREATOR_CATEGORIES
        .iter()
        .map(|row| row.rows_per_type.iter().map(|n| u32::from(*n)).sum::<u32>())
        .sum();
    let excluded: u32 = CREATOR_CATEGORIES
        .iter()
        .map(|row| {
            row.excluded_per_type
                .iter()
                .map(|n| u32::from(*n))
                .sum::<u32>()
        })
        .sum();
    assert_eq!(total, 253);
    assert_eq!(excluded, 12);

    let skin = CREATOR_CATEGORIES
        .iter()
        .find(|row| row.suffix == "SkinTone")
        .expect("SkinTone category");
    assert_eq!(skin.rows_per_type, [8, 8, 8, 8]);
    assert_eq!(skin.excluded_per_type, [2, 2, 2, 2]);
    let face_type = CREATOR_CATEGORIES
        .iter()
        .find(|row| row.suffix == "FaceType")
        .expect("FaceType category");
    assert_eq!(face_type.rows_per_type, [4, 4, 4, 4]);
    assert_eq!(face_type.excluded_per_type, [1, 1, 1, 1]);
    let face_hair = CREATOR_CATEGORIES
        .iter()
        .find(|row| row.suffix == "FaceHair")
        .expect("FaceHair category");
    assert_eq!(face_hair.scope_race, "Human");
    assert_eq!(face_hair.scope_gender, "Male");
    assert_eq!(face_hair.rows_per_type, [0, 0, 7, 0]);
    let wings = CREATOR_CATEGORIES
        .iter()
        .find(|row| row.suffix == "WingsPixie")
        .expect("WingsPixie category");
    assert_eq!(wings.scope_race, "Pixie");
    assert_eq!(wings.rows_per_type, [6, 0, 0, 6]);
}

#[test]
fn twelve_rows_are_excluded_and_they_are_the_class_one_family() {
    assert_eq!(EXCLUDED_CREATOR_ROWS.len(), 12);
    assert_eq!(excluded_rows_for("SkinTone").len(), 8);
    assert_eq!(excluded_rows_for("FaceType").len(), 4);
    assert_eq!(excluded_rows_for("Clothes").len(), 0);

    for row in EXCLUDED_CREATOR_ROWS
        .iter()
        .filter(|r| r.category == "SkinTone")
    {
        assert_eq!(row.reason, ExclusionReason::SkinTextureAbsent);
        assert!(matches!(row.definition_id, 78090 | 78091));
        assert!(matches!(row.mapping_id, 1 | 272));
    }
    for row in EXCLUDED_CREATOR_ROWS
        .iter()
        .filter(|r| r.category == "FaceType")
    {
        assert_eq!(row.reason, ExclusionReason::NotOfferedByPublicTables);
        assert!((78086..=78089).contains(&row.definition_id));
        assert!((268..=271).contains(&row.mapping_id));
        assert!(row.model_name.ends_with("_head_china.adr"));
    }
    // The skin rows carry the appearance index that selects the missing head texture.
    let mut indexes: Vec<i32> = EXCLUDED_CREATOR_ROWS
        .iter()
        .filter(|row| row.category == "SkinTone")
        .map(|row| row.appearance_index)
        .collect();
    indexes.sort_unstable();
    indexes.dedup();
    assert_eq!(indexes, vec![7, 8]);
}

#[test]
fn wire_level_lookups_reject_only_the_recovered_unsupported_ids() {
    for id in [1, 2, 3, 4, 5, 6] {
        assert_eq!(skin_tone_exclusion(id), None, "skin {id} is offered");
    }
    for id in EXCLUDED_SKIN_TONE_IDS {
        assert_eq!(
            skin_tone_exclusion(id),
            Some(ExclusionReason::SkinTextureAbsent)
        );
    }
    for id in [1, 2, 3] {
        assert_eq!(head_exclusion(id), None, "head {id} is offered");
    }
    for id in EXCLUDED_HEAD_IDS {
        assert_eq!(
            head_exclusion(id),
            Some(ExclusionReason::NotOfferedByPublicTables)
        );
    }
    // Id 4 exists in HeadMappings.txt but carries no creator row in either table; it stays a
    // reported gap instead of being silently accepted or rejected.
    assert_eq!(head_exclusion(4), None);
}

#[test]
fn unreachable_mapping_rows_are_reported_per_type() {
    assert_eq!(OFFERED_MAPPING_ROWS, 223);
    assert_eq!(
        UNOFFERED_MAPPING_ROWS,
        [(0, 16), (1, 30), (3, 1), (4, 1), (5, 1)]
    );
}

#[test]
fn module_invariants_hold() {
    assert_eq!(validate(), Ok(()));
}
