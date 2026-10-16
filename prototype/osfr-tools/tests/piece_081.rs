//! Item 081: inventory instance lifecycle.
//!
//! These vectors run against `src/restoration/piece_081.rs`, which is included by `#[path]` so the
//! module compiles before the lead registers it in `src/lib.rs`.
//!
//! ORIGINAL facts used by these vectors (all outside Git, see
//! `notes/Research/Pieces/081 Inventory instance lifecycle.md`):
//! * the supplied client and the prototype gateway serialize the same item row: definition, tint,
//!   instance id, count, consumed count, last cast time, rental flag, ability count
//!   (`lab/decompiled/server/Gateway/Gateway.Player/PlayerCode.cs` and the pinned community
//!   `Sanctuary.Packet.Common/ItemInstance.cs`).
//! * `BaseItemDefinition.MaxStackSize` is an original definition field
//!   (`lab/decompiled/server/Gateway/Gateway/BaseItemDefinition.cs` line 59, written by
//!   `PlayerCode.SendPlayerUpdateItemDefinitions`).
//! * the preserved original character self holds 1386 unique live instance ids in 2..=4595 with
//!   3209 permanent gaps, and one live row (id 3650, definition 9119, count 10) sits above that
//!   definition's declared max stack of 1
//!   (`lab/references/rust-gameplay/inventory.json`). Both shapes are data, not errors.
//!
//! RECONSTRUCTED here: the durable counter document, the burn-on-failure ordering, every numeric
//! bound and every error. No original server allocation routine was recovered, so "never reuse an
//! id" is a local invariant justified by the sparse captured id space, not a native rule.

#[path = "../src/restoration/piece_081.rs"]
#[allow(dead_code)]
mod piece_081;

use osfr_tools::inventory::{Inventory, InventoryItem, InventoryState};
use piece_081::{CounterError, InstanceCounter};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn item(id: i32, definition_id: i32, count: u32) -> InventoryItem {
    InventoryItem {
        id,
        definition_id,
        tint: 0,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
    bytes.push(0);
    bytes.extend(item.ability_count.to_le_bytes());
    bytes.push(0);
    bytes
}

/// A directory with the smallest valid prepared inventory content plus a counter path.
struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-piece-081-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let state = InventoryState {
            next_instance_id: None,
            coins: 4,
            items: vec![item(2, 100, 2)],
            equipment: Vec::new(),
        };
        let mut packet = vec![5u8, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        let coins_offset = packet.len();
        packet.extend(4i32.to_le_bytes());
        let items_offset = packet.len();
        packet.extend(1i32.to_le_bytes());
        packet.extend(serialize_item(&state.items[0]));
        let length = packet.len();
        packet[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        packet[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());
        let span = |kind: &str, offset: usize, length: usize, profile_id: i32| {
            json!({
                "kind": kind,
                "offset": offset,
                "length": length,
                "profile_id": profile_id,
                "sha256": digest(&packet[offset..offset + length]),
            })
        };
        let definitions: Vec<_> = [100i32, 102]
            .into_iter()
            .map(|id| {
                let bytes = [id.to_le_bytes(), (id - 100).to_le_bytes()].concat();
                json!({
                    "id": id,
                    "slot": 0,
                    "class_id": 1,
                    "default_tint": 0,
                    "max_stack": if id == 100 { 10 } else { 1 },
                    "model": "",
                    "texture": "",
                    "tint_alias": "",
                    "effect": 0,
                    "wield_type": 0,
                    "serialized_hex": bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
                    "sha256": digest(&bytes),
                })
            })
            .collect();
        let content = json!({
            "schema_version": 1,
            "player_guid": 1,
            "default_state": state,
            "profiles": [],
            "spans": [
                span("coins", coins_offset, 4, 0),
                span("items", items_offset, packet.len() - items_offset, 0),
            ],
            "definitions": definitions,
        });
        fs::write(
            directory.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { directory }
    }

    fn inventory(&self) -> Inventory {
        Inventory::load(&self.directory, None).unwrap()
    }

    fn counter_path(&self) -> PathBuf {
        self.directory.join("instance-counter.json")
    }

    fn counter(&self) -> InstanceCounter {
        InstanceCounter::open(self.counter_path(), &[]).unwrap()
    }

    fn counter_over(&self, live_instance_ids: &[i32]) -> InstanceCounter {
        InstanceCounter::open(self.counter_path(), live_instance_ids).unwrap()
    }

    fn try_counter(&self) -> Result<InstanceCounter, CounterError> {
        InstanceCounter::open(self.counter_path(), &[])
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn live_ids(inventory: &Inventory) -> Vec<i32> {
    inventory.state().items.iter().map(|item| item.id).collect()
}

#[test]
fn counter_hands_out_ids_above_every_live_instance() {
    let f = Fixture::new();
    let mut fresh = f.counter();
    assert_eq!(fresh.allocate().unwrap(), 1);
    assert_eq!(fresh.next_instance_id(), 2);

    let mut reconciled = f.counter_over(&[2, 5, 7]);
    assert_eq!(reconciled.next_instance_id(), 8);
    assert_eq!(reconciled.allocate().unwrap(), 8);
    assert_eq!(reconciled.allocate().unwrap(), 9);
}

/// The acceptance case: the highest live instance retires and its id must stay retired.
#[test]
fn counter_never_reuses_the_highest_id_after_it_is_removed() {
    let f = Fixture::new();
    let mut counter = f.counter_over(&[2]);
    let first = counter.allocate().unwrap();
    assert_eq!(first, 3);
    let second = counter.allocate().unwrap();
    assert!(
        second > first,
        "retired id {first} was handed out again as {second}"
    );
    assert_ne!(second, first);
}

#[test]
fn counter_persists_across_reopen() {
    let f = Fixture::new();
    {
        let mut counter = f.counter_over(&[2]);
        assert_eq!(counter.allocate().unwrap(), 3);
    }
    let mut reopened = f.counter();
    assert_eq!(reopened.allocate().unwrap(), 4);
}

#[test]
fn counter_recovers_from_a_lost_or_stale_document() {
    let f = Fixture::new();
    {
        let mut counter = f.counter_over(&[2]);
        counter.allocate().unwrap();
    }
    fs::remove_file(f.counter_path()).unwrap();
    let mut recovered = f.counter_over(&[99]);
    assert_eq!(recovered.allocate().unwrap(), 100);
    assert_eq!(f.counter().next_instance_id(), 101);

    // A stale document below the live rows is floored and repaired on open.
    fs::write(
        f.counter_path(),
        br#"{"schema_version":1,"next_instance_id":1}"#,
    )
    .unwrap();
    let stale = f.counter_over(&[99]);
    assert_eq!(stale.next_instance_id(), 100);
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(f.counter_path()).unwrap()).unwrap();
    assert_eq!(persisted["next_instance_id"].as_i64().unwrap(), 100);
}

#[test]
fn counter_refuses_to_exceed_the_instance_id_bound() {
    let f = Fixture::new();
    fs::write(
        f.counter_path(),
        format!("{{\"schema_version\":1,\"next_instance_id\":{}}}", i32::MAX),
    )
    .unwrap();
    let mut counter = f.counter();
    assert_eq!(counter.allocate().unwrap(), i32::MAX);
    let after_last = fs::read(f.counter_path()).unwrap();
    assert!(matches!(counter.allocate(), Err(CounterError::Exhausted)));
    assert_eq!(counter.next_instance_id(), i64::from(i32::MAX) + 1);
    assert_eq!(fs::read(f.counter_path()).unwrap(), after_last);
}

/// The counter is written before the id is returned, so a grant that never lands burns the id
/// instead of freeing it for a later grant.
#[test]
fn counter_records_an_id_before_handing_it_out() {
    let f = Fixture::new();
    let mut counter = f.counter();
    let first = counter.allocate().unwrap();
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(f.counter_path()).unwrap()).unwrap();
    assert_eq!(
        persisted["next_instance_id"].as_i64().unwrap(),
        i64::from(first) + 1
    );
    assert_ne!(counter.allocate().unwrap(), first);
}

#[test]
fn counter_write_failure_leaves_memory_and_the_document_unchanged() {
    let f = Fixture::new();
    let mut counter = f.counter_over(&[2]);
    counter.allocate().unwrap();
    let next = counter.next_instance_id();
    fs::remove_file(f.counter_path()).unwrap();
    fs::create_dir(f.counter_path()).unwrap();
    assert!(counter.allocate().is_err());
    assert_eq!(counter.next_instance_id(), next);
}

#[test]
fn counter_rejects_corrupt_documents() {
    for (label, bytes) in [
        (
            "unsupported version",
            br#"{"schema_version":2,"next_instance_id":1}"#.to_vec(),
        ),
        (
            "zero id",
            br#"{"schema_version":1,"next_instance_id":0}"#.to_vec(),
        ),
        (
            "negative id",
            br#"{"schema_version":1,"next_instance_id":-4}"#.to_vec(),
        ),
        (
            "unknown field",
            br#"{"schema_version":1,"next_instance_id":1,"extra":1}"#.to_vec(),
        ),
    ] {
        let f = Fixture::new();
        fs::write(f.counter_path(), bytes).unwrap();
        assert!(
            matches!(f.try_counter(), Err(CounterError::Corrupt(_))),
            "accepted a counter document with {label}"
        );
    }
    let f = Fixture::new();
    fs::write(
        f.counter_path(),
        vec![b'0'; piece_081::MAX_COUNTER_BYTES + 1],
    )
    .unwrap();
    assert!(matches!(f.try_counter(), Err(CounterError::Corrupt(_))));
}

#[test]
fn counter_verify_requires_a_unique_positive_leading_counter() {
    let f = Fixture::new();
    let counter = f.counter_over(&[2, 4]);
    assert_eq!(counter.verify(&[2, 4]).unwrap(), 4);
    assert!(matches!(
        counter.verify(&[0]),
        Err(CounterError::InvalidInstanceId { id: 0 })
    ));
    assert!(matches!(
        counter.verify(&[2, 2]),
        Err(CounterError::DuplicateInstanceId { id: 2 })
    ));
    assert!(matches!(
        counter.verify(&[9]),
        Err(CounterError::BehindHighestInstance {
            next: 5,
            highest: 9
        })
    ));
    let mut counter = f.counter_over(&[2]);
    let handed_out = counter.allocate().unwrap();
    assert_eq!(counter.verify(&[handed_out]).unwrap(), handed_out);
}

/// Production still allocates `max(live id) + 1`, so retiring the top instance frees its id. This
/// pins the gap the counter closes; delete it when `inventory.rs` takes its id from the counter.
#[test]
fn production_reward_never_reuses_a_retired_top_instance_id() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    inventory.grant(0, 102, 1).unwrap();
    let first = inventory.state().items.last().unwrap().id;
    assert_eq!(first, 3);
    inventory.consume(first, 1).unwrap();
    assert_eq!(live_ids(&inventory), vec![2]);
    inventory.grant(0, 102, 1).unwrap();
    let reused = inventory.state().items.last().unwrap().id;
    assert!(reused > first, "retired instance must remain retired");
}

/// The intended call site: allocate from the counter, then let the reward path create the row. The
/// counter agrees with production on the first grant and diverges exactly where production reuses.
#[test]
fn counter_supplied_ids_close_the_production_reuse_gap() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let mut counter = f.counter_over(&live_ids(&inventory));
    let first = counter.allocate().unwrap();
    assert_eq!(first, 3);
    inventory.grant(0, 102, 1).unwrap();
    assert_eq!(inventory.state().items.last().unwrap().id, first);
    inventory.consume(first, 1).unwrap();
    let second = counter.allocate().unwrap();
    assert_eq!(second, 4);
    inventory.grant(0, 102, 1).unwrap();
    let production_reused = inventory.state().items.last().unwrap().id;
    assert_eq!(production_reused, 4);
    assert_eq!(
        production_reused, second,
        "the counter must be the allocator once production takes its id from it"
    );
}

#[test]
fn counter_stays_monotonic_across_a_simulated_restart_after_deleting_the_top_row() {
    let f = Fixture::new();
    let first = {
        let mut counter = f.counter_over(&[2, 3]);
        let handed_out = counter.allocate().unwrap();
        assert_eq!(handed_out, 4);
        handed_out
    };
    let mut reopened = f.counter();
    let next = reopened.allocate().unwrap();
    assert_eq!(next, first + 1);
}

/// Private evidence: the preserved original catalogue's live id shape must already satisfy the
/// counter invariant, and its one above-max-stack row must not be treated as corruption.
#[test]
#[ignore = "requires the preserved original inventory catalogue"]
fn private_original_catalogue_ids_are_unique_and_sparse() {
    let path = PathBuf::from(
        std::env::var_os("OSFR_PIECE_081_CATALOG").expect("preserved catalogue required"),
    );
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let state = manifest
        .get("initial_state")
        .cloned()
        .unwrap_or_else(|| manifest["default_state"].clone());
    let ids: Vec<i32> = state["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_i64().unwrap() as i32)
        .collect();
    assert_eq!(ids.len(), 1386);
    let f = Fixture::new();
    let counter = f.counter_over(&ids);
    let highest = counter.verify(&ids).unwrap();
    assert_eq!(highest, 4595);
    assert_eq!(counter.next_instance_id(), 4596);
    let distinct: std::collections::BTreeSet<_> = ids.iter().copied().collect();
    assert_eq!(distinct.len(), ids.len());
    let gaps = (1..=highest).filter(|id| !distinct.contains(id)).count();
    assert_eq!(gaps, 3209);
    println!("1386 live ids in 2..=4595 with {gaps} permanent gaps");
}

/// Private evidence: the counter opens above the live rows of the current campaign branch and its
/// first hand-out is the id the retired gap below the maximum cannot affect.
#[test]
#[ignore = "requires the current campaign character snapshot"]
fn private_campaign_counter_leads_the_live_rows() {
    let path =
        PathBuf::from(std::env::var_os("OSFR_PIECE_081_SAVE").expect("campaign snapshot required"));
    let snapshot: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let ids: Vec<i32> = snapshot["gameplay"]["inventory"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_i64().unwrap() as i32)
        .collect();
    assert_eq!(ids.len(), 34);
    let f = Fixture::new();
    let mut counter = f.counter_over(&ids);
    assert_eq!(counter.verify(&ids).unwrap(), 35);
    assert_eq!(counter.next_instance_id(), 36);
    assert_eq!(counter.allocate().unwrap(), 36);
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(f.counter_path()).unwrap()).unwrap();
    assert_eq!(persisted["next_instance_id"].as_i64().unwrap(), 37);
    println!(
        "current campaign: {} live ids, highest 35, counter hands out 36 and records 37",
        ids.len()
    );
}
