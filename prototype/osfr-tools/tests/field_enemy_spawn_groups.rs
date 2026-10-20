use osfr_tools::encounter::{EncounterEngine, EncounterOutcome, Mode};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const GUID: u64 = 131000102001;
static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t014-{}-{}",
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
        let other = json!({"guid":200,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic wolf","texture_alias":"","position":[0,0,0],"max_health":100,"damage":7,"boss":false});
        fs::write(
            dir.join("encounters.json"),
            json!({
                "schema_version":1,
                "template_file":"npc.bin",
                "template_sha256":format!("{:x}",Sha256::digest(&template)),
                "world":zone,
                "world_enemies":[],
                "dungeons":[{
                    "id":411,
                    "title_id":12,
                    "description_id":13,
                    "objective_id":14,
                    "zone":cave,
                    "guide":null,
                    "enemies":[other],
                    "reward_coins":0,
                    "reward_items":[]
                }]
            })
            .to_string(),
        )
        .unwrap();
        fs::copy(
            content_dir().join("field_enemies.json"),
            dir.join("field_enemies.json"),
        )
        .unwrap();
        Self(dir)
    }

    fn load(&self) -> EncounterEngine {
        let mut e = EncounterEngine::load(&self.0, None).unwrap();
        // Production loads presets before the actors that name them.
        e.load_behaviours(&content_dir().join("behaviours.json"))
            .unwrap();
        e.load_field_enemies(&self.0.join("field_enemies.json"))
            .unwrap();
        e
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn at(e: &EncounterEngine, guid: u64) -> [f32; 3] {
    e.field_teleport_targets()
        .into_iter()
        .find(|target| target.guid == guid)
        .unwrap()
        .position
}

fn horizontal_radius(position: [f32; 3], home: [f32; 3]) -> f32 {
    (position[0] - home[0]).hypot(position[2] - home[2])
}

fn hp(out: &EncounterOutcome, guid: u64) -> Option<(u32, u32)> {
    out.replies
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 5, 0]) && packet[4..12] == guid.to_le_bytes())
        .map(|packet| {
            (
                u32::from_le_bytes(packet[12..16].try_into().unwrap()),
                u32::from_le_bytes(packet[16..20].try_into().unwrap()),
            )
        })
}

fn spawned(out: &EncounterOutcome) -> BTreeSet<u64> {
    out.replies
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet.len() >= 12)
        .map(|packet| u64::from_le_bytes(packet[4..12].try_into().unwrap()))
        .collect()
}

#[test]
fn starting_regions_have_grouped_reconstructed_placements_and_presets() {
    let field: Value =
        serde_json::from_slice(&fs::read(content_dir().join("field_enemies.json")).unwrap())
            .unwrap();
    let behaviours: Value =
        serde_json::from_slice(&fs::read(content_dir().join("behaviours.json")).unwrap()).unwrap();
    let preset = &behaviours["presets"]["field_roamer"];
    assert_eq!(preset["mode"], "melee");
    assert_eq!(preset["leash"], 12.0);
    assert_eq!(preset["reach"], 2.5);

    let groups = field["spawn_groups"].as_array().unwrap();
    let expected = [
        ("starting_farnum_wolves", "Farnum outskirts", "W102-F", 45),
        (
            "starting_highroad_hooligans",
            "Highroad outskirts",
            "W102-H",
            60,
        ),
        (
            "starting_queensfields_bixies",
            "Sanctuary / Queensfields",
            "W102-Q",
            45,
        ),
        (
            "starting_wilds_shamans",
            "Lakeshore / northeast Wilds",
            "W102-T",
            60,
        ),
    ];
    assert!(groups.len() >= expected.len());

    let enemies = field["enemies"].as_array().unwrap();
    let mut grouped = BTreeSet::new();
    for (group, (id, region, area_prefix, respawn)) in groups.iter().zip(expected) {
        assert_eq!(group["id"], id);
        assert_eq!(group["region"], region);
        assert_eq!(group["zone_id"], 0);
        assert_eq!(group["behaviour"], "field_roamer");
        assert_eq!(group["respawn_seconds"], respawn);
        assert_eq!(group["aggro_range"], 8.0);
        assert_eq!(group["leash_radius"], 12.0);
        assert!(group["source"].as_array().is_some_and(|sources| {
            sources.iter().any(|source| {
                source["kind"] == "reconstructed"
                    && source["evidence"]
                        .as_str()
                        .is_some_and(|text| text.starts_with("(R)"))
            })
        }));
        let members = group["members"].as_array().unwrap();
        assert_eq!(members.len(), 3);
        for member in members {
            let guid = member.as_u64().unwrap();
            assert!(grouped.insert(guid), "duplicate grouped GUID {guid}");
            let actor = enemies.iter().find(|actor| actor["guid"] == guid).unwrap();
            assert_eq!(actor["quest_ids"], json!([]));
            assert!(
                actor["spawn_area"]["id"]
                    .as_str()
                    .is_some_and(|area| area.starts_with(area_prefix))
            );
            assert!(actor["position"].as_array().is_some_and(|p| p.len() == 3));
            let sources = actor["source"].as_array().unwrap();
            for kind in ["client", "reference", "legacy-data", "reconstructed"] {
                assert!(sources.iter().any(|source| source["kind"] == kind));
            }
            assert!(sources.iter().any(|source| {
                source["kind"] == "reconstructed"
                    && source["evidence"]
                        .as_str()
                        .is_some_and(|text| text.starts_with("(R)"))
            }));
        }
    }
    assert_eq!(grouped.len(), 12);
    // Every always-on actor belongs to a group. T245's mid-level camps are
    // appended groups too; this test pins the four starter groups above.
    let all_grouped: BTreeSet<u64> = groups
        .iter()
        .flat_map(|group| group["members"].as_array().unwrap())
        .map(|member| member.as_u64().unwrap())
        .collect();
    for actor in enemies
        .iter()
        .filter(|actor| actor["quest_ids"] == json!([]))
    {
        assert!(
            all_grouped.contains(&actor["guid"].as_u64().unwrap()),
            "{} is always-on but ungrouped",
            actor["guid"]
        );
    }
}

#[test]
fn group_spawns_roam_kill_and_respawn_on_the_authored_timer() {
    let fixture = Fixture::new();
    let mut e = fixture.load();
    let spawned = spawned(&e.on_zone_ready(11, 0).unwrap());
    assert!(spawned.len() >= 12);
    for guid in 131000102001..=131000102012 {
        assert!(spawned.contains(&guid), "starter {guid} must still spawn");
    }
    assert!(spawned.contains(&GUID));
    let behaviour = e.behaviour(GUID).unwrap();
    assert_eq!(behaviour.mode, Mode::Melee);
    assert_eq!(behaviour.leash, 12.0);

    let home = at(&e, GUID);
    let observer = [home[0] + 100.0, home[1], home[2]];
    let mut moved = false;
    for t in 0..100 {
        e.tick(observer, 11, t * 100).unwrap();
        let position = at(&e, GUID);
        let radius = horizontal_radius(position, home);
        moved |= radius > 0.5;
        assert!(radius <= 6.001, "roamed {radius} m from its 6 m disc");
    }
    assert!(moved, "the group member must leave its exact spawn point");

    let target = at(&e, GUID);
    let mut killed_at = None;
    let mut death = None;
    for now in [20_000_u64, 20_700, 21_400] {
        let out = e
            .attack(GUID, [target[0] + 1.0, target[1], target[2]], 11, 0, now)
            .unwrap();
        death = Some(out);
        if !e
            .field_teleport_targets()
            .iter()
            .any(|enemy| enemy.guid == GUID)
        {
            killed_at = Some(now);
            break;
        }
    }
    let killed_at = killed_at.expect("three basic attacks must defeat the level-1 wolf");
    let death = death.unwrap();
    assert!(death.replies.iter().any(|packet| {
        packet.starts_with(&[35, 0, 3, 0]) && packet[6..14] == GUID.to_le_bytes()
    }));

    let before = e
        .tick(
            [target[0] + 1.0, target[1], target[2]],
            11,
            killed_at + 44_999,
        )
        .unwrap();
    assert!(
        !e.field_teleport_targets()
            .iter()
            .any(|enemy| enemy.guid == GUID),
        "the group timer must not expire one millisecond early"
    );
    assert!(!before.replies.iter().any(|packet| {
        (packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == GUID.to_le_bytes())
            || (packet.starts_with(&[35, 0, 5, 0]) && packet[4..12] == GUID.to_le_bytes())
    }));
    let at_respawn = e
        .tick(
            [target[0] + 1.0, target[1], target[2]],
            11,
            killed_at + 45_100,
        )
        .unwrap();
    assert_eq!(hp(&at_respawn, GUID), Some((100, 100)));
    assert!(at_respawn.replies.iter().any(|packet| {
        packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == GUID.to_le_bytes()
    }));
    assert_eq!(at(&e, GUID), home);
}

#[test]
fn malformed_group_reload_is_atomic() {
    let fixture = Fixture::new();
    let mut e = fixture.load();
    let before = e
        .field_teleport_targets()
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect::<Vec<_>>();
    let behaviour = e.behaviour(GUID).cloned().unwrap();

    let path = fixture.0.join("field_enemies.json");
    let mut field: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    field["spawn_groups"][0]["members"] = json!([GUID, GUID]);
    fs::write(&path, field.to_string()).unwrap();

    assert!(e.reload_field_enemies(&path).is_err());
    let after = e
        .field_teleport_targets()
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect::<Vec<_>>();
    assert_eq!(after, before);
    assert_eq!(e.behaviour(GUID), Some(&behaviour));
}
