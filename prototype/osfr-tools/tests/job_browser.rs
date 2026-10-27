//! T497: the Jobs window's Become-a list (147/1 `ActivityProfileList`) and the 49/15
//! Start quest requests it leads to.
//!
//! The byte test pins the packet against our own 2014 capture (copied to
//! `tests/fixtures/job_browser/`, client ctor `0x00be5490`, reader `0x00be6f20`, dispatch
//! `0x00be7360` sub 1). The route tests drive the real `Gameplay` request path.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, breadcrumb_start, character_at_for, content_dir, pending_offer, saved, send,
};
use osfr_tools::{
    gameplay::Gameplay,
    job_browser::{self, ProfileRow},
    quest_engine::{QuestProgress, QuestStatus},
    quest_journal::{BREADCRUMB_REFUSED, read_crosswalk},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

const CAPTURE_SHA256: &str = "ab3ec4f7878d3eb0acf69eaf1b0d5f3c2e2c051eb577f6f2559156026c712d3e";
/// The captured packet is a 7-byte tunnel envelope, then the 427-byte application packet.
const ENVELOPE: usize = 7;
/// The first 12 captured rows are the Become-a rows; the last three are the captured
/// character's own in-progress quests, which are never shipped.
const BECOME_A_ROWS: usize = 12;
const FISHERMAN_ORIGINAL: i32 = 1968;
const FISHERMAN_REGIONAL: u32 = 3_070_001;
const ARCHER_ORIGINAL: i32 = 1952;
/// A quest that is active in the fixture's saved state, so the login sync sends a 49/3.
const ACTIVE_QUEST: u32 = 3_050_004;

fn capture() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/job_browser/op147-sub1-capture.bin");
    fs::read(path).expect("the capture fixture is checked in")
}

/// Rows of a 147/1: seven `i32` each (key, profile, quest, icon, badge, title, description).
fn rows(packet: &[u8]) -> Vec<[i32; 7]> {
    assert_eq!(packet[..3], [147, 0, 1]);
    let count = u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize;
    assert_eq!(packet.len(), 7 + count * 28, "no extra or missing bytes");
    packet[7..]
        .chunks_exact(28)
        .map(|row| {
            std::array::from_fn(|i| i32::from_le_bytes(row[i * 4..i * 4 + 4].try_into().unwrap()))
        })
        .collect()
}

fn profile_rows(rows: &[[i32; 7]]) -> Vec<ProfileRow> {
    rows.iter()
        .map(|row| ProfileRow {
            profile_id: row[1] as u32,
            original_quest_id: row[2] as u32,
            icon_id: row[3] as u32,
            badge_id: row[4] as u32,
            title_string_id: row[5] as u32,
            description_string_id: row[6] as u32,
        })
        .collect()
}

fn shipped() -> job_browser::JobBrowser {
    let (_, listed) = read_crosswalk(&content_dir()).unwrap();
    job_browser::load(&content_dir(), &listed).unwrap()
}

#[test]
fn packet_matches_the_capture() {
    let capture = capture();
    assert_eq!(capture.len(), 434);
    assert_eq!(format!("{:x}", Sha256::digest(&capture)), CAPTURE_SHA256);
    let captured = rows(&capture[ENVELOPE..]);
    assert_eq!(captured.len(), 15);
    assert!(
        captured.iter().all(|row| row[0] == row[1]),
        "the key is the ProfileId"
    );
    assert_eq!(
        job_browser::encode(&profile_rows(&captured)),
        capture[ENVELOPE..]
    );
}

#[test]
fn shipped_rows_resolve() {
    let browser = shipped();
    let sent = rows(&browser.packet);
    assert_eq!(sent.len(), BECOME_A_ROWS);
    // The shipped rows are the captured Become-a rows, in the captured order.
    let capture = capture();
    assert_eq!(sent, rows(&capture[ENVELOPE..])[..BECOME_A_ROWS]);

    let locale: Value = serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/reports/locale-id-to-text.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let locale: BTreeSet<i64> = locale
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_i64().unwrap())
        .collect();
    let jobs: Value =
        serde_json::from_slice(&fs::read(content_dir().join("jobs.json")).unwrap()).unwrap();
    let profiles: BTreeSet<i64> = jobs["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|job| job["profile_id"].as_i64())
        .collect();
    for row in &sent {
        assert!(
            profiles.contains(&(row[1] as i64)),
            "profile {} is in jobs.json",
            row[1]
        );
        for string in &row[5..] {
            assert!(
                locale.contains(&(*string as i64)),
                "string {string} resolves in the locale"
            );
        }
    }
    assert_eq!(
        browser.crosswalk,
        BTreeMap::from([
            (1968, 3_070_001),
            (1965, 3_140_001),
            (1979, 3_100_001),
            (1964, 3_130_001)
        ])
    );
}

fn character() -> osfr_tools::character_store::CharacterState {
    character_at_for(CHARACTER, 0, [0.0; 3])
}

/// The replies that start with `prefix`: 147/1 has a one-byte sub-opcode (`piece_020`), 49/3 a
/// two-byte one.
fn frames<'a>(replies: &'a [Vec<u8>], prefix: &[u8]) -> Vec<(usize, &'a Vec<u8>)> {
    replies
        .iter()
        .enumerate()
        .filter(|(_, packet)| packet.starts_with(prefix))
        .collect()
}

const LIST: [u8; 3] = [147, 0, 1];

#[test]
fn world_entry_sends_one_147_1() {
    let fixture = Fixture::new();
    let mut state = fixture.game(None).state();
    state.npc_quests.as_mut().unwrap().quests.insert(
        ACTIVE_QUEST,
        QuestProgress {
            status: QuestStatus::Active,
            counts: vec![1, 0],
        },
    );
    let mut game = fixture.game(Some(state));
    let entry = game.initialize_world(&character(), 0).unwrap();
    let lists = frames(&entry.replies, &LIST);
    assert_eq!(lists.len(), 1, "exactly one 147/1 at world entry");
    assert_eq!(lists[0].1, &shipped().packet);
    let quest_frames = frames(&entry.replies, &[49, 0, 3, 0]);
    assert!(
        !quest_frames.is_empty(),
        "the fixture's active quest is synced"
    );
    assert!(
        quest_frames.iter().all(|(index, _)| *index < lists[0].0),
        "the list follows the 49/3 sync"
    );

    // The same session asks again (zone ready): nothing more is sent.
    let again = game.initialize_world(&character(), 1).unwrap();
    assert!(frames(&again.replies, &LIST).is_empty());

    // A reconnect is a new session: one more.
    let mut reconnect = fixture.game(Some(game.state()));
    let entry = reconnect.initialize_world(&character(), 2).unwrap();
    assert_eq!(frames(&entry.replies, &LIST).len(), 1);
}

#[test]
fn two_players_each_get_their_own_147_1() {
    let (first, second) = (
        Fixture::for_character(CHARACTER),
        Fixture::for_character(CHARACTER + 1),
    );
    let (mut one, mut two) = (first.game(None), second.game_as(CHARACTER + 1, None));
    let a = one.initialize_world(&character(), 0).unwrap();
    let b = two
        .initialize_world(&character_at_for(CHARACTER + 1, 0, [0.0; 3]), 0)
        .unwrap();
    for entry in [&a, &b] {
        let lists = frames(&entry.replies, &LIST);
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].1, &shipped().packet);
    }
    // Sending to one session adds nothing to the other: nothing is a broadcast.
    assert!(
        frames(
            &one.initialize_world(&character(), 1).unwrap().replies,
            &LIST
        )
        .is_empty()
    );
}

#[test]
fn start_quest_with_a_disabled_or_unmapped_id() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let refusal = vec![osfr_tools::chat::system(BREADCRUMB_REFUSED)];
    // 1968 maps to 3070001, which is disabled today; 1952 (Archer) has no regional quest.
    for (step, id) in [FISHERMAN_ORIGINAL, ARCHER_ORIGINAL]
        .into_iter()
        .enumerate()
    {
        let before = saved(&game);
        let out = send(
            &mut game,
            &character(),
            &breadcrumb_start(id),
            100 + step as u64,
        );
        assert!(!out.changed, "id {id}");
        assert_eq!(out.replies, refusal, "id {id}");
        assert_eq!(saved(&game), before, "id {id} changed the save");
    }
}

#[test]
fn zero_or_negative_breadcrumb_is_refused_politely() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let refusal = vec![osfr_tools::chat::system(BREADCRUMB_REFUSED)];
    for (step, id) in [0, -1, i32::MIN].into_iter().enumerate() {
        let before = saved(&game);
        let out = game
            .handle_in_world(&breadcrumb_start(id), &character(), 200 + step as u64)
            .unwrap_or_else(|error| panic!("id {id} ended the session: {error}"));
        assert!(!out.changed, "id {id}");
        assert_eq!(out.replies, refusal, "id {id}");
        assert_eq!(saved(&game), before, "id {id} changed the save");
    }
    // The session goes on: an ordinary request still answers.
    assert_eq!(
        send(
            &mut game,
            &character(),
            &breadcrumb_start(ARCHER_ORIGINAL),
            300
        )
        .replies,
        refusal
    );
    // A request that is not even the right size is still an error, and 47/12 is untouched.
    assert!(
        game.handle_in_world(&[49, 0, 15, 0, 0, 0, 1, 0, 0], &character(), 301)
            .is_err()
    );
    assert!(
        game.handle_in_world(&[47, 0, 12, 0, 0, 0, 0], &character(), 302)
            .is_err()
    );
}

/// A scratch content folder: the bind's minimum (quests, npcs, jobs) plus whatever a test writes.
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static SERIAL: AtomicU32 = AtomicU32::new(0);
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t497-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::copy(content_dir().join("jobs.json"), dir.join("jobs.json")).unwrap();
        for folder in ["quests", "npcs"] {
            fs::create_dir_all(dir.join(folder)).unwrap();
            for entry in fs::read_dir(content_dir().join(folder)).unwrap() {
                let path = entry.unwrap().path();
                fs::copy(&path, dir.join(folder).join(path.file_name().unwrap())).unwrap();
            }
        }
        Self(dir)
    }
    fn write(&self, name: &str, document: &Value) {
        fs::write(self.0.join(name), serde_json::to_vec(document).unwrap()).unwrap();
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn shipped_file() -> Value {
    serde_json::from_slice(&fs::read(content_dir().join("job_browser.json")).unwrap()).unwrap()
}

#[test]
fn start_quest_with_a_mapped_id() {
    let scratch = Scratch::new();
    scratch.write("job_browser.json", &shipped_file());
    scratch.write(
        "breadcrumb_quests.json",
        &json!({"schema_version": 1, "records": []}),
    );
    // Enable the Fisherman's Become-a quest and give it one Talk goal.
    let path = scratch.0.join("quests/fisherman.json");
    let mut file: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let quest = file["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|quest| quest["id"] == FISHERMAN_REGIONAL)
        .unwrap();
    quest["enabled"] = json!(true);
    quest["objectives"] = json!([{
        "kind": "talk_to_npc", "id": 1, "npc_id": 9_000_000_217u64,
        "source": [{"kind": "reconstructed", "evidence": "(R) T497 test objective"}]
    }]);
    fs::write(&path, serde_json::to_vec(&file).unwrap()).unwrap();

    let fixture = Fixture::new();
    let mut game = Gameplay::load(fixture.path(), None).unwrap();
    game.bind_npc_quest_catalogue(&scratch.0, CHARACTER)
        .unwrap();
    let out = send(
        &mut game,
        &character(),
        &breadcrumb_start(FISHERMAN_ORIGINAL),
        400,
    );
    assert!(out.changed);
    assert!(
        out.replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
        "the 49/1 offer"
    );
    assert!(
        !out.replies
            .contains(&osfr_tools::chat::system(BREADCRUMB_REFUSED))
    );
    assert_eq!(pending_offer(&game), Some(FISHERMAN_REGIONAL));
}

#[test]
fn bad_job_browser_data_is_refused() {
    let scratch = Scratch::new();
    let good = shipped_file();
    let none = BTreeSet::new();
    let load = |document: &Value, listed: &BTreeSet<u32>| {
        scratch.write("job_browser.json", document);
        job_browser::load(&scratch.0, listed)
    };
    let with = |edit: &dyn Fn(&mut Vec<Value>)| {
        let mut document = good.clone();
        edit(document["records"].as_array_mut().unwrap());
        document
    };
    assert!(
        load(&good, &none).is_ok(),
        "the shipped file loads in the scratch folder"
    );

    let fails = |document: Value, listed: &BTreeSet<u32>, needle: &str| {
        let error = load(&document, listed)
            .err()
            .unwrap_or_else(|| panic!("{needle}: loaded"))
            .to_string();
        assert!(error.contains(needle), "{needle}: {error}");
    };
    fails(with(&|rows| rows.push(rows[0].clone())), &none, "duplicate");
    fails(
        with(&|rows| rows[0]["profile_id"] = json!(9_999)),
        &none,
        "unknown profile",
    );
    fails(
        with(&|rows| rows[0]["icon_id"] = json!(0)),
        &none,
        "invalid",
    );
    fails(
        with(&|rows| rows[0]["badge_id"] = json!(i32::MAX as u64 + 1)),
        &none,
        "invalid",
    );
    fails(
        with(&|rows| rows[0]["source"] = json!([])),
        &none,
        "invalid",
    );
    fails(
        with(&|rows| rows[0]["source"] = json!([{"kind": "reconstructed", "evidence": "a guess"}])),
        &none,
        "invalid",
    );
    fails(
        with(&|rows| {
            let first = rows[0].clone();
            rows.extend(vec![first; 60]);
        }),
        &none,
        "too many",
    );
    fails(json!({"schema_version": 2, "records": []}), &none, "schema");

    // An original id the breadcrumb crosswalk lists, mapped or not, is refused.
    let breadcrumb_record = |original: u32, regional: Value| {
        json!({"original_quest_id": original, "poi_ids": [7], "poi_name": "Synthetic", "regional_quest_id": regional,
            "source": [{"kind": "reconstructed", "evidence": "(R) synthetic test row"}]})
    };
    scratch.write(
        "breadcrumb_quests.json",
        &json!({"schema_version": 1, "records": [breadcrumb_record(1968, Value::Null)]}),
    );
    let (_, listed) = read_crosswalk(&scratch.0).unwrap();
    assert_eq!(listed, BTreeSet::from([1968]));
    fails(good.clone(), &listed, "breadcrumb crosswalk");
    // The shipped files do not overlap.
    shipped();

    // Over 64 KiB, even when it is only whitespace.
    scratch.write("job_browser.json", &good);
    let mut text = fs::read(scratch.0.join("job_browser.json")).unwrap();
    text.resize(70_000, b' ');
    fs::write(scratch.0.join("job_browser.json"), text).unwrap();
    assert!(
        job_browser::load(&scratch.0, &none)
            .unwrap_err()
            .to_string()
            .contains("64 KiB")
    );

    // The error reaches the bind, so a bad file never survives to login.
    let fixture = Fixture::new();
    let mut game = Gameplay::load(fixture.path(), None).unwrap();
    assert!(
        game.bind_npc_quest_catalogue(&scratch.0, CHARACTER)
            .is_err()
    );
}
