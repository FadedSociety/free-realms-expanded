//! T296: the level 11-14 high field camps and the 14-20 kill-reward band.
//!
//! Content slice. The two new camps sit on audited centres (T415 moved them off
//! the W102 starter centres; `common/field_camp_centres.rs`), the existing
//! `field_roamer` preset and the shipped wolf/shaman definitions.
//! The reward band is one appended row in `field_kill_rewards.json`. No
//! production behaviour, packet or schema changed. The load root and the shared
//! reward helpers are `tests/common/field_reward_fixture.rs`, shared with the
//! T240 payout suite.
//!
//! Client evidence for the existing paths this suite asserts:
//! - 35/2 AddNpc reader `0x00921A10`: the u64 actor id is the first field and
//!   there is no per-actor level field.
//! - 35/5 UpdateHitpoints reader `0x008D6F10`: a u64 actor id and then three
//!   i32 (current, maximum, zero); this suite reads the maximum at bytes 16..20.
//! - 38/19 coin balance: one i32, client constructor `0x0089E670`.
//! - 38/14 job update: six i32, client reader `0x008D86E0`.

use osfr_tools::{
    encounter::{EncounterEngine, EncounterOutcome},
    gameplay::{Gameplay, GameplayState},
    inventory::{EquippedItem, InventoryItem},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs};

#[path = "common/field_camp_centres.rs"]
mod field_camp_centres;
#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "common/field_reward_fixture.rs"]
mod field_reward_fixture;

use field_camp_centres::{placement_masked, verified_centre};
use field_enemy_fixture::{Fixture as EngineFixture, content_dir, content_path, spawned};
use field_reward_fixture::{
    CREDIT_TTL_MS, RewardFixture, attack_one, bank, character_at, kill_one, player, player_state,
    shipped_field_actor, source,
};

#[path = "support/packets.rs"]
mod packets;

use packets::{carrier, count, packet};

/// The four new actors in file order: GUID, authored level, spawn area and the
/// pre-T296 W102 row they reuse. Copied fields and provenance are compared to
/// that row, not to a copied constant; T415 moved the position to an audited
/// centre.
const HIGH: [(u64, u32, &str, u64); 4] = [
    (131000102021, 11, "HIGH-F3", 131000102003),
    (131000102022, 12, "HIGH-F2", 131000102002),
    (131000102023, 13, "HIGH-T3", 131000102012),
    (131000102024, 14, "HIGH-T2", 131000102011),
];

/// The 24 pre-T296 field actors in file order: 12 starter, 4 quest-gated and 8
/// MID-level. Each entry is the GUID and the sha256 of the row's canonical
/// `serde_json` encoding, so an edited, added or removed field fails, not only
/// identity and order. A mismatch names the row; `git diff` names the field.
/// The 8 MID rows are hashed with their placement masked (`placement_masked`):
/// T415 moved them, and the masked values are the pre-T415 rows' digests.
#[rustfmt::skip]
const EXISTING_ACTORS: [(u64, &str); 24] = [
    (100000004154, "663672b99c34e6f5468bd2c646bbf3c9028ce6a2974c9cb6771a8664263a4f5e"),
    (100000004155, "0b6ec28910a2517f33a21567386ab6d403f4f5ba82f93d9601493feb6face326"),
    (5064655448674664449, "a6608fbba6942152cc10ffecb6e86a04d673be5cbf3bb5028b23b242690b4b1e"),
    (5064655448674664450, "42e35b556e6ef7e8c6358775ee13906d519eb86ae78c871a30c1e75535467822"),
    (131000102001, "992fad9793bd47b14b1b3502acad0b7f91329a7064831bd2d111be446c201c76"),
    (131000102002, "3b346cf2ca945162ac07182ec24d1ff70d38e95c9d4a029b205c940d6c009df9"),
    (131000102003, "53721d8e82ad921772a2ad34ecadbef67d97eb4970ab5f5d40069166a3706c17"),
    (131000102004, "848628c88b490b613e9475aa10a6dfc922247771b55067824e9723c8b329c073"),
    (131000102005, "7e02101bac22badc3adfacd808855f3e95a87143e1f32c2f9787e6cf0fa1cfc8"),
    (131000102006, "3f8624d116a932444f6d12723b979417cbd08e81a88305a5229d9fa8366a32aa"),
    (131000102007, "cd111ca73770fb33f32f536e50780f269a1281125046190bf56704ea1dc14b7e"),
    (131000102008, "751bf397a0744b42c0701b86c7cb1f844c2404a372812e8257cb43c74b0b08bb"),
    (131000102009, "57e1958988a21e20f608b4c468c082401344c968ca2688134d054f11b3f7df22"),
    (131000102010, "7b6000ac2cb55616bae0e24c9a65e2dd99cc5acabf6d553085111224bb0eeb24"),
    (131000102011, "147f051194673438471741f7aca493a31fb317b5fdea589af1ecce4009400bab"),
    (131000102012, "a1cd884e4e9a53d4ea36d808b8799bef2c3f22129d62c1ce6b818dd6f00c9236"),
    (131000102013, "1ffc4161d81b86f13a68841343686cb552f5d1e117e2344788c6e3d18fa67dd1"),
    (131000102014, "0de76f7e8d5bdec9c277715ca697c1f30e8e6878d9e4a10c5e632ccc6c613565"),
    (131000102015, "618656411055f6c59e00379290ad5d215903e1be12ba177bb3de91170c054fb4"),
    (131000102016, "eaa0c43e0c6e28f84c9918ef47820c771e81f93853807b107866d09d756045f3"),
    (131000102017, "925a94ac887c3e94feb7e5e8e856806d365a489802563ae00ea1539359490fea"),
    (131000102018, "c76d2bf2c45b2da6ee8ebc4644be054c397f505a734c477c669c08aa4574ec5f"),
    (131000102019, "2fab27c957a63ed49a490a2d62fed9e9b3102aadfdd03e02927b739fd5fb53bb"),
    (131000102020, "da2762a99d74d4c8724f0c8d24558f5e5e1dee68359daf847ccd84697af3f9b0"),
];

/// The eight pre-T296 spawn groups in file order, pinned the same way.
#[rustfmt::skip]
const EXISTING_GROUPS: [(&str, &str); 8] = [
    ("starting_farnum_wolves", "62b096a253c7c3ea3253166888f9fa29274a3eb841e56defc526152c3d292252"),
    ("starting_highroad_hooligans", "a4e71214e32fb677f5cab5c42161635c340536d706f157b25ef75d0ad7e0bc8b"),
    ("starting_queensfields_bixies", "bdd9bf276afde78cd89fe30894333d1fc238d5ea9c9344a849dbfc815f764123"),
    ("starting_wilds_shamans", "73a70feddba0e246f0a2ebc7bd9cc40412e0810b9ef825e5b4823214e852cd92"),
    ("mid_farnum_wolves", "c885b4860b17e7910848b44444bb606807c5595057895d91be64ba4ee9f73caa"),
    ("mid_highroad_hooligans", "56a3902826924ae5cfbb283675783914a2882790b40d69143985ab81bd43f917"),
    ("mid_wilds_shamans", "8342ace34ed3205c69a19794fec3c16902f532e740325d38d70b42d2c4e244ff"),
    ("mid_queensfields_bixies", "61b65abc5a1bd9230dcc3814807268cc7303023e4065c2a0651124206633b488"),
];

/// The four pre-T296 reward bands in file order, pinned by the sha256 of each
/// row's canonical `serde_json` encoding, so an edited value or provenance text
/// fails, not only the four payout numbers.
#[rustfmt::skip]
const EXISTING_BANDS: [&str; 4] = [
    "79b56a8735c90ab706eecf86cf7f3723e7d47168e4249bc2769c33f97814063f",
    "931d59338da3721d075e1038637f4481007a0409e348dc21e1a852fe983e06f4",
    "0f7a25c76e89c9807930c25013b6fef22255dc5e5b4a5d46e2d240d83666ad05",
    "ef5ac778c5192dd5576853362fe9ea56e287c9473c8c46ad1a253a62441fa3b2",
];
/// The appended 14-20 band.
const HIGH_BAND: (u32, u32, u32, u32) = (14, 20, 18, 35);
/// The one pre-T296 definition override (Hooligan Wolf), pinned by the same
/// canonical digest so its values and provenance cannot change unnoticed.
const EXISTING_OVERRIDE: &str = "407b100e2645dbbbeb8b06e1faaebd849933c3c882bf6308bf2e9eba34d79a39";

const HIGH_SHAMAN_13: u64 = 131000102023;
const HIGH_SHAMAN_14: u64 = 131000102024;
const HIGH_SHAMAN_14_HOME: [f32; 3] = verified_centre("HIGH-T2");
const HIGH_SHAMAN_13_HOME: [f32; 3] = verified_centre("HIGH-T3");
const BIXIE_10: u64 = 131000102019;
const BIXIE_10_HOME: [f32; 3] = verified_centre("MID-Q1");
const LEVEL_TWENTY_GUID: u64 = 131000110020;
/// Same-kind spawns a moved camp may name as `reference_spawn_guid`:
/// FreeRealms-Legacy 9e08202 `src/Scripts/Zone/FabledRealms.lua` rows (positions
/// only, no code). Hooligan Wolf 3156-3158 (lines 1899-1901) and 4164-4166
/// (2830-2832), the unnamed `wolf_generic` 2529 (1272); Troll Bone Shaman 3149
/// and 3150 (1892, 1893).
const LEGACY_SAME_KIND: [(&str, &[u64]); 2] = [
    (
        "Hooligan Wolf",
        &[
            100000002529,
            100000003156,
            100000003157,
            100000003158,
            100000004164,
            100000004165,
            100000004166,
        ],
    ),
    ("Troll Bone Shaman", &[100000003149, 100000003150]),
];

fn shipped_field() -> Value {
    serde_json::from_slice(&fs::read(content_path()).unwrap()).unwrap()
}

fn shipped_rewards() -> Value {
    serde_json::from_slice(&fs::read(content_dir().join("field_kill_rewards.json")).unwrap())
        .unwrap()
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

/// The canonical encoding hash a pinned row is compared by.
fn digest(value: &Value) -> String {
    format!("{:x}", Sha256::digest(serde_json::to_vec(value).unwrap()))
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

/// The maximum health from a 35/5 UpdateHitpoints reply. The client reader
/// `0x008D6F10` reads the u64 actor id and then three i32: current health
/// (bytes 12..16), maximum (16..20) and a zero (20..24).
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

fn level_twenty_actor() -> Value {
    json!({
        "guid": LEVEL_TWENTY_GUID,
        "definition_id": 930108,
        "model_id": 3,
        "name_id": 100,
        "name": "Synthetic field reward",
        "texture_alias": "",
        "position": [0.0, 0.0, 0.0],
        // Authored level-one HP; the loader scales it by level 20 (x14.27).
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

fn band_row(min_level: u32, max_level: u32, coins: u32, job_xp: u32) -> Value {
    json!({
        "min_level": min_level,
        "max_level": max_level,
        "coins": coins,
        "job_xp": job_xp,
        "source": source()
    })
}

fn overlap_table() -> Value {
    json!({
        "schema_version": 1,
        "level_rewards": [band_row(1, 2, 2, 5), band_row(2, 3, 4, 10)],
        "definition_overrides": []
    })
}

fn overflow_table() -> Value {
    json!({
        "schema_version": 1,
        "level_rewards": [band_row(1, 2, 1001, 5)],
        "definition_overrides": []
    })
}

fn equip_archer_weapon(state: &mut GameplayState) {
    let instance = 760_000_296;
    state.inventory.items.push(InventoryItem {
        definition_id: 4266,
        tint: 0,
        id: instance,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    state
        .inventory
        .equipment
        .retain(|row| !(row.profile_id == 35 && row.slot == 7));
    state.inventory.equipment.push(EquippedItem {
        profile_id: 35,
        slot: 7,
        item_id: instance,
        category: 7,
    });
}

#[test]
fn the_loader_accepts_the_high_camps_and_exposes_them() {
    let mut engine = EngineFixture::shipped().load();
    let out = engine.on_zone_ready(11, 0).unwrap();
    let spawned = spawned(&out);
    let mut scaled = Vec::new();
    for (guid, level, area, _) in HIGH {
        assert!(
            spawned.contains(&guid),
            "{area} ({guid}) must spawn through the existing 35/2 AddNpc path"
        );
        assert_eq!(
            engine.field_level(guid),
            Some(level),
            "the loader must consume {area}'s authored level"
        );
        assert_eq!(engine.behaviour(guid).unwrap().leash, 12.0);
        let hp = max_health(&out, guid).expect("35/5 HP packet");
        assert!(
            hp > 100,
            "{area} authored 100 HP at level 1; the loader must scale level {level} once, got {hp}"
        );
        scaled.push(hp);
    }
    assert!(
        scaled.windows(2).all(|pair| pair[0] < pair[1]),
        "each higher authored level must raise the scaled HP: {scaled:?}"
    );
}

#[test]
fn the_high_camps_use_verified_w102_centres_and_unique_guids() {
    let field = shipped_field();
    let enemies = field["enemies"].as_array().unwrap();
    let groups = field["spawn_groups"].as_array().unwrap();
    assert_eq!(
        enemies.len(),
        50,
        "24 pre-T296 + 4 HIGH + 6 elite T332 actors + 15 T699 and 1 T730 quest targets"
    );
    assert_eq!(
        groups.len(),
        13,
        "eight pre-T296 + two HIGH + three elite T332 groups"
    );

    let mut guids = BTreeSet::new();
    let mut areas = BTreeSet::new();
    for row in enemies {
        assert!(
            guids.insert(row["guid"].as_u64().unwrap()),
            "duplicate actor GUID {}",
            row["guid"]
        );
        if let Some(area) = row.get("spawn_area") {
            assert!(
                areas.insert(area["id"].as_str().unwrap().to_owned()),
                "duplicate spawn area id {}",
                area["id"]
            );
        }
    }
    assert_eq!(areas.len(), 30, "30 actors carry a spawn area");
    assert_eq!(guids.len(), 50);

    for (index, (guid, level, area, reference_guid)) in HIGH.into_iter().enumerate() {
        let row = actor(&field, guid);
        let reference = actor(&field, reference_guid);
        let w102 = format!("W102-{}", area.strip_prefix("HIGH-").unwrap());
        assert_eq!(row["spawn_area"]["id"], area);
        // T415 moved the camp off the W102 row's centre onto its audited centre.
        assert_eq!(
            position(&row["position"]).map(f32::to_bits),
            verified_centre(area).map(f32::to_bits),
            "{area} must stay on its verified centre"
        );
        assert_eq!(
            position(&row["spawn_area"]["centre"]).map(f32::to_bits),
            verified_centre(area).map(f32::to_bits),
            "{area} centre must equal its position"
        );
        assert_eq!(
            row["spawn_area"]["radius"], reference["spawn_area"]["radius"],
            "{area} copies the {w102} roam radius"
        );
        assert_eq!(
            row["spawn_area"]["count"], reference["spawn_area"]["count"],
            "{area} copies the {w102} actor count"
        );
        assert_eq!(row["level"], level);
        assert_eq!(row["quest_ids"], json!([]), "{guid} must be always-on");
        assert_eq!(row["boss"], false);
        let (_, same_kind) = LEGACY_SAME_KIND
            .iter()
            .find(|(name, _)| row["name"] == *name)
            .unwrap_or_else(|| panic!("{area} has no Legacy kind"));
        assert!(
            same_kind.contains(&row["reference_spawn_guid"].as_u64().unwrap()),
            "{area} must name a same-kind Legacy spawn, not {}",
            row["reference_spawn_guid"]
        );

        // Every field the row claims to copy comes from the named W102 row, so a
        // typo repeated in both the content and a test constant cannot pass.
        for key in [
            "definition_id",
            "model_id",
            "name_id",
            "name",
            "max_health",
            "damage",
            "texture_alias",
            "power_up_table",
        ] {
            assert_eq!(
                row[key], reference[key],
                "{guid} {key} must match the {w102} row it reuses"
            );
        }

        let group_id = if index < 2 {
            "high_farnum_wolves"
        } else {
            "high_wilds_shamans"
        };
        let group = group(&field, group_id);
        assert_eq!(
            group["members"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|member| member.as_u64() == Some(guid))
                .count(),
            1,
            "{guid} must be in {group_id} exactly once"
        );
        let band = group["level_band"].as_array().unwrap();
        assert!(
            band[0].as_u64().unwrap() <= u64::from(level)
                && u64::from(level) <= band[1].as_u64().unwrap(),
            "{guid} level {level} outside {group_id}'s band"
        );
        assert_eq!(group["zone_id"], 0);
        assert_eq!(
            group["region"],
            if index < 2 {
                "Farnum outskirts"
            } else {
                "Lakeshore / northeast Wilds"
            }
        );
        assert_eq!(group["respawn_seconds"], if index < 2 { 120 } else { 150 });
        assert_eq!(group["aggro_range"], 8.0);
        assert_eq!(group["leash_radius"], 12.0);
        assert_eq!(group["behaviour"], "field_roamer");

        let kinds: BTreeSet<&str> = row["source"]
            .as_array()
            .unwrap()
            .iter()
            .map(|source| source["kind"].as_str().unwrap())
            .collect();
        for kind in ["client", "reference", "legacy-data", "reconstructed"] {
            assert!(kinds.contains(kind), "{guid} lost its {kind} provenance");
        }
        let reconstructed = row["source"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap();
        let evidence = reconstructed["evidence"].as_str().unwrap();
        assert!(evidence.starts_with("(R)"));
        assert!(
            evidence.contains(&format!("row {reference_guid}")),
            "{area} must name its source W102 row: {evidence}"
        );
        assert!(
            evidence.contains(&w102),
            "{area} must name the reused {w102} centre: {evidence}"
        );
        assert!(
            evidence.contains(&format!("level {level}")),
            "{area} must record its level policy: {evidence}"
        );
        assert!(
            evidence.contains(&format!("{} s respawn", group["respawn_seconds"])),
            "{area} must record its respawn policy: {evidence}"
        );
        assert!(group["source"].as_array().unwrap().iter().any(|source| {
            source["kind"] == "reconstructed"
                && source["evidence"]
                    .as_str()
                    .is_some_and(|text| text.starts_with("(R)"))
        }));
    }
}

#[test]
fn the_existing_camps_and_rewards_are_unchanged() {
    let field = shipped_field();
    let enemies = field["enemies"].as_array().unwrap();
    let groups = field["spawn_groups"].as_array().unwrap();
    assert_eq!(
        enemies.len(),
        50,
        "24 pre-T296 + 4 HIGH + 6 elite T332 actors + 15 T699 and 1 T730 quest targets"
    );
    assert_eq!(
        groups.len(),
        13,
        "eight pre-T296 + two HIGH + three elite T332 groups"
    );

    // Full equality, not just identity and order: a changed position, level,
    // health, source or group policy fails and names the row.
    for (row, (guid, expected)) in enemies
        .iter()
        .take(EXISTING_ACTORS.len())
        .zip(EXISTING_ACTORS)
    {
        assert_eq!(row["guid"].as_u64(), Some(guid));
        assert_eq!(
            digest(&placement_masked(row)),
            expected,
            "pre-T296 field actor {guid} changed: {row}"
        );
    }
    assert_eq!(
        enemies
            .iter()
            .skip(EXISTING_ACTORS.len())
            .take(HIGH.len())
            .map(|row| row["guid"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        HIGH.iter().map(|(guid, ..)| *guid).collect::<Vec<_>>(),
        "the four HIGH actors remain the first appended actors"
    );
    for (row, (id, expected)) in groups
        .iter()
        .take(EXISTING_GROUPS.len())
        .zip(EXISTING_GROUPS)
    {
        assert_eq!(row["id"].as_str(), Some(id));
        assert_eq!(
            digest(row),
            expected,
            "pre-T296 spawn group {id} changed: {row}"
        );
    }
    assert_eq!(
        groups
            .iter()
            .skip(EXISTING_GROUPS.len())
            .take(2)
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["high_farnum_wolves", "high_wilds_shamans"],
        "the two HIGH groups remain the first appended groups"
    );

    let rewards = shipped_rewards();
    let bands = rewards["level_rewards"].as_array().unwrap();
    assert_eq!(bands.len(), 5);
    let high = &bands[4];
    assert_eq!(
        (
            high["min_level"].as_u64().unwrap() as u32,
            high["max_level"].as_u64().unwrap() as u32,
            high["coins"].as_u64().unwrap() as u32,
            high["job_xp"].as_u64().unwrap() as u32,
        ),
        HIGH_BAND
    );
    for (band, expected) in bands.iter().zip(EXISTING_BANDS) {
        assert_eq!(
            digest(band),
            expected,
            "pre-T296 reward band {}-{} changed: {band}",
            band["min_level"],
            band["max_level"]
        );
    }
    let overrides = rewards["definition_overrides"].as_array().unwrap();
    assert_eq!(
        overrides.len(),
        17,
        "the wolf definition override plus one 0/0 override per T699 and T730 quest target"
    );
    assert_eq!(
        digest(&overrides[0]),
        EXISTING_OVERRIDE,
        "the 3156 definition override changed: {}",
        overrides[0]
    );
}

#[test]
fn a_level_fourteen_kill_pays_the_new_band_once() {
    let fixture = RewardFixture::new(vec![shipped_field_actor(HIGH_SHAMAN_14).0]);
    let at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let character = character_at(101, at);
    let mut game = player(&fixture, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();

    let (lethal, killed_at) = kill_one(&mut game, &mut field, &character, HIGH_SHAMAN_14, 0);
    assert_eq!(field.health(HIGH_SHAMAN_14), Some(0));
    assert_eq!(count(&lethal.replies, 38, 19), 1);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "the level-14 shaman pays the 14-20 band's 18 coins"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice(),
        "and 35 active-job XP"
    );
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 18);
    assert_eq!(xp.get(&43), Some(&35));

    // A replay or a corpse hit changes neither the wallet nor the field.
    let before = bank(&game);
    for now in [killed_at + 700, killed_at + 1_400] {
        let replay = attack_one(&mut game, &mut field, &character, HIGH_SHAMAN_14, now);
        assert!(!replay.changed && replay.replies.is_empty());
    }
    assert_eq!(bank(&game), before);
    let quiet = game
        .tick_shared_effects(&mut field, &character, killed_at + 50)
        .unwrap();
    assert!(!quiet.changed && quiet.replies.is_empty());
    assert_eq!(bank(&game), before, "the killer is not paid twice");
}

#[test]
fn the_active_job_not_the_actor_kind_receives_the_xp() {
    let fixture = RewardFixture::new(vec![shipped_field_actor(HIGH_SHAMAN_14).0]);
    let at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let character = character_at(202, at);
    // Profile 35's initial save has no weapon; give it the Archer starter bow
    // so the production weapon gate admits its basic attack.
    let mut state = player_state(&fixture.base_state(), 35, 0, Some((43, 7)));
    equip_archer_weapon(&mut state);
    let mut game = Gameplay::load(fixture.dir(), Some(state)).unwrap();
    game.initialize_world(&character, 0).unwrap();
    let mut field = game.take_shared_fields().unwrap();

    let (_, _) = kill_one(&mut game, &mut field, &character, HIGH_SHAMAN_14, 0);
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 18);
    assert_eq!(xp.get(&35), Some(&35), "the active job receives the XP");
    assert_eq!(
        xp.get(&43),
        Some(&7),
        "the inactive combat profile is untouched"
    );
    assert_eq!(
        xp.get(&930025),
        None,
        "the defeated actor's kind is not a job profile"
    );
}

#[test]
fn the_level_thirteen_and_ten_controls_keep_the_nine_thirteen_band() {
    let fixture = RewardFixture::new(vec![
        shipped_field_actor(HIGH_SHAMAN_13).0,
        shipped_field_actor(BIXIE_10).0,
    ]);
    let shaman_at = [
        HIGH_SHAMAN_13_HOME[0] + 1.0,
        HIGH_SHAMAN_13_HOME[1],
        HIGH_SHAMAN_13_HOME[2],
    ];
    let bixie_at = [BIXIE_10_HOME[0] + 1.0, BIXIE_10_HOME[1], BIXIE_10_HOME[2]];
    let a_character = character_at(101, shaman_at);
    let b_character = character_at(202, bixie_at);
    let mut a = player(&fixture, 101, 43, shaman_at, 0, None);
    let mut b = player(&fixture, 202, 43, bixie_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();

    let (lethal, _) = kill_one(&mut a, &mut field, &a_character, HIGH_SHAMAN_13, 0);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 12, 0, 0, 0].as_slice(),
        "a level-13 shaman stays on the 9-13 band"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 25, 0, 0, 0]).as_slice()
    );
    assert_eq!(a.state().inventory.coins, 12);
    assert_eq!(a.state().profiles.unwrap().experience.get(&43), Some(&25));

    let (lethal, _) = kill_one(&mut b, &mut field, &b_character, BIXIE_10, 7_000);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 12, 0, 0, 0].as_slice(),
        "the level-10 bixie still pays the 9-13 band"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 25, 0, 0, 0]).as_slice()
    );
    assert_eq!(b.state().inventory.coins, 12);
    assert_eq!(b.state().profiles.unwrap().experience.get(&43), Some(&25));
}

#[test]
fn a_level_twenty_actor_pays_the_new_band() {
    let fixture = RewardFixture::new(vec![level_twenty_actor()]);
    let at = [1.0, 0.0, 0.0];
    let character = character_at(101, at);
    let mut game = player(&fixture, 101, 43, at, 0, None);
    let mut field = game.take_shared_fields().unwrap();

    let (lethal, _) = kill_one(&mut game, &mut field, &character, LEVEL_TWENTY_GUID, 0);
    assert_eq!(
        packet(&lethal.replies, 38, 19),
        [38u8, 0, 19, 0, 18, 0, 0, 0].as_slice(),
        "level 20 is the top of the 14-20 band"
    );
    assert_eq!(
        packet(&lethal.replies, 38, 14),
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice()
    );
    assert_eq!(game.state().inventory.coins, 18);
}

#[test]
fn a_level_outside_the_bands_pays_nothing_without_an_error() {
    let fixture = RewardFixture::new(vec![level_twenty_actor()]);
    // A valid table with no band covering 20: the lookup, not the loader,
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

    let (lethal, _) = kill_one(&mut game, &mut field, &character, LEVEL_TWENTY_GUID, 0);
    assert!(
        lethal.changed,
        "the server-authoritative kill still happens"
    );
    assert_eq!(field.health(LEVEL_TWENTY_GUID), Some(0));
    assert_eq!(count(&lethal.replies, 38, 19), 0);
    assert_eq!(count(&lethal.replies, 38, 14), 0);
    let (coins, xp) = bank(&game);
    assert_eq!(coins, 0);
    assert_eq!(xp.get(&43), None, "an uncovered level pays no job XP");
}

#[test]
fn an_unclaimed_high_camp_credit_expires_or_is_lost_on_leaving_and_pays_nothing() {
    let fixture = RewardFixture::new(vec![shipped_field_actor(HIGH_SHAMAN_14).0]);
    let a_at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let b_at = [
        HIGH_SHAMAN_14_HOME[0] - 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();

    attack_one(&mut a, &mut field, &a_character, HIGH_SHAMAN_14, 0);
    attack_one(&mut b, &mut field, &b_character, HIGH_SHAMAN_14, 0);
    let (_, killed_at) = kill_one(&mut a, &mut field, &a_character, HIGH_SHAMAN_14, 700);
    assert_eq!(a.state().inventory.coins, 18, "the killer is paid at once");

    let before = bank(&b);
    let expired = b
        .tick_shared_effects(&mut field, &b_character, killed_at + CREDIT_TTL_MS + 1)
        .unwrap();
    assert!(!expired.changed && expired.replies.is_empty());
    assert_eq!(bank(&b), before, "an expired credit pays nothing");
    let again = b
        .tick_shared_effects(&mut field, &b_character, killed_at + CREDIT_TTL_MS + 100)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
    assert_eq!(bank(&b), before, "the expired credit was consumed once");
}

#[test]
fn two_players_share_a_high_camp_kill_and_each_is_paid_once() {
    let fixture = RewardFixture::new(vec![shipped_field_actor(HIGH_SHAMAN_14).0]);
    let a_at = [
        HIGH_SHAMAN_14_HOME[0] + 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let b_at = [
        HIGH_SHAMAN_14_HOME[0] - 1.0,
        HIGH_SHAMAN_14_HOME[1],
        HIGH_SHAMAN_14_HOME[2],
    ];
    let a_character = character_at(101, a_at);
    let b_character = character_at(202, b_at);
    let mut a = player(&fixture, 101, 43, a_at, 0, None);
    let mut b = player(&fixture, 202, 43, b_at, 0, None);
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let full_health = field.health(HIGH_SHAMAN_14).unwrap();

    attack_one(&mut a, &mut field, &a_character, HIGH_SHAMAN_14, 0);
    attack_one(&mut b, &mut field, &b_character, HIGH_SHAMAN_14, 0);

    // Checklist 12: each player's own production view holds the shared high
    // actor; the gateway then sends `field.initialization` for every wanted GUID
    // the session did not know yet.
    let interests = field.interest_candidates(false);
    for (game, character) in [(&a, &a_character), (&b, &b_character)] {
        let (wanted, _) = game
            .shared_npc_view(character, &interests, &BTreeSet::new())
            .unwrap();
        assert!(
            wanted.contains(&HIGH_SHAMAN_14),
            "player {} must see the high camp actor",
            character.guid
        );
    }
    let add = field.initialization(HIGH_SHAMAN_14).unwrap();
    assert!(
        add.iter().any(|packet| {
            packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == HIGH_SHAMAN_14.to_le_bytes()
        }),
        "the shared field must yield the high actor's 35/2 AddNpc body"
    );

    let (lethal, killed_at) = kill_one(&mut a, &mut field, &a_character, HIGH_SHAMAN_14, 700);
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
        carrier(38, 14, &[43, 1, 35, 0, 0, 0]).as_slice()
    );
    assert_eq!(b.state().inventory.coins, 18);
    assert_eq!(b.state().profiles.unwrap().experience.get(&43), Some(&35));
    let paid = bank(&b);
    let again = b
        .tick_shared_effects(&mut field, &b_character, killed_at + 100)
        .unwrap();
    assert!(!again.changed && again.replies.is_empty());
    assert_eq!(bank(&b), paid, "each player is paid once");
    assert_eq!(field.health(HIGH_SHAMAN_14), Some(0), "one shared kill");

    // The group's 150 s timer respawns the actor, not one millisecond early.
    let players = vec![
        a.field_participant(&a_character).unwrap(),
        b.field_participant(&b_character).unwrap(),
    ];
    a.tick_shared_motion(&mut field, &players, killed_at + 149_900, 100)
        .unwrap();
    assert_eq!(field.health(HIGH_SHAMAN_14), Some(0));
    a.tick_shared_motion(&mut field, &players, killed_at + 150_100, 100)
        .unwrap();
    assert_eq!(field.health(HIGH_SHAMAN_14), Some(full_health));
    assert_eq!(field.respawn_at(HIGH_SHAMAN_14), None);
}

#[test]
fn the_band_table_is_sorted_non_overlapping_and_bounded() {
    let rewards = shipped_rewards();
    let bands = rewards["level_rewards"].as_array().unwrap();
    let mut previous_max = 0;
    for band in bands {
        let min = band["min_level"].as_u64().unwrap() as u32;
        let max = band["max_level"].as_u64().unwrap() as u32;
        let coins = band["coins"].as_u64().unwrap() as u32;
        let job_xp = band["job_xp"].as_u64().unwrap() as u32;
        // The loader allows an uncovered level and pays nothing; the shipped
        // table's exact rows are pinned by `the_existing_camps_and_rewards_are_unchanged`.
        assert!(
            min > previous_max,
            "bands must be sorted and non-overlapping"
        );
        assert!(min >= 1 && min <= max && max <= 20);
        assert!(coins <= 1000 && job_xp <= 1000);
        assert!(!band["source"].as_array().unwrap().is_empty());
        previous_max = max;
    }

    // A synthetic overlapping or overflowing band is refused at load, before
    // any actor exists or any payout can be computed.
    let fixture = RewardFixture::new(vec![shipped_field_actor(HIGH_SHAMAN_14).0]);
    for bad in [overlap_table(), overflow_table()] {
        fixture.write_rewards(&bad);
        assert!(
            EncounterEngine::load(fixture.dir(), None).is_err(),
            "a malformed band must fail the engine load"
        );
    }
    fixture.write_rewards(&shipped_rewards());
    assert!(EncounterEngine::load(fixture.dir(), None).is_ok());
}

#[test]
fn a_malformed_high_camp_or_band_is_refused_atomically() {
    let fixture = EngineFixture::copied();
    let mut engine = fixture.load();
    let before = engine.field_teleport_targets();
    let behaviour = engine.behaviour(HIGH_SHAMAN_14).cloned().unwrap();
    let original = shipped_field();
    let mutations: [(&str, fn(&mut Value)); 3] = [
        ("duplicate member", |field| {
            let group = field["spawn_groups"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|group| group["id"] == "high_wilds_shamans")
                .unwrap();
            group["members"] = json!([HIGH_SHAMAN_13, HIGH_SHAMAN_13]);
        }),
        ("level outside band", |field| {
            let row = field["enemies"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["guid"] == HIGH_SHAMAN_14)
                .unwrap();
            row["level"] = json!(15);
        }),
        ("position off its centre", |field| {
            let row = field["enemies"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["guid"] == HIGH_SHAMAN_14)
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
            engine.behaviour(HIGH_SHAMAN_14),
            Some(&behaviour),
            "{label} changed the live behaviour"
        );
    }

    // The malformed band is likewise a load refusal, not a payout-time panic.
    let rewards = RewardFixture::new(vec![shipped_field_actor(HIGH_SHAMAN_14).0]);
    rewards.write_rewards(&overlap_table());
    assert!(EncounterEngine::load(rewards.dir(), None).is_err());
    rewards.write_rewards(&shipped_rewards());
    assert!(EncounterEngine::load(rewards.dir(), None).is_ok());
}
