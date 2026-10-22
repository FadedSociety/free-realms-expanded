//! T437: a world reload replays the Ready startup stream (indices 8..46) without what only a login
//! should carry, through the real gateway with the opening content and a level-10 Brawler.
//!
//! * The welcome splash (opcode 93, index 41) goes out once per login: the client copies the
//!   packet's show flag (`00ad0790`, reader `009264b0`) and calls OnShowWelcomeScreen on every one
//!   (`00ad0000`), which runs WelcomeHandler:Show (ScriptsBase.lua 14615-14623, 14227-14278).
//! * 38/1 (index 33) carries the encounter's health after the first world init, not the captured
//!   400/400; a house load (`housing_loaded`) sends no later correction.
//! * The AnnouncementData row (opcode 193, reader `00a40aa0`/`00a3f350`) says 'closeWindow' (R),
//!   which WelcomeHandler:OnUserEvent handles (ScriptsBase.lua 14479); its captured 'Exit' is not.
//!
//! The private references under `lab/references` are the same ones `housing_session.rs` loads.
use osfr_tools::{gameplay::Gameplay, gateway::Gateway};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/gateway_packets.rs"]
#[allow(dead_code)]
mod gateway_packets;
use gateway_packets::login;
#[path = "../src/restoration/piece_096.rs"]
#[allow(dead_code)]
mod piece_096;

static NEXT: AtomicU64 = AtomicU64::new(0);
const WELCOME: u16 = 93;
const GRANT: [u8; 4] = [127, 0, 23, 0];
const LEAVE: [u8; 4] = [127, 0, 10, 0];
const BRAWLER: i32 = 43;
/// The announcement body (after opcode 193) the gateway replayed before T437, from the capture.
const CAPTURED_ANNOUNCEMENT: [u8; 62] = [
    0x02, 0x01, 0x00, 0x00, 0x00, 0x8b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x78, 0xba, 0x00,
    0x00, 0xa3, 0xde, 0x4d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x77, 0x07, 0x00, 0x00, 0x04, 0x00, 0x00,
    0x00, 0x45, 0x78, 0x69, 0x74, 0x05, 0x00, 0x00, 0x00, 0x50, 0x61, 0x72, 0x61, 0x6d, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The opening reference content plus the repository's houses.
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let source = std::env::var_os("OSFR_ACTIVITY_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                manifest().join("../../../lab/references/rust-opening-cooking-original-assets")
            });
        let dir = std::env::temp_dir().join(format!(
            "osfr-t437-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        content_copy::copy_tree(&source, &dir);
        fs::copy(
            manifest().join("data/content/houses.json"),
            dir.join("houses.json"),
        )
        .unwrap();
        Self(dir)
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

/// SOE application bundle (`00 19`, one-byte lengths: every child here is short).
fn bundle(children: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for child in children {
        bytes.push(u8::try_from(child.len()).unwrap());
        bytes.extend(child);
    }
    bytes
}

fn enter(house_guid: u64) -> Vec<u8> {
    let mut inner = vec![127, 0, 19, 0];
    inner.extend(house_guid.to_le_bytes());
    inner.extend(0i32.to_le_bytes());
    inner.push(0);
    inner
}

/// The opcode of every tunnel frame among the replies.
fn opcodes(replies: &[Vec<u8>]) -> Vec<u16> {
    replies
        .iter()
        .filter_map(|r| r.get(7..9))
        .map(|o| u16::from_le_bytes([o[0], o[1]]))
        .collect()
}

fn count(replies: &[Vec<u8>], opcode: u16) -> usize {
    opcodes(replies)
        .into_iter()
        .filter(|o| *o == opcode)
        .count()
}

/// Every 38/1 (current, max) among the replies.
fn health(replies: &[Vec<u8>]) -> Vec<(i32, i32)> {
    let word = |r: &[u8], at: usize| i32::from_le_bytes(r[at..at + 4].try_into().unwrap());
    replies
        .iter()
        .filter(|r| r.len() == 19 && r[7..11] == [38, 0, 1, 0])
        .map(|r| (word(r, 11), word(r, 15)))
        .collect()
}

struct Entry {
    login: Vec<Vec<u8>>,
    ready: Vec<Vec<u8>>,
    loaded: Vec<Vec<u8>>,
}

/// A fresh gateway for a level-10 Brawler (a combat job above the base level), logged in: the
/// first Ready and the first world init.
fn first_login(content: &Content) -> (Gateway, Entry) {
    let reference = manifest().join("../../../lab/references/rust-gateway-reference");
    let character = Gateway::reference_state(&reference).unwrap();
    let mut state = Gameplay::load(&content.0, None).unwrap().state();
    let profiles = state.profiles.as_mut().unwrap();
    assert!(profiles.unlocked.contains(&BRAWLER));
    profiles.active = BRAWLER;
    profiles.experience.insert(BRAWLER, 900);
    let mut gateway =
        Gateway::load_gameplay(&reference, character, &content.0, Some(state)).unwrap();
    let login = gateway.handle_at(&login(1), 10_000).unwrap().replies;
    let ready = gateway.handle_at(&wrap(&[13, 0]), 10_000).unwrap().replies;
    let loaded = gateway.handle_at(&wrap(&[10, 0]), 10_000).unwrap().replies;
    let entry = Entry {
        login,
        ready,
        loaded,
    };
    (gateway, entry)
}

/// 127/23 grants the starter lot; its guid from the 127/39 list.
fn grant(gateway: &mut Gateway) -> u64 {
    let listed = gateway.handle_at(&wrap(&GRANT), 11_000).unwrap();
    let list = listed
        .replies
        .iter()
        .filter_map(|r| r.get(7..))
        .find(|r| r.get(..4) == Some(&[127, 0, 39, 0][..]))
        .expect("127/39 lists the granted lot");
    osfr_tools::housing_native::InstanceList::decode(&list[2..])
        .unwrap()
        .records[0]
        .instance_guid
}

/// A scene change and the reload that follows it: the Ready replies, then the Loaded replies.
fn reload(gateway: &mut Gateway, request: &[u8], now_ms: u64) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let out = gateway.handle_at(&wrap(request), now_ms).unwrap();
    assert!(out.began_zoning, "{:?}", out.trace);
    let ready = gateway.handle_at(&wrap(&[13, 0]), now_ms).unwrap().replies;
    let loaded = gateway.handle_at(&wrap(&[10, 0]), now_ms).unwrap().replies;
    (ready, loaded)
}

#[test]
fn the_welcome_splash_goes_out_once_per_login() {
    let content = Content::new();
    let (mut gateway, first) = first_login(&content);
    assert_eq!(count(&first.ready, WELCOME), 1, "the first login shows it");
    let house = grant(&mut gateway);
    let (inside, _) = reload(&mut gateway, &enter(house), 12_000);
    assert_eq!(count(&inside, WELCOME), 0, "not on the house load");
    let (outside, _) = reload(&mut gateway, &LEAVE, 14_000);
    assert_eq!(count(&outside, WELCOME), 0, "not on the way back out");
    assert_eq!(
        opcodes(&first.ready).len(),
        opcodes(&outside).len() + 1,
        "only the splash is dropped from the reload stream"
    );
    gateway.reset_session();
    gateway.handle_at(&login(1), 16_000).unwrap();
    let relog = gateway.handle_at(&wrap(&[13, 0]), 16_000).unwrap();
    assert_eq!(
        count(&relog.replies, WELCOME),
        1,
        "a new login shows it again"
    );
}

#[test]
fn a_refused_bundle_does_not_spend_the_splash() {
    let content = Content::new();
    let reference = manifest().join("../../../lab/references/rust-gateway-reference");
    let character = Gateway::reference_state(&reference).unwrap();
    let state = Gameplay::load(&content.0, None).unwrap().state();
    let mut gateway =
        Gateway::load_gameplay(&reference, character, &content.0, Some(state)).unwrap();
    gateway.handle_at(&login(1), 10_000).unwrap();
    // Ready and Loaded, then a 90/1 warp request with no body: refused, so nothing commits.
    let bad = bundle(&[wrap(&[13, 0]), wrap(&[10, 0]), wrap(&[90, 0, 1, 0])]);
    assert!(gateway.handle_at(&bad, 10_000).is_err());
    let ready = gateway.handle_at(&wrap(&[13, 0]), 10_000).unwrap();
    assert_eq!(
        count(&ready.replies, WELCOME),
        1,
        "the refused Ready did not count"
    );
}

#[test]
fn a_reload_reports_the_job_health_not_the_captured_pair() {
    let content = Content::new();
    let (mut gateway, first) = first_login(&content);
    assert_eq!(
        health(&first.ready),
        [(400, 400)],
        "before any world init the captured pair stays"
    );
    let world = *health(&first.loaded)
        .last()
        .expect("the world init reports the job's health");
    assert!(
        world.1 > 1000,
        "a level-10 Brawler is above the base 1000: {world:?}"
    );
    let house = grant(&mut gateway);
    let (inside, inside_loaded) = reload(&mut gateway, &enter(house), 12_000);
    assert_eq!(
        health(&inside),
        [world],
        "the house load carries the live health"
    );
    assert!(
        health(&inside_loaded).is_empty(),
        "a house sends no later correction, so the Ready value is what the player sees"
    );
    let (outside, _) = reload(&mut gateway, &LEAVE, 14_000);
    assert_eq!(health(&outside), [world]);
}

#[test]
fn the_thanks_row_closes_the_splash_and_keeps_every_captured_field() {
    let content = Content::new();
    let (_, first) = first_login(&content);
    let frame = first
        .login
        .iter()
        .find(|r| r.get(7..9) == Some(&193u16.to_le_bytes()[..]))
        .expect("the login stream carries AnnouncementData");
    let piece_096::InitMessage::AnnouncementData(mut data) = piece_096::decode(6, frame).unwrap()
    else {
        panic!("slot 6 decodes as AnnouncementData");
    };
    assert_eq!(data.rows.len(), 1);
    // ScriptsBase.lua 14479: the OnUserEvent case that hides the splash.
    assert_eq!(data.rows[0].action, "closeWindow");
    data.rows[0].action = "Exit".into();
    let captured = [&193u16.to_le_bytes()[..], &CAPTURED_ANNOUNCEMENT].concat();
    assert_eq!(
        piece_096::InitMessage::AnnouncementData(data).encode_payload(),
        captured,
        "only the action differs from the capture"
    );
}
