//! Score coherence for the cave results panel.
//!
//! Proven binding (`notes/Research/Cave Results Score Binding.md`): TOTAL SCORE
//! is the fifth i32 (Points) of the `39/47` row whose Order/Score-Type is 4.
//! The lead's policy reuses the existing local objective award as the won
//! total and sends zero on a loss, without inventing kill points or historical
//! score. The win golden below is the exact byte output of the independently
//! compiled pinned community writer with `Value = 4`, `Points = 5000`
//! (`lab/reports/cave-score-binding-20261009T121500Z/writer-probe.log`).
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

/// Independently compiled `MiniGameGameEndScorePacket` (StateId/GameId 411,
/// instance 1, `scoreEnemiesDefeated` Value 4 Points 0, `scoreTotalScore`
/// Order 4 Points 5000).
const WIN_GOLDEN: &str = "27002f9b010000ffffffff9b010000020000001400000073636f7265456e656d6965734465666561746564ffffffff0000000004000000ffffffff000000000f00000073636f7265546f74616c53636f7265ffffffff04000000ffffffffffffffff88130000";

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
            "osfr-encounter-score-{}-{}",
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
        let actor = |guid: u64, def: i32, hp: u32, damage: u32| {
            json!({"guid":guid,"definition_id":def,"model_id":3,"name_id":100,
                "name":"Synthetic actor","texture_alias":"","position":[2.0,0.0,0.0],
                "max_health":hp,"damage":damage,"boss":false,"stages":[]})
        };
        let zone = |id: u32, name: &str| {
            json!({"zone_id":id,"name":name,"sky":"","geometry_id":214,
                "position":[0.0,0.0,0.0],"rotation":[0.0,0.0,0.0],"tutorial":id!=0})
        };
        let c = json!({
            "schema_version":1,
            "template_file":"npc.bin",
            "template_sha256":format!("{:x}",Sha256::digest(t)),
            "world":zone(0,"SyntheticWorld"),
            "world_enemies":[actor(100,400,100,10)],
            "dungeons":[{
                "id":411,"title_id":12,"description_id":13,"objective_id":14,
                "zone":zone(411,"SyntheticCave"),
                "guide":actor(200,0,1000,0),
                "enemies":[actor(201,400,100,1000),actor(202,400,100,1000),
                           actor(203,400,100,1000),actor(204,400,100,1000)],
                "reward_coins":50,"reward_items":[[1195,1]]
            }]
        });
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

fn start(engine: &mut EncounterEngine, guid: u64, position: [f32; 3], now: u64) {
    let packet = [
        vec![26, 0, 10, 0],
        guid.to_le_bytes().to_vec(),
        104_i32.to_le_bytes().to_vec(),
    ]
    .concat();
    engine.handle(&packet, position, 43, now).unwrap();
}

fn close_packet(encounter: i32, instance: i32) -> Vec<u8> {
    [
        vec![41, 0, 109, 0],
        encounter.to_le_bytes().to_vec(),
        instance.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn attack_until_dead(
    engine: &mut EncounterEngine,
    guid: u64,
    mut now: u64,
) -> (u64, osfr_tools::encounter::EncounterOutcome) {
    for _ in 0..32 {
        let state = engine.state().clone();
        let enemies = state
            .active
            .as_ref()
            .map(|active| &active.enemies)
            .unwrap_or(&state.world_enemies);
        if enemies
            .iter()
            .find(|enemy| enemy.guid == guid)
            .is_some_and(|enemy| enemy.health == 0)
        {
            unreachable!("the killing blow returns before this check");
        }
        let outcome = engine.attack(guid, [0.0; 3], 43, 0, now).unwrap();
        now += 700;
        let dead = engine
            .state()
            .active
            .as_ref()
            .unwrap()
            .enemies
            .iter()
            .find(|enemy| enemy.guid == guid)
            .is_some_and(|enemy| enemy.health == 0);
        if dead {
            return (now, outcome);
        }
    }
    panic!("enemy {guid} did not die");
}

fn score_rows(packet: &[u8]) -> Vec<(String, [i32; 5])> {
    assert!(packet.starts_with(&[39, 0, 47]), "not a 39/47 score packet");
    let count = i32::from_le_bytes(packet[15..19].try_into().unwrap());
    let mut rows = Vec::new();
    let mut at = 19;
    for _ in 0..count {
        let len = i32::from_le_bytes(packet[at..at + 4].try_into().unwrap()) as usize;
        at += 4;
        let name = String::from_utf8(packet[at..at + len].to_vec()).unwrap();
        at += len;
        let mut fields = [0_i32; 5];
        for field in &mut fields {
            *field = i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
            at += 4;
        }
        rows.push((name, fields));
    }
    assert_eq!(at, packet.len(), "packet has trailing bytes");
    rows
}

fn frame(out: &osfr_tools::encounter::EncounterOutcome, sub: u8) -> Option<&Vec<u8>> {
    out.replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, sub]))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn won_total_row_carries_the_local_objective_award() {
    let fixture = Fixture::new();
    let mut engine = fixture.load();
    engine.begin(411, location(), 43).unwrap();
    engine.on_zone_ready(43, 0).unwrap();
    start(&mut engine, 200, [0.0; 3], 0);
    let mut now = 1000;
    let mut won = None;
    for guid in [201_u64, 202, 203, 204] {
        let (next, outcome) = attack_until_dead(&mut engine, guid, now);
        now = next;
        won = Some(outcome);
    }
    let won = won.expect("four kills");
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        EncounterPhase::Won
    );
    let scores = frame(&won, 47).expect("win score rows");
    assert_eq!(
        hex(scores),
        WIN_GOLDEN,
        "the win 39/47 must equal the compiled community writer with total 5000"
    );
    let rows = score_rows(scores);
    assert_eq!(rows[0].0, "scoreEnemiesDefeated");
    assert_eq!(rows[0].1, [-1, 0, 4, -1, 0], "defeat count stays actual");
    assert_eq!(rows[1].0, "scoreTotalScore");
    assert_eq!(rows[1].1, [-1, 4, -1, -1, 5000]);
}

#[test]
fn loss_total_row_stays_zero_and_keeps_the_real_defeat_count() {
    let fixture = Fixture::new();
    let mut engine = fixture.load();
    engine.begin(411, location(), 43).unwrap();
    engine.on_zone_ready(43, 0).unwrap();
    start(&mut engine, 200, [0.0; 3], 0);
    let mut now = 1000;
    for guid in [201_u64, 202, 203] {
        let (next, _) = attack_until_dead(&mut engine, guid, now);
        now = next;
    }
    let loss = engine.tick([0.0; 3], 43, now + 700).unwrap();
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        EncounterPhase::Lost
    );
    let scores = frame(&loss, 47).expect("loss score rows");
    let rows = score_rows(scores);
    assert_eq!(rows[0].1, [-1, 0, 3, -1, 0], "three real defeats");
    assert_eq!(rows[1].1, [-1, 4, -1, -1, 0], "a loss awards no total");
}

#[test]
fn duplicate_presentation_and_removal_ack_do_not_exit() {
    let fixture = Fixture::new();
    let mut engine = fixture.load();
    engine.begin(411, location(), 43).unwrap();
    engine.on_zone_ready(43, 0).unwrap();
    start(&mut engine, 200, [0.0; 3], 0);
    let mut now = 1000;
    for guid in [201_u64, 202, 203, 204] {
        let (next, _) = attack_until_dead(&mut engine, guid, now);
        now = next;
    }
    let repeat = engine.tick([0.0; 3], 43, now).unwrap();
    assert!(frame(&repeat, 47).is_none(), "results present exactly once");
    assert!(
        engine.active(),
        "the terminal state is held until a real close"
    );
    let ack = engine.handle(&[39, 0, 7, 0], [0.0; 3], 43, now).unwrap();
    assert!(
        ack.zoning.is_none() && engine.active(),
        "the client removal ACK must not exit"
    );
    let wrong = engine
        .handle(&close_packet(0, 2), [0.0; 3], 43, now)
        .unwrap();
    assert!(
        wrong.zoning.is_none() && engine.active(),
        "a close for another instance is refused"
    );
    let bound = engine
        .handle(&close_packet(0, 1), [0.0; 3], 43, now)
        .unwrap();
    assert!(bound.zoning.is_some(), "the bound instance close exits");
    assert!(!engine.active(), "the run clears only on the real close");
}
