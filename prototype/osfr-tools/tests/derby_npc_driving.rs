//! L931: server-driven derby NPCs (`NpcDriving`). Without it the NPC drivers are inert
//! placeholders at an unreachable spot, so a match's placement and knock-outs never differ; with it
//! each NPC appears on a ring around the first human pose, chases the nearest living car and bumps
//! it, and the human's placement and knock-out facts follow. All numbers are (R) tunables.
//!
//! These tests drive `DerbyService` directly with explicit clocks; the route tests that enter the
//! shipped rows through a real gateway live in `derby_rows.rs`.

use std::collections::BTreeSet;

use osfr_tools::derby::{
    DerbyAction, DerbyMessage, DerbyPhase, DerbyRules, DerbyService, DerbyVehicleTemplate,
    NpcDriving, ParticipantKey, VehicleHitReport, VehiclePose, decode_inner, encode_inner,
};

const HUMAN: ParticipantKey = ParticipantKey(0x1000_0000_0000_0001);
const INSTANCE: i32 = 931;
const ROW: i32 = 607;
const START: [f32; 3] = [100.0, 6.0, 200.0];

fn driving() -> NpcDriving {
    NpcDriving {
        pose_interval_ms: 100,
        speed_mps: 12.0,
        spawn_radius_m: 12.0,
        bump_impact: 5.0,
        bump_interval_ms: 1_500,
    }
}

fn service(rules: DerbyRules, driving: Option<NpcDriving>) -> DerbyService {
    let template = DerbyVehicleTemplate {
        name: Vec::new(),
        definition: vec![1, 2, 3],
        start_transform_bits: [0; 16],
        unknown_flag: false,
        unknown_value_bits: 0,
    };
    let mut service = DerbyService::new(INSTANCE, ROW, HUMAN, template, rules, Vec::new()).unwrap();
    service.set_npc_driving(driving).unwrap();
    service
}

fn pose_packet(key: ParticipantKey, position: [f32; 3]) -> Vec<u8> {
    let transform = [
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
    ];
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: transform.map(f32::to_bits),
        vector_a_bits: [0; 4],
        vector_b_bits: [0; 4],
        controls_bits: [0; 3],
        flags: [0; 4],
        sequence: 1,
    }))
    .unwrap()
}

/// Offer, Go, register, the client-owned phase gates and the countdown: Running from `4_100` ms.
fn run_to_live(service: &mut DerbyService) {
    service.offer_packet().unwrap();
    let mut go = vec![39, 0, 5];
    go.extend(INSTANCE.to_le_bytes());
    go.extend((-1_i32).to_le_bytes());
    go.extend((-1_i32).to_le_bytes());
    service.handle_minigame_packet(&go).unwrap();
    let register = encode_inner(&DerbyMessage::RegisterPlayer {
        key: HUMAN,
        unknown_key: 0,
    })
    .unwrap();
    service.handle_client(HUMAN, &register, 0).unwrap();
    for phase in 3..=8 {
        let packet = encode_inner(&DerbyMessage::PlayerPhase { key: HUMAN, phase }).unwrap();
        service.handle_client(HUMAN, &packet, phase as u64).unwrap();
    }
    for count in (0..=4).rev() {
        let packet = encode_inner(&DerbyMessage::CountdownStatus { key: HUMAN, count }).unwrap();
        service
            .handle_client(HUMAN, &packet, 100 + (4 - count) as u64 * 1_000)
            .unwrap();
    }
    assert_eq!(service.phase(), DerbyPhase::Running);
}

fn messages(actions: &[DerbyAction]) -> Vec<DerbyMessage> {
    actions
        .iter()
        .map(|action| decode_inner(&action.packet).unwrap())
        .collect()
}

/// The NPC poses in `actions`, as `(key, position)`.
fn npc_poses(actions: &[DerbyAction]) -> Vec<(ParticipantKey, [f32; 3])> {
    messages(actions)
        .into_iter()
        .filter_map(|message| match message {
            DerbyMessage::VehiclePose(pose) if pose.key != HUMAN => Some((
                pose.key,
                [12, 13, 14].map(|slot| f32::from_bits(pose.transform_bits[slot])),
            )),
            _ => None,
        })
        .collect()
}

fn gap(left: [f32; 3], right: [f32; 3]) -> f32 {
    (left[0] - right[0]).hypot(left[2] - right[2])
}

fn npc_keys(service: &DerbyService) -> Vec<ParticipantKey> {
    (1..=7)
        .map(|slot| ParticipantKey(0xd000_0000_0000_0000 | ((INSTANCE as u64) << 8) | slot))
        .inspect(|key| assert!(service.driver(*key).is_some_and(|npc| !npc.human)))
        .collect()
}

/// Tick every 100 ms from `from_ms` until `done`, at most `ticks` times; returns the clock.
fn tick_until(
    service: &mut DerbyService,
    from_ms: u64,
    ticks: u64,
    mut done: impl FnMut(&DerbyService) -> bool,
) -> u64 {
    let mut now = from_ms;
    for _ in 0..ticks {
        if done(service) {
            break;
        }
        service.tick(now).unwrap();
        now += 100;
    }
    now
}

#[test]
fn inert_npcs_never_move_without_npc_driving() {
    let mut service = service(DerbyRules::default(), None);
    run_to_live(&mut service);
    service
        .handle_client(HUMAN, &pose_packet(HUMAN, START), 4_200)
        .unwrap();
    for step in 0..50 {
        let actions = service.tick(4_300 + step * 100).unwrap();
        assert!(npc_poses(&actions).is_empty(), "step {step}");
    }
}

#[test]
fn driven_npcs_wait_for_a_human_pose_then_ring_it() {
    let mut service = service(DerbyRules::default(), Some(driving()));
    run_to_live(&mut service);
    let before = service.tick(4_200).unwrap();
    assert!(
        npc_poses(&before).is_empty(),
        "no human pose, nobody placed"
    );

    service
        .handle_client(HUMAN, &pose_packet(HUMAN, START), 4_250)
        .unwrap();
    let placed = npc_poses(&service.tick(4_400).unwrap());
    assert_eq!(placed.len(), 7, "all seven NPCs appear at once");
    for (key, position) in &placed {
        assert!(npc_keys(&service).contains(key));
        assert!((gap(*position, START) - 12.0).abs() < 0.01, "{position:?}");
        assert_eq!(position[1], START[1], "the ring keeps the human's height");
    }
    let keys: BTreeSet<_> = placed.iter().map(|(key, _)| *key).collect();
    assert_eq!(keys.len(), 7);
}

#[test]
fn driven_npcs_close_in_bump_and_hurt_the_human() {
    let mut service = service(DerbyRules::default(), Some(driving()));
    run_to_live(&mut service);
    service
        .handle_client(HUMAN, &pose_packet(HUMAN, START), 4_200)
        .unwrap();
    let mut bumpers = BTreeSet::new();
    let mut now = 4_300;
    for _ in 0..1_200 {
        for message in messages(&service.tick(now).unwrap()) {
            if let DerbyMessage::VehicleHit(report) = message {
                bumpers.insert(report.reporter);
                assert_eq!(f32::from_bits(report.impact_bits), 5.0);
            }
        }
        now += 100;
        if service.driver(HUMAN).unwrap().damage_received > 0 {
            break;
        }
    }
    let human = service.driver(HUMAN).unwrap();
    assert!(
        human.damage_received > 0,
        "an NPC reaches the human within two minutes"
    );
    assert_eq!(
        human.damage_received % 5,
        0,
        "every bump is impact 5 at 1 damage per unit"
    );
    assert!(bumpers.iter().all(|key| npc_keys(&service).contains(key)));
    let dealt: u32 = npc_keys(&service)
        .into_iter()
        .map(|key| service.driver(key).unwrap().damage_dealt)
        .sum();
    assert!(
        dealt >= human.damage_received,
        "the damage is credited to the NPCs"
    );
}

#[test]
fn a_human_hit_lands_on_a_driven_npc_where_it_stands() {
    let mut service = service(DerbyRules::default(), Some(driving()));
    run_to_live(&mut service);
    service
        .handle_client(HUMAN, &pose_packet(HUMAN, START), 4_200)
        .unwrap();
    let placed = npc_poses(&service.tick(4_300).unwrap());
    let (npc, position) = placed[0];

    // The human drives to where the server says the NPC stands and reports the contact.
    service
        .handle_client(HUMAN, &pose_packet(HUMAN, position), 4_310)
        .unwrap();
    let report = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: HUMAN,
        other: npc,
        impact_bits: 100.0_f32.to_bits(),
    }))
    .unwrap();
    service.handle_client(HUMAN, &report, 4_320).unwrap();
    assert_eq!(
        service.driver(npc).unwrap().health,
        0,
        "a 100 impact knocks out a 100 health NPC"
    );
    assert_eq!(service.driver(HUMAN).unwrap().score, 2);
}

#[test]
fn a_bumped_out_human_places_behind_and_reports_the_knock_out() {
    let rules = DerbyRules {
        start_health: 10,
        time_limit_ms: Some(120_000),
        ..DerbyRules::default()
    };
    let mut service = service(rules, Some(driving()));
    run_to_live(&mut service);
    service
        .handle_client(HUMAN, &pose_packet(HUMAN, START), 4_200)
        .unwrap();
    let now = tick_until(&mut service, 4_300, 1_100, |service| {
        service.driver(HUMAN).unwrap().knockouts_taken > 0
    });
    assert!(
        service.driver(HUMAN).unwrap().knockouts_taken > 0,
        "NPC bumps knock a 10 health human out"
    );

    // The clock ends the match; the human never scored, a bumper did.
    service.tick(4_100 + 120_000).unwrap();
    assert!(now < 4_100 + 120_000);
    assert_eq!(service.phase(), DerbyPhase::Results);
    let result = service.participant_result(HUMAN).unwrap();
    assert!(result.knocked_out);
    assert_eq!(result.score, 0);
    assert!(result.placement > 1, "an NPC outscored the human");
    assert!(
        !result
            .quest_result()
            .meets(&osfr_tools::content::MinigameRequirement {
                no_knockout: true,
                ..Default::default()
            })
    );
}

#[test]
fn npcs_fight_each_other_so_an_idle_human_cannot_stay_first() {
    let rules = DerbyRules {
        start_health: 20,
        time_limit_ms: Some(120_000),
        ..DerbyRules::default()
    };
    let mut service = service(rules, Some(driving()));
    run_to_live(&mut service);
    service
        .handle_client(HUMAN, &pose_packet(HUMAN, START), 4_200)
        .unwrap();
    // Idle through the whole clock; the NPCs score on each other.
    let mut now = 4_300;
    while service.phase() == DerbyPhase::Running && now < 130_000 {
        service.tick(now).unwrap();
        now += 100;
    }
    assert_eq!(service.phase(), DerbyPhase::Results);
    let scored = npc_keys(&service)
        .into_iter()
        .any(|key| service.driver(key).unwrap().score > 0);
    assert!(scored, "no NPC scored in two minutes");
    let result = service.participant_result(HUMAN).unwrap();
    assert!(result.placement > 1, "an idle human is not first");
}

#[test]
fn an_npc_reaching_the_score_target_ends_the_match() {
    let rules = DerbyRules {
        start_health: 5,
        score_target: Some(4),
        ..DerbyRules::default()
    };
    let mut service = service(rules, Some(driving()));
    run_to_live(&mut service);
    service
        .handle_client(HUMAN, &pose_packet(HUMAN, START), 4_200)
        .unwrap();
    tick_until(&mut service, 4_300, 3_000, |service| {
        service.phase() != DerbyPhase::Running
    });
    assert_eq!(service.phase(), DerbyPhase::Results);
}

#[test]
fn out_of_bounds_driving_numbers_are_refused() {
    let bad = |edit: fn(&mut NpcDriving)| {
        let mut config = driving();
        edit(&mut config);
        config
    };
    let template = DerbyVehicleTemplate {
        name: Vec::new(),
        definition: Vec::new(),
        start_transform_bits: [0; 16],
        unknown_flag: false,
        unknown_value_bits: 0,
    };
    let mut service = DerbyService::new(
        INSTANCE,
        ROW,
        HUMAN,
        template,
        DerbyRules::default(),
        Vec::new(),
    )
    .unwrap();
    for config in [
        bad(|config| config.pose_interval_ms = 0),
        bad(|config| config.pose_interval_ms = 60_000),
        bad(|config| config.speed_mps = f32::NAN),
        bad(|config| config.speed_mps = 1_000.0),
        bad(|config| config.spawn_radius_m = -1.0),
        bad(|config| config.bump_impact = 0.0),
        bad(|config| config.bump_impact = 1_000.0),
        bad(|config| config.bump_interval_ms = 10),
    ] {
        assert!(service.set_npc_driving(Some(config)).is_err(), "{config:?}");
    }
    assert!(service.set_npc_driving(Some(driving())).is_ok());
    assert!(service.set_npc_driving(None).is_ok());
}
