//! T623: the login self data (opcode 12) lists only the character's own quests.
//!
//! The captured template (`lab/references/rust-gateway-reference/0007-login.bin`) carries the
//! original character's 25 active quests, then the tail `241, 2513, true, 10, 30` (completed
//! count, selected quest, auto-switch, the two quest limits; reader `00940c60`). Every
//! character used to inherit them: a new one's Other Quests page showed 25/30.
//!
//! Driven through the real `Gateway` with the real capture, once with opening content and once
//! with the local-quest content only (no `opening.json`).
use osfr_tools::{
    gameplay::GameplayState,
    gateway::Gateway,
    startup_selfdata::{QuestRecords, parse_self_with_quest_bindings},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const SPLIT_QUEST: u32 = 3_150_005; // Warrior: Achieve Level 5 (`data/content/quests/warrior.json`)

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

fn reference() -> PathBuf {
    lab().join("references/rust-gateway-reference")
}

/// The opening gameplay content the other gateway suites load (read only).
fn opening_content() -> PathBuf {
    lab().join("references/opening-anvil-placement-v1/content")
}

/// The same pack without class or opening content: only the local quests (1617, 900001).
struct LocalQuests(PathBuf);

impl LocalQuests {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-t623-local-quests-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        for file in [
            "inventory.json",
            "categories.json",
            "quests.json",
            "quest-journal.bin",
            "combat.json",
            "combat-template.bin",
        ] {
            fs::copy(opening_content().join(file), directory.join(file)).unwrap();
        }
        Self(directory)
    }
}

impl Drop for LocalQuests {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as u32).to_le_bytes());
    bytes.extend(value.as_bytes());
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn load(content: &Path, saved: Option<GameplayState>) -> Gateway {
    let character = Gateway::reference_state(&reference()).unwrap();
    Gateway::load_gameplay(&reference(), character, content, saved).unwrap()
}

fn login(gateway: &mut Gateway) -> Vec<Vec<u8>> {
    let manifest: Value =
        serde_json::from_slice(&fs::read(reference().join("manifest.json")).unwrap()).unwrap();
    let mut bytes = 1u16.to_le_bytes().to_vec();
    put_string(&mut bytes, "t623-ticket");
    bytes.extend(gateway.state().guid.to_le_bytes());
    put_string(&mut bytes, manifest["client_version"].as_str().unwrap());
    put_string(&mut bytes, "");
    let replies = gateway.handle(&bytes).unwrap().replies;
    assert_eq!(replies.len(), 8);
    replies
}

/// The quest block of the self data sent at login.
fn journal(gateway: &mut Gateway) -> QuestRecords {
    parse_self_with_quest_bindings(&login(gateway)[7], &[]).quest_records
}

fn ids(records: &QuestRecords) -> Vec<u32> {
    records
        .first
        .iter()
        .map(|record| record.quest_id())
        .collect()
}

/// Completed count, selected quest, auto-switch byte, the two limits (`00940c60`).
fn tail(records: &QuestRecords) -> (i32, i32, u8, i32, i32) {
    let field = |at: usize| i32::from_le_bytes(records.first_tail[at..at + 4].try_into().unwrap());
    (
        field(0),
        field(4),
        records.first_tail[8],
        field(9),
        field(13),
    )
}

/// A fresh character's save, edited as JSON.
fn saved(content: &Path, edit: impl FnOnce(&mut Value)) -> GameplayState {
    let fresh = load(content, None).snapshot().gameplay.unwrap();
    let mut state = serde_json::to_value(fresh).unwrap();
    edit(&mut state);
    serde_json::from_value(state).unwrap()
}

/// `(state, current, required, fourth)` of the record's first objective.
fn first_objective(records: &QuestRecords) -> [i32; 4] {
    let tail = records.first[0].objectives[0].tail;
    std::array::from_fn(|index| {
        i32::from_le_bytes(tail[index * 4..index * 4 + 4].try_into().unwrap())
    })
}

#[test]
fn the_capture_is_the_inherited_25_quest_journal() {
    let packet = fs::read(reference().join("0007-login.bin")).unwrap();
    let captured = parse_self_with_quest_bindings(&packet, &[]).quest_records;
    assert_eq!(captured.first.len(), 25);
    assert!(ids(&captured).contains(&1617));
    assert_eq!(tail(&captured), (241, 2513, 1, 10, 30));
}

#[test]
fn fresh_opening_character_sends_the_empty_journal() {
    let records = journal(&mut load(&opening_content(), None));
    assert_eq!(ids(&records), Vec::<u32>::new(), "no inherited quest");
    // The client's empty form: count 0, then the tail with nothing completed or selected.
    assert_eq!(tail(&records), (0, -1, 0, 10, 30));
}

#[test]
fn an_accepted_opening_stage_is_the_only_quest_listed() {
    let content = opening_content();
    let state = saved(&content, |state| {
        state["opening"]["stages"]["910001"]["accepted"] = json!(true);
    });
    let records = journal(&mut load(&content, Some(state)));
    assert_eq!(ids(&records), [910001]);
    let prefix = &records.first[0].prefix;
    assert_eq!(
        prefix[4..8],
        4_419_559i32.to_le_bytes(),
        "the stage's title"
    );
    assert_eq!(records.first[0].objectives.len(), 2);
    assert_eq!(tail(&records), (0, 910_001, 0, 10, 30));
}

#[test]
fn local_quest_journal_keeps_only_1617_with_its_bound_counter() {
    let content = LocalQuests::new();
    let fresh = journal(&mut load(&content.0, None));
    assert_eq!(
        ids(&fresh),
        [1617],
        "1617 starts accepted; 900001 has no record"
    );
    assert_eq!(tail(&fresh), (0, 1617, 0, 10, 30));

    let state = saved(&content.0, |state| {
        state["quests"]["quests"]["1617"]["counts"] = json!([7]);
    });
    let records = journal(&mut load(&content.0, Some(state)));
    assert_eq!(ids(&records), [1617]);
    // The binding (span offset 4112) is the objective's fourth dword; it survives the move.
    assert_eq!(first_objective(&records), [0, 0, 20, 7]);
    assert_eq!(records.first[0], {
        let mut expected = fresh.first[0].clone();
        expected.objectives[0].tail[12..16].copy_from_slice(&7i32.to_le_bytes());
        expected
    });

    let state = saved(&content.0, |state| {
        state["quests"]["quests"]["1617"] =
            json!({"accepted": true, "counts": [20], "completed": true, "rewarded": true});
    });
    let records = journal(&mut load(&content.0, Some(state)));
    assert_eq!(
        ids(&records),
        Vec::<u32>::new(),
        "a rewarded quest leaves the list"
    );
    assert_eq!(tail(&records), (1, -1, 0, 10, 30));
}

#[test]
fn reference_only_session_sends_the_empty_journal() {
    let character = Gateway::reference_state(&reference()).unwrap();
    let mut gateway = Gateway::load(&reference(), character).unwrap();
    let records = journal(&mut gateway);
    assert_eq!(ids(&records), Vec::<u32>::new());
    assert_eq!(tail(&records), (0, -1, 0, 10, 30));
}

#[test]
fn a_saved_split_quest_comes_back_once_by_49_3_not_in_the_self_data() {
    let content = opening_content();
    let state = saved(&content, |state| {
        state["npc_quests"]["quests"][SPLIT_QUEST.to_string()] =
            json!({"status": "active", "counts": [0]});
    });
    let mut gateway = load(&content, Some(state));
    let mut replies = login(&mut gateway);
    let records = parse_self_with_quest_bindings(&replies[7], &[]).quest_records;
    assert_eq!(
        ids(&records),
        Vec::<u32>::new(),
        "the split quest is not in the span"
    );
    replies.extend(gateway.handle(&wrap(&[13, 0])).unwrap().replies);
    replies.extend(gateway.handle(&wrap(&[10, 0])).unwrap().replies);
    let adds: Vec<_> = replies
        .iter()
        .filter(|packet| packet.len() > 7 && packet[..3] == [5, 0, 1])
        .map(|packet| &packet[7..])
        .filter(|body| body.len() >= 10 && body[..6] == [49, 0, 3, 0, 0, 0])
        .map(|body| u32::from_le_bytes(body[6..10].try_into().unwrap()))
        .collect();
    assert_eq!(
        adds,
        [SPLIT_QUEST],
        "exactly one 49/3 adds it to the journal"
    );
}
