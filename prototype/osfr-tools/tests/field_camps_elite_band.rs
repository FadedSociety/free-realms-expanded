//! T332: the level 15-20 elite field camps and the shipped 14-20 reward band.
//!
//! Content slice. The three new camps sit on six audited centres (T415 moved
//! them off the W102 starter centres; `common/field_camp_centres.rs`), the
//! existing `field_roamer` preset and the shipped troll/bixie/hooligan
//! definitions; the reward band is the existing `field_kill_rewards.json`
//! 14-20 row. No production behaviour, packet or schema changed. The load root
//! and the reward helpers are `tests/common/field_reward_fixture.rs`, shared
//! with the T240 payout suite.
//!
//! Client evidence for the existing paths this suite asserts (re-checked with
//! `lab/autopilot/ida_query.py`):
//! - 35/2 AddNpc constructor `0x0091CAC0`, body reader `0x00921A10`: the u64
//!   actor id is the first field and there is no per-actor level field.
//! - 38/19 coin balance: one i32, client constructor `0x0089E670`.
//! - 38/14 job update: six i32, client reader `0x008D86E0`.
//! - 36/10 basic attack: client function `0x00A35150`, writer `0x00A342E0`.

use osfr_tools::{
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterOutcome},
    gameplay::{Gameplay, Outcome},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

#[path = "common/field_camp_centres.rs"]
mod field_camp_centres;
#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "common/field_reward_fixture.rs"]
mod field_reward_fixture;

use field_camp_centres::{placement_masked, verified_centre};
use field_enemy_fixture::{Fixture as EngineFixture, content_dir, content_path, spawned};
use field_reward_fixture::{
    RewardFixture, attack_one, bank, character_at, kill_one, player, shipped_field_actor, source,
};

#[path = "support/packets.rs"]
mod packets;

use packets::{carrier, count, packet};

/// The six elite actors in file order: GUID, authored level, area, its audited
/// centre (T415), the W102 area and row whose leash it reuses, and the
/// loader-consumed definition/model/name/stats.
#[derive(Clone, Copy)]
struct Elite {
    guid: u64,
    level: u32,
    area: &'static str,
    centre: [f32; 3],
    w102: &'static str,
    w102_row: u64,
    definition: u64,
    model: u64,
    name_id: u64,
    name: &'static str,
    health: u64,
    damage: u64,
}

const ELITE: [Elite; 6] = [
    Elite {
        guid: 131000102031,
        level: 15,
        area: "ELITE-T1",
        centre: verified_centre("ELITE-T1"),
        w102: "W102-T1",
        w102_row: 131000102010,
        definition: 930025,
        model: 168,
        name_id: 72269,
        name: "Troll Bone Shaman",
        health: 100,
        damage: 7,
    },
    Elite {
        guid: 131000102032,
        level: 16,
        area: "ELITE-T2",
        centre: verified_centre("ELITE-T2"),
        w102: "W102-T3",
        w102_row: 131000102012,
        definition: 930025,
        model: 168,
        name_id: 72269,
        name: "Troll Bone Shaman",
        health: 100,
        damage: 7,
    },
    Elite {
        guid: 131000102033,
        level: 17,
        area: "ELITE-Q1",
        centre: verified_centre("ELITE-Q1"),
        w102: "W102-Q3",
        w102_row: 131000102009,
        definition: 930053,
        model: 220,
        name_id: 440711,
        name: "Bixie Soldier",
        health: 180,
        damage: 15,
    },
    Elite {
        guid: 131000102034,
        level: 18,
        area: "ELITE-Q2",
        centre: verified_centre("ELITE-Q2"),
        w102: "W102-Q1",
        w102_row: 131000102007,
        definition: 930053,
        model: 220,
        name_id: 440711,
        name: "Bixie Soldier",
        health: 180,
        damage: 15,
    },
    Elite {
        guid: 131000102035,
        level: 19,
        area: "ELITE-H1",
        centre: verified_centre("ELITE-H1"),
        w102: "W102-H3",
        w102_row: 131000102006,
        definition: 930060,
        model: 203,
        name_id: 5100401,
        name: "Hooligan",
        health: 180,
        damage: 15,
    },
    Elite {
        guid: 131000102036,
        level: 20,
        area: "ELITE-H2",
        centre: verified_centre("ELITE-H2"),
        w102: "W102-H1",
        w102_row: 131000102004,
        definition: 930060,
        model: 203,
        name_id: 5100401,
        name: "Hooligan",
        health: 180,
        damage: 15,
    },
];

/// The three appended camps: id, band, members, region.
#[derive(Clone, Copy)]
struct Camp {
    id: &'static str,
    band: [u32; 2],
    members: [u64; 2],
    region: &'static str,
}

const CAMPS: [Camp; 3] = [
    Camp {
        id: "elite_trolls",
        band: [15, 16],
        members: [131000102031, 131000102032],
        region: "Lakeshore / northeast Wilds",
    },
    Camp {
        id: "elite_bixies",
        band: [17, 18],
        members: [131000102033, 131000102034],
        region: "Sanctuary / Queensfields",
    },
    Camp {
        id: "elite_hooligans",
        band: [19, 20],
        members: [131000102035, 131000102036],
        region: "Highroad outskirts",
    },
];

/// The 28 pre-T332 actors and 10 groups in file order. The elite rows must be
/// appended after them, never inserted or reordered.
const EXISTING_ACTORS: [u64; 28] = [
    100000004154,
    100000004155,
    5064655448674664449,
    5064655448674664450,
    131000102001,
    131000102002,
    131000102003,
    131000102004,
    131000102005,
    131000102006,
    131000102007,
    131000102008,
    131000102009,
    131000102010,
    131000102011,
    131000102012,
    131000102013,
    131000102014,
    131000102015,
    131000102016,
    131000102017,
    131000102018,
    131000102019,
    131000102020,
    131000102021,
    131000102022,
    131000102023,
    131000102024,
];

const EXISTING_GROUPS: [&str; 10] = [
    "starting_farnum_wolves",
    "starting_highroad_hooligans",
    "starting_queensfields_bixies",
    "starting_wilds_shamans",
    "mid_farnum_wolves",
    "mid_highroad_hooligans",
    "mid_wilds_shamans",
    "mid_queensfields_bixies",
    "high_farnum_wolves",
    "high_wilds_shamans",
];

/// (R) Pinned deep-equal digest of the 28 actor rows and 10 group rows before
/// this slice appended to them. Any changed, reordered or reformatted prefix
/// field (including evidence text) changes this digest, except the placement
/// of the 12 MID/HIGH rows T415 moved (`placement_masked`); the value is the
/// pre-T415 prefix's masked digest.
const EXISTING_PREFIX_SHA256: &str =
    "30411707307c43dbf7544bd3748306f6777fa29a033028d4de720c379f3eb749";

/// The existing level-10 and level-14 controls used to prove the actor level,
/// not the player level, selects the reward band.
const HIGH_SHAMAN_14: u64 = 131000102024;
const HIGH_SHAMAN_14_HOME: [f32; 3] = verified_centre("HIGH-T2");
const MID_BIXIE_10: u64 = 131000102020;
const MID_BIXIE_10_HOME: [f32; 3] = verified_centre("MID-Q2");
const LEVEL_TWENTY_ACTOR: u64 = 131000110020;
const LEVEL_TWENTY_WOLF: u64 = 131000110156;
/// The elite camp's shared 180 s respawn.
const ELITE_RESPAWN_MS: u64 = 180_000;
/// T699 appended this many quest-gated quest-target actors after the elite six.
const T699_ACTORS: usize = 15;
/// T730 appended one more (the Gloam Creature of One More Gloam).
const T730_ACTORS: usize = 1;
/// T889 appended the hostile Hooligan and Hooligan Archer of the Highroad tent (one group).
const T889_ACTORS: usize = 2;

fn shipped_field() -> Value {
    serde_json::from_slice(&fs::read(content_path()).unwrap()).unwrap()
}

fn shipped_rewards() -> Value {
    serde_json::from_slice(&fs::read(content_dir().join("field_kill_rewards.json")).unwrap())
        .unwrap()
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn actor<'a>(field: &'a Value, guid: u64) -> &'a Value {
    field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["guid"].as_u64() == Some(guid))
        .unwrap_or_else(|| panic!("field actor {guid} missing"))
}

fn group<'a>(field: &'a Value, id: &str) -> &'a Value {
    field["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("field spawn group {id} missing"))
}

/// Compare authored decimal coordinates at the runtime's `f32` precision.
fn position(value: &Value) -> [f32; 3] {
    let points = value.as_array().unwrap();
    [
        points[0].as_f64().unwrap() as f32,
        points[1].as_f64().unwrap() as f32,
        points[2].as_f64().unwrap() as f32,
    ]
}

fn bits(value: [f32; 3]) -> [u32; 3] {
    value.map(f32::to_bits)
}

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

fn band_row(min_level: u32, max_level: u32, coins: u32, job_xp: u32) -> Value {
    json!({
        "min_level": min_level,
        "max_level": max_level,
        "coins": coins,
        "job_xp": job_xp,
        "source": source()
    })
}

/// A level-20 actor on definition 930108, used for the uncovered-level case.
fn level_twenty_actor() -> Value {
    json!({
        "guid": LEVEL_TWENTY_ACTOR,
        "definition_id": 930108,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic field reward",
        "texture_alias": "",
        "position": [0.0, 0.0, 0.0],
        // Authored level-one HP; the loader scales it by level 20.
        "max_health": 35,
        "damage": 0,
        "boss": false,
        "level": 20,
        "respawn_seconds": 60,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode": "stationary", "reach": 2.5, "attack_ms": 1000},
        "quest_ids": []
    })
}

/// A level-20 actor on definition 3156, whose shipped override pays 2/5 at
/// every level. The elite camps must not use this definition.
fn level_twenty_wolf() -> Value {
    json!({
        "guid": LEVEL_TWENTY_WOLF,
        "definition_id": 3156,
        "model_id": 3284,
        "name_id": 5100200,
        "name": "Hooligan Wolf",
        "texture_alias": "",
        "position": [0.0, 0.0, 0.0],
        "max_health": 35,
        "damage": 0,
        "boss": false,
        "level": 20,
        "respawn_seconds": 60,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode": "stationary", "reach": 2.5, "attack_ms": 1000},
        "quest_ids": []
    })
}

fn switch_job(
    game: &mut Gameplay,
    character: &CharacterState,
    profile: i32,
    now_ms: u64,
) -> Outcome {
    let mut select = vec![26, 0, 13, 0];
    select.extend(profile.to_le_bytes());
    game.handle_in_world(&select, character, now_ms).unwrap()
}

/// Item instances and counts the player owns; a field kill must not change them.
fn items(game: &Gameplay) -> Vec<(i32, u32)> {
    game.state()
        .inventory
        .items
        .iter()
        .map(|item| (item.definition_id, item.count))
        .collect()
}

/// The canonical deep-equal digest of the 28 old actors (placement masked) and
/// 10 old groups.
fn existing_prefix_digest(field: &Value) -> String {
    let enemies = &field["enemies"].as_array().unwrap()[..EXISTING_ACTORS.len()];
    let prefix = json!({
        "enemies": enemies.iter().map(placement_masked).collect::<Vec<_>>(),
        "spawn_groups": &field["spawn_groups"].as_array().unwrap()[..EXISTING_GROUPS.len()],
    });
    format!("{:x}", Sha256::digest(serde_json::to_vec(&prefix).unwrap()))
}

#[test]
fn six_elite_camps_load_with_levels_fifteen_to_twenty() {
    // The real loader is the gate: a duplicate GUID/area, an out-of-band level,
    // a missing group member or a malformed row fails here before any assertion.
    let mut engine = EngineFixture::shipped().load();
    let out = engine.on_zone_ready(11, 0).unwrap();
    let spawned = spawned(&out);
    let field = shipped_field();
    let enemies = field["enemies"].as_array().unwrap();
    let groups = field["spawn_groups"].as_array().unwrap();

    assert_eq!(
        enemies.len(),
        49 + T730_ACTORS + T889_ACTORS,
        "28 pre-T332 + 6 elite actors + 15 T699 and 1 T730 quest targets + 2 T889 tent bodies"
    );
    assert_eq!(
        groups.len(),
        14,
        "10 pre-T332 + 3 elite groups + the T889 tent group"
    );
    assert_eq!(
        engine.field_guids().len(),
        49 + T730_ACTORS + T889_ACTORS,
        "the live field vector grows from 28 to 34 + 15 T699, 1 T730 quest targets and 2 T889 tent bodies"
    );
    assert_eq!(
        &engine.field_guids()[..EXISTING_ACTORS.len()],
        EXISTING_ACTORS.as_slice(),
        "the 28 pre-T332 actors keep their order"
    );

    for camp in CAMPS {
        let row = group(&field, camp.id);
        assert_eq!(row["zone_id"], 0);
        assert_eq!(row["level_band"], json!(camp.band));
        assert_eq!(row["respawn_seconds"], 180);
        assert_eq!(row["aggro_range"], 8.0);
        assert_eq!(row["leash_radius"], 12.0);
        assert_eq!(row["behaviour"], "field_roamer");
        assert_eq!(row["region"], camp.region);
        assert_eq!(row["members"], json!(camp.members));
        assert!(camp.band[0] >= 1 && camp.band[1] <= 20);
        let reconstructed = row["source"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap_or_else(|| panic!("{} needs reconstructed provenance", camp.id));
        let evidence = reconstructed["evidence"].as_str().unwrap();
        assert!(evidence.starts_with("(R)"));
        assert!(evidence.contains("180 s respawn"));
        assert!(evidence.contains("six-actor/three-group"));
    }

    for elite in ELITE {
        let row = actor(&field, elite.guid);
        assert_eq!(row["level"], elite.level, "{} level policy", elite.area);
        assert_eq!(
            row["quest_ids"],
            json!([]),
            "{} must be always-on",
            elite.area
        );
        assert_eq!(row["boss"], false);
        assert_eq!(row["power_up_table"], "standard");
        assert_eq!(row["definition_id"], elite.definition);
        assert_eq!(row["model_id"], elite.model);
        assert_eq!(row["name_id"], elite.name_id);
        assert_eq!(row["name"], elite.name);
        assert_eq!(row["max_health"], elite.health);
        assert_eq!(row["damage"], elite.damage);
        assert_eq!(row["spawn_area"]["id"], elite.area);
        assert_eq!(row["spawn_area"]["radius"], 6.0);
        assert_eq!(row["spawn_area"]["count"], 1);
        assert_eq!(bits(position(&row["position"])), bits(elite.centre));
        assert_eq!(
            bits(position(&row["spawn_area"]["centre"])),
            bits(elite.centre)
        );
        assert_eq!(
            engine.field_level(elite.guid),
            Some(elite.level),
            "the loader must consume {}'s authored level",
            elite.area
        );
        assert_eq!(engine.behaviour(elite.guid).unwrap().leash, 12.0);
        assert!(
            spawned.contains(&elite.guid),
            "{} must spawn through 35/2",
            elite.area
        );
        let source = row["source"].as_array().unwrap();
        let reference = source
            .iter()
            .find(|source| source["kind"] == "reference")
            .unwrap_or_else(|| panic!("{} needs reference provenance", elite.area));
        let reference_text = reference["evidence"].as_str().unwrap();
        assert!(reference_text.contains(elite.w102));
        assert!(reference_text.contains(&format!("row {}", elite.w102_row)));
        let reconstructed = source
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap_or_else(|| panic!("{} needs reconstructed provenance", elite.area));
        let evidence = reconstructed["evidence"].as_str().unwrap();
        assert!(evidence.starts_with("(R)"));
        assert!(evidence.contains(&format!("level {}", elite.level)));
        assert!(evidence.contains("six-actor/three-group"));
        assert!(evidence.contains("180 s"));
        let camp = CAMPS
            .iter()
            .find(|camp| camp.members.contains(&elite.guid))
            .unwrap();
        assert_eq!(
            group(&field, camp.id)["members"],
            json!(camp.members),
            "{} belongs to {}",
            elite.area,
            camp.id
        );
    }
}

#[test]
fn every_elite_home_is_a_verified_w102_centre() {
    // T415 moved the elite camps off the W102 starter centres: each home is
    // its own audited centre (the W102 checker plus the T415 separation rules).
    let engine = EngineFixture::shipped().load();
    let field = shipped_field();
    let mut used = BTreeSet::new();
    for elite in ELITE {
        let row = actor(&field, elite.guid);
        let centre = verified_centre(elite.area);
        assert_eq!(
            bits(position(&row["position"])),
            bits(centre),
            "{} must stay on its verified centre",
            elite.area
        );
        assert_eq!(
            bits(position(&row["spawn_area"]["centre"])),
            bits(centre),
            "{} centre must equal its position",
            elite.area
        );
        assert_eq!(row["spawn_area"]["radius"], 6.0);
        assert_eq!(row["spawn_area"]["count"], 1);
        assert!(
            used.insert(bits(centre)),
            "{} reuses a verified centre",
            elite.area
        );
        assert_eq!(engine.behaviour(elite.guid).unwrap().leash, 12.0);
    }
    assert_eq!(used.len(), 6, "six distinct verified centres");
}

#[test]
fn the_elite_camps_spawn_in_on_zone_ready() {
    let mut engine = EngineFixture::shipped().load();
    let out = engine.on_zone_ready(11, 0).unwrap();
    let spawned = spawned(&out);
    assert_eq!(
        spawned.len(),
        32,
        "24 pre-T332 always-on + 6 elite actors + 2 T889 tent bodies"
    );
    let mut scaled = Vec::new();
    for elite in ELITE {
        assert!(
            spawned.contains(&elite.guid),
            "{} ({}) must spawn through the existing 35/2 AddNpc path",
            elite.area,
            elite.guid
        );
        let hp = max_health(&out, elite.guid).expect("35/5 HP packet");
        assert!(
            hp > elite.health as u32,
            "{} authored {} HP at level 1; the loader must scale level {} once, got {hp}",
            elite.area,
            elite.health,
            elite.level
        );
        scaled.push(hp);
    }
    assert!(
        scaled.windows(2).all(|pair| pair[0] < pair[1]),
        "each higher authored level must raise the scaled HP: {scaled:?}"
    );
}

#[test]
fn a_level_twenty_kill_pays_the_14_20_band_once() {
    let elite = ELITE[5];
    let fixture = RewardFixture::new(vec![shipped_field_actor(elite.guid).0]);
    let at = [elite.centre[0] + 1.0, elite.centre[1], elite.centre[2]];
    let character = character_at(101, at);
    let mut game = player(&fixture, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();
    let items_before = items(&game);

    let (lethal, killed_at) = kill_one(&mut game, &mut field, &character, elite.guid, 0);
    assert_eq!(field.health(elite.guid), Some(0));
    assert_eq!(count(&lethal.replies, 38, 19), 1);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "the level-20 Hooligan pays the 14-20 band's 18 coins"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice(),
        "and 35 active-job XP"
    );
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 18);
    assert_eq!(xp.get(&43), Some(&35));
    assert_eq!(items(&game), items_before, "a field kill grants no item");

    // A replay or a corpse hit changes neither the wallet nor the field.
    let before = bank(&game);
    for now in [killed_at + 700, killed_at + 1_400] {
        let replay = attack_one(&mut game, &mut field, &character, elite.guid, now);
        assert!(!replay.changed && replay.replies.is_empty());
    }
    assert_eq!(bank(&game), before);
    let quiet = game
        .tick_shared_effects(&mut field, &character, killed_at + 50)
        .unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
    assert_eq!(bank(&game), before, "the killer is not paid twice");

    // A wallet or active job that cannot represent the payout refuses it whole
    // through the normal checked paths: no server error, no half payout.
    let capped_fixture = RewardFixture::new(vec![shipped_field_actor(elite.guid).0]);
    let capped_character = character_at(303, at);
    let near_coins = i32::MAX as u32 - 10;
    let near_xp = u32::MAX - 20;
    let mut capped_game = player(
        &capped_fixture,
        303,
        43,
        at,
        near_coins,
        Some((43, near_xp)),
    );
    let mut capped_field = capped_game.take_shared_fields().unwrap();
    let (capped_kill, capped_at) = kill_one(
        &mut capped_game,
        &mut capped_field,
        &capped_character,
        elite.guid,
        0,
    );
    assert!(
        capped_kill.changed,
        "the server-authoritative kill still happens"
    );
    assert_eq!(
        bank(&capped_game).0,
        near_coins,
        "the coins are not half-paid"
    );
    assert_eq!(
        bank(&capped_game).1.get(&43),
        Some(&near_xp),
        "the XP is not half-paid"
    );
    let capped_quiet = capped_game
        .tick_shared_effects(&mut capped_field, &capped_character, capped_at + 50)
        .unwrap();
    assert!(!capped_quiet.changed && capped_quiet.replies.is_empty());
    assert_eq!(
        bank(&capped_game).0,
        near_coins,
        "the refused payout stays refused"
    );
    assert_eq!(
        bank(&capped_game).1.get(&43),
        Some(&near_xp),
        "the refused XP stays refused"
    );
}

#[test]
fn a_level_fifteen_kill_pays_the_same_band_and_a_level_fourteen_control_still_pays_old() {
    let fifteen = ELITE[0];
    let fixture = RewardFixture::new(vec![
        shipped_field_actor(fifteen.guid).0,
        shipped_field_actor(HIGH_SHAMAN_14).0,
        shipped_field_actor(MID_BIXIE_10).0,
    ]);
    let fifteen_at = [
        fifteen.centre[0] + 1.0,
        fifteen.centre[1],
        fifteen.centre[2],
    ];
    let fourteen_at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let ten_at = [
        MID_BIXIE_10_HOME[0] + 1.0,
        MID_BIXIE_10_HOME[1],
        MID_BIXIE_10_HOME[2],
    ];
    let fifteen_character = character_at(101, fifteen_at);
    let fourteen_character = character_at(202, fourteen_at);
    let ten_character = character_at(303, ten_at);
    let mut a = player(&fixture, 101, 43, fifteen_at, 0, None);
    let mut b = player(&fixture, 202, 43, fourteen_at, 0, None);
    let mut c = player(&fixture, 303, 43, ten_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    let _ = b.take_shared_fields();
    let _ = c.take_shared_fields();

    let (fifteen_out, _) = kill_one(&mut a, &mut field, &fifteen_character, fifteen.guid, 0);
    assert_eq!(
        packet(&fifteen_out.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "a level-15 elite actor uses the 14-20 band"
    );
    assert_eq!(
        packet(&fifteen_out.replies, 38, 14),
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice()
    );
    assert_eq!(a.state().inventory.coins, 18);
    assert_eq!(a.state().profiles.unwrap().experience.get(&43), Some(&35));

    let (fourteen_out, _) = kill_one(&mut b, &mut field, &fourteen_character, HIGH_SHAMAN_14, 0);
    assert_eq!(
        packet(&fourteen_out.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "the existing level-14 HIGH-T2 still uses the same band"
    );
    assert_eq!(
        packet(&fourteen_out.replies, 38, 14),
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice()
    );
    assert_eq!(b.state().inventory.coins, 18);
    assert_eq!(b.state().profiles.unwrap().experience.get(&43), Some(&35));

    let (ten_out, _) = kill_one(&mut c, &mut field, &ten_character, MID_BIXIE_10, 0);
    assert_eq!(
        packet(&ten_out.replies, 38, 19),
        [38u8, 0, 19, 0, 12, 0, 0, 0].as_slice(),
        "a level-10 MID-Q2 bixie still uses the 9-13 band"
    );
    assert_eq!(
        packet(&ten_out.replies, 38, 14),
        carrier(38, 14, &[43, 1, 25, 0, 0, 0]).as_slice()
    );
    assert_eq!(c.state().inventory.coins, 12);
    assert_eq!(c.state().profiles.unwrap().experience.get(&43), Some(&25));
}

#[test]
fn two_players_share_an_elite_kill_and_each_is_paid_once() {
    let elite = ELITE[5];
    let fixture = RewardFixture::new(vec![shipped_field_actor(elite.guid).0]);
    let a_at = [elite.centre[0] + 1.0, elite.centre[1], elite.centre[2]];
    let b_at = [elite.centre[0] - 1.0, elite.centre[1], elite.centre[2]];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    let _ = b.take_shared_fields();
    let full_health = field.health(elite.guid).unwrap();

    attack_one(&mut a, &mut field, &a_character, elite.guid, 0);
    attack_one(&mut b, &mut field, &b_character, elite.guid, 0);
    // B's job changes after its hit and before A's kill: the XP follows the
    // job active at the claim, not a profile recorded at damage time.
    let switched = switch_job(&mut b, &b_character, 35, 100);
    assert!(
        switched.changed,
        "{}",
        switched
            .replies
            .iter()
            .map(|row| String::from_utf8_lossy(row))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let (lethal, killed_at) = kill_one(&mut a, &mut field, &a_character, elite.guid, 700);
    assert_eq!(count(&lethal.replies, 38, 19), 1);
    assert_eq!(a.state().inventory.coins, 18);
    assert_eq!(a.state().profiles.unwrap().experience.get(&43), Some(&35));

    let claimed = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 50)
        .unwrap();
    assert_eq!(
        packet(&claimed.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "the credited second player is paid the same band"
    );
    assert_eq!(
        packet(&claimed.replies, 38, 14),
        carrier(38, 14, &[35, 1, 35, 0, 0, 0]).as_slice(),
        "B's new active job receives the XP"
    );
    let (coins, xp) = bank(&b);
    assert_eq!(coins, 18);
    assert_eq!(xp.get(&35), Some(&35));
    assert_eq!(xp.get(&43), None, "the inactive job is untouched");
    let paid = bank(&b);
    let again = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 100)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
    assert_eq!(bank(&b), paid, "each player is paid once");
    assert_eq!(field.health(elite.guid), Some(0), "one shared kill");

    // The group's 180 s timer respawns the actor, not one millisecond early.
    let players = vec![
        a.field_participant(&a_character).unwrap(),
        b.field_participant(&b_character).unwrap(),
    ];
    a.tick_shared_motion(
        &mut field,
        &players,
        killed_at + ELITE_RESPAWN_MS - 100,
        100,
    )
    .unwrap();
    assert_eq!(field.health(elite.guid), Some(0));
    a.tick_shared_motion(
        &mut field,
        &players,
        killed_at + ELITE_RESPAWN_MS + 100,
        100,
    )
    .unwrap();
    assert_eq!(field.health(elite.guid), Some(full_health));
    assert_eq!(field.respawn_at(elite.guid), None);
}

#[test]
fn a_malformed_or_duplicate_elite_camp_is_refused_atomically() {
    let fixture = EngineFixture::copied();
    let mut engine = fixture.load();
    let before = engine.field_teleport_targets();
    let behaviour = engine.behaviour(ELITE[5].guid).cloned().unwrap();
    let original = shipped_field();
    let mutations: [(&str, fn(&mut Value)); 3] = [
        ("duplicate member", |field| {
            let group = field["spawn_groups"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|group| group["id"] == "elite_hooligans")
                .unwrap();
            group["members"] = json!([ELITE[5].guid, ELITE[5].guid]);
        }),
        ("level outside band", |field| {
            let row = field["enemies"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["guid"] == ELITE[5].guid)
                .unwrap();
            row["level"] = json!(14);
        }),
        ("position off its centre", |field| {
            let row = field["enemies"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["guid"] == ELITE[5].guid)
                .unwrap();
            row["position"] = json!([0.0, 0.0, 0.0]);
        }),
    ];
    for (label, mutate) in mutations {
        let mut field = original.clone();
        mutate(&mut field);
        fs::write(fixture.field_path(), serde_json::to_vec(&field).unwrap()).unwrap();
        assert!(
            engine.reload_field_enemies(fixture.field_path()).is_err(),
            "{label} must be refused"
        );
        assert_eq!(
            engine.field_teleport_targets(),
            before,
            "{label} changed the live field"
        );
        assert_eq!(
            engine.behaviour(ELITE[5].guid),
            Some(&behaviour),
            "{label} changed the live behaviour"
        );
    }

    // The malformed reward table is likewise a load refusal, not a payout-time
    // panic, and the previous table is restored before the field is used.
    let rewards = RewardFixture::new(vec![shipped_field_actor(ELITE[5].guid).0]);
    rewards.write_rewards(&json!({
        "schema_version": 1,
        "level_rewards": [band_row(1, 2, 2, 5), band_row(2, 3, 4, 10)],
        "definition_overrides": []
    }));
    assert!(EncounterEngine::load(rewards.dir(), None).is_err());
    rewards.write_rewards(&shipped_rewards());
    assert!(EncounterEngine::load(rewards.dir(), None).is_ok());
}

#[test]
fn the_existing_camps_and_wolf_quest_are_unchanged() {
    let field = shipped_field();
    let enemies = field["enemies"].as_array().unwrap();
    let groups = field["spawn_groups"].as_array().unwrap();
    assert_eq!(
        enemies.len(),
        EXISTING_ACTORS.len() + ELITE.len() + T699_ACTORS + T730_ACTORS + T889_ACTORS
    );
    assert_eq!(groups.len(), EXISTING_GROUPS.len() + CAMPS.len() + 1);
    assert_eq!(
        enemies[..EXISTING_ACTORS.len()]
            .iter()
            .map(|row| row["guid"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        EXISTING_ACTORS.as_slice(),
        "the 28 pre-T332 actors keep their order"
    );
    assert_eq!(
        groups[..EXISTING_GROUPS.len()]
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        EXISTING_GROUPS.as_slice(),
        "the 10 pre-T332 groups keep their order"
    );
    assert_eq!(
        existing_prefix_digest(&field),
        EXISTING_PREFIX_SHA256,
        "the 28 old actors and 10 old groups are a deep-equal prefix"
    );
    assert_eq!(
        enemies[EXISTING_ACTORS.len()..EXISTING_ACTORS.len() + ELITE.len()]
            .iter()
            .map(|row| row["guid"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        ELITE.iter().map(|elite| elite.guid).collect::<Vec<_>>(),
        "the six ELITE actors are appended"
    );
    assert_eq!(
        groups[EXISTING_GROUPS.len()..EXISTING_GROUPS.len() + CAMPS.len()]
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        CAMPS.iter().map(|camp| camp.id).collect::<Vec<_>>(),
        "the three elite groups are appended"
    );
    assert_eq!(
        groups[EXISTING_GROUPS.len() + CAMPS.len()..]
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["mid_highroad_tent_hooligans"],
        "T889 appended the tent group after the elite three"
    );

    // Seven always-on definition-3156 wolves are live: three starter, two
    // level-5 MID-F and two level-11/12 HIGH-F. The elite camps avoid 3156.
    let wolves: Vec<u64> = enemies
        .iter()
        .filter(|row| {
            row["definition_id"].as_u64() == Some(3156)
                && row["quest_ids"]
                    .as_array()
                    .is_some_and(|ids| ids.is_empty())
        })
        .map(|row| row["guid"].as_u64().unwrap())
        .collect();
    assert_eq!(
        wolves,
        [
            131000102001,
            131000102002,
            131000102003,
            131000102013,
            131000102014,
            131000102021,
            131000102022,
        ],
        "three starter + two MID-F + two HIGH-F live wolves"
    );
    for elite in ELITE {
        assert_ne!(
            actor(&field, elite.guid)["definition_id"].as_u64(),
            Some(3156),
            "{} must not use the wolf definition override",
            elite.area
        );
    }
    let quests = read_json(&content_dir().join("quests/adventurer.json"));
    let fighting = quests["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|quest| quest["id"] == 3000009)
        .expect("Fighting the Pack 3000009");
    assert_eq!(fighting["enabled"], true);
    let objective = fighting["objectives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|objective| objective["id"] == 1)
        .expect("the defeat objective");
    assert_eq!(objective["kind"], "defeat");
    assert_eq!(objective["model_id"], 3156);
    assert_eq!(objective["count"], 5, "the quest still requires five");

    // The 14-20 band's shipped values are unchanged.
    let rewards = shipped_rewards();
    let bands = rewards["level_rewards"].as_array().unwrap();
    let values = bands
        .iter()
        .map(|band| {
            (
                band["min_level"].as_u64().unwrap() as u32,
                band["max_level"].as_u64().unwrap() as u32,
                band["coins"].as_u64().unwrap() as u32,
                band["job_xp"].as_u64().unwrap() as u32,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        [
            (1, 2, 2, 5),
            (3, 4, 4, 10),
            (5, 8, 7, 15),
            (9, 13, 12, 25),
            (14, 20, 18, 35),
        ]
    );
    assert_eq!(
        rewards["definition_overrides"].as_array().unwrap().len(),
        1 + T699_ACTORS + T730_ACTORS,
        "the wolf definition override plus one 0/0 override per T699 and T730 quest target"
    );
}

#[test]
fn the_uncovered_reward_level_or_definition_override_does_not_pay() {
    // The elite camps avoid the definition-3156 override entirely.
    let field = shipped_field();
    for elite in ELITE {
        assert_ne!(
            actor(&field, elite.guid)["definition_id"].as_u64(),
            Some(3156),
            "{} must use a level-band definition",
            elite.area
        );
    }

    let fixture = RewardFixture::new(vec![level_twenty_actor(), level_twenty_wolf()]);
    // A valid table with no band covering level 20: the lookup, not the loader,
    // decides the payout.
    fixture.write_rewards(&json!({
        "schema_version": 1,
        "level_rewards": [band_row(1, 13, 12, 25)],
        "definition_overrides": []
    }));
    let at = [1.0, 0.0, 0.0];
    let character = character_at(101, at);
    let mut game = player(&fixture, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();
    let (lethal, _) = kill_one(&mut game, &mut field, &character, LEVEL_TWENTY_ACTOR, 0);
    assert!(
        lethal.changed,
        "the server-authoritative kill still happens"
    );
    assert_eq!(field.health(LEVEL_TWENTY_ACTOR), Some(0));
    assert_eq!(count(&lethal.replies, 38, 19), 0);
    assert_eq!(count(&lethal.replies, 38, 14), 0);
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 0);
    assert_eq!(xp.get(&43), None, "an uncovered level pays no job XP");

    // The shipped definition override still pays 2/5 at level 20.
    fixture.write_rewards(&shipped_rewards());
    let mut game = player(&fixture, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();
    let (lethal, _) = kill_one(&mut game, &mut field, &character, LEVEL_TWENTY_WOLF, 0);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 2, 0, 0, 0].as_slice(),
        "definition 3156 keeps its 2-coin override at level 20"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 5, 0, 0, 0]).as_slice()
    );
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 2);
    assert_eq!(xp.get(&43), Some(&5));
}
