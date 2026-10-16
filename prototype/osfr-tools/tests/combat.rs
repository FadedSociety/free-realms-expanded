use osfr_tools::combat::{
    Combat, CombatState, ENEMY_GUID, INTERACTION_ATTACK, INTERACTION_QUEST, NPC_GUID,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SERIAL: AtomicU64 = AtomicU64::new(0);
struct Content(PathBuf);
impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Content {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-combat-test-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        // Synthetic fields only. Unknown fields deliberately retain recognizable sentinels.
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[33..37].copy_from_slice(&1_f32.to_le_bytes());
        template[49..53].copy_from_slice(&1_f32.to_le_bytes());
        template[61..65].copy_from_slice(&1_f32.to_le_bytes());
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        template[250] = 0x5a;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let config = serde_json::json!({"schema_version":1,"template_file":"npc.bin",
            "template_sha256":format!("{:x}",Sha256::digest(&template)),
            "npc":{"definition_id":0,"model_id":3311,"name":"Test Guide","texture_alias":"","position":[0.0,0.0,0.0]},
            "enemy":{"definition_id":1294,"model_id":4,"name":"Test Enemy","texture_alias":"longjohns","position":[4.0,0.0,0.0]},
            "max_health":100,"hit_damage":25,"range":8.0,"cooldown_ms":600});
        fs::write(
            dir.join("combat.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        Self(dir)
    }
    fn load(&self) -> Combat {
        Combat::load(&self.0, None).unwrap()
    }
}
fn request(guid: u64) -> Vec<u8> {
    let mut bytes = vec![26, 0, 8, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.push(0);
    bytes
}
fn select(guid: u64, id: i32) -> Vec<u8> {
    let mut bytes = vec![26, 0, 10, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.extend(id.to_le_bytes());
    bytes
}
fn ability(guid: u64) -> Vec<u8> {
    let mut bytes = vec![36, 0, 10, 0];
    for value in [4895_i32, 0, 0] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(guid.to_le_bytes());
    bytes
}
fn u32_at(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}

#[test]
fn spawn_preserves_opaque_fields_and_patches_correct_variable_offsets() {
    let content = Content::new();
    let combat = content.load();
    let packets = combat.initialization([999.0; 3]);
    assert_eq!(packets.len(), 3);
    let npc = &packets[0];
    let enemy = &packets[1];
    assert_eq!(&npc[4..12], &NPC_GUID.to_le_bytes());
    assert_eq!(&enemy[4..12], &ENEMY_GUID.to_le_bytes());
    assert_eq!(u32_at(enemy, 16), 4);
    assert_eq!(u32_at(enemy, 77), 0);
    assert_eq!(u32_at(enemy, 37), 4_f32.to_bits());
    assert_eq!(u32_at(enemy, 81), 9);
    assert_eq!(&enemy[85..94], b"longjohns");
    let name_offset = 106 + 9;
    assert_eq!(u32_at(enemy, name_offset), 10);
    assert_eq!(&enemy[name_offset + 4..name_offset + 14], b"Test Enemy");
    assert_eq!(u32_at(enemy, 133 + 9 + 10), 8);
    assert_eq!(enemy[224 + 9 + 10], 1);
    assert_eq!(enemy[250 + 9 + 10], 0x5a);
    // Independent byte layout for relevance: count2, guide talk cursor5, enemy combat cursor1.
    let mut expected = vec![35, 0, 12, 0, 2, 0, 0, 0];
    expected.extend(NPC_GUID.to_le_bytes());
    expected.extend([1, 5, 0]);
    expected.extend(ENEMY_GUID.to_le_bytes());
    expected.extend([1, 1, 0]);
    assert_eq!(packets[2], expected);
}

#[test]
fn interaction_and_menu_layout_are_bounded_by_identity_and_range() {
    let content = Content::new();
    let mut combat = content.load();
    let result = combat.handle(&request(NPC_GUID), [0.0; 3], 0).unwrap();
    assert_eq!(result.interacted, Some(NPC_GUID));
    assert!(!result.changed);
    let menu = &result.replies[0];
    assert_eq!(&menu[..4], &[26, 0, 9, 0]);
    assert_eq!(&menu[4..12], &NPC_GUID.to_le_bytes());
    assert_eq!(menu[12], 0);
    assert_eq!(u32_at(menu, 13), 2);
    assert_eq!(u32_at(menu, 17), INTERACTION_QUEST as u32);
    assert_eq!(u32_at(menu, 25), 79);
    assert_eq!(&menu[menu.len() - 2..], &[1, 0]);
    let enemy = combat.handle(&request(ENEMY_GUID), [0.0; 3], 0).unwrap();
    assert_eq!(enemy.replies[0][12], 1);
    assert_eq!(u32_at(&enemy.replies[0], 25), 22498);
    assert!(
        combat
            .handle(&request(123), [0.0; 3], 0)
            .unwrap()
            .replies
            .is_empty()
    );
    assert!(
        combat
            .handle(&request(NPC_GUID), [9.0, 0.0, 0.0], 0)
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(
        combat
            .handle(&select(NPC_GUID, INTERACTION_QUEST), [0.0; 3], 0)
            .unwrap()
            .interacted,
        Some(NPC_GUID)
    );
    assert_eq!(
        combat
            .handle(&select(ENEMY_GUID, INTERACTION_QUEST), [0.0; 3], 0)
            .unwrap()
            .interacted,
        None
    );
}

#[test]
fn hits_cooldown_defeat_despawn_and_respawn_do_not_duplicate_rewards() {
    let content = Content::new();
    let mut combat = content.load();
    assert!(!combat.attack([13.0, 0.0, 0.0], 0).changed);
    assert!(!combat.attack([f32::NAN, 0.0, 0.0], 0).changed);
    let first = combat
        .handle(&select(ENEMY_GUID, INTERACTION_ATTACK), [0.0; 3], 1000)
        .unwrap();
    assert!(first.changed);
    assert_eq!(combat.state().enemy_health, 75);
    assert!(!combat.attack([0.0; 3], 1599).changed);
    assert!(!combat.attack([0.0; 3], 999).changed);
    assert!(combat.attack([0.0; 3], 1600).changed);
    assert!(
        combat
            .handle(&ability(ENEMY_GUID), [0.0; 3], 2200)
            .unwrap()
            .changed
    );
    let kill = combat.attack([0.0; 3], 2800);
    assert_eq!(kill.defeated, Some(1294));
    assert_eq!(
        combat.state(),
        CombatState {
            enemy_health: 0,
            defeats: 1
        }
    );
    let mut remove = vec![35, 0, 3, 0, 1, 0];
    remove.extend(ENEMY_GUID.to_le_bytes());
    remove.push(0);
    remove.extend([0; 16]);
    assert_eq!(kill.replies[0], remove);
    assert_eq!(kill.replies.len(), 2);
    assert_eq!(combat.initialization([0.0; 3]).len(), 2);
    assert!(!combat.attack([0.0; 3], 3400).changed);
    assert_eq!(combat.attack([0.0; 3], 4000).defeated, None);
    assert!(!combat.respawn([100.0; 3]).changed);
    assert!(combat.respawn([0.0; 3]).changed);
    assert_eq!(
        combat.state(),
        CombatState {
            enemy_health: 100,
            defeats: 1
        }
    );
    assert!(!combat.respawn([0.0; 3]).changed);
    assert!(combat.attack([0.0; 3], 0).changed); // New encounter resets ephemeral cooldown.
}

#[test]
fn malformed_known_requests_never_mutate_and_clones_isolate_transaction_state() {
    let content = Content::new();
    let mut combat = content.load();
    let initial = combat.state();
    let mut bad_boolean = request(NPC_GUID);
    bad_boolean[12] = 2;
    let mut bad_ability = ability(ENEMY_GUID);
    bad_ability[8..12].copy_from_slice(&(-1_i32).to_le_bytes());
    for bad in [
        vec![26, 0, 8, 0],
        vec![26, 0, 10, 0],
        vec![36, 0, 10, 0],
        bad_boolean,
        bad_ability,
    ] {
        assert!(combat.handle(&bad, [0.0; 3], 0).is_err());
        assert_eq!(combat.state(), initial);
    }
    let mut transaction = combat.clone();
    transaction.attack([0.0; 3], 0);
    assert_eq!(combat.state(), initial);
    assert_eq!(transaction.state().enemy_health, 75);
    let bytes = serde_json::to_vec(&transaction.state()).unwrap();
    let restored: CombatState = serde_json::from_slice(&bytes).unwrap();
    let mut restored = Combat::load(&content.0, Some(restored)).unwrap();
    assert_eq!(restored.state().enemy_health, 75);
    assert!(restored.attack([0.0; 3], 0).changed);
    assert!(
        Combat::load(
            &content.0,
            Some(CombatState {
                enemy_health: 101,
                defeats: 0
            })
        )
        .is_err()
    );
}

#[test]
fn corrupt_private_template_and_escaping_filename_are_rejected() {
    let content = Content::new();
    let mut bytes = fs::read(content.0.join("npc.bin")).unwrap();
    bytes[250] ^= 1;
    fs::write(content.0.join("npc.bin"), bytes).unwrap();
    assert!(Combat::load(&content.0, None).is_err());
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(content.0.join("combat.json")).unwrap()).unwrap();
    config["template_file"] = "../npc.bin".into();
    fs::write(
        content.0.join("combat.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    assert!(Combat::load(&content.0, None).is_err());
}

#[test]
#[ignore = "requires explicitly supplied private combat content; OSFR_COMBAT_CONTENT directory"]
fn private_capture_template_loads_and_relocated_packets_keep_expected_shape() {
    let path = std::env::var("OSFR_COMBAT_CONTENT").expect("set OSFR_COMBAT_CONTENT explicitly");
    let mut combat = Combat::load(path, None).unwrap();
    let packets = combat.initialization([-1412.0295, -28.21986, 344.7286]);
    assert_eq!(packets.len(), 3);
    assert_eq!(&packets[0][..4], &[35, 0, 2, 0]);
    assert_eq!(&packets[1][..4], &[35, 0, 2, 0]);
    assert_eq!(
        combat
            .handle(&request(NPC_GUID), [-1412.0295, -28.21986, 344.7286], 0)
            .unwrap()
            .interacted,
        Some(NPC_GUID)
    );
    for now in [0, 600, 1200] {
        assert!(
            combat
                .attack([-1412.0295, -28.21986, 344.7286], now)
                .changed
        );
    }
    assert_eq!(
        combat
            .attack([-1412.0295, -28.21986, 344.7286], 1800)
            .defeated,
        Some(1294)
    );
}
