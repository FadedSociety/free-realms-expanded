use osfr_tools::opening_abilities::OpeningAbilities;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[allow(dead_code)]
#[path = "../src/bin/osfr-bot.rs"]
mod bot_client;

#[test]
#[ignore = "requires W101's owned scratch gateway and reference manifest"]
fn scratch_warrior_level_five_bot_reads_kick_and_spinning_blade_bar() {
    use std::time::Duration;
    let reference = PathBuf::from(std::env::var_os("OSFR_W101_REFERENCE").unwrap());
    let manifest: Value =
        serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
    let config = bot_client::BotConfig {
        gateway: std::env::var("OSFR_W101_GATEWAY").unwrap().parse().unwrap(),
        name: String::new(),
        connection_id: std::process::id(),
        guid: manifest["initial_state"]["guid"].as_u64().unwrap(),
        ticket: osfr_tools::login::DEFAULT_GATEWAY_TICKET.to_owned(),
        version: manifest["client_version"].as_str().unwrap().to_owned(),
        expect_timeout_ms: 10000,
        quiet: true,
    };
    let mut bot = bot_client::Bot::connect(config).unwrap();
    bot.login().unwrap();
    bot.run_step(
        &bot_client::parse_step("chat realm level 5")
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let timeout = Duration::from_secs(10);
    let mut definitions = Vec::new();
    for (id, name) in [(62004_i32, 24540), (62005, 4908)] {
        let packet = bot
            .wait_for(timeout, |p| {
                p.starts_with(&[36, 0, 13, 0]) && p[4..8] == id.to_le_bytes()
            })
            .unwrap();
        assert_eq!(i32::from_le_bytes(packet[10..14].try_into().unwrap()), name);
        definitions.push(json!({"definition_id":id,"name_id":name}));
    }
    let packet = bot
        .wait_for(timeout, |p| p.starts_with(&[36, 0, 5, 0]))
        .unwrap();
    assert_eq!(i32::from_le_bytes(packet[4..8].try_into().unwrap()), 32);
    assert_eq!(i32::from_le_bytes(packet[8..12].try_into().unwrap()), 8);
    let mut at = 12;
    let mut ids = Vec::new();
    for _ in 0..8 {
        let kind = i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
        assert!(kind == 0 || kind == 3);
        ids.push(if kind == 3 {
            i32::from_le_bytes(packet[at + 4..at + 8].try_into().unwrap())
        } else {
            0
        });
        at += if kind == 3 { 41 } else { 4 };
    }
    assert_eq!(at, packet.len());
    assert_eq!(ids, [4895, 62004, 62005, 0, 0, 0, 0, 0]);
    let receipt = json!({"profile":32,"level":5,"definitions":definitions,"toolbar_definition_ids":ids,"toolbar_bytes":packet.len(),"full_toolbar_decoded":true});
    fs::write(
        std::env::var_os("OSFR_W101_BOT_RECEIPT").unwrap(),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
    println!("W101 bot: Warrior 5 toolbar [4895, 62004 Kick, 62005 Spinning Blade, 0, 0, 0, 0, 0]");
    bot.disconnect();
}

// Independent expected levels: changing a content lock alone must fail this test.
#[test]
fn shipped_job_roster_unlocks_each_skill_at_its_declared_ladder() {
    let mut fixture = Fixture::new();
    fixture.content["basics"] = json!([11, 32, 12, 2, 43, 35].map(
        |profile| json!({"profile_id":profile,"definition_id":4895,"name_id":1,
            "description_id":0,"icon_id":1,"weapon_name_id":1})
    ));
    fixture.write();
    let jobs: Value =
        serde_json::from_str(include_str!("../data/content/job_abilities.json")).unwrap();
    let abilities = load_jobs(&fixture, &jobs).unwrap();
    for (profile, expected) in [
        (11, vec![(1, 1), (2, 1), (3, 5), (4, 10), (5, 15)]),
        (32, vec![(1, 1), (2, 5), (3, 10), (4, 15)]),
        (12, vec![(1, 1), (2, 5), (3, 10), (4, 15)]),
        (2, vec![(1, 1), (2, 5), (3, 10), (4, 15)]),
        (43, vec![(1, 1), (2, 5), (3, 10)]),
        (35, vec![(1, 1), (2, 5), (3, 10), (4, 15), (5, 20)]),
    ] {
        for (slot, minimum) in expected {
            let ability = abilities.job_slot(profile, slot, minimum).unwrap();
            assert_eq!(ability.min_level, minimum, "profile {profile}, slot {slot}");
            let id = ability.definition_id;
            assert!(abilities.job_slot(profile, slot, minimum - 1).is_none());
            for level in [minimum - 1, minimum, 20] {
                let packets = abilities.activation(profile, level).unwrap();
                let defined = packets.iter().any(|p| {
                    p.starts_with(&[36, 0, 13, 0])
                        && i32::from_le_bytes(p[4..8].try_into().unwrap()) == id
                });
                assert_eq!(
                    defined,
                    level >= minimum,
                    "profile {profile}, slot {slot}, level {level}"
                );
                let bar = packets.last().unwrap();
                let mut at = 12;
                for index in 0..8 {
                    let kind = i32::from_le_bytes(bar[at..at + 4].try_into().unwrap());
                    if index == slot {
                        assert_eq!(kind == 3, level >= minimum);
                    }
                    at += if kind == 3 { 41 } else { 4 };
                }
                assert_eq!(at, bar.len());
            }
        }
    }
}

#[test]
fn full_roster_preserves_campaigns_with_only_some_basic_attacks() {
    let fixture = Fixture::new(); // Brawler basic only
    let jobs: Value =
        serde_json::from_str(include_str!("../data/content/job_abilities.json")).unwrap();
    let abilities = load_jobs(&fixture, &jobs).unwrap();
    assert!(abilities.job_slot(43, 1, 1).is_some());
    assert!(abilities.job_slot(32, 1, 20).is_none());
    let bar = abilities.activation(32, 20).unwrap();
    assert_eq!(bar.len(), 1);
    assert_eq!(&bar[0][12..], &[0; 32]);
}
struct Fixture {
    path: PathBuf,
    content: Value,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-abilities-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let content = json!({"schema_version":1,"basics":[{"profile_id":43,"definition_id":1234,"name_id":2301,"description_id":2302,"icon_id":2303,"weapon_name_id":2401}]});
        let f = Self { path, content };
        f.write();
        f
    }
    fn write(&self) {
        fs::write(
            self.path.join("opening-abilities.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
    }
    fn load(&self) -> OpeningAbilities {
        OpeningAbilities::load(&self.path).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("opening-abilities.json"));
        let _ = fs::remove_file(self.path.join("job_abilities.json"));
        let _ = fs::remove_dir(&self.path);
    }
}

// (R) Synthetic levels exercise the shared gate for every combat profile;
// these are not historical unlock claims for Ninja, Brawler or Archer.
fn job_fixture(profile: i32) -> (Fixture, Value) {
    let mut fixture = Fixture::new();
    fixture.content["basics"][0]["profile_id"] = json!(profile);
    fixture.write();
    let slots: Vec<_> = [1, 5, 10, 15]
        .into_iter()
        .enumerate()
        .map(|(index, level)| {
            json!({"slot": index + 1, "definition_id": 2000 + index,
                "name_id": 3000 + index, "icon_id": 4000 + index,
                "heal": 1, "cooldown_ms": 1000, "min_level": level})
        })
        .collect();
    let jobs = json!({"schema_version": 1,
        "jobs": [{"profile_id": profile, "slots": slots}]});
    (fixture, jobs)
}

fn load_jobs(fixture: &Fixture, jobs: &Value) -> std::io::Result<OpeningAbilities> {
    let path = fixture.path.join("job_abilities.json");
    fs::write(&path, serde_json::to_vec(jobs).unwrap()).unwrap();
    let mut abilities = fixture.load();
    abilities.load_job_abilities(&path)?;
    Ok(abilities)
}

fn assert_job_levels(profile: i32) {
    let (fixture, jobs) = job_fixture(profile);
    let abilities = load_jobs(&fixture, &jobs).unwrap();
    for level in 0..=20 {
        let packets = abilities.activation(profile, level).unwrap();
        let definitions: Vec<_> = packets[..packets.len() - 1]
            .iter()
            .map(|packet| i32::from_le_bytes(packet[4..8].try_into().unwrap()))
            .collect();
        // Independently walk the native variable-length slot encoding, including
        // empty slots, so a gate on definitions alone cannot pass this test.
        let toolbar = packets.last().unwrap();
        let mut offset = 12;
        for slot in 0..8 {
            let kind = i32::from_le_bytes(toolbar[offset..offset + 4].try_into().unwrap());
            let unlocked = slot == 0 || (slot <= 4 && level >= [1, 5, 10, 15][slot - 1]);
            assert_eq!(
                kind,
                if unlocked { 3 } else { 0 },
                "profile {profile}, level {level}, slot {slot}"
            );
            offset += if kind == 3 { 41 } else { 4 };
        }
        assert_eq!(offset, toolbar.len());
        for (index, minimum) in [1, 5, 10, 15].into_iter().enumerate() {
            let unlocked = level >= minimum;
            assert_eq!(definitions.contains(&(2000 + index as i32)), unlocked);
            assert_eq!(
                abilities
                    .job_slot(profile, index as i32 + 1, level)
                    .is_some(),
                unlocked
            );
            assert!(
                abilities
                    .job_slot(profile + 1000, index as i32 + 1, 20)
                    .is_none()
            );
            if minimum > 1 {
                assert!(abilities.unlocks_between(profile, minimum - 1, minimum));
            }
        }
    }
    assert!(!abilities.unlocks_between(profile, 1, 4));
    assert!(!abilities.unlocks_between(profile, 15, 20));
}

#[test]
fn medic_level_boundaries() {
    assert_job_levels(11);
}
#[test]
fn ninja_level_boundaries() {
    assert_job_levels(2);
}
#[test]
fn warrior_level_boundaries() {
    assert_job_levels(32);
}
#[test]
fn wizard_level_boundaries() {
    assert_job_levels(12);
}
#[test]
fn brawler_level_boundaries() {
    assert_job_levels(43);
}
#[test]
fn archer_level_boundaries() {
    assert_job_levels(35);
}

#[test]
fn missing_level_lock_is_rejected_instead_of_unlocking_at_level_one() {
    let (fixture, mut jobs) = job_fixture(11);
    jobs["jobs"][0]["slots"][1]
        .as_object_mut()
        .unwrap()
        .remove("min_level");
    assert!(load_jobs(&fixture, &jobs).is_err());
}

#[test]
fn invalid_level_locks_are_rejected() {
    let (fixture, jobs) = job_fixture(11);
    for level in [
        json!(0),
        json!(21),
        json!(-1),
        json!(1.5),
        json!(null),
        json!("5"),
        json!(true),
    ] {
        let mut bad = jobs.clone();
        bad["jobs"][0]["slots"][1]["min_level"] = level;
        assert!(load_jobs(&fixture, &bad).is_err());
    }
}
fn vectors(name: &str) -> Vec<Vec<u8>> {
    let data: Value = serde_json::from_str(include_str!(
        "fixtures/opening-ability-community-vectors.json"
    ))
    .unwrap();
    data["vectors"][name]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            s.as_str()
                .unwrap()
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect()
        })
        .collect()
}

#[test]
fn basic_definition_and_toolbar_equal_independent_community_serializers() {
    let f = Fixture::new();
    assert_eq!(
        f.load().activation(43, 1).unwrap(),
        vectors("synthetic_basic")
    );
}

#[test]
fn noncombat_class_clears_all_eight_slots_and_invalid_profile_fails() {
    let f = Fixture::new();
    let abilities = f.load();
    assert_eq!(
        abilities.activation(1, 1).unwrap(),
        vectors("synthetic_empty")
    );
    assert!(abilities.activation(0, 1).is_err());
    assert!(abilities.activation(-1, 1).is_err());
}

#[test]
fn loaded_metadata_rejects_invalid_fields_duplicates_and_oversized_content() {
    let mut f = Fixture::new();
    let original = f.content.clone();
    for field in [
        "profile_id",
        "definition_id",
        "icon_id",
        "weapon_name_id",
        "weapon_class",
    ] {
        f.content = original.clone();
        f.content["basics"][0][field] = json!(0);
        f.write();
        assert!(OpeningAbilities::load(&f.path).is_err(), "{field}");
    }
    f.content = original.clone();
    f.content["basics"]
        .as_array_mut()
        .unwrap()
        .push(original["basics"][0].clone());
    f.write();
    assert!(OpeningAbilities::load(&f.path).is_err());
    f.content = original;
    f.content["schema_version"] = json!(2);
    f.write();
    assert!(OpeningAbilities::load(&f.path).is_err());
    fs::write(f.path.join("opening-abilities.json"), vec![b' '; 65537]).unwrap();
    assert!(OpeningAbilities::load(&f.path).is_err());
}

#[test]
fn returned_packets_and_clones_do_not_mutate_shared_metadata() {
    let f = Fixture::new();
    let abilities = f.load();
    let clone = abilities.clone();
    let mut replies = clone.activation(43, 1).unwrap();
    replies[0][10] ^= 1;
    assert_eq!(
        abilities.activation(43, 1).unwrap(),
        vectors("synthetic_basic")
    );
    assert_eq!(clone.activation(43, 1).unwrap(), vectors("synthetic_basic"));
}

#[test]
#[ignore = "requires OSFR_OPENING_CONTENT and private compiled community serializer fixtures"]
fn private_opening_hotbars_equal_compiled_community_packets() {
    let directory = PathBuf::from(std::env::var("OSFR_OPENING_CONTENT").expect("opening content"));
    let abilities = OpeningAbilities::load(&directory).unwrap();
    for profile in [43, 11, 35] {
        let expected: Vec<_> = (0..2)
            .map(|i| {
                fs::read(
                    directory
                        .join("opening-ability-oracle")
                        .join(format!("{profile}-{i}.bin")),
                )
                .unwrap()
            })
            .collect();
        assert_eq!(
            abilities.activation(profile, 1).unwrap(),
            expected,
            "profile{profile}"
        );
    }
}
