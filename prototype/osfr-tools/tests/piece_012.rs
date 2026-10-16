//! Piece 012 owned check: repeated Cooking recipe transactions stay atomic.
//!
//! The module is not registered in `lib.rs` yet; this test owns a `#[path]`
//! include so the lead can register it without a shared-file edit here.

#[path = "../src/restoration/piece_012.rs"]
mod piece_012;

use osfr_tools::cooking::{Cooking, CookingTransaction};
use piece_012::{
    FIRST_INSTANCE_ID, RECOVERED_MATERIALS, RECOVERED_PRODUCT, RecipeLedger,
    audit_repeated_transactions,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn fixture_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "osfr-piece-012-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

/// Synthetic, explicitly reconstructed content that mirrors the recovered
/// recipe contract. It is not a claim about the original phase tuning.
fn write_synthetic_content(dir: &Path) {
    let data = [
        [10, 8, 15, 2, 50, 1, 0, 0, 0, 0, 0],
        [24, 15, 1, 1, 100, 0, 0, 0, 0, 0, 0],
        [40, 2, 2, 0, 20, 20, 2, 3, 100, 200, 400],
        [49, 8, 3, 100, 500, 0, 0, 0, 0, 0, 0],
        [46, 0, 500, 0, 0, 0, 0, 0, 0, 0, 0],
        [46, 10, 0, 100, 0, 0, 0, 0, 0, 0, 0],
    ];
    let stages = [1, 2, 3, 6, 17, 7];
    let content = serde_json::json!({
        "schema_version": 1,
        "game_id": 46,
        "profile_id": 45,
        "name_id": 386023,
        "description_id": 20520,
        "icon_id": 916,
        "reconstructed_phase_values": true,
        "phase_notice": "Synthetic explicitly reconstructed phase rows for the piece 012 audit",
        "phases": data.iter().enumerate().map(|(i, d)| serde_json::json!({
            "id": 4601 + i as i32,
            "sequence": i as i32 + 1,
            "duration": 90,
            "objective_id": 0,
            "start_id": 0,
            "end_id": 0,
            "stage_type": stages[i],
            "data": d,
        })).collect::<Vec<_>>(),
        "recipe": {
            "consumes": RECOVERED_MATERIALS,
            "product_definition_id": RECOVERED_PRODUCT.0,
            "product_quantity": RECOVERED_PRODUCT.1,
        },
    });
    fs::write(
        dir.join("cooking.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .unwrap();
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_file(dir.join("cooking.json"));
    let _ = fs::remove_dir(dir);
}

fn recovered_transaction() -> CookingTransaction {
    CookingTransaction {
        consumes: RECOVERED_MATERIALS.to_vec(),
        grants: vec![RECOVERED_PRODUCT],
        // W60: recipe rows carry XP; current table values are 0 (R).
        experience_points: 0,
    }
}

#[test]
fn repeated_cooking_completions_consume_and_grant_exactly_once() {
    let dir = fixture_dir();
    write_synthetic_content(&dir);
    let report = audit_repeated_transactions(&dir, 4).unwrap();
    cleanup(&dir);

    assert_eq!(report.cycles, 4);
    assert_eq!(
        report.consumed,
        vec![(2862, 16), (2875, 16), (2884, 16), (2900, 16)]
    );
    assert_eq!(report.granted, vec![(35, 4)]);
    assert_eq!(report.live_replays_ignored, 4);
    assert_eq!(report.reload_replays_ignored, 4);
    assert_eq!(report.completed, 4);
    assert_eq!(report.next_instance_id, FIRST_INSTANCE_ID + 4);
    assert!(report.remaining_stock.is_empty());
}

#[test]
fn recipe_ledger_refuses_short_or_duplicated_stock_without_partial_consumption() {
    let mut short = RecipeLedger::seeded([(2862, 4), (2875, 4), (2884, 4), (2900, 3)]);
    assert!(short.apply(&recovered_transaction()).is_err());
    assert_eq!(
        [
            short.count(2862),
            short.count(2875),
            short.count(2884),
            short.count(2900)
        ],
        [4, 4, 4, 3]
    );
    assert_eq!(short.granted(RECOVERED_PRODUCT.0), 0);
    assert_eq!(short.applied(), 0);

    // Two separate line items for one material must be summed before the
    // availability check, otherwise a partly-available stack consumes twice.
    let mut duplicated = RecipeLedger::seeded([(2862, 6)]);
    let doubled = CookingTransaction {
        consumes: vec![(2862, 4), (2862, 4)],
        grants: vec![RECOVERED_PRODUCT],
        // W60: recipe rows carry XP; current table values are 0 (R).
        experience_points: 0,
    };
    assert!(duplicated.apply(&doubled).is_err());
    assert_eq!(duplicated.count(2862), 6);
    assert_eq!(duplicated.applied(), 0);

    let mut exact = RecipeLedger::seeded([(2862, 4), (2875, 4), (2884, 4), (2900, 4)]);
    exact.apply(&recovered_transaction()).unwrap();
    assert_eq!(exact.count(2862), 0);
    assert_eq!(exact.count(2900), 0);
    assert_eq!(exact.granted(RECOVERED_PRODUCT.0), 1);
    assert_eq!(exact.applied(), 1);
    // A repeated transaction has no materials left, so it cannot re-grant.
    assert!(exact.apply(&recovered_transaction()).is_err());
    assert_eq!(exact.granted(RECOVERED_PRODUCT.0), 1);
    assert_eq!(exact.applied(), 1);
}

#[test]
#[ignore = "requires the preserved private Cooking content under OSFR_COOKING_REFERENCE"]
fn private_original_cooking_content_matches_the_recovered_recipe_and_repeats_atomically() {
    let dir = std::env::var_os("OSFR_COOKING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-cooking-isolated")
        });
    let game = Cooking::load(&dir, None).unwrap();
    assert_eq!(game.requirements(), RECOVERED_MATERIALS.as_slice());
    drop(game);
    let report = audit_repeated_transactions(&dir, 3).unwrap();
    assert_eq!(report.cycles, 3);
    assert_eq!(
        report.consumed,
        vec![(2862, 12), (2875, 12), (2884, 12), (2900, 12)]
    );
    assert_eq!(report.granted, vec![(35, 3)]);
    assert_eq!(report.live_replays_ignored, 3);
    assert_eq!(report.reload_replays_ignored, 3);
    assert_eq!(report.completed, 3);
    assert_eq!(report.next_instance_id, FIRST_INSTANCE_ID + 3);
}
