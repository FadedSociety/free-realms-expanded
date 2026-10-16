//! Work item 097 acceptance: catalogue asset-reference validation.
//!
//! ORIGINAL evidence the vectors mirror (all outside this test):
//! * `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json` - 58,795 stock
//!   item rows with `NameId`, `DescriptionId`, `Class`, `CategoryId`, `ModelName`, `Type` and
//!   `Param1` cells (AGPL-3.0, revision `9e082025391177d5a33ee18c714ddd06b457a695`).
//! * `lab/references/FreeRealms-Legacy/src/Resources/{ItemClasses,ItemCategories,Models}.txt` -
//!   the sibling reference tables the item rows point at.
//! * `lab/manifests/rust-asset-catalog.tsv` - 108,543 names read from the 24 preserved packs.
//! * `lab/reports/locale-id-to-text.json` - the unmodified community locale export, 85,932 ids.
//!
//! The hermetic vectors are small synthetic rows that mirror the shipped shapes. The ignored
//! private gate reads the preserved lab inputs and is exercised explicitly; it is the only place
//! that asserts the measured catalogue counts.
//!
//! RECONSTRUCTED rules, labelled in the module: the four PC model variants used to expand
//! `<race>`/`<gender>` templates, and the per-type `Param1`-to-`Models.txt` relationship, which is
//! derived from the catalogue's own coverage instead of a recovered table.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_097;` under `src/restoration/mod.rs`/`src/lib.rs`.

#[path = "../src/restoration/piece_097.rs"]
#[allow(dead_code)]
mod piece_097;

use piece_097::{
    CategoryRow, ClassRow, ModelReference, ModelRow, RawItem, analyze, analyze_tables, asset_names,
    classify_model, expand_race_gender, locale_ids, parse_item_categories, parse_item_classes,
    parse_models,
};
use std::collections::{BTreeMap, BTreeSet};

fn item(
    id: i32,
    name_id: Option<i32>,
    description_id: Option<i32>,
    model: Option<&str>,
    class_id: Option<i32>,
    category_id: Option<i32>,
    item_type: Option<i32>,
    param1: Option<i32>,
) -> RawItem {
    RawItem {
        id,
        name_id,
        description_id,
        model: model.map(str::to_owned),
        class_id,
        category_id,
        item_type,
        param1,
    }
}

fn class_row(id: i32, name_id: i32, profile_name_id: i32) -> ClassRow {
    ClassRow {
        id,
        name_id,
        profile_name_id,
    }
}

fn category_row(id: i32, name_id: i32) -> CategoryRow {
    CategoryRow { id, name_id }
}

fn model_row(id: i32, file: &str, is_valid_for_pc: bool) -> ModelRow {
    ModelRow {
        id,
        file: file.to_owned(),
        race_id: 1,
        is_valid_for_pc,
    }
}

#[test]
fn model_references_classify_by_template_tokens() {
    assert_eq!(
        classify_model("worker_<race>_<gender>_head_hardhatwlight.adr"),
        ModelReference::RaceGender("worker_<race>_<gender>_head_hardhatwlight.adr".to_owned())
    );
    assert_eq!(
        classify_model("pet_dog_<descriptor>_head_turban.adr"),
        ModelReference::Unmapped {
            name: "pet_dog_<descriptor>_head_turban.adr".to_owned(),
            tokens: vec!["<descriptor>".to_owned()],
        }
    );
    assert_eq!(
        classify_model("archer_{Error with ID}"),
        ModelReference::Literal("archer_{error with id}".to_owned())
    );
    assert_eq!(
        classify_model("hsg_pool_basic_01_complete.agr"),
        ModelReference::Literal("hsg_pool_basic_01_complete.agr".to_owned())
    );
}

#[test]
fn race_gender_templates_expand_to_the_four_pc_variants() {
    // Only the recovered `<race>`/`<gender>` tokens are substituted; an unrecovered token such as
    // `<hairtype>` stays in place and the surviving variants are deduplicated.
    assert_eq!(
        expand_race_gender("<race>_m_hair_afro_<hairtype>.adr"),
        vec![
            "human_m_hair_afro_<hairtype>.adr".to_owned(),
            "fairy_m_hair_afro_<hairtype>.adr".to_owned(),
        ]
    );
    assert_eq!(
        expand_race_gender("worker_<race>_<gender>_head_light.adr"),
        vec![
            "worker_human_m_head_light.adr".to_owned(),
            "worker_human_f_head_light.adr".to_owned(),
            "worker_fairy_m_head_light.adr".to_owned(),
            "worker_fairy_f_head_light.adr".to_owned(),
        ]
    );
}

#[test]
fn reference_tables_parse_and_reject_malformed_rows() {
    let classes = parse_item_classes(
        "#ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^PROFILE_NAME_ID\n1^2921^38^0^16^2512\n",
    )
    .expect("valid class table");
    assert_eq!(classes.get(&1), Some(&class_row(1, 2921, 2512)));

    let categories = parse_item_categories(
        "#ID^NAME_ID^ICON_ID^UNKNOWN^UNKNOWN_2^DISPLAY\n308^435549^36680^1^1^1\n",
    )
    .expect("valid category table");
    assert_eq!(categories.get(&308), Some(&category_row(308, 435549)));

    let models = parse_models(
        "#ID^MODEL_FILE_NAME^RACE_ID^SCALE^DESCRIPTION^GENDER^AGE^DESCRIPTOR^MATERIAL_TYPE^WATER_DISPLACEMENT^TELEPORT_EFFECT_ID^CAMERA_DISTANCE^CAMERA_ANGLE^IS_VALID_FOR_PC^NAMEPLATE_OFFSET^CAPSULE_HEIGHT^RADIUS^\n\
         1^human_m.adr^1^0.0^^1^0^human^1^0.0^0^0.0^0.0^1^0.0^1.9^0.0^\n",
    )
    .expect("valid model table");
    assert_eq!(models.get(&1), Some(&model_row(1, "human_m.adr", true)));

    assert!(parse_item_classes("1^2^3^4^5\n").is_err());
    assert!(parse_item_categories("1^two^3^4^5^6\n").is_err());
    assert!(parse_models("1^human_m.adr^1^0.0^^1^0^\n").is_err());
}

#[test]
fn dangling_locale_class_category_and_model_references_are_reported() {
    let items = vec![
        // Fully resolvable row.
        item(
            1,
            Some(10),
            Some(11),
            Some("tool_pickaxe_01.adr"),
            Some(1),
            Some(5),
            Some(1),
            None,
        ),
        // Dangling locale ids, a literal model absent from the packs and an unknown class.
        item(
            2,
            Some(99),
            Some(98),
            Some("missing_thing_01.adr"),
            Some(7),
            Some(5),
            Some(1),
            None,
        ),
        // No NameId cell and a negative sentinel description id: reported, not dangling.
        item(3, None, Some(-1), None, None, None, None, None),
    ];
    let classes = [(1, class_row(1, 10, 12))].into_iter().collect();
    let categories = [(5, category_row(5, 11))].into_iter().collect();
    let models = [(1, model_row(1, "human_m.adr", true))]
        .into_iter()
        .collect();
    let assets: BTreeSet<String> = ["tool_pickaxe_01.adr", "human_m.adr"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let locale: BTreeSet<i32> = [10, 11, 12].into_iter().collect();
    let tables = analyze_tables(classes, categories, models, assets, locale);

    let findings = analyze(&items, &tables);
    assert_eq!(findings.items, 3);
    assert!(findings.duplicate_item_ids.is_empty());
    assert_eq!(findings.name_id_absent, 1);
    assert_eq!(findings.name_id_unset, Vec::<i32>::new());
    assert_eq!(findings.missing_name_ids, vec![(2, 99)]);
    assert_eq!(findings.description_id_unset, vec![3]);
    assert_eq!(findings.missing_description_ids, vec![(2, 98)]);
    assert_eq!(
        findings.missing_literal_models,
        vec![(2, "missing_thing_01.adr".to_owned())]
    );
    assert_eq!(findings.missing_item_classes, vec![7]);
    assert!(findings.missing_item_categories.is_empty());
}

#[test]
fn unmapped_templates_are_reported_not_asserted_missing() {
    let items = vec![item(
        1,
        Some(10),
        None,
        Some("pet_cat_<descriptor>_head_crown.adr"),
        None,
        None,
        Some(3),
        None,
    )];
    let tables = analyze_tables(
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeSet::new(),
        [10].into_iter().collect(),
    );
    let findings = analyze(&items, &tables);
    assert_eq!(findings.unmapped_template_rows, 1);
    assert_eq!(
        findings.unmapped_template_tokens,
        [("<descriptor>".to_owned(), 1usize)].into_iter().collect()
    );
    assert!(findings.missing_literal_models.is_empty());
    assert!(findings.missing_race_gender_models.is_empty());
}

#[test]
fn param1_model_conflicts_are_reported_only_for_mixed_relationship_types() {
    let items = vec![
        // Type 12 has one resolved and one unresolved Param1 value: the type demonstrably uses
        // model ids, so the unresolved row is a conflict.
        item(1, None, None, None, None, None, Some(12), Some(1)),
        item(2, None, None, None, None, None, Some(12), Some(99)),
        // Type 19 resolves every Param1 value it carries: no conflict.
        item(3, None, None, None, None, None, Some(19), Some(2)),
        item(4, None, None, None, None, None, Some(19), Some(2)),
        // Type 14 resolves no Param1 value at all, so the relationship is not established for it
        // and its unresolved rows stay in coverage rather than in the conflict list.
        item(5, None, None, None, None, None, Some(14), Some(99)),
    ];
    let models = [
        (1, model_row(1, "human_m.adr", true)),
        (2, model_row(2, "fairy_m.adr", true)),
    ]
    .into_iter()
    .collect();
    let tables = analyze_tables(
        BTreeMap::new(),
        BTreeMap::new(),
        models,
        BTreeSet::new(),
        BTreeSet::new(),
    );
    let findings = analyze(&items, &tables);
    assert_eq!(findings.param1_conflicts, vec![(2, 12, 99)]);
    assert_eq!(findings.param1_conflict_types, vec![12]);
    assert_eq!(
        findings
            .param1_coverage
            .get(&Some(12))
            .map(|c| (c.present, c.resolved)),
        Some((2, 1))
    );
    assert_eq!(
        findings
            .param1_coverage
            .get(&Some(19))
            .map(|c| (c.present, c.resolved)),
        Some((2, 2))
    );
    assert_eq!(
        findings
            .param1_coverage
            .get(&Some(14))
            .map(|c| (c.present, c.resolved)),
        Some((1, 0))
    );
}

#[test]
fn duplicate_item_ids_and_reference_table_gaps_are_reported() {
    let items = vec![
        item(1, Some(10), None, None, Some(1), None, None, None),
        item(1, Some(10), None, None, Some(1), None, None, None),
    ];
    let classes = [(1, class_row(1, 97, 98))].into_iter().collect();
    let categories = [(5, category_row(5, 99))].into_iter().collect();
    let models = [(7, model_row(7, "dwarf_m_miner_01.adr", false))]
        .into_iter()
        .collect();
    let assets: BTreeSet<String> = ["human_m.adr"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let locale: BTreeSet<i32> = [10].into_iter().collect();
    let tables = analyze_tables(classes, categories, models, assets, locale);

    let findings = analyze(&items, &tables);
    assert_eq!(findings.duplicate_item_ids, vec![1]);
    assert_eq!(findings.class_locale_gaps, vec![1]);
    assert_eq!(findings.category_locale_gaps, vec![5]);
    assert_eq!(findings.model_asset_gaps, vec![7]);
    assert_eq!(findings.model_rows, 1);
    assert_eq!(findings.models_valid_for_pc, 0);
    assert_eq!(findings.param1_coverage.len(), 0);
}

#[test]
fn asset_and_locale_lookups_are_case_insensitive() {
    let assets = asset_names(
        "pack\tentry\tname\toffset\tbytes\tcrc32\tpath\n\
                              AssetsW_000.pack\t0\tHSG_Chair_01.ADR\t1\t2\t00\tpack_00/000000.adr\n",
    );
    assert!(assets.contains("hsg_chair_01.adr"));
    let locale = locale_ids(r#"[{"id":10,"text":"Hat"},{"id":11,"text":"Cap"}]"#);
    assert!(locale.contains(&10) && locale.contains(&11));
}

#[test]
#[ignore = "requires the preserved lab catalogue, reference tables, packs and locale export"]
fn private_catalogue_references_match_preserved_lab_inputs() {
    let stock_path = std::env::var_os("OSFR_PIECE_097_STOCK")
        .expect("OSFR_PIECE_097_STOCK must point at ClientItemDefinitions.json");
    let assets_path = std::env::var_os("OSFR_PIECE_097_ASSETS")
        .expect("OSFR_PIECE_097_ASSETS must point at rust-asset-catalog.tsv");
    let locale_path = std::env::var_os("OSFR_PIECE_097_LOCALE")
        .expect("OSFR_PIECE_097_LOCALE must point at locale-id-to-text.json");
    let classes_path = std::env::var_os("OSFR_PIECE_097_CLASSES")
        .expect("OSFR_PIECE_097_CLASSES must point at ItemClasses.txt");
    let categories_path = std::env::var_os("OSFR_PIECE_097_CATEGORIES")
        .expect("OSFR_PIECE_097_CATEGORIES must point at ItemCategories.txt");
    let models_path = std::env::var_os("OSFR_PIECE_097_MODELS")
        .expect("OSFR_PIECE_097_MODELS must point at Models.txt");

    let items: Vec<RawItem> =
        serde_json::from_slice(&std::fs::read(&stock_path).expect("readable stock catalogue"))
            .expect("stock catalogue parses");
    let classes =
        parse_item_classes(&std::fs::read_to_string(&classes_path).expect("readable class table"))
            .expect("class table parses");
    let categories = parse_item_categories(
        &std::fs::read_to_string(&categories_path).expect("readable category table"),
    )
    .expect("category table parses");
    let models =
        parse_models(&std::fs::read_to_string(&models_path).expect("readable model table"))
            .expect("model table parses");
    let assets =
        asset_names(&std::fs::read_to_string(&assets_path).expect("readable asset catalogue"));
    let locale =
        locale_ids(&std::fs::read_to_string(&locale_path).expect("readable locale export"));
    let tables = analyze_tables(classes, categories, models, assets, locale);

    let findings = analyze(&items, &tables);
    assert_eq!(findings.items, 58795);
    assert!(findings.duplicate_item_ids.is_empty());
    assert_eq!(findings.name_id_absent, 250);
    assert_eq!(findings.name_id_unset.len(), 4);
    assert_eq!(findings.missing_name_ids.len(), 113);
    assert_eq!(findings.missing_name_ids[0], (15978, 21674));
    assert_eq!(findings.description_id_absent, 3533);
    assert_eq!(findings.description_id_unset.len(), 7);
    assert_eq!(findings.missing_description_ids.len(), 16);
    assert_eq!(findings.literal_model_rows, 9209);
    assert_eq!(findings.missing_literal_models.len(), 21);
    assert_eq!(findings.race_gender_model_rows, 31962);
    assert_eq!(findings.missing_race_gender_models.len(), 5);
    assert_eq!(findings.unmapped_template_rows, 8617);
    assert_eq!(findings.unmapped_template_tokens.len(), 2);
    assert!(findings.missing_item_classes.is_empty());
    assert_eq!(findings.missing_item_categories.len(), 29);
    assert_eq!(findings.class_locale_gaps, vec![999]);
    assert!(findings.category_locale_gaps.is_empty());
    assert_eq!(findings.model_rows, 3743);
    assert_eq!(findings.models_valid_for_pc, 4);
    assert_eq!(findings.model_asset_gaps.len(), 187);
    assert_eq!(findings.param1_conflict_types, vec![1, 2, 6, 14, 19]);
    assert_eq!(findings.param1_conflicts.len(), 340);
    assert_eq!(
        findings
            .param1_coverage
            .get(&Some(12))
            .map(|c| (c.present, c.resolved)),
        Some((541, 541))
    );
}
