use osfr_tools::{
    inventory::Inventory,
    profiles::ProfileState,
    rewards::{ItemGrant, RewardBundleRow, RewardDisplay, RewardGrant, RewardState, apply_grant},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct InventoryFixture {
    path: PathBuf,
}

impl InventoryFixture {
    fn new(coins: u32) -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-rewards-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();

        let coin_bytes = (coins as i32).to_le_bytes();
        let empty_count = 0i32.to_le_bytes();
        let definition_bytes = [100i32.to_le_bytes(), 8i32.to_le_bytes()].concat();
        let content = json!({
            "schema_version": 1,
            "player_guid": 800,
            "default_state": {
                "coins": coins,
                "items": [],
                "equipment": [],
                "next_instance_id": null
            },
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [
                span("coins", 13, 0, &coin_bytes),
                span("items", 17, 0, &empty_count),
                span("equipment", 21, 1, &empty_count)
            ],
            "definitions": [{
                "id": 100,
                "slot": 3,
                "class_id": 1,
                "default_tint": 5,
                "max_stack": 10,
                "model": "synthetic.adr",
                "texture": "synthetic",
                "tint_alias": "synthetic",
                "effect": 0,
                "wield_type": 0,
                "serialized_hex": hex(&definition_bytes),
                "sha256": digest(&definition_bytes)
            }]
        });
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { path }
    }

    fn inventory(&self) -> Inventory {
        Inventory::load(&self.path, None).unwrap()
    }
}

impl Drop for InventoryFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("inventory.json"));
        let _ = fs::remove_dir(&self.path);
    }
}

fn span(kind: &str, offset: usize, profile_id: i32, bytes: &[u8]) -> Value {
    json!({
        "kind": kind,
        "offset": offset,
        "length": bytes.len(),
        "profile_id": profile_id,
        "sha256": digest(bytes)
    })
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn state(coins: u32) -> (InventoryFixture, RewardState) {
    let fixture = InventoryFixture::new(coins);
    let profiles = ProfileState {
        version: 1,
        active: 1,
        unlocked: BTreeSet::from([1, 2]),
        experience: BTreeMap::from([(1, 20), (2, 40)]),
    };
    let state = RewardState::new(fixture.inventory(), profiles, 4);
    (fixture, state)
}

fn grant() -> RewardGrant {
    RewardGrant {
        coins: 5,
        items: vec![ItemGrant {
            definition_id: 100,
            count: 2,
        }],
        job_xp: BTreeMap::from([(1, 15)]),
        sc: 0,
        display: RewardDisplay {
            source_guid: 700,
            player_guid: 800,
            reward_kind: 2,
            icon_id: 12,
            name_id: 34,
            rows: vec![RewardBundleRow {
                type_id: 3,
                body: vec![0; 34],
            }],
        },
    }
}

fn signed(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[test]
fn applies_wallet_item_profile_xp_and_display_in_wire_order() {
    let (_fixture, mut state) = state(10);
    state.profiles.experience.insert(1, 90);
    let (next, messages) = apply_grant(&state, &grant(), "opening:stage:7").unwrap();

    assert_eq!(messages.len(), 5);
    assert_eq!(&messages[0], &[38, 0, 19, 0, 15, 0, 0, 0]);
    assert_eq!(&messages[1], &[38, 0, 18, 0, 1, 0, 0, 0, 2, 0, 0, 0]);
    assert_eq!(&messages[2][..4], &[38, 0, 14, 0]);
    assert_eq!(signed(&messages[2], 4), 1);
    assert_eq!(signed(&messages[2], 8), 2);
    assert_eq!(signed(&messages[2], 12), 5);

    let bundle = &messages[4];
    assert_eq!(&bundle[..4], &[50, 0, 1, 1]);
    assert_eq!(signed(bundle, 4), 5);
    assert_eq!(signed(bundle, 8), 15);
    assert_eq!(signed(bundle, 16), 2);
    assert_eq!(f32::from_le_bytes(bundle[28..32].try_into().unwrap()), 1.0);
    assert_eq!(u64::from_le_bytes(bundle[40..48].try_into().unwrap()), 700);
    assert_eq!(u64::from_le_bytes(bundle[48..56].try_into().unwrap()), 800);
    assert_eq!(signed(bundle, 64), 1);
    assert_eq!(u32::from_le_bytes(bundle[68..72].try_into().unwrap()), 3);
    assert_eq!(&bundle[72..106], &[0; 34]);
    assert_eq!(signed(bundle, 106), 0);

    assert_eq!(next.inventory.state().coins, 15);
    assert_eq!(next.inventory.state().items.len(), 1);
    assert_eq!(next.inventory.state().items[0].definition_id, 100);
    assert_eq!(next.inventory.state().items[0].count, 2);
    assert!(
        next.inventory.state().next_instance_id.unwrap()
            > i64::from(next.inventory.state().items[0].id)
    );
    assert_eq!(next.profiles.experience.get(&1), Some(&105));
    assert!(next.applied_grant_keys.contains("opening:stage:7"));

    let item = &next.inventory.state().items[0];
    let mut item_payload = Vec::new();
    for value in [
        item.definition_id,
        item.tint,
        item.id,
        item.count as i32,
        item.consumed_count as i32,
        i32::try_from(item.last_cast_time).unwrap(),
    ] {
        item_payload.extend(value.to_le_bytes());
    }
    item_payload.push(0); // no rental
    item_payload.extend(item.ability_count.to_le_bytes());
    item_payload.push(0); // activation disabled
    item_payload.extend(100i32.to_le_bytes());
    item_payload.extend(8i32.to_le_bytes());
    let mut expected_item_add = vec![38, 0, 2, 0];
    expected_item_add.extend((item_payload.len() as i32).to_le_bytes());
    expected_item_add.extend(item_payload);
    assert_eq!(messages[3], expected_item_add);
}

#[test]
fn retry_with_same_key_returns_same_state_and_no_packets() {
    let (_fixture, state) = state(10);
    let grant = grant();
    let (committed, first_messages) = apply_grant(&state, &grant, "opening:stage:7").unwrap();
    assert!(!first_messages.is_empty());

    let (retried, retry_messages) = apply_grant(&committed, &grant, "opening:stage:7").unwrap();
    assert!(retry_messages.is_empty());
    assert_eq!(retried.inventory.state(), committed.inventory.state());
    assert_eq!(retried.profiles, committed.profiles);
    assert_eq!(retried.applied_grant_keys, committed.applied_grant_keys);
}

#[test]
fn insufficient_bag_capacity_fails_without_changing_input() {
    let fixture = InventoryFixture::new(10);
    let state = RewardState::new(
        fixture.inventory(),
        ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::from([1]),
            experience: BTreeMap::from([(1, 20)]),
        },
        0,
    );
    let before_inventory = state.inventory.state().clone();
    let before_profiles = state.profiles.clone();
    let mut grant = grant();
    grant.job_xp.clear();

    assert!(apply_grant(&state, &grant, "full-bag").is_err());
    assert_eq!(state.inventory.state(), &before_inventory);
    assert_eq!(state.profiles, before_profiles);
    assert!(state.applied_grant_keys.is_empty());
}

#[test]
fn wallet_overflow_fails_without_changing_input() {
    let (_fixture, state) = state(i32::MAX as u32);
    let before_inventory = state.inventory.state().clone();
    let mut grant = grant();
    grant.items.clear();
    grant.job_xp.clear();

    assert!(apply_grant(&state, &grant, "overflow").is_err());
    assert_eq!(state.inventory.state(), &before_inventory);
    assert!(state.applied_grant_keys.is_empty());
}

#[test]
fn unknown_item_definition_fails_without_committing_other_awards() {
    let (_fixture, state) = state(10);
    let before_inventory = state.inventory.state().clone();
    let before_profiles = state.profiles.clone();
    let mut grant = grant();
    grant.items[0].definition_id = 999;

    assert!(apply_grant(&state, &grant, "unknown-item").is_err());
    assert_eq!(state.inventory.state(), &before_inventory);
    assert_eq!(state.profiles, before_profiles);
    assert!(state.applied_grant_keys.is_empty());
}

#[test]
fn empty_bundle_is_69_byte_reader_body_after_three_byte_header() {
    let (_fixture, state) = state(0);
    let grant = RewardGrant {
        coins: 0,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        sc: 0,
        display: RewardDisplay {
            source_guid: 0,
            player_guid: 800,
            reward_kind: 16,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    };
    let (_next, messages) = apply_grant(&state, &grant, "empty-display").unwrap();
    let bundle = &messages[0];

    // Client reward-bundle body layout recovered by reader 008e7930 is 69 bytes when entry count is 0.
    assert_eq!(bundle.len(), 72);
    assert_eq!(&bundle[..4], &[50, 0, 1, 1]);
    assert_eq!(signed(bundle, 4), 0);
    assert_eq!(signed(bundle, 8), 0);
    assert_eq!(signed(bundle, 16), 16);
    assert_eq!(signed(bundle, 64), 0);
    assert_eq!(signed(bundle, 68), 0);
}
