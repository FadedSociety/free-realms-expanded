use osfr_tools::{
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterPhase},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-encounter-test-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let mut t = vec![0; 334];
        t[..4].copy_from_slice(&[35, 0, 2, 0]);
        t[133..137].copy_from_slice(&100_i32.to_le_bytes());
        t[224] = 1;
        t[250] = 0x5a;
        fs::write(dir.join("npc.bin"), &t).unwrap();
        let actor = |guid, def, hp, damage, stages: Vec<u32>| {
            json!({"guid":guid,"definition_id":def,"model_id":3,"name_id":100,
            "name":"Synthetic actor","texture_alias":"","position":[2.0,0.0,0.0],"max_health":hp,"damage":damage,"boss":def==500,"stages":stages})
        };
        let zone = |id, name| json!({"zone_id":id,"name":name,"sky":"","geometry_id":214,"position":[0.0,0.0,0.0],"rotation":[0.0,0.0,0.0],"tutorial":id!=0});
        let c = json!({"schema_version":1,"template_file":"npc.bin","template_sha256":format!("{:x}",Sha256::digest(t)),
            "world":zone(0,"SyntheticWorld"),"world_enemies":[actor(100_u64,400,100,1000,vec![7]),actor(101_u64,400,100,10,vec![7])],
            "dungeons":[{"id":411,"title_id":12,"description_id":13,"objective_id":14,"zone":zone(411,"SyntheticCave"),
                "guide":actor(200_u64,0,1000,0,vec![]),"enemies":[actor(201_u64,500,140,1000,vec![])],"reward_coins":50,"reward_items":[[1195,1]]}]});
        fs::write(dir.join("encounters.json"), serde_json::to_vec(&c).unwrap()).unwrap();
        Self(dir)
    }
    fn load(&self) -> EncounterEngine {
        EncounterEngine::load(&self.0, None).unwrap()
    }
}
fn location() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [9.0, 2.0, 3.0],
        rotation: [0.1, 0.2, 0.3],
    }
}

fn start(e: &mut EncounterEngine, guid: u64, position: [f32; 3], now: u64) {
    let packet = [
        vec![26, 0, 10, 0],
        guid.to_le_bytes().to_vec(),
        104_i32.to_le_bytes().to_vec(),
    ]
    .concat();
    e.handle(&packet, position, 43, now).unwrap();
}

fn close_packet(encounter: i32, instance: i32) -> Vec<u8> {
    [
        vec![41, 0, 109, 0],
        encounter.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn results_lifecycle(out: &osfr_tools::encounter::EncounterOutcome) -> Vec<u8> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[39, 0]))
        .map(|p| p[2])
        .collect()
}

#[test]
fn results_count_real_defeats_on_loss_restore_and_retry() {
    let f = Fixture::new();
    let path = f.0.join("encounters.json");
    let mut content: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut second = content["dungeons"][0]["enemies"][0].clone();
    second["guid"] = json!(202);
    content["dungeons"][0]["enemies"]
        .as_array_mut()
        .unwrap()
        .push(second);
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let defeated_count = |out: &osfr_tools::encounter::EncounterOutcome| {
        let packet = out
            .replies
            .iter()
            .find(|p| p.starts_with(&[39, 0, 47]))
            .unwrap();
        assert_eq!(i32::from_le_bytes(packet[15..19].try_into().unwrap()), 2);
        let name_len = i32::from_le_bytes(packet[19..23].try_into().unwrap()) as usize;
        assert_eq!(&packet[23..23 + name_len], b"scoreEnemiesDefeated");
        let count_at = 23 + name_len + 8;
        i32::from_le_bytes(packet[count_at..count_at + 4].try_into().unwrap())
    };
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    start(&mut e, 200, [0.0; 3], 0);
    e.attack(201, [0.0; 3], 43, 0, 0).unwrap();
    let kill = e.attack(201, [0.0; 3], 43, 0, 600).unwrap();
    assert!(results_lifecycle(&kill).is_empty()); // A surviving enemy keeps the encounter open.
    let loss = e.tick([0.0; 3], 43, 700).unwrap();
    assert_eq!(
        e.state().active.as_ref().unwrap().phase,
        EncounterPhase::Lost
    );
    assert_eq!(defeated_count(&loss), 1);
    assert!(loss.rewards.is_empty() && loss.zoning.is_none());
    let mut restored = EncounterEngine::load(&f.0, Some(e.state().clone())).unwrap();
    assert_eq!(defeated_count(&restored.on_zone_ready(43, 800).unwrap()), 1);
    restored.retry(43, 900).unwrap();
    restored.on_zone_ready(43, 900).unwrap();
    start(&mut restored, 200, [0.0; 3], 900);
    let retry_loss = restored.tick([0.0; 3], 43, 1000).unwrap();
    assert_eq!(defeated_count(&retry_loss), 0); // No score rows leak from the previous attempt.
    assert_eq!(results_lifecycle(&retry_loss), [18, 47, 19]);
}

#[test]
fn loss_hides_only_surviving_boss_displays_and_retry_registers_them_again() {
    let f = Fixture::new();
    let path = f.0.join("encounters.json");
    let mut content: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut boss = content["dungeons"][0]["enemies"][0].clone();
    boss["guid"] = json!(202);
    let mut ordinary = boss.clone();
    ordinary["guid"] = json!(203);
    ordinary["definition_id"] = json!(400);
    ordinary["boss"] = json!(false);
    content["dungeons"][0]["enemies"]
        .as_array_mut()
        .unwrap()
        .extend([boss, ordinary]);
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let display = |guid: u64, on: bool| {
        [
            vec![32, 0, 9, 0],
            guid.to_le_bytes().to_vec(),
            vec![u8::from(on)],
        ]
        .concat()
    };
    let displays = |out: &osfr_tools::encounter::EncounterOutcome| {
        out.replies
            .iter()
            .filter(|p| p.starts_with(&[32, 0, 9, 0]))
            .cloned()
            .collect::<Vec<_>>()
    };
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    e.attack(201, [0.0; 3], 43, 0, 0).unwrap();
    let kill = e.attack(201, [0.0; 3], 43, 0, 600).unwrap();
    assert_eq!(displays(&kill), [display(201, false)]);
    let before = e.state().clone();
    let loss = e.tick([0.0; 3], 43, 700).unwrap();
    // Exact community/native layout: u16 main/sub, u64 GUID, one false byte.
    assert_eq!(
        displays(&loss),
        [vec![32, 0, 9, 0, 202, 0, 0, 0, 0, 0, 0, 0, 0]]
    );
    let hide_at = loss
        .replies
        .iter()
        .position(|p| *p == display(202, false))
        .unwrap();
    let scores_at = loss
        .replies
        .iter()
        .position(|p| p.starts_with(&[39, 0, 47]))
        .unwrap();
    assert!(hide_at < scores_at);
    assert_eq!(results_lifecycle(&loss), [18, 47, 19]);
    assert!(!loss.replies.iter().any(|p| p.starts_with(&[35, 0, 3, 0])));
    assert!(loss.rewards.is_empty() && loss.zoning.is_none());
    assert!(loss.completed.is_none() && loss.defeated.is_empty());
    let saved = e.state().clone();
    assert_eq!(saved.player_health, 0);
    assert_eq!(saved.active.as_ref().unwrap().phase, EncounterPhase::Lost);
    // Only the ordinary knockout fields change; no fake death, move or reward.
    let mut expected = before;
    expected.player_health = 0;
    expected.active.as_mut().unwrap().phase = EncounterPhase::Lost;
    assert_eq!(saved, expected);
    let duplicate = e.tick([0.0; 3], 43, 1700).unwrap();
    assert!(duplicate.replies.is_empty() && !duplicate.changed);
    assert_eq!(e.state(), &saved);

    let mut restored = EncounterEngine::load(&f.0, Some(saved.clone())).unwrap();
    let ready = restored.on_zone_ready(43, 2000).unwrap();
    assert_eq!(displays(&ready), [display(202, true), display(202, false)]);
    assert_eq!(results_lifecycle(&ready), [17, 18, 47, 19]);
    let restored_hide = ready
        .replies
        .iter()
        .position(|p| *p == display(202, false))
        .unwrap();
    let restored_scores = ready
        .replies
        .iter()
        .position(|p| p.starts_with(&[39, 0, 47]))
        .unwrap();
    assert!(restored_hide < restored_scores);
    assert!(!ready.changed && ready.rewards.is_empty() && ready.zoning.is_none());
    assert_eq!(restored.state(), &saved);
    let retry = restored.retry(43, 3000).unwrap();
    assert!(retry.changed && retry.rewards.is_empty() && retry.zoning.is_some());
    assert_eq!(restored.state().player_health, 1000);
    assert_eq!(
        restored.state().active.as_ref().unwrap().phase,
        EncounterPhase::Loading
    );
    let retry_ready = restored.on_zone_ready(43, 3000).unwrap();
    assert_eq!(
        displays(&retry_ready),
        [display(201, true), display(202, true)]
    );
    assert_eq!(results_lifecycle(&retry_ready), [17]);
    start(&mut restored, 200, [0.0; 3], 3000);
    let retry_loss = restored.tick([0.0; 3], 43, 3100).unwrap();
    assert_eq!(
        displays(&retry_loss),
        [display(201, false), display(202, false)]
    );
    assert_eq!(results_lifecycle(&retry_loss), [18, 47, 19]);
    assert!(retry_loss.rewards.is_empty() && retry_loss.zoning.is_none());
}

#[test]
fn terminal_results_wait_for_native_close_and_remove_ack_never_returns_to_world() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    e.attack(201, [0.0; 3], 43, 0, 0).unwrap();
    let win = e.attack(201, [0.0; 3], 43, 0, 600).unwrap();
    assert_eq!(results_lifecycle(&win), [18, 47, 19]);
    assert!(win.zoning.is_none());
    assert_eq!(win.rewards.len(), 1);
    let before = e.state().clone();
    let ack = [
        vec![39, 0, 7],
        411_i32.to_le_bytes().to_vec(),
        (-1_i32).to_le_bytes().repeat(2),
    ]
    .concat();
    let acknowledged = e.handle(&ack, [0.0; 3], 43, 601).unwrap();
    assert!(acknowledged.zoning.is_none());
    assert!(!acknowledged.changed && acknowledged.replies.is_empty());
    assert_eq!(e.state(), &before);
    let idle = e.tick([0.0; 3], 43, 10000).unwrap();
    assert!(idle.replies.is_empty() && idle.zoning.is_none());
    assert!(
        e.attack(201, [0.0; 3], 43, 0, 10001)
            .unwrap()
            .rewards
            .is_empty()
    );
    let exit = e.handle(&close_packet(0, 1), [0.0; 3], 43, 10002).unwrap();
    assert_eq!(exit.completed, Some(411));
    assert_eq!(exit.zoning.as_ref().unwrap().position, location().position);
    assert!(results_lifecycle(&exit).is_empty()); // The same native state is removed only once.
    assert!(exit.rewards.is_empty() && !e.active());
    assert!(
        !e.handle(&close_packet(0, 1), [0.0; 3], 43, 10003)
            .unwrap()
            .changed
    );
}

#[test]
fn native_close_requires_exact_current_instance_and_malformed_requests_preserve_progress() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    let before = e.state().clone();
    for p in [close_packet(0, 0), close_packet(0, 2), close_packet(999, 1)] {
        let result = e.handle(&p, [0.0; 3], 43, 0).unwrap();
        assert!(!result.changed && result.zoning.is_none());
        assert_eq!(e.state(), &before);
    }
    let mut short = close_packet(0, 1);
    short.pop();
    let mut long = close_packet(0, 1);
    long.push(0);
    for p in [short, long] {
        assert!(e.handle(&p, [0.0; 3], 43, 0).is_err());
        assert_eq!(e.state(), &before);
    }
    // Ui.QuitMiniGame uses the same real header while fighting. It leaves
    // without claiming dungeon completion or rewards.
    let quit = e.handle(&close_packet(0, 1), [0.0; 3], 43, 1).unwrap();
    assert!(quit.zoning.is_some() && quit.completed.is_none() && quit.rewards.is_empty());
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 2).unwrap();
    assert!(
        !e.handle(&close_packet(0, 1), [0.0; 3], 43, 3)
            .unwrap()
            .changed
    );
    assert!(
        e.handle(&close_packet(0, 2), [0.0; 3], 43, 4)
            .unwrap()
            .zoning
            .is_some()
    );
}

#[test]
fn loss_presents_results_without_zoning_and_retry_resets_presentation() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    start(&mut e, 200, [0.0; 3], 0);
    let loss = e.tick([0.0; 3], 43, 100).unwrap();
    assert_eq!(results_lifecycle(&loss), [18, 47, 19]);
    assert!(loss.zoning.is_none() && loss.rewards.is_empty());
    assert_eq!(
        e.state().active.as_ref().unwrap().phase,
        EncounterPhase::Lost
    );
    assert!(e.tick([0.0; 3], 43, 200).unwrap().replies.is_empty());
    e.retry(43, 1000).unwrap();
    e.on_zone_ready(43, 1000).unwrap();
    start(&mut e, 200, [0.0; 3], 1000);
    let again = e.tick([0.0; 3], 43, 1100).unwrap();
    assert_eq!(results_lifecycle(&again), [18, 47, 19]);
    let exit = e.handle(&close_packet(0, 1), [0.0; 3], 43, 1200).unwrap();
    assert!(exit.zoning.is_some() && exit.completed.is_none() && exit.rewards.is_empty());
    assert!(results_lifecycle(&exit).is_empty());
}

#[test]
fn native_bar_one_self_centered_basic_resolves_only_a_live_enemy_in_reach() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    let packet = |bar: i32, slot: i32| {
        [
            vec![36, 0, 10, 0],
            bar.to_le_bytes().to_vec(),
            slot.to_le_bytes().to_vec(),
            0_i32.to_le_bytes().to_vec(),
            1_u64.to_le_bytes().to_vec(),
        ]
        .concat()
    };
    for (bar, slot, position) in [(43, 0, [0.0; 3]), (1, 1, [0.0; 3]), (1, 0, [10.0; 3])] {
        assert!(
            !e.handle(&packet(bar, slot), position, 43, 1000)
                .unwrap()
                .changed
        );
    }
    assert!(!e.tick([0.0; 3], 43, 2000).unwrap().changed); // Invalid casts never arm.
    assert!(e.handle(&packet(1, 0), [0.0; 3], 43, 2100).unwrap().changed);
    assert_eq!(e.state().active.as_ref().unwrap().enemies[0].health, 40);
    assert!(!e.handle(&packet(1, 0), [0.0; 3], 43, 2699).unwrap().changed);
    let win = e.handle(&packet(1, 0), [0.0; 3], 43, 2700).unwrap();
    assert_eq!(win.defeated, vec![500]);
    assert_eq!(win.rewards.len(), 1);
    assert!(!e.handle(&packet(1, 0), [0.0; 3], 43, 4000).unwrap().changed);
}

#[test]
fn cave_waits_for_valid_start_or_attack_and_rejoining_disarms() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    let before = e.state().clone();
    assert!(!e.tick([0.0; 3], 43, 5000).unwrap().changed);
    assert!(!e.attack(201, [100.0; 3], 43, 0, 5000).unwrap().changed);
    start(&mut e, 200, [100.0; 3], 5000);
    assert!(!e.tick([0.0; 3], 43, 6000).unwrap().changed);
    assert_eq!(e.state(), &before);
    assert!(e.attack(201, [0.0; 3], 43, 0, 6000).unwrap().changed);
    let saved = e.state().clone();
    let mut restored = EncounterEngine::load(&f.0, Some(saved.clone())).unwrap();
    restored.on_zone_ready(43, 7000).unwrap();
    assert!(!restored.tick([0.0; 3], 43, 9000).unwrap().changed);
    assert_eq!(restored.state(), &saved);
    start(&mut restored, 200, [0.0; 3], 9000);
    assert!(!restored.tick([0.0; 3], 43, 9099).unwrap().changed);
    assert!(restored.tick([0.0; 3], 43, 9100).unwrap().changed);
    assert_eq!(restored.state().player_health, 0);
    // The original live connection was armed by its valid first attack.
    assert!(e.tick([0.0; 3], 43, 6100).unwrap().changed);
}

#[test]
fn loading_gates_combat_and_ready_creates_native_state() {
    let f = Fixture::new();
    let mut e = f.load();
    let begin = e.begin(411, location(), 1).unwrap();
    assert_eq!(begin.zoning.unwrap().name, "SyntheticCave");
    assert!(!e.attack(201, [0.0; 3], 1, 0, 0).unwrap().changed);
    let ready = e.on_zone_ready(1, 0).unwrap();
    assert!(ready.changed);
    assert_eq!(
        e.state().active.as_ref().unwrap().phase,
        EncounterPhase::Fighting
    );
    assert!(
        ready
            .replies
            .iter()
            .any(|p| p.starts_with(&[41, 0, 114, 0]))
    );
    assert!(
        ready
            .replies
            .iter()
            .any(|p| p.starts_with(&[41, 0, 2, 0]) && p.len() == 21)
    );
    assert!(
        ready
            .replies
            .iter()
            .any(|p| p.starts_with(&[39, 0, 17]) && p.len() == 15)
    );
    let started = ready
        .replies
        .iter()
        .find(|p| p.starts_with(&[39, 0, 17]))
        .unwrap();
    assert_eq!(&started[3..7], &411_i32.to_le_bytes());
    assert!(
        ready
            .replies
            .iter()
            .any(|p| p.starts_with(&[38, 0, 1, 0]) && p.len() == 12)
    );
    assert!(
        ready
            .replies
            .iter()
            .any(|p| p.starts_with(&[32, 0, 9, 0]) && p.len() == 13)
    );
}

#[test]
fn save_restart_win_and_exit_preserve_reward_exactly_once() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 1).unwrap();
    e.on_zone_ready(1, 0).unwrap();
    assert!(e.attack(201, [0.0; 3], 1, 0, 0).unwrap().rewards.is_empty());
    let saved = serde_json::to_vec(e.state()).unwrap();
    let mut e = EncounterEngine::load(&f.0, Some(serde_json::from_slice(&saved).unwrap())).unwrap();
    e.on_zone_ready(1, 100).unwrap();
    let win = e.attack(201, [0.0; 3], 1, 0, 700).unwrap();
    assert_eq!(win.defeated, vec![500]);
    assert_eq!(win.rewards.len(), 1);
    assert_eq!(win.rewards[0].coins, 50);
    assert_eq!(win.rewards[0].items, vec![(1195, 1)]);
    assert_eq!(
        e.state().active.as_ref().unwrap().phase,
        EncounterPhase::Won
    );
    let mut e = EncounterEngine::load(&f.0, Some(e.state().clone())).unwrap();
    let restored_results = e.on_zone_ready(1, 0).unwrap();
    assert!(restored_results.rewards.is_empty());
    assert_eq!(results_lifecycle(&restored_results), [17, 18, 47, 19]);
    assert!(restored_results.zoning.is_none());
    let removed = restored_results
        .replies
        .iter()
        .find(|p| p.starts_with(&[39, 0, 19]))
        .unwrap();
    assert_eq!(&removed[3..7], &411_i32.to_le_bytes());
    assert!(
        e.attack(201, [0.0; 3], 1, 0, 5000)
            .unwrap()
            .rewards
            .is_empty()
    );
    let exit = e.exit().unwrap();
    assert_eq!(exit.completed, Some(411));
    let z = exit.zoning.as_ref().unwrap();
    assert_eq!(z.position, location().position);
    assert_eq!(z.rotation, location().rotation);
    assert!(results_lifecycle(&exit).is_empty());
    assert!(!e.exit().unwrap().changed);
    e.begin(411, location(), 1).unwrap();
    e.on_zone_ready(1, 0).unwrap();
    e.attack(201, [0.0; 3], 1, 4, 0).unwrap();
    assert!(e.state().rewarded.contains(&411));
    assert_eq!(
        e.state().active.as_ref().unwrap().phase,
        EncounterPhase::Won
    );
}

#[test]
fn ai_is_bounded_loss_retry_and_far_attacks_do_not_award() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 1).unwrap();
    e.on_zone_ready(1, 0).unwrap();
    assert!(!e.attack(201, [100.0; 3], 1, 0, 0).unwrap().changed);
    start(&mut e, 200, [0.0; 3], 0);
    assert!(!e.tick([0.0; 3], 1, 99).unwrap().changed);
    let loss = e.tick([0.0; 3], 1, 100).unwrap();
    assert!(loss.changed);
    assert_eq!(e.state().player_health, 0);
    assert_eq!(
        e.state().active.as_ref().unwrap().phase,
        EncounterPhase::Lost
    );
    assert!(loss.rewards.is_empty());
    assert!(!e.tick([0.0; 3], 1, u64::MAX).unwrap().changed);
    let retry = e.retry(1, 2000).unwrap();
    assert_eq!(retry.zoning.unwrap().zone_id, 411);
    assert_eq!(
        e.state().active.as_ref().unwrap().phase,
        EncounterPhase::Loading
    );
    assert_eq!(e.state().player_health, 1000);
    assert!(e.state().rewarded.is_empty());
    e.on_zone_ready(1, 2000).unwrap();
    assert!(!e.tick([0.0; 3], 1, 2001).unwrap().changed);
    assert!(!e.tick([0.0; 3], 1, 5000).unwrap().changed);
    assert_eq!(e.state().player_health, 1000);
}

#[test]
fn melee_requires_contact_and_enemy_motion_attacks_have_native_animation() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    // The previous eight-unit reach accepted this visibly distant melee hit.
    assert!(!e.attack(201, [10.0, 0.0, 0.0], 43, 0, 0).unwrap().changed);
    start(&mut e, 200, [0.0; 3], 0);
    let chase = e.tick([10.0, 0.0, 0.0], 43, 1000).unwrap();
    let position = e.state().active.as_ref().unwrap().enemies[0].position;
    assert_eq!(position, [2.75, 0.0, 0.0]); // Bounded catch-up, no teleport across a gap.
    let movement = chase
        .replies
        .iter()
        .find(|p| p.starts_with(&[125, 0]))
        .unwrap();
    assert_eq!(movement.len(), 36);
    assert_eq!(
        &movement[22..34],
        &[
            1_f32.to_le_bytes(),
            0_f32.to_le_bytes(),
            0_f32.to_le_bytes()
        ]
        .concat()
    );
    assert_eq!(&movement[34..], &[1, 0]);
    assert!(
        chase.replies.iter().any(|p| p.starts_with(&[35, 0, 8, 0])
            && p[12..16] == 3_i32.to_le_bytes()
            && p[20] == 1)
    );
    assert_eq!(e.state().player_health, 1000);
    let attack = e.tick([4.75, 0.0, 0.0], 43, 1100).unwrap();
    assert!(attack.replies.iter().any(|p| p.starts_with(&[35, 0, 8, 0])
        && p[12..16] == 1001_i32.to_le_bytes()
        && p[20] == 0));
    assert_eq!(e.state().player_health, 0); // Fixture's attack deals 1000 only at contact.
}

#[test]
fn pursuer_reaches_real_scale_contact_without_rounding_stall_and_stops_after_aggro() {
    let f = Fixture::new();
    let path = f.0.join("encounters.json");
    let mut content: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    content["dungeons"][0]["enemies"][0]["position"] = json!([292.0, 28.0, 289.0]);
    content["dungeons"][0]["enemies"][0]["damage"] = json!(20);
    fs::write(&path, serde_json::to_vec(&content).unwrap()).unwrap();
    let mut e = f.load();
    e.begin(411, location(), 43).unwrap();
    e.on_zone_ready(43, 0).unwrap();
    let player = [287.0, 27.953926, 286.0];
    start(&mut e, 200, [0.0; 3], 0);
    for now in (100..=2000).step_by(100) {
        e.tick(player, 43, now).unwrap();
    }
    assert_eq!(e.state().player_health, 980);
    e.tick(player, 43, 2100).unwrap();
    assert_eq!(e.state().player_health, 980); // Update frequency does not increase attack rate.
    e.tick(player, 43, 2200).unwrap();
    assert_eq!(e.state().player_health, 960);
    e.tick([300.0, 28.0, 289.0], 43, 2300).unwrap();
    let disengage = e.tick([1000.0, 28.0, 289.0], 43, 2400).unwrap();
    assert!(
        disengage
            .replies
            .iter()
            .any(|p| p.starts_with(&[35, 0, 8, 0])
                && p[12..16] == 1_i32.to_le_bytes()
                && p[20] == 1)
    );
}

#[test]
fn malformed_requests_are_non_mutating_and_cooldown_cannot_burst() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 1).unwrap();
    e.on_zone_ready(1, 0).unwrap();
    let before = e.state().clone();
    for p in [
        vec![26, 0, 8],
        vec![36, 0, 10],
        vec![41, 0, 109],
        vec![26, 0, 8, 0],
        vec![26, 0, 10, 0],
        vec![36, 0, 10, 0],
        vec![41, 0, 109, 0],
        vec![39, 0, 5],
    ] {
        assert!(e.handle(&p, [0.0; 3], 1, 0).is_err());
        assert_eq!(e.state(), &before);
    }
    let hit = e.attack(201, [0.0; 3], 1, 0, 0).unwrap();
    assert!(hit.changed);
    assert!(!e.attack(201, [0.0; 3], 1, 0, 599).unwrap().changed);
    assert!(e.attack(201, [0.0; 3], 1, 0, 600).unwrap().changed);
}

#[test]
fn world_stage_gates_targets_and_persists_individual_defeats() {
    let f = Fixture::new();
    let mut e = f.load();
    assert!(!e.attack(100, [0.0; 3], 1, 4, 0).unwrap().changed);
    assert!(e.set_world_stage(7).changed);
    assert!(
        e.world_initialization()
            .iter()
            .any(|p| p.starts_with(&[35, 0, 2, 0]))
    );
    let win = e.attack(100, [0.0; 3], 1, 4, 0).unwrap();
    assert_eq!(win.defeated, vec![400]);
    assert!(win.rewards.is_empty());
    let mut e = EncounterEngine::load(&f.0, Some(e.state().clone())).unwrap();
    assert!(!e.attack(100, [0.0; 3], 1, 4, 1000).unwrap().changed);
    assert_eq!(
        e.attack(101, [0.0; 3], 1, 4, 2000).unwrap().defeated,
        vec![400]
    );
    e.set_world_stage(8);
    assert!(!e.attack(101, [0.0; 3], 1, 0, 4000).unwrap().changed);
}

#[test]
fn corrupt_content_and_inconsistent_saves_fail_closed() {
    let f = Fixture::new();
    let mut e = f.load();
    e.begin(411, location(), 1).unwrap();
    let mut s = e.state().clone();
    s.active.as_mut().unwrap().phase = EncounterPhase::Won;
    assert!(EncounterEngine::load(&f.0, Some(s)).is_err());
    let mut s = e.state().clone();
    s.world_enemies[0].guid = 999;
    assert!(EncounterEngine::load(&f.0, Some(s)).is_err());
    let mut s = e.state().clone();
    // Above the level-20 health cap (max health now scales with job level).
    s.player_health = 20_001;
    assert!(EncounterEngine::load(&f.0, Some(s)).is_err());
    let mut bytes = fs::read(f.0.join("npc.bin")).unwrap();
    bytes[10] ^= 1;
    fs::write(f.0.join("npc.bin"), bytes).unwrap();
    assert!(EncounterEngine::load(&f.0, None).is_err());
}

#[test]
#[ignore = "requires explicitly supplied private opening content and independent community serializer oracle"]
fn actual_opening_content_and_community_packet_oracle() {
    let directory =
        PathBuf::from(std::env::var_os("OSFR_OPENING_CONTENT").expect("set OSFR_OPENING_CONTENT"));
    let mut e = EncounterEngine::load(&directory, None).unwrap();
    let zone = e.begin(411, location(), 1).unwrap().zoning.unwrap();
    assert_eq!(
        osfr_tools::encounter::zone_begin_packet(&zone),
        fs::read(directory.join("encounter-oracle/zone-begin.bin")).unwrap()
    );
    let ready = e.on_zone_ready(1, 0).unwrap();
    for (file, prefix) in [
        (
            "../encounter-oracle-combat-v1/details-combat.bin",
            vec![41, 0, 114, 0],
        ),
        ("rules-on.bin", vec![62, 0]),
        ("player-hp.bin", vec![38, 0, 1, 0]),
        (
            "../encounter-oracle-combat-v2/game-start-state411.bin",
            vec![39, 0, 17],
        ),
        ("goal.bin", vec![45, 0, 1]),
    ] {
        let expected = fs::read(directory.join("encounter-oracle").join(file)).unwrap();
        let actual = ready
            .replies
            .iter()
            .find(|p| p.starts_with(&prefix))
            .unwrap();
        assert_eq!(actual, &expected, "{file}");
    }
    let again = EncounterEngine::load(&directory, Some(e.state().clone())).unwrap();
    assert_eq!(again.state(), e.state());
    let enemy = e.state().active.as_ref().unwrap().enemies[0].clone();
    let mut position = enemy.position;
    position[0] -= 10.0;
    start(&mut e, osfr_tools::encounter::GUIDE_GUID, zone.position, 0);
    let tick = e.tick(position, 1, 1000).unwrap();
    let movement = tick
        .replies
        .iter()
        .find(|p| p.starts_with(&[125, 0]) && p[2..10] == enemy.guid.to_le_bytes())
        .unwrap();
    assert_eq!(
        movement,
        &fs::read(directory.join("encounter-oracle/movement-v2.bin")).unwrap()
    );
    let hit = e.attack(enemy.guid, enemy.position, 1, 0, 2000).unwrap();
    for (file, prefix) in [
        ("attack.bin", [32, 0, 7, 0]),
        ("enemy-hp.bin", [35, 0, 5, 0]),
    ] {
        assert_eq!(
            hit.replies.iter().find(|p| p.starts_with(&prefix)).unwrap(),
            &fs::read(directory.join("encounter-oracle").join(file)).unwrap()
        );
    }
}

#[test]
#[ignore = "requires private compiled native results vectors; set OSFR_NATIVE_RESULTS_VECTORS"]
fn native_results_close_and_terminal_packets_match_compiled_community_writers() {
    let directory = PathBuf::from(
        std::env::var_os("OSFR_NATIVE_RESULTS_VECTORS").expect("set OSFR_NATIVE_RESULTS_VECTORS"),
    );
    let native_close = fs::read(directory.join("native-score-exit-instance1.bin")).unwrap();
    assert_eq!(native_close, close_packet(0, 1));
    for won in [true, false] {
        let f = Fixture::new();
        let mut e = f.load();
        e.begin(411, location(), 43).unwrap();
        e.on_zone_ready(43, 0).unwrap();
        let result = if won {
            e.attack(201, [0.0; 3], 43, 0, 0).unwrap();
            e.attack(201, [0.0; 3], 43, 0, 600).unwrap()
        } else {
            start(&mut e, 200, [0.0; 3], 0);
            e.tick([0.0; 3], 43, 100).unwrap()
        };
        let expected_over = fs::read(directory.join(if won {
            "terminal-win-state411.bin"
        } else {
            "terminal-loss-state411.bin"
        }))
        .unwrap();
        let expected_remove = fs::read(directory.join("present-results-state411.bin")).unwrap();
        assert_eq!(
            result
                .replies
                .iter()
                .find(|p| p.starts_with(&[39, 0, 18]))
                .unwrap(),
            &expected_over,
        );
        assert_eq!(
            result
                .replies
                .iter()
                .find(|p| p.starts_with(&[39, 0, 19]))
                .unwrap(),
            &expected_remove,
        );
        let expected_scores = fs::read(directory.join(if won {
            "end-scores-win-state411.bin"
        } else {
            "end-scores-loss-state411.bin"
        }))
        .unwrap();
        assert_eq!(
            result
                .replies
                .iter()
                .find(|p| p.starts_with(&[39, 0, 47]))
                .unwrap(),
            &expected_scores,
        );
        assert_eq!(results_lifecycle(&result), [18, 47, 19]);
        assert!(result.zoning.is_none());
        assert!(
            e.handle(&native_close, [0.0; 3], 43, 1000)
                .unwrap()
                .zoning
                .is_some()
        );
    }
}
