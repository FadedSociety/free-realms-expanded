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
        Self::sized(tables, 1)
    }
    fn sized(tables: &[Option<&str>], health: u32) -> Self {
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
                let mut enemy = json!({"guid":FIRST + i as u64,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic wolf","texture_alias":"","position":[0,0,0],"max_health":health,"damage":0,"boss":false,"respawn_seconds":3600,"aggro_range":8});
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
        power_ups["tables"]["t_roll"] =
            json!({"health_roll":{"chance":0.5,"pool":{"health_small":1}},"source":"test table"});
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
    assert_eq!(near.group_heal.map(|g| g.percent), Some(50));
    assert!(near.changed, "health is saved, so the outcome is changed");
    assert_eq!(e.player_health().0, 900);
    let again = e.tick(SPOT, 11, 1_600).unwrap();
    assert!(removed(&again).is_empty() && again.group_heal.is_none());
}

#[test]
fn the_small_heart_heals_only_the_player() {
    let f = Fixture::new(&[Some("t_small")]);
    let mut e = f.load_hurt(400);
    kill(&mut e, FIRST, 1_000);
    let out = e.tick(SPOT, 11, 1_200).unwrap();
    assert_eq!(health(&out), Some((550, 1000)), "15% of max health");
    assert!(out.group_heal.is_none());
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
    assert!(health(&late).is_none() && late.group_heal.is_none());
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
fn an_instance_bind_discards_ground_pickups_but_not_the_held_power_up() {
    let f = Fixture::new(&[Some("t_flame"), Some("boss")]);
    let mut e = f.load_hurt(400);
    e.enter_zone(0);
    // A first drop is picked up; the held kind is player state.
    let first = pickups(&kill(&mut e, FIRST, 1_000))[0];
    let taken = e.tick(SPOT, 11, 1_200).unwrap();
    assert_eq!(removed(&taken), [first]);
    assert_eq!(e.held_power_up(), Some("flame_wave"));
    // A second drop is still on the ground when the session binds another
    // membership instance: the client reloaded the scene, so it is gone.
    kill(&mut e, FIRST + 1, 2_000);
    assert_eq!(e.pickup_count(), 1);
    assert!(
        !e.bind_run_instance(0, false).unwrap(),
        "the same instance keeps the drop"
    );
    assert_eq!(e.pickup_count(), 1);
    e.bind_run_instance(1, false).unwrap();
    assert_eq!(e.pickup_count(), 0, "a new instance is a scene reload");
    assert_eq!(
        e.held_power_up(),
        Some("flame_wave"),
        "the held power-up is player state, not scene state"
    );
    let after = e.tick(SPOT, 11, 2_200).unwrap();
    assert!(
        removed(&after).is_empty() && health(&after).is_none(),
        "the old instance's drop must not pay out after the reload"
    );
}

#[test]
fn a_revive_or_same_instance_transfer_reload_discards_ground_pickups() {
    // A free revive at the safe location and a host-authorized transfer to the
    // current instance both send opcode 31 (reader 0x008e4ae0) with the same
    // world instance: the client reloads its scene, so the old drop is gone.
    let f = Fixture::new(&[Some("boss")]);
    let mut e = f.load_hurt(400);
    e.enter_zone(0);
    kill(&mut e, FIRST, 1_000);
    assert_eq!(e.pickup_count(), 1);
    assert!(
        !e.bind_run_instance(0, false).unwrap(),
        "an ordinary commit is not a scene reload"
    );
    assert_eq!(e.pickup_count(), 1);
    e.bind_run_instance(0, true).unwrap();
    assert_eq!(
        e.pickup_count(),
        0,
        "a begin-zoning reload clears the old scene's pickups"
    );
    let after = e.tick(SPOT, 11, 1_200).unwrap();
    assert!(
        removed(&after).is_empty() && health(&after).is_none(),
        "the stale big heart must not heal after the reload"
    );
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

// ---- Slice 2b: shared fields, one roll and one pickup per participant ----

/// Packets the gateway broadcasts to everyone near the field (osfr-gateway
/// `field_packet`); a personal pickup must never look like one of these.
fn broadcast_to_field(p: &[u8]) -> bool {
    p.starts_with(&[125, 0])
        || p.get(..4).is_some_and(|h| {
            matches!(
                h,
                [35, 0, 5, 0] | [35, 0, 8, 0] | [35, 0, 16, 0] | [32, 0, 7, 0]
            )
        })
}
const A: u64 = 101;
const B: u64 = 202;
const C: u64 = 303;
const NEAR: [f32; 3] = [1.0, 0.0, 0.0];

struct Shared {
    a: EncounterEngine,
    b: EncounterEngine,
    c: EncounterEngine,
    field: osfr_tools::encounter::SharedFieldState,
}
fn shared(f: &Fixture) -> Shared {
    let mut a = f.load();
    let mut b = f.load();
    let mut c = f.load();
    let field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    c.take_shared_fields();
    Shared { a, b, c, field }
}
fn hit(
    e: &mut EncounterEngine,
    field: &mut osfr_tools::encounter::SharedFieldState,
    who: u64,
    enemy: u64,
    now: u64,
) -> EncounterOutcome {
    e.shared_attack(field, who, enemy, NEAR, 2, now).unwrap()
}

#[test]
fn every_participant_gets_their_own_pickup_and_nobody_sees_anothers() {
    let f = Fixture::sized(&[Some("boss")], 150);
    let mut s = shared(&f);
    let first = hit(&mut s.a, &mut s.field, A, FIRST, 0);
    assert!(spawned(&first).is_empty(), "no kill yet");
    let kill = hit(&mut s.b, &mut s.field, B, FIRST, 100);
    assert_eq!(s.field.health(FIRST), Some(0), "B lands the kill");
    // B (the killer) has the one pickup at once; A has nothing until A's own tick.
    let b_drops = pickups(&kill);
    assert_eq!(b_drops.len(), 1);
    let a_claim = s.a.shared_effects(&mut s.field, A, 200).unwrap();
    let a_drops = pickups(&a_claim);
    assert_eq!(a_drops.len(), 1, "A rolls on A's own engine");
    assert_eq!(spawned(&a_claim)[0].1, 736);
    // Claimed once; nobody earns a second pickup from the same kill.
    for (engine, who) in [(&mut s.a, A), (&mut s.b, B)] {
        let again = engine.shared_effects(&mut s.field, who, 300).unwrap();
        assert!(spawned(&again).is_empty());
    }
    // Personal: a pickup packet is never a field broadcast, and each engine
    // collects only the pickup it spawned itself (the server tracks it).
    for out in [&kill, &a_claim] {
        assert!(
            out.replies
                .iter()
                .filter(|p| p.starts_with(&[35, 0, 2, 0]))
                .all(|p| pickups_in(p) && !broadcast_to_field(p))
        );
    }
    let a_take = s.a.tick(SPOT, 2, 400).unwrap();
    let b_take = s.b.tick(SPOT, 2, 400).unwrap();
    assert_eq!(removed(&a_take), a_drops);
    assert_eq!(removed(&b_take), b_drops);
    // Canary: a third engine that never saw either kill cannot collect anything.
    assert!(removed(&s.c.tick(SPOT, 2, 400).unwrap()).is_empty());
}
fn pickups_in(p: &[u8]) -> bool {
    u64::from_le_bytes(p[4..12].try_into().unwrap()) & 0xffff_ffff_0000_0000 == PICKUP_BASE
}

#[test]
fn a_bystander_who_never_hit_the_enemy_gets_nothing() {
    let f = Fixture::sized(&[Some("boss")], 150);
    let mut s = shared(&f);
    hit(&mut s.a, &mut s.field, A, FIRST, 0);
    hit(&mut s.b, &mut s.field, B, FIRST, 100);
    assert_eq!(s.field.health(FIRST), Some(0));
    let out = s.c.shared_effects(&mut s.field, C, 200).unwrap();
    assert!(spawned(&out).is_empty());
    assert!(removed(&s.c.tick(SPOT, 2, 300).unwrap()).is_empty());
}

#[test]
fn only_recent_damage_counts_as_taking_part() {
    let f = Fixture::sized(&[Some("boss")], 150);
    let mut s = shared(&f);
    hit(&mut s.a, &mut s.field, A, FIRST, 0);
    // The shipped window is 30 s; B's kill lands at 31 s.
    let kill = hit(&mut s.b, &mut s.field, B, FIRST, 31_000);
    assert_eq!(s.field.health(FIRST), Some(0));
    assert_eq!(pickups(&kill).len(), 1, "the killer always takes part");
    let out = s.a.shared_effects(&mut s.field, A, 31_100).unwrap();
    assert!(spawned(&out).is_empty(), "A hit too long ago");
}

#[test]
fn a_participant_who_left_the_field_loses_the_drop() {
    let f = Fixture::sized(&[Some("boss")], 150);
    let mut s = shared(&f);
    hit(&mut s.a, &mut s.field, A, FIRST, 0);
    hit(&mut s.b, &mut s.field, B, FIRST, 100);
    s.field
        .retain_casters(&std::collections::BTreeSet::from([B]), 200);
    assert!(
        spawned(&s.a.shared_effects(&mut s.field, A, 200).unwrap()).is_empty(),
        "A is no longer in the instance"
    );
}

#[test]
fn an_unclaimed_drop_lapses() {
    let f = Fixture::sized(&[Some("boss")], 150);
    let mut s = shared(&f);
    hit(&mut s.a, &mut s.field, A, FIRST, 0);
    hit(&mut s.b, &mut s.field, B, FIRST, 100);
    let late = s.a.shared_effects(&mut s.field, A, 100 + 10_000).unwrap();
    assert!(spawned(&late).is_empty());
}

#[test]
fn the_group_multiplier_thins_shared_rolls_but_not_solo_ones_or_guaranteed_ones() {
    const N: usize = 60;
    let tables = vec![Some("t_roll"); N];
    let f = Fixture::sized(&tables, 150);
    f.edit_power_ups(|file| {
        file["participation"]["group_chance_multiplier"] = json!(0.05);
    });
    // Solo kills (no shared field): the full 50% chance per kill.
    let mut solo = f.load();
    let mut solo_drops = 0;
    for i in 0..N as u64 {
        let at = SPOT;
        let first = solo.attack(FIRST + i, at, 2, 0, i * 1_400).unwrap();
        let second = solo.attack(FIRST + i, at, 2, 0, i * 1_400 + 700).unwrap();
        solo_drops += pickups(&first).len() + pickups(&second).len();
    }
    assert!(
        solo_drops >= 12,
        "solo rolls keep their chance: {solo_drops}"
    );
    // Two players on every enemy: each roll is scaled to 2.5%.
    let mut s = shared(&f);
    let mut shared_drops = 0;
    for i in 0..N as u64 {
        hit(&mut s.a, &mut s.field, A, FIRST + i, i * 1_400);
        shared_drops += pickups(&hit(&mut s.b, &mut s.field, B, FIRST + i, i * 1_400 + 700)).len();
        shared_drops += pickups(
            &s.a.shared_effects(&mut s.field, A, i * 1_400 + 800)
                .unwrap(),
        )
        .len();
    }
    assert!(
        shared_drops <= 12,
        "two participants thin the chance: {shared_drops} of 120 rolls"
    );
    // Guaranteed drops are not scaled.
    let g = Fixture::sized(&[Some("boss")], 150);
    g.edit_power_ups(|file| file["participation"]["group_chance_multiplier"] = json!(0.05));
    let mut s = shared(&g);
    hit(&mut s.a, &mut s.field, A, FIRST, 0);
    assert_eq!(
        pickups(&hit(&mut s.b, &mut s.field, B, FIRST, 100)).len(),
        1
    );
    assert_eq!(
        pickups(&s.a.shared_effects(&mut s.field, A, 200).unwrap()).len(),
        1
    );
}

#[test]
fn participation_settings_are_bounded() {
    let parse = |edit: &dyn Fn(&mut Value)| {
        let mut file: Value = serde_json::from_str(SHIPPED).unwrap();
        edit(&mut file);
        PowerUps::from_json(file.to_string().as_bytes())
    };
    assert!(parse(&|_| {}).is_ok());
    for (key, value) in [
        ("window_ms", json!(999)),
        ("window_ms", json!(300_001)),
        ("group_chance_multiplier", json!(0.04)),
        ("group_chance_multiplier", json!(1.01)),
        ("source", json!("  ")),
    ] {
        assert!(
            parse(&|f| f["participation"][key] = value.clone()).is_err(),
            "{key} {value}"
        );
    }
    // The block is optional: defaults are a 30 s window and no thinning.
    let bare = parse(&|f| {
        f.as_object_mut().unwrap().remove("participation");
    })
    .unwrap();
    assert_eq!(bare.participation_window_ms(), 30_000);
    assert_eq!(bare.group_chance_multiplier(), 1.0);
}

#[test]
fn roll_scaled_scales_chances_only() {
    use osfr_tools::power_ups::{Rng, roll_scaled};
    let p = PowerUps::from_json(SHIPPED.as_bytes()).unwrap();
    let table = p.table("standard").unwrap();
    let count = |scale: f64| {
        let mut rng = Rng::seeded(7);
        (0..20_000)
            .map(|_| roll_scaled(table, &mut rng, scale).len())
            .sum::<usize>()
    };
    assert_eq!(count(0.0), 0);
    assert!(count(0.1) * 5 < count(1.0));
    let boss = p.table("boss").unwrap();
    assert_eq!(roll_scaled(boss, &mut Rng::seeded(1), 0.0), ["health_big"]);
}

#[test]
fn pickups_do_not_follow_the_player_into_another_zone() {
    let f = Fixture::new(&[Some("boss")]);
    let mut e = f.load();
    e.enter_zone(0);
    kill(&mut e, FIRST, 1_000);
    e.enter_zone(0); // same zone: still there
    assert_eq!(removed(&e.tick(SPOT, 2, 1_200).unwrap()).len(), 1);
    let mut e = f.load();
    e.enter_zone(0);
    kill(&mut e, FIRST, 1_000);
    e.enter_zone(411);
    e.enter_zone(0);
    assert!(
        removed(&e.tick(SPOT, 2, 1_200).unwrap()).is_empty(),
        "a pickup left behind in another zone never comes back"
    );
}

#[test]
fn big_hearts_from_one_kill_share_a_kill_key_so_the_group_heals_once() {
    let f = Fixture::sized(&[Some("boss"), Some("boss")], 150);
    let mut s = shared(&f);
    hit(&mut s.a, &mut s.field, A, FIRST, 0);
    hit(&mut s.b, &mut s.field, B, FIRST, 100);
    s.a.shared_effects(&mut s.field, A, 200).unwrap();
    let a = s.a.tick(SPOT, 2, 300).unwrap().group_heal.unwrap();
    let b = s.b.tick(SPOT, 2, 300).unwrap().group_heal.unwrap();
    assert_eq!((a.percent, a.kill), (50, b.kill), "one kill, one key");
    assert_eq!(a.kill.0, FIRST);
    // A second kill is a different key (so it may heal the group again).
    hit(&mut s.a, &mut s.field, A, FIRST + 1, 1_000);
    hit(&mut s.b, &mut s.field, B, FIRST + 1, 1_100);
    let next = s.b.tick(SPOT, 2, 1_200).unwrap().group_heal.unwrap();
    assert_ne!(next.kill, a.kill);
}

#[test]
fn a_group_heal_restores_a_hurt_member_and_is_silent_at_full_health() {
    let f = Fixture::new(&[]);
    let mut hurt = f.load_hurt(400);
    let out = hurt.receive_group_heal(50);
    assert_eq!(health(&out), Some((900, 1000)));
    assert_eq!(heal_numbers(&out), [500]);
    assert!(out.changed);
    let again = hurt.receive_group_heal(50);
    assert_eq!(health(&again), Some((1000, 1000)));
    let full = hurt.receive_group_heal(50);
    assert!(full.replies.is_empty() && !full.changed);
}
