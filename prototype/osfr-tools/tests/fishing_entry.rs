//! W160: fishing from an ordinary entry. The guide NPC menu admits the player (`enter`), `route`
//! plays the round, and the catch is paid once by the server. The service-level tests run
//! everywhere; `gateway_route` needs a playtest bundle (see its ignore reason).

use osfr_tools::{
    fishing::{Handle, Phase, ServerPacket, Sub},
    fishing_service::{
        Change, EntryDenied, FISHERMAN_PROFILE, FishingService, FishingSpots, MAX_SESSIONS,
    },
    inventory::Inventory,
    profiles::ProfileState,
    rewards::RewardState,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const SPOTS_JSON: &str = include_str!("../data/content/fishing_spots.json");
const PLAYER: u64 = 0x0000_00a1_b2c3_d4e5;
const OTHER_PLAYER: u64 = 0x0000_00a1_b2c3_d4e6;
const ACTOR: Handle = Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};
const REED: u64 = 9_000_000_181;
const SELVI: u64 = 9_000_000_194;
const SPOT: [f32; 3] = [100.0, 5.0, -40.0];
const T0: u64 = 1_000_000;
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[path = "support/fishing_packets.rs"]
mod fishing_packets;
#[path = "../src/bin/osfr_gateway/fishing_fixtures.rs"]
mod fixtures;

// ---- fixtures ---------------------------------------------------------------------------

fn spots() -> FishingSpots {
    FishingSpots::parse(SPOTS_JSON.as_bytes()).unwrap()
}

/// Every item id the spots file binds, so the fixture inventory knows them all.
fn bound_item_ids() -> BTreeSet<i32> {
    let json: Value = serde_json::from_str(SPOTS_JSON).unwrap();
    json["items"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|sizes| sizes.as_array().unwrap().iter())
        .map(|id| id.as_i64().unwrap() as i32)
        .filter(|id| *id > 0)
        .collect()
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-fishing-entry-{}-{}",
            std::process::id(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        let coins = 0u32.to_le_bytes();
        let empty = 0i32.to_le_bytes();
        let definitions: Vec<Value> = bound_item_ids()
            .into_iter()
            .map(|id| {
                let bytes = [id.to_le_bytes(), 8i32.to_le_bytes()].concat();
                json!({
                    "id": id,
                    "slot": 3,
                    "class_id": 1,
                    "default_tint": 5,
                    "max_stack": 10,
                    "model": "fixture.adr",
                    "texture": "fixture",
                    "tint_alias": "fixture",
                    "effect": 0,
                    "wield_type": 0,
                    "serialized_hex": hex(&bytes),
                    "sha256": digest(&bytes)
                })
            })
            .collect();
        let content = json!({
            "schema_version": 1,
            "player_guid": PLAYER,
            "default_state": {"coins": 0, "items": [], "equipment": [], "next_instance_id": null},
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [
                span("coins", 13, 0, &coins),
                span("items", 17, 0, &empty),
                span("equipment", 21, 1, &empty)
            ],
            "definitions": definitions
        });
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self(path)
    }

    fn inventory(&self) -> Inventory {
        Inventory::load(&self.0, None).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("inventory.json"));
        let _ = fs::remove_dir(&self.0);
    }
}

fn profiles(fisherman: bool) -> ProfileState {
    let mut unlocked = BTreeSet::from([1]);
    if fisherman {
        unlocked.insert(FISHERMAN_PROFILE);
    }
    ProfileState {
        version: 1,
        active: 1,
        unlocked,
        experience: BTreeMap::new(),
    }
}

fn span(kind: &str, offset: usize, profile_id: usize, bytes: &[u8]) -> Value {
    json!({"kind": kind, "offset": offset, "length": bytes.len(), "profile_id": profile_id, "sha256": digest(bytes)})
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ---- packets ----------------------------------------------------------------------------

use fishing_packets::{encounter_close, handle_bytes, lifecycle, request};

fn register() -> Vec<u8> {
    fishing_packets::register(ACTOR)
}

fn cast_at(distance: f32) -> Vec<u8> {
    // The service measures from the spot, so aim relative to it.
    fishing_packets::cast_at(ACTOR, SPOT, distance)
}

fn reel() -> Vec<u8> {
    fishing_packets::reel(ACTOR)
}

fn ids(replies: &[Vec<u8>]) -> Vec<(u16, u16)> {
    replies
        .iter()
        .map(|packet| {
            let family = u16::from_le_bytes([packet[0], packet[1]]);
            let sub = if family == 39 || family == 50 {
                u16::from(packet[2])
            } else {
                u16::from_le_bytes([packet[2], packet[3]])
            };
            (family, sub)
        })
        .collect()
}

fn status_flags(replies: &[Vec<u8>]) -> Vec<u32> {
    replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 20, 0]))
        .map(|p| u32::from_le_bytes(p[12..16].try_into().unwrap()))
        .collect()
}

// ---- a player's round, driven the way the gameplay glue drives it ------------------------

struct Round {
    service: FishingService,
    spots: FishingSpots,
    inventory: Inventory,
    profiles: ProfileState,
    _fixture: Fixture,
}

impl Round {
    fn new(fisherman: bool) -> Self {
        let fixture = Fixture::new();
        Self {
            service: FishingService::new(),
            spots: spots(),
            inventory: fixture.inventory(),
            profiles: profiles(fisherman),
            _fixture: fixture,
        }
    }

    fn rewards(&self, bag: usize) -> RewardState {
        RewardState::new(self.inventory.clone(), self.profiles.clone(), bag)
    }

    fn enter(&mut self, npc: u64, busy: bool) -> Result<Change, EntryDenied> {
        let rewards = self.rewards(100);
        self.service
            .enter(&self.spots, npc, PLAYER, 0, SPOT, busy, 1, rewards)
    }

    /// `route`, then adopt a paid catch exactly as `Gameplay::handle_fishing` does.
    fn send(
        &mut self,
        packet: &[u8],
        now: u64,
    ) -> Result<Option<Change>, osfr_tools::fishing::Error> {
        let routed = self
            .service
            .route(PLAYER, packet, now, &self.inventory, &self.profiles)?;
        if routed.as_ref().is_some_and(|change| change.paid) {
            let next = self.service.reward_state(PLAYER).unwrap().clone();
            self.inventory = next.inventory;
            self.profiles = next.profiles;
        }
        Ok(routed)
    }

    /// Entry, the client's Go (39/5) and the registration: the round is ready to cast.
    fn started(fisherman: bool) -> Self {
        Self::started_with(spots(), fisherman)
    }

    fn started_with(spots: FishingSpots, fisherman: bool) -> Self {
        let mut round = Self::new(fisherman);
        round.spots = spots;
        round.start();
        round
    }

    /// Offer and start a round with the current `spots`. Also reuses the same service, so a
    /// session-scoped cap can be observed across trips.
    fn start(&mut self) {
        let offer = self.enter(REED, false).unwrap();
        assert_eq!(ids(&offer.replies), vec![(39, 16)]);
        let instance = self.service.instance_id(PLAYER).unwrap();
        let go = self.send(&lifecycle(5, instance), T0).unwrap().unwrap();
        assert_eq!(ids(&go.replies), vec![(41, 2), (35, 20)]);
        assert_eq!(
            status_flags(&go.replies),
            vec![0x40_0001],
            "fishing flag on"
        );
        let registered = self.send(&register(), T0).unwrap().unwrap();
        assert_eq!(ids(&registered.replies)[0], (138, 3));
    }

    /// Cast, wait out the bite and hook, reel. Returns the reel's change.
    fn catch(&mut self, at: u64) -> Change {
        self.send(&cast_at(10.0), at + 10).unwrap().unwrap();
        self.service.tick(PLAYER, at + 1_510).unwrap();
        self.service.tick(PLAYER, at + 2_410).unwrap();
        assert_eq!(self.service.phase(PLAYER), Some(Phase::Reeling));
        self.send(&reel(), at + 2_420).unwrap().unwrap()
    }

    fn fish_held(&self) -> u32 {
        let ids = bound_item_ids();
        self.inventory
            .state()
            .items
            .iter()
            .filter(|item| ids.contains(&item.definition_id))
            .map(|item| item.count)
            .sum()
    }

    fn fisherman_xp(&self) -> u32 {
        self.profiles
            .experience
            .get(&FISHERMAN_PROFILE)
            .copied()
            .unwrap_or(0)
    }

    fn coins(&self) -> u32 {
        self.inventory.state().coins
    }
}

// ---- the spots file -----------------------------------------------------------------------

#[test]
fn shipped_spots_file_is_valid_and_names_real_npcs() {
    let spots = spots();
    assert_eq!(spots.gate_npcs().len(), 6, "one guide per fishing area");
    let json: Value = serde_json::from_str(SPOTS_JSON).unwrap();
    assert!(
        !json["catch_cap"]["enabled"].as_bool().unwrap(),
        "the shipped cap is off until the operator picks a value"
    );
    assert!(
        json["catch_cap"]["max_paid_catches"].is_null(),
        "the shipped file deliberately has no cap value"
    );
    assert_eq!(json["payout"]["by_grade"].as_array().unwrap().len(), 4);
    for (index, row) in json["payout"]["by_grade"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        assert_eq!(row["grade"], json!(index + 1));
        assert_eq!(row["coins"], json!(0), "no coin payout is evidenced");
        assert_eq!(row["fisherman_xp"], json!(10), "the previous (R) policy");
    }
    assert_eq!(spots.menu_string_id(), 73_552, "locale 'Go fish!'");
    // Every guide must be NPC content, or the menu option could never appear.
    let mut known = BTreeSet::new();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/npcs");
    for entry in fs::read_dir(dir).unwrap() {
        let json: Value =
            serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap();
        collect_npc_ids(&json, &mut known);
    }
    for npc in spots.gate_npcs() {
        assert!(known.contains(&npc), "gate {npc} is not NPC content");
    }
}

fn collect_npc_ids(value: &Value, out: &mut BTreeSet<u64>) {
    match value {
        Value::Object(map) => {
            if let Some(id) = map.get("npc_id").and_then(Value::as_u64) {
                out.insert(id);
            }
            map.values().for_each(|v| collect_npc_ids(v, out));
        }
        Value::Array(items) => items.iter().for_each(|v| collect_npc_ids(v, out)),
        _ => {}
    }
}

fn edited(mutate: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut json: Value = serde_json::from_str(SPOTS_JSON).unwrap();
    mutate(&mut json);
    serde_json::to_vec(&json).unwrap()
}

fn spots_with(mutate: impl FnOnce(&mut Value)) -> FishingSpots {
    FishingSpots::parse(&edited(mutate)).unwrap()
}

#[test]
fn malformed_spots_files_are_refused() {
    let bad: Vec<(&str, Vec<u8>)> = vec![
        ("schema 2", edited(|j| j["schema_version"] = json!(2))),
        ("label 0", edited(|j| j["menu_string_id"] = json!(0))),
        (
            "payout source without provenance",
            edited(|j| j["payout"]["source"] = json!("values")),
        ),
        (
            "payout xp too high",
            edited(|j| j["payout"]["by_grade"][0]["fisherman_xp"] = json!(1_001)),
        ),
        (
            "payout coins too high",
            edited(|j| j["payout"]["by_grade"][0]["coins"] = json!(1_000_001)),
        ),
        (
            "payout missing grade",
            edited(|j| {
                j["payout"]["by_grade"].as_array_mut().unwrap().pop();
            }),
        ),
        (
            "payout wrong grade order",
            edited(|j| j["payout"]["by_grade"][1]["grade"] = json!(1)),
        ),
        (
            "cap enabled without a limit",
            edited(|j| j["catch_cap"]["enabled"] = json!(true)),
        ),
        (
            "cap limit zero",
            edited(|j| {
                j["catch_cap"]["enabled"] = json!(true);
                j["catch_cap"]["max_paid_catches"] = json!(0);
            }),
        ),
        (
            "session cap over sanity bound",
            edited(|j| {
                j["catch_cap"]["enabled"] = json!(true);
                j["catch_cap"]["max_paid_catches"] = json!(10_001);
            }),
        ),
        (
            "hour cap over sanity bound",
            edited(|j| {
                j["catch_cap"]["enabled"] = json!(true);
                j["catch_cap"]["window"] = json!("hour");
                j["catch_cap"]["max_paid_catches"] = json!(3_601);
            }),
        ),
        (
            "unknown cap window",
            edited(|j| j["catch_cap"]["window"] = json!("day")),
        ),
        (
            "cap source without provenance",
            edited(|j| j["catch_cap"]["source"] = json!("off")),
        ),
        ("zero npc", edited(|j| j["spots"][0]["npc"] = json!(0))),
        (
            "duplicate npc",
            edited(|j| j["spots"][1]["npc"] = j["spots"][0]["npc"].clone()),
        ),
        (
            "unknown area",
            edited(|j| j["spots"][0]["area"] = json!("nowhere")),
        ),
        ("game id 0", edited(|j| j["spots"][0]["game_id"] = json!(0))),
        (
            "negative name",
            edited(|j| j["spots"][0]["name_id"] = json!(-1)),
        ),
        (
            "unknown species",
            edited(|j| j["items"]["unicorn"] = json!([1, 1, 1, 1])),
        ),
        (
            "species without any item",
            edited(|j| j["items"]["flutterfish"] = json!([0, 0, 0, 0])),
        ),
        (
            "negative item",
            edited(|j| j["items"]["flutterfish"] = json!([1, -1, 1, 1])),
        ),
        ("unknown field", edited(|j| j["extra"] = json!(1))),
        (
            "17 spots",
            edited(|j| {
                let first = j["spots"][0].clone();
                j["spots"] = json!(vec![first; 17]);
            }),
        ),
        ("not json", b"{".to_vec()),
        ("over 64 KiB", vec![b' '; 64 * 1024 + 1]),
    ];
    for (name, bytes) in bad {
        assert!(
            FishingSpots::parse(&bytes).is_err(),
            "{name} must be refused"
        );
    }
}

#[test]
fn spots_load_needs_a_regular_file_and_absent_means_nobody_offers_fishing() {
    let dir = std::env::temp_dir().join(format!("osfr-spots-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    assert!(FishingSpots::load(&dir).unwrap().is_none(), "absent file");
    fs::create_dir(dir.join("fishing_spots.json")).unwrap();
    assert!(
        FishingSpots::load(&dir).is_err(),
        "a directory in its place"
    );
    fs::remove_dir(dir.join("fishing_spots.json")).unwrap();
    fs::write(dir.join("fishing_spots.json"), SPOTS_JSON).unwrap();
    assert_eq!(
        FishingSpots::load(&dir).unwrap().unwrap().gate_npcs().len(),
        6
    );
    fs::write(dir.join("fishing_spots.json"), vec![b' '; 64 * 1024 + 1]).unwrap();
    assert!(FishingSpots::load(&dir).is_err(), "an oversize file");
    fs::remove_dir_all(&dir).unwrap();
}

// ---- admission ----------------------------------------------------------------------------

#[test]
fn only_listed_guides_admit_and_a_refusal_changes_nothing() {
    let mut round = Round::new(true);
    for npc in [0, REED + 1, REED - 1, u64::MAX, 9_000_000_048] {
        assert_eq!(round.enter(npc, false).unwrap_err(), EntryDenied::NotAGate);
    }
    assert_eq!(round.enter(REED, true).unwrap_err(), EntryDenied::Busy);
    assert!(round.service.is_empty(), "no refusal left a round behind");
    assert_eq!(round.fish_held(), 0);

    let offer = round.enter(REED, false).unwrap();
    assert_eq!(
        ids(&offer.replies),
        vec![(39, 16)],
        "the offer is the only effect"
    );
    assert!(offer.changed && !offer.paid);
    assert_eq!(round.service.len(), 1);
    assert_eq!(
        round.enter(SELVI, false).unwrap_err(),
        EntryDenied::AlreadyFishing,
        "a second trip while one is open"
    );
    assert_eq!(round.service.len(), 1);
    // Every refusal speaks.
    for denied in [
        EntryDenied::NotAGate,
        EntryDenied::AlreadyFishing,
        EntryDenied::Busy,
        EntryDenied::Full,
        EntryDenied::Unavailable,
    ] {
        assert!(!denied.message().is_empty());
    }
}

#[test]
fn every_guide_opens_its_own_area() {
    // The six guides map to six different MiniGameData rows (596..=601), read from the 39/16.
    let spots = spots();
    let mut games = BTreeSet::new();
    for npc in spots.gate_npcs() {
        let mut round = Round::new(false);
        let offer = round
            .service
            .enter(&spots, npc, PLAYER, 0, SPOT, false, 1, round.rewards(100))
            .unwrap();
        let game = i32::from_le_bytes(offer.replies[0][11..15].try_into().unwrap());
        assert!((596..=601).contains(&game), "game id {game}");
        games.insert(game);
    }
    assert_eq!(games.len(), 6);
}

#[test]
fn the_service_holds_at_most_max_sessions_players() {
    let spots = spots();
    let fixture = Fixture::new();
    let mut service = FishingService::new();
    for player in 1..=MAX_SESSIONS as u64 {
        let rewards = RewardState::new(fixture.inventory(), profiles(false), 100);
        service
            .enter(&spots, REED, player, 0, SPOT, false, 1, rewards)
            .unwrap();
    }
    let rewards = RewardState::new(fixture.inventory(), profiles(false), 100);
    assert_eq!(
        service
            .enter(&spots, REED, 99, 0, SPOT, false, 1, rewards)
            .unwrap_err(),
        EntryDenied::Full
    );
    assert_eq!(service.len(), MAX_SESSIONS);
}

// ---- the round ----------------------------------------------------------------------------

#[test]
fn one_catch_is_paid_once_with_the_item_and_fisherman_xp() {
    let mut round = Round::started(true);
    let caught = round.catch(T0 + 100);
    assert!(caught.paid && caught.changed);
    assert_eq!(ids(&caught.replies)[0], (138, 14), "the catch card first");
    assert!(
        caught.notices.is_empty(),
        "a normal catch needs no notice: {:?}",
        caught.notices
    );
    assert_eq!(round.fish_held(), 1, "exactly one fish");
    assert_eq!(
        round.fisherman_xp(),
        10,
        "(R) one catch pays 10 Fisherman XP"
    );

    // The same reel bytes again, in the same window or later: nothing more is paid.
    for at in [T0 + 2_530, T0 + 2_600, T0 + 60_000] {
        let replay = round.send(&reel(), at);
        assert!(
            replay.is_err() || replay.unwrap().is_none_or(|c| !c.paid),
            "a replayed reel must pay nothing"
        );
    }
    assert_eq!(round.fish_held(), 1);
    assert_eq!(round.fisherman_xp(), 10);
}

#[test]
fn a_locked_fisherman_job_still_catches_but_gets_no_xp() {
    let mut round = Round::started(false);
    let caught = round.catch(T0 + 100);
    assert!(caught.paid);
    assert_eq!(round.fish_held(), 1);
    assert_eq!(round.fisherman_xp(), 0);
}

#[test]
fn casting_again_after_a_catch_pays_the_next_catch_once_more() {
    let mut round = Round::started(true);
    let mut at = T0 + 100;
    for caught in 1..=3u32 {
        assert!(round.catch(at).paid);
        assert_eq!(round.fish_held(), caught, "one fish per catch");
        assert_eq!(round.fisherman_xp(), 10 * caught);
        at += 10_000;
    }
}

#[test]
fn the_cap_off_path_pays_every_catch_as_before() {
    let mut round = Round::started(true);
    let mut at = T0 + 100;
    for caught in 1..=6u32 {
        assert!(round.catch(at).paid);
        assert_eq!(round.fish_held(), caught, "one fish per catch");
        assert_eq!(round.fisherman_xp(), 10 * caught);
        assert_eq!(round.coins(), 0, "the shipped table pays no coins");
        at += 10_000;
    }
}

#[test]
fn the_payout_row_for_the_caught_grade_drives_coins_and_xp() {
    let spots = spots_with(|j| {
        for (index, row) in j["payout"]["by_grade"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
        {
            let grade = index as u64 + 1;
            row["coins"] = json!(grade * 10);
            row["fisherman_xp"] = json!(grade);
        }
    });
    let mut round = Round::started_with(spots, true);
    let caught = round.catch(T0 + 100);
    assert!(caught.paid);
    let grade = match ServerPacket::decode_known(Sub::Result, &caught.replies[0][4..]).unwrap() {
        ServerPacket::Result { body } => body.at_3c,
        packet => panic!("expected a catch result, got {packet:?}"),
    };
    assert!((1..=4).contains(&grade));
    assert_eq!(round.fish_held(), 1);
    assert_eq!(
        round.fisherman_xp(),
        grade,
        "the caught grade's XP row, not a fixed value"
    );
    assert_eq!(round.coins(), grade * 10, "the caught grade's coin row");
}

#[test]
fn a_session_cap_stops_payouts_and_survives_a_new_trip() {
    let mut round = Round::started_with(
        spots_with(|j| {
            j["catch_cap"]["enabled"] = json!(true);
            j["catch_cap"]["window"] = json!("session");
            j["catch_cap"]["max_paid_catches"] = json!(2);
        }),
        true,
    );
    let mut at = T0 + 100;
    for _ in 0..2 {
        assert!(round.catch(at).paid);
        at += 10_000;
    }
    let before_inventory = round.inventory.state().clone();
    let before_profiles = round.profiles.clone();
    let before_service = round.service.reward_state(PLAYER).unwrap().clone();

    let blocked = round.catch(at);
    assert!(!blocked.paid);
    assert!(blocked.changed, "the attempt still ends");
    assert_eq!(
        ids(&blocked.replies),
        vec![(138, 10), (138, 14)],
        "the fish escapes instead of paying"
    );
    match ServerPacket::decode_known(Sub::Result, &blocked.replies[1][4..]).unwrap() {
        ServerPacket::Result { body } => assert_eq!(body.at_18, 0),
        packet => panic!("expected an escape result, got {packet:?}"),
    }
    assert_eq!(blocked.notices.len(), 1);
    assert!(blocked.notices[0].contains("reward limit"));
    assert!(blocked.quest_events.is_empty());
    assert_eq!(round.fish_held(), 2);
    assert_eq!(round.fisherman_xp(), 20);
    assert_eq!(round.coins(), 0);
    assert_eq!(round.inventory.state(), &before_inventory);
    assert_eq!(round.profiles, before_profiles);
    let after_service = round.service.reward_state(PLAYER).unwrap().clone();
    assert_eq!(
        after_service.inventory.state(),
        before_service.inventory.state()
    );
    assert_eq!(after_service.profiles, before_service.profiles);
    assert_eq!(
        after_service.applied_grant_keys, before_service.applied_grant_keys,
        "a refused catch adds no grant key"
    );

    // Re-entering within the same login session does not refresh the cap.
    round.service.leave(PLAYER).unwrap();
    round.start();
    let blocked_again = round.catch(T0 + 100_000);
    assert!(!blocked_again.paid);
    assert_eq!(round.fish_held(), 2);

    // Dropping the login session clears the ledger.
    assert_eq!(round.service.reset_all(), 1);
    round.start();
    assert!(
        round.catch(T0 + 200_000).paid,
        "a fresh session starts the cap again"
    );
}

#[test]
fn an_hour_cap_uses_a_rolling_window() {
    let mut round = Round::started_with(
        spots_with(|j| {
            j["catch_cap"]["enabled"] = json!(true);
            j["catch_cap"]["window"] = json!("hour");
            j["catch_cap"]["max_paid_catches"] = json!(1);
        }),
        true,
    );
    assert!(round.catch(T0 + 100).paid);
    let blocked = round.catch(T0 + 10_000);
    assert!(!blocked.paid);
    assert_eq!(round.fish_held(), 1);

    // The paid catch was at T0+2_520; one hour later its slot is free.
    assert!(round.catch(T0 + 3_610_000).paid);
    // The new catch is still inside the hour, so the next one is refused again.
    let blocked_again = round.catch(T0 + 3_620_000);
    assert!(!blocked_again.paid);
    assert_eq!(round.fish_held(), 2);
}

#[test]
fn reconnect_clears_only_the_session_count_not_the_hour_history() {
    let mut round = Round::started_with(
        spots_with(|j| {
            j["catch_cap"]["enabled"] = json!(true);
            j["catch_cap"]["window"] = json!("hour");
            j["catch_cap"]["max_paid_catches"] = json!(1);
        }),
        true,
    );
    assert!(round.catch(T0 + 100).paid);
    let history = round.service.hour_history(PLAYER);
    assert_eq!(history, vec![T0 + 2_520]);
    assert_eq!(round.service.reset_all(), 1);
    assert_eq!(round.service.hour_history(PLAYER), history);

    let before_bad_restore = round.service.hour_history(PLAYER);
    assert!(
        round
            .service
            .restore_hour_history(PLAYER, vec![T0 + 3, T0 + 1])
            .is_err()
    );
    assert_eq!(
        round.service.hour_history(PLAYER),
        before_bad_restore,
        "a refused ledger restore changes nothing"
    );

    round.start();
    let blocked = round.catch(T0 + 20_000);
    assert!(!blocked.paid, "the reconnect did not refresh the hour cap");
    assert_eq!(round.fish_held(), 1);
    assert!(round.catch(T0 + 3_700_000).paid);
}

#[test]
fn reeling_before_the_hook_pays_nothing() {
    let mut round = Round::started(true);
    round.send(&cast_at(10.0), T0 + 10).unwrap();
    let early = round.send(&reel(), T0 + 20).unwrap().unwrap();
    assert!(!early.paid);
    assert_eq!(
        ids(&early.replies),
        vec![(138, 10), (138, 14)],
        "the fish escapes"
    );
    assert_eq!(round.fish_held(), 0);
    assert_eq!(round.fisherman_xp(), 0);
}

#[test]
fn letting_the_hooked_fish_wait_too_long_pays_nothing() {
    let mut round = Round::started(true);
    round.send(&cast_at(10.0), T0 + 10).unwrap();
    round.service.tick(PLAYER, T0 + 1_510).unwrap();
    round.service.tick(PLAYER, T0 + 2_410).unwrap();
    // The 7 s reel window has passed: the late reel is refused or finds nothing.
    let late = round.send(&reel(), T0 + 2_410 + 7_001);
    assert!(late.is_err() || late.unwrap().is_none_or(|c| !c.paid));
    assert_eq!(round.fish_held(), 0);
}

#[test]
fn forged_and_hostile_frames_pay_nothing_and_never_panic() {
    let mut round = Round::started(true);
    let mut other_actor = request(7, &handle_bytes(Handle { id: 9, seq: 9 }));
    other_actor.push(1);
    let mut forged_result = request(14, &handle_bytes(ACTOR));
    forged_result.extend(5u32.to_le_bytes());
    let mut forged_register = request(2, &handle_bytes(Handle { id: 7, seq: 7 }));
    forged_register.push(0);
    let frames: Vec<Vec<u8>> = vec![
        forged_result,          // sub 14 is server-to-client only
        other_actor,            // a different actor handle
        forged_register,        // a second actor binding to the same round
        request(17, &[0; 8]),   // unassigned sub
        request(6, &[1, 2, 3]), // truncated cast
        request(7, &[]),        // empty reel
        vec![138],              // a lone opcode byte
        vec![138, 0],           // header cut short
        vec![0xff; 600],        // oversize garbage
        cast_at(500.0),         // far outside the cast range
        cast_at(f32::NAN),      // non-finite aim
        reel(),                 // no cast yet
    ];
    for frame in frames {
        let _ = round.send(&frame, T0 + 50);
        assert_eq!(
            round.fish_held(),
            0,
            "frame {:02x?} paid",
            &frame[..frame.len().min(6)]
        );
        assert_eq!(round.fisherman_xp(), 0);
    }
    // The round still works afterwards.
    assert!(round.catch(T0 + 1_000).paid);
    assert_eq!(round.fish_held(), 1);
}

#[test]
fn unrelated_packets_and_other_players_are_not_routed() {
    let mut round = Round::started(true);
    assert!(
        round.send(&[15, 0, 1, 0], T0).unwrap().is_none(),
        "chat is not fishing"
    );
    assert!(round.send(&[], T0).unwrap().is_none());
    let other = round
        .service
        .route(
            OTHER_PLAYER,
            &register(),
            T0,
            &round.inventory,
            &round.profiles,
        )
        .unwrap();
    assert!(
        other.is_none(),
        "a player with no round gets no fishing handling"
    );
    assert_eq!(round.service.len(), 1);
}

#[test]
fn only_sizes_with_an_item_are_caught() {
    // Bind only the medium size: every catch must be a medium (grade 2), never a missing size.
    let fixture = Fixture::new();
    let mut service = FishingService::new();
    let json: Value = serde_json::from_str(SPOTS_JSON).unwrap();
    let mut items = serde_json::Map::new();
    for key in json["items"].as_object().unwrap().keys() {
        let id = json["items"][key]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .find(|id| *id > 0)
            .unwrap();
        items.insert(key.clone(), json!([0, id, 0, 0]));
    }
    let mut edited_json = json.clone();
    edited_json["items"] = Value::Object(items);
    let spots = FishingSpots::parse(&serde_json::to_vec(&edited_json).unwrap()).unwrap();
    let rewards = RewardState::new(fixture.inventory(), profiles(true), 1_000);
    service
        .enter(&spots, REED, PLAYER, 0, SPOT, false, 20, rewards)
        .unwrap();
    let instance = service.instance_id(PLAYER).unwrap();
    let mut inventory = fixture.inventory();
    let mut state = profiles(true);
    let mut route = |service: &mut FishingService, packet: &[u8], now: u64| {
        let change = service
            .route(PLAYER, packet, now, &inventory, &state)
            .unwrap()
            .unwrap();
        if change.paid {
            let next = service.reward_state(PLAYER).unwrap().clone();
            inventory = next.inventory;
            state = next.profiles;
        }
        change
    };
    route(&mut service, &lifecycle(5, instance), T0);
    route(&mut service, &register(), T0);
    let mut grades = BTreeSet::new();
    for n in 0..40u64 {
        let at = T0 + 100 + n * 10_000;
        route(&mut service, &cast_at(10.0), at + 10);
        service.tick(PLAYER, at + 1_510).unwrap();
        service.tick(PLAYER, at + 2_410).unwrap();
        let caught = route(&mut service, &reel(), at + 2_420);
        assert!(caught.paid, "cast {n} paid");
        match ServerPacket::decode_known(Sub::Result, &caught.replies[0][4..]).unwrap() {
            ServerPacket::Result { body } => grades.insert(body.at_3c),
            packet => panic!("expected a catch result, got {packet:?}"),
        };
    }
    assert_eq!(
        grades,
        BTreeSet::from([2]),
        "only the bound size is ever caught"
    );
}

#[test]
fn a_catch_that_cannot_be_stored_is_released_and_says_so() {
    // `handle` keeps the bag floor given at entry (route raises it), so 0 forces the failure.
    let mut round = Round::new(true);
    let full = round.rewards(0);
    round
        .service
        .enter(&round.spots, REED, PLAYER, 0, SPOT, false, 1, full)
        .unwrap();
    let instance = round.service.instance_id(PLAYER).unwrap();
    round.send(&lifecycle(5, instance), T0).unwrap();
    let at = T0 + 100;
    round.service.handle(&register(), PLAYER, at).unwrap();
    round
        .service
        .handle(&cast_at(10.0), PLAYER, at + 10)
        .unwrap();
    round.service.tick(PLAYER, at + 1_510).unwrap();
    round.service.tick(PLAYER, at + 2_410).unwrap();
    let caught = round.service.handle(&reel(), PLAYER, at + 2_420).unwrap();
    assert!(!caught.paid);
    assert_eq!(caught.notices.len(), 1);
    assert_eq!(round.fish_held(), 0);
    assert_eq!(round.fisherman_xp(), 0);
    assert_eq!(
        round.service.phase(PLAYER),
        Some(Phase::Idle),
        "ready to cast again"
    );
}

#[test]
fn a_large_existing_inventory_does_not_block_catches() {
    // Fish stack without limit, so the bag floor is current stacks + every fish definition.
    let mut round = Round::started(true);
    let some_id = *bound_item_ids().iter().next().unwrap();
    for _ in 0..300 {
        // 300 other stacks: the entry floor (100) is already exceeded.
        round.inventory.grant(0, some_id, 10).unwrap();
    }
    assert!(round.inventory.state().items.len() >= 100);
    assert!(round.catch(T0 + 100).paid);
}

#[test]
fn the_payout_lands_on_the_live_inventory_not_the_entry_snapshot() {
    // Another system grants an item mid-round (here: a second fixture inventory with one more
    // stack). Whatever the caller passes to `route` is what the catch is added to.
    let mut round = Round::started(true);
    let live = {
        let fixture = Fixture::new();
        let mut inventory = fixture.inventory();
        let some_id = *bound_item_ids().iter().next().unwrap();
        inventory.grant(0, some_id, 3).unwrap();
        inventory
    };
    round.inventory = live;
    assert_eq!(round.fish_held(), 3);
    assert!(round.catch(T0 + 100).paid);
    assert_eq!(round.fish_held(), 4, "the earlier 3 are kept");
}

// ---- quit, leave, disconnect, reconnect ---------------------------------------------------

#[test]
fn quitting_mid_attempt_pays_nothing_and_turns_the_flag_off() {
    for quit in ["41/109", "39/6"] {
        let mut round = Round::started(true);
        round.send(&cast_at(10.0), T0 + 10).unwrap();
        round.service.tick(PLAYER, T0 + 1_510).unwrap();
        let instance = round.service.instance_id(PLAYER).unwrap();
        let packet = if quit == "41/109" {
            encounter_close(instance)
        } else {
            lifecycle(6, instance)
        };
        let left = round.send(&packet, T0 + 2_000).unwrap().unwrap();
        assert!(!left.paid, "{quit}");
        assert_eq!(status_flags(&left.replies), vec![1], "{quit}: flag off");
        assert!(round.service.is_empty(), "{quit}: the round is gone");
        assert_eq!(round.fish_held(), 0, "{quit}");
        assert_eq!(round.fisherman_xp(), 0, "{quit}");
        // The reel that would have caught it now finds no round.
        assert!(round.send(&reel(), T0 + 2_420).unwrap().is_none());
        // And the player can go again.
        assert!(round.enter(REED, false).is_ok(), "{quit}: a new trip");
    }
}

#[test]
fn cancelling_the_offer_with_26_42_frees_the_player_without_a_flag() {
    let mut round = Round::new(true);
    round.enter(REED, false).unwrap();
    let cancelled = round.send(&[26, 0, 42, 0], T0).unwrap().unwrap();
    assert!(round.service.is_empty());
    // The flag was never on, but switching it off is idempotent and keeps the client sane.
    assert_eq!(status_flags(&cancelled.replies), vec![1]);
    assert!(round.enter(SELVI, false).is_ok());
}

#[test]
fn reconnect_releases_the_round_and_keeps_what_was_already_paid() {
    let mut round = Round::started(true);
    assert!(round.catch(T0 + 100).paid);
    // A second cast is in flight when the connection drops.
    round.send(&cast_at(10.0), T0 + 20_000).unwrap();
    round.service.tick(PLAYER, T0 + 21_510).unwrap();
    assert_eq!(
        round.service.reset_all(),
        1,
        "Gameplay::reset_session drops the round"
    );
    assert!(round.service.is_empty());
    assert!(
        round.send(&reel(), T0 + 22_500).unwrap().is_none(),
        "no round, nothing paid"
    );
    assert_eq!(round.fish_held(), 1, "the first catch was paid and stays");
    assert_eq!(round.fisherman_xp(), 10);
    // After the reconnect a new trip starts from scratch.
    let offer = round.enter(REED, false).unwrap();
    assert_eq!(ids(&offer.replies), vec![(39, 16)]);
    assert!(
        round
            .service
            .phase(PLAYER)
            .is_some_and(|p| p == Phase::Idle)
    );
}

#[test]
fn straying_past_the_leash_or_leaving_the_zone_ends_the_round_for_free() {
    let mut round = Round::started(true);
    round.send(&cast_at(10.0), T0 + 10).unwrap();
    // Standing at the spot, ticks keep the round.
    assert!(
        round
            .service
            .supervise(PLAYER, 0, SPOT, T0 + 100)
            .unwrap()
            .is_some_and(|c| status_flags(&c.replies).is_empty())
    );
    assert_eq!(round.service.len(), 1);
    // 49 m away is still fine; 51 m ends it.
    let near = [SPOT[0] + 49.0, SPOT[1], SPOT[2]];
    round.service.supervise(PLAYER, 0, near, T0 + 200).unwrap();
    assert_eq!(round.service.len(), 1);
    let far = [SPOT[0] + 51.0, SPOT[1], SPOT[2]];
    let ended = round
        .service
        .supervise(PLAYER, 0, far, T0 + 300)
        .unwrap()
        .unwrap();
    assert_eq!(status_flags(&ended.replies), vec![1]);
    assert!(
        ended.replies.iter().any(|p| p.starts_with(&[39, 0, 19])),
        "state removal"
    );
    assert!(round.service.is_empty());
    assert_eq!(round.fish_held(), 0);

    // A zone change ends it too.
    let mut round = Round::started(true);
    let ended = round
        .service
        .supervise(PLAYER, 3, SPOT, T0 + 100)
        .unwrap()
        .unwrap();
    assert_eq!(status_flags(&ended.replies), vec![1]);
    assert!(round.service.is_empty());
    // NaN positions never keep a round alive.
    let mut round = Round::started(true);
    let nan = [f32::NAN, 0.0, 0.0];
    round.service.supervise(PLAYER, 0, nan, T0 + 100).unwrap();
    assert!(round.service.is_empty());
}

#[test]
fn the_bite_timers_run_from_supervise() {
    let mut round = Round::started(true);
    round.send(&cast_at(10.0), T0 + 10).unwrap();
    let bite = round
        .service
        .supervise(PLAYER, 0, SPOT, T0 + 1_510)
        .unwrap()
        .unwrap();
    assert_eq!(ids(&bite.replies), vec![(138, 10)]);
    assert_eq!(round.service.phase(PLAYER), Some(Phase::Hook));
    assert_eq!(
        round
            .service
            .supervise(OTHER_PLAYER, 0, SPOT, T0)
            .unwrap()
            .map(|_| ()),
        None
    );
}

// ---- gateway proof (private bundle) -------------------------------------------------------

mod gateway_route {
    use super::fixtures::{copy_tree, fish_option, npc, wrap};
    use super::*;
    use osfr_tools::{account_store::AccountRole, gateway::Gateway};

    /// Reed Stillwater: the Sacred Grove Shallows guide (content NPC, Stillwater Crossing).
    const GATE_POSITION: [f32; 3] = [-1319.765, -65.62, 898.9357];
    /// Anna Mudskipper stands beside Reed but is not a guide.
    const NOT_A_GATE: u64 = 9_000_000_010;
    const NOW: u64 = 10_000;

    fn login() -> Vec<u8> {
        super::fixtures::login("w160-private-test")
    }
    fn chat(text: &str) -> Vec<u8> {
        let mut b = vec![15, 0, 1, 0, 0, 0];
        b.extend(1u64.to_le_bytes());
        b.extend(0u64.to_le_bytes());
        b.extend([0; 40]);
        b.extend((text.len() as u32).to_le_bytes());
        b.extend(text.bytes());
        b.extend([0; 28]);
        b
    }
    fn send_at(g: &mut Gateway, packet: &[u8], now: u64) -> Vec<Vec<u8>> {
        let out = g.handle_at(&wrap(packet), now).unwrap();
        out.replies.iter().map(|p| p[7..].to_vec()).collect()
    }
    fn tick(g: &mut Gateway, now: u64) -> Vec<Vec<u8>> {
        let out = g.tick_at(now).unwrap();
        out.replies.iter().map(|p| p[7..].to_vec()).collect()
    }
    fn fish_items(g: &Gateway) -> u32 {
        let ids = bound_item_ids();
        g.snapshot()
            .gameplay
            .unwrap()
            .inventory
            .items
            .iter()
            .filter(|item| ids.contains(&item.definition_id))
            .map(|item| item.count)
            .sum()
    }

    #[test]
    #[ignore = "requires OSFR_FISHING_BUNDLE (a playtest bundle), OSFR_GATEWAY_STARTUP_SNAPSHOT=<bundle>/startup/startup.json and the gameplay/npc_interaction patch"]
    fn private_ordinary_player_fishes_from_the_guide_menu() {
        let bundle = PathBuf::from(std::env::var_os("OSFR_FISHING_BUNDLE").unwrap());
        let content = std::env::temp_dir().join(format!("fishing-bundle-{}", std::process::id()));
        let _ = fs::remove_dir_all(&content);
        copy_tree(&bundle.join("content"), &content);
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(content.clone());
        let reference = bundle.join("reference");
        let connect = |with_spots: bool| {
            let path = content.join("fishing_spots.json");
            let _ = fs::remove_file(&path);
            if with_spots {
                fs::write(&path, SPOTS_JSON).unwrap();
            }
            let mut character = Gateway::reference_state(&reference).unwrap();
            character.position = GATE_POSITION;
            let mut g = Gateway::load_gameplay(&reference, character, &content, None).unwrap();
            g.set_dev_tools(false);
            g.set_role(AccountRole::Player);
            for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
                g.handle_at(&packet, NOW).unwrap();
            }
            g
        };
        let label = spots().menu_string_id();
        let menu = |g: &mut Gateway, target: u64| send_at(g, &npc(8, target, None), NOW);
        let offered = |g: &mut Gateway, target: u64| fish_option(&menu(g, target), label);
        let offers = |packets: &[Vec<u8>]| packets.iter().any(|p| p.starts_with(&[39, 0, 16]));

        // 1. No spots file: nobody offers fishing, and a forged choice starts nothing.
        let mut g = connect(false);
        assert!(offered(&mut g, REED).is_none());
        for forged in [2, 3] {
            let replies = send_at(&mut g, &npc(10, REED, Some(forged)), NOW);
            assert!(!offers(&replies), "no file, forged choice {forged}");
        }

        // 2. With the file only the listed guide offers it.
        let mut g = connect(true);
        let option = offered(&mut g, REED).expect("Reed must list the fishing trip");
        assert!(offered(&mut g, NOT_A_GATE).is_none(), "other NPCs must not");

        // 3. Forged choices start nothing: ids that were never offered and a menu for another NPC.
        menu(&mut g, REED);
        for forged in [0, 9, 99, -1, i32::MAX, option + 1] {
            let replies = send_at(&mut g, &npc(10, REED, Some(forged)), NOW);
            assert!(!offers(&replies), "forged choice {forged}");
            menu(&mut g, REED);
        }
        let replies = send_at(&mut g, &npc(10, NOT_A_GATE, Some(option)), NOW);
        assert!(!offers(&replies), "a choice naming another NPC");
        // `realm` commands stay developer-only.
        send_at(&mut g, &chat("realm race"), NOW);

        // 4. The real choice offers the round (39/16), and is single-use: replaying it
        //    after the menu closed starts nothing more.
        menu(&mut g, REED);
        let replies = send_at(&mut g, &npc(10, REED, Some(option)), NOW);
        let offer = replies
            .iter()
            .find(|p| p.starts_with(&[39, 0, 16]))
            .expect("the guide menu choice must offer the round");
        let instance = i32::from_le_bytes(offer[3..7].try_into().unwrap());
        let replay = send_at(&mut g, &npc(10, REED, Some(option)), NOW);
        assert!(!offers(&replay), "a replayed choice");
        // A second trip while one is open is refused with a chat line, not a second offer.
        menu(&mut g, REED);
        let again = send_at(&mut g, &npc(10, REED, Some(option)), NOW);
        assert!(!offers(&again), "no second offer while one is open");

        // The character may already hold items that share an id with a fish: count the change.
        let base = fish_items(&g);

        // 5. The client's Go: 41/2 and the fishing flag.
        let go = send_at(&mut g, &lifecycle(5, instance), NOW);
        assert_eq!(ids(&go), vec![(41, 2), (35, 20)]);
        assert_eq!(status_flags(&go), vec![0x40_0001]);

        // 6. Register, cast, bite, hook, reel: one catch, paid once.
        let registered = send_at(&mut g, &register(), NOW);
        assert_eq!(ids(&registered)[0], (138, 3));
        // The cast aims relative to the player's position (the spot anchor).
        let mut body = handle_bytes(ACTOR);
        for coordinate in [
            GATE_POSITION[0] + 10.0,
            GATE_POSITION[1],
            GATE_POSITION[2],
            1.0f32,
        ] {
            body.extend(coordinate.to_le_bytes());
        }
        body.push(0);
        let cast = send_at(&mut g, &request(6, &body), NOW + 10);
        assert_eq!(ids(&cast), vec![(138, 8)]);
        let bite = tick(&mut g, NOW + 1_600);
        assert!(
            ids(&bite).contains(&(138, 10)),
            "bite timer from the tick: {:?}",
            ids(&bite)
        );
        let _ = tick(&mut g, NOW + 2_600);
        assert_eq!(fish_items(&g), base);
        let caught = send_at(&mut g, &reel(), NOW + 2_700);
        assert_eq!(ids(&caught)[0], (138, 14), "{:?}", ids(&caught));
        assert_eq!(fish_items(&g), base + 1, "one fish, paid by the server");
        let job = g.snapshot().gameplay.unwrap().profiles.unwrap();
        assert!(
            job.unlocked.contains(&FISHERMAN_PROFILE),
            "(R) first trip unlocks the job"
        );
        assert_eq!(
            job.experience.get(&FISHERMAN_PROFILE),
            Some(&10),
            "(R) 10 XP per catch"
        );
        assert_ne!(
            job.active, FISHERMAN_PROFILE,
            "fishing leaves the active job alone"
        );
        // Replayed and forged catches pay nothing.
        for at in [NOW + 2_710, NOW + 9_000] {
            let _ = g.handle_at(&wrap(&reel()), at);
            assert_eq!(fish_items(&g), base + 1, "a replayed reel");
        }
        let mut forged = request(14, &handle_bytes(ACTOR));
        forged.extend(5u32.to_le_bytes());
        let _ = g.handle_at(&wrap(&forged), NOW + 3_000);
        assert_eq!(fish_items(&g), base + 1, "a forged result");

        // 7. Quit: the round ends, the flag goes off, nothing more is paid.
        let quit = send_at(&mut g, &encounter_close(instance), NOW + 4_000);
        assert_eq!(status_flags(&quit), vec![1]);
        let _ = g.handle_at(&wrap(&reel()), NOW + 4_100);
        assert_eq!(fish_items(&g), base + 1);

        // 8. Disconnect mid-attempt, then reconnect: released, the earlier catch stays, and a
        //    new trip is offered fresh.
        menu(&mut g, REED);
        let replies = send_at(&mut g, &npc(10, REED, Some(option)), NOW + 5_000);
        let offer = replies
            .iter()
            .find(|p| p.starts_with(&[39, 0, 16]))
            .unwrap();
        let instance = i32::from_le_bytes(offer[3..7].try_into().unwrap());
        send_at(&mut g, &lifecycle(5, instance), NOW + 5_100);
        send_at(&mut g, &register(), NOW + 5_100);
        send_at(&mut g, &request(6, &body), NOW + 5_200);
        g.reset_session();
        assert_eq!(fish_items(&g), base + 1, "the dropped cast paid nothing");
        for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
            g.handle_at(&packet, NOW + 6_000).unwrap();
        }
        let _ = g.handle_at(&wrap(&reel()), NOW + 8_000);
        assert_eq!(
            fish_items(&g),
            base + 1,
            "a reel after the reconnect finds no round"
        );
        menu(&mut g, REED);
        let replies = send_at(&mut g, &npc(10, REED, Some(option)), NOW + 9_000);
        assert!(
            offers(&replies),
            "after the reconnect the player can fish again"
        );
    }
}
