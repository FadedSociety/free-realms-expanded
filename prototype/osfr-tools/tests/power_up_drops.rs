//! Combat power-up drops, slice 2: a kill rolls the enemy's table, the pickup
//! actor appears, walking onto it applies or stores it, and it despawns after
//! its ttl. Slot display and use are slice 3 and are not tested here.
use osfr_tools::{
    encounter::{EncounterEngine, EncounterOutcome},
    opening_abilities::JobAbility,
    power_ups::PowerUps,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);
const FIRST: u64 = 100;
const PICKUP_BASE: u64 = 0x5057_5550_0000_0000;
const SHIPPED: &str = include_str!("../data/content/power_ups.json");
const ONE_AT_A_TIME: &str = "You can have one power up active at a time.";

/// One synthetic field enemy per table, all at the origin with 1 health.
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Fixture {
    fn new(tables: &[Option<&str>]) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "w158-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = json!({"zone_id":0,"name":"SyntheticWorld","sky":"","geometry_id":214,"position":[0,0,0],"rotation":[0,0,0],"tutorial":false});
        let mut cave = zone.clone();
        cave["zone_id"] = json!(411);
        fs::write(
            dir.join("encounters.json"),
            json!({"schema_version":1,"template_file":"npc.bin","template_sha256":format!("{:x}",Sha256::digest(template)),"world":zone,"world_enemies":[],"dungeons":[{"id":411,"title_id":12,"description_id":13,"objective_id":14,"zone":cave,"guide":null,"enemies":[{"guid":200,"definition_id":401,"model_id":3,"name_id":100,"name":"Synthetic cave enemy","texture_alias":"","position":[0,0,0],"max_health":100,"damage":1,"boss":false}],"reward_coins":0,"reward_items":[]}]}).to_string(),
        )
        .unwrap();
        let enemies: Vec<Value> = tables
            .iter()
            .enumerate()
            .map(|(i, table)| {
                let mut enemy = json!({"guid":FIRST + i as u64,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic wolf","texture_alias":"","position":[0,0,0],"max_health":1,"damage":0,"boss":false,"respawn_seconds":3600,"aggro_range":8});
                if let Some(table) = table {
                    enemy["power_up_table"] = json!(table);
                }
                enemy
            })
            .collect();
        fs::write(
            dir.join("field_enemies.json"),
            json!({"schema_version":1,"enemies":enemies}).to_string(),
        )
        .unwrap();
        let mut power_ups: Value = serde_json::from_str(SHIPPED).unwrap();
        power_ups["kinds"]["energy"]["energy"] = json!(20);
        let t = |guaranteed: Value| json!({"guaranteed":guaranteed,"source":"test table"});
        for (id, kinds) in [
            ("t_small", json!(["health_small"])),
            ("t_energy", json!(["energy"])),
            ("t_flame", json!(["flame_wave"])),
            ("t_shield", json!(["super_shield"])),
            (
                "t_triple",
                json!(["health_big", "health_big", "health_big"]),
            ),
        ] {
            power_ups["tables"][id] = t(kinds);
        }
        power_ups["tables"]["t_none"] =
            json!({"health_roll":{"chance":0.0,"pool":{"health_small":1}},"source":"test table"});
        fs::write(dir.join("power_ups.json"), power_ups.to_string()).unwrap();
        Self(dir)
    }
    fn path(&self) -> PathBuf {
        self.0.join("power_ups.json")
    }
    fn edit_power_ups(&self, edit: impl FnOnce(&mut Value)) {
        let mut file: Value = serde_json::from_slice(&fs::read(self.path()).unwrap()).unwrap();
        edit(&mut file);
        fs::write(self.path(), file.to_string()).unwrap();
    }
    fn load(&self) -> EncounterEngine {
        self.engine(None)
    }
    /// The player starts at `health` of 1000.
    fn load_hurt(&self, health: u32) -> EncounterEngine {
        let mut state = self.engine(None).state().clone();
        state.player_health = health;
        self.engine(Some(state))
    }
    fn engine(&self, state: Option<osfr_tools::encounter::EncounterState>) -> EncounterEngine {
        let mut e = EncounterEngine::load(&self.0, state).unwrap();
        e.load_field_enemies(&self.0.join("field_enemies.json"))
            .unwrap();
        e.load_power_ups(&self.path()).unwrap();
        e
    }
}

fn position(e: &EncounterEngine, guid: u64) -> [f32; 3] {
    e.field_teleport_targets()
        .into_iter()
        .find(|a| a.guid == guid)
        .unwrap()
        .position
}
fn kill(e: &mut EncounterEngine, guid: u64, now: u64) -> EncounterOutcome {
    let at = position(e, guid);
    let out = e.attack(guid, at, 11, 0, now).unwrap();
    assert!(removed(&out).contains(&guid), "the hit must kill");
    out
}
/// (guid, model id) of every AddNpc except the enemy's own removal.
fn spawned(out: &EncounterOutcome) -> Vec<(u64, i32)> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .map(|p| {
            (
                u64::from_le_bytes(p[4..12].try_into().unwrap()),
                i32::from_le_bytes(p[16..20].try_into().unwrap()),
            )
        })
        .collect()
}
fn removed(out: &EncounterOutcome) -> Vec<u64> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 3, 0]))
        .map(|p| u64::from_le_bytes(p[6..14].try_into().unwrap()))
        .collect()
}
fn pickups(out: &EncounterOutcome) -> Vec<u64> {
    spawned(out)
        .into_iter()
        .map(|(guid, _)| guid)
        .filter(|g| g & 0xffff_ffff_0000_0000 == PICKUP_BASE)
        .collect()
}
fn health(out: &EncounterOutcome) -> Option<(i32, i32)> {
    out.replies
        .iter()
        .rfind(|p| p.starts_with(&[38, 0, 1, 0]))
        .map(|p| {
            (
                i32::from_le_bytes(p[4..8].try_into().unwrap()),
                i32::from_le_bytes(p[8..12].try_into().unwrap()),
            )
        })
}
fn heal_numbers(out: &EncounterOutcome) -> Vec<i32> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 35, 0]))
        .map(|p| i32::from_le_bytes(p[21..25].try_into().unwrap()))
        .collect()
}
fn energy(out: &EncounterOutcome) -> Option<i32> {
    out.replies
        .iter()
        .rfind(|p| p.starts_with(&[38, 0, 13, 0]))
        .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
}
const FAR: [f32; 3] = [900.0, 0.0, 900.0];
/// Where every fixture enemy stands (and so where its pickups land).
const SPOT: [f32; 3] = [0.0; 3];

#[test]
fn kill_spawns_a_pickup_actor_with_the_kind_model_at_the_kill() {
    // "boss" is the shipped table: it guarantees one big heart (model 736).
    let f = Fixture::new(&[Some("boss")]);
    let mut e = f.load();
    let out = kill(&mut e, FIRST, 1_000);
    let drops = spawned(&out);
    assert_eq!(drops.len(), 1);
    assert_eq!(drops[0].1, 736);
    assert_eq!(drops[0].0 & 0xffff_ffff_0000_0000, PICKUP_BASE);
    assert_ne!(drops[0].0, FIRST);
    let packet = out
        .replies
        .iter()
        .find(|p| p.starts_with(&[35, 0, 2, 0]))
        .unwrap();
    assert_eq!(
        f32::from_le_bytes(packet[37..41].try_into().unwrap()),
        SPOT[0]
    );
    assert_eq!(
        f32::from_le_bytes(packet[45..49].try_into().unwrap()),
        SPOT[2]
    );
}

#[test]
fn kinds_use_their_own_models() {
    let f = Fixture::new(&[Some("t_flame"), Some("t_shield"), Some("t_energy")]);
    let mut e = f.load();
    let models: Vec<i32> = (0..3)
        .map(|i| spawned(&kill(&mut e, FIRST + i, 1_000 + i * 700))[0].1)
        .collect();
    assert_eq!(models, [1949, 1951, 737]);
}

#[test]
fn enemies_without_a_usable_table_drop_nothing() {
    let f = Fixture::new(&[None, Some("no_such_table"), Some("t_none")]);
    let mut e = f.load();
    for i in 0..3 {
        let out = kill(&mut e, FIRST + i, 1_000 + i * 700);
        assert!(spawned(&out).is_empty(), "enemy {i}");
    }
    // And with no power_ups.json loaded at all (the gateway's state today).
    let mut bare = EncounterEngine::load(&f.0, None).unwrap();
    bare.load_field_enemies(&f.0.join("field_enemies.json"))
        .unwrap();
    assert!(spawned(&kill(&mut bare, FIRST, 1_000)).is_empty());
}

#[test]
fn an_unloaded_engine_drops_nothing() {
    let f = Fixture::new(&[Some("boss")]);
    let mut e = f.load();
    e.unload_power_ups();
    assert!(spawned(&kill(&mut e, FIRST, 1_000)).is_empty());
}

#[test]
fn walking_onto_the_big_heart_heals_from_the_server_position_and_asks_for_the_group() {
    let f = Fixture::new(&[Some("boss")]);
    let mut e = f.load_hurt(400);
    let pickup = pickups(&kill(&mut e, FIRST, 1_000))[0];
    // A player far away collects nothing, whatever the client might claim.
    let far = e.tick(FAR, 11, 1_200).unwrap();
    assert!(removed(&far).is_empty() && health(&far).is_none());
    let near = e.tick(SPOT, 11, 1_400).unwrap();
    assert_eq!(removed(&near), [pickup]);
    assert_eq!(health(&near), Some((900, 1000)), "50% of max health");
    assert_eq!(heal_numbers(&near), [500]);
    assert_eq!(near.group_heal_percent, Some(50));
    assert!(near.changed, "health is saved, so the outcome is changed");
    assert_eq!(e.player_health().0, 900);
    let again = e.tick(SPOT, 11, 1_600).unwrap();
    assert!(removed(&again).is_empty() && again.group_heal_percent.is_none());
}

#[test]
fn the_small_heart_heals_only_the_player() {
    let f = Fixture::new(&[Some("t_small")]);
    let mut e = f.load_hurt(400);
    kill(&mut e, FIRST, 1_000);
    let out = e.tick(SPOT, 11, 1_200).unwrap();
    assert_eq!(health(&out), Some((550, 1000)), "15% of max health");
    assert!(out.group_heal_percent.is_none());
}

#[test]
fn energy_pickup_adds_energy_up_to_the_bar() {
    let f = Fixture::new(&[Some("t_energy")]);
    let mut e = f.load();
    let spend: JobAbility = serde_json::from_value(json!({"slot":1,"definition_id":62000,"name_id":1,"icon_id":1,"min_level":1,"cooldown_ms":5000,"energy_cost":60,"heal":1,"effects":[]})).unwrap();
    let cast = e.job_ability(&spend, 0, [0.0; 3], 11, 0).unwrap();
    assert_eq!(energy(&cast), Some(40));
    kill(&mut e, FIRST, 50);
    let out = e.tick(SPOT, 11, 200).unwrap();
    assert_eq!(
        energy(&out),
        Some(60),
        "the test file's energy kind gives 20"
    );
}

#[test]
fn only_one_stored_kind_is_held_and_a_refused_pickup_stays_on_the_ground() {
    let f = Fixture::new(&[Some("t_flame"), Some("t_shield"), Some("t_small")]);
    let mut e = f.load_hurt(400);
    let first = pickups(&kill(&mut e, FIRST, 1_000))[0];
    let taken = e.tick(SPOT, 11, 1_200).unwrap();
    assert_eq!(removed(&taken), [first]);
    assert!(taken.feedback.is_none());
    assert_eq!(e.held_power_up(), Some("flame_wave"));

    let second = pickups(&kill(&mut e, FIRST + 1, 2_000))[0];
    let refused = e.tick(SPOT, 11, 2_200).unwrap();
    assert!(!removed(&refused).contains(&second), "the pickup stays");
    assert_eq!(refused.feedback.as_deref(), Some(ONE_AT_A_TIME));
    assert_eq!(e.held_power_up(), Some("flame_wave"));
    // Told once, not every tick.
    let quiet = e.tick(SPOT, 11, 2_400).unwrap();
    assert!(quiet.feedback.is_none() && !removed(&quiet).contains(&second));

    // Automatic kinds never need the slot, even while one is held.
    kill(&mut e, FIRST + 2, 3_000);
    let heart = e.tick(SPOT, 11, 3_200).unwrap();
    assert_eq!(health(&heart), Some((550, 1000)));
    assert_eq!(e.held_power_up(), Some("flame_wave"));
}

#[test]
fn unclaimed_pickups_despawn_after_the_ttl_and_cannot_be_taken_late() {
    let f = Fixture::new(&[Some("boss")]);
    let ttl = u64::from(PowerUps::load(f.path()).unwrap().pickup.ttl_ms);
    let mut e = f.load_hurt(400);
    let pickup = pickups(&kill(&mut e, FIRST, 1_000))[0];
    let early = e.tick(FAR, 11, 1_000 + ttl - 1).unwrap();
    assert!(removed(&early).is_empty());
    // On the SPOT at the deadline: the despawn wins, nothing is healed.
    let late = e.tick(SPOT, 11, 1_000 + ttl).unwrap();
    assert_eq!(removed(&late), [pickup]);
    assert!(health(&late).is_none() && late.group_heal_percent.is_none());
    let after = e.tick(SPOT, 11, 1_000 + ttl + 200).unwrap();
    assert!(removed(&after).is_empty());
}

#[test]
fn a_refused_pickup_also_despawns_on_its_ttl() {
    let f = Fixture::new(&[Some("t_flame"), Some("t_shield")]);
    let ttl = u64::from(PowerUps::load(f.path()).unwrap().pickup.ttl_ms);
    let mut e = f.load();
    kill(&mut e, FIRST, 1_000);
    e.tick(SPOT, 11, 1_200).unwrap();
    let second = pickups(&kill(&mut e, FIRST + 1, 2_000))[0];
    e.tick(SPOT, 11, 2_200).unwrap();
    let gone = e.tick(FAR, 11, 2_000 + ttl).unwrap();
    assert_eq!(removed(&gone), [second]);
}

#[test]
fn the_ground_holds_at_most_max_alive_pickups() {
    let f = Fixture::new(&[Some("t_triple")]);
    f.edit_power_ups(|file| file["pickup"]["max_alive_per_instance"] = json!(2));
    let mut e = f.load();
    assert_eq!(pickups(&kill(&mut e, FIRST, 1_000)).len(), 2);
}

#[test]
fn zoning_clears_pickups_but_not_the_held_power_up() {
    let f = Fixture::new(&[Some("t_flame"), Some("t_shield")]);
    let mut e = f.load();
    kill(&mut e, FIRST, 1_000);
    e.tick(SPOT, 11, 1_200).unwrap();
    kill(&mut e, FIRST + 1, 2_000);
    e.reset_session();
    let out = e.tick(SPOT, 11, 2_200).unwrap();
    assert!(removed(&out).is_empty() && out.feedback.is_none());
    assert_eq!(e.held_power_up(), Some("flame_wave"));
}

#[test]
fn a_bad_reload_keeps_the_previous_tables() {
    let f = Fixture::new(&[Some("boss")]);
    let mut e = f.load();
    fs::write(f.path(), b"{not json").unwrap();
    assert!(e.load_power_ups(&f.path()).is_err());
    assert_eq!(spawned(&kill(&mut e, FIRST, 1_000)).len(), 1);
}

#[test]
fn shipped_enemy_content_only_names_shipped_tables() {
    let tables: Value = serde_json::from_str(SHIPPED).unwrap();
    let mut seen = 0;
    for file in ["field_enemies.json", "field_battles.json"] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/content")
            .join(file);
        let mut stack = vec![serde_json::from_slice::<Value>(&fs::read(path).unwrap()).unwrap()];
        while let Some(value) = stack.pop() {
            match value {
                Value::Object(map) => {
                    if let Some(table) = map.get("power_up_table") {
                        let id = table.as_str().expect("table id is a string");
                        assert!(tables["tables"].get(id).is_some(), "{file}: {id}");
                        seen += 1;
                    }
                    stack.extend(map.into_iter().map(|(_, v)| v));
                }
                Value::Array(items) => stack.extend(items),
                _ => {}
            }
        }
    }
    assert!(seen > 0, "no shipped enemy drops anything");
}
