const E: u64 = 131000108001;
const M: u64 = 131000108002;

/// Per-run scratch content; removed on drop, including while a test unwinds.
struct Scratch(PathBuf);

impl Scratch {
    fn path(name: &str) -> PathBuf {
        static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "{name}-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ))
    }

    fn new(name: &str) -> Self {
        let path = Self::path(name);
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl std::ops::Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[path = "common/combat_scale.rs"]
mod combat_scale;
/// The fixture's authored 300 health on the live scale.
fn full() -> u32 {
    combat_scale::enemy_health(300)
}

mod unit {
    // W116 synthetic shared-field oracle. All combat rules here are (R).
    use super::{E, combat_scale, full};
    /// The Ninja basic attack the fixture's players strike with (profile 2).
    pub fn n() -> u32 {
        combat_scale::basic(2)
    }
    /// The authored 204 health the area-and-DoT test uses: it dies on the
    /// second DoT tick, as 100 health did when a hit was 100.
    pub fn small() -> u32 {
        combat_scale::enemy_health(204)
    }
    use osfr_tools::encounter::EncounterEngine;
    use osfr_tools::opening_abilities::JobAbility;
    use serde_json::json;
    use sha2::{Digest, Sha256};
    use std::fs;
    pub fn fixture() -> super::Scratch {
        let dir = super::Scratch::new("w116");
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = json!({"zone_id":0,"name":"0","sky":"","geometry_id":214,"position":[0,0,0],"rotation":[0,0,1],"tutorial":false});
        let actor = json!({"guid":E,"definition_id":930108,"model_id":3,"name_id":100,"name":"930108","texture_alias":"","position":[0,0,0],"max_health":300,"damage":7,"boss":false,"respawn_seconds":2,"aggro_range":8,"leash_radius":12,"behaviour":{"mode":"stationary","reach":2.5,"attack_ms":1000}});
        let mut cave = zone.clone();
        cave["zone_id"] = json!(411);
        let mut other = actor.clone();
        other["guid"] = json!(200);
        fs::write(dir.join("encounters.json"), json!({"schema_version":1,"template_file":"npc.bin","template_sha256":format!("{:x}",Sha256::digest(template)),"world":zone,"world_enemies":[],"dungeons":[{"id":411,"title_id":12,"description_id":13,"objective_id":14,"zone":cave,"guide":null,"enemies":[other],"reward_coins":0,"reward_items":[]}]}).to_string()).unwrap();
        fs::write(
            dir.join("field_enemies.json"),
            json!({"schema_version":1,"enemies":[actor]}).to_string(),
        )
        .unwrap();
        let marker = json!({"guid":super::M,"definition_id":930109,"model_id":3,"name_id":100,"name":"930109","texture_alias":"","position":[0,0,0],"max_health":1,"damage":0,"boss":false});
        let arena_enemy = json!({"guid":300,"definition_id":930110,"model_id":3,"name_id":100,"name":"930110","texture_alias":"","position":[0,0,0],"max_health":300,"damage":0,"boss":false});
        let arena = json!({"zone_id":9600,"name":"9600","sky":"","geometry_id":214,"position":[1,0,0],"rotation":[0,0,1],"tutorial":false});
        fs::write(dir.join("field_battles.json"),json!({"schema_version":1,"battles":[{"id":96,"title_id":12,"description_id":13,"objective_id":14,"difficulty":1,"spawner":marker,"zone":arena,"enemies":[arena_enemy],"reward_coins":0}]}).to_string()).unwrap();
        dir
    }
    fn load(dir: &std::path::Path) -> EncounterEngine {
        let mut e = EncounterEngine::load(dir, None).unwrap();
        e.load_field_enemies(&dir.join("field_enemies.json"))
            .unwrap();
        e.load_field_battles(&dir.join("field_battles.json"))
            .unwrap();
        e
    }
    fn hp(out: &osfr_tools::encounter::EncounterOutcome) -> u32 {
        let p = out
            .replies
            .iter()
            .find(|p| p.starts_with(&[35, 0, 5, 0]))
            .unwrap();
        u32::from_le_bytes(p[12..16].try_into().unwrap())
    }
    fn hit(out: &osfr_tools::encounter::EncounterOutcome, attacker: u64) -> u32 {
        let p = out
            .replies
            .iter()
            .find(|p| {
                p.starts_with(&[32, 0, 7, 0])
                    && p.get(4..12) == Some(attacker.to_le_bytes().as_slice())
            })
            .unwrap();
        u32::from_le_bytes(p[28..32].try_into().unwrap())
    }
    fn caster_buff() -> JobAbility {
        serde_json::from_value(json!({"slot":1,"definition_id":62010,"name_id":1,"icon_id":1,"min_level":1,"cooldown_ms":5000,"damage_multiplier":0,"heal":0,"effects":[{"kind":"buff","recipient":"caster","duration_ms":30000,"damage_multiplier":2.0,"damage_taken_multiplier":1.0,"crit_bonus":0,"crit_damage_bonus":0.0}]})).unwrap()
    }
    fn target_debuff(duration_ms: u64) -> JobAbility {
        serde_json::from_value(json!({"slot":1,"definition_id":62011,"name_id":1,"icon_id":1,"min_level":1,"cooldown_ms":5000,"range":5,"damage_multiplier":0,"heal":0,"effects":[{"kind":"buff","recipient":"target","duration_ms":duration_ms,"damage_multiplier":1.0,"damage_taken_multiplier":2.0,"crit_bonus":0,"crit_damage_bonus":0.0}]})).unwrap()
    }
    #[test]
    fn two_players_share_hp_and_keep_independent_cooldowns() {
        let dir = fixture();
        let mut a = load(&dir);
        let mut b = load(&dir);
        let canary = std::env::var_os("OSFR_W116_LOCAL_COPY_CANARY").is_some();
        let mut field = a.take_shared_fields().unwrap();
        if !canary {
            b.take_shared_fields();
        }
        assert_eq!(
            hp(&a
                .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 0)
                .unwrap()),
            full() - n()
        );
        assert_eq!(
            hp(&if canary {
                b.attack(E, [-1., 0., 0.], 2, 0, 100).unwrap()
            } else {
                b.shared_attack(&mut field, 202, E, [-1., 0., 0.], 2, 100)
                    .unwrap()
            }),
            full() - 2 * n(),
            "local-copy canary: B must strike the HP left by A"
        );
        assert!(
            a.shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 100)
                .unwrap()
                .replies
                .is_empty()
        );
        assert_eq!(field.health(E), Some(full() - 2 * n()));
        for t in 1..=10 {
            for engine in [&mut a, &mut b] {
                let out = engine.tick([1., 0., 0.], 2, t * 100).unwrap();
                assert!(
                    out.replies
                        .iter()
                        .all(|p| !p.starts_with(&[32, 0, 7, 0]) && !p.starts_with(&[35, 0, 2, 0])),
                    "local path emitted a field hit/spawn"
                );
            }
        }
        assert_eq!(field.health(E), Some(full() - 2 * n()));
    }

    #[test]
    fn two_players_do_not_accelerate_motion_or_enemy_attack_clock() {
        let dir = fixture();
        // A chaser (not the fixture's stationary default) so the far-player
        // phase exercises the enemy's `moving` set as well as the marker's
        // `wander` walk.
        let path = dir.join("field_enemies.json");
        let mut content: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        content["enemies"][0]["aggro_range"] = json!(40);
        content["enemies"][0]["behaviour"]["mode"] = json!("melee");
        fs::write(&path, content.to_string()).unwrap();
        let mut a = load(&dir);
        let mut b = load(&dir);
        let mut field = a.take_shared_fields().unwrap();
        b.take_shared_fields();
        a.shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 0)
            .unwrap();
        b.shared_attack(&mut field, 202, E, [-1., 0., 0.], 2, 100)
            .unwrap();
        let mut control = field.clone();
        let mut driver = a.clone();
        let players = [
            a.field_participant(101, [1., 0., 0.], true),
            b.field_participant(202, [-1., 0., 0.], true),
        ];
        for t in 1..=10 {
            let now = t * 100;
            let elapsed = field.due(now).unwrap();
            let control_elapsed = control.due(now).unwrap();
            let (_, hits) = a.shared_motion(&mut field, &players, now, elapsed).unwrap();
            let (_, control_hits) = driver
                .shared_motion(&mut control, &players[..1], now, control_elapsed)
                .unwrap();
            assert_eq!(hits, control_hits);
            assert!(hits.iter().all(|(target, _)| *target == 101));
            assert_eq!(field.health(E), control.health(E));
            assert_eq!(field.position(E), control.position(E));
            assert_eq!(field.attack_clock(E), control.attack_clock(E));
            assert_eq!(field.target(E), control.target(E));
            assert!(
                field.due(now).is_none(),
                "second observer must not advance the same world tick"
            );
        }
        assert!(
            field.attack_clock(E).is_some(),
            "the enemy attack clock is swapped into the shared field"
        );
        // Far players let both the chaser and the battle marker walk: each
        // writes one of the swapped collections, so a swap that missed them
        // would leave the canonical state empty.
        let far = [
            a.field_participant(101, [10., 0., 0.], true),
            b.field_participant(202, [-10., 0., 0.], true),
        ];
        for t in 11..=15 {
            let now = t * 100;
            let elapsed = field.due(now).unwrap();
            let control_elapsed = control.due(now).unwrap();
            let (_, hits) = a.shared_motion(&mut field, &far, now, elapsed).unwrap();
            let (_, control_hits) = driver
                .shared_motion(&mut control, &far[..1], now, control_elapsed)
                .unwrap();
            assert_eq!(hits, control_hits);
            assert_eq!(field.position(E), control.position(E));
            assert_eq!(field.position(super::M), control.position(super::M));
            assert_eq!(field.attack_clock(E), control.attack_clock(E));
            assert!(field.due(now).is_none());
        }
        assert_ne!(
            field.position(super::M),
            Some([0., 0., 0.]),
            "the marker's walk is swapped into the shared field"
        );
        let initial = field.initialization(E).unwrap();
        let animation = initial
            .iter()
            .find(|p| p.starts_with(&[35, 0, 8, 0]))
            .expect("the late-join animation");
        // 35/8 SetAnimation: guid then the animation id. Ctor 0x008b3ee0
        // calls the family ctor 0089fbb0(8); dispatcher 0x0092f540 case 8
        // calls it at 0x009315ce. Id 3 is the walking pose.
        assert_eq!(
            i32::from_le_bytes(animation[12..16].try_into().unwrap()),
            3,
            "the walk animation state is swapped into the shared field"
        );
    }
    #[test]
    fn malformed_and_unknown_targets_do_not_mutate_or_consume_swing() {
        let dir = fixture();
        let mut a = load(&dir);
        let mut field = a.take_shared_fields().unwrap();
        let character = osfr_tools::character_store::CharacterState {
            guid: 101,
            zone_id: 0,
            position: [1., 0., 0.],
            rotation: [0., 0., 1.],
        };
        let good = [
            vec![36, 0, 10, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            E.to_le_bytes().to_vec(),
        ]
        .concat();
        let before = (
            field.health(E),
            field.position(E),
            field.attack_clock(E),
            field.pending_kill_credits(101),
        );
        for end in 0..good.len() {
            assert!(!matches!(
                a.shared_action(&good[..end], &character),
                Ok(Some(_))
            ));
        }
        let mut trailing = good.clone();
        trailing.push(0);
        assert!(a.shared_action(&trailing, &character).is_err());
        let mut bad_kind = good.clone();
        bad_kind[12] = 2;
        assert!(a.shared_action(&bad_kind, &character).is_err());
        for target in [0, 1, 202, u64::MAX, E + 1] {
            assert!(
                a.shared_attack(&mut field, 101, target, [1., 0., 0.], 2, 0)
                    .unwrap()
                    .replies
                    .is_empty()
            );
        }
        assert!(
            a.shared_attack(&mut field, 101, E, [f32::NAN, 0., 0.], 2, 0)
                .is_err()
        );
        assert_eq!(
            (
                field.health(E),
                field.position(E),
                field.attack_clock(E),
                field.pending_kill_credits(101),
            ),
            before,
            "a refused shared action changes nothing"
        );
        assert_eq!(field.health(E), Some(full()));
        assert_eq!(
            hp(&a
                .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 0)
                .unwrap()),
            full() - n()
        );
    }
    #[test]
    fn world_refuses_unready_unknown_or_other_instance_field_access() {
        use osfr_tools::{
            content::Zone,
            world::{World, WorldPose},
        };
        let dir = fixture();
        let mut e = load(&dir);
        let mut world = World::runtime();
        world.install_field(0, e.take_shared_fields().unwrap());
        world
            .register_runtime_zone(Zone {
                id: 9600,
                name: "9600".into(),
                geometry_id: Some(9600),
                zone_type: Some(2),
                tutorial: Some(false),
                sky: Some(String::new()),
                source: vec![],
            })
            .unwrap();
        let a = world
            .prepare_membership(101, 0, None, WorldPose::ORIGIN, true, 0)
            .unwrap();
        world.commit_membership(a);
        let b = world
            .prepare_membership(202, 9600, None, WorldPose::ORIGIN, true, 0)
            .unwrap();
        world.commit_membership(b);
        assert!(world.field_for_player(101, 0).is_some());
        assert!(world.field_for_player(202, 0).is_none());
        assert!(world.field_for_player(202, 9600).is_none());
        assert!(world.field_for_player(999, 0).is_none());
        assert!(world.field_for_player(101, 9600).is_none());
        let a = world
            .prepare_membership(101, 0, Some(0), WorldPose::ORIGIN, false, 1)
            .unwrap();
        world.commit_membership(a);
        assert!(world.field_for_player(101, 0).is_none());
        assert_eq!(world.field(0).unwrap().health(E), Some(full()));
    }
    // The protected name's "no personal credit" means T240 *reward* credit:
    // the shared DoT pays no `rewards` here. Since T256 the caster's engine
    // reports the defeats it lands, which is that caster's personal quest credit
    // (asserted below). Rename only when the protected list is refreshed.
    #[test]
    fn area_and_dot_damage_share_hp_and_never_deliver_personal_credit() {
        use osfr_tools::opening_abilities::JobAbility;
        let dir = fixture();
        let path = dir.join("field_enemies.json");
        let mut content: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let mut other = content["enemies"][0].clone();
        other["guid"] = json!(E + 10);
        other["position"] = json!([0, 0, 1]);
        content["enemies"].as_array_mut().unwrap().push(other);
        for enemy in content["enemies"].as_array_mut().unwrap() {
            enemy["max_health"] = json!(204);
        }
        let dot = (n() as f32 * 0.5).round() as u32;
        fs::write(path, content.to_string()).unwrap();
        let mut a = load(&dir);
        let mut b = load(&dir);
        let mut field = a.take_shared_fields().unwrap();
        b.take_shared_fields();
        let skill:JobAbility=serde_json::from_value(json!({"slot":1,"definition_id":62000,"name_id":1,"icon_id":1,"min_level":1,"cooldown_ms":5000,"damage_multiplier":1.,"heal":0,"effects":[{"kind":"area_damage","radius":6,"max_targets":2,"centered_on_caster":true},{"kind":"damage_over_time","multiplier":0.5,"duration_ms":2000,"interval_ms":1000}]})).unwrap();
        assert_eq!(
            hp(&a
                .shared_job_ability(&mut field, 101, &skill, 101, [1., 0., 0.], 2, 0)
                .unwrap()),
            small() - n()
        );
        assert_eq!(
            hp(&b
                .shared_job_ability(&mut field, 202, &skill, 202, [-1., 0., 0.], 2, 100)
                .unwrap()),
            small() - 2 * n()
        );
        assert_eq!(field.health(E + 10), Some(small() - 2 * n()));
        let out = a.shared_effects(&mut field, 101, 1000).unwrap();
        assert_eq!(hp(&out), small() - 2 * n() - dot);
        let out = b.shared_effects(&mut field, 202, 1100).unwrap();
        assert_eq!(hp(&out), 0);
        assert_eq!(field.health(E + 10), Some(0));
        // T256: the caster's DoT lands both lethal hits, so its own engine
        // reports both defeats (the caster's quest credit); the T240 payout
        // is still unclaimed.
        assert_eq!(out.defeated, [930108, 930108]);
        assert!(out.rewards.is_empty() && out.completed.is_none());
        let out = a.shared_effects(&mut field, 101, 2000).unwrap();
        assert!(out.replies.is_empty());
        assert!(
            a.shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 3000)
                .unwrap()
                .replies
                .is_empty()
        );
        assert_eq!(field.health(E), Some(0));
    }
    #[test]
    fn caster_buff_is_personal_and_target_debuff_is_shared_until_expiry() {
        let dir = fixture();
        let path = dir.join("field_enemies.json");
        let mut content: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        content["enemies"][0]["max_health"] = json!(2000);
        fs::write(&path, content.to_string()).unwrap();
        let mut a = load(&dir);
        let mut b = load(&dir);
        let mut field = a.take_shared_fields().unwrap();
        b.take_shared_fields();
        let caster = caster_buff();
        let debuff = target_debuff(2000);
        a.shared_job_ability(&mut field, 101, &caster, 0, [1., 0., 0.], 2, 0)
            .unwrap();
        a.shared_effects(&mut field, 101, 100).unwrap();
        a.shared_motion(&mut field, &[], 200, 100).unwrap();
        let out = a
            .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 700)
            .unwrap();
        assert_eq!(hit(&out, 101), 2 * n());
        let out = b
            .shared_attack(&mut field, 202, E, [-1., 0., 0.], 2, 1300)
            .unwrap();
        assert_eq!(hit(&out, 202), n(), "B must not inherit A's caster buff");
        a.shared_job_ability(&mut field, 101, &debuff, E, [1., 0., 0.], 2, 1400)
            .unwrap();
        a.shared_effects(&mut field, 101, 1500).unwrap();
        let players = [
            a.field_participant(101, [1., 0., 0.], true),
            b.field_participant(202, [-1., 0., 0.], true),
        ];
        a.shared_motion(&mut field, &players, 1600, 100).unwrap();
        let out = b
            .shared_attack(&mut field, 202, E, [-1., 0., 0.], 2, 2100)
            .unwrap();
        assert_eq!(
            hit(&out, 202),
            2 * n(),
            "the debuff applies to either attacker"
        );
        let out = a
            .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 2800)
            .unwrap();
        assert_eq!(
            hit(&out, 101),
            4 * n(),
            "A keeps its caster buff after shared calls"
        );
        let out = b
            .shared_attack(&mut field, 202, E, [-1., 0., 0.], 2, 3500)
            .unwrap();
        assert_eq!(hit(&out, 202), n(), "the debuff expires at 3400");
        let out = a
            .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 4200)
            .unwrap();
        assert_eq!(
            hit(&out, 101),
            2 * n(),
            "the caster buff outlives the debuff"
        );
    }
    #[test]
    fn refused_shared_ability_leaves_skill_effects_untouched() {
        let dir = fixture();
        let path = dir.join("field_enemies.json");
        let mut content: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        content["enemies"][0]["max_health"] = json!(2000);
        fs::write(&path, content.to_string()).unwrap();
        let mut a = load(&dir);
        let mut b = load(&dir);
        let mut field = a.take_shared_fields().unwrap();
        b.take_shared_fields();
        let caster = caster_buff();
        let debuff = target_debuff(30000);
        a.shared_job_ability(&mut field, 101, &caster, 0, [1., 0., 0.], 2, 0)
            .unwrap();
        a.shared_job_ability(&mut field, 101, &debuff, E, [1., 0., 0.], 2, 700)
            .unwrap();
        // An invalid position errors inside the swap.
        assert!(
            b.shared_job_ability(&mut field, 202, &debuff, E, [f32::NAN, 0., 0.], 2, 1300)
                .is_err()
        );
        // An unavailable ability returns the 286 refusal through the same swap.
        let mut unavailable = target_debuff(30000);
        unavailable.unavailable = true;
        let out = b
            .shared_job_ability(&mut field, 202, &unavailable, E, [-1., 0., 0.], 2, 1300)
            .unwrap();
        assert!(
            out.replies
                .iter()
                .any(|p| p.as_slice() == [36u8, 0, 1, 0, 30, 1, 0, 0]),
            "the unavailable cast returns the 286 failure"
        );
        let out = a
            .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 2000)
            .unwrap();
        assert_eq!(
            hit(&out, 101),
            4 * n(),
            "both skill maps survived the refused casts"
        );
    }
    #[test]
    fn canonical_run_id_is_sent_and_survives_solo_validation() {
        let dir = fixture();
        let mut e = load(&dir);
        e.begin(
            411,
            osfr_tools::character_store::CharacterState {
                guid: 101,
                zone_id: 0,
                position: [1., 0., 0.],
                rotation: [0., 0., 1.],
            },
            2,
        )
        .unwrap();
        assert!(e.bind_run_instance(22, false).unwrap());
        let out = e.on_zone_ready(2, 0).unwrap();
        let packets: Vec<_> = out
            .replies
            .iter()
            .filter(|p| p.starts_with(&[41, 0]) && p.len() >= 12)
            .collect();
        assert!(!packets.is_empty());
        for p in packets {
            assert_eq!(i32::from_le_bytes(p[8..12].try_into().unwrap()), 22);
        }
        let restored = EncounterEngine::load(&dir, Some(e.state().clone())).unwrap();
        assert_eq!(restored.state().active.as_ref().unwrap().instance_id, 22);
        assert!(restored.state().next_instance_id > 22);
    }
}
#[path = "../src/bin/osfr-bot.rs"]
#[allow(dead_code)]
mod bot;
#[path = "../src/bin/osfr-gateway.rs"]
#[allow(dead_code)]
mod gateway_runner;
use gateway_runner::{Options, Server};
use osfr_tools::{
    account::{AccountFlags, SlotPolicy, create_character},
    account_store::AccountStore,
    character_store::CharacterState,
    identity::{NativeCreation, Resources},
    inventory::{EquippedItem, InventoryItem},
    login,
};
use serde_json::json;
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
const VERSION: &str = "1.910.1.530630";
const HANDOFF_KEY: &[u8] = b"synthetic-handoff-key";

type HostAction = Box<dyn FnOnce(&mut Server) + Send>;
struct Fixture {
    addr: SocketAddr,
    tx: Option<mpsc::Sender<HostAction>>,
    handle: Option<thread::JoinHandle<()>>,
    root: PathBuf,
    accounts: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = Scratch::path("socket");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-gateway-reference");
        let accounts = create_account_store(&root);
        let content = create_content(&root);
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference,
            state: root.join("state.json"),
            content: Some(content),
            identity: None,
            account_store: Some(accounts.clone()),
            gateway_ticket: String::new(),
            handoff_key: HANDOFF_KEY.to_vec(),
            seconds: 0,
            dev_tools: false,
            studio_port: 0,
            client_logs: None,
            terrain: Some({
                let directory = root.join("terrain");
                fs::create_dir_all(&directory).unwrap();
                let mut grid =
                    osfr_tools::terrain::HeightGrid::new([-32., -32.], [32., 32.], 2.).unwrap();
                grid.add_triangle([[-32., 0., -32.], [32., 0., -32.], [32., 0., 32.]]);
                grid.add_triangle([[-32., 0., -32.], [32., 0., 32.], [-32., 0., 32.]]);
                fs::write(directory.join("zone_0.hgt"), grid.to_bytes()).unwrap();
                std::sync::Arc::new(osfr_tools::terrain::Terrain::load(&directory).unwrap())
            }),
            world: None,
            stock_text: None,
            allow_lan: false,
            security_log: None,
            group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
            guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
        };
        let (tx, rx) = mpsc::channel::<HostAction>();
        let (addr_tx, addr_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let mut server = Server::new(options).unwrap();
            addr_tx.send(server.local_addr().unwrap()).unwrap();
            loop {
                match rx.try_recv() {
                    Ok(action) => action(&mut server),
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
                server.step().unwrap();
            }
            server.shutdown().unwrap();
        });
        Self {
            addr: addr_rx.recv_timeout(PATIENCE).unwrap(),
            tx: Some(tx),
            handle: Some(handle),
            root,
            accounts,
        }
    }

    fn host<T: Send + 'static>(&self, f: impl FnOnce(&mut Server) -> T + Send + 'static) -> T {
        let (tx, rx) = mpsc::channel();
        self.tx
            .as_ref()
            .unwrap()
            .send(Box::new(move |server| {
                tx.send(f(server)).unwrap();
            }))
            .unwrap();
        rx.recv_timeout(PATIENCE).unwrap()
    }

    fn bot(&self, guid: u64, connection_id: u32) -> bot::Bot {
        let store = AccountStore::load(&self.accounts).unwrap();
        let account = store
            .account(store.character(guid).unwrap().owner_account_id)
            .unwrap();
        bot::Bot::connect(bot::BotConfig {
            gateway: self.addr,
            name: guid.to_string(),
            connection_id,
            guid,
            ticket: login::gateway_binding_ticket(HANDOFF_KEY, account, guid),
            version: VERSION.into(),
            expect_timeout_ms: PATIENCE.as_millis() as u64,
            quiet: true,
        })
        .unwrap()
    }

    fn ready(&self, guid: u64, a: &mut bot::Bot, b: &mut bot::Bot) -> [Vec<Vec<u8>>; 2] {
        let deadline = Instant::now() + PATIENCE;
        let mut messages = [Vec::new(), Vec::new()];
        let mut load_sent = false;
        loop {
            let batch = collect(a, b, Duration::from_millis(5));
            messages[0].extend(batch[0].clone());
            messages[1].extend(batch[1].clone());
            if self.host(move |s| s.test_location(guid).is_some_and(|row| row.2)) {
                return messages;
            }
            if !load_sent {
                let bot = if guid == 101 { &mut *a } else { &mut *b };
                bot.send_message(&packet(13, &[]), "13").unwrap();
                bot.send_message(&packet(10, &[]), "10").unwrap();
                load_sent = true;
            }
            assert!(Instant::now() < deadline, "readiness barrier for {guid}");
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
        // The scratch directory is removed after the server stops.
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Upper bound for any wait on the server or the other bot; never a pace. Parallel
/// suites starve the gateway thread for seconds, so fixed 3 s / 250 ms waits flaked (W157).
const PATIENCE: Duration = Duration::from_secs(30);

/// Collect into `acc` until `done` holds (or PATIENCE), then keep listening 500 ms so a
/// duplicate delivery still fails the exact-count assertions that follow.
fn settle(
    a: &mut bot::Bot,
    b: &mut bot::Bot,
    acc: &mut [Vec<Vec<u8>>; 2],
    done: impl Fn(&[Vec<Vec<u8>>; 2]) -> bool,
) {
    let deadline = Instant::now() + PATIENCE;
    while !done(acc) && Instant::now() < deadline {
        let batch = collect(a, b, Duration::from_millis(5));
        for i in 0..2 {
            acc[i].extend(batch[i].clone());
        }
    }
    let grace = collect(a, b, Duration::from_millis(500));
    for i in 0..2 {
        acc[i].extend(grace[i].clone());
    }
}

/// After a committed result reached its recipient, keep servicing both bots until the
/// server's reliable queues are acknowledged (the other bot's copy has landed), then
/// 100 ms more so a duplicate still fails the exact-count assertions.
fn drain_deliveries(f: &Fixture, a: &mut bot::Bot, b: &mut bot::Bot, acc: &mut [Vec<Vec<u8>>; 2]) {
    let deadline = Instant::now() + PATIENCE;
    while !f.host(|s| s.test_queues_idle()) && Instant::now() < deadline {
        let batch = collect(a, b, Duration::from_millis(5));
        for i in 0..2 {
            acc[i].extend(batch[i].clone());
        }
    }
    let tail = collect(a, b, Duration::from_millis(100));
    for i in 0..2 {
        acc[i].extend(tail[i].clone());
    }
}

fn collect(a: &mut bot::Bot, b: &mut bot::Bot, interval: Duration) -> [Vec<Vec<u8>>; 2] {
    let mut out = [Vec::new(), Vec::new()];
    let deadline = Instant::now() + interval;
    while Instant::now() < deadline {
        for (index, bot) in [&mut *a, &mut *b].into_iter().enumerate() {
            bot.pump(Duration::from_millis(2)).unwrap();
            while let Ok(message) = bot.wait_for(Duration::ZERO, |_| true) {
                out[index].push(message);
            }
        }
    }
    out
}

fn is_sub(message: &[u8], family: u16, sub: u16) -> bool {
    message.starts_with(&[
        family as u8,
        (family >> 8) as u8,
        sub as u8,
        (sub >> 8) as u8,
    ])
}
fn actor_packets(messages: &[Vec<u8>], sub: u16, guid: u64) -> usize {
    messages
        .iter()
        .filter(|m| is_sub(m, 35, sub) && m.get(4..12) == Some(guid.to_le_bytes().as_slice()))
        .count()
}
fn numeric_name(id: u64) -> String {
    id.to_string()
        .bytes()
        .map(|digit| char::from(b'a' + digit - b'0'))
        .collect()
}
fn wrapped(outer: u16, inner: &[u8]) -> Vec<u8> {
    let mut out = outer.to_le_bytes().to_vec();
    out.push(1);
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}
fn packet(opcode: u16, fields: &[u8]) -> Vec<u8> {
    let mut inner = opcode.to_le_bytes().to_vec();
    inner.extend_from_slice(fields);
    wrapped(5, &inner)
}
fn create_account_store(root: &Path) -> PathBuf {
    let resources_dir = root.join("creator");
    let character_create = resources_dir.join("CharacterCreate");
    fs::create_dir_all(&character_create).unwrap();
    let mut model = vec!["0"; 17];
    model[0] = "1";
    model[5] = "1";
    model[13] = "1";
    fs::write(resources_dir.join("Models.txt"), model.join("^")).unwrap();
    for (file, row) in [
        ("HeadMappings.txt", "1^head.adr^"),
        ("HairMappings.txt", "2^hair.adr^"),
        ("ModelCustomizationMappings.txt", "1^beard.adr^"),
        ("FacePaintMappings.txt", "0^none^"),
        ("SkinToneMappings.txt", "1^skin.adr^"),
    ] {
        fs::write(character_create.join(file), row).unwrap();
    }
    let mut definitions = String::new();
    for (id, slot) in [(201, 5), (189, 4), (199, 3)] {
        let mut row = vec!["0".to_owned(); 42];
        row[0] = id.to_string();
        row[12] = slot.to_string();
        row[16] = "item.adr".into();
        row[17] = "1".into();
        row[18] = "cloth".into();
        row[19] = "0".into();
        row[41] = "tint".into();
        definitions.push_str(&row.join("^"));
        definitions.push('\n');
    }
    fs::write(character_create.join("ItemDefinitions.txt"), definitions).unwrap();
    let resources = Resources::load(&resources_dir).unwrap();
    let path = root.join("accounts.json");
    AccountStore::new().save(&path).unwrap();
    for id in [101u64, 202] {
        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document["store"]["next_account_id"] = json!(id);
        document["store"]["next_character_guid"] = json!(id);
        fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        let mut store = AccountStore::load(&path).unwrap();
        let account = store
            .create_account(
                [id as u8; 32],
                AccountFlags {
                    open_chat: true,
                    ..Default::default()
                },
                SlotPolicy { max_characters: 1 },
            )
            .unwrap();
        let mut choices = [0; 21];
        for (index, value) in [
            (0, 1),
            (1, 1),
            (2, 2),
            (5, 1),
            (7, 201),
            (8, 189),
            (9, 199),
            (20, 1),
        ] {
            choices[index] = value;
        }
        let reply = create_character(
            &mut store,
            account,
            NativeCreation {
                first_name: numeric_name(id),
                last_name: numeric_name(id),
                temporary_first_name: String::new(),
                temporary_last_name: String::new(),
                choices,
            },
            &resources,
        );
        assert_eq!(
            reply.status,
            osfr_tools::account::CharacterCreateStatus::Created
        );
        store.save(&path).unwrap();
    }
    path
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for row in fs::read_dir(from).unwrap() {
        let path = row.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            fs::copy(&path, target).unwrap();
        }
    }
}
fn create_content(root: &Path) -> PathBuf {
    let source = PathBuf::from(
        std::env::var_os("OSFR_PLAY_PACKAGE").expect("private fixture requires OSFR_PLAY_PACKAGE"),
    )
    .join("content");
    let content = root.join("content");
    copy_tree(&source, &content);
    let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    copy_tree(&bundled.join("npcs"), &content.join("npcs"));
    for file in ["zones.json", "npc_spawns_extra.json"] {
        fs::copy(bundled.join(file), content.join(file)).unwrap();
    }
    let synthetic = unit::fixture();
    fs::copy(
        synthetic.join("field_enemies.json"),
        content.join("field_enemies.json"),
    )
    .unwrap();
    fs::copy(
        synthetic.join("field_battles.json"),
        content.join("field_battles.json"),
    )
    .unwrap();
    let mut seed = osfr_tools::gameplay::Gameplay::load(&content, None)
        .unwrap()
        .state();
    seed.profiles.as_mut().unwrap().unlocked.insert(2);
    let weapon_instance = 760_000_001;
    seed.inventory.items.push(InventoryItem {
        definition_id: 7592,
        tint: 0,
        id: weapon_instance,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    seed.inventory
        .equipment
        .retain(|row| !(row.profile_id == 43 && row.slot == 7));
    seed.inventory.equipment.push(EquippedItem {
        profile_id: 43,
        slot: 7,
        item_id: weapon_instance,
        category: 7,
    });
    let mut game = osfr_tools::gameplay::Gameplay::load(&content, Some(seed)).unwrap();
    let at = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [1., 0., 0.],
        rotation: [0., 0., 1.],
    };
    game.handle_in_world(&[26, 0, 13, 0, 43, 0, 0, 0], &at, 0)
        .unwrap();
    fs::create_dir_all(root.join("characters")).unwrap();
    for guid in [101, 202] {
        let mut character = at.clone();
        character.guid = guid;
        if guid == 202 {
            character.position = [-1., 0., 0.];
        }
        osfr_tools::game_store::save(
            &root.join(format!("characters/{guid}.json")),
            &osfr_tools::game_store::Snapshot {
                character,
                gameplay: Some(game.state()),
                pets: None,
            },
        )
        .unwrap();
    }
    content
}
fn hp_values(messages: &[Vec<u8>]) -> Vec<u32> {
    messages
        .iter()
        .filter(|p| is_sub(p, 35, 5) && p.get(4..12) == Some(E.to_le_bytes().as_slice()))
        .map(|p| u32::from_le_bytes(p[12..16].try_into().unwrap()))
        .collect()
}
#[test]
#[ignore = "requires private OSFR_PLAY_PACKAGE; all actors/accounts/requests are synthetic"]
fn two_bot_socket_shared_hp_route() {
    let f = Fixture::new();
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let store = AccountStore::load(&f.accounts).unwrap();
    for (guid, bot) in [(101, &mut a), (202, &mut b)] {
        bot.send_message(
            &bot::login_message(
                guid,
                &login::gateway_binding_ticket(HANDOFF_KEY, store.account(guid).unwrap(), guid),
                VERSION,
            ),
            "1",
        )
        .unwrap();
        bot.wait_for(PATIENCE, |_| true).unwrap();
        bot.send_message(&packet(13, &[]), "13").unwrap();
        bot.wait_for(PATIENCE, |_| true).unwrap();
    }
    a.send_message(&packet(10, &[]), "10").unwrap();
    let mut startup = f.ready(101, &mut a, &mut b);
    b.send_message(&packet(10, &[]), "10").unwrap();
    let batch = f.ready(202, &mut a, &mut b);
    for i in 0..2 {
        startup[i].extend(batch[i].clone());
    }
    // Barrier on server-observed reliable drain, while servicing both sockets.
    let deadline = Instant::now() + PATIENCE;
    loop {
        let batch = collect(&mut a, &mut b, Duration::from_millis(20));
        for i in 0..2 {
            startup[i].extend(batch[i].clone());
        }
        if f.host(|s| s.test_queues_idle()) {
            break;
        }
        assert!(Instant::now() < deadline, "startup queues did not drain");
    }
    for (i, packets) in startup.iter().enumerate() {
        println!(
            "startup {i}: E={} M={} hp={:?}",
            actor_packets(packets, 2, E),
            actor_packets(packets, 2, M),
            hp_values(packets)
        );
        assert_eq!(actor_packets(packets, 2, E), 1, "E {i}");
        assert_eq!(actor_packets(packets, 2, M), 1, "M {i}");
        assert_eq!(hp_values(packets), vec![full()]);
    }
    let cast = [
        vec![36, 0, 10, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        E.to_le_bytes().to_vec(),
    ]
    .concat();
    let mut self_cast = cast.clone();
    self_cast[16..24].copy_from_slice(&101_u64.to_le_bytes());
    a.send_message(&bot::wrap(&self_cast), "36/10 self")
        .unwrap();
    let first = attack_result(&f, &mut a, &mut b, 0);
    assert_eq!(hp_values(&first[0]), vec![full() - unit::n()]);
    b.send_message(&bot::wrap(&cast), "36/10").unwrap();
    a.send_message(&bot::wrap(&cast), "36/10 early").unwrap();
    let second = attack_result(&f, &mut a, &mut b, 1);
    assert_eq!(
        hp_values(&second[1]),
        vec![full() - 2 * unit::n()],
        "local-copy canary: B must use canonical HP"
    );
    assert_eq!(hp_values(&first[1]), vec![full() - unit::n()]);
    assert_eq!(hp_values(&second[0]), vec![full() - 2 * unit::n()]);
    for (packets, actor) in [(&first[0], 101_u64), (&second[1], 202_u64)] {
        assert_eq!(
            packets
                .iter()
                .filter(|p| is_sub(p, 32, 7)
                    && p.get(4..12) == Some(actor.to_le_bytes().as_slice())
                    && p.get(20..28) == Some(E.to_le_bytes().as_slice()))
                .count(),
            1
        );
    }
    let early = collect(&mut a, &mut b, Duration::from_millis(500));
    for messages in early {
        assert!(hp_values(&messages).is_empty());
        assert_eq!(actor_packets(&messages, 2, E), 0);
    }
    assert_eq!(
        f.host(|s| s.test_field_health(E)),
        Some(full() - 2 * unit::n())
    );
    // A parsed but unknown target, and a truncated targeted ability, cannot mutate.
    let mut unknown = cast.clone();
    unknown[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    a.send_message(&bot::wrap(&unknown), "36/10 unknown")
        .unwrap();
    a.send_message(&bot::wrap(&cast[..23]), "36/10 truncated")
        .unwrap();
    let refused = collect(&mut a, &mut b, Duration::from_millis(500));
    assert!(refused.iter().all(|p| hp_values(p).is_empty()));
    assert_eq!(
        f.host(|s| s.test_field_health(E)),
        Some(full() - 2 * unit::n())
    );
    // Live equipment removal must revoke shared casts for B alone.
    let unequip = [
        vec![42, 0, 2, 0],
        7_i32.to_le_bytes().to_vec(),
        43_i32.to_le_bytes().to_vec(),
    ]
    .concat();
    b.send_message(&bot::wrap(&unequip), "42/2").unwrap();
    b.send_message(&bot::wrap(&cast), "36/10 unarmed").unwrap();
    let refused = collect(&mut a, &mut b, Duration::from_millis(500));
    assert!(refused.iter().all(|p| hp_values(p).is_empty()));
    assert_eq!(
        f.host(|s| s.test_field_health(E)),
        Some(full() - 2 * unit::n())
    );
    // Existing original battle-marker / Go route, with separate canonical runs.
    let interact = [vec![26, 0, 8, 0], M.to_le_bytes().to_vec(), vec![0]].concat();
    let go = [
        vec![41, 0, 108, 0],
        96_i32.to_le_bytes().to_vec(),
        1_i32.to_le_bytes().to_vec(),
    ]
    .concat();
    for who in 0..2 {
        let client = if who == 0 { &mut a } else { &mut b };
        client
            .send_message(&bot::wrap(&interact), "26/8 marker")
            .unwrap();
        let deadline = Instant::now() + PATIENCE;
        loop {
            let packets = collect(&mut a, &mut b, Duration::from_millis(10));
            if packets[who].iter().any(|p| is_sub(p, 41, 114)) {
                break;
            }
            assert!(Instant::now() < deadline);
        }
        let client = if who == 0 { &mut a } else { &mut b };
        client.send_message(&bot::wrap(&go), "41/108").unwrap();
        let guid = if who == 0 { 101 } else { 202 };
        let deadline = Instant::now() + PATIENCE;
        loop {
            collect(&mut a, &mut b, Duration::from_millis(10));
            if f.host(move |s| {
                s.test_location(guid)
                    .is_some_and(|row| row.0.zone_id == 9600)
            }) {
                break;
            }
            assert!(Instant::now() < deadline);
        }
        let client = if who == 0 { &mut a } else { &mut b };
        client.send_message(&packet(13, &[]), "13").unwrap();
        client.send_message(&packet(10, &[]), "10").unwrap();
        let mut packets = f.ready(guid, &mut a, &mut b);
        let deadline = Instant::now() + PATIENCE;
        loop {
            let batch = collect(&mut a, &mut b, Duration::from_millis(20));
            for i in 0..2 {
                packets[i].extend(batch[i].clone());
            }
            if f.host(|s| s.test_queues_idle()) {
                break;
            }
            assert!(Instant::now() < deadline);
        }
        let canonical = f.host(move |s| s.test_location(guid).unwrap().1);
        let saved: serde_json::Value = serde_json::from_slice(
            &fs::read(f.root.join(format!("characters/{guid}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(
            saved["gameplay"]["encounter"]["active"]["instance_id"],
            json!(canonical)
        );
        assert!(
            packets[who].iter().any(|p| is_sub(p, 41, 114)
                && i32::from_le_bytes(p[8..12].try_into().unwrap()) == canonical)
        );
    }
    assert_ne!(
        f.host(|s| s.test_location(101).unwrap().1),
        f.host(|s| s.test_location(202).unwrap().1)
    );
    b.send_message(&bot::wrap(&cast), "36/10 other instance")
        .unwrap();
    collect(&mut a, &mut b, Duration::from_millis(500));
    assert_eq!(
        f.host(|s| s.test_field_health(E)),
        Some(full() - 2 * unit::n())
    );
    println!(
        "W116 receipt: shared HP [200,100], early/unknown/truncated/foreign-instance requests refused; canonical run ids match saves and 41/114. Scratch {}, removed when the fixture drops",
        f.root.display()
    );
}

fn attack_result(
    f: &Fixture,
    a: &mut bot::Bot,
    b: &mut bot::Bot,
    recipient: usize,
) -> [Vec<Vec<u8>>; 2] {
    let deadline = Instant::now() + PATIENCE;
    let mut messages = [Vec::new(), Vec::new()];
    loop {
        let batch = collect(a, b, Duration::from_millis(5));
        for i in 0..2 {
            messages[i].extend(batch[i].clone());
        }
        if !hp_values(&messages[recipient]).is_empty() {
            break;
        }
        assert!(Instant::now() < deadline, "committed HP did not arrive");
    }
    drain_deliveries(f, a, b, &mut messages);
    messages
}
