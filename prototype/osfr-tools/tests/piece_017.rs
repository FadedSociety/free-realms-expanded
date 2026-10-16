//! Piece 017: minigame disconnect recovery contract.
//!
//! The audit module is included by path until the lead registers it in
//! `lib.rs`. Private lab evidence is read only; no save, client or movie is
//! touched by this file.
#[path = "../src/restoration/piece_017.rs"]
mod piece_017;

use piece_017::{ClearedRun, audit_conservation, audit_recovery};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

fn item(definition: i32, id: i32, count: u32) -> Value {
    json!({
        "definition_id": definition,
        "tint": 229,
        "id": id,
        "count": count,
        "consumed_count": 0,
        "last_cast_time": 2147483647,
        "rental": null,
        "ability_count": 0,
        "activate_enabled": false
    })
}

fn movie(active: Option<(i32, &str)>, next: i32, completed: u32) -> Value {
    json!({
        "version": 1,
        "next_instance_id": next,
        "completed": completed,
        "active": active.map(|(instance_id, phase)| json!({
            "instance_id": instance_id,
            "phase": phase,
            "data_sent": phase != "Offered"
        }))
    })
}

/// Persisted gameplay shape shared with `osfr_tools::gameplay::GameplayState`.
fn state(coins: u32, items: Vec<Value>, next: i32, active: Option<(i32, &str)>) -> Value {
    json!({
        "inventory": {"coins": coins, "items": items, "equipment": []},
        "quests": {"version": 1, "quests": {}, "trial_best_ms": null, "trial_completions": 0},
        "cooking": movie(active, next, 0),
        "smelting": movie(None, 77_000_001, 0),
        "forging": movie(None, 139_000_001, 0)
    })
}

fn live_cooking() -> Value {
    state(
        110,
        vec![item(2862, 11, 4), item(2884, 12, 4), item(2900, 13, 4)],
        46_000_002,
        Some((46_000_001, "Playing")),
    )
}

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

#[test]
fn recovery_boundary_reports_the_cancelled_run() {
    let before = live_cooking();
    let mut after = before.clone();
    after["cooking"]["active"] = Value::Null;
    let audit = audit_recovery(&before, &after).unwrap();
    assert_eq!(
        audit.cleared,
        vec![ClearedRun {
            service: "cooking",
            instance_id: 46_000_001,
            phase: "Playing".into()
        }]
    );
}

#[test]
fn recovery_boundary_rejects_a_surviving_run() {
    let before = live_cooking();
    let mut after = before.clone();
    after["cooking"]["active"]["phase"] = json!("Offered");
    let error = audit_recovery(&before, &after).unwrap_err();
    assert!(
        error.contains("cooking") && error.contains("46000001"),
        "{error}"
    );
}

#[test]
fn conservation_rejects_lost_materials() {
    let before = live_cooking();
    let mut after = before.clone();
    after["cooking"]["active"] = Value::Null;
    after["inventory"]["items"][0]["count"] = json!(3);
    let error = audit_recovery(&before, &after).unwrap_err();
    assert!(error.contains("2862"), "{error}");
}

#[test]
fn conservation_rejects_a_cancel_reward() {
    let before = live_cooking();
    let mut after = before.clone();
    after["cooking"]["active"] = Value::Null;
    after["inventory"]["items"]
        .as_array_mut()
        .unwrap()
        .push(item(35, 90, 1));
    let error = audit_recovery(&before, &after).unwrap_err();
    assert!(error.contains("35"), "{error}");
}

#[test]
fn conservation_rejects_wallet_and_equipment_changes() {
    let before = live_cooking();
    let mut paid = before.clone();
    paid["cooking"]["active"] = Value::Null;
    paid["inventory"]["coins"] = json!(130);
    assert!(
        audit_recovery(&before, &paid)
            .unwrap_err()
            .contains("coins")
    );
    let mut equipped = before.clone();
    equipped["cooking"]["active"] = Value::Null;
    equipped["inventory"]["equipment"] = json!([{"profile_id": 16, "item_id": 11, "slot": 0}]);
    assert!(
        audit_recovery(&before, &equipped)
            .unwrap_err()
            .contains("equipment")
    );
}

#[test]
fn conservation_rejects_a_regressed_or_reused_attempt_counter() {
    let before = live_cooking();
    let mut regressed = before.clone();
    regressed["cooking"]["active"] = Value::Null;
    regressed["cooking"]["next_instance_id"] = json!(46_000_001);
    assert!(
        audit_recovery(&before, &regressed)
            .unwrap_err()
            .contains("next_instance_id")
    );
    let mut reused = before.clone();
    reused["cooking"]["active"] = json!({
        "instance_id": 46_000_001,
        "phase": "Playing",
        "data_sent": true
    });
    reused["cooking"]["next_instance_id"] = json!(46_000_002);
    assert!(
        audit_conservation(&before, &reused)
            .unwrap_err()
            .contains("reused")
    );
}

#[test]
fn conservation_accepts_a_fresh_attempt_replacing_a_stale_offer() {
    let before = state(
        110,
        vec![item(2862, 11, 4)],
        46_000_002,
        Some((46_000_001, "Offered")),
    );
    let mut after = before.clone();
    after["cooking"]["active"] = json!({
        "instance_id": 46_000_002,
        "phase": "Playing",
        "data_sent": true
    });
    after["cooking"]["next_instance_id"] = json!(46_000_003);
    let audit = audit_conservation(&before, &after).unwrap();
    assert_eq!(audit.cleared.len(), 1);
    assert_eq!(audit.cleared[0].instance_id, 46_000_001);
    // Recovery is stricter: the persisted run must not survive a reload.
    assert!(audit_recovery(&before, &after).is_err());
}

#[test]
#[ignore = "reads private lab evidence; no writes"]
fn private_forging_abandon_restart_pair_conserves_materials() {
    let state = lab().join("state");
    let before: Value = serde_json::from_slice(
        &fs::read(state.join("opening-after-smelting-forging-incomplete-173814.json")).unwrap(),
    )
    .unwrap();
    let after: Value = serde_json::from_slice(
        &fs::read(state.join("opening-after-smelting-restart-before-retry-181038.json")).unwrap(),
    )
    .unwrap();
    let audit = audit_recovery(&before, &after).unwrap();
    assert_eq!(
        audit.cleared,
        vec![ClearedRun {
            service: "forging",
            instance_id: 139_000_001,
            phase: "Playing".into()
        }]
    );
    // The reviewed gameplay objects differ in exactly the cleared run (the
    // character pose in the two captures is a separate native round trip);
    // prove it here so a rewritten evidence file cannot pass silently.
    let mut expected = before.clone();
    expected["gameplay"]["forging"]["active"] = Value::Null;
    assert_eq!(expected["gameplay"], after["gameplay"]);
}

#[test]
#[ignore = "reads private lab evidence; no writes"]
fn private_cooking_native_midround_disconnect_conserves_materials() {
    let run = lab().join("runs/client-baseline-20261008T162238Z");
    let before: Value =
        serde_json::from_slice(&fs::read(run.join("character-before.json")).unwrap()).unwrap();
    let after: Value =
        serde_json::from_slice(&fs::read(run.join("character-after.json")).unwrap()).unwrap();
    let audit = audit_conservation(&before, &after).unwrap();
    assert_eq!(audit.cleared.len(), 1);
    assert_eq!(
        (
            before["gameplay"]["cooking"]["active"]["instance_id"].as_i64(),
            after["gameplay"]["cooking"]["active"]["instance_id"].as_i64(),
        ),
        (Some(46_000_001), Some(46_000_002))
    );
    assert_eq!(after["gameplay"]["inventory"]["coins"], json!(110));
}
