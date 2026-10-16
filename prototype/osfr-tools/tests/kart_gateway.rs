use osfr_tools::{
    character_store::CharacterState,
    kart_entry::{ProxiedVehicle, RaceConfig, RaceEntryContent, RaceEntryContext},
    kart_gateway::{KartGateway, KartGatewayContent},
    kart_progress_native::ProgressPayload,
    zone_transition::ZoneTransition,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "kart-gateway-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let fixture = Self(path);
        fixture.write(&content());
        fixture
    }
    fn write(&self, content: &KartGatewayContent) {
        fs::write(
            self.0.join("kart_entry.json"),
            serde_json::to_vec(content).unwrap(),
        )
        .unwrap();
    }
    fn load(&self) -> KartGateway {
        KartGateway::load(&self.0, None).unwrap().unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [10., 20., 30.],
        rotation: [0., 0., 1.],
    }
}
fn content() -> KartGatewayContent {
    let context = RaceEntryContext {
        session: 123,
        instance_id: 1008,
        encounter_id: 917566,
        scene: ZoneTransition {
            envelope_flag: 1,
            scene: "sg_kt_track".into(),
            type_id: 2,
            position: [719., 72., 421., 1.],
            rotation: [0., 0., 1., 0.],
            sky: "sky.xml".into(),
            geometry_id: 101,
            ..Default::default()
        },
    };
    let details = |state: u32| {
        let mut b = vec![0u8; 391];
        b[..4].copy_from_slice(&[41, 0, 114, 0]);
        for (offset, word) in [
            (4, context.encounter_id),
            (8, context.instance_id),
            (12, context.encounter_id),
            (16, state),
        ] {
            b[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        b[32..40].copy_from_slice(&context.session.to_le_bytes());
        for i in 0..3 {
            b[51 + i * 4..55 + i * 4].copy_from_slice(&context.scene.position[i].to_le_bytes());
        }
        b
    };
    KartGatewayContent {
        schema_version: 1,
        details_loading: details(2),
        details_ready: details(4),
        vehicle_guid_offset: 0,
        race: None,
        entry: RaceEntryContent {
            race_config: RaceConfig { words: vec![0; 21] },
            entry_vehicles: vec![ProxiedVehicle {
                body_asset: "test.cdt".into(),
                first_attribute: 400.,
                attribute_block: 123u64.to_le_bytes().to_vec(),
            }],
            client_data: vec![0; 85],
            player_count: 1,
            npc_count: 0,
            progress: vec![ProgressPayload {
                racer_guid: 123,
                slot: 0,
                unknown_0c_bits: 0,
                unknown_10_bits: 0,
                laps_completed: 0,
                local_racer: 0,
                lap_limit: 3,
                unknown_1d_bits: 0,
                best_lap_bits: 0,
                total_time_bits: 0,
                tail_0x29: 123,
                tail_0x31: 0,
                tail_0x39: 0,
                tail_0x41: 0,
            }],
        },
        context,
    }
}
fn register(guid: u64) -> Vec<u8> {
    let mut b = vec![33, 0, 9, 0];
    b.extend(guid.to_le_bytes());
    b.extend([0; 8]);
    b
}
#[test]
fn entry_rebinds_known_slots_and_preserves_quaternion_and_return() {
    let f = Fixture::new();
    let mut g = f.load();
    let before = g.state().clone();
    let (replies, scene) = g.begin(&character()).unwrap();
    assert_eq!(scene.rotation[3], 0.);
    assert_eq!(scene.id, 48_000_001);
    assert_eq!(&replies[0][32..40], &1u64.to_le_bytes());
    assert_eq!(&replies[0][8..12], &48_000_001u32.to_le_bytes());
    let init = g.initialize().unwrap();
    assert_eq!(init[1].len(), 21);
    assert_eq!(&init[1][8..12], &48_000_001u32.to_le_bytes());
    let mut expected_status = vec![35, 0, 20, 0];
    expected_status.extend(1u64.to_le_bytes());
    expected_status.extend(0x401u32.to_le_bytes());
    assert_eq!(init[3], expected_status);
    assert_eq!(init[2].len(), init[0].len());
    let mut repeated = init[0].clone();
    let activity = repeated.len() - 15;
    repeated[activity..activity + 4].fill(0);
    assert_eq!(init[2], repeated);
    assert!(g.handle_entry(&register(123)).unwrap().is_empty());
    let started = g.handle_entry(&register(1)).unwrap();
    assert_eq!(started.len(), 4);
    let data = started.iter().find(|b| b[..4] == [33, 0, 5, 0]).unwrap();
    assert_eq!(&data[81..89], &[0; 8]);
    let mut racer = character();
    racer.zone_id = 48_000_001;
    assert!(g.supports_location(&racer));
    assert_eq!(g.exit(), Some(character()));
    assert!(!g.active());
    assert_eq!(g.state().next_instance, before.next_instance + 1);
}
#[test]
fn duplicate_begin_and_malformed_register_leave_state_unchanged() {
    let f = Fixture::new();
    let mut g = f.load();
    g.begin(&character()).unwrap();
    let before = g.state().clone();
    assert!(g.begin(&character()).is_err());
    assert!(g.handle_entry(&[33, 0, 9, 0]).is_err());
    assert_eq!(g.state(), &before);
}
#[test]
fn active_snapshot_roundtrip_and_content_binding() {
    let f = Fixture::new();
    let mut g = f.load();
    g.begin(&character()).unwrap();
    let state = serde_json::from_slice(&serde_json::to_vec(g.state()).unwrap()).unwrap();
    let restored = KartGateway::load(&f.0, Some(state)).unwrap().unwrap();
    assert_eq!(restored.state(), g.state());
    let mut content = content();
    content.entry.npc_count = 1;
    f.write(&content);
    assert!(KartGateway::load(&f.0, Some(g.state().clone())).is_err());
}
#[test]
fn bad_proven_slots_and_spawn_refused_before_entry() {
    let f = Fixture::new();
    let mut c = content();
    c.vehicle_guid_offset = usize::MAX;
    f.write(&c);
    assert!(KartGateway::load(&f.0, None).is_err());
    let mut c = content();
    c.details_ready[51] = 255;
    f.write(&c);
    assert!(KartGateway::load(&f.0, None).is_err());
    let mut c = content();
    c.entry.progress[0].tail_0x29 = 99;
    f.write(&c);
    assert!(KartGateway::load(&f.0, None).is_err());
}
#[test]
fn absent_optional_content_and_missing_required_content() {
    let f = Fixture::new();
    let g = f.load();
    fs::remove_file(f.0.join("kart_entry.json")).unwrap();
    assert!(KartGateway::load(&f.0, None).unwrap().is_none());
    assert!(KartGateway::load(&f.0, Some(g.state().clone())).is_err());
}

#[test]
fn original_close_is_bound_to_the_live_encounter_and_exact_width() {
    let f = Fixture::new();
    let mut g = f.load();
    g.begin(&character()).unwrap();
    let mut packet = vec![39, 0, 7];
    packet.extend(917566u32.to_le_bytes());
    packet.extend([255; 8]);
    assert!(g.accepts_close(&packet).unwrap());
    packet[3..7].copy_from_slice(&123u32.to_le_bytes());
    assert!(!g.accepts_close(&packet).unwrap());
    assert!(g.accepts_close(&[39, 0, 7]).is_err());
    g.exit();
    assert!(!g.accepts_close(&packet).unwrap());
}
