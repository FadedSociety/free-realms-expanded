//! Ordered wire pins for the overworld field AI before and after the S14
//! locomotion-helper refactor. Every case drives the production `tick` path
//! (solo field enemies plus battle-marker wanderers) with a fixed clock and a
//! deterministic ground grid, then asserts the complete reply sequence.
//!
//! Wire evidence: movement is the standalone opcode 125 read at `0x008e5940`
//! (u16 opcode 125, u64 actor, 3 f32 position, 3 f32 direction, state,
//! unknown); there is no subtype after it, and its direction is a horizontal
//! vector the client normalizes before `atan2(X, Z)` (`0x0095dae0`).
//! Animation is family 35/subtype 8 read at
//! `0x00908a50` -> `0x008e5dd0` (u64 actor, i32 animation, i32 unknown, flags
//! byte; bit 0 assigns the base animation). Ids 1/2/3 are loc_stand/loc_walk/
//! loc_run and 1001 is the melee attack clip.
use osfr_tools::{
    encounter::{EncounterEngine, EncounterOutcome},
    terrain::{HeightGrid, Terrain},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

const FIELD_GUID: u64 = 100;
const SPAWNER_GUID: u64 = 900;
const BATTLE_ID: u32 = 96;
const MARKER_HOME: [f32; 3] = [-200.0, 0.0, -200.0];

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    dir: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn actor(guid: u64, position: [f32; 3], max_health: u32, damage: u32) -> Value {
    json!({
        "guid": guid,
        "definition_id": 400,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic actor",
        "texture_alias": "",
        "position": position,
        "max_health": max_health,
        "damage": damage,
        "boss": false
    })
}

/// A fast, flat melee actor: 10 m/s keeps every golden position an exact f32
/// multiple of the tick step, and the 12 m leash matches the shipped preset.
fn behaviour() -> Value {
    json!({
        "move_speed": 10.0,
        "run_speed": 10.0,
        "attack_ms": 1000,
        "reach": 2.5,
        "leash": 12.0
    })
}

impl Fixture {
    fn new(home: [f32; 3], behaviour: Value) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "w359-field-ai-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = json!({
            "zone_id": 0,
            "name": "SyntheticWorld",
            "sky": "",
            "geometry_id": 214,
            "position": [0, 0, 0],
            "rotation": [0, 0, 0],
            "tutorial": false
        });
        let mut cave = zone.clone();
        cave["zone_id"] = json!(411);
        fs::write(
            dir.join("encounters.json"),
            json!({
                "schema_version": 1,
                "template_file": "npc.bin",
                "template_sha256": format!("{:x}", Sha256::digest(&template)),
                "world": zone,
                "world_enemies": [],
                "dungeons": [{
                    "id": 411,
                    "title_id": 12,
                    "description_id": 13,
                    "objective_id": 14,
                    "zone": cave,
                    "guide": null,
                    "enemies": [actor(200, [0.0, 0.0, 0.0], 100, 1)],
                    "reward_coins": 0,
                    "reward_items": []
                }]
            })
            .to_string(),
        )
        .unwrap();
        let mut enemy = actor(FIELD_GUID, home, 1000, 7);
        enemy["level"] = json!(1);
        enemy["zone_id"] = json!(0);
        enemy["respawn_seconds"] = json!(30);
        enemy["aggro_range"] = json!(8.0);
        enemy["leash_radius"] = json!(12.0);
        enemy["behaviour"] = behaviour;
        fs::write(
            dir.join("field_enemies.json"),
            json!({"schema_version": 1, "enemies": [enemy]}).to_string(),
        )
        .unwrap();
        let mut marker = actor(SPAWNER_GUID, MARKER_HOME, 1, 0);
        marker["definition_id"] = json!(930096);
        marker["model_id"] = json!(168);
        marker["name_id"] = json!(15623);
        marker["name"] = json!("Synthetic marker");
        fs::write(
            dir.join("field_battles.json"),
            json!({
                "schema_version": 1,
                "battles": [{
                    "id": BATTLE_ID,
                    "title_id": 42056,
                    "description_id": 42059,
                    "objective_id": 42071,
                    "difficulty": 1,
                    "spawner": marker,
                    "zone": cave,
                    "enemies": [actor(901, [0.0, 0.0, 0.0], 100, 1)]
                }]
            })
            .to_string(),
        )
        .unwrap();
        Self { dir }
    }

    /// Load enemies then battles, the production order, and optionally bind a
    /// terrain grid for the ground-refusal case.
    fn load(&self, terrain: Option<HeightGrid>) -> EncounterEngine {
        if let Some(grid) = terrain {
            fs::write(self.dir.join("zone_0.hgt"), grid.to_bytes()).unwrap();
        }
        let mut engine = EncounterEngine::load(&self.dir, None).unwrap();
        engine
            .load_field_enemies(&self.dir.join("field_enemies.json"))
            .unwrap();
        engine
            .load_field_battles(&self.dir.join("field_battles.json"))
            .unwrap();
        if self.dir.join("zone_0.hgt").is_file() {
            engine.set_terrain(Some(Arc::new(Terrain::load(&self.dir).unwrap())));
        }
        engine
    }
}

/// One decoded reply frame: exactly the fields the pins compare.
#[derive(Debug, PartialEq)]
enum Frame {
    Move {
        guid: u64,
        position: [f32; 3],
        heading: [f32; 3],
    },
    Anim {
        guid: u64,
        id: i32,
        base: bool,
    },
    EnemyHp {
        guid: u64,
        hp: u32,
        max: u32,
    },
    PlayerHp {
        hp: u32,
        max: u32,
    },
    Hit {
        attacker: u64,
        target: u64,
        damage: u32,
        hp: u32,
        max: u32,
    },
    Rules {
        combat: bool,
    },
    Flag {
        sub: u16,
        on: bool,
    },
}

fn f32_at(packet: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

fn i32_at(packet: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

fn u32_at(packet: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

fn u64_at(packet: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(packet[offset..offset + 8].try_into().unwrap())
}

fn frames(out: &EncounterOutcome) -> Vec<Frame> {
    out.replies
        .iter()
        .map(|packet| {
            let family = u16::from_le_bytes([packet[0], packet[1]]);
            let sub = u16::from_le_bytes([packet[2], packet[3]]);
            match family {
                // Opcode 125 is standalone: packet[2..10] is the u64 actor, so
                // `sub` above is the GUID's low half, not a subtype.
                125 => {
                    assert_eq!(packet.len(), 36, "movement body");
                    Frame::Move {
                        guid: u64_at(packet, 2),
                        position: [f32_at(packet, 10), f32_at(packet, 14), f32_at(packet, 18)],
                        heading: [f32_at(packet, 22), f32_at(packet, 26), f32_at(packet, 30)],
                    }
                }
                35 => match sub {
                    8 => {
                        assert_eq!(packet.len(), 21, "animation body");
                        Frame::Anim {
                            guid: u64_at(packet, 4),
                            id: i32_at(packet, 12),
                            base: packet[20] == 1,
                        }
                    }
                    5 => Frame::EnemyHp {
                        guid: u64_at(packet, 4),
                        hp: u32_at(packet, 12),
                        max: u32_at(packet, 16),
                    },
                    _ => panic!(
                        "unexpected field-AI reply {family}/{sub} ({} bytes)",
                        packet.len()
                    ),
                },
                38 if sub == 1 => Frame::PlayerHp {
                    hp: u32_at(packet, 4),
                    max: u32_at(packet, 8),
                },
                32 if sub == 7 => Frame::Hit {
                    attacker: u64_at(packet, 4),
                    target: u64_at(packet, 20),
                    damage: u32_at(packet, 28),
                    max: u32_at(packet, 32),
                    hp: u32_at(packet, 46),
                },
                62 => Frame::Rules {
                    combat: packet[2] == 1,
                },
                41 if sub == 132 || sub == 133 => Frame::Flag {
                    sub,
                    on: packet[12] == 1,
                },
                _ => panic!(
                    "unexpected field-AI reply {family}/{sub} ({} bytes)",
                    packet.len()
                ),
            }
        })
        .collect()
}

fn tick(engine: &mut EncounterEngine, at: [f32; 3], now_ms: u64) -> Vec<Frame> {
    frames(&engine.tick(at, 1, now_ms).unwrap())
}

fn enemy_at(engine: &EncounterEngine) -> [f32; 3] {
    engine
        .field_teleport_targets()
        .into_iter()
        .find(|target| target.guid == FIELD_GUID)
        .expect("the field actor is alive and placed")
        .position
}

fn combat_on() -> [Frame; 3] {
    [
        Frame::Rules { combat: true },
        Frame::Flag { sub: 132, on: true },
        Frame::Flag { sub: 133, on: true },
    ]
}

fn combat_off() -> [Frame; 3] {
    [
        Frame::Rules { combat: false },
        Frame::Flag {
            sub: 132,
            on: false,
        },
        Frame::Flag {
            sub: 133,
            on: false,
        },
    ]
}

#[test]
fn chase_start_is_movement_then_run_animation() {
    let fixture = Fixture::new([0.0, 0.0, 0.0], behaviour());
    let mut engine = fixture.load(None);
    let out = engine.tick([7.0, 0.0, 0.0], 1, 100).unwrap();
    let mut expected = vec![
        Frame::Move {
            guid: FIELD_GUID,
            position: [1.0, 0.0, 0.0],
            heading: [1.0, 0.0, 0.0],
        },
        Frame::Anim {
            guid: FIELD_GUID,
            id: 3,
            base: true,
        },
    ];
    expected.extend(combat_on());
    assert_eq!(frames(&out), expected);
    assert!(out.changed);

    // The walk animation starts once; a later step is movement only.
    let out = engine.tick([7.0, 0.0, 0.0], 1, 350).unwrap();
    assert_eq!(
        frames(&out),
        vec![Frame::Move {
            guid: FIELD_GUID,
            position: [3.5, 0.0, 0.0],
            heading: [1.0, 0.0, 0.0],
        }]
    );
    assert!(out.changed);
}

#[test]
fn non_axis_chase_heading_is_not_a_unit_axis() {
    let fixture = Fixture::new([0.0, 0.0, 0.0], behaviour());
    let mut engine = fixture.load(None);
    let out = engine.tick([3.0, 0.0, 4.0], 1, 100).unwrap();
    let mut expected = vec![
        Frame::Move {
            guid: FIELD_GUID,
            position: [0.6, 0.0, 0.8],
            heading: [0.6, 0.0, 0.8],
        },
        Frame::Anim {
            guid: FIELD_GUID,
            id: 3,
            base: true,
        },
    ];
    expected.extend(combat_on());
    assert_eq!(frames(&out), expected);
    // The client-normalized direction is the exact 3-4-5 diagonal, not an axis.
    let movement = &out.replies[0];
    assert_eq!(
        &movement[22..34],
        &[0.6_f32, 0.0, 0.8]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<u8>>()
    );
    assert_eq!(movement[34..], [1, 0]);
}

#[test]
fn chase_stops_in_reach_then_attacks() {
    let fixture = Fixture::new([0.0, 0.0, 0.0], behaviour());
    let mut engine = fixture.load(None);
    tick(&mut engine, [7.0, 0.0, 0.0], 100);
    tick(&mut engine, [7.0, 0.0, 0.0], 350);
    // The stop step lands inside reach, so the same tick also swings.
    let out = engine.tick([7.0, 0.0, 0.0], 1, 600).unwrap();
    assert_eq!(
        frames(&out),
        vec![
            Frame::Move {
                guid: FIELD_GUID,
                position: [4.6, 0.0, 0.0],
                heading: [1.0, 0.0, 0.0],
            },
            Frame::Anim {
                guid: FIELD_GUID,
                id: 1,
                base: true,
            },
            Frame::Anim {
                guid: FIELD_GUID,
                id: 1001,
                base: false,
            },
            Frame::Hit {
                attacker: FIELD_GUID,
                target: 1,
                damage: 7,
                hp: 993,
                max: 1000,
            },
            Frame::PlayerHp { hp: 993, max: 1000 },
        ]
    );
    assert!(out.changed);
    assert_eq!(enemy_at(&engine), [4.6, 0.0, 0.0]);
}

#[test]
fn evade_returns_home_heals_and_stands() {
    let fixture = Fixture::new([0.0, 0.0, 0.0], behaviour());
    let mut engine = fixture.load(None);
    for now in [100, 350, 600, 850] {
        tick(&mut engine, [7.0, 0.0, 0.0], now);
    }
    // Damage the actor so the return home has a health correction to send.
    let hit = engine
        .attack(FIELD_GUID, enemy_at(&engine), 1, 0, 900)
        .unwrap();
    assert!(
        hit.replies
            .iter()
            .any(|packet| packet.starts_with(&[32, 0, 7, 0]))
    );
    assert_eq!(
        engine
            .field_teleport_targets()
            .into_iter()
            .find(|target| target.guid == FIELD_GUID)
            .unwrap()
            .position,
        [4.6, 0.0, 0.0]
    );

    let out = engine.tick([100.0, 0.0, 0.0], 1, 1150).unwrap();
    let mut expected = vec![
        Frame::Move {
            guid: FIELD_GUID,
            position: [2.1, 0.0, 0.0],
            heading: [-1.0, 0.0, 0.0],
        },
        Frame::Anim {
            guid: FIELD_GUID,
            id: 3,
            base: true,
        },
    ];
    expected.extend(combat_off());
    assert_eq!(frames(&out), expected);
    assert!(out.changed);

    // The last step lands exactly home and emits the stop frame before the heal.
    let out = engine.tick([100.0, 0.0, 0.0], 1, 1400).unwrap();
    assert_eq!(
        frames(&out),
        vec![
            Frame::Move {
                guid: FIELD_GUID,
                position: [0.0, 0.0, 0.0],
                heading: [-1.0, 0.0, 0.0],
            },
            Frame::Anim {
                guid: FIELD_GUID,
                id: 1,
                base: true,
            },
            Frame::EnemyHp {
                guid: FIELD_GUID,
                hp: 1000,
                max: 1000,
            },
        ]
    );
    assert!(out.changed);
    assert_eq!(enemy_at(&engine), [0.0, 0.0, 0.0]);
}

#[test]
fn leash_refuses_the_step_before_it_crosses() {
    let fixture = Fixture::new([0.0, 0.0, 0.0], behaviour());
    let mut engine = fixture.load(None);
    // Walk the actor to x = 11 one 250 ms step at a time, always keeping the
    // player six metres ahead so it keeps chasing rather than stopping in reach.
    for (now, player) in [
        (100, [7.0, 0.0, 0.0]),
        (350, [10.5, 0.0, 0.0]),
        (600, [13.0, 0.0, 0.0]),
        (850, [15.5, 0.0, 0.0]),
        (1100, [18.0, 0.0, 0.0]),
    ] {
        tick(&mut engine, player, now);
    }
    assert_eq!(enemy_at(&engine), [11.0, 0.0, 0.0]);

    // 11 -> 13.5 would cross the 12 m home leash: refuse the step, turn for home,
    // and emit no movement or animation for it.
    let out = engine.tick([20.5, 0.0, 0.0], 1, 1350).unwrap();
    let refused = frames(&out);
    assert!(
        !refused
            .iter()
            .any(|frame| matches!(frame, Frame::Move { .. } | Frame::Anim { .. })),
        "a refused step emits no movement or animation: {refused:?}"
    );
    assert_eq!(refused, Vec::from(combat_off()));
    assert_eq!(enemy_at(&engine), [11.0, 0.0, 0.0]);
    assert!(!out.changed);

    // It turns around at run speed; the walk was never stopped, so no new
    // start animation is sent for this step.
    let out = engine.tick([20.5, 0.0, 0.0], 1, 1600).unwrap();
    assert_eq!(
        frames(&out),
        vec![Frame::Move {
            guid: FIELD_GUID,
            position: [8.5, 0.0, 0.0],
            heading: [-1.0, 0.0, 0.0],
        }]
    );

    // Home again: the stop animation is the only locomotion frame left.
    for now in [1850, 2100, 2350] {
        tick(&mut engine, [20.5, 0.0, 0.0], now);
    }
    let out = engine.tick([20.5, 0.0, 0.0], 1, 2600).unwrap();
    assert_eq!(
        frames(&out),
        vec![
            Frame::Move {
                guid: FIELD_GUID,
                position: [0.0, 0.0, 0.0],
                heading: [-1.0, 0.0, 0.0],
            },
            Frame::Anim {
                guid: FIELD_GUID,
                id: 1,
                base: true,
            },
        ]
    );
    assert_eq!(enemy_at(&engine), [0.0, 0.0, 0.0]);
}

/// Flat ground everywhere except a 4 x 6 m hole starting at x = 12, just past
/// a 12 m home leash. A step that would cross both the leash and the hole must
/// be judged by the leash first, as before the helpers.
fn hole() -> HeightGrid {
    let mut grid = HeightGrid::new([-20.0, -20.0], [20.0, 20.0], 1.0).unwrap();
    let mut quad = |x0: f32, x1: f32, z0: f32, z1: f32| {
        grid.add_triangle([[x0, 0.0, z0], [x1, 0.0, z0], [x1, 0.0, z1]]);
        grid.add_triangle([[x0, 0.0, z0], [x1, 0.0, z1], [x0, 0.0, z1]]);
    };
    quad(-20.0, 12.0, -20.0, 20.0);
    quad(16.0, 20.0, -20.0, 20.0);
    quad(12.0, 16.0, -20.0, -3.0);
    quad(12.0, 16.0, 3.0, 20.0);
    grid
}

#[test]
fn leash_precedes_a_ground_refusal() {
    let fixture = Fixture::new([0.0, 0.0, 0.0], behaviour());
    let mut engine = fixture.load(Some(hole()));
    for (now, player) in [
        (100, [7.0, 0.0, 0.0]),
        (350, [10.5, 0.0, 0.0]),
        (600, [13.0, 0.0, 0.0]),
        (850, [15.5, 0.0, 0.0]),
        (1100, [18.0, 0.0, 0.0]),
    ] {
        tick(&mut engine, player, now);
    }
    assert_eq!(enemy_at(&engine), [11.0, 0.0, 0.0]);

    // 13.5 is both past the leash and off the grid. The leash decision wins,
    // so this is the silent turn, not the ground pause + stand animation.
    let out = engine.tick([20.5, 0.0, 0.0], 1, 1350).unwrap();
    let refused = frames(&out);
    assert!(
        !refused
            .iter()
            .any(|frame| matches!(frame, Frame::Move { .. } | Frame::Anim { .. })),
        "{refused:?}"
    );
    assert_eq!(refused, Vec::from(combat_off()));
    assert_eq!(enemy_at(&engine), [11.0, 0.0, 0.0]);
    assert!(!out.changed);
}

/// Flat ground on the left of x = 0, a 10 m wall on the right: a step whose
/// interpolated height would be more than MAX_SNAP (6 m) above the actor is
/// refused by `Ground::field_position`.
fn wall() -> HeightGrid {
    let mut grid = HeightGrid::new([-20.0, -20.0], [20.0, 20.0], 1.0).unwrap();
    grid.add_triangle([[-20.0, 0.0, -20.0], [0.0, 0.0, -20.0], [0.0, 0.0, 20.0]]);
    grid.add_triangle([[-20.0, 0.0, -20.0], [0.0, 0.0, 20.0], [-20.0, 0.0, 20.0]]);
    grid.add_triangle([[0.0, 10.0, -20.0], [20.0, 10.0, -20.0], [20.0, 10.0, 20.0]]);
    grid.add_triangle([[0.0, 10.0, -20.0], [20.0, 10.0, 20.0], [0.0, 10.0, 20.0]]);
    grid
}

#[test]
fn ground_refusal_stands_without_moving() {
    let fixture = Fixture::new([-4.0, 0.0, 0.0], behaviour());
    let mut engine = fixture.load(Some(wall()));
    let out = engine.tick([3.0, 0.0, 0.0], 1, 100).unwrap();
    let mut expected = vec![
        Frame::Move {
            guid: FIELD_GUID,
            position: [-3.0, 0.0, 0.0],
            heading: [1.0, 0.0, 0.0],
        },
        Frame::Anim {
            guid: FIELD_GUID,
            id: 3,
            base: true,
        },
    ];
    expected.extend(combat_on());
    assert_eq!(frames(&out), expected);

    // The snapped step climbs the wall to y = 5, which is still standable.
    let out = engine.tick([3.0, 0.0, 0.0], 1, 350).unwrap();
    assert_eq!(
        frames(&out),
        vec![Frame::Move {
            guid: FIELD_GUID,
            position: [-0.5, 5.0, 0.0],
            heading: [1.0, 0.0, 0.0],
        }]
    );

    // The next step reaches a 10 m height from y = 2.95: refused, so the old
    // position stays, the walk stops, and only the combat-off frames follow.
    let out = engine.tick([3.0, 0.0, 0.0], 1, 600).unwrap();
    let mut expected = vec![Frame::Anim {
        guid: FIELD_GUID,
        id: 1,
        base: true,
    }];
    expected.extend(combat_off());
    assert_eq!(frames(&out), expected);
    assert_eq!(enemy_at(&engine), [-0.5, 5.0, 0.0]);
    assert!(!out.changed);

    // The refusal returns it to the ground it came from; the walk was stopped,
    // so this step starts a fresh run animation.
    let out = engine.tick([3.0, 0.0, 0.0], 1, 850).unwrap();
    let returned = frames(&out);
    assert_eq!(returned.len(), 2, "{returned:?}");
    assert!(matches!(
        &returned[0],
        Frame::Move { guid, position, heading }
            if *guid == FIELD_GUID && position[0] < -0.5 && heading[0] < 0.0
    ));
    assert!(matches!(
        returned[1],
        Frame::Anim {
            guid: FIELD_GUID,
            id: 3,
            base: true
        }
    ));
}

#[test]
fn wanderer_walks_then_faces_a_nearby_player() {
    let fixture = Fixture::new([50.0, 0.0, 50.0], behaviour());
    let mut engine = fixture.load(None);
    // A player 50 m south of the marker is seen but not noticed: it walks.
    let out = engine.tick([-200.0, 0.0, -150.0], 1, 100).unwrap();
    let walked = frames(&out);
    assert_eq!(walked.len(), 2, "{walked:?}");
    let (position, heading) = match &walked[0] {
        Frame::Move {
            guid,
            position,
            heading,
        } => {
            assert_eq!(*guid, SPAWNER_GUID);
            (*position, *heading)
        }
        other => panic!("the marker must move first: {other:?}"),
    };
    assert!(heading[0] > 0.0 && heading[2] > 0.0);
    assert!(matches!(
        walked[1],
        Frame::Anim {
            guid: SPAWNER_GUID,
            id: 2,
            base: true
        }
    ));
    assert!(!out.changed, "marker motion is not a changed-state packet");

    // Now the player stands 3 m east of the marker: it stops and faces them.
    let out = engine
        .tick([position[0] + 3.0, position[1], position[2]], 1, 200)
        .unwrap();
    assert_eq!(
        frames(&out),
        vec![
            Frame::Move {
                guid: SPAWNER_GUID,
                position,
                heading: [1.0, 0.0, 0.0],
            },
            Frame::Anim {
                guid: SPAWNER_GUID,
                id: 1,
                base: true,
            },
        ]
    );
    assert!(!out.changed);

    // The walk is already stopped: a second notice tick emits nothing.
    let out = engine
        .tick([position[0] + 3.0, position[1], position[2]], 1, 300)
        .unwrap();
    assert!(frames(&out).is_empty());
}

#[test]
fn wanderer_notice_faces_with_the_old_clamped_divisor() {
    let fixture = Fixture::new([50.0, 0.0, 50.0], behaviour());
    let mut engine = fixture.load(None);
    // Let the marker take one walking step, then put the player almost on top
    // of it: 0.9 mm east and 1.2 mm up (1.5 mm 3-D). The notice facing divides
    // the horizontal delta by near.max(0.01) = 1 cm, so it is 0.09 long: below
    // the client's 0.010000001 squared-length threshold (0x0095dae0), which
    // keeps its facing. A 3-D unit direction would be 0.6 (squared 0.36) and
    // turn the marker.
    let out = engine.tick([-200.0, 0.0, -150.0], 1, 100).unwrap();
    let position = match frames(&out).first() {
        Some(Frame::Move { position, .. }) => *position,
        other => panic!("the marker must walk first: {other:?}"),
    };
    let player = [position[0] + 0.0009, position[1] + 0.0012, position[2]];
    let out = engine.tick(player, 1, 200).unwrap();
    let dx = player[0] - position[0];
    let dy = player[1] - position[1];
    let dz = player[2] - position[2];
    let near = (dx * dx + dy * dy + dz * dz).sqrt();
    let d = near.max(0.01);
    assert_eq!(
        frames(&out),
        vec![
            Frame::Move {
                guid: SPAWNER_GUID,
                position,
                heading: [dx / d, 0.0, dz / d],
            },
            Frame::Anim {
                guid: SPAWNER_GUID,
                id: 1,
                base: true,
            },
        ]
    );
    let face = [dx / d, 0.0, dz / d];
    assert!(
        face[0] * face[0] + face[2] * face[2] < 0.010000001,
        "the client ignores this facing"
    );
}

#[test]
fn wanderer_arrives_pauses_and_resumes() {
    let fixture = Fixture::new([50.0, 0.0, 50.0], behaviour());
    let mut engine = fixture.load(None);
    let home = MARKER_HOME;
    let player = [home[0], home[1], home[2] + 35.0];
    let angle = std::f32::consts::FRAC_PI_4;
    let goal = [
        home[0] + 5.0 * angle.cos(),
        home[1],
        home[2] + 5.0 * angle.sin(),
    ];
    let face = [(goal[0] - home[0]) / 5.0, 0.0, (goal[2] - home[2]) / 5.0];

    let first = tick(&mut engine, player, 100);
    assert_eq!(first.len(), 2, "{first:?}");
    let mut now = 100;
    let mut arrival = None;
    for _ in 0..40 {
        now += 250;
        let out = tick(&mut engine, player, now);
        if out.iter().any(|frame| {
            matches!(
                frame,
                Frame::Anim {
                    guid: SPAWNER_GUID,
                    id: 1,
                    ..
                }
            )
        }) {
            assert_eq!(
                out,
                vec![
                    Frame::Move {
                        guid: SPAWNER_GUID,
                        position: goal,
                        heading: face,
                    },
                    Frame::Anim {
                        guid: SPAWNER_GUID,
                        id: 1,
                        base: true,
                    },
                ]
            );
            arrival = Some(now);
            break;
        }
    }
    let arrival = arrival.expect("the marker reaches its first waypoint in 10 s");

    // Paused for WANDER_PAUSE_MS (3 s): no frame at all.
    assert!(tick(&mut engine, player, arrival + 100).is_empty());

    // After the pause it walks to the next waypoint.
    let resumed = tick(&mut engine, player, arrival + 3_000);
    assert_eq!(resumed.len(), 2, "{resumed:?}");
    assert!(
        matches!(
            &resumed[0],
            Frame::Move {
                guid: SPAWNER_GUID,
                heading,
                ..
            } if heading[0] < 0.0 && heading[2] == 0.0
        ),
        "{resumed:?}"
    );
    assert!(matches!(
        resumed[1],
        Frame::Anim {
            guid: SPAWNER_GUID,
            id: 2,
            base: true
        }
    ));
}
