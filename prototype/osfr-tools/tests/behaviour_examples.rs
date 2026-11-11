//! The twelve real enemies of the combat design (section 9) as the shipped presets of
//! `data/content/behaviours.json`, each played through `brain::step` against a scripted
//! fight, then the same language through the engine's real entry points: a summoner battle
//! (begin, zone ready, attack, tick) and an open-world enemy (tick).
use osfr_tools::{
    character_store::CharacterState,
    encounter::{
        EncounterEngine, EncounterOutcome, Mode,
        brain::{self, Act, AttackKind, Memory, Move, Place, Program, TargetRule, View, Who},
        wire::How,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn shipped() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn presets() -> BTreeMap<String, Program> {
    brain::presets(&shipped().join("behaviours.json")).unwrap()
}

/// One enemy in a scripted fight: the test sets what it sees and steps the clock.
struct Script {
    program: Program,
    memory: Memory,
    health_pct: u32,
    target_distance: Option<f32>,
    nearest_player: Option<f32>,
    tags: BTreeMap<String, u32>,
}

impl Script {
    fn new(name: &str) -> Self {
        Self {
            program: presets().remove(name).unwrap_or_else(|| panic!("no preset {name}")),
            memory: Memory::seeded(1),
            health_pct: 100,
            target_distance: Some(6.0),
            nearest_player: Some(6.0),
            tags: BTreeMap::new(),
        }
    }

    fn engaged(mut self, now_ms: u64) -> Self {
        self.memory.engaged(now_ms);
        self
    }

    fn at(&mut self, now_ms: u64) -> Vec<Act> {
        let view = View {
            now_ms,
            health_pct: self.health_pct,
            position: [0.0; 3],
            home: [0.0; 3],
            target: Some(1),
            target_distance: self.target_distance,
            nearest_player: self.nearest_player,
            tags_alive: &self.tags,
            signals: &[],
            phase: self.memory.phase(),
            night: false,
        };
        let mut out = Vec::new();
        brain::step(&self.program, &mut self.memory, &view, &mut out);
        out
    }

    /// Every act between two times, at the 250 ms decision cadence, with its time.
    fn until(&mut self, from_ms: u64, to_ms: u64) -> Vec<(u64, Act)> {
        let mut all = Vec::new();
        for now in (from_ms..=to_ms).step_by(250) {
            all.extend(self.at(now).into_iter().map(|act| (now, act)));
        }
        all
    }

    fn stance(&self) -> (Move, Option<String>) {
        let stance = self.memory.stance(&self.program);
        (stance.movement.clone(), stance.attack.clone())
    }
}

fn summon(kind: &str, count: u8, at: Place, radius: f32, tag: &str, mound: bool) -> Act {
    Act::Summon {
        kind: kind.into(),
        count,
        at,
        radius,
        tag: tag.into(),
        mound,
    }
}

fn said(act: &Act) -> (&str, How) {
    match act {
        Act::Say(line) => (line.text.as_deref().unwrap_or_default(), line.how),
        other => panic!("not a line: {other:?}"),
    }
}

/// 1. The owner's example: keeps away, is elsewhere after two hits, raises helpers from
/// mounds on a ladder, repeats, and never attacks.
#[test]
fn troll_summoner_slips_away_after_two_hits_and_raises_helpers_on_its_ladder() {
    let mut s = Script::new("troll_summoner");
    assert_eq!(s.stance(), (Move::Hold, None), "it stands and has no attack at all");
    // Helpers are out from the start, before anyone swings.
    assert_eq!(s.at(0), [summon("troll_scoutling", 2, Place::Own, 4.0, "helper", true)]);
    s.memory.engaged(1000);
    s.memory.hit();
    assert!(s.at(1000).is_empty(), "one hit: it stays");
    s.memory.hit();
    assert_eq!(
        s.at(1700),
        [Act::Relocate {
            to: Place::Away,
            min: 8.0,
            max: 18.0,
            effect: Some("smoke".into()),
        }]
    );
    // Three seconds after the slip, one more helper where it now stands.
    assert_eq!(
        s.until(1750, 5000),
        [(4750, summon("troll_scoutling", 1, Place::Own, 3.0, "helper", true))]
    );
    // And again: two more hits once the four seconds are over.
    s.memory.hit();
    s.memory.hit();
    assert!(matches!(s.at(5750).as_slice(), [Act::Relocate { .. }]));
    // The ladder: Ragelings at half and at a quarter, each rung once.
    s.health_pct = 49;
    assert_eq!(s.at(6000), [summon("troll_rageling", 1, Place::Own, 3.0, "helper", true)]);
    s.health_pct = 24;
    assert_eq!(s.at(6250), [summon("troll_rageling", 2, Place::Target, 3.0, "helper", true)]);
    s.health_pct = 60;
    s.at(6500);
    s.health_pct = 10;
    let rest = s.until(6750, 30_000);
    assert!(
        rest.iter().all(|(_, act)| matches!(act, Act::Summon { count: 1, .. })),
        "only the slip's late helper is left: {rest:?}"
    );
    assert_eq!(s.stance(), (Move::Hold, None), "it never takes up an attack");
    // The older five-kind ladder ships too, for a fight that has those actors.
    let old = presets().remove("troll_summoner_2009").unwrap();
    let ids: Vec<&str> = old.rules.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ["start", "slip", "scout", "archer", "rage", "guards"]);
}

/// 2. Flee, heal, return: twice, and never a third time.
#[test]
fn robgoblin_adept_blinks_and_heals_twice() {
    let mut s = Script::new("robgoblin_adept").engaged(0);
    assert_eq!(
        s.stance(),
        (Move::KeepAway { min: 8.0, max: 14.0 }, Some("adept_firebolt".into()))
    );
    assert_eq!(s.program.attack("adept_firebolt").unwrap().kind, AttackKind::Lob);
    s.target_distance = Some(10.0);
    s.health_pct = 44;
    let blink = |min, max| Act::Relocate {
        to: Place::Away,
        min,
        max,
        effect: Some("flash".into()),
    };
    assert_eq!(
        s.until(250, 2000),
        [
            (250, blink(20.0, 30.0)),
            (1750, Act::Heal { who: Who::Own, pct: 50 })
        ]
    );
    s.health_pct = 94;
    s.at(2250);
    s.health_pct = 21;
    assert_eq!(
        s.until(2500, 4250),
        [
            (2500, blink(20.0, 30.0)),
            (4000, Act::Heal { who: Who::Own, pct: 48 })
        ]
    );
    // Healed and beaten down a third time: no third escape.
    s.health_pct = 69;
    s.at(4500);
    s.health_pct = 5;
    assert!(s.until(4750, 8750).is_empty());
    // It hops away on its timer only when the player has closed in.
    assert!(s.at(9000).is_empty(), "the 9 s timer rang with the player at 10 m");
    s.target_distance = Some(4.0);
    assert_eq!(s.until(9250, 18_250), [(18_000, blink(10.0, 16.0))]);
}

/// 3. Open-world notice, chase, give up: the field roamer with the captured bite.
#[test]
fn bixie_soldier_roams_until_it_has_a_target_then_chases_and_bites() {
    let mut s = Script::new("bixie_soldier");
    assert_eq!(s.stance(), (Move::Roam(4.0), Some("bite".into())));
    let bite = s.program.attack("bite").unwrap();
    assert_eq!((bite.interval_ms, bite.effect), (2300, 5409));
    assert_eq!(bite.special.as_ref().map(|x| (x.hit, x.effect)), Some((1.5, 5622)));
    s.nearest_player = Some(30.0);
    assert!(s.at(0).is_empty());
    // A player inside 8 m, or a neighbour 6 m away getting a target, ends the roaming.
    s.nearest_player = Some(7.5);
    assert!(matches!(s.at(250).as_slice(), [Act::Stance(x)] if x.movement == Move::Chase));
    let mut other = Script::new("bixie_soldier");
    other.nearest_player = Some(30.0);
    other.memory.ally_engaged(5.0, &[]);
    assert!(matches!(other.at(0).as_slice(), [Act::Stance(x)] if x.movement == Move::Chase));
    s.memory.engaged(500);
    s.at(500);
    assert_eq!(s.stance(), (Move::Chase, Some("bite".into())));
    // The plain roamer is the same without the bite.
    let roamer = presets().remove("field_roamer").unwrap();
    assert_eq!(roamer.stance.attack.as_deref(), Some("melee"));
    assert_eq!(roamer.attack("melee").unwrap().interval_ms, 1000);
}

/// 4. Rush from anywhere: 2.2 s after spawning, straight at the nearest player.
#[test]
fn frostfang_wave_wolf_rushes_the_nearest_player_after_2200_ms() {
    let mut s = Script::new("frostfang_wave_wolf");
    assert_eq!(s.stance().0, Move::Hold);
    let acts = s.until(0, 5000);
    assert_eq!(acts.len(), 3, "{acts:?}");
    assert!(acts.iter().all(|(at, _)| *at == 2250), "the first decision after 2.2 s");
    assert!(matches!(&acts[0].1, Act::Set(flags) if flags.speed == Some(6.0)));
    assert_eq!(acts[1].1, Act::Target(TargetRule::Nearest));
    assert_eq!(s.stance(), (Move::Chase, Some("bite".into())));
}

/// 5. Stand and throw.
#[test]
fn robgoblin_boomer_stands_and_lobs_at_long_range() {
    let mut s = Script::new("robgoblin_boomer");
    assert_eq!(s.stance(), (Move::Hold, Some("boomer_bomb".into())));
    let bomb = s.program.attack("boomer_bomb").unwrap();
    assert_eq!((bomb.kind, bomb.reach(), bomb.travel_ms), (AttackKind::Lob, 25.0, 900));
    assert_eq!(bomb.riders[0].status, "burn");
    s.nearest_player = Some(40.0);
    assert!(s.at(0).is_empty());
    s.nearest_player = Some(24.0);
    assert_eq!(s.at(250), [Act::Target(TargetRule::Nearest)]);
    s.nearest_player = Some(2.0);
    assert!(s.until(500, 10_000).is_empty(), "no retreat when the player closes");
    assert_eq!(s.stance().0, Move::Hold);
}

/// 6. Call for help by speech.
#[test]
fn rowdy_hooligan_shouts_and_its_pack_arrives_two_seconds_later() {
    let mut s = Script::new("rowdy_hooligan").engaged(0);
    assert_eq!(s.stance(), (Move::Chase, Some("melee".into())));
    let acts = s.until(0, 6000);
    assert_eq!(acts.len(), 2, "{acts:?}");
    assert_eq!((acts[0].0, said(&acts[0].1)), (0, ("Hey! Over here!", How::Bubble)));
    assert_eq!(acts[1], (2000, Act::Wake("hooligan_pack".into())));
}

/// 7. The announced special.
#[test]
fn drone_fauzz_announces_then_slams_three_seconds_later() {
    let mut s = Script::new("drone_fauzz").engaged(0);
    let acts = s.until(0, 41_000);
    let times: Vec<u64> = acts.iter().map(|(at, _)| *at).collect();
    assert_eq!(times, [12_000, 15_000, 37_000, 40_000]);
    assert_eq!(
        said(&acts[0].1),
        ("Drone Fauzz prepares for a big attack!", How::System)
    );
    assert_eq!(acts[1].1, Act::Attack("big_slam".into()));
    let slam = s.program.attack("big_slam").unwrap();
    assert_eq!((slam.kind, slam.radius, slam.hit), (AttackKind::Burst, 6.0, 7.0));
    assert_eq!(slam.riders[0].status, "knock_down");
}

/// 8. Phase, intermission object, second form.
#[test]
fn necronomicus_hides_behind_a_portal_and_returns_in_its_second_form() {
    let mut s = Script::new("necronomicus").engaged(0);
    assert!(
        s.until(0, 45_000).is_empty(),
        "no stun in the first phase, however long it lasts"
    );
    s.health_pct = 4;
    let acts = s.at(45_250);
    assert_eq!(acts.len(), 4, "{acts:?}");
    assert_eq!(said(&acts[0]).1, How::Red);
    assert!(matches!(&acts[1], Act::Set(f) if f.untargetable == Some(true) && f.invulnerable == Some(true)));
    assert_eq!(acts[2], summon("zombie_portal", 1, Place::Points, 0.0, "portal", false));
    assert_eq!(acts[3], Act::Phase(1));
    assert_eq!(s.memory.phase(), 1);
    // While the portal stands it only stuns on its timer.
    s.tags.insert("portal".into(), 1);
    let stuns = s.until(45_500, 70_000);
    assert!(!stuns.is_empty() && stuns.iter().all(|(_, a)| *a == Act::Attack("necro_stun".into())));
    s.tags.insert("portal".into(), 0);
    let back = s.at(70_250);
    assert_eq!(
        back,
        [Act::Become {
            definition: "necronomicus_risen".into(),
            health_pct: 100
        }]
    );
}

/// 9. Feeds on objects.
#[test]
fn grave_elemental_heals_only_while_a_bone_cluster_stands() {
    let mut s = Script::new("grave_elemental").engaged(0);
    s.tags.insert("bone_cluster".into(), 2);
    let fed = s.until(0, 12_000);
    assert_eq!(
        fed,
        [4000, 8000, 12_000].map(|at| (at, Act::Heal { who: Who::Own, pct: 6 }))
    );
    s.tags.insert("bone_cluster".into(), 0);
    let starved = s.until(12_250, 36_000);
    assert_eq!(
        starved,
        [18_000, 36_000].map(|at| (
            at,
            Act::Status {
                id: "stun".into(),
                to: Who::Target,
                ms: 3000
            }
        )),
        "the clusters are gone: only its stun is left"
    );
}

/// 10. Objects that make enemies.
#[test]
fn a_spawner_object_hatches_until_it_has_enough_out() {
    let mut s = Script::new("spawner_object").engaged(0);
    assert_eq!(s.stance(), (Move::Hold, None));
    let bat = summon("cave_bat", 1, Place::Own, 3.0, "spawn_of_this", false);
    assert_eq!(s.until(0, 10_000), [(2000, bat.clone()), (10_000, bat.clone())]);
    s.tags.insert("spawn_of_this".into(), 4);
    assert!(s.until(10_250, 40_000).is_empty(), "four out: it waits");
    s.tags.insert("spawn_of_this".into(), 3);
    assert_eq!(s.until(40_250, 48_000).len(), 1);
}

/// 11. The boss as stage host.
#[test]
fn wizard_hildegarde_hosts_two_stages_before_she_can_be_fought() {
    let mut s = Script::new("wizard_hildegarde");
    let intro = s.at(0);
    assert!(matches!(intro.as_slice(), [Act::Set(f)] if f.disposition == Some(2) && f.invulnerable == Some(true)));
    // One talk is one stage, even though both stage rules wait for a talk.
    s.memory.talked();
    let stage1 = s.at(1000);
    assert_eq!(stage1.len(), 3, "{stage1:?}");
    assert_eq!(said(&stage1[0]), ("Get them!", How::Bubble));
    assert_eq!(stage1[1..], [Act::Wake("pack1".into()), Act::Phase(1)]);
    s.tags.insert("pack1".into(), 3);
    assert!(s.until(1250, 5000).is_empty());
    s.tags.insert("pack1".into(), 0);
    assert_eq!(
        s.at(5250),
        [Act::Relocate {
            to: Place::Point("stage2".into()),
            min: 0.0,
            max: 0.0,
            effect: None
        }]
    );
    s.memory.talked();
    assert_eq!(s.at(6000), [Act::Wake("medics".into()), Act::Phase(2)]);
    s.tags.insert("medics".into(), 2);
    s.at(6250);
    s.tags.insert("medics".into(), 0);
    let fight = s.at(6500);
    assert!(matches!(&fight[0], Act::Set(f) if f.disposition == Some(0) && f.invulnerable == Some(false)));
    assert_eq!(
        s.stance(),
        (Move::KeepAway { min: 6.0, max: 12.0 }, Some("hildegarde_bolt".into()))
    );
    s.memory.talked();
    assert!(s.at(7000).is_empty(), "nothing left to say");
}

/// 12. Timed adds and a stunning heavy hit; sticks to its target.
#[test]
fn zandro_rageracer_calls_adds_on_a_timer_and_lands_a_heavy_hit() {
    let mut s = Script::new("zandro_rageracer").engaged(0);
    assert_eq!(s.stance(), (Move::Chase, Some("boss_light".into())));
    assert!(!s.memory.stance(&s.program).leash, "it follows without a leash");
    let acts = s.until(0, 60_000);
    let adds: Vec<u64> = acts
        .iter()
        .filter(|(_, a)| *a == summon("skullz_commander", 2, Place::Ring, 10.0, "adds", false))
        .map(|(at, _)| *at)
        .collect();
    assert_eq!(adds, [4000, 52_000], "whatever its health");
    let heavy: Vec<u64> = acts
        .iter()
        .filter(|(_, a)| *a == Act::Attack("boss_heavy_stun".into()))
        .map(|(at, _)| *at)
        .collect();
    assert_eq!(heavy, [10_000, 32_000, 54_000]);
    assert_eq!(said(&acts[0].1), ("Commanders, to me!", How::Red));
    let stun = &s.program.attack("boss_heavy_stun").unwrap().riders[0];
    assert_eq!((stun.status.as_str(), stun.ms), ("stun", 2500));
}

#[test]
fn every_shipped_preset_loads_and_the_old_four_keep_their_names() {
    let all = presets();
    for name in ["field_roamer", "melee", "skittish", "stationary"] {
        assert!(all.contains_key(name), "{name}");
    }
    assert_eq!(all.len(), 17, "the old four, the twelve examples and the 2009 ladder");
    assert_eq!(all["skittish"].stance.movement, Move::Flee { ms: 4000, speed: 3.5 });
    assert_eq!(all["stationary"].stance.movement, Move::Hold);
    assert_eq!(all["melee"].stance.movement, Move::Chase);
}

// ---- through the engine ----

static SERIAL: AtomicU64 = AtomicU64::new(0);
const BRAWLER: i32 = 43;
const BATTLE: u32 = 411;
const SUMMONER: u64 = 5001;
const HOME: [f32; 3] = [100.0, 0.0, 100.0];

struct Fixture {
    dir: PathBuf,
    attach: Option<&'static str>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn actor(guid: u64, name: &str, position: [f32; 3], max_health: u32, damage: u32) -> Value {
    json!({"guid": guid, "definition_id": 400 + guid as i32, "model_id": 3, "name_id": 0, "name": name,
        "texture_alias": "", "position": position, "max_health": max_health, "damage": damage, "boss": false})
}

/// Battle 96's shape: a summoner and six helpers asleep on the old 75 / 50 / 25 rows.
fn roster(summoner: Value) -> Vec<Value> {
    let mut rows = vec![summoner];
    for (i, (name, below)) in [
        ("Troll Scoutling", 75),
        ("Troll Scoutling", 75),
        ("Troll Scoutling", 50),
        ("Troll Rageling", 50),
        ("Troll Rageling", 25),
        ("Troll Rageling", 25),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = if i % 2 == 0 { -3.0 } else { 3.0 };
        let mut helper = actor(
            SUMMONER + 1 + i as u64,
            name,
            [HOME[0] + offset, 0.0, HOME[2] - 2.0],
            140,
            3,
        );
        helper["summon_below_percent"] = json!(below);
        rows.push(helper);
    }
    rows
}

impl Fixture {
    fn new(enemies: Vec<Value>, field: Vec<Value>) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "combat-brain-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = json!({"zone_id": 0, "name": "SyntheticWorld", "sky": "", "geometry_id": 214,
            "position": [0, 0, 0], "rotation": [0, 0, 0], "tutorial": false});
        let mut arena = zone.clone();
        arena["zone_id"] = json!(BATTLE);
        fs::write(
            dir.join("encounters.json"),
            json!({"schema_version": 1, "template_file": "npc.bin",
                "template_sha256": format!("{:x}", Sha256::digest(&template)),
                "world": zone, "world_enemies": [],
                "dungeons": [{"id": BATTLE, "title_id": 12, "description_id": 13, "objective_id": 14,
                    "zone": arena, "guide": null, "enemies": enemies,
                    "reward_coins": 0, "reward_items": []}]})
            .to_string(),
        )
        .unwrap();
        fs::write(
            dir.join("field_enemies.json"),
            json!({"schema_version": 1, "enemies": field}).to_string(),
        )
        .unwrap();
        Self { dir, attach: None }
    }

    fn attach(mut self, preset: &'static str) -> Self {
        self.attach = Some(preset);
        self
    }

    /// The production order: encounter content, the shipped behaviours, then the field.
    /// The summoner takes its behaviour the way battle 96's does: by an `attach` entry of
    /// `behaviours.json` (a private copy of the shipped file with this fixture's guid).
    fn load(&self) -> EncounterEngine {
        let mut engine = EncounterEngine::load(&self.dir, None).unwrap();
        let mut file: Value =
            serde_json::from_slice(&fs::read(shipped().join("behaviours.json")).unwrap()).unwrap();
        file["attach"] = match self.attach {
            Some(preset) => json!({SUMMONER.to_string(): preset}),
            None => json!({}),
        };
        // The shipped file keeps the open-world packets off until P1's writers are proven in
        // the client (lead 2026-11-10); these examples still prove them with the captured values.
        if file.get("field").is_none() {
            file["field"] = json!({"home_speed": 10.0, "idle_state": 1, "fight_state": 32769,
                "home_state": 16385, "home_tag": 2, "home_effect": 5, "join_radius": 6.0,
                "note": "Captured: 0x1, 0x8001, 0x4001, speed 10.0, tag 2 / effect 5. (R) join radius."});
        }
        fs::write(self.dir.join("behaviours.json"), file.to_string()).unwrap();
        fs::copy(shipped().join("attacks.json"), self.dir.join("attacks.json")).unwrap();
        engine
            .load_behaviours(&self.dir.join("behaviours.json"))
            .unwrap();
        engine
            .load_field_enemies(&self.dir.join("field_enemies.json"))
            .unwrap();
        engine
    }

    fn begin(&self) -> (EncounterEngine, EncounterOutcome) {
        let mut engine = self.load();
        engine.set_player_job(BRAWLER, 1);
        let world = engine.world().clone();
        let home = CharacterState {
            guid: 1,
            zone_id: world.zone_id,
            position: world.position,
            rotation: world.rotation,
        };
        engine.begin(BATTLE, home, BRAWLER).unwrap();
        let ready = engine.on_zone_ready(BRAWLER, 0).unwrap();
        (engine, ready)
    }
}

fn is(packet: &[u8], family: u16, sub: u16) -> bool {
    packet.len() >= 4 && packet[..2] == family.to_le_bytes() && packet[2..4] == sub.to_le_bytes()
}

fn guid_at(packet: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(packet[at..at + 8].try_into().unwrap())
}

/// Enemy hits (32/7) by attacker.
fn swings(out: &EncounterOutcome) -> Vec<u64> {
    out.replies
        .iter()
        .filter(|p| is(p, 32, 7))
        .map(|p| guid_at(p, 4))
        .collect()
}

fn spawns(out: &EncounterOutcome) -> usize {
    out.replies.iter().filter(|p| is(p, 35, 2)).count()
}

fn place(engine: &EncounterEngine, guid: u64) -> Option<[f32; 3]> {
    engine
        .standing_enemies()
        .into_iter()
        .find(|(g, ..)| *g == guid)
        .map(|(_, at, _)| at)
}

fn health(engine: &EncounterEngine, guid: u64) -> u32 {
    let run = engine.state().active.as_ref().unwrap();
    run.enemies.iter().find(|s| s.guid == guid).unwrap().health
}

fn flat(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

/// Stand next to the enemy and swing until `hits` have landed; returns everything sent.
fn hit(engine: &mut EncounterEngine, guid: u64, hits: u32, now: &mut u64) -> Vec<EncounterOutcome> {
    let mut all = Vec::new();
    let mut landed = 0;
    while landed < hits {
        *now += 700;
        let at = place(engine, guid).expect("the enemy stands");
        let me = [at[0] + 1.0, at[1], at[2]];
        let before = health(engine, guid);
        all.push(engine.tick(me, BRAWLER, *now).unwrap());
        // The tick may have moved it; swing from beside where it is now.
        let at = place(engine, guid).expect("the enemy stands");
        let out = engine
            .attack(guid, [at[0] + 1.0, at[1], at[2]], BRAWLER, 0, *now)
            .unwrap();
        if health(engine, guid) < before {
            landed += 1;
        }
        all.push(out);
        assert!(*now < 3_000_000, "the fight goes nowhere");
    }
    all
}

/// Battle 96's Summoner through the real entry path: helpers out at the start, somewhere
/// else after two hits, more helpers down its ladder, and never a swing of its own.
#[test]
fn the_summoner_battle_plays_the_owners_pattern_through_the_engine() {
    let mut summoner = actor(SUMMONER, "Troll Summoner", HOME, 9000, 4);
    summoner["boss"] = json!(true);
    let fixture = Fixture::new(roster(summoner), Vec::new()).attach("troll_summoner");
    let (mut engine, ready) = fixture.begin();
    engine.toggle_god();
    let full = health(&engine, SUMMONER);
    // Zone ready shows the Summoner alone; its first rule raises two Scoutlings with the
    // fight's first swing (the first decision of a run, which a group shares).
    assert_eq!(spawns(&ready), 1);
    let mut now = 1000;
    let mut all = Vec::new();
    let start = place(&engine, SUMMONER).unwrap();
    all.extend(hit(&mut engine, SUMMONER, 1, &mut now));
    assert_eq!(all.iter().map(spawns).sum::<usize>(), 2, "two helpers rise at the start");
    assert_eq!(engine.standing_enemies().len(), 3);
    assert!(!engine.standing_enemies()[0].2, "the summoner is not shielded by its helpers");
    assert_eq!(place(&engine, SUMMONER), Some(start), "one hit: it holds its ground");
    all.extend(hit(&mut engine, SUMMONER, 1, &mut now));
    let slipped = place(&engine, SUMMONER).unwrap();
    assert!(
        flat(slipped, start) > 4.0,
        "after the second hit it is somewhere else: {start:?} -> {slipped:?}"
    );
    assert!(flat(slipped, HOME) <= 12.01, "inside its leash of the arena floor");
    // Three seconds later the third Scoutling rises where it now stands.
    for _ in 0..16 {
        now += 250;
        let at = place(&engine, SUMMONER).unwrap();
        all.push(engine.tick([at[0] + 30.0, at[1], at[2]], BRAWLER, now).unwrap());
    }
    assert_eq!(engine.standing_enemies().len(), 4, "one more helper after the slip");
    assert_eq!(place(&engine, SUMMONER), Some(slipped), "it stands still between slips");

    // Down the ladder: a Rageling under half, two more under a quarter.
    while health(&engine, SUMMONER) * 100 / full >= 50 {
        all.extend(hit(&mut engine, SUMMONER, 1, &mut now));
    }
    let ragelings = |engine: &EncounterEngine| {
        engine
            .standing_enemies()
            .iter()
            .filter(|(guid, ..)| (SUMMONER + 4..=SUMMONER + 6).contains(guid))
            .count()
    };
    assert_eq!(ragelings(&engine), 1);
    while health(&engine, SUMMONER) * 100 / full >= 25 {
        all.extend(hit(&mut engine, SUMMONER, 1, &mut now));
    }
    assert_eq!(ragelings(&engine), 3);
    // It moved many times and never once attacked; its helpers did.
    let attackers: Vec<u64> = all.iter().flat_map(swings).collect();
    assert!(!attackers.contains(&SUMMONER), "the summoner never attacks");
    assert!(!attackers.is_empty(), "its helpers do");
    assert_eq!(engine.behaviour(SUMMONER).unwrap().mode, Mode::Stationary);
}

/// The same battle played twice sends the same packets: relocation spots and helper
/// placement come from seeded generators.
#[test]
fn a_summoner_battle_replays_exactly() {
    let play = || {
        let summoner = actor(SUMMONER, "Troll Summoner", HOME, 9000, 4);
        let fixture = Fixture::new(roster(summoner), Vec::new()).attach("troll_summoner");
        let (mut engine, ready) = fixture.begin();
        engine.toggle_god();
        let mut now = 1000;
        let mut packets = ready.replies;
        for out in hit(&mut engine, SUMMONER, 12, &mut now) {
            packets.extend(out.replies);
        }
        (packets, engine.standing_enemies())
    };
    let (first, second) = (play(), play());
    assert!(first.0.len() > 50);
    assert_eq!(first.0, second.0);
    assert_eq!(first.1, second.1);
}

/// Old content: `skittish` and `summon_below_percent` rows play through the brain. The
/// ladder answers in the same reply as the hit that crossed it, the summoner is shielded
/// while a helper stands, fights beside its helpers and flees when alone.
#[test]
fn old_skittish_summoner_rows_become_rules_and_play_as_before() {
    let mut summoner = actor(SUMMONER, "Troll Summoner", HOME, 9000, 4);
    summoner["skittish"] = json!(true);
    let fixture = Fixture::new(roster(summoner), Vec::new());
    let (mut engine, ready) = fixture.begin();
    engine.toggle_god();
    assert_eq!(spawns(&ready), 1, "the helpers sleep until its health falls");
    let full = health(&engine, SUMMONER);
    let b = engine.behaviour(SUMMONER).unwrap();
    assert_eq!(b.mode, Mode::Skittish);
    assert!(b.rules.is_empty(), "the row itself is unchanged");
    let ids: Vec<&str> = b.program().rules.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ["summon_below_25", "summon_below_50", "summon_below_75", "alone"]);
    assert_eq!(b.program().stance.movement, Move::Flee { ms: 4000, speed: 3.5 });

    // Alone it runs from the player (and never swings).
    let mut now = 1000;
    let mut all = hit(&mut engine, SUMMONER, 1, &mut now);
    for _ in 0..8 {
        now += 250;
        all.push(engine.tick([HOME[0] + 1.0, 0.0, HOME[2]], BRAWLER, now).unwrap());
    }
    assert!(flat(place(&engine, SUMMONER).unwrap(), HOME) > 1.0, "it ran");
    assert!(all.iter().flat_map(swings).all(|g| g != SUMMONER));

    // Under 75 % the two first helpers appear in the reply to that very hit.
    let mut woke = None;
    while woke.is_none() {
        let outs = hit(&mut engine, SUMMONER, 1, &mut now);
        woke = outs.iter().map(spawns).find(|n| *n > 0);
    }
    assert_eq!(woke, Some(2));
    assert!(health(&engine, SUMMONER) * 100 / full < 75);
    assert!(engine.standing_enemies()[0].2, "shielded while a helper stands");
    let before = health(&engine, SUMMONER);
    now += 700;
    let at = place(&engine, SUMMONER).unwrap();
    engine.attack(SUMMONER, [at[0] + 1.0, at[1], at[2]], BRAWLER, 0, now).unwrap();
    assert_eq!(health(&engine, SUMMONER), before, "a hit on the shielded summoner does nothing");

    // Beside its helpers it fights: it closes in and swings.
    let mut fought = false;
    for _ in 0..80 {
        now += 250;
        let out = engine.tick([HOME[0] + 6.0, 0.0, HOME[2] + 6.0], BRAWLER, now).unwrap();
        fought |= swings(&out).contains(&SUMMONER);
    }
    assert!(fought, "with helpers out the old summoner swings");

    // Helpers gone: unshielded, and it flees again.
    for helper in [SUMMONER + 1, SUMMONER + 2] {
        while place(&engine, helper).is_some() {
            hit(&mut engine, helper, 1, &mut now);
        }
    }
    assert!(!engine.standing_enemies()[0].2, "the shield falls with the helpers");
    let mut swung = Vec::new();
    for _ in 0..24 {
        now += 250;
        let at = place(&engine, SUMMONER).unwrap();
        let out = engine.tick([at[0] + 2.0, at[1], at[2]], BRAWLER, now).unwrap();
        swung.extend(swings(&out));
    }
    assert!(!swung.contains(&SUMMONER), "alone again it only runs");
}

/// A row may build on a preset and add rules; a behaviour that loops is refused when the
/// file loads, and the engine keeps what it had.
#[test]
fn rows_add_rules_to_a_preset_and_a_looping_file_is_refused() {
    // A field row: the field loads after the presets, so its row may name one.
    let mut boss = field_row(SUMMONER, HOME);
    boss["behaviour"] = json!({"use": "melee", "tags": ["front"], "rules": [
        {"id": "roar", "when": {"engaged": {}}, "do": [{"say": {"text": "Changelings!", "how": "red"}}], "once": true}
    ]});
    let fixture = Fixture::new(vec![actor(200, "Cave wolf", [0.0; 3], 100, 1)], vec![boss]);
    let mut engine = fixture.load();
    let b = engine.behaviour(SUMMONER).unwrap();
    assert_eq!(b.program().rules.len(), 1);
    assert_eq!(b.program().stance.movement, Move::Chase);
    assert_eq!(b.tags, ["front"]);

    fs::create_dir_all(fixture.dir.join("bad")).unwrap();
    let bad = fixture.dir.join("bad/behaviours.json");
    let write = |presets: Value| {
        fs::write(&bad, json!({"schema_version": 2, "presets": presets}).to_string()).unwrap();
    };
    write(json!({"melee": {}, "echo": {"rules": [
        {"id": "a", "when": {"signal": {"name": "x"}}, "do": [{"signal": {"name": "y"}}]},
        {"id": "b", "when": {"signal": {"name": "y"}}, "do": [{"signal": {"name": "x"}}]}
    ]}}));
    let error = engine.load_behaviours(&bad).unwrap_err().to_string();
    assert!(error.contains("preset echo") && error.contains("loops back"), "{error}");
    write(json!({"a": {"use": "b"}, "b": {"use": "a"}}));
    let error = engine.load_behaviours(&bad).unwrap_err().to_string();
    assert!(error.contains("circle"), "{error}");
    write(json!({"melee": {"stance": {"move": "chase", "attack": "gnaw"}}}));
    let error = engine.load_behaviours(&bad).unwrap_err().to_string();
    assert!(error.contains("unknown attack 'gnaw'"), "{error}");
    // With no attacks.json beside it, the attacks this build shipped with are there.
    write(json!({"melee": {"stance": {"move": "chase", "attack": "bite"}}}));
    engine.load_behaviours(&bad).unwrap();
    engine.load_behaviours(&fixture.dir.join("behaviours.json")).unwrap();
    // Each refusal left the loaded behaviours alone.
    assert_eq!(engine.behaviour(SUMMONER).unwrap().program().rules.len(), 1);
}

// ---- the open world ----

const SOLDIER: u64 = 7001;
const NEIGHBOUR: u64 = 7002;
const FAR: u64 = 7003;

fn field_row(guid: u64, position: [f32; 3]) -> Value {
    let mut row = actor(guid, "Bixie Soldier", position, 1000, 7);
    row["level"] = json!(1);
    row["zone_id"] = json!(0);
    row["respawn_seconds"] = json!(30);
    row["aggro_range"] = json!(8.0);
    row["leash_radius"] = json!(14.0);
    row["behaviour"] = json!({"use": "bixie_soldier", "move_speed": 4.5});
    row
}

fn field_place(engine: &EncounterEngine, guid: u64) -> [f32; 3] {
    engine
        .field_teleport_targets()
        .into_iter()
        .find(|t| t.guid == guid)
        .unwrap()
        .position
}

/// 35/20 state values sent for `guid`, in order.
fn states(out: &EncounterOutcome, guid: u64) -> Vec<u32> {
    out.replies
        .iter()
        .filter(|p| is(p, 35, 20) && guid_at(p, 4) == guid)
        .map(|p| u32::from_le_bytes(p[12..16].try_into().unwrap()))
        .collect()
}

fn speeds(out: &EncounterOutcome, guid: u64) -> Vec<f32> {
    out.replies
        .iter()
        .filter(|p| is(p, 35, 23) && guid_at(p, 4) == guid)
        .map(|p| f32::from_le_bytes(p[12..16].try_into().unwrap()))
        .collect()
}

/// A field enemy notices, chases, gives up at its leash and runs home at speed 10 under
/// state 0x4001; a neighbour within 6 m joins and one farther off does not.
#[test]
fn a_field_enemy_notices_chases_gives_up_at_the_leash_and_runs_home() {
    let fixture = Fixture::new(
        vec![actor(200, "Cave wolf", [0.0; 3], 100, 1)],
        vec![
            field_row(SOLDIER, [0.0, 0.0, 0.0]),
            field_row(NEIGHBOUR, [-5.0, 0.0, 0.0]),
            field_row(FAR, [0.0, 0.0, 40.0]),
        ],
    );
    let mut engine = fixture.load();
    engine.toggle_god();
    // 30 m away nobody cares.
    let mut now = 1000;
    let out = engine.tick([30.0, 0.0, 0.0], BRAWLER, now).unwrap();
    assert!(states(&out, SOLDIER).is_empty());
    // At 7 m the soldier notices: fighting state, and it starts toward the player. The
    // neighbour is 12 m from the player, outside its own 8 m.
    now += 250;
    let out = engine.tick([7.0, 0.0, 0.0], BRAWLER, now).unwrap();
    assert_eq!(states(&out, SOLDIER), [0x8001]);
    assert!(states(&out, NEIGHBOUR).is_empty());
    assert!(field_place(&engine, SOLDIER)[0] > 0.5, "it closes in");
    // One tick later the neighbour has joined; the far one never does.
    now += 250;
    let out = engine.tick([7.0, 0.0, 0.0], BRAWLER, now).unwrap();
    assert_eq!(states(&out, NEIGHBOUR), [0x8001], "a neighbour within 6 m joins");
    // It bites on the captured rhythm once in reach.
    let mut bites = Vec::new();
    for _ in 0..40 {
        now += 250;
        let out = engine.tick([7.0, 0.0, 0.0], BRAWLER, now).unwrap();
        if swings(&out).contains(&SOLDIER) {
            bites.push(now);
        }
        assert!(states(&out, FAR).is_empty());
    }
    assert!(bites.len() >= 3, "{bites:?}");
    assert!(bites.windows(2).all(|w| w[1] - w[0] >= 2300), "2.3 s apart: {bites:?}");
    assert_eq!(field_place(&engine, FAR), [0.0, 0.0, 40.0], "40 m away it never joined");

    // The player walks off; past the leash the soldier gives up.
    let mut gave_up = None;
    let mut player = 7.0;
    for _ in 0..200 {
        now += 250;
        player += 1.0;
        let out = engine.tick([player, 0.0, 0.0], BRAWLER, now).unwrap();
        if states(&out, SOLDIER) == [0x4001] {
            assert_eq!(speeds(&out, SOLDIER), [10.0]);
            assert!(
                out.replies.iter().any(|p| is(p, 35, 41) && guid_at(p, 4) == SOLDIER),
                "the going-home effect is attached"
            );
            gave_up = Some(field_place(&engine, SOLDIER));
            break;
        }
    }
    let from = gave_up.expect("it gave up");
    assert!(from[0] <= 14.01 && from[0] > 8.0, "at its leash: {from:?}");
    // Home at 10 m/s: 2.5 m per 250 ms tick, then idle again with the effect gone.
    now += 250;
    let out = engine.tick([200.0, 0.0, 0.0], BRAWLER, now).unwrap();
    let step = from[0] - field_place(&engine, SOLDIER)[0];
    assert!((step - 2.5).abs() < 0.01, "one tick of the run home covered {step} m");
    let mut home = states(&out, SOLDIER);
    let mut walk = speeds(&out, SOLDIER);
    let mut detached = false;
    for _ in 0..12 {
        now += 250;
        let out = engine.tick([200.0, 0.0, 0.0], BRAWLER, now).unwrap();
        home.extend(states(&out, SOLDIER));
        walk.extend(speeds(&out, SOLDIER));
        detached |= out.replies.iter().any(|p| is(p, 35, 42) && guid_at(p, 4) == SOLDIER);
    }
    assert_eq!(home, [0x1], "idle again, told once");
    assert_eq!(walk, [4.5], "back to its own pace");
    assert!(detached);
    assert!(flat(field_place(&engine, SOLDIER), [0.0; 3]) < 0.01);
}

/// Without the `field` section in `behaviours.json` none of the new packets is sent and the
/// enemy returns at its own `run_speed`, as before.
#[test]
fn the_field_rules_are_data_and_absent_they_send_nothing_new() {
    let mut row = field_row(SOLDIER, [0.0, 0.0, 0.0]);
    row["behaviour"] = json!({"move_speed": 4.5, "run_speed": 3.5});
    let fixture = Fixture::new(vec![actor(200, "Cave wolf", [0.0; 3], 100, 1)], vec![row]);
    let mut engine = EncounterEngine::load(&fixture.dir, None).unwrap();
    engine
        .load_field_enemies(&fixture.dir.join("field_enemies.json"))
        .unwrap();
    engine.toggle_god();
    let mut now = 1000;
    let mut sent = Vec::new();
    for player in [7.0, 7.0, 7.0, 60.0, 60.0, 60.0, 60.0, 60.0, 60.0, 60.0, 60.0] {
        now += 250;
        sent.push(engine.tick([player, 0.0, 0.0], BRAWLER, now).unwrap());
    }
    for out in &sent {
        assert!(states(out, SOLDIER).is_empty() && speeds(out, SOLDIER).is_empty());
        assert!(!out.replies.iter().any(|p| is(p, 35, 41) || is(p, 35, 42)));
    }
    assert!(
        flat(field_place(&engine, SOLDIER), [0.0; 3]) < 0.01,
        "it came home all the same"
    );
}
