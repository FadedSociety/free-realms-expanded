//! Combat P4: the live enemy catalogue (`data/content/enemies.json`, combat design section 7).
//!
//! A kind with a `rank` takes its health and raw hit from the rank tables of
//! `combat_scale.json` at the tier of the row that places it; `health` and `hit` on the
//! definition state a number outright. A kind without a rank keeps the older rule (authored
//! level-one baselines, scaled where the row loads). Every test below goes through the
//! production loaders: `EncounterEngine::load` reads the catalogue from the content
//! directory, and the world and field loaders apply it.
use osfr_tools::encounter::{EncounterEngine, EncounterOutcome, Rank, scale};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);
const MEDIC: i32 = 11;
const WORLD_WOLF: u64 = 300;
const FIELD_KING: u64 = 100;
const FIELD_PLAIN: u64 = 101;

fn shipped() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn shipped_json(file: &str) -> Value {
    serde_json::from_slice(&fs::read(shipped().join(file)).unwrap()).unwrap()
}

/// A scratch content directory with its own catalogue: a ranked wolf (definition 400), a
/// boss-ranked king that states its numbers (401) and an unranked kind (402); one world row
/// of the wolf at level 8 and one field row each of the king and the unranked kind.
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Scratch {
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "enemy-catalogue-{}-{}",
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
        let row = |guid: u64, definition: i32, level: u32| json!({"guid":guid,"definition_id":definition,"model_id":3,"name_id":100,"name":"Synthetic","texture_alias":"","position":[0,0,0],"max_health":100,"damage":10,"boss":false,"level":level});
        let field = |guid: u64, definition: i32, area: &str| {
            let mut actor = row(guid, definition, 20);
            for (key, value) in [
                ("level_band", json!([1, 20])),
                ("zone_id", json!(0)),
                ("region", json!("Synthetic region")),
                ("spawn_area", json!({"id":area,"centre":[0,0,0],"radius":0,"count":1})),
                ("leash_radius", json!(12)),
                ("respawn_seconds", json!(3600)),
                ("aggro_range", json!(8)),
            ] {
                actor[key] = value;
            }
            actor
        };
        let mut wolf = row(WORLD_WOLF, 400, 8);
        wolf["stages"] = json!([0]);
        fs::write(dir.join("encounters.json"), json!({"schema_version":1,"template_file":"npc.bin","template_sha256":format!("{:x}",Sha256::digest(&template)),"world":zone,"world_enemies":[wolf],"dungeons":[{"id":411,"title_id":12,"description_id":13,"objective_id":14,"zone":cave,"guide":null,"enemies":[row(200, 402, 1)],"reward_coins":0,"reward_items":[]}]}).to_string()).unwrap();
        fs::write(
            dir.join("field_enemies.json"),
            json!({"schema_version":1,"enemies":[field(FIELD_KING, 401, "a"), field(FIELD_PLAIN, 402, "b")]}).to_string(),
        )
        .unwrap();
        let scratch = Self(dir);
        scratch.catalogue(|_| {});
        scratch
    }

    fn catalogue(&self, edit: impl FnOnce(&mut Value)) {
        let mut file = json!({"definitions": [
            {"id": "wolf", "kind": "Synthetic Wolf", "definition_id": 400, "rank": "normal", "tags": ["wolf"]},
            {"id": "king", "kind": "Synthetic King", "definition_id": 401, "rank": "boss", "tier": 2,
             "health": 5000, "hit": 77, "tags": ["royal", "caster"],
             "speeds": {"walk": 3.0, "run": 6.0, "home": 10.0}, "scale": 1.7},
            {"id": "plain", "kind": "Synthetic Plain", "definition_id": 402}
        ]});
        edit(&mut file);
        fs::write(self.0.join("enemies.json"), file.to_string()).unwrap();
    }

    fn scale(&self, edit: impl FnOnce(&mut Value)) {
        let mut file = shipped_json("combat_scale.json");
        edit(&mut file);
        fs::write(self.0.join("combat_scale.json"), file.to_string()).unwrap();
    }

    fn load(&self) -> EncounterEngine {
        let mut engine = EncounterEngine::load(&self.0, None).unwrap();
        engine
            .load_field_enemies(&self.0.join("field_enemies.json"))
            .unwrap();
        engine
    }
}

/// The maximum health from a 35/5 UpdateHitpoints reply (client reader `0x008D6F10`: the
/// u64 actor id, then current health at 12..16 and the maximum at 16..20).
fn max_health(out: &EncounterOutcome, guid: u64) -> Option<u32> {
    out.replies
        .iter()
        .find(|packet| {
            packet.len() >= 24
                && packet.starts_with(&[35, 0, 5, 0])
                && packet[4..12] == guid.to_le_bytes()
        })
        .map(|packet| u32::from_le_bytes(packet[16..20].try_into().unwrap()))
}

/// The damage dwords of the 32/7 AttackProcessed replies `attacker` sent (client reader
/// `0x00a2a910`: attacker at 4..12, damage at 28..32).
fn hits_from(out: &EncounterOutcome, attacker: u64) -> Vec<u32> {
    out.replies
        .iter()
        .filter(|p| {
            p.len() >= 32 && p.starts_with(&[32, 0, 7, 0]) && p[4..12] == attacker.to_le_bytes()
        })
        .map(|p| u32::from_le_bytes(p[28..32].try_into().unwrap()))
        .collect()
}

fn world_health(engine: &EncounterEngine, guid: u64) -> u32 {
    let state = engine.state();
    state
        .world_enemies
        .iter()
        .find(|enemy| enemy.guid == guid)
        .unwrap()
        .health
}

#[test]
fn the_captured_kinds_carry_their_exact_numbers() {
    let engine = EncounterEngine::load(shipped(), None).unwrap();
    // (id, health, raw hit) as the March 2014 captures read them, each at the tier the
    // definition says it was measured at.
    for (id, health, hit) in [
        ("bixie_soldier", 260, 25),
        ("bixie_battlemage", 260, 25),
        ("frostfang_growler", 760, 5),
        ("frostfang_prowler", 760, 5),
        ("frostfang_howler", 760, 5),
        ("frostfang_alpha", 3_800, 9),
        ("feathered_drake_defender", 16_143, 206),
        ("feathered_drake_ambusher", 16_143, 206),
    ] {
        let (kind, numbers) = engine
            .catalogue_kind(id, None, 1)
            .unwrap_or_else(|| panic!("{id} is a ranked kind of the shipped catalogue"));
        // As captured, times the owner's enemy multipliers (rc-0137: health x2, hits x1.5).
        let live = (scale::live_health(health), scale::live_hit(hit));
        assert_eq!(numbers, live, "{id} ({:?})", kind.rank);
        assert!(kind.health.is_none(), "{id}: the rank table gives the health");
        assert!(kind.speeds.is_some() && !kind.tags.is_empty(), "{id}");
    }
    let (alpha, _) = engine.catalogue_kind("frostfang_alpha", None, 1).unwrap();
    assert_eq!((alpha.rank, alpha.scale), (Some(Rank::Boss), Some(1.7)));
    // Somewhere else the same kind follows the tier it is placed at, and a group adds half.
    let at = |id: &str, tier, group| engine.catalogue_kind(id, Some(tier), group).unwrap().1.0;
    assert_eq!(at("bixie_soldier", 3, 1), scale::enemy_health(Rank::Minion, 3, 1));
    assert_eq!(at("frostfang_growler", 1, 3), scale::live_health(1_520));
    // An unranked kind is in the catalogue but has no rank numbers.
    assert!(engine.catalogue_kind("hooligan_wolf", None, 1).is_none());
    assert!(engine.catalogue_kinds().iter().any(|k| k.id == "hooligan_wolf"));
    assert_eq!(
        engine.catalogue_kinds().len(),
        shipped_json("enemies.json")["definitions"]
            .as_array()
            .unwrap()
            .len()
    );
}

#[test]
fn shipped_field_rows_of_a_ranked_kind_follow_the_rank_at_their_level() {
    let mut engine = EncounterEngine::load(shipped(), None).unwrap();
    engine
        .load_behaviours(&shipped().join("behaviours.json"))
        .unwrap();
    engine
        .load_field_enemies(&shipped().join("field_enemies.json"))
        .unwrap();
    let out = engine.on_zone_ready(MEDIC, 0).unwrap();
    let field = shipped_json("field_enemies.json");
    let mut seen = 0;
    for row in field["enemies"].as_array().unwrap() {
        if row["name"] != "Bixie Soldier" || row["definition_id"] != 930053 {
            continue;
        }
        seen += 1;
        let guid = row["guid"].as_u64().unwrap();
        let tier = scale::tier_of_level(row["level"].as_u64().unwrap() as u8);
        assert_eq!(
            max_health(&out, guid),
            Some(scale::enemy_health(Rank::Minion, tier, 1)),
            "{guid} at level {} is a tier-{tier} minion, whatever health the row wrote",
            row["level"]
        );
    }
    assert_eq!(seen, 4, "the four Queensfields Bixie Soldiers");
}

#[test]
fn rows_inherit_from_their_definition_and_unranked_rows_keep_the_old_rule() {
    let scratch = Scratch::new();
    let mut engine = scratch.load();
    // The world wolf wrote 100 health at level 8: a normal enemy of tier 3.
    assert_eq!(world_health(&engine, WORLD_WOLF), scale::enemy_health(Rank::Normal, 3, 1));
    assert_eq!(world_health(&engine, WORLD_WOLF), 4_647, "2,324 times the owner's x2 (rc-0137)");
    let out = engine.on_zone_ready(MEDIC, 0).unwrap();
    let stated = Some(scale::live_health(5_000));
    assert_eq!(max_health(&out, FIELD_KING), stated, "health stated outright, times the owner's x2");
    // The unranked field row: 100 authored at level 20 on the player curve and enemy scale.
    let scale_file = shipped_json("combat_scale.json");
    let ratio = scale_file["tier_ratio"].as_f64().unwrap();
    let legacy = 100.0
        * ratio.powf(19.0 / 4.0)
        * scale_file["enemy_health_scale"].as_f64().unwrap()
        * scale_file["enemy_health_multiplier"].as_f64().unwrap_or(1.0);
    assert_eq!(max_health(&out, FIELD_PLAIN), Some(legacy.round() as u32));
    engine.set_player_job(MEDIC, 20);
    let out = engine.tick([0.0; 3], MEDIC, 100).unwrap();
    assert_eq!(hits_from(&out, FIELD_KING), [scale::live_hit(77)], "raw hit stated outright, x1.5");
}

#[test]
fn a_kill_carries_the_catalogue_rank_and_tags() {
    let scratch = Scratch::new();
    let mut engine = scratch.load();
    engine.set_player_job(MEDIC, 20);
    engine.on_zone_ready(MEDIC, 0).unwrap();
    let mut kills = Vec::new();
    for swing in 0..10 {
        let out = engine
            .attack(FIELD_KING, [0.0; 3], MEDIC, 0, 1_000 + swing * 1_000)
            .unwrap();
        kills.extend(out.kills);
        if !kills.is_empty() {
            break;
        }
    }
    assert_eq!(kills.len(), 1, "the king falls inside ten level-20 swings");
    assert_eq!(kills[0].kind, 401);
    assert_eq!(kills[0].rank, Rank::Boss, "the row says boss: false, the catalogue says boss");
    assert_eq!(kills[0].tags, ["royal", "caster"]);
}

#[test]
fn the_rank_table_tunes_ranked_rows_from_data() {
    let scratch = Scratch::new();
    let before = world_health(&scratch.load(), WORLD_WOLF);
    scratch.scale(|file| {
        let basics = file["ranks"]["normal"]["hp_basics"].as_f64().unwrap();
        file["ranks"]["normal"]["hp_basics"] = json!(basics * 2.0);
    });
    let after = world_health(&scratch.load(), WORLD_WOLF);
    assert!(after.abs_diff(before * 2) <= 1, "{before} -> {after}");
    // The shipped table is untouched by another directory's file.
    assert_eq!(scale::enemy_health(Rank::Normal, 3, 1), before);
}

#[test]
fn a_bad_catalogue_refuses_the_load_and_names_the_file() {
    let scratch = Scratch::new();
    let bad: [(&str, fn(&mut Value)); 9] = [
        ("unknown rank", |v| v["definitions"][0]["rank"] = json!("mighty")),
        ("tier 0", |v| v["definitions"][0]["tier"] = json!(0)),
        ("tier 6", |v| v["definitions"][0]["tier"] = json!(6)),
        ("health 0", |v| v["definitions"][1]["health"] = json!(0)),
        ("a number without a rank", |v| v["definitions"][2]["hit"] = json!(5)),
        ("duplicate id", |v| v["definitions"][2]["id"] = json!("wolf")),
        ("speed 0", |v| v["definitions"][1]["speeds"]["run"] = json!(0.0)),
        ("empty tag", |v| v["definitions"][0]["tags"] = json!([""])),
        ("no id", |v| {
            v["definitions"][0].as_object_mut().unwrap().remove("id");
        }),
    ];
    for (why, edit) in bad {
        scratch.catalogue(edit);
        let error = EncounterEngine::load(&scratch.0, None)
            .err()
            .unwrap_or_else(|| panic!("{why} must refuse the load"));
        assert!(error.to_string().contains("enemies.json"), "{why}: {error}");
    }
    fs::write(scratch.0.join("enemies.json"), b"not json").unwrap();
    assert!(EncounterEngine::load(&scratch.0, None).is_err());
    scratch.catalogue(|_| {}); // canary: the unedited catalogue loads
    assert!(EncounterEngine::load(&scratch.0, None).is_ok());
    // No file at all is an empty catalogue: every row keeps the old rule.
    fs::remove_file(scratch.0.join("enemies.json")).unwrap();
    let engine = EncounterEngine::load(&scratch.0, None).unwrap();
    assert!(engine.catalogue_kinds().is_empty());
    assert_ne!(world_health(&engine, WORLD_WOLF), 2_324);
}

#[test]
fn every_shipped_ranked_kind_resolves_inside_the_engine_bounds() {
    let engine = EncounterEngine::load(shipped(), None).unwrap();
    let (_, tiers) = engine.scale_span();
    let mut ranked = 0;
    for kind in engine.catalogue_kinds() {
        if kind.rank.is_none() {
            continue;
        }
        ranked += 1;
        for tier in 1..=tiers {
            let (_, (health, hit)) = engine.catalogue_kind(&kind.id, Some(tier), 4).unwrap();
            assert!(health > 0 && hit > 0, "{} at tier {tier}", kind.id);
        }
    }
    assert!(ranked >= 8, "the captured kinds are ranked: {ranked}");
}
