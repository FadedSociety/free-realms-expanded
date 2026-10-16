use osfr_tools::{
    inventory::Inventory,
    profiles::ProfileState,
    progression_extras,
    rewards::{ItemGrant, RewardBundleRow, RewardDisplay, RewardGrant, RewardState},
};
use progression_extras::{
    AchievementProgression, AchievementRule, ClaimWindow, CollectionCatalogue,
    CollectionDefinition, CollectionEntryUpdate, CollectionEvent, CollectionEventOrigin,
    CollectionLedger, CollectionMessage, CollectionProgression, TitleState, claim_request,
    definitions_request, promotional_packet, title_select_request, validate_reward_rows,
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
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-progression-extras-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let coins = 0i32.to_le_bytes();
        let count = 0i32.to_le_bytes();
        let definition = [100i32.to_le_bytes(), 8i32.to_le_bytes()].concat();
        let manifest = json!({
            "schema_version": 1,
            "player_guid": 800,
            "default_state": {
                "coins": 0,
                "items": [],
                "equipment": [],
                "next_instance_id": null
            },
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [
                span("coins", 13, 0, &coins),
                span("items", 17, 0, &count),
                span("equipment", 21, 1, &count)
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
                "serialized_hex": hex(&definition),
                "sha256": digest(&definition)
            }]
        });
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        Self { path }
    }

    fn state(&self) -> RewardState {
        let inventory = Inventory::load(&self.path, None).unwrap();
        let profiles = ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::from([1]),
            experience: BTreeMap::new(),
        };
        RewardState::new(inventory, profiles, 8)
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

fn assert_locale_ids_resolve(value: &Value, locale: &BTreeMap<u64, String>) {
    match value {
        Value::Object(fields) => {
            for field in ["name_id", "description_id"] {
                if let Some(id) = fields.get(field).and_then(Value::as_u64) {
                    assert!(
                        locale.get(&id).is_some_and(|text| !text.trim().is_empty()),
                        "client locale {field} {id} is missing or empty"
                    );
                }
            }
            for child in fields.values() {
                assert_locale_ids_resolve(child, locale);
            }
        }
        Value::Array(rows) => {
            for row in rows {
                assert_locale_ids_resolve(row, locale);
            }
        }
        _ => {}
    }
}

fn definition_field_count(content: &Value, field: &str) -> usize {
    content["definitions"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| !row[field].is_null())
        .count()
}

fn coin_grant(coins: u32) -> RewardGrant {
    RewardGrant {
        coins,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        display: RewardDisplay {
            source_guid: 700,
            player_guid: 800,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn item_grant() -> RewardGrant {
    let mut item_body = vec![0; 34];
    item_body[17..21].copy_from_slice(&100u32.to_le_bytes());
    item_body.extend(0u32.to_le_bytes());
    RewardGrant {
        coins: 0,
        items: vec![ItemGrant {
            definition_id: 100,
            count: 1,
        }],
        job_xp: BTreeMap::new(),
        display: RewardDisplay {
            source_guid: 701,
            player_guid: 800,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: vec![RewardBundleRow {
                type_id: 1,
                body: item_body,
            }],
        },
    }
}

fn update(collection_id: u32, entry_id: u32) -> CollectionEntryUpdate {
    CollectionEntryUpdate {
        collection_id,
        id_pair: [0, entry_id],
        fields: [1, entry_id, 0, 0],
        collected: true,
    }
}

fn achievement_subtype(packet: &[u8]) -> Option<u32> {
    (packet.len() >= 6 && packet[..2] == 150u16.to_le_bytes())
        .then(|| u32::from_le_bytes(packet[2..6].try_into().unwrap()))
}

#[test]
fn collection_completes_and_grants_once_across_gameplay_sources() {
    let fixture = InventoryFixture::new();
    let rewards = fixture.state();
    let mut catalogue = CollectionCatalogue::new();
    catalogue
        .insert(CollectionDefinition {
            id: 11,
            category_id: 4,
            entries: vec![101, 102, 103],
        })
        .unwrap();
    let mut progression = CollectionProgression::new(catalogue.clone(), CollectionLedger::new());
    let reward = coin_grant(5);

    let first = progression
        .collect(
            CollectionEvent {
                entry_id: 101,
                origin: CollectionEventOrigin::HarvestNode,
                wire_update: Some(update(11, 101)),
            },
            &reward,
            &rewards,
        )
        .unwrap();
    assert!(first.outcome.completion.is_none());
    assert_eq!(first.collection_packets[0][..4], [38, 0, 10, 0]);
    assert_eq!(first.collection_packets[0].len(), 33);

    let second = progression
        .collect(
            CollectionEvent {
                entry_id: 102,
                origin: CollectionEventOrigin::QuestReward,
                wire_update: Some(update(11, 102)),
            },
            &reward,
            &first.reward_state,
        )
        .unwrap();
    assert!(second.outcome.completion.is_none());

    let completed = progression
        .collect(
            CollectionEvent {
                entry_id: 103,
                origin: CollectionEventOrigin::CollectionBundleEntry,
                wire_update: Some(update(11, 103)),
            },
            &reward,
            &second.reward_state,
        )
        .unwrap();
    assert!(completed.outcome.completion.is_some());
    assert_eq!(completed.reward_state.inventory.state().coins, 5);
    assert_eq!(completed.reward_packets.len(), 2);
    assert_eq!(completed.reward_packets[0][..4], [38, 0, 19, 0]);
    assert_eq!(completed.reward_packets[1][..3], [50, 0, 1]);
    assert_eq!(progression.progress(11).unwrap().collected, 3);
    let restored =
        CollectionProgression::from_ledger_bytes(catalogue, &progression.ledger_bytes().unwrap())
            .unwrap();
    assert!(restored.progress(11).unwrap().complete);

    let replay = progression
        .collect(
            CollectionEvent {
                entry_id: 103,
                origin: CollectionEventOrigin::CollectionBundleEntry,
                wire_update: Some(update(11, 103)),
            },
            &reward,
            &completed.reward_state,
        )
        .unwrap();
    assert!(!replay.outcome.newly_collected);
    assert!(replay.reward_packets.is_empty());
    assert_eq!(replay.reward_state.inventory.state().coins, 5);
}

#[test]
fn achievement_threshold_awards_once_and_emits_150_messages() {
    let fixture = InventoryFixture::new();
    let rewards = fixture.state();
    let mut progression = AchievementProgression::new(
        vec![AchievementRule {
            key: "test:counter".into(),
            client_id: Some(9001),
            counter_key: "harvest_count".into(),
            objective_id: 1,
            threshold: 2,
            trophy_id: Some(5),
            reward: coin_grant(1),
        }],
        Default::default(),
    )
    .unwrap();

    let first = progression.on_event("harvest_count", 1, &rewards).unwrap();
    assert!(first.newly_completed.is_empty());
    assert_eq!(first.packets.len(), 1);
    assert_eq!(achievement_subtype(&first.packets[0]), Some(7));
    assert_eq!(
        f32::from_le_bytes(first.packets[0][18..22].try_into().unwrap()),
        0.5
    );

    let completed = progression
        .on_event("harvest_count", 1, &first.reward_state)
        .unwrap();
    assert_eq!(completed.newly_completed, ["test:counter"]);
    assert_eq!(completed.packets.len(), 2);
    assert_eq!(achievement_subtype(&completed.packets[0]), Some(7));
    assert_eq!(achievement_subtype(&completed.packets[1]), Some(3));
    assert_eq!(completed.reward_state.inventory.state().coins, 1);

    let replay = progression
        .on_event("harvest_count", 1, &completed.reward_state)
        .unwrap();
    assert!(replay.packets.is_empty());
    assert!(replay.reward_packets.is_empty());
    assert_eq!(replay.reward_state.inventory.state().coins, 1);
}

#[test]
fn owned_title_selection_updates_state_and_uses_152() {
    let mut titles = TitleState::default();
    let added = titles.add([77, 0, 0, 0]).unwrap();
    assert_eq!(&added[..6], &[152, 0, 1, 0, 0, 0]);
    assert!(titles.contains(77));

    let update_all = titles
        .handle_select_request(&title_select_request(77))
        .unwrap();
    assert_eq!(titles.active(), Some(77));
    assert_eq!(&update_all[..6], &[152, 0, 3, 0, 0, 0]);
    assert_eq!(
        u32::from_le_bytes(update_all[22..26].try_into().unwrap()),
        1
    );
    assert!(
        titles
            .handle_select_request(&title_select_request(99))
            .is_err()
    );
    assert_eq!(
        &progression_extras::title_refresh_request()[..6],
        &[152, 0, 5, 0, 0, 0]
    );
    assert_eq!(&titles.remove(77).unwrap()[..6], &[152, 0, 2, 0, 0, 0]);
}

#[test]
fn claim_tracker_grants_only_queued_server_reward_once() {
    let fixture = InventoryFixture::new();
    let rewards = fixture.state();
    let mut claims = ClaimWindow::default();
    let claim_id = 0x0102_0304_0506_0708;
    claims
        .issue_server_grant(claim_id, item_grant(), claim_id.to_le_bytes().to_vec())
        .unwrap();
    assert_eq!(&ClaimWindow::list_request(), &[108, 0, 1, 0]);
    let list = claims.list_response().unwrap();
    assert_eq!(&list[..4], &[108, 0, 2, 0]);

    let request = claim_request(0xBEEF, &[claim_id]).unwrap();
    let claimed = claims.handle_claim_request(&request, &rewards).unwrap();
    assert!(claimed.accepted);
    assert_eq!(claimed.tracker_id, 0xBEEF);
    assert_eq!(claimed.claimed_ids, [claim_id]);
    assert_eq!(
        &claimed.response_packet[..8],
        &[108, 0, 4, 0, 0xef, 0xbe, 0, 0]
    );
    assert_eq!(claimed.reward_state.inventory.state().items.len(), 1);
    assert_eq!(
        claimed.reward_state.inventory.state().items[0].definition_id,
        100
    );
    assert_eq!(claims.pending_count(), 0);
    assert!(claims.claimed_ids().contains(&claim_id));

    let defs = definitions_request(0xBEEF, &[100]).unwrap();
    let defs_reply = claims
        .handle_definitions_request(&defs, &[1, 2, 3])
        .unwrap();
    assert_eq!(&defs_reply[..8], &[108, 0, 6, 0, 0xef, 0xbe, 0, 0]);
    assert_eq!(i32::from_le_bytes(defs_reply[9..13].try_into().unwrap()), 3);

    let replay = claims
        .handle_claim_request(&request, &claimed.reward_state)
        .unwrap();
    assert!(!replay.accepted);
    assert_eq!(replay.reward_state.inventory.state().items.len(), 1);
    assert!(replay.reward_packets.is_empty());
    let restored_claims = ClaimWindow::with_claimed_ids(claims.claimed_ids().clone());
    assert!(restored_claims.claimed_ids().contains(&claim_id));
}

#[test]
fn forbidden_reward_entry_types_are_rejected() {
    for type_id in [2, 4, 5, 9, 16, 21] {
        assert!(
            validate_reward_rows(&[RewardBundleRow {
                type_id,
                body: vec![0; 34],
            }])
            .is_err()
        );
    }
    validate_reward_rows(&[RewardBundleRow {
        type_id: 3,
        body: vec![0; 34],
    }])
    .unwrap();
}

#[test]
fn source_content_ids_are_unique_and_locale_backed() {
    let collections = progression_extras::parse_collection_content(include_bytes!(
        "../data/content/collections.json"
    ))
    .unwrap();
    let achievements = progression_extras::parse_achievement_content(include_bytes!(
        "../data/content/achievements.json"
    ))
    .unwrap();
    let titles =
        progression_extras::parse_title_content(include_bytes!("../data/content/titles.json"))
            .unwrap();

    assert_eq!(collections.categories.len(), 10);
    assert_eq!(collections.definitions.len(), 241);
    assert!(
        collections
            .definitions
            .iter()
            .all(|row| row.client_id.is_some())
    );
    assert_eq!(achievements.client_categories.len(), 23);
    assert_eq!(achievements.trophies.len(), 10);
    assert_eq!(achievements.definitions.len(), 318);
    assert!(
        achievements
            .definitions
            .iter()
            .all(|row| row.client_id.is_some())
    );
    assert_eq!(titles.definitions.len(), 90);
    assert!(titles.definitions.iter().all(|row| row.client_id.is_some()));

    let ids: Vec<u32> = collections
        .definitions
        .iter()
        .map(|row| row.client_id.unwrap())
        .chain(
            achievements
                .definitions
                .iter()
                .map(|row| row.client_id.unwrap()),
        )
        .chain(titles.definitions.iter().map(|row| row.client_id.unwrap()))
        .collect();
    assert_eq!(ids.len(), 649);
    assert_eq!(
        ids.iter().copied().collect::<BTreeSet<_>>().len(),
        ids.len()
    );
    assert_eq!(
        collections
            .definitions
            .iter()
            .filter(|row| row.client_id == Some(17054))
            .count(),
        1
    );
    assert_eq!(
        collections
            .definitions
            .iter()
            .filter(|row| row
                .client_id
                .is_some_and(|id| (0x4000_0001..=0x4000_00f1).contains(&id)))
            .count(),
        240
    );
    assert!(achievements.definitions.iter().all(|row| {
        row.client_id
            .is_some_and(|id| (0x4100_0001..=0x4100_013e).contains(&id))
    }));
    assert!(
        titles
            .definitions
            .iter()
            .all(|row| row.client_id.is_some_and(|id| (1..=90).contains(&id)))
    );

    let locale_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports/locale-id-to-text.json");
    let locale_rows: Vec<Value> = serde_json::from_slice(&fs::read(locale_path).unwrap()).unwrap();
    let locale: BTreeMap<u64, String> = locale_rows
        .iter()
        .map(|row| {
            (
                row["id"].as_u64().unwrap(),
                row["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    for content in [
        serde_json::from_slice(include_bytes!("../data/content/collections.json")).unwrap(),
        serde_json::from_slice(include_bytes!("../data/content/achievements.json")).unwrap(),
        serde_json::from_slice(include_bytes!("../data/content/titles.json")).unwrap(),
    ] {
        assert_locale_ids_resolve(&content, &locale);
    }

    let collection_json: Value =
        serde_json::from_slice(include_bytes!("../data/content/collections.json")).unwrap();
    let achievement_json: Value =
        serde_json::from_slice(include_bytes!("../data/content/achievements.json")).unwrap();
    let title_json: Value =
        serde_json::from_slice(include_bytes!("../data/content/titles.json")).unwrap();
    assert_eq!(definition_field_count(&collection_json, "name_id"), 238);
    assert_eq!(definition_field_count(&achievement_json, "name_id"), 187);
    assert_eq!(
        definition_field_count(&achievement_json, "description_id"),
        226
    );
    assert_eq!(definition_field_count(&title_json, "name_id"), 90);
    assert_eq!(
        definition_field_count(&collection_json, "description_id"),
        1
    );
    let trophy_rows = achievement_json["trophies"].as_array().unwrap();
    let trophy_ids: BTreeSet<u64> = trophy_rows
        .iter()
        .map(|row| row["id"].as_u64().unwrap())
        .collect();
    assert_eq!(trophy_ids.len(), 10);
    assert!(
        trophy_rows
            .iter()
            .all(|row| row["image_set_id"].as_u64().is_some())
    );
    assert!(
        achievement_json["definitions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| {
                row["reward"]["client_trophy_id"]
                    .as_u64()
                    .is_some_and(|id| trophy_ids.contains(&id))
            })
    );
    assert_eq!(
        achievement_json["definitions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["counter_key"].is_string())
            .count(),
        90
    );

    assert_eq!(promotional_packet(1, &[1, 2]).unwrap()[..4], [114, 0, 1, 0]);
    assert!(
        CollectionMessage::Remove { body: vec![0] }
            .encode()
            .unwrap()[..4]
            == [38, 0, 9, 0]
    );
    assert_eq!(
        CollectionMessage::Start { record: vec![1, 2] }
            .encode()
            .unwrap(),
        [38, 0, 8, 0, 2, 0, 0, 0, 1, 2]
    );
    assert_eq!(
        CollectionMessage::RemoveEntry {
            collection_id: 11,
            entry_id: 103,
        }
        .encode()
        .unwrap()[..4],
        [38, 0, 11, 0]
    );
}
