use osfr_tools::quests::QuestEngine;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Reference {
    directory: PathBuf,
    content: Value,
    message: Vec<u8>,
}

impl Reference {
    fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "osfr-quests-{}-{time}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let mut journal = vec![0u8; 32];
        journal[0..4].copy_from_slice(&1u32.to_le_bytes());
        fs::write(directory.join("journal.bin"), &journal).unwrap();
        let content = json!({"schema_version":1,"definitions":[
            {"id":1617,"title":"Synthetic original record","description":"Twenty matching authoritative defeats",
             "original":true,"initially_accepted":true,"giver_guid":0,
             "goals":[{"id":14096,"text":"matching enemies","required":20,"enemy_definition":777}],"coins":0,"items":[[1843,1]]},
            {"id":900001,"title":"Reconstructed patrol","description":"Three enemies",
             "original":false,"initially_accepted":false,"giver_guid":50,
             "goals":[{"id":1,"text":"training enemies","required":3,"enemy_definition":1294}],"coins":25,"items":[[1195,1]]}],
            "journal":{"file":"journal.bin","sha256":format!("{:x}",Sha256::digest(&journal)),"length":32,"offset":29,
              "bindings":[{"quest_id":1617,"objective_index":0,"offset":8}]},
            "trial":{"start":[0.0,0.0,0.0],"checkpoints":[[24.0,0.0,0.0],[24.0,0.0,24.0],[0.0,0.0,0.0]],
              "radius":3.0,"time_limit_ms":90000,"cooldown_ms":30000,"reward_coins":10}});
        let mut message = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        message.extend_from_slice(&[90; 16]);
        message.extend_from_slice(&journal);
        message.extend_from_slice(&[91; 4]);
        let tunnel_length = (message.len() - 7) as u32;
        let self_length = (message.len() - 13) as u32;
        message[3..7].copy_from_slice(&tunnel_length.to_le_bytes());
        message[9..13].copy_from_slice(&self_length.to_le_bytes());
        let reference = Self {
            directory,
            content,
            message,
        };
        reference.write();
        reference
    }
    fn write(&self) {
        fs::write(
            self.directory.join("quests.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
    }
    fn engine(&self) -> QuestEngine {
        QuestEngine::load(&self.directory, None).unwrap()
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        if self.directory.parent() == Some(std::env::temp_dir().as_path()) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}

#[test]
fn legacy_objective_drift_resets_active_counts_and_preserves_reward_history() {
    let reference = Reference::new();
    let initial = reference.engine().state().clone();
    for counts in [vec![], vec![1, 1], vec![21]] {
        let mut saved = initial.clone();
        saved.quests.get_mut(&1617).unwrap().counts = counts;
        let repaired = QuestEngine::load(&reference.directory, Some(saved)).unwrap();
        assert_eq!(repaired.state().quests[&1617].counts, [0]);
        assert!(repaired.state().quests[&1617].accepted);
    }
    let mut saved = initial;
    let progress = saved.quests.get_mut(&1617).unwrap();
    progress.counts = vec![20, 20];
    progress.completed = true;
    progress.rewarded = true;
    let repaired = QuestEngine::load(&reference.directory, Some(saved)).unwrap();
    assert_eq!(repaired.state().quests[&1617].counts, [20]);
    assert!(repaired.state().quests[&1617].completed);
    assert!(repaired.state().quests[&1617].rewarded);
    let mut content = reference.content.clone();
    content["definitions"]
        .as_array_mut()
        .unwrap()
        .retain(|q| q["id"] != 1617);
    content["journal"]["bindings"] = json!([]);
    fs::write(
        reference.directory.join("quests.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .unwrap();
    let removed = QuestEngine::load(&reference.directory, Some(repaired.state().clone())).unwrap();
    assert!(removed.state().quests[&1617].completed);
    assert!(removed.state().quests[&1617].rewarded);
    assert!(removed.state().quests[&1617].counts.is_empty());
}

#[test]
fn local_quest_requires_acceptance_and_grants_reward_exactly_once_after_reload() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    assert!(!engine.enemy_defeated(1294).unwrap().changed);
    assert!(engine.interaction(50).unwrap().changed);
    assert!(
        !engine
            .handle_command("quest accept 900001", 0)
            .unwrap()
            .changed
    );
    assert!(engine.enemy_defeated(1294).unwrap().rewards.is_empty());
    assert_eq!(engine.state().quests[&900001].counts, vec![1]);
    let snapshot = serde_json::from_slice(&serde_json::to_vec(engine.state()).unwrap()).unwrap();
    let mut engine = QuestEngine::load(&reference.directory, Some(snapshot)).unwrap();
    assert!(engine.enemy_defeated(1294).unwrap().rewards.is_empty());
    let completed = engine.enemy_defeated(1294).unwrap();
    assert_eq!(completed.rewards.len(), 1);
    assert_eq!(completed.rewards[0].coins, 25);
    assert_eq!(completed.rewards[0].items, vec![(1195, 1)]);
    assert!(engine.state().quests[&900001].completed);
    assert!(!engine.state().quests[&900001].rewarded);
    engine.acknowledge_reward(900001).unwrap();
    assert!(engine.state().quests[&900001].rewarded);
    let mut resumed =
        QuestEngine::load(&reference.directory, Some(engine.state().clone())).unwrap();
    assert!(resumed.enemy_defeated(1294).unwrap().rewards.is_empty());
    assert!(
        !resumed
            .handle_command("realm quest accept 900001", 0)
            .unwrap()
            .changed
    );
}

#[test]
fn original_counter_requires_twenty_matching_defeats_and_preserves_journal_bytes() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    assert!(!engine.enemy_defeated(1294).unwrap().changed);
    for count in 1..20 {
        assert!(engine.enemy_defeated(777).unwrap().rewards.is_empty());
        assert_eq!(engine.state().quests[&1617].counts, vec![count]);
    }
    let result = engine.enemy_defeated(777).unwrap();
    assert_eq!(result.rewards[0].items, vec![(1843, 1)]);
    let mut expected = reference.message.clone();
    expected[37..41].copy_from_slice(&20i32.to_le_bytes());
    assert_eq!(
        engine.patch_self_template(&reference.message).unwrap(),
        expected
    );
    let mut surrounding_change = reference.message.clone();
    surrounding_change[20] ^= 1;
    let result = engine.patch_self_template(&surrounding_change).unwrap();
    assert_eq!(result[20], surrounding_change[20]);
    surrounding_change[30] ^= 1;
    assert!(engine.patch_self_template(&surrounding_change).is_err());
}

#[test]
fn unrecovered_original_target_is_explicitly_disabled() {
    let mut reference = Reference::new();
    reference.content["definitions"][0]["goals"][0]["enemy_definition"] = json!(0);
    reference.write();
    let mut engine = reference.engine();
    for _ in 0..25 {
        assert!(!engine.enemy_defeated(777).unwrap().changed);
    }
    assert_eq!(engine.state().quests[&1617].counts, vec![0]);
    assert!(
        engine
            .handle_command("quest", 0)
            .unwrap()
            .messages
            .iter()
            .any(|s| s.contains("not recovered"))
    );
}

#[test]
fn checkpoint_course_uses_order_server_elapsed_and_a_cooldown() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    engine.on_position([0.0; 3], 1000).unwrap();
    engine.handle_command("race start", 1000).unwrap();
    assert!(
        engine
            .on_position([24.0, 0.0, 0.0], 2200)
            .unwrap()
            .rewards
            .is_empty()
    );
    let status = engine
        .handle_command("race status", 2200)
        .unwrap()
        .messages
        .join(" ");
    assert!(status.contains("checkpoint 2/3") && status.contains("Z=24.0"));
    // Returning to start cannot skip the second checkpoint.
    assert!(
        engine
            .on_position([0.0; 3], 3400)
            .unwrap()
            .rewards
            .is_empty()
    );
    assert!(
        engine
            .on_position([24.0, 0.0, 24.0], 5200)
            .unwrap()
            .rewards
            .is_empty()
    );
    let result = engine.on_position([0.0; 3], 7000).unwrap();
    assert_eq!(result.rewards[0].coins, 10);
    assert_eq!(engine.state().trial_best_ms, Some(6000));
    assert_eq!(engine.state().trial_completions, 1);
    assert!(
        engine
            .on_position([0.0; 3], 7100)
            .unwrap()
            .rewards
            .is_empty()
    );
    assert!(engine.handle_command("race start", 7100).is_err());
    assert!(engine.handle_command("race start", 37000).is_ok());
}

#[test]
fn trial_rejects_teleport_clock_abuse_timeouts_and_reconnect_replays() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    engine.on_position([0.0; 3], 0).unwrap();
    engine.handle_command("race start", 10).unwrap();
    assert!(engine.on_position([0.0; 3], 9).is_err());
    let result = engine.on_position([24.0, 0.0, 0.0], 11).unwrap();
    assert!(result.rewards.is_empty());
    assert!(result.messages.iter().any(|s| s.contains("implausible")));
    assert_eq!(engine.state().trial_completions, 0);
    engine.on_position([0.0; 3], 20).unwrap();
    engine.handle_command("race start", 20).unwrap();
    // Repeated small movement with no elapsed server time is also rejected.
    engine.on_position([1.0, 0.0, 0.0], 20).unwrap();
    engine.on_position([2.0, 0.0, 0.0], 20).unwrap();
    assert!(
        !engine
            .on_position([3.0, 0.0, 0.0], 20)
            .unwrap()
            .messages
            .is_empty()
    );
    engine.on_position([0.0; 3], 100).unwrap();
    engine.handle_command("race start", 100).unwrap();
    assert!(
        !engine
            .on_position([0.0; 3], 90101)
            .unwrap()
            .messages
            .is_empty()
    );
    engine.handle_command("race start", 90200).unwrap();
    engine.on_position([24.0, 0.0, 0.0], 91400).unwrap();
    engine.reset_session();
    assert!(
        engine
            .on_position([24.0, 0.0, 24.0], 92600)
            .unwrap()
            .rewards
            .is_empty()
    );
    let resumed = QuestEngine::load(&reference.directory, Some(engine.state().clone())).unwrap();
    assert_eq!(resumed.state().trial_completions, 0);
}

#[test]
fn counter_drift_repairs_but_invalid_reward_flags_and_private_content_fail() {
    let mut reference = Reference::new();
    let engine = reference.engine();
    let mut state = engine.state().clone();
    state.quests.get_mut(&900001).unwrap().counts[0] = 99;
    assert_eq!(
        QuestEngine::load(&reference.directory, Some(state))
            .unwrap()
            .state()
            .quests[&900001]
            .counts,
        [0]
    );
    let mut state = engine.state().clone();
    state.quests.get_mut(&900001).unwrap().rewarded = true;
    assert!(QuestEngine::load(&reference.directory, Some(state)).is_err());
    reference.content["journal"]["file"] = json!("../outside.bin");
    reference.write();
    assert!(QuestEngine::load(&reference.directory, None).is_err());
    let mut reference = Reference::new();
    reference.content["journal"]["sha256"] = json!("00".repeat(32));
    reference.write();
    assert!(QuestEngine::load(&reference.directory, None).is_err());
}

#[test]
fn invalid_quest_ids_and_status_aliases_are_safe_user_commands() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    let original = engine.state().clone();
    for command in [
        "quest accept typo",
        "quest accept 123456",
        "realm quest accept -1",
    ] {
        let outcome = engine.handle_command(command, 0).unwrap();
        assert!(!outcome.changed && outcome.rewards.is_empty() && outcome.replies.is_empty());
        assert!(!outcome.messages.is_empty());
        assert_eq!(engine.state(), &original);
    }
    let expected = engine.handle_command("quest", 0).unwrap().messages;
    for alias in ["quests", "status", "realm status"] {
        assert_eq!(engine.handle_command(alias, 0).unwrap().messages, expected);
    }
}

#[test]
#[ignore = "requires private quest content and original client reference"]
fn actual_original_journal_span_when_explicitly_configured() {
    let directory =
        PathBuf::from(std::env::var("OSFR_QUEST_CONTENT").expect("private quest content required"));
    let gateway = PathBuf::from(
        std::env::var("OSFR_GATEWAY_REFERENCE").expect("private gateway reference required"),
    );
    let content: Value =
        serde_json::from_slice(&fs::read(directory.join("quests.json")).unwrap()).unwrap();
    let gateway_manifest: Value =
        serde_json::from_slice(&fs::read(gateway.join("manifest.json")).unwrap()).unwrap();
    let template =
        fs::read(gateway.join(gateway_manifest["templates"][7]["file"].as_str().unwrap())).unwrap();
    let engine = QuestEngine::load(&directory, None).unwrap();
    assert_eq!(engine.patch_self_template(&template).unwrap(), template);
    let mut saved = engine.state().clone();
    saved.quests.get_mut(&1617).unwrap().counts[0] = 7;
    let engine = QuestEngine::load(&directory, Some(saved)).unwrap();
    let patched = engine.patch_self_template(&template).unwrap();
    let binding = &content["journal"]["bindings"][0];
    let offset = content["journal"]["offset"].as_u64().unwrap() as usize
        + binding["offset"].as_u64().unwrap() as usize;
    let mut expected = template;
    expected[offset..offset + 4].copy_from_slice(&7i32.to_le_bytes());
    assert_eq!(patched, expected);
}
