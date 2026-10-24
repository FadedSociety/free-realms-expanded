//! W178: title selection and the server-owned claim window.
//!
//! Recovered client facts: 108/2 is `bool + counted claim rows`; 108/4 is
//! `tracker u32 + bool + counted rows`; 152/3 is a length blob containing a
//! count, four dwords per title and the selected title id. Synthetic rewards
//! and ids below are test policy (R), never client-selected payouts.

use osfr_tools::{
    account_store::AccountRole,
    chat, game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::Inventory,
    profiles::ProfileState,
    progression_extras::{
        self, AchievementProgression, AchievementRule, AchievementTitleBinding, ClaimWindow,
        definitions_request,
    },
    rewards::{ItemGrant, RewardBundleRow, RewardDisplay, RewardGrant, RewardState},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const NOW: u64 = 20_000;
const CLAIM: u64 = 0x0102_0304_0506_0708;
static SERIAL: AtomicU64 = AtomicU64::new(0);

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

struct Fixture {
    content: PathBuf,
    reference: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl Fixture {
    fn new() -> Self {
        let source = std::env::var_os("OSFR_APP_FUZZ_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab().join("reports/round59-native-20261015T124443Z/content"));
        let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab().join("references/rust-gateway-reference"));
        assert!(
            source.join("inventory.json").is_file(),
            "prepared content fixture missing: {}",
            source.display()
        );
        let content = lab().join("build/w178-fixtures").join(format!(
            "titles-claims-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&content).unwrap();
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file()
                && matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("json" | "bin")
                )
            {
                fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
            }
        }
        Self { content, reference }
    }

    fn state(&self) -> GameplayState {
        Gameplay::load(&self.content, None).unwrap().state()
    }

    fn gateway(&self, saved: Option<GameplayState>) -> Gateway {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.position = [0.; 3];
        let state = saved.unwrap_or_else(|| self.state());
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, Some(state)).unwrap();
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        gateway.set_trace_enabled(true);
        for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway.handle_at(&packet, NOW).unwrap();
        }
        gateway
    }
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["w178-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if index == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

fn send(gateway: &mut Gateway, inner: &[u8]) -> GatewayOutcome {
    gateway.handle_at(&wrap(inner), NOW).unwrap()
}

fn inner_packets(out: &GatewayOutcome) -> Vec<Vec<u8>> {
    out.replies
        .iter()
        .map(|packet| packet[7..].to_vec())
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
struct WireClaimItem {
    ids: Vec<u64>,
    name_id: u32,
    description_id: u32,
    icon: [u32; 2],
    entries: Vec<[u32; 3]>,
    tail: [u32; 2],
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> u32 {
    let value = u32::from_le_bytes(bytes[*cursor..*cursor + 4].try_into().unwrap());
    *cursor += 4;
    value
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> u64 {
    let value = u64::from_le_bytes(bytes[*cursor..*cursor + 8].try_into().unwrap());
    *cursor += 8;
    value
}

/// Independent parser for the row consumed by native reader `00a040e0`.
fn decode_claim_rows(packet: &[u8], count_offset: usize) -> Vec<WireClaimItem> {
    assert_eq!(packet[count_offset - 1], 1, "success byte");
    let count =
        u32::from_le_bytes(packet[count_offset..count_offset + 4].try_into().unwrap()) as usize;
    assert!(count <= 256);
    let mut cursor = count_offset + 4;
    let mut rows = Vec::with_capacity(count);
    for _ in 0..count {
        let id_count = read_u32(packet, &mut cursor) as usize;
        assert!(id_count <= 256);
        let ids = (0..id_count)
            .map(|_| read_u64(packet, &mut cursor))
            .collect::<Vec<_>>();
        let name_id = read_u32(packet, &mut cursor);
        let description_id = read_u32(packet, &mut cursor);
        let icon = [read_u32(packet, &mut cursor), read_u32(packet, &mut cursor)];
        let entry_count = read_u32(packet, &mut cursor) as usize;
        assert!(entry_count <= 256);
        let entries = (0..entry_count)
            .map(|_| {
                [
                    read_u32(packet, &mut cursor),
                    read_u32(packet, &mut cursor),
                    read_u32(packet, &mut cursor),
                ]
            })
            .collect::<Vec<_>>();
        let tail = [read_u32(packet, &mut cursor), read_u32(packet, &mut cursor)];
        rows.push(WireClaimItem {
            ids,
            name_id,
            description_id,
            icon,
            entries,
            tail,
        });
    }
    assert_eq!(cursor, packet.len(), "complete claim response consumed");
    rows
}

/// Exact serialized definition row for catalogue item 8 from the prepared
/// content fixture (`inventory.json`, definition 8). The leading fields are
/// decoded in the client's order below; the rest is pinned so a catalogue
/// layout change cannot pass the production row and the test together.
const DEFINITION_8_ROW: [u8; 178] = [
    0x08, 0x00, 0x00, 0x00, 0xcc, 0x08, 0x00, 0x00, 0x9c, 0xb3, 0x00, 0x00, 0xe4, 0x06, 0x00, 0x00,
    0xe4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
    0x49, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xcc, 0x00, 0x00, 0x00, 0x01, 0x00, 0x0e, 0x00,
    0x00, 0x00, 0x62, 0x6f, 0x77, 0x6c, 0x5f, 0x62, 0x61, 0x73, 0x69, 0x63, 0x2e, 0x61, 0x64, 0x72,
    0x0a, 0x00, 0x00, 0x00, 0x62, 0x6f, 0x77, 0x6c, 0x2d, 0x62, 0x6f, 0x6e, 0x65, 0x73, 0x00, 0x00,
    0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00,
    0x00, 0x00, 0x64, 0x79, 0x65, 0x74, 0x69, 0x6e, 0x74, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00,
];

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

/// The client reads a definition as an id, name/description ids, the icon
/// pair, eleven leading words plus two flag bytes (`+0x2c`), two
/// length-prefixed strings, then gender usage and item type
/// (`notes/Build Guides/Gameplay/Items and Inventory.md`, runtime definition
/// record; readers `008e3760`/`00919e10`).
fn decode_definition_header(row: &[u8]) -> (i32, i32, i32, i32, i32, &str, &str, i32, i32) {
    let mut at = 46;
    let string = |at: &mut usize| {
        let len = usize::try_from(read_i32(row, *at)).unwrap();
        let text = std::str::from_utf8(&row[*at + 4..*at + 4 + len]).unwrap();
        *at += 4 + len;
        text
    };
    let model = string(&mut at);
    let texture = string(&mut at);
    (
        read_i32(row, 0),
        read_i32(row, 4),
        read_i32(row, 8),
        read_i32(row, 12),
        read_i32(row, 16),
        model,
        texture,
        read_i32(row, at),
        read_i32(row, at + 4),
    )
}

/// Independently decode the complete `108/6` body: tracker echo, success byte,
/// declared blob length, entry count, then one serialized `ClientItemDefinition`
/// row per entry. Each row must be byte-identical to the reader-derived fixture
/// (or the catalogue row for an id without one), and the cursor must land
/// exactly at the end. This proves the whole body, not only its header and
/// first id.
fn decode_definitions_response(packet: &[u8], tracker: u32, inventory: &Inventory) -> Vec<u32> {
    assert_eq!(&packet[..4], &[108, 0, 6, 0]);
    assert_eq!(
        u32::from_le_bytes(packet[4..8].try_into().unwrap()),
        tracker,
        "tracker echo"
    );
    assert_eq!(packet[8], 1, "success byte");
    let declared = i32::from_le_bytes(packet[9..13].try_into().unwrap());
    let blob = &packet[13..];
    assert_eq!(
        declared as usize,
        blob.len(),
        "the declared blob length is the whole body"
    );
    let count = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    let mut cursor = 4;
    let mut ids = Vec::with_capacity(count);
    for _ in 0..count {
        assert!(
            cursor + 4 <= blob.len(),
            "definition row has its leading id dword"
        );
        let id = i32::from_le_bytes(blob[cursor..cursor + 4].try_into().unwrap());
        let record = inventory
            .definition_record(id)
            .expect("definition row exists in the catalogue");
        let expected: &[u8] = if id == 8 { &DEFINITION_8_ROW } else { record };
        let end = cursor + record.len();
        assert_eq!(
            &blob[cursor..end],
            expected,
            "definition {id} body is the reader-derived row"
        );
        ids.push(u32::try_from(id).unwrap());
        cursor = end;
    }
    assert_eq!(cursor, blob.len(), "complete definitions blob consumed");
    ids
}

fn item_grant() -> RewardGrant {
    let mut body = vec![0; 34];
    body[17..21].copy_from_slice(&8u32.to_le_bytes());
    body.extend(0u32.to_le_bytes());
    RewardGrant {
        coins: 0,
        items: vec![ItemGrant {
            definition_id: 8,
            count: 1,
        }],
        job_xp: BTreeMap::new(),
        sc: 0,
        display: RewardDisplay {
            source_guid: 700,
            player_guid: 1,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: vec![RewardBundleRow { type_id: 1, body }],
        },
    }
}

fn empty_grant() -> RewardGrant {
    RewardGrant {
        coins: 0,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        sc: 0,
        display: RewardDisplay {
            source_guid: 700,
            player_guid: 1,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn item_count(snapshot: &game_store::Snapshot, definition_id: i32) -> u32 {
    snapshot
        .gameplay
        .as_ref()
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

fn adventurer_title_id() -> u32 {
    progression_extras::parse_title_content(
        &fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/titles.json"))
            .unwrap(),
    )
    .unwrap()
    .definitions
    .iter()
    .find(|title| title.name == "The Adventurer Enthusiast")
    .and_then(|title| title.client_id)
    .expect("the shipped level-20 Adventurer title")
}

fn enable_adventurer_claim_reward(fixture: &Fixture) {
    let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    for file in ["achievements.json", "titles.json", "jobs.json"] {
        fs::copy(bundled.join(file), fixture.content.join(file)).unwrap();
    }
    let path = fixture.content.join("achievements.json");
    let mut content: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let achievement = content["definitions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["key"] == "wiki:achievement:005:adventurer-level-20")
        .unwrap();
    achievement["reward"]["item_ids"] = serde_json::json!([8]);
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
}

fn assert_snapshot_canary(gateway: &Gateway) {
    let before = gateway.snapshot();
    let mut changed = before.clone();
    changed.gameplay.as_mut().unwrap().inventory.coins ^= 1;
    assert_ne!(changed, before, "snapshot equality canary went stale");
}

#[test]
fn achievement_title_selects_and_persists_while_forged_id_changes_nothing() {
    let fixture = Fixture::new();
    let inventory = Inventory::load(&fixture.content, None).unwrap();
    let rewards = RewardState::new(
        inventory,
        ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::from([1]),
            experience: BTreeMap::new(),
        },
        100,
    );
    let mut achievements = AchievementProgression::new(
        vec![AchievementRule {
            key: "test:title".into(),
            client_id: Some(9001),
            counter_key: "title_count".into(),
            objective_id: 1,
            threshold: 1,
            trophy_id: None,
            reward: empty_grant(),
        }],
        Default::default(),
    )
    .unwrap();
    let transition = achievements.on_event("title_count", 1, &rewards).unwrap();
    assert_eq!(transition.newly_completed, ["test:title"]);

    let mut gateway = fixture.gateway(None);
    let added = gateway
        .award_achievement_titles(
            &transition.newly_completed,
            &[AchievementTitleBinding {
                achievement_key: "test:title".into(),
                title: [77, 2, 100, 9],
            }],
        )
        .unwrap();
    assert_eq!(added.len(), 1);
    assert_eq!(&added[0][..6], &[152, 0, 1, 0, 0, 0]);

    let selected = send(&mut gateway, &progression_extras::title_select_request(77));
    assert_eq!(selected.trace[0].handler, "titles");
    let replies = inner_packets(&selected);
    assert_eq!(&replies[0][..6], &[152, 0, 3, 0, 0, 0]);
    assert_eq!(
        i32::from_le_bytes(replies[0][6..10].try_into().unwrap()),
        24
    );
    assert_eq!(
        u32::from_le_bytes(replies[0][30..34].try_into().unwrap()),
        77
    );

    assert_snapshot_canary(&gateway);
    let before = gateway.snapshot();
    let forged = send(&mut gateway, &progression_extras::title_select_request(78));
    assert!(inner_packets(&forged).is_empty());
    assert_eq!(gateway.snapshot(), before);

    let path = fixture.content.join("title-save.json");
    game_store::save(&path, &before).unwrap();
    let reloaded = game_store::load_or_create(&path, before.character.clone()).unwrap();
    let metadata = reloaded
        .gameplay
        .as_ref()
        .and_then(|state| state.progression_extras.as_ref())
        .unwrap();
    assert_eq!(metadata.titles.active(), Some(77));

    let mut fresh = fixture.gateway(reloaded.gameplay.clone());
    let refresh = send(&mut fresh, &progression_extras::title_refresh_request());
    let replies = inner_packets(&refresh);
    assert_eq!(
        u32::from_le_bytes(replies[0][30..34].try_into().unwrap()),
        77
    );
    let before = fresh.snapshot();
    let forged = send(&mut fresh, &progression_extras::title_select_request(78));
    assert!(inner_packets(&forged).is_empty());
    assert_eq!(fresh.snapshot(), before);
}

#[test]
fn claim_pays_once_replay_and_forged_ids_are_inert_and_history_survives_relog() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(None);
    let before = gateway.snapshot();
    assert_eq!(item_count(&before, 8), 0);

    gateway.issue_claim(CLAIM, item_grant()).unwrap();
    let listed = send(&mut gateway, &ClaimWindow::list_request());
    assert_eq!(listed.trace[0].handler, "claims");
    let list = inner_packets(&listed).pop().unwrap();
    let rows = decode_claim_rows(&list, 5);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].ids, [CLAIM]);
    assert_eq!(rows[0].entries, [[1, 8, 0]]);

    assert_snapshot_canary(&gateway);
    let before_forged = gateway.snapshot();
    let forged_id = CLAIM.wrapping_add(1);
    let forged = send(
        &mut gateway,
        &progression_extras::claim_request(0xDEAD, &[forged_id]).unwrap(),
    );
    let forged_reply = inner_packets(&forged).pop().unwrap();
    assert_eq!(&forged_reply[..9], &[108, 0, 4, 0, 0xad, 0xde, 0, 0, 0]);
    assert_eq!(gateway.snapshot(), before_forged);

    let request = progression_extras::claim_request(0xBEEF, &[CLAIM]).unwrap();
    let claimed = send(&mut gateway, &request);
    let replies = inner_packets(&claimed);
    let claimed_row = replies
        .iter()
        .find(|packet| packet.starts_with(&[108, 0, 4, 0, 0xef, 0xbe, 0, 0]))
        .unwrap();
    assert_eq!(decode_claim_rows(claimed_row, 9).len(), 1);
    let paid = gateway.snapshot();
    assert_eq!(item_count(&paid, 8), 1);
    assert!(
        paid.gameplay
            .as_ref()
            .unwrap()
            .progression_extras
            .as_ref()
            .unwrap()
            .claimed_ids
            .contains(&CLAIM)
    );

    let definitions = definitions_request(0xBEEF, &[8]).unwrap();
    let definitions = send(&mut gateway, &definitions);
    let response = inner_packets(&definitions).pop().unwrap();
    let inventory = Inventory::load(&fixture.content, None).unwrap();
    assert_eq!(
        decode_definitions_response(&response, 0xBEEF, &inventory),
        [8],
        "the post-claim 108/6 body decodes completely"
    );

    let before_replay = gateway.snapshot();
    let replay = send(&mut gateway, &request);
    let replay_reply = inner_packets(&replay).pop().unwrap();
    assert_eq!(&replay_reply[..9], &[108, 0, 4, 0, 0xef, 0xbe, 0, 0, 0]);
    assert_eq!(gateway.snapshot(), before_replay);

    let path = fixture.content.join("claim-save.json");
    game_store::save(&path, &paid).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        saved["gameplay"]["progression_extras"]["claimed_ids"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    let reloaded = game_store::load_or_create(&path, paid.character.clone()).unwrap();
    assert!(
        gateway_claim_history_contains(&reloaded, CLAIM),
        "claim history did not survive the save"
    );
    let mut fresh = fixture.gateway(reloaded.gameplay.clone());
    assert!(
        fresh.issue_claim(CLAIM, item_grant()).is_err(),
        "a claimed reward row was reissued"
    );
    let before = fresh.snapshot();
    let replay = send(&mut fresh, &request);
    assert_eq!(
        &inner_packets(&replay).pop().unwrap()[..9],
        &[108, 0, 4, 0, 0xef, 0xbe, 0, 0, 0]
    );
    assert_eq!(fresh.snapshot(), before);
}

#[test]
fn definitions_before_claim_use_pending_contents_without_consuming_them() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(None);
    gateway.issue_claim(CLAIM, item_grant()).unwrap();
    let before = gateway.snapshot();

    // Native 00a04ea0 and 00a04f40 both open a tracker and send 108/5 before
    // 108/3. Each path must receive its own complete definitions response.
    for tracker in [0x1111_u32, 0x2222] {
        let response = inner_packets(&send(
            &mut gateway,
            &definitions_request(tracker, &[8]).unwrap(),
        ))
        .pop()
        .expect("pre-claim definitions response");
        assert_eq!(
            &response[..9],
            &[108, 0, 6, 0, tracker as u8, (tracker >> 8) as u8, 0, 0, 1],
            "response must echo the client tracker"
        );
    }
    assert_eq!(
        gateway.snapshot(),
        before,
        "opening a definition path consumed the reward"
    );

    let paid = send(
        &mut gateway,
        &progression_extras::claim_request(0x3333, &[CLAIM]).unwrap(),
    );
    assert!(
        inner_packets(&paid)
            .iter()
            .any(|packet| packet.starts_with(&[108, 0, 4, 0, 0x33, 0x33, 0, 0, 1]))
    );
    assert_eq!(item_count(&gateway.snapshot(), 8), 1);
}

/// Native `00a04c10` appends the definition id of every claim entry and does
/// not deduplicate entries that share one, so `[id, id]` is a valid `108/5`.
/// The server must authorize both and describe the definition once.
#[test]
fn duplicate_definition_ids_are_authorized_and_described_once() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(None);
    let mut grant = item_grant();
    grant.items.push(ItemGrant {
        definition_id: 8,
        count: 2,
    });
    gateway.issue_claim(CLAIM, grant).unwrap();
    let before = gateway.snapshot();

    let listed = send(&mut gateway, &ClaimWindow::list_request());
    let rows = decode_claim_rows(&inner_packets(&listed).pop().unwrap(), 5);
    assert_eq!(rows.len(), 1);
    let requested: Vec<u32> = rows[0].entries.iter().map(|entry| entry[1]).collect();
    assert_eq!(
        requested,
        [8, 8],
        "one pending row can repeat a shared item definition"
    );

    let response = inner_packets(&send(
        &mut gateway,
        &definitions_request(0x1234, &requested).unwrap(),
    ))
    .pop()
    .expect("a duplicate id request must still receive 108/6");
    let inventory = Inventory::load(&fixture.content, None).unwrap();
    assert_eq!(
        decode_definitions_response(&response, 0x1234, &inventory),
        [8],
        "each definition is described once"
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "a definitions response consumed no claim"
    );

    // Deduplication must not widen authorization: an id absent from every
    // pending row is still refused without a reply or a state change. 25 is a
    // catalogue definition, so only the ownership check can refuse it.
    let after = gateway.snapshot();
    let refused = send(
        &mut gateway,
        &definitions_request(0x5678, &[8, 25]).unwrap(),
    );
    assert!(inner_packets(&refused).is_empty());
    assert_eq!(gateway.snapshot(), after);
}

#[test]
fn pending_claim_survives_a_reconnect_and_pays_exactly_once() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(None);
    gateway.issue_claim(CLAIM, item_grant()).unwrap();
    assert_eq!(item_count(&gateway.snapshot(), 8), 0);

    let path = fixture.content.join("pending-claim-save.json");
    let queued = gateway.snapshot();
    game_store::save(&path, &queued).unwrap();
    let reloaded = game_store::load_or_create(&path, queued.character.clone()).unwrap();
    let mut fresh = fixture.gateway(reloaded.gameplay.clone());

    let listed = send(&mut fresh, &ClaimWindow::list_request());
    let rows = decode_claim_rows(&inner_packets(&listed).pop().unwrap(), 5);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].ids, [CLAIM]);

    let claim = progression_extras::claim_request(0x4444, &[CLAIM]).unwrap();
    let paid = send(&mut fresh, &claim);
    assert!(
        inner_packets(&paid)
            .iter()
            .any(|packet| packet.starts_with(&[108, 0, 4, 0, 0x44, 0x44, 0, 0, 1]))
    );
    assert_eq!(item_count(&fresh.snapshot(), 8), 1);

    let replay = send(&mut fresh, &claim);
    assert_eq!(
        &inner_packets(&replay).pop().unwrap()[..9],
        &[108, 0, 4, 0, 0x44, 0x44, 0, 0, 0]
    );
    assert_eq!(item_count(&fresh.snapshot(), 8), 1);
}

#[test]
fn claim_payout_uses_live_coins_after_queueing() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(None);
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);
    gateway.issue_claim(CLAIM, item_grant()).unwrap();

    let _ = send(&mut gateway, &chat::local_request("realm coins 7"));
    let before = gateway.snapshot();
    assert_eq!(before.gameplay.as_ref().unwrap().inventory.coins, 7);

    let paid = send(
        &mut gateway,
        &progression_extras::claim_request(0x8888, &[CLAIM]).unwrap(),
    );
    assert!(
        inner_packets(&paid)
            .iter()
            .any(|packet| packet.starts_with(&[108, 0, 4, 0, 0x88, 0x88, 0, 0, 1]))
    );
    let after = gateway.snapshot();
    assert_eq!(
        after.gameplay.as_ref().unwrap().inventory.coins,
        7,
        "claim restored a stale wallet snapshot"
    );
    assert_eq!(item_count(&after, 8), 1);
}

#[test]
fn job_rank_completion_awards_a_data_bound_title_and_select_request_equips_it() {
    let fixture = Fixture::new();
    let mut state = fixture.state();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = 1;
    profiles.unlocked.insert(1);
    profiles.experience.insert(1, 1_800);
    let mut gateway = fixture.gateway(Some(state));
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);

    let title_id = adventurer_title_id();

    let earned = send(&mut gateway, &chat::local_request("realm level 20"));
    let earned_packets = inner_packets(&earned);
    assert!(
        earned_packets.iter().any(|packet| {
            packet.starts_with(&[152, 0, 1, 0])
                && packet.get(6..10) == Some(&title_id.to_le_bytes())
        }),
        "authoritative job XP completion did not award the bound title: {earned_packets:?}"
    );

    let selected = send(
        &mut gateway,
        &progression_extras::title_select_request(title_id),
    );
    let replies = inner_packets(&selected);
    assert_eq!(replies[0][..6], [152, 0, 3, 0, 0, 0]);
    assert_eq!(
        u32::from_le_bytes(replies[0][30..34].try_into().unwrap()),
        title_id
    );
}

/// The claim reply copies catalogue definition bytes, so the test must not
/// derive its expectation from that same function. This fixture was taken from
/// the prepared catalogue row and its leading fields are decoded in the
/// client's order; the client reader would reject a different layout.
#[test]
fn definition_row_matches_the_client_reader_and_the_pinned_fixture() {
    let fixture = Fixture::new();
    let inventory = Inventory::load(&fixture.content, None).unwrap();
    let row = inventory
        .definition_record(8)
        .expect("definition 8 exists in the catalogue");
    assert_eq!(
        row,
        &DEFINITION_8_ROW[..],
        "the catalogue row for item 8 changed; re-derive the fixture from the client layout"
    );
    let (id, name, description, icon, tint, model, texture, gender, kind) =
        decode_definition_header(row);
    assert_eq!(
        (id, name, description, icon, tint),
        (8, 2252, 45980, 1764, 228)
    );
    assert_eq!((model, texture), ("bowl_basic.adr", "bowl-bones"));
    assert_eq!((gender, kind), (0, 4));
    assert_eq!(
        (row[44], row[45]),
        (1, 0),
        "two flag bytes close the 46-byte prefix"
    );
}

#[test]
fn achievement_item_reward_is_issued_to_the_claim_window_by_gameplay() {
    let fixture = Fixture::new();
    enable_adventurer_claim_reward(&fixture);
    let mut state = fixture.state();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = 1;
    profiles.unlocked.insert(1);
    profiles.experience.insert(1, 1_800);
    let mut gateway = fixture.gateway(Some(state));
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);

    let _ = send(&mut gateway, &chat::local_request("realm level 20"));
    assert_eq!(item_count(&gateway.snapshot(), 8), 0);
    let listed = send(&mut gateway, &ClaimWindow::list_request());
    let rows = decode_claim_rows(&inner_packets(&listed).pop().unwrap(), 5);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entries, [[1, 8, 0]]);

    let paid = send(
        &mut gateway,
        &progression_extras::claim_request(0x7777, &rows[0].ids).unwrap(),
    );
    assert!(
        inner_packets(&paid)
            .iter()
            .any(|packet| packet.starts_with(&[108, 0, 4, 0, 0x77, 0x77, 0, 0, 1]))
    );
    assert_eq!(item_count(&gateway.snapshot(), 8), 1);
}

/// A full claim inbox must defer the achievement award, never fail the reward
/// transaction that moved the counter: the same settlement runs inside the
/// shared-field tick, where an `Err` would freeze every player in the instance.
/// The moved counter is re-observed by the next authoritative transaction.
#[test]
fn settlement_capacity_failure_defers_the_award_without_failing_the_transaction() {
    let fixture = Fixture::new();
    enable_adventurer_claim_reward(&fixture);
    let mut state = fixture.state();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.active = 1;
    profiles.unlocked.insert(1);
    profiles.experience.insert(1, 1_800);
    let mut gateway = fixture.gateway(Some(state));
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);

    let title_id = adventurer_title_id();

    // MAX_CLAIM_ITEMS pending rows: the level-20 item grant cannot be queued.
    for index in 0..256_u64 {
        gateway
            .issue_claim(CLAIM + 1 + index, item_grant())
            .unwrap();
    }
    assert_eq!(pending_claim_count(&mut gateway), 256);

    // The request commits the job XP; only the achievement award is deferred.
    let earned = send(&mut gateway, &chat::local_request("realm level 20"));
    assert!(
        !inner_packets(&earned).is_empty(),
        "the request still answers"
    );
    let deferred = gateway.snapshot();
    assert_eq!(
        deferred
            .gameplay
            .as_ref()
            .unwrap()
            .profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&1),
        Some(&1_900),
        "the refused settlement must not roll back the reward transaction"
    );
    let extras = deferred
        .gameplay
        .as_ref()
        .unwrap()
        .progression_extras
        .as_ref()
        .unwrap();
    assert!(
        !extras.titles.contains(title_id),
        "a deferred award must not add the title"
    );
    assert_eq!(
        pending_claim_count(&mut gateway),
        256,
        "a deferred award must not queue a claim"
    );

    // Claim one row so the retry has capacity; a reconnect's login sync is the
    // later authoritative transaction that observes the moved counter again.
    let rows = decode_claim_rows(
        &inner_packets(&send(&mut gateway, &ClaimWindow::list_request()))
            .pop()
            .unwrap(),
        5,
    );
    let paid = send(
        &mut gateway,
        &progression_extras::claim_request(0x9999, &rows[0].ids).unwrap(),
    );
    assert!(
        inner_packets(&paid)
            .iter()
            .any(|packet| packet.starts_with(&[108, 0, 4, 0, 0x99, 0x99, 0, 0, 1]))
    );
    assert_eq!(pending_claim_count(&mut gateway), 255);
    let saved = gateway.snapshot();

    let path = fixture.content.join("deferred-award-save.json");
    game_store::save(&path, &gateway.snapshot()).unwrap();
    let reloaded = game_store::load_or_create(&path, saved.character.clone()).unwrap();
    let mut fresh = fixture.gateway(reloaded.gameplay.clone());
    let titles = fresh
        .snapshot()
        .gameplay
        .and_then(|state| state.progression_extras.map(|extras| extras.titles))
        .unwrap();
    assert!(
        titles.contains(title_id),
        "the reconnect did not award the deferred level-20 title"
    );
    assert_eq!(
        pending_claim_count(&mut fresh),
        256,
        "the reconnect queued the deferred achievement claim"
    );
}

fn pending_claim_count(gateway: &mut Gateway) -> usize {
    decode_claim_rows(
        &inner_packets(&send(gateway, &ClaimWindow::list_request()))
            .pop()
            .expect("claim list response"),
        5,
    )
    .len()
}

fn gateway_claim_history_contains(
    snapshot: &osfr_tools::game_store::Snapshot,
    claim_id: u64,
) -> bool {
    snapshot
        .gameplay
        .as_ref()
        .and_then(|state| state.progression_extras.as_ref())
        .is_some_and(|state| state.claimed_ids.contains(&claim_id))
}
