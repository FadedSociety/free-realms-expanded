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

fn shipped_roster() -> OpeningAbilities {
    let mut fixture = Fixture::new();
    fixture.content["basics"] = json!([11, 32, 12, 2, 43, 35].map(
        |profile| json!({"profile_id":profile,"definition_id":4895,"name_id":1,
            "description_id":0,"icon_id":1,"weapon_name_id":1,"weapon_definition_id":1})
    ));
    fixture.write();
    let jobs: Value =
        serde_json::from_str(include_str!("../data/content/job_abilities.json")).unwrap();
    load_jobs(&fixture, &jobs).unwrap()
}

// (row, client name id, unlock level) of every skill the bar can show; a
// later skill on a row replaces the earlier one. The name ids come from the
// client locale, not from job_abilities.json, so a swapped skill fails here.
// Ninja's Shadow Minion (24516) is unavailable and is never shown.
const COMBAT_LADDERS: [(i32, &[(i32, i32, u32)]); 6] = [
    (
        11,
        &[
            (1, 442456, 1),
            (2, 3784, 1),
            (3, 3810, 5),
            (3, 24061, 10),
            (1, 3800, 15),
        ],
    ),
    (
        32,
        &[(1, 24540, 1), (2, 4908, 5), (3, 24694, 10), (2, 24706, 15)],
    ),
    (
        12,
        &[(1, 17071, 1), (2, 35492, 5), (3, 23679, 10), (3, 38016, 15)],
    ),
    (2, &[(1, 3085, 1), (2, 3086, 5), (3, 3855, 15)]),
    (
        43,
        &[(1, 24133, 1), (2, 15916, 5), (3, 24097, 10), (3, 21374, 15)],
    ),
    (
        35,
        &[
            (1, 24364, 1),
            (2, 4830, 5),
            (3, 24397, 10),
            (2, 20015, 15),
            (3, 24398, 20),
        ],
    ),
];

#[test]
fn every_job_bar_has_exactly_the_unlocked_ladder_at_every_level() {
    let abilities = shipped_roster();
    for (profile, ladder) in COMBAT_LADDERS {
        for level in 1..=20 {
            let bar = abilities.job_bar(profile, level);
            // The client draws the basic and three job rows (ScriptsBase.lua:5215):
            // rows 4-7 stay empty, filled rows leave no blank button before a
            // skill, and an unavailable skill is never shown.
            assert!(bar[3..].iter().all(Option::is_none), "profile {profile}");
            assert!(
                bar.windows(2)
                    .all(|pair| pair[0].is_some() || pair[1].is_none()),
                "profile {profile}, level {level}: blank button before a skill"
            );
            assert!(bar.iter().flatten().all(|ability| !ability.unavailable));
            for slot in 1..=7 {
                let expected = ladder
                    .iter()
                    .filter(|(row, _, minimum)| *row == slot && *minimum <= level)
                    .max_by_key(|(_, _, minimum)| *minimum)
                    .map(|(_, name, _)| *name);
                assert_eq!(
                    bar[slot as usize - 1].map(|ability| ability.name_id),
                    expected,
                    "profile {profile}, level {level}, row {slot}"
                );
            }

            // The wire bar must match the model: one basic, seven job slots,
            // a definition for every unlocked skill and an empty locked slot.
            let packets = abilities.activation(profile, level).unwrap();
            let mut expected_definitions = vec![4895];
            expected_definitions.extend(bar.iter().flatten().map(|a| a.definition_id));
            let definitions: Vec<i32> = packets[..packets.len() - 1]
                .iter()
                .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
                .collect();
            assert_eq!(definitions, expected_definitions);

            let toolbar = packets.last().unwrap();
            assert_eq!(
                i32::from_le_bytes(toolbar[4..8].try_into().unwrap()),
                profile
            );
            assert_eq!(i32::from_le_bytes(toolbar[8..12].try_into().unwrap()), 8);
            let mut at = 12;
            assert_eq!(
                i32::from_le_bytes(toolbar[at..at + 4].try_into().unwrap()),
                3
            );
            assert_eq!(
                i32::from_le_bytes(toolbar[at + 4..at + 8].try_into().unwrap()),
                4895
            );
            at += 41;
            for (entry, expected) in bar.iter().enumerate() {
                let kind = i32::from_le_bytes(toolbar[at..at + 4].try_into().unwrap());
                match expected {
                    Some(ability) => {
                        assert_eq!(kind, 3, "profile {profile}, level {level}, slot {entry}");
                        assert_eq!(
                            i32::from_le_bytes(toolbar[at + 4..at + 8].try_into().unwrap()),
                            ability.definition_id
                        );
                        at += 41;
                    }
                    None => {
                        assert_eq!(kind, 0, "profile {profile}, level {level}, slot {entry}");
                        at += 4;
                    }
                }
            }
            assert_eq!(at, toolbar.len());
        }
    }
}

// Independent expected (row, level): changing a content lock alone must fail
// this test. Each skill takes its row at its own level, replacing any earlier one.
#[test]
fn shipped_job_roster_unlocks_each_skill_at_its_declared_ladder() {
    let abilities = shipped_roster();
    for (profile, expected) in [
        (11, vec![(1, 1), (2, 1), (3, 5), (3, 10), (1, 15)]),
        (32, vec![(1, 1), (2, 5), (3, 10), (2, 15)]),
        (12, vec![(1, 1), (2, 5), (3, 10), (3, 15)]),
        (2, vec![(1, 1), (2, 5), (3, 15)]), // Shadow Minion (row 3, 10) is unavailable
        (43, vec![(1, 1), (2, 5), (3, 10), (3, 15)]), // W125: 2009 Brawler guide levels
        (35, vec![(1, 1), (2, 5), (3, 10), (2, 15), (3, 20)]), // W125: 2009 Archer guide levels
    ] {
        for (slot, minimum) in expected {
            let ability = abilities.job_slot(profile, slot, minimum).unwrap();
            assert_eq!(ability.min_level, minimum, "profile {profile}, slot {slot}");
            let id = ability.definition_id;
            for level in [minimum - 1, minimum] {
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
                        let shown = kind == 3
                            && i32::from_le_bytes(bar[at + 4..at + 8].try_into().unwrap()) == id;
                        assert_eq!(shown, level >= minimum, "profile {profile}, row {slot}");
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
    let name = |row, level| abilities.job_slot(43, row, level).map(|a| a.name_id);
    assert_eq!(name(1, 1), Some(24133)); // Leg Sweep, the level-1 Brawler skill (W125)
    assert_eq!(name(2, 4), None); // Rock Toss is a level-5 skill
    assert_eq!(name(2, 5), Some(15916));
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
        let content = json!({"schema_version":1,"basics":[{"profile_id":43,"definition_id":1234,"name_id":2301,"description_id":2302,"icon_id":2303,"weapon_name_id":2401,"weapon_definition_id":2501}]});
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
// (row, level): rows 1-3 open at 1, 5 and 10; row 1 upgrades at 15.
fn job_fixture(profile: i32) -> (Fixture, Value) {
    let mut fixture = Fixture::new();
    fixture.content["basics"][0]["profile_id"] = json!(profile);
    fixture.write();
    let slots: Vec<_> = [(1, 1), (2, 5), (3, 10), (1, 15)]
        .into_iter()
        .enumerate()
        .map(|(index, (slot, level))| {
            json!({"slot": slot, "definition_id": 2000 + index,
                "name_id": 3000 + index, "icon_id": 4000 + index,
                "heal": 1, "cooldown_ms": 1000, "min_level": level})
        })
        .collect();
    let jobs = json!({"schema_version": 1,
        "jobs": [{"profile_id": profile, "slots": slots}]});
    (fixture, jobs)
}

// The other tests re-serialize the roster compactly; the gateway loads the shipped,
// pretty-printed file. W125's provenance pushed it past the old 64 KiB cap unnoticed.
#[test]
fn shipped_job_abilities_file_loads_through_the_real_loader() {
    let fixture = Fixture::new();
    let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/job_abilities.json");
    let path = fixture.path.join("job_abilities.json");
    fs::copy(&shipped, &path).unwrap();
    let mut abilities = fixture.load();
    abilities.load_job_abilities(&path).unwrap();
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
        // Basic, then rows 1-3; row 1 switches to its upgrade at 15 and rows
        // 4-7 never fill.
        let open = |minimum, id| if level >= minimum { id } else { 0 };
        let row_one = if level >= 15 { 2003 } else { open(1, 2000) };
        let expected = [1234, row_one, open(5, 2001), open(10, 2002), 0, 0, 0, 0];
        let packets = abilities.activation(profile, level).unwrap();
        let definitions: Vec<_> = packets[..packets.len() - 1]
            .iter()
            .map(|packet| i32::from_le_bytes(packet[4..8].try_into().unwrap()))
            .collect();
        let shown: Vec<_> = expected.into_iter().filter(|id| *id != 0).collect();
        assert_eq!(definitions, shown, "profile {profile}, level {level}");
        // Independently walk the native variable-length slot encoding, including
        // empty slots, so a gate on definitions alone cannot pass this test.
        let toolbar = packets.last().unwrap();
        let mut offset = 12;
        for (slot, id) in expected.into_iter().enumerate() {
            let kind = i32::from_le_bytes(toolbar[offset..offset + 4].try_into().unwrap());
            let what = format!("profile {profile}, level {level}, slot {slot}");
            assert_eq!(kind, if id == 0 { 0 } else { 3 }, "{what}");
            if kind == 3 {
                let at = offset + 4..offset + 8;
                assert_eq!(i32::from_le_bytes(toolbar[at].try_into().unwrap()), id);
            }
            offset += if kind == 3 { 41 } else { 4 };
            if slot > 0 {
                let cast = abilities.job_slot(profile, slot as i32, level);
                assert_eq!(cast.map_or(0, |a| a.definition_id), id, "{what}");
                assert!(
                    abilities
                        .job_slot(profile + 1000, slot as i32, 20)
                        .is_none()
                );
            }
        }
        assert_eq!(offset, toolbar.len());
    }
    for minimum in [1, 5, 10, 15] {
        assert!(abilities.unlocks_between(profile, minimum - 1, minimum));
    }
    assert!(!abilities.unlocks_between(profile, 1, 4));
    assert!(!abilities.unlocks_between(profile, 10, 14));
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

/// T421: a row past the client's fourth button, or a second skill for the same
/// (row, level), is refused, and the refused file leaves the loaded bar alone.
#[test]
fn job_rows_past_the_fourth_button_or_doubled_are_refused() {
    let (fixture, jobs) = job_fixture(11);
    let mut abilities = load_jobs(&fixture, &jobs).unwrap();
    let before = abilities.activation(11, 20).unwrap();
    let path = fixture.path.join("job_abilities.json");
    // Row 1's level-15 upgrade on a fifth button, on no button, and on the
    // level-1 unlock of its own row.
    for (field, value) in [("slot", 4), ("slot", 0), ("min_level", 1)] {
        let mut bad = jobs.clone();
        bad["jobs"][0]["slots"][3][field] = json!(value);
        fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(
            abilities.load_job_abilities(&path).is_err(),
            "{field} {value}"
        );
        assert_eq!(
            abilities.activation(11, 20).unwrap(),
            before,
            "{field} {value}"
        );
    }
}

/// An unavailable skill is never placed: the row keeps the skill below it, or
/// stays empty, and reaching its level unlocks nothing new.
#[test]
fn unavailable_skills_leave_their_row_to_the_skill_below() {
    let (fixture, mut jobs) = job_fixture(11);
    jobs["jobs"][0]["slots"][2]["unavailable"] = json!(true);
    jobs["jobs"][0]["slots"][3]["unavailable"] = json!(true);
    let abilities = load_jobs(&fixture, &jobs).unwrap();
    let bar = abilities
        .job_bar(11, 20)
        .map(|row| row.map(|a| a.definition_id));
    assert_eq!(bar, [Some(2000), Some(2001), None, None, None, None, None]);
    assert!(!abilities.unlocks_between(11, 9, 10));
    assert!(!abilities.unlocks_between(11, 14, 15));
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
    f.content["basics"][0]["weapon_definition_id"] = json!(-1);
    f.write();
    assert!(OpeningAbilities::load(&f.path).is_err());
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
fn frozen_basic_rows_inherit_the_canonical_starter_weapon() {
    let mut f = Fixture::new();
    f.content["basics"][0]
        .as_object_mut()
        .unwrap()
        .remove("weapon_definition_id");
    f.write();
    assert_eq!(f.load().starter_weapon(43), Some(75030));
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
