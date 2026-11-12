use osfr_tools::{
    inventory::Inventory,
    profiles::ProfileState,
    progression_extras,
    rewards::{ItemGrant, RewardBundleRow, RewardDisplay, RewardGrant, RewardState},
};
use progression_extras::{
    AchievementBindings, AchievementProgression, AchievementRule, ClaimItem, ClaimWindow,
    CollectionCatalogue, CollectionDefinition, CollectionEntryUpdate, CollectionEvent,
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
        sc: 0,
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
        sc: 0,
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
        definition_id: entry_id,
        icon_id: 0,
        icon_tint_id: 0,
        name_id: 0,
        collection_id,
        index: 1,
        unknown: 0,
        collected: true,
    }
}

#[test]
fn add_entry_body_matches_client_reader_8d8570() {
    // Captured login row for entry 41 (0007-login.bin byte 8941), in AddEntry order.
    let packet = CollectionMessage::AddEntry(CollectionEntryUpdate {
        definition_id: 41,
        icon_id: 2124,
        icon_tint_id: 99,
        name_id: 17056,
        collection_id: 10,
        index: 1,
        unknown: 0,
        collected: true,
    })
    .encode()
    .unwrap();
    let mut expected = vec![38, 0, 10, 0];
    for value in [41u32, 2124, 99, 17056, 10, 1, 0] {
        expected.extend(value.to_le_bytes());
    }
    expected.push(1);
    assert_eq!(packet, expected);
    // Case 10 looks the collection up by packet +0x20 (0x00939e21).
    assert_eq!(packet[20..24], 10u32.to_le_bytes());
}

#[test]
fn remove_body_is_one_collection_id() {
    assert_eq!(
        CollectionMessage::Remove { collection_id: 10 }
            .encode()
            .unwrap(),
        [38, 0, 9, 0, 10, 0, 0, 0]
    );
}

#[test]
fn collect_refuses_wire_ids_that_do_not_name_the_entry() {
    let fixture = InventoryFixture::new();
    let rewards = fixture.state();
    let mut catalogue = CollectionCatalogue::new();
    for (id, entries) in [(11, vec![101, 102, -1]), (-1, vec![201])] {
        catalogue
            .insert(CollectionDefinition {
                id,
                category_id: 4,
                entries,
            })
            .unwrap();
    }
    let mut progression = CollectionProgression::new(catalogue, CollectionLedger::new());
    let before = progression.ledger_bytes().unwrap();
    for (entry_id, wire) in [
        (101, update(11, 102)),
        (-1, update(11, u32::MAX)),
        (201, update(0, 201)),
    ] {
        let refused = progression.collect(
            CollectionEvent {
                entry_id,
                wire_update: Some(wire),
            },
            &coin_grant(5),
            &rewards,
        );
        assert!(refused.is_err(), "entry {entry_id}");
        assert_eq!(progression.ledger_bytes().unwrap(), before);
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
fn completing_with_an_empty_grant_sends_no_reward_toast() {
    let fixture = InventoryFixture::new();
    let mut catalogue = CollectionCatalogue::new();
    catalogue
        .insert(CollectionDefinition {
            id: 11,
            category_id: 4,
            entries: vec![101],
        })
        .unwrap();
    let mut progression = CollectionProgression::new(catalogue, CollectionLedger::new());
    let done = progression
        .collect(
            CollectionEvent {
                entry_id: 101,
                wire_update: Some(update(11, 101)),
            },
            &coin_grant(0),
            &fixture.state(),
        )
        .unwrap();
    assert!(done.outcome.completion.is_some());
    assert_eq!(done.collection_packets.len(), 1);
    assert!(done.reward_packets.is_empty());
    assert!(done.reward_state.applied_grant_keys.is_empty());
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
    let added = titles.add([77, 2, 100, 9]).unwrap();
    assert_eq!(&added[..6], &[152, 0, 1, 0, 0, 0]);
    assert_eq!(
        added[6..]
            .chunks_exact(4)
            .map(|word| { u32::from_le_bytes(word.try_into().unwrap()) })
            .collect::<Vec<_>>(),
        [77, 2, 100, 9]
    );
    assert!(titles.contains(77));

    let update_all = titles
        .handle_select_request(&title_select_request(77))
        .unwrap();
    assert_eq!(titles.active(), Some(77));
    assert_eq!(&update_all[..6], &[152, 0, 3, 0, 0, 0]);
    assert_eq!(
        i32::from_le_bytes(update_all[6..10].try_into().unwrap()),
        24
    );
    assert_eq!(
        u32::from_le_bytes(update_all[10..14].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(update_all[22..26].try_into().unwrap()),
        100
    );
    assert_eq!(
        u32::from_le_bytes(update_all[26..30].try_into().unwrap()),
        9
    );
    assert_eq!(
        u32::from_le_bytes(update_all[30..34].try_into().unwrap()),
        77
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
    claims.issue_server_grant(claim_id, item_grant()).unwrap();
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
        .definitions_response(&defs, &claimed.reward_state.inventory)
        .unwrap();
    assert_eq!(&defs_reply[..8], &[108, 0, 6, 0, 0xef, 0xbe, 0, 0]);
    assert!(i32::from_le_bytes(defs_reply[9..13].try_into().unwrap()) > 0);

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
fn claim_row_encoder_and_decoder_round_trip_the_native_body() {
    let claim_id = 0x0102_0304_0506_0708;
    let row = ClaimItem::for_grant(claim_id, &item_grant()).unwrap();
    let bytes = row.encode().unwrap();
    let mut cursor = 0;
    assert_eq!(ClaimItem::decode(&bytes, &mut cursor).unwrap(), row);
    assert_eq!(cursor, bytes.len());
    let mut truncated = bytes.clone();
    truncated.pop();
    let mut cursor = 0;
    assert!(ClaimItem::decode(&truncated, &mut cursor).is_err());
}

#[test]
fn failed_definition_lookup_does_not_leave_a_claim_tracker() {
    let fixture = InventoryFixture::new();
    let inventory = fixture.state().inventory;
    let mut claims = ClaimWindow::default();
    claims.issue_server_grant(1001, item_grant()).unwrap();
    let mut unavailable = item_grant();
    unavailable.items[0].definition_id = 999_999;
    claims.issue_server_grant(1002, unavailable).unwrap();

    let tracker = 0xA5A5;
    let missing = definitions_request(tracker, &[999_999]).unwrap();
    assert!(claims.definitions_response(&missing, &inventory).is_err());
    let available = definitions_request(tracker, &[100]).unwrap();
    assert!(
        claims.definitions_response(&available, &inventory).is_ok(),
        "a failed lookup must not reserve the tracker or its requested ids"
    );
}

#[test]
fn claim_batch_cannot_grow_saved_history_past_its_load_bound() {
    let fixture = InventoryFixture::new();
    let rewards = fixture.state();
    let mut claims = ClaimWindow::with_claimed_ids((1..=4095).collect());
    claims.issue_server_grant(9001, coin_grant(1)).unwrap();
    claims.issue_server_grant(9002, coin_grant(1)).unwrap();

    let request = claim_request(0xCAFE, &[9001, 9002]).unwrap();
    let rejected = claims.handle_claim_request(&request, &rewards).unwrap();
    assert!(!rejected.accepted);
    assert_eq!(rejected.response_packet[8], 0);
    assert!(rejected.reward_packets.is_empty());
    assert_eq!(rejected.reward_state.inventory.state().coins, 0);
    assert_eq!(claims.claimed_ids().len(), 4095);
    assert_eq!(claims.pending_count(), 2);
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
        318,
        "every row has a counter key; the whole-table test checks what each one means"
    );

    assert_eq!(promotional_packet(1, &[1, 2]).unwrap()[..4], [114, 0, 1, 0]);
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

#[test]
fn shipped_achievement_rules_join_every_title_by_name() {
    let bindings =
        AchievementBindings::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"))
            .unwrap();
    // Every row except the five retired Pet Trainer ranks has a rule (and so does each title).
    assert!(bindings.rules().len() >= 313);
    assert!(bindings.title_bindings().len() >= 17);
    assert_eq!(bindings.job_slug(1), Some("adventurer"));
}

/// Why each `waiting:<event>` row has no trigger yet: the game, hook or data it needs does not
/// exist. The first segment after `waiting:` is the key; a new waiting row must be listed here.
const WAITING: &[(&str, &str)] = &[
    ("group_clear", "the win does not carry the group size (the shared arena is F5's)"),
    ("quest_title", "the wiki questline's last quest has no record in the quest files"),
    ("quest_disabled", "the target quest's record is disabled (no objectives yet)"),
    ("dungeon_no_fight", "the dungeon record names no fight yet (F5 maps it)"),
    ("enemy_missing", "no enemy of that name exists in the enemy or fight files"),
    ("recipes_unlearnable", "learn_recipe refuses recipes with no pattern item; quests teach 8 of 36"),
    ("questline", "the questline has no quest records yet"),
    ("quest_repeat_completions", "quest state keeps no count of repeat completions"),
    ("pvp_duel_won", "there is no player-versus-player duel"),
    ("pvp_duel_won_with_gleam", "no PvP duel and no Gleam Strength buff"),
    ("pvp_duel_won_with_gloam", "no PvP duel and no Gloam Power buff"),
    ("pvp_duel_won_with_buff", "no PvP duel and no Gleam/Gloam buff"),
    ("cow_tipped", "no cow-tipping interaction exists"),
    ("checkers_win", "checkers.rs raises no minigame result (MG's file)"),
    ("chess_win", "chess_service.rs raises no minigame result (MG's file)"),
    ("coins_rewarded", "no lifetime count of coins paid by rewards (rewards.rs, shared)"),
    ("style_card_equipped", "style_card_use.rs raises no event (CE's file)"),
    ("marketplace_visited", "the Marketplace window open is not an event (F3's file)"),
    ("rocket_launched", "housing placement raises no event (PL's files)"),
    ("house_items_max", "housing placement raises no item count (PL's files)"),
    ("house_visited", "housing visits raise no event (PL's files)"),
    ("house_customized", "housing customization raises no event (PL's files)"),
    ("tcg_cards_played", "the card DLL plays the game; the server sees only win/loss and time"),
    ("tcg_cards_played_type", "the card DLL plays the game; the server sees only win/loss and time"),
    ("tcg_battle_margin", "the card DLL plays the game; the server sees only win/loss and time"),
    ("tcg_game_turns", "the card DLL plays the game; the server sees only win/loss and time"),
    ("tcg_cards_destroyed", "the card DLL plays the game; the server sees only win/loss and time"),
    ("tcg_point_margin", "the card DLL plays the game; the server sees only win/loss and time"),
    ("spooktacular_three_of_three", "the One/Two/Three of Three fight is not built"),
    ("spooktacular_great_scare", "The Great Scare quest is not built"),
    ("spectral_spectacles_equipped", "no Spectral Spectacles item hook"),
    ("costume_shop_visited", "no Dress Up! costume shop visit event"),
    ("gifting_tree_gift", "the Snowhill Gifting Tree is not built"),
    ("snowhill_presents_solved", "the present-thief quest is not built"),
    ("gifting_tree_saved", "the Snowhill Gifting Tree is not built"),
    ("snow_days_dailies", "the Snow Days dailies are not all built"),
    ("snowball_fighting_win", "Snowball Fighting (SB) is being built; it raises this event"),
    ("eight_bit_meep_summoned", "the 8-Bit Meep pet is not built"),
    ("love_arrows_hit_stone_heart", "the Stone Heart arrow game is not built"),
];

#[test]
fn every_achievement_has_a_trigger_or_a_stated_reason() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let read = |name: &str| -> Value {
        serde_json::from_slice(&fs::read(root.join(name)).unwrap()).unwrap()
    };
    let table = read("achievements.json");
    let rows = table["definitions"].as_array().unwrap();
    let number = |row: &Value| -> u32 {
        row["key"]
            .as_str()
            .unwrap()
            .split(':')
            .nth(2)
            .unwrap()
            .parse()
            .unwrap()
    };
    // quest id -> enabled
    let mut quests_by_id = BTreeMap::new();
    for entry in fs::read_dir(root.join("quests")).unwrap() {
        let quests: Value =
            serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap();
        for record in quests["records"].as_array().unwrap() {
            quests_by_id.insert(
                record["id"].as_u64().unwrap(),
                record["enabled"].as_bool().unwrap_or(false),
            );
        }
    }
    let bindings = AchievementBindings::load(&root).unwrap();
    let enemy_names: Vec<&str> = bindings.enemy_names().collect();
    let has_fight = |dungeon: &str| {
        fs::read_to_string(root.join("dungeons").join(format!("{dungeon}.json")))
            .is_ok_and(|text| text.contains("\"fight_id\""))
    };
    let mount_ids: BTreeSet<u64> = read("mounts.json")["mounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_u64().unwrap())
        .collect();
    // Winning settles that raise the result: the five songs, checkers (21) and the chess rows.
    let chess = read("chess.json");
    let mut win_ids: BTreeSet<u64> = BTreeSet::from([21, chess["hosted_row"]["row_id"].as_u64().unwrap()]);
    win_ids.extend(chess["rows"].as_array().unwrap().iter().map(|r| r["row_id"].as_u64().unwrap()));
    let region_ids: BTreeSet<u64> = read("regions.json")["regions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_u64().unwrap())
        .chain([0])
        .collect();
    let mut song_ids: BTreeSet<u64> = read("music_game.json")["games"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["game_id"].as_u64().unwrap())
        .collect();
    song_ids.extend(win_ids.iter().copied());
    let jobs = read("jobs.json");
    let job_ids: BTreeSet<&str> = jobs["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|j| j["id"].as_str().unwrap())
        .collect();

    let mut problems = Vec::new();
    for row in rows {
        let name = row["name"].as_str().unwrap();
        let Some(key) = row["counter_key"].as_str() else {
            problems.push(format!("{name}: no counter_key"));
            continue;
        };
        let (kind, arg) = key.split_once(':').unwrap_or((key, ""));
        let threshold = row["threshold"].as_u64();
        match kind {
            "retired" => {
                // Pet Trainer was removed 2009-12-14 (A-V check); only its five rank rows.
                if !name.starts_with("Pet Trainer: Level") || arg.is_empty() || threshold.is_some()
                {
                    problems.push(format!("{name}: retired without the Pet Trainer reason"));
                }
                continue;
            }
            "waiting" => {
                let mut parts = arg.split(':');
                let event = parts.next().unwrap();
                let what = parts.next().unwrap_or("");
                if !WAITING.iter().any(|(e, why)| *e == event && !why.is_empty()) {
                    problems.push(format!("{name}: waiting on '{event}' with no stated reason"));
                }
                // A stated reason must still be true.
                match event {
                    "dungeon_no_fight" if has_fight(what) => {
                        problems.push(format!("{name}: {what} has a fight now: bind it"));
                    }
                    "quest_disabled" => {
                        if quests_by_id.get(&what.parse().unwrap_or(0)) != Some(&false) {
                            problems.push(format!("{name}: quest {what} is not a disabled record"));
                        }
                    }
                    "enemy_missing" if enemy_names.iter().any(|n| n.contains(what)) => {
                        problems.push(format!("{name}: an enemy named {what} exists now: bind it"));
                    }
                    _ => {}
                }
            }
            "job_rank" => {
                if !job_ids.contains(arg) {
                    problems.push(format!("{name}: unknown job {arg}"));
                }
            }
            "quest_done" => {
                if quests_by_id.get(&arg.parse::<u64>().unwrap_or(0)) != Some(&true) {
                    problems.push(format!("{name}: quest {arg} is not an enabled record"));
                }
            }
            "mounts_owned_of" => {
                for id in arg.split('|') {
                    if !mount_ids.contains(&id.parse::<u64>().unwrap_or(0)) {
                        problems.push(format!("{name}: no mount {id}"));
                    }
                }
            }
            "dungeon_clear" => {
                if !has_fight(arg) {
                    problems.push(format!("{name}: dungeon {arg} names no fight, so it can never clear"));
                }
            }
            "minigame_win" => {
                for id in arg.split('|') {
                    if !song_ids.contains(&id.parse::<u64>().unwrap_or(0)) {
                        problems.push(format!("{name}: minigame {id} has no producer of the win"));
                    }
                }
            }
            "kill" => {
                let mut parts = arg.splitn(2, ':');
                let region = parts.next().unwrap().parse::<u64>().unwrap_or(u64::MAX);
                let pattern = parts.next().unwrap_or("");
                if !region_ids.contains(&region)
                    || pattern.is_empty()
                    || !enemy_names.iter().any(|n| n.contains(pattern))
                {
                    problems.push(format!("{name}: kill key {key} matches no enemy"));
                }
            }
            "completed_of" => {
                for n in arg.split('|') {
                    let n: u32 = n.parse().unwrap_or(0);
                    let child = rows.iter().find(|r| number(r) == n);
                    if child.is_none_or(|c| {
                        c["counter_key"].as_str().unwrap_or("").starts_with("retired")
                    }) {
                        problems.push(format!("{name}: sum row names {n}, missing or retired"));
                    }
                }
            }
            "quest_completed_count" | "pet_trick" | "consumable_use" | "tcg_win"
            | "tcg_win_under" | "join_dance_team" | "job_rank_max" | "opening_complete"
            | "pets_owned" | "mounts_owned" | "recipes_all" => {}
            other => problems.push(format!("{name}: unknown counter kind {other}")),
        }
        if threshold.is_none_or(|t| t == 0) {
            problems.push(format!("{name}: no threshold"));
        }
    }
    assert!(problems.is_empty(), "{} rows:\n{}", problems.len(), problems.join("\n"));
}

#[test]
fn item_rewards_are_queued_as_claims_for_the_items_the_tables_hold() {
    let bindings =
        AchievementBindings::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"))
            .unwrap();
    let claims: BTreeMap<&str, Vec<i32>> = bindings
        .claim_bindings()
        .iter()
        .map(|c| {
            (
                c.achievement_key.as_str(),
                c.grant.items.iter().map(|i| i.definition_id).collect(),
            )
        })
        .collect();
    let by_number = |n: &str| {
        claims
            .iter()
            .find(|(k, _)| k.contains(&format!(":{n}:")))
            .map(|(_, v)| v.clone())
    };
    assert_eq!(by_number("309"), Some(vec![911083])); // Mini Holiday Tree
    assert_eq!(by_number("311"), Some(vec![911084])); // Green Santa Hat
    assert_eq!(by_number("313"), Some(vec![76623])); // Lump of Coal
    assert_eq!(by_number("318"), Some(vec![911045])); // Stone Heart's Pink Bear
    assert_eq!(by_number("269"), Some(vec![911257])); // Commander Kluggenkrush
    // Every bound item is a row of the client item tables.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release");
    let mut ids = BTreeSet::new();
    for file in ["item-table.json", "item-table-supplement.json"] {
        let table: Value = serde_json::from_slice(&fs::read(root.join(file)).unwrap()).unwrap();
        ids.extend(table["rows"].as_array().unwrap().iter().map(|r| r[0].as_i64().unwrap()));
    }
    for (key, items) in &claims {
        for item in items {
            assert!(ids.contains(&i64::from(*item)), "{key}: item {item} is in no item table");
        }
    }
}

/// A bounded reader written from the client's: `badc70` reads a count then `0093ca50` records;
/// each record holds the reward bundle `008e7930` (69 bytes when empty) and an objective list
/// (`0092b050` -> `008fd770`). It fails on any short read, like the client's error byte.
struct ClientReader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl ClientReader<'_> {
    fn take(&mut self, n: usize) -> &[u8] {
        let end = self.at + n;
        assert!(end <= self.bytes.len(), "short read at {}", self.at);
        let out = &self.bytes[self.at..end];
        self.at = end;
        out
    }
    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }
    fn bundle(&mut self) {
        let bundle = self.take(69);
        assert!(bundle.iter().all(|b| *b == 0), "an empty bundle is 69 zero bytes (T202)");
    }
}

#[test]
fn the_150_4_packet_reads_back_the_way_the_client_reads_it() {
    let bindings =
        AchievementBindings::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"))
            .unwrap();
    let mut state = progression_extras::AchievementState::default();
    let done = &bindings.client_rows()[3];
    state.completed.insert(done.key.clone());
    let packet = bindings.initialize_packet(&state);
    assert_eq!(&packet[..6], &[150, 0, 4, 0, 0, 0], "family 150, subtype 4");
    let mut reader = ClientReader { bytes: &packet, at: 6 };
    let count = reader.u32() as usize;
    assert_eq!(count, bindings.client_rows().len());
    assert!(count > 100 && count < 313, "only rows with a name id are sent: {count}");
    for row in bindings.client_rows() {
        assert_eq!(reader.u32(), row.client_id, "the record key leads, as the list reader keys it");
        assert_ne!(row.name_id, 0, "no nameless record in the window");
        assert_eq!(reader.u32(), row.name_id);
        assert_eq!(reader.u32(), row.description_id);
        reader.take(1 + 16 + 4); // bool, two i64, i32
        reader.bundle();
        assert_eq!(reader.u32(), 1, "one objective");
        reader.take(4 * 4 + 1); // key, definition id, name, description, bool
        reader.bundle();
        let state_value = reader.u32();
        let current = reader.u32();
        let required = reader.u32();
        reader.take(4 + 1 + 4); // extra, bool, i32
        reader.take(8); // IconData id and tint
        reader.take(8); // two trailing i32
        let completed = reader.take(1)[0];
        reader.take(4);
        assert_eq!(required, row.threshold);
        let this_done = row.key == done.key;
        assert_eq!((completed == 1, state_value == 2), (this_done, this_done));
        assert_eq!(current == required, this_done, "only the completed row shows full progress");
    }
    assert_eq!(reader.at, packet.len(), "the reader consumes exactly the packet");
}

#[test]
fn a_nameless_row_is_counted_but_sends_nothing() {
    let bindings =
        AchievementBindings::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"))
            .unwrap();
    let sent: BTreeSet<&str> = bindings.client_rows().iter().map(|r| r.key.as_str()).collect();
    let rule = bindings
        .rules()
        .iter()
        .find(|r| !sent.contains(r.key.as_str()))
        .expect("some rows have no name id");
    let mut progression = AchievementProgression::new(
        vec![rule.clone()],
        progression_extras::AchievementState::default(),
    )
    .unwrap();
    let fixture = InventoryFixture::new();
    let rewards = fixture.state();
    let out = progression.observe(&rule.counter_key, rule.threshold, &rewards).unwrap();
    assert_eq!(out.newly_completed, vec![rule.key.clone()], "still completes server-side");
    assert!(out.packets.is_empty(), "but no 150/7 or 150/3 reaches the client");
}
