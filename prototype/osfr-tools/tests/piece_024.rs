//! Work item 024: furniture catalogue relationships (`src/restoration/piece_024.rs`).
//!
//! The hermetic vectors below are small synthetic rows that mirror the shipped shapes.
//! The ignored private gate reads the preserved lab inputs and is exercised explicitly:
//!
//! * `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json`
//!   (sha256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`, AGPL-3.0,
//!   repository revision `9e082025391177d5a33ee18c714ddd06b457a695`)
//! * `lab/manifests/rust-asset-catalog.tsv` (24 preserved packs, 108,543 distinct names)
//! * `lab/reports/locale-id-to-text.json` (unmodified community locale reader export of the
//!   supplied client `locale/en_us_data.dat`/`.dir`)
//! * `lab/references/opening-anvil-placement-v1/content/inventory.json` (reconstructed definitions)
//!
//! This file is a `#[path]` include so it compiles before the lead registers the module in
//! `src/lib.rs`. Integration step for the lead: register `pub mod restoration::piece_024;` (or
//! move the module) and drop the include.

#[path = "../src/restoration/piece_024.rs"]
#[allow(dead_code)]
mod piece_024;

use piece_024::{
    Findings, RawIcon, RawItem, ReconstructedItem, analyze, collect_furniture, icon_asset_prefix,
    is_furniture_model,
};
use std::collections::BTreeSet;

fn row(
    id: i32,
    name_id: i32,
    description_id: Option<i32>,
    category_id: i32,
    model: &str,
) -> RawItem {
    RawItem {
        id,
        name_id: Some(name_id),
        description_id,
        icon: Some(RawIcon { id: Some(id) }),
        category_id: Some(category_id),
        model: Some(model.to_owned()),
    }
}

fn fixtures() -> (
    Vec<RawItem>,
    Vec<ReconstructedItem>,
    BTreeSet<String>,
    BTreeSet<i32>,
) {
    let stock = vec![
        // Fine stock furniture row.
        row(100, 1, Some(2), 53, "hsg_chair_lodge_01.adr"),
        // Furniture row whose declared model is absent from the pack catalogue and whose name and
        // description ids are absent from the locale export.
        row(200, 7, Some(8), 57, "hsg_pool_basic_01_complete.agr"),
        // Housing-category row that is not a furniture model.
        row(300, 3, Some(4), 53, "pet_penguin_01.adr"),
        // Non-housing row.
        row(400, 5, Some(6), 147, "tool_pickaxe_01.adr"),
    ];
    let reconstructed = vec![
        ReconstructedItem {
            id: 100,
            model: "hsg_chair_lodge_01.adr".to_owned(),
        },
        // Byte-identical duplicate must still be reported.
        ReconstructedItem {
            id: 100,
            model: "hsg_chair_lodge_01.adr".to_owned(),
        },
        // Same id as stock, different model.
        ReconstructedItem {
            id: 300,
            model: "hsg_bench_01.adr".to_owned(),
        },
        // No stock row at all.
        ReconstructedItem {
            id: 999,
            model: "hsg_newthing_01.adr".to_owned(),
        },
    ];
    let assets: BTreeSet<String> = [
        "hsg_chair_lodge_01.adr",
        "icon_item_hsg_chair_lodge_01_lodge-L1_64.dds",
    ]
    .iter()
    .map(|name| (*name).to_owned())
    .collect();
    let locale: BTreeSet<i32> = [1, 2, 3, 4, 5, 6].into_iter().collect();
    (stock, reconstructed, assets, locale)
}

#[test]
fn furniture_selection_uses_the_recovered_model_prefix_only() {
    assert!(is_furniture_model("hsg_block_01.adr"));
    assert!(is_furniture_model("HSG_Block_01.ADR"));
    assert!(!is_furniture_model("pet_penguin_01.adr"));
    assert!(!is_furniture_model(""));

    let (stock, ..) = fixtures();
    let furniture = collect_furniture(&stock);
    assert_eq!(
        furniture.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![100, 200]
    );
}

#[test]
fn icon_relationship_is_an_exact_model_stem_prefix() {
    assert_eq!(
        icon_asset_prefix("hsg_chair_lodge_01.adr").as_deref(),
        Some("icon_item_hsg_chair_lodge_01")
    );
    assert_eq!(icon_asset_prefix("not_a_model"), None);
}

#[test]
fn missing_assets_locale_and_icons_are_reported() {
    let (stock, reconstructed, assets, locale) = fixtures();
    let findings = analyze(&stock, &reconstructed, &assets, &locale);

    assert_eq!(findings.furniture, 2);
    assert_eq!(findings.missing_models, vec![200]);
    assert_eq!(findings.missing_locale, vec![200]);
    assert_eq!(findings.missing_icon_assets, vec![200]);
}

#[test]
fn stock_versus_reconstructed_delta_is_explicit() {
    let (stock, reconstructed, assets, locale) = fixtures();
    let Findings {
        stock_only,
        reconstructed_furniture,
        reconstructed_unknown,
        model_mismatches,
        duplicate_reconstructed,
        ..
    } = analyze(&stock, &reconstructed, &assets, &locale);

    assert_eq!(stock_only, vec![200]);
    // Three distinct ids arrive with a furniture model: 100 (matching stock), 300 (stock id exists
    // with a different, non-furniture model) and 999 (no stock row at all).
    assert_eq!(reconstructed_furniture, 3);
    assert_eq!(reconstructed_unknown, vec![999]);
    assert_eq!(model_mismatches, vec![300]);
    assert_eq!(duplicate_reconstructed, vec![100]);
}

#[test]
fn asset_lookups_are_case_insensitive() {
    let stock = vec![row(100, 1, Some(2), 53, "HSG_CHAIR_LODGE_01.ADR")];
    let assets: BTreeSet<String> = [
        "hsg_chair_lodge_01.adr",
        "ICON_ITEM_HSG_CHAIR_LODGE_01_x_64.dds",
    ]
    .iter()
    .map(|name| (*name).to_owned())
    .collect();
    let locale: BTreeSet<i32> = [1, 2].into_iter().collect();

    let findings = analyze(&stock, &[], &assets, &locale);
    assert!(findings.missing_models.is_empty());
    assert!(findings.missing_icon_assets.is_empty());
}

#[test]
fn stock_rows_whose_icon_block_has_no_id_still_parse() {
    // ORIGINAL quirk: the shipped catalogue has rows like item 15490 "Wolf" with
    // `"Icon": {"TintId": 536}` and no `Id`.
    let json = r#"[{"Comment":"Wolf","Id":15490,"NameId":6270,
        "Icon":{"TintId":536},"ModelName":"hsg_wolf_statue_01.adr"}]"#;
    let rows: Vec<RawItem> = serde_json::from_str(json).expect("stock row parses");
    let furniture = collect_furniture(&rows);
    assert_eq!(furniture.len(), 1);
    assert_eq!(furniture[0].icon_id, None);
}

#[test]
#[ignore = "requires preserved lab catalogue, assets, locale and reconstructed content"]
fn private_furniture_catalogue_matches_preserved_lab_inputs() {
    let stock_path = std::env::var_os("OSFR_PIECE_024_STOCK")
        .expect("OSFR_PIECE_024_STOCK must point at ClientItemDefinitions.json");
    let assets_path = std::env::var_os("OSFR_PIECE_024_ASSETS")
        .expect("OSFR_PIECE_024_ASSETS must point at rust-asset-catalog.tsv");
    let locale_path = std::env::var_os("OSFR_PIECE_024_LOCALE")
        .expect("OSFR_PIECE_024_LOCALE must point at locale-id-to-text.json");
    let content_path = std::env::var_os("OSFR_PIECE_024_CONTENT")
        .expect("OSFR_PIECE_024_CONTENT must point at the reconstructed inventory.json");

    let stock: Vec<RawItem> =
        serde_json::from_slice(&std::fs::read(&stock_path).expect("readable stock catalogue"))
            .expect("stock catalogue parses");
    let assets = piece_024::asset_names(
        &std::fs::read_to_string(&assets_path).expect("readable asset catalogue"),
    );
    let locale = piece_024::locale_ids(
        &std::fs::read_to_string(&locale_path).expect("readable locale export"),
    );
    let definitions = piece_024::reconstructed_definitions(
        &std::fs::read_to_string(&content_path).expect("readable reconstructed content"),
    );

    let findings = analyze(&stock, &definitions, &assets, &locale);
    assert_eq!(findings.furniture, 1167);
    assert_eq!(
        findings.missing_models,
        vec![9046, 16872, 16874, 18112, 18136]
    );
    assert_eq!(findings.missing_locale, vec![9046, 15978, 18167]);
    assert_eq!(findings.reconstructed_furniture, 31);
    assert_eq!(findings.duplicate_reconstructed, vec![17750, 27943]);
    assert!(findings.reconstructed_unknown.is_empty());
    assert!(findings.model_mismatches.is_empty());
    assert_eq!(findings.stock_only.len(), 1136);
}
