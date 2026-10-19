use osfr_tools::derby::{
    DEMO_DERBY_TYPE_ID, DerbyAction, DerbyConfig, DerbyMessage, DerbyPhase, DerbyPickup,
    DerbyPickupBody, DerbyRules, DerbyService, DerbyVehicleTemplate, ParticipantKey, PickupKind,
    SUBTYPE_SPECS, VehicleCreate, VehicleHitReport, VehiclePose, VehicleState, decode_inner,
    encode_inner,
};

const OWNER: ParticipantKey = ParticipantKey(0x1000_0000_0000_0001);
const OTHER: ParticipantKey = ParticipantKey(0x1000_0000_0000_0002);
const INSTANCE: i32 = 44;
const GAME_ROW: i32 = 39;

fn template() -> DerbyVehicleTemplate {
    DerbyVehicleTemplate {
        name: Vec::new(),
        definition: vec![0x11, 0x22, 0x33],
        start_transform_bits: [0; 16],
        unknown_flag: false,
        unknown_value_bits: 0,
    }
}

fn rules() -> DerbyRules {
    DerbyRules {
        max_drivers: 8,
        start_health: 20,
        contact_damage_per_unit: 1.0,
        contact_cooldown_ms: 500,
        contact_radius_m: 5.0,
        max_impact: 50.0,
        max_pose_age_ms: 5_000,
        max_damage_per_contact: 100,
        lobby_timeout_ms: 120_000,
        pickup_radius: 3.0,
        health_pickup: 10,
        boost_pickup: 15.0,
        shield_duration_ms: 5_000,
        attack_duration_ms: 5_000,
        ultimate_duration_ms: 5_000,
        attack_multiplier: 2.0,
        ultimate_multiplier: 2.0,
        respawn_delay_ms: 4_000,
        results_duration_ms: 1_000,
        knockout_points: 2,
        leader_knockout_points: 3,
        knockout_point_loss: 1,
        score_target: None,
        time_limit_ms: None,
        config: DerbyConfig {
            value_a_bits: 12.5_f32.to_bits(),
            value_b: 1_234,
        },
    }
}

fn pickup(id: u32, kind: PickupKind, x: f32) -> DerbyPickup {
    DerbyPickup {
        id,
        definition_id: 900 + id,
        kind,
        position_bits: [x.to_bits(), 0.0_f32.to_bits(), 0.0_f32.to_bits(), 0],
        unknown_floats_bits: [0; 2],
        response_parameters_bits: [5.0_f32.to_bits(), 2.0_f32.to_bits(), 0],
    }
}

fn service(driver_keys: &[ParticipantKey], pickups: Vec<DerbyPickup>) -> DerbyService {
    service_with_rules(driver_keys, pickups, rules())
}

fn service_with_rules(
    driver_keys: &[ParticipantKey],
    pickups: Vec<DerbyPickup>,
    rules: DerbyRules,
) -> DerbyService {
    let mut service = DerbyService::new(
        INSTANCE,
        GAME_ROW,
        driver_keys[0],
        template(),
        rules,
        pickups,
    )
    .unwrap();
    for key in driver_keys.iter().copied().skip(1) {
        assert!(service.add_human(key, template()).unwrap());
    }
    service
}

fn request_start(instance_id: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, 5];
    packet.extend(instance_id.to_le_bytes());
    packet.extend((-1_i32).to_le_bytes());
    packet.extend((-1_i32).to_le_bytes());
    packet
}

fn pose(key: ParticipantKey, position: [f32; 3]) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: [
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            position[0],
            position[1],
            position[2],
            1.0,
        ]
        .map(f32::to_bits),
        vector_a_bits: [0; 4],
        vector_b_bits: [0; 4],
        controls_bits: [0; 3],
        flags: [0; 4],
        sequence: 1,
    }))
    .unwrap()
}

fn messages(actions: &[DerbyAction]) -> Vec<DerbyMessage> {
    actions
        .iter()
        .map(|action| decode_inner(&action.packet).unwrap())
        .collect()
}

fn activate(service: &mut DerbyService, humans: &[ParticipantKey]) -> Vec<DerbyAction> {
    let offer = service.offer_packet().unwrap();
    assert_eq!(&offer[..3], &[39, 0, 16]);
    assert_eq!(i32::from_le_bytes(offer[35..39].try_into().unwrap()), 11);
    let start = service
        .handle_minigame_packet(&request_start(INSTANCE))
        .unwrap();
    assert_eq!(&start[..3], &[39, 0, 17]);

    let mut setup = Vec::new();
    for key in humans {
        let packet = encode_inner(&DerbyMessage::RegisterPlayer {
            key: *key,
            unknown_key: 0,
        })
        .unwrap();
        setup.extend(service.handle_client(*key, &packet, 0).unwrap());
    }
    let sent = messages(&setup);
    assert_eq!(sent.first(), Some(&DerbyMessage::SetConfig(rules().config)));
    assert!(
        sent.iter()
            .any(|message| *message == DerbyMessage::PlayerCount(humans.len() as i32))
    );
    assert!(sent.iter().any(|message| matches!(message, DerbyMessage::NpcCount(count) if *count == 8 - humans.len() as i32)));
    assert!(matches!(sent.last(), Some(DerbyMessage::GameState(3))));
    setup
}

fn run_to_live(service: &mut DerbyService, humans: &[ParticipantKey]) {
    activate(service, humans);
    for phase in 3..=8 {
        for key in humans {
            let packet = encode_inner(&DerbyMessage::PlayerPhase { key: *key, phase }).unwrap();
            service.handle_client(*key, &packet, phase as u64).unwrap();
        }
    }
    for count in (0..=4).rev() {
        for key in humans {
            let packet = encode_inner(&DerbyMessage::CountdownStatus { key: *key, count }).unwrap();
            service
                .handle_client(*key, &packet, 100 + (4 - count) as u64 * 1_000)
                .unwrap();
        }
    }
    assert_eq!(service.phase(), DerbyPhase::Running);
}

#[test]
fn all_twenty_client_records_roundtrip_with_reader_widths() {
    let messages = vec![
        DerbyMessage::GameState(3),
        DerbyMessage::PlayerPhase {
            key: OWNER,
            phase: 2,
        },
        DerbyMessage::PlayerCount(1),
        DerbyMessage::NpcCount(7),
        DerbyMessage::VehiclePose(VehiclePose {
            key: OWNER,
            transform_bits: [0; 16],
            vector_a_bits: [0; 4],
            vector_b_bits: [0; 4],
            controls_bits: [0; 3],
            flags: [0; 4],
            sequence: 9,
        }),
        DerbyMessage::RegisterPlayer {
            key: OWNER,
            unknown_key: 12,
        },
        DerbyMessage::CountdownStatus {
            key: OWNER,
            count: 4,
        },
        DerbyMessage::DestroyVehicle(OWNER),
        DerbyMessage::KickVehicle(OWNER),
        DerbyMessage::CreateVehicle(VehicleCreate {
            key: OWNER,
            name: b"npc".to_vec(),
            definition: vec![1, 2, 3, 4, 5],
            start_transform_bits: [0; 16],
            unknown_flag: true,
            unknown_value_bits: 0,
        }),
        DerbyMessage::VehicleState(VehicleState {
            key: OWNER,
            health: 20,
            score: 3,
            unknown_values: [4, 5],
            damage_dealt: 6,
            damage_received: 7,
            flags: [0, 1, 0],
            unknown_floats_bits: [0; 4],
        }),
        DerbyMessage::CreatePickup(DerbyPickupBody {
            id: 1,
            definition_id: 901,
            position_bits: [2.0_f32.to_bits(), 0, 0, 0],
            unknown_floats_bits: [0; 2],
        }),
        DerbyMessage::DestroyPickup(1),
        DerbyMessage::PickupHitRequest {
            key: OWNER,
            position_bits: [0; 4],
        },
        DerbyMessage::PickupHitResponse {
            key: OWNER,
            kind: PickupKind::Ultimate as u32,
            parameters_bits: [0; 3],
        },
        DerbyMessage::VehicleHit(VehicleHitReport {
            reporter: OWNER,
            other: OTHER,
            impact_bits: 10.0_f32.to_bits(),
        }),
        DerbyMessage::ItemExpired(3),
        DerbyMessage::ResetVehicle(OWNER),
        DerbyMessage::SetConfig(rules().config),
        DerbyMessage::Announcer {
            kind: 2,
            sub_kind: 0,
            first: OWNER,
            second: OTHER,
        },
    ];
    let expected_payload_lengths = [
        4,
        12,
        4,
        4,
        128,
        16,
        12,
        8,
        8,
        8 + 4 + 3 + 5 + 69,
        51,
        32,
        4,
        24,
        24,
        20,
        4,
        8,
        8,
        24,
    ];
    assert_eq!(SUBTYPE_SPECS.len(), 20);
    assert_eq!(DEMO_DERBY_TYPE_ID, 11);
    for (index, (message, expected_len)) in
        messages.iter().zip(expected_payload_lengths).enumerate()
    {
        let packet = encode_inner(message).unwrap();
        assert_eq!(u16::from_le_bytes([packet[0], packet[1]]), 34);
        assert_eq!(
            u16::from_le_bytes([packet[2], packet[3]]) as usize,
            index + 1
        );
        assert_eq!(packet.len() - 4, expected_len, "subtype {}", index + 1);
        assert_eq!(decode_inner(&packet).unwrap(), *message);
    }
    let unknown = DerbyMessage::Unknown {
        subtype: 99,
        body: vec![9, 8, 7],
    };
    assert_eq!(
        decode_inner(&encode_inner(&unknown).unwrap()).unwrap(),
        unknown
    );
}

#[test]
fn solo_offer_registers_npcs_runs_client_gates_through_state_eleven_and_closes_once() {
    let mut derby = service(&[OWNER], Vec::new());
    run_to_live(&mut derby, &[OWNER]);
    assert_eq!(derby.human_count(), 1);
    assert_eq!(derby.npc_count(), 7);

    let final_rows = derby.finish_match(6_000).unwrap();
    let result_messages = messages(&final_rows);
    assert_eq!(result_messages.last(), Some(&DerbyMessage::GameState(10)));
    assert_eq!(
        result_messages
            .iter()
            .filter(|message| matches!(message, DerbyMessage::VehicleState(_)))
            .count(),
        8
    );
    assert_eq!(derby.phase(), DerbyPhase::Results);
    let ack10 = encode_inner(&DerbyMessage::PlayerPhase {
        key: OWNER,
        phase: 10,
    })
    .unwrap();
    assert!(
        derby
            .handle_client(OWNER, &ack10, 6_100)
            .unwrap()
            .is_empty()
    );
    assert!(derby.tick(6_999).unwrap().is_empty());
    let close_ready = derby.tick(7_000).unwrap();
    assert_eq!(messages(&close_ready), vec![DerbyMessage::GameState(11)]);
    assert_eq!(derby.phase(), DerbyPhase::CloseEnabled);

    let close = derby.handle_close(OWNER, &[26, 0, 42, 0]).unwrap();
    assert!(close);
    assert_eq!(derby.phase(), DerbyPhase::Closed);
    let duplicate = derby.handle_close(OWNER, &[26, 0, 42, 0]).unwrap();
    assert!(!duplicate);
}

#[test]
fn contacts_apply_server_damage_knockout_scores_and_four_second_respawn() {
    let mut derby = service(&[OWNER, OTHER], Vec::new());
    run_to_live(&mut derby, &[OWNER, OTHER]);
    derby
        .handle_client(OWNER, &pose(OWNER, [0.0, 0.0, 0.0]), 5_000)
        .unwrap();
    derby
        .handle_client(OTHER, &pose(OTHER, [1.0, 0.0, 0.0]), 5_000)
        .unwrap();
    let hit = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: OWNER,
        other: OTHER,
        impact_bits: 25.0_f32.to_bits(),
    }))
    .unwrap();
    let actions = derby.handle_client(OWNER, &hit, 6_000).unwrap();
    let sent = messages(&actions);
    assert!(sent.contains(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: OWNER,
        other: OTHER,
        impact_bits: 25.0_f32.to_bits(),
    })));
    assert!(sent.iter().any(|message| matches!(message, DerbyMessage::VehicleState(state) if state.key == OTHER && state.health == 0)));
    assert!(
        sent.iter()
            .any(|message| matches!(message, DerbyMessage::Announcer { kind: 2, .. }))
    );
    assert_eq!(derby.driver(OWNER).unwrap().score, 2);
    assert_eq!(derby.driver(OTHER).unwrap().health, 0);
    assert!(derby.handle_client(OWNER, &hit, 6_100).unwrap().is_empty());

    let respawn = derby.tick(10_100).unwrap();
    let sent = messages(&respawn);
    assert!(sent.contains(&DerbyMessage::ResetVehicle(OTHER)));
    assert_eq!(derby.driver(OTHER).unwrap().health, 20);
}

#[test]
fn each_powerup_type_applies_once_and_expiry_uses_client_categories() {
    let pickups = vec![
        pickup(1, PickupKind::Health, 0.0),
        pickup(2, PickupKind::Boost, 10.0),
        pickup(3, PickupKind::Shield, 20.0),
        pickup(4, PickupKind::AttackUp, 30.0),
        pickup(5, PickupKind::Ultimate, 40.0),
    ];
    let mut derby = service(&[OWNER, OTHER], pickups);
    run_to_live(&mut derby, &[OWNER, OTHER]);
    derby
        .handle_client(OTHER, &pose(OTHER, [0.0, 0.0, 0.0]), 5_000)
        .unwrap();
    derby
        .handle_client(OWNER, &pose(OWNER, [1.0, 0.0, 0.0]), 5_000)
        .unwrap();

    let damage = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: OTHER,
        other: OWNER,
        impact_bits: 5.0_f32.to_bits(),
    }))
    .unwrap();
    derby.handle_client(OTHER, &damage, 6_000).unwrap();
    assert_eq!(derby.driver(OWNER).unwrap().health, 15);

    for id in 1..=5 {
        let x = (id as f32 - 1.0) * 10.0;
        let request = encode_inner(&DerbyMessage::PickupHitRequest {
            key: OWNER,
            position_bits: [x.to_bits(), 0, 0, 0],
        })
        .unwrap();
        let actions = derby
            .handle_client(OWNER, &request, 7_000 + id as u64)
            .unwrap();
        let sent = messages(&actions);
        assert!(
            sent.iter()
                .any(|message| *message == DerbyMessage::DestroyPickup(id))
        );
        assert!(sent.iter().any(|message| matches!(message, DerbyMessage::PickupHitResponse { kind, .. } if *kind as u32 == id)));
        assert!(
            derby
                .handle_client(OWNER, &request, 8_000 + id as u64)
                .unwrap()
                .is_empty()
        );
    }
    assert_eq!(derby.driver(OWNER).unwrap().health, 20);
    assert!(derby.driver(OWNER).unwrap().boost > 0.0);
    assert!(derby.driver(OWNER).unwrap().shield_until_ms.is_some());
    assert!(derby.driver(OWNER).unwrap().attack_until_ms.is_some());
    assert!(derby.driver(OWNER).unwrap().ultimate_until_ms.is_some());
    let expired = derby.tick(13_000).unwrap();
    let expired_messages = messages(&expired);
    assert!(expired_messages.contains(&DerbyMessage::ItemExpired(3)));
    assert!(expired_messages.contains(&DerbyMessage::ItemExpired(4)));
    assert!(expired_messages.contains(&DerbyMessage::ItemExpired(5)));
}

#[test]
fn contacts_require_fresh_nearby_poses_and_a_bounded_impact() {
    let mut derby = service(&[OWNER, OTHER], Vec::new());
    run_to_live(&mut derby, &[OWNER, OTHER]);
    let hit = |impact: f32| {
        encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
            reporter: OWNER,
            other: OTHER,
            impact_bits: impact.to_bits(),
        }))
        .unwrap()
    };

    // Both live poses are required, not just the target from a creation record.
    assert!(
        derby
            .handle_client(OWNER, &hit(5.0), 5_000)
            .unwrap()
            .is_empty()
    );
    assert_eq!(derby.driver(OTHER).unwrap().health, 20);
    derby
        .handle_client(OWNER, &pose(OWNER, [0.0, 0.0, 0.0]), 5_000)
        .unwrap();
    assert!(
        derby
            .handle_client(OWNER, &hit(5.0), 5_001)
            .unwrap()
            .is_empty()
    );

    // A real pair of poses must still describe a plausible contact.
    derby
        .handle_client(OTHER, &pose(OTHER, [50.0, 0.0, 0.0]), 5_000)
        .unwrap();
    assert!(
        derby
            .handle_client(OWNER, &hit(5.0), 5_001)
            .unwrap()
            .is_empty()
    );
    derby
        .handle_client(OTHER, &pose(OTHER, [1.0, 0.0, 0.0]), 5_001)
        .unwrap();
    assert!(
        derby
            .handle_client(OWNER, &hit(50.1), 5_002)
            .unwrap()
            .is_empty()
    );
    assert!(
        derby
            .handle_client(OWNER, &hit(5.0), 12_000)
            .unwrap()
            .is_empty()
    );
    derby
        .handle_client(OWNER, &pose(OWNER, [0.0, 0.0, 0.0]), 12_001)
        .unwrap();
    derby
        .handle_client(OTHER, &pose(OTHER, [1.0, 0.0, 0.0]), 12_001)
        .unwrap();

    let accepted = derby.handle_client(OWNER, &hit(5.0), 12_002).unwrap();
    assert!(
        messages(&accepted).iter().any(
            |message| matches!(message, DerbyMessage::VehicleState(state) if state.key == OTHER && state.health == 15)
        )
    );
}

#[test]
fn refused_contact_does_not_consume_the_pair_cooldown() {
    let mut bounded = rules();
    bounded.max_damage_per_contact = 5;
    let mut derby = service_with_rules(&[OWNER, OTHER], Vec::new(), bounded);
    run_to_live(&mut derby, &[OWNER, OTHER]);
    derby
        .handle_client(OWNER, &pose(OWNER, [0.0, 0.0, 0.0]), 5_000)
        .unwrap();
    derby
        .handle_client(OTHER, &pose(OTHER, [1.0, 0.0, 0.0]), 5_000)
        .unwrap();
    let hit = |impact: f32| {
        encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
            reporter: OWNER,
            other: OTHER,
            impact_bits: impact.to_bits(),
        }))
        .unwrap()
    };

    assert!(
        derby
            .handle_client(OWNER, &hit(10.0), 5_100)
            .unwrap()
            .is_empty()
    );
    assert_eq!(derby.driver(OTHER).unwrap().health, 20);
    let accepted = derby.handle_client(OWNER, &hit(5.0), 5_200).unwrap();
    assert_eq!(derby.driver(OTHER).unwrap().health, 15);
    assert!(
        messages(&accepted).iter().any(
            |message| matches!(message, DerbyMessage::VehicleState(state) if state.key == OTHER)
        )
    );
}

#[test]
fn pose_is_relayed_only_to_other_humans_and_disconnect_aborts_empty_match() {
    let mut derby = service(&[OWNER, OTHER], Vec::new());
    run_to_live(&mut derby, &[OWNER, OTHER]);
    let pose = DerbyMessage::VehiclePose(VehiclePose {
        key: OWNER,
        transform_bits: [1; 16],
        vector_a_bits: [2; 4],
        vector_b_bits: [3; 4],
        controls_bits: [4; 3],
        flags: [0, 1, 0, 0],
        sequence: 10,
    });
    let packet = encode_inner(&pose).unwrap();
    let relay = derby.handle_client(OWNER, &packet, 6_000).unwrap();
    assert_eq!(relay.len(), 1);
    assert_eq!(relay[0].recipients, vec![OTHER]);
    assert_eq!(decode_inner(&relay[0].packet).unwrap(), pose);

    let removed = derby.disconnect(OWNER).unwrap();
    assert!(messages(&removed).contains(&DerbyMessage::DestroyVehicle(OWNER)));
    assert_eq!(derby.phase(), DerbyPhase::Running);
    derby.disconnect(OTHER).unwrap();
    assert_eq!(derby.phase(), DerbyPhase::Aborted);
    assert!(derby.finish_match(9_000).unwrap().is_empty());
}
