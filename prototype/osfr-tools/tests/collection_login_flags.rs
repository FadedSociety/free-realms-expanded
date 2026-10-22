//! T448: each character's saved collection ledger sets its own login collected flags.
//!
//! Driven through the real `Gateway` with the synthetic reference. Its self-data template
//! is given two collections whose every entry is flagged collected, as in the capture
//! (`lab/references/rust-gateway-reference/0007-login.bin`): a 4-row one that includes a
//! key above `i32::MAX`, and the 8-entry one the client needs (`00c31f00`).
#[path = "common/gateway_reference.rs"]
mod gateway_reference;

use gateway_reference::{Reference, initial_state};
use osfr_tools::{
    gameplay::GameplayState,
    gateway::Gateway,
    progression_extras::ProgressionExtras,
    startup_selfdata::{self as selfdata, CollectionKeyedRow, CollectionRecord, RewardBundle},
    startup_snapshot::StartupSnapshot,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

const VERSION: &str = "synthetic.1";
const BIG_KEY: u32 = u32::MAX;

/// The opening gameplay content the other gateway suites load (read only).
fn content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-anvil-placement-v1/content")
}

fn collection(id: u32, keys: &[u32]) -> CollectionRecord {
    CollectionRecord {
        prefix: [id, 1, 2, 3, 4, 5, 6],
        reward: RewardBundle {
            success: 1,
            fixed_fields: std::array::from_fn(|index| index as u8 + 1),
            entries: Vec::new(),
            trailing_value: 77,
        },
        keyed_rows: keys
            .iter()
            .map(|&key| CollectionKeyedRow {
                key,
                // Distinct bytes everywhere, so a stray write shows up in the byte diffs;
                // the final (collected) byte is 1 as in the capture.
                payload: std::array::from_fn(|index| {
                    if index == 28 {
                        1
                    } else {
                        ((key as u8) ^ index as u8) | 0x80
                    }
                }),
            })
            .collect(),
    }
}

/// The synthetic reference with two all-collected collections injected into its template.
fn reference() -> Reference {
    let mut reference = Reference::new(VERSION, Some(&content()));
    let path = reference.directory.join("0007.bin");
    let mut data = selfdata::parse_self_with_quest_bindings(&fs::read(&path).unwrap(), &[]);
    let eight: Vec<u32> = (201..=208).collect();
    data.collections = vec![
        collection(10, &[41, 42, 43, BIG_KEY]),
        collection(20, &eight),
    ];
    let bytes = selfdata::build_self(&data);
    fs::write(&path, &bytes).unwrap();
    reference.manifest["templates"][7]["length"] = json!(bytes.len());
    reference.manifest["templates"][7]["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    reference.write_manifest();
    reference
}

fn template(reference: &Reference) -> selfdata::SelfData {
    let bytes = fs::read(reference.directory.join("0007.bin")).unwrap();
    selfdata::parse_self_with_quest_bindings(&bytes, &[])
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn login(version: &str) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    put_string(&mut bytes, "local-ticket");
    bytes.extend_from_slice(&initial_state().guid.to_le_bytes());
    put_string(&mut bytes, version);
    put_string(&mut bytes, "");
    bytes
}

/// Log in and return the self-data message (startup index 7).
fn self_packet(gateway: &mut Gateway, version: &str) -> Vec<u8> {
    let replies = gateway.handle(&login(version)).unwrap().replies;
    assert_eq!(replies.len(), 8);
    replies[7].clone()
}

/// `(key, collected byte)` for every keyed row, in wire order.
fn flags(packet: &[u8]) -> Vec<(u32, u8)> {
    selfdata::parse_self_with_quest_bindings(packet, &[])
        .collections
        .iter()
        .flat_map(|record| &record.keyed_rows)
        .map(|row| (row.key, row.payload[28]))
        .collect()
}

/// The template's rows with only `collected` flagged.
fn expected_flags(reference: &Reference, collected: &[u32]) -> Vec<(u32, u8)> {
    template(reference)
        .collections
        .iter()
        .flat_map(|record| &record.keyed_rows)
        .map(|row| (row.key, u8::from(collected.contains(&row.key))))
        .collect()
}

/// A fresh character's saved gameplay with `extras` as its `progression_extras` JSON.
fn saved(reference: &Reference, extras: Value) -> GameplayState {
    let fresh = load(reference, None).unwrap().snapshot().gameplay;
    let mut state = serde_json::to_value(fresh.unwrap()).unwrap();
    state["progression_extras"] = extras;
    serde_json::from_value(state).unwrap()
}

fn load(reference: &Reference, state: Option<GameplayState>) -> std::io::Result<Gateway> {
    Gateway::load_gameplay(&reference.directory, initial_state(), &content(), state)
}

fn login_with(reference: &Reference, state: Option<GameplayState>) -> Vec<u8> {
    self_packet(&mut load(reference, state).unwrap(), VERSION)
}

/// Byte offsets where two equally long packets differ.
fn diff(left: &[u8], right: &[u8]) -> Vec<usize> {
    assert_eq!(left.len(), right.len());
    (0..left.len()).filter(|&i| left[i] != right[i]).collect()
}

#[test]
fn fresh_character_sends_every_collection_with_no_entry_collected() {
    let reference = reference();
    let template = template(&reference);
    assert!(
        template
            .collections
            .iter()
            .all(|record| { record.keyed_rows.iter().all(|row| row.payload[28] == 1) })
    );
    let mut cleared = template.collections.clone();
    for row in cleared.iter_mut().flat_map(|record| &mut record.keyed_rows) {
        row.payload[28] = 0;
    }
    // A fresh character with gameplay, and a reference-only session without it.
    let mut reference_only = Gateway::load(&reference.directory, initial_state()).unwrap();
    let bare = self_packet(&mut reference_only, VERSION);
    for packet in [login_with(&reference, None), bare] {
        let sent = selfdata::parse_self_with_quest_bindings(&packet, &[]).collections;
        assert_eq!(sent.len(), 2, "every collection is sent");
        assert!(sent.iter().any(|record| record.keyed_rows.len() == 8));
        assert_eq!(sent, cleared, "only the collected bytes change, all to 0");
    }
}

#[test]
fn saved_entry_flags_only_its_own_row() {
    let reference = reference();
    let fresh = login_with(&reference, None);
    let one = login_with(
        &reference,
        Some(saved(
            &reference,
            json!({"collections": {"collected": [41]}}),
        )),
    );
    assert_eq!(flags(&one), expected_flags(&reference, &[41]));
    let changed = diff(&fresh, &one);
    assert_eq!(
        changed.len(),
        1,
        "exactly one byte differs from a fresh login"
    );
    assert_eq!((fresh[changed[0]], one[changed[0]]), (0, 1));
}

#[test]
fn collections_survive_save_and_reload() {
    let reference = reference();
    let ledger = json!({"collected": [41, 42], "completed": [10]});
    let mut gateway = load(
        &reference,
        Some(saved(&reference, json!({"collections": ledger}))),
    )
    .unwrap();
    let first = self_packet(&mut gateway, VERSION);
    assert_eq!(flags(&first), expected_flags(&reference, &[41, 42]));
    // The real save envelope carries it, through JSON, and a reload builds the same bytes.
    let bytes = serde_json::to_vec(&gateway.snapshot()).unwrap();
    let saved: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        saved["gameplay"]["progression_extras"]["collections"],
        ledger
    );
    let state: GameplayState = serde_json::from_value(saved["gameplay"].clone()).unwrap();
    assert_eq!(login_with(&reference, Some(state)), first);
}

#[test]
fn old_save_without_collections_loads_empty() {
    let reference = reference();
    let old = saved(&reference, json!({"claimed_ids": [7]}));
    let mut gateway = load(&reference, Some(old)).unwrap();
    assert_eq!(
        self_packet(&mut gateway, VERSION),
        login_with(&reference, None)
    );
    let extras = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .progression_extras
        .unwrap();
    assert_eq!(extras.claimed_ids.into_iter().collect::<Vec<_>>(), [7]);
    assert_eq!(extras.collections, Default::default());
}

#[test]
fn oversized_ledger_is_refused() {
    let reference = reference();
    let ids = |count: i32| json!((0..count).collect::<Vec<_>>());
    for set in ["collected", "completed"] {
        let error = load(
            &reference,
            Some(saved(
                &reference,
                json!({"collections": {set: ids(32_769)}}),
            )),
        )
        .err()
        .expect("an over-bound ledger is refused");
        assert_eq!(error.to_string(), "invalid collection ledger");
    }
    let at_bound = saved(
        &reference,
        json!({"collections": {"collected": ids(32_768)}}),
    );
    let extras = ProgressionExtras::load(at_bound.progression_extras.clone()).unwrap();
    assert_eq!(extras.collections().collected_count(), 32_768);
    assert!(load(&reference, Some(at_bound)).is_ok());
}

#[test]
fn unknown_entry_in_ledger_flags_nothing() {
    let reference = reference();
    // -1 is what a key above i32::MAX would wrap to; it must not flag that row.
    let ledger = json!({"collected": [-1, 999_999]});
    let mut gateway = load(
        &reference,
        Some(saved(&reference, json!({"collections": ledger}))),
    )
    .unwrap();
    let packet = self_packet(&mut gateway, VERSION);
    assert!(flags(&packet).iter().any(|&(key, _)| key == BIG_KEY));
    assert_eq!(packet, login_with(&reference, None));
    // Unknown ids are kept, in case their content comes back.
    let snapshot = serde_json::to_value(gateway.snapshot()).unwrap();
    assert_eq!(
        snapshot["gameplay"]["progression_extras"]["collections"]["collected"],
        ledger["collected"]
    );
}

/// Against the real capture: a fresh login equals the template except that exactly
/// the 841 captured collected bytes are now 0.
#[test]
#[ignore = "requires lab/references/rust-gateway-reference (OSFR_GATEWAY_REFERENCE)"]
fn reference_login_clears_exactly_the_captured_collected_bytes() {
    let directory = std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-gateway-reference")
        });
    let manifest: Value =
        serde_json::from_slice(&fs::read(directory.join("manifest.json")).unwrap()).unwrap();
    let file = manifest["templates"][7]["file"].as_str().unwrap();
    let captured = fs::read(directory.join(file)).unwrap();
    let state = Gateway::reference_state(&directory).unwrap();
    let mut gateway =
        Gateway::load_with_startup_snapshot(&directory, state, StartupSnapshot::empty()).unwrap();
    let version = manifest["client_version"].as_str().unwrap();
    let sent = self_packet(&mut gateway, version);
    let rows = flags(&captured);
    assert_eq!(selfdata::parse_self(&captured).collections.len(), 253);
    assert_eq!(rows.len(), 1_815);
    assert_eq!(rows.iter().filter(|row| row.1 == 1).count(), 841);
    assert!(rows.contains(&(41, 1)), "the captured player's entry 41");
    let changed = diff(&captured, &sent);
    assert_eq!(changed.len(), 841);
    assert!(changed.iter().all(|&at| (captured[at], sent[at]) == (1, 0)));
    assert!(flags(&sent).iter().all(|row| row.1 == 0));
}
